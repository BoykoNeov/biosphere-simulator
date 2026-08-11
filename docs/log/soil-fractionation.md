## **Soil carbon pool fractionation** (the chamber-scale diagnosis's named seam, taken — then RE-OPENED after the CUE, and REFUSED AGAIN ON A NEW LEG)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**⚠ READ THE RE-OPENING AT THE END OF THIS ROW FIRST: the CUE build discharged this
diagnosis's stated reason for the refusal one commit later, so the "6.47× against a 94× gap
/ structurally out of reach" framing below is DEAD and the live verdict is a different,
better-grounded one.** **DIAGNOSED 2026-08-10, NOT BUILT — read-only; and the seam
paragraph's OWN two load-bearing claims are measured false.**
`docs/plans/post-roadmap-soil-fractionation.md`; probes `M:/temp/soil_frac/`; **21 test
functions / 24 collected** (the roster pin is parametrized ×4; 10 slow) in
`tests/test_soil_fractionation.py` — ⚠ both numbers stated because my first draft wrote "24
test **functions**", a count lifted from pytest output and relabelled, i.e. the
count-vs-its-own-length shape caught **before** shipping this time. **Teeth verified by
MUTATION**: flipping the partition to RothC's *input* ratio (0.59 — the plausible wrong
reading, since §1.3 prints it in bold while the *standing* fraction must be derived) takes
**12 of 24** red. **THE INVARIANT WAS NAMED FIRST (the increment-1 requirement) AND BOTH
PRINCIPLED SIZINGS FAIL**: hold the t=0 CO₂ return fixed ⇒ seed **19.409**, `rationed = 11`;
hold the census inventory fixed ⇒ seed **3.0**, `annual_reset` **hard-errors** (the
fractionated pool returns carbon at 0.62/yr not 4.015/yr, so year 1 never fills enough grain
to re-sow). The *partition* is never free — RothC's Hoosfield equilibrium, first-hand — and
`litter_carbon0 = 3.0` was sized by probe for dramatic O₂ depletion and never cited, so the
**seed total is authoring while the rates and partition are science**; keeping those apart
is what would make sizing the total legitimate. **FINDING 1 — `C* = flux/k` does NOT
"decouple once there is more than one k".** At the cited standing partition (**3.305 %
DPM**, from `0.1533/(0.1533+4.4852)`) the aggregate rate is **0.6206/yr** vs our
**4.015/yr**, so the t=0 stock is `flux₀/k_agg` — **one number, not a free choice**, worth
**6.47×**. ⚠ Stated precisely because the flat version is wrong: the aggregate is **not
constant** (DPM/RPM drain at 33× different rates, so it decays 0.6206 → 0.3) **and that
decay IS the payoff** — return flux 12.045 → 4.17 → 3.09 → 1.26 mol C/m²·yr at 0/1/2/5 yr
against the one-pool form's 12.045 → 0.217 → **0.0039** → 0. ⚠ **AN ADVISOR-SUGGESTED
CORROBORATION WAS TAUTOLOGICAL AND SAYING SO IS THE RESULT**: "does the cited partition
agree with one fitted to hold our own flux?" — they **intersect at exactly one total by
construction**, so agreement is guaranteed; and the 6.47× inventory gain and 6.47× rate
ratio are **one number wearing two hats** (`stock = flux/k`), not two agreeing measurements.
Pinned as a **non-result** so it cannot later be quoted as support. **⚠⚠ FINDING 3 — THE
STRUCTURAL ONE, and it is why this is blocked rather than declined: our CUE = 1.0 CANNOT
EXPRESS THE SLOW POOLS.** Only DPM/RPM take fresh plant input; **BIO and HUM are FORMED by
the humification split** of decomposed material (RothC §1.3 + Figure 1), and `Decomposition`
moves **100 %** of decayed litter C into `microbial_carbon` with respiration a separate draw
(the deliberate Step-4/5 split) ⇒ no humification flux ⇒ a slow pool can be **seeded but
never refilled**. **Measured, not read off the figure** (the repo's own record being that
the derived-but-unmeasured claim is the one that turns out wrong): a HUM pool at k=0.02/yr
seeded at 33.447 mol C runs **strictly non-increasing at every one of 4,575 steps**, 77.83 %
retained at 15 yr, and **breaks closure while doing it** (`rationed = 5`). ⇒ the remaining
**14.5×** of the 94× census gap is **structurally out of reach**, and closing it needs a CUE
⚠⚠ **DISCHARGED 2026-08-10 — the CUE was built (the row above), so a slow pool now REFILLS
and this finding's reason for turning the seam down is gone. Its pin's assertions remain
sound (they measure this module's own inflow-less variant) and its CONCLUSION is false —
annotated at its own site, resolved not corrected. ⚠ Its WINDOW evidence is superseded too:
at seed 7.0 both gates now pass, so "narrower than 1.0 mol C" must be re-derived before
being quoted.** — which **option (B) measured as moving carbon and priced at option (D)'s
size**. **The seam is blocked by the obstacle option (B) already hit, one flow over** — a
strictly better place to be blocked than "priced, not proposed", because it is a property of
the FORM, not a judgement about effort. **FINDING 4 — it does NOT unblock stem-only, and THE
WINDOW IS THE EVIDENCE.** Controls reproduce the record exactly before any subject is read
(frozen min CO₂ **0.038734**, per-year `[0.074023, 0.038734, 0.054208, …]`; frozen+stem-only
`rationed = 1` @ step **502**, min **0.008674**). Sweeping the seed against **both** gates
(`rationed == 0` **and** the 0.05 decade floor) at 15 yr on **both** perennial scenarios:
**exactly one value in the swept set passes (6.5)**, bounded by a hard error at 6.0 and a
floor failure at 7.0 (`0.038341`). **A window narrower than 1.0 mol C in an uncited knob,
located by sweeping until the gate went green, is the consumer-chamber-2× / DPM-RPM-labile /
ruling-B shape** — and unlike the consumer chamber's 2× there is **no independent invariant
to size it on**, since the two that exist both fail. **REFUSED.** ⚠ **The sharper form: at
6.0 fractionation CREATES a failure in `consumer` that stem-only alone never caused**
(frozen `consumer` + stem-only is fine — `rationed = 0`, tail **0.148009**) ⇒ it does not
remove the refusal, **it moves it to another scenario**. **FINDING 2 — the form alone is
benign and mildly good, recorded as a PRICE not a proposal**: at seed 6.0 all six sealed
rows close with the CO₂ tail improving everywhere (perennial 0.038734 → **0.062892**, sealed
0.116 → 0.122, consumer 0.145 → 0.149, water_biting 0.084 → 0.095) at **2× inventory**,
where the one-pool form already rations at 6.0 (`rationed = 6`) — so the headroom is
genuinely the form's doing, and it still moves every carbon golden, both manifests, the Rust
mirror and crossport for **no beneficiary** (the canopy-regulator precedent). **FINDING 5 —
the N-free seed artefact becomes PERMANENT, and option (B)'s result quietly depended on it
washing out.** With one `litter_n` against two carbon pools N must leave on the
**aggregate** flux (`d(N/C)/dt = 0` exactly) — **measured with the seed removed, pool C:N ==
90 at every step to 2.8e-15**, so the design is right and the advisor's `dpm_n`/`rpm_n`
fractionation is **not owed** (two extra stocks for the same result; the pools' C:N can only
diverge under a differentiated input C:N, for which there is no source). ⚠ But **with** the
seed, one-pool drains it at 4.015/yr and converges on 90 (`sealed_chamber` peak **100.55**)
while fractionation parks **96.7 %** of it in RPM at 0.3/yr: **271.70 @3.0, 334.02 @6.0**,
`water_biting` **474.80** — **the very tail-persistence that is the seam's benefit preserves
the artefact** ⇒ the seam **owes `litter_n0`**. **⚠ FINDING 6 (method) — I COMMITTED
CORRECTION 2'S OWN ERROR ONE OPTION LATER**: measured pool C:N at `peak litter_n` on
**`perennial`**, which is **reset-driven**, so that peak is the **annual dump** (C:N set by
the dying plant) not the senescence maximum — it came back **0.20× shed, i.e. N-RICH**, the
opposite side from the explanation my own probe printed beside it. **A logged correction
does not inoculate the next piece of work against its own shape**; what caught it was the
number landing on the wrong *side*, not the discipline. Also verified rather than assumed,
since everything rests on it: `SplitSenescence` re-targets the litter leg **bit-exactly**
without re-scaling (0.000e+00 vs `NitrogenSenescence`'s independent recomputation, 218
sampled steps), and conservation across the custom two-way reset is carried by
`run_season`'s own `assert_conserved` — which is what makes the hard errors **real
starvation** rather than a probe leak. ⚠ **A retrieval hazard took round 6's rotated-table
finding to a THIRD instance**: `pdftotext -layout` detaches the Hoosfield table's label
column and shifts values three rows (naive read: `HUM 0.1533 / IOM 4.4852`); read off the
page render and authenticated **two ways** — the five pools sum **exactly** to the printed
33.8632, and p. 41 re-states each value bound to its pool inside worked arithmetic.
**Deliberately NOT taken: p. 41's `3.51/4.51` CO₂-vs-(BIO+HUM) split** — that is the CUE,
i.e. finding 3's wall, not a scoping convenience. The chamber-scale seam paragraph is
**annotated at its own site**, original kept (the (C)-diagnosis precedent, fifth
application). `open_season` structurally untouched (asserted: an open-field build carries
only `boundary.litter_sink`). No value/golden/param/manifest moved; `git diff src/` empty;
nothing unfrozen; ruff + pyright clean. **⇒ RE-OPENED AND REFUSED AGAIN (2026-08-10,
read-only) — the finding is that the refusal got BETTER, and the headline it was re-opened
FOR did not survive.** The CUE build landed in the commit immediately after this diagnosis
and discharged **finding 3**, its stated reason for turning the seam down; the humification
row also superseded finding 4's window evidence. So the price was re-derived on the
post-split tree (probes `M:/temp/soil_frac2/`; **+5 pins**, file now **26 functions / 29
collected / 15 slow**). **What is actually left to build is only the INPUT half** — RothC is
DPM/RPM (fresh input) feeding BIO/HUM (formed), and the tree now HAS the formed half — so
the live case was never inventory but **retiring `decomposition_rate`**, the last uncited
decomposer carbon rate, sitting at Olson's fast edge *because closure requires it*, by
swapping it for two cited rates + the cited 1.44 input ratio (the (A)/(B)
form-change-discharges-a-param move). **THE VERDICT: both principled sizings still fail on
`perennial`** — constant-flux (19.409) `rationed = 1` at step **807**, constant-inventory
(3.0) `annual_reset` **hard-errors** — ⇒ **`decomposition_rate` is measured UN-RETIRABLE by
the only cited alternative on the shelf**, which is the durable statement (a future reader
reaching for RothC to discharge that TODO can read this instead of re-deriving it). **⚠ THE
FLOOR FAILURE IS THE ATTRACTOR, CHECKED NOT ASSUMED**: the split lengthened the settling
transient to ~35 yr and anchored its own liveness floor on a measured equilibrium at ~yr 45,
so the same fairness question was asked *before* the refusal was written — run to **50
years** the CO₂ minimum rises monotonically and **asymptotes at 0.031741**, still **1.58×
below the 0.05 floor** (frozen control settles 0.073291, asserted alongside because "the
subject converges below the floor" is a verdict only if the control converges above it on
the same harness). **⚠⚠ MY PREDICTION WAS WRONG IN BOTH ITS HALVES AND THE MECHANISM IS THE
FINDING.** (i) It was written **flat across both regimes** — "59 % of every fresh input
decays at 10.0/yr" is a claim about *fresh input*, and only the shedding-fed chambers are
fed that way; a year after a dump the reset-driven comparison **inverts** (41 % left at
0.3/yr vs the bulk pool's `e^−4.015` = 1.8 %). The shedding-fed/reset-driven conflation
correction 2 and (B) finding 5 already logged twice, mine, caught **before** the first probe
— the tell was again a phrase doing unearned work. (ii) I then predicted the slow pool would
**starve** the loop (RPM's 0.3/yr ≈ the Zhang median the decomposer calibration measured as
starving it) — **REFUTED by the probe: the litter return flux at the trough is 2.84×
HIGHER** (8.1112 vs 2.8558 mol C/yr). **The real mechanism: seed 6.47×, return flux 2.84×,
plant 1.81×, and THE ATMOSPHERE THEY ALL TRANSACT THROUGH 1.00×** — `chamber_air_mol` and
the initial CO₂ are untouched by a litter change, so at **0.1 % of system carbon** (from 1.6
%) the jar records every instantaneous mismatch in full. The trough is a **flow-balance
moment**, not a supply shortage. **The chamber-scale diagnosis reached independently for the
fifth time** — the atmosphere is a buffer of *hours*, so enlarging the soil and the plant
while leaving the jar alone is paid for in the jar. Pinned as a *mechanism* because the
census alone + "the slow pool starved it" would assert something unmeasured (the CUE build's
finding 6 shape, one option on). **⚠ ONE SCENARIO BINDS**: `consumer` closes and clears the
floor at sizing 1 (0.129892), and **both shedding-fed chambers close at BOTH sizings with an
improved CO₂ tail** — but that improvement is **not quotable alone**: sizing 2 buys
`sealed_chamber` 0.076380→0.080342 at a **3.5× smaller plant** (1.844→0.520), so the two are
asserted in the same test. ⚠ **The firing step is BISECTED IN THE TEST, not typed**: a first
draft asserted `807 % 305 == 197` about a hand-written literal after a loop that had only
narrowed it to a 305-step window — a location reported under a constant it was never
measured into, the exact (C)-branch shape — so `drive()` gained a `steps=` truncation and
the test now *returns* 807. It lands on **day 197**, the identical within-season day
stem-only fired on (step 502) from an unrelated change ⇒ the day is a property of the
chamber's seasonal draw. ⚠ **Finding 4's window was deliberately NOT re-derived** (a seed
swept until the gate goes green is the refused shape, and that refusal never depended on the
CUE), and **0.59 was not tried as a seed partition** (the seed is a *standing* pool ⇒
Hoosfield's 3.305 %; 0.59 is the input ratio this file's own mutation test uses as the
plausible wrong read). ⚠ **THE NITROGEN SIDE WAS NOT RE-MEASURED and is recorded as
unmeasured** (the (C)/`sealed_station` precedent): finding 5's mechanism is untouched by the
split, so the seam **still owes `litter_n0`**; what *was* re-run is the harness's
(B)-identity self-check, which holds exactly (litter C:N constant to the last digit over all
916 steps with the seed removed, one-pool and fractionated alike) — a check on the harness,
not a re-measurement of finding 5. ⚠ That self-check first read **82× off** and the cause
was **my units, not the tree**: the record's "90" is *mass* C:N while the stocks are mol C
against kg N (×0.012011). No value/golden/param/manifest moved; `git diff src/` empty;
nothing unfrozen; 29 pass, ruff + pyright clean.
