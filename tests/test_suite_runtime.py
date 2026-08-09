"""Pins for the *test harness itself* — process priority and xdist work grouping.

These are not science gates; they guard two mechanisms in ``tests/conftest.py`` that
are otherwise **silent when broken**:

* The priority drop is a ctypes call into ``kernel32``. Its first implementation
  failed with ``ERROR_INVALID_HANDLE`` (ctypes marshalled the pointer-sized pseudo
  handle as a 32-bit int) and the suite ran at normal priority with every test
  green — exactly the class of failure a pin exists for.
* The hypothesis profile that removes the default 200ms per-example deadline. The
  deadline is a wall-clock assertion in a suite that has none, and it fires on load
  alone; the pin here is that the *strict* profile stays reachable, so the claim
  remains checkable rather than becoming folklore.

An earlier version of this file also pinned an ``xdist_group`` assignment. That
mechanism was **removed after being measured non-functional** — a group assigned from
a collection hook is dropped, because xdist stamps the ``@group`` nodeid suffix its
scheduler reads before this suite's hook runs. The pins went with it rather than being
kept green against a mechanism that does nothing.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent))
from conftest import _PRIORITY_ENV, _WIN_PRIORITY_CLASSES  # noqa: E402

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


def test_the_strict_hypothesis_deadline_profile_stays_available() -> None:
    """`--hypothesis-profile=strict-deadline` restores hypothesis's default 200ms.

    The suite runs with `deadline=None`, on the grounds that the deadline measures
    machine load rather than the code. That grounds statement is only honest while it
    stays *falsifiable*, so the strict profile is registered rather than the default
    simply discarded.
    """
    from hypothesis import settings

    assert settings.get_profile("strict-deadline").deadline is not None
    assert settings.get_profile("simsuite").deadline is None
