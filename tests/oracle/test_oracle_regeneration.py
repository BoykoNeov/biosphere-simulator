"""Oracle-marked: a fresh PCSE run reproduces the committed fixture (P5 pipeline).

This is the only test that needs PCSE (and network, for NASAPower weather). It proves
the runner is deterministic and the committed fixture is current: regenerating the
trajectory and comparing it to the committed reference via the behavioral-match helper
should agree to within float noise. A failure means either the runner drifted or the
upstream weather/param data was revised — both are real signals to regenerate.

Marked ``oracle`` (opt-in: ``-m oracle``) and guarded by ``importorskip`` so a plain
``uv run pytest`` on a machine without the ``oracle`` dep group skips it cleanly.
"""

import pytest

pytestmark = pytest.mark.oracle

pytest.importorskip("pcse", reason="oracle dep group not installed")

from lab.oracle_match import nrmse  # noqa: E402

from .runner import (  # noqa: E402
    OUTPUT_VARIABLES,
    column,
    load_fixture,
    run_winter_wheat,
)


def test_fresh_run_reproduces_committed_fixture() -> None:
    committed = load_fixture()
    fresh = run_winter_wheat()

    assert len(fresh["trajectory"]) == len(committed["trajectory"]), (
        "fresh run has a different number of days than the committed fixture — "
        "regenerate with `uv run --group oracle python -m tests.oracle.runner`"
    )

    for var in OUTPUT_VARIABLES:
        ref = column(committed, var)
        got = column(fresh, var)
        # Deterministic re-run ⇒ effectively identical; allow only float noise.
        assert nrmse(ref, got) < 1.0e-9, f"variable {var} drifted from the fixture"
