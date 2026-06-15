"""Skeleton smoke test (Phase 0, step 1).

Proves the editable install resolves every package and that `uv run pytest`
runs green. Real invariant tests (determinism, conservation, arbitration, ...)
arrive in later steps — see docs/plans/phase-0-engine-skeleton.md.
"""

import importlib

import pytest

PACKAGES = ["simcore", "sim_io", "config", "domains", "domains.biosphere"]


@pytest.mark.parametrize("name", PACKAGES)
def test_package_imports(name: str) -> None:
    assert importlib.import_module(name) is not None
