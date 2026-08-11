## **The science assertions get contract standing** (the acceptance gate's own finding 6, adjudicated)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE (2026-08-09) — a SCHEMA unfreeze of both manifests; no value, golden, param hash
or `src/` change.** `docs/plans/post-roadmap-acceptance-gate-standing.md`. **The user's
call** on the decision finding 6 explicitly refused to make: standing via a **manifest
field**, scope **bands + liveness floors**. **THE REFRAME THAT UNBLOCKED IT — read off the
PINS, the "two gates" never overlap**: closure binds on the 6 chambers and is **structurally
empty** for carbon on `open_season`; the bands live on `open_season` and *cannot* exist on a
52 g DM/m² carbon-limited rig. All three of finding 6's cases are two verdicts on
**different scenarios**, aggregated by the reader. ⇒ promoting a band on `open_season`
**cannot reverse a measured closure refusal** — the co-adaptation objection needs a verdict
to overrule and the cell is empty, which is exactly why the diagnosis was right to hesitate
and why the hesitation dissolves. **TWO FIELDS, NOT ONE (advisor, and it preserved the
user's scope while splitting its representation)**: `science_bands` bounds come from
**outside** the repo (real wheat ~5–8 LAI, V-K&S 6.0, Greenwood's 14.4248 t/ha, BVAD's RQ);
`liveness_floors` were **tuned to our own calibration** — the perennial plant floor moved
`>1.0`→`>0.9` when the decomposer calibration shrank the plant ~19 % — so they guard
**continuity with the current calibration, NOT plausibility**. Freezing a floor under the
bands' name would say "the frozen tree passes a bound the frozen tree set". **THE INCLUSION
RULE, and clause 3 came from a counter-example**: a gate (1) asserts a physical quantity of
a frozen-roster scenario **run as frozen**, (2) against an outside- or self-sourced bound,
(3) **and is satisfied by movement TOWARD the reference** — which excludes
`test_chamber_scale.py`'s `ours/BVAD > 20.0`, outside-sourced and about the frozen tree, yet
**failed by a chamber resized toward the flight spec**. That is a *characterization*, not a
gate. **Three exclusions, each measured**: margin-ratio/doc-staleness pins (`peak_w/14.4248 >
0.85` fails when **prose** drifts; `0.80 < peak/6.0 < 0.92` fails when the margin
**improves**) ⇒ **two committed tests SPLIT** so a marked test carries only its gate;
diagnosis pins about **refused** forms (`peak > 15.0` for (C)); and **calibration
identities** — most of `test_bvad_validation.py` asserts quantities the crew params were
*fitted to*, which **its own docstring already calls true "by construction"**, leaving only
the RQ structural prediction. **MECHANISM**: a `science_gate` marker enumerated statically
by `ast` (`tests/science_gates.py`) — a collection-hook registry goes red on a single-file
run (collection is partial), and subprocess `--collect-only` costs a second collection of
the suite whose runtime was just cut 3.3×. The manifest entry names **quantity + bound +
source + locus**, not a test id, so a bound cannot be loosened in place with the gate green.
⚠ **STATION-SIDE THE RESULT IS MOSTLY EMPTY AND THAT IS THE FINDING**: **11 of 13**
scenarios carry no outside-sourced bound — established mechanically (no station run-test
defines a module-level sourced constant at all); only `crew_mission` (BVAD) and
`sealed_station` (a node floor) do. Every roster scenario gets an **explicit empty list** —
an absent key and a deliberately-empty one are different claims, and `drift_summary` is the
case that forces it. ⚠ **A DRAFT ARGUMENT WAS MEASURED FALSE**: "a manifest band is vacuous
because the golden already freezes peak LAI" — `season_euler_state.json` is a single
**endpoint** (`n = 305`), so the goldens constrain a trajectory *only at its last step* and
every mid-run quantity was unfrozen. ⚠ **THE CONVENTION CHECK COUNTED ITSELF**: the first
decorator-form test compared textual `mark.science_gate` occurrences to collected gates and
failed **13 vs 10** — its own docstring and code literal were three of them; matching
`@pytest.mark.science_gate` would have gone green *while losing the case worth catching*
(`pytestmark = [...]` has no `@`) ⇒ replaced by a structural `ast` check, where prose is not
an attribute access. **A self-referential text check is not a weaker version of the right
check; it is a different one that happens to be green.** ⚠ **THE DIAGNOSIS'S OWN PIN CAUGHT
THIS WORK AND WAS RIGHT TO**:
`test_the_plausibility_bands_that_exist_are_named_by_no_manifest` is falsified **by design**
— **resolved, not corrected** (a true measurement of a contract since changed) — and
**replaced by its INVERSE**, not deleted or relaxed (the option-(B) precedent: *a pin
guarding a mechanism you removed is decoration*). Its neighbour still **passes and is still
true** (a scenario *entry* has no plausibility column; standing lives in top-level fields
keyed by scenario) while its **docstring's conclusion** is now false — annotated in place,
since a sound assertion under a stale conclusion is this repo's most-logged shape. **Teeth
verified by MUTATION** (drop the BVAD marker ⇒ the station gate goes red), not by a green
bar. Both reference docs' unfreeze discipline gained a **"report the science gates"** step:
a band failure is a **blocking finding to be argued past in writing, never a bound to
re-tune** — and, stated in the same breath, a band **passing is not an endorsement**
(`open_season` sits **3.8 %** above the LAI floor and **12 %** below the Greenwood
crossing). `git diff src/` empty; full suite **2107 passed**; ruff + pyright clean.
