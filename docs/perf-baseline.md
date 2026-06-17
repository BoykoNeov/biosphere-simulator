# Performance baseline (Phase 0.5, Step 6)

**NON-GATING.** Absolute numbers are machine-dependent — this is a tracked *regression reference*, not a pass/fail gate. Regenerate with `uv run python bench/perf.py --out docs/perf-baseline.md`.

- **Date (UTC):** 2026-06-17 10:39
- **Commit:** `d9cf43a`
- **Platform:** Windows-11-10.0.26200-SP0
- **CPU:** AMD64 Family 25 Model 97 Stepping 2, AuthenticAMD (16 logical cores)
- **Python:** CPython 3.13.0

## Method

Scenario: a **closed ring of `N` CARBON `POOL` stocks**, one first-order transfer flow per stock (`s_i -> s_{i+1 mod N}`, `k·dt = 0.05`). One outgoing flow per pool caps per-stock demand, so the ring stays well-fed (no Euler rationing, no RK4 over-draw hard-error) and closed (carbon conserved with no boundary reservoir) at every `N`. `N = 2` is the closed A⇌B exchange of `tests/test_stability.py`. Each measured cell is sanity-checked `rationed == 0` / no events.

**steps/sec** counts **master steps** (`n -> n+1`) for every scheme, so the three are directly comparable; each cell is calibrated to ~0.15s then measured best-of-3 (min time), with `gc` disabled across the timed interval. **Peak heap** is the high-water mark of the traced Python heap (`tracemalloc`, **not** RSS), measured in a separate run (50 steps) with the scenario built inside the traced region.

## Stock-count scaling — throughput (steps/sec)

Per-step cost is ~O(`N`) (one flow per stock); the three schemes also compare here. RK4 evaluates the flow set 4× per step (the per-step ratio to Euler grows toward 4× as fixed overhead is amortized at larger `N`); multi-rate is **slower still and is pure overhead in this scenario** — the fast/slow split is nominal (no stiffness separation), so it pays the splitting cost with none of the sub-stepping benefit it exists for (see `phase-0.5` N4).

| N (stocks) | Euler (steps/s) | RK4 (steps/s) | Multi-rate (steps/s) |
| --- | --- | --- | --- |
| 2 | 62,325 | 20,315 | 6,403 |
| 8 | 23,173 | 6,743 | 2,143 |
| 32 | 4,473 | 1,224 | 394 |
| 128 | 1,142 | 313 | 100 |
| 512 | 287 | 77 | 25 |

## Stock-count scaling — peak traced Python heap (KiB)

Memory scales ~O(`N`) (the stock dict, flow objects, and a few transient `State` snapshots); it does **not** grow with step count (old snapshots are GC'd, so the high-water mark is reached within a few steps).

| N (stocks) | Euler (KiB) | RK4 (KiB) | Multi-rate (KiB) |
| --- | --- | --- | --- |
| 2 | 23.7 | 23.9 | 25.6 |
| 8 | 29.6 | 32.2 | 34.3 |
| 32 | 58.6 | 69.2 | 72.0 |
| 128 | 176.9 | 219.7 | 226.4 |
| 512 | 650.0 | 820.5 | 842.5 |

## Domain-count scaling — throughput at fixed N = 128 (RK4)

Domain count is **orthogonal** to per-step cost: the hot loop never iterates domains — the conservation ledger partitions by `StockKind` and `Quantity` (not domain), and `Registry.domain_index` is O(stocks) regardless of how many domains those stocks span. So throughput here is flat (within measurement noise); domain count affects only one-time construction, not the step.

| D (domains) | RK4 (steps/s) |
| --- | --- |
| 1 | 310 |
| 2 | 308 |
| 4 | 310 |
| 8 | 306 |
| 32 | 306 |
| 128 | 308 |

