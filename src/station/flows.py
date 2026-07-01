"""Station-layer flows: the cross-domain seams that only exist coupled (P6.2+).

The assembly layer owns flows whose stocks belong to *different* domains — the ones that
cannot live in any single ``domains.*`` package without one domain importing another
(the finding-#1 discipline). Step 2's seam is **crew respiration made atom-coupled**:

  * **CrewRespiration** — the merged stoichiometric respiration flow
    ``food_store + cabin_o2 → cabin_co2 + fecal_waste``. It replaces, *in the coupled
    assembly only*, two decoupled standalone-crew flows: ``OxygenConsumption`` (which
    drew O₂ from a separate ``crew.o2_store`` into a decoupled sink) and the CO₂ leg of
    ``FoodMetabolism`` (which produced *pure-carbon* CO₂). Coupled, the crew breathes
    **cabin** O₂ and exhales into the **cabin** CO₂ pool, and — because ``cabin_co2`` is
    a composition ``{CARBON:1, OXYGEN:2}`` stock (see ``station.cabin``) — the O₂ those
    two oxygens came from must be named: it is drawn from ``cabin_o2`` (``{OXYGEN:2}``)
    at the respiratory quotient. This is the
    ``biosphere.microbial_respiration.MicrobialRespiration`` template one trophic level
    up (``microbial_C + o2_pool → carbon_pool``), and the flow that makes **OXYGEN close
    across the crew↔cabin loop**.

**Why the station layer, not the crew domain.** The crew domain *documents*
``C_food + O₂ → CO₂ + H₂O`` atom coupling as a deferred **Phase-6** seam (its
``o2_store`` / ``crew_o2_consumed`` are the honest decoupled stand-in); honoring that
boundary means the atom-coupled flow lives in the assembly layer that owns cross-domain
wiring, not in the crew package. Standalone Crew keeps its three decoupled flows
verbatim, so ``crew_state.json`` is untouched — Step 2 is a *separate* assembly.

**Forced, like the crew flows it merges (RK4-order-safe).** The magnitude is the forced
food-carbon intake ``q = env.get(crew_food_intake)·dt`` (mol C), never a stock read, so
CrewRespiration itself keeps ``crew.food_store`` bit-identical under Euler/RK4 (the
ECLSS control loops are the state-dependent part of the coupled system). Positivity on
``cabin_o2`` is by **well-fed sizing** (the ``LoadDraw`` way — the O₂-makeup regulator
keeps the cabin above 0; the draw is a small fraction of the pool), **not** structural.

**RQ = 1 is baked in by the PQ = 1 template — the honest simplification.** One respired
mol C consumes exactly one mol O₂ and yields one mol CO₂, so O₂ consumption *equals* CO₂
production in this single flow (unlike standalone ECLSS, which set an independent
``o2_consumption`` ≠ ``co2_production``, RQ ≈ 0.75). A realistic non-unity respiratory
quotient needs the metabolic-water / food-composition machinery (each mol food carrying
its own H/O), which the biosphere also defers — matching its ``{CARBON:1}`` biomass / PQ
= 1 convention. The **fecal** carbon is *not* oxidized (it is egested, not metabolized),
so only the ``respired`` fraction draws O₂ — the O₂ leg magnitude is ``respired``, not
the full intake ``q``.

Pure stdlib only. The carbon split reuses ``domains.crew.flows.carbon_split`` (the same
``respired_carbon_fraction`` physiology as standalone ``FoodMetabolism``); the water
side of respiration stays on the separate ``WaterBalance`` path (metabolic water
ignored, per the phase-6 plan's WATER scope boundary — food carries no WATER
composition).
"""

from dataclasses import dataclass

from domains.crew.flows import carbon_split
from domains.crew.stocks import FOOD_INTAKE_VAR
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class CrewRespiration:
    """CARBON+OXYGEN flow ``food_store + cabin_o2 → cabin_co2 + fecal_waste`` (P6.2).

    The atom-coupled merge of standalone crew's ``OxygenConsumption`` + the CO₂ leg of
    ``FoodMetabolism``. The forced food-carbon intake
    ``q = env.get(crew_food_intake)·dt`` (mol C) is withdrawn from the finite food store
    and **split** by ``respired_carbon_fraction`` (``carbon_split``) into ``respired``
    (metabolized to CO₂) and ``feces`` (egested). Four legs:

      * ``food_store −q``        (CARBON −q)
      * ``cabin_co2 +respired``  (CARBON +respired **and** OXYGEN +2·respired, via the
                                  ``{CARBON:1, OXYGEN:2}`` composition fold)
      * ``cabin_o2 −respired``   (OXYGEN −2·respired, via the ``{OXYGEN:2}`` fold)
      * ``fecal_waste +feces``   (CARBON +feces)

    CARBON balances (``−q + respired + feces = 0``) and OXYGEN balances
    (``+2·respired − 2·respired = 0``) in one flow at PQ = 1 — the
    ``MicrobialRespiration`` pattern. Only ``respired`` (not ``q``) draws O₂: egested
    feces is not oxidized. **Forced** (reads ``env``, not a stock), so ``food_store``
    stays bit-identical under Euler/RK4; ``flux = rate·dt`` is dt-linear. Always four
    legs (a zero-amount leg at ``f_resp = 1`` / an empty step), the ``SolarCharge``
    "emit the leg even at the degenerate split" convention. ``respired_carbon_fraction``
    rides on ``params`` (the crew ``CrewParams`` — the same physiology fraction
    standalone ``FoodMetabolism`` uses).
    """

    id: FlowId
    priority: int
    food_store: StockId
    cabin_co2: StockId
    cabin_o2: StockId
    fecal_waste: StockId
    respired_carbon_fraction: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        q = env.get(FOOD_INTAKE_VAR) * dt
        respired, feces = carbon_split(
            q, respired_carbon_fraction=self.respired_carbon_fraction
        )
        # respired mol C → +respired mol CO₂ into cabin_co2 (carrying 2·respired O) and
        # −respired mol O₂ out of cabin_o2 (supplying those 2·respired O), PQ = 1. feces
        # is egested carbon (not oxidized) — it draws no O₂.
        return FlowResult(
            legs=(
                Leg(self.food_store, -q),
                Leg(self.cabin_co2, respired),
                Leg(self.cabin_o2, -respired),
                Leg(self.fecal_waste, feces),
            )
        )
