//! The heat evaporation takes from the plant chamber, in the chamber's books
//! (`docs/plans/post-roadmap-latent-heat-books.md`) — built lab-only and ADOPTED 2026-10-15: every
//! sealed build carries it ([`book_latent_heat`], [`latent_heat_flow`]); [`without_latent_heat`] is
//! the lab switch back to a room whose water cost it nothing.
//!
//! The plants' water moves on the plant step (90 minutes); the chamber's heat store moves on the
//! minute. A window's evaporation (~0.7 MJ in the warm phase) drawn at once would drop the room
//! ~5 K at every plant step, so the heat is drawn **minute by minute across the window that moved
//! the water**:
//!
//! * [`WindowEvaporationRecorder`], a plant-step aux process, records the window's evaporation
//!   rate (kg/s) — the registry's own transpiration flow's leg on `SOIL_WATER`, evaluated on the
//!   plant step's snapshot and environment ([`SharedFlow`]: one instance, two readers, so the
//!   recorder reads what the flow moves and not a copy of its law).
//! * [`ChamberLatentHeat`], a fast flow, moves `λ · rate · dt` from the chamber's heat store to the
//!   thermal node each minute: the condensing coil, BVAD's one coolant loop (p. 145–146). The minutes
//!   after a plant step lie in that step's window (`driver::day_groups`), so nothing is keyed on `n`.
//!
//! ⚠ **The simple form** (§1a): the node is credited when the water evaporates, not when it
//! condenses; water that stays in the air has had its heat booked to the coil early. The error is
//! bounded by λ × the air's vapour.
//!
//! ⚠ The recorder sees the flow as EVALUATED. Under arbitration the applied flow is scaled and the
//! two differ; reference runs assert `rationed == 0`.

use std::collections::BTreeMap;
use std::sync::Arc;

use domains::biosphere::science::LATENT_HEAT_VAPORIZATION;
use domains::biosphere::stocks::SOIL_WATER;
use domains::thermal::NODE;
use simcore::auxiliary::AuxProcess;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::registry::Registry;
use simcore::state::State;

use crate::chamber::CHAMBER;

/// The transpiration flow's id: the water whose heat is booked.
pub const TRANSPIRATION: &str = "biosphere.transpiration";
/// The recorder's aux-process id.
pub const EVAPORATION_RECORDER: &str = "station.window_evaporation";
/// The aux key holding the plant window's evaporation rate (kg/s).
pub const EVAPORATION_RATE_AUX: &str = "station.window_evaporation.rate";
/// The aux key holding the water evaporated so far (kg, the recorder's running total). At a day
/// end every recorded window's heat has been drawn, so the heat booked to date is λ × this — a
/// ledger any state can be read against (`docs/plans/post-roadmap-latent-heat-books.md` §8a).
pub const EVAPORATED_TOTAL_AUX: &str = "station.window_evaporation.total_kg";
/// Flow id: the evaporation's heat, chamber → thermal node.
pub const CHAMBER_LATENT_HEAT: &str = "station.chamber_latent_heat";

const SECONDS_PER_DAY: f64 = 86400.0;

/// A flow held by the registry and by the recorder at once. Keeps the inner flow's type name, id
/// and priority: it is the same flow.
pub struct SharedFlow(Arc<dyn Flow>);

impl Flow for SharedFlow {
    fn type_name(&self) -> &'static str {
        self.0.type_name()
    }
    fn id(&self) -> &str {
        self.0.id()
    }
    fn priority(&self) -> i64 {
        self.0.priority()
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        self.0.evaluate(snapshot, env, dt)
    }
}

/// The plant-step aux process recording the window's evaporation rate (kg/s) into
/// [`EVAPORATION_RATE_AUX`] and adding the window's water to [`EVAPORATED_TOTAL_AUX`]. The aux
/// channel is additive, so the rate is written as its increment.
pub struct WindowEvaporationRecorder {
    flow: Arc<dyn Flow>,
}

/// The water a transpiration result takes off the root zone (kg, ≥ 0).
fn evaporated_kg(result: &FlowResult) -> Result<f64, SimError> {
    let kg: f64 = -result
        .legs
        .iter()
        .filter(|l| l.stock == SOIL_WATER)
        .map(|l| l.amount)
        .sum::<f64>();
    if kg < 0.0 {
        return Err(SimError::Validation(format!(
            "the transpiration flow put {} kg INTO the root zone; evaporation's heat has no sign \
             for that",
            -kg
        )));
    }
    Ok(kg)
}

impl AuxProcess for WindowEvaporationRecorder {
    fn type_name(&self) -> &'static str {
        "WindowEvaporationRecorder"
    }
    fn id(&self) -> &str {
        EVAPORATION_RECORDER
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let kg = evaporated_kg(&self.flow.evaluate(snapshot, env, dt)?)?;
        let rate = kg / (dt * SECONDS_PER_DAY);
        let old = snapshot
            .aux
            .get(EVAPORATION_RATE_AUX)
            .copied()
            .unwrap_or(0.0);
        Ok(BTreeMap::from([
            (EVAPORATION_RATE_AUX.to_string(), rate - old),
            (EVAPORATED_TOTAL_AUX.to_string(), kg),
        ]))
    }
}

/// ENERGY flow `thermal.chamber → thermal.node` — the heat the window's evaporation takes from the
/// room, handed to the condensing coil (2-leg). `λ · rate · dt`, the rate read from
/// [`EVAPORATION_RATE_AUX`]; a missing key is an error, not 0.
pub struct ChamberLatentHeat {
    id: String,
    chamber: String,
    node: String,
}

impl ChamberLatentHeat {
    /// Construct the flow between `chamber` and `node`.
    pub fn new(id: String, chamber: String, node: String) -> Self {
        ChamberLatentHeat { id, chamber, node }
    }
}

impl Flow for ChamberLatentHeat {
    fn type_name(&self) -> &'static str {
        "ChamberLatentHeat"
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
        let rate = snapshot
            .aux
            .get(EVAPORATION_RATE_AUX)
            .copied()
            .ok_or_else(|| {
                SimError::Validation(format!(
                    "the chamber's latent heat reads {EVAPORATION_RATE_AUX:?}, which is not in \
                     the state — the build did not seed it"
                ))
            })?;
        let heat = LATENT_HEAT_VAPORIZATION * rate * dt;
        FlowResult::new(vec![
            Leg::new(self.chamber.clone(), -heat)?,
            Leg::new(self.node.clone(), heat)?,
        ])
    }
}

/// The fast flow, chamber → thermal node, as the sealed station's canonical fast list carries it
/// (`sealed::sealed_fast_flows`).
pub fn latent_heat_flow() -> Box<dyn Flow> {
    Box::new(ChamberLatentHeat::new(
        CHAMBER_LATENT_HEAT.to_string(),
        CHAMBER.to_string(),
        NODE.to_string(),
    ))
}

/// The plant side of the books, on a sealed build's plant registry: the transpiration flow shared
/// with a new [`WindowEvaporationRecorder`] (appended to the aux list), and [`EVAPORATION_RATE_AUX`]
/// and [`EVAPORATED_TOTAL_AUX`] seeded at 0 in `state`. An error unless the transpiration flow is in `bio_reg`.
///
/// ⚠ Apply it **before** the plants are wrapped to read the chamber (`sealed::wrap_last`) and before
/// the lab's lamp shedding: both wrap every plant-step flow and aux process alike, so the recorder
/// and the flow it shares then read one environment.
pub fn book_latent_heat(state: State, bio_reg: Registry) -> Result<(State, Registry), SimError> {
    let (flows, mut bio_aux) = bio_reg.into_parts();
    let mut shared: Option<Arc<dyn Flow>> = None;
    let flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|f| {
            if f.id() == TRANSPIRATION {
                let arc: Arc<dyn Flow> = Arc::from(f);
                shared = Some(arc.clone());
                Box::new(SharedFlow(arc)) as Box<dyn Flow>
            } else {
                f
            }
        })
        .collect();
    let Some(flow) = shared else {
        return Err(SimError::Validation(format!(
            "the chamber's latent heat needs {TRANSPIRATION:?} on the plant step"
        )));
    };
    bio_aux.push(Box::new(WindowEvaporationRecorder { flow }));
    let mut state = state;
    state.aux.insert(EVAPORATION_RATE_AUX.to_string(), 0.0);
    state.aux.insert(EVAPORATED_TOTAL_AUX.to_string(), 0.0);
    let bio = Registry::new(flows, &state.stocks, bio_aux)?;
    Ok((state, bio))
}

/// LAB-ONLY, the books as they were before 2026-10-15 (`docs/plans/post-roadmap-latent-heat-books.md`
/// §8): `(state, bio_reg, fast_reg)` of a sealed build with the recorder, the fast flow and the two
/// seeded keys removed, so evaporation costs the room nothing. The shared transpiration flow stays — it
/// delegates, so it computes what the plain flow computes. An error unless all three are present, so
/// a build that never had the books cannot be "switched off" by mistake.
pub fn without_latent_heat(
    state: State,
    bio_reg: Registry,
    fast_reg: Registry,
) -> Result<(State, Registry, Registry), SimError> {
    let (flows, aux) = bio_reg.into_parts();
    let before = aux.len();
    let aux: Vec<Box<dyn AuxProcess>> = aux
        .into_iter()
        .filter(|a| a.id() != EVAPORATION_RECORDER)
        .collect();
    let (fast_flows, fast_aux) = fast_reg.into_parts();
    let fast_before = fast_flows.len();
    let fast_flows: Vec<Box<dyn Flow>> = fast_flows
        .into_iter()
        .filter(|f| f.id() != CHAMBER_LATENT_HEAT)
        .collect();
    let mut state = state;
    let seeded = state.aux.remove(EVAPORATION_RATE_AUX).is_some()
        & state.aux.remove(EVAPORATED_TOTAL_AUX).is_some();
    if aux.len() + 1 != before || fast_flows.len() + 1 != fast_before || !seeded {
        return Err(SimError::Validation(
            "without_latent_heat: the build does not carry the chamber's latent-heat books              (recorder, fast flow and both keys)"
                .into(),
        ));
    }
    let bio = Registry::new(flows, &state.stocks, aux)?;
    let fast = Registry::new(fast_flows, &state.stocks, fast_aux)?;
    Ok((state, bio, fast))
}

#[cfg(test)]
mod tests {
    use super::*;
    use simcore::environment::SourceResolver;
    use simcore::quantities::Quantity;
    use simcore::state::Stock;
    use std::collections::HashMap;

    fn energy_pool(id: &str, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            "thermal".to_string(),
            Quantity::Energy,
            Quantity::Energy.canonical_unit(),
            amount,
            simcore::quantities::StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .expect("stock")
    }

    fn state_with_rate(rate: Option<f64>) -> State {
        let mut stocks = BTreeMap::new();
        for s in [energy_pool(CHAMBER, 4.4e7), energy_pool(NODE, 1.7e9)] {
            stocks.insert(s.id.clone(), s);
        }
        let aux = rate
            .map(|r| BTreeMap::from([(EVAPORATION_RATE_AUX.to_string(), r)]))
            .unwrap_or_default();
        State::new(0, stocks, 0, aux).expect("state")
    }

    fn heat(rate: Option<f64>, dt: f64) -> Result<(f64, f64), SimError> {
        let s = state_with_rate(rate);
        let r = SourceResolver::new(HashMap::new(), HashMap::new()).expect("resolver");
        let flow = ChamberLatentHeat::new(
            CHAMBER_LATENT_HEAT.to_string(),
            CHAMBER.to_string(),
            NODE.to_string(),
        );
        let out = flow.evaluate(&s, &r.bind(&s, dt), dt)?;
        let get = |id: &str| out.legs.iter().find(|l| l.stock == id).expect("leg").amount;
        Ok((get(CHAMBER), get(NODE)))
    }

    /// λ · rate · dt leaves the chamber and lands on the node.
    #[test]
    fn the_room_pays_lambda_times_the_rate_and_the_coil_takes_it() {
        let rate = 3.0e-5; // kg/s, ~73 W
        let (ch, node) = heat(Some(rate), 60.0).expect("evaluate");
        assert_eq!(ch, -LATENT_HEAT_VAPORIZATION * rate * 60.0);
        assert_eq!(node, -ch);
    }

    /// A missing rate is an error, not a silent 0.
    #[test]
    fn an_unseeded_rate_is_refused() {
        assert!(heat(None, 60.0).is_err());
    }

    /// A transpiration result that puts water into the root zone is refused.
    #[test]
    fn evaporation_has_one_sign() {
        let into = FlowResult::new(vec![Leg::new(SOIL_WATER.to_string(), 1.0).expect("leg")])
            .expect("result");
        assert!(evaporated_kg(&into).is_err());
        let out = FlowResult::new(vec![Leg::new(SOIL_WATER.to_string(), -2.5).expect("leg")])
            .expect("result");
        assert_eq!(evaporated_kg(&out).expect("kg"), 2.5);
    }
}
