//! File composition: merge included **domain/species bundles** into a scenario — the
//! Rust mirror of Python `authoring.compose` (Phase 9, Step 6b).
//!
//! A scenario [`ScenarioSpec::includes`] a list of bundle-file paths
//! ([`crate::schema::BundleSpec`]); [`apply_includes`] reads each and merges its
//! `parameters` / `stocks` / `flows` / `forcings` into the scenario's, producing a
//! single **flat** [`ScenarioSpec`] the interpreter lowers exactly as before. This is
//! the "authored, not programmed" payoff one level up: a station is *composed* from
//! reusable bundles (a crew species, a battery domain) rather than re-declared inline.
//!
//! **Merge semantics** (a cross-port freeze surface — Step 7 inherits them; they must
//! stay identical to the Python `apply_includes`):
//!
//! - **Order:** included bundles first (in `includes` order), then the scenario's own
//!   inline declarations. So a scenario that is *only* an include reproduces the
//!   bundle's declaration order exactly (the single-bundle byte-identity anchor). The
//!   merged `Vec`s preserve this order — pinned by a unit test *before* the interpreter
//!   canonicalizes them into id-sorted maps (the serialized outputs are id-sorted, so
//!   order is only observable on the returned spec, not on a run/graph-dump).
//! - **No silent override:** a duplicate stock id, flow id, forcing key, or parameter
//!   name across any two sources is an [`AuthoringError`]. Disjoint-domain composition
//!   (crew + battery) needs no id-namespacing.
//! - **Multi-instance id-namespacing** (Step 6c): an include may be a
//!   [`IncludeSpec::Prefixed`] (`{bundle, prefix}`) instead of a bare path. A `prefix`
//!   namespaces every id the bundle declares (stock id / flow id / forcing key →
//!   `<prefix>.<id>`) and every reference to it — `wiring` values, `stoichiometry` keys,
//!   and the `stock(...)`/`forcing(...)` refs inside a `kinetics` rate (parsed to the
//!   AST, prefixed via [`prefix_expr_refs`], re-emitted by
//!   [`crate::expr_parser::render_rate_expr`]). This lets the **same** bundle be included
//!   more than once (two batteries) without the id collision a bare double-include hits.
//!   `param(...)` refs are never rewritten (a rate's `param` names a *frozen* param set,
//!   shared across instances); bundle-**parameter** namespacing is deferred. A bundle
//!   whose *frozen* flows bind a forcing by a hardcoded name (the crew flows'
//!   `crew_o2_intake` constant) can't reach a namespaced forcing key — the documented
//!   crew-forcing blocker; kinetics / disjoint bundles namespace cleanly.
//! - **Flat, one level deep:** a bundle carries no `includes` of its own (rejected by
//!   [`BundleSpec::from_yaml`]'s allowed-key set).
//! - **Run config lives only in the top-level scenario** (also enforced by the bundle
//!   allowed-key set).
//!
//! **Parameter packs inside an included bundle are deferred** (matching Step 1's
//! "full-file packs only" and the Rust port's "packs deferred"): a `{pack: …}`
//! reference on a bundle flow is a clean [`AuthoringError`]. (In the Rust port packs are
//! deferred everywhere, so this fires the same way at interpret time — but rejecting it
//! here keeps the message + the merge-semantics parity with Python.)

use std::collections::HashMap;
use std::path::Path;

use simcore::expr::Expr;

use crate::errors::AuthoringError;
use crate::expr_parser::{parse_rate_expr, render_rate_expr};
use crate::schema::{
    BundleSpec, FlowSpec, ForcingSpec, IncludeSpec, KineticsSpec, ParamsSpec, ScenarioSpec,
    StockSpec,
};
use crate::yaml::parse_document;

/// Merge `spec.includes` bundle files into a flat [`ScenarioSpec`].
///
/// Returns a clone of `spec` unchanged if it has no includes (the pre-Step-6 path, so
/// existing scenarios and bare specs are untouched). Otherwise reads each bundle
/// relative to `base_dir` (the scenario file's directory), validates it, and returns a
/// new spec with the merged parameters/stocks/flows/forcings and an emptied `includes` —
/// which the interpreter then lowers identically to a hand-flattened scenario.
pub fn apply_includes(
    spec: &ScenarioSpec,
    base_dir: &Path,
) -> Result<ScenarioSpec, AuthoringError> {
    if spec.includes.is_empty() {
        return Ok(spec.clone());
    }

    let mut merger = Merger::default();
    for inc in &spec.includes {
        let (path, prefix) = match inc {
            IncludeSpec::Bare(p) => (p.as_str(), None),
            IncludeSpec::Prefixed { bundle, prefix } => (bundle.as_str(), Some(prefix.as_str())),
        };
        let bundle = load_bundle(&base_dir.join(path), path)?;
        let (bundle, source) = match prefix {
            None => (bundle, format!("bundle {path:?}")),
            Some(px) => (
                namespace_bundle(&bundle, px)?,
                format!("bundle {path:?} (prefix {px:?})"),
            ),
        };
        merger.merge(
            &source,
            &bundle.parameters,
            &bundle.stocks,
            &bundle.flows,
            &bundle.forcings,
        )?;
    }
    // The scenario's own inline declarations merge LAST (includes-first-then-inline).
    merger.merge(
        "the scenario",
        &spec.parameters,
        &spec.stocks,
        &spec.flows,
        &spec.forcings,
    )?;

    Ok(ScenarioSpec {
        name: spec.name.clone(),
        integrator: spec.integrator.clone(),
        dt: spec.dt,
        steps: spec.steps,
        rng_seed: spec.rng_seed,
        includes: Vec::new(),
        parameters: merger.parameters,
        stocks: merger.stocks,
        flows: merger.flows,
        forcings: merger.forcings,
    })
}

/// The order-preserving merge accumulator + per-namespace source labels (for the
/// collision message). Mirrors the closures over the outer dicts in Python's
/// `apply_includes`.
#[derive(Default)]
struct Merger {
    parameters: Vec<(String, f64)>,
    stocks: Vec<StockSpec>,
    flows: Vec<FlowSpec>,
    forcings: Vec<(String, ForcingSpec)>,
    param_source: HashMap<String, String>,
    stock_source: HashMap<String, String>,
    flow_source: HashMap<String, String>,
    forcing_source: HashMap<String, String>,
}

impl Merger {
    /// Merge one source's declarations, raising on any cross-source duplicate id/key/
    /// name (no silent override). `source` is a human label for the collision message.
    fn merge(
        &mut self,
        source: &str,
        params: &[(String, f64)],
        stocks: &[StockSpec],
        flows: &[FlowSpec],
        forcings: &[(String, ForcingSpec)],
    ) -> Result<(), AuthoringError> {
        for (name, value) in params {
            if self.param_source.contains_key(name) {
                return Err(AuthoringError::new(format!(
                    "duplicate parameter {name:?}: declared by both {source} and an \
                     earlier source (parameters share one flat namespace across \
                     includes; rename or use 'overrides')"
                )));
            }
            self.param_source.insert(name.clone(), source.to_string());
            self.parameters.push((name.clone(), *value));
        }
        for st in stocks {
            if let Some(prev) = self.stock_source.get(&st.id) {
                return Err(AuthoringError::new(format!(
                    "duplicate stock id {:?}: declared by both {source} and {prev}",
                    st.id
                )));
            }
            self.stock_source.insert(st.id.clone(), source.to_string());
            self.stocks.push(st.clone());
        }
        for fl in flows {
            if let Some(prev) = self.flow_source.get(&fl.id) {
                return Err(AuthoringError::new(format!(
                    "duplicate flow id {:?}: declared by both {source} and {prev}",
                    fl.id
                )));
            }
            // Parameter packs inside an included bundle are deferred (a bundle pack must
            // resolve against the bundle's directory — per-flow source-dir threading no
            // Step-6 anchor exercises). A top-level scenario flow may still carry a pack
            // (the Rust port defers packs at interpret time, but the *merge* semantics
            // match Python: the bundle-flow pack is rejected here, by the merge).
            if source != "the scenario" {
                if let Some(ParamsSpec::Pack(_)) = &fl.params {
                    return Err(AuthoringError::new(format!(
                        "flow {:?} in {source}: parameter packs inside an included \
                         bundle are deferred (a bundle pack must resolve against the \
                         bundle's directory); name a frozen param set instead",
                        fl.id
                    )));
                }
            }
            self.flow_source.insert(fl.id.clone(), source.to_string());
            self.flows.push(fl.clone());
        }
        for (key, forcing) in forcings {
            if self.forcing_source.contains_key(key) {
                return Err(AuthoringError::new(format!(
                    "duplicate forcing {key:?}: declared by both {source} and an \
                     earlier source"
                )));
            }
            self.forcing_source.insert(key.clone(), source.to_string());
            self.forcings.push((key.clone(), forcing.clone()));
        }
        Ok(())
    }
}

/// Rewrite `stock`/`forcing` reference names in a rate AST under `prefix` (Step 6c) —
/// the Rust mirror of Python `_prefix_expr_refs`. `stock(...)`/`forcing(...)` args are
/// bundle ids/keys → `<prefix>.<name>`; `param(...)` is **left untouched** (in a rate it
/// names a *frozen* param set, shared across instances). `Const`/`StepN` carry no refs.
fn prefix_expr_refs(node: &Expr, prefix: &str) -> Expr {
    match node {
        Expr::StockRef(id) => Expr::StockRef(format!("{prefix}.{id}")),
        Expr::ForcingRef(name) => Expr::ForcingRef(format!("{prefix}.{name}")),
        Expr::Neg(operand) => Expr::Neg(Box::new(prefix_expr_refs(operand, prefix))),
        Expr::BinOp { op, left, right } => Expr::BinOp {
            op: *op,
            left: Box::new(prefix_expr_refs(left, prefix)),
            right: Box::new(prefix_expr_refs(right, prefix)),
        },
        Expr::Monod {
            substrate,
            half_saturation,
        } => Expr::Monod {
            substrate: Box::new(prefix_expr_refs(substrate, prefix)),
            half_saturation: Box::new(prefix_expr_refs(half_saturation, prefix)),
        },
        Expr::Const(_) | Expr::ParamRef(_) | Expr::StepN => node.clone(),
    }
}

/// Return a copy of `flow` with its id, wiring targets, stoichiometry keys and
/// kinetics-rate stock/forcing refs namespaced under `prefix` (the `_namespace_flow`
/// analogue).
fn namespace_flow(flow: &FlowSpec, prefix: &str) -> Result<FlowSpec, AuthoringError> {
    let wiring: Vec<(String, String)> = flow
        .wiring
        .iter()
        .map(|(k, v)| (k.clone(), format!("{prefix}.{v}")))
        .collect();
    let kinetics = match &flow.kinetics {
        None => None,
        Some(k) => {
            let ast = parse_rate_expr(&k.rate)?;
            Some(KineticsSpec {
                rate: render_rate_expr(&prefix_expr_refs(&ast, prefix)),
                stoichiometry: k
                    .stoichiometry
                    .iter()
                    .map(|(s, c)| (format!("{prefix}.{s}"), *c))
                    .collect(),
            })
        }
    };
    Ok(FlowSpec {
        id: format!("{prefix}.{}", flow.id),
        type_: flow.type_.clone(),
        priority: flow.priority,
        wiring,
        kinetics,
        params: flow.params.clone(),
    })
}

/// Return a copy of `bundle` with every declared id namespaced under `prefix` (the
/// `_namespaced_bundle` analogue). Stock ids, flow ids (+ their references) and forcing
/// keys become `<prefix>.<id>`; `parameters` are **not** prefixed (bundle-parameter
/// namespacing deferred — a param-bearing bundle is un-multi-instanceable for the
/// crew-forcing reason, so a second prefixed instance collides on the parameter name).
fn namespace_bundle(bundle: &BundleSpec, prefix: &str) -> Result<BundleSpec, AuthoringError> {
    let stocks: Vec<StockSpec> = bundle
        .stocks
        .iter()
        .map(|s| StockSpec {
            id: format!("{prefix}.{}", s.id),
            ..s.clone()
        })
        .collect();
    let mut flows = Vec::with_capacity(bundle.flows.len());
    for flow in &bundle.flows {
        flows.push(namespace_flow(flow, prefix)?);
    }
    let forcings: Vec<(String, ForcingSpec)> = bundle
        .forcings
        .iter()
        .map(|(k, v)| (format!("{prefix}.{k}"), v.clone()))
        .collect();
    Ok(BundleSpec {
        parameters: bundle.parameters.clone(),
        stocks,
        flows,
        forcings,
    })
}

/// Read + validate one bundle file (`extra="forbid"` — no run config, no nesting).
fn load_bundle(path: &Path, label: &str) -> Result<BundleSpec, AuthoringError> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        AuthoringError::new(format!(
            "included bundle {label:?} could not be read from {}: {e}",
            path.display()
        ))
    })?;
    let doc = parse_document(&text)?;
    BundleSpec::from_yaml(&doc)
}
