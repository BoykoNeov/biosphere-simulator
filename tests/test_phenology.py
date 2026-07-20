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

from domains.biosphere.loader import (
    PHENOLOGY_PARAMS_PATH,
    load_phenology_params,
    load_photoperiod_params,
    load_vernalization_params,
)
from domains.biosphere.phenology import (
    PhenologyParams,
    PhotoperiodParams,
    ThermalTimeAccumulation,
    VernalizationAccumulation,
    VernalizationParams,
    daily_thermal_time,
    development_stage,
    photoperiod_factor,
    vernalization_day,
    vernalization_factor,
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
            # Vernalization + photoperiod (post-roadmap scope (B) increment 1): the
            # schema grew, so the fixture grows with it. Values mirror the committed
            # file ([C] Soltani & Sinclair Ch. 7/8, "Winter Europe").
            "t_base_v": {"value": -1.0, "unit": "degC", "source": "[C]"},
            "t_opt_lower_v": {"value": 0.0, "unit": "degC", "source": "[C]"},
            "t_opt_upper_v": {"value": 8.0, "unit": "degC", "source": "[C]"},
            "t_ceiling_v": {"value": 12.0, "unit": "degC", "source": "[C]"},
            "vsen": {"value": 0.033, "unit": "1/day", "source": "[C]"},
            "vdsat": {"value": 50.0, "unit": "day", "source": "[C]"},
            "cpp": {"value": 16.0, "unit": "h", "source": "[C]"},
            "ppsen": {"value": 0.09, "unit": "1/h", "source": "[C]"},
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


# =============================================================================
# Vernalization + photoperiod (post-roadmap bucket-3 scope (B), increment 1)
# =============================================================================
# Clean-room from Soltani & Sinclair (2012), "Modeling Physiology of Crop Development,
# Growth and Yield", CABI. Ch. 8 Eqn 8.3/8.6 (vernalization) and Ch. 7 Eqn 7.6
# (photoperiod, long-day form). See docs/plans/post-roadmap-oracle-match.md.

# The committed winter-wheat values (mirror phenology.yaml: Fig. 8.1 / Table 8.1 /
# Table 7.2, all "Wheat / Winter Europe").
_VERN_CARDINALS = {
    "t_base_v": -1.0,
    "t_opt_lower_v": 0.0,
    "t_opt_upper_v": 8.0,
    "t_ceiling_v": 12.0,
}
_VSEN, _VDSAT = 0.033, 50.0
_CPP, _PPSEN = 16.0, 0.09


def _vern_params() -> VernalizationParams:
    return VernalizationParams(**_VERN_CARDINALS, vsen=_VSEN, vdsat=_VDSAT)


# --- vernalization_day: Eqn 8.3, the 3-segment cold response -----------------
@pytest.mark.parametrize(
    ("temp", "expected"),
    [
        (-5.0, 0.0),  # below base
        (-1.0, 0.0),  # AT base - the boundary is closed at 0
        (-0.5, 0.5),  # lower ramp midpoint
        (0.0, 1.0),  # lower optimum - full effect
        (4.0, 1.0),  # inside the optimum band
        (8.0, 1.0),  # upper optimum - still full effect
        (10.0, 0.5),  # upper ramp midpoint
        (12.0, 0.0),  # AT ceiling - the boundary is closed at 0
        (20.0, 0.0),  # above ceiling
    ],
)
def test_vernalization_day_matches_hand_computed_literals(
    temp: float, expected: float
) -> None:
    assert vernalization_day(temp, **_VERN_CARDINALS) == pytest.approx(
        expected, abs=1e-12
    )


def test_vernalization_day_reproduces_the_sources_own_worked_example() -> None:
    """The source's p.-91 example: 5 days at 7 C gives 5 VERDAY; 5 days at 10 C or
    -0.5 C gives only 2.5 VERDAY.

    This is the discriminating check on the TRANSCRIPTION. The source states an
    arithmetic consequence of Eqn 8.3, so reproducing it verifies the equation was read
    correctly rather than merely copied plausibly. (Scope (C) round 5: a quote check
    verifies characters; only arithmetic verifies numbers.)
    """
    assert 5 * vernalization_day(7.0, **_VERN_CARDINALS) == pytest.approx(5.0)
    assert 5 * vernalization_day(10.0, **_VERN_CARDINALS) == pytest.approx(2.5)
    assert 5 * vernalization_day(-0.5, **_VERN_CARDINALS) == pytest.approx(2.5)


def test_vernalization_day_is_bounded_and_unimodal() -> None:
    xs = [i * 0.25 - 8.0 for i in range(160)]
    vs = [vernalization_day(x, **_VERN_CARDINALS) for x in xs]
    assert all(0.0 <= v <= 1.0 for v in vs)
    # Unimodal: non-decreasing to the plateau, then non-increasing. No interior dip.
    peak = max(range(len(vs)), key=lambda i: vs[i])
    assert all(vs[i] <= vs[i + 1] + 1e-15 for i in range(peak))
    assert all(vs[i] >= vs[i + 1] - 1e-15 for i in range(peak, len(vs) - 1))


@pytest.mark.parametrize(
    ("bad", "field"),
    [
        ({"t_base_v": 0.0, "t_opt_lower_v": 0.0}, "t_base_v < t_opt_lower_v"),
        ({"t_opt_lower_v": 9.0}, "t_opt_lower_v <= t_opt_upper_v"),
        ({"t_ceiling_v": 8.0}, "t_opt_upper_v < t_ceiling_v"),
    ],
)
def test_vernalization_day_rejects_ill_ordered_cardinals(
    bad: dict[str, float], field: str
) -> None:
    kwargs = {**_VERN_CARDINALS, **bad}
    with pytest.raises(ValueError, match=field):
        vernalization_day(3.0, **kwargs)  # type: ignore[arg-type]


# --- vernalization_factor: Eqn 8.6, the saturation curve ---------------------
def test_vernalization_factor_is_qualitative_for_winter_europe_wheat() -> None:
    """vsen*VDSAT = 1.65 > 1, so the unclamped Eqn 8.6 is -0.65 at CUMVER = 0.

    The clamp is therefore load-bearing rather than defensive: this cultivar is
    QUALITATIVE in the source's terms (Fig. 8.2) - development is fully ARRESTED, not
    merely slowed, until ~19.7 vernalization days accrue. That is a property of the
    cited parameterization, not a modeling choice made here.
    """
    assert _VSEN * _VDSAT > 1.0
    assert vernalization_factor(0.0, vsen=_VSEN, vdsat=_VDSAT) == 0.0
    # Break-even: 1 - vsen*(VDSAT - c) = 0  =>  c = VDSAT - 1/vsen.
    breakeven = _VDSAT - 1.0 / _VSEN
    assert breakeven == pytest.approx(19.697, abs=1e-3)
    assert vernalization_factor(breakeven - 0.1, vsen=_VSEN, vdsat=_VDSAT) == 0.0
    assert vernalization_factor(breakeven + 0.1, vsen=_VSEN, vdsat=_VDSAT) > 0.0


def test_vernalization_factor_saturates_at_one_and_stays_there() -> None:
    assert vernalization_factor(_VDSAT, vsen=_VSEN, vdsat=_VDSAT) == 1.0
    assert vernalization_factor(_VDSAT + 500.0, vsen=_VSEN, vdsat=_VDSAT) == 1.0
    assert vernalization_factor(30.0, vsen=_VSEN, vdsat=_VDSAT) == pytest.approx(0.34)


def test_vernalization_factor_is_monotone_non_decreasing_in_cold() -> None:
    vs = [vernalization_factor(c * 0.5, vsen=_VSEN, vdsat=_VDSAT) for c in range(140)]
    assert all(vs[i] <= vs[i + 1] + 1e-15 for i in range(len(vs) - 1))
    assert all(0.0 <= v <= 1.0 for v in vs)


def test_a_quantitative_cultivar_never_reaches_the_clamp() -> None:
    """The other branch of Fig. 8.2: vsen*VDSAT < 1 => verfun > 0 with no cold."""
    assert vernalization_factor(0.0, vsen=0.003, vdsat=50.0) == pytest.approx(0.85)


@pytest.mark.parametrize(
    ("kwargs", "field"),
    [({"vdsat": 0.0}, "vdsat"), ({"vsen": -0.1}, "vsen")],
)
def test_vernalization_factor_rejects_bad_params(
    kwargs: dict[str, float], field: str
) -> None:
    full = {"vsen": _VSEN, "vdsat": _VDSAT, **kwargs}
    with pytest.raises(ValueError, match=field):
        vernalization_factor(10.0, **full)  # type: ignore[arg-type]


# --- photoperiod_factor: Eqn 7.6, the long-day response ----------------------
@pytest.mark.parametrize(
    ("daylength_h", "expected"),
    [
        (16.0, 1.0),  # AT the critical photoperiod - no slowdown
        (20.0, 1.0),  # above it - clamped at 1, never accelerates past
        (14.53, 1.0 - 0.09 * (16.0 - 14.53)),  # midsummer at lat 52
        (7.85, 1.0 - 0.09 * (16.0 - 7.85)),  # midwinter at lat 52
        (5.0, 1.0 - 0.09 * 11.0),  # deep short-day
        (0.0, 0.0),  # 1 - 0.09*16 = -0.44 => clamped to 0
    ],
)
def test_photoperiod_factor_matches_hand_computed_literals(
    daylength_h: float, expected: float
) -> None:
    assert photoperiod_factor(daylength_h, cpp=_CPP, ppsen=_PPSEN) == pytest.approx(
        expected, abs=1e-12
    )


def test_photoperiod_factor_clamps_at_zero_never_negative() -> None:
    """The source is explicit that a negative ppfun is replaced by zero, because
    phenological development is only a forward process and cannot be negative."""
    assert 1.0 - 0.2 * (16.0 - 0.0) < 0.0  # would go negative unclamped
    assert photoperiod_factor(0.0, cpp=16.0, ppsen=0.2) == 0.0


def test_photoperiod_factor_is_monotone_non_decreasing_in_daylength() -> None:
    vs = [photoperiod_factor(h * 0.25, cpp=_CPP, ppsen=_PPSEN) for h in range(97)]
    assert all(vs[i] <= vs[i + 1] + 1e-15 for i in range(len(vs) - 1))
    assert all(0.0 <= v <= 1.0 for v in vs)


@pytest.mark.parametrize(
    ("kwargs", "field"), [({"cpp": 0.0}, "cpp"), ({"ppsen": -0.1}, "ppsen")]
)
def test_photoperiod_factor_rejects_bad_params(
    kwargs: dict[str, float], field: str
) -> None:
    full = {"cpp": _CPP, "ppsen": _PPSEN, **kwargs}
    with pytest.raises(ValueError, match=field):
        photoperiod_factor(10.0, **full)  # type: ignore[arg-type]


# --- THE DISCRIMINATING PROPERTY: memory vs no memory ------------------------
def test_vernalization_has_memory_and_photoperiod_does_not() -> None:
    """The structural difference that settled which mechanism the oracle uses.

    Vernalization reads an ACCUMULATOR: once saturated it is pinned at 1 and cannot fall
    again, whatever the weather does afterwards. Photoperiod reads an INSTANTANEOUS
    driver: it rises and falls with the season. A trajectory whose development
    multiplier keeps climbing AFTER the cold requirement is met therefore cannot be
    vernalization-driven - which is exactly how the oracle's mechanism was identified
    (docs/plans/post-roadmap-oracle-match.md, "The check that stopped the ceremony").
    """
    # Vernalization: saturated stays saturated even as cold keeps accruing.
    assert vernalization_factor(_VDSAT, vsen=_VSEN, vdsat=_VDSAT) == 1.0
    assert vernalization_factor(_VDSAT * 3, vsen=_VSEN, vdsat=_VDSAT) == 1.0
    # Photoperiod: the same daylength always gives the same factor, and a later shorter
    # day drops it back down - no ratchet.
    long_day = photoperiod_factor(15.0, cpp=_CPP, ppsen=_PPSEN)
    short_day = photoperiod_factor(8.0, cpp=_CPP, ppsen=_PPSEN)
    assert short_day < long_day
    assert photoperiod_factor(15.0, cpp=_CPP, ppsen=_PPSEN) == long_day


# --- the aux processes -------------------------------------------------------
def _vern_aux() -> VernalizationAccumulation:
    return VernalizationAccumulation(
        id=AuxId("biosphere.vernalization_days"),
        accumulator="vernalization_days",
        temp_var="temp",
        params=_vern_params(),
    )


def test_vernalization_aux_satisfies_protocol() -> None:
    assert isinstance(_vern_aux(), AuxProcess)


def test_vernalization_aux_returns_increment_form() -> None:
    proc = _vern_aux()
    resolver = SourceResolver(forcings={"temp": constant(4.0)})
    blank = State(n=0, stocks={}, rng_seed=0)
    env = resolver.bind(blank, 0.5)
    # 4 C is inside the optimum band => VERDAY 1.0/day => increment 1.0*dt.
    assert dict(proc.evaluate(blank, env, 0.5)) == {"vernalization_days": 0.5}


def test_thermal_time_aux_without_modifiers_is_the_plain_rate() -> None:
    """The byte-for-byte guarantee: supplying neither modifier leaves the pre-existing
    degree-day behavior untouched, so a crop with no cold/daylength requirement is
    unchanged."""
    plain = ThermalTimeAccumulation(
        id=AuxId("biosphere.thermal_time"),
        accumulator="thermal_time",
        temp_var="temp",
        params=_params(),
    )
    resolver = SourceResolver(forcings={"temp": constant(18.0)})
    blank = State(n=0, stocks={}, rng_seed=0)
    assert dict(plain.evaluate(blank, resolver.bind(blank, 1.0), 1.0)) == {
        "thermal_time": 18.0
    }


def test_thermal_time_aux_applies_both_modifiers_multiplicatively() -> None:
    """Eqn 7.4's biological-day form BD = tempfun * ppfun, extended by Eqn 8.2's
    verfun: the modifiers MULTIPLY."""
    proc = ThermalTimeAccumulation(
        id=AuxId("biosphere.thermal_time"),
        accumulator="thermal_time",
        temp_var="temp",
        params=_params(),
        vernalization=_vern_params(),
        vernalization_accumulator="vernalization_days",
        photoperiod=PhotoperiodParams(cpp=_CPP, ppsen=_PPSEN),
        daylength_var="daylength_s",
    )
    # Vegetative (thermal_time 0 => DVS 0), fully vernalized, 10 h day.
    snap = State(
        n=0,
        stocks={},
        rng_seed=0,
        aux={"thermal_time": 0.0, "vernalization_days": 60.0},
    )
    resolver = SourceResolver(
        forcings={"temp": constant(18.0), "daylength_s": constant(10.0 * 3600.0)}
    )
    expected = 18.0 * 1.0 * (1.0 - _PPSEN * (_CPP - 10.0))
    got = proc.evaluate(snap, resolver.bind(snap, 1.0), 1.0)
    assert got["thermal_time"] == pytest.approx(expected)


def test_thermal_time_aux_arrests_completely_when_unvernalized() -> None:
    """The qualitative cultivar's arrest at the aux-process level: with no accumulated
    cold the factor is 0, so thermal time does not advance AT ALL despite warm
    weather."""
    proc = ThermalTimeAccumulation(
        id=AuxId("biosphere.thermal_time"),
        accumulator="thermal_time",
        temp_var="temp",
        params=_params(),
        vernalization=_vern_params(),
        vernalization_accumulator="vernalization_days",
    )
    snap = State(
        n=0, stocks={}, rng_seed=0, aux={"thermal_time": 0.0, "vernalization_days": 0.0}
    )
    resolver = SourceResolver(forcings={"temp": constant(18.0)})
    assert dict(proc.evaluate(snap, resolver.bind(snap, 1.0), 1.0)) == {
        "thermal_time": 0.0
    }


def test_thermal_time_modifiers_are_gated_off_at_and_after_anthesis() -> None:
    """Wheat is insensitive to both cold and daylength at/after anthesis, so past DVS 1
    the plain degree-day rate must be recovered EXACTLY - even with zero cold and a
    short day, which before anthesis would arrest development entirely."""
    proc = ThermalTimeAccumulation(
        id=AuxId("biosphere.thermal_time"),
        accumulator="thermal_time",
        temp_var="temp",
        params=_params(),
        vernalization=_vern_params(),
        vernalization_accumulator="vernalization_days",
        photoperiod=PhotoperiodParams(cpp=_CPP, ppsen=_PPSEN),
        daylength_var="daylength_s",
    )
    resolver = SourceResolver(
        forcings={"temp": constant(18.0), "daylength_s": constant(8.0 * 3600.0)}
    )
    # thermal_time == tsum_anthesis => DVS == 1.0 exactly (the boundary is closed).
    at_anthesis = State(
        n=0,
        stocks={},
        rng_seed=0,
        aux={"thermal_time": _TSUM_ANTHESIS, "vernalization_days": 0.0},
    )
    got = proc.evaluate(at_anthesis, resolver.bind(at_anthesis, 1.0), 1.0)
    assert dict(got) == {"thermal_time": 18.0}


def test_committed_file_loads_the_cited_winter_europe_values() -> None:
    """The committed params are the two "Winter Europe" rows - Table 8.1 and Table 7.2 -
    i.e. the SAME cultivar class in both chapters, not a mix of two."""
    vern = load_vernalization_params(PHENOLOGY_PARAMS_PATH)
    photo = load_photoperiod_params(PHENOLOGY_PARAMS_PATH)
    assert (vern.t_base_v, vern.t_opt_lower_v) == (-1.0, 0.0)
    assert (vern.t_opt_upper_v, vern.t_ceiling_v) == (8.0, 12.0)
    assert (vern.vsen, vern.vdsat) == (0.033, 50.0)
    assert (photo.cpp, photo.ppsen) == (16.0, 0.09)
