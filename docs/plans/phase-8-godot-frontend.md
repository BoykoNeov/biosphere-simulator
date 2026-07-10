# Phase 8 — Godot Front-End (the sim becomes visible)

**Status: IN PROGRESS — Steps 0–4 COMPLETE.** This is the plan of record for
putting a **Godot front-end** on top of the **frozen** native Rust core (Phase 7). Pre-plan
orientation complete and advisor-reviewed; two load-bearing scope decisions **USER-CONFIRMED**
(see "Confirmed scope decisions"). Steps 0–2 are designed concretely here; Steps 3–8 get
just-in-time design as each prior step nails the constraints (the Phase-5/6/7 discipline).

The roadmap's charge (lines 363–373):

> **Goal:** *Everything becomes visible. Nothing scientific changes.*
> **Rule:** *The game never computes domain logic. It only displays and manipulates the simulation.*

## Goal

Ship a Godot application that lets a player **build systems, perturb systems, fast-forward
decades, inspect flows, and observe failure or stability** across power · thermal · atmosphere ·
biosphere · crew — while the **exact same simulation** can run headless. Godot owns
visualization, interaction, time controls, objectives, save/load, and UI. It computes **no**
domain logic: every stock, flow, temperature, and conservation check comes from the Rust core.

### The architecture (USER-DECIDED)

**Godot consumes the existing Rust core via GDExtension/FFI. There is NO C# port.** The
roadmap listed C# as an alternate port target for Godot; the user chose instead to bind the
frozen Rust core directly through `gdext` (godot-rust). The Godot MCP plugin
(`addons/gdai-mcp-plugin-godot/`, already staged) is installed/enabled when Step 1 needs it.

### Non-goals (explicit scope fence)

- **No new science, no calibration, no new domains/flows/scenarios.** The Python reference and
  its Rust port are frozen (`docs/station-reference.md`, `docs/biosphere-reference.md`,
  `docs/native-port-reference.md`). Phase 8 *displays and manipulates* a frozen engine; it does
  not evolve it. A surfaced discrepancy is an **unfreeze-discipline finding**, never a silent
  UI-side "fix".
- **No C# port** (architecture decision above).
- **No declarative scenario/species/domain authoring** — the roadmap parks that in **Phase 9**.
  Phase 8 "build systems" means placing & connecting from a **fixed, code-defined component
  palette**; registry construction stays in Rust (confirmed decision #1).
- **No netcode / server infrastructure.** "Runs headless on a server" is satisfied by
  *architecture*: Phase 7 already delivered a headless-capable core, and Phase 8's obligation is
  only **not to break that** — a CLI / `cargo test` harness drives the *same* session to
  bit-identical results (confirmed decision #2). No network protocol, no client/server split.
- **No WASM yet.** Kept *possible* by the purity invariant below, but not built here.

## The load-bearing decision: parity is provable in pure Rust, *before* Godot (advisor #1)

The Phase-8 exit criterion — *"the exact same simulation runs headless"* — reduces to a claim
that needs **no Godot toolchain at all**:

> **Incremental stepping produces a trajectory bit-identical to run-to-completion.**

Each `EulerIntegrator::step_report(&state, resolver, dt)` is a **pure function of the previous
exact `State`**. So a stateful session advanced `N` times must equal the existing
`run_station(N)` **bit-for-bit** — same libm, same code, same op order. The *only* way to break
it is a binding that sneaks in reordering: accumulating `t += dt` instead of `n·dt`, re-batching
with a different `dt`, or letting the UI mutate state mid-loop. That test **is** the "same sim
headless" guarantee, and it lives in `cargo test`. **We establish it in Step 0, before any UI.**
Then Godot is just a caller that structurally *cannot* cheat.

**Corollary (bank it):** the counter-based RNG keyed by `(seed, key, n)` (a core invariant since
Phase 0) makes pause / resume / save / load **trivially deterministic** — there is no sequential
RNG state to serialize. Save/load carries `(scenario-id, State, n)` and nothing else.

**But this test is intra-process and cannot see the boundary that actually matters (advisor #2).**
`N×session.step() == run_station(N)` runs both sides in the *same* `cargo test` process — same
compiled code, same FP environment — so it is nearly tautological and, crucially, **blind to
Godot-hosted-vs-headless divergence**, which is what the exit criterion rides on. The concrete
break risk is **FP control flags (FTZ/DAZ in MXCSR)**: game engines sometimes set flush-to-zero /
denormals-are-zero per-thread for SIMD performance, so if Godot sets them on the thread calling
into Rust, denormal intermediates flush to zero and diverge from the IEEE-default headless run.
Whether Godot does this, and whether our Tier-1 scenarios produce denormals, are **both unknown
from here — verify, do not assume.** Therefore a *genuine cross-boundary* parity check — run ≥1
scenario through the **actual `gdext` cdylib** and compare final `State` to the headless
trajectory — is a first-class deliverable: a **smoke version in Step 1** (first time the boundary
exists; the Tier-1 bit-exact `cabin_gas` is the cleanest tripwire) and the **gating version in
Step 8**. Each carries an explicit "verify FP env (FTZ/DAZ, rounding mode) matches headless" line,
and the cdylib Godot loads must be built from the same profile / target-features as the headless
reference.

## The purity invariant across the language boundary — Phase-8's "`git diff src/` empty"

Phase 7's exit discipline was `git diff src/` empty (Rust lived in an additive `rust/`
workspace). Phase 8's analogue:

- **`gdext` lives ONLY in an additive binding crate** (`rust/crates/godot_bridge`). The engine
  crates — `simcore`, `domains`, `station` — **stay dependency-free and carry NO gdext types in
  their signatures.** Anything the binding needs is an **additive `pub` exposure**, tracked
  exactly like Phase 7 made `crew::carbon_split` pub. This is what keeps the WASM-future and the
  C#-someday options open, and keeps "core is pure" true across the FFI boundary.
- **`git diff src/` stays empty** (the Python reference is untouched). Godot project files live
  in a new top-level location (Step-1 decision — likely the repo root, since the MCP addon is
  already at `./addons/`); the binding crate lives under `rust/crates/`.

## The four named Rust work items (so none surprises us mid-phase — advisor)

1. **The steppable owned-state session.** `run_station` takes an `observer: &mut dyn FnMut(&State)`
   and *owns* the loop; Godot's game loop must own the loop instead (it calls `session.step()`).
   The new surface is a session struct holding `State + registries + resolver`, exposing `step()` /
   `step_n(k)` / `state()`. The per-step primitive already exists — this is mechanical extraction,
   but it is genuine new surface. **It must expose both granularities:** single-rate (Power/Thermal)
   ticks per step; sealed ticks per **master-day** (`run_master_day` = 1440 substeps + 1 biosphere
   step). Parity teeth apply to each. **Fast-forward is free and parity-safe** — "step N without
   observing" is just the loop with a silent observer. (Step 0.)
2. **The display projection (revive `observation`).** `observation` was a *conscious* Phase-7
   deferral. Everything the UI shows is computed **Rust-side**: temperature `T = T_space + Q/C`,
   SOC %, per-domain totals, conservation residual, flow legs for inspection. **Zero parity
   concern** (display-only, derived from the exact `State`) — develop freely without golden
   pressure. This is the clean separation from the parity-critical stepping path. (Steps 2, 4.)
3. **The snapshot *loader*.** `simcore::snapshot` is **emit-only** today. Save/load needs a Rust
   loader. **Scope-coupled to the fixed-palette decision:** a save is `scenario-id + State`, and we
   rebuild the registry from the id — cheap. We do **not** speculatively build a full
   flow/param/wiring serializer (that is Phase 9 / full-authoring territory). (Step 7.)
4. **Port the perturbation primitives to Rust.** `station/perturbations.py`
   (`window_override`, `with_forcing`, `LeakFlow`, `ScaledFlow`) was **deliberately excluded from
   Phase 7** ("diagnostics, no golden"). "Perturb systems" is in the Phase-8 exit criterion, so
   these now need a Rust home. They are forcing-overrides + added flows over the same engine, so
   they inherit the existing Phase-7 tier rules — but it is real work, not free. (Step 5.)

## Confirmed scope decisions (advisor-flagged → USER-CONFIRMED)

1. **"Build systems" = fixed palette (Phase 8), not declarative authoring (Phase 9).** The player
   places & connects from a fixed, code-defined component palette (add a battery, a radiator, a
   scrubber). Registry construction stays in Rust. Save = `scenario-id + State snapshot` (cheap
   loader, item #3). Arbitrary runtime stock/flow/param/wiring definition is Phase 9.
2. **"Runs headless on a server" = architecture-satisfies, no netcode.** Phase 8's obligation is
   only that the Godot layer stays a pure consumer and a CLI / `cargo test` harness drives the same
   session to bit-identical results. No server/network infrastructure.

## Steps

Early steps concrete; later steps just-in-time. Each step: additive-only, `git diff src/` empty,
engine crates carry no gdext types, all frozen goldens byte-identical unless a deliberate,
loudly-stated regen (none expected — Phase 8 changes no science).

### Step 0 (P8.0) — the steppable session + the parity teeth (pure Rust, no Godot) — **COMPLETE**

The riskiest *correctness* claim first, proven without the FFI toolchain: work item #1 (the
session) and the parity gate that makes the exit criterion true. **What landed** (all under
`rust/`; `git diff src/` empty; all 20 frozen goldens byte-identical — verified by re-emitting
`greenhouse`/`harvest`/`lighting`/`sealed_station` and diffing):

- **`station::session::SimSession`** — an enum-backed owned-state struct (`SingleRate` /
  `TwoRate`) with `single_rate(…)` / `two_rate(…)` constructors and `step()` / `step_n(k)` /
  `state()` / `n()` / `total_rationed()` / `events()`. One **mode-agnostic** `step()` advances
  the natural unit (one `step_report` single-rate; one **master day** two-rate) — cleaner for the
  Godot loop than separate `step`/`step_day`. Documents the `(seed, key, n)` determinism
  corollary.
- **The inversion is a shared-code extract, not a re-implementation** (so parity is by
  construction, not by luck): the per-day body of `driver::run_master_day` was extracted into a
  pub **`driver::advance_one_master_day`** that *both* the runner and the session call; the sealed
  re-sow closure was extracted into a pub **`sealed::sealed_reset_hook`** (`OwnedResetHook`) that
  both `run_sealed` and the two-rate session build. Both extractions are behavior-preserving
  (goldens byte-identical, incl. the ~1.3 M-substep sealed run).
- **Parity tests (`tests/session_parity.rs`, `cargo test`)**, states compared by exact hex-float
  JSON (bit-exact): single-rate `cabin_gas` (Tier-1) `N×step()` == `run_station(N)`; two-rate
  greenhouse (`reset=None`) `days×step()` == `run_greenhouse`; two-rate sealed
  (`reset=Some(sealed_reset_hook)`) short-horizon == `run_master_day`; plus an `#[ignore]`d
  full-horizon `run_sealed(915 days)` == `915×step()` (crosses all 3 season boundaries →
  reset-adopt branch; passes in 127 s release).
- **Scope note honored:** this intra-process gate is deliberately *not* the cross-boundary check.
  The genuine `gdext`-cdylib-vs-headless parity (FTZ/DAZ FP-env) is Step 1's smoke + Step 8's gate,
  as designed above.

### Step 1 (P8.1) — the GDExtension binding crate + minimal Godot vertical slice — **COMPLETE**

The riskiest *toolchain* thing (the FFI boundary) on the smallest surface — can't design around a
remembered `gdext`↔Godot version; prove it end-to-end. **What landed** (all additive under
`rust/crates/godot_bridge/`, `godot/`, `tests/crossport/`; `git diff src/` empty; engine crates
untouched; 20 frozen goldens byte-identical):

- **gdext↔Godot resolved empirically:** `godot = "0.5.4"` (has an `api-4-7` feature, defaults to the
  latest bundled API); installed Godot is **4.7.stable**. The load log confirms forward-compat
  (`API v4.6 → runtime v4.7`), so **no `api-custom` fallback**. Single-precision Godot is fine (`f64`
  returns are bit-preserved) — verified by the smoke, not just reasoned.
- **`rust/crates/godot_bridge`** (cdylib, the *only* gdext-dependent crate) wraps the Step-0 session as
  a `SimSession` GDExtension class: `build` / `step` / `step_n` / `step_count` / `total_rationed` /
  `stock_amount` **+ `snapshot_json()`** (Rust-side hex-float JSON — the parity path never leaves the
  golden codec) **+ `mxcsr()`/`fp_clean()`** (inline-asm `stmxcsr`, FTZ/DAZ read on the stepping
  thread). 4 crate unit tests (free-function core, no Godot runtime).
- **`godot/` project** (subdir, not repo root — keeps the importer off `rust/target`/Python/docs):
  `project.godot` + `simcore.gdextension` + `main.tscn`/`main.gd` (the live-Label slice) + `smoke.gd`.
- **The cross-boundary smoke** (`tests/crossport/test_godot_parity.py`, local-only `skipif
  godot||cargo`): drives Tier-1 `cabin_gas` through the *actual cdylib* and asserts the snapshot is
  bit-exact vs headless `emit_cabin_gas` **and** vs the frozen golden, `fp_clean` (FTZ/DAZ off,
  measured `mxcsr=0x1FA0`), and `rationed==0`/`step_count==900`. The Step-8 gating version promotes it.
- **Live-Label render** positively verified headless through the real frame loop
  (`eclss.cabin_o2 = 9.240222 mol`); on-screen pixels are the interactive (user-GUI / MCP) clause —
  the MCP is **not** needed for the load-bearing smoke (advisor).

Original design notes (kept for provenance):

- New additive `rust/crates/godot_bridge` (depends on `station` + `gdext`; **no gdext types leak
  into the engine crates**). A `SimSession` GDExtension class wrapping the Step-0 session:
  `build(scenario_id)`, `step()`, `stock_amount(id) -> f64`.
- A minimal Godot project (location TBD — likely repo root, per the staged `addons/`) that loads
  the `.gdextension`, and a GDScript that steps the sim and renders **one** stock's value in a
  `Label`. Install / enable the Godot MCP (`addons/gdai-mcp-plugin-godot/`).
- **Cross-boundary parity smoke (advisor #2):** run one Tier-1 scenario (`cabin_gas`) through the
  *actual* cdylib Godot loads and compare its final `State` to the headless trajectory — the real
  "FFI didn't corrupt determinism" proof (the Step-0 intra-process test structurally cannot see
  this). Verify the FP env (FTZ/DAZ, rounding mode) on the calling thread matches headless.
- Acceptance: Godot loads the extension, GDScript drives `step()`, one live value renders, and the
  cross-boundary smoke passes bit-exact.

### Step 2 (P8.2) — the display projection (revive `observation`) — **COMPLETE**

Multi-domain state visible. Work item #2. **What landed** (all additive under `rust/` + `godot/`;
`git diff src/` empty; engine crates carry no gdext types; 20 frozen goldens byte-identical — no
science changed):

- **Three layers, split by what each may touch** (advisor):
  1. **`simcore::observation`** — the faithful port of the frozen `observe` surface
     (`Observation { n, stocks }`, `StockObservation { id, domain, quantity, unit, amount }`,
     id-sorted; `amount` held as raw bits for `Eq`/`Hash`). **No aggregates bolted on** — Python's
     `observation.py` deliberately has none, and it is a frozen API. Held to the frozen surface:
     the 7 unit tests port `test_observation.py`'s teeth (exact-copy, id-order, insertion-order
     independence, empty, exact-amount, seed-drop, `n`-in-equality). The Phase-7 `lib.rs` deferral
     note is now the revival note.
  2. **`station::display`** (new) — the *whole* derived-readout layer: `group_by_domain` /
     `per_quantity_totals` (need only `&State`) + `temperature` (reuses `domains::thermal::temperature`
     verbatim) + SOC + `DisplayProjection::to_json` (plain decimal floats — the hex-float parity path
     stays on `simcore::snapshot`). The three things `State` lacks — thermal params, the SOC
     reference, and the shared-stock ids — are **declared per scenario in a `DisplayContext`** the
     bridge fills (the sharing is a *construction-time fact of the assembly*, not recoverable from a
     `Stock`'s single `domain` or a `Flow` — which exposes no static stock refs). **SOC is "% of
     initial charge," not "% of capacity"** (honest label — a POOL battery has no capacity param, so
     the only exact reference is `battery0`; SOC legitimately swings past 100% when charged above
     start).
  3. **`godot_bridge` + `godot/`** — `SimSession::observation_json()` (the P8.2 dashboard read) and
     the `godot/main.gd` multi-domain dashboard (per-domain groups, shared-stock `*` highlight,
     temperature / SOC / residual readouts).
- **Residual is session-level, not observation-level** (a per-step before→after ledger quantity):
  `SimSession` retains the pre-step `State` and computes `compute_ledger(prev, cur)` max-residual
  **on demand** (`max_residual()`), so a caller that never asks pays only the per-step clone. The
  clone is parity-neutral — it does not touch the integrator/op-order, so `session_parity.rs` stays
  bit-exact (verified: all three fast parity cases green).
- **Palette grown to `{cabin_gas, station}`** (advisor: `station` only — already ported, single-rate,
  the one entry with a *real* temperature and battery SOC; two-rate greenhouse/sealed deferred).
  Verified headless through the *actual cdylib* (`godot/dashboard_smoke.gd`): the `station` day-24
  dashboard reads **node T ≈ 160.08 K** (the documented Step-1 `T_eq`), **SOC = 100.0 % of initial**
  (returns to `battery0` after balanced days), `thermal.node` highlighted shared, ENERGY total
  conserved, residual at round-off. `cabin_gas` reports both scalars `null` and highlights the three
  cabin-air pools.
- **Zero parity concern for the new readouts** (display-only, plain floats) — but the frozen `observe`
  port is held to its surface. UI pixels remain the interactive (GUI / MCP) clause; the load-bearing
  proof is the Rust `station::display` + bridge tests + the headless dashboard smoke.

Original charge (kept for provenance): all stocks grouped per-domain + derived readouts computed
Rust-side and projected to Godot; UI renders a multi-domain dashboard with shared stocks highlighted
(roadmap 369). Zero parity concern.

### Step 3 (P8.3) — time controls

Play / pause / single-step / fast-forward decades. Variable stepping rate driven from the game
loop; the two-rate `step_day()` for sealed scenarios; fast-forward = the silent-observer loop
(free & parity-safe, item #1). UI: speed control + a horizon scrubber. **Threading (advisor #3):**
`step_day()` is 1440 substeps, so fast-forwarding decades is minutes of compute even in release —
it **must run off the render thread** or the UI freezes. That worker thread must carry the **same
FP env** as headless, so the FTZ/DAZ verification from Step 1 is **per-thread-the-sim-can-run-on**,
not one-time.

### Step 4 (P8.4) — flow inspection — **COMPLETE**

Expose per-flow **legs** (the flow-level slice of the display projection) so a player can see
where matter/energy moves. Item #2 (display-only, zero parity). **What landed** (all additive
under `rust/` + `godot/` + `tests/crossport/`; `git diff src/` empty; engine crates carry no
gdext types; 20 frozen goldens byte-identical — no science changed):

- **`station::inspection`** (new module, following the P8.2 split: the Rust-only *derived*
  read lives in `station`, not the frozen `simcore` port) — `inspect_flows(registry, state,
  resolver, dt) -> FlowInspection`. It **mirrors the integrator's private `evaluate_all`**:
  binds the resolver to the same `state`+`dt` and iterates `registry.flows()` in canonical
  order, so the inspected legs are **exactly the next Euler step's `k1`** (fidelity by
  construction, not approximation). `FlowInspection { n, flows: [InspectedFlow { id, legs }] }`
  + `flows_touching(stock)` (the "select a stock → contributing flows" primitive) + a plain-
  float `to_json` (reuses the crate-local display JSON helpers; the hex-float parity path stays
  on `simcore::snapshot`).
- **The truthfulness teeth (advisor #2), so the view can't lie:** the per-stock sum of the
  inspected legs, added to the current amount, reproduces the amount after `step()`. Proven
  both synthetically (a forced-inflow + donor-leak registry in `inspection.rs`) **and on the
  real single-rate `cabin_gas`** (`session.rs`): `before + Σ(inspected legs) == after` for
  every stock. Holds because the palette is `rationed == 0` **and** POOL-only (no extinction) —
  **both** well-fed assumptions named in the module doc as Step-5 seams (a rationed flow's raw
  legs, or an extinction loss-sink delta, would break the identity).
- **Single-rate only, two-rate deferred loudly (advisor #1).** `SimSession::inspect_flows()`
  returns `Some` for single-rate (`cabin_gas`, `station`) and **`None` for two-rate**
  (`greenhouse` / `sealed`). Inspecting a two-rate session's *fast* registry alone would show a
  greenhouse player everything *except* the plant (the biosphere's 17 carbon-moving flows live
  in the once-daily *slow* registry) — complete-looking but silently wrong. So it is scoped out
  exactly as P8.2 deferred the two-rate scalar readouts; a future step wanting it must surface
  both registries as *separately-labeled rate groups*, never summed.
- **Bridge + Godot.** `SimSession::flow_inspection_json()` (`""` for two-rate / pre-build).
  `godot/main.gd` renders the flow panel: each flow with its signed legs + a **rendered join
  for the highlighted shared stocks** (the `flows_touching` primitive; interactive
  click-selection is the GUI clause, per the P8.2/P8.3 precedent — the load-bearing proof is
  the Rust projection + headless smoke). Two headless cdylib smokes: `flow_smoke.gd`
  (`station` inspection crosses the boundary — real ids `power.solar_charge` / `power.load_draw`
  / `thermal.radiator_reject`, the `thermal.node` join shows the radiator withdrawing + Power's
  dissipation feeding it; `greenhouse` → `""`) and `main_ui_smoke.gd` (instantiates `main.tscn`
  so `main.gd`'s new panel is parsed/run — the Step-3 `ui_smoke.gd` precedent for UI otherwise
  loaded by nothing). `tests/crossport/test_godot_flow_inspection.py` gates both (local-only
  `skipif godot||cargo`).
- **Zero core + zero domain change** (`git diff src/{simcore/…}` — the Python tree — empty;
  all Rust changes under `rust/`); `cargo test` + `clippy --all-targets -D warnings` green;
  whole-project ruff/format/pyright green; full crossport suite green (48 passed); 20 frozen
  goldens byte-identical (no regen). Next: Step 5 (perturbations — the interactive cascades).

### Step 5 (P8.5) — perturbations (the interactive cascades)

Port the perturbation primitives to Rust (work item #4: `window_override` / `with_forcing` /
`LeakFlow` / `ScaledFlow`). Player triggers brownout / crew-load spike / atmospheric leak /
radiator failure from the UI; the cross-domain cascade emerges for free (the Phase-3/Step-8
discipline, now driven interactively). Inherits Phase-7 tier rules; conservation still asserted
every step.

### Step 6 (P8.6) — build systems (fixed palette)

Place & connect from a fixed, code-defined component palette; registry mutation stays Rust-side
and bounded (confirmed decision #1). The player composes a station from known parts; arbitrary
authoring is Phase 9.

### Step 7 (P8.7) — save / load

The Rust snapshot **loader** (work item #3). Save = `(scenario-id, State, n)`; rebuild the
registry from the palette-composed scenario id; resume is deterministic via `(seed, key, n)`. No
full structural serializer (fixed-palette scope).

### Step 8 (P8.8) — objectives + headless-parity harness + Phase-8 exit

- **The headless harness** (confirmed decision #2): a CLI / `cargo test` entry that drives the
  same `SimSession` to bit-identical results — the concrete "runs headless on a server"
  architecture proof. No netcode.
- **The gating cross-boundary parity check (advisor #2):** the Step-1 smoke promoted to a gate —
  ≥1 scenario (Tier-1 `cabin_gas`, ideally the sealed run too) driven through the *actual* `gdext`
  cdylib, final `State` compared to the headless trajectory, FP env asserted to match. This is the
  step that actually *verifies* the exit criterion's "exact same simulation" clause across the
  Godot boundary.
- **Objectives**: stability/failure goals a player pursues (survive N years, keep every quantity
  conserved under a perturbation schedule, avoid `rationed > 0`).
- **Exit criterion** (roadmap 373): a player can build · perturb · fast-forward decades · inspect
  flows · observe failure/stability across all five domains, while the exact same simulation runs
  headless. Doc/freeze as the phase warrants.

## Open items for just-in-time resolution

- Godot project root location (repo root vs a `godot/` subdir) — settle in Step 1 against where
  the `.gdextension` build output must land and where `addons/` already sits.
- `gdext` ↔ Godot version pinning — Step 1, empirically (not from memory).
- Whether Step 8 warrants a formal Phase-8 freeze contract like the biosphere/station references,
  or whether "no science changed" makes the Phase-7 freeze sufficient — decide at Step 8.
