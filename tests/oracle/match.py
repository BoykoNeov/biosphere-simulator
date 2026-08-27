"""Fixture-comparison helpers for the PCSE oracle carve-out.

⚠ MOVED HERE from `src/lab/oracle_match.py` by slice S6 (2026-08-27), unchanged.

`CLAUDE.md` says the surviving Python is *"`tests/oracle/` and its committed JSON
fixtures"*, and it was not true: two of those tests imported `lab.oracle_match`, so the
carve-out reached into the tree S6 deletes. Found by RUNNING the collection after the
deletion, not by reading the sentence — the same shape as `config/units.py`'s recorded
trap (*"retiring them without giving it an executing caller"*), arriving from the other
side: here the caller survived and the callee was being deleted underneath it.

The carve-out is now self-contained, which is what makes that sentence in `CLAUDE.md`
checkable by `git ls-files` rather than by trust. `max_abs_relative_deviation` came with
it — it is `within_band`'s own implementation, not a separate export.
"""

import math
from collections.abc import Sequence


def _validate_pair(reference: Sequence[float], candidate: Sequence[float]) -> None:
    """Shared guards: equal length, non-empty, finite. Mirrors ``fit_order``'s
    up-front ``ValueError`` discipline (fail loud on a malformed comparison rather
    than return a meaningless number)."""
    if len(reference) != len(candidate):
        raise ValueError(
            "reference and candidate must be the same length "
            f"({len(reference)} != {len(candidate)}) — align the trajectories on "
            "their common days before comparing"
        )
    if len(reference) == 0:
        raise ValueError("need at least one aligned point to compare")
    for label, series in (("reference", reference), ("candidate", candidate)):
        if any(not math.isfinite(v) for v in series):
            raise ValueError(f"{label} contains a non-finite value (NaN/Inf)")


def nrmse(reference: Sequence[float], candidate: Sequence[float]) -> float:
    """Root-mean-square error normalized by the reference's range.

    ``sqrt(mean((candidate - reference)**2)) / (max(reference) - min(reference))``.

    Range-normalization (not mean-normalization) keeps the metric well-defined when a
    trajectory passes through zero — a crop's LAI starts and ends at 0, so dividing by
    the mean would be unstable, while the range (peak-to-trough) is a stable scale.
    It is scale-free, so the same tolerance band reads sensibly across variables of
    very different magnitudes (LAI ~ O(1) vs biomass ~ O(10^4) kg/ha).

    Raises if the reference is flat (zero range): there is no scale to normalize by,
    so a relative band is undefined — compare such a variable with an absolute metric
    instead.
    """
    _validate_pair(reference, candidate)
    span = max(reference) - min(reference)
    if span == 0.0:
        raise ValueError(
            "reference range is zero (flat series); nrmse has no scale to normalize "
            "by — use an absolute-tolerance comparison for a constant variable"
        )
    n = len(reference)
    sse = sum((c - r) ** 2 for r, c in zip(reference, candidate, strict=True))
    return math.sqrt(sse / n) / span


def max_abs_relative_deviation(
    reference: Sequence[float],
    candidate: Sequence[float],
    *,
    floor: float,
) -> float:
    """Largest pointwise ``|candidate - reference| / max(|reference|, floor)``.

    Pointwise relative error, robust near zero via an explicit ``floor`` (the scale
    below which a relative error is meaningless — e.g. LAI of 1e-6 should not blow the
    ratio up). ``floor`` must be strictly positive; choose it as the smallest
    physically-meaningful magnitude of the variable. Captures a single bad day that an
    aggregate ``nrmse`` could average away.
    """
    if floor <= 0.0:
        raise ValueError(f"floor must be strictly positive, got {floor!r}")
    _validate_pair(reference, candidate)
    return max(
        abs(c - r) / max(abs(r), floor)
        for r, c in zip(reference, candidate, strict=True)
    )


def within_band(
    reference: Sequence[float],
    candidate: Sequence[float],
    *,
    tol: float,
) -> bool:
    """Whether the candidate stays within a relative ``tol`` band of the reference,
    measured by :func:`nrmse`. The thin pass/fail convenience over the measurement;
    the Phase-1 gate (which variables, what ``tol``) is set by the calling test."""
    if tol < 0.0:
        raise ValueError(f"tol must be non-negative, got {tol!r}")
    return nrmse(reference, candidate) <= tol
