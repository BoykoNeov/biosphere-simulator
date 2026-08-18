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
the goldens, not asserted here. Regeneration is a deliberate, separate action (the
golden discipline) and it is **the reference's** since C7's station half: on an
advisor-reviewed unfreeze, run ``cargo run --example dump_station_inventory --
--write-manifest`` from ``rust/`` and review the manifest diff. This module has no
``__main__``.
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
import sys
from pathlib import Path
from typing import Any

from config.paths import DOMAIN_PARAMS_ROOT, GOLDEN_DIR, STATION_PARAMS_DIR
from domains.crew.loader import load_crew_params
from domains.crew.system import build_crew
from domains.eclss.loader import load_eclss_params
from domains.eclss.system import build_eclss
from domains.power.loader import load_charge_params, load_self_discharge_params
from domains.power.system import build_power
from domains.thermal.loader import load_thermal_params
from domains.thermal.system import build_thermal

# ⚠ `gates_for` left in slice C4b with the last thing that called it: the two claims
# are the reference's now, and a helper that returns thirteen empty lists is worse
# than absent — it reads like a derivation. `FIELDS` stays; it is the field-name pair.
from science_gates import FIELDS
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

GOLDEN_DIR = GOLDEN_DIR

# The five sibling + one station params directories the station reference freezes. The
# biosphere params dir is deliberately ABSENT — the biosphere is delegated to its own
# manifest (see the module docstring). No exclusions are needed (no ``demo.yaml``-style
# skeleton lives in any of these dirs).
PARAM_DIRS: list[Path] = [
    DOMAIN_PARAMS_ROOT / "power",
    DOMAIN_PARAMS_ROOT / "thermal",
    DOMAIN_PARAMS_ROOT / "eclss",
    DOMAIN_PARAMS_ROOT / "crew",
    STATION_PARAMS_DIR,
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
#: ⚠ ``science_bands`` / ``liveness_floors`` joined in **slice C4b**, which moved this
#: contract's two claims into ``rust/crates/station/src/science_gates.rs``. They arrive
#: through the same splice ``param_files`` uses, for the four commits this writer has
#: left: C7's station half deletes :func:`_build_manifest` outright.
_RUST_DUMP_KEYS = frozenset(
    {
        "flow_set",
        "aux_set",
        "horizons",
        "param_files",
        "science_bands",
        "liveness_floors",
    }
)


# ⚠⚠ **`_rust_reference()` LEFT WITH THE WRITER (C7's station half).** It shelled
# `cargo run --example dump_station_inventory` and handed the parsed JSON to
# `_build_manifest`. The reference writes its own manifest now, so this module never
# runs cargo — which is why every gate below reads the **committed file** rather than a
# live tree. The cargo-gated comparisons live in `tests/crossport/`.
#
# ⚠ It took `_RUST_DUMP_KEYS` with it: that frozenset was the *forcing function* that
# stopped an unclassified dump key entering the frozen surface, and the equivalent now
# is `set(dump) == _STATION_DUMP_KEYS` in the crossport staleness gate plus the byte
# comparison in `tests/crossport/test_manifest_writer.py`. ⚠ And it took an
# `encoding="utf-8"` pin that slice C4b had just added — correctly then, because C4b was
# the first slice to send non-ASCII through that pipe; the surviving reader in
# `tests/crossport/test_inventory_parity.py` carries the same pin for the same reason.


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


# ⚠⚠ **THE `_AUTHORITY` LITERAL LEFT WITH THE WRITER (C7's station half).** It was the
# source of the committed `_authority` block, so it could not disagree with the file it
# wrote. `rust/crates/station/examples/dump_station_inventory.rs` owns that table now,
# and keeping a copy here purely to assert equality against would be exactly the stale
# second copy this repo keeps re-learning about. The prose was moved **mechanically**
# (generated from the committed manifest and diffed), never retyped: it is frozen
# contract text, and a re-anchoring that quietly reworded it would be a value change
# wearing a refactor's clothes.
#
# The gates below read `_authority` **out of the committed file**, which is strictly
# more of the tree than the literal covered — a hand edit to the block used to be
# invisible here, and is now caught by the byte comparison in
# `tests/crossport/test_manifest_writer.py`.


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

    ⚠⚠ **REPLACED IN SLICE C4b, the same substitution C4 made on the biosphere.** This
    asserted ``manifest[field] == gates_for(_ROSTER, field)`` — the manifest against the
    checker's own pytest markers. The two claims are declared in
    ``rust/crates/station/src/science_gates.rs`` now and spliced in by
    :func:`_build_manifest`, so that comparison would be the manifest against an
    **empty** Python census: thirteen empty lists on both sides, passing while the two
    claims had vanished. What is checkable **without cargo** is the roster's shape and
    where each claim lives; the value comparison against the live reference tree is
    cargo-gated, in ``tests/crossport/test_inventory_parity.py``.
    """
    manifest = _load_manifest()
    for field in FIELDS:
        assert set(manifest[field]) == _ROSTER, field
        for scenario, entries in manifest[field].items():
            for entry in entries:
                assert entry["locus"].startswith(
                    "rust/crates/station/src/science_gates.rs::"
                ), (scenario, entry)
    # ⚠ The two tripwires that survive the substitution unchanged, and they are the half
    # that matters: an empty census is what a silent re-anchoring failure looks like,
    # and
    # the roster check above is satisfied by thirteen empty lists.
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


def test_the_frozen_roster_is_the_references() -> None:
    """The scenario table, checker's copy against the file the reference writes.

    ⚠ **New in C7's station half, and it closes a hole the slice opened.** The roster —
    ``name -> (label, golden)`` — used to live in :data:`_SCENARIOS` *and* be written
    from it, so no gate was needed: the manifest could not disagree with its own source.
    C7 moved the writer, and the table went with it. What is left here is a second copy
    with nothing holding it to the first, which is the shape this repo keeps re-learning
    (*a rule with two copies has one that is stale*).

    ⚠ Names alone were already compared (the science-gate roster check) and the two
    sealed horizons separately. Unchecked were exactly the two fields ``_authority``
    marks ``hand`` — the human label and the golden's filename — and a hand field is the
    one a gate cannot re-derive if it drifts. Both prior halves of C7 opened the same
    hole in their own contracts; this is the third and last.
    """
    manifest = _load_manifest()
    assert set(manifest["scenarios"]) == set(_SCENARIOS)
    for name, (label, golden) in _SCENARIOS.items():
        entry = manifest["scenarios"][name]
        assert entry["scenario"] == label, name
        assert entry["golden"] == golden, name


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


def _authority_matches(
    path: str, authority: dict[str, dict[str, str]]
) -> list[tuple[int, str, dict[str, str]]]:
    """Every ``_authority`` pattern matching ``path``, with its specificity score.

    ⚠ Takes the block as an argument since C7's station half. It used to read a
    module-level literal; that literal left with the writer, so the caller supplies the
    committed file's block and these helpers stay pure.
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
    """Resolve a leaf path against the committed ``_authority``, most specific wins."""
    matches = _authority_matches(path, authority)
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
    authority = manifest["_authority"]
    classified = {k: v for k, v in manifest.items() if k != "_authority"}
    paths = _leaf_paths(classified)
    unclassified = [p for p in paths if _authority_for(p, authority) is None]
    assert not unclassified, (
        f"frozen fields with no _authority entry: {unclassified}. Every field of this "
        "contract has to say which side produces it — see the AUTHORITY table in "
        "rust/crates/station/examples/dump_station_inventory.rs."
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
    # ⚠ What replaced the fourth check, and why it is not a weakening. Until C7's
    # station half this ended with ``manifest["_authority"] == _AUTHORITY`` — the
    # committed block against the module's own literal. C7 deleted the literal with the
    # rest of the writer, and keeping a copy purely to assert against would be the stale
    # second copy this repo has been bitten by. What is checkable from here is the
    # block's SHAPE, and a malformed row is a failure the equality never caught
    # either: a row is ``{side, why}``, ``side`` is one of the three the contract
    # defines, and ``why`` is prose someone wrote rather than an empty string standing
    # in for a reason. The block's *content* is compared against what the reference
    # writes, by ``tests/crossport/test_manifest_writer.py``.
    for pattern, entry in sorted(authority.items()):
        assert set(entry) == {"side", "why"}, (pattern, sorted(entry))
        assert entry["side"] in {"rust", "python", "hand"}, (pattern, entry["side"])
        assert len(entry["why"]) > 10, (pattern, entry["why"])


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
        row = _authority_for(f"scenarios/{name}/golden_sha256", manifest["_authority"])
        assert row is not None, name
        side = row[1]["side"]
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


# ⚠ **`_regenerate()` and the `__main__` LEFT WITH THE WRITER.** Regeneration on a
# deliberate unfreeze is `cargo run --example dump_station_inventory --
# --write-manifest`, run from `rust/`. This module is a **checker only** now, like its
# two siblings.
