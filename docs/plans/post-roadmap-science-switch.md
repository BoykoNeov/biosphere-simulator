# The science switch — swapping a MECHANISM, not a value

> ## STATUS 2026-08-31 — PLANNED, NOTHING BUILT. `git diff rust/crates/` is empty.
>
> This is the **pricing pass** the value-switch record left open, and pricing is the
> deliverable: the user's charge of 2026-08-16 was a harness *"that permits easy toggle of
> parameters and science"*. The parameter half shipped 2026-08-27 (`docs/log/value-switch-harness.md`);
> that record's closing section lists **"the toggle-the-SCIENCE half … a different seam and
> has not been priced"** as what was not taken. This doc prices it.
>
> Forward-looking, so it carries no file in `docs/log/` and is **exempt from the log index**
> per the paragraph in `docs/post-roadmap-log.md`. ⚠ A record file must call it *"the
> science-switch plan"* and never name it by filename while the exemption stands — naming it
> there puts it on the record side of the parity check and turns that check red. ⚠ The
> exemption expires when the **first** slice lands, not the last (the rule the reference-flip
> exemption established after one day).

---

## 1. Why this is not simply "the same harness, for code"

The value seam is **pre-assembly**: `build_season_with(scenario, &BiosphereParams)` takes a
different set of numbers and builds the same machine. Every guard that makes it safe — the
schema, the exact-string unit check, the frozen bounds, the boundary folds — is a guard on
*numbers arriving through a reader*.

A mechanism has no reader. Swapping one is swapping **code**, so none of those four guards
apply, and the safety property has to be built somewhere else. §6 says where.

## 2. What "a mechanism" means here — four things, four prices, MEASURED

The word covers four distinct changes in this tree. They were separated by measurement
before any design, because three of them are already partly built and one is forbidden in a
science batch.

**A. Remove one process (a knockout).** *Exists, test-private.*
`rust/crates/domains/src/biosphere/system.rs:2117`, `trace_without_flow(scenario, drop_id)`
— builds the compartments, drops exactly one flow by id (asserting it was there), runs. Its
own docstring records why the crude control was rejected: zeroing a parameter to remove a
process changed *two* things once the soil geometry re-basing gave `EXTR` a second reader,
so *"a control that changes more than it claims is worse than no control."*

**B. Switch a process on or off (a scenario flag).** *Already built — AND FROZEN.*
`SeasonScenario` carries `sealed`, `stem_reserves`, `vernalization`, `consumer`, `perennial`,
and `compartments` branches on all five (system.rs:304, 419, 436, 542, 658, 672, 745, 786).
⚠ **This is a finding, not a shortfall: a real part of the user's charge is already
discharged and has been since Phase 1.** But it is discharged *inside the freeze* — the
manifest's `flow_set` is the union over the four canonical builds (§5), so those five flags
are contract inputs, not experiment knobs. They are how the frozen roster is *defined*.

**C. Replace form A with form B.** *The actual subject — and it has NO second side today.*
Measured: no alternative form of any biosphere process exists anywhere in
`rust/crates/domains/src/biosphere/`. There is one implementation of photosynthesis, one of
transpiration, one water-stress curve per stress. ⚠⚠ **So a substitution seam built now
would have nothing to substitute.** That is §7 of the value-switch plan one level up — the
shim-a-dead-path failure — and it is the same shape as that batch's own finding that a
funnel gate asserting "an override changes the output" is *inert by construction* when
there is one funnel. **The seam and its first pair must land together, or the seam proves
nothing.**

**D. Swap the integrator (Euler ↔ RK4).** *Out of scope, deliberately.*
Both exist (`simcore::integrator::{EulerIntegrator, Rk4Integrator}`). Excluded for two
independent reasons: CLAUDE.md's rule against taking a science item and a re-anchoring slice
in one batch, and the recorded finding that under RK4 the `water_biting` scenario converges
to a **qualitatively different** answer (the crop dies), stable under 8× refinement — which
makes an integrator swap a science question about a shipped golden, not a harness feature.
It is a legitimate later target; it is not this one.

## 3. ⚠ The parked leaf branch is NOT a second side — MEASURED, and this corrects the record

`docs/log/leaf-remeasurement.md:96` keeps the worktree `M:\claud_projects\temp\leaf-worktree`
and the branch `leaf-expansion-rebase` alive *"so a decision need not re-do the work"*. Both
still exist (`git worktree list` confirms, at `b865291`). But:

```
git diff main...leaf-expansion-rebase --stat
  src/domains/biosphere/leaf_area.py       | 392 +++++
  src/domains/biosphere/loader.py          | 158 ++-
  src/domains/biosphere/params/leaf_area.yaml | 121 +++
  … 9 files, 847 insertions — every one of them PYTHON
```

S6 deleted that tree on 2026-08-27. **The branch is a design record, not runnable code**, and
any use of it is a re-authoring in Rust priced as new work. The worktree is warm; the code is
dead. Stated here because "the decision is still cheap" was true when it was written and has
not been true since S6, and nothing was watching.

## 4. The seam — and the tree already contains its precedent

`rust/crates/domains/src/biosphere/perturbations.rs` (Phase 8, P8.5) defines exactly the
shape this needs, for a different purpose:

> *"A perturbation is a **scenario-layer intervention composed onto the already-assembled
> `(state, registry, resolver)`**, never a core/domain change."*

It adds a flow that no baseline build carries (`LeakFlow`, venting to a harness-local
`boundary.leak_sink`), it is deliberately excluded from every golden, and its insurance is
determinism rather than a pin. `station::perturbations` extends the same pattern with
`ScaledFlow` and five `with_*` composers.

**So the science seam is post-assembly composition on the registry**, and it is the mirror
image of the value seam:

| | value switch (BUILT) | science switch (this plan) |
|---|---|---|
| when | **before** assembly | **after** assembly |
| what moves | one entry of one param file's text | one `Box<dyn Flow>` in the built list |
| guarded by | schema + units + bounds + folds | §6's reachability gate — nothing else applies |
| precedent | `build_season_with` | `perturbations.rs`, `trace_without_flow` |

Three composers cover A, C and the diagnostic middle: **drop** one flow by id, **replace**
one flow by id with another, **add** one. `trace_without_flow` is the drop, already written
and in the wrong place.

## 5. The freeze question, ANSWERED BEFORE the design rather than during it

Both manifest inputs are **derived from the four canonical builds through `build_season`**,
with no roster anywhere:

* `freeze_manifest.rs:282` — `inventory()` walks `build_season(&scenario)` for the open
  field and the three chambers, unioning `Flow::type_name()` / `AuxProcess::type_name()`.
* `tests/type_identity.rs` walks the same four, and says in its own header why it holds no
  list: *"a second copy of the frozen manifest in the tree, and this repo has already paid
  for a rule whose two copies disagreed."*

**Therefore a flow type reachable only through the lab moves neither manifest** — the same
standing `LeakFlow` already has, and Phase 3's "diagnostics, no golden" precedent. A science
switch is manifest-free for the same structural reason the value switch was digest-free.

⚠ **The corollary is the risk, and it points at the gate this work owes.** Manifest-invisible
means the completeness half — the one that catches "added but exercised by nothing" —
**cannot see an alternative mechanism at all**. Nothing in the contract would notice an
alternative form leaking into the frozen path. So the gate is not the manifest; it is §6.

## 6. The safety property — the analogue of the value plan's §3

The value harness's property was *"it writes nothing, so no digest can move."* The science
harness's property is different in kind:

> **Every alternative mechanism is reachable ONLY through the lab. `build_season` can
> produce the frozen set and nothing else.**

The failure this prevents is a golden silently running a non-frozen mechanism set — the
science equivalent of a calibration wearing an experiment's name. Three gates, each aimed at
a failure the other two cannot see:

1. **A source scan** that no alternative-mechanism type is constructed anywhere under the
   biosphere spine — the instrument `tests/param_funnel.rs` and
   `tests/biosphere_spine_purity.rs` already use, for the same reason (it is the only
   instrument that can see a property of the *tree* rather than of a *run*).
2. **A both-directions run test**, per `tests/value_switch_run.rs`: an unswapped lab run is
   **bit-identical** to `build_season`, and a swapped one differs. Either half alone passes
   for the wrong reason.
3. **A mis-targeted swap is an error, never a quiet baseline** — dropping or replacing a
   flow id that is not in this scenario's registry fails loudly. `trace_without_flow`
   already asserts exactly this (`dropped == 1`); it must survive the lift.

## 7. Slices — what is buildable now, and what needs science authored first

**Slice 1 — lift the knockout into the lab. Buildable today; no new science; fixes a live
defect.** `trace_without_flow` does not call `build_season_with` — it **re-implements** its
body: stock collection, the carbon loss-sinks, the flow/aux extend, `State::new` with the
three state vars, `Registry::new`. Its docstring justifies this (*"no production seam was
added for this"*), and at the time that was true. It is the same shape as the last batch's
fixture lift, with the failure not yet realised: **if `build_season_with` gains a loss-sink
quantity or a state variable, the knockout control silently stops controlling against the run
it is compared against**, and every gate stays green. One assembly body, composed on
afterwards, delivers mechanism-removal as a real harness capability in the same move.

**Slice 2 — the replace/add composers + the three gates of §6.** Buildable, but §2C says it
is inert without slice 3, so the two land together.

**Slice 3 — the first A/B pair.** Needs science authored. See §8.

**Slice 4 — the report.** `lab::report` already renders a baseline-vs-variants table with the
two requirements the last batch earned the hard way (a fold reads *the trajectory's own*
params; a table that cannot show opposed movement must say so). Mechanism variants are more
rows, not a new renderer — **provided** the readouts stay identical, which a mechanism swap
does not guarantee the way a value swap does.

## 8. The first pair — the recommendation, and why it is a control rather than a question

The obvious instinct is to pick the most interesting open science question. That is the wrong
first target. **The first swap should be one whose answer is already measured**, because a
seam reporting a number nobody can check is exactly the failure §6 exists to prevent — and
this project has already logged a probe whose *name* and *arithmetic* disagreed three lines
apart, with the number travelling into a plan.

**Recommended: the big-leaf canopy vs the layered (Goudriaan) canopy.** On 2026-08-15
`photosynthesis.canopy_assimilation` stopped being a big-leaf aggregator (one call at
layer-mean light × intercepted fraction) and became a depth integral with Goudriaan
quadrature; the predecessor is gone from the tree, both forms are cited, and **the
consequence was measured at the time** (`docs/log/layered-canopy.md`). Re-authoring the
retired form as a lab-only alternative is small, and the seam's first output can be checked
against a number the repo already holds. ⚠ It also re-tests a finding worth re-testing: that
build retracted its own predecessor's headline number.

Other candidates, all of which are *questions* rather than controls, and any of which can
follow: the twice-refused soil fractionation (single vs multi-pool decomposition), nitrogen
option D, leaf expansion re-authored in Rust per §3.

## 9. Scope

**In:** the three composers on the assembled registry; the lift of the knockout; §6's three
gates; the first A/B pair per §8; the report rows.

**Out:** any change to a frozen flow, param, golden, manifest or science band; the five
scenario flags of §2B (they are contract inputs); the integrator swap of §2D; a central
register (the value plan's §2 — still the wrong build, still
`memory/context-budget-relocation-is-not-a-discipline`); and **taking any science decision**.
The harness regenerates evidence; it endorses nothing.

## 10. Exit criteria

* `git status` clean outside the harness's own files after a full run — no param YAML, no
  golden, no committed manifest, no science-gate bound moved.
* An unswapped lab run is **bit-identical** to `build_season`, asserted, both directions.
* A source scan proving no alternative mechanism is constructed under the biosphere spine.
* A mis-targeted drop/replace **fails**, and is not reported as no-change.
* One assembly body: `trace_without_flow`'s duplicate is gone, not merely deprecated.
* `cargo test --no-fail-fast` green + `cargo clippy --all-targets -D warnings` clean.
* The 19 goldens byte-identical, reported by the regeneration tool at every step.
