"""Phase-4 Step-5 (P4.3): the biosphere freeze manifest + its completeness gate.

The machine-readable companion to ``docs/biosphere-reference.md`` (the human-readable
freeze contract). This module owns the **manifest**
(``docs/biosphere-reference.manifest.json``) that names the frozen surface — the locked
integrator + dt, the decade-scale horizon, the flow-class set, the biosphere param
files, the canonical scenarios + their goldens, the driving weather fixture — and a
**completeness gate** aimed exactly where the golden byte-compares are blind.

**What the goldens already enforce (so this file does NOT re-check).** Every committed
golden (``test_regression_*.py``) byte-compares a scenario's serialized output to a
frozen snapshot. So a *value* change to any of the 13 frozen param files, a flow-law
edit, an integrator/dt switch, or a weather-fixture edit *already* moves a golden and
fails there — re-hashing those files here as a CI gate would be redundant
belt-and-suspenders, and on a hand-edited YAML under ``autocrlf`` a raw byte hash is not
even reproducible across platforms. The manifest records a **newline-normalized**
sha-256 of each frozen file as **provenance** — a re-derivable record of *which content*
was frozen, regenerated on a deliberate unfreeze — **not** an assertion. Value
enforcement is the goldens' job.

**What the goldens CANNOT see — the gap this gate owns: completeness.** A newly added
param file or flow class that is wired into no committed golden is invisible to every
byte compare. So the gate asserts the *set* of the frozen surface against the live tree:

* the param-file set (``params/*.yaml`` minus the Phase-0 ``demo.yaml`` skeleton) equals
  the manifest's param set — catches "added a param, forgot to freeze it";
* the flow-class set, **derived from freshly assembled canonical registries** (never
  hand-listed — a dozen-plus flow classes span the compartment modules), equals the
  manifest's flow set — catches "added a flow, forgot to freeze it";
* the manifest horizon equals the importable ``LONG_HORIZON_YEARS`` constant (the single
  source of truth the long-horizon golden and the decade probe share);
* every golden, param, and forcing file the manifest names exists on disk.

The integrator (Euler) and dt (1.0 day) have **no importable constant** — each
regression run helper selects them inline — so they are *documented* in the manifest and
*enforced* by the goldens (an RK4 or dt switch moves every golden), not asserted here.

Regeneration is a deliberate, separate ``__main__`` action (the golden discipline): on
an advisor-reviewed unfreeze, run ``uv run python tests/test_freeze_manifest.py`` and
review the manifest diff. Zero ``simcore`` change (docs + tests only).
"""

import hashlib
import json
from pathlib import Path
from typing import Any

import domains.biosphere
from domains.biosphere.season import (
    CONSUMER_CHAMBER_SCENARIO,
    CONSUMER_CHAMBER_YEARS,
    LONG_HORIZON_YEARS,
    PERENNIAL_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_SCENARIO,
    SEALED_CHAMBER_YEARS,
    build_season,
)

_REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = _REPO_ROOT / "docs" / "biosphere-reference.manifest.json"

PARAMS_DIR = Path(domains.biosphere.__file__).parent / "params"
GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# The Phase-0 engine-skeleton demo param file — NOT biosphere science. It feeds the demo
# flows in ``flows.py`` (no real biology), frozen separately by the Phase-0 demo
# regression goldens (``demo_euler_state.json`` / ``demo_rk4_state.json``). Scoped out
# of the biosphere reference, by name, so its absence reads as deliberate.
_EXCLUDED_PARAMS = frozenset({"demo.yaml"})

# Scenario name -> (human scenario label, year count, golden filename). The four Phase-3
# canonical scenarios + the three Phase-4 long-horizon artifacts (P4.2). The labels are
# documentation; the year counts come from importable constants so they cannot drift.
_SCENARIOS: dict[str, tuple[str, int, str]] = {
    "open_season": ("DEFAULT_SCENARIO (open field)", 1, "season_euler_state.json"),
    "sealed_chamber": (
        "SEALED_CHAMBER_SCENARIO",
        SEALED_CHAMBER_YEARS,
        "sealed_chamber_state.json",
    ),
    "perennial_chamber": (
        "PERENNIAL_CHAMBER_SCENARIO",
        PERENNIAL_CHAMBER_YEARS,
        "perennial_chamber_state.json",
    ),
    "consumer_chamber": (
        "CONSUMER_CHAMBER_SCENARIO",
        CONSUMER_CHAMBER_YEARS,
        "consumer_chamber_state.json",
    ),
    "perennial_long_horizon": (
        "PERENNIAL_CHAMBER_SCENARIO",
        LONG_HORIZON_YEARS,
        "perennial_long_horizon_state.json",
    ),
    "consumer_long_horizon": (
        "CONSUMER_CHAMBER_SCENARIO",
        LONG_HORIZON_YEARS,
        "consumer_long_horizon_state.json",
    ),
    "drift_summary": (
        "PERENNIAL_CHAMBER_SCENARIO + CONSUMER_CHAMBER_SCENARIO (stability signature)",
        LONG_HORIZON_YEARS,
        "drift_summary.json",
    ),
}


def _normalized_sha256(path: Path) -> str:
    """sha-256 over newline-normalized (LF) content — a reproducible provenance hash.

    Hashing the raw bytes would make the recorded value depend on the checkout's line
    endings (``autocrlf`` on Windows vs. LF on Linux), so the same frozen file would
    hash differently per platform. Normalizing to LF first makes the hash a stable
    record of *content*, independent of how git materialized the file. Provenance, not a
    gate — value enforcement is the scenario goldens (see the module docstring).
    """
    text = path.read_text(encoding="utf-8")
    normalized = "\n".join(text.splitlines())
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def _flow_set() -> list[str]:
    """The frozen flow-class set: the union of flow classes across the canonical builds.

    **Derived, never hand-listed** (the advisor's lever): assemble each canonical
    scenario's registry and collect ``type(flow).__name__``. The open field carries the
    boundary-atmosphere producer flows; the sealed chambers add the decomposer / water-
    cycle / consumer flows — so the union over all four is the complete frozen flow
    surface. A flow class added to any compartment builder but wired into no golden
    still appears here, so the completeness gate catches it.
    """
    builds = (
        build_season(),  # the open-field DEFAULT_SCENARIO
        build_season(SEALED_CHAMBER_SCENARIO),
        build_season(PERENNIAL_CHAMBER_SCENARIO),
        build_season(CONSUMER_CHAMBER_SCENARIO),
    )
    classes = {
        type(flow).__name__ for _state, registry in builds for flow in registry.flows
    }
    return sorted(classes)


def _frozen_param_files() -> list[str]:
    """The frozen biosphere param-file names (params dir minus the demo skeleton)."""
    return sorted(
        p.name for p in PARAMS_DIR.glob("*.yaml") if p.name not in _EXCLUDED_PARAMS
    )


def _build_manifest() -> dict[str, object]:
    """Assemble the manifest from the live tree — the single source for regeneration."""
    scenarios: dict[str, object] = {}
    for name, (label, years, golden) in _SCENARIOS.items():
        scenarios[name] = {
            "scenario": label,
            "years": years,
            "golden": golden,
            "golden_sha256": _normalized_sha256(GOLDEN_DIR / golden),
        }
    return {
        "_comment": (
            "Phase-4 freeze manifest (P4.3). Names the frozen biosphere reference "
            "surface. See docs/biosphere-reference.md for the freeze contract + the "
            "unfreeze discipline. Hashes are newline-normalized sha-256 PROVENANCE "
            "(value enforcement is the scenario goldens). Regenerate on a deliberate "
            "unfreeze: uv run python tests/test_freeze_manifest.py."
        ),
        "frozen_at_phase": 4,
        "reference_doc": "docs/biosphere-reference.md",
        "integrator": "EulerIntegrator",
        "dt_days": 1.0,
        "long_horizon_years": LONG_HORIZON_YEARS,
        "flow_set": _flow_set(),
        "forcing": {
            "weather_fixture": WEATHER_FIXTURE.name,
            "weather_sha256": _normalized_sha256(WEATHER_FIXTURE),
        },
        "param_files": {
            name: _normalized_sha256(PARAMS_DIR / name)
            for name in _frozen_param_files()
        },
        "scenarios": scenarios,
    }


def _manifest_dumps(manifest: dict[str, object]) -> str:
    """Serialize the manifest to canonical JSON — the project golden discipline.

    ``indent=2, sort_keys=True`` + a trailing newline, matching ``sim_io.dumps`` and the
    drift-summary golden, so the manifest reads and diffs like every other committed
    snapshot in the repo.
    """
    return json.dumps(manifest, indent=2, sort_keys=True) + "\n"


def _load_manifest() -> dict[str, Any]:
    # The committed manifest as parsed JSON (values are Any — the gate reads the frozen
    # sets out of it; pyright would otherwise type every value as ``object``).
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


# --- the completeness gate (what the goldens are blind to) -------------------


def test_frozen_param_set_is_complete() -> None:
    # Every biosphere param file (minus the Phase-0 demo skeleton) is frozen, and the
    # manifest names no param that has vanished. Catches an added-but-unfrozen param the
    # scenario goldens can't see (a new file wired into no committed golden).
    manifest = _load_manifest()
    on_disk = {p.name for p in PARAMS_DIR.glob("*.yaml")} - _EXCLUDED_PARAMS
    assert set(manifest["param_files"]) == on_disk


def test_frozen_flow_set_is_complete() -> None:
    # The manifest's flow set equals the flow classes assembled across the four
    # canonical scenarios — derived, not hand-listed. Catches an unfrozen flow.
    manifest = _load_manifest()
    assert set(manifest["flow_set"]) == set(_flow_set())


def test_manifest_horizon_matches_constant() -> None:
    # The frozen decade-scale horizon tracks the single importable source of truth
    # (LONG_HORIZON_YEARS), so the manifest cannot silently disagree with the
    # long-horizon golden / the decade probe.
    manifest = _load_manifest()
    assert manifest["long_horizon_years"] == LONG_HORIZON_YEARS


def test_manifest_named_files_exist() -> None:
    # Every golden, param, and forcing file the manifest names is present on disk — a
    # renamed or deleted frozen artifact fails here, not as a mysterious load error
    # later.
    manifest = _load_manifest()
    for name in manifest["param_files"]:
        assert (PARAMS_DIR / name).is_file(), name
    for entry in manifest["scenarios"].values():
        assert (GOLDEN_DIR / entry["golden"]).is_file(), entry["golden"]
    forcing = manifest["forcing"]["weather_fixture"]
    assert WEATHER_FIXTURE.is_file() and WEATHER_FIXTURE.name == forcing


def test_manifest_declares_locked_integrator_and_dt() -> None:
    # The integrator + dt have no importable constant to assert against (selected inline
    # in each regression run helper), so they live as documented strings in the file;
    # the goldens enforce them (an RK4 / dt switch moves every golden). This pins that
    # the manifest *records the lock* — the documentation half of "locked by end of P4".
    manifest = _load_manifest()
    assert manifest["integrator"] == "EulerIntegrator"
    assert manifest["dt_days"] == 1.0


def _regenerate() -> None:
    """Rewrite the committed manifest from the current live tree.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_freeze_manifest.py

    Review the diff before committing: a change means the frozen surface moved (a new
    flow / param / scenario, a moved horizon, or a frozen file's content) — i.e. an
    **unfreeze**, which the discipline in docs/biosphere-reference.md governs. Written
    via ``write_bytes`` (explicit LF, like the goldens) so the manifest is byte-stable
    across platforms.
    """
    MANIFEST_PATH.write_bytes(_manifest_dumps(_build_manifest()).encode("utf-8"))
    print(f"wrote {MANIFEST_PATH}")


if __name__ == "__main__":
    _regenerate()
