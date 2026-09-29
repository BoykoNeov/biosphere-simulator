# Post-roadmap — the chamber's humidity was a step-size number

**Opened 2026-09-29**, on the user's choice *"Fix chamber humidity first"*. They were offered two
options for making the plants read the chamber's own humidity: park it, or fix the chamber's
humidity first and then revisit. Predecessor: `docs/plans/post-roadmap-vapour-saturation.md`
(BUILT 2026-09-23). Record: `docs/log/vapour-step-artefact.md`.

## 1. The defect

The saturation build bounds the chamber's vapour, but the value it settles at depends on the
step size. In one step, both water flows read the vapour at the start of the step (`v0`):

* transpiration fills the air up to the cap: `to_air = min(F, max(0, cap − v0))`;
* condensation removes any excess plus the condenser's draw: `max(0, v0 − cap) + k·dt·min(v0, cap)`.

While transpiration can fill the room — one quarter-day step transpires up to 4.85× the room's
whole cap — the step ends at `v1 = cap − k·dt·v0`. The fixed point is **`v = cap / (1 + k·dt)`**,
which is 0.889 at `k = 0.5/day, dt = ¼` and would be 0.941 at `dt = ⅛`.

**Measured 2026-09-29** (probe `W:\temp\claude\vpd_probe`): the mean relative humidity at step
boundaries is 0.8903 in the sealed jar and 0.8890 in the perennial and consumer chambers,
against 1/(1 + k·dt) = 0.8889. So the humidity that nine goldens froze on 2026-09-23 is a
property of the step size, not of the chamber. Plants coupled to it (the deferred seam) would
be reading that property.

## 2. The fix: the cap applies to the END of the step

Transpiration's headroom must count what the condenser takes in the same step. Headroom is the
room left at the end of the step, if the condenser runs and transpiration fills:

`headroom = cap − v0 + condensed(v0) = max(0, cap − v0) + k·dt·min(v0, cap)`

It is never negative, and at or above the cap it reduces to the condenser's own draw. Then
`v1 = v0 + to_air − condensed = cap` whenever `F` can fill it, at **any** step. When `F` cannot
fill it, `to_air = F` and `v1` is unchanged from today, so only steps that were capped move.

* **One formula, not two.** A new `science::condensed_vapour_kg(v, cap, k, dt)` is the
  condenser's draw. `Condensation` and the headroom both call it, and both are wired from the
  same `p.water.condensation_rate`. This adds no new parameter or number.
* `VapourSaturation` gains `condensation_rate`. The only place it is built is `system.rs`
  (grepped: `authoring`, `station` and `godot_bridge` build neither flow).
* `water_cycle.yaml` is **not** edited, because its header stays true ("sends the air only its
  headroom"; the rate "applies to the saturated remainder"). So no param hash moves.

**What the fix means, in plain words.** When the plants transpire faster than the condenser
drains, the chamber air ends every step **saturated: 100 % relative humidity**. That is the
continuous-time answer of the model as it stands, because the condenser (0.5/day, a DESIGN
number) is tiny against the transpiration. The 89 % was not a real humidity. The follow-on
cost: if the plants later read the chamber's own humidity, they will read a vapour-pressure
deficit of **zero** whenever the chamber is saturated. That is the question the user deferred
to after this fix.

## 3. Prediction — written before any code

**Goldens (20 regenerated):**

* **The same 9 change** as in the saturation build: `sealed_chamber`, `perennial_chamber`,
  `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon` (biosphere);
  `greenhouse`, `lighting`, `harvest`, `sealed_station` (station). **The other 11 are
  byte-identical.**
* **Only water moves:** `water_vapor`, `condensate`, `soil_water`, and possibly
  `subsoil_water`. On capped steps, vapour at the step's end rises by **up to 1 + k·dt = 1.125×**
  (to the cap). The condenser's flux rises by the same factor, so condensate and soil water
  shift between each other. A final state whose last step was not capped (a senesced crop
  transpiring less than the headroom) may move by less, or not at all.
* **No boundary stock moves.** The chambers are sealed, and the saturation build's prediction
  that a boundary would move was wrong.
* **Carbon, O₂, N and consumer stocks are unchanged** in the three biosphere chambers, on a
  measurement: the water-stress factor is exactly 1 at every step (minimum fraction of
  transpirable soil water 0.86–0.87, stress starts at 0.5 or lower). ⚠ The four station
  chambers are unmeasured; last time they did not move. If biology moves anywhere, that is a
  finding to explain.
* **The wet-pressure peak** rises from 1.0235 by at most the vapour's own share × 0.125,
  to about **1.026**. No gate compares chamber pressure with a fixed number (grepped).

**Manifests:** 9 golden hashes. No param hash.

**Tests:**

* A new flow-level test: with a transpiration flux that can fill the room, the vapour at the
  end of the step equals the cap at **both** `dt = ¼` and `dt = ⅛`, from below the cap and
  from above it (a cooling step). It must go **red** with the new headroom term removed.
  Before the fix, that test's own arithmetic gives 0.889 and 0.941, which is the step
  dependence stated as a failure.
* The existing per-step bound (`vapour(n+1) ≤ cap · (1 + 1e-12)`) and the room-scaling check
  in `tests/atmosphere.rs` stay **green**. The vapour now sits *on* the cap, so the tolerance
  is what carries it, and that must be run, not read.
* The ring-order wiring test in `system.rs` (vapour at half the cap) keeps both transpiration
  sinks, because the headroom is still finite.

**Scope, unchanged from the saturation build:** the bound and the new equality are per-step
AMOUNTS, claimed for Euler at `dt = ¼` (the only live path for a sealed chamber). The flow-level
test shows the step-end value no longer depends on `dt`; it does not claim RK4.

## 4. The 100 % version: measured, every prediction held, NOT frozen

Built in the working tree and regenerated into a temp copy (`W:\temp\claude\vsa\golden`,
via `station::regen::regenerate_in`, so nothing under `rust/data/golden/` was written):

* **9 changed, 11 identical**, file for file as §3 said.
* **Only water moved**, in all nine: `water_vapor` ×**1.12500–1.12503**, with `condensate`,
  `soil_water` and (in five files) `subsoil_water` giving up the difference. No carbon, O₂, N,
  consumer or boundary value moved, in the four station chambers either.
* Mean relative humidity at step boundaries **0.889 → 1.00** in all three biosphere chambers.
  It reads 1.0013 in two of them because the observer reads the next day's cap: a cooler day
  lowers the cap after the step that filled it, and the next step removes the excess.
* Wet-pressure peak **1.0235 → 1.0264**, against the ≈1.026 predicted. Dry total exactly
  constant.
* The new flow test went red with the condenser's term removed (as did the old saturation
  test), under `--no-fail-fast`.

**Put to the user before `--write`, in plain words:** the fix makes the chamber sit at 100 %
humidity, and plants later coupled to it would read no drying pull at all. **The user chose
*"Hold; aim for a humidity setting"***: a real chamber's condenser holds humidity below
saturation, so find a published target before freezing anything.

## 5. The humidity setting — BVAD's "about 75 %"

**Source: BVAD Rev 2 (NASA/TP-2015-218570/Rev2, Feb 2022), §4.5.7 p. 130 and §4.14.1
p. 175** (the same paragraph printed twice; page images read, not only `pdftotext`):
*"Similarly, plants require higher relative humidity – about 75% – to avoid water stress and
minimize nutrient solution usage. Such humidity levels are at the high end for crew comfort."*

⚠ **The value is recorded as a CHOICE of locus, like the O₂ setpoint's.** The sentence states
what plants *need*, not what a condenser is *set to*, and "about" makes it approximate. The
humidity claim carries no citation of its own; the Wheeler (1993) in the same paragraph belongs
to the CO₂ sentence. Using it as the chamber's controlled humidity is our reading. As a
cross-check, the value equals the **upper** crew limit in Table 4-1, p. 63 (RH 25 / 40 / 75 %;
NASA-STD-3001 for the limits, typical ISS for the nominal), which is what "the high end for crew
comfort" means.

**Design, minimal, and it keeps §2's step fix:**

* `water_cycle.yaml` gains `humidity_setpoint: 0.75`, dimensionless, bounded `(0, 1]` with
  the loader's `require_half_open`. The header's deferred seam ("a controlled relative-humidity
  setpoint below saturation (a design number with no source here)") is discharged and
  rewritten. **That moves the file's manifest hash**, the one extra manifest line.
* One helper, `science::humidity_target_kg(T, n_ref, setpoint) = setpoint · saturation`. Both
  flows use the target where they used the saturation cap: transpiration's headroom
  `max(0, target − v + condensed)`, and the condenser's `max(0, v − target) + k·dt·min(v, target)`.
* **Why the step fix still has to be there:** without it the air would settle at
  0.75 / (1 + k·dt) = **0.667**, the same step-size artefact one level down.
* ⚠ **Scope line, not changed:** the first-order draw `k·dt·min(v, target)` still runs *below*
  the setting. A real dehumidifier stops at its setpoint. That matters only on steps the plants
  cannot fill (none in the frozen runs' capped regime), and it is recorded rather than changed.
* ⚠ **Not in the cross-port value table.** `biosphere_params.txt` is GENERATED by a Python
  script S6 deleted. Hand-writing a row would make it lie about its provenance (the O₂
  setpoint precedent), so the new value is pinned by its own loader test instead.

**Prediction — written before any code, against the COMMITTED goldens:**

* The same **9** goldens change; the other **11** are byte-identical.
* **Only water moves.** End-of-run `water_vapor` ×**0.84375** (0.75 × 1.125) wherever the last
  step was capped, which §4 measured as all nine. `condensate`, `soil_water` and
  `subsoil_water` absorb the difference; the sign of each is not predicted.
* **Carbon, O₂, N, consumer and boundary stocks unchanged** (the soil only gets wetter, and
  water stress was already 1 at every step).
* Mean step-boundary relative humidity ≈ **0.75** in the three biosphere chambers.
* Wet-pressure peak ≈ **1.0198** (1 + 0.75 × 0.0264).
* **Manifests:** 9 golden hashes + `water_cycle.yaml`'s param hash. Nothing else.

**Tests:**

* The per-step bound in `tests/atmosphere.rs` checks against *saturation*, so it would stay
  green if the setting were ignored. It is tightened to the target. **Mutation: force the
  setting to 1.0 in the builder; the tightened bound must go red.**
* Both flow tests are re-posed against the target, and the new step-size test must still go
  red with the condenser's term removed.
* A loader test pins 0.75 and rejects 0, above 1, and NaN.

**Parked, with the leads this search turned up (nothing built):**

* *Cabin gas check* (the user: keep it for later): Table 4-1 p. 63 gives an air leakage rate
  (0.01 / **0.02** / 0.09 kg/day/module, NASA 2019a,b) and total cabin pressure
  (48.0 / 101 or 70.3 or 56.5 / 102.7 kPa). ⚠ `pdftotext` scrambles that table's rows; the
  values above were read off the page image.
* *Plants reading the chamber's humidity* (item 2, now unblocked by a real humidity): BVAD's
  own crop transpiration model (§4.14, adapted from Monje 1998, Equation 4-23) computes VPD
  from the chamber's relative humidity, not the weather's.

## 6. The 75 % setting: measured, every prediction held (before `--write`)

Built in the working tree; regenerated into a fresh temp copy of the committed goldens.

* **9 changed, 11 identical**, as predicted.
* **Only water moved.** End-of-run `water_vapor` ×**0.84372–0.84377** against the predicted
  0.84375, in all nine. `condensate`, `soil_water` and `subsoil_water` took up the difference,
  mostly rising (+0.02 % to +1.6 %); `lighting`'s condensate fell 0.016 %. No carbon, O₂, N,
  consumer or boundary value moved.
* Mean step-boundary relative humidity **0.7512 / 0.7512 / 0.7505** (jar / perennial /
  consumer) against ≈0.75. The water-stress factor is still exactly 1 at every step (minimum
  fraction of transpirable water 0.865–0.872), and `rationed == 0`.
* Wet-pressure peak **1.01981** against ≈1.0198. Dry total exactly constant.
* **Mutations, `--no-fail-fast`:** the setting forced to 1.0 in the builder → the tightened
  per-step bound in `tests/atmosphere.rs` goes red. The step fix removed → both flow tests go
  red. All 420 lib tests green on the fixed tree.
* ⚠ **A finding for item 2, not acted on:** with the air held at 75 %, the chamber's own
  vapour-pressure deficit summed over a run is **1.50× the weather's** (1.498 / 1.497 / 1.500),
  and the transpiration it drives — Penman–Monteith with each step's own radiation and
  temperature, summed — is **1.21× (+21 %)** (1.206 / 1.206 / 1.207; the radiation term does not
  read VPD). So plants coupled to the chamber's humidity would transpire MORE than today, not
  less. At 100 % it would have been the opposite. Unmeasured: how low the soil water would then
  go. That coupling is the user's next call, and this is the number it starts from.
  ⚠ *Corrected the same day:* first recorded as **≈2.5×**, the MEAN of per-step ratios (2.52 /
  2.52 / 2.55), which days with a small weather VPD inflate — it had already been told to the
  user when the
  advisor caught it, and the correction was told too.
