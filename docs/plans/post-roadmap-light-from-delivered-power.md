# Post-roadmap — the crop's light follows the power the lamp actually gets (the 2026-09-29 review's Step 3, slice 3a)

**Opened 2026-10-01** on the user's *"work on step 3"*, against the review plan's recommendation
for Step 3 (3a and 3b as builds, 3c as a design note first). 3a is taken first, as ordered.
Review plan: `docs/plans/post-roadmap-review-2026-09-29.md`, Step 3.

**STOPPED BEFORE CODE on a finding, and waiting on the user's decision (§3).** Nothing is built;
nothing frozen has moved.

## 1. What 3a asked for

The crop in a lamp-lit build reads its light from the scenario's nameplate lamp power
(`lamp_power_w`). The `Lamp` flow draws its energy from the battery separately. So a station
whose battery runs flat keeps its crop fully lit — recorded as a limitation in
`log/crew-coupled-loop.md`. 3a: make the crop's light follow what the lamp actually delivered.

## 2. What reading the code found (2026-10-01, no runs except the goldens' own numbers)

1. **Today the only way the lamp gets less than nameplate is the arbitration backstop.**
   `Lamp` draws `lamp_power · dt` from `power.battery`, unconditionally. When the battery cannot
   cover every withdrawal, `simcore::arbitration::min_scaling` scales the drawing flows by
   `battery / total demand`, against the start-of-step amount. Its own doc calls it *"a rare
   numerical guard, not the ecological mechanism"*; golden runs assert it never fires, and under
   RK4 a needed scale is a hard error. It also splits a shortfall between life support
   (`LoadDraw`) and the lamp by the same factor, a priority nobody chose. A literal "light from
   delivered power" would make that guard the crop's dimming mechanism.
2. **No frozen scenario runs the lamp short.** From the goldens: `lighting` (7 days, battery,
   lamp, no charger) ends at 1.19e8 J of its 2.0e8 J. `sealed_station` ends at 5.95e9 J of its
   2.0e10 J — it never runs short, but it **falls by 14 GJ over the run**, so its supply and
   demand are not balanced. Not investigated here; recorded because any dimming threshold
   (option B) has to be placed against that minimum, which has to be measured, not read off the
   end value.
3. **No power shortage has ever been run against a lit crop.** `with_brownout` (scale the solar
   supply) runs only on the single-rate `station`, which has no crop. `with_lighting_failure` is
   the one lamp perturbation, and it has to cut the crop's light (`par`, slow resolver) and the
   lamp's draw (`lamp_power`, fast resolver) **by hand, together**, precisely because the two are
   not coupled.
4. **How a factor could reach the crop without editing `simcore` or the frozen biosphere.**
   `par` is a forcing, a pure function of `(n, dt)`; an env var is a forcing *or* a stock, never
   a product of both. But `Flow::evaluate` receives `&dyn Environment`, so a station-owned wrapper
   around each slow flow can hand the inner flow an environment whose `par` is the schedule's
   value times a factor read from the snapshot. There is **one** read site
   (`CarbonContext::budget`, `domains/src/biosphere/flows.rs:172`), reached by `Allocation`,
   `GrowthRespiration`, `MaintenanceRespiration` and one **aux process**, the lab leaf form's
   `LeafAreaExpansion` (`flows.rs:1748`). Wrapping must therefore cover the slow registry's aux
   processes as well as its flows, which covers every reader today and any later one. Ruled
   out:
   * a factor captured in a shared cell by the `par` closure — breaks the pure-schedule rule and
     save/reload (a snapshot carries `State`, not closures), so the factor must live in `State`;
   * `ScaledFlow` on the crop's flows — scaling the legs is not scaling the light, because the
     canopy's response to light is not linear.
   ⚠ A wrapper reports its own `type_name`, so the station manifest's flow-set rows move. That
   is a station unfreeze in any form of 3a: the lamp seam is documented as *"Power and the
   biosphere share no stock; the whole interface is the lamp-draw schedule"* (`lighting.rs`),
   and 3a changes exactly that.
5. **The lag is 1/16 day now, not the review plan's quarter-day.** Sealed builds: each group is
   1 plant step then 90 cabin minutes. `lighting`: 8 groups of 2 plant steps then 3 power hours,
   so the second plant step of a group reads light 1/16 day older than the first.

## 3. The decision (the user's)

* **A — the backstop dims the lamp.** The driver computes, after each power group, the share
  of the lamp's nameplate energy that reached `boundary.light_used`, stores it in `State.aux`,
  and the wrapped crop multiplies its light by it. No new number. The crop dims only when the
  emergency rationing fires, at the same share as life support. Recorded as a limitation.
* **B — the lamp gets its own dimming rule.** The lamp draws `nameplate × g(battery level)`,
  with `g = 1` above a threshold and falling below it, so it backs off before the battery is
  empty and the rationing never has to fire; the crop reads the same `g` from the same battery
  stock at its own step (no driver change, no stored factor). Needs a threshold and a shape.
  **None is on our shelf** (`docs/bvad-reference.md` and the docs searched for shedding,
  priority, dimming: nothing). Without a cited one it is WHAT-IF, lab-only, and cannot ship in
  the reference. A source search would come first.
* **Neither — skip 3a, go to 3b** (the crop feels the chamber's own humidity; already priced,
  with a precedent in BVAD). 3a stays recorded as "needs a lamp policy first".

**Recommendation: B, starting with a source search; if none is found, build it lab-only as a
WHAT-IF and take 3b next.** A makes a numerical guard into physics and ties the crop to life
support's share by accident; it is the only option the reference could ship without a source,
which is also why it is the tempting one.

## 4. Predictions (for whichever is chosen, written before code)

* **No golden value moves.** A: the factor is the literal `1.0` in any group with no rationing
  (not `Δlight_used / expected`, whose subtraction of a growing cumulative store would move every
  lamp golden by a few low bits). B: `g` is the literal `1.0` above the threshold, and every
  frozen run's battery must stay above it — **measured** against each run's minimum, not its
  end value.
* **Manifest:** the station flow-set rows move (wrapper `type_name`); `golden_sha256` rows do
  not.
* **The test that is red first:** a deep, long brownout on the sealed station (`solar_power`
  scaled toward 0 until the battery empties) must leave the crop's growth below the
  un-perturbed run. Today the crop is bit-identical to baseline under it.
* **The lighting-failure check:** under A, cutting `lamp_power` alone must darken the crop, so
  `with_lighting_failure`'s hand-cut of `par` becomes redundant (kept, and asserted to change
  nothing). Under B it does not: B dims for low battery, not for a failed lamp, so the hand-cut
  stays load-bearing — stated, not hidden.

## 5. Results

None yet.
