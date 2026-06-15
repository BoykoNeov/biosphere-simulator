# Phase 0 — Engine Skeleton

**Status:** Reviewed (advisor pass folded in) — ready to build on your go.
**Goal:** Freeze the engine architecture before any scientific complexity appears.
The architecture is multi-domain from the first commit; biosphere is simply the
first registered domain. We are building a deterministic stock-and-flow core and
proving its invariants on a trivial demo — *not* doing biology yet.

This plan is the concrete realization of the roadmap's Phase 0, incorporating the
design decisions settled during review (see **Locked Decisions** below).

---

## Locked decisions (from review)

These were debated and agreed before planning. They constrain the Phase-0 data
model and must not silently drift.

1. **Flows declare structured, per-stock withdrawals** — never an opaque net
   delta. This is the irreversible Phase-0 commitment. Arbitration is only
   *possible* if the integrator can see "this flow draws X from A, Y from B." The
   rationing *policy* layered on top stays swappable forever; the *data shape*
   does not.
2. **A flow is an atomic stoichiometric transfer.** It may move several
   quantities at once (e.g. CO₂-carbon → plant-carbon, consuming H₂O, producing
   O₂) at fixed internal ratios. Arbitration scales a **whole flow** as a unit,
   never individual legs — which is precisely what keeps *every* conserved
   quantity's ledger balanced automatically.
3. **Kinetic-primary, arbitration-as-backstop.** Competition / throttling
   emerges from saturating rate laws on shared stocks (Phase 1+). The hard
   arbitrator is a *rare numerical guard* for explicit-integration overshoot, not
   the ecological mechanism. (Phase 0 builds and tests the guard with synthetic
   scenarios; real kinetics arrive in Phase 1.)
4. **Backstop = single-pass min-scaling, always-on, counted.** Availability
   budget = start-of-step stock level (withdrawals never draw against same-step
   inflows). Each flow scales by the *minimum* over its resources' scaling
   factors. Conservation-safe by construction (proof below). The rationer runs on
   *one deterministic path* every step, **counts its firings, and golden
   scenarios assert the count ≈ 0** — frequent firing means dt too large or
   kinetics mis-scaled, i.e. silent systematic bias, the thing we most fear.
5. **Proportional sag is the physical default; priority is a *declared
   controller*.** Load-shedding ("crew O₂ before plant lights") is modeled
   control expressed as arbitration *data*, not free emergence. Policy is
   pluggable; proportional is the default.
6. **Extinction is an absorbing state — and conserves mass.** A stock below a
   deterministic threshold → set to exactly 0 + log an event; reintroduction only
   via scenario. **The snapped residual is routed into a tracked numerical-loss
   boundary sink** (see #13) so the ledger still balances — naively zeroing a
   stock would create an unaccounted ΔStored and fail the conservation gate. Only
   **absorbing-eligible** stocks (biomass / populations) may go extinct; **resource
   pools** (Atmospheric Carbon, Water) must never be zeroed-with-loss. The flag
   lives in the Phase-0 core state model because multispecies/pathogens come later.
7. **Determinism = bit-identical within a build, tolerance across ports.**
   Registration-order independence is achieved by **canonical reduction order**
   (sum each stock's deltas sorted by stable flow id). Cross-language
   (Python↔Rust, Phase 7) is tolerance-gated, not bit-exact.
8. **Energy: structure now, closure later.** The ledger carries energy from
   Phase 0, and flows may carry an (optional) energy leg, but energy *closure* is
   only enforced from Phase 5/6 (when Power exists). In Phase 0/1 energy is a
   diagnostic.
9. **Units validated at the boundary, floats in the hot loop.** Dimensional
   checks (pint) live in the outer param-loading layer — **never imported by
   `simcore`**. The pure core stores plain floats plus a canonical-unit *label*;
   a single canonical-unit table is the shared source of truth (and ports to Rust
   unchanged). The integrator runs on plain floats.
10. **Fidelity is 0-D well-mixed compartments** (a tank network). Any spatial
    structure (e.g. canopy light layers) is an *internal* computation inside a
    flow, never core spatial state.
11. **Core is pure — zero third-party dependencies.** `simcore` imports only the
    Python stdlib: no plotting, UI, database, networking, file-format, units, or
    numerics libraries. It exposes plain-data state; an *outer* `sim_io` layer
    owns serialization formats and an outer loader owns units/param parsing. This
    makes the Rust port a near-mechanical translation.
12. **RNG is counter-based and keyed, implemented in-core in pure Python.** Each
    draw derives from `(master_seed, key, step)` (Philox / splitmix64-style), so
    draws are **independent of consumption order** — required by determinism (#7).
    The seed lives in `State`. Sequential-state generators (PCG64 / Mersenne) are
    rejected because draw order would change output. The scheme mirrors exactly in
    Rust → cross-port bit-identical streams. RNG is only for scenario-level
    stochastic events (validated statistically, Tier 3).
13. **The system boundary is a set of reservoir stocks; every flow is internally
    balanced.** External forcing (irrigation adds water, harvest removes carbon,
    light/solar adds energy) is *unbalanced* against the modeled stocks, which the
    flow invariant (#2) forbids. Resolution: model "outside" as explicit **reservoir
    stocks** in a Boundary domain. Then every flow is balanced, `Σ legs = 0` over
    the augmented (closed) system, and the ledger's `Inputs / Outputs` are simply
    the boundary reservoirs' deltas. Source reservoirs (solar) carry an
    **`unclamped` flag** so arbitration's min-scaling never throttles them. This
    also hosts the extinction loss-sink (#6). Built in Phase 0 even though the
    closed demo barely exercises it — Phase 1 needs it on day one.
14. **Time is an integer step count.** Forcing and schedules are evaluated at
    `t = n · dt` (n: int), never an accumulated `t += dt` — float drift over 100k+
    steps would perturb forcing evaluation and undermine determinism/stability.
15. **Canonical order covers *every* reduction.** Not just the final per-stock
    delta sum, but the per-stock demand sum and the scale application too — any
    order-dependent reduction in the arbitration path breaks bit-identical-under-
    shuffle.
16. **Internal env resolution reads the immutable snapshot.** When `env.get(var)`
    resolves to a sibling domain's shared stock, it reads the same immutable
    snapshot that flows read — otherwise this separate read path reintroduces
    order-dependence.

---

## Scope

### In scope (Phase 0)
- Core state primitives: `Quantity`, `Stock`, `Flow`, `Domain`, `Environment`,
  `Observation`, `State`.
- Domain registry with cross-domain flows.
- Integrator interface (strategy) + **Euler** and **RK4**; interface is
  *implicit-ready* (may evaluate the flow function many times per step) but no
  implicit solver is implemented yet.
- Atomic application: pure flow evaluation against an immutable snapshot →
  collect → canonical reduce → write once.
- Arbitration backstop (min-scaling, counted) + pluggable policy seam
  (proportional default).
- Extinction absorbing-state handling.
- Conservation ledger + per-quantity balance assertion every step.
- Environment as **source resolver** (forcing schedule *or* shared stock; reader
  can't tell which), with the internal branch reading the immutable snapshot.
- Boundary domain: reservoir stocks (`unclamped` sources + numerical-loss sink)
  so every flow is internally balanced and Inputs/Outputs = boundary deltas.
- Plain-data state snapshot + round-trip via outer `sim_io`.
- Minimal **two-domain** demo:
  - *Biosphere* — `Atmospheric Carbon` ⇄ `Plant Carbon` via `Photosynthesis` /
    `Respiration` (trivial laws — no FvCB).
  - *Boundary* — `Outside Carbon` reservoir.
  - one **cross-domain** flow `Harvest` (Plant Carbon → Outside Carbon) to
    exercise the registry's cross-domain path and balanced boundary exchange.
  - one **internal source-resolver** case: an env var resolving to a Boundary
    stock, proving the reader can't tell forcing from shared-stock.
- Test suite (determinism incl. registration-order + bit-identical;
  conservation; serialization round-trip; arbitration backstop; extinction;
  numerical convergence/drift; regression snapshots).

### Explicitly deferred (do NOT build in Phase 0)
- Real biology / saturating kinetics, FvCB, Penman–Monteith → **Phase 1**.
- Implicit solver, adaptive RK45, multi-rate sub-stepping, perf baseline,
  stability sweeps → **Phase 0.5**.
- Energy *closure*, Power/Thermal/Crew/Atmosphere domains → **Phase 5/6**.
- Rust port → **Phase 7**.
- Numpy vectorization is *optional* and only if it preserves canonical-order
  determinism; default to plain typed data for clarity in the reference.

---

## Frozen API (the part that's expensive to change)

```python
# --- conserved quantities -------------------------------------------------
class Quantity(Enum):
    CARBON; WATER; NITROGEN; OXYGEN; ENERGY     # PHOSPHORUS reserved

class StockKind(Enum):
    POOL         # resource pool — never zeroed-with-loss
    POPULATION   # absorbing-eligible — may go extinct (residual -> loss-sink)
    BOUNDARY     # "outside" reservoir — its delta is a ledger Input/Output

# --- stocks ---------------------------------------------------------------
@dataclass
class Stock:
    id: StockId                 # stable, canonical-sortable
    domain: DomainId
    quantity: Quantity
    unit: UnitLabel             # canonical-unit *label* (str/enum); pint
                                # validation lives in the loader, not the core
    amount: float               # in canonical unit
    kind: StockKind             # POOL | POPULATION | BOUNDARY
    extinction_threshold: float = 0.0   # POPULATION only: below -> 0 + loss-sink
    unclamped: bool = False     # BOUNDARY source (e.g. solar): never min-scaled
    # POOL stocks are never zeroed-with-loss; only POPULATION stocks go extinct.
    # BOUNDARY stocks are the "outside" reservoirs; their deltas are the ledger's
    # Inputs/Outputs (see locked decision #13).

# --- flows: structured, atomic, stoichiometric ----------------------------
@dataclass(frozen=True)
class Leg:                       # one stock touched by a flow
    stock: StockId
    amount: float               # >0 deposit, <0 withdrawal, per dt

@dataclass(frozen=True)
class FlowResult:
    legs: tuple[Leg, ...]       # the *requested* transfer at this snapshot
    # invariant: per Quantity, sum(deposits) == sum(withdrawals) (+ declared
    # cross-quantity stoichiometry). Validated in tests.

class Flow(Protocol):
    id: FlowId                  # stable, canonical-sortable
    priority: int               # for declared-controller arbitration policies
    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult: ...
    # PURE: reads snapshot/env only; no mutation; deterministic.

# --- environment: source resolver ----------------------------------------
class Environment(Protocol):
    def get(self, var: str) -> float: ...   # resolves to forcing OR shared stock
    # caller cannot tell which; identical biosphere code runs standalone & coupled
    # forcing branch:  schedule evaluated at t = n*dt (integer n)
    # internal branch: reads the IMMUTABLE snapshot, same as flows (#16)

# --- integrator: strategy, implicit-ready --------------------------------
class Integrator(Protocol):
    def step(self, state: State, env: Environment, dt: float) -> State: ...
    # owns stepping; calls the (arbitrated) flow-evaluation function as many
    # times as the scheme needs (Euler:1, RK4:4, implicit-future:N).
    # CONTRACT: the min-scaling backstop is Euler-only. Under any higher-order
    # scheme (RK4+), a flow needing scale_f < 1 is a HARD ERROR — positivity of
    # an RK-combined update does not compose from clamped stages (mass
    # conservation does). Positivity there must come from kinetics, not the
    # backstop. (Positivity-preserving conservative integration — e.g. Modified
    # Patankar-RK — is a deferred research item, not Phase 0.)

# --- RNG: counter-based, keyed, pure-Python, seed carried in State -------
class Rng(Protocol):
    seed: int
    def draw(self, key: tuple, step: int) -> float: ...   # order-independent

# --- global state (immutable snapshot per step) --------------------------
@dataclass(frozen=True)
class State:
    n: int                      # integer step count; time t = n * dt (#14)
    stocks: Mapping[StockId, Stock]
    rng_seed: int               # carried; draws are keyed by (seed, key, n)
    # one global state, one ledger; domains are namespaces within stocks/flows.

# --- core surface (everything else is outside) ---------------------------
def init(config) -> State: ...
def step(state, env, dt) -> State: ...
def observe(state) -> Observation: ...   # plain data
```

### Step algorithm (one derivative evaluation = steps 1–4)
1. Take immutable `snapshot`.
2. `evaluate()` every registered flow against the snapshot → list of
   `FlowResult`. (Pure; order-independent.) Flows reading internal env vars read
   this same snapshot (#16).
3. **Arbitrate** (backstop — *Euler scheme only*; see integrator contract):
   - per stock, `demand_s = Σ |withdrawals from s|` (**canonical order**, #15);
   - skip `unclamped` BOUNDARY sources;
   - `scale_s = min(1, available_s / demand_s)`, `available_s` = snapshot level;
   - per flow, `scale_f = min(scale_s for s it withdraws from)`;
   - if any `scale_f < 1`: increment **rationing counter**; scale the *whole*
     flow (all legs) by `scale_f` (**canonical order**, #15). Under RK4+, a
     `scale_f < 1` is instead a **hard error**.
4. **Reduce**: per stock, sum scaled legs in **canonical (flow-id) order** (#15).
5. Combine the scheme's evaluations (Euler:1 / RK4:4), **apply once** (write `n→n+1`).
6. **Extinction pass** (POPULATION stocks only): any such stock
   `< extinction_threshold` → set to 0, **route the residual into the
   numerical-loss BOUNDARY sink** (keeps the ledger balanced), emit event. POOL
   stocks are never zeroed-with-loss.
7. **Conservation check**: per quantity, `|inputs − outputs − ΔStored| < tol`,
   where `inputs/outputs` are BOUNDARY-reservoir deltas (#13); failure raises
   (it's an engine bug).

**Conservation safety of min-scaling (proof, Euler):** for any stock *s*, realized
draw = Σ_f |w_fs|·scale_f ≤ Σ_f |w_fs|·scale_s = demand_s·scale_s ≤ available_s,
since scale_f ≤ scale_s for every flow touching *s*. So no stock goes negative, and
because whole flows scale as units, each flow's internal stoichiometry — hence
every quantity's balance — is preserved. **This is single-evaluation (Euler) only.**
Under RK4 the update is a weighted sum of clamped stage-derivatives and positivity
does **not** compose (mass conservation still does, being linear) — hence the
Euler-only backstop contract above; positivity under higher-order schemes must come
from the kinetics, not the guard.

---

## Test suite (exit gates)

| Test | Asserts |
|---|---|
| **Determinism** | identical (state, params, forcing) → **bit-identical** output across runs. |
| **Registration-order independence** | shuffling stock/flow registration → bit-identical output (validates canonical reduction). |
| **Conservation** | every quantity balances each step within tolerance, incl. arbitration & extinction events. |
| **Arbitration backstop** | synthetic over-draw → non-negative stocks, conserved, rationing counter > 0; ample-supply scenario → counter == 0. |
| **Extinction** | POPULATION stock snaps to 0 below threshold, residual lands in the loss-sink (ledger still balances), emits event, never revives from noise; POOL stocks never zeroed. |
| **Cross-domain + resolver** | a flow with source in one domain and sink in another runs under registration-shuffle; `env.get()` resolving to a sibling domain's stock matches the forcing-backed run (internal branch reads snapshot). |
| **Boundary exchange** | an "unbalanced" forcing flow (e.g. Harvest) balances once the Boundary reservoir is counted; `unclamped` source not throttled by arbitration. |
| **Convergence / drift** | a **non-arbitrating** oscillator (toy 2-stock predator–prey + harmonic) converges as dt→0 under RK4; Euler's spurious amplitude growth is *detected* (guards the L-V/Euler trap). |
| **Serialization round-trip** | state → plain data → state reproduces bit-identical continuation (JSON; **hex-float** in golden files for exact cross-run/cross-port comparison). |
| **Regression snapshot** | golden demo run; any bit change is surfaced. |
| **Units** | param load / flow registration reject dimensional mismatch. |

Property-based tests (Hypothesis) for the invariants that should hold over *any*
valid configuration: conservation, non-negativity, order-independence.

---

## Tooling (settled in review)

- **Language:** Python ≥ 3.12 (reference/laboratory; matches "Python never dies").
  Develop on 3.13 for wheel stability unless 3.14 is wanted.
- **Packaging/env:** `uv` + `pyproject.toml`, with a **committed lockfile** for
  reproducible numerics. (PCSE is pure-Python, so no conda is forced; `uv` stands.
  See `docs/reuse-and-licenses.md` — PCSE is EUPL, used only as an offline oracle.)
- **Tests:** `pytest` + `hypothesis` (property-based tests for the universal
  invariants: conservation, non-negativity, order-independence). Golden snapshots
  are hand-rolled files (full float-format control), not a snapshot plugin.
- **Units:** `pint`, **outer layer only** — `simcore` never imports it. The core
  carries canonical-unit labels; the loader validates/converts via pint.
- **Params:** YAML (`safe_load`) + **pydantic** schema validation on load (catches
  typos, out-of-range coefficients, unit mismatches). TOML is an acceptable swap
  for flatter param sets.
- **RNG:** small **counter-based, keyed** generator (Philox / splitmix64-style),
  **pure-Python, in `simcore`**, seed in `State`. Order-independent by
  construction; mirrors exactly in Rust. (Sequential-state PCG64/Mersenne rejected
  — draw order would break order-independence.)
- **Serialization (outer `sim_io`):** JSON checkpoints; **hex-float** in
  golden/regression files for unambiguous cross-run and cross-port comparison.
  NaN/Inf forbidden in state. Bulk time-series telemetry format (parquet/HDF5)
  deferred to Phase 8.
- **Core dependency budget:** `simcore` = **stdlib only, zero third-party deps**
  (numpy only enters outer layers or a later perf pass, and only if it preserves
  canonical-order determinism).
- **Quality:** `ruff` (lint + format), `pyright` (types; mypy optional in CI).

## Repo layout (proposed)

```
space-station/
  pyproject.toml
  src/simcore/            # PURE engine — stdlib only, ZERO third-party deps
    quantities.py  state.py  flow.py  domain.py  registry.py
    integrator.py  arbitration.py  conservation.py
    environment.py observation.py  determinism.py  events.py  rng.py
  src/sim_io/             # OUTSIDE the core: JSON checkpoints, hex-float golden
  src/config/             # OUTSIDE: YAML loader + pydantic schemas + pint units
  src/domains/biosphere/  # Phase-0 minimal demo
    flows.py  params/*.yaml
  tests/                  # mirrors the table above + tests/regression/golden/
  docs/plans/
```

---

## Exit criteria (all must hold)

- [ ] Deterministic replay (bit-identical re-run).
- [ ] Registration-order independence (bit-identical under shuffle).
- [ ] State serialization round-trips exactly.
- [ ] Conservation passes every step for all quantities.
- [ ] Arbitration backstop: non-negative + conserved under over-draw; counter == 0
      on the well-fed demo.
- [ ] Extinction absorbing state works, conserves mass via the loss-sink, never
      revives; POOL stocks never zeroed.
- [ ] Cross-domain flow + internal source-resolver exercised and shuffle-stable.
- [ ] Boundary exchange balances; `unclamped` sources not throttled.
- [ ] Convergence/drift test green; Euler oscillator-growth trap demonstrably
      caught.
- [ ] Engine + domain API frozen (this document's Frozen API section).

---

## Risks / watch-items

- **Backstop firing on the demo** would mean the demo dt or laws are wrong —
  treat a nonzero counter as a failing gate, not a warning.
- **Numpy reductions vs determinism:** if/when vectorized, must preserve
  canonical order; default to plain reduction in the reference.
- **Arbitration makes the derivative non-smooth** (min is non-differentiable),
  which will matter for the *future* implicit solver — acceptable now because the
  backstop fires ~never; note it for Phase 0.5.
- **Positivity under higher-order schemes is unsolved by the backstop.** RK4 relies
  on kinetics for non-negativity (the Euler-only guard hard-errors instead). If a
  real domain later needs robust positivity under RK4+, adopt a positivity-
  preserving conservative integrator (Modified Patankar-RK) — deferred research,
  flagged here so the integrator interface keeps room for it.
- **Priority policy chattering** near the scarcity threshold — out of Phase-0
  scope (proportional default), flagged for when load-shedding is modeled.

## Sequencing

1. Repo + tooling skeleton (`pyproject`, `uv`, `ruff`, `pytest`, CI-less local).
2. `quantities`, `Stock` (+ `kind`/`unclamped`), `State` (integer `n`),
   counter-based `rng`, units-at-boundary.
3. `Flow`/`Leg`/`FlowResult` + registry + cross-domain flows.
4. Boundary domain: reservoir stocks (`unclamped` sources + loss-sink).
5. `Environment` source resolver (forcing-at-`n·dt` + snapshot-reading shared-stock
   backends).
6. Integrator strategy: Euler (with backstop), then RK4 (backstop→hard error);
   atomic apply + canonical reduce everywhere.
7. Arbitration backstop + counter; extinction-with-loss-sink; events.
8. Conservation ledger (Inputs/Outputs = boundary deltas) + per-step assertion.
9. Outer `sim_io` snapshot round-trip (JSON; hex-float goldens).
10. Two-domain demo (Biosphere + Boundary; `Harvest` cross-domain flow; internal
    resolver case).
11. Full test suite + golden snapshot; freeze API.
```
