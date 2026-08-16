## **The reference flip — Rust becomes canonical** (target state B; eleven slices, six landed)

Plan: `docs/plans/post-roadmap-reference-flip.md`. **Planned 2026-08-16 in eleven
independently-landable slices**, on the user's explicit instruction (*"only plan now, work in
different slices. Don't bundle the whole work into one slice"*). **Slices 1–6 landed the same day.** The reference has
moved: two goldens are Rust's bytes, the cross-port contract is inverted, and the biosphere
manifest is re-anchored — with **mixed** authority, stated per key in the file itself. `git
diff src/` stays empty throughout.

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

### The state of the arc

Slices 6–11 are an unexecuted menu; the user takes them one at a time and none is scheduled.
The ordering that matters: **1–3 built nothing the reference depends on** (they de-risked the
export and proved the port can express the completeness contract *before* anything
re-anchors). **Slice 4 was to be where the reference actually moves — and the measurement was
that it barely had to**: sixteen of eighteen goldens were already byte-identical between the
ports, so what 4 could land alone was the *path* and the *census*, not a diff. **Slice 5 wrote
the other two, and with them the inversion**: the golden is now Rust's artifact, the Rust
byte census is unconditional, Python is the tolerance-gated side, and Python can no longer
author any of the eighteen. **6–8 are the three unfreeze ceremonies, biosphere
first; 6 has landed.** Its lasting finding is that a freeze contract does not re-anchor as a
unit: roughly half the biosphere manifest has no Rust referent and is now declared
Python-retained *in the file*, so **slices 7 and 8 should expect to classify, not to
convert**. Until they land, the station and authoring contracts are still Python-anchored, new
reference science outside the biosphere is still Python-canonical, and a science item must
never share a batch with a re-anchor slice.

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
