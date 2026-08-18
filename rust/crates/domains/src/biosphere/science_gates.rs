//! The biosphere's **science gates** — the census behind the freeze manifest's
//! `science_bands` / `liveness_floors` fields (slice C4 of the reference flip).
//!
//! ## What a science gate is
//!
//! Both freeze manifests name properties of the **run** (golden bytes, `rationed == 0`,
//! no extinction, conservation, determinism). Every assertion about the *science* — that
//! the canopy is a real wheat canopy, that the closed chamber's CO₂ attractor has not
//! collapsed — used to live in test files reachable from no manifest, so none could fail
//! an unfreeze ceremony (`docs/plans/post-roadmap-acceptance-gate.md` finding 5). A
//! science gate closes that: it is one committed assertion, filed under a frozen
//! scenario, carrying the CLAIM (`quantity` / `bound` / `source`) and not just a test id.
//!
//! The two fields are kept apart deliberately and the distinction is load-bearing: a
//! **band**'s bound comes from OUTSIDE this repo (a paper, a table), a **floor**'s was
//! tuned to our own calibration. Merging two claims of different strength under one name
//! is this project's recorded failure mode, so the field is part of the declaration
//! rather than inferred from the value.
//!
//! ## Why this is in Rust as of slice C4, and what that inverted
//!
//! Until C4 the census was `tests/science_gates.py`: a static `ast` walk over
//! `@pytest.mark.science_gate` decorators in the Python suite. It produced ~104 of the
//! biosphere manifest's 208 lines — **the single largest Python-authored block of any
//! contract** — and `test_freeze_manifest.py`'s own authority note said of it: *"There is
//! no Rust referent and there cannot be."* That sentence was true of pytest markers and
//! false of the claim it appeared to make; this module is what makes it false, and
//! rewriting it is part of C4's record.
//!
//! ⚠ **The mechanism had to change with the language, and that is the design question
//! slice 8 warned about.** Rust has no runtime introspection over `#[test]` functions, so
//! a table plus a set of tests would be a **hand-maintained roster** — exactly what the
//! Python census existed to avoid ("derive from the tree, never hand-list"). The answer
//! is that the table **is** the test roster: [`science_gates!`] takes one declaration per
//! gate and emits *both* the [`GATES`] row and the `#[test]` that executes it. A row with
//! no test, or a test outside the roster, is not a gate that a meta-test has to hunt for
//! textually — it is a thing you cannot write.
//!
//! ## Why a lib module and not `tests/`
//!
//! `examples/dump_biosphere_inventory.rs` — the producer of the reference's half of
//! `docs/biosphere-reference.manifest.json` since slice 6 — needs [`GATES`] at ordinary
//! (non-test) compile time. An integration test's items are invisible to it. So the table
//! is public library data and the assertions are `#[cfg(test)]` beside it, which is also
//! what makes the emitted `locus` honest: the manifest points at the file that holds both.
//!
//! ## The `locus` and the bound literals
//!
//! `locus` is `"<this file>::<test name>"`, built by the macro from the test's own
//! identifier — it cannot drift from the test it names. The Python-side gate
//! `test_science_gate_bounds_name_a_literal_present_at_their_locus` reads the file the
//! locus points at and requires every numeric literal in `bound` to appear in it
//! textually; [`the_bound_literals_appear_at_their_locus`] is the same check run from
//! this side, so the crude-but-real "the number moved and the record did not" path is
//! closed in the reference rather than only in the checker.
//!
//! ⚠ Consequences for how bounds are spelled here, learned the expensive way: a bound
//! reading `non_collapsing(floor=5e-4)` needs the literal `5e-4` — not `0.0005` — in the
//! source, and a floor this tree *derives* (`61.07` is `Γ*/ci_ratio`, computed, never
//! typed) still needs its recorded value carried as an explicit tripwire assertion.
//! [`the_floor_is_where_the_frozen_params_put_it`] is that tripwire and is why the five
//! CO₂ gates can stay derived.
//!
//! ## Scope: 13 gates, not 15
//!
//! The two remaining markers (`crew_mission`'s respiratory-quotient prediction,
//! `sealed_station`'s thermal fixed point) are **station**-manifest keys whose referents
//! do not exist in the reference yet (the RQ helper and
//! `predicted_equilibrium_temperature`). Slices 6–8 re-anchored one manifest per slice on
//! purpose, so those two are **C4b**, scheduled with their own ceremony — recorded as a
//! split, not as a deferral.
//!
//! ⚠ **Two `source` strings still name Python tests** (`test_the_shipped_floor_is_the_
//! conservative_one_against_the_cited_route`). Their text is frozen manifest content, so
//! editing it would be a value change to the contract rather than the locus re-anchoring
//! C4 is; the companion assertions they name were ported here under the same names minus
//! the `test_` prefix, and the strings are left alone deliberately. Named as residue for
//! whichever slice retires the Python file.

// ⚠ `cfg(test)`, not a plain import: the folds are reached only from the gate bodies,
// and an unconditional `use` here would be an unused import in every ordinary build —
// which `cargo clippy --all-targets -D warnings` turns into a failure.
#[cfg(test)]
use crate::biosphere::drift::{
    is_period_2, is_stationary, non_collapsing, same_phase_diffs, year_summaries,
};

/// One committed assertion that gates a frozen scenario's science.
///
/// The mirror of Python's `ScienceGate` dataclass, minus the derivation machinery: there
/// is nothing to parse here because the declaration *is* the data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScienceGate {
    /// The frozen scenario this claim is filed under — a key of the manifest's roster.
    ///
    /// ⚠ Field order matters: [`ScienceGate`]'s `Ord` is the manifest's grouping order,
    /// and it mirrors the Python dataclass's `order=True` field order exactly
    /// (`scenario, field, quantity, bound, source, locus`). Rust's `&str` ordering is
    /// byte order over UTF-8, which agrees with Python's code-point ordering, so the two
    /// sides sort identically — a property the re-anchoring depended on being true.
    pub scenario: &'static str,
    /// `"science_bands"` (bound from outside this repo) or `"liveness_floors"` (ours).
    pub field: &'static str,
    /// What is measured, in words a reader need not open the test to understand.
    pub quantity: &'static str,
    /// The bound as recorded in the contract. Its numeric literals must appear in this
    /// file — see the module note.
    pub bound: &'static str,
    /// Where the bound comes from. For a floor, honestly `"self — ..."`.
    pub source: &'static str,
    /// `"<file>::<test name>"`, generated from the test's identifier by the macro.
    pub locus: &'static str,
}

/// This file, repo-relative — the path half of every [`ScienceGate::locus`].
///
/// ⚠ Duplicated as a literal inside [`science_gates!`] because `concat!` takes only
/// literals. The duplication cannot rot silently:
/// [`the_bound_literals_appear_at_their_locus`] resolves every locus against the
/// filesystem, so a moved file turns it red on both sides.
pub const GATE_SOURCE_FILE: &str = "rust/crates/domains/src/biosphere/science_gates.rs";

/// The `science_bands` field name — a claim bounded by a source outside this repo.
pub const SCIENCE_BANDS: &str = "science_bands";
/// The `liveness_floors` field name — a claim tuned to our own calibration.
pub const LIVENESS_FLOORS: &str = "liveness_floors";

/// Declare science gates: each entry emits **both** a [`GATES`] row and the `#[test]`
/// that executes it.
///
/// This is the whole census mechanism. There is no registry, no introspection and no
/// meta-test hunting for unexercised rows, because the failure those would look for is
/// unrepresentable: the row and the assertion are one declaration. The cost is that the
/// table cannot be assembled from several files — which is a feature here, since a census
/// spread over a tree is what needed a static parser in the first place.
macro_rules! science_gates {
    ($(
        $(#[$attr:meta])*
        gate $name:ident {
            scenario: $scenario:literal,
            field: $field:literal,
            quantity: $quantity:literal,
            bound: $bound:literal,
            source: $source:literal,
            check: $body:block
        }
    )+) => {
        /// Every science gate in the reference, in declaration order.
        ///
        /// ⚠ Declaration order is NOT manifest order — the manifest groups by scenario
        /// and sorts by [`ScienceGate`]'s `Ord`. The dumper sorts; this array does not,
        /// so gates can be declared next to the runs they read.
        pub const GATES: &[ScienceGate] = &[
            $(ScienceGate {
                scenario: $scenario,
                field: $field,
                quantity: $quantity,
                bound: $bound,
                source: $source,
                locus: concat!(
                    "rust/crates/domains/src/biosphere/science_gates.rs::",
                    stringify!($name)
                ),
            },)+
        ];

        #[cfg(test)]
        mod gate_tests {
            use super::*;

            $(
                $(#[$attr])*
                #[test]
                fn $name() $body
            )+
        }
    };
}

// ---------------------------------------------------------------------------------
// The runs the gates read — collected ONCE each, as pre-reduced per-step scalars.
// ---------------------------------------------------------------------------------

#[cfg(test)]
mod runs {
    use crate::biosphere::stocks::{CARBON_POOL, CONSUMER_CARBON, LEAF_C, STEM_C, STORAGE_C};
    use crate::biosphere::system::{
        consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario,
    };
    use crate::biosphere::{
        run_perennial, run_season, season_setup, season_steps, steps_for, steps_for_years,
        SeasonScenario, BIO_DT, CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, LONG_HORIZON_YEARS,
        PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS, SEASON_DAYS,
    };
    use simcore::state::State;
    use std::sync::OnceLock;

    /// One trajectory, reduced to the scalar series the gates fold.
    ///
    /// ⚠⚠ **Pre-reduction is the station's precedent, and it opens a hole Python does not
    /// have.** `station/examples/emit_sealed_energy_drift.rs` already folds a per-step
    /// temperature series rather than materializing 109,801 `State`s, and `year_summaries`
    /// is generic precisely so it can. But that fold computes `n_years = (len - 1) / year`,
    /// so an observer emitting `steps` samples instead of `steps + 1` yields **14** annual
    /// summaries instead of 15 — and every gate still passes, because `non_collapsing`
    /// over 14 years passes exactly as well as over 15. Python never needed a guard for
    /// that; the pre-reduction is what creates it. Hence [`Trajectory::years`] and the
    /// count assertion in every decade gate.
    pub struct Trajectory {
        pub scenario: SeasonScenario,
        /// `biosphere.leaf_c` per step (initial state included).
        pub leaf_c: Vec<f64>,
        /// `biosphere.stem_c` per step.
        pub stem_c: Vec<f64>,
        /// `biosphere.storage_c` per step.
        pub storage_c: Vec<f64>,
        /// `biosphere.carbon_pool` per step — the chamber atmosphere.
        pub carbon_pool: Vec<f64>,
        /// `biosphere.consumer_carbon` per step, or empty where the stock is absent.
        pub consumer_c: Vec<f64>,
        /// Arbitration firings over the whole run. A band is a claim about a *well-fed*
        /// run; a rationed run's trace is not the model's answer.
        pub rationed: u64,
        /// Extinction events over the whole run.
        pub events: usize,
        /// Seasons run — what the annual summary count must equal.
        pub years: usize,
    }

    impl Trajectory {
        /// Samples per season, in **steps** (the unit the trajectory is indexed by).
        pub fn year(&self) -> usize {
            steps_for(SEASON_DAYS)
        }
    }

    fn trajectory(scenario: SeasonScenario, years: usize, perennial: bool) -> Trajectory {
        let (state, integrator, resolver) = season_setup(&scenario, years).expect("season setup");
        let steps = steps_for_years(years);
        let mut t = Trajectory {
            scenario,
            leaf_c: Vec::with_capacity(steps + 1),
            stem_c: Vec::with_capacity(steps + 1),
            storage_c: Vec::with_capacity(steps + 1),
            carbon_pool: Vec::with_capacity(steps + 1),
            consumer_c: Vec::new(),
            rationed: 0,
            events: 0,
            years,
        };
        {
            let mut observe = |s: &State| {
                t.leaf_c.push(s.stocks[LEAF_C].amount);
                t.stem_c.push(s.stocks[STEM_C].amount);
                t.storage_c.push(s.stocks[STORAGE_C].amount);
                // ⚠ Both of these are present only in some scenarios, and the empty
                // series that leaves is a SILENT-PASS hazard, not a convenience: an open
                // field has no `biosphere.carbon_pool` at all (unsealed runs draw on the
                // boundary atmosphere), and only the consumer chambers carry a herbivore.
                // A fold over an empty series returns the identity — `min` returns
                // +infinity, which is happily "above the compensation point". The folds
                // that read them assert non-emptiness for exactly that reason.
                if let Some(stock) = s.stocks.get(CARBON_POOL) {
                    t.carbon_pool.push(stock.amount);
                }
                if let Some(stock) = s.stocks.get(CONSUMER_CARBON) {
                    t.consumer_c.push(stock.amount);
                }
            };
            let (_final, rationed, events) = if perennial {
                run_perennial(
                    &integrator,
                    state,
                    &scenario,
                    &resolver,
                    BIO_DT,
                    steps,
                    season_steps(),
                    &mut observe,
                )
                .expect("perennial run")
            } else {
                run_season(
                    &integrator,
                    state,
                    &resolver,
                    BIO_DT,
                    steps,
                    None,
                    &mut observe,
                )
                .expect("season run")
            };
            t.rationed = rationed;
            t.events = events.len();
        }
        // The observer contract this whole module's arithmetic rests on: `run_season`
        // calls it on the initial state AND each produced state. Asserted rather than
        // trusted — see the `Trajectory` note.
        assert_eq!(t.leaf_c.len(), steps + 1, "observer sample count");
        t
    }

    macro_rules! cached_run {
        ($(#[$attr:meta])* $name:ident = ($scenario:expr, $years:expr, $perennial:expr);) => {
            $(#[$attr])*
            pub fn $name() -> &'static Trajectory {
                static CELL: OnceLock<Trajectory> = OnceLock::new();
                CELL.get_or_init(|| trajectory($scenario, $years, $perennial))
            }
        };
    }

    // ⚠ Cached, and not for tidiness. Python runs each decade trajectory once through a
    // `scope="module"` fixture; Rust has no fixtures, so thirteen independent `#[test]`s
    // would re-run the two 15-year chambers several times each. `OnceLock` is the
    // fixture's analogue and it is thread-safe, which matters because the test harness
    // runs these concurrently.
    //
    // ⚠ Each scenario is driven THE WAY ITS OWN GOLDEN DRIVES IT — `sealed_chamber`
    // through `run_season` with no re-sow, the other four through `run_perennial`'s
    // annual reset. Driving them uniformly is how the sealed chamber once acquired a
    // compensation-point crossing it does not have.
    cached_run! {
        /// The open field, one season — the reference scenario.
        open_season = (DEFAULT_SCENARIO, 1, false);
    }
    cached_run! {
        /// The sealed chamber, 3 seasons, no re-sow.
        sealed_chamber = (sealed_chamber_scenario(), SEALED_CHAMBER_YEARS, false);
    }
    cached_run! {
        /// The perennial chamber at its own frozen horizon.
        perennial_chamber = (perennial_chamber_scenario(), PERENNIAL_CHAMBER_YEARS, true);
    }
    cached_run! {
        /// The consumer chamber at its own frozen horizon.
        consumer_chamber = (consumer_chamber_scenario(), CONSUMER_CHAMBER_YEARS, true);
    }
    cached_run! {
        /// The perennial chamber over the 15-year decade horizon.
        perennial_long = (perennial_chamber_scenario(), LONG_HORIZON_YEARS, true);
    }
    cached_run! {
        /// The consumer chamber over the 15-year decade horizon.
        consumer_long = (consumer_chamber_scenario(), LONG_HORIZON_YEARS, true);
    }
}

// ---------------------------------------------------------------------------------
// The folds the gates share.
// ---------------------------------------------------------------------------------

#[cfg(test)]
mod folds {
    use super::runs::Trajectory;
    use crate::biosphere::params::{canopy, photosynthesis, MOLAR_MASS_CARBON_KG_PER_MOL};
    use crate::biosphere::science::leaf_area_index;
    use crate::biosphere::system::sealed_chamber_scenario;

    /// kg C / kg DM — Greenwood's basis (`nitrogen.yaml` / `canopy.yaml`, cited).
    pub const CARBON_FRACTION: f64 = 0.45;

    /// The CO₂ compensation point in **chamber** ppm, from the frozen params.
    ///
    /// `Γ*` is the compensation point in the *intercellular* air; the gate is on the
    /// *ambient* air, and the two are related by the C3 set point `Ci = ci_ratio · Ca`
    /// the sealed carbon budget already uses. So the ambient floor is `Γ*/ci_ratio`.
    /// Computed, never typed — which is why the recorded 61.07 needs its own tripwire.
    pub fn floor_ppm() -> f64 {
        photosynthesis().gamma_star / sealed_chamber_scenario().ci_ratio
    }

    /// Minimum chamber CO₂ (ppm) over the whole trajectory.
    pub fn min_ppm(t: &Trajectory) -> f64 {
        // ⚠ Not defensive tidying: an empty series folds to +infinity, which passes
        // `min > floor` vacuously. An unsealed scenario reaching this fold is a wiring
        // error that must be loud, not a band that quietly holds.
        assert!(
            !t.carbon_pool.is_empty(),
            "min_ppm on a run with no chamber carbon pool"
        );
        let air = t.scenario.chamber_air_mol;
        t.carbon_pool
            .iter()
            .map(|c| c / air * 1e6)
            .fold(f64::INFINITY, f64::min)
    }

    /// Peak leaf area index over the whole trajectory.
    pub fn peak_lai(t: &Trajectory) -> f64 {
        let sla = canopy().sla_per_mol_c;
        t.leaf_c
            .iter()
            .map(|c| leaf_area_index(*c, sla, t.scenario.ground_area))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// mol C → t DM/ha on Greenwood's basis (1 kg/m² == 10 t/ha).
    pub fn t_per_ha(mol_c: f64, ground_area: f64) -> f64 {
        ((mol_c * MOLAR_MASS_CARBON_KG_PER_MOL / CARBON_FRACTION) / ground_area) * 10.0
    }

    /// Peak whole-plant mass **excluding fibrous roots** (t/ha) — Greenwood's W.
    pub fn peak_w(t: &Trajectory) -> f64 {
        (0..t.leaf_c.len())
            .map(|i| {
                t_per_ha(
                    t.leaf_c[i] + t.stem_c[i] + t.storage_c[i],
                    t.scenario.ground_area,
                )
            })
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Per-year maximum of a per-step series (the `_peak_leaf` / segment-max fold).
    pub fn segment_max(seg: &[f64]) -> f64 {
        seg.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }

    /// Per-year minimum of a per-step series (the `_min_carbon_pool` fold).
    pub fn segment_min(seg: &[f64]) -> f64 {
        seg.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }

    /// Per-year LAST value of a per-step series (the `_year_end_consumer` fold).
    pub fn segment_last(seg: &[f64]) -> f64 {
        *seg.last().expect("non-empty year segment")
    }

    /// `max` over a slice — the scale the relative stationarity bounds are taken against.
    pub fn scale_of(values: &[f64]) -> f64 {
        segment_max(values)
    }
}

// ---------------------------------------------------------------------------------
// Shared constants of the ported gates (the reference's, not new choices).
// ---------------------------------------------------------------------------------

/// Van Keulen & Seligman 1987's mutual-shading loss rate, via [A] p. 101 (1/day).
pub const VKS_SHADE_RATE: f64 = 0.05;
/// Van Keulen & Seligman 1987's mutual-shading LAI threshold (m² m⁻²).
pub const VKS_LAI_THRESHOLD: f64 = 6.0;
/// Teh's specificity factor τ at 25 °C — the alternative route to `Γ*` (eq. 6.19).
pub const TEH_SPECIFICITY_FACTOR: f64 = 2600.0;

#[cfg(test)]
/// Same-phase diffs dropped before the non-amplifying trend is read (the sow-in).
///
/// ⚠ 2 → 3 on 2026-08-15 (the depth-resolved canopy). This is a TRANSIENT LENGTH, not a
/// tolerance: the same-phase differences do stop amplifying and trend to zero, and
/// nothing was widened to accommodate the change.
const TRANSIENT: usize = 3;
#[cfg(test)]
/// Years dropped before the period check — enough to reach the settled tail.
const PERIOD_TRANSIENT: usize = 8;
#[cfg(test)]
/// The cycle period the same-phase differences are read on.
const CYCLE_PERIOD: usize = 2;

// ---------------------------------------------------------------------------------
// The 13 gates.
// ---------------------------------------------------------------------------------

science_gates! {
    /// The baseline canopy band: a real wheat canopy peaks at ~5–8 LAI.
    ///
    /// The sibling band to the mutual-shading gate below, and the reason that one could
    /// be RESTATED rather than re-tuned: a peak of 6.02 is a perfectly ordinary wheat
    /// canopy by this band, so "above the shading threshold" cannot mean "unphysical".
    gate open_season_canopy_is_physical {
        scenario: "open_season",
        field: "science_bands",
        quantity: "peak LAI (m2 m-2)",
        bound: "5.0 < peak < 8.0",
        source: "real wheat peaks at ~5-8 LAI",
        check: {
            let t = runs::open_season();
            assert_eq!(t.rationed, 0, "the band run must be well-fed");
            let peak = folds::peak_lai(t);
            assert!(5.0 < peak && peak < 8.0, "peak LAI {peak}");
        }
    }

    /// ⚠ THE TRIPWIRE THAT FIRED, and this gate is its restatement.
    ///
    /// It read `peak < 6.0` for every scenario until 2026-08-15, when binding
    /// `specific_leaf_area` to its primary source grew the open-field canopy 11.9 %
    /// (5.3806 → 6.0228) and started a sourced, non-fitted mechanism firing inside a
    /// frozen scenario.
    ///
    /// ⚠ Re-tuning would have moved 6.0 to fit 6.0228; nothing of the sort happened. 6.0
    /// is still the threshold and the tree is still measured against it. What changed is
    /// the *consequence* of being above it: the old bound was a proxy for "no cited
    /// mechanism is firing that this tree fails to model", and that guarantee is now met
    /// by MODELLING the mechanism instead of by staying out of its way. It could not have
    /// been met any other way — the loss leaves the peak bit-identically unchanged,
    /// because the canopy crosses the threshold *at* its summit and the loss acts on the
    /// way down.
    ///
    /// ⚠ The roster is the four scenarios the reference carries. It was six until
    /// 2026-08-18, when C6 retired `n_limited` and `water_biting`; measured before that
    /// deletion, the departing peaks were 0.0869 and 0.4718 while the pinned
    /// `max(chambers)` is 0.5849 (`consumer_chamber`, a survivor), so this gate's numbers
    /// did not move and its claim is not truncated.
    gate the_vks_mutual_shading_regime_is_modelled_not_merely_avoided {
        scenario: "open_season",
        field: "science_bands",
        quantity: "peak LAI (m2 m-2) vs the mutual-shading threshold",
        bound: "peak < 6.0 OR the 5%/day mutual-shading loss is MODELLED",
        source: "Van Keulen & Seligman 1987 mutual-shading threshold, via [A] p. 101",
        check: {
            let peaks = [
                ("open_season", folds::peak_lai(runs::open_season())),
                ("sealed_chamber", folds::peak_lai(runs::sealed_chamber())),
                ("perennial_chamber", folds::peak_lai(runs::perennial_chamber())),
                ("consumer_chamber", folds::peak_lai(runs::consumer_chamber())),
            ];

            // The half of the original claim that is UNCHANGED, and still worth freezing:
            // the chambers are carbon-limited by design and cannot reach the regime.
            let chambers = folds::segment_max(
                &peaks
                    .iter()
                    .filter(|(name, _)| *name != "open_season")
                    .map(|(_, peak)| *peak)
                    .collect::<Vec<f64>>(),
            );
            assert!(chambers < 1.0, "chamber peak LAI {chambers} — {peaks:?}");

            // The restated guarantee, for EVERY scenario: be below the threshold, or
            // model the loss the source prescribes above it. Never in the regime,
            // unmodelled.
            let sen = crate::biosphere::params::senescence();
            for (name, peak) in peaks {
                assert!(
                    peak < VKS_LAI_THRESHOLD || sen.shade_rate > 0.0,
                    "{name}: {peak}"
                );
            }

            // ...and "modelled" must mean the CITED mechanism, not a knob that happens to
            // be non-zero. The constants are the source's and the form is its step.
            assert_eq!(sen.shade_rate, VKS_SHADE_RATE);
            assert_eq!(sen.lai_threshold, VKS_LAI_THRESHOLD);
            let below = crate::biosphere::science::mutual_shading_rate(
                VKS_LAI_THRESHOLD,
                sen.rdr_leaf,
                sen.shade_rate,
                sen.lai_threshold,
            );
            let above = crate::biosphere::science::mutual_shading_rate(
                VKS_LAI_THRESHOLD + 1e-9,
                sen.rdr_leaf,
                sen.shade_rate,
                sen.lai_threshold,
            );
            assert_eq!(below, sen.rdr_leaf, "inert AT the threshold (strict >)");
            assert_eq!(above, sen.rdr_leaf + sen.shade_rate);

            // And it genuinely bites on the scenario that crossed — a mechanism present
            // but never reached would satisfy everything above and guard nothing.
            assert!(peaks[0].1 > VKS_LAI_THRESHOLD, "{peaks:?}");
        }
    }

    /// ⚠ THE LOAD-BEARING MARGIN: `f_N == 1` across the frozen set is NOT structural.
    ///
    /// `open_season` is the only frozen scenario that enters Greenwood's declining branch
    /// at all, and it peaks well under the crossing mass at which the crop's target
    /// nitrogen concentration meets `n_critical`. A calibration growing the open-field
    /// crop enough pushes the target below it. That is exactly the kind of claim that
    /// rots silently in prose, so it is asserted.
    ///
    /// ⚠ As a guard it fires *before* the earliest measured bite (15.068 t/ha), which is
    /// the right direction for a tripwire to err in.
    gate open_season_peaks_below_the_greenwood_crossing {
        scenario: "open_season",
        field: "science_bands",
        quantity: "peak W excl. fibrous roots (t/ha)",
        bound: "peak_w < 14.4248",
        source: "Greenwood 1990 eqn (6) a=5.697 meets n_critical=1.5",
        check: {
            let t = runs::open_season();
            assert_eq!(t.rationed, 0);
            let w = folds::peak_w(t);
            assert!(w < 14.4248, "open_season entered the stressed branch — f_N moved: {w}");
        }
    }

    /// The sealed chamber never crosses the compensation point.
    ///
    /// ⚠ This scenario spent three days named as the crossing's locus, so the gate is
    /// worth more than its margin: it pins the CONFIGURATION (`run_season`, no re-sow)
    /// that makes the number what it is. Re-measure it through `run_perennial` and it
    /// reads a different, lower value on the same tree.
    gate sealed_chamber_stays_above_the_compensation_point {
        scenario: "sealed_chamber",
        field: "science_bands",
        quantity: "season-low chamber CO₂ (ppm)",
        bound: "min > Γ*/ci_ratio (61.07 ppm)",
        source: "FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. 1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so the verdict is provenance-insensitive — test_the_shipped_floor_is_the_conservative_one_against_the_cited_route",
        check: { band_gate(runs::sealed_chamber()); }
    }

    /// ⚠ THE ONE THAT WAS RED: 56.03 ppm at `dt = 1`, 70.25 at the shipped step.
    ///
    /// This is the scenario the whole step unfreeze was authorised on, and the gate that
    /// should have existed before it. It is also the tightest of the five.
    gate perennial_chamber_stays_above_the_compensation_point {
        scenario: "perennial_chamber",
        field: "science_bands",
        quantity: "season-low chamber CO₂ (ppm)",
        bound: "min > Γ*/ci_ratio (61.07 ppm)",
        source: "FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. 1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so the verdict is provenance-insensitive — test_the_shipped_floor_is_the_conservative_one_against_the_cited_route",
        check: { band_gate(runs::perennial_chamber()); }
    }

    /// The consumer chamber — was the tightest of the five, is now the loosest.
    gate consumer_chamber_stays_above_the_compensation_point {
        scenario: "consumer_chamber",
        field: "science_bands",
        quantity: "season-low chamber CO₂ (ppm)",
        bound: "min > Γ*/ci_ratio (61.07 ppm)",
        source: "FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. 1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so the verdict is provenance-insensitive — test_the_shipped_floor_is_the_conservative_one_against_the_cited_route",
        check: { band_gate(runs::consumer_chamber()); }
    }

    /// The 15-year perennial run's minimum is the SAME minimum as the 5-year run's, taken
    /// in year 2 — the trough is inside the shorter horizon, not beyond it.
    gate perennial_long_horizon_stays_above_the_compensation_point {
        scenario: "perennial_long_horizon",
        field: "science_bands",
        quantity: "season-low chamber CO₂ (ppm)",
        bound: "min > Γ*/ci_ratio (61.07 ppm)",
        source: "FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. 1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so the verdict is provenance-insensitive — test_the_shipped_floor_is_the_conservative_one_against_the_cited_route",
        check: { band_gate(runs::perennial_long()); }
    }

    /// The 15-year consumer run — again the same minimum as its 5-year sibling, in year 5.
    gate consumer_long_horizon_stays_above_the_compensation_point {
        scenario: "consumer_long_horizon",
        field: "science_bands",
        quantity: "season-low chamber CO₂ (ppm)",
        bound: "min > Γ*/ci_ratio (61.07 ppm)",
        source: "FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. 1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so the verdict is provenance-insensitive — test_the_shipped_floor_is_the_conservative_one_against_the_cited_route",
        check: { band_gate(runs::consumer_long()); }
    }

    /// The perennial chamber's per-year peak leaf carbon is stationary AND alive.
    ///
    /// ⚠ `non_collapsing` is the mandatory companion, not a stylistic one: a cycle
    /// decaying toward extinction has shrinking same-phase diffs, so `is_stationary`
    /// alone is blind to it. Only a LEVEL check on the summaries catches it.
    ///
    /// ⚠ This was one parametrized Python test carrying TWO markers; here the row is the
    /// test, so it is two tests with two loci. Same claims, same numbers.
    gate perennial_decade_leaf_cycle_is_stationary_and_alive {
        scenario: "perennial_long_horizon",
        field: "liveness_floors",
        quantity: "annual peak leaf carbon (mol C)",
        bound: "non_collapsing(floor=0.05)",
        source: "self — the calibrated attractor, not a cited value",
        check: { leaf_cycle_gate(runs::perennial_long()); }
    }

    /// The consumer chamber's per-year peak leaf carbon is stationary AND alive.
    gate consumer_decade_leaf_cycle_is_stationary_and_alive {
        scenario: "consumer_long_horizon",
        field: "liveness_floors",
        quantity: "annual peak leaf carbon (mol C)",
        bound: "non_collapsing(floor=0.05)",
        source: "self — the calibrated attractor, not a cited value",
        check: { leaf_cycle_gate(runs::consumer_long()); }
    }

    /// The perennial chamber settles to a period-1 FIXED POINT, not a period-2 cycle.
    ///
    /// ⚠ FLIPPED, not weakened. This asserted a period-2 limit cycle until 2026-07-20;
    /// that cycle was a property of the BROKEN CANOPY REGIME, and with vernalization +
    /// photoperiod the canopy closes, Beer–Lambert saturates, the return map's slope drops
    /// below 1 and the 2-cycle loses stability — converging upward to a fixed point.
    ///
    /// ⚠ RESTATED by the humification split (2026-08-10): the settling transient
    /// lengthened from ~3 years to ~35 because the humus pool fills on its own ~5-yr
    /// turnover, so at year 15 the chamber is still CONVERGING. An equality-shaped pin
    /// would assert something false; what is true is a monotone, decelerating approach.
    ///
    /// ⚠ The floor has moved twice (`> 1.0` → `> 0.9` → `> 0.55`) and its rationale
    /// INVERTED on 2026-08-15: the 50-year equilibrium now settles at 0.543748, BELOW the
    /// floor, so what this checks is the 15-year trajectory rather than the attractor it
    /// was named for. Recorded, not re-anchored — moving 0.55 to fit is the refused
    /// co-adaptation.
    gate perennial_leaf_cycle_is_a_fixed_point {
        scenario: "perennial_long_horizon",
        field: "liveness_floors",
        quantity: "converged peak-leaf fixed point (mol C)",
        bound: "max(tail) > 0.55",
        source: "self — originally anchored BELOW the measured 50-yr equilibrium (0.594984, reached ~yr 45), not on the 15-yr reading; 2.2x the 0.253 dead baseline. Moves: >1.0 -> >0.9 (decomposer calibration) -> >0.55 (humification split). ⚠⚠ THAT RATIONALE INVERTED 2026-08-15 (the layered canopy): the 50-yr equilibrium now settles at 0.543748, BELOW the floor. The bound is NOT re-anchored and is NOT red — it reads max(tail) on the 15-YEAR run, which is 0.578137 and clears by 5.1 %. So what this floor now checks is the 15-yr trajectory, not the attractor it was named for; both numbers are asserted in the tests so the gap is visible. Moving 0.55 to fit would be the refused co-adaptation",
        check: {
            let t = runs::perennial_long();
            let summaries = year_summaries(&t.leaf_c, t.year(), folds::segment_max);
            assert_eq!(summaries.len(), t.years, "annual summary count");
            // ⚠ `min_rel_gap` is spelled 1e-3 — Python's default. C4's own gating
            // measurement first compared this against 1e-2 and got `false` from both
            // sides, which looked like agreement and was two different parameterizations
            // landing on the same answer. A matching boolean from different arguments is
            // `[] == []` in a passing test's clothes.
            assert!(!is_period_2(&summaries, PERIOD_TRANSIENT, 1e-3));
            let tail = &summaries[PERIOD_TRANSIENT..];
            let diffs: Vec<f64> = (0..tail.len() - 1).map(|k| tail[k + 1] - tail[k]).collect();
            assert!(diffs.iter().all(|d| *d < 0.0), "monotone decline: {diffs:?}");
            assert!(
                (0..diffs.len() - 1).all(|k| diffs[k + 1].abs() < diffs[k].abs()),
                "decelerating — converging, not running away: {diffs:?}"
            );
            let scale = folds::scale_of(tail);
            assert!(diffs[diffs.len() - 1].abs() < 1e-2 * scale, "the approach is slow");
            assert!(scale > 0.55, "peak-leaf fixed point {scale}");
        }
    }

    /// The consumer trophic level persists: its standing biomass reaches a stationary,
    /// non-collapsing attractor over the decade — neither blowing up nor starving.
    gate decade_consumer_biomass_is_stationary_and_alive {
        scenario: "consumer_long_horizon",
        field: "liveness_floors",
        quantity: "year-end consumer carbon (mol C)",
        bound: "non_collapsing(floor=5e-4)",
        source: "self — the calibrated attractor, not a cited value",
        check: {
            let t = runs::consumer_long();
            let summaries = year_summaries(&t.consumer_c, t.year(), folds::segment_last);
            assert_eq!(summaries.len(), t.years, "annual summary count");
            let diffs = same_phase_diffs(&summaries, CYCLE_PERIOD);
            let scale = folds::scale_of(&summaries);
            assert!(is_stationary(&diffs, 0.2 * scale, 0.02 * scale, TRANSIENT));
            assert!(non_collapsing(&summaries, 5e-4), "{summaries:?}");
        }
    }

    /// The chamber's per-year minimum CO₂ pool stays bounded and never approaches
    /// exhaustion.
    ///
    /// ⚠ WHAT THIS GUARD ACTUALLY DETECTS, measured rather than inherited: not "closure
    /// is draining the atmosphere into biomass" — the drain mechanism is the recycling
    /// loop, and slowing it moves this trough the WRONG way. What the floor tracks is the
    /// chamber's BUFFER against the crop's peak demand.
    ///
    /// ⚠ The floor is anchored on the trough's MEASURED attractor, not on this horizon's
    /// reading, and it did NOT follow the ~35 % rise the step unfreeze produced. That is
    /// deliberate: re-anchoring a floor upward every time the reference moves is how a
    /// floor becomes a restatement of the current run. The clearance is recorded instead.
    gate decade_min_carbon_pool_stationary {
        scenario: "perennial_long_horizon",
        field: "liveness_floors",
        quantity: "annual minimum chamber CO2 pool (mol C)",
        bound: "non_collapsing(floor=0.05)",
        source: "self — anchored on the MEASURED trough attractor, converged well before yr 50, not on a 15-yr reading. ⚠ The attractor has moved and the CLEARANCE with it: 0.0758448 / 1.52x (2026-08-14, the light path) -> 0.072238 / 1.44x (2026-08-15, the layered canopy). The floor is not re-anchored; the ratio is asserted separately so a shrinking clearance cannot pass unread; teeth witnessed by a mutation independent of any candidate science change (the jar shrunk 0.65x at fixed composition trips it at 0.0492366). Window removed: floor[2:] -> floor",
        check: {
            let t = runs::perennial_long();
            let summaries = year_summaries(&t.carbon_pool, t.year(), folds::segment_min);
            assert_eq!(summaries.len(), t.years, "annual summary count");
            let diffs = same_phase_diffs(&summaries, CYCLE_PERIOD);
            let scale = folds::scale_of(&summaries);
            assert!(is_stationary(&diffs, 0.2 * scale, 0.02 * scale, TRANSIENT));
            // ⚠ No `[TRANSIENT:]` window: `non_collapsing(whole)` implies
            // `non_collapsing(sliced)`, so removing it TIGHTENED the check. A window that
            // is inert on the reference and load-bearing only on candidates is the one
            // shape a frozen contract's guard must not have.
            assert!(non_collapsing(&summaries, 0.05), "{summaries:?}");
        }
    }
}

// ---------------------------------------------------------------------------------
// The gate bodies shared by more than one row, and the census's own checks.
// ---------------------------------------------------------------------------------

#[cfg(test)]
mod support {
    use super::runs::Trajectory;
    use super::{folds, CYCLE_PERIOD, TRANSIENT};
    use crate::biosphere::drift::{
        is_stationary, non_collapsing, same_phase_diffs, year_summaries,
    };

    /// The CO₂ band, asserted the same way for all five scenarios.
    ///
    /// The pre-conditions are part of the claim, not hygiene: a band is a statement about
    /// a **closed, well-fed** run, and a rationed or extinction-hit trace is not the
    /// model's answer.
    pub fn band_gate(t: &Trajectory) {
        assert_eq!(t.rationed, 0, "band run must be well-fed");
        assert_eq!(t.events, 0, "band run must be extinction-free");
        let min = folds::min_ppm(t);
        let floor = folds::floor_ppm();
        assert!(min > floor, "season-low {min} ppm vs floor {floor} ppm");
    }

    /// The per-year peak-leaf stationarity + liveness pair, for both decade chambers.
    pub fn leaf_cycle_gate(t: &Trajectory) {
        let summaries = year_summaries(&t.leaf_c, t.year(), folds::segment_max);
        assert_eq!(summaries.len(), t.years, "annual summary count");
        let diffs = same_phase_diffs(&summaries, CYCLE_PERIOD);
        let scale = folds::scale_of(&summaries);
        assert!(is_stationary(&diffs, 0.1 * scale, 0.01 * scale, TRANSIENT));
        assert!(non_collapsing(&summaries, 0.05), "{summaries:?}");
    }
}

#[cfg(test)]
use support::{band_gate, leaf_cycle_gate};

#[cfg(test)]
mod census {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        // rust/crates/domains -> repo root
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
    }

    /// The two fields are a closed set — a third would be a new class of claim and must
    /// be a deliberate decision, not a typo that lands in the frozen contract.
    #[test]
    fn every_gate_declares_one_of_the_two_fields() {
        for gate in GATES {
            assert!(
                gate.field == SCIENCE_BANDS || gate.field == LIVENESS_FLOORS,
                "{gate:?}"
            );
        }
    }

    /// A manifest naming only a test id would freeze *that a test exists*, not *what it
    /// asserts* — so a bound could be loosened in place with the gate still green. The
    /// entry is the claim; the locus is where to go read it.
    #[test]
    fn every_gate_records_the_claim_not_just_a_test_id() {
        for gate in GATES {
            for (name, value) in [
                ("scenario", gate.scenario),
                ("field", gate.field),
                ("quantity", gate.quantity),
                ("bound", gate.bound),
                ("source", gate.source),
                ("locus", gate.locus),
            ] {
                assert!(!value.trim().is_empty(), "{name} empty on {gate:?}");
            }
        }
    }

    /// Two gates may share a scenario, never a locus — a duplicate would mean one
    /// declaration silently shadowed the other's claim.
    #[test]
    fn loci_are_unique() {
        let loci: BTreeSet<&str> = GATES.iter().map(|g| g.locus).collect();
        assert_eq!(loci.len(), GATES.len());
    }

    /// ⚠ The census's teeth, and the Python-side check run from the reference's own side.
    ///
    /// Deliberately crude — every numeric literal in `bound` must appear textually in the
    /// file the locus points at. It does not parse the expression, so it cannot prove the
    /// literal is *the* threshold; it does close the path where the number moves and the
    /// record does not, which is the retune-in-silence path `liveness_floors` exists to
    /// prevent and the family that has already been retuned twice.
    ///
    /// It also resolves the locus against the filesystem, which is what keeps
    /// [`GATE_SOURCE_FILE`] and the path literal inside the macro from drifting apart.
    #[test]
    fn the_bound_literals_appear_at_their_locus() {
        let mut checked = 0usize;
        for gate in GATES {
            let (file, test_name) = gate.locus.split_once("::").expect("locus is file::test");
            assert_eq!(file, GATE_SOURCE_FILE);
            let src = std::fs::read_to_string(repo_root().join(file))
                .unwrap_or_else(|e| panic!("locus {file} is not readable: {e}"));
            assert!(
                src.contains(test_name),
                "{test_name} is not present at {file}"
            );
            let literals = numeric_literals(gate.bound);
            assert!(
                !literals.is_empty(),
                "a bound with no number is not a bound: {gate:?}"
            );
            for literal in &literals {
                assert!(src.contains(literal.as_str()), "{} / {literal}", gate.locus);
            }
            checked += 1;
        }
        assert_eq!(checked, GATES.len());
    }

    /// The Python gate's regex, transcribed: `\d+\.\d+(?:[eE]-?\d+)?|\d+[eE]-\d+`.
    ///
    /// Hand-rolled because `simcore` and its dependents carry zero third-party crates and
    /// a regex engine is not worth breaking that for. The two alternatives are scanned in
    /// the same precedence order the regex uses.
    fn numeric_literals(bound: &str) -> Vec<String> {
        let chars: Vec<char> = bound.chars().collect();
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < chars.len() {
            if !chars[i].is_ascii_digit() {
                i += 1;
                continue;
            }
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let int_end = i;
            let mut end = None;
            if i < chars.len() && chars[i] == '.' {
                let mut j = i + 1;
                let frac_start = j;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if j > frac_start {
                    // `\d+\.\d+` matched; try the optional `(?:[eE]-?\d+)` suffix.
                    let mut k = j;
                    if k < chars.len() && (chars[k] == 'e' || chars[k] == 'E') {
                        let mut m = k + 1;
                        if m < chars.len() && chars[m] == '-' {
                            m += 1;
                        }
                        let digits = m;
                        while m < chars.len() && chars[m].is_ascii_digit() {
                            m += 1;
                        }
                        if m > digits {
                            k = m;
                        }
                    }
                    end = Some(k);
                }
            }
            if end.is_none() {
                // The second alternative: `\d+[eE]-\d+`.
                let mut j = int_end;
                if j + 1 < chars.len()
                    && (chars[j] == 'e' || chars[j] == 'E')
                    && chars[j + 1] == '-'
                {
                    let mut m = j + 2;
                    let digits = m;
                    while m < chars.len() && chars[m].is_ascii_digit() {
                        m += 1;
                    }
                    if m > digits {
                        j = m;
                        end = Some(j);
                    }
                }
            }
            if let Some(stop) = end {
                out.push(chars[start..stop].iter().collect());
                i = stop;
            }
            // No match starting here: `i` already sits past the integer run.
        }
        out
    }

    #[test]
    fn the_literal_scanner_matches_the_pythons_regex_on_every_shape_it_meets() {
        // Teeth on the scanner itself: it is a hand-rolled transcription of a regex, so
        // the shapes the frozen bounds actually use are pinned, INCLUDING the negatives.
        assert_eq!(numeric_literals("5.0 < peak < 8.0"), ["5.0", "8.0"]);
        assert_eq!(numeric_literals("non_collapsing(floor=5e-4)"), ["5e-4"]);
        assert_eq!(numeric_literals("non_collapsing(floor=0.05)"), ["0.05"]);
        assert_eq!(numeric_literals("peak_w < 14.4248"), ["14.4248"]);
        assert_eq!(numeric_literals("max(tail) > 0.55"), ["0.55"]);
        assert_eq!(numeric_literals("min > Γ*/ci_ratio (61.07 ppm)"), ["61.07"]);
        // ⚠ The one that decides the mutual-shading gate: bare integers do NOT match, so
        // "5%/day" contributes nothing and only 6.0 has to be present in the source.
        assert_eq!(
            numeric_literals("peak < 6.0 OR the 5%/day mutual-shading loss is MODELLED"),
            ["6.0"]
        );
        assert_eq!(numeric_literals("1.5e-3 and 2E-4"), ["1.5e-3", "2E-4"]);
        assert!(numeric_literals("no numbers here").is_empty());
        assert!(numeric_literals("bare 42 integers do not count").is_empty());
    }

    /// ⚠ The tripwire that lets the five CO₂ gates stay DERIVED.
    ///
    /// `Γ*` or `ci_ratio` moving is an unfreeze and should be loud. Every band gate
    /// compares a measured minimum against [`folds::floor_ppm`], which is computed rather
    /// than typed — so a silent re-value of `Γ*` would move all five bounds at once and
    /// no assertion would notice. This is the one place the number is pinned, and it is
    /// also what puts the literal `61.07` in this file for the locus check.
    #[test]
    fn the_floor_is_where_the_frozen_params_put_it() {
        assert!(
            (folds::floor_ppm() - 61.07).abs() < 5e-3,
            "{}",
            folds::floor_ppm()
        );
    }

    /// ⚠ The band does not depend on `Γ*`'s missing citation — measured, not assumed.
    ///
    /// `gamma_star` is `TODO(cite)`. The only route to the same quantity on the shelf is
    /// Teh eq. 6.19, `Γ* = O₂/(2·τ)`, with `τ` tabulated at 25 °C. It lands BELOW the
    /// shipped value, so the shipped floor is the harder test and closing the citation
    /// gap can only widen every margin.
    ///
    /// ⚠ A statement about the FLOOR, not an endorsement of swapping the value: Teh's
    /// companion constants disagree with ours, so the two are different parameterizations
    /// and mixing them would be the co-adaptation this project refuses. The comparison is
    /// legitimate *because* it only ever moves the bound in the harder direction.
    ///
    /// ⚠ Named without the `test_` prefix that five frozen `source` strings still spell —
    /// see the module note on why those strings were not edited.
    #[test]
    fn the_shipped_floor_is_the_conservative_one_against_the_cited_route() {
        let photo = crate::biosphere::params::photosynthesis();
        let teh_gamma = photo.o2 * 1000.0 / (2.0 * TEH_SPECIFICITY_FACTOR);
        let teh_floor = teh_gamma / crate::biosphere::system::sealed_chamber_scenario().ci_ratio;
        assert!((teh_floor - 57.69).abs() < 5e-3, "{teh_floor}");
        assert!(
            teh_floor < folds::floor_ppm(),
            "Teh's route no longer sits below the shipped floor — the robustness argument \
             is void and the band's provenance must be re-argued, not re-tuned"
        );
    }
}
