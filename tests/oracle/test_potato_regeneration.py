"""Oracle-marked: a fresh WOFOST run reproduces the committed potato fixture.

Proves the runner is deterministic and the committed fixture is current. Like the
LINTUL3 one this needs **no network** — the potato inputs ship with PCSE as its own
bundled demo database (``pcse/tests/test_data/pcse_dump.sql``, loaded at ``import
pcse``). Marked ``oracle`` (opt-in: ``-m oracle``) and guarded by ``importorskip`` so a
plain ``uv run pytest`` skips it cleanly.

This file also carries the check that is *new* for the potato oracle: the demo DB ships
``wofost_unittest_benchmarks``, PCSE's OWN expected daily output for this exact
(grid, crop, year, mode). Agreeing with it says we drove the model correctly — a
**runner** check, not a second oracle, and it carries no authority the model itself
lacks. It is the cheapest possible guard against the one mistake that would quietly
invalidate every number in ``tests/test_potato_crop.py``: running the wrong crop, site
or production mode and never noticing.
"""

import math

import pytest

pytestmark = pytest.mark.oracle

pytest.importorskip("pcse", reason="oracle dep group not installed")

from .wofost_potato_runner import (  # noqa: E402
    OUTPUT_VARIABLES,
    benchmark_deltas,
    column,
    load_fixture,
    run_potato,
)

# The demo DB stores its benchmark values as SQL text, so a round-trip loses a little
# precision; these are absolute tolerances per variable's own scale (kg/ha for the
# weights, dimensionless for DVS/LAI/SM, cm for RD/TRA). Sized ~1000x above the
# deviations actually observed (max 1e-3 on TAGP), so this fails on a WRONG RUN, not on
# formatting noise.
_BENCHMARK_ABS_TOL = 1.0


def test_fresh_run_reproduces_committed_fixture() -> None:
    committed = load_fixture()
    fresh = run_potato()

    assert len(fresh["trajectory"]) == len(committed["trajectory"]), (
        "fresh WOFOST run has a different number of days than the committed fixture — "
        "regenerate with "
        "`uv run --group oracle python -m tests.oracle.wofost_potato_runner`"
    )
    assert (
        fresh["provenance"]["milestones_days_since_emergence"]
        == committed["provenance"]["milestones_days_since_emergence"]
    )

    for var in OUTPUT_VARIABLES:
        for reference, generated in zip(
            column(committed, var), column(fresh, var), strict=True
        ):
            if reference is None or generated is None:
                continue
            assert math.isclose(reference, generated, rel_tol=1e-9, abs_tol=1e-9), (
                f"variable {var} drifted from the fixture ({reference} vs {generated})"
            )


def test_our_run_matches_pcses_own_shipped_expectation() -> None:
    # If this fails, the diagnostic in tests/test_potato_crop.py is comparing against
    # something other than the WOFOST potato potential-production run it claims to.
    deltas = benchmark_deltas(load_fixture()["trajectory"])
    assert deltas, (
        "no wofost_unittest_benchmarks rows matched this run's "
        "(grid, crop, year, mode) — the cross-check silently did nothing"
    )
    for variable, delta in deltas.items():
        assert delta < _BENCHMARK_ABS_TOL, (
            f"{variable} deviates from PCSE's own shipped benchmark by {delta} — "
            "we are not driving the model the way PCSE's own test does"
        )
