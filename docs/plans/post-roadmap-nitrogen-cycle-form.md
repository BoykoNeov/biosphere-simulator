# Post-roadmap — the nitrogen-cycle FORM gap

**Status: OPTION (A) COMPLETE (2026-07-27) — biosphere unfrozen + re-frozen, both ports.
Diagnosis advisor-reviewed at every hinge. `git diff src/simcore/` empty.
(B) immobilization, (C) the DS-dependent form and (D) the N→C throttle remain OPEN.**

**The one-line result: the form changed, the uncitable param is gone, and every carbon
trajectory is byte-identical** — 10 goldens moved and each moved *only* in its NITROGEN
stocks. See "THE IMPLEMENTATION" below, and `docs/biosphere-reference.md`'s unfreeze log
for the canonical record.

⚠ **The quote below is the premise this work started from, and the build falsified two of
its three clauses**: `mineralization_rate` is no longer behaviorally inert (it now sets
litter pool C:N) and N is no longer uncoupled from C. Only "wrong pool" survives. Kept
verbatim because the correction is the finding.

The chosen successor to the scope-(B) decomposer calibration
(`post-roadmap-decomposer-calibration.md`), which closed the **carbon** return side and
left the **nitrogen** side recorded as a *form* gap rather than a rate gap:

> `mineralization_rate` **investigated, NOT moved** — its cited range is the WRONG POOL
> (soil-N₀ vs our fresh-residue N), the N scale is non-physical (kg N uncoupled from
> mol C; `plant_n0` set high to force `f_N≡1`), and the rate is behaviorally **inert**
> […] the real gap is a missing immobilization **FORM** (a documented deferred seam),
> recorded not fitted.

Two frozen params carry a form gap, each with a move already named in the tree:

| param | value | gap as recorded | named move |
|---|---|---|---|
| `mineralization_rate` | 0.03 /day | positive-only; cannot immobilize | `litter_n → microbial_n → soil_n`, C:N-driven |
| `n_senescence_rate` | 0.01 /day | bare constant; retrieval **EXHAUSTED**, no 1/day source exists | adopt the DS-dependent form [B] actually gives |

## The diagnosis phase (read-only probes; no frozen file touched)

Probes in `M:/claud_projects/temp/ncycle/`, mirroring the decomposer calibration's
"measure before deciding" discipline. **Four results, and three of them falsified a
premise this plan started with.**

### Finding 1 — `plant_n0` is NOT what forces `f_N ≡ 1`. Uptake is.

The recorded diagnosis says `plant_n0` "set high to force `f_N≡1`". Measured, that is
**not the mechanism**. Re-scaling `plant_n0` from the frozen 0.5 kg down to a physical
seedling N (2 % and 4 % of sowing DM) leaves the run at **`f_N_min = 1`**, and all three
runs converge to the *same* `final_plant_n = 0.15 kg`.

The equilibrium is arithmetic, and it is exact:

```
plant_n*  =  max_uptake_capacity / n_senescence_rate  =  0.0015 / 0.01  =  0.15 kg
```

against the observed `0.150036`. **`plant_n`'s steady state is the ratio of two
uncited constants** — a capacity flagged as ~6× realistic peak uptake, over a rate for
which "retrieval is exhausted". That, not the IC, is why a 52 g plant holds 0.15 kg N.

### Finding 2 — in the sealed chamber, `f_N` cannot bite at any physically-defensible SOIL-N scale. Not a param artifact.

⚠ **Read the scope in that heading literally.** This was measured on **one** of the seven
frozen scenarios (`sealed_chamber`), with uptake **on**, varying the **soil** side. It is
*not* the claim "`f_N` cannot bite" — `n_limited` bites in the frozen tree at min **0.176**
(187/306 steps), and finding 5 below shows the **plant** side is where the real margin
lives. An earlier draft of this section, of the `CLAUDE.md` row and of the memory hook all
stated it flat; that is the project meta-finding's **11th** instance, with the
counterexample already recorded in the same file (advisor catch).

Moving the whole N sub-system together (`soil_n0`, the availability band,
`max_uptake_capacity`, `plant_n0`) to physical per-m² values, and then **starving** it to
5× the plant's whole-season demand:

| | frozen | physical | physical + real uptake | starved |
|---|---|---|---|---|
| `soil_n0` (kg N/m²) | 100.0 | 0.30 | 0.05 | 0.004 |
| availability min | 1 | 0.849 | 0.176 | 0.0026 |
| **`f_N_min`** | **1** | **1** | **1** | **1** |
| `final_storage_c` | 0.442874 | 0.442874 | 0.442874 | 0.442874 |
| `final_carbon_pool` | 2.357 | 2.357 | 2.357 | 2.357 |

**The carbon trajectory is byte-identical in every case.** The reason is structural, not
parametric: peak biomass is **1.9612 mol C = 52 g DM/m²**, so whole-season N demand at
the model's own critical concentration is **0.000785 kg** — which a per-area supply
covers even at availability 0.0026.

And that small plant is **not** a param error: implied SLA is **22 m² kg⁻¹ leaf DM**
(real wheat 20–25), peak LAI **0.51**. The sealed chamber is *carbon*-limited by design
(O₂ depletes ~99 %, carbon recycles through a small pool), so its plant is ~25× smaller
per m² than the field crop the N params are sized for. A field-sized soil-N pool is
therefore vastly in surplus for it.

⚠ **Consequence for scope**: making the N scale physical is **safe and additive**
(carbon-invariant, measured) — and, on its own, **behaviorally inert**. So is
immobilization. Neither becomes load-bearing through `f_N` in this chamber.

### Finding 3 — the coupled form lands litter C:N in the real range; the frozen form is 1–4 orders out.

Because Finding 2 establishes the N side has **zero** feedback on carbon, integrating the
coupled N equations on the **recorded** carbon trajectory is *exact*, not indicative —
it reproduces what the in-tree coupled model would produce. Measured litter C:N (mass):

| N-shedding form | litter C:N (final / last-year range) | real wheat straw |
|---|---|---|
| **frozen** `k_sen · plant_n`, in-run | **0.004** (≈ 1 C : 246 N) | ~80 : 1 |
| frozen, offline (no uptake refill) | 8.02 | ~80 : 1 |
| **coupled** @ 3.0 % tissue N | **77.5** (77–107) | ~80 : 1 |
| coupled @ 1.5 % tissue N (= the model's own `n_critical`) | 154.9 (155–214) | ~80 : 1 |
| coupled @ 0.6 % tissue N (mature straw) | 387 (387–536) | ~80 : 1 |

The **proportionality** is exact — every row is the same multiple of
`(tissue C:N) × (k_min/k_decomp)` — and that is what the finding rests on. ⚠ **But the
multiple itself (1.894 here) is NOT a constant of the model**: it is this scenario at this
horizon. Rows vary only in tissue N, so they share it by construction; measured across
scenarios and horizons the end-of-run multiple spans **0.855 → 48.4**, because litter input
is a pulse and the end-of-season pool is a drained tail. The implementation section's
correction note has the measurement. Read the law as a proportionality, never with a number
attached:

```
litter C:N  ∝  (tissue C:N) × (mineralization_rate / decomposition_rate)
```

⚠ **This is why the C:N is non-physical, stated precisely**: it is not that the *numbers*
are wrong, it is that **nothing in the frozen model ties N shed to C shed**, so litter
C:N is the emergent ratio of two independent first-order rates — a quantity with no
physical constraint on it. `mineralization.py` says this in as many words ("the litter
C:N is instead **emergent** from two independent first-order rates"), as a *deliberate*
JIT-minimal Step-6 choice with the coupled form named as the seam. The diagnosis is that
the choice has now been measured, and it costs 1–4 orders of magnitude.

### Finding 4 — the payoff is a CITABLE param replacing an uncitable one.

The coupled form's parameter is a **tissue N concentration (% DM)**, not a relative rate
(1/day). That is the shape the accessible literature actually publishes — and the shape
this repo *already* carries cited values in (`nitrogen.yaml`'s `n_critical` = 1.5 % DM;
`carbon_fraction` = 0.45, the project's "single strongest bind", first-hand to [B]).

So the form change **discharges** what `mineralization.yaml` calls "the highest
clean-room risk in the project":

> `n_senescence_rate` — NO ACCESSIBLE PRIMARY SUPPORT AT ALL […] The accessible
> N-resorption literature is built around resorption EFFICIENCY (a dimensionless %,
> ~50 % typical), not a relative rate (1/day) — the wrong SHAPE to bind a first-order
> constant.

⚠ And it does so **without a citation hunt**, which is the point: retrieval was declared
exhausted precisely because no source has the 1/day shape. Changing the *form* changes
what shape is needed. Resorption efficiency — the quantity the literature *does* publish
— is directly expressible in the coupled form (shed a fraction `1 − eff` of the tissue N
at senescence, retain the rest), so the ~50 % figure becomes usable where it was not.

### Finding 5 — the margin holding `f_N` at 1 is on the PLANT side, and (A) removes it

The advisor connected findings 1 and 2 and predicted the plan's "(A) is carbon-invariant"
footnote was wrong. The arithmetic:

| | value |
|---|---|
| whole-season N demand at `n_critical` | 0.000785 kg |
| frozen `plant_n` **equilibrium** (`uptake/k_sen`, finding 1) | 0.15 kg → **191×** critical |
| frozen `plant_n` **initial condition** (`plant_n0`) | 0.50 kg → **637×** critical |

Finding 2 varied the **soil** side while the frozen plant dynamics pinned `plant_n` at
0.15 kg *regardless* of it. Option (A) removes exactly that mechanism: demand-based uptake
drives the tissue concentration to a **target**, and coupled shedding removes the
`uptake/k_sen` brake. The margin collapses from ~191× to ~1×, so invariance is no longer
inherited — it is decided by where the target sits relative to `n_critical` = 1.5 % DM.
**So invariance had to be measured, not footnoted.**

### Finding 6 — invariance MEASURED: it holds for a flat coherent target, in the 7 scenarios probed; the target FORM is load-bearing

> ⚠ **SCOPE CORRECTION (finding 9): "all 7 scenarios" below is NOT the frozen seven.**
> The set probed here — sealed / perennial / consumer / `n_limited` / `water_biting` /
> `day_neutral` / `drought` — overlaps the manifest's frozen seven in **3 rows**. It omits
> `open_season`, `perennial_long_horizon`, `consumer_long_horizon` and `drift_summary`, and
> includes four scenarios that are not frozen at all. `open_season` is the omission that
> mattered: it is the only frozen scenario that reaches field scale, and therefore the only
> one that lands **inside** Greenwood's stated domain of validity. The *conclusion* survives
> finding 9 and comes out stronger — which is exactly what makes this easy to soft-pedal, so
> it is corrected here rather than footnoted. Meta-finding, **12th instance, mine**: the tell
> is that **the numeral matched by coincidence** — "7 scenarios" and "the frozen 7" were
> different sevens, and the number hid the gap. Check a scenario list against the *manifest*,
> never against its own length.

`probe_fn_under_A.py` / `probe_fn_all_scenarios.py` integrate the (A) plant-N ODE
(demand-deficit uptake + N:C-coupled shedding, ± 50 % resorption) on the **recorded**
carbon trajectory and evaluate `f_N` at every step. Finding 2's zero-feedback result is
what makes this **exact**: `f_N ≡ 1` ⇒ carbon unchanged ⇒ the recorded trajectory *is* the
(A) trajectory (a fixed point, not a screen).

`f_N` minimum under (A), per scenario:

| scenario | frozen | (A) flat 3.0 % DM | (A) dilution 4.5→1.2 % | (A) flat 1.5 % (= `n_critical`) |
|---|---|---|---|---|
| `sealed_chamber` | 1.0 | **1.0** | 1.0 | 0.828 |
| `perennial_chamber` | 1.0 | **1.0** | 1.0 | 0.828 |
| `consumer_chamber` | 1.0 | **1.0** | 1.0 | 0.754 |
| `water_biting` | 1.0 | **1.0** | 1.0 | 0.842 |
| `drought` | 1.0 | **1.0** | 1.0 | 0.808 |
| `day_neutral` | 1.0 | **1.0** | **0.713** ⚠ | 0.893 |
| `n_limited` (uptake **off**) | 0.176 | 0.0 | 0.0 | 0.0 |

Four results:

1. **At a flat target ≥ ~2× critical (3.0 % DM), (A) is carbon-invariant in every
   scenario** — `f_N ≡ 1.0`, zero steps below 1, and uptake capacity is **never** binding
   (0 capped steps: the 52 g plant's demand is met every step). Invariance is *proven*,
   not assumed.
2. **A target at `n_critical` itself bites everywhere** (0.75–0.89). That is not a
   surprise but a tautology: "critical" *means* "below this you are stressed", so
   targeting it is targeting your own stress threshold. The dip's size is a one-step lag
   against ~11 %/day relative growth at DVS 0.32 — **real, not a probe artifact**, since
   in-tree flows also evaluate at the step-entry snapshot.
3. **The advisor's declining-dilution-curve prediction is half right, and the half that
   fails is instructive.** A Greenwood-style 4.5 → 1.2 % DM target crosses below
   `n_critical` late season, yet `f_N` stays 1.0 in the winter-wheat scenarios — because
   **`f_N`'s denominator collapses**: `leaf+stem+root` falls from 1.9612 to 0.0157 mol C
   (**×0.008**) as carbon translocates to storage and senesces, while coupled shedding
   only removes N in proportion to *senescence* carbon. Concentration therefore **rises**
   late season instead of falling. Robust to the demand denominator (whole-plant incl.
   storage gives the identical numbers) and to 50 % resorption.
4. ⚠ **But it DOES bite in `day_neutral` (0.713)** — the one scenario whose phenology is
   not vernalization/photoperiod-gated, so DVS advances (and the target declines) while
   vegetative biomass is still growing. **So the target's functional form is a
   load-bearing scientific choice, not a detail**, and the answer is scenario-dependent.

⚠ **`n_limited` preserves the biting regime but not its magnitude** (0.176 → 0.0): with
uptake off, coupled shedding is concentration-*neutral*, so dilution by growth drives
conc → 0 unopposed. The regime pin holds; the value moves a lot. It is not one of the
7 frozen scenarios, but it does have a golden.

⚠ **"Carbon-invariant" ≠ "no golden moves".** The goldens record N stocks, so `plant_n` /
`litter_n` / `soil_n` values change under (A) in every case. The cascade is the same size
either way; what the target choice decides is whether the **science** (carbon) changes too.

### Finding 7 — `f_N` is STRUCTURALLY the only N→C channel (the fixed point is a proof, not evidence)

The exactness argument needs `f_N` to be the *only* path by which an N stock can affect a
carbon leg. Finding 2 established that empirically (carbon byte-identical over four soil
scalings); it is also structural. Every read of an N stock in the entire biosphere:

| site | flow | legs |
|---|---|---|
| `carbon_budget.py:206` | `Photosynthesis` (`f_N`) | **carbon** ← the only channel |
| `nitrogen.py:166` | `NitrogenUptake` | N only |
| `mineralization.py:139` | `NitrogenSenescence` | N only |
| `mineralization.py:176` | `Mineralization` | N only |

So `f_N ≡ 1` ⇒ **no** N value can reach a carbon leg ⇒ the recorded carbon trajectory is
unchanged ⇒ the trajectory the probe integrated on *is* the (A) trajectory. Proof by fixed
point, on four grep hits.

### Finding 8 — two probe→in-tree gaps, and the reset one is a THIRD documented seam

1. **The annual reset is carbon-only, and the frozen tree says so with the condition
   attached.** `season.py`'s `annual_reset` docstring: "**Carbon-only** — `plant_n`
   persists across the death (an N *windfall* for the small seedling), **harmless only
   while `f_N ≡ 1`**; a full N-reset is a deferred refinement." Two consequences:
   * The probe *re-seeded* `plant_n = target × biomass` at each re-sow, which is the
     **harsher** assumption (conc = target exactly, vs the in-tree windfall's conc ≫
     target). Invariance therefore holds *a fortiori* in-tree — the probe did not flatter
     itself here.
   * ⚠ But (A) cannot inherit the windfall: coupled shedding means death dumps `plant_n`
     **to `litter_n`**, and the seedling's N must then come *from* a stock (seed bank /
     boundary) because the conservation gate asserts every step. That is unbuilt work (A)'s
     estimate must carry, it is a **third** deferred seam the frozen tree already names,
     and it sets the re-sown crop's starting C:N — the very quantity finding 3 measures.
2. **Uptake availability was read from the frozen `soil_n` trajectory**, which (A) would
   evolve differently (less draw, different mineralization inflow). Benign here for a
   stated reason, not by luck: availability **saturates at 1.0** in every frozen scenario
   (`soil_n0 = 100` vs `sn_critical = 50`), and `n_limited` runs with uptake **off**. This
   is the one place the offline integration is not self-contained.

### Finding 9 — the target form is decided by READING Greenwood, and the answer is neither of the two forks

The user's criterion — *"I want what represents reality more faithfully"* — turned the
fork from a preference into a **retrieval question**, and `sources/greenwood1990.pdf` was
**already on the shelf**. Both candidate forms were invented; the primary gives a third.

Read first-hand (Greenwood et al. 1990, Ann. Bot. 66:425–436):

* eqn (6): **`%N = a·W^-b` for `W > 1.0 t ha⁻¹`**, with **`a = 5.697` (C3)**, **`b = 0.5`**.
  (The abstract rounds `a` to 5.7; eqn (6) prints 5.697.)
* *"Data obtained with W less than 1 t ha⁻¹ were **always omitted**"* — below 1 t/ha the
  paper deliberately has **no data**.
* **The paper says what happens there, with a mechanism and a citation**: *"The value of `a`
  is the %N in the crop when W = 1 t ha⁻¹. At this weight the growth rate gradually changes
  from being almost exponential to linear. **When growth is exponential plant %N remains
  constant and the critical concentration does not change with increase in plant mass**
  (Ågren, 1985). The value of a = 5.7 % is therefore the best estimate of %N needed in the
  dry matter of **young tissue** to permit the maximum growth rate of C3 crops."*
* *"Both %N and W refer to the dry matter in the whole plant (**excluding fibrous roots**)"*
  ⇒ Greenwood's `W` is `leaf + stem + storage`, **not** `f_N`'s own denominator
  (`leaf + stem + root`).

So the faithful form is **piecewise**: constant **5.697 % DM** below 1 t/ha, `5.697·W^-0.5`
above. The plateau is **the primary's own statement**, not our interpolation — which means
the "flat vs declining" fork was a false dichotomy. Measured on the **manifest's** scenario
set (`probe_greenwood_primary.py`), `f_N` minimum under (A):

| scenario | frozen? | peak `W` (t/ha) | in domain | **GW primary** | GW *incl. roots* | DVS-extrapolated |
|---|---|---|---|---|---|---|
| `open_season` | ✅ | **12.633** | **YES** (89 d) | **1.0000** | 0.9750 ⚠ | 1.0000 |
| `sealed_chamber` | ✅ | 0.351 | no | **1.0000** | 1.0000 | 1.0000 |
| `perennial_chamber` | ✅ | 0.364 | no | **1.0000** | 1.0000 | 1.0000 |
| `perennial_long_horizon` | ✅ | 0.364 | no | **1.0000** | 1.0000 | 1.0000 |
| `consumer_chamber` | ✅ | 0.430 | no | **1.0000** | 1.0000 | 1.0000 |
| `consumer_long_horizon` | ✅ | 0.430 | no | **1.0000** | 1.0000 | 1.0000 |
| `water_biting` | golden | 0.441 | no | **1.0000** | 1.0000 | 1.0000 |
| `day_neutral` | authored | 0.093 | no | **1.0000** | 1.0000 | **0.713** ⚠ |
| `n_limited` | golden | 0.071 | no | 0.0 | 0.0 | 0.0 |

Three consequences:

1. **`day_neutral`'s 0.713 was an artifact of extrapolating the curve outside its stated
   domain.** `day_neutral` peaks at 0.093 t/ha — an order of magnitude *below* the bound —
   so the declining ramp I was about to recommend as "the literature-shaped form" was using
   Greenwood exactly where Greenwood says he has no data. **The form I called faithful was
   the least faithful of the three.**
2. **Invariance comes out as a consequence of fidelity, not as a reason for choosing** — which
   dissolves the backfitting trap of open question 2 outright. Nothing was picked *because*
   carbon stayed byte-identical; the paper was read and invariance fell out.
3. ⚠ **But do not oversell that** (advisor). It holds on a **12 % margin against a
   denominator definition**, not a robust separation. Greenwood crosses `n_critical` at
   `W = (5.697/1.5)² = ` **14.42 t/ha**; `open_season` peaks at **12.633** — 88 % of the way
   there. And the *wrong* denominator (`W` including fibrous roots, which is `f_N`'s own
   denominator and therefore the tempting choice) **does bite**: 0.9750, 3 steps, in
   `open_season`. Two forms both defensible as "Greenwood", one moves a frozen golden. Both
   facts belong in the record, adjacent, not merged into "fidelity produced invariance".
   ⇒ **the 14.42 t/ha crossing point is a PIN, not a footnote** (the `test_oracle_gap.py`
   precedent): a weather-fixture change, a canopy improvement, or any calibration that grows
   the open-field crop ~15 % flips `open_season`.

### Finding 10 — (A) DELETES the knob `n_limited` is built on

The advisor blocked on `n_limited` → exactly 0.0. Resolved, and it is a **design requirement
on (A)**, discovered before building rather than after.

⚠ **First, the number is not licensed.** Finding 2's zero-feedback license holds *only where
`f_N ≡ 1`*. In `n_limited`, `f_N` already bites in the frozen run, so the recorded carbon
trajectory is **not** the (A) trajectory and every `n_limited` cell in findings 6 and 9 is a
**screen, not a measurement**. It is a screen with a known **sign** of error: `f_N` throttles
assimilation ⇒ less growth ⇒ less growth-dilution ⇒ higher concentration, a *negative*
feedback the open-loop probe suppresses by construction. So 0.0 is an **upper bound on the
severity**. *(A crude closed-loop screen still reaches 0 and drives the plant to zero carbon
— direction only; scaling net organ increments by `f_N` is not the model.)*

⚠ **Second — the probe's own baseline was wrong, and the golden caught it.** An earlier
version reconstructed the frozen `plant_n` as `plant_n0·(1−k_sen)ⁿ` and reported `f_N` min
0.0000 / 305 steps against the recorded **0.176 / 186**. The disagreement is the tell that
the *reconstruction* was wrong, not the run; reading the recorded stock reproduces
0.1759 / 186 / final 0.3754 exactly. **Reconstruct a frozen quantity only to check it against
the recorded one — never to replace it.**

**What the screen does establish, and it is structural:**

* the frozen scenario **recovers** (min 0.176, final 0.3754) — it is a *modulation* scenario;
* under (A) `f_N` hits 0 at step 195 and is **absorbing** (max after = 0.0), tissue
  concentration falling 5.463 % → 0.111 % DM;
* **the cause is that (A) makes `plant_n0` inert.** (A) seeds `plant_n` from the tissue-N
  *target*, so `N_LIMITED_SCENARIO`'s `plant_n0 = 6e-5` — a *tiny reserve* deliberately placed
  inside the `f_N` band — is simply overwritten by the Greenwood plateau (5.46 %). The
  scenario's stated purpose ("owns the `f_N` concentration ramp + the uptake-shutoff path")
  is not preserved by accident.

⇒ **(A) must expose a seedling-N knob** (seedling N as a fraction of target, or an explicit
initial concentration) for `n_limited` to keep testing what it was built to test. Re-sizing a
**non-frozen, non-manifest** scenario's IC to preserve its documented purpose under a new form
is legitimate scenario authoring, not backfitting — but it is *work (A) owes*, alongside
finding 8's re-sow seam.

### Finding 11 — the criterion reaches further than the available work

Applied consistently, "represents reality more faithfully" does not stop at (A): it reaches
**(D)**, the N→C throttle, because real soils *do* decay high-C:N residue more slowly. (D)
collides head-on with the decomposer calibration's **measured** requirement for the fast decay
edge, so the honest statement to make **before** building, not after:

> The faithful sub-chain that is *available* is (A) [+ (B)]. The fully faithful chain includes
> (D), and (D) is measured to fight closure in every sealed scenario. The obstacle is that the
> chamber is **carbon**-limited by design — a 52 g/m² plant against field-sized N params — not
> the N form. **Making N faithful does not make the chamber faithful.**

## THE IMPLEMENTATION — option (A), built 2026-07-27 (user: "go with it")

Scope as authorized: **(A) only** — N:C-coupled shedding **+** demand-deficit uptake, with the
target read off Greenwood. (B) immobilization, (C) the DS-dependent form and (D) the N→C
throttle stay open. The unfreeze followed `docs/biosphere-reference.md`'s discipline; that
file's unfreeze log carries the canonical record.

### What shipped

| | before | after |
|---|---|---|
| shedding | `n_senescence_rate · plant_n` (uncitable 1/day rate) | `min(tissue_conc, n_residual) · shed_C`, driven by `Senescence`'s own flux |
| uptake | `capacity · availability` (ignored demand) | `min(target·biomass − plant_n, capacity·availability)` |
| target | *(none)* | Greenwood eqn (6), piecewise, 3 cited params |
| `annual_reset` | carbon-only (N *windfall*) | N reset at the parent's concentration, balancing residual to litter |
| `plant_n0` | 0.5 kg (2055× target) | 2.43e-4 kg (the seedling at target) |
| params | 1 uncitable + 4 | **0 uncitable** + 7 |

### The results, measured

1. **Carbon invariance CONFIRMED IN-TREE, at golden level.** 10 goldens moved and **every one
   moved only in its NITROGEN stocks**; every carbon amount is byte-identical, and
   `drift_summary.json` (a carbon-side stability signature) regenerated **byte-unchanged**.
   `f_N ≡ 1.0000` with zero steps below 1 in all seven frozen scenarios. Finding 9's offline
   prediction held exactly — which is what the zero-feedback license promised.
2. **`n_limited` is byte-identical, for a reason finding 10 missed.** It is **open-field**, so
   `NitrogenSenescence` (sealed-only) is never built and the shedding form cannot reach it;
   `plant_n` is constant and `f_N` falls purely by growth dilution, exactly as before —
   min **0.1759**, 187/306 steps, unchanged. ⚠ **This falsifies finding 10's premise.** That
   finding said "(A) deletes the knob `n_limited` is built on" because the probe *re-seeded*
   `plant_n` from the target each cycle; **in-tree `plant_n0` is still the initial condition**,
   so the tiny reserve survives. No seedling-N knob is owed, and the pin needed no change.
   The lesson: **a probe's convenience choice can look like a property of the design.**
3. **The litter C:N deliverable splits in two, and the split is the finding.**
   * the **shed material** is straw-like: C:N = `carbon_fraction / n_residual` = **90** (both
     terms cited), against wheat straw's ~80. This is the quantity the form change was for.
   * the **litter pool** is **two regimes, not one number** (see correction 2 below): a
     *shedding-fed* chamber sits at **173–192** because mineralization drains N 2.7× faster
     than decomposition drains C, while a *reset-driven* chamber reads **~10**, dominated by
     the annual dump of N-rich dying tissue. From **0.004** to either is orders better; the
     residual is now attributable to *named mechanisms*, not to the form.

   ⚠ **CORRECTED 2026-07-27 (advisor catch, then measurement): the litter POOL C:N figures first recorded here were WRONG, and the error was the meta-finding's shape again — a number fitted to ONE scenario at ONE horizon, written as a law.** The original text asserted `pool C:N ≈ (shed C:N)·(k_min/k_decomp)·**1.894**` ≈ 465, with 1.894 called a "measured geometry factor". It was fitted to `sealed_chamber`'s **final** state after 3 years, and the end-of-run value is horizon-dependent across more than an order of magnitude: **210** (1 yr, `water_biting`), **465** (3 yr, `sealed_chamber`), **9076** and **11877** (5 yr, `perennial`/`consumer`). My first explanation for the outliers — seeded `litter_carbon0` with no N counterpart — was **also false**: all four scenarios seed `litter_carbon0 = 3.0`. **The real mechanism:** litter input is a *pulse* (the annual dump), not a continuous feed, and between pulses both currencies drain — carbon with a ~63-day half-life, nitrogen with a ~23-day one — so the end-of-season snapshot is a **tail**, and by year 5 it is the ratio of two vanishing numbers (`litter_n` = **1.3e-11 kg**). Quoting that as "the litter C:N" is quoting numerical dust. **What is actually true, measured at peak `litter_n` across all four sealed scenarios: pool C:N = 173–192**, a tight band sitting *below* the quasi-steady law `90 × 2.727 = 245.5` (0.71–0.78× it) because the pulsed pool never converges upward. That is **~2.2× wheat straw's ~80**, not ~5×. **And the scope-B projection shrinks with it**: applying Stanford & Smith's 39-soil range to the measured relationship gives **31–83 (pooled mean 47)**, not the "78–211, mean 119" this row first carried — every cited value lands at or below real residue, against **~184** for our uncited 0.03/day. **The DIRECTION is unchanged and is the point** (two independent lines still say `mineralization_rate` is too fast); only the magnitude was inflated. Pinned now — including an explicit anti-regression assertion that the end-of-run ratio spans >10× with horizon, so no constant factor may be written down again — in `tests/test_nitrogen_form.py`.

   ⚠⚠ **CORRECTION 2 (2026-07-27, found while scoping option (B)): CORRECTION 1'S OWN REPLACEMENT BAND WAS MEASURED ON A MIS-DRIVEN SCENARIO SET, AND THE ANTI-REGRESSION PIN IT LEFT BEHIND WAS CERTIFYING A DRIVER ARTEFACT.** Correction 1 drove **all four** sealed scenarios through `run_season`. But `PERENNIAL_CHAMBER_SCENARIO` and `CONSUMER_CHAMBER_SCENARIO` are driven by **`run_perennial`** in their own regression goldens — **the annual reset is what makes them perennial** — and dropping it changes the answer by an order of magnitude. Measured under each golden's own driver:

   | scenario | pool C:N @peak `litter_n` | fraction of the law | regime |
   |---|---|---|---|
   | `sealed_chamber` | 191.78 | 0.781 | shedding-fed |
   | `water_biting` | 173.37 | 0.706 | shedding-fed |
   | `perennial` | **10.91** | 0.044 | reset-dump-dominated |
   | `consumer` | **9.87** | 0.040 | reset-dump-dominated |

   So the true spread is **19.4×, not the 1.11× correction 1 measured**, and **both** assertions correction 1 left behind were driver artefacts: `max(peak_ratios)/min(peak_ratios) < 1.2` (truth: 19.4×) and `max(final_ratios)/min(final_ratios) > 10.0` (truth: **2.21×**). They are **RETIRED, not re-tuned** — a widened bound preserves the shape of a claim that is gone. ⚠ Note the shape: **the pin that existed *specifically* to stop a constant factor being written down was itself resting on a mis-driven run.** Two further sentences are **WITHDRAWN**: the end-of-run 5-year figures "**9076** and **11877**" are actually **242.9** and **235.2**; and "by year 5 `litter_n` is **1.3e-11 kg** … quoting numerical dust" is false for the actual perennial chamber, whose final `litter_n` is **6.05e-05 kg** — six orders larger, because the reset that was dropped is what refills it every year. **THE MECHANISM, which is why these are two quantities and not one wide band: "peak `litter_n`" silently names two different events.** In a shedding-fed chamber it is the seasonal senescence maximum; in a reset-driven one it is the **annual dump** (measured at step 611 of 1525 for `perennial` — one step past the year-2 boundary at 610), depositing the dying plant's whole retained N at its own elevated concentration, C:N **5.6–6.1**. That elevated concentration is *this work's own recorded limitation 5* (shedding at residual leaves a senescing plant holding its N while the biomass denominator collapses) — so limitation 5 turns out to be the thing that sets the reset-driven pool's C:N, not a harmless footnote. **The scope-B projection SURVIVES with its magnitude unmoved and its scope narrowed**: `observed_fraction = 0.75` was only ever entitled to the **shedding-fed** regime (a `k_min/k_decomp` relationship means nothing for a pool whose C:N comes from the dying plant), and correctly driven those two give 0.781 / 0.706 — so 31–83, mean 47, all stand. **A number can be right while the sentence justifying it is wrong.** Re-pinned in `tests/test_nitrogen_form.py::test_litter_pool_cn_is_TWO_regimes_set_by_which_event_fills_the_pool`, which now drives each scenario the way its golden does, asserts the two regimes separately, asserts the peak lands one step past a year boundary (the mechanism, not the number), and carries the **inverted** anti-regression pin (spread > 10×) so correction 1's band cannot be restored. **Sweep done, not assumed**: every other test that *runs* a perennial scenario (`test_biosphere_stress`, `test_compartment_ledger`, `test_consumer`, `test_decade_stability`) already uses `run_perennial`; `test_compartments`/`test_builders` only build. `test_nitrogen_form.py` was the sole site. Test-and-docs only — **no `src/` change, no golden moved, nothing unfrozen**.
4. ⚠ **Two of the three reasons for not recalibrating `mineralization_rate` are now FALSE.**
   The decomposer calibration declined to move it because (1) wrong pool, (2) N/C uncoupled so
   litter C:N is not physical, (3) behaviorally inert. (2) and (3) are dead: N and C *are*
   coupled now, and the rate *does* set an observable. Only (1) survives. And the numbers
   converge from two independent directions: Stanford & Smith's 39-soil range
   (0.005–0.0136/day) puts the pool at **31–83** and their pooled mean at **47** — at or below
   real residue — while our uncited 0.03/day gives **~184**. **Value UNMOVED** (scope B, a user
   decision); the consequence is pinned in `tests/test_nitrogen_form.py` instead.
5. ⚠ **A recorded limitation: the one-pool model shows through.** Shedding at the residual
   concentration means a senescing plant *retains* most of its N while its denominator
   collapses, so tissue concentration rises without bound as biomass → 0 — ~110× target in the
   3-year chamber, ~**6e6×** in the 5-year perennial. Harmless for carbon (`f_N` saturates) and
   N conserved exactly, but real remobilized N goes to **grain** and we have one whole-plant
   pool. Related deferred seam, now named: the chambers seed `litter_carbon0` with **no
   `litter_n0` counterpart**, which inflates their pool C:N further (and explains why perennial
   and consumer read far above the sealed chamber's 465).
6. **The `f_N ≡ 1` margin fell ~2.5 orders while the conclusion held** — 1000× (the old
   `uptake/k_sen` equilibrium) → 3.8× on the plateau → **~1.07×** at `open_season`'s peak. The
   crossing is 14.42 t/ha vs a 12.633 peak. `mineralization.py`'s "~1000× above critical"
   sentence was updated rather than left to rot.

### Tests

`tests/test_nitrogen_form.py` (12 pins): the curve's plateau/decline/continuity and its param
values; the **14.42 t/ha crossing** and `open_season`'s 88 % margin; that only `open_season`
enters the declining branch; that the shed-N flow's recomputed carbon flux equals
`Senescence`'s own litter leg (the drift hazard of recomputation); shed C:N 90; pool C:N against
the predicted formula to 2 %; and N conservation across `annual_reset` to 1e-12.

Rewritten rather than weakened, with the reason recorded in each: `test_mineralization.py`'s
rate-law tests (now coupling tests, both `min` branches), and — the one that mattered —
`test_sealed_plant_n_is_drained`, whose "plant_n declines over the season" was **true only
because of the absurd IC**; a growing crop accumulating N is correct, so it now pins the
withdrawal directly plus the target-as-floor invariant. In `test_nitrogen.py` the default
fixture held `plant_n0 = 0.2` kg, which under demand-deficit is 130× target ⇒ zero deficit ⇒
**every uptake test would have passed vacuously on zero legs**; the default is now an N-starved
plant and the demand-limited branch got its own tests.

## THE (B) DIAGNOSIS — measured 2026-07-27, **BUILT 2026-07-27** (see the build section below)

Read-only probes in `M:/claud_projects/temp/ncycle_b/`, the (A) discipline repeated. **The
headline is that (B) cannot deliver the thing it is named for, and the reason is measured.**

### B-finding 1 — CUE is 1.0 in this tree, which CONSTRAINS the form before any choice is made

The canonical immobilization treatment needs a carbon-use efficiency: the fraction of
decomposed litter C assimilated into microbes rather than respired. Ours is **1.0** —
`Decomposition` moves *100 %* of decayed litter C into `microbial_carbon`, and respiration is
a **separate** first-order draw on that pool (`decomposition.py` / `microbial_respiration.py`,
by the deliberate Step-4/Step-5 split). ⇒ **introducing a literature CUE (~0.3–0.4) would move
CARBON**, re-opening the decomposer calibration under its measured fast-edge closure
requirement — i.e. it would cost like **(D)**, not like (B).

⇒ **Design requirement, not a preference: (B)'s N legs must be computed off the carbon
partition `Decomposition` and `MicrobialRespiration` ALREADY apply**, never off a second,
independently-parameterised CUE. That is (A)'s recomputation-drift hazard one flow over (the
shed-C == `Senescence`'s-litter-leg pin) and needs the same pin.

### B-finding 2 — ⚠ the pool-identity objection BITES, so the C:N-driven mineral-N draw is UNAVAILABLE

The textbook form imposes a homeostatic microbial C:N (~8; CENTURY/RothC active pool) and
draws the shortfall from mineral N. That is only legitimate if our `microbial_carbon` *means*
what those models' pool means. Measured, it does not:

| | `microbial_carbon` / `litter_carbon` | real standing microbial biomass |
|---|---|---|
| peak / mean / final | up to 40 / 0.73–6.16 / — | a few **%** of litter C |

`microbial_carbon` peaks at ~0.95–1.01 mol C against a litter pool of ~3.0 — **comparable to
litter, not a few percent of it**. It is a *transit* pool holding carbon a real model would
already have respired (finding 1's CUE = 1.0 is exactly why). Imposing C:N = 8 on it would
demand **90–152× the litter N present** and hold an N stock inflated by the same pool
mismatch.

⇒ **Refused, for the reason this project has refused twice before** — `decomposition.yaml`'s
DPM/RPM labile-fraction re-read and `mineralization.yaml`'s soil-N₀-vs-litter-N re-anchoring:
*redefining what a pool MEANS so a literature constant fits is a semantic model change wearing
a provenance hat.* So **(B) does not deliver immobilization**, and the option's name in the
table below is wrong and is corrected there. What it delivers is **microbe-mediated N transit,
stoichiometric with carbon** — and the immobilization seam stays open with a **measured
obstacle** instead of a deferral, which is a stronger record than a half-built mechanism.

### B-finding 3 — the available form, and the identity that retires `mineralization_rate`

    litter_n  --> microbial_n     moved = decomposed_C * (litter_n / litter_C)
    microbial_n --> soil_n        moved = respired_C   * (microbial_n / microbial_C)

replacing the direct `Mineralization` (`litter_n → soil_n` at the free `mineralization_rate`).
Both recompute their carbon sibling's flux from the same params object — `f_O2` included on
the respiration leg, which is a *reason* to recompute rather than reuse a bare rate.

**The identity that makes this a citation upgrade rather than a calibration:** since
`decomposed_C / litter_C ≡ decomposition_rate`, the first N leg **is** `decomposition_rate ·
litter_n`. So the form **replaces an uncited free rate with the carbon rate stoichiometry
forces it to equal** — `mineralization_rate` is **RETIRED**, no new parameter appears, and
that is the second weakly-supported param discharged by a form change rather than a citation
hunt (after `n_senescence_rate`). ⚠ Write it in the **recomputed-stoichiometric** form, never
collapsed to the rate: the identity holds only while `Decomposition` stays first-order, and
the collapsed form would silently outlive that.

### B-finding 4 — INVARIANCE MEASURED, not inherited — including `sealed_station`

The plan recorded "(B) is behaviorally inert for carbon (Finding 2)". Inherited inertness is
exactly the claim finding 5 had to overturn for (A), so it was re-measured by *running* the
candidate flows in-process (not integrating offline):

| scenario | driver | carbon max abs delta | `rationed` | N conserved |
|---|---|---|---|---|
| `sealed_chamber` | `run_season` | **0.0 (byte-identical)** | 0 | exact |
| `perennial` | `run_perennial` | **0.0** | 0 | exact |
| `consumer` | `run_perennial` | **0.0** | 0 | exact |
| `water_biting` | `run_season` | **0.0** | 0 | exact |
| **`sealed_station`** | `run_sealed` (two-rate) | **0.0** | 0 | exact |

`sealed_station` was run **because no probe had touched it** and it is in the cascade (⇒ the
station manifest); the decomposer calibration had already found it behaves unlike the
biosphere chambers. The structural backing is finding 7 (`f_N` is the only N→C channel, and it
reads `plant_n`, which (B) never touches) plus `soil_n` staying ≫ `sn_critical` (100 → 99.98,
band top 50) so availability never leaves 1.0.

⚠ **A probe bug worth recording, because the control did NOT catch it.** The first run showed
a 2.281 carbon delta. Two confounders were tested and both came back **clean** (flow-id
reduction order; an added zero-amount stock) — which reads as "the effect is real". It was
not: `Registry(flows, stocks)` had silently dropped the **aux processes**, so `thermal_time`
froze at 0 and DVS never advanced. The tell was not the control but a **crash signature** —
`annual_reset` failing with `storage_c 0.0`, i.e. a plant that never filled grain. **A control
that comes back clean eliminates only the confounders you thought of; it is not evidence the
effect is real.**

### B-finding 5 — the payoff, stated at its real size, and it is TWO REGIMES

| scenario | regime | frozen | under (B) |
|---|---|---|---|
| `sealed_chamber` | shedding-fed | 191.8 | **100.6** |
| `water_biting` | shedding-fed | 173.4 | **98.7** |
| `sealed_station` | reset-driven | 51.4 | **44.2** |
| `perennial` | reset-driven | 10.9 | **10.1** |
| `consumer` | reset-driven | 9.9 | **9.1** |

In the shedding-fed chambers pool C:N goes from ~2.2× wheat straw to **~1.24×** — a **~1.8×
refinement, not a fix**, and it must not be written up as one (the ~4-orders headline belongs
to (A)). In the reset-driven chambers the pool is governed by the annual dump and barely
moves, because the dump's C:N is set by the dying plant, not by either rate. Emergent
microbial C:N is ~107–123 (shedding-fed) / ~11–13 (reset-driven), i.e. it inherits the litter
value as the linear structure predicts.

### B-finding 6 — `microbial_n` may be a POOL; the extinction hazard is structurally unreachable

`microbial_carbon` is an `organ_stock`, i.e. a POPULATION, so extinction could in principle
zero it with the residual to the loss-sink and orphan its nitrogen. It cannot here:
`organ_stock` sets `extinction_threshold = 0.0` and the pass fires on `amount < threshold`, so
it requires a **negative** amount, which the flows' structural positivity prevents (`events ==
()` in every run above corroborates). ⇒ `microbial_n` as a POOL is safe. **Named seam:** if
anyone ever raises `microbial_carbon`'s threshold above 0, the N counterpart must be zeroed
with it or the emergent C:N breaks.

### What (B) cost (recorded as diagnosed; the build section below reports what happened)

An unfreeze of **two** contracts: biosphere manifest (`flow_set` 17 → 18, `param_files` for
`mineralization.yaml`), station manifest, the goldens carrying N stocks (`sealed_chamber`,
`perennial`, `consumer`, both long-horizons, `water_biting`, `sealed_station`), the Rust
mirror (`biosphere_params.txt` loses a param; two flow structs; a stock) and the crossport
tier. Same size as (A)'s.

⚠ **The decision is the user's, and the reason is not process — it is that what (B) BUYS
changed.** The recorded case for (B) was immobilization; B-finding 2 measures that as
unavailable without a pool-identity re-anchoring this project refuses. What remains is
*retiring `mineralization_rate` by stoichiometry* (strong, and the same move that discharged
`n_senescence_rate`) plus a *~1.8× refinement* on one of two regimes (modest). That is a
different trade than the plan's table promised, and it should be seen before two frozen
contracts are opened.


## THE (B) IMPLEMENTATION — built 2026-07-27

Biosphere unfrozen + re-frozen; station manifest cascaded. **The headline is not the
~1.8x refinement the diagnosis priced — it is that the litter pool's C:N stopped being a
free quantity at all.**

### What shipped

* `Mineralization` (`litter_n -> soil_n` at a free `mineralization_rate`) **deleted**,
  replaced by two flows in `mineralization.py`:
  `LitterNitrogenTransfer` (`litter_n -> microbial_n`, carried by `Decomposition`'s
  `decomposed_C`) and `MicrobialNitrogenRelease` (`microbial_n -> soil_n`, carried by
  `MicrobialRespiration`'s `respired_C`, **`f_O2` included**). One shared kernel,
  `carried_nitrogen(moved_C, pool_N, pool_C)`.
* **`mineralization_rate` RETIRED, and `params/mineralization.yaml` with it** — the file
  had no other parameter left, since (A) had already retired `n_senescence_rate`. This is
  the first param *file* the project has removed rather than re-valued: a `param_files`
  **membership** change, not a hash move. Its provenance record (five rounds of negative
  retrieval results) is archived verbatim at `docs/retired/mineralization.yaml`, because a
  stale *negative* result suppresses the next search and is the more expensive thing to
  lose. Pinned by a test, so it cannot be quietly tidied away.
* New stock `biosphere.microbial_n` — a **POOL**, deliberately not a POPULATION like its
  carbon sibling (B-finding 6): `organ_stock`'s extinction pass would orphan nitrogen the
  carbon side still holds. The seam is pinned as a test, not a comment.

### The results, measured

* **Carbon byte-identical in all 8 scenarios probed, and `drift_summary.json` regenerated
  BYTE-UNCHANGED** — the zero-carbon-effect claim confirmed at the artifact level, not
  just per-stock. `rationed == 0`, `events == ()`, N conserved exactly everywhere.
* **10 goldens moved and every one moved ONLY in nitrogen** (verified structurally, by
  parsing both snapshots per stock and grouping by *quantity* — a grep over a unified diff
  cannot tell you which stock an `"amount"` line belongs to). Each gained `microbial_n` and
  moved exactly `soil_n` + `litter_n`. `open_season` and `n_limited` are **structurally
  untouched**: open field builds no `litter_n`, so there is no `Mineralization` to replace.
* Cross-port: **101 passed**; Rust reproduces the new stock and the moved values.

### THE FINDING — the C:N law changed KIND, not just magnitude

The diagnosis priced (B) as "a ~1.8x refinement in one of two regimes". That understated
it, and the reason is an identity the diagnosis had already written down without following
through: under the retired form, N left the litter pool at a FREE rate while C left at
`decomposition_rate`, so the pool was pushed 2.727x away from its input's ratio. Both
currencies now leave on the **same** flux, so the pushing factor is exactly **1**:

    old:  pool C:N -> (shed C:N) x (k_min / k_decomp)  =  90 x 2.727  =  245.5
    new:  pool C:N -> shed C:N                          =  90

Measured: shedding-fed chambers sit at **98.7-100.6** at peak `litter_n`, and
`sealed_chamber` **ends at 90.6** — within 0.7 % of the shed ratio. Against wheat straw's
~80 that is **~1.25x** as committed, and **exactly 1.125x for the model itself** once the
N-free seed is accounted for (see the correction below), where the pre-(A) form gave
**0.004** and the post-(A) direct form gave 173-192.

The point is not the number. **The litter pool's C:N stopped being an accident of two
unrelated rate constants and became a function of the composition of the material that
fell into it** — and both numbers fixing that composition are cited (`carbon_fraction`
0.45, `n_residual` Van Hecke 2020).

⚠ **CORRECTION (2026-07-27, advisor catch then measured) — the residual above the shed
ratio is the N-FREE SEED, not "the pulsed-input transient", and the correction makes the
result STRONGER.** The first write-up of this section explained the 98.7-100.6 measurement
as a pulsed-input transient around a shed ratio of 90. **That cannot be right, by one line
of algebra**: with both currencies draining on the same first-order flux, `dC/dt = -kC`
and `dN/dt = -kN`, so `d(C/N)/dt = 0` — the ratio is *exactly invariant* between pulses,
and pulsing structurally cannot move it. The transient was real under the **retired** form,
where N drained 2.7x faster than C. **It is the same failure this section retires three
other claims for: an explanation outliving the mechanism that made it true** — and I wrote
it in the same commit that retired them.

The pool can only sit above the shed ratio if something *entered* above it, and the sealed
chambers do exactly that: they seed `litter_carbon0 = 3.0` mol C with **no `litter_n0`
counterpart** (C:N = ∞) — a seam the (A) record had already named. Measured with the seed
removed, the litter pool C:N equals the shed ratio **to 1.4e-15 relative, at every step**
— an identity, not a band. So:

* **the MODEL's litter pool C:N is `carbon_fraction / n_residual` exactly = 90**, i.e.
  **1.125x** wheat straw's ~80, not 1.25x;
* the committed scenarios' deviation is a known **unphysical initial condition** that
  decays at `decomposition_rate` like anything else in the pool — which is why
  `sealed_chamber` (3 yr) ends at **90.6** while `water_biting` (1 yr) still reads 98.6
  with ~0.10 mol of seed carbon left. The end-vs-peak difference was never horizon
  *dependence* of the law; it is the seed washing out.

Pinned in `tests/test_nitrogen_form.py`
(`test_the_pool_cn_IS_the_shed_ratio_and_the_deviation_is_the_N_FREE_SEED`), and the
committed-scenario bounds are now labelled **scenario facts, not model facts**: adding a
`litter_n0` counterpart should turn them red, for a good reason.

### Three previously-pinned claims RETIRED, and none of them was wrong

Each was a true measurement of a form that no longer exists, so they are **resolved**
rather than corrected — the distinction matters, because this project's habit is retiring
*artefacts*, and these are not artefacts:

1. the **245.5 quasi-steady law** — its `k_min` no longer exists;
2. *"a shedding-fed pool runs N-poor at 0.71-0.78 of the law"* — it ran N-poor **because**
   N drained 2.7x faster than C; with equal drains it does not;
3. *"the end-of-run snapshot is inflated ~2.4x and horizon-dependent over an order of
   magnitude"* — the inflation **was** the differential drain showing in the tail between
   pulses. **The horizon-dependence that correction 1's anti-regression pin existed to
   guard against is gone at its source**, so that pin was replaced by its **inverse**
   (`end/peak` must now be ~1) rather than relaxed. A pin that guards against a mechanism
   you have removed is not protection, it is decoration.

The **scope-B projection test is retired too, and its premise rather than its arithmetic
is what failed**: it projected the pool C:N if `mineralization_rate` were moved into
Stanford & Smith's 39-soil range (31-83, mean 47). There is no such rate to move. The
question was not answered — no value was chosen from a cited band — it was **dissolved**.
That also disposes of the last surviving objection from the decomposer calibration: a
parameter that does not exist cannot be mis-anchored to the wrong pool.

⚠ **What this does NOT claim.** The decomposer cluster's **carbon** rates are untouched
and still run at the fast edge of their literature ranges (`decomposition_rate` 4.0/yr,
Olson's fastest ecosystem), and the litter pool's C:N now *inherits* that carbon rate. The
honest statement is that the N cycle no longer contributes a **separate** uncited rate —
not that the decomposer side is now fully cited.

⚠ **The second regime is untouched**, which is why it is still two regimes: a reset-driven
chamber's pool is filled by the annual dump, whose C:N is set by the dying plant rather
than by any rate (10.9 -> 10.0, 9.9 -> 9.1).

### The blocking catch this build started from

⚠ **B-finding 4's invariance table had five rows; the manifest freezes seven** — it
omitted both 15-year long-horizons, `drift_summary`, and `open_season`. That is this
project's own 12th meta-finding one option later ("check a scenario list against the
MANIFEST, never against its own length"), and it was not bookkeeping: (B) parks nitrogen
in a **standing** `microbial_n` pool, so `soil_n` sits permanently lower, and
`soil_n -> availability -> uptake -> plant_n -> f_N` is a real **second-order** channel.
Finding 7's "f_N is the only N->C channel" proof closes the **direct read**, not the
**supply path**.

Measured over the full manifest roster, each scenario driven the way its own golden drives
it: the drain is **standing, not accumulating** — `perennial`'s `min(soil_n)` is
`99.995967` at **both** 5 and 15 years, identically. Worst drain anywhere is `8.6e-4` kg
against a 100 kg pool with `sn_critical` 50, i.e. ~5 orders from biting, and a decade does
not deepen it. `f_N` stays exactly 1.0 in every sealed scenario; `n_limited` reproduces its
recorded 0.175851 / 187 steps unchanged.


## The scope options — (A) and (B) TAKEN; (C) DIAGNOSED + PRICED; (D) still open

Ordered by dependency.

⚠ **This section used to say "Each is carbon-invariant except (D)." That is FALSE for
(C), and it is the line that made (C) look cheap** (advisor catch, then measured — see
"THE (C) DIAGNOSIS" below). The sentence was written when (A) and (B) were the live
options and is true of *them*; it was never checked against (C). (C) keys the **carbon**
relative death rate on DVS, so every carbon golden moves and the cascade is (D)-sized.
There is no reading in which (C) touches only the N leg: [A] §3.2.6 is about *biomass*
loss, and under (A) the N shed is **carried by** the carbon flux, so a DS-keyed N-only
form would break (A)'s `shed C:N = carbon_fraction/n_residual = 90` identity. The
subset-claim-written-flat shape again — this time about our own option list.

* **(A) N:C-coupled shedding + demand-based uptake — "make the N cycle physically
  scaled".** Two seams already documented in the frozen tree (`mineralization.py`'s
  N:C-coupled shedding; `nitrogen.py`'s WOFOST demand-deficit uptake, the named
  refinement seam for the "fixed-flux lock"). **Both are needed together**: coupled
  shedding alone removes the `uptake/k_sen` brake, so a capacity-based uptake that
  ignores demand would let `plant_n` accumulate without bound. Retires `n_senescence_rate`
  (uncitable) in favour of a tissue N:C (citable). **Carbon invariance is now MEASURED,
  not assumed (finding 6), and it is conditional on the target form**: a flat target
  ≥ ~2× `n_critical` is invariant in all 7 scenarios; a declining Greenwood dilution
  target bites in `day_neutral`. That choice is (A)'s one real scientific decision.
* **(B) ~~Immobilization~~ → MICROBE-MEDIATED N TRANSIT — ✅ TAKEN, built 2026-07-27
  (see "THE (B) IMPLEMENTATION" above).** `microbial_n` stock +
  **stoichiometric** `litter_n → microbial_n → soil_n`, each leg carried by the carbon flux
  its sibling already moves. ⚠ **The name was wrong and is corrected here**: the
  C:N-driven mineral-N draw that "immobilization" means is **UNAVAILABLE**, because our
  `microbial_carbon` is a *transit* pool (comparable to litter C, not a few % of it — CUE
  is 1.0 in this tree), so imposing a homeostatic microbial C:N would be the pool-identity
  re-anchoring this project has twice refused. See **B-findings 1–2** above; the
  immobilization seam stays open with a *measured obstacle*. What (B) does deliver:
  **`mineralization_rate` RETIRED by stoichiometry** (`decomposed_C/litter_C ≡
  decomposition_rate`, so the free rate is replaced by the carbon rate it must equal —
  the `n_senescence_rate` move again) and a **~1.8×** pool-C:N refinement in the
  shedding-fed regime only. Carbon invariance is now **MEASURED in all five sealed
  scenarios incl. `sealed_station`** (B-finding 4), not inherited from Finding 2.
* **(C) The DS-dependent shedding form — ✅ DIAGNOSED + PRICED 2026-07-27, NOT built
  (see "THE (C) DIAGNOSIS" below).** ⚠ **This bullet used to read "([B], p. 95: 0/day
  before anthesis ramping to 0.15/day by DS 2.0)" and the magnitude was read off the
  WRONG LOCUS** — 0.15/day is an *exercise answer* on p. 113; the table p. 95 actually
  cites (Listing 5, the rice IR36 crop file, p. 212) peaks at **0.012/day**, 12.5×
  lower. Both agree on the *form* (zero below DS 1.0), which is the half that matters.
  The rest of the old bullet stands: the frozen carbon `rdr_leaf/stem/root` are flat
  constants, so (A) inherits flatness, **not** DS-dependence, and (C) is a separate
  item. What the diagnosis adds is the price — it is a **carbon** change (see the
  correction above), it **breaks `perennial`** (RK4 hard-error; the decade CO₂ floor
  fails by ~an order and loses its fixed point), and it takes `open_season`'s peak
  canopy to **LAI 16.4** against real wheat's 5–8, because our flat `rdr_leaf` has been
  standing in for canopy-regulation science the tree does not have.
* **(D) ⚠ The N→C throttle — the only option that makes the N cycle load-bearing, and it
  fights last week's calibration.** Real soils decompose high-C:N residue *slower*
  because microbes are N-starved. Adding that factor is what would give N a carbon
  effect — but the decomposer calibration measured that closure **requires the fast
  edge** (central literature rates starve the recycled-CO₂ loop and crash annual re-sow;
  RothC-BIO is infeasible at any litter size). An N-throttle only ever *slows*
  decomposition. **Expect it to break closure in every sealed scenario.** This is a
  genuine scientific conflict, not an implementation risk, and it should be priced before
  being attempted, not during.

## THE (C) DIAGNOSIS — measured 2026-07-27; **PRICED AT (D) SIZE AND NOT BUILT**

Read-only probes in `M:/claud_projects/temp/ncycle_c/`. Nothing in the repo was run
against a modified `src/`; the candidate flows are defined in the probe and swapped into
a freshly-built registry, so every number below is the model, run end to end.

⚠ **The zero-feedback license that made the (A)/(B) diagnoses cheap DOES NOT APPLY
HERE.** (A) and (B) could be integrated offline against the *recorded* carbon trajectory
because `f_N ≡ 1` made carbon independent of the N form. (C) changes carbon at the
source, so nothing could be reused and every scenario was re-run. And (C) is
**DVS-keyed**, i.e. it depends on exactly the aux accumulator that probe B2's first bug
froze at 0 — so the probes **assert `max(DVS) == 2.0`** before believing any delta
(they do, in all eight scenarios).

### C-finding 1 — our own record quotes the WRONG TABLE, and the source has two

[A] Penning de Vries et al. 1989 contains **two** definitions of `LLVT`, the leaf
relative-death-rate function:

| Locus | `LLVT` (DS → 1/day) | peak |
|---|---|---|
| **Listing 5, p. 212** — *"Crop data for rice (variety IR36)"*, the one §3.2.6 cites by name | `0,0  1.0,0  1.3,0.007  1.8,0.012  2.5,0.012` | **0.012** |
| **T10, p. 113** — an *exercise answer* deriving a table from a hypothetical loss pattern | `0,0  1.0,0  1.5,0.03  2.0,0.15` | **0.15** |

⚠ **And the failure is NOT that round 2 mis-read the page — it is that the qualifier
was dropped downstream, which is this project's meta-finding in its purest form.**
`docs/retired/mineralization.yaml:268` records the entry correctly and carefully:
*"Its own numeric example of that function (**p. 113, exercise T10**) is …"*, with the
currency and crop caveats attached (*"⚠ THAT IS A SCALE NOTE, NOT A DELTA … LLVT is leaf
BIOMASS death in RICE"*). Every **restatement** then de-qualified it into "the source's
function": the same file's own summary at line 424 (*"0/day before anthesis, ramping to
0.15/day by DS 2.0"*), `post-roadmap-citation.md:413`, and this plan's (C) bullet. The
careful sentence stayed put while the careless paraphrase travelled.

**And the more authoritative table was never retrieved at all.** §3.2.6 p. 95 says
*"Listing 5 contains a numerical example that corresponds with observations on a rice
crop in the Philippines (functions LLVT and LRTT)"* — it points at the crop file, not at
the exercise. For five citation rounds the record's number for this form came from the
lower-authority locus while the one the text cites sat unopened in the same PDF. 12.5×
apart at the terminal rate, and the direction of the comparison with our flat `0.02/day`
flips with it (Listing 5's peak is *below* ours; T10's is 7.5× *above*).

**Generalized**: *when a source is quoted for a number, check whether it also contains
that number somewhere more authoritative.* Round 4's rule was "open the paper you cite";
this is one level in — open the **right part** of the paper you cited, and treat an
example-in-an-exercise as provisional until the reference parameterization is looked
for. A locus error survives inside a correctly-attributed quote.

**The form claim survives both** — every reading of this source has the rate at **zero
below DS 1.0** — so the round-2 structural finding ("ours is the degenerate case of the
cited form, non-zero exactly where the source is zero") is unaffected. It is the
magnitude that was mis-sourced, and C-finding 4 shows the magnitude decides whether a
frozen golden's `f_N` moves.

Both tables were read off **page images**, forced not chosen: extraction renders the
line as `PUNCTION LLVT = 020., 1.,0:, 1:5,0.03; 2-,0.15`. And T10 supplies its own
**arithmetic check**, which is the round-5 discipline paying off — it states the loss
pattern the table reproduces (*"the reproductive period lasts about 30 days. In the
first 15 days, loss is 20 %, in the second 15 days 75 % of what remained"*), and the
digits satisfy both halves simultaneously: DS 1.0→1.5 at mean 0.015/day gives
`1−e^(−0.225)` = **20.2 %**, DS 1.5→2.0 at mean 0.09/day gives `1−e^(−1.35)` =
**74.1 %**.

### C-finding 2 — what the primary actually licenses (p. 95, first-hand)

* The descriptive form is DS-keyed for **leaf and root** (`LLVT`, `LRTT`). Listing 5's
  root table is `0,0  1.0,0  1.3,0.011  1.8,0.010  2.5,0.010` — and its post-anthesis
  plateau (0.010–0.011/day) is within 10 % of our flat `rdr_root = 0.01/day`.
* **There is no stem function at all**, in either the listing or the text: *"except for
  their reserves, stems do not lose weight."* Our `rdr_stem = 0.005/day` is unsupported
  by this source in **existence**, not merely in value.
* The worked example is **rice IR36**, not wheat. Same lineage caveat as the day-neutral
  crop work: the *shape* is transferable, the *numbers* are another crop's.
* The source states its own expected outcome — *"Description such as this usually
  results in a loss of 40-60 % of leaf area at harvest time"* — which is a checkable
  band, and C-finding 5 checks it.
* The source explicitly sanctions calibration: *"These numbers can be used as default
  values, but should be calibrated to mimick specific situations."* See C-finding 7 for
  why that permission does **not** unblock (C) here.

### C-finding 3 — (C) is a CARBON change; the plan's invariance line was false

Stated in the corrected section header above. Measured: every carbon stock moves in
every scenario. The reason it cannot be reduced to an N-only change is structural — [A]
§3.2.6 is about biomass loss, and (A) made the N shed a *function of* the carbon shed,
so DS-keying only the N leg would break the `shed C:N = 90` identity (A) shipped.

⚠ **And (C) has an (A)-style recomputation-drift hazard one flow over**:
`mineralization.py` recomputes `senescence_flux(organ, rdr_*)` off the **flat**
`SenescenceParams`, so a (C) that changed only `allocation.Senescence` would silently
keep shedding N at the old rates. The probe implements both legs off one shared table
object for exactly this reason; a build would have to do the same, and pin it (the
existing `shed-C == Senescence`'s litter leg pin is the precedent).

### C-finding 4 — the tripwire FIRES, and which table you read decides whether it does

`test_nitrogen_form.py` pins `open_season`'s peak at 12.633 t/ha against the **14.4248
t/ha** Greenwood crossing, with the comment *"any calibration that grows the open-field
crop ~15 % pushes the target below `n_critical` and moves a frozen golden."* (C) is that
calibration. **Measured, not inferred from the crossing** (`f_N` read off the
trajectory, and it is a live feedback — `carbon_budget.py:207` multiplies gross
assimilation by `f_water · f_N`, so the peak below is already self-consistent):

| open_season | peak W (t/ha) | vs crossing | min `f_N` | steps `f_N < 1` |
|---|---|---|---|---|
| frozen | 12.6331 | 0.876× | 1.000000 | 0 / 306 |
| **Listing 5** | **18.6777** (+47.8 %) | **1.295×** | **0.995213** | **6 / 306** |
| T10 | 13.8790 (+9.9 %) | 0.962× | 1.000000 | 0 / 306 |

So the pin did its job: this is the **first time `f_N` bites in a frozen scenario**.
⚠ **State the size honestly** — 0.5 % over 6 of 306 steps. The tripwire fires; nitrogen
does not thereby become load-bearing. And note T10 lands at **96.2 %** of the crossing,
i.e. the locus error of C-finding 1 was worth ~4 % of clearance either side of a
threshold: the wrong table would have reported "the tripwire does not fire."

### C-finding 5 — ⚠ THE STRUCTURAL FINDING: the flat `rdr_leaf` is standing in for canopy regulation we do not have

Peak leaf area index, `open_season` (real wheat peaks at ~5–8):

| | peak LAI | final LAI | leaf lost by season end |
|---|---|---|---|
| frozen (flat 0.02/day) | **5.1908** | 3.1927 | **38.5 %** |
| Listing 5 | **16.3971** | 11.4802 | 30.0 % |
| T10 | **16.1561** | 0.3356 | 97.9 % |

**Both source tables give the same peak canopy** (16.40 / 16.16) — because both are
zero below DS 1.0, and the peak is reached at anthesis. So this is **not** the
locus question of C-finding 1; it is the *shared* half of the form, the half every
reading of the source agrees on. A canopy of LAI 16 is 2–3× real wheat.

The tree has no self-shading leaf death, no leaf-age cohorts, no SLA aging — nothing
that regulates a canopy on the way up. **The flat `rdr_leaf` has been doing that job**,
and removing it (which is what the primary says to do) exposes the gap. This is the
decomposer calibration's deepest finding again, on the other side of the plant: *the
references were propped up by an unphysical rate, and making the rate faithful leaves
the demo measurably worse.*

⚠ **The corroboration is the primary's own stated outcome, and it cuts against the
primary's own table**: [A] says the descriptive form *"usually results in a loss of
40-60 % of leaf area at harvest time."* In our tree the **frozen flat form** lands at
**38.5 %** — just under the band — while Listing 5 gives **30.0 %** and T10 **97.9 %**.
That does not vindicate the flat form *as a form*: it says the flat rate was
(implicitly) sized to produce roughly the right *integrated* loss while getting the
*timing* entirely wrong — shedding hardest when the source says zero.

### C-finding 6 — closure: Listing 5 breaks `perennial`; T10 breaks re-sow everywhere

**Euler alone reads clean and that is the trap** — `rationed == 0` under Listing 5 in
all eight scenarios. Increment 1's record ("rationed under Euler, hard-errored under
RK4") repeats exactly:

* **`perennial`, Listing 5, RK4 → `ArbitrationError`** (`scale_f = 0.95277 < 1`). A
  needed scale is a hard error under a higher-order scheme, so this is not a
  near-miss. `test_decade_closure_held` runs RK4 over the full 15 years.
* **`perennial`'s decade CO₂ guard fails by ~an order AND loses its fixed point.**
  Per-year minimum `carbon_pool`, the way `test_decade_min_carbon_pool_stationary`
  computes it:
  * frozen: `[0.07402, 0.03873, 0.05421, 0.05481, 0.05484, 0.05484, …]` — settles on
    an attractor; past the `_TRANSIENT = 2` sow-in years the min is **0.054208 > 0.05**
    ✓. (This reconstruction was **validated against the committed test comment first**
    — *"dips to ~0.039 during soil establishment before settling to ~0.055"* — per
    finding 10's rule that you reconstruct a frozen quantity only to CHECK it. Probe C2
    had invented its own transient rule and reported 0.0387; C3 uses the test's.)
  * Listing 5: `[0.00848, 0.01853, 0.01092, 0.01020, 0.01396, 0.00591, 0.02020,
    0.02396, 0.00650, …]` — **fails the 0.05 floor by ~an order**, and never settles:
    it wanders 0.006–0.027 for fifteen years, so `is_stationary` fails too.
* **T10 hard-errors at `annual_reset` in all four reset-driven scenarios**: *"seed bank
  too small to re-sow — storage_c 0.1407 < seedling 0.16."* Grain never fills, so
  closure fails at the sow.
* `consumer`, `sealed_chamber` and `water_biting` **survive** Listing 5 on both
  integrators (`consumer`'s decade CO₂ min settles at 0.14083, clear of the floor).

Liveness moves the *other* way, as expected for a bigger plant: `perennial` peak leaf
0.994 → 1.424, `consumer` 1.076 → 1.254.

### C-finding 7 — the crop transfer, and the calibration trap the source itself opens

The rice table's *shape* transfers; the *consequence* is amplified by our crop. Rice
IR36's whole season is 102–135 days ([A], p. 113); our winter wheat reaches anthesis at
day ~251. "Zero below DS 1.0" therefore removes shedding over ~250 days rather than
~60 — and `e^(−0.02·250) ≈ 0.0067`, a ~150× difference in retained vegetative leaf
carbon. That, not the post-anthesis knots, is what produces LAI 16.

[A] explicitly permits calibrating the values (*"should be calibrated to mimick specific
situations"*), which is the obvious escape: pick a table that keeps `perennial` closed.
⚠ **Refused, for the reason this repo has refused three times before.** Our only
available calibration target is *closure of an artificial chamber* and *our own frozen
goldens* — not observations. Choosing a model form's constants for their effect on
frozen output is the consumer-chamber-2× / DPM-RPM-labile / ruling-B shape. If (C) is
ever built, its table must come from a source, and the chambers must be shown to
survive it — not the other way round.

### THE PRICE, and why (C) is blocked on a missing science rather than on effort

(C) is a full biosphere **and** station unfreeze — a carbon change moves *every* golden,
both manifests, `biosphere_params.txt`, the Rust mirror and the crossport tier — i.e.
the cascade this plan reserved for (D). And unlike (A)/(B), **it does not stand alone**:
taking the primary's form as printed either

* breaks `perennial` (RK4 hard error, decade CO₂ floor, lost fixed point), **or**
* requires the canopy-regulation science of C-finding 5, which the tree does not have
  and which is a scope of its own, **or**
* requires a calibration whose only target is our own goldens (C-finding 7, refused).

So the honest status is **DIAGNOSED, PRICED, NOT RECOMMENDED AS SPECIFIED** — the
`rdr_*` form gap is real and first-hand, and the thing that blocks closing it is a
*different* missing science, not the citation. That reading is the same one scope (A)
reached about the oracle gap ("not a calibration task"), arrived at independently.

The natural successor is therefore **not** (C) or (D) but the canopy regulator (leaf-age
or self-shading-driven death), which is what would let the primary's form be adopted
without a fitted table. Left as a user call; nothing here is built.

## Cost of the cascade (any of A–D)

Frozen-science, so **Python-canonical** under the Rust-primary pivot (rule 3): it moves
manifest-named items, so it is a biosphere **unfreeze**, not Rust-first content.

* biosphere manifest: `param_files` (mineralization/nitrogen/senescence), `flow_set`
  (17 flows; (B) adds one or two), plus golden hashes
* goldens carrying N stocks: `sealed_chamber`, `sealed_station` (⇒ **station** manifest
  cascade too, as the decomposer calibration hit), `n_limited`, `water_biting`
* the Rust mirror (`biosphere_params.txt` + the flow/param structs) and the crossport tier
* `n_limited` is the one scenario where `f_N` **does** bite (deliberately starved, uptake
  off) — it is *not* one of the 7 frozen scenarios, and it is the natural place to pin
  that (A) preserves the f_N-biting regime

## Open questions for the user

1. ✅ **RESOLVED — (A) and (B) are both TAKEN** (B built 2026-07-27; the user's
   direction was "do what you recommend", and the recommendation was (B) for the
   parameter retirement rather than for the C:N refinement). **What remains open is (C)
   and (D)**, and (D) still fights the fast-edge closure requirement head-on, so it stays
   a decision to price before attempting rather than one to drift into. The original
   framing is kept below for the record:

   ~~(A) is TAKEN. **(B) is now fully diagnosed and NOT started — awaiting this
   decision**~~ (see "THE (B) DIAGNOSIS" above): it is measured carbon-invariant
   in all five sealed scenarios and would retire `mineralization_rate` by stoichiometry, but
   it **cannot deliver immobilization** (pool identity), so the trade is "retire a contested
   param + a ~1.8× refinement" against "unfreeze two contracts". (D) is the only one with a
   carbon effect and is expected to conflict with the decomposer calibration.
2. ✅ **RESOLVED BY FINDING 9 — and neither branch of this fork was the answer.** The user's
   criterion ("what represents reality more faithfully") turned the fork into a **retrieval
   question**, and the paper was already in `sources/`. Greenwood's published equation carries
   its own domain bound (`W > 1 t/ha`), its own below-domain behaviour (**constant** 5.697 %,
   with a mechanism and an Ågren 1985 citation), and its own denominator (**excluding fibrous
   roots**). That form gives `f_N ≡ 1.0000` in **all seven frozen scenarios**, `water_biting`
   and `day_neutral`. So: **flat *was* right, at 5.697 % not my invented 3.0 %, and for the
   opposite reason** — not because a constant is convenient, but because our crops sit below
   the curve's domain, where the primary states %N *is* constant. The declining ramp was
   Greenwood extrapolated into the region he excluded, and `day_neutral`'s 0.713 was that
   extrapolation, not physics. **Both forks were invented; reading the source dissolved the
   choice.** The residual honest caveats are in finding 9 (the 14.42 t/ha crossing point, 12 %
   margin, and the roots-in-denominator variant that *does* bite). Kept below for the record:

   ~~Finding 6 makes this the *real* question, and it is a fork, not a footnote:~~
   * **flat target ≥ ~2× `n_critical`** (e.g. 3.0 % DM) → `f_N ≡ 1` in all 7 scenarios,
     **carbon byte-identical**, N stocks move. A fidelity-and-citation deliverable, the
     same honest shape as the original Step-6 one ("nitrogen mass cycles internally and is
     conserved," NOT "emergent N feedback") — but now *stated up front* rather than
     discovered at the end.
   * **declining Greenwood dilution target** (4.5 → 1.2 % DM) → more physical, still
     carbon-invariant in the six winter-wheat scenarios, but **moves `day_neutral`**
     (`f_N` → 0.713). A science change, with the authored crop as the thing that moves.

   Either way the goldens' N stocks move, so the cascade cost is the same.

   ⚠ **This fork is NOT neutral, and presenting it as neutral is itself the trap**
   (advisor catch). **The declining form is the literature-shaped one** — Greenwood et al.
   1990, *already cited in `nitrogen.py`*, is titled "Decline in percentage N of C3 and C4
   crops with increasing plant mass": a declining dilution curve *is* what that paper is
   about. Flat 3.0 % is the **invariance**-shaped one. So picking flat *because* `f_N ≡ 1`
   keeps carbon byte-identical is **choosing a model form for its effect on frozen
   output** — the co-adaptation/backfitting shape this repo has caught itself in
   repeatedly (the consumer-chamber 2×, the DPM/RPM labile re-read this project *refused*,
   ruling B's "the oracle is a diagnostic, never a fit target"). An earlier draft of this
   section listed "carbon byte-identical" as flat's **advantage**, which is exactly how the
   trap reads from the inside.

   Three honest ways out — the user should see which one they are picking:
   * justify flat **on its own merits** (a carbon-limited 52 g plant with a vegetative pool
     that collapses ×0.008 may genuinely not dilute much — arguable, and *measurable* from
     the recorded concentration trajectory);
   * take the **declining** form and accept that `day_neutral` moves;
   * present both with the trap named.

   ⚠ **And the cost asymmetry is smaller than it looks**: `day_neutral` is **authored
   content** — "authored ≠ validated", runtime-only, *not frozen*. A form that moves only
   `day_neutral` is **not a freeze event**. That materially weakens the case for flat.
3. **(D) as a separate, explicitly-priced decision?** It is where the science is, and it
   is also where the closure constraint bites hardest.
