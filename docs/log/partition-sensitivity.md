## **The partition table, measured** (the direction plan's cheapest free item — and the plan's biggest open item was premised on a deviation the frozen contract had already retired)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> No plan doc: filed under the September direction plan (referred to by that name here, never
> by filename — see the log's exemption note).

**MEASURED 2026-09-04.** A measurement, not a build: no param, golden, manifest key or band
moved, and no decision was taken. The item is the September plan's §2.1 item 3 — *"the
partition table is UNMEASURED as a cause"* of the too-small canopy — priced in §4 item 4 as
*"value switch, minutes"*. The measurement was made and it answers the question asked
(**yes, the table is a live lever, and the direction of the answer depends on a scheme
nobody had named**). But two of the three things the plan said about this item were wrong
before the first run, and the third — the premise the whole of §2.1 rests on — was wrong by
a whole retired deviation.

**FINDING 1 — the harness the item was priced on cannot address the table, and the claim
that it could was written by the re-read that was supposed to catch this.** `with_override`
(`rust/crates/config/src/params.rs`) rejects a **table-shaped** field before it rewrites
anything; its own comment names `allocation.yaml`'s partition rows as the case it is
refusing, and `config::params::tests` pins the refusal. The `+` form of
`domains::lab::parse_variants` joins several **scalar** substitutions into one column, and
the partition table has no scalar entry for a substitution to name — so neither form of the
value switch reaches it. ⚠ The plan's *"value switch, minutes"* was inherited; but its ⚠
*"a `+` column can perturb the whole partition table at once rather than one share at a
time"* was **added on 2026-09-02 by the plan's own re-read**, the one the gate had just
forced. The gate fired, the re-read happened, and the re-read wrote a **new** false claim
that the harness was cheaper than it is. *A re-read checks the claims it is looking at; the
claims it writes are not checked by anything.*

**FINDING 2 — §2.1's "State, verified" is the deviation the frozen contract retired three
weeks earlier.** The plan says the tree reads **5.03** shipped and **4.42** converged, and
that *"the band passes only because the observable is still moving between `dt = ¼` and the
limit"*. `docs/biosphere-reference.md` carries the step sweep as a table and its own verdict:
after the layered canopy and the SLA anchor, peak LAI is **6.0228 at `dt = ¼`** and
**5.4273 converged**, and ⇒ *"**The deviation is retired: the band now clears at every step
in the sweep, converged value included.**"* This item's baseline column reads **6.022837**,
independently, off the frozen params. ⚠ And 5.03 / 4.42 are not merely stale: they are
`log/canopy-magnitude.md`'s **probe** numbers (5.0314 / 4.4169) for the layered canopy taken
**alone**, computed with the midpoint rule that record **retracted in the same paragraph**,
for a candidate it **refused as a fix**. Three disqualifications, and the plan cites that
record by name. ⚠⚠ The plan's own preamble promises *"Every claim below was checked against
the tree the day it was written — a grep, a test name, a golden value — not carried over"*.
It was checked against a **record**, not against the tree, and the record it chose says on
its face that it built nothing. *"Verified" is a claim about a method; the method here was
reading, and reading a diagnosis cannot see the build that superseded it.*

**FINDING 3 — §2.1 item 1's citation question was answered in the same build.** The plan
calls `specific_leaf_area` *"the frozen 22.0 m²/kg"* whose table `canopy.yaml` *"does not
say which the table reports"*. `canopy.yaml` reads **23.53** with
`source: "[B], Table 19 p.100, 'Wheat, winter' (425 kg/ha per unit LAI → 10000/425)"`, and
its header states that `extinction_coef` is *"the live provenance queue, **alone**"*. The
`TODO(cite)` was retired and the value moved +7.0 % by the layered-canopy item. ⚠ **What
survives is a real question and a different one**: the page names the crop but not the leaf
*population*, so whether the constant should be development-keyed is still open — but it is
a *form* question against a **cited** constant, not the missing-citation retrieval §4 item 5
schedules, and the ±35 %/+75 % span it quotes was measured against the superseded 4.71/5.38
anchor.

**FINDING 4 — the table is a live lever, and it is strongly asymmetric: raising a non-leaf
share is steep, lowering one saturates, and raising the LEAF share directly is still
climbing at the top of the ladder.** Every organ share scaled at every
DVS knot, the other three compensated proportionally, `--long`, shipped step
(`M:/claud_projects/temp/partition-sensitivity/`):

Columns: `open_season` peak LAI (band 5.0–8.0), `open_season` peak W excluding fibrous
roots (cap 14.4248 t/ha), `perennial_long_horizon` converged peak-leaf (floor 0.55).

| column | peak LAI | peak W | perennial peak-leaf |
|---|---|---|---|
| **frozen** | **6.022837** | **13.379084** | **0.578137** |
| `fl×0.75` | 2.049844 (−65.97 %) | 8.112368 | 0.464368 ⚠ **below floor** |
| `fl×1.25` | 6.170247 (+2.45 %) | 13.616199 | 0.655672 |
| `fl×1.5` | 6.602815 (+9.63 %) | 13.417304 | 0.697592 |
| `fl×1.8` | 7.369052 (+22.35 %) | 12.961913 | 0.772486 |
| `fr×0.75` | 6.136594 (+1.89 %) | 14.462100 ⚠ **over cap** | 0.621794 |
| `fr×0.5` | 6.154555 (+2.19 %) | 15.478502 ⚠ **over cap** | 0.648529 |
| `fr×1.25` | 4.409408 (−26.79 %) | 11.416315 | 0.542943 ⚠ **below floor** |
| `fs×0.5` | 6.253383 (+3.83 %) | 11.394819 | 0.636538 |
| `fs×1.25` | 4.385688 (−27.18 %) | 13.131098 | 0.545389 ⚠ **below floor** |

A **25 % rise** in either non-leaf share costs ~27 % of peak LAI; a 25 % *fall* buys under
2 %, and the gain saturates (`fr×0.5` is barely better than `fr×0.75`). ⚠ **That saturation
is on the non-leaf-reduction side only.** Raising the leaf share *directly* has not flattened:
`fl×1.8` is **+22.35 %** and the ladder stops there because `fl×2.0` drives the `dvs 0` share
above 1 and is refused arithmetically — not because the response ran out. So the frozen point
is a local top **in the `fr`/`fs` directions**; the `fl` direction is **unexhausted**, which is
the more useful half of this finding for §2.1. ⚠ `fr×1.25` and `fs×1.25` land within 0.4 % of each other on
peak LAI while cutting the leaf share by very different amounts at very different knots
(−13.5 % at `dvs 0` vs −2.8 %; −6.3 % at anthesis vs −25 %); that coincidence is **recorded,
not explained**, and no mechanism claim is made from it.

**FINDING 5 — the compensation scheme is not a nuisance parameter: it decides which gate
breaks.** Sending more carbon to leaf is clean when spelled `fl×1.8` (LAI 7.37, inside the
band; biomass *falls* to 12.96; the liveness floor rises 34 %) and **breaks the biomass cap**
when spelled `fr×0.75` (14.4621 against 14.4248) for a tenth of the LAI gain. The reason is
mechanical rather than physiological: `peak W excl. fibrous roots` is an **above-ground**
band, so lowering the root share raises it *by arithmetic* — carbon moves into the band's own
scope. ⚠ The `fl` column's near-flat biomass is **not** the same mechanism and is left as
measured: at anthesis proportional compensation takes ~72 % of what `fl` gains from **stem**,
itself above-ground, so that column redistributes *within* the band rather than into it.
⚠ **Two perturbations that move leaf carbon the same
way land on opposite sides of a gate depending only on where the carbon came from** — which
is why `partition::render_header` prints the scheme above the table, and why a single "the
partition table's sensitivity" number would have been unreadable.

**Built to get the numbers** (`rust/crates/domains/src/lab/partition.rs`, 5 tests;
`examples/partition_switch.rs`, a thin `main`). ⚠ The obvious route — mutate
`BiosphereParams::alloc` after loading — **skips every rule the table has** (per-row sum to
1, each fraction in `[0,1]`, strictly increasing knots, all inline in `allocation_from` and
all unreachable from a built struct), and re-stating them in the lab would put a second copy
of a rule in the tree. So the perturbed rows are **re-emitted into the frozen file's own
text** and loaded by the frozen loader: the validation is *the* validation. Controls, in the
order they earn their keep: **`×1.0` is bit-identical to `frozen` through the round trip**
(and prints `<- UNCHANGED` on all 8 readouts in every run above, so the plumbing is faithful
end to end); **a large factor must move a readout** (without it an inert harness and an inert
parameter are indistinguishable, and "the table is not a suspect" is exactly the finding that
would be wrong); the re-emitted file keeps its header, schema keys and `source:` so the
loader's provenance guard still has something to check; and impossible requests are refused
by name rather than clamped (`fl×2.0` drives `fl` to 1.1 and is an error, which is why the
leaf ladder stops at 1.8). Nothing here discharges `allocation.yaml`'s `TODO(cite)`.

**What this does NOT say.** It measures the shipped step only — no converged column for
`open_season`, so nothing here re-measures the plan's 4.42 beyond noting the contract's own
5.4273. Proportional compensation is **one** scheme; a different destination rule is a
different experiment. And the table is `TODO(cite)` provisional, so a ladder around it prices
sensitivity, never correctness.

**Successor, and it is a re-read, not a build.** §2.1's remaining content is what its own
subtitle already says — *"a provenance item first"* — but with the retired deviation removed
it is a **smaller** item than the plan's biggest, and the ranking in §4 was computed against
5.03. That ranking is now unsourced. ⚠ The next re-read is owed the same thing this one
found: check the plan's numbers against a **run**, not against the record they cite.
