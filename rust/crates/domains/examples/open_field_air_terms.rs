//! **Why the open field's light + air water missed its prediction** — the decomposition
//! (`docs/plans/post-roadmap-open-field-night.md` §10).
//!
//! Runs the open field under light + air (the loader's since 2026-10-17) and, on every state of that
//! run, evaluates the `Transpiration` flow's crop leg (to `vapor_sink`) five ways — the run's own
//! light and lit share each time, only the air the water reads swapped:
//!
//! * `B`  — within-day temperature, within-day dryness (the run itself);
//! * `B0` — within-day temperature, no dryness (the radiation term alone);
//! * `L`  — the daily-mean temperature and dryness (light only, on this state);
//! * `L0` — the daily-mean temperature, no dryness;
//! * `M`  — the daily-mean temperature, within-day dryness (the dryness alone, Δ held).
//!
//! Night net radiation under the sun's path is 0, never negative, so Penman–Monteith's clamp at zero
//! cannot bind and `x − x0` is exactly the drying-air term. Before any factor is read: `Σ L` must
//! reproduce the separate light-only run's crop sink, and `Σ B` this run's. The same on the curve
//! shifted onto the observed mean, and under the old constant surface resistance (lab), as
//! sensitivity rows.
//!
//! ```text
//! cargo run --release -q -p domains --example open_field_air_terms
//! ```

use domains::biosphere::params;
use domains::biosphere::science::{AirTemperatureRead, SurfaceResistanceForm};
use domains::biosphere::stocks::{
    AIR_TEMP_WITHIN_DAY_VAR, RN_VAR, SOIL_EVAP_SINK, TEMP_VAR, VAPOR_SINK, VPD_VAR,
    VPD_WITHIN_DAY_VAR,
};
use domains::biosphere::system::{weather_resolver_lab, WeatherAir, WeatherRadiation};
use domains::biosphere::{
    run_season, season_setup_with, steps_for_years, BIO_DT, DEFAULT_SCENARIO,
};
use domains::lab::with_air_temperature;
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::Flow;
use simcore::state::State;

/// `base` with the water's two air names answered by `temp` / `vpd` instead.
struct Air<'a> {
    base: &'a dyn Environment,
    temp: f64,
    vpd: f64,
}

impl Environment for Air<'_> {
    fn get(&self, var: &str) -> Result<f64, SimError> {
        match var {
            AIR_TEMP_WITHIN_DAY_VAR => Ok(self.temp),
            VPD_WITHIN_DAY_VAR => Ok(self.vpd),
            _ => self.base.get(var),
        }
    }
}

/// The crop's and the soil's legs of one evaluation.
fn legs(flow: &dyn Flow, s: &State, env: &dyn Environment) -> (f64, f64) {
    let out = flow.evaluate(s, env, BIO_DT).expect("transpiration");
    let of = |id: &str| {
        out.legs
            .iter()
            .filter(|l| l.stock == id)
            .map(|l| l.amount)
            .sum::<f64>()
    };
    (of(VAPOR_SINK), of(SOIL_EVAP_SINK))
}

#[derive(Default, Clone, Copy)]
struct Sums {
    b: f64,
    b0: f64,
    l: f64,
    l0: f64,
    m: f64,
    soil_b: f64,
    soil_l: f64,
    steps: u64,
}

impl Sums {
    fn add(&mut self, o: &Sums) {
        self.b += o.b;
        self.b0 += o.b0;
        self.l += o.l;
        self.l0 += o.l0;
        self.m += o.m;
        self.soil_b += o.soil_b;
        self.soil_l += o.soil_l;
        self.steps += o.steps;
    }
}

struct Run {
    lit: Sums,
    dark: Sums,
    /// Lit steps, by the step's (1-based) month of the season: the share of the year's drying-air term.
    by_month: Vec<Sums>,
    end: State,
}

fn decompose(air: WeatherAir, rs: SurfaceResistanceForm) -> Run {
    let scenario = DEFAULT_SCENARIO;
    let mut p = with_air_temperature(params::biosphere(), AirTemperatureRead::WithinDay);
    p.transp.rs_form = rs;
    let (state, integrator, _) = season_setup_with(&scenario, 1, &p).expect("setup");
    let resolver =
        weather_resolver_lab(&scenario, 1, WeatherRadiation::SunPath, air).expect("weather");
    let light = weather_resolver_lab(
        &scenario,
        1,
        WeatherRadiation::SunPath,
        WeatherAir::DailyMean,
    )
    .expect("weather, daily-mean air");
    let steps = steps_for_years(1);
    let flows = integrator.registry().flows();
    let transpiration = flows
        .iter()
        .find(|f| f.type_name() == "Transpiration")
        .expect("one transpiration flow");
    let (mut lit, mut dark) = (Sums::default(), Sums::default());
    let steps_per_month = (30.0 / BIO_DT) as usize;
    let mut by_month = vec![Sums::default(); steps / steps_per_month + 1];
    let mut seen = 0usize;
    let mut observe = |s: &State| {
        seen += 1;
        if seen > steps {
            return; // the final state is not stepped
        }
        let env = resolver.bind(s, BIO_DT);
        let mean = light.bind(s, BIO_DT);
        let (t_day, v_day) = (
            env.get(AIR_TEMP_WITHIN_DAY_VAR).expect("air"),
            env.get(VPD_WITHIN_DAY_VAR).expect("vpd"),
        );
        let (t_mean, v_mean) = (
            mean.get(TEMP_VAR).expect("temp"),
            mean.get(VPD_VAR).expect("vpd"),
        );
        // The light the two resolvers give must be one light, or L is not light-only.
        assert_eq!(
            env.get(RN_VAR).expect("rn").to_bits(),
            mean.get(RN_VAR).expect("rn").to_bits()
        );
        let at = |temp, vpd| {
            legs(
                transpiration.as_ref(),
                s,
                &Air {
                    base: &env,
                    temp,
                    vpd,
                },
            )
        };
        let (b, soil_b) = at(t_day, v_day);
        let (l, soil_l) = at(t_mean, v_mean);
        let one = Sums {
            b,
            b0: at(t_day, 0.0).0,
            l,
            l0: at(t_mean, 0.0).0,
            m: at(t_mean, v_day).0,
            soil_b,
            soil_l,
            steps: 1,
        };
        if env.get(RN_VAR).expect("rn") > 0.0 {
            lit.add(&one);
            by_month[(seen - 1) / steps_per_month].add(&one);
        } else {
            dark.add(&one);
        }
    };
    let (end, rationed, events) = run_season(
        &integrator,
        state,
        &resolver,
        BIO_DT,
        steps,
        None,
        &mut observe,
    )
    .expect("run");
    assert_eq!(
        (rationed, events.len()),
        (0, 0),
        "a rationed or eventful run is not this subject"
    );
    Run {
        lit,
        dark,
        by_month,
        end,
    }
}

fn report(label: &str, r: &Run, light_only_crop: f64) {
    let (lit, dark) = (&r.lit, &r.dark);
    println!("== {label} ==");
    println!(
        "  the gate: Σ B {:.3} kg vs the run's crop sink {:.3}; Σ L {:.3} kg vs the light-only run's {:.3} ({:+.3} %)",
        lit.b + dark.b,
        r.end.stocks[VAPOR_SINK].amount,
        lit.l + dark.l,
        light_only_crop,
        100.0 * (lit.l + dark.l - light_only_crop) / light_only_crop,
    );
    for (name, x) in [("lit", lit), ("dark", dark)] {
        println!(
            "  {name:>4} ({} steps): B {:.3} (B0 {:.3}, air {:.3}); L {:.3} (L0 {:.3}, air {:.3}); M {:.3}; B/L {:.4}; soil B {:.3}, L {:.3}",
            x.steps,
            x.b,
            x.b0,
            x.b - x.b0,
            x.l,
            x.l0,
            x.l - x.l0,
            x.m,
            x.b / x.l,
            x.soil_b,
            x.soil_l,
        );
    }
    let (a_l, a_m, a_b) = (lit.l - lit.l0, lit.m - lit.l0, lit.b - lit.b0);
    let share = a_l / lit.l;
    let ratio = a_m / a_l;
    let delta_rad = lit.b0 / lit.l0;
    let delta_air = a_b / a_m;
    println!("  lit crop, light only → light + air:");
    println!("    drying-air share of light-only water  A_L/L   {share:.4}");
    println!("    dryness ratio, Δ held (coefficient-wtd) A_M/A_L {ratio:.4}");
    println!("    radiation term under the warmer air    B0/L0   {delta_rad:.4}");
    println!("    drying-air term, Δ's denominator       A_B/A_M {delta_air:.4}");
    println!(
        "    composed (1−share)·B0/L0 + share·ratio·Δ_air = {:.4}  (measured B/L {:.4})",
        (1.0 - share) * delta_rad + share * ratio * delta_air,
        lit.b / lit.l,
    );
    println!(
        "    the estimate's arithmetic at this share and ratio, Δ ignored: {:.4}; at share 0.5: {:.4}",
        (1.0 - share) + share * ratio,
        0.5 + 0.5 * ratio,
    );
    println!("  lit crop by month of the season (light-only water, its drying-air share, the month's dryness ratio):");
    for (i, m) in r.by_month.iter().enumerate().filter(|(_, m)| m.steps > 0) {
        let a = m.l - m.l0;
        println!(
            "    month {:>2}: L {:8.3} kg  share {:.3}  ratio {}",
            i + 1,
            m.l,
            if m.l > 0.0 { a / m.l } else { f64::NAN },
            if a > 0.0 {
                format!("{:.3}", (m.m - m.l0) / a)
            } else {
                "—".to_string()
            },
        );
    }
    println!();
}

/// The separate light-only run's crop sink, the gate's other side.
fn light_only_crop(rs: SurfaceResistanceForm) -> f64 {
    let scenario = DEFAULT_SCENARIO;
    let mut p = with_air_temperature(params::biosphere(), AirTemperatureRead::DailyMean);
    p.transp.rs_form = rs;
    let (state, integrator, _) = season_setup_with(&scenario, 1, &p).expect("setup");
    let resolver = weather_resolver_lab(
        &scenario,
        1,
        WeatherRadiation::SunPath,
        WeatherAir::DailyMean,
    )
    .expect("weather");
    let (end, _, _) = run_season(
        &integrator,
        state,
        &resolver,
        BIO_DT,
        steps_for_years(1),
        None,
        &mut |_: &State| {},
    )
    .expect("run");
    end.stocks[VAPOR_SINK].amount
}

fn main() {
    let loader = params::biosphere().transp.rs_form;
    let light_only = light_only_crop(loader);
    report(
        "open field, light + air (as printed)",
        &decompose(WeatherAir::WithinDay, loader),
        light_only,
    );
    report(
        "open field, light + air (shifted onto the observed mean)",
        &decompose(WeatherAir::WithinDayShifted, loader),
        light_only,
    );
    // The canopy's resistance cancels from the share within a step, but it decides which steps carry
    // the season's water (plan §10b): the same decomposition under the old constant 70 s/m.
    report(
        "open field, light + air (as printed), the constant surface resistance (lab)",
        &decompose(WeatherAir::WithinDay, SurfaceResistanceForm::Constant),
        light_only_crop(SurfaceResistanceForm::Constant),
    );
}
