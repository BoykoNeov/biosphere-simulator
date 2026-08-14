# The chamber CO₂ compensation-point band — the guard that would have caught it on day one

**BUILT 2026-08-14.** `science_bands` gains five entries; `git diff src/` empty; no golden
regenerated, no parameter moved, no Rust change. Gate report at the foot.

## Charge

The direction plan (§3 item 2, §4) filed this behind the step decision:

> A science band for the chamber's minimum CO₂. `science_bands` in both manifests already
> give assertions contract standing; *"the sealed chamber's season-low CO₂ stays above the
> compensation point"* is exactly that shape and would have caught this on day one. ⚠ It is
> **red on the frozen tree today**, so the *band* cannot land before the step decision.

The step decision was taken and shipped (`docs/log/step-unfreeze.md`, `dt = 1 → ¼`), which
turned the band from red to writable. The step ceremony deliberately did **not** bundle it —
`docs/biosphere-reference.md`: *"a band written in the same change that makes it pass is a
restatement of the run, not a contract on it."* This is that separate change.

---

## FINDING 1 — ⚠ THE PROPOSAL NAMED THE WRONG SCENARIO, AND IT WAS ALREADY KNOWN

Both the direction plan and the freeze doc's own successor sentence said **"the sealed
chamber's season-low CO₂"**. The sealed chamber is the one scenario that **never crossed**.

This was not a new discovery — `step.py`'s docstring had already corrected the pairing
during the step unfreeze, and `biosphere-reference.md` carries the correction in bold as
*"the sharpest thing on this page"*, including the exact consequence: *"a guard inherits the
locus of the diagnosis that motivated it; if the diagnosis names the wrong subject, the
guard is aimed at it too."* The recorded correction existed and the recorded proposal, three
sections earlier in the same file, still carried the pre-correction wording.

⇒ **A correction lands where it is written, not everywhere the claim was repeated.** The
band is written over the whole sealed roster, which is also the form that does not require
the diagnosis to have been right.

## FINDING 2 — ⚠⚠ THE TIGHTEST OF THE FIVE WAS NEVER MEASURED BY ANY WORK THAT ARGUED ABOUT IT

Measured at the shipped step, each scenario through **its own golden's driver**:

| scenario | driver | horizon | season-low CO₂ | margin |
|---|---|---|---|---|
| `sealed_chamber` | `run_season` (no re-sow) | 3 yr | 76.8196 ppm | 1.2579× |
| `perennial_chamber` | `run_perennial` | 5 yr | 75.4757 | 1.2359× |
| **`consumer_chamber`** | `run_perennial` | 5 yr | **74.4210** | **1.2186×** |
| `perennial_long_horizon` | `run_perennial` | 15 yr | 75.4757 | 1.2359× |
| `consumer_long_horizon` | `run_perennial` | 15 yr | 74.4210 | 1.2186× |

Every write-up of this defect — the enrichment record, the controller probe, the step sweep,
the step unfreeze, the freeze doc — quoted the **sealed** chamber (75.75 / 76.82) or the
**perennial** one (56.03 / 75.48). The consumer chamber sits **below both**, and no record in
the tree carried its number until this band enumerated the roster.

⇒ The probes that drove the step decision swept the scenarios the **argument** was about. A
band is about the **roster**. Those are different lists, and the second is the one a contract
is written over. Same family as `coverage-roster-is-not-the-manifest` (7 frozen vs 25 on
disk), arriving from the opposite direction: there the roster was over-read, here it was
under-read.

⚠ Note what this does *not* say: the consumer chamber is not in danger (1.22× is the same
order as the other four, and it was never the crossing). The finding is about **how the
subject list was chosen**, not about a near-miss.

## FINDING 3 — the horizon was checked, because it could have been false

The humification split pushed the chamber settling transient from ~3 yr to ~35
(`docs/log/cue-humification.md`) — **past every frozen horizon**. So "green on the golden" is
not automatically "green at equilibrium", and a band whose subject settles outside its own
horizon would be a contract the tree violates off the end of its own reference run.

Run to 50 yr, both re-sowing chambers take their **global** minimum *inside* the frozen
horizon and rise monotonically thereafter:

| | argmin | yr 1–5 | attractor (yr 45–50) |
|---|---|---|---|
| perennial | **year 2** | 76.26 75.48 75.54 75.58 75.67 | 75.84 |
| consumer | **year 5** | 76.04 74.56 74.49 74.44 74.42 | 75.06 |

⇒ **The band's worst case is what the golden already runs**, so horizon and contract coincide
rather than being asserted to. Had the minima fallen the other way, the honest move would
have been to state the band at the attractor and say the golden does not reach it.

## FINDING 4 — the uncited constant is discharged as a RISK, not as a debt

`gamma_star = 42.75 µmol/mol` is one of `photosynthesis.yaml`'s `TODO(cite)` entries, and the
direction plan is explicit that *"the number 61.07 ppm should not appear in a science claim
until it is sourced."* One targeted retrieval attempt was made, as specified.

**Retrieval: negative.** The shelf carries no source tabulating 42.75. (The triple
42.75 / 404.9 / 278.4 is recognisable as a single published parameterization, but the project
rule is that a locus **must have been opened** — a recognisable number is not a citation, and
writing one down from memory is exactly the failure `bucket3-scope-c-citation` records.)

**What the shelf does carry is an independent route to the same quantity.** Teh eq. 6.19,
`Γ* = O₂/(2·τ)`, with the specificity factor `τ = 2600 µmol/µmol` tabulated at 25 °C
(Table 6.2). On our own `o2` param that is `Γ* = 40.385` and a floor of **57.69 ppm — below
the shipped 61.07**.

⇒ **The shipped floor is the conservative one.** The citation gap understates every margin
here, and closing it can only move the verdict further from the floor. That is a *measured*
reason the band need not wait for the retrieval, and it is asserted as a test rather than as
a sentence (`test_the_shipped_floor_is_the_conservative_one_against_the_cited_route`).

⚠ **Explicitly NOT a licence to swap the value.** Teh's companion constants (`Kc` 300,
`Ko` 300 mmol/mol) disagree with ours (404.9 / 278.4), so the two are different
parameterizations; mixing them is the co-adaptation this project has refused three times.
The comparison is legitimate *only* because it moves the bound in the harder direction.

## FINDING 5 — the bound is derived at run time, and the literal is a tripwire

Writing `61.07` as the threshold would have made five contract entries depend on a value that
can be silently re-valued. Instead `floor_ppm()` computes `Γ*/ci_ratio` from the frozen params
and the literal appears once, as a **pin on the params**
(`test_the_floor_is_where_the_frozen_params_put_it`) — so moving `Γ*` goes red loudly, which
is correct, because moving it *is* an unfreeze.

⚠ This is also what satisfies the manifest gate's own rule that every numeric literal in a
`bound` string must appear textually at its locus — the gate wanted the literal in the file,
and the tripwire is the honest way to have it there.

⚠ **The band is one-sided (`>`) on purpose.** It must survive the next mechanism's golden
movement without being re-pinned. But a one-sided claim degrades silently — a change that
halves every margin leaves all five green — so the five margins are pinned separately and
loosely (2 %) as the **observable**, not the contract. That is the number the next unfreeze's
gate report quotes.

## FINDING 6 — five separate gates, not one parametrized gate

`tests/science_gates.py` builds the manifest by **static `ast` parsing**: the marker must be a
literal decorator with literal keyword arguments. A parametrized indirection is invisible to
it — it would look like five gates and freeze **nothing**. Written out five times deliberately;
the repetition is the mechanism working, not a missed refactor.

---

## The unfreeze this is

| What moves | Detail |
|---|---|
| `science_bands` | 5 entries added (was 5 empty lists) |
| Everything else in the manifest | **unchanged** — no hash, no scenario, no `liveness_floors` |
| Goldens | **none regenerated** |
| `src/` | **untouched** (`git diff src/` empty) |
| Rust / Godot | **untouched** — the band is a Python-side contract over existing runs |
| Freeze doc prose | `docs/biosphere-reference.md` gains the landed-band section |

**Diff predicted before regenerating** (`soil-layers-built`'s rule): *the biosphere manifest's
`science_bands` gains exactly 5 entries and nothing else moves.* Held — 40 insertions,
5 deletions, all of them the five `[]` becoming arrays.

## Gate report (ceremony step 5 — every gate, pass or fail, in writing)

- **The band itself:** all five pass. Margins 1.2186×–1.2579× against the derived floor.
- **The floor tripwire:** `Γ*/ci_ratio = 61.0714 ppm`, as frozen.
- **Robustness:** Teh route 57.6923 ppm, below the shipped floor (0.9447×) — passes.
- **The margin pins:** all five within 2 % of the recorded values.
- **Pre-golden closure gate**, inherited: every band run asserts `rationed == 0` and
  `events == ()` before its minimum is read. A rationed run's CO₂ trace is not the model's
  answer, so the band would be measuring the wrong thing without it.
- **Manifest completeness:** 15 gates in the tree, 15 in the manifests (was 10 vs 15 — the
  gate went red on the new tests before regeneration, which is the gate working).
- **Suite:** 2342 passed, 5 skipped, 7 m 32 s at `-n 12`, including `-m slow`.
- **`ruff check` / `ruff format` / `pyright`:** clean (0 errors).
- **`cargo test` / `cargo clippy --all-targets -D warnings`:** clean.

## What this does NOT close

- `Γ*`'s citation. Still `TODO(cite)`; the retrieval attempt came back negative and the risk
  is measured harmless rather than removed. It stays on the direction plan's §4 list.
- The **station** manifest's chamber-bearing scenarios (`greenhouse`, `sealed_station`, …).
  The biosphere is **delegated** in the station contract, so its bands are not restated
  there — but whether a station-side chamber can drive *its* CO₂ below the floor by a route
  the biosphere goldens do not exercise is a question this work did not ask.
- Any band on the **open-field** scenarios. The compensation-point criterion only exists
  where the chamber is sealed (the step sweep scoped this precisely: 4 of 8 configurations),
  so `open_season`, `n_limited`, `drought`, `water_biting` and `day_neutral` correctly have
  no entry of this kind.
