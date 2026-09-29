## **The chamber's humidity was the step size's, and now it is a setting** (the saturated air had settled at a fraction set by the time step; the fix was held for a published target, and the condenser now holds BVAD's 75 %)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-29 on the user's calls**, in two rounds. Offered two options for making the
plants read the chamber's humidity, park it or fix the humidity first, the user chose
*"Fix chamber humidity first"*. Shown that the fix puts the air at 100 %, they chose *"Hold;
aim for a humidity setting"*. Shown BVAD's 75 % built and measured, they chose *"Freeze it"*.
Plan, both rounds and every prediction: `docs/plans/post-roadmap-vapour-step-artefact.md`.
Predecessor: `docs/log/vapour-saturation.md`.

---

## The defect: a humidity set by the step

The saturation build (2026-09-23) bounded the chamber's vapour at the source, but both water
flows read the vapour at the start of the step. Transpiration filled the air to the cap; the
condenser then took `k·dt·v` of it in the same step. A step the plants could fill therefore
ended at `cap − k·dt·v`, whose fixed point is **`cap / (1 + k·dt)`**: 0.889 at `k = 0.5/day,
dt = ¼`, and it would have been 0.941 at `dt = ⅛`.

**Measured:** the mean relative humidity at step boundaries was **0.8903 / 0.8890 / 0.8890**
in the jar, perennial and consumer chambers, against 1/(1 + k·dt) = 0.8889. So the humidity
that nine goldens froze on 2026-09-23 belonged to the step size, not the chamber. It came up
while pricing item 2 (plants reading the chamber's humidity): coupled, the plants would have
been reading that artefact.

## Round 1 — the step fix: measured, NOT frozen

Transpiration's headroom counts what the condenser takes in the same step:
`headroom = max(0, cap − v + condensed)`, with `condensed` from one helper both flows call
(`science::condensed_vapour_kg`). A filling step then ends AT the cap at any step size.

Built and regenerated into a temp copy of the goldens (`station::regen::regenerate_in` on
`W:\temp\claude\vsa\golden`, so the frozen files were never written): **9 changed, 11
identical; vapour ×1.12500–1.12503; only water moved**; mean humidity 0.889 → 1.00; wet
pressure 1.0235 → 1.0264. Every prediction held.

⚠ **The honest answer was 100 % humidity**, because the 0.5/day condenser is tiny against
transpiration of up to 4.85× the room's cap per step. Plants later coupled to it would read no
drying pull at all. Told that in plain words before anything was frozen, **the user held it**
and asked for a published humidity setting.

## Round 2 — the setting: BVAD's "about 75 %"

**Source, read off the page images:** BVAD Rev 2 (NASA/TP-2015-218570/Rev2, Feb 2022), §4.5.7
p. 130 and §4.14.1 p. 175, the same paragraph printed twice: *"Similarly, plants require
higher relative humidity – about 75% – to avoid water stress and minimize nutrient solution
usage. Such humidity levels are at the high end for crew comfort."*

⚠ **A choice of locus, recorded as one** (the O₂ setpoint's precedent). The sentence says what
plants *need*, not what a condenser is *set to*, and "about" makes it approximate. The humidity
claim carries no citation of its own; the Wheeler (1993) beside it belongs to the CO₂ sentence.
Cross-check: 75 % is the **upper** crew relative humidity in Table 4-1, p. 63 (25 / 40 / 75 %).

**Design:** `water_cycle.yaml` gains `humidity_setpoint: 0.75` (dimensionless, bounded
`(0, 1]`); both flows aim at `science::humidity_target_kg = setpoint × saturation` in place of
the cap. The step fix stays, because without it the air would settle at 0.75 / 1.125 = **0.667**,
the same artefact one level down.

**Measured, against predictions written first:** 9 changed, 11 identical. End-of-run vapour
×**0.84372–0.84377** (predicted 0.84375 = 0.75 × 1.125). Only `water_vapor`, `condensate`,
`soil_water` and `subsoil_water` moved; no carbon, O₂, N, consumer or boundary value. Mean
humidity 0.7512 / 0.7512 / 0.7505. Wet pressure peak **1.0198** (predicted ≈1.0198). The
water-stress factor stayed exactly 1 at every step; `rationed == 0`. The written goldens are
byte-identical to the measured temp copy.

## Tests, and the blind spot the advisor named

* ⚠ **The per-step bound in `tests/atmosphere.rs` checked saturation, so it would have stayed
  green with the setting ignored.** Tightened to the target, read from the run's own params.
  Mutation: the setting forced to 1.0 in the builder → red.
* A flow test pins that a filling step ends AT the target at `dt = ¼` and `dt = ⅛`, from below,
  at the old fixed point, at the target, at saturation and above it. The saturation flow test
  was re-posed against the target by hand. Mutation: the condenser's term removed → both red.
* A loader test pins 0.75 and rejects 0, 1.01, −0.5 and NaN; 1.0 (plain saturation) loads.
* Two new tests declared in the claim census (68 → 70 `A` rows).

## What this leaves, and what it is not

* ⚠ **Scope:** the condenser's first-order draw still runs *below* the setting, which a real
  dehumidifier would not do. It only matters on steps the plants cannot fill. Recorded, not
  changed. The equality at the target is a per-step amount, claimed for Euler at `dt = ¼`.
* ⚠ `biosphere_params.txt` has no row for the new param: its generator was deleted in S6, and
  a hand-written row would misstate where the number came from.
* **For item 2 (plants reading the chamber's humidity), now unblocked:** at 75 % the chamber's
  own vapour-pressure deficit averages **≈2.5× the weather's** over a run (probe ratio
  2.52 / 2.52 / 2.55). Coupled plants would transpire *more* than today, not less. BVAD's own
  crop transpiration model (§4.14, Monje 1998, Equation 4-23) takes VPD from the chamber's
  relative humidity, a precedent for the form. Not built; the user's call.
* **For the parked cabin gas check** (the user: keep it for later, until there is a leak model
  and a sourced limit): Table 4-1 p. 63 gives both leads, an air leakage rate
  (0.01 / **0.02** / 0.09 kg/day/module, NASA 2019a,b) and total cabin pressure
  (48.0 / 101 or 70.3 or 56.5 / 102.7 kPa). ⚠ `pdftotext -layout` scrambles that table's rows,
  so read the page image. Nothing built.

## The lesson

**A bound that is met with equality inherits every lag in the step.** The saturation build's
invariant (vapour ≤ cap) was true and tested, and the value it settled at was still an
artefact, because a `≤` check cannot see *where* under the bound the state sits. The check that
caught it asked a different question: does the settled value move when `dt` does?
