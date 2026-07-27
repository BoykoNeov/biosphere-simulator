# Post-roadmap: the canopy regulator

**Status: DIAGNOSED 2026-07-27. The science the (C) diagnosis called missing is NOT
missing — it is on page 101 of a book already in `sources/` and already cited by the
file that says it is missing. It was retrieved, verified first-hand and measured
end-to-end. It fixes the canopy and it does NOT unblock (C), because the two live on
disjoint scenarios. Nothing built, nothing unfrozen, no golden moved.**

The named successor of `docs/plans/post-roadmap-nitrogen-cycle-form.md`, "THE (C)
DIAGNOSIS", which closed with:

> The natural successor is therefore **not** (C) or (D) but the canopy regulator
> (leaf-age or self-shading-driven death), which is what would let the primary's form be
> adopted without a fitted table.

That sentence has two halves. The first is confirmed — the regulator exists, is
wheat-specific, is sourced, and takes the canopy from unphysical to physical. **The
second is measured false.** Adopting it does not let the primary's form be adopted,
because the obstacle that actually blocks (C) is in scenarios the regulator provably
cannot reach.

Read-only probes: `M:/claud_projects/temp/canopy_reg/`. Pins: `tests/test_senescence_form.py`.

---

## FINDING 1 — the science was not missing; our *index* of it was

[A] Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M. & Bakema, A. (1989),
*Simulation of Ecophysiological Processes of Growth in Several Annual Crops*, PUDOC —
**p. 101**, verified on the rendered page image:

> "The rate of leaf area loss is computed in direct relation to the rate of leaf weight
> loss, assuming that the average value of the specific leaf weight applies (Listing 3
> Line 89, Listing 4 Line 117). **Van Keulen & Seligman (1987) calculated the rate of
> leaf area loss in wheat independently of leaf weight loss. They put it at 5 % d⁻¹ once
> the leaf area exceeds the value of 6 m² m⁻² to account for mutual shading.** The
> constant life span of cassava leaves (Subsection 3.2.6) is also reported to be due to
> young leaves that are produced at a constant rate shading old leaves (Cock et al.,
> 1979)."

Three properties of that sentence, and one cost:

* **The rate is FLAT above the threshold** — "once the leaf area exceeds", not
  proportional to the excess. The SUCROS/WOFOST `(LAI − LAI_crit)/LAI_crit` shape is a
  different lineage and is deliberately **not** imported; the reading is settled by the
  page, never by which form lands a chamber in closure.
* **It is WHEAT-specific.** Listing 5 — the table the whole (C) diagnosis runs on — is
  rice IR36 under a 102–135 day season, and C-finding 7 had to price that crop transfer
  explicitly. This one needs no transfer.
* **It carries a threshold AND a rate**, i.e. it is a complete regulator, not a form
  awaiting a value.
* ⚠ **It is [A] QUOTING Van Keulen & Seligman 1987** — a secondary-quoting-primary
  chain, the shape scope (C)'s citation rounds opened three times (FAO→Olson sound,
  Schomberg→S&S sound, Dunn fabricated). It is recorded as **first-hand [A], not
  first-hand V-K&S**. Van Keulen & Seligman 1987 is *not* on the shelf, so the
  transmission leg is unverified and the locus leg — the failure mode that killed Dunn —
  is unchecked. That is a live residual, not a formality.

**Why five citation rounds and a full (C) diagnosis walked past it.** §3.2.6
("Senescence and death", p. 95) is the section our senescence params cite, and it is
where every search went. This sentence is six pages later in the **leaf area** section,
because V-K&S's rule is expressed as a rate of *area* loss rather than a death rate.
The extracted text contains the string; nobody grepped for the *concept* under a
different section heading.

⚠ **The meta-finding takes another instance, and it is the freshest possible one:** the
(C) diagnosis's own headline was that a *locus* error survived inside a correctly
attributed quote — the wrong `LLVT` table, right book. This is the same failure one
level up: right book, right topic, **wrong section**, and the conclusion drawn was not
"we did not find it" but "**the tree does not have it and the science is missing**". A
retrieval ceiling is a fact about one afternoon's search — round 4's rule — and it
expires the same way here: **within one day**.

## FINDING 2 — the area↔weight identity licenses the transfer, and [A] states it itself

V-K&S's rule is a rate of **leaf area** loss, stated *"independently of leaf weight
loss"*. Our tree has no leaf-area state — P2, "LAI is derived, not stored". So the
transfer needs a licensing step, and it has one:

* `specific_leaf_area = 22.0 m²/kg` is a **single constant**, folded once at the config
  boundary into `sla_per_mol_c`, with **no DVS keying anywhere** (verified by sweep; the
  sole consumer is `carbon_budget.py:227`).
* Therefore `LAI = leaf_C · sla_per_mol_c / ground_area` is **linear in leaf carbon**, so
  `d(LAI)/dt = (sla/A) · d(leaf_C)/dt` and a **relative area-loss rate IS a relative
  carbon-loss rate, exactly**.
* And that is **[A]'s own default**, stated in the sentence immediately before the quote:
  area loss "computed in direct relation to the rate of leaf weight loss, assuming that
  the average value of the specific leaf weight applies".

⚠ **The limitation, stated rather than buried:** V-K&S separated the two *because*
specific leaf weight is not constant across leaf cohorts — and **Figure 40 is on the
same page**, plotting it from ~230 to ~530 kg ha⁻¹ over the season (maize, Sibma 1987).
We would be inheriting their rule under an assumption they explicitly declined to make.
That is a named limitation of our state model, not a defect in the transfer — but it
must not be soft-pedalled into "the rule transfers cleanly".

## FINDING 3 — Q1: YES, the regulator fixes the canopy

`open_season`, Euler, the DS-keyed Listing 5 form with and without the regulator:

| | peak LAI | peak W (t/ha) | leaf loss off peak |
|---|---|---|---|
| frozen flat form | 5.191 | 12.633 | 38.5 % |
| Listing 5 (the (C) form) | **16.397** | 18.678 | 30.0 % |
| Listing 5 + V-K&S regulator | **6.244** | 15.725 | 27.6 % |

C-finding 5's unphysical canopy — 16.40 against real wheat's ~5–8 — lands at **6.244**,
inside the band. The regulator does exactly the job the (C) diagnosis said was needed,
using a sourced threshold and a sourced rate, with no fitting.

*(The leaf-loss column is INDICATIVE only and no nearness metric is drawn from it: our
run ends at the weather fixture's end rather than at "harvest time", the loss is
measured off peak LAI, and [A]'s 40–60 % sentence describes rice. Ranking misses by
distance to a midpoint is the fitted comparison this work exists to refuse.)*

## FINDING 4 — ⚠ THE STRUCTURAL ONE: the regulator and the closure blocker live on DISJOINT scenarios

**The regulator fires in exactly one of eight scenarios.**

| scenario | peak LAI (frozen) | headroom to the LAI-6 threshold | regulator |
|---|---|---|---|
| `open_season` [manifest] | 5.191 | **0.809 (16 %)** | reachable |
| `water_biting` | 0.488 | 5.512 | ~12× below |
| `sealed_chamber` [manifest] | 0.508 | 5.492 | ~12× below |
| `perennial_chamber` / `_long_horizon` [manifest] | 0.586 | 5.414 | ~10× below |
| `consumer_chamber` / `_long_horizon` [manifest] | 0.632 | 5.368 | ~9× below |
| `n_limited` | 0.068 | 5.932 | ~88× below |

Added to the **frozen** tree the regulator is **bit-identically inert in all eight** —
verified at `to_bits()` precision over every stock at every step, not "the same to three
decimals".

And the blocker (C) actually died on was `perennial`'s closure. With the regulator:

* **RK4 `perennial` still hard-errors**, `ArbitrationError`, `scale_f =
  0.9527733243688737` — **the identical sixteen digits** with and without the regulator,
  because it never fires on the failing trajectory (peak LAI 0.556 under RK4).
* **The decade CO₂ guard still fails by ~an order.** Per-year minima computed the way
  `test_decade_min_carbon_pool_stationary` computes them (`_TRANSIENT = 2`), with the
  frozen column validated against the committed test's own comment first (finding 10's
  rule): frozen `[0.07402, 0.03873, 0.05421, 0.05481, 0.05484, …]` → past-transient min
  **0.054208**, PASS. Listing 5 **and** Listing 5 + regulator give the *same*
  `[0.00848, 0.01853, 0.01092, 0.01020, 0.01396, 0.00591, …]` → **0.005910**, FAIL.

**Why, and it is not bad luck.** The chambers are **carbon-limited by design** — the (A)
diagnosis already measured their plant at 52 g DM/m² with an implied peak LAI of 0.51.
A mutual-shading regulator is a *canopy-closure* mechanism; a canopy that never closes
cannot be regulated by one. So:

> **The canopy regulator is a field-scale mechanism, and (C)'s blocker is a chamber-scale
> one. Solving the first cannot touch the second.**

That is scope (A)'s finding 11 on the other side of the plant — *"making N faithful does
not make the CHAMBER faithful"* — arrived at independently, and it says the same thing:
**the chamber's problem is that its plant is 25× too small per m², not that any one of
our forms is wrong.**

⇒ **(C) stays refused, and exactly one of its three branches is discharged.** The (C)
diagnosis said taking the primary's form as printed either *breaks `perennial`*, **or**
*requires the canopy-regulation science the tree does not have*, **or** *requires a
calibration whose only target is our own goldens*. Precisely:

* **branch 2 is DISCHARGED** — the science exists, is sourced, and works on the canopy;
* **and discharging it did not help**, because branch 1 (`perennial`'s closure) is
  measured *identical* with the regulator in place;
* **branch 3 stays REFUSED** on principle, unchanged.

So the escape route the (C) diagnosis was holding open — "get the canopy science and the
form becomes adoptable" — is now closed **by measurement rather than by absence**, which
is a strictly better place to be blocked: C-finding 5's objection can be struck from the
list, and what remains is `perennial`'s closure with no un-refused way past it.

## FINDING 5 — a new tripwire: `open_season` sits 16 % under the threshold

Every chamber is 9–88× below the LAI-6 threshold, but `open_season` peaks at **5.191 —
86 % of the way there**. So if the regulator is ever adopted, `open_season` is the one
frozen scenario within reach of it, and any calibration growing the open-field canopy
~16 % makes a *sourced, non-fitted* mechanism start firing in a frozen scenario.

Pinned, in the style `test_nitrogen_form.py` established for the 14.4248 t/ha Greenwood
crossing — because a margin that lives only in prose is the "freeze's prose half is
ungated" shape.

## FINDING 6 — ⚠ a committed docstring's causal claim is measured FALSE (the pin itself is sound)

`tests/test_nitrogen_form.py` records, in module docstring item 2 and in
`test_open_season_peaks_below_the_crossing_with_the_margin_pinned`:

> "Anything that grows the open-field crop ~15 % moves a frozen golden."
> "…any calibration that grows the open-field crop ~15 % **pushes the target below
> `n_critical` and moves a frozen golden**."

`list5+R` is a **counterexample**, and it is +24.5 %, not +15 %:

| | peak W | past the 14.4248 crossing? | min `f_N` | steps < 1 |
|---|---|---|---|---|
| frozen | 12.633 | no | 1.000000 | 0/306 |
| Listing 5 | 18.678 | yes | **0.995213** | **6/306** |
| Listing 5 + regulator | **15.725** | **yes** | **1.000000** | **0/306** |

The **first** conjunct holds (the target at peak is 1.4367 % DM, below `n_critical`'s
1.5000 %). The **second does not follow**: `f_N` reads the plant's *actual* concentration,
not its target, and the plant sits **15–30 % above** its Greenwood target once past the
crossing (measured: `plant_n/(target·W)` ∈ [1.147, 1.301] for Listing 5, [1.153, 1.198]
with the regulator). Demand-deficit uptake clamps at zero deficit, so a plant above
target has no route back down — a property (A) recorded and this measures.

⚠ **Two hypotheses of mine were REFUTED by the probe, and both are recorded because the
refutations are what make the finding trustworthy:**

1. *"The bite is a growth-dilution transient."* **False.** Listing 5's dips occur at
   relative growth rates of 0.06–0.6 %/day, with the concentration hovering at 1.4952 %
   against a 1.5000 % critical. It is a hair-width crossing, not a rate event.
2. *"`conc_fN = target(W) · (W_C/veg_C)`, so the effective threshold is composition-
   dependent."* **False** — worst error 16.5–23.2 %. `plant_n` does not sit on the
   target, so the decomposition has no basis. (The first version of this probe also
   printed an effective threshold of 0.001 t/ha, from a percent-vs-fraction bug in the
   coefficient; the absurd number is what exposed the whole line of reasoning.)

**What is actually true is weaker and cleaner: peak mass does not order the bite at
all.** Listing 5 first crosses `n_critical` at W = **15.068** t/ha; Listing 5 + regulator
reaches a *higher* peak (15.725) and **never** crosses. There is no single W threshold —
the bite is trajectory-dependent.

⇒ **The pin is SOUND and CONSERVATIVE; its docstring's inference is not.** Asserting
`peak_w < 14.4248` fires *before* the earliest measured bite (15.068), which is exactly
what a tripwire should do. It is the causal sentence — "grows the crop ~15 % ⇒ a frozen
golden moves" — that overstates, and this project keeps those two apart deliberately
(round 4's `self_discharge`: *the value may stand* and *its justification is falsified*
are both true, and the first does not rescue the second). Corrected in place; the
assertion is untouched.

Related, and worth having: on the frozen tree the **concentration** margin is much
larger than the **mass** margin — `open_season`'s actual concentration bottoms at
**1.9305 % DM against a 1.5000 % critical (29 % clear)**, where the mass margin is 12 %.
The mass pin is the tighter of the two, which is the right way round for a guard.

## What would have to be true to BUILD this

Not recommended now — it is a carbon change with nothing to buy at present, since it is
bit-identically inert on the frozen tree and the one form it helps is refused. Recorded
so the price is not re-derived:

* **Cost:** a carbon change ⇒ every golden, both manifests, `biosphere_params.txt`, the
  Rust mirror, the crossport tier. Python-canonical under the pivot's rule 3.
* **Benefit on the frozen tree: exactly zero, bit-for-bit.** Adopting it alone would move
  nothing and cost a full cascade — its only value is as a *precondition* for a form the
  tree does not run.
* **It does not stand alone either.** Its value is conditional on (C), and (C) is blocked
  on `perennial`'s closure, which the regulator does not touch.
* **Retrieval residual:** Van Keulen & Seligman 1987 is not on the shelf. Before the
  values entered a param file, the transmission and locus legs should be checked against
  the primary — Dunn 2011 is the standing reason.
* **Two params, both sourced**, if it ever ships: a critical LAI (6 m² m⁻²) and a
  shading death rate (0.05/day), with the area↔weight identity of finding 2 recorded as
  the licensing step and Figure 40 recorded as the limitation.

## The honest one-line summary

**The canopy regulator the (C) diagnosis called a missing science was already on the
shelf, is wheat-specific, needs no fitting, and takes the unphysical canopy from LAI 16.4
to 6.2 — and it changes nothing that matters, because the seven scenarios that block (C)
never grow a canopy big enough to shade itself.**
