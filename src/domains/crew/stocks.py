"""The Crew stock-id catalog + constructors (Phase 5, Step 7 — the Crew sibling).

Crew is the last forward-pointer sibling and the first **net-consumer / open-loop**
domain. Where the biosphere *loops*, Power *oscillates*, and Thermal / ECLSS reach a
*steady state*, standalone Crew just **runs down**: forced metabolic draws deplete a set
of **finite provisioned stores** (food / water / O₂) and route the products to boundary
sinks. There is **no restoring force and no attractor** — the standalone behaviour is
inexorable depletion, and *that incompleteness is the argument for Phase-6 closure* (the
crew starves once the stores empty unless the biosphere + ECLSS regenerate them).

**Crew is the real version of ECLSS's forced ``CrewMetabolism`` stand-in.** ECLSS (Step
6) already defined this seam from the *cabin's* side — a 6-leg forced flow with
``boundary.metabolic_o2_sink`` / ``metabolic_co2_source`` / ``metabolic_h2o_source``
standing in for the absent crew. Crew is the **other side** of that seam: it *owns* the
crew's consumption/production. This is the exact analogue of "Thermal builds the
receiver Phase 6 wires Power's ``waste_heat`` into" — **Phase 6 deletes ECLSS's stand-in
``CrewMetabolism`` and wires Crew's outputs into the cabin stocks** (CO₂ →
``cabin_co2``, humidity → ``cabin_h2o``, O₂ intake ← ``cabin_o2``). Crew is a
**superset** of that seam: its urine, fecal waste, and O₂-consumed legs route to *other*
Phase-6 systems (water recovery, solid-waste, the atomic-coupling sink), not the cabin
air.

**Stores (POOL, finite — the provisioned mission).** Unlike Power's / ECLSS's
*unclamped* boundary sources, the crew's consumables are **finite POOL stocks** that
deplete monotonically. They are the stocks arbitration's backstop guards; ``rationed ==
0`` holds by **well-fed sizing** (the mission is shorter than the closed-form
time-to-depletion — the ``LoadDraw`` discipline, not a structural ``k·dt < 1`` claim,
because the draws are *forced*, not donor-controlled). Phase 6 replaces these stores
with regenerative sources (``cabin_o2`` / water recovery / biosphere food) — the inward
move.

  * ``crew.food_store`` — **POOL** (CARBON, mol): provisioned food carbon. Drawn by
    ``FoodMetabolism``, which splits it into respired CO₂ + egested feces.
  * ``crew.water_store`` — **POOL** (WATER, kg): provisioned potable water. Drawn by
    ``WaterBalance``, which splits it into insensible humidity + urine.
  * ``crew.o2_store`` — **POOL** (OXYGEN, mol): provisioned O₂ (tanks). Drawn by
    ``OxygenConsumption`` into the decoupled metabolic-O₂ sink.

Boundary reservoirs (all under the shared ``boundary`` namespace; ids chosen distinct
from the biosphere's *and* ECLSS's so Phase-6 assembly cannot collide). Each is a
**monotonic sink** ⇒ a free cumulative-output diagnostic (CO₂ exhaled, feces egested,
water condensed, urine produced, O₂ consumed):

  * ``boundary.exhaled_co2`` — respired CO₂ (CARBON). *→ Phase 6: wires to ECLSS
    ``cabin_co2``.*
  * ``boundary.fecal_waste`` — egested solid carbon (CARBON). *→ Phase 6: solid-waste
    system.*
  * ``boundary.crew_humidity`` — respiration + perspiration (WATER). *→ Phase 6: ECLSS
    ``cabin_h2o``.*
  * ``boundary.urine`` — urine (WATER). *→ Phase 6: water-recovery system.*
  * ``boundary.crew_o2_consumed`` — the metabolic-O₂ sink (OXYGEN). Decoupled: real
    respiration binds inhaled O₂ into exhaled CO₂/H₂O, so this sink and the CO₂/H₂O
    sources are **not** atom-coupled here (the honest analogue of ECLSS's permanent
    ``boundary.space`` / decoupled crew seam) — the atomic coupling ``C_food + O₂ → CO₂
    + H₂O`` needs composition stocks and is a **Phase-6** act.

Only the three ``crew.*`` stores carry the ``crew`` domain; the reservoirs carry the
shared ``boundary`` domain. Ids are ASCII so Python's str sort matches the future Rust
UTF-8 byte sort (#15). Pure stdlib.
"""

from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock

# The Crew domain id — a top-level sibling of ``biosphere`` / ``power`` / ``thermal`` /
# ``eclss`` (cross-domain coupling is Phase 6; here Crew is verified standalone). Only
# the three ``crew.*`` stores carry it; the reservoirs carry the shared ``boundary``
# domain.
CREW_DOMAIN: DomainId = DomainId("crew")

# --- store stock ids (the finite provisioned consumables) -------------------
FOOD_STORE: StockId = StockId("crew.food_store")
WATER_STORE: StockId = StockId("crew.water_store")
O2_STORE: StockId = StockId("crew.o2_store")

# --- boundary reservoir ids (monotonic output sinks) ------------------------
EXHALED_CO2: StockId = StockId("boundary.exhaled_co2")
FECAL_WASTE: StockId = StockId("boundary.fecal_waste")
CREW_HUMIDITY: StockId = StockId("boundary.crew_humidity")
URINE: StockId = StockId("boundary.urine")
CREW_O2_CONSUMED: StockId = StockId("boundary.crew_o2_consumed")

# --- forcing var names (resolved through env.get, #16) -----------------------
# The forced crew intake *rates* in canonical-unit/s; each flow multiplies by ``dt``
# (seconds) for the per-step increment (the increment-form contract — dt-linear,
# RK4-order-safe). Wired as constant forcing schedules (Step 7) or — under Phase-6
# coupling — a sibling domain's shared stock; the reader cannot tell (#16).
O2_INTAKE_VAR = "crew_o2_intake"  # mol/s, crew O₂ drawn from the O₂ store
FOOD_INTAKE_VAR = "crew_food_intake"  # mol/s (carbon), food drawn from the food store
WATER_INTAKE_VAR = "crew_water_intake"  # kg/s, water drawn from the water store


def _store(stock_id: StockId, quantity: Quantity, amount: float) -> Stock:
    """A finite, depleting provisioned-consumable POOL (its 1:1 default composition).

    ``amount`` is the initial provisioned inventory in the quantity's canonical unit.
    Arbitration may throttle withdrawals against it (a guarded stock — the well-fed
    sizing keeps it from emptying); it is never zeroed-with-loss (POOLs are not
    extinction-eligible).
    """
    return Stock(
        id=stock_id,
        domain=CREW_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
    )


def food_store_stock(amount: float) -> Stock:
    """The provisioned-food-carbon POOL ``crew.food_store`` (CARBON, mol)."""
    return _store(FOOD_STORE, Quantity.CARBON, amount)


def water_store_stock(amount: float) -> Stock:
    """The provisioned-potable-water POOL ``crew.water_store`` (WATER, kg)."""
    return _store(WATER_STORE, Quantity.WATER, amount)


def o2_store_stock(amount: float) -> Stock:
    """The provisioned-O₂ POOL ``crew.o2_store`` (OXYGEN, mol)."""
    return _store(O2_STORE, Quantity.OXYGEN, amount)
