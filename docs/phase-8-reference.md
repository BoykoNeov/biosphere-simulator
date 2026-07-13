# The Phase-8 reference — the Godot front-end (P8.8, the exit/architecture contract)

Phase 8 puts a **Godot front-end** on the **frozen** native Rust core (Phase 7). The roadmap
charge: *"Everything becomes visible. Nothing scientific changes. The game never computes
domain logic — it only displays and manipulates the simulation."* This file is the phase's
**exit / architecture contract**: what was built, the invariants that keep the core pure
across the language boundary, how the "the exact same simulation runs headless" claim is
*verified* (not merely asserted), and why Phase 8 needs **no freeze manifest of its own**.
The plan of record is [`docs/plans/phase-8-godot-frontend.md`](plans/phase-8-godot-frontend.md).

Like the freeze/tolerance contracts ([`docs/station-reference.md`](station-reference.md),
[`docs/native-port-reference.md`](native-port-reference.md)), this is **boundary-side docs
only**. The one machine-checked companion is the CI/parity harness under
`tests/crossport/`; nothing here overrides the frozen references.

## What Phase 8 delivered (Steps 0–8)

A Godot application that lets a player **build systems, perturb systems, fast-forward decades,
inspect flows, set objectives, save/load, and observe failure or stability** across power ·
thermal · atmosphere · biosphere · crew — while the **exact same** simulation runs headless.
Godot owns visualization, interaction, time controls, objectives, and save/load; it computes
**no** domain logic. Every stock, flow, temperature, and conservation check comes from the
frozen Rust core.

- **The steppable session** (`station::session::SimSession`, Step 0) — the caller-driven,
  owned-state inversion of the Phase-7 runners; `step()` advances one natural unit
  (a `step_report` single-rate; a master day two-rate).
- **The GDExtension binding** (`rust/crates/godot_bridge`, Steps 1–7) — the cdylib that wraps
  the session for Godot: build / step / fast-forward off the render thread / display
  projection / flow inspection / perturbations / fixed-palette composition / save-load /
  objectives.
- **The headless CLI** (`station`'s `sim` bin, Step 8) — the no-Godot entry point that drives
  the **same** session from a shell (confirmed decision #2).

## The purity invariant — Phase-8's "`git diff src/` empty"

The Python reference `src/` tree is **untouched** for the whole phase (`git diff src/` empty),
and the engine crates stay pure across the FFI boundary:

- **`gdext` appears only in `rust/crates/godot_bridge`.** The engine crates — `simcore`,
  `domains`, `station` — carry **no gdext types in their signatures**. Anything the binding
  needs is an **additive `pub` exposure** in an engine crate (the Phase-7 `crew::carbon_split`
  discipline), never an inward dependency. This keeps the WASM-future and the C#-someday
  options open and keeps "core is pure" true across the language boundary.
- **Nothing scientific changed.** All **20 frozen goldens are byte-identical** across every
  Phase-8 step (verified by re-emitting them; no regen). Phase 8 *displays and manipulates* a
  frozen engine — a surfaced discrepancy would be an unfreeze-discipline finding, never a
  UI-side "fix".

## The one shared builder — "the exact same simulation" by construction (Step 8)

The exit criterion is that *"the **exact same** simulation runs headless."* That is true **by
construction**, not by an agreeing re-implementation, because the Godot cdylib and the
headless CLI build every session through **one** gdext-free dispatch:

> **`station::palette::build_scenario(id) -> (SimSession, DisplayContext)`**

The bridge's `build` wraps it (adding only Godot plumbing); the `sim` bin calls it directly.
Neither re-implements the wiring, so they cannot drift — the same extraction discipline as
Step 0's `driver::advance_one_master_day` / `sealed::sealed_reset_hook`. The bit-identity is
gated: `sim <scenario> <steps>` is **byte-for-byte** equal to the corresponding `emit_*`
example (`tests/crossport/test_headless_cli.py`), for single-rate (`cabin_gas`, `station`) and
two-rate (`greenhouse`) alike.

## The Godot-boundary parity contract (the exit criterion's teeth)

Incremental stepping is bit-identical to run-to-completion **in pure Rust** (`N×step() ==
run_station(N)`, Step 0's `tests/session_parity.rs`). But that intra-process test is **blind
to Godot-hosted-vs-headless divergence** — the real risk is **per-thread FP control flags**
(FTZ/DAZ in MXCSR) a game engine may set for SIMD throughput, flushing denormals to zero and
diverging from the IEEE-default headless run. So a *genuine cross-boundary* check drives the
scenario through the **actual `gdext` cdylib Godot loads**, in `--headless` mode, and asserts:

1. **byte-identical snapshot** — the Rust-side hex-float snapshot (`snapshot_json`, the golden
   codec — no GDScript float printing in the parity path) equals the headless `emit_*` output
   **byte-for-byte** (the "the FFI didn't corrupt determinism" proof), and, where a frozen
   golden exists at that horizon, matches it at its [tier](native-port-reference.md);
2. **FP env clean** — `fp_clean()` read *on the stepping thread* reports **FTZ and DAZ OFF**
   (the direct check a bit-exact snapshot alone cannot make: a scenario may never produce a
   denormal);
3. **Tier-0 discretes** — `rationed == 0`, the expected step count.

**Coverage (`tests/crossport/test_godot_*.py`):**

| smoke | scenario | rate | new coverage |
|---|---|---|---|
| `test_godot_parity` | `cabin_gas` | single | the FFI boundary itself (Step 1) |
| `test_godot_compose` | `{power_plant, radiator}` | single | palette composition == frozen `station` |
| `test_godot_save_load` | `cabin_gas` | single | a real `FileAccess` disk round-trip |
| `test_godot_perturbations` | `station` brownout | single | a perturbed (rationing) run |
| `test_godot_two_rate_parity` (greenhouse) | `greenhouse` | **two** | the two-rate driver across the boundary |
| `test_godot_two_rate_parity` (sealed, **slow**) | `sealed` | **two** | 310 master days — the re-sow adopt branch crosses the boundary |
| `test_godot_objectives` | `station` | single | stability **and** failure are both reachable |

The full multi-year sealed **science** parity is gated in CI by the frozen
`sealed_station_state.json` golden (the `crossport` job); the full-horizon *resume*-parity
(`915×step() == run_sealed`) is an `#[ignore]`d run-manually test in `session_parity.rs`, not
a CI gate. The cross-boundary sealed smoke runs a few days past one 305-day season
(`SEALED_RESUME_DAYS = 310`) so the re-sow branch fires across the FFI without paying the
whole decade through headless Godot.

### Where the gate runs

- The **`crossport` CI job** gates the 20 goldens (Rust-vs-Python) and the **headless CLI
  bit-identity** (cargo only, no Godot) — bit-exact on Linux/glibc.
- The **`godot-parity` CI job** installs headless Godot and runs the cross-boundary Godot
  smokes (the fast set; `-m "not slow"`), so **Step-1's smoke is promoted from a
  silently-skipped local test to a real gate** — the genuine glibc-Rust-cdylib-vs-UCRT-golden
  cross-boundary check (de-risked in a Linux container before landing, the Phase-7-Step-6
  precedent). Both cross-boundary comparisons are either same-platform-emit (bit-exact
  regardless of libm) or Tier-1 (`cabin_gas`, transcendental-free), so the Tier-2 cross-libm
  bands are not exercised here.
- The **slow sealed cross-boundary smoke** (~4 min through the debug cdylib) is a
  **mandatory-local / release-time gate** (`-m slow`), kept out of the shared CI runner; its
  full-horizon relation is already gated intra-process + by the frozen golden.

## Objectives — pure predicates, zero domain logic (Step 8)

`station::objectives` turns the session diagnostics into a player's win/fail condition: an
`Objective::survive(target_step)` and an `ObjectiveReport` whose clauses (`reached_target`,
`conserved`, `no_rationing`, `no_extinction`, `survived`) are a boolean fold over `n`,
`total_rationed`, `events`, and `max_residual`. It is **not** a goal-tracking DSL or a
scheduler — the interesting dynamics come from the perturbations: a deep brownout drives
`rationed > 0`, flipping `survived` to `false`, so the *same* objective distinguishes a stable
run from a failing one. Zero parity concern (display-only, the display/inspection-split
precedent).

## The freeze decision — a doc, not a manifest

The station / biosphere / native-port manifests freeze **scientific** surface and **port
tolerance**. Phase 8 added a **consumer** (Godot binding), a **CLI**, and **objectives**,
changed **no science**, and kept all 20 goldens byte-identical throughout — there is **no new
frozen surface to gate**. So Phase 8 gets **this reference doc, not a freeze manifest with a
completeness gate**. The **Phase-7 freeze remains sufficient** for the science; the Godot
boundary is governed by the parity contract above, not by a hash of UI code that changes as
the front-end grows.

## Exit criterion — met

A player can **build** (fixed palette), **perturb** (windowed cascades), **fast-forward
decades** (off the render thread), **inspect flows**, **set objectives**, **save/load**, and
**observe failure or stability** across all five domains — while the **exact same** simulation
runs headless (the shared palette builder, the CLI, and the cross-boundary parity gate prove
it). **Phase 8 is complete.**
