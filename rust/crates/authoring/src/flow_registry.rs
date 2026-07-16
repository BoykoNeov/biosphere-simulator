//! The author-selectable frozen-flow surface — the Rust mirror of Python
//! `authoring.flow_registry` (Phase 9, Step 4b).
//!
//! A scenario file selects frozen `Flow` types by name; this module is the explicit
//! mapping from an authoring type name → the frozen constructor + its wiring/param
//! shape, plus the named frozen param sets a `kinetics` rate reads. **Explicit, not
//! introspected** — the registry *is* the authoring contract Step 7 freezes.
//!
//! Step 4b registered the standalone **Crew** flows (the composition anchor), matching
//! the Python Step-0 surface; the post-roadmap **Tier-1** unfreeze grew both ports to
//! twelve by adding the nine standalone Power / Thermal / ECLSS flows
//! (`docs/plans/post-roadmap-flow-registry-growth.md`). Frozen param values arrive from
//! the Option-C [`domains::params`] constants (the hex-float bundle,
//! `sibling_params.txt`), **not** a re-parsed YAML — so the Rust interpreter reads no
//! param file. **Parameter packs (arbitrary `{value,unit,source}` files) are deferred in
//! the Rust port** (a Rust pack reader would re-run the frozen bounds/unit validation on
//! an arbitrary file, which no anchor exercises and Step 5's Godot payoff does not need —
//! see [`crate::interpreter`]); a `params: {pack: …}` reference is an interpret-time error.
//!
//! **This mirror must stay in lockstep with Python `authoring.flow_registry`.** A
//! Python-only registration is a broken contract, not a half-done one: the authoring
//! manifest freezes the *Python* surface, so nothing here fails until an anchor exercises
//! the missing arm (`test_crossport.py`). The three surfaces below that must move together
//! for every added type are the [`flow_type`] match, the [`build_frozen_flow`] match, and
//! the hand-maintained [`FLOW_TYPE_NAMES`] list.
//!
//! **Registered ≠ calibrated**: selecting a frozen type means the *rate law* is frozen,
//! not that its numbers are validated — every param reachable here except `crew`'s two is
//! a `TODO(cite)` placeholder. See `docs/authoring-reference.md`, "Frozen is not
//! calibrated".

use std::collections::BTreeMap;

use domains::crew::{CrewParams, FoodMetabolism, OxygenConsumption, WaterBalance};
use domains::eclss::{CO2Scrubber, Condenser, CrewMetabolism, EclssParams, O2Makeup};
use domains::params;
use domains::power::{ChargeParams, LoadDraw, SelfDischarge, SelfDischargeParams, SolarCharge};
use domains::thermal::{HeatInput, RadiatorReject, ThermalParams};
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
        // --- Tier 1: Power ---
        "power.solar_charge" => Some(FlowTypeSpec {
            wiring_fields: &["solar_source", "battery", "waste_heat"],
            param_set: Some("charge"),
        }),
        "power.load_draw" => Some(FlowTypeSpec {
            wiring_fields: &["battery", "waste_heat"],
            param_set: None,
        }),
        "power.self_discharge" => Some(FlowTypeSpec {
            wiring_fields: &["battery", "waste_heat"],
            param_set: Some("self_discharge"),
        }),
        // --- Tier 1: Thermal ---
        "thermal.heat_input" => Some(FlowTypeSpec {
            wiring_fields: &["heat_source", "node"],
            param_set: None,
        }),
        "thermal.radiator_reject" => Some(FlowTypeSpec {
            wiring_fields: &["node", "space"],
            param_set: Some("thermal"),
        }),
        // --- Tier 1: ECLSS ---
        "eclss.crew_metabolism" => Some(FlowTypeSpec {
            wiring_fields: &[
                "cabin_o2",
                "cabin_co2",
                "cabin_h2o",
                "metabolic_o2_sink",
                "metabolic_co2_source",
                "metabolic_h2o_source",
            ],
            param_set: None,
        }),
        "eclss.co2_scrubber" => Some(FlowTypeSpec {
            wiring_fields: &["cabin_co2", "co2_removed"],
            param_set: Some("eclss"),
        }),
        "eclss.condenser" => Some(FlowTypeSpec {
            wiring_fields: &["cabin_h2o", "humidity_condensate"],
            param_set: Some("eclss"),
        }),
        "eclss.o2_makeup" => Some(FlowTypeSpec {
            wiring_fields: &["o2_supply", "cabin_o2"],
            param_set: Some("eclss"),
        }),
        _ => None,
    }
}

/// The known authoring flow-type names (for the "unknown flow type" error message and
/// the Step-7 completeness surface).
///
/// Hand-maintained and **sorted** — it is not derived from [`flow_type`] (a Rust `match`
/// cannot be enumerated), so adding a type means editing three places in this file. The
/// `flow_type_names_all_resolve` test below closes the gap in one direction (every name
/// here resolves); the cross-port anchors close the other.
pub const FLOW_TYPE_NAMES: &[&str] = &[
    "crew.food_metabolism",
    "crew.oxygen_consumption",
    "crew.water_balance",
    "eclss.co2_scrubber",
    "eclss.condenser",
    "eclss.crew_metabolism",
    "eclss.o2_makeup",
    "power.load_draw",
    "power.self_discharge",
    "power.solar_charge",
    "thermal.heat_input",
    "thermal.radiator_reject",
];

/// The known frozen param-set names (`PARAM_LOADERS` keys — the sets a `kinetics`
/// rate's `param("…")` may read, and the sets a frozen flow type names).
pub const PARAM_SET_NAMES: &[&str] = &["charge", "crew", "eclss", "self_discharge", "thermal"];

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
        // --- Tier 1: Power ---
        "power.solar_charge" => Ok(Box::new(SolarCharge::new(
            id.to_string(),
            wire("solar_source")?,
            wire("battery")?,
            wire("waste_heat")?,
            charge_params(),
        ))),
        "power.load_draw" => Ok(Box::new(LoadDraw::new(
            id.to_string(),
            wire("battery")?,
            wire("waste_heat")?,
        ))),
        "power.self_discharge" => Ok(Box::new(SelfDischarge::new(
            id.to_string(),
            wire("battery")?,
            wire("waste_heat")?,
            self_discharge_params(),
        ))),
        // --- Tier 1: Thermal ---
        "thermal.heat_input" => Ok(Box::new(HeatInput::new(
            id.to_string(),
            wire("heat_source")?,
            wire("node")?,
        ))),
        "thermal.radiator_reject" => Ok(Box::new(RadiatorReject::new(
            id.to_string(),
            wire("node")?,
            wire("space")?,
            thermal_params(),
        ))),
        // --- Tier 1: ECLSS ---
        "eclss.crew_metabolism" => Ok(Box::new(CrewMetabolism::new(
            id.to_string(),
            wire("cabin_o2")?,
            wire("cabin_co2")?,
            wire("cabin_h2o")?,
            wire("metabolic_o2_sink")?,
            wire("metabolic_co2_source")?,
            wire("metabolic_h2o_source")?,
        ))),
        "eclss.co2_scrubber" => Ok(Box::new(CO2Scrubber::new(
            id.to_string(),
            wire("cabin_co2")?,
            wire("co2_removed")?,
            eclss_params(),
        ))),
        "eclss.condenser" => Ok(Box::new(Condenser::new(
            id.to_string(),
            wire("cabin_h2o")?,
            wire("humidity_condensate")?,
            eclss_params(),
        ))),
        "eclss.o2_makeup" => Ok(Box::new(O2Makeup::new(
            id.to_string(),
            wire("o2_supply")?,
            wire("cabin_o2")?,
            eclss_params(),
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

/// The frozen Power charge param (η_c) from the Option-C bundle.
fn charge_params() -> ChargeParams {
    params::charge()
}

/// The frozen Power self-discharge param (k) from the Option-C bundle.
fn self_discharge_params() -> SelfDischargeParams {
    params::self_discharge()
}

/// The frozen Thermal radiator params from the Option-C bundle.
fn thermal_params() -> ThermalParams {
    params::thermal()
}

/// The frozen ECLSS control-loop params from the Option-C bundle.
fn eclss_params() -> EclssParams {
    params::eclss()
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
        // --- Tier 1: the three sets the newly-registered flow types name. Each mirrors
        // its Python loader's dataclass field-for-field: the Python side flattens with
        // `asdict()`, so a missing key here would make an authored rate that resolves on
        // one port fail on the other.
        "charge" => {
            let p = params::charge();
            map.insert("charge_efficiency".to_string(), p.charge_efficiency);
        }
        "thermal" => {
            let p = params::thermal();
            map.insert("emissivity".to_string(), p.emissivity);
            map.insert("radiator_area".to_string(), p.radiator_area);
            map.insert("heat_capacity".to_string(), p.heat_capacity);
            map.insert("space_temperature".to_string(), p.space_temperature);
        }
        "eclss" => {
            let p = params::eclss();
            map.insert("co2_scrub_rate".to_string(), p.co2_scrub_rate);
            map.insert("condense_rate".to_string(), p.condense_rate);
            map.insert("o2_makeup_gain".to_string(), p.o2_makeup_gain);
            map.insert("o2_setpoint".to_string(), p.o2_setpoint);
        }
        _ => {
            return Err(AuthoringError::new(format!(
                "unknown param set {param_set:?} (known: {PARAM_SET_NAMES:?})"
            )));
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_type_names_all_resolve() {
        // FLOW_TYPE_NAMES is hand-maintained beside two `match`es that cannot be
        // enumerated, so it can silently drift from them. This closes one direction: every
        // advertised name resolves in `flow_type` AND builds in `build_frozen_flow`. (The
        // other direction — a match arm missing from the list — is caught by the cross-port
        // anchors, which author files by name.)
        for name in FLOW_TYPE_NAMES {
            let spec = flow_type(name).unwrap_or_else(|| panic!("{name} has no flow_type"));
            let wiring: BTreeMap<String, String> = spec
                .wiring_fields
                .iter()
                .map(|f| ((*f).to_string(), format!("stock.{f}")))
                .collect();
            build_frozen_flow(name, "test.flow", 0, &wiring)
                .unwrap_or_else(|e| panic!("{name} failed to build: {e:?}"));
        }
    }

    #[test]
    fn flow_type_names_is_sorted_and_matches_param_set_names() {
        // Sorted so the list reads as a set and diffs cleanly; every param set a flow type
        // names must be a known set (the Python `param_set in PARAM_LOADERS` mirror).
        let mut sorted = FLOW_TYPE_NAMES.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, FLOW_TYPE_NAMES.to_vec());
        let mut sets = PARAM_SET_NAMES.to_vec();
        sets.sort_unstable();
        assert_eq!(sets, PARAM_SET_NAMES.to_vec());
        for name in FLOW_TYPE_NAMES {
            if let Some(set) = flow_type(name).and_then(|s| s.param_set) {
                assert!(PARAM_SET_NAMES.contains(&set), "{name} names unknown set {set}");
            }
        }
    }

    #[test]
    fn every_param_set_flattens() {
        // Each named set must flatten to a non-empty map — the authored-`kinetics`
        // `param("…")` path. An unknown set is an error, not a silent empty map.
        for set in PARAM_SET_NAMES {
            assert!(!kinetics_param_map(set).expect("known set").is_empty(), "{set}");
        }
        assert!(kinetics_param_map("nope").is_err());
    }

    #[test]
    fn authored_radiator_reject_reads_the_frozen_thermal_params() {
        // The one sliver no other gate can see. `thermal.radiator_reject` is excluded from
        // the bit-exact cross-port RUN comparison (it is Tier-2 — `powf`), and the
        // cross-port GRAPH DUMP that covers it does not render params. So a registry arm
        // that wired the WRONG param set would reach no gate at all.
        //
        // Compares the authored flow against a directly-constructed one rather than
        // recomputing Stefan-Boltzmann here: duplicating the frozen physics in a test
        // would just be a second place to get it wrong, and the question is only "did the
        // registry hand it params::thermal()".
        use std::collections::HashMap;

        use domains::thermal::{build_thermal, ThermalScenario};
        use simcore::environment::SourceResolver;

        // node0 far from the floor: at the frozen EQUILIBRIUM_SCENARIO's node0 = 0 the
        // radiator emits exactly 0 (T = T_space), which every param set agrees on — the
        // test would pass vacuously.
        let scenario = ThermalScenario {
            node0: 1.0e9,
            heat_load_w: 3000.0,
            dt_seconds: 3600.0,
        };
        let p = params::thermal();
        let (state, _registry) = build_thermal(&p, &scenario).expect("frozen build");
        let resolver = SourceResolver::new(HashMap::new(), HashMap::new()).expect("resolver");
        let env = resolver.bind(&state, scenario.dt_seconds);

        let wiring: BTreeMap<String, String> = [
            ("node".to_string(), "thermal.node".to_string()),
            ("space".to_string(), "boundary.space".to_string()),
        ]
        .into_iter()
        .collect();
        let authored = build_frozen_flow("thermal.radiator_reject", "thermal.r", 0, &wiring)
            .expect("builds");
        let reference = RadiatorReject::new(
            "thermal.r".to_string(),
            "thermal.node".to_string(),
            "boundary.space".to_string(),
            params::thermal(),
        );

        let rejected = |f: &dyn Flow| -> f64 {
            f.evaluate(&state, &env, scenario.dt_seconds)
                .expect("evaluates")
                .legs
                .iter()
                .find(|l| l.stock == "boundary.space")
                .expect("space leg")
                .amount
        };
        let got = rejected(authored.as_ref());
        assert_eq!(got, rejected(&reference), "authored radiator must use params::thermal()");

        // Teeth: the comparison is actually sensitive to the params — a different
        // emissivity gives a different answer, so the equality above is not vacuous.
        let wrong = RadiatorReject::new(
            "thermal.r".to_string(),
            "thermal.node".to_string(),
            "boundary.space".to_string(),
            ThermalParams {
                emissivity: p.emissivity / 2.0,
                ..params::thermal()
            },
        );
        assert_ne!(got, rejected(&wrong));
        assert!(got != 0.0, "the node must be off the floor or this proves nothing");
    }
}
