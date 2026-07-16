# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from commit 1;
biosphere is the first domain. Python is the canonical reference ("laboratory");
a native Rust core and a Godot front-end consume it. End goal (reached): a
science-credible Godot station sim that runs the *same* simulation headless.

**The roadmap (`roadmap_extracted.txt`) is COMPLETE through Phase 9 — its last
phase.** The simulator "is not really about plants; it is about closure of matter
— and energy — cycles" (roadmap).

## Where the detail lives (this file is a map, not a record)

- **`docs/plans/phase-*.md`** — the living record. Each carries its phase's
  design *and* per-step `COMPLETE`/`RESOLVED` outcome blocks (findings, the
  advisor calls, what was deferred and why). **This is the source of truth for
  what happened.**
- **`docs/*-reference.md` (+ `.manifest.json`)** — the freeze contracts.
- **`roadmap_extracted.txt`** — the original charge.
- Reference material: `docs/bvad-reference.md` (NASA BVAD Table 3-31 — the
  primary source behind the crew params), `docs/param-file-conventions.md`,
  `docs/perf-baseline.md`, `docs/reuse-and-licenses.md`.
- Do not restate any of it here. See "Working style".

## Phase status

| Phase | Topic | Status |
|---|---|---|
| 0 | Engine skeleton | COMPLETE |
| 0.5 | Numerical foundations | COMPLETE |
| 1 | Single producer | COMPLETE — quantitative oracle match DEFERRED (user decision) |
| 2 | Closed chamber (producer + decomposer) | COMPLETE |
| 3 | Modular biosphere / consumers | COMPLETE |
| 4 | Closed biosphere | COMPLETE — **biosphere FROZEN as the reference** |
| 5 | Sibling domains (power/thermal/eclss/crew) | COMPLETE |
| 6 | Station integration (cross-domain coupling) | COMPLETE — **station FROZEN as the multi-domain reference** |
| 7 | Native core (the Rust port) | COMPLETE |
| 8 | Godot front-end | COMPLETE |
| 9 | Scenario authoring & modding | COMPLETE — **the author-facing platform FROZEN** |

Each row's detail is in `docs/plans/phase-<n>-*.md`.

**Post-roadmap** (the roadmap has no Phase 10 — work past it is chosen, not scheduled):

| Work | Status |
|---|---|
| The first authored habitat (`scenarios/algae_habitat.yaml`) | COMPLETE — `docs/plans/post-roadmap-authored-habitat.md` |

## The freeze contracts (four; each has an unfreeze discipline — follow it)

Changing anything a manifest names is an **unfreeze event**, not a refactor —
each doc spells out the ceremony. The three manifests have a paired gate
(`tests/test_freeze_manifest.py` biosphere, `test_station_freeze_manifest.py`,
`test_authoring_freeze_manifest.py`) that owns **completeness** (something added
but exercised by nothing); the goldens own **values**.

| Contract | Freezes | Doc |
|---|---|---|
| Biosphere | the reference science: Euler/`dt=1`, 17 flows, aux, 13 params, 7 scenarios→goldens | `docs/biosphere-reference.md` + manifest |
| Station | the multi-domain assembly: sibling flows/params, the 4 seams, 13 scenarios→goldens (biosphere **delegated**) | `docs/station-reference.md` + manifest |
| Native port | the cross-port **tolerance** contract (not code): the 3 tiers + measured bands | `docs/native-port-reference.md` |
| Authoring | the author-facing **platform**: grammar, file schema, VM node/op set, flow-type registry | `docs/authoring-reference.md` + manifest |

`docs/phase-8-reference.md` is deliberately a doc with **no** manifest (Phase 8
added a consumer and changed no science).

## Layout

- `src/simcore/` — the pure engine. `src/domains/{biosphere,power,thermal,eclss,crew}/`
  — no domain imports another. `src/station/` — the assembly layer that owns all
  cross-domain wiring. `src/authoring/` — the declarative scenario platform
  (boundary code). `src/{sim_io,config,lab}/` — boundary.
- `rust/crates/{simcore,domains,station,authoring,godot_bridge}/` — the native port.
- `godot/` — the front-end (a subdir, so Godot's importer never scans the tree).
- `scenarios/` — authored **content** (runtime artifacts, never reference). Distinct
  from `tests/authoring/scenarios/`, which are fixtures / cross-port anchors.
- `tests/regression/golden/` — 25 golden files; the **20** in
  `tests/crossport/tiers.json` (7 biosphere + 13 station) carry the cross-port
  tier contract.

## Purity invariants (the exit criterion of every port/consumer phase)

- **`git diff src/` must come back empty** for Phase 7/8/9 work. The Rust port,
  the Godot front-end, and the authoring platform are *consumers* — they never
  edit the Python reference to suit themselves.
- **`gdext` appears in `rust/crates/godot_bridge` and nowhere else.** Engine
  crates carry no Godot types.
- **The port has NO reference authority.** A Rust/Godot run that surfaces a
  Python bug is an unfreeze-discipline finding, never a silent native-side fix.
- **"Authored ≠ validated."** Authored artifacts are runtime-only and never
  frozen; they get conservation + determinism, not scientific endorsement.

## Non-negotiable invariants (the things that are easy to get wrong)

- **Core is pure.** `simcore/` imports **stdlib only — zero third-party deps**
  (no numpy/pint/yaml/json/plotting/UI/net). Boundary stuff lives in `sim_io/`
  and `config/`. This keeps the Rust port mechanical.
- **Flows return structured per-stock legs, never a net delta.** A flow is an
  atomic stoichiometric transfer; arbitration scales the *whole flow*.
- **Every flow is internally balanced.** The "outside" is explicit BOUNDARY
  reservoir stocks; `Inputs = Outputs + ΔStored` where Inputs/Outputs are
  boundary deltas. Conservation is asserted every step — a failure is a bug.
- **Determinism:** bit-identical within a build. Time is an **integer step count**
  (`t = n*dt`, never `t += dt`). **Canonical (flow-id) order on every reduction**
  (demand sum, scaling, delta sum). Cross-port (Rust) is tolerance-gated.
- **Arbitration backstop is Euler-only and rare.** It runs always, counts
  firings; golden runs assert the count == 0. Under RK4+, a needed scale is a
  **hard error** (positivity comes from kinetics).
- **Extinction conserves mass:** POPULATION stock below threshold → 0 with the
  residual routed to the loss-sink. POOL stocks are never zeroed-with-loss.
- **RNG** is a counter-based, keyed, pure-Python generator in `simcore`, keyed by
  `(seed, key, n)` so draws are order-independent. No sequential-state RNG.
- **Units** validated at the boundary (pint, in `config/`); the core stores plain
  floats + a canonical-unit label.
- **Parameters are data** (YAML + pydantic schema). No hardcoded coefficients.

## Reuse & licensing (see docs/reuse-and-licenses.md)

- Reimplement science from **primary literature**; cite the paper, not PCSE.
- **PCSE is EUPL (copyleft): offline validation oracle only, never ported or
  imported.** The WOFOST param YAML repo has no license — don't copy it.
- Project's own license is **BNCL-1.0** (Boyko Non-Commercial License v1.0) —
  free to use/modify for non-commercial purposes; commercial use requires
  separate written permission from the copyright holder.

## Commands

```
uv sync                 # install/lock deps
uv run pytest           # tests (pytest + hypothesis)
uv run ruff check .     # lint
uv run ruff format .    # format
uv run pyright          # types
```

`cargo test` + `cargo clippy --all-targets -D warnings` in `rust/`.
Markers: `-m slow` (opt-out, ~9 min), `-m oracle` (opt-in).

## Testing

- Prefer **test-first** for engine invariants. Use **property-based** tests
  (hypothesis) for universal laws: conservation, non-negativity, order-independence.
- Golden/regression snapshots use **hex-float** for exact comparison.
- Never weaken or delete a test to make it pass; fix the code or flag the gap.

## Working style

- Plan before non-trivial work; keep `docs/plans/*` updated as living docs.
- Keep this file lean. Put detail in `docs/`, not here.
- Repo etiquette: branch before committing; Conventional Commits.
  (Commits keep the harness-required `Co-Authored-By: Claude` trailer.)
