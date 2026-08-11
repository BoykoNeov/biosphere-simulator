## **Scope (B): decomposer calibration — the carbon cluster moved above-range → top-of-range**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE (2026-07-21) — biosphere unfrozen + re-frozen; the "return side" mirror of scope
(A)'s canopy collapse, and the target scope (C) rounds 5–6 kept sharpening.**
`docs/plans/post-roadmap-decomposer-calibration.md`. `decomposition_rate` 0.02→**0.011**/day
(7.3→4.0/yr; Olson 1963's fastest ecosystem, near Zhang's 293-litter max — was ~1.5× ABOVE
the observed ceiling); `microbial_respiration_rate` 0.05→**0.016**/day (18.25→5.84/yr;
CENTURY/CLM5 active-SOM). `mineralization_rate` **investigated, NOT moved** — its cited
range is the WRONG POOL (soil-N₀ vs our fresh-residue N), the N scale is non-physical (kg N
uncoupled from mol C; `plant_n0` set high to force `f_N≡1`), and the rate is behaviorally
**inert** (moving it leaves carbon/closure byte-identical); the real gap is a missing
immobilization **FORM** (a documented deferred seam), recorded not fitted. **THE ANSWER to
the user's "we want real science — what's the problem?": real science gives a RANGE, not a
point** (microbial 0.66–7.3/yr, 11× by pool), and the closure physics picks **which cited
value is viable — the fast edge**: central literature values (Zhang median 0.30/yr, RothC
0.66/yr) starve the recycled-CO₂ loop and crash annual re-sow (measured; RothC-BIO
infeasible at any litter size — resizing overshoots into rationing). So **"runs fast" is
REDUCED (above-range → top-of-range), NOT RESOLVED** — the residual is documented (real
residue like wheat straw decays nearer the median; the strict RothC microbial-biomass
reading is **8.8× below** ours). micro was forced to CLM5's 5.9/yr, **not CENTURY's 7.3/yr
max, BY MEASUREMENT** (0.02 trips perennial rationing at the calibrated decomp) — the value
pair is measurement-locked, not preference. **Ruling-B honesty (advisor-enforced,
deliberately not softened)**: decomp 4.0/yr is bulk litter of the fastest measured
ecosystem, NOT a relabeled "labile fraction" (that re-reading stays refused); micro's move
DELIBERATELY re-anchors active-SOM over strict RothC "Microbial Biomass" — the trap scope C
flagged, made **openly**, load-bearing reason = **closure**, residual recorded. **THE DEEPER
FINDING (advisor)**: the closed-chamber references were **propped up by unphysically-fast
decomposers**; literature-real rates leave every closed demo measurably **more marginal** —
the closed-chamber plant shrinks **~19 %** (perennial fixed point 1.222→0.994;
`test_biosphere_stress` + `test_decade_stability` liveness floors `>1.0`→`>0.9`), the decade
CO₂-min guard now skips the sow-in transient because the sustained minimum (0.0548) barely
clears its 0.05 floor, and **sealed_station** needs `SEALED_STATION_YEARS` **3→4 +
`is_stationary(transient=1)`** — the same co-adaptation truth as increment-1's
consumer-chamber 2× and the resize probe. **The sealed_station wrinkle (the pre-flight
missed its stationarity gate, checking only `rationed==0`)**: the calibration enlarged the
soil-pool equilibria ~2–3× (litter ∝ 1/k_decomp, microbial ∝ 1/k_microresp), so the **year-1
soil-establishment spin-up** (the `annual_reset` plant-dump, ~60 mol C into litter — year 1
is the only year with no prior dump in the soil) now spans a full year (year-1→2 diff
**7.85** vs the old ~0.09). **Co-adapt-the-IC FAILED** (advisor's first choice):
`litter_carbon0` up to 32 only moves the diff to 4.88 (the year-2 peak is dominated by the
dump, not the initial), and `microbial_carbon` starts hardcoded 0 (not settable). So the fix
is `transient=1` (skip the documented spin-up year — **NOT** a relaxed amplitude bound;
bound stays 1.0) + horizon 4 for two genuinely-settling post-spin-up diffs `[0.329, 0.012]`.
The **year-5 rationing / year-6 collapse are MEASURED pre-existing AND rate-independent**
(OLD and NEW both `rationed==0` at year 4, both ration at year 5 with the **identical**
112667 count — a beyond-horizon tiling/reset artifact), so the calibration lengthened the
**settling transient**, not the **stable window** (an advisor catch — I had asserted
"pre-existing" for year-5 while only measuring it NEW-side; measuring OLD confirmed it).
**Cascade**: 6 frozen biosphere goldens (sealed/perennial/consumer/2× long-horizon/drift) +
water_biting + 4 station goldens (greenhouse/harvest/lighting/sealed_station); biosphere
manifest (3 param hashes incl. mineralization's comment-only edit + 6 golden hashes),
station manifest (`sealed_station_years` + 4 golden hashes); Rust `biosphere_params.txt` (2
hexfloats) + `SEALED_STATION_YEARS` 3→4. **The pre-flight missed the value-pinned tests**
(loader `== 0.02/0.05`, decade-stability thresholds tuned to the old 1.22 plant) — caught by
the full suite (`-m slow` is **opt-OUT**, so bare `pytest` runs the slow tier; **the
"non-slow suite" is a misnomer** — advisor). `git diff src/simcore/` empty; full suite
**1981 passed**, crossport **101**, cargo clippy/test green. Advisor-reviewed at every
hinge.
