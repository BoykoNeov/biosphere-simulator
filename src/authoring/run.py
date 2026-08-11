"""Run an interpreted scenario — the single-rate harness, and the multi-rate driver.

Mirrors the standalone ``run_crew`` / ``run_power`` / ``run_eclss`` drivers: step
the requested integrator ``steps`` times, returning the full trajectory (including
the initial state), the summed arbitration-backstop firings, and any extinction
events. **No reset hook** — phenology/annual-reset stays a station-driver concern.

The every-step conservation gate runs inside ``integrator.step_report`` (and, on the
multi-rate path, once per master step inside ``multirate_step``), so a completed run
is itself proof the authored graph balanced every step — an authored mis-wiring
surfaces here as a ``ConservationError`` (the safety property).

**Rationing is a hard error here (post-roadmap; see ``RationedError``).** Conservation
is *not* sufficient: the ``dt`` hazard produces a run that balances every step and
still asphyxiates the cabin, because the backstop clamps the over-draw at zero rather
than going negative. So this harness gates on ``total_rationed == 0`` too — the second
half of "the authored graph is sound".

**Two paths, and the branch is load-bearing (multi-rate Step 3).** A scenario that
declared a coupling cadence (``BuiltScenario.is_multirate``) is driven by
``simcore.multirate.multirate_step``; **everything else takes the pre-multi-rate loop
over the whole ``registry``, verbatim**. That is not an optimization. ``n_sub=1`` with
an empty slow set reproduces the single-rate trajectory bit-for-bit (measured, through
this layer, in ``tests/test_authoring_multirate_identity.py``) — so routing every
scenario through the driver *would* work, and would silently make all 25 goldens
rest on that identity holding forever. Keeping the untouched path costs one branch and
buys the golden-preservation guarantee by construction instead of by measurement.
Pinned by ``test_a_single_rate_scenario_never_touches_the_driver``.
"""

from __future__ import annotations

from authoring.errors import AuthoringError, RationedError, ReversedFlowError
from authoring.flow_registry import FLOW_TYPES
from authoring.interpreter import BuiltScenario
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator, Substepper
from simcore.multirate import Split, multirate_step
from simcore.state import State

_INTEGRATORS = {"euler": EulerIntegrator, "rk4": Rk4Integrator}

_SPLIT = Split.STRANG
"""The operator-splitting scheme — pinned, deliberately **not** author-visible.

Strang is the core's own default and carries the higher nominal order. The
justification is order/safety rather than performance: Lie is actually *cheaper* on the
slow set (one slow evaluation per master step against Strang's two), but Strang steps
the slow set at ``dt/2``, which is safer for the slow set's own ``k·dt``. ``simcore``
documents Lie as "fallback / comparison" — a **study** tool — and our frozen flows are
Euler, which collapses Strang back to 1st order anyway, so exposing the knob would buy
an author no order they could use. **Deferred by name:** author-visible ``split``.
"""


def _run_single_rate(
    built: BuiltScenario, integrator_cls: type[EulerIntegrator] | type[Rk4Integrator]
) -> tuple[list[State], int, list[Event]]:
    """The pre-multi-rate loop, unchanged: ``step_report`` over the whole registry."""
    integrator = integrator_cls(built.registry)
    state = built.state
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _ in range(built.steps):
        report = integrator.step_report(state, built.resolver, built.dt)
        state = report.state
        states.append(state)
        total_rationed += report.rationed
        events.extend(report.events)
    return states, total_rationed, events


def _run_multirate(
    built: BuiltScenario, integrator_cls: type[EulerIntegrator] | type[Rk4Integrator]
) -> tuple[list[State], int, list[Event]]:
    """Drive ``multirate_step`` once per master step over the interpreter's partition.

    Both sub-integrators are built from the scenario's **single** ``integrator`` —
    ``multirate_step`` would accept ``slow=rk4, fast=euler``, but a per-rate-class
    integrator is **deferred by name**. ``multirate_step`` aggregates ``rationed``
    across its sub-operations by contract, so the master-step summing below is the same
    arithmetic as the single-rate path's.
    """
    slow: Substepper = integrator_cls(built.slow_registry)
    fast: Substepper = integrator_cls(built.fast_registry)
    state = built.state
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _ in range(built.steps):
        report = multirate_step(
            slow, fast, state, built.resolver, built.dt, built.n_sub, split=_SPLIT
        )
        state = report.state
        states.append(state)
        total_rationed += report.rationed
        events.extend(report.events)
    return states, total_rationed, events


def _check_no_aux(built: BuiltScenario) -> None:
    """Refuse an aux-bearing graph on the multi-rate path (the P2 tripwire).

    ``step_report`` advances ``State.aux``; ``multirate_step`` deliberately **never**
    does (``simcore`` decision P2, *"Aux × multi-rate is out of scope"*). Routing an
    aux-bearing graph through the driver would therefore freeze every accumulator
    **silently** — no error, and the conservation gate cannot see it, because aux is
    non-conserved by definition. A number that simply stops moving.

    This **cannot fire from an authored file today**: ``interpret`` calls
    ``Registry(flows, stocks)`` and never wires ``aux_processes``, which is exactly the
    unstated precondition the ``n_sub=1`` identity rests on. It is a tripwire for the
    phase that makes the biosphere — the one aux-bearing domain, deferred from the flow
    registry for this family of reasons — authorable.

    **The guard lives here rather than in ``multirate_step``** because ``simcore`` is
    frozen and this is a consumer phase (``git diff src/simcore/`` must come back
    empty); and it is scoped to the multi-rate branch because single-rate
    ``step_report`` handles aux correctly, so refusing it there would ban a working
    shape. An ``AuthoringError`` despite firing at run time rather than interpret time:
    it is decidable from the graph's structure alone (that is what makes it this class
    rather than ``RationedError``'s state-dependent verdict), and it is raised before
    any step runs.
    """
    if built.registry.aux_processes:
        names = sorted(str(proc.id) for proc in built.registry.aux_processes)
        raise AuthoringError(
            f"scenario {built.name!r} declares a multi-rate cadence (n_sub="
            f"{built.n_sub}) but its graph carries aux process(es) {names}. "
            f"simcore.multirate.multirate_step never advances State.aux (decision P2, "
            f"'Aux x multi-rate is out of scope'), so running this would freeze every "
            f"accumulator silently — aux is non-conserved, so the conservation gate "
            f"cannot see it. Run this scenario single-rate (drop n_sub and any "
            f"'rate_class: slow'), where step_report advances aux correctly."
        )


def _rationed_message(built: BuiltScenario, total_rationed: int) -> str:
    """The ``RationedError`` text — conditional on which path produced the firing.

    "Increase ``n_sub``" is the honest advice on the multi-rate path and *wrong* on the
    single-rate one, where there is no such knob to raise: naming it would send an
    author looking for a key their scenario does not have. The multi-rate variant also
    reports the **effective sub-step**, since the master ``dt`` is no longer the step
    any flow was actually integrated at — quoting it would name the one number that is
    not the cause.
    """
    if built.is_multirate:
        where = (
            f"at an effective sub-step of dt/n_sub = {built.dt / built.n_sub!r} "
            f"(master dt={built.dt!r}, n_sub={built.n_sub}) over {built.steps} "
            f"master step(s)"
        )
        remedy = (
            "Increase n_sub (the fast set then sub-steps more finely, leaving the "
            "master export cadence untouched) or reduce dt. Note the slow set steps at "
            "dt/2 regardless of n_sub under Strang splitting, so if a 'rate_class: "
            "slow' flow is the one over-drawing, raising n_sub will not help it — "
            "reduce dt, or re-class that flow fast"
        )
    else:
        where = f"at dt={built.dt!r} over {built.steps} step(s)"
        remedy = "Reduce dt and re-run"
    return (
        f"the arbitration backstop fired {total_rationed} time(s) {where}. On an "
        f"authored graph this means the step is too large for some flow's frozen rate "
        f"constant: the over-draw was clamped at zero, so the run still conserved "
        f"every quantity and still finished — but a clamped stock is an emptied one "
        f"(a cabin with no oxygen conserves mass perfectly). {remedy}; each flow "
        f"type's constraint is tabulated under 'The dt constraint' in "
        f"docs/authoring-reference.md (e.g. ECLSS's frozen rates want dt <= ~60 s). "
        f"To inspect the rationed run instead of failing, pass allow_rationing=True."
    )


def _demand_controlled_by_class() -> dict[type, tuple[str, str]]:
    """Flow class -> ``(regulated wiring field, setpoint param)``, from the registry.

    Derived from :data:`FLOW_TYPES` rather than listed here, so registering a new
    demand-controlled type is what arms the check for it — the duplication this module
    would otherwise own is exactly the "declared copy drifted from the constructor"
    failure ``test_authoring_frozen_flows.py`` was written for.
    """
    return {
        spec.cls: spec.demand_controlled  # type: ignore[misc]
        for spec in FLOW_TYPES.values()
        if spec.demand_controlled is not None
    }


def _check_no_reversal(built: BuiltScenario, states: list[State]) -> None:
    """Refuse a run in which a demand-controlled flow pointed backwards.

    A demand-controlled magnitude is ``k·(setpoint − stock)``, so its **sign** is
    decided by which side of the setpoint the stock is on. Scanning the trajectory for
    ``stock > setpoint`` is therefore exactly equivalent to scanning for a negative
    magnitude, and needs no per-flow instrumentation — which matters because
    ``simcore.integrator.StepReport`` is frozen and reports no per-flow legs.

    ⚠ **What this cannot see, stated rather than implied: a sub-step-only excursion on
    the multi-rate path.** ``states`` holds one entry per **master** step; a fast-set
    sub-step that crosses the setpoint and comes back within the same master step is
    invisible here. On the single-rate path there are no sub-steps, so the scan is
    complete. The gap is not closable from this layer — seeing sub-steps means changing
    ``multirate_step``, and ``simcore`` is frozen (``git diff src/simcore/`` must come
    back empty). It is also not the shape an author hits: the crossing that motivated
    this gate is a **wiring** error, which is present at ``states[0]``.

    The other route to a crossing is **overshoot past the stability bound**: the update
    map is ``o2 → (1 − k·dt)·o2 + k·dt·setpoint``, so any ``k·dt ≥ 1`` alternates about
    the setpoint (converging below 2, diverging above it) and spends alternate steps
    *above* it. Multi-rate Step 5's build-time ``k·h < 1`` precondition refuses that
    whole family before any step runs, so an author reaches it only through
    ``allow_unsafe_step=True``.

    ⚠ **An earlier draft of this docstring named ``1 ≤ k·dt < 2`` as "the one route",
    which is a subset asserted of the set** — the divergent half is just as reachable,
    and four committed study tests hit it at ``k_makeup·dt = 7.2`` (they now pass
    ``allow_reversal=True``). Named because this whole piece of work is about that
    failure shape; see ``docs/plans/post-roadmap-o2-makeup-reversal.md``.

    The initial state is included in the scan on purpose: the common error is wiring a
    stock above the setpoint, which is wrong at ``t = 0`` and should not need a step to
    be reported.
    """
    regulated = _demand_controlled_by_class()
    if not regulated:  # pragma: no cover - defensive; the registry has one today
        return
    for flow in built.registry.flows:
        pair = regulated.get(type(flow))
        if pair is None:
            continue
        stock_field, setpoint_param = pair
        # `params` is not on the `Flow` protocol (it is a per-dataclass field), and a
        # param-free flow cannot be demand-controlled — a setpoint has to live
        # somewhere. Read defensively rather than asserting the registry and the
        # constructor agree; `test_authoring_frozen_flows.py` owns that agreement.
        params = getattr(flow, "params", None)
        if params is None:  # pragma: no cover - unreachable via FLOW_TYPES today
            continue
        stock_id = getattr(flow, stock_field)
        setpoint = getattr(params, setpoint_param)
        for n, state in enumerate(states):
            amount = state.stocks[stock_id].amount
            if amount > setpoint:
                raise ReversedFlowError(
                    _reversal_message(built, flow, stock_id, n, amount, setpoint)
                )


def _reversal_message(
    built: BuiltScenario,
    flow: Flow,
    stock_id: StockId,
    step: int,
    amount: float,
    setpoint: float,
) -> str:
    """The ``ReversedFlowError`` text — names the step, because *when* is the diagnosis.

    A crossing at step 0 is a wiring error (the author put the stock above the
    setpoint); a crossing later means something else is filling it faster than the
    regulator's target allows. Those are different fixes, and the step number is what
    distinguishes them, so it is never omitted.
    """
    when = (
        "at the INITIAL state, so this is a wiring error: the scenario file starts "
        "this stock above the setpoint"
        if step == 0
        else f"first at step {step} of {built.steps}, so some other flow in the "
        f"graph is filling this stock past the regulator's target"
    )
    return (
        f"flow {flow.id!s} is demand-controlled toward {setpoint!r}, and "
        f"{stock_id!s} reached {amount!r} — above it, {when}. A demand-controlled "
        f"magnitude is k*(setpoint - stock), so above the setpoint it goes NEGATIVE "
        f"and the flow runs backwards: it drains the stock it is named for, back "
        f"into its source. Nothing else in this stack objects — the two legs share "
        f"one magnitude so the run still conserves exactly, and the draw is "
        f"proportional to the setpoint ERROR rather than to the stock, so it never "
        f"over-draws and the arbitration backstop never fires (rationed stays 0). "
        f"Wire the stock at or below the setpoint, or drop the regulator and let the "
        f"stock float. To inspect the reversed run instead of failing, pass "
        f"allow_reversal=True."
    )


def run_scenario(
    built: BuiltScenario,
    *,
    allow_rationing: bool = False,
    allow_reversal: bool = False,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``built`` to completion; return ``(states, total_rationed, events)``.

    ``states`` has length ``steps + 1`` (initial + one per **master** step — sub-steps
    are internal and never commit, so a multi-rate run's trajectory has exactly the same
    shape as a single-rate one at the same ``steps``). An unknown integrator name is an
    ``AuthoringError``.

    A scenario that declared a coupling cadence is driven by ``multirate_step``; every
    other scenario takes the pre-multi-rate path unchanged (see the module docstring —
    the branch is the golden-preservation guarantee, not an optimization).

    **Raises ``RationedError`` if the Euler backstop fired at all** (see that class:
    on an authored graph, rationing means the step is wrong, and the failure is
    otherwise silent). Pass ``allow_rationing=True`` to opt back in to the old
    return-and-trust-the-caller behavior — for deliberately studying a rationed run
    (``tests/test_authoring_dt_hazard.py``), not for making a scenario "work".

    **Raises ``ReversedFlowError`` if a demand-controlled flow pointed backwards** —
    a *direction* defect the rationing gate structurally cannot see, because a
    demand-controlled draw never over-draws. ``allow_reversal=True`` opts out, for
    studying the reversal rather than fixing it
    (``tests/test_authoring_export_fidelity.py``). The two gates are checked in that
    order deliberately: a rationed run's trajectory is already suspect, so reporting the
    ``dt`` fault first avoids blaming a wiring the backstop's own clamping produced.

    Multi-rate does **not** make a coarse ``dt`` safe: it splits the master step into
    ``dt/n_sub``, so too small an ``n_sub`` is the identical hazard one level down
    (measured: ``n_sub=2`` at ``dt=3600`` gives 36.0 against a truth of 8.0). The
    build-time ``k·(dt/n_sub) < 1`` precondition is a later step; until it lands this
    ``RationedError`` is the (donor-controlled-only) run-time catch.
    """
    integrator_cls = _INTEGRATORS.get(built.integrator)
    if integrator_cls is None:
        raise AuthoringError(
            f"unknown integrator {built.integrator!r} (known: {sorted(_INTEGRATORS)})"
        )
    if built.is_multirate:
        _check_no_aux(built)
        states, total_rationed, events = _run_multirate(built, integrator_cls)
    else:
        states, total_rationed, events = _run_single_rate(built, integrator_cls)
    if total_rationed > 0 and not allow_rationing:
        raise RationedError(_rationed_message(built, total_rationed))
    if not allow_reversal:
        _check_no_reversal(built, states)
    return states, total_rationed, tuple(events)
