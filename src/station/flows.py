"""Station-layer flows: the cross-domain seams that only exist coupled (P6.2+).

The assembly layer owns flows whose stocks belong to *different* domains — the ones that
cannot live in any single ``domains.*`` package without one domain importing another
(the finding-#1 discipline). Three seams live here:

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
    ``SolarCharge`` / ``carbon_split`` idiom on the WATER quantity; **donor-controlled**
    on ``recovered_water`` (∝ its own amount), so positivity is structural (``k_rec·dt <
    1``) and — unlike the *forced* crew flows — it makes ``water_store``
    **state-dependent**, breaking the forced RK4 ≡ Euler bit-identity (the "it earned
    its keep" signal, the ``SelfDischarge`` analogue).

  * **Lamp** (Step 5) — the grow-lamp flow ``battery → light_used (+η_lamp) + waste_heat
    (+(1−η_lamp))`` that carries electrical ENERGY into biology. It is the phase's **one
    non-shared-stock coupling** (finding #3 / #16): Power and the biosphere share *no*
    stock; the link is the **lamp-draw schedule**, which feeds both this flow (the
    ENERGY it withdraws from ``power.battery``) *and* the biosphere's ``par`` /
    ``daylength_s`` **forcings** (``station.lighting`` computes ``PAR =
    lamp_power·efficacy/ground_area`` from the same schedule). A flow cannot tell
    forcing from a shared stock (#16), so the frozen biosphere is untouched — PAR stays
    a forcing, merely *computed from the lamp* instead of a weather table. The ENERGY
    split is the ``SolarCharge`` η-split: the radiant fraction ``η_lamp`` leaves as PAR
    light (→ a ``light_used`` boundary sink), the remainder is waste heat. **Forced**
    (reads the ``lamp_power`` schedule, not a stock), so RK4 ≡ Euler on the battery —
    but the biosphere it lights is Euler-locked by its freeze, so the coupled run is
    Euler-only (no cross-check, matching the greenhouse). The waste-heat leg lands in a
    ``boundary.waste_heat`` sink here (the standalone-Power seam); moving it inward to
    ``thermal.node`` is deferred to the sealed-station step, the "boundary now, inward
    later" rhythm Power's own dissipation followed.

  * **Harvest** (Step 6) — the trophic biomass→food flow ``storage_c → food_store`` that
    makes the crew's finite ``food_store`` **regenerative**. It is the CARBON twin of
    ``WaterRecovery``: donor-controlled (``k_harvest·storage_c·dt``), self-limiting, and
    the seam that closes CARBON through one trophic level — the biosphere fixes cabin
    CO₂ into grain and this flow moves that grain into the crew's food. Both stocks are
    ``{CARBON:1}``, so it is a single-currency transfer (no composition fold, no
    η-split); it reads a biosphere stock and writes a crew stock, so — like every seam
    here — it lives in the assembly layer, never in ``domains.*``.

**Why the station layer, not the crew domain.** The crew domain *documents* ``C_food +
O₂ → CO₂ + H₂O`` atom coupling as a deferred **Phase-6** seam (its ``o2_store`` /
``crew_o2_consumed`` are the honest decoupled stand-in); honoring that boundary means
the atom-coupled flow lives in the assembly layer that owns cross-domain wiring, not in
the crew package. Standalone Crew keeps its three decoupled flows verbatim, so
``crew_state.json`` is untouched — Step 2 is a *separate* assembly.

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

from domains.biosphere.weather import PAR_UMOL_PER_J
from domains.crew.flows import carbon_split
from domains.crew.stocks import FOOD_INTAKE_VAR
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State

# The ``lamp_power`` forcing var (W): the lamp's electrical draw schedule. The single
# source both the :class:`Lamp` flow (the ENERGY it withdraws) and the biosphere's PAR
# forcing (``station.lighting``) read — the non-shared-stock coupling (#16).
LAMP_POWER_VAR: str = "lamp_power"

# Mean PAR-band photon energy (J per µmol photons), the inverse of the biosphere's own
# McCree (1972) ``4.57 µmol J⁻¹`` conversion (``domains.biosphere.weather``). A single
# source of truth: the SAME constant the biosphere uses to turn PAR irradiance into a
# photon flux is inverted here to book the radiant PAR energy the lamp emits. It is a
# spectrum-averaged physical constant (a ``σ``/CODATA-style module constant with
# provenance, NOT a tunable param — the ``drift.py`` discipline); the PAR-band average
# is spectrum-dependent, and reusing the biosphere's daylight value for the LED here is
# an illustrative approximation (calibration deferred to the validation step).
PAR_PHOTON_ENERGY_J_PER_UMOL: float = 1.0 / PAR_UMOL_PER_J


@dataclass(frozen=True)
class CrewRespiration:
    """CARBON+OXYGEN flow ``food_store + cabin_o2 → cabin_co2 + fecal_waste`` (P6.2).

    The atom-coupled merge of standalone crew's ``OxygenConsumption`` + the CO₂ leg of
    ``FoodMetabolism``. The forced food-carbon intake ``q =
    env.get(crew_food_intake)·dt`` (mol C) is withdrawn from the finite food store and
    **split** by ``respired_carbon_fraction`` (``carbon_split``) into ``respired``
    (metabolized to CO₂) and ``feces`` (egested). Four legs:

      * ``food_store −q``        (CARBON −q)
      * ``cabin_co2 +respired``  (CARBON +respired **and** OXYGEN +2·respired, via the
        ``{CARBON:1, OXYGEN:2}`` composition fold)
      * ``cabin_o2 −respired``   (OXYGEN −2·respired, via the ``{OXYGEN:2}`` fold)
      * ``fecal_waste +feces``   (CARBON +feces)

    CARBON balances (``−q + respired + feces = 0``) and OXYGEN balances (``+2·respired −
    2·respired = 0``) in one flow at PQ = 1 — the ``MicrobialRespiration`` pattern. Only
    ``respired`` (not ``q``) draws O₂: egested feces is not oxidized. **Forced** (reads
    ``env``, not a stock), so ``food_store`` stays bit-identical under Euler/RK4; ``flux
    = rate·dt`` is dt-linear. Always four legs (a zero-amount leg at ``f_resp = 1`` / an
    empty step), the ``SolarCharge`` "emit the leg even at the degenerate split"
    convention. ``respired_carbon_fraction`` rides on ``params`` (the crew
    ``CrewParams`` — the same physiology fraction standalone ``FoodMetabolism`` uses).
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


@dataclass(frozen=True)
class LampParams:
    """Station-owned grow-lamp coefficient: the photosynthetic photon efficacy.

    Loads from ``station/params/lamp.yaml`` via ``station.loader`` with the same
    ``{value, unit, source}`` + exact-string-unit-guard discipline the sibling param
    files use (``photon_efficacy`` is not a conserved-Quantity canonical unit, so it is
    schema-validated + exact-string guarded, not routed through pint — the
    ``ChargeParams`` / ``EclssParams`` discipline):

      * ``photon_efficacy`` (µmol J⁻¹) > 0: PAR photons emitted per joule of electrical
        input — the standard grow-lamp figure of merit. It is the **one** lamp param:
        the PAR photon flux is ``photon_efficacy · lamp_power / ground_area``, and the
        ENERGY split ``η_lamp = photon_efficacy · PAR_PHOTON_ENERGY_J_PER_UMOL`` is
        *derived* from it (radiant PAR energy = photon flux × mean photon energy), so
        efficacy and the radiant fraction are two accountings of **one** device, made
        consistent by derivation (never two independently-set params that could
        disagree). The physical ceiling is ``PAR_UMOL_PER_J`` (all input → PAR photons ⇒
        η_lamp = 1, heat leg 0); the loader rejects a value above it (an over-unity
        lamp) and a non-positive one.
    """

    # η_φ, photosynthetic photon efficacy (µmol J⁻¹): PAR photons per joule electrical.
    photon_efficacy: float


def lamp_energy_split(
    draw_joules: float, *, photon_efficacy: float
) -> tuple[float, float]:
    """Split lamp electrical draw into ``(radiant_par, waste_heat)`` (J), summing to
    input.

    The radiant PAR energy is ``η_lamp · draw`` where the radiant fraction ``η_lamp =
    photon_efficacy · PAR_PHOTON_ENERGY_J_PER_UMOL`` (photons emitted × mean photon
    energy), and the remainder ``(1 − η_lamp) · draw`` is waste heat. The
    ``SolarCharge`` / ``carbon_split`` η-split idiom on ENERGY; the ~1e-15 round-off in
    ``radiant + heat`` vs ``draw`` is covered by ``assert_flow_balanced``'s relative
    tolerance (the invariant is determinism, not bit-exact ``Σ == 0``). At the physical
    ceiling ``photon_efficacy = PAR_UMOL_PER_J`` (η_lamp = 1) ``heat`` is exactly 0; at
    ``draw = 0`` both are 0.
    """
    radiant_fraction = photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL
    radiant = radiant_fraction * draw_joules
    heat = (1.0 - radiant_fraction) * draw_joules
    return radiant, heat


@dataclass(frozen=True)
class Lamp:
    """ENERGY flow ``battery → light_used (+η_lamp) + waste_heat (+(1−η_lamp))`` (P6.5).

    The grow-lamp that carries electrical energy into biology. The forced draw ``D =
    env.get(lamp_power)·dt`` (W·s = J) is withdrawn from ``power.battery`` and split
    (:func:`lamp_energy_split`): the radiant fraction ``η_lamp·D`` leaves as PAR light
    (into a ``light_used`` boundary sink — the photon energy the plants receive; its
    downstream fate as absorbed heat is out of scope), the remainder ``(1−η_lamp)·D`` is
    named as lamp waste heat (into ``waste_heat`` — a ``boundary.waste_heat`` sink here;
    the inward move to ``thermal.node`` is deferred to the sealed-station step). Three
    legs balance ENERGY exactly in intent (``−D + η·D + (1−η)·D = 0``). Always three
    legs (zero-amount at η_lamp = 1 / lamp off); ``flux = rate·dt`` is dt-linear
    (RK4-order-safe, multi-rate-safe).

    **Forced** (reads the ``lamp_power`` schedule, not a stock), so the battery is
    bit-identical under Euler/RK4 — but the biosphere this lamp lights is Euler-locked
    by its freeze, so the coupled lighting run is Euler-only. Positivity of the battery
    draw is a **sizing discipline** (a well-fed provisioned battery — the ``LoadDraw``
    way), not structural: the lamp is forced, not donor-controlled. The SAME
    ``lamp_power`` schedule also drives the biosphere's PAR forcing in
    ``station.lighting`` (the non-shared-stock coupling, #16) — this flow only books the
    *energy*; the *photons* are a forcing the frozen biosphere reads.
    """

    id: FlowId
    priority: int
    battery: StockId
    light_used: StockId
    waste_heat: StockId
    params: LampParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        draw = env.get(LAMP_POWER_VAR) * dt
        radiant, heat = lamp_energy_split(
            draw, photon_efficacy=self.params.photon_efficacy
        )
        return FlowResult(
            legs=(
                Leg(self.battery, -draw),
                Leg(self.light_used, radiant),
                Leg(self.waste_heat, heat),
            )
        )


@dataclass(frozen=True)
class HarvestParams:
    """Station-owned grain-harvest coefficient (the harvester's first-order rate).

    Loads from ``station/params/harvest.yaml`` via ``station.loader`` with the same
    ``{value, unit, source}`` + exact-string-unit-guard discipline the sibling param
    files use (``harvest_rate`` is not a conserved-Quantity canonical unit, so it is
    schema-validated + exact-string guarded, not routed through pint — the
    ``WaterRecoveryParams`` / ``LampParams`` discipline):

      * ``harvest_rate`` (k_harvest, 1/s) ≥ 0: the first-order rate at which the
        harvester draws grain from the biosphere's ``storage_c`` organ into the crew
        ``food_store``. Donor-controlled (∝ the standing grain), so structural
        positivity requires ``k_harvest·dt < 1`` (self-limiting to 0 as the grain
        empties — the ``SelfDischarge`` / biosphere ``Grazing`` pattern). A zero rate
        disables harvest (the "it bit" gate's ``with_harvest=False`` baseline).
    """

    harvest_rate: float


@dataclass(frozen=True)
class Harvest:
    """CARBON flow ``storage_c → food_store`` (P6.6) — the trophic biomass→food seam.

    The station-owned flow that makes the crew's finite ``food_store`` **regenerative**.
    The biosphere's grain / storage organ (``storage_c``, a pure ``{CARBON:1}`` pool the
    plant fills post-anthesis and — by ``allocation.py`` — the one organ **excluded**
    from maintenance / senescence / the ``f_N`` biomass sum, so it accumulates without
    being clawed back) is drawn into the crew ``food_store`` at ``harvested = k_harvest
    · storage_c · dt`` (mol C). Two legs balance CARBON exactly (``−harvested +
    harvested = 0``); both stocks are ``{CARBON:1}`` so it is a **single-currency
    transfer** — no composition fold (unlike ``CrewRespiration``) and no η-split (unlike
    ``WaterRecovery`` / ``Lamp``: harvested grain has a single fate, the crew's food).

    **Donor-controlled** (reads the ``storage_c`` stock, ∝ its amount), so positivity is
    **structural** (``k_harvest·dt < 1``, self-limiting to 0 as the grain empties — the
    biosphere ``Grazing`` / ``SelfDischarge`` pattern, *not* the forced "well-fed
    sizing" the crew flows lean on). It reads a **biosphere** stock and writes a
    **crew** stock — a cross-domain flow, so it belongs in the station layer, never
    in ``domains.*`` (the finding-#1 discipline). ``flux = rate·dt`` is dt-linear.

    The biosphere refills ``storage_c`` once per master day (the slow ``Allocation``
    step); this flow — living in the cabin / fast registry — drains it across the day's
    sub-steps, so the grain settles to a positive quasi-steady (daily harvest ≈ daily
    fill): a **regenerative** source being drained, not a static reservoir emptied. The
    ``Allocation`` grain-fill leg (``FO·DMI``) is independent of ``storage_c``'s own
    level, and no other biosphere flow reads ``storage_c`` (only ``annual_reset``, which
    does not fire in a ≤7-day run), so harvest is its **only** new sink and does not
    perturb the plant's carbon budget — the with/without-harvest grain fill is
    identical, which makes the two-way conservation identity (``Δfood_store = cumulative
    harvest = Δstorage_c``) exact to floating point.
    """

    id: FlowId
    priority: int
    storage_c: StockId
    food_store: StockId
    params: HarvestParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        harvested = (
            self.params.harvest_rate * snapshot.stocks[self.storage_c].amount * dt
        )
        return FlowResult(
            legs=(
                Leg(self.storage_c, -harvested),
                Leg(self.food_store, harvested),
            )
        )
