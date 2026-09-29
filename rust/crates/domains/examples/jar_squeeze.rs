//! Which part of the lab leaf form squeezes the sealed jar? — the diagnostic behind
//! `docs/plans/post-roadmap-leaf-rust-remeasure.md` §6, as one command:
//!
//! ```text
//! cargo run --release -q -p domains --example jar_squeeze
//! ```
//!
//! One season of the sealed jar (all five firings are in season 1), under Euler:
//!
//! 1. **Timeline** at fixed checkpoints — stored vs carbon-implied leaf area, whether the leaf
//!    envelope's clamp bound on that step and from WHICH side, and both runs' CO₂ / O₂ pools.
//! 2. **Local split at step 777** — the CO₂ DEMAND (the numerator only) of the lab state, with
//!    its stored area, CO₂ pool and O₂ pool swapped one at a time to the carbon-implied area /
//!    the frozen run's pools. Assimilation is not additive in area and CO₂, so both orders are
//!    printed.
//! 3. **Whole-run cell "capped after the cutoff"** — a wrapper around the leaf-area process,
//!    built HERE only (same aux id), that holds the stored area ≤ carbon-implied from the
//!    leaf-growth cutoff on. ⚠ A CONTROL to locate the squeeze, not a candidate: it is,
//!    structurally, the envelope retune `log/leaf-expansion.md` finding 9 refused.
//!    ⚠ **Measured UNINFORMATIVE by construction** (§6a): the cutoff falls at day 225.5 and the
//!    squeeze at days 193–197, so the cap never engages before the firings. Kept because its
//!    output prints that fact; it is not evidence either way.
//!
//! Every Euler step's firings are counted through `arbitration::scale_factors` and asserted
//! equal to the integrator's own. It writes nothing and takes no decision.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::readouts::withdrawal_demand;
use domains::biosphere::science::{self, LeafAreaForm};
use domains::biosphere::stocks::{
    CARBON_POOL, HUMUS_CARBON, LEAF_AREA_INDEX, LEAF_C, LITTER_CARBON, MICROBIAL_CARBON, O2_POOL,
    ROOT_C, STEM_C, STEM_RESERVE_C, STORAGE_C, THERMAL_TIME,
};
use domains::biosphere::system::{
    build_season_with, sealed_chamber_scenario, weather_resolver, SeasonScenario,
};
use domains::biosphere::{season_steps, BIO_DT};
use domains::lab::biosphere_with_leaf_form;
use simcore::arbitration;
use simcore::auxiliary::AuxProcess;
use simcore::environment::{Environment, SourceResolver};
use simcore::error::SimError;
use simcore::flow::FlowResult;
use simcore::integrator::{EulerIntegrator, Rk4Integrator};
use simcore::registry::Registry;
use simcore::state::State;
use std::collections::BTreeMap;

/// The squeeze's step (§5a): the reference jar's tightest, and the lab form's.
const SQUEEZE: u64 = 777;

/// The leaf-area process, capped at the carbon-implied area once thermal time reaches the
/// leaf-growth cutoff. Before the cutoff it is the lab process, untouched.
struct CappedAfterCutoff {
    inner: Box<dyn AuxProcess>,
    sla: f64,
    ground_area: f64,
}

impl AuxProcess for CappedAfterCutoff {
    fn type_name(&self) -> &'static str {
        "CappedAfterCutoff"
    }
    fn id(&self) -> &str {
        self.inner.id()
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<BTreeMap<String, f64>, SimError> {
        let mut out = self.inner.evaluate(snapshot, env, dt)?;
        let tt = snapshot.aux.get(THERMAL_TIME).copied().unwrap_or(0.0);
        if tt >= science::LEAF_TU_TLM {
            let lai = snapshot.aux[LEAF_AREA_INDEX];
            let cap =
                science::leaf_area_index(amount(snapshot, LEAF_C), self.sla, self.ground_area);
            let d = out
                .get_mut(LEAF_AREA_INDEX)
                .expect("the process advances its key");
            *d = (lai + *d).min(cap) - lai;
        }
        Ok(out)
    }
}

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map(|x| x.amount).unwrap_or(f64::NAN)
}

fn lab() -> BiosphereParams {
    biosphere_with_leaf_form(&[], LeafAreaForm::NodeEnvelope).expect("the frozen params load")
}

/// The season's registry, with the leaf-area process optionally wrapped.
fn registry(scenario: &SeasonScenario, p: &BiosphereParams, capped: bool) -> (State, Registry) {
    let (state, registry) = build_season_with(scenario, p).expect("build");
    if !capped {
        return (state, registry);
    }
    let (flows, aux) = registry.into_parts();
    let mut wrapped = 0;
    let aux: Vec<Box<dyn AuxProcess>> = aux
        .into_iter()
        .map(|a| -> Box<dyn AuxProcess> {
            if a.id() == "biosphere.leaf_area_index" {
                wrapped += 1;
                Box::new(CappedAfterCutoff {
                    inner: a,
                    sla: p.canopy.sla_per_mol_c,
                    ground_area: scenario.ground_area,
                })
            } else {
                a
            }
        })
        .collect();
    assert_eq!(wrapped, 1, "exactly one leaf-area process to wrap");
    let registry = Registry::new(flows, &state.stocks, aux).expect("re-register");
    (state, registry)
}

fn evaluate(registry: &Registry, state: &State, env: &SourceResolver) -> Vec<FlowResult> {
    let bound = env.bind(state, BIO_DT);
    registry
        .flows()
        .iter()
        .map(|f| f.evaluate(state, &bound, BIO_DT).expect("evaluate"))
        .collect()
}

fn co2_demand(registry: &Registry, state: &State, env: &SourceResolver) -> f64 {
    let results = evaluate(registry, state, env);
    withdrawal_demand(&results, &state.stocks)
        .get(CARBON_POOL)
        .copied()
        .unwrap_or(0.0)
}

/// One season under Euler: every step-entry state, plus the firing steps.
struct Run {
    states: Vec<State>,
    firings: Vec<(u64, f64)>,
    integrator: EulerIntegrator,
}

fn euler(p: &BiosphereParams, capped: bool, env: &SourceResolver) -> Run {
    let scenario = sealed_chamber_scenario();
    let (mut state, registry) = registry(&scenario, p, capped);
    let integrator = EulerIntegrator::new(registry);
    let mut states = Vec::with_capacity(season_steps() + 1);
    let mut firings = Vec::new();
    for _ in 0..season_steps() {
        let results = evaluate(integrator.registry(), &state, env);
        let factors = arbitration::scale_factors(&results, &state.stocks).expect("factors");
        let probed = factors.iter().filter(|f| **f < 1.0).count() as u64;
        let min_f = factors.iter().copied().fold(1.0, f64::min);
        let report = integrator
            .step_report(&state, env, BIO_DT)
            .expect("euler step");
        assert_eq!(
            report.rationed, probed,
            "the probe disagrees with the backstop at step {}",
            state.n
        );
        if probed > 0 {
            firings.push((state.n, min_f));
        }
        states.push(state);
        state = report.state;
    }
    states.push(state);
    Run {
        states,
        firings,
        integrator,
    }
}

fn rk4(p: &BiosphereParams, capped: bool, env: &SourceResolver) -> String {
    let scenario = sealed_chamber_scenario();
    let (mut state, registry) = registry(&scenario, p, capped);
    let integrator = Rk4Integrator::new(registry);
    for _ in 0..season_steps() {
        match integrator.step_report(&state, env, BIO_DT) {
            Ok(r) => state = r.state,
            Err(_) => {
                return format!(
                    "RAISES at step {} (day {:.2})",
                    state.n,
                    state.n as f64 * BIO_DT
                )
            }
        }
    }
    "clean".into()
}

/// Which side the envelope clamp bound on the step `s[n] → s[n+1]`, if either.
///
/// The envelope is taken on STEP-ENTRY leaf carbon (the process's own), and the next stored
/// area is compared to it with a relative tolerance of 1e-12: the process returns
/// `target − lai` and the integrator adds it back, which need not round-trip exactly.
fn clamp_side(p: &BiosphereParams, entry: &State, next: &State) -> &'static str {
    let g = sealed_chamber_scenario().ground_area;
    let (floor, ceiling) =
        science::leaf_thickness_envelope(amount(entry, LEAF_C), p.canopy.sla_per_mol_c, g);
    let a = next.aux[LEAF_AREA_INDEX];
    let near = |x: f64| (a - x).abs() <= 1e-12 * x.abs().max(1e-300);
    if near(ceiling) {
        "CEILING"
    } else if near(floor) {
        "floor"
    } else {
        "-"
    }
}

fn main() {
    let scenario = sealed_chamber_scenario();
    let env = weather_resolver(&scenario, 1).expect("resolver");
    let frozen_p = params::biosphere();
    let lab_p = lab();
    let sla = lab_p.canopy.sla_per_mol_c;
    let g = scenario.ground_area;
    let derived = |s: &State| science::leaf_area_index(amount(s, LEAF_C), sla, g);

    let frozen = euler(&frozen_p, false, &env);
    let lab_run = euler(&lab_p, false, &env);
    let capped = euler(&lab_p, true, &env);
    assert!(frozen.firings.is_empty(), "the reference jar rations");

    // --- 1. Timeline ------------------------------------------------------------------
    let tt = |s: &State| s.aux.get(THERMAL_TIME).copied().unwrap_or(0.0);
    let cutoff = lab_run
        .states
        .iter()
        .position(|s| tt(s) >= science::LEAF_TU_TLM)
        .expect("the cutoff is reached in season 1") as u64;
    let anthesis = lab_run
        .states
        .iter()
        .position(|s| tt(s) >= lab_p.pheno.tsum_anthesis)
        .map(|i| i as u64);
    let mut checkpoints: Vec<(u64, String)> = vec![
        (cutoff.saturating_sub(1), "cutoff - 1".into()),
        (cutoff, "leaf-growth cutoff".into()),
    ];
    if let Some(a) = anthesis {
        checkpoints.push((a, "anthesis".into()));
    }
    for n in [
        600u64, 625, 650, 675, 700, 725, 740, 750, 757, 769, 773, 777, 781, 789,
    ] {
        checkpoints.push((n, String::new()));
    }
    checkpoints.sort();
    checkpoints.dedup_by_key(|c| c.0);
    println!(
        "leaf-growth cutoff at step {cutoff} (day {:.2}); anthesis at step {}",
        cutoff as f64 * BIO_DT,
        anthesis
            .map(|a| a.to_string())
            .unwrap_or_else(|| "not reached".into())
    );
    println!(
        "\n== 1. timeline (lab form; stored ÷ carbon-implied area; clamp on the step n → n+1)"
    );
    println!(
        "  {:>5} {:>7}  {:>8} {:>8} {:>6} {:>8}   {:>10} {:>10}   {:>8} {:>8}  label",
        "step",
        "day",
        "stored",
        "implied",
        "ratio",
        "clamp",
        "CO2 lab",
        "CO2 froz",
        "O2 lab",
        "O2 froz"
    );
    for (n, label) in &checkpoints {
        let i = *n as usize;
        let s = &lab_run.states[i];
        let f = &frozen.states[i];
        let stored = s.aux[LEAF_AREA_INDEX];
        println!(
            "  {:>5} {:>7.2}  {:>8.5} {:>8.5} {:>6.4} {:>8}   {:>10.4e} {:>10.4e}   {:>8.4} {:>8.4}  {label}",
            n,
            *n as f64 * BIO_DT,
            stored,
            derived(s),
            stored / derived(s),
            clamp_side(&lab_p, s, &lab_run.states[i + 1]),
            amount(s, CARBON_POOL),
            amount(f, CARBON_POOL),
            amount(s, O2_POOL),
            amount(f, O2_POOL),
        );
    }
    // Clamp census over the season: how many steps each side bound, before/after the cutoff.
    let mut census: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for i in 0..season_steps() {
        let side = clamp_side(&lab_p, &lab_run.states[i], &lab_run.states[i + 1]);
        let phase = if (i as u64) < cutoff {
            "before cutoff"
        } else {
            "after cutoff"
        };
        *census.entry((phase, side)).or_insert(0) += 1;
    }
    println!("  clamp census (steps): {census:?}");
    println!("  lab firings: {:?}", lab_run.firings);

    // --- 2. Local split at the squeeze -------------------------------------------------
    let i = SQUEEZE as usize;
    let s_lab = &lab_run.states[i];
    let s_frz = &frozen.states[i];
    let swap = |area: bool, co2: bool, o2: bool| -> f64 {
        let mut s = s_lab.clone();
        if area {
            let d = derived(&s);
            s.aux.insert(LEAF_AREA_INDEX.to_string(), d);
        }
        if co2 {
            s.stocks.get_mut(CARBON_POOL).unwrap().amount = amount(s_frz, CARBON_POOL);
        }
        if o2 {
            s.stocks.get_mut(O2_POOL).unwrap().amount = amount(s_frz, O2_POOL);
        }
        co2_demand(lab_run.integrator.registry(), &s, &env)
    };
    let d_lab = swap(false, false, false);
    let d_area = swap(true, false, false);
    let d_co2 = swap(false, true, false);
    let d_both = swap(true, true, false);
    let d_all = swap(true, true, true);
    let d_frz = co2_demand(frozen.integrator.registry(), s_frz, &env);
    println!("\n== 2. CO2 demand at step {SQUEEZE} (mol per step; the numerator only)");
    println!(
        "  lab state                               {d_lab:.4e}  ({:+.1} % vs frozen)",
        pct(d_lab, d_frz)
    );
    println!(
        "  + area -> carbon-implied                {d_area:.4e}  ({:+.1} %)",
        pct(d_area, d_frz)
    );
    println!(
        "  + CO2 pool -> frozen's (area kept)      {d_co2:.4e}  ({:+.1} %)",
        pct(d_co2, d_frz)
    );
    println!(
        "  + area AND CO2 swapped                  {d_both:.4e}  ({:+.1} %)",
        pct(d_both, d_frz)
    );
    println!(
        "  + area, CO2 AND O2 swapped              {d_all:.4e}  ({:+.1} %)",
        pct(d_all, d_frz)
    );
    println!("  frozen state (frozen build)             {d_frz:.4e}");
    println!(
        "  area effect: {:+.1} % at the lab's CO2, {:+.1} % at frozen's; CO2 effect: {:+.1} % with stored area, {:+.1} % with implied",
        pct(d_area, d_lab),
        pct(d_both, d_co2),
        pct(d_co2, d_lab),
        pct(d_both, d_area),
    );
    println!(
        "  interaction (area x CO2, as a ratio): {:.4}",
        (d_both * d_lab) / (d_area * d_co2)
    );
    println!(
        "  held: lab {:.4e}, frozen {:.4e}; lab leaf C {:.4e} vs frozen {:.4e} ({:+.1} %)",
        amount(s_lab, CARBON_POOL),
        amount(s_frz, CARBON_POOL),
        amount(s_lab, LEAF_C),
        amount(s_frz, LEAF_C),
        pct(amount(s_lab, LEAF_C), amount(s_frz, LEAF_C)),
    );

    // --- 2b. Where the jar's carbon sits (mol C) — a "bigger plant" is a claim about ALL
    // of it, not the leaf alone.
    println!(
        "
== 2b. carbon ledger (mol C): lab / frozen"
    );
    for n in [650usize, 700, 757, SQUEEZE as usize] {
        let (a, b) = (&lab_run.states[n], &frozen.states[n]);
        let row = |s: &State| {
            [
                amount(s, LEAF_C),
                amount(s, STEM_C) + amount(s, STEM_RESERVE_C),
                amount(s, ROOT_C),
                amount(s, STORAGE_C),
                amount(s, LITTER_CARBON) + amount(s, MICROBIAL_CARBON) + amount(s, HUMUS_CARBON),
                amount(s, CARBON_POOL),
            ]
        };
        let (ra, rb) = (row(a), row(b));
        let plant = |r: &[f64; 6]| r[0] + r[1] + r[2] + r[3];
        println!(
            "  step {n:>4}: leaf {:.4}/{:.4}  stem+res {:.4}/{:.4}  root {:.4}/{:.4}  storage {:.4}/{:.4}  soil {:.4}/{:.4}  pool {:.4e}/{:.4e}  | plant {:.4}/{:.4} ({:+.1} %)",
            ra[0], rb[0], ra[1], rb[1], ra[2], rb[2], ra[3], rb[3], ra[4], rb[4], ra[5], rb[5],
            plant(&ra), plant(&rb), pct(plant(&ra), plant(&rb)),
        );
    }

    // --- 3. Whole-run cell ---------------------------------------------------------------
    let c = &capped.states[cutoff as usize];
    let l = &lab_run.states[cutoff as usize];
    println!("\n== 3. capped after the cutoff (a CONTROL, not a candidate)");
    println!(
        "  at the cutoff: stored {:.5} -> {:.5} on the first capped step (carbon-implied {:.5}); identical to lab before it: {}",
        l.aux[LEAF_AREA_INDEX],
        capped.states[cutoff as usize + 1].aux[LEAF_AREA_INDEX],
        derived(c),
        (0..=cutoff as usize).all(|k| {
            capped.states[k].aux == lab_run.states[k].aux
                && capped.states[k]
                    .stocks
                    .iter()
                    .all(|(id, x)| x.amount.to_bits() == lab_run.states[k].stocks[id].amount.to_bits())
        }),
    );
    let sq = &capped.states[i];
    println!(
        "  step {SQUEEZE}: CO2 held {:.4e} (lab {:.4e}, frozen {:.4e}); demand {:.4e} -> ratio {:.4}",
        amount(sq, CARBON_POOL),
        amount(s_lab, CARBON_POOL),
        amount(s_frz, CARBON_POOL),
        co2_demand(capped.integrator.registry(), sq, &env),
        co2_demand(capped.integrator.registry(), sq, &env) / amount(sq, CARBON_POOL),
    );
    println!(
        "  Euler firings {} {:?}; RK4 {}",
        capped.firings.len(),
        capped.firings,
        rk4(&lab_p, true, &env)
    );
    println!(
        "  (lab uncapped, same season: RK4 {})",
        rk4(&lab_p, false, &env)
    );
}

fn pct(a: f64, b: f64) -> f64 {
    (a / b - 1.0) * 100.0
}
