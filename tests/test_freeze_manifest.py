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

⚠⚠ **SLICE 6 OF THE REFERENCE FLIP (2026-08-16) CHANGED WHO PRODUCES THIS MANIFEST, AND
THE HEADLINE IS *MIXED AUTHORITY*, NOT "IT IS RUST-ANCHORED NOW."** The keys the Rust
reference tree can produce are now spliced in from it — ``flow_set``, ``aux_set``,
``forcing.light_path``, the horizons — by shelling
``cargo run --example dump_biosphere_inventory``. **Everything else is still Python's**,
and by content that is most of the file: ``science_bands`` + ``liveness_floors`` alone
are about half of it and are a static census of pytest markers, which has no Rust
referent and cannot have one while the science gates are pytest-side. Each key says
which side it comes from, and why, in the manifest's own ``_authority`` block
(:data:`_AUTHORITY`) — so a future reader cannot mistake a Python-retained field for a
Rust-derived one.

Two consequences worth stating where they will be read:

* **Regeneration now requires ``cargo``**, because it now reads the reference. The tests
  do not: nothing in this module shells cargo, and the base suite stays offline-clean.
  The cargo-side gates (the manifest is not *stale* with respect to the Rust tree; the
  frozen ``dt_days`` literal still matches ``BIO_DT`` in the reference tree) live in
  ``tests/crossport/test_inventory_parity.py``, on the ``crossport`` CI job.
* **The completeness gates below changed meaning without changing a line of their
  arithmetic.** ``set(manifest["flow_set"]) == set(_flow_set())`` used to say *the
  manifest froze everything Python has*; it now says *Python still matches the
  reference*. Same assertion, opposite reading, and the failure it reports is a
  **Python** drift.
"""

import hashlib
import json
import re
import subprocess
import sys
from functools import lru_cache
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

# Scenario name -> (human scenario label, horizon, golden filename). The four Phase-3
# canonical scenarios + the three Phase-4 long-horizon artifacts (P4.2). The labels are
# documentation.
#
# ⚠ **The horizon is a NAME, not a number, since slice 6.** It names a key of the
# reference tree's own horizon constants (Rust's ``horizons`` dump), because the year
# counts are a property of the reference and no longer of the checker. ``open_season``
# is the exception and carries the literal ``1``: a single season has no named constant
# on *either* side — it is what "one season" means, and inventing a Rust constant for
# it to make the table uniform would be a manifest field with a made-up referent.
_SCENARIOS: dict[str, tuple[str, str | int, str]] = {
    "open_season": ("DEFAULT_SCENARIO (open field)", 1, "season_euler_state.json"),
    "sealed_chamber": (
        "SEALED_CHAMBER_SCENARIO",
        "sealed_chamber_years",
        "sealed_chamber_state.json",
    ),
    "perennial_chamber": (
        "PERENNIAL_CHAMBER_SCENARIO",
        "perennial_chamber_years",
        "perennial_chamber_state.json",
    ),
    "consumer_chamber": (
        "CONSUMER_CHAMBER_SCENARIO",
        "consumer_chamber_years",
        "consumer_chamber_state.json",
    ),
    "perennial_long_horizon": (
        "PERENNIAL_CHAMBER_SCENARIO",
        "long_horizon_years",
        "perennial_long_horizon_state.json",
    ),
    "consumer_long_horizon": (
        "CONSUMER_CHAMBER_SCENARIO",
        "long_horizon_years",
        "consumer_long_horizon_state.json",
    ),
    "drift_summary": (
        "PERENNIAL_CHAMBER_SCENARIO + CONSUMER_CHAMBER_SCENARIO (stability signature)",
        "long_horizon_years",
        "drift_summary.json",
    ),
}

#: The **checker's** horizon constants, under the reference's names. Used only by the
#: conformance gate (:func:`test_python_horizons_match_the_reference`) — never by
#: :func:`_build_manifest`, which reads the reference. The two must agree, and a gate
#: says so; the point of keeping both is that a disagreement is visible rather than
#: resolved by whichever side happened to be imported.
_PYTHON_HORIZONS: dict[str, int] = {
    "sealed_chamber_years": SEALED_CHAMBER_YEARS,
    "perennial_chamber_years": PERENNIAL_CHAMBER_YEARS,
    "consumer_chamber_years": CONSUMER_CHAMBER_YEARS,
    "long_horizon_years": LONG_HORIZON_YEARS,
}


#: This manifest's scenario roster — the filter for the science-gate fields, so
#: a gate naming a station scenario cannot silently land in the biosphere manifest.
_ROSTER = frozenset(_SCENARIOS)


#: The reference tree's own dump of the keys it can author — the producer half of this
#: manifest since slice 6. Its doc comment is the authority on what it emits and why.
_RUST_CRATE_DIR = _REPO_ROOT / "rust" / "crates" / "domains"
_RUST_DUMP_EXAMPLE = "dump_biosphere_inventory"

#: The keys :func:`_build_manifest` consumes out of that dump, asserted as its **exact**
#: key set. ⚠ A forcing function, not a filter (slice 3's move, kept): a key added to
# the : dump program turns regeneration into a loud error rather than silently entering
# — or : silently *not* entering — the frozen surface. ``locked_dt_days`` is in the
# dump and : deliberately not spliced; see :data:`_AUTHORITY` for ``dt_days``.
_RUST_DUMP_KEYS = frozenset(
    {"flow_set", "aux_set", "horizons", "light_path_samples", "locked_dt_days"}
)


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
            f"regenerated FROM the Rust reference since slice 6 of the reference flip, "
            f"so regeneration needs a working Rust toolchain:\n{proc.stderr}"
        )
    dump: dict[str, Any] = json.loads(proc.stdout)
    if set(dump) != _RUST_DUMP_KEYS:
        raise SystemExit(
            f"{_RUST_DUMP_EXAMPLE} emitted {sorted(dump)}, expected "
            f"{sorted(_RUST_DUMP_KEYS)}. Read _AUTHORITY before widening this: a new "
            "key has to be classified, and one that cannot honestly come from the "
            "(a param-file list, a pytest-marker census) must not enter the manifest "
            "through here."
        )
    return dump


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

    ⚠ **Since slice 6 this is the CHECKER's half.** The manifest's value is the hash of
    the *reference* tree's samples (``dump_biosphere_inventory``'s
    ``light_path_samples``); this function recomputes the same grid in Python and
    ``test_manifest_pins_the_within_day_light_path`` compares the two. The sampling grid
    is therefore written out on both sides — a duplicated literal, and the tolerable
    kind: change either copy and the hashes stop matching, in both directions.
    """
    return _fingerprint(
        half_sine_window_mean(k * 0.25, 0.25, 400.0, daylength_h * 3600.0).hex()
        for daylength_h in (8.0, 12.0, 16.0)
        for k in range(4)
    )


def _fingerprint(samples: Any) -> str:
    """Hash an ordered run of hex-float sample strings — the shared formatting half.

    Split out of :func:`_light_path_fingerprint` so the two sides hash *identically* by
    construction and can differ only in the samples, which is the thing being compared.
    """
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


#: Per-**path** authority: which side each field of the frozen surface comes from, and
#: why. Written into the manifest itself (as ``_authority``) so the file states its own
#: mixed provenance to whoever opens it — slice 6 of the reference flip.
#:
#: ⚠ **Keyed by path, not by top-level key, because two keys split.** ``forcing`` has
#: three children with two different answers, and ``scenarios`` splits *inside one
#: scenario*: ``perennial_long_horizon_state.json`` is the reference's own output while
#: ``drift_summary.json`` is Python's fold of that **same run** (slice 5's handoff — the
#: fold is the artifact). A top-level classification would hide exactly that.
#:
#: ⚠ The three sides are claims of different kinds. ``rust`` — produced by the reference
#: tree and spliced in by :func:`_build_manifest`. ``python`` — produced by the checker
#: because the reference has no referent for it, with the reason stated and, where one
#: exists, the condition under which that could change. ``hand`` — a literal or a label
#: deliberately derived from neither, because a contract field that imports its own
# value : auto-follows the code.
_AUTHORITY: dict[str, dict[str, str]] = {
    "_comment": {"side": "hand", "why": "prose header"},
    "frozen_at_phase": {"side": "hand", "why": "the phase this surface froze at"},
    "reference_doc": {
        "side": "hand",
        "why": "pointer to the prose half of the contract",
    },
    "integrator": {
        "side": "hand",
        "why": (
            "one of the two deliberate anti-derived literals. Unlike dt_days it has no "
            "importable constant on EITHER side — each run helper selects the scheme "
            "inline — so it is documented here and enforced by the goldens (an RK4 "
            "switch moves every one). A literal typed into the Rust dump to make the "
            "pair symmetric would read like a gate and be none."
        ),
    },
    "dt_days": {
        "side": "hand",
        "why": (
            "the second anti-derived literal: a manifest that imported BIO_DT would "
            "auto-follow a step change, which is the opposite of a freeze — the "
            "2026-08-14 step move became a ceremony only because this literal went red."
            "Slice 6 added the missing half instead: the crossport gate checks it "
            "against the REFERENCE tree's BIO_DT, so moving Rust's step without the "
            "ceremony is red rather than silent."
        ),
    },
    "long_horizon_years": {
        "side": "rust",
        "why": "the reference tree's LONG_HORIZON_YEARS",
    },
    "flow_set": {
        "side": "rust",
        "why": (
            "the union of Flow::type_name() over the four canonical builds in the "
            "reference tree — derived from built registries, never hand-listed"
        ),
    },
    "aux_set": {
        "side": "rust",
        "why": "the same walk over AuxProcess::type_name()",
    },
    "forcing/light_path": {
        "side": "rust",
        "why": (
            "sha-256 of the reference tree's own light-path samples. Measured on "
            "2026-08-16 before re-anchoring: Rust reproduces all twelve hex-float "
            "samples byte for byte, so the hash did not move — this key is gated "
            "exactly, not tolerance-bound, and could not have been re-anchored on a "
            "prediction."
        ),
    },
    "forcing/weather_fixture": {
        "side": "python",
        "why": (
            "the driving weather is a Python-side oracle fixture; the port reads "
            "weather_facts.txt, generated FROM it — the same shape as param_files"
        ),
    },
    "forcing/weather_sha256": {
        "side": "python",
        "why": "provenance hash of that Python-side fixture; not compared",
    },
    "param_files/*": {
        "side": "python",
        "why": (
            "PYTHON-RETAINED UNTIL SLICE 9. The reference reads no YAML: it reads "
            "biosphere_params.txt, generated by Python out of the frozen loaders. Its "
            "prefixes are the generator's naming and not filenames (three come out of "
            "the single phenology.yaml; 17 loaders against 15 files). Anything the "
            "printed here would be this list travelling through Rust and back. Slice 9 "
            "decides who loads the params; this key can only re-anchor after it."
        ),
    },
    "science_bands/*": {
        "side": "python",
        "why": (
            "a static AST census of science_gate markers on pytest functions "
            "(tests/science_gates.py). There is no Rust referent and there cannot be "
            "one while the science gates are pytest-side. This and liveness_floors are "
            "about half the manifest by content, which is why 'the manifest is "
            "Rust-anchored' is the wrong summary of slice 6."
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
    "scenarios/*/years": {
        "side": "rust",
        "why": "the reference tree's horizon constant",
    },
    "scenarios/*/golden": {"side": "hand", "why": "the artifact's filename"},
    "scenarios/*/golden_sha256": {
        "side": "rust",
        "why": (
            "the golden is the reference tree's own output (golden_platform."
            "RUST_AUTHORED, which this block is checked against, not restating). "
            "Unlike "
            "the other hashes here this one IS gated against the file on disk: a "
            "golden is machine-generated and its hash is newline-normalized, so 'the "
            "manifest pins bytes that exist' is a completeness claim, not the value "
            "re-assertion that param_files declines."
        ),
    },
    "scenarios/drift_summary/golden_sha256": {
        "side": "python",
        "why": (
            "⚠ ONE RUN, TWO AUTHORS. This is drift.py's Python-side fold of the same "
            "15-yr perennial trajectory whose final state Rust authors next door, and "
            "the two engines differ by 1 ULP on it. The fold is the artifact, and its "
            "correct reference is Python's own output — so the golden axis is not '6 "
            "Rust, 1 folded' scenario by scenario."
        ),
    },
}


def _build_manifest() -> dict[str, object]:
    """Assemble the manifest — the reference tree's keys spliced into the checker's.

    ⚠ Since slice 6 this reads the **Rust** tree for everything :data:`_AUTHORITY` marks
    ``rust``, so it needs ``cargo``. It is reachable only from :func:`_regenerate`.
    """
    reference = _rust_reference()
    horizons = reference["horizons"]
    scenarios: dict[str, object] = {}
    for name, (label, horizon, golden) in _SCENARIOS.items():
        scenarios[name] = {
            "scenario": label,
            "years": horizon if isinstance(horizon, int) else horizons[horizon],
            "golden": golden,
            "golden_sha256": _normalized_sha256(GOLDEN_DIR / golden),
        }
    return {
        "_authority": _AUTHORITY,
        "_comment": (
            "Phase-4 freeze manifest (P4.3). Names the frozen biosphere reference "
            "surface. See docs/biosphere-reference.md for the freeze contract + the "
            "unfreeze discipline. Hashes are newline-normalized sha-256 PROVENANCE "
            "(value enforcement is the scenario goldens). Each key's producer and why "
            "is in _authority: this file has MIXED authority since slice 6 of the "
            "reference flip. Regenerate on a deliberate unfreeze: uv run python "
            "tests/test_freeze_manifest.py — which now shells cargo, because the "
            "_authority 'rust' keys are read from the reference tree."
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
        "long_horizon_years": horizons["long_horizon_years"],
        "flow_set": reference["flow_set"],
        "aux_set": reference["aux_set"],
        "forcing": {
            "weather_fixture": WEATHER_FIXTURE.name,
            "weather_sha256": _normalized_sha256(WEATHER_FIXTURE),
            # The within-day PAR shape — see _light_path_fingerprint for why a hash of
            # sampled values rather than a name. Unlike the two hashes above (provenance
            # only, never compared), this one IS gated:
            # test_manifest_pins_the_within_day_light_path. ⚠ Since slice 6 the samples
            # are the REFERENCE tree's; only the hashing is done here, because that is
            # pure formatting and the Rust crate carries no digest dependency.
            "light_path": _fingerprint(reference["light_path_samples"]),
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
    """⚠ Same assertion since P4.3, opposite reading since slice 6.

    It used to say *the manifest froze every flow Python has*. The manifest's flow set
    is now the **reference** tree's, so it says *the checker still has exactly the
    reference's flows* — a failure here is a Python drift (or a Rust flow added and
    frozen without its mirror), and it is NOT fixed by regenerating the manifest.
    """
    manifest = _load_manifest()
    assert set(manifest["flow_set"]) == set(_flow_set()), (
        "Python's canonical registries no longer carry the frozen flow set. The frozen "
        "set is the Rust reference's since slice 6 — so this is the checker drifting "
        "from the reference, not a manifest to regenerate."
    )


def test_frozen_aux_set_is_complete() -> None:
    # The aux axis of the same conformance check — the third 'wired into a registry'
    # axis (non-conserved accumulators) alongside flows + params.
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
    # ⚠ A conformance check since slice 6, not a derivation check. The frozen horizon is
    # the reference tree's constant; this asserts the checker's LONG_HORIZON_YEARS still
    # agrees with it, so the two ports cannot silently run different decades.
    manifest = _load_manifest()
    assert manifest["long_horizon_years"] == LONG_HORIZON_YEARS


def test_python_horizons_match_the_reference() -> None:
    """Every scenario's frozen run length, against the checker's own constants.

    The horizon axis widened with the roster: ``long_horizon_years`` is one of four, and
    the other three (3 / 5 / 5) reach the manifest through ``scenarios.*.years``, where
    nothing compared them to Python at all before this. ``open_season`` is excluded by
    construction — it carries the literal 1 on both sides (see :data:`_SCENARIOS`).
    """
    manifest = _load_manifest()
    for name, (_, horizon, _golden) in _SCENARIOS.items():
        expected = horizon if isinstance(horizon, int) else _PYTHON_HORIZONS[horizon]
        assert manifest["scenarios"][name]["years"] == expected, name


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


def _leaf_paths(node: Any, prefix: str = "") -> list[str]:
    """Every leaf path of the manifest, ``/``-joined.

    Dicts recurse; anything else (including a list) is a leaf, because the frozen units
    here are whole lists — ``flow_set`` is one claim, not 23. ``/`` rather than ``.``
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
    score, pattern, entry = max(matches, key=lambda m: m[0])
    return pattern, entry


def test_every_frozen_field_declares_who_produced_it() -> None:
    """The manifest states its own mixed authority, and the block cannot go stale.

    ⚠ **Checked in both directions**, because each direction fails differently: an
    unclassified field is a frozen value whose producer nobody stated (the thing slice 6
    exists to prevent), while a classification pattern matching nothing is a stale row
    describing a field that has been renamed or removed — which reads as coverage and is
    not.
    """
    manifest = _load_manifest()
    classified = {k: v for k, v in manifest.items() if k != "_authority"}
    paths = _leaf_paths(classified)
    unclassified = [p for p in paths if _authority_for(p) is None]
    assert not unclassified, (
        f"frozen fields with no _authority entry: {unclassified}. Every field of this "
        "contract has to say which side produces it — see _AUTHORITY in "
        "tests/test_freeze_manifest.py."
    )

    # ⚠ "Most specific wins" only decides anything while no two patterns TIE. Two of
    # equal specificity matching one path would resolve by dict order — a silent answer
    # to a question nobody asked, and the field would read as classified either way.
    # There is no such pair today (`scenarios/drift_summary/golden_sha256` beats
    # `scenarios/*/golden_sha256` 3-2); this keeps it that way (advisor).
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
    ties them to the one roster — including the case that makes the tie worth having:
    ``drift_summary``, whose golden is a Python fold of the same run whose final state
    Rust authors.
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
    """⚠ The one class of hash here that IS compared — and slice 5 measured the hole.

    Regenerating a frozen golden used to desynchronise this manifest **with every gate
    green**: ``golden_sha256`` is assembled inside ``_regenerate()`` and was never read
    back, so the file could pin bytes that no longer existed. Measured on 2026-08-16:
    swapping in two regenerated goldens turned four Python gates red and left both
    freeze-manifest gates green.

    ⚠ Deliberately **goldens only**. The param and weather hashes stay provenance for
    the reason the module docstring gives — they are hand-edited files whose *values*
    the goldens already enforce. A golden is different in kind: it is machine-generated,
    it **is** the value, and its newline-normalized hash is reproducible on every
    platform, so "the manifest pins bytes that exist" is a completeness claim rather
    than a redundant re-assertion.
    """
    manifest = _load_manifest()
    for entry in manifest["scenarios"].values():
        assert entry["golden_sha256"] == _normalized_sha256(
            GOLDEN_DIR / entry["golden"]
        ), (
            f"{entry['golden']} has moved since the manifest was regenerated. If the "
            "move was intended it is an unfreeze: follow the ceremony in "
            "docs/biosphere-reference.md and regenerate the manifest as its record."
        )


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
