## **The winter-wheat partition backfill**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**TAKEN and REFUSED 2026-08-11.** The frozen partition table stays uncited; `git diff src/`
empty; no golden, manifest or cross-port artifact moved. [E] Table 18 **does** carry a
"Wheat, winter" entry (p. 91, read off a page image — its text layer renders that very row's
CASST as `PUNCTION CASST= 0:,0:53:0.33,0.5;`), converted by potato's own derivation at the
union of the three curves' own knots. **The rounding rule had to be stated, not inherited:**
exact rational arithmetic at 12 dp with the residual on the LARGEST fraction — potato's 6 dp
**misses the loader's 1e-9 tolerance at dvs 0.62** (sum 1.000001), and a fixed-column
residual invents a 1e-12 grain share where [E] has CALVT+CASTT = 1.000000 exactly. **THE
REFUSAL IS A MEASUREMENT, NOT A JUDGEMENT:** the cited table drives `open_season` peak LAI
to **2.2005** against the contract-standing band **5.0-8.0** (real wheat) — a **2.36x**
miss. Run WITHOUT `-x` on purpose: of the **10** `science_gate`-marked contract items
**exactly one fails**; the VKS mutual-shading ceiling, the Greenwood peak-W band and **all
four liveness floors** pass. The whole suite shows 26 red, but the other 25 are pinned
diagnostics that move under *any* allocation change — **separating gates from pinned
measurements is exactly what the 2026-08-09 standing work was for, and this is its first use
in anger.** **The cause is isolated to ONE term:** swapping only the root share back to the
placeholder's — keeping [E]'s early leaf peak, hard stem takeover AND pre-anthesis grain —
recovers the canopy to **5.339** (placeholder 5.191); the control that removes the late leaf
collapse overshoots to 13.259. And the divergence is confined to **DVS 0-0.33** (roots 0.50
vs 0.35), converging by DVS 0.5 (0.2875 vs 0.275) — so **peak canopy is set by assimilate
diverted during the COMPOUNDING phase; diversions after it are nearly free**, which is why
[E]'s grain opening at DVS 0.77 (**before anthesis** — retiring the frozen header's "FO is 0
before anthesis", a sentence NO test encodes) costs essentially nothing. ⚠ **THE REFUSAL
DOES NOT MAKE THE PLACEHOLDER BETTER SCIENCE.** Against the oracle's own implied root share,
**neither table dominates**: WOFOST is 0.474 over DVS 0-0.2 (**[E]'s 0.50 is closer**),
0.380 over 0-0.33 (WOFOST sits *between* the two), 0.067 over 0.5-1.0 (**[E]'s ~0.115 is
closer** than the placeholder's ~0.24). The placeholder passes the band **because it was
fitted to it**. **THE REAL FINDING, and the successor work:** `ROOT_C` is read in exactly
one place outside plumbing (`nitrogen.py:256`, a biomass sum for N *demand*) — **there is no
uptake function**, N and water are non-limiting in `open_season` anyway, and senescence
bleeds roots away, so below-ground carbon is **dead weight by construction**. The frozen
table's canopy physicality **rests on a fitted root share compensating for a missing
mechanism**. A **LOCUS** failure in the project's established sense — faithful citation,
correct transcription, wrong model context. **Do not re-attempt until roots do work.** ⚠
**SUPERSEDED IN PART 2026-08-11:** that successor was taken the same day and its nitrogen
half is **REFUSED** — [E] p. 136 decouples root function from root MASS on purpose, and
uptake is demand-bound so no supply-side coupling can bite. "Until roots do work" is not
reachable by the route this sentence implied; the one live route is soil layers, for water.
See the root-functional-coupling row. ⚠ **A committed claim corrected:** potato's "one
cause, two symptoms" (canopy + tuber both downstream of the early tuber onset) was asserted,
never measured; wheat reproducing the same shortfall from *roots* with a storage organ
opening at 0.77 made it worth testing. Holding potato's tuber to anthesis closes **39 %** of
its canopy gap (roots-from-fitted-wheat 5 %; roots removed outright 86 %) — supported in
direction, overstated in strength, corrected in the potato doc and test comment. **The two
crops invert which term dominates** (roots for wheat, tuber for potato), which is why the
unifying statement is *early diversion, whatever the organ*. The 39 % is prose, **not
pinned** — a stated deferral. ⚠ **The premise the thread was picked on was FALSE:**
allocation splits a non-negative `DMI` among `[0,1]` fractions, so **no partition table can
ever move mass OUT of the stem** — the stem-reserve refusal survives untouched. But the
sweep that located Table 18 surfaced the actual blocker on **[E] p. 93 §3.2.4**: *"a certain
fraction of the increase in stem weight will be available for redistribution after
flowering"* + **Table 7** magnitudes + Listings 3/4 — a model FORM, on the shelf, in a
source already first-hand for four other rows. Logged as a separate work item: **a recorded
blocker is dated — re-check the artifact.** That lesson has now caught us twice.
`docs/plans/post-roadmap-wheat-partition-backfill.md`
