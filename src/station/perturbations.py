"""Cross-domain perturbation harness (Phase-6 Step 8, P6.8) — cascades, no cascade code.

The Phase-3 :mod:`domains.biosphere.perturbations` discipline carried **cross-domain**:
a perturbation is a **scenario-layer intervention composed onto the already-assembled
station inputs**, never a core/domain change. Each shipped perturbation is a **cascade
with no cascade code** — a disturbance applied to **one** domain propagates into
**another** through a shared stock (or a shared *forcing*, #16) alone, while
conservation holds and ``rationed`` *behaves*. The harness composes *outside* the
``build_*`` builders, so ``git diff src/{simcore,domains}/`` stays empty and all twenty
existing goldens stay byte-identical; a perturbation run gets **no golden** (a
behavioural demonstration — the Phase-3 "diagnostics, no golden" precedent).

**The genuinely-new thing vs Phase-3.** Phase-3's three perturbations were
*single-domain* (all inside the biosphere). Here each perturbation crosses a domain
boundary:

* **brownout** (``solar_power`` cut) — Power SOC↓ ⇒ the Thermal node **cools** (less
  dissipation); deep/long enough it empties the battery ⇒ ``rationed > 0`` **emerges**
  (the *failure* cascade the exit criterion wants).
* **radiator failure** (:class:`ScaledFlow` throttling ``RadiatorReject``) — the node
  can no longer shed Power's real dissipation ⇒ it **heats**, T rises. Energy stays
  in-system (conserved); ``rationed == 0`` (a POOL accumulation, not a withdrawal
  shortfall).
* **atmosphere leak** (:class:`~domains.biosphere.perturbations.LeakFlow` on the shared
  cabin air) — the two gas pools **do not fail the same way** (below): a ``CARBON_POOL``
  leak drops Ci ⇒ the plant assimilates less + the scrubber does less; an ``O2_POOL``
  leak is absorbed by ``O2Makeup`` (``o2_supply`` effort↑, ``cabin_o2`` flat).
* **crew load spike** (``food_intake`` ×factor) — both ECLSS regulators work harder
  (``co2_removed↑`` + ``o2_supply↑``) and ``food_store`` depletes faster.
* **lighting failure** (``par`` **and** ``lamp_power`` → 0) — the biosphere's growth
  stalls **and** the battery is spared: the #16 lamp is **one** intervention with a
  photon leg (a forcing) and an energy leg (the ``Lamp`` flow's draw).

**The load-bearing finding (advisor, spike-CONFIRMED): the station regulators ERASE the
naive pool-level signature.** Unlike Phase-3's un-regulated chamber, every station gas
pool is regulated, so in every matter perturbation the *day-boundary* ``CARBON_POOL`` /
``O2_POOL`` / ``Ci`` return **identical to baseline** (the Step-3 regulator-erasure
physics, under disturbance). The emergent signature is regulator **effort**
(``co2_removed`` / ``o2_supply``) + the sinks (``LEAK_SINK``, biomass), **not** pool
level — and the two pools differ: ``CARBON_POOL`` is only *removed* (``CO2Scrubber`` is
first-order donor-controlled — it cannot push CO₂ *up*), so a leak genuinely lowers it
*within* the window; ``O2_POOL`` is *defended* (``O2Makeup`` is demand-controlled toward
a setpoint), so a leak surfaces as makeup effort. Tests assert on the effort/sink
signal, never the erased pool level.

**Three seam-types.** (1) **Forcing override** — reuse Phase-3's generic
:func:`~domains.biosphere.perturbations.window_override` /
:func:`~domains.biosphere.perturbations.with_forcing` (pure ``(n, dt)``
schedule/resolver transforms), plus :func:`window_scale` (the station's *multiplicative*
window, for scaling an existing schedule rather than forcing a constant). (2) **Added
leak flow** — reuse Phase-3's :class:`~domains.biosphere.perturbations.LeakFlow` +
:data:`~domains.biosphere.perturbations.LEAK_SINK` (a generic, composition-mirroring
``pool → sink``). (3) **Windowed flow-scaler** — the legitimately-new
:class:`ScaledFlow` (scale an existing flow's *whole* output by a windowed ``health ∈
[0, 1]`` so it stays internally balanced — the "arbitration scales the whole flow"
invariant, applied as a perturbation).

**Rejected — a ``Perturbation`` protocol.** Speculative generality for 5 perturbations
(the Phase-3 call); the small builder functions + two shared helpers here are leaner,
additive if a real multi-perturbation composition need appears. Pure stdlib +
``simcore`` + ``domains`` + ``station`` (the assembly layer imports domains freely; no
domain imports another).
"""

from dataclasses import dataclass, replace

from domains.biosphere.perturbations import (
    LEAK_SINK,
    LEAK_VAR,
    LeakFlow,
    window_override,
    with_forcing,
)
from domains.biosphere.stocks import PAR_VAR
from domains.crew.stocks import FOOD_INTAKE_VAR
from domains.power.stocks import SOLAR_POWER_VAR
from domains.thermal.system import RADIATOR_REJECT
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.environment import Environment, Schedule, SourceResolver, constant
from simcore.flow import Flow, FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.quantities import StockKind
from simcore.registry import Registry
from simcore.state import State, Stock
from station.flows import LAMP_POWER_VAR

# The radiator's windowed "health" forcing var (1.0 = nominal, 0.0 = total loss). Local
# to this module — never added to any domain's stock/forcing catalog, so baseline
# assembly never carries it and the existing goldens are untouched (the ``LEAK_VAR``
# discipline).
RADIATOR_HEALTH_VAR: str = "radiator_health"

# The station leak flow's id — station-owned (the leak reads a shared gas pool and
# writes a boundary sink), kept out of the biosphere registry so the sealed build's
# flow-id disjointness guard holds (:func:`station.sealed._assert_flow_ids_disjoint`).
STATION_LEAK: FlowId = FlowId("station.atmospheric_leak")


# --- the multiplicative window (the complement to window_override) --------------------


def window_scale(base: Schedule, *, start: int, end: int, factor: float) -> Schedule:
    """A ``Schedule`` returning ``factor · base`` on ``[start, end)``, else ``base``.

    The *multiplicative* sibling of Phase-3's :func:`window_override` (which forces a
    fixed value): a pure function of the integer step ``n`` (#14) that **scales** an
    existing, ``n``-varying schedule over the window — a brownout dims the diurnal
    ``solar_power`` half-sine by ``factor``, a crew load spike lifts ``food_intake`` by
    ``factor > 1``. ``factor = 1`` is a no-op (bit-identical); ``factor = 0``
    degenerates to a blackout (equivalent to ``window_override(base, value=0)``). ``dt``
    threads to ``base`` unchanged.
    """

    def schedule(n: int, dt: float) -> float:
        if start <= n < end:
            return factor * base(n, dt)
        return base(n, dt)

    return schedule


# --- forcing-override perturbations (no new flow / no structural change) --------------


def with_brownout(
    resolver: SourceResolver, *, start: int, end: int, factor: float = 0.0
) -> SourceResolver:
    """Scale the ``solar_power`` forcing by ``factor`` over ``[start, end)`` (Power).

    The cross-domain energy perturbation: over the window the diurnal solar supply is
    dimmed (``factor = 0`` is a full blackout), so the battery SOC falls (Power) **and**
    the heat Power dissipates into ``thermal.node`` drops, so the node **cools**
    (Thermal) — a cascade with no cascade code. A *short/shallow* window stays graceful
    (``rationed == 0``, SOC dips but > 0); a *deep/long* one empties the battery so
    ``LoadDraw`` cannot be met and ``rationed > 0`` **emerges** (the failure cascade,
    still conserving — the Euler backstop conserves as it rations). Runs on the diurnal
    single-rate ``run_station``.
    """
    base = resolver.forcings[SOLAR_POWER_VAR]
    return with_forcing(
        resolver,
        SOLAR_POWER_VAR,
        window_scale(base, start=start, end=end, factor=factor),
    )


def with_crew_load_spike(
    resolver: SourceResolver, *, start: int, end: int, factor: float = 2.0
) -> SourceResolver:
    """Scale crew ``food_intake`` by ``factor`` over ``[start, end)`` (Crew).

    The cross-domain crew perturbation: a raised metabolic food intake drives more
    respiration, so the cabin CO₂/O₂ loads jump — but the regulators absorb the pools,
    so the emergent signature is **regulator effort** (``CO2Scrubber`` removes more,
    ``O2Makeup`` supplies more) plus a **faster ``food_store`` drawdown**, not a
    pool-level shift (the day-boundary ``CARBON_POOL`` / ``O2_POOL`` return to
    setpoint). O₂ consumption is derived from food (RQ = 1, the ``CrewRespiration``
    merge), so scaling food scales the O₂ side too; the WATER echo (a proportional
    ``water_intake`` spike) is left off — food is the gas-loop lever. Runs on the short
    two-rate ``run_sealed`` (the fast registry's ``food_intake``).
    """
    base = resolver.forcings[FOOD_INTAKE_VAR]
    return with_forcing(
        resolver,
        FOOD_INTAKE_VAR,
        window_scale(base, start=start, end=end, factor=factor),
    )


def with_lighting_failure(
    bio_resolver: SourceResolver,
    fast_resolver: SourceResolver,
    *,
    start: int,
    end: int,
) -> tuple[SourceResolver, SourceResolver]:
    """Zero ``par`` **and** ``lamp_power`` over ``[start, end)`` (#16 lamp).

    The one **two-resolver** perturbation: the lamp is a single device whose failure has
    a *photon* leg (the biosphere's ``par`` **forcing**, in ``bio_resolver``) and an
    *energy* leg (the ``Lamp`` flow's ``lamp_power`` draw, in ``fast_resolver``) — a
    flow cannot tell forcing from a shared stock (#16), so both must be cut
    **together**. The cascade: PAR → 0 ⇒ FvCB's light term collapses ⇒ growth stalls
    (biomass below baseline); and the lamp draws no energy ⇒ the battery is **spared**
    (drains slower). Returns the ``(bio, fast)`` resolver pair. Runs on the short
    two-rate ``run_sealed``.
    """
    par = bio_resolver.forcings[PAR_VAR]
    lamp = fast_resolver.forcings[LAMP_POWER_VAR]
    new_bio = with_forcing(
        bio_resolver, PAR_VAR, window_override(par, start=start, end=end, value=0.0)
    )
    new_fast = with_forcing(
        fast_resolver,
        LAMP_POWER_VAR,
        window_override(lamp, start=start, end=end, value=0.0),
    )
    return new_bio, new_fast


# --- the windowed flow-scaler (the new third seam-type) -------------------------------


@dataclass(frozen=True)
class ScaledFlow:
    """Wrap a flow; multiply **all** its legs by a windowed ``health ∈ [0, 1]`` forcing.

    The legitimately-new third seam-type (radiator failure): a *degrade an existing
    process* perturbation. It scales the **whole flow** — every leg by the same
    ``env.get(health_var)`` — so the result stays internally balanced (``Σ (α·leg) = α·Σ
    leg = 0`` per quantity), the "arbitration scales the whole flow" invariant applied
    as a disturbance rather than a backstop. ``health = 1`` outside the window
    reproduces the wrapped flow **bit-identically** (``x · 1.0 == x`` in IEEE), so the
    perturbation is confined to the window exactly. The wrapped flow is unmodified
    (composition over inheritance); ``id`` / ``priority`` delegate, so the ``Registry``
    sorts it into the wrapped flow's slot (order-independence preserved).
    """

    inner: Flow
    health_var: str

    @property
    def id(self) -> FlowId:
        return self.inner.id

    @property
    def priority(self) -> int:
        return self.inner.priority

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        alpha = env.get(self.health_var)
        result = self.inner.evaluate(snapshot, env, dt)
        return FlowResult(
            legs=tuple(Leg(leg.stock, leg.amount * alpha) for leg in result.legs)
        )


def with_radiator_failure(
    state: State,
    registry: Registry,
    resolver: SourceResolver,
    *,
    start: int,
    end: int,
    health: float = 0.0,
) -> tuple[Registry, SourceResolver]:
    """Throttle ``RadiatorReject`` to ``health`` over ``[start, end)`` (Thermal).

    Wraps the ``RADIATOR_REJECT`` flow in a :class:`ScaledFlow` reading a windowed
    :data:`RADIATOR_HEALTH_VAR` (``health`` inside the window, ``1.0`` outside) and
    rebuilds the registry over ``state.stocks`` (``Registry`` re-sorts by id, so append
    order is inert; aux processes are carried; ``state`` is read only for its stock
    mapping — the perturbation adds no stock). Over the window the radiator sheds only
    ``health ×`` its nominal rejection, so the node accumulates Power's real dissipation
    and **heats** — the emergent overheating cascade. Energy is conserved throughout
    (the scaled leg is still balanced — heat stays in the node instead of leaving to
    ``space``); ``rationed == 0`` (a POOL accumulation, not a withdrawal shortfall).
    Structural only (no new stock), so returns ``(registry, resolver)``. Runs on the
    single-rate ``run_station``.
    """
    flows: list[Flow] = [
        ScaledFlow(flow, RADIATOR_HEALTH_VAR) if flow.id == RADIATOR_REJECT else flow
        for flow in registry.flows
    ]
    new_registry = Registry(flows, state.stocks, aux_processes=registry.aux_processes)
    new_resolver = with_forcing(
        resolver,
        RADIATOR_HEALTH_VAR,
        window_override(constant(1.0), start=start, end=end, value=health),
    )
    return new_registry, new_resolver


# --- the added leak flow (two-registry sealed build) ----------------------------------


def with_station_leak(
    state: State,
    bio_registry: Registry,
    fast_registry: Registry,
    fast_resolver: SourceResolver,
    *,
    pool: StockId,
    k_leak: float,
    start: int,
    end: int,
) -> tuple[State, Registry, Registry, SourceResolver]:
    """Augment the sealed build with a windowed leak ``pool → LEAK_SINK`` (matter).

    The two-registry analogue of Phase-3's
    :func:`~domains.biosphere.perturbations.with_atmospheric_leak`. The sealed station
    has **two** registries over **one** shared stock dict, so:

    * a :data:`~domains.biosphere.perturbations.LEAK_SINK` BOUNDARY stock whose
      composition **mirrors ``pool``'s** is added to the shared stocks (a ``{C:1,O:2}``
      cabin-CO₂ pool vents CARBON **and** OXYGEN in balance);
    * **both** registries are rebuilt over the augmented stock dict (the biosphere-slow
      one preserving its ``thermal_time`` aux), and the :class:`LeakFlow` is appended to
      the **fast** registry (``dt = 60 s``, the same rate the cabin flows act on the
      shared pool; ``k_leak·dt < 1`` is trivial at 60 s). Its id (:data:`STATION_LEAK`)
      is kept **out of the biosphere registry**, so the sealed build's flow-id
      disjointness guard holds;
    * the :data:`~domains.biosphere.perturbations.LEAK_VAR` activation forcing is wired
      into the **fast** resolver as a windowed override (``1`` on ``[start, end)``,
      ``0`` else) — under ``substep`` the day count ``n`` is frozen within a day, so the
      window activates on whole master days (the sealed-station timing).

    The chamber interior's closure breaks over the window (mass leaves to ``LEAK_SINK``)
    but **total** mass (interior + sink) stays conserved — the leg is balanced, so a
    completed run (the driver's per-sub-step gate folds ``LEAK_SINK``) is itself the
    conservation proof. The emergent cascade is pool-specific: a ``CARBON_POOL`` leak
    lowers Ci within the window (biomass below baseline, scrubber does less), an
    ``O2_POOL`` leak is absorbed by ``O2Makeup`` (its ``o2_supply`` effort grows,
    ``cabin_o2`` stays flat).
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
        STATION_LEAK,
        0,
        pool=pool,
        sink=LEAK_SINK,
        leak_var=LEAK_VAR,
        k_leak=k_leak,
    )
    new_bio = Registry(
        list(bio_registry.flows),
        new_stocks,
        aux_processes=bio_registry.aux_processes,
    )
    new_fast = Registry([*fast_registry.flows, leak], new_stocks)
    new_fast_resolver = with_forcing(
        fast_resolver,
        LEAK_VAR,
        window_override(constant(0.0), start=start, end=end, value=1.0),
    )
    return new_state, new_bio, new_fast, new_fast_resolver
