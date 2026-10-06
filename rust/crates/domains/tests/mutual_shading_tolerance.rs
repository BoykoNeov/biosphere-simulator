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
/// The crest is 14.4435 and the cap it clears is 14.4248, 0.13 % below it. That near-agreement
/// invites a nitrogen reading, and `nitrogen.yaml` supplies the invitation in its own words:
/// the flat `n_critical` threshold and the Greenwood dilution curve *"[coincide] only at
/// W ≈ 14.44 t/ha"*, and `senescence.yaml` records 14.4248 as where `f_N` first bites. If the
/// crop were pinned there by its own nitrogen limitation, "the cap is unfalsifiable in this
/// direction" would mean something quite different — the observable held **at the gate's own
/// bound by a mechanism inside the model**, rather than approaching a physical limit.
///
/// **It is not nitrogen.** At the crest, dropping `n_critical` to 0.010 and doubling
/// `max_uptake_capacity` each leave `peak W` **bit-identical**: `f_N` is not biting at all, and
/// the 0.13 % agreement is a coincidence of two unrelated numbers.
///
/// **It is light interception, and that is measured rather than derived from `k`.** Cutting
/// `extinction_coef` 0.60 → 0.45 costs **25.1 %** of `peak W` at the frozen canopy and costs
/// **nothing** at the crest (it gains 1.1 %) — the signature of an interception that is already
/// saturated, so more leaf area buys no carbon.
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
        cost_at_frozen > 0.20,
        "a 25 % cut in k must cost the frozen canopy real biomass — {cost_at_frozen}"
    );
    assert!(
        cost_at_crest.abs() < 0.02,
        "...and must cost the crest nothing, because it intercepts everything already — \
         {cost_at_crest}"
    );
}

/// The SLA multiplier at which `read` crosses `bound`, bisected to ~0.1 % of the multiplier.
///
/// ⚠ **Bisection assumes the observable is monotone on `[1.0, hi]`, and one of the four
/// crossings is not monotone beyond its bracket** — `peak W` with the loss on crests near ×4.5
/// and falls away. So `hi` is a real argument, not a convenience: it is where each ladder's
/// monotone stretch ends. The two end checks below are what make a wrong `hi` a failure rather
/// than a plausible number.
fn crossing(shading: bool, bound: f64, read: fn((f64, f64)) -> f64, hi: f64) -> f64 {
    let at = |m: f64| read(open_field(m, shading));
    let (mut lo, mut hi) = (1.0_f64, hi);
    assert!(at(lo) < bound, "the frozen rung must be under {bound}");
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

/// FINDING 5's claim re-derived, its scope corrected — **and then removed by the step**.
///
/// ⚠ **At the quarter-day step (until 2026-09-30)** the loss was bit-identically inert on
/// `peak LAI` (6.022837: the canopy crossed the threshold *at* its summit) and live on
/// `peak W` (gap 0.00206), because `peak W` came later, after days of shedding leaf carbon.
/// That was this test, and "exactly inert" was a statement about one observable.
///
/// **At the 1/16-day step the canopy never reaches the threshold** (peak LAI 5.440614; the
/// converged answer is 5.4273), so the loss is bit-identically inert on **both** observables:
/// the crossing was the coarse step's canopy bias. Asserted as three facts, and the regime is
/// then shown reachable one rung up, so this file still has a run where the loss acts.
#[test]
fn the_loss_is_inert_on_both_observables_at_the_frozen_params_and_live_one_rung_up() {
    let (lai_on, w_on) = open_field(1.0, true);
    let (lai_off, w_off) = open_field(1.0, false);
    assert!(
        lai_on < 6.0,
        "the frozen canopy must stay under the 6.0 threshold at 1/16 — {lai_on}"
    );
    assert_eq!(lai_on, lai_off, "peak LAI: {lai_on} vs {lai_off}");
    assert_eq!(w_on, w_off, "peak W: {w_on} vs {w_off}");

    // One rung up (×1.1) the loss is live on both, each as an absolute fact. Measured at
    // 1/16: peak LAI 6.0797 on vs 6.8832 off; peak W 13.5804 on vs 13.8495 off (gap 0.0198).
    let (lai_on, w_on) = open_field(1.1, true);
    let (lai_off, w_off) = open_field(1.1, false);
    assert!(lai_off - lai_on > 0.5, "peak LAI: {lai_on} on vs {lai_off} off");
    let rel = (w_off - w_on) / w_on;
    assert!(
        (0.015..0.025).contains(&rel),
        "peak W gap {rel} (measured 0.0198: {w_on} on, {w_off} off)"
    );
}

/// **THE HEADLINE.** The loss roughly *doubles* the `specific_leaf_area` error the
/// `5.0 < peak < 8.0` ceiling absorbs before it reddens.
///
/// Measured crossings of `LAI_CEILING`: loss OFF between ×1.12 and ×1.14, loss ON between
/// ×2.00 and ×2.05 — an absorption factor of ~**1.77**.
///
/// ⚠ So the ceiling is **not** unreachable while the loss is modelled, which was the
/// hypothesis this ladder was built to test. The band's upper half is not blind; it is
/// *tolerant*, and the tolerance is the loss's doing rather than the canopy's.
#[test]
fn the_loss_roughly_doubles_the_sla_error_the_lai_ceiling_absorbs() {
    let off = crossing(false, LAI_CEILING, |q| q.0, 1.5);
    let on = crossing(true, LAI_CEILING, |q| q.0, 3.0);

    // ⚠ The two crossings are asserted as absolute facts BEFORE their ratio, because a ratio
    // alone would be satisfied by both arms moving together — which is exactly what a mutation
    // that disables the loss produces.
    // ⚠ Re-measured at the 1/16-day step (2026-09-30): ×1.181 and ×2.058, absorption 1.74.
    // At the quarter-day step they were ×1.138 and ×2.014 (1.77); the claim held, the rungs
    // moved up with the smaller canopy the finer step grows.
    assert!(
        (1.14..1.22).contains(&off),
        "loss-OFF crossing (measured x1.181) — {off}"
    );
    assert!(
        (2.00..2.12).contains(&on),
        "loss-ON crossing (measured x2.058) — {on}"
    );
    assert!(
        on / off > 1.7,
        "the loss must absorb at least 1.7x the SLA error — {on} / {off}"
    );
}

/// The biomass cap is crossed **later, and `peak W` saturates about 1 % above it**.
///
/// ⚠ **Re-measured at the 1/16-day step (2026-09-30), and the claim weakened.** At the
/// quarter-day step the crossings were ×1.170 off and ×3.81 on (a factor of ~3.3), and the
/// crest with the loss on was 14.4435 at ×4.5, only 0.13 % over the cap: "nearly
/// unfalsifiable in this direction", this test's name until then. At 1/16 they are **×1.210
/// off and ×2.526 on (a factor of ~2.1)**, and the crest is **14.5649 at ×4.5, 0.97 % over
/// the cap**. The loss still delays the cap and bounds the overshoot, but by far less; the
/// quarter-day number was partly the coarse step.
///
/// ⚠ Without the loss the two open-field bounds break within 3 % of each other (×1.181 and
/// ×1.210) — near-redundant detectors. With it they still separate, and the LAI ceiling
/// (×2.058) still fires first, now only just ahead of the cap (×2.526).
#[test]
fn the_loss_delays_the_biomass_cap_and_bounds_its_overshoot() {
    let off = crossing(false, W_CAP, |q| q.1, 1.5);
    let on = crossing(true, W_CAP, |q| q.1, 4.5);
    assert!(
        (1.17..1.25).contains(&off),
        "loss-OFF cap crossing (measured x1.210) — {off}"
    );
    assert!(
        (2.45..2.60).contains(&on),
        "loss-ON cap crossing (measured x2.526) — {on}"
    );
    assert!(
        on / off > 2.0,
        "the loss must absorb at least 2x the SLA error on the cap — {on} / {off}"
    );

    // The ceiling of the whole direction, not just of the crossing. ⚠ `peak W` is NOT monotone
    // in SLA with the loss on — it crests here and falls away, so the bisection above is only
    // valid because its bracket stops at the crest.
    let crest = open_field(4.50, true).1;
    let beyond = open_field(8.00, true).1;
    assert!(
        crest > W_CAP && crest > beyond,
        "x4.5 is the crest of the peak-W ridge — {crest} vs {beyond} at x8"
    );
    let overshoot = (crest - W_CAP) / W_CAP;
    assert!(
        (0.008..0.012).contains(&overshoot),
        "the crest overshoots the cap by about 1 % (measured 0.0097; 0.0013 at the quarter-day step) — {overshoot}"
    );
}

/// The loss is **one-sided**: below the threshold the two arms are bit-identical.
///
/// The control that says the OFF arm is switching off the cited mechanism and nothing else.
/// ×0.682 is the low rung of the recorded `specific_leaf_area` span (see the record), where
/// the canopy peaks at 0.79 — an order of magnitude under the 6.0 threshold.
#[test]
fn the_loss_is_one_sided_and_cannot_reach_below_its_threshold() {
    let (lai_on, w_on) = open_field(0.682, true);
    let (lai_off, w_off) = open_field(0.682, false);
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
