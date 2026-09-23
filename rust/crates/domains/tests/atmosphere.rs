//! **The chamber has an atmosphere** — total gas conserved, pressure derived.
//!
//! Plan: `docs/plans/post-roadmap-atmosphere.md`. Predecessor:
//! `tests/gas_composition_perturbations.rs`, whose eleven probes were built to be the
//! instrument that measures this work.
//!
//! # What this file is guarding
//!
//! Before 2026-09-08 the sealed chamber held CO₂ and O₂ and nothing else. The biology divided
//! each by `chamber_air_mol`, a **constant carried on the flow**, and the consequence was
//! recorded twice and tested never: *nothing in this model could lose an atmosphere.* There
//! was no inert gas, so a leak could only ever take one species; there was no total, so there
//! was no pressure.
//!
//! ⚠ **The fix is NOT "make the denominator live", and getting that backwards is the hazard
//! this file exists against.** At fixed V and T the ideal gas law gives `p_i = n_i·R·T/V`, so
//! a species' partial pressure is proportional to *its own* mole count and the ratio the
//! frozen FvCB constants are calibrated against is `p_i/P_ref = n_i/n_ref` — the room's
//! **reference** fill in the denominator, which is exactly what the old constant already was.
//! `n_i/n_total_live` is the mole *fraction*, and it does not move when a chamber
//! depressurizes at fixed composition, so a leaf reading it would be **blind to a hull
//! breach**. The name changed (`chamber_air_capacity_mol`) and the arithmetic did not.
//!
//! ⚠ And no golden could have caught the substitution: at charge `n_total == n_ref`, so both
//! denominators read identically. That is why
//! [`the_capacity_and_the_live_total_are_the_same_number_at_charge_and_only_there`] is
//! written as an explicit **contrast** rather than as a value pin.

use domains::biosphere::params;
use domains::biosphere::readouts::{
    dry_gas_mol, pressure_ratio, total_gas_mol, trajectory, Trajectory,
};
use domains::biosphere::science::{H2O_MOLAR_MASS_KG_PER_MOL, N2_MOLAR_MASS_KG_PER_MOL};
use domains::biosphere::system::{
    chamber_inert_charge_mol, consumer_chamber_scenario, perennial_chamber_scenario,
    sealed_chamber_scenario, SeasonScenario, CONSUMER_CHAMBER_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};

/// The three frozen sealed chambers, each driven the way its own golden drives it.
///
/// ⚠ The `perennial` flag is not cosmetic: `sealed_chamber` runs through `run_season` with no
/// re-sow and the other two through `run_perennial`. A probe that drove one of them the other
/// way would be measuring a run no golden pins.
fn chambers() -> Vec<(&'static str, SeasonScenario, usize, bool)> {
    vec![
        (
            "sealed_chamber",
            sealed_chamber_scenario(),
            SEALED_CHAMBER_YEARS,
            false,
        ),
        (
            "perennial_chamber",
            perennial_chamber_scenario(),
            PERENNIAL_CHAMBER_YEARS,
            true,
        ),
        (
            "consumer_chamber",
            consumer_chamber_scenario(),
            CONSUMER_CHAMBER_YEARS,
            true,
        ),
    ]
}

fn run(s: SeasonScenario, years: usize, perennial: bool) -> Trajectory {
    trajectory(s, years, perennial, &params::biosphere())
}

// ===================================================================================
// The charge — the room starts full, by construction rather than by tuning
// ===================================================================================

/// The inert charge is the room's remainder, so every chamber starts at **exactly** reference
/// pressure — `1.0`, bit-for-bit, not "within a tolerance".
///
/// This is the property that makes the charge derivable at all. Had the inert fill been a
/// scenario field someone typed, each chamber would start at whatever pressure that number
/// happened to imply, every one of them would need a citation, and none of them would be
/// exactly 1.0.
///
/// ⚠ `assert_eq!` on a float, deliberately. `capacity - co2 - o2 + co2 + o2` need not
/// round-trip in general, and if it does not, this is the test that says so rather than a
/// tolerance quietly absorbing it. Measured exact for all three frozen chambers.
#[test]
fn every_sealed_chamber_starts_at_exactly_reference_pressure() {
    for (name, scenario, years, perennial) in chambers() {
        let t = run(scenario, years, perennial);
        let p = pressure_ratio(&t);
        assert_eq!(
            p[0], 1.0,
            "{name}: starts at pressure {:?}, not 1.0 — the inert charge is not the room's \
             remainder",
            p[0]
        );
    }
}

/// The derived charge, against the plan's own table — one arithmetic pin so a sign error in
/// `chamber_inert_charge_mol` cannot hide behind the pressure being 1.0 either way.
///
/// ⚠ The pressure test above is **not** a substitute: it divides the total by the capacity,
/// and a charge computed as `capacity - co2 - o2` and a charge computed as
/// `capacity + co2 + o2` would both give a total of `capacity ± 2(co2+o2)` — only one of which
/// is 1.0, but neither of which pins that the *inert* term is the remainder rather than, say,
/// the whole capacity with the species double-counted. This one reads the charge itself.
#[test]
fn the_inert_charge_is_the_rooms_remainder() {
    let cases = [
        (sealed_chamber_scenario(), 1000.0 - 0.357 - 2.0),
        (perennial_chamber_scenario(), 1000.0 - 0.357 - 210.0),
        (consumer_chamber_scenario(), 2000.0 - 0.714 - 420.0),
    ];
    for (scenario, expected) in cases {
        assert_eq!(chamber_inert_charge_mol(&scenario).unwrap(), expected);
    }
}

/// A chamber whose named gases exceed its own room is **malformed**, and says so.
///
/// ⚠ Not a clamp. A clamp would silently give such a scenario a zero inert fill and a
/// pressure below 1.0 that looks like a modelling choice — and *"a clamp hides a wrong
/// amount ... it survives until the scale changes"* is a lesson this repo has already paid
/// for once.
#[test]
fn a_chamber_whose_gases_exceed_its_room_is_rejected() {
    let bad = SeasonScenario {
        chamber_o2_mol0: 2000.0,
        ..sealed_chamber_scenario()
    };
    assert!(chamber_inert_charge_mol(&bad).is_err());
    // ⚠ The boundary case is admitted, not rejected: a chamber of pure CO₂ + O₂ has no inert
    // fill and is perfectly well-posed. Without this the guard could be an off-by-one that
    // the test above would pass anyway.
    let exact = SeasonScenario {
        chamber_co2_mol0: 0.0,
        chamber_o2_mol0: 1000.0,
        ..sealed_chamber_scenario()
    };
    assert_eq!(chamber_inert_charge_mol(&exact).unwrap(), 0.0);
}

// ===================================================================================
// Sufficiency — why ONE inert stock is enough, and not a total-gas stock
// ===================================================================================

/// **The claim that makes an inert stock sufficient.** Every gas exchange in this chamber is
/// one mole for one mole — photosynthesis takes a CO₂ and returns an O₂
/// (`CO₂ + H₂O → CH₂O + O₂`), and every respiration in the tree runs it backwards at the same
/// stoichiometry (PQ = 1). So `n_CO₂ + n_O₂` is **conserved by the biology itself**.
///
/// That is why the atmosphere needed one inert term and not a total-gas stock: with the
/// reactive pair self-cancelling and the inert fill written by nothing, the only term that can
/// move the total is water vapour. Had this failed, a stock would genuinely have been needed
/// and the design would have been wrong.
///
/// ⚠ Tolerance is relative to the pair's own magnitude and is a real one — these are sums of
/// thousands of Euler steps, not a closed form. A *drift* would grow with the run; this
/// asserts the whole trajectory against its own start, so a systematic leak of either gas
/// shows up as the horizon lengthens rather than being averaged away.
#[test]
fn the_reactive_gas_pair_is_conserved_because_every_exchange_is_one_for_one() {
    for (name, scenario, years, perennial) in chambers() {
        let t = run(scenario, years, perennial);
        let pair = |i: usize| t.carbon_pool[i] + t.o2_pool[i];
        let start = pair(0);
        let worst = (0..t.carbon_pool.len())
            .map(|i| (pair(i) - start).abs() / start)
            .fold(0.0_f64, f64::max);
        assert!(
            worst <= 1.0e-12,
            "{name}: CO2+O2 drifted {worst:e} relative from its charge — the reactive pair is \
             NOT self-cancelling, and an inert stock alone is then not sufficient"
        );
    }
}

/// The inert fill is written by **no flow in this tree**, so it is constant to the bit.
///
/// ⚠ This is the test that would redden if someone wired nitrogen fixation, a hull breach, or
/// an inert make-up into the nominal build without saying so — and it is deliberately
/// `assert_eq!` rather than a tolerance, because "constant" here is a structural claim about
/// the flow list, not a numerical one about a rate that happens to be small.
#[test]
fn nothing_in_the_nominal_tree_writes_the_inert_fill() {
    for (name, scenario, years, perennial) in chambers() {
        let t = run(scenario, years, perennial);
        assert!(!t.inert_kg.is_empty(), "{name}: no inert series");
        for (i, kg) in t.inert_kg.iter().enumerate() {
            assert_eq!(
                *kg, t.inert_kg[0],
                "{name}: the inert fill moved at step {i} — some flow now writes it"
            );
        }
    }
}

// ===================================================================================
// The trap — the capacity and the live total agree at charge and ONLY at charge
// ===================================================================================

/// **The contrast the whole design rests on.** `chamber_air_capacity_mol` (the room) and the
/// live total gas are the same number at t = 0 and different afterwards.
///
/// ⚠ This is why no golden could catch a future "fix" that re-points the science's denominator
/// at the live total: at charge the two are identical, so every frozen run reads the same under
/// either. This test refuses to let that stay true by pinning that they DO separate — if they
/// never separated, the distinction would be untestable and the rename would be decoration.
///
/// The separation is water vapour: it starts at zero in every scenario and transpiration fills
/// it, so the live total rises above the capacity during a run. The direction is asserted; the
/// magnitude is a measurement and lives in the plan's §3, not here.
#[test]
fn the_capacity_and_the_live_total_are_the_same_number_at_charge_and_only_there() {
    for (name, scenario, years, perennial) in chambers() {
        let t = run(scenario, years, perennial);
        let total = total_gas_mol(&t);
        let capacity = t.scenario.chamber_air_capacity_mol;
        assert_eq!(
            total[0], capacity,
            "{name}: the live total and the capacity disagree at charge"
        );
        let peak = total.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            peak > capacity,
            "{name}: the live total NEVER leaves the capacity (peak {peak}, capacity \
             {capacity}) — the two are then indistinguishable on every run this suite has, \
             and the denominator distinction is untestable"
        );
    }
}

/// The fold is the sum of the four species in moles — asserted against the stocks it reads,
/// not against a second copy of the arithmetic.
///
/// ⚠ Written as an independent recomposition from the *series* rather than by calling
/// `total_gas_mol` twice: the failure it is against is a dropped term (the vapour, or the
/// molar-mass division), and a reimplementation that shares the bug catches nothing. The two
/// unit conversions are the part most likely to go wrong and are the part spelled out here.
#[test]
fn the_total_is_the_four_species_each_converted_to_moles() {
    let t = run(sealed_chamber_scenario(), SEALED_CHAMBER_YEARS, false);
    let total = total_gas_mol(&t);
    for i in [0, t.carbon_pool.len() / 2, t.carbon_pool.len() - 1] {
        let by_hand = t.carbon_pool[i]
            + t.o2_pool[i]
            + t.inert_kg[i] / N2_MOLAR_MASS_KG_PER_MOL
            + t.water_vapor_kg[i] / H2O_MOLAR_MASS_KG_PER_MOL;
        assert_eq!(total[i], by_hand, "the total disagrees at step {i}");
    }
    // The vapour term is the one that can be silently dropped — it is zero at charge, so a
    // fold that omitted it would still pass the t=0 pin above. Assert it is non-zero
    // somewhere, or this test is inert on its own subject.
    assert!(
        t.water_vapor_kg.iter().any(|kg| *kg > 0.0),
        "the chamber never holds vapour, so the vapour term of the fold is untested here"
    );
}

/// Pressure is `total / capacity` and nothing else — no gas constant, no volume, no
/// temperature, because at fixed V and T they all cancel.
#[test]
fn pressure_is_the_total_over_the_capacity() {
    let t = run(perennial_chamber_scenario(), PERENNIAL_CHAMBER_YEARS, true);
    let total = total_gas_mol(&t);
    let p = pressure_ratio(&t);
    let capacity = t.scenario.chamber_air_capacity_mol;
    for i in [0, total.len() / 2, total.len() - 1] {
        assert_eq!(p[i], total[i] / capacity);
    }
}

// ===================================================================================
// What the fold measured — two findings, recorded as tests rather than as prose
// ===================================================================================

/// **The DRY chamber sits at exactly reference pressure for the whole run.**
///
/// Not by tuning: the reactive pair is one-for-one
/// ([`the_reactive_gas_pair_is_conserved_because_every_exchange_is_one_for_one`]) and the inert
/// fill is written by nothing, so the only dry term that could move the total is one that does
/// not exist.
///
/// ⚠ **Exact at charge, exact to rounding afterwards** — and the distinction is measured, not
/// hedged. `capacity - co2 - o2 + co2 + o2` round-trips bit-for-bit at t = 0 (which is what
/// [`every_sealed_chamber_starts_at_exactly_reference_pressure`] asserts with `assert_eq!`),
/// but after a few thousand Euler steps the pair's two halves have rounded independently. The
/// drift **accumulates with the horizon**, which is the honest thing to say about it: 1 ULP by
/// step 116 of the 5-year perennial run, ~1e-12 relative by step 3186. The tolerance below is
/// the same 1e-12 its sibling
/// [`the_reactive_gas_pair_is_conserved_because_every_exchange_is_one_for_one`] uses, and for
/// the same reason — it is a floor set by the integrator, not a margin chosen to pass.
///
/// ⚠ The consequence is worth being explicit about, because it bounds what this model can say:
/// **a sealed chamber's dry pressure is structurally incapable of drifting here.** A slow
/// pressure loss — the thing a real habitat's leak-rate budget is about — has no representation
/// in the nominal tree at all. It takes an explicit breach (`tests/atmosphere_breach.rs`).
#[test]
fn the_dry_chamber_never_leaves_reference_pressure() {
    for (name, scenario, years, perennial) in chambers() {
        let t = run(scenario, years, perennial);
        let capacity = t.scenario.chamber_air_capacity_mol;
        for (i, dry) in dry_gas_mol(&t).into_iter().enumerate() {
            assert!(
                (dry - capacity).abs() <= 1.0e-12 * capacity,
                "{name}: dry gas left the capacity at step {i} ({dry} vs {capacity}) — that is                  above the rounding floor, so something dry is moving"
            );
        }
    }
}

/// **The chamber's water vapour obeys saturation** — the guard that replaced a tripwire.
///
/// # What stood here, and why it was deleted rather than re-thresholded
///
/// Until 2026-09-23 this slot held a **tripwire**: a test designed to go red when the water
/// model gained a saturation bound. It measured the defect the atmosphere work exposed — all
/// three chambers peaked at the **same 536.995 mol** of vapour in rooms of 1000 and 2000 mol,
/// wet pressure 1.537 against a saturation-implied ≈1.023, because transpiration filled the air
/// with no regard for whether it could hold the water. The fix
/// (`docs/plans/post-roadmap-vapour-saturation.md`) turned it red, and it was deleted as its
/// own text instructed.
///
/// ⚠ **Only ONE of its two assertions fired, and the silent one was mis-set.** Its
/// room-independence half went red, as designed. Its `peak > 1.023` half stayed GREEN on the
/// corrected model, because 1.023 is saturation at **20 °C** and the weather's warmest day is
/// warmer: the saturated jar peaks at 1.0235. A tripwire whose threshold is a single
/// temperature's ceiling cannot tell "saturated on a warm day" from "unbounded" by a margin of
/// 5e-4. The half that caught the fix was the one that asked about the SHAPE (does vapour
/// scale with the room?), not the one that asked about a VALUE.
///
/// # What this asserts instead
///
/// 1. **The bound, per step, at that step's own temperature** — never at a fixed 20 °C, for
///    exactly the reason above. `vapour(n+1) ≤ e_s(T_n)/P_std · n_ref`, with `T_n` the
///    temperature step `n` ran at (a day boundary may lower the NEXT cap; the next step's
///    condensation removes that excess).
/// 2. **The vapour now belongs to the room**: the 2000-mol consumer chamber holds twice the
///    sealed jar's peak, to the bit. That is the tripwire's live half, inverted.
#[test]
fn the_chamber_vapour_never_exceeds_saturation_and_scales_with_the_room() {
    use domains::biosphere::science::saturation_vapour_kg;
    use domains::biosphere::{weather, STEPS_PER_DAY};
    let dt = 1.0 / STEPS_PER_DAY as f64;
    let mut peak_vapour: Vec<(&str, f64, f64)> = Vec::new();
    for (name, scenario, years, perennial) in chambers() {
        let t = run(scenario, years, perennial);
        let capacity = t.scenario.chamber_air_capacity_mol;
        let (latitude, rows) = weather::weather_facts();
        let temp = weather::season_forcing(latitude, &rows, years).temp;
        let v = &t.water_vapor_kg;
        assert!(v.iter().any(|kg| *kg > 0.0), "{name}: no vapour — the bound is untested");
        for n in 0..v.len() - 1 {
            let day = ((n as f64 * dt) as usize).min(temp.len() - 1);
            let cap = saturation_vapour_kg(temp[day], capacity);
            assert!(
                v[n + 1] <= cap * (1.0 + 1.0e-12),
                "{name}: step {n} ended with {} kg of vapour above saturation {cap} kg at {} °C",
                v[n + 1],
                temp[day]
            );
        }
        let peak = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        peak_vapour.push((name, capacity, peak));
    }
    let (_, ref_room, ref_peak) = peak_vapour[0];
    for (name, room, peak) in &peak_vapour {
        let want = ref_peak * room / ref_room;
        assert!(
            (peak - want).abs() <= 1.0e-12 * want,
            "{name}: peak vapour {peak} kg is not the room-scaled {want} kg: {peak_vapour:?}"
        );
    }
}
