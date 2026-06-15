# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from commit 1;
biosphere is the first domain. Python is the canonical reference ("laboratory");
a Rust core + Godot front-end come much later. End goal: a science-credible Godot
station sim that runs the *same* simulation headless.

**Source of truth for current work:** `docs/plans/phase-0-engine-skeleton.md`.
Reuse/licensing rules: `docs/reuse-and-licenses.md`.

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
- Project's own license is **undecided** — assume clean-room until decided.

## Commands (once the skeleton exists)

```
uv sync                 # install/lock deps
uv run pytest           # tests (pytest + hypothesis)
uv run ruff check .     # lint
uv run ruff format .    # format
uv run pyright          # types
```

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
