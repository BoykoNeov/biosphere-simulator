//! The big station, Step 1 — the sealed station at 200 m² of crop and four crew, measured
//! (`docs/plans/post-roadmap-big-station.md` §3). A lab: it writes nothing and takes no decision.
//!
//! ```text
//! cargo run --release -q -p station --example big_station                 # 1 m² and 200 m², 1220 days
//! cargo run --release -q -p station --example big_station -- --days N
//! cargo run --release -q -p station --example big_station -- --area A     # another crop area (m²)
//! cargo run --release -q -p station --example big_station -- --control    # the a = 1 golden check
//! cargo run --release -q -p station --example big_station -- --crew N     # N crew in the big run
//! cargo run --release -q -p station --example big_station -- --co2-store   # Step 2a (below)
//! cargo run --release -q -p station --example big_station -- --food-loop   # Step 2b (below)
//! cargo run --release -q -p station --example big_station -- --nitrogen    # Step 2c (below)
//! cargo run --release -q -p station --example big_station -- --real-soil   # Step 3a (below)
//! cargo run --release -q -p station --example big_station -- --feed-crew   # Step 3c (below)
//! cargo run --release -q -p station --example big_station -- --co2-charge C --o2-charge O  # Step 4a
//! cargo run --release -q -p station --example big_station -- --co2-charge C --o2-charge O \
//!     --fertilizer KG --soil-n-level G                                                  # Step 4b
//! cargo run --release -q -p station --example big_station -- --co2-charge C --o2-charge O \
//!     --fertilizer KG --soil-n-level G --urine-any-time                                 # Step 4b-2
//! cargo run --release -q -p station --example big_station -- --co2-charge C --o2-charge O \
//!     --fertilizer KG --soil-n-level G --urine-any-time --burn-wastes \
//!     --burn-char-share F --burn-n-lost F --food-reserve-days D                          # Step 4c
//! cargo run --release -q -p station --example big_station -- <Step 4c's flags> //!     --marsh-area A --marsh-growth G --marsh-plant-n F --marsh-residence-days D //!     --marsh-plants-to soil|burner                                                     # Step 4d-1
//! cargo run --release -q -p station --example big_station -- <Step 4d-1's flags> --leaching  # 4d-2
//! ```
//!
//! `--co2-store` (Step 2a, plan §5): the big run keeps the crew's CO₂ for the plants, after CEEF
//! (Tako et al.): the scrubber's outlet is a CO₂ store ([`CO2_STORE`]) instead of
//! `boundary.co2_removed`, and an injector ([`Co2Injector`]) puts it back into the room while the
//! lamp is on, up to the scrubber's own target. The 1 m² run stays the reference, as the yardstick.
//!
//! `--food-loop` (Step 2b, plan §6; turns the store on too): the crop feeds the crew. At maturity
//! the grain less the next crop's seed goes to the crew's food store ([`harvest_at_maturity`]),
//! then the usual re-sow puts the straw, leaves and roots into the soil; the crew's feces go to the
//! soil too (`close_feces`).
//!
//! `--nitrogen` (Step 2c, plan §7; turns the food loop on too): the food carries nitrogen. The
//! imported food at BVAD's menu nitrogen ([`MENU_N_KG_PER_CREW_DAY`]), the grain at USDA's
//! ([`GRAIN_N_PER_DM`]) taken from the plant at harvest ([`harvest_at_maturity`]), and the crew pass
//! what they eat to the soil ([`CrewNitrogen`]): urine to its available nitrogen, feces to its litter.
//!
//! `--real-soil` (Step 3a, plan §8; turns the nitrogen on too): a soil of real size, after [F]
//! (Soltani & Sinclair) Ch. 18. The soil's solution nitrogen at sowing from a silt loam's routine data
//! ([`real_soil_n`], grams per m² instead of 100 kg), and the roots' reach as the nitrogen dissolved in
//! the plant-available water ([`SolutionUptake`]) instead of the 1–50 kg/m² availability band.
//!
//! `--feed-crew` (Step 3c, plan §9; turns the real soil on too, and the area to 700 m² unless
//! `--area` is given): a farm that feeds the crew. The imported food is a reserve
//! ([`FOOD_RESERVE`]) moved into the food store only when it would run short ([`FoodTopUp`]); the
//! crew's urine nitrogen goes to a tank ([`URINE_TANK`]), dosed to the soil at the roots' uptake
//! from sowing to two-thirds of the season ([`UrineDose`], EcoSanRes Fact Sheet 6).
//!
//! `--co2-charge MOL --o2-charge MOL` (Step 4a, plan §10.1; turns `--feed-crew` on too): gas tanks
//! charged at the start and refilled only from the room's air, after CEEF. The CO₂ store starts at
//! its charge, and the room's O₂ makeup draws from and returns to a tank ([`O2Tank`]) with a floor,
//! in place of the outside supply. Both charges are inputs, with no default.
//!
//! `--fertilizer KG --soil-n-level G` (Step 4b, plan §10.3; needs Step 4a's charges): the dose tops
//! the soil's solution nitrogen up to `G` g N/m² each plant step, from the urine tank inside its
//! window and from a fertilizer store ([`FERTILIZER`], starting at `KG` kg N) for the rest, at any
//! time from sowing to maturity (Linquist et al. 1992). Both are inputs, with no default.
//!
//! `--urine-any-time` (Step 4b-2, plan §10.5; needs Step 4b's two inputs): the urine tank may dose
//! at any time from sowing to maturity, not only inside EcoSanRes' window — a hygiene rule for crops
//! eaten raw or soon, set aside for threshed and cooked wheat (a reading, not a citation). Day 93
//! stays the label every readout splits on.
//!
//! `--burn-wastes --burn-char-share F --burn-n-lost F --food-reserve-days D` (Step 4c, plan §10.7;
//! needs Step 4b-2): CEEF's waste processing (Tako et al.). The straw (leaves, stems, stem reserve;
//! the roots stay in the soil), the crew's feces and, at each harvest, the food store above `D`
//! days of the crew's food go to a burner ([`Burner`]) that returns carbon × (1 − char share) to the
//! CO₂ tank, one O₂ from the O₂ tank for each, and sends the nitrogen × lost to N₂ (BVAD Eq. 4-1),
//! the rest to the urine tank. All three are inputs, with no default. `--burn-co2-leak F` (plan
//! §10.9; optional, absent = leak-free) sends carbon × F of the burned carbon outside as CO₂, taken
//! from the tank's share, its O₂ still drawn.
//!
//! `--marsh-area A --marsh-growth G --marsh-plant-n F --marsh-residence-days D --marsh-plants-to
//! soil|burner` (Step 4d-1, plan §10.11; needs Step 4c): Biosphere 2's wastewater marsh (Nelson et al.
//! 1999). The crew's urine nitrogen goes to the marsh's water ([`MARSH_N`]) instead of the urine
//! tank; [`Marsh`] sends it on to the tank over the residence time `D` days, and grows marsh plants
//! on `A` m² at `G` g dry mass/m²/day with `F` of it nitrogen, their carbon from the room's CO₂, cut as
//! they grow to the farm soil's litter or the burner's feed. All five are inputs, with no default.
//!
//! `--leaching` (Step 4d-2, plan §10.13; needs Step 4d-1): the drained water carries its nitrogen.
//! The flows that move water into, out of or below the root zone are wrapped
//! ([`CarriesNitrogen`]): the watering's share past the roots leaches the soil solution by [F]
//! Eqn 18.14 into [`SUBSOIL_N`], the roots' capture and the overflow carry the subsoil's nitrogen
//! back to the soil and on to the plants' tank ([`PLANT_TANK_N`]), and the watering brings the
//! tank's back.
//!
//! `--crew N` (the attribution check): the big run with N crew, their food and water stores ×
//! N / 4. At `N = 4a` every m² gets the 1 m² run's CO₂, so a crop per m² back at the 1 m² run's
//! says the shortfall at four crew is the CO₂ and nothing else the sizing moved.
//!
//! ```text
//! ```
//!
//! **The sizing** ([`sized`]): everything the reference sizes per m² of crop is multiplied by the
//! area `a` — the crop's and soil's starting amounts and the plants' water tank (its start is the
//! working level, which follows the area), the lamp, the chamber's heat capacity, cooler and
//! heater, and its walls by the reference's own box rule (a `√a × √a` floor, 0.78 m tall: walls,
//! floor and roof `2a + 4·0.78·√a`, so 5.12 m² at 1 m² and 444 m² at 200 m²). The crew (four, the
//! reference), their stores, the room's air (9 500 mol) and its CO₂ and O₂ targets are left as
//! they are. At `a = 1` [`sized`] changes nothing: `--control` runs it through the reference's
//! driver and checks the committed `sealed_station_state.json` byte for byte.
//!
//! **Two stand-ins, so that neither binds** ([`outside_supply`]), recorded as stand-ins:
//! * *Outside electricity*: a lab flow, [`GridSupply`], that tops the station's battery back up
//!   to its starting charge every minute from an outside source, `boundary.grid`. The
//!   electricity drawn is what the grid gave plus what the battery lost.
//!   ⚠ Not a battery of 1e14 J (tried first): the engine's every-step conservation check allows
//!   1e-9 of the step's largest transfer, and a stock that large rounds by ~0.016 J — the 1 m²
//!   run failed on day one. The same rounding bounds every stock that keeps a running total,
//!   the boundaries included: at 200 m² the heat sent to space reaches ~3e12 J over 1220 days.
//! * *Heat rejection*: the station's radiator and its structure's heat capacity, ×a. The chamber's
//!   cooler hands its heat to the structure and can do so only while the structure is colder
//!   than the room (no heat pump is modelled); with the 1 m² radiator the structure would run
//!   far above the room and the cooler would stop. The readouts count the minutes that happens.
//!
//! **The readouts.** Closure share: the ecosystem's net CO₂ draw (the crew's exhalation less what
//! the scrubber removed less the room's own change) over the crew's exhalation, per crop cycle;
//! crop per m²; the room's CO₂ and O₂; water; electricity; the chamber's and the structure's
//! temperature, the cooler's blocked and capped minutes and the heater's capped minutes; and
//! rationing. ⚠ `rationed == 0` cannot measure starvation, so it is reported, never the verdict.

use domains::biosphere::flows::NitrogenUptake;
use domains::biosphere::science::{nitrogen_stress_factor, root_zone_fraction, WATER_DENSITY};
use domains::biosphere::stocks::{
    CARBON_POOL, CONDENSATE, CONSUMER_CARBON, HUMUS_CARBON, HUMUS_N, LEAF_C, LITTER_CARBON,
    LITTER_N, LITTER_SINK, MICROBIAL_CARBON, MICROBIAL_N, O2_POOL, PLANT_N, ROOTED_DEPTH, ROOT_C,
    SOIL_N, SOIL_WATER, STEM_C, STEM_RESERVE_C, STORAGE_C, SUBSOIL_WATER, WATER_VAPOR,
};
use std::cell::RefCell;
use std::rc::Rc;

use domains::biosphere::system::SeasonScenario;
use domains::crew::{FOOD_INTAKE_VAR, FOOD_STORE, WATER_STORE};
use domains::eclss::{CO2Scrubber, O2Makeup, CO2_REMOVED, CO2_SCRUBBER, O2_MAKEUP, O2_SUPPLY};
use domains::params;
use domains::power::BATTERY;
use domains::thermal::{temperature, ThermalParams, NODE};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{
    chamber_temperature, ChamberParams, CHAMBER, CHAMBER_COOLING, CHAMBER_HEATER,
};
use station::driver::{DayOrder, OwnedResetHook, Side, TwoRate};
use station::flows::{CREW_RESPIRATION, LAMP_POWER_VAR};
use station::params as sp;
use station::scenario::{sealed_station_scenario, SealedStationScenario};
use station::sealed::{
    build_sealed_station, is_mature, run_sealed, sealed_bio_resolver, sealed_fast_resolver,
    sealed_reset_hook,
};
use station::sowing::twin;
use station::stocks::{co2_composition, gas_boundary, gas_pool, o2_composition, simple_pool};
use station::water_reserve::{working_level, RESERVE_TOP_UP, TANK_DRAIN};

/// The chamber's height (m) in `chamber.yaml`'s wall-area rule: BVAD's shoot + root zone.
const CHAMBER_HEIGHT: f64 = 0.78;
/// The outside supply [`GridSupply`] draws from.
const GRID: &str = "boundary.grid";
/// The lab's outside-electricity flow id.
const GRID_SUPPLY: &str = "lab.grid_supply";
/// Step 2a's CO₂ store: the tank the scrubber fills (CEEF's reservoir).
const CO2_STORE: &str = "lab.co2_store";
/// Step 2a's injector flow id (CEEF's mass-flow controller).
const CO2_INJECTOR: &str = "lab.co2_injector";

/// CARBON (as CO₂) `lab.co2_store → the room`: while the lamp draws power, `rate · (target −
/// room CO₂)` per second, no more than the store holds. The scrubber's own target and gain, so
/// the scrubber acts only above the target and the injector only below it. The gain is DESIGN
/// (CEEF's mass-flow controller is not published as a rate). Off when the lamp is: CEEF held the
/// plants' CO₂ constant in the light periods.
struct Co2Injector {
    target: f64,
    rate: f64,
    sc: SealedStationScenario,
}

impl Co2Injector {
    /// What the injector would give from `room` with the lamp at `lamp` W, before the store's cap.
    fn wanted(&self, room: f64, lamp: f64, dt: f64) -> f64 {
        if lamp > 0.0 {
            self.rate * (self.target - room).max(0.0) * dt
        } else {
            0.0
        }
    }
}

impl Flow for Co2Injector {
    fn type_name(&self) -> &'static str {
        "Co2Injector"
    }

    fn id(&self) -> &str {
        CO2_INJECTOR
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        // ⚠ The plain lamp variable is in no sealed resolver: a flow added after the build reads
        // its phase twin itself (the sowing clock wraps only the build's own flows).
        let lamp = env.get(&twin(LAMP_POWER_VAR, self.sc.phase(snapshot)?))?;
        let given = self
            .wanted(amount(snapshot, CARBON_POOL), lamp, dt)
            .min(amount(snapshot, CO2_STORE));
        FlowResult::new(vec![
            Leg::new(CO2_STORE.to_string(), -given)?,
            Leg::new(CARBON_POOL.to_string(), given)?,
        ])
    }
}

/// Step 4a's O₂ tank: CEEF's O₂ separated from the plants' air and kept. The room's makeup draws
/// from it and its surplus goes back into it; nothing comes from outside.
const O2_TANK: &str = "lab.o2_tank";

/// OXYGEN `lab.o2_tank ⇄ the room`, Step 4a: the reference's two-sided makeup ([`O2Makeup`], its
/// gain and setpoint) re-pointed at [`O2_TANK`], giving no more than the tank holds.
struct O2Tank {
    inner: O2Makeup,
}

impl Flow for O2Tank {
    fn type_name(&self) -> &'static str {
        "O2Tank"
    }

    fn id(&self) -> &str {
        O2_MAKEUP
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let given = leg(&self.inner.evaluate(snapshot, env, dt)?, O2_POOL)
            .min(amount(snapshot, O2_TANK));
        FlowResult::new(vec![
            Leg::new(O2_TANK.to_string(), -given)?,
            Leg::new(O2_POOL.to_string(), given)?,
        ])
    }
}

/// Step 2c's food nitrogen (kg N): the nitrogen in the crew's food store.
const FOOD_N: &str = "lab.food_n";
/// Step 2c's crew nitrogen flow id.
const CREW_NITROGEN: &str = "lab.crew_nitrogen";
/// The imported food's nitrogen per crew-day (kg N): BVAD Table 4-61, resupplied menus at 12.71
/// MJ/CM-d, 122 g protein per crew-day (the lowest of three menus, 122–125) ÷ 6.25. BVAD's caveat:
/// the table compares menus; it is not absolute values.
const MENU_N_KG_PER_CREW_DAY: f64 = 0.122 / 6.25;
/// Wheat grain nitrogen, kg N per kg dry mass: USDA FoodData Central 168890 (*Wheat, hard red
/// winter*, SR Legacy) — 12.61 g protein and 13.1 g water per 100 g, protein = N × 5.83.
const GRAIN_N_PER_DM: f64 = 12.61 / 5.83 / (100.0 - 13.1);
/// The share of excreted nitrogen in urine; the rest is in feces. Jönsson & Vinnerås (2004), as
/// tabulated in EcoSanRes Fact Sheet 6 (2005): urine 85–90 % of excreted N in every country listed
/// (China 3.5 of 4.0 kg/cap/yr, South Africa 3.0 of 3.4, Uganda 2.2 of 2.5).
const URINE_N_SHARE: f64 = 0.88;

/// NITROGEN `lab.food_n → soil_n (or the urine tank) + litter_n`: the crew eat nitrogen with their
/// carbon, at the food store's average N:C, and pass all of it (an adult is in nitrogen balance):
/// urine to `urine_to` (the soil's available nitrogen, or Step 3c's [`URINE_TANK`]), feces with
/// their carbon to `feces_to` (the litter, or Step 4c's [`BURNER_FEED_N`]).
struct CrewNitrogen {
    urine_to: &'static str,
    feces_to: &'static str,
}

impl Flow for CrewNitrogen {
    fn type_name(&self) -> &'static str {
        "CrewNitrogen"
    }

    fn id(&self) -> &str {
        CREW_NITROGEN
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let carbon = env.get(FOOD_INTAKE_VAR)? * dt;
        let eaten = carbon * amount(snapshot, FOOD_N) / amount(snapshot, FOOD_STORE);
        let urine = eaten * URINE_N_SHARE;
        FlowResult::new(vec![
            Leg::new(FOOD_N.to_string(), -eaten)?,
            Leg::new(self.urine_to.to_string(), urine)?,
            Leg::new(self.feces_to.to_string(), eaten - urine)?,
        ])
    }
}

/// Step 4c's burner feed (mol C) and its nitrogen (kg N): the straw, the crew's feces and the food
/// above the reserve, waiting for [`Burner`].
const BURNER_FEED: &str = "lab.burner_feed";
const BURNER_FEED_N: &str = "lab.burner_feed_n";
/// Step 4c's char (mol C): the carbon the processor keeps (Tako et al. Fig. 2(b), "Deposition").
const CHAR: &str = "lab.char";
/// Step 4c's nitrogen gas (kg N): the burned nitrogen as N₂ (BVAD Eq. 4-1), out of every plant's
/// reach while no legume is grown.
const N2_GAS: &str = "lab.n2_gas";
/// Step 4c's burner flow id.
const BURNER: &str = "lab.burner";
/// Step 4c's leaked CO₂ (mol C), outside the station: the burner's carbon that escapes as CO₂
/// before it reaches the tank (`--burn-co2-leak`; Tako et al. Fig. 2(b)'s "CO₂ leak", an
/// ESTIMATED remainder, plan §10.9).
const BURNER_LEAK: &str = "boundary.burner_co2_leak";

/// Step 4c's inputs, none with a default: the share of the burned carbon kept as char, the share of
/// the burned nitrogen lost as N₂ (the rest caught into the urine tank), and the food reserve in
/// days of the crew's food. The leak (`--burn-co2-leak`, plan §10.9) is the one optional input:
/// absent, the burner is leak-free (the DESIGN of §10.7) and builds exactly as before it existed.
#[derive(Debug, Clone, Copy)]
struct Burn {
    char_share: f64,
    n_lost: f64,
    reserve_days: f64,
    /// The share of the burned carbon that escapes as CO₂ to [`BURNER_LEAK`], not to the tank.
    co2_leak: Option<f64>,
}

/// CARBON + OXYGEN + NITROGEN `lab.burner_feed + lab.o2_tank → lab.co2_store + lab.char`, Step 4c:
/// CEEF's carbonizer and incinerator (Tako et al.). The whole feed each step (DESIGN: the burner
/// keeps up; its CO₂ goes to a tank, so its timing moves nothing the crop reads), capped by the O₂
/// tank. Carbon × (1 − char) to the CO₂ tank with one O₂ from the O₂ tank each (the model's
/// decomposer ratio), carbon × char kept; the feed's nitrogen burned in step with its carbon, × lost
/// to [`N2_GAS`], the rest to [`URINE_TANK`].
struct Burner {
    burn: Burn,
}

impl Flow for Burner {
    fn type_name(&self) -> &'static str {
        "Burner"
    }

    fn id(&self) -> &str {
        BURNER
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        _dt: f64,
    ) -> Result<FlowResult, SimError> {
        let Burn {
            char_share,
            n_lost,
            co2_leak,
            ..
        } = self.burn;
        let feed = amount(snapshot, BURNER_FEED);
        let o2 = amount(snapshot, O2_TANK);
        let burned = if char_share < 1.0 {
            feed.min(o2 / (1.0 - char_share))
        } else {
            feed
        };
        let share = if feed > 0.0 { burned / feed } else { 1.0 };
        let n = amount(snapshot, BURNER_FEED_N) * share;
        let co2 = burned * (1.0 - char_share);
        let mut legs = match co2_leak {
            // Every carbon burned to gas takes its O₂, the leaked as well as the returned.
            Some(leak) => {
                let leaked = burned * leak;
                vec![
                    Leg::new(BURNER_FEED.to_string(), -burned)?,
                    Leg::new(CO2_STORE.to_string(), co2 - leaked)?,
                    Leg::new(BURNER_LEAK.to_string(), leaked)?,
                    Leg::new(CHAR.to_string(), burned - co2)?,
                    Leg::new(O2_TANK.to_string(), -co2)?,
                ]
            }
            None => vec![
                Leg::new(BURNER_FEED.to_string(), -burned)?,
                Leg::new(CO2_STORE.to_string(), co2)?,
                Leg::new(CHAR.to_string(), burned - co2)?,
                Leg::new(O2_TANK.to_string(), -co2)?,
            ],
        };
        legs.extend([
            Leg::new(BURNER_FEED_N.to_string(), -n)?,
            Leg::new(N2_GAS.to_string(), n * n_lost)?,
            Leg::new(URINE_TANK.to_string(), n - n * n_lost)?,
        ]);
        FlowResult::new(legs)
    }
}

/// CARBON `crew.food_store → the room + lab.burner_feed`, Step 4c: the crew's own flow with its
/// fecal leg (the litter's since Step 2b) re-pointed at [`BURNER_FEED`].
struct FecesToBurner {
    inner: Box<dyn Flow>,
}

impl Flow for FecesToBurner {
    fn type_name(&self) -> &'static str {
        "FecesToBurner"
    }

    fn id(&self) -> &str {
        CREW_RESPIRATION
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let legs = self
            .inner
            .evaluate(snapshot, env, dt)?
            .legs
            .into_iter()
            .map(|l| {
                if l.stock == LITTER_CARBON {
                    Leg::new(BURNER_FEED.to_string(), l.amount)
                } else {
                    Ok(l)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        FlowResult::new(legs)
    }
}

/// The station with Step 4c's burner: its four stocks (empty), the crew's feces re-pointed at the
/// feed ([`FecesToBurner`]), and [`Burner`]. The straw and the food above the reserve reach the feed
/// at harvest ([`harvest_at_maturity`]).
fn with_burner(state: State, fast: Registry, burn: Burn) -> (State, Registry) {
    let mut stocks = state.stocks.clone();
    for s in [
        simple_pool(BURNER_FEED, "lab", Quantity::Carbon, 0.0),
        simple_pool(BURNER_FEED_N, "lab", Quantity::Nitrogen, 0.0),
        simple_pool(CHAR, "lab", Quantity::Carbon, 0.0),
        simple_pool(N2_GAS, "lab", Quantity::Nitrogen, 0.0),
    ] {
        let s = s.expect("Step 4c stock");
        stocks.insert(s.id.clone(), s);
    }
    if burn.co2_leak.is_some() {
        let s = gas_boundary(BURNER_LEAK, Quantity::Carbon, co2_composition(), false).expect("leak");
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (flows, aux) = fast.into_parts();
    let mut replaced = 0;
    let mut flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|f| -> Box<dyn Flow> {
            if f.id() == CREW_RESPIRATION {
                replaced += 1;
                Box::new(FecesToBurner { inner: f })
            } else {
                f
            }
        })
        .collect();
    assert_eq!(replaced, 1, "one crew respiration to re-point");
    flows.push(Box::new(Burner { burn }));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, fast)
}

/// Step 4d-1's marsh water nitrogen (kg N): the crew's urine on its way to the urine tank.
const MARSH_N: &str = "lab.marsh_n";
/// Step 4d-1's marsh flow id.
const MARSH: &str = "lab.marsh";
/// Biosphere 2's marsh lamps (Nelson et al. 1999, §3.2): six 1000 W sodium lamps over its 41.1 m²
/// (W/m²). A readout only — the marsh's lamp is not drawn from the grid nor heats the room (plan
/// §10.11, a named gap).
const B2_MARSH_LAMP_W_PER_M2: f64 = 6.0 * 1000.0 / 41.1;

/// Step 4d-1's inputs, none with a default (plan §10.11): the marsh's area (m²), its plants' growth
/// (g dry mass per m² per day; Biosphere 2's measured 40.5), their nitrogen (kg N per kg dry mass;
/// Reddy et al. 1985's 0.037), the marsh water's residence time (days; Biosphere 2's 4), and where
/// the cut plants go — the farm soil's litter, or the burner's feed (the user's switch).
#[derive(Debug, Clone, Copy)]
struct MarshOpts {
    area: f64,
    growth: f64,
    plant_n: f64,
    residence_days: f64,
    to_soil: bool,
}

/// CARBON + OXYGEN + NITROGEN, Step 4d-1: Biosphere 2's wastewater marsh (Nelson et al. 1999). The
/// crew's urine nitrogen arrives in [`MARSH_N`] ([`CrewNitrogen`] re-pointed). Each step:
/// * the outflow, the water's nitrogen × dt ÷ the residence time, goes on to [`URINE_TANK`] (Biosphere
///   2 sent its outflow to the crop irrigation);
/// * the plants grow at `growth` g DM/m²/day, harvested as they grow (no standing crop kept), taking
///   `plant_n` of their dry mass as nitrogen from what the outflow leaves — less if the water holds
///   less, the growth then what the nitrogen allows. Their carbon, dry mass ÷ the crop's own dry mass
///   per mol C (its carbon fraction, applied to marsh plants: a reading), comes from the room's CO₂
///   with one O₂ back to the room; carbon and nitrogen to the soil's litter or the burner's feed.
struct Marsh {
    opts: MarshOpts,
    dm_kg_per_mol_c: f64,
}

impl Flow for Marsh {
    fn type_name(&self) -> &'static str {
        "Marsh"
    }

    fn id(&self) -> &str {
        MARSH
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let o = self.opts;
        let water = amount(snapshot, MARSH_N);
        let out = water * (dt / (o.residence_days * 86_400.0)).min(1.0);
        let dm = o.area * o.growth * 1e-3 * dt / 86_400.0;
        let n = (dm * o.plant_n).min(water - out).max(0.0);
        let c = if o.plant_n > 0.0 { n / o.plant_n } else { dm } / self.dm_kg_per_mol_c;
        let (to_c, to_n) = if o.to_soil {
            (LITTER_CARBON, LITTER_N)
        } else {
            (BURNER_FEED, BURNER_FEED_N)
        };
        FlowResult::new(vec![
            Leg::new(CARBON_POOL.to_string(), -c)?,
            Leg::new(O2_POOL.to_string(), c)?,
            Leg::new(to_c.to_string(), c)?,
            Leg::new(MARSH_N.to_string(), -out - n)?,
            Leg::new(URINE_TANK.to_string(), out)?,
            Leg::new(to_n.to_string(), n)?,
        ])
    }
}

/// The station with Step 4d-1's marsh: [`MARSH_N`] (empty) and [`Marsh`]. The urine reaches it
/// through [`with_food_nitrogen`]'s `urine_to`.
fn with_marsh(
    sc: &SealedStationScenario,
    state: State,
    fast: Registry,
    opts: MarshOpts,
) -> (State, Registry) {
    let mut stocks = state.stocks.clone();
    let s = simple_pool(MARSH_N, "lab", Quantity::Nitrogen, 0.0).expect("marsh water");
    stocks.insert(s.id.clone(), s);
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (mut flows, aux) = fast.into_parts();
    flows.push(Box::new(Marsh {
        opts,
        dm_kg_per_mol_c: sc.plant_params().nitro.dm_kg_per_mol_c,
    }));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, fast)
}

/// Step 4d-2's nitrogen dissolved in the water below the roots, and in the plants' water tank (kg N).
const SUBSOIL_N: &str = "lab.subsoil_n";
const PLANT_TANK_N: &str = "lab.plant_tank_n";
/// The water flows [`CarriesNitrogen`] wraps, by id.
const WATERING: &str = "biosphere.recycling";
const DRAINAGE: &str = "biosphere.drainage";
const ROOT_ZONE_CAPTURE: &str = "biosphere.root_zone_capture";
const SUBSOIL_OVERFLOW: &str = "biosphere.subsoil_overflow";

/// What a wrapped water flow's water carries (Step 4d-2, plan §10.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Carry {
    /// Water onto the soil (the watering, the crew store's top-up) or past the roots (drainage):
    /// the share past the roots leaches the soil solution ([F] Eqn 18.14), and water drawn from
    /// the plants' tank brings the tank's nitrogen, split as the water splits.
    Down,
    /// Water from below the roots, to the root zone (the roots' capture) or the plants' tank (the
    /// overflow): the subsoil's nitrogen at its concentration.
    Subsoil,
    /// The plants' tank to the crew's recycler: the tank's nitrogen to the urine tank (the
    /// recycler's brine — a reading).
    TankDrain,
}

/// NITROGEN with WATER, Step 4d-2: a water flow whose own legs carry their dissolved nitrogen in
/// the same step — the water rule is the wrapped flow's, never a second copy. The soil solution is
/// the top layer's ([F]'s NSOL, the layer the roots' concentration reads), so the re-sow's return
/// of the abandoned root zone's water to the subsoil carries none.
struct CarriesNitrogen {
    inner: Box<dyn Flow>,
    carry: Carry,
    soil: SolutionUptake,
}

/// `n` × the share `out` of `held` moved, 0 when nothing is held or moved.
fn carried(n: f64, out: f64, held: f64) -> f64 {
    if out > 0.0 && held > 0.0 {
        n * (out / held).min(1.0)
    } else {
        0.0
    }
}

impl CarriesNitrogen {
    /// [F] Eqn 18.14, NLEACH = NSOL × DRAIN1 ÷ (WAT1 + DRAIN1), set to 0 when the concentration
    /// is at or below 1 mg/L (the book's listing: `If NCON <= 0.000001 Then NLEACH = 0`).
    fn leached(&self, s: &State, drain: f64) -> f64 {
        if drain <= 0.0 || self.soil.concentration(s) <= SOLUTION_N_FLOOR {
            return 0.0;
        }
        amount(s, SOIL_N) * drain / (self.soil.layer_water(s).1 + drain)
    }
}

fn add(n: &mut std::collections::BTreeMap<&'static str, f64>, id: &'static str, x: f64) {
    *n.entry(id).or_insert(0.0) += x;
}

impl Flow for CarriesNitrogen {
    fn type_name(&self) -> &'static str {
        "CarriesNitrogen"
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn priority(&self) -> i64 {
        self.inner.priority()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let res = self.inner.evaluate(snapshot, env, dt)?;
        let w = |id: &str| leg(&res, id);
        let mut n = std::collections::BTreeMap::new();
        match self.carry {
            Carry::Down => {
                let (to_soil, below) = (w(SOIL_WATER).max(0.0), w(SUBSOIL_WATER).max(0.0));
                let from_tank = carried(
                    amount(snapshot, PLANT_TANK_N),
                    -w(CONDENSATE),
                    amount(snapshot, CONDENSATE),
                );
                if from_tank > 0.0 && to_soil + below > 0.0 {
                    add(&mut n, PLANT_TANK_N, -from_tank);
                    add(&mut n, SOIL_N, from_tank * to_soil / (to_soil + below));
                    add(&mut n, SUBSOIL_N, from_tank * below / (to_soil + below));
                }
                let leached = self.leached(snapshot, below);
                add(&mut n, SOIL_N, -leached);
                add(&mut n, SUBSOIL_N, leached);
            }
            Carry::Subsoil => {
                let out = carried(
                    amount(snapshot, SUBSOIL_N),
                    -w(SUBSOIL_WATER),
                    amount(snapshot, SUBSOIL_WATER),
                );
                let (up, tank) = (w(SOIL_WATER).max(0.0), w(CONDENSATE).max(0.0));
                if out > 0.0 && up + tank > 0.0 {
                    add(&mut n, SUBSOIL_N, -out);
                    add(&mut n, SOIL_N, out * up / (up + tank));
                    add(&mut n, PLANT_TANK_N, out * tank / (up + tank));
                }
            }
            Carry::TankDrain => {
                let out = carried(
                    amount(snapshot, PLANT_TANK_N),
                    -w(CONDENSATE),
                    amount(snapshot, CONDENSATE),
                );
                add(&mut n, PLANT_TANK_N, -out);
                add(&mut n, URINE_TANK, out);
            }
        }
        let mut legs = res.legs;
        for (id, x) in n {
            if x != 0.0 {
                legs.push(Leg::new(id.to_string(), x)?);
            }
        }
        FlowResult::new(legs)
    }
}

/// The station with Step 4d-2's [`SUBSOIL_N`] and [`PLANT_TANK_N`] (both empty at the start), and
/// every flow that moves water into, out of or below the root zone wrapped in [`CarriesNitrogen`]:
/// the watering, the crew store's top-up and the drainage ([`Carry::Down`]), the roots' capture and
/// the overflow ([`Carry::Subsoil`]), the tank's drain ([`Carry::TankDrain`]). Each of the watering,
/// the capture and the overflow must be found once — a build that lost one would lose its nitrogen.
fn with_leaching(
    sc: &SealedStationScenario,
    state: State,
    bio: Registry,
    fast: Registry,
) -> (State, Registry, Registry) {
    let mut stocks = state.stocks.clone();
    for id in [SUBSOIL_N, PLANT_TANK_N] {
        let s = simple_pool(id, "lab", Quantity::Nitrogen, 0.0).expect("Step 4d-2 stock");
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let mut found = std::collections::BTreeMap::new();
    let mut wrap = |reg: Registry| -> Registry {
        let (flows, aux) = reg.into_parts();
        let flows = flows
            .into_iter()
            .map(|f| -> Box<dyn Flow> {
                let carry = match f.id() {
                    WATERING | RESERVE_TOP_UP | DRAINAGE => Carry::Down,
                    ROOT_ZONE_CAPTURE | SUBSOIL_OVERFLOW => Carry::Subsoil,
                    TANK_DRAIN => Carry::TankDrain,
                    _ => return f,
                };
                *found.entry(f.id().to_string()).or_insert(0) += 1;
                Box::new(CarriesNitrogen {
                    inner: f,
                    carry,
                    soil: SolutionUptake::new(sc),
                })
            })
            .collect();
        Registry::new(flows, &state.stocks, aux).expect("registry")
    };
    let (bio, fast) = (wrap(bio), wrap(fast));
    for id in [WATERING, ROOT_ZONE_CAPTURE, SUBSOIL_OVERFLOW] {
        assert_eq!(found.get(id), Some(&1), "Step 4d-2 wraps {id} exactly once");
    }
    (state, bio, fast)
}

/// Step 3c's imported food, held back (mol C) — and its nitrogen (kg N) at the menu's N:C.
const FOOD_RESERVE: &str = "lab.food_reserve";
const FOOD_RESERVE_N: &str = "lab.food_reserve_n";
/// Step 3c's top-up flow id.
const FOOD_TOP_UP: &str = "lab.food_top_up";
/// The food store's level (days of the crew's food) below which [`FoodTopUp`] takes from the
/// reserve. DESIGN: one day, so the crew eat grown food whenever there is any and the store never
/// runs short within a step.
const FOOD_BUFFER_DAYS: f64 = 1.0;
/// Step 3c's urine tank (kg N): the crew's urine nitrogen, kept for the soil.
const URINE_TANK: &str = "lab.urine_n";
/// Step 3c's dosing flow id.
const URINE_DOSE: &str = "lab.urine_dose";
/// The days after sowing the urine is dosed: EcoSanRes Fact Sheet 6, urine is "best utilized if
/// … applied prior to sowing and up until two-thirds of the period between sowing and harvest" —
/// two-thirds of 2b's measured 140-day crop (day 93, 47 days before the harvest; the sheet's "at
/// least one month prior to harvesting" holds).
const DOSE_WINDOW_DAYS: f64 = 140.0 * 2.0 / 3.0;

/// CARBON + NITROGEN `lab.food_reserve → crew.food_store`: when the store is below `level` (mol C),
/// the reserve fills it back to `level`, its nitrogen at the reserve's own N:C with it — so the
/// crew, who eat at the store's average, read the mix of grown and imported food truly.
struct FoodTopUp {
    level: f64,
}

impl Flow for FoodTopUp {
    fn type_name(&self) -> &'static str {
        "FoodTopUp"
    }

    fn id(&self) -> &str {
        FOOD_TOP_UP
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        _dt: f64,
    ) -> Result<FlowResult, SimError> {
        let reserve = amount(snapshot, FOOD_RESERVE);
        let carbon = (self.level - amount(snapshot, FOOD_STORE)).max(0.0).min(reserve);
        let n = if carbon > 0.0 {
            carbon * amount(snapshot, FOOD_RESERVE_N) / reserve
        } else {
            0.0
        };
        FlowResult::new(vec![
            Leg::new(FOOD_RESERVE.to_string(), -carbon)?,
            Leg::new(FOOD_STORE.to_string(), carbon)?,
            Leg::new(FOOD_RESERVE_N.to_string(), -n)?,
            Leg::new(FOOD_N.to_string(), n)?,
        ])
    }
}

/// Step 4b's fertilizer store (kg N): mineral nitrogen from outside, behind the urine tank.
const FERTILIZER: &str = "lab.fertilizer_n";
/// Grams of protein per gram of nitrogen: the factor BVAD's 122 g of protein was turned into the
/// menu's nitrogen with ([`MENU_N_KG_PER_CREW_DAY`]), so protein reads back on the same basis.
const PROTEIN_PER_N: f64 = 6.25;

/// NITROGEN `lab.urine_n → soil_n`, Step 3c: in the plant step, the roots' uptake that step
/// ([`SolutionUptake`], with its cap) is put back from the tank, so the soil's solution nitrogen
/// is held at its level by the dose — only from sowing to [`DOSE_WINDOW_DAYS`] and no more than
/// the tank holds. Outside the window the crop draws on the soil alone.
///
/// With Step 4b's `top_up` (kg N) the dose instead fills the soil solution up to that level: from
/// the tank inside the window, and from [`FERTILIZER`] for the rest at any time — EcoSanRes' cutoff
/// is a hygiene rule for urine, and late mineral nitrogen raises grain protein (Linquist, Cassman,
/// Fulton & Jackson 1992, California Agriculture 46(2):13–16).
///
/// With Step 4b-2's `any_time` the tank doses at any time too; [`UrineDose::open`] stays the
/// window, the label the readouts split on, and [`UrineDose::urine_allowed`] is the gate.
struct UrineDose {
    uptake: SolutionUptake,
    sc: SealedStationScenario,
    top_up: Option<f64>,
    any_time: bool,
}

impl UrineDose {
    /// Whether `s` is inside the dosing window (days since its crop's sowing).
    fn open(&self, s: &State) -> Result<bool, SimError> {
        let days = (s.n - station::sowing::sown_step(s)?) as f64 * self.sc.bio_dt;
        Ok(days < DOSE_WINDOW_DAYS)
    }

    /// Whether the tank may dose at `s`: inside the window, or at any time with Step 4b-2.
    fn urine_allowed(&self, s: &State) -> Result<bool, SimError> {
        Ok(self.any_time || self.open(s)?)
    }
}

impl Flow for UrineDose {
    fn type_name(&self) -> &'static str {
        "UrineDose"
    }

    fn id(&self) -> &str {
        URINE_DOSE
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let open = self.urine_allowed(snapshot)?;
        let tank = if open {
            amount(snapshot, URINE_TANK)
        } else {
            0.0
        };
        let Some(level) = self.top_up else {
            let dose = if open {
                self.uptake.parts(snapshot, env, dt)?.1.min(tank)
            } else {
                0.0
            };
            return FlowResult::new(vec![
                Leg::new(URINE_TANK.to_string(), -dose)?,
                Leg::new(SOIL_N.to_string(), dose)?,
            ]);
        };
        let deficit = (level - amount(snapshot, SOIL_N)).max(0.0);
        let urine = deficit.min(tank);
        let bought = (deficit - urine).min(amount(snapshot, FERTILIZER));
        FlowResult::new(vec![
            Leg::new(URINE_TANK.to_string(), -urine)?,
            Leg::new(FERTILIZER.to_string(), -bought)?,
            Leg::new(SOIL_N.to_string(), urine + bought)?,
        ])
    }
}

/// The station with Step 2c's food nitrogen: [`FOOD_N`] at the imported food's N:C, and
/// [`CrewNitrogen`]. With Step 3c's `reserve` (mol C of imported food held back) also
/// [`FOOD_RESERVE`] with its nitrogen, [`FoodTopUp`] to `level`, and the urine to [`URINE_TANK`].
/// With Step 4c's `burn` the feces' nitrogen goes to [`BURNER_FEED_N`] instead of the litter; with
/// Step 4d-1's `marsh` the urine goes to [`MARSH_N`] instead of the tank.
fn with_food_nitrogen(
    sc: &SealedStationScenario,
    state: State,
    fast: Registry,
    reserve: Option<(f64, f64)>,
    burn: bool,
    marsh: bool,
) -> (State, Registry) {
    let mut stocks = state.stocks.clone();
    let ratio = imported_n_per_mol_c(sc);
    let start = sc.cabin.food_store0 * ratio;
    let food_n = simple_pool(FOOD_N, "lab", Quantity::Nitrogen, start).expect("food nitrogen");
    stocks.insert(food_n.id.clone(), food_n);
    let (mut flows, aux) = fast.into_parts();
    let urine_to = if let Some((carbon, level)) = reserve {
        for s in [
            simple_pool(FOOD_RESERVE, "lab", Quantity::Carbon, carbon),
            simple_pool(FOOD_RESERVE_N, "lab", Quantity::Nitrogen, carbon * ratio),
            simple_pool(URINE_TANK, "lab", Quantity::Nitrogen, 0.0),
        ] {
            let s = s.expect("Step 3c stock");
            stocks.insert(s.id.clone(), s);
        }
        flows.push(Box::new(FoodTopUp { level }));
        if marsh {
            MARSH_N
        } else {
            URINE_TANK
        }
    } else {
        SOIL_N
    };
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    flows.push(Box::new(CrewNitrogen {
        urine_to,
        feces_to: if burn { BURNER_FEED_N } else { LITTER_N },
    }));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, fast)
}

/// The imported food's nitrogen per mol of its carbon: BVAD's menu nitrogen over the model's own
/// food carbon per crew-day.
fn imported_n_per_mol_c(sc: &SealedStationScenario) -> f64 {
    let crew = params::crew();
    let carbon_per_crew_day = sc.cabin.food_intake_rate(&crew) * 86_400.0 / sc.cabin.crew_count;
    MENU_N_KG_PER_CREW_DAY / carbon_per_crew_day
}

/// Step 3a's soil, [F] Table 13.1's silt loam: bulk density (kg/m³) and the water it holds below the
/// lower limit (m³/m³). Its extractable water, 0.132, is the model's 0.13.
const SILT_LOAM_BULK_DENSITY: f64 = 1370.0;
const SILT_LOAM_LOWER_LIMIT: f64 = 0.086;
/// The solution nitrogen at sowing, kg N per kg of soil: [F]'s worked example (Ch. 18, p. 250), 15
/// mg/kg nitrate and 1.8 mg/kg ammonium, as nitrogen (Eqn 18.22: × 14/62 and × 14/18).
const SOLUTION_N_PER_KG_SOIL: f64 = (15.0 * 14.0 / 62.0 + 1.8 * 14.0 / 18.0) * 1e-6;
/// [F] Eqn 18.5's floor (kg N per kg water): nitrogen below 1 mg/L is out of the roots' reach.
const SOLUTION_N_FLOOR: f64 = 1e-6;
/// The reference's uptake flow, which [`with_real_soil`] replaces.
const NITROGEN_UPTAKE: &str = "biosphere.nitrogen_uptake";

/// Step 3a's solution nitrogen at sowing (kg N): the reference layer's soil mass (Eqn 18.19, no
/// stones) × [`SOLUTION_N_PER_KG_SOIL`].
fn real_soil_n(bio: &SeasonScenario) -> f64 {
    SOLUTION_N_PER_KG_SOIL * bio.soil_layer_depth * SILT_LOAM_BULK_DENSITY * bio.ground_area
}

/// NITROGEN `soil_n → plant_n`, Step 3a: the reference's demand-deficit uptake with its availability
/// band switched off, capped per day at what the roots reach ([F] Eqn 18.5; Ch. 17: "if NUP is
/// calculated to be greater than SNAVL, it is adjusted at SNAVL") — `(NCON − 1 mg/L) × ATSW1 ×
/// FROOT1`, NCON the layer's nitrogen over all its water (Eqn 18.24). The reference layer is taken
/// to be as wet as the root zone: the model's one root-zone store has no layers.
struct SolutionUptake {
    inner: NitrogenUptake,
}

impl SolutionUptake {
    fn new(sc: &SealedStationScenario) -> Self {
        let p = sc.plant_params();
        let bio = &sc.bio;
        let inner = NitrogenUptake {
            id: NITROGEN_UPTAKE.to_string(),
            soil_n: SOIL_N.to_string(),
            plant_n: PLANT_N.to_string(),
            leaf_c: LEAF_C.to_string(),
            stem_c: STEM_C.to_string(),
            root_c: ROOT_C.to_string(),
            storage_c: STORAGE_C.to_string(),
            max_uptake_capacity: p.nitro.max_uptake_capacity,
            n_target_coefficient: p.nitro.n_target_coefficient,
            n_target_exponent: p.nitro.n_target_exponent,
            n_target_w_plateau: p.nitro.n_target_w_plateau,
            dm_kg_per_mol_c: p.nitro.dm_kg_per_mol_c,
            ground_area: bio.ground_area,
            // The band off: every amount ≥ 0 is at or above `sn_critical`, so it reads 1.
            sn_residual: -1.0,
            sn_critical: 0.0,
            rooted_depth_aux: ROOTED_DEPTH.to_string(),
            soil_layer_depth: bio.soil_layer_depth,
        };
        SolutionUptake { inner }
    }

    /// The reference layer's water (kg): plant-available (ATSW1 — the root zone's per metre of
    /// depth, × the layer's depth) and all of it (WAT1, the lower limit's water added).
    fn layer_water(&self, s: &State) -> (f64, f64) {
        let i = &self.inner;
        let per_m = WATER_DENSITY * i.ground_area;
        let depth = s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
        let atsw = if depth > 0.0 {
            amount(s, SOIL_WATER) / depth * i.soil_layer_depth
        } else {
            0.0
        };
        (atsw, SILT_LOAM_LOWER_LIMIT * i.soil_layer_depth * per_m + atsw)
    }

    /// NCON, the layer's solution nitrogen over all its water (kg N per kg water).
    fn concentration(&self, s: &State) -> f64 {
        amount(s, SOIL_N) / self.layer_water(s).1
    }

    /// SNAVL, what the roots reach (kg N, a daily amount).
    fn reach(&self, s: &State) -> f64 {
        let depth = s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
        (self.concentration(s) - SOLUTION_N_FLOOR).max(0.0)
            * self.layer_water(s).0
            * root_zone_fraction(depth, self.inner.soil_layer_depth)
    }

    /// The step's uptake without the cap, and with it (kg N).
    fn parts(&self, s: &State, env: &dyn Environment, dt: f64) -> Result<(f64, f64), SimError> {
        let wanted = leg(&self.inner.evaluate(s, env, dt)?, PLANT_N);
        Ok((wanted, wanted.min(self.reach(s) * dt)))
    }
}

impl Flow for SolutionUptake {
    fn type_name(&self) -> &'static str {
        "SolutionUptake"
    }

    fn id(&self) -> &str {
        NITROGEN_UPTAKE
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let (_, given) = self.parts(snapshot, env, dt)?;
        FlowResult::new(vec![
            Leg::new(SOIL_N.to_string(), -given)?,
            Leg::new(PLANT_N.to_string(), given)?,
        ])
    }
}

/// The plant registry with Step 3a's [`SolutionUptake`] in the reference uptake's place (the
/// reference's reads no environment, so its sowing-clock and chamber wrappers change nothing) —
/// and with `dose`, Step 3c's [`UrineDose`] (with Step 4b's `top_up` level, kg N).
fn with_real_soil(
    sc: &SealedStationScenario,
    state: &State,
    bio: Registry,
    dose: bool,
    top_up: Option<f64>,
    any_time: bool,
) -> Registry {
    let (flows, aux) = bio.into_parts();
    let before = flows.len();
    let mut flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .filter(|f| f.id() != NITROGEN_UPTAKE)
        .collect();
    assert_eq!(flows.len() + 1, before, "one reference uptake to replace");
    flows.push(Box::new(SolutionUptake::new(sc)));
    if dose {
        flows.push(Box::new(UrineDose {
            uptake: SolutionUptake::new(sc),
            sc: *sc,
            top_up,
            any_time,
        }));
    }
    Registry::new(flows, &state.stocks, aux).expect("plant registry")
}

/// The nitrogen the run books (kg N): the plant's, the soil's (available, litter, microbes, humus),
/// the food's, and held back (Step 3c's reserve and urine tank, Step 4b's fertilizer store, Step
/// 4c's burner feed and the N₂ it made, Step 4d-1's marsh water; Step 4d-2's subsoil nitrogen with
/// the soil's, its plants' tank with what is held back).
fn nitrogen(s: &State) -> (f64, f64, f64, f64) {
    let soil = [SOIL_N, LITTER_N, MICROBIAL_N, HUMUS_N, SUBSOIL_N]
        .iter()
        .map(|id| amount(s, id))
        .sum();
    (
        amount(s, PLANT_N),
        soil,
        amount(s, FOOD_N),
        amount(s, FOOD_RESERVE_N)
            + amount(s, URINE_TANK)
            + amount(s, FERTILIZER)
            + amount(s, BURNER_FEED_N)
            + amount(s, N2_GAS)
            + amount(s, MARSH_N)
            + amount(s, PLANT_TANK_N),
    )
}

/// The station with Step 4b's [`FERTILIZER`] store, starting at `charge` kg N.
fn with_fertilizer(state: State, charge: f64) -> State {
    let mut stocks = state.stocks.clone();
    let store = simple_pool(FERTILIZER, "lab", Quantity::Nitrogen, charge).expect("fertilizer");
    stocks.insert(store.id.clone(), store);
    State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state")
}

/// The station with Step 2a's CO₂ store: the scrubber re-pointed at [`CO2_STORE`] (starting at
/// `charge` mol — empty before Step 4a), and [`Co2Injector`].
fn with_co2_store(
    sc: &SealedStationScenario,
    state: State,
    fast: Registry,
    charge: f64,
) -> (State, Registry, Co2Injector) {
    let eclss = params::eclss();
    let air = sc.bio.chamber_air_capacity_mol;
    let mut stocks = state.stocks.clone();
    let store = gas_pool(CO2_STORE, Quantity::Carbon, charge, co2_composition()).expect("store");
    stocks.insert(store.id.clone(), store);
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (flows, aux) = fast.into_parts();
    let mut flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|f| -> Box<dyn Flow> {
            if f.id() == CO2_SCRUBBER {
                Box::new(CO2Scrubber::new(
                    CO2_SCRUBBER.to_string(),
                    CARBON_POOL.to_string(),
                    CO2_STORE.to_string(),
                    eclss,
                    air,
                ))
            } else {
                f
            }
        })
        .collect();
    let injector = || Co2Injector {
        target: eclss.co2_target_mol(air).expect("an above-target scrubber"),
        rate: eclss.co2_scrub_rate,
        sc: *sc,
    };
    flows.push(Box::new(injector()));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, fast, injector())
}

/// The station with Step 4a's [`O2_TANK`], starting at `charge` mol, in the reference makeup's
/// place ([`O2Tank`]); `boundary.o2_supply` stays, unused.
fn with_o2_tank(
    sc: &SealedStationScenario,
    state: State,
    fast: Registry,
    charge: f64,
) -> (State, Registry) {
    let air = sc.bio.chamber_air_capacity_mol;
    let mut stocks = state.stocks.clone();
    let tank = gas_pool(O2_TANK, Quantity::Oxygen, charge, o2_composition()).expect("O₂ tank");
    stocks.insert(tank.id.clone(), tank);
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (flows, aux) = fast.into_parts();
    let before = flows.len();
    let mut flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .filter(|f| f.id() != O2_MAKEUP)
        .collect();
    assert_eq!(flows.len() + 1, before, "one reference makeup to replace");
    flows.push(Box::new(O2Tank {
        inner: O2Makeup::new(
            O2_MAKEUP.to_string(),
            O2_TANK.to_string(),
            O2_POOL.to_string(),
            params::eclss(),
            air,
        ),
    }));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, fast)
}

/// ENERGY `boundary.grid → power.battery`: tops the battery back up to `level` (J) each step,
/// from the step's starting charge. A lab stand-in for a grid connection (module doc).
struct GridSupply {
    level: f64,
}

impl Flow for GridSupply {
    fn type_name(&self) -> &'static str {
        "GridSupply"
    }

    fn id(&self) -> &str {
        GRID_SUPPLY
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        _dt: f64,
    ) -> Result<FlowResult, SimError> {
        let deficit = (self.level - amount(snapshot, BATTERY)).max(0.0);
        FlowResult::new(vec![
            Leg::new(GRID.to_string(), -deficit)?,
            Leg::new(BATTERY.to_string(), deficit)?,
        ])
    }
}

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map_or(0.0, |x| x.amount)
}

fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

/// The carbon held in the room outside its air: the crop (with its stem reserve), the soil's
/// litter, microbes and humus, the consumers, and what the litter sink has taken.
fn held_c(s: &State) -> (f64, f64) {
    let soil = [LITTER_CARBON, MICROBIAL_CARBON, HUMUS_CARBON, CONSUMER_CARBON, LITTER_SINK]
        .iter()
        .map(|id| amount(s, id))
        .sum();
    (plant_c(s) + amount(s, STEM_RESERVE_C), soil)
}

fn plant_water(s: &State) -> f64 {
    [SOIL_WATER, SUBSOIL_WATER, CONDENSATE, WATER_VAPOR]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

/// The water stocks whose flows the run tallies.
const WATER_STOCKS: [&str; 5] = [SOIL_WATER, SUBSOIL_WATER, WATER_VAPOR, CONDENSATE, WATER_STORE];

fn tally(t: &mut std::collections::BTreeMap<(String, &'static str), f64>, id: &str, res: &FlowResult) {
    for stock in WATER_STOCKS {
        let v = leg(res, stock);
        if v != 0.0 {
            *t.entry((id.to_string(), stock)).or_insert(0.0) += v;
        }
    }
}

fn leg(res: &FlowResult, stock: &str) -> f64 {
    res.legs
        .iter()
        .filter(|l| l.stock == stock)
        .map(|l| l.amount)
        .sum()
}

/// The crop and its soil at `a` m²: every extensive starting amount × a, and the plants' tank.
/// The room's air and its gases and vapour are the room's, not the crop's: left alone.
fn crop_at(bio: &SeasonScenario, a: f64) -> SeasonScenario {
    let mut t = *bio;
    t.ground_area *= a;
    for f in [
        &mut t.leaf_c0,
        &mut t.stem_c0,
        &mut t.root_c0,
        &mut t.storage_c0,
        &mut t.co2_atmos0,
        &mut t.litter_carbon0,
        &mut t.consumer_c0,
        &mut t.soil_water0,
        &mut t.subsoil_water0,
        &mut t.water_source0,
        &mut t.condensate0,
        &mut t.soil_n0,
        &mut t.n_source0,
        &mut t.plant_n0,
        &mut t.sn_residual,
        &mut t.sn_critical,
    ] {
        *f *= a;
    }
    t
}

/// The chamber at `a` m²: heat capacity, cooler and heater × a; walls by the box rule.
fn chamber_at(c: &ChamberParams, a: f64) -> ChamberParams {
    let box_walls = |area: f64| 2.0 * area + 4.0 * CHAMBER_HEIGHT * area.sqrt();
    ChamberParams {
        heat_capacity: c.heat_capacity * a,
        cooling_capacity: c.cooling_capacity * a,
        heater_capacity: c.heater_capacity * a,
        wall_area: c.wall_area * (box_walls(a) / box_walls(1.0)),
        ..*c
    }
}

/// The sealed station sized for `a` m² of crop ([`crop_at`], [`chamber_at`], the lamp). The
/// identity at `a = 1`.
fn sized(a: f64) -> SealedStationScenario {
    let s = sealed_station_scenario();
    SealedStationScenario {
        bio: crop_at(&s.bio, a),
        lamp_power_w: s.lamp_power_w * a,
        chamber: chamber_at(&s.chamber, a),
        ..s
    }
}

/// The heat-rejection stand-in (module doc): the radiator and the structure × a.
fn outside_heat(a: f64) -> ThermalParams {
    let t = params::thermal();
    ThermalParams {
        radiator_area: t.radiator_area * a,
        heat_capacity: t.heat_capacity * a,
        ..t
    }
}

/// The station built on the grid: the sealed build plus `boundary.grid` and [`GridSupply`],
/// holding the battery at its starting charge.
fn on_grid(
    sc: &SealedStationScenario,
    thermal: &ThermalParams,
    close_feces: bool,
) -> (
    State,
    simcore::registry::Registry,
    simcore::registry::Registry,
) {
    let (state, bio, fast) = build(sc, thermal, close_feces);
    let mut stocks = state.stocks.clone();
    let grid = simcore::boundary::source(GRID.to_string(), Quantity::Energy, 0.0, true)
        .expect("grid");
    stocks.insert(grid.id.clone(), grid);
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (mut flows, aux) = fast.into_parts();
    flows.push(Box::new(GridSupply {
        level: sc.battery0,
    }));
    let fast = Registry::new(flows, &state.stocks, aux).expect("fast registry");
    (state, bio, fast)
}

fn build(
    sc: &SealedStationScenario,
    thermal: &ThermalParams,
    close_feces: bool,
) -> (
    State,
    simcore::registry::Registry,
    simcore::registry::Registry,
) {
    build_sealed_station(
        &params::charge(),
        thermal,
        &params::crew(),
        &params::eclss(),
        &sp::water_recovery(),
        &sp::lamp(),
        &sp::harvest(),
        sc,
        false,
        close_feces,
    )
    .expect("build sealed")
}

/// `--control`: [`sized`]`(1.0)` through the reference's own driver reproduces the committed
/// golden, byte for byte.
fn control() {
    let sc = sized(1.0);
    let (state, bio, fast) = build(&sc, &params::thermal(), false);
    let bio_r = sealed_bio_resolver(&sp::lamp(), &sc).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&params::charge(), &params::crew(), &sc).expect("fast");
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        state,
        &bio_r,
        &fast_r,
        &sc,
    )
    .expect("run sealed");
    let ours = simcore::snapshot::from_engine(states.last().expect("a day")).to_json();
    let committed = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/golden/sealed_station_state.json"),
    )
    .expect("golden file");
    println!(
        "control: sized(1.0) reproduces sealed_station_state.json byte for byte: {} \
         (rationed {rationed}, events {})",
        ours == committed,
        events.len()
    );
}

/// Step 2b's harvest: the station's re-sow hook, with the grain less the next crop's seed moved to
/// the crew's food store first when the crop has matured. Each harvest (mol C) is pushed onto
/// `harvests`. The seed is the seedling the re-sow starts (`reset_crop` refuses a smaller grain).
///
/// With Step 4c's `reserve` (mol C) two things go to [`BURNER_FEED`] first, each with its
/// nitrogen: the food store above `reserve` before the new grain goes in — only the old carryover,
/// so the harvest the crew need until the next one is never burned — and the straw: the leaves,
/// stems and stem reserve, with the plant's nitrogen in their carbon's share, so the seedling keeps
/// the parent's concentration as `reset_crop` gives it. The roots stay for the re-sow to put in
/// the soil, as CEEF's residue stayed in its planting medium (Tako et al., Fig. 2(b)).
fn harvest_at_maturity(
    sc: &SealedStationScenario,
    harvests: Rc<RefCell<Vec<Harvested>>>,
    with_nitrogen: bool,
    reserve: Option<f64>,
) -> OwnedResetHook {
    let base = sealed_reset_hook(sc);
    let pheno = sc.pheno();
    let p = sc.plant_params();
    let dm_per_mol_c = p.nitro.dm_kg_per_mol_c;
    let straw_n_per_mol_c = p.nitro.n_residual_per_mol_c;
    let seed = sc.bio.leaf_c0 + sc.bio.stem_c0 + sc.bio.root_c0;
    Box::new(move |n: u64, current: &State| -> Result<Option<State>, SimError> {
        if !is_mature(current, &pheno)? {
            return base(n, current);
        }
        let grain = amount(current, STORAGE_C);
        let eaten = (grain - seed).max(0.0);
        let mut h = Harvested {
            carbon: eaten,
            plant_n: amount(current, PLANT_N),
            ..Harvested::default()
        };
        let mut stocks = current.stocks.clone();
        let mut set = |id: &str, value: f64| -> Result<(), SimError> {
            let held = stocks[id].with_amount(value)?;
            stocks.insert(id.to_string(), held);
            Ok(())
        };
        // Step 4c's running amounts (the `set` closure holds the stocks while it lives).
        let mut food_n = amount(current, FOOD_N);
        let mut feed = amount(current, BURNER_FEED);
        let mut feed_n = amount(current, BURNER_FEED_N);
        // Step 4c: the old carryover above the reserve, before the new grain goes in.
        let mut food = amount(current, FOOD_STORE);
        if let Some(level) = reserve {
            let over = (food - level).max(0.0);
            if over > 0.0 {
                let n = over * food_n / food;
                h.food_burned_c = over;
                h.food_burned_n = n;
                food -= over;
                food_n -= n;
                feed += over;
                feed_n += n;
            }
        }
        // What is left of the grain is exactly the seed: `reset_crop` refuses a grain below it.
        set(STORAGE_C, if eaten > 0.0 { seed } else { grain })?;
        set(FOOD_STORE, food + eaten)?;
        let mut plant_n = h.plant_n;
        if with_nitrogen {
            // USDA's grain nitrogen, from the plant, which keeps the straw's cited residual and the
            // seed's share at that concentration (the re-sow's own rule: the seedling keeps the
            // parent's concentration).
            let straw: f64 = [LEAF_C, STEM_C, ROOT_C]
                .iter()
                .map(|id| amount(current, id))
                .sum();
            h.wanted_n = eaten * dm_per_mol_c * GRAIN_N_PER_DM;
            let kept = (straw + seed) * straw_n_per_mol_c;
            h.grain_n = h.wanted_n.min((h.plant_n - kept).max(0.0));
            plant_n = h.plant_n - h.grain_n;
            set(PLANT_N, plant_n)?;
            set(FOOD_N, food_n + h.grain_n)?;
        }
        if reserve.is_some() {
            // Step 4c: the straw to the burner. Its nitrogen is the plant's (after the grain's)
            // in the leaves' and stems' share of the vegetative carbon — `reset_crop`'s own
            // concentration basis, so the seedling's nitrogen is what it would have been.
            let (leaf, stem, root) = (
                amount(current, LEAF_C),
                amount(current, STEM_C),
                amount(current, ROOT_C),
            );
            let veg = leaf + stem + root;
            let n = if veg > 0.0 {
                plant_n * (leaf + stem) / veg
            } else {
                0.0
            };
            h.straw_c = leaf + stem + amount(current, STEM_RESERVE_C);
            h.straw_n = n;
            set(LEAF_C, 0.0)?;
            set(STEM_C, 0.0)?;
            if current.stocks.contains_key(STEM_RESERVE_C) {
                set(STEM_RESERVE_C, 0.0)?;
            }
            set(PLANT_N, plant_n - n)?;
            set(BURNER_FEED, feed + h.straw_c)?;
            set(BURNER_FEED_N, feed_n + n)?;
        }
        let harvested = State::new(current.n, stocks, current.rng_seed, current.aux.clone())?;
        harvests.borrow_mut().push(h);
        base(n, &harvested)
    })
}

/// One harvest: the grain carbon the crew got (mol C); with Step 2c also the plant's nitrogen at
/// maturity, the grain nitrogen USDA's content asks for, and the grain nitrogen given (kg N).
#[derive(Debug, Clone, Copy, Default)]
struct Harvested {
    carbon: f64,
    plant_n: f64,
    wanted_n: f64,
    grain_n: f64,
    /// Step 4c: the straw (leaves, stems, stem reserve) and the food above the reserve sent to the
    /// burner, mol C and kg N.
    straw_c: f64,
    straw_n: f64,
    food_burned_c: f64,
    food_burned_n: f64,
}

/// One crop cycle's readouts.
struct Cycle {
    days: usize,
    exhaled: f64,
    /// CO₂ that left the station (`boundary.co2_removed`).
    scrubbed: f64,
    pool_change: f64,
    /// The change of Step 2a's CO₂ store (0 without one).
    store_change: f64,
    store_end: f64,
    best_day_share: f64,
    grain_max: f64,
    plant_c_max: f64,
    /// Change over the cycle of the crop's and the soil's carbon ([`held_c`]).
    crop_change: f64,
    soil_change: f64,
    soil_end: f64,
    /// Step 2b: the grain the crew got from this cycle's crop (0 without one).
    harvested: f64,
    /// The harvest inside this cycle's books: the previous crop's, at this cycle's first re-sow.
    harvest_booked: f64,
    /// The crew's feces into the soil over the cycle (0 unless they go there).
    feces_to_soil: f64,
    /// Step 2c: this crop's harvest record, and the cycle's nitrogen books (kg N): the change of
    /// the plant's, the soil's and the food's nitrogen, and what the crew ate.
    harvest: Harvested,
    n_change: (f64, f64, f64, f64),
    n_eaten: f64,
    /// Step 3c (kg N unless said): the imported food moved to the store (mol C) and its nitrogen;
    /// the urine into the tank, the doses out, the tank at the cycle's end; the roots' uptake; the
    /// soil solution's lowest at a day's end; the food store and the reserve at the end (mol C).
    imported_c: f64,
    imported_n: f64,
    urine_in: f64,
    dosed: f64,
    tank_end: f64,
    uptake: f64,
    soil_n_min: f64,
    soil_n_start: f64,
    capped_with_urine: u64,
    /// The tank's lowest at a plant step inside the window (kg N; ∞ if no such step).
    tank_min_in_window: f64,
    /// The days inside the window the tank was empty at some plant step.
    tank_dry_days: usize,
    food_store_end: f64,
    reserve_end: f64,
    /// Step 3a: the soil's solution nitrogen at the cycle's end (kg N), the highest concentration at
    /// a day's end (mg/L), and the plant steps the roots' reach held the uptake back, by how much (kg N).
    soil_n_end: f64,
    ncon_max: f64,
    capped_steps: u64,
    n_short: f64,
    /// The smallest reach ÷ wanted over the cycle's plant steps with any uptake wanted (∞ if none):
    /// below 1 the cap binds; the margin it ran with otherwise.
    reach_ratio_min: f64,
    /// Step 4a: the tanks' lowest over the cycle and their ends (mol); the plant steps under
    /// nitrogen stress and the lowest stress factor.
    co2_store_low: f64,
    o2_tank_low: f64,
    o2_tank_end: f64,
    o2_tank_change: f64,
    n_stressed: u64,
    n_stress_min: f64,
    /// Of `n_stressed`, the plant steps in the cycle's cold weeks (the scenario's `cold.days`), and
    /// the lowest factor after them (a re-sown seedling keeps its parent's nitrogen concentration,
    /// at the straw's residual after a harvest, so the cold weeks' lowest is the re-sow's).
    n_stressed_cold: u64,
    n_stress_min_warm: f64,
    /// Step 4b (kg N): the fertilizer dosed inside the urine window and after it, and the store's end.
    fert_window: f64,
    fert_late: f64,
    fert_end: f64,
    /// Step 4b-2 (kg N): the urine dosed after the window.
    urine_late: f64,
    /// Step 4c: the change of the burner's feed and char (mol C); the CO₂ it returned to the tank
    /// (mol C); the nitrogen it sent to N₂ and caught (kg N) over the cycle.
    burner_change: f64,
    /// Step 4c, booked in this cycle (the previous crop's, at this cycle's first re-sow): the straw
    /// (mol C, kg N) and the food above the reserve; and the feces' nitrogen over the cycle (kg N).
    straw_booked: (f64, f64),
    food_burned_booked: f64,
    food_burned_n_booked: f64,
    feces_n: f64,
    burned_co2: f64,
    char_end: f64,
    /// Step 4c, `--burn-co2-leak`: the CO₂ the burner leaked outside over the cycle (mol C).
    leaked: f64,
    n_to_gas: f64,
    n_caught: f64,
    /// Step 4d-1 over the cycle: the urine into the marsh, the nitrogen its plants took, its
    /// outflow to the tank (kg N), the carbon they fixed (mol C), and its water's nitrogen at the end.
    marsh_in: f64,
    marsh_n: f64,
    marsh_out: f64,
    marsh_c: f64,
    marsh_end: f64,
    /// Step 4d-2 over the cycle: [`Minutes::leach`]'s six sums (kg N), and the nitrogen below the
    /// roots and in the plants' tank at the end.
    leach: [f64; 6],
    subsoil_n_end: f64,
    plant_tank_n_end: f64,
}

#[derive(Default)]
struct Minutes {
    chamber_min: f64,
    chamber_max: f64,
    node_min: f64,
    node_max: f64,
    co2_min: f64,
    co2_max: f64,
    o2_min: f64,
    o2_max: f64,
    cooler_blocked: u64,
    cooler_capped: u64,
    heater_capped: u64,
    above_setpoint: u64,
    store_min: f64,
    /// The crew's exhaled CO₂ (mol), per day — booked from the crew's own flow, not from the
    /// food store's fall (which a harvest refills).
    exhaled: Vec<f64>,
    /// Room CO₂ (mol) over the lit minutes: the lowest, and the sum for a mean.
    lit_co2_min: f64,
    lit_co2_sum: f64,
    lit_minutes: u64,
    /// Lit minutes the room is below 1 000 ppm.
    lit_below_1000: u64,
    /// Step 2a: the store's peak, and the lit minutes it gave less than the injector asked.
    co2_store_max: f64,
    store_short: u64,
    /// Step 2c: the crew's nitrogen eaten (kg N), per day.
    n_eaten: Vec<f64>,
    /// Step 3a: per day, the plant steps the roots' reach capped the uptake, and the uptake it held
    /// back (kg N).
    n_capped: Vec<u64>,
    n_short: Vec<f64>,
    reach_ratio_min: Vec<f64>,
    /// Step 3c, per day (kg N): the urine into the tank, the doses out of it, the roots' uptake.
    urine_in: Vec<f64>,
    dosed: Vec<f64>,
    uptake: Vec<f64>,
    /// Step 3c, per day: the plant steps capped while the tank held urine and the window was open;
    /// the tank's lowest at a plant step's start (kg N).
    capped_with_urine: Vec<u64>,
    tank_min: Vec<f64>,
    /// Step 4a, per day: the CO₂ and O₂ tanks' lowest at a minute's end (mol); the plant steps the
    /// crop's nitrogen stress factor was below 1, and its lowest (1 if never).
    co2_store_low: Vec<f64>,
    o2_tank_low: Vec<f64>,
    n_stressed: Vec<u64>,
    n_stress_min: Vec<f64>,
    /// Step 4a: the minutes the O₂ tank ended empty.
    o2_tank_empty: u64,
    /// Step 4b, per day (kg N): the fertilizer dosed inside the urine window, and after it.
    fert_window: Vec<f64>,
    fert_late: Vec<f64>,
    /// Step 4b-2, per day (kg N): the urine dosed after the window.
    urine_late: Vec<f64>,
    /// Step 4c, per day: the burner's CO₂ to the tank (mol C) and nitrogen caught (kg N).
    burned_co2: Vec<f64>,
    burn_n_caught: Vec<f64>,
    /// Step 4d-1, per day: the urine into the marsh (kg N), the carbon its plants fixed from the
    /// room (mol C), the nitrogen they took (kg N), and its outflow to the urine tank (kg N).
    marsh_in: Vec<f64>,
    marsh_c: Vec<f64>,
    marsh_n: Vec<f64>,
    marsh_out: Vec<f64>,
    /// Step 4d-2, per day (kg N): leached past the roots, taken back up by the roots' capture, sent
    /// by the overflow to the plants' tank, watered back from the tank, drained from the tank to the
    /// urine tank, and the soil solution's net change from all of them.
    leach: Vec<[f64; 6]>,
    /// Every flow's legs on the water stocks ([`WATER_STOCKS`]), summed over the run (kg).
    water: std::collections::BTreeMap<(String, &'static str), f64>,
}

struct Run {
    a: f64,
    cycles: Vec<Cycle>,
    m: Minutes,
    first: State,
    last: State,
    rationed: (u64, u64),
    days: usize,
    crew: f64,
    opts: Options,
    /// The crew's food intake over the run (mol C).
    eaten: f64,
}

/// Which steps the big run turns on; each turns on the ones it builds on ([`Options::closed`]).
#[derive(Debug, Clone, Copy, Default)]
struct Options {
    co2_store: bool,
    food_loop: bool,
    with_nitrogen: bool,
    real_soil: bool,
    feed_crew: bool,
    /// Step 4a: the gas tanks' starting charges (mol CO₂, mol O₂), inputs with no default.
    tanks: Option<(f64, f64)>,
    /// Step 4b: the fertilizer store's charge (kg N) and the soil solution's top-up level (g N/m²),
    /// inputs with no default; needs `tanks`.
    fertilizer: Option<(f64, f64)>,
    /// Step 4b-2: urine dosed at any time, not only inside the window; needs `fertilizer`.
    urine_any_time: bool,
    /// Step 4c: the straw, the feces and the food above a reserve burned back to CO₂; needs
    /// `urine_any_time`.
    burn: Option<Burn>,
    /// Step 4d-1: the crew's urine through a marsh on its way to the urine tank; needs `burn`.
    marsh: Option<MarshOpts>,
    /// Step 4d-2: the drained water carries its nitrogen ([`with_leaching`]); needs `marsh`.
    leaching: bool,
}

impl Options {
    /// The options with every step a chosen one builds on turned on.
    fn closed(self) -> Options {
        let feed_crew = self.feed_crew || self.tanks.is_some();
        let real_soil = self.real_soil || feed_crew;
        let with_nitrogen = self.with_nitrogen || real_soil;
        let food_loop = self.food_loop || with_nitrogen;
        Options {
            co2_store: self.co2_store || food_loop,
            food_loop,
            with_nitrogen,
            real_soil,
            feed_crew,
            tanks: self.tanks,
            fertilizer: self.fertilizer,
            urine_any_time: self.urine_any_time,
            burn: self.burn,
            marsh: self.marsh,
            leaching: self.leaching,
        }
    }
}

fn run(a: f64, crew: Option<f64>, days: usize, opts: Options) -> Run {
    let opts = opts.closed();
    let Options {
        co2_store,
        food_loop,
        with_nitrogen,
        real_soil,
        feed_crew,
        tanks,
        fertilizer,
        urine_any_time,
        burn,
        marsh,
        leaching,
    } = opts;
    assert!(
        fertilizer.is_none() || tanks.is_some(),
        "Step 4b builds on Step 4a's charged tanks"
    );
    assert!(
        !urine_any_time || fertilizer.is_some(),
        "Step 4b-2 builds on Step 4b's top-up"
    );
    assert!(
        burn.is_none() || urine_any_time,
        "Step 4c builds on Step 4b-2's urine at any time"
    );
    assert!(
        marsh.is_none() || burn.is_some(),
        "Step 4d-1 builds on Step 4c's burner"
    );
    assert!(!leaching || marsh.is_some(), "Step 4d-2 builds on Step 4d-1's marsh");
    let (mut sc, thermal) = (sized(a), outside_heat(a));
    if let Some(n) = crew {
        let k = n / sc.cabin.crew_count;
        sc.cabin.crew_count = n;
        sc.cabin.food_store0 *= k;
        sc.cabin.water_store0 *= k;
    }
    if real_soil {
        sc.bio.soil_n0 = real_soil_n(&sc.bio);
    }
    // Step 3c: the store starts at its one-day level, the rest of the imported food held back.
    let reserve = feed_crew.then(|| {
        let level =
            FOOD_BUFFER_DAYS * sc.cabin.food_intake_rate(&params::crew()) * 86_400.0;
        let held = sc.cabin.food_store0 - level;
        sc.cabin.food_store0 = level;
        (held, level)
    });
    let chamber = sc.chamber;
    let (state, bio, fast) = on_grid(&sc, &thermal, food_loop);
    let uptake = real_soil.then(|| SolutionUptake::new(&sc));
    // Step 4b's top-up level, g N/m² to kg N over the area.
    let top_up = fertilizer.map(|(_, level)| level * a / 1e3);
    let dose = feed_crew.then(|| UrineDose {
        uptake: SolutionUptake::new(&sc),
        sc,
        top_up,
        any_time: urine_any_time,
    });
    let (state, fast, injector) = if co2_store {
        let (s, f, i) = with_co2_store(&sc, state, fast, tanks.map_or(0.0, |t| t.0));
        (s, f, Some(i))
    } else {
        (state, fast, None)
    };
    let (state, fast) = match tanks {
        Some((_, o2)) => with_o2_tank(&sc, state, fast, o2),
        None => (state, fast),
    };
    let (state, fast) = if with_nitrogen {
        with_food_nitrogen(&sc, state, fast, reserve, burn.is_some(), marsh.is_some())
    } else {
        (state, fast)
    };
    let (state, fast) = match burn {
        Some(b) => with_burner(state, fast, b),
        None => (state, fast),
    };
    let (state, fast) = match marsh {
        Some(mo) => with_marsh(&sc, state, fast, mo),
        None => (state, fast),
    };
    // Step 4c's food reserve (mol C): the crew's intake × the input days.
    let food_reserve = burn
        .map(|b| b.reserve_days * sc.cabin.food_intake_rate(&params::crew()) * 86_400.0);
    let state = match fertilizer {
        Some((charge, _)) => with_fertilizer(state, charge),
        None => state,
    };
    let bio = if real_soil {
        with_real_soil(&sc, &state, bio, feed_crew, top_up, urine_any_time)
    } else {
        bio
    };
    let (state, bio, fast) = if leaching {
        with_leaching(&sc, state, bio, fast)
    } else {
        (state, bio, fast)
    };
    let air = sc.bio.chamber_air_capacity_mol;
    let first = state.clone();
    let bio_r = sealed_bio_resolver(&sp::lamp(), &sc).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&params::charge(), &params::crew(), &sc).expect("fast");
    let harvests: Rc<RefCell<Vec<Harvested>>> = Rc::default();
    let reset = if food_loop {
        harvest_at_maturity(&sc, Rc::clone(&harvests), with_nitrogen, food_reserve)
    } else {
        sealed_reset_hook(&sc)
    };
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: &bio_r,
        fast_resolver: &fast_r,
        steps_per_day: sc.steps_per_day,
        slow_steps_per_day: sc.bio_steps_per_day,
        slow_dt: sc.bio_dt,
        fast_dt: sc.cabin_dt,
        slow_reset: Some(&*reset),
    };
    let dt = sc.cabin_dt;
    let mut m = Minutes {
        chamber_min: f64::INFINITY,
        chamber_max: f64::NEG_INFINITY,
        node_min: f64::INFINITY,
        node_max: f64::NEG_INFINITY,
        co2_min: f64::INFINITY,
        co2_max: f64::NEG_INFINITY,
        o2_min: f64::INFINITY,
        o2_max: f64::NEG_INFINITY,
        store_min: f64::INFINITY,
        lit_co2_min: f64::INFINITY,
        exhaled: vec![0.0; days],
        n_eaten: vec![0.0; days],
        n_capped: vec![0; days],
        n_short: vec![0.0; days],
        reach_ratio_min: vec![f64::INFINITY; days],
        urine_in: vec![0.0; days],
        dosed: vec![0.0; days],
        uptake: vec![0.0; days],
        capped_with_urine: vec![0; days],
        tank_min: vec![f64::INFINITY; days],
        co2_store_low: vec![f64::INFINITY; days],
        o2_tank_low: vec![f64::INFINITY; days],
        n_stressed: vec![0; days],
        n_stress_min: vec![1.0; days],
        fert_window: vec![0.0; days],
        fert_late: vec![0.0; days],
        urine_late: vec![0.0; days],
        burned_co2: vec![0.0; days],
        burn_n_caught: vec![0.0; days],
        marsh_in: vec![0.0; days],
        marsh_c: vec![0.0; days],
        marsh_n: vec![0.0; days],
        marsh_out: vec![0.0; days],
        leach: vec![[0.0; 6]; days],
        ..Minutes::default()
    };
    let nitro = sc.plant_params().nitro;
    // The crop's cycle ends at each re-sow: the day index of each reset.
    let mut fast_steps: u64 = 0;
    let mut reset_days: Vec<usize> = Vec::new();
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => reset_days.push((fast_steps / sc.steps_per_day) as usize),
        Side::Slow => {
            let env = bio_r.bind(before, sc.bio_dt);
            let day = (fast_steps / sc.steps_per_day) as usize;
            if let Some(t) = m.tank_min.get_mut(day) {
                *t = t.min(amount(before, URINE_TANK));
            }
            let f_n = nitrogen_stress_factor(
                amount(before, PLANT_N),
                [LEAF_C, STEM_C, ROOT_C].iter().map(|id| amount(before, id)).sum(),
                nitro.n_residual_per_mol_c,
                nitro.n_critical_per_mol_c,
            );
            if f_n < 1.0 {
                if let (Some(c), Some(x)) = (m.n_stressed.get_mut(day), m.n_stress_min.get_mut(day)) {
                    *c += 1;
                    *x = x.min(f_n);
                }
            }
            for f in bio.registry().flows() {
                let res = f.evaluate(before, &env, sc.bio_dt).expect("plant flow");
                tally(&mut m.water, f.id(), &res);
                let into = |v: &mut Vec<f64>, x: f64| {
                    if let Some(d) = v.get_mut(day) {
                        *d += x;
                    }
                };
                match f.id() {
                    URINE_DOSE => {
                        into(&mut m.dosed, leg(&res, SOIL_N));
                        let bought = -leg(&res, FERTILIZER);
                        let urine = -leg(&res, URINE_TANK);
                        if bought > 0.0 || urine > 0.0 {
                            let open = dose.as_ref().is_some_and(|d| d.open(before).expect("window"));
                            into(if open { &mut m.fert_window } else { &mut m.fert_late }, bought);
                            if !open {
                                into(&mut m.urine_late, urine);
                            }
                        }
                    }
                    NITROGEN_UPTAKE => into(&mut m.uptake, leg(&res, PLANT_N)),
                    WATERING | RESERVE_TOP_UP | DRAINAGE if leaching && day < days => {
                        let from_tank = -leg(&res, PLANT_TANK_N);
                        let (up, below) =
                            (leg(&res, SOIL_WATER).max(0.0), leg(&res, SUBSOIL_WATER).max(0.0));
                        let below_share = if from_tank > 0.0 && up + below > 0.0 {
                            below / (up + below)
                        } else {
                            0.0
                        };
                        let l = &mut m.leach[day];
                        l[0] += leg(&res, SUBSOIL_N) - from_tank * below_share;
                        l[3] += from_tank;
                        l[5] += leg(&res, SOIL_N);
                    }
                    ROOT_ZONE_CAPTURE | SUBSOIL_OVERFLOW if leaching && day < days => {
                        let l = &mut m.leach[day];
                        l[1] += leg(&res, SOIL_N);
                        l[2] += leg(&res, PLANT_TANK_N);
                        l[5] += leg(&res, SOIL_N);
                    }
                    _ => {}
                }
            }
            if let Some(u) = &uptake {
                let (wanted, given) = u.parts(before, &env, sc.bio_dt).expect("uptake");
                if wanted > 0.0 {
                    if let Some(r) = m.reach_ratio_min.get_mut(day) {
                        *r = r.min(u.reach(before) * sc.bio_dt / wanted);
                    }
                }
                if given < wanted {
                    if let (Some(c), Some(x)) = (m.n_capped.get_mut(day), m.n_short.get_mut(day)) {
                        *c += 1;
                        *x += wanted - given;
                    }
                    // Step 3c: capped with the window open and enough urine in the tank to cover
                    // the shortfall — the dose rule holding a starved soil where it is, not the
                    // tank running short. Step 4b: or with fertilizer enough, at any time — the
                    // level set too low, not the stores running short.
                    if let Some(d) = &dose {
                        let open = d.urine_allowed(before).expect("window");
                        let tank = if open { amount(before, URINE_TANK) } else { 0.0 };
                        if tank + amount(before, FERTILIZER) >= wanted - given
                            && (open || d.top_up.is_some())
                        {
                            if let Some(c) = m.capped_with_urine.get_mut(day) {
                                *c += 1;
                            }
                        }
                    }
                }
            }
        }
        Side::Fast => {
            let day = (fast_steps / sc.steps_per_day) as usize;
            fast_steps += 1;
            let t_ch = chamber_temperature(amount(after, CHAMBER), &chamber);
            let t_node = temperature(
                amount(after, NODE),
                thermal.heat_capacity,
                thermal.space_temperature,
            );
            m.chamber_min = m.chamber_min.min(t_ch);
            m.chamber_max = m.chamber_max.max(t_ch);
            m.node_min = m.node_min.min(t_node);
            m.node_max = m.node_max.max(t_node);
            m.co2_min = m.co2_min.min(amount(after, CARBON_POOL));
            m.co2_max = m.co2_max.max(amount(after, CARBON_POOL));
            m.o2_min = m.o2_min.min(amount(after, O2_POOL));
            m.o2_max = m.o2_max.max(amount(after, O2_POOL));
            m.store_min = m.store_min.min(amount(after, WATER_STORE));
            if let Some(x) = m.co2_store_low.get_mut(day) {
                *x = x.min(amount(after, CO2_STORE));
            }
            if tanks.is_some() {
                if let Some(x) = m.o2_tank_low.get_mut(day) {
                    *x = x.min(amount(after, O2_TANK));
                }
                if amount(after, O2_TANK) <= 0.0 {
                    m.o2_tank_empty += 1;
                }
            }
            let env = fast_r.bind(before, dt);
            let t_before = chamber_temperature(amount(before, CHAMBER), &chamber);
            let node_before = temperature(
                amount(before, NODE),
                thermal.heat_capacity,
                thermal.space_temperature,
            );
            let phase = sc.phase(before).expect("the sowing clock");
            let setpoint = sc.chamber_setpoint(phase, chamber.warm_setpoint);
            let lamp = env.get(&twin(LAMP_POWER_VAR, phase)).expect("lamp power");
            if lamp > 0.0 {
                let room = amount(before, CARBON_POOL);
                m.lit_minutes += 1;
                m.lit_co2_min = m.lit_co2_min.min(room);
                m.lit_co2_sum += room;
                if room < 1e-3 * air {
                    m.lit_below_1000 += 1;
                }
                if let Some(inj) = &injector {
                    if inj.wanted(room, lamp, dt) > amount(before, CO2_STORE) {
                        m.store_short += 1;
                    }
                }
            }
            m.co2_store_max = m.co2_store_max.max(amount(after, CO2_STORE));
            if t_before > setpoint + 0.5 {
                m.above_setpoint += 1;
                if node_before >= t_before {
                    m.cooler_blocked += 1;
                }
            }
            for f in fast.registry().flows() {
                let cap = match f.id() {
                    CHAMBER_COOLING => chamber.cooling_capacity,
                    CHAMBER_HEATER => chamber.heater_capacity,
                    _ => {
                        let res = f.evaluate(before, &env, dt).expect("fast flow");
                        tally(&mut m.water, f.id(), &res);
                        if f.id() == CREW_RESPIRATION {
                            if let Some(e) = m.exhaled.get_mut(day) {
                                *e += leg(&res, CARBON_POOL);
                            }
                        }
                        if f.id() == BURNER {
                            if let (Some(c), Some(u)) =
                                (m.burned_co2.get_mut(day), m.burn_n_caught.get_mut(day))
                            {
                                *c += leg(&res, CO2_STORE);
                                *u += leg(&res, URINE_TANK);
                            }
                        }
                        if f.id() == CREW_NITROGEN {
                            if let Some(e) = m.n_eaten.get_mut(day) {
                                *e -= leg(&res, FOOD_N);
                            }
                            if let Some(u) = m.urine_in.get_mut(day) {
                                *u += leg(&res, URINE_TANK);
                            }
                            if let Some(u) = m.marsh_in.get_mut(day) {
                                *u += leg(&res, MARSH_N);
                            }
                        }
                        if f.id() == TANK_DRAIN && leaching && day < days {
                            m.leach[day][4] += leg(&res, URINE_TANK);
                        }
                        if f.id() == MARSH && day < days {
                            // The marsh's outflow is the urine reaching the tank.
                            m.urine_in[day] += leg(&res, URINE_TANK);
                            m.marsh_out[day] += leg(&res, URINE_TANK);
                            m.marsh_c[day] -= leg(&res, CARBON_POOL);
                            m.marsh_n[day] += leg(&res, LITTER_N) + leg(&res, BURNER_FEED_N);
                        }
                        continue;
                    }
                };
                let res = f.evaluate(before, &env, dt).expect("chamber flow");
                if leg(&res, CHAMBER).abs() >= 0.999 * cap * dt {
                    if f.id() == CHAMBER_COOLING {
                        m.cooler_capped += 1;
                    } else {
                        m.heater_capped += 1;
                    }
                }
            }
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, days, &mut observe)
        .expect("two-rate run");
    // Day-end states, with the start in front: day d runs from `ends[d]` to `ends[d + 1]`.
    let mut ends = vec![first.clone()];
    ends.extend(states.iter().cloned());
    let ends = &ends[ends.len() - days - 1..];
    let mut bounds = vec![0];
    bounds.extend(reset_days.iter().copied().filter(|&d| d > 0 && d < days));
    bounds.push(days);
    bounds.dedup();
    let exhaled_by_day = &m.exhaled;
    let f_resp = params::crew().respired_carbon_fraction;
    // Harvest k happens at the re-sow that ends cycle k — at the START of cycle k + 1, whose
    // books (pre-re-sow day end to pre-re-sow day end) therefore carry it.
    let harvests = harvests.borrow().clone();
    let eaten = m.exhaled.iter().sum::<f64>() / f_resp;
    let cycles = bounds
        .windows(2)
        .enumerate()
        .map(|(i, w)| {
            let (s, e) = (&ends[w[0]], &ends[w[1]]);
            let booked = i.checked_sub(1).and_then(|k| harvests.get(k)).copied();
            let exhaled: f64 = exhaled_by_day[w[0]..w[1]].iter().sum();
            let mut best: f64 = f64::NEG_INFINITY;
            let mut grain: f64 = 0.0;
            let mut plant: f64 = 0.0;
            for d in w[0]..w[1] {
                let (x, y) = (&ends[d], &ends[d + 1]);
                let ex = exhaled_by_day[d];
                let sc = amount(y, CO2_REMOVED) - amount(x, CO2_REMOVED);
                let dp = amount(y, CARBON_POOL) - amount(x, CARBON_POOL);
                let ds = amount(y, CO2_STORE) - amount(x, CO2_STORE);
                best = best.max((ex - sc - dp - ds) / ex);
                grain = grain.max(amount(y, STORAGE_C));
                plant = plant.max(plant_c(y));
            }
            Cycle {
                days: w[1] - w[0],
                exhaled,
                scrubbed: amount(e, CO2_REMOVED) - amount(s, CO2_REMOVED),
                pool_change: amount(e, CARBON_POOL) - amount(s, CARBON_POOL),
                store_change: amount(e, CO2_STORE) - amount(s, CO2_STORE),
                store_end: amount(e, CO2_STORE),
                best_day_share: best,
                grain_max: grain,
                plant_c_max: plant,
                crop_change: held_c(e).0 - held_c(s).0,
                soil_change: held_c(e).1 - held_c(s).1,
                soil_end: held_c(e).1,
                harvested: harvests.get(i).map_or(0.0, |h| h.carbon),
                harvest_booked: i
                    .checked_sub(1)
                    .and_then(|k| harvests.get(k))
                    .map_or(0.0, |h| h.carbon),
                harvest: harvests.get(i).copied().unwrap_or_default(),
                n_change: {
                    let (x, y) = (nitrogen(s), nitrogen(e));
                    (y.0 - x.0, y.1 - x.1, y.2 - x.2, y.3 - x.3)
                },
                n_eaten: m.n_eaten[w[0]..w[1]].iter().sum(),
                imported_c: amount(s, FOOD_RESERVE) - amount(e, FOOD_RESERVE),
                imported_n: amount(s, FOOD_RESERVE_N) - amount(e, FOOD_RESERVE_N),
                urine_in: m.urine_in[w[0]..w[1]].iter().sum(),
                dosed: m.dosed[w[0]..w[1]].iter().sum(),
                tank_end: amount(e, URINE_TANK),
                uptake: m.uptake[w[0]..w[1]].iter().sum(),
                soil_n_min: ends[w[0] + 1..=w[1]]
                    .iter()
                    .map(|x| amount(x, SOIL_N))
                    .fold(f64::INFINITY, f64::min),
                soil_n_start: amount(s, SOIL_N),
                capped_with_urine: m.capped_with_urine[w[0]..w[1]].iter().sum(),
                tank_min_in_window: m.tank_min[w[0]..w[1]]
                    .iter()
                    .take(DOSE_WINDOW_DAYS.ceil() as usize)
                    .copied()
                    .fold(f64::INFINITY, f64::min),
                tank_dry_days: m.tank_min[w[0]..w[1]]
                    .iter()
                    .take(DOSE_WINDOW_DAYS.ceil() as usize)
                    .filter(|&&t| t <= 0.0)
                    .count(),
                food_store_end: amount(e, FOOD_STORE),
                reserve_end: amount(e, FOOD_RESERVE),
                soil_n_end: amount(e, SOIL_N),
                ncon_max: uptake.as_ref().map_or(0.0, |u| {
                    ends[w[0] + 1..=w[1]]
                        .iter()
                        .map(|x| 1e6 * u.concentration(x))
                        .fold(0.0, f64::max)
                }),
                capped_steps: m.n_capped[w[0]..w[1]].iter().sum(),
                n_short: m.n_short[w[0]..w[1]].iter().sum(),
                reach_ratio_min: m.reach_ratio_min[w[0]..w[1]]
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min),
                co2_store_low: m.co2_store_low[w[0]..w[1]]
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min),
                o2_tank_low: m.o2_tank_low[w[0]..w[1]]
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min),
                o2_tank_end: amount(e, O2_TANK),
                o2_tank_change: amount(e, O2_TANK) - amount(s, O2_TANK),
                n_stressed: m.n_stressed[w[0]..w[1]].iter().sum(),
                n_stressed_cold: m.n_stressed[w[0]..w[1].min(w[0] + sc.cold.days)].iter().sum(),
                n_stress_min_warm: m.n_stress_min[w[1].min(w[0] + sc.cold.days)..w[1]]
                    .iter()
                    .copied()
                    .fold(1.0, f64::min),
                n_stress_min: m.n_stress_min[w[0]..w[1]]
                    .iter()
                    .copied()
                    .fold(1.0, f64::min),
                fert_window: m.fert_window[w[0]..w[1]].iter().sum(),
                fert_late: m.fert_late[w[0]..w[1]].iter().sum(),
                fert_end: amount(e, FERTILIZER),
                urine_late: m.urine_late[w[0]..w[1]].iter().sum(),
                straw_booked: booked.map_or((0.0, 0.0), |h| (h.straw_c, h.straw_n)),
                food_burned_booked: booked.map_or(0.0, |h| h.food_burned_c),
                food_burned_n_booked: booked.map_or(0.0, |h| h.food_burned_n),
                feces_n: if burn.is_some() {
                    m.n_eaten[w[0]..w[1]].iter().sum::<f64>() * (1.0 - URINE_N_SHARE)
                } else {
                    0.0
                },
                burner_change: amount(e, BURNER_FEED) + amount(e, CHAR)
                    - amount(s, BURNER_FEED)
                    - amount(s, CHAR),
                burned_co2: m.burned_co2[w[0]..w[1]].iter().sum(),
                char_end: amount(e, CHAR),
                leaked: if burn.is_some_and(|b| b.co2_leak.is_some()) {
                    amount(e, BURNER_LEAK) - amount(s, BURNER_LEAK)
                } else {
                    0.0
                },
                n_to_gas: amount(e, N2_GAS) - amount(s, N2_GAS),
                n_caught: m.burn_n_caught[w[0]..w[1]].iter().sum(),
                marsh_in: m.marsh_in[w[0]..w[1]].iter().sum(),
                marsh_c: m.marsh_c[w[0]..w[1]].iter().sum(),
                marsh_n: m.marsh_n[w[0]..w[1]].iter().sum(),
                marsh_out: m.marsh_out[w[0]..w[1]].iter().sum(),
                marsh_end: amount(e, MARSH_N),
                leach: m.leach[w[0]..w[1]].iter().fold([0.0; 6], |mut t, d| {
                    for (x, y) in t.iter_mut().zip(d) {
                        *x += y;
                    }
                    t
                }),
                subsoil_n_end: amount(e, SUBSOIL_N),
                plant_tank_n_end: amount(e, PLANT_TANK_N),
                feces_to_soil: if food_loop {
                    exhaled * (1.0 - f_resp) / f_resp
                } else {
                    0.0
                },
            }
        })
        .collect();
    Run {
        a,
        cycles,
        m,
        first,
        last: states.last().expect("a day").clone(),
        rationed: (totals.slow_rationed, totals.fast_rationed),
        days,
        crew: sc.cabin.crew_count,
        opts,
        eaten,
    }
}

fn report(r: &Run) {
    let a = r.a;
    let r_f_resp = params::crew().respired_carbon_fraction;
    let (s, e) = (&r.first, &r.last);
    let m = &r.m;
    let o = r.opts;
    let air = sealed_station_scenario().bio.chamber_air_capacity_mol;
    println!(
        "\n=== {a} m², {} crew, {} days{}; rationed (plant, minute) {:?} ===",
        r.crew,
        r.days,
        match (o.co2_store, o.food_loop) {
            (_, true) if o.leaching => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW + GAS TANKS + TOP-UP \
                 WITH FERTILIZER + URINE AT ANY TIME + WASTES BURNED + URINE MARSH + LEACHING"
            }
            (_, true) if o.marsh.is_some() => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW + GAS TANKS + TOP-UP \
                 WITH FERTILIZER + URINE AT ANY TIME + WASTES BURNED + URINE MARSH"
            }
            (_, true) if o.burn.is_some() => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW + GAS TANKS + TOP-UP \
                 WITH FERTILIZER + URINE AT ANY TIME + WASTES BURNED"
            }
            (_, true) if o.urine_any_time => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW + GAS TANKS + TOP-UP \
                 WITH FERTILIZER + URINE AT ANY TIME"
            }
            (_, true) if o.fertilizer.is_some() => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW + GAS TANKS + TOP-UP \
                 WITH FERTILIZER"
            }
            (_, true) if o.tanks.is_some() => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW + GAS TANKS CHARGED"
            }
            (_, true) if o.feed_crew => {
                ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL + FEED THE CREW (reserve, urine tank)"
            }
            (_, true) if o.real_soil => ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL",
            (_, true) if o.with_nitrogen => ", CO₂ STORE + FOOD LOOP + NITROGEN",
            (_, true) => ", CO₂ STORE + FOOD LOOP",
            (true, false) => ", CO₂ STORE",
            _ => "",
        },
        r.rationed
    );
    println!(
        "per crop cycle (closure share = ecosystem's net CO₂ draw / crew exhalation; 'scrubbed' = \
         left the station; the store's change is not a draw):"
    );
    for (i, c) in r.cycles.iter().enumerate() {
        let net = c.exhaled - c.scrubbed - c.pool_change - c.store_change;
        println!(
            "  cycle {i}: {:4} days; exhaled {:9.1} mol ({:.2}/day), scrubbed {:9.1}; closure share \
             {:5.1} % (best day {:5.1} %); grain max {:.3} mol C/m², plant C max {:.3} mol C/m²",
            c.days,
            c.exhaled,
            c.exhaled / c.days as f64,
            c.scrubbed,
            100.0 * net / c.exhaled,
            100.0 * c.best_day_share,
            c.grain_max / a,
            c.plant_c_max / a
        );
        println!(
            "           net draw {:9.1} mol C = crop {:+9.1} + soil {:+9.1} (soil ends {:9.1}; residual {:+.3}); peak crop / exhaled {:5.1} %",
            net,
            c.crop_change,
            c.soil_change,
            c.soil_end,
            net - c.crop_change - c.soil_change - c.burner_change - c.harvest_booked
                + c.feces_to_soil
                + c.food_burned_booked
                - c.leaked,
            100.0 * c.plant_c_max / c.exhaled
        );
        if o.co2_store {
            println!(
                "           store {:+9.1} mol over the cycle, ends {:9.1}, lowest {:9.1}",
                c.store_change, c.store_end, c.co2_store_low
            );
        }
        if o.tanks.is_some() {
            println!(
                "           O₂ tank {:+9.1} mol over the cycle, ends {:9.1}, lowest {:9.1}; O₂ change ÷ \
                 −(CO₂ store change) {:.4}",
                c.o2_tank_change,
                c.o2_tank_end,
                c.o2_tank_low,
                -c.o2_tank_change / c.store_change
            );
        }
        if o.food_loop {
            println!(
                "           harvest {:9.1} mol C ({:.3} mol C/m²; {:.1} % of the crew's food over the cycle); in these books: harvest {:9.1}, feces to soil {:9.1}",
                c.harvested,
                c.harvested / a,
                100.0 * c.harvested * r_f_resp / c.exhaled,
                c.harvest_booked,
                c.feces_to_soil
            );
        }
        if let Some(b) = o.burn {
            println!(
                "           burner: in {:.1} mol C (straw {:.1}, feces {:.1}, food above the reserve \
                 {:.1}), CO₂ back to the tank {:.1}, char {:+.1} (ends {:.1}); N in {:.3} kg (straw \
                 {:.3}, feces {:.3}, food {:.3}), to N₂ {:.3}, caught {:.3} (char share {}, N lost {})",
                c.straw_booked.0 + c.feces_to_soil + c.food_burned_booked,
                c.straw_booked.0,
                c.feces_to_soil,
                c.food_burned_booked,
                c.burned_co2,
                c.burner_change,
                c.char_end,
                c.straw_booked.1 + c.feces_n + c.food_burned_n_booked,
                c.straw_booked.1,
                c.feces_n,
                c.food_burned_n_booked,
                c.n_to_gas,
                c.n_caught,
                b.char_share,
                b.n_lost
            );
            if let Some(leak) = b.co2_leak {
                println!(
                    "           burner leak: {:.1} mol C out as CO₂ (share {leak})",
                    c.leaked
                );
            }
        }
        if o.with_nitrogen {
            let h = c.harvest;
            let (dp, ds, df, dh) = c.n_change;
            println!(
                "           N: plant at maturity {:.3} kg; grain asks {:.3}, given {:.3} ({:.1} %); \
                 crew ate {:.3} kg ({:.1} g/day); change plant {:+.3} soil {:+.3} food {:+.3} \
                 held back {:+.3} (sum {:+.2e})",
                h.plant_n,
                h.wanted_n,
                h.grain_n,
                100.0 * h.grain_n / h.wanted_n.max(f64::MIN_POSITIVE),
                c.n_eaten,
                1e3 * c.n_eaten / c.days as f64,
                dp,
                ds,
                df,
                dh,
                dp + ds + df + dh
            );
        }
        if o.feed_crew {
            // The soil solution's change, split: the doses in, the roots' uptake out, and the rest
            // (the soil's own mineralization and immobilization) — what the dose does not decide.
            let change = c.soil_n_end - c.soil_n_start;
            // The urine's share of the doses: Step 4b's fertilizer is the rest (0 before it).
            let urine_dosed = c.dosed - c.fert_window - c.fert_late;
            // The diet, calories and protein side by side (plan §10.2): the share of the crew's
            // carbon that was grown, and the nitrogen they ate as protein per person-day.
            println!(
                "           diet: calories {:.1} % home-grown; protein {:.1} g per person-day (BVAD's \
                 menu 122; N × {PROTEIN_PER_N})",
                100.0 - 100.0 * c.imported_c * r_f_resp / c.exhaled,
                1e3 * c.n_eaten * PROTEIN_PER_N / c.days as f64 / r.crew
            );
            if o.fertilizer.is_some() {
                println!(
                    "           fertilizer dosed {:.3} kg in the urine window + {:.3} kg after it \
                     ({:.2} g/m² in all); store ends {:.3} kg",
                    c.fert_window,
                    c.fert_late,
                    1e3 * (c.fert_window + c.fert_late) / a,
                    c.fert_end
                );
            }
            if o.leaching {
                println!(
                    "           soil solution N {:.2} → {:.2} g/m² (lowest {:.2}): dosed {:+.3} kg, roots \
                     took {:.3} kg, the drained water {:+.3} kg, the rest (the soil's own turnover) \
                     {:+.3} kg",
                    1e3 * c.soil_n_start / a,
                    1e3 * c.soil_n_end / a,
                    1e3 * c.soil_n_min / a,
                    c.dosed,
                    c.uptake,
                    c.leach[5],
                    change - c.dosed + c.uptake - c.leach[5]
                );
                println!(
                    "           drained water: leached {:.3} kg N past the roots ([F] Eqn 18.14); the \
                     roots took back {:.3} kg from below them; the overflow sent {:.3} kg to the plants' \
                     tank, the watering brought {:.3} kg back from it, its drain {:.3} kg to the urine \
                     tank; below the roots {:.3} kg, in the plants' tank {:.4} kg at the end",
                    c.leach[0],
                    c.leach[1],
                    c.leach[2],
                    c.leach[3],
                    c.leach[4],
                    c.subsoil_n_end,
                    c.plant_tank_n_end
                );
            } else {
                println!(
                    "           soil solution N {:.2} → {:.2} g/m² (lowest {:.2}): dosed {:+.3} kg, roots took \
                     {:.3} kg, the rest (the soil's own turnover) {:+.3} kg",
                    1e3 * c.soil_n_start / a,
                    1e3 * c.soil_n_end / a,
                    1e3 * c.soil_n_min / a,
                    c.dosed,
                    c.uptake,
                    change - c.dosed + c.uptake
                );
            }
            println!(
                "           imported food eaten {:9.1} mol C ({:.1} % of the crew's food), {:.3} kg N; \
                 food store ends {:.1}, reserve {:.1} mol C",
                c.imported_c,
                100.0 * c.imported_c * r_f_resp / c.exhaled,
                c.imported_n,
                c.food_store_end,
                c.reserve_end
            );
            println!(
                "           urine tank: in {:.3} kg ({:.1} g/day), dosed {:.3} kg ({:.2} g/m²; EcoSanRes \
                 person-day dose 13–17, toxic 52–69 g/m²), ends {:.3} kg ({:.0} days of inflow)",
                c.urine_in,
                1e3 * c.urine_in / c.days as f64,
                urine_dosed,
                1e3 * urine_dosed / a,
                c.tank_end,
                c.tank_end / (c.urine_in / c.days as f64).max(f64::MIN_POSITIVE)
            );
            if let Some(mo) = o.marsh {
                println!(
                    "           marsh ({} m²): urine in {:.3} kg; plants fixed {:.1} mol C ({:.3} mol C/m²/day) \
                 and took {:.3} kg N ({:.3} g/m²/day) to the {}; outflow to the urine tank {:.3} kg \
                 (the tank's 'in' above); water holds {:.1} g at the end",
                    mo.area,
                    c.marsh_in,
                    c.marsh_c,
                    c.marsh_c / mo.area / c.days as f64,
                    c.marsh_n,
                    1e3 * c.marsh_n / mo.area / c.days as f64,
                    if mo.to_soil {
                        "soil's litter"
                    } else {
                        "burner (not in its 'in' above)"
                    },
                    c.marsh_out,
                    1e3 * c.marsh_end
                );
            }
            if o.urine_any_time {
                println!(
                    "           urine dosed after the window {:.3} kg (inside it {:.3} kg)",
                    c.urine_late,
                    urine_dosed - c.urine_late
                );
            }
            println!(
                "           tank lowest in the window {:.4} kg, empty on {} of the window's days; uptake \
                 capped with the window open and the tank able to cover the shortfall in {} plant steps",
                c.tank_min_in_window, c.tank_dry_days, c.capped_with_urine
            );
        }
        if o.real_soil {
            println!(
                "           soil solution N ends {:.4} kg ({:.2} g/m²); highest {:.0} mg/L at a day's end; \
                 uptake capped by the roots' reach in {} plant steps, {:.4} kg held back; reach ÷ wanted at least {:.2}",
                c.soil_n_end,
                1e3 * c.soil_n_end / a,
                c.ncon_max,
                c.capped_steps,
                c.n_short,
                c.reach_ratio_min
            );
            println!(
                "           nitrogen stress (factor below 1, photosynthesis cut) in {} plant steps \
                 ({} in the cold weeks), lowest factor {:.3} ({:.3} after the cold weeks)",
                c.n_stressed, c.n_stressed_cold, c.n_stress_min, c.n_stress_min_warm
            );
        }
    }
    let (ex, scr, dp, ds) = r.cycles.iter().fold((0.0, 0.0, 0.0, 0.0), |(x, y, z, w), c| {
        (x + c.exhaled, y + c.scrubbed, z + c.pool_change, w + c.store_change)
    });
    let harvested: f64 = r.cycles.iter().map(|c| c.harvested).sum();
    println!(
        "  food: the crew ate {:.1} mol C; harvested {:.1} ({:.1} %); food store {:.1} → {:.1} ({:+.2}/day)",
        r.eaten,
        harvested,
        100.0 * harvested / r.eaten,
        amount(s, FOOD_STORE),
        amount(e, FOOD_STORE),
        (amount(e, FOOD_STORE) - amount(s, FOOD_STORE)) / r.days as f64
    );
    if o.feed_crew {
        let imported = amount(s, FOOD_RESERVE) - amount(e, FOOD_RESERVE);
        println!(
            "  imported food eaten {:.1} mol C ({:.0} days of the crew's food; {:.1} % of what they \
             ate; the food store ends {:.1}), its nitrogen {:.3} kg",
            imported,
            imported / (r.eaten / r.days as f64),
            100.0 * imported / r.eaten,
            amount(e, FOOD_STORE),
            amount(s, FOOD_RESERVE_N) - amount(e, FOOD_RESERVE_N)
        );
        let solution = |x: &State| amount(x, SOIL_N);
        let organic = |x: &State| {
            [LITTER_N, MICROBIAL_N, HUMUS_N]
                .iter()
                .map(|id| amount(x, id))
                .sum::<f64>()
        };
        println!(
            "  where the nitrogen went (kg N, start → end): plant {:.3} → {:.3}; soil solution {:.3} → \
             {:.3}; soil litter, microbes and humus {:.3} → {:.3}; food store {:.3} → {:.3}; urine tank \
             {:.3} → {:.3}; reserve {:.3} → {:.3}; fertilizer {:.3} → {:.3}",
            amount(s, PLANT_N),
            amount(e, PLANT_N),
            solution(s),
            solution(e),
            organic(s),
            organic(e),
            amount(s, FOOD_N),
            amount(e, FOOD_N),
            amount(s, URINE_TANK),
            amount(e, URINE_TANK),
            amount(s, FOOD_RESERVE_N),
            amount(e, FOOD_RESERVE_N),
            amount(s, FERTILIZER),
            amount(e, FERTILIZER)
        );
        if let Some(mo) = o.marsh {
            let (n_in, c_fixed, n_taken, out) = r.cycles.iter().fold((0.0, 0.0, 0.0, 0.0), |t, c| {
                (t.0 + c.marsh_in, t.1 + c.marsh_c, t.2 + c.marsh_n, t.3 + c.marsh_out)
            });
            println!(
                "  marsh over the run ({} m², growth {} g DM/m²/day, plant N {}, residence {} days, \
                 plants to the {}): urine in {:.3} kg N; plants fixed {:.1} mol C and took {:.3} kg N; \
                 outflow {:.3} kg N; water {:.1} → {:.1} g N; its lamp at Biosphere 2's {:.0} W/m² \
                 would draw {:.1} kWh/day (NOT drawn: a named gap)",
                mo.area,
                mo.growth,
                mo.plant_n,
                mo.residence_days,
                if mo.to_soil { "soil" } else { "burner" },
                n_in,
                c_fixed,
                n_taken,
                out,
                1e3 * amount(s, MARSH_N),
                1e3 * amount(e, MARSH_N),
                B2_MARSH_LAMP_W_PER_M2,
                B2_MARSH_LAMP_W_PER_M2 * mo.area * 24.0 / 1e3
            );
        }
        if o.leaching {
            let t = r.cycles.iter().fold([0.0; 6], |mut t, c| {
                for (x, y) in t.iter_mut().zip(c.leach) {
                    *x += y;
                }
                t
            });
            println!(
                "  drained water over the run (kg N): leached {:.3} past the roots; the roots took back \
                 {:.3}; the overflow sent {:.3} to the plants' tank, the watering brought {:.3} back, \
                 its drain {:.3} to the urine tank; below the roots {:.3} → {:.3}, in the plants' tank \
                 {:.4} → {:.4}",
                t[0],
                t[1],
                t[2],
                t[3],
                t[4],
                amount(s, SUBSOIL_N),
                amount(e, SUBSOIL_N),
                amount(s, PLANT_TANK_N),
                amount(e, PLANT_TANK_N)
            );
        }
        if o.burn.is_some() {
            let (in_c, back) = r.cycles.iter().fold((0.0, 0.0), |(x, y), c| {
                (
                    x + c.straw_booked.0 + c.feces_to_soil + c.food_burned_booked,
                    y + c.burned_co2,
                )
            });
            println!(
                "  burner over the run: in {in_c:.1} mol C, CO₂ back {back:.1}, char {:.1}, feed left \
                 {:.1}; N₂ {:.3} kg; food burned {:.1} mol C",
                amount(e, CHAR),
                amount(e, BURNER_FEED),
                amount(e, N2_GAS),
                r.cycles.iter().map(|c| c.food_burned_booked).sum::<f64>()
            );
            if o.burn.is_some_and(|b| b.co2_leak.is_some()) {
                println!(
                    "  burner leak over the run: {:.1} mol C out as CO₂",
                    amount(e, BURNER_LEAK)
                );
            }
        }
    }
    println!(
        "  whole run: exhaled {ex:.1} mol; closure share {:.1} %; left the station {:.1} %; into \
         the store {:.1} %",
        100.0 * (ex - scr - dp - ds) / ex,
        100.0 * scr / ex,
        100.0 * ds / ex
    );
    println!(
        "lit minutes {}: room CO₂ min {:.2} mol ({:.0} ppm), mean {:.0} ppm; minutes below 1000 \
         ppm {}; store peak {:.1} mol, lit minutes the store ran short {}",
        m.lit_minutes,
        m.lit_co2_min,
        1e6 * m.lit_co2_min / air,
        1e6 * m.lit_co2_sum / m.lit_minutes.max(1) as f64 / air,
        m.lit_below_1000,
        m.co2_store_max,
        m.store_short
    );
    println!(
        "room CO₂ {:.2}–{:.2} mol ({:.0}–{:.0} ppm of {air} mol); O₂ {:.1}–{:.1} mol ({:.2}–{:.2} %)",
        m.co2_min,
        m.co2_max,
        1e6 * m.co2_min / air,
        1e6 * m.co2_max / air,
        m.o2_min,
        m.o2_max,
        100.0 * m.o2_min / air,
        100.0 * m.o2_max / air
    );
    println!(
        "O₂ supply change {:+.1} mol (negative: made up from the tank; positive: surplus returned)",
        amount(e, O2_SUPPLY) - amount(s, O2_SUPPLY)
    );
    if o.tanks.is_some() {
        let low = |v: &[f64]| {
            v.iter()
                .copied()
                .enumerate()
                .fold((f64::INFINITY, 0), |b, (d, x)| if x < b.0 { (x, d) } else { b })
        };
        let (co2_low, co2_day) = low(&m.co2_store_low);
        let (o2_low, o2_day) = low(&m.o2_tank_low);
        let (dc, do2) = (
            amount(e, CO2_STORE) - amount(s, CO2_STORE),
            amount(e, O2_TANK) - amount(s, O2_TANK),
        );
        println!(
            "gas tanks: CO₂ {:.1} → {:.1} mol (lowest {:.1} on day {co2_day}); O₂ {:.1} → {:.1} mol \
             (lowest {:.1} on day {o2_day}; minutes empty {}); O₂ change ÷ −(CO₂ change) {:.4}",
            amount(s, CO2_STORE),
            amount(e, CO2_STORE),
            co2_low,
            amount(s, O2_TANK),
            amount(e, O2_TANK),
            o2_low,
            m.o2_tank_empty,
            -do2 / dc
        );
    }
    println!(
        "water: crew store {:.1} → {:.1} kg (min {:.1}); plants' water {:.1} → {:.1} kg",
        amount(s, WATER_STORE),
        amount(e, WATER_STORE),
        m.store_min,
        plant_water(s),
        plant_water(e)
    );
    println!("water moved, by flow and stock (kg over the run; + into the stock):");
    for ((id, stock), v) in &m.water {
        println!(
            "  {id:36} {stock:26} {v:+14.1} ({:+.2} kg/day)",
            v / r.days as f64
        );
    }
    let drawn = amount(s, BATTERY) - amount(e, BATTERY) + amount(s, GRID) - amount(e, GRID);
    println!(
        "electricity drawn {:.3e} J = {:.1} kWh/day",
        drawn,
        drawn / 3.6e6 / r.days as f64
    );
    println!(
        "chamber {:.2}–{:.2} °C; structure {:.1}–{:.1} K; minutes above setpoint+0.5 K {}, of them \
         cooler blocked (structure not colder) {}; cooler at cap {}; heater at cap {}",
        m.chamber_min - 273.15,
        m.chamber_max - 273.15,
        m.node_min,
        m.node_max,
        m.above_setpoint,
        m.cooler_blocked,
        m.cooler_capped,
        m.heater_capped
    );
    println!(
        "end: plant C {:.3} mol/m²; food store {:.1} mol; tank {:.1} kg; scrubbed total {:.1} mol",
        plant_c(e) / a,
        amount(e, FOOD_STORE),
        amount(e, CONDENSATE),
        amount(e, CO2_REMOVED) - amount(s, CO2_REMOVED)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.iter()
            .position(|x| x == name)
            .and_then(|i| args.get(i + 1))
    };
    if args.iter().any(|x| x == "--control") {
        control();
        return;
    }
    let sc = sealed_station_scenario();
    let days = arg("--days").map_or(sc.days(), |d| d.parse().expect("--days N"));
    let flag = |name: &str| args.iter().any(|x| x == name);
    let opts = Options {
        co2_store: flag("--co2-store"),
        food_loop: flag("--food-loop"),
        with_nitrogen: flag("--nitrogen"),
        real_soil: flag("--real-soil"),
        feed_crew: flag("--feed-crew"),
        tanks: match (arg("--co2-charge"), arg("--o2-charge")) {
            (Some(c), Some(o)) => Some((
                c.parse().expect("--co2-charge MOL"),
                o.parse().expect("--o2-charge MOL"),
            )),
            (None, None) => None,
            _ => panic!("Step 4a's tanks take both charges: --co2-charge MOL --o2-charge MOL"),
        },
        fertilizer: match (arg("--fertilizer"), arg("--soil-n-level")) {
            (Some(f), Some(l)) => Some((
                f.parse().expect("--fertilizer KG"),
                l.parse().expect("--soil-n-level G"),
            )),
            (None, None) => None,
            _ => panic!("Step 4b takes both: --fertilizer KG --soil-n-level G"),
        },
        urine_any_time: flag("--urine-any-time"),
        burn: flag("--burn-wastes").then(|| {
            let need = |name: &str| -> f64 {
                arg(name)
                    .unwrap_or_else(|| panic!("Step 4c takes {name} (no default)"))
                    .parse()
                    .unwrap_or_else(|_| panic!("{name} F"))
            };
            Burn {
                char_share: need("--burn-char-share"),
                n_lost: need("--burn-n-lost"),
                reserve_days: need("--food-reserve-days"),
                co2_leak: arg("--burn-co2-leak").map(|x| {
                    let leak: f64 = x.parse().expect("--burn-co2-leak F");
                    let char_share = need("--burn-char-share");
                    assert!(
                        (0.0..=1.0 - char_share).contains(&leak),
                        "--burn-co2-leak F: 0 to 1 − char share"
                    );
                    leak
                }),
            }
        }),
        marsh: arg("--marsh-area").map(|x| {
            let need = |name: &str| -> f64 {
                arg(name)
                    .unwrap_or_else(|| panic!("Step 4d-1 takes {name} (no default)"))
                    .parse()
                    .unwrap_or_else(|_| panic!("{name} X"))
            };
            let o = MarshOpts {
                area: x.parse().expect("--marsh-area M2"),
                growth: need("--marsh-growth"),
                plant_n: need("--marsh-plant-n"),
                residence_days: need("--marsh-residence-days"),
                to_soil: match arg("--marsh-plants-to").map(String::as_str) {
                    Some("soil") => true,
                    Some("burner") => false,
                    _ => panic!("Step 4d-1 takes --marsh-plants-to soil|burner (no default)"),
                },
            };
            assert!(
                o.area > 0.0 && o.growth >= 0.0 && o.residence_days > 0.0,
                "--marsh-area > 0, --marsh-growth >= 0, --marsh-residence-days > 0"
            );
            assert!((0.0..1.0).contains(&o.plant_n), "--marsh-plant-n: 0 to 1");
            o
        }),
        leaching: flag("--leaching"),
    };
    // Step 3c's farm (plan §9): 700 m², the round area above the 658 m² that feeds four crew.
    let default_area = if opts.closed().feed_crew { 700.0 } else { 200.0 };
    let area: f64 = arg("--area").map_or(default_area, |x| x.parse().expect("--area A"));
    let crew: Option<f64> = arg("--crew").map(|x| x.parse().expect("--crew N"));
    let big = sized(area);
    println!(
        "sizing at {area} m²: lamp {} W, chamber C {:.3e} J/K, cooler {} W, heater {} W, walls \
         {:.1} m², tank start {:.1} kg (working level {:.1} kg); crew {}",
        big.lamp_power_w,
        big.chamber.heat_capacity,
        big.chamber.cooling_capacity,
        big.chamber.heater_capacity,
        big.chamber.wall_area,
        big.bio.condensate0,
        working_level(&big.bio, &big.plant_params()),
        big.cabin.crew_count
    );
    let (one, many) = std::thread::scope(|s| {
        let one = s.spawn(|| run(1.0, None, days, Options::default()));
        let many = s.spawn(|| run(area, crew, days, opts));
        (one.join().expect("1 m²"), many.join().expect("big"))
    });
    report(&one);
    report(&many);
}
