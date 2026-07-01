"""The Thermal stock-id catalog + constructors (Phase 5, Step 5 — the Thermal sibling).

Thermal is the sibling that reveals **where Power's ``waste_heat`` "somewhere" actually
is**. Power (P5.2) dumped every degraded joule into a terminal ``boundary.waste_heat``
BOUNDARY sink — a deliberate *temporary seam* standing in for "heat goes somewhere".
Thermal makes that somewhere concrete: an **in-system thermal POOL** (``thermal.node``)
with a temperature, plus a **radiator** rejecting heat to the one **permanent, true**
boundary — deep space. All three stocks hold the *same* conserved currency
``Quantity.ENERGY`` (J); "thermal" vs "electrical" is a **form** carried by *which
stock* holds the joules, **not** a separate ``Quantity`` (the P5.1(c) argument — a
per-quantity ledger cannot balance a form conversion across two of them).

  * ``thermal.node`` — **POOL**, stored *sensible heat* (J), **referenced to
    ``T_space``**: its amount is ``Q = C·(T − T_space) ≥ 0`` (``C`` = heat capacity,
    J/K), so temperature is the derived readout ``T = T_space + Q/C`` (a pure function
    of the amount, computed at evaluate-time — **not** a stock, **not** an aux
    accumulator like the biosphere's DVS). Referencing to ``T_space`` is load-bearing:
    it puts the floor (``Q = 0``) at ``T = T_space`` where the radiator's ``T⁴ −
    T_space⁴`` driving term is exactly 0, so radiation **self-limits at the floor** and
    cannot pull ``Q`` negative (structural positivity at the bottom; a *warm* reference
    would break this). The one stock the backstop guards.
  * ``boundary.heat_source`` — **BOUNDARY source**, ``unclamped`` (#13): the forced heat
    input. Standalone, this is the stand-in for Power's electrical dissipation (a
    documented forcing, the way Power treats solar as forcing); **Phase 6 rewires
    Power's dissipation legs to feed ``thermal.node``** — the "inward move" is a Phase-6
    wiring act, and standalone Thermal builds the *receiver*. Its (negative-going)
    amount is cumulative supply bookkeeping.
  * ``boundary.space`` — **BOUNDARY sink**, deep space: receives every radiated joule.
    **This boundary is permanent and cannot be moved inward** — in vacuum, radiation to
    space is the *only* heat-rejection mode, and the joules genuinely leave forever. So
    standalone Thermal does **not** "close" anything (unlike Phase-3's water cycle);
    "closed" is decision #13's *augmented*-system sense (stocks + boundary reservoirs
    balance every step). Never withdrawn from ⇒ **monotonic by construction** ⇒ a free
    *heat-rejected* diagnostic.

**No temperature/heat stock beyond the node.** The node *is* the thermal mass; its
amount is its heat content and ``T`` is read off it. There is no separate temperature
state to keep in sync — the single-currency-J discipline (the biosphere's "one stock,
one quantity" carried to energy).

Boundary stocks live under the shared ``boundary`` namespace (built by
``simcore.boundary``); only ``thermal.node`` carries the ``thermal`` domain. Ids are
ASCII so Python's str sort matches the future Rust UTF-8 byte sort (#15). Pure stdlib.
"""

from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock

# The Thermal domain id — a top-level sibling of ``biosphere`` / ``power`` (cross-domain
# coupling is Phase 6; here Thermal is verified standalone). Only ``thermal.node``
# carries it; the source/sink reservoirs carry the shared ``boundary`` domain
# (simcore.boundary).
THERMAL_DOMAIN: DomainId = DomainId("thermal")

# --- stock ids --------------------------------------------------------------
NODE: StockId = StockId("thermal.node")
HEAT_SOURCE: StockId = StockId("boundary.heat_source")
SPACE: StockId = StockId("boundary.space")

# --- forcing var names (resolved through env.get, #16) -----------------------
# The forced heat *rate* in watts (W = J/s); a flow multiplies by ``dt`` (seconds) to
# get the per-step joule increment (the increment-form contract — dt-linear, RK4-safe).
# Wired as a constant forcing schedule (Step 5) or — under Phase-6 coupling — Power's
# dissipation legs feeding the node; the reader cannot tell (#16).
HEAT_LOAD_VAR = "heat_load"  # W, instantaneous forced heat input into the node


def node_stock(amount: float) -> Stock:
    """The sensible-heat POOL ``thermal.node`` (ENERGY, J), referenced to T_space.

    A single-currency ENERGY POOL (``{ENERGY: 1.0}``). ``amount`` is the initial
    sensible heat ``Q = C·(T − T_space)`` in joules (``0`` ⇒ node starts at ``T =
    T_space``, the radiator floor). Arbitration may throttle the radiator's withdrawal
    against it (the one guarded stock); it is never zeroed-with-loss (POOLs are not
    extinction-eligible). Temperature is the derived readout ``T = T_space + amount/C``
    (see ``domains.thermal.flows.temperature``), not stored here.
    """
    return Stock(
        id=NODE,
        domain=THERMAL_DOMAIN,
        quantity=Quantity.ENERGY,
        unit=canonical_unit(Quantity.ENERGY),
        amount=amount,
        kind=StockKind.POOL,
    )
