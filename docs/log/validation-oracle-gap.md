## Bucket 3 scope (A): validation — diagnose + pin the oracle gap

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

COMPLETE — **no golden moved, nothing unfrozen**. The gap is **structural, not merely
uncalibrated**: the canopy never bootstraps (1.75 % light interception at sowing vs the
oracle's 97.8 %; LAI peaks day 32 of ~305 and collapses *before* anthesis) and phenology
runs ~1.6x fast (no vernalization) — two *independent* missing sciences; param values are
only the **third** cause. So the deferred "quantitative oracle match" is **not a calibration
task**. Pinned by `tests/test_oracle_gap.py`; `docs/plans/post-roadmap-validation.md`
