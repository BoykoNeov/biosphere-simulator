# Post-roadmap — the crop's light follows the power the lamp actually gets (the 2026-09-29 review's Step 3, slice 3a)

**Opened 2026-10-01** on the user's *"work on step 3"*, against the review plan's recommendation
for Step 3 (3a and 3b as builds, 3c as a design note first). 3a is taken first, as ordered.
Review plan: `docs/plans/post-roadmap-review-2026-09-29.md`, Step 3.

**B CHOSEN 2026-10-01** (the user: *"Dimming rule"*), source search done (§5): the form has an
engineering precedent, the threshold has no source, so B is **lab-only, WHAT-IF**. One question
remains open with the user (§6): switch-off or gradual. Nothing is built; nothing frozen has moved.

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

## 5. The source search (2026-10-01, for B)

Searched: our shelf (`docs/bvad-reference.md`, every doc and param file: no shedding, priority or
dimming rule) and the web. What was read and what each supports:

* **Chung & Mazzocco, US patent 4,575,679 (General Electric, filed 1983), "Automatic load shed
  control for spacecraft power system"** — description, read through a fetch asked for verbatim
  text (⚠ a summariser sat between us and the page; re-read the page before binding a quote):
  *"it has become conventional to divide the spacecraft electrical load into an interruptible
  load and an uninterruptible load"*; *"Ideally, the interruptible load is shed when the battery
  system contains an amount of charge remaining that is sufficient to enable the spacecraft
  system to survive until a positive battery charging condition is achieved."* The relay
  *"interrupts the power … to thereby place spacecraft 10 in survival mode … until a positive
  charging condition can again be achieved."* **Supports the FORM**: the lamp is an
  interruptible load, life support (`LoadDraw`) is not; shedding is a **switch-off**, not a
  dimming; the threshold is a reserve, *the charge the essential load needs until charging
  resumes*. ⚠ It does **not** state a restore rule; "back on when charging is positive" is our
  reading of "survival mode … until". It gives **no number** for any other system.
* **Dalton & Cohen 2002, NASA/TM-2002-211721, "International Space Station Nickel-Hydrogen
  Battery On-Orbit Performance"** — pages 1–2 read as images. *"The batteries are designed to
  operate at a 35% depth of discharge (DOD) maximum during normal operation"* (abstract);
  requirement *"Contingency orbit capability consisting of one additional orbit at reduced power
  after a 35% DOD without recharge"* (p. 2). **Rejected as our threshold**: it is a battery-life
  limit for Ni-H₂ cells cycling every orbit, not the point loads are shed, and it is a fraction
  of a capacity this model does not have (`power.battery` is an unclamped pool; its "size" is
  only `battery0`). A checked lead, not a bind.
* **"ISS sheds loads at 82 V"** — appeared only in a search engine's summary, never on a page we
  read. Not citable; not used.
* **Crop lighting under a power shortage** — searched; nothing found on how a plant chamber's
  lamps are treated in a shortage.

**Net:** the threshold is WHAT-IF. B is built lab-only, tagged under
`docs/param-file-conventions.md` (WHAT-IF), and cannot enter the reference until a threshold is
sourced. No frozen byte moves (the frozen builds do not carry the rule), so this is not yet a
station unfreeze; §2.4's flow-set movement applies on promotion.

## 6. The sealed station cannot afford its lamp — measured, and a separate finding

`sealed_station`'s battery falls by **exactly** the lamp's energy: 2.0e10 − 5.9456e9 =
1.40544e10 J, and the lamp's daily average × 1220 days (4 × 305) = 1.40544e10 J, difference
**0.0**. `balanced_load_w` sizes the life-support load to the stored solar with `load_fraction
= 1.0` (`BOUNDED_SOC_SCENARIO`), so solar pays for life support and **nothing pays for the
lamp**: the frozen station runs its lamp off its starting charge and would empty around day
1736 (2.0e10 / 1.152e7 J/day). `lighting` has no charger at all; same shape. **Not fixed
here** — it is the frozen station's power budget, and changing it is its own decision.

What it means for B: with the lamp shed, net charging is exactly 0, so the patent's *"until a
positive charging condition"* is **never** met — a shed lamp stays off for good. That is the
honest answer for this station; it is also why the restore rule matters.

**Carrier, revised for a switch.** §3 B's "the crop reads g(battery) at its own step" works only
for a smooth `g`. A latched switch can flip inside the 90 cabin minutes after the crop's one
sample, and the latch is state the crop cannot see. So B uses A's carrier: a lab driver keeps
the latch in `State.aux` (fast sub-steps do not advance aux — `simcore` `substep` — so the lab
driver sets it between sub-steps), the lab lamp draws only while latched on, and after each
group the driver writes the **count** of sub-steps the lamp was on over the group's count into
`State.aux` — 90/90 is exactly 1.0 with nothing shed. The crop's wrapper multiplies `par` by it.
This also restores the lighting-failure check of §4.

**Predictions, revised (lab-only):** no golden, no manifest row. Control: the lab build with the
rule off reproduces the plain run bit for bit. Red first: a deep brownout on the lab sealed
station leaves the crop below its un-shed run, where the plain build is bit-identical to
baseline.

**Open with the user:** switch-off with a latch (the form has a precedent; only the threshold
is invented) or gradual dimming (shape and threshold both invented). Recommendation: switch-off.

## 7. Results

None yet.
