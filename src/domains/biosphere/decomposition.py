"""Litter decomposition: the first decomposer process (Phase-2 Step 4; CARBON only).

The producer half of the chamber (gas exchange, Steps 2/3) now has its mirror: dead
organic carbon re-enters the cycle. Senescence sheds organ carbon into a finite
``litter_carbon`` POOL (Step 4 promotes the Phase-1 ``litter_sink`` BOUNDARY to a live
pool, exactly as Step 2 promoted ``co2_atmos`` to a finite ``carbon_pool``); this module
holds the **decomposition flow** that transfers decaying litter carbon into microbial
biomass.

* **Decomposition** — ``litter_carbon -> microbial_carbon`` (Σ legs = 0). First-order
  donor-controlled decay (Olson 1963): ``decay = k · litter_carbon`` (mol C day⁻¹),
  proportional to the standing litter, so it → 0 as litter → 0 — positivity is
  **structural** (the senescence/respiration self-limiting pattern: a clamped POOL
  withdrawal ∝ the pool's own start-of-step amount, so ``k·dt < 1`` keeps the Euler
  backstop unfired). Both pools are **pure carbon** (``{CARBON: 1}``), so this is a
  single-currency CARBON flow — no core change, exactly the Phase-1 kind.

**Why no oxygen here — the deliberate Step-4/Step-5 split (P2.3).** Aerobic
decomposition *is* microbial respiration and genuinely consumes O₂ (``organic-C + O₂ →
CO₂``); in this model that is not optional but **gate-forced** — the chamber's CO₂ pool
is ``{CARBON: 1, OXYGEN: 2}``, so any CO₂ released into it drags 2 oxygens that
pure-carbon litter cannot supply (the P2.1 composition gate would hard-fail), and those
oxygens can only come from the O₂ pool. Step 4 therefore ships *only* the carbon-only
``litter → microbial`` transfer; the CO₂-releasing, O₂-consuming microbial respiration
(``microbial_C + O₂ → CO₂``) is **Step 5** — a pure *addition*, with nothing here torn
up. This honors the plan's "Decomposition's carbon/nitrogen transfers themselves need no
O₂" (P2.3) and mirrors the producer rhythm (Step 2's open draw-down → Step 3's closed
loop).

**Scope refinement vs the plan wording (advisor-reviewed, like Steps 2/3).** The Step-4
one-liner reads "decomposition to microbial biomass + CO₂"; the "+ CO₂" is the part that
is ahead of itself (it needs O₂, hence Step 5). Step 4 is the carbon-only transfer.
Consequence: under this split ``microbial_carbon`` only **grows** this step (decay
deposits; nothing withdraws until Step 5's respiration drains it) — an intentional
intermediate, exactly like ``plant_n`` only growing in Phase 1. **Deferred seams:**
microbial death / turnover (``microbial_C → litter_C`` recycling), microbe-explicit
Michaelis kinetics (``Vmax · microbial_C · litter/(K_m + litter)``, which would need a
microbial seed and active decomposers — first-order donor control is the right Step-4
minimal pick), and the CO₂/O₂-coupled microbial respiration (Step 5).

Pure stdlib only. Citation: Olson, J.S. (1963), "Energy storage and the balance of
producers and decomposers in ecological systems", Ecology 44(2):322–331 (first-order
litter-decay / single-exponential decomposition model; clean-room).
"""

from dataclasses import dataclass

from domains.biosphere.chamber import oxygen_limitation_factor
from domains.biosphere.humification import (
    HumificationParams,
    respired_and_stabilized,
)
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class DecompositionParams:
    """Loader-produced decomposition parameters: the first-order litter decay rate.

    Provisional literature-typical placeholder pending the Phase-2 validation gate (see
    ``params/decomposition.yaml``). A zero rate is valid (no decomposition); negative is
    rejected at the loader.
    """

    decomposition_rate: float  # k, first-order litter decay (mol C / mol C / day)


def decomposition_flux(litter_c: float, *, decomposition_rate: float) -> float:
    """Daily litter decomposition: ``decomposition_rate · litter_c`` (mol C day⁻¹).

    First-order donor-controlled decay (Olson 1963): proportional to the standing litter
    carbon, so it → 0 as litter → 0 (positivity is structural — the senescence /
    respiration self-limiting pattern). The decayed carbon is transferred to microbial
    biomass (no oxygen — the CO₂-releasing microbial respiration is Step 5).
    """
    return decomposition_rate * litter_c


@dataclass(frozen=True)
class Decomposition:
    """CARBON+OXYGEN flow ``litter_carbon + o2_pool → carbon_pool + microbial_carbon``.

    Decays ``decomposition_flux(litter_carbon, k)·f_O2·dt`` of carbon out of the litter
    POOL each step and **partitions it**: ``litter_respired_fraction`` is lost as CO₂
    (drawing an equal amount of O₂ at PQ=1) and the remainder is stabilised into
    microbial biomass. The draw self-limits two ways, so ``rationed == 0`` is
    structural:
    in the substrate (∝ the litter pool's own start-of-step amount, ``k·dt < 1``) and in
    O₂ (the ``f_O2`` Monod factor → 0 as O₂ → 0). ``flux = daily · f_O2 · dt`` —
    dt-linear.

    ⚠ **This flow was single-currency CARBON until 2026-08-10** — the deliberate
    Phase-2 Step-4/Step-5 split, whose reasoning is preserved in this module's header.
    The humification split gives it a CO₂ leg, and the P2.1 composition gate *forces*
    the
    O₂ draw that comes with it: CO₂ entering a ``{CARBON:1, OXYGEN:2}`` pool drags two
    oxygens pure-carbon litter cannot supply. What the Step-4/5 split got right was that
    a CO₂ leg is inseparable from an O₂ leg; what it left unstated was that a litter
    pool
    decaying with **no** CO₂ leg asserts a carbon-use efficiency of 1.0 — see
    ``humification.py``.
    """

    id: FlowId
    priority: int
    litter_carbon: StockId
    microbial_carbon: StockId
    co2_pool: StockId
    o2_pool: StockId
    params: DecompositionParams
    humification: HumificationParams
    # The decomposer community's whole-cell respiratory O₂ affinity. Read from
    # ``microbial_respiration.yaml`` rather than duplicated here: it is the SAME
    # microbial
    # population respiring, one flow over. (Contrast the scope-C cross-KINGDOM copy
    # finding — herbivory's ``o2_half_saturation`` mirrored from the microbial file —
    # which is a different organism and is flagged there as a defect.)
    o2_half_saturation: float
    # Total chamber air (mol) — the intensive basis for the ``f_O2`` mole fraction.
    air_mol: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        f_o2 = oxygen_limitation_factor(
            snapshot.stocks[self.o2_pool].amount,
            air_mol=self.air_mol,
            k_o2=self.o2_half_saturation,
        )
        decayed = (
            decomposition_flux(
                snapshot.stocks[self.litter_carbon].amount,
                decomposition_rate=self.params.decomposition_rate,
            )
            * f_o2
            * dt
        )
        respired, stabilized = respired_and_stabilized(
            decayed, self.humification.litter_respired_fraction
        )
        return FlowResult(
            legs=(
                Leg(self.litter_carbon, -decayed),
                Leg(self.microbial_carbon, stabilized),
                Leg(self.co2_pool, respired),
                Leg(self.o2_pool, -respired),
            )
        )
