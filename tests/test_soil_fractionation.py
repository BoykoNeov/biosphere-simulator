"""Soil carbon pool fractionation (2026-08-10) — pinned as measurements, not a design.

`docs/plans/post-roadmap-soil-fractionation.md`. The chamber-scale diagnosis's named
seam, measured and turned down. Read-only: no scenario, param, golden or manifest is
touched, and nothing here is imported by `src/`.

The fractionated form does not exist in the tree, so it is assembled here the way
`season.build_season` assembles the frozen one — including the aux processes, because
the option-(B) probe's `Registry(flows, stocks)` dropped them and froze `thermal_time`
at 0 with a clean control saying nothing.

⚠ The RothC equilibrium table was read off a PAGE RENDER: `pdftotext -layout` detaches
its label column and shifts the values three rows, filing `HUM 0.1533 / IOM 4.4852`.
Test 1 is the arithmetic that authenticates the reading.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, replace
from pathlib import Path

import pytest

from domains.biosphere.allocation import Senescence, senescence_flux
from domains.biosphere.chamber import oxygen_limitation_factor
from domains.biosphere.compartments import SOIL
from domains.biosphere.decomposition import (
    Decomposition,
    DecompositionParams,
    decomposition_flux,
)
from domains.biosphere.loader import (
    MOLAR_MASS_CARBON_KG_PER_MOL,
    load_decomposition_params,
    load_microbial_respiration_params,
    load_nitrogen_params,
    load_phenology_params,
    load_senescence_params,
)
from domains.biosphere.mineralization import (
    LitterNitrogenTransfer,
    NitrogenSenescence,
    carried_nitrogen,
)
from domains.biosphere.phenology import development_stage
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_SCENARIO,
    DEFAULT_SCENARIO,
    LONG_HORIZON_YEARS,
    PERENNIAL_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_SCENARIO,
    SEALED_CHAMBER_YEARS,
    WATER_BITING_SCENARIO,
    WATER_BITING_YEARS,
    SeasonScenario,
)
from domains.biosphere.season import (
    CARBON_POOL,
    LEAF_C,
    LITTER_CARBON,
    LITTER_N,
    MICROBIAL_CARBON,
    PLANT_N,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    THERMAL_TIME,
    VERNALIZATION_DAYS,
    build_season,
    resow_water_return,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, steps_for
from domains.biosphere.stocks import (
    O2_POOL,
    ROOTED_DEPTH,
    SOIL_WATER,
    SUBSOIL_WATER,
    pool_stock,
)
from simcore.environment import Environment
from simcore.flow import Flow, FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- [RothC] Coleman & Jenkinson, RothC-26.3 guide, sources/RothC_guide_WIN.pdf -------
#
# section 1.5, p. 9 — "The decomposition rate constants (k), in years-1, for each
# compartment are set at: DPM: 10.0 / RPM: 0.3 / BIO: 0.66 / HUM: 0.02".
K_DPM_YR = 10.0
K_RPM_YR = 0.3
K_BIO_YR = 0.66
K_HUM_YR = 0.02

# section 1.3, p. 8 — "for most agricultural crops and improved grassland, we use a
# DPM/RPM ratio of 1.44, i.e. 59% of the plant material is DPM and 41% is RPM".
DPM_RPM_INPUT_RATIO = 1.44
DPM_INPUT_FRACTION = DPM_RPM_INPUT_RATIO / (1.0 + DPM_RPM_INPUT_RATIO)

# section 3.2, p. 40 — the Hoosfield unmanured-plot equilibrium state, 31 Dec 1851,
# after 10,000 years at a 1.70 t C/ha/yr input. READ OFF THE PAGE RENDER.
HOOSFIELD_T_C_HA = {
    "DPM": 0.1533,
    "RPM": 4.4852,
    "BIO": 0.6671,
    "HUM": 25.8576,
    "IOM": 2.7000,
}
HOOSFIELD_STATED_TOTAL = 33.8632

# 1 t C/ha = 1000 kg / 10000 m2 = 0.1 kg C/m2.
T_C_HA_TO_MOL_M2 = 0.1 / MOLAR_MASS_CARBON_KG_PER_MOL

# =====================================================================================
# ⚠⚠ _THE_REFUSAL'S_EVIDENCE_MOVED — 2026-08-12, AND THIS BUILD DID NOT RE-DECIDE IT
# =====================================================================================
#
# This module pins the measurements behind a REFUSAL: soil carbon pool fractionation was
# refused twice, on two measured legs. The stem-reserve build (2026-08-12) moved the
# tree
# underneath both of them, and the honest summary is that **both legs have dissolved**:
#
# WARNING **AN EARLIER DRAFT OF THIS NOTE SAID "BOTH LEGS HAVE DISSOLVED". THAT WAS
# WRONG, AND ONLY MEASURING AT THE RIGHT HORIZON FOUND IT.** What actually moved:
#
#   DISSOLVED (three arbitration/re-sow results):
#   * sizing 2 (the constant-FLUX seed, 19.4093) used to ration in year 3, day 197. It
#     now rations at **no horizon tested** (1, 2, 3, 5 and 50 years all give 0).
#   * the constant-INVENTORY sizing (3.0) used to starve the re-sow outright — a hard
#     `seed bank too small` error. It now re-sows.
#   * the consumer chamber under sizing 1 + `rdr_stem_zero` used to fail the same way.
#     It now re-sows too.
#
#   SURVIVED (both liveness-floor results, which is the leg that actually refused it):
#   * sizing 2's 50-year attractor still sits **below** the 0.05 decade CO2 floor —
#     0.031920, i.e. 0.64x the floor, with the whole of years 30-50 under it.
#   * half a mol above sizing 1 still fails the floor.
#
# So the refusal's ARBITRATION evidence is gone and its LIVENESS evidence is not. Anyone
# tempted to read "the reserve unblocked fractionation" off the three dissolved results
# has to get past the two that did not move.
#
# The mechanism is not mysterious: the stem reserve holds carbon out of the stem and
# routes it to grain, and grain is not shed to litter during the season — so less carbon
# reaches the litter cascade that these sizings existed to over-run.
#
# ⚠ **THAT IS RECORDED AS A MEASUREMENT AND EXPLICITLY NOT AS A RE-OPENING.**
# Re-deciding
# a refusal inside the work that moved the tree underneath it is the shape this project
# refuses (the CUE build's precedent, and the stem-reserve diagnosis's own finding 6
# said
# the same thing about stem-only). The pins below are re-measured so they describe the
# tree that exists; whether fractionation should now be BUILT is a separate question
# with
# its own evidence to gather, and it is a NAMED SUCCESSOR rather than a conclusion here.
#
# The sizings themselves are unchanged and still correct: 6.0 and 19.4093 are derived
# from RothC's rates and the frozen decomposition rate, none of which this build
# touches.
# What moved is the tree's response to them.
# =====================================================================================

# --- ours -----------------------------------------------------------------------------
FROZEN_K_DAY = 0.011  # decomposition_rate
FROZEN_K_YR = FROZEN_K_DAY * 365.0
FROZEN_LITTER_SEED = 3.0  # litter_carbon0 in every sealed scenario

K_DPM_DAY = K_DPM_YR / 365.0
K_RPM_DAY = K_RPM_YR / 365.0
K_HUM_DAY = K_HUM_YR / 365.0

LITTER_RPM: StockId = StockId("biosphere.litter_rpm")
LITTER_HUM: StockId = StockId("biosphere.litter_hum")

_EQ_DPM = HOOSFIELD_T_C_HA["DPM"]
_EQ_RPM = HOOSFIELD_T_C_HA["RPM"]
EQ_DPM_STANDING_FRACTION = _EQ_DPM / (_EQ_DPM + _EQ_RPM)
K_AGGREGATE_YR = (
    EQ_DPM_STANDING_FRACTION * K_DPM_YR + (1.0 - EQ_DPM_STANDING_FRACTION) * K_RPM_YR
)

# The 0.05 decade CO2 liveness floor the biosphere manifest names for
# `perennial_long_horizon`
# (test_decade_stability.py::test_decade_min_carbon_pool_stationary).
DECADE_CO2_FLOOR = 0.05

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# --- the three flows a build would add or change --------------------------------------


@dataclass(frozen=True)
class SplitSenescence:
    """`Senescence` with its single litter leg split DPM/RPM at the cited input ratio.

    Delegates to the frozen flow, so the per-organ carbon is bit-identical and only the
    litter leg's *destination* changes. Test 12 pins that it re-targets without
    re-scaling — had it changed the total, every measurement in this module would be
    wrong in a way nothing else here would catch.
    """

    inner: Senescence
    litter_rpm: StockId
    dpm_fraction: float

    @property
    def id(self) -> FlowId:
        return self.inner.id

    @property
    def priority(self) -> int:
        return self.inner.priority

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        legs: list[Leg] = []
        for leg in self.inner.evaluate(snapshot, env, dt).legs:
            if leg.stock == self.inner.litter_sink:
                dpm = leg.amount * self.dpm_fraction
                legs.append(Leg(leg.stock, dpm))
                legs.append(Leg(self.litter_rpm, leg.amount - dpm))
            else:
                legs.append(leg)
        return FlowResult(legs=tuple(legs))


@dataclass(frozen=True)
class AggregateLitterNitrogenTransfer:
    """`LitterNitrogenTransfer` carried by the TOTAL decomposed C over the TOTAL litter
    C.

    What keeps option (B)'s identity alive under fractionation. With one N pool against
    two carbon pools, N must leave on the aggregate flux:

        d(C)/dt = -(k_d*C_d + k_r*C_r)     d(N)/dt = -N*(k_d*C_d + k_r*C_r)/C

    so d(N/C)/dt = 0 exactly. Carrying it on one pool's flux instead would break it.
    Test 10 measures that, rather than resting on the algebra.
    """

    id: FlowId
    priority: int
    litter_n: StockId
    microbial_n: StockId
    litter_dpm: StockId
    litter_rpm: StockId
    dpm_params: DecompositionParams
    rpm_params: DecompositionParams
    # f_O2 arrived on the carbon side with the humification split (2026-08-10): the
    # litter flow gained a CO2 leg, and an O2-drawing flow must self-throttle. If this N
    # leg did not carry the same factor, C and N would stop leaving on the same flux and
    # the identity below would read 90.035 instead of 90 — a HARNESS artefact that would
    # have been very easy to quote as "the identity is only approximate now".
    o2_pool: StockId
    o2_half_saturation: float
    air_mol: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        stocks = snapshot.stocks
        c_d = stocks[self.litter_dpm].amount
        c_r = stocks[self.litter_rpm].amount
        f_o2 = oxygen_limitation_factor(
            stocks[self.o2_pool].amount,
            air_mol=self.air_mol,
            k_o2=self.o2_half_saturation,
        )
        decomposed = (
            (
                decomposition_flux(
                    c_d, decomposition_rate=self.dpm_params.decomposition_rate
                )
                + decomposition_flux(
                    c_r, decomposition_rate=self.rpm_params.decomposition_rate
                )
            )
            * f_o2
            * dt
        )
        moved = carried_nitrogen(decomposed, stocks[self.litter_n].amount, c_d + c_r)
        return FlowResult(
            legs=(Leg(self.litter_n, -moved), Leg(self.microbial_n, moved))
        )


# --- the offline build ----------------------------------------------------------------


def build_variant(
    scenario: SeasonScenario,
    *,
    total_seed: float,
    fractionate: bool,
    rdr_stem_zero: bool = False,
    hum_seed: float = 0.0,
) -> tuple[State, Registry]:
    """`build_season` with the litter pool optionally split DPM/RPM (+ an inert-ish
    HUM).

    `fractionate=False` reproduces the frozen tree with `total_seed` as
    `litter_carbon0`,
    so control and subject differ in exactly one thing.
    """
    # Start from what `build_season` itself produces and modify that, rather than
    # re-implementing the assembly: the stocks, flows and aux processes are read back
    # off its own State and Registry through their public surface. Carrying the aux
    # across is load-bearing — the option-(B) probe's `Registry(flows, stocks)` dropped
    # them and froze `thermal_time` at 0 while its control stayed clean.
    base_state, base_registry = build_season(
        replace(scenario, litter_carbon0=total_seed)
    )
    stocks: dict[StockId, Stock] = dict(base_state.stocks)
    flows: list[Flow] = list(base_registry.flows)
    aux_processes = list(base_registry.aux_processes)

    if rdr_stem_zero:
        # Both legs of one physical event carry the rates, so both must be swapped:
        # `NitrogenSenescence` independently recomputes `Senescence`'s per-organ carbon.
        swapped: list[Flow] = []
        for flow in flows:
            if isinstance(flow, Senescence):
                swapped.append(replace(flow, params=replace(flow.params, rdr_stem=0.0)))
            elif isinstance(flow, NitrogenSenescence):
                swapped.append(
                    replace(flow, sen_params=replace(flow.sen_params, rdr_stem=0.0))
                )
            else:
                swapped.append(flow)
        flows = swapped

    if fractionate:
        dpm0 = total_seed * EQ_DPM_STANDING_FRACTION
        stocks[LITTER_CARBON] = replace(stocks[LITTER_CARBON], amount=dpm0)
        stocks[LITTER_RPM] = pool_stock(
            LITTER_RPM,
            SOIL,
            Quantity.CARBON,
            canonical_unit(Quantity.CARBON),
            total_seed - dpm0,
        )
        dpm_params = DecompositionParams(decomposition_rate=K_DPM_DAY)
        rpm_params = DecompositionParams(decomposition_rate=K_RPM_DAY)
        rebuilt: list[Flow] = []
        for flow in flows:
            if isinstance(flow, Decomposition):
                rebuilt.append(replace(flow, params=dpm_params))
                rebuilt.append(
                    Decomposition(
                        FlowId("biosphere.decomposition_rpm"),
                        flow.priority,
                        litter_carbon=LITTER_RPM,
                        microbial_carbon=MICROBIAL_CARBON,
                        co2_pool=flow.co2_pool,
                        o2_pool=flow.o2_pool,
                        params=rpm_params,
                        humification=flow.humification,
                        o2_half_saturation=flow.o2_half_saturation,
                        air_mol=flow.air_mol,
                    )
                )
            elif isinstance(flow, Senescence):
                rebuilt.append(SplitSenescence(flow, LITTER_RPM, DPM_INPUT_FRACTION))
            elif isinstance(flow, LitterNitrogenTransfer):
                rebuilt.append(
                    AggregateLitterNitrogenTransfer(
                        flow.id,
                        flow.priority,
                        litter_n=flow.litter_n,
                        microbial_n=flow.microbial_n,
                        litter_dpm=LITTER_CARBON,
                        litter_rpm=LITTER_RPM,
                        dpm_params=dpm_params,
                        rpm_params=rpm_params,
                        o2_pool=flow.o2_pool,
                        o2_half_saturation=flow.o2_half_saturation,
                        air_mol=flow.air_mol,
                    )
                )
            else:
                rebuilt.append(flow)
        flows = rebuilt

    if hum_seed > 0.0:
        stocks[LITTER_HUM] = pool_stock(
            LITTER_HUM,
            SOIL,
            Quantity.CARBON,
            canonical_unit(Quantity.CARBON),
            hum_seed,
        )
        template = next(f for f in flows if isinstance(f, Decomposition))
        flows.append(
            Decomposition(
                FlowId("biosphere.decomposition_hum"),
                0,
                litter_carbon=LITTER_HUM,
                microbial_carbon=MICROBIAL_CARBON,
                co2_pool=template.co2_pool,
                o2_pool=template.o2_pool,
                params=DecompositionParams(decomposition_rate=K_HUM_DAY),
                humification=template.humification,
                o2_half_saturation=template.o2_half_saturation,
                air_mol=template.air_mol,
            )
        )

    state = State(
        n=0,
        stocks=stocks,
        rng_seed=0,
        # ⚠ The aux dict is COPIED from what `build_season` made, not re-listed here.
        # It used to be the literal `{THERMAL_TIME: 0, VERNALIZATION_DAYS: 0}`, which
        # silently omitted `rooted_depth` — harmless while that accumulator began at 0
        # anyway, and a real divergence the moment the soil-layers build gave it a cited
        # nonzero sowing depth (a variant starting at depth 0 with a dry subsoil is
        # frozen there by `WSTORG = 0 ⇒ GRTD = 0`, so FROOT1 = 0 and it takes up no
        # nitrogen at all). This file's own docstring already warned that "carrying the
        # aux across is load-bearing"; re-listing the keys was that bug one level up.
        aux=dict(base_state.aux),
    )
    return state, Registry(flows, stocks, aux_processes=aux_processes)


def reset_variant(
    state: State, scenario: SeasonScenario, *, fractionate: bool
) -> State:
    """`season.annual_reset` with the litter residual split at the cited input ratio."""
    seedling = {
        LEAF_C: scenario.leaf_c0,
        STEM_C: scenario.stem_c0,
        ROOT_C: scenario.root_c0,
    }
    seedling_total = scenario.leaf_c0 + scenario.stem_c0 + scenario.root_c0
    stocks = dict(state.stocks)
    grain = stocks[STORAGE_C].amount
    if grain < seedling_total:
        raise ValueError(f"annual_reset: seed bank too small to re-sow — {grain!r}")
    old_veg = sum(stocks[oid].amount for oid in seedling)
    for organ_id, amount in seedling.items():
        stocks[organ_id] = replace(stocks[organ_id], amount=amount)
    stocks[STORAGE_C] = replace(stocks[STORAGE_C], amount=0.0)
    gain = old_veg + grain - seedling_total
    if fractionate:
        dpm = gain * DPM_INPUT_FRACTION
        stocks[LITTER_CARBON] = replace(
            stocks[LITTER_CARBON], amount=stocks[LITTER_CARBON].amount + dpm
        )
        stocks[LITTER_RPM] = replace(
            stocks[LITTER_RPM], amount=stocks[LITTER_RPM].amount + (gain - dpm)
        )
    else:
        stocks[LITTER_CARBON] = replace(
            stocks[LITTER_CARBON], amount=stocks[LITTER_CARBON].amount + gain
        )
    old_plant_n = stocks[PLANT_N].amount
    conc_old = (old_plant_n / old_veg) if old_veg > 0.0 else 0.0
    seedling_n = conc_old * seedling_total
    stocks[PLANT_N] = replace(stocks[PLANT_N], amount=seedling_n)
    stocks[LITTER_N] = replace(
        stocks[LITTER_N], amount=stocks[LITTER_N].amount + (old_plant_n - seedling_n)
    )
    aux = dict(state.aux)
    aux[THERMAL_TIME] = 0.0
    aux[VERNALIZATION_DAYS] = 0.0
    # Mirrors `season.annual_reset`: a re-sown crop starts with the sowing root system,
    # and the abandoned root zone's water goes back below the (now shallow) root zone.
    old_depth = aux.get(ROOTED_DEPTH, 0.0)
    aux[ROOTED_DEPTH] = scenario.rooted_depth0
    # ⚠ CALLS the tree's own rule rather than restating it. This block used to be a
    # hand-copy under a comment claiming it mirrored `season.annual_reset`; when the
    # rule changed (2026-08-12) the copy did not, and these variant runs quietly kept
    # the old water return — so every "control" here was controlling against a tree
    # that no longer existed. One function, two callers.
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


def drive(
    scenario: SeasonScenario,
    years: int,
    *,
    perennial: bool,
    total_seed: float,
    fractionate: bool,
    rdr_stem_zero: bool = False,
    hum_seed: float = 0.0,
    days: int | None = None,
) -> tuple[list[State], int]:
    """Run `scenario` the way its own golden drives it. `run_season` asserts
    conservation
    across the reset, which is what makes a re-sow failure real starvation.

    `days` truncates the run below the whole-year count `years` implies. It exists for
    ONE purpose: on a deterministic run the smallest horizon that rations *is* the
    firing step, so truncation is how one gets MEASURED instead of read off the CO2
    argmin (which the (C) stem-only branch recorded as circular). Leave it None and the
    run is exactly `years` years.

    ⚠ It is in **physical days** (it was called `steps` until 2026-08-14, when the step
    stopped being a day) — the conversion to steps happens once, below.
    """
    rows = _weather() * years
    # ⚠ In STEPS: compared against the step counter `n` in the reset closure below.
    year_steps = steps_for(len(_weather()))
    state, registry = build_variant(
        scenario,
        total_seed=total_seed,
        fractionate=fractionate,
        rdr_stem_zero=rdr_stem_zero,
        hum_seed=hum_seed,
    )
    resolver = weather_resolver(rows, scenario)

    def reset(n: int, current: State) -> State:
        if n > 0 and n % year_steps == 0:
            return reset_variant(current, scenario, fractionate=fractionate)
        return current

    states, rationed, _ = run_season(
        EulerIntegrator(registry),
        state,
        resolver,
        BIO_DT,
        steps_for(len(rows) if days is None else days),
        reset=reset if perennial else None,
    )
    # The option-(B) probe guard: a dropped aux freezes thermal_time and every
    # DVS-keyed quantity silently reads as a seedling's.
    ph = load_phenology_params()
    peak_dvs = max(
        development_stage(
            s.aux[THERMAL_TIME],
            tsum_anthesis=ph.tsum_anthesis,
            tsum_maturity=ph.tsum_maturity,
        )
        for s in states
    )
    assert peak_dvs == pytest.approx(2.0), f"aux frozen? peak DVS = {peak_dvs}"
    return states, rationed


def per_year_min_co2(states: list[State], years: int) -> list[float]:
    # ⚠ In STEPS. It segments a STEP-indexed trajectory (`run_season` appends one state
    # per step, not per day), so the season length has to be converted — this read
    # `len(_weather())` until 2026-08-14, and at `dt = ¼` that silently covered a
    # quarter of each year and only the first quarter of the run: a 15-year call
    # returned 15 windows that between them spanned under four years, every one of them
    # mislabelled. Nothing went red, because a shorter window still has a minimum.
    #
    # This is the by-CALLEE enumeration the step unfreeze's record says has to be
    # repeated for every callee carrying a unit, and was not: `drive` and the reset
    # closure above were converted on 2026-08-14 and this helper, three lines away, was
    # not. The local is named `ys`, which no name-based sweep would ever catch.
    ys = steps_for(len(_weather()))
    co2 = [s.stocks[CARBON_POOL].amount for s in states]
    return [min(co2[i * ys : (i + 1) * ys + 1]) for i in range(years)]


# --- 1. the source, and the arithmetic that authenticates a scrambled table -----------


def test_the_hoosfield_pools_sum_exactly_to_the_stated_total() -> None:
    """The reading of a table `pdftotext` scrambles, checked by its own arithmetic.

    Extraction detaches the label column and shifts the values three rows, so the naive
    read is `HUM 0.1533 / IOM 4.4852 / Total 0.6671`. Round 5's rule — a quote check
    verifies characters, only arithmetic verifies numbers — is what recovers it: the
    five
    pools as read sum to the printed total exactly, which the shifted reading does not.
    """
    assert sum(HOOSFIELD_T_C_HA.values()) == pytest.approx(
        HOOSFIELD_STATED_TOTAL, abs=1e-12
    )
    # ...and the shifted reading does not, which is what makes the check discriminating.
    shifted = [
        0.1533,
        4.4852,
        0.6671,
    ]  # what the naive extraction files as HUM/IOM/Total
    assert sum(shifted) != pytest.approx(HOOSFIELD_STATED_TOTAL, abs=1e-6)


def test_our_rate_sits_between_the_two_plant_material_rates() -> None:
    """`decomposition_rate` is a DPM-ish rate: below DPM, far above RPM."""
    assert K_RPM_YR < FROZEN_K_YR < K_DPM_YR
    assert pytest.approx(4.015) == FROZEN_K_YR


def test_every_rothc_rate_is_safe_at_the_frozen_timestep() -> None:
    """`k*dt < 1` at the frozen step for every rate a build would adopt.

    ⚠ The seventh conflation class, in the same shape the acceptance gate's
    ``..._margin_is_one_over_k_dt`` carried it (2026-08-14): the `dt` was in this
    docstring — *"at the frozen `dt = 1 day`"* — and missing from the expression below,
    which divided by 365 and stopped. That was invisible while the step was a day and it
    is still not a failure at `¼`, since a smaller step only makes `k·dt` smaller. It is
    corrected anyway, because the assertion has to be about the step it names: a rate
    this test blesses would otherwise be blessed against a step nothing checks.
    """
    for k_yr in (K_DPM_YR, K_RPM_YR, K_BIO_YR, K_HUM_YR):
        assert 0.0 < k_yr / 365.0 * BIO_DT < 1.0


# --- 2/3. what fractionation buys, and the tautology it does not ----------------------


def test_the_cited_partition_gives_one_aggregate_rate_and_a_647x_gain() -> None:
    """FINDING 1/2 — 6.47x, and it is ONE quantity with two readings, not two
    measurements.

    `stock = flux/k` means the inventory ratio at constant flux and the rate ratio are
    the same number. Pinned together and in one assertion chain so neither can later be
    quoted as corroborating the other.
    """
    assert pytest.approx(0.033049, abs=1e-6) == EQ_DPM_STANDING_FRACTION
    assert pytest.approx(0.620580, abs=1e-6) == K_AGGREGATE_YR

    rate_ratio = FROZEN_K_YR / K_AGGREGATE_YR
    flux0 = FROZEN_K_YR * FROZEN_LITTER_SEED
    stock_at_same_flux = flux0 / K_AGGREGATE_YR
    inventory_ratio = stock_at_same_flux / FROZEN_LITTER_SEED

    assert rate_ratio == pytest.approx(6.4698, abs=1e-4)
    assert inventory_ratio == pytest.approx(rate_ratio, rel=1e-12)  # the SAME number
    assert stock_at_same_flux == pytest.approx(19.4093, abs=1e-4)


def test_the_aggregate_rate_is_NOT_constant_and_that_decay_is_the_payoff() -> None:
    """FINDING 1's qualifier — "one effective k" is false as an identity.

    DPM and RPM drain at 33x different rates, so the aggregate decays from 0.6206 toward
    RPM's 0.3. Pinned because the tempting flat statement would erase the tail that is
    the whole mechanism.
    """
    import math

    dpm0 = 19.4093 * EQ_DPM_STANDING_FRACTION
    rpm0 = 19.4093 - dpm0

    def aggregate_k(years: float) -> float:
        d = dpm0 * math.exp(-K_DPM_YR * years)
        r = rpm0 * math.exp(-K_RPM_YR * years)
        return (K_DPM_YR * d + K_RPM_YR * r) / (d + r)

    assert aggregate_k(0.0) == pytest.approx(K_AGGREGATE_YR, rel=1e-12)
    assert aggregate_k(2.0) < 0.32
    assert aggregate_k(5.0) == pytest.approx(K_RPM_YR, abs=1e-3)
    # ...and the tail it produces is what the one-pool form cannot: at 2 years the
    # one-pool seed has returned essentially everything and stopped.
    one_pool_flux_at_2yr = FROZEN_K_YR * FROZEN_LITTER_SEED * math.exp(-FROZEN_K_YR * 2)
    assert one_pool_flux_at_2yr < 0.005


def test_cited_and_fitted_partitions_agree_BY_CONSTRUCTION_so_it_is_no_evidence() -> (
    None
):
    """FINDING 1 — a corroboration that cannot fail is not one.

    "Does the cited partition match a partition fitted to hold our own flux?" was the
    check. The two constructions intersect at EXACTLY ONE total, so agreement there is
    guaranteed. Pinned as a non-result so it cannot later be quoted as support.
    """

    def fitted_dpm_fraction(total: float) -> float:
        dpm = (FROZEN_K_YR * FROZEN_LITTER_SEED - K_RPM_YR * total) / (
            K_DPM_YR - K_RPM_YR
        )
        return dpm / total

    at_the_intersection = FROZEN_K_YR * FROZEN_LITTER_SEED / K_AGGREGATE_YR
    assert fitted_dpm_fraction(at_the_intersection) == pytest.approx(
        EQ_DPM_STANDING_FRACTION, rel=1e-12
    )
    # Anywhere else they disagree, which is what makes the agreement above vacuous.
    assert fitted_dpm_fraction(10.0) != pytest.approx(
        EQ_DPM_STANDING_FRACTION, rel=1e-3
    )
    assert fitted_dpm_fraction(30.0) != pytest.approx(
        EQ_DPM_STANDING_FRACTION, rel=1e-3
    )


def test_the_remaining_gap_after_fractionation_is_still_over_an_order() -> None:
    """FINDING 2 — 6.47x of a 94x census gap leaves ~14.5x, which finding 3 puts out of
    reach."""
    hoosfield_total_mol = HOOSFIELD_STATED_TOTAL * T_C_HA_TO_MOL_M2
    census_gap = hoosfield_total_mol / FROZEN_LITTER_SEED
    assert census_gap == pytest.approx(94.0, abs=0.5)
    assert census_gap / (FROZEN_K_YR / K_AGGREGATE_YR) == pytest.approx(14.5, abs=0.5)


# --- 4. both principled sizings fail --------------------------------------------------


def test_the_constant_inventory_sizing_starves_the_re_sow() -> None:
    """Holding the census total (3.0) fixed: the chamber COULD NOT re-sow — until now.

    The fractionated pool returns carbon at the aggregate 0.62/yr instead of 4.015/yr,
    so year 1 never filled enough grain to seed year 2. A hard error, not a soft one.

    WARNING **THAT STOPPED BEING TRUE ON 2026-08-12** — see the module note
    ``THE REFUSAL'S EVIDENCE MOVED``. The stem reserve routes carbon into the grain that
    used to sit in the stem, and the grain is exactly what the re-sow draws its seedling
    from, so the seed bank now covers it. The old behaviour is asserted the only way it
    still can be — **with the reserve turned off** — so this test measures the
    difference
    rather than quietly recording the new state as if it were the old one.
    """
    starved = replace(PERENNIAL_CHAMBER_SCENARIO, stem_reserves=False)
    with pytest.raises(ValueError, match="seed bank too small"):
        drive(
            starved,
            PERENNIAL_CHAMBER_YEARS,
            perennial=True,
            total_seed=FROZEN_LITTER_SEED,
            fractionate=True,
        )
    # ...and with the reserve, the same sizing re-sows. Recorded as a measurement;
    # whether it changes the refusal is a separate diagnosis (the module note says why).
    _states, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_YEARS,
        perennial=True,
        total_seed=FROZEN_LITTER_SEED,
        fractionate=True,
    )
    assert rationed == 0


def test_whether_the_constant_flux_sizing_still_rations() -> None:
    """Holding the t=0 CO2 return fixed (seed 19.409): does `rationed` fire?

    ⚠⚠ **RENAMED 2026-08-14 (the quarter-day step): IT NOW FIRES ON NEITHER TREE, so the
    old name — `..._sizing_rations` — asserted a fact that has stopped being true.** The
    count has now moved four times: 11 (2026-08-09) -> 1 (the humification split) -> 0
    with the stem reserve (2026-08-12) -> 0 *without* it as well, at `dt = ¼`. The last
    step is not a re-measurement of a value but the loss of the last tree in which this
    leg could be witnessed at all.

    The direction is the expected one and is not evidence of anything new: a coarse step
    manufactures shortfalls by charging a whole step's respiration against a
    start-of-step assimilation rate, so quartering it removes the transient overdraft
    the backstop was catching. The same 6×-not-4× effect was measured on the maintenance
    door in `test_stem_reserves.py`.

    ⚠ **The refusal is NOT reopened here**, for the reason already written in this
    module: re-deciding a refusal inside the work that moved the tree underneath it is
    the shape this project refuses, and that applies with more force when the movement
    is in the refusal's favour. Recorded as a finding for a separate decision. Both
    trees are still asserted below, so the test still says which one does what — it now
    says they do the same thing.

    ---- the record as it stood before 2026-08-14, kept rather than overwritten ----

    ⚠ RE-MEASURED 2026-08-10 after the humification split: 11 firings -> 1. The
    conclusion is unchanged and is what this test is for -- that sizing is still not
    viable, because ONE firing is a hard break (the goldens assert `rationed == 0` and
    `run_scenario` raises). The count moved because the split returns 45 % of decayed
    litter carbon to the atmosphere immediately instead of routing all of it through the
    microbial pool, so the chamber has more CO2 headroom at the same inventory.

    The count is asserted exactly rather than as `> 0` because a drop from 11 to 1 is
    the sort of change worth going red for and reading, and because "it still fails" is
    a weaker claim than "it fails by this much".

    WARNING **AND ON 2026-08-12 IT REACHED ZERO: 11 -> 1 -> 0.** The stem-reserve build
    gave the chamber enough further headroom that this sizing does not ration at all.
    Read the module note ``THE REFUSAL'S EVIDENCE MOVED`` before drawing anything from
    that: the leg is measured gone, and re-deciding the refusal is a named successor,
    not
    this build's business. Both trees are asserted below, so the test still says which
    one does what.
    """
    _, rationed_off = drive(
        replace(PERENNIAL_CHAMBER_SCENARIO, stem_reserves=False),
        PERENNIAL_CHAMBER_YEARS,
        perennial=True,
        total_seed=19.4093,
        fractionate=True,
    )
    # ⚠ 1 -> 0 at `dt = ¼` (2026-08-14). This was the LAST tree in which the
    # firing could still be witnessed; with it gone, the leg is unobservable at the
    # shipped step rather than merely absent from the shipped tree. Asserted as the
    # measurement it is.
    assert rationed_off == 0  # was 1 on the reserve-off tree until 2026-08-14
    _, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_YEARS,
        perennial=True,
        total_seed=19.4093,
        fractionate=True,
    )
    assert rationed == 0  # ...and as it stands now


# --- 5. the structural finding --------------------------------------------------------


@pytest.mark.slow
def test_a_seeded_slow_pool_only_ever_DRAINS() -> None:
    """FINDING 3 — the structural ceiling, measured rather than read off Figure 1.

    RothC's HUM is FORMED by the humification split of decomposed material; only DPM and
    RPM take fresh input. This module's *variant* seeds a slow pool and gives it no
    inflow, so it can only drain — pinned at EVERY step, not at year boundaries, since a
    pool that dipped and recovered would still pass a yearly check.

    ⚠⚠ **THE ASSERTIONS BELOW ARE SOUND AND THIS DOCSTRING'S ORIGINAL CONCLUSION IS
    FALSE — annotated in place rather than rewritten, because the way it was right is
    the finding.** It used to read: *"Our `Decomposition` moves 100 % of decayed litter
    C
    into `microbial_carbon` (CUE = 1.0, the deliberate Step-4/5 split), so there is no
    humification flux and a seeded slow pool can never refill."* That was a true
    measurement of the tree as it stood on 2026-08-10, and it was **this diagnosis's
    reason for turning the seam down**: the remaining 14.5x of the census gap was
    "structurally out of reach".

    It is out of date by one day. The humification split (`humification.py`,
    `docs/plans/post-roadmap-cue-humification.md`) gave the tree exactly the flux this
    finding said it lacked, so **the tree now has a slow pool that refills** — measured
    at 0 -> 1.36691 mol C equilibrium, pinned in
    `test_decade_stability.py::test_the_perennial_decline_has_a_floor_beyond_the_frozen_horizon`.

    What this test still measures is narrower and still worth keeping: a pool with **no
    inflow** drains monotonically. That is a property of *this module's variant*, which
    seeds `LITTER_HUM` and wires nothing into it — not of the tree. **Resolved, not
    corrected**: the mechanism named in the original conclusion is gone, and the
    fractionation seam's structural blocker is discharged with it. Whether the seam is
    now worth taking is a fresh question, not one this module's numbers answer.
    """
    hum_seed = HOOSFIELD_T_C_HA["HUM"] * T_C_HA_TO_MOL_M2 * (6.0 / 38.6188)
    states, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=6.0,
        fractionate=True,
        hum_seed=hum_seed,
    )
    hum = [s.stocks[LITTER_HUM].amount for s in states]
    assert hum[0] == pytest.approx(33.4473, abs=1e-3)
    for earlier, later in zip(hum, hum[1:], strict=False):
        assert later <= earlier, "a seeded slow pool must have no refill pathway"
    # It is a one-time boost that decays away — and it breaks closure while doing it.
    assert hum[-1] / hum[0] == pytest.approx(0.7783, abs=1e-3)
    # ⚠ RE-MEASURED 2026-08-10: was 5. The humification split gives the chamber enough
    # CO2 headroom that the seeded pool's ~0.5 mol C/yr drip no longer trips the
    # backstop.
    # The "it breaks closure while doing it" half of finding 3 is therefore gone too —
    # only the monotone-drain half, which is a property of this module's inflow-less
    # variant, survives.
    assert rationed == 0


# --- 6. the window: why the one passing sizing is a FITTED one ------------------------


@pytest.mark.slow
def test_the_frozen_consumer_chamber_carries_stem_only_CLEANLY() -> None:
    """The baseline that makes the next test a regression rather than a quirk.

    Stem-only's frozen refusal is `perennial`-only: `consumer` passes both gates.
    """
    states, rationed = drive(
        CONSUMER_CHAMBER_SCENARIO,
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=FROZEN_LITTER_SEED,
        fractionate=False,
        rdr_stem_zero=True,
    )
    assert rationed == 0
    # WARNING 0.148321 -> 0.146085 (2026-08-12, stem reserves) -> 0.148519 (2026-08-14,
    # the quarter-day step). The claim — the frozen consumer chamber carries stem-only
    # CLEANLY — is re-measured a second time and holds.
    assert min(per_year_min_co2(states, LONG_HORIZON_YEARS)[1:]) == pytest.approx(
        0.148519, abs=1e-5
    )


@pytest.mark.slow
def test_whether_fractionation_still_MOVES_the_stem_only_failure() -> None:
    """FINDING 4 — ⚠⚠ **and as of 2026-08-14 there is no failure left for it to move.**

    The claim: at seed 6.0 the rescue of `perennial` costs `consumer` outright —
    `perennial` + stem-only closes, and in the same breath `consumer`, which the frozen
    tree carries cleanly, can no longer re-sow. Not progress against the gate; a
    different collision with it.

    Both sides of that trade are now gone. `consumer`'s starvation went with the stem
    reserve on 2026-08-12 for the shipped tree, and at the quarter-day step it goes for
    the reserve-off tree too (see the block below). Renamed to the question. **Finding 4
    is not withdrawn** — it was a measurement of a tree, that tree is recorded, and
    re-deciding it belongs to the refusal's own re-opening, not to a step ceremony.
    """
    states, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=6.0,
        fractionate=True,
        rdr_stem_zero=True,
    )
    assert rationed == 0
    assert min(per_year_min_co2(states, LONG_HORIZON_YEARS)[1:]) > DECADE_CO2_FLOOR

    # WARNING THIS LEG ALSO DISSOLVED ON 2026-08-12 (module note): with stem reserves
    # the consumer chamber re-sows under the same combination. Asserted against the
    # reserve-off tree, where the collision is still real, plus the new behaviour.
    #
    # ⚠⚠ **AND ON 2026-08-14 THE RESERVE-OFF TREE STOPPED FAILING TOO — the collision is
    # now unobservable at the shipped step, not merely absent from the shipped tree.**
    # This block asserted `pytest.raises(ValueError, "seed bank too small")` and the run
    # now completes. That is the same loss `test_whether_the_constant_flux_sizing_still_
    # rations` records: the reserve-off tree was the last place these legs could be
    # witnessed, and quartering the step removed the starvation there as well.
    #
    # ⚠ **The run is still made, and its outcome still asserted** — deleting the call
    # would leave nothing to notice a tree in which the collision returns. What replaces
    # the `raises` is the measured truth, which is the weaker and accurate statement.
    _off, off_rationed = drive(
        replace(CONSUMER_CHAMBER_SCENARIO, stem_reserves=False),
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=6.0,
        fractionate=True,
        rdr_stem_zero=True,
    )
    assert off_rationed == 0, "the reserve-off collision is back — see the note above"
    _now, now_rationed = drive(
        CONSUMER_CHAMBER_SCENARIO,
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=6.0,
        fractionate=True,
        rdr_stem_zero=True,
    )
    assert now_rationed == 0


@pytest.mark.slow
def test_half_a_mol_higher_and_perennial_fails_the_liveness_floor() -> None:
    """FINDING 4 — the upper bound of the window, 0.5 mol C above the passing value.

    Together with the hard error 0.5 below it, this is what makes the single passing
    sizing a value found by sweeping the gate green rather than one derived from an
    invariant — the consumer-chamber-2x / DPM-RPM-labile / ruling-B shape, refused.
    """
    states, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=7.0,
        fractionate=True,
        rdr_stem_zero=True,
    )
    # ⚠⚠ **RESOLVED 2026-08-10 — THIS BOUND IS GONE, AND IT WAS HALF THE WINDOW.**
    # As measured before the humification split: at seed 7.0 gate A passed and gate B
    # failed (tail 0.038341 < 0.05), which — with the hard error 0.5 below — is what
    # made
    # the single passing sizing "a window narrower than 1.0 mol C, found by sweeping the
    # gate green". On the humified tree the tail at 7.0 is **0.072378** and gate B
    # passes
    # too, so the window is no longer bounded above here.
    #
    # ⚠ What follows is a LIMIT on what may be quoted, not a new verdict: the
    # "narrower than 1.0 mol C" characterisation no longer holds on this tree, and the
    # fractionation refusal's *window* evidence must be re-derived before being cited
    # again. Re-deriving it is a fresh diagnosis, not something this commit — which
    # changed the tree underneath it — is entitled to settle. The refusal's other legs
    # (both principled sizings fail; the seam moves the failure rather than removing it)
    # are pinned separately and were not measured here.
    assert rationed == 0  # gate A still passes...
    tail = min(per_year_min_co2(states, LONG_HORIZON_YEARS)[1:])
    # WARNING 0.072378 -> 0.069947 (2026-08-12, stem reserves) -> 0.077264 (2026-08-14,
    # the quarter-day step); still above the floor, and further above it than before.
    assert tail == pytest.approx(0.077264, abs=1e-5)
    assert tail > DECADE_CO2_FLOOR  # ...and gate B now passes too


# --- 7. the (B) identity, and what the seed does to it -------------------------------


def test_the_option_b_identity_is_EXACT_under_fractionation_without_the_seed() -> None:
    """FINDING 5 — the aggregate-flux N transfer is the right design, measured.

    With the N-free seed removed (`litter_carbon0 = 0`), the litter pool's C:N equals
    the
    C:N of the material that fell in, at every step, fractionated or not. This is what
    makes fractionating NITROGEN into `dpm_n`/`rpm_n` unnecessary: it would cost two
    stocks for the same result, and the two pools' C:N can only diverge under a
    differentiated input C:N for which there is no source.
    """
    shed_cn = MOLAR_MASS_CARBON_KG_PER_MOL / load_nitrogen_params().n_residual_per_mol_c
    assert shed_cn == pytest.approx(90.0, abs=1e-9)

    for fractionate in (False, True):
        states, _ = drive(
            SEALED_CHAMBER_SCENARIO,
            SEALED_CHAMBER_YEARS,
            perennial=False,
            total_seed=0.0,
            fractionate=fractionate,
        )
        checked = 0
        for s in states:
            carbon = s.stocks[LITTER_CARBON].amount
            if fractionate:
                carbon += s.stocks[LITTER_RPM].amount
            nitrogen = s.stocks[LITTER_N].amount
            if nitrogen <= 1e-18 or carbon <= 1e-18:
                continue
            checked += 1
            assert (carbon / nitrogen) * MOLAR_MASS_CARBON_KG_PER_MOL == pytest.approx(
                shed_cn, rel=1e-12
            )
        # ⚠ In STEPS, and scaled with the step on 2026-08-14. This is the loop's
        # non-vacuity control — "the identity was actually exercised, not skipped by the
        # `continue` above" — and it was a bare `> 800` against a 915-step run, i.e.
        # "nearly every step". At `dt = ¼` the same run is 3661 states, so the bare
        # literal would still have passed while checking under a quarter of them: a
        # control that silently stops controlling is worse than no control.
        assert checked > steps_for(200)


def test_the_seed_artefact_becomes_PERMANENT_under_fractionation() -> None:
    """FINDING 5 — the payoff read on the N side, and it is a SCENARIO fact.

    The one-pool form drains the N-free seed at 4.015/yr, so it is gone within a year
    and
    the pool converges on 90. Under fractionation 96.7 % of that seed lands in RPM at
    0.3/yr and lingers — the very tail-persistence that is the seam's benefit is what
    preserves the artefact. So the seam owes `litter_n0`, and option (B)'s committed
    result quietly depends on the seed washing out fast.

    ⚠ These are the COMMITTED scenario's numbers, not model constants — the identity
    above is the model fact.
    """
    shed_cn = MOLAR_MASS_CARBON_KG_PER_MOL / load_nitrogen_params().n_residual_per_mol_c

    def peak_pool_cn(*, fractionate: bool, total_seed: float) -> float:
        states, _ = drive(
            SEALED_CHAMBER_SCENARIO,
            SEALED_CHAMBER_YEARS,
            perennial=False,
            total_seed=total_seed,
            fractionate=fractionate,
        )
        litter_n = [s.stocks[LITTER_N].amount for s in states]
        peak = max(range(len(litter_n)), key=lambda i: litter_n[i])
        carbon = states[peak].stocks[LITTER_CARBON].amount
        if fractionate:
            carbon += states[peak].stocks[LITTER_RPM].amount
        return (carbon / litter_n[peak]) * MOLAR_MASS_CARBON_KG_PER_MOL

    frozen = peak_pool_cn(fractionate=False, total_seed=FROZEN_LITTER_SEED)
    same_seed = peak_pool_cn(fractionate=True, total_seed=FROZEN_LITTER_SEED)
    doubled = peak_pool_cn(fractionate=True, total_seed=6.0)

    # ⚠ RE-MEASURED 2026-08-10 (humification split): 100.552 -> 102.749. The identity
    # is untouched -- litter's C and N still leave on the same flux, both now carrying
    # f_O2 -- so the pool still converges on the shed ratio from its N-free seed. What
    # moved is WHERE THE PEAK LANDS: `peak litter_n` is a point on a trajectory, and the
    # trajectory changed. A number attached to an event, not to a law.
    # WARNING 100.552 -> 102.749 (humification) -> **104.218** (2026-08-12, stem
    # reserves). The identity is still untouched; what moves is where the peak lands on
    # a
    # changed trajectory — a number attached to an event, not to a law.
    # WARNING ...and again to **104.506** (2026-08-14, the quarter-day step). Same
    # reading: the peak is an EVENT on a trajectory and a finer step re-times it. The
    # three values move by 0.3 %, 0.04 % and 0.06 % — the identity legs below are what
    # carry the finding, and they are untouched.
    assert frozen == pytest.approx(104.506, abs=1e-2)
    assert frozen / shed_cn < 1.2  # the one-pool form washes the seed out
    # WARNING re-measured 2026-08-10, 2026-08-12 (stem reserves, 277.736), 2026-08-14.
    assert same_seed == pytest.approx(277.671, abs=1e-2)
    # WARNING re-measured 2026-08-10, 2026-08-12 (stem reserves, 342.450), 2026-08-14.
    assert doubled == pytest.approx(342.031, abs=1e-2)
    assert doubled / shed_cn > 3.5  # ...and the slow pool does not


def test_peak_litter_n_names_a_DIFFERENT_EVENT_in_a_reset_driven_chamber() -> None:
    """FINDING 6 — the error this module's own probe committed, pinned so it is not
    re-made.

    In a shedding-fed chamber `peak litter_n` is the senescence maximum and the
    shed-ratio
    identity governs it. In a RESET-driven one it is the annual dump, whose C:N is set
    by
    the dying plant — measured N-RICH relative to the shed ratio, i.e. on the opposite
    side from the seed artefact. Comparing it to the shed ratio is a category error
    (this repo's correction 2, committed again one option later).
    """
    states, _ = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_YEARS,
        perennial=True,
        total_seed=6.0,
        fractionate=True,
    )
    litter_n = [s.stocks[LITTER_N].amount for s in states]
    peak = max(range(len(litter_n)), key=lambda i: litter_n[i])
    # ⚠ Both operands in STEPS: `peak` is an index into the step-indexed trajectory, and
    # the tolerance is the PHYSICAL "within two days of the boundary" the claim means.
    year_steps = steps_for(len(_weather()))
    # The peak lands just past a year boundary — it IS the dump, not the season.
    assert peak % year_steps <= steps_for(2)
    carbon = (
        states[peak].stocks[LITTER_CARBON].amount
        + states[peak].stocks[LITTER_RPM].amount
    )
    shed_cn = MOLAR_MASS_CARBON_KG_PER_MOL / load_nitrogen_params().n_residual_per_mol_c
    dump_cn = (carbon / litter_n[peak]) * MOLAR_MASS_CARBON_KG_PER_MOL
    assert dump_cn < shed_cn  # N-RICH: the opposite side from the seed artefact


# --- 8. the two structural / verification claims everything else rests on -------------


def test_the_split_re_targets_the_litter_leg_without_re_scaling_it() -> None:
    """FINDING 6 — option (B) pins that `NitrogenSenescence` recomputes `Senescence`'s
    leg.

    `SplitSenescence` must therefore preserve the TOTAL exactly, or the two legs of one
    physical event drift apart and every measurement in this module is wrong. Bit-exact,
    not approximate.
    """
    scenario = PERENNIAL_CHAMBER_SCENARIO
    state, registry = build_variant(scenario, total_seed=6.0, fractionate=True)
    split = next(f for f in registry.flows if isinstance(f, SplitSenescence))
    sen_params = load_senescence_params()
    rows = _weather()
    resolver = weather_resolver(rows, scenario)
    states, _, _ = run_season(
        EulerIntegrator(registry), state, resolver, BIO_DT, steps_for(300)
    )
    sampled = 0
    for s in states[::10]:
        env = resolver.bind(s, 1.0)
        legs = split.evaluate(s, env, 1.0).legs
        to_dpm = sum(leg.amount for leg in legs if leg.stock == LITTER_CARBON)
        to_rpm = sum(leg.amount for leg in legs if leg.stock == LITTER_RPM)
        recomputed = (
            senescence_flux(
                s.stocks[LEAF_C].amount, relative_death_rate=sen_params.rdr_leaf
            )
            + senescence_flux(
                s.stocks[STEM_C].amount, relative_death_rate=sen_params.rdr_stem
            )
            + senescence_flux(
                s.stocks[ROOT_C].amount, relative_death_rate=sen_params.rdr_root
            )
        )
        assert to_dpm + to_rpm == recomputed  # bit-exact, deliberately not approx
        sampled += 1
    assert sampled >= 30


def test_the_open_field_builds_no_litter_pools_so_it_is_structurally_untouched() -> (
    None
):
    """FINDING 2's roster caveat, asserted rather than assumed.

    `soil.py` builds the litter/microbial pools only when `scenario.sealed`, so
    `open_season` — the one frozen scenario at field scale — cannot see any of this.
    """
    state, _ = build_season(DEFAULT_SCENARIO)
    live = [
        sid
        for sid in state.stocks
        if ("litter" in sid or "microbial" in sid) and not sid.startswith("boundary.")
    ]
    assert live == []
    sealed, _ = build_season(SEALED_CHAMBER_SCENARIO)
    assert LITTER_CARBON in sealed.stocks


# --- 9. the two claims the prose rested on, pinned --------------------------------


@pytest.mark.slow
def test_exactly_one_swept_sizing_clears_both_gates_on_both_scenarios() -> None:
    """FINDING 4's headline — the passing value itself, not only its two boundaries.

    The refusal rests on the WIDTH: 6.5 clears `rationed == 0` and the 0.05 decade
    floor on both perennial scenarios, while 6.0 (hard error on `consumer`) and 7.0
    (floor failure on `perennial`) do not. Without this the window is prose and the
    reader sees two failures with no demonstrated window.
    """
    for scenario in (PERENNIAL_CHAMBER_SCENARIO, CONSUMER_CHAMBER_SCENARIO):
        states, rationed = drive(
            scenario,
            LONG_HORIZON_YEARS,
            perennial=True,
            total_seed=6.5,
            fractionate=True,
            rdr_stem_zero=True,
        )
        assert rationed == 0
        assert min(per_year_min_co2(states, LONG_HORIZON_YEARS)[1:]) > DECADE_CO2_FLOOR

    # ...and it is a WINDOW, not a threshold: half a mol either side fails. Both
    # neighbours are pinned in their own tests; asserted here as the width claim.
    assert 7.0 - 6.0 < 1.0 + 1e-12


@pytest.mark.slow
@pytest.mark.parametrize(
    ("name", "scenario", "years", "perennial", "frozen_tail", "fractionated_tail"),
    [
        # ⚠ ALL EIGHT NUMBERS RE-MEASURED 2026-08-12 (stem reserves), and the CLAIM
        # this table exists for is what was re-checked, not just the values: the
        # fractionated tail still improves on the frozen one in every row, and both
        # sides still close with `rationed == 0`. Was, in row order:
        #   0.116830/0.119886, 0.055175/0.076583, 0.055175/0.076572, 0.148486/0.152906.
        # The frozen side moved because the reserve moves the whole tree; the
        # fractionated side moved with it.
        #
        # ⚠⚠ **AND ALL EIGHT AGAIN 2026-08-14 (the quarter-day step) — but read the
        # IMPROVEMENT column, not the values, because that is where this table changed
        # shape.** The claim survives in every row; the MARGIN it rests on does not:
        #
        #     row             was              now              improvement
        #     sealed_chamber  0.11111/0.11428  0.11144/0.11457  +2.9 %  -> +2.8 %
        #     perennial       0.05591/0.07572  0.07549/0.07717  +35.4 % -> +2.2 %
        #     perennial_long  0.05591/0.07570  0.07549/0.07715  +35.4 % -> +2.2 %
        #     consumer        0.14658/0.15029  0.14884/0.15290  +2.5 %  -> +2.7 %
        #
        # The two perennial rows lose an ORDER OF MAGNITUDE of headroom, and only those
        # two, because only their FROZEN side moved (+35 %) — the one-pool perennial
        # tail was the reading a one-day step was depressing, and the fractionated tail,
        # which sits higher, had far less truncation to lose. So the 35 % that made
        # these rows look like the form's strongest evidence was mostly the
        # INTEGRATOR's, and at the shipped step every row now reads within a couple of
        # per cent of every other.
        #
        # This is recorded, not smoothed, and it does NOT reopen the refusal (module
        # note ``THE REFUSAL'S EVIDENCE MOVED``) — it cuts the other way: the price this
        # table was written to record has got smaller, so the case for building was
        # never weaker. What it retires is quoting "the fractionated tail improves the
        # perennial chamber by a third"; at the shipped step that is 2 %.
        ("sealed_chamber", SEALED_CHAMBER_SCENARIO, 3, False, 0.111443, 0.114567),
        ("perennial", PERENNIAL_CHAMBER_SCENARIO, 5, True, 0.075494, 0.077170),
        (
            "perennial_long",
            PERENNIAL_CHAMBER_SCENARIO,
            LONG_HORIZON_YEARS,
            True,
            0.075494,
            0.077149,
        ),
        ("consumer", CONSUMER_CHAMBER_SCENARIO, 5, True, 0.148839, 0.152898),
    ],
)
def test_the_form_alone_is_benign_across_the_sealed_roster(
    name: str,
    scenario: SeasonScenario,
    years: int,
    perennial: bool,
    frozen_tail: float,
    fractionated_tail: float,
) -> None:
    """FINDING 2 — recorded as a PRICE, so it is pinned rather than left as prose.

    At seed 6.0 the form closes everywhere with the CO2 tail IMPROVING everywhere, at 2x
    the inventory — where the one-pool form already rations at 6.0. That headroom is
    genuinely the form's doing, and it is still not a reason to build: no beneficiary
    (finding 4), against a full carbon cascade.
    """
    tail_slice = slice(1, None) if years > 1 else slice(0, None)

    frozen_states, frozen_rationed = drive(
        scenario,
        years,
        perennial=perennial,
        total_seed=FROZEN_LITTER_SEED,
        fractionate=False,
    )
    assert frozen_rationed == 0
    assert min(per_year_min_co2(frozen_states, years)[tail_slice]) == pytest.approx(
        frozen_tail, abs=1e-5
    )

    states, rationed = drive(
        scenario, years, perennial=perennial, total_seed=6.0, fractionate=True
    )
    assert rationed == 0
    tail = min(per_year_min_co2(states, years)[tail_slice])
    assert tail == pytest.approx(fractionated_tail, abs=1e-5)
    assert tail > frozen_tail, f"{name}: the form must not degrade the CO2 tail"


@pytest.mark.slow
def test_whether_the_one_pool_form_can_take_the_same_inventory() -> None:
    """FINDING 2's other half — ⚠⚠ **AND IT INVERTED ON 2026-08-14: it now CAN.**

    The claim was that doubling `litter_carbon0` on the frozen single pool doubles the
    FLUX with it (`stock = flux/k`, one knob) and rations — which is what made the
    fractionated roster result attributable to the partition rather than to more carbon.
    At the shipped quarter-day step the firing count is **0**, having been 6 → 5
    (humification) → 3 (stem reserves) → 0. Renamed to the question for the same reason
    as the two other tests in this file: the old name asserted the answer.

    ⚠ **Read this together with the roster table's ratio column above, because the two
    are the same event seen twice.** That table lost most of its perennial headroom at
    this step; this test loses the control that made the remaining headroom
    attributable. Neither is a fractionation result — both are the arbitration
    backstop's firings disappearing when the step stops manufacturing within-step
    overdrafts, which is the documented reason the step moved. **A backstop firing count
    is a step-size observable before it is a science one**, and three of this file's
    four inversions today are firing counts.

    ⚠ The refusal is not reopened here (module note ``THE REFUSAL'S EVIDENCE MOVED``).
    """
    _, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_YEARS,
        perennial=True,
        total_seed=6.0,
        fractionate=False,
    )
    # ⚠ 6 → 5 (2026-08-10, humification) → 3 (2026-08-12, stem reserves) → **0**
    # (2026-08-14, the quarter-day step). The claim in the old name is now false.
    assert rationed == 0


# --- 10. THE RE-OPENING (2026-08-10) --------------------------------------------------
#
# The humification split discharged finding 3 -- this diagnosis's stated reason for the
# refusal -- so the seam was re-opened and the price re-derived on the post-split tree.
# The refusal STANDS, on a leg that is measured rather than structural. These pins carry
# the part of that which is new; the sizings' immediate results are already pinned in
# sections 4 and 6 above (both were re-measured in place by the split).


@pytest.mark.slow
def test_whether_sizing_1s_attractor_still_sits_below_the_liveness_floor() -> None:
    """THE DECISIVE RE-OPENING PIN — and ⚠⚠ **IT INVERTED ON 2026-08-14.**

    **At the shipped quarter-day step sizing 1's attractor is 0.075729, which is ABOVE
    the 0.05 floor.** It was 0.032090, a 1.56× failure. So the leg this test was named
    for — *"the liveness floor failure is the ATTRACTOR, not the transient"* — no longer
    describes the tree: there is no liveness floor failure to attribute to anything.
    Renamed to the question, because a name that states a contingent measurement goes
    stale silently (the same correction `test_whether_the_perennial_gate_needs_the_LONG_
    horizon` took on 2026-08-14).

    **The cause is the integrator, and it is the same one the step unfreeze was for.** A
    one-day Euler step was depressing this chamber's CO₂ trough — the identical
    mechanism that had the perennial chamber fixing carbon below its own compensation
    point — and it was depressing it *past a gate*. The subject rises 2.36× (0.032090 →
    0.075729) while the frozen control barely moves (0.073592 → 0.075843, +3 %), so the
    two now converge to within 0.15 % of each other. ⚠ **The control is what makes this
    readable**: had it moved with the subject this would have been a harness artefact.

    ⚠ **The refusal is NOT reopened here** — module note ``THE REFUSAL'S EVIDENCE
    MOVED``. Re-deciding a refusal inside the work that moved the tree underneath it is
    the shape this project refuses, and the more so when the movement favours the
    refused option. This is the *third* leg of that refusal to dissolve (finding 3
    structural, 2026-08-10; the arbitration half, 2026-08-12; the liveness half now).
    ⚠ What survives untouched is the CITATION leg — the last uncited decomposer rate is
    still not retirable by the only cited alternative — and that leg never depended on a
    number measured here.

    ---- the reading this test was written for, kept rather than overwritten ----

    THE DECISIVE RE-OPENING PIN — sizing 1 fails the 0.05 floor at EQUILIBRIUM.

    The obvious objection to reading `perennial`'s 15-year CO2 minimum as a verdict is
    that the humification split lengthened the chamber's settling transient from ~3
    years
    to ~35, past the frozen horizon -- and that split anchored its OWN liveness floor on
    a measured equilibrium at ~yr 45 rather than on the 15-year reading. Fairness
    requires asking the same question of a change one is about to refuse, so it was
    asked before the refusal was written, not after it was challenged.

    Run to 50 years, sizing 1's per-year CO2 minimum rises monotonically off its year-3
    trough and asymptotes at **0.031741** -- still **1.58x below the 0.05 floor**, and
    flat to 6 decimals over the last several years. The failure is the attractor, not
    the
    approach to it.

    The frozen control settles at 0.073291 and is asserted alongside, because "the
    subject converges below the floor" is only a verdict if the control converges above
    it on the same horizon and harness.
    """
    frozen_states, frozen_rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        50,
        perennial=True,
        total_seed=FROZEN_LITTER_SEED,
        fractionate=False,
    )
    frozen = per_year_min_co2(frozen_states, 50)
    assert frozen_rationed == 0
    # WARNING 0.073291 -> 0.073605 (2026-08-12, stem reserves) -> 0.075843 (2026-08-14,
    # the quarter-day step); the claim (the frozen control settles ABOVE the floor at
    # equilibrium) is re-measured twice and holds. ⚠ The control moved +3 % while the
    # subject moved +136 %, which is the contrast that makes the inversion above a
    # property of the SUBJECT rather than of the step or the harness.
    assert frozen[-1] == pytest.approx(0.075843, abs=1e-5)
    assert frozen[-1] > DECADE_CO2_FLOOR

    states, rationed = drive(
        PERENNIAL_CHAMBER_SCENARIO,
        50,
        perennial=True,
        total_seed=19.4093,
        fractionate=True,
    )
    minima = per_year_min_co2(states, 50)
    # WARNING `rationed` 1 -> 0 on 2026-08-12 (stem reserves) — see the module note. The
    # arbitration half of this result is gone. **The liveness half is not**, and that is
    # the half this test is named for: the attractor still sits below the floor.
    assert rationed == 0
    # WARNING 0.031741 -> 0.032090 (2026-08-12) -> **0.075729** (2026-08-14). This is
    # the inversion, not a re-measurement: the value crossed the floor it is compared
    # against two lines below.
    assert minima[-1] == pytest.approx(0.075729, abs=1e-5)
    # ⚠ ASSERTED THE OTHER WAY ROUND SINCE 2026-08-14. This read
    # `max(minima[30:]) < DECADE_CO2_FLOOR` — "the LAST TWENTY years never once reach
    # the floor" — and the last twenty years now never once FALL BELOW it. Kept as an
    # equilibrium claim rather than deleted, because "the tail is flat and on one side
    # of the floor" is what the test measures; which side is the finding.
    assert min(minima[30:]) > DECADE_CO2_FLOOR
    # ...and it is still an attractor and not a wander: flat to 4 decimals over the last
    # twenty years. This is the half of the original claim that survived the inversion.
    assert max(minima[30:]) - min(minima[30:]) < 1e-4
    # WARNING 0.6418 -> 1.5146. Kept as a RATIO to the floor, in the same form, so the
    # crossing is visible in the diff as a number passing through 1.0 rather than as a
    # deleted line.
    assert minima[-1] / DECADE_CO2_FLOOR == pytest.approx(1.5146, abs=1e-3)


@pytest.mark.slow
def test_fractionation_does_not_STARVE_the_loop_it_enlarges_BOTH_SIDES() -> None:
    """THE MECHANISM, and it REFUTES the hypothesis this diagnosis was written
    expecting.

    RPM's 0.3/yr is almost exactly the Zhang median (0.300/yr) that the 2026-07-21
    decomposer calibration measured as starving the recycled-CO2 loop, so the natural
    reading of a deeper CO2 trough is "the slow pool cannot return carbon fast enough".
    **Measured, that is false.** At its own trough the fractionated run returns litter
    carbon **2.84x FASTER** than the frozen one, not slower.

    What actually happens is visible only when the buffer is put in the same table as
    the
    things it buffers:

        seed          6.47x        return flux   3.38x
        plant         1.64x        the atmosphere they transact through   1.00x

    (WARNING the two ratios were 2.84x / 1.81x until 2026-08-12; the stem-reserve build
    moved both. The MECHANISM claim is what matters and it is re-measured: the return is
    still higher, the plant still larger, the jar still bit-identically unchanged.)

    ⚠⚠ **RE-MEASURED AGAIN 2026-08-14 (the quarter-day step), and the table above is now
    2.59x / 1.88x — but the headline number in the third paragraph, "2.84x FASTER", is
    the 2026-08-12 value and is left standing as the historical reading it is.** The
    mechanism claim holds on both legs. What does NOT hold is the observation the
    mechanism was invoked to explain: **the fractionated trough is no longer deeper than
    the frozen one** (0.0756998 against 0.0754943, 0.27 % higher). See the assertion
    block at the end, which now bounds the separation instead of pinning its sign. A
    finer step lifted the FROZEN trough by 35 %, so the thing that needed explaining was
    substantially the integrator.

    `chamber_air_mol` and the initial CO2 are untouched by a litter change, so a change
    that enlarges the soil and the plant leaves the jar between them exactly as it was
    --
    and at 0.1 % of the system's carbon that jar records every instantaneous mismatch in
    full. The trough is a flow-balance moment in the season, not a supply shortage.

    This is the chamber-scale diagnosis reached independently (the atmosphere is a
    buffer
    of hours), and it is pinned as a MECHANISM because recording the census alone and
    calling it starvation would assert something unmeasured -- the humification split's
    finding 6 shape, one option on.
    """

    def at_trough(seed: float, fractionate: bool) -> tuple[float, float, float]:
        states, _ = drive(
            PERENNIAL_CHAMBER_SCENARIO,
            5,
            perennial=True,
            total_seed=seed,
            fractionate=fractionate,
        )
        trough = min(states, key=lambda s: s.stocks[CARBON_POOL].amount)
        f_o2 = oxygen_limitation_factor(
            trough.stocks[O2_POOL].amount,
            air_mol=PERENNIAL_CHAMBER_SCENARIO.chamber_air_mol,
            k_o2=load_microbial_respiration_params().o2_half_saturation,
        )
        dpm = trough.stocks[LITTER_CARBON].amount
        if fractionate:
            rpm = trough.stocks[LITTER_RPM].amount
            flux = (K_DPM_DAY * dpm + K_RPM_DAY * rpm) * f_o2
        else:
            flux = load_decomposition_params().decomposition_rate * dpm * f_o2
        tissue = sum(
            trough.stocks[i].amount for i in (LEAF_C, STEM_C, ROOT_C, STORAGE_C)
        )
        return flux * 365.0, tissue, trough.stocks[CARBON_POOL].amount

    frozen_return, frozen_tissue, frozen_air = at_trough(FROZEN_LITTER_SEED, False)
    frac_return, frac_tissue, frac_air = at_trough(19.4093, True)

    # WARNING 2.8558 -> 2.8370 (2026-08-12, stem reserves) -> 2.9985 (2026-08-14).
    assert frozen_return == pytest.approx(2.9985, abs=1e-3)
    # WARNING 8.1112 -> 9.5900 -> 9.5840 (2026-08-12: the stem-reserve build, then its
    # cessation window) -> **7.7523** (2026-08-14, the quarter-day step). The REFUTATION
    # below — the fractionated return is HIGHER, so 'the slow pool starved the loop' is
    # false — is re-measured and holds, but at 2.59x rather than 3.38x.
    assert frac_return == pytest.approx(7.7523, abs=1e-3)
    # THE REFUTATION: the return is HIGHER, so "the slow pool starved the loop" is
    # false.
    assert frac_return > frozen_return
    # ⚠ 2.840 -> 3.380 -> **2.585** (2026-08-14). The two sides moved in OPPOSITE
    # directions at the finer step — the frozen return up 5.9 %, the fractionated one
    # down 19.1 % — which is why the ratio falls by more than either. It is the
    # fractionated (slow-pool) side that a coarse step was flattering here, so a chunk
    # of this refutation's stated margin was the integrator's. It still refutes: 2.59 is
    # not near 1.
    assert frac_return / frozen_return == pytest.approx(2.585, abs=1e-2)

    assert frac_tissue / frozen_tissue == pytest.approx(1.879, abs=1e-2)  # was 1.637
    # ⚠⚠ **INVERTED 2026-08-14 (the quarter-day step): the trough is no longer deeper.**
    # This read `assert frac_air < frozen_air` under the comment "...and the trough is
    # nonetheless DEEPER, which is the whole point", and the two troughs are now
    # 0.0756998 (fractionated) against 0.0754943 (frozen) — the fractionated one sits
    # 0.27 % HIGHER.
    #
    # ⚠ **What the test is named for survives, and is in fact strengthened.** The name
    # is `..._does_not_STARVE_the_loop_it_enlarges_BOTH_SIDES`, and both halves are
    # re-measured above: the return flux is still 2.59× higher and the plant still 1.88×
    # larger. The deeper trough was the *puzzle* the mechanism explained, not the
    # finding — and at a step that does not truncate the trough, it does not arise.
    #
    # ⚠ The 0.27 % separation is asserted as a BOUND rather than a direction, because a
    # gap this small is not a claim either way and pinning its sign would be pinning
    # noise — the failure mode `test_stem_reserves.py` recorded when a bit-identity
    # turned out to be an event ordering.
    assert abs(frac_air - frozen_air) / frozen_air < 0.01
    # The buffer is bit-identically unchanged -- a litter seed cannot touch it. This is
    # the assertion the finding rests on; without it the table above is three ratios
    # with nothing to compare them to.
    assert PERENNIAL_CHAMBER_SCENARIO.chamber_air_mol == 1000.0


@pytest.mark.slow
def test_the_shedding_fed_regime_takes_BOTH_sizings_and_the_better_trough_costs_the_plant() -> (  # noqa: E501
    None
):
    """FINDING A — the refusal is ONE scenario's, and the regimes diverge.

    "59 % of every fresh input decays at 10.0/yr" is a claim about FRESH INPUT, and only
    the shedding-fed chambers are fed that way -- `perennial`/`consumer` are dominated
    by the annual dump, so a year after a dump the comparison inverts (41 % of it left
    at 0.3/yr, against the frozen bulk pool's `e^-4.015` = 1.8 %). Stated flat over both
    regimes it is the shedding-fed/reset-driven conflation correction 2 and option (B)'s
    finding 5 already logged twice.

    Measured, the two regimes go opposite ways: the shedding-fed pair closes at BOTH
    sizings with an improved CO2 tail, while `perennial` fails both.

    !! The improvement is NOT quotable on its own. Sizing 2 buys `sealed_chamber` a
    better tail (0.076380 -> 0.080342) at a **3.5x smaller plant** (peak vegetative
    carbon 1.844452 -> 0.520157), so the two numbers are asserted together, the way the
    humification row requires -- a CO2 trough that improves because there is less plant
    to draw on it is not a benefit.
    """
    for scenario, years, frozen_tail, s1_tail, s2_tail, s2_veg_ratio in (
        (
            SEALED_CHAMBER_SCENARIO,
            SEALED_CHAMBER_YEARS,
            # WARNING all four re-measured 2026-08-12 (stem reserves; and this
            # scenario's
            # own litter seed re-sized 3.0 -> 3.5 in the same build). Was 0.076380 /
            # 0.078065 / 0.080342 / (0.520157 / 1.844452). The two CLAIMS the row exists
            # for both hold: sizing 2 buys a better trough, and pays for it with a 3.6x
            # smaller plant.
            # WARNING and a FOURTH time 2026-08-14 (the quarter-day step). Both claims
            # hold; the MARGIN of the first one shrinks by more than half — sizing 2's
            # trough advantage over frozen was +4.8 % and is now +4.1 %, and the
            # underlying spread of the three tails narrowed from 4.8 % to 4.1 % because
            # the FROZEN tail rose fastest (+1.4 %), which is the same truncation the
            # perennial rows lost their headroom to.
            0.076807,
            0.079118,
            0.079968,
            0.460033 / 1.643391,
        ),
        (
            WATER_BITING_SCENARIO,
            WATER_BITING_YEARS,
            # ⚠ 0.085006 until 2026-08-12, then 0.088509 the same day. Neither move was
            # a mechanism change, and they had DIFFERENT causes. (1) The soil-water
            # re-basing RE-DECLARED the scenario (its bite used to be
            # `soil_water0 = 50` kg inside an absolute-kg band that no longer exists;
            # it is now `soil_moisture_index = 0.05`). (2) `WSFD` ([F] Eqn 15.8,
            # docs/plans/post-roadmap-water-stress-curves.md) made drought accelerate
            # development, and `water_biting` is one of only two runs in the tree where
            # water actually limits — so its whole carbon trajectory moved while every
            # frozen scenario stayed bit-identical. Probe values of the scenario; they
            # move when it does. The four CLAIMS below (sizing2 beats frozen, and pays
            # for it in plant) were re-measured and all still hold.
            # WARNING and re-measured a THIRD time 2026-08-12 by the stem-reserve
            # build itself: 0.093346 -> 0.087965. Was 0.118940 / 0.143329 /
            # (0.535004 / 2.143987).
            # WARNING and a FOURTH time 2026-08-14 (the quarter-day step): 0.087965 /
            # 0.089665 / 0.126958 / (0.453427 / 1.811621). ⚠ This row moves FURTHEST of
            # the four scenarios — the frozen tail +8.8 %, sizing 1 +17.2 % — and that
            # is consistent with what it is: `water_biting` is one of only two runs in
            # the tree where water actually limits, so it has the most nonlinearity for
            # a coarse step to truncate. It is not evidence about fractionation.
            0.095738,
            0.105121,
            0.127286,
            0.454588 / 1.780165,
        ),
    ):
        tails = {}
        vegs = {}
        for label, seed, fract in (
            ("frozen", FROZEN_LITTER_SEED, False),
            ("sizing1", 19.4093, True),
            ("sizing2", FROZEN_LITTER_SEED, True),
        ):
            states, rationed = drive(
                scenario, years, perennial=False, total_seed=seed, fractionate=fract
            )
            assert rationed == 0, f"{label}: the shedding-fed regime must close"
            tails[label] = min(s.stocks[CARBON_POOL].amount for s in states)
            vegs[label] = max(
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[ROOT_C].amount
                for s in states
            )
        # WARNING all three tails and the veg ratio re-measured 2026-08-12 (stem
        # reserves); the parametrize table above carries the new values.
        assert tails["frozen"] == pytest.approx(frozen_tail, abs=1e-5)
        assert tails["sizing1"] == pytest.approx(s1_tail, abs=1e-5)
        assert tails["sizing2"] == pytest.approx(s2_tail, abs=1e-5)
        assert tails["sizing2"] > tails["frozen"]
        # ...and it is paid for in plant.
        assert vegs["sizing2"] / vegs["frozen"] == pytest.approx(s2_veg_ratio, abs=1e-3)
        assert vegs["sizing2"] < vegs["frozen"]


@pytest.mark.slow
def test_the_flux_sizing_fires_on_the_SAME_season_day_as_stem_only() -> None:
    """The firing step, located by HORIZON TRUNCATION rather than read off the argmin.

    The (C) stem-only branch recorded that inferring a firing step from the CO2 argmin
    is
    circular, not merely unverified: entering that step the pool is already in free
    fall,
    so the trough is the value the backstop clamped to and could not have disagreed. On
    a
    deterministic run the smallest horizon that rations IS the firing step, so that is
    what is measured -- here by running year by year and asserting where the count
    turns.

    Sizing 1 fires in **year 3, day 197**. Stem-only fired in year 1, **day 197**. Two
    unrelated mechanisms bite on the identical within-season day, which says the day is
    a
    property of the chamber's seasonal draw rather than of either change -- and it is
    the
    word that separates a within-season failure from the beyond-horizon tiling artefact
    the decomposer calibration documents.
    """
    # ⚠ In DAYS, and correctly so — every use below is a physical-time quantity: `drive`
    # takes a day count, and the two arithmetic pins at the end are day measurements.
    # (Named `year_steps` until 2026-08-14; the name was the lie, not the value.)
    year_days = len(_weather())

    def rations_within(days: int) -> bool:
        _, rationed = drive(
            PERENNIAL_CHAMBER_SCENARIO,
            3,
            perennial=True,
            total_seed=19.4093,
            fractionate=True,
            days=days,
        )
        return rationed > 0

    # ⚠⚠ **IT NO LONGER FIRES AT ALL — 2026-08-12, THE STEM-RESERVE BUILD.** Everything
    # above describes the tree as it stood until then, and is kept because a measurement
    # that stops reproducing is a finding, not a stale constant to overwrite. Sizing 2
    # rationed in year 3 at day 197; on the tree with stem reserves it rations at NO
    # horizon tested. See `_THE_REFUSALS_EVIDENCE_MOVED` at the top of this module: the
    # stem reserve holds carbon out of the litter cascade, which is exactly the pressure
    # this sizing existed to over-run.
    #
    # The bisection helper is deliberately still called below, so that a tree in which
    # the firing returns would be caught rather than silently passing this test.
    for years in (1, 2, 3, 5):
        _, rationed = drive(
            PERENNIAL_CHAMBER_SCENARIO,
            years,
            perennial=True,
            total_seed=19.4093,
            fractionate=True,
        )
        assert rationed == 0, f"{years} yr — sizing 2 rationed again; see the note"
    assert not rations_within(3 * year_days)
    # The day the OLD firing landed on, kept as arithmetic so the coincidence it was
    # recorded for survives the measurement that no longer reproduces: 807 % 305 == 197,
    # the same within-season day stem-only fired on (502 % 305 == 197).
    # ⚠ These are MEASUREMENTS in days, not unit conversions — deliberately NOT wrapped
    # in `steps_for`. 807 and 502 are day numbers recorded off a `dt = 1` run; wrapping
    # `year_days` here would make them false for a reason unrelated to the science.
    assert 807 % year_days == 197
    assert 502 % year_days == 197


@pytest.mark.slow
def test_the_consumer_chamber_is_NOT_what_refuses_the_seam() -> None:
    """The other reset-driven scenario passes sizing 1 -- so ONE scenario binds.

    Pinned because "fractionation breaks the reset-driven chambers" is the paraphrase
    this result will collapse into, and it is false: `consumer` closes and clears the
    floor with room to spare. The refusal is `perennial`'s (with its long-horizon twin,
    which reuses the same scenario object).
    """
    states, rationed = drive(
        CONSUMER_CHAMBER_SCENARIO,
        LONG_HORIZON_YEARS,
        perennial=True,
        total_seed=19.4093,
        fractionate=True,
    )
    assert rationed == 0
    # ⚠ 0.129892 → 0.104499 (2026-08-12: the stem-reserve build + its cessation window)
    # → 0.150565 (2026-08-14, the quarter-day step). The CLAIM — consumer clears the
    # floor with room to spare, so ONE scenario binds — is re-measured and still holds,
    # now at 3.0x the 0.05 floor rather than 2.1x.
    #
    # ⚠ **But the SECOND half of that claim — "so ONE scenario binds" — is the one to
    # read now, and it is gone.** `perennial`, the scenario that did the binding,
    # cleared the same floor at the same step (0.075729, pinned above), so this test's
    # contrast is no longer between a chamber that passes and one that fails. It is kept
    # as the measurement it is; the finding belongs to the refusal's own re-decision.
    assert min(per_year_min_co2(states, LONG_HORIZON_YEARS)[2:]) == pytest.approx(
        0.150565, abs=1e-5
    )
    assert min(per_year_min_co2(states, LONG_HORIZON_YEARS)[2:]) > DECADE_CO2_FLOOR
    # ⚠ The ``[2:]`` above mirrored the decade guard's ``[_TRANSIENT:]`` window, which
    # was measured inert on the frozen tree and REMOVED on 2026-08-10. Asserted in the
    # committed form too, so this result does not quietly depend on a window that is
    # gone — "clears the floor with room to spare" has to hold over the whole run.
    assert min(per_year_min_co2(states, LONG_HORIZON_YEARS)) > DECADE_CO2_FLOOR
