//! Run the open-field `DEFAULT_SCENARIO` season in the Rust port and emit its final
//! `State` (Phase-7 P7.4). Compared to `season_euler_state.json` at **Tier 2** — the
//! FvCB / Penman–Monteith / weather transcendental surface. Euler-daily, 1 season.

fn main() {
    print!("{}", domains::goldens::season());
}
