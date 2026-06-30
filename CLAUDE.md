# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from commit 1;
biosphere is the first domain. Python is the canonical reference ("laboratory");
a Rust core + Godot front-end come much later. End goal: a science-credible Godot
station sim that runs the *same* simulation headless.

**Source of truth for current work:** Phases 0, 0.5, 1, and 2 are **complete**
(`docs/plans/phase-{0-engine-skeleton,0.5-numerical-foundations,1-single-producer,2-closed-chamber}.md`).
**Phase 3 — the subsystem hierarchy / multi-compartment structure — is COMPLETE (exits)**
(`docs/plans/phase-3-modular-biosphere.md` — Steps 1–7 COMPLETE (hierarchy representation +
reusable compartment builders: `season.py` split into `scenario`/`stocks`/`atmosphere`/`soil`/`plants`/`water`;
water cycle closed, P3.3 — `soil_water`→`water_vapor`→`condensate`→`soil_water`, sealed now
closed for all four quantities; P3.4 closure-preserving mortality + annual reset — `annual_reset`
driver transform + `PERENNIAL_CHAMBER_SCENARIO` → sustained multi-year oscillation, death routes
to litter not the loss-sink; P3.1 ledger discharge / Step 5 — per-compartment boundary ledger
balances every step/quantity/compartment on the perennial run, extinction exception via the
`expected_extinction_residuals` helper, all diagnostics-only with no behavior change;
P3.5 perturbation harness / Step 6 — `perturbations.py` composes drought/lighting-failure/
atmospheric-leak onto the assembled inputs outside `build_season`, each a cascade-for-free
with conservation + `rationed == 0` + per-compartment ledger balanced under the perturbed
resolver; zero core change, three goldens byte-identical, no new golden;
Step 7 minimal consumer — a fifth leaf `biosphere.consumers` + `herbivory.py` (first-order
`Grazing`/`ConsumerRespiration`/`ConsumerMortality`, the decomposer pattern one trophic level
up), `CONSUMER_CHAMBER_SCENARIO`, fourth golden; consumer persists, genuinely closed,
`rationed == 0`, leaf↓/CO₂↑ cascade, per-compartment ledger balanced incl. CONSUMERS; zero
core change, three producer-only goldens byte-identical);
**Phase 3 exits. Phase 4 — decade-scale stability + freeze-as-reference** is underway
(`docs/plans/phase-4-closed-biosphere.md`): **Steps 1–4 COMPLETE.** Step 1 (P4.1) —
`domains/biosphere/drift.py` (pure-stdlib drift instrument: `total_quantity` promoting the
`_total` fold + three axes — mass-drift ceiling/detector, `is_stationary`/`non_collapsing`
stationarity split, `is_period_2`) + `test_drift.py` + `test_decade_stability.py`; both closed
scenarios probed Euler **and** RK4 to 15 yr → **Euler LOCKED, with evidence** (drift jitters at
√N round-off; cycle bounded/non-amplifying/non-collapsing — perennial settles to a **period-2**
cycle, consumer to a **period-1 fixed point** (herbivore damps the producer oscillation, measured
not assumed); closure held; RK4 cross-check retired the preconditions & structurally agrees → Step
2 escalation skipped). Step 3 (100k-step stress) — `test_biosphere_stress.py` (marked-slow,
streaming-chunked, bit-identical to a continuous run): both closed scenarios Euler-daily to **328
yr (100,040 steps)**, the real slow-drift detector → **EULER HOLDS, NO DRIFT** (mass-drift slope
flat at machine-ε over the 22×-longer run, both detector bounds span both horizons; period class
sustained the full horizon — perennial period-2, consumer period-1; closure held every step);
zero core change, four Phase-3 goldens untouched. Step 4 (P4.2, golden capture) —
`test_regression_long_horizon.py` pins the closed biosphere at the **decade-scale horizon**
(`LONG_HORIZON_YEARS = 15`, new shared `scenario.py` constant): 15-yr perennial+consumer
final-`State` hex goldens + the **drift-summary golden** (per-year cycle summaries + period
class — the stability signature; mass-drift round-off deliberately NOT pinned, it's noise).
Pre-golden closure gate + load-back + `__main__` regen mirror the existing discipline; the
four Phase-3 goldens re-affirmed byte-identical (Step 2 skipped → no regen); zero core change.
Next: Step 5 (freeze contract — `docs/biosphere-reference.md` + manifest + unfreeze
discipline). Roadmap `roadmap_extracted.txt`.
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
- Project's own license is **BNCL-1.0** (Boyko Non-Commercial License v1.0) —
  free to use/modify for non-commercial purposes; commercial use requires
  separate written permission from the copyright holder.

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
