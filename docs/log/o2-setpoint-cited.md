## **The cabin oxygen setpoint is CITED** (and the number was never inert — six frozen flows had been reading it for four phases while nobody could see it)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> Plan of record: `post-roadmap-o2-form-adoption.md`, §11 (the plan and its predictions) and
> §12 (what the run said). Filed under the third direction plan.

**BUILT 2026-09-06 on the user's call.** Offered adoption of the live-O₂ FvCB form, the user
chose *"fix the setting first"* — so this is **slice 1 of two**, and adoption is not taken here.
`eclss.yaml`'s `o2_setpoint` moves **10.0 → 1995.0 mol**, derived from a cited cabin
atmosphere. An unfreeze of the **station** contract: six goldens, one param hash, six golden
hashes.

**WHY IT WAS FOUND AT ALL, and this is the whole shape of the item.** The oxygen form's
adoption was priced in the third direction plan at *"only `sealed_chamber` — the jar… the two
big chambers move +0.140 % / +0.118 %"*. That is a true statement about the biosphere lab's
seven-scenario roster and **false about the tree**: the lab's harness builds biosphere params
and cannot reach a station assembly. Measured on a reverted working tree, adoption moves
`sealed_station`'s principal stocks by **60–75 %** — and the cause was not the science.
`greenhouse_bio_scenario` carries **two gas charges against the same 9500 mol of air**: CO₂ at
3.796 mol reads 399.6 ppm, a real cabin; O₂ at 10.0 mol reads **1.05 mmol/mol, half a percent
of breathable**. The CO₂ charge was set against the air inventory, the O₂ charge to match this
param — two coherent conventions, never reconciled, and **nothing in the tree had ever divided
the oxygen pool by the air.** Adoption is the first thing that would have.

**FINDING 1 — the param file predicted this finding in its own `source:` string, and was
wrong about why.** It read *"PERMANENTLY un-bindable as written — real systems regulate O₂
PARTIAL PRESSURE, never a mole count, so the units themselves foreclose citation"*, and named
the fix as *"a model change, not a citation edit."* The prediction was right and the diagnosis
was not: **`V` and `T` are constants in this model** — there is no volume state and no cabin
thermodynamics — so a mole target and a partial-pressure target differ by a fixed factor, and
regulating one *is* regulating the other. It needed Dalton's law and the cabin's own air
inventory, both already in the tree. *An "un-bindable" verdict can be a fact about the missing
derivation rather than about the model form, and the two look identical from inside the file.*

**FINDING 2 — the uncited number was NEVER INERT, which is the argument for doing this
first.** Six frozen respiration flows throttle by `f_O2 = x/(k + x)` on this same pool
(`respiration.yaml`'s `o2_half_saturation = 1e-4`), and that coupling is live, not lab-only.
So the station's soil respiration had run at **f_O2 = 0.8951** for four phases, where every
correctly-charged chamber sits at **0.99952**. Adoption did not create the defect; it was the
first thing that would have made it *visible*. ⚠ Measured consequence at the fix: microbial
carbon **+10.86 %** (greenhouse) and **+11.21 %** (harvest) against an 11.7 % release of the
throttle — the number lands almost exactly on its own cause. *A value nothing divides is not a
value nothing reads.*

**FINDING 3 — a rate change raises a pool early and lowers it late, and one horizon alone
gives the wrong sign.** The 28-step `greenhouse` / `harvest` runs show humus **+24 %** and
microbial carbon **+11 %**; the 4880-step `sealed_station` shows microbial **−12.77 %** and
litter **−12.08 %**. Same mechanism, opposite signs: faster decomposition builds the pools
before it draws the litter down. The plants barely move either way (leaf −0.047 % on the
station, −0.335 % on the greenhouse), which is the right shape — `f_O2` gates *respiration*,
and the microbes are the ones respiring.

**FINDING 4 — a bit-identity prediction across a translation is a claim about REAL arithmetic,
and the machine does not make it.** §11f predicted in bold that `boundary.o2_supply` would be
**bit-identical** on the three ECLSS-only runs, because the regulator's dynamics live in
`e = setpoint − cabin_o2` and both the setpoint and the initial condition shift by the same
+1985.0. The pools did shift by **exactly +1985**, and only 2 of 9–10 stocks moved at all — but
the supply moved by **7.24e-11** (3.4e-13 relative). Subtracting two numbers near 1995 rounds
differently from subtracting two numbers near 10. ⚠ Worth keeping as a fact about the model
rather than about this edit: **the regulator is now solving for a small deviation on top of a
large inventory**, so it is less well-conditioned at 1995 mol than at 10 — harmlessly at
3e-13, but that is the direction.

**FINDING 5 — a search that finds an instance of what it is looking for stops looking.** Two
reds went unpredicted. `params::tests::eclss_loader_reads_the_committed_params` was written off
in the plan as one of the synthetic-YAML loader tests — true of its neighbours, false of it.
And two **flow-level** makeup tests built states at the old setpoint; the plan had checked the
two `makeup_flux_*` unit tests, found they pass their arguments explicitly and stay green,
reasoned correctly about those, and wrote the conclusion as if it covered the family. The pair
sits one screen further down the same file. **Third instance in this item's own document** —
the ×685 margin was read at the wrong instant for the same reason.

⚠ **Both were re-posed rather than re-valued, and that is a strengthening.** They now read
`HAND.o2_setpoint − deficit` and `HAND.o2_setpoint`, because their claims are *"proportional to
the deficit"* and *"idle at the setpoint"* — neither is a claim about where the setpoint sits.
The old form pinned two things in one assertion and went red for the wrong one. The setpoint's
value is pinned once, in the loader test, which is where it belongs.

**FINDING 6 — a control whose generator is deleted cannot absorb a deliberate value move.**
`domains::params::tests::every_value_matches_the_generated_table` asserts bit equality of twelve
params against `src/sibling_params.txt`, whose header reads *"GENERATED, do not edit. Source of
truth: the frozen Python loaders"* and whose regeneration line names a file **S6 deleted**
(`tests/crossport/` no longer exists). Hand-editing the hex float was **refused**: the file
asserts its numbers came from Python's loaders, and writing one Python never produced makes it
lie about its own provenance. The `o2_setpoint` **row is retired**, the count assertion goes
**12 → 11**, and that is called a weakening rather than filed as a tidy-up. ⚠ Same class as
`drift_summary`'s pre-2026-09-06 defect — *a frozen artifact with no regeneration path* — and
unlike that one it cannot be fixed, because the port it was a control against is gone.

**THE CHOICE OF ATMOSPHERE IS SCIENTIFIC AND IS RECORDED AS ONE.** The citation is BVAD Rev 2
§4.1.1 **p. 61**, verbatim: *"ISS EVA operations originate from 21% oxygen and 101.3 kPa (14.7
psia) of pressure"* — a cabin **composition**, which is the quantity this model needs.
⚠ Deliberately **not** Table 4-1's *"p[O₂] for Crew; nominal no impairment"* 21.2 kPa, which is
a crew physiological requirement and a different quantity; that it agrees to 0.4 % (20.93 % vs
21 %) is a cross-check across two loci, not a second derivation. ⚠ The same page offers **34 %
oxygen at 57 kPa** (Norcross 2013) for surface habitats doing frequent EVA, and Table 4-1 gives
three nominal total pressures (101 / 70.3 / 56.5 kPa). **The crop reads the mole fraction**, so
those are not interchangeable: this habitat is modelled ISS-like and sea-level-equivalent, and
that is a choice. ⚠ **The shelf paid again** — the PDF this came from is the same document the
2026-07-02 crew-params retrieval fetched, and an extract of it was already sitting in the temp
tree. *Before pricing a retrieval, open the shelf* — this is the sixth instance and the first
where the shelf held the *next* section of a document already cited.

⚠ **THE TRIGGER IS IN THE `source:` STRING, because the simplification is real.** The value
silently encodes the **one** air inventory within `O2Makeup`'s reach (9500 mol); the other four
ECLSS scenarios — standalone, `cabin_gas`, `water_recovery`, `crew` — carry **no air inventory
at all** and never convert a pool to a concentration. The structurally tidier design (setpoint
as a mole *fraction*, the flow multiplying by the air) was advised and **rejected on that
measurement**: it would force inventing a cabin size for four scenarios that lack the concept.
Where the setpoint is used, the two designs differ by a constant. So the file carries the
condition: *if an ECLSS is ever wired to a cabin whose air is not 9500 mol, this MUST become a
fraction and the flow MUST read the air.* **A documented simplification and a hidden one differ
by exactly that sentence.**

**Predictions, scored.** Goldens: predicted **6 of 20**, measured **6 of 20 and the same six**
— every scenario predicted unchanged came back `identical`, including both drift summaries. The
+1985.0 translation: exact. The manifest: predicted *"the `eclss.yaml` sha-256"*, measured
**seven hashes** — that one plus the six `golden_sha256` the manifest carries per scenario;
benign, and caught only because the ceremony asks for the prediction. Relation pins held as
designed: `eclss_run.rs`'s `o2_eq = o2_setpoint − Con/k` and the station's `crew_mission` gate
are both green, because **a pin on a relation survives a value move and a pin on a value does
not**.

**Gates, stated as what was actually run at commit time rather than as a summary.**
`-p domains --lib`: **404 passed, 0 failed.** `-p station -p domains --test manifest_writer`:
green after regeneration. `-p repo_gates`: **25 passed**, covering the index/pointer/record
parity, the plan-doc indexing, the direction-plan re-read gate and the memory bounds.
`regen_goldens`: **20 of 20 run, 6 rewritten.** ⚠ The first
`cargo test --workspace --no-fail-fast` was **still running at the first commit**,
and that commit's record said so rather than claiming a result it did not have. **It came back
with four reds** — FINDING 7, which no targeted run could have found. `-p authoring`:
**96 passed, 0 failed** after the fix.
**FINAL**: `cargo clippy --all-targets -- -D warnings` **exit 0**;
`cargo test --workspace --no-fail-fast` **1139 passed, 1 failed** — and the one red was this
file, over the 120-char record-line cap, because the gates were run and *then* the record was
appended to. ⚠ *A green gate is a claim about the tree that existed when it ran*, which is the
same shape as everything else in this item; wrapped and re-run to 1140/0.
⚠ The station manifest byte gate **reddened first and by itself** — the automatic guard this
unfreeze had, confirmed present before the edit rather than hoped for.

**FINDING 7 — a FOURTH copy of the setpoint lived in AUTHORED CONTENT, and the full suite
is what found it.** The targeted runs were green and the workspace run was not: four
`authoring` tests reddened. `rust/data/scenarios/eclss_cabin.yaml` carries
`amount: 10.0  # EclssScenario.cabin_o2_0 (== the frozen o2_setpoint)` — a fixture whose
oxygen amount **means** *"at the setpoint"*, which is precisely what the direction gate's
boundary case is pinned on (*"a `>=` gate would have condemned the platform's own example"*).
It moved to 1995.0, because its meaning is what must be preserved, not its digits. Two tests
that wired the cabin *above* the old setpoint (20.0) now wire it above the new one, through a
named `SETPOINT` / `ABOVE_SETPOINT` pair rather than four scattered literals.

⚠ **One test in that family stayed GREEN while its stated premise became false.**
`at_the_setpoint_the_gate_is_silent_because_the_flow_does_not_reverse` passed throughout —
but between the setpoint moving and the fixture following it, the fixture was **not** at the
setpoint, and the test passed because the flow was strongly *forward* rather than because the
magnitude was zero. *The most dangerous member of a broken family is the one that did not go
red.*

⚠ **And the hazard count moved 37 → 38, against a prediction of "fewer, possibly zero".**
Wrong in direction: a 200× larger inventory needs **one more overshoot** to reach the zero
clamp, not fewer. The mechanism is unchanged — the airless-cabin assertion still holds at 38 —
so it was **re-measured, not re-tuned**. ⚠ The real cost is the other half: 37 was the last
number **both ports** ever produced, and S6 deleted the Python side, so *"both ports ration
identically"* can never be re-established at 38. `log/rationing-gate.md` and its plan are
dated rather than corrected. *A cross-port parity number outlives the port that made it
checkable; the day one side moves is the day it stops being a gate and becomes a date.*

**What is still owed: slice 2, adoption.** The band re-poses cleanly (measured ×10.67 at its
binding instant, not the ×685 the direction plan quoted at the wrong instant), the `ci_ratio`
objection was refuted on its sign, and the station's 60–75 % is now expected to shrink to the
control-sized move adoption was priced at. **That last sentence is a prediction and has not
been measured.**
