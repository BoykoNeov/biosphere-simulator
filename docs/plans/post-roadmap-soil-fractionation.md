# Post-roadmap: soil carbon pool fractionation — DIAGNOSED, NOT BUILT

**DIAGNOSED 2026-08-10. Read-only. No value, golden, param or manifest moved;
`git diff src/` empty; nothing unfrozen.** Probes in `M:/claud_projects/temp/soil_frac/`;
pins in `tests/test_soil_fractionation.py`.

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

`tests/test_soil_fractionation.py` — **21 test functions, 24 collected tests** (the
roster pin is parametrized ×4), of which **10 are `slow`**. Read-only, no fixture, no
unfreeze. ⚠ Both numbers are given because they are different quantities: the first draft
of this line said "24 test **functions**", a count lifted from pytest's output and
relabelled — this repo's most-repeated shape, caught here before it shipped. The claims
they carry:

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
