//! Run the fully-coupled sealed station `SEALED_STATION_SCENARIO` (two-rate, Euler,
//! `with_harvest=False` / `close_feces=False` — the Tier-2 scope) over the multi-year
//! horizon and emit its day-boundary final `State` as `sim_io`-shaped JSON (Phase-7 Step 5).
//! Compared to `sealed_station_state.json` at **Tier 2**. The ~1.3 M-sub-step run's real
//! payload is the per-sub-step conservation assert inside the driver (the Tier-0 gate): a
//! completed run is itself proof the combined ledger balanced every sub-step over the full
//! five-domain assembly.

fn main() {
    print!("{}", station::goldens::sealed_station());
}
