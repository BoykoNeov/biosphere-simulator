## **The crew-coupled loop** (the chamber-scale seam's standing alternative, taken)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED 2026-08-10, read-only — the route is TAKEN and REFUSED on a measured leg, and
the headline is one nobody had written down.**
`docs/plans/post-roadmap-crew-coupled-loop.md`; probes `M:/temp/crew_loop/`; 8 pins in
`tests/test_crew_coupled_loop.py` (4 slow). **THE REFRAME: it is not a build.**
`station.sealed` already assembles the fully-coupled multi-year station and `run_master_day`
already carries the `slow_reset` re-sow hook — `sealed_station` is a **frozen** 4-year
golden. So the task was *close the loop that is already there*, open in three recorded
places (`with_harvest=False`, `close_feces=False`, provisioning). **FINDING 1 — the
atmosphere is NOT where the 1,137× lives.** The seam paragraph offered this route for
*"three orders more carbon"*; it is real and **all of it is in `crew.food_store`, a pool the
plant cannot breathe**. The *atmosphere* is **3.70 h** of one BVAD crewmember against the
frozen jar's whole-inventory **3.4 h** ⇒ **1.09×**, and **16.7 minutes** against the crew
the scenario actually carries. The soil-fractionation re-refusal's mechanism one level up
(*"…and the atmosphere they all transact through 1.00×"*); reaching for the route
**because** of the 1,137× would have been a false headline. **⚠⚠ FINDING 2 — THE ONE NO
DOCUMENT SAYS: THE COUPLED PLANT IS FIELD-SCALE, AND A CHAMBER IS CARBON-LIMITED BY
*ISOLATION*, NOT BY VOLUME.** Peak LAI **5.0400** vs `open_season`'s pinned **5.191**
(**0.971×**), peak W **11.7024**/**13.7462** t/ha vs 12.633/14.954 — measured through
`canopy.leaf_area_index` with the **loaded** `sla_per_mol_c` and the **built**
`ground_area`, not inferred from a doc row. Against the *standalone* chambers the same tree
grows 52–70 g DM/m², LAI 0.51–0.63. **Mechanism, measured:** `carbon_pool` reads **3.796000
at every one of the 306 day boundaries** — crew production `f_resp·food_intake` against
scrubber removal `k_scrub·pool` pins it at `P/k_scrub`, so **`ci` is a regulated constant**
(ambient 399.6 ppm), functionally the same unclamped supply `open_season` has. This is the
acceptance-gate diagnosis's own parting *inference* (*"binding-ness is a property of
**isolation** rather than of chambers"*, recorded there as an inference **because the
mechanism had not been isolated**) — now with the mechanism. ⚠ **Scope, and four diagnoses
walked past it**: chamber-scale's census, the canopy regulator's "bit-identically inert",
(C)'s `perennial` closure and the acceptance gate's ranking all measured the **standalone**
chambers; `test_senescence_form.py`'s `_ROSTER` is **six biosphere scenarios**, so its
*"every chamber between 0.068 and 0.632, i.e. **9–88× below**"* the LAI-6 threshold is a
sentence about that roster while the **coupled** chamber sits at **0.84×** it. **A scope
finding, not a falsification.** **FINDING 3 — the sizing, on BVAD's own two numbers**: crew
**13.3031 CM** (its own rate ÷ Table 3-31's 24.654 mol C/CM-d) × **14.0909 m²/CM** (24.654 ÷
Table 4-91's 1.7496 mol C/m²·d) ⇒ the assembly needs **187.45 m²** and has **1.00**. ⚠ Kept
deliberately apart from finding 4's **4,364×** food-return ratio — **different
denominators**, only one is a sizing error (the Greenwood-`W` / mass-vs-area shape, twice
bitten). **FINDING 4 — closure measured with a metric the frozen gate cannot supply.** ⚠
`rationed == 0` **cannot** be the closure gate here: the three ECLSS loops absorb the whole
gas imbalance, so it passes **whether or not the plant does anything** — the acceptance
gate's finding 1, structurally. Every run here reports `rationed == 0` and it means nothing.
On net ECLSS throughput: one season, crew ate **105,408 mol C**, harvest returned **24.146**
= **0.023 %**, scrubber removed **+99,948.4 mol C**. **The loop is 0.023 % closed and ECLSS
does 100 % of the gas work.** **⚠ FINDING 5 — A RECORDED SCOPE REASON IS STALE.**
`station.sealed`/`SealedStationScenario` both give, as the reason `with_harvest` defaults
off, *"harvest drains `storage_c` to ~0.01 mol … below the 0.16-mol seed bank … so it
starves the re-sow"* (spike-measured at P6.7). Measured on the current tree over the **full
frozen 4-year horizon**: `rationed == 0`, `events == ()`, boundary `storage_c` **0.2118155
at years 1 AND 4, bit-identical** (the period-1 plant) — **1.32× the requirement, flat not
decaying**. The decomposer calibration, the N-cycle form changes and the humification split
all moved the plant since. Pinned as *"the reason is stale"*, **NOT** *"harvest is safe to
enable"* — a frozen scenario's defaults are a contract question. **⚠ FINDING 6 — AND THE
CONTROL IS WHAT MAKES IT A FINDING.** An 8-year run hard-errors (`annual_reset: seed bank
too small — storage_c 4.81e-31`). Attributing that to harvest without the control is the
exact error the decomposer calibration logged an advisor catch for. Run both: harvest
**OFF** rations **112,667** at year 5, collapses year 6 (LAI 0.4565), hard-errors year 7;
harvest **ON** rations **112,264** at year 5 and hard-errors year 6. **−0.36 %** apart ⇒
**pre-existing and harvest-independent**; and **112,667 is the exact count `CLAUDE.md`
already records** for the biosphere `perennial` year-5 rationing, *"a beyond-horizon
tiling/reset artifact"*. Harvest ON does **not cause** the collapse. ⚠ It also dies a year
earlier (6 vs 7) **and** holds a **115× smaller** grain reserve (0.2118 vs 24.3576) when the
collapse arrives — recorded **side by side and NOT joined**: the reserve is the obvious
candidate but was **not isolated** (no run varied it), and writing *"dies earlier BECAUSE
the reserve is smaller"* would be the causal-sentence-on-a-correlation this same row refuses
one clause over for the 112,667 match. ⚠ The six-digit match **across different scenarios**
is recorded as an **observation, not an identification** — consistent with a
calendar-determined count, but the mechanism was **not** isolated here. **FINDING 7 — the
sizing lever, verified BEFORE it was used.** `ground_area` is **scenario** data by design;
scaling *stocks* would model **one enormous plant** (interception saturates in LAI) while
scaling **area** preserves LAI = A replicated 1-m² canopies. Measured, not asserted:
`ground_area` + every extensive IC × 187.45 on the standalone chamber over the full 916-step
horizon gives worst relative deviation **1.713e-14** across **every stock at every step**,
peak LAI **0.470657 vs 0.470657** (worst |Δ| **6.661e-16**) ⇒ an **exact similarity
transform**. **FINDING 8 — the BVAD-matched station: THREE NESTED FAILURES, and only the
last is about carbon.** At 187.45 m²: (a) **area alone ⇒ peak LAI 0.0294** — `PAR =
photon_efficacy·lamp_power_w/ground_area`, so the same 200 W lamp over 187× the plot dims
the crop by exactly 187×; **BVAD's 14.091 m²/CM closes the GAS loop and says nothing about
the LIGHT**, which in a station comes off the same bus. (b) **+ lamp (37.5 kW) ⇒ `rationed`
852,010** — the microgrid sized for 200 W; **+ microgrid ⇒ 282**. ⚠ **The plant is
BIT-IDENTICAL across that fix** (LAI 0.2772, organic C 767.811) because the biosphere's PAR
is **open-loop forcing** read off the *scenario's* `lamp_power_w` — **a browning-out station
does not dim its own greenhouse** (a P6.5 limitation recorded, not a defect found). (c) **+
cabin air (×187) is WRONG and worth writing down**: the scrubber's equilibrium `P/k_scrub`
is an **AMOUNT** independent of volume (`carbon_pool` starts 711.57, is scrubbed straight
back to 3.796000) while `ci` is a **mole fraction** ⇒ 187× the air is 187× less CO₂ and LAI
falls to 0.0333. **The regulator holds moles; the plant reads a concentration.** **⚠⚠
FINDING 9 — WHY IT CAPS, and it is the fifth independent arrival at the chamber-scale
diagnosis: THE TWO-RATE SPLIT MAKES THE CROP'S CARBON SUPPLY THE STANDING POOL, NOT THE
FLUX.** The biosphere is the **slow** registry — one Euler step per master day, drawing from
the **standing** 3.796 mol pool, while the crew's **327.974 mol C/d** (**86× the pool**) is
delivered by the fast registry across 1440 sub-steps *after* the plant has had its one shot.
⚠⚠ **THAT IS A MECHANISM, SO IT WAS ISOLATED RATHER THAN ARGUED — and the advisor's blocking
catch was that my first draft had NOT isolated it.** The arithmetic only *predicts* the cap;
the measurement at 187.45 m² was a single `rationed = 282`, and **`run_master_day` sums the
slow and fast reports into ONE integer** — in a run that also had a power bus under load.
Reading that sum as *"the biosphere rationed"* is the (C)-branch error this repo already
logs (*a location reported under a constant it was never measured into*). Split, with the
binding stock recorded through `arbitration.min_scaling`'s own demand accumulation:
**slow-side 282, fast-side 0** (battery never below 3.09e12 J), rationing on **282 of 305
days**, binding stock **`biosphere.carbon_pool`** (argmin on 296/305 days, below 1.0 on
**exactly 282**, worst margin **0.182976** ⇒ demand **5.5×** the pool). ⚠ **`o2_pool` was a
LIVE candidate, not a straw one** — the *other* unscaled cabin gas pool, drawn by
decomposers that **did** scale with the litter — and it **never binds**; that had to be
measured. Free corroboration: the runner-up argmin is `water_vapor` at **exactly 2.000000 =
1/(k·dt)**, the acceptance gate's *rate-determined* margin, turning up in a measurement not
looking for it. ⚠ **AND MY SECOND PREDICTION WAS REFUTED BY THE SAME PROBE**: I predicted
the 187 m² crop would *pin at* the pool (~3.796 mol C/d); its peak net gain is **1.238150 =
0.326× the pool**, only **2.06×** the 1 m² plant's. **The cap is on DEMAND, not on
DELIVERY** — the backstop clips *gross* assimilation while maintenance respiration and
senescence scale with the ×187 biomass and not with the clipped supply, so a bigger crop on
a fixed ration keeps more mouths on the same food. *"Demand exceeds the pool"* and *"the
plant receives the pool"* are different statements and only the first is measured. Measured:
the 1 m² plant's largest single-day **net** gain is **0.601205 mol C/d** ⇒ **6.31×
headroom**, which is the only reason the frozen scenario never sees this; the same per-area
demand at BVAD's **14.091 m² — ONE crewmember's worth of crop — is 8.471 mol C/d = 2.23× the
pool**, and at 187.45 m² **29.69×**. ⇒ **the shared cabin cannot supply even one
crewmember's worth of crop, before any question of closure arises.** Net gain is a **lower
bound** on gross uptake, so it binds sooner. *"The atmosphere is a buffer of hours"* becomes
a **hard per-day ceiling**, because a buffer refilled between draws is a flux and a buffer
drawn once per day is a stock. Recorded as a property of the **coupling**, not a bug in
either domain. **THE SEAM LEFT OPEN, with a measured obstacle (not a recommendation)**: a
station-scale crop needs the **coupling** changed — sub-daily carbon exchange between the
registries, or a cabin buffer sized to the crop's daily draw (29.69× at 187 m²) — and
finding 8(c) shows that **cannot** be bought with air volume. Whoever takes it names the
invariant first (the increment-1 precedent). No value, golden, param or manifest moved; `git
diff src/` empty; nothing unfrozen; ruff + pyright clean.
