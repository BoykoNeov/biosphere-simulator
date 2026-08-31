//! The **mechanism-switch lab**, first composer — a season with named flows *removed*.
//!
//! Plan: the science-switch plan (`docs/plans/post-roadmap-science-switch.md`), slice 1.
//! Where [`super::biosphere_with`] substitutes a **number** before assembly, this removes a
//! **process** after it: `build_season_with` builds the frozen machine, and the composer
//! takes one `Box<dyn Flow>` back out of the built [`Registry`].
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
//! # This module takes no decision and endorses no science
//!
//! A knockout regenerates evidence about a mechanism's contribution. It says nothing about
//! whether the mechanism should be there — the same standing `lab` has had since the value
//! half (`docs/log/value-switch-harness.md`).

use crate::biosphere::params::BiosphereParams;
use crate::biosphere::system::{build_season_with, SeasonScenario};
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::registry::Registry;
use simcore::state::State;

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
    for (i, id) in drop_ids.iter().enumerate() {
        if drop_ids[..i].contains(id) {
            return Err(SimError::Validation(format!(
                "{id:?} is dropped twice — the second drop would silently do nothing"
            )));
        }
    }
    let (state, registry) = build_season_with(scenario, p)?;
    let (flows, aux) = registry.into_parts();

    let mut matched: Vec<&str> = Vec::with_capacity(drop_ids.len());
    let mut kept: Vec<Box<dyn Flow>> = Vec::with_capacity(flows.len());
    for flow in flows {
        match drop_ids.iter().find(|id| **id == flow.id()) {
            Some(id) => matched.push(id),
            None => kept.push(flow),
        }
    }
    if matched.len() != drop_ids.len() {
        let missing: Vec<&&str> = drop_ids.iter().filter(|id| !matched.contains(id)).collect();
        return Err(SimError::Validation(format!(
            "{missing:?} is not in this scenario's registry — a drop that misses would be \
             read back as the baseline"
        )));
    }
    let registry = Registry::new(kept, &state.stocks, aux)?;
    Ok((state, registry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biosphere::params;
    use crate::biosphere::system::{sealed_chamber_scenario, DEFAULT_SCENARIO};
    use std::collections::BTreeSet;

    const ROOT_ZONE_CAPTURE: &str = "biosphere.root_zone_capture";

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
