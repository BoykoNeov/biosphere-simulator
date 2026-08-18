//! Run the standalone Thermal `EQUILIBRIUM_SCENARIO` in the Rust port and emit its final
//! `State` (Phase-7 Step 3). Compared to `thermal_state.json` at **Tier 2 (measured
//! band)** — the Stefan-Boltzmann `RadiatorReject` computes `(T⁴ − T_space⁴)` via
//! `powf(4.0)` every step (the nonlinear attractor, the plan's first real libm audit).

fn main() {
    print!("{}", domains::goldens::thermal());
}
