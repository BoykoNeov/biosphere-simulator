//! Run the standalone ECLSS `STEADY_STATE_SCENARIO` in the Rust port and emit its final
//! `State` as `sim_io`-shaped JSON (Phase-7 Step 3). Compared to `eclss_state.json` at
//! **Tier 1 (bit-exact)** — ECLSS is transcendental-free (linear control loops), so the
//! op-order of each `(k · stock) · dt` / `(setpoint − cabin_o2)` is load-bearing.

fn main() {
    print!("{}", domains::goldens::eclss());
}
