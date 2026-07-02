"""The Station biomass/food loop: biosphere grain → crew ``food_store`` (P6.6).

Step 6's assembly — built on the Step-3 :mod:`station.greenhouse` (the only assembly
where a live plant already shares the cabin air, so it is where biomass can flow into
food). The seam adds **one** station-owned flow, :class:`station.flows.Harvest`
(``storage_c → food_store``, donor-controlled), to the cabin / fast registry: the
biosphere fixes cabin CO₂ into grain (``storage_c``) and the harvest moves that grain
into the crew's finite ``food_store``, so the store — open-loop and monotonically
depleting in standalone Crew / the greenhouse — becomes **regenerative** up to the
harvest it captures. It is the **CARBON twin of Step 4's** ``WaterRecovery`` (WATER),
one trophic level.

**This is seam 1 of the two the plan splits (advisor-flagged).** It lands the
``Harvest`` flow alone; the ``fecal_waste → litter_carbon`` re-pointing (seam 2, which
makes the CARBON ring fully *closed*) is a **separate** increment, because crew-scale
feces dumped into a seedling-scale litter pool dominates the litter dynamics (~3400×
mismatch) and must be understood on its own, not conflated with the harvest signal.

**Why the reproductive plant.** ``storage_c`` fills only after anthesis (``FO > 0``
needs ``DVS > 1``), and the default seedling sits at ``DVS < 1`` with ``storage_c0 = 0``
— a zero harvest source. :attr:`station.scenario.HarvestScenario.thermal_time0` starts
the biosphere phenology accumulator **past** anthesis (``tsum_anthesis = 1100`` °C·day),
so grain is actively *filling* while harvest drains it — a genuinely regenerative
source. This is injected at :class:`State` construction here (the station owns the
greenhouse ``State``'s ``thermal_time`` aux), so ``SeasonScenario`` is untouched (a
``thermal_time0`` field there would be a domain change, forbidden by the Phase-6 exit
criterion).

**The exact with/without-harvest identity (the "it bit" payload).** The ``Allocation``
grain-fill leg (``FO·DMI``) is independent of ``storage_c``'s own level, and no other
biosphere flow reads ``storage_c`` (only ``annual_reset``, which does not fire in a
≤7-day run), so harvest is its **only** new sink and does not perturb the plant's carbon
budget. Grain fill is therefore *identical* with and without harvest, so the two-way
identity ``Δfood_store = cumulative harvest = Δstorage_c`` holds to floating point (the
``with_harvest`` baseline arm drops the ``Harvest`` flow: ``storage_c`` then accumulates
all fill and ``food_store`` depletes at the full crew rate). ``CrewRespiration`` is
**forced** (reads the intake schedule, not ``food_store``), so the regenerated (higher)
``food_store`` in the harvest arm does not perturb the crew CO₂ draw → the cabin gas
trajectory is identical too.

**The two-rate driver + Euler-only.** Runs under the shared
:func:`station.driver.run_master_day` (the greenhouse rhythm: biosphere slow once/day —
refilling grain — then cabin fast ×``steps_per_day`` — draining it), so the day-boundary
snapshot is the intra-day *minimum* ``storage_c`` (post-drain), which the harvest rate
is sized to keep positive (a regenerative source, not a static reservoir emptied). The
greenhouse biosphere is **Euler-locked by its freeze**, so this is an Euler-only run (no
RK4 cross-check — the ``WaterRecovery`` "state-dependent breaks RK4≡Euler" signal does
not apply on a biosphere-coupled build).

Pure stdlib only in the spine; the harvest rate loads via ``station.loader``. Reuses
``build_greenhouse`` wholesale (zero greenhouse change) — this module only adds the one
flow + the phenology injection.
"""

from domains.biosphere.stocks import STORAGE_C, THERMAL_TIME
from domains.crew.flows import CrewParams
from domains.crew.stocks import FOOD_STORE
from domains.eclss.flows import EclssParams
from simcore.environment import SourceResolver
from simcore.events import Event
from simcore.ids import FlowId
from simcore.integrator import EulerIntegrator
from simcore.registry import Registry
from simcore.state import State
from station.driver import run_master_day
from station.flows import Harvest, HarvestParams
from station.greenhouse import (
    build_greenhouse,
    greenhouse_bio_resolver,
    greenhouse_cabin_resolver,
)
from station.scenario import HARVEST_SCENARIO, HarvestScenario

# The station-owned harvest flow id (the biosphere-grain → crew-food seam). Distinct
# from every cabin flow id (``station.crew_respiration`` etc.) and every biosphere flow
# id — the flow-registry disjointness assert in ``build_harvest`` enforces it.
HARVEST: FlowId = FlowId("station.harvest")


def build_harvest(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    harvest_params: HarvestParams,
    scenario: HarvestScenario = HARVEST_SCENARIO,
    *,
    with_harvest: bool = True,
) -> tuple[State, Registry, Registry]:
    """Assemble the harvest greenhouse: ``(state, bio_reg, cabin_reg)``.

    Reuses :func:`station.greenhouse.build_greenhouse` (the sealed biosphere ↔ cabin gas
    loop, verbatim — zero greenhouse change) and layers on the two Step-6 additions:

      1. the biosphere ``thermal_time`` aux is started at ``scenario.thermal_time0``
         (past anthesis ⇒ a grain-filling plant), overriding the greenhouse's ``0.0`` —
         the station owns the ``State``'s aux dict, so ``SeasonScenario`` stays
         untouched; and
      2. the :class:`station.flows.Harvest` flow (``storage_c → food_store``) is
         appended to the cabin / fast registry (``with_harvest=True``).

    ``with_harvest=False`` is the **baseline arm**: the identical greenhouse +
    phenology, but **no** ``Harvest`` flow — so ``storage_c`` accumulates all the grain
    fill and ``food_store`` depletes at the full crew rate (the "it bit" gate compares
    the two).

    The two flow registries (biosphere-slow vs cabin-fast) are asserted to have
    **disjoint flow-id sets** over the one shared stock dict: a duplicate ``FlowId``
    across the two registries would be a silent cross-domain wiring bug (each
    ``Registry`` already rejects a duplicate *within* itself, but not across the pair
    the driver steps together).
    """
    state, bio_reg, cabin_reg = build_greenhouse(
        crew_params, eclss_params, scenario.greenhouse, with_plants=True
    )

    # (1) Start the biosphere phenology past anthesis (a grain-filling plant). The
    # station owns the greenhouse State's aux dict, so this is a station-level injection
    # — no SeasonScenario / domain change.
    state = State(
        n=state.n,
        stocks=state.stocks,
        rng_seed=state.rng_seed,
        aux={THERMAL_TIME: scenario.thermal_time0},
    )

    # (2) Append the Harvest flow to the cabin / fast registry (drained across the day's
    # sub-steps; the biosphere refills grain in its once-daily slow step).
    cabin_flows = list(cabin_reg.flows)
    if with_harvest:
        cabin_flows.append(
            Harvest(
                HARVEST,
                0,
                storage_c=STORAGE_C,
                food_store=FOOD_STORE,
                params=harvest_params,
            )
        )
    cabin_reg = Registry(cabin_flows, state.stocks)

    _assert_flow_ids_disjoint(bio_reg, cabin_reg)
    return state, bio_reg, cabin_reg


def _assert_flow_ids_disjoint(bio_reg: Registry, cabin_reg: Registry) -> None:
    """Guard: the biosphere-slow and cabin-fast registries share no ``FlowId``.

    The driver steps both registries over one shared stock dict; a flow id present in
    both would be a silent cross-domain wiring bug (a ``Registry`` rejects duplicates
    only *within* itself). This is the flow-id analogue of the greenhouse's stock-id
    disjointness check.
    """
    bio_ids = {flow.id for flow in bio_reg.flows}
    cabin_ids = {flow.id for flow in cabin_reg.flows}
    overlap = bio_ids & cabin_ids
    if overlap:
        raise ValueError(
            "harvest flow-id collision between the biosphere and the cabin registries: "
            f"{sorted(overlap)} (the two flow sets the driver steps together must be "
            "disjoint)"
        )


def harvest_bio_resolver(
    weather: list[dict[str, float | str]],
    scenario: HarvestScenario = HARVEST_SCENARIO,
) -> SourceResolver:
    """The biosphere forcing resolver — the greenhouse's, over the embedded scenario.

    Delegates to :func:`station.greenhouse.greenhouse_bio_resolver` (the frozen
    ``weather_resolver`` + the default sealed shared map, ``CO2_POOL_VAR → CARBON_POOL``
    = the cabin air). Step 6 changes no biosphere forcing — only the ``thermal_time``
    initial condition (set in :func:`build_harvest`) and the added harvest sink.
    """
    return greenhouse_bio_resolver(weather, scenario.greenhouse)


def harvest_cabin_resolver(
    scenario: HarvestScenario = HARVEST_SCENARIO,
) -> SourceResolver:
    """The cabin forcing resolver — the greenhouse's two constant crew intake rates.

    Delegates to :func:`station.greenhouse.greenhouse_cabin_resolver`. The ``Harvest``
    flow is **donor-controlled** (reads ``storage_c``, needs no forcing var), so no new
    forcing is added.
    """
    return greenhouse_cabin_resolver(scenario.greenhouse)


def run_harvest(
    bio_integrator: EulerIntegrator,
    cabin_integrator: EulerIntegrator,
    state: State,
    bio_resolver: SourceResolver,
    cabin_resolver: SourceResolver,
    scenario: HarvestScenario = HARVEST_SCENARIO,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """The two-rate master-step driver: one day per step (biosphere-slow / cabin-fast).

    A thin wrapper over :func:`station.driver.run_master_day` with the biosphere as the
    **slow** domain (once/day, advancing ``thermal_time`` — so grain refills — and
    ``n``) and the cabin (now carrying the ``Harvest`` flow) as the **fast** domain
    (``substep`` ×``steps_per_day``, ``n`` kept — so grain drains across the day).
    ``states`` holds one entry per day boundary (the golden pins the final one);
    ``total_rationed`` sums the Euler-backstop firings (validation asserts ``== 0``);
    ``events`` are extinction events (empty — the crew has no POPULATION stock, and the
    plant is well past anthesis but short of maturity over the horizon). Euler-only (the
    biosphere is Euler-locked by its freeze), so the integrators are
    :class:`EulerIntegrator`.

    Timing comes from the embedded greenhouse scenario (``days`` / ``steps_per_day`` /
    ``cabin_dt`` / ``bio_dt``), so the driver's ``fast_dt·steps_per_day == 86400`` (one
    day) invariant holds by construction.
    """
    gh = scenario.greenhouse
    return run_master_day(
        bio_integrator,
        cabin_integrator,
        state,
        bio_resolver,
        cabin_resolver,
        days=gh.days,
        steps_per_day=gh.steps_per_day,
        slow_dt=gh.bio_dt,
        fast_dt=gh.cabin_dt,
    )
