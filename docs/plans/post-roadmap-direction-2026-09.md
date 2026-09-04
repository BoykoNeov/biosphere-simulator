# Post-roadmap direction, September 2026 — the open queue, re-read against the tree

**Written 2026-09-02 on `b70bacf` plus the FvCB provenance item, superseding
`post-roadmap-direction.md` (2026-08-13).** Every claim below was checked against the
tree the day it was written — a grep, a test name, a golden value — not carried over from
the predecessor. The predecessor is kept whole as the record of a plan and of every place it
went stale.

**Re-read against the record's last row:** `partition-sensitivity.md`

⚠ **That line is a gate, not a note.** `repo_gates` asserts it names the record table's
*last* row. Landing a new item appends a row, so this doc goes red until someone re-reads it
against that item — strikes what it discharged, adds what it named — and moves the marker.
The predecessor was found stale by four successive records, each of which ended *"the next
re-read is owed the next time an item is taken off the list"*; the gate makes that owed
re-read a red test instead of a promise. It cannot judge the re-read. Do not move the marker
without one.

**How to read this doc.** §1 is where the project stands. §2 is the *science* queue, one
item per subsection, each ending with what it needs (a build, a retrieval, a measurement, or
a decision). §3 is the *structural* queue — the repo, not the sim. §4 is the recommended
order, and it separates *work* from *decisions* because the record shows they were mixed:
three items sat for weeks as "the user's call" while being listed as if they were next.
Strike-through is kept when this doc is wrong, per the predecessor's rule.

---

## 1. Where the project stands (2026-09-02)

- Roadmap Phases 0–9 COMPLETE; **53 post-roadmap items** in `docs/post-roadmap-log.md`.
- **Rust is the reference** (2026-08-16); the Python checker is gone (S6, 2026-08-27).
  `git ls-files '*.py'` is thirteen files, all the PCSE oracle carve-out or its one path.
- Four freeze contracts hold: biosphere (Euler, `dt = ¼`, 15 param files, 7 scenarios),
  station (13 scenarios, biosphere delegated), native-port tolerance, authoring platform.
- `cargo test --workspace --no-fail-fast`: **1108 passed on Windows / 1109 on Linux** across
  64 result lines — the one‑test gap is `regen.rs`'s `#[cfg(not(windows))]` ulp‑only control,
  by construction; clippy
  clean; `regen_goldens` **19 of 19, 0 would change** (11 `ulp-only` on Linux — §3.1;
  **0 `ulp-only` on Windows**, which is the platform difference that item predicted and was
  confirmed on the author's box 2026-09-02, not a finding).
- The last four science items were: the step unfreeze (`dt = 1 → ¼`, the crossing fixed);
  the layered canopy; two provenance discharges (`carbon_fraction`, and now the FvCB
  constants). The last five *tooling* items were the reference flip, the value switch, the
  science switch (slices 0–4), the margin pin, and the regeneration tool's verdict.
- **The last TWO items were MEASUREMENTS**, and neither moved a number.
  `log/o2-coupling-measured.md` (2026-09-02) discharged §2.3.2, re-priced §2.1 item 4 and
  gave §2.1 a measured bound. `log/partition-sensitivity.md` (2026-09-04) discharged §2.1
  item 3 and §4 item 4 — and **struck §2.1's headline state and §2.1 item 1's premise**,
  both of which had been stale since 2026-08-15. Read those before planning from §4.
- **The science thread's shape since 2026-08-14:** every mechanism offered to the canopy
  problem (§2.1) has been measured and either refused on its sign or shipped and found
  inert on the chambers. What is left there is provenance and one form question, not a
  mechanism shortage.

---

## 2. The science queue

### 2.1 The canopy this tree cannot grow — THE BIGGEST OPEN ITEM, and it is a provenance item first

~~**State, verified:** `open_season`'s peak LAI band is `5.0 < peak < 8.0` (*"real wheat peaks
at ~5–8"*). At the shipped step the tree reads **5.03** and converged **4.42**
(`log/canopy-magnitude.md`; the layered canopy moved it 5.38 → 5.03 shipped). The band
passes only because the observable is still moving between `dt = ¼` and the limit.~~

⚠⚠ **STRUCK 2026-09-04 — this paragraph is the deviation the frozen contract RETIRED three
weeks before this doc was written** (`log/partition-sensitivity.md`, FINDING 2). The band is
`5.0 < peak < 8.0`; the tree reads **6.022837** at the shipped step, measured off the frozen
params by that item's own baseline column, and `docs/biosphere-reference.md` carries the step
sweep as a table — **6.0228 at `dt = ¼`, 5.4273 converged** — under its own verdict
*"**The deviation is retired: the band now clears at every step in the sweep, converged value
included.**"* So the struck last sentence is false in both halves: the band does not pass
narrowly, and it does not depend on the step. ⚠ 5.03 / 4.42 were never the tree's — they are
`log/canopy-magnitude.md`'s **probe** figures (5.0314 / 4.4169) for the layered canopy taken
*alone*, computed with the midpoint rule that record **retracted in the same paragraph**, for
a candidate it **refused as a fix**. This doc cites that record by name while its own second
paragraph promises every claim was *"checked against the tree … not carried over"*: it was
checked against a **record**, and that record says on its face that it built nothing.

**Verified state, 2026-09-04:** peak LAI **6.0228** shipped / **5.4273** converged, inside
`5.0 < peak < 8.0` at every step in the sweep; **0.38 % over** the *separate* `peak < 6.0`
check, which is satisfied by its restated *"…**or** the 5 %/day mutual-shading loss is
MODELLED"* clause (that loss is inert — 6.0228 either way). ⚠ **So this item is SMALLER than
the "THE BIGGEST OPEN ITEM" in its own title, and §4's ranking was computed against 5.03.**
What is left is the provenance the subtitle already names, plus item 4's form question — not
a canopy the tree cannot grow.

**Eliminated by measurement, so not re-proposed:** the canopy regulator (a loss; built,
inert on chambers); the parked leaf mechanism (below the frozen tree on both observables at
every step); the intra-canopy light path (the sign is backwards — `Ag` is concave in PAR);
re-tuning the bound (refused three times).

**What is actually open, in order of leverage:**

1. ~~**`specific_leaf_area` keyed to development — a CITATION question.** Measured
   ~~2026-08-15: keying the constant spans peak LAI **3.04 (−35 %) to 8.24 (+75 %)** depending~~
   ~~on whether the frozen 22.0 m²/kg ([B] Table 19, "425 kg/ha per unit LAI") is read as the~~
   ~~*young*- or the *mature*-leaf value — and `canopy.yaml` does not say which the table~~
   ~~reports. The late-anchored reading also moves the independently pinned "LAI peaks after~~
   ~~anthesis" defect the right way (DVS 1.37 → 0.96). **Needs: one retrieval** — what leaf~~
   ~~population Table 19's figure describes — then a form build if it is age-specific.~~
   ⚠ **PREMISE STRUCK 2026-09-04** (`log/partition-sensitivity.md`, FINDING 3). `canopy.yaml`
   reads **23.53**, not 22.0, with `source: "[B], Table 19 p.100, 'Wheat, winter' (425 kg/ha
   per unit LAI → 10000/425)"` — the `TODO(cite)` was retired and the value moved +7.0 % by
   the layered-canopy item, and the file's own header now says `extinction_coef` is *"the live
   provenance queue, **alone**"*. So this is **not a missing-citation retrieval**, and the
   −35 %/+75 % span was measured against the superseded 4.71 / 5.38 anchor.
   **What survives, restated:** the page names the crop but not the leaf **population**, so
   whether a *cited* constant should be development-keyed is still open. **Needs: one
   retrieval** — what leaf population Table 19's figure describes — then a form build if it is
   age-specific. Re-measure the span before quoting it.
2. **`extinction_coef` 0.60 / 0.65 / 0.68 — a DECISION, priced.** Three shelf readings
   disagree (`log/canopy-provenance.md`); +8.3 % on `k` buys +0.8 % of peak LAI shipped,
   +7.3 % converged, moves the LAI peak before anthesis, loosens all five CO₂ bands, and
   spends the perennial liveness floor down to 0.40 %. **Needs: the user's call**; the
   record recommends 0.60 (coherence with Goudriaan's quadrature, conservative on the
   gates) or 0.65 (crop-specific) and refuses 0.68 (unpublished).
3. ~~**The partition table is UNMEASURED as a cause.** It was fitted against the pre-light-path
   assimilation (`log/wheat-partition-backfill.md`), so it is a suspect in exactly the way
   the band is. The value switch can measure a uniform root-share perturbation in minutes.
   **Needs: a measurement**, no decision.~~ **MEASURED 2026-09-04 —
   `log/partition-sensitivity.md`. It IS a live lever, and strongly asymmetric.** Every organ
   share scaled at every DVS knot, others compensated proportionally: a **25 % rise** in
   either non-leaf share costs ~**27 %** of peak LAI (`fr×1.25` 4.4094, `fs×1.25` 4.3857
   against 6.0228), while a 25 % **fall** buys under **2 %** and saturates. ⚠ That saturation
   is on the non-leaf-reduction side **only**: raising the leaf share directly is still
   climbing at `fl×1.8` (**7.369**, +22.35 %), and the ladder stops there because `fl×2.0` is
   arithmetically impossible, not because the response flattened — **an unexhausted direction
   on this item's own observable**. ⚠ **The compensation scheme decides which
   gate breaks**, so it is a variable, not a nuisance: more carbon to leaf is clean spelled
   `fl×1.8` (LAI **7.369**, biomass *falls* to 12.96, liveness floor +34 %) and **breaks the
   above-ground biomass cap** spelled `fr×0.75` (**14.4621** against 14.4248) for a tenth of
   the LAI gain — because that cap excludes fibrous roots, so shrinking the root share raises
   it by arithmetic. ⚠ **And the harness this item was priced on cannot address the table:**
   `with_override` refuses a table-shaped field by construction and `+` joins **scalar**
   substitutions, so §4 item 4's ⚠ was false when it was written. `lab::partition` +
   `examples/partition_switch.rs` now spell it, by re-emitting the perturbed rows into the
   frozen file's own text so the loader's rules stay the only copy.
4. **The temperature form of the kinetics (§2.3)** — ~~raises assimilation at 15–25 °C
   relative to the cardinal multiplier if it does anything~~. **RE-PRICED 2026-09-02 and the
   old wording was too small.** Measured (`log/o2-coupling-measured.md`): `vcmax` +10 % moves
   `open_season` peak LAI by **+0.000 %** and `jmax` +10 % moves it +0.756 %, so the open
   field runs on the **light-limited branch** and the whole Rubisco branch is *unreachable*
   there. But the margin is thin — `vcmax` 90 is still +0.000 %, 80 is −0.69 %, 70 −6.20 %,
   60 −18.80 %, 50 −42.08 %, so **the crossover sits between 80 and 90, ~15 % below the
   shipped Vcmax**. ⚠ That the temperature form moves Vcmax at all is **understood, not
   retrieved** — no Vcmax Arrhenius parameters are on the shelf, and the owed page check is
   what would settle it. *If* it cuts Vcmax at the cool end of a 5–25 °C season, the form
   therefore does not "raise assimilation": it **switches which process is limiting**. That
   makes it the most likely mover of this item's observable *and* the least predictable —
   predict the crossing before building, not after. ⚠ `vcmax = 50` also drops
   `perennial_long_horizon`'s converged peak-leaf to 0.545058, **below its 0.55 floor**.

### 2.2 Provenance — what is still `TODO(cite)`, and what it would take

**State, verified 2026-09-02:** `grep -rl 'TODO(cite)' rust/crates/domains/params` —
**56 markers in the biosphere and sibling param files** (60 before the FvCB item), 4 in
`station`. Bucket 3(C) closed the no-oracle set as *blocked on retrieval, not effort*; do
not reopen it wholesale. Single params when a finding leans on one:

- **The Bernacchi page check — OWED, ten minutes with the PDF.** The FvCB item bound
  Kc/Ko/Γ* to Bernacchi et al. (2001) from the constants as the literature reproduces
  them; the paper itself was unreachable from the box. Put the PDF in `sources/`, read
  Table 1, confirm 404.9 / 278.4 / 42.75 at 25 °C, and strike the "owed" in the file
  header. If a digit differs: record it, do not move the number (a value move is a
  13-golden ceremony and its own decision).
  ⚠ **Re-checked 2026-09-02 from the author's box: the blocker is NOT the network.** Web
  access works here; Wiley returns 403 and no open copy of the paper is on the web. So the
  retrieval is not "try again from a better box" — it needs the PDF put into `sources/` by
  hand, or a logged-in fetch. **Asked of the user 2026-09-02; still owed.** ⚠ Do not
  discharge it with a secondary table (PhotoGEA, Sharkey 2007, an R package's constants):
  those are the *same* reproduced-by-the-literature evidence the file already cites, so
  fetching one adds nothing the header does not already say.
  ⚠ **It is now worth more than it was.** As of the o2-coupling item this retrieval also
  settles the *second* question §2.1 item 4 and §2.3.1 both now hang a conditional on —
  whether the paper gives **Vcmax** a temperature response, and with what parameters. That
  is the difference between item 3 being "raises assimilation a little" and "switches which
  process limits", so it is a prerequisite for pricing item 3 rather than a tidy-up.
- **The eight remaining `photosynthesis.yaml` markers.** `vcmax` 100 and `jmax` 180: a
  species survey exists (Wullschleger 1993, *J. Exp. Bot.* 44:907, tabulates wheat) and is
  the retrieval to make; ⚠ a wheat-specific value is a *value* question — the pair moves
  every golden and every margin (`vcmax` +5 % moves the five margins < 2 %, +30 % moves
  them 4.4 %, `log/co2-margin-pin.md` FINDING 3). `quantum_yield` 0.3 and `theta` 0.7:
  Bernacchi et al. (2003, *PCE* 26:1419) give the electron-transport response; θ is
  conventionally 0.7–0.9. The four cardinals are the [B] TMPFTB idiom, superseded in
  principle by §2.3. **Needs: retrieval; then decisions where a value would move.**
- **`nitrogen.yaml` `max_uptake_capacity`** — cited-against: 15 kg N/ha/day is *~6× the fastest
  period-average reported for high-yielding wheat* (the file says so). Measured inert
  because uptake is demand-bound everywhere (`log/root-functional-coupling.md`), which is
  why it has been safe to leave. **Needs: nothing until a mechanism makes it bind.**
- **`decomposition_rate`, `microbial_respiration_rate`** — open with findings; the
  fractionation that would retire the first was refused twice on the shelf. **Parked.**

### 2.3 Two form gaps from the FvCB file — the science switch's first scientific pair

The science-switch plan's slice 3b is *"a scientific pair"* and its own §2C measured that
**no alternative form of any biosphere process exists in the tree**. The FvCB item found
two, from the paper it cited:

1. **Temperature response of Kc, Ko, Γ*.** The tree scales the whole assimilation rate by
   the [B] cardinal multiplier over 25 °C constants. Bernacchi (2001) gives each constant
   an Arrhenius function (the paper's subject). A lab-only mechanism —
   `build_season_replacing` the FvCB aux with a temperature-resolved one — is exactly
   slice 3b's shape: no unfreeze, both forms cited, the harness prints the peak-LAI band,
   the five margins and the liveness floors side by side. **Needs: a build, no decision.**
   ⚠ It will not be inert: the open field runs 5–25 °C through a season and the chambers
   are held near 20 °C (check `weather.rs` and each scenario's forcing before predicting
   a sign).
   ⚠⚠ **RE-PRICED 2026-09-02 — bigger than "not inert".** Per §2.1 item 4, the open field
   sits only ~15 % of Vcmax above the light/Rubisco crossover. If this form's Vcmax response
   cuts more than that at the cool end, the build changes *which* limitation binds, and in
   the same move makes item 2's oxygenation term — measured as unreachable in the open field
   — reachable there. So the two form gaps are **not independent**, which is how they were
   listed. Predict the crossing, and use the `vcmax` ladder in
   `log/o2-coupling-measured.md` as the baseline it is read against.
2. ~~**`o2` is a constant; chamber O₂ is a stock.**~~ **MEASURED 2026-09-02 —
   `log/o2-coupling-measured.md`.** The question this asked (*"do the jar's science bands
   move?"*) is answered **yes**, and the answer inverts the item as written. Kept below with
   what it got right; what it got wrong is struck. Measured off the goldens 2026-09-02:

   | scenario | `biosphere.o2_pool` at the golden's end | vs its charge |
   |---|---|---|
   | `sealed_chamber` (the jar, charge 2.0 mol) | **0.033 mol** | **−98 %** |
   | `perennial_chamber` (charge 210 mol) | 210.24 mol | +0.1 % |
   | `consumer_chamber` (charge 420 mol) | 420.46 mol | +0.1 % |
   | both long horizons | as their 5-year siblings | — |

   In the jar the crop's Rubisco oxygenates against 210 mmol/mol of an O₂ it has already
   consumed; with the live pool, Γ* would fall toward zero and the compensation floor with
   it. In the two big chambers the coupling is inert at the 0.1 % level. ~~**Needs: a
   measurement** through the value switch first (`o2` at the jar's actual fraction), then a
   form build only if the jar's science bands move — they are the ones that would.~~

   **What the measurement found, and where this entry was wrong.** The jar's fraction is
   **2.0 mmol/mol at charge and 0.033 at the end** against a frozen 210 — 105× and 6329×,
   and the constant is *exactly right* (210.0/210.2) for the other two chambers. The two
   halves of the form move the band in **opposite** directions: the oxygenation denominator
   alone cuts the jar's headroom above the floor 10.364 → 5.853 ppm (**−43 %**), while the
   whole form (Γ* ∝ O₂, a **derivation**, not a retrieved number) takes the ratio 1.1697 →
   **12.35** at charge and **685** at the end. So **a half-built form is worse than the
   frozen constant**, and the entry's own phrasing — which names only the denominator —
   describes the dangerous half. The denominator also **saturates by ~2 mmol/mol**, so all
   the time-dependence lives in Γ*, none in the term this entry was about.

   ⚠ **And the build would retire its own guard:** `min > Γ*/ci_ratio` is written against a
   *constant* floor, so a live Γ* makes it a pointwise claim — a different assertion, not a
   re-tuned one — and at ratios of 12 to 685 it stops discriminating.

   **Needs: a DECISION, not a measurement** (moved to §4's decision list): build both halves
   or neither, and re-pose the band first. Only `sealed_chamber` can move under the form.

### 2.4 The chamber CO₂ controller — priced, a `dt = ¼` object, and it vents

State unchanged since `log/co2-controller.md`: a setpoint controller does not remove the
step defect, needs `~3000 ppm` to clear the margins on the plant-only chambers, and a
two-sided setpoint **vents 235 of 429 mol** — an odd thing for a closure sim. Now that the
step is `¼`, it is buildable; whether a habitat that vents carbon is the right realism move
is **the user's decision**, and this doc recommends *not yet*: §2.1 and §2.3 change the
numbers it would be tuned against.

### 2.5 The parked leaf mechanism — a decision, recommended REFUSE

`leaf-expansion-blocked` sits below the frozen tree on both gated observables at every step
(`log/leaf-remeasurement.md`). Its ship/refuse call has been the user's since 2026-08-14.
Recommendation: refuse, and retire the branch with a record; nothing in the queue is waiting
on it, and §2.1's inventory does not include it.

### 2.6 Potato stage 2 — the Rust habitat mirror; buildable, no decision

The params crossed with the flip (`params/biosphere/crops/potato/`, 4 files) and
`system.rs` records that the Rust roster has no potato build. It is the second species'
second half, authored-not-validated by the day-neutral precedent. **Needs: a build.**

---

## 3. The structural queue — the repository, not the simulation

### 3.1 DONE this item, listed so the next re-read can check they held

- **The regeneration tool byte-compared.** On Linux — CI's platform — it reported 11 of 19
  goldens `CHANGED` on the untouched tree. It now reaches the gate's verdict and reports
  `ulp-only`, never rewriting those. ⚠ Still true: the eleven transcendental goldens can
  only be regenerated on Windows/UCRT; an unfreeze that moves one of them from a Linux box
  has no regeneration step there. Record it in the ceremony, do not `--write` around it.
- **The direction plan rotted four times.** This doc carries the re-read gate (the marker under its title).
  ⚠ It fired for the first time on 2026-09-02, on the o2-coupling item, exactly as designed:
  the row landed, the gate reddened, and the re-read that cleared it is what struck §2.3.2
  and re-priced §2.1 item 4 and §2.3.1. The mechanism works; it is the re-read that has to
  be honest, and the gate still cannot tell.
  ⚠⚠ **It fired a SECOND time on 2026-09-04, and this time the re-read found the previous
  re-read's own damage** (`log/partition-sensitivity.md`): the 09-02 pass wrote a *new* false
  claim (§4 item 4's `+` ⚠) about a harness it had not run, and it left §2.1's headline
  numbers — a deviation the frozen contract had **retired** three weeks earlier — untouched,
  because it checked them against the record they cite rather than against a run. **The
  mechanism forces a re-read; it cannot force the re-read to RUN anything, and reading is how
  both misses happened.** The countermeasure adopted here, and it is cheap: a re-read of a
  numeric claim quotes a **measured baseline column**, never a record.
- **The value switch could not spell a coupled column.** `report::compare` always accepted a
  multi-substitution variant; `examples/value_switch.rs` parsed every spec into exactly one,
  so a form that moves two numbers together could be *argued* across two columns and never
  *measured*. The grammar moved into `domains::lab::parse_variants` (gated, five tests, and
  a collapsing mutation reddens two of them) and `+` joins targets into one column.
- **`README.md` described the Python-canonical project** and listed five deleted
  directories. Rewritten to the tree.
- **`ci.yml` said there was no Python job** two paragraphs below a Python job; the job ran
  `-m "not slow"` over a marker no test carried. Comment corrected, filter dropped, the two
  no-subject markers retired from `pyproject.toml` (their "one release" grace had no end,
  because this project has no releases).
- **`CLAUDE.md`** lost the `test-suite-runtime.md` warning to that doc's own header
  (10,906 B against a 12,000 B ceiling; it was 11,205).

### 3.2 OPEN

- **`drift_summary.json` is unregenerable by any path** (`biosphere-reference.md`, the
  unfreeze discipline, step 3). Slice C5 ported the fold; converting `emit_drift` to emit
  the summary directly is now a small job and the record says so. An unfreeze that moves
  that golden has no regeneration step until it is done. **Needs: a build.**
- **The memory index is outside the repo.** `repo_gates` reads
  `~/.claude/projects/M--claud-projects-space-station/memory/MEMORY.md` and says loudly
  when it is absent — so on CI and on any box but the author's the ceiling is unchecked,
  and the "memory file" the working style requires per item cannot be written from a
  remote session (this one did not). Not a defect to fix in the repo; a fact to know
  when reading a CI-green claim about the memory budget.
- **`canopy.yaml`'s header still says a provenance edit is invisible** (*"which the manifest
  records and no test can see"*), falsified by C7. Left deliberately — a comment edit is a
  manifest hash and an unfreeze — and to be corrected the next time that file's manifest
  entry moves for a real reason. Same for `self_discharge.yaml`'s stale pointer.
- **Two records say "Γ* is TODO(cite)" as of their date** (`co2-margin-pin.md`,
  `co2-compensation-band.md`). Dated records, not maintained — the rule; listed so nobody
  "fixes" them.

---

## 4. Recommended order

**Work (no decision needed):**

1. The Bernacchi page check (§2.2) — a retrieval, and it closes the FvCB item's one "owed".
   ⚠ **BLOCKED on a PDF, not on a box** (§2.2) — asked of the user 2026-09-02.
2. ~~The `o2` measurement through the value switch (§2.3.2)~~ — **DONE 2026-09-02,
   `log/o2-coupling-measured.md`.** It produced a decision, not a next task; see below.
3. The temperature-kinetics form as the science switch's first scientific pair (§2.3.1) —
   lab-only, no unfreeze, and it is the measurement §2.1 item 4 is waiting on.
   ⚠ **Now the highest-leverage item in this list, and the least predictable** — it is the
   one candidate measured to move `open_season`'s observable, and it does so by switching
   which process limits (§2.1 item 4). Predict the crossing before building.
4. ~~The partition-table sensitivity measurement (§2.1 item 3) — value switch, minutes.
   ⚠ Now cheaper than when written: a `+` column can perturb the whole partition table at
   once rather than one share at a time (§3.1).~~ **DONE 2026-09-04,
   `log/partition-sensitivity.md`** — and **BOTH struck sentences were false**. The value
   switch cannot address a table-shaped field in either of its forms; the ⚠ that said `+`
   could was **added by this doc's own 2026-09-02 re-read**, i.e. the re-read the gate had
   just forced wrote a *new* false cheapening claim about a harness it did not run. *A
   re-read checks the claims it is looking at; the claims it writes are checked by nothing.*
   The measurement produced a live result (§2.1 item 3) and a struck §2.1 premise.
5. The `specific_leaf_area` retrieval (§2.1 item 1) — the highest-leverage provenance
   question on the peak-LAI observable. ⚠ **RE-SCOPED 2026-09-04** — the constant is CITED (23.53, [B]
   Table 19 p.100); what is owed is the leaf **population** the page reports, not the
   citation, and the −35 %/+75 % span it was ranked on is against a superseded anchor.
6. Potato stage 2 (§2.6); `drift_summary` regeneration (§3.2). Either can go any time.

**Decisions (the user's; listed apart because listing them as work is how the predecessor
went stale):** `extinction_coef` (§2.1 item 2); **the live-O₂ FvCB form (§2.3.2, NEW
2026-09-02 — the measurement turned it from a task into a call)**; the parked leaf mechanism
(§2.5, recommended refuse); the CO₂ controller (§2.4, recommended not yet); the product track
(dormant by the 2026-08-13 decision; re-open when the science thread reaches a natural
stop — §2.1 resolving would be one).

**Not recommended:** reopening the citation bucket wholesale; any value move on the FvCB
constants before the page check; a `--write` of a transcendental golden from a Linux box;
**building the oxygenation half of §2.3.2 without the Γ* half** — measured to be the one
change that makes the jar's band *tighter*, and it is the half the gap's own wording names.

---

## 5. Rules for this doc

- It is a **plan**, not a record. A finished item earns the normal three (index line,
  pointer row, record file) and *leaves* this doc — struck in place with the record named,
  never silently deleted.
- **Records refer to it as "the September direction plan", never by filename** — the
  plan-doc parity gate reads the index section against the record files, and a filename
  in a record with no index row turns it red (`post-roadmap-log.md`, the exemption note).
- **The re-read marker moves only with a re-read.** The gate cannot tell the difference;
  the reader of the diff can.
- **Supersede, do not strike a fourth time.** When the struck spans outnumber the live
  ones, write the successor, move `DIRECTION_PLAN` in `repo_gates`, and banner this one.
