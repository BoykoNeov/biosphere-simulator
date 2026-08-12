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
- **`docs/post-roadmap-log.md`** — the same, for everything after Phase 9. It carries
  **its own index**, and this file does not duplicate it; the record itself is one file
  per work item in `docs/log/`, each naming its `docs/plans/post-roadmap-*.md`.
- **`docs/*-reference.md` (+ `.manifest.json`)** — the freeze contracts.
- **`roadmap_extracted.txt`** — the original charge.
- **`docs/context-budget.md`** — why this file is small and what keeps it small.
- Reference material: `docs/bvad-reference.md` (NASA BVAD Table 3-31 — the
  primary source behind the crew params), `docs/param-file-conventions.md`,
  `docs/perf-baseline.md`, `docs/reuse-and-licenses.md`.
- Do not restate any of it here. See "Working style".

## Status — all COMPLETE; the detail is not loaded here

Roadmap Phases 0–9 are all **COMPLETE** (`docs/phase-index.md`; per-phase detail in
`docs/plans/phase-<n>-*.md`). The roadmap has no Phase 10, so work past it is chosen,
not scheduled — its index is in `docs/post-roadmap-log.md` (newest concerns last) and its
record is one file per work item in `docs/log/`.

**Do not re-index either one here.** A finished piece of work earns a line in the log's
index, a pointer row, a file in `docs/log/`, and a memory file — not a row in this file.
See "Working style".

## The freeze contracts (four; each has an unfreeze discipline — follow it)

Changing anything a manifest names is an **unfreeze event**, not a refactor —
each doc spells out the ceremony. The three manifests have a paired gate
(`tests/test_freeze_manifest.py` biosphere, `test_station_freeze_manifest.py`,
`test_authoring_freeze_manifest.py`) that owns **completeness** (something added
but exercised by nothing); the goldens own **values**.

**A provenance-only edit is an unfreeze that NOTHING CATCHES.** The per-file sha-256 is
recorded but never compared, so editing just a param's `source:` moves the hash and turns
nothing red. It is still an unfreeze — the ceremony is simply honor-system, so follow it
deliberately: advisor review → regenerate the manifest as the git-visible record → document.

| Contract | Freezes | Doc |
|---|---|---|
| Biosphere | the reference science: Euler/`dt=1`, the flow/aux/param sets, 7 scenarios→goldens | `docs/biosphere-reference.md` + manifest |
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
uv run pytest -n 12     # ...in parallel (xdist); grouping is handled in conftest
uv run ruff check .     # lint
uv run ruff format .    # format
uv run pyright          # types
```

`cargo test` + `cargo clippy --all-targets -D warnings` in `rust/`.
Markers: `-m slow` (opt-out; ~9 min serial, **7m05s at `-n 12`**, re-measured 2026-08-10;
was 4m33s at ~160 fewer tests), `-m oracle` (opt-in).

**Suite runtime** — the measurements and their controls are in
`docs/test-suite-runtime.md`. The two rules that bite: do **not** reach for
`--dist loadgroup` (a group assigned from a collection hook is silently dropped, and the
mode doubled the full run), and hypothesis's 200 ms per-example deadline stays disabled
(a wall-clock assertion in a suite that has none; `--hypothesis-profile=strict-deadline`
restores it). Pytest runs at below-normal priority *class* so the xdist workers and
`cargo` children inherit it; `SIMTEST_PRIORITY=normal` opts out.

## Testing

- Prefer **test-first** for engine invariants. Use **property-based** tests
  (hypothesis) for universal laws: conservation, non-negativity, order-independence.
- Golden/regression snapshots use **hex-float** for exact comparison.
- Never weaken or delete a test to make it pass; fix the code or flag the gap.

## Working style

- Plan before non-trivial work; keep `docs/plans/*` updated as living docs.
- **This file carries only what you need BEFORE you know what you are working on** —
  invariants, layout, commands, contract names, pointers. It is loaded unconditionally,
  so a byte here is a tax on every task, including the ones it cannot help.
- **On finishing a piece of work: a line in the log's index, a pointer row, a file in
  `docs/log/`, a memory file. Nothing here.** A finished item leaves the always-loaded map
  the moment its lesson is written down elsewhere. Rationale + the paired ceiling test:
  `docs/context-budget.md`, `tests/test_context_budget.py`.
- Repo etiquette: branch before committing; Conventional Commits.
  (Commits keep the harness-required `Co-Authored-By: Claude` trailer.)
