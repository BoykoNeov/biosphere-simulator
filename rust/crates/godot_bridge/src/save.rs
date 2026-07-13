//! Phase-8 (P8.7) save / load — the versioned save-record wrapper.
//!
//! A save is `(recipe, State)` and **nothing else** (the determinism corollary: the
//! core RNG is `(seed, key, n)`-keyed, so a resume needs only the recipe to rebuild the
//! registry and the exact `State` to restore stocks/aux/`n`/seed — see
//! [`station::session::SimSession::load_state`]).
//!
//! # The wrapper *embeds*, it does not *extend*, the frozen snapshot (advisor)
//!
//! The `sim_io` state snapshot is frozen at schema v3 and is byte-identical across the
//! Python/Rust ports. This wrapper must not add keys into it (that would bump the schema
//! and break the frozen goldens). Instead it is a **separate** format that nests the
//! state snapshot verbatim under a `"state"` key and adds its own [`SAVE_VERSION`],
//! composing the two codecs rather than merging them:
//!
//! ```text
//! { "save_version": 1, "recipe": { "scenario": "station" }, "state": { <v3 snapshot> } }
//! ```
//!
//! Its own version field gets the same fail-loud-at-parse discipline the snapshot uses
//! (a serialization format is the one place forward-compat cannot be retrofitted).
//!
//! # Scope
//!
//! The recipe is a fixed-palette [`Recipe::Named`] scenario id or a [`Recipe::Composed`]
//! component list — the two build paths P8.6 established. **Perturbed sessions are not
//! saveable** (deferred loudly, exactly as two-rate flow inspection was): a perturbation
//! is a build-time window that would need its own recipe surface; the bridge simply
//! reports save unavailable for one.

use simcore::error::SimError;
use simcore::json::{self, JsonValue};
use simcore::snapshot::{from_engine, from_json_value};
use simcore::state::State;

/// The save-record schema version. Bumped only if the *wrapper* shape changes (the
/// embedded state snapshot carries its own independent `version`). An unknown value is
/// rejected at parse — no migration machinery until a second version exists.
pub const SAVE_VERSION: u32 = 1;

/// How to rebuild the registry on load — the fixed-palette recipe (confirmed decision #1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recipe {
    /// A pre-built palette scenario id (`"station"`, `"cabin_gas"`, `"greenhouse"`,
    /// `"sealed"`) — rebuilt via the bridge's `build_session`.
    Named(String),
    /// A composed station: the chosen component ids (`"power_plant"`, `"radiator"`,
    /// `"self_discharge"`) — rebuilt via the bridge's `build_composed_session`.
    Composed(Vec<String>),
}

impl Recipe {
    /// The recipe as a JSON object (embedded under `"recipe"` in a save record).
    fn to_json(&self) -> String {
        match self {
            Recipe::Named(id) => format!("{{\"scenario\": {}}}", json_string(id)),
            Recipe::Composed(ids) => {
                let items: Vec<String> = ids.iter().map(|s| json_string(s)).collect();
                format!("{{\"components\": [{}]}}", items.join(", "))
            }
        }
    }

    /// Parse a recipe from its JSON object (`{"scenario": ...}` or `{"components": [...]}`).
    fn from_json_value(v: &JsonValue) -> Result<Recipe, SimError> {
        if let Some(id) = v.get("scenario").and_then(JsonValue::as_str) {
            return Ok(Recipe::Named(id.to_string()));
        }
        if let Some(arr) = v.get("components").and_then(JsonValue::as_array) {
            let ids = arr
                .iter()
                .map(|item| {
                    item.as_str().map(str::to_string).ok_or_else(|| {
                        SimError::Validation("recipe component id is not a string".to_string())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Recipe::Composed(ids));
        }
        Err(SimError::Validation(
            "save recipe needs a \"scenario\" (named) or \"components\" (composed) field"
                .to_string(),
        ))
    }
}

/// Serialize a save record: the [`Recipe`] plus the current [`State`] as an embedded v3
/// snapshot (byte-verbatim from [`from_engine`] → `to_json`). The wrapper is compact; the
/// nested snapshot keeps its canonical multi-line form (the reader is whitespace-tolerant).
pub fn save_record_json(recipe: &Recipe, state: &State) -> String {
    let snapshot = from_engine(state).to_json();
    format!(
        "{{\n  \"save_version\": {SAVE_VERSION},\n  \"recipe\": {},\n  \"state\": {}\n}}\n",
        recipe.to_json(),
        snapshot.trim_end(),
    )
}

/// Parse a save record back into `(recipe, State)`. Rejects an unknown/missing
/// `save_version` (fail-loud), then reconstructs the state through the frozen snapshot
/// loader ([`from_json_value`]), so every core invariant re-fires on load.
pub fn parse_save_record(text: &str) -> Result<(Recipe, State), SimError> {
    let root = json::parse(text).map_err(|e| SimError::Validation(e.to_string()))?;
    let version = root.get("save_version").and_then(JsonValue::as_i64);
    if version != Some(SAVE_VERSION as i64) {
        return Err(SimError::Validation(format!(
            "unsupported save_version {:?}; this build reads version {SAVE_VERSION} only",
            root.get("save_version")
        )));
    }
    let recipe = root
        .get("recipe")
        .ok_or_else(|| SimError::Validation("save record missing 'recipe'".to_string()))?;
    let recipe = Recipe::from_json_value(recipe)?;
    let state_value = root
        .get("state")
        .ok_or_else(|| SimError::Validation("save record missing 'state'".to_string()))?;
    let state = from_json_value(state_value)?;
    Ok((recipe, state))
}

/// Minimal JSON string literal (recipe ids are safe ASCII, but escape defensively so the
/// record stays well-formed for any future id).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use simcore::quantities::{Quantity, StockKind};
    use simcore::state::Stock;

    fn tiny_state() -> State {
        let stock = Stock::new(
            "power.battery".to_string(),
            "power".to_string(),
            Quantity::Energy,
            "J".to_string(),
            std::f64::consts::PI,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap();
        State::new(
            12,
            BTreeMap::from([(stock.id.clone(), stock)]),
            0xABCD,
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn named_recipe_round_trips_with_bit_exact_state() {
        let state = tiny_state();
        let text = save_record_json(&Recipe::Named("station".to_string()), &state);
        let (recipe, back) = parse_save_record(&text).unwrap();
        assert_eq!(recipe, Recipe::Named("station".to_string()));
        // Bit-exact: re-emitting the reloaded state matches the embedded snapshot.
        assert_eq!(from_engine(&back).to_json(), from_engine(&state).to_json());
        assert_eq!(back.n, 12);
        assert_eq!(back.rng_seed, 0xABCD);
    }

    #[test]
    fn composed_recipe_round_trips() {
        let state = tiny_state();
        let recipe = Recipe::Composed(vec!["power_plant".to_string(), "radiator".to_string()]);
        let text = save_record_json(&recipe, &state);
        let (back_recipe, _state) = parse_save_record(&text).unwrap();
        assert_eq!(back_recipe, recipe);
    }

    #[test]
    fn embedded_snapshot_is_the_verbatim_v3_snapshot() {
        // The wrapper embeds the frozen snapshot; it must contain its unmodified v3 form,
        // never an extended one (advisor: embed, don't extend).
        let state = tiny_state();
        let text = save_record_json(&Recipe::Named("station".to_string()), &state);
        assert!(text.contains("\"version\": 3"), "embeds the v3 snapshot");
        assert!(text.contains("\"save_version\": 1"), "own wrapper version");
        // The embedded snapshot parses on its own as a v3 State (composition of codecs).
        let root = json::parse(&text).unwrap();
        let state_v = root.get("state").unwrap();
        assert!(from_json_value(state_v).is_ok());
    }

    #[test]
    fn rejects_unknown_save_version_and_bad_recipe() {
        let bad_ver = r#"{"save_version": 99, "recipe": {"scenario":"x"}, "state": {}}"#;
        assert!(matches!(
            parse_save_record(bad_ver),
            Err(SimError::Validation(_))
        ));
        // Missing both recipe discriminators.
        let bad_recipe = format!(
            "{{\"save_version\": {SAVE_VERSION}, \"recipe\": {{}}, \"state\": {}}}",
            from_engine(&tiny_state()).to_json().trim_end()
        );
        assert!(matches!(
            parse_save_record(&bad_recipe),
            Err(SimError::Validation(_))
        ));
    }
}
