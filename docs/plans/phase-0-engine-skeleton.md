# Phase 0 — Engine Skeleton

**Status:** In progress — steps 1–10 complete (repo/tooling skeleton; core state
primitives: quantities, Stock, State, RNG, units-at-boundary core seam; flows:
`Leg`/`FlowResult`/`Flow` + balance helpers, `Environment` protocol, `Registry`;
Boundary domain: `source`/`sink`/`loss_sink` reservoir constructors + the
per-quantity numerical-loss-sink identity; Environment source resolver:
`SourceResolver`/`BoundEnvironment` + `Schedule`/`constant`, forcing-at-`n·dt`
and snapshot-reading shared-stock backends behind the frozen `get`; integrator
strategy: `Integrator` protocol + `EulerIntegrator`/`Rk4Integrator`,
increment-form RK4, canonical reduce + apply-once, referential integrity in the
apply path; arbitration backstop: `simcore.arbitration` min-scaling (Euler-only,
`unclamped`-skip, whole-flow scaling) with the RK4 `check_no_overdraw` hard-error
asymmetry; extinction-with-loss-sink + `ExtinctionEvent`; functional `StepReport`
`(state, events, rationed)` keeping the core mutation-free; conservation ledger:
`simcore.conservation` `QuantityLedger`/`compute_ledger`/`assert_conserved`, the
always-on every-step gate wired into the integrator's shared `_finalize` tail;
outer `sim_io` snapshot round-trip: `State ⇄ JSON` with hex-float amounts +
hex-string seed, byte-exact committed golden; two-domain Biosphere+Boundary demo
wiring the spine end-to-end via `domains/biosphere/{flows,demo}.py`). Steps 3–10
were built test-first against their design sections below (advisor-reviewed). Step
10 added the demo: Photosynthesis/Respiration/`Harvest` (cross-domain) flows, the
internal source-resolver case proven as a **bit-identical forcing≡shared run**
(#16), and the **well-fed backstop gate** (`rationed == 0` / no RK4 over-draw).
Step 11 (full test suite + golden snapshot; freeze API) is a bundle, now **in
progress**: **cluster 1 — the `config/` loader + `params/demo.yaml` + the param-load
half of the Units gate (Scope A)** is done (pydantic/pint/yaml confined to
`config/` + `domains/biosphere/loader.py`; `DemoParams`' inline defaults removed,
the YAML is now the single source of truth; amounts declared in canonical units so
the demo run stays bit-identical). **Cluster 2 — the `simcore`-purity test** is now
done: a static AST guard (`tests/test_simcore_purity.py`) asserting every core file
imports stdlib or `simcore` only — tight enough to also catch a core→outer-layer
leak — with discrimination controls (a pure `_third_party_imports` helper proven to
flag `numpy`/`pint`) and a vacuous-pass guard. Remaining clusters: convergence/drift
oscillator, golden demo regression snapshot, `observe`/`Observation`,
conservation-tol carry-forward, API freeze. (Earlier: Reviewed, advisor pass folded
in.)
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
@dataclass(frozen=True)   # step-2 decision: Stock is FROZEN, matching the
                          # immutable-snapshot/determinism model — a step writes
                          # a new State via dataclasses.replace, never mutates a
                          # Stock in place. (Perf is explicitly deferred.)
class Stock:
    id: StockId                 # stable, canonical-sortable
    domain: DomainId
    quantity: Quantity
    unit: UnitLabel             # canonical-unit *label* (str/enum); pint
                                # validation lives in the loader, not the core.
                                # The Quantity->UnitLabel canonical table lives in
                                # simcore.quantities (the shared source of truth,
                                # #9). Step-2 note: the *specific* labels (mol vs
                                # kg ...) are PROVISIONAL — they don't affect any
                                # Phase-0 invariant (conservation only needs one
                                # consistent unit per quantity); the science pick
                                # is a Phase-1 decision.
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
    def step(self, state: State, env: SourceResolver, dt: float) -> State: ...
    # env reconciled to the SourceResolver binding source in step 6 (was
    # `env: Environment`): a BoundEnvironment is pinned to one snapshot, but RK4
    # must rebind per stage, which only the resolver can do. Registry is injected
    # at construction. See "Step 6 design" below for the full rationale.
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
    def draw(self, key: tuple[int, ...], step: int) -> float: ...  # order-independent
    # Cross-port contract (step 2): keyed splitmix64 *finalizer* used as a
    # stateless hash of (seed, *key, step); all ops masked to 64 bits; float via
    # (x >> 11) / 2**53 in [0,1) (no 1.0, no NaN/Inf). `key` is a tuple of ints
    # for Phase 0 — string-folding (which would need a pinned byte-hash like
    # FNV-1a + its own vectors) is deferred until a scenario needs it. Golden
    # hex vectors in the tests are the Rust port's conformance target.

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

## Step 3 design — flows, balance helper, registry

*Design pass (settled with advisor); implementation next.* This realizes the
Frozen API's flow/registry portion. The shapes here are the irreversible
"structured per-stock legs" commitment (#1/#2), so they are pinned before coding.

### New modules
- `simcore/flow.py` — `Leg`, `FlowResult`, the `Flow` Protocol, and pure
  *result-level* helpers (`per_quantity_residual`, `assert_flow_balanced`,
  `domains_touched`).
- `simcore/environment.py` — the **`Environment` Protocol only** (`get(var) -> float`).
  The interface is needed for `Flow.evaluate`'s signature; the forcing /
  shared-stock *backends* are step 5. Building just the Protocol now keeps step 3
  from pulling step 5 forward.
- `simcore/registry.py` — `Registry`.

### Data shapes (frozen, stdlib, NaN/Inf-rejecting like `Stock`)
- `Leg(stock: StockId, amount: float)` — `amount` per dt; `>0` deposit, `<0`
  withdrawal. `__post_init__` rejects non-finite `amount`.
- `FlowResult(legs: tuple[Leg, ...])` — **at most one leg per `StockId`** (reject
  duplicates in `__post_init__`: a flow nets its own touches on a stock). Empty
  `legs` is a valid no-op step. One-leg-per-stock also keeps arbitration's
  `demand_s` / `scale_f` (#15, step 7) a clean per-(stock, flow) quantity.
- `Flow(Protocol)` — `id: FlowId`, `priority: int`,
  `evaluate(snapshot: State, env: Environment, dt: float) -> FlowResult`. PURE.
  `priority` is carried for declared-controller policies (#5) but **unused under
  the proportional default**; canonical reduction order is always id-sorted (#15),
  never priority.

### Balance is evaluation-time, not registration-time (key correction)
A `Flow` exposes only `evaluate`; **legs exist only after evaluation against a
snapshot.** So nothing leg-shaped is knowable at registration. Therefore both the
balance check and referential integrity are *evaluation-time*:
- **Balance** is a property of an *evaluated* `FlowResult`: per `Quantity`,
  `Σ legs == 0` within tolerance (every mole withdrawn from one carbon stock lands
  in another carbon stock, incl. boundary reservoirs). Resolving each leg's
  quantity needs a stock lookup, so the check is a pure helper over
  `(result, stocks)` — used in step-3 tests and **reused by the step-8 every-step
  conservation assertion**.
- **`ENERGY` is exempt** (#8 — energy closure is Phase 5/6; here it is diagnostic).
  `per_quantity_residual` returns *all* residuals incl. ENERGY (the diagnostic);
  `assert_flow_balanced` checks only the **asserted set**. That asserted set
  (all mass quantities = every `Quantity` except `ENERGY` for Phase 0) lives as
  **one constant in `simcore/quantities.py`** so step 3 and the step-8 ledger
  cannot drift apart.
- **Referential integrity** (every produced `Leg.stock` is a real stock) is also
  evaluation-time: an assertion in the **apply path** (step 5/6), not a
  registration check. Cost: a typo'd stock id surfaces at first step, not at build
  — acceptable for Phase 0; static-wiring validation is a noted future enhancement.

### Registry (frozen, build-once config)
Because referential integrity and "which domains a flow touches" are
evaluation-time, the registry's real job shrinks to **structure only**:
- holds the flow set; **rejects duplicate `FlowId`**;
- exposes **canonical id-sorted** flow iteration — this is the registration-order-
  independence guarantee (#7/#15): shuffling registration yields bit-identical
  iteration;
- a **domain index** `Mapping[DomainId, frozenset[StockId]]` derived from
  `Stock.domain` over the initial stocks. This *is* Phase-0's "Domain" primitive
  (scope list): a `DomainId` namespace + its stock membership, **not** a rich class.
- The registry is **injected into the integrator at construction** (the frozen
  `Integrator.step(state, env, dt)` has no registry param) — built here, consumed
  in step 6.

"Cross-domain flow" is then a property of an *evaluated* result:
`domains_touched(result, stocks) -> frozenset[DomainId]` maps legs → `Stock.domain`;
a flow is cross-domain iff it touches >1 domain. Step-3 tests exercise this with a
**synthetic** Harvest flow — the real Photosynthesis/Respiration/Harvest demo is
step 10; the cross-domain test must not drag the biosphere demo forward.

### Why per-flow balance ⟹ global conservation (forward note)
Arbitration scales a whole flow by one factor (#2/#4), and `scale_f · Σlegs = 0`,
so scaling preserves per-flow balance *exactly*; summed over flows this yields the
global ledger balance (#13). **Caveat:** extinction's loss-sink routing (step 7)
is a *balanced non-flow* state change — so "every flow is balanced" must **not** be
mis-stated as "every state change is a flow."

### Step-3 test plan
- `Leg`/`FlowResult`: frozen; non-finite `amount` rejected; duplicate-`StockId`
  `FlowResult` rejected; empty legs allowed.
- Balance helper: a balanced synthetic flow passes; a carbon-imbalanced flow fails;
  an energy-only imbalance is **tolerated** (ENERGY exempt); near-zero residual
  within `tol` passes (tolerance is actually applied).
- `Flow` purity: evaluating a synthetic flow twice on the same snapshot returns an
  equal `FlowResult`.
- Registry: rejects duplicate `FlowId`; **registration-order independence** —
  shuffling the flow list yields bit-identical canonical iteration (Hypothesis
  property test); domain index matches `Stock.domain`.
- Cross-domain: a synthetic Harvest (plant-carbon → outside-carbon across two
  domains) is balanced and `domains_touched == {biosphere, boundary}`.

---

## Step 4 design — Boundary domain reservoir stocks

*Design pass (settled with advisor); implemented test-first.* This realizes
decision #13 (and hosts #6's loss-sink). **Key framing:** the `Stock` *data
shape* needed here already exists — step 2 gave `Stock` its `BOUNDARY` kind and
`unclamped` flag. So step 4 is **not** a `Stock` change; it is the boundary
*module* — the canonical identities + safe constructors that later steps
reference **by name**.

### New module — `simcore/boundary.py` (not `domain.py`)
`domain.py`'s original "rich Domain class" role was dissolved into the registry's
domain index in step 3 ("This index *is* Phase-0's Domain primitive … not a rich
class"). Reviving it for unrelated content would mislead, so the boundary content
lives in `boundary.py`; the repo-layout block above is updated to match.

Three reservoir roles, all `BOUNDARY`-kind, each built by a constructor so call
sites cannot misconfigure `kind` / `unit` / `unclamped` (the `unit` is always
derived from `quantity` via the canonical-unit table — single source of truth,
#9):
- `source(stock_id, quantity, amount, *, unclamped=True)` — an "outside" supply
  (solar; outside atmosphere as a source). **`unclamped` defaults True** — the
  decision-#13 default that is easy to get wrong, encoded here so min-scaling
  never throttles a supply. Overridable for a finite/throttleable supply.
- `sink(stock_id, quantity, amount=0.0)` — an "outside" disposal reservoir
  (receives outputs, never withdrawn from, so min-scaling — withdrawals only —
  never applies; stays clamped). `amount` is a cumulative-output accumulator.
- `loss_sink(quantity, amount=0.0)` + `loss_sinks(quantities=ASSERTED_QUANTITIES)`
  — the numerical-loss reservoir extinction routes a snapped POPULATION residual
  into (#6). **One per conserved (mass) quantity**, because a `Stock` holds
  exactly one `Quantity` and conservation is per-quantity — a single multi-quantity
  sink is impossible by the data model. ENERGY gets none (balance-exempt, #8;
  POPULATION biomass is carbon anyway). Identity is canonical/deterministic:
  `loss_sink_id(q) == "boundary.loss.<q.value>"`, with `is_loss_sink(id)` so the
  step-8 ledger / diagnostics can separate routed numerical-loss deltas from
  legitimate boundary exchange.

`BOUNDARY_DOMAIN = DomainId("boundary")` is the canonical namespace (already used
by existing fixtures, e.g. `boundary.outside_c`).

### Decisions pinned here
- **Guard `unclamped ⇒ kind == BOUNDARY` in `Stock.__post_init__`** (a step-2
  primitive tightened mid-step-4, called out in the commit). This is a
  *conservation* guard, not tidiness: if step 7's arbitration skip is later
  written as the natural `if stock.unclamped: skip` (dropping the kind check), an
  unclamped POOL would never be throttled and could go negative — a silent
  conservation break. Rejecting `unclamped=True` on a non-BOUNDARY at construction
  closes that single-point hazard and loses nothing (it is meaningless per #13).
- **Unclamped sources may go negative *by design*.** The non-negativity invariant
  guards POOL (via arbitration) and POPULATION (via extinction), **not** unclamped
  sources, whose magnitude is pure ledger bookkeeping. (Documented in the module so
  a future reader does not "fix" it.)
- **Constructor is necessary but not sufficient.** Loss-sinks (and any boundary
  reservoir) must actually be placed into the initial `State` by step 10's init,
  or step 7's extinction deposit / step 8's ledger hit a referential-integrity
  `KeyError` at the first step (referential integrity is the apply path's job,
  step 5/6 — not a build-time check).

### Step-4 test plan (construction-level; later machinery not pulled forward)
- Loss-sink identity: `loss_sink_id` deterministic + quantity-specific;
  `is_loss_sink` true for a loss-sink id, false for a normal boundary/biosphere id.
- `loss_sink(q)` is a zeroed `BOUNDARY` reservoir (right quantity/unit/domain,
  `unclamped=False`); `loss_sinks()` covers exactly `ASSERTED_QUANTITIES`, keyed by
  id (ready to merge into `State.stocks`), with no ENERGY sink; explicit-set arg
  honoured.
- `source` is `BOUNDARY` + `unclamped=True` by default and overridable; `sink` is a
  clamped `BOUNDARY` accumulator.
- **Boundary-exchange (conservation half):** a synthetic Harvest (plant carbon →
  boundary sink) **balances under `assert_flow_balanced` once the boundary reservoir
  is counted** (#13) — reusing the step-3 helper. *(The "unclamped source is not
  throttled" half of that exit-gate needs the arbitrator, step 7.)*
- **Loss-sink routing conserves:** a withdrawal + matching loss-sink deposit
  balances per-quantity — pinning that step 7's routing will conserve, without
  dragging step 7's mechanism forward.
- Guard: `unclamped=True` on POOL/POPULATION raises; on BOUNDARY it is allowed.

---

## Step 5 design — Environment source resolver

*Design pass (settle with advisor); implement test-first.* Realizes the scope
item "Environment as source resolver" plus decisions #14 (forcing at integer
`t = n·dt`) and #16 (internal branch reads the immutable snapshot). Step 3 built
the `Environment` **Protocol** (`get(var) -> float`); step 5 builds the concrete
**backends** behind it without touching the frozen interface.

### The binding model (the crux)
`Environment.get(var)` takes only `var` (frozen API) — so all per-step context
(*which* snapshot, *which* `dt`) must be **bound into** the object before it
reaches a flow. Step 5 therefore splits into:
- **`SourceResolver`** — immutable, build-once *wiring* (mirrors `Registry`): two
  **disjoint** var maps, `forcings: var -> Schedule` and `shared: var -> StockId`.
- **a bound view** — `resolver.bind(snapshot, dt)` returns a lightweight object
  satisfying `Environment`; the integrator (step 6) rebinds it **per derivative
  evaluation** (Euler: once; RK4: per stage) and hands it to `flow.evaluate`.

`get(var)` dispatches: a forcing var → `schedule(snapshot.n, dt)`; a shared var →
`snapshot.stocks[stock_id].amount`. The caller cannot tell which — the
indistinguishability that lets *identical* domain code run standalone (forcing)
and coupled (shared stock).

**Step-6 seam contract (pin now, enforce there):** the integrator must `bind`
the env to the **same snapshot object** it passes positionally to
`flow.evaluate(snapshot, env, dt)`. This is the mechanism that *makes* #16 hold —
if step 6 ever binds to a different state than it passes, a flow's direct snapshot
reads and its `env.get` shared-stock reads silently diverge, and **no step-5 test
can catch it** (it is integrator behaviour, not resolver behaviour). The natural
step-6 code gets this right; the contract is written down so it stays right.

### Why `n` comes from the bound snapshot (#14 + #16 unified)
Both branches read the *one* bound snapshot: the internal branch reads its stock
amounts (#16 — the same snapshot flows read), and the forcing branch reads its
integer `n` (`n = snapshot.n`) to evaluate `schedule(n, dt)` at `t = n·dt`, never
an accumulated `t` (#14). Tying forcing-time to the bound snapshot also settles
RK4 for free: RK4's intermediate stage-states keep the step's `n` (only amounts
are perturbed; `n` increments only at apply), so **forcing is piecewise-constant
within a step**. This is *exact* (zero error), not a tolerated approximation,
because **time-varying-forcing × RK4 simply does not occur in Phase 0**: the demo
forcing is constant, and the convergence/drift oscillator (Lotka–Volterra +
harmonic) is **autonomous** — its oscillation emerges from 2-stock coupling, with
no explicit `t` in any `env.get`. Time-varying forcing under RK4 would want
sub-stage evaluation at `(n + c_i)·dt`; that refinement is **deferred** and noted
so step 6 isn't boxed in.

### Schedule shape
`Schedule = Callable[[int, float], float]` — a pure function of `(n, dt)`. Passing
the integer `n` *and* `dt` (not a precomputed float `t`) keeps the integer visible,
so step-index schedules ("every k steps") and wall-time schedules (`t = n·dt`) are
both expressible drift-free (#14). A `constant(value)` helper covers the Phase-0
demo (validates `value` finite at construction). `get` guards the forcing result
with `math.isfinite` — stock amounts are already finite (`Stock.__post_init__`), so
only forcing can introduce NaN/Inf; a bad schedule fails loudly instead of
poisoning a downstream leg.

### Referential integrity = resolve-time (with eyes open)
A `shared` var pointing at an unknown `StockId`, or an unknown `var`, surfaces as a
`KeyError` at the first `get` — **not** a construction check. Note the analogy to
flow legs is *imperfect* and we own that: flow legs genuinely *cannot* be checked
at build (they don't exist until `evaluate`), whereas the `shared: var -> StockId`
map **is** fully known at construction. So the honest justification is not "can't
check it" but "**the cost of deferring is ~nil**": a missing target fails at the
*first* `get` (n = 0), loud and immediate, never a 100k-step time bomb — so a
build-time check buys almost nothing while coupling the resolver to the initial
stock set. (If step 10 later wants belt-and-suspenders, passing the stock-id set
and rejecting unknown targets is one set-difference — cheap to add then.) Overlap
*within* the wiring (a var in both maps) **is** rejected at construction — a
structural property of the wiring itself, knowable at build and independent of any
stock set (analogous to the registry's dup-id reject).

### Step-5 test plan
- `constant`: returns its value for any `(n, dt)`; rejects non-finite.
- Forcing branch: the `schedule` receives the integer `snapshot.n` and `dt`; the
  value changes with `n` for a time-varying schedule, is fixed for a constant.
- Shared branch: returns the bound snapshot's stock amount; **rebinding to a
  snapshot with a different amount changes the result** (proves it reads the bound
  snapshot, #16 — no caching of a stale value).
- Indistinguishability gate: a synthetic flow that withdraws `env.get("x")`
  produces an **equal `FlowResult`** whether `"x"` is a constant forcing `V` or a
  stock holding `V` — compared **across several `n`** (a one-binding comparison
  could pass even if the forcing branch accidentally ignored `n`; spanning `n`
  proves constant-forcing and static-stock stay equivalent as steps advance, the
  shape the step-10/11 full-run gate needs).
- Branch isolation (Hypothesis): the forcing value depends only on `(n, dt)`, not
  on the snapshot's stock contents.
- Construction: an overlapping var is rejected (`ValueError`); empty wiring is a
  valid resolver.
- Resolve-time errors: an unknown var → `KeyError`; a shared var → missing stock →
  `KeyError`.
- `bind(...)` satisfies the `Environment` Protocol (`isinstance`), tying the
  backend to the interface.

---

## Step 6 design — integrator strategy (Euler, RK4)

*Design pass (settled with advisor); implement test-first.* Realizes the scope
item "Integrator interface (strategy) + Euler and RK4" and the Frozen-API
`Integrator` Protocol + step-algorithm steps 1–2, 4–5. The arbitration backstop
(step-alg #3) and extinction/conservation (steps #6–7) are **deferred to steps
7–8**; step 6 builds the stepping spine with a clean seam for them.

### New module — `simcore/integrator.py`
The `Integrator` Protocol realization plus two concrete strategies
(`EulerIntegrator`, `Rk4Integrator`). Pure stdlib.

### The increment-form insight (why RK4 falls out cleanly)
`Flow.evaluate(snapshot, env, dt)` returns legs that are the **per-step
increment** `dt·rate(snapshot)`, not a bare rate (the "amount per dt" contract).
With `f(y) := reduce(evaluate-all-flows(y), dt)` the per-stock delta map:
- **Euler:** `y_{n+1} = y_n + f(y_n)` — one evaluation.
- **RK4 (increment form):** `k1=f(y_n)`, `k2=f(y_n+½k1)`, `k3=f(y_n+½k2)`,
  `k4=f(y_n+k3)`, `y_{n+1}=y_n+(k1+2k2+2k3+k4)/6`.

Because each `k_i` already carries `dt`, the ⅙-combine reproduces classical
`y+dt/6(k1'+2k2'+2k3'+k4')` exactly (advisor-verified). Arbitration stays
compatible: it compares per-step withdrawal deltas to snapshot levels (step 7).

### Load-bearing contract (the easy-to-rot one) — pin it
The increment-form identity holds **only if** `evaluate(y, dt) = dt·rate(y)` with
`rate` **independent of `dt`**. A flow that uses `dt` non-linearly (analytic
sub-step, dt-gated logic) still runs and still conserves mass, but **silently
drops RK4 to lower order** — and a single autonomous-oscillator convergence check
won't catch it (the demo flows are dt-linear). So:
- Document on `Flow.evaluate`: "returns `dt·rate`; `rate` must be `dt`-independent
  (linear in `dt`). Non-linear `dt` use still conserves mass but forfeits RK4
  order."
- Step-6 test: evaluate a demo flow at `dt` and `2·dt`; assert every leg scales
  ×2 (the dt-linearity guard).

### Derivative helper + stage states
`_derivative(stage_state, resolver, dt) -> dict[StockId, float]`: bind `resolver`
to `stage_state` (the **same** object the flows read — the step-5 seam contract
#16), `evaluate` every registry flow in canonical id-order, reduce legs per-stock
by summing in that canonical order (#15). Called 1× (Euler) / 4× (RK4).

RK4 **stage states keep `n`** (only amounts perturbed, via `dataclasses.replace`)
→ forcing is piecewise-constant within a step — *exact* here (the Phase-0
oscillator is autonomous; step-5 note). Intermediate stages may hold **negative**
amounts (allowed — `Stock` forbids only NaN/Inf); positivity under RK4 comes from
kinetics, not a guard (the integrator contract).

### Combine over the **union** of stage keys (missing ⇒ 0)
A state-gated flow can emit a leg at one stage but not another (a rate crossing a
threshold at a perturbed stage). Combining must iterate
`keys(k1)∪…∪keys(k4)` with `k_i.get(s, 0.0)`, never one map's keys — else a stock
silently drops. (Won't bite the structurally-fixed demo flows, but a state-gated
test pins it.)

### Apply-once + referential integrity
`step` builds the next `State` (n→n+1) by adding the combined delta to each named
stock (via `dataclasses.replace`). This is the **apply path** the plan assigns
referential integrity to (step 5/6): a leg naming a stock absent from
`State.stocks` raises a clear error at the first step (not a silent drop). Stocks
with no delta pass through unchanged.

### `env` is the binding source (`SourceResolver`) — a conscious reconciliation
The frozen `Integrator.step(state, env, dt)` annotated `env: Environment`, written
**before** the step-5 resolver existed. A `BoundEnvironment` is pinned to one
snapshot — RK4 must *rebind* per stage, which only `SourceResolver.bind` does. We
reconcile the slot to `env: SourceResolver` (the binding source; the integrator
produces `Environment` views internally), keeping `env` a **per-step argument**
while `Registry` is a **construction** dependency. This honors the frozen API's
*deliberate* split — registry = model *structure* → construction (per
`registry.py`'s "the frozen `Integrator.step` has no registry param" note); env =
per-run *scenario* wiring (same model, different forcing/coupling) → step arg —
rather than collapsing both to construction. The parameter is named honestly as a
binding source, not a pre-bound `Environment`. (Symmetry with `SourceResolver`'s
"mirrors Registry" docstring would instead argue for construction + `step(state,
dt)`; we chose the per-run-input reading deliberately, not by default.)

### Arbitration seam — deferred, not pre-built
Step 6 ships `_derivative` as pure evaluate→reduce with **no** arbitration
parameter (a guessed no-op signature is speculative interface step 7 would
rework). The two integrator classes are the place step 7 attaches the asymmetry
(Euler min-scaling vs RK4 hard-error); step 7 extends `_derivative`/the classes
then.

### Step-6 test plan
- **Euler correctness:** one step of a balanced flow equals `y_n + dt·rate`
  exactly.
- **RK4 correctness + order:** on analytic exponential decay (`dy/dt=-λy`, a POOL
  stock → boundary sink, so balanced and clear of extinction/over-draw), RK4's
  error vs `y0·e^{-λt}` is far below Euler's, and halving `dt` shrinks RK4 error
  ~16× vs Euler ~2× (4th vs 1st order).
- **dt-linearity contract:** legs scale ×2 from `dt` to `2·dt` (guards the
  load-bearing contract above).
- **Conservation of the new arithmetic (test-level, *not* the step-8 gate):** the
  realized per-step delta map, wrapped as a `FlowResult`, passes step-3
  `assert_flow_balanced` — for **both** Euler and RK4 (each `k_i` is a sum of
  balanced legs; the linear ⅙-combine preserves balance).
- **Union-of-keys combine:** a state-gated flow emitting a leg only at some stages
  still contributes (no dropped stock).
- **Stage states keep `n`:** RK4 reads a time-varying schedule at the step's `n`
  for all four stages (forcing piecewise-constant within a step).
- **Referential integrity:** a flow emitting a leg on an unknown stock id raises
  at apply.
- **Determinism + registration-order:** a step is bit-identical across runs and
  under flow/stock registration shuffle (Hypothesis), validating canonical reduce
  in the apply path.
- **Protocol satisfaction:** `EulerIntegrator`/`Rk4Integrator` satisfy the
  `Integrator` Protocol (`isinstance`).

---

## Step 8 design — conservation ledger + per-step assertion

*Design pass (settled with advisor); implement test-first.* Realizes step-algorithm
#7 ("per quantity, `|inputs − outputs − ΔStored| < tol`, failure raises"), the
"Conservation" exit gate, and the `conservation.py` module in the repo layout.

### The invariant — the augmented system is closed (#13)
Every Phase-0 state change is **balanced**: a flow has `Σ legs == 0` per quantity
(arbitration scales *whole* flows, preserving that), and extinction's loss-sink
routing is a balanced *non-flow* change (withdraw from the POPULATION, deposit the
residual into the loss-sink). So over the augmented (modeled + boundary) closed
system, **per asserted quantity the total mass across all stocks is unchanged
step-to-step**, within tolerance:

  `residual_q = Σ_{stocks s of quantity q} (after[s].amount − before[s].amount) ≈ 0`.

This is the **bug-resistant form** of the plan's `inputs − outputs − ΔStored = 0`:
they are equal up to sign (`inputs − outputs = −boundary_delta`, so
`inputs − outputs − ΔStored = −(boundary_delta + stored_delta) = −residual`), but
the total-mass residual needs only the hard `StockKind` partition for the
*diagnostic* decomposition — never for the pass/fail — so there is no sign/
classification surface in the gate itself.

### Why this is **not** "every state change is a flow"
Tempting shortcut: wrap the whole-step per-stock delta as one `FlowResult` and reuse
`assert_flow_balanced`. **Rejected** — it mis-states the step-3 caveat that
extinction's loss-sink routing is a *balanced non-flow* change. The gate reasons
about **state deltas**, not flows. So `conservation.py` computes the residual itself
and shares with `flow.py` only the contract constants (`ASSERTED_QUANTITIES`,
`BALANCE_ATOL`/`BALANCE_RTOL`, `ConservationError`) — which *is* the real drift risk,
and is covered.

### New module — `simcore/conservation.py` (pure stdlib)
- `QuantityLedger` (frozen, clean accounting): `(quantity, boundary_delta,
  stored_delta, residual)`.
  - `boundary_delta = Σ Δ over BOUNDARY stocks` of the quantity — this *is* the
    ledger's net Input/Output (#13); the net flux *into* the modeled system is
    `−boundary_delta`.
  - `stored_delta = Σ Δ over POOL + POPULATION stocks` — the modeled ΔStored.
  - `residual = boundary_delta + stored_delta` (≡ total-mass Δ; should be ~0).
  - No `scale` field — the relative-tolerance basis is a tolerance detail, computed
    inside `assert_conserved`, not stored in the diagnostic record.
- `compute_ledger(before, after) -> tuple[QuantityLedger, ...]` — pure; covers every
  quantity **present** in the stocks (incl. `ENERGY` as a diagnostic, mirroring
  `per_quantity_residual`), in canonical quantity-name order. Per-stock deltas are
  summed in **sorted stock-id order** (#15 — "every reduction"; float sums are
  non-associative, so this is required for bit-identical-under-shuffle, not a
  nicety). Asserts `before`/`after` share the same stock-id key set (an engine-bug
  guard: Phase-0 never adds/removes stocks mid-run).
- `assert_conserved(before, after, *, atol=BALANCE_ATOL, rtol=BALANCE_RTOL) -> None`
  — raises `ConservationError` (imported from `flow.py`; no cycle) for any
  **`ASSERTED_QUANTITIES`** member whose `|residual| > atol + rtol * scale`, where
  `scale = max |per-stock Δ|` for that quantity (transfer-scaled, matching
  `assert_flow_balanced`'s scale notion — see below). Thresholds the residual
  `compute_ledger` already computed (one residual computation; DRY within the
  module). ENERGY is skipped (balance-exempt, #8).

### Tolerance: transfer-scaled, with a flagged future revisit
`scale = max |per-stock Δ|` matches the flow helper. For Phase-0 magnitudes
(O(1)–O(1e3)) `rtol·max|Δ|` sits comfortably above the stored-rounding floor
(~`eps·|amount|` ≈ 1e-13 here), so the gate stays tight. **Watch-item:** the
*stored*-state residual's rounding error scales with `|amount|`, not transfer — so at
~1e6+ amounts with tiny transfers, an `amount`-scaled basis (`Σ|amount|`) would be
the principled choice. Not Phase 0; noted so a future reader doesn't read
significance into the transfer-scaling.

### Wiring — always-on, inside the integrator (#7 / CLAUDE.md)
`step_report` calls `conservation.assert_conserved(state, nxt)` **after the
extinction pass** (`nxt` is the post-extinction state). Always-on, every step — "a
failure is a bug", not an opt-in harness check; this matches arbitration /
extinction / referential-integrity already living in the integrator. **No `ledger`
field is added to `StepReport`**: there is no consumer until the step-10/11 demo, and
adding one now repeats the speculative-param smell step 6 correctly avoided. The
decomposition *exists and is tested* now; surfacing it waits for observation.

### Step-8 test plan
- **Conserves on a balanced step** (Euler + RK4): a multi-flow scenario steps without
  raising; `compute_ledger` residual ~0; `boundary_delta + stored_delta == residual`.
- **Gate catches a bug (isolated):** an unbalanced `−5/+3` transfer between two
  **ample POOLs of the same quantity** — no over-draw (arbitration can't pre-empt),
  no POPULATION (extinction can't), real stocks (referential integrity can't) — so
  `step` raises `ConservationError`. Also a function-level `assert_conserved` case on
  hand-built before/after states.
- **ENERGY exempt:** an energy-only imbalance does **not** raise (mirrors the flow
  test); `compute_ledger` still reports the ENERGY residual (diagnostic).
- **Conserves under arbitration:** the throttled-Euler over-draw scenario passes the
  gate (min-scaling preserves whole-flow balance) — the suite table names
  "incl. arbitration & extinction events".
- **Conserves across extinction:** a snap-to-zero step passes the gate (residual ~0
  once the loss-sink deposit is counted).
- **Ledger decomposition:** a Harvest-like flow (modeled POOL → boundary sink) yields
  `boundary_delta` = the sink's gain, `stored_delta` = the pool's loss, `residual` ~0.
- **Tolerance applied:** a residual just within `atol + rtol·scale` passes; just
  outside raises (the relative term is exercised, not just `atol`).
- **Registration-order independence:** `compute_ledger`/`assert_conserved` are
  bit-identical under flow/stock registration shuffle (Hypothesis), validating the
  sorted-stock-id reduction (#15).
- **Key-set guard:** `compute_ledger` on before/after with differing stock-id keys
  raises (engine-bug guard).
- After wiring, the **full suite stays green** — the verification that no existing
  scenario silently violated conservation.

---

## Step 10 design — two-domain demo (Biosphere + Boundary)

*Design pass (settled with advisor); implement test-first.* Realizes the scope's
"Minimal two-domain demo" and the cross-domain + internal-resolver exit gates. This
is the first assembly that wires *all* the prior steps together: stocks across two
domains, three flows (one cross-domain), a `SourceResolver`, both integrators, the
arbitration/extinction/conservation tail. It builds **no new core machinery** — it
is the integration that proves the spine. Trivial laws only: **no FvCB, no
saturating kinetics** (Phase 1).

### New modules
- `src/domains/biosphere/flows.py` — three frozen-dataclass `Flow`s. All are
  **strictly proportional (first-order) and dt-linear** (`leg = dt·rate`, `rate`
  `dt`-independent — the step-6 increment-form contract), so RK4 stays 4th order:
  - **`Photosynthesis`** `atmospheric_c → plant_c`, flux `= k_photo·light·atm_c·dt`.
    `light = env.get(light_var)` is read as a **scalar rate multiplier**, *not* a
    consumed leg (energy is structure-only in Phase 0 — decision #8). This is the
    flow that carries the **internal-resolver** case.
  - **`Respiration`** `plant_c → atmospheric_c`, flux `= k_resp·plant_c·dt`.
    Autonomous (no `env`).
  - **`Harvest`** (cross-domain) `plant_c → boundary.outside_c`, flux
    `= k_harv·plant_c·dt`. Exercises the registry cross-domain path + balanced
    boundary exchange (#13): harvested carbon leaves the modeled system *into* a
    boundary reservoir, so the flow stays internally balanced.
- `src/domains/biosphere/demo.py` — the **scenario assembly** (lives under the
  biosphere domain — it is the biosphere's showcase — even though it wires in
  boundary reservoirs). Stock-id constants, a frozen `DemoParams` (coefficients +
  initial amounts + `light` level + `dt`), `build_demo(params) -> (State, Registry)`,
  and the two resolver builders (`forcing_resolver` / `coupled_resolver`). A thin
  `run(integrator, state, resolver, dt, steps) -> (State, total_rationed, events)`
  helper folds the per-step `step_report` loop (summing `rationed`, collecting
  events) so the well-fed gate and golden run share one driver.

### Stocks (carbon-only dynamics; one inert energy driver)
`DomainId("biosphere")` (the *real* namespace — `test_integrator`'s `"bio"` is
test-local) and `BOUNDARY_DOMAIN`:
- `biosphere.atmospheric_c` — **POOL**, carbon, ~`1000`.
- `biosphere.plant_c` — **POPULATION**, carbon, ~`100`, `extinction_threshold=0.0`.
  POPULATION is the honest biomass label (#6); dynamically cosmetic here (POOL and
  POPULATION are both "stored", and the snap condition `amount < 0 and amount != 0`
  never fires while plant_c stays positive), but it makes this a **loss-sink-bearing
  assembly** — the plan wanted the boundary machinery built "even though the closed
  demo barely exercises it".
- `boundary.outside_c` — BOUNDARY `sink` (Harvest's destination; its per-step delta
  *is* the carbon ledger's Output, #13).
- `boundary.light` — BOUNDARY **ENERGY `source`** (`unclamped`), constant level `L`,
  **never touched by any leg** — it exists purely to be *read* (the scalar driver).
  Its delta is always 0; ENERGY is balance-exempt (#8), so it appears in
  `compute_ledger` only as a 0 diagnostic.
- `loss_sink(CARBON)` — the documented "step-10 init **must** include the boundary
  loss-sinks" requirement (step-4 design); inert here (no extinction), present so the
  assembly is referentially complete.

Initial **amounts are kept O(1)–O(1e3)** deliberately: the always-on conservation
gate (`tol = atol + rtol·max|Δ|`) runs *every step of this run*, and at these
magnitudes the float residual (~1e-13) sits far under tol — we avoid the step-11
"large amounts / tiny transfers" watch-item by construction, not by luck.

### Well-fed is a structural bound, not "gentle constants" (the backstop gate)
The plan treats a nonzero rationing counter as a **hard gate failure**. Because every
withdrawal is first-order in the stock it draws from, the demand/available ratio on
any clamped stock is exactly `(Σ rates on that stock)·dt` — **independent of
trajectory and level**. So the entire well-fed guarantee reduces to two inequalities:
- `k_photo·light·dt < 1`            (atmospheric_c — only Photosynthesis withdraws)
- `(k_resp + k_harv)·dt < 1`        (plant_c — Respiration + Harvest withdraw)

Pick constants with margin (`rate·dt ≈ 0.1`, e.g. `dt=0.5`, `k_photo=0.2`, `L=1.0`,
`k_resp=0.1`, `k_harv=0.05`). This also clears RK4: a perturbed stage that drove a
stock negative would *flip* its first-order term to a deposit (no withdrawal → no
demand), and with `rate·dt < 1` the stages stay positive anyway — so RK4 never hits
its hard-error path. **The proof says it is safe; the gate is still asserted
empirically** over the full run (`Σ rationed == 0` for Euler; no `ArbitrationError`
for RK4). Keeping all three flows strictly proportional is load-bearing — a
constant/zeroth-order withdrawal would break the trajectory-independent bound.

### The internal-resolver case = bit-identical forcing ≡ shared (decision #16)
The demo state **always** includes `boundary.light`; the *only* difference between
the two runs is how `"light"` is wired:
- `forcing_resolver`: `SourceResolver(forcings={"light": constant(L)})`.
- `coupled_resolver`: `SourceResolver(shared={"light": boundary.light})`.

Because no flow touches `boundary.light`, it stays `L` at every step **and every RK4
stage** (perturbation only shifts stocks with deltas), so `env.get("light")` returns
`L` identically in both wirings → the two full runs are **bit-identical**. That is
the integrated form of decision #16 (the reader cannot tell forcing from shared
stock), and it makes the golden **wiring-independent** — so the **canonical golden
uses `coupled_resolver`**, the wiring that actually exercises the #16 read path each
step.

### Deliberate deviation: config loader deferred to step 11
`DemoParams` is a frozen dataclass — coefficients are **declared data injected into
the flows**, not magic numbers buried in flow logic, so "parameters are data" holds
in spirit. The full **YAML + pydantic + pint** `config/` loader (and
`domains/biosphere/params/*.yaml`) is **deliberately deferred to step 11**, which
owns the **Units** exit gate ("param load / flow registration reject dimensional
mismatch") — consistent with the plan's repeated deferral of pint validation to "the
param-loader step". This is called out as a *conscious* deviation from the CLAUDE.md
"parameters are data (YAML + pydantic)" invariant, **not** a silent one; step 11 must
land `config/` + the `params/*.yaml` + the Units gate so it does not fall through a
crack. (`observe`/`Observation` is likewise a step-11 concern — step-10 tests read
`state.stocks[id].amount` directly and the golden round-trips full `State` via the
step-9 `sim_io`.) **Landed in step 11, cluster 1** — see *Step 11 design — config
loader + Units gate (Scope A)* below; `DemoParams`' inline defaults are gone and
`params/demo.yaml` is now the single source of truth.

### Step-10 test plan
- **Cross-domain Harvest:** evaluating `Harvest` against the demo state is
  `assert_flow_balanced` (carbon) and `domains_touched == {biosphere, boundary}`
  (reuses the step-3 helpers).
- **Internal-resolver indistinguishability (headline):** a multi-step run under
  `forcing_resolver` vs `coupled_resolver` is **bit-identical** (exact `==` on all
  amounts), for **both** Euler and RK4.
- **Well-fed backstop gate:** an Euler run sums `rationed == 0`; an RK4 run completes
  with **no** `ArbitrationError` (over the full run length, not one step).
- **Conservation / boundary exchange:** the always-on gate not raising over the run
  *is* the per-step assertion; plus an explicit `compute_ledger` check at a step that
  `boundary_delta + stored_delta == residual ≈ 0` and outside_c's gain equals the
  carbon drained from atm+plant (Inputs/Outputs = boundary delta, #13).
- **No spurious extinction:** the well-fed run emits **no** events (plant_c
  POPULATION never snaps).
- **dt-linearity:** each demo flow's legs scale ×2 from `dt` to `2·dt` (the step-6
  contract, applied to the real demo flows — incl. Photosynthesis reading `env`).
- **Registration-order independence + determinism:** the run is bit-identical under
  flow-registration shuffle (Hypothesis) and across repeat runs.

---

## Step 11 design — config loader + Units gate (Scope A — cluster 1)

Step 11 is a *bundle*; this is its first cluster — the outer **`config/` loader**, the
demo **params YAML**, and the **Units** exit gate carried forward from step 10. The
remaining clusters (convergence/drift oscillator, `simcore`-purity test, golden demo
regression snapshot, `observe`/`Observation`, the conservation-tol carry-forward, and
the API freeze) are tracked in sequencing item 11 and land after.

### Layering — the spine stays headless
The pure core (`simcore`) and the simulation spine (`domains/biosphere/{flows,demo}`)
stay **stdlib-only**. pydantic / pint / yaml are confined to:
- `config/` — `errors` (`ConfigError`/`UnitValidationError`), `loader.load_yaml` (safe
  `safe_load` read + shape guard), `units.to_canonical` (pint dimensional check +
  conversion to the canonical unit).
- `domains/biosphere/loader.py` — the **only** biosphere module importing the config
  stack; holds the pydantic schema and the field→`Quantity` map, returning a plain
  `DemoParams` in canonical units. `demo.py` no longer carries inline defaults —
  `params/demo.yaml` is the single source of truth (the "parameters are data"
  invariant, now in letter, not just spirit — this closes the step-10 deviation).

### Scope A — what the Units gate validates now (advisor-confirmed)
Only conserved-quantity **amounts** are unit-validated against the canonical-unit table
(decision #9): each amount is a unit-bearing string (`"1000.0 mol"`), parsed by pint
and converted to its quantity's canonical unit, or rejected as a `UnitValidationError`.
**Rate coefficients are dimensionless** schema-validated floats (`gt=0`); full rate-law
dimensional closure is deferred to Phase 1 — it would require new per-leg dimensional
signatures on the **about-to-be-frozen `Flow` protocol**, and adding API right before
the freeze is backwards.

The Units exit gate therefore has two halves now: **param-load** (this cluster) and the
**eval-time** `assert_flow_balanced`, which already resolves each leg's quantity via
stock lookup and so catches a carbon-leg-into-an-oxygen-stock mismatch. Rate-law closure
is the Phase-1 third half.

### Determinism is unchanged
The demo amounts are declared **in** their canonical unit, so conversion is identity and
the loaded floats are **bit-identical** to the step-10 inline defaults — the golden run,
the well-fed bound, and every determinism gate are unaffected. (A pydantic gotcha pinned
here: amount fields are typed `str`, not `float`, or the unit-bearing string would be
rejected before pint ever runs.)

### Cluster-1 test plan (`tests/test_config.py`)
- `to_canonical` accepts identity + compatible conversions (kmol→mol, kJ→J, exact);
  rejects wrong dimension (kg / J as carbon, mol / kg as energy), bare dimensionless,
  and unparseable units — all as `UnitValidationError` (a `ConfigError` subclass).
- `load_yaml` reads a mapping; wraps missing-file / non-mapping / invalid-YAML as
  `ConfigError`.
- `load_demo_params()` yields the exact canonical floats and `DEMO_PARAMS_PATH` exists;
  the full schema + unit gate fires end-to-end through the real loader (kg amount →
  `UnitValidationError`; kmol amount → converted; non-positive rate/`dt`, unknown /
  missing field → pydantic `ValidationError`).
- `test_biosphere_demo.py` now sources params via `load_demo_params()` (a `DEMO_PARAMS`
  module constant), proving the YAML drives the existing step-10 gates unchanged.

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
| **Units** | param load rejects a dimensional mismatch on conserved-quantity *amounts* (Scope A, step-11 cluster 1); eval-time `assert_flow_balanced` rejects a leg landing in a wrong-quantity stock. Full rate-law dimensional closure → Phase 1. |

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
    quantities.py  state.py  flow.py  boundary.py  registry.py
    integrator.py  arbitration.py  conservation.py
    environment.py observation.py  determinism.py  events.py  rng.py
    # (no domain.py: the Phase-0 "Domain" primitive is the registry's domain
    #  index, not a rich class — see Step 3 design. boundary.py holds the
    #  Boundary-domain reservoir constructors — see Step 4 design.)
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
- [x] State serialization round-trips exactly.
- [ ] Conservation passes every step for all quantities.
- [x] Arbitration backstop: non-negative + conserved under over-draw (step 7);
      counter == 0 on the well-fed demo (step 10).
- [ ] Extinction absorbing state works, conserves mass via the loss-sink, never
      revives; POOL stocks never zeroed.
- [x] Cross-domain flow + internal source-resolver exercised and shuffle-stable
      (step 10: `Harvest`; bit-identical forcing≡shared run).
- [x] Boundary exchange balances (step 10); `unclamped` sources not throttled
      (step 7).
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

1. ✅ Repo + tooling skeleton (`pyproject`, `uv`, `ruff`, `pytest`, CI-less local).
   *Done:* uv-managed Python 3.13, committed `uv.lock`, multi-package src-layout,
   all gates green (ruff/pytest+hypothesis/pyright), smoke test resolves packages.
2. `quantities`, `Stock` (+ `kind`/`unclamped`), `State` (integer `n`),
   counter-based `rng`, units-at-boundary.
   *Done:* `simcore.{ids,quantities,state,rng}` — frozen `Stock`/`State`
   (MappingProxyType snapshot, integer `n`, finiteness + key/id guards),
   counter-based keyed splitmix64-finalizer `rng` (order-independent, 64-bit
   masked, `key:tuple[int]`) with cross-port golden hex vectors. *units-at-
   boundary:* the **core-side** seam is built (the `Quantity→UnitLabel`
   canonical table is the single source of truth, #9); the **pint validation at
   the `config/` boundary is deferred to the param-loader step** — there is
   nothing to validate until params exist, and the "Units" exit gate lives in
   the step-11 suite. *(Landed in step 11, cluster 1: the param-load half — pint
   validation of conserved-quantity amounts at the `config/` boundary, Scope A.)*
3. ✅ `Flow`/`Leg`/`FlowResult` + registry + cross-domain flows.
   *Done* (built test-first against "Step 3 design" above): frozen `Leg`/
   `FlowResult` (one-leg-per-stock, NaN/Inf-rejecting, list→tuple coercion),
   `Flow` Protocol (read-only `id`/`priority` so frozen flows satisfy it),
   minimal `Environment` Protocol (interface only — backends are step 5).
   Balance + referential integrity are *evaluation-time*: helpers
   `per_quantity_residual` / `assert_flow_balanced` (raises `ConservationError`;
   `abs(residual) <= atol + rtol*scale`; canonical leg-sort #15) / `domains_touched`
   over an evaluated `FlowResult`. Registry does structure only: dup-id reject +
   canonical id-sorted iteration + a `Stock.domain` index = the Phase-0 "Domain"
   primitive. ENERGY-exempt `ASSERTED_QUANTITIES` + `BALANCE_ATOL`/`BALANCE_RTOL`
   are constants in `quantities.py`, shared with step 8. Tests: flow / registry /
   environment incl. a Hypothesis registration-order-independence property; all
   gates green (ruff, pyright, pytest).
4. ✅ Boundary domain: reservoir stocks (`unclamped` sources + loss-sink).
   *Done* (built test-first against "Step 4 design" above): `simcore/boundary.py`
   — `BOUNDARY_DOMAIN`; `source`/`sink`/`loss_sink`/`loss_sinks` constructors
   (unit derived from quantity; `source` defaults `unclamped=True`); per-quantity
   loss-sink identity `loss_sink_id`/`is_loss_sink` over `ASSERTED_QUANTITIES`.
   Hardened the step-2 `Stock` with an `unclamped ⇒ BOUNDARY` guard (a
   conservation single-point fix, not tidiness). Tests: boundary constructors +
   loss-sink identity + the conservation half of the Boundary-exchange gate
   (reusing step-3 `assert_flow_balanced`) + loss-sink routing conserves + the
   guard. All gates green (ruff, ruff format, pyright, pytest — 78 passed).
5. ✅ `Environment` source resolver (forcing-at-`n·dt` + snapshot-reading
   shared-stock backends).
   *Done* (built test-first against "Step 5 design" above): `simcore/environment.py`
   — `SourceResolver` (build-once, MappingProxyType-wrapped disjoint var maps;
   rejects a var wired as both forcing and shared) + `BoundEnvironment` (frozen,
   lightweight per-evaluation view satisfying the `Environment` Protocol) +
   `Schedule = Callable[[int, float], float]` and a `constant` helper. Forcing
   branch evaluates `schedule(snapshot.n, dt)` at integer `n` (#14, finiteness-
   guarded — only forcing can leak NaN/Inf); shared branch reads the bound
   snapshot's stock amount (#16). Referential integrity is resolve-time (`KeyError`
   at first `get`). Step-6 seam contract pinned in the design: the integrator binds
   env to the *same* snapshot it passes to `flow.evaluate`. Tests: constant
   schedule, forcing-at-`n·dt`, shared-snapshot rebinding, the indistinguishability
   gate across several `n`, a Hypothesis branch-isolation property, wiring/overlap
   rejection, resolve-time errors, Protocol satisfaction, plus a mixed-wiring
   dispatch test (one resolver routing both branches — the step-10 demo shape).
   All gates green (ruff, ruff format, pyright, pytest — 95 passed).
6. ✅ Integrator strategy: Euler (with backstop), then RK4 (backstop→hard error);
   atomic apply + canonical reduce everywhere.
   *Done* (built test-first against "Step 6 design" above): `simcore/integrator.py`
   — `Integrator` Protocol (`env` reconciled to the `SourceResolver` binding source;
   `Registry` injected at construction) + `EulerIntegrator`/`Rk4Integrator`. Shared
   `_derivative` binds the resolver to the same stage-state it passes to
   `flow.evaluate` (#16 seam), evaluates flows in canonical id-order, reduces legs
   per-stock in that order (#15). RK4 in **increment form** (legs are `dt·rate`, so
   the ⅙-combine reproduces classical RK4 exactly — advisor-verified); stage states
   keep `n` (forcing piecewise-constant within a step); the combine folds over the
   **union** of stage keys; intermediate stages may go negative (positivity is the
   kinetics' job). Apply-once writes `n→n+1` and owns **referential integrity**
   (unknown leg stock → `KeyError`). Arbitration/extinction are the *deferred*
   step-7 seam (no speculative no-op param). Tests: Euler exactness; RK4 4th-order
   vs Euler 1st-order on analytic exp-decay; the dt-linearity contract; conservation
   of the applied delta (Euler+RK4, via step-3 `assert_flow_balanced`); union-of-keys
   combine; forcing piecewise-constant under RK4; referential-integrity raise;
   determinism + a Hypothesis registration-order-independence property; Protocol
   satisfaction. All gates green (ruff, ruff format, pyright, pytest — 108 passed).
7. ✅ Arbitration backstop + counter; extinction-with-loss-sink; events.
   *Done* (built test-first against the Frozen-API step algorithm #3/#6 + decisions
   #4/#6/#13/#15): `simcore/arbitration.py` — single-pass min-scaling
   (`min_scaling` → scaled whole flows + firing count; `check_no_overdraw` →
   `ArbitrationError`). Demand `Σ|withdrawals|` summed in canonical flow-id order
   (#15), `unclamped` BOUNDARY sources skipped (never throttled, impose no
   constraint, #13), `scale_s = min(1, available/demand)` on start-of-step levels,
   `scale_f = min` over the clamped stocks a flow withdraws from; whole-flow scaling
   keeps every quantity balanced (the conservation-safety proof). **Euler-only**:
   `EulerIntegrator` scales + reports firings; `Rk4Integrator` makes a needed
   `scale_f < 1` a hard error per stage (positivity under RK4 is the kinetics' job).
   `simcore/events.py` — frozen `ExtinctionEvent(n, stock, quantity, residual)`.
   Integrator extinction pass (post-apply, scheme-independent): POPULATION stocks
   with `amount < threshold and amount != 0` snap to 0, residual routed to
   `loss_sink_id(quantity)` (KeyError if the loss-sink is absent — referential
   integrity), event emitted in canonical id-order. The `amount != 0` guard makes
   extinction absorbing without event-spam (no re-fire at 0; sub-threshold "noise"
   re-snapped, not revived; supra-threshold scenario inflow survives). Diagnostics
   are **functional, not mutable**: `step_report(...) -> StepReport(state, events,
   rationed)`; the frozen `step(...) -> State` is `step_report(...).state` — the core
   keeps no mutable event log/counter (advisor: preserves purity, dodges the
   integrator-reuse re-accumulation footgun). Existing well-fed/POOL tests stay
   bit-identical (scale_f==1 reuses the result object; extinction is POPULATION-only).
   Tests: proportional sag + non-negativity + conservation; `unclamped` skip;
   multi-quantity whole-flow balance; RK4 hard-error vs well-fed; registration-order
   independence under arbitration (Hypothesis); extinction snap/route/event
   (Euler+RK4), POOL-never-zeroed, absorbing (no re-fire / re-snap / survive),
   missing-loss-sink KeyError; event data shape. All gates green (ruff, ruff format,
   pyright, pytest — 129 passed).
8. ✅ Conservation ledger (Inputs/Outputs = boundary deltas) + per-step assertion.
   *Done* (built test-first against "Step 8 design" above): `simcore/conservation.py`
   — `QuantityLedger` `(quantity, boundary_delta, stored_delta, residual)`;
   `compute_ledger(before, after)` decomposes the per-step change per quantity
   (BOUNDARY deltas = the ledger's Inputs/Outputs #13, POOL+POPULATION = ΔStored),
   summing per-stock deltas in **sorted stock-id order** (#15); `assert_conserved`
   thresholds the total-mass residual (`= boundary+stored`) at
   `atol + rtol·max|Δ|` over `ASSERTED_QUANTITIES` (ENERGY exempt #8), raising the
   shared `ConservationError`. The residual is the bug-resistant form of
   `inputs − outputs − ΔStored` (equal up to sign by #13, no sign/classification
   surface in the pass/fail). The gate is **always-on**: the integrator's new shared
   `_finalize` tail runs extinction → `assert_conserved(state, nxt)` → `StepReport`,
   so neither scheme can skip it ("conservation is asserted every step — a failure is
   a bug"). Deliberately **not** reusing `assert_flow_balanced` over a whole-step
   delta (that mis-states extinction's balanced *non-flow* routing as a flow); shares
   only the contract constants with `flow.py`. No `ledger` field on `StepReport` yet
   (no consumer until step 10/11). Tests: balanced-step conservation (Euler+RK4),
   isolated gate-catches-bug raise, ENERGY-exempt-but-reported, conserves under
   arbitration + across extinction, boundary/stored decomposition, the relative-tol
   term, a Hypothesis insertion-order-independence property, key-set guard. All gates
   green (ruff, ruff format, pyright, pytest — 142 passed).
9. ✅ Outer `sim_io` snapshot round-trip (JSON; hex-float goldens).
   *Done* (built test-first against the "Serialization round-trip" exit gate):
   `src/sim_io/snapshot.py` — `State ⇄ JSON` via `state_to_dict`/`state_from_dict`
   + `dumps`/`loads` (re-exported from `sim_io`). Floats (`amount`,
   `extinction_threshold`) serialize as **hex-float strings** (`float.hex()`/
   `fromhex`) — bit-exact for every finite double incl. `-0.0`/subnormals, C99
   cross-port, and keeps the JSON free of native NaN/Inf tokens. **`rng_seed` is a
   `0x`-hex string, not a JSON number** (advisor catch): a full 64-bit seed loses
   precision above 2**53 in an f64 reader (Rust/JS), which a Python-only round-trip
   would hide — same exactness discipline as decision #12. `n` stays a native int.
   Stocks serialize as a **list sorted by id** (canonical #15; each carries its own
   id, so no key/id divergence); `unit` is stored verbatim (no silent re-derive).
   Reconstruction routes through `Stock(...)`/`State(...)`, so every core invariant
   re-fires (NaN/Inf reject, `unclamped⇒BOUNDARY`, key/id, `n>=0`) — a tampered
   golden fails loudly at load. A `version` marker + reject-unknown-version guard is
   the one narrow exception to the project's anti-speculation norm (a format is the
   rare place forward-compat can't be retrofitted), with **no** migration machinery.
   Committed golden `tests/regression/golden/state_snapshot.json`, compared
   **byte-exact** (`dumps(...).encode() == read_bytes()` — not text-mode, dodging
   the Windows CRLF desync). Tests: round-trip equality + per-amount hex-exactness
   (nasty doubles), >2**53 seed survival, byte-exact golden + golden-loads-back,
   bit-identical continuation (Euler step on round-tripped == on original),
   insertion-order independence, canonical id-order output, fail-loud tampering
   (NaN/Inf, unclamped-non-boundary), version guard. (Layering — `simcore` never
   imports `sim_io` — holds; the dedicated purity test is step 11.) All gates green
   (ruff, ruff format, pyright, pytest — 169 passed).
10. ✅ Two-domain demo (Biosphere + Boundary; `Harvest` cross-domain flow; internal
    resolver case).
    *Done* (built test-first against "Step 10 design" above): `domains/biosphere/`
    — `flows.py` (`Photosynthesis` reading `env.get("light")` as a scalar driver,
    `Respiration`, cross-domain `Harvest`; all first-order/dt-linear) + `demo.py`
    (`DemoParams` declared-data coefficients, `build_demo` → two-domain `State` with
    `boundary.{outside_c,light}` + a carbon loss-sink + the flow `Registry`,
    `forcing_resolver`/`coupled_resolver`, a `run` driver folding `step_report`).
    The internal-resolver case is proven as a **bit-identical forcing≡shared run**
    (#16, both integrators); the **well-fed backstop gate** holds as a
    trajectory-independent first-order bound, asserted empirically (`rationed == 0`
    over 200 Euler steps / no RK4 `ArbitrationError`); Harvest is cross-domain +
    carbon-balanced; the run conserves carbon via boundary exchange (#13) and emits
    no events; dt-linearity, determinism, and registration-order independence all
    green. No new core machinery. **Deliberate deviation** (advisor-flagged): the
    YAML + pydantic + pint `config/` loader is deferred to step 11 (see below) —
    `DemoParams` keeps coefficients as injected data in the meantime. All gates green
    (ruff, ruff format, pyright, pytest — 183 passed).
11. Full test suite + golden snapshot; freeze API.
    *Carry-forward from step 8:* the conservation gate is **transfer-scaled**
    (`tol = atol + rtol·max|Δ|`). The golden run is where `BALANCE_RTOL` first meets
    real demo magnitudes — confirm the per-step residual stays comfortably under tol
    at the demo's actual amounts; if the goldens are large amounts with small
    transfers, switch the scale basis to amount-scaled (`Σ|amount|`). Flagged, not
    changed now.
    *Carry-forward from step 10 (owns these — do not let them fall through a crack):*
    ✅ **cluster 1 done** — built the outer **`config/` loader** (YAML `safe_load` +
    pydantic schema + pint dimensional validation) and `domains/biosphere/params/demo.yaml`,
    replaced `DemoParams`' inline defaults with loaded params, and landed the
    **param-load half of the Units exit gate** (amounts unit-validated against the
    canonical table; rates dimensionless — **Scope A**, rate-law closure → Phase 1; the
    eval-time half is the existing `assert_flow_balanced`). See *Step 11 design* above.
    ✅ **cluster 2 done** — the `simcore`-**purity test** (`tests/test_simcore_purity.py`):
    a static AST scan asserting every core file imports stdlib or `simcore` only (allow-set
    deliberately tight, so it also catches a core→`sim_io`/`config`/`domains` leak), via a
    pure `_third_party_imports` helper with discrimination controls (flags `numpy`/`pint`)
    and a non-vacuous-discovery guard. The spine (`domains/biosphere`) purity check is a
    cheap extension of the same helper, deferred to a later cluster. Still owed: the
    **convergence/drift** oscillator gate, the golden demo regression snapshot,
    `observe`/`Observation`, and the API freeze.
```
