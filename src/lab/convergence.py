"""Convergence-order measurement: fit the order ``p`` in ``error ≈ C·dt**p``.

The reusable harness for the Phase-0.5 convergence / timestep-sensitivity study
(Step 1), reused unchanged by the RK45-referenced (Step 2) and multi-rate (Step 3)
order checks. It generalizes the ad-hoc halving-ratio assertions already in
``tests/test_integrator`` (decay order) and ``tests/test_oscillator`` (V-drift order)
into one systematic estimator: run a scheme over a geometric ``dt`` ladder, then fit
the observed order from the error-vs-``dt`` curve.

Out-of-core ``lab`` tooling (analysis, not engine), so it carries none of the core's
determinism guarantees — a least-squares fit is plain float arithmetic. Pure stdlib.
"""

import math
from collections.abc import Callable, Sequence


def fit_order(dts: Sequence[float], errors: Sequence[float]) -> float:
    """Observed convergence order ``p`` from ``error ≈ C·dt**p``.

    The least-squares slope of ``log(error)`` vs ``log(dt)`` — it uses *every* rung
    of the ladder (more robust than a single halving ratio, which leans on one pair).

    Requires at least two rungs, strictly positive and not-all-equal ``dt``, and
    strictly positive ``error``. A zero (or negative) error means the rung has hit
    the floating-point round-off floor rather than measuring truncation error — keep
    the ``dt`` ladder coarse enough to stay above it, or the fitted order is noise.
    """
    if len(dts) != len(errors):
        raise ValueError(
            f"dts and errors must be the same length ({len(dts)} != {len(errors)})"
        )
    if len(dts) < 2:
        raise ValueError("need at least two (dt, error) rungs to fit an order")
    if any(d <= 0.0 for d in dts):
        raise ValueError(f"dts must be strictly positive, got {list(dts)!r}")
    if any(e <= 0.0 for e in errors):
        raise ValueError(
            "errors must be strictly positive; a non-positive error is the round-off "
            f"floor, not a truncation measurement (got {list(errors)!r}) — use a "
            "coarser dt ladder"
        )
    xs = [math.log(d) for d in dts]
    ys = [math.log(e) for e in errors]
    n = len(xs)
    sx = sum(xs)
    sy = sum(ys)
    sxx = sum(x * x for x in xs)
    sxy = sum(x * y for x, y in zip(xs, ys, strict=True))
    denom = n * sxx - sx * sx
    if denom == 0.0:
        raise ValueError("dt ladder is degenerate (all dts equal); cannot fit a slope")
    return (n * sxy - sx * sy) / denom


def convergence_order(
    error_of_dt: Callable[[float], float], dts: Sequence[float]
) -> float:
    """``fit_order`` over the errors ``error_of_dt(dt)`` produces for each ``dt``.

    The convenience the per-scheme convergence tests call: pass a function that runs
    the scheme to its endpoint error at a given ``dt`` and the ``dt`` ladder; get back
    the observed order.
    """
    return fit_order(list(dts), [error_of_dt(dt) for dt in dts])
