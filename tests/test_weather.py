"""Phase-1 Step-11 tests: weather → flow-driver conversions (clean-room, cited).

``domains.biosphere.weather`` turns raw NASAPower facts (IRRAD, TEMP, VAP, latitude,
day-of-year) into the scalar drivers the season's flows read (incident PAR, net
radiation, VPD, photoperiod). Each conversion is checked against an **independent
hand-computed literal** (not a restatement of the implementation), plus the documented
edge cases (equatorial 12 h day, polar clamp, zero radiation, the VPD ≥ 0 clamp).
"""

import math

import pytest

from domains.biosphere.transpiration import saturation_vapor_pressure
from domains.biosphere.weather import (
    ALBEDO,
    PAR_ENERGY_FRACTION,
    PAR_UMOL_PER_J,
    daylength_seconds,
    incident_par,
    net_radiation,
    vapor_pressure_deficit,
)


# --- daylength_seconds ------------------------------------------------------
def test_daylength_at_equator_is_twelve_hours() -> None:
    # At the equator ωs = arccos(0) = π/2 ⇒ N = 12 h for every day-of-year.
    for doy in (1, 80, 172, 200, 355):
        assert math.isclose(daylength_seconds(0.0, doy), 12.0 * 3600.0, rel_tol=1e-12)


def test_daylength_longer_summer_than_winter_in_north() -> None:
    # 52°N: midsummer (~doy 172) day is much longer than midwinter (~doy 355).
    summer = daylength_seconds(52.0, 172)
    winter = daylength_seconds(52.0, 355)
    assert summer > 15.0 * 3600.0  # > 15 h
    assert winter < 9.0 * 3600.0  # < 9 h
    assert summer > winter


def test_daylength_polar_clamp_is_full_day() -> None:
    # Above the Arctic circle near the solstice the arccos argument exceeds 1 and is
    # clamped: a full 24 h polar day rather than a math-domain error.
    assert math.isclose(daylength_seconds(80.0, 172), 24.0 * 3600.0, rel_tol=1e-12)


# --- incident_par -----------------------------------------------------------
def test_incident_par_known_value() -> None:
    # IRRAD = 8.64e6 J/m²/day over a 43200 s daylight window:
    #   mean PAR irradiance = 0.5 · 8.64e6 / 43200 = 100 W/m²; × 4.57 = 457 µmol/m²/s.
    assert math.isclose(incident_par(8.64e6, 43200.0), 457.0, rel_tol=1e-12)


def test_incident_par_matches_formula() -> None:
    irrad, daylen = 2.5e7, 57600.0
    expected = PAR_ENERGY_FRACTION * irrad / daylen * PAR_UMOL_PER_J
    assert math.isclose(incident_par(irrad, daylen), expected, rel_tol=1e-12)


def test_incident_par_zero_radiation_is_zero() -> None:
    assert incident_par(0.0, 43200.0) == 0.0


def test_incident_par_rejects_non_positive_daylength() -> None:
    with pytest.raises(ValueError, match="daylength_s must be > 0"):
        incident_par(1.0e7, 0.0)


# --- net_radiation ----------------------------------------------------------
def test_net_radiation_known_value() -> None:
    # IRRAD = 8.64e6 J/m²/day ⇒ Rs = 100 W/m²; Rns = (1 − 0.23)·100 = 77 W/m².
    assert math.isclose(net_radiation(8.64e6), 77.0, rel_tol=1e-12)
    assert ALBEDO == 0.23


def test_net_radiation_zero_is_zero() -> None:
    assert net_radiation(0.0) == 0.0


# --- vapor_pressure_deficit -------------------------------------------------
def test_vpd_known_value() -> None:
    # VPD = e_s(20 °C) − e_a, e_a = 15 hPa = 1500 Pa. e_s via the shared Tetens form.
    expected = saturation_vapor_pressure(20.0) - 1500.0
    assert expected > 0.0  # sanity: e_s(20) ≈ 2338 Pa > 1500 Pa
    assert math.isclose(vapor_pressure_deficit(20.0, 15.0), expected, rel_tol=1e-12)


def test_vpd_clamps_to_zero_when_saturated() -> None:
    # e_a above e_s(T) (a humid daily mean) ⇒ VPD clamps to 0, never negative.
    assert vapor_pressure_deficit(10.0, 50.0) == 0.0


def test_vpd_rejects_negative_vap() -> None:
    with pytest.raises(ValueError, match="vap_hpa must be >= 0"):
        vapor_pressure_deficit(20.0, -1.0)
