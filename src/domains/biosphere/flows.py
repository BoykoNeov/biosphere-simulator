"""Biosphere demo flows: trivial, dt-linear, first-order transfers (step 10).

No real biology yet — no FvCB / Penman–Monteith / saturating kinetics (those are
Phase 1). Each flow is **strictly proportional** to the stock it withdraws from, so
``leg == dt·rate`` with ``rate`` independent of ``dt`` (the step-6 increment-form
contract → RK4 stays 4th order) *and* the well-fed guarantee becomes a
trajectory-independent bound (see the step-10 design in the plan):

  * ``Photosynthesis``  ``atmospheric_c -> plant_c``   ``flux = k_photo·light·atm·dt``
        ``light = env.get(light_var)`` is read as a **scalar rate multiplier** — it
        is *not* a consumed energy leg (this demo tracks no ENERGY transfer; the
        ``boundary.light`` stock is constant forcing). ``ENERGY`` itself became an
        asserted conserved quantity in Phase 5 — see ``simcore.quantities`` — but
        that does not change this flow: it touches only CARBON.
        This is the flow that carries the internal source-resolver case (#16): the
        caller cannot tell whether ``light`` came from a forcing schedule or a
        sibling-domain boundary stock.
  * ``Respiration``     ``plant_c -> atmospheric_c``    ``flux = k_resp·plant·dt``
        Autonomous (ignores ``env``).
  * ``Harvest``         ``plant_c -> outside_c``        ``flux = k_harv·plant·dt``
        Cross-domain (biosphere → boundary): harvested carbon leaves the modeled
        system *into* a boundary reservoir, so the flow is still internally balanced
        (#13).

Every flow is internally balanced in carbon (``Σ legs == 0``): one withdrawal and
one deposit of equal magnitude. Pure stdlib only.
"""

from dataclasses import dataclass

from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class Photosynthesis:
    """``atmospheric_c -> plant_c`` at first-order rate ``k_photo·light`` (#16).

    ``light`` is read from ``env`` as a scalar rate multiplier (not a consumed leg);
    the carbon transfer itself is balanced. ``flux = k_photo·light·atm_c·dt``.
    """

    id: FlowId
    priority: int
    atmospheric_c: StockId
    plant_c: StockId
    light_var: str
    k_photo: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        light = env.get(self.light_var)
        flux = self.k_photo * light * snapshot.stocks[self.atmospheric_c].amount * dt
        return FlowResult(
            legs=(Leg(self.atmospheric_c, -flux), Leg(self.plant_c, flux))
        )


@dataclass(frozen=True)
class Respiration:
    """``plant_c -> atmospheric_c`` at first-order rate ``k_resp`` (autonomous).

    ``flux = k_resp·plant_c·dt``.
    """

    id: FlowId
    priority: int
    plant_c: StockId
    atmospheric_c: StockId
    k_resp: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        flux = self.k_resp * snapshot.stocks[self.plant_c].amount * dt
        return FlowResult(
            legs=(Leg(self.plant_c, -flux), Leg(self.atmospheric_c, flux))
        )


@dataclass(frozen=True)
class Harvest:
    """``plant_c -> outside_c`` (boundary) at first-order rate ``k_harv``.

    Cross-domain (biosphere → boundary). ``flux = k_harv·plant_c·dt``. ``outside_c``
    is a BOUNDARY reservoir, so this "unbalanced" removal is balanced once the
    boundary is counted (#13).
    """

    id: FlowId
    priority: int
    plant_c: StockId
    outside_c: StockId
    k_harv: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        flux = self.k_harv * snapshot.stocks[self.plant_c].amount * dt
        return FlowResult(legs=(Leg(self.plant_c, -flux), Leg(self.outside_c, flux)))
