## **Experiments may ask what-if questions** (a user rule change, and the lab tool that honours it — the checks it lifts were validity checks, not official limits)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-29 on the user's call.** Two commits: `c2df283` (the rule) and `2e9664f` (the
tool). **20 of 20 goldens identical, no manifest byte moved** — the reference loaders are unchanged.

## The rule

The user: *"making up science with no citation, or tuning numbers to pass — can be used in
mechanisms for tests and experiments ('what would happen if this works that way...')"*. It is
the fourth provenance class, **WHAT-IF**, written into `docs/param-file-conventions.md`: lab-only,
tagged, a tuned pass is reported as FITTED, and it reaches the reference only by a citation plus
the unfreeze ceremony — never by relabelling. Licensing and the PCSE rule are untouched. What it
reopens for the parked leaf mechanism: `docs/plans/post-roadmap-leaf-rust-remeasure.md` §7.

## The tool

The user's next call was *"change it"* — the lab refused a substituted value that failed a
loader's range check, calling it "an unfreeze, not an experiment". Now:

* the 16 range-checked loaders take `Bounds::{Enforce, WhatIf}`; `*_from` still enforces, and
  unit, schema and field-name checks run on both routes (they catch typos, not questions);
* `lab::biosphere_what_if` / `value_switch -- <field>=<value> --what-if`; the column heading
  reads `WHAT-IF …` by code, not by the caller's label;
* a what-if whose params fold to a non-finite value (`carbon_fraction = 0` divides) is refused;
* a run whose readout goes NaN or infinite prints as dead, not as a number.

⚠ **I misdescribed the checks to the user before building.** I called them "official limits";
they are validity checks (a rate must be positive, a fraction must lie in `[0, 1]`, a band must
be ordered). Read a guard before naming it.

⚠ **The documented example was wrong on first writing**: `decomposition_rate=0` needs no flag —
zero is inside its check — so it could not show what the flag does. Caught by running the doc's
own command both ways; the doc now uses `max_extension_rate=0`, which panics without the flag
and runs with it.

## A finding on the way — ⚠ OVERSTATED, corrected the same day

~~The engine's per-step conservation check compares `residual.abs() > tol`, and **a NaN residual
passes it silently** (every comparison with NaN is false). The engine core is frozen, so it was
not changed; the lab now marks non-finite runs dead so a what-if cannot report a NaN as a result.
Whether the core should reject NaN is an unfreeze question for the user.~~

⚠⚠ **Corrected 2026-09-29, on the user's call to work it.** The comparison *is* blind to NaN,
but the engine never hands it one, and "an unfreeze question" was wrong twice over: nothing needs
fixing in the core, and the freeze doc says there is **no** unfreeze path that edits `simcore/`
at all (`biosphere-reference.md`, the unfreeze discipline, step 2). I told the user it needed the
formal unfreeze process; it did not.

* **Every way a value enters a step refuses non-finite first**: a leg (`Leg::new`), a stock
  amount (`Stock::new` / `with_amount`), an aux value (`State::new`), a forcing
  (`SourceResolver::bind`). Aux and forcing were already tested in `simcore`, and so was
  `Stock::new` — but **not** `Leg::new`, and **not** `with_amount`, the per-step write the
  integrator actually uses. Both are now pinned directly, from outside the frozen crate:
  `rust/crates/domains/tests/non_finite_refusal.rs`.
* **Measured, two layers deep.** With `Leg::new`'s check disabled, a NaN flow is still refused
  when `with_amount` writes the pool. With **both** disabled, the step completes, the pool reads
  NaN, and the conservation check passes it. So the blind spot is real, and covered twice.
* ⚠ The only theoretical route past both is overflow (two finite amounts near ±1.8e308 whose
  difference is infinite) — not guarded, and no scenario is within 300 orders of magnitude of it.

**And the probe found the lab's REAL hole, which the wrong story had hidden.** With the what-if
refusal bypassed, `carbon_fraction = 0` left every stock finite — the infinity sits in
`sla_per_mol_c`, and the canopy's exponentials absorb it — and put it in a **report fold**:
`open_season`'s peak LAI printed as `inf`. The dead-run guard read only the raw series, so it
passed. The report now also marks a run dead when a *folded* readout is non-finite
(`lab::tests::an_infinite_param_that_reaches_a_run_is_reported_dead_not_printed`; red with the new
condition disabled). The what-if refusal of infinite params stays, for that reason rather than
the engine's.

Mutation battery (`--no-fail-fast`, each reverted by restoring the saved file): `Leg::new`'s
check alone → 2 of 4 red (its direct test, and the step test only incidentally, as an RK4
over-draw); `with_amount`'s alone → 1 of 4 red (its direct test — the step test cannot see it,
because `Leg::new` refuses first); both → 3 of 4 red, the step test with the NaN pool; the fold
check alone → the lab test red. ⚠ The first draft of the test header said the step test goes
red under *either* removal alone. Only one had been measured — the same overclaim this section
corrects, caught in review before commit.

## Mutation battery (`--no-fail-fast`)

Each guard disabled turned tests red: range check always on → 2 red; range check never on → 15;
ordering rules never on → 6; the finite-params refusal removed → 1; the dead-run marker never
firing → 1. One run first produced no result (the mutation did not compile) and was re-run
rather than read as a pass.
