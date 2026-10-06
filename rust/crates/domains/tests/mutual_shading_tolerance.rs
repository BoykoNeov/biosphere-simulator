//! **How much of the open field's science bands is the mutual-shading loss holding up?**
//!
//! `docs/log/temperature-kinetics.md` FINDING 5 measured the Van Keulen & Seligman 5 %/day
//! loss as **exactly inert** in the frozen tree (peak LAI 6.022837 with the term on and with
//! `shade_rate = 0`, bit-identical) while it absorbed 7.31 of LAI under an alternative
//! kinetics form. That leaves a question about the *contract* rather than about any candidate
//! science: a mechanism that does nothing at the frozen params is nonetheless the thing that
//! decides what the bands can detect once anything moves.
//!
//! ⚠ **Re-measured at the 1/16-day step, 2026-09-30** (`docs/plans/post-roadmap-step-sixteenth.md`).
//! The frozen canopy no longer reaches the threshold (5.4406), so the loss is inert on both
//! observables; the ceiling's absorption held (1.74, was 1.77); the cap's fell (2.1, was 3.3)
//! and its crest overshoot rose (0.97 %, was 0.13 %). Each test carries both numbers. The
//! prediction table below is the quarter-day record, left as written.
//!
//! ⚠⚠ **RE-MEASURED 2026-10-06 for the leaf-shedding unfreeze, and the picture INVERTED**
//! (`docs/plans/post-roadmap-leaf-shedding.md` §10–§12). With no leaf shed from age before
//! anthesis the frozen canopy reaches the threshold (peak LAI 6.177) and the loss is what holds
//! it there — 14.825 with it off. So the loss is no longer a mechanism that "does nothing at the
//! frozen params": it is the canopy's regulator, as the July (C) diagnosis predicted. Measured
//! on this tree: the LAI ceiling crossed at ×1.616 with the loss on and ×0.719 with it off; the
//! biomass cap NEVER crossed on this axis with the loss on (peak W crests at 14.186, ×3, 1.65 %
//! under the cap) and at ×0.760 with it off. Each test below carries the new numbers; the
//! history stays as written.
//!
//! This file measures that, on **one knob** — `specific_leaf_area`, the linear carbon→area
//! conversion (`require_positive`, no upper bound) — swept with the loss ON and with
//! `shade_rate = 0.0`. The number that answers it is the SLA multiplier at which each bound
//! is crossed in each arm; the ratio is how much error the loss absorbs.
//!
//! ⚠ **`specific_leaf_area` and not `quantum_yield`**, which was the other candidate: the
//! latter is capped at `require_half_open(0.0, 1.0)` (×3.33 from frozen) *and* changes which
//! photosynthetic branch limits as it rises (`log/temperature-kinetics.md` FINDING 2/8:
//! Rubisco-bound lit steps 1.1 % → 9.1 % under a comparable supply gain). That confound has
//! nothing to do with shading and would have to be unpicked afterwards.
//!
//! # ⚠ This endorses nothing and proposes nothing
//!
//! Every column is a substitution rewritten in memory by [`domains::lab`]; no param file,
//! golden, manifest digest or gate bound can move. `specific_leaf_area` is **cited** ([B]
//! Table 19 p.100) and is not under review here — it is the instrument, chosen because it
//! addresses the observable directly.
//!
//! # The predictions, written before the ladder was run, and scored
//!
//! Recorded in `M:/claud_projects/temp/sla-ladder/PREDICTION.md` before the first run:
//!
//! | # | predicted | measured | |
//! |---|---|---|---|
//! | 1 | loss-OFF peak LAI crosses 8.0 at ×1.25–1.35 | **×1.12–1.14** | ❌ low by ~2 rungs |
//! | 2 | loss-ON crosses 8.0 at ×1.6–2.2, *not never* | **×2.00–2.05** | ✅ |
//! | 3 | the biomass cap breaks FIRST, at ×1.1–1.2 | **×3.8–4.0, after the LAI ceiling** | ❌ |
//! | 4 | the chambers reach `< 1.0` at about ×1.6 | **between ×2.5 and ×3.0** | ❌ low |
//! | 5 | the recorded SLA span's high end was measured with no loss | **confirmed by git** | ✅ |
//!
//! **Prediction 3 is the interesting miss.** It was reasoned from the frozen headroom to the
//! cap (7.8 %) without asking whether `peak W` *saturates* — it does, at ~14.4435, which is
//! 0.13 % above the recorded 14.4248 cap. So with the loss modelled the cap is not merely
//! late, it is barely reachable at all in this direction.

use domains::biosphere::params::BiosphereParams;
use domains::biosphere::readouts::{peak_lai, peak_w, trajectory};
use domains::biosphere::science_gates::VKS_LAI_THRESHOLD;
use domains::biosphere::system::{
    consumer_chamber_scenario, perennial_chamber_scenario, sealed_chamber_scenario,
    CONSUMER_CHAMBER_YEARS, DEFAULT_SCENARIO, PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS,
};
use domains::lab::{biosphere_with, Substitution};

/// `canopy.yaml`, the cited value the ladder is expressed as a multiple of.
const FROZEN_SLA: f64 = 23.53;

/// `open_season_canopy_is_physical`'s upper bound, `5.0 < peak < 8.0`.
const LAI_CEILING: f64 = 8.0;

/// The above-ground biomass cap, `peak_w < 14.4248` (the Greenwood point).
const W_CAP: f64 = 14.4248;

/// The frozen params with SLA scaled, and the loss optionally switched off.
///
/// ⚠ `shade_rate = 0.0` is *inside* the frozen bound (`require_non_negative`), so the OFF arm
/// is an ordinary substitution the loader accepts — not a monkeypatch and not a widened bound.
fn params(mult: f64, shading: bool) -> BiosphereParams {
    let mut subs =
        vec![Substitution::resolve("specific_leaf_area", FROZEN_SLA * mult).expect("one owner")];
    if !shading {
        subs.push(Substitution::resolve("shade_rate", 0.0).expect("one owner"));
    }
    biosphere_with(&subs).expect("the ladder stays inside every frozen bound")
}

/// `open_season`'s two gated quantities at one rung: `(peak LAI, peak W t/ha)`.
fn open_field(mult: f64, shading: bool) -> (f64, f64) {
    let t = trajectory(DEFAULT_SCENARIO, 1, false, &params(mult, shading));
    assert_eq!(t.rationed, 0, "a rationed run is not a band measurement");
    (peak_lai(&t), peak_w(&t))
}

/// `open_season`'s peak W at an SLA rung with one further substitution applied.
fn peak_w_with(mult: f64, field: &str, value: f64) -> f64 {
    let subs = [
        Substitution::resolve("specific_leaf_area", FROZEN_SLA * mult).expect("one owner"),
        Substitution::resolve(field, value).expect("one owner"),
    ];
    let p = biosphere_with(&subs).expect("inside every frozen bound");
    peak_w(&trajectory(DEFAULT_SCENARIO, 1, false, &p))
}

/// **Why `peak W` saturates — measured, because a causal claim earns the experiment that
/// removes the cause.**
///
/// The crest invites a nitrogen reading (`nitrogen.yaml`: the flat `n_critical` threshold and
/// the Greenwood curve "[coincide] only at W ≈ 14.44 t/ha"). **It is not nitrogen.** At the
/// crest, dropping `n_critical` to 0.010 and doubling `max_uptake_capacity` each leave `peak W`
/// **bit-identical**: `f_N` is not biting at all.
///
/// **It is light interception.** Cutting `extinction_coef` 0.60 → 0.45 costs the crest nothing
/// (it gains). ⚠ Re-measured 2026-10-06: the frozen canopy itself is now close to saturation
/// (peak LAI 6.18, held by the shading loss), so the cut costs it only **3.1 %** (it cost 25.1 %
/// when the frozen canopy peaked at 5.44) and the crest **gains 1.9 %**. The contrast that
/// carried this test — frozen costly, crest free — has mostly closed, because the frozen tree
/// moved toward the crest; the nitrogen half is unchanged.
#[test]
fn the_peak_w_crest_is_light_saturation_and_not_nitrogen() {
    const CREST: f64 = 4.50;
    let crest = open_field(CREST, true).1;

    for (field, value) in [("n_critical", 0.010), ("max_uptake_capacity", 0.0030)] {
        let moved = peak_w_with(CREST, field, value);
        assert_eq!(
            moved, crest,
            "{field} must leave the crest BIT-identical — {moved} vs {crest}"
        );
    }

    let frozen = open_field(1.0, true).1;
    let frozen_dim = peak_w_with(1.0, "extinction_coef", 0.45);
    let crest_dim = peak_w_with(CREST, "extinction_coef", 0.45);
    let cost_at_frozen = (frozen - frozen_dim) / frozen;
    let cost_at_crest = (crest - crest_dim) / crest;
    assert!(
        (0.02..0.045).contains(&cost_at_frozen),
        "a 25 % cut in k costs the near-saturated frozen canopy a little (measured 0.0310) — \
         {cost_at_frozen}"
    );
    assert!(
        cost_at_crest < 0.0,
        "...and costs the crest nothing — it gains (measured -0.0192) — {cost_at_crest}"
    );
}

/// The SLA multiplier at which `read` crosses `bound` within `[lo, hi]`, bisected to ~0.1 %.
///
/// ⚠ `lo` became an argument 2026-10-06: with the loss OFF the frozen rung (×1) is already over
/// both bounds, so those crossings lie BELOW ×1 and are bracketed from under it.
///
/// ⚠ **Bisection assumes the observable is monotone on `[1.0, hi]`, and one of the four
/// crossings is not monotone beyond its bracket** — `peak W` with the loss on crests near ×4.5
/// and falls away. So `hi` is a real argument, not a convenience: it is where each ladder's
/// monotone stretch ends. The two end checks below are what make a wrong `hi` a failure rather
/// than a plausible number.
fn crossing(
    shading: bool,
    bound: f64,
    read: fn((f64, f64)) -> f64,
    lo: f64,
    hi: f64,
) -> f64 {
    let at = |m: f64| read(open_field(m, shading));
    let (mut lo, mut hi) = (lo, hi);
    assert!(at(lo) < bound, "the bracket's bottom must be under {bound}");
    assert!(at(hi) > bound, "the bracket's top must be over {bound}");
    for _ in 0..10 {
        let mid = 0.5 * (lo + hi);
        if at(mid) < bound {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// FINDING 5's claim re-derived — **and inverted by the leaf-shedding unfreeze**.
///
/// ⚠ At the quarter-day step the loss was bit-identically inert on `peak LAI` and live on
/// `peak W`; at the 1/16-day step (2026-09-30) inert on both (the canopy peaked at 5.44, under
/// the threshold). **Since 2026-10-06 it is LIVE on both at the frozen params**: with no leaf
/// shed from age before anthesis the canopy reaches 6.177 with the loss on and 14.825 with it
/// off; peak W 13.888 against 15.408 (a 10.9 % gap). The cited V-K&S loss is now the frozen
/// canopy's regulator, the role July's (C) diagnosis said the flat `rdr_leaf` had been playing.
#[test]
fn the_loss_is_live_on_both_observables_at_the_frozen_params() {
    let (lai_on, w_on) = open_field(1.0, true);
    let (lai_off, w_off) = open_field(1.0, false);
    assert!(
        lai_on > 6.0,
        "the frozen canopy must reach the 6.0 threshold — {lai_on}"
    );
    assert!(
        lai_off - lai_on > 8.0,
        "peak LAI: {lai_on} on vs {lai_off} off (measured 6.177 vs 14.825)"
    );
    let rel = (w_off - w_on) / w_on;
    assert!(
        (0.09..0.13).contains(&rel),
        "peak W gap {rel} (measured 0.1094: {w_on} on, {w_off} off)"
    );
}

/// **THE HEADLINE**, re-measured 2026-10-06: without the loss the frozen canopy is ALREADY
/// outside the `5.0 < peak < 8.0` band; with it, the ceiling absorbs a ×1.6 error.
///
/// History: crossings of `LAI_CEILING` were loss OFF ×1.12–1.14 / ON ×2.00–2.05 at the
/// quarter-day step, ×1.181 / ×2.058 at 1/16. With no leaf shed from age before anthesis they
/// are **×0.719 off** (below the frozen params: the loss is what keeps the frozen canopy inside
/// the band) and **×1.616 on**, a factor of 2.25.
#[test]
fn the_loss_roughly_doubles_the_sla_error_the_lai_ceiling_absorbs() {
    let off = crossing(false, LAI_CEILING, |q| q.0, 0.3, 1.0);
    let on = crossing(true, LAI_CEILING, |q| q.0, 1.0, 3.0);

    // ⚠ The two crossings are asserted as absolute facts BEFORE their ratio, because a ratio
    // alone would be satisfied by both arms moving together — which is exactly what a mutation
    // that disables the loss produces.
    assert!(
        (0.68..0.76).contains(&off),
        "loss-OFF crossing (measured x0.719) — {off}"
    );
    assert!(
        (1.55..1.70).contains(&on),
        "loss-ON crossing (measured x1.616) — {on}"
    );
    assert!(
        on / off > 2.0,
        "the loss must absorb at least 2x the SLA error — {on} / {off}"
    );
}

/// The biomass cap: **with the loss modelled it cannot be reached on this axis at all**
/// (re-measured 2026-10-06); without it the frozen params are already over it.
///
/// History: at the quarter-day step the crossings were ×1.170 off and ×3.81 on, the crest
/// 0.13 % over the cap; at 1/16 ×1.210 / ×2.526, the crest 0.97 % over. With no leaf shed from
/// age before anthesis: loss OFF crosses at **×0.760** (the frozen params read 15.408, over the
/// cap); loss ON, `peak W` rises to a crest of **14.186 at ×3** — 1.65 % UNDER the cap — and
/// falls away (14.110 at ×4.5, 13.584 at ×8). So the cap is a detector of this knob only
/// without the loss, and the LAI ceiling (×1.616) is the band's only one with it.
#[test]
fn the_loss_delays_the_biomass_cap_and_bounds_its_overshoot() {
    let off = crossing(false, W_CAP, |q| q.1, 0.3, 1.5);
    assert!(
        (0.72..0.80).contains(&off),
        "loss-OFF cap crossing (measured x0.760) — {off}"
    );
    assert!(
        open_field(1.0, false).1 > W_CAP,
        "without the loss the frozen params must be over the cap"
    );
    // The ridge with the loss on: under the cap at the frozen rung, at its crest and beyond.
    let ridge: Vec<f64> = [1.0, 3.0, 4.5, 8.0]
        .iter()
        .map(|m| open_field(*m, true).1)
        .collect();
    assert!(
        ridge.iter().all(|w| *w < W_CAP),
        "the loss-on ridge reached the cap — {ridge:?}"
    );
    assert!(
        ridge[1] > ridge[0] && ridge[1] > ridge[2] && ridge[2] > ridge[3],
        "x3 is the crest of the peak-W ridge — {ridge:?}"
    );
    let shortfall = (W_CAP - ridge[1]) / W_CAP;
    assert!(
        (0.010..0.025).contains(&shortfall),
        "the crest sits about 1.65 % under the cap (measured 0.0165) — {shortfall}"
    );
}

/// The loss is **one-sided**: below the threshold the two arms are bit-identical.
///
/// The control that says the OFF arm is switching off the cited mechanism and nothing else.
/// ⚠ The low rung moved 2026-10-06 from ×0.682 to **×0.3**: with no leaf shed from age before
/// anthesis the canopy at ×0.682 reaches 6.04 and the loss acts there. At ×0.3 it peaks at
/// 0.131 — far under the threshold. (The canopy is steeply non-linear in SLA here: 0.021 at ×0.2,
/// 0.131 at ×0.3, 6.04 at ×0.682 — the seedling either bootstraps or does not.)
#[test]
fn the_loss_is_one_sided_and_cannot_reach_below_its_threshold() {
    let (lai_on, w_on) = open_field(0.3, true);
    let (lai_off, w_off) = open_field(0.3, false);
    assert_eq!(lai_on, lai_off, "{lai_on} vs {lai_off}");
    assert_eq!(w_on, w_off, "{w_on} vs {w_off}");
    assert!(
        lai_on < 1.0,
        "the low rung must be far under the threshold — {lai_on}"
    );
}

/// The **second** LAI gate's other assertion — `chambers < 1.0` — swept for the first time.
///
/// `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` asserts that the three
/// chambers stay an order of magnitude below the mutual-shading threshold, i.e. that they are
/// carbon-limited by design and cannot reach the regime at all. Nothing had ever moved a knob
/// against that assertion; the shared lab report carries no chamber peak-LAI row, so it was
/// unmeasurable from the harness.
///
/// Measured (until 2026-10-06): 0.5425 / 0.4927 / 0.5849 frozen, still under 1.0 at ×2.5
/// (0.9305 / 0.8439 / 0.9634), all three over it by ×3.5 (1.2061 / 1.0908 / 1.1873). So the
/// chamber assertion was the **second** detector on this knob — later than the LAI ceiling
/// (×2.01), earlier than the biomass cap (×3.81).
///
/// ⚠ **RESTATED 2026-10-06** (the leaf-shedding note §10a; the user's decision). With no leaf
/// shed from age before anthesis the jar peaks at 1.024 at the frozen params, so the gate now
/// asserts the claim itself — chambers below the source's 6.0 threshold — and this sweep says
/// what that costs: measured 1.0245 / 0.8818 / 0.7670 at ×1, 2.5667 / 2.2153 / 1.8258 at ×2.5,
/// 3.5940 / 3.1027 / 2.5478 at ×3.5 (sealed / perennial / consumer). The restated bound is NOT a
/// detector anywhere in the recorded span; that role is lost, recorded rather than kept by a
/// fitted number.
///
/// ⚠ **Why the report cannot show this, and it is not an oversight to add a row for.**
/// `ReadoutSpec::informs` resolves a gate *under the same scenario*, and this gate's scenario
/// is `open_season` — so a chamber row could not declare the gate it serves. A gate that reads
/// four scenarios is representable in the report by exactly one of them. Recorded rather than
/// rebuilt.
#[test]
fn the_chamber_half_of_the_gate() {
    let runs: [(&str, _, usize, bool); 3] = [
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
    ];
    for (name, scenario, years, perennial) in runs {
        let quiet = peak_lai(&trajectory(scenario, years, perennial, &params(2.50, true)));
        let loud = peak_lai(&trajectory(scenario, years, perennial, &params(3.50, true)));
        let frozen = peak_lai(&trajectory(scenario, years, perennial, &params(1.0, true)));
        // The chambers' canopy follows the knob (a sweep, not three unrelated numbers)…
        assert!(
            frozen < quiet && quiet < loud,
            "{name}: {frozen} / {quiet} / {loud}"
        );
        // …and stays under the source's threshold across the whole recorded span: the
        // restated bound is NO detector of a `specific_leaf_area` error up to ×3.5.
        assert!(
            loud < VKS_LAI_THRESHOLD,
            "{name} reached the mutual-shading regime at x3.5 — {loud}"
        );
    }
}
