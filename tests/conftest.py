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

3. **A recorded negative result about xdist grouping**, in :func:`pytest_configure`:
   worker affinity via ``xdist_group`` cannot be assigned from a collection hook, and
   ``--dist loadgroup`` is the wrong mode for a suite that is mostly ungrouped. Kept
   as a comment because both halves look obviously right and were measured false.
"""

from __future__ import annotations

import os
import sys

import pytest
from hypothesis import HealthCheck, settings

# --------------------------------------------------------------------------- #
# Hypothesis: no wall-clock deadline                                          #
# --------------------------------------------------------------------------- #

# Hypothesis's default `deadline=200ms` is a **wall-clock** assertion, and this
# suite's property tests each build and run a whole scenario — `test_biosphere_demo
# ::test_demo_run_is_registration_order_independent[Rk4Integrator]` measured ~440ms
# for a single example on a machine running 12 test workers, and failed.
#
# ⚠ The margin is NOT thin, and an earlier draft of this comment claimed it was
# ("a latent flake serially too, within ~2x of the default") — an inference from
# the 440ms, which is a *loaded-machine* number, dressed as a property of the test.
# Measured serially at normal priority, that test's whole call — every example —
# takes **0.10s**, i.e. ~17ms per example, over 10x clear of the deadline. So the
# deadline fires on *contention alone*, which does not weaken the case for
# deselecting it — it is the case. Re-check with
# `pytest --hypothesis-profile=strict-deadline`.
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
# The default kept available under a name, so the claim "these tests would trip the
# 200ms deadline" stays *checkable* instead of becoming folklore once the profile
# above hides it: `uv run pytest --hypothesis-profile=strict-deadline`.
settings.register_profile("strict-deadline", deadline=200)
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

    # NOTE: `--dist` is deliberately NOT overridden here. An earlier version of this
    # file forced `loadgroup` under `-n` so that xdist_group markers assigned in
    # `pytest_collection_modifyitems` would pin `tests/crossport/` and the
    # `sealed_tier2_run` consumers to single workers. Both halves were measured false:
    #
    #  * A marker added by a *collection hook* never takes effect. xdist applies the
    #    `@group` nodeid suffix its scheduler reads in `xdist/remote.py`, and that runs
    #    before this conftest's hook — so the group is silently dropped. Verified with
    #    a synthetic 4-file suite sharing one session fixture: hook-assigned markers
    #    left the fixture computed in **4 processes**, the documented in-file
    #    `pytestmark = pytest.mark.xdist_group(...)` collapsed it to **1**.
    #  * `loadgroup` is `LoadScopeScheduling`, which dispatches per *scope*. With most
    #    of a suite ungrouped every test is its own scope, and the scheduler is far
    #    slower than `load`'s per-test dispatch — 10.0s vs 1.07s on that same 8-test
    #    probe, and ~202s vs ~94s on this suite's fast loop.
    #
    # xdist's own default under `-n` is `load`, which is the right one here. See
    # docs/test-suite-runtime.md for what that leaves on the table.


def pytest_report_header(config: pytest.Config) -> list[str]:
    status = config.stash.get(_PRIORITY_STASH, None)
    return [f"process priority: {status}"] if status else []


# --------------------------------------------------------------------------- #
# Collection hooks                                                            #
# --------------------------------------------------------------------------- #


def pytest_collection_modifyitems(
    config: pytest.Config, items: list[pytest.Item]
) -> None:
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

    ⚠ **Under xdist, "once per session" means once per WORKER.** Its six consumers
    (four in the stability gate, two in the regression golden) load-balance across
    workers, so a parallel run of the slow tier recomputes this several times —
    measured as four ~305s setups in one ``-n 12`` full run. That is why parallelism
    buys the slow tier much less than it buys the fast loop, and it is not fixable by
    a marker: see the negative result in :func:`pytest_configure`.

    ⚠ **2026-08-10: "four ... in one full run" no longer reproduces, and the count is
    not a constant.** Re-measured under the shipped ``load`` default: the **full suite**
    pays **two** setups (379.6s, 374.3s) and ``-m slow`` **alone** pays **four**. So the
    recomputation count tracks the SELECTION, not the distribution mode — and the
    consequence is that running only the slow tier is *slower* than running everything
    (504.47s vs 425.49s). The sentence above is kept because the way it is wrong is the
    finding: a count measured once, under a mode since removed, written as a property of
    "a full run". See docs/test-suite-runtime.md.
    """
    from sealed_tier2_helper import run_tier2

    return run_tier2()
