## **The chamber CO₂ compensation-point band** (the guard that would have caught the step defect on day one)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-14** — the step unfreeze's own named successor, taken one ceremony later.
`docs/plans/post-roadmap-co2-compensation-band.md`; probes
`M:/claud_projects/temp/co2-band/`. `science_bands` goes **10 → 15 entries**, one over each
of the five chamber scenarios, asserting *season-low chamber CO₂ > `Γ*/ci_ratio`*. **A
schema-free unfreeze: `git diff src/` empty, no golden regenerated, no parameter moved, no
Rust or Godot change** — the whole ceremony is five manifest entries, one test file and the
freeze doc's prose half. The step ceremony had deliberately refused to bundle it
(*"a band written in the same change that makes it pass is a restatement of the run, not a
contract on it"*), which is why it is a separate item at all. **FINDING 1 — ⚠ THE PROPOSAL
NAMED THE WRONG SCENARIO, AND THE CORRECTION ALREADY EXISTED IN THE SAME FILE.** Both the
direction plan and `biosphere-reference.md`'s successor sentence proposed *"the **sealed**
chamber's season-low CO₂"* — the one scenario that **never crossed** (76.82 ppm at the
shipped step in its own no-re-sow configuration; the 57.89 ppm that rode its name came from
driving it through `run_perennial`'s unconditional re-sow, which its reference does not
perform). This was not a fresh discovery: `step.py` had corrected the pairing days earlier
during the step unfreeze, and the freeze doc carries that correction in bold as *"the
sharpest thing on this page"* — while the proposal three sections away in the **same file**
still carried the pre-correction wording. ⇒ **A correction lands where it is written, not
everywhere the claim was repeated**, and a guard inherits the locus of the diagnosis that
motivated it. The band is therefore written over the whole sealed roster, which is also the
form that does not need the diagnosis to have been right. **FINDING 2 — ⚠⚠ THE TIGHTEST OF
THE FIVE WAS NEVER MEASURED BY ANY OF THE WORK THAT ARGUED ABOUT IT.** Measured at the
shipped step, each scenario through its own golden's driver: `sealed_chamber` (`run_season`,
3 yr) **76.8196 ppm / 1.2579×**; `perennial_chamber` (5 yr) **75.4757 / 1.2359×**;
**`consumer_chamber` (5 yr) 74.4210 / 1.2186×**; the two 15-yr long horizons reproduce their
5-yr counterparts exactly. Every write-up of this defect — the enrichment record, the
controller probe, the step sweep, the step unfreeze, the freeze doc — quoted **sealed**
(75.75 / 76.82) or **perennial** (56.03 / 75.48). The consumer chamber sits **below both**,
and **no record in the tree carried its number** until the band enumerated the roster. ⇒ the
probes that drove the step decision swept the scenarios the **argument** was about; a band is
about the **roster**, and those are different lists. Same family as
`coverage-roster-is-not-the-manifest`, arriving from the opposite direction — there the
roster was over-read, here under-read. ⚠ It is a finding about **how the subject list was
chosen**, not a near-miss: 1.22× is the same order as the other four. **FINDING 3 — THE
HORIZON WAS CHECKED RATHER THAN ASSUMED, BECAUSE IT COULD HAVE BEEN FALSE.** The humification
split put the chamber settling transient at ~35 yr — **past every frozen horizon** — so
"green on the golden" is not automatically "green at equilibrium", and a band whose subject
settles outside its own horizon would be a contract the tree violates off the end of its own
reference run. Run to 50 yr, both re-sowing chambers take their **global** minimum *inside*
the frozen horizon (perennial **year 2**, consumer **year 5**) and rise monotonically to
75.84 / 75.06. ⇒ **the band's worst case is what the golden already runs**, so horizon and
contract coincide rather than being asserted to. **FINDING 4 — `Γ*`'s CITATION ATTEMPT CAME
BACK NEGATIVE, AND THE GAP IS DISCHARGED AS A MEASURED RISK RATHER THAN A DEBT.** `gamma_star
= 42.75` is `TODO(cite)` and the direction plan forbids the literal entering a science claim
while it is. One targeted retrieval was made, as specified: **the shelf tabulates no 42.75**
(the triple 42.75 / 404.9 / 278.4 is recognisable as one published parameterization, but a
recognisable number is not a citation and the locus must have been opened). What the shelf
*does* carry is an independent **route** to the same quantity — Teh eq. 6.19, `Γ* = O₂/(2·τ)`
with `τ = 2600 µmol/µmol` at 25 °C (Table 6.2) — giving `Γ* = 40.385` and a floor of **57.69
ppm, BELOW the shipped 61.07**. ⇒ **the shipped floor is the conservative one**: the citation
gap *understates* every margin, and closing it can only move the verdict further from the
floor. Asserted as a test, not as a sentence. ⚠ Explicitly **not** a licence to swap the
value — Teh's companion constants (`Kc` 300, `Ko` 300 mmol/mol) disagree with ours (404.9 /
278.4), so the two are different parameterizations and mixing them is the co-adaptation this
project has refused three times; the comparison is legitimate *only* because it moves the
bound in the harder direction. **FINDING 5 — THE BOUND IS DERIVED AT RUN TIME AND THE LITERAL
IS A TRIPWIRE.** `floor_ppm()` computes `Γ*/ci_ratio` from the frozen params; `61.07` appears
once, as a pin **on the params**, so re-valuing `Γ*` goes red loudly instead of silently
moving five contract entries at once — which is correct, because moving it *is* an unfreeze.
⚠ It also satisfies the manifest gate's own rule that every numeric literal in a `bound`
string appear textually at its locus: the gate wanted the number in the file, and the
tripwire is the honest way to have it there. ⚠ **The band is one-sided (`>`) on purpose** so
the next mechanism's golden movement cannot force a re-pin — but a one-sided claim degrades
silently (a change halving every margin leaves all five green), so the five margins are
pinned **separately and loosely** (2 %) as the *observable*, and that is the number the next
unfreeze's gate report quotes. **FINDING 6 — FIVE SEPARATE GATES, NOT ONE PARAMETRIZED ONE.**
`tests/science_gates.py` builds the manifest by static `ast` parsing and the marker must be a
literal decorator with literal keyword arguments, so a parametrized indirection would look
like five gates and freeze **nothing**. The repetition is the mechanism working, not a missed
refactor. **PREDICTED DIFF HELD** (`soil-layers-built`'s rule, applied before regenerating):
*the manifest's `science_bands` gains exactly 5 entries and nothing else moves* — 40
insertions, 5 deletions, all of them the five `[]` becoming arrays; no hash, no scenario, no
`liveness_floors`. **GATE REPORT**: all five bands pass; floor tripwire 61.0714 ppm; Teh route
57.6923 (0.9447×) passes; the five margin pins hold; the inherited pre-golden closure gate
(`rationed == 0`, `events == ()`) fires on every band run, because a rationed run's CO₂ trace
is not the model's answer; manifest completeness 15 gates in tree = 15 in manifests (it went
**red at 10 vs 15** before regeneration, which is the gate working); suite **2342 passed, 5
skipped, 7 m 32 s** at `-n 12` incl. `-m slow`; `ruff` / `pyright` / `cargo test` / `cargo
clippy -D warnings` all clean. **WHAT IT DOES NOT CLOSE**: `Γ*`'s citation (still open, risk
measured harmless, stays on the direction plan's list); the **station** manifest's
chamber-bearing scenarios — the biosphere is *delegated* there, so whether a station-side
chamber can reach the floor by a route the biosphere goldens do not exercise was **not
asked**; and any band on the open-field scenarios, which correctly have none because the
compensation-point criterion **only exists where the chamber is sealed**.
