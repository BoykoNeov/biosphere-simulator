"""Phase-1 Step-7 tests: Penman–Monteith transpiration + root uptake (WATER flows).

The first WATER-currency process. Layers (mirroring Steps 5/6):

* **Rate laws** (``domains.biosphere.transpiration``, pure stdlib): the saturation
  vapour pressure against textbook/FAO-56 magnitudes and its slope against the analytic
  derivative (finite-difference cross-check); the Penman–Monteith combination equation
  at a pinned operating point, its dark/dry floor, and the negative-radiation clamp; the
  soil-water stress factor ``f_water`` cardinal values + the wilting/critical clamp.
* **The assembled flows**: ``Transpiration`` (water-balanced, dt-linear, self-limiting
  via ``f_water`` on the step-entry soil water) and ``Irrigation`` (water-balanced,
  tracking the irrigation forcing).
* **Config boundary** (``load_transpiration_params``): the committed file loads to the
  expected params; bad units / out-of-range values / a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.loader import (
    TRANSPIRATION_PARAMS_PATH,
    load_transpiration_params,
)
from domains.biosphere.soil_layers import captured_water
from domains.biosphere.transpiration import (
    Irrigation,
    Transpiration,
    TranspirationParams,
    fraction_transpirable,
    penman_monteith_transpiration,
    saturation_vapor_pressure,
    slope_svp,
    soil_water_stress,
    transpirable_capacity,
    water_stress_factor,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# The committed winter-wheat provisional placeholders (mirror transpiration.yaml).
_RA, _RS = 50.0, 70.0


def _params() -> TranspirationParams:
    return TranspirationParams(aerodynamic_resistance=_RA, surface_resistance=_RS)


# --- saturation vapour pressure + slope: FAO-56 magnitudes ------------------
# The e_s magnitudes are textbook/FAO-56 values (e_s(20 °C) ≈ 2.34 kPa) — a genuine
# external cross-check. The slope literals are formula-consistent (FAO-56's 4098-form
# and this module's analytic B·C derivative agree to these digits), so the *independent*
# slope verification is the finite-difference derivative test below, not these numbers.
@pytest.mark.parametrize(
    ("temp", "es_kpa", "slope_kpa"),
    [
        (0.0, 0.6108, 0.04445),
        (10.0, 1.2280, 0.08228),
        (20.0, 2.3383, 0.14474),
        (30.0, 4.2431, 0.24336),
    ],
)
def test_svp_and_slope_match_fao56_table(
    temp: float, es_kpa: float, slope_kpa: float
) -> None:
    assert math.isclose(saturation_vapor_pressure(temp) / 1000.0, es_kpa, rel_tol=2e-3)
    assert math.isclose(slope_svp(temp) / 1000.0, slope_kpa, rel_tol=2e-3)


def test_slope_is_the_analytic_derivative_of_svp() -> None:
    # Δ ≈ d(e_s)/dT by finite difference (the slope is the exact derivative).
    t, h = 18.0, 1e-4
    numeric = (saturation_vapor_pressure(t + h) - saturation_vapor_pressure(t - h)) / (
        2 * h
    )
    assert math.isclose(slope_svp(t), numeric, rel_tol=1e-6)


# --- Penman–Monteith: pinned operating point + the floor --------------------
def test_penman_monteith_pinned_value() -> None:
    # Rn=200 W/m², VPD=1000 Pa, T=20 °C, r_a=50, r_s=70 s/m ⇒ ~6.16 mm/day (a
    # realistic summer potential transpiration). Pinned regression literal.
    tp = penman_monteith_transpiration(
        200.0, 1000.0, 20.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    assert math.isclose(tp, 6.158958394549651, rel_tol=1e-12)


def test_penman_monteith_zero_energy_zero_vpd_is_zero() -> None:
    # No available energy and no vapour deficit ⇒ no evaporative demand.
    tp = penman_monteith_transpiration(
        0.0, 0.0, 20.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    assert tp == 0.0


def test_penman_monteith_clamps_negative_radiation_to_zero() -> None:
    # Daily-average net radiation goes negative on short midwinter days (the
    # winter-wheat season overwinters). Unclamped λE would be negative (≈−0.47
    # mm/day here), flipping the sink into a deposit — the demand-side analogue of
    # the Step 5/6 clamps. Potential transpiration is clamped to 0 (no dew model, P1).
    tp = penman_monteith_transpiration(
        -80.0, 50.0, 2.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    assert tp == 0.0


def test_penman_monteith_rejects_non_positive_aerodynamic_resistance() -> None:
    with pytest.raises(ValueError, match="aerodynamic_resistance"):
        penman_monteith_transpiration(
            200.0, 1000.0, 20.0, aerodynamic_resistance=0.0, surface_resistance=_RS
        )


# --- soil-water stress: TTSW / FTSW / WSFG ----------------------------------
# ⚠ These replaced an absolute-kg ramp on 2026-08-12 (the geometry re-basing). The old
# tests pinned `f_water(soil_water, sw_wilting=10, sw_critical=30)`; those thresholds
# were kilograms, and a kilogram threshold is only meaningful against a store of one
# size. See docs/plans/post-roadmap-soil-water-rebasing.md.
#
# The geometry used below is deliberately round: depth 1.0 m x EXTR 0.13 x rho 1000 x
# 1 m2 = 130 kg of capacity, so FTSW = soil_water / 130 and the WSSG = 0.30 threshold
# sits at exactly 39 kg. Nothing here depends on the frozen scenario's numbers.
_EXTR = 0.13
_WSSG = 0.30


def test_transpirable_capacity_is_the_column_arithmetic() -> None:
    # m x (m3/m3) x kg/m3 x m2 = kg, linear in every argument.
    assert transpirable_capacity(
        1.0, soil_extractable_water=_EXTR, ground_area=1.0
    ) == pytest.approx(130.0)
    assert transpirable_capacity(
        0.15, soil_extractable_water=_EXTR, ground_area=2.0
    ) == pytest.approx(39.0)
    assert (
        transpirable_capacity(0.0, soil_extractable_water=_EXTR, ground_area=1.0) == 0.0
    )


def test_transpirable_capacity_agrees_with_captured_water() -> None:
    """The two names for one product must not drift.

    ``captured_water`` prices a newly explored SLAB, ``transpirable_capacity`` the whole
    COLUMN. A season is a closed cycle only while they are the same arithmetic — the
    re-sow return uses one and the stress denominator the other.
    """
    for depth in (0.15, 0.4, 1.3):
        for area in (1.0, 2.5):
            assert transpirable_capacity(
                depth, soil_extractable_water=_EXTR, ground_area=area
            ) == captured_water(depth, soil_extractable_water=_EXTR, ground_area=area)


@pytest.mark.parametrize(
    ("soil_water", "expected"),
    [
        (0.0, 0.0),  # empty root zone
        (39.0, 0.30),  # exactly at the WSSG threshold
        (65.0, 0.50),  # half the capacity
        (130.0, 1.0),  # the drained upper limit
        (260.0, 2.0),  # ⚠ NOT clamped: over-filled is a real, reportable state
    ],
)
def test_fraction_transpirable_is_atsw_over_ttsw(
    soil_water: float, expected: float
) -> None:
    assert math.isclose(
        fraction_transpirable(soil_water, 130.0), expected, rel_tol=1e-12
    )


def test_fraction_transpirable_returns_zero_for_zero_capacity() -> None:
    """A root zone of no depth holds no transpirable water, so a crop in it is
    maximally stressed. Returning 0.0 rather than raising keeps callers branch-free;
    it is reachable only before emergence, since ``rooted_depth0`` is a cited positive.
    """
    assert fraction_transpirable(5.0, 0.0) == 0.0
    assert fraction_transpirable(0.0, 0.0) == 0.0


@pytest.mark.parametrize(
    ("ftsw", "expected"),
    [
        (0.0, 0.0),  # no transpirable water left
        (0.075, 0.25),  # a quarter of the way up the WSSG ramp
        (0.15, 0.5),  # half
        (0.30, 1.0),  # at the threshold
        (0.85, 1.0),  # above it: unstressed
        (1.30, 1.0),  # over-filled is still just unstressed, never > 1
    ],
)
def test_water_stress_factor_cardinal_values(ftsw: float, expected: float) -> None:
    assert math.isclose(
        water_stress_factor(ftsw, threshold=_WSSG), expected, rel_tol=1e-12
    )


def test_water_stress_factor_has_no_wilting_floor() -> None:
    """[F] Box 14.1's ``WSFG = FTSW / WSSG`` reaches zero only AT ``FTSW = 0``.

    The absolute-kg ramp this replaced returned a hard 0.0 at and below ``sw_wilting``,
    which is where the old structural-positivity guarantee came from. The fraction form
    is asymptotic instead, so positivity is measured (the arbitration backstop stays at
    zero firings across the roster) rather than given. Pinned so the difference is not
    rediscovered as a surprise.
    """
    assert water_stress_factor(1e-12, threshold=_WSSG) > 0.0
    assert water_stress_factor(0.0, threshold=_WSSG) == 0.0


def test_water_stress_factor_rejects_non_positive_threshold() -> None:
    with pytest.raises(ValueError, match="threshold > 0"):
        water_stress_factor(0.5, threshold=0.0)


def test_soil_water_stress_composes_the_three() -> None:
    # 39 kg in a 1.0 m root zone is FTSW 0.30 == WSSG, so exactly unstressed;
    # 19.5 kg is FTSW 0.15, i.e. half the ramp.
    assert soil_water_stress(
        39.0, 1.0, soil_extractable_water=_EXTR, ground_area=1.0, threshold=_WSSG
    ) == pytest.approx(1.0)
    assert soil_water_stress(
        19.5, 1.0, soil_extractable_water=_EXTR, ground_area=1.0, threshold=_WSSG
    ) == pytest.approx(0.5)


def test_a_full_root_zone_is_unstressed_at_every_depth() -> None:
    """FTSW0 = MAI independent of depth — the property that makes the re-basing safe.

    This is the whole reason the frozen roster survives the change: at the drained
    upper limit a crop is unstressed however shallow its root zone. The deleted
    absolute-kg band could not express it, and read a full 19.5 kg zone as BELOW
    wilting — which killed every sealed chamber (that is not a hypothetical; it was
    measured before this form was written).
    """
    for depth in (0.15, 0.3, 0.75, 1.3):
        full = transpirable_capacity(
            depth, soil_extractable_water=_EXTR, ground_area=1.0
        )
        assert (
            soil_water_stress(
                full,
                depth,
                soil_extractable_water=_EXTR,
                ground_area=1.0,
                threshold=_WSSG,
            )
            == 1.0
        )


# --- the assembled flows ----------------------------------------------------
_BIO = DomainId("biosphere")
_BOUNDARY = DomainId("boundary")
_SOIL_WATER = StockId("biosphere.soil_water")
_VAPOR = StockId("boundary.vapor")
_WATER_SOURCE = StockId("boundary.irrigation_supply")

# The assembled-flow fixtures use the same round geometry: a 1.0 m root zone over 1 m2
# holds 130 kg, so `soil_water0=25.0` is FTSW 0.1923 and WSFG = 0.641025641...
_ROOTED_DEPTH = "biosphere.rooted_depth"
_DEPTH = 1.0


def _state(soil_water0: float, depth: float = _DEPTH) -> State:
    water = canonical_unit(Quantity.WATER)
    soil = Stock(
        id=_SOIL_WATER,
        domain=_BIO,
        quantity=Quantity.WATER,
        unit=water,
        amount=soil_water0,
        kind=StockKind.POOL,
    )
    vapor = Stock(
        id=_VAPOR,
        domain=_BOUNDARY,
        quantity=Quantity.WATER,
        unit=water,
        amount=0.0,
        kind=StockKind.BOUNDARY,
    )
    supply = Stock(
        id=_WATER_SOURCE,
        domain=_BOUNDARY,
        quantity=Quantity.WATER,
        unit=water,
        amount=1.0e9,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )
    return State(
        n=0,
        stocks={_SOIL_WATER: soil, _VAPOR: vapor, _WATER_SOURCE: supply},
        rng_seed=0,
        aux={_ROOTED_DEPTH: depth},
    )


def _env(snapshot: State, dt: float, *, rn: float = 200.0, irrigation: float = 5.0):  # noqa: ANN202
    resolver = SourceResolver(
        forcings={
            "rn": constant(rn),
            "vpd": constant(1000.0),
            "temp": constant(20.0),
            "irrigation": constant(irrigation),
        }
    )
    return resolver.bind(snapshot, dt)


def _transpiration_flow(ground_area: float = 1.0) -> Transpiration:
    return Transpiration(
        id=FlowId("biosphere.transpiration"),
        priority=0,
        soil_water=_SOIL_WATER,
        vapor_sink=_VAPOR,
        rn_var="rn",
        vpd_var="vpd",
        temp_var="temp",
        params=_params(),
        ground_area=ground_area,
        rooted_depth_aux=_ROOTED_DEPTH,
        soil_extractable_water=_EXTR,
        wssg=_WSSG,
    )


def test_transpiration_leg_is_pm_times_fwater_times_area() -> None:
    # soil_water=25 in a 130 kg root zone ⇒ FTSW=0.1923… ⇒ WSFG=FTSW/0.30=0.641025…;
    # potential=6.1590 mm/day; area=1 m².
    state = _state(soil_water0=25.0)
    result = _transpiration_flow(ground_area=1.0).evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    potential = penman_monteith_transpiration(
        200.0, 1000.0, 20.0, aerodynamic_resistance=_RA, surface_resistance=_RS
    )
    f_water = (25.0 / 130.0) / _WSSG
    expected = potential * f_water * 1.0
    assert math.isclose(legs[_SOIL_WATER], -expected, rel_tol=1e-12)
    assert math.isclose(legs[_VAPOR], expected, rel_tol=1e-12)
    # Cross-check against the pinned composed literal (kg/day at this point).
    assert math.isclose(expected, 3.948050252916443, rel_tol=1e-12)


def test_transpiration_is_water_balanced() -> None:
    state = _state(soil_water0=25.0)
    result = _transpiration_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_transpiration_shuts_off_at_an_empty_root_zone() -> None:
    # ⚠ The shutoff moved: WSFG reaches 0 only at FTSW = 0, i.e. an EMPTY root zone,
    # where the old kg-ramp shut off at a nonzero wilting mass. An empty zone still
    # gives exactly zero, so the flow can never drive the pool negative from 0.
    state = _state(soil_water0=0.0)
    result = _transpiration_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_SOIL_WATER] == 0.0
    assert legs[_VAPOR] == 0.0


def test_transpiration_scales_with_ground_area_under_the_extensive_transform() -> None:
    """Tripling the PLOT triples the flux only if the WATER triples with it.

    ⚠ The 2026-08-12 re-basing changed what this property means, and the change is the
    physically right one. ``ground_area`` used to appear once (in the demand); it now
    appears in ``TTSW`` as well, so a three-times-larger plot holding the SAME absolute
    water is three times drier and does NOT transpire three times as much. The
    similarity transform that does hold scales the store too — the same extensive-IC
    reasoning ``test_crew_coupled_loop`` pins for whole scenarios.
    """
    one = next(
        leg.amount
        for leg in _transpiration_flow(1.0)
        .evaluate(_state(25.0), _env(_state(25.0), 1.0), 1.0)
        .legs
        if leg.stock == _VAPOR
    )
    state3 = _state(75.0)
    triple = next(
        leg.amount
        for leg in _transpiration_flow(3.0)
        .evaluate(state3, _env(state3, 1.0), 1.0)
        .legs
        if leg.stock == _VAPOR
    )
    assert math.isclose(triple, 3.0 * one, rel_tol=1e-12)


def test_a_bigger_plot_on_the_same_water_is_drier() -> None:
    """The other half of the above, stated as its own claim rather than left implied."""
    state = _state(soil_water0=25.0)
    one = next(
        leg.amount
        for leg in _transpiration_flow(1.0).evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _VAPOR
    )
    triple = next(
        leg.amount
        for leg in _transpiration_flow(3.0).evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _VAPOR
    )
    # Three times the demand, one third the FTSW: the two factors cancel exactly.
    assert math.isclose(triple, one, rel_tol=1e-12)


def test_transpiration_scales_linearly_with_dt() -> None:
    state = _state(soil_water0=25.0)
    flow = _transpiration_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _SOIL_WATER
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _SOIL_WATER
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


def _irrigation_flow(ground_area: float = 2.0) -> Irrigation:
    return Irrigation(
        id=FlowId("biosphere.irrigation"),
        priority=0,
        water_source=_WATER_SOURCE,
        soil_water=_SOIL_WATER,
        irrigation_var="irrigation",
        ground_area=ground_area,
        rooted_depth_aux=_ROOTED_DEPTH,
        soil_extractable_water=_EXTR,
    )


def test_irrigation_leg_is_rate_times_area() -> None:
    # 5 mm/day over 2 m² ⇒ 10 kg/day into soil_water — the CAPACITY limb, which binds
    # here because the 1.0 m root zone over 2 m² holds 260 kg and only has 15.

    state = _state(soil_water0=15.0)
    result = _irrigation_flow(2.0).evaluate(
        state, _env(state, 1.0, irrigation=5.0), 1.0
    )
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_SOIL_WATER], 10.0, rel_tol=1e-12)
    assert math.isclose(legs[_WATER_SOURCE], -10.0, rel_tol=1e-12)


def test_irrigation_is_water_balanced() -> None:
    state = _state(soil_water0=15.0)
    result = _irrigation_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


# --- config boundary: load_transpiration_params -----------------------------
def test_transpiration_params_file_exists() -> None:
    assert TRANSPIRATION_PARAMS_PATH.is_file()


def test_load_transpiration_params_matches_committed_values() -> None:
    p = load_transpiration_params()
    assert isinstance(p, TranspirationParams)
    assert (p.aerodynamic_resistance, p.surface_resistance) == (_RA, _RS)


def _valid_transp() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "transpiration",
        "parameters": {
            "aerodynamic_resistance": {"value": 50.0, "unit": "s/m", "source": "[A]"},
            "surface_resistance": {"value": 70.0, "unit": "s/m", "source": "[A]"},
        },
    }


def _write_transp(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "transpiration.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_transp_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_transpiration_params(_write_transp(tmp_path, _valid_transp()))
    assert p.aerodynamic_resistance == 50.0
    assert p.surface_resistance == 70.0


def test_transp_loader_rejects_a_wrong_unit(tmp_path: Path) -> None:
    data = _valid_transp()
    data["parameters"]["aerodynamic_resistance"]["unit"] = "min/m"  # wrong scale
    with pytest.raises(ValueError, match="aerodynamic_resistance"):
        load_transpiration_params(_write_transp(tmp_path, data))


@pytest.mark.parametrize("field", ["aerodynamic_resistance", "surface_resistance"])
def test_transp_loader_rejects_non_positive(tmp_path: Path, field: str) -> None:
    data = _valid_transp()
    data["parameters"][field]["value"] = 0.0
    with pytest.raises(ValueError, match=field):
        load_transpiration_params(_write_transp(tmp_path, data))


def test_transp_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_transp()
    del data["parameters"]["surface_resistance"]["source"]
    with pytest.raises(ValidationError):
        load_transpiration_params(_write_transp(tmp_path, data))


def test_transp_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_transp()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "s/m", "source": "x"}
    with pytest.raises(ValidationError):
        load_transpiration_params(_write_transp(tmp_path, data))


def test_irrigation_stops_at_the_drained_upper_limit() -> None:
    """[F] Eqn 14.8's limb: ``IRGW = TTSW − ATSW``, so a full zone takes nothing.

    ⚠ This is the behaviour that makes ``Drainage`` inert on the frozen roster, and it
    is why "water non-limiting" is now a checkable claim rather than a label: the supply
    tracks the deficit instead of pushing a flat rate into a bucket with a bottom.
    """
    full = transpirable_capacity(_DEPTH, soil_extractable_water=_EXTR, ground_area=2.0)
    state = _state(soil_water0=full)
    result = _irrigation_flow(2.0).evaluate(
        state, _env(state, 1.0, irrigation=5.0), 1.0
    )
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_SOIL_WATER] == 0.0
    assert legs[_WATER_SOURCE] == 0.0
    # And an OVER-full zone does not run the supply backwards.
    over = _state(soil_water0=full * 2.0)
    legs_over = {
        leg.stock: leg.amount
        for leg in _irrigation_flow(2.0)
        .evaluate(over, _env(over, 1.0, irrigation=5.0), 1.0)
        .legs
    }
    assert legs_over[_SOIL_WATER] == 0.0


def test_irrigation_takes_the_smaller_of_capacity_and_deficit() -> None:
    """Both limbs, at the crossover, so neither can be dropped without a red test."""
    full = transpirable_capacity(_DEPTH, soil_extractable_water=_EXTR, ground_area=1.0)
    # deficit 4 kg, capacity 5 kg ⇒ the DEFICIT binds
    state = _state(soil_water0=full - 4.0)
    legs = {
        leg.stock: leg.amount
        for leg in _irrigation_flow(1.0)
        .evaluate(state, _env(state, 1.0, irrigation=5.0), 1.0)
        .legs
    }
    assert math.isclose(legs[_SOIL_WATER], 4.0, rel_tol=1e-12)
    # deficit 9 kg, capacity 5 kg ⇒ the CAPACITY binds
    state = _state(soil_water0=full - 9.0)
    legs = {
        leg.stock: leg.amount
        for leg in _irrigation_flow(1.0)
        .evaluate(state, _env(state, 1.0, irrigation=5.0), 1.0)
        .legs
    }
    assert math.isclose(legs[_SOIL_WATER], 5.0, rel_tol=1e-12)


def test_a_zero_capacity_is_still_a_hard_off() -> None:
    """DROUGHT cuts irrigation by forcing the capacity to zero over a window, and that
    mechanism had to survive the rate → capacity reinterpretation unchanged."""
    state = _state(soil_water0=1.0)  # a deep deficit, so only the zero can stop it
    legs = {
        leg.stock: leg.amount
        for leg in _irrigation_flow(1.0)
        .evaluate(state, _env(state, 1.0, irrigation=0.0), 1.0)
        .legs
    }
    assert legs[_SOIL_WATER] == 0.0
