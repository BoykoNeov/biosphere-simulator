//! Why does the sealed jar break under the lab leaf form? — the control behind
//! `docs/plans/post-roadmap-leaf-rust-remeasure.md` §5, as one command:
//!
//! ```text
//! cargo run --release -q -p domains --example jar_control
//! ```
//!
//! Four cells — leaf form {`Derived`, `NodeEnvelope`} × O₂ form {`LivePool`, `Constant`} — each
//! the sealed jar driven the way its golden drives it (`run_season`, no re-sow). For every cell:
//!
//! * **Euler**: every step's flows are evaluated at the step-entry state and handed to
//!   [`arbitration::scale_factors`], the same computation the backstop applies, so each firing
//!   is NAMED — the flow, the stock it overdraws, and that stock's summed demand vs its amount.
//!   The probe's own count is checked against the integrator's `rationed`, so a probe that
//!   drifted from the backstop would say so rather than report a different run.
//! * **RK4**: the engine's error, with "flow #i" resolved to its id through the registry.
//!   ⚠ RK4 fails at a STAGE; the entry state it reports is the step's, not the stage's.
//! * **LAI**: the run's max derived and stored LAI against mutual shading's threshold.
//!
//! It writes nothing and takes no decision.

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::science::{self, LeafAreaForm, O2Form};
use domains::biosphere::stocks::{CARBON_POOL, LEAF_AREA_INDEX, LEAF_C, O2_POOL};
use domains::biosphere::system::{
    build_season_with, sealed_chamber_scenario, weather_resolver, SEALED_CHAMBER_YEARS,
};
use domains::biosphere::{season_steps, steps_for_years, BIO_DT};
use domains::lab::biosphere_with_leaf_form;
use simcore::arbitration;
use simcore::environment::SourceResolver;
use simcore::flow::FlowResult;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::registry::Registry;
use simcore::state::State;
use std::collections::BTreeMap;

fn params(leaf: LeafAreaForm, o2: O2Form) -> BiosphereParams {
    // A fresh object per cell; the O₂ form is set on the leaf-form params, never on a shared one.
    let mut p = biosphere_with_leaf_form(&[], leaf).expect("the frozen params load");
    p.photo.o2_form = o2;
    p
}

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map(|x| x.amount).unwrap_or(f64::NAN)
}

/// Every flow at `state`, in canonical order — `integrator::evaluate_all`'s body.
fn evaluate(registry: &Registry, state: &State, env: &SourceResolver) -> Vec<FlowResult> {
    let bound = env.bind(state, BIO_DT);
    registry
        .flows()
        .iter()
        .map(|f| f.evaluate(state, &bound, BIO_DT).expect("evaluate"))
        .collect()
}

/// Per-stock withdrawal demand, summed in canonical order (arbitration's own sum).
fn demand(results: &[FlowResult]) -> BTreeMap<String, f64> {
    let mut d = BTreeMap::new();
    for r in results {
        for leg in &r.legs {
            if leg.amount < 0.0 {
                *d.entry(leg.stock.clone()).or_insert(0.0) -= leg.amount;
            }
        }
    }
    d
}

struct Cell {
    firings: u64,
    lines: Vec<String>,
    max_lai_derived: f64,
    max_lai_stored: Option<f64>,
    /// The CO₂ pool's tightest step: max of (summed withdrawal demand ÷ amount held). A
    /// firing is > 1; the frozen jar's value is how close it runs to one.
    max_co2_draw: f64,
    max_co2_draw_step: u64,
    min_co2: f64,
}

fn euler(p: &BiosphereParams) -> Cell {
    let scenario = sealed_chamber_scenario();
    let (mut state, registry) = build_season_with(&scenario, p).expect("build");
    let env = weather_resolver(&scenario, SEALED_CHAMBER_YEARS).expect("resolver");
    let integrator = EulerIntegrator::new(registry);
    let sla = p.canopy.sla_per_mol_c;
    let mut cell = Cell {
        firings: 0,
        lines: Vec::new(),
        max_lai_derived: 0.0,
        max_lai_stored: None,
        max_co2_draw: 0.0,
        max_co2_draw_step: 0,
        min_co2: f64::INFINITY,
    };
    for _ in 0..steps_for_years(SEALED_CHAMBER_YEARS) {
        let derived = science::leaf_area_index(amount(&state, LEAF_C), sla, scenario.ground_area);
        let stored = state.aux.get(LEAF_AREA_INDEX).copied();
        cell.max_lai_derived = cell.max_lai_derived.max(derived);
        if let Some(s) = stored {
            cell.max_lai_stored = Some(cell.max_lai_stored.unwrap_or(0.0).max(s));
        }
        let results = evaluate(integrator.registry(), &state, &env);
        let factors = arbitration::scale_factors(&results, &state.stocks).expect("factors");
        let d = demand(&results);
        let co2 = amount(&state, CARBON_POOL);
        cell.min_co2 = cell.min_co2.min(co2);
        if let Some(dc) = d.get(CARBON_POOL) {
            if dc / co2 > cell.max_co2_draw {
                cell.max_co2_draw = dc / co2;
                cell.max_co2_draw_step = state.n;
            }
        }
        let mut probed = 0u64;
        for (i, f) in factors.iter().enumerate() {
            if *f < 1.0 {
                probed += 1;
                let flow = integrator.registry().flows()[i].id().to_string();
                let short: Vec<String> = results[i]
                    .legs
                    .iter()
                    .filter(|l| l.amount < 0.0 && d[&l.stock] > amount(&state, &l.stock))
                    .map(|l| {
                        format!(
                            "{} demand {:.4e} > held {:.4e}",
                            l.stock,
                            d[&l.stock],
                            amount(&state, &l.stock)
                        )
                    })
                    .collect();
                cell.lines.push(format!(
                    "  step {:>5} (season {}, day {:>6.2})  {flow:<34} f={f:.4}  [{}]  LAI derived {derived:.4} stored {}  CO2 {:.4e}  O2 {:.4}",
                    state.n,
                    state.n as usize / season_steps() + 1,
                    (state.n as usize % season_steps()) as f64 * BIO_DT,
                    short.join("; "),
                    stored.map(|s| format!("{s:.4}")).unwrap_or_else(|| "-".into()),
                    amount(&state, CARBON_POOL),
                    amount(&state, O2_POOL),
                ));
            }
        }
        let report = integrator
            .step_report(&state, &env, BIO_DT)
            .expect("euler step");
        assert_eq!(
            report.rationed, probed,
            "the probe disagrees with the backstop at step {} — it is not measuring the run",
            state.n
        );
        cell.firings += report.rationed;
        state = report.state;
    }
    cell
}

fn rk4(p: &BiosphereParams) -> String {
    let scenario = sealed_chamber_scenario();
    let (mut state, registry) = build_season_with(&scenario, p).expect("build");
    let env = weather_resolver(&scenario, SEALED_CHAMBER_YEARS).expect("resolver");
    let ids: Vec<String> = registry
        .flows()
        .iter()
        .map(|f| f.id().to_string())
        .collect();
    let integrator = Rk4Integrator::new(registry);
    for _ in 0..steps_for_years(SEALED_CHAMBER_YEARS) {
        match integrator.step_report(&state, &env, BIO_DT) {
            Ok(r) => state = r.state,
            Err(e) => {
                let msg = e.to_string();
                let flow = msg
                    .split("flow #")
                    .nth(1)
                    .and_then(|t| t.split_whitespace().next())
                    .and_then(|i| i.parse::<usize>().ok())
                    .map(|i| ids[i].clone())
                    .unwrap_or_else(|| "?".into());
                return format!(
                    "RAISES at step {} (season {}, day {:.2}), flow #… = {flow}; entry CO2 {:.4e} O2 {:.4}\n    {msg}",
                    state.n,
                    state.n as usize / season_steps() + 1,
                    (state.n as usize % season_steps()) as f64 * BIO_DT,
                    amount(&state, CARBON_POOL),
                    amount(&state, O2_POOL),
                );
            }
        }
    }
    "clean".into()
}

fn main() {
    let cells = [
        ("frozen x LivePool", LeafAreaForm::Derived, O2Form::LivePool),
        ("frozen x Constant", LeafAreaForm::Derived, O2Form::Constant),
        (
            "lab    x LivePool",
            LeafAreaForm::NodeEnvelope,
            O2Form::LivePool,
        ),
        (
            "lab    x Constant",
            LeafAreaForm::NodeEnvelope,
            O2Form::Constant,
        ),
    ];
    for (name, leaf, o2) in cells {
        let p = params(leaf, o2);
        let e = euler(&p);
        println!(
            "\n== {name}: Euler firings {}, CO2 pool min {:.4e} mol, tightest step draws {:.4}x it (step {}), max LAI derived {:.4} stored {}, RK4 {}",
            e.firings,
            e.min_co2,
            e.max_co2_draw,
            e.max_co2_draw_step,
            e.max_lai_derived,
            e.max_lai_stored.map(|s| format!("{s:.4}")).unwrap_or_else(|| "-".into()),
            rk4(&p)
        );
        for l in e.lines {
            println!("{l}");
        }
    }
}
