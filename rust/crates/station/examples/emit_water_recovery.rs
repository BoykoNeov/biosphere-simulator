//! Run the crew water-recovery `WATER_RECOVERY_SCENARIO` and emit its final `State` as
//! `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `water_recovery_state.json` at
//! **Tier 1 (bit-exact)** — the donor-controlled `WaterRecovery` is still only `*`/`+`/`-`/
//! `/` atop the transcendental-free cabin (no biosphere).

fn main() {
    print!("{}", station::goldens::water_recovery());
}
