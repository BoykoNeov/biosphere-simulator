# Post-roadmap: the humification split (a CUE) — the seam fractionation named

**BUILT 2026-08-10 — biosphere unfrozen and re-frozen; the station manifest cascaded.**
Probes in `M:/claud_projects/temp/cue/`.

**The user's instruction was to pay the (D)-sized cascade, and the fork the measurement
opened — what to do about the two `perennial_long_horizon` pins the change falsifies —
was put to them and answered: build, and restate the pins at the frozen 15-year horizon
rather than extend it.**

The soil-fractionation diagnosis ended by naming its own replacement:

> Not fractionation — **the humification split (a CUE)**, which finding 3 identifies as
> what actually decouples stock from flux in this tree, and which option (B) already
> priced as carbon-moving. It is the same wall option (D) faces, approached from the
> soil side. Anyone taking it prices it as (D), not as this.

The user's instruction was to **pay the (D)-sized cascade**. That authorizes the *cost*.
It does not authorize a green gate obtained by tuning — see "The commitment" below.

---

## THE VARIANT, chosen before any measurement, on a stated criterion

Three builds were available and they are not interchangeable:

1. split `Decomposition`'s output into CO₂ / biomass / humus;
2. keep `Decomposition` carbon-only and hang humification off `MicrobialRespiration`;
3. the full multi-pool structure with every decomposing pool splitting the same way.

**Criterion (advisor's, adopted): which variant introduces the fewest uncited free
numbers?** A number measured on one form outlives the form it was measured on — this
repo's most-logged failure — so the form is fixed first and the probe reports a property
of *that* form.

**Chosen: the CENTURY structure (Parton et al. 1987), variant 3 applied at the three
pools this tree has.** The reasons are in this order:

1. **Lineage.** `microbial_respiration_rate` is *already* anchored first-hand to
   CENTURY's K5 active-SOM pool (0.016/day, the 2026-07-21 decomposer calibration, which
   deliberately and openly chose the active-SOM lineage over RothC's "Microbial
   Biomass"). Importing RothC's CO₂/(BIO+HUM) partition onto a CENTURY-rated pool would
   be lineage-mixing — the pool-identity re-anchoring this project has refused three
   times.
2. **Completeness.** CENTURY publishes every constant this build needs, and all of them
   are first-hand in `sources/parton1987.pdf`.
3. **RothC's route was already measured and refused** one item ago (soil fractionation).

### The constants, all first-hand from `sources/parton1987.pdf`

| quantity | value | locus |
|---|---|---|
| CO₂ loss, **surface** nonlignin structural litter | **0.45** | p. 1174 prose |
| CO₂ loss, soil structural / metabolic / slow / passive | 0.55 | p. 1174 prose |
| `Es` — CO₂ lost when active SOM is stabilised into slow SOM | **0.85 − 0.68·T** | eq. [6], p. 1176 |
| `K6` — slow-SOM decay | **0.0038 /week** | p. 1176 prose |

`K6` is not a new retrieval: `params/microbial_respiration.yaml`'s header already quotes
it ("The decay rate for the slow SOM pool (K6 = 0.0038 week⁻¹)") from the round-2
first-hand read of this paper.

### The two mapping choices, made on structure and stated before measuring

* **Litter uses 0.45, not 0.55.** Our `litter_carbon` is senescence-shed material lying
  in the chamber — *surface* residue, not incorporated soil residue. Parton p. 1174:
  *"Nonlignin surface structural litter has a low respiration loss (45 %), since fungi
  are the primary decomposers of surface litter and more efficiently stabilize C into
  microbial biomass."* ⚠ Our single litter pool aggregates CENTURY's structural **and**
  metabolic litter, and metabolic sits at 0.55, so 0.45 is the *surface-structural* end
  of a 0.45–0.55 band our one-pool model cannot resolve. Both ends are measured and
  reported; neither is chosen by outcome.
* **`T = 0`, so `Es = 0.85`.** `T` is the silt+clay fraction. This chamber has **no
  mineral soil at all** — no silt, no clay, no mineral matrix anywhere in the stock set —
  so `T = 0` is the closest value the model can express, and it is the boundary of
  CENTURY's fitted domain rather than an extrapolation beyond it.
  ⚠ **Stated openly because the direction is unhelpful to honesty:** `T = 0` is the
  *least*-stabilising end of eq. [6], i.e. it parks the least carbon in the slow pool and
  is therefore the **gate-friendliest** value available. It is chosen on the structural
  ground above, fixed before the first run, and its sensitivity (`T = 0.5 ⇒ Es = 0.51`)
  is reported as a sensitivity, never as a knob.

### The structure

| flow | from | to | O₂ |
|---|---|---|---|
| `Decomposition` (changed) | `litter_carbon` | CO₂ **0.45** + `microbial_carbon` 0.55 | draws on the CO₂ leg |
| `MicrobialRespiration` (changed) | `microbial_carbon` | CO₂ **0.85** + `humus_carbon` 0.15 | draws on the CO₂ leg |
| `HumusDecomposition` (**new**) | `humus_carbon` | CO₂ **0.55** + `microbial_carbon` 0.45 | draws on the CO₂ leg |

`humus_carbon` is a **POOL, not a POPULATION** — the option-(B) `microbial_n` precedent:
`organ_stock`'s extinction pass would orphan carbon the rest of the system still counts.

Three structural consequences, named now rather than discovered later:

* `Decomposition` acquires an O₂ draw, so it stops being the single-currency CARBON flow
  its docstring is built around (the deliberate Phase-2 Step-4/5 split). The composition
  gate forces this: CO₂ into a `{CARBON:1, OXYGEN:2}` pool drags two oxygens pure-carbon
  litter cannot supply.
* An O₂ draw needs the Step-7 `f_O2` self-limit, or `rationed == 0` stops being
  structural. Both new O₂-drawing flows are throttled by `f_O2` on the **whole** flux,
  matching `MicrobialRespiration`'s existing treatment (aerobic decomposition *is* the
  O₂-consuming process; throttling only the CO₂ leg would let litter keep decaying into
  biomass under anoxia, which is a different organism).
* **Nitrogen follows the carbon partition in the same fractions**, so every organic pool
  inherits the C:N of the material that fell in and mineral N is released ∝ respired C.
  That preserves option (B)'s identity exactly, adds **no** N parameter, and avoids the
  homeostatic-C:N re-anchoring refused twice. ⚠ Recorded as a limitation, not a thing to
  fit: real humus runs C:N ~10 and ours would run at the shed ratio (~90).
  `litter_n0` is **owed** (fractionation finding 5): a slow pool preserves the N-free
  seed artefact instead of draining it within a year.

---

## THE COMMITMENT (written before the first measurement)

If the cited constants break closure, there is exactly one family of knobs that would fix
it: the retained fractions (via `T`), or `litter_carbon0`. **They will not be swept until
a gate goes green.** That is verbatim the shape soil fractionation was refused for — *a
window found by sweeping the gate green is a fitted value* — and there is no independent
invariant to size them on here either.

The legitimate outcome if closure breaks is a **measurement plus a decision handed back**,
the way fractionation, option (C) and stem-only were handled.

---

## THE PREDICTION, on the record before the probe runs

Humification parks carbon in a pool with a multi-year residence time. A sealed chamber's
carbon inventory is **fixed** (chamber-scale census: 3.517 mol C, of which the atmosphere
is ~10 %), so carbon parked in humus is carbon removed from the CO₂ the crop breathes.
The repo's own logged form of this: *any change parking carbon in a standing pool is paid
out of the CO₂ trough* (stem-only), and fractionation's finding 3 already measured a
**seeded** HUM pool breaking closure (`rationed = 5`) on a drip of ~0.5 mol C/yr.

Counter-effect, which is why this is measured rather than asserted: a CUE < 1 at the
litter step returns 45 % of decayed carbon to the atmosphere **immediately**, instead of
routing 100 % of it through a microbial pool with a ~62-day residence time. Early CO₂
helps the trough; long-term sequestration hurts it. Which dominates is the question.

---

## THE PREDICTION WAS WRONG, AND SO WAS THE HEURISTIC BEHIND IT

Measured, on the whole manifest roster, each scenario driven the way its own golden
drives it, **both integrators**: `rationed == 0` everywhere, no extinction events, the
conservation gate (asserted every step inside `run_season`) never fired, and
`annual_reset` re-sowed every year. `sealed_station` — the station-manifest cascade,
recorded as an *unmeasured* leg by the (C) diagnosis — also closes.

The repo's own logged heuristic — *any change parking carbon in a standing pool is paid
out of the CO₂ trough* — is **false here**, and the reason is structural rather than
lucky: a CUE < 1 at the litter step returns 45 % of decayed carbon to the atmosphere
**immediately** instead of routing 100 % of it through a microbial pool with a ~62-day
residence time. Early return more than pays for the humus sink at these constants.

**Controls first, validated before any subject was read.** `perennial` reproduces the
recorded per-year CO₂ minima `[0.074023, 0.038734, 0.054208, 0.054814, 0.054837]` and
min 0.038734 exactly, and the frozen+control build is bit-identical to `build_season`.

| scenario | yr | frozen `rationed` / CO₂ tail | humified (Euler) | humified (RK4) |
|---|---|---|---|---|
| `sealed_chamber` | 3 | 0 / 0.115998 | 0 / 0.116830 | 0 / 0.116970 |
| `perennial_chamber` | 5 | 0 / 0.038734 | 0 / 0.055175 | 0 / 0.076829 |
| `perennial_long_horizon` | 15 | 0 / 0.038734 | 0 / 0.055175 | 0 / 0.076829 |
| `consumer_chamber` | 5 | 0 / 0.144698 | 0 / 0.148486 | 0 / 0.151390 |
| `consumer_long_horizon` | 15 | 0 / 0.144698 | 0 / 0.148486 | 0 / 0.151390 |
| `water_biting` | 1 | 0 / 0.084481 | 0 / 0.085006 | 0 / 0.087038 |
| `sealed_station` (Tier-2) | 4 | 0, events `()` | 0, events `()` | — (Euler-locked) |

⚠ **The improved CO₂ tail is NOT a benefit and must never be quoted alone.** It improves
*because the plant is smaller* — see finding 2. The two numbers travel together in every
table here, deliberately.

`open_season` is **structurally untouched**, asserted not assumed: an open-field build
carries only `boundary.litter_sink` (no litter, microbial or humus stock), so its three
`science_bands` cannot move.

The carbon-only probe is **exact**, not indicative: `f_N ≡ 1.0` with **0 of 18,301 steps**
below 1, so the (A)-finding-7 zero-feedback license holds — checked, not inherited.

---

## FINDING 1 — the frozen form asserts values that are OFF THE END of the cited functions

This is the strongest result here and it **does not depend on the build**.

Our `Decomposition` moves 100 % of decayed litter carbon into `microbial_carbon`, and
`MicrobialRespiration` converts 100 % of microbial turnover to CO₂. Stated in CENTURY's
own variables, the frozen tree implicitly asserts:

* a **litter CO₂ fraction of 0.0**, against Parton's measured 0.45 (surface) / 0.55 (soil);
* **`Es = 1.0`** — and eq. [6] `Es = 0.85 − 0.68·T` **cannot reach 1.0 at any texture**.
  Its range over `T ∈ [0, 1]` is **[0.17, 0.85]**; the frozen value is above the maximum.

That is exactly the shape bucket-3 scope C found for the decomposer *rates* — a value
sitting outside the entire range of the source it is nominally anchored to. Here it is
the **partition** rather than the rate, and it has been invisible because the partition
was never a parameter: it was a structural assumption with no number to audit.

⚠ Precisely: `microbial_respiration_rate` is anchored to CENTURY's K5, and K5 **is** a
*decay* rate — the pool turnover is faithfully cited. What is not cited is where the
decayed carbon goes. The frozen tree sends all of it to CO₂; CENTURY sends `1 − Es` of it
to slow SOM. So the citation covers the rate and never covered the partition.

---

## FINDING 2 — the closure gate stayed green while the plant lost 40 %

`perennial_long_horizon`, 15 yr, Euler — the frozen reference configuration:

| quantity | frozen | humified |
|---|---|---|
| `rationed` | 0 | **0** |
| peak-leaf fixed point (yr 15) | 0.994199 | **0.634352** |
| min CO₂ / yr (yr 15) | 0.054838 | 0.073367 |
| humus carbon (yr 15) | — | **1.34446** |

The chamber's whole carbon inventory is **3.517 mol C** (chamber-scale census). Humus
holds **1.367 mol C at equilibrium — 38.9 % of every carbon atom in the system.** The
plant is what pays, and `rationed == 0` throughout.

**This is the acceptance-gate diagnosis confirmed by an independent event.** That work
measured that the frozen contract's acceptance test is one stock in one rig and that
`rationed == 0` is a property of the *run*, not of the *science*. Here a change removes
40 % of the plant and the closure gate reports success — while the CO₂ trough *improves*,
so a maintainer reading the closure gate and the trough together would conclude the
change was beneficial.

**What caught it was a `liveness_floors` entry** — `max(tail) > 0.9`, measured 0.634 —
i.e. the first live exercise of the contract standing the acceptance-gate-standing work
created a day earlier. The two-field split (bands = outside-sourced plausibility, floors
= continuity with the current calibration) is doing exactly the job it was split for.

---

## FINDING 3 — the decline HAS a floor, ~35 years past the frozen horizon

Run beyond the frozen horizon (diagnostic, never a gate):

| yr | 1 | 5 | 10 | 15 | 20 | 30 | 50 | 60 |
|---|---|---|---|---|---|---|---|---|
| peak leaf | 0.8415 | 0.7093 | 0.6234 | 0.6011 | 0.5965 | 0.5951 | **0.594984** | **0.594984** |
| humus | 0.296 | 1.012 | 1.278 | 1.344 | 1.361 | 1.367 | **1.36691** | **1.36691** |

`rationed == 0` across all 60 years; the last six same-phase diffs are **exactly 0.0**. So
the humified chamber is a genuine period-1 fixed point, 40 % smaller, reached at ~year 45.

⇒ **the change does not destabilise the chamber; it lengthens its settling transient by
an order of magnitude** — from ~3 years to ~35. The frozen horizon is 15. This is the
decomposer calibration's `SEALED_STATION_YEARS` 3→4 wrinkle, one order of magnitude up.

---

## FINDING 4 — the texture window: the structural argument argues SMALL, not ZERO

`Es = 0.85 − 0.68·T` swept on `perennial`, 15 yr:

| T | 0.00 | 0.05 | 0.10 | **0.15** | 0.25 | 0.50 | 0.75 |
|---|---|---|---|---|---|---|---|
| `Es` | 0.850 | 0.816 | 0.782 | 0.748 | 0.680 | 0.510 | 0.340 |
| peak leaf @15 yr | 0.6011 | 0.5479 | 0.4994 | — | — | — | — |
| gate | 0 | 0 | 0 | **hard error: cannot re-sow** | hard error | hard error | hard error |

Real agricultural soils run `T ≈ 0.3–0.7`. **The chamber survives humification only at
the sandiest ~14 % of the parameter's range.**

⚠ **The honest reading, and my first draft got it wrong.** I wrote that `T = 0` was
"chosen on structure and happens to be survivable". That understates it: *no mineral
soil* argues `T` is **small**, not that it is **zero**, and the survivable band ends at
**0.10** — so the structural argument lands the build **near the edge of the window, not
clear of it**. And eq. [6] was fitted across textured soils (Sorensen's incubation
series), so `T = 0` is the boundary of the fitted domain, where the function has least
support.

**What keeps this from being fractionation's refused window is a design decision, not an
argument:** the build ships the three CO₂ fractions as **constants with no texture
input** — there is no silt/clay quantity anywhere in this tree, and adding a `T`
parameter would be shipping a knob whose viable range was determined by sweeping. A
cited constant and a fitted one differ by whether the knob exists.

This is the chamber-scale diagnosis's **fourth** independent witness: the obstacle is
that a 1 m², 3.5 mol C jar cannot host a soil, not that the soil science is wrong.

---

## FINDING 5 — fractionation's structural ceiling is lifted, by construction

Soil fractionation's finding 3 measured that *a seeded slow pool is strictly
non-increasing at every one of 4,575 steps and never refills*, because CUE = 1.0 leaves
no humification flux. Under this change a slow pool **is** refilled — measured, it grows
from 0 to its equilibrium 1.36691 and stays there.

That pin (`test_a_seeded_slow_pool_only_ever_DRAINS`) becomes **false about the tree**. It
is **resolved, not corrected** — a true measurement of a form that no longer exists — and
its replacement is its **inverse**, per the option-(B) precedent: *a pin guarding a
mechanism you removed is decoration.*

---

## The other cited litter fraction, reported rather than chosen

At the soil end of the band (`litter CO₂ = 0.55` instead of 0.45), everything still
closes: `perennial` 15 yr `rationed = 0`, tail 0.057074, humus 1.2058. Both ends of the
band are viable; 0.45 is used because the litter is *surface* residue, which is a
structural reading of what senescence sheds, not an outcome-based pick.

---

## The cascade, priced

New stock `humus_carbon` (+ `humus_n` if N follows the partition) ⇒ biosphere `flow_set`
17→18 and `param_files` + a new `params/humification.yaml` (4 cited params); every sealed
carbon golden; `drift_summary`; the station manifest and `sealed_station`;
`biosphere_params.txt`; the Rust mirror; the crossport tier. Plus `litter_n0`
(fractionation finding 5).

⚠ **Named because it is worse than a value pin:** `sealed_tier2_helper.BIO_ORGANIC_C`
lists the organic-carbon stocks the station's biomass **stationarity watch** sums. It
does not include humus, so under this change that watch reads a *wrong total* rather than
a moved value — a monitor silently measuring the wrong quantity.

Two committed pins on `perennial_long_horizon` fail, and **they are different kinds**:

1. `test_perennial_leaf_cycle_is_a_fixed_point` / `test_stress_perennial_fixed_point_sustained`
   — the liveness floor `max(tail) > 0.9`, measured **0.634** at yr 15 (0.595 at
   equilibrium). A **bound**, arguable past in writing; precedent exists (1.0 → 0.9 when
   the decomposer calibration shrank the plant 19 %). ⚠ But that would be the *second*
   time this floor moved to accommodate a shrinking plant.
2. The same test's `gap < 1e-3 · max(tail)` — **not arguable past at the frozen horizon.**
   The attractor genuinely is not reached in 15 years (finding 3). It needs either a
   horizon change (`LONG_HORIZON_YEARS`, itself a frozen manifest item, at ~3× the
   slow-tier runtime) or a restated pin.


---

## WHAT SHIPPED

`params/humification.yaml` (new, 4 cited params) + `humification.py` (the shared
`respired_and_stabilized` kernel, `HumificationParams`, `HumusDecomposition`), two new
stocks (`humus_carbon`, `humus_n` — both **POOLs**, the option-(B) `microbial_n`
precedent), and the partition applied at all three decomposer steps. `flow_set` 18 → 20,
`param_files` 12 → 13.

⚠ **Two existing flows changed CURRENCY, which `flow_set` cannot see** (it freezes class
*names*): `Decomposition` was single-currency CARBON since Phase-2 Step 4 and is now
CARBON+OXYGEN. That is written into `docs/biosphere-reference.md` because no gate carries
it.

Nitrogen follows the carbon partition in the same fractions at every step, so the three N
legs stay *carried* — no N rate exists anywhere in the decomposer chain, which is option
(B)'s result extended to a third pool. `test_mineralization` pins **why** that is the
right law rather than a convenience: the textbook mineralization balance reduces to *the
nitrogen of the carbon that left as CO₂* exactly when the receiving pool's C:N equals the
donor's, which is this tree's case because it deliberately imposes no homeostatic
microbial C:N.

## THE CASCADE, AS BUILT

10 goldens regenerated (6 biosphere + `greenhouse`/`harvest`/`lighting`/`sealed_station`,
plus `drift_summary` and `sealed_energy_drift_summary`); both manifests; `open_season`
**structurally untouched and its golden hash unmoved** — the claim confirmed by the gate
rather than by assertion.

⚠ **The omitted-humus hazard the advisor named materialised in FIVE places**, and the
spread is the lesson. Five test modules each carry their own "total organic carbon" tuple.
Four were caught only by moved goldens; one — `test_greenhouse_run`'s offload identity —
**failed by exactly the humus amount (8.2e-7 mol)**; and the dangerous one,
`sealed_tier2_helper.BIO_ORGANIC_C`, feeds a *stationarity watch* that would have gone on
passing while summing the wrong total. A sixth site turned up later inside
`test_senescence_form`'s inventory pin. The duplication is now guarded **structurally**:
`test_decomposition::test_every_organic_carbon_pool_is_named_by_the_summary_tuples`
asserts the full organic-carbon stock set of a sealed build and names the sites in its
failure message.

## FINDING 6 — the one fact behind every pin that had to be restated

The split does not destabilise anything. It **lengthens the chamber's settling transient
by an order of magnitude**, from ~3 years to ~35, because the humus pool fills on its own
~5-year turnover. Four separate committed guards were measuring "settled" over horizons
that no longer contain the transient:

| guard | was | now |
|---|---|---|
| `test_perennial_leaf_cycle_is_a_fixed_point` (a manifest science gate) | `gap < 1e-3·scale` + floor `> 0.9` | monotone + decelerating + floor `> 0.55` |
| `test_consumer_leaf_converges_to_a_fixed_point` | same gap test | same restatement |
| `test_biosphere_stress`'s two fixed-point pins | same gap test | same restatement, with a round-off tolerance — its horizon is 328 yr, so its tail *is* the reached attractor |
| `test_regression_sealed_station`'s pre-golden biomass gate | `is_stationary(bound=1.0, transient=1)` | positive + strictly decelerating |

**None of the four was re-tuned to a looser amplitude.** Each was replaced by the claim
that is *still true* and that still fails on the failure mode the original guarded
(amplification / a dead plant). The liveness floor is the one bound that moved, and it is
anchored on the **measured equilibrium (0.594984)** rather than on the 15-year reading
(0.634352), so it does not depend on the horizon — and the equilibrium itself is now a
test (`test_the_perennial_decline_has_a_floor_beyond_the_frozen_horizon`, 50 yr, slow),
because "the decline converges" is the load-bearing claim behind all four restatements and
a plant walking to zero would also look monotone over a short enough window.

⚠ **The floor has now moved twice for a smaller plant** (1.0 → 0.9 at the decomposer
calibration, 0.9 → 0.55 here). Its manifest `source` records the full chain, deliberately,
because the second move is the one a reader should be able to notice.

## FINDING 7 — the acceptance-gate census, re-measured, and three sub-claims changed

The diagnosis's central finding **survives and widens**: the six tightest *live* margins in
the whole 20-scenario roster are still `biosphere.carbon_pool` in the six sealed scenarios.
What changed:

* the margins **loosened** (1.126/1.491/1.802 → 1.500/1.512/2.112/2.340), so the `< 2.0`
  bound is gone — replaced by the **rank**, not re-tuned upward;
* the **runner-up changed identity**: `sealed_chamber`'s `o2_pool` (8.944) is no longer
  7th, and the first non-`carbon_pool` margin is now `power.battery` at 11.086. The
  corollary *"even the runner-up is a chamber property"* is **retired, not restated**;
* the raw all-class ranking leads with **two** `carbon_pool` entries instead of five before
  the rate-determined 2.0 ties appear ⇒ **the `LIVE` qualifier the diagnosis fought to keep
  in the test's name is now doing more work, not less**;
* `litter_carbon`/`litter_n` moved from **rate-determined to live**, because
  `Decomposition` gained an O₂ draw and therefore `f_O2`. They joined exactly the category
  the microbial pair already occupied, for the reason that test's docstring already gave;
* **`test_tripling_the_horizon_does_not_tighten_the_gate` is resolved**: `perennial`'s
  minimum now lies *outside* the 5-year window (1.51249 → 1.50040) while `consumer` stays
  bit-identical. Finding 6 again, from a fifth direction; both halves pinned so the
  asymmetry cannot be lost.

## FINDING 8 — the split DISCHARGES stem-only's rationing refusal, and that is not a re-opening

(C)'s stem-only branch was refused on `perennial`'s **closure**: `rationed 0 → 1` under
Euler at step 502. Under the humified tree stem-only runs **`rationed == 0`** at both 5 and
15 years, and the trough that was a backstop clamp (0.008674, reached in free fall) is a
value the dynamics reach (0.046065).

**The mechanism is measured and it inverts a law this repo had written down.**
`test_senescence_form`'s inventory pin used to show the extra standing stem funded ~67 %
from the soil and ~33 % from the atmosphere — and the atmospheric third is what pushed the
trough into the backstop. Now the soil funds **essentially all of it** (`d_soil/d_tissue`
= −1.007) and the CO₂ pool at the trough is very slightly **up** (+0.00056 where it was
−0.0392). So *"any change parking carbon in a standing pool is paid out of the CO₂
trough"* — the repo's own logged generalisation, and the basis of my pre-probe prediction —
is **false as a law**. It was true of a soil with one fast pool.

⚠ **This does not re-open (C), and the precision matters.** Stem-only's refusal had two
closure legs; one is discharged, the other **survives**: the decade CO₂ floor still fails
(0.046065 against 0.05). ⚠ And my first rewrite of that pin got the survivor wrong in the
flattering direction — I wrote that the attractor was now "comfortably above the floor"
with a single year dipping, and **the stationarity assertion caught it**: the series is not
settled at 15 years, so both guards fire, and the refusal now rests on a longer transient
rather than on a settled collapse. Whether that still justifies refusing the branch is for
whoever revisits (C) with the measurement in hand — re-deciding it inside the commit that
moved the tree underneath it is the co-adaptation shape this project refuses.

⚠ **TAKEN UP 2026-08-10 — see `docs/plans/post-roadmap-nitrogen-cycle-form.md`, "THE
(C)/STEM-ONLY RE-PRICE", and `tests/test_senescence_form.py` §8.** Two corrections to the
paragraph above, both from measurement. (1) **"A longer transient" is not what the guards
are measuring.** Both are horizon-INVARIANT: the floor guard's failing year IS index 2 —
the first year `transient=2` lets it see — and stationarity's offending same-phase diffs
sit at fixed indices 2 and 3, so both verdicts are identical at 15 and at 50 years. They
are **window** questions, not horizon questions. (2) **Run to 50 years, stem-only settles
at 0.075339 — 1.51× the floor and ABOVE the frozen control's own attractor (0.073291)**,
`rationed == 0` on both, and it **clears the manifest-named liveness floor** (0.643676 vs
0.55). Exactly one year of fifty is below the floor. The refusal is therefore a
single-establishment-year one — and the branch still costs **11.8 % of the grain**, so it
is not free. **The verdict stays open and is a contract question** (whether `transient=2`
fits a tree whose transient the split measured at ~35 years); it was deliberately not
decided there either.

## FINDING 9 — fractionation's structural blocker is discharged

Soil fractionation's finding 3 measured that a seeded slow pool is *strictly
non-increasing at every one of 4,575 steps and never refills*, because CUE = 1.0 leaves no
humification flux — and that was **its stated reason for turning the seam down**. The tree
now has that flux. The pin's assertions are sound (they measure that module's own
inflow-less variant) and its **conclusion is false**, so it is annotated in place —
*resolved, not corrected* — rather than rewritten, because the way it was right is the
finding. Whether fractionation is now worth taking is a fresh question this module's
numbers do not answer.

Its behavioural numbers were re-measured on the new tree (the constant-flux sizing rations
**1** time instead of 11 — still a hard break, so that sizing is still not viable). ⚠ One
of those re-measurements was a **harness artefact caught before it became a finding**: the
variant's aggregate N transfer did not carry `f_O2` while the carbon side now does, which
read as "the option-(B) identity is only approximate now" (90.035 instead of 90). Carrying
`f_O2` on both restores it exactly.

## WHAT THIS DOES NOT CLAIM

* **Not** that the chamber's soil is now realistic. Humus reaches 1.367 mol C against the
  chamber-scale census's 94×-short litter pile.
* **Not** that the CO₂ trough improved *for a good reason*. It improved partly because the
  plant is ~40 % smaller. Those two numbers travel together in every table here.
* **Not** that the decomposer carbon *rates* are now cited-and-central. `decomposition_rate`
  still runs at Olson's fast edge; this change cites the **partition**, which was
  previously not a parameter at all.
* **Not** that the split is viable at a real soil texture. It is measured viable only for
  `T ≤ ~0.10` against real agricultural soils at 0.3–0.7 — the chamber-scale diagnosis's
  fourth independent witness.
