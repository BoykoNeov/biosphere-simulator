"""Crew flows: forced metabolic draws on finite stores, per-quantity balanced (Step 7).

Three flows of the standalone crew core, each internally balanced **per conserved
quantity** (``assert_flow_balanced`` groups legs by each stock's composition and asserts
every asserted quantity independently). **All three are FORCED** — each reads an intake
*rate* from ``env`` and multiplies by ``dt`` (never reads a store amount), so:

  * positivity is a **well-fed sizing** discipline (the ``LoadDraw`` way — the store
    never empties over the mission), **not** structural (``k·dt < 1`` would need a
    donor-controlled ``k·store`` draw, which these are not); and
  * because no flow reads a stock, the forced-only **RK4 ≡ Euler bit-identity** (``k1 =
    k2 = k3 = k4``) *returns* — the symmetric bookend to ECLSS / Thermal /
    ``SelfDischarge``, which broke it. Crew is the second forced-only domain (after
    Power's two-flow ``BOUNDED_SOC``), so it revives that identity.

The flows:

  * **OxygenConsumption** — ``crew.o2_store → boundary.crew_o2_consumed``, magnitude
    ``env.get(crew_o2_intake)·dt`` (mol). Two legs, one magnitude ⇒ OXYGEN balances
    exactly (``−q + q = 0``). The O₂ leaves to a **decoupled** sink (its atoms end up
    inside exhaled CO₂/H₂O in reality — the atomic coupling deferred to Phase-6
    composition stocks, ECLSS's atom-seam analogue).
  * **FoodMetabolism** — ``crew.food_store → boundary.exhaled_co2 (+f_resp) +
    boundary.fecal_waste (+(1−f_resp))``. The forced food-carbon intake ``q =
    env.get(crew_food_intake)·dt`` (mol) **splits** by ``respired_carbon_fraction`` into
    respired CO₂ and egested feces. **Three legs** (the ``SolarCharge`` fractional-split
    idiom, now on a *mass* quantity), balancing CARBON exactly in intent (``−q + f·q +
    (1−f)·q = 0``). The split is **not** cosmetic: CO₂ and feces route to *different*
    Phase-6 destinations (ECLSS cabin air vs the solid-waste system) — that is what
    justifies naming them as two legs.
  * **WaterBalance** — ``crew.water_store → boundary.crew_humidity (+f_insensible) +
    boundary.urine (+(1−f_insensible))``. The forced water intake ``q =
    env.get(crew_water_intake)·dt`` (kg) **splits** by ``insensible_water_fraction``
    into insensible/respiratory humidity and urine. **Three legs** balancing WATER
    exactly. Again the split routes to *different* Phase-6 destinations (ECLSS cabin
    humidity vs the water-recovery system). Water in == water out here: **metabolic
    water** generated from oxidising food-hydrogen is the atomic-coupling piece deferred
    with composition stocks (Phase 6), not modelled standalone.

**Multi-quantity, like ECLSS (CARBON / OXYGEN / WATER), no cross-quantity conversion.**
Each quantity is conserved independently over its augmented system; the crew does
**not** transmute O₂ into CO₂/H₂O standalone (that is respiration's atomic
stoichiometry, which needs composition stocks — Phase 6). So each store/sink is a
single-quantity pool and the per-quantity ledger balances trivially.

Pure stdlib only. Citations: the fractional split of ingested carbon into respired CO₂
vs fecal loss, and of water intake into insensible loss vs urine, are textbook human
metabolism / nutrition balance (clean-room). The split fractions are
``params/crew.yaml`` (illustrative ``TODO(cite)`` placeholders — NOT NASA BVAD / BioSim
numbers; calibration is Phase 6); the intake *rates* are scenario data
(``scenario.py``).
"""

from dataclasses import dataclass

from domains.crew.stocks import (
    FOOD_INTAKE_VAR,
    O2_INTAKE_VAR,
    WATER_INTAKE_VAR,
)
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class CrewParams:
    """Loader-produced crew metabolic-split coefficients (the physiology fractions).

    Provisional literature-typical placeholders pending the validation gate (see
    ``params/crew.yaml``). Both are dimensionless fractions in ``[0, 1]``, exact-string
    unit-guarded at the loader (neither is a conserved-Quantity canonical unit, so
    neither routes through pint — the ``ChargeParams`` ``charge_efficiency``
    discipline):

      * ``respired_carbon_fraction`` (f_resp) ∈ [0, 1]: the fraction of ingested food
        carbon respired as CO₂; the remainder ``(1 − f_resp)`` is egested as fecal solid
        carbon. The ``SolarCharge`` η_c analogue on a mass quantity.
      * ``insensible_water_fraction`` (f_insensible) ∈ [0, 1]: the fraction of water
        intake leaving as insensible/respiratory humidity; the remainder ``(1 −
        f_insensible)`` leaves as urine.

    Endpoints (0 / 1) collapse one output leg to exactly 0 (a valid degenerate split —
    the flow still emits both legs, the ``SolarCharge`` "three legs even at η_c = 1"
    convention).
    """

    respired_carbon_fraction: float
    insensible_water_fraction: float


def carbon_split(
    food_mol: float, *, respired_carbon_fraction: float
) -> tuple[float, float]:
    """Split ingested food carbon into ``(respired_co2, egested_feces)`` (mol).

    ``respired = f_resp · food``; ``feces = (1 − f_resp) · food``. Plain, deterministic
    arithmetic — the ~1e-15 round-off in ``respired + feces`` vs ``food`` is covered by
    ``assert_flow_balanced``'s relative tolerance, exactly as for ``SolarCharge``'s
    ``charge_split`` (the invariant is determinism, not bit-exact ``Σ == 0``). At
    ``f_resp = 1`` ``feces`` is exactly 0; at ``food = 0`` both are 0.
    """
    respired = respired_carbon_fraction * food_mol
    feces = (1.0 - respired_carbon_fraction) * food_mol
    return respired, feces


def water_split(
    water_kg: float, *, insensible_water_fraction: float
) -> tuple[float, float]:
    """Split water intake into ``(humidity, urine)`` (kg), summing to the input.

    ``humidity = f_insensible · water``; ``urine = (1 − f_insensible) · water``. The
    ``carbon_split`` / ``charge_split`` idiom for the water quantity; the ~1e-15
    round-off is covered by the flow-balance relative tolerance.
    """
    humidity = insensible_water_fraction * water_kg
    urine = (1.0 - insensible_water_fraction) * water_kg
    return humidity, urine


@dataclass(frozen=True)
class OxygenConsumption:
    """OXYGEN flow ``crew.o2_store → boundary.crew_o2_consumed`` — forced O₂ intake.

    Withdraws the forced crew O₂ intake ``q = env.get(crew_o2_intake)·dt`` (mol) from
    the finite O₂ store and deposits **all** of it into the decoupled metabolic-O₂ sink.
    A single magnitude ``q`` in both legs ⇒ OXYGEN balances exactly (``−q + q = 0``).
    **Forced** (reads ``env``, not the store), so positivity is by well-fed sizing (the
    store never empties over the mission) and RK4 ≡ Euler bit-identically. ``flux =
    rate·dt`` is dt-linear (RK4-order-safe, Phase-6-multi-rate-safe).
    """

    id: FlowId
    priority: int
    o2_store: StockId
    o2_consumed: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        q = env.get(O2_INTAKE_VAR) * dt
        return FlowResult(
            legs=(
                Leg(self.o2_store, -q),
                Leg(self.o2_consumed, q),
            )
        )


@dataclass(frozen=True)
class FoodMetabolism:
    """CARBON flow — food carbon split into respired CO₂ + egested feces (forced).

    The forced food-carbon intake ``q = env.get(crew_food_intake)·dt`` (mol) is
    withdrawn from the finite food store and **split**: ``f_resp·q`` respired as CO₂,
    ``(1−f_resp)·q`` egested as fecal solid carbon. Three legs balance CARBON exactly in
    intent (``−q + f·q + (1−f)·q = 0``). Always three legs (zero-amount at ``f_resp =
    1`` / an empty step); the two output legs route to *different* Phase-6 destinations
    (cabin CO₂ vs solid-waste). **Forced**; ``flux = rate·dt`` is dt-linear.
    """

    id: FlowId
    priority: int
    food_store: StockId
    exhaled_co2: StockId
    fecal_waste: StockId
    params: CrewParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        q = env.get(FOOD_INTAKE_VAR) * dt
        respired, feces = carbon_split(
            q, respired_carbon_fraction=self.params.respired_carbon_fraction
        )
        return FlowResult(
            legs=(
                Leg(self.food_store, -q),
                Leg(self.exhaled_co2, respired),
                Leg(self.fecal_waste, feces),
            )
        )


@dataclass(frozen=True)
class WaterBalance:
    """WATER flow ``crew.water_store → crew_humidity (+f_ins) + urine (+(1−f_ins))``.

    The forced water intake ``q = env.get(crew_water_intake)·dt`` (kg) is withdrawn from
    the finite water store and **split**: ``f_insensible·q`` lost as
    insensible/respiratory humidity, ``(1−f_insensible)·q`` as urine. Three legs balance
    WATER exactly. The two output legs route to *different* Phase-6 destinations (cabin
    humidity vs water-recovery). Water in == water out (metabolic water from food
    oxidation is the deferred atomic-coupling piece). **Forced**; ``flux = rate·dt`` is
    dt-linear.
    """

    id: FlowId
    priority: int
    water_store: StockId
    crew_humidity: StockId
    urine: StockId
    params: CrewParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        q = env.get(WATER_INTAKE_VAR) * dt
        humidity, urine = water_split(
            q, insensible_water_fraction=self.params.insensible_water_fraction
        )
        return FlowResult(
            legs=(
                Leg(self.water_store, -q),
                Leg(self.crew_humidity, humidity),
                Leg(self.urine, urine),
            )
        )
