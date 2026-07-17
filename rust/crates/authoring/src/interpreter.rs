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

use std::collections::{BTreeMap, BTreeSet, HashMap};

use simcore::environment::{constant, Schedule, SourceResolver};
use simcore::expr::{DeclarativeFlow, Expr};
use simcore::flow::Flow;
use simcore::quantities::{Quantity, StockKind, BALANCE_ATOL, BALANCE_RTOL};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

use crate::compose::apply_includes;
use crate::errors::AuthoringError;
use crate::expr_parser::parse_rate_expr;
use crate::flow_registry::{
    build_frozen_flow, flow_type, frozen_rate_value, kinetics_param_map, FLOW_TYPE_NAMES,
};
use crate::schema::{FlowSpec, ParamsSpec, ScenarioSpec, StockSpec};
use crate::template::{eval_numeric_field, resolve_parameters};
use crate::yaml::parse_document;

/// The legal [`FlowSpec::rate_class`] values — the author-visible **rate-class
/// vocabulary** (mirrors Python `interpreter._RATE_CLASSES`).
///
/// **Closed at two, by the core's own signature**: [`simcore::multirate::multirate_step`]
/// takes exactly two `Substepper`s (decision N3 — the driver takes the two pre-built
/// integrators and does not infer the partition). So unlike the flow-type registry, which
/// is explicitly expected to grow, this vocabulary cannot grow without a `simcore` change.
pub const RATE_CLASSES: &[&str] = &["fast", "slow"];

/// The divisor giving the SLOW set's effective step under the pinned split: `dt/2`.
///
/// **This tracks [`crate::run::SPLIT`], and the coupling is the point.** Strang runs the
/// slow set as two `dt/2` half-steps around the fast block
/// (`simcore::multirate`: `ops = [(slow, dt/2), *fast_ops, (slow, dt/2)]`), so `dt/2` —
/// *not* `dt/n_sub` — is the step a slow flow is actually integrated at. Under **Lie**
/// the slow set would step at the full `dt`, making this divisor **too permissive by 2×,
/// silently, in the unsafe direction**. `slow_step_tracks_the_split_actually_used` in
/// [`crate::run`]'s tests asserts the split is still Strang, so flipping it goes red here
/// rather than quietly loosening the check.
const SLOW_STEP_DIVISOR: f64 = 2.0;

/// The step size a flow is ACTUALLY integrated at — the number `k` multiplies (mirrors
/// Python `interpreter._effective_step`).
///
/// Three cases, and conflating them is the trap this function exists to name:
///
/// * **single-rate** → `dt`. The whole registry steps once per master step.
/// * **multi-rate, fast** → `dt/n_sub`. The point of the cadence knob.
/// * **multi-rate, slow** → `dt/2` ([`SLOW_STEP_DIVISOR`]), *independent of* `n_sub`,
///   because Strang splits the slow set into two half-steps.
///
/// **The multi-rate plan specified `k·(dt/n_sub) < 1` for every flow, and that is
/// measured WRONG for the slow set** — a false PASS in the unsafe direction. With
/// `eclss.co2_scrubber` classed slow at master `dt=3600`, `n_sub=60`, the formula reports
/// `k·h = 0.06` (safe) while the flow truly steps at 1800 s → `k·h = 1.8`: the run rations
/// 24 times and empties `cabin_co2` to exactly 0.0. It is the **same Strang fact** that
/// turned that plan's predicted 60× Thermal saving into a measured 30×.
pub fn effective_step(dt: f64, n_sub: u32, multirate: bool, slow: bool) -> f64 {
    if !multirate {
        return dt;
    }
    if slow {
        dt / SLOW_STEP_DIVISOR
    } else {
        dt / f64::from(n_sub)
    }
}

/// The interpreted graph plus its run config — everything a run needs (the
/// `BuiltScenario` analogue).
pub struct BuiltScenario {
    pub name: String,
    pub state: State,
    /// The **whole** flow set — the single-rate path, and what [`crate::graph_dump`]
    /// renders.
    pub registry: Registry,
    /// The **disjoint partition** of [`BuiltScenario::registry`] by rate class (decision
    /// N3), over the same stock dict — what [`simcore::multirate::multirate_step`]
    /// consumes. `slow ∪ fast == registry` and `slow ∩ fast == ∅` by construction.
    ///
    /// **Three registries, deliberately** (the Python ruling): keeping the whole
    /// `registry` is what lets [`crate::run`] take the pre-multi-rate single-rate code
    /// path **verbatim** when no partition is declared, rather than leaning on the
    /// (measured, but then load-bearing) `n_sub=1` identity.
    ///
    /// **Rust builds these by lowering the flows a second time**, where Python simply
    /// shares its flow objects across three `Registry` views: a `Box<dyn Flow>` is
    /// *owned*, so the same flow cannot sit in two registries. `build_flow` is a pure,
    /// deterministic function of `(spec, stocks)` — frozen params come from the
    /// `domains::params` constants and an authored rate from the same parsed AST — so the
    /// two lowerings are identical by construction. The cost is build-time only.
    pub slow_registry: Registry,
    pub fast_registry: Registry,
    pub resolver: SourceResolver,
    /// The requested integrator kind (`"euler"` / `"rk4"`); [`crate::run`] constructs
    /// the matching integrator.
    pub integrator: String,
    pub dt: f64,
    pub steps: u64,
    /// The fast set's sub-step count inside one master `dt` (see [`ScenarioSpec::n_sub`]).
    pub n_sub: u32,
    /// True if any flow is an authored [`DeclarativeFlow`] — the **"authored ≠
    /// validated"** marker (decision B): conservation + determinism are guaranteed for
    /// such a run, scientific validity is not.
    pub has_authored_kinetics: bool,
}

impl BuiltScenario {
    /// True if this scenario declared a multi-rate cadence — i.e. needs the driver.
    ///
    /// Either a sub-stepped fast set (`n_sub > 1`) *or* a non-empty slow partition. The
    /// two are not independent: [`interpret`] refuses a slow set at `n_sub == 1`, so on
    /// any **built** scenario this is equivalent to `n_sub > 1`. The robust form is
    /// written anyway — the equivalence is a consequence of that refusal, not a property
    /// of multi-rate, and hard-coding it would silently mislower a scenario the day the
    /// refusal is relaxed.
    pub fn is_multirate(&self) -> bool {
        self.n_sub > 1 || !self.slow_registry.flows().is_empty()
    }
}

/// Build the runnable graph from a scenario spec (the `interpret` analogue).
///
/// Any `includes` are merged first ([`crate::compose::apply_includes`]): each bundle
/// file's parameters/stocks/flows/forcings are flattened into the scenario (bundles
/// first, then inline; a duplicate across sources is an error), yielding a
/// self-contained spec lowered exactly as a hand-flattened one — so composition adds no
/// per-step surface. `base_dir` is the directory bundle paths resolve against (mirroring
/// Python `interpret(spec, base_dir=…)`); its **only** Rust use is bundle resolution
/// (parameter packs are deferred in the Rust port), and running the merge here — not
/// only in [`load_scenario`] — means a spec with non-empty `includes` can never reach
/// the rest of `interpret` un-applied.
///
/// Template `parameters` are resolved next (defaults + `overrides`), then any stock
/// `amount` / forcing `const` expression over them is evaluated to a literal. Stocks
/// are lowered and keyed by id (a duplicate id is an error); flows are lowered via the
/// registry (`Registry` re-sorts into canonical id order, so authoring order is inert);
/// forcings become constant schedules.
pub fn interpret(
    spec: &ScenarioSpec,
    base_dir: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
) -> Result<BuiltScenario, AuthoringError> {
    interpret_inner(spec, base_dir, overrides, false)
}

/// [`interpret`] with the **rate precondition skipped** — the study hatch.
///
/// The `run_scenario_allowing_rationing` idiom (Rust has no default arguments, and
/// widening `interpret`'s signature would churn every caller for a flag that should be
/// rare), and it exists for the same purpose: **studying** an unsafe run, never making a
/// scenario "work". It does not make the step safe; it makes the platform stop objecting.
///
/// **This and the rationing hatch are two gates at two stages, and neither implies the
/// other**: this one opens the *build*, `run_scenario_allowing_rationing` opens the *run*.
/// The verbosity is the feature.
pub fn interpret_allowing_unsafe_step(
    spec: &ScenarioSpec,
    base_dir: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
) -> Result<BuiltScenario, AuthoringError> {
    interpret_inner(spec, base_dir, overrides, true)
}

fn interpret_inner(
    spec: &ScenarioSpec,
    base_dir: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
    allow_unsafe_step: bool,
) -> Result<BuiltScenario, AuthoringError> {
    let spec = apply_includes(spec, base_dir)?;
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

    // The rate-class partition (N3). Runs AFTER the include merge, so a
    // bundle-contributed `rate_class: slow` flow is seen.
    let slow_ids = slow_flow_ids(&spec)?;
    if spec.n_sub == 1 && !slow_ids.is_empty() {
        return Err(AuthoringError::new(format!(
            "n_sub=1 with a non-empty slow set ({:?}): a partition at n_sub=1 buys NO \
             rate separation (the fast set takes one full-dt sub-step) and no \
             performance win — yet it is not inert. It does not reproduce the \
             single-rate trajectory: the slow set is split into two dt/2 half-steps \
             ((1-k*dt/2)^2 != (1-k*dt)) and, the dominant effect, the fast flows read \
             slow-updated stocks mid-step. So this is a misconfiguration that would \
             silently move the answer. Either raise n_sub (the fast set then sub-steps \
             at dt/n_sub, which is the point of the partition), or drop the 'rate_class: \
             slow' key(s) to run single-rate.",
            slow_ids.iter().collect::<Vec<_>>()
        )));
    }
    if !allow_unsafe_step {
        check_rate_preconditions(&spec, &slow_ids)?;
    }
    // Lower the flows a SECOND time to build the disjoint partition: a `Box<dyn Flow>` is
    // owned, so (unlike Python, which shares one flow object across three `Registry`
    // views) the same flow cannot sit in two registries. `build_flow` is deterministic in
    // `(spec, stocks)`, so this lowering is identical to the one above by construction.
    // Built unconditionally, mirroring Python: with no authored `rate_class` keys the
    // slow set is empty and `fast_registry` holds every flow, so it is inert for every
    // pre-multi-rate scenario.
    let mut slow_flows: Vec<Box<dyn Flow>> = Vec::new();
    let mut fast_flows: Vec<Box<dyn Flow>> = Vec::new();
    for flow_spec in &spec.flows {
        let flow = build_flow(flow_spec, &stocks)?;
        if slow_ids.contains(&flow_spec.id) {
            slow_flows.push(flow);
        } else {
            fast_flows.push(flow);
        }
    }
    let slow_registry = Registry::flows_only(slow_flows, &stocks)?;
    let fast_registry = Registry::flows_only(fast_flows, &stocks)?;

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
        slow_registry,
        fast_registry,
        resolver,
        integrator: spec.integrator.clone(),
        dt: spec.dt,
        steps: spec.steps,
        n_sub: spec.n_sub,
        has_authored_kinetics,
    })
}

/// Read a scenario YAML file, validate its schema, and interpret it (the
/// `load_scenario` analogue). `overrides` instantiate a template's `parameters`.
pub fn load_scenario(
    path: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
) -> Result<BuiltScenario, AuthoringError> {
    load_scenario_inner(path, overrides, false)
}

/// [`load_scenario`] with the **rate precondition skipped** — the file-loading half of
/// the study hatch (see [`interpret_allowing_unsafe_step`]).
///
/// Threaded rather than omitted so that the file-loading surface an author actually calls
/// is not the one place the hatch is unavailable.
pub fn load_scenario_allowing_unsafe_step(
    path: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
) -> Result<BuiltScenario, AuthoringError> {
    load_scenario_inner(path, overrides, true)
}

fn load_scenario_inner(
    path: &std::path::Path,
    overrides: &BTreeMap<String, f64>,
    allow_unsafe_step: bool,
) -> Result<BuiltScenario, AuthoringError> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        AuthoringError::new(format!("cannot read scenario file {}: {e}", path.display()))
    })?;
    let doc = parse_document(&text)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    // Bundle paths (Step 6) resolve relative to the scenario file's directory.
    let base_dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    interpret_inner(&spec, base_dir, overrides, allow_unsafe_step)
}

/// The declared **slow** partition, with the rate-class vocabulary validated (the
/// `_slow_flow_ids` analogue).
///
/// Must run over the **post-**[`apply_includes`] spec, and that is the whole reason this
/// is not a schema-level check: a *bundle* may contribute `rate_class: slow` flows, and a
/// schema-level validator sees only the scenario's own inline `flows` — it would miss
/// them, and (worse) would miss them silently, lowering a partitioned scenario as if it
/// were single-rate.
fn slow_flow_ids(spec: &ScenarioSpec) -> Result<BTreeSet<String>, AuthoringError> {
    let mut slow = BTreeSet::new();
    for flow_spec in &spec.flows {
        if !RATE_CLASSES.contains(&flow_spec.rate_class.as_str()) {
            return Err(AuthoringError::new(format!(
                "flow {:?}: unknown rate class {:?} (known: {RATE_CLASSES:?})",
                flow_spec.id, flow_spec.rate_class
            )));
        }
        if flow_spec.rate_class == "slow" {
            slow.insert(flow_spec.id.clone());
        }
    }
    Ok(slow)
}

/// The `AuthoringError` text for a failed rate precondition — the remedy is conditional
/// on the flow's rate class (the `_rate_precondition_message` analogue).
///
/// "Increase `n_sub`" is honest for a **fast** flow and actively misleading for a
/// **slow** one, which steps at `dt/2` however large `n_sub` grows: an author who followed
/// that advice would raise `n_sub`, watch nothing change, and conclude the check is
/// broken. Every remedy quotes a **concrete number** the author can act on rather than
/// restating the inequality — they already know `k*h < 1` failed; what they need is the
/// value that satisfies it. (The message text is **not** a parity target — see
/// [`crate::errors`] — but the *conditional structure* mirrors Python's deliberately.)
struct RateViolation<'a> {
    flow_id: &'a str,
    flow_type: &'a str,
    param: &'a str,
    /// The declared rate constant, /s.
    k: f64,
    /// The **effective** step this flow is integrated at — see [`effective_step`]. Not
    /// necessarily `dt`, and for a slow flow not `dt/n_sub` either; that distinction is
    /// the whole point of the type.
    h: f64,
    /// The master step, kept alongside `h` because the remedies quote it.
    dt: f64,
    slow: bool,
    multirate: bool,
}

fn rate_precondition_message(v: &RateViolation<'_>) -> String {
    let RateViolation {
        flow_id,
        flow_type,
        param,
        k,
        h,
        dt,
        slow,
        multirate,
    } = *v;
    let (where_, remedy) = if !multirate {
        (
            format!("dt={dt:?}"),
            format!(
                "Reduce dt below {:?} s, or adopt a multi-rate cadence: keep dt as the \
                 export cadence neighbours see and add 'n_sub' > {:?} so this flow \
                 sub-steps at dt/n_sub",
                1.0 / k,
                k * dt
            ),
        )
    } else if slow {
        (
            format!("dt/2={h:?} (the slow set's Strang half-step, NOT dt/n_sub)"),
            format!(
                "This flow is rate_class 'slow', so it steps at dt/2 REGARDLESS of n_sub \
                 (Strang splitting) — raising n_sub will NOT help it. Either reduce dt \
                 below {:?} s, or re-class this flow 'fast' so that n_sub governs its \
                 step too",
                2.0 / k
            ),
        )
    } else {
        (
            format!("dt/n_sub={h:?} (master dt={dt:?})"),
            format!(
                "Increase n_sub past {:?} (i.e. to at least {}), which leaves the master \
                 export cadence untouched, or reduce dt",
                k * dt,
                (k * dt) as u64 + 1
            ),
        )
    };
    format!(
        "flow {flow_id:?} ({flow_type}): param {param:?} = {k:?} /s is a first-order rate \
         constant, and this scenario integrates the flow at {where_}, giving k*h = {:?} \
         >= 1. The step is too large for this flow's frozen rate: over one step the law \
         removes more than the whole stock (or, for a demand-controlled flow, overshoots \
         its setpoint and oscillates). k*h < 1 is the platform's EXPORT-FIDELITY bound, \
         deliberately stricter than the textbook stability bound k*h < 2: this engine \
         couples domains, so a neighbour must be able to USE the exported value, not \
         merely watch it converge eventually. {remedy}. See 'The dt constraint' in \
         docs/authoring-reference.md. To build the unsafe scenario anyway — to STUDY it, \
         not to make it work — use interpret_allowing_unsafe_step / \
         load_scenario_allowing_unsafe_step.",
        k * h
    )
}

/// Refuse a scenario whose step is too large for a declared first-order rate — the
/// build-time `k · h < 1` precondition (the `_check_rate_preconditions` analogue).
///
/// For every frozen flow type declaring [`FlowTypeSpec::rate_params`], read each `k` and
/// require `k · h < 1` at that flow's [`effective_step`]. Transcendental-free
/// (`+ − × <`) ⇒ byte-safe against the Python mirror.
///
/// **Where `k` comes from differs from Python, and the difference is load-bearing** —
/// see [`crate::flow_registry::frozen_rate_value`]. Python reads the **pack-resolved**
/// value off the built flow (a pack may inflate a gain, which is *why* the check is at
/// build time); Rust reads the **frozen constant**, which is sound only because packs are
/// deferred here. So this mirror carries the rule, not the rationale, and its
/// unique-over-rationing value narrows to exactly `eclss.o2_makeup` (the one
/// demand-controlled flow, invisible to the run-time backstop at any `dt`).
///
/// **What this honestly does NOT cover, by declaration rather than omission**: authored
/// `kinetics` (no `FlowTypeSpec`, so it skips by construction),
/// `thermal.radiator_reject` (`τ ≫ dt` is not a predicate), and `eclss.crew_metabolism`
/// (`forced draw < stock` is state-dependent). The claim is "the platform catches the
/// `k·dt` family", never "your dt is safe".
pub fn check_rate_preconditions(
    spec: &ScenarioSpec,
    slow_ids: &BTreeSet<String>,
) -> Result<(), AuthoringError> {
    let multirate = spec.n_sub > 1 || !slow_ids.is_empty();
    for flow_spec in &spec.flows {
        // Authored kinetics: structurally uncheckable (the author wrote the rate law).
        let Some(type_name) = flow_spec.type_.as_deref() else {
            continue;
        };
        // `build_flow` already validated the name by the time this runs.
        let Some(type_spec) = flow_type(type_name) else {
            continue;
        };
        if type_spec.rate_params.is_empty() {
            continue;
        }
        let Some(param_set) = type_spec.param_set else {
            // A type declaring rate_params but no param set is a registry bug, not
            // authored input — loud, never silently unchecked.
            return Err(AuthoringError::new(format!(
                "flow {:?} ({type_name}): declares rate_params {:?} but no param set; \
                 the flow-type registry is inconsistent",
                flow_spec.id, type_spec.rate_params
            )));
        };
        let slow = slow_ids.contains(&flow_spec.id);
        let h = effective_step(spec.dt, spec.n_sub, multirate, slow);
        for param in type_spec.rate_params {
            let k = frozen_rate_value(param_set, param)?;
            if k * h >= 1.0 {
                return Err(AuthoringError::new(rate_precondition_message(
                    &RateViolation {
                        flow_id: &flow_spec.id,
                        flow_type: type_name,
                        param,
                        k,
                        h,
                        dt: spec.dt,
                        slow,
                        multirate,
                    },
                )));
            }
        }
    }
    Ok(())
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
        Expr::Monod {
            substrate,
            half_saturation,
        } => {
            collect_refs(substrate, params, stocks);
            collect_refs(half_saturation, params, stocks);
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
