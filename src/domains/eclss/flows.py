"""ECLSS flows: the forced crew seam + three ECLSS control loops (Step 6).

Four flows of the standalone cabin-air core. Each is internally balanced **per conserved
quantity** (``assert_flow_balanced`` groups legs by each stock's composition and asserts
every asserted quantity independently), which is what lets one flow touch three
quantities at once:

  * **CrewMetabolism** — the **forced, multi-quantity** crew/Phase-6 seam (the analogue
    of Thermal's forced ``heat_source``). One flow, **six legs across three quantities,
    each balanced independently**: O₂ ``cabin_o2 → boundary.metabolic_o2_sink`` (crew
    consumes O₂), CO₂ ``boundary.metabolic_co2_source → cabin_co2`` (crew produces CO₂),
    H₂O ``boundary.metabolic_h2o_source → cabin_h2o`` (crew produces humidity). Three
    forced rates ``env.get(...)·dt``. **Forced** (reads ``env``, not a stock); it does
    **not** tie the crew's atoms together (metabolic O₂ leaves to a sink; CO₂/H₂O enter
    from separate sources) — the decoupled crew seam Phase-6 coupling + composition
    stocks close (see ``stocks``). This is the single seam Phase-6 rewires to the Crew
    domain.
  * **CO2Scrubber** — ``cabin_co2 → boundary.co2_removed``, **first-order
    donor-controlled** ``R = k_scrub·cabin_co2·dt`` (mol). The ``SelfDischarge``
    pattern, but load-bearing: the domain's CO₂ restoring force. Two legs, one magnitude
    ⇒ CARBON balances exactly.
  * **Condenser** — ``cabin_h2o → boundary.humidity_condensate``, **first-order
    donor-controlled** ``R = k_cond·cabin_h2o·dt`` (kg). The humidity restoring force.
    Two legs ⇒ WATER balances exactly.
  * **O2Makeup** — ``boundary.o2_supply → cabin_o2``, **demand-controlled toward a
    setpoint** ``S = k_makeup·(o2_setpoint − cabin_o2)·dt`` (mol) — the real ECLSS
    O₂-partial-pressure regulator. Two legs, one magnitude ⇒ OXYGEN balances exactly.

**Why demand-control for O₂ (the load-bearing choice).** A *forced-constant* makeup
would leave ``cabin_o2`` a restoring-force-free accumulator — Power's derived-load
situation, bounded only by exact balance. The proportional controller makes O₂ a
**restoring force** (``−k_makeup·cabin_o2`` term) with no readout needed, so **all three
species share one "restoring force → attractor" story**. The controller is linear (a
proportional gain, not clamped): in every standalone scenario ``cabin_o2 ≤ o2_setpoint``
(the consuming crew keeps it below), so makeup only ever *adds* (``S ≥ 0``); an
above-setpoint venting clamp is a deferred seam that never arises here and would break
the clean linearity.

**Linear ⇒ geometric, NOT the T⁴ nonlinear attractor (honest framing).** Every control
law is first-order in the stock amount (or demand-controlled in it), so contraction is
**geometric** — two runs differing only in one species' initial amount contract by the
exact ``d_n = d_0·(1 − k·dt)^n`` law (the ``SelfDischarge`` idiom, per species), **not**
Thermal's nonlinear monotone contraction. Each species reaches a steady state: ``co2_eq
= P_co2/k_scrub``, ``h2o_eq = P_h2o/k_cond``, ``o2_eq = o2_setpoint − Con_o2/k_makeup``
(see ``system.steady_state``).

**Positivity — a mix (honest about how each holds).** CO₂/H₂O are **structural** (``k·dt
< 1``: donor-controlled, ∝ the stock's own amount, self-limiting to 0 — the
``SelfDischarge`` way). O₂ is by **well-fed sizing** (the ``LoadDraw`` way): the
depletion side is ``CrewMetabolism``'s O₂ leg, sized so ``cabin_o2`` never empties;
makeup only adds to it. ``rationed == 0`` holds by both, per side.

**RK4 ≢ Euler (a tolerance agreement).** The three control flows read stocks, so the
forced-only bit-identity does not hold (``k1 ≠ k2``); the integrators agree to
``O(dt²)``.

Pure stdlib only. Citations: first-order gas scrubbing / condensation and proportional
partial-pressure control are textbook process-control physics (clean-room). The rate
constants / setpoint are ``params/eclss.yaml`` (illustrative ``TODO(cite)`` placeholders
— NOT NASA BVAD / BioSim numbers; calibration is Phase 6).
"""

from dataclasses import dataclass

from domains.eclss.stocks import (
    CO2_PRODUCTION_VAR,
    H2O_PRODUCTION_VAR,
    O2_CONSUMPTION_VAR,
)
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class EclssParams:
    """Loader-produced ECLSS equipment parameters (the control-loop coefficients).

    Provisional literature-typical placeholders pending the validation gate (see
    ``params/eclss.yaml``). All four are exact-string unit-guarded at the loader (none
    is a conserved-Quantity canonical unit, so none routes through pint — the
    ``ChargeParams`` / ``ThermalParams`` discipline):

      * ``co2_scrub_rate`` (k_scrub, 1/s) > 0: first-order CO₂ removal rate; ``co2_eq =
        P_co2 / k_scrub``. Structural positivity requires ``k_scrub·dt < 1``.
      * ``condense_rate`` (k_cond, 1/s) > 0: first-order humidity removal rate; ``h2o_eq
        = P_h2o / k_cond``. Structural positivity requires ``k_cond·dt < 1``.
      * ``o2_makeup_gain`` (k_makeup, 1/s) > 0: proportional gain of the O₂ regulator;
        ``o2_eq = o2_setpoint − Con_o2 / k_makeup``.
      * ``o2_setpoint`` (mol) > 0: the target cabin O₂ inventory the regulator drives
        toward (the analogue of a partial-pressure setpoint at fixed
        volume/temperature).
    """

    co2_scrub_rate: float
    condense_rate: float
    o2_makeup_gain: float
    o2_setpoint: float


def scrub_flux(cabin_co2: float, *, co2_scrub_rate: float) -> float:
    """Instantaneous CO₂ scrub rate ``k_scrub · cabin_co2`` (mol/s).

    First-order donor-controlled (the ``Decomposition`` / ``SelfDischarge`` form), so it
    → 0 as ``cabin_co2 → 0`` (structural positivity). A flow multiplies by ``dt`` for
    the per-step increment. At ``cabin_co2 = 0`` it is exactly 0.
    """
    return co2_scrub_rate * cabin_co2


def condense_flux(cabin_h2o: float, *, condense_rate: float) -> float:
    """Instantaneous humidity-condensation rate ``k_cond · cabin_h2o`` (kg/s).

    First-order donor-controlled, → 0 as ``cabin_h2o → 0`` (structural positivity). A
    flow multiplies by ``dt`` for the per-step increment.
    """
    return condense_rate * cabin_h2o


def makeup_flux(cabin_o2: float, *, o2_makeup_gain: float, o2_setpoint: float) -> float:
    """Instantaneous O₂-makeup rate ``k_makeup · (o2_setpoint − cabin_o2)`` (mol/s).

    A **proportional** controller (demand-controlled toward the setpoint): positive when
    ``cabin_o2 < o2_setpoint`` (adds O₂), 0 at the setpoint. It supplies the ``−k_makeup
    · cabin_o2`` restoring force that gives O₂ an attractor with no readout. Not clamped
    — in every standalone scenario ``cabin_o2 ≤ o2_setpoint`` (the consuming crew keeps
    it below), so it only ever adds; an above-setpoint venting clamp is a deferred seam.
    """
    return o2_makeup_gain * (o2_setpoint - cabin_o2)


@dataclass(frozen=True)
class CrewMetabolism:
    """Forced multi-quantity flow — the crew/Phase-6 seam (Step 6).

    Six legs across three quantities, each balanced independently: O₂ ``cabin_o2 →
    metabolic_o2_sink`` (magnitude ``env.get(o2_consumption)·dt``), CO₂
    ``metabolic_co2_source → cabin_co2`` (``env.get(co2_production)·dt``), H₂O
    ``metabolic_h2o_source → cabin_h2o`` (``env.get(h2o_production)·dt``). Each
    quantity's two legs share one magnitude ⇒ CARBON / OXYGEN / WATER each balance
    exactly (``−x + x = 0``). **Forced** (reads ``env``, not a stock); ``flux =
    rate·dt`` is dt-linear (RK4-order-safe, Phase-6-multi-rate-safe). Standalone this
    stands in for the Crew domain; Phase-6 coupling feeds the cabin from the Crew
    domain's own state.
    """

    id: FlowId
    priority: int
    cabin_o2: StockId
    cabin_co2: StockId
    cabin_h2o: StockId
    metabolic_o2_sink: StockId
    metabolic_co2_source: StockId
    metabolic_h2o_source: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        o2 = env.get(O2_CONSUMPTION_VAR) * dt
        co2 = env.get(CO2_PRODUCTION_VAR) * dt
        h2o = env.get(H2O_PRODUCTION_VAR) * dt
        return FlowResult(
            legs=(
                # OXYGEN: crew consumes O₂ out of the cabin.
                Leg(self.cabin_o2, -o2),
                Leg(self.metabolic_o2_sink, o2),
                # CARBON: crew exhales CO₂ into the cabin.
                Leg(self.metabolic_co2_source, -co2),
                Leg(self.cabin_co2, co2),
                # WATER: crew adds humidity to the cabin.
                Leg(self.metabolic_h2o_source, -h2o),
                Leg(self.cabin_h2o, h2o),
            )
        )


@dataclass(frozen=True)
class CO2Scrubber:
    """CARBON flow ``cabin_co2 → boundary.co2_removed`` — the first-order scrubber.

    Removes ``scrub_flux(cabin_co2, k_scrub)·dt`` of CO₂ each step to the monotonic
    ``co2_removed`` sink. Two legs use the same magnitude ⇒ CARBON balances exactly
    (``−R + R = 0``). **Donor-controlled** (the domain's CO₂ restoring force ⇒ ``co2_eq
    = P_co2/k_scrub``); positivity structural (``k_scrub·dt < 1``). ``flux = rate·dt``
    is dt-linear. Because it reads a stock, RK4 ≢ Euler (a tolerance agreement).
    """

    id: FlowId
    priority: int
    cabin_co2: StockId
    co2_removed: StockId
    params: EclssParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        removed = (
            scrub_flux(
                snapshot.stocks[self.cabin_co2].amount,
                co2_scrub_rate=self.params.co2_scrub_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.cabin_co2, -removed),
                Leg(self.co2_removed, removed),
            )
        )


@dataclass(frozen=True)
class Condenser:
    """WATER flow ``cabin_h2o → boundary.humidity_condensate`` — the condenser (Step 6).

    Removes ``condense_flux(cabin_h2o, k_cond)·dt`` of humidity each step to the
    monotonic ``humidity_condensate`` sink. Two legs, one magnitude ⇒ WATER balances
    exactly. **Donor-controlled** (the humidity restoring force ⇒ ``h2o_eq =
    P_h2o/k_cond``); positivity structural (``k_cond·dt < 1``). ``flux = rate·dt`` is
    dt-linear.
    """

    id: FlowId
    priority: int
    cabin_h2o: StockId
    humidity_condensate: StockId
    params: EclssParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        condensed = (
            condense_flux(
                snapshot.stocks[self.cabin_h2o].amount,
                condense_rate=self.params.condense_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.cabin_h2o, -condensed),
                Leg(self.humidity_condensate, condensed),
            )
        )


@dataclass(frozen=True)
class O2Makeup:
    """OXYGEN flow ``boundary.o2_supply → cabin_o2`` — the O₂ regulator (Step 6).

    Adds ``makeup_flux(cabin_o2, k_makeup, o2_setpoint)·dt`` of O₂ each step from the
    unclamped ``o2_supply`` tank. Two legs use the same magnitude ⇒ OXYGEN balances
    exactly (``−S + S = 0``). **Demand-controlled** toward the setpoint (the O₂
    restoring force ⇒ ``o2_eq = o2_setpoint − Con_o2/k_makeup``) — the advisor's fix for
    "O₂ recapitulates Power's constructed-balance problem". Positivity on ``cabin_o2``
    is by well-fed sizing (makeup only adds; the depleting side is ``CrewMetabolism``,
    sized so ``cabin_o2`` never empties). Because it reads a stock, RK4 ≢ Euler. ``flux
    = rate·dt`` is dt-linear.
    """

    id: FlowId
    priority: int
    o2_supply: StockId
    cabin_o2: StockId
    params: EclssParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        supplied = (
            makeup_flux(
                snapshot.stocks[self.cabin_o2].amount,
                o2_makeup_gain=self.params.o2_makeup_gain,
                o2_setpoint=self.params.o2_setpoint,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.o2_supply, -supplied),
                Leg(self.cabin_o2, supplied),
            )
        )
