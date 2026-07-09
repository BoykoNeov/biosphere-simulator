//! Serialize-only state snapshot in the `sim_io` JSON shape (schema version 3).
//!
//! This is the *emit* half of the cross-port interchange. The structs carry
//! exactly the fields that appear in a golden snapshot and **no invariant logic**:
//! validation is the engine's job (Step 2) and, on the Python side, the `State` /
//! `Stock` constructors re-fire every invariant when `sim_io.loads` reads this
//! output. The emitter therefore only owes a byte stream Python `loads` accepts.
//!
//! The shape (see `src/sim_io/snapshot.py`): a top-level object with `version`,
//! `n` (native int), `rng_seed` (a `0x`-hex string), `aux` (name → hex-float),
//! and `stocks` (a list, id-sorted). Each stock carries `id`, `domain`,
//! `quantity`, `unit`, `amount` (hex-float), `kind`, `extinction_threshold`
//! (hex-float), `unclamped`, and `composition` (quantity → hex-float coeff).
//!
//! Floats are emitted through [`crate::hexfloat::format`]; the seed through a
//! `0x`-hex string (`int(s, 0)` reads it on the Python side without the >2^53
//! precision loss a JSON number would suffer). We hand-write the JSON rather than
//! pull a serde dependency: the shape is fixed and small, and staying zero-dep
//! mirrors the Python core's stdlib-only discipline.

use crate::hexfloat;
use crate::state as engine;

/// The on-disk schema version. Must equal `sim_io.snapshot.SCHEMA_VERSION`;
/// Python `loads` rejects any other value outright.
pub const SCHEMA_VERSION: u32 = 3;

/// One stock, serialize-only. String fields hold the canonical enum *values*
/// (e.g. `quantity = "carbon"`, `kind = "pool"`) exactly as the Python golden
/// spells them — the Rust engine's typed enums arrive in a later step.
#[derive(Debug, Clone)]
pub struct Stock {
    pub id: String,
    pub domain: String,
    pub quantity: String,
    pub unit: String,
    pub amount: f64,
    pub kind: String,
    pub extinction_threshold: f64,
    pub unclamped: bool,
    /// Element composition: (quantity value, coefficient) pairs. A 1:1 stock is
    /// `[("carbon", 1.0)]`. Emitted key-sorted by quantity value.
    pub composition: Vec<(String, f64)>,
}

/// A full state snapshot, serialize-only.
#[derive(Debug, Clone)]
pub struct State {
    pub n: u64,
    pub rng_seed: u64,
    /// Non-conserved auxiliary channel: (name, value) pairs. Emitted key-sorted.
    pub aux: Vec<(String, f64)>,
    pub stocks: Vec<Stock>,
}

impl State {
    /// Serialize to JSON text in the `sim_io` shape (schema v3).
    ///
    /// Stocks are emitted id-sorted and `aux` / `composition` key-sorted, matching
    /// the Python canonical ordering — though `sim_io.loads` is order-agnostic, so
    /// this is for determinism and human diffing, not correctness. A trailing
    /// newline is appended (a well-formed file line), as `sim_io.dumps` does.
    pub fn to_json(&self) -> String {
        let mut stocks = self.stocks.clone();
        stocks.sort_by(|a, b| a.id.cmp(&b.id));

        let mut aux = self.aux.clone();
        aux.sort_by(|a, b| a.0.cmp(&b.0));

        let mut out = String::new();
        out.push_str("{\n");

        // aux
        out.push_str("  \"aux\": ");
        if aux.is_empty() {
            out.push_str("{}");
        } else {
            out.push_str("{\n");
            for (i, (name, val)) in aux.iter().enumerate() {
                out.push_str("    ");
                push_json_string(&mut out, name);
                out.push_str(": ");
                push_json_string(&mut out, &hexfloat::format(*val));
                out.push_str(if i + 1 < aux.len() { ",\n" } else { "\n" });
            }
            out.push_str("  }");
        }
        out.push_str(",\n");

        // n
        out.push_str(&format!("  \"n\": {},\n", self.n));

        // rng_seed as a 0x-hex string (matches Python hex()).
        out.push_str("  \"rng_seed\": ");
        push_json_string(&mut out, &format!("0x{:x}", self.rng_seed));
        out.push_str(",\n");

        // stocks (id-sorted list)
        out.push_str("  \"stocks\": [\n");
        for (i, stock) in stocks.iter().enumerate() {
            push_stock(&mut out, stock);
            out.push_str(if i + 1 < stocks.len() { ",\n" } else { "\n" });
        }
        out.push_str("  ],\n");

        // version
        out.push_str(&format!("  \"version\": {SCHEMA_VERSION}\n"));

        out.push_str("}\n");
        out
    }
}

/// Project a computed engine [`engine::State`] into this serialize-only snapshot —
/// the bridge Step 3 needs to emit a *run's* result (Step 0's examples hand-built
/// the snapshot directly). The engine's typed `Quantity`/`StockKind` enums become
/// their lowercase canonical values (`Quantity::value` / `StockKind::value`), exactly
/// as the Python golden spells them; every float is carried verbatim (the hex-float
/// formatting happens in [`State::to_json`]). No invariant logic — the engine already
/// validated the state, and Python's constructors re-fire on load.
pub fn from_engine(state: &engine::State) -> State {
    let stocks = state
        .stocks
        .values()
        .map(|s| Stock {
            id: s.id.clone(),
            domain: s.domain.clone(),
            quantity: s.quantity.value().to_string(),
            unit: s.unit.clone(),
            amount: s.amount,
            kind: s.kind.value().to_string(),
            extinction_threshold: s.extinction_threshold,
            unclamped: s.unclamped,
            composition: s
                .composition
                .iter()
                .map(|(q, coeff)| (q.value().to_string(), *coeff))
                .collect(),
        })
        .collect();
    let aux = state.aux.iter().map(|(k, v)| (k.clone(), *v)).collect();
    State {
        n: state.n,
        rng_seed: state.rng_seed,
        aux,
        stocks,
    }
}

/// Emit one stock object (indented two levels, field keys sorted alphabetically
/// to mirror the Python `sort_keys=True` layout).
fn push_stock(out: &mut String, s: &Stock) {
    out.push_str("    {\n");
    // Field order: amount, composition, domain, extinction_threshold, id, kind,
    // quantity, unclamped, unit — alphabetical, matching json.dumps(sort_keys=True).
    out.push_str("      \"amount\": ");
    push_json_string(out, &hexfloat::format(s.amount));
    out.push_str(",\n");

    out.push_str("      \"composition\": ");
    let mut comp = s.composition.clone();
    comp.sort_by(|a, b| a.0.cmp(&b.0));
    if comp.is_empty() {
        out.push_str("{}");
    } else {
        out.push_str("{\n");
        for (i, (q, coeff)) in comp.iter().enumerate() {
            out.push_str("        ");
            push_json_string(out, q);
            out.push_str(": ");
            push_json_string(out, &hexfloat::format(*coeff));
            out.push_str(if i + 1 < comp.len() { ",\n" } else { "\n" });
        }
        out.push_str("      }");
    }
    out.push_str(",\n");

    out.push_str("      \"domain\": ");
    push_json_string(out, &s.domain);
    out.push_str(",\n");

    out.push_str("      \"extinction_threshold\": ");
    push_json_string(out, &hexfloat::format(s.extinction_threshold));
    out.push_str(",\n");

    out.push_str("      \"id\": ");
    push_json_string(out, &s.id);
    out.push_str(",\n");

    out.push_str("      \"kind\": ");
    push_json_string(out, &s.kind);
    out.push_str(",\n");

    out.push_str("      \"quantity\": ");
    push_json_string(out, &s.quantity);
    out.push_str(",\n");

    out.push_str(&format!("      \"unclamped\": {},\n", s.unclamped));

    out.push_str("      \"unit\": ");
    push_json_string(out, &s.unit);
    out.push('\n');

    out.push_str("    }");
}

/// Append a JSON string literal, escaping the characters JSON requires. Our values
/// (dotted ids, unit labels, hex strings) contain none of these in practice, but
/// escaping defensively keeps the emitter honest for any future field.
fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_engine_projects_typed_state() {
        use crate::quantities::{Quantity, StockKind};
        use crate::state::{State as EngineState, Stock as EngineStock};
        use std::collections::BTreeMap;

        // A gas-phase composition stock + a non-empty aux + a non-zero seed exercise the
        // enum→value projection and every branch to_json cares about.
        let stock = EngineStock::new(
            "eclss.cabin_co2".to_string(),
            "eclss".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            3.0,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
        )
        .unwrap();
        let engine_state = EngineState::new(
            5,
            BTreeMap::from([(stock.id.clone(), stock)]),
            0xDEAD,
            BTreeMap::from([("thermal_time".to_string(), 1.5)]),
        )
        .unwrap();

        let snap = from_engine(&engine_state);
        assert_eq!(snap.n, 5);
        assert_eq!(snap.rng_seed, 0xDEAD);
        assert_eq!(snap.aux, vec![("thermal_time".to_string(), 1.5)]);
        assert_eq!(snap.stocks.len(), 1);
        let s = &snap.stocks[0];
        assert_eq!(s.quantity, "carbon"); // lowercase enum value
        assert_eq!(s.kind, "pool");
        assert_eq!(
            s.composition,
            vec![("carbon".to_string(), 1.0), ("oxygen".to_string(), 2.0)]
        );
        // The projection is a valid snapshot the emitter serializes without panic.
        assert!(snap.to_json().contains("\"quantity\": \"carbon\""));
    }

    #[test]
    fn emits_parseable_shape() {
        let state = State {
            n: 168,
            rng_seed: 0,
            aux: vec![],
            stocks: vec![Stock {
                id: "crew.food_store".to_string(),
                domain: "crew".to_string(),
                quantity: "carbon".to_string(),
                unit: "mol".to_string(),
                amount: 1.5,
                kind: "pool".to_string(),
                extinction_threshold: 0.0,
                unclamped: false,
                composition: vec![("carbon".to_string(), 1.0)],
            }],
        };
        let json = state.to_json();
        // Structural smoke checks; the authoritative round-trip is the Python-side
        // `sim_io.loads` test under tests/crossport/.
        assert!(json.starts_with("{\n"));
        assert!(json.ends_with("}\n"));
        assert!(json.contains("\"version\": 3"));
        assert!(json.contains("\"rng_seed\": \"0x0\""));
        assert!(json.contains("\"n\": 168"));
        assert!(json.contains(&hexfloat::format(1.5)));
    }
}
