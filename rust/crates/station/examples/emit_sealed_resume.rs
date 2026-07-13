//! Phase-8 (P8.8) — the headless reference for the **sealed cross-boundary parity** smoke.
//! Builds the full sealed station through the shared [`station::palette::build_scenario`] (the
//! same builder the Godot cdylib uses) and steps [`SEALED_RESUME_DAYS`] master days — a few
//! days past one season, so the re-sow (`slow_reset`) adopt branch fires — then emits the
//! `sim_io` hex-float snapshot.
//!
//! `godot/sealed_smoke.gd` drives the identical `build("sealed") + step_n(SEALED_RESUME_DAYS)`
//! through the actual cdylib; `tests/crossport/test_godot_two_rate_parity.py` asserts the two
//! snapshots are **byte-for-byte** identical — the "the two-rate, season-crossing sim survives
//! the FFI boundary bit-exact" proof (Step-1's smoke was single-rate; this is the multi-domain,
//! re-sown, "fast-forward decades" arm). Both sides are pure Rust on the same libm.

use station::palette::build_scenario;
use station::scenario::SEALED_RESUME_DAYS;

fn main() {
    let (mut session, _display) = build_scenario("sealed").expect("build sealed palette session");
    session
        .step_n(SEALED_RESUME_DAYS)
        .expect("step sealed to the resume horizon");

    assert_eq!(
        session.total_rationed(),
        0,
        "Tier-0: sealed resume horizon must be well-fed (rationed == 0)"
    );
    assert!(
        session.events().is_empty(),
        "Tier-0: sealed resume horizon must have no extinction events"
    );

    print!("{}", simcore::snapshot::from_engine(session.state()).to_json());
}
