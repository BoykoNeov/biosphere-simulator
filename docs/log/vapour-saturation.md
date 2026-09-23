## **The chamber's water vapour obeys saturation** (the biggest known physical defect, closed at the source — and the tripwire meant to catch the fix had one half that never could)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-23 on the user's call** (*"go with your recommendation"*). Plan and every
prediction: `docs/plans/post-roadmap-vapour-saturation.md`. Predecessor: `docs/log/atmosphere.md`,
finding 2 — the first of the three successors it named.

---

## The defect

In a sealed chamber, transpiration filled `water_vapor` at a rate set by the **weather's** VPD,
and `Condensation` drained it first-order at 0.5/day. Nothing asked whether the air could hold
the water. Steady state `v = T/k` made the vapour a property of the plot alone: all three
chambers peaked at the same 536.995 mol in rooms of 1000 and 2000 mol, wet pressure 1.537
against a saturation-implied ≈1.023.

## Measured before designing — the step is the constraint

One ¼-day step transpires up to **80.13 mol**; the 1000-mol room's whole saturation capacity is
**5.28–26.41 mol** across the weather's temperatures — up to **4.85×**. Vapour sat above
saturation on 3660 of 3661 steps. Three consequences, each settling a design choice:

* A bound inside `Condensation` alone cannot hold: it sees only start-of-step vapour and would
  overshoot by a step's transpiration every step.
* A faster first-order condenser needs `k·dt ≈ 3–4` — Euler-infeasible; the backstop would
  ration.
* **So the bound sits at the source.** Sealed transpiration sends the air only its headroom,
  `clamp(cap − v, 0, F)`, and the rest to `condensate` in the same step. `Condensation` removes
  `max(0, v − cap) + k·dt·min(v, cap)` — the excess after a cooler day, plus the engineered
  condenser on the saturated remainder.

**No new number.** `cap = e_s(T) / 101 325 Pa · chamber_air_capacity_mol`: FAO-56's
already-cited curve, the standard atmosphere (a definition, 10th CGPM 1954), the room the
scenario already carries. Relative humidity ceiling 1 — no setpoint invented. One term dropped
deliberately: the `T/T_ref` factor in the ideal-gas conversion (≈7 %), since the model has no
chamber temperature state and drops it for every other gas too.

## Predictions, scored — the golden ones held; two others MISSED

Written before any code:

* **9 goldens change** (the 5 sealed biosphere, the 4 sealed station) and the rest do not —
  held exactly, file for file: 9 changed, 11 identical of the 20 regenerated (the plan wrote
  "the other 12", counting `state_snapshot.json`, which is not a reference output);
  `season_euler_state.json` (open field) byte-identical.
* **Only water stocks move.** Held in all 9: `water_vapor`, `condensate`, `soil_water`, and in
  five of them `subsoil_water` (both consumer, both perennial, `sealed_station`). Not one
  carbon, O₂, nitrogen or consumer value changed —
  including the four **station** chambers, which the prediction had flagged as unmeasured, and
  the root extension's dry-subsoil stop, flagged as the one unmeasured reader in the biosphere.
  The stress factor had been measured at exactly 1 on every step of all three chambers under
  both thresholds, so the up to ~8 kg of water returned from the air to the soil changes nothing the
  crop reads. Water totals are unchanged to the printed precision in every chamber — a
  redistribution, not a leak.
* ⚠ **MISSED: "the irrigation/drainage boundaries move with them".** No boundary stock moved in
  any of the 9 — the water stayed among the four in-chamber stores. The prediction over-reached;
  why irrigation drew exactly the same was not investigated.
* ⚠ **MISSED: "the tripwire goes red on BOTH assertions"** — see the finding below.
* **Manifests:** 9 `golden_sha256` lines plus `water_cycle.yaml`'s hash (its header was
  rewritten — it still called saturation a deferred refinement) and nothing else; the flow set
  is unmoved because both flows keep their ids and type names.

Measured after: vapour ends every step at or below saturation at that step's temperature in all
three chambers, `rationed == 0`, and wet pressure peaks at **1.0235** (was 1.537). The 2000-mol
chamber now holds exactly twice the jar's peak vapour (47.09 vs 23.55 mol) — the vapour belongs
to the room.

## ⚠ Finding — the tripwire had one live half

The atmosphere work left a **labelled tripwire**, meant to redden when this defect was fixed. It
did — but only its room-independence assertion fired. Its `peak > 1.023` assertion stayed
**green on the corrected model**: 1.023 is saturation at **20 °C**, and the weather's warmest day
is warmer, so the saturated jar peaks at 1.0235 — 5e-4 above a line meant to separate "bounded"
from "twenty times too high".

**The half that caught the fix asked about a SHAPE (does vapour scale with the room?); the half
that missed it asked about a VALUE at one temperature.** Had the tripwire been written with only
its threshold, the fix would have landed with it green and the stale finding would have outlived
the defect. It was deleted, as its own text instructed, and replaced by a guard that checks the
bound per step at that step's own temperature, and the room scaling to 1e-12.

## Controls

* Split removed (sealed transpiration given no saturation) → the chamber guard reddens at step 0.
* Above-cap condensation term zeroed → both the new unit test and the chamber guard redden.
* ⚠ The first run of the second control **printed nothing and proved nothing**: a compile error
  in the library tests (a missing trait import in the rewritten ring test) was hidden by my own
  output filter, which grepped for panics and result lines only. Rerun visibly after the fix.

## Tests changed, and why each is not a weakening

* `the_two_cycle_flows_are_first_order_in_their_own_donor_pool` — now given a room large enough
  that every pool sits below the cap, because the first-order law is now the *below-saturation*
  law. The above-cap regime has its own test.
* `the_three_cycle_flows_carry_only_water_in_ring_order` — its fixture charged 5 kg of vapour,
  ~12× a 1000-mol room's cap, which would now hide the vapour leg entirely. Vapour now sits at
  half the cap, and transpiration may have two sinks, air first.
* New: `vapour_above_saturation_condenses_and_transpiration_stops_at_the_cap` (unit, hand
  arithmetic on FAO-56's 20 °C value) and
  `the_chamber_vapour_never_exceeds_saturation_and_scales_with_the_room` (the three chambers).

## Left open

* **Scope of the bound, stated after the advisor's final review:** the bound is an
  Euler-at-¼-day construct: above the cap both flows move a per-step AMOUNT (the whole excess;
  the headroom), not a rate, and no live path runs a sealed chamber under RK4 or another step,
  so it is untested there. And the cap is PER STOCK, not per room: in the station's sealed
  assemblies the crew's humidity sits in a separate `eclss.cabin_h2o` in the same cabin air —
  3.75 mol at the end of every run against a 115–219 mol cap, so the cabin's total water can
  exceed saturation by at most ~3 %. The biosphere's vapour is bounded; the cabin's humidity is
  not claimed to be.
* **The plants still read the weather's VPD, not the chamber's humidity.** A saturated jar
  transpires as if the air were Dutch weather. This does not block the bound — Penman–Monteith's
  radiation term transpires at VPD 0, so coupling alone could not have bounded the vapour — but
  it is the same *"reads something other than the chamber's air"* shape the atmosphere work
  found for CO₂ and O₂. Coupling it would move transpiration, and therefore soil water; the stress
  factor's headroom says the crop would not notice in these chambers.
* The station census row for a cabin gas band (the atmosphere work's third successor) can now
  name either total — the wet one is no longer a defect — though the dry one remains the one
  that cannot drift.
