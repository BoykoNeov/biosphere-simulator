## **The value-switch harness** (the user's named successor to the reference flip, taken the day it finished)

Plan: `post-roadmap-value-switch-harness.md`. **BUILT 2026-08-27**, in five commits, on the day
the reference flip finished — which is the event the user attached it to on 2026-08-16:
*"after the switch to Rust, work will continue on the universal harness, that permits easy
toggle of parameters and science."*

## ⚠⚠ The plan's own status block was FALSE the day the work started

It read *"THE SEAM IS BUILT; THE HARNESS IS NOT"*, describing `src/config/overrides.py` and its
19 tests — deleted with the other 271 Python files by S6 (`4f7168e`) and never noticed. So the
remaining scope was **both** halves in Rust, not the reporting layer alone. Corrected in place
rather than in a session note, because a plan doc that survives its own tree is how a stale
record does its damage.

## ⚠ The premise of the plan's design section did not survive either, and what replaced it is better

All 23 frozen param YAMLs are `include_str!`-ed in at compile time, so Python's route — hooking
a runtime YAML load — **does not exist**. Options A/B/C were not stale, their premise was gone.

What the Rust tree offers instead is stronger: 17 `pub fn <loader>_from(text, name)` entry
points beside the 17 zero-argument loaders, the *same* reader, unit guard, bounds and folds. So
a substitution is **modified YAML text**, validated on the way in. The deleted Python harness
used `dataclasses.replace` on the constructed object and bypassed the schema, the units and the
bounds; an out-of-range experimental value ran silently. Here it cannot.

⚠ A consequence stated rather than discovered: a substitution outside a frozen bound **panics**,
exactly as a committed one would. That is the guard working — a value the bound rejects is an
unfreeze request, not an experiment.

## What was built

* **The seam.** `build_season_with(scenario, &BiosphereParams)`, with `build_season` delegating.
  Measured first: `build_season` held the **only** production param load in the biosphere (every
  other call site is `#[cfg(test)]`, checked by line number) and `compartments` already threads
  the struct all the way down. Predicted byte-neutral before running; 19/19 goldens identical.
* **The substitution.** `config::with_override` rewrites one entry's value in a file's *text*;
  `domains::lab::biosphere_with` hands it back through the ordinary loaders. Nothing is written
  to disk, so no manifest digest can move.
* **The lifted fixture.** `biosphere/readouts.rs` — the gates' `Trajectory` and folds, moved out
  of `#[cfg(test)]` so a non-test binary can obtain them. **The fixture moved; the census did
  not.** Not one gate declaration changed.
* **The report + the command.** `domains::lab::report` and `examples/value_switch.rs`.

## The gates, and what each is FOR

* **`tests/param_funnel.rs` — the funnel is one.** ⚠ The obvious guard ("assert an override
  changes the output") is **inert by construction** with a single funnel: it passes for a reason
  unrelated to what it claims. The property that can actually rot is the funnel being singular,
  so that is what is gated, by source scan — the only instrument that can see it, for the same
  reason `biosphere_spine_purity.rs` gives.
* **`tests/value_switch_run.rs` — both directions**, because either alone passes for the wrong
  reason: unsubstituted is bit-identical to the frozen entry point, substituted differs, and the
  frozen run sits strictly between a lower and a higher value.
* **`readouts::tests`** — a fold reads *the trajectory's own* params.

## ⚠⚠ Two findings, and both are about a check being blind rather than a defect shipping

**1. An end-to-end test is not a test of the mechanism.** Mutating a fold to read the frozen
params instead of the trajectory's left `value_switch_run.rs` **GREEN**: it substitutes the
canopy coefficient, the mutated fold reads specific leaf area. A behavioural A/B test sees this
class of defect only when the field it happens to substitute is one the fold happens to read —
a coincidence, not a design. Only the funnel gate caught it, and only because that mutation
happened to add a production param load. The property is now asserted directly.

**2. The cheap report gave a wrong reading, by omission.** Without the 15-year rows the table
said *"5 rose, 0 fell"* — a clean one-directional improvement. Every short row informs
`science_bands`, because the roster's one `liveness_floors` quantity is a 15-year one, so the
**opposed movement** the report exists to surface was invisible. It now prints *"NO
liveness_floors ROW IS IN THIS TABLE … opposed movement CANNOT be read from it"* whenever the
rendered set carries one authority. With `--long` the same run reads **6 rose, 1 fell —
OPPOSED**.

Also found by its own gate on the first run: the science-gate macro's **invocation** carries no
`#[cfg(test)]` — the attribute is inside the macro definition — so a gate body read as
production. Excluded as an *assertion*, not a list entry.

## The result, and it validates against the probes it replaces

The long table reproduces the 2026-08-15 session's hand-measured readings exactly: the perennial
fixed point moves 0.578137 → 0.552202 at `k = 0.65`, **2.285× → 2.183×** the 0.253 degenerate
baseline (three throwaway scripts then quoted 2.29× → 2.18×), and the clearance against the 0.55
bound moves 5.12 % → 0.40 %.

⚠ **A result recorded and not acted on:** peak LAI is **non-monotone** in the coefficient —
`0.68` gives a *lower* peak than `0.65` while every other quantity keeps rising, which the
uniform-amplifier framing does not predict. It argues for no value and changes nothing.

⚠ Any run including the frozen value carries a free self-check: that column prints `UNCHANGED`
on every row, so a drifted substitution path would show itself.

## What is NOT built, and what is NOT taken

* **The sibling domains and the station.** Their `*_from` loaders are private and their params
  load at ~15 scattered production sites with no funnel. A biosphere-shaped seam does not
  generalise, and a harness covering half while presenting itself as universal is the failure
  mode it was built to avoid.
* **The `extinction_coef` decision.** Open, and the user's. The file still reads 0.6.
* **The "toggle the SCIENCE" half** of the user's charge. Values toggle; swapping a *mechanism*
  is a different seam and has not been priced.

Verification throughout: `cargo test --no-fail-fast` green, `cargo clippy --all-targets -D
warnings` clean, the regeneration tool reporting **19 of 19 goldens identical** at every step.
