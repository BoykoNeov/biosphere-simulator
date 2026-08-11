# Post-roadmap work log

**The roadmap has no Phase 10 — everything here was chosen, not scheduled.** One row
per piece of work, newest concerns last; each names its own plan doc, which is the
fuller record (`docs/plans/post-roadmap-*.md`).

**Why this file exists.** These rows lived in `CLAUDE.md` until 2026-08-11, where they
had grown to ~206 KB — about 50k tokens loaded into *every* session, against a file
whose own opening line calls itself "a map, not a record". The rows were moved here
**verbatim**, byte-for-byte, rather than summarised: condensing 23 rows by hand is
exactly how a finding gets silently dropped.

**That fix did not hold, and this file now carries both halves.** `CLAUDE.md` kept a
one-line index row per entry — and those rows averaged 243 bytes, so the index re-grew
14.4 KB → 17.7 KB *the same day* it was cut. Moving the record without changing the rule
that generates rows only bought a few weeks. On 2026-08-12 the index moved here too,
again verbatim, and `CLAUDE.md` now points at this file instead of summarising it. The
diagnosis and the rules are in `docs/context-budget.md`; the size ceiling has a paired
test (`tests/test_context_budget.py`), because every other contract in this repo has one
and this one had re-bloated inside 24 hours without it.

**And the record then had to leave this file too, on 2026-08-12.** Not for size — for
shape: one work item was one physical *line* of a markdown table, up to 54,343 characters
of it, which defeats `Grep`, `Read` and `git diff` alike. The record now lives one file
per item in `docs/log/`, moved verbatim and digest-checked; the table below is pointers.
That is rule 4 of `docs/context-budget.md`.

So: **this file is the index, plus a pointer table into `docs/log/`.** New work adds one
line to the index, one pointer row, and one file in `docs/log/` — and adds nothing to
`CLAUDE.md`. The paired test fails if a row exists on one side and not the other.

⚠ Prose elsewhere in the tree still points at "the CLAUDE.md status row" (the
compose-gap discipline: *re-read the status row for the bucket you closed*). That rule
survives both moves; **its target is now the item's file in `docs/log/`** — the index line
and the pointer row here are navigation, not the record.
`src/domains/power/params/self_discharge.yaml`
also refers to something "CLAUDE.md left on the table" — deliberately **not** edited,
because a comment-only edit to a param file moves its manifest hash and is an unfreeze
event; a pointer fix is not worth a freeze ceremony.

## Index — one line per row, in the same order

Navigation only; **the record is one file per row in `docs/log/`.** This index moved here verbatim
from `CLAUDE.md` on 2026-08-12, under the retirement rule in `docs/context-budget.md`:
it was 7.3 KB loaded into *every* session, to index a file most sessions never open.

| Work | Status | Detail |
|---|---|---|
| Development posture — the Rust-primary pivot | **DECIDED 2026-07-20: Option A** — new content is Rust-first; Python stays frozen-canonical and green for validated science | `post-roadmap-rust-primary-pivot.md` |
| Scope (B): the day-neutral habitat crop | COMPLETE 2026-07-20 — diagnosed against an offline LINTUL3 oracle; authored, not validated; no golden moved | `post-roadmap-day-neutral-crop.md` |
| Scope (B): decomposer calibration | COMPLETE 2026-07-21 — the carbon rates moved above-range → top-of-range; biosphere re-frozen. Closure requires the fast edge | `post-roadmap-decomposer-calibration.md` |
| The nitrogen-cycle FORM gap | (A)+(B) **BUILT** 2026-07-27; (C) **REFUSED**; (D) **not buildable as recorded**. Two weakly-sourced params retired by changing the form, not by finding a citation | `post-roadmap-nitrogen-cycle-form.md` |
| The canopy regulator | DIAGNOSED 2026-07-27, **NOT BUILT** — fixes the canopy and is bit-identically inert on every frozen scenario | `post-roadmap-canopy-regulator.md` |
| The chamber-scale diagnosis | DIAGNOSED 2026-08-09 — the sealed jar holds ~2 days of one crop's carbon; enlarging it is refuted by BVAD. The defect is the gate, not the chamber | `post-roadmap-chamber-scale.md` |
| The acceptance gate | DIAGNOSED 2026-08-09 — the six tightest margins in the whole roster are one stock in one rig; `open_season`'s carbon source is unclamped and holds 0.0. **Its finding 6 ("the decision is the user's") was ADJUDICATED the same day — see the next row** | `post-roadmap-acceptance-gate.md` |
| The science assertions get contract standing | COMPLETE 2026-08-09 — `science_bands` + `liveness_floors` in both manifests; a schema unfreeze, no value moved | `post-roadmap-acceptance-gate-standing.md` |
| The humification split (a CUE) | **BUILT 2026-08-10** — CENTURY's partition; the settling transient grew ~3 yr → ~35, past the frozen horizon | `post-roadmap-cue-humification.md` |
| Soil carbon pool fractionation | **REFUSED TWICE** (2026-08-10) — `decomposition_rate` measured un-retirable by the only cited alternative on the shelf | `post-roadmap-soil-fractionation.md` |
| The crew-coupled loop | TAKEN and **REFUSED** 2026-08-10 — a chamber is carbon-limited by *isolation*, not volume; the two-rate split caps the crop at the standing pool | `post-roadmap-crew-coupled-loop.md` |
| The decade CO₂ guard, re-anchored | COMPLETE 2026-08-10 — the window was measured inert on the frozen tree and removed; a tightening, not a re-tune | `post-roadmap-co2-guard-reanchor.md` |
| Stem-reserve remobilization | DIAGNOSED + PRICED 2026-08-10, **NOT BUILT** — the stem cannot feed the seed; blocked on the uncited partition table, which is the real successor | `post-roadmap-stem-reserves.md` |
| Test-suite runtime (tooling, not science) | COMPLETE 2026-08-09 — `-n 12`, below-normal priority class; whole suite 7m05s | `docs/test-suite-runtime.md` |
| The first authored habitat | COMPLETE | `post-roadmap-authored-habitat.md` |
| The second authored habitat (`bioregenerative_station`) | COMPLETE 2026-08-11 — a frozen flow's "boundary" wiring field is a **name, not a constraint**, so the calibrated equipment recycles | `post-roadmap-bioregenerative-station.md` |
| Tier 1: grow the flow registry | COMPLETE — authoring platform unfrozen (`flow_types` 3→12, `param_loaders` 2→5) | `post-roadmap-flow-registry-growth.md` |
| Tier 2: the grammar — `monod` | COMPLETE — grammar unfrozen (`expr_nodes` 7→8); saturation is now sayable | `post-roadmap-grammar-monod.md` |
| Bucket 3 (A): diagnose + pin the oracle gap | COMPLETE — the gap is **structural**, so the deferred quantitative match is not a calibration task | `post-roadmap-validation.md` |
| Bucket 3 (B): the full oracle match | INCREMENT 1 + CEREMONY 2 COMPLETE 2026-07-20 — vernalization + photoperiod shipped. **The oracle is a diagnostic, never a fit target** | `post-roadmap-oracle-match.md` |
| Bucket 2: the export-fidelity hazard | COMPLETE — `rationed == 0` is not "the export is right"; closed by multi-rate Step 5's build-time `k·h < 1` precondition | `tests/test_authoring_export_fidelity.py` |
| Bucket 2: multi-rate authoring | COMPLETE — all 7 steps; the authoring platform re-frozen with the multi-rate surface in it, both ports | `post-roadmap-multirate-authoring.md` |
| Bucket 2: the rationing gate | COMPLETE — `run_scenario` now raises on rationing; conservation is not survival | `post-roadmap-rationing-gate.md` |
| Bucket 3 (C): cite the no-oracle params | CLOSED after 7 rounds — 8 cited, 14 design, 7 unciteable; blocked on retrieval, not effort, with the residual risk documented | `post-roadmap-citation.md` |
| The O₂ regulator's reversal | DIAGNOSED + CORRECTED 2026-08-11 — the clamp was already refused; the reversal is **not** author-only — it fires in 3 frozen runs the goldens are blind to. A sentence true when written, falsified by a seam three phases later | `post-roadmap-o2-makeup-reversal.md` |
| The direction gate (`ReversedFlowError`) | **BUILT 2026-08-11** — a third run-time verdict, `RationedError`'s sibling not its variant; authoring platform unfrozen (`flow_types` gains `demand_controlled`), both ports | `post-roadmap-o2-makeup-reversal.md` |
| Tooling: the PDF-backed citation pins (not science) | FIXED 2026-08-11 — a poppler upgrade re-wrapped quoted phrases; `sources/` is gitignored, so **CI green means nothing was checked** there | `docs/post-roadmap-log.md` (last row) |
| **Potato — the first SECOND species** | Stage 1 COMPLETE 2026-08-11 (stage 2, the Rust habitat mirror, deferred) — a crop is now a param SET; PCSE's bundled demo DB ships 6 offline oracles, so "new species = authored-only" was a stale reading. Two sources disagree qualitatively about when a tuber starts filling. **Its "one cause, two symptoms" canopy attribution was CORRECTED 2026-08-11 (measured at 39 %, not all)** | `post-roadmap-potato-crop.md` |
| The winter-wheat partition backfill | TAKEN and **REFUSED 2026-08-11** — the cited table misses the peak-LAI band 2.36×; cause isolated to root share over DVS 0–0.33. Neither table dominates the oracle: the frozen one passes because it was **fitted**. Successor named as root functional coupling — **that successor was TAKEN and its nitrogen half REFUSED the same day; see the next-but-one row** | `post-roadmap-wheat-partition-backfill.md` |
| Stem reserves: the model FORM found | LEAD 2026-08-11, **NOT STARTED** — [E] p. 93 §3.2.4 + Table 7 + Listings 3/4 carry the form the earlier refusal lacked; a partition table provably cannot substitute for it | `post-roadmap-wheat-partition-backfill.md` |
| Root functional coupling (the wheat refusal's successor) | **REFUSED on the measurement, then BUILT on the user's call 2026-08-11** — [E] p. 136 decouples rooted depth from root mass **on purpose**, so root CARBON stays refused *by citation*. The depth gate is bit-identically inert (uptake is demand-bound everywhere), was recorded NOT BUILT, and the user overruled: shipped as `aux_set` 2→3 + `root_depth.yaml`, both ports, **no value moved**. Successor is soil layers, for water | `post-roadmap-root-functional-coupling.md` |
| Soil layers — the water side of root depth | **BUILT 2026-08-11** — the price ("the largest single piece considered") assumed N layers; [F] endorses **two stores**, so the design was refuted by one more sentence of a source already on the shelf. `flow_set` 20→21; the golden diff was **predicted before regeneration and held** — no C/N/O amount moved anywhere. Depth now buys a 2.5× canopy where the topsoil runs out | `post-roadmap-soil-layers.md` |
| Tooling: the context budget (not science) | COMPLETE 2026-08-12 — the 2026-08-11 cut was a **relocation, not a discipline**: the index regrew 14.4→17.7 KB *the same day*, ~239 B per finished item. Retirement rule + one unconditional index + a paired ceiling test; `CLAUDE.md` 17,715→9,520 B, nothing condensed | `docs/context-budget.md` |
| Tooling: the record split (that rule set's own rule 4, built) | COMPLETE 2026-08-12 — the record table was 255,567 B in 32 physical *lines*, the longest 54,343 characters, which defeats `Grep`/`Read`/`git diff` alike. One file per item in `docs/log/`, **moved verbatim** — both sha-256 digests measured and recorded. The gate now caps line length, so a relocation cannot pass as the fix | `docs/context-budget.md` |

## The record

**One file per work item, in the same order as the index above.** Each file is that row's
Status cell, **moved verbatim** on 2026-08-12: the cell was one physical line, and the only
change is where it breaks — rejoining a file's body lines on single spaces reproduces the
cell character-for-character. Both digests are in `docs/context-budget.md` (rule 4), along
with why the table could not stay: 255,567 bytes in 32 physical lines defeats `Grep` (a
54,343-character row comes back as "one match"), `Read` (no way to page into a row) and
`git diff` (a one-word edit rewrites the whole line).

| Work | Record |
|---|---|
| **Development posture — the Rust-primary pivot** | [the record](log/rust-primary-pivot.md) |
| **Scope (B): the day-neutral habitat crop, validated vs LINTUL3 spring wheat** | [the record](log/day-neutral-crop.md) |
| **Scope (B): decomposer calibration — the carbon cluster moved above-range → top-of-range** | [the record](log/decomposer-calibration.md) |
| **The nitrogen-cycle FORM gap** (the decomposer calibration's named successor) | [the record](log/nitrogen-cycle-form.md) |
| **The canopy regulator** (the (C) diagnosis's named successor) | [the record](log/canopy-regulator.md) |
| **The chamber-scale diagnosis** (the upstream blocker with THREE witnesses) | [the record](log/chamber-scale.md) |
| **The acceptance gate** (the chamber-scale diagnosis's own conclusion, measured) | [the record](log/acceptance-gate.md) |
| **The science assertions get contract standing** (the acceptance gate's own finding 6, adjudicated) | [the record](log/acceptance-gate-standing.md) |
| **The humification split (a CUE)** — the soil-fractionation seam's named successor, **taken** | [the record](log/cue-humification.md) |
| **Soil carbon pool fractionation** (the chamber-scale diagnosis's named seam, taken — then RE-OPENED after the CUE, and REFUSED AGAIN ON A NEW LEG) | [the record](log/soil-fractionation.md) |
| **The crew-coupled loop** (the chamber-scale seam's standing alternative, taken) | [the record](log/crew-coupled-loop.md) |
| **The decade CO₂ guard, re-anchored** (the stem-only verdict's blocking contract question, answered by measurement rather than by choosing) | [the record](log/co2-guard-reanchor.md) |
| **Stem-reserve remobilization** (the user's own question: does the stem feed the seed?) | [the record](log/stem-reserves.md) |
| **Test-suite runtime** (tooling, not science) | [the record](log/test-suite-runtime.md) |
| The first authored habitat (`scenarios/algae_habitat.yaml`) | [the record](log/authored-habitat.md) |
| Tier 1: grow the flow registry (9 Power/Thermal/ECLSS flows + 3 loaders) | [the record](log/flow-registry-growth.md) |
| Tier 2: the grammar — `monod` | [the record](log/grammar-monod.md) |
| Bucket 3 scope (A): validation — diagnose + pin the oracle gap | [the record](log/validation-oracle-gap.md) |
| Bucket 3 scope (B): the full oracle match | [the record](log/oracle-match.md) |
| Bucket 2 (cont.): the export-fidelity hazard — **the one rationing cannot see** | [the record](log/export-fidelity.md) |
| Bucket 2 (cont.): **multi-rate authoring** — the chosen fix | [the record](log/multirate-authoring.md) |
| Bucket 2: the rationing gate — make the `dt` hazard loud | [the record](log/rationing-gate.md) |
| Bucket 3 scope (C): cite the no-oracle params | [the record](log/citation.md) |
| **The O₂ regulator's reversal is inside the freeze, not outside it** | [the record](log/o2-makeup-reversal.md) |
| **The direction gate — `ReversedFlowError`** | [the record](log/direction-gate.md) |
| **Tooling: the PDF-backed citation pins are green-by-skip on CI** | [the record](log/pdf-pins-ci.md) |
| **The second authored habitat — `scenarios/bioregenerative_station.yaml`** (the first habitat's finding #1, discharged) | [the record](log/bioregenerative-station.md) |
| **Potato — the first SECOND species** (stage 1 of 2) | [the record](log/potato-crop.md) |
| **The winter-wheat partition backfill** | [the record](log/wheat-partition-backfill.md) |
| **Root functional coupling** (the wheat refusal's named successor) | [the record](log/root-functional-coupling.md) |
| **Soil layers — the water side of root depth** | [the record](log/soil-layers.md) |
| **Tooling: the context budget** (the 2026-08-11 CLAUDE.md cut, re-opened because it did not hold) | [the record](log/context-budget.md) |
| **Tooling: the record split** (that rule set's own rule 4, taken as its own piece of work) | [the record](log/record-split.md) |
