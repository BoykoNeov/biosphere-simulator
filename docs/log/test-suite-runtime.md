## **Test-suite runtime** (tooling, not science)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE (2026-08-09)** — `docs/test-suite-runtime.md`; pins in
`tests/test_suite_runtime.py`. `pytest -n 12` via xdist: fast loop **167.6s -> 51.1s
(3.3x)**, and the **whole** suite (slow tier included) **4m33s** ⚠ **re-measured 2026-08-10
at 7m05s** (2162 tests vs 2003; no growth *ratio* quoted — the comparison would conflate
suite size with machine state, and the fast loop reproducing 51.1s→49.58s is what licenses
the reading at all). **Two findings from that re-measure.** (i) **The Tier-2 recomputation
count tracks the SELECTION, not the distribution mode** — the doc said "two under `load`,
four under `loadgroup`" as though the mode set it; under the shipped `load` the *full suite*
pays **two** (379.6/374.3s) and **`-m slow` alone pays four** ⇒ **running only the slow tier
is SLOWER than running everything (504.47s vs 425.49s)**; the cause is recorded as
**unmeasured** (the scheduler claim needs the doc's own PID-count probe). (ii) ⚠ **A 2197s
reading was taken the same day, reported as fact, and used to conclude the doc was "~8×
stale" — DISCARDED, and the error was the inference, not the reading**: one measurement in
an unverified machine state, the exact shape this doc's own warning exists for, committed
one screen after quoting that warning. Cause never isolated ⇒ recorded as unexplained,
**not** as self-inflicted contention. **A run's verdict and its wall clock fail
independently** — that run's 2161 passed was never in doubt. The pytest process runs at
**below-normal priority CLASS** (`SIMTEST_PRIORITY=normal|idle`) — a class, not
`PROCESS_MODE_BACKGROUND_BEGIN`, because a class is **inherited**, so one call also covers
the xdist workers and the `cargo`/`rustc` children `tests/crossport/` spawns; pinned,
because the first implementation failed with `ERROR_INVALID_HANDLE` (ctypes marshalled the
pointer-sized pseudo-handle as 32-bit) **with every test still green**. **THE FIND:
hypothesis's default 200 ms per-example deadline is a WALL-CLOCK ASSERTION inside a suite
that has none** — `test_biosphere_demo::…order_independent[Rk4Integrator]` hit 441.58 ms
under 12 workers and failed. Deselected via a settings profile; not a weakened gate (no
property here is about *speed*; `bench/perf.py` owns performance non-gatingly). ⚠ **I then
wrote the meta-finding's next instance myself**: the draft called it *"a latent flake
serially too, within ~2x of the default"* — an **inference** from 441 ms, a *loaded-machine*
number, dressed as a property of the test. **Measured: 0.10 s for the whole call serially,
~17 ms/example, over 10x clear.** The deadline fires on **contention alone**, which
*strengthens* the decision; the default stays reachable as
`--hypothesis-profile=strict-deadline` so the claim remains checkable rather than becoming
folklore. ⚠⚠ **A WHOLE SHIPPED DESIGN WAS THEN MEASURED FALSE AND REMOVED — and it had been
committed, pushed, and documented while green.** The first version forced `--dist loadgroup`
under `-n` and assigned `xdist_group` markers from `pytest_collection_modifyitems`, to pin
`tests/crossport/` (cargo's `target/` lock is exclusive: 41.6s pinned vs **51.3s at -n 8**,
so spreading them is *slower*) and the six `sealed_tier2_run` consumers (session scope is
**per worker** under xdist). **Both halves false.** (1) **A group assigned by a collection
hook is silently dropped** — xdist stamps the `@group` nodeid suffix its scheduler reads
(`xdist/remote.py`) *before* this conftest's hook runs. Measured on a synthetic 4-file suite
sharing one session fixture: hook-assigned ⇒ **4 processes**, the documented in-file
`pytestmark` ⇒ **1**. (2) **`loadgroup` is `LoadScopeScheduling`**, dispatching per *scope*;
with most tests ungrouped every test is its own scope, so it is far slower than `load` —
**10.0s vs 1.07s** on that probe, and it **DOUBLED the full suite (613.7s vs 273.4s,
back-to-back)**. ⚠ **I had explicitly dismissed the 202s-vs-94s signal as machine contention
and written a *structural* argument that the two modes were equivalent** — the argument was
read off `_split_scope` and was correct about that function while false about the scheduler
around it. **The tell I ignored was a measurement I already had** — and my first write-up of
the finding then *cited that same unmatched pair as evidence*, in a document whose own
discipline section discards unmatched comparisons; corrected to the 613.7/273.4 pair, with
202/94 relabelled as the dismissed signal it was. The probe that settles it: give a
session-scoped fixture a side effect keyed by `os.getpid()` and **count the processes** —
*"the suite passed"* and *"the grouping worked"* are unrelated statements, and every wrong
version here was green. Residual, recorded not fixed: `sealed_tier2_run` is still recomputed
**twice** at `-n 12`; the two ways out (in-file `pytestmark` + `loadgroup`'s cost, or
caching the Tier-2 trajectory to disk = a stale-artifact hazard in the run that gates a
frozen golden) are priced, neither taken. Verified the way this repo needs rather than by
two green bars: matched-pair serial vs `-n 12` JUnit XML compared by test id — **2003 ids,
identical sets, ZERO outcome differences**, `git status` unchanged by either. ⚠ Benchmarking
on this box is unreliable and the cause was found rather than assumed: identical commands
read 94/197/202/310s while **another project's** orphaned `pytest -n 4` workers (~620s CPU
each), a census script and Defender saturated it ⇒ **quote the ratio, and only from a
back-to-back pair**. Nothing unfrozen, no golden moved, `git diff src/` empty;
`pytest-xdist` is a **dev** dep and does not touch the core purity invariant (that is about
`src/simcore` imports, gated by `test_simcore_purity.py`).
