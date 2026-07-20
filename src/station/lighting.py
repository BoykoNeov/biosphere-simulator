"""The Station lighting seam: Power → biosphere grow lamp (P6.5) — energy into biology.

Step 5's assembly — the fifth cross-domain seam, and the phase's **one non-shared-stock
coupling** (finding #3 / #16). Power and the biosphere share *no* stock; the whole
interface is the **lamp-draw schedule**, which drives both:

  * the station-owned :class:`station.flows.Lamp` flow (the ENERGY it withdraws from
    ``power.battery`` → ``light_used`` + ``waste_heat``), and
  * the biosphere's ``par`` / ``daylength_s`` **forcings** — this module computes ``PAR
    = photon_efficacy · lamp_power_w / ground_area`` (:func:`lamp_par`) and
    ``daylength_s = photoperiod_hours · 3600`` from the same schedule.

A flow cannot tell a forcing from a shared stock (#16), so the **frozen biosphere is
untouched**: PAR stays a forcing, merely *computed from the lamp* instead of read from a
weather table. Zero frozen / zero domain / zero core change (finding #3).

**The daylength coupling (the correctness crux, advisor).** ``incident_par`` returns a
*daytime-mean* photon flux and the FvCB aggregator re-multiplies by ``daylength_s`` for
the daily photon dose (dose = PAR × daylength). Overriding PAR alone would silently
corrupt the dose. So **both** PAR and ``daylength_s`` come from the lamp. The only
runtime consumer of ``daylength_s`` is photosynthesis (phenology / transpiration /
net-radiation do not read it), so "day = lamp photoperiod" is consistent everywhere it
is read. The chamber's non-light forcings (temperature, VPD, net radiation) stay
weather-driven (reused from the winter-wheat fixture — the greenhouse precedent); a
fully controlled-environment chamber is a deferred refinement.

**The lamp's daily-average draw (a consequence of the frozen-``n`` fast domain).** Under
the two-rate driver the biosphere lumps once per day and Power sub-steps 24× — but
``substep`` **keeps** ``n`` (the day count), so a within-day top-hat (lamp on
``photoperiod_hours``, off the rest) is not expressible as an ``n``-schedule. The
biosphere already carries the photoperiod *internally* via ``daylength_s``; Power only
has to draw the correct **daily energy**. So the lamp flow reads a constant
**daily-average** power ``lamp_power_w · photoperiod_hours / 24``
(:func:`lamp_average_power`): its daily energy is ``lamp_power_w · photoperiod_hours ·
3600`` **exactly**, and so are the daily ``light_used`` (η_lamp × that) and
``waste_heat`` legs. Only the unobserved intra-day *instantaneous* power is smeared;
every daily total — and the biosphere's PAR × daylength dose — is exact. The biosphere's
PAR uses the **on-window** intensity ``lamp_power_w · efficacy / ground_area`` (the
plant sees the lamp at full brightness for ``photoperiod_hours``), not the average.

**Minimal Power (the lamp is the whole load).** The Power side is a battery POOL + the
Lamp flow only — no ``SolarCharge`` / ``LoadDraw`` (they re-show Step-1 machinery with
no new thesis). The battery is a finite provisioned energy store draining via the lamp
(the Crew-store pattern); positivity is well-fed sizing (``rationed == 0``), not
structural. The lamp's ``waste_heat`` lands in a ``boundary.waste_heat`` sink (the
standalone-Power seam); moving it inward to ``thermal.node`` is deferred to the
sealed-station step — the "boundary now, inward later" rhythm Power's own dissipation
followed. (This deviates from the plan's parenthetical ``(→ thermal.node)``, which would
only re-test Step-1's node seam — the Steps-3/4 precedent for correcting the plan's
first framing.)

**The two-rate driver (see ``station.driver.run_master_day``).** The biosphere is
structurally ``dt = 1`` day; Power runs sub-daily (``power_dt = 3600`` s ×
``steps_per_day = 24``). Per master day the biosphere ``step_report`` runs once
(advancing phenology aux **and** ``n``), then Power ``substep`` ×24 (``n`` kept,
conservation asserted after each) — the greenhouse rhythm with Power as the fast domain.
Because Power and the biosphere share no stock, the combined ledger balances
per-quantity trivially (each flow touches only its own domain's stocks): ENERGY over the
Power stocks, mass over the biosphere stocks.

**The payload — the signed "it bit" contrast.** With the lamp on the seedling
net-assimilates under the lamp (``bio_organic_C`` grows); with the lamp **off** (PAR =
0) gross assimilation is 0, so the plant only respires and ``bio_organic_C`` declines —
the lamp genuinely carries the energy that drives carbon fixation. Plus: ENERGY closed
every step (the lamp names every joule), the biosphere internal CARBON / OXYGEN / WATER
/ NITROGEN loops still close, ``rationed == 0``, ``events == ()``. Euler-only (the
biosphere is Euler-locked by its freeze — no RK4 cross-check, matching the greenhouse).

Pure stdlib only in the spine; the crop params load via the biosphere loaders, the lamp
photon efficacy via ``station.loader``.
"""

from domains.biosphere.season import build_season, weather_resolver
from domains.biosphere.stocks import (
    DAYLENGTH_VAR,
    PAR_VAR,
    THERMAL_TIME,
    VERNALIZATION_DAYS,
)
from domains.power.stocks import BATTERY, WASTE_HEAT, battery_stock
from simcore import boundary
from simcore.environment import SourceResolver, constant
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State
from station.driver import run_master_day
from station.flows import LAMP_POWER_VAR, Lamp, LampParams
from station.scenario import LIGHTING_SCENARIO, LightingScenario

LightingIntegrator = EulerIntegrator | Rk4Integrator

# The radiant-PAR-energy boundary sink (the ``η_lamp`` leg of the Lamp flow): the photon
# energy the plants receive. ENERGY; a legitimate boundary Output (the photons'
# downstream fate as absorbed heat is out of scope — the standalone-Power ``waste_heat``
# seam, one leg over). NOT a biosphere ENERGY stock: the biosphere tracks PAR as a
# *forcing*, never an energy inventory, so light_used and the PAR forcing are disjoint
# accountings (ENERGY vs the CARBON that PAR drives) — no double-count.
LIGHT_USED: StockId = StockId("boundary.light_used")

# The station-owned grow-lamp flow id (ASCII so str sort == future Rust byte sort, #15).
LAMP: FlowId = FlowId("station.lamp")


def lamp_par(
    lamp_params: LampParams, scenario: LightingScenario = LIGHTING_SCENARIO
) -> float:
    """The on-window PAR photon flux the lamp delivers (µmol photons m⁻² s⁻¹).

    ``photon_efficacy · lamp_power_w / ground_area`` — the lamp's electrical draw turned
    into a PAR photon flux (efficacy) spread over the illuminated chamber footprint (the
    biosphere's own ``ground_area``, so the lamp lights exactly the plant's plot). This
    is the biosphere's ``par`` forcing under the lamp, replacing the weather-table PAR.
    The **on-window** intensity (the lamp at full brightness), paired with ``daylength_s
    = photoperiod_hours·3600`` for the daily dose — NOT the daily-average the battery
    draw uses (:func:`lamp_average_power`). Reconstructed byte-for-byte by the
    validation gate (the ``f_N``-style "the factor actually bit" check).
    """
    return (
        lamp_params.photon_efficacy * scenario.lamp_power_w / scenario.bio.ground_area
    )


def lamp_average_power(scenario: LightingScenario = LIGHTING_SCENARIO) -> float:
    """The constant daily-average lamp electrical power the Lamp flow draws (W).

    ``lamp_power_w · photoperiod_hours / 24`` — the on-window power smeared over the
    full day, so the daily energy ``· 86400 = lamp_power_w · photoperiod_hours · 3600``
    is **exactly** the real (on ``photoperiod_hours`` at ``lamp_power_w``) lamp's daily
    energy. The daily-average form is forced by the frozen-``n`` fast domain
    (``substep`` keeps ``n``, so a within-day top-hat is not an ``n``-schedule); the
    biosphere carries the photoperiod internally via ``daylength_s``, so Power need only
    match the daily energy. Every daily total (energy / ``light_used`` / ``waste_heat``)
    is exact; only the unobserved intra-day instantaneous power is smeared.
    """
    return scenario.lamp_power_w * scenario.photoperiod_hours / 24.0


def build_lighting(
    lamp_params: LampParams,
    scenario: LightingScenario = LIGHTING_SCENARIO,
    *,
    with_lamp: bool = True,
) -> tuple[State, Registry, Registry]:
    """Assemble the lighting station: ``(state, bio_registry, power_registry)``.

    Two disjoint registries over one shared stock dict (the two-rate model). The
    **biosphere** registry is :func:`build_season`'s output verbatim (the sealed
    self-contained chamber — its own CARBON/OXYGEN/WATER/NITROGEN stocks + flows + the
    ``thermal_time`` aux). The **Power** registry is the single
    :class:`station.flows.Lamp` flow over the biosphere stocks ∪ the three Power ENERGY
    stocks: the ``power.battery`` POOL (initial ``battery0``), the
    ``boundary.light_used`` sink, and the ``boundary.waste_heat`` sink.

    The two stock-id sets are asserted **disjoint** (``biosphere.*`` / ``boundary.*``
    biosphere reservoirs vs ``power.battery`` / ``boundary.light_used`` /
    ``boundary.waste_heat``): a silent dict overwrite would be a hard-to-see wiring bug.

    ``with_lamp=False`` builds the **lamp-off baseline**: the identical stock set
    (battery included), but an **empty** Power registry — so the Lamp draws nothing (the
    battery stays flat) and the biosphere is driven with PAR = 0
    (:func:`lighting_bio_resolver` with ``with_lamp=False``). The plant is still present
    but unlit; the signed feedback gate compares the two (lamp-on grows, lamp-off
    declines).
    """
    bio_state, bio_registry = build_season(scenario.bio)
    bio_stocks = dict(bio_state.stocks)

    battery = battery_stock(scenario.battery0)
    light_used = boundary.sink(LIGHT_USED, Quantity.ENERGY, 0.0)
    waste_heat = boundary.sink(WASTE_HEAT, Quantity.ENERGY, 0.0)
    power_stocks = {s.id: s for s in (battery, light_used, waste_heat)}

    overlap = bio_stocks.keys() & power_stocks.keys()
    if overlap:
        raise ValueError(
            "lighting stock-id collision between the biosphere and Power: "
            f"{sorted(overlap)} (the two stock sets must be disjoint — Power and the "
            "biosphere share NO stock, only the lamp-draw schedule)"
        )
    stocks = {**bio_stocks, **power_stocks}
    state = State(
        n=0,
        stocks=stocks,
        rng_seed=0,
        aux={THERMAL_TIME: 0.0, VERNALIZATION_DAYS: 0.0},
    )

    power_flows: list[Flow] = (
        [
            Lamp(
                LAMP,
                0,
                battery=BATTERY,
                light_used=LIGHT_USED,
                waste_heat=WASTE_HEAT,
                params=lamp_params,
            )
        ]
        if with_lamp
        else []
    )
    power_registry = Registry(power_flows, stocks)
    return state, bio_registry, power_registry


def lighting_bio_resolver(
    weather: list[dict[str, float | str]],
    lamp_params: LampParams,
    scenario: LightingScenario = LIGHTING_SCENARIO,
    *,
    with_lamp: bool = True,
) -> SourceResolver:
    """The biosphere's forcing resolver: weather-driven, PAR + daylength from the lamp.

    Starts from the frozen ``weather_resolver`` (temperature / VPD / net-radiation / Ci
    / irrigation / fertilization — and the sealed ``shared`` map ``{SOIL_WATER_VAR:
    soil_water, CO2_POOL_VAR: carbon_pool}``, so FvCB's Ci reads the live chamber CO₂
    unchanged), then **overrides two forcings from the lamp**: ``PAR_VAR`` →
    :func:`lamp_par` (0 when ``with_lamp=False`` — the dark baseline) and
    ``DAYLENGTH_VAR`` → ``photoperiod_hours·3600``. Both must come from the lamp
    together (PAR and daylength are coupled in the daily photon dose); ``daylength_s``
    stays positive even when unlit (a valid, zero-dose integration window), so the
    biosphere never divides by a zero day-length.
    """
    base = weather_resolver(weather, scenario.bio)
    par = lamp_par(lamp_params, scenario) if with_lamp else 0.0
    forcings = dict(base.forcings)
    forcings[PAR_VAR] = constant(par)
    forcings[DAYLENGTH_VAR] = constant(scenario.photoperiod_hours * 3600.0)
    return SourceResolver(forcings=forcings, shared=dict(base.shared))


def lighting_power_resolver(
    scenario: LightingScenario = LIGHTING_SCENARIO,
) -> SourceResolver:
    """The Power forcing: the constant daily-average ``lamp_power`` draw.

    The single ``lamp_power`` forcing the :class:`station.flows.Lamp` reads — the
    daily-average power (:func:`lamp_average_power`), constant because the fast domain's
    ``n`` is frozen within a day (``substep`` keeps ``n``). Kept **separate** from the
    biosphere resolver (each integrator reads its own; the two domains' forcing vars are
    disjoint). When the Power registry is empty (the lamp-off baseline) this forcing is
    simply unread — harmless.
    """
    return SourceResolver(
        forcings={LAMP_POWER_VAR: constant(lamp_average_power(scenario))}
    )


def run_lighting(
    bio_integrator: LightingIntegrator,
    power_integrator: LightingIntegrator,
    state: State,
    bio_resolver: SourceResolver,
    power_resolver: SourceResolver,
    scenario: LightingScenario = LIGHTING_SCENARIO,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """The two-rate driver: one master day = biosphere-slow (once) + Power-fast (×24).

    A thin wrapper over the shared :func:`station.driver.run_master_day` with the
    biosphere as the **slow** domain (once/day at ``bio_dt``, advancing ``thermal_time``
    + ``n``) and Power as the **fast** domain (``substep`` ×``steps_per_day`` at
    ``power_dt``, ``n`` kept). ``states`` holds one entry per day boundary (length
    ``days + 1``; the golden pins the final one); ``total_rationed`` sums the
    Euler-backstop firings (validation asserts ``== 0``); ``events`` are extinction
    events (empty — the well-fed lit chamber does not go extinct). The driver requires
    ``power_dt · steps_per_day == 86400`` (one day) so the biosphere's once-daily step
    maps ``n`` to the day the weather table indexes.
    """
    return run_master_day(
        bio_integrator,
        power_integrator,
        state,
        bio_resolver,
        power_resolver,
        days=scenario.days,
        steps_per_day=scenario.steps_per_day,
        slow_dt=scenario.bio_dt,
        fast_dt=scenario.power_dt,
    )
