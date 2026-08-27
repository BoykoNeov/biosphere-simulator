//! The comparison report: baseline vs variants, tabulated.
//!
//! Every requirement below is in §6 of `docs/plans/post-roadmap-value-switch-harness.md`, and
//! each exists because reporting *without* it produced a wrong read on 2026-08-15:
//!
//! 1. **Distance-from-degenerate, never clearance-vs-bound alone.** "The floor's clearance
//!    falls 5.12 % → 0.40 %" reads as a plant nearly dying; against the measured stunted
//!    regime the same run moves 2.29× → 2.18×. The first number alone misled that session's
//!    own recommendation.
//! 2. **Label each gate's authority.** `science_bands` are bound from outside this repo,
//!    `liveness_floors` are tuned to our own calibration. Merging two claims of different
//!    strength under one name is this project's recorded failure mode.
//! 3. **Opposed movement is a first-class result.** At `k = 0.65` the chamber CO₂ bands
//!    loosen while the liveness floor tightens, *for one reason*. Reading either family alone
//!    gives the wrong answer.
//! 4. **Never store a ranking; re-derive it.** "The tightest of the five" inverted in six
//!    commits. Nothing here is cached between runs.
//! 5. **State what did NOT move.** A null result is the finding.
//!
//! ## ⚠⚠ What this report is NOT
//!
//! It prints measured **quantities** beside each claim's bound **as recorded** — not pass/fail
//! verdicts. [`ScienceGate::bound`] is a human-readable string (`"5.0 < peak < 8.0"`,
//! `"non_collapsing(floor=5e-4)"`), not an evaluator; parsing it would put a second copy of the
//! census in the harness, which is the *a rule with two copies has one that is stale* failure
//! this tree names more than any other. The header of every rendered report says so, because a
//! reader who mistakes a quantity table for a verdict table is worse off than one who never saw
//! it.
//!
//! ## ⚠ And it takes no decision
//!
//! The `extinction_coef` question this was built for is open and the user's. The report
//! regenerates the evidence; it does not choose.

use super::Substitution;
use crate::biosphere::drift::year_summaries;
use crate::biosphere::params::BiosphereParams;
use crate::biosphere::readouts::{
    floor_ppm, min_ppm, peak_lai, peak_w, segment_max, trajectory, Trajectory,
};
use crate::biosphere::science_gates::{ScienceGate, GATES};
use crate::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario, SeasonScenario,
    CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};

/// Years dropped before the fixed point is read — the perennial gate's own transient.
const FIXED_POINT_TRANSIENT: usize = 8;

/// One quantity the report measures, and which recorded claim it informs.
pub struct ReadoutSpec {
    /// The frozen scenario, spelled as [`ScienceGate::scenario`] spells it.
    pub scenario: &'static str,
    /// What is measured, in the report's own words.
    pub quantity: &'static str,
    /// The `quantity` strings of the gates this feeds, **under the same scenario**.
    ///
    /// ⚠ This is the link a reader needs and the one thing here that can rot silently, so
    /// [`tests::every_informed_gate_resolves`] resolves each string against [`GATES`].
    pub informs: &'static [&'static str],
    /// The measured **degenerate** value this quantity is read against, where one is on
    /// record — requirement 1.
    pub degenerate: Option<f64>,
    /// Needs the 15-year horizon: minutes rather than seconds.
    pub long: bool,
    fold: fn(&Trajectory) -> f64,
}

fn fixed_point(t: &Trajectory) -> f64 {
    let summaries = year_summaries(&t.leaf_c, t.year(), segment_max);
    assert_eq!(summaries.len(), t.years, "annual summary count");
    segment_max(&summaries[FIXED_POINT_TRANSIENT..])
}

/// Everything the report measures.
///
/// ⚠ Each fold is driven **only against its own scenario**. `min_ppm` on the open field would
/// fold an empty series — an unsealed run has no chamber pool at all — and the fold panics
/// rather than returning `+inf`, which would read as "comfortably above the compensation
/// point". The pairing is data here so it cannot be got wrong by a loop.
pub const SPECS: &[ReadoutSpec] = &[
    ReadoutSpec {
        scenario: "open_season",
        quantity: "peak LAI (m2 m-2)",
        informs: &[
            "peak LAI (m2 m-2)",
            "peak LAI (m2 m-2) vs the mutual-shading threshold",
        ],
        degenerate: None,
        long: false,
        fold: peak_lai,
    },
    ReadoutSpec {
        scenario: "open_season",
        quantity: "peak W excl. fibrous roots (t/ha)",
        informs: &["peak W excl. fibrous roots (t/ha)"],
        degenerate: None,
        long: false,
        fold: peak_w,
    },
    ReadoutSpec {
        scenario: "sealed_chamber",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: false,
        fold: min_ppm,
    },
    ReadoutSpec {
        scenario: "perennial_chamber",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: false,
        fold: min_ppm,
    },
    ReadoutSpec {
        scenario: "consumer_chamber",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: false,
        fold: min_ppm,
    },
    ReadoutSpec {
        scenario: "perennial_long_horizon",
        quantity: "converged peak-leaf fixed point (mol C)",
        informs: &["converged peak-leaf fixed point (mol C)"],
        // ⚠ The ONE degenerate baseline this tree has on record: the stunted regime, named in
        // that gate's own frozen `source` ("2.2x the 0.253 dead baseline"). It is carried
        // there in PROSE, not as an assertion — cited here as exactly that, and no baseline is
        // invented for the rows that have none.
        degenerate: Some(0.253),
        long: true,
        fold: fixed_point,
    },
    ReadoutSpec {
        scenario: "perennial_long_horizon",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: true,
        fold: min_ppm,
    },
];

/// The runs the specs read, `(name, scenario, years, perennial)`.
///
/// ⚠ Each scenario is driven the way its own golden drives it — `sealed_chamber` through
/// `run_season` with no re-sow, the chambers through `run_perennial`'s annual reset.
fn runs() -> Vec<(&'static str, SeasonScenario, usize, bool)> {
    vec![
        ("open_season", DEFAULT_SCENARIO, 1, false),
        (
            "sealed_chamber",
            sealed_chamber_scenario(),
            SEALED_CHAMBER_YEARS,
            false,
        ),
        (
            "perennial_chamber",
            perennial_chamber_scenario(),
            PERENNIAL_CHAMBER_YEARS,
            true,
        ),
        (
            "consumer_chamber",
            consumer_chamber_scenario(),
            CONSUMER_CHAMBER_YEARS,
            true,
        ),
        (
            "perennial_long_horizon",
            perennial_chamber_scenario(),
            LONG_HORIZON_YEARS,
            true,
        ),
    ]
}

/// One measured column of the table.
pub struct Column {
    /// What was substituted, in the reader's words (`"frozen"` for the baseline).
    pub label: String,
    /// `(spec index, value)` for every spec measured.
    pub values: Vec<(usize, f64)>,
    /// The compensation-point floor at these params.
    ///
    /// ⚠ A column, not a constant: it is `Γ*/ci_ratio`, so a substitution touching
    /// `photosynthesis.yaml` moves the *floor* as well as the readings taken against it.
    pub floor_ppm: f64,
    /// Arbitration firings summed across the runs. ⚠ A band is a claim about a **well-fed**
    /// run; a rationed column's numbers are not the model's answer, and the report says so
    /// rather than printing them as if they were.
    pub rationed: u64,
    /// Extinction events summed across the runs.
    pub events: usize,
}

/// Measure every applicable spec against `p`.
pub fn measure(label: &str, p: &BiosphereParams, long: bool) -> Column {
    let mut values = Vec::new();
    let mut rationed = 0;
    let mut events = 0;
    for (name, scenario, years, perennial) in runs() {
        let needed: Vec<usize> = SPECS
            .iter()
            .enumerate()
            .filter(|(_, s)| s.scenario == name && (long || !s.long))
            .map(|(i, _)| i)
            .collect();
        if needed.is_empty() {
            continue;
        }
        let t = trajectory(scenario, years, perennial, p);
        rationed += t.rationed;
        events += t.events;
        for i in needed {
            values.push((i, (SPECS[i].fold)(&t)));
        }
    }
    Column {
        label: label.to_string(),
        values,
        floor_ppm: floor_ppm(p),
        rationed,
        events,
    }
}

/// The baseline column plus one per variant, in the order given.
pub fn compare(
    variants: &[(String, Vec<Substitution>)],
    long: bool,
) -> Result<Vec<Column>, config::ConfigError> {
    let mut columns = vec![measure("frozen", &super::biosphere_with(&[])?, long)];
    for (label, subs) in variants {
        columns.push(measure(label, &super::biosphere_with(subs)?, long));
    }
    Ok(columns)
}

/// The gates a spec informs, resolved against [`GATES`] — requirement 2's authority label.
pub fn gates_for(spec: &ReadoutSpec) -> Vec<&'static ScienceGate> {
    GATES
        .iter()
        .filter(|g| g.scenario == spec.scenario && spec.informs.contains(&g.quantity))
        .collect()
}

fn value_of(col: &Column, spec: usize) -> Option<f64> {
    col.values.iter().find(|(i, _)| *i == spec).map(|(_, v)| *v)
}

/// Render the comparison as text. See the module header for what these numbers are and are not.
pub fn render(columns: &[Column], long: bool) -> String {
    let mut out = String::new();
    out.push_str(
        "value-switch report — MEASURED QUANTITIES, not pass/fail verdicts.\n\
         Each row gives what the model produces and the bound as the contract RECORDS it;\n\
         comparing the two is the reader's job, because a bound is prose, not an evaluator.\n\n",
    );
    let base = &columns[0];
    for (i, spec) in SPECS.iter().enumerate() {
        if spec.long && !long {
            continue;
        }
        let gates = gates_for(spec);
        let mut authority: Vec<&str> = gates.iter().map(|g| g.field).collect();
        authority.sort_unstable();
        authority.dedup();
        out.push_str(&format!("{} / {}\n", spec.scenario, spec.quantity));
        out.push_str(&format!(
            "  informs: {}\n",
            if gates.is_empty() {
                "(no recorded gate — diagnostic only)".to_string()
            } else {
                authority.join(" + ")
            }
        ));
        for g in &gates {
            out.push_str(&format!("    bound as recorded: {}\n", g.bound));
        }
        let base_v = value_of(base, i);
        for col in columns {
            let (b, v) = match (base_v, value_of(col, i)) {
                (Some(b), Some(v)) => (b, v),
                _ => continue,
            };
            let mut line = format!("    {:<38} {v:>14.6}", col.label);
            if col.label != base.label {
                let delta = v - b;
                let rel = if b == 0.0 {
                    f64::NAN
                } else {
                    delta / b * 100.0
                };
                line.push_str(&format!("  {delta:+.6} ({rel:+.3} %)"));
                if delta == 0.0 {
                    line.push_str("  <- UNCHANGED");
                }
            }
            if let Some(d) = spec.degenerate {
                line.push_str(&format!("  [{:.3}x the {d} degenerate baseline]", v / d));
            }
            line.push('\n');
            out.push_str(&line);
        }
        out.push('\n');
    }

    out.push_str("chamber CO2 compensation point (ppm) — the floor the CO2 rows are read against\n");
    for col in columns {
        out.push_str(&format!("    {:<38} {:>14.6}\n", col.label, col.floor_ppm));
    }
    out.push('\n');

    // ⚠⚠ Requirement 3 has a precondition this report FAILED on its first run, and the check
    // is that finding turned into a line of output. Opposed movement can only be read from a
    // table that carries both claim families — and in the short report every row informs
    // `science_bands`, because the one `liveness_floors` quantity on the roster is a 15-year
    // one. So the short table showed "5 rose, 0 fell" and looked like a clean, one-directional
    // improvement. That is exactly the wrong read requirement 3 exists to prevent, arrived at
    // by omission rather than by misreading.
    let present: Vec<&str> = {
        let mut a: Vec<&str> = SPECS
            .iter()
            .enumerate()
            .filter(|(_, s)| long || !s.long)
            .flat_map(|(_, s)| gates_for(s))
            .map(|g| g.field)
            .collect();
        a.sort_unstable();
        a.dedup();
        a
    };
    for missing in ["science_bands", "liveness_floors"] {
        if !present.contains(&missing) {
            out.push_str(&format!(
                "⚠ NO {missing} ROW IS IN THIS TABLE — every row above informs {}. Opposed \
                 movement CANNOT be read from it, and a column that moves one way throughout \
                 is not evidence that nothing moves the other way.\n",
                present.join(" + ")
            ));
        }
    }

    // Requirement 3 — opposed movement, stated rather than left to be spotted.
    for col in columns.iter().skip(1) {
        let (mut up, mut down, mut flat) = (0, 0, 0);
        for (i, spec) in SPECS.iter().enumerate() {
            if spec.long && !long {
                continue;
            }
            match (value_of(base, i), value_of(col, i)) {
                (Some(b), Some(v)) if v > b => up += 1,
                (Some(b), Some(v)) if v < b => down += 1,
                (Some(_), Some(_)) => flat += 1,
                _ => {}
            }
        }
        out.push_str(&format!(
            "{}: {up} rose, {down} fell, {flat} did not move{}\n",
            col.label,
            if up > 0 && down > 0 {
                "  <- OPPOSED: reading either family alone gives the wrong answer"
            } else {
                ""
            }
        ));
    }

    // The run-level preconditions a band depends on, and requirement 5's null results.
    out.push('\n');
    for col in columns {
        if col.rationed > 0 || col.events > 0 {
            out.push_str(&format!(
                "⚠ {}: {} arbitration firings, {} extinction events — a band is a claim about a \
                 WELL-FED run, so this column's numbers are not the model's answer\n",
                col.label, col.rationed, col.events
            ));
        }
    }
    if !long {
        out.push_str(
            "\n⚠ NOT MEASURED: every 15-year row, including the only quantity with a degenerate \
             baseline on record. Re-run with --long.\n",
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠ The link between a measured quantity and the claim it informs is the one thing here
    /// that could rot silently — a gate's `quantity` string is frozen manifest content and can
    /// be re-worded by an unfreeze, leaving this roster pointing at nothing while the report
    /// still prints a row with no authority label. Resolved against [`GATES`], not trusted.
    #[test]
    fn every_informed_gate_resolves() {
        for spec in SPECS {
            for want in spec.informs {
                assert!(
                    GATES
                        .iter()
                        .any(|g| g.scenario == spec.scenario && g.quantity == *want),
                    "{}/{want:?} names no gate — the roster is stale",
                    spec.scenario
                );
            }
            assert!(
                !gates_for(spec).is_empty(),
                "{}/{} informs nothing",
                spec.scenario,
                spec.quantity
            );
        }
    }

    /// ⚠⚠ **A spec whose scenario has no `runs()` entry is silently unmeasured**, and that is
    /// this design's version of *a census ported as a LIST is the failure it prevents*.
    /// [`measure`] iterates the runs and filters the specs by name, so a spec naming a scenario
    /// that is not in `runs()` matches nothing and simply produces no row — no error, no gap in
    /// the table, just a claim quietly not measured. The short report's count assertion catches
    /// it for a short spec and **nothing catches it for a `long: true` one**, which is not
    /// hypothetical: `consumer_long_horizon` is already a scenario in [`GATES`] with no `runs()`
    /// entry, so the next spec added under it is the one that would vanish.
    #[test]
    fn every_spec_names_a_scenario_that_is_actually_run() {
        let names: Vec<&str> = runs().iter().map(|(n, _, _, _)| *n).collect();
        for spec in SPECS {
            assert!(
                names.contains(&spec.scenario),
                "{}/{} names no run — it would be silently unmeasured, not reported missing",
                spec.scenario,
                spec.quantity
            );
        }
    }

    /// Anti-vacuity: the roster must actually cover both authorities, or requirement 2's
    /// labelling is decoration.
    #[test]
    fn the_roster_covers_both_authorities() {
        let fields: Vec<&str> = SPECS
            .iter()
            .flat_map(gates_for)
            .map(|g| g.field)
            .collect();
        assert!(fields.contains(&"science_bands"), "{fields:?}");
        assert!(fields.contains(&"liveness_floors"), "{fields:?}");
    }

    /// The short report must not silently omit a scenario it claims to cover, and it must not
    /// drive a chamber fold against an unsealed run.
    #[test]
    fn the_short_report_measures_every_short_spec() {
        let col = measure("frozen", &crate::biosphere::params::biosphere(), false);
        let short: Vec<usize> = SPECS
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.long)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(col.values.len(), short.len());
        for i in short {
            assert!(
                value_of(&col, i).expect("measured").is_finite(),
                "{} / {} is not finite",
                SPECS[i].scenario,
                SPECS[i].quantity
            );
        }
        assert_eq!(col.rationed, 0, "the frozen baseline must be well-fed");
        assert_eq!(col.events, 0, "the frozen baseline must not go extinct");
    }

    fn at_k(k: f64, long: bool) -> String {
        let columns = compare(
            &[(
                format!("k={k}"),
                vec![Substitution::new("canopy.yaml", "extinction_coef", k)],
            )],
            long,
        )
        .expect("compare");
        render(&columns, long)
    }

    /// ⚠⚠ **The short table cannot show opposition, and it must SAY so.** This is the report's
    /// own first finding: every short row informs `science_bands`, because the roster's one
    /// `liveness_floors` quantity is a 15-year one. Without the warning the short table reads
    /// "5 rose, 0 fell" — a clean one-directional improvement — which is requirement 3's wrong
    /// answer reached by omission rather than by misreading.
    #[test]
    fn the_short_table_declares_the_family_it_cannot_show() {
        let text = at_k(0.65, false);
        assert!(text.contains("MEASURED QUANTITIES, not pass/fail"), "{text}");
        assert!(
            text.contains("NO liveness_floors ROW IS IN THIS TABLE"),
            "{text}"
        );
    }

    /// ⚠ Requirement 3 itself, and it is why the report exists as one table: on the canopy
    /// coefficient the two claim families move in OPPOSITE directions, for one reason. Reading
    /// either alone gives the wrong answer — which is what happened on 2026-08-15.
    ///
    /// Costs the two 15-year runs (~20 s) and is not `#[ignore]`d for that: an ignored test is
    /// a test that never runs, and this is the one assertion that the harness's whole reason
    /// for existing still holds.
    #[test]
    fn the_long_table_surfaces_opposed_movement_on_the_canopy_coefficient() {
        let text = at_k(0.65, true);
        assert!(!text.contains("NO liveness_floors ROW"), "{text}");
        assert!(text.contains("degenerate baseline"), "{text}");
        assert!(
            text.contains("OPPOSED"),
            "the canopy coefficient no longer splits the two families:\n{text}"
        );
    }
}
