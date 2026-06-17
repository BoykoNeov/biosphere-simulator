"""Integrator strategies: Euler and RK4, with the step-7 backstop + extinction.

Realizes the Frozen-API ``Integrator`` Protocol and the full step algorithm:
take an immutable snapshot, evaluate every flow against it, **arbitrate**, reduce
legs per-stock in canonical order, combine the scheme's evaluations, **apply once**
(``n -> n+1``), run the **extinction pass**, then assert the **every-step
conservation gate** (``simcore.conservation`` — total mass per asserted quantity
unchanged across the step, decision #13 / step-alg #7).

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

**Arbitration asymmetry (step 7, the integrator contract).** The min-scaling
backstop (``simcore.arbitration``) is **Euler-only**: ``EulerIntegrator`` scales
the over-drawing flows and reports the firing count; ``Rk4Integrator`` instead
treats a needed ``scale_f < 1`` as a **hard error** (``ArbitrationError``), because
the single-evaluation conservation-safety proof does not carry to a weighted sum of
clamped stage derivatives — positivity under RK4 must come from the kinetics.

**Extinction (step 7, decision #6).** After apply, any POPULATION stock below its
``extinction_threshold`` (and not already exactly 0) snaps to 0 and its residual is
routed to the quantity's numerical-loss boundary sink so the ledger still balances;
an ``ExtinctionEvent`` is recorded. POOL/BOUNDARY stocks are never zeroed-with-loss.

**Diagnostics are functional, not mutable.** ``step_report`` returns a ``StepReport``
``(state, events, rationed)``; the Protocol-conforming ``step`` returns just the
``State``. The integrator keeps **no** mutable event log or counter — the core stays
pure and re-runnable, and a caller (a scenario/golden harness) accumulates its own
running totals (the rationing gate asserts its sum ``== 0`` on a well-fed run).

**Seam (#16).** ``_evaluate_all`` binds the resolver to the **same** stage-state it
passes to ``flow.evaluate``. RK4 stage states keep the step's integer ``n`` (only
amounts perturbed), so forcing is piecewise-constant within a step — exact for the
autonomous Phase-0 oscillator (step-5 note). Intermediate RK4 stage amounts may go
negative — allowed (``Stock`` forbids only NaN/Inf); positivity under RK4 is the
kinetics' job, not a guard.

**Aux channel (Phase-1 P2).** Alongside the conserved stocks the integrator advances
``State.aux`` — non-conserved scalar accumulators (thermal time, …) with no balanced
counterparty (``simcore.auxiliary``). Aux is advanced by **one explicit-Euler
evaluation at the step-entry snapshot** in ``step_report`` (``_aux_increments`` →
``_apply``),
*independent of the stock scheme*: RK4 stage states carry aux unchanged (only amounts
perturb, exactly like ``n``), so a flow reading aux sees a within-step constant, and
aux advances exactly once per full step. It is **outside** the conservation gate by
construction (``conservation.compute_ledger`` reasons only over ``stocks``). ``substep``
(the multi-rate primitive) leaves aux untouched — see its docstring.

Pure stdlib only.
"""

from collections.abc import Mapping, Sequence
from dataclasses import dataclass, replace
from typing import Protocol, runtime_checkable

from simcore import arbitration, conservation
from simcore.boundary import loss_sink_id
from simcore.environment import SourceResolver
from simcore.events import Event, ExtinctionEvent
from simcore.flow import FlowResult
from simcore.ids import StockId
from simcore.quantities import StockKind
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

    The frozen surface is ``step``. The concrete strategies also expose the richer
    ``step_report`` (``StepReport``) for callers that need the step's events /
    rationing-firing count; ``step`` is exactly ``step_report(...).state``.
    """

    def step(self, state: State, env: SourceResolver, dt: float) -> State: ...


@dataclass(frozen=True)
class StepReport:
    """One step's result plus its diagnostics (the functional side-channel).

    ``state`` is the produced ``State`` (``n -> n+1``). ``events`` are the discrete
    events the step emitted (Phase 0: extinctions), in canonical stock-id order.
    ``rationed`` is the number of **flows scaled** by the Euler backstop this step
    (one per flow with ``scale_f < 1``); it is always 0 for RK4, which hard-errors
    on an over-draw instead of scaling. A golden run sums ``rationed`` over steps and
    asserts ``== 0`` (a nonzero total means ``dt`` is too large or the kinetics are
    mis-scaled — a failing gate, not a warning).
    """

    state: State
    events: tuple[Event, ...]
    rationed: int


@runtime_checkable
class Substepper(Protocol):
    """A concrete integrator's *amounts-only* advance (the multi-rate building block).

    ``substep`` is like ``step_report`` but **keeps** ``State.n`` (it advances
    amounts and runs arbitration + extinction, but does **not** bump the integer
    clock and does **not** assert conservation). It is the primitive
    ``simcore.multirate`` composes — the driver owns the single ``n -> n+1`` commit
    and the composite conservation gate (decisions N2/N4/N5).

    This is deliberately **separate** from the frozen ``Integrator`` Protocol
    (Phase-0 surface): ``substep`` is an *additive* capability on the concrete
    strategies, not part of the frozen ``step`` contract. The concrete
    ``EulerIntegrator`` / ``Rk4Integrator`` satisfy **both** protocols.
    """

    def substep(self, state: State, env: SourceResolver, dt: float) -> StepReport: ...


def _evaluate_all(
    registry: Registry, stage_state: State, env: SourceResolver, dt: float
) -> list[FlowResult]:
    """Evaluate every flow against ``stage_state``, in canonical id-order (#15).

    Binds ``env`` to ``stage_state`` (the **same** object handed to
    ``flow.evaluate`` — the #16 seam). The returned list is in the registry's
    canonical flow-id order, which both the arbitration demand sum and the per-stock
    reduction depend on for determinism (#15).
    """
    bound = env.bind(stage_state, dt)
    return [flow.evaluate(stage_state, bound, dt) for flow in registry.flows]


def _reduce(results: Sequence[FlowResult]) -> dict[StockId, float]:
    """Per-stock delta map: sum legs over flows in canonical (input) order (#15)."""
    deltas: dict[StockId, float] = {}
    for result in results:
        for leg in result.legs:
            deltas[leg.stock] = deltas.get(leg.stock, 0.0) + leg.amount
    return deltas


def _aux_increments(
    registry: Registry, snapshot: State, env: SourceResolver, dt: float
) -> dict[str, float]:
    """Per-name aux increment: one Euler evaluation at ``snapshot`` (P2/P3).

    Binds ``env`` to ``snapshot`` (the #16 seam — the *same* snapshot aux reads) and
    evaluates every aux process in canonical ``AuxId`` order (the registry guarantees
    it), summing increments per accumulator name (sorted within a process). The
    cross-process per-name sum runs in AuxId order, so the result is bit-identical
    under aux-process registration shuffle (#15). One evaluation, at the step-entry
    snapshot, regardless of the stock scheme — aux is never sub-staged through RK4.
    Returns ``{}`` when there are no aux processes (the empty-default fast path).
    """
    increments: dict[str, float] = {}
    if not registry.aux_processes:
        return increments
    bound = env.bind(snapshot, dt)
    for proc in registry.aux_processes:
        result = proc.evaluate(snapshot, bound, dt)
        for name in sorted(result):
            increments[name] = increments.get(name, 0.0) + result[name]
    return increments


def _advanced_aux(state: State, increments: Mapping[str, float]) -> Mapping[str, float]:
    """``state.aux`` with each named accumulator advanced by its increment (P2).

    Increment-form, like stocks: ``new[name] = old.get(name, 0) + increment[name]``.
    Empty increments ⇒ ``state.aux`` is returned unchanged (the no-aux fast path that
    keeps the Phase-0/0.5 empty default and goldens intact). Sorted iteration is for
    deterministic ordering only — each name is one independent addition. The returned
    mapping is re-wrapped immutably by ``State.__post_init__`` on ``replace``.
    """
    if not increments:
        return state.aux
    merged = dict(state.aux)
    for name in sorted(increments):
        merged[name] = merged.get(name, 0.0) + increments[name]
    return merged


def _rk4_stage(
    registry: Registry, stage_state: State, env: SourceResolver, dt: float
) -> dict[StockId, float]:
    """One RK4 derivative evaluation: evaluate → hard-error guard → reduce.

    The min-scaling backstop is Euler-only; under RK4 a needed ``scale_f < 1`` is a
    hard error (``ArbitrationError``) rather than a silent clamp — positivity under
    a higher-order scheme must come from the kinetics (the integrator contract).
    """
    results = _evaluate_all(registry, stage_state, env, dt)
    arbitration.check_no_overdraw(results, stage_state.stocks)
    return _reduce(results)


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


def _apply(
    state: State,
    deltas: Mapping[StockId, float],
    aux_increments: Mapping[str, float],
) -> State:
    """Write the step result: amounts shifted by ``deltas``, aux advanced, ``n -> n+1``.

    Aux is advanced **here**, in the single ``n -> n+1`` commit reached only by
    ``step_report`` — never in ``_perturb`` (RK4 stages keep aux, like ``n``) and
    never in ``substep`` (the multi-rate primitive leaves aux untouched, P2). So aux
    advances exactly once per full step, by one Euler increment, independent of the
    stock scheme.
    """
    return replace(
        state,
        n=state.n + 1,
        stocks=_shifted_stocks(state, deltas, 1.0),
        aux=_advanced_aux(state, aux_increments),
    )


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


def _extinction_pass(state: State) -> tuple[State, tuple[ExtinctionEvent, ...]]:
    """Step-algorithm #6: snap below-threshold POPULATION stocks to 0 (decision #6).

    For each POPULATION stock with ``amount < extinction_threshold`` **and** amount
    not already exactly 0, set it to 0 and route the snapped residual into the
    quantity's numerical-loss boundary sink (``boundary.loss_sink_id``) so the
    per-quantity ledger still balances. An ``ExtinctionEvent`` is recorded. The
    ``amount != 0`` guard is what makes extinction absorbing without event-spam: an
    already-extinct stock (sitting at exactly 0 below a positive threshold) does not
    re-fire, and a sub-threshold inflow ("noise") is re-snapped rather than reviving
    the stock. POOL/BOUNDARY stocks are never zeroed-with-loss.

    Stocks are scanned and the loss-sink deposits applied in canonical (sorted) id
    order, so events and the (per-quantity) residual sums are deterministic (#15).
    A routed residual whose loss-sink is absent from ``State.stocks`` raises
    ``KeyError`` (referential integrity — the initial state must include the
    boundary loss-sinks).
    """
    events: list[ExtinctionEvent] = []
    snapped: dict[StockId, Stock] = {}
    loss_deltas: dict[StockId, float] = {}
    for sid in sorted(state.stocks):
        stock = state.stocks[sid]
        if stock.kind is not StockKind.POPULATION:
            continue
        if stock.amount < stock.extinction_threshold and stock.amount != 0.0:
            residual = stock.amount
            snapped[sid] = replace(stock, amount=0.0)
            ls_id = loss_sink_id(stock.quantity)
            loss_deltas[ls_id] = loss_deltas.get(ls_id, 0.0) + residual
            events.append(
                ExtinctionEvent(
                    n=state.n, stock=sid, quantity=stock.quantity, residual=residual
                )
            )
    if not snapped:
        return state, ()
    new_stocks = dict(state.stocks)
    new_stocks.update(snapped)
    for ls_id in sorted(loss_deltas):
        ls = new_stocks.get(ls_id)
        if ls is None:
            raise KeyError(
                f"extinction routes a residual to loss-sink {ls_id!r} but it is "
                "absent from State.stocks; the initial state must include the "
                "boundary loss-sinks (decision #6 / referential integrity)"
            )
        new_stocks[ls_id] = replace(ls, amount=ls.amount + loss_deltas[ls_id])
    return replace(state, stocks=new_stocks), tuple(events)


def _finalize(before: State, applied: State, rationed: int) -> StepReport:
    """Shared post-apply tail: extinction pass → conservation gate → ``StepReport``.

    The every-step conservation gate (step 8) lives here, in **one** place, so
    neither scheme can skip it: per asserted quantity the total mass across all
    stocks must be unchanged from ``before`` to the post-extinction state (decision
    #13 / step-alg #7). A violation raises ``ConservationError`` — it is an engine
    bug, not a recoverable condition. ``before`` is the snapshot the flows evaluated
    against; ``applied`` is the post-apply (pre-extinction) state.
    """
    nxt, events = _extinction_pass(applied)
    conservation.assert_conserved(before, nxt)
    return StepReport(state=nxt, events=events, rationed=rationed)


def _finalize_substep(advanced: State, rationed: int) -> StepReport:
    """Multi-rate sub-step tail: extinction pass only — **no** conservation gate.

    Mirrors ``_finalize`` but deliberately omits the conservation assert: under
    operator splitting (``simcore.multirate``) conservation is asserted **once**, at
    the composite master-step boundary, not per sub-operation (decisions N4/N5). The
    sub-step keeps ``State.n`` (``advanced`` is an amounts-only advance produced by
    ``_perturb(..., 1.0)``); the multi-rate driver owns the single ``n -> n+1``
    commit. Extinction still runs per sub-operation (as in single-rate), so a routed
    residual lands in the loss-sink within the sub-step and the composite gate then
    sees a balanced whole.
    """
    nxt, events = _extinction_pass(advanced)
    return StepReport(state=nxt, events=events, rationed=rationed)


class _BaseIntegrator:
    """Shared spine: registry injection, the ``step``→``step_report`` delegation, and
    the ``substep`` primitive (the multi-rate building block).

    Subclasses implement ``_deltas`` — the scheme's per-step combined delta map plus
    its rationing-firing count. Everything else is shared: ``step_report`` applies the
    deltas with ``n -> n+1`` and the full conservation gate; ``substep`` applies the
    **same** deltas amounts-only (keeping ``n``, no conservation gate) for
    ``simcore.multirate`` to compose. Because both consume the identical ``_deltas``,
    an all-fast ``n_sub == 1`` multi-rate step reproduces the single-rate ``step``
    exactly. ``step`` is the frozen-API surface returning just the produced ``State``.
    """

    def __init__(self, registry: Registry) -> None:
        self._registry = registry

    @property
    def registry(self) -> Registry:
        """The flow registry this integrator steps."""
        return self._registry

    def step(self, state: State, env: SourceResolver, dt: float) -> State:
        return self.step_report(state, env, dt).state

    def step_report(self, state: State, env: SourceResolver, dt: float) -> StepReport:
        """One full step: ``_deltas`` → apply with ``n -> n+1`` → extinction +
        the every-step conservation gate.

        Aux (the non-conserved channel, P2) is advanced **only here**: one Euler
        evaluation of every aux process at the step-entry ``state``
        (``_aux_increments``) is folded into the single ``n -> n+1`` commit by
        ``_apply``. ``substep`` shares the ``_deltas`` spine but deliberately does
        **not** advance aux (see ``substep``).
        """
        deltas, rationed = self._deltas(state, env, dt)
        aux_increments = _aux_increments(self._registry, state, env, dt)
        return _finalize(state, _apply(state, deltas, aux_increments), rationed)

    def substep(self, state: State, env: SourceResolver, dt: float) -> StepReport:
        """Amounts-only advance (keeps ``n``); the ``simcore.multirate`` primitive.

        Computes the **same** per-step deltas as ``step_report`` (via ``_deltas``)
        but applies them with ``_perturb(..., 1.0)`` — amounts shift, ``n`` is kept —
        and runs only the extinction pass (``_finalize_substep``), **not** the
        conservation gate. The multi-rate driver composes several sub-steps, commits
        the single ``n -> n+1``, and asserts conservation once at the composite
        boundary (decisions N2/N4/N5). Arbitration/extinction behave exactly as in
        single-rate (Euler scales + counts; RK4 hard-errors on over-draw).

        **Aux is deliberately untouched here** (P2). ``substep`` keeps ``n`` and only
        perturbs stock amounts, so ``State.aux`` passes through unchanged — advancing
        it in the shared sub-step path would advance it ``n_sub``× per master step
        (wrong). Aux × multi-rate is out of scope in Phase 1 (single-rate); the
        ``multirate_step`` driver therefore does not advance aux across a master step.
        Pinned by the placement test in ``tests/test_aux.py``.
        """
        deltas, rationed = self._deltas(state, env, dt)
        return _finalize_substep(_perturb(state, deltas, 1.0), rationed)

    def _deltas(
        self, state: State, env: SourceResolver, dt: float
    ) -> tuple[dict[StockId, float], int]:
        """The scheme's combined per-step delta map and its rationing-firing count.

        The one piece that differs between schemes; ``step_report``/``substep`` share
        everything else. Returns ``(deltas, rationed)`` where ``deltas`` is the
        canonical-order-reduced per-stock increment for this ``dt`` and ``rationed``
        is the Euler-backstop firing count (always 0 for RK4).
        """
        raise NotImplementedError


class EulerIntegrator(_BaseIntegrator):
    """Explicit Euler: one derivative evaluation per step, with the min-scaling
    backstop.

    The ``Registry`` is injected at construction (model structure); the resolver is
    passed per ``step`` (per-run wiring). The backstop scales over-drawing whole
    flows (conservation-safe by the single-evaluation proof) and the firing count is
    reported in the ``StepReport``.
    """

    def _deltas(
        self, state: State, env: SourceResolver, dt: float
    ) -> tuple[dict[StockId, float], int]:
        results = _evaluate_all(self._registry, state, env, dt)
        scaled, rationed = arbitration.min_scaling(results, state.stocks)
        return _reduce(scaled), rationed


class Rk4Integrator(_BaseIntegrator):
    """Classical 4th-order Runge–Kutta in increment form (four evaluations/step).

    Stage states keep the step's ``n`` (only amounts perturbed). A needed
    ``scale_f < 1`` at any stage is a **hard error** (``ArbitrationError``):
    positivity under RK4 comes from the kinetics, not the Euler-only backstop. So
    ``StepReport.rationed`` is always 0 here.
    """

    def _deltas(
        self, state: State, env: SourceResolver, dt: float
    ) -> tuple[dict[StockId, float], int]:
        reg = self._registry
        k1 = _rk4_stage(reg, state, env, dt)
        k2 = _rk4_stage(reg, _perturb(state, k1, 0.5), env, dt)
        k3 = _rk4_stage(reg, _perturb(state, k2, 0.5), env, dt)
        k4 = _rk4_stage(reg, _perturb(state, k3, 1.0), env, dt)
        return _combine(k1, k2, k3, k4), 0
