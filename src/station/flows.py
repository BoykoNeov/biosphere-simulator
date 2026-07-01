"""Station-layer flows: the cross-domain seams that only exist coupled (P6.2+).

The assembly layer owns flows whose stocks belong to *different* domains — the ones that
cannot live in any single ``domains.*`` package without one domain importing another
(the finding-#1 discipline). Two seams live here:

  * **CrewRespiration** (Step 2) — the merged stoichiometric respiration flow
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

  * **WaterRecovery** (Step 4) — the recovery flow that CLOSES the crew water loop:
    ``recovered_water → water_store (+η_w) + brine (+(1−η_w))``. Standalone Crew's
    ``water_store`` is a finite POOL that only *depletes* (``WaterBalance`` draws it to
    humidity + urine, both terminal sinks). Step 4 re-points those disposal paths into a
    ``recovered_water`` buffer POOL (the crew analogue of the biosphere's ``condensate``
    — the ECLSS ``Condenser`` product + the crew urine collect there), and this flow
    returns the recovered fraction ``η_w`` to ``water_store``, venting only the
    unrecoverable remainder ``(1−η_w)`` to a ``brine`` sink. So the crew's water becomes
    **regenerative up to the recovery efficiency** — the net drain drops from the full
    intake to ``(1−η_w)·intake``, closed exactly at ``η_w = 1`` (``brine`` is the honest
    remaining WATER boundary, the Thermal ``boundary.space`` analogue). The split is the
    ``SolarCharge`` / ``carbon_split`` idiom on the WATER quantity;
    **donor-controlled** on ``recovered_water`` (∝ its own amount), so positivity is
    structural (``k_rec·dt < 1``) and — unlike the *forced* crew flows — it makes
    ``water_store`` **state-dependent**, breaking the forced RK4 ≡ Euler bit-identity
    (the "it earned its keep" signal, the ``SelfDischarge`` analogue).

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
composition). The ``WaterRecovery`` split reuses the same fractional-split idiom
(``water_recovery_split``, the WATER analogue of ``carbon_split``); its rate +
efficiency are the first **station-owned** params
(``station/params/water_recovery.yaml``, loaded by ``station.loader``), unlike
``CrewRespiration`` which reused the crew's fraction.
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


@dataclass(frozen=True)
class WaterRecoveryParams:
    """Station-owned water-recovery coefficients (the processor's rate + efficiency).

    The first **station-owned** params (``CrewRespiration`` reused the crew's fraction);
    they load from ``station/params/water_recovery.yaml`` via ``station.loader`` with
    the same ``{value, unit, source}`` + exact-string-unit-guard discipline the sibling
    param files use. Both are exact-string guarded (neither is a conserved-Quantity
    canonical unit, so neither routes through pint — the ``ChargeParams`` /
    ``EclssParams`` discipline):

      * ``recovery_rate`` (k_rec, 1/s) ≥ 0: the first-order rate at which the water
        processor draws down the ``recovered_water`` buffer. Donor-controlled (∝ the
        buffer's own amount), so structural positivity requires ``k_rec·dt < 1``.
      * ``recovery_efficiency`` (η_w, dimensionless) ∈ [0, 1]: the fraction of processed
        water returned to ``water_store`` as potable; the remainder ``(1 − η_w)`` is
        vented to ``brine``. η_w = 1 is perfect closure (``brine`` leg exactly 0); η_w =
        0 is open-loop (the store gets nothing back — the "it bit" gate's baseline).
    """

    recovery_rate: float
    recovery_efficiency: float


def water_recovery_split(
    processed_kg: float, *, recovery_efficiency: float
) -> tuple[float, float]:
    """Split processed water into ``(potable, brine)`` (kg), summing to the input.

    ``potable = η_w · processed``; ``brine = (1 − η_w) · processed``. The
    ``carbon_split`` / ``water_split`` / ``charge_split`` idiom on the WATER quantity;
    the ~1e-15 round-off in ``potable + brine`` vs ``processed`` is covered by
    ``assert_flow_balanced``'s relative tolerance (the invariant is determinism, not
    bit-exact ``Σ == 0``). At η_w = 1 ``brine`` is exactly 0 (perfect closure); at η_w =
    0 ``potable`` is exactly 0 (open-loop); at ``processed = 0`` both are 0.
    """
    potable = recovery_efficiency * processed_kg
    brine = (1.0 - recovery_efficiency) * processed_kg
    return potable, brine


@dataclass(frozen=True)
class WaterRecovery:
    """WATER flow ``recovered_water → water_store (+η_w) + brine (+(1−η_w))`` (P6.4).

    The station-owned flow that closes the crew water loop. The processed water
    ``processed = k_rec · recovered_water · dt`` (kg) is withdrawn from the
    ``recovered_water`` buffer POOL (fed by the ECLSS ``Condenser`` product + the crew
    urine — the two disposal paths Step 4 re-points inward) and **split**:
    ``η_w·processed`` returned to ``crew.water_store`` as recovered potable water,
    ``(1−η_w)·processed`` vented to the ``brine`` sink. Three legs balance WATER exactly
    (``−processed + η·processed + (1−η)·processed = 0`` in intent; the ~1e-15 round-off
    is covered by the flow-balance tolerance). Always three legs (a zero-amount leg at
    η_w ∈ {0, 1} / an empty step), the ``SolarCharge`` "emit the leg even at the
    degenerate split" convention.

    **Donor-controlled** (reads the ``recovered_water`` stock, ∝ its amount), so:

      * positivity is **structural** (``k_rec·dt < 1``, self-limiting to 0 as the buffer
        empties — the ``SelfDischarge`` / ECLSS ``Condenser`` pattern), and
      * ``water_store`` becomes **state-dependent** (its recovery inflow depends on the
        buffer level), so the forced RK4 ≡ Euler bit-identity the standalone/cabin crew
        stores had is **broken** — a tolerance agreement now (the "it earned its keep"
        signal, the ``SelfDischarge`` analogue).

    ``flux = rate·dt`` is dt-linear (RK4-order-safe). At steady state the buffer holds
    ``recovered_water* = intake/k_rec`` and returns ``η_w·intake`` per unit time, so the
    store's net drain is ``(1−η_w)·intake`` — regenerative up to the recovery
    efficiency, fully closed only at η_w = 1.
    """

    id: FlowId
    priority: int
    recovered_water: StockId
    water_store: StockId
    brine: StockId
    params: WaterRecoveryParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        processed = (
            self.params.recovery_rate
            * snapshot.stocks[self.recovered_water].amount
            * dt
        )
        potable, brine = water_recovery_split(
            processed, recovery_efficiency=self.params.recovery_efficiency
        )
        return FlowResult(
            legs=(
                Leg(self.recovered_water, -processed),
                Leg(self.water_store, potable),
                Leg(self.brine, brine),
            )
        )
