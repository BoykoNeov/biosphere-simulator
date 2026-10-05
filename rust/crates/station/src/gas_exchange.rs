//! The crop's gas exchange on the cabin's minute step (Step 3c, the user's decision of
//! 2026-10-03; `docs/plans/post-roadmap-room-temperature.md` §16–§17).
//!
//! The plant step is 1/16 day (90 minutes). On it, the crop's CO₂ uptake is solved against the
//! chamber's CO₂ at that one instant (option C), and nothing refills the chamber inside the step.
//! In a small chamber fed by a fan ([`crate::air_split`]) that starves the crop by construction
//! of the step: measured, a 27.66-mol chamber's crop drew 0.174 of shared air's CO₂, and the
//! same 0.174 at any fan rate. This module moves the crop's three carbon-budget flows onto the
//! fast step, where the fan, the crew and the scrubber already run, so the crop draws on the
//! air minute by minute as it is refilled.
//!
//! # What moves, and what does not
//!
//! [`CARBON_BUDGET_FLOWS`] — `Allocation`, `GrowthRespiration`, `MaintenanceRespiration` —
//! move **together and unchanged**. They share one gross assimilation by design (each calls
//! the same `budget()`), so none may move alone. No stock is added: a buffer of fixed carbon
//! between the two steps would be a new mechanism with no source. Everything else — phenology,
//! senescence, nitrogen, water, transpiration — stays on the plant step.
//!
//! # The light and temperature: recorded by the plant step, not copied at build time
//!
//! The three flows read two forcings, PAR and temperature ([`WINDOW_VARS`]). The plant step
//! RECORDS them, for its own window and from its own environment, into the state's aux
//! ([`PlantWindowRecorder`], keys [`window_key`]); the minute step reads them back. Two traps
//! close by construction:
//!
//! * **The window.** The fast steps after a plant step lie in that step's window, and the
//!   values were recorded at that step. There is no `n − 1` to get wrong.
//! * **Whoever changes the plant's inputs reaches the crop's carbon budget.** A perturbed
//!   resolver, a held room temperature, the lab's lamp shedding (which wraps every plant-step
//!   aux, this recorder included) — all of them act on the plant step's environment, so they
//!   are what gets recorded. ⚠ The first build (2026-10-03, same day) copied a plant resolver
//!   into the adapter at build time instead; every one of those changes would then have missed
//!   photosynthesis without a single red.
//!
//! ⚠ The aux channel is ADDITIVE (`simcore::integrator::advanced_aux`), so "record X" is the
//! increment `X − old`, and the stored value is `old + (X − old)`: X, or within an ULP of it.
//!
//! # The other traps
//!
//! * **The step unit.** The inner flow multiplies daily rates by `dt` in days: it is handed
//!   `fast dt / 86400`.
//! * **Shared reads.** The chamber CO₂ and the soil water are read live off the minute's own
//!   snapshot, so `Allocation`'s check that its CO₂ variable IS the stock it draws still holds.
//! * **No double count.** [`split_carbon_budget`] takes the three out of the plant registry.
//! * **Any other read is an error**, so a flow that grows a new forcing cannot silently get a
//!   stale or default value.
//!
//! Die-off and the safety net need nothing here: `substep` runs arbitration and the extinction
//! pass exactly as a full step does, and skips only the aux and `n`, which these flows never
//! write.
//!
//! ⚠ Light keeps the plant step's resolution — the window's mean PAR, every minute of it — so
//! this changes the CO₂ timing and nothing else. ⚠ Development stage (thermal time) and the
//! stress factors are read after the plant step that opens the window rather than before it.

use std::collections::{BTreeMap, HashMap};

use domains::biosphere::light_path::SECONDS_PER_DAY;
use domains::biosphere::stocks::{PAR_VAR, TEMP_VAR};
use simcore::auxiliary::AuxProcess;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult};
use simcore::ids::StockId;
use simcore::registry::Registry;
use simcore::state::{State, Stock};

/// The crop's three carbon-budget flows: one gross assimilation, three readers.
pub const CARBON_BUDGET_FLOWS: [&str; 3] = [
    "biosphere.allocation",
    "biosphere.growth_respiration",
    "biosphere.maintenance_respiration",
];

/// The forcings the carbon budget reads in a sealed build, recorded by the plant step.
pub const WINDOW_VARS: [&str; 2] = [PAR_VAR, TEMP_VAR];

/// The recorder's aux-process id.
pub const PLANT_WINDOW_RECORDER: &str = "station.plant_window";

/// The aux key holding the plant window's value of forcing `var`.
pub fn window_key(var: &str) -> String {
    format!("{PLANT_WINDOW_RECORDER}.{var}")
}

/// Which step the crop's gas exchange is taken on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GasExchangeStep {
    /// With the rest of the plant, once per plant step (the form until this change).
    PlantStep,
    /// On the cabin's fast step, every minute ([module docs](self)).
    Minute,
}

/// The plant-step aux process that records [`WINDOW_VARS`] for its window.
pub struct PlantWindowRecorder;

impl AuxProcess for PlantWindowRecorder {
    fn type_name(&self) -> &'static str {
        "PlantWindowRecorder"
    }

    fn id(&self) -> &str {
        PLANT_WINDOW_RECORDER
    }

    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        _dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let mut increments = BTreeMap::new();
        for var in WINDOW_VARS {
            let key = window_key(var);
            let old = snapshot.aux.get(&key).copied().unwrap_or(0.0);
            increments.insert(key, env.get(var)? - old);
        }
        Ok(increments)
    }
}

/// What a minute-stepped plant flow reads: the recorded window, and shared stocks live.
struct WindowEnv<'a> {
    snapshot: &'a State,
    shared: &'a HashMap<String, StockId>,
    window: &'a [&'a str],
}

impl Environment for WindowEnv<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if self.window.contains(&var) {
            let key = window_key(var);
            return self.snapshot.aux.get(&key).copied().ok_or_else(|| {
                SimError::Validation(format!(
                    "gas exchange on the fast step: {key:?} is not in the state — no plant step \
                     has recorded its window yet (the master day must open with a plant step)"
                ))
            });
        }
        if let Some(sid) = self.shared.get(var) {
            return self
                .snapshot
                .stocks
                .get(sid)
                .map(|s| s.amount)
                .ok_or_else(|| {
                    SimError::Reference(format!(
                        "shared env var {var:?} points at missing stock {sid:?}"
                    ))
                });
        }
        Err(SimError::Reference(format!(
            "a plant flow on the fast step reads {var:?}, which is not in its recorded window \
             ({:?}) and is not a shared stock",
            self.window
        )))
    }
}

/// A plant flow stepped on the fast step ([module docs](self)). Keeps the inner flow's id and
/// type: it is the same flow, taken on a different step.
pub struct OnFastStep {
    inner: Box<dyn Flow>,
    shared: HashMap<String, StockId>,
    window: &'static [&'static str],
}

impl OnFastStep {
    /// Wrap `inner`, reading [`WINDOW_VARS`]; `shared` is the plant side's shared-stock wiring
    /// (var → stock).
    pub fn new(inner: Box<dyn Flow>, shared: HashMap<String, StockId>) -> Self {
        Self::reading(inner, shared, &WINDOW_VARS)
    }

    /// Wrap `inner`, reading the forcings in `window` from their [`window_key`]s. Nothing here
    /// checks that the plant step records them: a missing key errors at the first read.
    pub fn reading(
        inner: Box<dyn Flow>,
        shared: HashMap<String, StockId>,
        window: &'static [&'static str],
    ) -> Self {
        OnFastStep {
            inner,
            shared,
            window,
        }
    }
}

impl Flow for OnFastStep {
    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }

    fn id(&self) -> &str {
        self.inner.id()
    }

    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let env = WindowEnv {
            snapshot,
            shared: &self.shared,
            window: self.window,
        };
        self.inner.evaluate(snapshot, &env, dt / SECONDS_PER_DAY)
    }
}

/// The plant registry taken apart: `(flows without the carbon budget, the carbon budget, aux)`.
pub type SplitPlantRegistry = (
    Vec<Box<dyn Flow>>,
    Vec<Box<dyn Flow>>,
    Vec<Box<dyn AuxProcess>>,
);

/// Take the carbon-budget flows out of the plant registry. An error unless all three are
/// found, so a renamed flow cannot silently stay behind.
pub fn split_carbon_budget(bio_reg: Registry) -> Result<SplitPlantRegistry, SimError> {
    let (flows, aux) = bio_reg.into_parts();
    let mut budget: Vec<Box<dyn Flow>> = Vec::new();
    let mut rest: Vec<Box<dyn Flow>> = Vec::new();
    for f in flows {
        if CARBON_BUDGET_FLOWS.contains(&f.id()) {
            budget.push(f);
        } else {
            rest.push(f);
        }
    }
    if budget.len() != CARBON_BUDGET_FLOWS.len() {
        let found: Vec<&str> = budget.iter().map(|f| f.id()).collect();
        return Err(SimError::Validation(format!(
            "gas exchange on the fast step: the plant registry holds {found:?} of the carbon \
             budget's {CARBON_BUDGET_FLOWS:?}; they move together or not at all"
        )));
    }
    Ok((rest, budget, aux))
}

/// Refuse a day whose fast steps do not each follow exactly ONE plant step: with more, the
/// window recorded last would stand for both.
pub fn require_one_plant_step_per_group(
    steps_per_day: u64,
    plant_steps_per_day: u64,
) -> Result<(), SimError> {
    if plant_steps_per_day == 0 || !steps_per_day.is_multiple_of(plant_steps_per_day) {
        return Err(SimError::Validation(format!(
            "gas exchange on the fast step needs each plant step followed by its own fast steps, \
             but {steps_per_day} fast steps a day do not divide into {plant_steps_per_day} plant \
             steps"
        )));
    }
    Ok(())
}

/// Move the crop's gas exchange onto the fast step of an assembled two-rate build:
/// `(bio_reg, fast_reg)` in, the same pair out — the three carbon-budget flows taken out of the
/// first and wrapped into the second, and [`PlantWindowRecorder`] added to the first. `shared`
/// is the plant side's shared-stock wiring (`weather_shared`).
pub fn gas_exchange_on_fast_step(
    stocks: &BTreeMap<StockId, Stock>,
    bio_reg: Registry,
    fast_reg: Registry,
    shared: &HashMap<String, StockId>,
) -> Result<(Registry, Registry), SimError> {
    let (rest, budget, mut bio_aux) = split_carbon_budget(bio_reg)?;
    bio_aux.push(Box::new(PlantWindowRecorder));
    let (mut fast_flows, fast_aux) = fast_reg.into_parts();
    for inner in budget {
        fast_flows.push(Box::new(OnFastStep::new(inner, shared.clone())));
    }
    Ok((
        Registry::new(rest, stocks, bio_aux)?,
        Registry::new(fast_flows, stocks, fast_aux)?,
    ))
}
