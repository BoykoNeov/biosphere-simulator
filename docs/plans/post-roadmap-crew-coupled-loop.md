# The crew-coupled loop — the chamber-scale seam's standing alternative, measured

**Status: DIAGNOSED 2026-08-10, read-only.** Probes in `M:/claud_projects/temp/crew_loop/`;
pins in `tests/test_crew_coupled_loop.py`.

The chamber-scale diagnosis left exactly one route standing after it refuted the
atmospheric one: *"the crew-coupled route already exists in the tree as
`GREENHOUSE_BIO_SCENARIO` … **three orders more carbon than the frozen chamber
(~1,137×)** — but it runs 7 days with a seedling."* This document takes that route and
measures it.

**The headline is not the one the seam paragraph predicted.** The 1,137× is real and it
is entirely in the **food store**; the *atmosphere* is 1.09× the frozen jar. And the
thing that actually matters was never about carbon inventory at all: **the coupled
assembly already grows a field-scale crop, and no document in this repo says so.**

---

## 0. What was already there (the reframe)

The route is not a build. `station.sealed` assembles the fully-coupled multi-year
station — crew, ECLSS, Power, Thermal, lamp and a **sealed biosphere sharing the cabin
air** — and `station.driver.run_master_day` already carries a `slow_reset` hook that
fires `annual_reset` at each season boundary. `sealed_station` is a **frozen** station
golden at 4 years.

So the task is not "build the crew-coupled loop". It is **close the loop that is already
there**, and the loop is deliberately open in three places, all recorded in
`station.sealed`'s own docstring:

* `with_harvest=False` — "harvest starves the re-sow";
* `close_feces=False` — "the litter/microbial loop is the one *unregulated* loop and
  grows unbounded at illustrative crew-vs-plant scale (Step 6's ~3400× mismatch)";
* store provisioning open — the crew eats from a finite `food_store` with no resupply.

Two of those three claims are re-measured below. One of them is **stale**.

---

## 1. The atmosphere is NOT where the 1,137× lives

`probe1_sizing.py`, arithmetic only — every input is scenario data, a loaded param, or a
BVAD figure already first-hand in `docs/bvad-reference.md`.

| quantity | value |
|---|---|
| `chamber_co2_mol0` (greenhouse **and** sealed station) | 3.796 mol C |
| vs **one BVAD crewmember** (24.654 mol C/CM-d) | **3.70 h** |
| vs the **scenario's own** crew (327.974 mol C/d) | **16.67 min** |
| vs 1 m² of BVAD field wheat (1.7496 mol C/m²·d) | 52.07 h |

The chamber-scale census measured the frozen jar's **entire** carbon inventory at
**3.4 h** of one crewmember. So the crew-coupled cabin's atmospheric headroom is
**1.09×** the frozen jar's whole inventory — the three orders of extra carbon are all in
`crew.food_store`, a pool the plant cannot breathe.

⚠ **This is the soil-fractionation re-refusal's mechanism one level up** — *"seed 6.47×,
return flux 2.84×, plant 1.81×, and the atmosphere they all transact through 1.00×"*.
Reaching for the crew-coupled route **because** it holds 1,137× more carbon would have
been a false headline. It survives for a different reason: it has a genuine source, a
genuine sink, and a multi-year reservoir, so it can demonstrate **cycling** the
plant-only jar structurally cannot.

---

## 2. ⚠ THE FINDING — the coupled plant is FIELD-SCALE, and the chamber is carbon-limited by ISOLATION, not by volume

`probe3_lai.py`, one season (305 master days, 1440 sub-steps/day), LAI computed through
`canopy.leaf_area_index` with the **loaded** `sla_per_mol_c` (0.5872044444444445 m²/mol C)
and the **built** `ground_area` (1.0 m²) — not from a value quoted in a doc row.

| quantity | `sealed_station` (coupled) | `open_season` (frozen field) | ratio |
|---|---|---|---|
| peak LAI | **5.0400** (day 225) | 5.191 | **0.971×** |
| peak W excl. fibrous roots | **11.7024** t/ha | 12.633 | 0.926× |
| peak W incl. fibrous roots | **13.7462** t/ha | 14.954 | 0.919× |

Against the *standalone* chambers the same tree grows **52–70 g DM/m², LAI 0.51–0.63**
(chamber-scale's census). The coupled chamber is a **~24× larger plant in the same 1 m²**.

**The mechanism, measured:** `biosphere.carbon_pool` reads **3.796000 mol at every one of
the 306 day boundaries**. The crew produce CO₂ at a constant `f_resp·food_intake` and the
scrubber removes it at `k_scrub·pool`, so the equilibrium is `P/k_scrub = 3.796e-3/1e-3`
and **`ci` is a regulated constant** (ambient 399.6 ppm, `ci` = 0.7 × that ≈ 280 µmol
mol⁻¹).

⚠ **The flatness is partly by construction and that should not read as emergent:**
`chamber_co2_mol0` **is** that equilibrium — the scenario is filled *at* its fixed point
(asserted in PIN 5, `production / co2_scrub_rate ≈ chamber_co2_mol0`), so the pool starts
there and the regulator keeps it there. What is *not* by construction is that a
field-scale crop's daily draw never moves it: at 1 m² the plant is a **0.6 mol/day**
perturbation on a 3.796 mol pool refilled at 328 mol/day, which §9 measures as the very
headroom that hides the ceiling. Functionally that is the *same unclamped supply*
`open_season` has — the acceptance-gate diagnosis measured `open_season`'s carbon source
as an unclamped boundary stock holding 0.0.

⇒ **A sealed chamber is carbon-limited by ISOLATION, not by volume.** This is the
acceptance-gate diagnosis's own parting inference — *"the binding-ness is a property of
**isolation** rather than of chambers"*, recorded there explicitly as an inference
because the mechanism had not been isolated — now with the mechanism.

⚠ **Scope, stated because four diagnoses walked past this.** Chamber-scale's census, the
canopy regulator's "bit-identically inert", (C)'s `perennial` closure and the acceptance
gate's margin ranking all measured the **standalone** chambers. Their claims are true as
scoped. But `tests/test_senescence_form.py`'s `_ROSTER` is **six biosphere scenarios**,
and the regulator docstring's *"every chamber between 0.068 and 0.632, i.e. **9–88×
below** [the LAI-6 threshold]"* is a sentence about that roster: the **coupled** chamber
sits at **5.0400 = 0.84×** the threshold. Not a falsification — a **scope** finding, and
the same shape this repo has logged a dozen times: a fact about one roster reads like a
property of chambers.

---

## 3. The sizing: the crew is 187× oversized against BVAD's own arithmetic

Two BVAD numbers, both already first-hand in the repo:

```
crew size   = f_resp · food_intake_rate · 86400 / 24.654          = 13.3031 CM
area per CM = 24.654 (mol C/CM-d) / 1.7496 (mol C/m²-d)           = 14.0909 m²/CM
=> the assembly's own crew needs                                    187.45 m²
   and has                                                            1.00 m²
```

⚠ **Keep this separate from the closure ratio below.** 187× is a **sizing** number
(scenario crew vs BVAD *field* wheat on 1 m²); the 4,364× in §4 is a **food-return**
number (harvested grain vs food eaten) and carries the chamber's own shortfall inside it.
Different denominators. This repo has conflated quantities that differ only by
denominator twice (Greenwood's `W` vs `f_N`'s; mass vs area margins), and both times the
tell was a ratio quoted without its denominator.

---

## 4. Closure, measured with a metric the frozen gate cannot supply

⚠ **`rationed == 0` cannot be the closure gate here.** The cabin carries the three ECLSS
control loops as restoring forces, so they absorb the whole gas imbalance and the run
passes **whether or not the plant does anything** — structurally the acceptance-gate
diagnosis's finding 1 (an assertion true by construction w.r.t. carbon). Every run in
this document reports `rationed == 0`, and it means nothing about closure.

The metric that does measure it is **net ECLSS throughput per crop cycle**: a closed loop
is one where the scrubber and the O₂ makeup net to ~zero.

One season (305 d), the frozen sizing, harvest ON:

| quantity | value |
|---|---|
| crew ate | 105,408.0 mol C |
| harvest returned | **24.146 mol C** = **0.023 %** |
| ECLSS scrubbed (`Δ co2_removed`) | **+99,948.4 mol C** |
| ECLSS injected (`Δ o2_supply`) | −99,946.5 mol O₂ |
| plant peak organic C | 86.9 mol |

**The loop is 0.023 % closed and ECLSS does 100 % of the gas work.** The food-return
mismatch is **4,364×** — within 30 % of the ~3400× `station.sealed` records for Step 6,
which is close enough that the two must be checked for being the same quantity before
either is quoted; they are recorded separately here until they are.

---

## 5. ⚠ A RECORDED SCOPE REASON IS STALE — harvest no longer starves the re-sow

`station.sealed` and `station.scenario.SealedStationScenario` both record, as the reason
`with_harvest` defaults off: *"harvest drains `storage_c` to ~0.01 mol by the year
boundary — below the 0.16-mol seed bank `annual_reset` needs — so it starves the
re-sow"*, spike-measured at Phase 6 Step 7.

Measured on the current tree over the **full frozen 4-year horizon**, `with_harvest=True`:

| | value |
|---|---|
| `rationed` | **0** |
| `events` | **()** |
| boundary `storage_c`, year 1 | **0.211816** mol |
| boundary `storage_c`, year 4 | **0.211816** mol (bit-identical — the period-1 plant) |
| seed bank required | 0.16 mol (`leaf_c0 + stem_c0 + root_c0`) |

The margin is **1.32×** and the series is **flat, not decaying**. The recorded reason
was true when it was measured and the tree has moved underneath it since — the decomposer
calibration, the N-cycle form changes and the humification split all moved the plant.

⚠ The claim is scoped to what was run: the boundary value is flat across the frozen
horizon and beyond it (§6), but the margin is 32 %, not an order, so this is *"the
recorded reason is stale"* — **not** *"harvest is safe to turn on"*, which is a contract
question about a frozen scenario's defaults and is not this document's to answer.

---

## 6. Beyond the frozen horizon — and the control is what makes it a finding

An 8-year run hard-errors (`annual_reset: seed bank too small to re-sow — storage_c
4.807958878020194e-31 < seedling 0.16`). Before that can be attributed to harvest it
needs its **control** — the decomposer calibration logged an advisor catch for asserting
"pre-existing" while having measured only one side. `probe7_horizon.py`, both
configurations, same harness, year by year:

| year | harvest **OFF** (the shipped configuration) | harvest **ON** |
|---|---|---|
| 1 | `rationed` 0, `storage_c` 24.357608 | `rationed` 0, `storage_c` 0.2118155 |
| 2 | 0, 24.357608 | 0, 0.2118155 |
| 3 | 0, 24.357608 | 0, 0.2118155 |
| 4 | 0, 24.357608 | 0, 0.2118155 |
| 5 | **112,667**, 0.2137806 | **112,264**, 4.81e-31 |
| 6 | 709,606, 0.0, peak LAI 0.4565 | **hard error** |
| 7 | **hard error** | — |

**The collapse is pre-existing and essentially harvest-independent.** The rationing
counts at year 5 differ by **0.36 %** (112,264 vs 112,667), and **112,667 is the exact
count `CLAUDE.md` already records** for the biosphere `perennial` scenario's year-5
rationing, documented there as *"a beyond-horizon tiling/reset artifact"*. Harvest ON does
not cause the collapse.

⚠ It also **dies a year earlier** (6 vs 7) **and** holds a **115× smaller grain reserve**
(0.2118 vs 24.3576 mol) when the collapse arrives. Those two facts are recorded **side by
side and not joined**: the reserve is the obvious candidate mechanism, but it was **not
isolated** — no run varied the reserve independently — and writing *"it dies earlier
**because** the reserve is smaller"* would be a causal sentence resting on a correlation,
which is exactly what the paragraph above refuses for the 112,667 match.

⚠ The two counts matching to six digits across *different* scenarios (a 9,500-mol cabin
chamber here vs a 1,000-mol standalone jar there) is **recorded as an observation, not an
identification**. It is consistent with the documented reading — a count determined by a
calendar/tiling boundary rather than by the physics would be scenario-independent — but
the mechanism was not isolated here, and this repo's own record is that the
derived-but-unmeasured claim is the one that turns out wrong.

⚠ Peak LAI is **5.0400 in every year 1–5 of both runs** — the period-1 plant, unchanged
by harvest.

---

## 7. The sizing lever, verified before it was used

`ground_area` is **scenario** data by explicit design (`canopy.py`: *"`ground_area` (m²)
is **scenario** data, not crop data"*), and it is the right lever: scaling *stocks* by A
would model **one enormous plant** (light interception saturates in LAI), while scaling
**area** preserves LAI and models **A replicated 1-m² canopies sharing one atmosphere**.

That is a claim about the model, so it was measured rather than asserted
(`probe5_area_scaling.py`): scaling `ground_area` **and every extensive initial stock**
by A = 187.45 on the standalone sealed chamber, over the full 916-step frozen horizon:

* worst relative deviation from exact A-scaling, **every stock at every step**:
  **1.713e-14** (at `biosphere.o2_pool`, step 912) — float round-off;
* peak LAI **0.470657** at A = 1 and **0.470657** at A = 187.45, worst absolute
  difference over the whole trajectory **6.661e-16**.

⇒ area-scaling is an **exact similarity transform**, so sizing the growing area on
BVAD's 14.091 m²/CM is a **sizing on an outside invariant**, not a new model.

---

## 8. The BVAD-area-matched station — three nested failures, and only the last is about carbon

`probe6_bvad_area.py`, one season, growing area 187.4523 m², harvest ON. Each row adds
one thing to the row above it.

| what is scaled with the area | `rationed` | peak LAI | harvest return |
|---|---|---|---|
| nothing (area alone) | 0 | **0.0294** | 0.000 % |
| + the lamp (200 W → 37,490.5 W) | **852,010** | 0.2772 | 0.171 % |
| + the microgrid (solar 1 kW → 187.5 kW, battery ×A) | **282** | **0.2772** | 0.171 % |
| + the cabin air (9,500 → 1,780,797 mol) | 0 | **0.0333** | 0.000 % |

**(a) The light, which BVAD's area arithmetic does not mention.** `PAR =
photon_efficacy · lamp_power_w / ground_area` (`station/lighting.py:123`), so spreading
the same 200 W lamp over 187× the plot dims the crop by exactly 187× and it never grows.
BVAD's 14.091 m²/CM closes the **gas** loop on paper; in a *station* the light for that
area comes off the same power bus.

**(b) The power bus, which the lamp then breaks.** 852,010 firings ≈ 2 flows ×
439,200 registry calls — the microgrid, sized for a 200 W lamp, cannot carry a 37.5 kW
one. Scaling solar and battery with the area drops it to **282**.

⚠ **And the plant is BIT-IDENTICAL across that fix** (LAI 0.2772, peak organic C
767.811 both times) — because the biosphere's `PAR` is **open-loop forcing** read off the
*scenario's* `lamp_power_w`, not off the energy the bus actually delivered. A browning-out
station does not dim its own greenhouse. Recorded as a modelling limitation of the P6.5
lamp seam, not as a defect found here.

**(c) Scaling the cabin air is WRONG, and the reason is worth writing down.** The
scrubber removes at `k_scrub · pool`, so the equilibrium is `P/k_scrub` **in moles,
independent of the air volume** — the run shows `carbon_pool` starting at 711.57 (the
scaled fill) and being scrubbed straight back down to 3.796000. But `ci` is a *mole
fraction*, so 187× the air at the same moles is **187× less CO₂ concentration**, and the
crop starves worse than before. The ECLSS regulator holds an **amount**; the plant reads a
**concentration**.

---

## 9. ⚠ WHY IT CAPS — the two-rate split makes the crop's carbon supply the STANDING POOL, not the FLUX

`probe8_daily_draw.py`, measured rather than inferred:

| quantity | value |
|---|---|
| cabin CO₂ **standing pool** | 3.796000 mol C |
| crew CO₂ **production** | 327.974 mol C/d — **86× the pool** |
| the 1 m² plant's largest single-day **net** gain | **0.601205** mol C/d (day 194) |
| ⇒ headroom at 1 m² | **6.31×** the pool |
| the same per-area demand at BVAD's **14.091 m²** (ONE crewmember) | 8.471 mol C/d = **2.23×** the pool |
| at the assembly's own 187.45 m² | 112.696 mol C/d = **29.69×** the pool |

The biosphere is the **slow** registry: it steps once per master day and takes its whole
day's carbon in **one Euler step, out of the standing pool** — the crew's 327.974 mol C/d
is delivered by the **fast** registry across 1440 sub-steps *after* the plant has had its
one shot.

⚠ **That is a mechanism, so it was ISOLATED rather than argued** (`probe9_split_counter.py`,
`probe10_which_stock.py`). The arithmetic above only *predicts* the cap; the measurement at
187.45 m² was a single `rationed = 282`, and `run_master_day` sums the slow and fast
reports into **one integer** — in a run that also had a power bus under load. Reading that
sum as "the biosphere rationed" would have been the (C)-branch error this repo already
logs (*"a location reported under a constant it was never measured into"*). Split, and
with the binding stock recorded through `arbitration.min_scaling`'s own accumulation:

| | measured |
|---|---|
| slow-side (biosphere) firings | **282** |
| fast-side (cabin / power) firings | **0** — and `power.battery` never falls below 3.09e12 J |
| days the slow step rationed | **282 of 305** |
| binding stock (argmin `available/demand`) | **`biosphere.carbon_pool`**, 296 of 305 days |
| stock whose margin actually went below 1.0 | `carbon_pool`, on **exactly 282** days |
| its worst margin | **0.182976** — demand **5.5×** the pool |
| `biosphere.o2_pool` (also unscaled, also drawn by the ×187 decomposers) | **never binds** |

⚠ `o2_pool` is named explicitly because it was a **live** candidate, not a straw one: it is
the *other* cabin gas pool the area scaling deliberately leaves alone, and decomposition
and microbial respiration draw O₂ and did scale with the litter. It does not bind, and
that had to be measured.

Free corroboration: the runner-up argmin is `biosphere.water_vapor` at a margin of exactly
**2.000000 = 1/(k·dt)** — the acceptance-gate diagnosis's *rate-determined* margin (*"on
donor-controlled stocks the gate is a `dt` CHECK, not a scarcity check"*), turning up in a
measurement that was not looking for it.

So the crop's per-day carbon is capped by the standing pool no matter how much area is
added, and:

> **the shared cabin cannot supply even ONE BVAD crewmember's worth of crop** — it is
> exceeded 2.23× at 14.091 m², before any question of *closure* arises.

Net gain is a **lower bound** on gross uptake (respiration and senescence are already
netted out), so the true cap binds sooner than these ratios say.

⚠ **AND A SECOND PREDICTION OF MINE WAS REFUTED BY THE SAME PROBE, WHICH IS WHY IT IS
RECORDED.** I predicted the 187 m² crop would *pin at* the standing pool — take
~3.796 mol C/d, the whole thing, every day. It does not: its peak single-day net gain is
**1.238150 mol C/d = 0.326× the pool**, only **2.06×** the 1 m² plant's 0.601205. The cap
is on **demand**, not on delivery: the backstop clips *gross* assimilation to what the
pool holds, and what survives into standing carbon is that clipped gross **minus**
maintenance respiration and senescence — which scale with the ×187 biomass and do **not**
scale with the clipped supply. A bigger crop on a fixed carbon ration keeps more mouths on
the same food. **"Demand exceeds the pool" and "the plant receives the pool" are different
statements and only the first is measured.**

⚠ **This is the chamber-scale diagnosis reached independently for the fifth time**, and
it sharpens it: *"the atmosphere is a buffer of hours"* becomes, under the two-rate split,
a **hard per-day ceiling**, because a buffer that is refilled between draws is a flux and
a buffer that is drawn once per day is a stock. The frozen `sealed_station` never sees it
only because a 1 m² crop sits at 6.31× headroom — the same *"one stock in one rig at
11–80 % headroom"* shape the acceptance-gate diagnosis measured, one seam over.

⚠ Recorded as a property of the **coupling**, not a bug in either domain: physically a
greenhouse crop draws continuously and would meet the crew's production; the ceiling is a
consequence of the operator split. Whoever wants a station-scale crop has to change how
the biosphere is *coupled* (sub-daily carbon exchange, or a cabin buffer sized to the
crop's daily draw), not how big the chamber is.

---

## 10. The verdict, and what it does NOT claim

**The route is taken and it is turned down for a measured reason** — the fourth option
this post-roadmap work has refused, and like the soil-fractionation re-refusal the
refusal is better-grounded than the one it replaces. The chamber-scale seam paragraph
offered the crew-coupled route as *"three orders more carbon than the frozen chamber"*.
Measured: the extra carbon is real, it is all in the food store, and **the binding
constraint was never the carbon inventory** — it is the *rate* at which a once-per-day
slow registry can draw from a shared standing pool, which is exceeded at **one
crewmember's worth of crop**.

**What it does NOT claim:**

* **Not that the loop can be closed.** §4 measures it at 0.023 %, §8 at 0.171 %.
* **Not that `with_harvest` should default on.** §5 retires a stale *reason* on the
  frozen horizon; §6 shows harvest ON has a **115× smaller** reserve when the
  beyond-horizon collapse arrives. The default is a frozen scenario's contract and is
  not this document's to change.
* **Not that the year-5 collapse is understood.** §6 establishes it is pre-existing and
  harvest-independent; the mechanism is inherited from `CLAUDE.md`'s tiling/reset reading
  and was **not** isolated here.
* **Not that the coupled chamber is a BLSS analogue.** Chamber-scale's refutation of the
  atmospheric route stands untouched, and §9 adds a second, independent reason.
* **Not a claim about the standalone chambers.** §2 is about the *coupled* one; every
  prior chamber measurement in this repo remains true as scoped, including the canopy
  regulator's "9–88× below" — whose roster is six biosphere scenarios.
* **Nothing was built.** No value, golden, param or manifest moved; `git diff src/` is
  empty; nothing was unfrozen.

**The seam this leaves open, with a measured obstacle (not a recommendation).** A
station-scale crop needs the **coupling** changed, not the chamber enlarged: either
sub-daily carbon exchange between the two registries (the biosphere's `dt = 1 day` is
structural and frozen, so this is a *coupling* change, not a biosphere one), or a cabin
CO₂ buffer sized to the crop's daily draw — which §9 measures at 29.69× the current pool
for 187 m², and which §8(c) shows **cannot** be bought with air volume, since the
regulator holds moles while the plant reads a concentration. Whoever takes it names the
invariant first (the increment-1 precedent).
