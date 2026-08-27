//! The **basis** under the Tier-2 bands — the station half.
//!
//! The seams, the metric and the `domains`-side measurements live in [`domains::ulp_probe`],
//! whose module header carries the whole design and the finding it turned up. This module
//! composes those seams onto the two coupled runs the station's six Tier-2 bands are sized
//! against, exactly as `tests/crossport/measure_tier2_bands.py`'s Step-5 half does.
//!
//! * **`station_heat_closure` / `sealed_energy_drift`** (band `1e-12`) — Power→Thermal only,
//!   no biosphere, so the two transcendentals are the half-sine `sin` and the Stefan-Boltzmann
//!   `t⁴`. They are perturbed **separately** and the worse taken, as in Python: a probe that
//!   moves both at once can have them cancel, which understates the sensitivity. The 15-year
//!   energy-drift run shares the exact same graph and its `T⁴` attractor only damps the
//!   perturbation further, so the cheap 7-day measure bounds both — the Phase-7 argument,
//!   inherited unchanged.
//! * **`greenhouse` / `lighting` / `harvest` / `sealed_station`** (band `1e-11`, shared with
//!   the seven biosphere goldens) — a biosphere FvCB transcendental is in the graph, measured
//!   on the cheap 7-day greenhouse rather than the 1.3 M-sub-step sealed run. That is the same
//!   deliberate cost choice Python made, and it rests on the same structural argument: the
//!   station regulators hold the shared gas pools at their setpoints between the once-daily
//!   biosphere lumps, so a one-ULP nudge cannot amplify across master days.
//!
//! ⚠ [`domains::ulp_probe::nudged_power_resolver`] is the station's solar seam and not a
//! second copy of one: `crate::system::station_resolver` **is** `power_resolver`.

use domains::params;
use domains::ulp_probe::{
    nudge_forcing, nudge_radiator, nudged_power_resolver, worst_over_both_directions, Nudge,
};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

use crate::greenhouse::{
    build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver, run_greenhouse,
};
use crate::run_station;
use crate::scenario::{greenhouse_scenario, HEAT_CLOSURE_DAYS, HEAT_CLOSURE_SCENARIO};
use crate::system::build_station;

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

/// Which of the coupled energy run's two transcendentals a measurement perturbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergySeam {
    /// The half-sine `sin` of the solar schedule (and the load derived from it).
    Solar,
    /// The Stefan-Boltzmann `t⁴` inside the radiator flow.
    Radiator,
}

/// [`crate::goldens::station`] with one of its two transcendentals nudged.
pub fn station_energy_snapshot(seam: EnergySeam, nudge: Nudge) -> String {
    let charge = params::charge();
    let thermal = params::thermal();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) =
        build_station(&charge, &thermal, &scenario, None).expect("build_station");
    let (registry, resolver) = match seam {
        EnergySeam::Solar => (
            registry,
            nudged_power_resolver(&charge, &scenario.power, nudge).expect("station_resolver"),
        ),
        EnergySeam::Radiator => (
            nudge_radiator(registry, &state.stocks, &thermal, nudge).expect("nudge radiator"),
            nudged_power_resolver(&charge, &scenario.power, Nudge::Off).expect("station_resolver"),
        ),
    };
    let integrator = EulerIntegrator::new(registry);
    let mut noop = |_: &State| {};
    let (final_state, _, _) = run_station(
        &integrator,
        state,
        &resolver,
        scenario.power.dt_seconds,
        HEAT_CLOSURE_DAYS * scenario.power.steps_per_day,
        &mut noop,
    )
    .expect("run station");
    snapshot(&final_state)
}

/// [`crate::goldens::greenhouse`] with the Beer-Lambert `exp` nudged through the `par`
/// forcing (the equivalence is argued and checked in [`domains::ulp_probe`]'s header).
pub fn greenhouse_snapshot(nudge: Nudge) -> String {
    let crew = params::crew();
    let eclss = params::eclss();
    let scenario = greenhouse_scenario();
    let (state, bio_reg, cabin_reg) =
        build_greenhouse(&crew, &eclss, &scenario, true, domains::crew::FECAL_WASTE)
            .expect("build_greenhouse");
    let bio_resolver = greenhouse_bio_resolver(&scenario).expect("bio_resolver");
    let bio_resolver =
        nudge_forcing(bio_resolver, domains::biosphere::stocks::PAR_VAR, nudge).expect("nudge par");
    let cabin_resolver = greenhouse_cabin_resolver(&scenario).expect("cabin_resolver");
    let (states, _, _) = run_greenhouse(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(cabin_reg),
        state,
        &bio_resolver,
        &cabin_resolver,
        &scenario,
    )
    .expect("run greenhouse");
    snapshot(states.last().expect("at least one day boundary"))
}

/// The propagated ±1-ULP sensitivity of the coupled Power→Thermal run — the worse of the two
/// seams, each perturbed alone.
pub fn station_energy_sensitivity() -> (f64, String) {
    let mut worst = 0.0_f64;
    let mut where_ = String::new();
    for seam in [EnergySeam::Solar, EnergySeam::Radiator] {
        let (deviation, path) =
            worst_over_both_directions(|nudge| station_energy_snapshot(seam, nudge));
        if deviation >= worst {
            worst = deviation;
            where_ = path;
        }
    }
    (worst, where_)
}

/// The propagated ±1-ULP Beer-Lambert sensitivity of the 7-day greenhouse.
pub fn greenhouse_sensitivity() -> (f64, String) {
    worst_over_both_directions(greenhouse_snapshot)
}
