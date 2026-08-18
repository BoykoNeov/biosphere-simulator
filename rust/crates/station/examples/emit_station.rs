//! Run the coupled Power → Thermal `HEAT_CLOSURE_SCENARIO` (single-rate, Euler) and emit
//! its 7-day final `State` as `sim_io`-shaped JSON (Phase-7 Step 5). Compared to
//! `station_state.json` at **Tier 2 (measured band)** — Power's half-sine (`sin`) dissipates
//! into Thermal's `T⁴` radiator, both transcendentals in one graph. The node starts at the
//! dissipation-set equilibrium (`node0 = None ⇒ equilibrium_node_heat`).

fn main() {
    print!("{}", station::goldens::station());
}
