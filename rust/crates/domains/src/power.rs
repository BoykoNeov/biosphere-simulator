//! The standalone Power domain — the Rust port of `domains.power` (Phase-7 P7.3).
//!
//! ENERGY (J) only. Two forced flows ([`SolarCharge`] 3-leg with heat closure,
//! [`LoadDraw`] 2-leg dissipative) plus an opt-in donor-controlled [`SelfDischarge`].
//! The load is **derived** for exact daily balance ([`balanced_load_w`]) — ported, not
//! smuggled — off the discrete half-sine [`solar_schedule`]. Power is **Tier-2** (the
//! `sin` in the schedule), so bit-exactness is not required; the arithmetic is still
//! mirrored faithfully so the measured band stays tiny.

use std::collections::{BTreeMap, HashMap};

use simcore::boundary;
use simcore::environment::{constant, Environment, Schedule, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

// --- stock ids + forcing vars + flow ids (ASCII; str sort == Rust byte sort, #15) ---
/// The Power domain id (only `power.battery` carries it; reservoirs are `boundary`).
pub const POWER_DOMAIN: &str = "power";
/// The stored-electrical-energy POOL id.
pub const BATTERY: &str = "power.battery";
/// The unclamped solar supply BOUNDARY source id.
pub const SOLAR_SOURCE: &str = "boundary.solar_source";
/// The monotonic waste-heat BOUNDARY sink id.
pub const WASTE_HEAT: &str = "boundary.waste_heat";
/// Forcing var: instantaneous solar electrical supply (W).
pub const SOLAR_POWER_VAR: &str = "solar_power";
/// Forcing var: instantaneous dissipative load demand (W).
pub const LOAD_POWER_VAR: &str = "load_power";
/// Flow id: the solar charger.
pub const SOLAR_CHARGE: &str = "power.solar_charge";
/// Flow id: the dissipative load.
pub const LOAD_DRAW: &str = "power.load_draw";
/// Flow id: the first-order self-discharge leak.
pub const SELF_DISCHARGE: &str = "power.self_discharge";

/// The one-way charge efficiency η_c (`charge.yaml`), stored/supplied, ∈ (0, 1].
#[derive(Debug, Clone, Copy)]
pub struct ChargeParams {
    /// η_c — one-way charge efficiency (dimensionless). NOT round-trip.
    pub charge_efficiency: f64,
}

/// The first-order self-discharge rate k (`self_discharge.yaml`, 1/s), ≥ 0.
#[derive(Debug, Clone, Copy)]
pub struct SelfDischargeParams {
    /// k — self-discharge rate battery → waste_heat (1/s).
    pub self_discharge_rate: f64,
}

/// Power scenario data (sizing / forcing shape; not the charge coefficient).
#[derive(Debug, Clone, Copy)]
pub struct PowerScenario {
    /// Initial battery state of charge (J).
    pub battery0: f64,
    /// Diurnal peak solar electrical supply (W).
    pub solar_peak_w: f64,
    /// Load sizing as a dimensionless fraction of daily *stored* solar (1.0 ⇒ balance).
    pub load_fraction: f64,
    /// Daylight window length (h), centred at solar noon.
    pub daylight_hours: f64,
    /// Integration step (s).
    pub dt_seconds: f64,
    /// Steps per 24 h day.
    pub steps_per_day: u64,
}

/// The standalone validation scenario (`BOUNDED_SOC_SCENARIO`): a daily-balanced
/// microgrid whose SOC oscillates within a bounded band and returns each day.
pub const BOUNDED_SOC_SCENARIO: PowerScenario = PowerScenario {
    battery0: 2.0e7,
    solar_peak_w: 1000.0,
    load_fraction: 1.0,
    daylight_hours: 12.0,
    dt_seconds: 3600.0,
    steps_per_day: 24,
};

/// The `BOUNDED_SOC` horizon in days (168 hourly steps).
pub const BOUNDED_SOC_DAYS: u64 = 7;
/// The self-discharge horizon in days (336 hourly steps; reuses `BOUNDED_SOC_SCENARIO`).
pub const SELF_DISCHARGE_DAYS: u64 = 14;

/// Split supplied energy into `(stored, lost_to_heat)` (J). Op-order mirrors Python
/// `charge_split`: `stored = η·supply`, `lost = (1 − η)·supply` (NOT `supply − stored`).
fn charge_split(supply_joules: f64, charge_efficiency: f64) -> (f64, f64) {
    let stored = charge_efficiency * supply_joules;
    let lost = (1.0 - charge_efficiency) * supply_joules;
    (stored, lost)
}

/// First-order self-discharge leak `k · battery` (W). A flow multiplies by dt.
fn self_discharge_flux(battery_joules: f64, self_discharge_rate: f64) -> f64 {
    self_discharge_rate * battery_joules
}

/// Read a donor stock's amount, mirroring Python `snapshot.stocks[id].amount` (a
/// missing stock is a referential-integrity error, Python's `KeyError`).
fn donor_amount(snapshot: &State, id: &str) -> Result<f64, SimError> {
    snapshot
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("flow reads unknown stock {id:?}")))
}

/// ENERGY flow `solar_source → battery (+η_c) + waste_heat (+(1−η_c))` (forced, 3-leg).
pub struct SolarCharge {
    id: String,
    solar_source: String,
    battery: String,
    waste_heat: String,
    params: ChargeParams,
}

impl SolarCharge {
    /// Construct a `SolarCharge` with the given ids — the station re-points `waste_heat`
    /// at `thermal.node` (the Step-1 inward heat seam); standalone Power builds it with
    /// `waste_heat = WASTE_HEAT` internally.
    pub fn new(
        id: String,
        solar_source: String,
        battery: String,
        waste_heat: String,
        params: ChargeParams,
    ) -> Self {
        SolarCharge {
            id,
            solar_source,
            battery,
            waste_heat,
            params,
        }
    }
}

impl Flow for SolarCharge {
    fn type_name(&self) -> &'static str {
        "SolarCharge"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let supply = env.get(SOLAR_POWER_VAR)? * dt;
        let (stored, lost) = charge_split(supply, self.params.charge_efficiency);
        FlowResult::new(vec![
            Leg::new(self.solar_source.clone(), -supply)?,
            Leg::new(self.battery.clone(), stored)?,
            Leg::new(self.waste_heat.clone(), lost)?,
        ])
    }
}

/// ENERGY flow `battery → waste_heat` — the 100%-dissipative forced load (2-leg).
pub struct LoadDraw {
    id: String,
    battery: String,
    waste_heat: String,
}

impl LoadDraw {
    /// Construct a `LoadDraw` with the given ids — the station re-points `waste_heat` at
    /// `thermal.node` (the Step-1 inward heat seam).
    pub fn new(id: String, battery: String, waste_heat: String) -> Self {
        LoadDraw {
            id,
            battery,
            waste_heat,
        }
    }
}

impl Flow for LoadDraw {
    fn type_name(&self) -> &'static str {
        "LoadDraw"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let draw = env.get(LOAD_POWER_VAR)? * dt;
        FlowResult::new(vec![
            Leg::new(self.battery.clone(), -draw)?,
            Leg::new(self.waste_heat.clone(), draw)?,
        ])
    }
}

/// ENERGY flow `battery → waste_heat` — the first-order standing leak (donor-controlled).
pub struct SelfDischarge {
    id: String,
    battery: String,
    waste_heat: String,
    params: SelfDischargeParams,
}

impl SelfDischarge {
    /// Construct a `SelfDischarge` with the given ids — the station/palette builder re-points
    /// `waste_heat` at `thermal.node` when a radiator is present (the dissipation seam), else
    /// the boundary sink. Additive over [`build_power`]'s internal struct-literal construction
    /// (which stays untouched, so the frozen sibling goldens can't move).
    pub fn new(
        id: String,
        battery: String,
        waste_heat: String,
        params: SelfDischargeParams,
    ) -> Self {
        SelfDischarge {
            id,
            battery,
            waste_heat,
            params,
        }
    }
}

impl Flow for SelfDischarge {
    fn type_name(&self) -> &'static str {
        "SelfDischarge"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        // Op-order mirrors Python: self_discharge_flux(battery) * dt = (k * battery) * dt.
        let leak = self_discharge_flux(
            donor_amount(snapshot, &self.battery)?,
            self.params.self_discharge_rate,
        ) * dt;
        FlowResult::new(vec![
            Leg::new(self.battery.clone(), -leak)?,
            Leg::new(self.waste_heat.clone(), leak)?,
        ])
    }
}

/// The stored-electrical-energy POOL `power.battery` (ENERGY, J).
pub fn battery_stock(amount: f64) -> Result<Stock, SimError> {
    Stock::new(
        BATTERY.to_string(),
        POWER_DOMAIN.to_string(),
        Quantity::Energy,
        Quantity::Energy.canonical_unit(),
        amount,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
}

/// Assemble the standalone Power system's initial `State` and flow `Registry`.
///
/// `self_discharge` is the opt-in third flow (`None` ⇒ the two-forced-flow build).
pub fn build_power(
    charge: &ChargeParams,
    scenario: &PowerScenario,
    self_discharge: Option<SelfDischargeParams>,
) -> Result<(State, Registry), SimError> {
    let battery = battery_stock(scenario.battery0)?;
    let solar_source = boundary::source(SOLAR_SOURCE.to_string(), Quantity::Energy, 0.0, true)?;
    let waste_heat = boundary::sink(WASTE_HEAT.to_string(), Quantity::Energy, 0.0)?;
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for s in [battery, solar_source, waste_heat] {
        stocks.insert(s.id.clone(), s);
    }
    let state = State::new(0, stocks.clone(), 0, BTreeMap::new())?;
    let mut flows: Vec<Box<dyn Flow>> = vec![
        Box::new(SolarCharge {
            id: SOLAR_CHARGE.to_string(),
            solar_source: SOLAR_SOURCE.to_string(),
            battery: BATTERY.to_string(),
            waste_heat: WASTE_HEAT.to_string(),
            params: *charge,
        }),
        Box::new(LoadDraw {
            id: LOAD_DRAW.to_string(),
            battery: BATTERY.to_string(),
            waste_heat: WASTE_HEAT.to_string(),
        }),
    ];
    if let Some(sd) = self_discharge {
        flows.push(Box::new(SelfDischarge {
            id: SELF_DISCHARGE.to_string(),
            battery: BATTERY.to_string(),
            waste_heat: WASTE_HEAT.to_string(),
            params: sd,
        }));
    }
    let registry = Registry::flows_only(flows, &stocks)?;
    Ok((state, registry))
}

/// The diurnal solar forcing (W): a half-sine over the daylight window, 0 at night.
///
/// Op-order mirrors Python `solar_schedule`: `phase = (n mod spd)/spd`, and over
/// `[sunrise, sunset)` the supply is `peak · sin(π·(phase − sunrise)/daylight_fraction)`.
/// The `sin` is what makes Power Tier-2 (`f64::sin` ≈ CPython `math.sin` to a measured
/// band, not bit-exact).
pub fn solar_schedule(scenario: &PowerScenario) -> Schedule {
    let spd = scenario.steps_per_day;
    let peak = scenario.solar_peak_w;
    let daylight_fraction = scenario.daylight_hours / 24.0;
    let sunrise = 0.5 - daylight_fraction / 2.0;
    let sunset = 0.5 + daylight_fraction / 2.0;
    Box::new(move |n: u64, _dt: f64| {
        let phase = (n % spd) as f64 / spd as f64;
        if sunrise <= phase && phase < sunset {
            peak * (std::f64::consts::PI * (phase - sunrise) / daylight_fraction).sin()
        } else {
            0.0
        }
    })
}

/// The discrete solar energy supplied over one day (J): `Σ_day solar(n)·dt`, accumulated
/// naively in canonical step order (0 → spd−1).
///
/// ⚠⚠ **This does NOT mirror the Python checker's `sum(...)`, and the comment here said it
/// did until 2026-08-27.** CPython's builtin `sum` has used **Neumaier compensation** for
/// float arguments since 3.12 — it is not a left-to-right accumulation and agrees with
/// `math.fsum`. The two reductions happen to produce identical bits on the frozen scenario,
/// which is why every golden matches and nothing ever caught it; they diverge by one ULP as
/// soon as the summands move. `ulp_probe` found it by measuring the Tier-2 band basis and
/// reading exactly half of the Python instrument's number, and reproduced the Python figure
/// **bit for bit** by compensating this one sum.
///
/// The naive accumulation is what the *reference* computes, so it is the definition and it
/// stays; what was wrong was the claim about the other port. ⚠ The general point outlives
/// this function: a compensated reduction is a **second** source of cross-port divergence,
/// independent of libm, and the tolerance contract's rationale names only transcendentals.
/// Five more `sum()` calls over floats live in the retiring Python tree — the drift
/// regression and the perennial re-sow among them.
pub fn daily_solar_energy(scenario: &PowerScenario) -> f64 {
    let solar = solar_schedule(scenario);
    let dt = scenario.dt_seconds;
    let mut total = 0.0;
    for n in 0..scenario.steps_per_day {
        total += solar(n, dt) * dt;
    }
    total
}

/// The forced load (W) balancing `load_fraction` of the daily *stored* solar.
///
/// Op-order mirrors Python `balanced_load_w`: `(load_fraction · stored_per_day) /
/// day_seconds`, with `day_seconds = steps_per_day · dt_seconds`. This is the derivation
/// the plan says to PORT (not emit its output) — it *is* the resolver's job.
pub fn balanced_load_w(charge: &ChargeParams, scenario: &PowerScenario) -> f64 {
    let stored_per_day = charge.charge_efficiency * daily_solar_energy(scenario);
    let day_seconds = scenario.steps_per_day as f64 * scenario.dt_seconds;
    scenario.load_fraction * stored_per_day / day_seconds
}

/// The day/night forcing: the diurnal `solar_power` half-sine + the derived `load_power`.
pub fn power_resolver(
    charge: &ChargeParams,
    scenario: &PowerScenario,
) -> Result<SourceResolver, SimError> {
    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    forcings.insert(SOLAR_POWER_VAR.to_string(), solar_schedule(scenario));
    forcings.insert(
        LOAD_POWER_VAR.to_string(),
        constant(balanced_load_w(charge, scenario))?,
    );
    SourceResolver::new(forcings, HashMap::new())
}

#[cfg(test)]
mod tests {
    //! The flow-level Power gates — Stage-3 slice S3 of the reference flip.
    //!
    //! Subject: `tests/test_power_flows.py`'s 21 flow-level cases (its 8 loader cases port
    //! to [`crate::params`], where the file the loader reads actually lives). **In-src
    //! rather than in `crates/domains/tests/`** because [`charge_split`] and
    //! [`self_discharge_flux`] are private: widening them to `pub` to suit a test file
    //! would be the tail wagging the dog, and `biosphere::science` set the in-src
    //! precedent.
    //!
    //! Every oracle here is a **property** (the split partitions the supply, legs balance,
    //! flux is dt-linear, a zero input is a no-op) or a **hand value re-derived in place**.
    //! Nothing reads a number off a Python run — §5v forbids it, because inheriting a
    //! Python-produced constant would re-create Python's reference authority at the moment
    //! it is being deleted.

    use super::*;
    use simcore::flow::{assert_flow_balanced_default, per_quantity_residual, FlowResult};

    /// The three Power stocks (battery POOL + the two boundary reservoirs).
    ///
    /// ⚠ `boundary::source` takes an unclamped flag Python's does not: the solar source is
    /// unclamped (it goes arbitrarily negative as cumulative supply).
    fn state(battery: f64) -> State {
        let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
        for s in [
            battery_stock(battery).expect("battery"),
            boundary::source(SOLAR_SOURCE.to_string(), Quantity::Energy, 0.0, true)
                .expect("solar source"),
            boundary::sink(WASTE_HEAT.to_string(), Quantity::Energy, 0.0).expect("waste heat"),
        ] {
            stocks.insert(s.id.clone(), s);
        }
        State::new(0, stocks, 0, BTreeMap::new()).expect("state")
    }

    /// Constant solar/load forcing, ready to `bind`.
    fn forcing(solar_power: f64, load_power: f64) -> SourceResolver {
        let mut forcings: HashMap<String, Schedule> = HashMap::new();
        forcings.insert(
            SOLAR_POWER_VAR.to_string(),
            constant(solar_power).expect("solar"),
        );
        forcings.insert(
            LOAD_POWER_VAR.to_string(),
            constant(load_power).expect("load"),
        );
        SourceResolver::new(forcings, HashMap::new()).expect("resolver")
    }

    fn solar_charge(eta: f64) -> SolarCharge {
        SolarCharge::new(
            SOLAR_CHARGE.to_string(),
            SOLAR_SOURCE.to_string(),
            BATTERY.to_string(),
            WASTE_HEAT.to_string(),
            ChargeParams {
                charge_efficiency: eta,
            },
        )
    }

    fn load_draw() -> LoadDraw {
        LoadDraw::new(
            LOAD_DRAW.to_string(),
            BATTERY.to_string(),
            WASTE_HEAT.to_string(),
        )
    }

    fn self_discharge(k: f64) -> SelfDischarge {
        SelfDischarge::new(
            SELF_DISCHARGE.to_string(),
            BATTERY.to_string(),
            WASTE_HEAT.to_string(),
            SelfDischargeParams {
                self_discharge_rate: k,
            },
        )
    }

    fn evaluated(
        flow: &dyn Flow,
        state: &State,
        dt: f64,
        solar_power: f64,
        load_power: f64,
    ) -> FlowResult {
        let r = forcing(solar_power, load_power);
        let env = r.bind(state, dt);
        flow.evaluate(state, &env, dt).expect("evaluate")
    }

    /// Evaluate `flow` as a stock → amount map (the Python `{leg.stock: leg.amount}` idiom).
    fn legs(
        flow: &dyn Flow,
        state: &State,
        dt: f64,
        solar_power: f64,
        load_power: f64,
    ) -> BTreeMap<String, f64> {
        evaluated(flow, state, dt, solar_power, load_power)
            .legs
            .iter()
            .map(|l| (l.stock.clone(), l.amount))
            .collect()
    }

    /// The `math.isclose(..., rel_tol=1e-12)` the Python cases use, with an absolute floor
    /// so a comparison against 0.0 is not vacuously true.
    fn close(a: f64, b: f64) {
        assert!(
            (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0),
            "{a:?} != {b:?}"
        );
    }

    fn touched(result: &FlowResult, state: &State) -> Vec<Quantity> {
        per_quantity_residual(result, &state.stocks)
            .expect("residual")
            .into_keys()
            .collect()
    }

    // --- rate law: charge_split ---------------------------------------------------
    /// ⚠ **The M-power mutation site.** Swapping [`charge_split`]'s two returned legs
    /// (§5q's Control B) must redden HERE, by name — not only through a golden byte
    /// compare, which reports that bytes moved and cannot say which coefficient is wrong
    /// (§5v measurement 2).
    #[test]
    fn charge_split_is_the_efficiency_partition() {
        let (stored, lost) = charge_split(100.0, 0.95);
        close(stored, 95.0);
        close(lost, 5.0);
    }

    #[test]
    fn charge_split_sums_to_supply() {
        // The two fractions sum back to the input — every joule named, none lost.
        let (stored, lost) = charge_split(137.0, 0.88);
        close(stored + lost, 137.0);
    }

    #[test]
    fn charge_split_lossless_at_eta_one() {
        let (stored, lost) = charge_split(100.0, 1.0);
        assert_eq!(stored, 100.0);
        assert_eq!(lost, 0.0);
    }

    #[test]
    fn charge_split_zero_supply() {
        assert_eq!(charge_split(0.0, 0.95), (0.0, 0.0));
    }

    // --- SolarCharge --------------------------------------------------------------
    #[test]
    fn solar_charge_splits_supply_into_battery_and_heat() {
        let s = state(1_000.0);
        let l = legs(&solar_charge(0.95), &s, 1.0, 100.0, 0.0);
        close(l[SOLAR_SOURCE], -100.0);
        close(l[BATTERY], 95.0);
        close(l[WASTE_HEAT], 5.0);
    }

    #[test]
    fn solar_charge_balances_energy_only() {
        let s = state(1_000.0);
        let result = evaluated(&solar_charge(0.95), &s, 1.0, 100.0, 0.0);
        assert_flow_balanced_default(&result, &s.stocks).expect("balanced");
        assert_eq!(touched(&result, &s), vec![Quantity::Energy]);
    }

    #[test]
    fn solar_charge_always_three_legs() {
        // The structural-legs convention: three legs at η<1, at η=1 (heat leg 0) and at
        // night (all 0) — never a variable leg count.
        let s = state(1_000.0);
        for (eta, solar) in [(0.95, 100.0), (1.0, 100.0), (0.95, 0.0)] {
            assert_eq!(legs(&solar_charge(eta), &s, 1.0, solar, 0.0).len(), 3);
        }
    }

    #[test]
    fn solar_charge_lossless_at_eta_one() {
        let s = state(1_000.0);
        let l = legs(&solar_charge(1.0), &s, 1.0, 100.0, 0.0);
        assert_eq!(l[WASTE_HEAT], 0.0);
        close(l[BATTERY], 100.0);
    }

    #[test]
    fn solar_charge_night_is_noop() {
        let s = state(1_000.0);
        for (_, amount) in legs(&solar_charge(0.95), &s, 1.0, 0.0, 0.0) {
            assert_eq!(amount, 0.0);
        }
    }

    #[test]
    fn solar_charge_is_dt_linear() {
        // flux = rate·dt — the increment-form contract (RK4 order; multi-rate-safe).
        let s = state(1_000.0);
        let half = legs(&solar_charge(0.95), &s, 0.5, 100.0, 0.0);
        let full = legs(&solar_charge(0.95), &s, 1.0, 100.0, 0.0);
        close(full[BATTERY], 2.0 * half[BATTERY]);
        close(full[WASTE_HEAT], 2.0 * half[WASTE_HEAT]);
    }

    // --- LoadDraw -----------------------------------------------------------------
    #[test]
    fn load_draw_dissipates_battery_to_heat() {
        let s = state(1_000.0);
        let l = legs(&load_draw(), &s, 1.0, 0.0, 40.0);
        close(l[BATTERY], -40.0);
        close(l[WASTE_HEAT], 40.0);
    }

    #[test]
    fn load_draw_balances_energy_only() {
        let s = state(1_000.0);
        let result = evaluated(&load_draw(), &s, 1.0, 0.0, 40.0);
        assert_flow_balanced_default(&result, &s.stocks).expect("balanced");
        assert_eq!(touched(&result, &s), vec![Quantity::Energy]);
    }

    #[test]
    fn load_draw_zero_load_is_noop() {
        let s = state(1_000.0);
        for (_, amount) in legs(&load_draw(), &s, 1.0, 0.0, 0.0) {
            assert_eq!(amount, 0.0);
        }
    }

    #[test]
    fn load_draw_is_dt_linear() {
        let s = state(1_000.0);
        let half = legs(&load_draw(), &s, 0.5, 0.0, 40.0);
        let full = legs(&load_draw(), &s, 1.0, 0.0, 40.0);
        close(full[WASTE_HEAT], 2.0 * half[WASTE_HEAT]);
    }

    // --- SelfDischarge: the first donor-controlled Power flow ----------------------
    #[test]
    fn self_discharge_flux_is_first_order_donor_controlled() {
        // leak = k · battery — proportional to the donor's own amount, so it self-limits
        // as the battery empties.
        close(self_discharge_flux(1_000.0, 1.0e-8), 1.0e-5);
    }

    #[test]
    fn self_discharge_flux_zero_at_empty_battery() {
        assert_eq!(self_discharge_flux(0.0, 1.0e-8), 0.0);
    }

    #[test]
    fn self_discharge_flux_zero_at_zero_rate() {
        // k = 0 ⇒ an ideal leak-free cell (inert machinery, the "a zero rate is valid" case).
        assert_eq!(self_discharge_flux(1_000.0, 0.0), 0.0);
    }

    #[test]
    fn self_discharge_leaks_battery_to_heat() {
        let s = state(1_000.0);
        let l = legs(&self_discharge(1.0e-8), &s, 1.0, 0.0, 0.0);
        // A SET comparison, as in the Python: the map is id-sorted, so the pair comes
        // back in byte order, not in leg-construction order.
        let mut ids: Vec<&str> = l.keys().map(String::as_str).collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![WASTE_HEAT, BATTERY]);
        close(l[BATTERY], -1.0e-5);
        close(l[WASTE_HEAT], 1.0e-5);
    }

    #[test]
    fn self_discharge_balances_energy_only() {
        let s = state(1_000.0);
        let result = evaluated(&self_discharge(1.0e-8), &s, 1.0, 0.0, 0.0);
        assert_flow_balanced_default(&result, &s.stocks).expect("balanced");
        assert_eq!(touched(&result, &s), vec![Quantity::Energy]);
    }

    #[test]
    fn self_discharge_two_legs() {
        // Always two legs — and at battery 0 both are still emitted (zero-amount), unlike
        // a flow that would drop a leg it has nothing to put in.
        for battery in [1_000.0, 0.0] {
            let s = state(battery);
            assert_eq!(legs(&self_discharge(1.0e-8), &s, 1.0, 0.0, 0.0).len(), 2);
        }
    }

    #[test]
    fn self_discharge_is_dt_linear() {
        // Evaluated on the SAME snapshot, so the leak scales linearly in dt.
        let s = state(1_000.0);
        let half = legs(&self_discharge(1.0e-8), &s, 0.5, 0.0, 0.0);
        let full = legs(&self_discharge(1.0e-8), &s, 1.0, 0.0, 0.0);
        close(full[WASTE_HEAT], 2.0 * half[WASTE_HEAT]);
    }
}
