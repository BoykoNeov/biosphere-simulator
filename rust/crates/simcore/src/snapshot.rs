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

use std::collections::BTreeMap;

use crate::error::SimError;
use crate::hexfloat;
use crate::json::{self, JsonValue};
use crate::quantities::{Quantity, StockKind};
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

/// Parse `sim_io` snapshot JSON text into a computed engine [`engine::State`] — the
/// **load** half of the codec (P8.7, work item #3), the inverse of
/// [`from_engine`] → [`State::to_json`]. Convenience wrapper over [`json::parse`] +
/// [`from_json_value`]; the save-record wrapper (`godot_bridge`) parses its outer
/// object once and calls [`from_json_value`] on the embedded `state` sub-value.
pub fn from_json(text: &str) -> Result<engine::State, SimError> {
    let value = json::parse(text).map_err(|e| SimError::Validation(e.to_string()))?;
    from_json_value(&value)
}

/// Reconstruct an engine [`engine::State`] from a parsed snapshot [`JsonValue`].
///
/// Mirrors Python `sim_io.snapshot.state_from_dict`: an unknown/missing schema
/// `version` is rejected at parse (fail-loud, no migration machinery), and every
/// stock/state routes through [`engine::Stock::new`] / [`engine::State::new`] so the
/// core invariants re-fire on load — a tampered save (non-finite amount, an
/// unclamped non-BOUNDARY stock, a key/id mismatch) fails loudly here rather than
/// producing a malformed state. Floats are read through the exact hex-float
/// [`hexfloat::parse`]; `rng_seed` from its `0x`-hex (or decimal) string, so a
/// >2^53 seed survives bit-for-bit.
pub fn from_json_value(value: &JsonValue) -> Result<engine::State, SimError> {
    let version = value.get("version").and_then(JsonValue::as_i64);
    if version != Some(SCHEMA_VERSION as i64) {
        return Err(SimError::Validation(format!(
            "unsupported snapshot schema version {:?}; this build reads version \
             {SCHEMA_VERSION} only",
            value.get("version")
        )));
    }

    let n = value
        .get("n")
        .and_then(JsonValue::as_u64)
        .ok_or_else(|| SimError::Validation("snapshot missing integer 'n'".to_string()))?;

    let seed_str = value
        .get("rng_seed")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| SimError::Validation("snapshot missing string 'rng_seed'".to_string()))?;
    let rng_seed = parse_seed(seed_str)?;

    let mut aux = BTreeMap::new();
    let aux_obj = value
        .get("aux")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| SimError::Validation("snapshot missing object 'aux'".to_string()))?;
    for (name, hexv) in aux_obj {
        let h = hexv.as_str().ok_or_else(|| {
            SimError::Validation(format!("aux[{name:?}] is not a hex-float string"))
        })?;
        aux.insert(name.clone(), parse_hex(h)?);
    }

    let stocks_arr = value
        .get("stocks")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| SimError::Validation("snapshot missing array 'stocks'".to_string()))?;
    let mut stocks = BTreeMap::new();
    for s in stocks_arr {
        let stock = stock_from_json(s)?;
        stocks.insert(stock.id.clone(), stock);
    }

    engine::State::new(n, stocks, rng_seed, aux)
}

/// One stock object → a validated engine [`engine::Stock`] (through the constructor).
fn stock_from_json(v: &JsonValue) -> Result<engine::Stock, SimError> {
    let get_str = |key: &str| -> Result<&str, SimError> {
        v.get(key)
            .and_then(JsonValue::as_str)
            .ok_or_else(|| SimError::Validation(format!("stock missing string field {key:?}")))
    };

    let composition_obj = v
        .get("composition")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| SimError::Validation("stock missing object 'composition'".to_string()))?;
    let mut composition = BTreeMap::new();
    for (q, coeffv) in composition_obj {
        let coeff = coeffv.as_str().ok_or_else(|| {
            SimError::Validation(format!("composition[{q:?}] is not a hex-float string"))
        })?;
        composition.insert(Quantity::from_value(q)?, parse_hex(coeff)?);
    }

    let unclamped = v
        .get("unclamped")
        .and_then(JsonValue::as_bool)
        .ok_or_else(|| SimError::Validation("stock missing bool 'unclamped'".to_string()))?;

    engine::Stock::new(
        get_str("id")?.to_string(),
        get_str("domain")?.to_string(),
        Quantity::from_value(get_str("quantity")?)?,
        get_str("unit")?.to_string(),
        parse_hex(get_str("amount")?)?,
        StockKind::from_value(get_str("kind")?)?,
        parse_hex(get_str("extinction_threshold")?)?,
        unclamped,
        composition,
    )
}

/// A hex-float string → `f64`, mapping the codec's [`hexfloat::ParseError`] into a
/// [`SimError::Validation`] (the fail-loud discipline: a tampered value never coerces).
fn parse_hex(s: &str) -> Result<f64, SimError> {
    hexfloat::parse(s).map_err(|e| SimError::Validation(e.to_string()))
}

/// Parse the `rng_seed` string — a `0x`-hex (as [`State::to_json`] emits) or a plain
/// decimal, mirroring Python's `int(s, 0)`. Stored as a string (not a JSON number)
/// so a full 64-bit seed survives without the >2^53 f64 precision loss.
fn parse_seed(s: &str) -> Result<u64, SimError> {
    let s = s.trim();
    let parsed = match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16),
        None => s.parse::<u64>(),
    };
    parsed.map_err(|_| SimError::Validation(format!("invalid rng_seed {s:?}")))
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

    /// A representative engine state (nasty floats, POPULATION with extinction, an
    /// unclamped BOUNDARY source, a multi-key composition, aux, a >2^53 seed) round-
    /// trips through `to_json` → `from_json` bit-for-bit. Bit-exactness is asserted by
    /// re-emitting: the hex-float codec distinguishes `-0.0` from `0.0` where `==`
    /// would not, so equal `to_json` bytes ⇒ every bit survived.
    #[test]
    fn from_json_round_trips_bit_for_bit() {
        use crate::quantities::{Quantity, StockKind};
        use crate::state::{State as EngineState, Stock as EngineStock};

        let stocks = BTreeMap::from([
            (
                "bio.atmo_c".to_string(),
                EngineStock::new(
                    "bio.atmo_c".to_string(),
                    "bio".to_string(),
                    Quantity::Carbon,
                    "mol".to_string(),
                    std::f64::consts::PI,
                    StockKind::Pool,
                    0.0,
                    false,
                    BTreeMap::new(),
                )
                .unwrap(),
            ),
            (
                "bio.plant_c".to_string(),
                EngineStock::new(
                    "bio.plant_c".to_string(),
                    "bio".to_string(),
                    Quantity::Carbon,
                    "mol".to_string(),
                    0.1,
                    StockKind::Population,
                    1e-6,
                    false,
                    BTreeMap::new(),
                )
                .unwrap(),
            ),
            (
                "bio.co2".to_string(),
                EngineStock::new(
                    "bio.co2".to_string(),
                    "bio".to_string(),
                    Quantity::Carbon,
                    "mol".to_string(),
                    -0.0, // signed zero survives via hex-float
                    StockKind::Pool,
                    0.0,
                    false,
                    BTreeMap::from([(Quantity::Carbon, 1.0), (Quantity::Oxygen, 2.0)]),
                )
                .unwrap(),
            ),
            (
                "boundary.solar".to_string(),
                EngineStock::new(
                    "boundary.solar".to_string(),
                    "boundary".to_string(),
                    Quantity::Energy,
                    "J".to_string(),
                    5e-324, // smallest positive subnormal
                    StockKind::Boundary,
                    0.0,
                    true, // unclamped source
                    BTreeMap::new(),
                )
                .unwrap(),
            ),
        ]);
        let aux = BTreeMap::from([
            ("thermal_time".to_string(), std::f64::consts::PI),
            ("neg_zero".to_string(), -0.0),
        ]);
        let state = EngineState::new(7, stocks, 0x0123_4567_89AB_CDEF, aux).unwrap();

        let text = from_engine(&state).to_json();
        let back = from_json(&text).expect("from_json must accept our own to_json");
        assert_eq!(back.n, 7);
        assert_eq!(back.rng_seed, 0x0123_4567_89AB_CDEF);
        // Re-emit and byte-compare: equal hex-float bytes ⇒ bit-exact (incl. -0.0).
        assert_eq!(from_engine(&back).to_json(), text);
    }

    /// The cross-port proof: the Rust loader reads the **Python-generated** frozen
    /// `state_snapshot.json` golden and reconstructs the exact bits (parsed-value
    /// equality — no dependence on emitter formatting). The golden spans pi, `0.1`,
    /// a subnormal, signed `-0.0`, a >2^53 seed, POPULATION with extinction, and an
    /// unclamped ENERGY source.
    #[test]
    fn loads_the_python_golden_bit_exact() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../tests/regression/golden/state_snapshot.json"
        );
        let text = std::fs::read_to_string(path).expect("read state_snapshot.json golden");
        let state = from_json(&text).expect("Rust loads the Python golden");

        assert_eq!(state.n, 42);
        assert_eq!(state.rng_seed, 0x0123_4567_89AB_CDEF); // >2^53, survives exactly
        // Exact bits via .hex()-equivalent (compare raw f64 bits — distinguishes -0.0).
        assert_eq!(
            state.stocks["bio.atmo_c"].amount.to_bits(),
            std::f64::consts::PI.to_bits()
        );
        assert_eq!(state.stocks["bio.plant_c"].extinction_threshold, 1e-6);
        assert!(matches!(
            state.stocks["bio.plant_c"].kind,
            crate::quantities::StockKind::Population
        ));
        assert_eq!(state.stocks["bio.water"].amount, 5e-324);
        // The loss-sink's -0.0 keeps its sign bit (value equality would miss it).
        assert_eq!(state.stocks["boundary.loss.carbon"].amount.to_bits(), (-0.0_f64).to_bits());
        assert!(state.stocks["boundary.solar"].unclamped);
        assert_eq!(state.aux["neg_zero"].to_bits(), (-0.0_f64).to_bits());

        // Bonus (byte-identity holds on this platform — Rust to_json == Python dumps):
        // re-emitting the loaded golden reproduces the file byte-for-byte.
        assert_eq!(from_engine(&state).to_json(), text);
    }

    #[test]
    fn from_json_rejects_bad_version_and_tampering() {
        // Unknown schema version → loud reject (no migration machinery).
        let v2 = r#"{"aux":{},"n":0,"rng_seed":"0x0","stocks":[],"version":2}"#;
        assert!(matches!(from_json(v2), Err(SimError::Validation(_))));

        // A non-finite amount is caught by Stock::new (invariants re-fire on load).
        let nan = r#"{"aux":{},"n":0,"rng_seed":"0x0","version":3,"stocks":[
            {"amount":"nan","composition":{"carbon":"0x1.0p+0"},"domain":"d",
             "extinction_threshold":"0x0.0p+0","id":"x","kind":"pool",
             "quantity":"carbon","unclamped":false,"unit":"mol"}]}"#;
        assert!(matches!(from_json(nan), Err(SimError::Validation(_))));

        // An unclamped non-BOUNDARY stock is rejected by the constructor guard.
        let unclamped_pool = r#"{"aux":{},"n":0,"rng_seed":"0x0","version":3,"stocks":[
            {"amount":"0x1.0p+0","composition":{"carbon":"0x1.0p+0"},"domain":"d",
             "extinction_threshold":"0x0.0p+0","id":"x","kind":"pool",
             "quantity":"carbon","unclamped":true,"unit":"mol"}]}"#;
        assert!(matches!(from_json(unclamped_pool), Err(SimError::Validation(_))));

        // An unknown quantity value fails at from_value.
        let bad_q = r#"{"aux":{},"n":0,"rng_seed":"0x0","version":3,"stocks":[
            {"amount":"0x1.0p+0","composition":{"carbon":"0x1.0p+0"},"domain":"d",
             "extinction_threshold":"0x0.0p+0","id":"x","kind":"pool",
             "quantity":"unobtainium","unclamped":false,"unit":"mol"}]}"#;
        assert!(matches!(from_json(bad_q), Err(SimError::Validation(_))));
    }

    #[test]
    fn parse_seed_accepts_hex_and_decimal() {
        assert_eq!(super::parse_seed("0x0").unwrap(), 0);
        assert_eq!(super::parse_seed("0x123456789abcdef").unwrap(), 0x0123_4567_89AB_CDEF);
        assert_eq!(super::parse_seed("42").unwrap(), 42);
        assert!(super::parse_seed("0xnope").is_err());
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
