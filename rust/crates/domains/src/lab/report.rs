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
//! 6. **A cell that was not measured says so.** Added for the mechanism variants of slice 4,
//!    because until then it could not arise: a parameter substitution applies to every
//!    scenario, so every column had every row. A mechanism swap need not — ten of the
//!    twenty-three biosphere flows are in all four canonical builds, and the other thirteen
//!    are where the soil-carbon and nitrogen science lives. The renderer used to `continue`
//!    past a missing value, dropping the cell with no marker and shrinking the rose/fell
//!    count silently, which is `every_spec_names_a_scenario_that_is_actually_run`'s failure
//!    arriving through a new door.
//! 7. **A readout whose series never moved says so.** ⚠ Slice 4 of the science-switch plan
//!    named the wrong hazard here, and the correction is worth more than the guard. The plan
//!    expected an *empty* series folding to `+infinity` — "comfortably above the compensation
//!    point". That cannot happen: a composition rewrites the flow list and stock presence
//!    comes from `build_season_with`, so every series is exactly as long as the frozen run's,
//!    and `min_ppm` asserts non-emptiness anyway. What a swap **can** do is remove a stock's
//!    only writer, leaving the series **constant** — and then `min_ppm` returns the initial
//!    charge, which is finite, plausible and passes any reading of the floor. That is the
//!    worse failure, because `+infinity` is conspicuous and a plausible number is not.
//! 8. **A run that did not survive the season is a result, not a crash.** ⚠ The one nobody
//!    predicted — not the plan, not the design review, and it appeared on the first mechanism
//!    column ever built. Knocking out a load-bearing process does not merely move the numbers:
//!    drop root water uptake and the crop never stores enough carbon to re-sow, so the two
//!    perennial chambers raise at the annual reset. That is the *ordinary* outcome of an
//!    interesting knockout, and arguably the strongest result one can produce, so it is
//!    printed in the engine's own words — and kept distinct from requirement 6, because
//!    "this scenario has no such process" says nothing about the science and "this scenario
//!    dies without it" says a great deal.
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

use super::mechanism::Composition;
use super::Substitution;
use crate::biosphere::drift::year_summaries;
use crate::biosphere::params::BiosphereParams;
use crate::biosphere::readouts::{
    floor_ppm, min_ppm, peak_lai, peak_w, segment_max, try_trajectory_composed, Trajectory,
    TrajectoryError,
};
use crate::biosphere::science_gates::{ScienceGate, GATES};
use crate::biosphere::{
    build_season_with, consumer_chamber_scenario, perennial_chamber_scenario,
    sealed_chamber_scenario, SeasonScenario, CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO,
    LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS,
};
use simcore::error::SimError;

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
    /// Needs the 15-year horizon, so it is off by default and `--long` asks for it.
    ///
    /// ⚠ *"minutes rather than seconds"* until 2026-09-01, when it was measured rather than
    /// asserted. Adding this roster's fifth CO₂ row — a second 15-year run — moved a two-column
    /// `--long` report from **2.3 s to 3.6 s**, and the short report **not at all** (1.1 s
    /// either side). Release build, min-of-5, both binaries alternated in one window because
    /// single passes on this box disagreed by 3× with the antivirus scanning.
    ///
    /// The short report's zero is structural, not luck: [`measure_composed`] skips a run whose
    /// `needed` set is empty, so a `long: true` spec costs nothing on the default path. Between
    /// them those two numbers are what discharged the price the fifth row had been parked
    /// behind — *"a second 15-year run on every long-report invocation"* was true and cost
    /// about a second.
    pub long: bool,
    fold: fn(&Trajectory) -> f64,
    /// The per-step series `fold` reads — requirement 7's subject.
    ///
    /// ⚠ Data beside the fold rather than re-derived in the constancy check, for the reason
    /// the `scenario` pairing is data: a loop that guessed which series a fold reads would be
    /// a second copy of the pairing, and the copies would disagree the first time a fold
    /// changed. `peak_w` reads three, which is why this is a list and not one accessor.
    series: fn(&Trajectory) -> Vec<&[f64]>,
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
        series: |t| vec![&t.leaf_c],
    },
    ReadoutSpec {
        scenario: "open_season",
        quantity: "peak W excl. fibrous roots (t/ha)",
        informs: &["peak W excl. fibrous roots (t/ha)"],
        degenerate: None,
        long: false,
        fold: peak_w,
        series: |t| vec![&t.leaf_c, &t.stem_c, &t.storage_c],
    },
    ReadoutSpec {
        scenario: "sealed_chamber",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: false,
        fold: min_ppm,
        series: |t| vec![&t.carbon_pool],
    },
    ReadoutSpec {
        scenario: "perennial_chamber",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: false,
        fold: min_ppm,
        series: |t| vec![&t.carbon_pool],
    },
    ReadoutSpec {
        scenario: "consumer_chamber",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: false,
        fold: min_ppm,
        series: |t| vec![&t.carbon_pool],
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
        series: |t| vec![&t.leaf_c],
    },
    ReadoutSpec {
        scenario: "perennial_long_horizon",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: true,
        fold: min_ppm,
        series: |t| vec![&t.carbon_pool],
    },
    ReadoutSpec {
        scenario: "consumer_long_horizon",
        quantity: "season-low chamber CO2 (ppm)",
        informs: &["season-low chamber CO₂ (ppm)"],
        degenerate: None,
        long: true,
        fold: min_ppm,
        series: |t| vec![&t.carbon_pool],
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
        // ⚠ The same scenario as `consumer_chamber`, driven 15 years instead of 5 — a second
        // run, not a longer one, because `measure_composed` folds each run once and the two
        // horizons are two rows. Its trough falls in year 5, so today it reads identically to
        // its 5-year sibling; `margins::the_five_margins_are_pinned_not_merely_positive` says
        // why that is a claim rather than a duplicate.
        (
            "consumer_long_horizon",
            consumer_chamber_scenario(),
            LONG_HORIZON_YEARS,
            true,
        ),
    ]
}

/// What one column varies against the frozen tree.
///
/// The two halves of the harness, as one type so a table can hold both: `Values` is the
/// value switch (`docs/log/value-switch-harness.md`), `Mechanism` the science switch.
pub enum Change {
    /// Param substitutions, applied before assembly. **Always applicable** — every scenario
    /// loads every param file, so a substitution reaches every row of every column.
    Values(Vec<Substitution>),
    /// A flow composition, applied after assembly, at the frozen params. Applicable only to
    /// the scenarios whose registry contains its targets — see [`Composition`].
    Mechanism(Composition),
}

/// One measured column of the table.
pub struct Column {
    /// What was substituted, in the reader's words (`"frozen"` for the baseline).
    pub label: String,
    /// `(spec index, value)` for every spec measured.
    pub values: Vec<(usize, f64)>,
    /// `(spec index, why)` for every spec this column could **not** be measured on.
    ///
    /// ⚠ Requirement 6. A mechanism variant whose target is not in a scenario's registry is
    /// not a failure and not a zero — it is a question that scenario cannot answer. Carried
    /// so the renderer prints a marked cell; what it replaced was a cell that simply was not
    /// there, and a rose/fell count quietly taken over fewer rows.
    pub not_applicable: Vec<(usize, String)>,
    /// `(spec index, why)` for every spec whose run **did not survive the season**.
    ///
    /// ⚠ Requirement 8, and it is the one nobody predicted — neither the plan nor the design
    /// review. Knocking out a load-bearing process does not merely move the numbers: it can
    /// end the run. Dropping root water uptake starves the crop, the perennial chambers raise
    /// at the annual reset (*"seed bank too small to re-sow"*), and that is the **ordinary**
    /// outcome of an interesting knockout rather than an edge case. It is a result — arguably
    /// the strongest one a knockout can produce — so it is printed, with the engine's own
    /// words, and kept distinct from [`Column::not_applicable`]: "this scenario has no such
    /// process" says nothing about the science, "this scenario dies without it" says a lot.
    pub failed: Vec<(usize, String)>,
    /// Spec indices whose input series never moved over the whole run — requirement 7.
    pub constant: Vec<usize>,
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

/// Whether a per-step series ever moved. Bitwise, so a `-0.0`/`0.0` pair reads as movement
/// and a series that returns to where it started still counts as having moved.
fn is_constant(series: &[f64]) -> bool {
    match series.first() {
        // ⚠ An EMPTY series is deliberately not "constant". Emptiness is the folds' own
        // assertion to make (`min_ppm` panics on it), and calling it constant here would turn
        // a loud wiring error into a printed warning.
        None => false,
        Some(first) => series.iter().all(|v| v.to_bits() == first.to_bits()),
    }
}

/// Whether every series a readout folds never moved — requirement 7's predicate, and the
/// one place `all` rather than `any` is decided.
///
/// ⚠ **`all`, and the difference is scientific rather than stylistic.** `peak_w` folds three
/// organ series. If storage froze while leaf and stem grew, the quantity the row reports still
/// moved and the number is doing its job — flagging it would put a "never moved" warning on a
/// live readout, which is how a warning stops being read. Only a readout whose *every* input
/// is frozen is reporting its own initial condition.
///
/// ⚠ It is a named function rather than a line inside the loop because that is the only way
/// the rule has a subject a test can point at: the mixed case (one series frozen, the others
/// not) has **no demonstrated composition that produces it** — see
/// `a_readout_is_frozen_only_when_every_series_it_folds_is` — so the rule is pinned over a
/// constructed trajectory. That test is evidence about the *rule*, not about a run.
fn readout_is_frozen(spec: &ReadoutSpec, t: &Trajectory) -> bool {
    (spec.series)(t).iter().all(|s| is_constant(s))
}

/// Measure every applicable spec at `p`, through the frozen build — the value-switch column.
/// ⚠ The `expect` is not a shrug. With no composition the only `Err` route left is
/// `build_season_with` itself refusing these params — every *run* failure is captured as a
/// dead cell rather than returned — and a substitution that a compartment builder rejects has
/// already been rejected by the frozen bounds in [`super::biosphere_with`]. If this ever fires
/// it is the value seam's guard having been bypassed, not a case this function should absorb.
pub fn measure(label: &str, p: &BiosphereParams, long: bool) -> Column {
    measure_composed(label, p, long, None)
        .expect("the frozen build refused params the value seam's bounds had accepted")
}

/// The one measurement body: every spec of every run, optionally through a composition.
///
/// `comp` is `None` for the baseline and for a value variant, and the flow list is then the
/// frozen one. With a composition, each run is checked for applicability **before** it is
/// driven, so a scenario the request does not reach costs one assembly rather than a season.
pub fn measure_composed(
    label: &str,
    p: &BiosphereParams,
    long: bool,
    comp: Option<&Composition>,
) -> Result<Column, SimError> {
    let mut values = Vec::new();
    let mut not_applicable = Vec::new();
    let mut failed = Vec::new();
    let mut constant = Vec::new();
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
        // ⚠ Applicability is asked **before** the run and answered per scenario. A composition
        // naming a flow this scenario's build does not carry is not an error here: the frozen
        // scenarios do not share a flow set, and refusing the whole comparison would make the
        // report unusable for exactly the swaps worth running. Every OTHER refusal in
        // `build_season_composed` is about the request rather than the scenario, so it still
        // propagates and stops the comparison.
        let absent = match comp {
            Some(c) => c.absent_targets(&scenario, p)?,
            None => Vec::new(),
        };
        if !absent.is_empty() {
            let why = format!("{absent:?} is not in {name}'s registry");
            not_applicable.extend(needed.iter().map(|i| (*i, why.clone())));
            continue;
        }
        let build = |s: &SeasonScenario, p: &BiosphereParams| match comp {
            Some(c) => c.apply(s, p),
            None => build_season_with(s, p),
        };
        // ⚠ The two failure modes are handled differently on purpose (see `TrajectoryError`).
        // A malformed composition is wrong under every scenario and stops the comparison; a
        // run that does not survive the season is this scenario's answer to the question, and
        // requirement 8 is that it gets printed rather than crashing the report.
        let t = match try_trajectory_composed(scenario, years, perennial, p, &build) {
            Ok(t) => t,
            Err(TrajectoryError::Setup(e)) => return Err(e),
            Err(TrajectoryError::Run(e)) => {
                let why = format!("{name}: {e}");
                failed.extend(needed.iter().map(|i| (*i, why.clone())));
                continue;
            }
        };
        rationed += t.rationed;
        events += t.events;
        for i in needed {
            values.push((i, (SPECS[i].fold)(&t)));
            if readout_is_frozen(&SPECS[i], &t) {
                constant.push(i);
            }
        }
    }
    Ok(Column {
        label: label.to_string(),
        values,
        not_applicable,
        failed,
        constant,
        floor_ppm: floor_ppm(p),
        rationed,
        events,
    })
}

/// The baseline column plus one per variant, in the order given.
pub fn compare(
    variants: &[(String, Vec<Substitution>)],
    long: bool,
) -> Result<Vec<Column>, SimError> {
    let changes: Vec<(String, Change)> = variants
        .iter()
        .map(|(label, subs)| (label.clone(), Change::Values(subs.clone())))
        .collect();
    compare_changes(&changes, long)
}

/// The baseline column plus one per variant, values and mechanisms mixed.
///
/// ⚠ A mechanism column takes the **frozen** params. This harness changes one thing at a time
/// by construction; a column varying a coefficient *and* a process is spelled by a caller who
/// means it, through [`measure_composed`] against substituted params.
pub fn compare_changes(variants: &[(String, Change)], long: bool) -> Result<Vec<Column>, SimError> {
    let frozen = super::biosphere_with(&[]).map_err(as_request_error)?;
    let mut columns = vec![measure("frozen", &frozen, long)];
    for (label, change) in variants {
        columns.push(match change {
            Change::Values(subs) => {
                let p = super::biosphere_with(subs).map_err(as_request_error)?;
                measure(label, &p, long)
            }
            Change::Mechanism(comp) => measure_composed(label, &frozen, long, Some(comp))?,
        });
    }
    Ok(columns)
}

/// A bad substitution is a bad **request**, and the report has one error type so a caller does
/// not have to know which half of the harness refused it.
fn as_request_error(e: config::ConfigError) -> SimError {
    SimError::Validation(e.to_string())
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
    // ⚠ "lab report", not "value-switch report", which is what it said until slice 4 handed
    // the same renderer to the mechanism half. A shared renderer announcing one of its two
    // callers is a small thing that misfiles evidence: a table headed "value-switch" over a
    // knockout column is the kind of caption a reader quotes later.
    out.push_str(
        "biosphere lab report — MEASURED QUANTITIES, not pass/fail verdicts.\n\
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
        // ⚠ The baseline column measures every rendered row by construction — the frozen build
        // reaches every scenario. If it ever does not, the row is unreadable rather than
        // partly readable, and saying so beats printing variant numbers with nothing to
        // difference them against.
        let Some(b) = value_of(base, i) else {
            out.push_str(
                "    ⚠ THE BASELINE DID NOT MEASURE THIS ROW — no cell below can be compared\n\n",
            );
            continue;
        };
        for col in columns {
            let mut line = match value_of(col, i) {
                Some(v) => {
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
                    // Requirement 7: a plausible number off a series that never moved.
                    if col.constant.contains(&i) {
                        line.push_str(
                            "  <- CONSTANT SERIES: this quantity's stock never moved, so the \
                             number is the run's starting value and not a result",
                        );
                    }
                    line
                }
                // Requirements 6 and 8: a marked cell, never a missing one.
                None => {
                    let na = col.not_applicable.iter().find(|(j, _)| *j == i);
                    let dead = col.failed.iter().find(|(j, _)| *j == i);
                    match (na, dead) {
                        (Some((_, why)), _) => format!(
                            "    {:<38} {:>14}  <- NOT APPLICABLE: {why}",
                            col.label, "n/a"
                        ),
                        (None, Some((_, why))) => format!(
                            "    {:<38} {:>14}  <- RUN DID NOT COMPLETE: {why}",
                            col.label, "dead"
                        ),
                        (None, None) => format!(
                            "    {:<38} {:>14}  <- MEASURED BY NOTHING, and reported neither \
                             inapplicable nor dead — this is a bug in the report",
                            col.label, "?"
                        ),
                    }
                }
            };
            line.push('\n');
            out.push_str(&line);
        }
        out.push('\n');
    }

    out.push_str(
        "chamber CO2 compensation point (ppm) — the floor the CO2 rows are read against\n",
    );
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
    //
    // ⚠ The unmeasured rows are counted and named here, not skipped. Until the mechanism
    // variants arrived every column had every row, so `_ => {}` was unreachable; a mechanism
    // swap makes it ordinary, and a summary reading "1 rose, 0 fell" over a table where four
    // rows were never measured is requirement 3's wrong answer with a new cause.
    for col in columns.iter().skip(1) {
        let (mut up, mut down, mut flat, mut absent) = (0, 0, 0, 0);
        for (i, spec) in SPECS.iter().enumerate() {
            if spec.long && !long {
                continue;
            }
            match (value_of(base, i), value_of(col, i)) {
                (Some(b), Some(v)) if v > b => up += 1,
                (Some(b), Some(v)) if v < b => down += 1,
                (Some(_), Some(_)) => flat += 1,
                (Some(_), None) => absent += 1,
                _ => {}
            }
        }
        out.push_str(&format!(
            "{}: {up} rose, {down} fell, {flat} did not move{}{}\n",
            col.label,
            if absent > 0 {
                format!(", {absent} NOT MEASURED (see the marked cells above)")
            } else {
                String::new()
            },
            if up > 0 && down > 0 {
                "  <- OPPOSED: reading either family alone gives the wrong answer"
            } else {
                ""
            }
        ));
    }

    // Requirement 7 — the stock that never moved, gathered where a reader scanning the
    // warnings will see it rather than only beside its own cell.
    for col in columns {
        for i in &col.constant {
            out.push_str(&format!(
                "⚠ {}: {} / {} was folded over a series that NEVER MOVED — a mechanism swap can \
                 remove a stock's only writer, and the fold then returns the run's starting \
                 value, which is finite and plausible and means nothing\n",
                col.label, SPECS[*i].scenario, SPECS[*i].quantity
            ));
        }
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
    use crate::lab::mechanism::{FlowFactory, ScaledMechanism};
    use simcore::flow::Flow;

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
    /// it for a short spec and **nothing catches it for a `long: true` one**, which is why this
    /// exists.
    ///
    /// ⚠ Its example was `consumer_long_horizon` — *"already a scenario in [`GATES`] with no
    /// `runs()` entry, so the next spec added under it is the one that would vanish"* — and on
    /// 2026-09-01 that spec was added. **The prediction was exercised before it was fixed**: the
    /// spec landed alone first and this test failed by name on it, then the `runs()` entry landed
    /// and it passed. So the example is spent, and it is recorded as a discharged prediction
    /// rather than reworded into a fresh hypothetical — a guard whose example has been *run* is
    /// worth more than one whose example is still imaginary.
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
        let fields: Vec<&str> = SPECS.iter().flat_map(gates_for).map(|g| g.field).collect();
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
        assert!(
            text.contains("MEASURED QUANTITIES, not pass/fail"),
            "{text}"
        );
        assert!(
            text.contains("NO liveness_floors ROW IS IN THIS TABLE"),
            "{text}"
        );
    }

    // -------------------------------------------------------------------------------------
    // The mechanism half — slice 4 of the science-switch plan.
    // -------------------------------------------------------------------------------------

    /// A flow this scenario's own build carries, wrapped by `ScaledMechanism` at `factor`.
    ///
    /// Built per run, which is what [`FlowFactory`] exists for: five runs, and
    /// `Box<dyn Flow>` is not `Clone`.
    fn scaled(id: &'static str, factor: f64) -> (String, FlowFactory) {
        (
            id.to_string(),
            Box::new(move |s: &SeasonScenario, p: &BiosphereParams| {
                let (_state, registry) = build_season_with(s, p).expect("build");
                let flow = registry
                    .into_parts()
                    .0
                    .into_iter()
                    .find(|f| f.id() == id)
                    .unwrap_or_else(|| {
                        // ⚠ Not a convenience unwrap. A replacement factory is run only after
                        // `absent_targets` has cleared this scenario, so reaching this arm is the
                        // applicability pre-check having been skipped — which is what it says.
                        panic!(
                            "{id} is not in this build — the applicability pre-check did not run"
                        )
                    });
                Box::new(ScaledMechanism::new(flow, factor)) as Box<dyn Flow>
            }),
        )
    }

    fn short_report(label: &str, comp: Composition) -> (Vec<Column>, String) {
        let columns = compare_changes(&[(label.to_string(), Change::Mechanism(comp))], false)
            .expect("compare");
        let text = render(&columns, false);
        (columns, text)
    }

    /// ⚠⚠ **Direction one**, and it is the control the rest of this section rests on: a
    /// composition that asks for nothing must reproduce the baseline column exactly. The
    /// mechanism path assembles, tears the registry apart and rebuilds it; if that round trip
    /// moved a number, every mechanism column below would be part composition and part
    /// harness, with no way to tell which.
    #[test]
    fn an_empty_composition_reproduces_the_baseline_column_bit_for_bit() {
        let (columns, _) = short_report("nothing", Composition::default());
        let (base, empty) = (&columns[0], &columns[1]);
        assert_eq!(base.values.len(), empty.values.len(), "row count");
        for ((i, b), (j, v)) in base.values.iter().zip(&empty.values) {
            assert_eq!(i, j, "the rows came back in a different order");
            assert_eq!(
                b.to_bits(),
                v.to_bits(),
                "{} / {}: the composers' round trip moved it ({b} vs {v})",
                SPECS[*i].scenario,
                SPECS[*i].quantity
            );
        }
        assert!(empty.not_applicable.is_empty());
        assert!(empty.failed.is_empty(), "{:?}", empty.failed);
        assert!(empty.constant.is_empty(), "{:?}", empty.constant);
        assert_eq!(empty.rationed, 0);
        assert_eq!(empty.events, 0);
    }

    /// **Direction two**: a knockout reaches the table and moves it.
    ///
    /// `stem_remobilization` is in all four builds and the crop survives without it, so this
    /// is the clean case: every row measured, numbers moved, no marker of any kind. The
    /// interesting knockouts do not behave like this — see the test below.
    #[test]
    fn a_knockout_reaches_the_table() {
        let (columns, text) = short_report(
            "drop stem_remobilization",
            Composition::dropping(&["biosphere.stem_remobilization"]),
        );
        let (base, dropped) = (&columns[0], &columns[1]);
        assert!(
            dropped.not_applicable.is_empty() && dropped.failed.is_empty(),
            "expected every row measured, got n/a {:?} and dead {:?}",
            dropped.not_applicable,
            dropped.failed
        );
        assert_eq!(base.values.len(), dropped.values.len());
        assert!(
            base.values
                .iter()
                .zip(&dropped.values)
                .any(|((_, b), (_, v))| b.to_bits() != v.to_bits()),
            "the knockout column equals the baseline — it never reached the run:\n{text}"
        );
        assert!(!text.contains("NOT APPLICABLE"), "{text}");
        assert!(!text.contains("DID NOT COMPLETE"), "{text}");
        assert!(!text.contains("MEASURED BY NOTHING"), "{text}");
    }

    /// ⚠⚠ **Requirement 8**, and it is the finding this slice did not go looking for: a
    /// knockout can **end the run**, and that is the ordinary case rather than the exception.
    /// Without root water uptake the crop never stores enough carbon to re-sow, so both
    /// perennial chambers raise at the annual reset while the two non-perennial runs complete
    /// normally.
    ///
    /// Before this was handled the whole report panicked from inside `readouts`, four levels
    /// below the caller, on the first mechanism column anyone would think to run.
    ///
    /// ⚠ Both directions in one column, which is what makes it evidence: the dead runs are
    /// reported dead **and** the surviving runs are still measured. A report that gave up on
    /// the whole column would pass a one-sided version of this.
    #[test]
    fn a_knockout_that_kills_the_run_reports_it_and_keeps_the_runs_that_survived() {
        let (columns, text) = short_report(
            "drop root_zone_capture",
            Composition::dropping(&["biosphere.root_zone_capture"]),
        );
        let col = &columns[1];
        assert!(
            col.not_applicable.is_empty(),
            "root_zone_capture is in all four builds: {:?}",
            col.not_applicable
        );
        let dead: Vec<&str> = col.failed.iter().map(|(i, _)| SPECS[*i].scenario).collect();
        assert_eq!(
            dead,
            vec!["perennial_chamber", "consumer_chamber"],
            "the perennial runs are the ones that cannot re-sow without root uptake"
        );
        assert!(
            col.failed.iter().all(|(_, why)| why.contains("seed bank")),
            "the cell does not carry the engine's own reason: {:?}",
            col.failed
        );
        assert!(
            col.values.len() == 3,
            "the surviving runs stopped being measured: {:?}",
            col.values
        );
        assert!(text.contains("RUN DID NOT COMPLETE"), "{text}");
        assert!(text.contains("2 NOT MEASURED"), "{text}");
        assert!(!text.contains("MEASURED BY NOTHING"), "{text}");
    }

    /// ⚠⚠ **Requirement 6.** `biosphere.decomposition` is in the three chambers and not in the
    /// open field, so two of the five rows cannot be asked of this variant at all. Before this
    /// slice the renderer dropped those cells with no marker and the movement summary was
    /// silently taken over the rows that remained.
    ///
    /// The assertion is in both directions on purpose: the inapplicable rows say so **and** the
    /// applicable ones are still measured. A column that simply failed everywhere would pass a
    /// one-sided version of this test.
    ///
    /// ⚠ And this variant turns out to be the one that carries **every** cell state at once —
    /// two rows the open field cannot be asked, two runs that die without soil carbon, one
    /// number. Four of the five short rows answer nothing, which the summary line says. A
    /// reader who saw only "1 rose, 0 fell" would take a one-row table for a five-row one.
    #[test]
    fn a_variant_that_does_not_reach_a_scenario_says_so_in_the_cell_and_the_summary() {
        let (columns, text) = short_report(
            "drop decomposition",
            Composition::dropping(&["biosphere.decomposition"]),
        );
        let col = &columns[1];
        let absent: Vec<&str> = col
            .not_applicable
            .iter()
            .map(|(i, _)| SPECS[*i].scenario)
            .collect();
        assert_eq!(
            absent,
            vec!["open_season", "open_season"],
            "the open field's two rows are the ones decomposition cannot be asked of"
        );
        let dead: Vec<&str> = col.failed.iter().map(|(i, _)| SPECS[*i].scenario).collect();
        assert_eq!(
            dead,
            vec!["perennial_chamber", "consumer_chamber"],
            "without soil decomposition the perennial chambers cannot re-sow"
        );
        let measured: Vec<&str> = col.values.iter().map(|(i, _)| SPECS[*i].scenario).collect();
        assert_eq!(
            measured,
            vec!["sealed_chamber"],
            "the one run that both carries decomposition and survives without it"
        );
        assert!(text.contains("NOT APPLICABLE"), "{text}");
        assert!(
            text.contains("is not in open_season's registry"),
            "the cell does not say WHY:\n{text}"
        );
        assert!(text.contains("RUN DID NOT COMPLETE"), "{text}");
        assert!(
            text.contains("4 NOT MEASURED"),
            "the movement summary counted over the measured rows only:\n{text}"
        );
        assert!(!text.contains("MEASURED BY NOTHING"), "{text}");
    }

    /// ⚠⚠ **Requirement 7 — the guard slice 4 actually owed**, and the plan named a different
    /// one (see the module header). Zeroing every flow that writes leaf carbon leaves the leaf
    /// series flat for the whole run: `peak_lai` then returns the sown value, which is a
    /// perfectly ordinary-looking number off a run where nothing grew.
    ///
    /// ⚠ Three flows, not one, and that is a measurement rather than a convenience: every
    /// stock these readouts fold has **at least two** writers on the frozen tree, so no single
    /// swap can freeze one. `build_season_composed` takes several changes at once precisely
    /// because that is the shape a real pair takes.
    ///
    /// The two-direction half is [`the_frozen_baseline_has_no_constant_series`] — without it
    /// this assertion would also pass on a report that flagged everything.
    #[test]
    fn a_readout_folded_over_a_frozen_series_is_flagged() {
        let comp = Composition {
            replacements: vec![
                scaled("biosphere.allocation", 0.0),
                scaled("biosphere.maintenance_respiration", 0.0),
                scaled("biosphere.senescence", 0.0),
            ],
            ..Composition::default()
        };
        let (columns, text) = short_report("no leaf carbon flows", comp);
        let col = &columns[1];
        let flagged: Vec<&str> = col.constant.iter().map(|i| SPECS[*i].quantity).collect();
        assert!(
            flagged.contains(&"peak LAI (m2 m-2)"),
            "the leaf series was frozen and nothing said so: {flagged:?}\n{text}"
        );
        assert!(text.contains("CONSTANT SERIES"), "{text}");
        assert!(text.contains("NEVER MOVED"), "{text}");

        // ⚠ And NOT every row: the chamber pool still has writers this composition left alone,
        // so a flag on it would mean the check is reporting the column rather than the series.
        assert!(
            !flagged.contains(&"season-low chamber CO2 (ppm)"),
            "the chamber CO2 series was flagged too — the guard is not reading its own series"
        );
    }

    /// Anti-vacuity for the guard above: on the frozen tree **no** readout's series is
    /// constant, so the flag cannot be firing green and being weakened later to silence it.
    #[test]
    fn the_frozen_baseline_has_no_constant_series() {
        let col = measure("frozen", &crate::biosphere::params::biosphere(), false);
        assert!(
            col.constant.is_empty(),
            "the frozen tree already folds over a series that never moves: {:?}",
            col.constant
                .iter()
                .map(|i| (SPECS[*i].scenario, SPECS[*i].quantity))
                .collect::<Vec<_>>()
        );
        assert!(!col.values.is_empty(), "nothing was measured");
    }

    /// ⚠⚠ **The negative half of requirement 7, and the mutation battery is why it exists.**
    /// `a_readout_folded_over_a_frozen_series_is_flagged` freezes leaf, stem *and* storage at
    /// once, so it cannot tell `all` from `any` — turning the fold into `any` left the whole
    /// suite green. The rule is that a readout is degenerate only when **every** series it
    /// folds is frozen; `peak_w` reads three, and one frozen organ out of three is a live
    /// number, not a warning.
    ///
    /// ⚠ Built rather than run, and the record says so: no composition in the tree freezes one
    /// of the three organs alone (they share `biosphere.allocation` as their only writer, so
    /// removing it freezes all three — measured in
    /// `the_frozen_baselines_organ_series_each_move` below). This pins the **rule**; the mixed
    /// case's reachability from a real season is unmeasured and may be nil.
    #[test]
    fn a_readout_is_frozen_only_when_every_series_it_folds_is() {
        let spec = SPECS
            .iter()
            .find(|s| s.scenario == "open_season" && s.quantity.starts_with("peak W"))
            .expect("the three-series readout is gone — this test has lost its subject");
        assert_eq!(
            (spec.series)(&stub(&[1.0, 1.0], &[2.0, 2.0], &[3.0, 3.0])).len(),
            3,
            "peak W no longer folds three series; the all-vs-any question moved with it"
        );

        let mixed = stub(&[1.0, 1.5], &[2.0, 2.0], &[3.0, 3.0]);
        assert!(
            !readout_is_frozen(spec, &mixed),
            "a readout with one live series was called frozen — the fold is `any`, not `all`,              and every three-organ row would carry a warning while its number moved"
        );

        let all_frozen = stub(&[1.0, 1.0], &[2.0, 2.0], &[3.0, 3.0]);
        assert!(
            readout_is_frozen(spec, &all_frozen),
            "nothing moved and the readout was not flagged"
        );
    }

    /// A `Trajectory` carrying only the three organ series — enough for
    /// [`readout_is_frozen`], which reads nothing else.
    fn stub(leaf: &[f64], stem: &[f64], storage: &[f64]) -> Trajectory {
        Trajectory {
            scenario: DEFAULT_SCENARIO,
            params: crate::biosphere::params::biosphere(),
            leaf_c: leaf.to_vec(),
            stem_c: stem.to_vec(),
            storage_c: storage.to_vec(),
            carbon_pool: Vec::new(),
            consumer_c: Vec::new(),
            rationed: 0,
            events: 0,
            years: 1,
        }
    }

    /// Anti-vacuity for `all`: on the frozen tree each of the three organ series moves **on
    /// its own**. If one were already frozen, `all` would be silently masking it and the
    /// choice above would be hiding a real degeneracy rather than avoiding a false alarm.
    #[test]
    fn the_frozen_baselines_organ_series_each_move() {
        let t = crate::biosphere::readouts::trajectory(
            DEFAULT_SCENARIO,
            1,
            false,
            &crate::biosphere::params::biosphere(),
        );
        for (name, series) in [
            ("leaf_c", &t.leaf_c),
            ("stem_c", &t.stem_c),
            ("storage_c", &t.storage_c),
        ] {
            assert!(
                !is_constant(series),
                "{name} never moves in the frozen open season, so `all` is masking a                  permanently degenerate input to peak W"
            );
        }
    }

    /// ⚠ The applicability pre-check must cover **replacements**, not only drops. The battery
    /// found this open: `Composition::targets` chaining nothing instead of the replacement
    /// ids left the whole suite green, because every applicability test used a drop.
    ///
    /// `biosphere.decomposition` is absent from `open_season` and present in the chambers, so
    /// one column carries both answers. The factor is 1.0 — the run must be *identical* where
    /// it applies, so this test is about the cell's marking and nothing else.
    #[test]
    fn a_replacement_that_does_not_reach_a_scenario_is_marked_not_applicable() {
        let comp = Composition {
            replacements: vec![scaled("biosphere.decomposition", 1.0)],
            ..Composition::default()
        };
        let (columns, text) = short_report("replace decomposition (x1)", comp);
        let (base, col) = (&columns[0], &columns[1]);

        let na: Vec<&str> = col
            .not_applicable
            .iter()
            .map(|(i, _)| SPECS[*i].scenario)
            .collect();
        assert!(
            na.contains(&"open_season"),
            "open_season carries no decomposition and the cell was not marked n/a: {na:?}
{text}"
        );
        assert!(
            text.contains("NOT APPLICABLE"),
            "the reader is not told why the cell is blank
{text}"
        );

        // ⚠ And it must still MEASURE where the flow exists — an applicability check that
        // marked every cell would pass the assertion above while measuring nothing.
        let measured: Vec<&str> = col.values.iter().map(|(i, _)| SPECS[*i].scenario).collect();
        assert!(
            measured.contains(&"sealed_chamber"),
            "the chambers do carry decomposition and nothing was measured there: {measured:?}"
        );
        for (i, v) in &col.values {
            let b = base
                .values
                .iter()
                .find(|(j, _)| j == i)
                .map(|(_, b)| *b)
                .expect("the baseline did not measure a row the variant did");
            assert_eq!(
                b.to_bits(),
                v.to_bits(),
                "{}: a x1.0 replacement moved the run",
                SPECS[*i].quantity
            );
        }
    }

    /// A request that is wrong about *itself*, rather than about a scenario, still stops the
    /// whole comparison. The applicability path must not have turned every refusal into an
    /// `n/a` cell.
    #[test]
    fn a_bad_request_is_still_an_error_and_not_a_blank_column() {
        let comp = Composition::dropping(&["biosphere.allocation", "biosphere.allocation"]);
        let err = compare_changes(&[("twice".to_string(), Change::Mechanism(comp))], false)
            .map(|columns| columns.len())
            .expect_err("naming one target twice is a bad request");
        assert!(
            format!("{err:?}").contains("named twice"),
            "refused, but not by the guard under test: {err:?}"
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
