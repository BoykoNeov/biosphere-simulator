# Test-suite runtime — parallelism, priority, and what is *not* claimed

**NON-GATING.** Nothing here changes a scientific result; it changes how long the
suite takes and how much of the machine it takes while doing it. The sibling document
`docs/perf-baseline.md` is about *engine* throughput and is a different subject.

## The commands

```
uv run pytest                 # everything (slow tier included — `slow` is opt-OUT)
uv run pytest -m "not slow"   # the fast loop
uv run pytest -n 12           # parallel; --dist loadgroup is turned on for you
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

## Why `--dist loadgroup`, and the two groups

`-n` without a `--dist` flag selects `loadgroup` (an explicit `--dist` is always
honoured). Two groups exist, both about **cost**, neither about balance:

| Group | Members | Why |
|---|---|---|
| `crossport` | everything under `tests/crossport/` | each test shells out to `cargo run --example`, and cargo takes an **exclusive** lock on `target/`. Spreading them across workers does not corrupt anything, it *blocks* — measured 41.6 s pinned to one worker against 51.3 s at `-n 8`. Pinned is both faster and the only arrangement that does not burn eight cores waiting on a lock. |
| `sealed_tier2` | the two consumers of the session-scoped `sealed_tier2_run` fixture | session scope under xdist is **per worker**, and the two consumers live in *different files* — so file affinity would not be enough either. Without the group the ~3-minute Tier-2 run is computed twice. |

Everything else is left unmarked, and that is free rather than a compromise:
`LoadGroupScheduling._split_scope` collapses a scope only for nodeids carrying an
`@group` suffix and returns the **full nodeid** otherwise, so an unmarked test is its
own scope and `loadgroup` balances it per-test exactly as `load` would.

A third group — every remaining item grouped by its file, i.e. `--dist loadfile`
semantics, so this suite's ~30 module-scoped scenario fixtures are not recomputed on
each worker receiving one of their tests — is **rejected**: it would demote ~1900
tests from per-test to per-file balance to save fixture work in the tail.

## Hypothesis deadlines are off

Parallelising the suite turned a latent flake into a reproducible failure:

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
priority drop deliberately vary. It was also already near the edge serially: the RK4
examples sit within ~2x of the default, so this was a flake waiting for a busy
afternoon.

## Measurements, and why only two of them are quoted

Machine: 16 logical cores, Windows 11, Python 3.13.

| Run | Wall clock | Speed-up |
|---|---|---|
| `-m "not slow"`, serial — quiet machine | 270.4 s | — |
| `-m "not slow"`, `-n 12` — quiet machine | 94.1 s | **2.9x** |
| `-m "not slow"`, serial — contended machine | 530.3 s | — |
| `-m "not slow"`, `-n 12` — contended machine | 178.0 s | **3.0x** |
| `tests/crossport` alone, serial (rust pre-built) | 41.6 s (96 tests) | — |
| `tests/crossport` alone, `-n 8` | 51.3 s | **0.8x** |

The two pairs are each **back-to-back on the same machine state**, which is the only
reason they can be compared at all — the absolute numbers differ by ~2x between the
pairs while the ratio does not.

⚠ **Wall-clock numbers taken across different machine states are discarded, not
reported.** Re-runs of identical commands came back at 197 s, 202 s and 310 s —
including one *slower than serial* — and the cause was found rather than assumed:
unrelated jobs (another project's `pytest -n 4`, whose orphaned workers each held
~620 s of CPU, plus a long-running census script and Windows Defender at ~2600 s) were
saturating the box. A wall-clock number from a contended machine measures the
contention.

So the honest statement is **~3x at `-n 12` on a 16-core box, reproduced across two
machine states** — not a scaling law, and well short of 12x, because the `crossport`
group is a serial floor under any parallel run.

## Serial and parallel agree on outcomes, not just on colour

"Both runs were green" is not the check this repo needs — the goldens are the point.
Verified on the fast loop, serial vs `-n 12`, JUnit XML from each compared by test id:

* **2005 test ids in both**, identical sets, **zero outcome differences**;
* `git status --porcelain` identical before and after both runs — nothing regenerated
  a committed artifact into an order-dependent value.

Worth re-running after any change to the grouping or worker count.

## The remaining lever, priced but not taken

`tests/crossport/` spends ~0.43 s per test, most of it `cargo run` overhead re-checking
freshness on an already-built workspace, and it is the critical path of any parallel
run. Invoking the pre-built example binaries directly
(`rust/target/debug/examples/*.exe`) would remove both the overhead and the lock — but
it means changing how ~12 files in the cross-port *contract's* harness invoke the port,
and it trades cargo's freshness check for a stale-binary hazard that nothing would
catch. Recorded as a seam with a named obstacle, not a recommendation.
