"""Offline WOFOST winter-wheat runner — produces the committed oracle fixture.

Phase 1, Step 3 (P5). Drives PCSE/WOFOST for **winter wheat** under a fixed
weather/management forcing and captures a reference **trajectory** (development,
canopy, organ biomass, water use) for the behavioral-match gate
(``lab/oracle_match.py``; the full gate is Step 11).

LICENSING — the load-bearing rule (``docs/reuse-and-licenses.md``):
  * PCSE is EUPL → running it as an oracle is *mere use*; its **output is facts**.
  * Winter-wheat **inputs** come from the ``WOFOST_crop_parameters`` repo, which has
    **no license = all rights reserved** ("Do NOT copy the files"). PCSE downloads it
    to the user's ``~/.pcse`` cache — a private, transient copy used to run the model.
  * Therefore we commit **ONLY the output trajectory + a provenance record, NEVER the
    parameter YAML**. The provenance makes the clean lineage auditable.

This module is deliberately **not** named ``test_*`` so pytest never imports it at
collection time on a machine without ``pcse`` (no collection-time ``ImportError``).
The ``oracle``-marked tests in ``test_oracle_regeneration.py`` import it behind
``importorskip``.
Regenerate the fixture with::

    uv run --group oracle python -m tests.oracle.runner
"""

from __future__ import annotations

import datetime as _dt
import json
from pathlib import Path
from typing import Any

# Fixture lives beside this module; the committed reference + its provenance.
FIXTURE_PATH = Path(__file__).with_name("winter_wheat_reference.json")

# --- forcing definition (also recorded verbatim in the fixture provenance) ----
CROP_NAME = "wheat"
VARIETY_NAME = "Winter_wheat_101"
LATITUDE = 52.0
LONGITUDE = 5.0
SOW_DATE = _dt.date(2006, 10, 1)
HARVEST_DATE = _dt.date(2007, 8, 1)
MODEL_VARIANT = "Wofost72_PP"  # potential production (light/temperature-limited only)

# Output variables captured per day. Kept to the potential-production set for the
# first fixture; water-limited (SM-driven) and N-limited variables are JIT additions
# when the water (Step 7) and nitrogen (Step 10) processes land.
OUTPUT_VARIABLES = (
    "DVS",  # development stage [-]
    "LAI",  # leaf area index [m2 m-2]
    "TAGP",  # total above-ground production [kg ha-1]
    "TWLV",  # total weight leaves [kg ha-1]
    "TWST",  # total weight stems [kg ha-1]
    "TWRT",  # total weight roots [kg ha-1]
    "TWSO",  # total weight storage organs / yield [kg ha-1]
    "TRA",  # transpiration rate [cm day-1]
)


def run_winter_wheat() -> dict[str, Any]:
    """Run the WOFOST winter-wheat season and return ``{provenance, trajectory}``.

    Imports PCSE lazily so the module stays importable for documentation/introspection
    without the oracle dep installed (the actual call still requires it).
    """
    import pcse
    from pcse.base import ParameterProvider
    from pcse.input import (
        DummySoilDataProvider,
        NASAPowerWeatherDataProvider,
        WOFOST72SiteDataProvider,
        YAMLCropDataProvider,
    )
    from pcse.models import Wofost72_PP

    crop = YAMLCropDataProvider()
    crop.set_active_crop(CROP_NAME, VARIETY_NAME)
    soil = DummySoilDataProvider()
    site = WOFOST72SiteDataProvider(WAV=10)
    params = ParameterProvider(cropdata=crop, soildata=soil, sitedata=site)

    agro = [
        {
            SOW_DATE: {
                "CropCalendar": {
                    "crop_name": CROP_NAME,
                    "variety_name": VARIETY_NAME,
                    "crop_start_date": SOW_DATE,
                    "crop_start_type": "sowing",
                    "crop_end_date": HARVEST_DATE,
                    "crop_end_type": "harvest",
                    "max_duration": 365,
                },
                "TimedEvents": None,
                "StateEvents": None,
            }
        }
    ]

    weather = NASAPowerWeatherDataProvider(latitude=LATITUDE, longitude=LONGITUDE)
    model = Wofost72_PP(params, weather, agro)
    model.run_till_terminate()
    raw = model.get_output()

    trajectory = [
        {"day": row["day"].isoformat()}
        | {var: float(row[var]) for var in OUTPUT_VARIABLES}
        for row in raw
    ]

    provenance = {
        "description": (
            "WOFOST winter-wheat potential-production reference trajectory. "
            "OUTPUT of PCSE (a EUPL oracle) — facts, not PCSE code. NO crop-parameter "
            "YAML is committed (it is unlicensed; PCSE downloads it locally for the "
            "run only). See docs/reuse-and-licenses.md."
        ),
        "pcse_version": pcse.__version__,
        "model_variant": MODEL_VARIANT,
        "crop_parameter_source": crop.repository,
        "crop_parameter_compatible_version": crop.compatible_version,
        "crop_name": CROP_NAME,
        "variety_name": VARIETY_NAME,
        "weather_source": "NASAPowerWeatherDataProvider",
        "latitude": LATITUDE,
        "longitude": LONGITUDE,
        "sow_date": SOW_DATE.isoformat(),
        "harvest_date": HARVEST_DATE.isoformat(),
        "output_variables": list(OUTPUT_VARIABLES),
        "n_days": len(trajectory),
    }
    return {"provenance": provenance, "trajectory": trajectory}


def load_fixture() -> dict[str, Any]:
    """Load the committed reference fixture (no PCSE needed)."""
    return json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))


def column(fixture: dict[str, Any], variable: str) -> list[float]:
    """Extract one variable's daily series from a loaded fixture (helper for the
    behavioral-match comparison)."""
    return [float(row[variable]) for row in fixture["trajectory"]]


def write_fixture() -> Path:
    """Regenerate and overwrite the committed fixture. Run via ``-m``; requires the
    ``oracle`` dep group + network (NASAPower)."""
    result = run_winter_wheat()
    FIXTURE_PATH.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return FIXTURE_PATH


if __name__ == "__main__":
    path = write_fixture()
    data = json.loads(path.read_text(encoding="utf-8"))
    prov = data["provenance"]
    print(f"wrote {path} — {prov['n_days']} days, pcse {prov['pcse_version']}")
