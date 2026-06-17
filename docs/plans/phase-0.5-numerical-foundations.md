# Phase 0.5 — Numerical Foundations

**Status:** IN PROGRESS — design pass complete (advisor-reviewed); **Steps 1–2 done**.
This is the living plan for Phase 0.5, written design-first before any code, mirroring
how Phase 0 was run. The multi-rate sub-stepping **contract is locked here** (Step 3)
before it is written, as the working style requires. Steps are test-first.

Step 1 (convergence/timestep-sensitivity harness) is complete: `lab/convergence.py`
(`fit_order`/`convergence_order` — the reusable log-log least-squares order estimator,
the first `lab/` module) + `tests/test_convergence.py` gate the observed order on
analytic decay (measured Euler 1.018 ∈ [0.8,1.3]; RK4 4.035 ∈ [3.6,4.3], finest RK4
error 7.6e-11 ≫ the round-off floor), with the fitter pinned independently on synthetic
power-law data. The `lab` package is registered in `pyproject.toml` (out-of-core; outside
the simcore purity gate).

**Goal (roadmap exit):** *"You trust the math before trusting the science."* Add the
numerical machinery and the trust-gates that must exist before any Phase-1 biology:
a high-accuracy validation oracle, multi-rate sub-stepping, a systematic
convergence/timestep-sensitivity study, a long-run (100k+) stability gate, an
edge-case suite, and a performance baseline. **No biology, no new science.**

**Source of truth for the phase sequence:** `roadmap_extracted.txt` (Phase 0.5 =
lines 201–219). Phase 0 is complete and its API is frozen
(`docs/plans/phase-0-engine-skeleton.md`); Phase 0.5 is **additive** — it must not
break the frozen Phase-0 surface.

---

## Relationship to the roadmap + reconciliation

The roadmap's Phase 0.5 deliverables (lines 204–217):

- **Integrators:** Euler, RK4 *(both done in Phase 0)*, **Adaptive RK45 (lab only)**.
- **Multi-rate sub-stepping:** fast domains sub-step inside slow steps; fixed count,
  seeded, deterministic; one master clock; conservation still global.
- **Timestep sensitivity tests:** results converge as `dt` decreases.
- **Stability tests:** 100k+ steps, no drift.
- **Edge cases:** zero stocks, depleted resources, negative-flow attempts, overflow
  protection.
- **Performance baseline:** steps/sec, memory, scaling with stock and domain count.

**Reconciliation — implicit solver is *deferred*, not in Phase 0.5.** The Phase-0
"explicitly deferred" list named *"Implicit solver, adaptive RK45, multi-rate
sub-stepping, perf baseline, stability sweeps → Phase 0.5"*. The authoritative
roadmap's Phase 0.5 (lines 205–208) lists only **explicit** schemes
(Euler / RK4 / adaptive RK45). An implicit/stiff solver needs per-step nonlinear
root-finding (Newton / fixed-point), which means either a forbidden third-party
dependency in `simcore` (violates the core-purity invariant) or a research-grade
hand-rolled solver; the Phase-0 integrator contract already files
positivity-preserving conservative integration (e.g. Modified Patankar–RK) as a
*deferred research item*. **Decision: Phase 0.5 is explicit-only.** Implicit/stiff
integration is deferred to a later numerical phase, to be planned if and when a
stiff scenario actually demands it.

---

## Locked decisions (Phase 0.5)

New decisions, numbered `N1…`, carrying the Phase-0 invariants they depend on. They
constrain the implementation and must not silently drift.

### N1 — Adaptive RK45 is an **out-of-core validation oracle**, not an engine integrator.
Error-controlled adaptive step size means a **variable `dt`**, which breaks Phase-0
**decision #14** (time is an integer step count; `t = n·dt`). It does **not** break
determinism — an adaptive step is still a deterministic function of state — so the
correct framing is "#14, not #7." Because it cannot honor `t = n·dt`, RK45 is **not**
a `simcore.Integrator`; it lives **outside `simcore`** (package `lab/`), carries its
own float `t` and `dt` history, and is explicitly excluded from the
integer-clock / deterministic-replay contract. Its role is to generate a
high-accuracy reference trajectory for the convergence study (analogous to PCSE as
an *offline oracle*, never the shipped engine). It may import the core's
flow/registry machinery; since flows return `dt·rate` (the increment-form contract),
RK45 recovers `rate = leg / dt` per stage evaluation. Conservation is
**dt-independent** (it compares total mass before/after), so RK45 may still reuse the
`simcore.conservation` gate.

### N2 — Multi-rate sub-stepping is a **driver in pure `simcore`** that *preserves* #14.
Multi-rate **keeps** `t = n·dt` if `State.n` counts the **slow (master) step** and
the fast sub-steps are **internal** — amounts perturbed, `n` unchanged — exactly how
RK4 stage states already keep `n`. So the multi-rate driver is pure-stdlib core
(`simcore/multirate.py`), unlike RK45. *This #14-placement asymmetry — multi-rate
preserves the integer clock, RK45 breaks it — is precisely why one is core and the
other is lab.* The driver layers **above** the existing fixed-step integrators; the
frozen `Integrator.step(state, env, dt)` Protocol is untouched.

### N3 — The multi-rate partition is **per-flow** (rate-class), not per-domain.
"Domains step at different rates" (roadmap line 147) is the *concept*; the
*mechanism* must be per-flow, because a **cross-domain flow** (e.g. `Harvest`,
biosphere→boundary — the roadmap's own coupling mechanism) has no well-defined
single-domain rate. The partition is therefore expressed at scenario-assembly time
as **two disjoint flow sets over the shared stock dict** (a slow `Registry` and a
fast `Registry`), or equivalently a `FlowId → rate-class` map. **Cross-domain flows
are assigned a single rate-class explicitly by the scenario** (the assembler
decides). The Phase-0.5 multi-rate **test scenario keeps fast and slow domains
coupled only through shared stocks** (every flow single-domain — the roadmap's
"domains meet only at shared stocks" model), so the assignment is unambiguous; the
cross-domain-flow assignment rule is stated in the contract but its *automatic*
inference is out of scope.

### N4 — Multi-rate uses **operator splitting**; default **Strang** (2nd-order).
Within one master step `dt`, the slow and fast operators are **sequenced (split)**,
not applied simultaneously. **Lie splitting is 1st-order global even when the fast
sub-steps use RK4** — the split caps the composite order, the exact silent
order-reduction the increment-form contract warns against. **Default: Strang
splitting** (symmetric half-slow / full-fast / half-slow → 2nd-order, one extra slow
evaluation). Every sub-operation is internally balanced (a scaled balanced delta is
still balanced), so **conservation is exact** and asserted at the **composite
slow-step boundary**.

**Order — to realize Strang's 2nd order, *both* operators must be RK4.** The
composite global order is `min(splitting_order, every sub-integrator's order)`.
Strang's splitting order is 2, but the only sub-integrators that exist are Euler
(order 1) and RK4 (order 4), so: Strang + **RK4 on both** operators →
`min(2,4,4) = 2`; Strang + **Euler anywhere** → `min(2,1) = 1` (silently collapses to
Lie — the exact order-reduction this phase exists to catch). A Euler slow half-step
therefore does **not** save work for free; it forfeits the 2nd order Strang was
chosen for. **The multi-rate convergence test asserts the *actual* order** and pins
the sub-integrator (Strang+RK4(both) → 2; Strang+Euler → 1) — never a higher order
the split cannot deliver.

**Honest trade-off — multi-rate caps at 2nd order regardless.** Strang's splitting
order is a hard ceiling (the operators' non-commutativity is an O(`dt²`) term no
matter how accurately each operator is sub-integrated), so multi-rate is strictly an
**efficiency** feature that *costs* accuracy versus single-rate RK4 (order 4): you
accept order 2 to avoid stepping a slow domain at the fast `dt`. Higher-order
multirate-RK (which would recover >2nd order across the split) is deferred — this is
why one would ever accept the order-2 path.

### N5 — Atomicity (#11) becomes **per-operator**; the relaxation is explicit.
Phase-0 atomicity is "no flow reads a stock another flow mutated in the same step."
Under splitting this holds **per sub-operation** (all slow flows see one snapshot;
each fast sub-step sees its own snapshot) but **not across the slow/fast split** —
fast flows deliberately read slow-updated stocks. That is the *point* of multi-rate
coupling and is a documented, intentional relaxation, priced as the O(`dt`)
(Strang: O(`dt²`)) splitting error the convergence test measures.

### N6 — The 100k stability gate asserts **long-run boundedness**, not trajectory
fidelity. Per-step conservation is already proven (the always-on gate) and was argued
*length-independent* (the Phase-0 conservation-tol carry-forward). So 100k makes the
length-independence **empirical** and additionally pins what is genuinely **new**:
no `NaN`/`Inf`/overflow over a long run, and bounded state. The scenario must be a
**dissipative system relaxing to a stable fixed point** (an oscillator drifts
secularly under RK4 over 100k and would conflate trajectory-drift with the gate).

### Carried Phase-0 invariants that constrain Phase 0.5
- **Core purity** (#11): `simcore` (incl. the new `multirate.py`) stays stdlib-only;
  the purity AST gate (`tests/test_simcore_purity.py`) must keep passing. `lab/` and
  the perf bench live *outside* `simcore` and outside that gate.
- **Canonical order on every reduction** (#15): every new reduction (sub-step demand
  sums, composite combine) sorts by stable id.
- **Determinism — bit-identical within a build** (#7): multi-rate runs are
  bit-identical and registration-order-independent; RK45 is deterministic but exempt
  from the *bit-identical-across-ports* clause (it is lab-only).
- **Frozen Phase-0 API:** additions only (new modules; additive `substep`/strategy
  methods outside the frozen `Integrator` Protocol).

---

## Scope

### In scope (Phase 0.5)
- Adaptive **RK45** validation oracle (out-of-core `lab/`), with embedded
  error-estimate step-size control.
- **Multi-rate sub-stepping** driver (`simcore/multirate.py`) per the N2–N5 contract.
- **Convergence / timestep-sensitivity** study against the two existing analytic
  references (decay `y₀e^{-λt}`; conserved LV `V`), plus the multi-rate order check
  and (enrichment) RK45-referenced convergence for a non-analytic case.
- **100k-step stability** gate (dissipative relaxation scenario).
- **Edge-case** suite: empty/zero stocks, depleted resources, deliberate over-draw
  (Euler backstop vs RK4 hard-error), negative/overflow handling.
- **Performance baseline**: steps/sec + memory vs stock count, domain count, scheme;
  committed as a tracked reference (`docs/perf-baseline.md`), **non-gating**.

### Explicitly deferred (do NOT build in Phase 0.5)
- **Implicit / stiff solvers** and positivity-preserving conservative integration
  (Modified Patankar–RK) → later numerical phase, on demand.
- **Higher-order multi-rate** (multirate-RK methods, e.g. Günther–Rentrop) that
  preserve >2nd order across the split → deferred; Strang (2nd) suffices for Phase 0.5.
- **Adaptive multi-rate** (error-controlled sub-step counts) → deferred; the count is
  fixed + seeded (roadmap).
- **Sub-stage time-varying forcing** under RK4/RK45 (evaluation at `(n+cᵢ)·dt`) →
  still deferred (Phase-0 step-5 note); Phase-0.5 scenarios stay autonomous.
- Real biology / kinetics → Phase 1.

---

## API additions (additive; frozen Phase-0 surface untouched)

```python
# --- simcore/multirate.py (PURE core; preserves #14) ----------------------
def multirate_step(
    slow: Integrator, fast: Integrator, state: State, env: SourceResolver,
    dt: float, n_sub: int, *, split: Split = Split.STRANG,
) -> StepReport: ...
    # ONE master step: advances State.n by exactly 1; fast flows sub-step n_sub
    # times internally (n unchanged within). Composite conservation asserted at the
    # boundary. slow/fast are integrators over disjoint flow registries sharing one
    # stock dict (N3). Returns a StepReport (state, events, rationed) like step_report.

# additive on the concrete integrators (OUTSIDE the frozen Integrator Protocol):
class _BaseIntegrator:
    def substep(self, state: State, env: SourceResolver, dt: float) -> StepReport: ...
        # like step_report but KEEPS State.n (amounts-only advance) — the building
        # block the multirate driver composes; the driver owns the single n -> n+1.

# --- lab/rk45.py (OUT-OF-CORE oracle; breaks #14 by design, N1) ------------
def rk45_trajectory(
    registry: Registry, state0: State, env: SourceResolver,
    t_end: float, *, atol: float, rtol: float, dt0: float,
) -> Trajectory: ...
    # Dormand–Prince RK45 with embedded error control; carries its OWN float t and
    # adaptive dt. Returns sampled (t, stocks) for use as a convergence reference.
```

`Split` is an enum (`STRANG` default, `LIE` fallback). No change to `Stock`,
`State`, `Flow`, `Quantity`, the resolver, or `Integrator`.

---

## Step 1 design — convergence / timestep-sensitivity study

*Realizes "results converge as `dt` decreases."* **Not blocked on RK45**: the suite
already has two exact references — the analytic decay `y₀e^{-λt}`
(`tests/test_integrator.py`) and the conserved LV invariant `V`
(`tests/test_oscillator.py`, where RK4's 4th-order drop is already measured). Step 1
generalizes these into a **systematic order-measurement harness**
(`tests/test_convergence.py`): run a scheme over a geometric ladder of `dt`, fit the
error-vs-`dt` slope (or assert halving-ratios), and check the **observed order**
matches the scheme (Euler→1, RK4→4) with a banded tolerance. This is the reusable
harness Steps 2–3 plug their schemes into. (The existing oscillator/decay assertions
stay; Step 1 factors out the measurement, it does not duplicate the scenarios.)

**Test plan:** error→0 monotonically as `dt`→0 for Euler and RK4 on the decay
reference; fitted order within band ([0.8,1.3] Euler, [3.6,4.3] RK4); the harness
is reused unchanged by Steps 2–3.

✅ **done** — `lab/convergence.py` (`fit_order`: log-log least-squares slope over the
whole `dt` ladder, NaN/round-off-floor guards; `convergence_order`: the
`error_of_dt`→order convenience) + `tests/test_convergence.py`. The decay scenario is
test-local (repo convention). Measured: Euler 1.018, RK4 4.035 (both comfortably
centered in band); RK4 errors 3.3e-7→7.6e-11 strictly decreasing, well above the f64
floor. The fitter is pinned independently on synthetic `C·dt^p` data (the
discriminating control) and rejects bad input (length mismatch, <2 rungs, non-positive
dt/error). The decision to home the fitter in `lab/` (created now, not deferred to
Step 2) keeps the engine free of analysis tooling and gives Steps 2–3 a clean import.

## Step 2 design — adaptive RK45 oracle (out-of-core)

*Realizes "Adaptive RK45 (lab only)" per N1.* New package `lab/` (sibling of
`simcore`, **outside** the purity gate). `lab/rk45.py` implements Dormand–Prince
(RK45) with the embedded 4th/5th-order error estimate driving step-size control;
carries its own float `t`/`dt` (N1). Reuses `simcore` flow evaluation (recovering
`rate = leg/dt`) and may reuse `simcore.conservation` (dt-independent). Output: a
sampled reference trajectory.

**Test plan:** on the analytic decay, RK45 matches `y₀e^{-λt}` within its requested
`atol`/`rtol` (tolerance-honoring); step-size *adapts* (smaller `dt` where the
solution is stiffer); the oracle conserves mass; used as a reference, fixed-step RK4
converges *toward the RK45 trajectory* on a non-analytic scenario (enriches Step 1
beyond the analytic cases). RK45 is **not** added to the determinism/bit-identical
gates (it is lab-only).

✅ **done** — `lab/rk45.py` (Dormand–Prince embedded 4(5); carries its own float
`t`/adaptive `dt`, N1) + `tests/test_rk45.py`. The derivative is recovered as
`rate = leg / dt` evaluated at `dt = 1.0` (legs *are* rates there, no round-off) —
load-bearing on the same dt-linearity contract RK4 leans on, documented in the module.
The per-step advance is factored into `_rk45_step(...) -> (y_new, error)` so the embedded
estimate's order is pinnable independently; the controller is the scipy-default RMS norm
with `safety=0.9`, factor∈[0.2,10], exponent −0.2; the final step is clipped to land on
`t_end`. **No FSAL** (the one-eval saving isn't worth the accept/reject bug surface in an
oracle) and **no arbitration/extinction** (positivity is the kinetics' job, scenarios are
well-fed); conservation is verified at the call site via `assert_conserved` over rebuilt
states (dt-independent), not inside the oracle.

The tableau is guarded by **two controls** (the Step-1 `fit_order` analogue, because
tolerance-honoring alone can't catch a transcribed coefficient): a *static* consistency
check (each `A`-row sums to its node `c`; `ΣB=ΣB*=1`) and an *empirical* one — the
embedded error estimate is **5th-order** in `dt` (measured 5.03 via `fit_order` over a
single-step ladder). Stated honestly about scope: these pin the **error estimator's**
order (that step control is driven correctly) and the tableau's consistency, *not* the
propagated solution's own nonlinear 5th order — that rests on "it is the standard,
consistency-checked DOPRI5 tableau", reinforced by the linear-scalar decay accuracy and
(tableau-independent) conservation. A direct nonlinear-accuracy pin (RK45's drift in the
nonlinear LV invariant `V`) is **filed-not-built** — beyond Step 2. Measured: tolerance-honoring on analytic decay
(tol 1e-6→err 2.9e-7, 1e-9→3.2e-10, tighter strictly better); step size adapts (LV orbit,
max/min ≈ 27.6 over 67 accepted / 3 rejected steps); mass conserved (LV total-carbon drift
2.3e-13); and **fixed-step RK4 converges toward the RK45 reference at 4th order on the
non-analytic LV scenario** (measured 3.97 — the enrichment beyond Step 1's analytic
cases). For that last gate the finest RK4 rung (err 3.6e-9) is kept ~3800× above the
reference's self-consistency floor (9.3e-13, from a 10× tighter reference) so the fit
measures truncation error, not reference noise — the "stay above the floor" discipline
from Step 1, with the floor being the reference's accuracy rather than machine eps.

**Note — adaptation is demonstrated on LV, not decay.** The Step-2 test plan groups
step-size adaptation under the *decay* case; decay *does* adapt (a few ×) under mixed
atol/rtol, but an LV orbit drives an order-of-magnitude swing in the admissible step, a
sharper, less brittle demonstrator. Same property, regrouped onto the scenario that shows
it best (the LV scenario already had to exist for the convergence-reference gate). The
oracle is **autonomous-only** for Phase 0.5 (forcing read at the template's fixed `n`);
sub-stage time-varying forcing stays deferred.

## Step 3 design — multi-rate sub-stepping driver (THE locked contract)

*Realizes multi-rate per N2–N5 — the load-bearing section.* New pure-core module
`simcore/multirate.py`.

**Mechanism (Strang, default).** One master step of size `dt`, with `n_sub` fast
sub-steps:
1. **slow half** — advance the *slow* operator by `dt/2` (amounts only, `n` kept).
2. **fast full** — `n_sub` sub-steps of the *fast* operator at `dt/n_sub` (amounts
   only, `n` kept), each sub-step seeing the prior sub-step's amounts.
3. **slow half** — advance the *slow* operator by `dt/2` (amounts only, `n` kept).
4. **commit** — `n → n+1` once; assert **composite conservation** over the whole
   master step (`before` = entry state, `after` = post-commit).

Each of 1–3 is an integrator `substep` (the additive amounts-only-advance primitive,
built on the existing `_perturb`/`_apply` split — `_perturb` already keeps `n`). Each
sub-operation is internally balanced, so the composite conserves exactly by linearity
(step 4's assert is belt-and-suspenders + the engine-bug tripwire). The slow and fast
operators are separate `Integrator`s over **disjoint flow registries sharing one
stock dict** (N3). Arbitration/extinction run per sub-operation as in single-rate.

**Determinism.** Canonical flow-id order within each registry (#15); fixed `n_sub`;
the composite is bit-identical and registration-order-independent.

**`Split.LIE`** (fallback): slow-full then fast-full (1st-order). Offered for
comparison/cost; **Strang is the default** and the gated path.

**Test plan:**
- **Equivalence:** with `n_sub = 1` and **all flows fast** (empty slow registry),
  the multi-rate step reproduces the single-rate `step` to the float floor — the slow
  half-steps are no-ops and the one full fast sub-step *is* the single-rate step. (The
  *all-slow* degenerate case does **not** reproduce single-rate under Strang: two
  `dt/2` half-steps differ from one `dt` step at O(`dt²`) — that asymmetry is itself
  worth a note.)
- **Conservation:** composite conservation holds every master step for a coupled
  fast/slow scenario (shared-stock coupling, N3); a deliberately *unbalanced*
  injected sub-delta trips the boundary assert (the tripwire fires).
- **Order (N4):** on a scenario with an analytic/reference solution and the
  sub-integrator **pinned**, Strang+RK4-on-**both**-operators exhibits **2nd-order**
  global convergence (asserted via the Step-1 harness) — *not* 4th; Strang+**Euler**
  exhibits **1st-order** (the silent collapse to Lie); Lie exhibits 1st-order. This is
  the assertion that catches a botched split — and proves the order math of N4.
- **`n` accounting (N2):** after `k` master steps, `state.n == k` regardless of
  `n_sub` (sub-steps are internal; #14 preserved).
- **Determinism:** bit-identical across runs and under flow-registration shuffle
  within each registry (Hypothesis).
- **Speedup sanity:** a fast/slow split with `n_sub > 1` does fewer slow-flow
  evaluations than single-rate at the fast `dt` (the efficiency the feature exists
  for) — a count assertion, not a wall-clock one.

## Step 4 design — 100k-step stability gate

*Realizes "100k+ steps, no drift" per N6.* `tests/test_stability.py`. Scenario: a
**dissipative relaxation to a stable fixed point** (a balanced two-stock exchange
that settles, or the demo extended — its structural well-fed bound is
trajectory-independent, so it stays non-arbitrating for any length and relaxes toward
a depleted/steady state with total carbon conserved). Run 100k+ steps under Euler and
RK4 (and one multi-rate config) and assert what is **new**: every amount stays
finite (no `NaN`/`Inf`/overflow), state stays **bounded** (relaxes, does not blow
up), `rationed == 0` over the whole run, and the always-on conservation gate (running
every step) completes — making the length-independence of the conservation tolerance
**empirical**, not just analytic. (Marked slow; opt-in or a reduced count in the
default suite if runtime warrants.)

**Test plan:** 100k steps complete without `ConservationError`; `max|amount|`
bounded; no non-finite amount; `rationed == 0`; results are reproducible run-to-run.

## Step 5 design — edge-case suite

*Realizes "zero stocks, depleted resources, negative-flow attempts, overflow
protection."* `tests/test_edge_cases.py`. Much machinery exists (arbitration handles
depletion; `Stock.__post_init__` rejects non-finite); Step 5 pins the behaviors
explicitly and adds any missing guard.

**Test plan:**
- **Empty/zero:** an empty registry / empty stocks steps cleanly (no-op,
  `n→n+1`, conserves trivially); a stock at exactly 0 with a withdrawal flow →
  Euler backstop scales the draw to 0 (`scale_f == 0`), stays ≥ 0, conserves.
- **Depletion:** a POOL drawn to exactly empty stays at 0 (no negative) under Euler;
  the same over-draw under RK4 raises `ArbitrationError` (the asymmetry).
- **Negative-flow attempt:** a flow withdrawing more than available is throttled
  (Euler) / hard-errors (RK4) — re-pinning the backstop contract at the boundary.
- **Overflow protection:** a flow driving an amount toward overflow produces `Inf`,
  which `Stock.__post_init__` rejects → a clear error rather than a silent `Inf`
  poisoning the ledger. (If a gap surfaces — e.g. overflow inside a reduction before
  `Stock` construction — add the guard and note it here.)

## Step 6 design — performance baseline

*Realizes "steps/sec, memory, scaling with stock and domain count."* A measurement
harness **outside `simcore`** (`bench/`, repo root), using stdlib only
(`time.perf_counter`, `tracemalloc`) — **no third-party deps**, and not in `simcore`
so the purity gate is unaffected. Sweeps: stock count, domain count, and scheme
(Euler / RK4 / multi-rate). Output committed as `docs/perf-baseline.md` (a tracked
reference table) — **non-gating** (absolute numbers are machine-dependent; this is a
regression *reference*, not a pass/fail). A future phase may add a relative-regression
check; Phase 0.5 only establishes the baseline.

**Deliverable:** `bench/` script + committed `docs/perf-baseline.md` with steps/sec
and peak memory across the sweep, on the dev machine, with the machine/commit noted.

---

## Exit criteria (Phase 0.5 — "trust the math")

- [x] Convergence/timestep-sensitivity: observed order matches each scheme (Euler→1,
      RK4→4) across a `dt` ladder *(Step 1 done)*; multi-rate (Strang)→2 *(Step 3)*.
- [x] Adaptive RK45 oracle exists (out-of-core), tolerance-honoring, conserving; used
      as a convergence reference for a non-analytic case *(Step 2 done — RK4→RK45
      observed order 3.97 on LV; embedded estimate order 5.03)*.
- [ ] Multi-rate sub-stepping: deterministic, conserving, `n`-preserving (#14),
      registration-order-independent, with the asserted split order.
- [ ] 100k+ step run: bounded, no `NaN`/`Inf`/overflow, conservation holds
      (length-independence now empirical), non-arbitrating.
- [ ] Edge cases pinned (empty/zero, depletion, over-draw asymmetry, overflow).
- [ ] Performance baseline committed (`docs/perf-baseline.md`).
- [ ] Core purity, determinism, and the frozen Phase-0 API all still hold (the
      Phase-0 gates stay green).
```
