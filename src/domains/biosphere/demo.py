"""The Phase-0 two-domain demo: Biosphere + Boundary (step 10).

Wires the engine spine end-to-end — two domains, three flows (``Harvest`` is
cross-domain), a ``SourceResolver``, the loss-sink-bearing boundary — into one
runnable scenario. This module builds **no new core machinery**; it is the
integration that proves the spine on trivial laws (see ``biosphere.flows``). It is
the biosphere domain's showcase, and it wires in Boundary reservoirs because the
demo's boundary exchange (#13) and internal resolver (#16) need them.

**Parameters are declared data.** ``DemoParams`` holds the coefficients / initial
amounts / light level / ``dt`` and is injected into the flows — no magic numbers in
flow logic. It carries **no inline defaults**: ``params/demo.yaml`` is the single
source of truth, loaded + unit-validated by ``domains.biosphere.loader`` (step 11).
``build_demo`` / ``forcing_resolver`` therefore take ``params`` as a required arg.

Pure stdlib only — the YAML + pydantic + pint loading lives in ``loader.py`` so this
assembly (and ``flows``) stays importable headless.
"""

from dataclasses import dataclass

from domains.biosphere.flows import Harvest, Photosynthesis, Respiration
from simcore import boundary
from simcore.environment import SourceResolver, constant
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- canonical namespace + ids ---------------------------------------------
# The *real* biosphere domain id (test_integrator's "bio" is test-local). The
# cross-domain Harvest assertion depends on this exact string.
BIOSPHERE: DomainId = DomainId("biosphere")

ATMOSPHERIC_C: StockId = StockId("biosphere.atmospheric_c")
PLANT_C: StockId = StockId("biosphere.plant_c")
OUTSIDE_C: StockId = StockId("boundary.outside_c")
LIGHT: StockId = StockId("boundary.light")

# The env var Photosynthesis reads. Wired as either a forcing schedule or the
# shared boundary.light stock — the reader cannot tell (decision #16).
LIGHT_VAR: str = "light"

PHOTOSYNTHESIS: FlowId = FlowId("biosphere.photosynthesis")
RESPIRATION: FlowId = FlowId("biosphere.respiration")
HARVEST: FlowId = FlowId("biosphere.harvest")

# A concrete integrator that exposes step_report (the two Phase-0 strategies). The
# frozen Integrator Protocol carries only step(); run() needs the per-step report.
DemoIntegrator = EulerIntegrator | Rk4Integrator


@dataclass(frozen=True)
class DemoParams:
    """Declared demo parameters — data, not magic numbers in flow logic.

    The coefficients satisfy the **well-fed structural bound** (step-10 design): the
    demand/available ratio on any clamped stock is exactly ``(Σ rates on it)·dt``,
    trajectory-independent because every withdrawal is first-order, so

        ``k_photo·light·dt < 1``        (atmospheric_c)
        ``(k_resp + k_harv)·dt < 1``    (plant_c)

    keep the arbitration backstop from ever firing. Amounts are O(1)–O(1e3) so the
    always-on conservation gate's relative tolerance stays well above the float
    floor. Units are PROVISIONAL (decision #9 / Phase 1).
    """

    atmospheric_c0: float
    plant_c0: float
    outside_c0: float
    light: float
    k_photo: float
    k_resp: float
    k_harv: float
    dt: float


def build_demo(params: DemoParams) -> tuple[State, Registry]:
    """Build the demo's initial ``State`` and flow ``Registry``.

    The state **always** includes ``boundary.light`` (the inert energy driver) and
    the carbon loss-sink, so (a) the forcing/shared resolver variants share an
    identical stock set — the indistinguishability gate compares them bit-for-bit —
    and (b) the assembly is referentially complete for extinction routing (#6), even
    though the well-fed demo never triggers it.
    """
    carbon = canonical_unit(Quantity.CARBON)
    atmospheric = Stock(
        id=ATMOSPHERIC_C,
        domain=BIOSPHERE,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=params.atmospheric_c0,
        kind=StockKind.POOL,
    )
    plant = Stock(
        id=PLANT_C,
        domain=BIOSPHERE,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=params.plant_c0,
        kind=StockKind.POPULATION,
        extinction_threshold=0.0,
    )
    outside = boundary.sink(OUTSIDE_C, Quantity.CARBON, params.outside_c0)
    # An "outside" solar reservoir, read as a scalar driver and never touched by a
    # leg, so its delta is always 0. ENERGY is now an asserted conserved quantity
    # (Phase 5), but a zero per-step delta conserves trivially — this stock passes
    # the gate untouched (and the Phase-0 demo gains a free ENERGY-conservation
    # check). unclamped=True.
    light = boundary.source(LIGHT, Quantity.ENERGY, params.light)

    stocks: dict[StockId, Stock] = {
        s.id: s for s in (atmospheric, plant, outside, light)
    }
    # Only carbon biomass can go extinct here, so only the carbon loss-sink is needed.
    stocks.update(boundary.loss_sinks({Quantity.CARBON}))

    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        Photosynthesis(
            PHOTOSYNTHESIS, 0, ATMOSPHERIC_C, PLANT_C, LIGHT_VAR, params.k_photo
        ),
        Respiration(RESPIRATION, 0, PLANT_C, ATMOSPHERIC_C, params.k_resp),
        Harvest(HARVEST, 0, PLANT_C, OUTSIDE_C, params.k_harv),
    ]
    return state, Registry(flows, stocks)


def forcing_resolver(params: DemoParams) -> SourceResolver:
    """``light`` wired as a constant forcing schedule (the standalone-biosphere view).

    Produces a run **bit-identical** to ``coupled_resolver``'s (decision #16) because
    ``boundary.light`` is constant at ``params.light``.
    """
    return SourceResolver(forcings={LIGHT_VAR: constant(params.light)})


def coupled_resolver() -> SourceResolver:
    """``light`` wired to the ``boundary.light`` shared stock (the canonical #16 path).

    The canonical golden wiring: it actually exercises the internal-resolver read
    path every step (the run is wiring-independent by the indistinguishability gate).
    """
    return SourceResolver(shared={LIGHT_VAR: LIGHT})


def run(
    integrator: DemoIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[State, int, tuple[Event, ...]]:
    """Step ``steps`` times, summing the rationing-firing count and events.

    The single driver shared by the well-fed gate and (step 11) the golden run. On
    the well-fed demo ``total_rationed`` must be 0 (the backstop gate) and ``events``
    empty (no extinction). Returns ``(final_state, total_rationed, events)``.
    """
    total_rationed = 0
    events: list[Event] = []
    for _ in range(steps):
        report = integrator.step_report(state, resolver, dt)
        state = report.state
        total_rationed += report.rationed
        events.extend(report.events)
    return state, total_rationed, tuple(events)
