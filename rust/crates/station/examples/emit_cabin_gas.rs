//! Run the coupled crew ↔ ECLSS `CABIN_GAS_SCENARIO` and emit its final `State` as
//! `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `cabin_gas_state.json` at **Tier 1
//! (bit-exact)** — the cabin loop is transcendental-free (forced/linear crew respiration +
//! first-order ECLSS controls; no biosphere, no `sin`/`pow`), the strongest cross-port gate.

fn main() {
    print!("{}", station::goldens::cabin_gas());
}
