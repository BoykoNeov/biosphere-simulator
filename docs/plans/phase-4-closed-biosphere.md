# Phase 4 — Closed Biosphere (the reference domain, frozen)

**Status: DRAFT — not started. Phase 3 exits.**
Phases 0, 0.5, 1, 2, and 3 are complete and regression-pinned
(`docs/plans/phase-{0-engine-skeleton,0.5-numerical-foundations,1-single-producer,2-closed-chamber,3-modular-biosphere}.md`).
This plan **locks the load-bearing Phase-4 decision** (what "conservation-stable over
decade-scale runs" *operationally means* for an emergent limit cycle, P4.1) and **enumerates**
the process steps as forward-pointers, each to be designed just-in-time — the Phase-1/2/3
rhythm. Per that rhythm and the advisor's standing "design the load-bearing decision in full
and review it before enumerating process steps," P4.1 is designed in detail below; the
remaining steps are sketched only. **No step is started until P4.1 is advisor-reviewed.**

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

- **(a) Mass-conservation drift — the hard invariant.** The per-step gate already bounds the
  *per-step* residual to `BALANCE_ATOL/RTOL`. The decade-scale concern is **accumulation**: does
  `Σ`(per-step residual) over `N` steps walk the **total** of each conserved quantity away from
  its `step-0` value? For each quantity `q ∈ {CARBON, OXYGEN, NITROGEN, WATER}`, track
  `total_q(n) = Σ`(amount · composition[q]) over **all** in-system + boundary stocks (the
  `_total` sum the perturbation tests already use). Pass criterion:
  `|total_q(N) − total_q(0)| ≤ DRIFT_ATOL(N)`, with `DRIFT_ATOL(N)` set to the **worst-case
  accumulation bound** (`≈ N · BALANCE_ATOL`, the triangle-inequality ceiling) rather than a
  hand-tuned magic number. This is the literal *"conservation of matter holds over decade-scale
  runs."* (At `N ≈ 3050` and `BALANCE_ATOL ~ 1e-12`, the ceiling is `~3e-9` — comfortably tight;
  Phase 2 already measured float-exact `~1e-13` totals at 305 steps.)

- **(b) Limit-cycle stationarity — the emergent-behavior invariant.** Define a small vector of
  **per-year scalar summaries** (candidates: peak `leaf_c`, min `carbon_pool`, year-end
  `consumer_carbon`, the period-2 phase). Across years `k = 1…Y`, the cycle is **stationary**
  iff these summaries reach an attractor and **hold** it: `|summary(k) − summary(k−1)|` is
  bounded and **non-increasing past the transient** — i.e. **not amplifying** (creeping toward
  blow-up / `annual_reset` raising) and **not decaying** (creeping toward extinction). This is
  direction-of-trend, not a magic equality — the Step-4/5/6/7 anti-flakiness discipline. The
  5-year run is already a stable period-2 attractor (Phase 3 de-risk evidence), so the
  expectation is **stationarity confirmed**; the metric exists to *catch* a slow drift that
  5 years is too short to reveal.

- **(c) No structural failure over the full horizon — the closure invariant carried.** The
  Phase-3 closure asserts, now held for the **entire** decade-scale horizon: `rationed == 0`
  every step (kinetics, not the Euler backstop), `events == ()` (no extinction), carbon
  loss-sink stays `0.0`.

**The integrator question — PROBE Euler first; escalate only on evidence (the Step-4
discipline).** The biosphere runs **Euler-daily** (P1.P3); RK4 exists in `simcore` since Phase
0.5 but the biology has never used it. Decade-scale truncation error *might* drift the limit
cycle — but escalating to RK4 is **not free**:
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
- **Core pure** (stdlib only); `git diff src/simcore/` **empty** unless a *named,
  advisor-reviewed* integrator change is forced by P4.1's probe (the sole allowed exception).
- **Determinism** bit-identical within a build; integer step count (`t = n·dt`, never `+=`);
  canonical (flow-id) order on every reduction.
- **Conservation asserted every step**; `rationed == 0` from kinetics (Euler backstop count `== 0`
  on goldens; under RK4 a needed scale is a hard error).
- **Extinction conserves mass**; death routes to **litter**, not the loss-sink.
- **Parameters are data** (YAML + schema); no hardcoded coefficients — including the new
  drift tolerances, which are *derived* (`N · BALANCE_ATOL`), not tuned.

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
- **No new core surface** unless P4.1's probe *forces* an integrator change — and then only that
  one change, advisor-reviewed, with regenerated goldens + provenance.

## API additions (additive; ideally **zero core change**)
- `domains/biosphere/` or a new reporting/lab module: a **decade-scale driver wrapper** (reuses
  `run_perennial` — just a larger `steps`) + **drift-metric computation** (per-year summary
  vector; the `total_q` mass tracker reusing the perturbation `_total` fold).
- `config/`: the **derived** drift tolerances (`DRIFT_ATOL(N) = N · BALANCE_ATOL`), surfaced as
  data, not a literal.
- `tests/`: long-horizon golden(s) + drift-metric assertions; a marked-slow 100k-step stress.
- `docs/biosphere-reference.md` + a freeze manifest (scenario → golden-hash, param-file hashes,
  integrator + dt).

## Step sequence (sketched; each designed just-in-time, P4.1 first)
1. **Decade-scale Euler probe + drift instrumentation (P4.1).** Run perennial + consumer to
   ≥10 yr; compute the three drift metrics (a/b/c); **decide Euler-holds vs escalate**. The
   de-risk that gates everything — designed in full before code, advisor-reviewed.
2. **(Conditional) integrator escalation.** *Only if Step 1 drifts:* RK4 for the biosphere, with
   the `annual_reset`×multistage design + the "needed scale is a hard error" precondition check.
   **Skipped entirely if Euler holds** (the expected outcome).
3. **100k-step stability stress (line 211).** The "no drift" stress as a **marked-slow opt-in**
   test; report the residual-accumulation bound vs the `N · BALANCE_ATOL` ceiling.
4. **Canonical golden capture (P4.2).** Pin the long-horizon perennial + consumer goldens + the
   drift-summary golden; re-affirm the four Phase-3 goldens (byte-identical, or
   regenerated-with-provenance if Step 2 changed the integrator).
5. **The freeze contract (P4.3).** `docs/biosphere-reference.md` + the manifest + the unfreeze
   discipline; the formal freeze of the biosphere domain.

## Exit criteria (Phase 4 — "closed biosphere, frozen as reference")
- **Conservation of matter holds over decade-scale runs:** total CARBON/OXYGEN/NITROGEN/WATER
  bounded-drift (`≤ N · BALANCE_ATOL`) over ≥10 yr; the 100k-step stress run + its bound reported.
- **The emergent limit cycle is stationary** (bounded, non-amplifying, non-decaying) over the
  decade horizon; `rationed == 0`, `events == ()`, loss-sink `0.0` the whole way.
- **The reference integrator + dt are locked** (Euler-held or RK4-escalated, **with evidence**).
- **Canonical golden scenarios captured:** long-horizon goldens pinned + the drift-summary golden;
  the four Phase-3 goldens byte-identical (or regenerated-with-provenance).
- **The biosphere domain is frozen:** `docs/biosphere-reference.md` + manifest naming the frozen
  surface + the unfreeze discipline.
- `git diff src/simcore/` **empty** (or exactly the one named, advisor-reviewed integrator change).
- Full suite green; ruff + pyright clean.
- **Next: Phase 5 — Sibling Domains** (power / thermal / atmosphere-ECLSS / crew), each verified
  standalone against its own references before it touches the now-frozen biosphere (lines 313–325).
