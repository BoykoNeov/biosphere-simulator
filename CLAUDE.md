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
- **`docs/post-roadmap-log.md`** — the same, for everything after Phase 9, plus
  its own `docs/plans/post-roadmap-*.md` per entry. The table below indexes it.
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

**Post-roadmap** (the roadmap has no Phase 10 — work past it is chosen, not scheduled).
**The record is `docs/post-roadmap-log.md`** — one row per piece of work, with the
findings, the advisor calls, what was refused and why. Below is the index only: one
line each, newest concerns last. Add detail to the log, never to this table.

| Work | Status | Detail |
|---|---|---|
| Development posture — the Rust-primary pivot | **DECIDED 2026-07-20: Option A** — new content is Rust-first; Python stays frozen-canonical and green for validated science | `post-roadmap-rust-primary-pivot.md` |
| Scope (B): the day-neutral habitat crop | COMPLETE 2026-07-20 — diagnosed against an offline LINTUL3 oracle; authored, not validated; no golden moved | `post-roadmap-day-neutral-crop.md` |
| Scope (B): decomposer calibration | COMPLETE 2026-07-21 — the carbon rates moved above-range → top-of-range; biosphere re-frozen. Closure requires the fast edge | `post-roadmap-decomposer-calibration.md` |
| The nitrogen-cycle FORM gap | (A)+(B) **BUILT** 2026-07-27; (C) **REFUSED**; (D) **not buildable as recorded**. Two weakly-sourced params retired by changing the form, not by finding a citation | `post-roadmap-nitrogen-cycle-form.md` |
| The canopy regulator | DIAGNOSED 2026-07-27, **NOT BUILT** — fixes the canopy and is bit-identically inert on every frozen scenario | `post-roadmap-canopy-regulator.md` |
| The chamber-scale diagnosis | DIAGNOSED 2026-08-09 — the sealed jar holds ~2 days of one crop's carbon; enlarging it is refuted by BVAD. The defect is the gate, not the chamber | `post-roadmap-chamber-scale.md` |
| The acceptance gate | DIAGNOSED 2026-08-09 — the six tightest margins in the whole roster are one stock in one rig; `open_season`'s carbon source is unclamped and holds 0.0. **Its finding 6 ("the decision is the user's") was ADJUDICATED the same day — see the next row** | `post-roadmap-acceptance-gate.md` |
| The science assertions get contract standing | COMPLETE 2026-08-09 — `science_bands` + `liveness_floors` in both manifests; a schema unfreeze, no value moved | `post-roadmap-acceptance-gate-standing.md` |
| The humification split (a CUE) | **BUILT 2026-08-10** — CENTURY's partition; the settling transient grew ~3 yr → ~35, past the frozen horizon | `post-roadmap-cue-humification.md` |
| Soil carbon pool fractionation | **REFUSED TWICE** (2026-08-10) — `decomposition_rate` measured un-retirable by the only cited alternative on the shelf | `post-roadmap-soil-fractionation.md` |
| The crew-coupled loop | TAKEN and **REFUSED** 2026-08-10 — a chamber is carbon-limited by *isolation*, not volume; the two-rate split caps the crop at the standing pool | `post-roadmap-crew-coupled-loop.md` |
| The decade CO₂ guard, re-anchored | COMPLETE 2026-08-10 — the window was measured inert on the frozen tree and removed; a tightening, not a re-tune | `post-roadmap-co2-guard-reanchor.md` |
| Stem-reserve remobilization | DIAGNOSED + PRICED 2026-08-10, **NOT BUILT** — the stem cannot feed the seed; blocked on the uncited partition table, which is the real successor | `post-roadmap-stem-reserves.md` |
| Test-suite runtime (tooling, not science) | COMPLETE 2026-08-09 — `-n 12`, below-normal priority class; whole suite 7m05s | `docs/test-suite-runtime.md` |
| The first authored habitat | COMPLETE | `post-roadmap-authored-habitat.md` |
| The second authored habitat (`bioregenerative_station`) | COMPLETE 2026-08-11 — a frozen flow's "boundary" wiring field is a **name, not a constraint**, so the calibrated equipment recycles | `post-roadmap-bioregenerative-station.md` |
| Tier 1: grow the flow registry | COMPLETE — authoring platform unfrozen (`flow_types` 3→12, `param_loaders` 2→5) | `post-roadmap-flow-registry-growth.md` |
| Tier 2: the grammar — `monod` | COMPLETE — grammar unfrozen (`expr_nodes` 7→8); saturation is now sayable | `post-roadmap-grammar-monod.md` |
| Bucket 3 (A): diagnose + pin the oracle gap | COMPLETE — the gap is **structural**, so the deferred quantitative match is not a calibration task | `post-roadmap-validation.md` |
| Bucket 3 (B): the full oracle match | INCREMENT 1 + CEREMONY 2 COMPLETE 2026-07-20 — vernalization + photoperiod shipped. **The oracle is a diagnostic, never a fit target** | `post-roadmap-oracle-match.md` |
| Bucket 2: the export-fidelity hazard | COMPLETE — `rationed == 0` is not "the export is right"; closed by multi-rate Step 5's build-time `k·h < 1` precondition | `tests/test_authoring_export_fidelity.py` |
| Bucket 2: multi-rate authoring | COMPLETE — all 7 steps; the authoring platform re-frozen with the multi-rate surface in it, both ports | `post-roadmap-multirate-authoring.md` |
| Bucket 2: the rationing gate | COMPLETE — `run_scenario` now raises on rationing; conservation is not survival | `post-roadmap-rationing-gate.md` |
| Bucket 3 (C): cite the no-oracle params | CLOSED after 7 rounds — 8 cited, 14 design, 7 unciteable; blocked on retrieval, not effort, with the residual risk documented | `post-roadmap-citation.md` |
| The O₂ regulator's reversal | DIAGNOSED + CORRECTED 2026-08-11 — the clamp was already refused; the reversal is **not** author-only — it fires in 3 frozen runs the goldens are blind to. A sentence true when written, falsified by a seam three phases later | `post-roadmap-o2-makeup-reversal.md` |
| The direction gate (`ReversedFlowError`) | **BUILT 2026-08-11** — a third run-time verdict, `RationedError`'s sibling not its variant; authoring platform unfrozen (`flow_types` gains `demand_controlled`), both ports | `post-roadmap-o2-makeup-reversal.md` |
| Tooling: the PDF-backed citation pins (not science) | FIXED 2026-08-11 — a poppler upgrade re-wrapped quoted phrases; `sources/` is gitignored, so **CI green means nothing was checked** there | `docs/post-roadmap-log.md` (last row) |
| **Potato — the first SECOND species** | Stage 1 COMPLETE 2026-08-11 (stage 2, the Rust habitat mirror, deferred) — a crop is now a param SET; PCSE's bundled demo DB ships 6 offline oracles, so "new species = authored-only" was a stale reading. Two sources disagree qualitatively about when a tuber starts filling. **Its "one cause, two symptoms" canopy attribution was CORRECTED 2026-08-11 (measured at 39 %, not all)** | `post-roadmap-potato-crop.md` |
| The winter-wheat partition backfill | TAKEN and **REFUSED 2026-08-11** — the cited table misses the peak-LAI band 2.36×; cause isolated to root share over DVS 0–0.33. Neither table dominates the oracle: the frozen one passes because it was **fitted**. Successor is root functional coupling, not a citation hunt | `post-roadmap-wheat-partition-backfill.md` |
| Stem reserves: the model FORM found | LEAD 2026-08-11, **NOT STARTED** — [E] p. 93 §3.2.4 + Table 7 + Listings 3/4 carry the form the earlier refusal lacked; a partition table provably cannot substitute for it | `post-roadmap-wheat-partition-backfill.md` |
| Root functional coupling (the wheat refusal's successor) | TAKEN and **REFUSED 2026-08-11** — [E] p. 136 decouples rooted depth from root mass **on purpose**; a depth gate on N is bit-identical on all 8 scenarios. Uptake is demand-bound everywhere, so **no supply-side root coupling can bite**. The last uncited N param is citable but its band is scientifically inert. Successor is soil layers (for water), priced not attempted | `post-roadmap-root-functional-coupling.md` |

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
| Biosphere | the reference science: Euler/`dt=1`, 18 flows, aux, 12 params, 7 scenarios→goldens | `docs/biosphere-reference.md` + manifest |
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
- **Keep this file lean — it is loaded into every session, so bytes here are a tax on
  every task.** It is a map: phase status, invariants, layout, commands. Findings,
  measurements and rationale go in `docs/post-roadmap-log.md` and the plan docs. On
  finishing a piece of work, append a row to the **log** and *one line* to the index
  table above. If an entry needs a paragraph here, it belongs in the log.
  (This rule was added after the index grew to 206 KB of record — ~50k tokens per
  session — under a heading that already called itself "a map, not a record".)
- Repo etiquette: branch before committing; Conventional Commits.
  (Commits keep the harness-required `Co-Authored-By: Claude` trailer.)
