//! The station's water reserve (`docs/plans/post-roadmap-shared-room-humidity.md` §13): the
//! top-up makes a short watering whole and nothing more, the drain takes exactly the plants'
//! extra, and the separate-air lab, which waters by its own rule, never carries either.

use domains::biosphere::science::transpirable_capacity;
use domains::biosphere::stocks::{CONDENSATE, ROOTED_DEPTH, SOIL_WATER, SUBSOIL_WATER};
use domains::crew::WATER_STORE;
use domains::params;
use simcore::environment::SourceResolver;
use simcore::flow::FlowResult;
use simcore::registry::Registry;
use simcore::state::State;
use station::air_split::{split_scenario, AirSplit};
use station::gas_exchange::GasExchangeStep;
use station::params as sp;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{build_sealed_station, sealed_bio_resolver};
use station::water::RECOVERED_WATER;
use station::water_reserve::{working_level, WaterReserve, RESERVE_TOP_UP, TANK_DRAIN};

const WATERING: &str = "biosphere.recycling";

fn sealed(reserve: WaterReserve) -> (SealedStationScenario, State, Registry, Registry) {
    let sc = SealedStationScenario {
        water_reserve: reserve,
        ..sealed_station_scenario()
    };
    let (state, bio, fast) = build_sealed_station(
        &params::charge(),
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &sp::water_recovery(),
        &sp::lamp(),
        &sp::harvest(),
        &sc,
        false,
        false,
    )
    .expect("build sealed");
    (sc, state, bio, fast)
}

fn ids(reg: &Registry) -> Vec<String> {
    reg.flows().iter().map(|f| f.id().to_string()).collect()
}

fn leg(res: &FlowResult, stock: &str) -> f64 {
    res.legs
        .iter()
        .filter(|l| l.stock == stock)
        .map(|l| l.amount)
        .sum()
}

fn with(state: &State, stock: &str, kg: f64) -> State {
    let mut s = state.clone();
    let st = s.stocks[stock].with_amount(kg).expect("amount");
    s.stocks.insert(stock.to_string(), st);
    s
}

/// The reserve adds exactly two flows, one to each step, and moves nothing else in either registry.
#[test]
fn the_reserve_adds_the_top_up_and_the_drain_only() {
    let (_, _, bio_off, fast_off) = sealed(WaterReserve::Off);
    let (_, _, bio_on, fast_on) = sealed(WaterReserve::CrewStore);
    let (mut bio, mut fast) = (ids(&bio_off), ids(&fast_off));
    bio.push(RESERVE_TOP_UP.to_string());
    fast.push(TANK_DRAIN.to_string());
    bio.sort();
    fast.sort();
    let (mut got_bio, mut got_fast) = (ids(&bio_on), ids(&fast_on));
    got_bio.sort();
    got_fast.sort();
    assert_eq!(got_bio, bio);
    assert_eq!(got_fast, fast);
}

/// A watering at the trigger: with the plants' tank short, their own watering and the top-up
/// together deliver exactly the whole watering — the deficit to the root zone, the drip loss
/// below it — and the crew's tank pays only the shortfall. With the tank holding enough, the
/// top-up gives no legs. Off the trigger, neither waters.
#[test]
fn the_top_up_makes_a_short_watering_whole_and_nothing_more() {
    let (sc, state, bio, _) = sealed(WaterReserve::CrewStore);
    let p = sc.plant_params();
    let mut dry = state.clone();
    dry.aux.insert(ROOTED_DEPTH.to_string(), 1.0);
    let capacity = transpirable_capacity(1.0, sc.bio.soil_extractable_water, sc.bio.ground_area);
    let dry = with(&dry, SOIL_WATER, 0.2 * capacity);
    let deficit = 0.8 * capacity;
    let gross = deficit / p.water.drip_efficiency;
    let resolver = sealed_bio_resolver(&sp::lamp(), &sc).expect("bio resolver");
    let eval = |s: &State, id: &str| {
        let env = resolver.bind(s, sc.bio_dt);
        bio.flows()
            .iter()
            .find(|f| f.id() == id)
            .expect("flow")
            .evaluate(s, &env, sc.bio_dt)
            .expect("evaluate")
    };
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-12 * b.abs().max(1.0);

    let short = with(&dry, CONDENSATE, 10.0);
    let (own, top) = (eval(&short, WATERING), eval(&short, RESERVE_TOP_UP));
    assert!(close(leg(&own, CONDENSATE), -10.0));
    assert!(close(leg(&top, WATER_STORE), -(gross - 10.0)));
    assert_eq!(leg(&top, CONDENSATE), 0.0);
    assert!(close(
        leg(&own, SOIL_WATER) + leg(&top, SOIL_WATER),
        deficit
    ));
    assert!(close(
        leg(&own, SUBSOIL_WATER) + leg(&top, SUBSOIL_WATER),
        gross - deficit
    ));

    let full = with(&dry, CONDENSATE, gross + 1.0);
    assert!(eval(&full, RESERVE_TOP_UP).legs.is_empty());
    let wet = with(&short, SOIL_WATER, 0.9 * capacity);
    assert!(eval(&wet, RESERVE_TOP_UP).legs.is_empty());
    assert!(eval(&wet, WATERING).legs.is_empty());
}

/// The drain takes everything above one full watering, to the crew's recycler, and nothing at or
/// below it.
#[test]
fn the_drain_takes_exactly_the_extra() {
    let (sc, state, _, fast) = sealed(WaterReserve::CrewStore);
    let level = working_level(&sc.bio, &sc.plant_params());
    let drain = fast
        .flows()
        .iter()
        .find(|f| f.id() == TANK_DRAIN)
        .expect("the drain");
    let eval = |kg: f64| {
        let s = with(&state, CONDENSATE, kg);
        drain
            .evaluate(
                &s,
                &SourceResolver::empty().bind(&s, sc.cabin_dt),
                sc.cabin_dt,
            )
            .expect("evaluate")
    };
    let over = eval(level + 7.0);
    assert!((leg(&over, CONDENSATE) + 7.0).abs() <= 1e-9);
    assert_eq!(leg(&over, RECOVERED_WATER), -leg(&over, CONDENSATE));
    assert!(eval(level).legs.is_empty());
    assert!(eval(level - 7.0).legs.is_empty());
}

/// The separate-air lab waters from the crew's store by its own rule; a reserve would water twice,
/// so its scenario never carries one.
#[test]
fn separate_air_carries_no_reserve() {
    let sc = SealedStationScenario {
        water_reserve: WaterReserve::CrewStore,
        ..sealed_station_scenario()
    };
    let split = split_scenario(
        &sc,
        &AirSplit {
            chamber_air_mol: station::air_split::BVAD_CHAMBER_AIR_MOL,
            fan_mol_per_s: 1.0,
            vapour_crosses: true,
            gas_exchange: GasExchangeStep::Minute,
            watering: true,
            transpiration: GasExchangeStep::PlantStep,
        },
    );
    assert_eq!(split.water_reserve, WaterReserve::Off);
}
