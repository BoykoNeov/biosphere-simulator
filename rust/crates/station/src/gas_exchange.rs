//! The crop's gas exchange on the cabin's minute step (Step 3c, the user's decision of
//! 2026-10-03; `docs/plans/post-roadmap-room-temperature.md` §16).
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
//! # The four traps [`OnFastStep`] closes
//!
//! * **The forcing window.** The fast steps that follow a plant step lie in THAT plant step's
//!   window, and by then `state.n` has already advanced. So every forcing is read at
//!   `(n − 1, plant dt)`. Read at `n`, the light would run 90 minutes early, and nothing would
//!   go red. ⚠ The light keeps the plant step's resolution — the window's mean PAR, every minute
//!   of it — so this changes the CO₂ timing and nothing else.
//! * **The step unit.** The inner flow multiplies daily rates by `dt` in days: it is handed
//!   `fast dt / 86400`.
//! * **Shared reads.** The chamber CO₂ and the soil water are read live off the minute's own
//!   snapshot, so `Allocation`'s check that its CO₂ variable IS the stock it draws still holds.
//! * **No double count.** [`split_carbon_budget`] takes the three out of the plant registry.
//!
//! Die-off and the safety net need nothing here: `substep` runs arbitration and the extinction
//! pass exactly as a full step does, and skips only the aux and `n`, which these flows never
//! write.
//!
//! ⚠ Two reads move by one plant window: development stage (thermal time) and the stress
//! factors are read after the plant step that opens the window rather than before it.

use std::collections::{BTreeMap, HashMap};

use domains::biosphere::light_path::SECONDS_PER_DAY;
use simcore::environment::{Environment, Schedule, SourceResolver};
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

/// Which step the crop's gas exchange is taken on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GasExchangeStep {
    /// With the rest of the plant, once per plant step (the frozen form until this change).
    PlantStep,
    /// On the cabin's fast step, every minute ([module docs](self)).
    Minute,
}

/// The plant side's forcings, read for the plant window a fast step lies in.
pub struct PlantWindow {
    forcings: HashMap<String, Schedule>,
    shared: HashMap<String, StockId>,
    plant_dt: f64,
}

impl PlantWindow {
    /// Take over a plant-side resolver (one per flow: a schedule is not `Clone`).
    pub fn new(resolver: SourceResolver, plant_dt: f64) -> Self {
        let (forcings, shared) = resolver.into_parts();
        PlantWindow {
            forcings,
            shared,
            plant_dt,
        }
    }

    /// The environment a fast step at `snapshot` sees: forcings for window `snapshot.n − 1`,
    /// shared stocks live.
    pub fn at<'a>(&'a self, snapshot: &'a State) -> Result<WindowEnv<'a>, SimError> {
        let n = snapshot.n.checked_sub(1).ok_or_else(|| {
            SimError::Validation(
                "gas exchange on the fast step: a fast step before the first plant step has no \
                 plant window to read (the master day must open with a plant step)"
                    .into(),
            )
        })?;
        Ok(WindowEnv {
            window: self,
            snapshot,
            n,
        })
    }
}

/// [`PlantWindow`] bound to one fast step. Mirrors `simcore`'s bound environment, except that
/// forcings are read at the plant window `n` it was given rather than at `snapshot.n`.
pub struct WindowEnv<'a> {
    window: &'a PlantWindow,
    snapshot: &'a State,
    n: u64,
}

impl Environment for WindowEnv<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        if let Some(schedule) = self.window.forcings.get(var) {
            let value = schedule(self.n, self.window.plant_dt);
            if !value.is_finite() {
                return Err(SimError::Validation(format!(
                    "forcing schedule for env var {var:?} returned non-finite value: {value:?}"
                )));
            }
            return Ok(value);
        }
        if let Some(sid) = self.window.shared.get(var) {
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
            "unknown env var {var:?} (wired as neither forcing nor shared stock)"
        )))
    }
}

/// A plant flow stepped on the fast step ([module docs](self)). Keeps the inner flow's id and
/// type: it is the same flow, taken on a different step.
pub struct OnFastStep {
    inner: Box<dyn Flow>,
    window: PlantWindow,
}

impl OnFastStep {
    /// Wrap `inner`, reading its forcings through `window`.
    pub fn new(inner: Box<dyn Flow>, window: PlantWindow) -> Self {
        OnFastStep { inner, window }
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
        let env = self.window.at(snapshot)?;
        self.inner.evaluate(snapshot, &env, dt / SECONDS_PER_DAY)
    }
}

/// Take the carbon-budget flows out of the plant registry: `(plant registry without them, the
/// three)`. An error unless all three are found, so a renamed flow cannot silently stay behind.
pub fn split_carbon_budget(
    bio_reg: Registry,
    stocks: &BTreeMap<StockId, Stock>,
) -> Result<(Registry, Vec<Box<dyn Flow>>), SimError> {
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
    Ok((Registry::new(rest, stocks, aux)?, budget))
}

/// Refuse a day whose fast steps do not each follow exactly ONE plant step: with more, "the
/// plant window this fast step lies in" has two answers.
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
/// `(bio_reg, fast_reg)` in, the same pair out with the three carbon-budget flows taken out of
/// the first and wrapped into the second. `window` builds the plant-side resolver each wrapped
/// flow reads (called once per flow).
pub fn gas_exchange_on_fast_step(
    stocks: &BTreeMap<StockId, Stock>,
    bio_reg: Registry,
    fast_reg: Registry,
    plant_dt: f64,
    mut window: impl FnMut() -> Result<SourceResolver, SimError>,
) -> Result<(Registry, Registry), SimError> {
    let (bio_reg, budget) = split_carbon_budget(bio_reg, stocks)?;
    let (mut fast_flows, fast_aux) = fast_reg.into_parts();
    for inner in budget {
        fast_flows.push(Box::new(OnFastStep::new(
            inner,
            PlantWindow::new(window()?, plant_dt),
        )));
    }
    Ok((bio_reg, Registry::new(fast_flows, stocks, fast_aux)?))
}
