# Phase 7 — Native Core (the Rust port)

**Status: PLANNED — not started. Awaiting Step-0 kickoff.** This is the plan of record for
porting the **frozen** multi-domain station (Phase 6's `docs/station-reference.md` +
`docs/biosphere-reference.md`) to a native Rust core, validated against the frozen golden
suite within a **defined tolerance contract**. Pre-plan orientation complete and
advisor-reviewed (six sharpenings folded in below; see "Load-bearing findings"). Steps 0–2
are designed concretely here; Steps 3–6 get just-in-time design as each prior step nails the
constraints (the Phase-5/6 discipline). The roadmap's one-line charge (line 7):
*"We port a stable multi-domain engine, not an evolving one."*

## Goal

Reproduce the frozen Python reference — `simcore` + the four Phase-5 siblings + the biosphere
+ the station seams — as a native Rust crate that, replaying the **same** frozen scenarios,
matches the committed goldens within the tolerance contract this plan defines. **No science
happens in Phase 7** (roadmap line 352). The exit criterion (roadmap line 362): *native and
Python outputs match within defined tolerances, for station scenarios, not just biosphere.*

### Non-goals (explicit scope fence)

- **No new science, no calibration, no new domains/flows/scenarios.** The Python reference is
  frozen; Phase 7 is a *translation*, not an evolution. If the port surfaces a Python bug or
  an underspecified corner, that is a **finding routed through the unfreeze discipline**
  (`docs/station-reference.md`), **never** a silent Rust-side "fix" (advisor #6). The Rust
  port has no authority to move the reference.
- **The entire Python `src/` stays untouched.** Phase 7's analogue of "`git diff src/simcore/`
  empty" is **`git diff src/` empty** — Rust lives in a new sibling `rust/` workspace; the
  only Python additions are the *cross-port comparison harness* under `tests/` (which reads
  both ports' output — it is test tooling, not reference code).
- **No C# yet.** The roadmap lists C# as a second port target (for Godot, Phase 8). Phase 7 is
  **Rust-only**. The one durable C# investment made here is that the JSON interchange + the
  comparator are built **port-agnostic** (they validate *any* port's snapshot, not Rust's
  specifically), so C# reuses the whole harness at the Phase-8 boundary for free. (User-scoped
  to Rust; advisor-endorsed.)
- **No Godot, no UI, no WASM.** Those are Phase 8+. Phase 7 delivers a headless native core.
- **`src/lab/` is NOT ported** (dev oracle / convergence / rk45 — analysis instruments, not
  engine), nor is `src/station/perturbations.py` (diagnostics, no golden, explicitly outside
  the freeze), nor `src/domains/biosphere/drift.py` **as engine code** (see Step 5 — the
  drift *summary* is computed Python-side from raw Rust output).

## The load-bearing decision: the cross-port parity contract

Phase 4/6 froze the reference as **byte-identical within a Python build** (hex-float goldens).
That guarantee **does not cross the language boundary**: the frozen scenarios are saturated
with transcendentals — 658 `**`/`math.*` sites across the domains (FvCB photosynthesis, the
weather half-sine, phenology, the Stefan–Boltzmann `T⁴` radiator, ECLSS/thermal equilibria) —
and `exp`/`pow`/`sin` differ at the last ULP between CPython's libm and Rust's. So a raw
byte-compare of a Rust snapshot against a Python golden **would fail on physically-meaningless
noise**. The contract therefore has **three tiers**, applied per **scenario** (advisor #1 —
the tier is a property of the scenario's *evaluation graph*, not of individual flows, because
in a coupled run every downstream flow operates on already-diverged inputs):

### Tier 0 — structural/discrete invariants: EXACT for **every** scenario (the primary gate)

These are integers and classifications; a float divergence large enough to flip one is a
**real port bug**, not last-ULP noise, so they are asserted **exactly** even for
tolerance-tier scenarios (advisor #2 — the divergence tripwire):

- the integer step count `n` and the stock-id **set** (structure never drifts mid-run);
- `events` (the extinction event stream — an extinction firing in one port but not the other
  is a bug) and `rationed == 0` (the well-fed backstop-firing count);
- the **stability signature** — the period class and per-year cycle summary in
  `sealed_energy_drift_summary.json` (period-2 vs period-1, the plant fixed point). This is a
  *classification*, so it is Tier 0, which makes the drift-summary golden a **better** primary
  cross-port acceptance target than a raw final-State hex compare: it pins the *scientific*
  invariant, which is what "faithful port" actually means;
- **conservation holds every step in Rust, independently** — the per-quantity ledger residual
  ≤ the same `atol + rtol·scale`, asserted inside the Rust integrator exactly as in Python.
  This is near-libm-independent (it is the *same* arithmetic over the *same* per-step values)
  and is the single strongest structural-fidelity signal the port has.

### Tier 1 — bit-exact float trajectories: for scenarios with **no transcendental in the graph**

Where the whole evaluation graph is pure arithmetic (`+ − × ÷`, comparisons, `min`/`max`),
IEEE-754 is deterministic across ports *given identical operation order* — and the core's
canonical id-sorted reduction (`#15`) plus ASCII-only ids (Python `str` sort == Rust UTF-8
byte sort) preserve that order. These scenarios get a **bit-pattern-exact** float gate. The
bit-exact set is **small and must be verified, not assumed** (advisor #1):

- the **RNG splitmix64 hex vectors** — pure `u64` integer arithmetic, definitely bit-exact
  (the cross-port conformance target the RNG was designed around; `src/simcore/rng.py` header);
- **candidate:** `MISSION_SCENARIO` / `crew_state.json` — forced constant rates, linear
  depletion, splits are `×` a constant. **Verify transcendental-free before claiming it.**
- **NOT bit-exact:** `power_state.json` — its solar schedule is a **half-sine** (`sin`). Power
  is tolerance-tier despite its flows being linear (advisor #1's exact caution).

### Tier 2 — tolerance-gated float trajectories: everything a transcendental touches

The default for the coupled and biosphere scenarios. The gate is a **relative-deviation band**
on the parsed final-State amounts (reusing `lab/oracle_match.py`'s `max_abs_relative_deviation`
with a per-quantity floor), plus the Tier-0 invariants which still hold exactly. The tolerance
is **measured, not derived a priori** (advisor, closing note): capture the observed cross-port
deviation, set the band **above** the observed last-ULP-propagated noise but **below** any
physically-meaningful drift, and frame it in terms of the sim's *use* — period class exact,
equilibria to N significant figures, conservation to eps-scale. A deviation that *exceeds* the
band is a port bug to hunt, not a tolerance to loosen.

**Comparison mechanics (advisor #5).** Compare **parsed `f64` bit-patterns, not JSON bytes**.
"Byte-identical golden" is a within-Python-build concept; across ports, Rust *emits* the
`sim_io` JSON snapshot (hex-float, the exact interchange format — module already cross-port by
design) and **Python does all parsing and comparison** (Tier-0 exact, Tier-1 bit-pattern,
Tier-2 relative band). This sidesteps any "does Rust's hex-float string match `float.hex()`
byte-for-byte" question entirely — we compare the *values*, decoded. The comparator is written
to validate **any** port's snapshot JSON.

## Load-bearing findings (from orientation, advisor-reviewed)

1. **The core is tiny and purpose-built for this** — `simcore/` is ~2,200 lines of
   **stdlib-only** Python (the purity invariant exists precisely so this port is mechanical).
   The determinism disciplines (integer `t = n·dt`; canonical id-sorted reduction on *every*
   float sum; counter-based RNG masked to 64 bits; ASCII ids so sort orders agree) were all
   written **for** the Rust port. This is the payoff.
2. **RNG is bit-exact-portable but exercised by NO scenario** (`grep` for `.draw(` in
   `src/domains` → zero hits). So RNG conformance is validated by its **own** hex vectors
   (Step 1), fully decoupled from the scenario goldens — the cheapest, purest first win.
3. **Transcendentals are pervasive and force Tier 2** — confirmed by the 658-site count; the
   biosphere and the SB radiator cannot be bit-matched. This is why the roadmap said
   "tolerance-gated" and why Tier 0 (structural) carries the real fidelity weight.
4. **The op-for-op libm trap** (advisor #4): matching the *mathematical* answer is not enough
   — you must mirror the exact primitive CPython called. `T**4` in CPython routes through C
   `pow()`, so Rust must use `powf(4.0)`, **not** `powi(4)` (repeated multiply — bit-different,
   and it *widens* the Tier-2 deviation needlessly). A dedicated audit pass maps every `**` /
   `math.*` site to its exact Rust equivalent (`.powf`, `.exp`, `.sin`, `.ln`, `.sqrt`, …).
5. **`sim_io` hex-float JSON is the ready-made interchange** — designed cross-port (C99
   hex-float, `0x`-hex seed, id-sorted stock list, schema `version` gate). Rust needs only an
   *emitter*; the reader/comparator stays Python.
6. **`lab/oracle_match.py` already provides the Tier-2 band machinery** (`nrmse`,
   `max_abs_relative_deviation`, `within_band`) — the comparator composes it rather than
   inventing tolerance math.

## Port surface (what actually gets translated)

Mirrors the Python package layout so the port stays a line-by-line correspondence:

- **`simcore` (all of it):** `ids`, `quantities`, `state` (Stock/State + invariants),
  `flow` (Leg/FlowResult/balance), `arbitration` (min-scaling + `check_no_overdraw`),
  `conservation` (ledger + every-step gate), `integrator` (Euler + RK4 + extinction + aux +
  `substep`), `registry`, `environment` (SourceResolver bind), `boundary`, `events`,
  `auxiliary`, `multirate`, `rng`, `observation`.
- **The frozen flow set** (16 classes, `station-reference.manifest.json`): power
  (`SolarCharge`/`LoadDraw`/`SelfDischarge`), thermal (`HeatInput`/`RadiatorReject`), eclss
  (`CrewMetabolism`/`CO2Scrubber`/`Condenser`/`O2Makeup`), crew (`OxygenConsumption`/
  `FoodMetabolism`/`WaterBalance`), station seams (`CrewRespiration`/`WaterRecovery`/`Lamp`/
  `Harvest`), **plus the whole biosphere flow set** (delegated but still ported —
  `Allocation`, `MicrobialRespiration`, FvCB photosynthesis, phenology/`ThermalTimeAccumulation`
  aux, allocation, nitrogen, decomposition, transpiration, consumers, the compartment/chamber
  builders).
- **The param loaders + exact-string unit guards** (advisor #6) — 8 station/sibling YAMLs + the
  13 biosphere YAMLs. Rust reads the **same** frozen files (`serde` + a YAML crate); the
  exact-string unit guard is a cheap string compare ported verbatim (it catches a wrong-unit
  param swap port-side too). Units are **not** re-validated with a pint analogue — Python
  already validated at authoring; the frozen files carry canonical-unit floats + a label.
- **The scenario builders + drivers:** `build_{power,thermal,eclss,crew,season,station,sealed}`
  and `run_{power,station,master_day,sealed}` (the two-rate `station.driver.run_master_day`,
  the sealed composition). The `slow_reset`/`annual_reset` re-sow hook.

**Excluded (Step-3 scope cut):** `src/lab/*`, `station/perturbations.py`, `biosphere/drift.py`
(analysis, computed Python-side), `biosphere/demo.py` beyond what a demo golden needs.

## Steps

### Step 0 (P7.0) — workspace + the port-agnostic comparison harness — ✅ COMPLETE

> **DONE.** `rust/` workspace (member `crates/simcore`; `domains`/`station` deferred to
> Steps 3/5 — no speculative empty crates). Hand-rolled C99 hex-float codec pinned against
> 30 Python-emitted vectors (both directions bit-exact; `format` matches CPython spelling).
> Zero-dep `sim_io`-shape snapshot emitter (serialize-only, no invariant logic). Python-side
> `tests/crossport/`: `compare.py` (Tier-0 exact / Tier-1 `struct`-pack bit-exact / Tier-2
> `lab.oracle_match` measured band, compares parsed f64 not JSON bytes, refuses to invent a
> band) + `tiers.json` (all 20 classified by executed-ops, grep evidence; Tier-1 = crew /
> eclss / cabin_gas / water_recovery, verified transcendental-free). Acceptance met: Rust
> `emit_crew` → `sim_io.loads` → `dumps` == `crew_state.json` byte-for-byte (+ `emit_composite`
> covers the aux/multi-composition branches). First CI added. `git diff src/` empty; 20 goldens
> byte-identical; Python 1324 passed + Rust cargo test/clippy green. Commits `f5759ee`, `9694dba`.


Stand up the `rust/` cargo workspace (crates mirroring `simcore` / `domains` / `station`) and,
**Python-side**, the cross-port comparator + a golden-classification table. Concretely:

- `rust/` workspace skeleton; CI runs `cargo test` + `cargo clippy` alongside the Python suite.
- A **portable hex-float codec** in Rust (C99 `float.hex()` ⇄ `f64`) — std has no hex-float
  parse; pin it with a round-trip test against a vector of `float.hex()` strings emitted by
  Python (covers `-0.0`, subnormals, the schema's exact forms).
- A **JSON snapshot emitter** in Rust producing byte-for-byte the `sim_io.dumps` shape (so the
  Python `sim_io.loads` reads it unchanged) — validated by emitting a hand-built State and
  round-tripping through Python `loads`.
- **`tests/crossport/compare.py`** — loads a Python golden and a port snapshot, applies the
  tier rules (Tier-0 exact, Tier-1 bit-pattern via `struct` pack, Tier-2 `oracle_match` band),
  reads each scenario's tier from a **classification manifest** (`tests/crossport/tiers.json`)
  that this step authors by inspecting each golden's evaluation graph. Port-agnostic: takes a
  path to *any* port's JSON.
- **Verify the Tier-1 candidates** here: confirm `crew_state` is transcendental-free and
  `power_state` is not (the half-sine), recording the verdict in `tiers.json`.

*Acceptance:* the harness round-trips a Python State through the Rust codec/emitter and back;
`tiers.json` classifies all 20 goldens with the graph-inspection evidence noted.

### Step 1 (P7.1) — port `simcore.rng` + the splitmix64 hex-vector conformance — ✅ COMPLETE

> **DONE.** `rust/crates/simcore/src/rng.rs` ports `mix64` / `keyed_hash` / `CounterRng`
> (`draw_u64` → `u64`, `draw` → `f64`) with native `u64` wrapping (`wrapping_mul`/
> `wrapping_add`; the Python `& MASK64` masks are implicit in `u64`). The fold order is the
> load-bearing bit: `step` folded first, then each key word, each mixed as
> `mix64((h + GAMMA) ^ word)`. **The three Python `keyed_hash` `TypeError` guards have no Rust
> analogue by design** — a non-`u64` word does not type-check, so the `u64` type system
> subsumes them statically (their absence is the check moving to compile time, not a dropped
> behavior). `draw` is **bit-exact by construction**, not by luck: `x >> 11` is ≤53 bits (`as
> f64` lossless) and the divisor is 2⁵³ (power-of-two division ⇒ no rounding).
>
> **Vectors follow the Step-0 generated-file discipline** (the grid is what justifies the
> machinery over hand-inlining 9 constants): `tests/crossport/gen_rng_vectors.py` computes
> `mix64` (published splitmix64-seed0 + edges) and `draw` (the 6 `_GOLDEN` fixed inputs + a
> seed×key×step grid, 73 rows) from the frozen `simcore.rng`, writing committed
> `rust/crates/simcore/tests/data/rng_vectors.txt`. Rust `tests/rng_vectors.rs` gates every row
> **Tier-1 bit-exact** — `draw_u64` as an exact `u64`, `draw` as its exact `float.hex()`
> spelling decoded through the Step-0 hex-float codec (a mismatch surfaces as a bit difference,
> never rounding noise). **The circularity is dissolved** by
> `test_crossport.py::test_rng_vectors_anchor_to_published_known_answers`: it binds the file's
> fixed rows to the hand-pinned `_GOLDEN` / `_SPLITMIX64_SEED0` in `tests/test_rng.py` (grounded
> against *published* splitmix64), so the chain is Rust == file == Python, Python == published —
> not self-referential. Plus `test_rng_vectors_in_sync` (regen-drift guard). **Zero core +
> zero domain change** (`git diff src/` empty; Rust under `rust/`, generator + tests under
> `tests/crossport/` — permitted test tooling). All 20 frozen goldens byte-identical (no
> regen); `cargo test` + `clippy -D warnings` green; Python suite green incl. `-m slow`.


The tightest, purest constraint first (advisor: "go RNG-vectors first"). Port `mix64` /
`keyed_hash` / `CounterRng` with `u64` wrapping (`wrapping_mul`/`wrapping_add`, `& MASK64` free
in Rust `u64`). Gate against the **existing Python hex vectors** (the `draw_u64` golden vectors
in the RNG tests) — this is a **Tier-1 bit-exact** match, no tolerance. Proves the cross-port
integer discipline end-to-end before any float enters.

*Acceptance:* Rust `draw_u64`/`draw` reproduce the Python hex vectors bit-for-bit.

### Step 2 (P7.2) — port `simcore` engine + a synthetic pure-arithmetic bit-exact gate — ✅ COMPLETE

> **DONE.** Ported the whole engine to `rust/crates/simcore/src/` (12 new modules +
> `error.rs`): `ids`/`quantities` (the two-projection `Quantity::value`/`name` + the
> name-sorted `ASSERTED_QUANTITIES`), `state` (Stock/State + all invariants, re-fired on
> every `with_amount`/`State::new` "replace"), `flow` (Leg/FlowResult/`Flow` trait +
> balance), `arbitration` (min-scaling + `check_no_overdraw`), `conservation` (ledger +
> every-step gate), `environment` (`SourceResolver`/`BoundEnvironment` + the `Environment`
> trait), `boundary`, `events`, `auxiliary` (`AuxProcess` trait), `registry`, `integrator`
> (Euler + RK4 + extinction + aux + `substep`, the `_BaseIntegrator` spine as a private
> `Scheme` trait + shared free-fns), `multirate` (Strang/Lie). Errors are one
> `SimError { Conservation, Arbitration, Validation, Reference }` enum mirroring Python's
> four raise sites (`ConservationError`/`ArbitrationError`/`ValueError`/`KeyError`); the
> three Python `keyed_hash`/`TypeError`-style guards that the type system subsumes stay
> subsumed. **`observation` is a conscious deferral** (a consumer-facing projection; no
> golden is an `Observation`, the cross-port gate compares `State` snapshots) — noted in
> `lib.rs` so Step 6 doesn't assume a complete surface.
>
> **The load-bearing discipline (advisor): op-order, not math, is what bit-exactness
> lives on.** Float `+`/`*` are commutative but not associative, so every arithmetic
> grouping in the integrator mirrors the Python source **character-for-character**
> (`(k1 + 2.0*k2 + 2.0*k3 + k4) / 6.0`, `stock.amount + factor * delta`), and every
> Python `sorted()` is replicated. `BTreeMap<String, _>` gives sorted-by-id iteration for
> free (String UTF-8 order == Python `str` sort for ASCII ids), but the **three distinct
> reduction orders** are each walked over the correct ordered source, never collect-then-
> refold: `reduce`/`_scale_factors` in **flow-order × leg-order**, `per_quantity_residual`
> in **sorted-leg order**, `compute_ledger` in **sorted-stock order**, `aux_increments` in
> **process-order × sorted-name**.
>
> **The gate:** `tests/crossport/gen_engine_vectors.py` defines a **synthetic,
> transcendental-free** scenario *on the frozen `src/simcore`* (a forced unclamped-source
> inflow, a donor-controlled leak + transfer off a pool — so RK4's 4 stages genuinely
> differ — a donor-controlled drain taking a POPULATION stock to **extinction**, one aux
> process reading the pool), runs it under **Euler / RK4 / multi-rate Strang** + a second
> **rationing** scenario (Euler min-scaling fires), and writes the committed flat
> `rust/crates/simcore/tests/data/engine_vectors.txt` (per-step per-stock `float.hex()` +
> aux + rationed + events). Rust `tests/engine_vectors.rs` defines the **same** scenario
> and gates every step **Tier-1 bit-exact** via the Step-0 hex-float codec (stock amounts,
> aux, `rationed` count, extinction event stock/quantity/residual). **No external anchor**
> (unlike the RNG's published splitmix64): `src/simcore` *is* the reference, so Rust ==
> Python is the whole goal; `test_crossport.py::test_engine_vectors_in_sync` only checks
> the file stays in sync. **The advisor's dead-branch catch:** `_combine`'s "missing key ⇒
> 0.0" union fallback never fires in the trajectory (every stage emits the same stock
> set), so it gets a **dedicated Rust unit test** on disjoint-key stages rather than a
> contorted scenario. Error paths are Rust unit tests: an imbalanced flow →
> `ConservationError`; an RK4 over-draw → `ArbitrationError`. Aux stays **frozen** under
> multi-rate (`substep` leaves it untouched — verified: `thermal_time` constant at 7.0
> across the whole multirate run).
>
> **Zero core + zero domain change** (`git diff src/` empty; the Rust lives under `rust/`,
> the generator + test tooling under `tests/crossport/`). All 20 frozen goldens
> byte-identical (no regen). `cargo test` (44: 34 lib + 3 engine + 4 hexfloat + 3 rng) +
> `clippy -D warnings` green; the full Python suite incl. `-m slow` + ruff + pyright green.

Port the whole engine (state/flow/arbitration/conservation/integrator/registry/environment/
boundary/events/aux/multirate). Validate with a **synthetic, transcendental-free scenario**
(a couple of linear forced flows + a boundary sink) run through **both** Euler and RK4 —
gated **bit-exact (Tier 1)** on the full trajectory (advisor #6: get integrator + arbitration
+ conservation + the every-step gate proven pure *before* any domain transcendental muddies
it). This is where the integrator's subtle bits are pinned cross-port: canonical id-sorted
reduction, the RK4 ⅙-combine over the union of stage keys, the extinction loss-sink routing,
the aux single-Euler-increment placement, `substep` keeping `n`.

*Acceptance:* the synthetic scenario is bit-identical Rust↔Python for Euler and RK4; the
every-step conservation gate fires identically; a deliberately-imbalanced synthetic flow
raises `ConservationError` in Rust too.

### Step 3 (P7.3) — port the four Phase-5 siblings; validate the 5 standalone goldens

Port power/thermal/eclss/crew flows + loaders + scenarios + `run_power`. Validate
`power_state`, `power_self_discharge_state`, `thermal_state`, `eclss_state`, `crew_state`.
Expected tiers: **`crew_state` Tier 1** (bit-exact — the verified transcendental-free forced
depletion), the rest **Tier 2** (half-sine solar / `T⁴` radiator / geometric equilibria). This
step exercises the **libm audit** (advisor #4) on its first real transcendentals: `RadiatorReject`'s
`(T⁴ − T_space⁴)` → `powf(4.0)`, the solar `sin`. Tier-0 invariants (`rationed==0`, conservation
every step, steady-state reached) asserted exactly for all five.

*Acceptance:* 5 standalone goldens pass their assigned tier; the measured Tier-2 deviations are
recorded and the bands set above them (the first real tolerance calibration data point).

### Step 4 (P7.4) — port the biosphere; validate the 7 frozen biosphere goldens

The bulk of the work — the clean-room crop science (FvCB photosynthesis, the weather half-sine
+ daylength, phenology/thermal-time aux, allocation, the carbon budget, nitrogen, water cycle,
decomposition/mineralization, consumers, the compartment/chamber builders + `run_season` +
`annual_reset`). All **Tier 2**. The heaviest libm-audit surface (`exp`/`pow`/`log` in FvCB,
`sin` in weather). Sub-divide just-in-time (the plan gets refined here once Step 3 sets the
translation cadence). The biosphere is **Euler-locked by its own freeze** — no RK4 cross-check
needed (matches the Phase-6 greenhouse runs).

*Acceptance:* the 7 frozen biosphere goldens (`biosphere-reference.manifest.json`) pass Tier 2;
Tier-0 invariants exact (the biosphere's period/stability signature included where its goldens
carry one — the Phase-4 drift-summary).

### Step 5 (P7.5) — port the station assembly; validate the 8 station/coupled goldens

Port the station seams (`CrewRespiration`/`WaterRecovery`/`Lamp`/`Harvest`, the inline
composition-stock wiring), `run_master_day` (the two-rate driver), `build_sealed_station` +
`slow_reset`. Validate `station_state`, `cabin_gas_state`, `greenhouse_state`,
`water_recovery_state`, `lighting_state`, `harvest_state`, `sealed_station_state`,
`sealed_energy_drift_summary`. All **Tier 2** on floats; **Tier 0 carries the weight** here —
the sealed multi-year run's whole point is that conservation holds every step and the
stability signature matches. Per advisor #3, the **drift summary is computed Python-side** from
raw Rust final-States / trajectory output (`drift.py` is not ported as engine code); the Rust
side emits the raw states, Python folds them into the summary and compares the **classification
exactly**.

*Acceptance:* the 8 coupled goldens pass; the ~1.3 M-substep sealed Tier-2 run conserves every
step in Rust; the energy drift-summary period class matches exactly (Tier 0).

### Step 6 (P7.6) — full-suite parity gate + the cross-port reference doc; PHASE 7 EXITS

Wire the whole 20-golden cross-port suite into CI (`cargo test` produces the snapshots; the
Python comparator gates them). Write **`docs/native-port-reference.md`** — the cross-port
**tolerance contract** (the mirror of the freeze contracts): the per-scenario tier assignment,
the measured Tier-2 bands with their justification (the "measured, framed by use" record), the
op-for-op libm audit table, and the **discovered-discrepancy protocol** (any Python
bug/ambiguity the port surfaced → routed through the station/biosphere unfreeze discipline,
with the finding recorded). One line noting the harness is port-agnostic and C# reuses it at
the Phase-8 boundary.

*Acceptance (roadmap exit):* native and Python outputs match within the defined tolerances for
**all** station + biosphere scenarios; Tier-0 invariants exact across the board; `git diff
src/` empty (Rust is additive); `cargo test`/`clippy` + the full Python suite green. **Phase 7
EXITS → Phase 8 (Godot front-end).**

## Decisions settled (advisor-reviewed)

1. **Three-tier parity, tiered by scenario not flow** (#1) — divergence propagates through
   coupled graphs, so "which port does this scenario land in" is decided by whether *any*
   transcendental touches its evaluation graph.
2. **Tier 0 (structural/discrete) is the primary gate and is EXACT for every scenario** (#2) —
   conservation-every-step + period class + `events` + `rationed==0` + stock-id set are the
   real fidelity signals; a flipped classification is a bug, not noise.
3. **Hard scope cut** (#3) — `lab/`, `perturbations.py`, `drift.py`-as-engine excluded; the
   drift summary is a Python-side fold over raw Rust output.
4. **Op-for-op libm matching** (#4) — `powf(4.0)` not `powi(4)`; a dedicated audit pass.
5. **Compare parsed `f64`, not JSON bytes** (#5) — Rust emits, Python parses/compares; the
   comparator validates any port's snapshot (C# reuse).
6. **Tolerances measured, not derived** — capture cross-port deviation, gate above the
   last-ULP noise and below physical-drift meaning; frame by the sim's use.
7. **Rust-only; C# deferred** to the Phase-8 boundary, reusing the same interchange + comparator.
8. **The port has no reference authority** — a surfaced Python bug is an unfreeze-discipline
   finding, never a silent Rust fix.

## Exit criteria

- All 20 frozen goldens (13 station/sibling + 7 biosphere) pass their assigned tier;
  Tier-0 invariants exact for every scenario, conservation asserted every step in Rust.
- The RNG hex vectors match bit-for-bit; the synthetic pure-arithmetic scenario matches
  bit-exact under Euler and RK4.
- `docs/native-port-reference.md` records the tier table, the measured bands + justification,
  the libm audit, and the discovered-discrepancy protocol.
- `git diff src/` empty (the Python reference is untouched; Rust + the comparator are additive);
  `cargo test` + `cargo clippy` + the full Python suite (incl. `-m slow`) + `ruff` + `pyright`
  all green.
- **Phase 7 EXITS → Phase 8 (Godot front-end).**
