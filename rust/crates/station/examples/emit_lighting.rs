//! Run the Power → biosphere `LIGHTING_SCENARIO` (two-rate, Euler) and emit its day-7 final
//! `State` as `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `lighting_state.json` at
//! **Tier 2** — the lamp forces the FvCB biosphere's PAR; the biosphere runs every master day.

fn main() {
    print!("{}", station::goldens::lighting());
}
