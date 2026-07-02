//! Emit a small synthetic snapshot that exercises the two `snapshot::to_json`
//! branches the `crew_state` example does NOT reach:
//!
//!   * a **non-empty `aux`** map (crew's is `{}`), and
//!   * a **multi-element `composition`** (crew's are all 1:1) — the comma logic
//!     between coefficient entries.
//!
//! Both are live paths for real frozen data: the biosphere goldens carry a
//! `thermal_time` aux, and the station CO₂ stocks are `{carbon:1, oxygen:2}`
//! (cabin_gas / greenhouse / sealed). The values here are transcribed from actual
//! goldens (a biosphere `thermal_time`, the cabin_gas `boundary.co2_removed` CO₂
//! stock) so `sim_io.loads` reconstructs them through the real invariant checks.
//! `tests/crossport/` round-trips this and asserts the aux value and both
//! composition coefficients survive.

use simcore::hexfloat;
use simcore::snapshot::{State, Stock};

fn main() {
    let state = State {
        n: 5,
        rng_seed: 0xDEAD_BEEF, // also exercises the 0x-hex seed formatting (non-zero)
        // Non-empty aux — value from sealed_chamber_state.json.
        aux: vec![(
            "thermal_time".to_string(),
            hexfloat::parse("0x1.31851eb851eb8p+13").unwrap(),
        )],
        stocks: vec![
            // Multi-element composition {carbon:1, oxygen:2} — the CO₂ molecule
            // tracked by its carbon (from cabin_gas_state.json boundary.co2_removed).
            Stock {
                id: "boundary.co2_removed".to_string(),
                domain: "boundary".to_string(),
                quantity: "carbon".to_string(),
                unit: "mol".to_string(),
                amount: hexfloat::parse("0x1.926041893744ep+7").unwrap(),
                kind: "boundary".to_string(),
                extinction_threshold: 0.0,
                unclamped: false,
                composition: vec![
                    ("carbon".to_string(), 1.0),
                    ("oxygen".to_string(), 2.0),
                ],
            },
            // A single-element O₂ stock with a non-unit coefficient {oxygen:2}.
            Stock {
                id: "eclss.cabin_o2".to_string(),
                domain: "eclss".to_string(),
                quantity: "oxygen".to_string(),
                unit: "mol".to_string(),
                amount: hexfloat::parse("0x1.0000000000000p+3").unwrap(),
                kind: "pool".to_string(),
                extinction_threshold: 0.0,
                unclamped: false,
                composition: vec![("oxygen".to_string(), 2.0)],
            },
        ],
    };

    print!("{}", state.to_json());
}
