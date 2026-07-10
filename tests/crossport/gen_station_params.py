"""Generate the station-owned param vectors the Rust port reads (Phase-7 Step 5, P7.5).

The reference is the **frozen Python station loaders** (``station.loader``). Each of the
three station-owned coefficients (water-recovery rate + efficiency, lamp photon
efficacy, harvest rate) is loaded through the *actual* pydantic schema + unit guard +
bound check (not re-read as raw YAML), then emitted as a C99 hex-float. The Rust
``station`` crate reads this committed file via ``include_str!`` (``src/params.rs``), so
it never links a YAML parser — the Step-3/4 Option-C precedent (``gen_sibling_params`` /
``gen_biosphere_params``).

**Why generate, not hardcode.** Decimal param values round-trip bit-identically through
any correctly-rounding parser, so the *values* are not where cross-port bit-exactness is
at risk. Generating still earns its keep: (1) the values pass through the frozen loader
(so a param file edit or a loader bound-check change surfaces here), and (2)
``test_crossport.py::test_station_params_in_sync`` guards drift.

This is the **single writer** of ``rust/crates/station/src/station_params.txt``.

Regenerate with::

    uv run python tests/crossport/gen_station_params.py
"""

from __future__ import annotations

from pathlib import Path

from station.loader import (
    load_harvest_params,
    load_lamp_params,
    load_water_recovery_params,
)

# The committed param file the Rust `station` crate includes at compile time.
PARAMS_PATH = (
    Path(__file__).resolve().parents[2]
    / "rust"
    / "crates"
    / "station"
    / "src"
    / "station_params.txt"
)


def _params() -> list[tuple[str, float]]:
    """The three station-owned coefficients, each through its frozen Python loader.

    Every name is unique across the three station param files, so a flat
    ``name → value`` table needs no prefix. The Rust reader is order-independent (keyed
    lookup); this ordering is only for a stable, human-diffable file.
    """
    recovery = load_water_recovery_params()
    lamp = load_lamp_params()
    harvest = load_harvest_params()
    return [
        ("recovery_rate", recovery.recovery_rate),
        ("recovery_efficiency", recovery.recovery_efficiency),
        ("photon_efficacy", lamp.photon_efficacy),
        ("harvest_rate", harvest.harvest_rate),
    ]


def render() -> str:
    """The committed file's exact text (LF line endings, trailing newline)."""
    lines = [
        "# Cross-port station params (Phase-7 Step 5, P7.5) — GENERATED, do not edit.",
        "# Source of truth: the frozen Python station loaders (station.loader). Each",
        "# value passes through the actual pydantic schema + unit guard + bound check,",
        "# then is emitted as a C99 hex-float; the Rust crate reads this file via",
        "# include_str! (src/params.rs) — no YAML parser. Decimal params round-trip",
        "# bit-identically across correct-rounding parsers, so these hex-floats pin",
        "# the loader-produced bits exactly.",
        "# Regenerate: uv run python tests/crossport/gen_station_params.py",
        "#",
        "# name\thexfloat",
    ]
    for name, value in _params():
        lines.append(f"{name}\t{value.hex()}")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    PARAMS_PATH.write_text(render(), encoding="utf-8")
    print(f"wrote {PARAMS_PATH}")
