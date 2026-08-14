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
import re
import sys
from pathlib import Path
from typing import Any

import domains.biosphere
from domains.biosphere.light_path import half_sine_window_mean
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
from science_gates import (
    FIELDS,
    REQUIRED_KEYS,
    collect_science_gates,
    gates_for,
    non_decorator_marker_sites,
    unknown_scenarios,
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


#: This manifest's scenario roster — the filter for the science-gate fields, so
#: a gate naming a station scenario cannot silently land in the biosphere manifest.
_ROSTER = frozenset(_SCENARIOS)


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


def _light_path_fingerprint() -> str:
    """A value-level fingerprint of the within-day PAR **shape** (2026-08-14).

    ⚠ **Without this the manifest has a hole the gate cannot see.** ``forcing`` named
    the weather fixture and its hash, and the light path changes how a day's photons
    are *distributed* while leaving that fixture byte-identical — so swapping the
    sinusoid for a top-hat, or moving solar noon, or dropping the ``π/2`` peak factor
    would move every golden while the manifest read exactly the same. That is the
    "field absent from both sides ⇒ gate green" blindness recorded in
    ``multirate-effective-step-is-per-rate-class``, and it is why this is a **sampled
    fingerprint** rather than a prose name: a name records what someone meant, a
    fingerprint records what the code does.

    Sampled on a fixed grid (three day lengths × the day's quarters at the shipped
    step) and hashed as hex-float text, so it is exact rather than tolerance-bound and
    moves on any change to the shape — including one that preserves the daily dose.
    """
    samples = [
        half_sine_window_mean(k * 0.25, 0.25, 400.0, daylength_h * 3600.0).hex()
        for daylength_h in (8.0, 12.0, 16.0)
        for k in range(4)
    ]
    return hashlib.sha256("|".join(samples).encode("utf-8")).hexdigest()


def _canonical_registries() -> list[object]:
    """The assembled registries of the four canonical scenarios (open field + chambers).

    The single place the flow/aux sets are derived from — built fresh from the live
    compartment builders, never hand-listed. The open field carries the
    boundary-atmosphere producer flows; the sealed chambers add the decomposer /
    water-cycle / consumer flows; the union over all four is the complete frozen set.
    """
    scenarios = (
        None,  # the open-field DEFAULT_SCENARIO
        SEALED_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_SCENARIO,
        CONSUMER_CHAMBER_SCENARIO,
    )
    return [(build_season() if sc is None else build_season(sc))[1] for sc in scenarios]


def _flow_set() -> list[str]:
    """The frozen flow-class set: the union of flow classes across the canonical builds.

    **Derived, never hand-listed** (the advisor's lever): collect
    ``type(flow).__name__`` from each canonical registry. A flow class added to any
    compartment builder but wired into no golden still appears here, so the completeness
    gate catches it. (Note: gross assimilation is a recomputed *quantity* inside the
    shared carbon budget, not a standalone flow — it enters via ``Allocation`` — so no
    ``Photosynthesis`` class appears here.)
    """
    return sorted(
        {
            type(flow).__name__
            for registry in _canonical_registries()
            for flow in registry.flows  # type: ignore[attr-defined]
        }
    )


def _aux_set() -> list[str]:
    """The frozen aux-process-class set — the third 'wired into a registry' axis.

    Symmetric with :func:`_flow_set`, derived from the public read-only
    ``registry.aux_processes`` (**zero core change** — a property, not a new accessor).
    Aux processes are the non-conserved accumulators the goldens are otherwise as blind
    to as they are to a wired-but-output-inert flow; today the only one is the
    thermal-time / DVS accumulator, but freezing the *set* means a future aux process
    added but wired into no golden is caught here too.
    """
    return sorted(
        {
            type(proc).__name__
            for registry in _canonical_registries()
            for proc in registry.aux_processes  # type: ignore[attr-defined]
        }
    )


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
        # ⚠ A SECOND hand-maintained literal, and it is not the gate — the gate is the
        # separate literal in test_manifest_declares_locked_integrator_and_dt. The two
        # must be edited together, and the assertion is what forces you to notice: on
        # 2026-08-14 the step moved to 1/4, this line was missed, and the regenerated
        # manifest still read 1.0 until the assertion went red. That is the design
        # working. Do not "simplify" either one to import BIO_DT.
        "dt_days": 0.25,
        "long_horizon_years": LONG_HORIZON_YEARS,
        "flow_set": _flow_set(),
        "aux_set": _aux_set(),
        "forcing": {
            "weather_fixture": WEATHER_FIXTURE.name,
            "weather_sha256": _normalized_sha256(WEATHER_FIXTURE),
            # The within-day PAR shape — see _light_path_fingerprint for why a hash of
            # sampled values rather than a name. Unlike the two hashes above (provenance
            # only, never compared), this one IS gated:
            # test_manifest_pins_the_within_day_light_path.
            "light_path": _light_path_fingerprint(),
        },
        "param_files": {
            name: _normalized_sha256(PARAMS_DIR / name)
            for name in _frozen_param_files()
        },
        "science_bands": gates_for(_ROSTER, "science_bands"),
        "liveness_floors": gates_for(_ROSTER, "liveness_floors"),
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


def test_frozen_aux_set_is_complete() -> None:
    # The manifest's aux set equals the aux-process classes across the canonical
    # scenarios — the third 'wired into a registry' axis (non-conserved accumulators)
    # alongside flows + params. Catches an added-but-unfrozen aux process.
    manifest = _load_manifest()
    assert set(manifest["aux_set"]) == set(_aux_set())


def test_frozen_science_gates_are_complete() -> None:
    """The science half of the contract: bands and floors, derived from the tree.

    Before this field the frozen acceptance set was {golden bytes, ``rationed == 0``, no
    extinction, conservation, determinism} — every one a property of the RUN, not the
    SCIENCE (``post-roadmap-acceptance-gate.md`` finding 5). A committed band could be
    deleted with every gate green. Now it cannot: the manifest is compared against the
    live marker set, so adding, editing or dropping a gate turns this red.
    """
    manifest = _load_manifest()
    for field in FIELDS:
        assert manifest[field] == gates_for(_ROSTER, field), field


def test_every_roster_scenario_has_an_explicit_science_gate_entry() -> None:
    """An absent key and a deliberately-empty one are different claims.

    ``drift_summary`` is the case that forces this: it is a derived stability signature
    over two scenarios that are themselves in the roster, so it carries no gate of its
    own. Recording that as an empty list says "measured, none"; omitting the key would
    say nothing, and a reader using ``.get(name, [])`` could not tell them apart.
    """
    manifest = _load_manifest()
    for field in FIELDS:
        assert set(manifest[field]) == _ROSTER, field
    assert manifest["science_bands"]["drift_summary"] == []
    assert manifest["liveness_floors"]["drift_summary"] == []


def test_no_science_gate_names_a_scenario_outside_both_manifests() -> None:
    """A gate on an unfrozen scenario is claiming standing it cannot have.

    Both rosters are read, not just this one: the fields filter by scenario, so a typo
    a gate on authored content would otherwise be dropped by BOTH manifests in silence —
    the filter looking exactly like a clean result.
    """
    station = json.loads(
        (MANIFEST_PATH.parent / "station-reference.manifest.json").read_text(
            encoding="utf-8"
        )
    )
    known = _ROSTER | frozenset(station["scenarios"])
    assert unknown_scenarios(known) == ()


def test_science_gate_is_decorator_form_only() -> None:
    """The convention that makes static enumeration sound, pinned rather than trusted.

    ``ast`` cannot see a marker applied through ``pytestmark``, a fixture or a
    parametrized indirection. Rather than leave that a silent hole, the decorator
    form is *required*, enforced structurally: every ``mark.science_gate`` attribute
    access in the test tree must sit in a decorator position.

    ⚠ The first version counted the string and failed at 13 vs 10 — **its own docstring
    and code literal were three of the occurrences**. Matching
    ``@pytest.mark.science_gate`` would have silenced that while missing the case worth
    catching (``pytestmark = [...]`` has no ``@``), so the check reads the AST instead.
    """
    assert non_decorator_marker_sites() == (), (
        "a science_gate marker is not in decorator form — the science fields "
        "are enumerated statically and would silently miss it"
    )


def test_science_gate_entries_record_the_claim_not_just_a_test_id() -> None:
    """Each entry carries quantity + bound + source + locus.

    A manifest naming only a test id would freeze *that a test exists*, not *what it
    asserts* — so a bound could be loosened in place with the gate still green. The
    entry is the claim; the locus is where to go read it.
    """
    manifest = _load_manifest()
    for field in FIELDS:
        for entries in manifest[field].values():
            for entry in entries:
                assert set(entry) == set(REQUIRED_KEYS) - {"scenario", "field"} | {
                    "locus"
                }
                assert all(str(v).strip() for v in entry.values())


def test_science_gate_bounds_name_a_literal_present_at_their_locus() -> None:
    """Tie the recorded ``bound`` to the executed one — the retune-in-silence path.

    ⚠ Without this the ``bound`` field is **documentation, not an assertion**: it is
    prose *describing* it, so ``non_collapsing(floor=0.05)`` could be retuned to 0.02 in
    ``test_decade_stability.py`` with the manifest text left stale and every gate green.
    That is exactly what ``liveness_floors`` exists to prevent, and the floors are the
    family that **has already been retuned once** (``> 1.0`` → ``> 0.9``, when the
    decomposer calibration shrank the plant ~19 %).

    Deliberately crude — every numeric literal in ``bound`` must appear textually in the
    file the entry points at. It does not parse the expression, so it cannot prove the
    literal is *the* threshold; it does close the path where the number moves and the
    record does not. The `science_bands` are better protected anyway, because their
    constants are named (``VKS_LAI_THRESHOLD``, ``14.4248``); a floor is a bare literal
    in a call, which is why the weaker family sets the requirement.
    """
    numeric = re.compile(r"\d+\.\d+(?:[eE]-?\d+)?|\d+[eE]-\d+")
    manifests = (
        MANIFEST_PATH,
        MANIFEST_PATH.parent / "station-reference.manifest.json",
    )
    checked = 0
    for manifest_path in manifests:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        for field in FIELDS:
            for entries in manifest[field].values():
                for entry in entries:
                    filename = entry["locus"].split("::")[0]
                    src = (_REPO_ROOT / filename).read_text(encoding="utf-8")
                    literals = numeric.findall(entry["bound"])
                    assert literals, entry  # a bound with no number is not a bound
                    for literal in literals:
                        assert literal in src, (entry["locus"], literal)
                    checked += 1
    assert checked == len(collect_science_gates()), checked


def test_completeness_gate_detects_an_unfrozen_param(monkeypatch, tmp_path) -> None:
    # Teeth: the gate is plain set equality, so an unfrozen file on disk must break it.
    # Seed a temp params dir with the frozen names + one phantom, point the glob there,
    # and confirm the comparison no longer holds. The real params dir is untouched.
    frozen = set(_load_manifest()["param_files"])
    for name in frozen:
        (tmp_path / name).touch()
    (tmp_path / "phantom.yaml").touch()
    monkeypatch.setattr(sys.modules[__name__], "PARAMS_DIR", tmp_path)
    on_disk = {p.name for p in PARAMS_DIR.glob("*.yaml")} - _EXCLUDED_PARAMS
    assert on_disk != frozen  # the phantom param is detected — the gate has teeth


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
    # The integrator + dt live as documented values in the manifest; the goldens enforce
    # them (an RK4 / dt switch moves every golden). This pins that the manifest *records
    # the lock* — the documentation half of "locked by end of P4".
    #
    # ⚠ KEEP THESE HARD-CODED LITERALS. There is now an importable ``BIO_DT``, and
    # comparing against it would be the natural "cleanup" — do not. A contract that
    # imports its own value from the code auto-follows the code, which is the opposite
    # of a freeze: this assertion's entire job is to go red when someone moves the step,
    # so the unfreeze discipline in docs/biosphere-reference.md gets followed instead of
    # skipped. It did exactly that on 2026-08-14, which is how the step move became a
    # deliberate ceremony rather than a silent edit.
    manifest = _load_manifest()
    assert manifest["integrator"] == "EulerIntegrator"
    assert manifest["dt_days"] == 0.25


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


def test_manifest_pins_the_within_day_light_path() -> None:
    """⚠ The one hash in ``forcing`` that IS compared, and why it has to be.

    The other two hashes here are provenance (the goldens enforce their values). This
    one
    is a gate because the thing it fingerprints — the *shape* of the day's PAR — can
    be changed without touching a single file the manifest hashes: the weather fixture
    is unchanged by construction, since the light path redistributes a day's photons
    rather than adding any. A silent swap of the sinusoid would move every golden and
    leave the manifest reading identical, which is the failure mode
    ``multirate-effective-step-is-per-rate-class`` records.
    """
    manifest = _load_manifest()
    assert manifest["forcing"]["light_path"] == _light_path_fingerprint()


if __name__ == "__main__":
    _regenerate()
