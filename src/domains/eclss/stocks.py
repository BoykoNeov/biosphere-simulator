"""The ECLSS stock-id catalog + constructors (Phase 5, Step 6 — the Atmosphere sibling).

ECLSS is the **cabin-air medium** — the shared compartment both the Crew sibling (next)
and, in Phase 6, the biosphere couple into. It is the **first multi-quantity sibling**:
where Power and Thermal carried only ``Quantity.ENERGY``, ECLSS tracks **three**
already-asserted conserved mass quantities at once —

  * ``eclss.cabin_o2`` — **POOL** (OXYGEN, mol): breathable O₂ in the cabin air. The one
    O₂ stock arbitration's backstop guards; crew metabolism draws it down, the O₂-makeup
    regulator tops it up toward a setpoint.
  * ``eclss.cabin_co2`` — **POOL** (CARBON, mol): metabolic CO₂ accumulating in the
    cabin. The scrubber removes it first-order; the one CO₂ stock the backstop guards.
  * ``eclss.cabin_h2o`` — **POOL** (WATER, kg): cabin humidity (respiration +
    perspiration). The condenser removes it first-order.

Each cabin pool is **single-quantity** (its 1:1 ``{quantity: 1.0}`` default
composition): no standalone flow **converts** between species (no O₂↔CO₂ transmutation —
that is crew respiration's atomic stoichiometry, which needs composition stocks and
arrives with **Phase-6 crew coupling**), so single-quantity pools balance per-quantity
trivially and composition (CO₂ = ``{CARBON:1, OXYGEN:2}``) stays a **flagged Phase-6
seam**.

Boundary reservoirs (all under the shared ``boundary`` namespace; ids chosen distinct
from the biosphere's ``vapor_sink`` / ``water_source`` / ``co2_resp`` / … so Phase-6
assembly cannot collide):

  * ``boundary.o2_supply`` — **BOUNDARY source**, ``unclamped`` (#13): the O₂-makeup
    tank (electrolysis / stored O₂). A documented forcing seam — Phase 6 couples it to
    Power / water electrolysis. Its (negative-going) amount is cumulative supply
    bookkeeping.
  * ``boundary.co2_removed`` — **BOUNDARY sink**: scrubber product (CDRA/LiOH). Never
    withdrawn from ⇒ **monotonic by construction** ⇒ a free *CO₂-scrubbed* diagnostic.
  * ``boundary.humidity_condensate`` — **BOUNDARY sink**: condenser product. Monotonic ⇒
    a free *water-recovered* diagnostic.
  * ``boundary.metabolic_o2_sink`` / ``boundary.metabolic_co2_source`` /
    ``boundary.metabolic_h2o_source`` — the **crew-metabolism seam** reservoirs. Real
    respiration is ``C_food + O₂ → CO₂ + H₂O``, so the O₂ crew inhale ends up *inside*
    the exhaled CO₂/H₂O; without composition stocks these three reservoirs are
    **decoupled** (metabolic O₂ leaves to a sink; CO₂/H₂O carbon+water enter from
    separate sources). This is the honest analogue of Thermal's permanent
    ``boundary.space``: standalone ECLSS conserves **each quantity** over its augmented
    system every step (the payload) but does **not** tie the crew's atoms together —
    that atomic coupling is a **Phase-6** act (crew coupling + composition stocks).
    "Closed" is decision #13's *augmented*-system sense.

Only the three ``cabin_*`` stocks carry the ``eclss`` domain; the reservoirs carry the
shared ``boundary`` domain. Ids are ASCII so Python's str sort matches the future Rust
UTF-8 byte sort (#15). Pure stdlib.
"""

from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock

# The ECLSS domain id — a top-level sibling of ``biosphere`` / ``power`` / ``thermal``
# (cross-domain coupling is Phase 6; here ECLSS is verified standalone). Only the three
# ``cabin_*`` stocks carry it; the reservoirs carry the shared ``boundary`` domain.
ECLSS_DOMAIN: DomainId = DomainId("eclss")

# --- cabin stock ids (the medium) -------------------------------------------
CABIN_O2: StockId = StockId("eclss.cabin_o2")
CABIN_CO2: StockId = StockId("eclss.cabin_co2")
CABIN_H2O: StockId = StockId("eclss.cabin_h2o")

# --- boundary reservoir ids -------------------------------------------------
O2_SUPPLY: StockId = StockId("boundary.o2_supply")
CO2_REMOVED: StockId = StockId("boundary.co2_removed")
HUMIDITY_CONDENSATE: StockId = StockId("boundary.humidity_condensate")
METABOLIC_O2_SINK: StockId = StockId("boundary.metabolic_o2_sink")
METABOLIC_CO2_SOURCE: StockId = StockId("boundary.metabolic_co2_source")
METABOLIC_H2O_SOURCE: StockId = StockId("boundary.metabolic_h2o_source")

# --- forcing var names (resolved through env.get, #16) -----------------------
# The forced crew-metabolism *rates* in canonical-unit/s; the CrewMetabolism flow
# multiplies each by ``dt`` (seconds) for the per-step increment (the increment-form
# contract — dt-linear, RK4-safe). Wired as constant forcing schedules (Step 6) or —
# under Phase-6 coupling — the Crew domain's own state; the reader cannot tell (#16).
O2_CONSUMPTION_VAR = "o2_consumption"  # mol/s, crew O₂ intake out of the cabin
CO2_PRODUCTION_VAR = "co2_production"  # mol/s, crew CO₂ exhaled into the cabin
H2O_PRODUCTION_VAR = (
    "h2o_production"  # kg/s, crew humidity (respiration + perspiration)
)


def _cabin_pool(stock_id: StockId, quantity: Quantity, amount: float) -> Stock:
    """A single-quantity cabin-air POOL (its 1:1 default composition).

    ``amount`` is the initial cabin inventory in the quantity's canonical unit.
    Arbitration may throttle withdrawals against it (a guarded stock); it is never
    zeroed-with-loss (POOLs are not extinction-eligible).
    """
    return Stock(
        id=stock_id,
        domain=ECLSS_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
    )


def cabin_o2_stock(amount: float) -> Stock:
    """The breathable-O₂ cabin POOL ``eclss.cabin_o2`` (OXYGEN, mol)."""
    return _cabin_pool(CABIN_O2, Quantity.OXYGEN, amount)


def cabin_co2_stock(amount: float) -> Stock:
    """The metabolic-CO₂ cabin POOL ``eclss.cabin_co2`` (CARBON, mol)."""
    return _cabin_pool(CABIN_CO2, Quantity.CARBON, amount)


def cabin_h2o_stock(amount: float) -> Stock:
    """The cabin-humidity POOL ``eclss.cabin_h2o`` (WATER, kg)."""
    return _cabin_pool(CABIN_H2O, Quantity.WATER, amount)
