"""Always-run checks on the committed LINTUL3 spring-wheat fixtures (no PCSE).

Guards that the committed oracle + weather artifacts stay well-formed and physically
plausible (a corruption/regression catch). PCSE is **not** imported — only the committed
JSON is read — so this runs on every plain ``uv run pytest``. Plan of record:
``docs/plans/post-roadmap-day-neutral-crop.md``.
"""

from .lintul3_runner import (
    OUTPUT_VARIABLES,
    WEATHER_VARIABLES,
    column,
    load_fixture,
    load_weather,
)


def test_reference_fixture_loads_and_is_well_formed() -> None:
    fixture = load_fixture()
    assert set(fixture) == {"provenance", "trajectory"}
    traj = fixture["trajectory"]
    assert len(traj) == fixture["provenance"]["n_days"] > 100
    for row in traj:
        assert "day" in row
        assert set(OUTPUT_VARIABLES) <= set(row)


def test_provenance_records_clean_offline_lineage() -> None:
    prov = load_fixture()["provenance"]
    assert prov["pcse_version"]
    assert prov["model"] == "LINTUL3"
    assert prov["crop_name"] == "wheat"
    assert prov["variety_name"] == "spring-wheat"
    # Offline, bundled: the weather source is CABO (ships with PCSE), NOT NASAPower.
    assert "CABO" in prov["weather_source"]


def test_spring_wheat_trajectory_is_physically_plausible() -> None:
    fixture = load_fixture()
    # Pre-emergence cells are null; the crop life is the non-null tail.
    dvs = [v for v in column(fixture, "DVS") if v is not None]
    lai = [v for v in column(fixture, "LAI") if v is not None]
    wso = [v for v in column(fixture, "WSO") if v is not None]

    # Development runs emergence → maturity (DVS reaches ~2.0), monotone non-decreasing.
    assert dvs[0] < 0.1
    assert max(dvs) >= 2.0
    assert dvs == sorted(dvs), "DVS must be monotone non-decreasing"

    # Canopy rises to a realistic peak LAI then senesces back toward zero.
    assert 3.0 <= max(lai) <= 10.0
    assert lai[-1] < 1.0

    # Grain fills to a realistic spring-wheat yield (order 10^2–10^3 g m⁻²).
    assert max(wso) > 0.0
    assert 3.0e2 <= max(wso) <= 2.0e3


def test_weather_fixture_is_well_formed_and_maps_our_schema() -> None:
    weather = load_weather()
    assert set(weather) == {"provenance", "weather"}
    rows = weather["weather"]
    assert len(rows) == weather["provenance"]["n_days"] > 100
    for row in rows:
        assert set(("day", *WEATHER_VARIABLES)) <= set(row)
    # Spring/summer at NL: daily-mean temperatures span a sane, mostly-warm band.
    temps = [row["TEMP"] for row in rows]
    assert -5.0 < min(temps) < 15.0
    assert 15.0 < max(temps) < 30.0
