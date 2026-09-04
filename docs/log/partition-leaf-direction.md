## **The leaf direction, exhausted** (the partition ladder's own unexhausted lead, taken — and the first ladder had been stopped by the knot with the LEAST leverage)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> No plan doc: filed under the September direction plan. Predecessor:
> [`partition-sensitivity.md`](partition-sensitivity.md), whose FINDING 4 named this lead.

**MEASURED 2026-09-04.** A measurement, not a build: no param, golden, manifest key or band
moved, and no decision was taken. `allocation.yaml`'s `TODO(cite)` is untouched — a ladder
around a provisional table prices **sensitivity**, never correctness. A prediction was written
down before the first run and is scored below; **two of its clauses are falsified**.

---

**FINDING 1 — the every-knot ladder's ceiling was set by the knot with the LEAST leverage, so
"the leaf direction is unexhausted" was an arithmetic artifact and not a physiological
statement.** The predecessor stopped at `fl×1.8` because `fl×2.0` drives the `dvs 0` share
(0.55) above 1. But a uniform scale is capped at the `min` over rows of `1/fl`, which is
**1.818 at emergence** against **3.333 at anthesis** — so the uniform ladder could never spend
the anthesis knot's headroom, however far it was run. *A perturbation's ceiling is a fact about
the parameterization; reading it as a fact about the response is how a lever gets called
exhausted while it is still climbing.*

⚠ **Why `dvs 1` is the knot with the leverage, stated carefully.** Two reasons, and the order
matters: it has **1.83× more headroom**, and — because the table is interpolated — raising it
lifts the leaf share across the **whole** `0 < dvs < 2` interior, not one development stage.
`open_season`'s peak LAI falls at **DVS 1.306192** (day 262.750 of 305; anthesis is crossed at
day 250.250), past anthesis, where the interpolated share is governed by the `dvs 1.0` and
`dvs 2.0` knots — but that is **corroborating, not causal**, and this record's first draft had
it as the mechanism. Peak *leaf carbon* is accumulated over the season, and 250 of its 305 days
are pre-anthesis; the ladders say so directly, since `fl@dvs0` does nothing at all at DVS 1.306
and still buys +9.9 %. *The instant an observable peaks is not the window that built it.*

⚠ The DVS-of-peak figure was **measured, not carried over**: the plan quotes ~1.37, from the
superseded 4.71/5.38 anchor. The scratch example that produced 1.306192 is saved beside the
ladders as `dvs-of-peak.rs` / `.txt`; it is not in the tree, because a one-shot orientation
that reproduces the frozen 6.022837 exactly does not earn permanent surface.

**FINDING 2 — with the knot as an axis the direction IS exhaustible, and the first gate to
break is the peak-LAI ceiling, at a +211 % perturbation of one number.** `fl` scaled at the
`dvs 1.0` knot only, the other three shares at that knot compensated proportionally, every
other row left frozen, shipped step, `--long`
(`M:/claud_projects/temp/partition-sensitivity/fl-at-dvs1-ladder.txt`):

| column | peak LAI (band 5–8) | peak W (cap 14.4248) | sealed CO₂ (floor 61.07) | peak-leaf (floor 0.55) |
|---|---|---|---|---|
| **frozen** | **6.022837** | **13.379084** | **71.435803** | **0.578137** |
| `fl@dvs1×1.25` | 6.136526 (+1.89 %) | 13.266993 | 71.000949 | 0.601992 |
| `fl@dvs1×1.5` | 6.241015 (+3.62 %) | 13.019084 | 70.604352 | 0.624904 |
| `fl@dvs1×2` | 6.401863 (+6.29 %) | 12.359371 | 69.911319 | 0.668830 |
| `fl@dvs1×2.5` | 6.871124 (+14.09 %) | 11.527707 | 69.331527 | 0.712420 |
| `fl@dvs1×3` | 7.740112 (+28.51 %) | 10.659096 | 68.845115 | 0.753156 |
| `fl@dvs1×3.3` | 8.435905 (+40.07 %) ⚠ **over the band** | 10.064585 | 68.591805 | 0.774100 |

Bracketed to 0.005 in the factor (`fl-at-dvs1-bracket.txt`, `-crossing-fine.txt`): **8.0 is
crossed between `×3.11` (7.995473) and `×3.115` (8.008653)** — the anthesis leaf share must
rise from **0.30 to 0.933–0.934** before the contract goes red. ⚠ That bracket is trustworthy
only because the rung's own noise was measured at ~0.001 in LAI (FINDING 4), which is smaller
than the 0.013 step between rungs; on the *emergence* rung, whose noise is 20× larger, a
bracket this tight would not have been reportable.

So the answer to *"how much leaf-share error can this `TODO(cite)` table absorb?"* is **a factor
of three at the knot the observable responds to**, and the frozen point is nowhere near an edge
in this direction.

⚠ The other three gates all move **away** from their bounds on this rung, two of them fast:
the above-ground biomass cap gains 24.8 % of clearance by `×3.3` and the perennial liveness
floor 33.9 %. The CO₂ floor falls, but spends only 2.84 ppm of its 10.36 ppm of headroom over
the whole rung. The LAI ceiling is the **only** candidate here, and it is not a close race.

**FINDING 3 — which gate breaks first depends on the KNOT, not only on the compensation
scheme, and the two leaf knots put a different gate nearest.** The predecessor's FINDING 5
established that *where the carbon comes from* decides which gate breaks. The knot axis
generalises it: at the **emergence** knot the near gate is the biomass cap, at the **anthesis**
knot it is the LAI ceiling — same organ, same direction (`fl-at-dvs0-ladder.txt`):

| column | peak LAI | peak W (cap 14.4248) | sealed CO₂ | perennial peak-leaf |
|---|---|---|---|---|
| **frozen** | **6.022837** | **13.379084** | **71.435803** | **0.578137** |
| `fl@dvs0×1.25` | 6.126344 (+1.72 %) | 13.857331 | 71.203167 | 0.638031 |
| `fl@dvs0×1.5` | 6.268256 (+4.08 %) | 14.039548 | 69.961082 | 0.669830 |
| `fl@dvs0×1.8` | 6.620340 (+9.92 %) | 14.119719 | 68.494138 | 0.747051 |
| `fl@dvs0×1.815` | 6.667933 (+10.71 %) | 14.122422 ⚠ **2.10 % under the cap** | 68.432363 | 0.749616 |
| `fl@dvs0×1.8181` | 6.649603 (+10.41 %) ⚠ **not monotone — see FINDING 4** | 14.120347 | 68.419742 | 0.750122 |

`fl@dvs0` **raises** peak W (+5.6 %) where `fl@dvs1` cuts it by a quarter, and the mechanism is
the predecessor's, unchanged: `peak W excl. fibrous roots` is an **above-ground** band, and at
`dvs 0` the compensated shares are stem 0.10 / root 0.35 — so raising leaf there takes mostly
from **root**, moving carbon into the band's own scope. At `dvs 1` they are stem 0.50 / root
0.20 (storage is 0 at that knot, so proportional compensation leaves grain untouched *there*),
so the same move takes mostly from **stem**, inside the band, and starves the later grain fill
as well. ⚠ **Neither gate actually breaks on the emergence rung**: it runs out of arithmetic
first, at 2.10 % of the biomass cap and 16.7 % below the LAI ceiling.

**FINDING 4 — the emergence rung is NOT smooth near its top, and the "turning point" this
record first claimed there is RETRACTED: it zigzags, and no mechanism is offered.** The first
pass saw `fl@dvs0` peak at `×1.815` (LAI 6.667933) and fall to 6.649603 at `×1.8181`, and
attributed it to early root collapse — root and stem at that knot are ~4e-5 and ~1e-5 there.
⚠ **A control at nine rungs refutes the attribution.** Between `×1.79` and `×1.8181` peak LAI
runs 6.601078, 6.624909, 6.620340, 6.654419, 6.644526, 6.656237, 6.667933, 6.643133, 6.649603
— up, down, up, down, up, up, down, up. Peak W does the same in both directions. A root
starvation that overtakes the leaf gain gives a **single-peaked** curve; this is small-amplitude
zigzag of about **0.3 %** riding on a monotone rise, over a 1.6 % change in the parameter. So
what is on record is: **the emergence rung is noisy at the 0.3 % level in its last ~1 %, cause
unexplained, and no claim is made from it.** *A causal claim earns the experiment that removes
the cause; this one was written from four points and did not survive nine.*

⚠ The anthesis rung is a different object at the same resolution and this is what makes the
comparison mean something: eleven rungs from `×3.09` to `×3.14` step by a near-constant
**0.013181** in LAI, with two single-point dips of ~**0.001** — a wobble **20× smaller** than
the emergence rung's, on the same observable and the same harness. `fl@dvs1` is monotone and
accelerating over its whole length (+1.89 / +3.62 / +6.29 / +14.09 / +28.51 / +40.07 %). ⚠ *The
direction is convex, so a linear reading of the low rungs understates the top by a wide margin*
— which is exactly how the prediction below got its factor wrong.

---

### The prediction, scored

Written before the first run
(`M:/claud_projects/temp/partition-sensitivity/prediction-per-knot.md`).

**1 — `fl@dvs1` is a stronger lever than the uniform ladder, dominating `fl×1.8` past
anthesis. CONFIRMED**: 8.435905 at `×3.3`, against a uniform ladder that could not reach past
7.369052.

**2a — peak LAI crosses 8.0 first; the CO₂ floor is never reached. CONFIRMED**: CO₂ stays
above 67.4 ppm on every rung, against a floor of 61.07.

**2b — the crossing falls between `×2.0` and `×2.5`.** ⚠ **FALSIFIED** — it is between `×3.11`
and `×3.115`, some 40 % further out. The response is convex and the low rungs understate it.

**3 — the response may turn over at the top as the root share collapses.** ⚠ **FALSIFIED,
twice over.** The anthesis rung is monotone and accelerating; and what looked like the
predicted turnover on the *emergence* rung survived four points and died at nine — it zigzags
rather than turning over, so there is no turning point on either rung and no root-collapse
mechanism on record (FINDING 4).

**4 — `fl@dvs0` alone buys strictly less than uniform at the same factor. CONFIRMED**:
6.620340 against 7.369052 at ×1.8.

**5 — `fl@dvs2` is refused by name rather than printed as an inert column. CONFIRMED**:
*"fl is 0 at the dvs [2.0] knot(s) … a structural null, not a measurement"*.

⚠ The prediction's most useful clause was the one it got **wrong**. 2b was reasoned from a
linear read of the uniform ladder's low rungs, and the direction is strongly convex; had the
ladder been stopped at `×2.5` on the strength of that estimate, it would have reported the band
as unreachable and the direction as saturating. *An estimate of WHERE a bound falls earns its
place precisely because being wrong about it is what tells you the shape.*

---

**Built to get the numbers** (`rust/crates/domains/src/lab/partition.rs`, 7 tests — 5 before,
2 added; `examples/partition_switch.rs`, the `organ@dvs=…` form). The perturbed rows are still
re-emitted into the frozen file's own text and loaded by the frozen loader, so the validation
is *the* validation. What the knot axis needed on top of that:

- **The ×1.0 control extended over the knot axis.** The uniform control cannot see a
  mis-resolved row — a wrong index still reproduces the table when every row is unchanged.
- ⚠⚠ **`a_per_knot_perturbation_moves_exactly_its_own_row` — the control the axis is worthless
  without.** A `Knot::At` that silently scaled every row would print plausible numbers, and
  they would be the *uniform* ladder's under a per-knot label. No caption can detect that;
  bit-identity on the untouched rows can.
- **A perturbation that leaves the table bit-identical is refused by name.** `fl` is 0.00 at
  `dvs 2.0`, so `fl@dvs2×1.5` returns the frozen table — a column of `<- UNCHANGED` on all
  eight readouts, which reads as *"the harness is broken"* or, worse, as *"the table does not
  matter here"*. Both are wrong; the share is simply not there to scale. `×1.0` is exempt,
  because being bit-identical is the control's whole job.
- **A non-knot value is refused, not snapped.** `fl@1.5` errors and names the knots.
- **`render_header` takes the knot rather than appending to a fixed "at every DVS knot"
  string**, and `label_of` puts the knot in every column heading — so a per-knot column cannot
  be quoted later as a rung of the every-knot ladder.

**Liveness measured, not assumed** (`--no-fail-fast`, so a truncated run cannot under-report):
ignoring the knot target reddens **3** tests; dropping the inertness guard reddens exactly
`a_perturbation_that_cannot_move_the_table_is_refused_by_name`; snapping a non-knot to row 0
reddens exactly `impossible_requests_are_refused`. The tree restores clean at 7/7.

**What this does NOT say.** Shipped step only, no converged column. Proportional compensation
is still **one** scheme. The two ladders here are **different experiments from each other and
from the every-knot one** — three perturbations, not one surface; nothing here licenses reading
a slope across them. And the `dvs 2.0` knot is unmeasurable for leaf **by construction**, so
"the leaf direction" means the two knots where leaf exists.

---

### The re-read the gate forced, done against a RUN

Landing this record appends a row, so `the_direction_plan_was_re_read_against_the_latest_record`
goes red until the September plan is re-read and its marker moved. The predecessor ended
*"check the plan's numbers against a **run**, not against the record they cite"*, so this one
was (`reread-extinction-coef.txt`), on §2.1 item 2's priced `extinction_coef` decision:

- **The shipped numbers hold.** *"+8.3 % on `k` buys +0.8 % of peak LAI shipped"* → measured
  **6.069990, +0.783 %** at `k = 0.65`. *"spends the perennial liveness floor down to 0.40 %"*
  → measured **0.552202** against the 0.55 floor, **+0.40 %**. Both confirmed to the digit the
  plan quotes. ⚠ The converged half (+7.3 %) was **not** re-checked — it needs the step sweep,
  not this harness, and saying so beats implying a full pass.
- ⚠⚠ **But the run carries a harder ground for refusing 0.68 than the plan records.** At
  `k = 0.68` the perennial fixed point is **0.538913 — below the 0.55 liveness floor**. The
  plan refuses 0.68 as *"unpublished"*, a provenance argument; it is also **red on a gate**.
- ⚠ And 0.68 is **non-monotone on the observable it was proposed to move**: peak LAI 6.058617
  (+0.594 %), *lower* than 0.65's +0.783 %. Neither fact appears in the record the plan cites.

*Two of the plan's numbers survived a run and a third option it had ranked on provenance turned
out to break a gate. The re-read is worth doing on the numbers that pass, too — that is how you
learn which of them were checked.*

---

**Successor.** The leaf direction is **exhausted**: the table absorbs a threefold error at the
knot that matters before any gate breaks, so `allocation.yaml`'s provisional numbers are not a
suspect in any bound the contract currently records. What that leaves is the predecessor's
successor, unchanged and now better founded — §2.1 is a **provenance** item, and §4's ranking
is still computed against the struck 5.03.
