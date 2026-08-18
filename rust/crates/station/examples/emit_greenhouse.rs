//! Run the biosphere ↔ cabin `GREENHOUSE_SCENARIO` (two-rate, Euler) and emit its day-7
//! final `State` as `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `greenhouse_state.json`
//! at **Tier 2** — the FvCB biosphere runs every master day. The per-sub-step conservation
//! assert inside the two-rate driver is the Tier-0 gate (a completed run is the proof).

fn main() {
    print!("{}", station::goldens::greenhouse());
}
