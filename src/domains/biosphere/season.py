"""The single-producer season — the compartment-composition layer (P3.2).

Phase-3 Step 2 split the monolithic ``build_season`` assembly into per-compartment
**builder modules** (``atmosphere`` / ``soil`` / ``plants``); Step 3 added ``water``
(the condensate pool + recycler) when it closed the water cycle. This module is now the
thin **composition**: it calls each builder,
unions the parts (the integrator stays global — one clock, one ledger, one conservation
gate; compartments are a *grouping*, never a sub-solver), adds the cross-cutting carbon
loss-sink, and hands the flat union to the ``Registry`` (which re-sorts flows by id, so
builder/union order is behaviorally inert). The restructure is **behavior-preserving** —
same stocks, flows, ids, amounts, wiring — so the open + sealed regression goldens
pass **byte-identical without regeneration** (the proof it was safe). New science (water
cycle, mortality, perturbations) lands in the *separate* later steps, never mixed here.

**Weather-agnostic (the demo.py precedent).** :func:`build_season` builds the stocks +
flow/aux registry from declared params + a :class:`SeasonScenario`; the forcing resolver
is built separately (:func:`weather_resolver`) from a daily weather table. Both route
through the single :func:`_compartments` aggregator — ``build_season`` unions
stocks/flows/aux, ``weather_resolver`` merges the resolver ``shared`` maps (the #16
live-stock seam) — so shared wiring has one source of truth. Euler at ``dt = 1 day``.

**Re-exports.** The stock-id catalog + ``STOCK_DOMAIN`` now live in ``stocks.py``, the
scenario in ``scenario.py``, and ``_carbon_context`` in ``plants.py``; this module
re-exports every symbol the tests import from ``season`` so no test import path changed.

**DOCUMENTED FINDING — the committed season is NOT a validated oracle match.** The
season ships the *machinery* (single-currency flows, the conservation gate,
``rationed == 0`` by construction, determinism, the golden) — not behavioural
validation.

⚠ **SUPERSEDED by scope (B) increment 1 + ceremony 2 (2026-07-20) — the ranked scope-A
diagnosis below is kept as the historical record but no longer describes the model.**
Increment 1 added **vernalization + photoperiod** (so "no vernalization term exists" in
cause 2 is now FALSE — see ``phenology.py``), which slowed development and, as a
*downstream* effect (``Allocation`` reads DVS), let the canopy bootstrap to **95.6 %**
interception with **no canopy science** — so cause 1's "needs a juvenile canopy-
expansion phase" did **not** hold. Ceremony 2 then found cause 3 already sound: the two
`tsum` values are literature-centred (Penning de Vries 1989), the residual
reproductive-phase gap is **cultivar variation** vs the oracle, and **no value moved**.
Full record:
``docs/plans/post-roadmap-oracle-match.md``.

The gap is **structural, not merely uncalibrated** (measured; bucket 3 / scope A, pinned
by ``tests/test_oracle_gap.py``, planned in ``docs/plans/post-roadmap-validation.md``).
An earlier reading of this docstring — "uncalibrated placeholders + no vernalization,
~2 orders of magnitude below the oracle" — was incomplete on both counts: it named the
weakest of three causes first, and magnitude is not the most diagnostic signal. Ranked:

1. **The canopy never bootstraps** (dominant). The sown seedling intercepts **1.75 %**
   of incident light; assimilation is too small for leaf growth to outpace the
   2 %/day leaf death rate, so LAI peaks on **day 32** and collapses *before* anthesis.
   The oracle reaches **97.8 %** interception. LAI₀ matches the oracle — the initial
   condition is fine; the growth dynamics are not. Needs a juvenile canopy-expansion
   phase (temperature-driven, not assimilate-limited) — new science.
2. **Phenology runs ~1.6x fast** — anthesis in mid-February (day 138 vs 217). No
   vernalization term exists (see ``phenology.py``). **Independent of (1)**: DVS runs on
   thermal time, so neither fix implies the other.
3. **Param values** — real, but third. No tuning within literature ranges fixes a canopy
   that intercepts 1.75 % of light.

Consequently the deferred Phase-1 "quantitative oracle match" is **not a calibration
task**; two of its three causes are missing science.

Pure stdlib only (the YAML/pint loading is in ``loader.py``).
"""

from collections.abc import Callable
from dataclasses import replace
from datetime import date

from domains.biosphere.atmosphere import build_atmosphere
from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.consumers import build_consumers
from domains.biosphere.loader import crop_param_set, load_canopy_params
from domains.biosphere.plants import _carbon_context as _carbon_context
from domains.biosphere.plants import build_plants
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_SCENARIO as CONSUMER_CHAMBER_SCENARIO,
)
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_YEARS as CONSUMER_CHAMBER_YEARS,
)
from domains.biosphere.scenario import (
    DEFAULT_SCENARIO,
    SeasonScenario,
)
from domains.biosphere.scenario import (
    LONG_HORIZON_YEARS as LONG_HORIZON_YEARS,
)
from domains.biosphere.scenario import (
    PERENNIAL_CHAMBER_SCENARIO as PERENNIAL_CHAMBER_SCENARIO,
)
from domains.biosphere.scenario import (
    PERENNIAL_CHAMBER_YEARS as PERENNIAL_CHAMBER_YEARS,
)
from domains.biosphere.scenario import (
    SEALED_CHAMBER_SCENARIO as SEALED_CHAMBER_SCENARIO,
)
from domains.biosphere.scenario import (
    SEALED_CHAMBER_YEARS as SEALED_CHAMBER_YEARS,
)
from domains.biosphere.soil import build_soil
from domains.biosphere.stocks import (
    CARBON_POOL as CARBON_POOL,
)
from domains.biosphere.stocks import (
    CI_VAR,
    DAYLENGTH_VAR,
    FERTILIZATION_VAR,
    IRRIGATION_VAR,
    LEAF_AREA_INDEX,
    PAR_VAR,
    RN_VAR,
    ROOTED_DEPTH,
    SUBSOIL_WATER,
    TEMP_VAR,
    THERMAL_TIME,
    VERNALIZATION_DAYS,
    VPD_VAR,
    CompartmentBuild,
    chamber_wiring,
)
from domains.biosphere.stocks import (
    CO2_ATMOS as CO2_ATMOS,
)
from domains.biosphere.stocks import (
    CO2_RESP as CO2_RESP,
)
from domains.biosphere.stocks import (
    CONDENSATE as CONDENSATE,
)
from domains.biosphere.stocks import (
    CONSUMER_CARBON as CONSUMER_CARBON,
)
from domains.biosphere.stocks import (
    HUMUS_CARBON as HUMUS_CARBON,
)
from domains.biosphere.stocks import (
    HUMUS_N as HUMUS_N,
)
from domains.biosphere.stocks import (
    LEAF_C as LEAF_C,
)
from domains.biosphere.stocks import (
    LITTER_CARBON as LITTER_CARBON,
)
from domains.biosphere.stocks import (
    LITTER_N as LITTER_N,
)
from domains.biosphere.stocks import (
    LITTER_SINK as LITTER_SINK,
)
from domains.biosphere.stocks import (
    MICROBIAL_CARBON as MICROBIAL_CARBON,
)
from domains.biosphere.stocks import (
    O2_POOL as O2_POOL,
)
from domains.biosphere.stocks import (
    PLANT_N as PLANT_N,
)
from domains.biosphere.stocks import (
    ROOT_C as ROOT_C,
)
from domains.biosphere.stocks import (
    SOIL_WATER as SOIL_WATER,
)
from domains.biosphere.stocks import (
    STEM_C as STEM_C,
)
from domains.biosphere.stocks import (
    STEM_RESERVE_C as STEM_RESERVE_C,
)
from domains.biosphere.stocks import (
    STOCK_DOMAIN as STOCK_DOMAIN,
)
from domains.biosphere.stocks import (
    STORAGE_C as STORAGE_C,
)
from domains.biosphere.stocks import (
    WATER_VAPOR as WATER_VAPOR,
)
from domains.biosphere.water import build_water
from domains.biosphere.weather import (
    daylength_seconds,
    incident_par,
    net_radiation,
    vapor_pressure_deficit,
)
from simcore import boundary, conservation
from simcore.auxiliary import AuxProcess
from simcore.environment import Schedule, SourceResolver
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State, Stock

SeasonIntegrator = EulerIntegrator | Rk4Integrator


def _compartments(scenario: SeasonScenario) -> tuple[CompartmentBuild, ...]:
    """The per-compartment builds for ``scenario`` — the single source of truth.

    Both :func:`build_season` (unions stocks/flows/aux) and :func:`weather_resolver`
    (merges the ``shared`` maps) route through here. The ``ChamberWiring`` (the
    sealed-dependent cross-compartment ids) is computed once, threaded to each builder.
    Builder/union order is inert (Registry re-sorts flows by id; the snapshot serializer
    sorts stocks by id).
    """
    wiring = chamber_wiring(scenario.sealed)
    return (
        build_atmosphere(scenario, wiring),
        build_soil(scenario, wiring),
        build_plants(scenario, wiring),
        build_water(scenario, wiring),
        build_consumers(scenario, wiring),
    )


def build_season(scenario: SeasonScenario = DEFAULT_SCENARIO) -> tuple[State, Registry]:
    """Assemble the season's initial ``State`` and the flow + aux ``Registry``.

    Composes the compartment builds: unions their stocks (keyed by id), flows, and aux,
    then adds the cross-cutting carbon **loss-sink** (extinction routing, #6) at the
    composition level (it spans compartments, so no single builder owns it). The flat
    union goes to ``Registry``, which sorts flows by id — so the assembly is
    order-independent and the goldens reproduce byte-identically.
    """
    builds = _compartments(scenario)
    stocks: dict[StockId, Stock] = {}
    for build in builds:
        for stock in build.stocks:
            stocks[stock.id] = stock
    # Only POPULATION carbon organs are extinction-eligible ⇒ only the carbon loss-sink.
    stocks.update(boundary.loss_sinks({Quantity.CARBON}))
    flows: list[Flow] = [flow for build in builds for flow in build.flows]
    aux_processes: list[AuxProcess] = [aux for build in builds for aux in build.aux]
    state = State(
        n=0,
        stocks=stocks,
        rng_seed=0,
        # ⚠ ``rooted_depth`` starts at the scenario's SOWING DEPTH, not at 0. The 0 this
        # replaced was uncited; [F] Soltani & Sinclair Ch. 14 states the quantity is an
        # input ("The value of DEPORT at crop emergence must be provided to the model.
        # It is normally between 150 to 400 mm"). See SeasonScenario.rooted_depth0.
        aux={
            THERMAL_TIME: 0.0,
            VERNALIZATION_DAYS: 0.0,
            ROOTED_DEPTH: scenario.rooted_depth0,
            # ⚠ Leaf area starts at exactly the value the DERIVED form would have given
            # for the seedling, so wiring the mechanism does not move step 0 — the whole
            # difference between the two is how it EVOLVES, and seeding it any other way
            # would smuggle a second change into the golden diff. There is deliberately
            # no ``leaf_area0`` scenario field: a seedling's area is its leaf carbon
            # times the crop's specific leaf area, and a settable initial LAI would let
            # a scenario contradict its own ``leaf_c0``.
            #
            # Always present, even when the mechanism is off (potato), so the aux dict's
            # KEY SET does not depend on the switch — a state whose shape varies with a
            # scenario flag is the kind of thing a cross-port comparison trips over.
            # Nothing reads it when ``plant_density is None``.
            LEAF_AREA_INDEX: leaf_area_index(
                scenario.leaf_c0,
                sla_per_mol_c=load_canopy_params(
                    crop_param_set(scenario.crop).paths["canopy"]
                ).sla_per_mol_c,
                ground_area=scenario.ground_area,
            ),
        },
    )
    return state, Registry(flows, stocks, aux_processes=aux_processes)


def _table(values: list[float]) -> Schedule:
    """A forcing ``Schedule`` reading a precomputed per-day table (clamped at the end).

    ``schedule(n, dt) = values[min(n, last)]`` — the first genuinely ``n``-dependent
    forcing (P3). Clamping past the last day keeps a longer-than-table run well-defined.
    """
    last = len(values) - 1

    def schedule(n: int, dt: float) -> float:
        return values[min(n, last)]

    return schedule


def weather_resolver(
    weather: list[dict[str, float | str]], scenario: SeasonScenario = DEFAULT_SCENARIO
) -> SourceResolver:
    """Build the forcing resolver from a daily raw-weather table (NASAPower facts).

    Each row is ``{day, TEMP, IRRAD, VAP}``; the clean-room conversions in
    ``domains.biosphere.weather`` derive the per-day drivers (PAR, net radiation, VPD,
    photoperiod) the flows read. ``Ci``/irrigation/fertilization are constant schedules.
    The resolver ``shared`` map (the #16 live-stock seam — ``soil_water`` always, plus
    the sealed chamber's ``co2_pool``) is **merged from the compartment builds** (one
    source of truth with ``build_season``); #16 makes shared/forcing indistinguishable,
    so this is golden-safe.
    """
    temp: list[float] = []
    par: list[float] = []
    daylen: list[float] = []
    rn: list[float] = []
    vpd: list[float] = []
    for row in weather:
        t = float(row["TEMP"])
        irrad = float(row["IRRAD"])
        vap = float(row["VAP"])
        doy = date.fromisoformat(str(row["day"])).timetuple().tm_yday
        dl = daylength_seconds(scenario.latitude, doy)
        temp.append(t)
        daylen.append(dl)
        par.append(incident_par(irrad, dl))
        rn.append(net_radiation(irrad))
        vpd.append(vapor_pressure_deficit(t, vap))
    shared: dict[str, StockId] = {}
    for build in _compartments(scenario):
        shared.update(build.shared)
    return SourceResolver(
        forcings={
            TEMP_VAR: _table(temp),
            PAR_VAR: _table(par),
            DAYLENGTH_VAR: _table(daylen),
            RN_VAR: _table(rn),
            VPD_VAR: _table(vpd),
            CI_VAR: _table([scenario.ci]),
            IRRIGATION_VAR: _table([scenario.irrigation_mm_day]),
            FERTILIZATION_VAR: _table([scenario.fertilization_kg_m2_day]),
        },
        shared=shared,
    )


def run_season(
    integrator: SeasonIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
    *,
    reset: Callable[[int, State], State] | None = None,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    ``states`` is the full trajectory incl. the initial state (length ``steps + 1``):
    used by liveness, the oracle comparison, and the golden. ``total_rationed``
    sums the Euler backstop firings (the golden asserts ``== 0``); ``events`` are the
    extinction events (empty on the well-fed season).

    **Scheduled scenario reset (P3.4, Step 4).** ``reset`` is an optional
    **schedule-agnostic** hook ``(n, state) -> state`` consulted **before each step**:
    it returns ``state`` *unchanged* (the same object) on a non-reset step, or a new
    ``State`` on a reset boundary. The calendar (e.g. ``n % year == 0``) lives **inside
    the caller's closure** — scheduling is a scenario/caller concern, not the driver's.
    When a reset is applied the driver re-asserts the conservation gate across it
    (:func:`conservation.assert_conserved`), so "conserved at every point" stays
    literally true even though the reset is a discrete scenario intervention rather than
    a flow-step (its only non-conserved write is ``thermal_time``, an aux accumulator
    invisible to the gate). The stored trajectory records the **pre-reset** state (the
    reset instant is not appended): the day a reset fires, ``states`` jumps from the
    pre-reset state straight to the post-step state. **Default ``None`` ⇒ the loop is
    byte-identical to the pre-Step-4 season** (the open/sealed regression goldens are
    unaffected) — pure indirection.
    """
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _ in range(steps):
        if reset is not None:
            reset_state = reset(state.n, state)
            if reset_state is not state:
                # The redistribution must balance every asserted quantity (carbon is
                # only moved between in-system stocks; the aux thermal_time zero is
                # outside the gate). A violation is a reset bug, not recoverable state.
                conservation.assert_conserved(state, reset_state)
                state = reset_state
        report = integrator.step_report(state, resolver, dt)
        state = report.state
        states.append(state)
        total_rationed += report.rationed
        events.extend(report.events)
    return states, total_rationed, tuple(events)


def resow_water_return(
    soil_water: float, old_depth: float, rooted_depth0: float
) -> float:
    """The water a shrinking root zone leaves behind at a re-sow, in kg.

    **A MODULE-LEVEL FUNCTION SO IT CANNOT BE RE-IMPLEMENTED.** It was inline in
    :func:`annual_reset`, and a test helper (``test_soil_fractionation.reset_variant``)
    hand-copied it under a comment reading "Mirrors ``season.annual_reset``". When the
    rule changed on 2026-08-12 the copy did not, so the variant runs silently kept the
    old water rule and diverged from the tree they were meant to be a control for —
    caught only because a pinned CO2 trough moved. That file's own docstring already
    warned about a helper diverging from the assembly it mirrors; the durable fix is one
    function with two callers, not a sharper comment.

    ``returned = soil_water · (old_depth − rooted_depth0) / old_depth`` — the abandoned
    FRACTION of the water, from the declared-uniform distribution through the zone. It
    preserves ``FTSW`` exactly across the re-sow, needs no clamp (the fraction is < 1),
    and at the drained upper limit equals ``captured_water(old_depth − rooted_depth0)``,
    the cited-geometry form it generalises. Returns 0.0 for a zone that did not shrink.
    """
    if old_depth <= 0.0:
        return 0.0
    fraction = (old_depth - rooted_depth0) / old_depth
    return soil_water * fraction if fraction > 0.0 else 0.0


def annual_reset(state: State, scenario: SeasonScenario) -> State:
    """The annual phenology reset / re-sow (P3.4) — a pure, carbon-conserving transform.

    At each year boundary the **old plant dies/harvests entirely to litter, except the
    seedling carbon retained from the grain (the seed bank)**, and ``thermal_time``
    resets so the new seedling develops from DVS 0:

    * new ``leaf_c``/``stem_c``/``root_c`` := the scenario sowing amounts;
    * ``storage_c`` (grain) := 0 (all of it is either re-sown or shed to litter);
    * ``stem_reserve_c`` := 0 — the stem's shielded starch dies with the stem, and it is
      **not** part of the seedling (the reserve is formed out of stem *growth*, so a
      newly sown crop has had none). Whatever the season did not remobilize into grain
      is real residue, and it goes to litter with everything else;
    * ``litter_carbon`` += the **balancing residual**
      ``old_veg + grain − seedling_total`` (the senescence/maintenance idiom — balance
      by construction, not an independent formula), so CARBON is conserved exactly and
      the loss-sink is never touched;
    * ``thermal_time`` := 0 (an aux accumulator, invisible to the conservation gate).

    **Grain → 0 every year is what keeps the cycle sustained** (a seed bank that only
    shed ``seedling_total`` would grow unboundedly and drain the active cycle — the
    damped-cascade trap); the dumped grain decomposes (litter → microbial → CO₂) to
    refuel next year's photosynthesis.

    **NITROGEN resets too, at the parent's own concentration (post-roadmap: the N-cycle
    form gap).** This used to be **carbon-only**, and the docstring named the
    consequence: ``plant_n`` persisted across the death as "an N *windfall* for the
    small seedling, harmless only while ``f_N ≡ 1``". Once N shedding is coupled to
    carbon, leaving that would be incoherent — the point of the coupled form is that
    nitrogen goes where the carbon it was in goes, so a plant that dies to litter cannot
    keep its N. So:

    * ``plant_n`` := ``conc_old · seedling_total``, where ``conc_old`` is the dying
      plant's whole-plant N concentration (kg N per mol C). The seed carries its
      parent's tissue concentration — which needs **no new parameter and no target curve
      in this module**, and is the right physical story (the seedling's N comes from the
      grain, which was filled by remobilization out of that same plant).
    * ``litter_n`` += the **balancing residual** ``old_plant_n − seedling_n`` — exactly
      the carbon idiom above, balance by construction rather than an independent
      formula, so NITROGEN is conserved exactly and the loss-sink is never touched.

    ``conc_old`` is taken over ``leaf + stem + root`` — **``f_N``'s own denominator** —
    and ``seedling_total`` is that same trio, so the seedling starts at exactly the
    concentration ``f_N`` was reading and the limiter is **continuous across the
    re-sow**. ⚠ That is the whole justification, and a tempting stronger claim is false:
    this is
    **not** the parent's *whole-plant* concentration, because ``plant_n`` includes the
    grain's nitrogen while this denominator excludes ``storage_c`` — the largest organ
    at reset. So ``conc_old`` is inflated relative to whole-plant and the seedling is
    correspondingly N-rich. Conservation is unaffected (the litter leg is a balancing
    residual either way). The direction is physically sensible — real seeds *are* N-rich
    relative to straw — but that is a **consequence, not the design**, and it is the
    one-pool limitation showing through: with a single whole-plant N pool there is no
    separate grain N to draw the seed from. Because ``seedling_total < old_veg``
    whenever a season grew at all, the residual cannot go negative, so the N side needs
    no seed-bank-style guard (carbon needs one only because its seedling is drawn from
    *grain*, a different pool than the one it measures).

    **Sealed-chamber only:** it sheds to the
    in-system ``litter_carbon`` POOL and re-sows from the in-system grain (the open
    field has no closed loop to re-sow into).

    Raises ``ValueError`` if the seed bank cannot cover a seedling
    (``grain < seedling_total``) — re-sow would conjure carbon or drive ``storage_c``
    negative (the closure caveat: the seedling's carbon comes from an in-system pool).
    """
    seedling = {
        LEAF_C: scenario.leaf_c0,
        STEM_C: scenario.stem_c0,
        ROOT_C: scenario.root_c0,
    }
    seedling_total = scenario.leaf_c0 + scenario.stem_c0 + scenario.root_c0
    stocks = dict(state.stocks)
    grain = stocks[STORAGE_C].amount
    if grain < seedling_total:
        raise ValueError(
            f"annual_reset: seed bank too small to re-sow — storage_c {grain!r} < "
            f"seedling {seedling_total!r}; the seedling's carbon must come from the "
            "in-system grain (closure caveat P3.4)"
        )
    old_veg = sum(stocks[oid].amount for oid in seedling)
    # The stem's shielded starch dies with the stem that held it (post-roadmap stem
    # reserves). It is NOT part of the seedling: the reserve is formed out of stem
    # growth, so a newly sown crop has had none. Absent for a crop without the
    # mechanism — the stock is not built at all, so ``.get`` rather than ``[]``.
    reserve_stock = stocks.get(STEM_RESERVE_C)
    held_reserve = reserve_stock.amount if reserve_stock is not None else 0.0
    if reserve_stock is not None:
        stocks[STEM_RESERVE_C] = replace(reserve_stock, amount=0.0)
    for organ_id, amount in seedling.items():
        stocks[organ_id] = replace(stocks[organ_id], amount=amount)
    stocks[STORAGE_C] = replace(stocks[STORAGE_C], amount=0.0)
    # The balancing residual — carbon in, carbon out, computed rather than formulated
    # (the senescence/maintenance idiom), so the reserve's inclusion cannot leak.
    litter_gain = old_veg + grain + held_reserve - seedling_total
    stocks[LITTER_CARBON] = replace(
        stocks[LITTER_CARBON], amount=stocks[LITTER_CARBON].amount + litter_gain
    )
    # The NITROGEN half: the seed keeps the parent's tissue concentration, the rest dies
    # to litter. `old_veg` is f_N's denominator (leaf+stem+root), which is the pool
    # `plant_n` is the nitrogen OF, so conc_old·seedling_total makes f_N continuous
    # across the re-sow.
    old_plant_n = stocks[PLANT_N].amount
    conc_old = (old_plant_n / old_veg) if old_veg > 0.0 else 0.0
    seedling_n = conc_old * seedling_total
    stocks[PLANT_N] = replace(stocks[PLANT_N], amount=seedling_n)
    stocks[LITTER_N] = replace(
        stocks[LITTER_N], amount=stocks[LITTER_N].amount + (old_plant_n - seedling_n)
    )
    aux = dict(state.aux)
    aux[THERMAL_TIME] = 0.0
    # A re-sown crop must re-vernalize: the cold requirement is per-cycle, so the
    # second accumulator resets alongside the first (both are outside the gate).
    aux[VERNALIZATION_DAYS] = 0.0
    # A re-sown crop also starts with NO ROOT SYSTEM: rooted depth is a property of the
    # standing crop, not of the soil, so it resets with the other per-cycle
    # accumulators.
    # ⚠ Measured bit-identical WITH this reset on every frozen scenario (the chambers
    # re-sow many times over 3-15 years), so the reset is a modelling choice the
    # goldens
    # cannot check — the pin is in tests/test_root_depth.py.
    #
    # It resets to the scenario's SOWING depth, not to 0: a re-sown crop starts with the
    # root system a sown crop has (see SeasonScenario.rooted_depth0, cited).
    old_depth = aux.get(ROOTED_DEPTH, 0.0)
    aux[ROOTED_DEPTH] = scenario.rooted_depth0
    # ⚠⚠ AND SO DOES LEAF AREA — and unlike the three resets above, THIS ONE IS NOT A
    # MODELLING CHOICE THE GOLDENS CANNOT CHECK. Omitting it handed each new seedling
    # the DEAD crop's canopy while its leaf carbon reset to a seed, so a chamber
    # assimilated at full canopy from day 0 of every cycle, emptied its finite CO₂ pool
    # and **rationed 85 times** on `consumer_long_horizon`. That is how the defect was
    # found: not as a wrong number, but as the rationing gate going off — which is
    # exactly what `post-roadmap-rationing-gate.md` made it loud for.
    #
    # ⚠ **THE QUESTION ONLY EXISTS BECAUSE THIS BUILD MADE LEAF AREA A STATE.** While
    # LAI was DERIVED from leaf carbon it re-sowed itself for free — resetting the organ
    # pools reset the canopy, with nothing to remember or forget. A state variable has
    # to be told how each cycle ends, and every accumulator added from here owes the
    # same answer. That is the standing price of reversing the "LAI is derived, not
    # stored" lock, and it is written here rather than left to be re-discovered.
    #
    # It resets to the SEEDLING'S OWN derived area — the identical expression
    # ``build_season`` seeds — so a re-sown crop starts precisely where a sown one does.
    aux[LEAF_AREA_INDEX] = leaf_area_index(
        scenario.leaf_c0,
        sla_per_mol_c=load_canopy_params(
            crop_param_set(scenario.crop).paths["canopy"]
        ).sla_per_mol_c,
        ground_area=scenario.ground_area,
    )
    # --- THE WATER HALF OF THE RE-SOW (post-roadmap soil layers) ---------------------
    # The root zone just shrank from ``old_depth`` back to the sowing depth. Water in
    # soil does not move when a plant dies, so the abandoned column's extractable water
    # is once again BELOW the root zone: it returns to ``subsoil_water``.
    #
    # ⚠ THIS RULE IS OURS. [F] Soltani & Sinclair is single-season and says nothing
    # about it. It is derived from conservation-plus-geometry, and it exists because
    # ``RootZoneCapture`` is one-way within a season (we do not model the drainage that
    # is WSTORG's only input in [F]): without a return leg, every re-sow would ratchet
    # more of the profile permanently into the root zone, and a 15-year chamber would
    # end with the whole soil column pumped up into it — a monotone drift with no
    # physical referent.
    #
    # **The rule is the abandoned FRACTION of the water, not the abandoned column at the
    # drained upper limit.** Water is (declared) uniformly distributed through the root
    # zone, so shrinking the zone from ``old_depth`` to ``rooted_depth0`` leaves behind
    # exactly the depth fraction that is no longer rooted:
    #
    #     returned = soil_water · (old_depth − rooted_depth0) / old_depth
    #
    # which preserves ``FTSW`` **exactly** across the re-sow (both ATSW and TTSW get
    # scaled by ``rooted_depth0/old_depth``) — the right invariant for a uniformly wet
    # profile, and the reason no clamp is needed: the fraction is < 1 by construction,
    # so this can never overdraw.
    #
    # ⚠ **THE PREVIOUS FORM WAS ``min(captured_water(abandoned), soil_water)`` AND IT
    # SURVIVED ONLY BECAUSE THE STORE COULD NOT RUN OUT.** It returned the abandoned
    # column *at the drained upper limit* — 149.58 kg for a 1.3 m zone — which was a
    # rounding error against a 1150 kg store and is more than the whole store once the
    # store is 19.5–169 kg. Its ``min`` then fired every re-sow and handed the *entire*
    # root zone to the subsoil, leaving the new seedling in a bone-dry bed. Measured
    # consequence: the 4-year sealed station made **no grain at all** and
    # ``annual_reset`` raised "seed bank too small to re-sow — storage_c 0.0". The old
    # comment even anticipated the shortfall ("the root zone may hold less than its
    # geometry allows") and clamped instead of re-deriving — a clamp that turns a wrong
    # amount into a survivable one hides the wrongness until the store shrinks.
    #
    # At the drained upper limit the two forms AGREE exactly (``soil_water = old_depth ·
    # EXTR · ρ · A`` makes the fraction equal ``captured_water(abandoned)``), so this is
    # a generalisation of the cited-geometry case, not a departure from it — and a
    # season ending full is still an exactly closed cycle with the capture that
    # filled it.
    returned = resow_water_return(
        stocks[SOIL_WATER].amount, old_depth, scenario.rooted_depth0
    )
    if returned > 0.0:
        stocks[SOIL_WATER] = replace(
            stocks[SOIL_WATER], amount=stocks[SOIL_WATER].amount - returned
        )
        stocks[SUBSOIL_WATER] = replace(
            stocks[SUBSOIL_WATER], amount=stocks[SUBSOIL_WATER].amount + returned
        )
    return replace(state, stocks=stocks, aux=aux)


def run_perennial(
    integrator: SeasonIntegrator,
    state: State,
    scenario: SeasonScenario,
    resolver: SourceResolver,
    dt: float,
    steps: int,
    *,
    year: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """:func:`run_season` with :func:`annual_reset` applied every ``year`` steps (P3.4).

    The concrete perennial driver: builds the schedule closure (reset at each ``n`` with
    ``n > 0 and n % year == 0``) and hands it to :func:`run_season`'s ``reset`` hook
    (which re-asserts conservation across each reset). ``year`` is the season length in
    steps (``len(weather)``, the tiling period). Sustained multi-year oscillation, no
    control code — the carbon recycles through the closed loop and each reset re-sows
    the seedling from the grain.
    """

    def reset(n: int, current: State) -> State:
        if n > 0 and n % year == 0:
            return annual_reset(current, scenario)
        return current

    return run_season(integrator, state, resolver, dt, steps, reset=reset)
