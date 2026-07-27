# Post-roadmap — the nitrogen-cycle FORM gap

**Status: DIAGNOSIS COMPLETE (2026-07-27), scope NOT yet decided. No file touched.**

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

### Finding 2 — `f_N` cannot bite at ANY physically-defensible N scale. Not a param artifact.

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

## The scope options (NOT yet decided — needs advisor review + a user call)

Ordered by dependency. Each is carbon-invariant except (D).

* **(A) N:C-coupled shedding + demand-based uptake — "make the N cycle physically
  scaled".** Two seams already documented in the frozen tree (`mineralization.py`'s
  N:C-coupled shedding; `nitrogen.py`'s WOFOST demand-deficit uptake, the named
  refinement seam for the "fixed-flux lock"). **Both are needed together**: coupled
  shedding alone removes the `uptake/k_sen` brake, so a capacity-based uptake that
  ignores demand would let `plant_n` accumulate without bound. Retires `n_senescence_rate`
  (uncitable) in favour of a tissue N:C (citable). Carbon-invariant — to be *verified*,
  not assumed, since `f_N` reads `plant_n`.
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
2. **Is a behaviorally-inert deliverable acceptable?** (A)+(B) improve *physical
   fidelity and citation standing* with a measured **zero** effect on every carbon
   trajectory. That is the same shape as the original Step-6 deliverable ("nitrogen mass
   cycles internally and is conserved," NOT "emergent N feedback") — honest, but it must
   be *stated* as such up front rather than discovered at the end.
3. **(D) as a separate, explicitly-priced decision?** It is where the science is, and it
   is also where the closure constraint bites hardest.
