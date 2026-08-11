## **The direction gate — `ReversedFlowError`**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-11 (the user's call, taken the same day as the diagnosis above).**
`docs/plans/post-roadmap-o2-makeup-reversal.md`. The diagnosis priced this and left it as a
decision; the decision was build it. **A THIRD run-time verdict, and it is `RationedError`'s
SIBLING, not its variant** — they catch **disjoint** failures: a rationed run over-drew a
stock; a reversed run *never over-draws anything*, which is precisely why the backstop
cannot see it, and conservation cannot either (two legs, one magnitude, balanced whichever
way it points). Reversal is a **DIRECTION** defect and neither existing gate measures
direction, so a third gate is the honest shape rather than a stretched version of either.
**Data-driven off the registry, deliberately**: a new `FlowTypeSpec.demand_controlled` =
`(regulated wiring field, setpoint param)` — set on `eclss.o2_makeup`, `None` on the other
eleven — so *registering* a demand-controlled type is what arms the check, instead of a
hard-coded class list in the harness drifting from the registry it duplicates. Frozen in the
manifest for `rate_params`' exact reason (clearing it silently un-arms the check: a file
that used to raise starts completing quietly). **Mirrored on BOTH ports**
(`ErrorKind::Reversed`, `RunResult::first_reversal`, the watch threaded through all three
Rust stepping loops) — a Python-only gate would let an authored file raise in the laboratory
and complete **silently in the game path**, the exact divergence the port-mirror discipline
exists to stop; ⚠ the one real divergence is *where the pair is resolved* (Python reads it
off the built flow with `getattr`; Rust's flows own their fields privately, so `interpret`
resolves it) — same states judged, so the **rule** mirrors even though the plumbing does
not. **THE RECONCILIATION IS THE PART THAT COULD HAVE GONE WRONG**: three *frozen* scenarios
reverse **legitimately**, and a gate condemning them would be a regression dressed as a fix.
It cannot reach them for **two independent reasons, both PINNED rather than argued** — the
frozen builds never pass through `interpret`, and the biosphere is absent from the flow-type
registry, so no authored file can put a crop on the far side of the regulator. ⚠ **Reason 2
has an expiry date and the test says so**: making the biosphere authorable turns
`test_no_frozen_scenario_reaches_the_reversal_gate` RED, forcing the premise to be
re-decided rather than quietly outlived. **Three things measurement changed while
building**: (1) the gate is `>` **not** `>=` — the platform's own committed fixture wires
`cabin_o2` *exactly at* the setpoint (regulator idling), so `>=` would have condemned the
example the docs point authors at; pinned on both ports rather than left to luck; (2)
`test_authoring_export_fidelity.py` needed the opt-out on **all 13** call sites, because
that file exists to *study* the reversal — recorded in the file itself, with the teeth
deliberately housed elsewhere, since *a file that opts out of a gate is the wrong place to
assert the gate works*; ⚠ **and 13-in-1-file was itself a subset asserted of the set — the
real population is 17 sites across 4 files, and the extra 4 falsified a sentence written
inside this very work.** Running the suite turned `test_authoring_dt_hazard.py` (2),
`test_authoring_multirate_run.py` (1) and `test_authoring_multirate_composability.py` (1)
red: they study the `dt = 3600` hazard, where `k_makeup·dt = 7.2` puts the regulator past
its stability bound and `cabin_o2` oscillates to **72.0 mol against a 10.0 setpoint**,
tripping the new gate at step 2. `_check_no_reversal`'s docstring had said the only route to
a mid-run crossing was the `1 ≤ k·dt < 2` oscillation; the update map is `o2 → (1 − k·dt)·o2 +
k·dt·setpoint`, which alternates about the setpoint for **any** `k·dt ≥ 1` (converging below
2, diverging above), so the divergent half is just as reachable. Corrected in the docstring
and in `docs/authoring-reference.md`. **This is the third instance of this work's own thesis
inside this work** — after the "~0.08 % each" rounding and the "the goldens pin it"
inference — and the first two were caught by re-reading while this one was caught only by
*running*: a claim about a **family** is not checkable by re-reading the sentence that makes
it. The payoff is a wider gate than advertised: it is the only thing that reports the
excursion an oscillating regulator actually **exported**, at the step it happened, where
rationing catches only the violent end and conservation catches neither; (3) **the two
run-time gates barely overlap in practice**, because a run that both rations and reverses
can only be constructed with `allow_unsafe_step=True` — multi-rate Step 5's build-time `k·h
< 1` precondition refuses the coarse `dt` at interpret time — so the ordering (rationing
first, so a suspect trajectory is not blamed on wiring) only ever decides what a deliberate
study sees. **NOT covered, named rather than implied**: a sub-step-only excursion on the
multi-rate path (both ports scan committed **master** states; not closable from this layer
because `simcore` is frozen, and not the shape an author hits — the motivating crossing is a
wiring error present at `states[0]`). Manifest diff is **exactly the new field, 11 `null` +
1 pair**; no other value moved, **20 frozen goldens byte-identical**, `git diff
src/simcore/` empty. The decision **not** to clamp the physics is untouched and was
re-confirmed. ⚠ **Not advisor-reviewed before landing** — the advisor reviewed the
*diagnosis* (and corrected two claims in it), then was unavailable; the authoring discipline
asks for review on a **grammar** change especially, and this is a run-harness verdict plus
one registry field, no grammar/schema/value moved. Recorded as a deviation, not glossed.
