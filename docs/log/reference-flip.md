## **The reference flip — Rust becomes canonical** (target state B → C; eleven slices, eight landed, then C1, C2, C5, C8, C9, C3, C6 and C4 of the C re-plan — the param load, the twelve laws, the drift folds, the param-file list and the weather path all moved into the reference 2026-08-17, the posture itself into CLAUDE.md 2026-08-18, the four Python-only scenarios retired the same day, the science-gate census — half the biosphere manifest — re-anchored the same day, and C4b then C7's station half landed the same day — no Python program writes a frozen contract any more; Stage 3's classification pass then measured the whole 2,452-test Python suite against the reference's 445 and found the flip's remaining hole is not the science tests but the ground under them — the reference compiles 24 files out of the tree being deleted, nothing in Rust compares a run to a golden, and for the four sibling domains the cross-port checker is the only gate there is — S1 then moved all of that data into `rust/` in two halves the same day, so `cargo build` and every `cargo test` binary now pass with `src/` and `tests/` renamed away)

Plan: `docs/plans/post-roadmap-reference-flip.md`. **Planned 2026-08-16 in eleven
independently-landable slices**, on the user's explicit instruction (*"only plan now, work in
different slices. Don't bundle the whole work into one slice"*). **Slices 1–7 landed the same
day.** The reference has moved: two goldens are Rust's bytes, the cross-port contract is
inverted, and **two of the three freeze manifests are re-anchored** — each with **mixed**
authority, stated per key in the file itself. `git diff src/` stays empty throughout.

### The decision

The user re-opened *"the complete switch to Rust"* and chose **target state B** of the
Rust-primary pivot (`post-roadmap-rust-primary-pivot.md` §3) — flip the reference so **Rust
becomes canonical and Python becomes the checker**. B was the option *rejected* in July. Its
cost was priced and stated before the choice; the user took it anyway, and that cost is
recorded **once**, in the plan's §4, as a documented consequence rather than an argument to
be re-run at each slice: **the two ports stop being independent**, so Python is fitted to Rust
rather than checked against it. That mechanism was still paying out four days before the
decision — the native mirror caught a scenario constant the whole Python suite was blind to.

### Why the question came back — the July policy was measured *inert*, not wrong

Option A split work by *type* (new content Rust-first, validated science Python-canonical) on
the estimate that *"~90 % of remaining work is content + gameplay"*. Measured over the 157
commits since: **0 touched `godot/`**, 1 touched `scenarios/`, and everything else was
science. **Rule 1 never once fired**, because no work of the kind it governed was done. The
mix was not 90/10, it was about 0/100.

⚠ **The lesson is about how to judge a policy, not about Rust:** A was not wrong, it was a
policy with opinions only about work that was not happening. Check whether a policy has
*engaged* before concluding it failed — and the felt cost the user was reporting (building
every science item twice) was real precisely because A's rules never addressed it.

⚠ A related miss, recorded because it explains why the posture never became a default: the A
plan called landing its rules in `CLAUDE.md` *"the single most important step"* — and **it was
never done**. The posture lived only in a plan doc and a memory file. Slice 11 closes this.

### Two of my three headline costs were RETRACTED by measurement

Both by reading the file rather than reasoning about the language.

1. ⚠ *"Regenerating the goldens from Rust will disturb the science gates."* **False.** The
   measured cross-port bands are **`1e-11` to `1e-12` relative**; the tightest science gate in
   the tree is **`4e-3`**. Eight orders of margin. **Numerically the flip is nearly free — the
   entire cost of B is structural.** This is the most decision-relevant fact in the plan and
   it points *toward* B.
2. ⚠ *"The completeness gates can't port — Rust has no introspection."* **False.** They do not
   introspect a module namespace: the helper **builds the canonical registries and reads
   them**, which Rust does identically. Measured across all three manifests: **23 derived
   assertions vs exactly 2 deliberately hard-coded literals** (both in the biosphere gate,
   both step-constant-adjacent, both flagged *do not simplify* in place). A mechanical rewrite
   with two judgement calls.

⚠ I then over-corrected the second one to *"ports cheaply"* on the strength of a **single**
helper, and the advisor caught it. The honest position is the measured one above.

**One cost CONFIRMED and larger than the old plan said:** the crop-model comparison. The
arithmetic half is pure float-series work and ports freely, **but its callers walk the season
in-process, day by day**, while Rust's 24 emitters produced only final-state snapshots. A
per-step trajectory export **did not exist** — a new interface, and the reason slice 1 exists.

**One trap nobody had priced:** after the flip Rust loads the params, so Python's unit check
becomes **a green test guarding a path that no longer executes** — the same shape as the pin
that read one registry twice. Slice 9's acceptance criterion is therefore that *deleting the
check must turn something red*.

**One irony worth keeping:** Python freezes the flow **class** name; Rust's `id()` is an
**instance** id. So Rust needs a `type_name()` on the trait — **the opening move of making
Rust the reference is an unfreeze of Rust.**

### Slice 1 — the per-step trajectory export — COMPLETE 2026-08-16

Built: `simcore::snapshot::TrajectoryWriter` (+ `TRAJECTORY_VERSION`), the `emit_trajectory`
example, and one parametrized cross-port gate. Additive throughout — no existing example's
bytes changed, no contract touched.

**Which quantities → all of them, as a full snapshot per row**, in the frozen row shape the
port already emits for a final state. Repeating each stock's metadata on every row costs ~3/4
of the payload (6.75 MB season, 17.8 MB perennial) and is still the right trade: **the export
inherits the cross-port interchange contract instead of opening a second one** — the existing
loader validates every row and the comparator needed no new code.

**Granularity → every step, and deliberately no stride knob.** Down-sampling to days is a
*consumer* concern with exactly one blessed implementation (`tests/day_index.py`, which exists
because the idiom was invented five ways in five files first). A stride argument on the
emitter would put a second one on the far side of the port, out of that module's reach — the
days-vs-steps hazard the slice was warned about, re-introduced by hand.

⚠ **The acceptance criterion as written would have left the reset path unproven, and it was
widened (advisor).** The open-field season runs with **no reset**, so a season-only slice never
exercises the reset hook — and that is the one place the two ports' observer semantics could
genuinely differ, because *both* drivers record the **pre-reset** state and never the reset
instant itself. So the example takes a scenario argument and the gate has two rows.

⚠ **2 years, not the perennial golden's 5, and the number is load-bearing:** the driver
consults the reset hook with the *pre-step* `n`, so a 1-year run never reaches the boundary.
Two years is the smallest horizon that fires it.

**Two things keep the gate from being inert, both checked by measurement, not assumed:**

1. **The comparator matches list elements positionally and only checks list *length*** — two
   equal-length series shifted the same way compare clean. Every row is self-identifying via
   its step index, so the gate asserts the sequence on **both** sides. Measured: deleting the
   initial row is caught; without the assertion it is not.
2. **The perennial case asserts the reset actually fired inside the exported window**, by the
   one signature it leaves in the data (thermal time is non-decreasing within a season and
   only the reset lowers it). Measured: exactly one drop, at row 1220, in both ports.

**What it bought, measured:** the season comparison walks **62,272 numeric leaves** against
~51 for the existing final-state test, and a **1-part-per-million** perturbation of a single
stock in a single middle row fails the band. Every cross-port biosphere comparison until now
was on a **final** state — two ports could have reached the same endpoint along different
paths and nothing would have said so.

⚠⚠ **What the new gate is NOT, recorded so nobody counts it twice.** It **borrows** the
tolerance band rather than exercising it, and it is **not cross-platform-math coverage**.
Every golden comparison beside it compares the Rust port against goldens generated by a
*different* C math library; this one compares Rust against Python **in the same
environment**, so both sides call one library and the deviation is ~0.0 by construction on
either platform. Correct for what it tests — the *shape of the path*, not the last bit — but
a pass says nothing about whether the band is wide enough. The precedent that makes this
worth writing down: a sensitivity probe in this same suite measured exactly `0.0` for weeks
after the code moved out from under it, and kept passing against nothing. ⚠ It *does* run on
CI (the cross-port job runs the whole directory with no marker filter), so the `slow` mark is
not a green-by-skip — checked, because this repo has been bitten by that twice.

**Deliberately not done here:** the crop-model comparison is *not* wired to the new export.
That needs matched-stage / organ-basis plumbing, which is real judgement work for a later
slice. Slice 1 is the interface only, and nothing consumes it yet.

### Slice 2 — `type_name()` on the traits — COMPLETE 2026-08-16

The first unfreeze of the Rust core, and the mechanical half of the class-vs-instance irony
above. A required `type_name()` on the flow trait and the aux trait, across **58 impl sites**
in 11 files, plus five tests. **246 insertions, 0 deletions**; no golden moved, no manifest
touched, `git diff src/` empty.

⚠ **The acceptance criterion as written was passable by a no-op, and was widened (advisor).**
*"No golden moves, clippy clean"* is satisfied by a method **nobody ever calls** — the same
defect shape slice 1 had recorded one day earlier, and the same shape as the probe that
measured `0.0` for weeks against nothing. The real criterion became: exercise the method
**through the trait object, out of a built canonical registry** — the only path that matters,
because that is how the reference derives its frozen set — and give every assertion a measured
negative control. ⚠ **Generalize: an interface slice's acceptance criterion must name a call
site, or it accepts an interface nobody has run.**

⚠ **The design went AGAINST the cheap option, on an asymmetry between two failure modes.**
Rust can derive a type's name automatically at zero cost per site, and that was rejected. Not
because it is wrong today — it produces the right answer today — but because **its output
format is disclaimed by its own documentation as unstable across compiler versions**, and from
slice 6 this string is a value a freeze manifest is anchored to. The hand-written alternative's
weakness (someone renames a type and leaves the literal stale) **is caught by slice 3, one
slice from now**; the automatic version's weakness (a toolchain upgrade turns a freeze gate red
for a non-science reason) **is caught by nothing this repo has**. ⚠ *Choose between two
imperfect options by asking which failure your gates can already see — not which is cheaper to
write.* The secondary benefit is a forcing function: a new flow is now a **compile error**
until its author states the contract identity, which under B is exactly when they should be
thinking about it.

⚠ **That choice paid for itself within the hour: the compiler found 4 impls the grep did not.**
An anchored search found 54 sites; the build failed on **4 more, nested inside test modules**.
The automatic version would have silently handed those four whatever the compiler produced. ⚠
And my own eye-count off the grep output was **48** against an actual 58. *A count read off a
grep is a guess; the thing that must compile is the census.*

**Measured, and it answers slice 3's question before slice 3 asks it:** the names Rust reports
across the four canonical builds are **exactly the 23 flows and 3 aux processes of the frozen
Python manifest, name for name**. So the port can already express the completeness contract —
which is the single thing the flip is riskiest for — before anything is built that depends on
it.

**What is deliberately absent: the roster.** No list of those 23 names appears anywhere in
Rust, and neither does their count. That comparison belongs to slice 3, against the manifest
file itself; a copy here would be the *"a rule with two copies has one that is stale"* hazard,
and the count would be a third place to edit whenever a flow is added. What the Rust tests own
is what slice 3 **cannot** check: that the values are well-formed, that they are a function of
the **type** and not the instance, and that they do not collapse onto each other.

**Six assertions-and-controls, each turning exactly one test red and leaving the others
green** — so they are independent and none is inert: a path-qualified name; two flows
reporting the same name; the type name colliding with the instance id; and a wrapper
delegating.

⚠ **The last two exist because the first four were all on ONE of the two traits.** The
non-conserved side was being read by the same tests, but nothing had shown that a defect
*there* turns anything red — the exact standard this slice had set for itself and applied
rigorously one trait over. It matters because that side is half of what slice 6 re-anchors.
Both added controls bite. **The gap was in the evidence, not in the code, and it was found
by asking which rows of my own control table were missing — not by re-reading the code.**

⚠ **Two claims that are not the same evidence, separated after the advisor pushed on it.**
"No golden moved" was established by **no file under the golden directory changing on
disk** — the direct check. The 138 passing gates *compare against* those goldens, so a pass
is also consistent with a comparison that never ran; quoting it as proof would be leaning on
the weaker fact. Both are true here; only one is the reason.

⚠ **The wrapper case is the sharpest statement of the whole class-vs-instance irony, and it
runs the two axes in opposite directions.** The scaling wrapper delegates its *instance id* to
the flow it wraps **on purpose**, so the registry sorts it into that flow's slot and
order-independence survives. But the reference's class name sees the **wrapper**. So one
accessor must delegate and the other must not. A later refactor making them consistent — which
is exactly what "tidying" looks like — would desynchronise the port's inventory from the
reference for any wrapped scenario, and that test is the only thing that would say so.

⚠ **A suspicion CHECKED and CLEARED, recorded because a cleared one is worth as much as a found
one.** The authoring platform was the place a genuine divergence was expected: the reference
instantiates real domain classes for typed entries but a generic declarative flow for authored
kinetics, and a port that wrapped *everything* in the generic one would report a wrong
inventory. It does not — both ports make the same distinction at the same point. Slice 3 does
not need to hunt there.

⚠ **Found in passing: the Rust tree is not `rustfmt`-clean, and CI has no formatting gate.**
Six files carry pre-existing drift at lines this slice never touched. All 58 inserted blocks
were already canonical, so only the new file was formatted. Reformatting the rest would have
produced a large diff, unrelated to this slice, inside the frozen core — **the same hazard the
*never bare `cargo fmt`* rule exists for, reached one file at a time instead of all at once.**

⚠ **The plan doc's own status header was already false within a day of being written** — it
still read *"NOTHING BUILT"* and *"`git diff rust/` empty"* right through slice 1, which had
updated the slice table but not the header above it. Fixed to point at the table as the single
place slice state lives. *A summary line beside a structured record is a second copy, and it is
the one that rots.*

### Slice 3 — Rust dumps its inventory, checked against the frozen manifests — COMPLETE 2026-08-16

Two Rust dump programs (one per manifest that carries a registry-derived inventory) plus one
new Python gate of three tests. Additive throughout: three new files, nothing else touched,
no golden regenerated, no manifest re-anchored.

**The headline: no divergence.** Both inventories match their manifest name for name —
**23 flows / 3 aux** on the biosphere side, **16 flows / 0 aux** on the station side. The
slice's stop-rule did not fire, and slice 4 — the one that actually moves the reference — is
unblocked on this axis.

⚠ **The slice was planned with three axes and shipped with two, and the missing one is a
finding rather than a simplification.** It was to compare the flow set, the aux set **and
the param-file list**. The port has **no referent for the third**: it reads no YAML at all,
only a flat name→value file **generated by the reference side** out of the frozen loaders,
whose names are the generator's naming and not filenames (one source file feeds three of
them; 17 loaders against 15 frozen files; the station equivalents carry no prefix at all).
Anything the port printed under that key would be the reference's own list travelling out
and back, so the "parity" gate would compare the reference **against itself** — the
self-referential shape this repo already had to dissolve once, for the RNG vectors. The gate
asserts the dump's **exact key set**, so putting the axis back is a red test rather than a
silent tautology: the same forcing-function move as slice 2's required trait method.

⚠⚠ **The knock-on inverts the plan's own dependency table for that one axis** — the param
list cannot be re-anchored until the port is the thing that loads the params, which is a
*later* slice's decision, while the table has that slice depending on this one.

⚠⚠ **And the class is bigger than the instance — my first write-up named only the param
list and the advisor caught the understatement.** Classifying *every* key of the frozen
contract by whether the port can produce it at all: the two inventories can, the light-path
fingerprint can, and **three groups cannot** — the param list, the weather-fixture hash
(the same shape: a hash of a reference-side file the port only sees a generated projection
of), and, largest by far, the **science bands and liveness floors**, which are a static
census of markers on *test functions* — roughly half the file, with no port referent and no
prospect of one while the science gates live in the test suite. The later slice about param
loading resolves only one of the three. So the re-anchoring slice must make an explicit,
recorded choice **per key** — declare it reference-retained with the reason written beside
it, or wait for something that would give it a referent — and must not regenerate a
"re-anchored" contract that silently carries fields nothing on the new reference side
produces. ⚠ *Doing this classification before the ceremony is cheap; discovering it
mid-ceremony with the contract half-regenerated is not.* **Recorded in the plan, not
resolved here.**

⚠ **One claim in the shipped program was reasoning rather than measurement, and was
measured on review.** A doc comment said a sixth build flag "wires no flow either way" — true,
but read off the source rather than run, and it is a claim the gate *cannot* check: if it
were false, the divergence would read as a mistake in one of the five selection calls and
the hunt would start in the wrong place. Flipping it and re-running produced byte-identical
output. *An unmeasured factual claim in a file the next person will trust is the "asserted
attributions rot" lesson in a new location.*

⚠ **Why the second manifest was included, and it is the whole discovery value of the
slice.** The first one had already been measured in slice 2 and was therefore **known in
advance to pass** — a gate that could find nothing. The second was unmeasured, and my
pre-flight estimate that it would match came from **counting matches in a grep by eye**,
which is exactly the evidence this log had recorded as unreliable *one day earlier*, after
the compiler found four cases the same grep missed. The estimate happened to be right. That
is not a reason it was evidence, and "it will probably pass" is not a reason to skip the
half nobody has run.

⚠ **The real hazard was never the port — it was the dump's registry SELECTION.** The
reference-side gate encodes five judgement calls about *which* builds make up the canonical
set (which optional flows are switched on, which half of a two-rate assembly is read, which
delegated registry is excluded). The dump mirrors every one by hand, with nothing checking
the mirror, and **each one is load-bearing on the answer**: get one wrong and the gate goes
red for a mis-specified dump, and the slice is spent hunting a port bug that does not exist.
All five are written down in the program itself, all five were measured, and the gate's
failure message tells the next reader to check the selection *before* believing the
divergence.

**Nine negative controls, each turning exactly one row red and leaving the other green**, and
the tree measured green again after every revert: a renamed name on each of the four
axis/side combinations, the key-set forcing function, and one per selection call.

⚠ **One control exists only because the advisor named the gap before it was built — and it
is the same gap that caught slice 2 one day earlier.** The station aux set is legitimately
**empty**, and `[] == []` is satisfied by a dump that never reached the aux accessor at all:
a green row that has checked nothing. Rows were checked not merely for "something went red"
but for **which assertion fired**, because a control that trips an earlier assertion proves
the earlier one, not the targeted one. ⚠ **An empty frozen set is an inert comparison
wearing the clothes of a passing one** — it needs a control or a stated reason, never a
green run.

**Checked rather than inherited:** the CI job that makes this cross-platform runs the test
*directory*, so a new file is collected automatically. Slice 1's record asserts that in
prose; it was re-read out of the workflow file anyway, because this repo has two recorded
green-by-skip incidents.

**Found in passing, not fixed:** the native build emits a pre-existing **output-filename
collision** warning — one example name is used in two crates, and the toolchain says this
"may become a hard error in the future". Unrelated to this slice, but the repo runs examples
by name.

### Slice 4 — the golden census + the Rust-side regeneration path — COMPLETE

(⚠ heading split for the 120-char record-line cap; its two stragglers landed in slice 5.)

**Built.** `tests/crossport/regen_goldens_from_rust.py` — the committed, reviewable
**Rust-side** regeneration entry point (report by default, `--write` explicit), carrying the
golden census as data — and `tests/crossport/test_golden_provenance.py`, 6 tests / 23 cases,
gating that census and pinning Rust's bytes. Two new files, nothing else touched; ruff /
ruff-format / pyright clean, the whole `tests/crossport/` directory and all three manifest
gates green.

**⚠ The headline is that the reference barely has to move at all. Sixteen of the eighteen
goldens Rust can emit are *byte-identical* to the committed file** — not "inside the band",
identical, including the ~1.3 M-substep sealed station. The plan predicted `< 1e-11`
relative; measured, it is **zero** on sixteen of eighteen. Two are not, both biosphere:
`consumer_chamber_state.json` (7 of 205 leaves, worst 4.6e-16 ≈ 2 ULP) and
`perennial_long_horizon_state.json` (1 of 196, 1.6e-16 ≈ 1 ULP) — five orders **inside**
their own Tier-2 band, structure exact, so the stop-rule did not fire. **Accumulated last-bit
noise, not an op-level difference:** slice 1's trajectory export walks 2440 steps of the
perennial scenario with *zero* bitwise divergence, so there is nothing systematic to hunt.
⚠ *Slice 1 was built as an interface with no consumer; its first real use was diagnosing a
divergence three slices later. The value of a per-step view is that it turns "the endpoints
differ" into "they never differ along the way."*

**⚠ A confound was baked into the first measurement and the advisor caught it before it
became a finding.** It was taken under `--release`; `test_crossport.py` runs the biosphere
family in **debug**, and both divergent cases are biosphere. Re-measured across both
profiles: **all 18 agree**, so the flag is a speed choice. Had it not been, *"regenerate from
Rust"* would have been under-specified until the build profile joined the reference
definition — a frozen hash moving for a toolchain reason, the exact failure mode slice 2
rejected the automatic `type_name` derivation over. ⚠ **Generalize: when you measure a port
against a committed artifact, measure it the way the suite invokes it.**

**⚠⚠ The plan's own arithmetic was wrong and the gap is 7, not 1.** §2f named identifying
"the 25th emitter" as slice 4's first act, on *"24 programs against 25 goldens; one is missing
or one emits two."* All three clauses are off: two programs each serve two goldens, **four**
programs serve no golden at all, and **seven** goldens have no program that emits their bytes.
The replacement census is now gated: **18** Rust emits the artifact; **2** Rust emits a raw
series that `drift.py` folds Python-side (*the fold is the artifact* — slice 3's `param_files`
shape exactly); **5** have no Rust referent at all. ⚠ The sharpest of the five is
`state_snapshot.json`: not a run, a `sim_io` fixture that **Rust reads**, so it is an *input*
to the port and "regenerating it from Rust" is the round trip in its purest form.

**⚠⚠ The blast radius the plan's table understated, measured by swapping the files in and
running every gate that touches them.** Slice 4 was listed as *"25 goldens, reversible
(git)"*. In fact:

* **Both freeze-manifest gates stay GREEN.** `golden_sha256` is assembled only inside
  `_regenerate()` and **never compared** — regenerating a frozen golden silently
  desynchronises the manifest from the file it pins. The *provenance-only edit that nothing
  catches* covers the **goldens**, not just the params.
* **Four Python gates go red**, and all four are `@windows_golden_only` — so the change is
  **green on CI, red only on the developer's box**. [[pdf-pins-green-by-skip-on-ci]] with the
  arrow reversed again.

**⚠⚠ Slices 4 and 5 are not independently landable in the stated order, and this is the
finding worth carrying.** What gives the cross-port comparison its meaning is **not who wrote
the golden** — provenance does not survive in the bytes — it is that **both ports are
byte-pinned to the same file**. That holds for all 18 today. It cannot hold for a golden the
ports disagree on: one side must become tolerance-gated, and moving that pinning to the Python
side *is* slice 5. So the two divergent goldens are **held**, and slice 5 inherits them. ⚠
*"Independently landable" was a property of the plan, not of the tree; the coupling only
appeared once the artifacts were measured.*

**What the new gate buys, and the limit stated rather than implied.** The byte census is **~5
orders tighter than the Tier-2 comparison beside it** — those two goldens drifted from 0 to 2
ULP with nothing noticing, which is how `tiers.json` still carries a measurably false
*"max_rel_dev 0.0"* for both (its file is slice 5's; the ungated prose half again). The
`PORTS_DISAGREE` roster is checked **in both directions**, so a divergence that heals is as
red as one that appears and the roster cannot decay into an unre-measured exemption. ⚠ But
**no byte-level check can say which side produced a golden** while the ports agree; slice 4
makes the *path* structural, not a property of the files. That sentence is in the module, not
just here.

**⚠ The tautology the map exists to make unreachable.** `emit_crew` lives in **two** crates
and `simcore`'s **parses `crew_state.json`'s own hex-floats and re-emits them** — a codec
fixture. It is also the output-filename collision slice 3 found in passing, so a script that
shelled `target/*/examples/emit_crew.exe` would take whichever built last and could write the
golden **from itself**. Every invocation is `-p <crate> --example`, and a test pins the crate.
⚠ *Slice 3 recorded that collision as an unrelated curiosity; one slice later it was a live
hazard on the exact path being built.*

**Ten negative controls, each turning exactly one test red on the intended assertion**, green
again after every revert, and checked for *which* assertion fired: an unclassified golden on
disk; one classified twice; a **frozen** golden parked in the no-referent group; a folded
golden's reason gutted; a typo'd example name; `crew` re-pointed at the echoing emitter; a
known-divergent golden dropped from the roster; an **agreeing** golden added to it; the
last-bit ceiling lowered below the measured divergence; and a crate whose *directory* name
stops matching its declared *package* name. ⚠ That last one is an advisor catch and a
small instance of a recurring shape: the map's crate key silently does **two jobs** — it
locates a directory in Python and it is handed to `cargo run -p` as a package. They agree
across all four crates and nothing but that assertion said they must.

### Slice 5 — the contract inverts; the two stragglers land — COMPLETE 2026-08-16

**Built.** The two divergent goldens regenerated from Rust; the biosphere freeze manifest
re-anchored to them; the divergence roster moved from the Rust census to the Python gates and
renamed; two choke points in `tests/golden_platform.py` (`assert_matches_golden`,
`write_python_golden`); three new gates in `test_golden_provenance.py`; every regression
module's compare **and** write routed through the choke points. Ruff / pyright clean, 113
targeted gates green, all three manifest gates green, `git diff src/` empty, no Rust touched.

**The diff was predicted before it was written, and held exactly:** 8 changed hex-float
leaves across two files, 0 added, 0 removed, no structural field moving. Then exactly two
`golden_sha256` values in the biosphere manifest and nothing else.

**⚠ The blocking hazard was real and was measured, not reasoned about.** Both divergent
goldens sit in the biosphere manifest, whose `golden_sha256` is assembled inside
`_regenerate()` and **never compared**. Before the write all 20 hashes matched disk; the
write turned four Python gates red and **both freeze-manifest gates stayed green while the
manifest pinned bytes that no longer existed**. The ceremony ran here rather than deferring to
slice 6 — slice 6 re-anchors which *keys* derive from Rust; keeping the hash honest about the
bytes on disk was this slice's debt. ⚠ *The "provenance-only edit nothing catches" is not a
quirk of params: it covers a frozen golden's VALUES, and the only thing that surfaced it was
swapping the file in and running everything.*

**⚠ The roster was re-homed, not emptied — and a symmetric name outlives its contract by
exactly one slice.** The first instinct was `PORTS_DISAGREE = {}`, which discards what slice 4
built. The set is still true; what changed is *who consults it*. Before, the golden was
Python's and the question was whether Rust matched. Now the golden **is** Rust's, so
`Rust == golden` has one allowed answer — the census became unconditional, no exemptions — and
the entire open question is about the checker. Same two entries, same measured sizes, opposite
consumer, both-directions non-decay preserved: `golden_platform.PYTHON_DIVERGES`.

**⚠⚠ Two, not eighteen — because a tolerance cannot see a reduction-order change.** Canonical
flow-id order on every reduction is a non-negotiable invariant, and reordering moves values by
a ULP or two, i.e. *inside* any band this repo would write. The byte compare is the only
Python-side gate that sees that class at all. ⚠ And for these two it is surrendered only at
*this horizon*: `emit_consumer` and `emit_perennial` each serve two goldens and the sibling
horizon stays byte-gated in both cases — so the coverage moved rather than vanished. That is
an **observation, so it is asserted**; a third roster entry that would take the last byte gate
off a scenario is red.

**⚠⚠ A negative control killed the first design, and the fix was to generalise it.** Draft 1
converted only the two rostered modules. Control 2 — *put an agreeing golden on the roster,
expect the heal direction to fire* — came back **green**: the heal check is only live where a
module consults the roster, so a third entry landing on an unconverted module would sit inert
forever. Every regression module now routes through both choke points, the seven
Python-authored goldens included. ⚠ *A policy with two implementations has one that is stale —
and it was the control, not the review, that said which.*

**⚠⚠ `loads_back` was reformulated and the price was measured, not argued.** Its codec half
(parse through the core constructors, re-emit byte-stably) is engine-independent, so it stays
**exact** and the flip does not reach it; its equality half is the other test's assertion and
is not duplicated. Measured on the two rostered goldens: a **gross** value tamper is red at
`matches_the_reference` and green at `loads_back`; a **last-nibble** tamper (~1.4e-15
relative) is **green on both**. So a sub-`1e-14` tamper on those two files is invisible to
Python — unavoidable, since it is by construction indistinguishable from the divergence the
roster permits. ⚠ Not a hole: the byte-exact backstop moved to the side that owns the bytes
(`test_rust_reproduces_the_committed_golden_bytes`, unconditional) — which is Windows +
`cargo` + `slow`, i.e. local, not CI.

**⚠⚠ The plan's stated reason to re-measure the bands was false; re-measuring found something
else.** The plan says the bands were measured "through the **Rust-side** transcendentals";
`measure_tier2_bands.py` is pure Python and shims CPython's own `math`. The basis was always
Python-side, and what it measures — how far a one-ULP libm disagreement moves a *trajectory* —
is a property of the scenario's dynamics, not the language. Re-measured anyway: **every figure
in `docs/native-port-reference.md` reproduced exactly.** ⚠ **But `tiers.json` — the file that
calls itself authoritative — was two corrections behind**, still carrying `6.7e-14` and
`2.7e-15` against `3.5e-15` and `2.8e-16`. The doc had been fixed on 2026-08-14 and again on
2026-08-15 and neither fix reached the JSON, *while both files say the doc must not contradict
the JSON*. ⚠ *Two prose halves of one contract can disagree, and the one declared
authoritative is the one nobody re-reads.*

**⚠ And my first write-up of that was wrong.** I called it an ungated band and added three new
gates; `test_biosphere_tier2_band_sits_above_measured_sensitivity` and two siblings already
existed and already reject a zero sensitivity. I had read one test, seen it covered three
keys, and generalised. The tests were deleted before landing. ⚠ *The gates were live the whole
time and no band moved — what rotted was only the record of why each band sits where it does.*

**⚠⚠ A finding for slice 6, noticed while regenerating and easy to lose: the biosphere
manifest now freezes two artifacts of ONE run with two different authors.**
`perennial_long_horizon_state.json` is the Rust port's final state of the 15-yr perennial
run; `drift_summary.json` is `drift.py`'s Python-side fold of the *same* trajectory (Rust
streams the raw series — *the fold is the artifact*). The two engines differ by 1 ULP on
that run, so the manifest's two entries for it are separated by a last bit and by an
authorship boundary. **Nothing goes red and nothing should**: the drift summary is still
compared against Python's own output, which is the correct reference for a Python-authored
artifact. But slice 6's per-key classification meets it directly — the golden axis is not
"18 Rust, 2 folded" scenario-by-scenario, it is *18 Rust and 2 folded, with one scenario
appearing on both sides of the line*. ⚠ Whoever takes slice 6 should decide explicitly
whether that is acceptable as a permanent state or whether the fold gets ported, rather than
discovering it with a manifest half-regenerated.

**⚠ The plan's open question 3 is answered: the Godot consumer does not notice.** §6
asked *"does the Godot consumer notice? It consumes Rust and should not, but slice 4 moves
the goldens it is indirectly pinned against. Check, do not assume."* Checked rather than
assumed: **no test under `tests/crossport/test_godot_*.py` references either moved golden**,
and the only two goldens any of the nine name at all are `cabin_gas_state.json` and
`crew_state.json` — both byte-identical between the ports and untouched by this slice.
Nothing under `godot/` references them either. The question is closed.

**Twelve negative controls, each turning exactly one gate red on the intended assertion**,
including one per branch of the authorship-dependent failure message: a tampered
`drift_summary.json` must be told **Python** is its reference, a tampered
`thermal_state.json` that **Rust** is. ⚠ *No test catches a correct assertion giving wrong
advice* — the assertion fires identically either way, and the first draft sent all seven
Python-authored goldens to look at Rust. The only defence is to read the message under a
real failure. ⚠ Two
early runs reported **false greens from a broken harness** — `sys.executable` outside
`uv run` gives a python with no pytest, exit code 4, which the probe read as "passed". Both
were re-run directly against the real interpreter. *Check a control's own exit code before
believing what it says about its subject.*

⚠ **Left standing, deliberately:** the `windows_golden_only` marker on the two converted
gates. Their original rationale (byte-exactness is platform-bound) no longer applies now that
they are tolerance comparisons — but the band for glibc-CPython against a UCRT-Rust golden has
never been measured, and inventing one is the "derived, not measured" move this contract
exists to refuse. ⚠ **Pre-existing, untouched:** `tests/test_co2_compensation_band.py` carries
7 `E501` errors at `HEAD`, verified against a clean checkout — `uv run ruff check .` was
already red before this slice and was not folded into its diff.

### Slice 6 — the biosphere manifest is re-anchored — COMPLETE 2026-08-16

**Built.** `dump_biosphere_inventory` widened from a *witness* into the **producer** of the
biosphere manifest's Rust half; `_build_manifest()` shells it and splices `flow_set`,
`aux_set`, `forcing.light_path`, `long_horizon_years` and every `scenarios.*.years`; the
manifest gained an `_authority` block naming the producer of **every** key, with the reason
written beside it. Four new Python gates, two new cargo-side gates. `git diff src/` empty;
ruff / ruff-format / pyright clean; `cargo clippy --all-targets -D warnings` and the whole
`cargo test` suite green.

**⚠⚠ The prediction was written down first and held exactly: the only changes to the manifest
are the `_authority` block and the `_comment`.** No hash, no set, no horizon moved.

**⚠⚠ And that is precisely why a green suite proved nothing — the third acceptance criterion
in a row that a relabel would have passed (advisor).** Slice 2 had already measured Rust's
names as exactly the manifest's 23/3, so re-anchoring produces a byte-identical file:
*nothing* in the diff or the suite separates "the manifest now reads Rust" from "the comment
now says it does". The criterion became a **measured pair**:

| Control | Manifest | Python conformance gate |
|---|---|---|
| rename a flow's `type_name()` in **Rust**, regenerate | **MOVED**, one leaf | RED |
| rename the **Python** class, regenerate | **byte-identical** | RED |

Either alone is worthless — the first passes for any file that happens to change, the second
for a manifest nobody regenerated. ⚠ *An interface slice's criterion must name a call site
(slice 2); a re-anchoring slice's must name a **direction**.*

**⚠⚠ The one key that could have failed was measured before being touched — and the
measured pair is NOT the pair the gate runs (advisor).** `forcing.light_path` is the only hash
in `forcing` that is **gated exactly**: CI recomputes it in glibc-CPython and compares
strings. What was measured is **UCRT-Rust against UCRT-CPython, on this box — all twelve
samples byte for byte**. Writing that up as "the cross-libm pair is measured" would have been
a claim nobody made.

**What actually closes it is that the pair never arises**: because the two writers agree byte
for byte, **the manifest's stored value did not move at all**, so the CI gate compares the
identical string it compared before the slice — a comparison that has been green throughout,
which is the standing evidence that these samples are cross-libm stable in CPython.
Re-anchoring added **no new exposure**; it did not add one and then clear it. ⚠ Had the writers
differed in the last nibble — and **two of the twelve samples sit one ULP from their
neighbours**, exactly where that lands — the hash would have moved, the CI pair would have been
new and unmeasured, and the key would have been declared Python-retained.

**⚠⚠ CONFIRMED ON CI — after finding that CI had not been running the Python suite at all.**
The glibc-CPython recomputation passed on `626bd7d`, which is the direct evidence. Reaching
it meant discovering that the Python job (`ruff · pyright · pytest`) had been red at the
**lint** step for 12+ commits, and ruff runs *before* pytest — so **no Python test had
executed on CI** that entire time, including this very gate. Two earlier records called those
lint errors pre-existing and deliberately unfolded; neither followed the consequence.
⚠ *A job's failure at step 1 makes every later step's "greenness" a statement nobody made.*
Fixed in `626bd7d` (line wrapping only): all four CI jobs green for the first time in 12+
commits.

**⚠ The classification is keyed by PATH, because two keys split (advisor).** `forcing` has
three children with two answers; `scenarios` splits *inside one scenario* — slice 5's handoff,
where `perennial_long_horizon_state.json` is Rust's and `drift_summary.json` is Python's fold
of that same run. A top-level block would have hidden exactly what slice 5 handed over. The
golden rows are **checked against** `golden_platform.RUST_AUTHORED`, never restated: that
roster already has two copies held equal by a gate, and a third is the stale-copy hazard.

**⚠⚠ The honest headline is MIXED AUTHORITY, not "the manifest is Rust-anchored" — and the
qualifier is the part that would have been dropped.** By content most of the file is still
Python's: `science_bands` + `liveness_floors` are ~104 of 208 lines, a static AST census of
pytest markers with no Rust referent while the science gates are pytest-side; `param_files`
is Python-retained **until slice 9**; so are the weather fixture, its hash, and
`drift_summary`'s golden hash. The `_authority` block exists so the next reader cannot lose
that.

**⚠ Two anti-derived literals, one new check.** `dt_days` is now compared against the
reference tree's `BIO_DT` — the frozen literal still forces the ceremony, and the reference
can no longer move under it in silence. `integrator` deliberately did not get the same:
neither side has an importable scheme name, so the symmetric version would be two hand
literals checked against each other, which reads like a gate and is none.

**Also landed:** `golden_sha256` is now compared against the files on disk, closing the hole
slice 5 measured. Scoped to goldens only — a golden is machine-generated and *is* the value,
while the param/weather hashes are hand-edited files the goldens already enforce.

**⚠ Two gates in one file stopped meaning the same thing, and the failure message had to
branch.** `test_inventory_parity.py`'s biosphere case is now a **staleness** check (the
manifest is generated from that dump); the station case is still a genuine two-port **parity**
check until slice 7. The assertion fires identically, and its advice — *"a finding to hunt; do
NOT adjust either side to agree"* — is right for one and actively wrong for the other. Slice
5's lesson, applied without waiting to be caught by it again.

**Nine negative controls, each turning exactly one gate red on its intended assertion:** the
rename pair; a golden tampered; an unclassified field; an `_authority` pattern matching
nothing; `drift_summary` reclassified as Rust-authored; the reference's `BIO_DT` → 0.5; its
light-path peak factor → π/2.1; its `LONG_HORIZON_YEARS` → 16; a frozen chamber horizon
tampered, which only the new Python horizon-conformance gate sees; and two `_authority`
patterns of **equal** specificity matching one path — "most specific wins" decides nothing on
a tie, and the field reads as classified either way (advisor, on the closing review). ⚠ **One control run was
invalid and re-run**: a stray command corrupted the manifest's JSON, so sixteen tests failed on
a *parse error* rather than the assertion under test. *A control that turns the whole file red
has measured nothing* — slice 5's "check the control's own exit code", one layer up.

**⚠ A process failure worth recording because it cost real work:** reverting a control with
`git checkout <file>` discarded the **entire uncommitted slice** in that file, not just the
control. The work was recoverable only because every edit was still in the session transcript.
*Snapshot the working files before running controls on them; `git checkout` is not an undo for
a change that was never committed.*

### Slice 7 — the station manifest is re-anchored — COMPLETE 2026-08-16

**Built.** The station dump widened from a *witness* into the **producer** of that manifest's
Rust half (it gained the two sealed horizons); the generator shells it and splices the flow
set, the aux set and both horizons; the manifest gained an `_authority` block naming the
producer of **every** key. Four new Python gates, one new cargo-side gate. `git diff src/`
empty; ruff / ruff-format / pyright clean; clippy and the whole `cargo test` suite green.

**⚠⚠ The prediction was written down first and held exactly: the only changes are the
`_authority` block and the `_comment`.** Not one frozen value moved — same 16 names, same
empty aux set, same horizons, not one hash. Same outcome as slice 6 and for the same reason:
slice 3 had already proved the sets identical, so the ceremony was always going to be a
relabel unless something was wrong. **So the criterion was again a measured pair** — renaming
a flow in **Rust** moves the manifest and reddens the Python gate; renaming the **Python**
class leaves the manifest byte-identical and reddens the same gate. ⚠ The **horizon** axis got
its own pair rather than inheriting the flow axis's: *a re-anchoring slice's criterion must
name a direction per axis, not once per slice.*

**⚠⚠ The blocking item was an axis whose value is EMPTY, and it needed a control shape the
previous slice never had to invent (advisor, before the build).** The station has no aux
process, so every assertion about that set is `[] == []` — and this slice **escalates** it
from "compared" to "written into the frozen contract by a regeneration". The rename control
is *unrunnable* there: there is nothing to rename. The substitute is to **wire a throwaway
one in** and drive the regeneration path — the manifest then *gains* the name and exactly one
gate reddens. ⚠ *When the value under test is empty, the control has to change the value, not
the name.*

**⚠ A gate that was buildable was PRICED AND REFUSED, and the refusal is the finding.** The
station's integration steps live inside a prose string that nothing checks — recorded as such
in that module for two days — and the reference tree *does* carry the constants behind those
numbers, so the previous slice's treatment (keep the literal hand-written, add the missing
comparison against the reference) would work. It was not built, on one asymmetry: **slice 6
added zero new frozen values.** The classification block is metadata about the contract, and
the step gate covered a key that already existed; this one would need a **new structured key**,
i.e. a *widening of the frozen surface* — its own unfreeze with its own ceremony, not a rider
on a re-anchoring. ⚠ *"The gate is buildable" and "the gate belongs in this slice" are
different claims.* Recorded in the key's own classification entry, in the dump, and in the
reference doc, so the hole is a stated claim rather than an omission.

**⚠ Found while classifying, and nobody had written it down: the two contracts now share a
reference-side constant.** The station's 15-year energy horizon *is* the biosphere's decade
horizon constant in the Rust tree. After this slice both manifests are anchored to it, so
moving it is **one edit and two ceremonies**. A reader who assumes the contracts are
independent will predict the wrong diff — which is exactly the failure this slice's own
predict-first rule exists to prevent.

**⚠ The failure advice branched rather than collapsed (advisor).** Both cases of the staleness
gate mean the same thing now, so the obvious move was one message. But the station dump mirrors
five registry-selection judgement calls **by hand**, and after this slice a mis-mirrored one is
*written into the frozen contract* by the very regeneration the message sends you to. So
"check the selection first" is **more** load-bearing than before, not less.

**Also landed:** the golden hashes are now compared against the files on disk, closing on this
contract the desync hole slice 5 measured on the other. And the classification is keyed by
**path** here too — twelve of the thirteen goldens are Rust's, while the energy-drift summary
is a Python-side *fold* of a raw Rust series. The station's copy of the one-run-two-authors
case, found by classifying rather than by being caught by it.

**⚠ The closing review found a duplicate this slice inherited and then doubled.** The dump's
exact key set is declared **twice per contract** — once in the generator, where it stops a
regeneration from splicing an unclassified key, and once in the staleness gate. Both copies
were correct and a control proved each bites, but the failure is **one-sided**: widen the
generator's and forget the gate's, and regeneration accepts the new key while the gate
reddens blaming the *dump* — the wrong place to look. Measured: doing exactly that leaves the
gate's own key-set assertion **green**. Closed the way the previous slice closed its roster —
the generators own the definition, one gate asserts agreement, nobody writes a third copy.
⚠ *A duplicate that a control reddens today is still a duplicate: the control shows both
copies are right now, not that they must stay equal.*

**Twelve negative controls, each turning exactly one gate red on the intended assertion**,
green again after every revert. ⚠ **One had to be re-run because it fired on the wrong
assertion.** The stale-classification control was first run by editing the generator *without*
regenerating: the gate went red, but on "the committed block is not what this module would
write" rather than on the stale-pattern check it was aimed at. The ghost has to reach the file
before the check that hunts ghosts in the file can see it. *A control that reddens the target
test has still measured nothing until you check WHICH line failed* — slice 3's lesson,
arriving on schedule.

### Slice 8 — the authoring manifest is re-anchored — COMPLETE 2026-08-17

The platform contract's half is now generated from `cargo run --example
dump_authoring_inventory` (a new `authoring::surface` census module). **The prediction was
written down before regenerating and held exactly: the only changes are the `_authority`
block and the header comment; no frozen value moved** — same grammar, same eight spec
models, same twelve flow types, same loaders. The criterion was a measured **pair** for the
third re-anchoring running: renaming a wiring field in *Rust* moves the manifest and reddens
the checker, renaming the same field in *Python* leaves the manifest byte-identical and
reddens the checker. **Seventeen negative controls**, each turning exactly one gate red on
the intended assertion; for the two that redden the *same* test, which line fired was read
off a real failure.

⚠⚠ **This is the contract where the flip is NOT free, and that is the slice's finding.**
Slices 6 and 7 were relabels because both ports enumerate a **built registry** the same way —
that is what the plan's §2b retraction ("the gates don't introspect the namespace, they read
`registry.flows`") actually bought. **It does not transfer here.** This manifest freezes a
*platform*, which has no runtime object to interrogate: Python derives it by **language
introspection** (`typing.get_args` over the closed node union, a scan of the schema module,
pydantic field lists, a dict), and the reference offers an `enum`, a `match` and a set of
`const`s instead. Re-anchoring therefore **traded a derived census for a partly
hand-maintained one**, and the manifest now says per key which half is which. *"The reference
can express this contract" is a claim about the MECHANISM, not about the language — check
whether the mechanism that made the last one cheap is even present.*

The mitigation is that several rosters are **load-bearing** rather than descriptive: they are
the tables the parser and interpreter reject against, so dropping a name changes what the
platform *accepts*. ⚠ That was **measured, and the first measurement was wrong for a harness
reason** — `cargo test` stops at the first failing target, so it reported one red test where
`--no-fail-fast` reports **fourteen across five targets**. *Slice 5's "check the control's own
exit code" has a sibling: check that its runner reached every target it was supposed to.* The
same class of error bit twice the same day — a backgrounded suite inherited a `cd` into the
Rust tree and reported **exit 0 / "no tests ran"**.

⚠ **A frozen field that NOTHING checked, found only by building a control for it.** The
grammar's step token was written into the generator as a literal and compared by no gate at
all; changing it moved the manifest and reddened nothing. *An ungated field does not announce
itself — a control with no test to turn red IS the finding.*

⚠ **A third anti-derived literal, where the plan's own census never looked.** The plan counted
"23 derived assertions vs 2 hard-coded" across the three gates, but surveyed only the
biosphere gate's step-size pair; the authoring gate hard-codes the operator set as well. Kept:
it guards a *decision* (that division is deliberately deferred), not a value the tree may move
on its own.

**Because the reference side is weaker here, the Python derivations were kept and their
meaning inverted in place** — the identical assertion now asks *"has the checker drifted from
the contract?"*. A silent reversal of exactly that shape is already in this log, so a new test
pins the direction and asserts the conformance-checked set **equals** the spliced set. And one
buildable forcing function was **priced and deferred** on slice 7's precedent: making a new
spec model a compile error would restructure eight parser call sites to serve a manifest key —
its own change, not a rider on a re-anchoring.

### ⚠⚠ The target state changed mid-slice — B became C (2026-08-17)

The user, while slice 8 was in flight: *"the whole project should become rust based, python
can be used only when using external software as a reference, or in the process of
rewriting."* The plan on disk executes **B** and says explicitly that it *"is not a retirement
of Python (that is C)"*. This is C, with the carve-out the plan had already identified as the
hinge: the crop-model laboratory, which can still mint oracle traces from external software
and has no Rust equivalent.

Slice 8 was **finished under B's design rather than re-scoped mid-flight** — it is required
under both targets, and re-scoping a freeze ceremony halfway is how a manifest ends up
half-regenerated. The delta is written into the plan as §5b and is **a re-plan awaiting the
user**, not a new plan. What changes in kind: B's price was that the two ports stop being
independent, so a disagreement is resolved in Rust's favour; **C's price is that the
disagreement stops being detectable at all.** Every mechanism the project leans on that lives
in Python now needs an owner or an end date, and the manifest keys currently marked
"Python-retained" stop being a classification and become a **queue**.

### C1 — the params move into the reference — COMPLETE 2026-08-17

**The first slice of the C re-plan, and the first that removes Python from the canonical
run rather than re-labelling who owns a contract.** Until now the port read hex-float
tables that `tests/crossport/gen_*_params.py` produced by running the *Python* loaders:
the schema, the unit guard, the bound check and both core-ready folds all executed on the
Python side and the "reference" consumed the answer. All **23** frozen param files — 15
biosphere, 5 sibling, 3 station — are now loaded by Rust itself.

**Built:** a zero-dep `config` crate mirroring Python's `src/config`, sitting *below*
`domains` in the layering. The closed-subset YAML reader **moved** into it from
`authoring` (not reimplemented — a second reader is slice 5's lesson repeated), and
`authoring` re-exports it at its original path, so that crate's public surface is
unchanged and its ~39 call sites compiled **untouched**, carried by a single
`From<ConfigError>` impl.

⚠⚠ **The slice was priced by a measurement taken before a line of Rust was written, and
that is why it stayed cheap.** The hazard was concrete: if Rust re-derived the folded
values from the decimal YAML and one bit moved, C1 would stop being a re-anchoring and
become an unfreeze with **18 Rust-authored goldens** behind it, because slice 5 made the
Rust byte census *unconditional*. The prediction was written down first and then measured
in Python with no Rust at all: **pint contributes zero bits at all six live call sites**
(every one is an identity — `convert` is called with the unit the file already declares),
**75 of 80** generated scalars reproduce a declared YAML literal bit-for-bit, and the four
folded values reproduce exactly. **Bit-neutral, and the Rust gate then passed on its first
run.** *A slice whose risk is a numerical claim can have that claim settled in the language
you are leaving.*

⚠ **The dimensional check turned out not to be a units library, and that re-prices §2e's
trap.** Every declared unit in the frozen tree — 18 `dimensionless`, 17 `degC`, 16 `1/day`,
… — is validated by **exact string comparison**; only two Python functions genuinely
convert, and their six callers are all identities. What C1 had to build was a string
compare plus a correctly-rounded decimal parse.

⚠ **Two things the prediction did not anticipate, both found before code.** (1)
`radiator.yaml`'s `heat_capacity: 1.0e7` is resolved by pyyaml as a **string** (YAML 1.1
wants a signed exponent; `1.0e+7` is a float, `1.0e7`/`1e7`/`1.0E7` are not) and coerced by
pydantic — **the `1.0e7` hazard `yaml.rs` cites as its own reason for existing, live in the
frozen tree rather than hypothetical.** Handled by parsing the scalar's *text*, which is
what pydantic does and what keeps the bits identical. (2) **The reader could not parse our
own param files at all**: both allocation tables were written in YAML **flow style**, which
the closed subset excludes by design. **The subset this project froze for *authored* files
never covered the project's own *param* files.**

**Resolved by reformatting the two files, not by widening the grammar** — the deciding
measurement is that flow style appears in **exactly those two files and zero authored
scenarios**. No value moved (`gen_biosphere_params.py` reproduces its file byte-for-byte)
and **the manifest diff was predicted before regenerating and held exactly**: one line,
`param_files["allocation.yaml"]`'s sha-256. Since those hashes are recorded and **never
compared**, the provenance ceremony was run deliberately. ⚠ Checked before choosing, because
it decides which contract is touched: **the authoring manifest does not name the YAML subset
at all**, and the reference doc never mentions it — the grammar is documented **only in a
Rust source docstring**, outside the frozen contract.

⚠⚠ **A negative control found a live defect in the frozen reader — it shipped as its own
commit, and it is the finding of the slice.** The check asserting *"flow style is rejected,
not silently mis-parsed"* **failed**. The guard lived only in `parse_scalar`, so flow style
was rejected in the mapping-**value** position (`a: {b: 1}` — the only form the test named
`flow_style_is_rejected` had ever covered) and **silently mis-parsed** in the
sequence-**item** position: `- {dvs: 0.0, fl: 0.55}` has a `key:` head, so the mapping path
returned the key `"{dvs"` with the value `"0.0, fl: 0.55}"` and **no error at all**. That is
the exact form this repository's own param files had been written in for years, and any
author writing a flow-style list entry in a scenario file got a silent mis-parse instead of
the documented rejection. Fixed on the key side with one shared excluded-leader constant so
the guards cannot drift, the regression case added **to that test** rather than a crate
away, and measured inert on every existing file. ***A test that names a behaviour is not
evidence it covers the case that matters*** — the same shape as the layered canopy's *"a
probe that NAMES a scheme is not evidence it IMPLEMENTED it"*, one level up: here it is the
*test* that named without covering.

⚠ **The gate is two-directional on purpose.** `every_value_matches_the_generated_table`
compares `to_bits()` for all 66 biosphere scalars, the partition table, and the 12+4 sibling
and station values — **and** asserts the two *name sets* match, because a scalar the control
names and the loader never reads would otherwise pass unnoticed. The generators and their
three `.txt` files are **retained as that control**, per §5c's rule that a generator is not
touched before its successor is green.

**Asserted for the first time:** the MUST-EQUAL constraint between `canopy.yaml`'s and
`nitrogen.yaml`'s `carbon_fraction` — documented in Python since Phase 1 and enforced
nowhere. A divergence models a plant whose leaf area and nitrogen thresholds disagree about
what a mol of carbon weighs.

⚠ **A process failure worth recording: clippy was not run before the first two commits**
and failed on the new crate. Three of the four fixes were cosmetic; **one was not, and
taking the obvious suggestion would have been a silent regression.** `require_positive` was
written `if !(value > 0.0)`; clippy proposes `value <= 0.0`, which **accepts NaN** where the
original rejects it — and the Python loaders are written `if not value > 0.0` for exactly
that reason. Rewritten with `partial_cmp` so the incomparable case is explicit. *A lint
suggestion about float comparison is a semantic proposal, not a formatting one.*

⚠⚠ **The slice's own acceptance criterion is HALF discharged, and the other half now has an
expiry date rather than a fix.** C1 *is* the old slice 9, whose criterion was written in
advance: *"removing the check must turn something red."* Rust-side, done — two tests redden
if the unit guard goes. **Python-side it holds only by accident**: §2e's trap is that
`config/units.py` becomes a green test guarding a path that no longer executes, and the sole
reason that has not happened is that **the retained generators still run the Python loaders,
so pint is still on a live path.** Those generators are retained *as C1's control* — so two
of this slice's own decisions are load-bearing for each other. **The moment
`gen_biosphere_params.py` and its siblings retire, `config/units.py` IS the defect §2e
named**, and retiring them is exactly what "a generator goes once its successor is green"
sets up. Written into the plan's deferred list as an owned expiry condition, because *an
exemption written for a temporary state is a deletion someone must remember* — the lesson
that once left three checks red for five commits.

⚠ **Two prose-only claims were given owners in the same pass.** The NaN rationale on the
bound helpers was a doc comment asserting behaviour nothing checked (`every_bound_rejects_nan`
now does), and the closed-YAML-subset rule the reformat created was enforced by one test in
a domain crate and documented **nowhere** — `docs/param-file-conventions.md`, the page that
governs param-file shape, was silent on it. *The freeze's prose half is ungated*, landing on
the one page that needed the sentence.

**Deferred with reasons, not missed:** `param_files` stays classified `python` in both
manifests. Re-anchoring it needs **sha-256 in Rust** (there is none in the tree, and every
engine crate is zero-dep by charter), a **newline-normalization rule** (the box is Windows,
CI is Linux, and `golden_sha256` is normalized — an unstated rule diverges by *platform*,
not by content), and the **census rule**: the manifest names **15** files where the directory
holds **20**, and the five excluded — four potato overrides plus `demo.yaml` — are excluded
for **two different reasons**. *"A directory is not a category"*, one commit after it was
written down. Also deferred: the weather path, and the relocation of the YAML out of `src/`,
which the `include_str!` paths now reaching out of the Rust tree will force.

**Verification:** Python **2471 passed, 5 skipped, 0 failed**; Rust **347 passed, 0 failed**;
`clippy --all-targets -D warnings` exit 0; `ruff` and `pyright` clean.

### C2 — the twelve laws get Rust versions, and eight get stronger — COMPLETE 2026-08-17

Taken with C8 on the user's instruction (*"do both"*), as separate commits. Two new test
files, `git diff src/` empty, no golden, no manifest, no contract. Rust **348 → 363** tests.

⚠⚠ **The plan's instrument was wrong for two thirds of the set, and measurement pointed at
something stronger rather than a workaround.** §5c said *"add `proptest`"*. Measured against
the actual `@given` sites: **eight of the twelve laws are permutations of three or four
elements**, so Hypothesis samples ~100 draws from a space of **6** or **24** and Rust
**enumerates all of it**. And it is a *choice*, not a constraint — `proptest 1.11.0` and its
whole tree are already in the local registry cache. The four laws that genuinely need
generated values get a deterministic LCG, **deliberately not the engine's own `mix64`**,
because `CounterRng` is the *subject* of two of them and seeding its case set from its own
mixer is the self-referential shape this project dissolved once for the RNG vectors. What is
given up is stated: no shrinking.

⚠⚠ **Three of the reference laws are UNFALSIFIABLE in Rust as written — and one such test was
already in the tree.** They shuffle the insertion order of a Python `dict`; `State.stocks` is
a `BTreeMap`, so a shuffled build and a canonical build **are the same map**.
`observation.rs::insertion_order_independent` is exactly that shape: its two maps are
identical before `observe` is ever called. Re-expressed on the axis that *is* falsifiable —
the **value** of the fold, with fixtures whose sorted and reversed accumulations differ in
bits. ⚠ *A language feature can make a ported law inert without changing a word of it.*

⚠⚠ **Nine controls, and THREE came back green — every one a defect in the new test, not in
the engine.** This is the slice's real content.

1. **The multirate law had no discriminator at all.** Deleting the flow sort in
   `Registry::new` left it green: an order-independence law passing against a registry that
   never sorted. Fixed with three flows on one stock spanning sixteen orders, and
   `dt / n_sub` pinned to exactly `1.0` so the asserted magnitudes are the ones that reach
   the reduction.
2. ⚠⚠ **The ledger law asserted sensitivity on the NOMINAL deltas while the ledger folds the
   RECOVERED ones.** `(1e8 + 0.1) - 1e8` is `0.09999999403953552`, and that set cancels to
   exactly `0.0` in **both** directions. The discriminator is now read off the two states so
   it cannot drift from what is folded. **The reference's own fixture has the same shape and
   its comment claims the opposite** (*"a naive (unsorted) sum would differ by ULPs under
   reordering"*); measured, both directions give `0.0`.
3. ⚠⚠ **The season law is inert by NATURE, not by fixture.** At real physical magnitudes every
   per-stock leg sum is of comparable size, so re-associating them moves no bits — **realism
   cost the discriminator**, which is exactly what the reference's synthetic skeletons are
   for. It now also asserts the rebuilt registry's *iteration order*, which the same control
   does redden. ⚠ *Re-homing a law onto a bigger, more realistic subject can make it weaker.*

⚠ A fourth, caught earlier by the same mechanism: the discriminator helper rejected **three of
my own fixtures** before they could ship green, one of which cannot be sensitive at all —
**a two-element float sum is commutative, and the reference's integrator fixture drains its
source with exactly two flows.**

**Law 3 had no Rust subject and was re-homed rather than filed as a gap.** There is no `demo`
scenario anywhere in `rust/` (checked), and **C6 retires the Python one**, so porting the law
would mean porting a scenario scheduled for deletion. `Registry::into_parts()` is public and
its docstring names rebuild-through as its purpose, so the law lands on the **real season
registry**. What that loses is named in the file: `demo`'s topology, the **RK4 arm** (the
frozen biosphere is Euler-only by charter — that arm survives on the engine-level subject),
and the discriminator above.

### C8 — `param_files` re-anchors to the reference — COMPLETE 2026-08-17

The successor C1 named, with its three stated blockers resolved: **sha-256 in Rust** (hand
rolled; every engine crate is zero-dep by charter), a **newline rule**, the **15-of-20 census
rule**. Both manifests regenerated.

⚠⚠ **The 23 digits are AUTHOR-NEUTRAL, so "`param_files` is now Rust's" is the wrong
headline.** Both trees compute the same digest of the same file under the same rule. Measured
before a line of Rust was written: all 15 biosphere + 8 station recorded hashes reproduce
under Python's rule *and* under the narrow rule Rust would implement. **The prediction was
written down first and held exactly: the entire diff across two contracts is the
`_authority["param_files/*"]` entry in each.** What re-anchored is a pair of **rules** — the
**census** (the files the reference *loads*, a compile-time `include_str!` list, instead of a
**glob of a Python package directory**; directional, and that is the point: a param file wired
into no loader used to *enter* the frozen surface and now drops out of it) and the
**normalization**. That is what `_authority` says; anything stronger would be a claim nobody
made.

⚠⚠ **The newline rule is load-bearing TODAY, and the obvious explanation is FALSE.**
`git ls-files --eol` over the 24 param files: the index is **LF on every one** and
`.gitattributes` declares `eol=lf` — yet the **working-tree** copy of `senescence.yaml` on
this box is **CRLF**. "autocrlf converts on checkout" would have hit all 24. What is true is
narrower and worse: **`include_str!` embeds the working tree**, so the reference's own
compiled-in bytes for one frozen param file differ between this box and Linux CI *right now*.
Measured: the un-normalized digest is `a7c55528…` against the frozen `21163d3c…`. ⚠ *A right
conclusion reached by a wrong mechanism is still a rotten record.*

⚠⚠ **The control pair exists, on the "who does the generator ASK" axis — not "whose bytes".**
My first reading was that both sides hash the same file so there *is* no direction; the
advisor corrected it before the build. Editing a param file's content without regenerating
reddens the conformance gate; changing **Python's** census rule and regenerating leaves the
manifest **byte-identical** and reddens the same gate.

⚠ **A free negative control was already on disk.** The 15-of-20 rule has **two** exclusion
mechanisms — four `crops/potato/*.yaml` by **non-recursion**, `demo.yaml` by **name**. A
recursive walk picks the potato files up, and **two of the four share a basename with a frozen
file**, so it would not merely add names, it could overwrite a frozen hash in place.

⚠ **An in-crate test that looked stronger than it was, deleted before it shipped.** The first
sha-256 padding coverage asserted the padding invariants by **re-deriving the padding with a
copy of the implementation's own loop** — two copies of the code under test, *guaranteed* to
agree rather than merely at risk of diverging. Replaced by 201 digests minted from CPython's
`hashlib` (OpenSSL; no shared ancestry) plus the four published FIPS 180-4 strings. ⚠ And the
frozen param files are **not** that coverage: only **two** of 24 normalized lengths land in
the `len % 64 >= 56` window where the length field spills into a second block, no file is a
single block, none is empty — coverage a one-character content edit would silently remove.

⚠ **Newly asserted, and nothing had checked it: basename uniqueness across the station's six
param directories.** The key is basename-**keyed**, so a collision would collapse two files
into one entry; Python's `_param_paths()` *documents* uniqueness and its dict would quietly
keep whichever directory it read last.

⚠ **A forcing function fired as designed, and its own message was the thing to fix.** Adding
the key reddened the parity gate first, whose text read *"a param-file list would make this
gate compare Python against Python"* — the claim C8 refutes. Three places said it; all three
were rewritten to record *why it was true until C1 and is not now*, rather than deleted.
⚠ *A forcing function that was right for three slices is not wrong when it fires; it is asking
whether its premise still holds.*

**Retained with meaning inverted, not deleted:** `_frozen_param_files()`, `_param_paths()` and
`_normalized_sha256()` are now **conformance checks on the checker** — slices 6 and 8's
treatment. Deleting them would have thrown away the only thing that says the two hash rules
still agree. `EXOTIC_LINE_SEPARATORS` makes the one place they *could* differ **unreachable**
rather than unobserved: Python's `splitlines` breaks on eight characters the narrow rule does
not, and a gate asserts no frozen param file carries one.

⚠⚠ **The sharpest finding of the slice: TWO CONTROLS CAUGHT A ROSTER WITH NO CONSUMER, in my
own new gate.** C8's first draft added `_COMPARED_MAPPINGS = frozenset({"param_files"})` to the
cross-port parity gate and **the loop that reads it never landed** — a string replacement
silently matched nothing and I did not assert that it had. The result was a constant that reads
exactly like coverage. Nothing said so: the key-set forcing function was green (it checks key
*sets*, not comparisons), `ruff` does not flag an unused module-level name, and the whole
suite — 2473 tests — was green. **What said so was running the controls**: editing a param file
without regenerating, and removing newline normalization from the reference, both left that gate
**passing**. Fixed, and both controls now redden it. *This is `docs/log/authoring-manifest-*`'s
lesson — **a control with no test to redden IS the finding** — arriving with the roles swapped:
here the control had a test, and the test had no assertion.*

⚠ A useful split fell out of it. Control D (normalization removed from the reference) reddens
the **parity** gate on both contracts and correctly leaves the **checker's** digest gate green —
the checker still agrees with the frozen value; it is the reference that moved. The two gates
are not redundant, and the control is what shows which is which.

**Prose half updated in the same pass** (*the freeze's prose half is ungated*): both reference
docs' authority tables and unfreeze logs, plus `docs/param-file-conventions.md`, which gains a
line-endings section and two checklist items — including that a **whitespace-only edit is an
unfreeze that reddens nothing**.

⚠⚠ **A follow-up pass closed two claims that were prose-only, and produced a near-miss
worth more than either.** The advisor's closing review found that three places — the
manifest's `_authority`, `docs/biosphere-reference.md` and the plan — all asserted *"a
recursive walk would pick the potato overrides up and redden the census"*, and **nothing had
measured it**; likewise the newly-advertised basename-uniqueness assertion had **no control**
(the one that seemed to cover it planted a duplicate *on disk*, which reddens the *census*
instead — uniqueness is over the two **compile-time lists**, unreachable from disk). Both now
have measured controls. Flipping the real census walk to descend reddens
`the_census_matches_the_directory_on_disk` **on its roster assertion**, and its output shows
the sharp half concretely: the recursive listing contains `allocation.yaml`, `canopy.yaml`,
`phenology.yaml` and `root_depth.yaml` **twice**, so a basename-keyed manifest would not
merely gain entries — it could overwrite frozen ones. ⚠ Also stated rather than implied: the
directory-level uniqueness claim is **composed** from two gates (list uniqueness + per-directory
census), not asserted by one.

⚠⚠ **The near-miss: I ran a 191-file line-ending sweep on an assumption I had not
measured, and it reached into `src/`.** Having established that the *param files'* index is
LF everywhere, I generalised that to the repo and normalized every CRLF working-tree file.
git then reported **191 files modified** — the opposite of the clean-by-normalization
behaviour the param files show — including files under `src/`, where the purity invariant
requires an empty diff. Reverted in full (`git diff --ignore-cr-at-eol` was empty for every
one, so it really was line endings alone). *A measurement's scope is part of the measurement:
"the index is LF" was true of the 24 files I checked and false of the repo.* The conventions
page now says the fix for a stray CRLF file is **that file, not a sweep**, and stops short of
claiming a mechanism it cannot reproduce.

⚠ **And I repeated slice 6's own recorded mistake while doing it**: reverting a control with
`git checkout <file>` discarded the uncommitted edits in that same file. The only reason it
cost nothing is that the edit was a **script** rather than hand-typing — which is the
mitigation worth generalising, since "snapshot before running controls" has now failed twice.

**Deferred with reasons:** the weather path (still the generator with no successor); the
relocation of the YAML out of `src/` — C8 makes that reach-out **worse**, a runtime directory
walk joining the compile-time `include_str!`, which sharpens the trigger; retiring
`gen_*_params.py`, still C1's control and still carrying §2e's expiry condition; and the
light-path fingerprint's hashing, whose dump docstring says it is left to Python *"because …
this crate has no digest dependency"* — **C8 falsifies that premise** but moving it is a second
re-anchoring with its own control.

### C9 — the weather path: the reference reads its own forcing data — COMPLETE 2026-08-17

The generator the C re-plan found with **no successor anywhere in C1–C7**, and the last piece
of the reference's own *input* arriving through a Python script. `biosphere::weather` now
reads the committed raw-weather fixture (`tests/oracle/winter_wheat_weather.json`) directly,
via a closed-subset JSON reader and an ISO-date → day-of-year computation added to the
`config` crate. `gen_biosphere_weather.py`, the `weather_facts.txt` table it wrote, and the
Python gate that kept the two in sync are all **deleted**.

**Bit-neutral, and measured before a line of Rust was written** — C1's and C8's discipline.
All **916** values (latitude + 3 × 305 observations) parse from the fixture's decimal literals
to exactly the bits the port was reading out of the hex-float table. Both conversions are
correctly rounded so they must agree; "must" is measured here. No golden moved; the only diff
in any contract is prose. ⚠ The measurement needed the *literal text*, which `json.loads`
throws away — `parse_float=str` hands it back, and that turned a 916-value comparison into a
five-line script plus a throwaway `rustc` program.

⚠⚠ **The only genuinely new logic is the one piece no golden can check, and the data is why.**
The fixture runs 2006-10-01 → 2007-08-01: no leap year, no 29 February. So the leap rule is
**unreachable from the fixture** — and this was measured, not argued: with the rule broken to
the naive `year % 4 == 0`, `cargo test -p domains` is **48 passed / 0 failed**, including the
bit-for-bit control against the generated table and every season run. Only the hand-computed
calendar tests redden, which is exactly why they are hand-computed against 1900 and 2000
rather than against the fixture. *The float parse was the safe part and the calendar was the
risk; the two have opposite test-ability, and the plan's own summary had it the other way
round.* The other two controls do bite — reversing row order fails five tests, two of them
season runs.

⚠ **A rationale inside the frozen contract was falsified verbatim by this slice.** The
manifest said the weather keys were Python's *"because the port reads `weather_facts.txt`,
generated FROM it"*; after C9 that file does not exist. The *side* did not change and the
*reason* did — the case the authority map's own header anticipates. ⚠ And the pair did **not**
re-anchor to Rust despite being buildable: `include_str!` takes a literal, not a `const`, so
the reference knows the fixture's **bytes** and not its **name**, and a Rust-authored filename
would be *a literal dressed as a derivation*; re-anchoring the hash alone would manufacture
the split authority slice 6 exists to record. Named condition: the relocation slice gives the
fixture a runtime-readable home, and then both keys move together.

**What was built instead is the check that did not exist** — the `locked_dt_days` pattern, a
value emitted **to be checked and never spliced**: the checker hashes the file on disk, the
reference hashes what it *compiled in*, and a new gate fails if they diverge. Not
hypothetical — C9's `include_str!` climbs out of the Rust tree into the Python one, and the
relocation slice is scheduled to disturb that exact path.

⚠ **C1's expiry condition does not apply here, and saying so is part of the work**: the three
`gen_*_params.py` generators are what keep pint on a live path, and this one imported only
`json`/`datetime`/`pathlib`. Deleting it is not a down-payment on §2e.

⚠ A process note, since "never bare `cargo fmt`" has a sibling: **`rustfmt lib.rs` is not
file-scoped either** — it follows `mod` declarations and reformatted two untouched files in
the same crate. Caught in the diff and reverted.

**Deferred, named:** the fixture's new home (the *same* move as the param YAML out of `src/`,
and doing it here would have touched 47 Python test files for a slice that otherwise touches
none), the other five oracle fixtures, and C8's light-path hashing.

#### The post-commit pass — the gate got its control, and the prose sweep found three defects

C9 was committed and pushed before this pass; advisor review afterwards named three things the
green suite could not have caught, and all three were real.

**1. The one NEW gate had no control** — the exact failure C8 recorded ("a control found my
gate had no assertion"), repeated one slice later. Controlled properly: one digit changed in
one `TEMP` value, `test_the_weather_hash_matches_the_reference_tree` reddens with the intended
message, and the **dump hash moves too**, which proves the compiled-in side genuinely rebuilt
rather than the checker comparing a stale constant against itself. A whitespace perturbation
would have been the wrong control — both sides normalize newlines by design, so it cannot
discriminate a working gate from an inert one.

**2. Two sentences in the frozen contract doc became false, and nothing gates prose.** This is
the hazard already on the shelf as *"the freeze's prose half is ungated"*, and C9 walked into
it. `docs/biosphere-reference.md` said the port *"reads a file generated from"* the fixture —
after C9 there is no such file — and its "goldens only" note listed the weather hash among the
ones **never compared**, which C9 is precisely the end of. Both repaired in place, plus a C9
entry in the ceremony history marked **not an unfreeze**: the entry exists *because two
sentences went false*, not because a value moved. Stale delivery-path prose in the phase-7 plan
and the C-plan's own re-anchoring table got dated supersede markers rather than rewrites —
those are records of what happened, and what happened does not change.

**3. `include_str!` silently promoted a test fixture to shipped content.** The licensing doc's
whole EUPL argument turns on *distribution of a derivative work*, and it had settled the oracle
question with "running PCSE to produce fixtures is mere **use**." C9 moved this one file to the
other side of that sentence — it is now inside the binary. The answer is clean, and for a
reason worth having in writing: the rows are **NASA POWER observations**, public domain on the
same footing this repo already grants NASA BVAD, and `NASAPowerWeatherDataProvider` names the
client that *fetched* them, not an author. A derivative of PCSE would have to derive from
PCSE, which a temperature reading does not. `docs/reuse-and-licenses.md` gained the analysis
and the forward rule: check the provenance block before embedding — facts and our own output
may ship, PCSE model output and WOFOST parameter values may not.

⚠ **And the sweep turned up a defect that has nothing to do with C9**: the same licensing doc
claimed the project is **Apache-2.0** and cited `/LICENSE` for it, while `/LICENSE` has been
**BNCL-1.0** since commit `c205560` on 2026-06-28. Wrong for ~7 weeks, by the identical
mechanism — no gate reads prose. Corrected, with the Apache history preserved and the
clean-room conclusion re-derived under the license that actually applies.

### C5 — the drift folds, and the slice that measured its own successor's blocker — COMPLETE 2026-08-17

**Taken out of order, because C4 turned out to depend on it.** The user chose C4 (the
science-gate census) next. Measuring C4's surface first showed **5 of its 15 gates cannot be
written in Rust until `drift.py`'s folds exist there** — and the evidence was not inference:
`emit_drift.rs` and `emit_sealed_energy_drift.rs` **say so in their own comments**, and a grep
for the fold names across `rust/crates` returned only those two comments. §5c lists C4 and C5
as independent items in the same stage. ⚠ *A slice's prerequisites are not in its own row of
the plan table* — had C4 gone first, the folds would have been written inside it and C5 would
have quietly become a duplicate.

**Built.** `domains::biosphere::drift` — the trace builders, the shared OLS primitive, the
four folds, the period-2 check, and the two derived bounds **with their whole provenance
block** (a bound nobody can reproduce is what that block exists to prevent). 16 integration
tests that deliberately **share `test_drift.py`'s own fixtures**, so a behavioural divergence
from the reference shows up as a failing assertion rather than a passing test that checks
something else. `emit_sealed_energy_drift` emits the summary; the checker's copy of the fold
is deleted rather than kept as a second opinion.

**⚠⚠ The gating measurement split the slice in two, and only one half could land.** Predicted,
measured, then built — C1/C8/C9's discipline. `sealed_energy_drift_summary.json` came out
**byte-identical**: authorship re-anchored, the hash unmoved, so *the rule moved and the digits
did not* — C8's finding again. `drift_summary.json` moved **4 of its 45 values** (≤7 ULP).

**⚠⚠ The gate that blocked it was written two slices ago, and its error message was the
instruction.** The obvious next step — regenerate, then add the file to `PYTHON_DIVERGES` as
slice 5 did for its two stragglers — is **red by construction**:
`test_every_diverging_scenario_keeps_a_byte_gated_sibling` finds that `emit_drift` serves
exactly one golden. It says *"Diagnose the divergence instead of adding it to the roster."* So
it was diagnosed: the consumer trajectory first differs by **1 ULP at step 4095**, 1750 of
18301 steps then differ, and **the final state is byte-identical again**. A contracting
attractor damps the difference back to zero — which is why the final-state golden stays
byte-gated and green, and why the per-year peaks are the *only* artifact that samples the
difference while it is alive.

**⚠⚠ The gate was left alone even though widening it is defensible on the merits.** Its key is
the emitter *program*, a proxy for "the scenario", and the proxy under-approximates: the two
scenarios `emit_drift` runs each keep a byte-gated golden elsewhere, so admitting the entry
would remove no coverage. The argument is sound and it was still refused, because **it was
constructed while trying to add the entry it licenses**. Slice 5 turned "coverage survives"
from a sentence into an assertion precisely so it could not be re-argued; re-arguing it from
inside the slice that needs it relaxed is the co-adaptation refused for the stem-only branch
and for the canopy floors. ⚠ The contract-preserving alternative was priced and does **not**
hold: `Emitter.run()` returns a program's entire stdout as one golden's bytes, so `emit_drift`
cannot serve a second golden without re-pointing one away from `emit_consumer` — manufacturing
the sibling rather than discovering it. **The authorship move is deferred to its own ceremony,
and `PYTHON_FOLDED` now carries the measurement and the blocker in place of the old reason,
which C5 itself made false.**

**⚠ A port defect found by porting the reference's EDGE CASES first.** `(len - 1) // year` is
`-1` in Python and `range(-1)` is empty; the literal Rust transcription **underflows `usize`**.
Found by porting `test_year_summaries_handles_short_trajectories`, not by reading the code, and
the guard carries a measured control — reverting it reddens exactly that test. *Generalize:
port the edge-case tests before the happy path; that is where two languages disagree about what
an expression means.*

**⚠ A dated figure in the tolerance contract is no longer true, and C5 is what makes it
matter.** `tiers.json` records `drift_summary` as *"bit-exact locally, max_rel_dev 0.0"* at
P7.4; measured now it is 9.955e-16 — 4 orders inside the band, explicitly dated, nothing red.
Recorded because **a tolerance band hides a difference until the flip makes one side the
author**: today those ULPs are a tolerated deviation; the moment Rust authors the golden they
are the reference value.

### The state of the arc

Slices 8–11 are an unexecuted menu; the user takes them one at a time and none is scheduled.
The ordering that matters: **1–3 built nothing the reference depends on** (they de-risked the
export and proved the port can express the completeness contract *before* anything
re-anchors). **Slice 4 was to be where the reference actually moves — and the measurement was
that it barely had to**: sixteen of eighteen goldens were already byte-identical between the
ports, so what 4 could land alone was the *path* and the *census*, not a diff. **Slice 5 wrote
the other two, and with them the inversion**: the golden is now Rust's artifact, the Rust
byte census is unconditional, Python is the tolerance-gated side, and Python can no longer
author any of the eighteen. **6–8 are the three unfreeze ceremonies, biosphere first; all three
have landed.** Their lasting finding is that a freeze contract does not re-anchor as a unit:
roughly half of each manifest has no Rust referent and is now declared Python-retained *in
the file*, so **slice 8 should expect to classify, not to convert** — and its surface is
different in kind again (grammar, VM node/op set, flow-type registry, with no flow/aux
registry axis), so it will not look like either of the two that landed. Until it does, the
authoring contract is still Python-anchored, new reference science outside the biosphere and
the station assembly is still Python-canonical, and a science item must never share a batch
with a re-anchor slice.

⚠ **This item's log exemption was deleted with slice 1, one day after it was written.** It was
written on the premise *"forward-looking, no finished work behind it"*; the first slice ended
that premise, so the doc took the normal three then rather than carrying a false exemption
through ten more slices. **An exemption expires when the first slice of its plan lands, not
the last.**

### Recorded, not planned: what the user says follows the flip

**2026-08-16, in the same message as "begin slice 1":** after the switch to Rust, work
continues on the **universal harness** — easy toggling of parameters and science. It has a
plan and a built foundation already (the value-switch plan): the in-memory parameter-override
seam shipped 2026-08-15, but **its reporting layer, the actual deliverable, is still open**.
⚠ That seam is Python-side, in the *checker's* half of the tree under B — whether the harness
is rebuilt against the Rust loader or stays Python and drives Rust is the **same question** as
slice 9's unit validation, for the same reason. Take them together, or slice 9 first.

⚠ One phrase is recorded as the user's goal and **not** adopted as a design property: *"ease
the need to run tests and constant verification."* Read as *"experiments must be cheap enough
that nobody hand-writes a probe script for each one"* — the measured motivation behind the
existing plan, **42 throwaway probe scripts across 16 plan docs** — it is exactly right and is
what the harness is for. Read as *"the harness stands in for verification"* it inverts this
repo's posture, and every mechanism this flip touches exists because that substitution is not
available. The distinction decides whether the harness's output is *a finding you then gate*
or *a gate*. It is the first.

### C3 — the posture lands in `CLAUDE.md` (2026-08-18)

**COMPLETE, and the whole slice is a docs edit: `git diff src/`, `git diff rust/`, the 25
goldens and all three manifests came back empty**, exactly as `PREDICTION.md` said before the
first character was typed. Plan A had called landing its rules in `CLAUDE.md` *"the single most
important step; the rest is tooling"* and never did it; the posture then lived only in a plan
doc and a memory file, which is why it never became a default. That is now closed for C: the
always-loaded file states that Rust is the reference, that Python has no reference authority,
that `git diff src/` empty has **inverted** (`src/` is what shrinks), and that the only Python
surviving the plan is the hand-run PCSE oracle carve-out.

⚠⚠ **THE SLICE FOUND A THIRD FALSE STATEMENT, AND IT WAS NOT THE FLIP'S.** The freeze-contract
table still read **`Euler/dt=1`** — stale since the step unfreeze four days earlier, a change
that ran as a full ceremony with a plan doc, a regenerated manifest and a re-pinned literal.
*A file loaded unconditionally is read constantly and audited never: the ceiling test bounds
its size and the parity tests bound its index, but no gate in this repo compares a sentence in
it against the tree. Every unfreeze should re-grep it.*

**The `windows_golden_only` marker, left to this slice by slice 5, was refused on a number.**
Its rationale (byte-exactness is platform-bound) no longer reaches the two gates that are now
tolerance comparisons, so the tempting move was to unskip them on Linux under the 1e-14
last-bit-noise ceiling. But the worst propagated ±1-ULP transcendental sensitivity in that
scenario group is **3.520e-15** — under **3×** of headroom for *one* perturbed site, while
glibc-vs-UCRT perturbs all four sites at once. The evidence argues against the assumption, so
the marker stays, with an **expiry condition** at its definition naming the only thing that
retires it: one Linux run reporting the max observed deviation.

**The byte budget was the slice's own control.** Predicted 10,200–10,900 B against a 12,000 B
ceiling; the first draft came in at 11,452 — inside the ceiling, past the band, and past this
slice's own written trigger to cut. Cut by **retiring**, not by trimming: the entire `Status`
section went, because every sentence in it was a second copy of the header, the pointer list or
Working style. Final 10,906 B with 1,094 B spare. ⚠ *When an always-loaded file must grow, the
budget is paid by finding the duplicate — a section that only restates other sections is the
cheapest thing in the file and the hardest to notice.*

### C6 — the four Python-only scenarios retire, and the successor had to land first (2026-08-18)

The user's decision of 2026-08-17 executed: `n_limited`, `water_biting`, `demo_euler` and
`demo_rk4` deleted rather than ported, each with a written reason. Taken before C4's build
because C4's mutual-shading gate asserts over six scenarios, two of which were this slice's
subject. The goldens go 25 → 21 and the reference's share of them 19/25 → 19/21, with **no
value moving anywhere** — all four names occur zero times in all three manifests, so nothing
here was an unfreeze.

**⚠ Retiring the LAST run that reaches a branch is not a deletion, it is an orphaning — and the
grep that proves it takes a minute.** `nitrogen_stress_factor` had **zero test callers anywhere
in `rust/`**. Every Rust scenario holds the nitrogen limiter at exactly 1, so `n_limited` was
the only run in either tree that ever drove it below 1; deleting it would have left a branch of
the *reference* exercised by nothing, and that is not fixable after the fact the way a stale
comment is. The successor landed **first**, in the shape Rust already used for the water side:
manufacture the condition in a test rather than carry a scenario. Two negative controls,
each reddening a different load-bearing line and **nothing else in the suite** — dropping the
`f_water * f_n` multiply reddens exactly one of 45 tests; flattening the ramp reddens two.
*The generalizable form: before deleting a scenario, ask which branches it is the last witness
of, and answer it by grepping the REFERENCE for test callers — not by reading the scenario's
own docstring, which describes what it does and never what nothing else does.*

**Two claims had no successor, and the slice's value is that they are named rather than
absorbed.** A two-row CO₂-trough comparison lost the only row that took its non-bit-identical
branch, so the bound it carried has no subject left; and a reserve-vs-frozen check on the
nitrogen-limited regime cannot follow, because its second arm is a *candidate form from a
decision already taken* — there is nothing left to compare the frozen tree against. Both got a
tombstone comment at the old site. Two further claims narrowed rather than died: the
shedding-fed litter C:N regime now has one witness instead of two, and the fractionation
table's four claims now rest on three carbon-limited runs, none of which exercises water
limitation at all.

**⚠ A one-element `for label in (...)` is one deletion away from asserting nothing.** Reducing
a two-arm comparison to one arm left exactly that shape, so the survivor was rewritten without
the loop and negative-controlled: inverted, it fails at 105.93 against 90.0 — a margin, not a
near miss. *A loop over a literal collection is a vacuity hazard the moment the collection
stops being plural.*

**What did NOT need a successor was also measured, not assumed.** The water-store geometry
identities the deleted test asserted are already asserted for *every* scenario by a test that
enumerates from the module rather than a hand list — so it auto-shrank with the deletion and
lost nothing. ⚠ *Before writing a successor, check whether the roster-wide test already covers
the claim; a per-scenario pin is often a survivor of the era before the roster-wide one
existed.*

**The blocker C4 was waiting on cleared without a number moving, and that was measured before
the edit.** The mutual-shading gate pins `max(chambers) < 1.0` with an inline `0.585`, and a
departing scenario owning that number would have forced a re-measurement. Peaks taken for all
six first: the pinned number is `consumer_chamber`'s 0.5849, a survivor; the departing peaks
were 0.4718 and 0.0869, neither ever binding. ⚠ *Measure the pin's provenance before shrinking
the roster it reads — the falsifier is cheap and finding it afterwards is not.* The edit also
dropped a mislabel: the gate calls every label except `open_season` a "chamber", and one of the
departing scenarios was an open-field run.

**Prose was corrected by TENSE — the third instance of C3's finding, arriving from a new
direction.** A dozen sites carried present-tense claims about what the tree *contains* ("one of
only two runs in the tree where water limits", "the one place `f_N` bites", "a sealed chamber
outside the manifest entirely"). Every one became false the moment the scenario went, and no
gate in either tree compares such a sentence to the tree. They were rewritten with the
retirement date rather than deleted, because the measurement each records is still true of the
day it was taken. ⚠ *A separate class was deliberately left alone: prose naming a **discipline**
("the `n_limited` precedent") survives its exemplar; only the dangling **file** pointers needed
repair. Deleting both would have thrown away the reason the discipline exists.*

⚠ **Scope held: the demo skeleton stays.** C6 retired the two demo goldens and their regression
gate, not `build_demo` — whose own tests assert engine-assembly properties, a different subject
and Stage 3's problem. The consequence is recorded where it bites: `demo.yaml` is now frozen by
nothing.

**⚠ And the sweep was scoped too narrowly — caught in review, not by a gate.** Every
post-deletion sweep ran over `tests/ src/ --include=*.py`, so the two **living freeze-contract
docs** were in the slice's first grep and none of its later ones; both still described the
retired scenarios in the present tense, inside their own "scoped out, by name" sections. The
gating check that opened the slice proved no *machine-readable* half of any contract moved —
and that is precisely the half that has a gate. *A contract here is doc + manifest, and only
the manifest is gated.* The live scope statements were updated; the dated unfreeze-log entries
naming those scenarios were **left alone**, under one new note saying entries are records of
their date, because rewriting a measurement to match a later tree falsifies the measurement.
⚠ *After any deletion, sweep the living contract docs explicitly — they are in neither `src/`
nor `tests/`, so a language-scoped grep cannot see them.*


### C4 — the science-gate census moves to the reference, and the diff was 13 loci (2026-08-18)

The largest Python-authored block of any contract — about **half the biosphere manifest by
content** — is now the reference's. §5j of the plan measured it first and split it: 13
biosphere gates here, the two station ones (`crew_mission`, `sealed_station`) as **C4b**,
because slices 6–8 re-anchor one manifest per slice and the station's two need referents the
reference does not carry yet.

**What moved: 13 `locus` strings and two `_authority` notes.** Nothing else. All 13
`quantity`/`bound`/`source` strings are byte-identical, every gate value re-measured matches
§5j's release probe to 17 digits, and the station manifest is untouched.

#### The mechanism, and the failure it does *not* prevent

§5j's open question was that Rust has no introspection, so a table plus tests is a
hand-maintained roster — the exact thing the Python `ast` census existed to avoid. The answer
is that **the row IS the test**: one `macro_rules!` invocation declares each gate once and
emits both the roster entry and the `#[test]`, with the `locus` built from the test's own
identifier.

⚠ **Measured, not assumed: that makes an *unexercised row* unrepresentable and does nothing
about a *deleted claim*.** Removing a whole declaration leaves the Rust suite **green** — the
test ceases to exist — and it is the manifest comparison in `test_inventory_parity.py` that
reddens. Two failures, two mechanisms; the macro covers one of them.

#### ⚠⚠ The finding: the first regeneration froze mojibake, and every gate was green

The reference emits UTF-8; `subprocess.run(text=True)` decoded it with the Windows locale's
cp1252, so `—` entered the contract as `â€"` and `Γ` as `Î"`. **Nothing was red, because the
manifest and the checker agreed** — the corruption happened on the way in, so the comparison
was between two identically-mangled sides.

It was caught by **predicting the diff before regenerating** and getting 37 changed lines
where 17 were predicted. Every byte this dump had ever emitted was ASCII (names, hex floats,
digests): *the first non-ASCII key is the one that finds this*, and frozen claim text is the
first prose in the contract. Both readers now pin `encoding="utf-8"`; losing it on one side
is red in the parity gate, losing it on **both** compares equal, so the characters themselves
are asserted separately. ⚠ ~20 other `text=True` pipes read Rust output and none is red today
because none carries non-ASCII — recorded as a **named condition, not a swept fix**.

#### The hole the pre-reduction opened

The gates fold per-step scalar series rather than `Vec<State>` (the station's own precedent).
`year_summaries` computes `n_years = (len - 1) / year`, so an observer emitting `steps`
instead of `steps + 1` gives **14** annual summaries — and every gate still passes, because
`non_collapsing` over 14 years passes as well as over 15. Python never needed that guard.
Controlled: dropping one sample from one series reddens **exactly one** gate (the count
assert) and the CO₂ band does not notice, because its minimum is not at the end.

#### A pin went red, and that was the pin working

`test_the_plausibility_bands_are_now_named_by_a_manifest` asserted that the manifest names
`test_senescence_form` and `test_nitrogen_form` — the Python **files** the loci pointed at
when the science was granted standing. Re-pointed at the new loci, **not relaxed**: the same
treatment its own docstring records for the pin it replaced.

#### Cost and residue

Runtime `+~3.4 s` on `cargo test` (the six trajectories shared through `OnceLock`, Rust's
answer to Python's module fixture). The 13 markers left the Python suite with a comment at
each site naming its Rust successor; **the test functions stay** as the checker's copy —
deleting them is Stage 3's call. Five frozen `source` strings still spell a Python test name;
the companion assertion was ported under the same name, and the strings were deliberately not
edited, because that would be a value change rather than a locus re-anchoring.

Full detail, including the five negative controls: `docs/plans/post-roadmap-reference-flip.md`
§5l.

### C7 — the manifest writer moves to the reference; the biosphere half (2026-08-18)

The last Stage-2 slice, and the one whose absence made the flip's own headline false. The
three frozen contracts were *authored* by the reference key by key — slices 6, C4, C8, C9
each moved a block — and **written** by the checker:
`tests/test_freeze_manifest.py::_build_manifest()` shelled the reference's dump, spliced
its keys into its own, serialized, and wrote the file. A contract whose first line says
Rust is the reference had a Python program in the middle of it.

Biosphere only. The order — biosphere → authoring → C4b → station — was set before any
code, and the station half has a **measured prerequisite**: its two science claims (the
respiratory-quotient comparison and the thermal node's non-collapse floor) have no Rust
referent, and a writer that cannot derive them must not hand-carry them. **C4b is a
prerequisite of C7's station half, not a follow-up** — the C5-before-C4 shape again.

#### The objection that dissolved: does moving the writer make everything Rust-authored?

It looked like the blocker and the tree had already answered it. `_authority` records **who
produced the value**, not who ran the digest and not who wrote the file — and the
precedent was sitting inside the block being worried about:
`scenarios/*/golden_sha256` has read **`rust`** since slice 4 while *Python* computed the
digest, because the golden is the reference's own output. The mirror image now holds: this
program hashes `drift_summary.json` without becoming its author, exactly as Python hashed
six Rust-authored goldens without becoming theirs.

So the move is authority-neutral by construction. No schema change, and the two rows that
*did* change side changed for their own stated reasons, not as a side effect.

#### The gating measurement, and it came back free

The question was whether Rust can reproduce `json.dumps(indent=2, sort_keys=True) + "\n"`
byte for byte. Measured across all three manifests before designing anything: **no
character above the basic plane** (so no surrogate-pair logic — the writer panics on one
rather than carrying code no test could exercise), 232 `\uXXXX` escapes in lowercase hex,
39 empty containers, 15 `null`s, and **exactly one number that is not an integer** — the
frozen step `0.25`, itself a hand literal. That last one is why the float-formatting risk
(`repr` vs `{}` on values like `1e-05`) never arises.

**Byte-identical on the first run.** The writer move moved no byte of the contract.

#### ⚠⚠ The finding: the trap this slice sets is invisible to the gate that guards it

`dt_days` is frozen by hand on purpose — a manifest that read `BIO_DT` would auto-follow a
step change, which is the opposite of a freeze, and the 2026-08-14 step move became a
ceremony only because that literal went red. C7 moves the writer **into the crate that owns
`BIO_DT`**, where splicing it in is a one-character edit.

Measured, not argued: replacing `Json::num("0.25")` with `Json::num(format!("{BIO_DT}"))`
produces a **byte-identical manifest**. So C7's own gate — regenerate and compare — is
blind to it, and so is the cross-port check that compares the frozen literal against
`BIO_DT`, which compares equal either way. What that check protects is the *ceremony*, and
the ceremony exists only while the literal is typed.

This is the step unfreeze's lesson recurring in a new place: *no test at `dt = 1` can tell
a correct conversion from a wrong one, because the two are the same integer.* Two guards,
because neither alone is enough — the serializer's `Number` is constructed only from
**text** (`Json::num` takes no `f64`, so splicing is a visible `format!` rather than a
silent coercion), and `rust/crates/domains/tests/manifest_writer.rs` reads the writer's own
source and asserts the emission site is a quoted literal that does not mention `BIO_DT`.
⚠ That test's own control earned its keep immediately: the first anchor was the bare key
`"dt_days"`, which matches **two** lines — the emission site and the `_authority` row that
classifies it — so the check would have read whichever came first.

#### Deleting the Python writer reddened nothing — and the deletion opened a hole

Asked before deleting, per the authoring slice's lesson: `_build_manifest` is reachable
only from `_regenerate`, reachable only from `__main__`, invoked by no test. **A control
with no test to turn red IS the finding**, recorded rather than quietly fixed.

But the deletion did more than remove dead code. The **scenario roster** (`name -> label,
horizon, golden`) lived in the Python module *and was written from it*, so it needed no
gate: the manifest could not disagree with its own source. Moving the writer turned one
source into **two copies with nothing holding them together** — and the two fields at risk
are exactly the ones `_authority` marks `hand`, the human label and the golden's filename,
which no gate can re-derive if they drift. Names were already compared; run lengths were
already compared; those two were not. `test_the_frozen_roster_is_the_references` closes it,
and a control confirms it reddens.

A second duplication was caught and removed rather than gated: the first draft had the
writer re-walk the registries, putting two derivations of the same flow/aux sets in one
file. Sharing one walk makes the drift impossible instead of merely detectable, and costs
the parity gate nothing — its subject is staleness, the live tree against the frozen file,
not one code path against another.

#### The deliberate diff: five lines, and no value

Two reclassifications were taken as a visible, stated diff rather than smuggled into the
re-anchoring. `forcing/weather_fixture` moves `python` → **`hand`**: C9's finding stands
(the reference `include_str!`s the fixture, so it knows the bytes and not the name), and
while the checker wrote the file `python` fairly described who typed the name — once it
does not touch the file, `python` names a producer that is not there. `forcing/weather_sha256`
moves `python` → **`rust`**, and it is free *now* precisely because C9 declined to do it:
C9 would have had to split the pair while the name had no Rust referent, and a gate has
held the two sides' bytes equal ever since. `_comment` changed because it names the
regeneration command, which was about to become false.

`git diff --stat`: 5 insertions, 5 deletions. No hash, set, claim or bound moved.

#### What the new gate catches that nothing did before

`tests/crossport/test_manifest_writer.py` regenerates into a temp file and compares
**bytes**. The existing parity gate compares derived sets axis by axis and says nothing
about the hand-authored half, the serialization, or the keys the checker still authors. So
three failures became visible, and one of them is not new but was never watched: **a hand
edit to the committed manifest.** It is a generated artifact, and before C7 a typo in
`_comment` or a hand-patched hash simply stood.

⚠ Measured while checking it: a warning in `regen_goldens_from_rust.py` that a `--write`
moving a frozen golden *"turns nothing red"* is now **false for the biosphere** — the
writer hashes the goldens from disk, so the regenerated manifest differs and the gate is
red. It remains true for the station. Corrected there rather than left standing.

#### And the pipe is gone

The file is written with `std::fs::write`, not printed for the checker to capture. C4's
first regeneration froze cp1252-mangled prose into this contract **with every gate green**,
because a `subprocess` pipe decoded UTF-8 with the Windows locale and both sides were
mangled identically. C7 deletes the class rather than inheriting it. ⚠ The *dump* path is
deliberately unchanged — it still emits raw UTF-8 through a pipe, because the parity gate
reads it and the encoding pin it grew after C4 is a control that only has teeth while there
is non-ASCII to mangle.

#### Two things the full run found that the targeted runs did not

The crossport suite (12 minutes) went red on
`test_the_dump_key_sets_are_the_ones_the_generators_consume`, which tied the crossport
copy of each dump's key set to the generator's copy. **The biosphere row lost its second
side**: with no generator, nothing consumes the dump, and a row there would compare the
module against itself. Retired with the reason written down, and with what replaced the
forcing function named — the dump's key-set assertion in the parity gate, plus the new
byte comparison. ⚠ The row does not come back when the other writers land; those two rows
*leave* with them.

⚠ Checked afterwards rather than assumed, because the loose version of that reasoning is
wrong: what the surviving assertion forces is that a new dump key be **declared**, not
that it be *classified* — and a control confirms a dump-only key still reddens it.
Classification was never the dump's job in the first place: three of its keys
(`locked_dt_days`, `horizons`, `light_path_samples`) have never reached the manifest and
so have never carried an `_authority` row, because they are emitted to be checked against
rather than frozen. `_authority` coverage is over **manifest** keys, and C7 does not touch
it.

Also worth recording as a repeat: running `rustfmt` on `crates/config/src/lib.rs`
reformatted `params.rs` and `yaml.rs` too, because rustfmt follows `mod` declarations. That
is [[dev-env-never-bare-cargo-fmt]] in a shape the memory does not name — the hazard is not
the `cargo fmt` wrapper, it is formatting any file that is a module root. Reverted.

#### Verification

`cargo test` (all green) + `cargo clippy --all-targets -D warnings` clean; `ruff`, `pyright`
and the Python suite green. Controls: escaping removed → red; empty-container layout changed
→ red; hand-edited manifest → red; drifted roster label → red; moved golden → red; the step
constant spliced → **green, which is the finding**. Probes: `M:/claud_projects/temp/c7-build/`.

### C7 — the authoring half: the platform contract's writer moves (2026-08-18)

The second of C7's three halves, in the order set before any code (biosphere → authoring →
C4b → station). Nine derived keys, five hand ones, two provenance hashes — and **every
derived key regenerated byte-identical on the first run**.

#### The deliberate diff: two lines, and one of them is a finding

`_comment` named a regeneration command C7 makes false. The other line is the interesting
one.

**`parity_vectors/*`'s `why` argued against C7.** It read *"A Rust-side hash would compare
the checker's own output with itself"* — and the new writer hashes those files. That is the
objection the biosphere half dissolved, except frozen inside a contract instead of raised in
a design discussion: it conflates **who produces a value** with **who computes its digest**.
The precedent is one file over — `scenarios/*/golden_sha256` has read `rust` since slice 4
while *Python* hashed it. So the `side` did not move (the generator that writes these files
is still live, checked rather than assumed); only the reasoning was wrong, and it was
corrected in place rather than left to argue against its own writer.

⚠ Not the same as C9's `forcing/weather_fixture`, which went `python` → `hand`. That one
moved because no Python touched the file any more. Here the producer is still there.

#### ⚠⚠ Deleting the writer opened the same hole, in a different key

The **roster** — which files `parity_vectors` records — lived in the Python module *and was
written from it*, so it needed no gate: the manifest could not disagree with its own source.
Moving the writer made it two copies with nothing holding them, and the copy at risk is the
one marked `python`. A file dropped from the reference's list simply stops being hashed and
every other gate stays green. `test_the_frozen_vector_roster_is_the_generators` closes it,
tied to the **generator's own output paths** rather than a list retyped in the checker.

⚠ It also adds a value check that never existed: the two hashes were provenance nothing
recomputed. The gate now recomputes them under Python's rule against the reference's
narrower one — C8's two-rules-held-equal tie, arriving here through C7.

#### The trap: measured for, and absent

The biosphere half's finding was a frozen literal left one character from auto-following a
constant, invisibly. Asked here, the answer is **none**: the hand keys are a phase number,
two repo paths and two blocks of prose, and this crate owns no constant they could be
spliced from. Recorded as a measurement rather than answered with a guard invented to match
the other half's — a guard with no trap to catch reads as coverage.

#### ⚠ A recorded reason that was too broad, exposed by the second case

`test_the_dump_key_sets_are_the_ones_the_generators_consume` lost its authoring row as
scheduled — but the reason written there when the biosphere row left, *"nothing consumes the
dump"*, is **false here**: this dump is still consumed by the parity gate. What licenses the
removal is narrower — there is no second copy of the key set left, because the copy lived in
the writer. Same outcome, different reason, corrected there.

#### The control hardcoded to one row

`test_the_writer_refuses_an_unknown_argument` named the biosphere example by hand, written
when the table had one row. By its own reasoning — a writer that ignores its flag makes the
byte comparison pass while proving the wrong thing — that would have shipped this writer's
argument handling unasserted. Parametrized.

#### Verification

`cargo test` + `cargo clippy --all-targets -D warnings` clean; `ruff`, `pyright`, the Python
suite green. Controls: hand-edited manifest → red; drifted vector file → red; a file dropped
from the roster → red twice over; a wiring field renamed in **Rust** → manifest moves,
checker green, and in **Python** → checker red, manifest byte-identical. ⚠ The first attempt
at that last control mutated a string the registry does not contain and came back green — an
inert control, caught by checking the mutation rather than trusting the verdict.

Also corrected: `docs/authoring-reference.md` named the retired command in three places
including its own ceremony (nothing gates that file), and `tests/test_freeze_manifest.py`
carried a stray trailing line from the biosphere half that `ruff format --check` would have
reddened on CI. Full detail: `docs/plans/post-roadmap-reference-flip.md` §5n.

#### ⚠ Addendum — caught in review, none of it visible to a gate

A one-element `for` loop left where the *next* scheduled change empties it (C6's own
lesson, three commits old); the contract's prose still calling the two vector hashes "not
assertions" after this slice made them assertions (C3's finding a fourth time, and the
sentence was quoted in the plan as evidence); the new gate's **value** half claimed but
never controlled — the drifted-file control had run against the byte comparison, which
reddens anyway.

⚠⚠ And the delegation tie was first written against **a naming convention this repo
never adopted**: `<loader>.yaml` fits four of five loaders and is wrong for `thermal`,
which loads `radiator.yaml`. The gate reported an authored scenario reaching unfrozen
values and the tree was fine. Rewritten to ask the loader for its default path. **A
convention invented at the gate is not a property of the tree, and its first red looks
exactly like a real finding.** Full detail: plan §5n.

### C4b — the station's two science claims move to the reference (2026-08-18)

C7's station half was blocked on this: the station manifest's `science_bands` +
`liveness_floors` are two real claims, and a writer that cannot derive a claim must not
hand-carry it. Landed as its own commit — the order is load-bearing, not schedule.

#### ⚠ The prerequisite's own cost estimate was false when written

C7's record said `predicted_equilibrium_temperature` was Python-only. It was at
`rust/crates/station/src/system.rs:44`, with `mean_dissipated_power` beside it, the four
drift folds in `domains::biosphere::drift`, and the whole 15-yr energy run *with its
per-year peak fold* already in `emit_sealed_energy_drift.rs`. The claim was load-bearing —
it is why C4 split at all — and it was repeated in two contract docs. **A present-tense
claim about the tree went false and nothing re-reads it**, the C3 finding again. Corrected
in four places; the wrong line is left standing with the correction beside it, because the
point is that it stood.

#### Why C4b could not ride along with the writer

Moving two `locus` strings is a **value** diff to the contract. C7's station half is gated
by *regenerate and compare bytes*, and both prior halves got their result from
byte-identical on the first run. Bundled, that comparison cannot tell "the writer reproduces
the contract" from "the writer produced a file I just changed". So C4b regenerated through
the **existing Python writer**, via four lines deleted one commit later. Hand-editing the
manifest is the thing C7's biosphere half added a gate to catch — and the station gate does
not exist yet, so it would have stood silently.

#### ⚠⚠ Finding 1: the first regeneration silently dropped eleven keys

Predicted: two loci plus two `_authority` rows. Actual: that **plus 22 deleted lines**.

The reference's *dump* emits only scenarios that carry a claim, on purpose — the roster is
the manifest's hand-authored set and a program inventing keys would claim authority over a
set it cannot see. The biosphere **writer** fills the roster; the **dump** does not. The
Python census being replaced filled it. So splicing the dump's shape into the writer's slot
deleted the eleven `[]` entries — which on this contract `_authority` calls *"itself the
frozen claim"*, because 11 of 13 station scenarios carrying no outside-sourced bound is the
measured result. `[]` says *measured, none*; an absent key says nothing.

⚠ It also validates the ordering from the other side: bundled with the writer, those 22
dropped lines would have arrived **inside a "byte-identical" claim**.

#### ⚠⚠ Finding 2: the bound-literal check could not fail, and never could

The control — delete the assertion carrying the recorded number, expect the locus check
red — took **three attempts to bite**, and each failure was its own lesson:

1. `0.8814` → `0.88140`: green, because `contains` is a substring test. *An inert control,
   caught by checking the mutation rather than the verdict.*
2. `0.8814` → `0.8815`: green, because the `bound:` **record lives in the same file** and
   supplies its own number.
3. Subtracting the records' own occurrences: green for six biosphere literals, because the
   scanner's pin test quotes six real frozen bounds as test data.

So the rule as C4 ported it — *"every numeric literal in `bound` appears textually in the
file its `locus` names"* — was **true by construction**, and the `science_gates!` design
(declaration and `#[test]` are one thing) is exactly what guarantees it. The Python original
had the same defect and predates the flip.

Fixed with `code_only` — the source stripped of comments and string literals — so the number
must be in **executable** text. Measured after: 16 of 16 frozen literals appear in code,
eleven exactly once, and the control reddens on both tables. ⚠ The scanner's *own* first
draft failed its own negative: an up-front `src.contains("r\"")` raw-string guard fires on
ordinary prose ending a word in `r` before a quote, so the guard rejected the tree it was
written to protect. Pinned as a test.

⚠ The **checker's** copy was retired, not fixed, on a **narrow** reason: the rule needs the
locus file's syntax and every locus is now `.rs`, so the checker would need a second Rust
lexer written in Python. The broad reason ("the census is Rust's now") is the one this flip
has already recorded as too broad twice. Three replacements named in place.

⚠ A consequence for annotation style: quoting a bound's value in a comment beside its
assertion used to satisfy the check and now does not.

#### ⚠ Finding 3: a control that only becomes necessary when the data changes

The checker shells the dump with `text=True` and **no `encoding=`** — the Windows locale.
That is the mechanism that froze mojibake into the biosphere contract in C4 with every gate
green. The pin was added to the crossport reader then and not here, and that was *correct*:
nothing this dump emitted was above ASCII. C4b is the first slice to send an em dash through
it. Pinned.

#### Two process notes that nearly reached the contract

* **A mechanical re-wrap corrupted four prose strings and `ruff` passed on all four** —
  `manifest'ssingle`, `sentencethe`, `readthe`, `split,the`, inside `_AUTHORITY`, which
  *writes* a frozen artifact. Caught only by reading the regenerated diff.
* **`git checkout <file>` to revert a control discarded uncommitted work** — it threw away
  C4b's own edit to that file, visible only because of a `grep -c` afterwards. The safe
  shape is the `cp … .bak` pattern used for the Rust files.

#### Verification

`cargo test` + `cargo clippy --all-targets -D warnings` clean; `ruff check`,
`ruff format --check`, `pyright`, the full Python suite and the crossport suite green.
Controls: recorded literal's assertion deleted → bound-literal check red, gate green, on
both tables; a `science_gate` marker re-added in `tests/` → the census-exhausted gate red.
Measured: the node's annual peaks sit at 160.12 K against the frozen floor of 100.0 (1.6×
clearance). Full detail: `docs/plans/post-roadmap-reference-flip.md` §5o.

### C7 — the station half: the last writer moves (2026-08-18)

The third of C7's three halves, and the one that makes the flip's headline literally true:
**no Python program writes a frozen contract any more.** Blocked on C4b, which landed
first.

**Byte-identical on the first run** — third for three. The deliberate diff is three rows
and no value: `_comment` named the retired command; `aux_set`'s `why` said *"the splice is
what a regeneration writes"* and there is no splice now; `numerics_note`'s `why` documents
a hole this slice made bigger.

#### ⚠⚠ The finding: the trap is PARTIAL, which is worse than the biosphere's

`numerics_note` is prose naming three integration steps, and the writer now lives in the
crate that owns all three. Measured before the guard was designed:

| referent | as written | spliced | regeneration gate |
|---|---|---|---|
| `bio_dt` | `dt=1/4 day` | `dt=0.25 day` | **red** |
| `cabin_dt` | `dt=60 s` | `dt=60 s` | **green** — `60.0_f64` Displays as `60` |
| `power_dt` | `dt=3600 s` | `dt=3600 s` | **green** |

Two of three would auto-follow the code invisibly. Run end to end: splicing `cabin_dt` and
regenerating printed **`unchanged`**, and only the new source-text guard reddened.

⚠ **And unlike the biosphere there is no second guard.** That contract's `dt_days` is at
least compared against `BIO_DT` across the port boundary; this manifest has no structured
step key, and adding one widens the frozen surface — declined for the third time rather
than smuggled in as a rider.

⚠ The step unfreeze's lesson in a third place, with a new mechanism: there the collision
was between two *values*; here between a value and its **rendering**.

⚠ The guard's own control earned its keep the way the biosphere's did — the bare key
`numerics_note` appears **three** times in the writer (const, emission site, `_authority`
row), so an anchor on the key alone would check the wrong line. The ambiguity is asserted,
so the reason is a measurement rather than a claim.

#### Deleting the writer reddened nothing, and opened the same hole a third time

`_build_manifest` was reachable only from `_regenerate` ← `__main__`, invoked by no test.
**A control with no test to turn red IS the finding.** And the roster (`name -> label,
golden`) lived in the module *and was written from it*, so nothing held it once the writer
left — with the at-risk fields being exactly the two `_authority` marks `hand`.
`test_the_frozen_roster_is_the_references` closes it; a control confirms it reddens alone.

⚠ `_filed_under_the_roster` was **one commit old** when deleted — C4b added it, and the
Rust `census_json` does the same filling with the same panic-on-unknown-scenario.

#### The `_authority` literal went too, and the equality became a shape check

`manifest["_authority"] == _AUTHORITY` compared the committed block against the module's
own literal. The literal left with the writer; keeping a copy purely to assert against is
the stale second copy. The gates read the block out of the committed file now, and the
fourth check became shape checks — which catch a malformed row the equality never caught
either. The prose was moved **mechanically** (generated from the manifest and diffed),
never retyped.

#### Two sentences this slice falsified, both corrected in place

* `regen_goldens_from_rust.py`'s warning has now been corrected **twice, both times by the
  slice that falsified it**: C7's biosphere half made *"a `--write` that moves a frozen
  golden turns nothing red"* false for the biosphere and wrote *"it is still true for the
  station"*. Measured here — a moved station golden reddens the byte gate.
* `test_inventory_parity.py` printed the Rust regeneration command for the biosphere and
  the Python one for the station. **De-branched, exactly as its own comment scheduled.**
* And the ungated prose half: `docs/station-reference.md` named the retired command inside
  step 4 of its own unfreeze ceremony.

#### Verification

`cargo test` + `cargo clippy --all-targets -D warnings` clean; `ruff`, `pyright`, the
Python suite green. Controls: hand-edited manifest → byte gate red; drifted roster label →
roster gate alone; moved golden → byte gate red; an aux process wired in → the regenerated
manifest **gains the name** *and* the checker's aux gate reddens (the substitute for a
rename control this empty axis cannot run); the `numerics_note` splice → **manifest
unchanged, source-text guard red**, which is the finding rather than a pass. Full detail:
`docs/plans/post-roadmap-reference-flip.md` §5p.

### ⚠⚠ Addendum — a standing rule in `CLAUDE.md` went false, and nothing was watching

The always-loaded map has said, since the freeze contracts existed:

> **A provenance-only edit is an unfreeze that NOTHING CATCHES.** The per-file sha-256 is
> recorded but never compared, so editing just a param's `source:` moves the hash and
> turns nothing red.

**C7 falsified it and the falsification went unrecorded for two commits.** Every writer
hashes the files it compiles in, and `tests/crossport/test_manifest_writer.py` compares
the committed manifest byte for byte — so a `source:`-only edit leaves the manifest stale
and **red**. Measured on **both** contracts rather than reasoned from one: a probe edit to
`charge.yaml`'s `source:` reddens the station row, and one to `photosynthesis.yaml`'s
reddens the biosphere row.

⚠ The rule is corrected rather than deleted, because the *precise* claim it was making is
still true and still worth carrying: the hash is not asserted as a **value**, so a
provenance edit adds no evidence about anything. What changed is that it can no longer be
made *silently* — the regeneration is now forced, and only the **ceremony** around it
(advisor review, documentation) is honor-system.

⚠ This is [[posture-landed-in-claude-md]]'s finding a second time, and the shape is worth
naming: **`CLAUDE.md` is audited by nothing, so a claim in it survives exactly as long as
nobody re-reads it.** C3 found a stale `dt=1` there; this is a stale "nothing catches
this" — and both were load-bearing, because a reader acts on them.

### ⚠ A third process trap: a mutation control run WHILE a full suite is in flight

The clean suite is green, but the first run of it came back **19 failed, 4 errors** — and
none of them was a regression. Two provenance probes (`charge.yaml`, `photosynthesis.yaml`)
were edited and restored *while* `pytest -n 12` was executing, and the biosphere probe left
the file briefly unparseable, so every test that loads it failed. Both files were restored
and `git status src/` was clean by the time the run finished, which is exactly what makes
this dangerous: **the evidence of the cause is gone before the verdict arrives.**

The rule, alongside this slice's other two (`git checkout` discarding uncommitted work; a
mechanical re-wrap corrupting frozen prose): **a control mutates the tree, so it may not run
concurrently with anything that reads the tree.** Run controls against a targeted subset, or
wait for the full run to finish. A red suite whose cause has already been reverted is worse
than a red suite — it invites either a false regression hunt or, worse, a shrug.

---

### Stage 3 — the suite classification pass (2026-08-18)

A **measurement**, not a build: every one of the 174 Python test files given a verdict
against the reference, and no test file touched. Full tables and per-file rows:
`docs/plans/post-roadmap-reference-flip.md` §5q.

**The shape.** 2,452 collected Python tests / 57,974 lines against **445** Rust tests /
38,742 lines. The `domains` row carries the whole plan: 1,240 Python checks against 89.
Retirable with nothing else built: **10 files, 147 tests — 6.0 % of the suite.** Work that
is new Rust which does not exist: **92 files, 1,398 tests**, before the residue inside the 36
partly-covered files.

⚠ **The verdict table contradicted its own finding, and the review caught it rather than a
gate.** `test_crossport.py` and `test_inventory_parity.py` were filed *retire free* on the
plan's standing reasoning (*"their entire subject is the two ports agreeing"*) while the
finding directly below them said that reasoning is false for the sibling domains. A reader
works the table, not the prose, and the contradiction pointed toward deleting coverage. A
seventh code — **`R!`, retire only once its successor stands** — now carries those two rows.
*A finding that does not change the row it is about has not landed.*

#### ⚠⚠ Stage 3 does not begin with tests

The reference `include_str!`s **24 files out of the tree being deleted** — all 23 param
YAMLs under `src/`, plus C9's `tests/oracle/winter_wheat_weather.json` — and reads three
more at runtime (`tests/authoring/scenarios/`, 26 fixtures behind 40 Rust tests and
`godot_bridge`; `tests/regression/golden/state_snapshot.json`). **`rm -rf src/ tests/` does
not fail a test; it fails the build.** Relocating the data is the first slice, and it is
not a test slice.

#### ⚠⚠ No Rust test compares a run against a committed golden

Searched exhaustively. The reference *emits* the goldens (23 `emit_*` examples) and Python
alone compares them: 17 `test_regression_*.py`, `tests/golden_platform.py` (the policy
choke point), `test_golden_provenance.py`. The plan's own line — *"the regression/golden
gates mostly exist in Rust already"* — was false when written and is corrected in place.
The same shape holds for the **manifest** byte gate (`tests/crossport/test_manifest_writer.py`),
which is what arms C7's provenance trap: **C7 moved the writers and left every checker in
the dying tree.**

#### ⚠⚠ The checker is not a second opinion on the sibling domains — it is the gate

Control B, on a clean tree: swap the two legs of `charge_split` in `domains/src/power.rs`
so the battery stores `(1-η)` and loses `η`. Conservation still holds exactly — a pure
science error.

* `cargo test --workspace`: **2 of 445 red**, both in `godot_bridge` readout assertions
  that happen to pin a battery number. **Nothing in `domains` or `station` noticed.**
* `pytest tests/crossport/test_crossport.py -k power`: **3 red**, by name.

`crew.rs`, `eclss.rs`, `power.rs`, `thermal.rs` carry **0** `#[test]` between 1,411 lines;
Python carries 158. ⚠ *Zero markers is not "unexercised"*: the code is stepped from
`authoring`'s flow-registry test, from `station`'s builder and palette, and from seven
`emit_*` examples — and Control B measured exactly what that incidental exercise catches,
which is two front-end readouts and nothing that names the domain. So "delete the
cross-port comparison, its subject is the two ports agreeing" is true of its *mechanism*
and false of its *effect*: for these domains it is the only gate there is.

#### ⚠ A control corrected this entry's own draft

Control A disabled the extinction branch in `integrator.rs`. The draft finding read
*"implemented in the reference and untested there"*; **one test reddened** —
`engine_vectors.rs::engine_synthetic_trajectory_is_bit_exact`. The corrected claim is
narrower and more useful: extinction has no *direct* test in the reference, and is held by
a single bit-exact vector that pins the whole state, so it reports *that* something changed
and never *which* mechanism broke. ⚠ That vector is generated by a Python generator queued
for deletion.

#### ⚠ Name overlap is a lookup, never a verdict — and it lies in both directions

74 names match across the two suites. `test_observation.py` matches 1 of 13 yet its
subjects are all in `observation.rs` under names that dropped a prefix (false negative);
`test_authoring_monod.py` matches 4 and expands to **216** collected cases against 6 Rust
tests (false positive). Related: `grep -c '#[test]'` returns **455**, the parsed index
**445** — the ten-item gap is `#[test]` written inside doc comments. *A grep of a marker is
not a census when the marker is also prose.*

#### ⚠ Things guarded by nothing, on either side

*"`simcore` carries zero third-party deps"* — the Python purity tests scan Python packages
only; no Rust test reads a `Cargo.toml`. *"`gdext` appears in `godot_bridge` and nowhere
else"* — one matching line in the tree, and it is a doc comment. Both are `CLAUDE.md`
non-negotiables. Retiring the Python guards costs no coverage because there is none; it
removes the last thing that *looks* like a gate.

Also named: seven Rust tests whose oracle is the dying side (`canonical_units_match_python_table`
and friends); 725 of the 2,452 cases come from `parametrize`, so a function-for-function
port silently narrows; the nine Godot files are Rust-vs-Rust-in-Godot and are Phase 8's
exit criterion rather than port-parity; and C1's *"take the user's harness with it"* never
happened — `src/config/overrides.py` has no Rust counterpart, so Stage 3 collides with the
value-switch plan.

#### The order this implies — six slices, the first two not about tests

**S1** the reference's own ground (relocate the data) → **S2** the three gates that check
the contracts (manifest bytes, goldens, dumps) → **S3** the four sibling domains → **S4**
the engine residue (extinction, aux, environment, integrator, multirate, the two ungated
invariants) → **S5** the ~600 biosphere mechanism tests, in batches → **S6** the
retirements, and only then. The four **D** rows (the Godot drivers, `test_headless_cli.py`,
`test_context_budget.py`) are decisions the user owns and S6 is blocked on them.
⚠ `test_context_budget.py` is the one file with no home on either side: its subject is this
repo's own documentation discipline, so "port to Rust" is a category error and deleting it
removes the only enforcement the context budget has ever had.

#### Verification

Both controls run against a clean tree with nothing else in flight, both files restored
with `git checkout --`, workspace re-verified green afterwards — §5p's third process trap,
observed. `git diff` is empty for `src/`, `rust/`, the goldens and all three manifests.
Inventories: `M:/claud_projects/temp/stage3-sorting/`.

### S1 — the reference's own ground; the compile-time half (2026-08-18)

Plan: `docs/plans/post-roadmap-reference-flip.md` §5r.

Stage 3's FINDING 1: `rm -rf src/ tests/` would not fail a *test*, it would fail the
**build** — `rust/` compiled 24 files out of the tree being deleted. S1 splits along that
dependency. This half moved the two compile-time reach-outs: **23 frozen param YAMLs** (plus
`demo.yaml` and the four `crops/potato/` overrides — 28 files) into
`rust/crates/{domains,station}/params/<domain>/`, and `winter_wheat_weather.json` into
`rust/crates/domains/data/`. Every one is a git-recorded 100 %-similarity **rename**.

**The slice's category was a measurement, not a judgement.** All three manifests were grepped
for path fragments before anything moved: `param_files` is basename-keyed, `scenarios/*`
records a golden basename, `parity_vectors` likewise — **zero path hits**. So the move is a
*pure rename*, no unfreeze ceremony, and the byte gate proves it instead of the plan asserting
it. `.gitattributes` was checked for the same reason: it is global (`* text=auto eol=lf`), so
the new home inherits identical normalization — which matters because `include_str!` embeds
the **working tree** and one frozen file is CRLF on this box.

**⚠⚠ The whole directory moved, and that was forced by a control rather than chosen for
tidiness.** `a_recursive_walk_reddens_the_census` proves *"a directory is not a category"* by
asserting the recursive walk sees exactly four more files than the census. That assertion has
teeth **only because the four potato overrides sit in a subdirectory of `PARAMS_DIR`** — take
the fifteen frozen files and leave the rest, and the control stops measuring a hazard the tree
no longer contains. `demo.yaml` came for the parallel reason: it keeps the exclusion-**by-
name** rule true verbatim, so three literals (`param_files()`, the dump's `assert_eq!(15)`,
the census test's own count) stay correct with no value change. Both die at S6, inside a
retirement, where the rules dissolve deliberately.

**⚠ A negative assertion about a directory goes vacuous when the directory moves.**
`test_mineralization.py` asserts a retired param file does *not* exist there. A directory that
does not exist satisfies that silently, so a mis-resolved path would have turned a real check
into a green no-op. The positive half now runs first. The general question — *what does this
assert if the path is wrong?* — is the only thing separating a live negative from a vacuous
one, and it has to be asked of every re-pointed path, not just the suspicious ones.

**The two-direction control:** with `src/` and `tests/` renamed away, `cargo build` **succeeds**
(false this morning) and `cargo test` **fails**, its panics naming `scenario_files.rs` and
`snapshot.rs` — exactly the runtime reach-outs the second half owes. The control is also a
to-do list that cannot be padded.

**Python became a tenant.** Six loaders spelled their own `Path(__file__).parent / "params"`
and 40 test modules spelled the weather fixture; `src/config/paths.py` now holds that climb
once. Priced consequence, stated rather than discovered: the Python packages no longer carry
their own data, so a *non-editable* wheel would ship loaders without params. The project
installs editable and the checker was already checkout-only, so nothing breaks — but "the
Python tree is installable standalone" stopped being true.

**The deliberate manifest diff was two prose strings, predicted then measured.** Two
`_authority` `why` entries described this slice in the **future tense** — the repo's
most-repeated failure mode. `forcing/weather_fixture`'s own text said its name becomes
derivable once the relocation lands. ⚠ S1 met that condition **and deliberately did not act on
it**: deriving the name flips the key `hand → rust`, which is a re-anchoring, and taking it
inside a slice claiming *data moved, authority did not* would make the byte-neutrality claim
unfalsifiable. Named as the successor in the manifest itself.

**Found while measuring, left for the second half:** the golden count is stale in three places
(plan, `golden_platform.py`, `CLAUDE.md` all say 25; disk holds 21 since C6 deleted four).

#### Verification

`cargo build` / `cargo test` / `cargo clippy --all-targets` clean; `uv run pytest -n 12` →
2,447 passed, 5 skipped (13m08s). `git status` on `tests/regression/` and on the other two
manifests is empty — byte-neutrality in its checkable form.

### S1 — the runtime half; FINDING 1 discharged (2026-08-18)

Plan: `docs/plans/post-roadmap-reference-flip.md` §5s.

The 26 authored-scenario fixtures and the 21 regression goldens moved to `rust/data/`. The
control §5r left standing was re-run: with `src/` and `tests/` renamed away, `cargo build`
**and** all 30 `cargo test` binaries now pass. FINDING 1 — *"`rm -rf src/ tests/` does not
fail a test, it fails the build"* — is no longer true of either half.

**Workspace-level, not crate-local, and the rule did not change to get there.** Put the data
where the dependency is: the fixtures are read by `authoring`'s tests *and* by `godot_bridge`,
and the goldens are emitted by `emit_*` programs in **four** crates. Neither can sit inside one
crate without the others reaching into its private tree — the exact thing this slice exists to
stop. ⚠ Deliberately not the repo-root `scenarios/`, which is authored *content*.

**⚠⚠ The finding: the golden census prose was stale in two directions at once, and nothing
gates it.** Re-measured from disk rather than copied: **21 goldens, 19 Rust-authored, 2
Python-authored**. Four files said otherwise — `golden_platform.py` (*"Eighteen of the
twenty-five"*), `regen_goldens_from_rust.py`, `test_golden_provenance.py` and **`CLAUDE.md`**.
Both halves had rotted independently: **18 → 19** when C5 folded the station drift summary in
Rust, **25 → 21** when C6 retired four Python-only goldens.

Nothing was broken and nothing was ungated — which is why it survived. The *rosters* are
derived and were right throughout; the census test enumerates from the directory precisely
because this repo has been caught trusting a hand-maintained list. What rotted is the layer no
gate owns: the sentences a reader uses to orient. Fourth instance of this family in the flip,
and the first with the stale number sitting in the always-loaded `CLAUDE.md`.

**So the fix is not just the four corrections.** `test_golden_provenance.py` gains
`test_the_golden_census_counts_are_what_the_prose_says` — two counted literals `(21, 19)` whose
failure message **names the four prose sites**. A forcing function, not a second census: it
cannot check that a sentence is true, only that somebody looked when the count moved. Stated as
such so nobody later "simplifies" it into a derived check that would rot silently.

**Control, run before the gate was believed:** one name added to `RUST_AUTHORED` → red, and the
message names the prose sites; reverted → green. ⚠ Reverted by an in-place reverse edit from a
copy in the temp tree, **never `git checkout`** — that file held uncommitted work, and this
flip has already paid for that mistake once.

**Worth naming: the two Godot `.gd` scenario constants.** Plain strings, resolved at runtime by
the editor, no compiler sees them, and the tests exercising them are `skipif`-ed on CI. They are
the easiest thing in the repo to move and not notice.

#### Verification

`cargo test` / `cargo clippy --all-targets` clean; `uv run pytest -n 12` → 2,448 passed, 5
skipped (11m40s). All three manifests byte-identical — they record hashes of golden **content**,
and the writers read the same bytes from a new directory. All 47 moved files are git-recorded
100 %-similarity renames.
