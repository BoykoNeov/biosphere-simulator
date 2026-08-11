# Post-roadmap: the O₂ regulator's reversal is inside the freeze, not outside it

**Status: DIAGNOSED + CORRECTED 2026-08-11. Read-only on the science — no clamp, no
value moved, no golden regenerated, no manifest hash touched.** Probe in
`M:/claud_projects/temp/o2_makeup/probe.py`; pins in `tests/test_o2_makeup_reversal.py`
(4 cases, none `slow`).

## What was being asked, and what it turned into

The task taken was "fix the oxygen-makeup leak" — the hazard
`memory/post-roadmap-direction.md` named on 2026-07-17 as the natural next bucket-2
increment:

> `eclss.o2_makeup` is unclamped, so an author wiring `cabin_o2` above the setpoint
> silently vents O₂ back to the tank. That is the natural next bucket-2 increment.

**Both halves of that turned out to be superseded, in opposite directions.**

1. **The clamp is already refused, on the record, with a physical reason.** The
   export-fidelity work (bucket 2) measured the reversal and `docs/authoring-reference.md`
   decided it: *"**Do not 'fix' this with a clamp**: the symmetry IS the restoring force —
   `o2_eq = o2_setpoint − Con_o2/k_makeup` is an attractor from both sides only because
   the controller is linear, and clamping would trade a clean geometric contraction for a
   piecewise nonlinearity. The reversal is correct P-control."* That decision stands and
   this work does not reopen it.
2. **The hazard is not author-only, which is what nobody had measured.** Three committed
   loci scope the reversal to authored content. Measured, it fires **inside the frozen
   goldens**.

So the deliverable is not a fix. It is a **falsified scope claim**, corrected at every
locus and pinned.

## The measurement

Method: patch the module-level `makeup_flux` (which `O2Makeup.evaluate` looks up by name
at call time, so every evaluation is recorded — including multi-rate sub-steps that never
commit a state, which a scan over committed states would miss), then drive each frozen
scenario with **the runner its own golden uses**, imported from the committed
`test_regression_*` modules rather than re-derived — `post-roadmap-acceptance-gate.md`'s
method, for its reason.

| frozen scenario | regulator watches | calls | peak O₂ | reversed |
|---|---|---|---|---|
| `eclss_steady_state` | `eclss.cabin_o2` | 900 | 10.000000 | **0** |
| `cabin_gas` | `eclss.cabin_o2` | 900 | 10.000000 | **0** |
| `water_recovery` | `eclss.cabin_o2` | 1 800 | 10.000000 | **0** |
| `greenhouse` | `biosphere.o2_pool` | 10 080 | 10.008081 | **1** |
| `harvest` | `biosphere.o2_pool` | 30 240 | 10.008081 | **3** |
| `sealed_station` | `biosphere.o2_pool` | 1 756 800 | 10.000304 | **1** |
| `lighting` | — | — | — | no `O2Makeup` in the build |

Setpoint 10.0 mol. The split is **exactly** standalone-vs-plant-coupled, 3–0 and 0–3.

⚠ **The three excursions are NOT one magnitude, and a draft of this document said they
were.** `greenhouse`/`harvest` overshoot by **0.081 %**, `sealed_station` by **0.0030 %**
— a **27× spread**, written as "~0.08 % each" in three loci before the census table it
was drawn from was re-read. That is *this document's own thesis* — a number true of a
subset asserted of the set — committed inside the correction of it. Caught in review; the
loci now carry both numbers. A document about scope drift cannot round its own subsets
together.

**Why these pins carry no `science_gate` marker.** They fail clause 3 of
`post-roadmap-acceptance-gate-standing.md`'s inclusion rule — *a gate must be satisfied
by movement toward its cited reference*. A model change that legitimately **stopped** the
overshoot (a bigger cabin, a smaller crop, a real ppO₂ regulator) would turn
`test_the_regulator_does_reverse_…` **red** while improving the model. That is the
`test_chamber_scale.py` characterization shape exactly, and characterizations do not get
contract standing.

## FINDING 1 — the sentence was true of its subject and false of the roster

`domains/eclss/flows.py` said the venting clamp is *"a deferred seam that never arises
here"*; `docs/authoring-reference.md` quoted it and glossed it *"true of every frozen
scenario, but an author can wire `cabin_o2` above the setpoint"*;
`authoring/flow_registry.py` called the boundary *"reachable by an author"*.

Every one of those is correct about the **standalone ECLSS cabins**, where the claim was
originally written (Phase 5, Step 6 — the only scenarios that existed then). None is
correct about the roster, which Phase 6 grew by wiring the regulator across a seam.

⚠ **This is the project's most-logged failure shape, in a new place.** It is the same
mechanism as *"any 'every source says X' is a claim about YOUR SHELF"*
(`post-roadmap-citation.md` round 6) and the acceptance gate's *"the careful sentence
stays put and the paraphrase travels"* — but here nothing was paraphrased. **The sentence
never changed; the world under it did.** The claim was true when written and was falsified
by P6.3's seam, three phases later, and no gate anywhere connects the two.

## FINDING 2 — the mechanism is a seam, and that generalizes Tier 1's lesson by inverting its example

`build_eclss` / `build_cabin` / `build_water_recovery` give the regulator a cabin stock
whose **only** O₂ source is the regulator itself, starting at the setpoint under a
consuming crew — so the error `(o2_setpoint − cabin_o2)` structurally cannot go negative.
`build_greenhouse` / `build_sealed_station` wire it to the **biosphere's** `O2_POOL`
("the seam: crew breathes the biosphere O₂ pool"), which has a second and larger source
the regulator does not model: **the plant.** Photosynthesis overshoots the setpoint; the
regulator pushes the excess back into `boundary.o2_supply`.

Pinned structurally (`test_the_reversal_is_the_biosphere_seam_not_a_cabin_stock`) so the
finding cannot be re-read as "the greenhouse cabin happens to run rich".

Tier 1's transferable generalization was: *a frozen flow's safety argument is scoped to
its frozen scenario data, and **authoring** is what escapes that scope.* That survives —
with its own example inverted. **A cross-domain seam escapes it too, and does so inside
the freeze**, where no authored file and no author is involved.

## FINDING 3 — nothing was going to catch this, and the reasons are three different ones

* **Conservation cannot see it.** Two legs, one magnitude: reversed or not, OXYGEN
  balances to the last digit. Already pinned by
  `test_the_reversal_conserves_which_is_why_no_gate_sees_it`.
* **Rationing cannot see it.** The draw is proportional to the setpoint *error*, not to
  the stock, so it never over-draws and the backstop never fires — the export-fidelity
  finding, restated on frozen rather than authored content.
* **The goldens cannot see it — measured, after a draft asserted the opposite.** They
  freeze the **endpoint**, and these excursions are 1–3 single calls mid-run
  (`sealed_station`: 1 in 1 756 800). A draft wrote that they are therefore *"in the
  frozen bytes — the goldens pin the reversal rather than refute it"*. That was an
  **inference, and it is false.** Applying `max(0.0, …)` to `makeup_flux` and re-running
  the golden gates leaves **every plant-coupled golden byte-identical**:
  `greenhouse`, `harvest`, `sealed_station` **and** `sealed_energy_drift` all pass
  `*_golden_bytes_match` **unchanged**. So the goldens do not pin the reversal *or*
  refute it — they are **blind** to it, and the blindness is the third mechanism, not a
  restatement of the first two.

  ⚠ **The consequence is bigger than the correction.** A clamp is **bit-identically
  inert on the entire frozen roster** — which means the clamp's price was never the
  golden cascade this work assumed it would be, and the refusal rests **solely** on the
  physical argument in `docs/authoring-reference.md` (the attractor is two-sided only
  while the law is linear). That is a *stronger* place for a refusal to rest than a cost
  argument, and it is now known rather than presumed. **This is the canopy regulator's
  shape exactly** (`post-roadmap-canopy-regulator.md`: sourced, ready, and
  bit-identically inert on every frozen scenario) — the second independent instance, so
  the pattern is worth naming: *in this tree, "it moves no golden" and "it changes
  nothing that matters" are different statements, and the first does not imply the
  second.*

  The measurement cost ~30 s for the fast pair and one 4-minute Tier-2 run. It was not
  run in the first pass because the sentence "read obviously true", which is the same
  reason `docs/test-suite-runtime.md` records for keeping its two negative results.

## What this does NOT claim

* **Not that the model is wrong.** On these magnitudes (~0.08 % over setpoint, peak
  10.008081) the controller is doing its job on a cabin whose plants out-produce its crew.
  Removing the reversal would remove the upper half of the attractor.
* **Not that the clamp question is reopened.** It is decided and the decision stands.
  ⚠ But be precise about *what changed underneath it*: the reversal is now known to be
  real in three frozen runs rather than hypothetical, **and** the clamp is now known to
  move no golden byte anywhere in the roster. So the refusal no longer has a cost
  argument standing behind it — it rests entirely on the physical one, which is where a
  refusal should rest, and this is recorded so a future reader does not re-derive the
  cascade price this work started out assuming.
* **Not that the author-facing trap is closed.** An author wiring `cabin_o2 = 20.0` still
  gets a silent `−1.2 mol/step` drain with no error. `run_scenario` raises on rationing but
  has no notion of a demand-controlled flow running backwards. **Priced, not proposed:** a
  reversal gate at `run_scenario` (the `RationedError` locus and shape, with an
  `allow_*=True` opt-out) would close it — but it is an authoring-platform change, so an
  unfreeze with its own ceremony, and it would have to be reconciled with the fact that
  three *frozen* scenarios reverse legitimately. Left as a decision.
* **Not advisor-reviewed.** The advisor was unavailable throughout this session (three
  attempts, all "temporarily overloaded"). That is why this work stops at the line it
  does: correcting prose and adding pins needs no ceremony, while anything that moved a
  golden would need the review step the station unfreeze discipline puts first.

## Pins (`tests/test_o2_makeup_reversal.py`)

1. the three standalone cabins never reverse, and their peak is **exactly** the setpoint
   (the control — without it, "it reverses somewhere" is compatible with plain
   instability);
2. `greenhouse` + `harvest` do reverse, with the peak inside a band;
3. the greenhouse regulator is wired to `biosphere.o2_pool` and not to `eclss.cabin_o2`
   (the mechanism, structurally);
4. `sealed_station` is **named as uncovered**, with the reason. It reverses (measured
   once: 1 in 1 756 800, peak 10.000304) but is not pinned: recording flux needs the
   trajectory re-run — `run_tier2` is not cached at the function level, and the session
   fixture caches *states*, which cannot show a single mid-run call — so a pin would add
   a fresh ~3 min Tier-2 run. `post-roadmap-acceptance-gate.md` measured that exact cost
   (22m34s → 6m47s) and established the rule this defers to: **the number of expensive
   *tests*, not of expensive scenarios, sets the bill.** Stated in a test rather than a
   comment so the omission is not silent — this repo's "no silent caps" rule.

⚠ **Counts and peaks are asserted as existence + band, never as exact values.** The
excursion is downstream of the biosphere's `math` transcendentals, so an exact pin would
be the cross-libm trap (`memory/ci-python-job-red-on-linux.md`). Exact numbers live in the
table above, where a stale one is harmless. The standalone side *is* exact — those
trajectories start at `cabin_o2_0 = 10.0` and only decrease, so the maximum is the initial
value, `+ − ×` with no transcendental in it.

**Teeth verified by mutation, not by a green bar.** With `max(0.0, …)` applied to
`makeup_flux`, `test_the_regulator_does_reverse_in_the_frozen_plant_coupled_scenarios`
goes red and the other two stay green. The tree was restored via `git checkout`.
