"""Tests for the behavioral-match helper (``lab/oracle_match.py``) and the discipline
that ``lab`` stays PCSE-free.

These are pure-stdlib tests — they need **no** PCSE (they compare in-memory series),
so they run on every plain ``uv run pytest``. The PCSE-dependent oracle regeneration
lives separately under ``tests/oracle/`` behind the ``oracle`` marker + importorskip.
"""

import ast
from pathlib import Path

import pytest

import lab
from lab.oracle_match import (
    max_abs_relative_deviation,
    nrmse,
    within_band,
)

# A small synthetic "reference trajectory" with a realistic crop shape: rise to a
# peak then fall back toward zero (like LAI over a season).
_REFERENCE = [0.0, 0.5, 1.5, 3.0, 4.0, 3.2, 1.8, 0.6, 0.0]


# --- the measurement is correct on known inputs ------------------------------
def test_nrmse_zero_for_identical_series() -> None:
    assert nrmse(_REFERENCE, _REFERENCE) == 0.0


def test_nrmse_known_value() -> None:
    # Constant offset of 0.4 everywhere over a reference of range 4.0 (max 4.0, min
    # 0.0): RMSE = 0.4, nrmse = 0.4 / 4.0 = 0.1.
    candidate = [r + 0.4 for r in _REFERENCE]
    assert nrmse(_REFERENCE, candidate) == pytest.approx(0.1)


def test_max_abs_relative_deviation_picks_the_worst_day() -> None:
    candidate = list(_REFERENCE)
    candidate[3] = _REFERENCE[3] * 1.5  # one day 50% high (3.0 -> 4.5)
    dev = max_abs_relative_deviation(_REFERENCE, candidate, floor=0.1)
    assert dev == pytest.approx(0.5)


# --- the discriminating control: the band actually bites ---------------------
# Per the project norm (purity/`fit_order` controls): a matcher that cannot reject is
# worthless. Prove within_band accepts a within-band candidate AND rejects an
# out-of-band one.
def test_within_band_accepts_small_perturbation() -> None:
    candidate = [r * 1.03 for r in _REFERENCE]  # 3% scaling — within a 5% band
    assert within_band(_REFERENCE, candidate, tol=0.05) is True


def test_within_band_rejects_large_perturbation() -> None:
    candidate = [r * 1.5 for r in _REFERENCE]  # 50% scaling — out of a 5% band
    assert within_band(_REFERENCE, candidate, tol=0.05) is False


# --- guards fail loud --------------------------------------------------------
def test_length_mismatch_raises() -> None:
    with pytest.raises(ValueError, match="same length"):
        nrmse(_REFERENCE, _REFERENCE[:-1])


def test_empty_raises() -> None:
    with pytest.raises(ValueError, match="at least one"):
        nrmse([], [])


def test_non_finite_raises() -> None:
    with pytest.raises(ValueError, match="non-finite"):
        nrmse([1.0, 2.0], [1.0, float("nan")])


def test_flat_reference_raises() -> None:
    with pytest.raises(ValueError, match="range is zero"):
        nrmse([2.0, 2.0, 2.0], [2.0, 2.1, 1.9])


def test_non_positive_floor_raises() -> None:
    with pytest.raises(ValueError, match="floor must be strictly positive"):
        max_abs_relative_deviation([1.0], [1.0], floor=0.0)


def test_negative_tol_raises() -> None:
    with pytest.raises(ValueError, match="tol must be non-negative"):
        within_band([1.0, 2.0], [1.0, 2.0], tol=-0.1)


# --- discipline: `lab` ships in the wheel, so it must stay PCSE-free ----------
# The AST purity gate only scans `simcore`; `lab`'s cleanliness is by discipline.
# PCSE (EUPL) must never enter a shipped `src/` package — keep it in tests/oracle/.
def test_lab_imports_no_pcse() -> None:
    lab_dir = Path(lab.__file__).parent
    lab_files = sorted(lab_dir.rglob("*.py"))
    assert lab_files, "no lab source files discovered — the scan is vacuous"
    for path in lab_files:
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            names: list[str] = []
            if isinstance(node, ast.Import):
                names = [a.name.split(".", 1)[0] for a in node.names]
            elif isinstance(node, ast.ImportFrom) and node.module and not node.level:
                names = [node.module.split(".", 1)[0]]
            assert "pcse" not in names, (
                f"{path.name} imports pcse — the EUPL oracle must stay in "
                "tests/oracle/, never a shipped src/ package"
            )
