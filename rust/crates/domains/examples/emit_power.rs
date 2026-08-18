//! Run the standalone Power `BOUNDED_SOC_SCENARIO` in the Rust port and emit its 7-day
//! final `State` as `sim_io`-shaped JSON (Phase-7 Step 3). Compared to `power_state.json`
//! at **Tier 2 (measured band)** — the half-sine `solar_schedule` (`sin`) is the
//! transcendental. The derived `balanced_load_w` is *re-computed* here (ported, not
//! smuggled from the golden).

fn main() {
    print!("{}", domains::goldens::power());
}
