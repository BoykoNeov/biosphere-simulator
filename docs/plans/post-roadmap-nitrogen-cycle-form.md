# Post-roadmap — the nitrogen-cycle FORM gap

**Status: DIAGNOSIS COMPLETE (2026-07-27), advisor-reviewed. Scope NOT yet decided —
a user call. No frozen file touched; `git diff src/` empty.**

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

The scaling law is exact — all rows are 1.894× `(tissue C:N) × (k_min/k_decomp)`, the
constant being the litter pool's distance from `I_c/k_decomp` equilibrium:

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

### Finding 6 — invariance MEASURED: it holds for a flat coherent target, in all 7 scenarios; the target FORM is load-bearing

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

## The scope options (NOT yet decided — needs advisor review + a user call)

Ordered by dependency. Each is carbon-invariant except (D).

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
* **(B) Immobilization** — `microbial_n` stock + C:N-driven `litter_n → microbial_n →
  soil_n`. Meaningful only after (A) makes C:N a real ratio. Behaviorally inert for
  carbon on its own (Finding 2).
* **(C) The DS-dependent shedding form** ([B], p. 95: 0/day before anthesis ramping to
  0.15/day by DS 2.0). Largely *subsumed* by (A) if C senescence is the driver — but note
  the frozen carbon `rdr_leaf/stem/root` are themselves flat constants, so (A) inherits
  flatness, **not** DS-dependence. (C) stays a separate, additive item.
* **(D) ⚠ The N→C throttle — the only option that makes the N cycle load-bearing, and it
  fights last week's calibration.** Real soils decompose high-C:N residue *slower*
  because microbes are N-starved. Adding that factor is what would give N a carbon
  effect — but the decomposer calibration measured that closure **requires the fast
  edge** (central literature rates starve the recycled-CO₂ loop and crash annual re-sow;
  RothC-BIO is infeasible at any litter size). An N-throttle only ever *slows*
  decomposition. **Expect it to break closure in every sealed scenario.** This is a
  genuine scientific conflict, not an implementation risk, and it should be priced before
  being attempted, not during.

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

1. **Which option(s)?** (A) is the coherent minimum and the one that discharges the
   uncitable param. (B) needs (A). (D) is the only one with a carbon effect and is
   expected to conflict with the decomposer calibration.
2. **Which tissue-N target form — and therefore, is the deliverable inert or not?**
   Finding 6 makes this the *real* question, and it is a fork, not a footnote:
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
