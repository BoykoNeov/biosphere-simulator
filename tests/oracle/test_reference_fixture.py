"""Always-run checks on the committed winter-wheat reference fixture (no PCSE).

These guard that the committed artifact stays well-formed and physically plausible (a
corruption/regression catch), and they exercise ``lab/oracle_match`` against real
oracle data. PCSE is **not** imported here — only the committed JSON is read — so this
runs on every plain ``uv run pytest``.
"""

from lab.oracle_match import nrmse, within_band

from .runner import OUTPUT_VARIABLES, column, load_fixture


def test_fixture_loads_and_is_well_formed() -> None:
    fixture = load_fixture()
    assert set(fixture) == {"provenance", "trajectory"}
    traj = fixture["trajectory"]
    assert len(traj) == fixture["provenance"]["n_days"] > 100
    # Every row carries the day plus every declared output variable.
    for row in traj:
        assert "day" in row
        assert set(OUTPUT_VARIABLES) <= set(row)


def test_provenance_records_clean_lineage() -> None:
    prov = load_fixture()["provenance"]
    # The licensing-critical fields: this is OUTPUT, sourced from the (unlicensed,
    # uncommitted) param repo, run by a recorded PCSE version.
    assert prov["pcse_version"]
    assert "WOFOST_crop_parameters" in prov["crop_parameter_source"]
    assert prov["crop_name"] == "wheat"
    assert prov["variety_name"] == "Winter_wheat_101"


def test_winter_wheat_trajectory_is_physically_plausible() -> None:
    fixture = load_fixture()
    dvs = column(fixture, "DVS")
    lai = column(fixture, "LAI")
    tagp = column(fixture, "TAGP")
    twso = column(fixture, "TWSO")

    # Development runs from emergence toward maturity (DVS reaches ~2.0).
    assert dvs[0] < 0.5
    assert max(dvs) >= 2.0
    assert dvs == sorted(dvs), "DVS must be monotonic non-decreasing"

    # Canopy rises to a realistic peak LAI then senesces back toward zero.
    assert 3.0 <= max(lai) <= 10.0
    assert lai[-1] < 1.0

    # Biomass accumulates; final above-ground and yield are realistic for winter
    # wheat (order 10^4 kg/ha).
    assert tagp[-1] > tagp[0]
    assert 1.0e4 <= tagp[-1] <= 4.0e4
    assert 5.0e3 <= twso[-1] <= 2.0e4


def test_helper_consumes_real_trajectory() -> None:
    # The behavioral-match helper applied to genuine oracle data: identical compares
    # to zero discrepancy, a tiny perturbation stays within a 5% band, and a gross
    # 30% inflation lands well outside it.
    fixture = load_fixture()
    tagp = column(fixture, "TAGP")
    assert nrmse(tagp, tagp) == 0.0
    assert within_band(tagp, [v * 1.02 for v in tagp], tol=0.05) is True
    assert within_band(tagp, [v * 1.30 for v in tagp], tol=0.05) is False
