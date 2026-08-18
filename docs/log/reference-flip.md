## **The reference flip — Rust becomes canonical** (target state B → C; eleven slices, eight landed, then C1, C2, C5, C8, C9, C3 and C6 of the C re-plan — the param load, the twelve laws, the drift folds, the param-file list and the weather path all moved into the reference 2026-08-17, the posture itself into CLAUDE.md 2026-08-18, and the four Python-only scenarios retired the same day)

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
