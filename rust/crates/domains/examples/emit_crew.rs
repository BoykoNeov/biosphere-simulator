//! Run the standalone Crew `MISSION_SCENARIO` in the Rust port and emit its 7-day
//! final `State` as `sim_io`-shaped JSON (Phase-7 Step 3).
//!
//! Unlike the Step-0 `simcore/examples/emit_crew.rs` (which hand-built the golden's
//! own values to test the *interchange*), this **computes** crew_state from the ported
//! engine — the real cross-port validation. `tests/crossport/test_crossport.py` runs
//! this and compares the output to `crew_state.json` at **Tier 1 (bit-exact)**.
//!
//! Tier-0 invariants are asserted here in Rust: `rationed == 0`, `events == ()`, and —
//! implicitly — conservation every step (the run would have errored inside
//! `step_report` otherwise).

fn main() {
    print!("{}", domains::goldens::crew());
}
