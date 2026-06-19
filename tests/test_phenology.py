"""Phase-1 Step-8 tests: thermal-time phenology (the first non-conserved aux process).

The first real consumer of the Step-2 auxiliary channel (P2). Layers (mirroring
Steps 5/6/7):

* **Rate laws** (``domains.biosphere.phenology``, pure stdlib): the cardinal-cap
  growing-degree-day rate ``daily_thermal_time`` against hand-computed literals (below
  base → 0; mid-band linear; at/above cap → ``t_cap − t_base``), its monotonicity and
  band guard; the two-phase development stage ``development_stage`` at its cardinal
  points (DVS 0/1/2), linear midpoints, the 2.0 cap, and the positive-sum guard.
* **The aux process** ``ThermalTimeAccumulation``: it returns the increment-form
  ``{thermal_time: daily_thermal_time(T)·dt}``, reads temperature through ``env.get``
  (#16), and satisfies the ``AuxProcess`` protocol; integrated through the
  ``EulerIntegrator`` a constant-temperature season accumulates to ``rate·n·dt`` (the
  ``test_aux`` precedent — aux advances once per step at the step-entry snapshot).
* **Config boundary** (``load_phenology_params``): the committed file loads to the
  expected params; bad units / out-of-range values / a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.loader import PHENOLOGY_PARAMS_PATH, load_phenology_params
from domains.biosphere.phenology import (
    PhenologyParams,
    ThermalTimeAccumulation,
    daily_thermal_time,
    development_stage,
)
from simcore.auxiliary import AuxId, AuxProcess
from simcore.environment import SourceResolver, constant
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.registry import Registry
from simcore.state import State

# The committed winter-wheat provisional placeholders (mirror phenology.yaml).
_T_BASE, _T_CAP = 0.0, 30.0
_TSUM_ANTHESIS, _TSUM_MATURITY = 1100.0, 750.0


def _params() -> PhenologyParams:
    return PhenologyParams(
        t_base=_T_BASE,
        t_cap=_T_CAP,
        tsum_anthesis=_TSUM_ANTHESIS,
        tsum_maturity=_TSUM_MATURITY,
    )


# --- daily_thermal_time: the cardinal-cap GDD rate --------------------------
# Hand-computed literals with t_base=5, t_cap=25 (band = 20 °C): below/at base → 0;
# mid-band → T − t_base; at/above cap → t_cap − t_base = 20.
@pytest.mark.parametrize(
    ("temp", "expected"),
    [
        (-3.0, 0.0),  # below base
        (5.0, 0.0),  # at base (boundary → 0)
        (12.0, 7.0),  # mid-band: 12 − 5
        (15.0, 10.0),  # mid-band: 15 − 5
        (25.0, 20.0),  # at cap → t_cap − t_base
        (33.0, 20.0),  # above cap → plateau
    ],
)
def test_daily_thermal_time_cardinal_values(temp: float, expected: float) -> None:
    assert math.isclose(
        daily_thermal_time(temp, t_base=5.0, t_cap=25.0), expected, rel_tol=1e-12
    )


def test_daily_thermal_time_is_monotone_nondecreasing() -> None:
    temps = [-10.0, 0.0, 5.0, 5.0001, 15.0, 24.999, 25.0, 50.0]
    rates = [daily_thermal_time(t, t_base=5.0, t_cap=25.0) for t in temps]
    assert rates == sorted(rates)


def test_daily_thermal_time_is_bounded_by_the_band() -> None:
    # The plateau is exactly the band width; nothing exceeds it however hot.
    for t in (25.0, 100.0, 1e6):
        assert daily_thermal_time(t, t_base=5.0, t_cap=25.0) == 20.0


def test_daily_thermal_time_rejects_inverted_band() -> None:
    with pytest.raises(ValueError, match="t_base < t_cap"):
        daily_thermal_time(20.0, t_base=25.0, t_cap=5.0)


# --- development_stage: the two-phase TSUM ramp -----------------------------
# Round sums (TSUM1=1000, TSUM2=500) for clean literals. Cardinal points: DVS 0 at
# emergence, 0.5 at half of TSUM1, 1 at anthesis, 1.5 halfway through TSUM2, 2 at
# maturity, and a 2.0 cap beyond.
@pytest.mark.parametrize(
    ("thermal_time", "expected"),
    [
        (-50.0, 0.0),  # before emergence (clamped)
        (0.0, 0.0),  # emergence
        (500.0, 0.5),  # half the vegetative sum
        (1000.0, 1.0),  # anthesis (phase boundary)
        (1250.0, 1.5),  # halfway through the reproductive sum
        (1500.0, 2.0),  # maturity
        (3000.0, 2.0),  # past maturity → capped
    ],
)
def test_development_stage_cardinal_values(
    thermal_time: float, expected: float
) -> None:
    assert math.isclose(
        development_stage(thermal_time, tsum_anthesis=1000.0, tsum_maturity=500.0),
        expected,
        rel_tol=1e-12,
    )


def test_development_stage_is_monotone_nondecreasing() -> None:
    tts = [0.0, 250.0, 1000.0, 1100.0, 1500.0, 5000.0]
    dvs = [
        development_stage(tt, tsum_anthesis=1000.0, tsum_maturity=500.0) for tt in tts
    ]
    assert dvs == sorted(dvs)


@pytest.mark.parametrize("field", ["tsum_anthesis", "tsum_maturity"])
def test_development_stage_rejects_non_positive_sums(field: str) -> None:
    kwargs = {"tsum_anthesis": 1000.0, "tsum_maturity": 500.0}
    kwargs[field] = 0.0
    with pytest.raises(ValueError, match=field):
        development_stage(500.0, **kwargs)  # type: ignore[arg-type]


# --- ThermalTimeAccumulation: the aux process -------------------------------
def _aux_process() -> ThermalTimeAccumulation:
    return ThermalTimeAccumulation(
        id=AuxId("biosphere.thermal_time"),
        accumulator="thermal_time",
        temp_var="temp",
        params=_params(),
    )


def test_aux_process_satisfies_protocol() -> None:
    assert isinstance(_aux_process(), AuxProcess)


def test_aux_process_returns_increment_form() -> None:
    # The increment is daily_thermal_time(T)·dt (increment form, like a Flow's dt·rate).
    proc = _aux_process()
    resolver = SourceResolver(forcings={"temp": constant(18.0)})
    env = resolver.bind(State(n=0, stocks={}, rng_seed=0), 0.5)
    increment = proc.evaluate(State(n=0, stocks={}, rng_seed=0), env, 0.5)
    expected = daily_thermal_time(18.0, t_base=_T_BASE, t_cap=_T_CAP) * 0.5
    assert dict(increment) == {"thermal_time": expected}
    assert math.isclose(expected, 9.0, rel_tol=1e-12)  # (18 − 0) · 0.5


def test_aux_process_reads_temperature_via_env() -> None:
    # Cold below base → no thermal time; the process resolves forcing through env (#16).
    proc = _aux_process()
    resolver = SourceResolver(forcings={"temp": constant(-4.0)})
    env = resolver.bind(State(n=0, stocks={}, rng_seed=0), 1.0)
    assert proc.evaluate(State(n=0, stocks={}, rng_seed=0), env, 1.0) == {
        "thermal_time": 0.0
    }


@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_constant_temperature_season_accumulates_to_rate_times_n_dt(
    integrator_cls: type,
) -> None:
    # Integrated over a constant-temperature season the accumulator reaches
    # daily_thermal_time(T)·n·dt — aux advances once per step at the step-entry
    # snapshot under both schemes (the test_aux precedent, now with real phenology).
    temp, dt, steps = 12.0, 0.5, 6
    reg = Registry(flows=[], stocks={}, aux_processes=[_aux_process()])
    integ = integrator_cls(reg)
    resolver = SourceResolver(forcings={"temp": constant(temp)})
    state = State(n=0, stocks={}, rng_seed=0)
    for _ in range(steps):
        state = integ.step(state, resolver, dt)
    rate = daily_thermal_time(temp, t_base=_T_BASE, t_cap=_T_CAP)
    assert state.n == steps
    assert state.aux["thermal_time"] == rate * dt * steps
    assert math.isclose(rate * dt * steps, 36.0, rel_tol=1e-12)  # 12 · 0.5 · 6


def test_derived_dvs_tracks_the_accumulator() -> None:
    # The headline deliverable composes: DVS = f(thermal_time) read off the accumulator
    # after a season (derived, not stored — the P2 lock). 36 °C·day ≪ TSUM1 ⇒ early
    # vegetative DVS = 36 / 1100.
    reg = Registry(flows=[], stocks={}, aux_processes=[_aux_process()])
    integ = EulerIntegrator(reg)
    resolver = SourceResolver(forcings={"temp": constant(12.0)})
    state = State(n=0, stocks={}, rng_seed=0)
    for _ in range(6):
        state = integ.step(state, resolver, 0.5)
    dvs = development_stage(
        state.aux["thermal_time"],
        tsum_anthesis=_TSUM_ANTHESIS,
        tsum_maturity=_TSUM_MATURITY,
    )
    assert math.isclose(dvs, 36.0 / _TSUM_ANTHESIS, rel_tol=1e-12)


# --- config boundary: load_phenology_params ---------------------------------
def test_phenology_params_file_exists() -> None:
    assert PHENOLOGY_PARAMS_PATH.is_file()


def test_load_phenology_params_matches_committed_values() -> None:
    p = load_phenology_params()
    assert isinstance(p, PhenologyParams)
    assert (p.t_base, p.t_cap, p.tsum_anthesis, p.tsum_maturity) == (
        _T_BASE,
        _T_CAP,
        _TSUM_ANTHESIS,
        _TSUM_MATURITY,
    )


def _valid_phen() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "phenology",
        "parameters": {
            "t_base": {"value": 0.0, "unit": "degC", "source": "[A]"},
            "t_cap": {"value": 30.0, "unit": "degC", "source": "[A]"},
            "tsum_anthesis": {"value": 1100.0, "unit": "degC*day", "source": "[B]"},
            "tsum_maturity": {"value": 750.0, "unit": "degC*day", "source": "[B]"},
        },
    }


def _write_phen(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "phenology.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_phen_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_phenology_params(_write_phen(tmp_path, _valid_phen()))
    assert (p.t_base, p.t_cap, p.tsum_anthesis, p.tsum_maturity) == (
        0.0,
        30.0,
        1100.0,
        750.0,
    )


@pytest.mark.parametrize(
    ("field", "bad_unit"),
    [
        ("t_base", "K"),  # wrong temperature unit
        ("tsum_anthesis", "degC"),  # a sum is not a bare temperature
    ],
)
def test_phen_loader_rejects_a_wrong_unit(
    tmp_path: Path, field: str, bad_unit: str
) -> None:
    data = _valid_phen()
    data["parameters"][field]["unit"] = bad_unit
    with pytest.raises(ValueError, match=field):
        load_phenology_params(_write_phen(tmp_path, data))


def test_phen_loader_rejects_inverted_cardinal_band(tmp_path: Path) -> None:
    data = _valid_phen()
    data["parameters"]["t_base"]["value"] = 40.0  # above t_cap
    with pytest.raises(ValueError, match="t_base < t_cap"):
        load_phenology_params(_write_phen(tmp_path, data))


@pytest.mark.parametrize("field", ["tsum_anthesis", "tsum_maturity"])
def test_phen_loader_rejects_non_positive_sum(tmp_path: Path, field: str) -> None:
    data = _valid_phen()
    data["parameters"][field]["value"] = 0.0
    with pytest.raises(ValueError, match=field):
        load_phenology_params(_write_phen(tmp_path, data))


def test_phen_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_phen()
    del data["parameters"]["tsum_maturity"]["source"]
    with pytest.raises(ValidationError):
        load_phenology_params(_write_phen(tmp_path, data))


def test_phen_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_phen()
    data["parameters"]["bogus"] = {"value": 1.0, "unit": "degC", "source": "x"}
    with pytest.raises(ValidationError):
        load_phenology_params(_write_phen(tmp_path, data))
