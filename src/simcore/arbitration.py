"""Arbitration backstop: single-pass min-scaling (decisions #3/#4/#15).

A *rare numerical guard*, not the ecological mechanism. Competition/throttling is
meant to emerge from saturating kinetics on shared stocks (Phase 1+); this backstop
only catches explicit-integration overshoot — a flow trying to withdraw more from a
stock than the stock holds at the start of the step.

**Algorithm (one deterministic pass).** Given the step's evaluated ``FlowResult``s
in **canonical flow-id order** (the order the registry yields), and the start-of-
step stock levels:

  * per stock ``s``: ``demand_s = Σ |withdrawals from s|`` over all flows (summed in
    the given canonical order, #15). Only *withdrawals* (legs with ``amount < 0``)
    count. ``unclamped`` BOUNDARY sources are **skipped** — a supply like solar is
    never throttled (decision #13), and skipping it also means it imposes no
    constraint on the flows that draw from it.
  * ``scale_s = min(1, available_s / demand_s)`` with ``available_s`` the start-of-
    step level (so withdrawals never draw against same-step inflows). ``demand_s ==
    0`` ⇒ ``scale_s = 1``.
  * per flow: ``scale_f = min(scale_s over the clamped stocks it withdraws from)``,
    defaulting to 1 (a flow that withdraws from nothing clamped is unthrottled).

**Conservation safety (Euler only).** Realized draw on ``s`` is
``Σ_f |w_fs|·scale_f ≤ Σ_f |w_fs|·scale_s = demand_s·scale_s ≤ available_s`` since
``scale_f ≤ scale_s`` for every flow touching ``s`` — so no clamped stock goes
negative. Scaling the **whole flow** by one factor preserves its internal
stoichiometry (``scale_f · Σlegs == 0``), hence every quantity's balance. This proof
is single-evaluation: under RK4 a step is a weighted sum of clamped stage
derivatives and positivity does **not** compose. Hence the asymmetry:

  * ``min_scaling`` (Euler): scale the offending whole flows, count the firings.
  * ``check_no_overdraw`` (RK4+): a needed ``scale_f < 1`` is a **hard error**
    (``ArbitrationError``) — positivity under higher-order schemes must come from
    the kinetics, not this guard.

Pure stdlib only.
"""

from collections.abc import Mapping, Sequence

from simcore.flow import FlowResult, Leg
from simcore.ids import StockId
from simcore.state import Stock


class ArbitrationError(Exception):
    """A flow needs ``scale_f < 1`` under a higher-order scheme (RK4+).

    Min-scaling is Euler-only (its conservation-safety proof is single-evaluation).
    Under RK4+, an over-draw is **not** silently clamped — positivity must come from
    the kinetics. A needed scale signals mis-scaled kinetics or too-large ``dt``,
    i.e. an engine/scenario bug, not a recoverable condition; hence a hard error
    with its own type (not ``ValueError``/``AssertionError``).
    """


def _scale_factors(
    results: Sequence[FlowResult], stocks: Mapping[StockId, Stock]
) -> list[float]:
    """Per-flow scale factors aligned with ``results`` (canonical-order demand sum).

    ``results`` MUST already be in canonical flow-id order (the registry's
    iteration order) — the per-stock ``demand_s`` sum is accumulated in that order
    so the float result is bit-identical under registration shuffle (#15).
    """
    # demand_s over clamped stocks, summed in the given canonical order (#15).
    demand: dict[StockId, float] = {}
    for result in results:
        for leg in result.legs:
            if leg.amount < 0.0 and not stocks[leg.stock].unclamped:
                demand[leg.stock] = demand.get(leg.stock, 0.0) - leg.amount
    # scale_s is per-stock and order-independent (each value is a self-contained
    # min of a single ratio), so dict iteration order here does not affect results.
    scale_s: dict[StockId, float] = {
        sid: (min(1.0, stocks[sid].amount / d) if d > 0.0 else 1.0)
        for sid, d in demand.items()
    }
    factors: list[float] = []
    for result in results:
        f = 1.0
        for leg in result.legs:
            if leg.amount < 0.0 and leg.stock in scale_s:
                f = min(f, scale_s[leg.stock])
        factors.append(f)
    return factors


def min_scaling(
    results: Sequence[FlowResult], stocks: Mapping[StockId, Stock]
) -> tuple[list[FlowResult], int]:
    """Euler backstop: scale the over-drawing whole flows; return (scaled, fired).

    ``fired`` counts the **flows scaled this step** (one per flow with
    ``scale_f < 1``) — the rationing-firing diagnostic the golden gate sums over
    steps and asserts ``== 0`` on a well-fed run. A flow with ``scale_f == 1`` is
    returned unchanged (the common path; ``x * 1.0 == x`` exactly, but reusing the
    object also avoids needless allocation). The returned list stays in the input's
    canonical order so the caller's per-stock reduction stays deterministic (#15).
    """
    factors = _scale_factors(results, stocks)
    scaled: list[FlowResult] = []
    fired = 0
    for result, f in zip(results, factors, strict=True):
        if f < 1.0:
            fired += 1
            legs = tuple(Leg(leg.stock, leg.amount * f) for leg in result.legs)
            scaled.append(FlowResult(legs=legs))
        else:
            scaled.append(result)
    return scaled, fired


def check_no_overdraw(
    results: Sequence[FlowResult], stocks: Mapping[StockId, Stock]
) -> None:
    """RK4+ backstop: raise ``ArbitrationError`` if any flow needs ``scale_f < 1``.

    The Euler-only min-scaling proof does not carry to higher-order schemes, so an
    over-draw here is a hard error rather than a silent clamp (the integrator
    contract). ``unclamped`` sources are skipped exactly as in ``min_scaling``, so a
    flow drawing only from a supply is never falsely flagged.
    """
    for i, f in enumerate(_scale_factors(results, stocks)):
        if f < 1.0:
            raise ArbitrationError(
                f"flow #{i} (canonical order) would over-draw a stock "
                f"(scale_f={f!r} < 1) under a higher-order scheme; min-scaling is "
                "Euler-only — positivity under RK4+ must come from the kinetics, "
                "not the backstop (too-large dt or mis-scaled kinetics)"
            )
