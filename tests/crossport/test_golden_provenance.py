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
without a single test noticing, and ``tiers.json`` still describes them as
``max_rel_dev 0.0``. The byte census below is ~5 orders tighter, so the *next* pair to
drift apart is a red test rather than a stale sentence.

⚠ **What none of this can establish.** While the two ports emit identical bytes, no
byte-level check can say which side produced a golden — provenance does not survive in
the artifact. What slice 4 makes structural is the **path**
(``regen_goldens_from_rust.py``), not a property of the files.

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

import compare  # noqa: E402
import regen_goldens_from_rust as regen  # noqa: E402

from golden_platform import windows_golden_only  # noqa: E402

DOCS_DIR = regen.REPO_ROOT / "docs"
MANIFESTS = ("biosphere-reference.manifest.json", "station-reference.manifest.json")

# The ceiling the two known-divergent goldens must stay under. ⚠ This is a *last-bit
# noise* ceiling, not a science band: the measured worst is 4.6e-16, this is ~20x above
# it and **1000x tighter than the Tier-2 band** those scenarios are otherwise gated at.
# A real defect in either scenario blows through it long before `tiers.json` would care.
DISAGREEMENT_CEILING = 1e-14


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
    """
    for name, emitter in regen.RUST_EMITTERS.items():
        assert emitter.source.is_file(), (
            f"{name} maps to {emitter.crate}/examples/{emitter.example}.rs, "
            f"which does not exist"
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
    """Rust's stdout equals the committed golden, byte for byte — bar a named roster.

    Sixteen of the eighteen are byte-identical on this UCRT box. The two that are not
    are listed in ``regen.PORTS_DISAGREE`` with their measured size, and this test
    checks that roster **in both directions**: a golden joining it is red, and a golden
    *leaving* it is red too, so the roster cannot quietly become a stale exemption
    nobody re-measures. (An exemption written for a temporary state, left in place after
    the state passed, is this log's own hardest lesson.)
    """
    emitter = regen.RUST_EMITTERS[golden]
    produced = emitter.run()
    current = (regen.GOLDEN_DIR / golden).read_bytes()
    expected_to_differ = golden in regen.PORTS_DISAGREE

    if produced == current:
        assert not expected_to_differ, (
            f"{golden} is recorded in PORTS_DISAGREE as "
            f"{regen.PORTS_DISAGREE[golden]!r}, but the ports now agree byte for byte. "
            "Drop it from the roster — a divergence that healed must not be left "
            "standing as an exemption."
        )
        return

    assert expected_to_differ, (
        f"{golden}: Rust's bytes no longer equal the committed golden, and it is not "
        "in the recorded disagreement roster.\n"
        "⚠ This is ~5 orders tighter than the Tier-2 band next door, so it is a "
        "*finding*, not necessarily a failure: the ports have drifted apart at the "
        "last bit somewhere. Measure it (uv run python "
        "tests/crossport/regen_goldens_from_rust.py), decide whether it is accumulated "
        "noise or a real op-level difference, and record it — do not widen the band."
    )

    # It differs, and it is allowed to. Pin *how much*: structure exact, deviation under
    # a last-bit-noise ceiling far below the scenario's own Tier-2 band.
    result = compare.compare(
        compare.load_json(regen.GOLDEN_DIR / golden),
        json.loads(produced.decode("utf-8")),
        tier=2,
        band=DISAGREEMENT_CEILING,
        floor=1e-12,
    )
    assert result.ok, (
        f"{golden} is a known last-bit disagreement "
        f"({regen.PORTS_DISAGREE[golden]}), but the divergence has grown past "
        f"{DISAGREEMENT_CEILING:g}:\n{result.report()}"
    )
    assert result.numeric_pairs, "expected numeric leaves to be compared"
