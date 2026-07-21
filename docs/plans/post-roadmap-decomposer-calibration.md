# Post-roadmap — Scope (B): decomposer calibration (biosphere unfreeze)

**Status: COMPLETE (2026-07-21). Biosphere unfrozen + re-frozen; two carbon-side
decomposer rates moved from above-range to top-of-range; mineralization investigated
and deliberately NOT moved. 11 goldens regenerated, both ports level, both affected
manifests regenerated.**

The "return side" mirror of scope (A)'s canopy-collapse finding. Scope (C) rounds 5–6
kept sharpening a scope-B target: the whole decomposer cluster runs **fast** vs the
literature. This is that calibration.

## The charge

Scope (C)'s diagnosis (recorded in the param files, first-hand from primary sources):

- `decomposition_rate` 0.02/day = **7.3/yr** — ~1.5× ABOVE the max of Zhang 2008's
  293-litter global database (0.006–4.993/yr) and ~1.8× above the fastest ecosystem
  Olson 1963 measured. **Outside the entire observed distribution.**
- `microbial_respiration_rate` 0.05/day = **18.25/yr** — above *every* published
  constant for a nominally-microbial pool; 2.5× CENTURY active-SOM, **28× RothC BIO**.
- `mineralization_rate` 0.03/day = 0.21/wk — ~3.9× the Stanford & Smith mean, 2.2×
  above the fastest of their 39 soils.

## What was measured before deciding (the diagnosis phase)

Read-only probes (`M:/claud_projects/temp/decomposer_*.py`), no frozen file touched:

1. **Pool identity** (advisor's blocking reframe — the cited range depends on what the
   pool represents). `litter_carbon`/`litter_n` are the same fresh-residue pool;
   `microbial_carbon` is a genuine model-semantics choice (RothC BIO 0.66/yr vs CENTURY
   active-SOM 7.3/yr — an 11× spread).
2. **The mineralization "gap" is not a rate target.** Its range (Stanford & Smith) is
   **soil organic N₀**, a different pool from our fresh-residue `litter_n`. And the
   model's N (kg) and C (mol) are on an **uncoupled, non-physical scale** (`plant_n0`
   set high to force `f_N≡1`), so the litter "C:N" is not a physical ratio. The number
   is also **behaviorally inert** — moving it leaves every carbon/closure trajectory
   byte-identical. The real gap is a missing **form** (the flow is positive-only and
   cannot immobilize N; immobilization `litter_n → microbial_n → soil_n` is a documented
   deferred seam). Verdict: **record the form gap, don't move the rate.**
3. **The carbon decomposers are LOAD-BEARING for chamber closure.** The sealed chambers'
   only sustained carbon source is recycled CO₂ (litter → microbial → CO₂ →
   photosynthesis). Slowing the rate starves the loop; the plant can't refill the seed
   bank and **annual re-sow crashes** (`storage_c < 0.16` seedling). Measured survivable
   floor ~0.004/day; **RothC-BIO (0.002/day) crashes** at frozen sizing.
4. **The resize test (advisor's steady-state note), MEASURED, does NOT rescue slow k.**
   At cyclo-stationary state `k·C_litter* = senescence input`, so annual CO₂ return is
   k-independent; k only sets the standing pool size. So a slow k should close if litter
   starts near the larger `input/k` equilibrium. **It doesn't:** RothC-slow × litter
   ×5 survives re-sow but trips perennial `rationed=5` (policy violation); ×10/×20
   explode (152 → 257). Because `flux = k·C_litter`, a bigger litter pile at fixed k
   means bigger **absolute** turnover → the plant grows huge, microbial respiration
   drains the sealed O₂ floor to 0, and the backstop rations. **The binding constraint
   is the chamber's O₂ headroom, not the litter size.** RothC-BIO is measured infeasible
   at any chamber size.

## The resolution (why "just calibrate to real science" has a real answer)

"Real science" hands you a **range**, not a point (microbial 0.66–7.3/yr, an 11× spread
by pool definition). Only the **fast end** of the cited range closes cleanly in this
chamber; central values (Zhang median 0.30/yr; RothC 0.66/yr) starve the loop. So
calibrating *to* real science means picking the fastest defensible cited anchor. **The
calibration moves the cluster from above-range to top-of-range** — a genuine correction —
while recording that the closure constraint forces the fast edge.

## The values (measurement-locked, not preference)

| param | was | now | = /yr | anchor |
|---|---|---|---|---|
| `decomposition_rate` | 0.02 | **0.011** | 4.0 | Olson 1963 [A] fastest ecosystem (African tropical forest); near Zhang [B] max |
| `microbial_respiration_rate` | 0.05 | **0.016** | 5.84 | active-SOM range (CLM5 [F] 5.9/yr, CENTURY K5 max [A] 7.3/yr) |
| `mineralization_rate` | 0.03 | **0.03 (unchanged)** | — | form gap recorded; wrong-pool + non-physical + inert |

**micro was forced to CLM5's 5.9/yr, not CENTURY's 7.3/yr max, by measurement:**
`decomp 0.011 / micro 0.02` trips perennial `rationed=4`; only `micro 0.016` is clean.

### The honest framing (ruling B — advisor-reviewed, deliberately not softened)

- **decomp 4.0/yr is defended as BULK litter of the fastest measured ecosystem** — NOT
  by relabeling the single litter pool a "labile fraction" (that re-reading stays
  **refused**, per `decomposition.yaml`'s own discipline).
- **micro's move DELIBERATELY re-anchors** to the active-SOM lineage over the strict
  RothC "Microbial Biomass" pool (0.66/yr, "ours by name and definition"). This is the
  re-anchoring scope C flagged as a trap; scope B makes it **openly**: the value moves
  *down* into a cited range, the load-bearing reason is **closure**, and the **residual
  is recorded** — 5.84/0.66 = **~8.8× above the strict microbial-biomass reading**.
- **"Runs fast" is REDUCED, not RESOLVED.** The real habitat residue (wheat straw,
  C/N~80) decays nearer Zhang's 0.30/yr median than the tropical extreme. The honest
  claim is "now in-range at the fast edge, large physical residual documented," **not
  "now it's validated real science."**

## Edge checks BEFORE regenerating (advisor-required — the trajectory sits near an edge)

All at the locked pair `decomp 0.011 / micro 0.016`, against the real frozen code:

| check | result |
|---|---|
| 3 standalone chambers (exact pair) | `rationed=0` (micro 0.02 rejected — trips perennial) |
| station greenhouse / lighting / harvest | `rationed=0` |
| sealed_station (Tier-2 multi-year) | `rationed=0, events=()` |
| period class (Tier-0 **exact**, hard-pinned False) | both stay **period-1**; perennial → fixed point 0.994 (no flip) |
| 328-yr stress (`-m slow`) | `rationed=0`; 23/24 — the `>1.0` guard now 0.994 (see below) |
| crossport sensitivity | biosphere 6.47e-15 (was 6.7e-14), greenhouse 2.9e-15, station 5.2e-15 — all ≪ band 1e-11 |

**The one non-green was not a closure failure** — it is a trajectory-pinned numeric
guard. `test_stress_perennial_fixed_point_sustained` asserted the sustained perennial
fixed point `> 1.0`; the calibration shrinks the closed-chamber plant ~19% (1.222 →
0.994), so the guard was updated to `> 0.9` (still ~3.9× the 0.253 dead-plant baseline;
CO₂min 0.039, storage 0.308 ≫ 0.16 seed — robustly alive), with the reason documented in
place. This is the advisor's predicted cost of the calibration, recorded not hidden.

## The cascade (what moved)

- **6 frozen biosphere goldens**: sealed_chamber, perennial_chamber, consumer_chamber,
  perennial_long_horizon, consumer_long_horizon, drift_summary.
- **1 non-frozen biosphere golden**: water_biting (sealed chamber).
- **4 station goldens**: greenhouse, harvest, lighting, sealed_station. (crew / eclss /
  cabin / water_recovery / power / thermal / station / sealed_energy_drift are biosphere-
  free and byte-identical. Open season + n_limited are open-field — no decomposer flows —
  byte-identical.)
- **Rust**: `biosphere_params.txt` regenerated (only the two hexfloats changed).
- **Manifests**: biosphere (3 param hashes — incl. mineralization's comment-only edit —
  + 6 golden hashes) and station (4 golden hashes). `sibling_params.txt` byte-identical.
- **Purity**: `git diff src/simcore/` empty; `git diff src/` limited to the two
  `params/*.yaml` value edits + the mineralization comment (a deliberate biosphere
  science unfreeze, not a port/consumer edit).

## The sealed_station wrinkle (the one edge check the pre-flight missed, and its fix)

The station closure pre-flight checked `rationed==0` but not sealed_station's **pre-golden
stationarity gate**. At the new rates that gate (`is_stationary`, `bound=1.0`, on
peak-total-organic-C) failed: year-1→2 diff **7.85 > 1.0**. Not rationing, not amplifying
(the run *converges* — the non-amplifying slope check passes) — only the amplitude cap.

**Mechanism (measured, per-stock).** The plant is flat from year 1 (leaf 8.583, storage
24.4 — lamp-lit, cabin-regulated CO₂, so *not* decomposer-limited). The ramp is the **soil
pools**: litter peak 23.7→73.8→76.3, microbial 15.2→30.0→33.2, settling by year 3. From
`annual_reset`, each year-end sheds the whole plant (`old_veg+grain−seedling ≈ 60` mol C)
into litter — so **year 1 is a one-time spin-up**: the only year with no prior annual
plant-dump already in the soil. The calibration enlarged the soil equilibria ~2–3× (litter
∝ 1/k_decomp, microbial ∝ 1/k_microresp), pushing that spin-up across the year-1→2 boundary
(the old fast-decomposer transient was ~0.09, inside the bound).

**Fixes measured, not guessed.** Co-adapting the initial condition (advisor's first choice)
**fails**: `litter_carbon0` up to 32 only moves the diff 7.85→4.88 (the year-2 peak is
dominated by the plant-dump, not the initial), and `microbial_carbon` starts hardcoded at 0
(not settable without a scope-creep code change). Horizon probe: yr3 `rationed=0`; yr4
`rationed=0`; **yr5 rations**; yr6 collapses (both pre-existing at old rates).

**The fix (advisor-endorsed): `SEALED_STATION_YEARS` 3→4 + `is_stationary(transient=1)`.**
`transient=1` skips the documented year-1 soil-establishment spin-up; horizon 4 gives two
genuinely-settling post-spin-up diffs `[0.329, 0.012]` (year 3→4 = 0.012 proves
convergence; horizon 3's lone 0.329 is itself still-settling). The amplitude **bound stays
1.0** — this is a spin-up skip, not a relaxed bound (amplifying drift past year 1 still
fails). Applied identically to the regen `_gate` and `test_sealed_station_stability` (whose
stale "~0.09 transient" comment was corrected). Honest margin note: rationing onset is
**year 5** (1-year margin beyond the pinned horizon), not year 6.

**The calibration did NOT narrow the stability window — measured, not assumed.** OLD
(0.02/0.05) and NEW (0.011/0.016) both run `rationed==0` at year 4 and both ration at
year 5 with the **identical** count (112667) — so the year-5 rationing onset is
pre-existing *and* rate-independent (a beyond-horizon artifact of the tiling/reset
schedule, not the decomposer rates). The calibration's only sealed_station effect is the
**longer soil-settling transient** (year-1 spin-up now spans a full year → horizon 3→4 +
spin-up skip), *not* a shorter stable window. OLD could itself have run at horizon 4; it
used 3 because the fast-decomposer soil settled within year 1.

## The residual (a permanent, explained pin, not a to-do)

The decomposer cluster is now **in-range at the fast edge**, not central. "Runs fast" is
**reduced, not resolved** — a physical residual remains (real residue decays slower than
the tropical-forest anchor; the strict RothC microbial-biomass reading is 8.8× below
ours), forced by the chamber's closure/O₂-headroom constraint. `mineralization_rate`'s
form gap (no immobilization) is likewise recorded, not built. Both are documented in the
param `source:` strings and here — the honest deliverable is a calibration with its gaps
cited, exactly as ruling B intends.
