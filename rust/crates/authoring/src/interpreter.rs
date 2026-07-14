//! Lower a validated [`ScenarioSpec`] to a runnable engine graph — the Rust mirror of
//! Python `authoring.interpreter` (Phase 9, Step 4b).
//!
//! The interpreter turns declarative data into `(State, Registry, SourceResolver)` **by
//! calling the frozen constructors** ([`simcore::boundary`] / [`Stock::new`] / the
//! frozen crew flow constructors / [`simcore::expr::DeclarativeFlow`]). It does no
//! trajectory float math; the one arithmetic it does is Step 3's build-time
//! **template boundary-eval** (`param('crew_count') * 1000.0`, via [`crate::template`]).
//!
//! Everything decidable from the file alone is checked here and returned as an
//! [`AuthoringError`] (unknown flow type, wiring that does not match the type's fields,
//! a missing/spurious param reference, an unbalanced authored stoichiometry). A
//! *well-formed* scenario that wires a flow badly interprets cleanly and surfaces as a
//! runtime [`SimError::Conservation`] on the first step (the safety property, raised
//! from the run — not this layer's job). **Parameter packs are deferred in the Rust
//! port** (see [`crate::flow_registry`]): a `params: {pack: …}` reference is an error.

use std::collections::{BTreeMap, HashMap};

use simcore::environment::{constant, Schedule, SourceResolver};
use simcore::expr::{DeclarativeFlow, Expr};
use simcore::flow::Flow;
use simcore::quantities::{Quantity, StockKind, BALANCE_ATOL, BALANCE_RTOL};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::errors::AuthoringError;
use crate::expr_parser::parse_rate_expr;
use crate::flow_registry::{build_frozen_flow, flow_type, kinetics_param_map, FLOW_TYPE_NAMES};
use crate::schema::{FlowSpec, ParamsSpec, ScenarioSpec, StockSpec};
use crate::template::{eval_numeric_field, resolve_parameters};
use crate::yaml::parse_document;

/// The interpreted graph plus its run config — everything a run needs (the
/// `BuiltScenario` analogue).
pub struct BuiltScenario {
    pub name: String,
    pub state: State,
    pub registry: Registry,
    pub resolver: SourceResolver,
    /// The requested integrator kind (`"euler"` / `"rk4"`); [`crate::run`] constructs
    /// the matching integrator.
    pub integrator: String,
    pub dt: f64,
    pub steps: u64,
    /// True if any flow is an authored [`DeclarativeFlow`] — the **"authored ≠
    /// validated"** marker (decision B): conservation + determinism are guaranteed for
    /// such a run, scientific validity is not.
    pub has_authored_kinetics: bool,
}

/// Build the runnable graph from a scenario spec (the `interpret` analogue).
///
/// Template `parameters` are resolved first (defaults + `overrides`), then any stock
/// `amount` / forcing `const` expression over them is evaluated to a literal. Stocks
/// are lowered and keyed by id (a duplicate id is an error); flows are lowered via the
/// registry (`Registry` re-sorts into canonical id order, so authoring order is inert);
/// forcings become constant schedules.
pub fn interpret(
    spec: &ScenarioSpec,
    overrides: &BTreeMap<String, f64>,
) -> Result<BuiltScenario, AuthoringError> {
    let params = resolve_parameters(&spec.parameters, overrides)?;

    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    for stock_spec in &spec.stocks {
        let stock = build_stock(stock_spec, &params)?;
        if stocks.contains_key(&stock.id) {
            return Err(AuthoringError::new(format!(
                "duplicate stock id {:?}",
                stock.id
            )));
        }
        stocks.insert(stock.id.clone(), stock);
    }
    let state = State::new(0, stocks.clone(), spec.rng_seed, BTreeMap::new())?;

    let mut flows: Vec<Box<dyn Flow>> = Vec::new();
    for flow_spec in &spec.flows {
        flows.push(build_flow(flow_spec, &stocks)?);
    }
    let registry = Registry::flows_only(flows, &stocks)?;

    let mut forcings: HashMap<String, Schedule> = HashMap::new();
    for (name, forcing) in &spec.forcings {
        let value =
            eval_numeric_field(&forcing.const_, &params, &format!("forcing {name:?} const"))?;
        forcings.insert(name.clone(), constant(value)?);
    }
    let resolver = SourceResolver::new(forcings, HashMap::new())?;

    let has_authored_kinetics = spec.flows.iter().any(|f| f.kinetics.is_some());

    Ok(BuiltScenario {
        name: spec.name.clone(),
        state,
        registry,
        resolver,
        integrator: spec.integrator.clone(),
        dt: spec.dt,
        steps: spec.steps,
        has_authored_kinetics,
    })
}

/// Read a scenario YAML file, validate its schema, and interpret it (the
/// `load_scenario` analogue). `overrides` instantiate a template's `parameters`.
pub fn load_scenario(
    path: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
) -> Result<BuiltScenario, AuthoringError> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        AuthoringError::new(format!("cannot read scenario file {}: {e}", path.display()))
    })?;
    let doc = parse_document(&text)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret(&spec, overrides)
}

/// Lower one [`StockSpec`] to a frozen `Stock` (the `_build_stock` analogue). The unit
/// is derived from the quantity (never authored); an unknown quantity/kind is an
/// [`AuthoringError`]; the deeper `Stock::new` invariants surface as `SimError` mapped
/// to `AuthoringError`.
fn build_stock(spec: &StockSpec, params: &BTreeMap<String, f64>) -> Result<Stock, AuthoringError> {
    let amount = eval_numeric_field(&spec.amount, params, &format!("stock {:?} amount", spec.id))?;
    let quantity = Quantity::from_value(&spec.quantity).map_err(|_| {
        AuthoringError::new(format!(
            "stock {:?}: unknown quantity {:?}",
            spec.id, spec.quantity
        ))
    })?;
    let kind = StockKind::from_value(&spec.kind).map_err(|_| {
        AuthoringError::new(format!("stock {:?}: unknown kind {:?}", spec.id, spec.kind))
    })?;
    let mut composition: BTreeMap<Quantity, f64> = BTreeMap::new();
    if let Some(comp) = &spec.composition {
        for (qname, coeff) in comp {
            let q = Quantity::from_value(qname).map_err(|_| {
                AuthoringError::new(format!(
                    "stock {:?}: unknown composition quantity {:?}",
                    spec.id, qname
                ))
            })?;
            composition.insert(q, *coeff);
        }
    }
    Ok(Stock::new(
        spec.id.clone(),
        spec.domain.clone(),
        quantity,
        quantity.canonical_unit(),
        amount,
        kind,
        spec.extinction_threshold,
        spec.unclamped,
        composition,
    )?)
}

/// Lower one [`FlowSpec`] to a `Flow`: a frozen `type`, or authored `kinetics` (the
/// `_build_flow` analogue). The schema has already guaranteed exactly one of
/// `type`/`kinetics` is set.
fn build_flow(
    spec: &FlowSpec,
    stocks: &BTreeMap<String, Stock>,
) -> Result<Box<dyn Flow>, AuthoringError> {
    if spec.kinetics.is_some() {
        return build_declarative_flow(spec, stocks);
    }
    let type_name = spec.type_.as_deref().expect("schema: type xor kinetics");
    let type_spec = flow_type(type_name).ok_or_else(|| {
        AuthoringError::new(format!(
            "flow {:?}: unknown flow type {type_name:?} (known: {FLOW_TYPE_NAMES:?})",
            spec.id
        ))
    })?;

    // The frozen crew flow constructors carry no priority field (they are shared with
    // the station callers, which construct them at the trait-default priority), so the
    // Rust port cannot yet honor a non-zero priority on a **frozen** flow type. Rather
    // than silently drop it (a divergence from Python, which does honor it), reject it
    // loudly — no anchor uses a non-zero frozen-flow priority, and an authored
    // `kinetics` flow honors priority fully. (A follow-up threads priority through the
    // frozen constructors if a real scenario needs it.)
    if spec.priority != 0 {
        return Err(AuthoringError::new(format!(
            "flow {:?} ({type_name}): a non-zero priority on a frozen flow type is not \
             yet supported in the Rust port (the frozen constructors carry no priority \
             field); use 0, or an authored 'kinetics' flow (which honors priority)",
            spec.id
        )));
    }

    // Wiring keys must match the flow type's fields exactly (set equality).
    let wiring: BTreeMap<String, String> = spec.wiring.iter().cloned().collect();
    let mut wiring_keys: Vec<&str> = wiring.keys().map(|s| s.as_str()).collect();
    wiring_keys.sort_unstable();
    let mut expected: Vec<&str> = type_spec.wiring_fields.to_vec();
    expected.sort_unstable();
    if wiring_keys != expected {
        return Err(AuthoringError::new(format!(
            "flow {:?} ({type_name}): wiring keys {wiring_keys:?} do not match this \
             flow type's fields {expected:?}",
            spec.id
        )));
    }

    // Params validation (the `_resolve_params` analogue): a params-taking type needs a
    // matching named set (a pack is deferred in the Rust port); a param-free type must
    // carry no `params`.
    match type_spec.param_set {
        Some(set) => match &spec.params {
            None => {
                return Err(AuthoringError::new(format!(
                    "flow {:?} ({type_name}): this flow type requires 'params' (the set \
                     name {set:?})",
                    spec.id
                )));
            }
            Some(ParamsSpec::Named(name)) if name == set => {}
            Some(ParamsSpec::Named(name)) => {
                return Err(AuthoringError::new(format!(
                    "flow {:?} ({type_name}): param set {name:?} does not match this flow \
                     type's set {set:?}",
                    spec.id
                )));
            }
            Some(ParamsSpec::Pack(_)) => {
                return Err(AuthoringError::new(format!(
                    "flow {:?} ({type_name}): parameter packs are deferred in the Rust \
                     port (Step 4b); name the frozen set {set:?} instead",
                    spec.id
                )));
            }
        },
        None => {
            if spec.params.is_some() {
                return Err(AuthoringError::new(format!(
                    "flow {:?} ({type_name}): this flow type takes no params, but 'params' \
                     was given",
                    spec.id
                )));
            }
        }
    }

    build_frozen_flow(type_name, &spec.id, spec.priority, &wiring)
}

/// Lower a `kinetics` [`FlowSpec`] to a [`DeclarativeFlow`] (the
/// `_build_declarative_flow` analogue): parse the rate, resolve the param map, then
/// apply the build-time structural checks (referential integrity, non-empty
/// stoichiometry over known stocks, balance-by-construction).
fn build_declarative_flow(
    spec: &FlowSpec,
    stocks: &BTreeMap<String, Stock>,
) -> Result<Box<dyn Flow>, AuthoringError> {
    let kinetics = spec.kinetics.as_ref().expect("caller: kinetics branch");
    let rate = parse_rate_expr(&kinetics.rate)?;
    let stoichiometry: Vec<(String, f64)> = kinetics.stoichiometry.clone();
    if stoichiometry.is_empty() {
        return Err(AuthoringError::new(format!(
            "flow {:?}: kinetics 'stoichiometry' is empty",
            spec.id
        )));
    }

    // The authored-kinetics param map (packs deferred; None → empty).
    let param_map = match &spec.params {
        None => BTreeMap::new(),
        Some(ParamsSpec::Named(set)) => kinetics_param_map(set).map_err(|e| {
            AuthoringError::new(format!("flow {:?}: {}", spec.id, e.message))
        })?,
        Some(ParamsSpec::Pack(_)) => {
            return Err(AuthoringError::new(format!(
                "flow {:?}: parameter packs for authored 'kinetics' flows are deferred; \
                 name a frozen param set instead",
                spec.id
            )));
        }
    };

    // Referential integrity: every param the rate reads must be in the param map, and
    // every stock the rate reads OR the stoichiometry names must exist.
    let mut ref_params: Vec<String> = Vec::new();
    let mut ref_stocks: Vec<String> = Vec::new();
    collect_refs(&rate, &mut ref_params, &mut ref_stocks);
    ref_params.sort();
    ref_params.dedup();
    for name in &ref_params {
        if !param_map.contains_key(name) {
            let available: Vec<&String> = param_map.keys().collect();
            return Err(AuthoringError::new(format!(
                "flow {:?}: rate references param {name:?} not in its param set \
                 (available: {available:?})",
                spec.id
            )));
        }
    }
    ref_stocks.sort();
    ref_stocks.dedup();
    let stoich_stocks: Vec<String> = stoichiometry.iter().map(|(s, _)| s.clone()).collect();
    for stock_id in ref_stocks.iter().chain(stoich_stocks.iter()) {
        if !stocks.contains_key(stock_id) {
            return Err(AuthoringError::new(format!(
                "flow {:?}: references unknown stock {stock_id:?}",
                spec.id
            )));
        }
    }

    check_stoichiometry_balanced(&spec.id, &stoichiometry, stocks)?;

    let params: Vec<(String, f64)> = param_map.into_iter().collect();
    Ok(Box::new(DeclarativeFlow::new(
        spec.id.clone(),
        spec.priority,
        rate,
        stoichiometry,
        params,
    )))
}

/// Recursively collect `param`/`stock` reference names from a rate AST (the
/// `_collect_refs` analogue). `forcing` refs are intentionally NOT collected — their
/// referential integrity is resolve-time (`env.get`), not a build check.
fn collect_refs(node: &Expr, params: &mut Vec<String>, stocks: &mut Vec<String>) {
    match node {
        Expr::ParamRef(name) => params.push(name.clone()),
        Expr::StockRef(stock) => stocks.push(stock.clone()),
        Expr::ForcingRef(_) => {} // resolve-time (env.get), by design
        Expr::Neg(operand) => collect_refs(operand, params, stocks),
        Expr::BinOp { left, right, .. } => {
            collect_refs(left, params, stocks);
            collect_refs(right, params, stocks);
        }
        Expr::Const(_) | Expr::StepN => {}
    }
}

/// Verify the coefficient vector balances per quantity (decision C, build time) — the
/// `_check_stoichiometry_balanced` analogue. `Σ(coeff · composition[q])` for each
/// quantity must be within `assert_flow_balanced`'s relative tolerance (exact for
/// integer coeffs; tolerance-backed for fractional splits). Because the single scalar
/// `rate·dt` multiplies every leg, a balanced coefficient vector keeps `Σ legs = 0` for
/// any rate/state — so an unbalanced authored flow is rejected here, before it runs.
fn check_stoichiometry_balanced(
    flow_id: &str,
    stoichiometry: &[(String, f64)],
    stocks: &BTreeMap<String, Stock>,
) -> Result<(), AuthoringError> {
    let mut residual: BTreeMap<Quantity, f64> = BTreeMap::new();
    let mut scale: BTreeMap<Quantity, f64> = BTreeMap::new();
    for (stock_id, coeff) in stoichiometry {
        let stock = stocks
            .get(stock_id)
            .expect("caller checked stoichiometry stocks exist");
        for (quantity, comp) in &stock.composition {
            *residual.entry(*quantity).or_insert(0.0) += coeff * comp;
            let s = scale.entry(*quantity).or_insert(0.0);
            *s = s.max((coeff * comp).abs());
        }
    }
    // Sort by the uppercase member name, mirroring Python's `sorted(residual, key=q.name)`.
    let mut quantities: Vec<Quantity> = residual.keys().copied().collect();
    quantities.sort_by_key(|q| q.name());
    for quantity in quantities {
        let scale_q = scale.get(&quantity).copied().unwrap_or(0.0);
        let tol = BALANCE_ATOL + BALANCE_RTOL * scale_q;
        let res = residual[&quantity];
        if res.abs() > tol {
            return Err(AuthoringError::new(format!(
                "flow {flow_id:?}: authored stoichiometry is not balanced for {} \
                 (Σ coeff·composition = {res:?}, tolerance {tol:?}); an authored flow \
                 must conserve every quantity",
                quantity.name()
            )));
        }
    }
    Ok(())
}
