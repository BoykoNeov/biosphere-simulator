# The remaining water-stress curves — `WSFD` built, `WSFL` refused

> The two named successors the soil-water re-basing left behind
> (`docs/log/soil-water-rebasing.md`, "NOT built"). [F] applies one deficit factor to
> **four** processes; after the re-basing we carry **one** (`WSFG`, growth /
> transpiration / root extension). This takes the other two.
>
> Probes: `M:/claud_projects/temp/water-stress-curves/`.

## The charge, and what the record got wrong about it

The soil-water re-basing recorded both successors in one sentence:

> **`WSSL` (leaf-area expansion, 0.40) and `WSSD` (phenology, 0.40)** — [F] applies the
> deficit factor to four processes **with different thresholds**; we carry one, because
> we have no water-gated leaf-expansion or drought-accelerated development term for the
> others to attach to. That is a real gap, not a simplification.

⚠ **"With different thresholds" is wrong for `WSSD`, and the error is load-bearing.**
Table 15.1's own caption (page render, PDF p. 210 = printed p. 195) reads:

> Threshold FTSW for leaf area development (**WSSL**) and growth (**WSSG**), **and a
> coefficient of phenological development response to drought (WSSD)**

`WSSD` is not a threshold on `FTSW` at all. [F] Eqn 15.8 is

```
WSFD = (1 − WSFG) · WSSD + 1
```

— driven by `WSFG`, which the tree already computes against `wssg = 0.30`. So the
phenology curve needs **no new `FTSW` call site, no new threshold, and no second
comparison**: one scalar and the factor we already have. The price recorded for it was
wrong in the expensive direction, and correcting it downward is the first finding.

The same render re-confirms the wheat row (`WSSL` 0.40, `WSSG` 0.30, `WSSD` 0.40), the
printed-table typo on soybean's `WSSG` (`0..25`), and adds one thing the previous read
did not record: **`WSSD` is populated for only two of the ten crops** — wheat 0.40 and
chickpea 0.40. Every other row is blank, matching [F]'s own text: *"the scientific basis
and a procedure to measure WSSD need to be sought."* There is **no potato row at all**.
That is a citation-grade reason for the coefficient to be optional, not a convenience.

## `WSFL` — REFUSED, and the reason is about the SOURCE, not about us

The recorded blocker was a claim about our tree ("no water-gated leaf-expansion term to
attach to"). Reading [F]'s own listing upgrades it to a claim about **[F]**, which is a
much harder thing to erode.

[F] Box 16.2 computes the daily LAI increase in **three** phenological branches, and
applies `WSFL` to exactly one of them:

```vb
If CTU <= tuEMR Then                      ' pre-emergence
    GLAI = 0: DLAI = 0
ElseIf CTU > tuEMR And CTU <= tuTLM Then   ' node-driven expansion
    INODE = DTU / PHYL
    MSNN  = MSNN + INODE
    PLA2  = PLACON * MSNN ^ PLAPOW
    GLAI  = ((PLA2 - PLA1) * PDEN / 10000) * WSFL      '  <-- WSFL applied HERE
    PLA1  = PLA2
    DLAI  = 0
ElseIf CTU > tuTLM And CTU <= tuBSG Then   ' carbon-driven expansion
    GLAI = GLF * SLA                                   '  <-- and NOT here
    ...
```

The branch that carries `WSFL` is **sink-limited**: leaf area comes from main-stem node
number through an allometric power law (Eqn 9.3–9.5) and is *independent of dry matter*.
Drought there produces less leaf area for the same leaf mass — thicker leaves — which is
why it needs its own factor. The branch that does **not** carry `WSFL` is
**source-limited**: `GLAI = GLF · SLA`, leaf area from leaf dry matter, whose growth was
already scaled by `WSFG` through `RUE = IRUE · TCFRUE · WSFG`. Applying `WSFL` there
would multiply the deficit in twice, and [F] declines to.

**Our canopy is only ever [F]'s second branch.** `canopy.leaf_area_index` is
`LAI = leaf_carbon · sla_per_mol_c / ground_area` — the P2 lock, "LAI is derived, not
stored". The mapping is term-for-term:

| [F] (post-TLM) | ours |
|---|---|
| `RUE = IRUE · TCFRUE · WSFG` | `carbon_budget.limitation` = `f_water · f_N` on gross assimilation |
| `DDMP = SRAD · 0.48 · FINT · RUE` | the shared `CarbonContext` increment `DMI` |
| `GLF` (leaf share of `DDMP`) | `allocation.partition` `FL(DVS)` share of `DMI` |
| `GLAI = GLF · SLA` | `LAI = leaf_carbon · sla_per_mol_c / ground_area` |

So the drought response of our leaf area is not missing — it is **present and complete,
and it is the response [F] itself specifies for a model of this shape.** The gap the
record named is real about the *node-driven* branch, which we do not have and which
`WSFL` is the factor for.

**What building it would actually cost**, so nobody prices it as "one more multiply":
a main-stem node accumulator (`PHYL`), the `PLACON`/`PLAPOW` allometry, a `tuTLM`
phase boundary, and — the expensive part — **leaf area as a state variable**, since
`WSFL` scales an *expansion rate* and the area it withholds must stay withheld. Applying
it to standing `LAI` instead would make the canopy shrink and re-grow with soil water,
which is wilting/rolling, a different mechanism and not this citation. That is a direct
reversal of the "LAI is derived, not stored" lock, and it would decouple leaf area from
leaf carbon, which senescence and the nitrogen gate both read.

**Recommendation: do not build `WSFL`.** ⚠ Per this record's own standing rule, that is a
recommendation and not a verdict — the user's call. What changes is that the successor
should now be filed as *"a node-driven, sink-limited leaf-expansion phase"* (of which
`WSFL` is one line), never as *"the missing `WSFL` multiply"*.

## `WSFD` — BUILT

### The equation and where it attaches

```
WSFD = (1 − WSFG) · WSSD + 1                     [F] Eqn 15.8
DTU  = DTU · WSFD        (after emergence)       [F] Eqn 15.9 / Box 16.2
```

`ThermalTimeAccumulation` already multiplies its degree-day rate by two optional
modifiers (`verfun`, Eqn 8.6; `ppfun`, Eqn 7.6). `WSFD` is the third, with two
differences that are both [F]'s, not ours:

1. **It can exceed 1.** Drought *hastens* development (Table 15.2: acceleration is the
   more common response). `WSFD` is bounded on `[1, 1 + WSSD]` for `WSSD > 0` — the
   first modifier in this module that is not a `[0, 1]` limitation factor.
2. **It is not phase-gated.** `verfun`/`ppfun` are gated to `DVS < 1` (wheat is
   insensitive to cold and daylength past anthesis). [F] gates `WSFD` only on
   `CTU > tuEMR`, and our accumulator *starts* at emergence (`thermal_time = 0 ⇒
   DVS = 0`), so the gate is satisfied by construction and the factor runs through grain
   filling too.

Negative `WSSD` is [F]'s own provision for species that are *delayed* by drought
("Eqn 15.8 can still be used with negative values"). `WSSD = −1` arrests development at
`WSFG = 0`; below that `WSFD` goes negative, which [F] forbids in the same breath it
forbids it for photoperiod (*"development is only a forward process and cannot be
negative"*). So the constructor rejects `wssd < −1`, and that bound is cited rather than
defensive.

### Measured FIRST: does anything catch it?

`WSFD ≡ 1` wherever `WSFG ≡ 1`, so the question is which runs ever see stress at all.
Instrumenting `transpiration.water_stress_factor` — the single point all consumers reach
— and replaying every regression module's own final-state builder on the **frozen** tree
(`measure_wsfg.py`, `measure_scenarios.py`):

| run | min `FTSW` | min `WSFG` | max `WSFD` (`WSSD` = 0.40) |
|---|---|---|---|
| `open_season` (DEFAULT) | 0.9041 | **1.0000** | 1.0000 — inert |
| `sealed_chamber` | 0.7867 | **1.0000** | 1.0000 — inert |
| `perennial_chamber` / `consumer_chamber` / both long-horizon | 0.7867 | **1.0000** | 1.0000 — inert |
| `n_limited` | 0.9041 | **1.0000** | 1.0000 — inert |
| `greenhouse` / `harvest` / `lighting` | 0.7867–0.9626 | **1.0000** | 1.0000 — inert |
| `station` / `water_recovery` / `cabin_gas` | — | — | no biosphere water-stress evaluation |
| `drought` (no golden) | 0.7039 | **1.0000** | 1.0000 — inert |
| `potato` / `day_neutral` (no golden) | 0.9018 / 0.9041 | **1.0000** | 1.0000 — inert |
| **`water_biting`** (golden, non-frozen) | 0.0500 | **0.1667** | **1.3333** |
| **`deep_water`** (no golden) | 0.0803 | **0.2677** | **1.2929** |

**Every one of the 7 frozen reference scenarios is inert, measured rather than argued** —
and confirmed by running the build with `wssd` declared and active: the frozen roster
comes back **bit-identical in every stock and every aux**. Exactly two runs move, and
only one of them has a golden.

⚠ **Consequence, recorded because nothing else will catch it.** `aux_set` is unchanged
(no new accumulator), `flow_set` is unchanged (22), `param_files` is unchanged, and all
7 frozen goldens are byte-identical — so **this unfreeze moves nothing in
`docs/biosphere-reference.manifest.json` at all.** The manifest gate cannot see it and
the frozen goldens cannot see it. The only automatic gate is
`water_biting_state.json`, which is *not* in the manifest. This is the honor-system
ceremony CLAUDE.md warns about, entered through a new door: not a provenance-only edit,
but a **form** change whose whole effect lands outside the frozen roster.

### The positive feedback, measured rather than reasoned away

`WSFD` speeds `DVS` → root extension stops at the `DVS ≥ 1` gate earlier → shallower
zone → smaller `TTSW` → lower `FTSW` → larger `WSFD`. It is bounded (`WSFD ≤ 1 + WSSD`)
so it cannot run away, but it is the same shape as the absorbing-state trap this record
has hit twice, so it was measured (`proto_feedback.py`), including at an absurd
`WSSD = 1.50`:

| run | depth stops (day) | `DVS ≥ 1` (day) | which gate stopped the roots | min `FTSW` |
|---|---|---|---|---|
| `water_biting`, frozen | 12 | 251 | `subsoil_water ≤ 0` (dry subsoil) | 0.0500 |
| `water_biting`, `WSSD` 0.40 | 12 | 241 | same | 0.0500 |
| `water_biting`, `WSSD` 1.50 | 12 | 222 | same | 0.0500 |
| `deep_water`, frozen | 107 | 251 | `depth ≥ max_rooted_depth` (crop cap) | 0.0803 |
| `deep_water`, `WSSD` 0.40 | 107 | 248 | same | 0.0803 |
| `deep_water`, `WSSD` 1.50 | 107 | 244 | same | 0.0803 |

**The loop is inert on both live runs, and not by luck: root growth has already stopped
for a different cited reason long before `DVS` reaches 1** (day 12 vs 251; day 107 vs
251, at `DVS = 0.0442`). The rooted-depth trajectory is untouched, so `WSFD` never
re-enters the water state on these runs. That is a statement about *these two scenarios*,
not a general safety property — a run whose roots are still extending at anthesis would
close the loop, and the bound `1 + WSSD` is what keeps it from running away.

### Predicted diff — WRITTEN BEFORE REGENERATION

From the monkeypatch prototype (`proto_wsfd.py`), with `WSSD = 0.40`:

| | frozen | with `WSFD` |
|---|---|---|
| `water_biting` `thermal_time` | 2027.7279 | **2449.1632** (+20.8 %) |
| `water_biting` peak leaf C | 0.762103 | **0.694088** (−8.9 %) |
| `water_biting` storage C (grain) | 0.245173 | **0.326621** (+33.2 %) |
| `water_biting` max rooted depth | 0.220334 | 0.220334 (unchanged) |
| `deep_water` `thermal_time` | 2027.7279 | 2343.3628 (+15.6 %) |
| `deep_water` peak leaf C | 5.164899 | 5.130878 (−0.7 %) |
| `deep_water` storage C | 2.878496 | 3.595607 (+24.9 %) |
| every frozen scenario, every stock and aux | — | **bit-identical** |

**Leaf carbon falls and grain rises.** That is the mechanism, not a surprise: faster
development means anthesis arrives sooner, so less of a water-limited season is spent
building canopy and more of it is spent filling grain. A drought-escape response is
exactly what `WSSD > 0` encodes.

⚠ The prototype multiplied the *increment* (`rate · dt · WSFD`) where the build
multiplies the *rate* (`rate · WSFD · dt`). Float multiplication is not associative, so
those agree only because `dt = 1.0` exactly on every run here. The build re-measures
rather than trusting the table above.

### The prediction held, to every digit

Re-measured on the shipped tree (`verify_build.py`): `water_biting` 2027.7279 →
**2449.1632** / 0.762103 → **0.694088** / 0.245173 → **0.326621**; `deep_water`
**2343.3628** / **5.130878** / **3.595607**; and every frozen scenario — plus `n_limited`,
`drought`, `potato`, `day_neutral` — **bit-identical in every stock and every aux** against
the same run with `wssd = None`.

The regenerated golden splits **14 moved / 6 bit-identical**, and the split is itself the
finding:

| | |
|---|---|
| bit-identical | `soil_water`, `subsoil_water`, `condensate`, `water_vapor`, both boundary sinks, **and `rooted_depth`** |
| moved (carbon) | leaf **−13.6 %**, root **−7.9 %**, stem **+4.2 %**, **grain +33.2 %**, litter −4.6 %, microbial −3.2 %, humus −0.5 %, chamber CO₂ +8.9 % |
| moved (N, O) | plant N −0.9 %, litter N −5.0 %, microbial N −4.4 %, humus N −2.6 %, soil N +0.00 %, O₂ −0.01 % |

**Not one water amount moved**, and the reason is structural rather than lucky: potential
transpiration is a Penman–Monteith function of *weather*, not of leaf area, so a changed
canopy does not change the water draw; and the roots had already stopped on the
dry-subsoil gate by day 12, so the `DVS ≥ 1` stop never binds. `rationed == 0`,
`events == ()`, loss-sink empty, `n = 305` and the seed unchanged.

### Two pinned probe values moved, and their CLAIMS were re-measured rather than re-tuned

`test_soil_fractionation` and `test_stem_reserves` both pin a `water_biting` CO₂ trough
(0.085006 → 0.088509 on the re-basing → **0.093346** here — three values, two unrelated
causes, all on 2026-08-12). Every *claim* those tests make was re-measured and still holds:
sizing 2 still beats the frozen tail (0.143329 > 0.093346), still pays for it in plant
(veg ratio 0.535004/2.143987), and the trough is still bit-identical with and without the
stem reserve. `sealed_chamber`'s trough did not move on either occasion, which is the
control.

### What actually gates this build

Nothing automatic gates the *frozen* side, so the pins had to be built and then broken.
**Seven Python mutations, all caught** (`mutate.py`): the multiply moved inside the
vegetative branch; the `+ 1` dropped; the deficit inverted; the forward-only bound dropped;
`WSFD` given its own threshold; potato silently inheriting wheat's coefficient; a hardcoded
`ground_area`.

⚠ Two of those seven were **initially uncaught and the tests had to be strengthened**, both
for the same reason — *a pin that reads a default is tautological*. `assert
built.ground_area == DEFAULT_SCENARIO.ground_area` is `1.0 == 1.0`, so a hardcoded `1.0`
passed it; the pin now constructs an off-default plot (3.5 m², `EXTR` 0.09, `wssg` 0.42).
And every WSFD test built its aux process *directly*, so wiring `drought=` unconditionally
in `plants.py` — the break that would give potato a coefficient [F] does not publish —
passed the entire file. That pin now walks the registry `build_season` produced.

**Five Rust mutations, all caught** (`mutate_rust.py`) — and the run also *measures* the
claim written into their docstrings: with the new pins skipped, **four of the five leave
the rest of the Rust suite green**, including replacing Eqn 15.8 with a constant `1.0` and
deleting the multiply outright. No Rust scenario is water-limited, so the pins have to
manufacture the condition, exactly as the soil-layers build's did. ⚠ Here too the first
attempt was too weak: a season-level pin asserting only *direction* (`accelerated > off`)
stayed green when `WSFD` was given the wrong threshold and when it was gated to the
vegetative phase — a stressed season is mostly vegetative, so it still accelerates, just by
the wrong amount. **A direction assertion is not a value assertion**, and neither is a
bound; a second pin now calls `evaluate` directly with an exact expected value, past
anthesis, on a 3 m² plot.

### The manifest really is untouched — checked, not assumed

Regenerating it produces no substantive diff. The only bytes that move are a pre-existing
mismatch between the generator (which writes `—` escapes) and the committed file
(literal em-dashes) — equal as parsed JSON, which is why nothing tests it. Reverted rather
than committed as unrelated churn, and noted here because it means **the manifest on disk
is not byte-reproducible from its own generator**.

### Gates

2321 Python tests, 21 Rust test binaries, all 101 cross-port parity checks, `ruff`,
`pyright`, `cargo clippy -D warnings` — green. `git diff src/simcore/` empty.

### Where `wssd` lives, and why potato must not inherit it

`wssd` goes on `SeasonScenario` beside `wssg`, defaulting to **0.40** (wheat), with
`POTATO_SCENARIO` overriding to `None`. ⚠ The tension is recorded rather than hidden:
in [F], Table 15.1 is indexed by **crop**, so `wssd` is crop data and arguably belongs in
`params/crops/*/phenology.yaml`. It sits on the scenario because `wssg` — *the same table
row* — already does, and splitting one row of one table across two homes would be worse
than either choice. Consequence: `param_files` stays unchanged, as it did for
`wssg`/`MAI`/`DRAINF`.

Potato opts out because **[F] Table 15.1 has no potato row** — an absence in the source,
not a modelling preference. The day-neutral crop keeps 0.40: it *is* the winter-wheat
param files with the two phenology gates switched off.

## What changes

| piece | shape |
|---|---|
| `phenology.drought_development_factor(wsfg, *, wssd)` | new pure function, [F] Eqn 15.8; rejects `wssd < −1` |
| `phenology.DroughtDevelopmentParams` | `wssd` + the `FTSW` geometry `WSFG` needs (`wssg`, `soil_extractable_water`, `ground_area`) |
| `ThermalTimeAccumulation` | `+ drought`, `+ soil_water`, `+ rooted_depth_aux`; a third rate multiplier, **ungated by phase** |
| `plants.build_plants` | wires it from the scenario when `wssd is not None` |
| `SeasonScenario` | `+ wssd: float \| None = 0.40`; `POTATO_SCENARIO` → `None` |
| Rust mirror | `science.rs` factor + `system.rs` wiring, same optionality |
| goldens | **`water_biting_state.json` only** |
| manifest | **nothing** — see the warning above |

## Sources

* **[F]** Soltani, A. & Sinclair, T.R., *Modeling Physiology of Crop Development, Growth
  and Yield*, CABI. **Ch. 9** (Eqns 9.3–9.6, Box 16.2's `CropLAI` — the two leaf-area
  branches), **Ch. 15** (Eqns 15.3–15.9, Table 15.1 and its caption, Table 15.2,
  Fig. 15.3), **Ch. 16** (Box 16.2's VBA listing — the authoritative order of
  operations). Table 15.1 read off a **page render** (`pdftoppm -r 170`, PDF p. 210 =
  printed p. 195), never the column-scrambled `pdftotext` output.
