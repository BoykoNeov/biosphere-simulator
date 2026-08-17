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

⚠⚠ **SLICE 7 OF THE REFERENCE FLIP (2026-08-16) CHANGED WHO PRODUCES THIS MANIFEST, AND
THE HEADLINE IS *MIXED AUTHORITY*, NOT "IT IS RUST-ANCHORED NOW."** This is slice 6's
ceremony, one contract over. The keys the Rust reference tree can produce are spliced in
from it — ``flow_set``, ``aux_set``, the two sealed horizons — by shelling
``cargo run --example dump_station_inventory``. **Everything else is still Python's or
hand-written**, and by content that is most of the file: ``science_bands`` +
``liveness_floors`` are a static census of pytest markers with no Rust referent.
⚠ ``param_files`` **joined the spliced set in slice C8**, after slice C1 gave the
reference the YAML loaders — and what re-anchored there is the *census* rule and the
*normalization* rule, not the digits, which are author-neutral either way. Each key says
which side it comes from, and why, in the manifest's own ``_authority`` block
(:data:`_AUTHORITY`).

Four consequences worth stating where they will be read:

* **Regeneration now requires ``cargo``**, because it now reads the reference. The tests
  do not: nothing in this module shells cargo, and the base suite stays offline-clean.
  The cargo-side staleness gates live in ``tests/crossport/test_inventory_parity.py``,
  on the ``crossport`` CI job.
* **The completeness gates below changed meaning without changing a line of their
  arithmetic.** ``set(manifest["flow_set"]) == set(_flow_set())`` used to say *the
  manifest froze everything Python has*; it now says *Python still matches the
  reference*. Same assertion, opposite reading, and what it reports is a **Python**
  drift. The same flip reaches :func:`test_manifest_horizons_match_constants` and
  :func:`test_frozen_flow_set_covers_the_four_station_seams`, whose four hand literals
  are now asserted against a Rust-derived set.
* ⚠⚠ **The ``aux_set`` axis is the one this slice had to work for.** It is legitimately
  ``[]``, so every assertion about it is ``[] == []`` — and after this slice that empty
  list is *written into the frozen manifest* from the Rust dump rather than merely
  compared. Slice 6's two-direction rename control cannot be run on it (there is no
  station aux process to rename), so the substitute is recorded in the dump example and
  was measured when this landed: wire one in temporarily, and the regenerated manifest
  gains the name while the Python conformance gate goes red.
* ⚠ **The station's dt is prose and stays prose.** ``numerics_note`` carries the steps
  as hand-maintained English that nothing checks — this module has said so since
  2026-08-14. The reference tree *does* have referents for those numbers, so slice 6's
  ``dt_days`` treatment is buildable here; it needs a structured manifest key that does
  not exist, and adding one widens the frozen surface, which is its own unfreeze rather
  than a rider on this one. Recorded, not closed — see that key's ``_authority`` entry.
"""

import hashlib
import json
import subprocess
import sys
from functools import lru_cache
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
from science_gates import FIELDS, gates_for
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


#: This manifest's scenario roster, as a set — the filter for the science-gate fields.
_ROSTER = frozenset(_SCENARIOS)


#: The reference tree's own dump of the keys it can author — the producer half of this
#: manifest since slice 7. Its doc comment is the authority on what it emits and why.
_RUST_CRATE_DIR = _REPO_ROOT / "rust" / "crates" / "station"
_RUST_DUMP_EXAMPLE = "dump_station_inventory"

#: The keys :func:`_build_manifest` consumes out of that dump, asserted as its **exact**
#: key set. ⚠ A forcing function, not a filter (slice 3's move, kept through slices 6, 7
#: and C8): a key added to the dump turns regeneration into a loud error rather than
#: silently entering — or silently *not* entering — the frozen surface.
#:
#: ⚠⚠ ``param_files`` joined in **slice C8**, and this comment used to say it *must not
#: reach the manifest this way*. That was right for its day: the port read
#: ``sibling_params.txt`` / ``station_params.txt``, tables Python generated from the
#: Python
#: loaders, so any list printed here would have been this one travelling out and back.
#: Slice
#: **C1** moved the loaders and **C8** the census + digest. See :data:`_AUTHORITY`.
_RUST_DUMP_KEYS = frozenset({"flow_set", "aux_set", "horizons", "param_files"})


@lru_cache(maxsize=1)
def _rust_reference() -> dict[str, Any]:
    """Run the reference tree's inventory dump and parse its JSON.

    ⚠ **Called only from :func:`_build_manifest`, i.e. only from the regeneration
    ``__main__``.** No test in this module reaches it, so the base suite neither needs
    ``cargo`` nor pays for a build. The gates that *do* compare this manifest against a
    live Rust tree are cargo-gated and live in
    ``tests/crossport/test_inventory_parity.py``.
    """
    proc = subprocess.run(
        ["cargo", "run", "-q", "--example", _RUST_DUMP_EXAMPLE],
        cwd=_RUST_CRATE_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"cargo run --example {_RUST_DUMP_EXAMPLE} failed — the manifest is "
            f"regenerated FROM the Rust reference since slice 7 of the reference flip, "
            f"so regeneration needs a working Rust toolchain:\n{proc.stderr}"
        )
    dump: dict[str, Any] = json.loads(proc.stdout)
    if set(dump) != _RUST_DUMP_KEYS:
        raise SystemExit(
            f"{_RUST_DUMP_EXAMPLE} emitted {sorted(dump)}, expected "
            f"{sorted(_RUST_DUMP_KEYS)}. Read _AUTHORITY before widening this: a new "
            "key has to be classified, and one that cannot honestly come from the "
            "reference (a pytest-marker census) must not enter the manifest through "
            "here."
        )
    return dump


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
    """The **checker's** station+sibling flow-class set — derived, never hand-listed.

    The union of ``type(flow).__name__`` across the canonical registries. A flow class
    added to any sibling or the station assembly but wired into no golden still appears
    here, so the completeness gate catches it.

    ⚠ **Since slice 7 this no longer produces the manifest — it checks against it.** The
    frozen ``flow_set`` is the reference tree's ``Flow::type_name()`` union, spliced in
    by :func:`_build_manifest`; this walk is what says *Python still matches*. Same
    code, opposite direction, and a disagreement is now a **Python** finding.
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

    ⚠⚠ **Empty on both sides since slice 7, which is why this axis needed a control
    rather than a green run.** The manifest's ``aux_set`` is now the reference tree's
    walk, so ``[] == []`` here is satisfied by two sides that both never reached the
    accessor. Measured when slice 7 landed: wiring one aux process into a canonical Rust
    station build makes the regenerated manifest gain the name **and** turns
    :func:`test_frozen_station_aux_set_is_complete` red. That is the evidence; this
    equality is not.
    """
    return sorted(
        {
            type(proc).__name__
            for registry in _station_registries()
            for proc in registry.aux_processes  # type: ignore[attr-defined]
        }
    )


#: Per-**path** authority: which side each field of the frozen surface comes from, and
#: why. Written into the manifest itself (as ``_authority``) so the file states its own
#: mixed provenance to whoever opens it — slice 7 of the reference flip, mirroring what
#: slice 6 did for the biosphere.
#:
#: ⚠ **Keyed by path, not top-level key, because ``scenarios`` split *inside itself*.**
#: Through slice 7 twelve of the thirteen goldens were the reference tree's own output
#: and the thirteenth, ``sealed_energy_drift_summary.json``, was ``drift.py``'s
#: Python-side **fold** of a raw Rust series — the one-run-two-authors shape the
#: biosphere manifest still carries for ``drift_summary``. A top-level classification
#: would have hidden exactly that.
#:
#: ⚠ **Slice C5 closed that split and the per-path keying is KEPT anyway.** All thirteen
#: are now Rust's, so ``scenarios/*/golden_sha256`` has no exception left; the structure
#: stays because the reason it was introduced — a contract axis that is not uniform — is
#: a property this file must be able to express, not one it happens not to need today.
#:
#: ⚠ The three sides are claims of different kinds. ``rust`` — produced by the reference
#: tree and spliced in by :func:`_build_manifest`. ``python`` — produced by the checker
#: because the reference has no referent for it, with the reason stated and, where one
#: exists, the condition under which that could change. ``hand`` — a literal or a label
#: deliberately derived from neither, because a contract field that imports its own
#: value auto-follows the code.
_AUTHORITY: dict[str, dict[str, str]] = {
    "_comment": {"side": "hand", "why": "prose header"},
    "frozen_at_phase": {"side": "hand", "why": "the phase this surface froze at"},
    "reference_doc": {
        "side": "hand",
        "why": "pointer to the prose half of the contract",
    },
    "delegates_to": {
        "side": "hand",
        "why": (
            "pointer to the biosphere manifest, which this contract delegates rather "
            "than re-hashes. A path, not a derived value — its target's existence is "
            "checked by test_manifest_named_files_exist"
        ),
    },
    "integrator": {
        "side": "hand",
        "why": (
            "the deliberate anti-derived literal, and unlike the biosphere's dt_days "
            "it has no importable constant on EITHER side — each run helper selects "
            "the scheme inline — so it is documented here and enforced by the goldens "
            "(an "
            "RK4 switch moves every one). A literal typed into the Rust dump to make "
            "the pair symmetric would read like a gate and be none."
        ),
    },
    "numerics_note": {
        "side": "hand",
        "why": (
            "⚠ HAND-MAINTAINED PROSE THAT NOTHING CHECKS, and slice 7 deliberately "
            "left it that way. Unlike the biosphere the station has no structured dt "
            "key: "
            "the steps live inside this English sentence, so flipping one reddens "
            "nothing here. The reference tree DOES have referents — "
            "sealed_station_scenario()'s bio_dt / cabin_dt and the energy scenario's "
            "power_dt — so slice 6's dt_days treatment is buildable. It needs a "
            "structured key that does not exist, and adding one WIDENS the frozen "
            "surface, which is its own unfreeze with its own ceremony rather than a "
            "rider on a re-anchoring. Recorded here so the hole is a stated claim."
        ),
    },
    "sealed_station_years": {
        "side": "rust",
        "why": "the reference tree's SEALED_STATION_YEARS",
    },
    "sealed_energy_years": {
        "side": "rust",
        "why": (
            "the reference tree's SEALED_ENERGY_YEARS. ⚠ In that tree it is DEFINED as "
            "LONG_HORIZON_YEARS, so since slice 7 this contract and the biosphere's "
            "are anchored to one reference-side constant: moving the decade horizon "
            "is a single edit that unfreezes two manifests. A reader who assumes they "
            "are "
            "independent will predict the wrong diff."
        ),
    },
    "flow_set": {
        "side": "rust",
        "why": (
            "the union of Flow::type_name() over the canonical station registries in "
            "the reference tree — the four standalone siblings plus the maximal sealed "
            "fast registry, derived from built registries and never hand-listed"
        ),
    },
    "aux_set": {
        "side": "rust",
        "why": (
            "the same walk over AuxProcess::type_name(). ⚠ Legitimately EMPTY (the "
            "siblings and seams are all conserved-quantity flows; the biosphere's "
            "accumulators live in the delegated slow registry), so every assertion "
            "about it is [] == [] and the splice is what a regeneration writes. The "
            "evidence that the walk happens at all is a measured control, not a green "
            "run — see the dump example and the module docstring."
        ),
    },
    "param_files/*": {
        "side": "rust",
        "why": (
            "⚠ RE-ANCHORED IN SLICE C8, the same finding as the biosphere's: what moved"
            " "
            "is the RULE, not the number. All 8 digests here (and all 15 there) are "
            "author-neutral — both sides hash the same file the same way — so the "
            "re-anchoring moved none of them. The CENSUS is now the set the reference "
            "LOADS: domains::params::param_files (power × 2, thermal, eclss, crew) plus"
            " "
            "station::params::param_files (water_recovery, lamp, harvest), eight "
            "compile-time include_str! entries instead of a glob over six Python "
            "package "
            "directories. ⚠ NO exclusion rule on this side, unlike the biosphere's "
            "15-of-20 — these six directories hold nothing but frozen files, and the "
            "asymmetry is stated per side so nobody generalises the harder rule. The "
            "NORMALIZATION is config::provenance (hand-rolled sha-256 over "
            "LF-normalized "
            "text; every engine crate is zero-dep by charter). ⚠ Newly asserted with "
            "the "
            "re-anchor: every basename is unique across the six directories — this key "
            "is basename-KEYED, so a collision would silently collapse two files into "
            "one entry, and nothing had checked it. Prerequisite: slice C1."
        ),
    },
    "science_bands/*": {
        "side": "python",
        "why": (
            "a static AST census of science_gate markers on pytest functions "
            "(tests/science_gates.py). There is no Rust referent and there cannot be "
            "one while the science gates are pytest-side. ⚠ On this manifest the "
            "census is mostly EMPTY — 11 of 13 scenarios carry no outside-sourced "
            "bound — and "
            "the emptiness is itself the frozen claim."
        ),
    },
    "liveness_floors/*": {
        "side": "python",
        "why": "the same census, for the bounds tuned to our own calibration",
    },
    "scenarios/*/scenario": {
        "side": "hand",
        "why": "a human label for the scenario, not an identifier anything resolves",
    },
    "scenarios/*/golden": {"side": "hand", "why": "the artifact's filename"},
    "scenarios/*/golden_sha256": {
        "side": "rust",
        "why": (
            "the golden is the reference tree's own output (golden_platform."
            "RUST_AUTHORED, which this block is checked against, not restating). "
            "Unlike the param hashes here this one IS gated against the file on disk: "
            "a golden is machine-generated and its hash is newline-normalized, so 'the "
            "manifest "
            "pins bytes that exist' is a completeness claim, not the value "
            "re-assertion that param_files declines. "
            "⚠ SLICE C5 removed this key's ONE exception and that is why the axis is "
            "now uniform. `scenarios/sealed_energy_drift/golden_sha256` used to be "
            "carved out as `python` — 'ONE RUN, TWO AUTHORS: drift.py's Python-side "
            "fold of the 15-yr sealed energy series; the fold IS the artifact, so its "
            "correct reference is Python's own output.' That stopped being true when "
            "`domains::biosphere::drift` gained the fold kit and "
            "`emit_sealed_energy_drift` began emitting the summary itself. ⚠ The HASH "
            "did not move (measured byte-identical before the change), so this is an "
            "authorship re-anchoring, not a value unfreeze — the same shape C8 found "
            "for param_files, where the digits were author-neutral and the RULE moved."
        ),
    },
}


def _build_manifest() -> dict[str, object]:
    """Assemble the manifest — the reference tree's keys spliced into the checker's.

    ⚠ Since slice 7 this reads the **Rust** tree for everything :data:`_AUTHORITY` marks
    ``rust``, so it needs ``cargo``. It is reachable only from :func:`_regenerate`.
    """
    reference = _rust_reference()
    horizons = reference["horizons"]
    scenarios: dict[str, object] = {}
    for key, (label, golden) in _SCENARIOS.items():
        scenarios[key] = {
            "scenario": label,
            "golden": golden,
            "golden_sha256": _normalized_sha256(GOLDEN_DIR / golden),
        }
    return {
        "_authority": _AUTHORITY,
        "_comment": (
            "Phase-6 Step-10 station freeze manifest (P6.10). Names the frozen "
            "WHOLE-ASSEMBLY station reference surface (Phase-5 siblings + the station "
            "seams); the biosphere is delegated to "
            "docs/biosphere-reference.manifest.json (see delegates_to). See "
            "docs/station-reference.md for the freeze contract + the unfreeze "
            "discipline. Hashes are newline-normalized sha-256 PROVENANCE "
            "(value enforcement is the scenario goldens). Each key's producer and why "
            "is in _authority: this file has MIXED authority since slice 7 of the "
            "reference flip. Regenerate on a deliberate unfreeze: uv run python "
            "tests/test_station_freeze_manifest.py — which now shells cargo, because "
            "the _authority 'rust' keys are read from the reference tree."
        ),
        "frozen_at_phase": 6,
        "reference_doc": "docs/station-reference.md",
        "delegates_to": BIOSPHERE_MANIFEST,
        "integrator": "EulerIntegrator",
        # ⚠ HAND-MAINTAINED PROSE, AND NOTHING CHECKS IT. This literal is compared only
        # against the on-disk manifest generated from this same literal, so the two
        # agree whatever the code does — flipping bio_dt reddens nothing here. The
        # biosphere contract is different: test_freeze_manifest.py asserts dt_days
        # against a hard-coded number and fails loudly, and since slice 6 that literal
        # is checked against the REFERENCE tree's BIO_DT too. Do not assume either gate
        # on that side covers this one. Update this string by hand as part of any step
        # ceremony (done 2026-08-14, dt=1 -> dt=1/4).
        #
        # ⚠ Slice 7 looked at closing this and deliberately did not — the referents do
        # exist in the reference tree (sealed_station_scenario()'s bio_dt / cabin_dt,
        # the energy scenario's power_dt), but the gate needs a structured manifest key
        # that does not exist, and adding one widens the frozen surface. See _AUTHORITY.
        "numerics_note": (
            "Euler everywhere; dt per scenario (enforced by goldens, no importable "
            "constant). Sealed reference: biosphere-slow dt=1/4 day, 4 slow sub-steps "
            "per master day + everything-fast dt=60 s; Tier-1 energy single-rate "
            "dt=3600 s."
        ),
        "sealed_station_years": horizons["sealed_station_years"],
        "sealed_energy_years": horizons["sealed_energy_years"],
        "flow_set": reference["flow_set"],
        "aux_set": reference["aux_set"],
        # ⚠⚠ SLICE C8: spliced from the REFERENCE, not derived here (`param_paths`
        # is retained as the conformance check on the checker — see
        # test_the_frozen_param_set_matches_the_reference_census). Same finding as the
        # biosphere's: the digests are author-neutral, the CENSUS and NORMALIZATION
        # rules are what moved. See _AUTHORITY.
        "param_files": reference["param_files"],
        "science_bands": gates_for(_ROSTER, "science_bands"),
        "liveness_floors": gates_for(_ROSTER, "liveness_floors"),
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
    """⚠ Same assertion since Phase 6, opposite reading since slice C8.

    It used to say *the manifest froze every param file on disk*. ``param_files`` is now
    spliced from the **reference's** census — the set it actually loads — so this says
    *the checker's directory rule still agrees with what the reference reads*. A failure
    is a roster finding with two different repairs: a file added and wired into no Rust
    loader, or a loader dropped with its file left behind.
    """
    manifest = _load_manifest()
    assert set(manifest["param_files"]) == _param_names_on_disk(), (
        "the frozen param census and these directories disagree. Since slice C8 the "
        "manifest names the files the REFERENCE loads, so the finding is a roster one. "
        "Do not 'fix' it by regenerating."
    )


def test_the_frozen_station_param_hashes_match_the_checker_s_own_digest() -> None:
    """⚠⚠ The retained Python digest, kept as a **conformance check on the checker**.

    The twin of the biosphere's. Slice C8 moved the census and the normalization rule to
    the reference; deleting the Python side would have thrown away the only thing that
    says the two rules still agree, so :func:`_normalized_sha256` stays with its meaning
    inverted — the identical assertion now asks *has the checker's hashing drifted from
    the contract?*

    ⚠ And the honest boundary: these 8 values are **author-neutral**, so a green run is
    evidence about the two rules agreeing, not about who owns the number.
    """
    manifest = _load_manifest()
    paths = _param_paths()
    mismatched = {
        name: (recorded, _normalized_sha256(paths[name]))
        for name, recorded in manifest["param_files"].items()
        if name in paths and recorded != _normalized_sha256(paths[name])
    }
    assert not mismatched, (
        "the checker's newline-normalized sha-256 disagrees with the frozen "
        f"(reference-produced) one for {sorted(mismatched)}. Either a param file was "
        "edited without the unfreeze ceremony, or the two normalization rules have "
        "diverged — check config::provenance::normalize_newlines against "
        "_normalized_sha256 before touching the manifest."
    )


def test_frozen_station_flow_set_is_complete() -> None:
    # ⚠ A PYTHON-CONFORMANCE check since slice 7, not a completeness derivation. The
    # arithmetic is unchanged; the reading is not. The frozen set is now the reference
    # tree's, so a failure means *the checker* has a flow the reference does not (or
    # lacks one it has) — not "someone added a flow and forgot to freeze it". That
    # completeness question moved to tests/crossport/test_inventory_parity.py, on the
    # side that now owns it.
    manifest = _load_manifest()
    assert set(manifest["flow_set"]) == set(_flow_set()), (
        "Python's station flow classes no longer match the frozen (Rust-derived) set. "
        "Since slice 7 of the reference flip this is a CHECKER drift: the reference "
        "tree is docs/station-reference.manifest.json's producer."
    )


def test_frozen_flow_set_covers_the_four_station_seams() -> None:
    # An explicit guard on the advisor's trap: the four station-owned seams must all be
    # in the frozen flow set (the with_harvest=True derivation is what makes Harvest
    # appear; a default-only derivation would silently drop it).
    #
    # ⚠ Since slice 7 these four hand literals are asserted against a **Rust-derived**
    # set, which is what makes the test worth keeping: it is now the one place a
    # mis-mirrored registry selection in the dump example (with_harvest=false, the
    # fast-vs-slow index) fails by name rather than as a set difference to decode.
    frozen = set(_load_manifest()["flow_set"])
    assert {"CrewRespiration", "WaterRecovery", "Lamp", "Harvest"} <= frozen


def test_frozen_station_aux_set_is_complete() -> None:
    # ⚠⚠ [] == [] ON BOTH SIDES. Since slice 7 the frozen aux set is the reference
    # tree's walk and this is Python's, and the station legitimately has no aux process
    # — so a green here is consistent with neither side ever reaching `aux_processes()`.
    # The evidence that it does is the measured control recorded in `_aux_set`'s
    # docstring and in the dump example; the reason the set is empty is stated in
    # test_inventory_parity.py::test_the_station_aux_axis_is_empty_by_delegation.
    manifest = _load_manifest()
    assert set(manifest["aux_set"]) == set(_aux_set())


def test_frozen_station_science_gates_are_complete() -> None:
    """The station half of the science contract — derived from the tree, as everywhere.

    ⚠ The measured result is mostly EMPTY, and that is the finding rather than a gap:
    11 of the 13 station scenarios carry no outside-sourced bound at all. Established
    mechanically — no station run-test defines a module-level sourced constant. Only
    ``crew_mission`` (BVAD Table 3-31) and ``sealed_station`` (a thermal-node floor)
    have one. Freezing the emptiness is the point: a band cannot be added silently, and
    the absence is now a recorded claim instead of an unexamined assumption.
    """
    manifest = _load_manifest()
    for field in FIELDS:
        assert manifest[field] == gates_for(_ROSTER, field), field
        assert set(manifest[field]) == _ROSTER, field
    assert manifest["science_bands"]["crew_mission"], "the BVAD RQ band went missing"
    assert manifest["liveness_floors"]["sealed_station"], "the node floor went missing"


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
    # ⚠ A conformance check since slice 7, not a derivation check. The frozen horizons
    # are the reference tree's constants; this asserts the checker's still agree with
    # them, so the two ports cannot silently run different sealed horizons.
    #
    # ⚠ `SEALED_ENERGY_YEARS` is `LONG_HORIZON_YEARS` in *both* trees, so this row and
    # the biosphere manifest's `long_horizon_years` are two frozen copies of one
    # reference constant. Moving the decade horizon is one edit and two ceremonies.
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


def _leaf_paths(node: Any, prefix: str = "") -> list[str]:
    """Every leaf path of the manifest, ``/``-joined.

    Dicts recurse; anything else (including a list) is a leaf, because the frozen units
    here are whole lists — ``flow_set`` is one claim, not 16. ``/`` rather than ``.``
    because param-file keys *are* filenames and carry dots.
    """
    if isinstance(node, dict):
        return [p for k, v in node.items() for p in _leaf_paths(v, f"{prefix}{k}/")]
    return [prefix.rstrip("/")]


def _authority_matches(path: str) -> list[tuple[int, str, dict[str, str]]]:
    """Every ``_AUTHORITY`` pattern matching ``path``, with its specificity score."""
    segments = path.split("/")
    matches = []
    for pattern, entry in _AUTHORITY.items():
        parts = pattern.split("/")
        if len(parts) != len(segments):
            continue
        if any(p not in ("*", s) for p, s in zip(parts, segments, strict=True)):
            continue
        matches.append((sum(p != "*" for p in parts), pattern, entry))
    return matches


def _authority_for(path: str) -> tuple[str, dict[str, str]] | None:
    """Resolve a leaf path against :data:`_AUTHORITY`, most specific wins."""
    matches = _authority_matches(path)
    if not matches:
        return None
    _score, pattern, entry = max(matches, key=lambda m: m[0])
    return pattern, entry


def test_every_frozen_field_declares_who_produced_it() -> None:
    """The manifest states its own mixed authority, and the block cannot go stale.

    ⚠ **Checked in both directions**, because each direction fails differently: an
    unclassified field is a frozen value whose producer nobody stated (the thing this
    slice exists to prevent), while a classification pattern matching nothing is a stale
    row describing a field that has been renamed or removed — which reads as coverage
    and is not.
    """
    manifest = _load_manifest()
    classified = {k: v for k, v in manifest.items() if k != "_authority"}
    paths = _leaf_paths(classified)
    unclassified = [p for p in paths if _authority_for(p) is None]
    assert not unclassified, (
        f"frozen fields with no _authority entry: {unclassified}. Every field of this "
        "contract has to say which side produces it — see _AUTHORITY in "
        "tests/test_station_freeze_manifest.py."
    )

    # ⚠ "Most specific wins" only decides anything while no two patterns TIE. Two of
    # equal specificity matching one path would resolve by dict order — a silent answer
    # to a question nobody asked, and the field would read as classified either way.
    # (Until slice C5 `scenarios/sealed_energy_drift/golden_sha256` beat
    # `scenarios/*/golden_sha256` 3-2. That override is gone — the goldens axis is
    # uniform again — so this currently guards a tie that no pair produces. It is kept
    # deliberately: cheap, and the next per-scenario carve-out re-creates exactly the
    # situation it exists for.)
    for path in paths:
        top = max(s for s, _, _ in _authority_matches(path))
        tied = sorted(p for s, p, _ in _authority_matches(path) if s == top)
        assert len(tied) == 1, (
            f"{path} is matched by {len(tied)} _authority patterns of equal "
            f"specificity: {tied}. Which one applies would be decided by dict order — "
            "make one of them strictly more specific."
        )

    matched = {_authority_for(p)[0] for p in paths}  # type: ignore[index]
    stale = sorted(set(manifest["_authority"]) - matched)
    assert not stale, f"_authority patterns matching no field: {stale}"
    assert manifest["_authority"] == _AUTHORITY, (
        "the committed _authority block is not the one this module would write — "
        "regenerate the manifest"
    )


def test_golden_authority_agrees_with_the_rust_authored_roster() -> None:
    """The classification is *checked against* the roster, never a third copy of it.

    ⚠ ``golden_platform.RUST_AUTHORED`` and ``regen_goldens_from_rust.RUST_EMITTERS``
    are already two copies held equal by a gate; writing the names a third time here is
    the hazard this repo keeps re-learning. So the block names sides and *this* test
    ties them to the one roster.

    ⚠ Until slice C5 the case that made this tie worth having was
    ``sealed_energy_drift``, whose golden was a Python fold of a raw Rust series. It is
    Rust's now, so this test currently confirms a uniform axis — and it is exactly what
    FORCED the ``_authority`` update when the roster moved, instead of leaving the block
    stating a classification that had quietly stopped being true.
    """
    from golden_platform import RUST_AUTHORED  # noqa: PLC0415

    manifest = _load_manifest()
    for name, entry in manifest["scenarios"].items():
        side = _authority_for(f"scenarios/{name}/golden_sha256")[1]["side"]  # type: ignore[index]
        expected = "rust" if entry["golden"] in RUST_AUTHORED else "python"
        assert side == expected, (
            f"scenarios/{name}/golden_sha256 is classified {side!r}, but "
            f"{entry['golden']} is {'on' if expected == 'rust' else 'not on'} "
            "golden_platform.RUST_AUTHORED"
        )


def test_manifest_golden_hashes_match_the_files_on_disk() -> None:
    """⚠ The one class of hash here that IS compared — slice 5 measured the hole.

    Regenerating a frozen golden used to desynchronise a manifest **with every gate
    green**: ``golden_sha256`` is assembled inside :func:`_regenerate` and was never
    read back, so the file could pin bytes that no longer existed. Closed for the
    biosphere in slice 6 and for this contract in slice 7 — the station manifest's
    twelve Rust-authored goldens have exactly the same exposure.

    ⚠ Deliberately **goldens only**. The param hashes stay provenance for the reason the
    module docstring gives — they are hand-edited files whose *values* the goldens
    already enforce. A golden is different in kind: machine-generated, it **is** the
    value, and its newline-normalized hash is reproducible on every platform, so "the
    manifest pins bytes that exist" is a completeness claim, not a re-assertion.
    """
    manifest = _load_manifest()
    for entry in manifest["scenarios"].values():
        assert entry["golden_sha256"] == _normalized_sha256(
            GOLDEN_DIR / entry["golden"]
        ), (
            f"{entry['golden']} has moved since the manifest was regenerated. If the "
            "move was intended it is an unfreeze: follow the ceremony in "
            "docs/station-reference.md and regenerate the manifest as its record."
        )


def _regenerate() -> None:
    """Rewrite the committed station manifest from the current live tree.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_station_freeze_manifest.py

    Review the diff before committing: a change means the frozen station surface moved
    (a new flow / param / scenario, a moved sealed horizon, or a frozen file's content),
    i.e. an **unfreeze**, which the discipline in docs/station-reference.md governs.
    Written via ``write_bytes`` (explicit LF, like the goldens) so the manifest is
    byte-stable across platforms.

    ⚠ **Needs ``cargo`` since slice 7**: the keys :data:`_AUTHORITY` marks ``rust`` are
    read out of the reference tree by :func:`_rust_reference`, not derived here. A
    toolchain-less run exits with that message rather than quietly writing a manifest
    with the checker's values in the reference's slots.
    """
    MANIFEST_PATH.write_bytes(_manifest_dumps(_build_manifest()).encode("utf-8"))
    print(f"wrote {MANIFEST_PATH}")


if __name__ == "__main__":
    _regenerate()
