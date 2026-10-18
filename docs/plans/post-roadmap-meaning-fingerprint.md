# Fingerprint what a param file means, not its text — Step 4 of the 2026-09-29 review

**Opened 2026-10-18** on the user's *"1 and 2"* (Step 4, "cheaper change", and Step 7,
"authoring can reach plants", of the review proposal written 2026-09-29). This doc is Step 4.

## 1. The problem, re-checked on the day

The review plan's premise was that a comment-only edit to a frozen param file moves its manifest
hash, so it is a formal unfreeze, so comments known to be false are left in place. Checked against
the tree on 2026-10-18:

* **Still true.** Both manifests recorded `normalized_sha256` of each file's *text*
  (`config::provenance`), so any byte of a comment moved a frozen entry.
* **One of the two named stale comments is gone**: `senescence.yaml`'s `shade_rate` note was annotated
  STALE in place on 2026-10-06 (`e9a70da`), when that file's entry moved for a real reason. **The other is still
  there**: `self_discharge.yaml` points three times at Python files deleted in S6
  (`test_power_self_discharge.py`, `scenario.py`, `config/units.py`), where the proof now lives in
  `rust/crates/domains/tests/power_self_discharge.rs`.

## 2. 4b — decided, not asked

Option 1, *record and live with it*: the eleven transcendental goldens regenerate on Windows only.
The user's standing call of 2026-10-10 ("Linux is not a goal"; the Linux CI job deleted) settles it.
Option 2 (the project's own `exp`/`sin`) is re-opened only if a second development machine becomes
real.

## 3. 4a — what was built

`config::provenance::param_fingerprints(text)` parses the file with the loaders' own reader
(`config::yaml::parse_document`, after newline normalization) and returns two sha-256s:

| fingerprint | over | moves on |
|---|---|---|
| `values` | the whole parsed document with each `parameters.<name>.source` removed | a value, unit, name, process or table edit |
| `sources` | the `parameter -> source` pairs, file order | a citation edit |

Comments, blank lines, indentation and line endings enter neither. The encoding is tagged and
length-prefixed, so two different trees cannot encode to the same bytes.

**Deliberately still counted as a change** (each a safe-side miss — it costs a ceremony, never the
freeze):

* **A scalar is its text.** `0.60` → `0.6` moves `values`. The advisor suggested hashing numbers by
  value; not taken, because this module would then decide which scalars are numbers — the schema
  decides that per field — and a wrong call is a *missed* change.
* **Key order** and **quoting** (the scenario schema reads `"1.0"` as a string).
* Only a parameter's *own* `source:` scalar is a citation; a parameter *named* `source` stays in
  `values` (tested).

The manifests' `param_files` entries become `{sources, values}` (`param_files_json`), in both
writers and both inventory printers. The authoring manifest hashes no param file and is untouched.

## 4. The ceremony, and what moved

Re-anchoring ceremony on the biosphere and station manifests. **Predicted diff: `param_files` and
its `_authority` row in each, nothing else. Measured: exactly that** (a key-by-key compare of the
regenerated manifests against copies taken before). No golden, set, claim, horizon or census moved.

Prose updated: both reference docs (the provenance sentence, the `_authority` table row in the
biosphere doc, the goldens-vs-manifest paragraph, an unfreeze-log entry each),
`docs/param-file-conventions.md` (the line-endings section marked history, the "whitespace edit is
an unfreeze" paragraph struck and superseded, two checklist items), and one clause of `CLAUDE.md`.

## 5. Controls, kept as tests

* `config::provenance` tests, on a fixture: comment/layout/CRLF/CR edits move nothing; a citation
  edit (an entry's and a table's) moves only `sources`; seven value-side edits move only `values`;
  a parameter named `source`; the encoding does not collide across a key/value or sequence split.
* `rust/crates/station/tests/param_fingerprints.rs`, on all 25 real files: stripping every comment
  moves nothing; a citation edit moves only `sources` (all 25 cite); a value edit moves only
  `values`.
* **Mutation:** making the extraction look for a key that does not exist turned one test red in
  each file, run with `--no-fail-fast`.

**The live control** is the follow-up commit: correcting `self_discharge.yaml`'s stale pointers must
leave both manifests byte-identical.
