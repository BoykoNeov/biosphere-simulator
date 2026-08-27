# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from commit 1;
biosphere is the first domain. **Rust (`rust/crates/`) is the canonical reference**
since 2026-08-16 (the flip; see "Posture"); the Python checker is **retired** (S6), and a
Godot front-end consumes the Rust core. End goal (reached): a science-credible Godot
station sim that runs the *same* simulation headless.

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
- **`docs/phase-index.md`** — the Phase 0–9 table; every row reads COMPLETE. Work past
  Phase 9 is chosen, not scheduled.
- **`roadmap_extracted.txt`** — the original charge.
- **`docs/context-budget.md`** — why this file is small and what keeps it small.
- Reference material: `docs/bvad-reference.md` (NASA BVAD Table 3-31 — the
  primary source behind the crew params), `docs/param-file-conventions.md`,
  `docs/perf-baseline.md`, `docs/reuse-and-licenses.md`.
- Do not restate any of it here. See "Working style".

## The freeze contracts (four; each has an unfreeze discipline — follow it)

Changing anything a manifest names is an **unfreeze event**, not a refactor —
each doc spells out the ceremony. The three manifests have a paired gate
(`tests/test_freeze_manifest.py` biosphere, `test_station_freeze_manifest.py`,
`test_authoring_freeze_manifest.py`) that owns **completeness** (something added
but exercised by nothing); the goldens own **values**.

**A provenance-only edit is an unfreeze, and since C7 it FORCES a regeneration.** The
per-file sha-256 is still never compared as a *value*, so editing a param's `source:`
asserts nothing new — but the reference now writes each manifest from the files it compiles
in, and `tests/crossport/test_manifest_writer.py` compares the committed file byte for byte.
So a `source:`-only edit leaves the manifest **stale and red** until it is regenerated.
⚠ This paragraph said *"NOTHING CATCHES"* until 2026-08-18; C7 falsified it and nothing was
watching — measured, not assumed. What is still honor-system is the **ceremony**, not the
regeneration: advisor review → regenerate as the git-visible record → document.

| Contract | Freezes | Doc |
|---|---|---|
| Biosphere | the reference science: Euler/`dt=¼`, the flow/aux/param sets, 7 scenarios→goldens | `docs/biosphere-reference.md` + manifest |
| Station | the multi-domain assembly: sibling flows/params, the 4 seams, 13 scenarios→goldens (biosphere **delegated**) | `docs/station-reference.md` + manifest |
| Native port | the cross-port **tolerance** contract (not code): the 3 tiers + measured bands | `docs/native-port-reference.md` |
| Authoring | the author-facing **platform**: grammar, file schema, VM node/op set, flow-type registry | `docs/authoring-reference.md` + manifest |

`docs/phase-8-reference.md` is deliberately a doc with **no** manifest (Phase 8
added a consumer and changed no science).

**All three manifests are now anchored to the Rust tree, with MIXED authority stated
per key** in their own `_authority` block (`rust` / `python` / `hand`). A `python` key
is *not yet ported* — a queue, not a classification. Read the key's note before
assuming which side authored it.

## Layout

- `rust/crates/{simcore,domains,station,authoring,config,godot_bridge}/` — **the
  reference.** (`repo_gates` is a seventh, dev-only: its subject is this repo, not the sim.)
- `src/` — **DELETED by S6 (2026-08-27), all 112 modules.** One file survives,
  `config/paths.py`, read for one constant by a crossport tool that dies with it.
- `godot/` — the front-end (a subdir, so Godot's importer never scans the tree).
- `scenarios/` — authored **content** (runtime artifacts, never reference). Distinct
  from `rust/data/scenarios/`, which are fixtures / cross-port anchors.
- `rust/data/golden/` — 21 golden files (19 the reference's own bytes); the **20** in
  `rust/data/tiers.json` (7 biosphere + 13 station) carry the cross-port
  tier contract.

## Posture — Rust is the reference; Python is RETIRED (the flip, 2026-08-16; S6, 2026-08-27)

Standing rules, not a finished-work record — this is the "before you know what you are
working on" category. The record and its cost: `docs/log/reference-flip.md`.

- **New science is authored in Rust.** Do not build it twice. A Python-side item with
  no Rust mirror is now a gap in the *reference*, not in a copy.
- **Python has NO reference authority** (this inverts Phase 7/8/9's rule). A Python run
  that disagrees with Rust is a finding to investigate, never grounds for a silent
  Python-side fix. The two ports are no longer independent — that was the priced cost.
- **Sixteen Python files remain, and `git ls-files '*.py'` is the whole list.** The PCSE
  oracle carve-out (`tests/oracle/`, hand-run `-m oracle`, a diagnostic and never a gate);
  three crossport gates with no Rust successor **yet** — the manifest byte gate, the golden
  regeneration tool and its provenance tests — plus their two helpers and `config/paths.py`.
  Those three are S6's remaining build items; when they land, the count is the oracle alone.
- **`gdext` appears in `rust/crates/godot_bridge` and nowhere else.** Engine
  crates carry no Godot types.
- **"Authored ≠ validated."** Authored artifacts are runtime-only and never
  frozen; they get conservation + determinism, not scientific endorsement.
- ⚠ The flip changed which language spells the reference, **not** the unfreeze
  ceremony. And do not take a science item and a re-anchoring slice in one batch.

## Non-negotiable invariants (the things that are easy to get wrong)

- **Core is pure.** `simcore` carries **zero third-party deps** (no numpy/pint/yaml/
  json/plotting/UI/net; the Rust crates hand-roll their readers). Boundary code lives in
  `rust/crates/config`, which sits *below* `domains` and may not reach up into the engine.
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
- **RNG** is a counter-based, keyed, dependency-free generator in `simcore`, keyed by
  `(seed, key, n)` so draws are order-independent. No sequential-state RNG.
- **Units** validated at the file boundary — **`rust/crates/config`** is now the only one
  (an exact unit-string guard; every live pint conversion had been measured to be an
  identity). The core stores plain floats + a unit label.
- **Parameters are data** (YAML, loaded by Rust). No hardcoded coefficients.

## Reuse & licensing (see docs/reuse-and-licenses.md)

- Reimplement science from **primary literature**; cite the paper, not PCSE.
- **PCSE is EUPL (copyleft): offline validation oracle only, never ported or
  imported.** The WOFOST param YAML repo has no license — don't copy it.
- Project's own license is **BNCL-1.0** (Boyko Non-Commercial License v1.0) —
  free to use/modify for non-commercial purposes; commercial use requires
  separate written permission from the copyright holder.

## Commands

`cargo test` + `cargo clippy --all-targets -D warnings` in `rust/` — the reference's
own gates, so they run first. ⚠ A **mutation battery must pass `--no-fail-fast`**: cargo
stops at the first failing test *binary*, so a truncated run reports **fewer** reds — which
reads as "the new tests are inert", not as a broken instrument.

What is left of Python is a **46-test harness**, ~2 min, no parallelism worth
configuring: three crossport gates awaiting Rust successors, and the opt-in oracle.

```
uv sync                 # install/lock deps (no runtime deps since S6)
uv run pytest           # 42 tests + 4 oracle skips
uv run pytest -m oracle # the PCSE carve-out (opt-in; needs the `oracle` group + network)
uv run ruff check .     # lint
uv run pyright          # types
```

⚠ **`docs/test-suite-runtime.md` describes a suite that no longer exists** — the xdist
rules, the `loadgroup` trap, the 7m05s measurement and the below-normal priority class all
had ~2,400 tests as their subject. Kept as the record of why those rules existed; do not
read it as current advice.

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
