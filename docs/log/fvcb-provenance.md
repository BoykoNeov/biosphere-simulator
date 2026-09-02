## **The FvCB constants bound to their sources** (the direction plan's standing free item, taken — and the regeneration tool could not give the ceremony its control)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> Plan of record: `post-roadmap-fvcb-provenance.md`. Filed under the September direction
> plan (referred to by that name here, never by filename — see the log's exemption note).

**BUILT 2026-09-02**, in an unattended session, as a provenance-only unfreeze: `gamma_star`,
`kc` and `ko` bound to Bernacchi et al. (2001), `o2` to the atmosphere's mole fraction; one
param-file hash and five `science_bands.source` strings moved in the biosphere manifest, **no
number, no golden**. The manifest diff was predicted as six lines before regenerating and
held exactly. ⚠ The advisor review the ceremony names is **owed on the branch**, not taken
before the regeneration — an unattended session cannot take it.

**FINDING 1 — the values were the paper's to every digit, and the file never said so.**
404.9 / 278.4 / 42.75 are Bernacchi et al. (2001)'s 25 °C Kc / Ko / Γ*; no other
parameterization on the shelf gives that triple (Teh's route gives Γ* = 40.4, and the gate
`the_shipped_floor_is_the_conservative_one_against_the_cited_route` has measured that gap
since 2026-08-14). Fifth instance of *check your own shelf before treating a value as
unsourced* — the first where the shelf was the literature the values were copied from rather
than another file of ours. ⚠ **Bound without the page in hand**: `sources/` holds no PDF and
every scholarly host was blocked from the box (PubMed, Semantic Scholar, Crossref), so the
binding is to the constants as the FvCB literature reproduces them, stated in the file header
and in each `source:` string. The page-level check is owed and can only confirm matching
digits or expose a transcription this record would then be wrong about; it cannot move the
value, which was never on the table.

**FINDING 2 — the blessed regeneration path could not give the ceremony its control.** On
the untouched tree, on Linux, `regen_goldens` reported `19 of 19 goldens run; 11 would
change` while `cargo test` was green (1101 passed). The eleven are the transcendental
goldens: UCRT-minted, last-ULP different under glibc, and compared *structurally* by the
gate off their generation platform — but the tool byte-compared. A tool that calls the
reference "moved" on a tree nothing moved cannot serve as the before/after control an
unfreeze needs, and a `--write` there would have re-minted eleven goldens on the wrong
platform. Fixed: `station/src/regen.rs` now reaches the gate's verdict through
`domains::goldens::compare`, reports such goldens as `ulp-only`, counts them separately, and
never rewrites them even under `--write`. Control test seeded with a fresh golden minus one
hex digit; a real (non-float) difference stays `CHANGED`. After the fix the untouched tree
reads **0 would change, 11 ulp-only**, and the edited tree reads the same. ⚠ A tooling fix
inside a science batch, named rather than slipped in: it re-anchors nothing.

**FINDING 3 — five frozen strings asserted the citation was missing, and one of them said
"two".** All five compensation-point band `source` strings read *"Γ* is TODO(cite)"* and named
a Python test by its `test_` prefix, residue C4 left on purpose and S6 did not retire. Both
claims went false with the citation, so they moved in the same ceremony, as frozen content,
under this freeze's own rule that a correction lands where it is written. The module note
that catalogued them said two strings; there were five.

**Two form gaps named, neither built.** `o2` is a 210 mmol/mol constant while every sealed
chamber carries O₂ as a live stock, so oxygenation sees an atmosphere the crop no longer
breathes; and Kc/Ko/Γ* are single-temperature constants under the [B] cardinal multiplier
while the paper now cited gives each its own Arrhenius response. The second is a cited
alternative FORM of a biosphere process — the thing the science-switch plan's slice 3b
measured the tree as lacking. Both are on the September direction plan.

**Gates run on the committed tree, counts read off the whole output, written after the run
rather than before it:** `cargo test --workspace --no-fail-fast` from `rust/` — **1103 passed, 0 failed**
across 64 result lines, exit 0 (1101 + the two tests this batch adds — the regen control
and the direction plan's re-read gate; the total moved by exactly the tests added, which is
the check the 2026-09-01 record asked for);
`cargo clippy --all-targets -- -D warnings` clean; `cargo run --release -q -p station
--example regen_goldens` → **19 of 19 run; 0 would change; 11 ulp-only** on Linux;
`uv run pytest` → 8 passed, 3 skipped; `ruff check`, `ruff format --check` and `pyright` clean.
