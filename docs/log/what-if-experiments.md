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

## A finding on the way, not acted on

The engine's per-step conservation check compares `residual.abs() > tol`, and **a NaN residual
passes it silently** (every comparison with NaN is false). The engine core is frozen, so it was
not changed; the lab now marks non-finite runs dead so a what-if cannot report a NaN as a result.
Whether the core should reject NaN is an unfreeze question for the user.

## Mutation battery (`--no-fail-fast`)

Each guard disabled turned tests red: range check always on → 2 red; range check never on → 15;
ordering rules never on → 6; the finite-params refusal removed → 1; the dead-run marker never
firing → 1. One run first produced no result (the mutation did not compile) and was re-run
rather than read as a pass.
