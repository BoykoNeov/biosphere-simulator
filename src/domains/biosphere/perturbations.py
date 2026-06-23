"""Perturbation harness (Phase-3 Step 6, P3.5) — forcing + flow interventions.

A perturbation is a **scenario-layer intervention composed onto the already-assembled
``(state, registry, resolver)``**, not a core change. Each shipped perturbation shows a
**cascade with no cascade code** while **conservation + ``rationed == 0``** hold through
it (P3.5). The harness composes *outside* ``build_season``, so the three existing
goldens and ``src/simcore/`` are untouched (``git diff src/simcore/`` stays empty); a
perturbation run gets **no golden** (it is a behavioural demonstration — Phase 4 owns
freeze-as-reference; the Step-5 "diagnostics, no golden" precedent).

Every perturbation reduces to a **pure function of the integer step ``n``** — the
legitimate forcing seam (#14: schedules take ``(n, dt)``, evaluated at ``t = n·dt``).
**Two seam-types, both already legitimate:**

1. **Forcing perturbation** (drought, lighting failure) — a pure schedule transform that
   wraps **one** forcing var's :data:`~simcore.environment.Schedule` with a windowed
   override and rebuilds the resolver with that var replaced (:func:`with_forcing` +
   :func:`window_override`). **No new stock / flow / sink; the ledger structure is
   untouched** (only a forcing *value* changes over a window), so conservation is
   structurally unaffected. :func:`with_drought` cuts irrigation to 0; the open field is
   the only scenario with irrigation to cut (the sealed chamber dropped it in Step 3 for
   genuine water closure). :func:`with_lighting_failure` cuts PAR to 0.

2. **Flow perturbation** (atmospheric leak) — :func:`with_atmospheric_leak` augments
   ``(state, registry)`` with a :class:`LeakFlow` + a boundary leak-sink stock, the
   leak's **timing gated by a windowed forcing var** read via ``env.get`` (so the
   calendar lives in a *schedule*, not the rate law — honouring the Step-4 "no calendar
   in a flow body" discipline; the flow itself is a pure first-order rate law). The leak
   var (:data:`LEAK_VAR`) is **local to this module, never added to the ``stocks.py``
   catalog** — baseline assembly must never see it, or the existing goldens are touched.

**Rejected — a ``Perturbation`` protocol / dataclass with
``.apply(state, registry, resolver)``.** Speculative generality for 2–3 perturbations
(the same critique that killed the Step-5 observe-hook): the three small builder
functions + the two shared helpers here are leaner and add no surface for a consumer
that does not exist. A protocol later is additive if a real multi-perturbation
composition need appears.

**Recoverable-regime constraint (a Step-6 probe finding).** Graceful cascades require a
window **within one year** and a magnitude bounded so the plant refills its grain
(``storage_c``) before the next annual reset. A *severe or permanent* perturbation
suppresses photosynthesis enough that the grain never refills, and at the year boundary
:func:`~domains.biosphere.season.annual_reset` **raises ``ValueError`` (seed bank too
small to re-sow)** — a driver crash, not a cascade, and arguably correct (a sealed
chamber whose plant cannot rebuild its seed bank genuinely cannot re-sow). The shipped
perturbations stay transient; the regime boundary is pinned by a characterization test,
not grown into scope.

Pure stdlib + ``simcore`` + ``domains`` (no builder imports another, P3.3).
"""

from dataclasses import dataclass, replace

from domains.biosphere.stocks import IRRIGATION_VAR, PAR_VAR
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.environment import (
    Environment,
    Schedule,
    SourceResolver,
    constant,
)
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.quantities import StockKind
from simcore.registry import Registry
from simcore.state import State, Stock

# The leak's windowed-activation forcing var (1.0 inside the window, 0.0 outside).
# **Local to this module** — never added to the ``stocks.py`` catalog, so the baseline
# season/resolver never carry it and the three existing goldens stay byte-identical.
LEAK_VAR = "atmospheric_leak"
# The boundary reservoir the leak vents into (boundary-domain, so the chamber's interior
# closure breaks but total mass — interior + this sink — is conserved). Also local.
LEAK_SINK: StockId = StockId("boundary.leak_sink")


# --- the two shared helpers (forcing-schedule perturbations) -----------------


def window_override(base: Schedule, *, start: int, end: int, value: float) -> Schedule:
    """A ``Schedule`` forcing ``value`` on ``[start, end)``, else the wrapped ``base``.

    The override is a pure function of the integer step ``n`` (#14) — the legitimate
    forcing seam: inside ``[start, end)`` it forces ``value`` (e.g. irrigation/PAR → 0,
    or a leak's activation → 1), outside it defers to the wrapped ``base`` schedule
    (the unperturbed forcing). ``dt`` is threaded to ``base`` unchanged.
    """

    def schedule(n: int, dt: float) -> float:
        if start <= n < end:
            return value
        return base(n, dt)

    return schedule


def with_forcing(
    resolver: SourceResolver, var: str, schedule: Schedule
) -> SourceResolver:
    """Rebuild ``resolver`` with one forcing ``var`` replaced/added; shared map kept.

    The structural complement to :func:`window_override`: the override changes a *value*
    over a window, this swaps the *wiring* for one var and rebuilds the immutable
    resolver. The ``shared`` map (the #16 live-stock seam) is copied verbatim, so the
    forcing/shared disjointness ``SourceResolver`` enforces is preserved (a new forcing
    var like :data:`LEAK_VAR` must not collide with a shared var — it does not).
    """
    return SourceResolver(
        forcings={**dict(resolver.forcings), var: schedule},
        shared=dict(resolver.shared),
    )


def with_drought(resolver: SourceResolver, *, start: int, end: int) -> SourceResolver:
    """Cut irrigation to 0 over ``[start, end)`` (the drought perturbation; open field).

    Wraps the irrigation forcing with a windowed ``value=0`` override: over the window
    the soil-water pool stops refilling and transpiration draws it across the stress
    band, so ``f_water < 1`` and assimilation falls — a cascade, no cascade code. The
    open field is the only scenario with an irrigation forcing to cut (sealed dropped it
    in Step 3); on a sealed resolver this raises ``KeyError`` at the lookup, by design.
    """
    base = resolver.forcings[IRRIGATION_VAR]
    return with_forcing(
        resolver, IRRIGATION_VAR, window_override(base, start=start, end=end, value=0.0)
    )


def with_lighting_failure(
    resolver: SourceResolver, *, start: int, end: int
) -> SourceResolver:
    """Cut PAR to 0 over ``[start, end)`` (the lighting-failure perturbation).

    Wraps the PAR forcing with a windowed ``value=0`` override: over the window FvCB's
    light term collapses (``J → 0``), assimilation → 0, growth stalls, O₂ production
    drops and — in the sealed chamber — the carbon pool stops drawing down, so chamber
    CO₂ *rises*. A cascade with no cascade code; conservation is structurally unaffected
    (only a forcing value changes).
    """
    base = resolver.forcings[PAR_VAR]
    return with_forcing(
        resolver, PAR_VAR, window_override(base, start=start, end=end, value=0.0)
    )


# --- the flow perturbation (atmospheric leak) --------------------------------


@dataclass(frozen=True)
class LeakFlow:
    """A windowed boundary leak ``pool -> sink``, first-order in the pool (P3.5).

    The pure rate law is first-order donor control ``k_leak · pool · active(n) · dt``
    where ``active(n) = env.get(leak_var) ∈ {0, 1}`` carries the calendar (the window
    lives in the *schedule*, not the flow body — the Step-4 discipline). Structural
    positivity: the draw → 0 as the pool → 0, and ``k_leak·dt < 1`` keeps the Euler
    backstop unfired (``rationed == 0``) with no ``max(0, …)`` clamp — the decomposition
    / condensation precedent. ``sink`` mirrors ``pool``'s element composition (built by
    :func:`with_atmospheric_leak`), so the leg is per-quantity balanced and the gate
    folds it exactly (a CO₂ pool vents both CARBON and OXYGEN, 2:1). ``flux = daily·dt``
    (dt-linear), so the RK4 increment-form contract holds.
    """

    id: FlowId
    priority: int
    pool: StockId
    sink: StockId
    leak_var: str
    k_leak: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        active = env.get(self.leak_var)
        amount = self.k_leak * snapshot.stocks[self.pool].amount * active * dt
        return FlowResult(legs=(Leg(self.pool, -amount), Leg(self.sink, amount)))


def with_atmospheric_leak(
    state: State,
    registry: Registry,
    resolver: SourceResolver,
    *,
    pool: StockId,
    k_leak: float,
    start: int,
    end: int,
) -> tuple[State, Registry, SourceResolver]:
    """Augment the assembled inputs with a windowed leak ``pool -> leak_sink``.

    Builds three things and returns the augmented triple (the baseline inputs are
    untouched — this composes onto them):

    * a :data:`LEAK_SINK` BOUNDARY stock whose composition **mirrors ``pool``'s**
      (so a multi-quantity CO₂ pool vents CARBON+OXYGEN in balance; built directly, not
      via ``boundary.sink`` which is single-quantity — the multi-quantity BOUNDARY is
      legal, only POPULATION is single-quantity-constrained);
    * a :class:`LeakFlow` ``pool -> leak_sink`` appended to the registry (Registry
      re-sorts by id, so append order is inert; the existing aux processes are carried);
    * the :data:`LEAK_VAR` activation forcing wired into the resolver as a windowed
      override (``1`` on ``[start, end)``, ``0`` else) — so the leak's calendar is a
      schedule, read by the flow via ``env.get``.

    The chamber's interior closure breaks over the window (mass leaves to the boundary
    leak-sink) but **total** mass (interior + leak-sink) stays conserved — the leg is
    balanced, the gate folds it. ``k_leak·dt < 1`` keeps ``rationed == 0``.
    """
    pool_stock = state.stocks[pool]
    leak_sink = Stock(
        id=LEAK_SINK,
        domain=BOUNDARY_DOMAIN,
        quantity=pool_stock.quantity,
        unit=pool_stock.unit,
        amount=0.0,
        kind=StockKind.BOUNDARY,
        composition=dict(pool_stock.composition),
    )
    new_stocks = {**dict(state.stocks), LEAK_SINK: leak_sink}
    new_state = replace(state, stocks=new_stocks)
    leak = LeakFlow(
        FlowId("biosphere.atmospheric_leak"),
        0,
        pool=pool,
        sink=LEAK_SINK,
        leak_var=LEAK_VAR,
        k_leak=k_leak,
    )
    new_registry = Registry(
        [*registry.flows, leak], new_stocks, aux_processes=registry.aux_processes
    )
    new_resolver = with_forcing(
        resolver,
        LEAK_VAR,
        window_override(constant(0.0), start=start, end=end, value=1.0),
    )
    return new_state, new_registry, new_resolver
