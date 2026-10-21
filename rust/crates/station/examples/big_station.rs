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
use domains::biosphere::science::{root_zone_fraction, WATER_DENSITY};
use domains::biosphere::stocks::{
    CARBON_POOL, CONDENSATE, CONSUMER_CARBON, HUMUS_CARBON, HUMUS_N, LEAF_C, LITTER_CARBON,
    LITTER_N, LITTER_SINK, MICROBIAL_CARBON, MICROBIAL_N, O2_POOL, PLANT_N, ROOTED_DEPTH, ROOT_C,
    SOIL_N, SOIL_WATER, STEM_C, STEM_RESERVE_C, STORAGE_C, SUBSOIL_WATER, WATER_VAPOR,
};
use std::cell::RefCell;
use std::rc::Rc;

use domains::biosphere::system::SeasonScenario;
use domains::crew::{FOOD_INTAKE_VAR, FOOD_STORE, WATER_STORE};
use domains::eclss::{CO2Scrubber, CO2_REMOVED, CO2_SCRUBBER, O2_SUPPLY};
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
use station::stocks::{co2_composition, gas_pool, simple_pool};
use station::water_reserve::working_level;

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

/// NITROGEN `lab.food_n → soil_n + litter_n`: the crew eat nitrogen with their carbon, at the food
/// store's average N:C, and pass all of it (an adult is in nitrogen balance): urine to the soil's
/// available nitrogen, feces with their carbon to the litter.
struct CrewNitrogen;

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
            Leg::new(SOIL_N.to_string(), urine)?,
            Leg::new(LITTER_N.to_string(), eaten - urine)?,
        ])
    }
}

/// The station with Step 2c's food nitrogen: [`FOOD_N`] at the imported food's N:C, and
/// [`CrewNitrogen`].
fn with_food_nitrogen(
    sc: &SealedStationScenario,
    state: State,
    fast: Registry,
) -> (State, Registry) {
    let mut stocks = state.stocks.clone();
    let start = sc.cabin.food_store0 * imported_n_per_mol_c(sc);
    let food_n = simple_pool(FOOD_N, "lab", Quantity::Nitrogen, start).expect("food nitrogen");
    stocks.insert(food_n.id.clone(), food_n);
    let state = State::new(state.n, stocks, state.rng_seed, state.aux.clone()).expect("state");
    let (mut flows, aux) = fast.into_parts();
    flows.push(Box::new(CrewNitrogen));
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
/// reference's reads no environment, so its sowing-clock and chamber wrappers change nothing).
fn with_real_soil(sc: &SealedStationScenario, state: &State, bio: Registry) -> Registry {
    let (flows, aux) = bio.into_parts();
    let before = flows.len();
    let mut flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .filter(|f| f.id() != NITROGEN_UPTAKE)
        .collect();
    assert_eq!(flows.len() + 1, before, "one reference uptake to replace");
    flows.push(Box::new(SolutionUptake::new(sc)));
    Registry::new(flows, &state.stocks, aux).expect("plant registry")
}

/// The nitrogen the run books (kg N): the plant's, the soil's (available, litter, microbes, humus)
/// and the food's.
fn nitrogen(s: &State) -> (f64, f64, f64) {
    let soil = [SOIL_N, LITTER_N, MICROBIAL_N, HUMUS_N]
        .iter()
        .map(|id| amount(s, id))
        .sum();
    (amount(s, PLANT_N), soil, amount(s, FOOD_N))
}

/// The station with Step 2a's CO₂ store: the scrubber re-pointed at [`CO2_STORE`] (starting
/// empty), and [`Co2Injector`].
fn with_co2_store(
    sc: &SealedStationScenario,
    state: State,
    fast: Registry,
) -> (State, Registry, Co2Injector) {
    let eclss = params::eclss();
    let air = sc.bio.chamber_air_capacity_mol;
    let mut stocks = state.stocks.clone();
    let store = gas_pool(CO2_STORE, Quantity::Carbon, 0.0, co2_composition()).expect("store");
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
fn harvest_at_maturity(
    sc: &SealedStationScenario,
    harvests: Rc<RefCell<Vec<Harvested>>>,
    with_nitrogen: bool,
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
        // What is left of the grain is exactly the seed: `reset_crop` refuses a grain below it.
        set(STORAGE_C, if eaten > 0.0 { seed } else { grain })?;
        set(FOOD_STORE, amount(current, FOOD_STORE) + eaten)?;
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
            set(PLANT_N, h.plant_n - h.grain_n)?;
            set(FOOD_N, amount(current, FOOD_N) + h.grain_n)?;
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
    n_change: (f64, f64, f64),
    n_eaten: f64,
    /// Step 3a: the soil's solution nitrogen at the cycle's end (kg N), the highest concentration at
    /// a day's end (mg/L), and the plant steps the roots' reach held the uptake back, by how much (kg N).
    soil_n_end: f64,
    ncon_max: f64,
    capped_steps: u64,
    n_short: f64,
    /// The smallest reach ÷ wanted over the cycle's plant steps with any uptake wanted (∞ if none):
    /// below 1 the cap binds; the margin it ran with otherwise.
    reach_ratio_min: f64,
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
    co2_store: bool,
    food_loop: bool,
    with_nitrogen: bool,
    real_soil: bool,
    /// The crew's food intake over the run (mol C).
    eaten: f64,
}

fn run(
    a: f64,
    crew: Option<f64>,
    days: usize,
    co2_store: bool,
    food_loop: bool,
    with_nitrogen: bool,
    real_soil: bool,
) -> Run {
    let with_nitrogen = with_nitrogen || real_soil;
    let food_loop = food_loop || with_nitrogen;
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
    let chamber = sc.chamber;
    let (state, bio, fast) = on_grid(&sc, &thermal, food_loop);
    let uptake = real_soil.then(|| SolutionUptake::new(&sc));
    let bio = if real_soil {
        with_real_soil(&sc, &state, bio)
    } else {
        bio
    };
    let (state, fast, injector) = if co2_store || food_loop {
        let (s, f, i) = with_co2_store(&sc, state, fast);
        (s, f, Some(i))
    } else {
        (state, fast, None)
    };
    let (state, fast) = if with_nitrogen {
        with_food_nitrogen(&sc, state, fast)
    } else {
        (state, fast)
    };
    let air = sc.bio.chamber_air_capacity_mol;
    let first = state.clone();
    let bio_r = sealed_bio_resolver(&sp::lamp(), &sc).expect("bio resolver");
    let fast_r = sealed_fast_resolver(&params::charge(), &params::crew(), &sc).expect("fast");
    let harvests: Rc<RefCell<Vec<Harvested>>> = Rc::default();
    let reset = if food_loop {
        harvest_at_maturity(&sc, Rc::clone(&harvests), with_nitrogen)
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
        ..Minutes::default()
    };
    // The crop's cycle ends at each re-sow: the day index of each reset.
    let mut fast_steps: u64 = 0;
    let mut reset_days: Vec<usize> = Vec::new();
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => reset_days.push((fast_steps / sc.steps_per_day) as usize),
        Side::Slow => {
            let env = bio_r.bind(before, sc.bio_dt);
            for f in bio.registry().flows() {
                let res = f.evaluate(before, &env, sc.bio_dt).expect("plant flow");
                tally(&mut m.water, f.id(), &res);
            }
            if let Some(u) = &uptake {
                let day = (fast_steps / sc.steps_per_day) as usize;
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
                        if f.id() == CREW_NITROGEN {
                            if let Some(e) = m.n_eaten.get_mut(day) {
                                *e -= leg(&res, FOOD_N);
                            }
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
                    (y.0 - x.0, y.1 - x.1, y.2 - x.2)
                },
                n_eaten: m.n_eaten[w[0]..w[1]].iter().sum(),
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
        co2_store: co2_store || food_loop,
        food_loop,
        with_nitrogen,
        real_soil,
        eaten,
    }
}

fn report(r: &Run) {
    let a = r.a;
    let r_f_resp = params::crew().respired_carbon_fraction;
    let (s, e) = (&r.first, &r.last);
    let m = &r.m;
    let air = sealed_station_scenario().bio.chamber_air_capacity_mol;
    println!(
        "\n=== {a} m², {} crew, {} days{}; rationed (plant, minute) {:?} ===",
        r.crew,
        r.days,
        match (r.co2_store, r.food_loop) {
            (_, true) if r.real_soil => ", CO₂ STORE + FOOD LOOP + NITROGEN + REAL SOIL",
            (_, true) if r.with_nitrogen => ", CO₂ STORE + FOOD LOOP + NITROGEN",
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
            net - c.crop_change - c.soil_change - c.harvest_booked + c.feces_to_soil,
            100.0 * c.plant_c_max / c.exhaled
        );
        if r.co2_store {
            println!(
                "           store {:+9.1} mol over the cycle, ends {:9.1}",
                c.store_change, c.store_end
            );
        }
        if r.food_loop {
            println!(
                "           harvest {:9.1} mol C ({:.3} mol C/m²; {:.1} % of the crew's food over the cycle); in these books: harvest {:9.1}, feces to soil {:9.1}",
                c.harvested,
                c.harvested / a,
                100.0 * c.harvested * r_f_resp / c.exhaled,
                c.harvest_booked,
                c.feces_to_soil
            );
        }
        if r.with_nitrogen {
            let h = c.harvest;
            let (dp, ds, df) = c.n_change;
            println!(
                "           N: plant at maturity {:.3} kg; grain asks {:.3}, given {:.3} ({:.1} %); \
                 crew ate {:.3} kg ({:.1} g/day); change plant {:+.3} soil {:+.3} food {:+.3} \
                 (sum {:+.2e})",
                h.plant_n,
                h.wanted_n,
                h.grain_n,
                100.0 * h.grain_n / h.wanted_n.max(f64::MIN_POSITIVE),
                c.n_eaten,
                1e3 * c.n_eaten / c.days as f64,
                dp,
                ds,
                df,
                dp + ds + df
            );
        }
        if r.real_soil {
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
    let area: f64 = arg("--area").map_or(200.0, |x| x.parse().expect("--area A"));
    let crew: Option<f64> = arg("--crew").map(|x| x.parse().expect("--crew N"));
    let co2_store = args.iter().any(|x| x == "--co2-store");
    let food_loop = args.iter().any(|x| x == "--food-loop");
    let with_nitrogen = args.iter().any(|x| x == "--nitrogen");
    let real_soil = args.iter().any(|x| x == "--real-soil");
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
        let one = s.spawn(|| run(1.0, None, days, false, false, false, false));
        let many =
            s.spawn(|| run(area, crew, days, co2_store, food_loop, with_nitrogen, real_soil));
        (one.join().expect("1 m²"), many.join().expect("big"))
    });
    report(&one);
    report(&many);
}
