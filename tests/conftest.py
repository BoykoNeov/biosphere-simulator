"""Pytest collection hooks shared across the suite.

Robust opt-in for the ``oracle`` marker. A naive ``addopts = "-m 'not oracle'"`` is
*not* override-proof: pytest's ``-m`` is last-wins, so the documented fast-loop
command ``-m "not slow"`` would replace it and silently re-include the oracle test —
which, on a machine that has ``pcse`` installed, then hits the network. This hook
enforces opt-in regardless of the ``-m`` expression: oracle-marked items are skipped
**unless** the marker expression explicitly mentions ``oracle`` (``-m oracle`` to run
them; ``-m "not oracle"`` to deselect them — both honored, anything else skips them).
"""

import pytest


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
    """
    from sealed_tier2_helper import run_tier2

    return run_tier2()
