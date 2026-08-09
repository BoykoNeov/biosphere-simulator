"""Pins for the *test harness itself* — process priority and xdist work grouping.

These are not science gates; they guard two mechanisms in ``tests/conftest.py`` that
are otherwise **silent when broken**:

* The priority drop is a ctypes call into ``kernel32``. Its first implementation
  failed with ``ERROR_INVALID_HANDLE`` (ctypes marshalled the pointer-sized pseudo
  handle as a 32-bit int) and the suite ran at normal priority with every test
  green — exactly the class of failure a pin exists for.
* The xdist groups are load-bearing for *cost*, not correctness of results: lose the
  ``crossport`` group and 96 tests contend on cargo's exclusive ``target/`` lock;
  lose ``sealed_tier2`` and its ~3-minute fixture is computed on two workers. Both
  degrade silently — the suite still passes, just slower.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent))
from conftest import _PRIORITY_ENV, _WIN_PRIORITY_CLASSES, _xdist_group  # noqa: E402

_OPTED_OUT = os.environ.get(_PRIORITY_ENV, "below").strip().lower() in (
    "normal",
    "off",
    "0",
    "no",
)

pytestmark = pytest.mark.skipif(
    _OPTED_OUT, reason=f"{_PRIORITY_ENV} opts out of the priority drop"
)


def _win_priority_class(pid: int | None = None) -> int:
    import ctypes

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.GetPriorityClass.restype = ctypes.c_uint32
    kernel32.GetPriorityClass.argtypes = [ctypes.c_void_p]
    if pid is None:
        kernel32.GetCurrentProcess.restype = ctypes.c_void_p
        handle = kernel32.GetCurrentProcess()
        return kernel32.GetPriorityClass(handle)
    PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
    kernel32.OpenProcess.restype = ctypes.c_void_p
    kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
    handle = kernel32.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
    assert handle, f"OpenProcess failed: {ctypes.get_last_error()}"
    try:
        return kernel32.GetPriorityClass(ctypes.c_void_p(handle))
    finally:
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle(ctypes.c_void_p(handle))


@pytest.mark.skipif(sys.platform != "win32", reason="Windows priority classes")
def test_this_process_runs_below_normal() -> None:
    """The conftest hook actually took effect in *this* process.

    ``pytest_configure`` runs in the master and in every xdist worker, so this holds
    whichever process the test lands in.
    """
    assert _win_priority_class() == _WIN_PRIORITY_CLASSES["below"]


@pytest.mark.skipif(sys.platform != "win32", reason="Windows priority classes")
def test_the_priority_class_is_inherited_by_child_processes() -> None:
    """The reason a priority *class* was chosen over ``PROCESS_MODE_BACKGROUND_BEGIN``.

    The heaviest processes this suite starts are not Python: ``tests/crossport/``
    spawns ``cargo run``, which spawns ``rustc``/linkers. A class is inherited by
    children, so one call in the pytest process covers all of them; background mode
    would have covered the caller only. Measured here rather than asserted in a
    comment, because a switch to background mode would leave every other pin green.
    """
    proc = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(5)"])
    try:
        assert _win_priority_class(proc.pid) == _WIN_PRIORITY_CLASSES["below"]
    finally:
        proc.kill()
        proc.wait()


@pytest.mark.skipif(sys.platform == "win32", reason="POSIX niceness")
def test_this_process_is_niced() -> None:
    assert os.nice(0) > 0  # type: ignore[attr-defined]


# --------------------------------------------------------------------------- #
# xdist grouping                                                              #
# --------------------------------------------------------------------------- #


class _StubItem:
    """Minimal stand-in for the two attributes ``_xdist_group`` reads."""

    def __init__(self, path: Path, fixturenames: tuple[str, ...] = ()) -> None:
        self.path = path
        self.fixturenames = fixturenames


_TESTS_DIR = Path(__file__).parent


def test_every_crossport_test_lands_in_one_group() -> None:
    """Because cargo's ``target/`` lock is exclusive — measured 41.6s pinned vs 51.3s
    at ``-n 8``, i.e. spreading them out is *slower*, not merely wasteful."""
    crossport_files = sorted((_TESTS_DIR / "crossport").glob("test_*.py"))
    assert crossport_files, "expected the crossport suite to exist"
    groups = {_xdist_group(_StubItem(p)) for p in crossport_files}
    assert groups == {"crossport"}


def test_the_sealed_tier2_consumers_share_a_group_across_files() -> None:
    """File affinity is not enough: the ~3-minute fixture is session-scoped, and
    session scope under xdist is **per worker** — so its two consumers, which live in
    two different files, have to be pinned together explicitly."""
    consumers = [
        _StubItem(
            _TESTS_DIR / "test_regression_sealed_station.py", ("sealed_tier2_run",)
        ),
        _StubItem(
            _TESTS_DIR / "test_sealed_station_stability.py", ("sealed_tier2_run",)
        ),
    ]
    assert {_xdist_group(i) for i in consumers} == {"sealed_tier2"}


def test_ordinary_tests_are_left_free_to_load_balance() -> None:
    """The rejected third group, pinned so it is not re-introduced by instinct.

    Grouping every remaining item by its file (``--dist loadfile`` semantics) would
    stop this suite's ~30 module-scoped scenario fixtures from being recomputed on
    each worker that receives one of their tests — at the price of demoting ~1900
    tests from per-test to per-file balance. Leaving them unmarked is free: xdist's
    ``LoadGroupScheduling`` collapses a scope only for nodeids carrying an ``@group``
    suffix and returns the full nodeid otherwise, so an unmarked test is its own
    scope and ``loadgroup`` balances it exactly as ``load`` would.
    """
    assert _xdist_group(_StubItem(_TESTS_DIR / "test_carbon_budget.py")) is None
