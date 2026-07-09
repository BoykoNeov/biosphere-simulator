"""Generate the raw-weather-facts table the Rust biosphere port reads (Phase-7 P7.4).

The reference is the committed oracle fixture
(`tests/oracle/winter_wheat_weather.json`): raw NASAPower **facts** (IRRAD, TEMP, VAP,
an ISO date), license-clean per `docs/reuse-and-licenses.md`. This generator emits those
facts as a flat hex-float table plus the day-of-year (a *calendar* computation,
`date.fromisoformat().tm_yday`, NOT a libm op, so precomputing it Python-side is
legitimate) and the latitude.

**Why emit raw facts, not the precomputed forcing tables.** The clean-room weather
conversions (`daylength_seconds` with sin/acos/tan, `incident_par`, `net_radiation`,
`vapor_pressure_deficit` with exp) are the *heaviest libm-audit surface* of Phase 7 (the
plan's Step 4). Emitting the raw facts and porting the conversions to Rust means those
transcendentals run **in Rust**, exercising the cross-port libm the way the plan
intends; emitting the precomputed tables would compute them in Python and skip it.

The Rust `biosphere::weather` module reads this via `include_str!` and computes the
per-day PAR / net-radiation / VPD / daylength forcing tables itself.
`test_crossport.py::test_biosphere_weather_facts_in_sync` guards drift.

Single writer of `rust/crates/domains/src/biosphere/weather_facts.txt`.

Regenerate with::

    uv run python tests/crossport/gen_biosphere_weather.py
"""

from __future__ import annotations

import json
from datetime import date
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE = REPO_ROOT / "tests" / "oracle" / "winter_wheat_weather.json"
FACTS_PATH = (
    REPO_ROOT
    / "rust"
    / "crates"
    / "domains"
    / "src"
    / "biosphere"
    / "weather_facts.txt"
)


def _rows() -> tuple[float, list[tuple[int, float, float, float]]]:
    """(latitude, [(day_of_year, TEMP, IRRAD, VAP), ...]) from the frozen fixture."""
    data = json.loads(FIXTURE.read_text(encoding="utf-8"))
    latitude = float(data["provenance"]["latitude"])
    out: list[tuple[int, float, float, float]] = []
    for row in data["weather"]:
        doy = date.fromisoformat(str(row["day"])).timetuple().tm_yday
        out.append((doy, float(row["TEMP"]), float(row["IRRAD"]), float(row["VAP"])))
    return latitude, out


def render() -> str:
    """The committed file's exact text (LF line endings, trailing newline)."""
    latitude, rows = _rows()
    lines = [
        "# Cross-port raw-weather facts (Phase-7 Step 4). GENERATED, do not edit.",
        "# Source: tests/oracle/winter_wheat_weather.json (raw NASAPower facts).",
        "# The Rust biosphere::weather reads this via include_str! and runs the",
        "# clean-room conversions (daylength sin/acos/tan, incident_par,",
        "# net_radiation, vpd exp) ITSELF, exercising the weather libm cross-port.",
        "# day_of_year is a calendar computation (tm_yday), not a libm op, so it",
        "# is precomputed. Regenerate via gen_biosphere_weather.py.",
        "#",
        f"latitude\t{latitude.hex()}",
        "# doy\ttemp_c\tirrad_j_m2_day\tvap_hpa",
    ]
    for doy, temp, irrad, vap in rows:
        lines.append(f"{doy}\t{temp.hex()}\t{irrad.hex()}\t{vap.hex()}")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    FACTS_PATH.parent.mkdir(parents=True, exist_ok=True)
    FACTS_PATH.write_text(render(), encoding="utf-8")
    print(f"wrote {FACTS_PATH}")
