## **The humification split (a CUE)** — the soil-fractionation seam's named successor, **taken**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-10 — biosphere unfrozen + re-frozen, station manifest cascaded, both ports
level.** `docs/plans/post-roadmap-cue-humification.md`. The user's call was to **pay the
(D)-sized cascade**; the one fork the measurement opened was put to them and answered
(**restate the pins at the frozen horizon, do not extend it**). Every decomposer carbon flux
is now partitioned between CO₂ and the pool the remainder stabilises into, at **CENTURY's
own constants** (Parton et al. 1987, first-hand from `sources/parton1987.pdf`): litter → CO₂
**0.45** + active SOM; active SOM → CO₂ **0.85** (`Es` at `T = 0`) + slow SOM; slow SOM →
CO₂ **0.55** + active SOM at **K6 = 0.0038/wk**. `flow_set` 18→**20**, `param_files`
12→**13** (`humification.yaml`), +2 POOL stocks (`humus_carbon`/`humus_n` — the option-(B)
`microbial_n` precedent). **N follows the carbon partition at every step**, so no N rate
exists anywhere in the decomposer chain — (B)'s result extended to a third pool, and
`test_mineralization` pins *why* it is the right law rather than a convenience (the textbook
mineralization balance **reduces** to "the nitrogen of the carbon that left as CO₂" exactly
when the receiving pool's C:N equals the donor's, which is this tree's case because it
refuses homeostasis). **THE REASON IT WAS AN UNFREEZE: the frozen form asserted values OFF
THE END of the source's own functions** — a litter CO₂ fraction of **0.0** against a
measured 0.45–0.55, and **`Es = 1.0` where eq. [6] `Es = 0.85 − 0.68·T` cannot exceed 0.85
at ANY texture** (range [0.17, 0.85]). That is bucket-3 scope C's shape one level down:
**the citation covered the RATE and never covered where the decayed carbon goes**, because
the partition was not a parameter. `microbial_respiration_rate` is anchored to K5, and K5
*is* a **decay** rate (eq. [5]); `Es` partitions the flow it drives. **⚠ THE PREDICTION WAS
WRONG AND THE REPO'S OWN LAW WITH IT.** I predicted (and the advisor concurred) that parking
carbon in a slow pool would break closure — the logged generalisation *"any change parking
carbon in a standing pool is paid out of the CO₂ trough"*. Measured: `rationed == 0` on the
**whole manifest roster, both integrators, plus `sealed_station`**, with the CO₂ trough
**improving**. The mechanism is measured, not guessed: `test_senescence_form`'s inventory
pin shows the extra standing stem now funded **~100 % by the soil** (`d_soil/d_tissue` =
−1.007) with the CO₂ pool at the trough very slightly **UP** (+0.00056 where it was −0.0392)
⇒ **the law was true of a soil with ONE FAST POOL, not of soils**. ⚠ **But the improved
trough is NOT a benefit to quote alone** — it is partly a ~40 % smaller plant, and the two
numbers travel together in every table. **⚠⚠ THE ONE FACT BEHIND EVERY RESTATED PIN: the
split lengthens the chamber's SETTLING TRANSIENT from ~3 years to ~35** (humus fills on its
own ~5-yr turnover), past the frozen 15-year horizon. Four committed guards were measuring
"settled" over a window that no longer contains the transient — the two decade-stability
fixed-point pins, the two `test_biosphere_stress` twins, and `sealed_station`'s pre-golden
biomass gate (+ its stability-file duplicate). **None was re-tuned to a looser amplitude**:
each was replaced by the claim still true at the frozen horizon (**monotone +
decelerating**), which still fails on the failure mode the original guarded. The **liveness
floor** is the single bound that moved (`> 0.9` → `> 0.55`) and it is anchored on the
**MEASURED equilibrium 0.594984** (reached ~yr 45, now its own 50-yr slow test) rather than
on the 15-yr reading 0.634352, so it does not depend on the horizon; 2.2× the recorded 0.253
dead baseline. ⚠ **That floor has now moved TWICE for a smaller plant** (1.0 → 0.9 at the
decomposer calibration) and its manifest `source` records the whole chain deliberately,
because the second move is the one a reader should notice. **SCIENCE GATES (discipline step
5): `open_season`'s three `science_bands` are structurally untouched and its golden hash did
not move** — the claim confirmed by the gate, not asserted. **⚠ THE SPLIT DISCHARGES
STEM-ONLY'S RATIONING REFUSAL — AND THAT IS NOT A RE-OPENING OF (C).** (C)'s stem-only
branch died on `perennial`'s closure (`rationed 0 → 1` at step 502); it now runs `rationed
== 0` at 5 **and** 15 years. Its refusal had **two** closure legs and the other
**survives**: the decade CO₂ floor still fails (0.046065 vs 0.05). ⚠ **My first rewrite of
that pin got the survivor wrong in the flattering direction** — I wrote "the attractor is
comfortably above the floor, a single year dips" and **the stationarity assertion caught
it**: the series is not settled at 15 years, so *both* guards fire. Whether the thinner
refusal still holds is for whoever revisits (C) — re-deciding it inside the commit that
moved the tree underneath it is the co-adaptation shape this project refuses. ⚠ **ANSWERED
ONE COMMIT LATER (2026-08-10, the row above): both surviving guards are WINDOW questions,
not horizon questions** — at 50 years stem-only settles at **0.075339**, above the frozen
control's own attractor, with the manifest liveness floor clearing; what is left is the
**contract** question of whether `transient=2` fits a tree whose settling transient this
build measured at ~35 yr. Deliberately still not decided — but the parking sentence stood
pointing at nobody while the work was already done, which is the status-row-is-ungated
shape. ⚠ **AND THE WINDOW QUESTION WAS THEN DISSOLVED THE SAME DAY** (the re-anchor row
below): the floor's window was measured inert **on the frozen tree** and removed. ⚠ **This
build restated four guards and the decade CO₂ floor was not among them** — so its assertions
kept passing for one commit while the comment justifying them (*"dips to ~0.039 … settling
to ~0.055"*, both pre-split numbers of this very build's making) described a tree that no
longer existed. **The restatement sweep must be driven by the QUANTITY the transient moved,
not by the list of guards you happened to touch.** **FRACTIONATION'S STRUCTURAL BLOCKER IS
DISCHARGED**: its finding 3 (*a seeded slow pool never refills, because CUE = 1.0*) was
**its stated reason for turning the seam down**, and the tree now has that flux. The pin's
assertions are sound and its **conclusion is false** ⇒ annotated in place, *resolved not
corrected*. ⚠ One of its re-measurements was a **harness artefact caught before it became a
finding**: the variant's aggregate N transfer did not carry `f_O2` while the carbon side now
does, reading as "the (B) identity is only approximate now" (90.035 vs 90). ⚠ **Its WINDOW
evidence is superseded**: at seed 7.0 both gates now pass, so *"a window narrower than 1.0
mol C"* no longer holds and must be re-derived before being quoted. **THE ACCEPTANCE-GATE
CENSUS RE-MEASURED — central finding survives and WIDENS, three sub-claims change**: the six
tightest **live** margins are still `biosphere.carbon_pool` in the six sealed scenarios, but
(i) they **loosened** (1.126/1.491/1.802 → 1.500/1.512/2.112/2.340) so the `< 2.0` bound is
**replaced by the rank, not re-tuned upward**; (ii) the **runner-up changed identity** —
`sealed_chamber`'s `o2_pool` (8.944) is no longer 7th and the first non-`carbon_pool` margin
is `power.battery` **11.086**, outside the biosphere ⇒ the corollary *"even the runner-up is
a chamber property"* is **RETIRED, not restated**; (iii) the raw all-class ranking leads
with **two** `carbon_pool` entries instead of five before the rate-determined 2.0 ties ⇒
**the `LIVE` qualifier the diagnosis fought to keep in the test's NAME is doing MORE work,
not less**; (iv) `litter_carbon`/`litter_n` moved **rate-determined → live** (they gained
`f_O2` with the O₂ draw), joining the category the microbial pair already occupied for the
reason that test's docstring already gave; (v)
`test_tripling_the_horizon_does_not_tighten_the_gate` is **resolved** — `perennial`'s
minimum now lies **outside** the 5-year window (1.51249 → 1.50040) while `consumer` stays
bit-identical, both halves pinned so the asymmetry cannot be lost. **⚠ THE OMITTED-POOL
HAZARD (advisor-named) MATERIALISED IN SIX PLACES and the spread is the lesson**: six
modules carry their own "total organic carbon" tuple; four were caught only by moved
goldens, one — `test_greenhouse_run`'s offload identity — **failed by exactly the humus
amount (8.2e-7 mol)**, and the dangerous one, `sealed_tier2_helper.BIO_ORGANIC_C`, feeds a
**stationarity watch** that would have kept passing while summing the wrong total. Now
guarded **structurally** by
`test_decomposition::test_every_organic_carbon_pool_is_named_by_the_summary_tuples`, which
asserts the sealed build's full organic-carbon stock set and names the sites in its failure
message. **⚠ THE TEXTURE WINDOW, recorded not buried**: the chambers survive only for `T ≤
~0.10` (at 0.15 `annual_reset` hard-errors) against real agricultural soils at 0.3–0.7 — and
*"no mineral soil"* argues `T` is **small**, not **zero**, so the structural pick lands
**near the edge** of the survivable band. What keeps this from being fractionation's refused
window is a **design decision, not an argument**: the build ships the fractions as
**constants with no texture input**, because a `T` parameter would be a knob whose viable
range was found by sweeping. **The chamber-scale diagnosis's fourth independent witness.**
**⚠ Two flows changed CURRENCY, which `flow_set` cannot see** (it freezes class *names*):
`Decomposition` was single-currency CARBON since Phase-2 Step 4 and is now CARBON+OXYGEN —
written into `docs/biosphere-reference.md` because no gate carries it. Cascade: 10 goldens,
both manifests, `biosphere_params.txt` (+4 hexfloats), the Rust mirror (2 flows changed, 2
added, 2 stocks, 1 params struct) and the crossport tier. `git diff src/simcore/` empty;
full suite **2139 passed**, crossport **101**, cargo clippy/test green, ruff + pyright
clean. Advisor-reviewed before the variant was chosen and before the build.
