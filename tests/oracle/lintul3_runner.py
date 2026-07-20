"""Offline LINTUL3 spring-wheat runner — produces the committed oracle fixture.

Post-roadmap: the day-neutral-crop validation. Plan of record:
`docs/plans/post-roadmap-day-neutral-crop.md`.
Drives PCSE's **LINTUL3** model for **spring wheat** under its *bundled* forcing and
captures a reference **trajectory** (development, canopy, organ biomass, water use) for
the diagnostic comparison against our clean-room day-neutral crop. LINTUL3 is a
**light-use-efficiency** model — a *different family* from our FvCB core — so the
comparison is a **diagnostic, never a fit target** (ruling B); the honest match is on
phenology (DVS timing) and canopy (LAI) shape, not absolute biomass.

LICENSING — why this oracle is cleaner than the winter-wheat one
(`docs/reuse-and-licenses.md`):
  * PCSE is EUPL → running it as an oracle is *mere use*; its **output is facts**.
  * Unlike the WOFOST winter-wheat inputs (which come from the unlicensed
    ``WOFOST_crop_parameters`` repo and are downloaded to a private cache), the
    LINTUL3 spring-wheat **inputs ship WITH PCSE** as its own test data
    (``pcse/tests/test_data/lintul3_springwheat.{agro,crop,site,soil}`` + the CABO
    ``NL1`` weather), covered by PCSE's own EUPL. The run is therefore **fully offline**
    — no network, no unlicensed cache.
  * We still commit **ONLY the output trajectory + a provenance record, NEVER the
    parameter values** (the honor-system discipline held for all oracles). The crop's
    own phenology params are sourced **independently from primary literature**, never
    copied from ``lintul3_springwheat.crop`` (that would be reverse-engineering PCSE).

Deliberately **not** named ``test_*`` so pytest never imports it at collection time on a
machine without ``pcse``. The ``oracle``-marked regeneration test imports it behind
``importorskip``. Regenerate the fixtures with::

    uv run --group oracle python -m tests.oracle.lintul3_runner
"""

from __future__ import annotations

import datetime as _dt
import json
import os
from pathlib import Path
from typing import Any

# Fixtures live beside this module: the reference trajectory (PCSE OUTPUT) and the daily
# weather (CABO NL1 facts, mapped to our {TEMP, IRRAD, VAP} schema) the season is driven
# by — both license-clean (facts), neither containing crop-parameter values.
FIXTURE_PATH = Path(__file__).with_name("spring_wheat_reference.json")
WEATHER_FIXTURE_PATH = Path(__file__).with_name("spring_wheat_weather.json")

# --- forcing definition (recorded verbatim in the fixture provenance) ---------
# All from PCSE's bundled ``lintul3_springwheat.agro``: spring wheat, EMERGENCE-started
# (no sowing→emergence lag, no vernalization — a spring crop), CABO ``NL1`` weather,
# 1997.
CROP_NAME = "wheat"
VARIETY_NAME = "spring-wheat"
CABO_STATION = "NL1"
EMERGENCE_DATE = _dt.date(
    1997, 3, 31
)  # agro crop_start_date, crop_start_type: emergence
CONFIG = "Lintul3.conf"
RUN_DAYS = 300  # the PCSE LINTUL3 test's own horizon (terminates at maturity earlier)

# Weather fields we record, mapped to OUR resolver's schema (as the winter fixture):
# daily-mean TEMP (°C) = (TMIN+TMAX)/2 — exactly what LINTUL3 uses for thermal time;
# IRRAD (J m⁻² day⁻¹ shortwave); VAP (hPa vapour pressure). FACTS; the clean-room
# conversions to flow drivers live in src/domains/biosphere/weather.py.
WEATHER_VARIABLES = ("TEMP", "IRRAD", "VAP")

# Output variables captured per day (LINTUL3 state; g m⁻² for the weights, LAI [-],
# DVS [-], TRAN cm day⁻¹, TRANRF the water-stress reduction factor [-]).
OUTPUT_VARIABLES = (
    "DVS",  # development stage [-]
    "LAI",  # leaf area index [m2 m-2]
    "WLVG",  # weight green leaves [g m-2]
    "WLVD",  # weight dead leaves [g m-2]
    "WST",  # weight stems [g m-2]
    "WSO",  # weight storage organs / grain [g m-2]
    "WRT",  # weight roots [g m-2]
    "TAGBM",  # total above-ground biomass [g m-2]
    "TGROWTH",  # total growth [g m-2]
    "TRAN",  # transpiration rate [cm day-1] (== mm/day)
    "TRANRF",  # transpiration reduction factor (water stress) [-]
)


def _test_data_dir() -> str:
    """The bundled ``pcse/tests/test_data`` directory (ships with the EUPL package)."""
    import pcse

    return os.path.join(os.path.dirname(pcse.__file__), "tests", "test_data")


def _build_engine() -> Any:
    """Construct the LINTUL3 spring-wheat engine from the bundled inputs (offline)."""
    import yaml
    from pcse.base import ParameterProvider
    from pcse.engine import Engine
    from pcse.input import CABOWeatherDataProvider, PCSEFileReader

    td = _test_data_dir()
    with open(os.path.join(td, "lintul3_springwheat.agro")) as agro_file:
        amgt = yaml.safe_load(agro_file)["AgroManagement"]
    soil = PCSEFileReader(os.path.join(td, "lintul3_springwheat.soil"))
    site = PCSEFileReader(os.path.join(td, "lintul3_springwheat.site"))
    crop = PCSEFileReader(os.path.join(td, "lintul3_springwheat.crop"))
    weather = CABOWeatherDataProvider(CABO_STATION, td, ETmodel="P")
    params = ParameterProvider(sitedata=site, soildata=soil, cropdata=crop)
    return Engine(params, weather, agromanagement=amgt, config=CONFIG)


def _cell(value: Any) -> float | None:
    """Cast a LINTUL3 output cell to float; keep a pre-emergence ``None`` as null."""
    return None if value is None else float(value)


def run_spring_wheat() -> dict[str, Any]:
    """Run the LINTUL3 spring-wheat season and return ``{provenance, trajectory}``.

    Imports PCSE lazily so the module stays importable without the oracle dep. The
    trajectory keeps **all** model days; pre-emergence cells are ``null`` (LINTUL3
    reports ``None`` before emergence) and the comparison skips them.
    """
    import pcse

    model = _build_engine()
    model.run(days=RUN_DAYS)
    raw = model.get_output()

    trajectory = [
        {"day": row["day"].isoformat()}
        | {var: _cell(row.get(var)) for var in OUTPUT_VARIABLES}
        for row in raw
    ]

    provenance = {
        "description": (
            "LINTUL3 spring-wheat reference trajectory. OUTPUT of PCSE (a EUPL "
            "oracle) — facts, not PCSE code. Inputs ship WITH PCSE as its own bundled "
            "test data (lintul3_springwheat.{agro,crop,site,soil} + CABO NL1 weather), "
            "so the run is fully offline and license-clean. NO parameter values are "
            "committed; the crop's own phenology is sourced independently from primary "
            "literature (never copied from lintul3_springwheat.crop). LINTUL3 is a "
            "LIGHT-USE-EFFICIENCY model, a different family from our FvCB core — read "
            "as a DIAGNOSTIC, never a fit target (ruling B). See "
            "docs/plans/post-roadmap-day-neutral-crop.md and "
            "docs/reuse-and-licenses.md."
        ),
        "pcse_version": pcse.__version__,
        "model": "LINTUL3",
        "config": CONFIG,
        "crop_name": CROP_NAME,
        "variety_name": VARIETY_NAME,
        "weather_source": f"CABOWeatherDataProvider({CABO_STATION}) — bundled",
        "emergence_date": EMERGENCE_DATE.isoformat(),
        "output_variables": list(OUTPUT_VARIABLES),
        "n_days": len(trajectory),
    }
    return {"provenance": provenance, "trajectory": trajectory}


def run_weather() -> dict[str, Any]:
    """Capture the daily CABO ``NL1`` weather over the crop life, in our fixture schema.

    One row per day from **emergence** (1997-03-31) to the end of the LINTUL3 run, as
    ``{day, TEMP, IRRAD, VAP}`` — ``TEMP = (TMIN+TMAX)/2`` (the daily mean LINTUL3 uses
    for thermal time), ``IRRAD`` (J m⁻² day⁻¹), ``VAP`` (hPa). Our crop drives its
    forcing from this (via the clean-room conversions in ``domains.biosphere.weather``),
    so the comparison to the oracle is under the *same* weather.
    """
    import pcse
    from pcse.input import CABOWeatherDataProvider

    provider = CABOWeatherDataProvider(CABO_STATION, _test_data_dir(), ETmodel="P")
    # The run terminates at maturity; capture emergence → that horizon inclusively.
    last_day = EMERGENCE_DATE + _dt.timedelta(days=RUN_DAYS)
    weather: list[dict[str, Any]] = []
    day = EMERGENCE_DATE
    while day <= last_day:
        rec = provider(day)
        tmean = (float(rec.TMIN) + float(rec.TMAX)) / 2.0
        weather.append(
            {
                "day": day.isoformat(),
                "TEMP": tmean,
                "IRRAD": float(rec.IRRAD),
                "VAP": float(rec.VAP),
            }
        )
        day += _dt.timedelta(days=1)

    provenance = {
        "description": (
            "Daily CABO NL1 (1997) weather over the LINTUL3 spring-wheat season — "
            "observational FACTS (not PCSE code, not crop-parameter values), mapped "
            "to our {TEMP, IRRAD, VAP} schema (TEMP = (TMIN+TMAX)/2, the daily mean "
            "LINTUL3 uses for thermal time). Drives our day-neutral crop via the "
            "clean-room conversions in domains/biosphere/weather.py, so the comparison "
            "is under the same forcing. See docs/reuse-and-licenses.md."
        ),
        "pcse_version": pcse.__version__,
        "weather_source": f"CABOWeatherDataProvider({CABO_STATION}) — bundled",
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
    result = run_spring_wheat()
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
    print(f"wrote {ref} — {ref_data['provenance']['n_days']} days")
    wx = write_weather_fixture()
    wx_data = json.loads(wx.read_text(encoding="utf-8"))
    print(f"wrote {wx} — {wx_data['provenance']['n_days']} days")
