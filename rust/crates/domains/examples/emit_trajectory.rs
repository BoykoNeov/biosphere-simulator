//! Emit a biosphere run's **per-step trajectory** — every state it passes through, not
//! just its final one (the reference-flip plan, slice 1).
//!
//! Python's `run_season` has always returned the whole `states` list, and the oracle
//! comparison walks it day by day; the Rust port's 24 `emit_*` examples emit only a final
//! `State`. This example closes that gap. Nothing consumes it yet: it is additive, and it
//! exists so the one unknown in the flip (a per-step export that does not exist today) is
//! de-risked before anything with a contract behind it moves.
//!
//! ⚠ **A separate binary on purpose — `emit_season` is not given a flag.** That example's
//! stdout feeds a frozen golden comparison; its bytes stay exactly as they are.
//!
//! Two runs, by `$1`:
//!
//! * `season` (default) — `DEFAULT_SCENARIO`, 1 weather year, `run_season`, **no reset**.
//!   Mirrors `emit_season`'s run, so the trajectory's last row is that golden's state.
//! * `perennial` — `perennial_chamber_scenario()` for [`PERENNIAL_TRAJECTORY_YEARS`],
//!   `run_perennial`, **reset armed**. This case exists for the reset hook alone: it is
//!   the one place the two ports' observer semantics could genuinely differ, because both
//!   drivers record the **pre-reset** state and never the reset instant itself. A
//!   season-only slice would leave that unproven.

use domains::biosphere::{
    perennial_chamber_scenario, run_perennial, run_season, season_setup, season_steps,
    steps_for_years, SeasonScenario, BIO_DT, DEFAULT_SCENARIO,
};
use simcore::snapshot::TrajectoryWriter;
use simcore::state::State;

/// Horizon of the perennial trajectory case, in years.
///
/// ⚠ **2 is the SMALLEST horizon that fires the annual reset, and that is the whole
/// reason for the number.** `run_perennial` consults the hook with the *pre-step* `n`, so
/// a 1-year run (`steps == season_steps()`) checks `n = 0 ..= season_steps() - 1` and the
/// boundary is never reached. This is deliberately **not** the 5-year
/// `perennial_chamber_state` golden's horizon: this case is about driver/observer
/// semantics, not about that golden, and a 5-year trajectory is ~5x the bytes for no
/// additional reset behaviour.
const PERENNIAL_TRAJECTORY_YEARS: usize = 2;

fn main() {
    let perennial = matches!(std::env::args().nth(1).as_deref(), Some("perennial"));
    let (scenario, years): (SeasonScenario, usize) = if perennial {
        (perennial_chamber_scenario(), PERENNIAL_TRAJECTORY_YEARS)
    } else {
        (DEFAULT_SCENARIO, 1)
    };

    let (state, integrator, resolver) = season_setup(&scenario, years).expect("season_setup");
    let steps = steps_for_years(years);

    // The observer fires on the initial state and after each step, so the writer ends with
    // `steps + 1` rows and row `i` is `n == i`.
    let mut writer = TrajectoryWriter::new();
    let mut observe = |s: &State| writer.push(s);

    let (_final_state, rationed, events) = if perennial {
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
    }
    .expect("run");

    // The same Tier-0 structural invariants the state emitters assert. A trajectory taken
    // from an arbitrating or extinction-hit run would be comparing two different regimes.
    assert_eq!(rationed, 0, "Tier-0: trajectory run rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: trajectory run events must be empty"
    );
    assert_eq!(
        writer.len(),
        steps + 1,
        "observer must fire once on the initial state and once per step"
    );

    print!("{}", writer.finish(BIO_DT));
}
