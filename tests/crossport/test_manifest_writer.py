"""The committed freeze manifests are what the **reference** writes — slice C7.

`docs/plans/post-roadmap-reference-flip.md`. Until C7 the three
`docs/*-reference.manifest.json` files were assembled and serialized by Python:
`tests/test_freeze_manifest.py::_build_manifest()` shelled the reference's dump,
spliced its keys into the checker's own, and wrote the file. The manifest was therefore
*authored* by the reference and *written* by the checker — the Python-shaped hole in the
middle of a contract whose first line says Rust is the reference.

The writer now lives in the reference (`cargo run --example dump_biosphere_inventory --
--write-manifest`), and this is its gate: **regenerate into a temporary file and demand
the committed one is byte-identical.**

# What this catches that nothing else does

`test_inventory_parity.py` compares the reference's *derived sets* against the manifest,
axis by axis. It says nothing about the hand-authored half (the roster, the two
anti-derived literals, the `_authority` prose), nothing about serialization, and nothing
about the keys the checker still authors. This compares the whole file, so:

* a frozen surface that moved and was not regenerated is red (the same staleness
  `test_inventory_parity` catches, now for **every** key rather than the compared axes);
* a hand edit to the committed manifest is red — it is a generated artifact again, and
  before C7 a typo in `_comment` or a hand-fixed hash was invisible to every gate;
* a change to the writer's own serialization is red.

# ⚠ What it deliberately does NOT catch, measured before it was written

The step. `dt_days` is a hand literal precisely so a step change is a ceremony;
splicing `BIO_DT` into the writer instead produces a **byte-identical manifest**
(measured), so this gate is blind to it and so is the cross-port `dt` check. That guard
is a source-text one and lives with the writer,
`rust/crates/domains/tests/manifest_writer.rs`.

⚠ The authoring contract was measured for the same trap and has none: its hand-authored
keys are a phase number, two repo paths and two blocks of prose, and the `authoring`
crate owns no constant any of them could be spliced from. Recorded as "none" rather than
answered with a guard invented to match the biosphere's — a control with no test to
redden is the finding, not a gap to fill.

# ⚠ No pipe

The reference writes the file itself. Slice C4 froze cp1252-mangled prose into a
contract with every gate green because a `subprocess` pipe decoded UTF-8 with the
Windows locale and *both* sides were mangled identically. Nothing here decodes the
manifest — the bytes are compared as bytes.
"""

import shutil
import subprocess
from pathlib import Path

import pytest

_REPO_ROOT = Path(__file__).resolve().parent.parent.parent
_RUST_DIR = _REPO_ROOT / "rust"
_DOCS_DIR = _REPO_ROOT / "docs"

#: `(label, crate, example, manifest filename)` — one row per contract with a Rust
#: writer. ⚠⚠ **All three are here as of C7's station half, and the note that used to
#: stand in this place said what a missing row meant: a contract Python still writes,
#: which is a gap and not a policy.** The station row waited on slice C4b, which gave
#: the reference a referent for the two station science claims a Rust writer would
#: otherwise have had to hand-carry. There is no fourth contract —
#: `docs/phase-8-reference.md` is deliberately a doc with no manifest — so this table is
#: now the whole set, and a contract added without a row here is the gap the old note
#: describes.
_WRITERS = [
    (
        "biosphere",
        "domains",
        "dump_biosphere_inventory",
        "biosphere-reference.manifest.json",
    ),
    (
        "authoring",
        "authoring",
        "dump_authoring_inventory",
        "authoring-reference.manifest.json",
    ),
    (
        "station",
        "station",
        "dump_station_inventory",
        "station-reference.manifest.json",
    ),
]


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
@pytest.mark.parametrize("label,crate,example,manifest_name", _WRITERS)
def test_the_committed_manifest_is_what_the_reference_writes(
    label: str, crate: str, example: str, manifest_name: str, tmp_path: Path
) -> None:
    """Regenerate into `tmp_path` and compare the bytes.

    Runs wherever `cargo` is on PATH, and on the `crossport` CI job — which invokes
    `uv run pytest tests/crossport/` on the **directory**, so this file is collected
    without being listed anywhere. (Checked, not inherited: this repo has two recorded
    green-by-skip incidents.)
    """
    target = tmp_path / manifest_name
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "--example",
            example,
            "-p",
            crate,
            "--",
            "--write-manifest",
            str(target),
        ],
        cwd=_RUST_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    assert proc.returncode == 0, (
        f"cargo run {example} --write-manifest failed:\n{proc.stderr}"
    )
    assert target.is_file(), f"{example} wrote no file at {target}"

    committed = (_DOCS_DIR / manifest_name).read_bytes()
    regenerated = target.read_bytes()
    assert committed == regenerated, (
        f"the committed {manifest_name} is not what the reference writes today.\n"
        "Two readings, and the first question decides which:\n"
        "  * the reference tree changed and the manifest was not regenerated — that is "
        "an UNFREEZE. Follow the ceremony in the contract's own doc, then re-run the "
        "writer and review the diff.\n"
        "  * the manifest was edited by hand — it is a generated artifact; the edit "
        "belongs in the writer "
        f"(rust/crates/{crate}/examples/{example}.rs), which is what makes it "
        "reproducible.\n"
        f"Regenerate with: cd rust && cargo run --example {example} -- --write-manifest"
    )


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
@pytest.mark.parametrize("label,crate,example,manifest_name", _WRITERS)
def test_the_writer_refuses_an_unknown_argument(
    label: str, crate: str, example: str, manifest_name: str
) -> None:
    """⚠ The control on the gate above, and it is not pedantry.

    The test passes `--write-manifest <path>`. A writer that ignored its arguments and
    wrote its default location would leave `tmp_path` empty — caught by the `is_file`
    assert — but a writer that ignored the *flag* and wrote the file anyway would make
    the comparison pass while proving the wrong thing. So the argument handling is
    asserted to be real: an unknown flag exits non-zero rather than falling through to
    the dump or to a default write.

    ⚠ **Parametrized over `_WRITERS` since the authoring writer landed.** It was
    hardcoded to the biosphere example while that was the only row, which would have
    left every later writer's argument handling unasserted — and by this test's own
    reasoning that is exactly the state in which the byte comparison passes while
    proving the wrong thing.
    """
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "--example",
            example,
            "-p",
            crate,
            "--",
            "--nonsense",
        ],
        cwd=_RUST_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    assert proc.returncode != 0, "an unknown argument must not be silently ignored"
    assert "usage:" in proc.stderr
