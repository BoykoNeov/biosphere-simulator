"""The Station greenhouse: couple the frozen biosphere ↔ the cabin air (P6.3).

Step 3's assembly — the frozen sealed biosphere's gas exchange meets the Step-2 cabin,
so plants + soil microbes + crew all breathe **one** cabin-air stock and the CO₂/O₂
feedback (crew exhales → cabin CO₂ rises → plants assimilate → cabin O₂ rises → crew
breathes) emerges with **no control code**. It imports both the biosphere and the cabin
siblings and wires them at the shared gas pools; **no domain imports another**
(finding #1).

**The seam is REVERSED from the plan's first framing (see the phase-6 plan Step 3).**
The naive seam ("point the biosphere's ``ChamberWiring`` at the cabin's CO₂/O₂ ids") is
blocked: only ``plants`` consumes the wiring, while ``soil.MicrobialRespiration`` (built
for **every** sealed chamber) and ``consumers.ConsumerRespiration`` read the pool ids
``CARBON_POOL`` / ``O2_POOL`` from the **catalog, hardcoded** — so re-pointing the
wiring would redirect only plant gas and leave microbial gas dangling. Instead we **keep
the biosphere's ``CARBON_POOL`` (``{C:1,O:2}``) / ``O2_POOL`` (``{O:2}``) as the shared
cabin air and re-point the CABIN's five all-parameterised flows** (``CrewRespiration`` /
``CO2Scrubber`` / ``O2Makeup`` / ``Condenser`` / ``WaterBalance``) at those ids — the
side that *can* be re-pointed. This reuses :func:`build_season` **wholesale**
(``build_atmosphere`` included, the CARBON loss-sink included, the default ``sealed``
wiring + the default ``{CO2_POOL_VAR: CARBON_POOL}`` Ci map all unchanged), needs **zero
frozen / zero domain / zero core change**, and is the correct closed-station physics.

**The two-rate master-step driver (:func:`run_greenhouse`).** The biosphere is
structurally ``dt = 1`` **day** (weather indexed by the integer step count ``n``); the
cabin is ``dt = 60 s`` **per second** (ECLSS ``k_scrub·dt < 1``). These are different
time units, which ``simcore.multirate`` cannot bridge — it splits one shared master
``dt`` (``dt/n_sub``) and composes ``substep`` only, which by design freezes the
biosphere's ``thermal_time`` aux (phenology). So the driver does the operator split **by
hand**: per day, the biosphere ``step_report`` runs once (advancing aux **and** ``n``),
then the cabin sub-steps ``steps_per_day`` times (keeping ``n``, conservation asserted
after each — ``substep`` skips it). Because only the biosphere bumps ``n``, ``n`` stays
the day count and the frozen weather resolver's ``_table(n)`` is reused unchanged. This
is the shared :func:`station.driver.run_master_day` (the greenhouse was its first
instance; lighting the second) — :func:`run_greenhouse` is a thin biosphere-slow /
cabin-fast wrapper over it.

**Scale (illustrative; calibration deferred to Step 9).** The crew dominates the cabin
gas (~345 mol C/day) vs the 1 m² seedling (~0.1 mol C/day); the ECLSS scrubber holds
``CARBON_POOL ≈ P/k_scrub`` and the makeup holds ``O2_POOL`` near the setpoint, so the
plant is a tiny (~3e-4 relative) perturbation — Step 3 is a **direction / conservation**
demo (plant is a net CO₂ sink / O₂ source; every quantity closes; the biosphere's
internal water + N loops still close), not a magnitude-balanced loop.

Pure stdlib only in the spine; crew / ECLSS / crop params load via the sibling loaders.
"""

from collections.abc import Mapping

from domains.biosphere.season import build_season, weather_resolver
from domains.biosphere.stocks import (
    CARBON_POOL,
    O2_POOL,
    THERMAL_TIME,
    VERNALIZATION_DAYS,
)
from domains.crew.flows import CrewParams, WaterBalance
from domains.crew.stocks import (
    FECAL_WASTE,
    FOOD_INTAKE_VAR,
    FOOD_STORE,
    URINE,
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
    HUMIDITY_CONDENSATE,
    O2_SUPPLY,
    cabin_h2o_stock,
)
from domains.eclss.system import CO2_SCRUBBER, CONDENSER, O2_MAKEUP
from simcore import boundary
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.environment import SourceResolver, constant
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock
from station.cabin import CO2_COMPOSITION, CREW_RESPIRATION, O2_COMPOSITION
from station.driver import run_master_day
from station.flows import CrewRespiration
from station.scenario import GREENHOUSE_SCENARIO, GreenhouseScenario

GreenhouseIntegrator = EulerIntegrator | Rk4Integrator


def _gas_boundary(
    stock_id: StockId,
    quantity: Quantity,
    composition: Mapping[Quantity, float],
    *,
    unclamped: bool,
) -> Stock:
    """A composition-carrying BOUNDARY reservoir (``co2_removed`` sink / ``o2_supply``).

    The ECLSS scrubber removes 2 O per CO₂ into ``co2_removed`` and the makeup adds 2 O
    per O₂ from ``o2_supply``; both reservoirs must carry the gas composition or OXYGEN
    unbalances at the boundary (the ``station.cabin`` rationale, one module over — the
    ``boundary.source`` / ``sink`` helpers cannot set composition, and extending them is
    a core change).
    """
    return Stock(
        id=stock_id,
        domain=BOUNDARY_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=0.0,
        kind=StockKind.BOUNDARY,
        unclamped=unclamped,
        composition=composition,
    )


def _cabin_flows(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    fecal_waste_target: StockId = FECAL_WASTE,
) -> list[Flow]:
    """The five cabin flows, re-pointed at the biosphere's gas pools (the reverse seam).

    ``CrewRespiration`` / ``CO2Scrubber`` act on ``CARBON_POOL`` (the shared cabin CO₂);
    ``O2Makeup`` / ``CrewRespiration`` on ``O2_POOL`` (the shared cabin O₂);
    ``Condenser`` / ``WaterBalance`` on the cabin-owned ``CABIN_H2O`` (WATER stays
    decoupled from the biosphere's internal water cycle — Step 4 closes the *crew* water
    loop independently on the cabin; unifying the two humid-air stocks is a deferred
    fidelity refinement, not a closure requirement). Identical construction to
    ``station.cabin.build_cabin`` except the two gas ids are the biosphere's, not the
    ECLSS ``cabin_o2`` / ``cabin_co2``.

    ``fecal_waste_target`` is where ``CrewRespiration`` egests its fecal carbon; the
    default ``FECAL_WASTE`` boundary sink is the greenhouse's open loop (Steps 3–5).
    Step 6 seam 2 passes the biosphere's ``LITTER_CARBON`` pool to close the trophic
    CARBON ring (feces → litter → microbes → CO₂); the caller then omits the
    ``FECAL_WASTE`` boundary sink (see :func:`build_greenhouse`), so the re-point is
    structural (no shadow sink). Only ``build_harvest`` passes a non-default target —
    the default keeps the greenhouse / lighting / water goldens byte-identical.
    """
    return [
        CrewRespiration(
            CREW_RESPIRATION,
            0,
            food_store=FOOD_STORE,
            cabin_co2=CARBON_POOL,  # the seam: crew exhales into the biosphere CO₂ pool
            cabin_o2=O2_POOL,  # the seam: crew breathes the biosphere O₂ pool
            fecal_waste=fecal_waste_target,
            respired_carbon_fraction=crew_params.respired_carbon_fraction,
        ),
        WaterBalance(
            WATER_BALANCE,
            0,
            water_store=WATER_STORE,
            crew_humidity=CABIN_H2O,
            urine=URINE,
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
            humidity_condensate=HUMIDITY_CONDENSATE,
            params=eclss_params,
        ),
        O2Makeup(
            O2_MAKEUP, 0, o2_supply=O2_SUPPLY, cabin_o2=O2_POOL, params=eclss_params
        ),
    ]


def build_greenhouse(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    scenario: GreenhouseScenario = GREENHOUSE_SCENARIO,
    *,
    with_plants: bool = True,
    fecal_waste_target: StockId = FECAL_WASTE,
) -> tuple[State, Registry, Registry]:
    """Assemble the greenhouse: ``(state, bio_reg, cabin_reg)`` (biosphere ↔ cabin).

    Two disjoint registries over one shared stock dict (the multi-rate model). The
    **biosphere** registry is :func:`build_season`'s output verbatim (all sealed-chamber
    stocks — incl. ``CARBON_POOL`` ``{C:1,O:2}`` / ``O2_POOL`` ``{O:2}`` — plus the
    CARBON loss-sink and the ``thermal_time`` aux). The **cabin** registry is the five
    cabin flows re-pointed at those gas ids (:func:`_cabin_flows`), over the union of
    the biosphere stocks + the cabin-only stocks (the two finite crew stores,
    ``cabin_h2o``, and five boundary reservoirs — the gas ones composition-carrying).

    The two stock-id sets are asserted **disjoint** (biosphere ``biosphere.*`` /
    ``boundary.*`` vs cabin ``crew.*`` / ``eclss.*`` / ``boundary.*`` reservoirs): a
    silent dict overwrite would be a hard-to-see wiring bug.

    ``with_plants=False`` builds the **no-plant baseline**: the identical stock set +
    cabin flows, but an **empty** biosphere registry (no biosphere flows, no aux) — so
    ``CARBON_POOL`` / ``O2_POOL`` relax to the crew-driven ECLSS steady state with no
    plant draw. The signed feedback gate compares the two (with-plants CO₂ lower / O₂
    higher).

    ``fecal_waste_target`` is where ``CrewRespiration`` egests fecal carbon. The default
    ``FECAL_WASTE`` is the greenhouse's open loop, and its boundary sink is included in
    the cabin stocks. Step 6 seam 2 (:func:`station.harvest.build_harvest`) passes the
    biosphere's ``LITTER_CARBON`` to close the trophic CARBON ring — a **biosphere**
    stock already in the shared dict, so the ``FECAL_WASTE`` boundary sink is then
    **omitted** (the re-point is structural, no shadow sink — the Step-1 property). The
    default keeps the greenhouse / lighting / water goldens byte-identical.
    """
    bio_state, full_bio_registry = build_season(scenario.bio)
    bio_stocks = dict(bio_state.stocks)

    cabin_stock_seq = [
        food_store_stock(scenario.cabin.food_store0),
        water_store_stock(scenario.cabin.water_store0),
        cabin_h2o_stock(scenario.cabin.cabin_h2o_0),
        _gas_boundary(O2_SUPPLY, Quantity.OXYGEN, O2_COMPOSITION, unclamped=True),
        _gas_boundary(CO2_REMOVED, Quantity.CARBON, CO2_COMPOSITION, unclamped=False),
        boundary.sink(HUMIDITY_CONDENSATE, Quantity.WATER, 0.0),
        boundary.sink(URINE, Quantity.WATER, 0.0),
    ]
    # The FECAL_WASTE boundary sink exists only for the open loop; when feces is
    # re-pointed into the biosphere (seam 2) it would be an orphan shadow sink, so it is
    # omitted — feces then lands in LITTER_CARBON (already a biosphere stock).
    if fecal_waste_target == FECAL_WASTE:
        cabin_stock_seq.append(boundary.sink(FECAL_WASTE, Quantity.CARBON, 0.0))
    cabin_stocks = {s.id: s for s in cabin_stock_seq}
    overlap = bio_stocks.keys() & cabin_stocks.keys()
    if overlap:
        raise ValueError(
            "greenhouse stock-id collision between the biosphere and the cabin: "
            f"{sorted(overlap)} (the two stock sets must be disjoint — a silent dict "
            "overwrite would drop a stock)"
        )
    stocks = {**bio_stocks, **cabin_stocks}
    state = State(
        n=0,
        stocks=stocks,
        rng_seed=0,
        aux={THERMAL_TIME: 0.0, VERNALIZATION_DAYS: 0.0},
    )

    cabin_registry = Registry(
        _cabin_flows(crew_params, eclss_params, fecal_waste_target), stocks
    )
    # with_plants=False ⇒ the biosphere contributes no flows/aux (the no-plant
    # baseline); ``CARBON_POOL``/``O2_POOL`` then move only under the cabin (crew +
    # ECLSS).
    bio_registry = full_bio_registry if with_plants else Registry((), stocks)
    return state, bio_registry, cabin_registry


def greenhouse_bio_resolver(
    weather: list[dict[str, float | str]],
    scenario: GreenhouseScenario = GREENHOUSE_SCENARIO,
) -> SourceResolver:
    """The biosphere's forcing resolver — the frozen ``weather_resolver``, reused as-is.

    Carries the per-day weather ``_table(n)`` forcings **and** the default sealed
    ``shared`` map ``{SOIL_WATER_VAR: soil_water, CO2_POOL_VAR: CARBON_POOL}`` — the
    reverse seam means ``CO2_POOL_VAR`` still points at ``CARBON_POOL`` (now the cabin
    air), so FvCB's ``Ci`` reads the live shared gas pool with **no** map surgery. The
    biosphere ``step_report`` reads it at ``n`` = the day count.
    """
    return weather_resolver(weather, scenario.bio)


def greenhouse_cabin_resolver(
    scenario: GreenhouseScenario = GREENHOUSE_SCENARIO,
) -> SourceResolver:
    """The cabin's forcing resolver — the two constant crew intake rates.

    Kept **separate** from the biosphere resolver (each integrator reads its own): the
    two domains' forcing vars are disjoint and their steps happen at different points in
    the master step, so a single merged resolver buys nothing. O₂ intake is derived via
    RQ = 1 inside ``CrewRespiration`` (not a forcing var); the ECLSS loops read stocks,
    not ``env``.
    """
    return SourceResolver(
        forcings={
            FOOD_INTAKE_VAR: constant(scenario.cabin.food_intake_rate),
            WATER_INTAKE_VAR: constant(scenario.cabin.water_intake_rate),
        }
    )


def run_greenhouse(
    bio_integrator: GreenhouseIntegrator,
    cabin_integrator: GreenhouseIntegrator,
    state: State,
    bio_resolver: SourceResolver,
    cabin_resolver: SourceResolver,
    scenario: GreenhouseScenario = GREENHOUSE_SCENARIO,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """The two-rate master-step driver: one day per master step (biosphere-slow /
    cabin-fast).

    A thin wrapper over the shared :func:`station.driver.run_master_day` with the
    biosphere as the **slow** domain (once/day at ``bio_dt``, advancing ``thermal_time``
    + ``n``) and the cabin as the **fast** domain (``substep`` ×``steps_per_day`` at
    ``cabin_dt``, ``n`` kept). ``states`` holds one entry per day boundary (length
    ``days + 1``; the golden pins the final one); ``total_rationed`` sums the
    Euler-backstop firings (validation asserts ``== 0``); ``events`` are extinction
    events (empty — the crew has no POPULATION stock and the well-fed greenhouse does
    not go extinct). The driver requires ``cabin_dt · steps_per_day == 86400`` (one day)
    so the biosphere's once-daily step maps ``n`` to the day the weather table indexes.
    """
    return run_master_day(
        bio_integrator,
        cabin_integrator,
        state,
        bio_resolver,
        cabin_resolver,
        days=scenario.days,
        steps_per_day=scenario.steps_per_day,
        slow_dt=scenario.bio_dt,
        fast_dt=scenario.cabin_dt,
    )
