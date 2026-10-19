# Stale record clean-up — Step 8 of the 2026-09-29 review

**Opened and built 2026-10-19** on the user's *"1, 2 and 3 — do them"* (Step 8 was item 2 of three
offered). Slices 1 and 2 of the review's Step 8; **slice 3 (one "what this is today" page per domain)
was not offered and is not built.** Comments and docs only: no value, golden, manifest or `source:`
string moved.

## 1. The premise, re-checked on the day

The review counted 648 lines in 87 Rust files mentioning Python, and quoted one false rule:
`domains/src/biosphere/mod.rs`, *"If the two ever disagree, Python is right by definition."*

* **That line was already gone.** `e15b9f3` (the 1/16-day step, 2026-09-30) removed it.
* **The count was a different rule's.** Matching `python|\.py\b|pytest` (case-insensitive) outside
  `simcore` gives **873** lines in the Rust crates on 2026-10-19: **818 comment lines** (`//`, `///`,
  `//!`) and **55 code lines** — the latter are string literals, almost all manifest notes in the three
  `freeze_manifest.rs` writers, and editing one is a manifest change, not a clean-up.
* A further **~140 test comments said `Mirrors \`test_x\``** with no `.py` (the count misses them):
  pointers to deleted Python tests worded as if those tests still existed.
* Step 4a (2026-10-18) made param-file comments free, so they joined this pass as the review intended.

## 2. The sort (slice 1)

Three classes, as the review set them. The rule used for each:

| Class | Rule | Action |
|---|---|---|
| **False rule** | a present-tense claim that Python is the reference, gates something, runs, pins, reads or keeps something — or a procedure step that runs Python tools | **edited** to past tense, with what replaced it where a successor was found |
| **Pointer to a deleted file** | a direction to go and look (`see the Python X`, `lives in tests/...py`, `Mirrors \`test_x\``) | **edited**: `Mirrors` → `Ported from`; "see X" → X plus `(deleted by S6; 4f7168e^)` |
| **Harmless history** | provenance labelled as such (`Ported from …`, `until C7 …`, dated records) | **left**, plus one crate-level note (below) |

`4f7168e` is S6's commit (2026-08-27) that deleted all of `src/` and the Python `tests/`; every deleted
name in a comment can be read at `4f7168e^`. A note at the top of the `domains`, `station`, `authoring`
and `config` crate roots now says so, once, so the ~500 provenance lines left as written do not each
need it: *a comment saying the code was "ported from" or "mirrors" Python records where it came from,
not a copy it must keep matching.*

**Not touched:** `simcore` (no path edits it); `docs/log/` (records); dated unfreeze-log entries in the
contract docs; every string literal; every param `source:` string.

## 3. The edit (slice 2)

| Where | Lines changed | What |
|---|---|---|
| Rust comments, 43 files | ~300 | 138 `Mirrors` → `Ported from`; ~90 present-tense claims → past; 4 crate headers (the `domains` one said its params *"arrive from the frozen Python loaders via a generated hex-float file"*, false since C1) |
| Param-file `#` comments, 23 files | 47 | 17× `(P4 / config/units.py)` → `(P4; now rust/crates/config/src/params.rs)`, the precedent `self_discharge.yaml` set on 2026-10-18; Python module pointers → their Rust files or "deleted by S6" |
| The four contract docs, live text only | ~100 | below |

**The contract docs held false procedure, not just stale pointers.** Each freeze doc's unfreeze
discipline told the reader to re-run `-m slow`, `ruff` and `pyright` (the Python suite S6 deleted), and
two said an undocumented unfreeze *"fails CI by construction"* — no CI job has run the reference's tests
since 2026-10-10. The authoring discipline's step 3 read *"Land it on BOTH ports."* The biosphere and
authoring disciplines told the reader to keep `git diff src/simcore/` empty — a directory that no longer
exists (the station doc had already been corrected the same way). All now name the Rust gates
(`cargo test --workspace --no-fail-fast`, `-- --ignored`, clippy) and say the run is by hand. Each
"Tests of record" line now names the Rust tests and lists the deleted Python ones as history. The
native-port doc, which opens with a banner of slice-5 corrections, gains a fourth: S6 and the CI deletion.

## 4. Found while sorting — gaps the comments were hiding

None is fixed here (comments and docs only); each is named for a later item.

1. **The station manifest names a test that no longer exists.** Its `delegates_to` note says the
   biosphere manifest path's existence *"is checked by test_manifest_named_files_exist"*. That was a
   Python test; S6 deleted it and **no Rust test checks the path exists**. The note is a manifest string,
   so correcting it is a station unfreeze; adding the Rust check is free.
2. **Manifest notes still describe deleted Python files as live.** All three `COMMENT` strings end
   *"tests/test_*freeze_manifest.py has none and is now only a checker"*; the authoring manifest's
   `parity_vectors/*` row still reads `python`, though its generator was deleted. Manifest changes, with
   the ceremony.
3. **Four param `source:` strings name deleted Python tests** (potato `canopy.yaml` carbon_fraction,
   `senescence.yaml` ×2, `stem_reserves.yaml`). A provenance-only unfreeze each.
4. **BVAD's seven bookkeeping checks were deleted unported.** S3 deferred `test_bvad_validation.py` as a
   science decision; S6 deleted it; only the one structural gate exists in Rust.
5. **Two authoring gates have no Rust successor by name**: *"`monod` is bit-identical to the frozen
   `f_O2`"* (`test_authoring_monod.py`) and the Godot "uncalibrated" banner check
   (`test_godot_from_file.py`). Not searched beyond names.
6. **A rooting-cap branch is pinned only by its unit test now.** The Python side also pinned it over a
   whole season; nothing in Rust does (`science.rs`, the shallow-soil test).

## 5. Checked

`cargo test --workspace --no-fail-fast`, `cargo test --release --workspace --no-fail-fast -- --ignored`,
`cargo clippy --workspace --all-targets -- -D warnings`, and `regen_goldens` (report mode), all at
below-normal priority — results in §6. No `.json`, golden or manifest file is in the diff.

## 6. Results (2026-10-19, Windows, below-normal priority, on the final tree)

* `cargo test --workspace --no-fail-fast`: **1447 passed, 0 failed, 12 ignored** over 103 result lines.
  The first run, on a tree mid-edit, was 1445 / **2 failed**, both `repo_gates` on this item's own
  paperwork (a memory index line at 247 B against the 240 B cap; the third direction plan's re-read
  marker); both fixed, with the re-read recorded in that plan.
* `cargo test --release --workspace --no-fail-fast -- --ignored`: **12 passed, 0 failed.**
* `cargo clippy --workspace --all-targets -- -D warnings`: clean. ⚠ The first pass was red on a
  doc-list indent that this item's own re-wrapping script had broken (`freeze_manifest.rs`).
* `regen_goldens` (report): **20 of 20 run, 0 would change.**
