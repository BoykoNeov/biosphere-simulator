//! Run the O₂-poor sealed chamber (`SEALED_CHAMBER_SCENARIO`, 3 yr, `run_season`) in the
//! Rust port and emit its final `State` (Phase-7 P7.4). Compared to
//! `sealed_chamber_state.json` at **Tier 2** — the closed biosphere (FvCB + the
//! decomposer gas loop + f_O2 self-limitation).

fn main() {
    print!("{}", domains::goldens::sealed_chamber());
}
