"""The Power stock-id catalog + constructors (Phase 5, P5.2).

The three stocks the standalone power system needs, all holding the **one** conserved
energy currency — ``Quantity.ENERGY`` in joules (J). Electricity vs heat is a *form*
distinction carried by **which stock** holds the joules, **not** a separate
``Quantity`` (a per-quantity ledger cannot balance an electricity→heat conversion
across two of its quantities — see ``docs/plans/phase-5-sibling-domains.md`` P5.1(c)):

  * ``power.battery`` — **POOL**, stored *electrical* energy (state of charge). The one
    stock arbitration's backstop guards; loads draw from it, solar charges it.
  * ``boundary.solar_source`` — **BOUNDARY source**, ``unclamped`` (#13): the
    electrical supply. Panel conversion is folded into the forcing (incident sunlight is
    *not* a tracked stock — the biosphere treats light as forcing too). Its
    negative-going amount is cumulative supply bookkeeping. *Nuclear/grid input is the
    same shape with a flat schedule — a documented seam, not built.*
  * ``boundary.waste_heat`` — **BOUNDARY sink**, *thermal* energy: receives every
    degraded joule (the charge-conversion loss, the dissipative load, and — when the
    opt-in ``SelfDischarge`` flow is present, P5.5 — the standing self-discharge leak).
    Never withdrawn from, so it is **monotonic by construction** ⇒ a free
    *heat-generated* diagnostic
    (the "usefulness is not conserved" accumulator, roadmap line 50). This is the **seam
    the Thermal sibling later moves inward** into a real heat stock + radiator rejection
    — the water-cycle-closure analogue for energy.

There is **no pass-through "bus" POOL**: a near-zero pass-through pool cannot source a
flow under the arbitration backstop (it scales withdrawals against the *start-of-step*
amount — the Step-11 ``plant_c``-buffer lesson). So solar deposits **directly into** the
battery and loads draw **directly from** it — the battery *is* the bus + storage.

**No capacity parameter (deliberate, not an omission).** POOL stocks have **no upper
clamp** in this engine (arbitration only guards withdrawals from going negative; there
is no max-fill check). So a battery "capacity" is not a flow coefficient — it is
**sizing / scenario data** (the initial amount + the well-fed sizing that keeps the SOC
bounded), landed with ``build_power`` in Step 3, not here.

Boundary stocks live under the shared ``boundary`` namespace (built by
``simcore.boundary``); only ``power.battery`` carries the ``power`` domain. Ids are
ASCII so Python's str sort matches the future Rust UTF-8 byte sort (#15). Pure stdlib.
"""

from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock

# The Power domain id — a top-level sibling of ``biosphere`` (cross-domain coupling is
# Phase 6; here Power is verified standalone). Only ``power.battery`` carries it; the
# source/sink reservoirs carry the shared ``boundary`` domain (``simcore.boundary``).
POWER_DOMAIN: DomainId = DomainId("power")

# --- stock ids --------------------------------------------------------------
BATTERY: StockId = StockId("power.battery")
SOLAR_SOURCE: StockId = StockId("boundary.solar_source")
WASTE_HEAT: StockId = StockId("boundary.waste_heat")

# --- forcing var names (resolved through env.get, #16) -----------------------
# Power *rates* in watts (W = J/s); a flow multiplies by ``dt`` (seconds) to get the
# per-step joule increment (the increment-form contract — dt-linear, RK4-order-safe).
# Wired as forcing schedules (the day/night table, Step 3) or — under Phase-6 coupling —
# a sibling domain's shared stock; the reader cannot tell (#16).
SOLAR_POWER_VAR = "solar_power"  # W, instantaneous solar electrical supply
LOAD_POWER_VAR = "load_power"  # W, instantaneous dissipative load demand


def battery_stock(amount: float) -> Stock:
    """The stored-electrical-energy POOL ``power.battery`` (ENERGY, J).

    A single-currency ENERGY POOL (``{ENERGY: 1.0}``). ``amount`` is the initial state
    of charge in joules. Arbitration may throttle draws against it (the one guarded
    stock); it is never zeroed-with-loss (POOLs are not extinction-eligible).
    """
    return Stock(
        id=BATTERY,
        domain=POWER_DOMAIN,
        quantity=Quantity.ENERGY,
        unit=canonical_unit(Quantity.ENERGY),
        amount=amount,
        kind=StockKind.POOL,
    )
