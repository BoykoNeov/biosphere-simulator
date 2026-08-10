# Test-suite runtime — parallelism, priority, and what is *not* claimed

**NON-GATING.** Nothing here changes a scientific result; it changes how long the
suite takes and how much of the machine it takes while doing it. The sibling document
`docs/perf-baseline.md` is about *engine* throughput and is a different subject.

## The commands

```
uv run pytest                 # everything (slow tier included — `slow` is opt-OUT)
uv run pytest -m "not slow"   # the fast loop
uv run pytest -n 12           # parallel (xdist's default --dist load)
uv run pytest -n auto         # one worker per logical core (16 here)
SIMTEST_PRIORITY=normal uv run pytest    # run at normal priority
SIMTEST_PRIORITY=idle   uv run pytest    # yield harder than below-normal
```

`pytest-xdist` is a **dev** dependency. It does not touch the core purity invariant —
that invariant is about what `src/simcore` imports and is enforced by
`tests/test_simcore_purity.py`, which is unaffected by anything in `tests/`.

`-n` is deliberately **not** baked into `addopts`. Making every invocation parallel
would change the CI jobs and single-test debugging as a side effect of a local
ergonomics choice.

**CI is deliberately left serial** — `.github/workflows/ci.yml` still runs
`uv run pytest -m "not slow"` and `pytest tests/crossport/` without `-n`. Parallelising
the runners is a separate decision with its own trade-offs (hosted runners have few
cores, and the `crossport` job is cargo-lock-bound anyway), not an oversight. Note that
the priority drop **is** live in CI, since `pytest_configure` is unconditional; on a
dedicated runner with nothing competing that is a no-op, which is why it is not gated.

## Below-normal priority (on by default)

`tests/conftest.py::pytest_configure` drops the process priority. On Windows it sets
the priority **class** (`BELOW_NORMAL_PRIORITY_CLASS`); on POSIX it nices the process.

A priority *class* rather than `PROCESS_MODE_BACKGROUND_BEGIN` because a class is
**inherited by child processes**: one call in the pytest process therefore also covers
the xdist workers and the `cargo run` / `rustc` children that `tests/crossport/`
spawns, which are the heaviest things the suite starts. Background mode covers the
caller only, and throttles I/O hard enough to be its own source of weirdness.
`tests/test_suite_runtime.py` pins the inheritance, so switching to background mode
would go red rather than silently narrowing the effect.

`pytest_configure` runs in the master **and** in every xdist worker, so the setting
does not depend on inheritance to be correct — inheritance is what extends it to the
non-Python children.

> **This is a scheduling hint, not core reservation.** 12 below-normal workers still
> occupy 12 cores and all the memory bandwidth. Windows will preempt them for your
> foreground app, but cache and bandwidth pressure remain. Use a worker count below
> the core count *and* the priority drop, not one instead of the other.

The first implementation of this call failed with `ERROR_INVALID_HANDLE` — ctypes
marshalled the pointer-sized pseudo-handle as a 32-bit `int` — and the suite ran at
normal priority with every test green. That is why the mechanism has pins rather than
a comment.

## Worker distribution: `load`, and a negative result worth keeping

`-n` uses xdist's own default, `--dist load` (per-test balance). Nothing in this repo
overrides it, and the reason is a **design that was built, measured, and removed**.

The removed design forced `--dist loadgroup` under `-n` and assigned `xdist_group`
markers from `pytest_collection_modifyitems`, to pin two things to single workers:
`tests/crossport/` (cargo's `target/` lock is exclusive — 41.6 s pinned against 51.3 s
at `-n 8`, so spreading them out is *slower*, not merely wasteful) and the six
consumers of the session-scoped `sealed_tier2_run` fixture (session scope is **per
worker** under xdist, so the ~5-minute Tier-2 run is otherwise recomputed).

Both halves are false, and neither is visible from a green suite:

1. **A group assigned by a collection hook is silently dropped.** xdist stamps the
   `@group` suffix onto the nodeid — the thing its scheduler actually reads
   (`xdist/remote.py`) — *before* this suite's `pytest_collection_modifyitems` runs.
   Measured on a synthetic 4-file suite sharing one session fixture: with the marker
   assigned by a hook the fixture was computed in **4 processes**; with the documented
   in-file `pytestmark = pytest.mark.xdist_group("g")` it collapsed to **1**.
2. **`loadgroup` is the wrong mode for a mostly-ungrouped suite.** It is
   `LoadScopeScheduling`, dispatching one *scope* at a time; with most tests ungrouped
   every test becomes its own scope. Same 8-test probe: **10.0 s** against `load`'s
   **1.07 s**. On this suite, back-to-back in one session: the full run took
   **613.7 s under `loadgroup` against 273.4 s under `load`**.

⚠ An earlier draft of this section argued (2) from "~202 s against ~94 s on the fast
loop" — two numbers taken in **different machine states**, which is exactly the
comparison the Measurements section below discards. Those two runs were the *signal I
dismissed as contention* on the way to this finding; they are not evidence for it. The
613.7/273.4 pair is, because it is a pair.

The measurement that exposed (1) is worth repeating on any future attempt: give a
session-scoped fixture a side effect keyed by `os.getpid()`, and **count the
processes**. "The suite passed" and "the grouping worked" are unrelated statements —
and the probe works because it observes the *mechanism* rather than the *outcome*. The
two residuals recorded below (the Tier-2 recomputation count, cargo's lock
serialization) are claimed in prose with nothing that would go red if they changed; the
PID-count probe generalizes directly to the first of them.

### What `load` therefore leaves on the table

* `tests/crossport/` self-serializes on cargo's lock — correct, just with workers idle
  against it. It is the floor under any parallel run of the fast loop.
* **`sealed_tier2_run` is recomputed per worker**, because session scope is per
  process. Measured at `-n 12`: **two** setups of ~240 s under the shipped `load`
  default, and **four** of ~305 s under the removed `loadgroup` one. So the slow tier
  gains less than the fast loop does — parallelism cannot amortize a per-process
  fixture, it can only stop making it worse.
  ⚠ **2026-08-10: the count is set by the SELECTION, not only by the mode, and the
  sentence above reads as though the mode determined it.** Under the shipped `load`
  default on the same box within one hour: the **full suite** paid **two** setups
  (379.6 s, 374.3 s) while **`-m slow` alone** paid **four** (318.7/316.7/315.8/262.2 s).
  Consequence, and it is the counterintuitive part: **running only the slow tier is
  SLOWER than running the whole suite** — 504.47 s against 425.49 s, i.e. adding 2080
  fast tests made the run finish sooner. Why is **not measured**: the plausible reading
  is that with more items in the queue the six consumers land on fewer distinct workers,
  but that is a claim about xdist's scheduler and the probe that would settle it is the
  PID count already prescribed above. Recorded as a measured *count* with an unmeasured
  *cause*.

Fixing that properly means either declaring the group in-file (`pytestmark` in both
`test_sealed_station_stability.py` and `test_regression_sealed_station.py`) and paying
`loadgroup`'s scheduling cost for the whole suite, or caching the Tier-2 trajectory to
disk — which introduces a stale-artifact hazard into the one run that gates a frozen
golden. Neither is taken here; both are recorded rather than rediscovered.

## Hypothesis deadlines are off

Parallelising the suite produced a failure that has nothing to do with the code:

```
test_biosphere_demo.py::test_demo_run_is_registration_order_independent[Rk4Integrator]
hypothesis.errors.DeadlineExceeded: Test took 441.58ms, which exceeds the deadline of 200.00ms
```

Hypothesis's default `deadline=200ms` is a **wall-clock assertion**, and these property
tests each build and run a whole scenario. `tests/conftest.py` registers a profile with
`deadline=None` and suppresses `HealthCheck.too_slow`.

This is not weakening a gate. No property in this repo is about *speed* — they are
conservation, non-negativity and order-independence laws; performance is tracked
separately and non-gatingly by `bench/perf.py`. A per-example timer measures the
machine's spare capacity, which is exactly the quantity the parallel loop and the
priority drop deliberately vary.

⚠ **A first draft of this section added "and it was already near the edge serially —
the RK4 examples sit within ~2x of the default". That is false, and it was an
inference, not a measurement.** 441 ms is a *loaded-machine* number; scaling it by an
assumed contention factor to land near 200 ms is the shape this repo keeps catching —
a figure from one condition written as a property of the code. Measured instead:
serially at normal priority, that test's entire call (every example) takes **0.10 s**,
~17 ms per example, **over 10x clear** of the deadline.

The correction strengthens the decision rather than undermining it: the deadline fires
on *contention alone*, so it was never measuring the test. The default is kept
available under a name so this stays checkable rather than becoming folklore:

```
uv run pytest --hypothesis-profile=strict-deadline
```

## Measurements

Machine: 16 logical cores, Windows 11, Python 3.13. Rows are only comparable **within**
a matched pair — see the warning below.

| Run | Wall clock | Speed-up |
|---|---|---|
| fast loop (`-m "not slow"`), serial | 167.6 s | — |
| fast loop, `-n 12` | **51.1 s** | **3.3x** |
| full suite (slow tier included), `-n 12` | **273.4 s** | vs `~9 min` serial as documented |
| full suite, `-n 12`, under the *removed* `loadgroup` design | 613.7 s | — |
| `tests/crossport` alone, serial (rust pre-built) | 41.6 s (96 tests) | — |
| `tests/crossport` alone, `-n 8` | 51.3 s | **0.8x** |
| earlier matched pair, before `loadgroup` was removed | 270.4 s → 94.1 s | 2.9x |

Re-measured **2026-08-10**, all three inside one hour on the same box, `-n 12`:

| Run | Wall | Tests |
|---|---|---|
| fast loop (`-m "not slow"`) | **49.58 s** | 2080 |
| slow tier (`-m slow`) | **504.47 s** | 82 |
| full suite | **425.49 s** | 2162 |

The fast-loop row is the **control**: it reproduces the 51.1 s above to within 3 %, which
is what licenses the other two as readings of the box rather than of its load. No growth
*ratio* is quoted against the 273.4 s row — the suite gained ~160 tests in between,
several of them minute-scale, so that comparison would conflate suite size with machine
state. The absolute figure, dated, is the honest form.

⚠ **A 2197 s (36 m 37 s) reading of the full suite was taken the same day and is
DISCARDED — and the error was not the reading, it was what I did with it.** It was
reported as fact and then used to conclude this document was "~8x stale", which is an
inference from one measurement in an unverified machine state — the exact shape the
warning below exists for, committed one screen after quoting that warning. The cause was
**not isolated**: a concurrent run of the same suite is the leading candidate (an earlier
background invocation whose completion could not be confirmed — its output file is empty
and its timestamps neither establish nor rule out overlap), but that was never proven, so
it is recorded as unexplained rather than as self-inflicted contention. What the reading
*does* license: nothing. The pass count from that run (2161) is unaffected and was never
in doubt — **a run's verdict and its wall clock fail independently.**

The full-suite rows are the ones that justify having chased the distribution mode:
dropping `loadgroup` **halved** the full run, and took the Tier-2 fixture from four
recomputations to two.

⚠ **Wall-clock numbers taken across different machine states are discarded, not
reported.** Re-runs of identical commands came back at 197 s, 202 s and 310 s —
including one *slower than serial* — and the cause was found rather than assumed:
unrelated jobs (another project's `pytest -n 4`, whose orphaned workers each held
~620 s of CPU, plus a long-running census script and Windows Defender at ~2600 s) were
saturating the box. A wall-clock number from a contended machine measures the
contention. Two separate matched pairs give 3.3x and 2.9x while their absolute numbers
differ by ~1.6x — quote the ratio, and only from a back-to-back pair.

## Serial and parallel agree on outcomes, not just on colour

"Both runs were green" is not the check this repo needs — the goldens are the point.
Verified on the fast loop as a matched pair, serial vs `-n 12`, JUnit XML from each
compared by test id:

* **2003 test ids in both**, identical sets, **zero outcome differences**;
* `git status --porcelain` identical before and after both runs — nothing regenerated
  a committed artifact into an order-dependent value.

Worth re-running after any change to the distribution mode or worker count.

## The remaining lever, priced but not taken

`tests/crossport/` spends ~0.43 s per test, most of it `cargo run` overhead re-checking
freshness on an already-built workspace, and it is the floor under any parallel fast
loop. Invoking the pre-built example binaries directly
(`rust/target/debug/examples/*.exe`) would remove both the overhead and the lock — but
it means changing how ~12 files in the cross-port *contract's* harness invoke the port,
and it trades cargo's freshness check for a stale-binary hazard that nothing would
catch. Recorded as a seam with a named obstacle, not a recommendation.
