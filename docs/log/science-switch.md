## **The science switch — slices 0 and 1** (the harness charge's second half, started: the flag gate and the one assembly body)

Plan: `post-roadmap-science-switch.md`. **BUILT 2026-08-31**, the same day it was priced, and
this landing is what expires that doc's index exemption (the rule the reference-flip exemption
established: *an exemption expires when the FIRST slice lands, not the last*). The charge is the
user's of 2026-08-16 — a harness *"that permits easy toggle of parameters and science"*; the
parameter half shipped 2026-08-27 (`docs/log/value-switch-harness.md`), and this is the first
build on the science side.

**Scope, deliberately narrow.** Slice 0 (an assertion owed regardless) and slice 1 (lift the
knockout onto a real seam). The plan's §2C measured that **no alternative form of any biosphere
process exists in the tree**, so the replace/add composers of slices 2–3 would have nothing to
substitute; they wait for a second form, and are not built here.

## Slice 1 — one assembly body, because the control was assembled by the other one

`system.rs`'s test-private `trace_without_flow` — the control the two `root_zone_capture`
diagnostics are differenced against — did not call `build_season_with`. It **re-implemented** it:
its own stock collection, its own carbon loss-sinks, its own flow/aux extend, its own
`State::new` and `Registry::new`. Its docstring justified that (*"no production seam was added
for this"*), and on the day it was written that was true.

⚠ **The failure was dated, not present.** The two bodies agree today and will go on agreeing
until one of them is edited. The day `build_season_with` gains a loss-sink quantity or a state
variable, the copy does not — the control quietly stops controlling for the run it is compared
against, the diagnostic reports a mechanism difference *plus* an assembly divergence with no way
to tell them apart, and **every gate in the repo stays green**.

Built: `domains::lab::mechanism::build_season_without(scenario, &BiosphereParams, &[flow_ids])`
— `build_season_with` for the assembly, then `Registry::into_parts` → filter → `Registry::new`
over the same stocks. The shape is the tree's own precedent, `station::perturbations`'
`with_radiator_failure`. `trace_without_flow` is now four lines over that seam, and both
diagnostics pass **with their assertions untouched**, which is the regression check the lift got
for free.

Two decisions worth recording:

* **`p: &BiosphereParams` is threaded, not loaded.** So the two harness halves *compose* — one
  substituted coefficient *and* one removed process is one call, not a third assembly path — and
  the biosphere keeps its single production param load (`tests/param_funnel.rs`).
* **"Exactly one match" needs no counting.** The lifted helper asserted `dropped == 1`; the
  registry being filtered was already built by `Registry::new`, which rejects duplicate flow ids,
  so a matched id matched exactly one flow by that constructor's contract. A *missed* id is still
  an error — that is the science-side twin of the value harness's §7 failure (patch nothing, read
  the baseline back, report "no effect" as a finding), and it fires on the live case: dropping
  `biosphere.recycling` from the open field, where only a sealed chamber wires it.

## ⚠ The finding: our own instinct for the slice's gate was a round trip

The obvious control — *"no drops reproduces the ordinary build, bit for bit"* — is
`Registry::new(into_parts(Registry::new(…)))`. It holds **by construction**, and it is exactly
the shape this repo has already paid for: *if one side's copy came from the other, the gate is a
round trip*. Worth stating because the value harness's empty-substitution control is **not** the
same shape and reads identically: there, `biosphere_with(&[])` and `params::biosphere()` reach
one object down two independent routes, so it can fail. Here there is no second route.

The control is kept (it is cheap, and would catch a `Registry::new` that was not order-invariant)
and labelled in place as proving less than it looks like it does. The gate with teeth is a
**source scan**, `tests/one_assembly_body.rs`: the three steps only an assembly takes — walk the
compartment builds, add the boundary loss-sinks, close a registry over them — must each happen
exactly once under `src/biosphere/`, inside `build_season_with`'s line range. Re-fork the body
anywhere in the spine and it goes red by file and line.

⚠ **The first draft of that gate had a hole, found by review before it shipped:** it looked for
the literal `Registry::new(`, and `Registry::flows_only` delegates to it — so a fork spelling the
registry step the other way would have passed. Measured at zero calls under the spine, then
closed by **deriving** the spellings from `simcore`'s own `impl Registry` (every `pub fn`
returning `Result<Registry, …>`) and combining the counts. A third constructor tomorrow is
scanned for without editing the gate. *A name is not a claim its arithmetic checks.*

**And the one-directory scope is a proof, not a limitation:** `fn compartments(` is
module-private, so nothing outside `system.rs` can walk the compartment builds at all. The
privacy is pinned by the same test that pins the exclusion, because the scope argument rests on
it.

Three details it earned:

* **`State::new(` is deliberately not a needle.** It appears in fifteen flow unit tests, where a
  two-stock state is the ordinary way to exercise a rate law; forbidding it would forbid unit
  testing. The three chosen needles are what an assembly does and a unit test never does.
* **`#[cfg(test)]` is NOT skipped**, unlike `param_funnel.rs`'s scan. The duplicate body *was*
  test code — a scan that skipped test code would have been green throughout the defect.
* **The gate would also pass if the helper had simply been deleted**, taking both diagnostics
  with it, so a second assertion pins that `trace_without_flow` still exists and goes through the
  seam.

## Slice 0 — a flag can only subtract from the frozen roster

`SeasonScenario` carries five `bool` flags and `compartments` branches on all of them, so they
are mechanism switches sitting in **production scenario config**. They are harmless today for a
contingent reason: `DEFAULT_SCENARIO` sets `vernalization: true` and `stem_reserves: true`, and
the chambers add `sealed` / `consumer`, so every type any flag can wire is ON in one of the four
canonical builds — the same four the freeze manifest's `flow_set` / `aux_set` are unioned from.
Flip either literal and `StemRemobilization` / `NitrogenSenescence` leave that union; the
manifest is **derived**, so it would follow the code silently. Nothing gated that.

`tests/scenario_flag_types.rs` now does: every flag, both values, on all four canonical
scenarios — 40 builds — must produce a type set **inside** the canonical union. The claim is
comparative and names no type, because a roster here would be a second copy of the frozen
manifest (`type_identity.rs` refuses one for the same reason).

Three anti-vacuity halves, since a subset check passes over an empty set:

* the sweep's 40 combinations are counted, and an `Err` build is read as "unreachable" rather
  than silently skipped;
* four of the five flags must **gain** at least one type when switched on — and `photoperiod`
  must gain **zero**. That last one turns the plan's *"four flow-gating flags, not five"* into a
  measurement rather than something inherited from a draft that also miscounted `perennial` as a
  fifth flag. ⚠ `consumer` gains nothing without `sealed` (`build_consumers` returns an empty
  build otherwise), so gains are unioned across the canonical scenarios rather than asserted per
  scenario;
* the toggle roster is read off the `SeasonScenario` **declaration**: a sixth `bool` field
  reddens the test by name instead of quietly escaping the sweep.

⚠ **What the sweep is, said plainly:** one flip from each of four canonical bases — not the
32-point flag space. A type gated by a conjunction two flips from every base would be outside
its reach; none is today, and that is a measurement of this roster rather than a property of the
design.

## What is NOT built, and what is NOT taken

* **Slices 2 and 3** — the replace/add composers, their three gates, and the two constructional
  controls (no-op replacement, scaled replacement). Inert until a second form exists; the plan
  keeps them together for that reason.
* **The first A/B science pair.** The plan's §8 already refuted the tempting candidate: the repo
  holds **no isolated big-leaf-vs-layered number**, because that build moved three things at once.
* **The integrator swap**, out of scope by the plan and by CLAUDE.md's rule against taking a
  science item and a re-anchoring slice in one batch.
* **No science decision.** The harness regenerates evidence; it endorses nothing. The
  `extinction_coef` question is still open and still the user's.

## Verification

`cargo test --workspace --no-fail-fast` green — **1069 passed, 0 failed** —
`cargo clippy --all-targets -D warnings` clean, and the regeneration tool reporting **19 of 19
goldens identical**. That last was expected before it was run rather than hoped for: the seam is
post-assembly and no canonical scenario goes through it.

**A four-mutation battery, because four new gates green prove nothing until they are shown to go
red.** Each reddened exactly its intended assertion, with the message that assertion was written
to print:

* `DEFAULT_SCENARIO.stem_reserves = false` → the union gate (*"`StemRemobilization` … which no
  canonical build carries"*), **and** the defaults pin, both by name;
* the vernalization aux wired unconditionally → the gain control (*"vernalization gates no type
  on any canonical scenario"*);
* a sixth `bool` field on `SeasonScenario` → the roster gate, naming `extra_switch`;
* a second assembly body, written into test code → the source scan, at `system.rs:2086`, quoting
  the offending line;
* **(after the widening)** a registry built by the *other* constructor and nothing else → the
  registry step, listing both derived spellings. This is the mutation the first draft would have
  passed.

⚠ The last one is also the evidence that the scan reads test code: the fork it caught was inside
`#[cfg(test)]`, where the original defect lived.
