# Stem-reserve remobilization — REFUSED (2026-08-10), then BUILT (2026-08-12)

> **Status: BUILT.** The refusal below stands as written and was *overruled by the user*,
> which is what it asked for: its blocker was a **provenance judgement** (`fstr` is CABO
> unpublished; the trigger is ours), and this project's standing rule is that my own
> refusal is a recommendation, not a verdict. Everything from "The question" to "The
> verdict" is the 2026-08-10 record, unedited. What was built, and the **cessation window**
> the user asked for afterwards, are in the two sections after it.
>
> Sections 7–8 (the build) carry the current state; **23 pins** in
> `tests/test_stem_reserves.py`, 13 goldens, the biosphere manifest.

Read-only *as of 2026-08-10*: no `src/`, param, golden or manifest change; `git diff src/`
empty; nothing unfrozen. Probes in `M:/claud_projects/temp/stem_reserves/`; **18 pins** in
`tests/test_stem_reserves.py` (2 slow). Teeth verified by **mutation**: changing the one
load-bearing number from Table 7's wheat row (0.40) to barley's (0.30) takes **12 of 18**
red.

## The question, and why it was asked

The user's question was the plain one, and it is a good one: *real wheat stems grow, then
the plant sheds its seed and the stems and ungerminated seeds die and decompose — is that
what our simulation does?*

Half of it is. `annual_reset` already kills the whole plant to litter each year,
including all leftover grain. What is missing is the other half.

## What the frozen tree actually does

**Our stem never stops growing.** Measured on `open_season`: stem carbon at flowering
12.04 mol C, at the end of the season 19.48 — **+62 %**, and it is still gaining within
days of the last step. Real wheat's stem peaks around anthesis and then *loses* weight as
its stored carbohydrate moves into the filling grain.

**And there is no path for it to move.** Stem carbon can only be shed to litter or
burned; it can never become grain. So "the stem feeds the seed" is not a mistuned
parameter here — the mechanism is **absent**.

⚠ Three counts, and only one of them is the quantity — measured rather than read off the
declarations, because the first count I wrote down was wrong. **Five** flows *reference*
`stem_c` (growth respiration and nitrogen uptake only read it, for the shared carbon
budget and for the nitrogen demand's denominator). **Three** ever emit a leg on it. But
at any single mid-season step only **two** do, because the maintenance draw is
CONDITIONAL — it opens only on days when assimilation does not cover upkeep. The pin
measures all three over the whole trajectory and asserts them apart.

⚠ And the frequency of that conditional door is **scenario-dependent, so it too is
measured on two scenarios rather than stated flat off the one in hand** (an advisor
catch): it is open on **9.5 %** of `open_season`'s days and **63.7 %** of
`sealed_chamber`'s — a factor of ~7. A one-snapshot count is therefore not merely wrong,
it is *unstably* wrong: in the sealed chamber a random snapshot would usually have said
three. In neither scenario is it always open, which is the part that does generalise.

The consequence is visible against the committed oracle fixture (`allocation.py`'s own
docstring: TWSO ≈ 11.5 of TAGP ≈ 20.4 t/ha ⇒ harvest index ≈ 0.564). The frozen crop's
grain **fraction** is **0.84×** that, and its grain **mass** is **0.52×**. ⚠ Those are two
different quantities and are stated apart deliberately — the mass-vs-fraction conflation
this repo has logged twice.

## The science, first-hand, and it was already on the shelf

[A] Penning de Vries et al. 1989 — the book `allocation.py` already cites. Extraction of
the shelf copy; the quotes below are verbatim.

**§2.2.2 "Temporary storage" (pp. 46–47)** and its **Table 7**:

> "At flowering, 20 % or more of the weight of vegetative organs may consist of
> mobilizable starch, particularly in cereals. **There is little data published on stem
> reserve contents around flowering**, but Table 7 shows some indicative values."

Table 7's caption: *"The fraction of stem weight at flowering consisting of remobilizable
carbohydrates (starch, sucrose plus glucose). **Data are unpublished results provided by
scientists at the Centre for Agrobiological Research (CABO), Wageningen**, unless
indicated otherwise."* — **Wheat 0.40**, with no "Source estimate" annotation, i.e. it is
the CABO unpublished column. Range across the table: 0.1 (cotton, sunflower, millet,
tulip) to 0.5 (sugar-cane).

> "A simple view is that redistribution starts **once stems stop growing**, and then
> continues at a rate of **0.1 d⁻¹** of the redistributable starch (Listing 3 Line 35)."

⚠ **A provenance distinction worth keeping straight.** The book's explicit disclaimer —
*"This level and rate are **chosen without an experimental basis**, but in many cases
yield a reasonable pattern of stem weight loss"* — attaches to the **alternative** (L1Q)
hypothesis, which triggers on the storage organ's growth rate and happens to carry the
same numeral 0.1. The simple view's 0.1/day is stated bare with a code-line pointer and no
citation. **Uncited is not the same as self-disclaimed**, and writing the stronger one
would be a locus error of exactly the kind the (C) diagnosis logged.

**§3.2.4 "Formation of shielded reserves" (p. 93)** — the formation half, and it corrected
the design drafted before it was read:

> "A simple way to deal with the formation of shielded reserves is to assume that **a
> certain fraction of the increase in stem weight will be available for redistribution
> after flowering** (Listing 3 Lines 17, 35). This fraction is assumed to consist only of
> starch. Some data on the magnitude of the remobilizable fraction are given Table 7."

So [A] does **not** snapshot 40 % of the stem at flowering: the reserve accumulates
continuously as a fraction of stem *growth*, and the stem's dry weight is structural +
starch throughout. Corroborated by the book's own exercise S6 ("the final weight of
structural biomass of the stem changes in accordance to the stem reserves"; "sensitivity
of rice grain yield for the fraction stem reserves (0.0 to 0.5)").

**§3.2.6 (p. 95)** — *"except for their reserves, stems do not lose weight"* — the
sentence this repo already recorded, now with its other half: a stem's weight loss **is**
the reserve draining. So the physically coherent form is the reserve path **plus**
`rdr_stem = 0`, which is the stem-only branch already priced and refused.

## The candidate

A new POOL stock `stem_reserve_c` (carbon, plants domain, starts 0):

* **Fill** — a fraction `fstr` of the frozen `Allocation`'s own stem leg is diverted.
  Written as a post-process of the frozen flow's legs, so the partition maths cannot drift
  from the frozen one.
* **Drain** — `stem_reserve_c → storage_c` at `0.1/day` once `DVS ≥ 1.0`. Donor-controlled
  and therefore self-limiting; the Euler backstop is structurally unreachable on it.
* **Maintenance / f_N** — the reserve sits outside `leaf + stem + root`, the same
  treatment `storage_c` already gets, for the same recorded reason (a storage carbohydrate
  is not respiring tissue and carries no nitrogen). Measured as its own variant rather
  than folded in silently — see finding 5.
* **Senescence** — the reserve is not shed.
* **Annual reset** — the reserve dumps to `litter_carbon` with the rest of the dead plant.
  `run_season` re-asserts conservation across every reset, so a mistake here hard-fails.

⚠ **The trigger is OURS, and it is labelled as ours.** [A] induces remobilization "once
stems stop growing", which **can never fire in this tree** — `fs` is 0.10 at DVS 2.0 and
the table flat-extrapolates, so our stem grows for as long as there is assimilate. The
substitute is the weaker *availability* condition §3.2.4 states in its own words
("available for redistribution after flowering"), i.e. `DVS ≥ 1.0`.

---

## FINDING 1 — the SOURCED form fixes the stem and overshoots the grain; the form that lands on the grain is OURS

| form | stem shape (end ÷ flowering) | harvest index ÷ oracle |
|---|---|---|
| frozen | **1.618** (grows 62 %) | 0.840 |
| §3.2.4 growth fraction — **what [A] programs** (Listing 3 Lines 17, 35) | **0.985** (stops gaining) | **1.151** |
| one-shot at flowering — **OURS**, no listing line in [A] | 1.349 (still +35 %) | 1.000 |

⚠⚠ **The two are NOT symmetric in provenance, and an earlier draft of this document said
they were.** It called them "two readings of the same source ... both are [A]". They are
not. Every formation pointer in the book is one of **two programmed models** — Listing
3's growth fraction, and Listing 4's sink-limited overflow ("adding those carbohydrates
that growing organs cannot absorb", Lines 32–33, 37–38) — and Table 7 is cited *into the
first* as the source of its parameter ("some data on the magnitude of the remobilizable
fraction are given Table 7"). **A data table is not a model form.** Established by
extracting every `Listing 3` / `Listing 4` pointer in the book and reading its context,
not by skimming.

So: the form the book actually programs **misses** the harvest index by 15 %, and the
form that **hits** it is a reconstruction of ours. That makes the refusal stronger, not
weaker, and it is the Greenwood precedent repeating — reading the primary *dissolved* the
fork instead of balancing it.

A consistency check on the growth-fraction reading, worth recording and worth not
overselling: filling at 0.40 of stem growth produces a stem that is **0.4201** starch at
flowering — Table 7's own quantity, reproduced as a *consequence* rather than imposed. ⚠
That is near-tautological (fill at 0.40, stand at ≈0.40) and is pinned as a consistency
check, not a validation: the two coincide only while the stem's losses are small.

⚠⚠ **THE ORACLE HARVEST INDEX IS NOT A TARGET AND WAS NOT USED AS ONE.** Two sampled
variants land within a few parts in ten thousand of it — our reconstruction at
`fstr = 0.40`, and the sourced form at `fstr = 0.20`. **That is a coincidence
of where the sweep was sampled**, it is pinned with that label so it cannot later be
quoted as a match, and `fstr = 0.20` is **Sorghum's** row, not Wheat's. The standing
ruling — the oracle is a diagnostic, never a fit target — is what makes tuning `fstr`
until the harvest index lands the refused shape, and it is refused.

## FINDING 2 — THE STRUCTURAL ONE: the partition table is the blocker, and it is uncited

The reserve is fighting `allocation.yaml`. The stem fraction is **0.10 at DVS 2.0** and
the interpolation **flat-extrapolates**, so 10 % of every day's growth goes on being
routed to the stem right through grain fill and past maturity. A reserve can move carbon
out of the stem; it cannot stop the allocation putting it back.

**[A]'s trigger is not merely unfireable here — it is a statement about [A]'s partition
table**, in which the stem fraction reaches zero. Ours does not, and the file itself flags
the whole table `TODO(cite) — provisional, literature-typical … primary citation pending`.

⇒ the mechanism is blocked by a *different* missing piece, which is the (C)-diagnosis /
canopy-regulator shape a third time. With one difference that matters: **this mechanism is
not inert.** The canopy regulator was bit-identically inert on the frozen tree; this moves
grain by half and passes every gate. The refusal is *"what it rests on is uncited"*, not
*"it does not work"*.

## FINDING 3 — the provenance ranking of the three numbers is exactly inverted against what matters

| number | what the book gives | measured effect |
|---|---|---|
| `fstr` = 0.40 | **tabulated** for wheat (CABO, unpublished) | the only one that moves anything |
| rate = 0.1/day | stated bare, no citation | **bit-inert on carbon** |
| trigger DVS ≥ 1 | **ours** — [A]'s cannot fire | near-inert; peak LAI bit-identical |

**The drain rate is bit-inert**, checked at `to_bits()` over every stock at every step, not
at printed precision: across rates 0.05, 0.2 and 1.0 the *only* stocks that differ from the
0.1/day run are the starch and the grain. The mechanism is plain — once carbon is in the
reserve it is already outside maintenance and outside senescence, and grain is too, so
moving it between them is a **rename**. (The stock allowed to differ is asserted to
actually differ, so the check cannot pass by the runs being identical for a trivial
reason.)

⚠ One exception, measured rather than reasoned about: at `rate = 0` — a degenerate form,
not a candidate — the grain is ~10.7 mol C smaller, and grain sits inside Greenwood's `W`,
the denominator of the nitrogen *target*; a smaller `W` raises the target, raises the
demand, and the plant takes up more nitrogen. Every non-zero rate leaves nitrogen
bit-identical.

**The trigger is near-inert too**: sweeping DVS 0.0 → 1.5 moves final grain by 0.7 % and
leaves peak LAI **bit-identical**, while the standing starch at its maximum varies 4.8×.

⇒ the two numbers that would have been the provenance problem turn out not to matter, and
the one that does is the one the book tabulates. That is a genuinely good position to be
in and it is *not* what makes the answer "no".

## FINDING 4 — the extra grain is the TRANSFER, not the two exemptions

Three mechanisms are entangled: (a) the transfer starch → grain, (b) starch being outside
the maintenance biomass, (c) starch not being shed at `rdr_stem`. Turning **both**
exemptions off — starch treated exactly like stem carbon in every respect except where it
eventually goes — still gives **+49.6 %** grain against the full form's +53.5 %. The
mechanism does what it says on the tin.

Whole-season carbon accounting on `open_season` (mol C): the system total barely moves
(133.95 → 136.78), litter falls 24.42 → 23.34, respiration falls 53.50 → 51.18, and grain
rises 22.41 → 34.40. It is a **redistribution**, not extra photosynthesis.

## FINDING 5 — closure holds, on the whole roster and on the station

The gate every biosphere science change has actually been judged by. Controls reproduce
the record before any subject reading is trusted (frozen `perennial` trough **0.055175**,
frozen fixed point **0.634352**, stem-only's failure **0.046065** — so the harness is known
to be able to report a failure before it reports a pass).

Reserve-only (`fstr` 0.40, rate 0.1, trigger DVS ≥ 1):

* `rationed == 0`, no extinction events, in `sealed_chamber` (3 yr), `water_biting` (1 yr),
  `perennial` (5 and 15 yr), `consumer` (5 and 15 yr), **and under RK4** — the integrator
  that killed the full (C) form on this same chamber.
* ⚠ **Our reconstruction was measured too, not left as an unmeasured leg** — `rationed
  == 0` and no events on all four chambers and under RK4. Its **discrete one-shot switch**
  is the specific reason not to have assumed RK4 would be fine with it: a state-dependent
  switch is what a multi-stage integrator handles badly. A detail worth keeping: in the
  two shedding-fed chambers it leaves the CO₂ trough **bit-identical to frozen**, because
  the trough happens before the single fill event ever fires.
* ⚠ **The frozen roster is SEVEN scenarios.** Four are in the closure loop above,
  `open_season` is the bands bullet below, `perennial_long_horizon` /
  `consumer_long_horizon` are the same scenario objects at a longer horizon, and
  `drift_summary` is **derived** from the two long-horizon runs rather than being a run of
  its own — its inputs are measured and it would move with them. Checked against the
  manifest, not against the loop's own length; this is the shape logged three times
  already in this repo's record.
* `perennial_long_horizon`'s four manifest gates all pass: CO₂ floor (trough **0.055977**,
  *above* the frozen 0.055175), CO₂ stationarity, the leaf floor, and the liveness floor
  (**0.637424** > 0.55).
* `open_season`'s outside-sourced bands hold — peak LAI **5.4624**, inside real wheat's
  5–8 and below the Van Keulen & Seligman threshold of 6; Greenwood's `W` **14.1516 t/ha**,
  below the 14.4248 crossing. ⚠ Both pass; **neither passes comfortably** — the LAI
  clearance goes 86.5 % → 91.0 % of the threshold and `W` goes 87.6 % → 98.1 % of the
  crossing. At Table 7's **top** row (sugar-cane, 0.50) `W` crosses.
* `sealed_station` (4 yr, two-rate): `rationed == 0`, no events, grain 24.358 → 39.123.
  ⚠ This leg was recorded **unmeasured** by the stem-only work because the biosphere gate
  failed first there; here the biosphere gates pass, so it is not moot and it was run.
* `n_limited` **keeps the regime it was built for** — the one place `f_N` bites, and the
  reserve takes carbon out of `f_N`'s own denominator, so this had to be measured. Minimum
  0.178930 over 186 steps against the recorded 0.175851 over 187: weakened under 2 %.
  Option (A) deleted this knob; this candidate does not.
* **Option (B)'s litter C:N identity survives.** Starch is nitrogen-free and is exactly
  the thing that could break it. In the shedding-fed chambers the starch never reaches
  litter at all (it drains to grain) and the pool moves **1.4 %** (102.75 → 104.22); in the
  reset-driven ones it is dumped with the dead plant and the pool moves ~18 % (10.89 →
  12.80) — *toward* real residue, since that regime's C:N of ~10 is this tree's recorded
  limitation 5.

⚠ **The maintenance treatment is a fork and it is named rather than buried.** The variant
where starch pays maintenance **fails** CO₂ stationarity on `perennial_long_horizon` while
the exempt variant passes. Choosing the exempt form *because* it goes green would be the
refused shape; it is chosen because `storage_c` is already exempt for the same stated
reason, i.e. on an independent argument that predates this work. Both are recorded.

## FINDING 6 — it rescues one of stem-only's two surviving closure legs, and that does not reopen stem-only

Stem-only fails `perennial_long_horizon`'s CO₂ floor (0.046065) **and** its stationarity.
Reserve + stem-only gives trough **0.053127** — above the 0.05 floor — while stationarity
still fails. ⚠ Recorded as a measurement, **not** as a re-opening: re-deciding a refusal
inside the work that moved the tree underneath it is the shape this project refuses (the
CUE build's own precedent). Reserve + stem-only also **crosses** the Greenwood tripwire
(`W` = 14.76 t/ha vs 14.4248), which the reserve alone does not.

---

## The verdict

**Not built.** The mechanism is real, sourced first-hand from the book we already cite,
closure-safe on the whole roster and on the station, and it halves the largest remaining
gap to the oracle's grain. What stops it is finding 2: **its own source's trigger is a
statement about a partition table we do not have**, our table is flagged uncited and
provisional, and the sourced form misses the harvest index by 15 % *because* the table
keeps refilling the stem. The form that would land on the reference is one we
constructed, and picking it on that basis is the refused shape.

**The natural successor is therefore not this mechanism but the partition table** — a
DVS-keyed `fs` that reaches zero, cited to a primary. With that in place [A]'s own trigger
becomes fireable, and the choice of formation form stops being ours to make.

Price if taken later, recorded so it is not re-derived: a new stock and two new flows ⇒
biosphere `flow_set` + `param_files` + a new param file, every carbon golden, the station
manifest, `biosphere_params.txt`, the Rust mirror, the cross-port tier.

Also standing, unmeasured and named as such: whether a partition table whose `fs` reaches
zero *by itself* — with no reserve at all — fixes the stem shape. It would shrink the stem
without giving its carbon to the grain, so it is a different change with a different sign
on the harvest index, and no run here measures it.

---

# 7. THE BUILD (2026-08-12) — what shipped, and the one scenario it re-sized

The candidate above, built as described, with `git diff src/` no longer empty because that
purity rule is a *port/consumer* rule and this is reference science.

* **`stem_reserve_c`** — a new POOL stock (not a POPULATION: `organ_stock`'s extinction
  pass zeroes-with-loss, and a carbohydrate store is not a population that can go extinct).
  Starts **empty**, with no scenario field: a seedling has had no stem growth, so a
  settable initial amount would be a number no source gives.
* **The fill lives inside `Allocation`**, as a split of its own stem leg, and that is a
  **correctness choice rather than a tidy one**. A separate flow would have to *withdraw*
  from `stem_c`, and `simcore.arbitration` scales withdrawals against the **start-of-step**
  amount — at emergence the day's stem growth can exceed the whole seedling stem, so such a
  flow would ration. Splitting the deposit cannot: `organ_total` is unchanged, so the CO₂
  and O₂ legs never move.
* **`StemRemobilization`** — the drain, first-order and therefore donor-controlled, so the
  Euler backstop is structurally unreachable on it.
* **`annual_reset`** dumps the standing reserve to litter with the rest of the dead plant,
  through the **balancing residual** (computed, not formulated), so it cannot leak.
* **`SeasonScenario.stem_reserves`** — a boolean, not a value, and the distinction is
  load-bearing: `crop_param_set` falls a missing file back to the reference, so a
  value-only switch would hand **every** second species wheat's tabulated 0.40 by default.
  **Potato turns it off** — [E] Table 7 gives potato **"0.2-0.4"** where wheat gets a
  single 0.4, and picking inside someone else's range is our number wearing their name.

⚠ **`SEALED_CHAMBER_SCENARIO.litter_carbon0` 3.0 → 3.5, because the build ABOLISHED the
phenomenon that scenario exists to show.** More carbon fixed ⇒ more O₂ released (PQ = 1)
exactly where the trough forms, and the pool bottomed at **5.08 %** of its fill against a
**≥ 95 % depletion** contract — which also made `f_O2` stop being load-bearing (the
`k_O2 = 0` control no longer rationed at all, so the test that proves the self-limit does
the work would have been asserting nothing). Peak litter and peak microbial biomass are
**unchanged to four figures**, so this is not "less decomposition" — measured, because the
obvious explanation was the wrong one. The sweep, and why the *litter seed* moved rather
than the O₂ fill, are recorded in `scenario.py` beside the value.

# 8. THE CESSATION WINDOW (2026-08-12) — "the stem should stop feeding the seed"

The user's second question about this mechanism, and the answer is smaller and sharper than
the question sounds.

## 8.1 What was actually wrong

As first built, **neither half ever stopped**. The drain fired at every step from anthesis
onward; the fill kept diverting starch for as long as `allocation.yaml` fed the stem, which
is forever (finding 2). Measured before ranking anything:

| | `open_season` | `sealed_chamber` | `perennial_long_horizon` |
|---|---|---|---|
| anthesis / maturity | step 251 / **294** of 305 | 251 / 294 | 251 / 294 |
| reserve, anthesis → year end | 5.490 → **0.495** | 0.317 → ~0 | 0.286 → 0.007 |
| remobilized **after maturity** | **6.6 %** | 7.0 % | 3.0 % |

⚠ **The "perpetual conveyor" the advisor predicted does not exist**, and the arithmetic
that predicted it (`R* ≈ 0.4·DMI`) is wrong because post-anthesis `DMI` collapses while the
0.1/day draw does not: the reserve peaks at anthesis and **drains 91 %** by season's end.
The defect is therefore *not* "it never stops" — it is that **the rule** never stops, and
3–7 % of the transfer happens **after the crop is physiologically dead**. `sealed_chamber`
is the sharp case: it never re-sows, so it spends **two full years** past maturity.

## 8.2 Four candidates, measured on `open_season`

| variant | grain | HI ÷ oracle | stem+starch shape | peak LAI | `W` |
|---|---|---|---|---|---|
| as built (no stop) | 34.396 | 1.151 | 0.985 | 5.4624 | 14.1516 |
| (a) drain stops at maturity | 33.726 | 1.128 | 1.037 | 5.4624 | 14.1516 |
| (b) fill stops at anthesis | 28.866 | **1.002** | **1.267** | 5.4509 | 13.6444 |
| (c) (b) + (a) | 28.825 | **1.000** | 1.270 | 5.4509 | 13.6444 |
| **(d) BOTH stop at maturity** | **33.714** | 1.128 | 1.036 | 5.4624 | 14.1457 |

**(b) and (c) are refused, on two independent grounds.** They hand the stem back its
post-flowering growth and so **reinstate the defect this mechanism exists to fix** (shape
0.985 → 1.267 against a frozen 1.618); and they land the harvest index **on the oracle**,
which is the refused shape by the standing ruling — the same trap `fstr = 0.20` set
earlier. ⚠ Worth stating plainly: they are *also* physically worse, so the oracle
coincidence is not even a consolation. Recorded, with [E]'s own §2.2.1 p. 46 sentence
("glucose formed **before flowering** and stored as polysaccharides") that makes (b) a
readable interpretation — in [E]'s tree the two readings **cannot be told apart**, because
its stem fraction reaches zero and no stem growth survives flowering anyway.

**(a) vs (d) is decided by 0.03 % of grain**, i.e. not by numbers. It is a coherence
question — (a) leaves the dead plant stashing starch it can never withdraw — and it was
**put to the user, who chose (d)**.

## 8.3 The bound is [E]'s, and its exact strength

Extraction of the shelf copy, not a skim. **Listing 3 Line 114** — the run-control line of
module L1D, the module whose Lines 17 and 35 *are* this mechanism:

    114  FINISH DS = 2., CELVN = 3.

and twice in the prose:

> §3.1.4 p. 81: "This simulation of the development process contains no explanatory
> mechanism for the crop ripening. **Simulation is halted by imposing an end when the
> development stage reaches the value of 2.0**, by including the statement FINISH DS = 2.0,
> or FINISH DS = 0.95 for sugar crops and for sweet potato that are harvested before
> flowering."

> §3.4.2 p. 105: "The FINish TIMe of 1000. is never reached: **simulation always stops when
> either the FINISH conditions of maturity (DS = 2.0)**, or that of severe carbohydrate
> shortage (CELVN = 3.0) is reached."

⚠⚠ **READ AT ITS EXACT STRENGTH — this is the locus error this record has logged three
times, and it would have been easy to commit here.** `FINISH` is a **run-control**
statement. [E] does **not** say "remobilization ceases at maturity"; it says **its program
has no state there**. So `cessation_dvs` is the source's **domain boundary**, and adopting
it is a decision *not to extrapolate a form outside the program that defines it* — not a
cited cessation rule. The question only arises in our tree because we have no `FINISH`: DVS
merely **caps** at 2.0 and the season keeps stepping.

That cap is also why the loader refuses `cessation_dvs > 2`: such a value would not
postpone the cessation, it would **silently restore the unbounded behaviour** while reading
like a deliberate choice, and no golden could tell the difference.

## 8.4 What it cost, honestly

* **Grain −2.0 %** on `open_season` (34.396 → 33.714).
* ⚠ **One claim got WEAKER and is re-pinned rather than restated.** The build's headline
  was *"the stem stops gaining"* (shape 0.985). It now **gains 3.6 %** (1.036), because the
  fill stops at maturity while the partition table goes on handing the stem 10 % of every
  day's growth for the 11 remaining steps. Against a frozen stem that gains **61.8 %** the
  headline survives — "+62 % → essentially flat" — but the literal sentence does not, and
  `test_NEITHER_form_fixes_both_halves` now asserts the **ordering against the frozen
  control** instead of `shape < 1.0`. The residual is the partition table again, not this
  mechanism.
* ⚠ **The trigger got MORE load-bearing**, which is the sort of thing a window does and is
  worth having measured: with no end, the drain ran to the last step whatever the trigger,
  so the DVS 0.0–1.5 sweep moved grain **0.7 %**; with an end, the trigger sets the
  window's *length* and the same sweep moves it **2.3 %**. Still near-inert, still
  bit-identical on peak LAI, and the loss is **one-sided** — only the late trigger that
  eats into its own window costs anything.
* Every frozen band and closure gate holds; the two 50-year diagnostic attractors moved in
  their last digits (trough 0.0736507 → 0.0736681, fixed point 0.593864 → 0.593883) and
  were re-measured rather than re-toleranced.
* ⚠ **§7's gate readings are SUPERSEDED by these, and are kept as the build's own
  measurement rather than silently rewritten.** The window moves every one of them a little:
  `perennial_long_horizon`'s CO₂ trough 0.055977 → **0.056030** and its liveness floor
  0.637424 → **0.637504** (both still clear 0.05 and 0.55); stem-only's trough in finding 6
  0.053127 → **0.053177** (still above the floor, so finding 6's claim is unchanged); and
  `open_season`'s Greenwood `W` 14.1516 → **14.1457 t/ha**, which is §8.2's row (d) and the
  one figure of the four that a reader could mistake for a band value rather than a
  diagnostic. Peak LAI is **bit-identical** at 5.4624, so the LAI band is untouched. No
  verdict in §7 turns on any of the differences — which is the point of checking rather
  than assuming. ⚠ `W`'s "98.1 % of the crossing" survives the move, which is exactly why
  nothing went red and why the sweep had to be done by hand.

## 8.5 The prediction, written before regenerating

Predicted: the 13 goldens the build had already moved would move again, **no fourteenth**,
nothing non-wheat. Measured: **10** moved; `greenhouse`, `lighting` and `harvest` did not —
and the reason is structural rather than lucky, since those are **7-day** station runs that
end 287 days short of maturity, so the window provably cannot reach them. Potato is
bit-identical (its reserve is off), which is the leak check.

# 9. THE NATIVE MIRROR (2026-08-12) — and the stale scenario it caught

The Rust port of the *whole* mechanism, done after the window so it was mirrored once
against the final form: `STEM_RESERVE_C`, `StemReserveParams` (four values, read from the
generated `biosphere_params.txt`), `Allocation`'s three reserve fields and its split,
`StemRemobilization`, the `stem_reserves` scenario flag, and the `annual_reset` leg —
**203 lines across 5 files**, none of them `simcore`. `cargo test` 38+ suites green,
`cargo clippy --all-targets -D warnings` clean, and only the four touched files were
formatted (never a bare `cargo fmt`, which would reformat the frozen core).

⚠ **THE MIRROR CAUGHT SOMETHING THE PYTHON SIDE COULD NOT.** With every flow ported, one
cross-port test still failed — `sealed_chamber`'s O₂, Rust 0.0910 against Python
0.00021982, a factor of **413**. The cause was not the new code: the Rust
`sealed_chamber_scenario()` still carried `litter_carbon0 = 3.0`, because the build's
**scenario re-size to 3.5 was never mirrored**. The Python-side suite cannot see that —
its goldens are generated from the Python scenario — so the discrepancy was invisible
until the two ports were compared. That is precisely the job the cross-port tier exists to
do, and it is the second time this record has watched a *scenario constant* rather than a
*science change* be the thing that drifted.

Handled under the discipline the contract prescribes: the port has **no reference
authority**, so this was a mirroring of the Python decision, not a re-taking of it, and
the Rust comment says so and points at the file that carries the sweep. All **101**
cross-port tests pass, including the tier-2 band re-measurement (each band is asserted
above its own ±1-ULP sensitivity, so no tolerance was loosened to fit).
