## Bucket 3 scope (B): the full oracle match

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**INCREMENT 1 + CEREMONY 2 BOTH COMPLETE (2026-07-20) — biosphere unfrozen, re-frozen (inc.
1); ceremony 2 moved NO value.** `docs/plans/post-roadmap-oracle-match.md`. The phenology
sciences shipped: **vernalization + photoperiod** (clean-room, Soltani & Sinclair 2012 Ch.
8/7), `phenology.yaml` 4→12 cited params, a 2nd aux accumulator, 12 goldens + manifest
regenerated, hand-mirrored into Rust, full suite + `-m slow` green, `git diff src/simcore/`
empty. **The plan's own scope was falsified four times over — the value is the findings:**
(1) scope (A)'s "phenology and canopy are independent, *and vice versa*" is FALSE — the
coupling is one-directional (`Allocation` reads DVS), so fixing phenology raised peak LAI
**9.9×** and the "dominant, structural" canopy collapse **dissolved with no canopy science**
⇒ **juvenile canopy expansion is NOT needed**; (2) vernalization's endpoint match was a
**FALSE POSITIVE** — the oracle uses **photoperiod**, not cold arrest (its DVS climbs
*through* winter; the multiplier tracks daylength at r=0.972 and keeps rising *after* our
cold saturates), caught by an advisor "what mechanism does the oracle actually use" check ⇒
**an endpoint match is not evidence of a mechanism, and an endpoint-based bar would have
rewarded the wrong physics**; (3) the perennial **period-2 limit cycle was a broken-canopy
artifact** — either science alone flattens the return map to a period-1 fixed point
(converges UP); (4) the frozen scenarios were **co-adapted to the starved plant** — the ~5×
bigger plant over-drew the CONSUMER chamber's CO₂ (rationed under Euler, hard-errored under
RK4), fixed by a **consumer-specific 2× enlargement** (all 3 gas pools scaled so Ci₀=250 and
x_O₂=0.21 both invariant; `air_mol` feeds BOTH mole fractions — advisor caught that
carbon-only scaling would silently change respiration). The **META-FINDING took its 11th–?
instances here**, the first *scientific* ones: a claim written as symmetric ("and vice
versa") when the evidence was one-directional; the tell is the phrase doing unearned work.
The Rust mirror **surfaced a genuine cross-port bug** (the reset didn't re-zero the
vernalization accumulator ⇒ year-2 crops skipped arrest). **THE BAR IS DECIDED (user):
literature-ranges-only; the oracle is a DIAGNOSTIC, never a fit target** — retiring the
Phase-1 "clean-room forbids backfitting" tension. **Residual = cause 3, the `tsum` phase
partition** (reproductive phase 43 d vs the oracle's 75 d). **CEREMONY 2 (2026-07-20) — the
recalibration premise was FALSIFIED; NO value/golden/code moved, only two `source:`
strings** (an honor-system provenance unfreeze — `phenology.yaml`'s manifest hash moved,
Rust `biosphere_params.txt` byte-identical). **(a)** Both `tsum` values are **already
literature-centred** — first-hand to Penning de Vries 1989 (Tables 12 & 15, read off the
page images; [E] gives a rate-constant, so the °C·day sum is a documented base-temp-free
derivation = phase duration at a constant 20 °C): `tsum_maturity=750` is **dead-centre** of
the winter-wheat range [727, 784], `tsum_anthesis=1100` ∈ [1026, 1333]. The oracle's implied
TSUM2 (~1207, its 75-day grain fill) is ~1.5× above these — **cultivar variation** (a
longer-grain-fill parameterization), NOT our error; matching it leaves the cited range =
backfitting, forbidden by ruling B. ⚠ n=2 cultivars, one lineage — a *cited value*, not the
definitive range (a shelf fact, dated; the meta-finding shape, advisor-caught). **(b)** The
partition is **calendar-impossible anyway** (a 2nd, citation-independent leg): at our
day-251 anthesis — June 9, warm July grain fill, NOT a cooling tail — only ~54 fixture-days
remain, so `tsum_maturity > ~912` **never matures in any season**. **(c)** The maturity
"match" (294 vs 292) is **TWO ERRORS CANCELLING** (+34 d late anthesis, −32 d short grain
fill) — a fake validation, the test renamed + re-documented (ungated-prose-half). **The user
chose to "reopen the double-modulation"** (target the +34 d overshoot, not `tsum`);
measured, removing a term **re-opens the canopy gap** (canopy closure scales with vegetative
length — the overshoot is its *price*) and does NOT close the partition either. There **is**
a citable double-count case (oracle is photoperiod-driven per inc. 1; [E] itself excludes
vernalization for winter cereals) — but June 9 is a realistic anthesis date, so under ruling
B "+34 d vs a non-target" may not be a defect. **Resolved by the user's "why not both": the
tension was never model-level** (vernalization is optional by design; loaders take a
crop-file path), only that there is ONE frozen reference crop — so **keep the validated
winter wheat (both terms) untouched, and add a day-neutral/photoperiod-only crop as AUTHORED
content** (lamp-controllable via `lighting.py`; "authored ≠ validated") when the warm closed
habitat needs one. **Open after ceremony 2:** the day-neutral habitat crop (small plumbing,
no frozen-science change); the partition residual is now a *permanent, explained* pin in
`test_oracle_gap.py`, not a to-do. Scope (C)'s decomposer rates + `n_senescence_rate`
form-gap remain separate open pieces
