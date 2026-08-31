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

`cargo test --workspace --no-fail-fast` green — **1070 passed, 0 failed** —
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

⚠ **One flake, recorded rather than swallowed.** In the middle full-workspace run,
`godot_bridge`'s `the_authored_kinetics_marker_crosses_the_boundary` failed once; it passed alone
and passed again with all 19 of its binary's smokes (218 s) and in a third full run. Those smokes
launch Godot child processes under a wall-clock bound, and the run was loaded. **The failure
message was not captured** — the log was filtered to `test result` lines — so the cause is
unknown rather than diagnosed, and it is written here as an unknown. Nothing in this batch
compiles into that crate: the edits between the clean run and the flake were two `domains` test
binaries and prose.

---

## **The science switch — slices 2 and 3** (the replace/add composers, and the two answers arithmetic fixes in advance)

**BUILT 2026-08-31**, a second batch the same day, on the user's call ("finish the mechanism-swap
harness"). Slices 0 and 1 are above; this is the substitution half.

⚠ **This is not the "seam without a second side" §2C warned about.** That section's rule is *the
seam and its first pair must land together, or the seam proves nothing* — and §8 answers what the
first pair is: **the two constructional controls**, not a science pair. Both are available on the
frozen tree, so the seam lands with them and with nothing scientific claimed.

## What was built

Three named composers over one public body, in `domains::lab::mechanism`:

* `build_season_without(scenario, p, &[ids])` — the knockout, from slice 1;
* `build_season_replacing(scenario, p, vec![(id, flow)])` — one flow out, one in, **same id**;
* `build_season_adding(scenario, p, vec![flow])` — a process the frozen build does not carry;
* `build_season_composed(scenario, p, drops, replacements, additions)` — all three at once, and
  **public on purpose**: the collisions *between* the composers are checkable only here, and a
  guard no caller can reach is a guard no mutation can redden. It is also the shape a real A/B
  pair takes (swap the form, drop the process it makes redundant).

Plus `ScaledMechanism`, a wrapper multiplying every leg by a constant. It scales the *whole*
flow, so the result stays internally balanced; `id`/`priority` delegate (so the wrapper keeps the
wrapped flow's slot in the id-sorted reduction order) and `type_name` does not, which is what
makes a replacement visible in an inventory without running anything.

**A replacement must carry its target's id.** A renaming replacement is refused with an error
saying what to do instead: a rename is a drop plus an add. Two changes wearing one name is how
"the new form of X did this" gets attributed to a difference that was partly a different process
arriving.

## ⚠ The finding: the instrument the plan named does not exist for this caller

§8 said to wrap the target in `station::perturbations::ScaledFlow`, *"already written"*. It is
unreachable, for two independent reasons, and either alone is fatal:

* **the dependency runs the wrong way** — `station` depends on `domains`, so the lab cannot see it;
* **it reads its factor from a forcing var**, so using it would mean adding a forcing to the frozen
  weather resolver before a lab replacement could run at all.

*An instrument named in a plan is a claim about a crate graph.* That one was written from the
shape of the code — the wrapper does exactly the right thing — without checking which side of the
graph it sits on. The plan's §8 now carries the correction beside the original sentence.

⚠ And a third instance of this same replace-a-flow-by-id shape already existed and was found only
while looking for the second: `domains::ulp_probe::nudge_radiator`, for thermal. Three hand-rolled
bodies now (that one, `station::with_radiator_failure`, and this). A shared primitive belongs in
`simcore::registry`; moving one is not this batch, and it is written down here so the next reader
does not have to re-find it.

## ⚠⚠ The two controls are NOT equally strong, and §8 did not say so

This is the batch's main correction, and it came from review before the code was written.

* **The scaled replacement is the evidence.** At 1.0 the run is bit-identical (`x · 1.0 == x`); at
  0.5 the target's legs halve **exactly** (both factors are exact in binary floating point) and
  the run moves.
* **The no-op replacement — replace a flow with a freshly built identical instance — is nearly
  blind.** A composer that locates the target, drops it and quietly keeps the **original** box
  passes it green: the argument never inserted, the run unchanged, and even the registry's type
  names identical either way. It is the *"and nothing else moved"* half, not the evidence.

Both are kept and both are labelled in place, the same treatment slice 1's empty-drop round trip
got. Two cheap additions make the insertion directly observable without a run: after a scaled
replacement the target's slot must report `type_name == "ScaledMechanism"`, and the baseline's
must not.

⚠ **"A freshly built identical instance" has only one honest source**: a *second* ordinary build.
The flows are constructed inside `compartments`, which is module-private to `system.rs` — so
building one by hand would be the second assembly body slice 1 just deleted. Same body, run
twice, which is also part of why this control is the weaker one.

⚠ **The halving control asserts its subject is non-zero.** `maintenance_respiration` was chosen
because standing biomass burns from the first step, and the test asserts a non-zero leg before
comparing halves — `0.0 == 0.5 · 0.0` passes vacuously, which is exactly the failure the ULP
probe logged for weeks when a shimmed function the carbon path no longer called measured 0.0.

## The gate: what a source scan can see that a manifest cannot

§6's property is *every alternative mechanism is reachable only through the lab*.
`tests/lab_only_mechanisms.rs` is the gate, and **the obvious half of it was measured and left
out**: "build the four canonical scenarios and assert no lab type is in the inventory" is
redundant — `freeze_manifest::inventory()` walks exactly those four builds and
`tests/manifest_writer.rs` compares the written manifest byte for byte, so a lab type reaching a
canonical build already reddens that gate. *A redundant guard has no mutation that reddens.*

Two failures survive that reasoning and are what the file gates:

1. **a lab type constructed in spine code on a path no canonical scenario reaches** — behind a
   flag no frozen scenario sets, or in a helper nothing calls yet. Invisible to every run and to
   every manifest; visible only to a scan of the tree;
2. **a lab type wired in *and* the manifest regenerated.** The manifest is *derived*, so it
   follows the code silently — the auto-follow hazard `locked_dt_days` is hand-written to avoid.
   The committed manifests are therefore read as **text** and asserted not to name a lab type.

⚠⚠ **Failure 2 was measured, not argued** (below). It is the one that would otherwise pass the
whole repo.

**The roster is derived from the `type_name` string literal, not from the struct name**, and the
two are asserted to agree. They are different axes: `Flow::type_name` is hand-written on purpose
(*"deliberately not defaulted from `std::any::type_name`"*), so an `impl Flow for AltPhotosynthesis`
returning `"CanopyAssimilation"` — a copy-paste, or a disguise — walks straight past a
struct-name-derived roster wearing a frozen name. A disagreement is itself the finding.

## The battery: five mutations, five reds — and the battery itself was the defect

Run 2026-08-31 from `rust/`, script kept at
`M:\claud_projects\temp\science-switch-mutations\run.sh`. Every mutation reddened its intended
gate **by name**:

**M1 — the composer keeps the *original* box (the argument is never inserted).** Reddens
`a_replacement_takes_its_targets_slot`, `the_general_composer_takes_all_three_at_once` and the
run-level `a_halved_mechanism_moves_the_run_in_the_direction_the_science_says`.

**M2 — the target is dropped and nothing inserted.** Reddens those, plus
`a_no_op_replacement_is_invisible_in_the_inventory` and **both** bit-identity controls.

**M3 — a mis-targeted replacement returns `Ok`.** Reddens
`a_replacement_that_matches_nothing_is_an_error`, and that one alone.

**M4 — `ScaledMechanism` constructed in the spine.** Reddens
`no_lab_mechanism_is_named_under_the_spine`, and that one alone.

**M5 — a lab type reports a frozen type's name.** Reddens
`every_lab_flow_type_reports_its_own_name`, plus the spine and **committed-manifest** scans.

M5 is the measurement behind *failure 2* above: with `InertFlow` renamed to `"Decomposition"`,
`no_committed_manifest_names_a_lab_mechanism` reddens because the frozen manifest really does
name that type. The manifest half is not decoration.

⚠ **A claim above is corrected by M1.** `a_scaled_replacement_halves_its_targets_legs_exactly`
constructs the wrapper and evaluates it **directly** — so it is evidence about the wrapper's
arithmetic, and is immune to a broken composer by construction. It stayed green under M1 and M2,
correctly. The *insertion* is carried by the registry `type_name` check and by the run-level
direction test, both of which M1 does redden. The coverage is complete; the sentence describing
it was not.

## ⚠⚠ The battery destroyed the work it was written to check, and then reported nothing

Both halves are instrument defects, and the second is the transferable one.

**1 — a revert-first battery ran against an uncommitted subject.** The script opened with
`git checkout -- <mechanism.rs> <system.rs>` so each mutation would start clean. The 467 new
lines in `mechanism.rs` had never been committed, so that first line deleted them. The two new
test files survived only because they were untracked. Recovery was a replay of the nine `Edit`
operations out of the session transcript onto the reverted file — each `old_string` matched
exactly once, which is what makes the reconstruction sound — plus one `sed` rename found the same
way; `rustfmt` accounted for the only remaining difference, and the surviving tests (24, all
green) are the independent oracle. The script now snapshots to a temp copy, restores from that,
and **refuses to run at all if either target is dirty in git**.

**2 — the filter that greps for reds cannot tell "no failures" from "no run."** After the revert,
the two test files referenced functions that no longer existed, so nothing compiled. The filter
matched only `test result`, `panicked` and `assertion` lines, and compile errors match none of
those — so the battery printed **five headers and no other output**, which reads as five clean
runs. This is the same family as CLAUDE.md's `--no-fail-fast` warning (*a truncated run reports
fewer reds, which reads as "the new tests are inert"*), and it is worse: there the count is low,
here it is silent. The script now (a) names a build failure explicitly instead of dropping it,
(b) treats a run that emitted **no `test result:` line at all** as a hard instrument error, and
(c) verifies each mutation actually changed the file, since a `sed` that matches nothing runs the
original code and its green means nothing.

**Nothing else moved.** `cargo test --workspace --no-fail-fast` green, `cargo clippy
--all-targets -- -D warnings` clean, and the 19 goldens byte-identical — predicted before running
on the grounds that the seam is post-assembly and no canonical scenario reaches `lab::mechanism`.

⚠ **It bit a third time in the same session, in the verification itself.** The full-suite command
was written as `cargo test --workspace --no-fail-fast | grep … | head -20`, and the workspace has
**60** test binaries — so the stream was cut at 20 and the pipeline's exit code was the `echo`'s,
not cargo's. Counting the result lines caught it; reading them would not have. The rule the two
defects above share, stated once: **a check must report how much it looked at, not only what it
found.**

## Five more mutations, and the finding they produced: `is_err()` is not a gate

The first battery mutated **one** of `build_season_composed`'s five guards (M3, every-target-
matched). Five more were written for the other four, and the run was the finding.

**M7, M9 and M10 each disabled a guard and the whole battery came back with ZERO failures.**
Four of the five guards are redundant with a *later* error: `Registry::new` rejects a duplicate
flow id by itself, and guard 5 catches a target that never matched. So a test asserting only
`is_err()` stays green with the guard deleted — it observes *a* refusal, never *this* refusal.

Guard 4's own comment had already said what those guards are for — the engine would call it
"duplicate flow id", which reads as an engine fault rather than as this composition asking for a
replacement under the wrong name. **They earn their place by the message.** So the message is
what the tests assert now, through `refused_with(result, needle)`, whose docstring carries the
measurement so the next reader does not weaken it back.

⚠ M6 and M8 did not apply at all on the first attempt — a `sed` replacement containing a raw
newline. **The liveness check added an hour earlier caught it and said so**, instead of printing
a clean run. That is the fix from the section above working on its first real occasion.

With the messages asserted, **all ten mutations redden, each on exactly one test.** M6, M8 and M9
land on three *different* assertion lines inside `the_cross_composer_collisions_are_refused`
(651, 669, 660), so they are three distinct guards and not one test answering for all of them.

**The census, stated as a number rather than as "green":** `cargo test --workspace
--no-fail-fast` = **64 test binaries, 1088 passed, 0 failed, exit 0**. ⚠ An earlier capture in
this session showed 60 binaries; that stream had been filtered, so 60 was never a census. 64 is
the number to reproduce.

---

## **The science switch — slice 4** (the mechanism half of the comparison report)

**BUILT 2026-08-31**, a third batch the same day, on the user's call ("slice 4"). Slices 0–3 are
above. This is the reporting half: a knockout or a swap becomes a **column** of the same table
the value harness prints, and the slice's own gates are the three ways that table could lie.

⚠⚠ **The plan's description of this slice was wrong in two places and incomplete in a third, and
the corrections are the substance of the batch.** The code is small; what it cost was finding out
that "more rows, not a new renderer" is false three times over.

## What was built

* `readouts::trajectory_composed` / `try_trajectory_composed` — the trajectory against a
  caller-supplied **build**. `trajectory` is now that function at the frozen build, not its
  sibling, so the composed run goes through the *same* observer body: the stock sampling, the
  empty-series note and the `steps + 1` count assertion have one copy, not two. This is
  `one_assembly_body.rs`'s argument applied one layer up, before the second body existed.
* `biosphere::season_setup_composed` beneath it, with `season_setup_with` delegating. It takes a
  **build**, not a built pair: a caller handing in `(State, Registry)` could hand in one
  assembled from a different scenario than the resolver is bound to, and nothing could tell.
* `lab::mechanism::Composition` — a mechanism change held as a *request* (drops, replacements,
  additions) so it can be asked of several scenarios, plus `absent_targets` to ask **before**
  running whether a scenario can answer it at all.
* `lab::report::Change` (`Values` | `Mechanism`), `measure_composed`, `compare_changes`, and
  three new cell states in the renderer: not applicable, dead, constant-series.
* `examples/science_switch.rs` — `science_switch <flow.id> [...] [--long]`, the knockout as one
  command. Only the knockout is expressible from a command line: the other two composers take a
  *flow*, and there is still no second form of any biosphere process in this tree.
* The shared renderer stopped calling itself *"value-switch report"*. A caption is quoted later.

## ⚠⚠ Finding 1: the hazard this slice was written around cannot happen

The plan named it precisely: *a swap can remove a stock's only writer, the series goes empty, and
a `min` fold over an empty series returns +infinity, which reads as "comfortably above the
compensation point"*. **Measured: unreachable.** A composition rewrites the flow list; stock
presence is decided by `build_season_with`'s compartments, and the observer gates on
`s.stocks.get(CARBON_POOL)`. So every series under a composition is exactly as long as the frozen
run's. `min_ppm` already asserts non-emptiness, with a test behind it since the readouts moved.

**The reachable form is a *constant* series, and it is worse.** Remove a stock's only writer and
the fold returns the run's starting value — finite, plausible, comfortably above the floor, and
attached to a run where nothing happened. `+infinity` is conspicuous; 71.4 ppm is not.

⚠ *A named hazard is not the same as a measured one.* This one was written into a plan, survived
a design review, and was false about its own mechanism the whole time — while pointing at a real
defect one step to the left.

## ⚠⚠ Finding 2: the frozen scenarios do not share a flow set, and that breaks the renderer

**Ten of the twenty-three biosphere flows are in all four canonical builds.** The other thirteen
are scenario-specific — and they are where the interesting science lives: decomposition,
humification, microbial respiration, the three nitrogen releases, grazing, condensation,
irrigation. Measured, not assumed.

So "swap the soil carbon scheme" is an ordinary request that **cannot be asked of the open
field** — and the renderer's `(Some, Some) => …, _ => continue` dropped that cell with no marker,
while the movement counter's `_ => {}` shrank the count. A column reading *"1 rose, 0 fell"* over
a table where four of five rows were never measured. That is
`every_spec_names_a_scenario_that_is_actually_run`'s own failure — a claim quietly not measured —
arriving through a door that test cannot see, and it was **safe until this slice** only because a
param substitution applies to every scenario by construction.

⚠ The alternative was to refuse a variant that does not apply uniformly, which is simpler. The
measurement is what ruled it out: with 13 of 23 flows scenario-specific, refusing would have made
the harness useless for most of the swaps worth running. **The design question was decided by one
two-minute measurement, not by argument.**

## ⚠⚠ Finding 3: a knockout ENDS the run, and that is the ordinary case

Neither the plan nor the design review predicted this. It appeared on the first mechanism column
anyone would think to run.

Drop `biosphere.root_zone_capture` (root water uptake) and the crop never stores enough carbon to
re-sow, so **both perennial chambers raise at the annual reset** — `annual_reset: seed bank too
small to re-sow — storage_c 0.0115 < seedling 0.16`. Drop `biosphere.decomposition` and the same
two die for the same reason. Two of the first two knockouts tried; this is not an edge case, it
is what knocking out a load-bearing process *does*.

Before it was handled the whole report **panicked from inside `readouts`**, four levels below the
caller, because `trajectory` was written for the frozen path where a run cannot fail.

**It is a result, and arguably the strongest one a knockout can produce** — "this chamber cannot
close its cycle without this process" is a bigger statement than any number in the table. So it
is printed, in the engine's own words, in its own cell state.

⚠ **And it is kept distinct from finding 2's cell**, which is the part worth remembering: *"this
scenario has no such process"* says nothing about the science, *"this scenario dies without it"*
says a great deal. One marker for both would have merged a fact about the roster with a result.
`TrajectoryError` splits the two at the source — `Setup` is a bad **request**, wrong under every
scenario, and still stops the whole comparison; `Run` is this scenario's answer.

## The guard the slice actually owed

`ReadoutSpec` now carries the **series its fold reads** as data beside the fold, and a column
flags any readout whose every input series never moved. Data rather than a loop that re-derives
the pairing, for the reason the scenario pairing is data: two copies, one stale.

Two-direction, because a flag that fires everywhere proves nothing: the frozen baseline has **no**
constant series (asserted, so the guard cannot be firing green and later weakened to silence it),
and a composition zeroing all three writers of leaf carbon flags `peak LAI` **and not** the
chamber CO₂ row, whose writers that composition leaves alone.

⚠ **Three flows, not one, and that is a measurement.** Every stock these readouts fold has at
least two writers on the frozen tree, so no single swap can freeze one. `build_season_composed`
takes several changes at once precisely because that is the shape a real pair takes.

## The science this incidentally produced, and did not interpret

Dropping soil decomposition **raises** the sealed chamber's season-low CO₂ by 21.0 %
(71.44 → 86.43 ppm). That is the opposite sign to the naive reading — decomposition puts carbon
into the chamber air — and it is exactly the kind of number this harness exists to generate and
**not** to explain. No cause is asserted here; this repo has logged what an asserted attribution
costs (`asserted-attributions-rot`). It is written down as a measurement with a knockout beside
it, and the explaining is a separate piece of work with its own control.

## Verification

* **`cargo test --workspace --no-fail-fast` = 64 test binaries, 1095 passed, 0 failed, exit 0.**
  The previous batch recorded 64 / 1088; slice 4 adds exactly its seven new tests and moves
  nothing else. The `readouts` refactor is the one place this slice could have moved a frozen
  number — it is shared with every science gate — so it was **measured**, not argued.
* **19 of 19 goldens byte-identical**, reported by `regen_goldens` (report mode). No param file,
  manifest, science band or golden touched; `git status` clean outside this batch's own files.
* `cargo clippy --all-targets -- -D warnings` clean.

⚠ **One instrument note, and it is the same shape as the last batch's "60 was never a census".**
The first attempt at this census was captured through a background stream and came back as
**12 binaries, 40 passed** — a number that would have read as a green run to anything grepping
only for failures. It was a truncated capture, not a truncated run. *A census that disagrees
with the last recorded one by a factor of twenty-five is an instrument reading, not a result*;
this one was re-run writing to a file of its own and reproduces 64 exactly.

## The mutation battery — and the instrument was the first finding

Eleven mutations, each a one-line edit to `rust/crates/domains/src/lab/{report,mechanism}.rs` or
`biosphere/readouts.rs`, run one at a time against `cargo test --no-fail-fast -p domains` and
restored with `git checkout --`. Driver: `M:\claud_projects\temp\science-switch-slice4\battery.py`.

**The first run reported ten of eleven as "DID NOT COMPILE" and nothing as red.** That was the
classifier, not the tree. It decided compilation with `^error(\[|:)`, and cargo prints
`error: test failed, to rerun pass ...` once per test binary that *went red* — so the signature of
a working guard was being read as a broken build. Seven genuine reds were thrown away by a regex.

⚠ **The tell was available and I did not use it: a battery in which nothing reddens has almost
certainly measured itself.** The battery now says so out loud — it appends
`*** INSTRUMENT: not one mutation classified RED — read this as a broken classifier until proven
otherwise ***` when a whole run finds no red — and classifies compilation on `error[E####]` /
`could not compile` alone. This is the *third* instrument failure in slice 4 (after the truncated
census and the heredoc that ate its own backslashes); all three shared one shape: a reading that
disagreed with the last recorded one, accepted instead of questioned.

Re-classified from the logs already on disk — no re-run needed for the eight that had reported —
the true first tally was **7 red, 1 that genuinely did not compile, 3 with no red**.

### The one that never ran

**M5** (skip the applicability pre-check) was written as `Some(_) => Vec::new(),`, which leaves
the element type uninferable: `error[E0282]`. The mutation had therefore measured **nothing** in
any earlier run, while reading in the summary exactly like a red-free result. With
`Vec::<String>::new()` it compiles and reddens two tests.

### The two real holes, and what closed them

**M10 — a replacement's target was never checked for applicability.** Chaining
`std::iter::empty()` instead of the replacement ids left the suite green, because every
applicability test in the slice used a *drop*. Closed by
`a_replacement_that_does_not_reach_a_scenario_is_marked_not_applicable`: it replaces
`biosphere.decomposition` (absent from `open_season`, present in both chambers) so one column
carries a `NOT APPLICABLE` marking **and** a live measurement at once, and asserts the specific
marking rather than `is_ok()`/`is_err()` — the same failure the slice-0–4 record already names.

**M11 — `any` in place of `all` in the frozen-readout flag.** The only constancy test froze all
three of `peak W`'s organ series together, so it could not tell the two apart. The predicate is
now a named function, `readout_is_frozen`, precisely so the rule has a subject a test can point
at, and `a_readout_is_frozen_only_when_every_series_it_folds_is` pins the mixed case over a
constructed trajectory (one organ frozen, two moving → **not** flagged).

⚠ That test is evidence about the **rule**, not about a run, and it says so: no composition on
this tree is known to freeze one organ and not its siblings. The anti-vacuity partner
(`the_frozen_baselines_organ_series_each_move`) checks the other end — each of the three organ
series moves in the frozen baseline, so `all` is not quietly riding on a permanent degeneracy.

### The one that stays open, and is not a gap

**M2** — treating an *empty* series as constant — does not redden, and no test was written for
it. It is **unreachable, not untested**: a composition rewrites the flow list, but stock presence
and series length come from `build_season_with`'s compartments, so no composition can produce an
empty series. `None => false` stays as a documented dead arm. Inventing a test here would have
rebuilt the defect the slice-2/3 record already names — a test immune by construction to the
thing it claims to check.

### After the fixes

`M5`, `M10`, `M11` all red on re-run, so the battery stands at **10 red of 11, one unreachable**.

⚠ **That tally was a composite of two trees, and a composite is not a measurement.** The seven
original reds were measured *before* the predicate was extracted, the call site changed and the
three tests added; the re-run covered only the three. Written as "eleven mutations, ten red", it
read as one run of the shipped tree and was not — the same shape as everything else this slice
found. So the whole battery was re-run with no filter against the committed tree (`full.log`):
**M1 and M3–M11 red, M2 the only no-red**, tree clean after restore, and the sentence is now true
of the code that shipped. *Re-running eight already-green mutations cost eight minutes; carrying
an unverified composite in the record would have cost the record's credibility.*

One thing the whole run adds that the composite could not: M1 (the constancy check never fires)
now reddens **both** frozen-readout tests, and M9 (the composed build ignored, the frozen one run
instead) reddens five — the report's guards overlap, and that overlap is only visible when every
mutation is measured against the same tree.

⚠ And the third answer to "why did this not redden", which the earlier write-up left out: not
only *uncovered* or *unreachable*, but **the mutation never applied**. The script asserts its
needle matched and prints `DID NOT APPLY … instrument failure, not a pass` when it does not —
checked here rather than assumed, because M11's needle had already gone stale under this slice's
own refactor. Rule that one out first; it is the only one that is a fact about the instrument.

`cargo test --workspace --no-fail-fast` = 64 binaries, **1098 passed**, 0 failed
(1095 before; exactly the three new tests). `cargo clippy --all-targets -- -D warnings` clean.
Working tree clean after restore, checked by the battery itself.
