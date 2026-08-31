//! The **mechanism-switch lab** — a season with named flows removed, replaced or added.
//!
//! Plan: the science-switch plan (`docs/plans/post-roadmap-science-switch.md`), slices 1
//! (the knockout) and 2 + 3 (the replace/add composers and their two constructional
//! controls). Where [`super::biosphere_with`] substitutes a **number** before assembly, this
//! changes a **process** after it: `build_season_with` builds the frozen machine, and the
//! composers move `Box<dyn Flow>`s in and out of the built [`Registry`].
//!
//! # Why this is a production seam and not a test helper
//!
//! It was a test helper. `system.rs`'s `trace_without_flow` did exactly this job for the
//! root-zone-capture diagnostics, and its own docstring justified re-implementing the
//! assembly body: *"no production seam was added for this"*. That was true when it was
//! written and it is the defect this module closes — **two assembly bodies, one of them the
//! control the other is compared against.** If `build_season_with` grows a loss-sink
//! quantity or a state variable, the copy does not, and the diagnostic quietly stops
//! controlling for the run it is differenced against while every gate stays green.
//!
//! So there is one assembly body now, composed onto afterwards. `tests/one_assembly_body.rs`
//! is what keeps it one: a source scan, because the property is about the *tree* rather than
//! about any run, and nothing else in the tree can see a second body appearing.
//!
//! # ⚠ The empty-drop control is a ROUND TRIP, and is not this slice's gate
//!
//! `build_season_without(s, p, &[])` is `Registry::new(into_parts(Registry::new(…)))`, so its
//! bit-identity with [`build_season_with`] holds by construction. It is kept (it is cheap,
//! and it would catch a `Registry::new` that was not order-invariant) but it is **not**
//! evidence the lift worked — *"if one side's copy came from the other, the gate is a round
//! trip"*. The value harness's empty-substitution control is not the same shape: there
//! `biosphere_with(&[])` and `params::biosphere()` reach one object down two independent
//! routes. Here there is no second route, so the gate is the source scan.
//!
//! # The three composers are one body, and the collisions between them are errors
//!
//! [`build_season_without`], [`build_season_replacing`] and [`build_season_adding`] are thin
//! callers of one [`build_season_composed`], for the reason slice 1 existed at all: three
//! copies of "take the registry apart, change one thing, rebuild" is three places to diverge.
//! Composing them in one call also makes a family of silent no-ops expressible — drop *and*
//! replace the same id, add an id that is already there, name one target twice — and every one
//! of them is an error rather than a run whose difference the caller misattributes. ⚠ That
//! general entry point is **public rather than private on purpose**: those collision guards
//! are reachable only through it, and a guard no caller can reach is a guard no mutation can
//! redden. **A replacement must carry its target's id**: renaming a process is a drop plus an
//! add, and saying so keeps "the composed run differs from the baseline **in one named
//! place**" true by construction.
//!
//! # ⚠ Which control is the evidence, and which one is nearly blind
//!
//! §8 of the plan asks for two answers known *by construction*, and they are not equal:
//!
//! * **the scaled replacement is the evidence.** [`ScaledMechanism`] at 0.5 must halve the
//!   target's legs exactly and move the run; at 1.0 it must reproduce the baseline bit for
//!   bit (`x · 1.0 == x`). Only this one can see a composer that locates the target, drops it,
//!   and quietly keeps the **original** box — the argument never inserted;
//! * **the no-op replacement is the "and nothing else moved" half.** Replacing a flow with a
//!   freshly built identical instance must be bit-identical — but a composer that ignored the
//!   argument passes it green, and even the registry's own type names are unchanged either
//!   way. It is kept and labelled, exactly like the empty-drop round trip above it.
//!
//! # ⚠ Why the scaler is written here rather than reused from `station`
//!
//! The plan says to wrap a flow in `station::perturbations::ScaledFlow`, "already written".
//! It is not reachable and would not fit if it were: `station` depends on `domains`, so the
//! dependency runs the wrong way, and that wrapper reads its factor from a **forcing var**,
//! which a lab replacement would have to add to the frozen weather resolver before it could
//! run. [`ScaledMechanism`] takes a plain constant instead and needs no resolver change.
//! (`domains::ulp_probe::nudge_radiator` is the third hand-rolled instance of this same
//! replace-a-flow-by-id shape, for thermal. A shared primitive would belong in
//! `simcore::registry`, and moving one is not this batch.)
//!
//! # This module takes no decision and endorses no science
//!
//! A knockout regenerates evidence about a mechanism's contribution. It says nothing about
//! whether the mechanism should be there — the same standing `lab` has had since the value
//! half (`docs/log/value-switch-harness.md`). ⚠ The tree still holds **no second form of any
//! biosphere process** (§2C of the plan, measured), so nothing here has yet been pointed at a
//! scientific question; the composers' answers so far are the two that arithmetic fixes in
//! advance.

use crate::biosphere::params::BiosphereParams;
use crate::biosphere::system::{build_season_with, SeasonScenario};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::registry::Registry;
use simcore::state::State;
use std::collections::{BTreeMap, BTreeSet};

/// Wrap a flow, multiplying **every** leg by a constant `factor` — the lab's scaled
/// replacement, and the one instrument whose answer arithmetic fixes in advance.
///
/// It scales the whole flow, so the result stays internally balanced
/// (`Σ (α·leg) = α·Σ leg = 0` per quantity) — "arbitration scales the *whole* flow", applied
/// as an experiment rather than as a backstop. `factor = 1.0` reproduces the wrapped flow
/// **bit-identically** (`x · 1.0 == x`) and `0.5` halves each leg exactly (both are exact in
/// binary floating point, which is why the controls use those two and not, say, 0.9).
///
/// `id` and `priority` delegate, so [`Registry::new`] sorts the wrapper into the wrapped
/// flow's slot and the reduction order is untouched. `type_name` does **not** delegate — a
/// wrapper reports its own name (the [`Flow::type_name`] contract), which is what makes a
/// scaled replacement visible in a registry inventory without running anything, and what
/// `tests/lab_only_mechanisms.rs` scans for.
///
/// ⚠ A non-finite `factor` is not rejected here; it surfaces as a [`Leg::new`] validation
/// error at the first `evaluate`, which is the same guard a non-finite rate law meets.
pub struct ScaledMechanism {
    inner: Box<dyn Flow>,
    factor: f64,
}

impl ScaledMechanism {
    /// Wrap `inner`, scaling all its legs by `factor`.
    pub fn new(inner: Box<dyn Flow>, factor: f64) -> ScaledMechanism {
        ScaledMechanism { inner, factor }
    }

    /// The constant every leg is multiplied by.
    pub fn factor(&self) -> f64 {
        self.factor
    }
}

impl Flow for ScaledMechanism {
    fn type_name(&self) -> &'static str {
        "ScaledMechanism"
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
        let result = self.inner.evaluate(snapshot, env, dt)?;
        let legs: Vec<Leg> = result
            .legs
            .iter()
            .map(|leg| Leg::new(leg.stock.clone(), leg.amount * self.factor))
            .collect::<Result<Vec<_>, _>>()?;
        FlowResult::new(legs)
    }
}

/// The season with every flow in `drop_ids` removed from the assembled registry.
///
/// The stocks, aux processes, state variables and boundary sinks are the ordinary build's:
/// only the flow list is shorter, which is the *"changes exactly one thing"* property the
/// root-zone-capture diagnostic needs and the crude control (zeroing a parameter) lost when
/// the soil geometry re-basing gave `EXTR` a second reader.
///
/// # ⚠ A drop that misses is an error, never a quiet baseline
///
/// The failure this shape is prone to is the science-side twin of the value harness's §7:
/// name a flow id that is not in *this* scenario's registry — a typo, or a flow the
/// scenario's flags never wired — get a clean run, and read the baseline back as "removing
/// it changed nothing". So an id that matched nothing is a [`SimError`], and so is the same
/// id twice (the second would be a no-op the caller cannot see).
///
/// **"Exactly one" needs no counting here**, unlike the helper this replaces: the registry
/// this filters was already built by [`Registry::new`], which rejects duplicate flow ids. A
/// matched id therefore matched exactly one flow, by that constructor's contract rather than
/// by an assertion of ours.
///
/// `p` is threaded rather than loaded so the two harness halves **compose** — a run with one
/// coefficient substituted *and* one process removed is `build_season_without(s,
/// &biosphere_with(&subs)?, &[id])`, not a third assembly path. It also keeps the biosphere's
/// single production param load single (`tests/param_funnel.rs`).
pub fn build_season_without(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
    drop_ids: &[&str],
) -> Result<(State, Registry), SimError> {
    build_season_composed(scenario, p, drop_ids, Vec::new(), Vec::new())
}

/// The season with each named flow **replaced** by the flow given for it.
///
/// The replacement carries the target's id (enforced below), so it takes the target's slot in
/// the id-sorted reduction order and the run differs in exactly one named place. This is the
/// composer an alternative *form* of a process arrives through — and the tree has none today
/// (§2C), so its live callers are the two constructional controls of
/// `tests/mechanism_switch_run.rs`: a freshly built identical instance (bit-identical) and a
/// [`ScaledMechanism`] (halves its target's legs exactly).
///
/// # ⚠ Same id, or it is a drop plus an add
///
/// A replacement whose id differs from its target is refused. It would be two changes wearing
/// one name — one process gone and a differently-identified one arrived — and the caller
/// would attribute the whole difference to "the new form of X". Spell it as
/// [`build_season_without`] plus [`build_season_adding`] and the two changes stay countable.
///
/// A target that matches nothing is an error, for the reason [`build_season_without`] gives:
/// a swap that misses is read back as the baseline.
pub fn build_season_replacing(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
    replacements: Vec<(&str, Box<dyn Flow>)>,
) -> Result<(State, Registry), SimError> {
    build_season_composed(scenario, p, &[], replacements, Vec::new())
}

/// The season with extra flows **added** to the assembled registry.
///
/// The "add one process" half of the plan's §4 — a mechanism the frozen build does not carry
/// at all. An id already in this scenario's registry is refused: that is a replacement, and
/// letting it through would mean [`Registry::new`]'s duplicate-id rejection reporting the
/// composition's mistake in the engine's words rather than the lab's.
///
/// ⚠ **Flows only, over the frozen stock set.** An addition whose legs touch a stock the
/// season does not have fails at the first step, and adding the stock is deliberately not
/// offered here: it changes what conservation is asserted over, which is a bigger claim than
/// a mechanism swap. `station::perturbations::with_station_leak` is the shape that does it,
/// and it is a station-layer composer with no golden behind it.
pub fn build_season_adding(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
    additions: Vec<Box<dyn Flow>>,
) -> Result<(State, Registry), SimError> {
    build_season_composed(scenario, p, &[], Vec::new(), additions)
}

/// Drop, replace and add in **one** composition: assemble, change the flow list, rebuild.
///
/// The one body behind the three named composers above, and public rather than private for a
/// reason this tree has paid for twice: a guard on a branch no caller can reach is a guard
/// nothing can redden. Every collision *between* the three — a target named twice, an id both
/// dropped and added, a renaming replacement — is checked here, and each is reachable only
/// through this entry point. Two mechanism changes in one run is also the shape a real A/B
/// pair takes (swap the form, drop the process it makes redundant), so this is the general
/// case and the three above are its convenient names.
///
/// Every check runs **before** the season is built, so a bad request fails as a statement
/// about the request rather than about the run.
pub fn build_season_composed(
    scenario: &SeasonScenario,
    p: &BiosphereParams,
    drop_ids: &[&str],
    replacements: Vec<(&str, Box<dyn Flow>)>,
    additions: Vec<Box<dyn Flow>>,
) -> Result<(State, Registry), SimError> {
    // 1. One id may be named as a target once, across both composers. Twice is either a
    //    no-op the caller cannot see (two drops) or two changes to one process.
    let mut targets: BTreeSet<&str> = BTreeSet::new();
    for id in drop_ids.iter().chain(replacements.iter().map(|(id, _)| id)) {
        if !targets.insert(id) {
            return Err(SimError::Validation(format!(
                "{id:?} is named twice as a target — dropping and replacing one process in \
                 one composition, or naming it twice, hides a change the caller cannot count"
            )));
        }
    }
    // 2. A replacement keeps its target's id (see `build_season_replacing`).
    for (target, flow) in &replacements {
        if flow.id() != *target {
            return Err(SimError::Validation(format!(
                "the replacement for {target:?} has id {:?} — a replacement that renames is a \
                 drop plus an add, and must be spelled as one",
                flow.id()
            )));
        }
    }
    // 3. Additions are distinct from each other and from every target.
    let mut added: BTreeSet<&str> = BTreeSet::new();
    for flow in &additions {
        if !added.insert(flow.id()) {
            return Err(SimError::Validation(format!(
                "{:?} is added twice — `Registry::new` would reject the pair, in the engine's \
                 words rather than the lab's",
                flow.id()
            )));
        }
        if targets.contains(flow.id()) {
            return Err(SimError::Validation(format!(
                "{:?} is both a target and an addition — re-implementing a process in place \
                 is a replacement",
                flow.id()
            )));
        }
    }

    let (state, registry) = build_season_with(scenario, p)?;
    let (flows, aux) = registry.into_parts();

    // 4. An addition must not already be there. `Registry::new` would catch the duplicate,
    //    but as "duplicate flow id", which reads as an engine fault rather than as this
    //    composition asking for a replacement under the wrong name.
    for flow in &additions {
        if flows.iter().any(|f| f.id() == flow.id()) {
            return Err(SimError::Validation(format!(
                "{:?} is already in this scenario's registry — that is a replacement, not an \
                 addition",
                flow.id()
            )));
        }
    }

    let mut pending: BTreeMap<&str, Box<dyn Flow>> = replacements.into_iter().collect();
    let mut dropped: BTreeSet<&str> = BTreeSet::new();
    let mut kept: Vec<Box<dyn Flow>> = Vec::with_capacity(flows.len() + additions.len());
    for flow in flows {
        if let Some(id) = drop_ids.iter().find(|id| **id == flow.id()) {
            dropped.insert(id);
            continue;
        }
        // The `remove` is bound before the match: `flow.id()` borrows `flow`, and a borrow
        // living across the arms would forbid moving `flow` into `kept` in the `None` one.
        let replacement = pending.remove(flow.id());
        match replacement {
            Some(replacement) => kept.push(replacement),
            None => kept.push(flow),
        }
    }
    // 5. Every target matched. A miss is the failure this whole shape is built around: a
    //    typo, or a flow this scenario's flags never wired, giving a clean run that reads
    //    back as the baseline.
    if dropped.len() != drop_ids.len() || !pending.is_empty() {
        let mut missing: Vec<&str> = drop_ids
            .iter()
            .copied()
            .filter(|id| !dropped.contains(id))
            .collect();
        missing.extend(pending.keys().copied());
        return Err(SimError::Validation(format!(
            "{missing:?} is not in this scenario's registry — a drop or replacement that \
             misses would be read back as the baseline"
        )));
    }
    kept.extend(additions);
    let registry = Registry::new(kept, &state.stocks, aux)?;
    Ok((state, registry))
}

/// A flow built fresh for one `(scenario, params)` pair.
///
/// ⚠ A factory rather than a `Box<dyn Flow>`, and the reason is the comparison report: it
/// drives **five** runs per column, `Box<dyn Flow>` is not `Clone`, and the composers consume
/// what they are given. Handing the same box to five runs is not expressible; re-deriving one
/// per run in the caller is the roster-in-a-loop shape this module already refuses elsewhere.
pub type FlowFactory = Box<dyn Fn(&SeasonScenario, &BiosphereParams) -> Box<dyn Flow>>;

/// One mechanism change, held as a **request** so it can be applied to several scenarios.
///
/// [`build_season_composed`] takes a composition and a scenario together and answers for that
/// pair. A report column is one composition against *many* scenarios, which raises a question
/// a single call never has to answer: **the frozen scenarios do not share a flow set.** Ten of
/// the twenty-three biosphere flows are in all four canonical builds; the other thirteen are
/// scenario-specific, and they are where the interesting science lives — decomposition,
/// humification, microbial respiration, the nitrogen releases, grazing. Measured 2026-08-31.
///
/// So "swap the soil carbon scheme" is a perfectly ordinary request that **cannot apply to the
/// open field**, and a harness that treats that as an error is unusable for most of the swaps
/// anyone would want. [`Composition::absent_targets`] is how a caller asks *before* running,
/// so a scenario the request does not reach can be reported as not applicable rather than as a
/// failure — or, worse, silently left out of the table.
#[derive(Default)]
pub struct Composition {
    /// Flow ids removed — [`build_season_without`]'s argument.
    pub drop_ids: Vec<String>,
    /// `(target id, the flow to run instead)` — [`build_season_replacing`]'s argument.
    pub replacements: Vec<(String, FlowFactory)>,
    /// Flows the frozen build does not carry — [`build_season_adding`]'s argument.
    pub additions: Vec<FlowFactory>,
}

impl Composition {
    /// A knockout: the named flows removed.
    pub fn dropping(ids: &[&str]) -> Composition {
        Composition {
            drop_ids: ids.iter().map(|id| id.to_string()).collect(),
            ..Composition::default()
        }
    }

    /// One flow replaced by the flow `make` builds for each run.
    pub fn replacing(target: &str, make: FlowFactory) -> Composition {
        Composition {
            replacements: vec![(target.to_string(), make)],
            ..Composition::default()
        }
    }

    /// One flow the frozen build does not carry, added.
    pub fn adding(make: FlowFactory) -> Composition {
        Composition {
            additions: vec![make],
            ..Composition::default()
        }
    }

    /// The ids this composition needs to find in a scenario's registry: everything it drops
    /// or replaces.
    ///
    /// ⚠ Additions are **not** here, and that asymmetry is the point. A missing target means
    /// "this scenario does not have this process", which is a fact about the scenario; an
    /// addition whose id is already present means "this is a replacement wearing the wrong
    /// name", which is a fact about the *request* and stays an error.
    pub fn targets(&self) -> Vec<&str> {
        self.drop_ids
            .iter()
            .map(String::as_str)
            .chain(self.replacements.iter().map(|(id, _)| id.as_str()))
            .collect()
    }

    /// Which of [`Composition::targets`] this scenario's registry does not contain.
    ///
    /// Empty means [`Composition::apply`] will not fail *for that reason* — every other
    /// refusal in [`build_season_composed`] is about the request and applies to every
    /// scenario equally.
    ///
    /// ⚠ This builds the season a second time rather than reading the ids out of a build the
    /// caller already has. That is deliberate: the alternative is to hand the assembled parts
    /// to a second composition path, which is the two-bodies defect this module exists to
    /// close. An assembly is cheap next to the run it precedes — the long-horizon column
    /// spends minutes in `run_perennial` — so the honest shape is affordable here.
    pub fn absent_targets(
        &self,
        scenario: &SeasonScenario,
        p: &BiosphereParams,
    ) -> Result<Vec<String>, SimError> {
        let (_state, registry) = build_season_with(scenario, p)?;
        let present: BTreeSet<&str> = registry.flows().iter().map(|f| f.id()).collect();
        Ok(self
            .targets()
            .into_iter()
            .filter(|id| !present.contains(id))
            .map(str::to_string)
            .collect())
    }

    /// This composition, applied — [`build_season_composed`] with the factories run.
    pub fn apply(
        &self,
        scenario: &SeasonScenario,
        p: &BiosphereParams,
    ) -> Result<(State, Registry), SimError> {
        let drops: Vec<&str> = self.drop_ids.iter().map(String::as_str).collect();
        let replacements: Vec<(&str, Box<dyn Flow>)> = self
            .replacements
            .iter()
            .map(|(id, make)| (id.as_str(), make(scenario, p)))
            .collect();
        let additions: Vec<Box<dyn Flow>> =
            self.additions.iter().map(|make| make(scenario, p)).collect();
        build_season_composed(scenario, p, &drops, replacements, additions)
    }

    /// Whether this composition asks for anything at all.
    ///
    /// ⚠ An empty composition is the science half's `UNCHANGED` column — a round trip through
    /// the composers that must reproduce the frozen run bit for bit. It is not an error, but a
    /// *report* built from one is measuring nothing, and the caller that offers a column
    /// should say so rather than print a second baseline under an experiment's label.
    pub fn is_empty(&self) -> bool {
        self.drop_ids.is_empty() && self.replacements.is_empty() && self.additions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biosphere::params;
    use crate::biosphere::system::{sealed_chamber_scenario, DEFAULT_SCENARIO};
    use std::collections::BTreeSet;

    const ROOT_ZONE_CAPTURE: &str = "biosphere.root_zone_capture";
    /// The replacement controls' subject: a flow with non-zero legs from the first step
    /// (standing biomass burns at sowing), so a scaled replacement is measurable at `n = 0`.
    const MAINTENANCE: &str = "biosphere.maintenance_respiration";

    fn frozen() -> BiosphereParams {
        params::biosphere()
    }

    fn flow_ids(registry: &Registry) -> BTreeSet<String> {
        registry
            .flows()
            .iter()
            .map(|f| f.id().to_string())
            .collect()
    }

    /// The round trip of the module header: no drops is the ordinary build. Kept because it
    /// is cheap, labelled because it proves less than it looks like it does.
    /// A refusal must be **this** guard's refusal.
    ///
    /// ⚠⚠ `is_err()` alone is not enough, and that was measured rather than reasoned. Four of
    /// the five guards in [`build_season_composed`] are redundant with a *later* error —
    /// `Registry::new` rejects a duplicate flow id on its own, and guard 5 catches a target that
    /// never matched — so an `is_err()` test stays green with the guard deleted. Mutations M7,
    /// M9 and M10 each disabled one guard and the whole battery came back with **zero
    /// failures**. Those guards earn their place by the *message* they produce (guard 4's own
    /// comment says so: the engine would call it "duplicate flow id", which reads as an engine
    /// fault rather than as this composition asking for a replacement under the wrong name), so
    /// the message is what these tests assert.
    #[track_caller]
    fn refused_with<T>(result: Result<T, SimError>, needle: &str) {
        match result {
            Ok(_) => panic!("expected a refusal mentioning {needle:?}, but the build succeeded"),
            Err(err) => {
                let text = format!("{err:?}");
                assert!(
                    text.contains(needle),
                    "refused, but not by the guard under test: expected {needle:?} in {text:?}"
                );
            }
        }
    }

    #[test]
    fn dropping_nothing_is_the_ordinary_build() {
        let (base_state, base) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
        let (state, registry) =
            build_season_without(&DEFAULT_SCENARIO, &frozen(), &[]).expect("no drops");
        assert_eq!(flow_ids(&registry), flow_ids(&base));
        assert_eq!(registry.aux_processes().len(), base.aux_processes().len());
        assert_eq!(state.stocks.len(), base_state.stocks.len());
        assert_eq!(format!("{:?}", state.aux), format!("{:?}", base_state.aux));
    }

    /// One flow leaves; **nothing else does.** The stocks, the aux channel and the boundary
    /// sinks are the ordinary build's, which is the whole reason a registry drop was chosen
    /// over zeroing a parameter.
    #[test]
    fn dropping_one_flow_removes_that_flow_and_nothing_else() {
        let (base_state, base) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
        let (state, registry) =
            build_season_without(&DEFAULT_SCENARIO, &frozen(), &[ROOT_ZONE_CAPTURE])
                .expect("one drop");

        let gone: BTreeSet<String> = flow_ids(&base)
            .difference(&flow_ids(&registry))
            .cloned()
            .collect();
        assert_eq!(
            gone,
            BTreeSet::from([ROOT_ZONE_CAPTURE.to_string()]),
            "the drop removed something other than what it named"
        );
        assert_eq!(registry.len(), base.len() - 1);
        assert_eq!(registry.aux_processes().len(), base.aux_processes().len());
        assert_eq!(
            state.stocks.keys().collect::<Vec<_>>(),
            base_state.stocks.keys().collect::<Vec<_>>()
        );
    }

    /// Two ids at once, on the chamber build — the composer is not one-drop-only, and the
    /// sealed scenario is where the flows the open field never wires live.
    #[test]
    fn two_drops_both_land() {
        let scenario = sealed_chamber_scenario();
        let (_, base) = build_season_with(&scenario, &frozen()).expect("build");
        let ids = ["biosphere.recycling", "biosphere.decomposition"];
        for id in ids {
            assert!(flow_ids(&base).contains(id), "{id} is not in the premise");
        }
        let (_, registry) = build_season_without(&scenario, &frozen(), &ids).expect("two drops");
        assert_eq!(registry.len(), base.len() - 2);
        for id in ids {
            assert!(!flow_ids(&registry).contains(id));
        }
    }

    /// ⚠ The failure the whole module is shaped around: a miss must be loud. An unknown id,
    /// and — the live case — a real flow id that *this* scenario's flags never wired.
    #[test]
    fn a_drop_that_matches_nothing_is_an_error() {
        assert!(
            build_season_without(&DEFAULT_SCENARIO, &frozen(), &["biosphere.no_such"]).is_err()
        );
        // `recycling` exists, but only inside a sealed chamber. Dropping it from the open
        // field is the shape of "the experiment silently did nothing".
        let sealed_only = "biosphere.recycling";
        let (_, chamber) = build_season_with(&sealed_chamber_scenario(), &frozen()).expect("build");
        assert!(
            flow_ids(&chamber).contains(sealed_only),
            "the premise is gone"
        );
        assert!(build_season_without(&DEFAULT_SCENARIO, &frozen(), &[sealed_only]).is_err());
        // One good id does not license a bad one beside it.
        assert!(
            build_season_without(&DEFAULT_SCENARIO, &frozen(), &[ROOT_ZONE_CAPTURE, "nope"])
                .is_err()
        );
    }

    #[test]
    fn dropping_one_id_twice_is_refused() {
        assert!(build_season_without(
            &DEFAULT_SCENARIO,
            &frozen(),
            &[ROOT_ZONE_CAPTURE, ROOT_ZONE_CAPTURE]
        )
        .is_err());
    }

    /// A test-local flow that moves nothing, for the composition guards that need an id and
    /// no behaviour. An empty `FlowResult` is a valid no-op ([`FlowResult::empty`]).
    struct InertFlow(String);

    impl Flow for InertFlow {
        fn type_name(&self) -> &'static str {
            "InertFlow"
        }
        fn id(&self) -> &str {
            &self.0
        }
        fn evaluate(
            &self,
            _snapshot: &State,
            _env: &dyn Environment,
            _dt: f64,
        ) -> Result<FlowResult, SimError> {
            Ok(FlowResult::empty())
        }
    }

    fn inert(id: &str) -> Box<dyn Flow> {
        Box::new(InertFlow(id.to_string()))
    }

    /// A freshly built instance of `id`, taken out of a **second** ordinary build.
    ///
    /// ⚠ The only way to obtain one without re-forking the assembly: the flows are
    /// constructed inside `compartments`, which is module-private to `system.rs`
    /// (`tests/one_assembly_body.rs` pins that). So "a fresh identical instance" means *the
    /// same body, run twice*, which is exactly what the no-op control needs — and is also why
    /// that control is the weaker of the two.
    fn fresh_flow(scenario: &SeasonScenario, id: &str) -> Box<dyn Flow> {
        let (_, registry) = build_season_with(scenario, &frozen()).expect("build");
        let (flows, _) = registry.into_parts();
        flows
            .into_iter()
            .find(|f| f.id() == id)
            .unwrap_or_else(|| panic!("{id} is not in the second build"))
    }

    fn type_name_of(registry: &Registry, id: &str) -> String {
        registry
            .flows()
            .iter()
            .find(|f| f.id() == id)
            .unwrap_or_else(|| panic!("{id} is not in this registry"))
            .type_name()
            .to_string()
    }

    /// ⚠⚠ **The insertion check, without running anything.** A composer that located the
    /// target, dropped it and kept the *original* box would leave `type_name` unchanged here;
    /// the wrapper reports its own name, so the argument arriving in the target's slot is
    /// directly observable. The id set and the flow count are untouched, which is the
    /// "one named place" property.
    #[test]
    fn a_replacement_takes_its_targets_slot() {
        let (_, base) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
        let wrapped = ScaledMechanism::new(fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE), 0.5);
        let (_, registry) = build_season_replacing(
            &DEFAULT_SCENARIO,
            &frozen(),
            vec![(MAINTENANCE, Box::new(wrapped))],
        )
        .expect("replacement");

        assert_eq!(flow_ids(&registry), flow_ids(&base), "an id moved");
        assert_eq!(registry.len(), base.len());
        assert_eq!(registry.aux_processes().len(), base.aux_processes().len());
        assert_eq!(
            type_name_of(&registry, MAINTENANCE),
            "ScaledMechanism",
            "the replacement never reached the registry — the composer kept the original box"
        );
        assert_ne!(type_name_of(&base, MAINTENANCE), "ScaledMechanism");
    }

    /// The other side of the same coin, stated as a test so the blindness is on the record: a
    /// no-op replacement is invisible in the inventory. Nothing distinguishes it from the
    /// composer having ignored its argument — only the run does, and only barely (it cannot
    /// either). This is why the scaled control, not this one, is the evidence.
    #[test]
    fn a_no_op_replacement_is_invisible_in_the_inventory() {
        let (_, base) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
        let (_, registry) = build_season_replacing(
            &DEFAULT_SCENARIO,
            &frozen(),
            vec![(MAINTENANCE, fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE))],
        )
        .expect("no-op replacement");
        assert_eq!(flow_ids(&registry), flow_ids(&base));
        assert_eq!(
            type_name_of(&registry, MAINTENANCE),
            type_name_of(&base, MAINTENANCE)
        );
    }

    /// A replacement that renames is two changes wearing one name.
    #[test]
    fn a_replacement_that_renames_is_refused() {
        let other = fresh_flow(&DEFAULT_SCENARIO, ROOT_ZONE_CAPTURE);
        refused_with(
            build_season_replacing(&DEFAULT_SCENARIO, &frozen(), vec![(MAINTENANCE, other)]),
            "a replacement that renames is a drop plus an add",
        );
    }

    /// The miss, on the replace side: an unknown id, and a real flow this scenario's flags
    /// never wired.
    #[test]
    fn a_replacement_that_matches_nothing_is_an_error() {
        assert!(build_season_replacing(
            &DEFAULT_SCENARIO,
            &frozen(),
            vec![("biosphere.no_such", inert("biosphere.no_such"))]
        )
        .is_err());
        let sealed_only = "biosphere.recycling";
        assert!(build_season_replacing(
            &DEFAULT_SCENARIO,
            &frozen(),
            vec![(
                sealed_only,
                fresh_flow(&sealed_chamber_scenario(), sealed_only)
            )]
        )
        .is_err());
    }

    /// An addition lands once, and leaves everything else alone.
    #[test]
    fn an_addition_lands_once() {
        let (_, base) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
        let (_, registry) = build_season_adding(
            &DEFAULT_SCENARIO,
            &frozen(),
            vec![inert("biosphere.lab_extra")],
        )
        .expect("addition");
        assert_eq!(registry.len(), base.len() + 1);
        let gained: BTreeSet<String> = flow_ids(&registry)
            .difference(&flow_ids(&base))
            .cloned()
            .collect();
        assert_eq!(gained, BTreeSet::from(["biosphere.lab_extra".to_string()]));
        assert_eq!(type_name_of(&registry, "biosphere.lab_extra"), "InertFlow");
    }

    /// Adding an id the season already carries is a replacement asking for the wrong name.
    /// `Registry::new` would also reject it — as a duplicate id, in the engine's words.
    #[test]
    fn an_addition_that_is_already_there_is_refused() {
        refused_with(
            build_season_adding(&DEFAULT_SCENARIO, &frozen(), vec![inert(MAINTENANCE)]),
            "is already in this scenario's registry",
        );
    }

    /// ⚠ The collisions **between** the composers, reachable only through the general entry
    /// point — which is why that entry point is public. Each of these is a silent no-op or a
    /// double change if it is allowed through.
    #[test]
    fn the_cross_composer_collisions_are_refused() {
        let dropped_and_replaced = build_season_composed(
            &DEFAULT_SCENARIO,
            &frozen(),
            &[MAINTENANCE],
            vec![(MAINTENANCE, fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE))],
            Vec::new(),
        );
        refused_with(dropped_and_replaced, "is named twice as a target");

        let dropped_and_added = build_season_composed(
            &DEFAULT_SCENARIO,
            &frozen(),
            &[MAINTENANCE],
            Vec::new(),
            vec![inert(MAINTENANCE)],
        );
        refused_with(dropped_and_added, "is both a target and an addition");

        let added_twice = build_season_composed(
            &DEFAULT_SCENARIO,
            &frozen(),
            &[],
            Vec::new(),
            vec![inert("biosphere.lab_extra"), inert("biosphere.lab_extra")],
        );
        refused_with(added_twice, "is added twice");
    }

    /// The general entry point does all three at once, and the arithmetic of the flow count
    /// is the check: one gone, one swapped in place, one arrived.
    #[test]
    fn the_general_composer_takes_all_three_at_once() {
        let (_, base) = build_season_with(&DEFAULT_SCENARIO, &frozen()).expect("build");
        let wrapped = ScaledMechanism::new(fresh_flow(&DEFAULT_SCENARIO, MAINTENANCE), 0.5);
        let (_, registry) = build_season_composed(
            &DEFAULT_SCENARIO,
            &frozen(),
            &[ROOT_ZONE_CAPTURE],
            vec![(MAINTENANCE, Box::new(wrapped))],
            vec![inert("biosphere.lab_extra")],
        )
        .expect("all three");
        assert_eq!(registry.len(), base.len());
        assert!(!flow_ids(&registry).contains(ROOT_ZONE_CAPTURE));
        assert_eq!(type_name_of(&registry, MAINTENANCE), "ScaledMechanism");
        assert_eq!(type_name_of(&registry, "biosphere.lab_extra"), "InertFlow");
    }

    /// The two halves compose: a substituted **value** and a removed **process** in one
    /// build, with no third assembly path. This is the reason `p` is a parameter.
    #[test]
    fn a_substitution_and_a_drop_compose() {
        let p = super::super::biosphere_with(&[super::super::Substitution::new(
            "canopy.yaml",
            "extinction_coef",
            0.65,
        )])
        .expect("substitution");
        let (_, registry) =
            build_season_without(&DEFAULT_SCENARIO, &p, &[ROOT_ZONE_CAPTURE]).expect("both");
        assert!(!flow_ids(&registry).contains(ROOT_ZONE_CAPTURE));
        assert_eq!(p.canopy.extinction_coef, 0.65);
    }
}
