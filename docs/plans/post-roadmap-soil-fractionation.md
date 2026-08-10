# Post-roadmap: soil carbon pool fractionation — DIAGNOSED, NOT BUILT; then RE-OPENED

**DIAGNOSED 2026-08-10. Read-only. No value, golden, param or manifest moved;
`git diff src/` empty; nothing unfrozen.** Probes in `M:/claud_projects/temp/soil_frac/`;
pins in `tests/test_soil_fractionation.py`.

> ⚠⚠ **EVERY NUMBER BELOW THIS LINE WAS MEASURED ON THE CUE = 1.0 TREE, one commit
> before the humification split landed (79bece4).** The split discharged finding 3 — the
> structural blocker that was *this document's stated reason for turning the seam down* —
> and superseded finding 4's window evidence. The re-opening, its prediction written
> before the first probe, and what the re-measurement found are in
> **"THE RE-OPENING (2026-08-10)"** at the end of this document. Read the old findings as
> a record of the tree they were measured on, not as the live price.

The chamber-scale diagnosis's named seam, taken as its own item. That document priced it
and deliberately did **not** recommend it:

> **Soil carbon pool fractionation** — DPM/RPM/BIO/HUM/IOM, or any subset with at least
> one slow pool — is the shape of what would let a chamber hold a realistic carbon
> inventory *without* a proportionally huge CO₂ flux, because `C* = flux/k` **decouples
> once there is more than one `k`**. […] Whoever takes it must name the invariant first.

This document names the invariant, measures what the seam delivers, and turns it down.
**Two of that paragraph's load-bearing claims are measured false, and the second is the
headline: the decoupling is not available at a cited partition, and the pools that would
deliver the census gap cannot be expressed by this tree's structure at all.**

---

## THE VERDICT IN ONE PARAGRAPH

At the partition RothC's own worked example gives, fractionation buys **6.47×**, against
a census gap of **94×**. It does not unblock the science it was reached for: the one
sizing that carries stem-only past both gates on both perennial scenarios is a **window
narrower than 1.0 mol C in an uncited knob, found by sweeping the gate green** — and at
the neighbouring value it *creates* a failure in `consumer` that stem-only alone never
caused. The remaining **14.5×** lives in RothC's BIO/HUM/IOM, which are **formed by the
humification split of decomposed material**, a flux our CUE = 1.0 structure does not have
— measured, not inferred. **So the seam is blocked by the same obstacle option (B) hit,
and that is a fact about the model's form rather than about a sizing sweep.**

---

## The invariant, named first (the increment-1 requirement)

Two sizings were available and **both are principled; neither was chosen by outcome**:

| sizing | what it holds fixed | seeded total | result |
|---|---|---|---|
| constant initial flux | the t=0 CO₂ return `Σkᵢ·Cᵢ` = 12.045 mol C/m²·yr | **19.409** | `rationed = 11` |
| constant inventory | the census total, 3.0 mol C | **3.0** | `annual_reset` hard-errors (re-sow starves) |

The partition itself is **never** free: it is RothC's Hoosfield equilibrium, first-hand.
`litter_carbon0 = 3.0` was sized by probe to make O₂ depletion dramatic and was never a
cited value, so the *seed total* is scenario authoring while the *rates and the partition*
are the science — keeping those apart is what makes sizing the total legitimate rather
than backfitting. **Both principled sizings fail.** Everything below that mentions a
different total is a *sweep*, and is labelled as one.

---

## The sources, first-hand

`sources/RothC_guide_WIN.pdf` (Coleman & Jenkinson, RothC-26.3) — already the decomposer
calibration's source.

**§1.5, p. 9** — decomposition rate constants, "not normally altered when using the
model": DPM **10.0**, RPM **0.3**, BIO **0.66**, HUM **0.02** /yr; IOM inert.

**§1.3, p. 8** — the structure and the input split: *"Incoming plant carbon is split
between DPM and RPM […] for most agricultural crops and improved grassland, we use a
DPM/RPM ratio of 1.44, i.e. 59 % of the plant material is DPM and 41 % is RPM."*

**§3.2, p. 40** — the Hoosfield unmanured-plot equilibrium state on 31 Dec 1851, after
10,000 years at a 1.70 t C/ha/yr input:

| pool | t C/ha | mol C/m² | k [1/yr] | k·C [mol C/m²·yr] |
|---|---|---|---|---|
| DPM | 0.1533 | 1.2763 | 10.00 | 12.7633 |
| RPM | 4.4852 | 37.3424 | 0.30 | 11.2027 |
| BIO | 0.6671 | 5.5541 | 0.66 | 3.6657 |
| HUM | 25.8576 | 215.2827 | 0.02 | 4.3057 |
| IOM | 2.7000 | 22.4794 | — | 0 |
| **Total** | **33.8632** | **281.9349** | | |

⚠ **A retrieval hazard materialised and the visual channel caught it — round 6's
rotated-table finding taking a THIRD instance.** `pdftotext -layout` detaches this
table's label column and shifts the values three rows, so the naive read files
`HUM 0.1533 / IOM 4.4852 / Total 0.6671`. Read off the page render. Two independent
confirmations were available and both were run: the five pools **sum exactly** to the
stated 33.8632, and p. 41 re-states every value bound to its pool name inside worked
arithmetic (`DPM becomes 0.1533 * exp[-10 * 0.3561 / 12] = 0.1140`, …). Round 5's rule —
*a quote check verifies characters; only arithmetic verifies numbers* — is what makes the
extraction recoverable here rather than merely suspect.

**Deliberately NOT taken: p. 41's `(3.51/4.51)` CO₂ vs `(1/4.51)` (BIO+HUM) split.**
That is a carbon-use efficiency. Option (B) measured that introducing a literature CUE
**moves carbon**, re-opening the decomposer calibration under its fast-edge closure
requirement, i.e. costing like option (D). Only the **rate constants** and the
**equilibrium partition** are in scope. This constraint returns as finding 3 — it is not
a scoping convenience, it is the wall.

---

## FINDING 1 — the seam's own premise is false at a cited partition

The seam paragraph says `C* = flux/k` *"decouples once there is more than one `k`"*.
Measured, it does not:

* the cited equilibrium **standing** plant-material partition is **3.305 % DPM**
  (`0.1533 / (0.1533 + 4.4852)`);
* its **aggregate** rate is `Σkᵢ·Cᵢ / ΣCᵢ` = **0.6206 /yr** against our single pool's
  **4.015 /yr**;
* so at t = 0 the seeded stock is `flux₀ / k_agg` — **one number, not a free choice.**

⚠ **Stated precisely, because the tempting stronger version is wrong.** The aggregate
rate is **not** a constant: DPM and RPM drain at 33× different rates, so it decays
continuously from 0.6206 toward 0.3, and *that decay is the tail which is the whole
payoff* (probe 1: return flux 12.045 → 4.17 → 3.09 → 1.26 mol C/m²·yr at 0/1/2/5 yr,
against the one-pool form's 12.045 → 0.217 → **0.0039** → 0). So "fractionation replaces
one `k` with one effective `k`" is **false as an identity** and is not claimed. What is
claimed, and is enough: **the sizing knob is single-valued at the moment you seed it.**
Genuine decoupling requires choosing the partition, which is the fitted thing.

⚠ **An advisor-suggested check turned out TAUTOLOGICAL and saying so is the result.**
The check was: does the cited partition agree with a partition *fitted* to hold our own
flux? At the total the cited sizing produces (19.409) the fitted DPM fraction is 3.443 %
against the cited 3.305 % — but the two constructions **intersect at exactly one total by
construction**, so agreement there is evidence of nothing. The 6.47× inventory gain and
the 6.47× rate ratio are likewise **one number wearing two hats** (`stock = flux/k`), not
two agreeing measurements. Recorded because a corroboration that cannot fail is the
shape this project's meta-finding keeps taking.

---

## FINDING 2 — what it actually buys: 6.47×, against a 94× gap

The chamber-scale census put our seeded litter pile **94× short** of Hoosfield's
equilibrium (3.0 vs 281.9 mol C/m²). Fractionating into the two pools that our structure
can feed closes **6.47×** of that. The remaining **14.5×** is HUM + IOM + BIO — see
finding 3.

Measured on the frozen roster (each scenario driven the way its own golden drives it),
the **form alone at a seeded total of 6.0** is benign and mildly beneficial:

| scenario | yrs | frozen `rationed` / CO₂ tail | fractionated @6.0 |
|---|---|---|---|
| `sealed_chamber` | 3 | 0 / 0.115998 | 0 / **0.122192** |
| `perennial_chamber` | 5 | 0 / 0.038734 | 0 / **0.063384** |
| `perennial_long_horizon` | 15 | 0 / 0.038734 | 0 / **0.062892** |
| `consumer_chamber` | 5 | 0 / 0.144698 | 0 / **0.148887** |
| `consumer_long_horizon` | 15 | 0 / 0.144698 | 0 / **0.148807** |
| `water_biting` | 1 | 0 / 0.084481 | 0 / **0.095119** |

`rationed == 0` everywhere and the CO₂ tail improves everywhere — at **2× the inventory**,
where the one-pool form already rations at 6.0 (measured: `rationed = 6`). That headroom
is genuinely the form's doing.

⚠ **This is recorded as a price, not a proposal.** It moves every carbon golden, both
manifests, `biosphere_params.txt`, the Rust mirror and the crossport tier, for a 2×
inventory against a 94× gap, **with no beneficiary** (finding 4). The canopy-regulator row
is the precedent for writing a real-but-unmotivated result down rather than building it.

⚠ **The roster is checked against the MANIFEST, not against this table's own length** —
the shape that has now bitten this repo three times. `open_season` is absent because it is
**structurally untouched**, asserted rather than assumed: `soil.py` builds the litter and
microbial pools only when `scenario.sealed`, and an open-field build carries only
`boundary.litter_sink`. `drift_summary` is derived.

---

## FINDING 3 — THE STRUCTURAL ONE: our CUE = 1.0 cannot express the slow pools

**This is the finding that survives any later recalibration, because it is about form.**

RothC's Figure 1 and §1.3: **only DPM and RPM take fresh plant input.** BIO and HUM are
*formed* — they are where the humification split routes the decomposed material of every
pool, including their own. IOM never decomposes and never forms.

Our `Decomposition` moves **100 %** of decayed litter C into `microbial_carbon`, with
respiration a separate draw (the deliberate Phase-2 Step-4/5 split). Carbon-use efficiency
is **1.0**. There is no humification flux, so **a slow pool can be seeded but never
refilled.**

Measured rather than read off the figure (`probe7_structural.py`) — a HUM pool at
k = 0.02 /yr, seeded at Hoosfield's HUM scaled like the DPM/RPM seed (33.447 mol C/m²),
run on `perennial_long_horizon`:

| year | HUM [mol C] | Δ |
|---|---|---|
| 0 | 33.447335 | — |
| 1 | 32.892982 | −0.554353 |
| 5 | 30.765938 | −0.518505 |
| 10 | 28.299503 | −0.476938 |
| 15 | 26.030796 | −0.438703 |

**Strictly non-increasing at every one of 4,575 steps.** 77.83 % retained after 15 years
(implied half-life 34.6 yr), and it **never once refills**. It is a one-time inventory
boost that decays away — and it *breaks closure while doing it*: `rationed = 5`, because
the extra ~0.5 mol C/yr drip is drawn from a jar with 11–80 % headroom.

⇒ **The 94× is unreachable by any subset this tree's structure can express.** Closing it
needs the CO₂/BIO/HUM partition of decomposed carbon — a CUE — which option (B) measured
as moving carbon and priced at option (D)'s size. **The seam is blocked by the obstacle
option (B) already hit, one flow over.** That is a strictly better place to be blocked
than "priced, not proposed": it is a measured property of the form, not a judgement about
effort.

---

## FINDING 4 — it does NOT unblock stem-only, and the WINDOW is the evidence

The decisive test (the advisor's framing): re-run **stem-only** — the cheapest of the
three refused changes, with a measured baseline — under **Euler** on `perennial`, which is
where it died (`rationed` 0 → 1 at step 502, min CO₂ 0.008674, decade attractor 0.01619
against the 0.05 floor).

**The controls reproduce the record exactly** before any subject is read: frozen baseline
min CO₂ **0.038734** with per-year minima `[0.074023, 0.038734, 0.054208, 0.054814,
0.054837]`, and frozen + stem-only `rationed = 1` at step **502**, min **0.008674**.

Sweeping the seeded total against **both** gates (`rationed == 0` **and** the 0.05 decade
liveness floor), at 15 years, on **both** perennial scenarios:

| seed | perennial `rationed` / tail | consumer `rationed` / tail | both? |
|---|---|---|---|
| 5.50 | hard error | hard error | no |
| **6.00** | 0 / 0.062667 ✓ | **hard error** | no |
| **6.50** | 0 / 0.063689 ✓ | 0 / 0.148621 ✓ | **YES** |
| 7.00 | 0 / **0.038341** ✗ | 0 / 0.148468 ✓ | no |
| 8.00 | 0 / 0.034296 ✗ | 0 / 0.148648 ✓ | no |
| 10.00 | **11** / 0.009997 ✗ | 0 / 0.133475 ✓ | no |
| 20.00 | 56 / 0.018150 ✗ | 0 / **0.048228** ✗ | no |

**Exactly one value in the swept set passes**, bounded by a hard error 0.5 below it and a
floor failure 0.5 above it. **A window narrower than 1.0 mol C, in a knob with no cited
value, located by sweeping until the gate went green, is the consumer-chamber-2× /
DPM-RPM-labile / ruling-B shape** — and unlike the consumer chamber's 2×, there is no
independent invariant it can be sized on: the two that exist both fail (see "The invariant,
named first"). **Refused.**

⚠ **The sharper form, and it is the one to quote:** at 6.0 fractionation **creates** a
failure in `consumer` that stem-only alone never caused. The frozen tree's
`consumer` + stem-only is *fine* — `rationed = 0`, tail **0.148009**. So on the
neighbouring sizing the change does not remove the refusal, **it moves it to a different
scenario.** Adding a mechanism that turns one passing scenario into a hard error while
rescuing another is not progress against the gate; it is a different collision with it.

---

## FINDING 5 — the N-free seed artefact becomes PERMANENT, and (B) depended on it washing out

Option (B) established that the litter pool's C:N equals the C:N of the material that
fell in — an **identity**, not a band — and recorded the committed excess above it
(`sealed_chamber` ends at 90.6 against the shed ratio 90) as *the N-free seed*: the
chambers seed `litter_carbon0` with **no `litter_n0` counterpart**, a seam (A) named.

The N design here preserves that identity **exactly**. With one `litter_n` against two
carbon pools, N must leave on the **aggregate** flux:

    d(C)/dt = −(k_d·C_d + k_r·C_r)      d(N)/dt = −N·(k_d·C_d + k_r·C_r)/C   ⇒   d(N/C)/dt = 0

Carrying it on one pool's flux instead would break it. **Measured with the seed removed
(`litter_carbon0 = 0.0`), the pool C:N equals 90 at every step to 2.8e-15 relative
(fractionated) and 1.6e-15 (one-pool)** — so the aggregate-flux transfer is right, and
the advisor's alternative (fractionating N into `dpm_n`/`rpm_n`) is **not owed**: it costs
two extra stocks for the same result, and the two pools' C:N could only diverge under a
differentiated input C:N, for which there is no source.

⚠ **But with the seed present, fractionation makes the artefact permanent** — measured on
the shedding-fed chambers, where this identity is the governing one:

| scenario | frozen 1-pool @3.0 | fractionated @3.0 | fractionated @6.0 |
|---|---|---|---|
| `sealed_chamber` peak C:N | **100.55** (1.12× shed) | 271.70 (3.02×) | **334.02** (3.71×) |
| `water_biting` peak C:N | **98.68** (1.10× shed) | 369.89 (4.11×) | **474.80** (5.28×) |

The mechanism is the payoff read on the N side: the one-pool form drains the N-free seed
at 4.015/yr so it is gone within a year and the pool converges to 90; under fractionation
**96.7 % of that seed lands in RPM at 0.3/yr** and lingers for the whole run. **The very
tail-persistence that is the seam's benefit is what preserves the artefact.**

⇒ the seam **owes `litter_n0`**, and — recorded because it is a fact about already-frozen
work — **option (B)'s headline result quietly depends on the seed washing out fast.** It
is a scenario-authoring fix, not a form fix, but it is not optional here: taken without
it, this change would regress (B)'s "~1.25× wheat straw" to 3–5×.

---

## FINDING 6 (method) — I committed correction 2's own error, one option later

Probe 7 measured pool C:N at `peak litter_n` on **`perennial`** and compared it to the
shed ratio. `perennial` is **reset-driven**, so its `peak litter_n` is the **annual dump**
— whose C:N is set by the dying plant, not by the shed ratio — and the number came back
**0.20× shed**, i.e. N-*rich*, in the opposite direction from the N-free-seed explanation
I had written into the probe's own output.

That is verbatim the repo's already-logged correction 2: *"peak `litter_n` silently names
TWO DIFFERENT EVENTS — the seasonal senescence maximum in a shedding-fed chamber, versus
the annual dump in a reset-driven one."* The identity is a claim about the **shedding-fed**
regime; finding 5 is measured there (`sealed_chamber`, `water_biting`, `run_season`, no
reset). **A logged correction does not inoculate the next piece of work against its own
shape** — what caught it was the number coming back on the wrong *side*, not the
discipline.

Also verified rather than assumed, since every number above rests on it: `SplitSenescence`
re-targets the litter leg without re-scaling it — `dpm_leg + rpm_leg` equals the
per-organ carbon `NitrogenSenescence` independently recomputes, **bit-exactly (0.000e+00)
across 218 sampled steps**. Had the split changed the total, every measurement here would
be wrong in a way no gate in the probe would have caught. Conservation across the custom
two-way reset is carried by `run_season`'s own `assert_conserved`, which is what makes the
hard errors at 5.5 real starvation rather than a probe leak.

---

## What this does NOT claim

* **Not** that fractionation is wrong science. It is RothC's structure, and at the cited
  partition it is measured benign on the whole sealed roster with a better CO₂ tail.
* **Not** that the chamber's soil is now realistic — 6.47× of 94× leaves it ~14.5× short,
  and finding 3 says the rest is structurally out of reach.
* **Not** that stem-only, option (C) or option (D) are re-refused on new grounds. Their
  refusals stand where they were; what is added is that **this seam does not lift them.**
* **Not** that the decomposer carbon rates are cited. `decomposition_rate` still runs at
  the fast edge (4.015/yr, Olson's fastest ecosystem) and the DPM/RPM pair would *replace*
  it with two cited rates — which is the one genuinely attractive by-product here, and is
  not on its own worth the cascade.

## The price, recorded so it is not re-derived

New stocks and flows ⇒ biosphere `flow_set` + `param_files`; every sealed carbon golden;
`drift_summary`; the station manifest and its sealed-biosphere goldens;
`biosphere_params.txt`; the Rust mirror; the crossport tier. Plus `litter_n0` (finding 5).
Against: a 2× inventory, no beneficiary, and a structural ceiling at 6.47×.

## The seam that replaces this one

Not fractionation — **the humification split (a CUE)**, which finding 3 identifies as what
actually decouples stock from flux in this tree, and which option (B) already priced as
carbon-moving. It is the same wall option (D) faces, approached from the soil side. Anyone
taking it prices it as (D), not as this.

## Pins

`tests/test_soil_fractionation.py` — originally **21 test functions, 24 collected tests**
(the roster pin is parametrized ×4), of which **10 were `slow`**; the re-opening added 5
functions, so the file now stands at **26 functions / 29 collected / 15 slow**.
Read-only, no fixture, no unfreeze. ⚠ Both numbers are given because they are different
quantities: the first draft of this line said "24 test **functions**", a count lifted
from pytest's output and relabelled — this repo's most-repeated shape, caught here before
it shipped. The claims they carry:

1. the RothC constants and the arithmetic that authenticates the scrambled table
   (five pools sum exactly to the stated total);
2. the aggregate rate 0.6206/yr and the 6.47× — asserted **as one quantity with two
   readings**, never as two agreeing measurements;
3. the tautology: the cited and fitted partitions intersect at exactly one total **by
   construction**, so their agreement is pinned as *not evidence*;
4. both principled sizings fail (19.409 rations; 3.0 hard-errors on re-sow);
5. the structural one: a seeded slow pool is **strictly non-increasing at every step** and
   never refills;
6. the window, **all three points**: 6.5 clears both gates on both perennial scenarios;
   `consumer` hard-errors at 6.0 while its frozen self carries stem-only cleanly; and
   `perennial` fails the 0.05 floor at 7.0. The passing value is pinned alongside its
   bounds — pinning only the two failures would leave the width claim as prose;
7. the (B) identity holds exactly under fractionation **with the seed removed**, and the
   committed-seed C:N is pinned as a **scenario fact, not a model fact**;
8. `open_season` builds no litter pools — the structural-untouched claim, asserted;
9. finding 2's roster, parametrized: the form alone closes with a **strictly improved**
   CO₂ tail on every sealed row, **and** the one-pool form rations at the same seed —
   which is what attributes the headroom to the partition rather than to more carbon.
   Pinned because it is the result recorded as a *price*, and an unpinned price is one a
   future reader can re-derive as a reason to build.

**Teeth verified by MUTATION, not by a green bar**: flipping `EQ_DPM_STANDING_FRACTION`
from Hoosfield's *standing* partition to RothC's *input* ratio (0.59 — the plausible
wrong reading, since it is the number §1.3 prints in bold) takes **12 of the 24** red.

---

# THE RE-OPENING (2026-08-10)

The humification split landed the same day this was written, in the commit immediately
after it, and it discharged **finding 3 — the structural blocker that was this
document's stated reason for the refusal.** The split's own row records that, and records
that finding 4's window evidence is superseded too (*"at seed 7.0 both gates now pass, so
a window narrower than 1.0 mol C no longer holds and must be re-derived before being
quoted"*). So the refusal above rests on two legs, and both moved.

## What is actually left to build, and it is NOT what the findings above priced

RothC's structure is **DPM/RPM** (the two pools that take fresh plant input) feeding
**BIO/HUM** (the two that are *formed* by humification), plus inert IOM. The tree now
has the formed half — `microbial_carbon` (CENTURY active SOM) and `humus_carbon`
(slow SOM), with the humification flux that fills them. **What fractionation still adds
is only the input half: splitting `litter_carbon` into DPM (10.0 /yr) and RPM
(0.3 /yr), fed at RothC's cited 1.44 input ratio.**

⇒ **The value proposition is different from the one this document turned down**, and
saying so plainly is the point of re-opening rather than re-reading:

* the old headline was **inventory** — 6.47× against a 94× census gap, with the
  remaining 14.5× structurally unreachable. That framing is dead: the unreachable part
  is now reachable, so "6.47× of 94×" is not the live number;
* the live headline is **the last uncited decomposer carbon rate.** `decomposition_rate`
  = 0.011 /day = 4.015 /yr is calibrated to Olson's *fastest measured ecosystem* and is
  there **because closure requires the fast edge**, not because the science put it there
  — its own param file says so in the value's `source:` string, and the humification
  split's "WHAT THIS DOES NOT CLAIM" says it again. Fractionation would **replace it with
  two cited rates and a cited input ratio**, discharging it by *changing the form* rather
  than by finding a citation — the move that retired `n_senescence_rate` in option (A)
  and `mineralization_rate` in option (B).

That reframing is also what kills this document's own dismissal of it (*"the one
genuinely attractive by-product here, and is not on its own worth the cascade"*): it was
written when the by-product sat next to a structural ceiling. With the ceiling gone, the
by-product is the whole case.

## THE PREDICTION, on the record before the first probe runs

1. **Closure holds, and the CO₂ trough improves.** The decomposer calibration measured
   that the chambers close *only* at the fast edge, and RPM's 0.3 /yr is almost exactly
   the Zhang median (0.30 /yr) that crashed re-sow — so the naive read is that
   fractionation starves the loop. I predict it does **not**, because 59 % of every
   fresh input decays at **10.0 /yr — 2.5× faster than the calibrated bulk rate** — so
   the immediate return goes *up* and only the 41 % remainder makes the tail.
2. **The plant shrinks again**, the way it did under the split, and the trough and the
   plant size must therefore travel together in every table.
3. **The constant-inventory sizing (3.0) may now survive** where it hard-errored before,
   because the formed pools now return carbon instead of being a dead end.
4. **Finding 5's N identity survives, and `litter_n0` is still owed** — more so, since
   96.7 % of the N-free seed lands in RPM at 0.3 /yr and lingers.

⚠⚠ **PREDICTION 1 WAS WRITTEN FLAT AND COVERS ONLY ONE OF THE TWO REGIMES — caught on
review, before the first probe, and it changes what gets measured.** "59 % of every fresh
input decays at 10.0 /yr" is a claim about *fresh input*, and only `sealed_chamber` /
`water_biting` are fed that way. `perennial` / `consumer` and both long-horizons are
**reset-driven**: the annual dump is the dominant input, so the governing question is not
how fast fresh litter decays but **what the standing pool looks like a year after a
dump** — and there the comparison inverts. A year on, fractionation's DPM share is gone
(10.0 /yr ⇒ ~5-week half-life) and what remains is **41 % of the dump at 0.3 /yr**,
against the frozen bulk pool's `e^(−4.015)` ≈ **1.8 %**. Whether that is more or less
carbon *in the atmosphere at the trough* depends on where in the year the trough falls —
and (C)'s stem-only branch measured `perennial` firing at step 502, **day 197,
mid-drain**. This is the shedding-fed/reset-driven split that correction 2 and (B)'s
finding 5 already logged twice; writing a single prediction over both is its next
instance, mine, and the tell was again a phrase doing unearned work ("every fresh
input").

⇒ **The discriminator, run first because it is cheapest and decides whether there is a
build at all:** both principled sizings on `perennial` under **Euler at 15 years**,
reporting the **per-year CO₂ minimum series** rather than the run minimum — the 0.05
decade floor is the gate that survived stem-only's rescue, and it is the one that kills
things here.

⚠ **Finding 4's window is deliberately NOT re-derived.** The split superseded its
evidence, and the instinct is to re-sweep for the new window — but that sweep is the
refused shape itself, and the refusal on that leg (*a window located by sweeping until
the gate went green, with no independent invariant to size it on*) never depended on the
CUE. Both principled sizings get measured; if both fail, that is the answer, not a prompt
to sweep.

⚠ **Held loosely, for a stated reason: my last closure prediction was wrong, and so was
the repo-wide law behind it.** The humification split's row records it — *"any change
parking carbon in a standing pool is paid out of the CO₂ trough"* was measured false; it
was true of a soil with one fast pool, not of soils. Prediction 1 is the same shape of
reasoning about the same jar.

---

## THE VERDICT: STILL REFUSED, on a leg that is now measured rather than structural

**Both principled sizings still fail on `perennial`, and the failure survives past the
transient.** Probes `M:/claud_projects/temp/soil_frac2/`, each scenario driven the way
its own golden drives it, Euler at `dt = 1`.

| sizing | seed | `perennial` `rationed` | settled CO₂ min | verdict |
|---|---|---|---|---|
| CONTROL frozen (1 pool) | 3.000 | **0** | **0.073291** | passes |
| 1 — constant initial flux | 19.409 | **1** @ step 807 | **0.031741** | fails both gates |
| 2 — constant inventory | 3.000 | — | — | `annual_reset` **hard-errors** (re-sow starves) |

The sizings' *arithmetic* is unchanged by the split (both are properties of the litter
pool's own decay flux and of RothC's standing partition; neither reads a humification
fraction — and holding the decay flux fixed also holds the t=0 CO₂ return fixed, since
`litter_respired_fraction` applies to frozen and fractionated alike). What changed is the
tree they run on, so both were re-measured rather than inherited.

⚠ **The floor failure is NOT a transient, and it was checked rather than assumed.** The
split lengthened the settling transient to ~35 years and anchored its own liveness floor
on a measured equilibrium at ~yr 45 — so fairness required asking the same question here
before calling 0.019267 a failure of the form. Run to **50 years**, sizing 1's per-year
CO₂ minimum rises monotonically and **asymptotes at 0.031741**, still **1.58× below the
0.05 floor** (the frozen control settles at 0.073291). The failure is the attractor, not
the approach to it.

⚠ **Finding 4's window was not re-derived**, and `0.59` was not tried as a seed
partition: the seed is a *standing* pool, so Hoosfield's 3.305 % is the right reading and
0.59 is the input ratio this document's own mutation test uses as the plausible wrong
one.

## FINDING A — the two regimes diverge, and only ONE scenario refuses it

| scenario | regime | sizing 1 | sizing 2 |
|---|---|---|---|
| `sealed_chamber` (3 yr) | shedding-fed | `rationed 0`, CO₂ 0.076380 → **0.078065** | `rationed 0`, CO₂ → **0.080342** |
| `water_biting` (1 yr) | shedding-fed | `rationed 0`, CO₂ 0.085006 → **0.085055** | `rationed 0`, CO₂ → **0.101867** |
| `consumer` (5/15 yr) | reset-driven | `rationed 0`, floor **PASS** | **hard error** |
| `perennial` (5/15 yr) | reset-driven | `rationed 1`, floor **FAIL** | **hard error** |

The shedding-fed pair is benign at **both** sizings and its CO₂ tail improves — but ⚠
sizing 2's improvement comes with a **3.5× smaller plant** (peak vegetative carbon 0.5202
against the frozen 1.8445), so those two numbers travel together here exactly as they do
in the humification row. The binding scenario is **`perennial` alone** (with its
long-horizon twin, which reuses the same scenario object).

⚠ **The firing step was measured by horizon truncation, not read off the CO₂ argmin** —
the (C) stem-only branch recorded that inference as *circular*, since entering the firing
step the pool is already in free fall and the trough is the value the backstop clamped
to. Measured: **step 807 = year 3, day 197.** Stem-only fired at step 502 = year 1,
**day 197** — the identical within-season day, from an unrelated change. That the
seasonal draw peaks at one point and the backstop bites there is a property of the
chamber, not of either mechanism.

## FINDING B — the mechanism, and MY HYPOTHESIS WAS REFUTED BY THE PROBE

The census at each run's own trough showed a *deeper* trough beside a *bigger* plant and
5.7× the inventory. Two mechanisms fit that, and they are different claims: the slow pool
returns carbon too slowly, or the plant's demand grew faster than the supply. I predicted
the first — RPM at 0.3 /yr is almost exactly the Zhang median the decomposer calibration
measured as starving the loop. **Measured, it is false.**

| quantity, at each run's own trough | frozen | fractionated, sizing 1 | ratio |
|---|---|---|---|
| litter return flux | 2.8558 mol C/yr | **8.1112 mol C/yr** | **2.84×** |
| return per unit standing tissue | 1.4765 /yr | **2.3128 /yr** | 1.57× |
| standing tissue | 1.934197 mol C | 3.507145 mol C | 1.81× |
| system carbon inventory | 3.517000 mol C | 19.926256 mol C | 5.67× |
| **the atmosphere they transact through** | **0.055175** | **0.019267** | **0.35×** |
| plant's share of all carbon | 55.0 % | 17.6 % | |
| air's share of all carbon | 1.6 % | **0.1 %** | |

**The loop is not starved — it is running 2.84× faster on both sides of a buffer that
did not grow at all.** The seed grew 6.47×, the return flux 2.84×, the plant 1.81×, and
the atmosphere those three transact through grew **1.00×**: `chamber_air_mol` and the
initial CO₂ are untouched by a litter change. The trough is a *flow-balance* moment in
the season, not a supply shortage, and at 0.1 % of the inventory the atmosphere records
any instantaneous mismatch in full.

⇒ **This is the chamber-scale diagnosis reached independently for the fifth time**, and
in its own words: the atmosphere is a buffer of *hours*, so a change that enlarges the
plant and the soil while leaving the jar alone is paid for in the jar. Recording the
census alone and calling it "the slow pool starved the loop" would have been asserting a
mechanism I had not measured — the humification split's finding 6 shape, one option on.

## What this changes about the case for building it

**The re-opening's own headline does not survive.** The live case was retiring
`decomposition_rate` — the last uncited decomposer carbon rate, sitting at Olson's fast
edge *because closure requires it* — by replacing it with RothC's two cited rates and
cited input ratio. That is precisely what cannot be done: **at every principled sizing,
adopting the cited rates breaks the chamber that the fast edge exists to keep alive.**
The 2026-07-21 calibration's finding is confirmed from a new direction — it measured that
central literature values starve the loop; this measures that *splitting* the pool at
cited rates does not evade that, because the binding constraint was never the pool's
aggregate rate.

⇒ **`decomposition_rate` is now measured UN-RETIRABLE by the only cited alternative on
the shelf**, and that is the durable statement rather than "the seam is refused". RothC's
DPM/RPM pair *is* the two-pool option — it is the lineage the param file's own counter-
reading names, and the one this project refused to reach by relabelling — and it breaks
the binding scenario at every principled sizing. So the fast edge is not a placeholder
awaiting the right citation; it is where the chamber puts the value, and a future reader
reaching for RothC to discharge that TODO can read this instead of re-deriving it.

⇒ the seam stays **refused**, and the refusal is now **better grounded than the one it
replaces**: the old one rested on a structural ceiling (finding 3) that the humification
split has since removed, plus a fitted-window objection. This one rests on a measured
equilibrium beyond the transient, on the one scenario that binds, with the mechanism
established rather than inferred. ⚠ **And it is a fact about the chamber, not about
RothC** — the shedding-fed chambers take the same change without complaint.

⚠ **THE NITROGEN SIDE WAS NOT RE-MEASURED, and that is recorded rather than left to be
inferred from the section's silence** (the (C) diagnosis's `sealed_station` precedent).
Finding 5 — the N-free seed becoming *permanent* because 96.7 % of it parks in RPM at
0.3 /yr, so the seam **owes `litter_n0`** — rests on a mechanism the humification split
does not touch, and nothing in the re-opening's pins measures nitrogen. The harness's own
(B)-identity self-check *was* re-run and holds exactly (litter pool C:N constant to the
last digit across all 916 steps with the seed removed, one-pool and fractionated alike),
which is what licenses the carbon numbers above — but that is a check on the harness, not
a re-measurement of finding 5. Whoever takes this up re-measures the N side; it is an
**unmeasured** leg, not a clean one.

## The pins the re-opening added (section 10 of the test file)

Five, all `slow`, taking the file to **26 functions / 29 collected / 15 slow**:

1. **the floor failure is the ATTRACTOR** — 50 years, sizing 1 asymptotes at 0.031741
   against the 0.05 floor, with the frozen control asserted alongside at 0.073291,
   because "the subject converges below the floor" is a verdict only if the control
   converges above it on the same horizon and harness;
2. **the mechanism, as a refutation** — the return flux is 2.84× *higher*, and the
   `chamber_air_mol == 1000.0` invariance is asserted, because without it the ratio table
   is three numbers with nothing to compare them to;
3. **the regime split** — both shedding-fed chambers close at both sizings, with the
   plant-size cost asserted *in the same test* as the improved tail;
4. **the firing step by truncation**, and the day-197 coincidence with stem-only;
5. **`consumer` is not what refuses it** — pinned because *"fractionation breaks the
   reset-driven chambers"* is the paraphrase this result will collapse into, and it is
   false.

The sizings' immediate results were already pinned (sections 4 and 6) and the
humification split had re-measured both in place, so they are not duplicated here.

⚠ **The horizon is a live constraint, not a footnote.** The split lengthened the
chamber's settling transient from ~3 years to **~35**, past the frozen 15-year horizon.
RPM's half-life is 2.3 years, so fractionation will lengthen it further. **Every number
measured at 15 years is measured inside a transient** and must be labelled as such —
four committed guards had to be restated for exactly this reason.
