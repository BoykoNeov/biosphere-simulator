//! Run the biomass/food `HARVEST_SCENARIO` (two-rate, Euler; `with_harvest=True`,
//! `close_feces=True` — the closed trophic ring) and emit its day-7 final `State` as
//! `sim_io`-shaped JSON (Phase-7 Step 5). Compared to `harvest_state.json` at **Tier 2** —
//! built on the FvCB greenhouse. `thermal_time0` starts the plant past anthesis (grain-filling).

fn main() {
    print!("{}", station::goldens::harvest());
}
