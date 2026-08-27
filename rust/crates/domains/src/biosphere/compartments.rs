//! The compartment hierarchy view and the per-compartment boundary ledger — the Rust
//! port of `domains.biosphere.compartments` (Phase-3 P3.1), written by slice S6 as the
//! prerequisite the deletion slice could not reach as a free deletion.
//!
//! # ⚠ Why this arrived last, and why it had to arrive before a single file was deleted
//!
//! Batch C recorded `compartment_boundary_ledger` as having **no Rust equivalent anywhere
//! in the tree** — a gap, not a decision. Twenty-two Python tests across
//! `tests/test_compartments.py` (15) and `tests/test_compartment_ledger.py` (7) are its
//! only checks, in either port. Deleting them with nothing standing is the pattern
//! `docs/log/scenarios-retired-c6.md` names: *retiring the LAST run reaching a branch is
//! an ORPHANING*. So the successor lands first, exactly as C6's did.
//!
//! # Why it lives here and not in `simcore`
//!
//! It reads only engine types, so `simcore` would compile it — and that is precisely the
//! placement Phase 3 refused. The hierarchy view is **domain-side by decision**
//! (`docs/plans/phase-3-parent-map-domain-side.md`; the check was `git diff src/simcore/`
//! staying empty), because conservation is enforced **globally and never per-domain**.
//! Putting a per-compartment identity inside the engine would invite exactly the
//! per-domain enforcement the roadmap rules out.
//!
//! # ⚠⚠ It is a DIAGNOSTIC and it never fails a step
//!
//! The roadmap wants `Inputs = Outputs + ΔStored` to hold *per subsystem*; the engine's
//! every-step gate is the only enforcement. This computes the same identity from the same
//! legs and state deltas so that (1) per-boundary flux is *reportable* — "net carbon
//! plants→atmosphere this step" — and (2) a **balanced-but-misapplied** delta, which nets
//! to zero globally and so is invisible to the global gate, has something that trips.
//!
//! It does **not** catch a flow wired into the wrong compartment: both sides of the
//! identity move together under a mislabel. That is a behavioural assertion per
//! cross-compartment flow, and it is stated here so the ledger is not read as covering it.

use std::collections::{BTreeMap, BTreeSet};

use simcore::events::ExtinctionEvent;
use simcore::flow::FlowResult;
use simcore::ids::{DomainId, StockId};
use simcore::quantities::Quantity;
use simcore::state::State;

use super::stocks::{ATMOSPHERE, CONSUMERS, PLANTS, SOIL, WATER};

/// The biosphere's own compartment root.
pub const BIOSPHERE: &str = "biosphere";

/// The leaf → parent map: a **flat two-level tree**, five leaves under one root.
///
/// Deliberately not deeper. Phase 3's decision was that the hierarchy exists to make a
/// four-compartment system reportable, not to model nesting the science does not have.
pub fn biosphere_parents() -> BTreeMap<DomainId, DomainId> {
    [ATMOSPHERE, SOIL, PLANTS, WATER, CONSUMERS]
        .iter()
        .map(|leaf| ((*leaf).to_string(), BIOSPHERE.to_string()))
        .collect()
}

/// Union of `root`'s own stocks and those of **all its transitive descendants**.
///
/// `domain_index` is read off `Registry`'s public accessor, never the reverse. With an
/// empty `parents` a domain has no children, so this reduces to the flat pre-hierarchy
/// behaviour — the property `flat_default_reproduces_the_domain_index` pins.
///
/// ⚠ The walk carries a `visited` set rather than recursing, so a malformed `parents`
/// containing a **cycle** terminates instead of overflowing the stack. That is not
/// defensive breadth: `a_malformed_cycle_terminates_instead_of_recursing_forever` is one
/// of the ported tests, and the Python original carries the same guard for the same
/// reason.
pub fn descendant_stocks(
    domain_index: &BTreeMap<DomainId, BTreeSet<StockId>>,
    parents: &BTreeMap<DomainId, DomainId>,
    root: &str,
) -> BTreeSet<StockId> {
    let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (child, parent) in parents {
        children.entry(parent.as_str()).or_default().push(child);
    }
    let mut collected: BTreeSet<StockId> = BTreeSet::new();
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = vec![root];
    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        if let Some(ids) = domain_index.get(node) {
            collected.extend(ids.iter().cloned());
        }
        if let Some(kids) = children.get(node) {
            stack.extend(kids.iter().copied());
        }
    }
    collected
}

/// One `(compartment, quantity)` row of the ledger.
///
/// `residual = crossing_in − crossing_out − stored_delta`, which is zero **by
/// construction on a clean step** — post-arbitration legs, no non-flow routing. The two
/// cases where it legitimately is not are expected exceptions rather than bugs: a
/// rationed step (arbitration scaled a withdrawal after the legs were read) and an
/// extinction step (a balanced non-flow change, corrected by
/// [`expected_extinction_residuals`]).
#[derive(Debug, Clone, PartialEq)]
pub struct CompartmentFlux {
    pub domain: DomainId,
    pub quantity: Quantity,
    pub crossing_in: f64,
    pub crossing_out: f64,
    pub stored_delta: f64,
    pub residual: f64,
}

/// Per-`(compartment, quantity)` boundary ledger for the step `before → after`.
///
/// `flow_results` are the step's evaluated flows; the integrator yields them in canonical
/// flow-id order and cross-flow sums fold in that order, mirroring the engine's own
/// reduction. Membership and composition are read from the **`before`** snapshot.
///
/// ⚠ **Determinism is by construction, not by convention.** Legs fold in canonical
/// stock-id order and the rows come back in canonical `(domain, quantity)` order, so the
/// result does not depend on the order legs or stocks were built in — the property
/// `the_ledger_is_leg_order_independent` pins, and #15's rule.
///
/// A fully-internal flow — every leg in one compartment — crosses no boundary and is
/// skipped. A leg books **each quantity its stock carries**, so a CO₂-pool leg books both
/// carbon and oxygen; that composition fold is the same one the global gate performs.
///
/// # Panics
///
/// On a leg naming a stock absent from `before`. Referential integrity is the apply
/// path's job, and a silent zero here would make a typo look like a clean step.
pub fn compartment_boundary_ledger(
    before: &State,
    after: &State,
    flow_results: &[FlowResult],
) -> Vec<CompartmentFlux> {
    let stocks = &before.stocks;
    let at = |id: &StockId| {
        stocks
            .get(id)
            .unwrap_or_else(|| panic!("ledger: leg names an unknown stock {id:?}"))
    };
    let mut crossing_in: BTreeMap<(DomainId, Quantity), f64> = BTreeMap::new();
    let mut crossing_out: BTreeMap<(DomainId, Quantity), f64> = BTreeMap::new();
    for result in flow_results {
        let footprint: BTreeSet<&str> = result
            .legs
            .iter()
            .map(|leg| at(&leg.stock).domain.as_str())
            .collect();
        if footprint.len() <= 1 {
            continue;
        }
        let mut legs: Vec<&simcore::flow::Leg> = result.legs.iter().collect();
        legs.sort_by(|a, b| a.stock.cmp(&b.stock));
        for leg in legs {
            let stock = at(&leg.stock);
            for (quantity, coeff) in &stock.composition {
                let folded = leg.amount * coeff;
                let key = (stock.domain.clone(), *quantity);
                if folded >= 0.0 {
                    *crossing_in.entry(key).or_insert(0.0) += folded;
                } else {
                    *crossing_out.entry(key).or_insert(0.0) -= folded;
                }
            }
        }
    }
    let mut stored: BTreeMap<(DomainId, Quantity), f64> = BTreeMap::new();
    for (sid, b) in stocks {
        let delta = after.stocks[sid].amount - b.amount;
        for (quantity, coeff) in &b.composition {
            *stored
                .entry((b.domain.clone(), *quantity))
                .or_insert(0.0) += delta * coeff;
        }
    }
    let keys: BTreeSet<(DomainId, Quantity)> = crossing_in
        .keys()
        .chain(crossing_out.keys())
        .chain(stored.keys())
        .cloned()
        .collect();
    keys.into_iter()
        .map(|key| {
            let cin = crossing_in.get(&key).copied().unwrap_or(0.0);
            let cout = crossing_out.get(&key).copied().unwrap_or(0.0);
            let sd = stored.get(&key).copied().unwrap_or(0.0);
            CompartmentFlux {
                domain: key.0,
                quantity: key.1,
                crossing_in: cin,
                crossing_out: cout,
                stored_delta: sd,
                residual: cin - cout - sd,
            }
        })
        .collect()
}

/// The per-`(compartment, quantity)` ledger correction for a step's extinctions.
///
/// ⚠⚠ Extinction is a **balanced non-flow** change the ledger cannot see, because the
/// ledger folds only flow legs. A sub-threshold POPULATION stock snaps to zero — its
/// compartment loses a residual `r` that *no leg* withdrew — and the same `r` routes to
/// the boundary-domain loss sink that *no leg* deposited. So on an extinction step the
/// **raw** residual is `+r` for the organ's compartment and `−r` for `boundary`, every
/// other row still clean. This returns exactly that expected correction, so a caller
/// asserts `|entry.residual − expected| <= tol` rather than `|entry.residual| <= tol`.
///
/// ⚠ **Kept separate from the ledger deliberately, and the reason is the point of both.**
/// "Residual ≈ 0 by construction on a clean step" is precisely what makes a *nonzero*
/// residual diagnostic; folding this correction in would blunt the check it exists for.
///
/// `before` supplies each extinct stock's compartment, read at the start-of-step
/// snapshot — extinction never moves a stock between compartments. Extinctions sharing a
/// `(compartment, quantity)` accumulate.
pub fn expected_extinction_residuals(
    before: &State,
    events: &[ExtinctionEvent],
) -> BTreeMap<(DomainId, Quantity), f64> {
    let mut out: BTreeMap<(DomainId, Quantity), f64> = BTreeMap::new();
    for ev in events {
        let domain = before.stocks[&ev.stock].domain.clone();
        *out.entry((domain, ev.quantity)).or_insert(0.0) += ev.residual;
        *out.entry((
            simcore::boundary::BOUNDARY_DOMAIN.to_string(),
            ev.quantity,
        ))
        .or_insert(0.0) -= ev.residual;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biosphere::stocks::{CARBON_POOL, LEAF_C, O2_POOL, SOIL_WATER};
    use crate::biosphere::system::{
        build_season, consumer_chamber_scenario, perennial_chamber_scenario,
        sealed_chamber_scenario, DEFAULT_SCENARIO,
    };
    use simcore::flow::Leg;
    use simcore::quantities::StockKind;
    use simcore::state::Stock;

    fn stock(id: &str, domain: &str, q: Quantity, amount: f64) -> Stock {
        Stock::new(
            id.to_string(),
            domain.to_string(),
            q,
            "mol".to_string(),
            amount,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .expect("a valid fixture stock")
    }

    fn state_of(stocks: Vec<Stock>) -> State {
        let map: BTreeMap<String, Stock> =
            stocks.into_iter().map(|s| (s.id.clone(), s)).collect();
        State::new(0, map, 0, BTreeMap::new()).expect("a valid fixture state")
    }

    fn row<'a>(ledger: &'a [CompartmentFlux], domain: &str, q: Quantity) -> &'a CompartmentFlux {
        ledger
            .iter()
            .find(|r| r.domain == domain && r.quantity == q)
            .unwrap_or_else(|| panic!("no ledger row for {domain}/{q:?}"))
    }

    // ---------------------------------------------------------------------------
    // The hierarchy view. Ported from `tests/test_compartments.py`.
    // ---------------------------------------------------------------------------

    /// The parent map is a FLAT two-level tree: five leaves, one root, no leaf is a
    /// parent. Mirrors `test_parent_map_is_flat_two_level_tree_under_biosphere`.
    #[test]
    fn the_parent_map_is_a_flat_two_level_tree_under_one_root() {
        let parents = biosphere_parents();
        assert_eq!(parents.len(), 5);
        for (leaf, parent) in &parents {
            assert_eq!(parent, BIOSPHERE, "{leaf} hangs off the root");
            assert!(
                !parents.values().any(|p| p == leaf),
                "{leaf} must not itself be a parent — the tree is two levels"
            );
        }
    }

    /// The root's descendants are the union of every leaf's stocks, and each leaf's are
    /// exactly its own members. Mirrors `test_descendant_stocks_root_unions_all_leaves`
    /// and `test_descendant_stocks_leaf_equals_its_own_members`.
    #[test]
    fn the_root_unions_every_leaf_and_a_leaf_is_exactly_its_own_members() {
        let (_state, registry) = build_season(&sealed_chamber_scenario()).expect("built");
        let index = registry.domain_index();
        let parents = biosphere_parents();

        let mut union: BTreeSet<StockId> = BTreeSet::new();
        for leaf in [ATMOSPHERE, SOIL, PLANTS, WATER, CONSUMERS] {
            let own = descendant_stocks(index, &parents, leaf);
            assert_eq!(
                own,
                index.get(leaf).cloned().unwrap_or_default(),
                "{leaf} has no children, so it is its own membership"
            );
            union.extend(own);
        }
        assert_eq!(descendant_stocks(index, &parents, BIOSPHERE), union);
        // ...and the union is not vacuous, which is what stops this passing on an empty
        // index. The sealed chamber populates at least the three carbon compartments.
        assert!(union.len() >= 3, "the roster is not empty: {union:?}");
        assert!(union.contains(LEAF_C) && union.contains(CARBON_POOL));
    }

    /// ⚠ The FLAT default reproduces the domain index exactly — the pre-hierarchy
    /// behaviour, which is what makes the parent map an addition rather than a change.
    /// Mirrors `test_flat_default_reproduces_domain_index`.
    #[test]
    fn the_flat_default_reproduces_the_domain_index() {
        let (_state, registry) = build_season(&sealed_chamber_scenario()).expect("built");
        let index = registry.domain_index();
        let flat = BTreeMap::new();
        for domain in index.keys() {
            assert_eq!(
                descendant_stocks(index, &flat, domain),
                index[domain],
                "with no parents, {domain} is its own membership"
            );
        }
        // The root itself has no stocks of its own, so flat it is EMPTY while hierarchical
        // it is everything — the one comparison that shows the map is doing work.
        assert!(descendant_stocks(index, &flat, BIOSPHERE).is_empty());
        assert!(!descendant_stocks(index, &biosphere_parents(), BIOSPHERE).is_empty());
    }

    /// The open field is a strict SUBSET of the sealed chamber: no water compartment, no
    /// consumers. Mirrors `test_descendant_stocks_open_field_is_a_subset`,
    /// `test_water_compartment_populated_when_sealed_empty_when_open` and
    /// `test_consumers_compartment_populated_when_enabled_empty_otherwise`.
    #[test]
    fn an_empty_compartment_contributes_nothing_and_the_open_field_is_a_subset() {
        let parents = biosphere_parents();
        let (_s, open) = build_season(&DEFAULT_SCENARIO).expect("built");
        let (_s, sealed) = build_season(&sealed_chamber_scenario()).expect("built");
        let (_s, consumer) = build_season(&consumer_chamber_scenario()).expect("built");

        let open_all = descendant_stocks(open.domain_index(), &parents, BIOSPHERE);
        let sealed_all = descendant_stocks(sealed.domain_index(), &parents, BIOSPHERE);
        assert!(
            open_all.is_subset(&sealed_all) && open_all.len() < sealed_all.len(),
            "the open field is a strict subset of the sealed chamber"
        );
        // The consumer compartment is populated only when the consumer is enabled, and an
        // absent compartment contributes nothing rather than erroring.
        assert!(descendant_stocks(open.domain_index(), &parents, CONSUMERS).is_empty());
        assert!(!descendant_stocks(consumer.domain_index(), &parents, CONSUMERS).is_empty());
    }

    /// ⚠ A malformed `parents` containing a CYCLE terminates instead of recursing
    /// forever. Mirrors `test_descendant_stocks_tolerates_a_malformed_cycle`.
    ///
    /// The value is that this test can only fail by hanging or overflowing, so it is
    /// written against a cycle that has a stock in it — a walk that bailed out early
    /// would come back short rather than come back at all.
    #[test]
    fn a_malformed_cycle_terminates_instead_of_recursing_forever() {
        let index: BTreeMap<DomainId, BTreeSet<StockId>> = [
            ("a".to_string(), BTreeSet::from(["x".to_string()])),
            ("b".to_string(), BTreeSet::from(["y".to_string()])),
        ]
        .into_iter()
        .collect();
        let cycle: BTreeMap<DomainId, DomainId> = [
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "a".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            descendant_stocks(&index, &cycle, "a"),
            BTreeSet::from(["x".to_string(), "y".to_string()]),
            "both members are collected exactly once"
        );
    }

    // ---------------------------------------------------------------------------
    // The ledger. Ported from `tests/test_compartment_ledger.py`.
    // ---------------------------------------------------------------------------

    /// A two-compartment transfer reports its crossing flux on both sides and balances.
    /// Mirrors `test_two_compartment_transfer_reports_crossing_flux_and_balances`.
    #[test]
    fn a_two_compartment_transfer_reports_its_crossing_flux_and_balances() {
        let before = state_of(vec![
            stock("p.c", "plants", Quantity::Carbon, 10.0),
            stock("a.c", "atmosphere", Quantity::Carbon, 10.0),
        ]);
        let after = state_of(vec![
            stock("p.c", "plants", Quantity::Carbon, 13.0),
            stock("a.c", "atmosphere", Quantity::Carbon, 7.0),
        ]);
        let flows = vec![FlowResult::new(vec![
            Leg::new("a.c".to_string(), -3.0).unwrap(),
            Leg::new("p.c".to_string(), 3.0).unwrap(),
        ])
        .unwrap()];
        let ledger = compartment_boundary_ledger(&before, &after, &flows);

        let plants = row(&ledger, "plants", Quantity::Carbon);
        assert_eq!((plants.crossing_in, plants.crossing_out), (3.0, 0.0));
        assert_eq!(plants.stored_delta, 3.0);
        assert_eq!(plants.residual, 0.0);

        let atmos = row(&ledger, "atmosphere", Quantity::Carbon);
        assert_eq!((atmos.crossing_in, atmos.crossing_out), (0.0, 3.0));
        assert_eq!(atmos.stored_delta, -3.0);
        assert_eq!(atmos.residual, 0.0);
    }

    /// A fully-internal flow crosses no boundary, so it contributes ZERO crossing flux —
    /// while its stored delta is still booked. Mirrors
    /// `test_internal_flow_contributes_zero_crossing_flux`.
    #[test]
    fn a_fully_internal_flow_contributes_no_crossing_flux() {
        let before = state_of(vec![
            stock("p.leaf", "plants", Quantity::Carbon, 10.0),
            stock("p.stem", "plants", Quantity::Carbon, 10.0),
        ]);
        let after = state_of(vec![
            stock("p.leaf", "plants", Quantity::Carbon, 6.0),
            stock("p.stem", "plants", Quantity::Carbon, 14.0),
        ]);
        let flows = vec![FlowResult::new(vec![
            Leg::new("p.leaf".to_string(), -4.0).unwrap(),
            Leg::new("p.stem".to_string(), 4.0).unwrap(),
        ])
        .unwrap()];
        let ledger = compartment_boundary_ledger(&before, &after, &flows);
        let plants = row(&ledger, "plants", Quantity::Carbon);
        assert_eq!((plants.crossing_in, plants.crossing_out), (0.0, 0.0));
        // The internal move nets to zero within the compartment, so the row is clean.
        assert_eq!(plants.stored_delta, 0.0);
        assert_eq!(plants.residual, 0.0);
    }

    /// ⚠⚠ THE CHECK THE GLOBAL GATE STRUCTURALLY CANNOT MAKE: a balanced-but-MISAPPLIED
    /// delta. Mirrors `test_balanced_but_misapplied_delta_trips_the_identity`.
    ///
    /// The step's legs say 3 mol crossed atmosphere → plants; the applied state moved 3
    /// mol out of the atmosphere and into a *third* compartment. Globally the carbon
    /// still sums, so `assert_conserved` is silent by construction. Two compartment rows
    /// go nonzero, which is the entire reason this diagnostic exists.
    #[test]
    fn a_balanced_but_misapplied_delta_trips_the_identity_the_global_gate_cannot_see() {
        let before = state_of(vec![
            stock("p.c", "plants", Quantity::Carbon, 10.0),
            stock("a.c", "atmosphere", Quantity::Carbon, 10.0),
            stock("s.c", "soil", Quantity::Carbon, 10.0),
        ]);
        // Misapplied: the soil gained what the legs gave to the plants.
        let after = state_of(vec![
            stock("p.c", "plants", Quantity::Carbon, 10.0),
            stock("a.c", "atmosphere", Quantity::Carbon, 7.0),
            stock("s.c", "soil", Quantity::Carbon, 13.0),
        ]);
        let total_before = 30.0;
        let total_after: f64 = after.stocks.values().map(|s| s.amount).sum();
        assert_eq!(
            total_after, total_before,
            "globally conserved — this is what makes the defect invisible upstream"
        );

        let flows = vec![FlowResult::new(vec![
            Leg::new("a.c".to_string(), -3.0).unwrap(),
            Leg::new("p.c".to_string(), 3.0).unwrap(),
        ])
        .unwrap()];
        let ledger = compartment_boundary_ledger(&before, &after, &flows);
        assert_eq!(row(&ledger, "plants", Quantity::Carbon).residual, 3.0);
        assert_eq!(row(&ledger, "soil", Quantity::Carbon).residual, -3.0);
        assert_eq!(row(&ledger, "atmosphere", Quantity::Carbon).residual, 0.0);
    }

    /// A leg books EVERY quantity its stock carries: a CO₂-pool leg books both carbon and
    /// oxygen. Mirrors `test_composition_fold_books_both_carbon_and_oxygen`.
    #[test]
    fn the_composition_fold_books_both_carbon_and_oxygen_for_one_leg() {
        let co2 = Stock::new(
            "a.co2".to_string(),
            "atmosphere".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            10.0,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        )
        .unwrap();
        let mut after_co2 = co2.clone();
        after_co2.amount = 7.0;
        let before = state_of(vec![co2, stock("p.c", "plants", Quantity::Carbon, 10.0)]);
        let after = state_of(vec![
            after_co2,
            stock("p.c", "plants", Quantity::Carbon, 13.0),
        ]);
        let flows = vec![FlowResult::new(vec![
            Leg::new("a.co2".to_string(), -3.0).unwrap(),
            Leg::new("p.c".to_string(), 3.0).unwrap(),
        ])
        .unwrap()];
        let ledger = compartment_boundary_ledger(&before, &after, &flows);
        let carbon = row(&ledger, "atmosphere", Quantity::Carbon);
        let oxygen = row(&ledger, "atmosphere", Quantity::Oxygen);
        assert_eq!(carbon.crossing_out, 3.0);
        // ...twice as much oxygen, from the SAME leg and the same coefficient the global
        // ledger folds with. This is the assertion a per-quantity ledger exists for.
        assert_eq!(oxygen.crossing_out, 6.0);
        assert_eq!(oxygen.stored_delta, -6.0);
        assert_eq!(oxygen.residual, 0.0);
    }

    /// ⚠ Leg order does not move a single number. Mirrors
    /// `test_boundary_ledger_is_leg_order_independent`.
    ///
    /// Not a tautology in a language with ordered containers: the fold sorts by stock id
    /// precisely so a differently-built `FlowResult` cannot produce different last bits.
    #[test]
    fn the_ledger_is_leg_order_independent() {
        let before = state_of(vec![
            stock("p.c", "plants", Quantity::Carbon, 10.0),
            stock("a.c", "atmosphere", Quantity::Carbon, 10.0),
            stock("s.c", "soil", Quantity::Carbon, 10.0),
        ]);
        let after = state_of(vec![
            stock("p.c", "plants", Quantity::Carbon, 10.3),
            stock("a.c", "atmosphere", Quantity::Carbon, 9.6),
            stock("s.c", "soil", Quantity::Carbon, 10.1),
        ]);
        let legs = |order: [(&str, f64); 3]| {
            vec![FlowResult::new(
                order
                    .iter()
                    .map(|(id, amt)| Leg::new((*id).to_string(), *amt).unwrap())
                    .collect(),
            )
            .unwrap()]
        };
        let forward = compartment_boundary_ledger(
            &before,
            &after,
            &legs([("a.c", -0.4), ("p.c", 0.3), ("s.c", 0.1)]),
        );
        let reversed = compartment_boundary_ledger(
            &before,
            &after,
            &legs([("s.c", 0.1), ("p.c", 0.3), ("a.c", -0.4)]),
        );
        assert_eq!(forward, reversed, "bit-for-bit, not merely close");
    }

    /// The ledger balances on EVERY step of a real perennial run, and the crossing flux
    /// is nonzero — the run-level check the hand-built fixtures cannot make. Mirrors
    /// `test_perennial_full_ledger_balances_every_step` and
    /// `test_perennial_cross_compartment_carbon_cycles`.
    ///
    /// ⚠ The `crossed > 0` half is the anti-vacuity guard: a ledger that folded nothing
    /// would balance perfectly and prove nothing, which is the failure mode this repo has
    /// met from several directions.
    #[test]
    fn the_ledger_balances_on_every_step_of_a_real_run_and_the_flux_is_not_zero() {
        use crate::biosphere::{season_setup, steps_for, BIO_DT};

        let scenario = perennial_chamber_scenario();
        let (mut state, integrator, resolver) = season_setup(&scenario, 1).expect("setup");
        let mut worst: f64 = 0.0;
        let mut crossed: f64 = 0.0;
        for _ in 0..steps_for(60) {
            // ⚠ The integrator does not hand back its flow results, so the legs are
            // re-evaluated here at the SAME start-of-step snapshot Euler reads. That is
            // exact rather than approximate for Euler (one evaluation per step) and it is
            // why `rationed == 0` is asserted below: on a rationed step the applied
            // deltas are the SCALED legs and these are the unscaled ones, which is one of
            // the two documented cases where the identity legitimately does not hold.
            let env = resolver.bind(&state, BIO_DT);
            let results: Vec<FlowResult> = integrator
                .registry()
                .flows()
                .iter()
                .map(|f| f.evaluate(&state, &env, BIO_DT).expect("a flow evaluates"))
                .collect();
            let report = integrator
                .step_report(&state, &resolver, BIO_DT)
                .expect("a clean step");
            assert_eq!(report.rationed, 0, "the identity assumes a clean step");
            let ledger = compartment_boundary_ledger(&state, &report.state, &results);
            for r in &ledger {
                worst = worst.max(r.residual.abs());
                crossed += r.crossing_in + r.crossing_out;
            }
            state = report.state;
        }
        assert!(worst < 1e-9, "worst per-compartment residual was {worst}");
        assert!(crossed > 0.0, "no flux crossed any boundary — the fold is vacuous");
    }

    /// Extinction books `+r` to the organ's compartment and `−r` to `boundary`, and
    /// extinctions sharing a compartment ACCUMULATE. Mirrors
    /// `test_expected_extinction_residuals_books_organ_and_boundary`,
    /// `_empty_events_is_empty` and `_accumulates_within_a_compartment`.
    #[test]
    fn extinction_books_the_organ_and_the_boundary_and_accumulates() {
        let before = state_of(vec![
            stock(LEAF_C, "plants", Quantity::Carbon, 0.0),
            stock("p.stem", "plants", Quantity::Carbon, 0.0),
            stock(SOIL_WATER, "water", Quantity::Water, 0.0),
        ]);
        assert!(
            expected_extinction_residuals(&before, &[]).is_empty(),
            "no events is no correction, not a zero-filled map"
        );

        let events = vec![
            ExtinctionEvent {
                n: 1,
                stock: LEAF_C.to_string(),
                quantity: Quantity::Carbon,
                residual: 0.25,
            },
            ExtinctionEvent {
                n: 1,
                stock: "p.stem".to_string(),
                quantity: Quantity::Carbon,
                residual: 0.75,
            },
        ];
        let out = expected_extinction_residuals(&before, &events);
        // Both organs are in `plants`, so the two corrections SUM rather than the second
        // replacing the first — the property a map-insert bug would break silently.
        assert_eq!(out[&("plants".to_string(), Quantity::Carbon)], 1.0);
        assert_eq!(
            out[&(
                simcore::boundary::BOUNDARY_DOMAIN.to_string(),
                Quantity::Carbon
            )],
            -1.0
        );
        assert_eq!(out.len(), 2, "only the quantity that snapped is booked");
    }

    /// ⚠ The two halves meet: on a hand-built extinction step the RAW ledger residual is
    /// exactly what the correction predicts, so `raw − expected` is clean. Mirrors
    /// `test_handbuilt_extinction_step_residual_and_correction`.
    ///
    /// This is the test that makes the separation of the two functions load-bearing
    /// rather than stylistic — it is the only one that evaluates them against each other.
    #[test]
    fn a_handbuilt_extinction_step_is_clean_only_after_the_correction() {
        let before = state_of(vec![
            stock(LEAF_C, "plants", Quantity::Carbon, 0.4),
            stock(CARBON_POOL, "atmosphere", Quantity::Carbon, 10.0),
            stock("boundary.loss.carbon", "boundary", Quantity::Carbon, 0.0),
        ]);
        // The organ snapped to zero and its 0.4 routed to the loss sink — no leg did
        // either, which is exactly what the ledger cannot see.
        let after = state_of(vec![
            stock(LEAF_C, "plants", Quantity::Carbon, 0.0),
            stock(CARBON_POOL, "atmosphere", Quantity::Carbon, 10.0),
            stock("boundary.loss.carbon", "boundary", Quantity::Carbon, 0.4),
        ]);
        let ledger = compartment_boundary_ledger(&before, &after, &[]);
        let expected = expected_extinction_residuals(
            &before,
            &[ExtinctionEvent {
                n: 1,
                stock: LEAF_C.to_string(),
                quantity: Quantity::Carbon,
                residual: 0.4,
            }],
        );
        for r in &ledger {
            let corrected =
                r.residual - expected.get(&(r.domain.clone(), r.quantity)).copied().unwrap_or(0.0);
            assert!(
                corrected.abs() < 1e-12,
                "{}/{:?} residual {} is not explained by the correction",
                r.domain,
                r.quantity,
                r.residual
            );
        }
        // ...and the RAW residual is genuinely nonzero, so the loop above is not passing
        // on an already-clean ledger.
        assert_eq!(row(&ledger, "plants", Quantity::Carbon).residual, 0.4);
        assert_eq!(row(&ledger, "boundary", Quantity::Carbon).residual, -0.4);
        let _ = O2_POOL;
    }
}
