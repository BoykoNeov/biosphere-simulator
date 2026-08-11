## Bucket 2: the rationing gate — make the `dt` hazard loud

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

COMPLETE — **nothing unfrozen, no golden moved**. `authoring.run_scenario` now **raises**
`RationedError` (Rust: `ErrorKind::Rationed`) when the Euler backstop fires;
`allow_rationing=True` / `run_scenario_allowing_rationing` is the escape hatch. This is the
item Tier 1 named and deferred, and it is a **consistency fix, not a new policy**: the
goldens, `StepReport`'s own docstring ("a failing gate, not a warning"), RK4's hard error,
and `station.objectives` (`survived = False`) had *all* already called rationing failure —
`run_scenario` was the lone surface that detected it and returned an integer. The apparent
counter-example (blackouts legitimately ration `load_draw`) **dissolved**: that is the
station path, which already scores it a *lost game* — same verdict, player idiom vs author
idiom. **The silence is fixed; the hazard is NOT** — the cabin still asphyxiates at
`dt=3600` (37 firings, both ports — a free cross-port confirmation, now pinned).
`docs/plans/post-roadmap-rationing-gate.md`
