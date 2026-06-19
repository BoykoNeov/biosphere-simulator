"""Phase-1 Step-5 tests: FvCB photosynthesis (Farquhar, von Caemmerer & Berry 1980).

The first carbon source process. Three layers:

* **Leaf-level FvCB physics** (``domains.biosphere.photosynthesis``, pure stdlib): the
  Rubisco-limited, electron-transport, light-limited, and co-limited gross rates plus
  the cardinal-temperature factor, checked against **independent hand-computed**
  literals (not restatements of the implementation), the ``Ci ≤ Γ*`` → 0 clamp, the
  PAR → 0 limit, and monotonicity.
* **The provisional big-leaf canopy aggregator**: the LAI → 0 finiteness limit, the
  LAI = 0 zero, and a composed daily-flux literal.
* **The assembled ``GrossAssimilation`` flow**: a carbon-balanced ``FlowResult`` with
  the hand-computed daily mol-C leg, and the rate set by the ``Ci`` forcing (not the
  CO₂ boundary amount).
* **Config boundary** (``load_photosynthesis_params``): the committed file loads to the
  expected params; bad units / out-of-range values / a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.canopy import CanopyParams
from domains.biosphere.loader import (
    PHOTOSYNTHESIS_PARAMS_PATH,
    load_photosynthesis_params,
)
from domains.biosphere.photosynthesis import (
    GrossAssimilation,
    PhotosynthesisParams,
    daily_canopy_assimilation,
    electron_transport_rate,
    gross_leaf_assimilation,
    light_limited_rate,
    rubisco_limited_rate,
    temperature_factor,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# The committed winter-wheat provisional placeholders (mirror photosynthesis.yaml).
# Held as a literal so the physics literals below are independent of the loader.
_VCMAX, _JMAX, _ALPHA, _THETA = 100.0, 180.0, 0.3, 0.7
_GAMMA_STAR, _KC, _KO, _O2 = 42.75, 404.9, 278.4, 210.0
_TMIN, _TOPT_LO, _TOPT_HI, _TMAX = 0.0, 15.0, 25.0, 35.0

# Canopy diagnostic params (from canopy.yaml; see test_canopy).
_SLA_PER_MOL_C = 0.5872044444444445
_K = 0.6


def _params() -> PhotosynthesisParams:
    return PhotosynthesisParams(
        vcmax=_VCMAX,
        jmax=_JMAX,
        quantum_yield=_ALPHA,
        theta=_THETA,
        gamma_star=_GAMMA_STAR,
        kc=_KC,
        ko=_KO,
        o2=_O2,
        t_min=_TMIN,
        t_opt_lo=_TOPT_LO,
        t_opt_hi=_TOPT_HI,
        t_max=_TMAX,
    )


def _canopy() -> CanopyParams:
    return CanopyParams(sla_per_mol_c=_SLA_PER_MOL_C, extinction_coef=_K)


# --- leaf-level FvCB: independent known values ------------------------------
def test_rubisco_limited_rate_known_value() -> None:
    # Ac = 100·(400 − 42.75) / (400 + 404.9·(1 + 210/278.4)); hand-computed literal.
    ac = rubisco_limited_rate(
        400.0, vcmax=_VCMAX, gamma_star=_GAMMA_STAR, kc=_KC, ko=_KO, o2=_O2
    )
    assert math.isclose(ac, 32.17540139669239, rel_tol=1e-12)


def test_electron_transport_rate_known_value() -> None:
    # Smaller root of θJ² − (I₂+Jmax)J + I₂·Jmax with I₂ = 0.3·500.
    j = electron_transport_rate(500.0, jmax=_JMAX, quantum_yield=_ALPHA, theta=_THETA)
    assert math.isclose(j, 105.36937435075244, rel_tol=1e-12)


def test_electron_transport_zero_par_is_zero() -> None:
    # No light, no electron transport: I₂ = 0 ⇒ smaller root is 0 exactly.
    assert (
        electron_transport_rate(0.0, jmax=_JMAX, quantum_yield=_ALPHA, theta=_THETA)
        == 0.0
    )


def test_electron_transport_saturates_to_jmax() -> None:
    # Saturating light drives J toward Jmax from below (never exceeding it).
    j = electron_transport_rate(1.0e6, jmax=_JMAX, quantum_yield=_ALPHA, theta=_THETA)
    assert j < _JMAX
    assert math.isclose(j, _JMAX, rel_tol=1e-3)


def test_light_limited_rate_known_value() -> None:
    j = electron_transport_rate(500.0, jmax=_JMAX, quantum_yield=_ALPHA, theta=_THETA)
    aj = light_limited_rate(400.0, j, gamma_star=_GAMMA_STAR)
    assert math.isclose(aj, 19.383732742948663, rel_tol=1e-12)


def test_gross_leaf_assimilation_is_the_co_limited_min() -> None:
    # At Ci=400, PAR=500: Aj (19.38) < Ac (32.18), so the leaf is light-limited.
    ag = gross_leaf_assimilation(400.0, 500.0, params=_params())
    assert math.isclose(ag, 19.383732742948663, rel_tol=1e-12)


@pytest.mark.parametrize("ci", [42.75, 30.0, 0.0])
def test_gross_leaf_assimilation_clamps_at_or_below_gamma_star(ci: float) -> None:
    # At Ci ≤ Γ* the (Ci − Γ*) factor is ≤ 0; gross uptake clamps to 0 so the *source*
    # flow never becomes a withdrawal from plant carbon (the load-bearing clamp).
    assert gross_leaf_assimilation(ci, 500.0, params=_params()) == 0.0


def test_gross_leaf_assimilation_zero_par_is_zero() -> None:
    # No light ⇒ Aj = 0 ⇒ min(Ac, Aj) = 0 (assimilation → 0 as light → 0, P3).
    assert gross_leaf_assimilation(400.0, 0.0, params=_params()) == 0.0


def test_gross_leaf_assimilation_monotone_increasing_in_par() -> None:
    p = _params()
    rates = [
        gross_leaf_assimilation(400.0, par, params=p) for par in (50, 100, 200, 400)
    ]
    assert all(b > a for a, b in zip(rates, rates[1:], strict=False))


# --- cardinal-temperature factor: independent literals ----------------------
@pytest.mark.parametrize(
    ("temp", "expected"),
    [
        (-5.0, 0.0),  # below t_min
        (0.0, 0.0),  # at t_min
        (7.5, 0.5),  # midpoint of the up-ramp [0, 15]
        (15.0, 1.0),  # start of the plateau
        (20.0, 1.0),  # within the plateau
        (25.0, 1.0),  # end of the plateau
        (30.0, 0.5),  # midpoint of the down-ramp [25, 35]
        (35.0, 0.0),  # at t_max
        (40.0, 0.0),  # above t_max
    ],
)
def test_temperature_factor_cardinal_points(temp: float, expected: float) -> None:
    f = temperature_factor(
        temp, t_min=_TMIN, t_opt_lo=_TOPT_LO, t_opt_hi=_TOPT_HI, t_max=_TMAX
    )
    assert math.isclose(f, expected, abs_tol=1e-12)


# --- the provisional big-leaf canopy aggregator -----------------------------
def test_daily_canopy_assimilation_known_value() -> None:
    # LAI=2.936 (5 mol leaf C / 1 m², SLA fold), f_int≈0.828; daily flux composed from
    # the hand literals above. dt is applied by the flow, not the aggregator.
    daily = daily_canopy_assimilation(
        800.0,
        5.0 * _SLA_PER_MOL_C / 1.0,  # LAI
        400.0,
        20.0,
        43200.0,
        params=_params(),
        canopy=_canopy(),
        ground_area=1.0,
    )
    assert math.isclose(daily, 1.3778614691309006, rel_tol=1e-12)


def test_daily_canopy_assimilation_temperature_halves_at_ramp_midpoint() -> None:
    # f_temp(7.5) = 0.5 scales the whole daily flux by exactly half.
    common: dict[str, Any] = dict(params=_params(), canopy=_canopy(), ground_area=1.0)
    lai = 5.0 * _SLA_PER_MOL_C
    warm = daily_canopy_assimilation(800.0, lai, 400.0, 20.0, 43200.0, **common)
    cool = daily_canopy_assimilation(800.0, lai, 400.0, 7.5, 43200.0, **common)
    assert math.isclose(cool, warm * 0.5, rel_tol=1e-12)


def test_daily_canopy_assimilation_zero_lai_is_zero() -> None:
    assert (
        daily_canopy_assimilation(
            800.0,
            0.0,
            400.0,
            20.0,
            43200.0,
            params=_params(),
            canopy=_canopy(),
            ground_area=1.0,
        )
        == 0.0
    )


def test_daily_canopy_assimilation_finite_in_small_lai_limit() -> None:
    # The mean-absorbed-PAR-per-leaf ratio f_int/LAI → k as LAI → 0 (f_int ≈ k·LAI),
    # so the flux stays finite (no 0/0 blow-up) and vanishes smoothly toward LAI=0.
    common: dict[str, Any] = dict(params=_params(), canopy=_canopy(), ground_area=1.0)
    flux = [
        daily_canopy_assimilation(800.0, lai, 400.0, 20.0, 43200.0, **common)
        for lai in (1e-3, 1e-6, 1e-9)
    ]
    assert all(math.isfinite(f) for f in flux)
    assert all(f > 0.0 for f in flux)
    assert flux[0] > flux[1] > flux[2]  # smaller LAI ⇒ less canopy ⇒ less assimilation


@pytest.mark.parametrize("bad_area", [0.0, -1.0])
def test_daily_canopy_assimilation_rejects_non_positive_ground_area(
    bad_area: float,
) -> None:
    with pytest.raises(ValueError, match="ground_area"):
        daily_canopy_assimilation(
            800.0,
            2.0,
            400.0,
            20.0,
            43200.0,
            params=_params(),
            canopy=_canopy(),
            ground_area=bad_area,
        )


@pytest.mark.parametrize("bad_daylength", [0.0, -3600.0])
def test_daily_canopy_assimilation_rejects_non_positive_daylength(
    bad_daylength: float,
) -> None:
    with pytest.raises(ValueError, match="daylength_s"):
        daily_canopy_assimilation(
            800.0,
            2.0,
            400.0,
            20.0,
            bad_daylength,
            params=_params(),
            canopy=_canopy(),
            ground_area=1.0,
        )


# --- the assembled GrossAssimilation flow -----------------------------------
_BIO = DomainId("biosphere")
_PLANT_C = StockId("biosphere.plant_c")
_CO2 = StockId("boundary.co2")


def _state(plant_c0: float, co2_0: float) -> State:
    carbon = canonical_unit(Quantity.CARBON)
    plant = Stock(
        id=_PLANT_C,
        domain=_BIO,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=plant_c0,
        kind=StockKind.POPULATION,
        extinction_threshold=0.0,
    )
    co2 = Stock(
        id=_CO2,
        domain=DomainId("boundary"),
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=co2_0,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )
    return State(n=0, stocks={_PLANT_C: plant, _CO2: co2}, rng_seed=0)


def _flow() -> GrossAssimilation:
    return GrossAssimilation(
        id=FlowId("biosphere.gross_assimilation"),
        priority=0,
        co2_source=_CO2,
        plant_c=_PLANT_C,
        par_var="par",
        ci_var="ci",
        temp_var="temp",
        daylength_var="daylength_s",
        params=_params(),
        canopy=_canopy(),
        ground_area=1.0,
    )


def _env(snapshot: State, dt: float):  # noqa: ANN202 - BoundEnvironment is internal
    resolver = SourceResolver(
        forcings={
            "par": constant(800.0),
            "ci": constant(400.0),
            "temp": constant(20.0),
            "daylength_s": constant(43200.0),
        }
    )
    return resolver.bind(snapshot, dt)


def test_gross_assimilation_leg_is_the_hand_computed_daily_flux() -> None:
    state = _state(plant_c0=5.0, co2_0=1.0e9)
    flow = _flow()
    result = flow.evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    # plant_c gains the gross daily flux (dt=1 day); the CO₂ boundary loses it.
    assert math.isclose(legs[_PLANT_C], 1.3778614691309006, rel_tol=1e-12)
    assert math.isclose(legs[_CO2], -1.3778614691309006, rel_tol=1e-12)


def test_gross_assimilation_is_carbon_balanced() -> None:
    state = _state(plant_c0=5.0, co2_0=1.0e9)
    flow = _flow()
    result = flow.evaluate(state, _env(state, 1.0), 1.0)
    # One withdrawal, one equal deposit, both CARBON ⇒ the gate passes (P1).
    assert_flow_balanced(result, state.stocks)


def test_gross_assimilation_rate_is_set_by_ci_not_co2_boundary_amount() -> None:
    # Unlike the demo's Photosynthesis (rate ∝ withdrawn amount), the FvCB rate is the
    # Ci forcing — doubling the CO₂ boundary reservoir leaves the flux unchanged.
    flow = _flow()
    small = flow.evaluate(s := _state(5.0, 1.0e6), _env(s, 1.0), 1.0)
    large = flow.evaluate(s2 := _state(5.0, 2.0e6), _env(s2, 1.0), 1.0)
    small_plant = next(leg.amount for leg in small.legs if leg.stock == _PLANT_C)
    large_plant = next(leg.amount for leg in large.legs if leg.stock == _PLANT_C)
    assert small_plant == large_plant


def test_gross_assimilation_scales_linearly_with_dt() -> None:
    # Increment form: the leg is dt · (daily rate). (The aggregator is itself dt-
    # nonlinear via the day-length integration, P3 — but the dt factor here is linear.)
    state = _state(5.0, 1.0e9)
    flow = _flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _PLANT_C
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _PLANT_C
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- config boundary: load_photosynthesis_params ----------------------------
def test_photosynthesis_params_file_exists() -> None:
    assert PHOTOSYNTHESIS_PARAMS_PATH.is_file()


def test_load_photosynthesis_params_matches_committed_values() -> None:
    p = load_photosynthesis_params()
    assert isinstance(p, PhotosynthesisParams)
    assert p.vcmax == _VCMAX
    assert p.jmax == _JMAX
    assert p.quantum_yield == _ALPHA
    assert p.theta == _THETA
    assert p.gamma_star == _GAMMA_STAR
    assert p.kc == _KC
    assert p.ko == _KO
    assert p.o2 == _O2
    assert (p.t_min, p.t_opt_lo, p.t_opt_hi, p.t_max) == (
        _TMIN,
        _TOPT_LO,
        _TOPT_HI,
        _TMAX,
    )


def _valid_photo() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "fvcb_photosynthesis",
        "parameters": {
            "vcmax": {"value": 100.0, "unit": "umol/m^2/s", "source": "[A]"},
            "jmax": {"value": 180.0, "unit": "umol/m^2/s", "source": "[A]"},
            "quantum_yield": {"value": 0.3, "unit": "mol/mol", "source": "[A]"},
            "theta": {"value": 0.7, "unit": "dimensionless", "source": "[A]"},
            "gamma_star": {"value": 42.75, "unit": "umol/mol", "source": "[A]"},
            "kc": {"value": 404.9, "unit": "umol/mol", "source": "[A]"},
            "ko": {"value": 278.4, "unit": "mmol/mol", "source": "[A]"},
            "o2": {"value": 210.0, "unit": "mmol/mol", "source": "[A]"},
            "t_min": {"value": 0.0, "unit": "degC", "source": "[B]"},
            "t_opt_lo": {"value": 15.0, "unit": "degC", "source": "[B]"},
            "t_opt_hi": {"value": 25.0, "unit": "degC", "source": "[B]"},
            "t_max": {"value": 35.0, "unit": "degC", "source": "[B]"},
        },
    }


def _write_photo(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "photosynthesis.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_photo_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_photosynthesis_params(_write_photo(tmp_path, _valid_photo()))
    assert p.vcmax == 100.0
    assert p.t_max == 35.0


def test_photo_loader_rejects_a_wrong_unit(tmp_path: Path) -> None:
    data = _valid_photo()
    data["parameters"]["vcmax"]["unit"] = "mol/m^2/s"  # right dimension, wrong scale
    with pytest.raises(ValueError, match="vcmax"):
        load_photosynthesis_params(_write_photo(tmp_path, data))


@pytest.mark.parametrize("field", ["vcmax", "jmax", "gamma_star", "kc", "ko", "o2"])
def test_photo_loader_rejects_non_positive_kinetics(tmp_path: Path, field: str) -> None:
    data = _valid_photo()
    data["parameters"][field]["value"] = 0.0
    with pytest.raises(ValueError, match=field):
        load_photosynthesis_params(_write_photo(tmp_path, data))


@pytest.mark.parametrize("field", ["quantum_yield", "theta"])
@pytest.mark.parametrize("bad", [0.0, -0.1, 1.5])
def test_photo_loader_rejects_out_of_unit_interval(
    tmp_path: Path, field: str, bad: float
) -> None:
    data = _valid_photo()
    data["parameters"][field]["value"] = bad
    with pytest.raises(ValueError, match=field):
        load_photosynthesis_params(_write_photo(tmp_path, data))


@pytest.mark.parametrize(
    "cardinals",
    [
        (15.0, 15.0, 25.0, 35.0),  # t_min == t_opt_lo (up-ramp denominator 0)
        (0.0, 25.0, 15.0, 35.0),  # t_opt_lo > t_opt_hi (inverted plateau)
        (0.0, 15.0, 35.0, 35.0),  # t_opt_hi == t_max (down-ramp denominator 0)
        (30.0, 15.0, 25.0, 35.0),  # t_min > t_opt_lo
    ],
)
def test_photo_loader_rejects_bad_cardinal_temperatures(
    tmp_path: Path, cardinals: tuple[float, float, float, float]
) -> None:
    data = _valid_photo()
    for field, value in zip(
        ("t_min", "t_opt_lo", "t_opt_hi", "t_max"), cardinals, strict=True
    ):
        data["parameters"][field]["value"] = value
    with pytest.raises(ValueError, match="cardinal temperatures"):
        load_photosynthesis_params(_write_photo(tmp_path, data))


def test_photo_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_photo()
    del data["parameters"]["vcmax"]["source"]
    with pytest.raises(ValidationError):
        load_photosynthesis_params(_write_photo(tmp_path, data))


def test_photo_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_photo()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "dimensionless", "source": "x"}
    with pytest.raises(ValidationError):
        load_photosynthesis_params(_write_photo(tmp_path, data))
