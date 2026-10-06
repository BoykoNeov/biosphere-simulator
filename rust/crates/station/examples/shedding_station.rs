//! LAB (slice of `docs/plans/post-roadmap-leaf-shedding.md` §7): the sealed station with the
//! shedding form swapped (`domains::lab::shedding`), full horizon. Writes nothing.
//!
//! ```text
//! cargo run --release -q -p station --example shedding_station
//! ```
//! Per form: rationed / events; the crop at the end of season 1's cold weeks (day 56); grain on
//! each re-sow eve; the end state's crop and soil. `FLAT` must equal `frozen` bit for bit.
use domains::biosphere::stocks::{
    HUMUS_CARBON, LEAF_C, LITTER_CARBON, MICROBIAL_CARBON, ROOT_C, STEM_C, STEM_RESERVE_C,
    STORAGE_C,
};
use domains::lab::shedding::{
    carbon_flow, nitrogen_flow, SheddingForm, NITROGEN_SENESCENCE_ID, SENESCENCE_ID,
};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{plants_read_chamber, ChamberSurroundings};
use station::gas_exchange::GasExchangeStep;
use station::scenario::sealed_station_scenario;
use station::sealed::{
    build_sealed_station_unread, run_sealed, sealed_bio_resolver, sealed_fast_resolver,
};

fn run(form: Option<SheddingForm>) -> (Vec<State>, u64, usize) {
    let charge = domains::params::charge();
    let lamp = station::params::lamp();
    let scenario = sealed_station_scenario();
    let (state, bio_reg, fast_reg) = build_sealed_station_unread(
        &charge,
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &lamp,
        &station::params::harvest(),
        &scenario,
        false,
        false,
        GasExchangeStep::Minute,
        ChamberSurroundings::Outdoor,
    )
    .unwrap();
    let bio_reg = match form {
        None => bio_reg,
        Some(form) => {
            let p = domains::biosphere::params::biosphere();
            let (flows, aux) = bio_reg.into_parts();
            let mut found = 0;
            let flows = flows
                .into_iter()
                .map(|f| {
                    if f.id() == SENESCENCE_ID {
                        found += 1;
                        carbon_flow(form, &scenario.bio, &p)
                    } else if f.id() == NITROGEN_SENESCENCE_ID {
                        found += 1;
                        nitrogen_flow(form, &scenario.bio, &p)
                    } else {
                        f
                    }
                })
                .collect();
            assert_eq!(found, 2, "both shedding flows must be on the plant step");
            Registry::new(flows, &state.stocks, aux).unwrap()
        }
    };
    let bio_reg = plants_read_chamber(bio_reg, &state.stocks, &station::params::chamber()).unwrap();
    let bio = sealed_bio_resolver(&lamp, &scenario).unwrap();
    let fast = sealed_fast_resolver(&charge, &scenario).unwrap();
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        &scenario,
    )
    .unwrap();
    (states, rationed, events.len())
}

fn main() {
    let season = sealed_station_scenario().season_days;
    let frozen = run(None);
    let a = |s: &State, k: &str| s.stocks[k].amount;
    let crop = |s: &State| {
        [LEAF_C, STEM_C, ROOT_C, STORAGE_C, STEM_RESERVE_C]
            .iter()
            .map(|k| a(s, k))
            .sum::<f64>()
    };
    for (label, form) in [
        ("frozen", None),
        ("FLAT", Some(SheddingForm::FLAT)),
        ("L", Some(SheddingForm::LEAF)),
        ("LR", Some(SheddingForm::LEAF_ROOT)),
        ("LRS", Some(SheddingForm::ALL)),
        ("LRSpdv", Some(SheddingForm::ALL_PDV)),
    ] {
        let (states, rationed, events) = if form.is_none() {
            (frozen.0.clone(), frozen.1, frozen.2)
        } else {
            run(form)
        };
        let end = states.last().unwrap();
        let same = end == frozen.0.last().unwrap();
        println!(
            "{label:<6} rationed {rationed} events {events} | crop day 56 {:.4} | grain on re-sow eves {:?} | end: leaf {:.3} stem {:.3} root {:.3} grain {:.3} | humus {:.3} litter {:.3} microbial {:.3} | end state == frozen: {same}",
            crop(&states[56]),
            (1..=4).map(|k| (a(&states[(k * season).min(states.len() - 1)], STORAGE_C) * 1000.0).round() / 1000.0).collect::<Vec<_>>(),
            a(end, LEAF_C), a(end, STEM_C), a(end, ROOT_C), a(end, STORAGE_C),
            a(end, HUMUS_CARBON), a(end, LITTER_CARBON), a(end, MICROBIAL_CARBON),
        );
    }
}
