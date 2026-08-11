## **The nitrogen-cycle FORM gap** (the decomposer calibration's named successor)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**OPTIONS (A) AND (B) BOTH COMPLETE (2026-07-27) — biosphere unfrozen + re-frozen twice; the
FORM changed, and EVERY CARBON TRAJECTORY IS BYTE-IDENTICAL through both. (A) below; (B)'s
build record is at the END of this row.** **(A):** Shipped: N:C-**coupled** shedding
(`min(tissue_conc, n_residual)·shed_C`, driven by `Senescence`'s own flux) **+**
**demand-deficit** uptake (`min(target·biomass − plant_n, capacity·availability)`) against
**Greenwood eqn (6)** — the two are one change, since coupled shedding removes the
`uptake/k_sen` brake. **`n_senescence_rate` is RETIRED** — the project's highest clean-room
risk, a 1/day rate five citation rounds proved no source publishes — and it was discharged
**by changing the FORM, not by finding a citation**: the replacement param is a tissue N
concentration, and `n_residual` was *already cited* (Van Hecke 2020, "N left after N
remobilization"). +3 cited params, −1 uncitable ⇒ **zero uncited params in
`nitrogen.yaml`/`mineralization.yaml`'s shedding path**. Also: `annual_reset` now resets
**N** (was carbon-only, the *windfall* its own docstring flagged), and `plant_n0`
0.5→**2.43e-4 kg** (the old IC was **2055× target** — an artefact of the fixed-flux law, and
it cannot self-correct downward since a plant above target has zero deficit). **RESULTS: 10
goldens moved and every one moved ONLY in its NITROGEN stocks**; every carbon amount
byte-identical, `drift_summary.json` regenerated **byte-unchanged**, `f_N ≡ 1.0000` (0 steps
<1) in all 7 frozen scenarios — finding 9's offline prediction held exactly, which is the
zero-feedback license paying off. **`n_limited` is byte-identical and finding 10's premise
was FALSE**: it is **open-field**, so the sealed-only shedding flow is never built ⇒
`plant_n0` still governs ⇒ min 0.1759/187 unchanged, **no seedling-N knob owed** (the
probe's *re-seeding convenience* had looked like a property of the design). **THE
DELIVERABLE SPLITS, and the split IS the finding**: the **shed material** is straw-like (C:N
= `carbon_fraction/n_residual` = **90**, both cited, vs wheat straw ~80 — the quantity the
change was for), while the **litter pool** is **TWO REGIMES, not one number** (see
correction 2 below): **173–192** in the *shedding-fed* chambers because N mineralizes out
2.7× faster than C decomposes out (a band *below* the quasi-steady law `90 × 2.727 =
245.5`), and **~10** in the *reset-driven* ones, dominated by the annual dump of N-rich
dying tissue. From **0.004** to either is orders better, with the residual now attributable
to *named mechanisms* rather than to the form. ⚠ **CORRECTED 2026-07-27 (advisor catch, then
measurement): the litter POOL C:N figures first recorded here were WRONG, and the error was
the meta-finding's shape again — a number fitted to ONE scenario at ONE horizon, written as
a law.** The original text asserted `pool C:N ≈ (shed C:N)·(k_min/k_decomp)·**1.894**` ≈
465, with 1.894 called a "measured geometry factor". It was fitted to `sealed_chamber`'s
**final** state after 3 years, and the end-of-run value is horizon-dependent across more
than an order of magnitude: **210** (1 yr, `water_biting`), **465** (3 yr,
`sealed_chamber`), **9076** and **11877** (5 yr, `perennial`/`consumer`). My first
explanation for the outliers — seeded `litter_carbon0` with no N counterpart — was **also
false**: all four scenarios seed `litter_carbon0 = 3.0`. **The real mechanism:** litter
input is a *pulse* (the annual dump), not a continuous feed, and between pulses both
currencies drain — carbon with a ~63-day half-life, nitrogen with a ~23-day one — so the
end-of-season snapshot is a **tail**, and by year 5 it is the ratio of two vanishing numbers
(`litter_n` = **1.3e-11 kg**). Quoting that as "the litter C:N" is quoting numerical dust.
**What is actually true, measured at peak `litter_n` across all four sealed scenarios: pool
C:N = 173–192**, a tight band sitting *below* the quasi-steady law `90 × 2.727 = 245.5`
(0.71–0.78× it) because the pulsed pool never converges upward. That is **~2.2× wheat
straw's ~80**, not ~5×. **And the scope-B projection shrinks with it**: applying Stanford &
Smith's 39-soil range to the measured relationship gives **31–83 (pooled mean 47)**, not the
"78–211, mean 119" this row first carried — every cited value lands at or below real
residue, against **~184** for our uncited 0.03/day. **The DIRECTION is unchanged and is the
point** (two independent lines still say `mineralization_rate` is too fast); only the
magnitude was inflated. Pinned now — including an explicit anti-regression assertion that
the end-of-run ratio spans >10× with horizon, so no constant factor may be written down
again — in `tests/test_nitrogen_form.py`. ⚠⚠ **CORRECTION 2 (2026-07-27, found while scoping
option (B)) — CORRECTION 1'S OWN REPLACEMENT BAND WAS MEASURED ON A MIS-DRIVEN SCENARIO SET,
AND THE ANTI-REGRESSION PIN IT LEFT BEHIND WAS CERTIFYING A DRIVER ARTEFACT.** Correction 1
drove **all four** sealed scenarios through `run_season` — but `perennial`/`consumer` are
driven by **`run_perennial`** in their own goldens, and **the annual reset is what makes
them perennial**. Correctly driven: `sealed_chamber` **191.78**, `water_biting` **173.37**,
`perennial` **10.91**, `consumer` **9.87** ⇒ true spread **19.4×, not 1.11×**. **Both**
assertions correction 1 left behind were artefacts — `peak spread < 1.2` (truth 19.4×) and
`final spread > 10` (truth **2.21×**) — and are **RETIRED, not re-tuned** (a widened bound
preserves the shape of a claim that is gone). Two sentences **WITHDRAWN**: the 5-yr
end-of-run "**9076**/**11877**" are really **242.9**/**235.2**, and "by year 5 `litter_n` is
1.3e-11 kg — numerical dust" is false for the *actual* perennial chamber (**6.05e-05 kg**,
six orders larger) because the dropped reset is what refills it. **THE MECHANISM: "peak
`litter_n`" silently names TWO DIFFERENT EVENTS** — the seasonal senescence maximum in a
shedding-fed chamber, versus the **annual dump** in a reset-driven one (step 611 of 1525 =
one step past the year-2 boundary), which deposits the dying plant's retained N at C:N
**5.6–6.1**. That elevated concentration is *this work's own recorded limitation 5*, so
limitation 5 turns out to **set** the reset-driven pool's C:N rather than being a footnote.
**The scope-B projection SURVIVES, magnitude unmoved, scope narrowed**: `observed_fraction =
0.75` was only ever entitled to the **shedding-fed** regime (a `k_min/k_decomp` relationship
means nothing for a pool whose C:N comes from the dying plant), and those two give
0.781/0.706 ⇒ **31–83, mean 47, all stand**. **A number can be right while the sentence
justifying it is wrong.** Re-pinned with each scenario driven the way its golden is, the two
regimes asserted separately, the peak-lands-one-step-past-a-year-boundary *mechanism*
asserted, and an **inverted** anti-regression pin (spread > 10×). **Sweep done, not
assumed**: every other test that *runs* a perennial scenario already uses `run_perennial`;
`test_nitrogen_form.py` was the sole site. Test-and-docs only — **no `src/` change, no
golden moved, nothing unfrozen**. ⚠ **TWO OF THE THREE REASONS THE DECOMPOSER CALIBRATION
GAVE FOR NOT MOVING `mineralization_rate` ARE NOW FALSE** — it is no longer "behaviorally
inert" (it sets this ratio) and N/C are no longer "uncoupled" (which was the other half of
"litter C:N isn't physical"); only the **pool-identity** objection survives. And two
independent lines converge: S&S's 39-soil range (0.005–0.0136/day) puts the pool at
**31–83**, pooled mean **47** — at or below real residue — vs our uncited 0.03/day giving
**~184**. **Value UNMOVED** (scope B, a user call); the consequence is **pinned**. ⚠ **The
`f_N ≡ 1` margin fell ~2.5 orders while the conclusion held**: 1000× (the old `uptake/k_sen`
equilibrium) → 3.8× on the plateau → **~1.07×** at `open_season`'s peak; crossing **14.42
t/ha** vs a **12.633** peak (88 %), so any change growing the open-field crop ~15 % moves a
frozen golden — `mineralization.py`'s "~1000× above critical" sentence was **updated, not
left to rot**. ⚠ **Recorded limitation — the ONE-POOL model showing through**: shedding at
residual means a senescing plant *retains* its N while the denominator collapses, so
concentration rises unbounded as biomass→0 (~110× target at 3 yr, **~6e6×** at 5 yr) —
harmless (f_N saturates, N conserved exactly) but real remobilized N goes to **grain**;
related named seam: chambers seed `litter_carbon0` with **no `litter_n0` counterpart**,
inflating perennial/consumer pool C:N far above the sealed 465. **Tests**:
`tests/test_nitrogen_form.py` (12 pins incl. the crossing, the plateau-vs-extrapolation
reading, and shed-C == `Senescence`'s litter leg — the recomputation drift hazard).
**Rewritten not weakened, each with its reason**: `test_sealed_plant_n_is_drained`'s
"plant_n declines" was **true only because of the absurd IC** (a growing crop accumulating N
is correct) ⇒ now pins the withdrawal + target-as-floor; and `test_nitrogen.py`'s default
`plant_n0 = 0.2` is **130× target under the new law ⇒ zero deficit ⇒ every uptake test would
have passed VACUOUSLY on zero legs** — the default is now N-starved and the demand branch
has its own tests. `src/simcore/` diff empty; Rust hand-mirrored
(`science::target_n_concentration`, both flows, the reset, `plant_n0`, 4 params in/1 out of
`biosphere_params.txt`). (B) was then **diagnosed and BUILT** — the diagnosis is next, the
build at the end of this row; (C)/(D) remain OPEN; (D) still fights the fast-edge closure
requirement. **THE (B) DIAGNOSIS (2026-07-27, probes in `M:/claud_projects/temp/ncycle_b/`):
(B) CANNOT DELIVER THE THING IT IS NAMED FOR, and the option's name in the plan was
corrected accordingly (`Immobilization` → `microbe-mediated N transit`).** (1) **CUE is 1.0
in this tree** — `Decomposition` moves *100 %* of decayed litter C into `microbial_carbon`
and respiration is a separate draw (the deliberate Step-4/5 split) ⇒ introducing a
literature CUE (~0.3–0.4) **would move CARBON**, re-opening the decomposer calibration under
its fast-edge closure requirement, i.e. costing like **(D)**; so (B)'s N legs **must** be
computed off the carbon partition already applied, never off a second CUE (the (A)
recomputation-drift hazard one flow over). (2) ⚠ **The pool-identity objection BITES**:
`microbial_carbon` peaks ~0.95–1.01 mol C against a ~3.0 litter pool — **comparable to
litter, not the few % that standing microbial biomass is** — so it is a *transit* pool
(CUE=1.0 is why), and imposing the textbook homeostatic microbial C:N (~8) would demand
**90–152× the litter N present**. **REFUSED for the reason this project refused twice
before** (DPM/RPM labile re-read, S&S soil-N₀-vs-litter-N): *redefining what a pool MEANS so
a constant fits is a semantic model change wearing a provenance hat.* ⇒ the immobilization
seam stays open with a **measured obstacle** instead of a deferral. (3) **What (B) CAN
deliver, and the identity is the payoff**: `litter_n→microbial_n` at
`decomposed_C·(litter_n/litter_C)` and `microbial_n→soil_n` at
`respired_C·(microbial_n/microbial_C)` (f_O2 included — a *reason* to recompute rather than
reuse a bare rate); since `decomposed_C/litter_C ≡ decomposition_rate`, the first leg **IS**
`decomposition_rate · litter_n` ⇒ **`mineralization_rate` RETIRED**, no new param, **the
second weakly-supported param discharged by a FORM change rather than a citation hunt**
(after `n_senescence_rate`). ⚠ Must be written **recomputed-stoichiometric, never collapsed
to the rate** — the identity holds only while `Decomposition` stays first-order. (4)
**Invariance MEASURED, not inherited** (the plan's "(B) is behaviorally inert" is the exact
claim finding 5 had to overturn for (A)): the candidate flows were *run* in-process — carbon
**byte-identical**, `rationed==0`, N exact in
`sealed_chamber`/`perennial`/`consumer`/`water_biting` **and `sealed_station`**, the last
run specifically *because no probe had touched it* and it is in the cascade. ⚠ **A probe bug
the CONTROL DID NOT CATCH**: a first run showed a 2.281 carbon delta and *both* hypothesized
confounders (flow-id reduction order; an added zero stock) came back **clean** — reading as
"the effect is real". It was not: `Registry(flows, stocks)` had dropped the **aux
processes**, freezing `thermal_time` at 0. The tell was a **crash signature**
(`annual_reset`: `storage_c 0.0` — a plant that never filled grain), not the control. **A
clean control eliminates only the confounders you thought of; it is not evidence the effect
is real.** (5) **The payoff at its real size, and it is TWO REGIMES**: shedding-fed
191.8→**100.6** / 173.4→**98.7** (~2.2× straw → **~1.24×**, a **~1.8× refinement, NOT a
fix** — the 4-orders headline belongs to (A)); reset-driven barely moves (51.4→44.2,
10.9→10.1, 9.9→9.1) because the dump's C:N is set by the dying plant, not by either rate.
(6) **`microbial_n` may be a POOL**: `organ_stock` sets `extinction_threshold = 0.0` and the
pass needs `amount < threshold`, i.e. a *negative* amount, which structural positivity
prevents ⇒ orphaned-N is unreachable; **named seam** — raising that threshold above 0 would
require zeroing the N counterpart with it. **THE DECISION WAS THE USER'S AND THE REASON WAS
NOT PROCESS** (resolved: "do what you recommend" ⇒ BUILT): the recorded case for (B) was
immobilization, which is measured unavailable, so the real trade was "retire a contested
param by stoichiometry + a ~1.8× refinement on one of two regimes" against "unfreeze **two**
contracts" (biosphere `flow_set` 17→18 + `param_files`, station manifest, 6+ goldens, the
Rust mirror losing a param, crossport). **DIAGNOSIS (2026-07-27), which the build then
partly corrected:** `docs/plans/post-roadmap-nitrogen-cycle-form.md`. Read-only probes
(`M:/temp/ncycle/`), and **three premises this repo had recorded turned out false**: (1)
**`plant_n0` is NOT what forces `f_N≡1` — uptake is.** The steady state is
`max_uptake_capacity / n_senescence_rate = 0.0015/0.01 = 0.15 kg` vs the observed
**0.150036**, i.e. `plant_n`'s equilibrium is **the ratio of two uncited constants** (a
capacity flagged ~6× realistic over a rate whose retrieval is "exhausted") — *that*, not the
IC, is why a 52 g plant holds 0.15 kg N; re-scaling `plant_n0` 0.5→physical leaves
`f_N_min=1` and converges to the same 0.15. (2) **in the sealed chamber, `f_N` cannot bite
at any physically-defensible SOIL-N scale** (⚠ the scope is load-bearing — an earlier draft
of this row, the plan heading and the memory hook all said "at ANY N scale", flat, which is
the **meta-finding's 11th instance** with the counterexample *in the same row*: `n_limited`
bites at 0.176, and finding 5 puts the real margin on the **plant** side; measured on
**one** of the 7 frozen scenarios, uptake **on**) — moving `soil_n0` 100→0.30→0.05→**0.004**
kg N/m² (5× the plant's whole-season demand, availability down to 0.0026) keeps `f_N_min=1`
with the **carbon trajectory byte-identical** in all four; because peak biomass is **52 g
DM/m²** ⇒ demand 0.000785 kg, which per-area supply covers regardless. Not a param error:
implied SLA **22 m²/kg** (real wheat 20–25), peak LAI **0.51** — the sealed chamber is
*carbon*-limited **by design**, so its plant is ~25× smaller per m² than the crop the N
params are sized for. ⇒ making N physical, and immobilization, are **safe/additive but
behaviorally INERT**. (3) **The gap is precisely that nothing ties N shed to C shed**, so
litter C:N is the unconstrained ratio of two independent rates — and it **costs 1–4 orders
of magnitude**: frozen form gives litter C:N **0.004** in-run (≈1 C:246 N) / 8.02 offline,
vs wheat straw ~80; the documented **N:C-coupled** seam @ 3 % tissue N gives **77–107**.
Exact law `litter C:N ∝ (tissue C:N) × (mineralization_rate/decomposition_rate)` (all rows
1.894× it). Finding 2 is what makes this measurement **exact** rather than indicative — zero
N→C feedback means the coupled N ODEs integrated on the *recorded* carbon trajectory ARE the
in-tree result. **The payoff: the form change discharges "the highest clean-room risk in the
project"** — the coupled param is a **tissue N % DM** (citable; `n_critical` 1.5 % and
`carbon_fraction` 0.45 already are) not a 1/day rate (for which retrieval was declared
*exhausted because no source has that shape*); resorption **efficiency** (~50 %, what the
literature *does* publish) becomes expressible. **Options, dependency-ordered**: (A)
N:C-coupled shedding **+** demand-based uptake — the coherent minimum, and the two are
needed **together** since coupled shedding removes the `uptake/k_sen` brake ⇒
capacity-uptake would let `plant_n` grow unbounded; (B) immobilization (`microbial_n`, needs
A); (C) the DS-dependent form, **not** subsumed by A (frozen carbon `rdr_*` are themselves
flat); (D) ⚠ **the N→C throttle — the ONLY option with a carbon effect, and it fights the
decomposer calibration head-on**: real soils decay high-C:N residue *slower*, but closure
was measured to **require the fast edge**, so expect it to break closure in every sealed
scenario — a scientific conflict to price *before* attempting. Cascade if taken:
Python-canonical unfreeze (pivot rule 3) — biosphere manifest params+`flow_set`,
`sealed_chamber`/`sealed_station` (⇒ **station** manifest too)/`n_limited`/`water_biting`
goldens, Rust mirror + crossport. `n_limited` is the one place `f_N` **does** bite
(verified: min **0.176**, 187/306 steps) and is *not* one of the 7 frozen scenarios — the
natural pin that (A) preserves that regime. **ADVISOR-REVIEWED at last (2026-07-27, after 4
overload failures), and it moved the deliverable**: the review's blocking catch was that
**finding 1 predicts (A) is NOT carbon-invariant** — the margin pinning `f_N` at 1 is
**plant**-side (`plant_n` equilibrium 0.15 kg = **191×** the 0.000785 kg critical demand;
the IC 0.5 is 637×), finding 2 varied the **soil** side while the frozen dynamics held
`plant_n` at 0.15 *regardless*, and (A) deletes exactly that mechanism ⇒ margin 191×→~1× ⇒
invariance is decided by the target vs `n_critical`, so it had to be **measured, not
footnoted** ("the more physically honest the form, the less likely it's inert"). **Measured
(findings 5–6, `probe_fn_under_A.py` + `probe_fn_all_scenarios.py`, exact by the
zero-feedback license — `f_N ≡ 1` ⇒ carbon unchanged ⇒ the recorded trajectory IS the (A)
trajectory, a fixed point not a screen): invariance HOLDS for a flat target ≥ ~2× critical
(3.0 % DM) in ALL 7 scenarios** (`f_N ≡ 1.0`, 0 steps below 1, uptake capacity **never**
binding — the 52 g plant's demand is met every step), **and the TARGET FORM is the
load-bearing choice**: a target *at* `n_critical` bites everywhere (0.75–0.89 — a tautology,
"critical" *means* stressed-below, and the dip is a real one-step lag vs ~11 %/day RGR, not
a probe artifact), while a **declining Greenwood dilution target (4.5→1.2 % DM) stays
invariant in the six winter-wheat scenarios but BITES in `day_neutral` (0.713)**. **The
advisor's dilution-curve prediction was HALF right and the failing half is the finding**:
the curve *does* cross below critical late season, yet `f_N` holds at 1 because **its
denominator collapses** — `leaf+stem+root` falls 1.9612→0.0157 mol C (**×0.008**) into
storage+senescence while coupled shedding removes N only ∝ *senescence* carbon, so
concentration **RISES**; robust to the demand denominator (whole-plant incl. storage:
identical) and to 50 % resorption. ⚠ `n_limited` **preserves the regime but not the
magnitude** (0.176→**0.0**: uptake off ⇒ coupled shedding is conc-*neutral* ⇒
growth-dilution runs unopposed). ⚠ **"Carbon-invariant" ≠ "no golden moves"** — the goldens
record N stocks, so the cascade is the same size either way; the target choice decides only
whether the **science** moves too. **THE FORK IS NOT NEUTRAL, and a neutral presentation is
itself the trap** (2nd advisor catch): the **declining form is the LITERATURE-shaped one** —
Greenwood 1990, *already cited in `nitrogen.py`*, is titled "Decline in percentage N … with
increasing plant mass" — while flat 3.0 % is the **invariance**-shaped one, so picking flat
*because* carbon stays byte-identical is **choosing a model form for its effect on frozen
output**, the co-adaptation/backfitting shape this repo has refused before (consumer-chamber
2×, the DPM/RPM labile re-read, ruling B). My draft had listed "carbon byte-identical" as
flat's **advantage** — exactly how the trap reads from inside. And the **cost asymmetry is
smaller than it looks**: `day_neutral` is **authored** content (runtime-only, not frozen),
so a form moving only it is **not a freeze event** ⇒ the case for flat is *weaker*, not
stronger. **Two further findings from the review**: (7) `f_N` is **STRUCTURALLY** the only
N→C channel — all four reads of an N stock in the biosphere are `carbon_budget.py:206`
(`f_N`, the one carbon leg) + `nitrogen.py:166`/`mineralization.py:139`/`:176` (N-only legs)
⇒ the fixed-point argument is a **proof on four grep hits**, not just the empirical
byte-identity; (8) the probe **re-seeded** `plant_n` at each re-sow, but in-tree
`annual_reset` is **carbon-only** — `plant_n` *persists* as "an N **windfall** for the small
seedling, harmless only while `f_N ≡ 1`" (the docstring's own condition) ⇒ the probe took
the **harsher** assumption, so invariance holds *a fortiori*, **but** (A) cannot inherit the
windfall (coupled shedding dumps `plant_n`→`litter_n`, and the conservation gate then forces
seedling N to come *from* a stock) ⇒ a **THIRD** deferred seam the tree already names, and
it sets the re-sown crop's starting C:N — the quantity finding 3 measures. Availability was
read off the frozen `soil_n`, benign **for a stated reason** (it saturates at 1.0
everywhere: `soil_n0=100` vs `sn_critical=50`; `n_limited` has uptake off), the one place
the offline integration is not self-contained. **THEN THE USER SET THE CRITERION — "I want
what represents reality more faithfully" — AND IT DISSOLVED THE FORK BY TURNING IT INTO A
RETRIEVAL QUESTION: `sources/greenwood1990.pdf` was ALREADY ON THE SHELF, and BOTH branches
were invented.** Read first-hand: eqn (6) `%N = a·W^-b` **for `W > 1.0 t/ha`**,
`a=`**5.697** (C3), `b=`0.5; *"Data obtained with W less than 1 t ha⁻¹ were **always
omitted**"*; and the paper states the below-domain behaviour itself with a mechanism + an
Ågren 1985 cite — *"At [W=1 t/ha] the growth rate gradually changes from being almost
exponential to linear. **When growth is exponential plant %N remains constant** … a = 5.7 %
is therefore the best estimate of %N needed in the dry matter of **young tissue**"*; plus
*"%N and W refer to … the whole plant (**excluding fibrous roots**)"* ⇒ `W =
leaf+stem+storage`, **NOT** `f_N`'s own `leaf+stem+root`. So the faithful form is
**piecewise — constant 5.697 % below 1 t/ha, `5.697·W^-0.5` above — and the plateau is the
PRIMARY'S OWN STATEMENT, not our interpolation.** ⇒ **flat WAS right, at 5.697 % not my
invented 3.0 %, and for the OPPOSITE reason** (our crops sit *below* the curve's domain,
where %N *is* constant), while **the declining ramp I had just recommended as "the
literature-shaped form" was Greenwood extrapolated into the region he EXCLUDED — the least
faithful of the three — and `day_neutral`'s 0.713 was that extrapolation, not physics.**
Measured on the **manifest's** set (`probe_greenwood_primary.py`): `f_N ≡ 1.0000` in **all 7
frozen scenarios** + `water_biting` + `day_neutral`. **⚠ FINDING 9 ALSO CORRECTED FINDINGS
5–6 — the meta-finding's 12th instance, mine, and the tell is that THE NUMERAL MATCHED BY
COINCIDENCE**: "invariance holds in ALL 7 scenarios" (committed to this row, the plan and
memory) was measured on a set overlapping the frozen seven in **3 rows** — it omitted
`open_season` + both long-horizons + `drift_summary` and included 4 non-frozen scenarios.
`open_season` is the omission that mattered: **the only frozen scenario reaching field scale
(12.633 t/ha, 89 d) and therefore the only one INSIDE Greenwood's domain.** The conclusion
survives and is *stronger*, which is exactly what makes it easy to soft-pedal ⇒ check a
scenario list against the **manifest**, never against its own length. **⚠ AND DO NOT
OVERSELL "fidelity produced invariance"** (advisor): it holds on a **12 % margin against a
DENOMINATOR DEFINITION**, not a robust separation — Greenwood crosses `n_critical` at **`W =
(5.697/1.5)² = 14.42 t/ha`** vs `open_season`'s 12.633 (88 % of the way), and the *wrong*
denominator (`W` **incl.** fibrous roots — which is `f_N`'s own, hence the tempting choice)
**DOES bite** (0.9750, 3 steps, `open_season`). Two forms both defensible as "Greenwood",
one moves a frozen golden ⇒ both facts adjacent, never merged; **the 14.42 t/ha crossing is
a PIN** (`test_oracle_gap.py` precedent) since any calibration growing the open-field crop
~15 % flips a frozen golden. **FINDING 10 — (A) DELETES THE KNOB `n_limited` IS BUILT ON**
(the advisor's blocking item, resolved into a *design requirement discovered before
building*): ⚠ first, **the 0.0 is NOT LICENSED** — the zero-feedback license holds only
where `f_N ≡ 1`, and `f_N` already bites in `n_limited`, so every `n_limited` cell in
findings 6/9 is a **screen** whose error has a **known sign** (lower `f_N` ⇒ less growth ⇒
less growth-dilution ⇒ *higher* conc, a negative feedback the open loop suppresses) ⇒ 0.0 is
an **upper bound on severity**; ⚠ second, **the probe's own baseline was wrong and the
GOLDEN caught it** — reconstructing `plant_n` as `plant_n0·(1−k_sen)ⁿ` gave min 0.0000/305
vs the recorded **0.176/186**, and reading the recorded stock reproduces 0.1759/186/final
0.3754 exactly ⇒ **reconstruct a frozen quantity only to CHECK it against the recorded one,
never to replace it**; what the screen *does* establish is structural — the frozen scenario
**recovers** (min 0.176 → final 0.375, a *modulation* scenario) while under (A) `f_N` hits 0
at step 195 and is **ABSORBING**, because **(A) makes `plant_n0` INERT** (it seeds `plant_n`
from the tissue-N *target*, overwriting the deliberately-tiny `6e-5` reserve with the 5.46 %
plateau) ⇒ **(A) must expose a seedling-N knob** for `n_limited` to keep testing what it was
built for — legitimate authoring on a non-frozen scenario, but **work (A) owes**, alongside
finding 8's re-sow seam. **FINDING 11 — the criterion reaches further than the available
work**: consistently applied it does not stop at (A), it reaches **(D)**, which is measured
to fight the fast-edge closure requirement ⇒ state up front that **the available faithful
sub-chain is (A)[+(B)]**, and that **making N faithful does not make the CHAMBER faithful**
(a 52 g/m² carbon-limited plant against field-sized N params is the obstacle, not the N
form). Justes 1994 (the wheat-specific curve) is on the shelf but is a **scan with no text
layer** (11 bytes) — deliberately NOT opened: Greenwood's C3 curve is the one `nitrogen.py`
cites and it is now read first-hand. **RECOMMENDATION (advisor-endorsed): scope (A), target
= Greenwood eqn (6) piecewise with the stated domain bound + the excluding-fibrous-roots
denominator, all three constants first-hand — with the 14.42 t/ha crossing PINNED and
`n_limited`'s seedling-N knob designed in.** **⇒ OPTION (B) BUILT 2026-07-27 (user: "do what
you recommend"), biosphere unfrozen + re-frozen + station manifest cascaded — and THE PAYOFF
WAS BIGGER THAN THE DIAGNOSIS PRICED, because the C:N law changed KIND rather than
magnitude.** Shipped: `Mineralization` **deleted**, replaced by `LitterNitrogenTransfer`
(`litter_n→microbial_n`, carried by `Decomposition`'s `decomposed_C`) +
`MicrobialNitrogenRelease` (`microbial_n→soil_n`, carried by `MicrobialRespiration`'s
`respired_C`, **`f_O2` included**), one shared kernel `carried_nitrogen(moved_C, pool_N,
pool_C)`; new stock `microbial_n` as a **POOL** (deliberately not a POPULATION like its
carbon sibling — `organ_stock`'s extinction pass would orphan N the carbon side still holds;
pinned as a TEST, not a comment). **`mineralization_rate` RETIRED and
`params/mineralization.yaml` DELETED — the first param FILE this project has removed rather
than re-valued** (a `param_files` **membership** change, not a hash move), since (A) had
already retired its only other param. Its five rounds of negative retrieval results are
archived verbatim at `docs/retired/mineralization.yaml` and pinned, because **a stale
NEGATIVE result suppresses the next search** and is the more expensive thing to lose.
**RESULTS: carbon byte-identical in all 8 scenarios, `drift_summary.json` regenerated
BYTE-UNCHANGED** (the zero-carbon-effect claim confirmed at the *artifact* level, not just
per-stock); **10 goldens moved and every one moved ONLY in nitrogen** — verified
**structurally**, by parsing both snapshots per stock and grouping by *quantity*, because
**a grep over a unified diff cannot tell you which stock an `"amount"` line belongs to**;
`open_season`/`n_limited` **structurally untouched** (open field builds no `litter_n` ⇒ no
`Mineralization` to replace); crossport **101**, cargo clippy/test green, `git diff
src/simcore/` empty. **⚠ THE FINDING — the diagnosis's own "~1.8× refinement in one of two
regimes" UNDERSTATED IT, and the reason was an identity the diagnosis had already written
down without following through**: under the retired form N left the litter pool at a FREE
rate while C left at `decomposition_rate`, pushing the pool **2.727×** away from its input's
ratio; both currencies now leave on the **SAME** flux, so the pushing factor is exactly
**1** and `pool C:N → shed C:N = 90` (was `90 × 2.727 = 245.5`). Measured: shedding-fed
**98.7–100.6** at peak and `sealed_chamber` **ends at 90.6**, within 0.7 % of the shed ratio
⇒ **~1.25× wheat straw's ~80** as committed — and **exactly 1.125× for the MODEL**, because
⚠ **the residual above 90 is the N-FREE SEED, not a "pulsed transient" (advisor catch, then
measured, and it makes the result STRONGER)**: with both currencies on the same flux
`d(C/N)/dt = 0`, so pulsing **cannot** move the ratio — that mechanism belonged to the
RETIRED differential-drain form, i.e. **I wrote an explanation that outlived its mechanism
in the very commit that retired three others for exactly that**. The chambers seed
`litter_carbon0 = 3.0` with **no `litter_n0`** (C:N = ∞, a seam (A) had already named);
remove it and pool C:N **equals the shed ratio to 1.4e-15 relative AT EVERY STEP** — an
identity, not a band — so the committed excess is a known unphysical IC decaying at
`decomposition_rate` (3-yr `sealed_chamber` ends 90.6; 1-yr `water_biting` still 98.6).
Committed-scenario bounds are now labelled **scenario facts, not model facts**. Where
pre-(A) gave **0.004** and post-(A) direct gave 173–192. **The point is not the number: the
litter pool's C:N stopped being an accident of two unrelated rate constants and became a
function of the COMPOSITION of the material that fell in** — both of whose numbers are
cited. **⚠ THREE PREVIOUSLY-PINNED CLAIMS RETIRED AND NONE WAS WRONG** — each a true
measurement of a form that no longer exists, so **resolved, not corrected** (the distinction
matters: this project's habit is retiring *artefacts*, and these are not): the 245.5 law
(its `k_min` is gone); "a shedding-fed pool runs N-poor at 0.71–0.78 of the law" (it ran
N-poor *because* N drained 2.7× faster); and the end-of-run inflation — **which WAS the
differential drain**, so the horizon-dependence correction 1's anti-regression pin existed
to guard against is **gone at its source**, and that pin was replaced by its **INVERSE**
(`end/peak` must now be ≈1) rather than relaxed. **A pin guarding a mechanism you removed is
not protection, it is decoration.** The **scope-B projection test is retired too, and its
PREMISE rather than its arithmetic failed**: there is no `mineralization_rate` left to move
into S&S's range, so the question was **dissolved, not answered** — which also disposes of
the last surviving objection (a param that does not exist cannot be mis-anchored to the
wrong pool). **⚠ WHAT THIS DOES NOT CLAIM**: the decomposer **carbon** rates are untouched
and still run at the fast edge (`decomposition_rate` 4.0/yr, Olson's fastest), and the pool
C:N now *inherits* that rate — the honest statement is that the N cycle no longer
contributes a **separate** uncited rate, NOT that the decomposer side is cited. The **second
regime is untouched** (reset-driven 10.9→10.0, 9.9→9.1; the dump's C:N is set by the dying
plant). **⚠ THE BLOCKING CATCH THIS BUILD STARTED FROM (advisor): B-finding 4's invariance
table had FIVE rows and the manifest freezes SEVEN** — it omitted both 15-yr long-horizons,
`drift_summary` and `open_season`, i.e. **the 12th meta-finding one option later**. Not
bookkeeping: (B) parks N in a **standing** pool so `soil_n` sits permanently lower, and
`soil_n→availability→uptake→plant_n→f_N` is a real **SECOND-ORDER** channel — **finding 7's
"only N→C channel" proof closes the DIRECT READ, not the SUPPLY PATH**. Measured over the
full manifest roster, each scenario driven the way its own golden drives it: the drain is
**standing, not accumulating** (`perennial`'s `min(soil_n)` is `99.995967` at **both** 5 and
15 years, identically), worst drain `8.6e-4` kg vs a 100 kg pool with `sn_critical` 50 ⇒ ~5
orders from biting; `f_N ≡ 1.0` in every sealed scenario and `n_limited` reproduces its
recorded 0.175851/187 unchanged. **⇒ OPTION (C) DIAGNOSED + PRICED 2026-07-27 AND
DELIBERATELY NOT BUILT — the first option this work has turned down, and the reason is a
MEASUREMENT.** `docs/plans/post-roadmap-nitrogen-cycle-form.md`, "THE (C) DIAGNOSIS";
read-only probes in `M:/temp/ncycle_c/`, 12 pins in `tests/test_senescence_form.py`. ⚠ **The
plan's "each is carbon-invariant except (D)" was FALSE for (C)** (advisor catch, then
measured): [A] §3.2.6 is *biomass* death, and under (A) the N shed is CARRIED BY the carbon
flux, so there is no reading where (C) touches only the N leg ⇒ it moves **every carbon
golden**, i.e. a **(D)-sized** cascade. The line was written when (A)/(B) were the live
options and never re-checked — subset-claim-written-flat, about our own option list. ⚠ **THE
ZERO-FEEDBACK LICENSE IS GONE** (carbon changes at the source), so every number came from a
full re-run — and (C) is DVS-keyed, i.e. it rides exactly the aux that probe B2's bug froze
at 0, so the probes ASSERT `max(DVS)==2` first. **FINDING 1 — our own record quoted the
wrong table, and the failure was DE-QUALIFICATION, not mis-reading.** The source has TWO
`LLVT` tables: **Listing 5** (p. 212, "Crop data for rice (variety IR36)", the one §3.2.6
cites by name) peaking at **0.012/day**, and **T10** (p. 113, an *exercise answer*) peaking
at **0.15/day** — 12.5× apart. `docs/retired/mineralization.yaml:268` recorded the locus
CORRECTLY ("p. 113, exercise T10", with rice/biomass caveats); every restatement downstream
— the same file's own summary, `post-roadmap-citation.md:413`, this plan's (C) bullet —
dropped it. **The careful sentence stayed put while the careless paraphrase travelled**, and
the table §3.2.6 actually cites was never retrieved in five citation rounds. ⇒ **a locus
error survives inside a correctly-attributed quote**; round 4's "open the paper you cite"
one level in — open the **right part**, and treat an example-in-an-exercise as provisional
until the reference parameterization is looked for. The **FORM** claim survives both
readings (every one is ZERO below DS 1.0) — only the magnitude was mis-sourced, and it
decided a threshold (below). **FINDING 2 — what the primary licenses** (p. 95, first-hand):
DS-keyed for **leaf and root**; **no stem function exists** ("except for their reserves,
stems do not lose weight") ⇒ `rdr_stem` is an **existence** gap, not a value gap; the
example is **rice IR36**, not wheat; and the source states its own outcome band (**"40-60 %
of leaf area"** lost at harvest) plus an explicit licence to calibrate. `rdr_root` (0.01
flat) is within **10 %** of Listing 5's plateau — so the "runs fast" reading belongs to
`rdr_leaf` alone and must not be quoted as covering all three. ⚠ **STEM-ONLY WAS NOT
MEASURED** (advisor): every number here runs the **combined** form, and zeroing `rdr_stem`
alone is the one piece plausibly **separable** from the canopy problem (it shrinks the plant
rather than blowing up LAI) ⇒ "(C) is refused" must **not** be read as "stem-only was
evaluated and refused" — it is **unpriced**, not priced-and-rejected. ⚠⚠ **MEASURED
2026-07-28 (C-finding 8) — PRICED AND REFUSED, and the parenthesis above was WRONG TWICE.**
(i) **It GROWS the plant**: `rdr_stem` is a LOSS term, so `open_season` peak W **12.633 →
13.639 t/ha (+7.96 %)**, closing more than half the margin to the Greenwood crossing without
crossing it (0.876 → 0.946). A prediction written in the grammar of a measurement — *and not
baseless*, since three of four organs do shrink (leaf −3.96 %, root −3.91 %, storage −3.97
%, a **common haircut** whose measured cause is the extra maintenance respiration of a
bigger standing stem, +1.49 mol C): it named the whole plant for the behaviour of the
majority of its organs while the one dissenting term (stem **+23.4 %**) dominates. Honest
reading: **"stem up, grain down"** — bigger and worse — and the branch would **open** a form
gap, since one `stem_c` pool cannot express [A]'s own *"except for their reserves"*. (ii)
**"Separable" is half true and the true half is worthless**: it IS separable from the canopy
problem (peak LAI **falls** 5.191 → 4.985) — **one single-number change moves the mass and
area tripwires in OPPOSITE directions**, the sharpest demonstration yet that they are
different quantities — but the canopy branch was already discharged by the regulator, so
**`perennial`'s CLOSURE is the only branch left and stem-only hits it**: `rationed` **0 → 1
under EULER** (the frozen reference configuration; one firing is a hard break — goldens
assert `rationed == 0`, `run_scenario` raises), firing at **step 502 (year 1, day 197)** —
within-season, not a horizon artefact; min CO₂ 0.008674 vs the frozen 0.038734. ⚠ **THE
FIRING STEP IS MEASURED AND MY DRAFT INFERRED IT** (advisor catch): I read the location off
the **CO₂ argmin** and reported it as the *rationing* location, under a constant named for
the rationing step — two different quantities, never measured to coincide, and
"within-season" is the exact word separating this from the beyond-horizon tiling artefact
the decomposer row documents. Re-measured by **horizon truncation** (deterministic run ⇒ the
smallest horizon that rations *is* the firing step): they coincide at 502 — **and the
inference was CIRCULAR, not merely unverified**, since entering that step the pool is in
free fall (0.727→0.504→0.222→0.009) so the trough is *the value the backstop clamped to*;
the argmin is **downstream of** the firing and could not have disagreed. Both now asserted
separately, and, independently, the decade CO₂ attractor collapses **0.05484 → 0.01619** vs
the committed 0.05 floor — **3.4× too low WHILE STAYING STATIONARY**, so the *level* guard
catches it and a stationarity check alone would have passed it. ⚠ **THE INTEGRATOR PATTERN
INVERTS**: finding 5's *"Euler reads clean and that is the trap"* has a **mirror** — here
Euler rations and **RK4 survives to 15 years** with its CO₂ minimum unmoved (0.075815 →
0.075893) ⇒ **neither integrator screens for the other**, and Euler is decisive only because
the contract is *about* Euler. ⚠ **THE MECHANISM is a STANDING STOCK, and my first
hypothesis was refuted by the probe**: I predicted litter *starvation*, but the litter
pool's mean falls ~13 % and microbial 0.5 % against a **55 %** fall in the CO₂ trough — the
recycling is not starved. A sealed chamber's carbon inventory is **fixed** (measured
identical to <1e-9, 3.517000 mol C), a pool's equilibrium size goes as **1/(loss rate)**, so
zeroing the rate makes the stem a one-way sink and every other pool funds it: at the trough,
standing tissue **+0.1179 mol C** drawn **~67 % from the soil pools**, **~33 % from the
atmosphere**. **That is why the open field grows on the change the chamber chokes on** —
scope (A)'s finding 11 from the other side, reached a third time independently. ⇒ **ALL
THREE BRANCHES OF (C)'s REFUSAL ARE NOW EXAMINED RATHER THAN ASSUMED** (2 discharged by
measurement, 3 refused on principle, **1 measured to catch the full form, the
regulator-assisted form AND the smallest separable piece**). ⚠ `sealed_station` **NOT run**
— the biosphere gate fails first so the cascade is moot; recorded as an **unmeasured** leg,
not a clean one, since creating that exact debt silently is what this finding exists to
discharge. 9 pins in `tests/test_senescence_form.py` §7 (34 pass); **cascade: exactly 1
manifest hash, biosphere-only** (`senescence.yaml`, comment-only — honor-system unfreeze,
`grep -l` confirmed, fifth application). No value/golden/code moved; `git diff src/simcore`
empty; Rust untouched. **FINDING 3 — the tripwire FIRES, measured not inferred**:
`open_season` peak **12.633 → 18.678 t/ha (+47.8 %)**, past the **14.4248** Greenwood
crossing `test_nitrogen_form.py` laid down for exactly this ⇒ **`f_N` = 0.995213 for 6/306
steps, the FIRST `f_N` bite in a frozen scenario** (a live feedback — `carbon_budget`
multiplies GASS by `f_water·f_N` — so the peak is self-consistent). ⚠ **State the size
honestly: 0.5 % over 6 steps.** The tripwire fires; N does **not** thereby become
load-bearing. And T10 lands at **96.2 %** of the crossing ⇒ **the wrong table reports "no
tripwire"** — the locus error was worth ~4 % of clearance either side of a threshold. **⚠
FINDING 4 — THE STRUCTURAL ONE, and it is why (C) is refused: THE FLAT `rdr_leaf` HAS BEEN
STANDING IN FOR CANOPY-REGULATION SCIENCE THE TREE DOES NOT HAVE.** Peak LAI **5.19 →
16.40** against real wheat's ~5–8 — and **both tables give the same peak** (16.40/16.16),
because both are zero below DS 1.0 ⇒ **not** the locus question, the half every reading
agrees on. No self-shading leaf death, no leaf-age cohorts, no SLA aging: this constant was
the canopy's only regulator on the way up. **The decomposer calibration's deepest finding
again, on the other side of the plant** ("the references were propped up by an unphysical
rate"). Corroborated **against the source's own table**: [A]'s stated 40-60 % leaf-area loss
is met by the **FROZEN flat form (38.5 %)** and missed by Listing 5 (**30.0 %**) and T10
(**97.9 %**) — which does *not* vindicate flat as a form, but says the constant was
implicitly sized to the right **INTEGRATED** loss with the **TIMING** entirely wrong. ⚠
**The 40-60 % comparison is INDICATIVE, not like-for-like** (advisor): our run ends at the
weather fixture's end, not at "harvest time"; the loss is measured off **peak LAI**; and
[A]'s sentence is about rice under a 102–135 d season. It supports the **ordering and rough
magnitude** — *none* of the three is inside the band — and **no distance-to-the-midpoint
comparison is drawn**, in prose or in the pins, since a nearness metric invented to rank two
misses is the fitted comparison this work exists to refuse. Amplified by crop transfer:
rice's season is 102–135 d, our anthesis is day ~251, so "zero below DS 1.0" removes ~250
days of shedding, not ~60 (`e^(−0.02·250) ≈ 0.0067`). **FINDING 5 — Euler reads clean and
that is the trap.** `rationed == 0` under Euler in all 8 scenarios; under **RK4 `perennial`
HARD-ERRORS** (`ArbitrationError`, scale_f 0.95277) — increment 1's "rationed under Euler,
hard-errored under RK4" repeating exactly. And `perennial`'s decade per-year CO₂ minimum
falls from a settled **0.05484** attractor to a **wandering 0.006–0.027**: fails the 0.05
floor by ~an order *and* loses stationarity. (The guard was recomputed **the way the
committed test computes it** and validated against its own comment first — "dips to ~0.039 …
settling to ~0.055" — after probe C2 invented its own transient rule and got 0.0387; finding
10's rule.) T10 is worse: `annual_reset` hard-errors ("seed bank too small to re-sow") in
**all four** reset-driven scenarios. `consumer`/`sealed_chamber`/`water_biting` survive
Listing 5 on both integrators. **THE VERDICT: (C) is BLOCKED ON A MISSING SCIENCE, NOT ON
EFFORT** — taking the primary's form as printed either breaks `perennial`, or needs the
canopy regulator finding 4 exposes, or needs a calibration whose **only** target is our own
goldens. [A] *does* license calibration ("should be calibrated to mimick specific
situations") — ⚠ **REFUSED**, the consumer-chamber-2× / DPM-RPM-labile / ruling-B shape.
Independently the same conclusion scope (A) reached about the oracle gap ("not a calibration
task"). **The natural successor is therefore NOT (C) or (D) but the CANOPY REGULATOR**
(leaf-age or self-shading-driven death), which is what would let the primary's form be
adopted without a fitted table. ⚠ **TAKEN THE SAME DAY AND HALF OF THAT SENTENCE IS NOW
MEASURED FALSE — see the row below.** **Also fixed**: the two de-qualified restatements
annotated in place (originals kept — the way they are wrong IS the finding), and
`senescence.yaml`'s three `TODO(cite)` tags now carry the measurement (⇒ **exactly 1
manifest hash, biosphere-only**, an honor-system provenance unfreeze; `grep -l` confirmed no
other manifest names the file — round 4's reflex error not repeated). No value/golden/code
moved; `git diff src/simcore` empty; Rust untouched. **(D) remains OPEN and still fights the
fast-edge closure requirement.** **⇒ (C)'s STEM-ONLY BRANCH AND (D) BOTH RE-PRICED
2026-08-10, read-only — and each re-price changed the KIND of its answer, not its
magnitude.** `tests/test_senescence_form.py` §8 (2 pins, 1 slow) +
`tests/test_nitrogen_throttle.py` (7 pins); probes `M:/temp/c_reprice/`. **(C)/STEM-ONLY —
the question the CUE build parked, answered as far as a run can answer it, verdict LEFT
OPEN.** ⚠ **The surviving leg's framing was half wrong and I had it wrong before measuring:
BOTH guards are WINDOW questions, not horizon questions.** The floor guard's failing year
**IS index 2** — the first year `transient=2` lets it see — so
`non_collapsing(summaries[2:])` contains 0.046065 at *every* horizon; and stationarity's
offending same-phase diffs sit at **fixed indices 2 and 3**
(`series[4]−series[2]`=+0.029145, `series[5]−series[3]`=+0.016837, vs a 0.015208 bound), so
`is_stationary` is False at 15 **and** 50 years while the series is flat to 8 decimals over
its last five. ⚠ Scoped as **measured at two horizons, not proved** (`bound = 0.2·max`, and
the max lands in year 0). **Run to 50 yr, subject and control on one harness: stem-only
settles at 0.075339 = 1.51× the 0.05 floor and ABOVE the frozen control's own attractor
(0.073291), `rationed == 0` on both** (checked because this family documents a
beyond-horizon tiling artefact). **Exactly ONE year of fifty is below the floor**; the
control has none. **The contrast is the point: the soil-fractionation re-refusal asked the
IDENTICAL 50-year question of a change it was about to refuse and got 0.031741 — 1.58× BELOW
the floor. Same test, opposite answer**, which is why it must be asked in both directions
rather than assumed. **The MANIFEST-named gate CLEARS**:
`perennial_long_horizon`/`liveness_floors` `max(tail) > 0.55` gives **0.643676** vs the
control's 0.634352 ⇒ **no third leg** (it had never been checked for stem-only — an advisor
catch, since that bound is contractual where the decade-CO₂ pin is only a committed test). ⚠
**AND THE PLANT IS NOT FREE — reading the improved trough alone is what the CUE row
forbids**: peak stem **+51.8 %**, peak **grain −11.8 %** ⇒ (C)-finding 8's “stem up, grain
down” holds here and **harder** than on `open_season` (+23.4 %/−3.97 %), and grain is the
seed bank the re-sow draws on; peak leaf **+1.98 %** where `open_season` gave **−3.96 %**,
so **the leaf sign does not transfer between scenarios** and was measured here rather than
inherited. ⚠ **A COUNTERFACTUAL REFUTED MY OWN HYPOTHESIS AND THE COMMITTED PIN WAS RIGHT**:
both guards fire inside years 2–5, so I expected them to be two readings of ONE event (which
would have made the committed “both halves are asserted” an overstatement) — splice the
control's year 2 into the subject and **the floor flips True while stationarity stays
False**, because `diffs[3]` does not involve year 2. Stated precisely, they are still **not
causally separate**: year 3 (0.053922) is inside the same dip, so the honest claim is *the
stationarity failure does not DEPEND on the single sub-floor year*. **THE VERDICT IS
DELIBERATELY NOT DECIDED**: whether `transient=2` fits a tree whose settling transient the
split measured at ~35 yr is a **contract question**. `transient=3` clears the floor, `5`
clears stationarity, the control passes at `0` — and picking a window because the subject
goes green is the consumer-chamber-2× / DPM-RPM / ruling-B / fractionation-seed-sweep shape,
refused four times. ⚠ **ANSWERED 2026-08-10 (the re-anchor row below) AND NOT BY CHOOSING A
WINDOW — so the sentence above is the question as it stood, not as it stands.** The floor's
window was measured **inert on the frozen tree** (whole-run min 0.055175 = 1.103× the floor,
no year below it, `non_collapsing` True sliced *and* whole) and **removed** — a strict
tightening; stationarity's `transient=2` **stays**, because its binding same-phase diff sits
at index 2 and is not dropped by the window anyway. ⇒ *"is `transient=2` right?"* is
dissolved for both halves. What is still the user's is the **different** question finding 6
sharpened: is a **deeper sow-in transient with a healthier attractor** a failure (frozen
1.103× the floor, stem-only 0.921×, settling **above** the control)? Stem-only's verdict is
unchanged (0.046065, fails inside *and* outside the removed window). Free corroboration: the
control's 0.073291 and 0.594984 reproduce `test_soil_fractionation.py`'s and
`test_decade_stability.py`'s independently-pinned values to the digit. ⚠ This is about
**stem-only, not (C)** — (C)'s full form stays refused on branch 3 and on LAI 16.40. **(D) —
NOT BUILT, and the verdict changed KIND: it is not refused ON CLOSURE, it is NOT BUILDABLE
AS RECORDED.** ⚠⚠ **Finding 1: the recorded price (“expect it to break closure in every
sealed scenario”) described a mechanism that COULD NOT HAVE FIRED.** It was written while
(A)/(B) were still live, i.e. against a tree whose litter held **0.004** C per N (~250 N per
carbon); a throttle reduces decay when N is *scarce relative to C*, so in that N-rich limit
**every such factor, whatever its form, sits at 1 — INERT**. That leg needs no curve, no
threshold and no citation, which is why the headline rests on it. Post-(A)+(B) the shed
ratio is a **cited parameter identity**, `M_C/n_residual_per_mol_c` = **exactly 90** ⇒ the
quantity a throttle would read went from *the ratio of two unrelated rate constants* to *the
composition of the material that fell in*. ⚠ **The “and now it would bite” half is
deliberately weaker and is NOT the headline** — it needs a threshold, and the ~25–30 C:N
figure usually quoted is **not on this shelf**; what Parton gives first-hand is *“uptake of
N from the soil does not occur if the C/N ratio is < 10 (Pinck 1950)”*, a **different
quantity** (immobilisation onset, not decay throttling), recorded with that caveat rather
than promoted. **Finding 2 — it would not bite uniformly**: measured with each scenario
driven the way its OWN golden drives it, the pool at peak `litter_n` is **102.7/98.9**
(shedding-fed) vs **10.9/9.8** (reset-driven, N-RICH — the dump's C:N is the dying plant's)
⇒ near-saturated right after each reset **in exactly the two scenarios where closure is
tightest**; the two-regime split, third appearance. **Finding 3 — neither decomposer primary
held first-hand carries an N throttle**, both established by **extraction, not skimming**:
**RothC mentions “nitrogen” TWICE in the whole guide, both in the BIBLIOGRAPHY** (a
carbon-only model — it can neither license nor refute a throttle), and **CENTURY keys decay
on LIGNIN and texture** (`K1 = Ks·exp(−3.0·Ls)`, eq [3]). ⚠⚠ **FINDING 4, THE STRUCTURAL
ONE: in the cited primary, (D) and SOIL FRACTIONATION ARE ONE MECHANISM.** CENTURY *does*
model “high-C:N residue decays slower” — as a **pool partition**, `FM = 0.85 − 0.018·(L/N)`
(eq [2], cited to Melillo 1984), which **is the input half of the fractionation seam this
project refused twice** (its re-opening measured both principled sizings failing on
`perennial`). So (D)-as-recorded is a **form the primary does not use**, and the form it
*does* use is **blocked by a measurement already in hand** — two independent reasons,
neither of them “we ran it and closure broke”. ⚠ The **keying quantity differs too (L/N, not
C:N)**, needing a lignin state the tree lacks — **and the CUE build had ALREADY written that
obstacle down** in `humification.yaml`'s own comments (*“ONE litter pool with no lignin
fraction”*): half of (D)'s blocker was recorded one build earlier and nothing routed
attention to it. ⚠ **FINDING 5 — A SELF-CORRECTION, and it is why the section is
trustworthy**: my first reading was that Parton carried the **OPPOSITE sign**, because it
says immobilised N *“can stimulate the decomposition of low-N plant residue”* — but **low-N
residue IS high-C:N residue**, so that sentence *states* (D)'s premise. **Refusing (D) on
“the primary contradicts it” would have been a refusal on a FALSE PREMISE**, caught before
it was written down; what differs is the FORM and the KEYING QUANTITY, which is a *stronger*
claim than “opposite sign”. ⚠ **NO INVENTED THROTTLE WAS RUN, deliberately** — fitting an
`f(C:N)` whose only target is our own goldens is the refused shape ⇒ the recorded closure
conflict is **neither confirmed nor discharged**, recorded as an **UNMEASURED leg** (the
`sealed_station` precedent), and retrieval is **EXHAUSTED FOR THIS SHELF, dated
2026-08-10**, never *“the science does not exist”* (the canopy regulator expired that
inference in one day). No value/golden/param/manifest moved; `git diff src/` empty; nothing
unfrozen.
