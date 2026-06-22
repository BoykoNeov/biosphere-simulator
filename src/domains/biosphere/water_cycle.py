"""Water cycle closure: condensation + recycling (Phase-3 Step 3; WATER only).

The one cycle still open. Carbon (Phase-2 Steps 2–5) and nitrogen (Step 6) close
internally; water did not — Phase 1/2 transpiration drained ``soil_water`` to a
``vapor_sink`` BOUNDARY (water *out*) and irrigation refilled it from a ``water_source``
BOUNDARY (water *in*), so the sealed chamber still exchanged water with the outside.
Step 3 closes both crossings into a ring, mirroring how carbon dropped
``co2_atmos``/``co2_resp`` for the finite ``carbon_pool``:

```
soil_water (soil) --Transpiration--> water_vapor (atmosphere)
water_vapor (atmosphere) --Condensation--> condensate (water)
condensate (water) --Recycling--> soil_water (soil)
```

``Transpiration`` (plants-owned, retargeted via ``ChamberWiring.vapor_target``) feeds
the ring; this module holds the two **new** closing flows — the condenser and the
recycler — exactly as ``mineralization.py`` holds the two coupled flows that close the
nitrogen loop:

* **Condensation** — ``water_vapor -> condensate`` (Σ legs = 0). First-order
  donor-controlled: ``condensation = k_cond · water_vapor`` (kg day⁻¹), self-limiting
  → 0 as vapor → 0 (the decomposition / mineralization positivity pattern: a clamped
  POOL withdrawal ∝ its own start-of-step amount, so ``k_cond·dt < 1`` keeps the Euler
  backstop unfired).

* **Recycling** — ``condensate -> soil_water`` (Σ legs = 0). First-order
  donor-controlled: ``recycling = k_rec · condensate`` (kg day⁻¹), self-limiting → 0 as
  condensate → 0 (the same structural positivity). Returns the recovered water to the
  rooting zone.

Both are **single-currency WATER** (``soil_water`` / ``water_vapor`` / ``condensate``
are all ``{WATER: 1}``), so the every-step conservation gate folds them exactly like the
Phase-1 transpiration / irrigation flows — **no core change**. Sealed-chamber only
(``water_vapor`` / ``condensate`` exist only when sealed); appended to the registry like
``Decomposition`` / ``Mineralization``. The closed loop conserves total water
``soil_water + water_vapor + condensate`` (each leg is a balanced 1:1 WATER transfer),
distributed around the ring — **emergent from stock coupling, no control code**: each
flow names a sibling compartment's shared stock and the resolver (#16) cannot tell
shared from forcing.

**Citation framing (the honest part).** First-order means vapor condenses *regardless of
humidity* — **wrong for natural atmospheric condensation** (which needs supersaturation;
that saturation/dew-point refinement needs chamber volume + temperature coupling with
zero architectural payoff here), but **right for an engineered condenser +
water-recovery loop** — a dehumidifier / condensing heat-exchanger at fixed clearance,
which is what a sealed bioregenerative life-support chamber actually has (CELSS;
Biosphere 2 condensate management). The two rates ship as ``TODO(cite)``
literature-typical placeholders pending a later validation gate — consistent with the
season's documented "machinery, not validated behaviour" honesty and the decomposition /
mineralization first-order precedent.

Pure stdlib only. Citations: the first-order donor-controlled transfer is the standard
linear-reservoir form (e.g. Olson 1963 for decay; the same algebra governs an engineered
condenser at fixed clearance); MacElroy, R.D. & Bredt, J. (1984), "Current concepts and
future directions of CELSS", Adv. Space Res. 4(12):221–229 (closed-loop water recovery
in controlled ecological life-support); Nelson, M. et al. (1993), "Using a closed
ecological system to study Earth's biosphere: initial results from Biosphere 2",
BioScience 43(4):225–236 (condensate management in a sealed chamber). Provisional
``TODO(cite)`` rate values pending the validation gate (see
``params/water_cycle.yaml``), clean-room.
"""

from dataclasses import dataclass

from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class WaterCycleParams:
    """Loader-produced water-cycle parameters: the two first-order rates.

    Provisional literature-typical placeholders pending the validation gate (see
    ``params/water_cycle.yaml``). Zero rates are valid (no condensation / no recycling);
    negative is rejected at the loader.
    """

    # first-order condensation rate, water_vapor → condensate (kg / kg / day)
    condensation_rate: float
    # first-order recycling rate, condensate → soil_water (kg / kg / day)
    recycling_rate: float


def condensation_flux(water_vapor: float, *, condensation_rate: float) -> float:
    """Daily condensation ``condensation_rate · water_vapor`` (kg day⁻¹).

    First-order donor-controlled (the engineered-condenser form), so it → 0 as the vapor
    pool → 0 (positivity is structural — the decomposition / mineralization
    self-limiting pattern). The condensed water enters the ``condensate`` POOL (the
    :class:`Condensation` flow).
    """
    return condensation_rate * water_vapor


def recycling_flux(condensate: float, *, recycling_rate: float) -> float:
    """Daily recycling ``recycling_rate · condensate`` (kg day⁻¹).

    First-order donor-controlled, so it → 0 as the condensate pool → 0 (positivity is
    structural — the same self-limiting pattern). The recovered water returns to the
    ``soil_water`` POOL (the :class:`Recycling` flow).
    """
    return recycling_rate * condensate


@dataclass(frozen=True)
class Condensation:
    """WATER flow ``water_vapor -> condensate`` (balanced, P3 Step 3).

    Condenses ``condensation_flux(water_vapor, k_cond)·dt`` of water from the
    atmosphere's ``water_vapor`` POOL into the water compartment's ``condensate`` POOL
    each step — the middle leg of the closed water ring. Single-currency WATER (both
    pools are ``{WATER: 1}``), so the gate folds it identically to the Phase-1
    transpiration flow.
    Self-limiting (∝ the vapor pool's amount), so ``rationed == 0`` is structural
    (``k_cond·dt < 1``). Sealed-only; ``flux = daily·dt`` (dt-linear).
    """

    id: FlowId
    priority: int
    water_vapor: StockId
    condensate: StockId
    params: WaterCycleParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        condensed = (
            condensation_flux(
                snapshot.stocks[self.water_vapor].amount,
                condensation_rate=self.params.condensation_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.water_vapor, -condensed),
                Leg(self.condensate, condensed),
            )
        )


@dataclass(frozen=True)
class Recycling:
    """WATER flow ``condensate -> soil_water`` (balanced, P3 Step 3).

    Returns ``recycling_flux(condensate, k_rec)·dt`` of recovered water from the water
    compartment's ``condensate`` POOL back to the soil's ``soil_water`` POOL each step —
    closing the ring ``soil_water → water_vapor → condensate → soil_water`` that Phase
    1/2 left open (transpiration to a ``vapor_sink`` BOUNDARY, irrigation from a
    ``water_source`` BOUNDARY). Single-currency WATER (both pools are ``{WATER: 1}``).
    Self-limiting (∝ the condensate pool's amount), so ``rationed == 0`` is structural
    (``k_rec·dt < 1``). Sealed-only; ``flux = daily·dt`` (dt-linear).
    """

    id: FlowId
    priority: int
    condensate: StockId
    soil_water: StockId
    params: WaterCycleParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        recycled = (
            recycling_flux(
                snapshot.stocks[self.condensate].amount,
                recycling_rate=self.params.recycling_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.condensate, -recycled),
                Leg(self.soil_water, recycled),
            )
        )
