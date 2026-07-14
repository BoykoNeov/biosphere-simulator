//! The declarative scenario-file schema — the Rust mirror of Python
//! `authoring.schema` (Phase 9, Step 4b).
//!
//! Binds a [`crate::yaml::YamlValue`] tree (from the closed-subset reader) into typed
//! specs the interpreter lowers to frozen engine objects, exactly as the pydantic
//! models do on the Python side. Every mapping is **`extra="forbid"`** — an unknown
//! key is a schema error, not a silent drop — and the `type`-xor-`kinetics` flow
//! exclusivity is enforced here (mirroring `FlowSpec._check_type_xor_kinetics`).
//!
//! **The pyyaml numeric hazard is handled in [`crate::yaml::is_yaml_number`].** A
//! stock `amount` / forcing `const` is a `number | expression` union: a **bare**
//! YAML-1.1 number lowers to [`NumericField::Number`]; a quoted scalar or a non-number
//! bare scalar is a template [`NumericField::Expr`] — the same split pyyaml makes when
//! it returns a `float` vs a `str`.

use crate::errors::AuthoringError;
use crate::yaml::{is_yaml_number, YamlValue};

/// A numeric scenario field (`StockSpec.amount`, `ForcingSpec.const`): a literal
/// number, or a template expression over the scenario's `parameters` (Step 3).
#[derive(Debug, Clone, PartialEq)]
pub enum NumericField {
    /// A bare YAML-1.1 number, already parsed to `f64` (correctly-rounded via
    /// `f64::from_str`, matching Python `float()`).
    Number(f64),
    /// A template expression string (a quoted scalar or a non-number bare scalar),
    /// parsed + evaluated to a literal at interpret time (Step 3 boundary-eval).
    Expr(String),
}

/// One stock's declaration (mirrors `StockSpec`).
#[derive(Debug, Clone, PartialEq)]
pub struct StockSpec {
    pub id: String,
    pub domain: String,
    pub quantity: String,
    pub kind: String,
    pub amount: NumericField,
    pub composition: Option<Vec<(String, f64)>>,
    pub unclamped: bool,
    pub extinction_threshold: f64,
}

/// An authored-kinetics flow's rate × stoichiometry (mirrors `KineticsSpec`).
#[derive(Debug, Clone, PartialEq)]
pub struct KineticsSpec {
    pub rate: String,
    pub stoichiometry: Vec<(String, f64)>,
}

/// A flow's params selector (mirrors the `str | ParamPackRef | None` union).
#[derive(Debug, Clone, PartialEq)]
pub enum ParamsSpec {
    /// A frozen param-set name (`params: crew`).
    Named(String),
    /// A parameter pack (`params: {pack: …}`) — **deferred in the Rust port** (Step 4b
    /// keeps named sets only; a pack surfaces as an interpret-time "deferred" error).
    Pack(String),
}

/// One flow: a frozen `type` + wiring, or authored `kinetics` (mirrors `FlowSpec`).
#[derive(Debug, Clone, PartialEq)]
pub struct FlowSpec {
    pub id: String,
    pub type_: Option<String>,
    pub priority: i64,
    pub wiring: Vec<(String, String)>,
    pub kinetics: Option<KineticsSpec>,
    pub params: Option<ParamsSpec>,
}

/// A constant forcing schedule (mirrors `ForcingSpec`; Step-0 constants only).
#[derive(Debug, Clone, PartialEq)]
pub struct ForcingSpec {
    pub const_: NumericField,
}

/// A reusable **domain / species bundle**: stocks + flows + forcings + parameters
/// (mirrors Python `authoring.schema.BundleSpec`, Phase 9, Step 6). A scenario
/// [`ScenarioSpec::includes`] one or more bundle files; [`crate::compose::apply_includes`]
/// merges each bundle's declarations into the scenario's flat graph.
///
/// A bundle carries **no run config** (integrator/dt/steps/name/rng_seed) and **no
/// nested** `includes` — both are rejected by the allowed-key set below (exactly
/// `[parameters, stocks, flows, forcings]`), the `extra="forbid"` analogue: run config
/// lives only in the top-level scenario, and includes are flat, one level deep.
#[derive(Debug, Clone, PartialEq)]
pub struct BundleSpec {
    pub parameters: Vec<(String, f64)>,
    pub stocks: Vec<StockSpec>,
    pub flows: Vec<FlowSpec>,
    pub forcings: Vec<(String, ForcingSpec)>,
}

impl BundleSpec {
    /// Validate and bind a parsed bundle document into a [`BundleSpec`] (the
    /// `BundleSpec.model_validate` analogue). The allowed-key set is exactly
    /// `[parameters, stocks, flows, forcings]`, so a stray `steps:`/`integrator:` (run
    /// config) or a nested `includes:` is a schema error — the two Python
    /// bundle-schema-reject cases.
    pub fn from_yaml(doc: &YamlValue) -> Result<BundleSpec, AuthoringError> {
        let entries = doc.as_mapping("bundle")?;
        reject_unknown_keys(
            entries,
            &["parameters", "stocks", "flows", "forcings"],
            "bundle",
        )?;
        let parameters = match field(entries, "parameters") {
            None => Vec::new(),
            Some(value) => parse_parameters(value)?,
        };
        let stocks = parse_stock_list(entries, "bundle.stocks")?;
        let flows = parse_flow_list(entries, "bundle.flows")?;
        let forcings = match field(entries, "forcings") {
            None => Vec::new(),
            Some(value) => parse_forcings(value)?,
        };
        Ok(BundleSpec {
            parameters,
            stocks,
            flows,
            forcings,
        })
    }
}

/// A whole authored scenario (mirrors `ScenarioSpec`).
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioSpec {
    pub name: String,
    pub integrator: String,
    pub dt: f64,
    pub steps: u64,
    pub rng_seed: u64,
    /// Bundle-file paths to compose in (Step 6), resolved relative to the scenario
    /// file's directory. Merged by [`crate::compose::apply_includes`] before interpret.
    pub includes: Vec<String>,
    pub parameters: Vec<(String, f64)>,
    pub stocks: Vec<StockSpec>,
    pub flows: Vec<FlowSpec>,
    pub forcings: Vec<(String, ForcingSpec)>,
}

impl ScenarioSpec {
    /// Validate and bind a parsed document into a [`ScenarioSpec`] (the
    /// `ScenarioSpec.model_validate` analogue).
    pub fn from_yaml(doc: &YamlValue) -> Result<ScenarioSpec, AuthoringError> {
        let entries = doc.as_mapping("scenario")?;
        reject_unknown_keys(
            entries,
            &[
                "name",
                "integrator",
                "dt",
                "steps",
                "rng_seed",
                "includes",
                "parameters",
                "stocks",
                "flows",
                "forcings",
            ],
            "scenario",
        )?;
        let name = require_str(entries, "name", "scenario")?;
        let integrator = require_str(entries, "integrator", "scenario")?;
        let dt = require_f64(entries, "dt", "scenario")?;
        let steps = require_u64(entries, "steps", "scenario")?;
        let rng_seed = opt_u64(entries, "rng_seed", "scenario")?.unwrap_or(0);
        let includes = match field(entries, "includes") {
            None => Vec::new(),
            Some(value) => value
                .as_sequence("scenario.includes")?
                .iter()
                .map(|v| scalar_str(v, "scenario.includes item"))
                .collect::<Result<Vec<_>, _>>()?,
        };
        let parameters = match field(entries, "parameters") {
            None => Vec::new(),
            Some(value) => parse_parameters(value)?,
        };
        // `stocks`/`flows` are OPTIONAL (Step 6, carry-forward i): a scenario may omit
        // its inline stocks/flows when it composes them from `includes` (the frozen
        // `crew_station.yaml` has neither). The Python `ScenarioSpec` relaxed these to
        // `Field(default_factory=list)`; this mirror matches.
        let stocks = parse_stock_list(entries, "scenario.stocks")?;
        let flows = parse_flow_list(entries, "scenario.flows")?;
        let forcings = match field(entries, "forcings") {
            None => Vec::new(),
            Some(value) => parse_forcings(value)?,
        };
        Ok(ScenarioSpec {
            name,
            integrator,
            dt,
            steps,
            rng_seed,
            includes,
            parameters,
            stocks,
            flows,
            forcings,
        })
    }
}

/// Parse an optional `stocks:` sequence (shared by `ScenarioSpec` and `BundleSpec`);
/// an absent key is an empty list (Step 6, carry-forward i).
fn parse_stock_list(
    entries: &[(String, YamlValue)],
    ctx: &str,
) -> Result<Vec<StockSpec>, AuthoringError> {
    match field(entries, "stocks") {
        None => Ok(Vec::new()),
        Some(value) => value
            .as_sequence(ctx)?
            .iter()
            .map(parse_stock)
            .collect::<Result<Vec<_>, _>>(),
    }
}

/// Parse an optional `flows:` sequence (shared by `ScenarioSpec` and `BundleSpec`); an
/// absent key is an empty list.
fn parse_flow_list(
    entries: &[(String, YamlValue)],
    ctx: &str,
) -> Result<Vec<FlowSpec>, AuthoringError> {
    match field(entries, "flows") {
        None => Ok(Vec::new()),
        Some(value) => value
            .as_sequence(ctx)?
            .iter()
            .map(parse_flow)
            .collect::<Result<Vec<_>, _>>(),
    }
}

fn parse_parameters(value: &YamlValue) -> Result<Vec<(String, f64)>, AuthoringError> {
    let mut out = Vec::new();
    for (name, v) in value.as_mapping("scenario.parameters")? {
        out.push((name.clone(), scalar_f64(v, &format!("parameter {name:?}"))?));
    }
    Ok(out)
}

fn parse_stock(value: &YamlValue) -> Result<StockSpec, AuthoringError> {
    let entries = value.as_mapping("stock")?;
    reject_unknown_keys(
        entries,
        &[
            "id",
            "domain",
            "quantity",
            "kind",
            "amount",
            "composition",
            "unclamped",
            "extinction_threshold",
        ],
        "stock",
    )?;
    let id = require_str(entries, "id", "stock")?;
    let ctx = format!("stock {id:?}");
    let composition = match field(entries, "composition") {
        None => None,
        Some(v) => {
            let mut comp = Vec::new();
            for (q, coeff) in v.as_mapping(&format!("{ctx} composition"))? {
                comp.push((q.clone(), scalar_f64(coeff, &format!("{ctx} composition {q:?}"))?));
            }
            Some(comp)
        }
    };
    Ok(StockSpec {
        id: id.clone(),
        domain: require_str(entries, "domain", &ctx)?,
        quantity: require_str(entries, "quantity", &ctx)?,
        kind: require_str(entries, "kind", &ctx)?,
        amount: require_numeric(entries, "amount", &ctx)?,
        composition,
        unclamped: opt_bool(entries, "unclamped", &ctx)?.unwrap_or(false),
        extinction_threshold: opt_f64(entries, "extinction_threshold", &ctx)?.unwrap_or(0.0),
    })
}

fn parse_flow(value: &YamlValue) -> Result<FlowSpec, AuthoringError> {
    let entries = value.as_mapping("flow")?;
    reject_unknown_keys(
        entries,
        &["id", "type", "priority", "wiring", "kinetics", "params"],
        "flow",
    )?;
    let id = require_str(entries, "id", "flow")?;
    let ctx = format!("flow {id:?}");
    let type_ = opt_str(entries, "type", &ctx)?;
    let priority = opt_i64(entries, "priority", &ctx)?.unwrap_or(0);
    let wiring = match field(entries, "wiring") {
        None => Vec::new(),
        Some(v) => {
            let mut w = Vec::new();
            for (k, val) in v.as_mapping(&format!("{ctx} wiring"))? {
                w.push((k.clone(), scalar_str(val, &format!("{ctx} wiring {k:?}"))?));
            }
            w
        }
    };
    let kinetics = match field(entries, "kinetics") {
        None => None,
        Some(v) => Some(parse_kinetics(v, &ctx)?),
    };
    let params = match field(entries, "params") {
        None => None,
        Some(v) => Some(parse_params(v, &ctx)?),
    };

    // The `type`-xor-`kinetics` exclusivity + no-wiring-on-kinetics validation
    // (mirrors `FlowSpec._check_type_xor_kinetics`).
    if type_.is_some() == kinetics.is_some() {
        return Err(AuthoringError::new(format!(
            "{ctx}: give exactly one of 'type' (a frozen flow type) or 'kinetics' \
             (an authored rate×stoichiometry flow)"
        )));
    }
    if kinetics.is_some() && !wiring.is_empty() {
        return Err(AuthoringError::new(format!(
            "{ctx}: an authored 'kinetics' flow takes no 'wiring' (its stoichiometry \
             names stocks directly)"
        )));
    }

    Ok(FlowSpec {
        id,
        type_,
        priority,
        wiring,
        kinetics,
        params,
    })
}

fn parse_kinetics(value: &YamlValue, ctx: &str) -> Result<KineticsSpec, AuthoringError> {
    let entries = value.as_mapping(&format!("{ctx} kinetics"))?;
    reject_unknown_keys(entries, &["rate", "stoichiometry"], &format!("{ctx} kinetics"))?;
    let rate = require_str(entries, "rate", &format!("{ctx} kinetics"))?;
    let mut stoichiometry = Vec::new();
    for (stock, coeff) in require_field(entries, "stoichiometry", &format!("{ctx} kinetics"))?
        .as_mapping(&format!("{ctx} kinetics stoichiometry"))?
    {
        stoichiometry.push((
            stock.clone(),
            scalar_f64(coeff, &format!("{ctx} stoichiometry {stock:?}"))?,
        ));
    }
    Ok(KineticsSpec { rate, stoichiometry })
}

fn parse_params(value: &YamlValue, ctx: &str) -> Result<ParamsSpec, AuthoringError> {
    match value {
        YamlValue::Scalar { text, .. } => Ok(ParamsSpec::Named(text.clone())),
        YamlValue::Mapping(_) => {
            let entries = value.as_mapping(&format!("{ctx} params"))?;
            reject_unknown_keys(entries, &["pack"], &format!("{ctx} params"))?;
            let pack = require_str(entries, "pack", &format!("{ctx} params"))?;
            Ok(ParamsSpec::Pack(pack))
        }
        _ => Err(AuthoringError::new(format!(
            "{ctx}: 'params' must be a param-set name or a {{pack: …}} reference"
        ))),
    }
}

fn parse_forcings(value: &YamlValue) -> Result<Vec<(String, ForcingSpec)>, AuthoringError> {
    let mut out = Vec::new();
    for (name, v) in value.as_mapping("scenario.forcings")? {
        let entries = v.as_mapping(&format!("forcing {name:?}"))?;
        reject_unknown_keys(entries, &["const"], &format!("forcing {name:?}"))?;
        let const_ = require_numeric(entries, "const", &format!("forcing {name:?}"))?;
        out.push((name.clone(), ForcingSpec { const_ }));
    }
    Ok(out)
}

// --------------------------------------------------------------------------- #
// Field accessors + `extra="forbid"` enforcement.                              #
// --------------------------------------------------------------------------- #

fn field<'a>(entries: &'a [(String, YamlValue)], key: &str) -> Option<&'a YamlValue> {
    entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn require_field<'a>(
    entries: &'a [(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<&'a YamlValue, AuthoringError> {
    field(entries, key).ok_or_else(|| AuthoringError::new(format!("{ctx}: missing key {key:?}")))
}

fn reject_unknown_keys(
    entries: &[(String, YamlValue)],
    allowed: &[&str],
    ctx: &str,
) -> Result<(), AuthoringError> {
    for (key, _) in entries {
        if !allowed.contains(&key.as_str()) {
            return Err(AuthoringError::new(format!(
                "{ctx}: unknown key {key:?} (allowed: {allowed:?})"
            )));
        }
    }
    Ok(())
}

fn scalar_str(value: &YamlValue, ctx: &str) -> Result<String, AuthoringError> {
    let (text, _) = value.as_scalar(ctx)?;
    Ok(text.to_string())
}

fn scalar_f64(value: &YamlValue, ctx: &str) -> Result<f64, AuthoringError> {
    let (text, _) = value.as_scalar(ctx)?;
    text.parse::<f64>()
        .map_err(|_| AuthoringError::new(format!("{ctx}: {text:?} is not a number")))
}

fn require_str(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<String, AuthoringError> {
    scalar_str(require_field(entries, key, ctx)?, &format!("{ctx}.{key}"))
}

fn opt_str(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<Option<String>, AuthoringError> {
    match field(entries, key) {
        None => Ok(None),
        Some(v) => Ok(Some(scalar_str(v, &format!("{ctx}.{key}"))?)),
    }
}

fn require_f64(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<f64, AuthoringError> {
    scalar_f64(require_field(entries, key, ctx)?, &format!("{ctx}.{key}"))
}

fn opt_f64(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<Option<f64>, AuthoringError> {
    match field(entries, key) {
        None => Ok(None),
        Some(v) => Ok(Some(scalar_f64(v, &format!("{ctx}.{key}"))?)),
    }
}

fn require_u64(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<u64, AuthoringError> {
    let ctx2 = format!("{ctx}.{key}");
    let (text, _) = require_field(entries, key, ctx)?.as_scalar(&ctx2)?;
    text.parse::<u64>()
        .map_err(|_| AuthoringError::new(format!("{ctx2}: {text:?} is not a non-negative integer")))
}

fn opt_u64(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<Option<u64>, AuthoringError> {
    match field(entries, key) {
        None => Ok(None),
        Some(_) => Ok(Some(require_u64(entries, key, ctx)?)),
    }
}

fn opt_i64(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<Option<i64>, AuthoringError> {
    match field(entries, key) {
        None => Ok(None),
        Some(v) => {
            let ctx2 = format!("{ctx}.{key}");
            let (text, _) = v.as_scalar(&ctx2)?;
            Ok(Some(text.parse::<i64>().map_err(|_| {
                AuthoringError::new(format!("{ctx2}: {text:?} is not an integer"))
            })?))
        }
    }
}

fn opt_bool(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<Option<bool>, AuthoringError> {
    match field(entries, key) {
        None => Ok(None),
        Some(v) => {
            let ctx2 = format!("{ctx}.{key}");
            let (text, _) = v.as_scalar(&ctx2)?;
            match text {
                "true" => Ok(Some(true)),
                "false" => Ok(Some(false)),
                other => Err(AuthoringError::new(format!(
                    "{ctx2}: {other:?} is not a boolean (use true/false)"
                ))),
            }
        }
    }
}

/// Bind a `number | expression` union field (the `float | str` union): a bare YAML-1.1
/// number → [`NumericField::Number`]; a quoted or non-number scalar →
/// [`NumericField::Expr`] (the pyyaml float-vs-str split).
fn require_numeric(
    entries: &[(String, YamlValue)],
    key: &str,
    ctx: &str,
) -> Result<NumericField, AuthoringError> {
    let ctx2 = format!("{ctx}.{key}");
    let (text, quoted) = require_field(entries, key, ctx)?.as_scalar(&ctx2)?;
    if !quoted && is_yaml_number(text) {
        Ok(NumericField::Number(text.parse::<f64>().map_err(|_| {
            AuthoringError::new(format!("{ctx2}: {text:?} is not a number"))
        })?))
    } else {
        Ok(NumericField::Expr(text.to_string()))
    }
}
