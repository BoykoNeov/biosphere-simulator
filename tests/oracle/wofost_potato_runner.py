"""Offline WOFOST **potato** runner — produces the committed potato oracle fixture.

Post-roadmap: the potato crop — the biosphere's first genuine *second species*. Plan of
record: ``docs/plans/post-roadmap-potato-crop.md``.

Drives PCSE's **WOFOST 7.2** in **potential production** (``pp``) mode for potato under
its *bundled demo database*, and captures a reference **trajectory** (development,
canopy,
organ biomass, water use) for the diagnostic comparison against our clean-room potato.

WHY THIS ORACLE EXISTS AT ALL — the fact that decided the whole exercise
The day-neutral crop's plan recorded that only ``lintul3_springwheat`` shipped offline:
the winter-wheat WOFOST oracle needed the **unlicensed** ``WOFOST_crop_parameters`` repo
**plus network**. On that reading, any *new species* was authored-only by construction.
**That reading was stale.** ``pcse/tests/test_data/pcse_dump.sql`` — PCSE's own bundled
demo database, loaded automatically at ``import pcse`` — ships **six** crops with
complete
parameter sets (winter wheat, grain maize, spring barley, **potato**, winter rapeseed,
sunflower), their crop calendars, two years of weather for grid 31031, site and soil,
AND
a ``wofost_unittest_benchmarks`` table of pre-computed daily reference trajectories.
``pcse.start_wofost`` is a first-class supported API over exactly that. So a *validated*
second species is possible fully offline — no network, no unlicensed cache.

LICENSING — identical discipline to ``lintul3_runner``, no relaxation
(``docs/reuse-and-licenses.md``):
  * PCSE is EUPL → running it as an oracle is *mere use*; its **output is facts**.
  * The inputs ship **with PCSE** as its own test data, covered by PCSE's own EUPL. The
    run is fully **offline** — no network, no unlicensed cache.
  * We commit **ONLY the output trajectory + a provenance record, NEVER a parameter
    value**. Copying ``crop_parameter_value`` rows out of the demo DB would be
    reverse-engineering PCSE and is **refused**: potato's own params are sourced
    independently from primary literature (Penning de Vries et al. 1989, read off page
    images) in ``params/crops/potato/``.

⚠ **The oracle is a DIAGNOSTIC, never a fit target** (ruling B,
``docs/plans/post-roadmap-oracle-match.md``). No potato value is ever moved to close a
gap
to WOFOST. WOFOST's assimilation is an **AMAX / light-response** formalism — a different
family from our FvCB core — so absolute biomass is *reported*, never a pass/fail.

THE INDEPENDENT CROSS-CHECK (new here, and worth the few lines)
The demo DB also ships ``wofost_unittest_benchmarks`` — PCSE's *own* expected daily
output for this exact (grid, crop, year, mode). :func:`benchmark_deltas` compares our
run against it, and the result is recorded in the fixture provenance. That is a genuine
independent check on the *runner*, not a second oracle: it catches "we drove the model
wrong" without adding any authority the model itself does not have.

Deliberately **not** named ``test_*`` so pytest never imports it at collection time on a
machine without ``pcse``. The ``oracle``-marked regeneration test imports it behind
``importorskip``. Regenerate the fixtures with::

    uv run --group oracle python -m tests.oracle.wofost_potato_runner
"""

from __future__ import annotations

import datetime as _dt
import json
import os
import sqlite3
from pathlib import Path
from typing import Any

# Fixtures live beside this module: the reference trajectory (PCSE OUTPUT) and the daily
# weather (grid-31031 facts, mapped to our {TEMP, IRRAD, VAP} schema) our potato is
# driven
# by — both license-clean (facts), neither containing crop-parameter values.
FIXTURE_PATH = Path(__file__).with_name("potato_reference.json")
WEATHER_FIXTURE_PATH = Path(__file__).with_name("potato_weather.json")

# --- forcing definition (recorded verbatim in the fixture provenance) ---------
# All from the bundled demo DB's ``crop_calendar``/``grid`` rows for this run: potato
# (crop_no 7), EMERGENCE-started, grid 31031 = lat 37.64 / lon -6.09 / alt 47 m
# (Andalusia, Spain), year 2000, POTENTIAL production (no water/nutrient limitation —
# our own PP plot's regime, so the comparison is like-for-like).
GRID_NO = 31031  # the demo DB's only fully-provisioned grid (weather + site + soil)
CROP_NO = 7  # 'POTATO' in the demo DB's ``crop`` table
YEAR = 2000
MODE = "pp"  # potential production
CROP_NAME = "potato"
LATITUDE = 37.64
LONGITUDE = -6.09
ALTITUDE = 47.0
EMERGENCE_DATE = _dt.date(2000, 2, 20)  # crop_calendar crop_start_date (emergence)
RUN_DAYS = 200  # generous; the run terminates at maturity (~day 96) well before this

# Weather fields we record, mapped to OUR resolver's schema (the spring-wheat
# fixture's):
# daily-mean TEMP (°C) = (TMIN+TMAX)/2; IRRAD (J m⁻² day⁻¹ shortwave); VAP (hPa vapour
# pressure). FACTS; the clean-room conversions to flow drivers live in
# src/domains/biosphere/weather.py.
WEATHER_VARIABLES = ("TEMP", "IRRAD", "VAP")

# Output variables captured per day (WOFOST state; kg ha⁻¹ for the weights, LAI [-],
# DVS [-], TRA cm day⁻¹, RD cm, SM [-] volumetric soil moisture).
OUTPUT_VARIABLES = (
    "DVS",  # development stage [-]
    "LAI",  # leaf area index [m2 m-2]
    "TAGP",  # total above-ground production [kg ha-1]
    "TWSO",  # total weight storage organs / TUBERS [kg ha-1]
    "TWLV",  # total weight leaves [kg ha-1]
    "TWST",  # total weight stems [kg ha-1]
    "TWRT",  # total weight roots [kg ha-1]
    "TRA",  # transpiration rate [cm day-1]
    "RD",  # rooting depth [cm]
    "SM",  # volumetric soil moisture [-]
)

# The milestone fields lifted out of WOFOST's own summary output (dates, so they are
# converted to days-since-emergence in the fixture — the comparison is matched-DVS and
# emergence-aligned, exactly as the spring-wheat one).
SUMMARY_DATES = ("DOE", "DOA", "DOM")


def variety_no() -> int | None:
    """The demo DB's ``crop_calendar`` variety for this (grid, crop, year).

    Recorded in the fixture provenance because it is **load-bearing on the headline
    finding**. That finding is "two independent parameterizations of the same organ of
    the same crop disagree about when tuber filling begins", and our side names its
    cultivar explicitly (cv Mara, chosen on source-internal grounds — [E] carries two
    potato cultivars whose vegetative rates differ by 1.6×). Without the oracle's
    variety on the record a reader cannot tell whether the disagreement is
    cross-MODEL or merely cross-CULTIVAR, which is exactly the distinction the rest of
    this exercise is careful about. The demo DB holds 46 potato varieties, so the
    question is not academic.
    """
    con = sqlite3.connect(_demo_db_path())
    try:
        row = con.execute(
            "SELECT variety_no FROM crop_calendar "
            "WHERE grid_no=? AND crop_no=? AND year=?",
            (GRID_NO, CROP_NO, YEAR),
        ).fetchone()
    finally:
        con.close()
    return None if row is None else int(row[0])


def _demo_db_path() -> str:
    """The demo SQLite DB PCSE builds from its bundled dump at first ``import pcse``."""
    from pcse.settings import settings

    return os.path.join(settings.PCSE_USER_HOME, "pcse.db")


def _cell(value: Any) -> float | None:
    """Cast an output cell to float; keep a pre-emergence ``None`` as null."""
    return None if value is None else float(value)


def _days_since_emergence(day: _dt.date | None) -> int | None:
    return None if day is None else (day - EMERGENCE_DATE).days


def benchmark_deltas(trajectory: list[dict[str, Any]]) -> dict[str, float]:
    """Max deviation per variable between our run and PCSE's shipped expectation.

    The demo DB's ``wofost_unittest_benchmarks`` table holds PCSE's *own* expected daily
    output for this exact ``(grid, crop, year, simulation_mode)``. Comparing against it
    checks that we drove the model correctly — a **runner** check, not a second oracle
    (it carries no authority the model itself lacks). Returns ``{variable: max |Δ|}``
    over the days both cover; an empty dict means the table has no matching rows.
    """
    con = sqlite3.connect(_demo_db_path())
    try:
        con.row_factory = sqlite3.Row
        rows = con.execute(
            "SELECT * FROM wofost_unittest_benchmarks "
            "WHERE grid_no=? AND crop_no=? AND year=? AND simulation_mode=?",
            (GRID_NO, CROP_NO, YEAR, MODE),
        ).fetchall()
    finally:
        con.close()
    expected = {row["day"]: row for row in rows}
    deltas: dict[str, float] = {}
    for entry in trajectory:
        reference = expected.get(entry["day"])
        if reference is None:
            continue
        for variable in OUTPUT_VARIABLES:
            # NOT ``variable not in reference``: ``sqlite3.Row`` is a SEQUENCE, so
            # ``in`` tests its VALUES, not its column names — the membership test
            # silently fails for every variable and this function returns {}, which
            # reads as "the cross-check passed" while checking nothing. Ruff's SIM118
            # ("use `key not in dict`") is wrong here for exactly that reason. Caught
            # by test_our_run_matches_pcses_own_shipped_expectation's `assert deltas`.
            if variable not in reference.keys():  # noqa: SIM118
                continue
            ours, theirs = entry[variable], reference[variable]
            if ours is None or theirs is None:
                continue
            deltas[variable] = max(
                deltas.get(variable, 0.0), abs(float(ours) - float(theirs))
            )
    return deltas


def run_potato() -> dict[str, Any]:
    """Run the WOFOST potato season and return ``{provenance, trajectory}``.

    Imports PCSE lazily so the module stays importable without the oracle dep.
    """
    import pcse

    model = pcse.start_wofost(grid=GRID_NO, crop=CROP_NO, year=YEAR, mode=MODE)
    model.run(days=RUN_DAYS)
    raw = model.get_output()
    summary = model.get_summary_output()[0]

    trajectory = [
        {"day": row["day"].isoformat()}
        | {var: _cell(row.get(var)) for var in OUTPUT_VARIABLES}
        for row in raw
    ]
    milestones = {
        name: _days_since_emergence(summary.get(name)) for name in SUMMARY_DATES
    }

    provenance = {
        "description": (
            "WOFOST 7.2 potato reference trajectory, POTENTIAL production. OUTPUT of "
            "PCSE (a EUPL oracle) — facts, not PCSE code. Inputs ship WITH PCSE as its "
            "own bundled demo database (pcse/tests/test_data/pcse_dump.sql, loaded at "
            "import), so the run is fully offline and license-clean. NO parameter "
            "values are committed; our potato's params are sourced independently from "
            "primary literature (Penning de Vries et al. 1989, read off page images), "
            "never copied from the demo DB's crop_parameter_value rows. WOFOST uses an "
            "AMAX/light-response assimilation formalism, a different family from our "
            "FvCB core — read as a DIAGNOSTIC, never a fit target (ruling B). See "
            "docs/plans/post-roadmap-potato-crop.md and docs/reuse-and-licenses.md."
        ),
        "pcse_version": pcse.__version__,
        "model": "WOFOST72_PP",
        "crop_name": CROP_NAME,
        "crop_no": CROP_NO,
        "grid_no": GRID_NO,
        "year": YEAR,
        "variety_no": variety_no(),
        "mode": MODE,
        "site": {
            "latitude": LATITUDE,
            "longitude": LONGITUDE,
            "altitude_m": ALTITUDE,
            "note": "grid 31031 — Andalusia, Spain (the demo DB's own grid row)",
        },
        "weather_source": "GridWeatherDataProvider(31031) — bundled demo DB",
        "emergence_date": EMERGENCE_DATE.isoformat(),
        "milestones_days_since_emergence": milestones,
        "summary": {
            key: (value.isoformat() if isinstance(value, _dt.date) else value)
            for key, value in summary.items()
        },
        "output_variables": list(OUTPUT_VARIABLES),
        "n_days": len(trajectory),
        "benchmark_max_abs_delta": benchmark_deltas(trajectory),
        "benchmark_note": (
            "Max |our run − PCSE's shipped wofost_unittest_benchmarks| per variable. A "
            "RUNNER check (did we drive the model correctly), not a second oracle."
        ),
    }
    return {"provenance": provenance, "trajectory": trajectory}


def run_weather() -> dict[str, Any]:
    """Capture the daily grid-31031 weather over the crop life, in our fixture schema.

    One row per day from **emergence** (2000-02-20) forward, as
    ``{day, TEMP, IRRAD, VAP}`` — ``TEMP = (TMIN+TMAX)/2``, ``IRRAD`` (J m⁻² day⁻¹),
    ``VAP`` (hPa). Our potato drives its forcing from this (via the clean-room
    conversions in ``domains.biosphere.weather``), so the comparison to the oracle is
    under the *same* weather.
    """
    import pcse
    from pcse.tests.db_input import GridWeatherDataProvider

    con = sqlite3.connect(_demo_db_path())
    try:
        provider = GridWeatherDataProvider(con, GRID_NO)
    finally:
        con.close()

    last_day = EMERGENCE_DATE + _dt.timedelta(days=RUN_DAYS)
    weather: list[dict[str, Any]] = []
    day = EMERGENCE_DATE
    while day <= last_day:
        rec = provider(day)
        weather.append(
            {
                "day": day.isoformat(),
                "TEMP": (float(rec.TMIN) + float(rec.TMAX)) / 2.0,
                "IRRAD": float(rec.IRRAD),
                "VAP": float(rec.VAP),
            }
        )
        day += _dt.timedelta(days=1)

    provenance = {
        "description": (
            "Daily grid-31031 (Andalusia, 2000) weather over the WOFOST potato season "
            "— observational FACTS (not PCSE code, not crop-parameter values), mapped "
            "to our {TEMP, IRRAD, VAP} schema (TEMP = (TMIN+TMAX)/2). Drives our "
            "clean-room potato via the conversions in domains/biosphere/weather.py, so "
            "the comparison is under the same forcing. See docs/reuse-and-licenses.md."
        ),
        "pcse_version": pcse.__version__,
        "weather_source": "GridWeatherDataProvider(31031) — bundled demo DB",
        "latitude": LATITUDE,
        "emergence_date": EMERGENCE_DATE.isoformat(),
        "weather_variables": list(WEATHER_VARIABLES),
        "variable_units": {"TEMP": "degC", "IRRAD": "J/m2/day", "VAP": "hPa"},
        "n_days": len(weather),
    }
    return {"provenance": provenance, "weather": weather}


def load_fixture() -> dict[str, Any]:
    """Load the committed reference fixture (no PCSE needed)."""
    return json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))


def load_weather() -> dict[str, Any]:
    """Load the committed raw-weather fixture (no PCSE needed)."""
    return json.loads(WEATHER_FIXTURE_PATH.read_text(encoding="utf-8"))


def column(fixture: dict[str, Any], variable: str) -> list[float | None]:
    """Extract one variable's daily series from a loaded fixture (comparison helper)."""
    return [row[variable] for row in fixture["trajectory"]]


def write_fixture() -> Path:
    """Regenerate + overwrite the committed reference fixture. Run via ``-m``; requires
    the ``oracle`` dep group (offline — no network)."""
    result = run_potato()
    FIXTURE_PATH.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return FIXTURE_PATH


def write_weather_fixture() -> Path:
    """Regenerate + overwrite the committed raw-weather fixture. Run via ``-m``;
    requires the ``oracle`` dep group (offline)."""
    result = run_weather()
    WEATHER_FIXTURE_PATH.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return WEATHER_FIXTURE_PATH


if __name__ == "__main__":
    ref = write_fixture()
    ref_data = json.loads(ref.read_text(encoding="utf-8"))
    print(f"wrote {ref} - {ref_data['provenance']['n_days']} days")
    print(f"  milestones: {ref_data['provenance']['milestones_days_since_emergence']}")
    deltas = ref_data["provenance"]["benchmark_max_abs_delta"]
    print(f"  benchmark max-abs-delta: {deltas}")
    wx = write_weather_fixture()
    wx_data = json.loads(wx.read_text(encoding="utf-8"))
    print(f"wrote {wx} - {wx_data['provenance']['n_days']} days")
