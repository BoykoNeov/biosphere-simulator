# Phase 4 — Closed Biosphere (the reference domain, frozen)

> ⚠ **Superseded measurement (2026-07-20, scope (B) increment 1):** every "perennial holds
> a genuine **period-2** cycle" claim below was true of the biosphere Phase 4 froze, but that
> cycle was a property of the **broken canopy regime**. Adding vernalization + photoperiod
> closed the canopy, flattened the year-to-year return map below unit gain, and the perennial
> converged to a **period-1 fixed point** — so **both** chambers are now period-1. The
> historical numbers below are kept as the record of what Phase 4 measured; the current
> contract is `docs/biosphere-reference.md` and the change is in
> `docs/plans/post-roadmap-oracle-match.md`.

**Status: PHASE 4 COMPLETE — Steps 1–5 done (decade probe + drift instrumentation P4.1;
integrator escalation SKIPPED; the 100k-step biosphere slow-drift stress; the canonical
long-horizon golden capture P4.2; the freeze contract P4.3). The biosphere domain is
FROZEN AS THE REFERENCE. Phase 4 exits → Phase 5 (sibling domains).**

**Step 5 outcome (2026-06-30) — THE BIOSPHERE IS FROZEN.** Three artifacts, all
boundary-side (zero `simcore` change): **(1)** `docs/biosphere-reference.md` — the
human-readable **freeze contract**: what is frozen (the locked **Euler + `dt = 1.0 day`**;
the flow set; the 13 clean-room biosphere param files; the canonical scenarios + goldens;
the `LONG_HORIZON_YEARS = 15` horizon; the weather forcing), the stability evidence it
rests on (pointers to Steps 1/3, not restated), and the **unfreeze discipline**
(advisor-reviewed unfreeze → regenerate the affected goldens *and* the manifest → provenance
note; `git diff src/simcore/` empty unconditionally). The Phase-0 engine-skeleton demo
(`flows.py` / `demo.yaml`) is scoped **out**, by name (frozen separately by its own Phase-0
goldens). **(2)** `docs/biosphere-reference.manifest.json` — the **generated**
machine-readable manifest naming the same surface, incl. the **17-class flow set + the
aux set, both derived from freshly assembled registries** (the union over the open field +
the three chambers — never hand-listed). **Note (corrected post-advisor):** gross
assimilation is **not** an aux — it is a recomputed *quantity* inside the shared carbon
budget (the `GrossAssimilation` flow was dissolved at P1 Step 11), entering via the
`Allocation` flow; the **only** aux process is the thermal-time/DVS accumulator, now frozen
via `aux_set` (the third 'wired-into-a-registry' axis the advisor flagged, closed with zero
core change — `registry.aux_processes` is a public read-only property).
**Newline-normalized sha-256 provenance** for every param/golden/forcing file. **(3)**
`tests/test_freeze_manifest.py` (7 tests + `__main__` regen) — the **completeness gate**.
**The load-bearing design decision (advisor-steered):** the gate checks **completeness —
the frozen *sets* vs the live tree** (param-file set, the registry-derived flow + aux sets,
horizon == `LONG_HORIZON_YEARS`, file existence) + a **teeth test** (an unfrozen file
actually trips it), NOT byte-rehash. Rationale: all 13 param files feed ≥1 canonical golden
load-bearingly, so a *value* change already moves a golden and fails there — re-hashing
would be redundant, **non-reproducible under `autocrlf`** (Windows), and would freeze
*formatting* not *values*. So **value enforcement stays with the scenario goldens; the gate
owns the one thing they're blind to — a param/flow/aux added to the tree but wired into no
golden.** Manifest hashes are **provenance** (newline-normalized → cross-platform stable),
regenerated on a deliberate unfreeze. The four Phase-3 + three Phase-4 goldens stay
**byte-identical** (no regen); full suite green (1010 passed incl. the 27 slow-marked, 1
oracle skipped); ruff + pyright clean (incl. two pre-existing comment-only E501 reflows in
`scenario.py` / `test_decade_stability.py`); `git diff src/simcore/` empty. **Phase 4
EXITS.**

**Step 4 outcome (2026-06-30) — CANONICAL LONG-HORIZON GOLDENS CAPTURED.**
`tests/test_regression_long_horizon.py` (7 tests, ~2 s, module-scoped fixture runs each
15-yr scenario once) pins the closed biosphere at the **decade-scale horizon**
(`LONG_HORIZON_YEARS = 15`, a new `scenario.py` constant — single source of truth shared by
the long-horizon golden, the decade probe (`DECADE_YEARS` now aliases it), and the Step-5
freeze manifest). Three new goldens, two kinds: **(1)** the final-`State` hex-float
snapshots of the 15-yr perennial + consumer runs (`perennial_long_horizon_state.json`,
`consumer_long_horizon_state.json`) via `sim_io`, exactly as the Phase-3 5-yr goldens at
5 yr — the marginal value is real (post-yr-5 regressions in stocks the summaries don't
surface: soil_water, N pools, the `thermal_time` aux). **(2)** the **drift-summary golden**
(`drift_summary.json`, the genuinely new Phase-4 artifact) — the per-year limit-cycle
summaries (peak `leaf_c` for both; year-end `consumer_carbon` for the consumer) as hex-float
vectors + the period class (perennial `is_period_2` True, consumer False), the *stability*
signature that catches a cycle-*shape* regression a single endpoint snapshot cannot. The
mass-drift `max|d_q|`/`slope` numbers are **deliberately NOT byte-pinned** (~1e-11 round-off
noise — a meaningful leak moves the vectors + snapshot anyway; the bound checks live in the
decade/stress tests). Each golden bakes the **pre-golden closure gate** (`rationed == 0` /
`events == ()` / loss-sink `0.0` before pinning); round-trip load-back + `__main__`
`_regenerate()` mirror the existing discipline; hex-float via the stdlib `float.hex` (no new
`sim_io` API). The **four Phase-3 goldens are re-affirmed byte-identical** (their own
regression tests, zero new bytes — Step 2 was skipped, so nothing forced a regen).
`git diff src/simcore/` empty; full suite green (976 fast + 27 slow), ruff + pyright clean.

**Step 3 outcome (measured, 2026-06-30) — EULER HOLDS AT 100k+, NO DRIFT.**
`tests/test_biosphere_stress.py` (24 tests, marked `slow`, ~30 s) runs **both** closed
scenarios (`PERENNIAL_CHAMBER_SCENARIO`, `CONSUMER_CHAMBER_SCENARIO`) Euler-daily to **328
whole years (100,040 steps, "100k+")** — the **real slow-drift detector** (axis (b) is
horizon-blind to anything slower than the run). **Streaming-chunked** (one year per chunk,
carrying the exact `State` forward — bit-identical to a continuous run, *verified*; folds
`total_quantity` + per-year summaries out and discards states → ~11 MB, not the ~0.8 GB
retain-all). Euler-only: Step 1 already locked Euler *with evidence*, so RK4 (~4× cost) is
not re-run. **(a)** mass drift over the 22×-longer run: worst `max|d_q| ≈ 7.4e-11` (WATER;
CARBON ~2e-14) and worst `|slope| ≈ 7.5e-16` — the **slope stayed flat at ~3× machine-ε
(no growth)** while `max|d_q|` grew ~linearly *at that machine-ε slope* (`slope·N ≈ 7.4e-11
≈ max|d_q|`): deterministic round-off in the fold, **not** a leak. Both detector bounds
hold with margin (~13× under `MASS_DRIFT_ABS_BOUND`, ~4.5 orders under
`MASS_DRIFT_SLOPE_BOUND`), ~7 orders under the `N·BALANCE_ATOL ≈ 1e-4` ceiling — the bounds
**span both horizons**, no re-derivation (drift.py PROVENANCE updated with the 328-yr
numbers). **(b)** stationarity now **confirmed, not merely holding**: per-year `peak leaf_c`
bounded + non-amplifying + non-collapsing, and the **period class is sustained the full
horizon** — perennial holds genuine **period-2** (320 yr of strict alternation, gap ~0.07),
consumer holds its **period-1** fixed point (gap ~3.4e-5) with stationary, alive biomass.
**(c)** `rationed == 0`, `events == ()`, loss-sink `0.0` on **every** one of the 100,040
steps, both scenarios — the strongest closure statement. Assertion-only (no golden — Step 4
captures the *decade* snapshot); wall-clock + round-off-vs-ceiling **logged** (not capped);
`git diff src/simcore/` empty (drift.py is a domain module); the four Phase-3 goldens
untouched. Step 2 (integrator escalation) **stays SKIPPED** — Euler held at 100k+.

**Step 1 outcome (measured, 2026-06-30) — EULER LOCKED, with evidence.** `domains/biosphere/drift.py`
(pure-stdlib instrument: `total_quantity` promoting the `_total` fold + the three drift axes) +
`tests/test_drift.py` (18 synthetic instrument tests) + `tests/test_decade_stability.py` (31 probe
tests, ~7.7 s). Both closed scenarios run Euler **and** RK4 to a 15-yr horizon (4575 steps):
**(a)** mass drift JITTERS at √N round-off (worst `max|d_q| ≈ 3.3e-12` WATER, slope `≈ 7.3e-16` —
machine-ε, no trend), ~6–9 orders under the `N·BALANCE_ATOL ≈ 4.6e-6` ceiling; derived detector
bounds `MASS_DRIFT_ABS_BOUND = 1e-9` / `MASS_DRIFT_SLOPE_BOUND = 1e-11` sit *between* the noise
floor and a 1e-9/step real-bug leak. **(b)** peak-`leaf_c` cycle bounded + non-amplifying +
non-collapsing in both scenarios; the settled **period class differs (measured, not assumed):
perennial is a genuine period-2 cycle** (two branches, gap ~0.07), **consumer is period-1** — the
herbivore damps the producer oscillation to a **fixed point** (settled adjacent gap ~3e-5). Both
fully settled by ~yr 10. **(c)** `rationed == 0`,
`events == ()`, loss-sink `0.0` every step — for **both** integrators. The **RK4 cross-check
empirically retired** the two preconditions (survives `annual_reset`×multistage; no arbitration
hard-error) and **structurally agrees** with Euler → escalation (Step 2) is **skipped**. Zero
`simcore` change; the four Phase-3 goldens untouched; no new golden (capture is Step 4). The
`is_stationary`/`non_collapsing` split was an advisor correction folded in (see § P4.1, axis (b)).
Phases 0, 0.5, 1, 2, and 3 are complete and regression-pinned
(`docs/plans/phase-{0-engine-skeleton,0.5-numerical-foundations,1-single-producer,2-closed-chamber,3-modular-biosphere}.md`).
This plan **locks the load-bearing Phase-4 decision** (what "conservation-stable over
decade-scale runs" *operationally means* for an emergent limit cycle, P4.1) and **enumerates**
the process steps as forward-pointers, each to be designed just-in-time — the Phase-1/2/3
rhythm. Per that rhythm and the advisor's standing "design the load-bearing decision in full
and review it before enumerating process steps," P4.1 is designed in detail below; the
remaining steps are sketched only. **P4.1 has been advisor-reviewed** and revised accordingly:
the mass-drift axis is split into a structural **ceiling** + a measured-round-off **detector**
(the advisor's blocker — `N·BALANCE_ATOL` alone is a blind detector); the integrator-escalation
path is corrected to **zero core diff** (RK4 already ships in `simcore`); and axis (b)'s
**horizon-limited** reach + the period-2 **discrete** check are stated. The first *code* step
(Step 1) is now **designed in full and advisor-reviewed** below (§ *Step 1 — full design*) —
the decade-scale Euler probe + drift instrumentation; the remaining steps (2–5) stay
just-in-time forward-pointers.

## Relationship to the roadmap + the central reframing

**Goal (roadmap lines 304–312):** *Complete the reference domain. A genuinely closed
biosphere, conservation-stable over multi-year sealed runs, exhibiting emergent behavior.
"This is the proving ground. The engine is now trusted enough to carry other domains."*
**Exit criteria (roadmap):** biosphere domain **frozen as the reference**; **golden biosphere
scenarios captured**; **conservation of matter holds over decade-scale runs**.

**The reframing — Phase 4 adds NO new science.** Phase 3 already delivered *"a genuinely
closed ecosystem exhibiting emergent behavior"* (Phase 3 exit, roadmap line 303): the closed
chamber spans producer + decomposer + consumer trophic levels with a sustained, emergent
period-2 limit cycle, every-step four-quantity conservation, and `rationed == 0` from
kinetics. So Phase 4 introduces **no new flow, no new trophic level, no coupled
(Lotka-Volterra/Holling) dynamics**. Its three deliverables are about **trust and duration**,
not biology:

1. **Duration** — prove the closed biosphere is conservation-stable *and* its emergent limit
   cycle is **stationary** over **decade-scale** (≥10 yr) and toward **100k-step** (roadmap
   line 211, "100k+ steps, no drift") horizons — not merely the 5-year runs Phase 3 validated.
2. **Capture** — select and pin the **canonical golden scenarios** that *define* the reference.
3. **Freeze** — formally **freeze** the biosphere domain (flow set, params, scenarios,
   integrator + dt) as **THE reference** that Phase 5 sibling domains are verified against and
   that the (post-integration) Rust port ports verbatim. Roadmap line 7: *"We port a stable
   multi-domain engine, not an evolving one."* This is the Phase-0 *"freeze the engine
   architecture before scientific complexity appears"* discipline (line 164) applied one level
   up — to the biosphere **science**.

**The headline is a stability proof + a freeze discipline, not new physics.** The biosphere
stops being a moving target.

**Horizon arithmetic (grounding the targets).** A season is `len(weather)` ≈ **305 steps/year**
(Euler-daily; the perennial reset first fires at `n = 305`). The current longest validated runs
are **5 years (~1525 steps)** (`PERENNIAL_CHAMBER_YEARS = CONSUMER_CHAMBER_YEARS = 5`). So:
**decade-scale ≈ 10 yr ≈ 3050 steps**; the line-211 **100k-step** stress ≈ **~328 years**.
Phase 4 extends the validated horizon from 1525 steps to ≥3050, with a marked-slow stress
toward 10⁵.

## Locked decisions (Phase 4)

### P4.1 — "Conservation-stable over decade-scale runs" means a **stationary emergent limit cycle** + a **bounded-residual mass ledger**, measured by a small set of named drift metrics; the **reference integrator is locked by probe, Euler-first**. *(The load-bearing decision — designed in full below; advisor-reviewed before any process step.)*

"No drift" (line 211) is the subtle one: it is **both trivially falsifiable and trivially
satisfiable** for a system whose *entire purpose* is to oscillate. You cannot assert "the state
stops changing" — the emergent period-2 limit cycle is the deliverable, not a defect. So Phase 4
must define **drift operationally for a limit cycle**, along three independent axes:

- **(a) Mass-conservation drift — the hard invariant.** The decade-scale concern is
  **accumulation**: does the **total** of each conserved quantity walk away from its `step-0`
  value? For each `q ∈ {CARBON, OXYGEN, NITROGEN, WATER}`, track `total_q(n) = Σ`(amount ·
  composition[q]) over **all** in-system + boundary stocks (the `_total` fold the perturbation
  tests already use). The subtlety (advisor-flagged): the per-step **gate** residual
  (`BALANCE_ATOL`, what `assert_conserved` tolerates) is **not** the drift mechanism. Our flows
  are **structurally balanced per-stock by construction**, so `total_q` is conserved up to
  **float round-off**, not up to `BALANCE_ATOL` — these differ by orders of magnitude
  (`BALANCE_ATOL = BALANCE_RTOL = 1e-9` in `quantities.py`; Phase 0/2 measured round-off `~2e-13`
  at 305 steps — ~4 orders tighter than the gate). So a single `N·BALANCE_ATOL` test is a loose
  **ceiling** masquerading as a **detector**: at `N ≈ 3050` it is `~3e-6`, ~7 orders above the
  round-off it claims to watch — a real mass-drift bug at `~1e-9` would sail straight through.
  Axis (a) therefore has **two tiers, both derived (neither hand-tuned):**
  - **Ceiling (never breach):** `|total_q(N) − total_q(0)| ≤ N · BALANCE_ATOL` — the hard assert,
    the triangle-inequality worst case. Loud, loose, and structural; if it ever trips, the flow
    legs themselves are unbalanced.
  - **Detector (the real test):** fit `total_q(n) − total_q(0)` vs `n` and assert the **slope is
    flat at round-off scale** — sub-linear, no systematic growth. The tight bound is **derived
    from the round-off accumulation the Step-1 probe actually measures** over the decade run, not
    from `BALANCE_ATOL`. This is what makes *"conservation of matter holds over decade-scale runs"*
    a claim with teeth. *(Step 1 produces exactly this slope data — so this tier is finalized by
    the probe, not pre-guessed here.)*

- **(b) Limit-cycle stationarity — the emergent-behavior invariant.** Define a small vector of
  **per-year scalar summaries** (candidates: peak `leaf_c`, min `carbon_pool`, year-end
  `consumer_carbon`). Across years `k = 1…Y`, the cycle is **stationary** iff these summaries
  reach an attractor and **hold** it, tested on **same-phase** differences
  `|summary(k) − summary(k−2)|` (for a period-2 cycle the *adjacent* difference is the cycle
  amplitude and does not vanish). **Stationarity is TWO orthogonal axes, not one (advisor —
  forced by the math, IMPLEMENTED):**
  - **`is_stationary` = bounded + non-amplifying** (least-squares slope of `|diff|` ≤ tol — a
    direction-of-trend test, not a magic equality). This catches **amplifying** drift (creeping
    toward blow-up / `annual_reset` raising — `|diff|` grows, *diff-detectable*) and passes a
    settled *or still-converging* cycle (the lock does NOT require a reached attractor).
  - **`non_collapsing` = a level/floor check** (every summary stays above a floor). This is the
    **only** detector of creeping **decay toward extinction**, which is **diff-blind**: geometric
    decay `s[k] = C·rᵏ` shrinks `|diff|` *identically* to a cycle converging to a finite
    attractor — the only difference is the *limit* (zero vs nonzero), a property of the **level**,
    not the diffs. So `is_stationary` alone admits decay; the floor check is **mandatory, not
    stylistic**.
  The **period-2 structure** is a third, **discrete structural check** (`is_period_2` — the cycle
  *remains* period-2), kept **separate** from the scalar stationarity vector — a phase index is
  not a scalar you can call "non-increasing." **Horizon caveat (advisor):** axis (b) is **horizon-limited** — it
  cannot see a drift slower than the run: a 30-yr decay is invisible in 10 yr. At decade scale,
  axis (b) mostly **confirms** the Phase-3 attractor holds; it is the **100k-step stress**
  (Step 3, ~328 yr) that actually does slow-drift detection. State the **detectable drift-rate
  floor** the horizon affords (`~1/Y` per year) rather than claiming stationarity absolutely.
  The 5-year run is already a stable period-2 attractor (Phase 3 de-risk evidence), so the
  decade expectation is **stationarity confirmed**.

- **(c) No structural failure over the full horizon — the closure invariant carried.** The
  Phase-3 closure asserts, now held for the **entire** decade-scale horizon: `rationed == 0`
  every step (kinetics, not the Euler backstop), `events == ()` (no extinction), carbon
  loss-sink stays `0.0`.

**The integrator question — PROBE Euler first; escalate only on evidence (the Step-4
discipline).** The biosphere runs **Euler-daily** (P1.P3). Decade-scale truncation error *might*
drift the limit cycle. **Escalation is cheap mechanically but not free behaviorally** (advisor):
`simcore/integrator.py` **already ships `Rk4Integrator`** alongside `EulerIntegrator` (both since
Phase 0.5, both satisfying the same `Integrator`/`Substepper` protocols), and the biology has
never used it. So escalating is a **domain-side choice of which integrator class to instantiate
— `git diff src/simcore/` stays empty**; core purity is **not** at stake. What *is* at stake is
two **behavioral preconditions**, to verify not assume:
- RK4 interacts with the discrete `annual_reset` driver transform. (Resets land *on* step
  boundaries via `run_season`'s `reset` hook, between whole steps, so multistage substeps never
  straddle a reset — this is benign, but must be stated and tested, not assumed.)
- The CLAUDE.md invariant: *"under RK4+ a needed arbitration scale is a **hard error**
  (positivity comes from kinetics)."* The kinetics are first-order donor-controlled and already
  give `rationed == 0` under Euler; RK4 should preserve that, but it is a precondition to verify,
  not assume.

Therefore Phase 4 **does not pre-commit to an integrator change.** Step 1 is a **measurement**:
run Euler to decade-scale, compute (a)/(b)/(c), and **decide on evidence**. Keep Euler if it
holds (the likely outcome). Escalate to RK4 **only** if Euler demonstrably drifts, and only then
behind a dedicated reset×multistage design + the hard-error check. **Either way the reference
integrator + dt are LOCKED by the end of Phase 4** — you cannot freeze a reference whose
integrator is undecided.

### P4.2 — The canonical golden set is the **existing four goldens re-affirmed + extended to a decade-scale horizon**, not a new scenario zoo. *(Capture, not invention.)*

The four Phase-3 goldens — **open season**, **sealed chamber**, **perennial chamber**,
**consumer chamber** — already span producer / decomposer / consumer and open / closed. Phase 4
invents **no new ecosystem**. It (a) **re-affirms** the four as byte-identical (the standing
`git diff src/simcore/` empty + no-regeneration discipline), and (b) pins the **decade-scale**
perennial + consumer runs as the canonical **long-horizon** goldens — the runs that actually
exercise stability. "Golden biosphere scenarios captured" = a hex-float long-horizon snapshot
**plus** the **drift-metric summary** pinned as a golden, so any future regression in *stability*
(not just per-step values) is caught.

### P4.3 — "Frozen as reference" is a **documented freeze contract + manifest**, not a code lock. *(Discipline, boundary-side, zero core change.)*

Operationally: a `docs/biosphere-reference.md` + a **freeze manifest** that names the frozen
surface — the **flow set**, the **param files** (with content hashes), the **canonical
scenarios** + their **golden hashes**, the **locked integrator + dt** — and states the
**unfreeze discipline**: a change to any frozen item requires a documented, advisor-reviewed
unfreeze, regenerated goldens, and a provenance note (the Phase-1 PCSE/clean-room provenance
rigor, applied to our own reference). This is **boundary-side docs + a manifest**; **zero
`simcore` change** — the Phase-3 `git diff src/simcore/` empty discipline carries.

### Carried Phase-0/0.5/1/2/3 invariants that constrain Phase 4
- **Core pure** (stdlib only); `git diff src/simcore/` **empty** — *unconditionally*. Even a
  RK4 escalation reuses the existing `simcore.Rk4Integrator` (a domain-side instantiation
  choice), so it touches **no** core file. There is no Phase-4 path that edits `simcore/`.
- **Determinism** bit-identical within a build; integer step count (`t = n·dt`, never `+=`);
  canonical (flow-id) order on every reduction.
- **Conservation asserted every step**; `rationed == 0` from kinetics (Euler backstop count `== 0`
  on goldens; under RK4 a needed scale is a hard error).
- **Extinction conserves mass**; death routes to **litter**, not the loss-sink.
- **Parameters are data** (YAML + schema); no hardcoded coefficients — including the new
  drift tolerances, which are *derived*, not tuned: the **ceiling** `N · BALANCE_ATOL` (structural)
  and the **detector** slope bound (from the Step-1 measured round-off accumulation).

## Scope

### In scope (Phase 4)
- A **decade-scale (≥10-yr) run harness** + **drift instrumentation** — the three P4.1 metrics:
  mass-conservation accumulation, limit-cycle stationarity, structural-failure guards.
  Boundary/test-side.
- A **probe** of Euler stability to decade-scale (and a marked-slow stress toward 10⁵ steps);
  **locking the reference integrator + dt**.
- **Canonical golden capture**: long-horizon perennial + consumer goldens + the drift-summary
  golden; re-affirm the four Phase-3 goldens.
- The **freeze contract** (`docs/biosphere-reference.md` + manifest) + the **unfreeze discipline**.
- **Performance**: keep the long-horizon run tractable — a gated decade golden + a marked-slow
  100k stress; if pure-Python at 10⁵ steps with the every-step gate is too slow for the suite,
  the stress is opt-in (`-m slow`), and the slowness is **logged, not silently capped**.

### Explicitly deferred (do NOT build in Phase 4)
- **No new science / no new trophic level / no coupled (Lotka-Volterra/Holling) dynamics** — the
  Phase-3 capstone explicitly deferred these; Phase 4 is **stability + freeze**, not biology.
- **No sibling domains** (power / thermal / atmosphere-ECLSS / crew) — **Phase 5** (lines 313–325).
- **No Rust port** — moved to after station integration (line 7).
- **No new core surface, ever** — a RK4 escalation reuses the shipped `simcore.Rk4Integrator`
  (zero core diff); the only artifacts it forces are domain-side (the integrator-selection call,
  regenerated goldens + provenance), not a `simcore/` edit.

## API additions (additive; ideally **zero core change**)
- `domains/biosphere/` or a new reporting/lab module: a **decade-scale driver wrapper** (reuses
  `run_perennial` — just a larger `steps`) + **drift-metric computation** (per-year summary
  vector; the `total_q` mass tracker reusing the perturbation `_total` fold).
- `config/`: the **derived** two-tier drift tolerances — the ceiling `N · BALANCE_ATOL` and the
  round-off slope bound — surfaced as data, not literals.
- `tests/`: long-horizon golden(s) + drift-metric assertions; a marked-slow 100k-step stress.
- `docs/biosphere-reference.md` + a freeze manifest (scenario → golden-hash, param-file hashes,
  integrator + dt).

## Step sequence (sketched; each designed just-in-time, P4.1 first)
1. **Decade-scale Euler probe + drift instrumentation (P4.1). — COMPLETE.** Ran perennial +
   consumer to 15 yr; computed the three drift metrics (a/b/c); **Euler holds → locked, with
   evidence** (see the Step-1 outcome block at the top). The de-risk that gates everything —
   designed in full below (§ *Step 1 — full design*), advisor-reviewed, now implemented.
2. **(Conditional) integrator escalation. — SKIPPED.** Step 1 did not drift and the RK4
   cross-check structurally agreed, so escalation is unneeded. (Had it been needed: RK4 for the
   biosphere with the `annual_reset`×multistage design + the "needed scale is a hard error"
   precondition check.)
3. **100k-step stability stress (line 211). — COMPLETE.** `tests/test_biosphere_stress.py`
   (marked-slow, opt-out): both closed scenarios Euler-daily to **328 whole years (100,040
   steps)**, streaming-chunked (memory-safe, bit-identical to a continuous run). The **real
   slow-drift detector** (axis (b) is horizon-blind to anything slower than the run): mass
   drift slope stayed flat at machine-ε (no leak), period class sustained the full horizon,
   closure held every step. Round-off slope vs the `N · BALANCE_ATOL` ceiling **logged**. See
   the Step-3 outcome block at the top. Euler held → Step 2 stays skipped.
4. **Canonical golden capture (P4.2). — COMPLETE.** `tests/test_regression_long_horizon.py`
   pins the 15-yr (`LONG_HORIZON_YEARS`) perennial + consumer final-`State` goldens + the
   **drift-summary golden** (per-year cycle summaries + period class — the stability
   signature; mass-drift round-off NOT pinned). The four Phase-3 goldens re-affirmed
   byte-identical (Step 2 skipped → no regen). Pre-golden closure gate + load-back +
   `__main__` regen mirror the existing discipline. See the Step-4 outcome block at the top.
5. **The freeze contract (P4.3). — COMPLETE.** `docs/biosphere-reference.md` (freeze
   contract + unfreeze discipline) + `docs/biosphere-reference.manifest.json` (generated;
   the registry-derived flow set, the 13 param files, the 7 scenarios → goldens, the locked
   Euler/`dt`, newline-normalized provenance hashes) + `tests/test_freeze_manifest.py` (the
   **completeness** gate — frozen sets vs the live tree, NOT byte-rehash; value enforcement
   stays with the scenario goldens). The formal freeze of the biosphere domain. See the
   Step-5 outcome block at the top. **Phase 4 exits.**

## Step 1 — full design: decade-scale Euler probe + drift instrumentation (P4.1) — COMPLETE

*Implemented as designed (`drift.py` + `test_drift.py` + `test_decade_stability.py`); the
measured outcome is the **EULER LOCKED** block at the top of this plan. The design below is
retained as the as-built record. The de-risk that gates all of Phase 4 — designed in full and
advisor-reviewed before any code (the Phase-1/2/3 rhythm). A **measurement** step: run the two
closed scenarios to
decade scale, compute the three P4.1 drift axes, and **decide Euler-holds-vs-escalate on
evidence**. Boundary/test-side only — `git diff src/simcore/` stays empty; no new golden
(capture is P4.2/Step 4); the four Phase-3 goldens untouched.*

### Deliverables (three artifacts, all domain/test-side)
1. **`domains/biosphere/drift.py`** — pure-stdlib drift instrumentation over a trajectory
   (`list[State]`). Promotes the `_total` fold (today duplicated in three test files —
   `test_perennial_chamber`, `test_perturbations`, and implicitly the ledger test) to a
   shared `total_quantity(state, q)`, and adds:
   - **(a)** `mass_drift_trace(states, q) -> list[float]` = `[total_q(n) − total_q(0)]`;
     `drift_slope(trace)` (least-squares slope vs `n`); `max_abs(trace)`.
   - **(b)** `year_summaries(states, year, summary_fn) -> list[float]` (one scalar per year,
     reusing the `states[y*year:(y+1)*year+1]` segmentation the perennial tests already use);
     `same_phase_diffs(summaries, period=2)` = `[s[k] − s[k−2]]`; `is_stationary(diffs)` =
     bounded + non-increasing-past-transient.
   - **(b-discrete)** `period_phase(...)` — the period-2 structural check, kept **separate**
     from the scalar vector.
   It imports `simcore.state`/`simcore.quantities` + `domains.biosphere` only — a domain
   module, **not** core. Stdlib-only.
2. **`tests/test_drift.py`** — validates the *instrument* on **synthetic** traces, independent
   of the biology:
   - **slope recovery:** a known linear-drift trace recovers its slope;
   - **discrimination (the teeth — advisor fold 2):** a synthetic trace with a deliberate leak
     at **real-bug scale (~1e-9/step)** must **fail** the detector, while round-off-scale jitter
     must **pass** — proving the bound sits *between* round-off and real-bug scale, not merely
     that slope-of-a-line works;
   - **stationarity (the mandatory split — advisor, IMPLEMENTED):** `is_stationary` (bounded +
     non-amplifying) **cannot** detect creeping decay: geometric decay `s[k] = C·rᵏ` shrinks the
     same-phase diffs *identically* to a cycle converging to a finite attractor — the only
     difference is the *limit* (zero vs nonzero), a property of the summary **level**, not its
     diffs. So a stationary/converging trace **passes** `is_stationary`; an **amplifying** one
     **fails** it (`|diff|` grows — diff-detectable); a **decaying** one **passes** `is_stationary`
     (diff-blind) but **fails** the separate `non_collapsing` level/floor check, which owns
     extinction. The test asserts *both* detectors explicitly, so it documents which axis owns
     decay. *(This corrects the earlier "a decaying one fails `is_stationary`" — decay is routed
     to `non_collapsing`, a third orthogonal axis the design had conflated into the diff axis.)*
3. **`tests/test_decade_stability.py`** — the actual probe on **both**
   `PERENNIAL_CHAMBER_SCENARIO` and `CONSUMER_CHAMBER_SCENARIO`, asserting axes (a)/(b)/(c) +
   the Euler-vs-RK4 **structural** agreement. Module-scoped fixtures (each decade run executes
   once). Normal-speed (≈3050 steps × 2 scenarios × 2 integrators ≈ 12k gated steps — within
   suite budget; the 100k stress is Step 3, marked-slow).

### The three drift axes, made concrete
**(a) Mass-conservation drift — two tiers, shape measured first (advisor fold 3).**
`d_q(n) = total_q(n) − total_q(0)` for each `q ∈ {CARBON, OXYGEN, NITROGEN, WATER}`.
- **Ceiling (never breach):** `max|d_q| ≤ N · BALANCE_ATOL ≈ 3050 · 1e-9 ≈ 3e-6` — the
  triangle-inequality worst case; if it trips, the flow legs are unbalanced.
- **Detector (the real test):** the run is **bit-deterministic**, so `d_q(n)` is a *fixed
  measured trace*, not a random variable. **Measure its shape first** (bounded-oscillating /
  √N walk / linear trend) before fixing the primary statistic. For a structurally-conserved
  sum the error is most likely **bounded-oscillating or √N — not linearly trending** — so the
  least-squares slope is ≈ machine-ε noise. Slope is still the right **systematic-leak
  signature** (a leak is linear in `n`; round-off is not), so keep it, but **report `max|d_q|`
  alongside** as the directly-interpretable bound and set the slope bound a small margin **above
  the measured noise floor**. *Which is primary (slope vs `max|d_q|`) is decided after seeing
  whether `d_q` trends or jitters.* The bound is a **documented constant with provenance** in
  `drift.py` — **not** config YAML: a regression-guard threshold derived from round-off is part
  of the *test*, not a model coefficient (the `BALANCE_ATOL`-is-a-`simcore`-constant precedent;
  a pydantic schema for one diagnostic number is the speculative generality this codebase keeps
  rejecting). The provenance records the **measurement procedure** (scenario, horizon, observed
  `max|d_q|` + slope), so a future toolchain change can re-derive it — the goldens' regeneration
  discipline.

**(b) Limit-cycle stationarity — period-2-aware.** Per-year scalar summaries (peak `leaf_c`,
min `carbon_pool`, year-end `consumer_carbon`). **The catch:** for a period-2 cycle
`summary(k) − summary(k−1)` does **not** vanish — it is the cycle *amplitude*, oscillating
between the two branches. So stationarity is tested on **same-phase differences**
`summary(k) − summary(k−2)`, asserted **bounded and non-amplifying past the transient** by
`is_stationary` (least-squares slope of `|diff|` ≤ tol). **Stationary ≡ bounded +
converging-or-converged** — explicitly **not** "must have reached the attractor": a
still-converging-but-bounded cycle (amplitude shrinking toward a finite attractor) satisfies
non-amplifying + bounded and is freezable. **`is_stationary` is diff-blind to decay** (geometric
decay shrinks `|diff|` identically to convergence), so the **mandatory** companion
`non_collapsing` (a level/floor check) owns the extinction axis — the split is forced by the
math, not stylistic. The period-2 *structure* (the cycle remains period-2) is the separate
**discrete** `is_period_2` check.

**(c) Closure carried over the full horizon.** `rationed == 0`, `events == ()`, carbon
loss-sink `0.0` on **every** step of the decade run (both scenarios) — the Phase-3 closure
asserts, now held for the entire horizon.

### Horizon — budget 15–20 yr, do NOT gate the lock on full convergence (advisor fold 4)
At year 5 the period-2 gap is ~0.74 vs ~1.00 (≈26 % on an O(1) signal — the documented
mid-transient). Ten years gives only ~4 same-phase differences per branch — thin to call a
monotone trend, and the gap won't close by year 10. So **budget a ~15–20-yr working horizon up
front** (stated, not discovered mid-probe), with the adaptive extension **capped at a few
decades** so Step 1 doesn't bleed into the 100k stress (Step 3). **Critically: the lock criterion
is bounded + non-amplifying, which a still-converging cycle satisfies — the lock does NOT require
a reached attractor.** The Euler-holds verdict rests on:
- (a) conservation — rock-solid (integrator-independent: balanced legs conserve regardless);
- (c) closure — rock-solid;
- (b) bounded / non-amplifying — satisfied by a converging cycle;
- RK4 **structural** agreement (below).
Full convergence to the attractor is **Step 3's (100k) job**, not Step 1's.

### The decide-on-evidence core — Euler-vs-RK4 structural cross-check
Run **Euler** decade → axes (a/b/c). Then run **RK4** decade through the *same* `run_perennial`
(zero core change — `Rk4Integrator` already ships; `run_season` calls `integrator.step_report`,
shared on `_BaseIntegrator`). This RK4 run is a **one-shot cross-check — no golden, no permanent
RK4 instantiation; the shipped code stays Euler.** It also **empirically retires** the two RK4
preconditions the plan flagged (rather than assuming them): that RK4 survives the discrete
`annual_reset`×multistage interaction, and that no needed arbitration scale fires (under RK4 a
needed scale is a hard error — verify it does not occur).

**Agreement is QUALITATIVE / structural, not "within X" (advisor fold 1).** Euler and RK4 differ
by O(truncation), so attractors will **not** match numerically — and "within X" needs an `X` we
cannot principle for an uncalibrated model. Compare **structure**: both stationary, both period-2,
both bounded, both closed.

**Decision:**
- Euler stationary (b) **and** (a)/(c) pass **and** Euler/RK4 structurally agree → **lock Euler,
  with evidence** (the expected outcome). The RK4 cross-check is the *only* thing that
  distinguishes "Euler is fine" from "Euler's truncation error produced a *stably-wrong*
  attractor" — it is the evidence in *lock-with-evidence*, not over-building.
- Euler drifts (amplifying / decaying / period-break) where RK4 does **not** → **escalate**
  (Step 2): Euler's truncation error is the culprit.
- **Both** drift → the drift is **real slow dynamics**, not truncation → re-examine the scenario
  (a science finding, not an integrator choice).

### Why this is zero-core and golden-safe
`drift.py` is a domain module (imports `simcore` read-only + `domains.biosphere`); the tests are
additive; the RK4 run instantiates the already-shipped `simcore.Rk4Integrator`. No `simcore/`
edit, no new golden in Step 1, the four Phase-3 goldens untouched — `git diff src/simcore/` stays
empty.

## Exit criteria (Phase 4 — "closed biosphere, frozen as reference")
- **Conservation of matter holds over decade-scale runs:** total CARBON/OXYGEN/NITROGEN/WATER
  under the **ceiling** (`≤ N · BALANCE_ATOL`) **and** the **detector** (round-off-scale slope, no
  systematic growth) over ≥10 yr; the 100k-step stress run + its measured slope reported.
- **The emergent limit cycle is stationary** (bounded, non-amplifying, non-decaying) over the
  decade horizon; `rationed == 0`, `events == ()`, loss-sink `0.0` the whole way.
- **The reference integrator + dt are locked** (Euler-held or RK4-escalated, **with evidence**).
- **Canonical golden scenarios captured:** long-horizon goldens pinned + the drift-summary golden;
  the four Phase-3 goldens byte-identical (or regenerated-with-provenance).
- **The biosphere domain is frozen:** `docs/biosphere-reference.md` + manifest naming the frozen
  surface + the unfreeze discipline.
- `git diff src/simcore/` **empty** — unconditionally (RK4 escalation, if any, is domain-side).
- Full suite green; ruff + pyright clean.
- **Next: Phase 5 — Sibling Domains** (power / thermal / atmosphere-ECLSS / crew), each verified
  standalone against its own references before it touches the now-frozen biosphere (lines 313–325).
