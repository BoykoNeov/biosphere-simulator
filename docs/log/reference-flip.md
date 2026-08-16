## **The reference flip — Rust becomes canonical** (target state B; eleven slices, the first two landed)

Plan: `docs/plans/post-roadmap-reference-flip.md`. **Planned 2026-08-16 in eleven
independently-landable slices**, on the user's explicit instruction (*"only plan now, work in
different slices. Don't bundle the whole work into one slice"*). **Slices 1 and 2 landed the
same day.** Nothing else is built: no golden regenerated, no manifest re-anchored, no band
moved, `git diff src/` empty.

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

### The state of the arc

Slices 3–11 are an unexecuted menu; the user takes them one at a time and none is scheduled.
The ordering that matters: **1–3 build nothing the reference depends on** (they de-risk the
export and prove Rust can express the completeness contract *before* anything re-anchors);
**slice 4 is where the reference actually moves** and must not be taken before slice 3 passes;
**6–8 are the three unfreeze ceremonies, biosphere first.** Until 6–8 land, new reference
science is still Python-canonical — and a science item must never share a batch with a
re-anchor slice.

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
