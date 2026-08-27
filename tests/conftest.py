"""Pytest collection hooks for what is left of the Python suite.

**One concern lives here now: robust opt-in for the ``oracle`` marker.** A naive
``addopts = "-m 'not oracle'"`` is *not* override-proof — pytest's ``-m`` is last-wins,
so any other marker expression would replace it and silently re-include the oracle
tests, which on a machine that has ``pcse`` installed then hit the network. The hook
enforces opt-in regardless of the ``-m`` expression: oracle-marked items are skipped
**unless** the expression explicitly mentions ``oracle`` (``-m oracle`` to run them,
``-m "not oracle"`` to deselect them — both honored, anything else skips them).

⚠⚠ **Two other concerns were deleted here on 2026-08-27 (S6 build item 3), and the
reason is the trap this repo names for ``config/units.py``: a live import guarding a
dead path.**

* A **hypothesis** profile that disabled the wall-clock deadline and the ``too_slow``
  health check. Every property test in this repo went with the deleted suite —
  ``grep`` over ``tests/`` finds no ``@given`` and no ``strategies`` — so the profile
  configured nothing. It was also a **single point of failure**: an ``import
  hypothesis`` at the top of the one conftest the PCSE carve-out collects through,
  guarding no test, would take the carve-out down with it the day hypothesis fell out
  of the lock. The finding it recorded — that the 200 ms deadline fires on *contention
  alone*, ~17 ms per example measured serially — is in ``docs/test-suite-runtime.md``.
* A **below-normal process priority class**, whose stated rationale was that "the suite
  is CPU-bound and the parallel loop saturates every core". There is no parallel loop:
  twelve tests that read committed JSON fixtures run in under a second. Its recorded
  detail (the priority *class* rather than ``PROCESS_MODE_BACKGROUND_BEGIN``, so child
  processes inherit it) and the measured negative result about ``--dist loadgroup``
  (a marker added by a collection hook never takes effect; ``loadgroup`` is
  ``LoadScopeScheduling`` and was ~2x slower on this suite) are both in
  ``docs/test-suite-runtime.md``, which has been a **record** rather than current
  advice since S6.

⚠ Neither removal was a judgement about whether the reasoning was good — it was good,
and it is kept. Both had simply stopped having a subject, and this file is one of the
thirteen the repo now claims are load-bearing.
"""

from __future__ import annotations

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
