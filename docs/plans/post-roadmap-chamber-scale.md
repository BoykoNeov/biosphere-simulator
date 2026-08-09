# Post-roadmap: the chamber-scale diagnosis

**DIAGNOSED 2026-08-09. Read-only. No value, golden, param or manifest moved;
`git diff src/` empty; nothing unfrozen.** Probes in `M:/claud_projects/temp/chamber/`;
pins in `tests/test_chamber_scale.py`.

The upstream blocker with **three independent witnesses**, taken as its own item:

> * scope (A) finding 11 — *"making N faithful does not make the CHAMBER faithful"*; a
>   52 g DM/m² carbon-limited plant against field-sized N params is the obstacle.
> * canopy-regulator finding 4 — the regulator is **bit-identically inert** in all eight
>   scenarios because *"the chambers are CARBON-limited by design"* and their canopies
>   never close. *"A field-scale mechanism cannot solve a chamber-scale blocker."*
> * (C) finding 8 (stem-only) — *"a sealed chamber's carbon inventory is fixed, so any
>   change that parks carbon in a standing pool is paid for out of the CO₂ trough."*

Three refusals, reached independently, all bottoming out on one number. This document
measures that number and asks whether it is a defensible engineering spec or an artefact.

Per the advisor's framing at the outset: **whether the chamber is under-sized is a
retrieval question, not a modelling one** — closed-system programmes publish chamber
volume per unit growing area. It is, and the retrieval settles it.

---

## THE HEADLINE — the obvious fix is not merely unattractive, it is refuted

**You cannot fix this by making the chamber bigger.** Not "we chose not to"; the
engineering reference rules it out by three and a half orders of magnitude.

[BVAD] Table 4-88 (p. 170), *Plant Growth Chamber Equivalent System Mass per Growing
Area*, from Drysdale 1999b — a flight-projected biomass production chamber:

| Component | Volume [m³/m²] |
|---|---|
| Shoot zone | 0.67 |
| Root zone, water and nutrients | 0.11 |
| Lamps | 0.25 |
| **Total** | **1.03** |

Ours: `chamber_air_mol = 1000` mol over `ground_area = 1.0` m² = **24.06 m³/m²** at
20 °C (22.41 at STP).

* The chamber is **not air-starved. It is ~23× MORE generous than the flight design**
  (~36× on the shoot-zone line, which is the free air above the canopy).
* To hold **one field crop's standing carbon** (56.03 mol C/m², measured below) in that
  air at its 357 ppm needs **157× the air = 3,775 m³/m² = 3,665× BVAD's design volume**.
* Elevating CO₂ does not rescue it. [BVAD] gives the plant optimum as *"the optimum
  partial pressure of carbon dioxide for plant growth is roughly 0.10 to 0.20 kPa
  (Wheeler, et al., 1993)"* — 987–1,974 ppm, i.e. 2.8–5.5× ours. At the **top** of that
  cited band the air still needs **28× enlargement = 663× BVAD**.
  *(Locus, checked rather than assumed: that sentence appears **twice**, printed pp. 130
  and 175 — identical wording both times, so the duplication is benign. Recorded anyway,
  because the (C) diagnosis's finding 1 was a locus error inside a correctly-attributed
  quote.)*

⇒ **At any defensible chamber spec the atmosphere is a buffer of HOURS, not a
reservoir.** This is what makes "resize the chamber" a dead branch rather than a
tempting one — and it means the co-adaptation trap never has to be argued. We do not
have to say resizing to rescue a refused science *would* be backfitting (it would); the
resize is independently impossible.

**The one legitimate atmospheric knob is small and cited:** raising the chamber toward
the plant-optimum p(CO₂) is worth **2.8–5.5×**, not 157×. Recorded as available, not
taken — it moves every frozen golden for a fraction of the needed factor.

---

## THE CENSUS — what the inventory is, and what the plant is holding

`probe1_inventory.py`, each scenario driven the way its own golden drives it
(`run_season` vs `run_perennial` — the (B)-diagnosis lesson). Carbon is conserved
exactly in all four, so t=0 partition == the whole inventory forever.

| scenario | inventory [mol C] | partition at t=0 | peak plant [mol C] | **as % of inventory** | peak DM [g/m²] | peak LAI |
|---|---|---|---|---|---|---|
| `sealed_chamber` | 3.517 | litter 3.0 / air 0.357 / seedling 0.16 | 1.961 | **55.8 %** | 52.3 | 0.508 |
| `perennial` | 3.517 | same | 2.248 | **63.9 %** | 60.0 | 0.586 |
| `consumer` | 3.884 | litter 3.0 / air 0.714 / seedling 0.16 / herbivore 0.01 | 2.602 | **67.0 %** | 69.5 | 0.632 |
| `open_season` | *unbounded source* | seedling 0.16 | 56.027 | — | 1495.4 | 5.191 |

The whole sealed inventory is **42.2 g C/m²**, of which **85 % is the seeded litter pile
and 10 % is the air**. At peak the plant is holding **56–67 % of every carbon atom in
the system**.

**In units of demand** (`probe2_arithmetic.py`), against [BVAD] Table 4-91's wheat row —
nominal CO₂ uptake **77.00 g CO₂/m²·d = 1.7496 mol C/m²·d**:

* the **entire chamber inventory** is **2.01 days** of one square metre of wheat;
* the **atmosphere alone** is **4.9 hours** of it;
* and against [BVAD] Table 3-31's crew CO₂ load (24.654 mol C/CM-d, already first-hand
  in `docs/bvad-reference.md`), the entire inventory is **3.4 hours of one crewmember's
  exhalation**.

That chamber is then asked to run **3, 5 and 15 years closed**.

⚠ **THE ROSTER IS CHECKED AGAINST THE MANIFEST, NOT AGAINST THE TABLE'S OWN LENGTH** —
this repo has been bitten by that exact shape twice ((B)-finding 4's five rows against
seven frozen scenarios; (A)-finding 9's list checked against its own length), and the
first draft of this census was the third instance: three sealed rows, no `water_biting`,
and no 15-yr row under a sentence that says "3, 5 and **15** years closed". Measured
rather than argued (`probe3_roster.py`), and the answer is that **three rows cover all
four sealed chambers and all six frozen sealed rows BY CONSTRUCTION**:

* the manifest's `perennial_long_horizon` / `consumer_long_horizon` / `drift_summary`
  reuse the **same scenario objects** as the 5-yr rows and differ only in `years`, and
  the inventory is a **t=0 property** ⇒ inventories are **bit-identical**
  (`3.517000` and `3.884000`, same hex), with the peak-plant fraction unmoved at
  63.9 % / 67.0 % and `rationed == 0` at both horizons. The horizon lengthens the run,
  not the jar;
* `water_biting` is `sealed=True, litter_carbon0=3.0` with **every gas default**, i.e.
  the perennial chamber with `soil_water0` moved 1000 → 50 ⇒ its inventory is the
  perennial one **bit-exactly** (`0x1.c22d0e5604189p+1`), and its plant peaks at
  **64.9 %** of it. Not a fourth jar; the same jar under water stress.

So the honest scope of "3.517 / 3.517 / 3.884" is **all four sealed chambers at every
frozen horizon**, which is a stronger statement than the table alone makes — and it is
worth stating precisely *because* it was not obvious from the row count.

⚠ **A retrieval hazard materialised here and the visual channel is what caught it.**
`pdftotext -layout` scrambles Table 4-91's columns and files **Rice's** row
(30.23 / 39.0 / 42 %) under the name **Wheat**. Wheat's true row is **50.00 / 150.0 /
42 % / 56.00 / 77.00 / 11.79**. Read off a 170 dpi page render; Table 4-88 was
re-verified the same way even though its extraction happened to be correct. This is
round 6's rotated-table hazard taking a **second** instance — and the naive read would
have put Rice's numbers into this document under Wheat's name.

**The scale gap, stated once:** the same crop, same params, same weather, with an
unbounded carbon source reaches **1495 g DM/m²** and **LAI 5.19**; inside the chamber it
reaches **52–70 g DM/m²** and **LAI 0.51–0.63**. That is **~24× in mass** and **~9× in
leaf area** — and it is not a modelling defect, it is the inventory.

⚠ **Two denominators, kept apart.** `open_season`'s peak is **14.954 t/ha** including
fibrous roots and **12.633 t/ha** excluding them; the 12.633 quoted throughout
`CLAUDE.md` is Greenwood's root-excluding `W`. Both measured here, reconciled to the
digit. This repo has been bitten twice by exactly this ambiguity (Greenwood's `W` vs
`f_N`'s denominator; mass vs concentration), so neither number is quoted bare.

---

## THE MECHANISM — `stock = flux / k`, and one pool gives you one of them

Why is the soil pile 3.0 mol C when a real soil holds hundreds? Because the tree has
**one** litter pool with **one** rate, and a first-order pool's standing stock is pinned
by its rate: `C* = flux / k`.

[RothC] Coleman & Jenkinson, RothC-26.3 guide (`sources/RothC_guide_WIN.pdf` — already
the decomposer calibration's source), §1.5, first-hand:

| pool | k [1/yr] | vs our litter (4.01/yr) |
|---|---|---|
| DPM Decomposable Plant Material | 10.0 | 0.40× |
| RPM Resistant Plant Material | 0.3 | **13×** |
| BIO Microbial Biomass | 0.66 | 6.1× |
| HUM Humified Organic Matter | 0.02 | **201×** |
| IOM Inert Organic Matter | inert | ∞ |

Our `decomposition_rate` 0.011/day = **4.01/yr** is a *decomposable-plant-material* rate.
A real soil keeps most of its carbon in RPM/HUM/IOM — **13× to 201× slower, and one pool
that never decomposes at all.**

⇒ **With one pool you can match the soil's CO₂ FLUX or its CARBON STOCK, never both.**
That is an identity, not a measurement, and it is the whole diagnosis:

* [RothC]'s Hoosfield worked example: soil organic C **33.8 t C/ha = 281.4 mol C/m²** at
  equilibrium (incl. 2.7 t C/ha IOM), sustained by a plant input of **1.70 t C/ha/yr =
  14.15 mol C/m²·yr**.
* Our seeded litter stock: **3.0 mol C/m² = 0.36 t C/ha** — **94× short**.
* Our litter *flux* `k·C`: **12.04 mol C/m²·yr** — the **same order** as Hoosfield's
  14.15.

⚠ **The flux agreement is n=1 and must not be read as a law.** One worked example, one
soil, one crop, 1852 arable — and our `litter_carbon0 = 3.0` was sized by probe to make
**O₂ depletion dramatic**, not to match a soil respiration rate, while
`decomposition_rate` was recalibrated separately for closure. Two free numbers landing
0.85× on one reference is **one point**, not a finding about the model; writing it as
"our flux is right" would be this project's own meta-finding again (a number fitted to
one scenario at one horizon, written as a law). **What is claimed is the ORDERING**: the
stock is short by ~2 orders while the flux is not short at all. The 94× stands on its
own arithmetic and does not need the coincidence.

**And this is exactly why the obvious soil fix already failed.** The decomposer
calibration measured it (finding 4): litter ×5 trips `perennial` `rationed = 5`; ×10/×20
explode. The reason is now legible — at a fixed `k`, stock and flux are the same knob.
Matching Hoosfield's stock at our `k` would return **1,130 mol C/m²·yr, ~80× real soil
respiration.** The prior measurement was right and its bolded explanation ("the binding
constraint is the chamber's O₂ headroom, **not the litter size**") named the *symptom*;
the cause is that the tree has no slow pool to park carbon in. ⚠ **And it is sharper than
"they missed it": that paragraph writes `flux = k·C_litter` down two sentences before
concluding against it.** Its opening steady-state argument ("annual CO₂ return is
k-independent; k only sets the standing pool size") is *correct* for a pool fed by a
fixed input — but a **seeded** pile is an initial condition draining at `k·C`, so the
identity that governs the experiment is the one it had already written. Annotated at its
own site (`post-roadmap-decomposer-calibration.md`, finding 4), original kept.

---

## SO WHAT IS THE SEALED CHAMBER A MODEL OF?

Stated plainly, because three separate pieces of work have now circled it:

**It is not a bioregenerative life-support analogue, and it cannot be made into one by
sizing.** A BLSS closes carbon through a **crew and its food/waste loop** — the crew is
the reservoir and the driver. A real one balances **~14 m² of wheat per crewmember** at
[BVAD] chamber rates (24.654 ÷ 1.7496). Ours has **1 m² and no crew at all**, and its
whole carbon inventory is 3.4 hours of one person's breathing.

*(No "at our field rate" area figure is quoted: the only annual number we have is peak
**standing** biomass, which is not annual **net** fixation — the crop respires and sheds.
Wrong-shaped denominator; the BVAD figure is clean and sufficient.)*

**What it legitimately is: a closed plant + soil test rig** — an instrument for showing
that a carbon/oxygen/water/nitrogen loop closes exactly, that mortality routes to litter
rather than to a sink, and that an emergent multi-year cycle exists at all. It does all
of that, and does it honestly. That is a legitimate thing to be.

⇒ **The defect is not the chamber's size. It is that the frozen contract uses this rig's
closure gate (`rationed == 0`) as the acceptance test for FIELD-scale plant science.**
Every one of the three witnesses is that same collision: a mechanism sized for a 56 mol
C/m² crop, judged by whether a 3.5 mol C jar survives it. `open_season` — the only
frozen scenario at field scale, and the one that grew *better* under two of the three
refused changes ((C) full form +47.8 %, stem-only +7.96 %; the canopy regulator alone is
inert there) — carries no **carbon-scarcity** gate at all: its CO₂ source is an
unclamped boundary stock, so a carbon rationing assertion is unfalsifiable there.

That is the upstream statement (C), stem-only and the canopy regulator were all
approaching from different sides.

---

## The seam, with a measured obstacle (NOT a recommendation)

**Soil carbon pool fractionation** — DPM/RPM/BIO/HUM/IOM, or any subset with at least
one slow pool — is the shape of what would let a chamber hold a realistic carbon
inventory *without* a proportionally huge CO₂ flux, because `C* = flux/k` decouples once
there is more than one `k`. The science is on the shelf and first-hand (RothC §1.5), and
the tree already refuses the cheap fake: re-labelling our single fast pool as a "labile
fraction" was **refused** by the decomposer calibration, and re-anchoring `microbial_n`
to a homeostatic C:N was **refused** by the (B) diagnosis. Both refusals stand; this is
the *form* change that would make the re-labelling unnecessary.

**Priced, not proposed.** New stocks and flows ⇒ biosphere `flow_set` + `param_files`,
every carbon golden, the station manifest, `biosphere_params.txt`, the Rust mirror, the
crossport tier. And on the frozen tree the benefit is **conditional**: it enables a
bigger inventory, it does not by itself deliver one. The canopy-regulator row is the
precedent for what a build whose frozen-tree benefit is zero costs — this one is not
zero, but it is not demonstrated either. Whoever takes it must name the invariant first
(the increment-1 precedent: the consumer chamber's 2× was legitimate **because** it held
Ci₀ = 250 and x_O₂ = 0.21 invariant — sized on an independent invariant, not on the
goldens).

**Also left standing, unchanged:** the atmospheric route is refuted above; the
crew-coupled route already exists in the tree as `GREENHOUSE_BIO_SCENARIO` (a
cabin-sized 9,500 mol atmosphere at ~400 ppm against a **4,000 mol C** food store —
`crew.food_store` is a CARBON pool, checked, not assumed — and a crew respiring into it)
— **three orders more carbon than the frozen chamber (~1,137×)** — but it runs
7 days with a seedling and is station-side, non-frozen, and outside the biosphere
contract.

---

## Pins

`tests/test_chamber_scale.py` — read-only assertions, no fixture, no unfreeze:

1. the inventory census per scenario (3.517 / 3.517 / 3.884 mol C) and its partition;
2. carbon conservation across the whole run for each (the census's premise);
3. the plant holds > 55 % of the system's carbon at peak;
4. the chamber:field peak ratio (~24× mass, ~9× LAI) — the "carbon-limited by design"
   claim as a number;
5. both `open_season` denominators (14.954 incl. roots / 12.633 excl.), asserted
   together so they can never be conflated again;
6. the BVAD volume arithmetic: ours ≥ 20× the design value, and the field-crop
   requirement ≥ 3,000× it — a guard against a future "just make the chamber bigger";
7. `stock = flux / k` as the identity it is, and the 94× stock shortfall vs Hoosfield —
   with the flux agreement asserted only as an **ordering** (same order of magnitude),
   never as a ratio, so the n=1 coincidence cannot harden into a law;
8. **the roster**: that the three census rows cover all four sealed chambers at every
   frozen horizon — the 15-yr inventories bit-identical to the 5-yr ones, and
   `water_biting` bit-identical to `perennial` with only `soil_water0` differing. Pinned
   so that adding a sealed scenario, or giving a long-horizon golden its own scenario
   object, goes red rather than quietly narrowing this document's scope.
