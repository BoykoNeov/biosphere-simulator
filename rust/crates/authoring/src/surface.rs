//! The frozen **author-facing platform surface**, enumerated — slice 8 of the reference
//! flip (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! `docs/authoring-reference.manifest.json` freezes what a mod author writes against: the
//! bounded kinetics grammar + the VM's node/op set, the scenario file schema, the
//! author-selectable flow-type registry, and the named param loaders. Until slice 8 that
//! manifest was generated from the **Python** package and this crate was merely gated
//! against it by the parse/traj vectors and the crossport anchors. Since slice 8 the
//! manifest's platform half is generated from **here**, and Python's introspective
//! derivation has become a conformance check on the checker.
//!
//! ## ⚠ This surface is NOT the same kind of census as the other two manifests'
//!
//! The biosphere and station manifests re-anchored (slices 6 and 7) to a **runtime
//! enumeration of a built registry** — `Flow::type_name()` over the flows a canonical
//! scenario actually wires. Nothing here has that shape, because the frozen thing is the
//! *platform*, not a wiring: there is no "build the canonical authoring scenario and walk
//! it". So the argument that made those two slices cheap — *the gates never introspected
//! the namespace, they read `registry.flows`, and Rust does that identically* — does not
//! transfer to this contract at all.
//!
//! What Python has here and Rust cannot have is **language introspection**:
//! `typing.get_args(Expr)` enumerates the node union, `vars(authoring.schema)` finds every
//! pydantic model, `model_fields` reads each one's fields, and `FLOW_TYPES` is a dict.
//! Rust's equivalents are an `enum`, a `match` and a set of `const`s, none of which is
//! enumerable at runtime. Each function below therefore says, in its own doc comment,
//! exactly which half of its value is derived and which half is a hand-maintained roster —
//! and the manifest's `_authority` block carries the same statement per key. **Do not read
//! "side: rust" on this contract as meaning what it means on the other two.**
//!
//! The mitigation, where one exists, is that the roster is **load-bearing** rather than
//! descriptive: [`ref_keywords`] and [`schema_fields`] return the very tables the parser
//! rejects against, so a name dropped from one of them changes what the platform *accepts*
//! and reddens the parse vectors and the anchors. Where even that is unavailable
//! ([`expr_nodes`], [`binary_ops`], [`flow_types`]'s roster, [`integrator_names`]) the
//! compiler is used as far as it reaches — a new `Expr` variant breaks
//! [`expr_node_name`]'s match, a new flow type breaks nothing until
//! `flow_type_names_all_resolve` runs — and the residue is written down rather than
//! implied.

use std::collections::{BTreeMap, BTreeSet};

use simcore::expr::{BinaryOp, Expr};

pub use crate::expr_parser::{REF_KEYWORDS, STEP_TOKEN};
pub use crate::flow_registry::{FLOW_TYPE_NAMES, PARAM_SET_NAMES};
pub use crate::interpreter::RATE_CLASSES;
pub use crate::run::INTEGRATOR_NAMES;

use crate::flow_registry::{build_frozen_flow, flow_type};

/// The frozen AST node name for one expression node — the Python class name.
///
/// ⚠ **The mapping is compiler-forced; the roster is not.** Adding a variant to
/// `simcore::expr::Expr` makes this `match` non-exhaustive and the crate stops compiling,
/// so no node can enter the grammar without an author naming it here. What that does *not*
/// do is put the new name in [`expr_nodes`]'s output — that comes from [`SAMPLE_NODES`],
/// which is hand-maintained. Python's side of this is `typing.get_args(Expr)`, which is
/// genuinely derived; this is the closest a Rust enum gets.
///
/// The names are the **Python class names** deliberately: the manifest is one contract
/// with one vocabulary, and `Expr::StockRef` and `simcore.expr.StockRef` are the same node.
pub fn expr_node_name(node: &Expr) -> &'static str {
    match node {
        Expr::Const(_) => "Const",
        Expr::StockRef(_) => "StockRef",
        Expr::ParamRef(_) => "ParamRef",
        Expr::ForcingRef(_) => "ForcingRef",
        Expr::StepN => "StepN",
        Expr::Neg(_) => "Neg",
        Expr::BinOp { .. } => "BinOp",
        Expr::Monod { .. } => "Monod",
    }
}

/// One inhabitant of every `Expr` variant — the hand-maintained half of the node census.
///
/// ⚠ A roster, not a derivation, and it is the honest weak point of `expr_nodes`. It is
/// built out of real values rather than written as strings so that the names still come
/// from [`expr_node_name`]: a variant *renamed* (arm updated) flows through, and only a
/// variant *added* can be missed. `sample_nodes_cover_every_name` checks the roster has no
/// duplicate names, which is what makes a missed variant show up as a count mismatch
/// against the frozen manifest rather than as a silent shrink.
fn sample_nodes() -> Vec<Expr> {
    vec![
        Expr::Const(0.0),
        Expr::StockRef("s".to_string()),
        Expr::ParamRef("p".to_string()),
        Expr::ForcingRef("f".to_string()),
        Expr::StepN,
        Expr::Neg(Box::new(Expr::StepN)),
        Expr::BinOp {
            op: BinaryOp::Add,
            left: Box::new(Expr::StepN),
            right: Box::new(Expr::StepN),
        },
        Expr::Monod {
            substrate: Box::new(Expr::StepN),
            half_saturation: Box::new(Expr::StepN),
        },
    ]
}

/// The frozen grammar's node vocabulary, sorted — the manifest's `expr_nodes`.
pub fn expr_nodes() -> Vec<&'static str> {
    let names: BTreeSet<&'static str> = sample_nodes().iter().map(expr_node_name).collect();
    names.into_iter().collect()
}

/// Every `BinaryOp` variant — the hand-maintained half of the operator census.
///
/// ⚠ Same shape as [`sample_nodes`]: the *symbols* come from `BinaryOp::symbol()`, which
/// is the parser's and the S-expr renderer's own spelling, so only an added variant can be
/// missed. `/` is deliberately absent from the type itself — the grammar defers division,
/// and an enum with three variants makes an unsupported op unrepresentable rather than a
/// runtime error.
const BINARY_OP_VARIANTS: &[BinaryOp] = &[BinaryOp::Add, BinaryOp::Sub, BinaryOp::Mul];

/// The frozen binary-operator symbols, sorted — the manifest's `binary_ops`.
pub fn binary_ops() -> Vec<&'static str> {
    let symbols: BTreeSet<&'static str> = BINARY_OP_VARIANTS.iter().map(|o| o.symbol()).collect();
    symbols.into_iter().collect()
}

/// Each scenario-file spec → its legal key set — the manifest's `schema_fields`.
///
/// ⚠ **The values are load-bearing, the labels are not.** Each list is the exact `const`
/// `schema::reject_unknown_keys` rejects against, so dropping a name here does not
/// mis-describe the parser, it *changes* it — a committed scenario file naming that key
/// stops loading, and the crossport anchors go red. That is a stronger tie than any of the
/// other rosters in this module.
///
/// The **labels** (`"ScenarioSpec"`, `"ParamPackRef"`, …) are the Python class names, typed
/// here by hand. Two of them name no Rust type at all — this port binds `params:` through a
/// `ParamsSpec` enum and `includes:` through an `IncludeSpec` enum — so the label is a
/// contract name in the way `crew.food_metabolism` is, not a symbol either port resolves.
/// `std::any::type_name` was not used to derive them, for slice 2's reason: its output
/// format is explicitly unspecified, and a toolchain bump must not move a frozen manifest.
pub fn schema_fields() -> BTreeMap<&'static str, Vec<&'static str>> {
    let mut out: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    for (label, keys) in [
        ("BundleSpec", crate::schema::BUNDLE_KEYS),
        ("FlowSpec", crate::schema::FLOW_KEYS),
        ("ForcingSpec", crate::schema::FORCING_KEYS),
        ("IncludeSpec", crate::schema::INCLUDE_KEYS),
        ("KineticsSpec", crate::schema::KINETICS_KEYS),
        ("ParamPackRef", crate::schema::PARAM_PACK_KEYS),
        ("ScenarioSpec", crate::schema::SCENARIO_KEYS),
        ("StockSpec", crate::schema::STOCK_KEYS),
    ] {
        let sorted: BTreeSet<&'static str> = keys.iter().copied().collect();
        out.insert(label, sorted.into_iter().collect());
    }
    out
}

/// Sort a roster into the manifest's canonical order.
///
/// ⚠ Every axis this module emits is **sorted**, and it is done here rather than in the
/// dump program on purpose. The first run of that program emitted `REF_KEYWORDS` in
/// declaration order — `[stock, param, forcing]` against the manifest's
/// `[forcing, param, stock]` — and the comparison caught it as a divergence. The *content*
/// was right; only the order was not. Sorting at the printer would have left every future
/// axis one forgotten `sort()` away from the same false alarm, and a false alarm on this
/// gate reads as a port divergence to hunt.
fn sorted(names: impl IntoIterator<Item = &'static str>) -> Vec<&'static str> {
    let unique: BTreeSet<&'static str> = names.into_iter().collect();
    unique.into_iter().collect()
}

/// The closed reference-keyword set, sorted — the manifest's `ref_keywords`.
///
/// Load-bearing: this is the table `expr_parser::Parser::ident` tests an identifier
/// against, so a name removed here is a form the grammar stops accepting.
pub fn ref_keywords() -> Vec<&'static str> {
    sorted(REF_KEYWORDS)
}

/// The legal `integrator:` names, sorted — the manifest's `integrator_names`.
pub fn integrator_names() -> Vec<&'static str> {
    sorted(INTEGRATOR_NAMES.iter().copied())
}

/// The legal `rate_class:` names, sorted — the manifest's `rate_classes`.
pub fn rate_classes() -> Vec<&'static str> {
    sorted(RATE_CLASSES.iter().copied())
}

/// The named frozen param sets, sorted — the manifest's `param_loaders`.
pub fn param_loaders() -> Vec<&'static str> {
    sorted(PARAM_SET_NAMES.iter().copied())
}

/// One author-selectable flow type's frozen contract.
pub struct FlowTypeSurface {
    /// The frozen `Flow` class this type lowers to, read from the **constructed flow's**
    /// `Flow::type_name()` — slice 2's trait method, on the one path that matters.
    pub cls: &'static str,
    /// The wiring keys an authored file must supply, sorted.
    pub wiring_fields: Vec<&'static str>,
    /// The named frozen param set the constructor consumes, if any.
    pub param_set: Option<&'static str>,
    /// The first-order rate constants the build-time `k·h < 1` precondition checks.
    pub rate_params: Vec<&'static str>,
    /// `(regulated wiring field, setpoint param)` if the type is demand-controlled.
    pub demand_controlled: Option<(&'static str, &'static str)>,
}

/// The author-selectable frozen-flow surface — the manifest's `flow_types`.
///
/// ⚠ **Roster hand-maintained, entries derived.** The names come from
/// [`FLOW_TYPE_NAMES`], which that module already documents as hand-maintained because a
/// Rust `match` cannot be enumerated; everything *about* each type is read out of its
/// `FlowTypeSpec` and — for `cls` — out of a flow this function actually **constructs**,
/// so a wiring field renamed in the constructor's arm, a param set moved, a rate param
/// dropped or a demand-control pair cleared all move the manifest.
///
/// Building each flow rather than tabulating its class name is deliberate: it is the same
/// `build_frozen_flow` path an authored scenario takes, so `cls` cannot claim a class the
/// registry would not actually instantiate. The wiring is fed placeholder stock ids (the
/// field names themselves) — no run happens, only construction.
pub fn flow_types() -> BTreeMap<&'static str, FlowTypeSurface> {
    let mut out: BTreeMap<&'static str, FlowTypeSurface> = BTreeMap::new();
    for name in FLOW_TYPE_NAMES {
        let spec = flow_type(name).unwrap_or_else(|| panic!("FLOW_TYPE_NAMES lists {name:?}, which flow_type() does not resolve — the two surfaces flow_registry names have drifted"));
        let wiring: BTreeMap<String, String> = spec
            .wiring_fields
            .iter()
            .map(|f| ((*f).to_string(), (*f).to_string()))
            .collect();
        let flow = build_frozen_flow(name, "surface.probe", 0, &wiring).unwrap_or_else(|e| {
            panic!("flow_type() resolves {name:?} but build_frozen_flow() refuses it: {e}")
        });
        let mut wiring_fields: Vec<&'static str> = spec.wiring_fields.to_vec();
        wiring_fields.sort_unstable();
        let mut rate_params: Vec<&'static str> = spec.rate_params.to_vec();
        rate_params.sort_unstable();
        out.insert(
            name,
            FlowTypeSurface {
                cls: flow.type_name(),
                wiring_fields,
                param_set: spec.param_set,
                rate_params,
                demand_controlled: spec.demand_controlled,
            },
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_nodes_cover_every_name() {
        // The roster's own guard: distinct samples, distinct names. A duplicate would let
        // a variant hide behind another's name and shrink the frozen set silently.
        let names: Vec<&str> = sample_nodes().iter().map(expr_node_name).collect();
        let unique: BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(
            names.len(),
            unique.len(),
            "duplicate names in sample_nodes()"
        );
        assert_eq!(expr_nodes().len(), names.len());
    }

    #[test]
    fn expr_node_names_are_bare_class_names() {
        // Slice 2's rule, applied to the grammar's vocabulary: a module path here would be
        // a name the Python contract cannot match, and the manifest is one vocabulary.
        for name in expr_nodes() {
            assert!(
                !name.contains("::") && name.chars().all(|c| c.is_ascii_alphanumeric()),
                "{name:?} is not a bare class name"
            );
        }
    }

    #[test]
    fn binary_op_symbols_come_from_the_renderer() {
        // Tie the census to the spelling the S-expr renderer and parse vectors use, rather
        // than to a second table of symbols.
        assert_eq!(binary_ops(), vec!["*", "+", "-"]);
        for op in BINARY_OP_VARIANTS {
            assert!(binary_ops().contains(&op.symbol()));
        }
    }

    #[test]
    fn schema_labels_carry_the_parsers_own_key_sets() {
        // The load-bearing half: each list must be the const the parser rejects against,
        // not a copy. Checked by identity of content against the module's consts.
        let fields = schema_fields();
        assert_eq!(
            fields["ScenarioSpec"].len(),
            crate::schema::SCENARIO_KEYS.len()
        );
        assert!(fields["ScenarioSpec"].contains(&"n_sub"));
        assert_eq!(fields["ParamPackRef"], vec!["pack"]);
        assert_eq!(fields.len(), 8, "a spec label was added or dropped");
    }

    #[test]
    fn flow_types_report_the_class_the_registry_builds() {
        // `cls` is the one value in this module that cannot be read off a table: it comes
        // from a constructed flow. Pin two ends of the registry and the cardinality.
        let types = flow_types();
        assert_eq!(types.len(), FLOW_TYPE_NAMES.len());
        assert_eq!(types["crew.food_metabolism"].cls, "FoodMetabolism");
        assert_eq!(types["eclss.o2_makeup"].cls, "O2Makeup");
        assert_eq!(
            types["eclss.o2_makeup"].demand_controlled,
            Some(("cabin_o2", "o2_setpoint"))
        );
        assert_eq!(
            types["power.self_discharge"].rate_params,
            vec!["self_discharge_rate"]
        );
    }

    #[test]
    fn the_step_token_is_the_one_the_parser_accepts() {
        // STEP_TOKEN is spliced into the manifest, so it has to be the token that actually
        // lowers to StepN rather than a literal beside the parser.
        let parsed = crate::expr_parser::parse_rate_expr(STEP_TOKEN).expect("step token parses");
        assert_eq!(expr_node_name(&parsed), "StepN");
    }
}
