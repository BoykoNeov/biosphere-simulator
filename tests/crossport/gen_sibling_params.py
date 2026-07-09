"""Generate the sibling-domain param vectors the Rust port reads (Phase-7 Step 3).

The reference is the **frozen Python loaders** (``domains.{power,thermal,eclss,crew}.
loader``). Each of the 12 Phase-5 coefficients is loaded through the *actual* pydantic
schema + exact-string unit guard + bound check (not re-read as raw YAML), then emitted
as a C99 hex-float. The Rust ``domains`` crate reads this committed file via
``include_str!`` (``src/params.rs``), so it never links a YAML parser — no
``serde_yaml`` dependency, and no YAML-1.1-vs-1.2 ``1.0e7`` parse-fidelity risk.

**Why generate, not hardcode (the advisor's call).** Decimal param values round-trip
bit-identically through any correctly-rounding parser (Python ``float`` and Rust
``f64::parse`` both round-to-nearest), so the *values* are not where cross-port
bit-exactness is at risk. Generating still earns its keep two ways over hand-inlining
12 constants: (1) the values pass through the *frozen loader* (so a param file edit or
a loader bound-check change surfaces here, honouring the plan's "Rust reads the same
frozen files" more faithfully than a literal transcription would), and (2)
``test_crossport.py::test_sibling_params_in_sync`` guards drift, the established
``gen_rng_vectors`` / ``gen_engine_vectors`` discipline.

This is the **single writer** of ``rust/crates/domains/src/sibling_params.txt``.

Regenerate with::

    uv run python tests/crossport/gen_sibling_params.py
"""

from __future__ import annotations

from pathlib import Path

from domains.crew.loader import load_crew_params
from domains.eclss.loader import load_eclss_params
from domains.power.loader import load_charge_params, load_self_discharge_params
from domains.thermal.loader import load_thermal_params

# The committed param file the Rust `domains` crate includes at compile time.
PARAMS_PATH = (
    Path(__file__).resolve().parents[2]
    / "rust"
    / "crates"
    / "domains"
    / "src"
    / "sibling_params.txt"
)


def _params() -> list[tuple[str, float]]:
    """The 12 sibling coefficients, each loaded through its frozen Python loader.

    Order is domain-then-declaration (power charge, power self-discharge, thermal ×4,
    eclss ×4, crew ×2); the Rust reader is order-independent (keyed lookup), so this
    ordering is only for a stable, human-diffable file. Every name is unique across the
    four domains, so a flat ``name → value`` table needs no domain prefix.
    """
    charge = load_charge_params()
    self_discharge = load_self_discharge_params()
    thermal = load_thermal_params()
    eclss = load_eclss_params()
    crew = load_crew_params()
    return [
        ("charge_efficiency", charge.charge_efficiency),
        ("self_discharge_rate", self_discharge.self_discharge_rate),
        ("emissivity", thermal.emissivity),
        ("radiator_area", thermal.radiator_area),
        ("heat_capacity", thermal.heat_capacity),
        ("space_temperature", thermal.space_temperature),
        ("co2_scrub_rate", eclss.co2_scrub_rate),
        ("condense_rate", eclss.condense_rate),
        ("o2_makeup_gain", eclss.o2_makeup_gain),
        ("o2_setpoint", eclss.o2_setpoint),
        ("respired_carbon_fraction", crew.respired_carbon_fraction),
        ("insensible_water_fraction", crew.insensible_water_fraction),
    ]


def render() -> str:
    """The committed file's exact text (LF line endings, trailing newline)."""
    lines = [
        "# Cross-port sibling params (Phase-7 Step 3, P7.3) — GENERATED, do not edit.",
        "# Source of truth: the frozen Python loaders (domains.*.loader). Each value",
        "# passes through the actual pydantic schema + unit guard + bound check, then",
        "# is emitted as a C99 hex-float; the Rust crate reads this file via",
        "# include_str! (src/params.rs) — no YAML parser, no serde_yaml. Decimal",
        "# params round-trip bit-identically across correct-rounding parsers, so",
        "# these hex-floats pin the loader-produced bits exactly.",
        "# Regenerate: uv run python tests/crossport/gen_sibling_params.py",
        "#",
        "# name\thexfloat",
    ]
    for name, value in _params():
        lines.append(f"{name}\t{value.hex()}")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    PARAMS_PATH.write_text(render(), encoding="utf-8")
    print(f"wrote {PARAMS_PATH}")
