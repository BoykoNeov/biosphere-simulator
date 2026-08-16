"""Reference flip, slice 4: every golden is classified, and Rust's bytes are pinned.

Slice 4 is where the reference moves — *the goldens are generated from Rust*. Two things
have to be true before that sentence means anything, and this module gates both.

**1. The census is complete.** Every file in ``tests/regression/golden/`` is classified
into exactly one of the three groups in ``regen_goldens_from_rust``: Rust emits its
bytes, Rust emits a raw series that Python folds, or Rust has no referent for it at all.
A golden added without a classification is red here — the same forcing-function move
slice 3 made with the dump's exact key set, and for the same reason: the alternative is
a filter that lets an unclassified artifact through silently.

**2. Rust's bytes are pinned tighter than the tolerance band.** The Tier-2 comparison
next door passes anything inside ``1e-11``; two goldens drifted to ~2 ULP under it
without a single test noticing. The byte census below is ~5 orders tighter, so the
next
pair to drift apart is a red test rather than a stale sentence.

⚠ **Slice 5 made that census unconditional, and that is the inversion.** Slice 4's
version consulted a two-entry exemption roster, because the golden was Python's output
and the two ports genuinely disagreed on two of them. The goldens are now Rust's own
bytes, so ``Rust == golden`` has exactly one allowed answer and there is no roster on
this side any more. The exemptions moved to the *checker*
(``golden_platform.PYTHON_DIVERGES``), which is what "Python becomes the tolerance-gated
side" means in practice.

⚠ **What none of this can establish.** While the two ports emit identical bytes, no
byte-level check can say which side produced a golden — provenance does not survive in
the artifact. What slice 4 makes structural is the **path**
(``regen_goldens_from_rust.py``), and what slice 5 adds is that the *Python* paths now
refuse (``golden_platform.write_python_golden``). Neither is a property of the files.

⚠ **The byte census is Windows-gated and that is not a green-by-skip.** The goldens are
UCRT-generated (``tests/golden_platform.py``); on the Ubuntu ``crossport`` job
glibc-Rust
against a UCRT golden differs at the last ULP *by design*, which is exactly what the
Tier-2 bands exist to absorb. Byte identity is only a meaningful question on the
generation platform, so this test skips on CI **and the tests beside it do not** — the
four cheap classification tests below are pure Python and run everywhere.
"""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent))
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import regen_goldens_from_rust as regen  # noqa: E402

import golden_platform  # noqa: E402
from golden_platform import windows_golden_only  # noqa: E402

DOCS_DIR = regen.REPO_ROOT / "docs"
MANIFESTS = ("biosphere-reference.manifest.json", "station-reference.manifest.json")


def _frozen_goldens() -> set[str]:
    """The goldens the two freeze manifests name. ⚠ Derived, never hand-listed."""
    frozen: set[str] = set()
    for name in MANIFESTS:
        manifest = json.loads((DOCS_DIR / name).read_text(encoding="utf-8"))
        frozen |= {entry["golden"] for entry in manifest["scenarios"].values()}
    return frozen


# --------------------------------------------------------------------------- #
# 1. The census — pure Python, runs everywhere                                 #
# --------------------------------------------------------------------------- #


def test_every_committed_golden_is_classified() -> None:
    """The three groups partition the goldens on disk — no leftovers, no phantoms.

    ⚠ Enumerated from the directory rather than from any roster: this repo has already
    been caught believing a hand-maintained list was the census
    (`docs/log/coverage-roster-is-not-the-manifest.md`).
    """
    classified = (
        set(regen.RUST_EMITTERS)
        | set(regen.PYTHON_FOLDED)
        | set(regen.NO_RUST_REFERENT)
    )
    on_disk = regen.committed_goldens()
    assert classified == on_disk, (
        "the golden census in regen_goldens_from_rust is out of step with the "
        "directory:\n"
        f"  unclassified on disk: {sorted(on_disk - classified)}\n"
        f"  classified but gone:  {sorted(classified - on_disk)}\n"
        "⚠ A new golden must be placed in one of the three groups — with the *reason* "
        "written beside it if it has no Rust referent. Do not widen this assertion."
    )


def test_the_three_groups_are_disjoint() -> None:
    """A golden cannot be both Rust-emitted and Rust-less."""
    groups = (regen.RUST_EMITTERS, regen.PYTHON_FOLDED, regen.NO_RUST_REFERENT)
    seen: set[str] = set()
    for group in groups:
        overlap = seen & set(group)
        assert not overlap, f"golden classified twice: {sorted(overlap)}"
        seen |= set(group)


def test_every_frozen_golden_has_a_rust_emitter_or_a_stated_reason() -> None:
    """Each of the 20 frozen goldens is either Rust-emitted or explained.

    This is the classification slice 6 is told to do *before* its ceremony rather than
    during one (`post-roadmap-reference-flip.md` §5, slice 6). On the golden axis the
    answer is: **18 yes, 2 no** — both folded summaries, whose Rust programs emit a raw
    series that `drift.py` turns into the artifact Python-side.
    """
    frozen = _frozen_goldens()
    assert len(frozen) == 20, f"expected 20 frozen goldens, found {len(frozen)}"
    unexplained = frozen - set(regen.RUST_EMITTERS) - set(regen.PYTHON_FOLDED)
    assert not unexplained, (
        f"frozen goldens with neither a Rust emitter nor a stated reason: "
        f"{sorted(unexplained)}"
    )
    # ⚠ And no frozen golden may be parked in the no-referent group: that group is for
    # additive Python-only pins, and a frozen contract quietly landing there is how a
    # re-anchored manifest ends up carrying a field nothing on the reference produces.
    assert not (frozen & set(regen.NO_RUST_REFERENT)), (
        f"frozen golden classified as having no Rust referent: "
        f"{sorted(frozen & set(regen.NO_RUST_REFERENT))}"
    )
    for name in sorted(frozen & set(regen.PYTHON_FOLDED)):
        assert len(regen.PYTHON_FOLDED[name]) > 40, (
            f"{name} is a frozen golden with no Rust referent and no real reason "
            "written beside it"
        )


def test_the_emitter_map_points_at_real_programs() -> None:
    """Every mapped ``(crate, example)`` is a file in the Rust tree.

    Catches a typo in the map without paying for a cargo run — and, because the map is
    keyed by crate, it is also what makes the `-p` disambiguation checkable at all.

    ⚠ **The crate key is doing two jobs, and they are not the same name.** It locates a
    *directory* here and it is passed to ``cargo run -p`` as a *package*. Those agree
    across all four crates today, and nothing but this assertion says they must: if they
    ever diverged, this test would pass while every regeneration failed at runtime. Loud
    rather than silent, but there is no reason to leave the map checkable only by
    running it.
    """
    for name, emitter in regen.RUST_EMITTERS.items():
        assert emitter.source.is_file(), (
            f"{name} maps to {emitter.crate}/examples/{emitter.example}.rs, "
            f"which does not exist"
        )
        cargo_toml = emitter.source.parents[1] / "Cargo.toml"
        declared = [
            line.split("=", 1)[1].strip().strip('"')
            for line in cargo_toml.read_text(encoding="utf-8").splitlines()
            if line.startswith("name")
        ]
        assert declared[:1] == [emitter.crate], (
            f"{name} maps to the directory `crates/{emitter.crate}/`, but that crate's "
            f"Cargo.toml declares the package name {declared[:1]} — `cargo run -p "
            f"{emitter.crate}` would not resolve"
        )


def test_the_crew_golden_maps_to_the_emitter_that_computes_it() -> None:
    """⚠ The one place in the map where the wrong choice is a silent round trip.

    ``emit_crew`` exists in **two** crates. ``simcore``'s parses ``crew_state.json``'s
    own hex-floats and re-emits them — a codec fixture, deliberately, since Phase-7
    Step 0. ``domains``' *computes* the mission from the ported engine. Regenerating the
    golden from the ``simcore`` one would write the file from itself, which is the exact
    shape slice 3 refused to ship for ``param_files``.

    The two are also the pre-existing cargo output-filename collision, so a regeneration
    that shelled ``target/*/examples/emit_crew.exe`` would pick whichever built last.
    ``Emitter.command`` never builds such a path; this pins the crate it does use.
    """
    assert regen.RUST_EMITTERS["crew_state.json"].crate == "domains", (
        "crew_state.json must be regenerated from `domains`' computing emitter — "
        "`simcore`'s emit_crew reads the golden and echoes it back"
    )
    assert "-p" in regen.RUST_EMITTERS["crew_state.json"].command()


# --------------------------------------------------------------------------- #
# 2. The byte census — Windows + cargo only                                    #
# --------------------------------------------------------------------------- #


@pytest.mark.slow
@windows_golden_only
@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
@pytest.mark.parametrize("golden", sorted(regen.RUST_EMITTERS))
def test_rust_reproduces_the_committed_golden_bytes(golden: str) -> None:
    """Rust's stdout equals the committed golden, byte for byte. **No exemptions.**

    ⚠ Slice 4's version of this test carried a two-entry roster, because the golden was
    Python's output and two of the eighteen genuinely differed. Slice 5 regenerated
    those
    two from Rust, so the golden *is* this program's output and an inequality here has
    only one meaning: **the reference moved**. Either the Rust engine changed and the
    goldens have not been regenerated, or something wrote them from elsewhere.

    The remedy is never to widen this test. Run ``uv run python
    tests/crossport/regen_goldens_from_rust.py`` to see the diff, decide whether the
    change to the reference is intended, and if so ``--write`` it *and* re-run the
    freeze-manifest ceremony for whichever contract names it.
    """
    produced = regen.RUST_EMITTERS[golden].run()
    assert produced == (regen.GOLDEN_DIR / golden).read_bytes(), (
        f"{golden}: the committed golden is no longer this Rust emitter's output.\n"
        "⚠ Since the reference flip the golden IS Rust's bytes, so this is the "
        "reference moving, not a port disagreement. See the docstring."
    )


def test_the_two_authorship_rosters_name_the_same_files() -> None:
    """The names in ``golden_platform`` and the commands here describe one set.

    ⚠ The duplication is deliberate and this is the gate that makes it safe. The base
    regression suite needs to know *which* goldens Rust authors (to refuse writing
    them);
    it must not need ``tests/crossport`` on its import path to find out. So the names
    live in ``golden_platform.RUST_AUTHORED`` and the ``cargo`` invocations live in
    ``regen.RUST_EMITTERS``, one direction only — and they are asserted equal here
    rather
    than trusted to stay in step.
    """
    authored = set(golden_platform.RUST_AUTHORED)
    emitters = set(regen.RUST_EMITTERS)
    assert emitters == authored, (
        "the two authorship rosters have diverged:\n"
        f"  emitter map only: {sorted(emitters - authored)}\n"
        f"  name roster only: {sorted(authored - emitters)}"
    )


def test_the_divergence_roster_is_a_subset_of_what_rust_authors() -> None:
    """Python can only *diverge from the reference* for a golden Rust actually wrote."""
    stray = set(golden_platform.PYTHON_DIVERGES) - golden_platform.RUST_AUTHORED
    assert not stray, (
        f"goldens on PYTHON_DIVERGES that Rust does not author: {sorted(stray)}. "
        "A Python-authored golden cannot disagree with itself; if one of these is "
        "genuinely unstable, that is a determinism bug, not a tolerance."
    )


def test_every_diverging_scenario_keeps_a_byte_gated_sibling() -> None:
    """⚠ No scenario may lose its **last** byte-exact Python gate to the roster.

    A tolerance-gated golden cannot see a reduction-order change: canonical flow-id
    order
    on every reduction is a non-negotiable invariant, and reordering moves values by a
    ULP or two — inside any band. Today both rostered goldens are safe by luck of the
    census: ``emit_consumer`` and ``emit_perennial`` each serve **two** goldens (a 5-yr
    and a 15-yr horizon), and in both cases the sibling is still byte-gated.

    That is an observation, so it is asserted rather than written down. A third entry
    landing on the roster — or a second entry for the same scenario — is red here, which
    is the only way "coverage survives" stays true instead of becoming a stale sentence.
    The emitter program is the scenario key: same ``(crate, example)``, same scenario,
    different argument.
    """
    for golden in sorted(golden_platform.PYTHON_DIVERGES):
        emitter = regen.RUST_EMITTERS[golden]
        siblings = {
            name
            for name, other in regen.RUST_EMITTERS.items()
            if (other.crate, other.example) == (emitter.crate, emitter.example)
            and name != golden
        }
        byte_gated = siblings - set(golden_platform.PYTHON_DIVERGES)
        assert byte_gated, (
            f"{golden} is tolerance-gated on the Python side and no other golden from "
            f"`{emitter.crate}/{emitter.example}` is still byte-gated. That scenario "
            "now has NO Python-side gate that can see a reduction-order change. "
            "Diagnose the divergence instead of adding it to the roster."
        )
