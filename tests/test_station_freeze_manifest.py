"""Phase-6 Step-10 (P6.10): the STATION freeze manifest + its completeness gate.

The machine-readable companion to ``docs/station-reference.md`` (the human-readable
station freeze contract), one assembly level up from the biosphere's
``tests/test_freeze_manifest.py``. This module owns the **station manifest**
(``docs/station-reference.manifest.json``) that names the frozen *whole-assembly*
surface — the locked Euler integrator, the station+sibling flow-class set, the eight
sibling + station param files, the 13 station/sibling scenarios + their goldens — and a
**completeness gate** aimed exactly where the golden byte-compares are blind.

**Whole-assembly scope (advisor-reviewed, user-confirmed).** Step 10 freezes the whole
integrated station: the **Phase-5 siblings** (power / thermal / eclss / crew — their
flow classes + param files) **and** the four station seams + three station-owned params
+ the 13 station/sibling scenarios → goldens. The **biosphere is delegated** to its own
``docs/biosphere-reference.manifest.json`` (referenced via ``delegates_to``, **not**
re-hashed here — it was frozen in Phase 4). Reference-only would have left the sibling
flows/params changeable with no unfreeze ceremony *in exactly the layer Phase 7 ports* —
a silent-change hole; freezing them (under a "frozen-but-illustrative" caveat for the
uncalibrated ECLSS / harvest / recovery rate-constants, consistent with Step 9 and with
the biosphere's own uncalibrated ``TODO(cite)`` freeze) closes it.

**What the goldens already enforce (so this file does NOT re-check): values.** Every
committed golden (``test_regression_*.py`` / ``test_*_run.py``) byte-compares a
scenario's serialized output to a frozen snapshot. So a *value* change to any of the
eight frozen param files, a flow-law edit, or an integrator/dt switch *already* moves a
golden and fails there. The manifest records a **newline-normalized** sha-256 of each
frozen param file as **provenance** — a re-derivable record of *which content* was
frozen, regenerated on a deliberate unfreeze — **not** an assertion (a raw byte hash of
hand-edited YAML is not reproducible under ``autocrlf``). Value enforcement is the
goldens' job.

**What the goldens CANNOT see — the gap this gate owns: completeness.** A newly added
param file or flow class wired into no committed golden is invisible to every byte
compare. So the gate asserts the *set* of the frozen surface against the live tree:

* the param-file set (the five sibling + three station ``params/*.yaml``) equals the
  manifest's param set — catches "added a param, forgot to freeze it";
* the flow-class set, **derived from freshly assembled canonical registries** (the four
  standalone siblings + the maximal sealed **fast** registry — never hand-listed),
  equals the manifest's flow set — catches "added a flow, forgot to freeze it";
* the two sealed horizons equal the importable ``SEALED_STATION_YEARS`` /
  ``SEALED_ENERGY_YEARS`` constants;
* every golden, param, and delegated manifest the station manifest names exists on disk.

**The ``with_harvest=True`` in the flow derivation is load-bearing.** The default
``build_sealed_station`` sets ``with_harvest=False`` (harvest starves the annual
re-sow), so a default-only derivation would silently drop ``Harvest`` — the one flow the
gate most needs to catch. And the five *dropped* stand-ins (``HeatInput`` /
``CrewMetabolism`` / ``OxygenConsumption`` / ``FoodMetabolism`` / ``SelfDischarge``)
live only in the **standalone** sibling builds (pinned by the standalone sibling
goldens), which is why the derivation unions those too, not just the coupled fast
registry. The biosphere's slow registry is **never** included (delegated), so no
biosphere flow leaks into the set.

The integrator (Euler) + the per-scenario dt have **no importable constant** (each run
helper selects them inline), so they are *documented* in the manifest and *enforced* by
the goldens, not asserted here. Regeneration is a deliberate, separate ``__main__``
action (the golden discipline): on an advisor-reviewed unfreeze, run
``uv run python tests/test_station_freeze_manifest.py`` and review the manifest diff.
Zero ``simcore`` change, zero ``domains`` change (docs + tests only).
"""

import hashlib
import json
import sys
from pathlib import Path
from typing import Any

import domains.crew
import domains.eclss
import domains.power
import domains.thermal
import station
from domains.crew.loader import load_crew_params
from domains.crew.system import build_crew
from domains.eclss.loader import load_eclss_params
from domains.eclss.system import build_eclss
from domains.power.loader import load_charge_params, load_self_discharge_params
from domains.power.system import build_power
from domains.thermal.loader import load_thermal_params
from domains.thermal.system import build_thermal
from station.loader import (
    load_harvest_params,
    load_lamp_params,
    load_water_recovery_params,
)
from station.scenario import SEALED_ENERGY_YEARS, SEALED_STATION_YEARS
from station.sealed import build_sealed_station

_REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = _REPO_ROOT / "docs" / "station-reference.manifest.json"
BIOSPHERE_MANIFEST = "docs/biosphere-reference.manifest.json"

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"

# The five sibling + one station params directories the station reference freezes. The
# biosphere params dir is deliberately ABSENT — the biosphere is delegated to its own
# manifest (see the module docstring). No exclusions are needed (no ``demo.yaml``-style
# skeleton lives in any of these dirs).
PARAM_DIRS: list[Path] = [
    Path(domains.power.__file__).parent / "params",
    Path(domains.thermal.__file__).parent / "params",
    Path(domains.eclss.__file__).parent / "params",
    Path(domains.crew.__file__).parent / "params",
    Path(station.__file__).parent / "params",
]

# Scenario key -> (human scenario label, golden filename). The five standalone sibling
# goldens + the eight station-step goldens = the 13 frozen station/sibling scenarios.
# The labels are documentation; the golden filenames are the frozen artifacts.
_SCENARIOS: dict[str, tuple[str, str]] = {
    "power_bounded_soc": (
        "BOUNDED_SOC_SCENARIO (standalone Power)",
        "power_state.json",
    ),
    "power_self_discharge": (
        "SELF_DISCHARGE (standalone Power + SelfDischarge)",
        "power_self_discharge_state.json",
    ),
    "thermal_equilibrium": (
        "EQUILIBRIUM_SCENARIO (standalone Thermal)",
        "thermal_state.json",
    ),
    "eclss_steady_state": (
        "STEADY_STATE_SCENARIO (standalone ECLSS)",
        "eclss_state.json",
    ),
    "crew_mission": ("MISSION_SCENARIO (standalone Crew)", "crew_state.json"),
    "station_heat_closure": (
        "HEAT_CLOSURE_SCENARIO (P6.1 Power→Thermal heat closure)",
        "station_state.json",
    ),
    "cabin_gas": ("CABIN_GAS_SCENARIO (P6.2 crew↔ECLSS)", "cabin_gas_state.json"),
    "greenhouse": (
        "GREENHOUSE_SCENARIO (P6.3 biosphere↔cabin)",
        "greenhouse_state.json",
    ),
    "water_recovery": (
        "WATER_RECOVERY_SCENARIO (P6.4 crew water loop)",
        "water_recovery_state.json",
    ),
    "lighting": (
        "LIGHTING_SCENARIO (P6.5 Power→biosphere lamp)",
        "lighting_state.json",
    ),
    "harvest": ("HARVEST_SCENARIO (P6.6 biomass→food)", "harvest_state.json"),
    "sealed_station": (
        "SEALED_STATION_SCENARIO (P6.7 Tier-2 combined-ledger multi-year)",
        "sealed_station_state.json",
    ),
    "sealed_energy_drift": (
        "HEAT_CLOSURE_SCENARIO 15-yr (P6.7 Tier-1 energy stability signature)",
        "sealed_energy_drift_summary.json",
    ),
}


def _normalized_sha256(path: Path) -> str:
    """sha-256 over newline-normalized (LF) content — a reproducible provenance hash.

    Hashing raw bytes would make the value depend on the checkout's line endings
    (``autocrlf`` on Windows vs. LF on Linux). Normalizing to LF first makes the hash a
    stable record of *content*. Provenance, not a gate — value enforcement is the
    scenario goldens (see the module docstring).
    """
    text = path.read_text(encoding="utf-8")
    normalized = "\n".join(text.splitlines())
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def _param_paths() -> dict[str, Path]:
    """Map each frozen param filename -> its path (names are unique across the dirs).

    The station reference's eight param files: ``charge`` / ``self_discharge`` (power),
    ``radiator`` (thermal), ``eclss`` (eclss), ``crew`` (crew), ``water_recovery`` /
    ``lamp`` / ``harvest`` (station). Every name is unique across ``PARAM_DIRS``, so a
    flat name→path map is unambiguous.
    """
    paths: dict[str, Path] = {}
    for directory in PARAM_DIRS:
        for yaml in directory.glob("*.yaml"):
            paths[yaml.name] = yaml
    return paths


def _param_names_on_disk() -> set[str]:
    """The frozen param-file names present across the live ``PARAM_DIRS``."""
    return {yaml.name for directory in PARAM_DIRS for yaml in directory.glob("*.yaml")}


def _station_registries() -> list[object]:
    """The canonical registries the station flow/aux sets are derived from.

    The four **standalone** sibling registries (so the dropped stand-ins ``HeatInput`` /
    ``CrewMetabolism`` / ``OxygenConsumption`` / ``FoodMetabolism`` / ``SelfDischarge``,
    pinned only by the standalone goldens, appear) **plus** the maximal sealed **fast**
    registry (``with_harvest=True`` so ``Harvest`` — dropped by the default sealed
    build — appears). The sealed build's biosphere-slow registry (index 1) is
    **omitted**: the biosphere is delegated to its own manifest, so no biosphere flow
    leaks into the set.
    """
    charge = load_charge_params()
    self_discharge = load_self_discharge_params()
    thermal = load_thermal_params()
    eclss = load_eclss_params()
    crew = load_crew_params()
    recovery = load_water_recovery_params()
    lamp = load_lamp_params()
    harvest = load_harvest_params()

    power_reg = build_power(charge, self_discharge_params=self_discharge)[1]
    thermal_reg = build_thermal(thermal)[1]
    eclss_reg = build_eclss(eclss)[1]
    crew_reg = build_crew(crew)[1]
    # index [2] is the fast registry (state, bio_reg, fast_reg); bio_reg is delegated.
    sealed_fast_reg = build_sealed_station(
        charge, thermal, crew, eclss, recovery, lamp, harvest, with_harvest=True
    )[2]
    return [power_reg, thermal_reg, eclss_reg, crew_reg, sealed_fast_reg]


def _flow_set() -> list[str]:
    """The frozen station+sibling flow-class set — derived, never hand-listed.

    The union of ``type(flow).__name__`` across the canonical registries. A flow class
    added to any sibling or the station assembly but wired into no golden still appears
    here, so the completeness gate catches it.
    """
    return sorted(
        {
            type(flow).__name__
            for registry in _station_registries()
            for flow in registry.flows  # type: ignore[attr-defined]
        }
    )


def _aux_set() -> list[str]:
    """The frozen aux-process-class set — symmetric with :func:`_flow_set`.

    Derived from the public read-only ``registry.aux_processes``. The siblings + station
    carry no aux process (all conserved-quantity flows) — the biosphere's
    ``ThermalTimeAccumulation`` lives in the delegated slow registry — so this is empty
    today, but freezing the *set* catches a future aux process added but wired into no
    golden.
    """
    return sorted(
        {
            type(proc).__name__
            for registry in _station_registries()
            for proc in registry.aux_processes  # type: ignore[attr-defined]
        }
    )


def _build_manifest() -> dict[str, object]:
    """Assemble the manifest from the live tree — the single source for regeneration."""
    param_paths = _param_paths()
    scenarios: dict[str, object] = {}
    for key, (label, golden) in _SCENARIOS.items():
        scenarios[key] = {
            "scenario": label,
            "golden": golden,
            "golden_sha256": _normalized_sha256(GOLDEN_DIR / golden),
        }
    return {
        "_comment": (
            "Phase-6 Step-10 station freeze manifest (P6.10). Names the frozen "
            "WHOLE-ASSEMBLY station reference surface (Phase-5 siblings + the station "
            "seams); the biosphere is delegated to "
            "docs/biosphere-reference.manifest.json (see delegates_to). See "
            "docs/station-reference.md for the freeze contract + the unfreeze "
            "discipline. Hashes are newline-normalized sha-256 PROVENANCE "
            "(value enforcement is the scenario goldens). Regenerate on a deliberate "
            "unfreeze: uv run python tests/test_station_freeze_manifest.py."
        ),
        "frozen_at_phase": 6,
        "reference_doc": "docs/station-reference.md",
        "delegates_to": BIOSPHERE_MANIFEST,
        "integrator": "EulerIntegrator",
        "numerics_note": (
            "Euler everywhere; dt per scenario (enforced by goldens, no importable "
            "constant). Sealed reference: biosphere-slow dt=1 day + everything-fast "
            "dt=60 s; Tier-1 energy single-rate dt=3600 s."
        ),
        "sealed_station_years": SEALED_STATION_YEARS,
        "sealed_energy_years": SEALED_ENERGY_YEARS,
        "flow_set": _flow_set(),
        "aux_set": _aux_set(),
        "param_files": {
            name: _normalized_sha256(param_paths[name]) for name in sorted(param_paths)
        },
        "scenarios": scenarios,
    }


def _manifest_dumps(manifest: dict[str, object]) -> str:
    """Serialize the manifest to canonical JSON — the project golden discipline.

    ``indent=2, sort_keys=True`` + a trailing newline, matching ``sim_io.dumps`` and the
    biosphere manifest, so it reads and diffs like every other committed snapshot.
    """
    return json.dumps(manifest, indent=2, sort_keys=True) + "\n"


def _load_manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


# --- the completeness gate (what the goldens are blind to) -------------------


def test_frozen_station_param_set_is_complete() -> None:
    # Every sibling + station param file is frozen, and the manifest names no param that
    # has vanished. Catches an added-but-unfrozen param the scenario goldens can't see.
    manifest = _load_manifest()
    assert set(manifest["param_files"]) == _param_names_on_disk()


def test_frozen_station_flow_set_is_complete() -> None:
    # The manifest's flow set equals the flow classes assembled across the four
    # standalone siblings + the maximal sealed fast registry — derived, not hand-listed.
    manifest = _load_manifest()
    assert set(manifest["flow_set"]) == set(_flow_set())


def test_frozen_flow_set_covers_the_four_station_seams() -> None:
    # An explicit guard on the advisor's trap: the four station-owned seams must all be
    # in the frozen flow set (the with_harvest=True derivation is what makes Harvest
    # appear; a default-only derivation would silently drop it).
    frozen = set(_load_manifest()["flow_set"])
    assert {"CrewRespiration", "WaterRecovery", "Lamp", "Harvest"} <= frozen


def test_frozen_station_aux_set_is_complete() -> None:
    # The manifest's aux set equals the aux-process classes across the canonical
    # registries (empty today — the biosphere aux is delegated). Catches a new aux.
    manifest = _load_manifest()
    assert set(manifest["aux_set"]) == set(_aux_set())


def test_completeness_gate_detects_an_unfrozen_param(monkeypatch, tmp_path) -> None:
    # Teeth: the gate is plain set equality, so an unfrozen file on disk must break it.
    # Seed a temp params dir with the frozen names + one phantom, point the dirs there,
    # and confirm the comparison no longer holds. The real params dirs are untouched.
    frozen = set(_load_manifest()["param_files"])
    for name in frozen:
        (tmp_path / name).touch()
    (tmp_path / "phantom.yaml").touch()
    monkeypatch.setattr(sys.modules[__name__], "PARAM_DIRS", [tmp_path])
    assert _param_names_on_disk() != frozen  # the phantom is detected — teeth


def test_manifest_horizons_match_constants() -> None:
    # The two frozen sealed horizons track their single importable sources of truth, so
    # the manifest cannot silently disagree with the sealed goldens' horizons.
    manifest = _load_manifest()
    assert manifest["sealed_station_years"] == SEALED_STATION_YEARS
    assert manifest["sealed_energy_years"] == SEALED_ENERGY_YEARS


def test_manifest_named_files_exist() -> None:
    # Every golden + param the manifest names, and the delegated biosphere manifest, is
    # present on disk — a renamed/deleted frozen artifact fails here, not as a mystery
    # load error later.
    manifest = _load_manifest()
    param_paths = _param_paths()
    for name in manifest["param_files"]:
        assert name in param_paths and param_paths[name].is_file(), name
    for entry in manifest["scenarios"].values():
        assert (GOLDEN_DIR / entry["golden"]).is_file(), entry["golden"]
    assert (_REPO_ROOT / manifest["delegates_to"]).is_file()


def test_manifest_delegates_to_biosphere() -> None:
    # The biosphere is frozen SEPARATELY (Phase 4); the station reference references it,
    # never re-hashes it. Pin that the delegation pointer is the biosphere manifest.
    manifest = _load_manifest()
    assert manifest["delegates_to"] == BIOSPHERE_MANIFEST


def test_manifest_declares_locked_integrator() -> None:
    # The integrator has no importable constant (selected inline in each run helper), so
    # it lives as a documented string; the goldens enforce it (an RK4 switch moves every
    # golden). This pins that the manifest *records the lock*.
    manifest = _load_manifest()
    assert manifest["integrator"] == "EulerIntegrator"


def _regenerate() -> None:
    """Rewrite the committed station manifest from the current live tree.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_station_freeze_manifest.py

    Review the diff before committing: a change means the frozen station surface moved
    (a new flow / param / scenario, a moved sealed horizon, or a frozen file's content),
    i.e. an **unfreeze**, which the discipline in docs/station-reference.md governs.
    Written via ``write_bytes`` (explicit LF, like the goldens) so the manifest is
    byte-stable across platforms.
    """
    MANIFEST_PATH.write_bytes(_manifest_dumps(_build_manifest()).encode("utf-8"))
    print(f"wrote {MANIFEST_PATH}")


if __name__ == "__main__":
    _regenerate()
