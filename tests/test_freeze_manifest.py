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

⚠⚠ **SLICE C7 (2026-08-18) TOOK THE WRITER AWAY FROM THIS MODULE.** It used to own
both halves: the completeness gate below *and* a ``__main__`` regeneration action that
assembled and serialized the manifest. The manifest was therefore authored by the
reference (slices 6, C4, C8, C9 moved key after key) and *written* by the checker —
which is the Python-shaped hole C7 closes. Regeneration is now, from ``rust/``::

    cargo run --example dump_biosphere_inventory -- --write-manifest

and the committed file is held to it byte for byte by
``tests/crossport/test_manifest_writer.py`` (cargo-gated, so this module stays offline).
What is left here is a **checker**: the completeness gates, and the conformance checks
that say the checker's own copies of the roster, the horizons, the flow/aux sets and the
param census still agree with the reference's.

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
# flows in ``flows.py`` (no real biology). Scoped out of the biosphere reference, by
# name, so its absence reads as deliberate.
# ⚠ Its two regression goldens (``demo_euler_state.json`` / ``demo_rk4_state.json``)
# were deleted on 2026-08-18 by C6 of the reference flip, so this param file is now
# frozen by NOTHING. ``build_demo`` and its unit tests survive in
# ``test_biosphere_demo.py``; whether the whole skeleton follows is a Stage-3
# question, not this exclusion's.
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
#: conformance gate (:func:`test_python_horizons_match_the_reference`) — the manifest's
#: numbers come from the reference's own constants, written by its writer. The two must
#: agree, and a gate says so; the point of keeping both is that a disagreement is
#: visible rather than resolved by whichever side happened to be imported.
_PYTHON_HORIZONS: dict[str, int] = {
    "sealed_chamber_years": SEALED_CHAMBER_YEARS,
    "perennial_chamber_years": PERENNIAL_CHAMBER_YEARS,
    "consumer_chamber_years": CONSUMER_CHAMBER_YEARS,
    "long_horizon_years": LONG_HORIZON_YEARS,
}


#: How many science gates the REFERENCE declares — 13 in the biosphere table (slice C4)
#: plus the 2 the station table took in slice C4b. ⚠ A literal, on purpose: this module
#: never shells cargo, so it cannot count them, and the point of the number is to be an
#: INDEPENDENT one the manifest is measured against rather than one derived from the
#: manifest it is checking. The derived comparison against the live Rust tree is
#: cargo-gated, in tests/crossport/test_inventory_parity.py.
_REFERENCE_GATE_COUNT = 15

#: The station contract, read alongside this one wherever a claim about the whole
#: reference is made — since slice C4b the science census spans both files.
_STATION_MANIFEST_PATH = MANIFEST_PATH.parent / "station-reference.manifest.json"


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
#: tree, which since slice C7 also writes the file. ``python`` — produced by the checker
#: because the reference has no referent for it, with the reason stated and, where one
#: exists, the condition under which that could change. ``hand`` — a literal or a label
#: deliberately derived from neither, because a contract field that imports its own
# value : auto-follows the code.
def _load_manifest() -> dict[str, Any]:
    # The committed manifest as parsed JSON (values are Any — the gate reads the frozen
    # sets out of it; pyright would otherwise type every value as ``object``).
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


# --- the completeness gate (what the goldens are blind to) -------------------


def test_frozen_param_set_is_complete() -> None:
    """⚠ Same assertion since Phase 1, opposite reading since slice C8.

    It used to say *the manifest froze every param file on disk*. The manifest's
    ``param_files`` is now spliced from the **reference's** census — the set it actually
    loads — so this says *the checker's directory rule still agrees with what the
    reference reads*. A failure is one of two roster findings, and they are not the
    same repair: a file added to the tree and wired into no Rust loader, or a loader
    dropped with its file left behind. Neither is fixed by regenerating.
    """
    manifest = _load_manifest()
    on_disk = {p.name for p in PARAMS_DIR.glob("*.yaml")} - _EXCLUDED_PARAMS
    assert set(manifest["param_files"]) == on_disk, (
        "the frozen param census and this directory disagree. Since slice C8 the "
        "manifest names the files the REFERENCE loads, so the finding is a roster "
        "one: either a param file was added here and wired into no Rust loader, or a "
        "loader was dropped and its file left behind. Do not 'fix' it by "
        "regenerating."
    )


def test_the_frozen_param_hashes_match_the_checker_s_own_digest() -> None:
    """⚠⚠ The retained Python digest, kept as a **conformance check on the checker**.

    Slice C8 moved the census and the normalization rule to the reference, and the
    obvious next move — delete the Python side — would have thrown away the only thing
    that says the two rules still agree. So :func:`_normalized_sha256` stays and its
    meaning inverts, exactly as slice 6 did for the flow set and slice 8 for the
    authoring rosters: the identical assertion now asks *has the checker's hashing
    drifted from the contract?*

    ⚠ This is also the honest boundary of the re-anchoring. The 15 values are
    **author-neutral**: both sides digest the same bytes under the same rule, which is
    why the ceremony moved not one of them. A green run here is therefore evidence
    about the *rules* agreeing, not about who owns the number — and stating that is
    the difference between a check and a claim.
    """
    manifest = _load_manifest()
    mismatched = {
        name: (recorded, _normalized_sha256(PARAMS_DIR / name))
        for name, recorded in manifest["param_files"].items()
        if recorded != _normalized_sha256(PARAMS_DIR / name)
    }
    assert not mismatched, (
        "the checker's newline-normalized sha-256 disagrees with the frozen "
        f"(reference-produced) one for {sorted(mismatched)}. Either a param file was "
        "edited without the unfreeze ceremony, or the two normalization rules have "
        "diverged — check config::provenance::normalize_newlines against "
        "_normalized_sha256 before touching the manifest."
    )


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


def test_the_frozen_science_gates_are_the_references() -> None:
    """The science half of the contract is the REFERENCE's, locus by locus.

    Before these fields the frozen acceptance set was {golden bytes, ``rationed == 0``,
    no extinction, conservation, determinism} — every one a property of the RUN, not the
    SCIENCE (``post-roadmap-acceptance-gate.md`` finding 5). A committed band could be
    deleted with every gate green.

    ⚠⚠ **REPLACED IN SLICE C4.** This asserted ``manifest[field] == gates_for(_ROSTER,
    field)`` — the manifest against the checker's own pytest markers. That comparison is
    now void in the direction that matters: the claims are declared in
    ``rust/crates/domains/src/biosphere/science_gates.rs`` and spliced in by
    :func:`_science_census`, so this module would be comparing the manifest against an
    empty Python census and passing.

    What is checkable **without cargo** is where every claim now lives, and that is not
    a formality: it is the one assertion that would go red if a biosphere gate crept
    back into ``tests/``, which is the concrete way this re-anchoring could be undone by
    accident. The value comparison against the live reference tree is cargo-gated, in
    ``tests/crossport/test_inventory_parity.py``.

    ⚠ **Widened in slice C4b, and the widening is the point.** Until C4b this asserted
    ``rust/crates/domains/`` on the biosphere manifest's 13 entries and the count
    was 13. The station's two claims now live in ``rust/crates/station/`` — a second
    table, because a gate lives with the runs it reads and those read ``station`` — so
    the assertion is read over BOTH manifests, the prefix is ``rust/crates/`` and the
    count is the whole reference's. Checking only this manifest would leave the station
    pair asserted by nothing on this side, which is exactly the hole C4b closes.
    """
    entries = [
        e
        for path in (MANIFEST_PATH, _STATION_MANIFEST_PATH)
        for field in FIELDS
        for v in json.loads(path.read_text(encoding="utf-8"))[field].values()
        for e in v
    ]
    assert len(entries) == _REFERENCE_GATE_COUNT, len(entries)
    for entry in entries:
        assert entry["locus"].startswith("rust/crates/"), entry


def test_the_frozen_claim_text_survived_the_pipe() -> None:
    """⚠ The residual half of the mojibake hole, closed by naming the characters.

    Slice C4's first regeneration wrote ``—`` into the contract as ``â€”``: the
    reference emits UTF-8, ``subprocess.run(text=True)`` decoded it with the Windows
    locale's cp1252, and nothing was red because the *manifest* and the *checker* agreed
    — the corruption happened on the way in. Both readers now pin ``encoding="utf-8"``,
    and the crossport parity gate catches losing it on **one** side.

    It cannot catch losing it on **both**: two identically-corrupted sides compare
    equal. So this asserts the characters themselves. Crude on purpose, and cheap: the
    claim text is frozen, the characters are in it today, and a decode that mangles them
    mangles all of them at once.
    """
    manifest = _load_manifest()
    text = json.dumps(manifest, ensure_ascii=False)
    for char, why in (
        ("Γ", "the compensation point's gamma"),
        ("₂", "the subscript in CO2"),
        ("—", "the em dash the first corruption ate"),
        ("⚠", "the warning marker"),
    ):
        assert char in text, (char, why)
    assert "â€" not in text, "cp1252-decoded UTF-8 is frozen in the contract"


def test_the_python_science_census_is_exhausted() -> None:
    """The pytest-marker census is **empty** — asserted, not assumed.

    ⚠⚠ **This test's claim inverted in slice C4b, and the machinery below it is kept
    for exactly that reason.** Slice C4 moved thirteen of the fifteen markers to the
    reference and this asserted the two survivors were the station pair; C4b moved those
    two (to ``rust/crates/station/src/science_gates.rs``), so the census that was the
    derivation is now a **forcing function**: nothing in ``tests/`` may carry a
    ``science_gate`` marker, because a claim filed here would be a claim the frozen
    contracts do not name.

    ⚠ An empty census is exactly the shape this repo has been bitten by — a check that
    passes against nothing. It is not inert here, and the difference is which direction
    it looks: it does not walk the census and assert something about each member (which
    an empty census would satisfy vacuously), it asserts the census *is* empty, so
    re-marking any test turns it red. Its companion in the other direction is
    :func:`test_the_frozen_science_gates_are_the_references`, which requires every
    frozen entry's locus to be under ``rust/crates/``.

    ⚠ ``tests/science_gates.py`` therefore stays live rather than being retired with its
    last producer: it is what makes "no marker in the checker" a mechanism rather than a
    convention. Retiring it is Stage 3's call, with the rest of the checker.
    """
    assert collect_science_gates() == ()
    for field in FIELDS:
        assert gates_for(_ROSTER, field) == {name: [] for name in sorted(_ROSTER)}


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


# ⚠⚠ **`test_science_gate_bounds_name_a_literal_present_at_their_locus` WAS RETIRED IN
# SLICE C4b, AND IT HAD NEVER BEEN ABLE TO FAIL.** It read every frozen `bound`'s
# numeric literals and required each to appear textually in the file the `locus`
# named. That is
# true by construction and always was: the *record* lives in the file the locus names —
# the `bound=` keyword of a `science_gate` marker before the flip, the `bound:` field of
# the reference's table after it — so the literal was put there by the very thing being
# checked. Measured, not reasoned: deleting `0.8814` from the RQ gate's assertion left
# it green, and so did subtracting the records' own occurrences (the scanner's pin test
# quotes six of the real frozen bounds as test data).
#
# The rule is FIXED rather than lost — `domains::biosphere::science_gates::code_only`
# strips comments and string literals first, so the literal must appear in *executable*
# code, and deleting an assertion that carries a recorded number is now red. What could
# not follow is the checker's copy: the rule requires reading the locus file's SYNTAX,
# and after C4b every locus is a `.rs` file, so expressing it here needs a second Rust
# lexer in Python — a rule with two copies, in the language the flip is retiring. **That
# narrow reason is what licenses the removal**, and it stops being true the day a Python
# locus reappears. It is not "the census is Rust's now"; that broader reason has already
# been recorded as too broad twice in this flip.
#
# Three things replace it, and none of them is a restatement:
#
#   (1) the rule itself, at the loci — `check_bound_literals` + `code_only`, run from
#       both census tables, with their own controls and a pinned scanner;
#   (2) the no-cargo half — `test_the_frozen_science_gates_are_the_references` below
#       already asserts the entry count against `_REFERENCE_GATE_COUNT` and every
#       locus's `rust/crates/` prefix, over BOTH manifests;
#   (3) the drift half — `tests/crossport/test_inventory_parity.py` holds each committed
#       manifest's census equal to the live Rust table, so a `bound` text that moved
#       away from the code is red there (cargo-gated).
#
# ⚠ `_REFERENCE_GATE_COUNT` now has exactly one consumer, in (2). It had two while this
# test lived, which is the shape that makes a number look derived when it is typed.


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


def test_the_frozen_roster_is_the_references() -> None:
    """The scenario table, checker's copy against the file the reference writes.

    ⚠ **New in slice C7, and it closes a hole the slice opened.** The roster —
    ``name -> (label, horizon, golden)`` — used to live here *and* be written from here,
    so no gate was needed: the manifest could not disagree with its own source. C7 moved
    the writer, and the table went with it. What is left in this module is a second copy
    with nothing holding it to the first, which is the shape the repo keeps re-learning
    (*a rule with two copies has one that is stale*).

    ⚠ Names alone were already compared (:func:`test_every_roster_scenario_has_an_
    explicit_science_gate_entry`) and the run lengths separately
    (:func:`test_python_horizons_match_the_reference`). Unchecked were the two fields
    ``_authority`` marks ``hand`` — the human label and the golden's filename — and a
    hand field is exactly the one a gate cannot re-derive if it drifts.
    """
    manifest = _load_manifest()
    assert set(manifest["scenarios"]) == set(_SCENARIOS)
    for name, (label, _horizon, golden) in _SCENARIOS.items():
        entry = manifest["scenarios"][name]
        assert entry["scenario"] == label, name
        assert entry["golden"] == golden, name


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


def _authority_matches(
    path: str, authority: dict[str, dict[str, str]]
) -> list[tuple[int, str, dict[str, str]]]:
    """Every ``_authority`` pattern matching ``path``, with its specificity score.

    ⚠ Takes the block as an argument since slice C7. It used to read a module-level
    ``_AUTHORITY`` literal that the writer spliced into the manifest and a test then
    compared back — with the writer gone there is no second copy, and a checker that
    kept one would be asserting a duplicate against the file it duplicates.
    """
    segments = path.split("/")
    matches = []
    for pattern, entry in authority.items():
        parts = pattern.split("/")
        if len(parts) != len(segments):
            continue
        if any(p not in ("*", s) for p, s in zip(parts, segments, strict=True)):
            continue
        matches.append((sum(p != "*" for p in parts), pattern, entry))
    return matches


def _authority_for(
    path: str, authority: dict[str, dict[str, str]]
) -> tuple[str, dict[str, str]] | None:
    """Resolve a leaf path against the manifest's ``_authority``, most specific wins."""
    matches = _authority_matches(path, authority)
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
    authority = manifest["_authority"]
    classified = {k: v for k, v in manifest.items() if k != "_authority"}
    paths = _leaf_paths(classified)
    unclassified = [p for p in paths if _authority_for(p, authority) is None]
    assert not unclassified, (
        f"frozen fields with no _authority entry: {unclassified}. Every field of this "
        "contract has to say which side produces it — see the AUTHORITY table in "
        "rust/crates/domains/examples/dump_biosphere_inventory.rs, which writes it."
    )

    # ⚠ "Most specific wins" only decides anything while no two patterns TIE. Two of
    # equal specificity matching one path would resolve by dict order — a silent answer
    # to a question nobody asked, and the field would read as classified either way.
    # There is no such pair today (`scenarios/drift_summary/golden_sha256` beats
    # `scenarios/*/golden_sha256` 3-2); this keeps it that way (advisor).
    for path in paths:
        top = max(s for s, _, _ in _authority_matches(path, authority))
        tied = sorted(p for s, p, _ in _authority_matches(path, authority) if s == top)
        assert len(tied) == 1, (
            f"{path} is matched by {len(tied)} _authority patterns of equal "
            f"specificity: {tied}. Which one applies would be decided by dict order — "
            "make one of them strictly more specific."
        )

    matched = {_authority_for(p, authority)[0] for p in paths}  # type: ignore[index]
    stale = sorted(set(authority) - matched)
    assert not stale, f"_authority patterns matching no field: {stale}"

    # ⚠ What replaced the fourth check, and why it is not a weakening. Until slice C7
    # this ended with ``manifest["_authority"] == _AUTHORITY`` — the committed block
    # against the module's own literal. C7 deleted the literal with the rest of the
    # writer, and keeping a copy purely to assert against would be the stale second copy
    # this repo has been bitten by. What is checkable from here is the block's SHAPE,
    # and
    # a malformed row is the failure the equality never caught either: a row is
    # ``{side, why}``, ``side`` is one of the three the contract defines, and ``why`` is
    # prose someone wrote rather than an empty string standing in for a reason.
    for pattern, entry in sorted(authority.items()):
        assert set(entry) == {"side", "why"}, (pattern, sorted(entry))
        assert entry["side"] in {"rust", "python", "hand"}, (pattern, entry["side"])
        assert len(entry["why"]) > 10, (pattern, entry["why"])


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
        side = _authority_for(  # type: ignore[index]
            f"scenarios/{name}/golden_sha256", manifest["_authority"]
        )[1]["side"]
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
