//! The station's **science gates** — the census behind
//! `docs/station-reference.manifest.json`'s `science_bands` / `liveness_floors` fields
//! (slice C4b of the reference flip).
//!
//! ## Why there are two census tables and not one
//!
//! Slice C4 moved thirteen gates into [`domains::biosphere::science_gates`] and left two
//! behind, filed under *station* scenarios: `crew_mission`'s respiratory-quotient
//! prediction and `sealed_station`'s thermal-node floor. They could not travel with the
//! other thirteen because a gate lives with the runs it reads, and these read the coupled
//! cabin and the Power→Thermal station — `station` types, in a crate that depends on
//! `domains` rather than the reverse.
//!
//! So the table is *here*, and the mechanism is shared: [`domains::science_gates!`] is
//! exported and this module invokes it, declaring its own `source_file` for the `locus`
//! half. One census mechanism, one transcribed regex
//! ([`domains::biosphere::science_gates::numeric_literals`]), two tables — split exactly
//! the way the two frozen contracts are, which is also how the two writers are split.
//!
//! ⚠ **The split is asserted, not documented.** A gate here naming a *biosphere*
//! scenario panics during regeneration (the station writer's roster check), and one there
//! naming a station scenario panics in the biosphere writer's. Neither filters the claim
//! away, because a filter that silently drops a gate looks exactly like a clean result.
//!
//! ## What C4b did NOT do
//!
//! It re-anchored the two **loci** and nothing else. The `quantity` / `bound` / `source`
//! strings are byte-identical to the ones the pytest markers carried, and both runs are
//! the ones the Python gates already drove. The Python test bodies **stay** — they are
//! the checker's own conformance half, and retiring them is Stage 3's call, not a free
//! consequence of moving a claim.
//!
//! ⚠ The estimate this slice was scheduled under said `predicted_equilibrium_temperature`
//! was Python-only. It was not: [`crate::system::predicted_equilibrium_temperature`], the
//! drift folds and the 15-yr energy run all already existed. Recorded because a
//! present-tense claim about the tree went false with nothing watching it.

// ⚠ `cfg(test)`, like the folds below: the two field names are read only by the census
// checks, and an unconditional `use` would be an unused import in every ordinary build.
#[cfg(test)]
use domains::biosphere::science_gates::{LIVENESS_FLOORS, SCIENCE_BANDS};

// ⚠ `cfg(test)`, not a plain import: the folds and runs are reached only from the gate
// bodies, and an unconditional `use` here would be an unused import in every ordinary
// build — which `cargo clippy --all-targets -D warnings` turns into a failure.
#[cfg(test)]
use domains::biosphere::drift::{is_stationary, non_collapsing, same_phase_diffs, year_summaries};

/// This file, repo-relative — the path half of every gate's `locus`.
///
/// ⚠ Duplicated as a literal at the [`domains::science_gates!`] invocation below, because
/// `concat!` takes only literals. The duplication cannot rot silently:
/// [`the_bound_literals_appear_at_their_locus`] resolves every locus against the
/// filesystem, so a moved file turns it red.
pub const GATE_SOURCE_FILE: &str = "rust/crates/station/src/science_gates.rs";

// ---------------------------------------------------------------------------------
// Gate 1's run: the BVAD-calibrated cabin.
// ---------------------------------------------------------------------------------

/// NASA BVAD Table 3-31 (Rev 2, Feb 2022, p. 58) — the nominal per-crewmember-per-day
/// metabolic interface values, and the arithmetic that turns them into this model's mol
/// accounting.
///
/// Verbatim from `docs/bvad-reference.md`, which is the primary source; we cite the
/// document and copy no dataset (`docs/reuse-and-licenses.md`).
///
/// ⚠ **Calibration ≠ validation, and this module ports only the validation half.** The
/// Python file this came from keeps three columns visibly separate: calibration
/// checkpoints (CO₂, feces, humidity, urine — we *set* the intake and the fractions to
/// reproduce these, so a match is bookkeeping), the structural prediction (the one
/// assertion that can genuinely fail), and closure. Only the structural prediction is a
/// frozen `science_bands` claim, so only it is a gate; the other seven assertions stay in
/// `tests/test_bvad_validation.py` as the checker's conformance half.
#[cfg(test)]
mod bvad {
    use crate::scenario::CabinScenario;

    /// −m Carbon Dioxide Load (kg/CM-d).
    pub const CO2_LOAD_KG: f64 = 1.085;
    /// +m Oxygen Consumed (kg/CM-d).
    pub const O2_CONSUMED_KG: f64 = 0.895;
    /// −m Respiration and Perspiration Water (kg/CM-d).
    pub const RESP_PERSP_WATER_KG: f64 = 2.946;
    /// −m Urine Water (kg/CM-d).
    pub const URINE_WATER_KG: f64 = 1.420;

    /// Molar masses (g/mol) for the CARBON/OXYGEN mol accounting our stocks use.
    pub const M_CO2: f64 = 44.009;
    pub const M_O2: f64 = 31.998;

    pub const SECONDS_PER_DAY: f64 = 86_400.0;

    /// BVAD's CO₂ load in mol — and, because `CrewRespiration` fixes RQ = 1, also what
    /// the model's O₂ consumption must equal.
    pub const CO2_MOL: f64 = CO2_LOAD_KG * 1000.0 / M_CO2;
    /// BVAD's O₂ consumption in mol.
    pub const O2_MOL: f64 = O2_CONSUMED_KG * 1000.0 / M_O2;
    /// The daily-effective molar RQ, blending BVAD's nominal + exercise periods.
    pub const RQ_EFFECTIVE: f64 = CO2_MOL / O2_MOL;
    /// The two modeled water fates (humidity + urine); metabolic + fecal water are not
    /// modeled.
    pub const MODELED_WATER_KG: f64 = RESP_PERSP_WATER_KG + URINE_WATER_KG;

    /// One crew configuration: a 4-crewmember complement. Everything is linear in the
    /// crew count, so the per-CM comparison is N-invariant; 4 keeps every cabin stock
    /// comfortably positive.
    pub const N_CREW: f64 = 4.0;

    /// The BVAD-calibrated cabin: crew load = `N_CREW` × per-CM BVAD, on the shipped
    /// ECLSS sizing.
    ///
    /// ⚠ Built here rather than in [`crate::scenario`] — deliberately, and the same
    /// choice the Python file made. It is a *validation* scenario, not a pinned run: it
    /// touches no golden and appears in no manifest roster, and putting it beside
    /// `CABIN_GAS_SCENARIO` would invite exactly that confusion.
    ///
    /// **Carbon is calibrated to BVAD's directly-measured CO₂ load, not to the derived
    /// food-carbon total** — the latter also depends on the 44–55 % feces-carbon
    /// assumption. That keeps the O₂ headline a clean number: model O₂ = model CO₂ =
    /// BVAD CO₂ exactly, at RQ = 1.
    pub fn cabin_scenario(respired_carbon_fraction: f64) -> CabinScenario {
        CabinScenario {
            food_store0: 2000.0,
            water_store0: 60.0,
            food_intake_rate: N_CREW * CO2_MOL / respired_carbon_fraction / SECONDS_PER_DAY,
            water_intake_rate: N_CREW * MODELED_WATER_KG / SECONDS_PER_DAY,
            ..crate::scenario::CABIN_GAS_SCENARIO
        }
    }
}

#[cfg(test)]
mod runs {
    use super::bvad;
    use domains::eclss::O2_SUPPLY;
    use domains::params;
    use domains::thermal::NODE;
    use simcore::integrator::EulerIntegrator;
    use simcore::state::State;
    use std::sync::OnceLock;

    /// The steady-state O₂-supply flux of the BVAD cabin, per crewmember per day (mol).
    ///
    /// ⚠ Read as a **one-step boundary delta between the last two states**, exactly as
    /// the Python gate reads it: at steady state the fluxes are constant, so one step's
    /// change in the boundary reservoir *is* the rate. `abs` because `boundary.o2_supply`
    /// is a source and drains as it supplies.
    ///
    /// ⚠ Pre-reduced in the observer rather than by materializing 901 `State`s — the
    /// station's own precedent (`examples/emit_sealed_energy_drift.rs`). The reduction is
    /// a `push` of one scalar, so there is no fold arithmetic to get wrong; what would be
    /// wrong is reading fewer than two samples, and `run_station` observes the initial
    /// state plus every produced one, which is asserted below.
    pub fn bvad_o2_per_cm_per_day() -> f64 {
        static CELL: OnceLock<f64> = OnceLock::new();
        *CELL.get_or_init(|| {
            let crew = params::crew();
            let eclss = params::eclss();
            let scenario = bvad::cabin_scenario(crew.respired_carbon_fraction);
            let (state, registry) =
                crate::cabin::build_cabin(&crew, &eclss, &scenario).expect("build_cabin");
            let resolver = crate::cabin::cabin_resolver(&scenario).expect("cabin_resolver");
            let integrator = EulerIntegrator::new(registry);
            let mut o2_supply: Vec<f64> = Vec::new();
            let (_final, rationed, events) = crate::run_station(
                &integrator,
                state,
                &resolver,
                scenario.dt_seconds,
                crate::scenario::CABIN_GAS_STEPS,
                &mut |s: &State| o2_supply.push(s.stocks[O2_SUPPLY].amount),
            )
            .expect("BVAD cabin run");

            // A band is a claim about a **well-fed** run; a rationed or extinction-hit
            // trace is not the model's answer. Both are preconditions of the claim, not
            // hygiene.
            assert_eq!(rationed, 0, "the BVAD cabin run must be well-fed");
            assert!(events.is_empty(), "the BVAD cabin run must be event-free");
            assert_eq!(
                o2_supply.len(),
                crate::scenario::CABIN_GAS_STEPS as usize + 1,
                "observer sample count"
            );

            let n = o2_supply.len();
            let flux_per_s = (o2_supply[n - 1] - o2_supply[n - 2]).abs() / scenario.dt_seconds;
            flux_per_s * bvad::SECONDS_PER_DAY / bvad::N_CREW
        })
    }

    /// The Tier-1 energy decade's per-year **peak node temperature** (K).
    ///
    /// The same run and the same fold `examples/emit_sealed_energy_drift.rs` emits as a
    /// golden: the 15-yr single-rate Power → Thermal `HEAT_CLOSURE_SCENARIO` at the
    /// diurnal `dt = 3600 s`, reduced per step to `T_space + node/C` and folded to an
    /// annual max.
    ///
    /// ⚠ `steps_per_year` is in the trajectory's **own index unit** — power steps, not
    /// days. `run_station` appends one state per power step, and passing days where steps
    /// are meant is the trap `docs/plans/post-roadmap-step-unfreeze.md` §1 records.
    pub fn node_peak_temps() -> &'static Vec<f64> {
        static CELL: OnceLock<Vec<f64>> = OnceLock::new();
        CELL.get_or_init(|| {
            let charge = params::charge();
            let thermal = params::thermal();
            let scenario = crate::scenario::HEAT_CLOSURE_SCENARIO;
            let (state, registry) =
                crate::system::build_station(&charge, &thermal, &scenario, None)
                    .expect("build_station");
            let resolver =
                crate::system::station_resolver(&charge, &scenario).expect("station_resolver");
            let integrator = EulerIntegrator::new(registry);
            let mut node_temp: Vec<f64> = Vec::new();
            let (_final, rationed, events) = crate::run_station(
                &integrator,
                state,
                &resolver,
                scenario.power.dt_seconds,
                crate::scenario::SEALED_ENERGY_DAYS * scenario.power.steps_per_day,
                &mut |s: &State| {
                    node_temp.push(
                        thermal.space_temperature + s.stocks[NODE].amount / thermal.heat_capacity,
                    )
                },
            )
            .expect("Tier-1 energy decade");
            assert_eq!(rationed, 0, "Tier-1 energy decade must be well-fed");
            assert!(events.is_empty(), "Tier-1 energy decade must be event-free");

            let steps_per_year =
                scenario.power.steps_per_day as usize * crate::scenario::SEALED_STATION_SEASON_DAYS;
            super::year_summaries(&node_temp, steps_per_year, |segment: &[f64]| {
                segment.iter().fold(f64::NEG_INFINITY, |acc, &t| acc.max(t))
            })
        })
    }
}

// ---------------------------------------------------------------------------------
// The 2 gates.
// ---------------------------------------------------------------------------------

domains::science_gates! {
    source_file: "rust/crates/station/src/science_gates.rs";

    /// THE headline BVAD result, and the one genuinely un-tuned output of the crew model.
    ///
    /// `CrewRespiration` fixes RQ = 1 (one mol O₂ consumed per mol CO₂ produced,
    /// independent of the fraction values), so calibrating CO₂ to BVAD forces the model's
    /// O₂ consumption to BVAD's *CO₂* molar value rather than its O₂ one. The ratio to
    /// BVAD's O₂ is the daily-effective molar RQ, ~11.8 % low — a documented consequence
    /// of the fixed RQ, not a parameter error.
    ///
    /// ⚠ Pinned as a **number**, not a bound (the `lab.fit_order` "measure the known
    /// structural error" discipline). A regression that silently changed the respiration
    /// stoichiometry moves it.
    ///
    /// ⚠ **Four assertions, ONE census row.** The recorded `bound` is a single claim, so
    /// splitting it into four gates would file four claims where the contract has one —
    /// the opposite of C4's parametrized biosphere marker, which carried *two* claims
    /// through one decorator and correctly became two rows.
    gate rq_structural_prediction {
        scenario: "crew_mission",
        field: "science_bands",
        quantity: "daily-effective molar respiratory quotient",
        bound: "CO2/O2 == approx(0.8814)",
        source: "NASA BVAD Table 3-31 (Rev 2, 2022, p. 58)",
        check: {
            let model_o2_mol = runs::bvad_o2_per_cm_per_day();

            // The model consumes exactly its CO₂ production in O₂ (RQ = 1) — so O₂ equals
            // the BVAD CO₂ molar value, not the BVAD O₂ value.
            assert!(
                (model_o2_mol - bvad::CO2_MOL).abs() <= 1e-6 * bvad::CO2_MOL.abs(),
                "model O2 {model_o2_mol} vs BVAD CO2 {}",
                bvad::CO2_MOL
            );

            // The headline: O₂ consumption is the daily-effective RQ fraction of BVAD's O₂.
            let ratio = model_o2_mol / bvad::O2_MOL;
            assert!(
                (ratio - bvad::RQ_EFFECTIVE).abs() <= 1e-6 * bvad::RQ_EFFECTIVE.abs(),
                "{ratio}"
            );
            // ⚠ The recorded number, spelled as the literal the contract carries. The
            // derived comparison above is the real check; this is the tripwire that puts
            // the recorded value in this file for the locus check, exactly as the CO2
            // compensation point's is on the biosphere side.
            //
            // ⚠ The number is deliberately NOT repeated in this comment. The locus check
            // counts occurrences and subtracts what the declared bounds contribute, so a
            // comment quoting the value would supply the surplus by itself and the check
            // would pass with this line deleted — measured, and it is why the control ran
            // twice before it bit.
            assert!((ratio - 0.8814).abs() < 1e-4, "{ratio}");

            // i.e. the model under-predicts O₂ consumption by ~11.8 % — a documented
            // consequence of the fixed RQ = 1.
            let model_o2_kg = model_o2_mol * bvad::M_O2 / 1000.0;
            let shortfall = (bvad::O2_CONSUMED_KG - model_o2_kg) / bvad::O2_CONSUMED_KG;
            assert!((shortfall - 0.118).abs() < 2e-3, "{shortfall}");
        }
    }

    /// The Tier-1 energy decade's node temperature is a **period-1 fixed point** — a real
    /// emergent equilibrium, not a construction.
    ///
    /// Power carries no seasonal forcing, so consecutive years are the same phase and
    /// every year's peak is identical. The recorded claim is the *floor*: the node must
    /// not collapse toward `T_space`. Stationarity is its mandatory companion and the
    /// proximity to the dissipation-set `T_eq` is what makes it an attractor rather than
    /// a survival, but the frozen bound is the one number tuned to our own calibration.
    ///
    /// ⚠ `peaks.len() >= 3` is not tidiness. The fold computes `n_years = (len - 1) /
    /// year`, so an observer emitting `steps` samples instead of `steps + 1` yields one
    /// summary fewer and every check still passes — `non_collapsing` over 14 years passes
    /// exactly as well as over 15. The pre-reduction is what creates that hole; the count
    /// assertion is what closes it.
    gate tier1_node_is_period_1_fixed_point {
        scenario: "sealed_station",
        field: "liveness_floors",
        quantity: "annual peak thermal-node temperature (K)",
        bound: "non_collapsing(floor=100.0)",
        source: "self — the node must not collapse toward T_space",
        check: {
            let peaks = runs::node_peak_temps();
            assert!(peaks.len() >= 3, "{} annual peaks", peaks.len());
            let diffs = same_phase_diffs(peaks, 1);
            assert!(
                is_stationary(&diffs, 0.1, 1e-3, 0),
                "node peak-T must be stationary over the decade, diffs={:?}",
                &diffs[..3.min(diffs.len())]
            );
            assert!(
                non_collapsing(peaks, 100.0),
                "node must not collapse toward T_space"
            );

            let t_eq = crate::system::predicted_equilibrium_temperature(
                &domains::params::charge(),
                &domains::params::thermal(),
                &crate::scenario::HEAT_CLOSURE_SCENARIO,
            );
            let last = *peaks.last().expect("a non-empty peak series");
            assert!(
                (last - t_eq).abs() < 1.0,
                "node peak-T {last:.3} must sit near the dissipation-set T_eq {t_eq:.3}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------
// The census's own checks.
// ---------------------------------------------------------------------------------

#[cfg(test)]
mod census {
    use super::*;
    use domains::biosphere::science_gates::check_bound_literals;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        // rust/crates/station -> repo root
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

    /// ⚠ The census's teeth, run from the reference's own side.
    ///
    /// The rule is [`check_bound_literals`], shared with the biosphere's half so the
    /// transcribed regex has exactly one copy in the tree.
    #[test]
    fn the_bound_literals_appear_at_their_locus() {
        check_bound_literals(GATES, GATE_SOURCE_FILE, &repo_root());
    }

    /// ⚠ The station half is exactly TWO claims, and the count is asserted rather than
    /// documented.
    ///
    /// It is the mirror of the biosphere table's own scope note, and of the Python
    /// checker's `test_the_python_science_census_is_exhausted`: after C4b the pytest
    /// marker census is empty and *these* two are what the reference carries in its
    /// place. A third gate declared here is a widening of the frozen contract and must
    /// arrive with the ceremony, not with a green run.
    #[test]
    fn the_station_census_is_the_two_c4b_moved() {
        let filed: BTreeSet<(&str, &str)> = GATES.iter().map(|g| (g.scenario, g.field)).collect();
        assert_eq!(
            filed,
            BTreeSet::from([
                ("crew_mission", SCIENCE_BANDS),
                ("sealed_station", LIVENESS_FLOORS),
            ])
        );
        assert_eq!(GATES.len(), 2);
    }
}
