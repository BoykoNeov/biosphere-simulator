"""Phase-6 Step 9 (P6.9): integrated crew-metabolic validation against NASA BVAD.

The literature-validation gate. We run the **integrated** cabin assembly (the real crew
respiring into the ECLSS air, ``station.cabin`` — *not* standalone-crew arithmetic) with
the crew load set to one NASA BVAD crew configuration, and check the crew consumption /
production against BVAD Table 3-31.

**Primary source** (in ``docs/bvad-reference.md``): NASA/TP-2015-218570
**Rev 2** (Feb 2022), *Life Support Baseline Values and Assumptions Document*, Table
**3-31** "Summary of Nominal Human Metabolic Interface Values", p. 58 — an 82 kg
reference crewmember, RQ 0.860. Public domain (US Govt work); we cite the document,
copy no dataset (``docs/reuse-and-licenses.md``).

**The load-bearing framing: calibration ≠ validation (the three columns).** Our crew
flows are *forced* (intake rates are scenario data) and the metabolic splits are params
(``crew.yaml``, now BVAD-calibrated), so every quantity we *set* matches BVAD **by
construction**. This test keeps the three columns visibly separate:

* **Calibration checkpoints** (``test_calibration_*``): CO₂, feces, humidity,
  urine. We *set* the intake rates + fractions to reproduce these, so a match is
  bookkeeping, **not** validation. Asserted to a tight band only to catch a wiring
  regression, and labelled as checkpoints, not the payload.
* **The structural prediction** (``test_rq_structural_prediction`` — THE payload, the
  one assertion that can genuinely fail) — ``CrewRespiration`` is PQ = 1 ⇒ **RQ = 1.0**
  (one mol O₂ consumed per mol CO₂ produced), independent of the fraction values.
  Calibrating CO₂ to BVAD forces the model's O₂ consumption **≈ 11.8 % low** vs BVAD
  (RQ 0.86 nominal / 0.881 daily-effective). Pinned as a **number**:
  ``model_O2 / bvad_O2 = 0.8814 ± tol`` — a regression that changed RQ trips it.
* **Closure** (``test_closure_*``) — what "integrated" buys over arithmetic: at steady
  state the ECLSS scrubber removal flux = crew CO₂ production flux and the O₂ makeup
  flux = crew O₂ consumption flux (equipment throughput matches the crew load).

**Not modeled (documented gaps, not tested as failures):** metabolic water (0.490
kg/CM-d — ``WaterBalance`` is intake-split only), metabolic heat (143.8 W/CM — the crew
is not an ENERGY source into ``thermal.node``), and RQ variation with activity. See the
module docstring of ``crew.yaml`` and ``docs/bvad-reference.md``.

No golden: this is a validation-against-reference test (the ``lab.oracle_match``
precedent — a computed comparison, not a regression pin). The crew ``crew.yaml``
recalibration this step ships *does* regenerate the six downstream non-frozen goldens
(``crew_state`` / ``cabin_gas_state`` / ``greenhouse_state`` / ``harvest_state`` /
``water_recovery_state`` / ``sealed_station_state``); the seven frozen biosphere goldens
+ Power/Thermal/ECLSS stay byte-identical (``crew.yaml`` is not in the freeze manifest).
"""

import pytest

from domains.crew.loader import load_crew_params
from domains.crew.stocks import FECAL_WASTE, URINE
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import (
    CABIN_CO2,
    CABIN_H2O,
    CABIN_O2,
    CO2_REMOVED,
    HUMIDITY_CONDENSATE,
    O2_SUPPLY,
)
from simcore.integrator import EulerIntegrator
from station.cabin import build_cabin, cabin_resolver, cabin_steady_state
from station.scenario import CABIN_GAS_STEPS, CabinScenario

# --- NASA BVAD Table 3-31 nominal per-crewmember-per-day values (Rev 2, 2022, p.58) ---
# Verbatim from docs/bvad-reference.md. "-d" = per crewmember per day.
BVAD_CO2_LOAD_KG = 1.085  # −m Carbon Dioxide Load
BVAD_O2_CONSUMED_KG = 0.895  # +m Oxygen Consumed
BVAD_FECAL_SOLID_DRY_KG = 0.032  # −m Fecal Solid Waste, dry basis
BVAD_RESP_PERSP_WATER_KG = 2.946  # −m Respiration and Perspiration Water
BVAD_URINE_WATER_KG = 1.420  # −m Urine Water
# (BVAD food solids 0.800 kg dry/CM-d is deliberately NOT validated: the model's food
# store is CARBON-only, so kg-of-dry-food never enters — only its carbon content does,
# via the C_CO₂ + C_feces balance below. BVAD nominal RQ is 0.860; the test uses the
# daily-effective molar RQ 0.881, which blends BVAD's nominal + exercise periods.)

# Molar masses (g/mol) for the CARBON/OXYGEN mol accounting our stocks use.
M_CO2 = 44.009
M_O2 = 31.998
M_C = 12.011

# Carbon fraction of dry human feces (Rose et al. 2015, 44–55 %; midpoint) — the one
# assumption needed to close the model's food-carbon balance against BVAD's kg-of-feces.
FECES_CARBON_FRACTION = 0.50

SECONDS_PER_DAY = 86_400.0

# BVAD per-CM-day values converted to our mol accounting.
BVAD_CO2_MOL = BVAD_CO2_LOAD_KG * 1000.0 / M_CO2  # 24.654 mol CO₂ (= mol C respired)
BVAD_O2_MOL = BVAD_O2_CONSUMED_KG * 1000.0 / M_O2  # 27.970 mol O₂
# Daily-effective molar RQ (blends BVAD's nominal + exercise periods): 24.654/27.970.
BVAD_RQ_EFFECTIVE = BVAD_CO2_MOL / BVAD_O2_MOL  # 0.8814

# The model's food-carbon intake per CM-day is, by the steady-body-mass carbon balance,
# C_food = C_CO₂ + C_feces (the two carbon fates CrewRespiration books) — what the crew
# load is calibrated to; f_resp then splits it back into the BVAD CO₂ and feces.
BVAD_FECES_C_MOL = (
    BVAD_FECAL_SOLID_DRY_KG * FECES_CARBON_FRACTION * 1000.0 / M_C
)  # 1.33
BVAD_FOOD_C_MOL = BVAD_CO2_MOL + BVAD_FECES_C_MOL  # 25.986 mol C
# The two modeled water fates (humidity + urine); metabolic + fecal water not modeled.
BVAD_MODELED_WATER_KG = BVAD_RESP_PERSP_WATER_KG + BVAD_URINE_WATER_KG  # 4.366 kg

# One crew configuration: a 4-crewmember station complement. Everything is linear in the
# crew count, so the per-CM comparison (throughput / N_CREW) is N-invariant; 4 is a
# realistic complement that keeps every cabin stock comfortably positive.
N_CREW = 4


_CREW = load_crew_params()
_ECLSS = load_eclss_params()


def _bvad_cabin_scenario() -> CabinScenario:
    """The BVAD-calibrated cabin: crew load = N_CREW × per-CM BVAD, on the shipped ECLSS
    sizing. Built here (not in ``station.scenario``) so it does not touch the shipped
    ``CABIN_GAS_SCENARIO`` or its golden — a validation scenario, not a pinned run.
    Stores are sized well-fed over the 15 h steady-state horizon (food depletes ~3 %,
    water ~18 %), so ``rationed == 0``; cabin O₂ is drawn to ~9.4 mol (below the 10 mol
    setpoint) but stays comfortably positive.

    **Carbon is calibrated to BVAD's CO₂ load, not the derived food-carbon total.** The
    food intake is set so ``f_resp · food_intake`` reproduces BVAD's directly-measured
    CO₂ (1.085 kg = 24.654 mol) *exactly given the shipped rounded f_resp*. CO₂
    is the primary interface value (sweat-test MetMan); the food-carbon total
    (``BVAD_FOOD_C_MOL``) also depends on the 44–55 % feces-carbon assumption, so
    feces is the secondary checkpoint (matching to the f_resp rounding, ~0.6 %). This
    keeps the O₂ headline a clean number (model O₂ = model CO₂ = BVAD CO₂ exactly,
    RQ = 1). Water is calibrated to the modeled-water total; humidity/urine match to
    the f_ins rounding (~0.1 %).
    """
    food_intake_rate = (
        N_CREW * BVAD_CO2_MOL / _CREW.respired_carbon_fraction / SECONDS_PER_DAY
    )
    water_intake_rate = N_CREW * BVAD_MODELED_WATER_KG / SECONDS_PER_DAY
    return CabinScenario(
        food_store0=2000.0,
        water_store0=60.0,
        food_intake_rate=food_intake_rate,
        water_intake_rate=water_intake_rate,
    )


_SCENARIO = _bvad_cabin_scenario()
_DT = _SCENARIO.dt_seconds


def _run_to_steady_state() -> tuple:
    """Run the integrated cabin to steady state; return the last two states (for
    reading steady-state fluxes as one-step boundary deltas) + the resolver."""
    state, registry = build_cabin(_CREW, _ECLSS, _SCENARIO)
    resolver = cabin_resolver(_SCENARIO)
    integrator = EulerIntegrator(registry)
    prev = state
    for _ in range(CABIN_GAS_STEPS):
        prev = state
        state = integrator.step_report(state, resolver, _DT).state
    return prev, state, resolver


def _flux(prev, final, stock_id) -> float:
    """Steady-state flux (per second) of a boundary reservoir, from a one-step delta.

    ``abs`` so it reads the same for a sink that grows (``co2_removed``) and a source
    that drains (``o2_supply`` goes negative as it supplies)."""
    return abs(final.stocks[stock_id].amount - prev.stocks[stock_id].amount) / _DT


def _per_cm_per_day(flux_per_s: float) -> float:
    """Convert a whole-crew per-second flux to a per-crewmember per-day amount."""
    return flux_per_s * SECONDS_PER_DAY / N_CREW


# --- Steady-state sanity: the run reached steady state, well-fed ----------------------


def test_reaches_steady_state_well_fed() -> None:
    """The cabin converges to its closed-form steady state with no rationing — the
    precondition for reading steady-state fluxes as one-step deltas."""
    prev, final, _ = _run_to_steady_state()
    ss = cabin_steady_state(_CREW, _ECLSS, _SCENARIO)
    assert final.stocks[CABIN_O2].amount == pytest.approx(ss.cabin_o2, abs=1e-6)
    assert final.stocks[CABIN_CO2].amount == pytest.approx(ss.cabin_co2, abs=1e-6)
    assert final.stocks[CABIN_H2O].amount == pytest.approx(ss.cabin_h2o, abs=1e-6)
    # At steady state consecutive states are equal to round-off (fluxes are constant).
    assert final.stocks[CABIN_O2].amount == pytest.approx(
        prev.stocks[CABIN_O2].amount, abs=1e-9
    )


# --- Calibration checkpoints (set → matches; NOT validation) --------------------------


def test_calibration_co2_production() -> None:
    """CO₂ production reproduces BVAD (1.085 kg/CM-d). CALIBRATION CHECKPOINT — we set
    the intake + f_resp to hit this, so the match is bookkeeping. The band is
    tight (a regression detector), not a validation tolerance."""
    prev, final, _ = _run_to_steady_state()
    co2_mol = _per_cm_per_day(_flux(prev, final, CO2_REMOVED))
    assert co2_mol == pytest.approx(BVAD_CO2_MOL, rel=1e-6)
    assert co2_mol * M_CO2 / 1000.0 == pytest.approx(BVAD_CO2_LOAD_KG, rel=1e-6)


def test_calibration_feces() -> None:
    """Fecal solid (dry) reproduces BVAD (0.032 kg/CM-d) via the 0.50 carbon fraction.
    CALIBRATION CHECKPOINT — feces is the *secondary* carbon fate (carbon is calibrated
    to CO₂), so it matches only to the precision of the rounded f_resp (~0.6 %),
    itself well inside the 44–55 % feces-carbon uncertainty. Band is rel=1e-2."""
    prev, final, _ = _run_to_steady_state()
    feces_c_mol = _per_cm_per_day(_flux(prev, final, FECAL_WASTE))
    feces_dry_kg = feces_c_mol * M_C / 1000.0 / FECES_CARBON_FRACTION
    assert feces_dry_kg == pytest.approx(BVAD_FECAL_SOLID_DRY_KG, rel=1e-2)


def test_calibration_humidity_and_urine() -> None:
    """Respiration+perspiration water (2.946) and urine water (1.420 kg/CM-d) reproduce
    BVAD via ``insensible_water_fraction`` = 0.675. CALIBRATION CHECKPOINT."""
    prev, final, _ = _run_to_steady_state()
    humidity_kg = _per_cm_per_day(_flux(prev, final, HUMIDITY_CONDENSATE))
    urine_kg = _per_cm_per_day(_flux(prev, final, URINE))
    assert humidity_kg == pytest.approx(BVAD_RESP_PERSP_WATER_KG, rel=1e-3)
    assert urine_kg == pytest.approx(BVAD_URINE_WATER_KG, rel=1e-3)


# --- The structural prediction (THE payload — can genuinely fail) ---------------------


def test_rq_structural_prediction() -> None:
    """THE headline result. ``CrewRespiration`` fixes RQ = 1.0 (PQ = 1), so with CO₂
    calibrated to BVAD the model's O₂ comes out at the *daily-effective molar
    RQ* below BVAD's O₂: ``model_O2 / bvad_O2 = 0.8814``. This is the one genuinely
    un-tuned output — it does not depend on the fraction values, and a regression that
    silently changed RQ (e.g. altering the respiration stoichiometry) would move it.

    Pinned as a number, not a bound (the ``lab.fit_order`` "measure the known structural
    error" discipline)."""
    prev, final, _ = _run_to_steady_state()
    model_o2_mol = _per_cm_per_day(_flux(prev, final, O2_SUPPLY))

    # The model consumes exactly its CO₂ production in O₂ (RQ = 1) — so O₂ equals the
    # BVAD CO₂ molar value, not the BVAD O₂ value.
    assert model_o2_mol == pytest.approx(BVAD_CO2_MOL, rel=1e-6)

    # The headline: O₂ consumption is the daily-effective RQ fraction of BVAD's O₂.
    ratio = model_o2_mol / BVAD_O2_MOL
    assert ratio == pytest.approx(BVAD_RQ_EFFECTIVE, rel=1e-6)
    assert ratio == pytest.approx(0.8814, abs=1e-4)
    # i.e. the model under-predicts O₂ consumption by ~11.8 % — a documented consequence
    # of the fixed RQ = 1, not a parameter error.
    model_o2_kg = model_o2_mol * M_O2 / 1000.0
    assert (BVAD_O2_CONSUMED_KG - model_o2_kg) / BVAD_O2_CONSUMED_KG == pytest.approx(
        0.118, abs=2e-3
    )


# --- Closure: equipment throughput = crew load (what "integrated" buys) ---------------


def test_closure_scrubber_matches_production() -> None:
    """At steady state the ECLSS CO₂ scrubber removal flux equals crew CO₂ production
    flux (``f_resp · food_intake``): equipment throughput = crew load. This is the
    integrated content — the scrubber is sized (illustrative τ) but its *throughput* is
    pinned to the crew, so it validates the same BVAD CO₂ number the crew produces."""
    prev, final, _ = _run_to_steady_state()
    scrubber_flux = _flux(prev, final, CO2_REMOVED)
    crew_co2_production = _CREW.respired_carbon_fraction * _SCENARIO.food_intake_rate
    assert scrubber_flux == pytest.approx(crew_co2_production, rel=1e-9)


def test_closure_makeup_matches_consumption() -> None:
    """At steady state the ECLSS O₂ makeup flux equals the crew O₂ consumption flux
    (``f_resp · food_intake``, RQ = 1): the makeup replaces exactly what the crew burns.
    Because RQ = 1, this equals the scrubber throughput — the model's O₂ and CO₂ loops
    move the same molar rate, which is precisely why it cannot match BVAD's RQ 0.86."""
    prev, final, _ = _run_to_steady_state()
    makeup_flux = _flux(prev, final, O2_SUPPLY)
    crew_o2_consumption = _CREW.respired_carbon_fraction * _SCENARIO.food_intake_rate
    assert makeup_flux == pytest.approx(crew_o2_consumption, rel=1e-9)


# --- The recalibrated crew.yaml values are the BVAD-derived ones ----------------------


def test_crew_params_are_bvad_calibrated() -> None:
    """The shipped ``crew.yaml`` carries the BVAD-derived fractions (guards against a
    silent revert to the old illustrative 0.85 / 0.4)."""
    assert _CREW.respired_carbon_fraction == pytest.approx(0.949, abs=1e-9)
    assert _CREW.insensible_water_fraction == pytest.approx(0.675, abs=1e-9)
