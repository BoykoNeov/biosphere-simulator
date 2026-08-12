## **Stem-reserve remobilization** (the user's own question: does the stem feed the seed?)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**REFUSED 2026-08-10 on PROVENANCE; BUILT 2026-08-12 ON THE USER'S CALL; then given a
CESSATION WINDOW on the user's second call.** `docs/plans/post-roadmap-stem-reserves.md`;
**23 pins** in `tests/test_stem_reserves.py`; +1 flow, +1 stock, +1 param file, **13
goldens**, the biosphere manifest, and the **native mirror** (203 lines, 5 files, no
`simcore`). ⚠⚠ **THE MIRROR CAUGHT WHAT THE PYTHON SIDE COULD NOT**: with every flow
ported, `sealed_chamber`'s O₂ still differed by a factor of **413** — because the build's
`litter_carbon0` 3.0 → 3.5 re-size was never mirrored, and a Python-only suite cannot see
that (its goldens come from the Python scenario). A **scenario constant**, not a science
change, was the thing that drifted; mirrored under the no-reference-authority rule.
**THE STARTING FACT**: our stem gained **62 % AFTER flowering** with **no path at all**
from stem carbon to grain. **THE SCIENCE WAS ON THE SHELF, in the
book `allocation.py` ALREADY CITES**: [E] §2.2.2 Table 7 (p. 46) gives wheat **0.40** as
the remobilizable fraction (**CABO unpublished**, and the caption says so), drained at
**0.1 d⁻¹**. **⚠⚠ THE 2026-08-10 FINDING, WHICH STANDS: the SOURCED form fixes the stem and
overshoots the grain, and the form that LANDS on the grain is ONE WE CONSTRUCTED** —
§3.2.4's growth fraction is what [E] **programs** (Listing 3 Lines 17, 35) and **a data
table is not a model form**. **FINDING 2, THE STRUCTURAL ONE: the partition table is the
blocker and it is UNCITED** (`fs = 0.10` at DVS 2.0, flat-extrapolated), so [E]'s trigger
("once stems stop growing") is **a statement about [E]'s OWN table** and ours substitutes
the weaker availability condition, `DVS >= 1.0`. **THE REFUSAL WAS A RECOMMENDATION, NOT A
VERDICT**, and the user overruled it — the [[root-coupling-refused]] shape. **THE BUILD**:
the fill is a **split of `Allocation`'s own stem leg** rather than a flow of its own, and
that is CORRECTNESS not tidiness — a withdrawing flow would ration at emergence, because
arbitration scales withdrawals against the **start-of-step** amount. ⚠
**`SEALED_CHAMBER_SCENARIO.litter_carbon0` 3.0 → 3.5 because the build ABOLISHED the
phenomenon that scenario exists to show** (O₂ bottomed at **5.08 %** of fill against a
≥ 95 %-depletion contract, and `f_O2` stopped being load-bearing); peak litter and peak
microbial biomass are **unchanged to four figures**, so "less decomposition" was the wrong
explanation and was measured rather than assumed. **⚠⚠ THE CESSATION, AND THE MEASUREMENT
THAT RE-SIZED THE QUESTION**: as built, **neither half ever stopped**. But the reserve
peaks at anthesis and **drains 91 %** by season's end, so the predicted "perpetual
conveyor" **does not exist** — the defect is that **the RULE** never stops, and **3–7 % of
the transfer happens after the crop is physiologically dead** (maturity is step **294 of
305**; `sealed_chamber` never re-sows and spends **two years** past it). Four candidates
measured; **(b)/(c) — stopping the FILL at anthesis — REFUSED on two independent grounds**:
they reinstate the very defect the mechanism exists to fix (stem shape 0.985 → **1.267**
against a frozen 1.618) **and** they land the harvest index **on the oracle** (1.002×),
the refused shape by the standing ruling. ⚠ [E]'s §2.2.1 p. 46 ("glucose formed **before
flowering**") makes (b) a readable interpretation, and in **[E]'s** tree the two readings
**cannot be told apart** because its stem fraction reaches zero. (a) vs (d) differ by
**0.03 % of grain**, so it was a coherence question, **put to the user, who chose (d) —
both halves stop at maturity**. **⚠⚠ THE BOUND IS [E]'S OWN, AND ITS STRENGTH IS THE WHOLE
POINT**: Listing 3 — the module whose Lines 17/35 ARE this mechanism — ends at **Line 114**
with `FINISH DS = 2., CELVN = 3.` (prose twice: §3.1.4 p. 81, §3.4.2 p. 105). But `FINISH`
is **RUN CONTROL**: [E] does not say remobilization ceases at maturity, it says **its
program has no state there**. So this is the source's **DOMAIN BOUNDARY** and adopting it
is a decision **not to extrapolate a form past the program that defines it** — writing it
as a cited cessation rule would be the locus error this record has logged three times. The
question only arises here because our tree has **no `FINISH`**: DVS merely **caps** at 2.0.
That cap is also why the loader refuses `cessation_dvs > 2` — it would **silently restore
the unbounded behaviour** while reading like a choice, and no golden could tell. **WHAT IT
COST, STATED RATHER THAN SMOOTHED**: grain **−2.0 %**; and ⚠ **one claim got WEAKER** — the
build's headline "the stem stops gaining" (0.985) is now a **3.6 % gain**, because the fill
stops while the partition table keeps feeding the stem for 11 more steps, so the pin now
asserts the **ordering against the frozen control** instead of `shape < 1.0`. ⚠ **The
trigger got MORE load-bearing** (sweep moves grain 0.7 % → **2.3 %**, one-sided: only the
late trigger that eats its own window costs anything) — a window converts a start time into
a *length*. **THE GATES**: `rationed == 0`, no events, every chamber at 5 and 15 yr and
under RK4; all four `perennial_long_horizon` gates (trough **0.056030** > 0.05, fixed point
**0.637384** > 0.55); `open_season`'s bands hold with **shrinking margins** (peak LAI
**5.4624** = 91.0 % of the V-K&S threshold; `W` **14.1457** = 98.1 % of the crossing, and
Table 7's **top** row crosses). **THE GOLDEN DIFF WAS PREDICTED BEFORE REGENERATION**: 13
moved by the build, **no fourteenth**, nothing non-wheat — measured, **10** moved and the 3
that held (`greenhouse`/`lighting`/`harvest`) are **7-day** runs that end 287 days short of
maturity, so the window provably cannot reach them; potato is bit-identical because its
reserve is **off** ([E] Table 7 gives potato a **range**, "0.2-0.4", where wheat gets a
point value). ⚠ **TWO COUNTS IN `docs/biosphere-reference.md` WERE ALREADY STALE** and are
corrected here (flow set said 21, manifest held 22 — `Drainage`): the manifest gate equates
the manifest with the tree and **the prose is not a side of that comparison**, the standing
gap hit again. Python suite green; ruff + pyright clean; `cargo test` + `cargo clippy
--all-targets -D warnings` clean; **101** cross-port tests pass, tier-2 bands re-measured
above their own ±1-ULP sensitivities rather than loosened.
