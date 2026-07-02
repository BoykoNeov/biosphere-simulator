"""The sealed station: the fully-coupled multi-year matter + energy build (P6.7).

Step 7's assembly — the Phase-4 analogue at station scale. It composes **every** Phase-6
shared-stock seam over one shared stock dict + two registries (biosphere-slow /
everything-
fast) and runs it multi-year to prove that the full assembly **sustains** every-quantity
**and** ENERGY conservation to round-off, with no drift, over many annual cycles, while
the regulated pools and the (period-1) coupled biosphere stay bounded.

**What is coupled (the union of Steps 1–6, at one fast rate).** The fast registry holds:

  * the 5 cabin flows (:class:`station.flows.CrewRespiration`, crew ``WaterBalance``,
    ECLSS ``CO2Scrubber`` / ``Condenser`` / ``O2Makeup``) re-pointed at the biosphere's
    ``CARBON_POOL`` / ``O2_POOL`` (the Step-3 greenhouse reverse seam) — plants +
    microbes
    + crew breathe one cabin-air stock;
  * ``SolarCharge`` / ``LoadDraw`` (Power) with their dissipation legs →
  ``thermal.node``
    and ``RadiatorReject`` (Thermal) rejecting the real load to deep space (the Step-1
    inward heat seam — ``boundary.waste_heat`` is **absent**);
  * the :class:`station.flows.Lamp` drawing ``power.battery`` → ``light_used`` +
  waste-heat
    (**also → ``thermal.node``**, the inward move Step 5 deferred), while the same lamp
    schedule sets the biosphere's PAR / daylength forcing (the Step-5 energy→biology
    coupling);
  * the station-owned :class:`station.flows.WaterRecovery` closing the crew water loop
    (the Step-4 ``recovered_water`` buffer + ``brine`` sink).

The **biosphere-slow** registry is :func:`build_season` verbatim (the sealed self-
contained chamber), re-sown each year by :func:`domains.biosphere.season.annual_reset`
via
the driver's new ``slow_reset`` hook — the machinery the ≤7-day greenhouse / lighting /
harvest runs never exercised (sub-seasonal, so ``annual_reset`` never fired).

**The two time scales.** The biosphere is structurally ``dt = 1`` day; everything else
is
``dt = 60 s`` (ECLSS's binding ``k_scrub·dt < 1``).
:func:`station.driver.run_master_day`
does the operator split by hand (slow once/day + fast ×1440), asserting conservation
after
**every** fast sub-step over the whole shared ledger. Power runs the **constant daily-
average** solar/load: under ``substep`` the day count ``n`` is frozen within a day, so
the
diurnal solar shape is not expressible (the Step-5 lamp-average precedent) — the diurnal
SOC swing + the node's emergent ``T_eq`` attractor are Tier 1's job (single-rate
:func:`station.system.run_station`, where ``n`` advances).

**Scope (spike-measured, advisor-endorsed — see
:class:`station.scenario.SealedStationScenario`).**
``with_harvest`` and ``close_feces`` both default **off** for the Tier-2 run: harvest
starves the annual re-sow (it drains ``storage_c`` below the seed bank), and the
litter/microbial loop is the one *unregulated* loop (unbounded at illustrative crew-vs-
plant scale) — so both are scoped out of the conservation/longevity proof and
``close_feces=True`` is *characterized* in the Tier-3 landmine test. Matter is then open
at
the feces boundary and at store provisioning (whole-system matter stationarity is
deferred
to Step 9 calibration); **energy** earns a genuine subsystem attractor (Tier 1),
**matter**
earns conservation-to-round-off + regulated-pool stationarity + the period-1 plant.

**Zero domain / zero core change** — assembly-level id choices only (finding #1). Pure
stdlib in the spine; every coefficient loads via the sibling / station loaders.
"""

from collections.abc import Callable

from domains.biosphere.season import annual_reset, build_season, weather_resolver
from domains.biosphere.stocks import (
    CARBON_POOL,
    DAYLENGTH_VAR,
    LITTER_CARBON,
    O2_POOL,
    PAR_VAR,
    STORAGE_C,
    THERMAL_TIME,
)
from domains.crew.flows import CrewParams, WaterBalance
from domains.crew.stocks import (
    FECAL_WASTE,
    FOOD_INTAKE_VAR,
    FOOD_STORE,
    WATER_INTAKE_VAR,
    WATER_STORE,
    food_store_stock,
    water_store_stock,
)
from domains.crew.system import WATER_BALANCE
from domains.eclss.flows import CO2Scrubber, Condenser, EclssParams, O2Makeup
from domains.eclss.stocks import (
    CABIN_H2O,
    CO2_REMOVED,
    O2_SUPPLY,
    cabin_h2o_stock,
)
from domains.eclss.system import CO2_SCRUBBER, CONDENSER, O2_MAKEUP
from domains.power.flows import ChargeParams, LoadDraw, SolarCharge
from domains.power.stocks import BATTERY, SOLAR_SOURCE, battery_stock
from domains.power.system import (
    LOAD_DRAW,
    LOAD_POWER_VAR,
    SOLAR_CHARGE,
    SOLAR_POWER_VAR,
    balanced_load_w,
    daily_solar_energy,
)
from domains.thermal.flows import RadiatorReject, ThermalParams
from domains.thermal.scenario import ThermalScenario
from domains.thermal.stocks import NODE, SPACE, node_stock
from domains.thermal.system import RADIATOR_REJECT, equilibrium_temperature
from simcore import boundary
from simcore.environment import SourceResolver, constant
from simcore.events import Event
from simcore.flow import Flow
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State
from station.cabin import CO2_COMPOSITION, CREW_RESPIRATION, O2_COMPOSITION
from station.driver import run_master_day
from station.flows import (
    LAMP_POWER_VAR,
    PAR_PHOTON_ENERGY_J_PER_UMOL,
    CrewRespiration,
    Harvest,
    HarvestParams,
    Lamp,
    LampParams,
    WaterRecovery,
    WaterRecoveryParams,
)
from station.harvest import HARVEST
from station.lighting import LAMP, LIGHT_USED, lamp_par
from station.scenario import (
    SEALED_STATION_SCENARIO,
    LightingScenario,
    SealedStationScenario,
)
from station.water import BRINE, RECOVERED_WATER, WATER_RECOVERY, _recovered_water_pool
from station.water import _gas_boundary as _composition_boundary


def _lighting_view(scenario: SealedStationScenario) -> LightingScenario:
    """A :class:`LightingScenario` view for the lamp helpers (PAR / average power).

    :func:`station.lighting.lamp_par` and :func:`station.lighting.lamp_average_power`
    read ``lamp_power_w`` / ``photoperiod_hours`` / ``bio.ground_area`` off a
    ``LightingScenario``; the sealed scenario carries those fields flat, so this wraps
    them (with the sealed biosphere, for ``ground_area``) to reuse the Step-5 helpers
    verbatim rather than re-deriving the PAR / daily-average formulas.
    """
    return LightingScenario(
        bio=scenario.bio,
        lamp_power_w=scenario.lamp_power_w,
        photoperiod_hours=scenario.photoperiod_hours,
        battery0=scenario.battery0,
    )


def _mean_solar_power(scenario: SealedStationScenario) -> float:
    """The constant daily-average solar supply (W) the fast Power flows read.

    ``daily_solar_energy / day_seconds`` — the half-sine's daily energy spread over the
    day (the diurnal shape is not expressible under the ``n``-frozen fast domain, so
    Power
    runs its daily average; the Step-5 lamp-average precedent). At ``load_fraction = 1``
    the derived load (:func:`domains.power.system.balanced_load_w`) equals ``η_c`` ×
    this,
    so Power's net battery flux is zero and the battery moves only under the lamp drain.
    """
    ph = scenario.power
    return daily_solar_energy(ph) / (ph.steps_per_day * ph.dt_seconds)


def sealed_node_heat(
    charge: ChargeParams,
    thermal_params: ThermalParams,
    lamp_params: LampParams,
    scenario: SealedStationScenario = SEALED_STATION_SCENARIO,
) -> float:
    """The node's initial heat ``Q_eq = C·(T_eq − T_space)`` (J), set by dissipation.

    The total heat into the node is constant (all forced): the charge-conversion loss
    ``(1−η_c)·solar_avg`` + the 100 %-dissipative ``LoadDraw`` (``balanced_load``) + the
    lamp waste heat ``(1−η_lamp)·lamp_avg`` (the lamp's radiant ``η_lamp`` leg leaves as
    PAR to ``light_used``, not to the node). :func:`build_sealed_station` starts the
    node at
    the ``Q_eq`` this dissipation implies (via Thermal's closed-form
    :func:`equilibrium_temperature`), so the coupled run begins at the attractor — a
    prediction placed as an initial condition, the Step-1 ``equilibrium_node_heat``
    rhythm.
    """
    solar_avg = _mean_solar_power(scenario)
    load_w = balanced_load_w(charge, scenario.power)
    lamp_avg = _lighting_average_power(scenario)
    eta_lamp = lamp_params.photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL
    heat_w = (
        (1.0 - charge.charge_efficiency) * solar_avg
        + load_w
        + (1.0 - eta_lamp) * lamp_avg
    )
    t_eq = equilibrium_temperature(thermal_params, ThermalScenario(heat_load_w=heat_w))
    return thermal_params.heat_capacity * (t_eq - thermal_params.space_temperature)


def _lighting_average_power(scenario: SealedStationScenario) -> float:
    """The constant daily-average lamp draw (W): ``lamp_power_w · photoperiod / 24``."""
    return scenario.lamp_power_w * scenario.photoperiod_hours / 24.0


def build_sealed_station(
    charge: ChargeParams,
    thermal_params: ThermalParams,
    crew_params: CrewParams,
    eclss_params: EclssParams,
    recovery_params: WaterRecoveryParams,
    lamp_params: LampParams,
    harvest_params: HarvestParams,
    scenario: SealedStationScenario = SEALED_STATION_SCENARIO,
    *,
    with_harvest: bool = False,
    close_feces: bool = False,
) -> tuple[State, Registry, Registry]:
    """Assemble the fully-coupled sealed station: ``(state, bio_reg, fast_reg)``.

    Two disjoint registries over one shared stock dict (the two-rate model). The
    **biosphere** registry is :func:`build_season`'s output verbatim (the sealed
    chamber's
    CARBON/OXYGEN/WATER/NITROGEN stocks + flows + the ``thermal_time`` aux). The
    **fast**
    registry is every Phase-6 fast seam: the 5 cabin flows re-pointed at the biosphere
    gas
    pools, ``SolarCharge`` / ``LoadDraw`` / ``Lamp`` (all waste-heat →
    ``thermal.node``),
    ``RadiatorReject``, and ``WaterRecovery`` — plus ``Harvest`` iff ``with_harvest``.

    The fast-domain stocks are the biosphere-disjoint set: the two crew stores, the
    cabin-owned ``cabin_h2o``, the ``recovered_water`` buffer + ``brine`` sink, the two
    composition-carrying gas reservoirs (``o2_supply`` / ``co2_removed``), the four
    ENERGY
    stocks (``battery`` + ``solar_source`` + ``thermal.node`` + ``space``), and
    ``light_used``. ``boundary.waste_heat`` / ``boundary.heat_source`` are **absent**
    (the
    dissipation lands in-system — the Step-1 structural redirection). The node starts at
    :func:`sealed_node_heat` (the dissipation-set equilibrium).

    ``with_harvest`` (default **off**): the biomass→food seam. Off for Tier 2 — harvest
    drains ``storage_c`` below the seed bank ``annual_reset`` needs, starving the re-sow
    (its conservation is pinned in Step 6). ``close_feces`` (default **off**): route the
    crew's fecal carbon to the biosphere ``LITTER_CARBON`` (closing the trophic ring) vs
    the open ``FECAL_WASTE`` sink. Off for Tier 2 — the litter loop is unregulated and
    grows unbounded at illustrative scale (the Tier-3 landmine).

    Two disjointness guards: the biosphere / fast **stock-id** sets (a silent dict
    overwrite would drop a stock) and the two registries' **flow-id** sets (a duplicate
    id
    across the pair the driver steps together is a silent cross-domain wiring bug — each
    ``Registry`` rejects duplicates only *within* itself).
    """
    # --- biosphere (slow) — build_season verbatim -----------------------------------
    bio_state, bio_reg = build_season(scenario.bio)
    bio_stocks = dict(bio_state.stocks)

    # --- fast-domain stocks (cabin + crew + Power/Thermal), biosphere-disjoint --------
    fecal_target = LITTER_CARBON if close_feces else FECAL_WASTE
    node0 = sealed_node_heat(charge, thermal_params, lamp_params, scenario)
    cabin_seq = [
        food_store_stock(scenario.cabin.food_store0),
        water_store_stock(scenario.cabin.water_store0),
        cabin_h2o_stock(scenario.cabin.cabin_h2o_0),
        _composition_boundary(
            O2_SUPPLY, Quantity.OXYGEN, O2_COMPOSITION, unclamped=True
        ),
        _composition_boundary(
            CO2_REMOVED, Quantity.CARBON, CO2_COMPOSITION, unclamped=False
        ),
        _recovered_water_pool(),
        boundary.sink(BRINE, Quantity.WATER, 0.0),
    ]
    # The FECAL_WASTE boundary sink exists only for the open loop; when feces is
    # re-pointed into LITTER_CARBON (close_feces) it would be an orphan shadow sink.
    if not close_feces:
        cabin_seq.append(boundary.sink(FECAL_WASTE, Quantity.CARBON, 0.0))
    power_seq = [
        battery_stock(scenario.battery0),
        boundary.source(SOLAR_SOURCE, Quantity.ENERGY, 0.0),
        node_stock(node0),
        boundary.sink(SPACE, Quantity.ENERGY, 0.0),
        boundary.sink(LIGHT_USED, Quantity.ENERGY, 0.0),
    ]
    fast_stocks = {s.id: s for s in cabin_seq + power_seq}

    overlap = bio_stocks.keys() & fast_stocks.keys()
    if overlap:
        raise ValueError(
            "sealed-station stock-id collision between the biosphere and the fast "
            f"domain: {sorted(overlap)} (the two stock sets must be disjoint — a "
            "silent dict overwrite would drop a stock)"
        )
    stocks = {**bio_stocks, **fast_stocks}
    state = State(n=0, stocks=stocks, rng_seed=0, aux={THERMAL_TIME: 0.0})

    # --- fast flows -----------------------------------------------------------------
    fast_flows: list[Flow] = [
        CrewRespiration(
            CREW_RESPIRATION,
            0,
            food_store=FOOD_STORE,
            cabin_co2=CARBON_POOL,  # the greenhouse seam: crew exhales into the bio CO₂
            cabin_o2=O2_POOL,  # the greenhouse seam: crew breathes the bio O₂
            fecal_waste=fecal_target,
            respired_carbon_fraction=crew_params.respired_carbon_fraction,
        ),
        WaterBalance(
            WATER_BALANCE,
            0,
            water_store=WATER_STORE,
            crew_humidity=CABIN_H2O,
            urine=RECOVERED_WATER,  # the Step-4 seam: urine → the recovery buffer
            params=crew_params,
        ),
        CO2Scrubber(
            CO2_SCRUBBER,
            0,
            cabin_co2=CARBON_POOL,
            co2_removed=CO2_REMOVED,
            params=eclss_params,
        ),
        Condenser(
            CONDENSER,
            0,
            cabin_h2o=CABIN_H2O,
            humidity_condensate=RECOVERED_WATER,  # the Step-4 seam: condensate → buffer
            params=eclss_params,
        ),
        O2Makeup(
            O2_MAKEUP, 0, o2_supply=O2_SUPPLY, cabin_o2=O2_POOL, params=eclss_params
        ),
        WaterRecovery(
            WATER_RECOVERY,
            0,
            recovered_water=RECOVERED_WATER,
            water_store=WATER_STORE,  # the Step-4 seam: recovered water → the store
            brine=BRINE,
            params=recovery_params,
        ),
        SolarCharge(
            SOLAR_CHARGE,
            0,
            solar_source=SOLAR_SOURCE,
            battery=BATTERY,
            waste_heat=NODE,  # the Step-1 inward seam: dissipation → the thermal node
            params=charge,
        ),
        LoadDraw(LOAD_DRAW, 0, battery=BATTERY, waste_heat=NODE),
        Lamp(
            LAMP,
            0,
            battery=BATTERY,
            light_used=LIGHT_USED,
            waste_heat=NODE,  # the inward move Step 5 deferred: lamp heat → the node
            params=lamp_params,
        ),
        RadiatorReject(
            RADIATOR_REJECT, 0, node=NODE, space=SPACE, params=thermal_params
        ),
    ]
    if with_harvest:
        fast_flows.append(
            Harvest(
                HARVEST,
                0,
                storage_c=STORAGE_C,
                food_store=FOOD_STORE,
                params=harvest_params,
            )
        )
    fast_reg = Registry(fast_flows, stocks)

    _assert_flow_ids_disjoint(bio_reg, fast_reg)
    return state, bio_reg, fast_reg


def _assert_flow_ids_disjoint(bio_reg: Registry, fast_reg: Registry) -> None:
    """Guard: the biosphere-slow and fast registries share no ``FlowId``.

    The driver steps both registries over one shared stock dict; a flow id present in
    both would be a silent cross-domain wiring bug (a ``Registry`` rejects duplicates
    only
    *within* itself). The flow-id analogue of the stock-id disjointness check (the
    ``build_harvest`` guard, generalized to the maximal assembly).
    """
    bio_ids = {flow.id for flow in bio_reg.flows}
    fast_ids = {flow.id for flow in fast_reg.flows}
    overlap = bio_ids & fast_ids
    if overlap:
        raise ValueError(
            "sealed-station flow-id collision between the biosphere and the fast "
            f"registries: {sorted(overlap)} (the two flow sets the driver steps "
            "together must be disjoint)"
        )


def sealed_bio_resolver(
    weather: list[dict[str, float | str]],
    lamp_params: LampParams,
    scenario: SealedStationScenario = SEALED_STATION_SCENARIO,
) -> SourceResolver:
    """The biosphere forcing: weather-driven, with PAR + daylength from the lamp.

    Starts from the frozen ``weather_resolver`` (temperature / VPD / net radiation / Ci
    /
    irrigation / fertilization + the sealed ``shared`` map ``{SOIL_WATER_VAR:
    soil_water,
    CO2_POOL_VAR: carbon_pool}`` — so FvCB's Ci reads the live shared cabin CO₂, the
    greenhouse reverse seam, unchanged), then overrides two forcings from the lamp:
    ``PAR_VAR`` → the on-window flux ``photon_efficacy·lamp_power_w/ground_area`` and
    ``DAYLENGTH_VAR`` → ``photoperiod_hours·3600`` (the Step-5 lighting coupling — both
    must come from the lamp together, since the daily photon dose is PAR × daylength).
    The
    ``weather`` must be tiled to cover the full horizon (``years×`` the season) so
    ``_table`` never end-clamps.
    """
    base = weather_resolver(weather, scenario.bio)
    forcings = dict(base.forcings)
    forcings[PAR_VAR] = constant(lamp_par(lamp_params, _lighting_view(scenario)))
    forcings[DAYLENGTH_VAR] = constant(scenario.photoperiod_hours * 3600.0)
    return SourceResolver(forcings=forcings, shared=dict(base.shared))


def sealed_fast_resolver(
    charge: ChargeParams,
    scenario: SealedStationScenario = SEALED_STATION_SCENARIO,
) -> SourceResolver:
    """The fast-domain forcing: crew intakes + lamp draw + constant solar/load.

    Merges every fast flow's forcing (disjoint vars): the two constant crew intake rates
    (``food_intake`` / ``water_intake``); the lamp's daily-average electrical draw
    (``lamp_power``); and Power's ``solar_power`` (the constant daily average — see
    :func:`_mean_solar_power`) + the derived ``load_power`` (``balanced_load_w``, sized
    so
    Power's net battery flux is zero, leaving the lamp as the battery's only net drain).
    The donor-controlled flows (ECLSS loops, ``WaterRecovery``, ``RadiatorReject``,
    ``Harvest``) read stocks, not ``env``, so they add no forcing.
    """
    return SourceResolver(
        forcings={
            FOOD_INTAKE_VAR: constant(scenario.cabin.food_intake_rate),
            WATER_INTAKE_VAR: constant(scenario.cabin.water_intake_rate),
            LAMP_POWER_VAR: constant(_lighting_average_power(scenario)),
            SOLAR_POWER_VAR: constant(_mean_solar_power(scenario)),
            LOAD_POWER_VAR: constant(balanced_load_w(charge, scenario.power)),
        }
    )


def sealed_reset(
    scenario: SealedStationScenario = SEALED_STATION_SCENARIO,
) -> Callable[[int, State], State]:
    """The slow-domain re-sow hook: ``annual_reset`` at each season boundary.

    The two-rate analogue of :func:`domains.biosphere.season.run_perennial`'s reset
    closure — fires ``annual_reset`` (die-to-litter + re-sow from the grain seed bank,
    ``thermal_time`` → 0) at each ``n % season_days == 0`` (``n > 0``). Handed to
    :func:`station.driver.run_master_day`'s ``slow_reset``, which re-asserts the coupled
    conservation gate across it (``annual_reset`` is CARBON-conserving, touching no
    other
    quantity). Without it the multi-year biosphere would never re-sow — the machinery
    the
    ≤7-day station runs never exercised.
    """

    def reset(n: int, current: State) -> State:
        if n > 0 and n % scenario.season_days == 0:
            return annual_reset(current, scenario.bio)
        return current

    return reset


def run_sealed(
    bio_integrator: EulerIntegrator,
    fast_integrator: EulerIntegrator,
    state: State,
    bio_resolver: SourceResolver,
    fast_resolver: SourceResolver,
    scenario: SealedStationScenario = SEALED_STATION_SCENARIO,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """The two-rate driver over the multi-year horizon, with the annual re-sow hook.

    A thin wrapper over :func:`station.driver.run_master_day` with the biosphere slow
    (once/day, advancing ``thermal_time`` + ``n``), everything else fast (``substep``
    ×``steps_per_day``, ``n`` kept, conservation asserted after each), and the
    :func:`sealed_reset` closure as ``slow_reset`` (re-sow each season boundary).
    ``states`` holds one entry per day boundary (length ``days + 1``; the golden pins
    the
    final one); ``total_rationed`` sums the Euler-backstop firings (validation asserts
    ``== 0``); ``events`` are extinction events (empty — the well-fed station does not
    go
    extinct). Euler-only (the biosphere is Euler-locked by its freeze).
    """
    return run_master_day(
        bio_integrator,
        fast_integrator,
        state,
        bio_resolver,
        fast_resolver,
        days=scenario.days,
        steps_per_day=scenario.steps_per_day,
        slow_dt=scenario.bio_dt,
        fast_dt=scenario.cabin_dt,
        slow_reset=sealed_reset(scenario),
    )
