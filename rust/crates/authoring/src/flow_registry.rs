//! The author-selectable frozen-flow surface — the Rust mirror of Python
//! `authoring.flow_registry` (Phase 9, Step 4b).
//!
//! A scenario file selects frozen `Flow` types by name; this module is the explicit
//! mapping from an authoring type name → the frozen constructor + its wiring/param
//! shape, plus the named frozen param sets a `kinetics` rate reads. **Explicit, not
//! introspected** — the registry *is* the authoring contract Step 7 freezes.
//!
//! Step 4b registers the standalone **Crew** flows (the composition anchor), matching
//! the Python Step-0 surface. Frozen param values arrive from the Option-C
//! [`domains::params`] constants (the hex-float bundle, `sibling_params.txt`), **not**
//! a re-parsed YAML — so the Rust interpreter reads no param file. **Parameter packs
//! (arbitrary `{value,unit,source}` files) are deferred in the Rust port** (a Rust
//! pack reader would re-run the frozen bounds/unit validation on an arbitrary file,
//! which no anchor exercises and Step 5's Godot payoff does not need — see
//! [`crate::interpreter`]); a `params: {pack: …}` reference is an interpret-time error.

use std::collections::BTreeMap;

use domains::crew::{CrewParams, FoodMetabolism, OxygenConsumption, WaterBalance};
use domains::params;
use simcore::flow::Flow;

use crate::errors::AuthoringError;

/// How one authoring flow-type name lowers to a frozen `Flow` constructor: the exact
/// set of wiring keyword fields (order-significant — it is the constructor's positional
/// order) and the frozen param set the flow's constructor needs (or `None`).
#[derive(Debug, Clone, Copy)]
pub struct FlowTypeSpec {
    /// The wiring field names, in constructor-argument order.
    pub wiring_fields: &'static [&'static str],
    /// The frozen param-set name this flow's constructor consumes, if any.
    pub param_set: Option<&'static str>,
}

/// Look up a frozen flow type by its authoring name (the `FLOW_TYPES` dict analogue).
pub fn flow_type(name: &str) -> Option<FlowTypeSpec> {
    match name {
        "crew.oxygen_consumption" => Some(FlowTypeSpec {
            wiring_fields: &["o2_store", "o2_consumed"],
            param_set: None,
        }),
        "crew.food_metabolism" => Some(FlowTypeSpec {
            wiring_fields: &["food_store", "exhaled_co2", "fecal_waste"],
            param_set: Some("crew"),
        }),
        "crew.water_balance" => Some(FlowTypeSpec {
            wiring_fields: &["water_store", "crew_humidity", "urine"],
            param_set: Some("crew"),
        }),
        _ => None,
    }
}

/// The known authoring flow-type names (for the "unknown flow type" error message and
/// the Step-7 completeness surface).
pub const FLOW_TYPE_NAMES: &[&str] = &[
    "crew.food_metabolism",
    "crew.oxygen_consumption",
    "crew.water_balance",
];

/// The known frozen param-set names (`PARAM_LOADERS` keys — the sets a `kinetics`
/// rate's `param("…")` may read, and the sets a frozen flow type names).
pub const PARAM_SET_NAMES: &[&str] = &["crew", "self_discharge"];

/// Construct a frozen-`type` flow from its wiring (the interpreter has already checked
/// the wiring keys match `wiring_fields` exactly). Loads the flow's frozen params from
/// the Option-C constants where needed. An unknown type is an [`AuthoringError`].
pub fn build_frozen_flow(
    type_name: &str,
    id: &str,
    _priority: i64,
    wiring: &BTreeMap<String, String>,
) -> Result<Box<dyn Flow>, AuthoringError> {
    // Helper: fetch a wired stock id by field name (guaranteed present by the caller's
    // exact-match wiring check, but surface a clear error rather than panic).
    let wire = |field: &str| -> Result<String, AuthoringError> {
        wiring
            .get(field)
            .cloned()
            .ok_or_else(|| AuthoringError::new(format!("flow {id:?}: missing wiring {field:?}")))
    };
    match type_name {
        "crew.oxygen_consumption" => Ok(Box::new(OxygenConsumption::new(
            id.to_string(),
            wire("o2_store")?,
            wire("o2_consumed")?,
        ))),
        "crew.food_metabolism" => Ok(Box::new(FoodMetabolism::new(
            id.to_string(),
            wire("food_store")?,
            wire("exhaled_co2")?,
            wire("fecal_waste")?,
            crew_params(),
        ))),
        "crew.water_balance" => Ok(Box::new(WaterBalance::new(
            id.to_string(),
            wire("water_store")?,
            wire("crew_humidity")?,
            wire("urine")?,
            crew_params(),
        ))),
        _ => Err(AuthoringError::new(format!(
            "flow {id:?}: unknown flow type {type_name:?} (known: {FLOW_TYPE_NAMES:?})"
        ))),
    }
}

/// The frozen Crew params from the Option-C bundle (`domains::params::crew()`).
fn crew_params() -> CrewParams {
    params::crew()
}

/// Flatten a named frozen param set to the `name -> f64` map an authored-`kinetics`
/// rate's `param("…")` reads (the Python `_kinetics_param_map` analogue). Values come
/// from the Option-C constants, so an authored rate constant *is* the frozen one —
/// bit-identical, passing the frozen bounds/unit guards. An unknown set (or a pack) is
/// an [`AuthoringError`].
pub fn kinetics_param_map(param_set: &str) -> Result<BTreeMap<String, f64>, AuthoringError> {
    let mut map = BTreeMap::new();
    match param_set {
        "crew" => {
            let p = params::crew();
            map.insert(
                "respired_carbon_fraction".to_string(),
                p.respired_carbon_fraction,
            );
            map.insert(
                "insensible_water_fraction".to_string(),
                p.insensible_water_fraction,
            );
        }
        "self_discharge" => {
            let p = params::self_discharge();
            map.insert("self_discharge_rate".to_string(), p.self_discharge_rate);
        }
        _ => {
            return Err(AuthoringError::new(format!(
                "unknown param set {param_set:?} (known: {PARAM_SET_NAMES:?})"
            )));
        }
    }
    Ok(map)
}
