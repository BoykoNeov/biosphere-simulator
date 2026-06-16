"""Integrator strategies: Euler and RK4 (step 6).

Realizes the Frozen-API ``Integrator`` Protocol plus step-algorithm steps 1–2 and
4–5: take an immutable snapshot, evaluate every flow against it, reduce legs
per-stock in canonical order, combine the scheme's evaluations, and **apply once**
(``n -> n+1``). The arbitration backstop (step-alg #3) and extinction/conservation
(steps #6–7) are deferred to steps 7–8; this module is the stepping spine with a
clean seam for them.

**Increment-form contract (load-bearing).** ``Flow.evaluate(snapshot, env, dt)``
returns legs that are the *per-step increment* ``dt·rate(snapshot)`` — not a bare
rate (the "amount per dt" contract). With ``f(y)`` the per-stock delta map from
one evaluation:

  * Euler:  ``y_{n+1} = y_n + f(y_n)``                              (one evaluation)
  * RK4:    ``k1=f(y_n)``, ``k2=f(y_n+½k1)``, ``k3=f(y_n+½k2)``,
            ``k4=f(y_n+k3)``, ``y_{n+1}=y_n+(k1+2k2+2k3+k4)/6``     (four)

Because every ``k_i`` already carries ``dt``, the ⅙-combine reproduces classical
``y+dt/6(k1'+2k2'+2k3'+k4')`` exactly. This identity holds **only if** ``rate`` is
independent of ``dt`` (linear in ``dt``); a flow that uses ``dt`` non-linearly
still conserves mass but silently forfeits RK4 order — see the ``Flow.evaluate``
note in ``simcore.flow`` and the step-6 dt-linearity test.

**Seam (#16).** ``_derivative`` binds the resolver to the **same** stage-state it
passes to ``flow.evaluate``, so a flow's direct snapshot reads and its
``env.get`` shared-stock reads stay consistent. RK4 stage states keep the step's
integer ``n`` (only amounts are perturbed), so forcing is piecewise-constant
within a step — exact for the autonomous Phase-0 oscillator (step-5 note).

**Positivity.** Intermediate RK4 stage amounts may go negative — allowed (``Stock``
forbids only NaN/Inf). Positivity under RK4 must come from the kinetics, not a
guard (the integrator contract); the Euler-only min-scaling backstop arrives in
step 7.

Pure stdlib only.
"""

from collections.abc import Mapping
from dataclasses import replace
from typing import Protocol, runtime_checkable

from simcore.environment import SourceResolver
from simcore.ids import StockId
from simcore.registry import Registry
from simcore.state import State, Stock


@runtime_checkable
class Integrator(Protocol):
    """Owns stepping: ``state -> state`` over one ``dt`` (decision: strategy).

    ``env`` is the **binding source** (``SourceResolver``), not a pre-bound
    ``Environment``: the integrator must rebind it per derivative evaluation (RK4
    binds per stage), which only the resolver can do. This reconciles the frozen
    ``step(state, env, dt)`` line, written before the step-5 resolver existed — see
    the step-6 design in the plan. The ``Registry`` is a *construction* dependency
    (model structure), while ``env`` stays a per-step argument (per-run scenario
    wiring).
    """

    def step(self, state: State, env: SourceResolver, dt: float) -> State: ...


def _derivative(
    registry: Registry, stage_state: State, env: SourceResolver, dt: float
) -> dict[StockId, float]:
    """Per-stock delta map for one derivative evaluation (step-alg 1–2, 4).

    Binds ``env`` to ``stage_state`` (the **same** object handed to
    ``flow.evaluate`` — the #16 seam) and evaluates every flow in the registry's
    canonical id-order, summing legs per-stock in that same order (#15). The legs
    are per-step increments ``dt·rate`` (the increment-form contract), so the
    returned map *is* the Euler increment / one RK4 stage's ``k``.
    """
    bound = env.bind(stage_state, dt)
    deltas: dict[StockId, float] = {}
    for flow in registry.flows:  # canonical id-sorted (#15)
        result = flow.evaluate(stage_state, bound, dt)
        for leg in result.legs:
            deltas[leg.stock] = deltas.get(leg.stock, 0.0) + leg.amount
    return deltas


def _shifted_stocks(
    state: State, deltas: Mapping[StockId, float], factor: float
) -> dict[StockId, Stock]:
    """``state.stocks`` with each named stock's amount shifted by ``factor*delta``.

    Owns **referential integrity** (the apply path's job, step 5/6): a delta keyed
    by a stock absent from ``state.stocks`` raises ``KeyError`` at the first step,
    rather than silently dropping the transfer. Sorted iteration is for
    determinism of iteration order only — each stock's amount is an independent
    single addition, so the float results do not depend on it.
    """
    stocks = dict(state.stocks)
    for sid in sorted(deltas):
        stock = stocks.get(sid)
        if stock is None:
            raise KeyError(
                f"flow produced a leg on unknown stock {sid!r}; referential "
                "integrity is checked in the integrator apply path (step 5/6)"
            )
        stocks[sid] = replace(stock, amount=stock.amount + factor * deltas[sid])
    return stocks


def _perturb(state: State, deltas: Mapping[StockId, float], factor: float) -> State:
    """An RK4 stage state: amounts shifted by ``factor*deltas``, keeping ``n``.

    Keeping ``n`` makes forcing piecewise-constant within the step (#14/#16);
    amounts may transiently go negative (allowed — positivity is the kinetics'
    job under RK4).
    """
    return replace(state, stocks=_shifted_stocks(state, deltas, factor))


def _apply(state: State, deltas: Mapping[StockId, float]) -> State:
    """Write the step result: amounts shifted by ``deltas`` and ``n -> n+1``."""
    return replace(state, n=state.n + 1, stocks=_shifted_stocks(state, deltas, 1.0))


def _combine(
    k1: Mapping[StockId, float],
    k2: Mapping[StockId, float],
    k3: Mapping[StockId, float],
    k4: Mapping[StockId, float],
) -> dict[StockId, float]:
    """RK4 ⅙-weighted combine over the **union** of stage keys (missing ⇒ 0).

    Iterating only one stage's keys would silently drop a stock that a state-gated
    flow touched at a perturbed stage but not at ``y_n``. Sorted for deterministic
    iteration (the float result is per-stock independent).
    """
    keys = set(k1) | set(k2) | set(k3) | set(k4)
    return {
        s: (
            k1.get(s, 0.0)
            + 2.0 * k2.get(s, 0.0)
            + 2.0 * k3.get(s, 0.0)
            + k4.get(s, 0.0)
        )
        / 6.0
        for s in sorted(keys)
    }


class EulerIntegrator:
    """Explicit Euler: one derivative evaluation per step.

    The ``Registry`` is injected at construction (model structure); the resolver
    is passed per ``step`` (per-run wiring). Step 7 will insert the Euler-only
    min-scaling backstop into this path.
    """

    def __init__(self, registry: Registry) -> None:
        self._registry = registry

    @property
    def registry(self) -> Registry:
        """The flow registry this integrator steps."""
        return self._registry

    def step(self, state: State, env: SourceResolver, dt: float) -> State:
        k1 = _derivative(self._registry, state, env, dt)
        return _apply(state, k1)


class Rk4Integrator:
    """Classical 4th-order Runge–Kutta in increment form (four evaluations/step).

    Stage states keep the step's ``n`` (only amounts perturbed). Step 7 will make
    a needed ``scale_f < 1`` a **hard error** here (positivity under RK4 comes from
    kinetics, not the Euler-only backstop).
    """

    def __init__(self, registry: Registry) -> None:
        self._registry = registry

    @property
    def registry(self) -> Registry:
        """The flow registry this integrator steps."""
        return self._registry

    def step(self, state: State, env: SourceResolver, dt: float) -> State:
        reg = self._registry
        k1 = _derivative(reg, state, env, dt)
        k2 = _derivative(reg, _perturb(state, k1, 0.5), env, dt)
        k3 = _derivative(reg, _perturb(state, k2, 0.5), env, dt)
        k4 = _derivative(reg, _perturb(state, k3, 1.0), env, dt)
        return _apply(state, _combine(k1, k2, k3, k4))
