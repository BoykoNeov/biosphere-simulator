## **The O₂ the jar cannot see** (the FvCB item's first form gap, measured — and the harness could not spell the column the answer rests on)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> Plan of record: `post-roadmap-o2-coupling.md`. Filed under the September direction plan
> (referred to by that name here, never by filename — see the log's exemption note).

**MEASURED 2026-09-02**, the day the FvCB item named the gap. A measurement, not a build: no
param, golden or manifest key moved, and no decision was taken. It answers the September
plan's item 2 — *"decides whether the jar's bands are sensitive to a form the tree does not
carry"* — **YES, and in the opposite direction to the one the gap's own wording implies.**

**FINDING 1 — the scenario built to show O₂ depletion is the one that cannot see it.**
`sealed_chamber` charges 2.0 mol O₂ into 1000 mol of air (**2.0 mmol/mol**) and its golden
ends at 0.033185896524892 mol (**0.033 mmol/mol**), while `rubisco_limited_rate` reads the
frozen `o2 = 210.0` throughout — 105× at charge, **6329× at the end**. ⚠ The other two sealed
chambers sit at 210.0 mmol/mol at charge and 210.2 at the end, so **the frozen constant is
exactly right for two of the three and wrong by three orders of magnitude for the third** —
and the third is the one whose docstring calls it "the O₂-poor sealed chamber" and whose
litter charge was re-sized in August to keep the pool bottoming below 5 % of its fill.

**FINDING 2 — the form moves two numbers in opposite directions, and a half-built form is
worse than the frozen constant.** O₂ enters FvCB twice: the Rubisco denominator
`Kc·(1 + O/Ko)`, which is the gap as written, and `Γ* = 0.5·O/(S_c/o)`, i.e. Γ* ∝ O₂.
⚠ That proportionality is a **derivation** from the standard FvCB relation, not a retrieved
number; it places a column, and moved nothing in the tree. On the jar's season-low CO₂
against its floor: frozen 71.435803 / 61.071429 = ratio **1.1697**; the oxygenation half
alone (`o2=2`) 66.924275 / 61.071429 = **1.0958** — headroom cut from 10.364 to 5.853 ppm,
a **43 % cut**; the whole form (`o2=2 + gamma_star=0.4071`) 7.183490 / 0.581571 = **12.35**,
and at the jar's end fraction 6.574840 / 0.009597 = **685**. So the half the gap names is the
only version that tightens the band, and the whole form loosens it ~10×. Also measured: the
denominator **saturates by ~2 mmol/mol** (`o2=2` and `o2=0.033` agree to six figures), so all
of the form's time-dependence lives in Γ* and none in the term the gap was written about.

**FINDING 3 — the open field is blind to the term, by ~15 % of Vcmax.** `o2` 210→2 and
`vcmax` +10 % both move `open_season` peak LAI by **+0.000 %**, while `jmax` +10 % moves it
+0.756 % and `quantum_yield` +10 % moves it +1.338 % — one family inert, the other not, so
the open field runs on the light-limited branch and the oxygenation term is **unreachable**
there rather than damped (the observable is live: Γ* moves it +13 %). ⚠ But `vcmax` 90 is
still +0.000 %, 80 is −0.69 %, 70 −6.20 %, 60 −18.80 %, 50 −42.08 % — **the crossover is
between 80 and 90.** ⚠ **Whether the plan's item 3 (the Arrhenius temperature form) moves
Vcmax at all is NOT established here** — it is understood to, but no Vcmax Arrhenius
parameters have been retrieved, which is the same class of unretrieved claim as Γ* ∝ O₂
above and is what the owed page check would settle. **Conditionally:** *if* it cuts Vcmax by
more than ~15 % at the cool end of a 5–25 °C season, then it does not merely "raise
assimilation at 15–25 °C" as the plan predicts — it **moves the tree across the branch
boundary** and makes this term reachable. Predict that before the build, do not discover it.
⚠ `vcmax=50` also puts `perennial_long_horizon`'s converged peak-leaf at 0.545058, **below
its 0.55 liveness floor** — the cool end has a floor to clear, not just a band.

**⚠ What these columns do NOT say.** The value switch substitutes globally, so the coupled
columns for `perennial_chamber` and `consumer_chamber` — which sit at 210.2 mmol/mol — are
**counterfactuals, not form consequences**, and so is the `perennial_long_horizon`
liveness-floor break at `gamma_star=0.4071` (0.578137 → 0.417240 against a `> 0.55` bound):
that scenario's Γ* does not move under the form. **Under the real form exactly one gated row
in the table can move**, `sealed_chamber / season-low chamber CO₂`.

**FINDING 4 — the build would retire the guard that motivated it.** `min > Γ*/ci_ratio` is
written against a **constant** floor, which is what lets it be one number computed once.
Under the form Γ* tracks a live stock, so the claim becomes `CO₂(t) > Γ*(t)/ci_ratio`
pointwise — a different assertion, not a re-tuned one, and not what the five frozen
`..._stays_above_the_compensation_point` gates say. And at ratios of 12 to 685 it would stop
discriminating. The "built, and inert on the chambers" shape again (the canopy regulator; the
parked leaf mechanism) — named as the build's price rather than discovered after it.

**FINDING 5 — the harness could not express the one column the conclusion rests on.**
`report::compare` always took `Vec<(String, Vec<Substitution>)>`, but `value_switch.rs`
parsed every spec into `vec![sub]`, so a coupled column could be **argued across two
single-parameter columns and never measured** — the difference between evidence and
arithmetic done by the reader. The grammar was lifted out of the `examples/` binary (where no
test could reach it) into `domains::lab::parse_variants`: `,` still sweeps, `+` joins targets
into one column, mixing the two in one spec is refused because `a=1,2+b=3` has two readings,
and `a=1+` is an error rather than degrading into a single-substitution column that would
*read* as the coupled measurement. Five tests; the mutation collapsing a `+` column into two
reddens two of them. ⚠ A tooling change inside a science batch, named rather than slipped in
— same shape as the FvCB item's FINDING 2, and for the same reason: the measurement could not
be made without it. It re-anchors nothing.

**Recommendation, the decision being the user's:** build both halves or neither; re-pose the
band as a pointwise assertion *first*, or the build lands with no gate that can see it;
predict the branch crossing before the temperature form; and note that only one scenario's
science is at stake.

**Gates run on the committed tree, counts read off the whole output, written after the run:**
`cargo test --workspace --no-fail-fast` from `rust/` — **1108 passed, 0 failed** across 64
result lines. ⚠ A first run read **1107 where 1108 was predicted**, and the gap was chased
rather than waved through: this box is Windows and the previous item's 1103 was Linux, and the
one test that does not exist here is
`regen.rs::a_last_bit_difference_off_platform_is_ulp_only_and_is_never_rewritten`,
`#[cfg(not(windows))]` **by construction** — it is the FvCB item's own control, and on the
generation platform the gate accepts nothing but byte-exact, so the classification it asserts
is unreachable here. So the Windows baseline is **1102**, and 1102 + the six parser tests this
batch adds = 1108; the total moved by exactly the tests added, which is the check the
2026-09-01 record asked for. `cargo clippy --all-targets -- -D warnings`
clean; `cargo run --release -q -p station --example regen_goldens` → **19 of 19 run; 0 would
change**, byte-exact on all 19 here where the Linux run read 11 `ulp-only` — the same
platform policy, and the difference that item predicted rather than a finding; `uv run
pytest` → **8 passed, 4 skipped** (⚠ the 2026-09-02 record's line reads *3 skipped*; four is
right and is what `CLAUDE.md` states — all four are the opt-in oracle carve-out. A dated
record, so it is noted here rather than edited there); `ruff check` and `ruff format --check`
clean; `pyright` **0 errors, 0 warnings, 0 informations**.
