//! Run the standalone Power `SELF_DISCHARGE` build (the two forced flows + the opt-in
//! donor-controlled `SelfDischarge`) over 14 days and emit its final `State` (Phase-7
//! Step 3). Compared to `power_self_discharge_state.json` at **Tier 2 (measured band)**
//! — it reuses `BOUNDED_SOC_SCENARIO`'s half-sine solar (inherits `sin`); the leak leg
//! itself is linear.

fn main() {
    print!("{}", domains::goldens::power_self_discharge());
}
