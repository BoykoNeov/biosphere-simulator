"""Oracle-marked: a fresh LINTUL3 run reproduces the committed spring-wheat fixture.

Proves the runner is deterministic and the committed fixture is current. Unlike the
winter-wheat regeneration test this needs **no network** — LINTUL3's inputs (params +
CABO weather) ship with PCSE (offline). Marked ``oracle`` (opt-in: ``-m oracle``) and
guarded by ``importorskip`` so a plain ``uv run pytest`` skips it cleanly.
"""

import math

import pytest

pytestmark = pytest.mark.oracle

pytest.importorskip("pcse", reason="oracle dep group not installed")

from .lintul3_runner import (  # noqa: E402
    OUTPUT_VARIABLES,
    column,
    load_fixture,
    run_spring_wheat,
)


def test_fresh_run_reproduces_committed_fixture() -> None:
    committed = load_fixture()
    fresh = run_spring_wheat()

    assert len(fresh["trajectory"]) == len(committed["trajectory"]), (
        "fresh LINTUL3 run has a different number of days than the committed fixture — "
        "regenerate with `uv run --group oracle python -m tests.oracle.lintul3_runner`"
    )

    for var in OUTPUT_VARIABLES:
        # Drop pre-emergence null cells (both series share the same null positions).
        # A deterministic re-run reproduces every cell to float noise — compare
        # elementwise (nrmse cannot normalize a flat series like TRANRF ≡ 1.0).
        for r, g in zip(column(committed, var), column(fresh, var), strict=True):
            if r is None or g is None:
                continue
            assert math.isclose(r, g, rel_tol=1e-9, abs_tol=1e-9), (
                f"variable {var} drifted from the fixture ({r} vs {g})"
            )
