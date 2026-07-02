//! Emit the `crew_state` golden snapshot from a hand-built Rust `State`, printing
//! the `sim_io`-shaped JSON to stdout.
//!
//! This is the Step-0 cross-port acceptance vehicle: `tests/crossport/` runs this
//! binary and asserts `sim_io.dumps(sim_io.loads(<stdout>)) == crew_state.json`.
//! The amounts are the golden's own hex-float strings, parsed through the Rust
//! codec — so the example exercises `hexfloat::parse` and `snapshot::to_json`
//! end-to-end against a frozen artifact, not synthetic data. It hardcodes the
//! `crew_state` values (the plan's "hand-built State"); the engine that would
//! *compute* them is a later step.

use simcore::hexfloat;
use simcore::snapshot::{State, Stock};

/// Build a stock, parsing `amount` from its golden hex-float string.
fn stock(
    id: &str,
    domain: &str,
    quantity: &str,
    unit: &str,
    amount_hex: &str,
    kind: &str,
) -> Stock {
    Stock {
        id: id.to_string(),
        domain: domain.to_string(),
        quantity: quantity.to_string(),
        unit: unit.to_string(),
        amount: hexfloat::parse(amount_hex).expect("golden amount must parse"),
        kind: kind.to_string(),
        extinction_threshold: 0.0,
        unclamped: false,
        composition: vec![(quantity.to_string(), 1.0)],
    }
}

fn main() {
    // Values transcribed verbatim from tests/regression/golden/crew_state.json.
    let state = State {
        n: 168,
        rng_seed: 0,
        aux: vec![],
        stocks: vec![
            stock(
                "boundary.crew_humidity",
                "boundary",
                "water",
                "kg",
                "0x1.87e90ff972484p+3",
                "boundary",
            ),
            stock(
                "boundary.crew_o2_consumed",
                "boundary",
                "oxygen",
                "mol",
                "0x1.2e66666666677p+9",
                "boundary",
            ),
            stock(
                "boundary.exhaled_co2",
                "boundary",
                "carbon",
                "mol",
                "0x1.1efa43fe5c91fp+8",
                "boundary",
            ),
            stock(
                "boundary.fecal_waste",
                "boundary",
                "carbon",
                "mol",
                "0x1.ed844d013a90bp+3",
                "boundary",
            ),
            stock(
                "boundary.urine",
                "boundary",
                "water",
                "kg",
                "0x1.79652bd3c3604p+2",
                "boundary",
            ),
            stock(
                "crew.food_store",
                "crew",
                "carbon",
                "mol",
                "0x1.5cccccccccd10p+9",
                "pool",
            ),
            stock(
                "crew.o2_store",
                "crew",
                "oxygen",
                "mol",
                "0x1.5cccccccccd10p+10",
                "pool",
            ),
            stock(
                "crew.water_store",
                "crew",
                "water",
                "kg",
                "0x1.4ed916872b068p+5",
                "pool",
            ),
        ],
    };

    print!("{}", state.to_json());
}
