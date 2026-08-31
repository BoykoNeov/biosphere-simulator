# The science switch — swapping a MECHANISM, not a value

> ## STATUS 2026-08-31 — SLICES 0, 1, 2 AND 3 BUILT. The pricing below stands; §7 is the queue.
>
> This began as the **pricing pass** the value-switch record left open — the user's charge of
> 2026-08-16 was a harness *"that permits easy toggle of parameters and science"*, and the
> parameter half shipped 2026-08-27 (`docs/log/value-switch-harness.md`). Priced in the
> morning, and the two slices that need neither new science nor a decision were built the same
> day; the record is `docs/log/science-switch.md`.
>
> * **Slice 1 — BUILT.** `domains::lab::mechanism::build_season_without(scenario, p, &[ids])`,
>   and `trace_without_flow`'s duplicate assembly body is gone. `tests/one_assembly_body.rs` is
>   the gate — ⚠ the empty-drop control §7 leaned on turned out to be a **round trip**, so the
>   source scan is the evidence, not it.
> * **Slice 0 — BUILT.** `tests/scenario_flag_types.rs`: no flag setting wires a type outside
>   the canonical union, with the roster derived from the struct declaration. `photoperiod`
>   measured at **zero** gated types, which is §2B's "four, not five" made a measurement.
> * **Slices 2 and 3 — BUILT** (a second batch the same day, on the user's call).
>   `build_season_replacing`, `build_season_adding`, and the general `build_season_composed`
>   behind all three; `ScaledMechanism` as the scaled instrument; `tests/mechanism_switch_run.rs`
>   for the two constructional controls and both run directions;
>   `tests/lab_only_mechanisms.rs` for §6's reachability property. ⚠ **This does not contradict
>   §2C's "the seam and its first pair must land together".** §8's resolution is that the first
>   pair is the pair of *constructional* controls — the no-op and the scaled replacement — and
>   both are available on the frozen tree today. What is still absent is a *scientific* pair.
>   ⚠ And §8's ranking of those two controls was **wrong by omission**: they are not equally
>   strong, and the record says which is which (`docs/log/science-switch.md`).
> * **Slices 3b and 4 — NOT built.** 3b needs a second form of some process **authored**, which
>   is a science decision and the user's. 4 (the report rows) was left deliberately: `report`
>   measures through `readouts::trajectory`, which is shared with the science gates and whose
>   own header names the failure a mechanism swap uniquely causes — a swap can remove a stock's
>   only writer, the series goes empty, and a `min` fold over an empty series returns +infinity,
>   which reads as *"comfortably above the compensation point"*. That is a slice with its own
>   guard, not a row.
>
> No longer forward-looking, so it is **indexed normally** in `docs/post-roadmap-log.md` and
> its exemption paragraph there was deleted in the landing commit — the rule that an exemption
> expires when the FIRST slice lands, applied to itself.

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

**B. Switch a process on or off (a scenario flag).** *Already built — AND FROZEN, but
CONTINGENTLY, which is the part worth writing down.*
`SeasonScenario` carries four flow-gating flags — `sealed`, `consumer`, `stem_reserves`,
`vernalization` — and `compartments` branches on all four (system.rs:304, 419, 436, 542, 658,
672, 745, 786). (*"Perennial" is a run mode, `run_perennial`, not a scenario field; an earlier
draft of this section listed it as a fifth flag and that was wrong.*)
⚠ **This is a finding, not a shortfall: a real part of the user's charge is already
discharged and has been since Phase 1.** And it is discharged *inside* the freeze — but not
structurally. Measured on the four canonical scenarios (system.rs:129–222):

* `DEFAULT_SCENARIO` sets **`vernalization: true` and `stem_reserves: true`**;
* the three chambers inherit it with `..DEFAULT_SCENARIO` and add `sealed: true`, one of them
  `consumer: true`.

So every flow type any flag can wire is ON in at least one canonical build, and the manifest's
`flow_set` — a **union** (§5) — already contains all of them. Turning a flag off can only
subtract from a run, never add a type outside the frozen roster. **The flags are contract
inputs, not experiment knobs.**

⚠⚠ **But that holds because of two literals in a default, and nothing gates it.** Flip
`DEFAULT_SCENARIO.stem_reserves` to `false` and `StemRemobilization` / `NitrogenSenescence`
leave the frozen union — at which point the flag becomes an **unfrozen mechanism switch
sitting in production scenario config**, reachable from `build_season`, and §6's whole property
is void. The union is derived, so the manifest would follow the code silently; that is exactly
the auto-follow failure `locked_dt_days` is hand-written to avoid. **A cheap assertion is owed
here regardless of whether the rest of this plan is built**: every flag-gated flow type is
wired ON by at least one canonical scenario.

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

**Slice 3 — the two constructional controls** (no-op replacement, scaled replacement). No new
science; they are what make slice 2's composers trustworthy. See §8.

**Slice 3b — the first A/B *science* pair.** Needs a second form authored. See §8.

**Slice 0 — one assertion owed regardless** (§2B): every flag-gated flow type is wired ON by at
least one canonical scenario, so the manifest's union genuinely contains them. Two literals in
`DEFAULT_SCENARIO` currently carry that property and nothing checks it. Cheap, independent of
every other slice, and it should not wait for them.

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

⚠⚠ **The obvious candidate for that control does NOT work, and finding out why is this
section's result.** The tempting pick is the retired big-leaf canopy against the shipped
layered (Goudriaan) one: on 2026-08-15 `photosynthesis.canopy_assimilation` stopped being a
big-leaf aggregator and became a depth integral, both forms are cited, and the build measured
plenty. But **the repo holds no isolated big-leaf-vs-layered number**, for two independent
reasons, both in `docs/log/layered-canopy.md`:

* **the build moved three things at once** — the canopy scheme, `specific_leaf_area`
  22.0 → 23.53 (a +7.0 % *calibration*), and a new 5 %/day mutual-shading loss — so the golden
  diff is a three-mechanism delta, not this pair's;
* the numbers that *are* in the record measure something else: the layered scheme against a
  100-layer reference (0.2 %, against the midpoint rule's −5.2 %), the peak LAI, and the
  LINTUL3 gap closing 22 % → 6.4 %. And the predecessor's headline number was **retracted by
  that same build**, because a probe's docstring named a scheme its arithmetic did not
  implement.

**So the pair is a question, not a control** — a perfectly good first *scientific* target, but
recovering an isolated delta is part of the work, not an input to it. ⚠ Taking it as a control
would have meant validating a new seam against a number produced by the one build in this
repo's history that caught its own instrument being self-inconsistent.

**What IS available is a control whose answer is known BY CONSTRUCTION, and that is stronger
than an archived measurement:**

* **the no-op replacement** — replace a flow with a freshly built identical instance; the run
  must be **bit-identical** to `build_season`. This exercises the composer's every step
  (locate by id, remove, insert, rebuild the registry) with a predicted answer that cannot
  drift, and it is the science-side analogue of the value harness's `UNCHANGED` column;
* **the scaled replacement** — wrap one flow in `ScaledFlow` (already written, in
  `station::perturbations`) at a known factor; at 1.0 it must be bit-identical, at 0.5 the
  affected legs must halve exactly on the first step. Arithmetic, not archive.
  ⚠ **CORRECTED 2026-08-31 when it was built: that instrument is not reachable, twice over.**
  `station` depends on `domains`, so the dependency runs the wrong way, and that wrapper reads
  its factor from a **forcing var** — using it would mean adding a forcing to the frozen
  weather resolver before a lab replacement could run at all. The built instrument is
  `lab::mechanism::ScaledMechanism`, a plain constant factor. *An instrument named in a plan
  is a claim about a crate graph, and this one was written from the shape of the code rather
  than from its dependencies.*

Only after both pass does a *scientific* pair mean anything. Candidates, all questions:
big-leaf vs layered per above, the twice-refused soil fractionation (single vs multi-pool
decomposition), nitrogen option D, leaf expansion re-authored in Rust per §3.

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
* The **no-op replacement** is bit-identical and the **scaled replacement** halves the affected
  legs exactly — the two answers known by construction, before any science pair is read.
* Every flag-gated flow type is wired ON by at least one canonical scenario (§2B, slice 0).
* A source scan proving no alternative mechanism is constructed under the biosphere spine.
* A mis-targeted drop/replace **fails**, and is not reported as no-change.
* One assembly body: `trace_without_flow`'s duplicate is gone, not merely deprecated.
* `cargo test --no-fail-fast` green + `cargo clippy --all-targets -D warnings` clean.
* The 19 goldens byte-identical, reported by the regeneration tool at every step.
