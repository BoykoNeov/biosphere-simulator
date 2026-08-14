"""The chamber-scale diagnosis (2026-08-09) — pinned as measurements, not as a design.

`docs/plans/post-roadmap-chamber-scale.md`. Three independent refusals — scope (A)
finding 11, canopy-regulator finding 4, and (C) finding 8 (stem-only) — all bottom out
on one fact: the sealed chamber's carbon inventory is fixed and tiny. These tests pin
what it is, why enlarging the chamber cannot fix it, and why the soil pile is small.

Read-only: no scenario, param, golden or manifest is touched. Every "external" number
is first-hand from a public-domain or on-shelf primary and is cited at its assertion.

⚠ Two of the external numbers were read off PAGE IMAGES, not extracted text, and that
is deliberate: `pdftotext -layout` scrambles BVAD Table 4-91's columns and files
**Rice's** row (30.23/39.0/42) under the name **Wheat**. See the plan doc.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from config import load_yaml
from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import MOLAR_MASS_CARBON_KG_PER_MOL, load_canopy_params
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_SCENARIO,
    CONSUMER_CHAMBER_YEARS,
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
    LEAF_C,
    ROOT_C,
    STEM_C,
    STEM_RESERVE_C,
    STORAGE_C,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, steps_for
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

# --- the external references (all first-hand) ----------------------------------------

# [BVAD] NASA/TP-2015-218570 Rev 2, February 2022 (US-Government work, public domain;
# NTRS 20210024855). The document `docs/bvad-reference.md` already cites for Table 3-31.
#
# Table 4-88, p. 170, "Plant Growth Chamber Equivalent System Mass per Growing Area"
# (from Drysdale, 1999b) — preliminary values for an optimized biomass production
# chamber, current NASA growth chambers projected to flight configurations.
# READ OFF THE PAGE IMAGE.
BVAD_CHAMBER_M3_PER_M2_TOTAL = 1.03
BVAD_CHAMBER_M3_PER_M2_SHOOT_ZONE = 0.67

# Table 4-91, p. 174, "Nominal and Highest Biomass Production, Composition, and
# Metabolic Products", WHEAT row. READ OFF THE PAGE IMAGE (see the module docstring).
BVAD_WHEAT_CO2_UPTAKE_G_PER_M2_D = 77.00

# Table 3-31, p. 58 — via docs/bvad-reference.md's own molar conversion:
# CO2 load 1.085 kg/CM-d / 44.009 g/mol = 24.654 mol C per crewmember-day.
BVAD_CREW_C_MOL_PER_DAY = 24.654

# [RothC] Coleman & Jenkinson, RothC-26.3 users' guide (sources/RothC_guide_WIN.pdf —
# the decomposer calibration's own source). Sec 1.5, decomposition rate constants
# (1/yr); IOM is inert. Hoosfield worked example: 33.8 t C/ha at equilibrium sustained
# by a plant input of 1.70 t C/ha/yr.
ROTHC_K_PER_YEAR = {"DPM": 10.0, "RPM": 0.3, "BIO": 0.66, "HUM": 0.02}
ROTHC_HOOSFIELD_STOCK_T_C_PER_HA = 33.8
ROTHC_HOOSFIELD_INPUT_T_C_PER_HA_YR = 1.70

M_CO2_G_PER_MOL = 44.009
R_GAS = 8.314462618
P_STD_PA = 101325.0

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
# ⚠ ``STEM_RESERVE_C`` joined this tuple 2026-08-12 (the stem-reserve build). It is the
# plant's own carbon, so leaving it out would make "the plant holds most of the
# chamber's
# carbon" understate the plant by exactly the reserve. This is the FIFTH "sum the pools"
# tuple in this repo to need a new pool added; see test_senescence_form's note.
_ORGANS = (LEAF_C, STEM_C, ROOT_C, STORAGE_C, STEM_RESERVE_C)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _carbon_total(state: State) -> float:
    return sum(
        st.amount for st in state.stocks.values() if st.quantity is Quantity.CARBON
    )


def _run(scenario: SeasonScenario, years: int, driver: str) -> list[State]:
    """Run a scenario THE WAY ITS OWN GOLDEN DRIVES IT.

    ``run_season`` vs ``run_perennial`` is not interchangeable — the annual reset is
    what makes a perennial chamber perennial. Driving all four through ``run_season``
    is the exact error correction 2 of the N-cycle work had to retract.
    """
    weather = _weather() * years
    state, registry = build_season(scenario)
    resolver = weather_resolver(weather, scenario)
    integ = EulerIntegrator(registry)
    if driver == "perennial":
        states, rationed, _ = run_perennial(
            integ,
            state,
            scenario,
            resolver,
            BIO_DT,
            steps_for(len(weather)),
            year=steps_for(len(_weather())),
        )
    else:
        states, rationed, _ = run_season(
            integ, state, resolver, BIO_DT, steps_for(len(weather))
        )
    assert rationed == 0, "the census is only meaningful on a non-rationing run"
    return states


_CHAMBERS = {
    "sealed_chamber": (SEALED_CHAMBER_SCENARIO, SEALED_CHAMBER_YEARS, "season"),
    "perennial": (PERENNIAL_CHAMBER_SCENARIO, PERENNIAL_CHAMBER_YEARS, "perennial"),
    "consumer": (CONSUMER_CHAMBER_SCENARIO, CONSUMER_CHAMBER_YEARS, "perennial"),
}

# Measured 2026-08-09 (probe1_inventory.py): the whole sealed carbon inventory.
# ⚠ ``sealed_chamber`` 3.517 -> 4.017 on 2026-08-12, and NOT because of the reserve
# (which starts empty, so it cannot move a t=0 inventory): the stem-reserve build
# re-sized this scenario's ``litter_carbon0`` 3.0 -> 3.5 to restore its O2-depletion
# contract, and +0.5 is exactly that. ``perennial``/``consumer`` were not re-sized and
# are unchanged to the digit, which is the check that the attribution is right.
_EXPECTED_LITTER_SEED = {
    "sealed_chamber": 3.5,  # re-sized 2026-08-12, see below
    "perennial": 3.0,
    "consumer": 3.0,
}

_EXPECTED_INVENTORY = {
    "sealed_chamber": 4.017,
    "perennial": 3.517,
    "consumer": 3.884,
}


@pytest.mark.slow
@pytest.mark.parametrize("name", sorted(_CHAMBERS))
def test_the_whole_sealed_inventory_is_a_few_mol_of_carbon(name: str) -> None:
    """PIN 1+2: the census, and the conservation that makes it a census at all.

    A sealed chamber's carbon is conserved exactly, so the t=0 partition IS the
    inventory for the whole run — there is never any more carbon than this.
    """
    scenario, years, driver = _CHAMBERS[name]
    states = _run(scenario, years, driver)

    inventory = _carbon_total(states[0])
    assert inventory == pytest.approx(_EXPECTED_INVENTORY[name], abs=1e-9)

    # The premise: conserved, so the initial partition is the standing inventory.
    assert _carbon_total(states[-1]) == pytest.approx(inventory, rel=1e-12)

    # And it is dominated by the seeded litter pile, not by the atmosphere.
    # ⚠ This used to be a single ``== 3.0`` for all three chambers. On 2026-08-12 the
    # stem-reserve build re-sized ``sealed_chamber``'s soil organic seed 3.0 -> 3.5 (the
    # reserve made the crop release enough extra O2 to abolish that scenario's
    # O2-depletion contract; the sweep is recorded in scenario.py) and left the other
    # two
    # alone — so the shared literal became a per-scenario fact and is now looked up.
    assert scenario.litter_carbon0 == _EXPECTED_LITTER_SEED[name]
    assert scenario.litter_carbon0 / inventory > 0.75


@pytest.mark.slow
def test_the_census_covers_every_sealed_chamber_at_every_frozen_horizon() -> None:
    """PIN 11: the ROSTER, checked against the manifest rather than the table's length.

    The census has three sealed rows. The biosphere manifest freezes SIX sealed ones
    (``sealed_chamber``, ``perennial_chamber``, ``consumer_chamber``, both 15-yr
    long-horizons, ``drift_summary``) and ``water_biting`` is a sealed chamber outside
    the manifest entirely. Three rows nevertheless cover all of them, and this pin is
    what makes that a MEASUREMENT rather than an assumption — the shape that has
    already produced two corrections here (B-finding 4's five rows against seven frozen
    scenarios, A-finding 9's list checked against its own length).
    """
    # (a) The long-horizon goldens reuse the SAME scenario objects; the inventory is a
    #     t=0 property, so lengthening the horizon cannot move it. Bit-identical, not
    #     "equal to N decimals".
    for scenario, short_years in (
        (PERENNIAL_CHAMBER_SCENARIO, PERENNIAL_CHAMBER_YEARS),
        (CONSUMER_CHAMBER_SCENARIO, CONSUMER_CHAMBER_YEARS),
    ):
        short = build_season(scenario)[0]
        assert short_years != LONG_HORIZON_YEARS  # the rows really are different runs
        # Same object => same t=0 state; assert it rather than reason about it.
        long_ = build_season(scenario)[0]
        assert _carbon_total(short).hex() == _carbon_total(long_).hex()

    # (b) `water_biting` is the perennial chamber with soil_water0 moved -- NOT a fourth
    #     carbon jar. Every gas field defaults; only the water IC differs.
    wb, per = WATER_BITING_SCENARIO, PERENNIAL_CHAMBER_SCENARIO
    assert wb.sealed and wb.litter_carbon0 == per.litter_carbon0
    assert wb.chamber_air_mol == per.chamber_air_mol
    assert wb.chamber_co2_mol0 == per.chamber_co2_mol0
    assert wb.chamber_o2_mol0 == per.chamber_o2_mol0
    assert wb.soil_water0 != per.soil_water0  # the one thing that DOES differ
    wb_inventory = _carbon_total(build_season(wb)[0])
    assert wb_inventory.hex() == _carbon_total(build_season(per)[0]).hex()

    # And it behaves like the same jar: the plant still holds most of the carbon.
    wb_states = _run(wb, WATER_BITING_YEARS, "season")
    peak = max(sum(s.stocks[o].amount for o in _ORGANS) for s in wb_states)
    assert 0.55 < peak / wb_inventory < 0.75


@pytest.mark.slow
@pytest.mark.parametrize("name", sorted(_CHAMBERS))
def test_the_plant_holds_most_of_the_systems_carbon_at_peak(name: str) -> None:
    """PIN 3: at peak the crop IS the majority of the chamber's carbon.

    This is the quantitative content of "a sealed chamber's carbon inventory is fixed,
    so any change that parks carbon in a standing pool is paid for out of the CO2
    trough" ((C) finding 8). There is no slack: the plant is already holding most of it.

    ⚠⚠ **RE-MEASURED 2026-08-10 (the humification split), and the SENTENCE ABOVE IS THE
    PART THAT MOVED.** The inventory is still fixed and the plant still holds the
    majority — but the fraction fell (0.558/0.639/0.649/0.670 -> ~0.52-0.55) because
    there is now a third soil pool holding ~10 % of the chamber's carbon at a ~5-yr
    residence time. And the quoted conclusion is measured FALSE as a law: with a slow
    pool present the soil funds a standing sink out of its own inventory rather than out
    of the atmosphere (``test_senescence_form``'s inventory pin measures the CO2 trough
    moving +0.0006 mol C where it used to move -0.0392). The claim was true of a soil
    with one fast pool.

    The bound is widened DOWNWARD only, to the measured range, and the reading it
    supports is the one that survives: at peak the crop is still the single largest
    holder of the chamber's carbon.
    """
    scenario, years, driver = _CHAMBERS[name]
    states = _run(scenario, years, driver)
    inventory = _carbon_total(states[0])
    peak_plant = max(sum(s.stocks[o].amount for o in _ORGANS) for s in states)
    assert 0.50 < peak_plant / inventory < 0.70


@pytest.mark.slow
def test_the_chamber_crop_is_an_order_of_magnitude_below_the_field_crop() -> None:
    """PIN 4+5: "carbon-limited by design", as a measured ratio — and BOTH denominators.

    Same crop, same params, same weather; the only difference is a finite carbon pool.

    The two W denominators are asserted TOGETHER and never bare: 14.954 t/ha includes
    fibrous roots, 12.633 t/ha excludes them (Greenwood's basis, the figure
    `docs/log/chamber-scale.md` quotes — that record was CLAUDE.md's, then the log's).
    This repo has twice been bitten by conflating quantities that differ only
    in their denominator, so the reconciliation is a pin rather than a comment.
    """
    canopy = load_canopy_params()
    carbon_fraction = float(
        load_yaml(
            Path(__file__).resolve().parents[1]
            / "src"
            / "domains"
            / "biosphere"
            / "params"
            / "canopy.yaml"
        )["parameters"]["carbon_fraction"]["value"]
    )

    def t_per_ha(mol_c: float) -> float:
        return mol_c * MOLAR_MASS_CARBON_KG_PER_MOL / carbon_fraction * 10.0

    field = _run(DEFAULT_SCENARIO, 1, "season")
    field_peak = max(sum(s.stocks[o].amount for o in _ORGANS) for s in field)
    field_peak_excl_root = max(
        sum(s.stocks[o].amount for o in (LEAF_C, STEM_C, STORAGE_C)) for s in field
    )
    field_lai = max(
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=canopy.sla_per_mol_c,
            ground_area=DEFAULT_SCENARIO.ground_area,
        )
        for s in field
    )

    # ⚠ BOTH re-measured 2026-08-12 (stem reserves): 14.954 -> 16.619173 and
    # 12.633 -> 14.019448. The two denominators still differ by exactly the fibrous
    # roots, which is the reconciliation this pair exists to make un-forgettable.
    #
    # ⚠ BOTH re-measured again 2026-08-14 (the step unfreeze): 16.612938 -> 16.813163
    # (+1.21 %) and 13.939142 -> 14.107660 (+1.21 %). **They moved by the same relative
    # amount to three figures**, which is the reconciliation holding: the finer step
    # re-integrates the whole plant, and the two bases differ by a fixed set of organs
    # rather than by anything the step touches. A pair that moved by *different*
    # fractions would have meant the reserve's share had shifted, and that is the thing
    # this pair exists to catch.
    #
    # ⚠ THE SECOND FIGURE DELIBERATELY EXCLUDES THE RESERVE. It is Greenwood's basis,
    # owned by ``test_nitrogen_form.py``, and a borrowed threshold must be read on its
    # owner's basis or the comparison is not the one that was calibrated. The first
    # figure is total plant carbon and DOES include it (see ``_ORGANS``). The two
    # bases are different on purpose and the difference is now non-trivial.
    assert t_per_ha(field_peak) == pytest.approx(16.813163, abs=5e-3)
    assert t_per_ha(field_peak_excl_root) == pytest.approx(14.107660, abs=5e-3)

    chamber = _run(PERENNIAL_CHAMBER_SCENARIO, PERENNIAL_CHAMBER_YEARS, "perennial")
    chamber_peak = max(sum(s.stocks[o].amount for o in _ORGANS) for s in chamber)
    chamber_lai = max(
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=canopy.sla_per_mol_c,
            ground_area=PERENNIAL_CHAMBER_SCENARIO.ground_area,
        )
        for s in chamber
    )

    # ~24x in mass, ~10x in leaf area. NOT the same ratio -- mass and leaf area are
    # different quantities (the canopy-regulator row's finding 5 caveat).
    # ⚠ The leaf-area ratio was 8-10x and is now 10.4x: the humification split shrinks
    # the CHAMBER crop (the soil holds ~10 % of the inventory) while leaving the OPEN
    # FIELD untouched -- an open-field build carries no litter, microbial or humus stock
    # at all, which is asserted in this module's structural pin. So the gap widened from
    # the chamber side only. The band is re-measured, not re-centred: it still says the
    # two crops differ by about an order of magnitude in leaf area.
    # ⚠ 2026-08-12 (stem reserves): the ratio moved 27.x -> 31.98. The CLAIM is 'an
    # order of magnitude', and 31.98x is further past it, not short of it — so the band
    # is re-cut around the measurement rather than the claim being restated.
    assert 30.0 < field_peak / chamber_peak < 35.0
    # ⚠⚠ **THIS BAND HAS BEEN RE-CUT THREE TIMES IN FIVE DAYS AND IS NOW DROPPED.**
    # 8-10x -> 10.4x (humification) -> 10.5-11.5 for 11.0125x (stem reserves) -> 11.606x
    # (2026-08-14, the step unfreeze), over the ceiling again. Each re-cut was honest on
    # its own and the sequence is not: a band that is re-centred on the measurement
    # every time the measurement moves is a record of the measurement, not a check on
    # it, and it costs a paragraph of justification each time.
    #
    # Replaced by the two things actually being said, which do not need re-cutting:
    # the ratio PINNED exactly — so any change at all is visible, which is strictly
    # more than any band caught — and the CLAIM the docstring makes, "about an order of
    # magnitude", stated once at the width those words mean.
    ratio = field_lai / chamber_lai
    assert ratio == pytest.approx(11.6065, rel=1e-4)
    assert 3.0 < ratio < 30.0, ("no longer 'about an order of magnitude'", ratio)


def test_making_the_chamber_bigger_cannot_be_the_fix() -> None:
    """PIN 6: the atmospheric route is refuted by the engineering reference.

    A guard against a future "just size the chamber up". The chamber is ALREADY far
    more generous than the flight design, and the enlargement a field-scale crop would
    need is three-plus orders past it -- not a trade-off, a dead branch.
    Pure arithmetic; no simulation.
    """
    scenario = PERENNIAL_CHAMBER_SCENARIO
    molar_volume_20c = R_GAS * 293.15 / P_STD_PA  # m3/mol
    ours_m3_per_m2 = scenario.chamber_air_mol * molar_volume_20c / scenario.ground_area

    assert ours_m3_per_m2 == pytest.approx(24.06, abs=0.01)
    # Already ~23x BVAD's whole design envelope, ~36x its shoot zone.
    assert ours_m3_per_m2 / BVAD_CHAMBER_M3_PER_M2_TOTAL > 20.0
    assert ours_m3_per_m2 / BVAD_CHAMBER_M3_PER_M2_SHOOT_ZONE > 30.0

    # One field crop's standing carbon, measured by pin 4.
    field_peak_mol_c = 56.0267
    factor = field_peak_mol_c / scenario.chamber_co2_mol0
    assert factor > 150.0
    assert (factor * ours_m3_per_m2) / BVAD_CHAMBER_M3_PER_M2_TOTAL > 3000.0

    # Elevating CO2 to the plant optimum is worth single digits, not two orders. [BVAD]:
    # "the optimum partial pressure of carbon dioxide for plant growth is roughly 0.10
    # to 0.20 kPa (Wheeler, et al., 1993)" -- printed pp. 130 AND 175, identical wording
    # in both places (locus checked, not assumed).
    opt_hi_ppm = 0.20 * 1000.0 / P_STD_PA * 1e6
    ours_ppm = scenario.chamber_co2_mol0 / scenario.chamber_air_mol * 1e6
    assert 2.5 < opt_hi_ppm / ours_ppm < 6.0


def test_the_chamber_inventory_measured_in_days_of_demand() -> None:
    """PIN: the inventory expressed against cited demand rates.

    The number that makes the diagnosis legible: this chamber is asked to run 3, 5 and
    15 years closed on ~2 days of one square metre of wheat.
    """
    wheat_c_mol_per_m2_day = BVAD_WHEAT_CO2_UPTAKE_G_PER_M2_D / M_CO2_G_PER_MOL
    assert wheat_c_mol_per_m2_day == pytest.approx(1.7496, abs=1e-4)

    inventory = _EXPECTED_INVENTORY["perennial"]
    assert inventory / wheat_c_mol_per_m2_day == pytest.approx(2.01, abs=0.02)

    atmosphere_hours = (
        PERENNIAL_CHAMBER_SCENARIO.chamber_co2_mol0 / wheat_c_mol_per_m2_day * 24.0
    )
    assert atmosphere_hours == pytest.approx(4.90, abs=0.05)

    # ... and 3.4 hours of ONE crewmember's exhalation.
    crew_hours = inventory / BVAD_CREW_C_MOL_PER_DAY * 24.0
    assert crew_hours == pytest.approx(3.42, abs=0.02)

    # A real BLSS balances ~14 m2 of wheat per crewmember. Ours has 1 m2 and no crew.
    assert BVAD_CREW_C_MOL_PER_DAY / wheat_c_mol_per_m2_day == pytest.approx(
        14.09, abs=0.05
    )
    assert PERENNIAL_CHAMBER_SCENARIO.ground_area == 1.0


def test_one_pool_pins_stock_to_flux_and_ours_is_a_fast_pool() -> None:
    """PIN 7: `stock = flux / k` — an IDENTITY, and the reason the soil pile is small.

    ⚠ The flux comparison is asserted ONLY as an ORDERING, deliberately. Our
    `litter_carbon0 = 3.0` was sized by probe to make O2 depletion dramatic and
    `decomposition_rate` was recalibrated separately for closure; that the product lands
    near ONE cited equilibrium (Hoosfield, n=1, 1852 arable) is a coincidence with a
    mechanism, not a law. Asserting the ratio would be this project's own meta-finding
    again: a number fitted to one scenario written down as a constant. The 94x STOCK
    shortfall is what carries the diagnosis and it stands on its own arithmetic.
    """
    decomposition = load_yaml(
        Path(__file__).resolve().parents[1]
        / "src"
        / "domains"
        / "biosphere"
        / "params"
        / "decomposition.yaml"
    )
    k_per_day = float(decomposition["parameters"]["decomposition_rate"]["value"])
    k_per_year = k_per_day * 365.0
    assert k_per_year == pytest.approx(4.015, abs=1e-3)

    # Ours is a DECOMPOSABLE-PLANT-MATERIAL rate. A real soil holds most of its carbon
    # in pools 13x-201x slower, plus one that never decomposes at all.
    assert k_per_year / ROTHC_K_PER_YEAR["RPM"] > 13.0
    assert k_per_year / ROTHC_K_PER_YEAR["HUM"] > 200.0
    assert k_per_year < ROTHC_K_PER_YEAR["DPM"]

    def t_c_per_ha_to_mol_c_per_m2(t_c_per_ha: float) -> float:
        g_c_per_m2 = t_c_per_ha * 1000.0 * 1000.0 / 1e4
        return g_c_per_m2 / (MOLAR_MASS_CARBON_KG_PER_MOL * 1000.0)

    hoosfield_stock = t_c_per_ha_to_mol_c_per_m2(ROTHC_HOOSFIELD_STOCK_T_C_PER_HA)
    hoosfield_flux = t_c_per_ha_to_mol_c_per_m2(ROTHC_HOOSFIELD_INPUT_T_C_PER_HA_YR)
    assert hoosfield_stock == pytest.approx(281.4, abs=0.2)

    ours_stock = PERENNIAL_CHAMBER_SCENARIO.litter_carbon0
    ours_flux = ours_stock * k_per_year

    # The claim: the STOCK is short by ~2 orders...
    assert hoosfield_stock / ours_stock > 90.0
    # ...while the FLUX is not short at all. ORDERING ONLY -- same order of magnitude.
    assert 0.1 < ours_flux / hoosfield_flux < 10.0

    # Which is why "just enlarge litter_carbon0" was measured to explode: at a fixed k,
    # stock and flux are the SAME knob (decomposer calibration, finding 4).
    assert (hoosfield_stock * k_per_year) / hoosfield_flux > 50.0
