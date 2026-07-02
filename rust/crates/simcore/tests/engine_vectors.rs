//! Engine cross-port bit-exact gate (Phase 7, Step 2 / P7.2).
//!
//! Defines the **same** synthetic transcendental-free scenario as the Python generator
//! (`tests/crossport/gen_engine_vectors.py`), runs it under Euler, RK4, and the
//! multi-rate Strang driver, and asserts every step is **bit-exact (Tier 1)** against
//! the committed `tests/data/engine_vectors.txt` trajectory, decoding each golden
//! amount through the Step-0 hex-float codec.
//!
//! The synthetic flow arithmetic is written **character-for-character identically** to
//! the Python source (float `*` is not associative, so the grouping — `(k * a) * dt` —
//! is load-bearing). Two error-path unit tests round out the acceptance: an imbalanced
//! flow raises `ConservationError`, and an RK4 over-draw raises `ArbitrationError`.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use simcore::auxiliary::AuxProcess;
use simcore::boundary;
use simcore::environment::{constant, Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::hexfloat;
use simcore::integrator::{EulerIntegrator, Rk4Integrator, StepReport};
use simcore::multirate::{multirate_step, Split};
use simcore::quantities::{Quantity, StockKind};
use simcore::registry::Registry;
use simcore::state::{State, Stock};

// --------------------------------------------------------------------------- //
// Shared numeric contract — decimal literals, identical to the Python source.  //
// --------------------------------------------------------------------------- //
const DT: f64 = 0.5;
const INFLOW: f64 = 2.0;
const LEAK_K: f64 = 0.04;
const TRANSFER_K: f64 = 0.02;
const DRAIN_KP: f64 = 0.2;
const AUX_K: f64 = 0.01;
const STEPS_A: usize = 20;
const N_SUB: u32 = 2;

const DT_B: f64 = 1.0;
const POOL_B0: f64 = 10.0;
const DRAIN_B: f64 = 6.0;
const STEPS_B: usize = 2;

fn amt(state: &State, id: &str) -> Result<f64, SimError> {
    state
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("missing stock {id:?}")))
}

// --------------------------------------------------------------------------- //
// Scenario A flows (grouping mirrors gen_engine_vectors.py exactly).           //
// --------------------------------------------------------------------------- //
struct ForcedIn;
impl Flow for ForcedIn {
    fn id(&self) -> &str {
        "sim.forced_in"
    }
    fn evaluate(&self, _s: &State, env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let q = env.get("inflow")?;
        let amt = q * dt;
        FlowResult::new(vec![
            Leg::new("boundary.src".to_string(), -amt)?,
            Leg::new("sim.a".to_string(), amt)?,
        ])
    }
}

struct Leak;
impl Flow for Leak {
    fn id(&self) -> &str {
        "sim.leak"
    }
    fn evaluate(&self, s: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let a = amt(s, "sim.a")?;
        let flux = (LEAK_K * a) * dt;
        FlowResult::new(vec![
            Leg::new("sim.a".to_string(), -flux)?,
            Leg::new("boundary.snk".to_string(), flux)?,
        ])
    }
}

struct Transfer;
impl Flow for Transfer {
    fn id(&self) -> &str {
        "sim.transfer"
    }
    fn evaluate(&self, s: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let a = amt(s, "sim.a")?;
        let flux = (TRANSFER_K * a) * dt;
        FlowResult::new(vec![
            Leg::new("sim.a".to_string(), -flux)?,
            Leg::new("sim.b".to_string(), flux)?,
        ])
    }
}

struct DrainP;
impl Flow for DrainP {
    fn id(&self) -> &str {
        "sim.drain_p"
    }
    fn evaluate(&self, s: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let p = amt(s, "sim.p")?;
        let flux = (DRAIN_KP * p) * dt;
        FlowResult::new(vec![
            Leg::new("sim.p".to_string(), -flux)?,
            Leg::new("boundary.snk".to_string(), flux)?,
        ])
    }
}

struct AuxThermal;
impl AuxProcess for AuxThermal {
    fn id(&self) -> &str {
        "sim.thermal"
    }
    fn evaluate(
        &self,
        s: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let a = amt(s, "sim.a")?;
        let inc = (AUX_K * a) * dt;
        Ok(BTreeMap::from([("thermal_time".to_string(), inc)]))
    }
}

fn scenario_a_stocks() -> BTreeMap<String, Stock> {
    let mut stocks: BTreeMap<String, Stock> = BTreeMap::new();
    let pool = |id: &str, a: f64| {
        Stock::new(
            id.to_string(),
            "sim".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            a,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    };
    stocks.insert("sim.a".to_string(), pool("sim.a", 100.0));
    stocks.insert("sim.b".to_string(), pool("sim.b", 0.0));
    stocks.insert(
        "sim.p".to_string(),
        Stock::new(
            "sim.p".to_string(),
            "sim".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            5.0,
            StockKind::Population,
            1.0,
            false,
            BTreeMap::new(),
        )
        .unwrap(),
    );
    let src = boundary::source("boundary.src".to_string(), Quantity::Carbon, 0.0, true).unwrap();
    let snk = boundary::sink("boundary.snk".to_string(), Quantity::Carbon, 0.0).unwrap();
    let ls = boundary::loss_sink(Quantity::Carbon, 0.0).unwrap();
    stocks.insert(src.id.clone(), src);
    stocks.insert(snk.id.clone(), snk);
    stocks.insert(ls.id.clone(), ls);
    stocks
}

fn scenario_a_state() -> State {
    State::new(
        0,
        scenario_a_stocks(),
        0,
        BTreeMap::from([("thermal_time".to_string(), 7.0)]),
    )
    .unwrap()
}

fn resolver_a() -> SourceResolver {
    let mut forcings: HashMap<String, simcore::environment::Schedule> = HashMap::new();
    forcings.insert("inflow".to_string(), constant(INFLOW).unwrap());
    SourceResolver::new(forcings, HashMap::new()).unwrap()
}

fn reg_full() -> Registry {
    let stocks = scenario_a_stocks();
    Registry::new(
        vec![
            Box::new(ForcedIn),
            Box::new(Leak),
            Box::new(Transfer),
            Box::new(DrainP),
        ],
        &stocks,
        vec![Box::new(AuxThermal)],
    )
    .unwrap()
}

// --------------------------------------------------------------------------- //
// Scenario B flows (forced over-withdrawal → Euler rationing / RK4 error).     //
// --------------------------------------------------------------------------- //
struct DrainForced(&'static str);
impl Flow for DrainForced {
    fn id(&self) -> &str {
        self.0
    }
    fn evaluate(&self, _s: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        let amt = DRAIN_B * dt;
        FlowResult::new(vec![
            Leg::new("simb.pool".to_string(), -amt)?,
            Leg::new("simb.snk".to_string(), amt)?,
        ])
    }
}

fn scenario_b_stocks() -> BTreeMap<String, Stock> {
    let pool = Stock::new(
        "simb.pool".to_string(),
        "simb".to_string(),
        Quantity::Carbon,
        "mol".to_string(),
        POOL_B0,
        StockKind::Pool,
        0.0,
        false,
        BTreeMap::new(),
    )
    .unwrap();
    let snk = boundary::sink("simb.snk".to_string(), Quantity::Carbon, 0.0).unwrap();
    BTreeMap::from([("simb.pool".to_string(), pool), (snk.id.clone(), snk)])
}

fn reg_b() -> Registry {
    let stocks = scenario_b_stocks();
    Registry::flows_only(
        vec![
            Box::new(DrainForced("simb.drain1")),
            Box::new(DrainForced("simb.drain2")),
        ],
        &stocks,
    )
    .unwrap()
}

// --------------------------------------------------------------------------- //
// Trajectory capture + the golden parser.                                     //
// --------------------------------------------------------------------------- //
struct Row {
    state: State,
    rationed: u64,
    events: Vec<simcore::events::ExtinctionEvent>,
}

fn run<F>(mut f: F, init: State, env: &SourceResolver, dt: f64, steps: usize) -> Vec<Row>
where
    F: FnMut(&State, &SourceResolver, f64) -> Result<StepReport, SimError>,
{
    let mut rows = vec![Row {
        state: init.clone(),
        rationed: 0,
        events: vec![],
    }];
    let mut cur = init;
    for _ in 0..steps {
        let rep = f(&cur, env, dt).expect("synthetic scenario must not error");
        cur = rep.state.clone();
        rows.push(Row {
            state: rep.state,
            rationed: rep.rationed,
            events: rep.events,
        });
    }
    rows
}

#[derive(Default)]
struct Golden {
    // key: "kind|scheme|n|id" -> f64
    values: HashMap<String, f64>,
    // "scheme|n" -> (rationed, n_events)
    meta: HashMap<String, (u64, usize)>,
    // "scheme|n" -> ordered [(stock, quantity_value, residual)]
    events: HashMap<String, Vec<(String, String, f64)>>,
    // "kind|scheme|n" -> count of ids present (to catch a missing/extra leaf)
    counts: HashMap<String, usize>,
}

impl Golden {
    fn load() -> Golden {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "tests",
            "data",
            "engine_vectors.txt",
        ]
        .iter()
        .collect();
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let mut g = Golden::default();
        for line in text.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            match f[0] {
                "stock" | "aux" => {
                    let (kind, scheme, n, id, hex) = (f[0], f[1], f[2], f[3], f[4]);
                    let v = hexfloat::parse(hex).expect("golden hex-float");
                    g.values.insert(format!("{kind}|{scheme}|{n}|{id}"), v);
                    *g.counts.entry(format!("{kind}|{scheme}|{n}")).or_insert(0) += 1;
                }
                "meta" => {
                    let (scheme, n, rationed, n_events) = (f[1], f[2], f[3], f[4]);
                    g.meta.insert(
                        format!("{scheme}|{n}"),
                        (rationed.parse().unwrap(), n_events.parse().unwrap()),
                    );
                }
                "event" => {
                    let (scheme, n, stock, quantity, residual) = (f[1], f[2], f[3], f[4], f[5]);
                    let v = hexfloat::parse(residual).expect("golden event hex-float");
                    g.events
                        .entry(format!("{scheme}|{n}"))
                        .or_default()
                        .push((stock.to_string(), quantity.to_string(), v));
                }
                other => panic!("unknown golden line kind {other:?}"),
            }
        }
        g
    }

    fn value(&self, kind: &str, scheme: &str, n: usize, id: &str) -> f64 {
        *self
            .values
            .get(&format!("{kind}|{scheme}|{n}|{id}"))
            .unwrap_or_else(|| panic!("golden missing {kind} {scheme} n={n} {id}"))
    }

    fn count(&self, kind: &str, scheme: &str, n: usize) -> usize {
        self.counts
            .get(&format!("{kind}|{scheme}|{n}"))
            .copied()
            .unwrap_or(0)
    }
}

fn assert_bits(scheme: &str, n: usize, what: &str, got: f64, want: f64) {
    assert_eq!(
        got.to_bits(),
        want.to_bits(),
        "[{scheme} n={n}] {what}: bit mismatch got {} ({:#018x}) vs want {} ({:#018x})",
        hexfloat::format(got),
        got.to_bits(),
        hexfloat::format(want),
        want.to_bits(),
    );
}

fn check_scheme(golden: &Golden, scheme: &str, rows: &[Row]) {
    for (n, row) in rows.iter().enumerate() {
        // Stocks: every produced amount is bit-exact, and the id set matches (no
        // missing/extra leaf).
        assert_eq!(
            row.state.stocks.len(),
            golden.count("stock", scheme, n),
            "[{scheme} n={n}] stock id-set size mismatch"
        );
        for (sid, stock) in &row.state.stocks {
            assert_bits(
                scheme,
                n,
                &format!("stock {sid}"),
                stock.amount,
                golden.value("stock", scheme, n, sid),
            );
        }
        // Aux.
        assert_eq!(
            row.state.aux.len(),
            golden.count("aux", scheme, n),
            "[{scheme} n={n}] aux name-set size mismatch"
        );
        for (name, val) in &row.state.aux {
            assert_bits(
                scheme,
                n,
                &format!("aux {name}"),
                *val,
                golden.value("aux", scheme, n, name),
            );
        }
        // Meta + events (per produced step; n=0 is the initial state).
        if n > 0 {
            let (rationed, n_events) = golden.meta[&format!("{scheme}|{n}")];
            assert_eq!(row.rationed, rationed, "[{scheme} n={n}] rationed count");
            assert_eq!(row.events.len(), n_events, "[{scheme} n={n}] event count");
            let ev_golden = golden.events.get(&format!("{scheme}|{n}"));
            for (i, ev) in row.events.iter().enumerate() {
                let (gs, gq, gr) = &ev_golden.expect("golden events")[i];
                assert_eq!(&ev.stock, gs, "[{scheme} n={n}] event {i} stock");
                assert_eq!(ev.quantity.value(), gq, "[{scheme} n={n}] event {i} quantity");
                assert_bits(scheme, n, &format!("event {i} residual"), ev.residual, *gr);
            }
        }
    }
}

// --------------------------------------------------------------------------- //
// THE GATE: the synthetic trajectory is bit-exact Rust↔Python for all schemes. //
// --------------------------------------------------------------------------- //
#[test]
fn engine_synthetic_trajectory_is_bit_exact() {
    let golden = Golden::load();
    let env = resolver_a();

    // Euler.
    let euler = EulerIntegrator::new(reg_full());
    let rows = run(
        |s, e, dt| euler.step_report(s, e, dt),
        scenario_a_state(),
        &env,
        DT,
        STEPS_A,
    );
    check_scheme(&golden, "euler", &rows);

    // RK4.
    let rk4 = Rk4Integrator::new(reg_full());
    let rows = run(
        |s, e, dt| rk4.step_report(s, e, dt),
        scenario_a_state(),
        &env,
        DT,
        STEPS_A,
    );
    check_scheme(&golden, "rk4", &rows);

    // Multi-rate (Strang) over disjoint slow/fast registries sharing the stock dict.
    let stocks = scenario_a_stocks();
    let slow = EulerIntegrator::new(Registry::flows_only(vec![Box::new(ForcedIn)], &stocks).unwrap());
    let fast = EulerIntegrator::new(
        Registry::flows_only(
            vec![Box::new(Leak), Box::new(Transfer), Box::new(DrainP)],
            &stocks,
        )
        .unwrap(),
    );
    let rows = run(
        |s, e, dt| multirate_step(&slow, &fast, s, e, dt, N_SUB, Split::Strang),
        scenario_a_state(),
        &env,
        DT,
        STEPS_A,
    );
    check_scheme(&golden, "multirate", &rows);

    // Scenario B (Euler rationing).
    let euler_b = EulerIntegrator::new(reg_b());
    let init_b = State::new(0, scenario_b_stocks(), 0, BTreeMap::new()).unwrap();
    let rows = run(
        |s, e, dt| euler_b.step_report(s, e, dt),
        init_b,
        &SourceResolver::empty(),
        DT_B,
        STEPS_B,
    );
    check_scheme(&golden, "ration", &rows);
}

// --------------------------------------------------------------------------- //
// Error-path acceptance (no golden — the failure type is the assertion).       //
// --------------------------------------------------------------------------- //

/// A deliberately imbalanced flow trips the every-step conservation gate.
struct Imbalanced;
impl Flow for Imbalanced {
    fn id(&self) -> &str {
        "sim.imbalanced"
    }
    fn evaluate(&self, _s: &State, _env: &dyn Environment, dt: f64) -> Result<FlowResult, SimError> {
        // Withdraw 1·dt from a, deposit only 0.5·dt into b — 0.5·dt CARBON vanishes.
        FlowResult::new(vec![
            Leg::new("sim.a".to_string(), -dt)?,
            Leg::new("sim.b".to_string(), 0.5 * dt)?,
        ])
    }
}

#[test]
fn imbalanced_flow_raises_conservation_error() {
    let pool = |id: &str, a: f64| {
        Stock::new(
            id.to_string(),
            "sim".to_string(),
            Quantity::Carbon,
            "mol".to_string(),
            a,
            StockKind::Pool,
            0.0,
            false,
            BTreeMap::new(),
        )
        .unwrap()
    };
    let stocks = BTreeMap::from([
        ("sim.a".to_string(), pool("sim.a", 100.0)),
        ("sim.b".to_string(), pool("sim.b", 0.0)),
    ]);
    let reg = Registry::flows_only(vec![Box::new(Imbalanced)], &stocks).unwrap();
    let euler = EulerIntegrator::new(reg);
    let state = State::new(0, stocks, 0, BTreeMap::new()).unwrap();
    let err = euler.step_report(&state, &SourceResolver::empty(), 1.0);
    assert!(
        matches!(err, Err(SimError::Conservation(_))),
        "expected ConservationError, got {err:?}"
    );
}

#[test]
fn rk4_overdraw_raises_arbitration_error() {
    // Scenario B under RK4: combined demand (12) exceeds the pool (10) at stage 1, and
    // RK4 hard-errors instead of rationing.
    let rk4 = Rk4Integrator::new(reg_b());
    let state = State::new(0, scenario_b_stocks(), 0, BTreeMap::new()).unwrap();
    let err = rk4.step_report(&state, &SourceResolver::empty(), DT_B);
    assert!(
        matches!(err, Err(SimError::Arbitration(_))),
        "expected ArbitrationError, got {err:?}"
    );
}
