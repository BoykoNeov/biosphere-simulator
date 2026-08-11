## **The O₂ regulator's reversal is inside the freeze, not outside it**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED + CORRECTED 2026-08-11 — no clamp, no value moved, no golden regenerated, no
manifest hash touched.** `docs/plans/post-roadmap-o2-makeup-reversal.md`. Taken as "fix the
oxygen-makeup leak" (the hazard `memory/post-roadmap-direction.md` named 2026-07-17 as the
next bucket-2 increment); **both halves of that were superseded, in opposite directions.**
(1) **The clamp is already refused on the record, with a physical reason** — the
export-fidelity work decided it in `docs/authoring-reference.md`: *"Do not 'fix' this with a
clamp: the symmetry IS the restoring force — `o2_eq = o2_setpoint − Con_o2/k_makeup` is an
attractor from both sides only because the controller is linear … the reversal is correct
P-control."* Not reopened. (2) **The hazard is NOT author-only, and nobody had measured
that.** Three committed loci (`domains/eclss/flows.py` *"a deferred seam that never arises
here"*, `docs/authoring-reference.md` *"true of every frozen scenario"*,
`authoring/flow_registry.py` *"reachable by an author"*) scope the reversal to authored
content. Measured against **the runner each golden uses** (the acceptance-gate method, not
re-derived), the split is **exactly standalone-vs-plant-coupled, 3–0 and 0–3**:
`eclss_steady_state`/`cabin_gas`/`water_recovery` peak at **exactly** 10.000000 with 0
reversals, while `greenhouse` (1/10 080), `harvest` (3/30 240) and `sealed_station` (**1 in
1 756 800**) all reverse, peaking 10.008081 (**0.081 %**) / 10.000304 (**0.0030 %**) against
the 10.0 mol setpoint — a **27× spread** first written as one "~0.08 % each" in three loci,
i.e. *this row's own thesis committed inside its correction*, caught in review and now
carried as two numbers. `lighting` carries no `O2Makeup` at all. **⚠ THE FAILURE SHAPE IS
NEW, and it is not the paraphrase one.** The sentence was **true when written** (Phase 5
Step 6, when only the standalone cabins existed) and was **falsified by P6.3's seam three
phases later** — nothing was misquoted; *the world under the sentence moved, and no gate
anywhere connects the two*. **The mechanism is the seam, pinned structurally** rather than
left as a number: the coupled builds wire the regulator to the **biosphere's** `O2_POOL`
("crew breathes the biosphere O₂ pool"), which has a second and larger source the regulator
does not model — the crop — so photosynthesis overshoots and the regulator pushes the excess
back into `boundary.o2_supply`. ⇒ **Tier 1's generalization survives with its own example
inverted**: *authoring* is not the only thing that escapes a frozen flow's safety scope —
**a cross-domain seam does it too, inside the freeze**, with no authored file and no author
involved. **Three gates were each blind for a different reason** (conservation: two legs one
magnitude, balances either way; rationing: the draw is ∝ the setpoint *error*, never
over-draws; the goldens: they freeze the **endpoint**, and these are 1–3 single mid-run
calls — so they are **BLIND** to it, which a draft of this row asserted the opposite of. ⚠
**Measured, not inferred, after advisor review**: clamping `makeup_flux` leaves `greenhouse`
/ `harvest` / `sealed_station` / `sealed_energy_drift` goldens **byte-identical**, so the
clamp is **bit-identically inert on the whole roster** — the canopy regulator's shape a
second time, and the reason it matters is that the refusal now rests SOLELY on the physical
argument, never on a cascade price this work had assumed). **NOT a defect**: at ~0.08 % over
setpoint the controller is working, and removing the reversal would remove the upper half of
the attractor. **What stays open, priced not proposed:** the author-facing trap is still
uncaught (`cabin_o2 = 20.0` ⇒ a silent −1.2 mol/step drain); a reversal gate at
`run_scenario` (the `RationedError` locus/shape) would close it, but it is an
authoring-platform **unfreeze** and would have to be reconciled with three *frozen*
scenarios that reverse legitimately. ⚠ **Not advisor-reviewed** — unavailable all session (3
attempts, all overloaded), which is exactly why this work stops at prose + pins: neither
needs ceremony, and anything that moved a golden would need the review step the station
discipline puts first. Pins in `tests/test_o2_makeup_reversal.py` (4 cases, 1 `slow`),
**teeth verified by mutation** (`max(0.0, …)` on `makeup_flux` turns the plant-coupled pin
red; tree restored via `git checkout`), counts/peaks asserted as existence + band because
the excursion is downstream of the biosphere transcendentals (the cross-libm trap), the
standalone side exact because it is `+ − ×` only.
