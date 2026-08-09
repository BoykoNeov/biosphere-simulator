"""Pytest collection hooks shared across the suite.

Three concerns live here:

1. **Robust opt-in for the ``oracle`` marker.** A naive ``addopts = "-m 'not oracle'"``
   is *not* override-proof: pytest's ``-m`` is last-wins, so the documented fast-loop
   command ``-m "not slow"`` would replace it and silently re-include the oracle test —
   which, on a machine that has ``pcse`` installed, then hits the network. The hook
   enforces opt-in regardless of the ``-m`` expression: oracle-marked items are skipped
   **unless** the marker expression explicitly mentions ``oracle`` (``-m oracle`` to run
   them; ``-m "not oracle"`` to deselect them — both honored, anything else skips them).

2. **Below-normal process priority (always on, opt-out).** The suite is CPU-bound and
   the parallel loop saturates every core; dropping the priority *class* keeps it from
   competing with interactive work on the same machine. It is a scheduling hint only —
   no test in this repo asserts wall-clock time (``docs/perf-baseline.md`` is produced
   by ``bench/perf.py``, a separate script, and is explicitly NON-GATING), so this
   cannot change a result. Opt out with ``SIMTEST_PRIORITY=normal``.

3. **xdist work grouping.** The parallel loop needs ``--dist loadgroup`` to be correct,
   not merely fast — see :func:`pytest_collection_modifyitems` for the two groups that
   are about correctness/cost rather than balance.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Protocol

import pytest
from hypothesis import HealthCheck, settings

# --------------------------------------------------------------------------- #
# Hypothesis: no wall-clock deadline                                          #
# --------------------------------------------------------------------------- #

# Hypothesis's default `deadline=200ms` is a **wall-clock** assertion, and this
# suite's property tests each build and run a whole scenario — `test_biosphere_demo
# ::test_demo_run_is_registration_order_independent[Rk4Integrator]` measures ~440ms
# per example on a machine running 12 test workers. It is a latent flake serially
# too (the RK4 examples already sit within ~2x of the default), and it became a
# reproducible failure the moment the suite was parallelized.
#
# Deselecting the deadline is not weakening a gate: no property in this repo is
# about *speed* — they are conservation, non-negativity and order-independence laws,
# and performance is tracked separately and NON-GATINGLY by `bench/perf.py` /
# `docs/perf-baseline.md`. A per-example timer is measuring the machine's spare
# capacity, which is precisely the thing the parallel loop and the priority drop
# below are deliberately varying.
#
# `too_slow` is suppressed for the same reason, one layer up: it is a health check
# on data-generation wall time, equally a statement about load rather than about the
# code under test.
settings.register_profile(
    "simsuite",
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow],
)
settings.load_profile("simsuite")

# --------------------------------------------------------------------------- #
# Process priority                                                            #
# --------------------------------------------------------------------------- #

# Windows priority classes (winbase.h). We deliberately use the priority *class*
# rather than PROCESS_MODE_BACKGROUND_BEGIN: a class is inherited by child
# processes, so setting it once covers the xdist workers *and* the `cargo run`
# subprocesses that tests/crossport/ spawns — which are the heaviest thing the
# suite starts. Background mode applies to the calling process only and throttles
# I/O hard enough to be its own source of weirdness.
_WIN_PRIORITY_CLASSES = {
    "below": 0x00004000,  # BELOW_NORMAL_PRIORITY_CLASS
    "idle": 0x00000040,  # IDLE_PRIORITY_CLASS
}
# POSIX nice increments (higher = nicer).
_POSIX_NICE = {"below": 5, "idle": 15}

_PRIORITY_ENV = "SIMTEST_PRIORITY"


def _lower_process_priority() -> str:
    """Drop this process's scheduling priority. Returns a short status string.

    Called from ``pytest_configure``, which runs in the master **and** in every
    xdist worker; ``SetPriorityClass``/``nice`` are idempotent enough that the
    belt-and-braces double application costs nothing (on POSIX the workers inherit
    the master's niceness and then re-apply their own increment relative to it,
    which is at worst *nicer* — never less nice).
    """
    level = os.environ.get(_PRIORITY_ENV, "below").strip().lower()
    if level in ("normal", "off", "0", "no"):
        return "normal (opt-out)"
    if level not in _WIN_PRIORITY_CLASSES:
        return f"normal (unknown {_PRIORITY_ENV}={level!r})"

    if sys.platform == "win32":
        import ctypes

        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        # The signatures are declared, not left to ctypes' defaults: a HANDLE is
        # pointer-sized and ctypes would otherwise marshal it as a 32-bit int,
        # truncating the -1 pseudo-handle and failing with ERROR_INVALID_HANDLE (6).
        kernel32.GetCurrentProcess.restype = ctypes.c_void_p
        kernel32.GetCurrentProcess.argtypes = []
        kernel32.SetPriorityClass.restype = ctypes.c_int
        kernel32.SetPriorityClass.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
        ok = kernel32.SetPriorityClass(
            kernel32.GetCurrentProcess(), _WIN_PRIORITY_CLASSES[level]
        )
        if not ok:
            return f"normal (SetPriorityClass failed: {ctypes.get_last_error()})"
        return level
    try:
        os.nice(_POSIX_NICE[level])  # type: ignore[attr-defined]
    except (AttributeError, OSError) as exc:  # pragma: no cover - platform-dependent
        return f"normal (nice failed: {exc})"
    return level


_PRIORITY_STASH: pytest.StashKey[str] = pytest.StashKey()


def pytest_configure(config: pytest.Config) -> None:
    status = _lower_process_priority()
    # Only the master reports; the workers would each repeat the line.
    if not hasattr(config, "workerinput"):
        config.stash[_PRIORITY_STASH] = status

    # `--dist loadgroup` is what makes the xdist_group markers assigned below take
    # effect. Without it, `-n auto` silently (a) scatters tests/crossport/ across
    # workers, where they contend on cargo's exclusive `target/` lock and run
    # *slower* than serial (measured: 96 tests, 41.6s serial vs 51.3s at -n 8), and
    # (b) splits the two `sealed_tier2_run` consumers across workers, paying that
    # ~3-minute run twice. So default it on whenever the user asks for workers, but
    # never override an explicitly-passed --dist.
    explicit_dist = any(
        arg == "-d" or arg.startswith("--dist") for arg in config.invocation_params.args
    )
    if (
        getattr(config.option, "numprocesses", None)
        and not explicit_dist
        and getattr(config.option, "dist", "no") in ("no", "load")
    ):
        config.option.dist = "loadgroup"


def pytest_report_header(config: pytest.Config) -> list[str]:
    status = config.stash.get(_PRIORITY_STASH, None)
    return [f"process priority: {status}"] if status else []


# --------------------------------------------------------------------------- #
# Collection hooks                                                            #
# --------------------------------------------------------------------------- #

_CROSSPORT_DIR = Path(__file__).parent / "crossport"


class _Groupable(Protocol):
    """The two attributes :func:`_xdist_group` reads off a collected item.

    Narrower than ``pytest.Item`` on purpose: ``fixturenames`` lives on ``Function``,
    not ``Item``, and typing the parameter by what is actually read lets the pins in
    ``tests/test_suite_runtime.py`` call it with a stub instead of constructing a real
    collection node.
    """

    @property
    def path(self) -> Path: ...


def _xdist_group(item: _Groupable) -> str | None:
    """The xdist worker-affinity group for ``item``, or ``None`` to load-balance it.

    Exactly two groups exist, and both are about **correctness or cost, never
    balance** — the rest of the suite is left free to load-balance:

    * ``crossport`` — every test under ``tests/crossport/`` shells out to
      ``cargo run --example``, and cargo takes an exclusive lock on ``target/``.
      Spreading them across workers does not corrupt anything, it *blocks*: measured
      41.6s pinned to one worker vs 51.3s at ``-n 8``. Pinned is both faster and the
      only arrangement that does not burn 8 cores on a lock.
    * ``sealed_tier2`` — the session-scoped ``sealed_tier2_run`` fixture (~3 min) is
      shared by two *different files*, so file affinity would not be enough anyway.
      Session scope is per-worker under xdist, so without this the run is paid twice.

    Returning ``None`` is not a gap — it is the whole point, and it rests on a
    *structural* fact about xdist rather than on a timing: ``LoadGroupScheduling``
    only collapses a scope when the nodeid carries an ``@group`` suffix, and returns
    the **full nodeid** otherwise (``xdist/scheduler/loadgroup.py``). So an unmarked
    test is its own scope, i.e. ``--dist loadgroup`` load-balances per test exactly
    like ``--dist load``, and marking the two special cases costs the rest of the
    suite nothing.

    A third, tempting group is therefore **rejected without needing a benchmark**:
    grouping every remaining item by its file (reproducing ``--dist loadfile``, so
    this suite's ~30 module-scoped scenario fixtures are not recomputed on each
    worker that receives one of their tests) would replace per-test balance with
    per-file balance for ~1900 tests, to save fixture work in the tail. It was tried
    and read ~2x slower, but that measurement was taken while unrelated jobs were
    saturating the machine and is **not** quoted as the reason.
    """
    if _CROSSPORT_DIR in Path(str(item.path)).parents:
        return "crossport"
    if "sealed_tier2_run" in getattr(item, "fixturenames", ()):
        return "sealed_tier2"
    return None


def pytest_collection_modifyitems(
    config: pytest.Config, items: list[pytest.Item]
) -> None:
    for item in items:
        group = _xdist_group(item)
        if group is not None:
            item.add_marker(pytest.mark.xdist_group(group))

    markexpr = config.getoption("markexpr") or ""
    # If the user named `oracle` in their expression at all, respect it verbatim
    # (`-m oracle` runs; `-m "not oracle"` deselects). Otherwise, opt-out by default.
    if "oracle" in markexpr:
        return
    skip_oracle = pytest.mark.skip(reason="oracle test: opt-in with `-m oracle`")
    for item in items:
        # Match the actual `oracle` marker — NOT `"oracle" in item.keywords`, which
        # also matches the `tests/oracle/` package name and would wrongly skip the
        # always-run fixture checks that live there.
        if item.get_closest_marker("oracle") is not None:
            item.add_marker(skip_oracle)


@pytest.fixture(scope="session")
def sealed_tier2_run():
    """The canonical Tier-2 sealed-station trajectory, run **once** per session (P6.7).

    The ~1.3 M-sub-step run (~3 min) is shared between the stability gate
    (``test_sealed_station_stability``) and the regression golden
    (``test_regression_sealed_station``) so it is not paid twice. Session-scoped, so it
    is
    computed lazily only when a slow-marked test that requests it actually runs (the
    fast
    ``-m "not slow"`` loop never triggers it). Returns a
    :class:`sealed_tier2_helper.Tier2Run` (the day-boundary states + rationed + events).

    Under xdist, session scope is **per worker** — the ``sealed_tier2`` xdist group in
    :func:`_xdist_group` is what keeps both consumers on one worker so this stays a
    single run.
    """
    from sealed_tier2_helper import run_tier2

    return run_tier2()
