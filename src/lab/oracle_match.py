"""Behavioral trajectory matching: is a candidate series within a band of a reference?

The measurement helper for the Phase-1 PCSE/WOFOST oracle gate (P5). Our crop model
is reimplemented clean-room from primary literature with independently-sourced
parameters, so it **cannot** reproduce the oracle bit-for-bit (that is what
determinism invariant #7 governs for *our* engine, not for a third-party reference).
The validation is therefore **behavioral** — trajectory *shape and magnitude within a
tolerance band* — exactly analogous to how ``lab/convergence.py:fit_order`` *measures*
an order instead of asserting an exact value.

This module is a **measurement**, not a pass/fail policy: it returns discrepancy
numbers; the calling test picks the variables and the band (the Phase-1 gate is wired
at Step 11). It is pure stdlib and PCSE-free by design — it compares two in-memory
series, so it (and its tests) run with **zero oracle dependency**; PCSE is needed only
to *regenerate* the reference fixture (``tests/oracle/runner.py``).

Out-of-core ``lab`` tooling (analysis, not engine): plain float arithmetic, none of
the core's determinism guarantees.
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
