## **The canopy regulator** (the (C) diagnosis's named successor)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED 2026-07-27, NOT BUILT — and the headline is that "blocked on a MISSING science"
was false the day it was written.** `docs/plans/post-roadmap-canopy-regulator.md`; probes
`M:/temp/canopy_reg/`; 7 new pins in `tests/test_senescence_form.py` (25 pass). **FINDING 1
— the science was on our own shelf, in the book the file already cites.** [A] Penning de
Vries 1989 **p. 101**, verified on the page image: *"Van Keulen & Seligman (1987) calculated
the rate of leaf area loss in wheat independently of leaf weight loss. They put it at **5 %
d⁻¹ once the leaf area exceeds the value of 6 m² m⁻²** to account for mutual shading."*
**FLAT** above the threshold ("once … exceeds"), not proportional to the excess — the
SUCROS/WOFOST `(LAI−LAIcrit)/LAIcrit` shape is a different lineage and was deliberately
**not** imported; **WHEAT**-specific, so unlike Listing 5's rice IR36 it needs **no crop
transfer**; threshold *and* rate, i.e. a complete regulator. ⚠ It is **[A] QUOTING V-K&S
1987**, which is **not** on the shelf ⇒ first-hand [A], **not** first-hand V-K&S; the
transmission and locus legs are unverified (Dunn 2011 is the standing reason to keep that
distinction). **Why five citation rounds and the whole (C) diagnosis walked past it: every
search went to §3.2.6 (p. 95, "Senescence and death"), and V-K&S's rule is filed six pages
later under LEAF AREA, because it is expressed as an area rate.** ⚠ **That is the (C) locus
finding ONE LEVEL UP — right book, right topic, WRONG SECTION — and the conclusion drawn was
not "we did not find it" but "the science is MISSING".** A retrieval ceiling is a fact about
one afternoon's *search*, never a property of the literature: this one **expired in one
day**. **FINDING 2 — the licensing step, and [A] states it itself.** V-K&S give an **area**
rate "independently of leaf weight loss" and our tree has no leaf-area state (P2: "LAI is
derived, not stored"). But `specific_leaf_area = 22.0 m²/kg` is a **single constant with no
DVS keying anywhere** (swept; sole consumer `carbon_budget.py:227`), so LAI is **linear** in
leaf carbon ⇒ `d(LAI)/dt = (sla/A)·d(leaf_C)/dt` ⇒ a relative *area* rate **IS** a relative
*carbon* rate — which is **[A]'s own default, stated one sentence earlier** ("computed in
direct relation to the rate of leaf weight loss, assuming that the average value of the
specific leaf weight applies"). ⚠ Limitation pinned, not buried: V-K&S separated the two
**because** specific leaf weight varies by cohort — **[A]'s Figure 40 is on the same page**,
~230→530 kg/ha over a season — so we would inherit their rule under an assumption they
explicitly declined to make. (The linearity pin is a few-ULP tolerance, not bit-exact:
`(x·sla)/A` and `x·((1·sla)/A)` associate differently. The identity is exact; its float
evaluation is not — stated rather than quietly loosened.) **FINDING 3 — Q1 answered YES: it
fixes the canopy.** `open_season` peak LAI **16.397 → 6.244**, inside real wheat's ~5–8,
**with nothing fitted** (both numbers off p. 101); peak W 18.678 → 15.725. ⚠ **FINDING 4 —
THE STRUCTURAL ONE, and it is why (C) STAYS REFUSED: the regulator and (C)'s blocker live on
DISJOINT SCENARIOS.** It fires in **exactly 1 of 8** scenarios. Every chamber peaks at LAI
**0.068–0.632** against a threshold of 6 (**9–88× below**), so added to the frozen tree it
is **BIT-IDENTICALLY inert in all eight** — verified at `to_bits()` over every stock at
every step, not "same to 3 decimals". And `perennial`, where (C) actually died, **still
hard-errors under RK4 at `scale_f = 0.9527733243688737` — the IDENTICAL sixteen digits with
and without it** (it never fires on the failing trajectory; RK4 peak LAI 0.556), with the
decade CO₂ minimum unchanged at **0.005910** vs the 0.05 floor (frozen `[0.07402, 0.03873,
0.05421, …]` → 0.054208 PASS, reconstructed the committed test's way and validated against
its own comment first, per finding 10's rule). **Why, and it is not bad luck: the chambers
are CARBON-limited by design** (the (A) diagnosis measured 52 g DM/m²); a mutual-shading
rule regulates canopy **CLOSURE**, and their canopies never close. **The regulator is a
FIELD-scale mechanism and (C)'s blocker is a CHAMBER-scale one.** That is scope (A)'s
finding 11 on the other side of the plant — *"making N faithful does not make the CHAMBER
faithful"* — reached independently. ⇒ **exactly ONE of (C)'s three branches is DISCHARGED,
and discharging it did not help**: branch 2 ("requires the canopy-regulation science the
tree does not have") is struck — the science exists and works on the canopy; branch 1
(`perennial`'s closure) is measured **identical** with the regulator in place; branch 3 (a
calibration targeting only our own goldens) stays **refused** on principle. So the escape
route the (C) diagnosis held open is now closed **by measurement rather than by absence** —
a strictly better place to be blocked, and `perennial`'s CLOSURE is left with no un-refused
way past it. **FINDING 5 — a new tripwire**: the chambers are 9–88× clear but `open_season`
peaks at **5.191 = 86 %** of the threshold, so any calibration growing the open-field canopy
**~16 %** starts a *sourced, non-fitted* mechanism firing in a frozen scenario. Pinned in
the style of the 14.4248 t/ha Greenwood crossing. ⚠ **This margin is on LAI, NOT on
biomass** — the Greenwood crossing is a *mass* margin (~12 %), this is a *leaf-area* one
(~16 %), and conflating the two is the ambiguity that has already bitten this repo twice
(Greenwood's `W` vs `f_N`'s denominator; mass vs concentration in finding 6). ⚠ **FINDING 6
— A COMMITTED DOCSTRING'S CAUSAL CLAIM IS MEASURED FALSE, AND THE PIN IT GUARDS IS SOUND.**
`test_nitrogen_form.py` says *"any calibration that grows the open-field crop ~15 % pushes
the target below `n_critical` **and moves a frozen golden**"*. `list5+R` grows it **+24.5
%** (12.633→15.725), **does** cross (target 1.4367 % vs critical 1.5000 %), and leaves
**`f_N` ≡ 1.000000, 0/306 steps** — where the bare (C) form at 18.678 bites (0.995213,
6/306). The second conjunct does not follow: **`f_N` reads the plant's ACTUAL concentration,
not its target**, and demand-deficit uptake **clamps at zero deficit**, so past the crossing
the plant sits **15–30 % ABOVE** target (measured `plant_n/(target·W)` ∈ [1.147, 1.301] /
[1.153, 1.198]) with no route back down. **And peak mass does not even ORDER the bite** —
the bare form first crosses `n_critical` at a **lower** mass (15.068 t/ha) than the
regulated form's peak (15.725), which never crosses; the bite is **trajectory**-dependent. ⇒
**the ASSERTIONS are untouched and are CONSERVATIVE by exactly the right sign** (14.4248
fires *before* the earliest measured bite, which is what a tripwire should do); it is the
causal **sentence** that overstated. *The value may stand* and *its justification is
falsified* are both true and the first does not rescue the second — round 4's
`self_discharge` discipline, applied to our own test prose. ⚠ **TWO OF MY OWN HYPOTHESES
WERE REFUTED BY THE PROBE AND ARE RECORDED BECAUSE THE REFUTATIONS ARE WHAT MAKE THE FINDING
TRUSTWORTHY**: (i) *"the bite is a growth-dilution transient"* — **false**, the dips sit at
RGR 0.06–0.6 %/day with concentration hovering at 1.4952 % against 1.5000 %, a hair-width
crossing; (ii) *"`conc_fN = target(W)·(W_C/veg_C)`, so the threshold is
composition-dependent"* — **false**, worst error 16.5–23.2 %, because `plant_n` does not sit
on the target at all. (The first run of that probe printed an "effective threshold" of
**0.001 t/ha** from a percent-vs-fraction bug in Greenwood's coefficient — **the absurd
number is what exposed the whole line of reasoning**, the (B)-probe lesson that the tell is
a crash signature, not a clean control.) Related: on the frozen tree the **concentration**
margin (29 % clear of critical) is much wider than the **mass** margin (12 %), so the mass
pin is the tighter guard — the right way round. **NOT BUILT, and the price is recorded so it
is not re-derived**: a carbon change ⇒ every golden, both manifests, `biosphere_params.txt`,
the Rust mirror, crossport — for a **bit-for-bit zero** benefit on the frozen tree,
conditional on a form that is refused. **Cascade actually shipped: exactly 1 manifest hash,
biosphere-only** (`senescence.yaml`, comment-only — an honor-system provenance unfreeze;
`grep -l` confirmed no other manifest names it, round 5's fix, third application). No
value/golden/code moved; `git diff src/simcore/` empty; Rust untouched.
