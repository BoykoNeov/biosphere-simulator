# Post-roadmap: the reference flip — Rust becomes canonical

**Status: SLICES 1–5 COMPLETE (2026-08-16); 6–11 are an unexecuted menu.** ⚠ **Read the
status column of the §5 table, not this line — that table is the one place slice state is
recorded, and this header was already false within a day of being written** (it said
*"NOTHING BUILT / `git diff rust/` empty"* through the whole of slice 1). The invariant
that still holds: **`git diff src/` empty.** ⚠ The other three retired with slice 5 —
**two goldens have been regenerated from Rust, the biosphere manifest has been re-anchored
to them, and `tiers.json`'s stale band figures have been corrected** (no band *value*
moved). Everything in §2 is a measurement taken on frozen `main` on 2026-08-16 and is
*not* re-taken as slices land.

**The decision.** The user re-opened the Rust question on 2026-08-16 ("let's discuss again
the complete switch to Rust") and, given the fork below, chose **target state B of
`post-roadmap-rust-primary-pivot.md` §3 — flip the reference: Rust becomes canonical,
Python becomes the checker.** I priced B's cost and named what it gives up (§4); the user
took it anyway. That is their call and this doc executes it. The cost is recorded once, in
§4, as a **documented consequence — not an argument to be re-run at each slice.**

**The user's second instruction, which shapes this whole doc: *"only plan now, work in
different slices. Don't bundle the whole work into one slice."*** So §5 is eleven slices,
each landable on its own, ordered so the two genuinely unknown pieces are de-risked before
anything with blast radius moves. There is deliberately **no big-bang slice**.

---

## §1 Why the question came back — the pivot's premise was empirically false

Option A (2026-07-20) split work by *type*: new content and gameplay land Rust-first and
owe no Python mirror; validated science stays Python-canonical and mirrors to Rust. The
plan doc justified the split with an estimate: *"~90 % of remaining work is **content +
gameplay** … the remaining ~10 % is residual science calibration."*

**Measured on 2026-08-16, over the 157 commits since that decision:**

| Area | Commits / files touched since 2026-07-20 |
|---|---|
| `godot/` (the product front-end) | **0 commits** |
| `scenarios/` (authored content) | **1 commit** |
| `src/domains/` (Python science) | 122 file-touches |
| `rust/crates/domains/` (the mirror) | 71 file-touches |
| `tests/`, `docs/` | 463, 323 file-touches |

The mix was not 90/10. It was approximately **0/100**. ⚠ **Rule 1 — "new content is
Rust-first" — has never once fired, because no work of the kind it governs has been done.**
The project has spent a month running in exactly the *"both ports, Python-first"* mode the
user declined when A was chosen, not because the policy changed but because **the policy
only had opinions about work that wasn't happening.**

That is the honest reason the question returned, and it reframes it: A was not *wrong*, it
was **inert**. The felt cost is the double build on every science item, which A's rules
never addressed because A assumed science items would be rare.

⚠ **A related miss, recorded because it explains why the posture never became a default:**
§6 step 1 of the pivot plan called landing Rules 1–5 in `CLAUDE.md` *"the single most
important step; the rest is tooling."* **It was never done.** `CLAUDE.md` has no
development-posture section; the posture lives only in that plan doc and a memory file. A
standing posture rule is squarely the *"what you need before you know what you're working
on"* category `CLAUDE.md` exists for — structurally distinct from the finished-work rows
`docs/context-budget.md` retires from it. Slice 11 closes this, for B's posture rather than
A's.

## §2 What was measured today — and two of my own worries, retracted

Every number here was taken on frozen `main` with nothing built. **Two of the three costs I
led with were refuted by measurement, one was confirmed and one was found that neither the
old plan nor the advisor had priced.**

### 2a ⚠ RETRACTED: "regenerating the goldens from Rust will disturb the science gates"

I raised this as the reason B might be expensive, on the reasoning that the cross-port
contract is a **tolerance** contract (so the ports are *not* bit-identical), and that
several science gates now sit at sub-percent clearance — the perennial liveness floor is at
**0.40 %** after the canopy-provenance work.

**Refuted by the contract's own measured bands** (`tests/crossport/tiers.json`,
`docs/native-port-reference.md`): the Tier-2 bands are **`1e-11` to `1e-12` relative**, with
a `1e-12` floor throughout — the two ports agree to about **eleven significant figures**.
The tightest science gate in the tree is `4e-3`. Regenerating from Rust therefore perturbs
the goldens roughly **eight orders of magnitude below anything that can go red.**

**Numerically, the flip is nearly free. The entire cost of B is structural.** This is the
single most decision-relevant fact in this doc and it points *toward* B, not away.

⚠ Do not over-read it: the bands are the **measured ±1-ULP transcendental sensitivity**
plus margin, and CI is the genuine cross-libm gate (Linux glibc Rust vs UCRT-generated
goldens). "Eight orders of margin" is a statement about *magnitude*, not a licence to skip
predicting the diff before regenerating (`soil-layers`' own lesson).

### 2b ⚠ RETRACTED: "the completeness gates can't port, because Rust has no introspection"

The three freeze manifests' gates (1,367 lines across
`test_freeze_manifest.py` / `test_station_freeze_manifest.py` /
`test_authoring_freeze_manifest.py`) own **completeness** — something added to the tree but
exercised by nothing. I assumed they introspect the Python *module namespace*, which Rust
cannot do.

**They don't.** `_flow_set()` (`tests/test_freeze_manifest.py:183`) is explicitly
*"derived, never hand-listed"* — it **builds the canonical registries and reads
`registry.flows`**, taking `type(flow).__name__`. `_aux_set()` is symmetric over
`registry.aux_processes`. That is a **runtime enumeration of what is actually wired into a
built scenario**, and Rust can do it identically: `rust/crates/simcore/src/flow.rs:81`
already declares `trait Flow { fn id(&self) -> &str; }`, and `auxiliary.rs:29` the same for
aux.

**Measured derived-vs-frozen ratio across the three gates** (the number the advisor
correctly identified as the real size of this item):

| Gate | Tests | Assertions against a derived-from-tree helper | Deliberately hard-coded literals |
|---|---|---|---|
| biosphere | 14 | 8 | **2** |
| station | 10 | 7 | 0 |
| authoring | 11 | 8 | 0 |
| **total** | **35** | **23** | **2** |

The two anti-derived sites are both in the biosphere gate and both concern the time step
(`tests/test_freeze_manifest.py:254` *"Do not 'simplify' either one to import `BIO_DT`"*;
`:479` *"⚠ KEEP THESE HARD-CODED LITERALS"* — a gate that imports its own value from the
code auto-follows the code, which is the opposite of a gate). Those get **re-authored by
hand** against the Rust tree, as judgements about what the literal guards. Everything else
is mechanical.

⚠ So this item is *a real rewrite with two judgement calls in it*, **not** 1,400 lines of
new thinking. The advisor's concern was right in kind and small in size, and I had it
backwards in the other direction an hour earlier — **both readings were corrected by
reading the file rather than reasoning about the language.**

### 2c CONFIRMED and larger than the old plan said: the crop-model comparison

`src/lab/oracle_match.py` — the comparison against WOFOST/PCSE — is **pure arithmetic over
`Sequence[float]`** (`nrmse`, `max_abs_relative_deviation`, `within_band`). No engine
objects, no Python state. That half ports or is simply re-called, freely.

**But its callers do not.** `tests/test_oracle_gap.py:103,175` call `build_season()` and
walk the season **in-process, day by day**, building the candidate series in memory; the
reference is a committed JSON `["trajectory"]`. Rust's 24 `emit_*` examples produce
**final-state snapshots**, not day-by-day histories.

⚠ **So B needs a per-step trajectory export from Rust that does not exist today.** That is
a new interface, not plumbing — and it is the one item in this plan where I would expect
surprises. It is why slice 1 exists.

### 2d The PCSE laboratory is banked for running, live for minting — and was woken 5 days ago

`pyproject.toml:29` — `oracle = ["pcse"]`, an **optional** dependency group behind an
opt-in `-m oracle` marker, `importorskip`-guarded, skipped cleanly when absent. The
everyday suite runs entirely off committed traces (`winter_wheat_reference.json`,
`spring_wheat_reference.json`, `potato_reference.json`). **Minting a new trace needs live
Python**, and one was minted **2026-08-11** for the potato.

This was the A-vs-C hinge and it is recorded here only as context: **B does not retire
Python**, so the laboratory survives the flip. It does, however, make 2c load-bearing — the
laboratory now has to compare *Rust's* trajectory against those traces.

### 2e The trap: unit validation becomes a green test guarding nothing

Python's `config/` boundary validates units (pint) as it loads every param YAML and hands
the core a canonical-unit label. **After the flip Rust is the loader for the canonical
run.** Leaving the Python check in place as a lint over the same files leaves it *passing*
while the path that actually executes is unvalidated.

⚠ That is precisely the defect shape already in this project's log — *the pin that should
have seen it had been reading one registry twice since the day it was written*
(`docs/log/step-unfreeze.md`). **This needs a decision, not a leave-in-place**, which is
why it is slice 9 and not a footnote.

### 2f The other four items, priced and small

- **The 25 goldens**: `rust/crates/{domains,station,authoring,simcore}/examples/` carry
  **24** `emit_*` programs against **25** golden files. One is missing or one program emits
  two; identifying which is slice 4's first act.
- **The two-ports-agree check**: `tiers.json` + the comparator invert as configuration.
- **The mathematical-law tests**: **12** `@given` sites across 13 files — conservation,
  non-negativity, order-independence. Rust carries **295** tests already and has **no**
  `proptest` dependency today. Small. ⚠ Note the "56 k lines of Python tests" figure is
  real but is overwhelmingly worked-example, regression and science-gate tests, **not**
  laws; quoting it as the reimplementation bill would be wrong.
- **The class-name / instance-id mismatch**: Python freezes `type(flow).__name__` (a
  **class**); Rust's `Flow::id()` is an **instance** identifier — one class may be
  instantiated per compartment with distinct ids, so they are not the same set. Rust needs
  a `type_name()` on the trait. ⚠ **That is a change to the frozen Rust core, so the
  opening move of making Rust the reference is an unfreeze of Rust.** Not a blocker; worth
  seeing.

## §3 What B is, precisely — and what it is not

**Is:** the goldens are generated from Rust; the three freeze manifests are equated against
the **Rust** tree; the cross-port contract inverts (Python is now judged faithful to Rust);
new *reference science* is authored Rust-first under the unfreeze discipline.

**Is not:** a retirement of Python (that is C, and 2d keeps it off the table while the
laboratory can still mint traces); a change to any scientific result (§2a — nothing moves
above `1e-11`); or a licence to stop keeping Python green (§4).

## §4 The cost, recorded once

**After the flip the two implementations stop being independent.** Python is fitted to
Rust rather than checked against it, so a disagreement can no longer arbitrate a shared
conceptual error — it is resolved in Rust's favour by definition.

**What that has cost historically, concretely:** the scope-B increment-1 mirror caught a
year-2 vernalization reset bug; the multi-rate phase's zero-coverage driver was caught the
same way; and **post-pivot, 2026-08-12, the stem-reserve mirror caught a scenario constant
the entire Python suite could not see** (`docs/log/stem-reserves.md`). That third one is
the important one: the mechanism was still paying out four days before this decision.

**This is a documented consequence of a decision the user made with the cost stated. It is
not re-litigated per slice.** What survives the flip and must be kept: Python stays green
(A's Rule 5 applies unchanged, with the roles swapped), so a *disagreement* is still
detected — what is lost is the ability to conclude that **Python** is the correct side.

## §5 The slices

Each slice lands on its own, has its own commit(s), and is listed with its blast radius.
**Slices 1–2 build nothing that the reference depends on** — they exist to de-risk 2c and
2f before anything with a contract behind it moves. Nothing here is scheduled; the user
takes them one at a time.

| # | Slice | Depends on | Blast radius | Reversible? |
|---|---|---|---|---|
| 1 | **Rust per-step trajectory export** — **COMPLETE 2026-08-16** | — | additive; no contract | yes |
| 2 | **`type_name()` on `Flow`/`Aux`** — **COMPLETE 2026-08-16** | — | frozen Rust core (small unfreeze) | yes |
| 3 | **Rust dumps its own inventory, checked against the *existing* manifest** — **COMPLETE 2026-08-16** | 2 | additive test only | yes |
| 4 | **The golden census + the Rust-side regeneration path** — **COMPLETE**; the 2 divergent goldens were regenerated by slice 5 | **3** | 2 new test files; **no golden moved** | yes (git) |
| 5 | **Invert the cross-port contract** — **COMPLETE 2026-08-16**; the 2 stragglers landed, and the biosphere manifest ceremony came with them | 4 | `tiers.json` + comparator + 4 byte-exact Python gates | yes |
| 6 | **Re-anchor the biosphere manifest to Rust** — **COMPLETE 2026-08-16**; mixed authority, no frozen value moved | 3, 5 | freeze contract 1 | ceremony |
| 7 | **Re-anchor the station manifest** — **COMPLETE 2026-08-16**; mixed authority, no frozen value moved | 6 | freeze contract 2 | ceremony |
| 8 | **Re-anchor the authoring manifest** | 6 | freeze contract 3 | ceremony |
| 9 | **Unit validation: decide and build** | 6 | `config/` + Rust loader | design decision |
| 10 | **The 12 laws get Rust equivalents** | — | additive (`proptest`) | yes |
| 11 | **Purity invariant re-written; posture landed in `CLAUDE.md`; memory + log** | all | docs + one `CLAUDE.md` section | yes |

**Slice 1 — the per-step trajectory export (do this first).** 2c is the only unknown with
real surprise potential, and it is *additive*: Rust gains a way to emit a per-step series,
nothing consumes it yet, no contract notices. Acceptance: the exported series for one
biosphere scenario reproduces the Python in-process series inside the scenario's own
Tier-2 band. ⚠ Design question to settle **inside** this slice, not before: which
quantities, and at what step granularity (the step is now `dt = ¼`, so a "day" is four
rows — and `docs/log/step-unfreeze.md` records seven distinct days-vs-steps conflations
found the last time this distinction was assumed rather than checked).

### Slice 1 — COMPLETE 2026-08-16

**Built.** `simcore::snapshot::TrajectoryWriter` (+ `TRAJECTORY_VERSION`), the
`emit_trajectory` example in `crates/domains/examples/`, and one new parametrized gate,
`test_rust_trajectory_matches_python_step_for_step` in `tests/crossport/test_crossport.py`.
`git diff src/` empty. No golden regenerated, no manifest touched, no existing example's
bytes changed. Ruff / pyright / `cargo clippy --all-targets -D warnings` / the full
`cargo test` suite all green.

**The two design questions the slice was told to settle, settled:**

* **Which quantities → all of them, as a full snapshot per row**, in the frozen
  `SCHEMA_VERSION` shape `State::to_json` already emits for a final state. Repeating each
  stock's metadata on every row costs ~3/4 of the payload (6.75 MB for the season, 17.8 MB
  for the perennial case) and is the right trade anyway: **the export inherits the
  cross-port interchange contract instead of opening a second one.** Python `sim_io.loads`
  validates every row and the comparator needed no new code. Choosing a subset would have
  meant guessing what the eventual consumers need.
* **Granularity → every step, and deliberately no stride knob.** Down-sampling to days is a
  *consumer* concern with exactly one blessed implementation (`tests/day_index.py`, which
  exists because the idiom was invented five ways in five files first); a stride argument on
  the emitter would put a second one on the far side of the port, out of that module's
  reach — the days-vs-steps hazard this slice was warned about, re-introduced by hand.
  Every row carries its own `n`, so a consumer that down-samples can prove which steps it
  kept.

**⚠ The acceptance criterion as written would have left the reset path unproven, and it was
widened (advisor).** `emit_season` runs `run_season` with **no reset**, so a season-only
slice never exercises the reset hook — and the reset path is the one place the two ports'
observer semantics could genuinely differ, because *both* drivers record the **pre-reset**
state and never the reset instant itself (verified by reading both, then measured). So the
example takes a scenario argument and the gate has two rows: `season` (open field, 1 yr,
1221 rows) and `perennial` (`run_perennial`, reset armed, 2441 rows).

**⚠ 2 years, not the golden's 5, and the number is load-bearing:** the driver consults the
reset hook with the *pre-step* `n`, so a 1-year run checks `n = 0 ..= season_steps - 1` and
the boundary is never reached. Two years is the smallest horizon that fires it. That case
is therefore compared to Python only, never to `perennial_chamber_state.json`; the season
case additionally anchors its last row to `season_euler_state.json`, which is what proves
the trajectory comes from *that* run and not a differently-configured one.

**Two things keep the gate from being inert, and both were checked by measurement rather
than assumed:**

1. **`compare.py` matches list elements positionally and only checks list *length*** — two
   equal-length series shifted the same way (a missing initial state, an off-by-one
   observer) compare clean. Each row is self-identifying via `n`, so the gate asserts the
   sequence is `0..steps` on **both** sides. Measured: deleting the initial row is caught;
   without the assertion it would not be.
2. **The perennial case asserts the reset actually fired inside the exported window**, by
   the one signature the reset leaves in the data — `thermal_time` is non-decreasing within
   a season and only the reset lowers it. Measured: exactly one drop, at row 1220, in both
   ports. A perennial case whose reset never fired would be an expensive duplicate of the
   season case.

**What it bought, measured.** The season comparison walks **62,272 numeric leaves**, against
~51 for the existing final-state test on the same scenario. Negative control: a **1-part-per-
million** perturbation of a single stock in a single middle row fails the Tier-2 band. Every
cross-port biosphere comparison until now was on a **final** `State` — two ports could in
principle have reached the same endpoint along different paths and nothing would have said
so. Marked `slow` (≈2.4 s per case against a ~24 ms fast-loop average).

**⚠⚠ What this gate is NOT, recorded so nobody counts it twice.** It **borrows** the Tier-2
band rather than exercising it, and it is **not cross-libm coverage**. Every golden
comparison beside it is genuinely cross-libm on CI — glibc-Rust against UCRT-*generated*
goldens — whereas this one compares glibc-Rust against glibc-CPython **in the same
environment**, so both sides call one libm and the deviation is ~0.0 by construction on
either platform. That is correct for what it tests (the *shape of the path*, not the last
ULP), but a pass says nothing about whether the band is wide enough. The repo's own
precedent is the reason this is written down: a Tier-2 sensitivity probe measured exactly
`0.0` for weeks after the code moved out from under it and kept passing against nothing.
⚠ It does run on CI — the `crossport` job runs the whole directory with no `-m` filter, by
design — so the `slow` marker here is not a green-by-skip.

**Not done here, on purpose: the oracle is not wired to it.** 2c's consumer — feeding
Rust's trajectory to `oracle_match` — needs the matched-DVS / organ-basis plumbing, which is
real judgement work and a later slice's. Slice 1 is the interface only, and nothing consumes
it yet.

**Slice 2 — `type_name()` on the traits.** Small, and the first frozen-Rust unfreeze. Add
`fn type_name(&self) -> &'static str` to `Flow` and the aux trait, implemented so it
matches the Python class name exactly. Acceptance: no golden moves (it is a pure addition),
`cargo clippy --all-targets -D warnings` clean. ⚠ Use `rustfmt <file>`, **never bare
`cargo fmt`** — it reformats the frozen simcore tree.

### Slice 2 — COMPLETE 2026-08-16

**Built.** `Flow::type_name` (`crates/simcore/src/flow.rs`) and `AuxProcess::type_name`
(`auxiliary.rs`), **required, no default body**, implemented across **58 impl sites** in 11
files. Three new tests in `crates/domains/tests/type_identity.rs`, one in the station
perturbation tests, one in the biosphere perturbation tests. `git diff src/` empty; nothing
outside `rust/` touched; the diff is **246 insertions, 0 deletions**. `cargo clippy
--all-targets -D warnings` clean, the whole `cargo test` suite green, and the 138 Python
cross-port + three-manifest gates green. ⚠ **Stated precisely, because the two claims are
not the same evidence:** what was established is that **no file under
`tests/regression/golden/` changed on disk** (`git status`), which is the direct check. The
138 passing gates *compare against* those goldens, so a pass is consistent with a
comparison path that never ran — the weaker of the two facts, and the one it would be easy
to quote as if it were the stronger.

**⚠ The acceptance criterion as written was passable by a no-op, and was widened
(advisor).** *"No golden moves, clippy clean"* is satisfied by a method **nobody ever
calls** — the same defect shape slice 1 recorded one day earlier (a gate that borrows a
band it never exercises) and the same shape as the sensitivity probe that read `0.0` for
weeks. So the slice's real criterion became: `type_name()` is exercised **through
`Box<dyn Flow>` out of a built canonical registry** — the only path that matters, because
that is how Python derives `flow_set` — and every assertion has a **measured negative
control**. ⚠ Generalize: *an interface slice's acceptance criterion must name a call site,
or it accepts an interface nobody has run.*

**⚠ The design decision, and it went AGAINST the cheap option: required, not defaulted.**
A default body of `std::any::type_name::<Self>()` (path-stripped) compiles, keeps the trait
object-safe, dispatches correctly through `Box<dyn Flow>`, and would have cost **zero** per-
impl lines. It was rejected on one asymmetry:

* **Rename drift under hand-written literals is caught by slice 3** — the manifest says the
  old name, Rust reports the new one, red. The auto-derive's whole advantage expires one
  slice from now.
* **Compiler-version dependence is caught by nothing.** `std::any::type_name`'s own docs
  disclaim any guaranteed output format, and from slice 6 this string is a value a **freeze
  manifest is anchored to**. A toolchain bump turning a manifest gate red for a non-science
  reason is a failure mode this repo has no mechanism for.
* A **required** method makes a new `impl Flow` a **compile error** until its author states
  the contract identity. Under B, adding a flow to the reference *is* an unfreeze event, so
  putting the author in front of that is worth more than the 58 saved lines.

**⚠ The required method immediately paid for itself: it found impls the grep did not.** The
anchored search found 54 sites; the compiler found **4 more** nested inside `mod tests`
blocks (`registry.rs`, `inspection.rs` ×3). A defaulted method would have silently given
those four whatever `std::any::type_name` returned. ⚠ Also: my own hand-count of the Flow
impls was **48**; the actual figure is 49 + 5 aux + 4 nested = 58. *A count taken by eye off
a grep is a guess; the compiler is the census.*

**Measured, and it settles a question slice 3 would otherwise have to ask.** The union of
`type_name()` over the four canonical biosphere builds is **exactly 23 flows / 3 aux, name
for name identical to the frozen Python manifest** — so Rust can already express the
completeness contract, before slice 3 builds the gate that asserts it. Per scenario:
11 / 19 / 19 / 22 flows, **each type instantiated exactly once** (no canonical biosphere
build wires a type twice).

**What is deliberately NOT here: the roster.** No list of the 23 names appears in Rust. That
comparison is **slice 3's**, in Python, against the manifest file itself; a second copy here
is precisely the *"a rule with two copies has one that is stale"* hazard. The union
cardinality is left out for the same reason — it would be a third place to edit when a flow
is added. What the Rust tests own is what slice 3 *cannot* check: that the values are
well-formed, are a function of the **type** rather than the instance, and do not collapse.

**The four assertions and their measured negative controls** — each control turned **exactly
one** test red and left the others green, so the assertions are independent, and none is
inert:

| Assertion | Control applied | Result |
|---|---|---|
| every name is a bare ASCII class name | `Senescence` returns `"domains::biosphere::flows::Senescence"` | `every_canonical_type_name_is_a_bare_class_name` RED |
| no two flows in one build share a name | `Senescence` returns `"Transpiration"` | `distinct_flows_report_distinct_type_names` RED |
| `type_name` ≠ `id` (class vs instance) | `Senescence::id()` returns `"Senescence"` | `type_name_is_not_the_instance_id` RED |
| a wrapper reports **itself** | `ScaledFlow::type_name` delegates to `inner` | `scaled_flow_type_name_does_not_delegate…` RED |
| *(aux axis)* no two aux processes share a name | `ThermalTimeAccumulation` returns `"RootDepthExtension"` | `distinct_flows_report_distinct_type_names` RED |
| *(aux axis)* every aux name is a bare class name | `ThermalTimeAccumulation` returns a module path | `every_canonical_type_name_is_a_bare_class_name` RED |

**⚠ The first four controls were all on the FLOW side — the aux trait was read but never
proven to bite, and the advisor caught it.** The three registry tests iterate
`aux_processes()` through the same chain, so aux values *were* being compared; what was
missing was any demonstration that an aux-side defect turns something red — which is
exactly the standard this slice set for itself and had applied rigorously one trait over.
It matters because `aux_set` is half of what slice 6 re-anchors. Two more controls were run
(rows 5–6 above): both bite, each turning exactly one test red. ⚠ **The gap was in the
evidence, not in the code — and that is the kind you only find by asking which rows of your
own control table are missing, not by re-reading the code.**

**⚠ `ScaledFlow` is where the two axes genuinely disagree, and it is the sharpest statement
of §2f's irony.** It delegates `id()` to the wrapped flow **on purpose** (so the registry
sorts it into the wrapped flow's slot and order-independence is preserved) — but Python's
`type(flow).__name__` sees the **wrapper**. So `type_name()` must *not* delegate while
`id()` must. A later refactor "helpfully" making them consistent would desynchronise the
port's reported inventory from the reference for any wrapped scenario, and that test is the
only thing that would say so.

**⚠ Checked and CLEARED — the divergence I expected and did not find.** Python has both
`simcore.expr.DeclarativeFlow` *and* an authoring registry whose entries name real domain
classes, so Rust wrapping everything in `DeclarativeFlow` would have been a genuine slice-3
finding. It does not: both ports lower a `kinetics` spec to `DeclarativeFlow`
(`interpreter.py:445` / `interpreter.rs:746`) and instantiate the real frozen class for a
typed spec (`type_spec.cls(...)`). Recorded because a *cleared* suspicion is worth as much
to slice 3 as a found one.

**⚠ Formatting: the Rust tree is NOT `rustfmt`-clean today, and slice 2 did not make it so.**
`rustfmt --check` reports pre-existing drift in six files (`crew.rs`, `eclss.rs`, `expr.rs`,
`registry.rs`, `engine_vectors.rs`, `inspection.rs`) at lines this slice never touched, and
**CI has no `cargo fmt` gate**. Every one of the 58 inserted blocks is already canonical (no
insertion site drew a diff), so only the **new file** was formatted. Reformatting the other
six would have produced a large diff, unrelated to this slice, inside frozen `simcore` —
which is the same hazard the *never bare `cargo fmt`* rule exists for, reached one file at a
time.

**Not done here, and it is slice 3's, not an oversight:** nothing yet compares Rust's
inventory to the manifest, and no Rust program *dumps* that inventory. `type_name()` is the
mechanism only; the gate is next.

**Slice 3 — Rust dumps its inventory, checked against the *existing* Python manifest.** The
pilot with zero blast radius: a Rust program builds the canonical registries and dumps the
flow set / aux set / param file list as JSON; a **new, additional** Python test asserts it
equals the manifest that is still Python-anchored. ⚠ **If Rust's inventory and Python's
manifest disagree, that is a finding to hunt before any re-anchoring** — the completeness
contract is the thing the flip is riskiest for, and this proves Rust can express it before
anything depends on it.

### Slice 3 — COMPLETE 2026-08-16

**Built.** Two Rust dump programs —
`crates/domains/examples/dump_biosphere_inventory.rs` and
`crates/station/examples/dump_station_inventory.rs` — each building the canonical
registries and printing the union of `type_name()` over them as JSON, plus one new Python
gate, `tests/crossport/test_inventory_parity.py` (3 tests). Additive throughout: `git diff
src/` empty, `git status` shows **only the three new files**, no golden regenerated, no
manifest touched, no band moved. `cargo clippy --all-targets -D warnings` clean, the full
`cargo test` suite green, ruff / ruff-format / pyright clean, and the 141 Python cross-port
+ three-manifest gates green.

**The headline: no divergence. Both inventories match their manifest name for name** —
biosphere **23 flows / 3 aux**, station **16 flows / 0 aux**. So the slice's stop-rule
(below) did not fire and slice 4 is unblocked on this axis.

**⚠ The slice was planned with three axes and shipped with two — `param_files` has NO
Rust referent, and that is a finding, not a simplification.** The plan says *"the flow set
/ aux set / **param file list**"*. This port reads no YAML: it reads
`biosphere_params.txt` / `sibling_params.txt` / `station_params.txt`, all three
**generated by Python** (`tests/crossport/gen_*_params.py`) out of the frozen loaders, as
flat `name → hexfloat` lines. Their prefixes are the generator's naming and not filenames
— `photo.` against `photosynthesis.yaml`, three prefixes out of the single
`phenology.yaml`, **17 loaders against 15 frozen files** — and the sibling/station files
carry no prefix at all. Anything Rust printed under that key would be the Python list
travelling through Rust and back, so the "parity" gate would compare **Python against
Python**: the self-referential shape this repo already had to dissolve once, for the RNG
vectors. The gate therefore asserts the dump's **exact key set**, so adding the axis back
is a red test rather than a silent tautology — the same forcing-function move as slice 2's
required trait method.

**⚠⚠ The knock-on, and it inverts this plan's own dependency table: slice 6 cannot
re-anchor `param_files` to Rust until slice 9 has decided who loads the YAML.** The table
says `9` depends on `6`. On this axis the arrow runs the other way. Slice 6 must therefore
either **declare `param_files` explicitly Python-retained, with the reason stated in the
manifest itself**, or **wait for slice 9** — and it must not silently regenerate a
Rust-anchored manifest carrying a Python-derived param list, because that is a frozen
contract with a field nothing on the reference side produces. Not resolved here; slice 6's
call. (⚠ It is also the same file set as §7's harness question, for the same reason.)

**⚠ Why the station manifest was included, and it is the whole discovery value of the
slice.** Slice 2 had already measured the biosphere names as exactly the manifest's 23/3,
so a biosphere-only slice 3 was **known in advance to pass** and would have found nothing.
The station set was unmeasured — and my pre-flight estimate that it *would* match (12
sibling `type_name` impls + 4 station ones = 16) was **a count taken by eye off a grep**,
which is precisely the evidence slice 2 recorded as unreliable one day earlier, after the
compiler found 4 impls the same grep had missed. The estimate happened to be right; that
is not a reason it was evidence.

**⚠ The real hazard was not the port — it was the dump's registry SELECTION.**
`_station_registries()` encodes five judgement calls, and the Rust dump mirrors every one
by hand with nothing checking the mirror. Get one wrong and the gate goes red for a
mis-specified dump, and the slice is spent hunting a phantom port bug. Recorded in the
example's own doc comment, and **each one measured** (below):

1. the four **standalone** sibling registries, so the stand-ins the sealed assembly drops
   (`HeatInput`, `CrewMetabolism`, `OxygenConsumption`, `FoodMetabolism`) appear at all;
2. `Some(self_discharge)` to `build_power` — `None` and `SelfDischarge` leaves the set;
3. `with_harvest = true` — `false` (the Tier-2 golden scope) and `Harvest` leaves the set;
4. the **fast** registry, `.2` of `(state, bio_reg, fast_reg)` — the tuple shape was read
   off `build_sealed_station`, not assumed;
5. the sealed build's biosphere **slow** registry deliberately excluded — include it and
   all 23 biosphere flows leak into the station set.

**The nine negative controls, each turning exactly one row red and leaving the other
green** — so the assertions are independent and none is inert, and the tree measured green
again after every revert:

| # | Axis / assertion | Control applied | Result |
|---|---|---|---|
| 1 | biosphere `flow_set` | a flow's `type_name` renamed | biosphere row RED |
| 2 | biosphere `aux_set` | an aux process's `type_name` renamed | biosphere row RED |
| 3 | station `flow_set` | a sibling flow's `type_name` renamed | station row RED |
| 4 | **station `aux_set`** | an aux process wired into a canonical station build | station row RED **on the `aux_set` assertion** |
| 5 | the key-set forcing function | a `param_files` key added to a dump | station row RED **on the key-set assertion** |
| 6 | selection (2) | `Some(self_discharge)` → `None` | station row RED (`SelfDischarge` missing) |
| 7 | selection (3) | `with_harvest` `true` → `false` | station row RED (`Harvest` missing) |
| 8 | selection (4) | fast registry `.2` → slow `.1` | station row RED |
| 9 | selection (1) | the four standalone siblings dropped | station row RED |

**⚠ Control 4 exists because the advisor named the gap before it was built, and it is the
same gap slice 2 was caught on one day earlier.** The station `aux_set` is legitimately
`[]`, and **`[] == []` is satisfied by a dump that never reached the aux accessor at
all** — a green row that has checked nothing, which is this repo's recurring defect shape
(the sensitivity probe that read `0.0` for weeks). Rows 4 and 5 were checked not just for
"something went red" but for **which assertion fired**, because a control that trips an
earlier assertion proves the earlier one, not the targeted one. Two things now stand for
that axis: the biosphere case exercises the identical `registry.aux_processes()` walk
non-trivially against three frozen names, and control 4 is recorded as measured. A third
test states *why* the set is empty and goes red if a station-side aux process is ever
added, at which point the parity row becomes load-bearing on its own.

**Not inert by skip, checked rather than inherited.** The gate is `skipif cargo is None`
locally, and the CI `crossport` job runs `uv run pytest tests/crossport/` — on the
**directory**, so a new file is collected automatically. Slice 1's record asserts this in
prose; it was re-read out of `.github/workflows/ci.yml` rather than trusted, because this
repo has two recorded green-by-skip incidents.

**Authoring stays out, on slice 2's evidence rather than by category.** Its manifest has no
flow/aux registry axis (grammar, VM node/op set, flow-type registry), and the divergence
that *would* have made it urgent — a port lowering every authored flow to the generic
declarative class and so reporting a wrong inventory — was checked and cleared in slice 2.
Slice 8 owns it.

**Found in passing, not fixed here:** `cargo test` emits a pre-existing **output filename
collision** warning — `emit_crew` is an example in *both* `simcore` and `domains`, and
cargo says this "may become a hard error in the future". Nothing to do with this slice (its
two examples have unique names, deliberately), but the repo runs examples by name and a
future cargo could turn that warning into a build failure.

**Slice 4 — regenerate the goldens from Rust.** ⚠⚠ **Must not be taken before slice 3
passes** — **it now does (2026-08-16), on both registry-derived manifests, with no
divergence**, so this dependency is discharged. This is the moment the reference actually
moves, and slice 3 is what establishes that the Rust tree's *completeness* matches the
manifest at all. Taking 4 first regenerates
the goldens from a tree nothing has checked — the table above enforces this as a dependency
because prose ordering does not. First identify which of the 25 golden files has no `emit_*`
program (24 exist). ⚠ **Predict the diff before regenerating** — the
prediction is `< 1e-11` relative on every value and *no* structural field moving at all
(Tier 0 is exact at every tier). A structural diff, or any value beyond band, is a port
bug to hunt and **stops this slice**.

### Slice 4 — the census + the Rust-side regeneration path — COMPLETE 2026-08-16 (its two stragglers landed in slice 5)

**Built.** Two new files, nothing else touched (`git diff` empty outside them):
`tests/crossport/regen_goldens_from_rust.py` — the committed, reviewable **Rust-side**
regeneration entry point, carrying the golden census as data — and
`tests/crossport/test_golden_provenance.py` (6 tests, 23 cases) gating that census and
pinning Rust's bytes. Ruff / ruff-format / pyright clean; the whole `tests/crossport/`
directory and all three manifest gates green.

**⚠ The prediction held, and then some: 16 of the 18 goldens Rust can emit are
*byte-identical* to the committed file on this UCRT box.** Not "inside the band" —
identical. The plan predicted `< 1e-11` relative with no structural field moving; measured,
the deviation is **zero** on sixteen of eighteen, including the ~1.3 M-substep sealed
station. **Two are not**, and both are biosphere:

| golden | leaves differing | worst deviation |
|---|---|---|
| `consumer_chamber_state.json` | 7 of 205 | 4.6e-16 (~2 ULP) |
| `perennial_long_horizon_state.json` | 1 of 196 | 1.6e-16 (~1 ULP) |

Both are ~5 orders **inside** their own Tier-2 band (`1e-11`), structure exact at Tier 0,
so the stop-rule did not fire. Diagnosed as **accumulated last-bit noise, not an op-level
port difference**: slice 1's trajectory export walks 2440 steps of the perennial scenario
with *zero* bitwise divergence, so there is no systematic disagreement to hunt.

**⚠ The profile confound, caught by the advisor before it became a false finding.** The
first measurement was taken under `--release`; `test_crossport.py` runs the biosphere
family in **debug**, and both divergent cases are biosphere. Re-measured: **all 18 agree
across profiles**, so the flag is a speed choice only. Had it not, *"regenerate from
Rust"* would have been under-specified until the build profile joined the reference
definition — a manifest gate turning red for a toolchain reason, which is precisely the
failure mode slice 2 rejected `std::any::type_name` over.

**⚠⚠ The plan's own arithmetic in §2f is wrong, and the gap is 7, not 1.** §2f says *"24
`emit_*` programs against 25 golden files. One is missing or one program emits two."*
Measured, all three clauses are off: two programs each serve two goldens
(`emit_perennial` / `emit_consumer`, by a `long` argument); **four** `emit_*` programs
serve no golden in that directory at all (`emit_authored`, `emit_perturbed_brownout`,
`emit_sealed_resume`, `emit_composite`); and **seven** goldens have no program that emits
their bytes. The census that replaces it, now gated:

| Group | Count | What it means |
|---|---|---|
| Rust emits the artifact | **18** | 16 byte-identical, 2 as above |
| Rust emits a raw series, **Python folds it** | **2** | `drift_summary`, `sealed_energy_drift_summary` — `drift.py` is Python-side by a deliberate Phase-7 decision (advisor #3). *The fold is the artifact.* Same shape as slice 3's `param_files` |
| **no Rust referent at all** | **5** | `n_limited` / `water_biting` (no such scenario in the Rust roster — the port says so in its own words at `biosphere/system.rs`), `demo_euler` / `demo_rk4` (no `build_demo` in Rust), `state_snapshot` (⚠ not a run at all — a `sim_io` fixture that Rust **reads**, so it is an *input* to the port) |

⚠ None of the five is in either manifest; all 20 frozen goldens are in the first two
groups. **So slice 6's per-key classification gains a clean answer on the golden axis: 18
yes, 2 folded** — and the two folded ones are `param_files`' shape again, a key whose Rust
program exists but does not produce the artifact.

**⚠⚠ The blast radius this table understated, measured rather than reasoned.** Slice 4 is
listed as *"25 goldens, reversible (git)"*. Swapping the two divergent files in and running
every gate that touches them:

* **Both freeze-manifest gates stay GREEN.** `golden_sha256` is assembled only inside
  `_regenerate()` and is **never compared** — so regenerating a frozen golden silently
  desynchronises the manifest from the file it pins. This is the *provenance-only edit that
  nothing catches* CLAUDE.md warns about, and it turns out to cover the **goldens**, not
  just the params.
* **Four Python gates go red** — `test_regression_consumer_season` and
  `test_regression_long_horizon`, each a byte-exact compare *and* an exact-`State`
  `loads_back`. ⚠ All four are `@windows_golden_only`, so the change would be **green on
  CI and red only on the developer's box** — the [[pdf-pins-green-by-skip-on-ci]] arrow,
  reversed.

**⚠⚠ And slices 4 and 5 are not independently landable in the stated order.** What gives
the cross-port comparison its meaning is not *who wrote* the golden — provenance does not
survive in the bytes — it is that **both ports are byte-pinned to the same file**. That
holds today for all 18. It cannot hold for a golden the ports disagree on: one side has to
become tolerance-gated, and moving that pinning to the Python side **is slice 5**. Take 5
first, or take 4 and 5 together; taking 4 alone leaves the two stragglers with a red
Windows gate and a Rust-vs-Rust comparison standing in for a cross-port one.

**⚠ `tiers.json`'s evidence prose is now measurably stale.** Both divergent scenarios still
read *"Rust-vs-Python bit-exact locally (same UCRT libm, max_rel_dev 0.0)"*. True when P7.4
measured it; false now. Not corrected here — `tiers.json` is slice 5's file, and this is
the ungated prose half of a contract again (`docs/log/freeze-prose-half-is-ungated.md`).

**What the new gate buys, and what it cannot.** The byte census is **~5 orders tighter than
the Tier-2 comparison beside it**: those two goldens drifted from 0 ULP to 2 ULP with
nothing in the tree noticing, which is exactly how the stale evidence string survived. The
`PORTS_DISAGREE` roster is checked **in both directions** — a golden joining it is red, and
a golden *leaving* it is red too, so it cannot decay into an unre-measured exemption. ⚠
What no byte-level check can do, stated in the module rather than implied: **while the two
ports emit identical bytes, nothing in the artifact says which side produced it.** Slice 4
makes the *path* structural, not a property of the files.

**⚠ The tautology the map had to be built to avoid.** `emit_crew` exists in **two** crates,
and `simcore`'s **parses `crew_state.json`'s own hex-floats and re-emits them** — a codec
fixture since Phase-7 Step 0. It is also the pre-existing cargo output-filename collision
slice 3 found, so a regeneration script that shelled `target/*/examples/emit_crew.exe`
would get whichever crate built last and could write the golden **from itself**. Every
invocation is `-p <crate> --example`, and one test pins the crate.

**Nine negative controls, each turning exactly one test red on the intended assertion**,
green again after every revert (checked for *which* assertion fired, per slice 3):
an unclassified golden appearing on disk; a golden classified into two groups; a *frozen*
golden parked in the no-referent group; a frozen folded golden whose reason is gutted; a
typo'd example name; `crew` re-pointed at the echoing `simcore` emitter; a known-divergent
golden dropped from the roster; an *agreeing* golden added to it; and the last-bit ceiling
lowered below the measured divergence.

**⚠ Not done, and it is the user's ordering call, not mine:** the two divergent goldens are
**not** regenerated. Doing so bundles slice 5 (the four byte-exact gates must become
tolerance comparisons) and touches slice 6's ceremony (the desynchronised `golden_sha256`).

**Slice 5 — invert the cross-port contract.** Python now the tolerance-gated side, Rust the
exact one. ⚠ The Tier-2 bands were measured as **±1-ULP sensitivity propagated through the
Rust-side transcendentals**; inverting the roles does not automatically make them valid in
the other direction. Re-measure rather than re-use, and update
`docs/native-port-reference.md`'s prose half — which **no gate checks**
(`docs/log/freeze-prose-half-is-ungated.md`).

### Slice 5 — COMPLETE 2026-08-16

**Built.** The two divergent goldens regenerated from Rust (8 changed hex-float leaves
across two files — exactly the predicted diff, no structural field moved); the biosphere
freeze manifest re-anchored to them; the divergence roster moved from the Rust census to
the Python gates and renamed; two new choke points in `tests/golden_platform.py`; three
new gates in `test_golden_provenance.py`; every regression module's compare *and* write
routed through those choke points. Ruff / pyright clean, 113 targeted gates green, all
three manifest gates green. `git diff src/` empty; no Rust touched.

**⚠ The advisor's blocking item was real: regenerating a frozen golden desyncs the
manifest with every gate green.** Both divergent goldens are in the biosphere manifest,
and `golden_sha256` is assembled inside `_regenerate()` and **never compared**. Measured
before and after: all 20 hashes matched disk beforehand; the write turned four Python
gates red and **both freeze-manifest gates stayed green while the manifest pinned bytes
that no longer existed**. The ceremony ran here rather than deferring to slice 6 — slice 6
re-anchors which *keys* derive from Rust; keeping the hash honest about the bytes on disk
is this slice's debt. Exactly two hashes moved, nothing else; the station manifest was
untouched, which is what bounded the ceremony to one contract.

**⚠ The roster was re-homed, not emptied — and its name was wrong after the flip.** The
first instinct was `PORTS_DISAGREE = {}`, which throws away what slice 4 built. The set is
still true; what changed is *which side consults it*. Before, the golden was Python's and
the open question was whether Rust matched. Now the golden **is** Rust's, so
`Rust == golden` has one allowed answer (the census is unconditional, no exemptions) and
the whole question is about the checker. ⚠ *A symmetric name survives only while the
contract is symmetric*: `PORTS_DISAGREE` became `golden_platform.PYTHON_DIVERGES` — same
two entries, same measured sizes, opposite consumer, both-directions non-decay preserved.

**⚠⚠ Convert two, keep sixteen — because a band cannot see a reduction-order change.**
Canonical flow-id order on every reduction is a non-negotiable invariant, and reordering
moves values by a ULP or two, i.e. *inside* any tolerance this repo would write. The byte
compare is the only Python-side gate that sees that class at all, so it is surrendered
only where it is provably unavailable. ⚠ And for these two it is surrendered only at
*this* horizon: `emit_consumer` and `emit_perennial` each serve two goldens, and in both
cases the sibling (5-yr perennial, 15-yr consumer) is still byte-gated. That is an
**observation, so it is asserted** — `test_every_diverging_scenario_keeps_a_byte_gated_sibling`
makes a third roster entry that would take the last byte gate off a scenario go red.

**⚠⚠ A negative control caught a hole in the first design, and the fix generalised it.**
Draft 1 converted only the two rostered modules and left the other fourteen with a raw
`==`. Control 2 — *put an agreeing golden on the roster and expect the heal direction to
fire* — came back **green**: the heal check is only live for a golden whose module
consults the roster, so a third entry landing on an unconverted module would sit inert
forever. Every regression module now routes its compare through `assert_matches_golden`
and its write through `write_python_golden`, the seven Python-authored goldens included.
⚠ *A policy with two implementations has one that is stale* — and the control, not the
review, is what showed which.

**⚠⚠ `loads_back` was reformulated, and the cost was measured rather than argued.** The
advisor's warning that its two jobs must not be flattened was right, and the outcome is
not the tidy one. The codec half (parse through the core constructors, re-emit
byte-stably) is engine-independent, so it stays **exact** and the flip does not reach it;
the equality half is `bytes_match`'s assertion and is not duplicated. Measured consequence
on the two rostered goldens:

| tamper | `matches_the_reference` | `loads_back` |
|---|---|---|
| gross value change | **red** | green |
| last nibble (~1.4e-15 rel) | **green** | green |

A sub-`1e-14` tamper on those two files is therefore invisible to Python — the honest
price of the roster, since it is by construction indistinguishable from the divergence the
roster permits. ⚠ It is **not** a hole: the byte-exact backstop moved to the side that owns
the bytes (`test_rust_reproduces_the_committed_golden_bytes`, unconditional). ⚠ That census
is Windows + `cargo` + `slow`, i.e. local, not the CI job.

**⚠⚠ The plan's own reason for re-measuring the bands was false, and the re-measurement
found something else instead.** The paragraph above says the bands were measured "through
the **Rust-side** transcendentals"; `measure_tier2_bands.py` is pure Python and shims
CPython's own `math`, so the basis was always Python-side and the inversion does not reach
it. What it measures — how far a one-ULP libm disagreement moves a *trajectory* — is a
property of the scenario's dynamics, not the language, and the two engines are
demonstrably running the same arithmetic (16/18 byte-identical, 18/18 inside 2 ULP, 2440
steps with zero bitwise divergence). Re-measured anyway: **every figure in
`docs/native-port-reference.md` reproduced exactly.**

⚠ **But `tiers.json` — the file that calls itself authoritative — was two corrections
behind.** Its `evidence` strings still carried `6.7e-14` (biosphere) and `2.7e-15`
(greenhouse) against the re-measured `3.5e-15` and `2.8e-16`. The doc had been corrected on
2026-08-14 and again on 2026-08-15; neither correction reached the JSON — *while both files
state that the doc's prose must not contradict the JSON*. Two prose halves of one contract,
disagreeing, with the authoritative half the stale one. ⚠ **My first write-up called this an
ungated band and added three new gates; that was wrong and the tests were deleted before
landing.** `test_biosphere_tier2_band_sits_above_measured_sensitivity` and two siblings
already existed and already reject a zero sensitivity — I had read one test, found it
covered three keys, and generalised. The gates were live throughout and no band moved; what
rotted was only the *record* of why each band sits where it does.

**Twelve negative controls, each turning exactly one gate red on the intended assertion.**
Roster entry dropped; an agreeing golden added (the heal direction); a golden Rust does not
author rostered; a scenario's last byte-gated sibling rostered; a name dropped from
`RUST_AUTHORED`; the noise ceiling lowered below the measured divergence; a Python
regeneration main run against a Rust-authored golden (refuses) and against a
Python-authored one (still writes); a Rust golden perturbed (census red, no roster escape);
plus the two tamper probes above, and one for each branch of the authorship-dependent failure message (a tampered `drift_summary.json` must be told Python is its reference; a tampered `thermal_state.json` must be told Rust is). ⚠ That last pair exists because the message is the **only** place a reader is told which side to look at, and the assertion fires identically whether the advice is right or backwards — the first draft told all seven Python-authored goldens to go look at Rust. *No test catches a correct assertion giving wrong advice; the only defence is to read the message under a real failure.* ⚠ Two early control runs reported false greens from a
broken harness (`sys.executable` outside `uv run` → pytest exit 4, read as "passed"); both
were re-run directly. *Check a control's own exit code before believing what it says about
the subject.*

**Also landed:** `pyproject.toml` gains `extraPaths = ["tests/crossport"]` so pyright can
follow the runtime `sys.path` insert `golden_platform` now needs, rather than silencing the
import per call site.

⚠ **Pre-existing and left alone:** `tests/test_co2_compensation_band.py` carries 7 `E501`
errors at `HEAD`, so `uv run ruff check .` was already red before this slice. Verified
against a clean checkout; deliberately not folded into this diff.

⚠ **Not done, and it is slice 11's:** the `windows_golden_only` marker stays on the two
converted gates. Now that they are tolerance comparisons the marker's original rationale
(byte-exactness is platform-bound) no longer applies to them — but the band for
glibc-CPython against a UCRT-Rust golden has never been measured, and inventing one is the
"derived, not measured" move this contract exists to refuse.

**Slices 6–8 — re-anchor the three manifests, one per slice, biosphere first.** Biosphere
first because it is the one carrying both anti-derived literals (§2b) and the crop-model
comparison, so it surfaces every hard problem at a third of the blast radius while the
other two contracts remain a working control. Each is a full unfreeze ceremony per its own
reference doc: advisor review → regenerate the manifest as the git-visible record →
document. ⚠ The two hard-coded `BIO_DT`-adjacent literals are **re-authored, not ported** —
each one is a decision about what it guards.

⚠⚠ **Slice 3's finding lands here, and the real question is bigger than the field that
surfaced it (advisor, on the closing review — my first write-up of this paragraph named
only `param_files` and was under-generalized).** The question slice 6 actually faces is
**which of the manifest's keys the Rust tree can produce at all**, and on the biosphere
manifest the answer is: *a minority of it, by content.* Classified:

| Key | Rust referent? | Why |
|---|---|---|
| `flow_set`, `aux_set` | **yes** | proved by slice 3 |
| `forcing.light_path` | **yes** | `light_path.rs` can recompute the sampled fingerprint |
| `long_horizon_years`, `scenarios.*` | yes, mechanically | constants + file hashes |
| `integrator`, `dt_days` | n/a | the two deliberate hand-written literals (§2b) — re-authored either way |
| `param_files` (~17 lines) | **no** | Rust reads no YAML; only a Python-generated hexfloat file whose names are not filenames |
| `forcing.weather_fixture` / `weather_sha256` | **no** | a hash of the Python-side oracle fixture; Rust reads `weather_facts.txt`, generated from it — **identical shape to `param_files`** ⚠ *the second clause died in **C9** (§5g): there is no such file, Rust `include_str!`s the fixture itself. The verdict stands but the reason changed — `include_str!` takes a literal, so the reference knows the bytes and not the name* |
| `science_bands`, `liveness_floors` (~104 lines of 208) | **no** | `gates_for()` is a static **AST census of `science_gate` markers on pytest functions** (`tests/science_gates.py`) — there is no Rust referent and there cannot be one while the science gates are pytest-side |

⚠⚠ **And slice 5 added one more, on the axis this table calls settled.** The biosphere
manifest now freezes **two artifacts of one run with two different authors**:
`perennial_long_horizon_state.json` is Rust's final state of the 15-yr perennial run, while
`drift_summary.json` is `drift.py`'s Python-side fold of that *same* trajectory — and the
two engines differ by 1 ULP on it. Nothing is red and nothing should be (the fold is
compared against Python's own output, which is its correct reference). But the golden axis
is therefore not "18 Rust, 2 folded" scenario-by-scenario: **one scenario appears on both
sides of the authorship line.** Decide that explicitly, not with a manifest
half-regenerated.

⚠ **So slice 9 resolves only the param half.** "Who loads the params" says nothing about
the science-gate census, which is the single largest block of the file. **Slice 6 must
make an explicit, recorded choice per key** — either (a) declare the key **Python-retained**
with the reason written beside it in the manifest, so a future reader cannot mistake it for
a Rust-derived field, or (b) wait for whatever would give it a referent. What slice 6 must
**not** do is regenerate a "Rust-anchored" manifest that silently carries Python-derived
fields — a frozen contract with fields nothing on the reference side produces is exactly the
§2e trap, in several locations at once. ⚠ Doing this classification *before* the ceremony is
cheap; discovering it mid-ceremony with a manifest half-regenerated is not. Slices 7 and 8
inherit the same exercise for their own manifests.

### Slice 6 — COMPLETE 2026-08-16

**Built.** `dump_biosphere_inventory` widened from a *witness* into the **producer** of the
biosphere manifest's Rust half; `_build_manifest()` now shells it and splices `flow_set`,
`aux_set`, `forcing.light_path`, `long_horizon_years` and every `scenarios.*.years`; the
manifest gained an `_authority` block naming the producer of **every** key, with the reason;
four new Python gates and two new cargo-side gates. `git diff src/` empty. Ruff / ruff-format
/ pyright clean, `cargo clippy --all-targets -D warnings` clean, the whole `cargo test` suite
green.

**⚠⚠ The prediction was written down before regenerating and held exactly: the only changes
to the manifest are the new `_authority` block and the `_comment`.** Not one frozen value
moved — no hash, no set, no horizon. That was the expected outcome and it is also the reason
the slice needed a *discriminating* control rather than a green suite (below).

**⚠⚠ The acceptance criterion as written was passable by a relabel, for the third slice
running (advisor).** "Re-anchor `flow_set`/`aux_set`" produces a byte-identical file, because
slice 2 already measured Rust's names as exactly the manifest's 23/3 — so nothing in the diff,
and nothing in the suite, distinguishes *the manifest now reads Rust* from *the comment now
says it does*. The criterion became a **measured pair**, and it is the slice's headline
evidence:

| Control | Manifest | Python conformance gate |
|---|---|---|
| rename a flow's `type_name()` in **Rust**, regenerate | **MOVED** (one leaf: `SenescenceRENAMED`) | RED |
| rename the **Python** class, regenerate | **byte-identical** | RED |

Either control alone proves nothing: the first is satisfied by any file that happens to
change, the second by a manifest nobody regenerated. Together they say the producer is Rust
and the checker is Python. ⚠ *An interface slice's criterion must name a call site (slice 2);
a re-anchoring slice's must name a **direction**.*

**⚠ `forcing.light_path` was the key most likely to be wrong, and it was measured before
being touched — but state precisely WHICH pair was measured (advisor).** Unlike every hash
beside it this one is **gated exactly**: CI recomputes the fingerprint in glibc-CPython and
compares strings. What was measured is **UCRT-Rust against UCRT-CPython on this box: all
twelve samples byte for byte identical.** That is *not* the CI pair, and the claim must not be
written as if it were.

**It is, however, what closes the question — by making the CI pair not arise.** Because the two
writers agree byte for byte, **the manifest's stored value did not change at all**
(`a84d3aa2…` before and after). So the CI gate compares the *identical string* it compared
before slice 6, and that comparison has been green throughout — which is itself the standing
evidence that these twelve samples are cross-libm stable in CPython. Re-anchoring introduced
**no new cross-libm exposure**, rather than introducing one that was then measured away.

⚠ Had the two writers disagreed even in the last nibble — and **two of the twelve samples sit
one ULP apart from their neighbours**, which is exactly where such a disagreement lands — the
hash *would* have moved, the CI pair *would* have been new and unmeasured, and the key would
have been declared Python-retained. Inventing a normalization is the move this contract
refuses.

**⚠ CONFIRMED ON CI 2026-08-16, and only after a second defect was cleared.** The
glibc-CPython recomputation (`test_manifest_pins_the_within_day_light_path`) **passed on
`626bd7d`** — the direct evidence, now on the record rather than argued. Getting it required
noticing that the Python CI job had been failing at the **lint** step for 12+ commits, and
that ruff runs *before* pytest: **no Python test had executed on CI that whole time.** Two
earlier records noted those lint errors as pre-existing and deliberately unfolded; neither
noted the consequence. ⚠ *"It is green on CI" is worth nothing until you check that the job
reached the step that runs it* — the green-by-skip family, one level up from the two this
repo already had.

**⚠ The classification is keyed by PATH, not by top-level key, because two keys split
(advisor).** `forcing` has three children with two answers, and `scenarios` splits *inside one
scenario* — slice 5's handoff, `perennial_long_horizon_state.json` being Rust's while
`drift_summary.json` is Python's fold of that same run. A top-level block would have hidden
exactly the distinction slice 5 handed this slice. The golden rows are **checked against**
`golden_platform.RUST_AUTHORED` rather than restating it: that roster already has two copies
held equal by a gate, and a third would be the *"a rule with two copies has one that is
stale"* hazard.

**⚠ The honest headline is MIXED AUTHORITY, not "the manifest is Rust-anchored."** By content
most of the file is still Python's: `science_bands` + `liveness_floors` are ~104 of 208 lines
and are a static AST census of pytest markers with no Rust referent while the science gates
are pytest-side; `param_files` is Python-retained **until slice 9**; so are the weather fixture
and its hash (the same shape), and `drift_summary`'s golden hash. The classification block
exists so that qualifier cannot be dropped by the next reader.

**⚠ Two literals stayed anti-derived, and only one of them got a new check.** `dt_days` is now
compared against the reference tree's `BIO_DT` — the frozen literal keeps forcing the ceremony,
and the reference can no longer move under it silently. `integrator` deliberately did **not**
get the same treatment: there is no importable scheme name on *either* side, so the only
symmetric implementation would be typing `"EulerIntegrator"` into the Rust dump and comparing
it with the manifest's copy — two hand literals checked against each other, which reads like a
gate and is none.

**Also landed: `golden_sha256` is now compared** against the files on disk, closing the hole
slice 5 measured (regenerating a frozen golden desynchronised the manifest with every manifest
gate green). Scoped to goldens only — a golden is machine-generated and *is* the value, while
the param/weather hashes are hand-edited files whose values the goldens already enforce, so
extending it there would be the redundant re-assertion the module has always declined.

**⚠ `test_inventory_parity.py`'s two cases stopped meaning the same thing, and the failure
message had to branch.** The biosphere case is now a **staleness** check (the manifest is
generated from that dump), while the station case is still a genuine two-port **parity** check
until slice 7. The assertion fires identically either way, and its advice — *"a finding to
hunt; do NOT adjust either side to agree"* — is right for the station and actively wrong for
the biosphere, where the fix is the regeneration ceremony. Branched, per slice 5's lesson that
*no test catches a correct assertion giving wrong advice*.

**Nine negative controls, each turning exactly one gate red on the intended assertion**, green
again after every revert: the two-direction rename pair above; a golden's bytes tampered
(hash gate); an unclassified field added to the manifest; an `_authority` pattern matching no
field; `drift_summary` reclassified as Rust-authored (caught against the roster); the reference
tree's `BIO_DT` moved to 0.5; its light-path peak factor moved to π/2.1; its
`LONG_HORIZON_YEARS` moved to 16 (which reached both the gate and three regenerated leaves);
a frozen chamber horizon tampered, which only the new Python horizon-conformance gate sees;
and two `_authority` patterns of **equal** specificity matching one path — added after the
closing review, because "most specific wins" decides nothing on a tie and the field would
read as classified either way (advisor).
⚠ One control run was **invalid and was re-run**: a stray command had corrupted the manifest's
JSON, so sixteen tests failed on a parse error rather than on the assertion under test —
*a control that turns the whole file red has measured nothing* (slice 5's "check the control's
own exit code", one layer up).

**Not done, and named rather than left implicit:** `param_files` and the weather fixture wait
for slice 9; the science-gate census has no route to the reference at all and is not a slice-9
question; and slices 7 and 8 inherit this same per-key exercise for their own manifests —
including the reading that the station case in `test_inventory_parity.py` is still a parity
check *because* slice 7 has not been taken.

### Slice 7 — COMPLETE 2026-08-16

**Built.** `dump_station_inventory` widened from a *witness* into the **producer** of the
station manifest's Rust half (it gained the two sealed horizons); `_build_manifest()` now
shells it and splices `flow_set`, `aux_set`, `sealed_station_years`, `sealed_energy_years`;
the manifest gained an `_authority` block naming the producer of **every** key with the
reason; four new Python gates and one new cargo-side gate. `git diff src/` empty. Ruff /
ruff-format / pyright clean, `cargo clippy --all-targets -D warnings` clean.

**⚠⚠ The prediction was written down before regenerating and held exactly: the only
changes to the manifest are the new `_authority` block and the `_comment`.** Not one frozen
value moved — the same 16 flow names, the same empty `aux_set`, the same 15/4 horizons, not
one hash. Measured *before* touching anything by running the dump and comparing by eye to
the committed file, per `soil-layers`' rule. Same outcome as slice 6, for the same reason:
slice 3 had already proved the sets identical, so this ceremony was always going to be a
relabel unless something was wrong.

**⚠⚠ Which is why the criterion is a measured PAIR again, and it is the slice's headline
evidence.** "Re-anchor `flow_set`" produces a byte-identical file, so nothing in the diff
and nothing in the suite distinguishes *the manifest now reads Rust* from *the comment now
says it does*:

| Control | Manifest | Python conformance gate |
|---|---|---|
| rename `SolarCharge::type_name()` in **Rust**, regenerate | **MOVED** (one leaf: `SolarChargeRENAMED`) | RED |
| rename the **Python** class, regenerate | **byte-identical** | RED |

⚠ The **horizon** axis got its own pair rather than inheriting the flow axis's: moving
Rust's `SEALED_STATION_YEARS` to 6 makes the regenerated manifest read 6 while Python's
constant stays 4 and `test_manifest_horizons_match_constants` goes red. A re-anchoring
slice's criterion must name a direction *per axis*, not once per slice.

**⚠⚠ The `aux_set` axis was the blocking item, and it needed a control shape slice 6 never
had to invent (advisor, before the build).** The station's aux set is legitimately `[]`, so
every assertion about it is `[] == []` — and slice 7 **escalates** that from "compared" to
"written into the frozen manifest by a regeneration", which is exactly the hazard the
biosphere dump records for itself. Slice 6's two-direction rename control is **unrunnable
here: there is no station aux process to rename.** The substitute, run through the
regeneration path: wire a throwaway `AuxProcess` into a canonical Rust station build →
the dump reports it → the regenerated manifest **gains** `ControlProbeAux` →
`test_frozen_station_aux_set_is_complete` goes red and the other twelve stay green → revert
→ green again. ⚠ *When the value under test is empty, the control has to change the value,
not the name.*

**⚠ The dt gate was priced and REFUSED, and the refusal is the finding worth carrying.**
The station's steps live inside `numerics_note`, a prose string this module has recorded as
hand-maintained-and-ungated since 2026-08-14, and the reference tree *does* have referents
for them (`sealed_station_scenario()`'s `bio_dt`/`cabin_dt`, the energy scenario's
`power_dt`) — so slice 6's `dt_days` treatment is buildable. It was not built, on one
asymmetry (advisor): **slice 6 added zero new frozen values.** `_authority` is metadata
about the contract and the `dt_days` gate covered a key that already existed, whereas a
`locked_dt` key here would *widen the frozen surface* — a separate unfreeze with its own
ceremony, not a rider on a re-anchoring. The precedent is this plan's own, twice: slice 3
declined `param_files` and made the exclusion a forcing function; slice 6 declined
`integrator` rather than type a second literal. Recorded in the key's `_authority` entry, in
the dump example, and in `docs/station-reference.md`.

**⚠ The classification is keyed by PATH, and `scenarios` splits inside itself here too.**
Twelve of the thirteen goldens are Rust's; `sealed_energy_drift_summary.json` is `drift.py`'s
Python-side fold of a raw Rust series — the station's copy of the biosphere's
`drift_summary` case, found by classifying rather than by being caught. The golden rows are
**checked against** `golden_platform.RUST_AUTHORED` rather than restating it.

**⚠ Found while classifying, and it is a cross-contract coupling nobody had written down:
`SEALED_ENERGY_YEARS = LONG_HORIZON_YEARS` in the reference tree.** After slice 7 the
station and biosphere manifests are anchored to the **same** reference-side constant, so
moving the decade horizon is one edit and *two* ceremonies. Recorded in the `_authority`
entry, the dump example, the Python gate and the reference doc, because a reader who assumes
the two contracts are independent will predict the wrong diff.

**⚠ The failure advice branched rather than collapsed (advisor).** Both cases in
`test_inventory_parity.py` are staleness checks now, so the obvious move was one message.
But the station dump mirrors five registry-selection judgement calls **by hand**, and after
this slice a mis-mirrored one is *written into the frozen manifest* by the very regeneration
the message sends you to — so "check the selection first" is more load-bearing than it was,
not less. The shared half is one string; the selection warning stays station-only.

**Also landed: `golden_sha256` is now compared** against the files on disk, closing on this
contract the hole slice 5 measured on the other (regenerating a frozen golden desynchronised
the manifest with every manifest gate green). Scoped to goldens only, for slice 6's reason.

**⚠ Closing review found a duplicate this slice inherited and then doubled (advisor).** The
dump's exact key set is declared **twice per contract** — once in the generator, where it
stops a regeneration from splicing an unclassified key, and once in the crossport gate. Slice
6 introduced the biosphere pair; slice 7 wrote the station one. Both copies are correct and
control 10 proved each bites, but the failure is **one-sided**: widen the generator's copy and
forget the gate's, and regeneration accepts the new key while the crossport gate reddens with
a message blaming the *dump* — the wrong place to look. Measured: doing exactly that leaves
the crossport key-set assertion **green**. Closed the way slice 6 closed `RUST_AUTHORED` —
the generators own the definition and one new gate asserts agreement, rather than a third
copy. ⚠ *A duplicate that a control reddens today is still a duplicate; what a control shows
is that both copies are right now, not that they must stay equal.*

**Twelve negative controls, each turning exactly one gate red on the intended assertion**,
green again after every revert: the two-direction rename pair; the aux-wiring pair (manifest
gains the name / gate reddens); the horizon pair (crossport staleness reddens *without*
regenerating, and the regenerated manifest follows Rust while Python's gate reddens); an
unclassified field added to the manifest; an `_authority` pattern matching no field; two
patterns of **equal** specificity matching one path; a golden's bytes tampered; the folded
`sealed_energy_drift` reclassified as Rust-authored (caught against the roster); and a
`param_files` key added to the dump, which **refused regeneration with exit 1** and reddened
the crossport key-set assertion; and the two dump-key copies made to disagree, which reddens
the new tie gate while every other row stays green.

**⚠ Both failure branches of the staleness message were read under a REAL failure**, not
reviewed — slice 5's rule that no test catches a correct assertion giving wrong advice. The
station branch leads with the registry-selection paragraph and points at
`docs/station-reference.md` + `tests/test_station_freeze_manifest.py`; the biosphere branch
carries no selection paragraph and points at its own two. Both correct, both read off a
genuine red run.

⚠ **One control had to be re-run because it fired on the wrong assertion, and that is slice
3's lesson arriving on schedule.** The stale-pattern control was first run by editing
`_AUTHORITY` *without* regenerating — which reddened the gate, but on
`manifest["_authority"] == _AUTHORITY` (the committed block was simply out of date), not on
the stale-pattern assertion it was aimed at. The ghost pattern has to reach the file before
the check that hunts ghosts in the file can see it. *A control that reddens the target test
has still measured nothing until you check WHICH line failed.*

**Not done, and named rather than left implicit:** `param_files` and the `numerics_note` dt
question both wait (slice 9 and its own ceremony respectively); the science-gate census has
no route to the reference at all; and slice 8 inherits this same per-key exercise for the
authoring manifest, whose surface has no flow/aux registry axis and so will not look like
either of these two.

### Slice 8 — COMPLETE 2026-08-17

**Built.** A new `authoring::surface` module (the platform census), a new
`dump_authoring_inventory` example, and the manifest's platform half re-anchored to it:
`_build_manifest()` now shells the dump and splices **all nine** platform keys, the manifest
gained an `_authority` block naming the producer of every key, and four consts were hoisted
in the reference tree so the dump reads what the parser enforces. Three new Python gates,
one new crossport staleness gate, six new cargo-side tests. `git diff src/` empty. Ruff /
ruff-format / pyright clean, `cargo clippy --all-targets -D warnings` clean, the whole
`cargo test` suite green, 151 authoring + crossport gates green.

**⚠⚠ The prediction was written down before regenerating and held exactly: the only changes
to the manifest are the new `_authority` block and the `_comment`.** Not one frozen value
moved — same grammar, same 8 spec models, same 12 flow types, same loaders. Measured
axis-by-axis against the committed file **before** any regeneration (the advisor's blocking
item), and the first comparison run was itself a control: it came back with **one**
divergence, `ref_keywords` in declaration order against the manifest's sorted order. The
*content* was right and only the order was not — fixed in `surface::sorted` rather than at
the printer, because a printer-side sort leaves every future axis one forgotten `sort()`
away from a false divergence, and a false alarm on this gate reads as a port bug to hunt.

**⚠⚠ This contract is where the flip is NOT free, and that is the slice's real finding.**
Slices 6 and 7 were relabels because slice 3 had already proved the sets identical *through
the same mechanism on both sides* — a runtime walk of a **built registry**. §2b's retraction
("the gates don't introspect the namespace, they read `registry.flows`, and Rust does that
identically") is what made those cheap. **It does not transfer here.** This manifest freezes
the *platform*, which has no runtime object to interrogate: Python derives it by language
introspection Rust does not have (`typing.get_args` over the closed `Expr` union, a scan of
`vars(authoring.schema)`, pydantic `model_fields`, a dict), and the reference side offers an
`enum`, a `match` and a set of `const`s instead. **Re-anchoring therefore traded a derived
census for a partly hand-maintained one**, per axis:

| Axis | Reference side | What nothing on that side catches |
|---|---|---|
| `ref_keywords`, `step_token`, `rate_classes`, `schema_fields` | **load-bearing** — the tables the parser/interpreter reject against | a whole new spec model (Python's module scan catches it; a new const is forced into nothing) |
| `expr_nodes` | names from an exhaustive `match` — a new variant is a **compile error** | the emitted list is a hand roster: a variant can be named and still omitted |
| `binary_ops` | symbols from `BinaryOp::symbol()`; `/` absent from the **type** | the three-variant roster |
| `flow_types` | entries fully derived; `cls` read off a **constructed** flow | the roster is `FLOW_TYPE_NAMES`, hand-maintained |
| `integrator_names` | both dispatch arms build their error from the slice; every listed name is **run** | a `match` arm added and not listed |

⚠ **"Load-bearing" is measured, not asserted** — and the first measurement was wrong for a
harness reason. `cargo test` stops at the first failing target, so the initial run reported
that dropping `n_sub` from `SCENARIO_KEYS` reddened **only** the new surface test, which
would have been a real coverage finding. Re-run with `--no-fail-fast`: it reddens **14 tests
across 5 targets**, the whole multi-rate suite included. Dropping `forcing` from
`REF_KEYWORDS` reddens `parse_parity_accept_and_reject` **and**
`trajectory_parity_all_scenarios` — the cross-port vector gates themselves; changing
`STEP_TOKEN` reddens the parser suite; dropping `slow` from `RATE_CLASSES` reddens the
multi-rate suite. *Slice 5's "check the control's own exit code" has a sibling: check that
the control's runner reached every target it was supposed to.*

**⚠ Because of that asymmetry the Python derivations were KEPT, and their meaning inverted
in place.** `manifest["expr_nodes"] == _expr_nodes()` is the identical assertion it was
yesterday and now asks the opposite question — *has the checker drifted from the contract?*
rather than *is the manifest a faithful record of Python?*. A silent reversal of exactly this
shape is already in the log (`o2-makeup-reversal-inside-the-freeze`), so a new test
(`test_the_python_derivations_are_conformance_checks_now`) pins the direction and asserts the
set of conformance-checked axes **equals** the spliced set — an axis spliced from Rust with
no Python derivation beside it would leave the checker unchecked there, which is allowed but
must be a decision.

**⚠ The acceptance criterion was a measured PAIR for the third re-anchoring running:**

| Control | Manifest | Python conformance gate |
|---|---|---|
| rename a wiring field in **Rust** (`co2_removed`), regenerate | **MOVED** | RED |
| rename the same field in **Python**, regenerate | **byte-identical** | RED |

**Seventeen negative controls, each turning exactly one gate red on the intended
assertion**, green again after every revert: the two-direction pair; one per axis
(`expr_nodes`, `binary_ops`, `ref_keywords`, `schema_fields`, `step_token`,
`integrator_names`, `rate_classes`, `param_loaders`); an unclassified manifest field; a
stale `_authority` pattern; two patterns of **equal** specificity; `_AUTHORITY` and the
splice list made to disagree; a new key added to the dump (**regeneration refused, exit 1**);
the two dump-key copies made to disagree; and a reference-side rename left un-regenerated
(the crossport staleness gate). ⚠ Controls 12 and 13 redden the *same test*, so which line
fired was read off a real failure — the stale-pattern assertion and the tie assertion
respectively (slice 7's lesson, arriving on schedule).

**⚠ Found by a control, not by review: `step_token` was a frozen value that NOTHING
checked.** It was written into `_build_manifest` as the literal `"n"` and no gate compared
it — so changing the reference's token moves the manifest and, before this slice, would have
reddened nothing at all. The new conformance gate is what closes it, and control 7 is the
measurement. *An ungated field does not announce itself; it turns up when you build a control
for it and find no test to redden.*

**⚠ The third anti-derived literal, found where §2b's census never looked.**
`test_manifest_records_the_grammar_is_incomplete` asserts `binary_ops == {"+","-","*"}` by
hand. §2b counted 23 derived assertions against 2 hard-coded literals and surveyed only the
biosphere gate's `BIO_DT` pair, so this one was never on the list. Control 4 confirms it is
live: dropping `Mul` reddens both it *and* the completeness gate. Kept as-is — it guards the
deliberate incompleteness of the grammar (that `/` is deferred), which is a decision, not a
value the tree should be able to move on its own.

**⚠ `parity_vectors` is PYTHON-RETAINED, and it is `param_files`' finding reached by a
different road.** `parse_vectors.txt` / `traj_vectors.txt` live in the *Rust* crate's
`tests/data`, which makes hashing them from the reference side look natural — but they are
**generated by `tests/crossport/gen_authoring_vectors.py`** and merely re-derived in Rust as
the parity check, so a Rust-side hash would compare the checker's output with itself. Slice
3's key-set forcing function is what makes adding it a loud refusal rather than a silent
tautology (control 15).

**⚠ The `SpecKind` forcing function was priced and DEFERRED, on slice 7's precedent.** The
one hole with a clean fix is `schema_fields`' completeness: threading a `SpecKind` enum
through `reject_unknown_keys` and matching it exhaustively in the dump would make a new spec
model a **compile error** until it is classified. It restructures eight parser call sites to
serve a manifest key, which is its own change rather than a rider on a re-anchoring — exactly
why slice 7 declined `locked_dt`. Recorded in `schema.rs`, in `surface.rs` and in the
manifest's `_authority` entry. ⚠⚠ **The 2026-08-17 target-state change raises its priority**:
under B the Python module scan backstops this hole, and under C that backstop is scheduled
for deletion.

**Also landed:** the two prose halves this slice makes false were moved with it, both quoted
rather than silently edited — `flow_registry.rs`'s *"the authoring manifest freezes the
Python surface, so nothing here fails until an anchor exercises the missing arm"* (the
exposure has swapped ends: a **Rust-only** registration now widens the frozen contract at the
next regeneration) and `docs/authoring-reference.md`'s *"Cross-port boundary, stated
honestly"* paragraph, which gains the per-axis table above and a new unfreeze-log entry.
The ceremony itself gained a step-4 note: regeneration now needs `cargo`, and step 3's "land
it on both ports" now has a **direction**.

**Not done, and named rather than left implicit:** `parity_vectors` waits for whatever
retires its Python generator; the `SpecKind` census is its own change; and the manifest's
`grammar_note` stays hand-written prose by design — it records which ops are deferred and
why, which is a decision rather than tree state.

---

## §5b ⚠⚠ The target state changed on 2026-08-17, mid-slice — B is now C

**The user's words, received while slice 8 was in flight:** *"the whole project should
become rust based, python can be used only when using external software as a reference, or
in the process of rewriting."*

This doc executes **B** (§3): *"Is not: a retirement of Python (that is C, and 2d keeps it
off the table while the laboratory can still mint traces)."* The instruction above **is C**,
with one carve-out that §2d already identified as the hinge: Python survives (a) as the
laboratory that talks to external software — the WOFOST/PCSE crop-model oracle — and (b) as
scaffolding during the rewrite. Everything else is scheduled for retirement rather than for
permanent checker duty.

**Recorded here, not acted on, and slice 8 was finished under B's design deliberately** —
slice 8 is required under both targets (the authoring contract re-anchors either way), and
re-scoping a ceremony mid-flight is how a manifest ends up half-regenerated. What changes:

* **§4's cost is partly obsolete.** B's price was "the two implementations stop being
  independent". C's is larger and different: the checker goes away, so the *disagreement*
  itself stops being detectable — not merely un-arbitrable. Every mechanism this doc leans on
  that lives in Python (the conformance gates, the three manifest generators, the science-gate
  census, `drift.py`'s folds, the vector generators) needs an owner or an end date.
* **Slice 9 loses one of its two candidate answers.** "(b) keep pint in Python but make the
  check provably read what Rust loaded" is a permanent-checker design. Under C only (a) —
  reimplement the dimensional check in the Rust loader — survives.
* **Slice 11's posture section is now C's posture, not B's.**
* **The three keys currently marked `python` in the manifests** (`param_files`, the weather
  fixture, the science-gate census) stop being "retained" and become "not yet ported" —
  a queue, not a classification. `parity_vectors` joins them.
* **New work with no slice yet:** the science gates are pytest markers (~104 of the biosphere
  manifest's 208 lines), the goldens' folds, and the oracle harness. None of these has a Rust
  home, and the science-gate census in particular is the single largest Python-authored block
  of any manifest.

⚠ **This is a re-plan, and it is the user's to approve before anything is re-scoped.** The
eleven slices stay as written until then; what is above is the delta, not a new plan.

**Slice 9 — unit validation.** The §2e trap. Two candidate answers, to be priced when
taken, not now: (a) reimplement the dimensional check in Rust's loader, so the validated
path is the executed path; (b) keep pint in Python but make the Python check provably read
*what Rust actually loaded* rather than the file Rust happens to also read. ⚠ Whatever is
chosen, the acceptance criterion is that **removing the check turns something red** — a
lint whose deletion changes nothing is the defect this slice exists to avoid.

**Slice 10 — the 12 laws in Rust.** Add `proptest`; mirror conservation, non-negativity and
order-independence natively. Independent of every other slice; can be taken at any point.

**Slice 11 — the posture lands.** Rewrite the purity invariant (`git diff src/` empty is
A's rule and becomes wrong under B — the replacement governs *Python* as the consumer), add
the development-posture section `CLAUDE.md` has never had (§1), and close this doc out with
the normal three: index line, pointer row, record file in `docs/log/`, plus a memory file.
⚠⚠ **The posture section is a deliberate exception to `CLAUDE.md`'s own working-style rule,
and the argument must be made here rather than looked up** — that rule says a finished piece
of work earns an index line, a pointer row, a `docs/log/` file and a memory file, and
**"Nothing here."** It is about *finished-work records*. A standing posture rule is the
opposite category: it is *"what you need BEFORE you know what you are working on"*, which is
exactly what that file says it carries and is why §6 step 1 of the A plan called landing it
there the single most important step. Whoever takes this slice will read the retirement rule
first; without this paragraph they will correctly conclude the section does not belong, and
B's posture will end up as undiscoverable as A's was.
⚠ **The log exemption is already gone — deleted with slice 1, not left for this slice.**
It was written on the premise that this doc was *forward-looking with no finished work
behind it*; slice 1 ended that premise the day after, so the doc took the normal three
(index line, pointer row, `docs/log/reference-flip.md`) then rather than carrying a
now-false exemption through ten more slices. The log's own hardest lesson is that an
exemption written for a temporary state is a deletion someone must remember, and forgetting
it left three checks red for five commits. **What this slice still owes the record is the
closing update to that file, plus the memory file** — not its creation.

## §5c The C re-plan — measured 2026-08-17; its three decisions ANSWERED the same day

The user asked for a re-plan rather than an absorb-as-you-go, so this section re-prices the
remaining work against C. Every number below was measured today, not estimated. **The three
questions it opened are answered** (see below): rewrite the study tools, retire the orphan
scenarios, delete Python. ⚠ **Nothing here is executed yet** — C1 has not been started, and
the slice numbering of §5 is superseded by C1–C7 plus Stage 3.

### What C actually costs, measured

| Python surface | Size | Under B | Under C |
|---|---|---|---|
| `src/simcore` + `domains` + `station` + `authoring` | 23,158 lines | the checker | **retired** once its gates move |
| `src/config` (param YAML loader, pint units, the override seam) | 369 lines, 28 YAML files | slice 9 decides | **must move** — Rust is the loader |
| `tests/oracle/` PCSE runners + the 6 JSON fixtures | 840 lines + 306 KB | kept | **KEPT — the user's carve-out, and the ONLY Python that survives** (see the boundary section) |
| `src/lab/oracle_match.py` (the comparison arithmetic) | 103 lines | kept | **→ Rust** — ours, not third-party; it never imports PCSE |
| `src/lab/rk45.py`, `convergence.py` | 409 lines | kept | **→ Rust** (decided 2026-08-17) — our own study tools, so the carve-out does not cover them |
| `tests/` minus crossport and oracle | 145 files, 51,315 lines, ~2,300 tests | kept green | **port or retire, test by test** |
| `tests/crossport` — **the comparison half** (`test_crossport.py`, `compare.py`, `measure_tier2_bands.py`, `tiers.json`) | ~2,000 lines | the whole point | **DELETED, not ported** — with one port there is nothing to compare |
| `tests/crossport` — **the generator half** (9 × `gen_*.py`, `regen_goldens_from_rust.py`) | ~1,400 lines | inputs | ⚠ **NOT the same row** — see the correction below. Each needs a **named successor** |
| `tests/crossport/test_godot_*.py` | 9 files, 1,668 lines | consumer parity | ⚠ **unclassified** — these exercise the Godot bridge, not port-vs-port. Needs a per-file pass before anything is deleted |
| the three manifest generators | inside `tests/` | Python writes the contracts | **must move**, or the frozen contracts stay Python-authored forever |
| the science-gate census | 15 markers → ~104 of 208 manifest lines | Python-retained | **needs a Rust home or an end date** |
| `drift.py` folds | 2 goldens, 2 manifests | "one run, two authors" | **must move** to end the split |
| 4 Python-only scenarios | `demo_euler`, `demo_rk4`, `n_limited`, `water_biting` | out of scope | **port or retire** — they have no Rust referent at all |

**Rust today: 334 tests. Python today: 2,471.** That ratio is the plan's real shape — everything
else is bookkeeping beside it.

### The revised slices

**Stage 1 — finish the flip (needed under B and C alike; two of these are already written).**

| # | Slice | Note |
|---|---|---|
| C1 | **Params + units move to Rust** (was slice 9) | ⚠ C **removes one of its two candidate answers**: "keep pint in Python and make it read what Rust loaded" was a permanent-checker design. Only "reimplement the dimensional check in the Rust loader" survives. Unblocks `param_files` in **both** other manifests **and** the weather fixture. ⚠ **Take the user's harness with it** — `config/overrides.py` is the same file set (§7) |
| C2 | **The 12 laws in Rust** (was slice 10) | unchanged; `proptest`; independent of everything |
| C3 | **Posture + purity invariant** (was slice 11) | ⚠ now C's posture, not B's. `git diff src/` empty **inverts**: under C, `src/` is what shrinks |

**Stage 2 — the pieces only Python can do today.** Each is a contract key that is currently
classified `python` and would otherwise stay that way forever.

| # | Slice | Why it is not optional |
|---|---|---|
| C4 | **The science-gate census** | 15 pytest markers produce ~104 of the biosphere manifest's 208 lines. It is the **single largest Python-authored block of any contract**, and it has no Rust route at all while the gates are pytest functions |
| C5 | **`drift.py`'s folds** | ends the "one run, two authors" split slice 5 created and slice 7 inherited — two goldens in two manifests |
| C6 | **The 4 Python-only scenarios — RETIRE** (decided 2026-08-17) — **COMPLETE 2026-08-18, §5k** | now a **deletion with a written reason per scenario**, not a port. `demo_*` is a skeleton; `n_limited`/`water_biting` are science the Rust roster never carried. The record is the whole slice — a silent retirement is what it exists to prevent |
| C7 | **The manifest generators** — **COMPLETE 2026-08-18: all three halves (§5m biosphere, §5n authoring, §5p station) plus the C4b prerequisite (§5o). No Python program writes a frozen contract any more.** | they are Python scripts that *write* the three frozen contracts. Until they move, "Rust is the reference" has a Python-shaped hole in the middle of it. ⚠ Splits by contract: the biosphere writer landed byte-neutral; **authoring** is next; **station is BLOCKED on C4b**, whose two claims the reference cannot derive |

**Stage 3 — the suite.** ~2,300 tests, 51k lines. Not one slice; a classification pass first,
then batches by kind: the laws (C2 has 12), the regression/golden gates (mostly exist in Rust
already), the science gates (C4), and the rest.

> ⚠⚠ **The classification pass is DONE (§5q, 2026-08-18) and it corrected this paragraph
> twice.** The real figure is **2,452 collected across 174 files**, and *"the regression/golden
> gates mostly exist in Rust already"* is **FALSE** — measured exhaustively, **no Rust test
> compares a run against a committed golden**; the whole comparison is Python-side (§5q
> FINDING 3). §5q also found that Stage 3 does not begin with tests at all: the reference
> `include_str!`s 24 files out of the tree being deleted, so `rm -rf src/ tests/` fails the
> *build* (FINDING 1). The six-slice order §5q derives supersedes "batches by kind".

### ⚠ CORRECTION 2026-08-17 — "delete `tests/crossport`" was wrong for a third of it

The row above originally read *"25 files, 6,552 lines, DELETED — their entire subject is the
two ports agreeing."* **That is true of the comparison tests and false of the generators**,
and the mistake was structural: the directory was priced as one thing because it has one name.

`tests/crossport/` also holds the **current path from source data into Rust** —
`gen_biosphere_params.py` → `biosphere_params.txt` via `include_str!`,
`gen_biosphere_weather.py` → a Rust source file, six more vector/param generators, and
`regen_goldens_from_rust.py`, **which is the Rust→golden path slice 4 built.** Deleting the
directory wholesale would have deleted the mechanism the flip runs on.

⚠ **And the deletion would redden nothing.** The generated files are checked in and consumed
by `include_str!`, so removing their generator leaves a green suite and **unregenerable data** —
[[authoring-manifest-reanchored]]'s lesson exactly: *a control with no test to turn red IS the
finding.*

**Each generator needs a named successor before it is touched:**

| Generator | Successor |
|---|---|
| `gen_biosphere_params.py`, `gen_sibling_params.py`, `gen_station_params.py` | **C1** (the Rust param loader) supersedes all three |
| `gen_biosphere_weather.py` | ⚠ was **NONE — a real gap in C1–C7**; scheduled into Stage 1 and **built as C9** (§5g). The reference reads the fixture JSON itself now, and the generator and its table are deleted |
| `gen_engine_vectors.py`, `gen_rng_vectors.py`, `gen_vectors.py`, `gen_authoring_vectors.py` | die **with** the comparison they feed — but confirm per file; a vector that anchors something other than port-parity outlives its reason |
| `regen_goldens_from_rust.py` | **must move to Rust, not die** — it is how a golden is regenerated at all |

### The three things that need the user — ✅ ANSWERED 2026-08-17

1. **`rk45.py` / `convergence.py`** → **rewrite to Rust.** They are *our* study tools, not
   third-party software, so the carve-out does not cover them. ⚠ This settles
   `oracle_match.py` by the same argument (see the boundary below): the comparison arithmetic
   is ours; only the thing that *talks to PCSE* is external.
2. **C6, the four orphan scenarios** → **retire.** Not ported. C6 stops being a port and
   becomes a deletion with a written reason per scenario (the record is the point — a silent
   retirement is what the slice exists to prevent).
3. **End state** → **Python deleted**, except the carve-out below. Not frozen-in-place. So
   every row of the cost table above that says "port or retire" has a *terminal* state, and
   `tests/` does not survive as a read-only second opinion.

### Where exactly the carve-out line falls — measured 2026-08-17

The user's words were *"python can be used only when using external software as a reference."*
Measured against the tree, that names **~840 lines out of ~2,800**, and the boundary is not
the `oracle` name — it is **who imports PCSE**.

| Piece | Lines | Needs PCSE? | Fate |
|---|---|---|---|
| `tests/oracle/{runner,lintul3_runner,wofost_potato_runner}.py` | 840 | **yes** — `importorskip("pcse")` | **STAYS PYTHON, permanently.** No Rust route exists and porting PCSE would breach EUPL |
| the 6 committed JSON fixtures | 306 KB | no — they are its *output* | **STAY** (data). ⚠ Need a new home: `tests/` is being deleted |
| `src/lab/oracle_match.py` | 103 | no — pure stdlib arithmetic | **→ Rust** (decision 1's rule: ours, not theirs) |
| `tests/test_oracle_{gap,gap_spring_wheat,smoke,match}.py` | 947 | no — they read the JSON | **→ Rust**, with the Stage-3 suite |
| the 3 `@pytest.mark.oracle` regeneration tests | 164 | yes | **STAY** with their runners |
| ⚠ `tests/oracle/test_{reference,lintul3}_fixture.py` | 135 | **no** — and **not marked `oracle`** | **→ Rust.** These run in *every normal build*; they are the fixtures' only routine validation |

⚠ **Do not read `tests/oracle/` as "the carve-out".** Two of its seven test files are unmarked
and PCSE-free — they validate the committed JSON on every run. Miss them and the JSON becomes
**unchecked input to 47 Rust tests**: a concrete mechanism for the silent-rot risk below,
not a hypothetical one. *The folder name is not the dependency.*

⚠ **The sleeper, and it is not in the oracle at all.** `tests/oracle/winter_wheat_weather.json`
is raw NASAPower *weather*, not model output — and **47 test files across the biosphere and
station suites read it** (counted 2026-08-17; an earlier draft of this section said "20+",
which was a truncated `head -20`, not a count), plus `tests/crossport/gen_biosphere_weather.py`,
which compiles it into a Rust source file. So the fixture is load-bearing for the whole suite
port, while the *plumbing that reads it* is Python that must be rebuilt — and that generator
is the one with **no successor anywhere in C1–C7**. Schedule the reader in Stage 1: it is a
Stage-3 blocker wearing an oracle's name.

⚠ **What the carve-out becomes: a hand-run laboratory, not a build step.** Those 3 tests are
already opt-in (`-m oracle`, enforced override-proof in `conftest.py`) and need network +
a heavyweight install, so they effectively never run today. Under C they become **the last
Python in the tree**, with nothing importing them and no build touching them — the failure
mode is silent rot, not breakage. The mitigation is the provenance record each fixture already
carries: regeneration must stay a documented manual ceremony with an owner. See
[[pcse-oracle-licensing]] for the rule that keeps it clean (commit output + provenance, never
the parameter YAML).

⚠ **And it is not a gate.** The user's own ruling (scope B, 2026-07-20) is *literature-ranges
only; the oracle is a **diagnostic**, never a fit target*. `test_oracle_gap.py` pins
**known-wrong behaviour as numbers** — green means "still wrong in exactly the documented
way". Porting it to Rust must carry that inversion across, or a rewritten gap test will be
read as a pass.

### ⚠ The price, restated because §4's version is now too small

§4 recorded B's cost: *the two implementations stop being independent, so a disagreement is
resolved in Rust's favour by definition.* **C's cost is one step further: with one
implementation there is no disagreement to resolve.** The mechanism being given up caught a
year-2 vernalization reset bug, the multi-rate phase's zero-coverage driver, and — 2026-08-12,
five days before this decision — a scenario constant the entire Python suite could not see.
Recorded as information. The user has taken the decision; this is what it costs.

## §5d C1 designed and gated — measured 2026-08-17, BEFORE any Rust was written

The re-plan says C1 *"unblocks `param_files` in both other manifests"*. This section takes
that literally and **narrows C1 to the loader**: build the Rust param load, gate it against
the artefact Python already produces, and leave the manifest re-anchoring to a successor with
its own control — the shape slices 6–8 each got. Every number below was measured on frozen
`main` (`git diff` empty, nothing regenerated) before the design was fixed.

### ⚠⚠ The gating measurement: C1 is BIT-NEUTRAL, so it is a re-anchoring slice, not an unfreeze

The hazard was concrete. Rust today consumes **post-fold** values as hex-floats that *Python*
computed; if Rust re-derives them from the decimal YAML and a single bit moves, C1 stops being
a loader change and becomes an unfreeze with 18 Rust-authored goldens behind it — slice 5 made
the Rust byte census **unconditional**, so a 1-ULP shift is not "inside a band". **The
prediction was written down first** (`PREDICTION.md`, per the discipline the last four slices
graded themselves on) and then measured, in Python, with no Rust written:

| Check | Result |
|---|---|
| pint's contribution at all **6** live call sites | **exact identity, bit-for-bit** |
| **75 of 80** generated scalars | reproduce a declared YAML literal bit-for-bit — a plain decimal parse is all Rust needs (`float()` and `str::parse::<f64>()` are both correctly-rounded by spec) |
| the **4** folded values | reproduce exactly in the recorded op order |
| the 3 committed `.txt` files | current, not stale — `render()` reproduces each byte-for-byte |

**So nothing moves, and no golden is expected to.** ⚠ The pint result is worth stating
plainly because it re-prices the slice: the whole dimensional surface on the canonical path is
**two functions with six callers**, and every one of them is currently an **identity** (`convert`
is called with the unit the file already declares, wheat and potato alike). Every *other*
`unit:` in the tree — 18 `dimensionless`, 17 `degC`, 16 `1/day`, … — is **exact-string
compared**, never converted. §5c's surviving answer to §2e ("reimplement the dimensional check
in the Rust loader") is therefore a string compare plus a decimal parse, not a units library.

⚠ **The op-order trap named in the prediction turned out inert on today's values** — canopy's
`(sla·M_C)/cf` and `sla·(M_C/cf)` give identical bits, as do both nitrogen orders. That is a
fact about these four numbers, **not** a licence to re-associate: a value change could split
them. Copy the op order.

### ⚠ Two things the prediction did not anticipate, both found before writing code

**A. `heat_capacity: 1.0e7` is parsed by pyyaml as a `str`, and pydantic coerces it.**
`thermal/params/radiator.yaml`. YAML 1.1's resolver wants a signed exponent, so `1.0e7`,
`1e7` and `1.0E7` all resolve as **strings** while `1.0e+7` resolves as a float. This is
**exactly the hazard `rust/crates/authoring/src/yaml.rs` cites in its own docstring** as the
reason the reader was hand-rolled — and it is **live in the frozen param tree**, not
hypothetical. Bit-neutral (parsing the text as `f64` gives the same bits), but the Rust schema
layer must coerce a string-typed scalar where a float is expected or one param file fails to
load. The Rust reader's "numeric typing is deferred to the schema" design is already the right
shape for this; the rule just has to be written down.

**B. ⚠⚠ The Rust YAML reader CANNOT PARSE THE PARAM FILES AS THEY STAND.** `allocation.yaml`
and `crops/potato/allocation.yaml` write their partition tables in **flow style** —
`- {dvs: 0.0, fl: 0.55, …}` — and `yaml.rs` **actively rejects** it (verified in the code, not
read off the docstring: line 394 rejects any scalar opening with `{}[]&*!|>`, and
`flow_style_is_rejected()` is a test). **The closed subset the authoring platform froze does
not cover this project's own param files.**

Resolved by **reformatting the two tables to block style**, not by widening the grammar, and
the deciding measurement is that **flow style appears in exactly those two files** — zero
authored scenarios under `scenarios/` or `tests/authoring/scenarios/` use it. Widening a
frozen grammar to accommodate two data files it was never asked about is the larger change and
the wrong one. ⚠ Checked before choosing, because it decides which contract is touched: **the
authoring manifest does not name the YAML subset at all** (no such key; `grammar_note` is
about the *kinetics expression* grammar, `schema_fields` about the scenario schema) and
`docs/authoring-reference.md` never mentions it — the subset is documented **only in a Rust
source docstring**, outside the frozen contract. So widening would not have been an unfreeze
by the letter, and the reformat **is** a provenance edit against a pinned hash. ⚠ One note for
whoever revisits this: the subset's stated rationale is *"reconciling two independent YAML-1.1
implementations"*. **Under C there is only one**, so "the subset is closed for parse parity"
stops being a permanent argument.

⚠ **The reformat is the provenance-only unfreeze that NOTHING CATCHES, and it was checked
rather than assumed.** `test_frozen_param_set_is_complete` compares the param_files **key
set**; the recorded sha-256 **values are never compared** (`test_manifest_named_files_exist`
only asserts the files exist). So `allocation.yaml`'s hash moves and no test goes red — the
honour-system ceremony applies deliberately: advisor review, regenerate the manifest as the
git-visible record, document here. The **value** side is not honour-system:
`test_biosphere_params_in_sync` reddens if any number moves, which is the gate that makes
"provenance-only" a checkable claim rather than an intention.

### The design

1. **A new zero-dep `config` crate**, mirroring Python's `src/config/` — the boundary layer,
   below `domains`. `yaml.rs` **moves** into it and `authoring` depends on `config` and
   **re-exports `yaml` at its existing public path**, so no caller changes and the authoring
   surface is byte-identical. ⚠ **Not a second reader** — slice 5's negative-control lesson
   verbatim: *a policy with two implementations has one that is stale*.
2. **`domains` and `station` load the YAML themselves**, producing the same structs
   `params.rs` produces today, folds included, **in the recorded op order**. Bytes arrive via
   `include_str!` at the files' existing paths.
3. **The gate: every loaded value equals the committed `.txt` entry bit-for-bit.** ⚠ **The
   generators and their three generated files are RETAINED as the control** — they are what
   proves the Rust load is byte-equal, and §5c's own correction says a generator is not touched
   before its successor is green.
4. Split across two commits, because the halves have different risk: **(a)** the `config`
   crate + the sibling/station loaders (8 files, no folds, no flow style); **(b)** the
   biosphere loaders (15 files, both folds, the partition table, the reformat).

### ⚠ Explicitly NOT in C1 — named so they are deferred, not missed

* **The `param_files` re-anchor itself.** It needs three things C1 does not: **sha-256 in
  Rust** (there is none anywhere in the Rust tree, and all four engine crates are zero-dep by
  charter), a **newline-normalization rule** (`golden_sha256` is normalized; the box is
  Windows and CI is Linux, so an unstated rule diverges by platform, not by content), and the
  **census rule**. ⚠ The census is *not* a directory walk: the manifest names **15** files and
  the biosphere params directory holds **20** — the four `crops/potato/*.yaml` (the port has no
  potato; stage 2 is deferred) and `demo.yaml` (a skeleton, feeding two scenarios C6 retires).
  **The five are excluded for two different reasons**, which is *"a directory is not a
  category"* arriving one commit after it was written down. Python's own rule — a
  **non-recursive** glob minus `demo.yaml` — is in `_frozen_param_files()` and is what the
  successor mirrors. The station side is 8 files, 8 entries, no exclusions, so the rule is
  biosphere-only.
* **The weather path** (`gen_biosphere_weather.py`, the generator §5c found with no successor
  anywhere in C1–C7). It needs a JSON reader over a closed subset **and** ISO-date →
  day-of-year, and its fixture has to find a new home because `tests/` is being deleted, with
  **47** test files reading it. Params first, weather second: blended, a moved bit would have
  two candidate causes.
* **Relocating the param YAML out of `src/`.** Under C the files cannot stay in a deleted
  Python package. ⚠ The manifest keys are **basenames**, so the move would shift neither a key
  nor a hash — but **no slice in C1–C7 owns it**, and until it happens C1's `include_str!`
  paths reach out of the Rust tree into the Python one. That ugliness is the successor's
  trigger, recorded here so it is not mistaken for an oversight.

⚠⚠ **AN EXPIRY CONDITION, because this slice creates one and this repo's own record says an
expiry condition is what gets forgotten.** §5b's acceptance criterion for this slice was
*"removing the check must turn something red — a lint whose deletion changes nothing is the
defect this slice exists to avoid."* On the **Rust** side that is discharged
(`a_re_declared_unit_is_rejected` / `a_wrong_declared_unit_is_rejected` redden if the guard
goes). On the **Python** side it is discharged **only by accident, and only for now**: §2e's
trap is that `config/units.py` becomes a green test guarding a path that no longer executes,
and the single reason it has not happened is that **the retained generators still run the
Python loaders, so pint is still on a live path.** Those generators are retained *as C1's
control*, which means two of this slice's own decisions are load-bearing for each other.

**So: the moment `gen_biosphere_params.py` and its two siblings retire, `config/units.py` IS
the defect §2e named** — and retiring them is exactly what the "generator is retired once the
gate is green" rule sets up. Whoever takes that step owns §2e's Python half in the same
commit: either delete the check with its loaders, or give it a caller that provably executes.
⚠ It is written here rather than left to be noticed because *"an exemption written for a
temporary state is a deletion someone must remember"* — the lesson that left three checks red
for five commits.

### C1 — COMPLETE 2026-08-17, in two commits plus a third for a defect it uncovered

**Built.** A zero-dep `config` crate (Python's `src/config`, ported): the closed-subset
YAML reader **moved** into it from `authoring`, the `{value, unit, source}` entry schema,
the exact-string unit guard, and the bound helpers. `authoring` re-exports `yaml` at its
original path and its ~39 call sites compile **untouched**, carried by one
`From<ConfigError>` impl. `domains` and `station` now load all **23** frozen param files
themselves — 15 biosphere, 5 sibling, 3 station — schema, units, bounds and **both
core-ready folds** included.

**The gate is bit equality, in both directions.** `every_value_matches_the_generated_table`
(one per crate) compares `to_bits()` for all 66 biosphere scalars + the partition table,
12 sibling and 4 station values against the retained generated tables, **and** asserts the
two name sets match — a scalar the control names and the loader never reads would
otherwise pass unnoticed. **It passed on the first run**, which is what §5d's measurement
predicted. Python: **2471 passed, 5 skipped, 0 failed**. Rust: green, `ruff`/`pyright`
clean. ⚠ **The generators and their three `.txt` files are RETAINED as that control** —
§5c's rule that a generator is not touched before its successor is green, applied at the
moment it bites.

⚠ **The format-only unfreeze, and it is the one nothing catches.** `allocation.yaml` (and
the potato override) were reformatted out of YAML flow style. **No value moved** —
`gen_biosphere_params.py` reproduces its file byte-for-byte — and the manifest diff was
**predicted before regenerating and held exactly**: one line, `param_files["allocation.yaml"]`'s
sha-256, no golden hash. Ceremony run deliberately, since those hashes are recorded and
never compared. Full entry in `docs/biosphere-reference.md`'s unfreeze log.

⚠⚠ **A negative control found a live defect in the frozen reader, and it shipped as its
own commit.** The check asserting *"flow style is rejected, not silently mis-parsed"*
**failed**: the reader rejected flow style as a mapping **value** (`a: {b: 1}`, the only
form its `flow_style_is_rejected` test ever covered) and **silently mis-parsed** it as a
**sequence item** — `- {dvs: 0.0, fl: 0.55}` has a `key:` head, so the mapping path
returned the key `"{dvs"` and the value `"0.0, fl: 0.55}"` with **no error at all**. That
is the exact form this repository's own param files had been written in for years, so
**the one case the test missed is the one that mattered**, and any author writing a
flow-style list entry got a silent mis-parse instead of the documented rejection. Fixed on
the key side with one shared excluded-leader constant, the case added **to that test**
rather than a crate away, and measured inert on every existing file. *A test that names a
behaviour is not evidence it covers the case that matters.*

⚠ **Two §5d predictions were confirmed by construction rather than by luck.** `radiator.yaml`'s
`heat_capacity: 1.0e7` — the scalar YAML 1.1 resolves as a **string** — loads correctly
because the entry schema parses the scalar's *text* rather than trusting a resolver's
verdict, which is exactly what pydantic does; it has its own test over all four spellings.
And the two folds' **differing association** was copied rather than tidied.

**Asserted here for the first time:** the MUST-EQUAL constraint between `canopy.yaml`'s and
`nitrogen.yaml`'s `carbon_fraction`. Python has *documented* it since Phase 1 and enforces
it nowhere; a divergence models a plant whose leaf area and nitrogen thresholds disagree
about what a mol of carbon weighs.

⚠ **What C1 did NOT do, unchanged from §5d and now with a landed reason:** `param_files`
is still classified `python` in both manifests. Re-anchoring it needs **sha-256 in Rust**
(there is none in the tree, and every engine crate is zero-dep by charter), a
**newline-normalization rule**, and the **15-of-20 census rule** — three things that are
its own slice, not a rider on this one. The weather path and the relocation of the YAML out
of `src/` stand where §5d left them. ⚠ The `include_str!` paths now reach out of the Rust
tree into the Python package; that is the relocation's trigger, recorded so it is not read
as an oversight.

## §5e C2 — the twelve mathematical laws, COMPLETE 2026-08-17

Taken together with C8 on the user's instruction (*"do both"*), as separate commits.

### ⚠⚠ The plan's instrument was wrong for two thirds of the set, and the alternative is STRONGER

§5c said C2 was *"unchanged; `proptest`; independent of everything"*. The independence held.
The instrument did not. Measured against the actual sites: **eight of the twelve reference
laws are permutations of three or four elements.** Hypothesis samples ~100 draws from a space
of **6** or **24**; Rust enumerates **all** of it. That is a measured improvement over the
reference law, not a workaround. ⚠ And it is a *choice*, not a constraint — `proptest 1.11.0`
and its whole dependency tree are already in the local registry cache. The choice is that a
fifteen-crate dev-dependency, in a workspace whose engine crates are zero-dep by charter, is
the wrong price for the four laws that genuinely need generated values. Those four get a
deterministic LCG, **deliberately not `simcore::rng::mix64`**, because `CounterRng` is the
subject of two of them and seeding its case set from its own mixer is the self-referential
shape this project had to dissolve once already for the cross-port RNG vectors.

**What is given up is stated rather than implied:** shrinking. An exhaustive permutation
failure is already minimal; a generated failure is deterministic and reproducible from its
seed, but nothing narrows it.

### ⚠⚠ Three reference laws are UNFALSIFIABLE in Rust, and one such test was already here

The composition-fold, ledger-residual and `observe` laws shuffle the **insertion order of a
Python `dict`**. `State.stocks` is a `BTreeMap`: "insertion order" is not expressible, so a
shuffled build and a canonical build **are the same map**. `observation.rs`'s own
`insertion_order_independent` is exactly that shape today — its `forward` and `backward` maps
are identical before `observe` is ever called. The three are re-expressed on the axis that *is*
falsifiable here: the **value** of the fold, with fixtures whose sorted and reversed
accumulations differ in bits, plus a discriminator assertion so the claim cannot decay into a
tautology. ⚠ *A language feature can make a ported law inert without changing a word of it.*

### ⚠⚠ Nine controls; THREE came back green, and all three were defects in the new tests

This is the slice's real content. Each control mutates the reference and names the law that
must redden.

| Control | Reddens |
|---|---|
| the **flow** sort deleted in `Registry::new` | laws 1, 3, 7, 8, 10 |
| the **aux** sort deleted (a separate function) | law 2 only |
| the ledger fold walks the stocks in reverse | laws 4, 5 |
| `observe` emits in reverse | law 9 |
| `draw` shifts 10 instead of 11 | law 11 |
| a forcing read leaks the snapshot's stock | law 6 |
| `permutations()` returns only the identity | the enumerator meta-test |
| `Lcg::shuffle` becomes a no-op | laws 10, 12 (their spread meta-assertions) |
| `CounterRng` given sequential state | laws 11, 12 |

The three that came back green first time:

1. **The multirate law had no discriminator at all** — an order-independence law passing
   against a registry that never sorted. Its fast set now drains one stock with three flows
   spanning sixteen orders, and `dt / n_sub` is pinned to exactly `1.0` so the asserted
   magnitudes are the ones that reach the reduction.
2. ⚠⚠ **The ledger-residual law asserted sensitivity on the NOMINAL deltas while the ledger
   folds the RECOVERED ones.** `(1e8 + 0.1) - 1e8` is `0.09999999403953552`, and the recovered
   set cancels to exactly `0.0` in **both** directions. The discriminator is now read off the
   two states so it cannot drift from what is folded. ⚠ **The reference's own fixture has the
   same shape and its comment claims the opposite** (*"a naive (unsorted) sum would differ by
   ULPs under reordering"*); measured, both directions give `0.0`.
3. ⚠⚠ **The season law is inert by NATURE, not by fixture.** At the season's real physical
   magnitudes every per-stock leg sum is of comparable size, so re-associating them moves no
   bits — **realism cost the discriminator**, which is exactly what the reference's synthetic
   skeletons are for. It now also asserts the rebuilt registry's *iteration order*, which the
   same control does redden, and says in the file that #15 stays pinned by the engine-level
   laws. ⚠ *Re-homing a law onto a bigger, more realistic subject can make it weaker. Measure;
   do not assume the bigger subject dominates.*

⚠ And a fourth, caught earlier by the same mechanism: the discriminator helper rejected **three
of my own fixtures** as order-insensitive before they could ship green, including one that
cannot be sensitive at all — **a two-element float sum is commutative, and the reference's
integrator fixture drains its source with exactly two flows.**

### Law 3 had no Rust subject, and was re-homed rather than recorded as a gap

Checked, not assumed: there is no `demo` scenario anywhere in `rust/`, and **C6 retires the
Python one**, so porting the law onto it would mean porting a scenario scheduled for deletion.
`Registry::into_parts()` is public and its own docstring names rebuild-through as its purpose,
so the law lands on the **real season registry** instead. What that loses is named in the file:
`demo`'s topology (which nothing else exercises either — C6's whole point), the **RK4 arm** (the
frozen biosphere is Euler-only by charter, so an RK4 arm would test an unsupported
configuration; that arm survives on the engine-level subject), and the discriminator above.

**Additive throughout:** two new test files, `git diff src/` empty, no golden, no manifest, no
contract. Rust **348 → 363** tests.

## §5f C8 — `param_files` re-anchors, COMPLETE 2026-08-17

The successor C1 named, with its three stated blockers resolved: sha-256 in Rust, a
newline-normalization rule, the 15-of-20 census rule.

### ⚠⚠ The 23 digits are AUTHOR-NEUTRAL, so "param_files is now Rust's" is the wrong summary

Both trees compute a newline-normalized sha-256 of the same file under the same rule.
**Measured before a line of Rust was written:** all 15 biosphere + 8 station recorded hashes
reproduce under Python's rule *and* under the narrow rule Rust would implement. So the ceremony
was predicted value-free and **was** value-free — the entire diff across two contracts is the
`_authority["param_files/*"]` entry in each. What re-anchored is a pair of **rules**:

1. **The census** — the manifest now names the files the reference *loads* (a compile-time
   `include_str!` list) instead of the files a **glob of a Python package directory** finds. The
   difference is directional and it is the point: a param file added to the tree and wired into
   no loader used to *enter* the frozen surface; now it drops out and the gate says so.
2. **The normalization** — `config::provenance`, hand-rolled sha-256 (zero-dep charter) over
   LF-normalized text.

⚠ That is what the `_authority` `why:` says, in both manifests. Anything stronger would be a
claim nobody made.

### ⚠⚠ The newline rule is load-bearing TODAY — and the obvious explanation is FALSE

`git ls-files --eol` over the 24 param files: the index is **LF on every one** and
`.gitattributes` declares `eol=lf`, yet the **working-tree** copy of `senescence.yaml` on this
box is **CRLF**. So "autocrlf converts on checkout" — the story one would write — is wrong; it
would have hit all 24. What is true is narrower and worse: **`include_str!` embeds the working
tree**, so the reference's own compiled-in bytes for one frozen param file differ between this
box and Linux CI right now. Measured: the un-normalized digest for that file is `a7c55528…`
against the frozen `21163d3c…`. Without normalization the regenerated manifest would be red on
the other machine. ⚠ *A right conclusion reached by a wrong mechanism is still a rotten record
— run `git ls-files --eol` before writing a line-endings rationale.*

### ⚠⚠ The control pair exists; it is on the "who does the generator ASK" axis, not "whose bytes"

The naive reading — *both sides hash the same file, so there is no direction* — is wrong, and
was corrected by the advisor before the build. Slices 6–8's rule (a re-anchoring criterion must
name a **direction** per axis) is satisfied:

| Control | Manifest | Python conformance gate |
|---|---|---|
| a param file's **content** edited, not regenerated | pins bytes that no longer exist | **RED** |
| **Python**'s census rule changed (stop excluding `demo.yaml`), regenerated | **byte-identical** | **RED** |

### The other findings

⚠ **A free negative control was already on disk.** The 15-of-20 rule has **two** exclusion
mechanisms — four `crops/potato/*.yaml` by **non-recursion**, `demo.yaml` by **name**. A
recursive walk picks the potato files up, and **two of the four share a basename with a frozen
file**, so it would not merely add names, it could overwrite a frozen hash in place. Asserted
both ways.

⚠ **An in-crate test that looked stronger than it was, replaced.** The first sha-256 padding
coverage asserted the padded length's invariants — by **re-deriving the padding with a copy of
the implementation's own loop**. Two copies of the code under test are not an oracle: if the
loop were wrong both would be wrong together and the test would be green. *A policy with two
implementations has one that is stale*, in its sharper form — they were **guaranteed** to agree.
Replaced by 201 digests minted from CPython's `hashlib` (OpenSSL, no shared ancestry) plus the
four published FIPS 180-4 strings. ⚠ Those vectors are a **published-standard** artifact with
an external authority, not the kind of Python-generated file §5c schedules for retirement.

⚠ **And the frozen param files are not that coverage.** Measured: only **two** of the 24
normalized lengths (`phenology.yaml` at 58, `radiator.yaml` at exactly **56**) land in the
`len % 64 >= 56` window where the length field spills into a second block; **no** file is a
single block and none is empty. That coverage is *incidental* — a one-character content edit
removes it and nothing would say so.

⚠ **Newly asserted, and nothing had checked it: basename uniqueness across the station's six
directories.** `param_files` is basename-**keyed**, so a name in two directories would collapse
two files into one entry. Python's `_param_paths()` *documents* uniqueness and its dict would
quietly keep whichever directory it read last.

⚠ **A forcing function fired exactly as designed, and its own message was the thing to fix.**
Adding the key to the dumps reddened the parity gate first, whose failure text read *"a
param-file list would make this gate compare Python against Python"* — the claim C8 refutes.
Three separate places said that (both generators' `_RUST_DUMP_KEYS` comments and the gate's
message) and all three were rewritten to record *why it was true until C1 and is not now*,
rather than deleted. ⚠ *A forcing function that was right for three slices is not wrong when it
fires; it is asking whether its premise still holds.*

**Retained with meaning inverted, not deleted:** `_frozen_param_files()`, `_param_paths()` and
`_normalized_sha256()` stay as **conformance checks on the checker** — the treatment slices 6
and 8 gave the flow set and the authoring rosters. Deleting them would have thrown away the
only thing that says the two hash rules still agree. And `EXOTIC_LINE_SEPARATORS` makes the one
place the rules *could* differ **unreachable** rather than merely unobserved: Python's
`splitlines` breaks on eight characters the narrow rule does not, and a gate asserts no frozen
param file carries one.

**Prose half updated in the same pass** (*the freeze's prose half is ungated*): both reference
docs' authority tables and unfreeze logs, and `docs/param-file-conventions.md`, which gains a
line-endings section and two checklist items — including that a whitespace-only edit is an
unfreeze that reddens nothing.

### ⚠⚠ Two controls caught a roster with no consumer — in this slice's own new gate

C8's first draft added `_COMPARED_MAPPINGS = frozenset({"param_files"})` to
`tests/crossport/test_inventory_parity.py` and **the loop that consumes it never landed**: a
string replacement matched nothing and I did not assert that it had. The constant read exactly
like coverage, and **nothing in the tree disagreed** — the dump's key-set forcing function was
green (it checks key *sets*, not comparisons), `ruff` does not flag an unused module-level name,
and the full 2473-test suite was green.

**What found it was running the negative controls.** Both of the two that should have reddened
the parity gate — a param file edited without regenerating, and newline normalization removed
from the reference — left it **passing**. Fixed; both now redden it, and the comment above the
loop records why it exists.

⚠ *This is slice 8's lesson with the roles swapped.* There, a control had no test to redden and
that absence WAS the finding. Here the control had a test and the test had no assertion — the
same class of hole, reachable only by actually running the control rather than by reading the
diff.

⚠ **A useful split fell out of it:** control D reddens the **parity** gate on both contracts and
correctly leaves the **checker's** digest gate green, because the checker still agrees with the
frozen value and it is the *reference* that moved. The two gates answer different questions, and
the control is what demonstrates which.

### ⚠⚠ The closing pass: two prose-only claims, and a near-miss on scope

The advisor's closing review found two assertions that existed only as prose. Both are now
measured, and one of the measurements is better evidence than the claim it was written for.

* **"A recursive walk reddens the census"** appeared in the manifest's `_authority`, in
  `docs/biosphere-reference.md` and in this plan, and **nothing had run it**. Flipping the
  real walk to descend reddens `the_census_matches_the_directory_on_disk` on its roster
  assertion — and the failure output shows the sharp half concretely: the recursive listing
  contains `allocation.yaml`, `canopy.yaml`, `phenology.yaml` and `root_depth.yaml` **twice**,
  so a basename-keyed manifest could *overwrite* frozen entries rather than merely gain them.
  A permanent test now reproduces the mistake rather than the fix.
* **Basename uniqueness had no control.** The control that looked like one planted a duplicate
  file *on disk*, which reddens the **census**, not uniqueness — the assertion is over the two
  **compile-time include lists** and is unreachable from disk. The real control adds a
  duplicate to a *list*. ⚠ And the directory-level claim both manifests advertise is
  **composed** from two gates (list uniqueness + per-directory census), which is now said
  rather than implied.

⚠⚠ **The near-miss is the more useful finding.** Having measured that the *param files'*
index is LF everywhere, I generalised that to the repo and normalized every CRLF working-tree
file. git reported **191 files modified** — including files under `src/`, where the purity
invariant requires an empty diff — and it was reverted in full. *A measurement's scope is part
of the measurement:* "the index is LF" was true of the 24 files checked and false of the 681 in
the repo. The conventions page records the practical rule (fix the one file, never sweep) and
deliberately stops short of a mechanism it cannot reproduce.

⚠ **Slice 6's recorded mistake repeated:** reverting a control with `git checkout <file>`
discarded the uncommitted edits in that same file. It cost nothing only because the edit was a
**script**, not hand-typing — which is the mitigation worth generalising, given that "snapshot
before running controls" has now failed twice.

### ⚠ Still NOT done, with reasons

* ~~**The weather path** (`gen_biosphere_weather.py`) — still the generator with no successor
  anywhere in C1–C8.~~ **DONE in C9** (§5g), except the fixture's new home, which moved to the
  relocation item below where it belongs (it is the same move as the param YAML).
* **Relocating the param YAML out of `src/`** — C8 makes the reach-out **worse**, not better: a
  runtime directory walk now joins the compile-time `include_str!`. That sharpens the trigger.
* **Retiring `gen_*_params.py`** — still C1's control, still carrying §2e's expiry condition.
* **The light-path fingerprint's hashing.** Its dump docstring says the hashing is left to
  Python "because … this crate has no digest dependency". **C8 falsifies that clause's
  premise.** Not acted on — moving it is a second re-anchoring with its own control, and this
  slice already had two contracts open.

## §5g C9 — the weather path, COMPLETE 2026-08-17

The generator §5c found with **no successor anywhere in C1–C7**, scheduled into Stage 1 and
built. The reference now reads `tests/oracle/winter_wheat_weather.json` itself, through a
closed-subset JSON reader and an ISO-date → day-of-year computation added to the `config`
crate; `tests/crossport/gen_biosphere_weather.py` and the `weather_facts.txt` table it wrote
are **deleted**, and so is the Python gate that kept them in sync.

### The gating measurement, taken before any Rust was written

Same discipline as C1 and C8: predict the diff, measure it, *then* build. The question was
whether Rust's `f64::from_str` on the fixture's decimal literals reproduces the bits the port
was reading out of the Python-generated hex-float table. **All 916 values** (latitude + 3 ×
305 observations) came back **bit-identical** — both conversions are correctly rounded, so
they must, but this repo's rule is that "must" is measured. So C9 is a **re-anchoring**, not
an unfreeze: no golden moved, and the only diff in any contract is prose.

⚠ The measurement was run through a throwaway `rustc` program over the literal text, *not* by
reading the two files by eye — the literal is what `f64::from_str` sees, and `json.loads`
discards it. `json.loads(..., parse_float=str)` hands it back, which is what made a
916-value comparison a five-line script.

### ⚠⚠ The one piece of C9 no golden can check, and it is the only new logic

The fixture runs **2006-10-01 → 2007-08-01**: neither year is a leap year and the span holds
no 29 February. So the leap rule in the new `iso_day_of_year` is **unreachable from the
data** — the naive `year % 4 == 0` produces byte-identical output on every row.

**Measured, not reasoned:** with the leap rule broken to the naive form, `cargo test -p
domains` is **48 passed / 0 failed**, including the bit-for-bit control against the generated
table and every season run. Only the hand-computed calendar tests in `config::date` go red,
and they exist *because* of this — they carry 1900 and 2000, which the fixture cannot reach.
*The float parse was the safe part; the calendar was the risk, and the two have opposite
test-ability.*

The other two controls did redden: reversing the row order fails **five** tests, two of them
season runs (so order is genuinely gated, not merely asserted), and the JSON number scan is
covered by the forms Rust accepts and JSON does not (`inf`, `NaN`, `+1`, `007`, `1.`).

### ⚠ A rationale in the frozen contract was falsified by this slice, verbatim

`_AUTHORITY["forcing/weather_fixture"]` said the key was Python's *"because the port reads
`weather_facts.txt`, generated FROM it"*. After C9 there is no such file. The **side** is
unchanged and the **reason** is now a different one, which is exactly the case the authority
map's own header anticipates ("with the reason stated and, where one exists, the condition
under which that could change"). Both `why` strings were rewritten and the manifest
regenerated; **the entire diff is those two strings.**

⚠ Why the pair did **not** re-anchor to Rust, having been measured buildable: the reference
embeds the fixture with a compile-time `include_str!`, so it knows the **bytes** and not the
**name** — `include_str!` takes a literal, not a `const`, so a Rust-authored `weather_fixture`
would be a hand-typed duplicate of the include path, *a literal dressed as a derivation*. And
re-anchoring the hash alone would manufacture the split authority slice 6 exists to record.
The condition is named: **the relocation slice** gives the fixture a data home the reference
can read at runtime (as C8's param census already does), and then both keys move together.

### What C9 built instead of that re-anchor — and it guards the reach-out

The `locked_dt_days` pattern: the dump emits `weather_sha256` **to be checked, never
spliced**. The checker hashes the file it finds on disk; the reference hashes what it
*compiled in*; `test_the_weather_hash_matches_the_reference_tree` fails if they stop being
the same bytes. They agree today (`cd61457e…`, the value already frozen).

⚠ This is not a hypothetical guard. C9's `include_str!` **climbs out of the Rust tree into
the Python one** — the same ugliness C1's param loads have — and the relocation slice is
*scheduled* to disturb exactly that path. Without this gate, a move that updated one side and
not the other would leave a stale copy driving one side with a green suite.

### ⚠ C1's expiry condition does NOT apply here, and the reason is worth writing down

§5d warns that retiring `gen_biosphere_params.py` and its siblings turns `config/units.py`
into §2e's defect, because those generators are the last thing keeping pint on a live path.
**`gen_biosphere_weather.py` is not one of them**: it imported `json`, `datetime` and
`pathlib` and never touched a Python loader, so deleting it moves nothing about pint. The
warning stands, undischarged, for the three param generators — this deletion is not a
down-payment on it.

### Deliberately NOT in C9

* **The fixture's new home.** Named in §5c as part of this work and moved out on purpose: it
  is the *same* move as relocating the param YAML out of `src/`, which no slice owns yet, and
  splitting it would put the reference's data in two places for one slice's convenience.
  Doing it here would also have moved **47 Python test files** for a slice that otherwise
  touches none.
* **The other five oracle fixtures.** They travel with the carve-out, not with this.
* **The light-path fingerprint's hashing** — still open from C8, still its own re-anchoring.

## §6 Open questions — none blocking, all answerable when their slice is taken

1. **Does new *reference science* wait for the flip?** B makes Rust the place science is
   authored, but slices 6–8 are where that becomes true. Work taken before then is still
   Python-canonical. ⚠ Do not start a science item and a re-anchor slice in the same batch.
2. **What happens to the deferred mirrors already on the books?** Potato stage 2 (the Rust
   habitat mirror) is deferred. Under B a Python-side item with no Rust mirror is now a gap
   in the *reference*, not in the copy — so it changes category.
3. **Does the Godot consumer notice?** — **ANSWERED 2026-08-16 (slice 5): no.** Checked
   rather than assumed: no test under `tests/crossport/test_godot_*.py` references either
   moved golden, and the only two goldens any of the nine name at all are
   `cabin_gas_state.json` and `crew_state.json` — both byte-identical between the ports
   and untouched. Nothing under `godot/` references them either.

## §7 What follows B — the user's stated next subject (recorded, not planned)

**Recorded 2026-08-16, in the same breath as "begin slice 1", so the sequencing is explicit
and not inferred.** The user's words: after the switch to Rust, *"work will continue on the
universal harness, that permits easy toggle of parameters and science, and will ease the
need to run tests and constant verification."*

This is a **direction note, not a plan** — nothing here is designed, priced or scheduled,
and it changes nothing about the eleven slices. Its purpose is to keep the ordering visible:
**the harness comes after the flip, so a slice must not be re-scoped to serve it.**

Two things it connects to, both already on the books:

* The harness already has a plan and a built foundation:
  `post-roadmap-value-switch-harness.md`. Its seam — scoped in-memory parameter overrides
  in `src/config/overrides.py` — shipped 2026-08-15; **its reporting layer, which is the
  actual deliverable, is still open**, and so is the extinction-coefficient value the seam
  was built to settle. "Toggle the science, not just the numbers" is a widening of that
  doc's scope, not a new subject.
* ⚠ **The seam is Python-side, in `src/config/`.** Under B that is the *checker's* half of
  the tree. Whether the harness is rebuilt against the Rust loader, or stays Python and
  drives Rust, is a real design question — and it is the same question as slice 9's unit
  validation (§2e), which is about the same file set for the same reason: after the flip,
  Rust is what loads the params. **Take them together or take slice 9 first.**

⚠ **One phrase is recorded as the user's goal and not adopted as a design property:
"ease the need to run tests and constant verification."** Read as *"experiments must be
cheap enough that you don't hand-write a probe script for each one"* — the measured
motivation behind the existing plan, 42 throwaway probe scripts across 16 plan docs — it is
exactly right and is what the harness is for. Read as *"the harness stands in for
verification"* it inverts this repo's whole posture, and every mechanism named in this
document (the goldens, the three manifests, the tolerance contract) exists because that
substitution is not available. The distinction is not a quibble about wording: it decides
whether the harness's output is *a finding you then gate*, or *a gate*. It is the first.

---

## §5h C5 — `drift.py`'s folds move to Rust; measured 2026-08-17, BEFORE any Rust was written

### ⚠⚠ The order changed: C5 comes BEFORE C4, and the dependency was measured, not guessed

The user picked C4 (the science-gate census) next, on my recommendation. Measuring C4's
surface first surfaced a hard prerequisite that neither §5c nor my own recommendation saw:
**5 of the 15 science gates cannot be written in Rust until `drift.py`'s folds exist there.**

| gate locus | folds it needs |
|---|---|
| `test_decade_stability.py::test_decade_leaf_cycle_is_stationary` (×2 — perennial + consumer) | `year_summaries`, `same_phase_diffs`, `is_stationary`, `non_collapsing` |
| `test_decade_stability.py::test_decade_consumer_biomass_is_stationary_and_alive` | same |
| `test_decade_stability.py::test_decade_min_carbon_pool_stationary` | same + `drift_slope` |
| `test_sealed_station_stability.py::test_tier1_node_is_period_1_fixed_point` | `year_summaries`, `same_phase_diffs`, `is_stationary`, `non_collapsing` |

The evidence is not inference — **both Rust emitters say so in their own comments**:
`emit_drift.rs:5` (*"the Python parity gate folds them … so this example reproduces NO
segmentation"*) and `emit_sealed_energy_drift.rs:6-7`. And `grep` for
`stationar|period_2|non_collapsing|year_summaries|least_squares` over `rust/crates`
returns **only those two comments** — Rust has zero fold functions.

*Generalize: a slice's prerequisites are not in its own row of the plan table.* §5c lists
C4 and C5 as independent items in the same stage; they are not. Had C4 been taken first,
the folds would have been written inside it and C5 would have become a duplicate or a
no-op — which is how a plan silently loses a slice's subject.

### ⚠⚠ The gating measurement: C5 SPLITS — bit-neutral for one golden, blocked for the other

Same discipline as C1/C8/C9 — predict the diff, measure it, *then* build. The question here
is **not** a locus question (that is C4's) but a floating-point one: do the two goldens come
out byte-identical when Rust folds instead of Python?

The measurement runs the two Rust emitters, folds their raw series in Python with the
*existing* fold code (`_fold_drift_summary` in `tests/crossport/test_crossport.py` does
exactly this, so the comparison is apples-to-apples), and byte-compares against the
committed goldens.

| golden | result |
|---|---|
| `sealed_energy_drift_summary.json` | ✅ **byte-identical** — bit-neutral, no unfreeze |
| `drift_summary.json` | ❌ **4 of 45 values move**, ≤7 ULP → **authorship DEFERRED**, see below |

The four, all in the **consumer** scenario and all in years 3–4 (the perennial is identical
throughout, and both `is_period_2` booleans are unchanged):

```
consumer.peak_leaf        yr 3   ulp=7   rel=9.955e-16
consumer.peak_leaf        yr 4   ulp=1   rel=1.493e-16
consumer.consumer_carbon  yr 3   ulp=2   rel=4.389e-16
consumer.consumer_carbon  yr 4   ulp=2   rel=2.289e-16
```

**The control that makes this a statement about the ports and not about my fold:** the
Python fold of the *Python* trajectory still reproduces the committed golden **exactly**
(`uv run pytest tests/test_regression_long_horizon.py -k drift_summary` — 3 passed), and my
Rust-fed fold reproduces **41 of 45** values bit-for-bit across three different summary
functions. So the fold arithmetic is exact and the difference is in the trajectories: the
two ports' 15-year *consumer* runs differ at ~1e-15.

### ⚠ A dated figure in the tolerance contract is no longer true, and C5 is what makes it matter

`tests/crossport/tiers.json`'s `drift_summary` evidence reads *"Step-4 (P7.4): Rust-vs-Python
bit-exact locally (same UCRT libm, **max_rel_dev 0.0**)"*. Measured 2026-08-17 that is
**9.955e-16**, not 0.0. Nothing is red — the band is `1e-11`, ~4 orders looser — and the
string is explicitly dated `P7.4`, so this is a stale figure rather than a false claim.

What makes it worth writing down is that **C5 changes its status**. Today those four ULPs are
a *tolerated deviation* absorbed by a band. Once Rust folds, they become **the golden's own
bytes** — the deviation stops being tolerated and becomes the reference value. That is
precisely what the reference flip means, and it is why this slice is an unfreeze and not the
re-anchoring C1/C8/C9 were.

*Generalize: a tolerance band hides a difference until the flip makes one side the author —
then the same number is a value, not a deviation.*

### Where the Rust fold goes, and why it mirrors Python

`domains::biosphere::drift` — `rust/crates/domains/src/biosphere/drift.rs`. Python puts the
kit in `domains/biosphere/drift.py` and the *station* tests import it from there; `station`
already depends on `domains` in its `Cargo.toml`, so the mirror reproduces the layering
rather than inventing one. The module is an **instrument**, not science: it is generic over
`State` (the caller supplies the per-year `summary_fn`), so it imports no stock-id catalog.

### Classify per function before porting — some of this is not a fold

Advisor's note, applied: `drift.py` is 265 lines and the folds are only part of it.

| function | kind | port? |
|---|---|---|
| `total_quantity`, `mass_drift_trace`, `drift_slope`, `max_abs` | trace **builders** over a trajectory | yes — no Rust equivalent exists (`grep` for `fn total_quantity` returns nothing) |
| `least_squares_slope` | the shared primitive both axes use | yes |
| `year_summaries`, `same_phase_diffs`, `is_stationary`, `non_collapsing` | the **folds** C4 needs | yes — the point of the slice |
| `is_period_2` | the discrete structural check | yes — it decides two booleans *in* the golden |
| `MASS_DRIFT_ABS_BOUND`, `MASS_DRIFT_SLOPE_BOUND` | derived constants with a provenance block | yes, **with the provenance comment** — a bound nobody can reproduce is the thing the block exists to prevent |

⚠ `relative_drift` is **not** in `drift.py` — it is in `tests/sealed_tier2_helper.py`. It is a
test helper, so it travels with Stage 3, not with C5.

### ⚠ The segmentation convention is the real porting risk, and it is off-by-one shaped

The arithmetic cannot drift — every float in both goldens is a *selected* value (`max` over a
segment, or the segment's last element), not a computed one, and the only computed outputs
are booleans. What *can* break is the segmentation:

* `year_summaries` slices `states[y*year : (y+1)*year + 1]` — **inclusive of the next year's
  boundary state** — and takes `n_years = (len(states) - 1) // year`.
* Both emitters stream `steps + 1` states (measured: `emit_drift` → 18301 = 15×305×4 + 1;
  `emit_sealed_energy_drift` → 109801 = 15×305×24 + 1), so the `+1`/`-1` pair is load-bearing.
* ⚠ The period is in **steps** for the biosphere (`steps_for(305)` = 1220 at `dt = ¼`) and in
  **days** for the sealed station's organic-carbon fold (`run_master_day` appends one state
  per master day). The two are different units in the same function; this is the trap
  `docs/plans/post-roadmap-step-unfreeze.md` §1 already records once.

A Rust fold that gets the inclusive boundary wrong changes **values**, not bytes-by-a-ULP —
so the 4-ULP prediction above is also the check: any other diff is a porting defect.

### Deliberately NOT in C5

* **The science gates themselves** — that is C4, and it starts the moment the kit lands.
* **`relative_drift` and the other `tests/` helpers** — Stage 3.
* **Re-anchoring `tiers.json`'s stale `max_rel_dev` figure** — it is a *dated* evidence
  string in the tolerance contract, whose own unfreeze discipline is
  `docs/native-port-reference.md`. Naming it here is the record; changing it is its own
  ceremony.

### C5 — COMPLETE 2026-08-17, with one half deferred for a measured reason

**Built.** `rust/crates/domains/src/biosphere/drift.rs` — the trace builders
(`total_quantity`, `mass_drift_trace`, `drift_slope`, `max_abs`), the shared OLS
primitive, the four folds C4 needs, `is_period_2`, and the two derived bounds **carried
over with their whole provenance block**. 16 integration tests in
`rust/crates/domains/tests/drift.rs`. `emit_sealed_energy_drift` now emits the summary
rather than a raw series; `_fold_energy_drift_summary` deleted from
`tests/crossport/test_crossport.py`. Rosters, census counts and the station manifest's
`_authority` updated. Ruff / ruff-format / pyright clean, clippy `-D warnings` clean, 47
targeted Python gates green.

#### ⚠⚠ The diagnosis the guard demanded, and it is what split the slice

The first design was: regenerate `drift_summary.json` from Rust, add it to
`golden_platform.PYTHON_DIVERGES` (Python becomes tolerance-gated, exactly as slice 5 did
for its two stragglers), done. That is **red by construction**, and the thing that says so
is a gate written two slices ago:
`test_golden_provenance.py::test_every_diverging_scenario_keeps_a_byte_gated_sibling`.
`emit_drift` serves **exactly one** golden, so under that gate's key — the emitter program
— the entry has no byte-gated sibling. Its own error message is the instruction:
*"Diagnose the divergence instead of adding it to the roster."*

So it was diagnosed rather than routed around. Comparing Python's per-step consumer
trajectory against Rust's:

* the first difference is at **step 4095 (year 3), 1 ULP**;
* **1750 of 18301** steps then differ;
* and the **final state is byte-identical again**.

That is the whole asymmetry in one line: a 1-ULP transcendental divergence appears
mid-run and the **contracting attractor damps it back to exactly zero** by year 15. The
final-state golden (`consumer_long_horizon_state.json`) is byte-gated and stays green
*because* the attractor contracts; the per-year peaks are the only artifact that samples
the trajectory **while the difference is still alive**. Not a port defect, and not
fixable.

#### ⚠⚠ Why the gate was NOT widened, when widening it is defensible on the merits

The gate's key is the **emitter program**, used as a proxy for "the scenario". The proxy
under-approximates: `emit_drift` runs the same two scenarios `emit_consumer` and
`emit_perennial` run, and both of those keep a byte-gated Python golden. So adding
`drift_summary.json` would remove **no** byte coverage that exists today, and the gate
would be firing a false positive.

It was still left alone, and the reason is the one this repo keeps re-learning: *that
argument was constructed while trying to add the entry it licenses.* The gate exists
precisely because "coverage survives" was once a sentence someone wrote down, and slice 5
converted it into an assertion so it could not be re-argued. Re-arguing it from inside the
slice that needs it relaxed is the co-adaptation refused at
[`post-roadmap-stem-only-branch`] and at the canopy floors. If the key really is too
narrow — and it may be — that is **its own ceremony with its own control**, and the
control has to be written by someone not holding an entry they want admitted.

⚠ **The contract-preserving alternative was priced and does not hold.** Have `emit_drift`
also emit the two final states, so it serves goldens with byte-gated siblings and the gate
passes unwidened. `Emitter.run()` returns a program's **entire stdout** as one golden's
bytes — one program, one artifact — so `emit_drift` cannot serve a second golden without
re-pointing an existing one away from `emit_consumer`. That manufactures the sibling
rather than discovering it, which is the same move wearing a different hat.

**So `drift_summary.json` stays Python-folded, and `PYTHON_FOLDED` now carries the
measurement and the blocker instead of the old "the fold has no Rust referent" reason —
which C5 made false.** The kit exists; only the authorship move is outstanding.

#### ⚠ A port defect the code review would not have found, and the test that did

`year_summaries`'s `n_years = (len(states) - 1) // year` is `-1` on an empty trajectory in
Python, and `range(-1)` is empty, so it returns `[]`. The literal Rust transcription
**underflows `usize`** — a debug panic, and in release a near-`usize::MAX` year count. It
was found by porting `test_drift.py`'s own `test_year_summaries_handles_short_trajectories`
rather than by reading the code, and the guard has a **measured control**: reverting
`saturating_sub` reddens exactly that test and nothing else.

*Generalize: port the reference's edge-case tests before its happy path — the edge cases
are where two languages' arithmetic disagrees about what an expression even means.*

#### ⚠ What C5 changed about a dated figure, without touching it

`tests/crossport/tiers.json`'s `drift_summary` evidence still reads *"Step-4 (P7.4):
Rust-vs-Python bit-exact locally (max_rel_dev **0.0**)"*. Measured 2026-08-17 it is
**9.955e-16** — inside the `1e-11` band by ~4 orders, and the string is explicitly dated,
so nothing is wrong and nothing is red. It is recorded here because **C5 is what would
change its status**: while Python authors the golden the four ULPs are a *tolerated
deviation*; the moment Rust authors it they are *the reference value*. Re-anchoring that
string belongs to `docs/native-port-reference.md`'s own discipline, not to this slice.

#### What this leaves for C4

The kit is in place and reachable from both crates, which was the blocker. C4's five
fold-dependent gates (`test_decade_stability.py` ×4, `test_sealed_station_stability.py` ×1)
now have Rust referents for every helper they call. ⚠ C4's own gating measurement — the
15 `locus` strings, and whether each `bound`'s numeric literals still appear textually in
the **Rust** file the locus will point at — is still unmeasured and still blocks C4's
ceremony, not its start.

## §5i C3 — the posture lands in `CLAUDE.md`, COMPLETE 2026-08-18

The slice §5's table numbered 11 and the C re-plan renumbered C3: rewrite the purity
invariant, add the development-posture section `CLAUDE.md` has never had, adjudicate the
`windows_golden_only` marker §5's slice-5 block left here, and close the record out.

**Nothing executable changed.** `git diff src/`, `git diff rust/`,
`tests/regression/golden/` and all three `.manifest.json` came back empty, as predicted
in `M:/claud_projects/temp/c3-posture/PREDICTION.md` before the first edit. This slice
authors no science and re-anchors no key; it *describes* the re-anchoring that slices 6–8
and C1/C5/C8/C9 already did.

### ⚠ Three statements in the always-loaded file were false, and only two were the flip's

The two the slice was chartered for (`Python is the canonical reference`; the port has no
reference authority / `git diff src/` empty) were known. The third was not: the
freeze-contract table still read **`Euler/dt=1`** — stale since the step unfreeze of
2026-08-14, four days before this slice and *nothing to do with the flip*. It survived a
step change that was itself run as a ceremony with a plan doc, a manifest regeneration
and a re-pinned literal.

⚠ *Generalize: a file that is loaded unconditionally is read constantly and audited
never. The ceiling test bounds its SIZE and the parity tests bound its INDEX; no gate in
this repo compares a sentence in it against the tree. Every unfreeze should re-grep it —
`dt` went red in the manifest, in a test literal, and in five prose figures, and this
line was in none of those places.*

### The marker decision, refused on a number rather than on caution

§5's slice-5 block left this: `windows_golden_only` still skips the two
tolerance-converted gates on Linux, and its stated rationale (byte-exactness is
platform-bound) no longer reaches a comparison that is no longer byte-exact. The tempting
move is to unskip them and let `DISAGREEMENT_CEILING = 1e-14` do the work.

**Refused, and the reason is arithmetic.** The worst propagated ±1-ULP transcendental
sensitivity in this scenario group is **3.520e-15** (`tiers.json`, canopy `exp`, perennial
15-yr, re-measured 2026-08-16) — under **3×** of headroom below the ceiling for a *single*
perturbed site, while glibc-vs-UCRT perturbs all four sites `tiers.json` lists for these
scenarios at once and by more than one ULP at some. The evidence available argues
*against* the assumption that a glibc run fits, so unskipping would ship a band nobody
measured — the derived-not-measured move this contract exists to refuse.

The comment landed at the marker definition (`tests/golden_platform.py`) with an
**expiry condition** naming the one thing that retires it: a Linux run of the two
`*_matches_the_reference` tests reporting their max observed deviation. ⚠ Written as an
expiry condition on purpose — this doc's own hardest lesson is that *an exemption written
for a temporary state is a deletion someone must remember*.

### The byte budget: the prediction was right about the direction and 6 B short on size

Predicted 10,200–10,900 B newline-normalized against a 12,000 B ceiling; the first draft
landed at **11,452** (548 B of headroom) — inside the ceiling but past the band, and past
this slice's own written trigger (*"if it lands over 11,500, cut; do NOT raise the
ceiling"*). Cut by **retirement, not by trimming adjectives**, which is rule 1 of
`docs/context-budget.md`:

* the whole `## Status` section — it duplicated the header's "roadmap COMPLETE through
  Phase 9", the detail list's log pointer, and Working style's retirement rule. Replaced
  by one `docs/phase-index.md` line in the pointer list. **A section whose every sentence
  is a second copy is the exact shape this file warns about.**
* the suite-runtime paragraph, compressed to its two operative rules.

Final: **10,906 B, 1,094 B of headroom**, all 10 context-budget assertions green.

### What C3 deliberately did NOT do

* **Delete anything.** The posture now says `src/` shrinks; acting on that is C4/C6/C7 and
  Stage 3, each with its own control.
* **Touch the `Native port` contract row.** The cross-port tolerance contract dies with
  the second implementation, not with the sentence describing it.
* **Re-word `config/units.py`'s expiry into an assertion.** C1 recorded that the Python
  unit check stays live only while the retained param generators call it; `CLAUDE.md` now
  carries the warning, but the gate that would make it red is still C7's to build.

## §5j C4 — the science-gate census: the gating measurement, taken 2026-08-18 BEFORE any design

Same discipline as C1/C5/C8/C9: measure first. C5's closing note said C4's ceremony was
blocked on an unmeasured question — *"the 15 `locus` strings, and whether each `bound`'s
numeric literals still appear textually in the Rust file the locus will point at"*. This
section takes that, and the measurement moved the slice's shape three times.

**Method.** A throwaway probe (`M:/claud_projects/temp/c4-science-gates/`, a temporary
example inside the workspace, deleted after; the tree came back clean) computes in the
**reference** every quantity the markers assert on, and a Python script computes the same
in the checker. Both outputs are committed to the probe directory. The Rust numbers were
taken in `--release`; slice 4 measured the profile byte-neutral for this family, and it is
said out loud because the permanent gates will run in `cargo test`'s debug profile.

### The result: verdict-neutral, and the only movement is the one C5 already owns

| | |
|---|---|
| keys compared | **39** |
| byte-identical | **37** |
| differing | **2 keys, 4 values** — `consumer.peak_leaf` yr 3–4, `consumer.year_end` yr 3–4 |
| worst | **7 ULP**, rel 9.955e-16 |
| verdicts (`is_stationary`, `non_collapsing`, `is_period_2`, every band) | **identical on both sides** |

Those four are **the same four C5 measured** on `drift_summary.json`, arrived at
independently through a different fold path — corroboration, not a new finding. Every
band's margin is orders above them, so no gate's verdict is reachable from that noise.

### ⚠⚠ THE MEASUREMENT COVERED 12 OF 15 GATES ON THE FIRST PASS, AND THE WRITE-UP SAID 15

Caught in review before it was recorded (advisor). The claim *"the 15 gates port
value-for-value"* was drawn from a probe that never exercised three of them, and the three
are exactly the ones that change the slice's scope. *The same shape as "the plan's own
arithmetic was wrong: the gap is 7 goldens, not 1" and "that census surveyed one gate" —
a count taken from the roster you built rather than from the roster you measured.*

**Gate 1 — the mutual-shading disjunct, and the probe's own arithmetic proves it is the
load-bearing half.** `bound = "peak < 6.0 OR the 5%/day mutual-shading loss is MODELLED"`,
and the probe measured `open_season` peak LAI = **6.0228** — *above* the threshold. So the
gate passes today only through the MODELLED branch, which the first pass did not measure at
all. Measured on the second pass: `shade_rate` 0.05, `lai_threshold` 6.0, `rdr_leaf` 0.02,
and `mutual_shading_rate` reads `0.02` **at** the threshold (inert, strict `>`) and `0.07`
just above it — all four byte-identical across the ports.

**Gates 2 and 3 are not biosphere at all, and that splits the slice** (see below).

### ⚠ An inert comparison, caught the same way

The first pass called Rust's `is_period_2(&p_peak, 8, 1e-2)` against Python's
`is_period_2(summaries, transient=8)` — whose `min_rel_gap` **defaults to 1e-3**. Both
returned `false`, which looked like agreement and was two different parameterizations
landing on the same answer. Re-run at `1e-3` on both sides: still `false`, now comparably.
*A matching boolean from two different arguments is `[] == []` in a passing test's
clothes* — slice 7's lesson, in a new place.

### ⚠⚠ The scope decision: C4 SPLITS BY MANIFEST, biosphere 13 + station 2

The 15 markers are **not one contract's**. `crew_mission` and `sealed_station` are
**station**-manifest keys, and their referents do not exist in the reference:

| gate | needs | in Rust today |
|---|---|---|
| `test_rq_structural_prediction` (`crew_mission`) | a crew steady-state run + the BVAD comparison constants (`0.8814` appears in **neither** tree — it is a test-side expectation) | the crew domain exists; the RQ helper does not |
| `test_tier1_node_is_period_1_fixed_point` (`sealed_station`) | `predicted_equilibrium_temperature(charge, thermal, HEAT_CLOSURE_SCENARIO)` | `HEAT_CLOSURE_SCENARIO` **yes**; the predictor **no** |

Slices 6–8 re-anchored **one manifest per slice** on purpose, and a re-anchoring
criterion *names a direction per axis*. So C4 takes the **13 biosphere gates** and
**C4b** takes the two station ones with the two helpers they need. Recorded as a split,
not as a deferral: both halves are scheduled, and the station half has its own ceremony.

### ⚠⚠ AND C4 HAS A C6 DEPENDENCY THE PLAN TABLE DOES NOT SHOW — the second time this has happened

`test_the_vks_mutual_shading_regime_is_MODELLED_not_merely_avoided` asserts over a roster
of **six** scenarios, and two of them — `n_limited` and `water_biting` — are precisely the
Python-only scenarios **C6 retires**. ⚠ **ANSWERED: C6 landed first, 2026-08-18 —
see §5k.** The roster is now exactly the four the reference carries, nothing is truncated,
and the gate's pinned numbers did not move. `grep` over `rust/crates` finds them only inside
comments *about* C6. So the gate cannot port as written: either it ships over four
scenarios (a silent truncation of its own claim) or **C6 lands first**, after which four
*is* the roster and nothing is truncated.

*Generalize, again: a slice's prerequisites are not in its own row of the plan table.* C5
found the same thing about C4 six hours earlier — from the other side.

### What this leaves for the build

* The bound literals: `61.07` is **derived** in Python (`Γ*/ci_ratio`) with the literal
  kept only as a tripwire, and the same two params are already in Rust — so the Rust gate
  derives it the same way. `14.4248` is a `senescence.yaml` value, which C1 put in Rust.
  `0.8814` is test-side and belongs to C4b. The rest are liveness floors, tuned to our own
  calibration, which the manifest already records as a different **class** of claim.
* The census mechanism is the open design question and the one slice 8 warns about: Rust
  has no introspection, so a table plus tests is a hand-maintained roster unless the table
  **is** the test roster (one `#[test]` emitted per row, so an unexercised entry is a
  compile error rather than something a meta-test hunts textually).
* **NOTHING IS BUILT.** `git diff` is empty for `src/`, `rust/`, the goldens and all three
  manifests; the probe lives in `M:/claud_projects/temp/c4-science-gates/`.


## §5k C6 — the four Python-only scenarios retire, COMPLETE 2026-08-18

The user's decision of 2026-08-17 executed: `n_limited`, `water_biting`, `demo_euler`
and `demo_rk4` **deleted, not ported**, each with a written reason. Taken deliberately
BEFORE C4's build, because C4's mutual-shading gate asserts over a six-scenario roster
two of whose members were this slice's subject (§5j's unlisted dependency).

### The gating check, done before predicting anything else

All four names occur **zero times** in all three `.manifest.json` files. So C6 is **not
an unfreeze of any contract** — it is a deletion of goldens no contract requires and of
tests no manifest names. That single grep is what made the rest of the slice ordinary.

Roster effect: **25 goldens → 21**, and the reference's share **19/25 → 19/21**, with no
value moving anywhere. `RUST_AUTHORED` is untouched at 19; the Python-authored remainder
falls from six to **two** (`drift_summary`, whose fold is the artifact — C5's measured
refusal — and `state_snapshot`, which Rust *reads*).

### ⚠ The successor landed FIRST, and the measurement that forced it

`nitrogen_stress_factor` had **zero test callers anywhere in `rust/`**. Every Rust
scenario holds `f_N ≡ 1`; `n_limited` was the one run in either tree that drove the
limiter below 1. Deleting it would have left a branch of the **reference** exercised by
nothing — the exact failure a "retirement with a written reason" exists to prevent, and
the one thing in this slice that could not have been fixed after the fact.

So the claims moved before the scenario died, in the shape Rust already uses for the
water side (`drought_acceleration_is_wired_into_the_accumulator_and_no_scenario_shows_it`
manufactures its condition rather than carrying `water_biting`):

* `science.rs::the_nitrogen_stress_ramp_is_linear_between_its_two_knots` — knots EXACT,
  monotone through the interior, plus `soil_n_below_the_residual_shuts_uptake_off_entirely`
  (the shutoff that made the scenario *pure dilution*).
* `system.rs::nitrogen_limitation_is_wired_into_assimilation_and_no_scenario_shows_it` —
  the wiring, on a constructed run copying the retired declaration: a real sustained
  bite, never N-dead, `rationed == 0` / no events, and a lower peak vegetative biomass
  than an otherwise-identical N-replete run.

**Two negative controls, each reddening a different load-bearing line and NOTHING else
in the Rust suite** — which is also the measurement behind the tests' own warnings:

| Control | Result |
|---|---|
| drop the `f_water * f_n` multiply in `flows.rs` | **1 of 45 red** — and on "stressed, never N-dead": unthrottled, the plant outgrows its fixed reserve and `f_N` reaches exactly 0 |
| flatten the ramp to `1.0` | **2 red** — the knot pin and "`f_N` never bit" |

⚠ **The linearity pin uses a band with representable knots, deliberately.** On the
frozen band (1/90, 1/45) the midpoint reads `0.49999999999999994` — the round-off of the
arithmetic *building the input*, not the function's. Pinning `== 0.5` there would have
frozen an accident. Recorded because the first draft did exactly that and went red.

### The price: two claims with no successor, and two narrowings

Retired explicitly rather than quietly, each with a tombstone comment at its old site:

1. **`test_stem_reserves`'s two-row CO₂-trough comparison.** `water_biting` was the only
   row taking the non-bit-identical branch — at `dt = 1` its trough preceded the single
   fill event, and the quarter-day step re-timed the two so it no longer did, which is
   what proved the bit-identity was a claim about *when* that scenario's trough happens
   rather than about the reserve being inert. Its bound (the reserve moves the trough by
   under 5 %) has no subject left. The exact half survives on `sealed_chamber`.
2. **The same file's reserve-vs-frozen check on the N-limited regime** (bite 0.1688 vs
   0.1730, biting 199 vs 198 days). Its second arm is a **candidate form from a decision
   already taken**, so nothing remains to compare the frozen tree against. ⚠ The
   reserve's effect ON the nitrogen bite is the part with **no successor** — named here
   as a gap rather than dropped.

And two narrowings, stated where they happened:

* the shedding-fed litter C:N regime is now witnessed by **one** scenario instead of
  two. The surviving arm was **de-looped** — a one-element `for label in (...)` is one
  deletion away from asserting nothing, silently — and negative-controlled: inverting it
  fails at **105.93 vs 90.0**, a margin rather than a near miss.
* the fractionation table's four claims now rest on **three carbon-limited runs**, none
  of which exercises water limitation at all. `water_biting` was the row that moved
  furthest on every re-measurement, precisely because it had the most nonlinearity for a
  coarse step to truncate.

### What did NOT need a successor — measured before touching anything

| Claim | Where it already lives |
|---|---|
| both water stores are `depth × EXTR × ρ × A × MAI` | `test_every_scenarios_water_stores_are_geometric`, parametrized over an enumeration **from the module** — it auto-shrank with the deletion |
| the drought accelerator is wired in | the Rust manufactured-condition test, which already named `water_biting` as the Python field it copies |
| `f_N` bites somewhere | the new Rust pair above |

### C4's blocker cleared, and a mislabel with it

Peak LAI measured for all six roster members **before** the edit, because the gate's
`max(chambers) < 1.0` pin carries an inline `0.585` and a departing scenario owning that
number would have forced a re-measurement (§5j's own falsifier):

| scenario | peak LAI | |
|---|---|---|
| `open_season` | 6.0228 | survives |
| `consumer_chamber` | **0.5849** | survives — **this is the pinned number** |
| `sealed_chamber` | 0.5425 | survives |
| `perennial_chamber` | 0.4927 | survives |
| `water_biting` | 0.4718 | retired |
| `n_limited` | 0.0869 | retired |

Neither departing peak was ever binding, so **the gate's numbers do not move** and it now
reads over exactly the four scenarios the reference carries — C4 can port it without
truncating its own claim. It also drops a mislabel: the gate calls every label except
`open_season` a "chamber", and `n_limited` is an **open-field** run by its own
declaration.

### ⚠ Prose was corrected by TENSE, and this is the C3 finding a third time

Roughly a dozen sites carried **present-tense claims about what the tree CONTAINS** —
"`water_biting` is one of only two runs in the tree where water limits", "`n_limited` is
the one place `f_N` bites", "`water_biting` is a sealed chamber outside the manifest
entirely". Every one became false the moment the scenario went, and **no gate in either
tree compares such a sentence to the tree**. They were rewritten to past tense with the
retirement date, not deleted — the measurement they record is still true of the day it
was taken. A separate class was left alone: "the `n_limited`/`water_biting` precedent"
in the authoring and power tests names a **discipline**, which survives its exemplar;
only the dangling *file* pointers (`Mirrors test_regression_n_limited_season.py`) were
repaired.

### Scope held deliberately

* **`build_demo` and `params/demo.yaml` STAY.** C6 retired the two demo *goldens* and
  their regression gate, not the Phase-0/1 skeleton — `test_biosphere_demo.py` asserts
  engine-assembly properties, a different subject, and its fate is Stage 3's.
  ⚠ Consequence recorded at the exclusion site: `demo.yaml` is now frozen by **nothing**.
* **No science was taken in the same batch** (CLAUDE.md's rule). The Rust successor adds
  no mechanism, no value and no scenario; it re-homes an existing exercise of existing
  frozen science.

### Verification

`cargo test` + `cargo clippy --all-targets` green; `uv run pytest -n 12` **2439 passed,
5 skipped** — with no bound anywhere loosened, no threshold widened and no golden
regenerated. Commits `34430ed` (successor) and `01bf957` (retirement); the probes are in
`M:/claud_projects/temp/c6-scenario-retirement/`.

### ⚠ Addendum — the sweep that was scoped too narrowly, caught in review

Every post-deletion sweep in this slice ran over `tests/ src/ --include=*.py`. The two
**living freeze-contract docs** — `docs/biosphere-reference.md` and
`docs/station-reference.md` — were in the very first grep of the slice and in none of the
later ones, so both still described the retired scenarios and the deleted demo goldens in the
present tense, inside their "Not part of the reference (scoped out, by name)" sections.

⚠ **This is the sharpest form of the finding §5k already records.** The gating check that
opened the slice — "all four names occur zero times in all three manifests" — proved no
*machine-readable* half of any contract moved. But a contract here is **doc + manifest**, the
manifest gate equates manifest↔tree, and *the doc is not a side*. So the half that was
verified is precisely the half that has a gate. Fourth instance of the stale-prose finding,
and the first inside a document that calls itself a freeze contract.

Fixed by the same discriminator, applied deliberately:

* **Live scope statements** (both docs' scoped-out sections) → updated. They now record the
  retirement, that no manifest moved, that the limiters are still exercised in the reference,
  and that `demo.yaml` is now frozen by **nothing**.
* **Dated unfreeze-log entries** naming the retired scenarios → **left alone**, with one note
  added at the top of the log saying entries are dated records not maintained afterwards.
  Rewriting a measurement to match a later tree falsifies the measurement.
* `docs/log/*.md` and `docs/plans/post-roadmap-*.md` (≈20 files) → untouched, same reason.

**The rule this leaves:** after any deletion, sweep the **living contract docs** explicitly.
They are not in `src/`, not in `tests/`, and not covered by a language-scoped grep — which is
exactly why three separate sweeps missed them.


## §5l C4 — the 13 biosphere science gates move to the reference, COMPLETE 2026-08-18

§5j measured this slice and split it; this is the build of the biosphere half. The census
that produced about **half the biosphere manifest by content** is now the reference's, and
the diff it left is **13 `locus` strings and two `_authority` notes — no recorded claim, no
value, no golden, and the station manifest byte-identical.**

### The mechanism: the row IS the test

§5j named the open design question — "Rust has no introspection, so a table plus tests is a
hand-maintained roster unless the table **is** the test roster". That is what was built.
`rust/crates/domains/src/biosphere/science_gates.rs` declares each gate once, and a
`macro_rules!` emits **both** the `GATES` row and the `#[test]` that executes it. The
`locus` is built from the test's own identifier with `stringify!`, so it cannot drift from
the test it names.

⚠ **What this makes impossible is narrower than it first looks, and the distinction is
measured rather than argued.** The macro makes an *unexercised row* unrepresentable. It does
**nothing** about a *deleted claim*: removing a whole declaration leaves the Rust suite green
— the test simply ceases to exist — and it is the manifest comparison in
`tests/crossport/test_inventory_parity.py` that reddens, naming the missing scenario. Two
different failures, two different mechanisms, and only one of them is structural.

### The three things §5j left, and how each landed

| §5j's open item | Outcome |
|---|---|
| the census mechanism | the macro above; the table is 13 declarations in one file |
| `61.07` is derived, not typed | the gate derives `Γ*/ci_ratio` exactly as Python does, and the recorded literal is carried by a **tripwire** (`the_floor_is_where_the_frozen_params_put_it`) — which is also what puts `61.07` in the file for the locus check |
| the C6 dependency | cleared before the build (§5k); the mutual-shading gate reads the four scenarios the reference carries and its pinned `0.5849` did not move |

### Runtime, measured rather than assumed

§5j took its numbers in `--release` and said out loud that the permanent gates would run in
`cargo test`'s debug profile. Measured: the 13 gates add **~3.4 s** to `cargo test`, against
a 27 s baseline. The `domains` lib goes **45 → 65** tests: the 13 gates plus 7 the census
carries about itself (the field set, the claim fields, locus uniqueness, the bound-literal
check and its hand-transcribed scanner's own pinned negatives, the `61.07` tripwire, and the
`Γ*`-provenance robustness argument the frozen `source` strings cite). The six trajectories are shared
through `OnceLock` — Python has a `scope="module"` fixture and Rust has no fixtures, so
without it the two 15-year chambers would be re-run several times each. Every value matches
the release probe to 17 significant digits, so the profile is byte-neutral for this family
too.

### ⚠⚠ The pre-reduced series, and the hole it opens that Python does not have

The gates fold **per-step scalar series**, not `Vec<State>` — the station's own precedent
(`emit_sealed_energy_drift.rs` folds a temperature series rather than materializing 109,801
states, and `year_summaries` is generic precisely so it can).

The hole: `year_summaries` computes `n_years = (len - 1) / year`, so an observer emitting
`steps` samples instead of `steps + 1` yields **14** annual summaries instead of 15 — and
**every gate still passes**, because `non_collapsing` over 14 years passes exactly as well as
over 15. Python never needed a guard for it; the pre-reduction is what creates it. Closed
with an observer-count assert and a per-gate summary-count assert, and both were controlled:
dropping the initial state reddens all 13; dropping **one** sample from **one** series
reddens **exactly one** gate — the count assert — while the CO₂ band does not notice, because
its minimum is not at the end of the run.

Two more silent-pass paths were closed on the way, both of the same shape (*a fold over an
empty series returns the identity*): the open field has no `biosphere.carbon_pool` at all and
only the consumer chambers carry a herbivore, so `min_ppm` over an absent series would return
`+∞` — happily "above the compensation point".

### ⚠⚠ THE MOJIBAKE, AND WHY IT IS THIS ENTRY'S REAL FINDING

The first regeneration wrote **corrupted text into the frozen contract with every gate
green**. The reference emits UTF-8; `subprocess.run(text=True)` decoded the pipe with the
Windows locale's cp1252, so `—` was frozen as `â€"`, `⚠` as `âš `, `Γ` as `Î"`, `CO₂` as
`COâ‚‚`. Nothing was red **because the manifest and the checker agreed** — the corruption
happened on the way in, so the comparison was between two identically-mangled sides.

What caught it was **predicting the diff before regenerating** (13 loci, two notes, zero
value changes) and getting 37 changed lines instead.

* Every byte this dump had ever emitted was ASCII — names, hex floats, sha-256 digests — so
  the pipe's encoding had never mattered. **The first non-ASCII key is the one that finds
  it**, and frozen *claim text* is the first thing in this contract that is prose.
* Both readers now pin `encoding="utf-8"`. Losing it on **one** side is red in the parity
  gate (controlled). Losing it on **both** compares equal and is *not*, which is why
  `test_the_frozen_claim_text_survived_the_pipe` asserts the characters themselves — also
  controlled: with both sides unpinned, it is the only thing red.
* ⚠ **The class is wider than the two files fixed.** Roughly twenty other `text=True`
  subprocess pipes read Rust output in `tests/crossport/`. None is red today because none
  carries non-ASCII, so this is recorded as a **named condition, not a swept fix**: the next
  Rust program to emit prose through a pipe needs the encoding pinned at the same time.

### What else went red, and why that was correct

`test_acceptance_gate.py::test_the_plausibility_bands_are_now_named_by_a_manifest` asserted
that the manifest names `test_senescence_form` and `test_nitrogen_form` — the Python **files**
the loci pointed at when the science was granted contract standing. Re-pointed at the new
loci, **not relaxed**, which is exactly the treatment its own docstring records for the pin
*it* replaced. It also now checks the band's text in the reference (`assert!(5.0 < peak &&
peak < 8.0`) as well as in the checker's surviving copy.

### The Python side: markers removed, tests kept, and the census inverted

Thirteen `@pytest.mark.science_gate` decorators were deleted, each site carrying a comment
naming its Rust successor. **The test functions stay** — they are the checker's own copy of
the assertion, and deleting them is Stage 3's call, not a free consequence of C4.

`tests/science_gates.py` survives for the two station gates. The Python-side gates inverted
the way slice 6's did for `_flow_set()`:

* `test_frozen_science_gates_are_complete` → `test_the_frozen_science_gates_are_the_references`
  (every frozen entry's locus is under `rust/crates/domains/`, and there are 13);
* new `test_the_python_science_census_is_only_the_station_pair` — the split asserted rather
  than written in a doc; re-marking a biosphere test is red from both directions;
* `test_science_gate_bounds_name_a_literal_present_at_their_locus` keeps working **unchanged
  in its body** — the locus is a `.rs` path and the check reads whatever file it names. ⚠ Its
  `checked == len(collect_science_gates())` tail could not survive, and replacing it with a
  literal `15` would have turned a derived count into the hand-maintained roster the whole
  census exists to prevent. It now derives the count from the manifests and separately asserts
  that every gate the *Python* census still finds is in one.

The same crude check now runs on the reference's side too
(`the_bound_literals_appear_at_their_locus`), with the regex hand-transcribed (no crate may
be added to a zero-dep tree) and its own pinned negatives — `5%/day` contributes no literal,
so only `6.0` has to be present for the mutual-shading bound.

### The residue, named rather than left

* **Five frozen `source` strings still spell a Python test name**
  (`test_the_shipped_floor_is_the_conservative_one_against_the_cited_route`). The companion
  assertion was ported under the same name minus the `test_` prefix; the strings were **not**
  edited, because editing them is a value change to the contract rather than the locus
  re-anchoring this slice is. For whichever slice retires the Python file.
* **C4b** — the two station gates and the two helpers they need
  (the RQ comparison; `predicted_equilibrium_temperature`). Both contract docs now say so.
  ⚠ **DONE 2026-08-18, §5o**, and one helper needed nothing:
  `predicted_equilibrium_temperature` already existed in the reference. The line above is
  left standing because §5o's first finding is that it was wrong.

### Verification

`cargo test` + `cargo clippy --all-targets` green; `uv run pytest -n 12` **2441 passed, 5
skipped**. No bound loosened, no threshold widened, no golden regenerated, no parameter
moved. Probes: `M:/claud_projects/temp/c4-build/`.

---

## §5m C7 — the manifest writer moves to the reference; the biosphere half COMPLETE 2026-08-18

The last Stage-2 slice, and the one whose absence made the flip's headline literally
false: the three `docs/*-reference.manifest.json` files were *authored* by the reference
key by key (slices 6, C4, C8, C9) and **written** by the checker.
`tests/test_freeze_manifest.py::_build_manifest()` shelled the reference's dump, spliced
its keys into its own, and serialized the result. A contract whose first line says Rust is
the reference had a Python program in the middle of it.

This section covers the **biosphere** half. The order — biosphere → authoring → C4b →
station — was set by the advisor and by a measured blocker; see "the order, and why the
station cannot go first" below.

### ⚠⚠ The design question that looked like a blocker and was not: does the writer carry authority?

The obvious objection to moving the writer is that it would make every key Rust-authored
by construction, destroying the `_authority` block's honesty — the very thing slices 6–C9
built. **The tree had already answered it, and the answer was sitting in the block being
worried about.**

`scenarios/*/golden_sha256` has been marked **`rust`** since slice 4 while *Python*
computed the digest, on the stated ground that the golden is the reference's own output.
So `_authority` records **who produced the value**, not who ran the digest, and by
extension not who wrote the file. C8 says the same thing in different words: *"the values
are author-neutral either way (both sides digest the same file); what moved is the CENSUS
rule and the NORMALIZATION rule."*

The move is therefore authority-neutral by construction — no schema change and no honesty
problem. The mirror image now holds: this program hashes `drift_summary.json` without
becoming its author, exactly as Python hashed six Rust-authored goldens without becoming
theirs.

### The gating measurement, taken before a line of the writer was designed

Same discipline as C1/C8/C9/C5 — enumerate the hazards against the artifact, then build.
The question was whether a Rust writer can reproduce
`json.dumps(obj, indent=2, sort_keys=True) + "\n"` byte for byte, so that the move is a
re-anchoring and not an unfreeze. Measured across **all three** manifests:

| Hazard | Measured | Consequence |
|---|---|---|
| non-BMP characters (surrogate-pair escaping) | **none** — not one character above `U+FFFF` | the writer *panics* on one rather than carrying logic no test could exercise |
| `ensure_ascii` escapes | 232, all lowercase `\uXXXX` | must be implemented; the existing dump's escaper deliberately does **not** do this |
| numbers that are not integers | **exactly one** — `dt_days: 0.25`, itself a hand literal | float-formatting risk is *zero*; `repr` vs `{}` never arises |
| empty containers under `indent=2` | 39 (25 of them station) | `"key": []`, never an indented pair of brackets |
| `null` | 15, all authoring | the writer needs it |
| key order | `sort_keys` is code-point order; Rust `&str` is UTF-8 byte order | they agree — the argument `ScienceGate::Ord` already makes |

**Result: byte-identical on the first run.** md5 `49fbe2b4…` on both sides, `cmp` clean.
The writer move moved no byte of the biosphere contract.

### ⚠⚠ THE FINDING: the trap this slice sets is invisible to the gate that guards it

`dt_days` and `integrator` are frozen **by hand** on purpose — a manifest that read
`BIO_DT` would auto-follow a step change, which is the opposite of a freeze, and the
2026-08-14 step move became a ceremony only because that literal went red. C7 moves the
writer **into the crate that owns `BIO_DT`**, where splicing it in is a one-character
edit. The advisor flagged the hazard; the question is what would catch it.

**Measured, not argued (control D): replacing `Json::num("0.25")` with
`Json::num(format!("{BIO_DT}"))` produces a byte-identical manifest.** So:

* C7's own gate — regenerate and compare — is **blind** to it;
* so is the cross-port check that compares the frozen literal against `BIO_DT`: it
  compares equal either way. What that check protects is the *ceremony*, and the ceremony
  exists only while the literal is typed.

That is the step unfreeze's own lesson recurring in a new place: *no test at `dt = 1` can
tell a correct conversion from a wrong one, because the two are the same integer.* Today
the literal and the constant are the same number; the day someone splices the constant is
the day the freeze quietly stops being one, with nothing red.

Two guards, because neither alone is enough:

1. **Structural** — `config::canonical_json::Json::Number` is constructed only from
   **text** (`Json::num` takes no `f64`). Splicing the constant is not a silent type
   coercion but a visible `format!`.
2. **Textual** — `rust/crates/domains/tests/manifest_writer.rs` reads the writer's own
   source and asserts the emission site is `Json::num("0.25")` and does not mention
   `BIO_DT`. Crude on purpose, with precedent in this tree
   (`the_bound_literals_appear_at_their_locus` greps a file for a recorded bound). ⚠ Its
   own control earned its keep on the first run: the original anchor was the bare key
   `"dt_days"`, which matches **two** lines — the emission site and the `_authority` row
   that classifies it — so `find` would have read whichever came first.

### The deliberate diff, predicted and then measured: 5 lines, no value

Two changes were *not* free and were taken as a visible, stated diff rather than smuggled
into the re-anchoring:

| Key | Before | After | Why |
|---|---|---|---|
| `forcing/weather_fixture` | `python` | **`hand`** | C9's finding stands — the reference `include_str!`s the fixture, so it knows the BYTES and not the NAME. While the checker wrote the file, `python` fairly described who typed the name; once it does not touch the file, `python` names a producer that is not there. It is the `integrator` category: a name with no importable referent on **either** side |
| `forcing/weather_sha256` | `python` | **`rust`** | the reference has emitted this hash since C9 (of the text it compiled in) for *checking*, and `test_the_weather_hash_matches_the_reference_tree` has held the two sides equal ever since — so this re-anchors between two sides a gate already pins. ⚠ It is free *now* precisely because C9 declined to do it: C9 would have had to split the pair while the name had no Rust referent. C7 resolves the split the other way |
| `_comment` | names the Python regeneration command | names the cargo one | it was about to become false |

`git diff --stat` on the manifest: **5 insertions, 5 deletions**, all inside `_authority`
and `_comment`. No hash, no set, no claim, no bound moved.

### The controls, all four

| # | Perturbation | Result |
|---|---|---|
| A | stop escaping non-ASCII in the writer | **differs** — reddens |
| B | indent empty arrays instead of emitting `[]` | **differs** — reddens |
| C | restore both | identical again |
| D | splice `BIO_DT` for the frozen literal | ⚠ **identical** — see the finding above |
| E | hand-edit one byte of the committed manifest | `test_manifest_writer.py` **red** |
| F | drift the checker's copy of a scenario label | `test_the_frozen_roster_is_the_references` **red** |

### ⚠ The hole C7 opened, found by asking the authoring slice's question

*Does deleting Python's `_build_manifest` redden anything?* Measured **before** deleting:
**no.** It is called only from `_regenerate`, which is called only from `__main__`, which
no test invokes. Grepped across the whole tree. That is
[[authoring-manifest-reanchored]]'s lesson in its usual shape — a control with no test to
turn red IS the finding — and it is recorded rather than quietly fixed.

But the deletion did more than remove dead code, and this is the part that needed a new
gate. The **scenario roster** (`name -> label, horizon, golden`) lived in the Python module
*and was written from it*, so no gate was needed: the manifest could not disagree with its
own source. Moving the writer turned that single source into **two copies with nothing
holding them together** — and the two fields at risk are exactly the ones `_authority`
marks `hand` (the human label, the golden's filename), which no gate can re-derive if they
drift. Names were already compared and run lengths were already compared; the labels and
filenames were not.

`test_the_frozen_roster_is_the_references` closes it, and control F confirms it reddens.

### What the new gate catches that nothing else did

`tests/crossport/test_manifest_writer.py` regenerates into a temp file and compares
**bytes**. `test_inventory_parity.py` compares derived sets axis by axis and says nothing
about the hand-authored half, the serialization, or the keys the checker still authors.
So three failures are newly visible:

* a frozen surface that moved without regeneration — for **every** key, not just the
  compared axes;
* **a hand edit to the committed manifest**, which was invisible to every gate before C7:
  a typo in `_comment` or a hand-patched hash simply stood;
* a change to the writer's own serialization.

⚠ Its own control (`test_the_writer_refuses_an_unknown_argument`) exists because a writer
that ignored the `--write-manifest` flag and wrote its default location would make the
byte comparison pass while proving the wrong thing.

### And the pipe is gone

The file is written with `std::fs::write`, not printed for the checker to capture. C4's
first regeneration froze cp1252-mangled prose into this contract **with every gate green**,
because a `subprocess` pipe decoded UTF-8 with the Windows locale and *both* sides were
mangled identically. C7 deletes the class rather than inheriting it: nothing decodes the
manifest, and the bytes are pure ASCII besides. ⚠ The *dump* path is deliberately unchanged
— it still emits raw UTF-8 through a pipe, because `test_inventory_parity.py` reads it and
the encoding pin it grew after C4 is a control that only has teeth while there is non-ASCII
to mangle.

### The order, and why the station cannot go first

The station manifest's `science_bands` + `liveness_floors` are two real claims (the
respiratory-quotient comparison and `non_collapsing(floor=100.0)` on the thermal node)
whose referents the reference does not carry. A Rust writer cannot derive them and must not
hand-carry them — a hand-typed claim is exactly the census this contract exists to prevent.
So **C4b is a prerequisite of C7's station half, not a follow-up**, the same shape as the
C5-before-C4 dependency §5h found by measuring. Remaining order: **authoring → C4b →
station**.

⚠ C4b's cost is not yet measured. `non_collapsing` already exists in `drift.rs`;
`predicted_equilibrium_temperature` is Python-only in `src/station/system.py`. If C4b turns
out to be more than those two claims plus two helpers, that is a scope question for the
user rather than something to absorb.

> ⚠⚠ **The second sentence was FALSE when it was written, and §5o records it as the C3
> finding recurring.** `predicted_equilibrium_temperature` was at
> `rust/crates/station/src/system.rs:44`, with `mean_dissipated_power` and
> `equilibrium_node_heat` beside it, and the 15-yr energy run with its per-year peak fold
> was already in `emit_sealed_energy_drift.rs`. C4b came in **under** the estimate and the
> escalation condition never fired. Left in place with this correction rather than edited
> away — the point is that nothing re-reads a present-tense claim about the tree.

### Deliberately NOT in C7's biosphere half

* **The station and authoring manifests.** Their writers are their own slices; the
  `_WRITERS` table in `tests/crossport/test_manifest_writer.py` has one row, and a contract
  absent from it is a gap rather than a policy — stated there so the single row cannot read
  as coverage.
* **Deleting the Python checker.** The ~22 gates in `tests/test_freeze_manifest.py` stay:
  they are the checker's own copy of the completeness assertions, and retiring them is
  Stage 3's call, not a free consequence of moving a writer.
* **The `red.Slice 6` typo** in the frozen `dt_days` rationale — a missing space, an
  artifact of the Python string concatenation the prose was assembled from. It survives
  into the Rust table byte for byte. Fixing it is a **value change to the contract**, not a
  re-anchoring, and it is not worth spending an unfreeze on by itself; the next deliberate
  edit to that row should take it.
* **`regen_goldens_from_rust.py`'s claim** that a `--write` moving a frozen golden *"turns
  nothing red"*. That is now false for the biosphere — the writer hashes the goldens from
  disk, so the regenerated manifest differs and `test_manifest_writer.py` is red. Corrected
  there rather than left standing.

## §5n C7's authoring half — the platform contract's writer moves, COMPLETE 2026-08-18

The second of C7's three halves, in the order §5m set (biosphere → **authoring** → C4b →
station). Same shape as the biosphere half and a smaller contract: nine derived keys, five
hand ones, and two provenance hashes.

### The gating measurement, and it came back free

`canonical_json` was measured against **all three** manifests when it was written for the
biosphere half, so the serialization question was already answered: this file carries no
character above the basic plane, no float, and its 232-escape budget is shared with the
other two. What was not known is whether the writer reproduces *this* file, and it does —
**every derived key regenerated byte-identical on the first run**. The two lines that moved
are prose, and both were predicted before running the writer.

### The deliberate diff: two lines, and one of them is a finding

`_comment` moved because it named the regeneration command (`uv run python
tests/test_authoring_freeze_manifest.py`), which C7 makes false.

**`parity_vectors/*`'s `why` moved because it argued against C7.** It read *"A Rust-side
hash would compare the checker's own output with itself"* — and the new writer does exactly
that hashing. This is the same objection §5m dissolved for the biosphere half, sitting
frozen inside a contract rather than raised in a design discussion: it conflates **who
produces a value** with **who computes its digest**. The precedent is one file over —
`scenarios/*/golden_sha256` has read `rust` since slice 4 while *Python* hashed it, because
the golden is the reference's output. The mirror holds here: the vector files are generated
by `tests/crossport/gen_authoring_vectors.py` (checked, not assumed — still live, still
gated by `test_crossport.py`'s in-sync guards), so the value is the checker's and the
`side` does not move. Only the reasoning was wrong, and it was corrected in place rather
than left to argue against its own writer.

⚠ Worth separating from C9's `forcing/weather_fixture`, which went `python` → **`hand`** and
looks like the same situation. It is not: that row moved because *no Python touched the file
any more*, so `python` named a producer that was not there. Here the producer is there.

### ⚠⚠ Deleting the writer opened the same hole it opened on the biosphere, in a different key

The **roster** — which files `parity_vectors` records — lived in the Python module *and was
written from it*, so it needed no gate: the manifest could not disagree with its own source.
Moving the writer turns one source into two copies with nothing holding them together, and
the copy at risk is the one `_authority` marks `python`. A file dropped from the reference's
`VECTOR_FILES` simply stops being hashed, and every other gate stays green.

`test_the_frozen_vector_roster_is_the_generators` closes it, and the tie is to
`gen_authoring_vectors`' own output paths rather than to a list retyped in the checker —
the generator *produces* these files, so it is the roster's single source of truth, the way
`golden_platform.RUST_AUTHORED` is for the goldens. A control confirms it reddens: dropping
a file from the writer and regenerating so the manifest agrees with the writer leaves this
gate as the only thing that notices.

⚠ **It also adds a value check that never existed.** The two hashes were provenance that
nothing recomputed — the contract's own prose said so ("not assertions"). The gate now
recomputes both under Python's `splitlines` rule against the reference's narrower
`config::provenance` one, which is C8's two-rules-held-equal tie arriving on this contract
through C7 instead.

### ⚠ The dump's serialization changed, and the encoding control survives it

The dump now goes through the same canonical writer as the manifest, so it is ASCII-escaped
where it used to emit raw UTF-8. Two things were checked rather than assumed. Its consumers
parse it (`json.loads`), so formatting is invisible to them — the one place that could have
cared was a text comparison, and there is none. And the `encoding="utf-8"` pin on the shared
`_rust_inventory` helper — C4's mojibake control — keeps its teeth, because the *biosphere*
dump still carries non-ASCII through that pipe. The authoring dump never did (measured: 3582
bytes, none above 0x7f), so it was not exercising the pin before and takes nothing with it.

### The trap: measured for, and absent

The biosphere half's finding was that moving the writer into the crate owning `BIO_DT` put
the frozen `dt_days` literal one character from auto-following the code — invisibly, since
the spliced constant produces a byte-identical manifest. The same question was asked here
and the answer is **none**: this contract's hand keys are a phase number, two repo paths and
two blocks of prose, and the `authoring` crate owns no constant any of them could be spliced
from. The keys that *should* follow the code (`step_token`, the integrator and rate-class
vocabularies) are already classified `rust`.

Recorded as a measurement rather than answered with a guard invented to match the other
half's. A control with no test to redden is the finding; a guard with no trap to catch is
worse — it reads as coverage.

### What left the checker, and the one gate that had to be re-pointed

Deleted with the writer: the `_AUTHORITY` literal, the frozen prose, `_build_manifest`,
`_manifest_dumps`, `_rust_reference`, `_RUST_DUMP_KEYS`, `VECTOR_FILES`, `STATION_MANIFEST`
and the `__main__`. Three gates changed subject rather than dying:

* `test_every_frozen_field_declares_who_produced_it` reads `_authority` **out of the
  committed file** now, and its fourth assertion (`manifest["_authority"] == _AUTHORITY`)
  became **shape** checks — a row is `{side, why}`, `side` is one of three, `why` is prose.
  Same substitution the biosphere half made, and it catches a malformed row the equality
  never caught either.
* `test_manifest_delegates_param_values_to_the_station` compared `delegates_to` against a
  module-level path literal. Once the reference writes the pointer that is a Python literal
  checked against a Rust literal. It now asserts the property the pointer exists **for**:
  the target is a real manifest and carries the `param_files` census this contract declines
  to re-hash.
* `test_the_reference_side_keys_are_exactly_what_the_generator_splices` had **no subject
  left** — it tied `_RUST_DUMP_KEYS` to `_AUTHORITY`, and neither exists here now. Retired,
  with its replacement named in the test that absorbed it:
  `test_the_python_derivations_are_conformance_checks_now` re-anchored on the committed
  `_authority` block's `rust` keys, which is the same claim stated against the file instead
  of against a copy of it.

### ⚠ A recorded reason that was too broad, and the authoring case is what exposed it

`test_the_dump_key_sets_are_the_ones_the_generators_consume` lost its authoring row, as its
own docstring scheduled. But the **reason** written there when the biosphere row left —
*"nothing consumes the dump"* — is false for this one: `dump_authoring_inventory` is still
consumed, by `test_rust_inventory_equals_the_frozen_manifest` in that very module. What
actually licenses the removal is narrower: there is no longer a *second copy* of the key
set to hold equal, because the copy lived in the writer. Same outcome, different reason,
and corrected there — a docstring carrying a reason that is not the reason is the defect
class this repo keeps recording.

### The control that was hardcoded to one row

`test_the_writer_refuses_an_unknown_argument` names the biosphere example by hand. It was
written when `_WRITERS` had one row, and by its own stated reasoning — a writer that ignores
its flag makes the byte comparison pass while proving the wrong thing — leaving it
unparametrized would have shipped the authoring writer's argument handling unasserted.
Parametrized over `_WRITERS`.

### The prose half, which nothing gates

`docs/authoring-reference.md` named the old regeneration command in three places, including
step 4 of its own unfreeze ceremony. Nothing compares that file to the manifest
(`freeze-prose-half-is-ungated`), so it goes stale silently; corrected in the same commit,
with an unfreeze-log entry.

### Verification

`cargo test` (all green) + `cargo clippy --all-targets -D warnings` clean; `ruff`, `pyright`
and the Python suite green. ⚠ `rustfmt` was run on the **example file only** — never a
module root, which reformats the whole subtree.

Controls: hand-edited manifest → red; a drifted vector file → red; a file dropped from the
roster → red at the byte gate, and red at the roster gate too when the manifest is
regenerated to agree with the writer; a wiring field renamed in **Rust** → manifest moves
(byte gate red), checker green; the same field renamed in **Python** → checker red, manifest
byte-identical. ⚠ The first attempt at the wiring-field control mutated a string that does
not exist in the registry and came back green — an inert control, caught by checking the
mutation rather than trusting the verdict.

### Deliberately NOT in this half

* **The station manifest's writer.** C4b first — its two science claims have no Rust
  referent, and a writer that cannot derive them must not hand-carry them.
  ⚠ **DONE 2026-08-18, §5p.**
  ⚠ **DONE 2026-08-18 (§5o), and the referent claim was false** —
  `predicted_equilibrium_temperature` and the folds already existed. The station half is
  unblocked.
* **Deleting the Python checker.** The completeness and conformance gates stay; retiring
  them is Stage 3's call.
* **`gen_authoring_vectors.py`.** It is scaffolding under the target state, and moving it
  is what would move `parity_vectors` off `python`. A successor item, not a rider.

### ⚠ Addendum — five things the gates could not see, caught in review

Taken in a follow-up commit; none moves a manifest byte.

1. **A one-element `for` loop survived the row removal** in
   `test_the_dump_key_sets_are_the_ones_the_generators_consume` — and here the deletion
   that empties it is *scheduled*, since the station row leaves when its writer lands.
   De-looped. C6 recorded this exact hazard on a surviving single arm three commits
   earlier; the lesson was in the index row this slice edited.
2. **The prose half said the two vector hashes are "not assertions"**, which this slice
   made false by asserting them. Corrected — and it is the C3 finding a fourth time: a
   present-tense claim about the tree went false and nothing compares that file to the
   tree. ⚠ Worse than the usual instance, because the sentence was *quoted in this very
   plan* as the evidence that the value check is new, and still not updated.
3. **The value half of the new roster gate was claimed, not measured.** The drifted-vector
   control ran against the byte gate, which reddens on any file change regardless — so
   the hash assertion itself had never been shown to bite. Re-run against the roster gate:
   red on its own line, baseline and restore both green.
4. **⚠⚠ The delegation tie was first written against a naming convention this repo never
   adopted**, and the convention manufactured a failure. `<loader>.yaml` looked right for
   four of the five loaders and is wrong for `thermal`, which loads `radiator.yaml` — so
   the gate reported an authored scenario reaching unfrozen values, and the tree was fine.
   Rewritten to ask the **loader** for its default path instead of guessing from its name.
   A convention invented at the gate is not a property of the tree, and its first red is
   indistinguishable from a real finding.
5. **`sys.path.insert(0, tests/crossport)` had no cleanup.** That directory holds
   generically-named modules (`compare`, `authoring_files`), so leaving it at position 0
   for the worker's lifetime lets a later plain import resolve there. Now
   `monkeypatch.syspath_prepend`, which unwinds.

Also pinned: `test_manifest_records_the_grammar_is_incomplete` asserts on prose the
reference now writes, so its meaning inverted the way slice 8 inverted the derivation
gates. Stated in the test rather than left implied.

## §5o C4b — the station's two science claims move to the reference, COMPLETE 2026-08-18

C7's station half had a **measured prerequisite**: the station manifest's `science_bands` +
`liveness_floors` are two real claims whose referents §5m said the reference did not carry,
and a writer that cannot derive a claim must not hand-carry it. This is that prerequisite,
landed as its own commit for a reason that is not just schedule — see "why the order is
load-bearing" below.

### ⚠ The correction: the prerequisite's own cost estimate was false when it was written

§5m wrote *"`non_collapsing` already exists in `drift.rs`; `predicted_equilibrium_temperature`
is Python-only in `src/station/system.py`"*, and §5l's residue line said the same. Measured
before designing anything: **`predicted_equilibrium_temperature` is at
`rust/crates/station/src/system.rs:44`**, `mean_dissipated_power` and `equilibrium_node_heat`
beside it, the four drift folds in `domains::biosphere::drift`, and the whole 15-yr energy
run *with its per-year peak fold* already written in
`crates/station/examples/emit_sealed_energy_drift.rs`. So C4b came in **under** its estimate
and §5m's escalation condition ("if C4b turns out to be more than those two claims plus two
helpers, that is a scope question for the user") never fired.

⚠ This is the C3 finding again, in the plan rather than in `CLAUDE.md`: **a present-tense
claim about the tree went false and nothing re-reads it.** The claim was load-bearing — it is
why C4 split at all — and it was still being repeated in two contract docs. Corrected in all
four places rather than only here.

### Why the order is load-bearing, not just the schedule

C4b moves two `locus` strings, which is a **value** diff to the station contract. C7's
station half is gated by *regenerate and compare bytes*, and both prior halves got their
result from **byte-identical on the first run**. Bundling them would leave that comparison
unable to distinguish "the writer reproduces the contract" from "the writer produced a file I
just changed".

So C4b regenerates through the **existing Python writer**, with a four-line splice that reads
the two census keys out of the reference's dump the way `param_files` already is. Those four
lines are deleted one commit later. Hand-editing the manifest instead is exactly what C7's
biosphere half added a gate to catch, and the station gate does not exist yet — it would have
stood silently.

### The two claims, and what each needed

| Claim | Needed | Status before C4b |
|---|---|---|
| `crew_mission` / `science_bands` — the BVAD respiratory quotient | the BVAD constants, the validation cabin scenario, a steady-state boundary flux | the run helpers existed; the constants and scenario are **new Rust** (test-side) |
| `sealed_station` / `liveness_floors` — the thermal node's non-collapse floor | `year_summaries` + `same_phase_diffs` + `is_stationary` + `non_collapsing`, `predicted_equilibrium_temperature`, the 15-yr energy run | **all of it already existed** |

They live in `rust/crates/station/src/science_gates.rs`, a **second census table** invoking
the same `science_gates!` macro. The macro is `#[macro_export]`ed and takes a `source_file:`
header, because `concat!` needs a literal for the `locus` path. Copying the macro was
rejected on the obvious ground: two copies of the census *mechanism* is the failure mode the
"one declaration, not a roster" design exists to prevent.

⚠ The **shared** pieces are shared as ordinary `pub fn`s, not `cfg(test)` ones — a
`cfg(test)` item in `domains` is invisible to `station`'s tests, so the choice was between
exporting them and transcribing the numeric-literal regex twice.

### ⚠⚠ Finding 1: the first regeneration silently dropped eleven keys

Predicted diff: two `locus` strings plus the two `_authority` rows. Actual: that, **plus 22
deleted lines** — the eleven `"scenario": []` entries in each census field.

The cause is exact and it is a difference between the *dump* and the *writer*. The
reference's dump emits only scenarios that carry a claim, deliberately (which scenarios get a
key is the manifest's hand-authored roster; a program inventing keys would claim authority
over a set it cannot see). The biosphere **writer**'s `census_json` fills the roster; the
**dump**'s `census` does not. The Python census being replaced filled it. So splicing the
dump's shape into the writer's slot deleted the roster.

On this contract that is not cosmetic: `_authority` calls the emptiness *"itself the frozen
claim"*, because 11 of 13 station scenarios carrying no outside-sourced bound is the measured
result the freeze pins. `[]` says *measured, none*; an absent key says nothing.
`_filed_under_the_roster` fills the roster around the reference's claims and **raises** on a
claim naming a scenario outside it — the same panic the biosphere writer has, for the same
reason (a filter that drops a mis-filed claim looks exactly like a clean result).

⚠ This validates the ordering decision from the other direction: bundled with the writer,
those 22 dropped lines would have arrived *inside* a "byte-identical" claim.

### ⚠⚠ Finding 2: the bound-literal check could not fail, and had never been able to

The control was "delete the assertion that carries the recorded number and confirm the
locus check reddens". It took **three** attempts to bite, and each failure was a separate
lesson:

1. `0.8814` → `0.88140`. Green: the check is `contains`, and `0.88140` contains `0.8814`.
   An inert control caught by checking the mutation, not by trusting the verdict.
2. `0.8814` → `0.8815`. Green: the `bound:` record — `"CO2/O2 == approx(0.8814)"` — is a
   literal **in the same file**, so it supplies its own number.
3. Subtract the declared bounds' own occurrences (`in_source > from_records`). Green on the
   biosphere for six literals: the scanner's own pin test quotes six real frozen bounds as
   test data.

So the rule as C4 ported it — *"every numeric literal in `bound` appears textually in the
file its `locus` names"* — is **true by construction**, and the `science_gates!` design is
what guarantees it. Worse, the Python original had the same defect and predates the flip: the
`bound=` marker keyword sat in the file its `locus` named.

The fix is `code_only`: the source with comments and string literals stripped, so the number
must appear in **executable** text. Measured after — all 16 frozen literal instances appear
in code, eleven exactly once, and the control now reddens on both sides. The scanner has its
own pinned tests, including a `should_panic` on raw strings and the negative that its own
first draft failed: `src.contains("r\"")` fires on ordinary prose ending a word in `r`
before a quote (`not a roster" design`), so the guard rejected the tree it was written to
protect. The language assumption is stated too — `check_bound_literals` asserts
`file == source_file` first, which is what keeps a non-Rust locus from being silently
mis-stripped.

⚠ The **checker's** copy was retired rather than fixed, on a narrow reason: the rule needs
the locus file's syntax, and after C4b every locus is a `.rs` file, so the checker would need
a second Rust lexer written in Python. The broad reason ("the census is Rust's now") is the
one that has already been recorded as too broad twice in this flip. Three replacements are
named in place.

⚠ A consequence for annotation style: quoting a bound's value in a comment beside the
assertion used to satisfy the check and now does not. One such comment was removed from the
station table for exactly that reason, with a note saying why.

### ⚠ Finding 3: a control that only becomes necessary when the data changes

`_rust_reference` shells the dump with `subprocess.run(text=True)` and **no `encoding=`** —
the Windows locale, i.e. cp1252. That is the exact mechanism that froze mojibake into the
biosphere contract in slice C4 with every gate green (both sides mangled identically). The
pin was added to the crossport reader then and *not* here, and that was **correct**: nothing
the station dump emitted was above ASCII. C4b is the first slice to put an em dash through
this pipe (`self — the node must not collapse toward T_space`). Pinned, and recorded rather
than fixed quietly, because a control whose necessity is created by a data change is
invisible until the day it is needed.

### Two process notes, both of which nearly reached the contract

* **A mechanical text repair corrupted four prose strings and `ruff` passed on all four.**
  Re-wrapping over-long lines split string literals; the repair that re-joined them dropped
  the trailing spaces, producing `manifest'ssingle`, `sentencethe`, `readthe`, `split,the` —
  inside `_AUTHORITY`, which *writes* a frozen artifact. Caught only by reading the
  regenerated diff. A mechanical edit to a file that writes a frozen artifact needs the diff
  read; the linter is not a substitute.
* **`git checkout <file>` to revert a control discarded uncommitted work.** Reverting a
  mutation in `tests/test_bvad_validation.py` that way threw away C4b's own edit to it, and
  the only reason it was visible was a `grep -c` afterwards. The safe shape is the
  `cp … .bak` / `cp .bak …` pattern used for the Rust files.

### What left the checker, and what changed subject

* the two `@pytest.mark.science_gate` decorators — **the bodies stay**, as the checker's
  conformance half; retiring them is Stage 3's call;
* `test_the_python_science_census_is_only_the_station_pair` →
  `test_the_python_science_census_is_exhausted`. ⚠ Its claim **inverted**: the census that
  was the derivation is now a forcing function, and `tests/science_gates.py` stays live for
  that reason. It is not vacuous-on-empty, because it asserts the census *is* empty rather
  than walking it;
* the second half of the bound-literal gate — `for gate in collect_science_gates()` — became
  a **zero-iteration loop** and was de-looped with its replacement named. C7's authoring
  addendum recorded the same hazard on a one-element `for`, on this same file;
* `test_the_frozen_science_gates_are_the_references` widened to both manifests, prefix
  `rust/crates/`, count 15. `_REFERENCE_GATE_COUNT` now has exactly one consumer;
* `test_frozen_station_science_gates_are_complete` got C4's substitution — it compared the
  manifest against `gates_for(_ROSTER, field)`, which after C4b is thirteen empty lists on
  both sides. Its two "the claim went missing" tripwires survive unchanged;
* `gates_for` left the station module's imports with its last caller;
* `tests/crossport/test_inventory_parity.py`: `_STATION_DUMP_KEYS` gains the two census keys,
  and the `if axis not in dump: continue` guard was **removed**. It was written so C4b would
  start comparing without an edit; with both dumps carrying a census it can never fire, and a
  `continue` that cannot fire is one that silently skips the day a dump stops emitting one.

### Deliberately NOT in C4b

* **The station manifest's writer** — C7's station half, now unblocked.
* **Deleting the Python test bodies or `tests/science_gates.py`.** Stage 3.
* **A structured `dt` key on the station manifest.** The symmetry pull toward the
  biosphere's `dt_days` treatment is real and was resisted again: the manifest's own
  `_authority` says adding one widens the frozen surface and is its own ceremony.
* **Sharing `json_string` / `census` between the two dump examples.** The station dump now
  carries a second copy of the biosphere dump's — acceptable for one commit, and C7's station
  half should share them rather than make a third.

### Verification

`cargo test` + `cargo clippy --all-targets -D warnings` clean; `ruff check`,
`ruff format --check`, `pyright` and the full Python suite green; the crossport suite green.
⚠ `rustfmt` on the two named files only — never a module root.

Controls: an assertion carrying a recorded literal deleted → the reference's bound-literal
check red on its own line, the gate itself green, on **both** tables; a `science_gate` marker
re-added in `tests/` → the census-exhausted gate red; the roster drop → caught by predicting
the diff. ⚠ Three of the controls were **inert on the first try**, and all three were caught
by checking the mutation rather than trusting the green.

## §5p C7's station half — the last writer moves, COMPLETE 2026-08-18

The third and last of C7's halves, in the order §5m set (biosphere → authoring → C4b →
**station**), and the one that makes the flip's headline literally true: no Python program
writes a frozen contract any more.

### The gating measurement, and it came back free

`canonical_json` was measured against **all three** manifests when it was written for the
biosphere half, so the serialization question was already answered. What was not known is
whether the writer reproduces *this* file — and it does. **Byte-identical on the first
run**, third for three.

### The deliberate diff: three rows, and no value

| row | why it moved |
|---|---|
| `_comment` | named `uv run python tests/test_station_freeze_manifest.py`, which C7 makes false |
| `_authority["aux_set"].why` | said *"the splice is what a regeneration writes"* — after the move there is no splice; the writer writes it |
| `_authority["numerics_note"].why` | the hole it documents got **bigger**, and this slice measured how |

`git diff --stat`: 3 insertions, 3 deletions. No hash, set, claim, bound or horizon moved.

### ⚠⚠ The finding: the trap is PARTIAL, which is worse than the biosphere's

`numerics_note` is hand-maintained prose naming three integration steps, and C7 moves the
writer **into the crate that owns all three** (`sealed_station_scenario()`'s `bio_dt` and
`cabin_dt`, the energy scenario's `power_dt`). Measured before designing the guard:

| referent | the note as written | spliced with `{}` | regeneration gate |
|---|---|---|---|
| `bio_dt` | `dt=1/4 day` | `dt=0.25 day` | **red** — the bytes move |
| `cabin_dt` | `dt=60 s` | `dt=60 s` | **green** — `60.0_f64` Displays as `60` |
| `power_dt` | `dt=3600 s` | `dt=3600 s` | **green** — same reason |

So two of the three would auto-follow the code with C7's whole gate seeing nothing. The
control was run end to end: splicing `cabin_dt` and regenerating printed **`unchanged`**,
and only the new source-text guard reddened.

⚠ **And unlike the biosphere there is no second guard to fall back on.** That contract's
`dt_days` is at least compared against `BIO_DT` across the port boundary (it compares
equal either way, but the *ceremony* survives while the literal is typed). This manifest
has no structured step key at all, and adding one widens the frozen surface — its own
ceremony, declined for the third time here rather than smuggled in as a rider.

The guard is `rust/crates/station/tests/manifest_writer.rs`. ⚠ Its own control earned its
keep the way the biosphere's did: the bare key `numerics_note` appears **three** times in
the writer (the const, the emission site, and the `_authority` row that classifies it), so
an anchor on the key alone would silently check the wrong line. Every anchor is emission
syntax, and the ambiguity is *asserted* so the reason is a measurement rather than a
claim.

⚠ This is the step unfreeze's lesson in a third place, and with a new mechanism: there the
collision was between two *values* (`dt = 1` and its own conversion); here it is between a
value and its **rendering**.

### Deleting the writer reddened nothing — and opened the same hole, for the third time

Asked before deleting, per the authoring half's lesson: `_build_manifest` was reachable
only from `_regenerate`, itself reachable only from `__main__`, invoked by no test. **A
control with no test to turn red IS the finding**, recorded rather than quietly fixed.

And the **roster** (`name -> label, golden`) lived in the Python module *and was written
from it*, so it needed no gate — the manifest could not disagree with its own source.
Moving the writer turned one source into two copies with nothing holding them, and the two
fields at risk are exactly the ones `_authority` marks `hand`: the human label and the
golden's filename, neither of which any gate can re-derive.
`test_the_frozen_roster_is_the_references` closes it, and a control confirms it reddens on
its own line.

⚠ `_filed_under_the_roster` was **one commit old** when it was deleted. C4b added it
because the dump emits only scenarios that carry a claim while the manifest wants a key for
all thirteen; the Rust `census_json` does the same filling, from the same roster, with the
same panic-on-unknown-scenario, in the crate that owns the claims.

### The `_authority` literal went too, and the equality became a shape check

`test_every_frozen_field_declares_who_produced_it` ended with `manifest["_authority"] ==
_AUTHORITY` — the committed block against the module's own literal. The literal left with
the writer; keeping a copy purely to assert against would be the stale second copy. The
first three checks now read the block **out of the committed file**, and the fourth became
shape checks (`{side, why}`, `side` in the three, `why` is prose) — which catch a
malformed row the equality never caught either. The block's *content* is compared against
what the reference writes, by the byte gate.

⚠ The prose was moved **mechanically** — generated from the committed manifest and
diffed — not retyped. It is frozen contract text.

### What left, what changed subject, and the two stale sentences

* `_rust_reference` left with the writer, taking `_RUST_DUMP_KEYS` **and** the
  `encoding="utf-8"` pin C4b had added one commit earlier. Both were correct while they
  existed; the surviving reader in `test_inventory_parity.py` carries the same pin for the
  same reason.
* `test_the_dump_key_sets_are_the_ones_the_generators_consume` **retired** — its last row
  was the station's, and it leaves for the narrow reason the authoring half established:
  *there is no second copy of the key set left, because the copy lived in the writer.* Not
  the broader "nothing consumes the dump", which the authoring case already proved false.
* `test_inventory_parity.py`'s regeneration advice **de-branched**, exactly as its own
  comment scheduled: it printed the Rust command for the biosphere and the Python one for
  the station, and both are Rust now.
* ⚠ `regen_goldens_from_rust.py`'s warning has now been corrected **twice, both times by
  the slice that falsified it**. C7's biosphere half made *"a `--write` that moves a frozen
  golden turns nothing red"* false for the biosphere and wrote *"it is still true for the
  station"*; this half falsified that clause. Measured, not assumed — a moved station
  golden was fed to the byte gate and it went red.
* `docs/station-reference.md` named the retired command in its own unfreeze ceremony
  (step 4). Nothing gates that file, so it goes stale silently; corrected with an
  unfreeze-log entry.

### Verification

`cargo test` + `cargo clippy --all-targets -D warnings` clean; `ruff check`,
`ruff format --check`, `pyright` and the full Python suite green. ⚠ `rustfmt` on the two
named files only — never a module root.

Controls, each turning exactly the predicted tests red: a hand-edited manifest → the byte
gate; a drifted roster label → the roster gate alone; a moved golden → the byte gate; an
aux process wired in → the regenerated manifest **gains the name** *and* the checker's aux
gate reddens (the substitute for the rename control this axis cannot run); the
`numerics_note` splice → **manifest unchanged, source-text guard red**, which is the
finding rather than a pass.

### Deliberately NOT in this half

* **A structured `dt` key.** Third refusal; it widens the frozen surface.
* **Deleting the Python checker.** The completeness and conformance gates stay; retiring
  them is Stage 3's call.
* **Sharing `json_string` / `census` between the two dump examples.** C4b left a second
  copy deliberately and named it as the one to delete; the *dumps* still hand-roll their
  JSON while the *writers* use `config::canonical_json`. Converging them is a follow-up,
  not a rider on a slice whose gate is byte equality.

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

## §5q Stage 3 — the suite classification pass, COMPLETE 2026-08-18 (a measurement; no test file touched)

Stage 2 ended with §5p: no Python program writes a frozen contract any more. Stage 3 is
the last and by far the largest piece — the ~2,300-test Python suite that the C re-plan
described in one line (*"Not one slice; a classification pass first, then batches by
kind"*). **This is that classification pass, and it is a measurement only.** Nothing was
ported, nothing was retired, no test file was edited. Where a file looked trivially
portable, that is a row in the table below and not a commit.

### The method, and the two ways it can lie

Four inventories, all derived rather than hand-listed: `pytest --collect-only` per file
(2,452 collected), a regex census of `#[test]` functions per Rust file (445), each Python
test file's `src/` imports (its subsystem), and each Rust module's own test count.

⚠ **Test-name overlap is a lookup index, never a verdict.** Stripping `test_` from every
Python name and intersecting with the Rust names gives **74** — a number that is neither
necessary nor sufficient and must not be reported as coverage. It is wrong in *both*
directions, measured on this tree:

* **False negative.** `tests/test_observation.py` matches 1 of 13 Rust names, yet
  `observation.rs`'s seven tests carry the same subjects under names that dropped the
  `observe_` prefix. `test_registry.py` and `test_state.py` are the same shape.
* **False positive.** `test_authoring_monod.py` shares four names with `expr.rs` — and
  expands to **216** collected cases against six Rust tests.

⚠ **And a grep of `#[test]` is not a test census when the marker also appears in prose.**
`grep -c` counts **455**; the parsed index finds **445**. The ten-item gap is entirely
`#[test]` written inside `//!` doc comments in `science_gates.rs` and the two dump
examples. 445 is the number; the reconciliation is recorded because an unexplained
10-test gap in a measurement is exactly the kind of thing this repo's contracts exist to
refuse.

The evidence standard is therefore **asymmetric**, and that is what made 154 files
tractable. A wrong "port it" verdict costs duplicated work. A wrong "retire it" verdict
deletes the only check of something and **reddens nothing** —
[[authoring-manifest-reanchored]]'s lesson exactly. So: cheap classification everywhere,
and the falsifier for a retire verdict is a **mutation control on the reference side** —
break what the Python test asserts, confirm a *Rust* test goes red. Two such controls were
run in this pass (below); the rest belong to the batch that acts on them.

### The shape, side by side

| Subsystem | Python files | collected | lines | Reference crate | tests | lines |
|---|---:|---:|---:|---|---:|---:|
| engine (`simcore`) | 19 | 231 | 3,787 | `simcore` | 106 | 6,986 |
| boundary (`sim_io`) | 1 | 32 | 334 | *(in `simcore`)* | — | — |
| params/units | *(in domains)* | — | — | `config` | 47 | 2,542 |
| domains | 65 | 1,240 | 28,028 | `domains` | 89 | 11,701 |
| station | 24 | 256 | 8,921 | `station` | 87 | 8,425 |
| authoring | 20 | 473 | 6,387 | `authoring` | 91 | 6,657 |
| study tools (`lab`) | 5 | 46 | 1,274 | — | 0 | — |
| oracle (carve-out) | 9 | 12 | 1,148 | — | — | — |
| crossport | 25 | 143 | 6,805 | `godot_bridge` | 35 | 2,431 |
| repo/tooling | 6 | 19 | 1,290 | — | 0 | — |
| **total** | **174** | **2,452** | **57,974** | | **445** | **38,742** |

The `domains` row is the whole plan in one line: **1,240 checks against 89**, over an
implementation that is itself 11,701 lines.

### The verdicts

| | files | tests | meaning |
|---|---:|---:|---|
| **C** | 3 | 52 | the reference tests the same subjects; retire after its control |
| **C?** | 36 | 783 | a named residue must move first |
| **P** | 62 | 1,204 | the reference *implements* it and tests it nowhere |
| **P!** | 30 | 194 | a gap in the reference, not only in its tests |
| **R** | 7 | 95 | subject is the Python tree itself, or unfalsifiable in Rust |
| **R!** | 2 | 89 | ⚠ retire **only once its successor stands** — FINDING 7 |
| **K** | 3 | 4 | the PCSE carve-out |
| **D** | 11 | 31 | no natural Rust home; needs a call |
| **total** | **154** | **2,452** | |

⚠ **`R!` exists because the first draft of this table contradicted its own FINDING 7.**
`test_crossport.py` and `test_inventory_parity.py` were filed **R — retire free** on the
reasoning the plan has carried since 2026-08-17: *"their entire subject is the two ports
agreeing"*. Control B then measured that this is false for the sibling domains, where those
cross-port assertions are the only by-name gate on a real science error. The finding was
written up and **the rows were left saying R**, which is worse than either answer alone: a
reader works the table, not the prose. `R!` means *retire only once its successor stands* —
these two retire behind **S3**, never before it.

**Retirable with nothing else built: 10 files, 147 tests — 6.0 % of the suite.** Everything
else is work, and **92 files / 1,398 tests** of it is *new Rust that does not exist*, before
counting the residue inside the partly-covered files.

### ⚠⚠ FINDING 1 — the reference does not stand on its own ground

`rust/` compiles **24 files out of the tree that is being deleted**, via `include_str!`
paths that climb five directories: all 23 param YAMLs under `src/domains/**/params/` and
`src/station/params/`, plus `tests/oracle/winter_wheat_weather.json` (C9's reach-out,
which C9 guarded rather than removed). Three more reach-outs are runtime rather than
compile-time: `tests/authoring/scenarios/` (26 fixture files, read by
`scenario_files.rs`'s 40 tests **and** by `godot_bridge`), and
`tests/regression/golden/state_snapshot.json`.

**So `rm -rf src/ tests/` does not fail a test — it fails the build.** The first Stage-3
slice is therefore not a test slice at all: the *data* must be given a home inside the
reference (or a neutral top-level `data/`) before any deletion is possible. The plan's own
row (§5f) called relocating the param YAML "not now"; this pass upgrades it from tidiness
to a prerequisite.

### ⚠⚠ FINDING 2 — three of the reference's own gates live in the tree being deleted

C7's headline is that no Python program *writes* a frozen contract. It remains true, and it
is not the whole picture: **the programs that check the contracts are still Python.**

* `tests/crossport/test_manifest_writer.py` — the byte-for-byte comparison of all three
  committed manifests against what the reference writes. This is the gate §5p calls "the
  byte gate", and the trap C7 set (a provenance edit now forces a regeneration) is armed
  *by this file*. The Rust-side `manifest_writer.rs` tests (3 + 4) check the emitted
  literals and formatting, not equality with the committed file.
* `tests/crossport/test_golden_provenance.py` and every `tests/test_regression_*.py` —
  see FINDING 3.
* `tests/crossport/test_inventory_parity.py` — the dumps' only consumer.
* ⚠ **Added 2026-08-18 by S1 — this list is now four, and S1 itself lengthened it.**
  `test_golden_provenance.py::test_the_golden_census_counts_are_what_the_prose_says`, the
  counted forcing literal S1 added after finding the golden census prose stale in two
  directions. It guards prose in `golden_platform.py`, `regen_goldens_from_rust.py`, that
  same file and **`CLAUDE.md`** — and it sits in the directory S2/S6 retires, so deleting
  it lets the prose rot again with nothing red. Writing a gate into the dying tree is
  sometimes the only place it *can* go today; recording that it is there is what stops it
  vanishing quietly.
* ⚠⚠ **Added 2026-08-19 by S2 — this list is now FIVE, and the fifth is not a gate at all.**
  `tests/crossport/tiers.json`, the file `docs/native-port-reference.md` calls the cross-port
  **tolerance contract** (the 3 tiers + the measured bands for 20 of the goldens), is read by
  **no program in `rust/`** — grepped, and the only occurrence of the name in the whole Rust
  tree is a doc-comment pointer in `domains/src/lib.rs`. So one of the four freeze contracts
  has its numbers stranded in the dying tree while the doc that names them lives on. Found
  only because §5t went looking for a measured band and then declined to use one. ⚠ Unlike
  the four above, porting it means porting the comparator and the tier machinery with it —
  a different subject from "does this run still produce these bytes" — so it is recorded as
  standing work rather than folded into S2. **S6 must not reach it as a free deletion.**
  ⚠ **SHARPENED 2026-08-25 (§5y, CORRECTION 2): this entry under-states it.** The problem is
  not only that the numbers are stranded — it is that the reference has **no numeric tolerance
  at all** off Windows (`compare_structural` checks a leaf parses finite, never its value),
  while Python's band gates run on the one CI job that is a genuine cross-libm measurement.
  **ANSWERED: port the tolerance checks to Rust**, re-measuring rather than translating.

Retiring the checker therefore disarms the traps the last four slices installed. Each
needs its successor built **before** its Python original goes, not after.

### ⚠⚠ FINDING 3 — the golden regression contract has no Rust owner at all

**No Rust test compares a run against a committed golden file.** Searched exhaustively:
the only Rust references to `tests/regression/golden/` are one `include_str!` of the
`state_snapshot` *input* fixture and a comment in `emit_crew.rs`. The reference *emits*
the goldens (23 `emit_*` examples) and Python alone compares them — 17
`test_regression_*.py` files (37 tests), `tests/golden_platform.py` (the platform +
authorship policy, and the single choke point every module routes through), and
`test_golden_provenance.py` (27 collected).

25 goldens on disk, 20 of them under the cross-port tier contract, and the whole
comparison is in the tree scheduled for deletion. This is the single largest and
earliest-ordered piece of Stage 3.

> ⚠ **DISCHARGED 2026-08-19 by S2's first half (§5t).** `domains/tests/golden_regression.rs`
> and `station/tests/golden_regression.rs` compare all 19 reference-authored goldens against
> `rust/data/golden/`, and the census gates moved with them. ⚠ Two numbers in the paragraph
> above are S1-stale and left as written because they date the finding: it is **21** on disk,
> not 25 (C6 retired four), and 19 are reference-authored, not 18. The "20 under the tier
> contract" clause is the one that survives unchanged — and §5t found that `tiers.json`
> itself has no Rust reader, which is now FINDING 2's fifth entry.

### ⚠ FINDING 4 — four sibling domains: 1,411 lines of reference, no test *in `domains`*

`domains/src/{crew,eclss,power,thermal}.rs` carry **0** `#[test]` between them; Python
carries 158.

⚠ **"Zero `#[test]`" is not "unexercised", and the difference is the finding.** The code
*is* stepped from elsewhere — `authoring/src/flow_registry.rs` builds thermal inside a
test, `station`'s builder and palette build power, and the seven `emit_*` examples run all
four. Control B measured exactly what that incidental exercise catches: **two front-end
readout assertions**, and nothing that names the domain. So the accurate claim is the more
damning one — the reference's sibling-domain science is checked only where some *other*
crate happens to pin a number that depends on it. Same discipline as the 455-vs-445
reconciliation above: a count of a marker is not a statement about coverage.

### ⚠ FINDING 5 — `biosphere/flows.rs` is 1,537 lines with no test of its own

The biosphere's mechanisms are tested in the reference only through the *frozen claims*:
`science_gates.rs` (16), `science.rs` (7), `system.rs` (12) — bands and gates, not
mechanisms. Python spends ~600 tests on the mechanisms themselves (photosynthesis,
allocation, phenology, senescence, mineralization, nitrogen, transpiration, canopy, the
soil column…). A band passing is not the mechanism being right; [[wheat-partition-backfill-refused]]
is the recorded case of a frozen table passing a band *because it was fitted*.

### ⚠ FINDING 6 — extinction, and the control that corrected this entry

Extinction is a non-negotiable invariant. `integrator.rs:216` implements it; `integrator.rs`
has two tests and neither is about it; `events.rs` has none. The draft of this entry read
*"implemented in the reference and untested there"*.

**Control A falsified that.** Disabling the extinction branch (`if false && …`) and running
the workspace: **one test reddens** — `engine_vectors.rs::engine_synthetic_trajectory_is_bit_exact`,
on the `boundary.loss.carbon` leaf at step 16. The corrected finding is narrower and more
useful: extinction has **no direct test in the reference**; it is held by a single
bit-exact trajectory vector that pins the entire state, so it reports *that something
changed*, never *which mechanism broke*. ⚠ And that vector file is generated by
`tests/crossport/gen_engine_vectors.py`, one of the generators whose successor the plan
lists as "confirm per file" — so the only thing holding extinction is anchored to a
generator queued for deletion.

### ⚠⚠ FINDING 7 — the checker is not a second opinion on the sibling domains; it is the gate

**Control B**, run against a clean tree: swap the two legs of `charge_split` in
`domains/src/power.rs` so the battery stores `(1-η)` and loses `η`. Conservation still
holds exactly — this is a pure science error, the kind a behavioural test exists to catch.

* `cargo test --workspace`: **2 of 445 red**, and both are in `godot_bridge`
  (`composed_energy_station_builds_steps_and_carries_readouts`,
  `observation_projection_carries_the_derived_readouts`) — front-end readout assertions
  that happen to pin a battery number. **Nothing in `domains` or `station` noticed.**
* `pytest tests/crossport/test_crossport.py -k power`: **3 red**, immediately and by name.

So for the sibling domains the cross-port comparison is not a duplicate of a Rust gate —
**it is the gate**, and `tests/crossport/test_crossport.py` is on the delete list because
"its entire subject is the two ports agreeing". That is true of its *mechanism* and false
of its *effect*. Deleting it without a successor removes coverage that exists nowhere else.

⚠ Both controls were run against a clean tree with nothing else in flight, and both files
were restored with `git checkout --` and the workspace re-verified green — §5p's third
process trap, observed.

### ⚠ FINDING 8 — two invariants in `CLAUDE.md` are guarded on neither side

* *"`simcore` in **both** trees carries zero third-party deps."* Python's
  `test_simcore_purity.py` / `test_biosphere_purity.py` scan **Python packages only**. No
  Rust test reads a `Cargo.toml`. The Rust half of that invariant has always been prose in
  a manifest comment.
* *"`gdext` appears in `rust/crates/godot_bridge` and nowhere else."* One matching line in
  the whole tree, and it is a doc comment. Nothing asserts it.

Retiring the Python purity tests loses no Rust coverage, because there is none — but it
removes the last thing in the repo that *looks* like a purity gate, so the successor has
to be written rather than assumed.

### ⚠ FINDING 9 — seven Rust tests name the dying side as their oracle

`quantities.rs::canonical_units_match_python_table`,
`snapshot.rs::loads_the_python_golden_bit_exact`,
`hexfloat_roundtrip.rs::{parse_reproduces_python_bits,format_reproduces_python_spelling}`,
`canonical_json.rs::{matches_python_dumps_on_every_shape_the_manifests_use,escapes_the_ascii_specials_python_escapes}`,
`science_gates.rs::the_literal_scanner_matches_the_pythons_regex_on_every_shape_it_meets`.

Each is a conformance test against a side that will not exist. They do not break — the
committed vector files and goldens survive — but each becomes a claim whose stated
authority is gone, and the name will read as a lie. Re-anchoring them (to the spelling, to
the vector file, to the format itself) belongs to whichever batch retires their subject.

### ⚠ FINDING 10 — the parametrize axis: 725 of the 2,452

1,721 test functions expand to 2,452 collected cases; **725 come from `parametrize`** and
12 sites use `hypothesis`. A port that copies the function and drops the case table
silently narrows the check. The worst offenders are named so the batches can price them:
`test_authoring_monod.py` **19 → 216**, `test_light_path.py` 10 → 60,
`test_crossport.py` 34 → 81, `test_phenology.py` 52 → 90, `test_photosynthesis.py` 25 → 50.

⚠ The `hypothesis` half is already settled and should not be re-litigated: C2 (§5c) mapped
all twelve `@given` law sites to `laws.rs` + `season_order_independence.rs` and recorded
why **exhaustive enumeration beat `proptest` for eight of them**. There is no `proptest`
in the workspace and that is a measured choice, not a gap.

### ⚠ FINDING 11 — the Godot tests are not port-parity, and they are the exit criterion

Nine files / 17 tests under `tests/crossport/` compare **Rust headless against Rust inside
Godot** (bit-exact snapshot, FTZ/DAZ flags clean, Tier-0 discretes) through the actual
`gdext` cdylib. Python is only the driver. Their subject outlives Python entirely — it is
Phase 8's exit criterion, *"the exact same simulation runs headless"* — and they are
`skipif`-ed on CI, so they are local-only today. Deleting them with the crossport
directory would delete the only cross-boundary proof in the repo. Their new driver is a
design question (a Rust integration test that shells out to Godot, or a script), which is
why they are filed **D** rather than **P**.

> ⚠ **CORRECTED 2026-08-25 (§5y): "local-only today" is FALSE and was already false when
> this entry was written.** The `godot-parity` CI job installs headless Godot and runs
> **15 of the 17**; only the two `-m slow` cases in `test_godot_two_rate_parity.py` are
> mandatory-local. This entry read the `skipif` in the test file rather than the workflow
> that defeats it. **ANSWERED 2026-08-25: ported to Rust** (`godot_bridge/tests/`), the
> design question settled toward the integration test rather than a script.

### ⚠ FINDING 12 — the value-switch seam never moved, and C1 said it would

C1's row reads *"⚠ **Take the user's harness with it** — `config/overrides.py` is the same
file set"*. It did not: there is no `override` anywhere in `rust/crates/config`, and
`tests/test_param_overrides.py` (19 tests) is the only thing exercising the seam. So the
user's in-memory parameter-substitution seam is Python-only, and Stage 3 collides with the
value-switch plan rather than being independent of it. ⚠ Same shape as §5j's finding that
*a slice's prerequisites are not in its own row of the plan table* — this one was in the
row and still did not happen.

### The batch order this implies

Ordering falls out of the findings rather than out of the file counts. Stage 3 is **six
slices, not one**, and the first two are not test work:

| # | Slice | Why here |
|---|---|---|
| **S1** | **The reference's own ground** — relocate the 23 param YAMLs + the weather fixture + the 26 scenario fixtures + the goldens into paths the reference owns | FINDING 1. Nothing can be deleted until this lands; it moves data, not science, so it should be byte-neutral and provable as such |
| **S2** | **The three gates come with it** — the manifest byte gate, the golden comparison + `golden_platform.py`'s policy, the inventory dumps' consumer | FINDINGS 2 + 3. These guard the *contracts*; every later slice runs behind them. ⚠ **Splits in two, and the split is structural, not a size call** (§5t): the golden comparison stops at `station`, so `station` can own its census; the manifest gate spans `domains` + `station` + `authoring`, so it cannot. **COMPLETE 2026-08-19** — first half (§5t) discharged FINDING 3 and carried the S1 forcing literal; second half (§5u) moved the three manifest writers into their crates and gave each contract a byte gate, plus Rust successors for the three `test_inventory_parity` claims the byte gate does NOT subsume. FINDING 2 grew to five on the way |
| **S3** | **The sibling domains** — crew, eclss, power, thermal: 158 Python tests, 0 Rust | FINDINGS 4 + 7. The cheapest large win, and Control B shows the coverage is real and currently on loan from the cross-port comparison. ⚠ **BUILT — see §5w (COMPLETE 2026-08-19).** ⚠ **Designed in §5v, and this row is superseded on two counts**: the real figure is **160 collected cases over nine files**, not 158 over eleven — `test_bvad_validation.py` and `test_crew_coupled_loop.py` are deferred to `station` as science items. And **Control B's verdict here is dated**: §5v re-ran it and got 12 red, not 2 |
| **S4** | **The engine residue** — extinction, aux, environment, integrator, multirate, the purity + `gdext` gates | FINDINGS 6 + 8. Small, invariant-shaped, and each one is a `CLAUDE.md` non-negotiable. ⚠ **BUILT — see §5x (COMPLETE 2026-08-25).** It is **two** slices wearing one row: five behavioural files whose subject is arithmetic, and two structure gates whose subject is what a manifest contains — different acceptance criteria, and the structure half taken first because it was the only part that could still change the slice's shape |
| **S5** | **The biosphere mechanisms** — **599 tests over 20 files**, whose reference owner is `science.rs`, not `flows.rs` | FINDING 5. The largest and the one to take last, because it is where a silent narrowing does the most damage. ⚠ **DESIGNED — see §5ad (measured 2026-08-26, before any Rust).** This row is superseded on three counts: the owner is `domains/src/biosphere/science.rs` (34 public functions, **7** tests), not `flows.rs`, which holds wiring; the batching is by **Rust surface**, not by crop or process — seven batches, one of which (soil carbon) has no extracted functions and must be tested at flow level; and a six-mutation control shows **five of the six caught by nothing that is about them**, so the coverage is on loan from goldens and unrelated bands. One cited `science.rs` function, `intercepted_fraction`, is **dead code** and would silently absorb a whole batch |
| **S6** | **The retirements** — the two `R!` rows (only once S3's successors stand), the generators (each against its named successor), the Python tree itself | Only after S1–S5 have successors standing. ⚠ The cross-port comparison is **not** a free deletion: FINDING 7 makes it the sibling domains' only gate until S3 lands. The **D** rows (Godot drivers, `test_context_budget.py`, `test_headless_cli.py`) need answers before this slice, not during it. ⚠ **ANSWERED 2026-08-25 (§5y) — and the answers put four BUILD items in front of S6, not four filings:** all four went maximal-Rust, so `test_context_budget.py`, `test_headless_cli.py`, the nine Godot drivers and the `tiers.json` band gates each need a Rust successor written, and the last carries an unfreeze ceremony on `docs/native-port-reference.md`. S6 is a deletion slice again only once those stand |

⚠ **`test_context_budget.py` is the one row with no home on either side.** It guards this
repo's own documentation discipline — `CLAUDE.md`'s byte ceiling, the log index ↔ record
pairing, the memory index. There is no simulation subject, so "port it to Rust" is a
category error and "delete it" removes the only enforcement the context budget has ever
had. Named as a decision, not bucketed by reflex. `test_suite_runtime.py` is the opposite
and needs no decision: its subject is pytest's own priority handling, so it dies with
pytest by definition.

### The per-file table

Verdict codes as above. "Reference owner" names the module that owns the same subject and
its test count; **(0 tests)** means the implementation is there and nothing checks it.


#### **C — covered by the reference** (retire after its control) — 3 files, 52 tests

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `test_drift.py` | 19 | domains/tests/drift.rs (18) | C5 moved the folds; residue is helper-level (max_abs, same_phase_diffs) inside the same file |
| `test_expr.py` | 18 | simcore/src/expr.rs (23) | all 18 subjects present; `unsupported_binop_raises` is unrepresentable (BinaryOp is a closed enum) |
| `test_rng.py` | 15 | rng.rs (4) + rng_vectors.rs (3) + laws.rs (3) | residue is type-rejection tests that are unrepresentable in Rust |

#### **C? — partly covered** (the residue moves first) — 36 files, 783 tests

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `test_authoring_monod.py` | 216 | simcore/src/expr.rs (6 monod tests) | FINDING 9 — 19 functions → 216 collected cases against 6 Rust tests |
| `test_light_path.py` | 60 | domains/src/biosphere/light_path.rs (3) | residue large: 10 functions → 60 collected against 3 Rust tests |
| `test_config.py` | 38 | config crate (47) | the pint half tests a mechanism the reference already replaced; it retires with the param generators |
| `test_sim_io_snapshot.py` | 32 | simcore/src/snapshot.rs (8) | residue: 10 of 18; and `loads_the_python_golden_bit_exact` is FINDING 8 again |
| `test_authoring_kinetics.py` | 31 | authoring/src/expr_parser.rs (20) |  |
| `test_authoring_frozen_flows.py` | 30 | authoring/src/flow_registry.rs (4) + scenario_files.rs |  |
| `test_lighting_run.py` | 24 | station/tests/day_neutral_lighting.rs (8) | residue: the whole lamp-flow and lamp-loader half (15 of 24) |
| `test_freeze_manifest.py` | 21 | domains/tests/manifest_writer.rs (3) + science_gates.rs (16) | completeness is now largely inherited by the writer; the self-controls are not |
| `test_authoring_compose.py` | 21 | authoring/tests/scenario_files.rs (40) | residue: the golden-bytes halves (FINDING 3 again) |
| `test_authoring_multirate_partition.py` | 20 | authoring/tests/multirate.rs (18) |  |
| `test_flow.py` | 18 | simcore/src/flow.rs (4) | residue: per-quantity energy diagnostic; absolute/relative balance tolerances; domains_touched |
| `test_station_perturbations.py` | 17 | station/tests/perturbations.rs (10) | residue: the conserves-with-sink and returns-to-setpoint halves |
| `test_state.py` | 16 | simcore/src/state.rs (5) | residue: non-finite threshold, stock defaults/replace; frozen-ness and negative-n are unfalsifiable |
| `test_observation.py` | 16 | simcore/src/observation.rs (7) | names differ, subjects mostly match; residue: hashability, frozen-ness (unfalsifiable) |
| `test_authoring_rate_precondition.py` | 16 | authoring/tests/multirate.rs (18) |  |
| `test_station_freeze_manifest.py` | 15 | station/tests/manifest_writer.rs (4) + science_gates.rs (5) | same shape as the biosphere one |
| `test_composition.py` | 14 | state.rs + flow.rs + conservation.rs | residue: composition validation rejects (non-finite coeff, non-Quantity key, own-quantity positive) |
| `test_authoring_templates.py` | 14 | authoring/tests/scenario_files.rs (40) |  |
| `test_authoring_freeze_manifest.py` | 14 | authoring inventory dump + the Python byte gate | FINDING 2 — the byte gate itself is Python |
| `test_conservation.py` | 13 | simcore/src/conservation.rs (6) | residue: conservation across extinction; the relative-tolerance term; boundary/stored ledger split |
| `test_authoring_multirate_composability.py` | 13 | authoring/tests/multirate.rs (18) |  |
| `test_weather.py` | 12 | domains/src/biosphere/weather.rs (4) | C9 moved the path; residue: the VPD helpers |
| `test_authoring_multirate_identity.py` | 12 | authoring/tests/multirate.rs (18) |  |
| `test_arbitration.py` | 11 | simcore/src/arbitration.rs (4) + laws.rs + engine_vectors.rs | residue: whole-flow multi-quantity scaling; standalone check_no_overdraw; the zero-firings report |
| `test_edge_cases.py` | 11 | arbitration.rs + engine_vectors.rs | residue: empty-registry passthrough; exact-fit withdrawal landing at zero without arbitrating |
| `test_perturbations.py` | 11 | domains/src/biosphere/perturbations.rs (4) | residue: the conservation-with-sink halves |
| `test_boundary.py` | 10 | simcore/src/boundary.rs (3) | residue: clamped sink accumulator; loss-sink routing conservation; boundary closing an unbalanced harvest |
| `test_authoring_multirate_run.py` | 9 | authoring/tests/multirate.rs (18) |  |
| `test_co2_compensation_band.py` | 8 | domains/src/biosphere/science_gates.rs (16) | C4 moved the band; residue: the probe arithmetic |
| `test_authoring_dt_hazard.py` | 8 | authoring/tests/scenario_files.rs (40) |  |
| `test_quantities.py` | 7 | simcore/src/quantities.rs (3) | FINDING 8 — `canonical_units_match_python_table` names the dying side as its oracle |
| `test_authoring_crew.py` | 7 | authoring/tests/scenario_files.rs (40) |  |
| `test_registry.py` | 5 | simcore/src/registry.rs (3) + laws.rs | residue: the domain index (matches stock domain; read-only) |
| `test_authoring_param_packs.py` | 5 | authoring/tests/scenario_files.rs (40) |  |
| `test_day_neutral_warm_habitat.py` | 4 | station/tests/day_neutral_lighting.rs (8) |  |
| `test_authoring_multirate_compose.py` | 4 | authoring/tests/multirate.rs (18) |  |

#### **P — port** (the reference implements it and tests it nowhere) — 62 files, 1204 tests

⚠⚠ **The `Reference owner` cell on every FINDING 5 row is WRONG — corrected in §5ad, measured 2026-08-26.** It names `flows.rs (0 tests)`, but `flows.rs` holds the flow *wiring*; the equations live in `domains/src/biosphere/science.rs` (34 public functions, 7 tests). The four soil-carbon rows (`mineralization`, `soil_fractionation`, `decomposition`, `microbial_respiration`) are the exception — those equations really are inline in `flows.rs`, which is why §5ad batches them apart. Do not port a mechanism against this column without reading §5ad first; one of the functions it would send you to is dead code.

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `test_phenology.py` | 90 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5; 52 functions → 90 collected |
| `test_photosynthesis.py` | 50 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5; 25 → 50 collected |
| `test_transpiration.py` | 46 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_allocation.py` | 43 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_nitrogen.py` | 37 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_senescence_form.py` | 37 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5; 2855 lines, the largest single file |
| `test_chamber.py` | 34 | domains/src/biosphere/system.rs (12) | scenario behaviour, not covered |
| `test_decade_stability.py` | 34 | domains/tests/drift.rs (18) | the decade / 50-yr guards; drift.rs holds the folds, not these bounds |
| `test_canopy.py` | 33 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_mineralization.py` | 32 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_power_flows.py` | 29 | domains/src/power.rs (0 tests) | FINDING 4 |
| `test_soil_fractionation.py` | 29 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_soil_layers.py` | 27 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_respiration.py` | 25 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_biosphere_stress.py` | 24 | domains/src/biosphere/system.rs (12) |  |
| `test_aux.py` | 23 | simcore/src/auxiliary.rs (0 tests) | only laws.rs touches aux, and only for order-independence |
| `test_eclss_flows.py` | 23 | domains/src/eclss.rs (0 tests) | FINDING 4 |
| `test_thermal_flows.py` | 22 | domains/src/thermal.rs (0 tests) | FINDING 4 |
| `test_carbon_budget.py` | 22 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_stem_reserves.py` | 22 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_environment.py` | 20 | simcore/src/environment.rs (3) | 15 of 18 subjects absent: protocol conformance, rejects, rebinding, mixed dispatch, unknown-var |
| `test_consumer.py` | 20 | domains/src/biosphere/system.rs (12) + emit_consumer.rs |  |
| `test_decomposition.py` | 19 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_crew_flows.py` | 18 | domains/src/crew.rs (0 tests) | FINDING 4 |
| `test_builders.py` | 18 | domains/src/biosphere/system.rs (12) | partial |
| `test_bioregenerative_station.py` | 18 | authoring (the fixtures are read by scenario_files.rs) | authored content, runtime-only |
| `test_microbial_respiration.py` | 17 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_water_cycle.py` | 17 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_multirate.py` | 17 | simcore/src/multirate.rs (0 tests) | the master-step driver has no direct Rust test |
| `test_eclss_run.py` | 16 | domains/src/eclss.rs (0 tests) | FINDING 4 |
| `test_root_depth.py` | 16 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_thermal_run.py` | 15 | domains/src/thermal.rs (0 tests) | FINDING 4 |
| `test_gas_exchange.py` | 15 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_nitrogen_form.py` | 15 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_compartments.py` | 15 | domains/src/biosphere/stocks.rs (0 tests) |  |
| `test_water_recovery_run.py` | 15 | station/src/water.rs (0 tests) |  |
| `test_power_run.py` | 14 | domains/src/power.rs (0 tests) | FINDING 4 |
| `test_potato_crop.py` | 14 | domains/src/biosphere/params.rs (9) | the second species: params covered, behaviour not |
| `test_station_run.py` | 14 | station/src/system.rs (0 tests) |  |
| `test_cabin_run.py` | 14 | station/src/cabin.rs (0 tests) |  |
| `test_integrator.py` | 13 | simcore/src/integrator.rs (2) + laws.rs | residue: the Euler formula itself, RK4 order-of-accuracy, dt-linearity, forcing piecewise-constant in a step |
| `test_perennial_chamber.py` | 13 | domains/src/biosphere/system.rs (12) + emit_perennial.rs |  |
| `test_crew_run.py` | 12 | domains/src/crew.rs (0 tests) | FINDING 4 |
| `test_sealed_chamber.py` | 12 | domains/src/biosphere/system.rs (12) |  |
| `test_harvest_run.py` | 12 | station/src/harvest.rs (0 tests) |  |
| `test_authoring_export_fidelity.py` | 12 | authoring/src/run.rs (0 tests) |  |
| `test_power_self_discharge.py` | 11 | domains/src/power.rs (0 tests) | FINDING 4 |
| `test_chamber_scale.py` | 11 | domains/src/biosphere/system.rs (12) | the chambers are built in the reference; the scale argument is not checked there |
| `test_season.py` | 10 | domains/src/biosphere/system.rs (12) + emit_season.rs |  |
| `test_greenhouse_run.py` | 9 | station/src/greenhouse.rs (0 tests) |  |
| `test_sealed_station_stability.py` | 9 | station/src/sealed.rs (0 tests) |  |
| `test_authored_habitat.py` | 9 | authoring (the fixtures are read by scenario_files.rs) | authored content, runtime-only |
| `test_crop_param_set.py` | 8 | domains/src/biosphere/params.rs (9) | partial: the loader is covered, the per-crop set is not |
| `test_bvad_validation.py` | 8 | domains/src/crew.rs (0 tests) | the crew calibration against BVAD Table 3-31; the params are in the reference, the calibration claim is not |
| `test_crew_coupled_loop.py` | 8 | station/examples/emit_sealed_station.rs (the coupled scenario) |  |
| `test_extinction.py` | 7 | simcore/src/integrator.rs:216 (implemented, 0 tests) | FINDING 6 — a non-negotiable invariant whose only direct check is this file |
| `test_nitrogen_throttle.py` | 7 | domains/src/biosphere/flows.rs (0 tests) | FINDING 5 |
| `test_compartment_ledger.py` | 7 | domains/src/biosphere/stocks.rs (0 tests) |  |
| `test_authoring_reversal_gate.py` | 7 | authoring/src/run.rs (0 tests) |  |
| `test_o2_makeup_reversal.py` | 4 | station (the seam is built in the reference) |  |
| `test_stability.py` | 3 | simcore (the engine; the 100k-step run is the test artifact) | no Rust referent for the run itself |
| `test_sealed_station_landmine.py` | 3 | station/src/sealed.rs (0 tests) |  |

#### **P! — port, and the reference does not implement it either** — 30 files, 194 tests

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `test_acceptance_gate.py` | 45 | none | the plausibility-band census and its margin arithmetic exist on neither side |
| `crossport/test_golden_provenance.py` | 27 | none | FINDING 3 |
| `test_param_overrides.py` | 19 | none — src/config/overrides.py never moved | FINDING 10; collides with the value-switch plan |
| `test_oracle_match.py` | 12 | none | src/lab/oracle_match.py; decided → Rust, and nothing is there yet |
| `test_oracle_gap.py` | 9 | none | PCSE-free (reads the committed JSON); the comparison arithmetic is Python-only |
| `test_regression_long_horizon.py` | 7 | none | FINDING 3 |
| `test_rk45.py` | 7 | none | src/lab; decided 2026-08-17 → Rust, and nothing is there yet |
| `test_oracle_gap_spring_wheat.py` | 6 | none | same |
| `crossport/test_manifest_writer.py` | 6 | none | FINDING 2 — the byte gate on all three frozen contracts is Python |
| `test_convergence.py` | 5 | none | src/lab; decided 2026-08-17 → Rust, and nothing is there yet |
| `test_oracle_smoke.py` | 5 | none | reads the committed JSON; PCSE-free; no Rust referent |
| `test_oscillator.py` | 4 | none | the RK4-vs-Euler invariant-drift study exists on neither side |
| `test_regression_sealed_station.py` | 4 | none | FINDING 3 |
| `oracle/test_reference_fixture.py` | 4 | none | PCSE-free; the fixtures' only routine validation, and no Rust referent |
| `oracle/test_lintul3_fixture.py` | 4 | none | same |
| `test_regression_consumer_season.py` | 2 | none | FINDING 3 |
| `test_regression_crew.py` | 2 | none | FINDING 3 |
| `test_regression_eclss.py` | 2 | none | FINDING 3 |
| `test_regression_perennial_season.py` | 2 | none | FINDING 3 |
| `test_regression_power.py` | 2 | none | FINDING 3 |
| `test_regression_power_self_discharge.py` | 2 | none | FINDING 3 |
| `test_regression_sealed_season.py` | 2 | none | FINDING 3 |
| `test_regression_season.py` | 2 | none | FINDING 3 |
| `test_regression_thermal.py` | 2 | none | FINDING 3 |
| `test_regression_cabin.py` | 2 | none | FINDING 3 |
| `test_regression_greenhouse.py` | 2 | none | FINDING 3 |
| `test_regression_harvest.py` | 2 | none | FINDING 3 |
| `test_regression_lighting.py` | 2 | none | FINDING 3 |
| `test_regression_station.py` | 2 | none | FINDING 3 |
| `test_regression_water_recovery.py` | 2 | none | FINDING 3 |

#### **R — retire free** (subject is the Python tree, or unfalsifiable in Rust) — 7 files, 95 tests

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `test_biosphere_purity.py` | 39 | domains/tests/biosphere_spine_purity.rs (10) | **FINDING 8**, not 7 — and this row is `R!`-shaped, not `R`: its successor had to be *written*. S4 wrote it (§5x) |
| `test_simcore_purity.py` | 20 | simcore/tests/workspace_purity.rs (18) | **FINDING 8**, not 7 — subject is the Python package; the Rust zero-dep charter was guarded by nothing until S4 (§5x). `R!`-shaped, not `R` |
| `test_biosphere_demo.py` | 17 | domains/biosphere/demo.py has no Rust referent | C6 retired both demo goldens; its one engine claim (decision #16) is covered by environment.rs::forcing_and_shared_resolve |
| `test_authoring_multirate_crossport_anchor.py` | 7 | authoring/tests/multirate.rs (18) | its whole subject is the two ports agreeing on the partition |
| `test_smoke.py` | 5 | none | imports the Python packages |
| `test_suite_runtime.py` | 4 | none | guards pytest's own priority handling; dies with pytest by definition |
| `test_events.py` | 3 | simcore/src/events.rs (0 tests) | subject is Python dataclass frozen-ness; a Rust struct is frozen by construction — unfalsifiable |

#### **R! — retire only once its successor stands** (⚠ FINDING 7 overrides the obvious reading) — 2 files, 89 tests

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `crossport/test_crossport.py` | 81 | n/a | ⚠ FINDING 7 overrides the obvious reading: its MECHANISM is port-vs-port, its EFFECT is the only by-name gate on the sibling domains. Retires behind S3, never before |
| `crossport/test_inventory_parity.py` | 8 | the three dump examples | ⚠ same shape: the dumps survive as the writers' input, but nothing else consumes them today |

#### **K — keep** (the PCSE carve-out) — 3 files, 4 tests

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `oracle/test_potato_regeneration.py` | 2 | n/a | the carve-out |
| `oracle/test_oracle_regeneration.py` | 1 | n/a | the carve-out |
| `oracle/test_lintul3_regeneration.py` | 1 | n/a | the carve-out |

#### **D — decide** (no natural home; the user's call) — 11 files, 31 tests

⚠ **ALL ELEVEN ANSWERED 2026-08-25 — see §5y.** Every one went the maximal-Rust way; no
"keep it as a script CI calls" island survives. These are now **build items before S6**, not
deletions, and none of them is free.

| File | tests | Reference owner | Note |
|---|---:|---|---|
| `test_context_budget.py` | 10 | none | guards the repo's own docs; no Rust subject exists. **ANSWERED → rewrite in Rust** (§5y); its crate home is still open |
| `crossport/test_headless_cli.py` | 4 | none | drives the Rust CLI from Python. **ANSWERED → faithful Rust port** (§5y), nested cargo accepted to keep byte-exactness off Windows |
| `crossport/test_godot_from_file.py` | 4 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_compose.py` | 2 | godot_bridge (35 tests) | FINDING 11 — Rust-headless vs Rust-in-Godot, not port-vs-port; ~~local-only~~ (**false — 15 of 17 run on CI, §5y**) — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_flow_inspection.py` | 2 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_save_load.py` | 2 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_time_controls.py` | 2 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_two_rate_parity.py` | 2 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_objectives.py` | 1 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_parity.py` | 1 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |
| `crossport/test_godot_perturbations.py` | 1 | godot_bridge (35 tests) | FINDING 11 — **ANSWERED → ported to `godot_bridge/tests/`** (§5y) |

### Deliberately NOT in this pass

* **Any edit to a test file.** Including the ~15 that are one-line ports.
* **The remaining mutation controls.** Two were run because they decided findings — and
  one of them (A) falsified this pass's own draft while the other (B) forced the `R!` code
  into existence, which is the argument for running the rest rather than reasoning them.
  Every remaining retire verdict gets its control in the slice that acts on it, one
  targeted subset at a time.
* **Answering the four **D** questions.** They are the user's, and S6 is blocked on them.
* **Re-pricing the `oracle` carve-out.** Unchanged from 2026-08-17: the three PCSE runners
  and the three `-m oracle` regeneration tests stay Python; the four PCSE-free fixture
  tests move.
* **Deleting anything.** `git diff` is empty for `src/`, `rust/`, the goldens and all three
  manifests; the only files this slice touches are this plan, the log and the memory index.

## §5r Stage 3 — S1, the reference's own ground; the COMPILE-TIME half COMPLETE 2026-08-18

§5q's FINDING 1 said the reference does not stand on its own ground: `rust/` compiled **24
files out of the tree being deleted**, so `rm -rf src/ tests/` would not fail a test — it
would fail the **build**. S1 is the slice that fixes that, and it splits the way the
dependency splits. This half moves the two **compile-time** reach-outs (the 23 frozen param
YAMLs + the weather fixture); the runtime ones (26 scenario fixtures, the goldens) follow in
the second half.

### Three gating measurements, taken before a file moved, that decided the slice's category

1. **The manifests are path-free.** `param_files` is basename → sha-256; each `scenarios/*`
   entry records a golden **basename**; the authoring contract's `parity_vectors` likewise.
   Grepped all three for `src/domains`, `src/station/params`, `tests/regression/golden`,
   `tests/authoring/scenarios`: **zero hits.** So S1 is a **pure rename**, not an unfreeze —
   and the byte gate is the proof rather than the plan's assertion.
2. **`.gitattributes` has no path-scoped rule** (`* text=auto eol=lf`, globally). This
   mattered because `include_str!` embeds the **working tree** and one frozen file
   (`senescence.yaml`) is CRLF on this box while the index is LF — if normalization were
   scoped by path, the new home would inherit different rules and a `git mv` that never
   touches content would still change the embedded bytes. It is not scoped, so it cannot.
3. **The golden count is stale in three places.** Plan, `golden_platform.py` and `CLAUDE.md`
   all say **25**; disk holds **21**. Cause identified rather than guessed: C6 (`01bf957`)
   deleted four Python-only goldens. Enumerate from disk —
   [[coverage-roster-is-not-the-manifest]] again. Left for the second half, which is the one
   that touches those files.

### The home, and the rule that chose it

Not tidiness: **where the dependency actually is.**

| data | home | why |
|---|---|---|
| 15 biosphere + 5 sibling + 3 station param YAML | `rust/crates/domains/params/<domain>/`, `rust/crates/station/params/` | compiled into *those* crates specifically; `include_str!("../../params/biosphere/canopy.yaml")` and `PARAMS_DIR = concat!(CARGO_MANIFEST_DIR, "/params/biosphere")` are **not reach-outs at all**, which is what "a path the reference owns" means |
| `winter_wheat_weather.json` | `rust/crates/domains/data/` | same rule. ⚠ The discriminator is *the reference compiles **this** one in*, **not** *the three weather series are a set* — `potato_weather.json` and `spring_wheat_weather.json` stay in the surviving `tests/oracle/` carve-out, which now reaches **into** the reference for the third. That direction is the correct one for a diagnostic |
| scenario fixtures, goldens | the second half | read/emitted **across** crates, so neither can be crate-owned without one crate reaching into another's private tree |

Per-domain subdirectories were kept: the manifests are **basename-keyed**, and C8 asserted
basename uniqueness across the six directories precisely because a collision would silently
collapse two frozen entries. Flattening would have re-opened that.

### ⚠⚠ The whole directory moved — `demo.yaml` and `crops/potato/` included — and that is not conservatism

The tempting move is to take the fifteen files the census names and leave the rest, since
`demo.yaml` is Python-only and the four potato overrides are a deferred second species. It
would have been wrong, and the reason is a **control**, not tidiness.

`tests::a_recursive_walk_reddens_the_census` proves *"a directory is not a category"* by
asserting `recursive.len() == census.len() + 4`. That assertion has teeth **only because the
four potato files sit in a subdirectory of `PARAMS_DIR`** — leave them behind and the
assertion cannot be satisfied at all (`recursive.len()` would equal `census.len()`), so it
goes **red**. ⚠ That is the sharper danger, not a milder one: a control that fails for a
reason nobody caused invites the repair *"delete the obsolete test"*, and the guard is lost
to a tidy-up rather than to a decision. Its sibling
`the_recursive_walk_would_see_four_more_and_the_census_does_not` is the same shape. Both went
green at the new home, which is positive evidence that the directory *shape* survived and not
merely its fifteen frozen files.

`demo.yaml` came for the parallel reason: it keeps the **exclusion-by-name** rule true
verbatim, so `param_files()`, the dump's `assert_eq!(files.len(), 15)` and the census test's
own literal all stay correct with zero value change. It dies with `demo.py` at S6, and the
by-name exclusion dissolves **then**, deliberately, inside a retirement — not here, inside a
slice whose entire claim is that it moves data and not science.

### ⚠ A negative assertion about a directory goes vacuous when the directory moves

`test_mineralization.py` asserts `not (params_dir / "mineralization.yaml").exists()` — the
record that a retired parameter stayed retired. A directory that does not exist satisfies
that vacuously, so a mis-resolved path would have turned a real check into a green no-op
**silently**. The positive half now runs first (`params_dir.is_dir()`, plus a file that must
be there). Found by asking of every re-pointed path *what does this assert if the path is
wrong* — the only question that separates a live negative from a vacuous one.

### The two-direction control: build vs test, with `src/` and `tests/` renamed away

Measured, not reasoned:

* **`cargo build` succeeds.** That is this half's claim, and it was false this morning.
* **`cargo test` fails**, and the panics name `crates/authoring/tests/scenario_files.rs`
  (plus `snapshot.rs`'s golden read) — *exactly* the runtime reach-outs the second half owes.
  The control therefore doubles as a to-do list that cannot be padded or forgotten.

### The Python side: one definition, not six

Every loader spelled its own `Path(__file__).parent / "params"`, and the weather fixture was
spelled out in **40** test modules. A repo-root climb copied into 46 places is the *a rule
with two copies has one that is stale* shape this repo refuses, so `src/config/paths.py`
holds it once: `REPO_ROOT`, `DOMAIN_PARAMS_ROOT`, `BIOSPHERE_PARAMS_DIR`,
`STATION_PARAMS_DIR`, `WINTER_WHEAT_WEATHER`. Adding a module to a retiring tree is a
deliberate call: it makes the eventual deletion **one file** instead of a hunt, and it is the
only place that states Python is now a tenant of the reference's ground rather than its owner.

⚠ **Priced consequence, stated rather than discovered later:** the Python packages no longer
carry their own data, so a *non-editable* wheel build would ship loaders whose params are not
in the wheel. The project installs editable (`_editable_impl_biosphere_sim.pth` → `src`) and
the checker was already checkout-only (it reads goldens out of `tests/`), so nothing breaks
today — but "the Python tree is installable standalone" stopped being true and is not coming
back.

### The deliberate manifest diff: two prose strings, predicted, then measured

Two `_authority` `why` entries in the biosphere contract described this slice **in the future
tense**, which is this repo's most-repeated failure mode. Both were rewritten and the manifest
regenerated as the git-visible record; the diff is **exactly two `why` strings** — no key, no
hash, no `side`.

* `param_files` — S1 moved the ground under the key without moving the key; not one of the 23
  hashes changed, and the 15-of-20 rule carried over *because the whole directory moved*.
* `forcing/weather_fixture` — its own text said the name becomes derivable once *"the
  relocation slice moves the fixture out of `tests/` to a data home the reference can read at
  runtime."* ⚠ **S1 met that condition and deliberately did not act on it.** Deriving the name
  flips this key's side `hand → rust`, which is a **re-anchoring**; taking it inside a slice
  whose claim is *data moved, authority did not* would make the byte-neutrality claim
  unfalsifiable. The successor is now named in the manifest rather than left implicit.

### Verification

`cargo build` / `cargo test` / `cargo clippy --all-targets` clean; `uv run pytest -n 12` →
**2,447 passed, 5 skipped (13m08s)**; `ruff check` / `ruff format` clean. The three manifests
and all 21 goldens are byte-identical apart from the two `why` strings above — `git status` on
`tests/regression/` and on the other two manifests is empty, which is the byte-neutrality
claim in its checkable form. All 28 param files and the weather fixture are recorded by git as
100 %-similarity **renames**.

### Deliberately NOT in this half

* **The runtime reach-outs** — 26 scenario fixtures and 21 goldens. Named by the control above.
* **Re-anchoring `forcing/weather_fixture`** — buildable now, and its own successor.
* **The stale golden count** (25 vs 21) in `golden_platform.py`, `CLAUDE.md` and this plan —
  it belongs to the half that edits those files.
* **Deleting anything.** `demo.yaml` and `crops/potato/` moved rather than being dropped, on
  purpose; every retirement is still S6's.

## §5s Stage 3 — S1's RUNTIME half; the reference now stands entirely on its own ground, COMPLETE 2026-08-18

§5r moved what the reference compiles in and left a control standing as the to-do list: with
`src/` and `tests/` renamed away, `cargo build` succeeded and `cargo test` failed, its panics
naming `scenario_files.rs` and `snapshot.rs`. This half moves what those two read at
**runtime** — the 26 authored-scenario fixtures and the 21 regression goldens — and closes
FINDING 1.

### The home, and why it is not crate-local this time

`rust/data/scenarios/` and `rust/data/golden/`, workspace-level rather than inside a crate.
The rule is unchanged from §5r — *put the data where the dependency is* — and it lands
somewhere different because the dependency is shaped differently:

* the scenario fixtures are read by `authoring`'s 40 tests **and** by `godot_bridge`;
* the goldens are emitted by `emit_*` programs in **four** crates.

Neither can live inside one crate without the others reaching into that crate's private tree,
which is the thing this slice exists to stop. `rust/data/` is the smallest home that is
nobody's private tree. ⚠ It is deliberately **not** the repo-root `scenarios/` directory —
that is authored *content* (runtime artifacts, never reference) and the two have never been
the same thing.

### The control from §5r, re-run: both directions now green

`cargo build` **and** `cargo test` both pass with `src/` and `tests/` renamed away. That is
FINDING 1 discharged in the form it was written: *"`rm -rf src/ tests/` does not fail a test —
it fails the build"* is no longer true of either.

### ⚠⚠ The finding: the golden census prose was stale in two directions at once, and nothing gates it

While re-pointing `GOLDEN_DIR` the counts were re-measured from disk rather than copied:
**21 goldens, 19 of them Rust-authored, 2 Python-authored** (`drift_summary.json`,
`state_snapshot.json`). Four files said something else — `golden_platform.py`'s
*"Eighteen of the twenty-five goldens are now written by the Rust port"*,
`regen_goldens_from_rust.py`'s *"The census — 25 goldens"*, `test_golden_provenance.py`'s
*"two of the eighteen"*, and `CLAUDE.md`'s *"25 golden files"*.

**Both halves of that sentence had rotted, in opposite directions and for different reasons:**

* **18 → 19 authored**, when C5 folded the station drift summary in Rust;
* **25 → 21 on disk**, when C6 (`01bf957`) retired four Python-only goldens.

⚠ **Nothing was broken and nothing was ungated — which is exactly why it survived.** The
*rosters* are derived and were right the whole time; `test_every_committed_golden_is_classified`
enumerates from the directory precisely because this repo has been caught trusting a
hand-maintained list before. What rotted is the layer no gate has ever owned: the sentences a
reader uses to orient. Fourth instance of [[freeze-prose-half-is-ungated]] in this flip, and
the first where the stale number sits in `CLAUDE.md`, the file loaded into every session.

**So the fix is not only the four corrections.** `test_golden_provenance.py` gains
`test_the_golden_census_counts_are_what_the_prose_says` — two counted literals `(21, 19)`
whose failure message **names the four prose sites**. Deliberately a *forcing function*, not a
second census: it cannot check that a sentence is true, only that somebody looked when the
count moved. Same shape as `param_files`'s `assert_eq!(files.len(), 15)` on the Rust side, and
stated as such so nobody later "simplifies" it into a derived check that would rot silently.

**Control, run before the gate was believed:** one name added to `RUST_AUTHORED` → the gate
goes red and the message names the prose sites; reverted → green. ⚠ Reverted by an in-place
reverse edit from a copy in the temp tree, **never `git checkout`** — that file carried
uncommitted work, and discarding an uncommitted slice with `git checkout` is a cost this flip
has already paid once ([[manifest-reanchored-mixed-authority]]).

### What moved, and what it cost each side

| | before | after |
|---|---|---|
| scenario fixtures (26) | `tests/authoring/scenarios/` | `rust/data/scenarios/` |
| goldens (21) | `tests/regression/golden/` | `rust/data/golden/` |
| Rust readers | 2 `CARGO_MANIFEST_DIR` climbs out of `rust/` | 2 climbs that stay inside it |
| the two manifest writers | `root/tests/regression/golden` | `root/rust/data/golden` |
| Godot | `res://../tests/authoring/scenarios/` ×2 | `res://../rust/data/scenarios/` ×2 |
| Python | 41 modules spelling their own path | `config.paths.{GOLDEN_DIR,SCENARIO_DIR}` |

⚠ The Godot `.gd` constants are worth naming because they are the easiest thing in this repo
to move and not notice: they are plain strings resolved at runtime by the editor, no compiler
sees them, and the tests that exercise them are `skipif`-ed on CI
([[godot-project-godot-skip-worktree]] is the neighbouring trap on the same directory).

### Verification

`cargo test` / `cargo clippy --all-targets` clean; `uv run pytest -n 12` → **2,448 passed,
5 skipped**; `ruff` clean. The three manifests are byte-identical — the golden hashes they
record are of file **content**, and the writers now read the same bytes from a different
directory. All 47 moved files are git-recorded 100 %-similarity renames.

### Deliberately NOT in this half

* **Re-anchoring `forcing/weather_fixture`** — still S1's named successor, still not taken.
* **S2's Rust-side comparators.** FINDING 3 stands: no Rust test compares a run to a committed
  golden. S1 only guarantees that when S2 writes them, the path they read is the final one.
  ⚠ **Taken 2026-08-19, §5t** — and S1's guarantee held: the comparators read
  `rust/data/golden/` through one helper, `domains::goldens::golden_dir`, so the two crates
  share a single spelling of the path rather than each climbing out on their own.
* **Deleting `tests/authoring/` and `tests/regression/`** as concepts — the directories are
  gone because they are empty, not because anything was retired.
* ⚠⚠ **Giving the new forcing literal a home that outlives Python.** It lives in
  `tests/crossport/test_golden_provenance.py`, which §5q's own table retires — so S1 closed a
  prose-rot hole with a gate that dies with the tree, and **added a fourth entry to
  FINDING 2's list in the act of fixing something else**. Recorded there as well as here.
  **S2 must carry it**, not merely delete it: two counted literals against
  `rust/data/golden/`, in Rust, is the smallest successor that survives S6.

## §5t Stage 3 — S2's first half: the reference compares its own runs, COMPLETE 2026-08-19

FINDING 3 in one sentence: *"No Rust test compares a run against a committed golden."* This
half discharges it. The manifest byte gate and the inventory dumps' consumer — the other two
of FINDING 2's list — are **not** in this half; see "Deliberately NOT in this half".

### The structural fact that had kept the comparison in Python

An `examples/` program is a **binary target**: no integration test can call into it. So the
19 runs the reference authors were unreachable from `cargo test` by construction, and
shelling out to `cargo run` — which is what `test_golden_provenance.py` does — was the only
way to reach them. That is *why* the comparison ended up on the checker's side, and it means
S2 does not begin by writing a test:

* the emitter bodies moved into the libraries, `domains::goldens` (11 goldens) and
  `station::goldens` (8), each an ordinary `pub fn () -> String`;
* the 17 `emit_*` examples became one-line wrappers that `print!` the same value, keeping
  their module docs;
* `domains/tests/golden_regression.rs` and `station/tests/golden_regression.rs` compare.

⚠ **`station` is the lowest crate that can see all nineteen**, because it depends on
`domains` and not the reverse — so the whole-census gates live there. A new workspace member
was considered and refused: unlike the three freeze manifests, which genuinely span
`domains` + `station` + `authoring`, the goldens stop at `station`. Same rule S1 used to put
the data in `rust/data/` — *put the thing where the dependency is* — reaching a different
answer because the dependency is shaped differently.

### ⚠⚠ The platform policy: `cargo test` runs on Linux, and a skip was the wrong port

`golden_platform.py`'s rule is that a hex-float golden is byte-exact only within a build on
its generation platform (Windows/UCRT here), so the transcendental ones carry
`windows_golden_only` — a pytest **skip**. The Rust job runs on `ubuntu-latest`, so this had
to be settled before the first line of code; the obvious translation, `#[cfg(windows)]`,
compiles the gate out, which is the shape this repo has been bitten by twice.

So the port is **classification, not exclusion**. `Numerics::PureArithmetic` goldens are
byte-compared on every platform; `Numerics::Transcendental` ones are byte-compared on the
generation platform and **structurally** compared everywhere else — identical JSON tree,
identical key order (the goldens are `sort_keys=True`), identical array lengths, identical
non-float leaves, every hex-float leaf finite on both sides. Exact, and **not a tolerance**.
That is strictly more than Python does off-Windows, where the test simply does not run.

⚠ **A band was deliberately not invented**, and this is the same refusal `golden_platform.py`
records at C3: *"writing a band nobody measured is the derived-not-measured move this
contract exists to refuse."* The classification is inherited from where
`@windows_golden_only` actually sits in `tests/test_regression_*.py` — a measurement, not a
fresh judgement — and two tests pin the four pure-arithmetic names so a reclassification
(which *weakens* a gate off Windows) is a visible edit rather than a one-word roster change.

### ⚠⚠ FINDING 2 gains a FIFTH entry: `tiers.json` has no Rust reader

Looking for a measured band turned one up. `tests/crossport/tiers.json` — the file
`docs/native-port-reference.md` calls the cross-port **tolerance contract**, carrying the
3 tiers and the measured bands for 20 of the goldens — is read by **no program in `rust/`**.
Grepped: the only occurrence of the name in the whole Rust tree is a doc-comment pointer in
`domains/src/lib.rs`. So the tolerance contract is a contract artifact stranded in the tree
S6 deletes, exactly like the four gates FINDING 2 already lists, and it was found only
because this slice went looking for a number it then declined to use.

⚠ It is **not** a gap this slice fills. Porting `tiers.json` means porting the comparator
and the tier machinery with it, which is a different subject from "does this run still
produce these bytes". Recorded so S6 cannot reach it as a free deletion — the same standing
`R!` has in §5q's table.

### The expensive golden — a decision, not a default

`sealed_station_state.json` is ~1.3 M sub-steps over five domains and costs **~100 s at every
optimization level**: measured 378 s at the stock dev profile, 116 s at `[profile.dev]
opt-level = 2`, 93 s in release. The cost is the run, not the build, so no build knob buys it
back. Against a warm `cargo test` of **7.9 s**, including it unconditionally is a 15×
regression on the reference's primary gate.

**Put to the user, who chose: off by default, on in CI.** It is `#[ignore]`d and the `rust`
CI job gained a `cargo test -- --ignored` step. ⚠ The `opt-level = 2` profile change was
measured (and found byte-neutral across all 19 goldens — a *third* profile beyond the
release/debug pair `regen_goldens_from_rust.py` already records) and then **reverted**,
because it belonged to an option the user did not pick. The measurement is kept here rather
than the change.

⚠⚠ **`#[ignore]` alone is the green-by-skip shape, so it does not stand alone.** `Cost` is a
roster field, and `the_ignored_set_is_exactly_the_expensive_roster` asserts both directions:
exactly one golden is `Expensive`, and exactly one `#[ignore]` attribute exists in the file.
A second `#[ignore]` added for convenience is red. What no test can guard is the CI step
itself — deleting that line fails nothing — so the step carries the warning in a comment.

### ⚠ The Linux path would have shipped unexercised

`compare_structural` is unreachable on the development box: `compare` routes to it only for
a transcendental golden **off** Windows. So the entire Linux branch was dead code locally,
and would first have executed on CI on the day something diverged, with nobody having ever
seen it work — the green-by-skip shape wearing different clothes. Eleven unit tests now
exercise the comparator directly on hand-built pairs, on every platform: a last-bit
perturbation passes, and a changed integer, label, stock id, key count, array length, key
*order*, and a float leaf that turned into a label each fail by name.

⚠ One of them asserts a **limitation** rather than a capability —
`a_wildly_different_hex_float_is_still_structurally_equal`. The structural comparison says
nothing about magnitude, which is precisely why it is the off-platform fallback and not the
contract, and why `tiers.json`'s absence above is a finding rather than a shrug.

### The controls

1. **Byte-neutrality of the relocation.** All 19 emitters' stdout was captured *before* the
   move and diffed against the committed goldens; 18 came back identical (the 19th, the
   sealed station, is covered by its own test rather than a second six-minute run). "One code
   path, two callers" is the claim; the diff is the control.
2. **The comparison is live.** `thermal()` was mutated to run one fewer step → the gate went
   red naming the exact divergence (`"n": 719` vs `720`). Reverted **in place from a copy in
   the temp tree, never `git checkout`** — this tree carried uncommitted work, and discarding
   an uncommitted slice that way is a cost this flip has already paid once.
3. **The census is live.** A 20th roster entry was added → five of the seven station gates
   went red, each from a different angle (the count literal, the classification partition,
   the uniqueness/reality check, and the run itself).
4. ⚠ **The control that caught its own author.** The first draft of
   `the_ignored_set_is_exactly_the_expensive_roster` counted the bare string `#[ignore` and
   found **12** — that file's own prose discusses the attribute eleven times. That is
   `manifest_writer.rs`'s recorded lesson landing again in a new place: *an anchor that
   matches prose as well as syntax checks whichever came first.* The count is now
   line-anchored, with a paired assertion that the bare string really is ambiguous here, so
   the reason is a measurement rather than a claim.

### Deliberately NOT in this half

* **The manifest byte gate** (FINDING 2's first entry) and **`test_inventory_parity.py`'s
  successor** (its third). Both need the manifest builders out of the three
  `dump_*_inventory` examples and into their crates, for the same binary-target reason as
  above — and unlike the goldens they genuinely span `domains` + `station` + `authoring`, so
  the "which crate owns it" question has a different answer. S2's second half.
* **Retiring anything on the Python side.** S2 builds successors; S6 retires originals, and
  only once S3–S5 have theirs. The two overlap on purpose: a slice that deletes its
  predecessor before the successor has run *in CI* is how a gate goes missing.
* **`golden_platform.py`'s other two policies.** `write_python_golden` (refusing Python
  authorship) and `PYTHON_DIVERGES` / `DISAGREEMENT_CEILING` (the checker's conformance)
  have no subject on the Rust side — they measure *Python*, so they die with it at S6 rather
  than porting here. Named so the omission is a decision and not an oversight.

## §5u Stage 3 — S2's second half: the contract gates come with the contracts, COMPLETE 2026-08-19

FINDING 2's first and third entries. §5t moved the *runs* out of the `examples/` binaries;
this moves the **manifest writers** for the same structural reason, and gives all three
freeze contracts a byte gate that survives S6.

### The move

`manifest()` / `dumps()` / `census_json()` / `authority_json()` / `file_sha256()` /
`repo_root()` and everything around them left `examples/dump_{biosphere,station,authoring}_inventory.rs`
for `crates/{domains,station,authoring}/src/freeze_manifest.rs`. The examples keep their
argument parsing and nothing else. Deliberately a **relocation, not a rewrite** — the
emitted bytes must not move, and the way to guarantee that is to not retype the code.

⚠ **Each crate gates its own contract**, which is why no new workspace member was needed
here either. The Python original parametrized one file over three writers because it shelled
out; in Rust the natural owner of the biosphere contract's gate is the crate that writes it.

### ⚠⚠ The finding: a mechanical rewrite reached into frozen contract prose

The move rewrote `domains::` → `crate::` for self-references. It hit **three** manifests'
`_authority` text, where the path is written from *outside* the crate as documentation:

| contract | the prose that moved |
|---|---|
| biosphere | `domains::biosphere::params::param_files` → `crate::…` |
| station | `station::params::param_files` → `crate::…` |
| authoring | `see authoring::surface` → `see crate::surface` |

**Three of the four freeze contracts silently re-worded by a refactor**, and each is a
`why` string inside the frozen `_authority` block — i.e. contract text, not code. ⚠ The
striking part is *what caught it*: **the byte gate being built in this very slice**, on its
first run, before it had a single test written around it. Nothing else in the repo would
have — the Python byte gate would have caught it too, but only if run, and this is exactly
the gate S6 was scheduled to delete.

⚠ The fix was three targeted restorations, **not** a narrower blanket rule. A rewrite that
knows the difference between code and prose is not available; a gate that compares the whole
artifact is. That is the argument for the byte gate in one incident.

### The residue: `test_inventory_parity.py` is NOT subsumed, measured rather than assumed

`test_manifest_writer.py`'s own docstring claims it catches *"the same staleness
`test_inventory_parity` catches, now for every key rather than the compared axes."* Read as
subsumption that is **wrong for three of the seven tests**, and the enumeration is the point
— the alternative was inheriting a scope claim from a docstring, which is the
[[multirate-crossport-anchor-partition-parity]] shape (*a scope decision recorded as a FACT
outlives its reasoning*).

| test | verdict |
|---|---|
| `test_rust_inventory_equals_the_frozen_manifest` | **subsumed** — the sets are in the file the byte gate compares |
| `test_the_frozen_biosphere_manifest_is_not_stale` | **subsumed** |
| `test_the_frozen_station_manifest_is_not_stale` | **subsumed** |
| `test_the_frozen_authoring_manifest_is_not_stale` | **subsumed** |
| `test_the_locked_dt_matches_the_reference_tree` | ⚠ **residue** — the byte gate regenerates from the same literal, so it agrees with itself |
| `test_the_weather_hash_matches_the_reference_tree` | ⚠ **residue** — `weather_sha256` is emitted for checking and **never spliced**, so the gate copies rather than derives it |
| `test_the_station_aux_axis_is_empty_by_delegation` | ⚠ **residue** — `[] == []` is inert, and since C7 the empty list is *written into* the contract |

All three residue claims now have Rust successors, each in the crate that owns the contract.
The `dt` one is the sharpest illustration of why the byte gate is not enough: the source-text
grep says the literal is *typed*, the byte gate says the file is *consistent*, and neither
says the typed number is still **true**. Only a comparison against `BIO_DT` does.

⚠ **`test_the_writer_refuses_an_unknown_argument` gets no successor, as a decision.** The
Python gate passed `--write-manifest <path>` to a subprocess, so its argument handling had to
be asserted or the byte comparison could pass while proving the wrong thing. The Rust gate
calls `manifest_text()` directly; the CLI is no longer load-bearing *for the gate*, only for
the regeneration command a human runs. Porting the check would gate a path nothing depends
on — recorded rather than done, since the Python original stays alive until S6 anyway.

### ⚠ Reading `docs/` is not an S1 regression

The byte gate `include_str!`s `docs/*-reference.manifest.json`, four directories up and out
of `rust/`. Stated explicitly because it looks like re-opening what S1 closed and is not:
S1's rule is that the reference must not reach into **the tree being deleted** (`src/`,
`tests/`). `docs/` is where the freeze *contracts* live, it outlives the checker, and the
writer's own `repo_root()` already made this exact climb to resolve `--write-manifest`'s
default. A manifest gate that could not see the committed manifest would have no subject.

⚠ It does introduce one new failure mode the Python original could not have — an
`include_str!` of a wrong-but-present path is silent, where a runtime read would have thrown.
`the_committed_manifest_is_actually_loaded` is the paired control: size plus the presence of
an `_authority` block.

### `authoring` gets a byte gate and no source-text half

New file, and the asymmetry is inherited rather than invented: C7 measured the authoring
contract for the anti-derived-literal trap its two siblings carry and found **none** — its
hand-authored keys are a phase number, two repo paths and two blocks of prose, and the crate
owns no constant any of them could be spliced from. *A control with no test to redden is the
finding, not a gap to fill.*

### The controls

1. **Byte-neutrality of the move** — all three manifests regenerated and compared to the
   committed files. Two rounds: the first found the prose rewrite above, the second came back
   identical on all three.
2. **The re-pointed source anchors actually redden.** `manifest_writer.rs`'s `include_str!`
   moved from the example to the module. A stale path could have found *zero* lines and
   passed; pointed at `src/lib.rs` deliberately, all three greps go red. Verified rather than
   argued, because "it would fail" is exactly the claim this slice caught itself getting
   wrong once already.
3. **A hand edit to a committed contract is red.** `"phase": 9` → `10` in the authoring
   manifest → the byte gate fires and names both sides.

### Also in this half: two review corrections to §5t

* ⚠ **A claim that shipped false.** §5t's comment on the `#[ignore]` control said *"nothing
  inside the suite can guard this line"* about the CI step that runs the expensive golden.
  That is false by this repo's own idiom three files away — `manifest_writer.rs` greps the
  writer's source, `science_gates` greps a file for a recorded bound. `ci_still_runs_the_ignored_tests`
  now pins the step textually, with a control that the match is a `run:` command and not the
  explanatory comment above it. The workflow was also parsed to confirm it is valid YAML: a
  malformed workflow does not fail, it silently does not run.
* The `#[ignore]` census read only the station test file, so a skip added on the `domains`
  side was invisible to the control written to make skipping visible. It now reads both.

### ⚠⚠ Standing work S2 leaves behind — three items, none of them blocking

Written down because each is an *interaction* rather than a defect in any single decision,
and interactions are what a slice's own record usually misses.

**1. The sealed station's byte-exactness is checked by nothing automatic.** The two
decisions are each right and their combination has a hole:

| where | what happens |
|---|---|
| Windows (the generation platform — the only place its bytes mean anything) | `#[ignore]`d, so `cargo test` skips it |
| Linux CI (`cargo test -- --ignored`) | runs, but `compare` routes a transcendental golden off-platform to the **structural** branch, which passes |

So the byte compare for the largest assembly in the repo happens only when a human types
`cargo test -- --ignored` on Windows. ⚠ And it is a step **down** from the Python original:
`test_rust_reproduces_the_committed_golden_bytes` is `slow`, which in this repo is *opt-out*
(it runs by default locally), while `#[ignore]` is *opt-in*. **S6 must not retire the Python
byte census believing this is a like-for-like successor.** The two remedies, neither taken:
un-ignore and pay ~100 s on every `cargo test`, or add a Windows CI runner. Structural
equality *is* checked automatically, and `tiers.json`'s Tier-2 band still gates the scenario
next door — but neither is the byte compare.

**2. The byte gate now depends on `.gitattributes`.** `include_str!` does not normalize line
endings; `dumps` always emits LF. A checkout producing CRLF manifests would redden the gate
with the **wrong diagnosis** — its message says "the manifest was edited by hand". Checked
rather than assumed: `.gitattributes` pins `* text=auto eol=lf`, `git check-attr` confirms
`eol: lf` on the manifests, and all three carry zero CR bytes. The dependency is real and
newly created by this slice (the Python original read at runtime), so it is written down in
each gate's doc comment rather than left to be rediscovered.

**3. `repo_root()` is `pub` on three crate libraries** purely so the examples can compute a
default `--write-manifest` path. `pub(crate)` plus the example spelling its own default would
keep the new public surface at zero. Cosmetic; noted so it is a choice rather than drift.

### ⚠ And one correction to this section's own first draft

`the_committed_manifest_is_actually_loaded` shipped with prose claiming it guarded *"an
`include_str!` of a wrong-but-present path"* — which the byte gate already catches loudly —
and it discriminated on `"_authority"`, **a string all three manifests carry**, so it could
not tell the contracts apart at all. Renamed to
`the_committed_manifest_is_this_contract_and_is_not_truncated`, given an assertion that
actually discriminates (each manifest names its own `docs/*-reference.md`), and its doc
narrowed to what it owns.

⚠ That is the *second* claim this slice had to retract for the same reason — after "nothing
inside the suite can guard this line". Both were **doc comments asserting a property nobody
tested**, in a slice whose entire subject is gates that assert what they claim. Recorded as a
pattern rather than twice as an incident: in this repo a `///` block is read as a finding, so
writing one costs the same care as writing the assertion under it.

## §5v Stage 3 — S3, the four sibling domains: designed and gated, measured 2026-08-19 BEFORE any Rust was written

S1 gave the reference its own ground and S2 gave it its own contract gates. S3 is the first
slice whose subject is **science**: `domains/src/{crew,eclss,power,thermal}.rs` carry 1,411
lines the reference calls canonical and **0 `#[test]` of their own**, against 160 collected
cases in the tree being deleted. This section is the design and, in the §5d / §5h / §5j
pattern, the measurements that chose it — all taken against a clean tree with nothing else
in flight, before a line of Rust was written.

### The count S3 is measured against, and it is 160, not 158

`pytest --collect-only -q` over the nine files: **160 collected ids from 158 `def`s.**

| File | ids |
|---|---:|
| `test_power_flows.py` | 29 |
| `test_eclss_flows.py` | 23 |
| `test_thermal_flows.py` | 22 |
| `test_crew_flows.py` | 18 |
| `test_eclss_run.py` | 16 |
| `test_thermal_run.py` | 15 |
| `test_power_run.py` | 14 |
| `test_crew_run.py` | 12 |
| `test_power_self_discharge.py` | 11 |
| **total** | **160** |

⚠ FINDING 10 put 725 of the suite's 2,452 on the parametrize axis, so the axis was measured
here rather than assumed. **It is two cases wide.** The only parametrized `def` in the nine
is `test_eclss_two_runs_contract_geometrically`, which fans to three. Recorded because the
*expectation* was a much larger gap: on these nine files "one `#[test]` per `def`" happens
to be nearly right, and it is right by measurement, not by luck holding elsewhere.

### ⚠ Two of FINDING 4's eleven files are NOT in S3, and that is a rule, not a size call

The classification pass listed eleven files under FINDING 4. Two of them leave:

* **`test_bvad_validation.py` (8).** Its reference owner is `station/src/science_gates.rs`
  — a different crate — whose own doc comment names `tests/test_bvad_validation.py` as
  *"the checker's conformance half"*. Porting it is not a translation; it is re-deciding
  what that sentence means once there is no checker. That is a science decision.
* **`test_crew_coupled_loop.py` (8, 672 lines).** Its subject is the sealed station's
  carbon budget at scale — the `crew-coupled-loop-refused` / `chamber-scale-diagnosed`
  findings. Those tests *encode an argument*, not coverage of `crew.rs`.

`CLAUDE.md` settles it in one line: **do not take a science item and a re-anchoring slice in
one batch.** Both are science items in `station`, so both are named here as deferred with
their destination, not folded in. **S3 proper is nine files / 160 cases.**

### Gating measurement 1 — the baseline

`cargo test --workspace`: **488 passed, 0 failed, 3 ignored**, ~87 s incremental. (§5q
recorded 445; S1 and S2 added the rest. The three `#[ignore]`s are S2's expensive goldens.)

### ⚠⚠ Gating measurement 2 — FINDING 7's control no longer measures what it measured

§5q's **Control B** — swap the two legs of `charge_split` in `power.rs` so the battery
stores `(1−η)` and loses `η` — was re-run. §5q recorded **2 of 445 red, both in
`godot_bridge`**. Today it is **12 of 488**, and one of them is in `domains`:

| Reddened | Crate | What kind of gate |
|---|---|---|
| `every_domains_golden_is_still_this_reference_s_output` | `domains` | **byte snapshot** (S2) |
| `every_cheap_station_golden_is_still_this_reference_s_output` | `station` | byte snapshot (S2) |
| `station_composition_matches_build_station_bit_exact` | `station` | composition parity |
| `self_discharge_composition_matches_standalone_build_power_bit_exact` | `station` | composition parity |
| `brownout_graceful_cools_node_without_rationing` | `station` | perturbation |
| `radiator_failure_heats_node_conserving_no_rationing` | `station` | perturbation |
| `radiator_failure_outside_window_is_baseline` | `station` | perturbation |
| `three_part_leaky_station_steps_well_fed_with_a_bounded_node` | `station` | perturbation |
| `palette::tests::station_carries_thermal_and_battery_context` | `station` | front-end context |
| `science_gates::gate_tests::tier1_node_is_period_1_fixed_point` | `station` | science gate |
| `tests::composed_energy_station_builds_steps_and_carries_readouts` | `godot_bridge` | readout |
| `tests::observation_projection_carries_the_derived_readouts` | `godot_bridge` | readout |

**FINDING 7's literal claim is now false and its substance is intact**, and the difference
matters enough to state both halves:

* *False:* "nothing in `domains` or `station` noticed." Eleven things in `station` notice,
  and one thing in `domains` does. Slices C4b, S1 and S2 built those gates after §5q took
  its reading, and nobody re-read the finding afterwards. **A control's verdict is dated to
  the tree it ran against** — the same shape as `canopy-regulator-diagnosed`, where a
  recorded blocker was false the day it was written.
* *Intact:* **not one of the twelve is a test of power science.** The `domains` entry is a
  golden byte compare, which reports "the bytes moved" and cannot say which coefficient is
  wrong; the rest are composition, parity, perturbation and front-end readouts that happen
  to depend on a battery number. Zero tests *named after* the thing that broke. So S3's
  justification stands, restated accurately: the reference has **no behavioural gate on the
  sibling-domain science**, and the cross-port comparison remains the only gate that fails
  *by name*.

⚠ The plan's §5q FINDING 7 text is left as written — it is a dated record — and this
section is its correction. Do not quote the "2 of 445" figure forward.

### ⚠⚠ Gating measurement 3 — the loader bound checks are inert, and this was not known

`params.rs::bounds_match_the_loaders` reads as the gate on the five param files' bound
wiring. It is not. Mutation **M-bound**: delete the `require_half_open(v[0], 0.0, 1.0, …)`
wrapper from `params::charge()` so the efficiency is taken raw.

**Result: 488 passed, 0 failed — bit-identical to the baseline.**

The reason is structural, and it is the "inert by construction" shape this log keeps
recording: `bounds_match_the_loaders` asserts that the *committed values* lie inside their
ranges. Deleting the check does not move a committed value, so the assertion cannot notice.
It can only fail if someone edits a YAML to a bad number — the one case where the loader's
own guard would have fired anyway. ⚠ It is also incomplete on the roster: it covers charge,
thermal and crew and **says nothing about `eclss.yaml`'s rate and setpoint bounds at all.**

This decides how the 23 loader tests port (below). It is not a defect S3 introduces; it is
one S3 is the first slice positioned to see.

### The three infrastructure gaps, and each one's design

**Gap 1 — there is no trajectory, and no second integrator. This blocks 68 of the 160.**

`domains::run` returns `(final_state, rationed, events)` and takes `&EulerIntegrator` by
concrete type. Every `*_run.py` test is trajectory-shaped (conserved *every step*, sinks
monotonic, SOC returns each day, two runs contract geometrically, difference-is-constant)
and several run **both** integrators — unlike the biosphere, the four siblings support RK4.

Measured before choosing: `Scheme` (integrator.rs:287) is **private**, and `step_report` is
an inherent method on each concrete integrator (:342, :398), so no public trait carries it.
The public `Substepper::substep` is not a substitute — by its own doc it keeps `n`, skips
aux, and **does not assert conservation**, which is the very thing these tests check.

*Decision:* a **`domains`-local trait** with impls for the two integrators, plus an additive
`run_trajectory` returning every state. Making `Scheme` public would reach into frozen
`simcore` to serve test ergonomics — unfreeze-adjacent, and not what S3 is for.

⚠ **Additive, not a change to `run`.** `goldens.rs` calls `run`, and `goldens.rs` produces
the frozen golden bytes. The predicted golden diff is **zero**, and S2's byte gate is what
proves it rather than the plan asserting it (`soil-layers-built`: predict the diff before
regenerating).

⚠ The 69 / 68 / 23 split (flow-level / run-level / loader) was checked rather than assumed
on the case that could most easily have been mis-filed: the ten `*_is_dt_linear` cases read
as run-shaped from their names, but `test_solar_charge_is_dt_linear` **calls `evaluate`
twice at two `dt` values and compares legs** — no registry, no step. They are flow-level,
and Gap 1 blocks 68, not more.

**Gap 2 — the flow-level helpers are private, so the tests must be in-src.**

`charge_split`, `scrub_flux`, `condense_flux`, `makeup_flux` and `radiated_power` are
private; only `thermal::temperature` is `pub`. About 40 of the 69 flow-level cases target
these directly. `crates/domains/tests/*.rs` cannot see them, and widening them to `pub` to
suit a test file is the tail wagging the dog.

*Decision:* flow-level cases go in `#[cfg(test)] mod tests` inside each domain file (the
`biosphere/science.rs` precedent); run-level cases go in `crates/domains/tests/{power,
eclss,thermal,crew}_run.rs` (the `season_order_independence.rs` precedent).

**Gap 3 — the 23 loader-rejection cases need a mechanism, not a translation.**

Python writes a malformed YAML to `tmp_path` and asserts the loader raises. Rust's
`params.rs` `include_str!`s five files at compile time and *panics*; there is no runtime
path to hand it a bad one. A mechanical port lands inert — which measurement 3 shows is
already the failure mode here.

Measured first, as required: `config` already owns the **generic** mechanism —
`a_wrong_declared_unit_is_rejected`, `a_non_numeric_value_is_rejected`,
`an_unwired_extra_param_is_rejected_not_ignored`, `every_bound_rejects_nan`,
`bounds_reject_what_the_loaders_reject`. What is unowned is the **per-file wiring**: that
`charge.yaml`'s efficiency is guarded half-open on (0,1], that `radiator.yaml`'s area is
required positive, that `eclss.yaml` is guarded at all.

*Decision:* give `params.rs` a testable inner function over `&str` per file, so each
rejection is exercised against a deliberately-bad text. **Every one carries its own control
proving it can go red** — and M-bound is the standing example of what happens without one.

### The oracle classification — and the one category that is forbidden

FINDING 9 already found seven Rust tests naming the dying side as their oracle. S3 is where
that could happen 160 more times, so every case is tagged before it is written:

* **property** (conserved, monotone, dt-linear, order-independent, balanced, noop-at-zero)
  — port freely; the law is the oracle.
* **closed form** (`test_crew_depletion_matches_closed_form`,
  `test_balanced_load_matches_daily_stored_solar`, `test_eclss_converges_to_the_steady_states`,
  `test_thermal_equilibrium_balances_radiation_against_load`,
  `test_radiated_power_matches_stefan_boltzmann`) — **re-derive the expression in the Rust
  test.** Do not transcribe the Python number.
* **snapshot** — already owned by `goldens.rs`; do not duplicate.
* **a value read off a Python run** — ⚠⚠ **forbidden.** Python has no reference authority,
  and inheriting a Python-produced constant re-creates that authority silently at the exact
  moment Python is being deleted.

⚠ `test_stefan_boltzmann_constant_is_codata_value` is the good case and the template: its
oracle is CODATA, and it should cite CODATA in Rust too.

⚠ And a trap `season_order_independence.rs` already paid for: **order-independence at real
physical magnitudes can be inert** — deleting the registry sort left it green, because at
comparable magnitudes re-associating the additions moves no bits. The four
`*_registration_order_independent` ports must assert the rebuilt registry's **iteration
order**, not only the final state, and must be controlled against exactly that mutation.

### The acceptance criterion: a five-mutation battery, pre-committed

S3 is not done when 160 tests exist. It is done when each of these reddens a test **in
`domains`, by name** — a test that says what broke, not a byte compare that says bytes moved:

| # | Mutation | Domain |
|---|---|---|
| M-power | swap `charge_split`'s legs (§5q's Control B) | power |
| M-eclss | flip `makeup_flux`'s sign so the regulator drives away from setpoint | eclss |
| M-thermal | change `t.powf(4.0)` to `t.powf(3.0)` **in `radiated_power`** (`powf(4.0)` occurs three times in `thermal.rs`; naming the site is what makes the control reproducible) | thermal |
| M-crew | swap the carbon split fractions | crew |
| M-bound | drop a per-file bound guard in `params.rs` | (all four) |

Each is run against a clean tree with nothing else in flight, and each is restored with
`git checkout --` and the workspace re-verified — §5p's third process trap, observed.

⚠ **M-bound is the cleanest of the five, and for a reason worth naming.** The other four
move a number, so a golden byte compare catches them whatever else exists — which is how
measurement 2's misleading "`domains` noticed" arose. M-bound moves **no committed value**,
so it has *no snapshot backstop at all*: measurement 3 found it green across all 488. It is
the one mutation whose verdict is a statement purely about behavioural coverage.

### S3's exit gate, stated forward to S6

S6 deletes `tests/crossport/test_crossport.py`. FINDING 7 says that deletion is not free
while it is the sibling domains' only by-name gate. So S3's exit criterion is written now,
in S6's terms, and not left to be inferred:

> Re-run the five mutations with the cross-port comparison **and `--test golden_regression`**
> both deselected. `domains` must still redden, by name, for all five.

⚠⚠ **The golden exclusion is not belt-and-braces; without it this gate is inert, and it was
inert in this section's own first draft.** Measurement 2 shows M-power already reddens
`domains` today — via `every_domains_golden_is_still_this_reference_s_output`, with zero
sibling tests written. So "the cross-port comparison is deselected and `domains` goes red"
is **already satisfied for power before S3 begins**, which makes it a gate that cannot
distinguish a finished S3 from an unstarted one. That is precisely the defect measurement 3
diagnoses in `bounds_match_the_loaders`, reproduced one section later in a section about it.
Caught on review, not by a test — the §5u pattern, again.

The discriminator was already in this section's prose (*"a test that says what broke, not a
byte compare that says bytes moved"*) and simply was not in the gate. It is now.

⚠ Deducible from measurement 2 rather than separately run: with the goldens excluded,
`domains` today has **nothing** that reddens under any of the five, because it has no
sibling-domain test at all. The tightened gate therefore reads zero before S3 and must read
five after it.

⚠ This is S2's lesson applied one slice earlier — *"S6 must not retire the Python byte
census believing this is like-for-like"* — and the reason it is stated here is that S6 will
otherwise inherit an **unmeasured** claim.

### Deliberately NOT in S3

* **`test_bvad_validation.py` and `test_crew_coupled_loop.py`** — deferred as above, with
  `station` as their destination. Named, not bucketed.
* **No new science.** Every ported case must be satisfiable by the code as it stands. A case
  that fails is a **finding to report**, never grounds for editing `domains` to suit it.
* **No golden regeneration.** The predicted diff is zero; if it is not, that is the finding.
* **No Python deletion.** The nine files stay green and running until S6.

## §5w Stage 3 — S3 BUILT, COMPLETE 2026-08-19: 160 tests, five mutations, and a control that was itself inert

§5v designed S3 and took three measurements before a line of Rust was written. This section
is what happened when it was built. The design held; two things did not, and both were found
by a control rather than by the suite.

### What landed, against §5v's own count

| | planned | built |
|---|---:|---:|
| flow-level (in-src `#[cfg(test)] mod tests`) | 69 | 69 |
| loader rejection (`params.rs`) | 23 | 23 |
| run-level (`crates/domains/tests/*_run.rs`) | 68 | 68 |
| **total** | **160** | **160** |

Case for case against the nine Python files, which still collect 160 and still pass. The
workspace went 488 → **648**; `clippy --all-targets -D warnings` is clean.

The one parametrized Python case (`test_eclss_two_runs_contract_geometrically`, three
species) is written out as three named tests, so a failure says *which control loop* broke
rather than which parameter id did.

### The three gaps, as built

**Gap 1.** A `domains`-local `StepIntegrator` trait with impls for both integrators, plus an
additive `run_trajectory` returning `(Vec<State>, rationed, events)` with the initial state
included. `run` is untouched. **The predicted golden diff was zero and it was zero** —
`--test golden_regression` is what says so, per `soil-layers-built`'s rule.

**Gap 2.** Flow-level cases in-src; run-level in `tests/{power_run, power_self_discharge,
eclss_run, thermal_run, crew_run}.rs`. Small helpers are duplicated across the five files
rather than pulled into a `tests/common/mod.rs` — `golden_regression.rs` states that choice
explicitly for three shared lines and this follows it.

**Gap 3.** Five `*_from(&str)` loaders returning `Result<_, ConfigError>`, with the public
entry points as thin panicking wrappers.

⚠⚠ **The bound guards had to move INSIDE the `*_from` functions, and that is the whole
point of the gap.** Had `charge_from` carried only the parse and the unit guard while
`charge()` kept `require_half_open`, the 23 new rejection tests would have exercised a path
the guards are not on — and M-bound would have stayed green, reproducing §5v measurement 3's
defect in the section written to close it. Flagged on review before the code was written,
then verified by running M-bound against the new tests specifically rather than against the
crate.

### The exit gate: FIVE, measured

Per §5v as corrected here (the section said "four" in three places while its table listed
five — M-bound had been added after the wording, and the gate one commits against must not
be off by the one mutation with no snapshot backstop).

Selection: `cargo test -p domains --lib --test power_run --test power_self_discharge --test
eclss_run --test thermal_run --test crew_run --no-fail-fast`. That is exactly "the cross-port
comparison and `--test golden_regression` both deselected".

| Mutation | Site | red, by name |
|---|---|---:|
| M-power | `charge_split` returns `(lost, stored)` | 13 |
| M-eclss | `makeup_flux` computes `(cabin_o2 − setpoint)` | 7 |
| M-thermal | `t.powf(3.0)` in `radiated_power` | 7 |
| M-crew | `carbon_split` returns `(feces, respired)` | 4 |
| M-bound | `charge_from` drops `require_half_open` | 2 |

Before S3 this gate read **zero** — not separately re-run, but read off measurement 2's own
table with the golden row removed, since the golden byte compare was the only `domains`
entry it found. Each mutation ran against a clean tree with nothing else in flight and was
restored from a backup, with the selection re-verified green (239 passed) afterwards.

⚠ **Two of the five are caught only outside the run suites, and both facts are findings.**

* **M-crew reddens nothing at run level.** Carbon is conserved whichever way the split goes,
  and `crew_run.rs` never compares `exhaled_co2` against `fecal_waste` — it checks each sink
  is monotone and that the three totals are invariant, all of which survive the swap. A
  12-case run suite is blind to the two fractions its domain exists to apply.
* **M-bound has no backstop anywhere in the workspace.** Measured rather than argued: under
  M-bound the full 648-test workspace reddens exactly `charge_loader_rejects_zero_efficiency`
  and `charge_loader_rejects_above_one_efficiency` and nothing else. Its verdict is therefore
  a statement purely about behavioural coverage, which is why §5v called it the cleanest of
  the five.

### ⚠⚠ FINDING: the registration-order control found MY OWN PROBE inert, not the subject

`season_order_independence.rs` had already established that a run comparison can be inert at
real magnitudes, so all four sibling order tests asserted the rebuilt registry's **iteration
order** from the first draft — §5v required it. The control was run anyway: delete
`flows.sort_by(...)` in `Registry::new`, expect all four to redden.

**Three reddened — crew, eclss, thermal. Power stayed green.**

The probe was `into_parts()` then `reverse()`. `build_power` constructs
`[SolarCharge, LoadDraw]`; canonical order is `[load_draw, solar_charge]`. So with the sort
deleted, `into_parts()` hands back *build* order and reversing it lands on canonical order —
the "permuted" registry iterated canonically anyway, and both assertions passed against a
registry that never sorted.

This is the season lesson's twin with the roles exchanged. There the **subject** was not a
discriminator (real magnitudes hide re-association). Here the **probe** was not: one
hand-picked permutation is a coin flip on whether it moves anything, and it lost the toss on
exactly one of four registries. A discriminating probe on three subjects is not evidence
about the fourth.

*Fix:* all four tests enumerate `n!` in full — 2, 6, 6 and 24 flows-permutations
respectively, which is cheap at this size and cannot miss — plus assertions that the family
is the full factorial and is not the identity alone. Re-run under the same control: **all
four redden.**

### ⚠ FINDING: the battery's own reporting instrument was inert

The first run of the five-mutation battery under-reported. The regex collecting failing test
names was `^    [a-z_]+(::[a-z_:]+)?$` — **no digits** — so every `o2_makeup_*` and `rk4`
case was dropped silently. Four of the five rows were short; the corrected numbers are the
ones tabled above.

It was noticed only because a name that had reddened in an earlier ad-hoc run was missing
from the summary. Recorded because the failure mode is the section's own subject one level
up: a control's *instrument* can be inert exactly as a control can, and a filter that
silently matches nothing looks identical to a subject that changed nothing.

### The oracle rule, held

None of the 160 takes a value from a Python run. The five closed forms are re-derived in the
tests from their own algebra — the equilibrium identity `εσA(T_eq⁴ − T_space⁴) = heat_load`,
the daily-balance identity, the three per-species steady states, crew endurance
`store0/rate`, and Stefan-Boltzmann. Three of those have **no Rust twin to import even if
the rule allowed it** (`relaxation_time`, `steady_state`) or are private
(`radiated_power`), so writing them out was forced as well as correct.
`stefan_boltzmann_constant_is_codata_value` keeps CODATA, as §5v's template.

### What S3 leaves standing

* `bounds_match_the_loaders` is **kept and re-documented**, not deleted: what it asserts is
  still true and cheap, it is simply not the gate its name implies. Its docstring now says
  so and points at its successors — never weaken or delete a test to make a point.
* `test_bvad_validation.py` and `test_crew_coupled_loop.py` remain deferred to `station`,
  named with their destination.
* The nine Python files stay green and running. **S6 retires them, and only once S4 and S5
  have theirs** — the overlap is deliberate.
* No science changed, no golden regenerated, no param file touched.

### ⚠ Four review follow-ups, and the first one is the only *new* unasserted claim S3 introduced

Found by advisor review after S3 was committed, none of them falsifying the five-of-five
result (that stands on 33 named reddenings) and all four worth doing before S4.

#### 1. Nothing tied `run_trajectory` to `run` — closed

S3's central additive decision was to leave [`domains::run`] alone because `goldens.rs`
calls it. That was justified by predicting a zero golden diff, and `--test
golden_regression` confirmed it — **but that check only ever exercises `run`.** Meanwhile
all 68 sibling run-level tests go through `run_trajectory`. The tree therefore ended up with
two step loops, one certified by the frozen bytes and one carrying the behavioural coverage,
and nothing tying them together.

Today they agree; the exposure is forward. An edit to either desynchronizes them silently,
the goldens stay green, and 68 tests certify a path the frozen bytes do not cover — which is
this slice's own subject, one level up from where it is already diagnosed.

`crates/domains/tests/run_helpers_agree.rs`: the four frozen sibling scenarios plus the
self-discharge arm, run through both helpers, compared on the whole final `State` and on
`(rationed, events.len())`. **Control:** delete the trailing `trajectory.push` in
`run_trajectory` — all five redden.

#### 2. The three `#[ignore]`d goldens had never been asked

`0 failed; 3 ignored` was reported as if it were a clean reading. S2's own record says the
sealed station's byte comparison "happens only when a human runs `cargo test -- --ignored`
on Windows", and this work was done on Windows. §5v's exit condition includes *"No golden
regeneration. The predicted diff is zero; if it is not, that is the finding."* — a claim
three goldens had not been asked to confirm.

`cargo test --workspace -- --ignored`: **`cargo test --workspace -- --ignored`, on Windows: **all three pass**, in 404 s, 660 s and
207 s — `the_sealed_station_golden_is_still_this_reference_s_output` (the byte comparison S2
recorded as running nowhere automatic), `two_rate_sealed_session_matches_full_horizon_bit_exact`
and `sealed_resume_across_a_season_boundary_is_bit_identical`. The zero-diff prediction now
covers all 21 goldens rather than 18.

⚠ **And the first reading of this was mine and wrong, in the same shape as the last two.** The
first `--ignored` run reported "0 passed, everything filtered out", which read as *the ignored
selection matched nothing*. It was a `| head -20` in my own command truncating the output
before the `station` binaries ever printed. **Third instrument error in one slice** — a regex
without digits, a probe that was a coin flip, and now a truncated pipe. All three had the same
signature: an instrument returning *nothing* is indistinguishable from a subject in which
*nothing happened*, and only the second look tells them apart.**

#### 3. "The gate read zero before S3" was two measurements and three extrapolations

§5w said the pre-S3 reading was "read off measurement 2's own table with the golden row
removed". True for **M-power** — that *is* measurement 2 — and separately measured for
**M-bound** (green across all 488). M-eclss, M-thermal and M-crew were never run pre-S3; the
zero was inferred. A reasonable inference, but presented as a reading, in a section whose
subject is claims that outrun their measurement.

Rather than soften the sentence, the three were run at **workspace** scope. That closes the
claim per mutation *and* produces the by-name coverage census S6 needs before it deletes
`tests/crossport/test_crossport.py` — the same data measurement 2 gave for M-power and which
nobody had for the other three.

| Mutation | red workspace-wide | of those, in `domains` | S3's own | pre-existing `domains`, goldens excluded |
|---|---:|---:|---:|---:|
| M-eclss | 24 | 8 | 7 | **0** |
| M-thermal | 11 | 8 | 7 | **0** |
| M-crew | 10 | 5 | 4 | **0** |

In each case the single non-S3 `domains` entry is `every_domains_golden_is_still_this_
reference_s_output` — the byte compare the exit gate deselects. Everything else that reddens
lives in `station` or `godot_bridge`: perturbation scenarios, the palette, session parity, two
science gates. **So the pre-S3 reading really was zero, for all five mutations, and it is now
measured rather than inferred for three of them.**

⚠ The census also confirms the M-crew asymmetry from a second direction: workspace-wide it
reddens ten tests and **not one of them is a run-level sibling test**. Its only by-name
coverage anywhere is the four flow-level cases S3 wrote.

#### 4. `bits()` compared less than the case it ports

The determinism / bit-identity / order-independence tests compared stock amounts via
`to_bits()`. The Python originals compared the whole `State` — `n`, `rng_seed` and `aux`
included. `to_bits()` is *stricter* on the floats (`PartialEq` treats `+0.0 == -0.0`), so
this was a narrowing rather than a hole, and both are now asserted.

⚠ The added half is **recorded as weak today, not as coverage**: on the four siblings `n` is
always the step count, `rng_seed` is 0 and `aux` is empty, so given the bit comparison
passes the `State` comparison cannot currently fail. It is there because the ported claim is
about the whole `State` and because it starts biting the moment a sibling gains an aux
process or an RNG draw. Its docstring says exactly that — a `///` block in this repo is read
as a finding, so it must not read as more than it is.

## §5x Stage 3 — S4, the engine residue: two slices wearing one row, COMPLETE 2026-08-25

S3 gave the four sibling domains their own tests. S4 is the row the batch table called *"the
engine residue — extinction, aux, environment, integrator, multirate, the purity + `gdext`
gates"*, and the first thing to say about it is that it is **not one slice**. Its two halves
have different subjects and need different acceptance criteria:

* **Five behavioural port files** — `test_extinction.py` (7 cases), `test_aux.py` (23),
  `test_environment.py` (20), `test_integrator.py` (13), `test_multirate.py` (17): 80
  collected cases whose subject is arithmetic. S3's shape transfers unchanged.
* **Two structure gates** — `test_simcore_purity.py` (20) and `test_biosphere_purity.py`
  (39): 59 cases whose subject is *what a manifest contains and where a name appears*. The
  "mutation" that tests them is an edit to a `Cargo.toml` or a source line, not a flipped
  sign. This is exactly the class S3 taught the batch to distrust — a file-reading gate that
  globs the wrong path passes green forever — so it was taken **first**, before 60
  behavioural tests were written against a slice whose shape it could still change. That
  ordering paid: it is where the slice's two real findings came from.

### Gating measurement 1 — the baseline

`cargo test --workspace` on a clean tree: **653 passed, 0 failed, 3 ignored**, across 39 test
binaries. (The three `#[ignore]`s are S2's expensive goldens; §5w's review ran all three.)

### ⚠⚠ Gating measurement 2 — the four structure mutations, each measured, none inferred

§5w's own review found that *"the gate read zero before S3"* had been **two measurements and
three extrapolations**. So each mutation below was applied to a clean tree, run at workspace
scope, and reverted — no zero inferred from a neighbour:

| Mutation | What it breaks | Red before S4 |
|---|---|---:|
| **MS-A** | `simcore` gains a third-party dependency (`quote`) in `[dependencies]` | **0 of 653** |
| **MS-B** | `simcore` gains a third-party **dev**-dependency | **0 of 653** |
| **MS-C** | `simcore` gains a path dependency on `config` — the core reaching down into the boundary layer | **0 of 653** |
| **MS-D** | `station` gains `godot = "0.5.4"` — the gdext edge escaping the bridge | **0 of 653** |

Zero in every case, which is what FINDING 8 predicted in prose and nothing had measured:
**before this slice no Rust test in the repo read a `Cargo.toml` at all.** The Python purity
scans read Python *packages*, so retiring them at S6 loses no Rust coverage — there was none
to lose — but it removes the last thing in the tree that *looks* like a purity gate.

⚠ **The first MS-A run was an instrument failure, not a reading.** It produced no
test-result lines at all — `LNK1104: cannot open file …\emit_crew.exe`, a Windows relink
lock that recurs on this box between back-to-back builds. The script printed *"NO TEST RESULT
LINES (build failed?)"* rather than a zero, because it counted `test result` lines before
reporting anything; without that guard a broken build and a workspace where nothing noticed
would have produced **the same output**. That is S3's instrument-error signature in a new
costume, and the only reason it was caught is that the guard was written before the
measurement rather than after it went wrong. ⚠ It was first blamed on a stray recursive
`grep` walking `rust/target` — a plausible story that the *second* occurrence, with no grep
running, falsified. The battery now retries a run whose log contains `LNK1104` and reports an
instrument failure only after three attempts; a blanket retry would have hidden a mutation
that genuinely broke the build.

### ⚠⚠ FINDING: the `gdext` gate's subject was an open question, and the obvious answer was wrong

`CLAUDE.md` states the invariant as *"`gdext` appears in `rust/crates/godot_bridge` and
nowhere else"*, and FINDING 8 recorded one matching line in the tree. Measured before writing
anything, the string appears in **five** places outside the bridge — `station/src/session.rs`,
`station/src/palette.rs`, `station/src/bin/sim.rs`, `station/tests/session_parity.rs`, and
`Cargo.lock` — and **every one is a doc comment or a lock entry**, most of them saying the
crate is deliberately gdext-*free*.

So a literal text scan would have reddened on a clean tree on day one, and the natural next
move — widening it until it passed — is how a gate ends up asserting nothing. The gate is
over the **dependency graph** instead, and the reason that is not merely convenient is
structural: in Python an import is the only coupling, but in Rust a crate cannot name a type
it has not declared a dependency on. `use godot::…` in an engine crate **cannot compile**
without the manifest edge, so here the text half is redundant *by construction* rather than
by judgement. One thing the edge check genuinely does not imply gets its own case: a gdext
type could still reach an engine crate by re-export if anything depended on the bridge, so
`nothing_depends_on_the_bridge` closes that.

### ⚠⚠ FINDING: that same reasoning does **not** carry to the biosphere half, and assuming it did would have shipped a gate that asserts nothing

`test_biosphere_purity.py`'s subject is *intra-package*: the biosphere spine stays stdlib-pure
and the loader is the sole module allowed to import `config`. The manifest gate cannot see
that violation at all — the biosphere lives **inside** `domains`, and `domains -> config` is
a legitimate declared edge that the param loader needs. Every module under
`domains/src/biosphere/` could `use config::…` freely with every manifest assertion green.

The first draft of `workspace_purity.rs` named both Python files as its subjects. It was
caught in advisor review before the commit, and the correction is a second file —
`domains/tests/biosphere_spine_purity.rs` — that scans the spine's **source**. Where the
manifest edge is the whole coupling (gdext) the text scan is redundant; where the edge
already exists and is legitimate (config inside `domains`) the text scan is the *only* thing
that can see a violation. Two gates that look alike, one generalisation that does not travel.

Two details of that scan are measurements rather than defensive coding:

* **The boundary is two modules in Rust, not one.** `biosphere/params.rs` is `loader.py`'s
  counterpart, and `biosphere/weather.rs` became a second boundary when slice C9 moved the
  raw-weather path into the reference. Each exclusion carries the Python original's paired
  assertion — that the excluded file genuinely *does* reach `config` — so an exclusion cannot
  quietly decay into a typo that hides a leak.
* **A `contains("config")` scan would flag `flows.rs`**, the largest module in the spine, for
  the phrase *"when drought acceleration is not **configur**ed"* — a substring hit in a doc
  comment, in a file that touches the boundary nowhere. The detector strips line comments and
  matches `config` as a whole token; both halves have a control, and the substring control
  cites that line.

### What landed

Seven new files, **89 tests**:

| File | tests | Subject |
|---|---:|---|
| `simcore/tests/workspace_purity.rs` | 18 | the zero-dep charter, the crate layering, gdext containment — and ten discrimination controls on the manifest reader |
| `simcore/tests/aux_channel.rs` | 16 | the non-conserved auxiliary channel (`auxiliary.rs` had **0** tests) |
| `simcore/tests/multirate_driver.rs` | 16 | the master-step driver (`multirate.rs` had **0** tests) |
| `simcore/tests/environment_wiring.rs` | 12 | the forcing ⊕ shared-stock resolver residue |
| `simcore/tests/integrator_schemes.rs` | 10 | the Euler/RK4 arithmetic residue |
| `domains/tests/biosphere_spine_purity.rs` | 10 | the biosphere spine is free of the config boundary |
| `simcore/tests/extinction.rs` | 7 | FINDING 6 — extinction had no by-name test anywhere |

### ⚠ 139 Python cases became 89 Rust tests, and the difference is a census rather than a shortfall

FINDING 10's failure mode is copying a function and dropping its case table, and only the
ratio makes that visible — so both counts are recorded. The seven Python files hold **78 test
functions** expanding to **139 collected cases**. Where the difference goes:

* **59 of the 139 are the two purity files, and 52 of those are one case per source file** —
  `parametrize(_CORE_FILES)` yields 17, `parametrize(_PURE_FILES)` yields 35. The Rust
  analogue of a per-file parametrize is not 52 tests; it is one assertion looping over the
  files, plus an anti-vacuity guard and the detector's own controls. That half is a change of
  shape, not of coverage — and the Rust side gained three claims the Python side never made
  (the layering, gdext containment, and the dev/build/target dependency tables).
* **Six behavioural cases are already covered by `laws.rs`** and were deliberately not
  re-ported: step order-independence for both schemes (2), multi-rate order-independence (1),
  the aux sum's order-independence (1) and its associativity pin (1), and the
  forcing-depends-only-on-`(n, dt)` law (1). `laws.rs` enumerates its permutations
  exhaustively where the Python originals sample with hypothesis, so a re-port would have been
  a second, weaker copy.
* **Eight cases are unrepresentable in Rust**, plus the immutability half of a ninth, and all
  are named in the file headers rather than dropped silently: three Protocol-conformance
  assertions on `Environment`, "both integrators satisfy the `Integrator` protocol", "both
  satisfy `Substepper`", `AuxProcess` being `runtime_checkable`, the aux mapping being
  read-only, and `State` detaching from the caller's dict. In Rust a type either implements a
  trait or does not compile, and a `State`'s map is owned and moved in — there is no failing
  state for a test to guard.

The remainder is roughly one-for-one, with the parametrized axes expanded by hand (both
integrators, three `n_sub` values, three non-finite shapes) rather than collapsed.

### ⚠ A finding about the *core*, found by writing the tests rather than by reading it

`StepIntegrator` — the trait that lets one function take either scheme — is defined in
**`domains`**, one crate *above* `simcore`. So `simcore` has no polymorphic step interface of
its own, and every `simcore` test wanting the Python `@parametrize("integrator_cls", [Euler,
Rk4])` axis has to dispatch by hand over a local enum. Four of the new files do exactly that,
and each says why. Recorded, not fixed: moving a trait between crates is a layering decision,
and S4 is not the slice for it.

### The acceptance battery: thirteen mutations, pre-committed

S4 is not done when 89 tests exist. It is done when each of these reddens a test **in one of
the seven new files** — not merely "reddens something", since the goldens and the
cross-boundary vectors light up for most of the behavioural ones.

Each mutation was applied to the finished tree, run at workspace scope with
`--no-fail-fast`, and reverted. The **before** column is exact rather than re-measured: the
seven new files are purely additive, so a mutation's pre-S4 reading is its total minus the
reds that live in them.

| Mutation | before | after | of those, in the new files |
|---|---:|---:|---|
| **MS-A** `simcore` gains a third-party dependency | 0 | 3 | **3** — the layering, third-party and zero-dep clauses |
| **MS-B** `simcore` gains a third-party **dev**-dependency | 0 | 3 | **3** — the table a narrower gate would have missed |
| **MS-C** `simcore` gains a path dependency on `config` | 0 | 2 | **2** — the third-party clause correctly stays green; this is a *layering* violation |
| **MS-D** `station` gains `godot` | 0 | 3 | **3**, including `only_the_bridge_depends_on_gdext` |
| **MB-9** a biosphere spine module reaches `config` | 0 | 1 | **1** — and it is the only test in the workspace that can see it |
| **MB-1** the extinction branch is disabled | 1 | 6 | **5** — four in `extinction.rs`, one in `multirate_driver.rs` |
| **MB-2** extinction snaps POOL stocks too | 110 | 111 | **1** — `a_pool_below_threshold_is_never_zeroed` |
| **MB-3** aux advances four times per step | 9 | 16 | **7** |
| **MB-4** `substep` advances aux as well as `step` | 0 | 1 | **1** — the only test in the workspace that can see it |
| **MB-5** the Strang slow halves step at `dt/n_sub` | 0 | **0**, then 1 | see the finding below |
| **MB-6** the master step commits `n + n_sub` | 2 | 5 | **3** |
| **MB-7** the forcing non-finite guard is removed | 0 | 1 | **1** — the only test in the workspace that can see it |
| **MB-8** the RK4 combine drops the key union | 1 | 2 | **1** |

⚠ **MB-1's before-reading of 1 independently reproduces FINDING 6's original control** — same
mutation, same single red (`engine_vectors.rs::engine_synthetic_trajectory_is_bit_exact`),
measured eight days later on a tree with 296 more tests. A useful check on the apparatus
itself.

⚠ **MB-6 has one case that is inert by construction and it is not counted as coverage.** At
`n_sub == 1`, `n + n_sub` *is* `n + 1`, so `n_advances_once_per_master_step_with_one_substep`
cannot discriminate it. Two of the three cases on that axis do the work; the third is there
because `n_sub = 1` is the degenerate partition the driver must also get right.

### ⚠⚠ FINDING: MB-5 reddened nothing — a wrong slow half-step size was invisible to the whole workspace

The mutation changes the Strang slow halves from `dt/2` to `dt/n_sub`. It reddened **0 of
741**, and the reason is that every candidate gate was blind for a different reason:

* The three **order-of-accuracy** cases all run at `n_sub == 2`, where `dt/n_sub` **is**
  `dt/2` — the mutation is literally a no-op for them. One hand-picked parameter value, and it
  happened to be the one that cancels: the same coin-flip S3 found in its own
  registration-order probe, in a new costume.
* **Conservation, determinism and the `n` contract** run at other `n_sub` but assert
  quantities a wrong step size does not move. Each half is balanced whatever its size, so
  carbon still closes exactly.
* The **eval-count** case runs at `n_sub == 4` and counts *evaluations*: `ops` holds exactly
  two slow entries whatever `n_sub` is, so the count stays 8. Its first docstring claimed the
  opposite — that it would read 16 — and advisor review caught that before the commit.
* `all_slow_strang_does_not_reproduce_a_single_rate_step` runs at `n_sub == 1`, where the
  mutation *does* change the numbers — but its assertion is that the gap exceeds `1e-6`, and
  the mutation makes the gap **larger**. Blind by the direction of its inequality.
* ⚠ And outside this file, `authoring`'s `a_non_empty_slow_set_is_driven_at_dt_over_2` has the
  behaviour **in its name** and asserts only that the slow flow's stock moved at all. It runs
  at `n_sub = 60`, where the wrong size is `60 s` against `1800 s` — a thirty-fold error its
  own assertion cannot see.

This is the third recorded instance of the same blind spot in this repo — reasoning about
`n_sub` as though it governed the slow rate class — after a performance prediction and a
safety predicate that false-PASSED. The first two were wrong *claims*; this one is a wrong
*absence of a gate*, which is why nothing caught it for a year.

The closing case is exact rather than asymptotic, because an order fit at one `n_sub` is what
failed here: with a **constant-rate** flow, an empty fast set and `n_sub ∈ {1, 2, 3, 5}`, the
slow operator must move exactly `rate·dt` — `dt/2 + dt/2` — and under the mutation it moves
`2·dt/n_sub` of it. No tolerance hides that and no `n_sub` cancels it.

**Re-run with that case in place: MB-5 reddens 1 of 742, and it is the new case.** Nothing
else in the workspace sees it, which is the same statement as "the pre-S4 reading was zero" —
and this one was found by the battery rather than predicted by the design, which is the whole
argument for pre-committing a mutation you expect to be caught.

### ⚠⚠ The battery's own instrument was wrong first, and it read as a result

The first battery pass reported **"MB-1: 1 red — `engine_vectors.rs`"**: extinction disabled,
and none of the seven new `extinction.rs` cases noticed. That is impossible, which is the only
reason it was caught. The cause was `cargo test` without `--no-fail-fast`: cargo stops after
the first failing test *binary*, and `engine_vectors` sorts before `extinction`, so every
simcore binary after it never ran. The log said `passed=552` against a baseline of 653 and
nothing in the report drew attention to the gap.

**The reading was not merely incomplete — it was the exact opposite of the truth**, and it
would have been recorded as "the new tests are inert, the old vector is still the only gate".
An instrument that stops early reports a *smaller* census, and a smaller census reads as
*less coverage* — the direction that looks like an honest negative finding. That is the
fourth instrument error in two slices and the first one whose failure mode was flattering to
nobody: S3's three all returned *nothing*, which at least looks broken. This one returned a
plausible number.

### S4's exit gate, stated forward to S6

S3's exit criterion was written in S6's terms so S6 would not have to infer it. S4's:

> **S6 may delete `tests/test_simcore_purity.py` and `tests/test_biosphere_purity.py` only
> once MS-A, MS-B, MS-C, MS-D and MB-9 each redden a named test in
> `simcore/tests/workspace_purity.rs` or `domains/tests/biosphere_spine_purity.rs`.**

Before S4 that gate read **0 of 653 for all five**, measured one mutation at a time. After
S4 it reads as the battery table above records. Pair this with the table correction below:
the two files are filed `R` — *retire free* — and they are `R!`-shaped.

### Two corrections to §5o's own tables

1. **The two purity files are `R!`, not `R`.** The `R` bucket means *"subject is the Python
   tree, or unfalsifiable in Rust"*; both halves are true of the *scan* and neither is true of
   the *conclusion*. FINDING 8's own text says the successor *"has to be written rather than
   assumed"*, which is the definition of `R!` (retire only once its successor stands). As of
   this slice the successor stands, so the practical effect is nil — but the row was wrong on
   the day it was written, and S6 reads those rows as a work order.
2. **Both rows cite FINDING 7 where the subject is FINDING 8.** FINDING 7 is the sibling
   domains' cross-port gate; FINDING 8 is the purity/`gdext` pair.

### What S4 leaves standing

* **No Python deletion.** Same rule S3 held: the seven files stay green and running until S6.
* **The rest of the `C?` engine residue is not covered and was not in this row**:
  `test_state.py`, `test_flow.py`, `test_composition.py`, `test_boundary.py`,
  `test_registry.py`, `test_observation.py`, `test_conservation.py`, `test_arbitration.py`,
  `test_edge_cases.py`, and `test_config.py`'s pint half. S4's row named five files and took
  five.
* **FINDING 9's seven oracle-named Rust tests** are untouched; they belong to whichever batch
  retires their subject.
* **The sibling domains' own source purity is unclaimed.** `biosphere_spine_purity.rs` scans
  the biosphere spine because that is what the Python original's subject was; `power.rs`,
  `thermal.rs`, `eclss.rs`, `crew.rs` and `domains/src/params.rs` are not scanned, and
  inventing a wider claim would have been a new gate wearing a port's name.

## §5y Stage 3 — the four **D** decisions, ANSWERED 2026-08-25 (a decision pass; no code touched)

§5q filed 11 files / 31 tests as **D — decide**: *"no natural Rust home; the user's call"*,
and S6's row says they *"need answers before this slice, not during it"*. They were put to
the user as four questions on 2026-08-25, after a fact-check pass that found **two of the
things the questions would have rested on were stale in this very document**. Both
corrections are below, because the second one changes what one of the answers costs.

### The answers — all four the same direction

The user chose the **maximal-Rust** option on every one. Stated as a posture rather than
four unrelated calls: **after S6 the repo contains no executing Python outside the PCSE
oracle carve-out.** No "keep it as a script CI calls" island survives — an option that was
offered on three of the four, recommended on two, and declined every time.

| # | Subject | Answer | What it costs |
|---|---|---|---|
| 1 | `test_context_budget.py` (10) | **Rewrite in Rust** | needs a home; see the crate question below |
| 2 | `crossport/test_headless_cli.py` (4) | **Faithful Rust port** | a cargo process launching cargo; keeps byte-exactness on every platform |
| 3 | the nine `crossport/test_godot_*.py` (17) | **Port to Rust tests** in `godot_bridge/tests/` | the CI job swaps `pytest` for `cargo test`; Godot install step unchanged |
| 4 | `crossport/tiers.json` + the band gates | **Port the tolerance checks to Rust** | ⚠ an **unfreeze event** on `docs/native-port-reference.md`; the numbers must be RE-MEASURED, not translated |

⚠ **This is four build items standing between S5 and the deletions, not four filing
decisions.** S6's row reads *"the retirements"*; it now has a prerequisite batch in front
of it, and #4 alone carries a freeze ceremony. Whether they land as one slice or four is
not settled here — what is settled is the direction, and that none of them is a deletion.

### ⚠ CORRECTION 1 — FINDING 11's "local-only" is STALE, and the D table repeated it

FINDING 11 says the nine Godot files *"are `skipif`-ed on CI, so they are local-only
today"*, and the D-table row for `test_godot_compose.py` repeats it as `local-only`. **False
since Phase 8 Step 8.** `.github/workflows/ci.yml` carries a dedicated `godot-parity` job
that installs headless Godot 4.7 plus the Rust toolchain and runs
`pytest tests/crossport/ -k godot -m "not slow"` — **15 of the 17 run on CI**; only the two
in `test_godot_two_rate_parity.py` are `-m slow` and therefore mandatory-local.

⚠ **And the job block was checked against the workflow's triggers, not read alone.** `on:` is
`push: branches: [main]` plus `pull_request`, and **no job in the file carries an `if:`** — so
`godot-parity` runs on every push to `main` and every PR, unconditionally. Stated because the
correction above generalises one level further than it first did: a `skipif` is a claim about an
*environment*, and a job block is a claim about a *workflow* — only the triggers say what actually
executes. Verifying the second was the same move as verifying the first, and it was nearly skipped.

The job's own comment says it *"promotes the Step-1 cross-boundary smoke from a silently-skipped
local test to a real gate"* — so the promotion was recorded where it happened and the
classification pass read the `skipif` in the test file instead of the workflow that defeats
it. **A `skipif` is a claim about an environment, not about CI**; the only way to know
whether it fires is to read the runner's install steps. This changes the decision's stakes
rather than its answer: deleting these deletes a **running** gate, not a dormant one.

### ⚠ CORRECTION 2 — the tolerance question had a measurable answer, and it is not "a data file"

The `tiers.json` row was written as an orphaned-data problem (§5q FINDING 2's fifth entry:
*"its numbers stranded in the dying tree"*). Measured before asking, the gap is larger:

* **The reference has no numeric tolerance at all.** `domains::goldens::compare` returns
  byte-exact when `Numerics::PureArithmetic` **or** `cfg!(windows)`; otherwise it falls to
  `compare_structural`, which asserts a hex-float leaf *parses finite on both sides* and
  compares nothing about its value. `goldens.rs:44` says so in its own words — *"a band was
  deliberately NOT invented"* — and that was the right call for S2, which had no measurement.
* **Python's band gates are live and are the only ones with teeth off Windows.**
  `test_rust_siblings_match_their_tier` and `test_rust_biosphere_states_match_tier2` compare
  parsed f64 against the committed band; `test_tier2_bands_sit_above_measured_sensitivity`
  re-derives the ±1-ULP sensitivity from the tree and asserts `band > sensitivity` **and**
  `band <= 1e-9`, so the band cannot be widened until it passes. Four more tests
  (`covers_exactly_the_frozen_goldens`, `entries_are_internally_consistent`,
  `tier1_set_is_the_four_transcendental_free_scenarios`, `power_is_tier2_not_tier1`) gate the
  contract's own shape.
* **So the CI matrix is the whole point.** Goldens are UCRT-generated on Windows; the
  `crossport` job runs glibc Rust against them. That job is the repo's only genuine
  cross-libm measurement, and on it the Rust-side comparison is structural while the
  Python-side one is banded. Retiring Python without the port turns the one place the bands
  were ever exercised into a presence-and-finiteness check.
* ⚠ **It cannot be translated line by line — and READ, the reason is worse than the claim.**
  This bullet first sourced *"shims CPython's own `math`"* from `tiers.json`'s own `_comment`,
  i.e. from the one file this document has caught carrying stale prose **three times in two
  slices**. Opening `measure_tier2_bands.py` sharpens it: it does not shim the global `math`
  module — it replaces the `math` reference **inside the Python domain modules**
  (`domains.power.system`, `domains.biosphere.canopy`) with a one-ULP-nudged stand-in and then
  runs the **Python engine** (`simcore.integrator.EulerIntegrator`) to propagate it. So the
  measuring instrument is built out of the tree S6 deletes; it dies with its subject, and a Rust
  port must re-measure against the *Rust* engine and land the same numbers. That is the ceremony,
  and it is the only thing that proves the port faithful. ⚠ The script's own lines 209–215 record
  the trap waiting for that port: both biosphere probes once shimmed a module the carbon path no
  longer called and measured **exactly 0.0 — passing vacuously**. A re-measurement that reads zero
  is the failure mode, not the result.

### The open sub-question decision 1 leaves — deliberately not answered here

`test_context_budget.py` has **no crate that owns its subject**: it reads `CLAUDE.md`,
`docs/post-roadmap-log.md`, `docs/log/*` and the memory index. Hosting it means either a new
workspace member — which `rust/Cargo.toml`'s own comment refuses in principle (*"no empty
speculative crates"*, written against exactly this move) — or hanging it off an existing
crate, which makes an engine crate reach **up and out** of `rust/` and re-opens FINDING 1,
the reach-out S1 spent a whole slice closing. Neither is free and the choice is a design
call for the slice that builds it, not a consequence of the answer recorded above.

⚠ Recording the collision now so the building slice cannot mistake it for a detail: the
workspace comment is a rule this plan has cited before, and quietly adding the crate the
rule forbids — without saying that is what is happening — is how a standing rule dies.

## §5z Stage 3 — D1 BUILT: the context budget gets a Rust home, COMPLETE 2026-08-25

The first of §5y's four build items. `tests/test_context_budget.py`'s ten gates now stand in
`rust/crates/repo_gates` — a **seventh workspace member**, dev-only, `publish = false`, whose
subject is this repository rather than the simulation.

### The crate question §5y left open, answered by reading the rule

§5y recorded a collision: both homes looked forbidden. **On reading, one of them is not.**
`rust/Cargo.toml` says *"no **empty** speculative crates **before then**"* — a rule against
standing a crate up *before it has content*, written when `domains` and `station` were still
speculative. A crate arriving with ten live gates and seventeen tests is neither empty nor
speculative, so the rule permits it. The reading is recorded in the crate's own manifest so
the next reader does not have to re-derive it.

⚠ **And it is not a re-opening of FINDING 1.** That finding was about *engine* crates
reaching out of `rust/` at **compile** time via `include_str!`, so `rm -rf src/ tests/` broke
the build. `repo_gates` reads `CLAUDE.md`, `docs/` and the memory index at **run** time; the
files are permanent repository documents rather than the tree being deleted; and **nothing
depends on it**. A failure here is a red test, never a broken build. The last clause is not
an intention — it is asserted (below).

### What the port is not allowed to be: three scanners, and why they are pinned separately

The Python original leans on three regular expressions. Rust got hand-rolled scanners rather
than a regex dependency — the house style, and the smaller change for a crate that reads four
markdown files. ⚠ **The danger is specific and it is not "the scanner is wrong".** Eight of
the ten gates are *set comparisons*; a scanner that silently matches **less** shrinks both
sides of every one of them, so the suite reads green while checking nothing. A set comparison
cannot see this from the inside. So `tests/scanners.rs` (7 tests) pins the two exclusions that
are easy to drop — the `memory/` lookbehind and the word boundary — plus greediness, the
`.md` suffix, and multi-hit lines.

### The controls — a differential, then nine mutations

**Control A, differential.** Both implementations were run over the real corpus and their
outputs diffed: **130 lines, identical** — every plan doc named by the index, every one named
by the record files, every pointer link. The port sees exactly what the regexes saw, measured
rather than argued.

**Control B, the mutation battery.** Nine repository mutations, each applied to the real tree,
run against **both** suites, then reverted:

| | Mutation | Rust red | Python red |
|---|---|---|---|
| M1 | `CLAUDE.md` padded past the ceiling | `claude_md_ceiling` | same |
| M2 | a status-ledger row appended to `CLAUDE.md` | `no_status_ledger_in_claude_md` | same |
| M3 | one index line deleted | row count **+** plan-doc parity | same two |
| M4 | a stray file in `docs/log/` | pointer parity | same |
| M5 | a record file's heading changed | heading parity | same |
| M6 | a 200-char line appended to a record file | line cap | same |
| M7 | an unindexed plan doc on disk | completeness | same |
| M8 | a phase-table row edited | phase-table pin | same |
| M9 | a record file renamed away | heading **+** pointer **+** plan-doc parity | same three |

**Nine of nine redden, and the two ports redden on the same gates every time.**

⚠ **M8's first attempt reddened NOTHING — in either language — and the probe was the defect.**
It replaced the first `COMPLETE` in `docs/phase-index.md`, which sits in the file's *prose*,
not in a table row; the gate hashes only `|`-lines and correctly ignored it. Re-aimed at an
actual row, both ports went red. This is S3's lesson arriving for the third time: **a control
that stays green is a claim about the control until you have shown the probe can bite.** The
inert reading was one sentence away from being written up as "the phase-table pin is inert".

### The gate that caught this work — S4's, doing exactly its job

`cargo test` went red the moment the crate joined the workspace:
`simcore/tests/workspace_purity.rs::the_scan_sees_every_workspace_member_and_no_others`, with
the message *"the workspace roster moved; give the new crate a layering rule in
`allowed_edges()` and a decision about whether it may carry third-party deps"*. That gate is
three commits old (§5x) and it stopped a new member from being silently exempted by a scan
that never looks at it. Two new assertions answer it:

* `the_repo_gate_crate_reaches_only_the_hash_helper` — exactly one edge, to `config`, for
  sha-256; and every dep is a path dep. Controlled: adding `regex = "1"` reddens it.
* `nothing_depends_on_the_repo_gate_crate` — the load-bearing direction.

⚠ **The second one nearly went in unmeasured, and the first control was run wrong.** The
control that added `station -> repo_gates` reddened `every_engine_edge_is_one_the_layering_allows`
— the *pre-existing* test — not the new one, which reads like the new assertion is redundant.
It is not: the layering test only inspects crates in `ENGINE_CRATES`, and `godot_bridge` is
not one. Re-run as `godot_bridge -> repo_gates`, **only the new assertion reddens**. The
justification in its doc comment is that measurement, not a plausible argument.

⚠ **One self-inflicted finding worth recording because it cost real work.** A control's
cleanup line was written as `git checkout -- <file> || git checkout -- .`. The crate was
untracked, so the first half failed and **the fallback reverted every tracked edit in the
tree** — the workspace roster and the purity gate's own changes. Nothing was lost that could
not be retyped, and the untracked crate survived precisely because git could not touch it.
*A cleanup command with a wider blast radius than the mutation it reverses is not a cleanup.*

### What D1 leaves standing

* **No Python deleted.** `tests/test_context_budget.py` stays green and running until S6 —
  the same rule S3 and S4 held, and the reason both suites could be run against every mutation
  above.
* **The memory-index assertion still does not run on CI**, by design, and now says so on
  stderr rather than through pytest's skip. `#[ignore]` was refused: it is opt-in, so it would
  have been silent *locally* too, which is the one place the assertion can actually run.
* **`docs/context-budget.md` is unchanged.** The rules did not move; only the language that
  enforces them.

## §5aa Stage 3 — D2 BUILT: the headless CLI gate, with no cargo on the reference side, COMPLETE 2026-08-25

`tests/crossport/test_headless_cli.py`'s four cases now stand in
`station/tests/headless_cli.rs` as three tests. The claim is unchanged: `sim <scenario>
<steps>` is **byte-for-byte** the same simulation as the reference run, for two single-rate
palette entries and one two-rate, and a bad invocation is visible to a shell.

### The design the port did NOT take, and why the obvious shortcut is a weakening

§5y and its advisor call both assumed a faithful port meant **cargo launching cargo** —
Python shells out twice because from outside the workspace an `examples/` program is a binary.
Reading the code killed that: every `emit_*` example is one line,
`print!("{}", station::goldens::cabin_gas())`. From inside the crate the reference side is a
**function call**. Same bytes, no subprocess, byte-exact on every platform.

⚠ **The other shortcut — compare the CLI against the committed goldens — was measured and
refused.** Off the generation platform `domains::goldens::compare` falls back to a *structural*
comparison for transcendental goldens, so `greenhouse` would silently stop being byte-compared
on the Linux CI job while still reading like it was. That is the §5t lesson (a platform policy
that classifies rather than skips) arriving as a trap for the next slice, and it is exactly the
shape this stage keeps finding: the cheaper route passes, and stops checking the thing.

### The assumption the port introduced, and the one place a nested cargo earns its keep

Calling the library function assumes the examples really are thin wrappers. **Nothing in
`rust/` referenced the `emit_*` programs at all** — grepped before writing, and the only hit
was a doc comment. So that assumption was gated by nothing until the next golden regeneration,
at which point a stray newline would surface as a golden diff that *looks like the science
moved*. `the_emit_examples_are_the_thin_wrappers_this_file_assumes` closes it, and does so by
**running the program**, never by scanning its source — §5x is the record of a text scan being
structurally unable to express the rule it was written for.

### The controls

`cargo test` deadlocking on its own build lock was treated as an open question rather than
reasoned about: a throwaway probe ran `cargo run --example` from inside `cargo test` and
exited 0. Then four mutations, each reverted, each naming the test it must redden:

| | Mutation | Reddens |
|---|---|---|
| C1 | `sim` prints a trailing newline | the byte-identity test |
| C2 | `sim` advances one extra step | the byte-identity test |
| C3 | `emit_cabin_gas` grows a newline | **only** the wrapper test |
| C4 | `sim` exits 0 on an unknown scenario | the bad-argument test |

**C3 is the one that pays for the extra test.** The example broke and the byte-identity test
stayed green — which is the whole reason the wrapper test exists, demonstrated rather than
asserted.

⚠ **All four were re-run after a clippy fix restructured the file** (`cases()` returned a
four-element tuple; `clippy::type_complexity` refused it, so it became a named `Case` struct).
The controls had already passed on the pre-refactor tree, and this repo's own rule is that a
control's verdict is dated to the tree it ran on. Re-run: four for four, unchanged.

### What D2 leaves standing

* **No Python deleted.** `test_headless_cli.py` stays green until S6.
* **The `sealed` palette entry is still uncovered.** `sim` accepts four scenarios; the Python
  original tested three and so does this. Widening the roster would be a new claim wearing a
  port's name — the same line §5x drew around the sibling domains' source purity.

## §5ab Stage 3 — D3 BUILT: the cross-boundary proof moves to Rust, COMPLETE 2026-08-25

The nine `tests/crossport/test_godot_*.py` modules — 1,671 lines, 17 tests — are now
`godot_bridge/tests/cross_boundary.rs`: **19 tests**, one harness, the same twelve GDScript
smokes and the same three claims (bit-exact snapshot, FTZ/DAZ off on the stepping thread,
Tier-0 discretes). The `godot-parity` CI job runs `cargo test` instead of `pytest`, and its
`Install uv` step went with the driver — nothing Python runs in that job any more.

### Two extra tests, and they are the two that earn their place

The port added `the_report_accessors_refuse_a_missing_or_mistyped_field` and
`the_godot_lookup_agrees_with_the_environment`. Neither has a Python original, and both exist
because a Rust port of a `skipif`-guarded subprocess harness has exactly two ways to be
silently inert: **the report accessors defaulting** instead of failing, and **the tool lookup
returning `None`** on a machine that has the tool. The second one earned itself inside an hour
(below).

### The findings, all four from running rather than reading

⚠ **1. `Path::parent` is lexical, and the self-check is what said so.** `repo_root()` was
built as `CARGO_MANIFEST_DIR` + `"/../.."`; `parent()` on a path *ending* in `..` strips that
component rather than resolving it, so the Godot project resolved to
`crates/godot_bridge/../godot`. Sixteen smokes failed with confusing "markers missing"
messages and one test said plainly that `godot/project.godot` was not where the code thought.
Rebuilt by walking **up** from the manifest dir.

⚠ **2. Three of the twelve smokes take a scenario path after `--`, and omitting it does not
crash.** `from_file_smoke.gd`, `from_file_template_smoke.gd` and `authored_marker_smoke.gd`
each printed a **well-formed report with `ok: false`** and zeroed numbers. A laxer port — one
that checked "markers present, JSON parses" — would have called that a pass and shipped three
tests that drive nothing. They were caught only because every assertion reads the report's own
`ok` flag first. Two field names were wrong in the same place (`base_food`/`big_food` for
`food_default`/`food_4x`), and that too surfaced as a red rather than a default.

⚠⚠ **3. `.trim()` is a narrowing, and only the control found it.** `assert_same_snapshot`
compared `produced.trim()` against `headless.trim()`. It reads as tidiness. The control that
made `emit_cabin_gas` print a trailing newline left the test **GREEN** — the one thing the
whole file exists to catch, absorbed by a courtesy call. The Python original compares the two
raw strings (only the JSON envelope between the markers is stripped). Trim removed from both
the snapshot and the golden comparison; the control then reddened by name. *"Byte-for-byte"
has to mean bytes.*

⚠ **4. `cargo test -p godot_bridge` does not build the cdylib.** Measured by deleting
`target/debug/godot_bridge.dll` and re-running: with `crate-type = ["cdylib"]` the test
profile builds a harness, not the artifact Godot `dlopen`s. So the port builds it exactly as
the Python driver did. A nested cargo inside `cargo test` does **not** deadlock on the build
lock — also measured, with a throwaway probe, before the file was written rather than reasoned
about afterwards.

### The controls

| | Mutation | Result |
|---|---|---|
| C-A | the headless reference grows a trailing newline | **first run: GREEN — finding 3**; after the trim came out, reddens by name |
| C-B | a marker constant no longer matches the smoke's output | reddens with "markers missing", not a pass |
| (live) | the path helper was wrong | 18 red, and the harness self-check named the cause |
| (live) | three smokes invoked with no scenario argument | 3 red on the report's own `ok` flag |

The last two were not designed — they happened, and they are recorded as controls because
they measured the same thing a designed mutation would have.

### The timeout was ported, not dropped

Every `subprocess.run` in the Python original passed `timeout=`. The Rust `Command::output()`
has no equivalent, so `run_bounded` drains both pipes on threads (a chatty child must not fill
a pipe and deadlock) and polls `try_wait` to a deadline, killing on breach. Dropping it would
have turned a hung headless Godot into a CI job that burns its whole budget without naming the
test that hung.

### The document nothing would have caught

`docs/phase-8-reference.md` hand-lists the coverage as a table of **Python file names**. It is
the one freeze doc with **no manifest** (Phase 8 added a consumer and changed no science), so
the port would have left it naming files that no longer exist with every gate green — the
"freeze's prose half is ungated" lesson, arriving exactly where that lesson predicts. Table
re-pointed at the Rust test names, with the driver move stated in place.

### What D3 leaves standing

* **No Python deleted.** The nine modules stay green and running until S6 — and on the
  `crossport` CI job they now simply skip, as they always did where Godot is absent.
* **The GDScript smokes are untouched.** The Godot side of the boundary is not what moved.
* **The slow pair is CI-excluded by name**, not by `#[ignore]`, so it still runs by default on
  the developer machine that is the only place it ever runs.

## §5ac Stage 3 — D4: the tolerance contract moves to the reference, COMPLETE 2026-08-25 (one half deferred, named)

The fourth and last of §5y's build items, and the only one carrying a freeze ceremony. The
cross-port tolerance contract — `docs/native-port-reference.md` and its numbers — is now read
and enforced by the reference.

### What moved

* **`tests/crossport/tiers.json` → `rust/data/tiers.json`**, beside the goldens it classifies.
  The Python checker follows it to the new path and stays green until S6. **No band, floor or
  tier changed** — the predicted diff was a path and nothing else, and that is what it was.
* **`domains/src/tiers.rs`** — the reader and the comparison. Tier 1 is bit-exact on parsed
  f64; Tier 2 is `max |c−r| / max(|r|, floor) ≤ band`. The table is *read*, never mirrored in
  Rust: a hand-copied roster is the defect `coverage-roster-is-not-the-manifest.md` records.
* **`domains/tests/tier_contract.rs` (7) + `station/tests/tier_contract.rs` (6)** — the gates.
  The shape gates live in `station` because it is the only crate that can see **both** frozen
  rosters, and `frozen_goldens()` was exposed on each `freeze_manifest` so the roster comes
  from the manifest's own source rather than from parsing a committed document.

### ⚠⚠ The hole this closes is bigger than "the data was stranded"

§5q filed this as orphaned data. Measured, it is a **missing assertion**.
`domains::goldens::compare` carries *no numeric tolerance at all*: byte-exact for
pure-arithmetic goldens and on Windows, and otherwise a **structural** walk that asserts a
hex-float leaf parses finite and says nothing about its value. So on the `crossport` CI job —
glibc Rust against UCRT-generated goldens, the repo's only genuine cross-libm measurement —
the banded assertion existed **only in Python**.

`the_structural_walk_is_blind_to_the_value_this_file_checks` pins that in-tree and on any
platform: hand `compare_structural` two snapshots differing by ten times a measured band and
it reports **equal**; the same pair fails `compare_at_tier`. Written because the golden-nudging
control below cannot demonstrate it on Windows, where both gates are strict.

### The controls

| | Mutation | Reddens |
|---|---|---|
| C1 | a golden leaf nudged **1e-11**, above power's 1e-12 band | the banded run gate, naming the leaf and both numbers |
| C5 | the same leaf nudged **1e-13**, below the band | **nothing — correct.** C1+C5 together prove the band *value* is load-bearing, not merely present |
| C2 | a classified row dropped | "classifies exactly the frozen goldens" |
| C3 | a Tier-1 golden promoted to Tier 2 | the Tier-1 set gate |
| C4 | a Tier-2 row half-calibrated (band null, floor kept) | internal consistency **and** the banded run gate |

Plus six unit controls on the arithmetic itself: a generous band cannot rescue a Tier-1
difference, an uncalibrated Tier-2 row refuses to compare rather than permitting anything, a
shape mismatch is an error, and **a comparison that finds no numeric leaves is a failure, not
a vacuous pass**.

⚠ **C3 first read as inert and the probe was again the defect.** The control harness grepped
failing test names with `^ *[a-z_]+$` — which excludes digits, and the test is called
`the_tier1_set_...`. Third time this session that a green control was a statement about the
control. Re-run with the output read directly: it reddens.

### ⚠ The direction of the floor, recorded because the natural reading is backwards

`floor` **enlarges** the denominator when a reference leaf is smaller than it, so it makes the
comparison *more* forgiving near zero and inert elsewhere. Dropping it would make the gate
**stricter**, not weaker — it would fail loudly rather than pass quietly. An advisor note in
this slice had it the other way round; the code is the authority and
`the_floor_is_permissive_and_only_near_zero` pins the direction so nobody re-derives it.

### ⚠ What is DEFERRED, and it is the half the user's answer named

The four `band > measured sensitivity` re-derivations are **not ported**. They perturb a
transcendental by one ULP and propagate it through the engine;
`tests/crossport/measure_tier2_bands.py` does this by substituting a `math` reference **inside
the Python domain modules** and running the **Python engine**. The instrument is built out of
the tree S6 deletes.

**It is portable** — checked rather than assumed, and the answer changed the verdict.
`solar_schedule` returns a public `Box<dyn Fn>` closure a test can wrap with a nudge, `Flow` is
a public trait and `RadiatorReject` a public struct, so a test can register a perturbed flow
exactly as Python replaces a module attribute — **no change to frozen engine code**. What it
costs is four bespoke perturbation seams (power ×2, thermal, the biosphere `exp`, the
greenhouse) and a re-measurement that must land the same numbers.

⚠ And the trap is recorded in the Python tool's own comments: both biosphere probes once
shimmed a module the carbon path no longer called and measured **exactly 0.0, passing
vacuously** for weeks. A re-measurement that reads zero is the failure mode, not the result.

**This is left for the user's call rather than done or dropped**, because it is a scope
decision they already paid for once: they chose "port the tolerance checks" knowing it meant
re-measuring. What is deferred is one clearly-named piece, and until it lands the committed
bands are *asserted* but no longer *justified in-tree* — the Python justification still runs,
and dies at S6.

### The ceremony

Advisor-reviewed; gates written before the data moved; mutation-controlled;
`docs/native-port-reference.md` carries a dated UNFREEZE block naming what moved, what did
not, and what is outstanding. **`CLAUDE.md`'s own pointer was repointed in the same commit** —
it named the old path, and the always-loaded map is audited by nothing (the C3 lesson).

## §5ad Stage 3 — S5, the biosphere mechanisms: MEASURED 2026-08-26, before any Rust was written

S5 is the last big porting slice and the one where a silent narrowing does the most damage:
**20 Python files, 599 collected tests**, all of them about the plant science itself. Same
shape as §5v — measure first, write the design against what was measured, and only then
write Rust.

### ⚠ The owner column in the "P — port" table above is WRONG, and it points at the wrong file

Every FINDING 5 row names `domains/src/biosphere/flows.rs (0 tests)`. Measured from the Rust
side, `flows.rs` holds **wiring** — 26 `Flow`/`AuxProcess` impls that say which stocks a
transfer moves — and computes almost none of the science. The equations live in
`domains/src/biosphere/science.rs`: **734 lines, 34 public functions, 7 tests**.

Those 7 cover six functions — `captured_water`, `drought_development_factor`,
`extension_rate` (+`root_zone_fraction`), `nitrogen_stress_factor`, `soil_n_availability`.
The other 28 have no direct test: FvCB (`rubisco_limited_rate`, `electron_transport_rate`,
`light_limited_rate`, `gross_leaf_assimilation`, `temperature_factor`,
`canopy_assimilation`), `leaf_area_index`, `q10_factor`, `maintenance_respiration_flux`,
`available_for_growth`, `slope_svp`, `penman_monteith_transpiration`, `transpirable_capacity`,
`fraction_transpirable`, `water_stress_factor`, `soil_water_stress`, `daily_thermal_time`,
`vernalization_day`, `vernalization_factor`, `photoperiod_factor`, `development_stage`,
`partition_fractions`, `partition`, `target_n_concentration`, `ci_from_co2_pool`,
`oxygen_limitation_factor`, `resow_water_return`, `mutual_shading_rate`.

This changes the slice's shape, not its size: a pure-function equation test is cheap, and
most of S5 is that. It also changes where the tests go.

⚠ **A second structural fact the table hides.** Python has extracted per-mechanism modules
(`decomposition`, `humification`, `mineralization`, `microbial_respiration`), and Rust does
**not** — those equations are inline inside `Flow::demand`/`apply` in `flows.rs`. So S5 is
two kinds of work, and they must not be batched together: equation tests against
`science.rs`, and **flow-level** tests that construct the struct and assert its legs for the
soil-carbon and soil-nitrogen families. Testing those at flow level is the default;
extracting them into `science.rs` to make them unit-testable would be a production-code
change smuggled in under a testing slice.

### ⚠ `intercepted_fraction` is dead code, and it is a live trap for this slice

`science.rs`'s Monsi–Saeki one-liner `1 − exp(−k·LAI)` — public, doc-commented, cited — has
**zero callers anywhere in the Rust tree**. The layered-canopy work replaced it with a
three-point Gaussian over canopy depth inside `canopy_assimilation`, which computes its own
`(-k · depth · LAI).exp()` per layer, and the old function was left behind.

The trap: `test_canopy.py`'s 33 tests are about light interception, and the obvious-looking
Rust function to point them at is the one nothing calls. A batch ported that way would
compile, pass, and check nothing. **Recorded as an S6 item** (delete it, or wire it back —
this slice does not decide which).

### The control battery: five mechanisms broken, and what saw it

`cargo test --workspace --no-fail-fast`, Windows, baseline **795 passed / 0 failed**; revert
verified byte-exact. Harness and logs: `M:\claud_projects\temp\s5-control`.

| # | mechanism broken | red | of which **about the mutated mechanism** |
|---|---|---:|---:|
| M1 | `intercepted_fraction` (Beer–Lambert) | **0** | — probe defect: the function is dead |
| M1b | extinction coefficient where the canopy uses it | 7 | **0** |
| M1c | 3-point Gaussian depth weights → a flat average | 4 | **0** — and all four reds are goldens/bands |
| M2 | FvCB co-limitation `min` → `max` | 11 | **0** |
| M3 | Q10 per-10 °C → per-5 °C | 6 | **0** |
| M4 | vegetative DVS scaled by the wrong TSUM | 6 | **0** |
| M5 | mutual shading term dropped | 4 | **1** |

⚠ **The "of which" column is the finding, and the first draft of this table got it wrong.**
It counted reds that name *a* mechanism as coverage of *the mutated* one. Each gate's subject
was then read from its own body: `open_season_canopy_is_physical` is a peak-LAI band,
`open_season_peaks_below_the_greenwood_crossing` is peak biomass against a nitrogen crossing,
`perennial_leaf_cycle_is_a_fixed_point` is a trajectory-convergence claim. None of them is
about photosynthesis, respiration, phenology or interception. **They redden because a broken
equation moves a trajectory and a band somewhere else notices** — which is the same failure
mode as a golden red, "a number moved", wearing a more reassuring name.

⚠⚠ **M1c is the sharpest reading in the battery and it deserves its own sentence.** Replacing the canopy's three-point Gaussian quadrature weights with a flat average — a change to the *numerical scheme* by which light is integrated over canopy depth — reddens **four tests, all four of them committed-byte comparisons**. Not one behavioural gate moved; the peak-LAI band did not notice, and neither did any trajectory check. The canopy's integration scheme is currently guarded by nothing except the goldens, which means it is guarded by nothing that would survive someone regenerating them.

⚠ **Scope of the claim, stated honestly.** The mutations were *selected* from mechanisms the
owner-map work had already shown carry no unit test, so this is not a census of the
reference's testing style; the four mechanisms that do have unit tests were not sampled. What
it establishes is exact: **six live mutations of mechanisms known to lack a unit test, and
five of the six are caught by nothing that is about them.**

### The one genuine direct catch is also the template — S5 invents no convention

M5's `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` calls
`mutual_shading_rate` directly with hand-chosen arguments and asserts exact returns either
side of the threshold — an equation-level check living inside a science gate. Alongside
`drought_development_factor_reproduces_the_sources_worked_examples`, a unit test in
`science.rs` that reproduces the cited source's own worked examples, S5 has **two existing
shapes to copy**. Both are the same idea: evaluate the equation, compare against a number the
source states, not against a number this tree produced.

### ⚠ The trap S5 sets for itself, and the instrument that removes it

Because the goldens and bands redden on *any* numeric movement, a batch of newly-ported tests
that check the wrong thing would leave this battery's output **completely unchanged**. "The
suite went red" is not evidence that an S5 batch works — it is the one thing that would be
true either way.

The fix is structural rather than name-based. `science_gates` and the behavioural tests are
`#[cfg(test)]` modules inside `domains/src/`; the goldens and tolerance bands are integration
tests under `domains/tests/`. So **`cargo test -p domains --lib` is exactly the set to
measure, and it excludes the golden/band noise by construction rather than by discounting it
afterwards.** M2's log times that binary at **5.25 s for 167 tests** against 119 s for the
slow one, so a per-mechanism control costs seconds rather than 25 minutes. One full-workspace
run per *batch* stays as the backstop.

### S5's exit gate, stated forward in S6's terms

As S3 and S4 both did, the gate is written as what must be true when Python is deleted:

1. For every mechanism in the roster below, **mutating that mechanism reddens a test whose
   subject is that mechanism**, measured with `cargo test -p domains --lib` — the goldens and
   bands not merely discounted but out of the binary.
2. Every ported test compares against a number **the cited source states**, a value
   **hand-computed from the cited equation and its parameters with the derivation written in
   the comment**, or an exactly-derivable property (a limit, a threshold either side, a linear
   knot) — never against a value read out of this tree. A test whose expected value came from
   running the reference is a snapshot, and the goldens already do snapshots better. ⚠ The
   middle clause was added after batch A was read: most of the Python literals
   (`32.17540139669239`, `1.3219831112621092`) are hand-computed rather than quoted, which the
   first draft of this gate would have rejected — rejecting the very tests it was written to
   license. A hand-computed pin is legitimate exactly when its derivation is in the comment,
   which is what makes it re-checkable without running anything.
3. A by-name claim census exists, so a mechanism that quietly loses its claim in translation
   is visible as a missing row rather than as a smaller number.
4. `intercepted_fraction` is resolved — deleted or wired — and not merely left unported.

### ⚠ The three argument guards are absent from Rust BY AN EXISTING DECISION, not by drift

`canopy_assimilation` in Python raises on non-positive `ground_area`, non-positive `window_s`
and negative `lai`. The Rust body validates none of them, and six Python tests exist for those
three rules.

⚠⚠ **This was first written up here as a behaviour gap, and that was wrong.**
`science.rs`'s own module header states the decision in as many words: *"The `ValueError`-raising
input guards (`ground_area > 0`, …) are omitted — they never fire for the frozen scenarios and
would force `Result` on hot rate laws; the *behavioral* clamps (`lai == 0 → 0`, `max(0, …)`,
piecewise cutoffs) are kept exactly."* It is a recorded Phase-7 port decision with a stated
reason, not something the reference lost. I found it by reading the file head **after** writing
the finding — the header was eight lines above the code I had already read twice. *Read the
module header before reporting that a module is missing something.*

What follows for S5 is unchanged in substance but different in kind: those six tests get **no
Rust successor, and correctly so**. They are recorded here so that their absence reads as an
inherited decision rather than as this slice narrowing the claim quietly — which is the whole
failure mode S5 exists to avoid.

⚠ One thing in that rationale is worth a later look and is NOT a finding today: *"they never
fire for the frozen scenarios"* is a claim about the roster as it stood in Phase 7, and this
repo has been bitten before by a scope claim outliving the roster that made it true. Cheap to
re-check; it belongs with the guards decision, not inside a testing batch.

### The roster, and the batching

20 files / 599 tests. Batched by **which Rust surface the test lands on**, because that is
what makes a batch reviewable, not by crop or by Python filename:

| batch | Python files | tests | lands on |
|---|---|---:|---|
| A — carbon capture | `test_photosynthesis` 50, `test_canopy` 33, `test_gas_exchange` 15 | 98 | `science.rs` FvCB + canopy functions — ⚠ **WRONG for the third file, corrected in §5ae**: `test_gas_exchange`'s 15 tests are flow-level and land on `flows.rs`. **BUILT 2026-08-26.** |
| B — timing | `test_phenology` 90 | 90 | `science.rs` thermal time, vernalization, photoperiod, DVS |
| C — water | `test_transpiration` 46, `test_soil_layers` 27, `test_water_cycle` 17, `test_root_depth` 16 | 106 | `science.rs` Penman-Monteith, stress, root zone |
| D — carbon spending | `test_allocation` 43, `test_respiration` 25, `test_carbon_budget` 22, `test_stem_reserves` 22 | 112 | `science.rs` partition, Q10, growth budget |
| E — nitrogen | `test_nitrogen` 37, `test_nitrogen_form` 15, `test_nitrogen_throttle` 7 | 59 | `science.rs` target N, uptake, stress — ⚠ **WRONG about two thirds of it, corrected in §5aj**: it lands on FOUR surfaces (`science.rs`, `flows.rs`, `params.rs`, `system.rs`), and 3 of the 59 are batch F's subject. **BUILT 2026-08-26.** |
| F — soil carbon | `test_mineralization` 32, `test_soil_fractionation` 29, `test_decomposition` 19, `test_microbial_respiration` 17, **+3 handed over by batch E** | **100** | **flow-level** — no extracted functions exist. ⚠ Batch E measured the carried-N family (`carried_nitrogen` and the three N legs) guarded by the GOLDENS ALONE; that measurement is this batch's input |
| G — senescence | `test_senescence_form` 37 | 37 | `science.rs` shading + `flows.rs` Senescence |

⚠ **F is the batch that is not like the others** and it is deliberately late: it is the only
one that cannot be written as pure-function tests without changing production code. If it
turns out to need an extraction, that is a separate decision with its own advisor call, not a
thing to slip inside a testing batch.

⚠ **A is not "canopy" despite the name**: `test_canopy.py`'s subject is interception, whose
Rust home is `canopy_assimilation`'s inner loop and **not** the dead `intercepted_fraction`.
Written here because this is exactly where the next reader would go wrong.

## §5ae Stage 3 — S5 batch A BUILT, COMPLETE 2026-08-26: the equations, then the gases

Batch A is the carbon-capture batch: `test_photosynthesis` 50, `test_canopy` 33,
`test_gas_exchange` 15 = 98 Python tests. It landed in two commits, and the second one is
where the roster in §5ad turned out to be wrong.

**Totals.** `cargo test -p domains --lib` 183 → 196; `cargo test --workspace` 795 → 820.
Clippy clean at `--all-targets -D warnings`. **No golden byte moved, no band, no floor, no
manifest** — the whole batch is tests.

### The first half: FvCB + canopy → `science.rs` (12 tests, 7 mutations)

Ported from `test_photosynthesis.py` and `test_canopy.py` onto the co-limitation
functions, the temperature response, the canopy aggregator and `leaf_area_index`. Every
literal is hand-computed from the cited equation with the arithmetic written into the
comment; the params fixture is held as literals rather than read through
`params::photosynthesis()`, so a loader regression cannot silently move a physics pin.

⚠ **The exit gate's clause 2 was widened here, and the reason is worth keeping.** As first
written it demanded a value *the cited source states*. Most of the Python literals
(`32.17540139669239`, `1.3219831112621092`) are hand-computed from the source's equation
rather than quoted from its page, so the clause as drafted would have rejected the very
tests it was written to license. A hand-computed pin is legitimate exactly when its
derivation is in the comment — that is what makes it re-checkable without running anything.

⚠⚠ **The canopy quadrature stopped being golden-only.** §5ad's M1c measured that flattening
the three-point Gaussian depth weights to a flat average reddens **four tests, all four
committed-byte comparisons** — the numerical scheme by which light is integrated over canopy
depth was guarded by nothing that would survive a golden regeneration. It now reddens
`the_depth_quadrature_conserves_photons_against_beer_lambert`, which checks the depth
integral against the closed-form Beer–Lambert total in the linear-response regime.

⚠ **That test's tolerance was wrong first, and is recorded as wrong.** A flat `1e-4` held at
LAI 2.936 and failed at LAI 6, because 3-point Gauss error grows as the sixth power of
`k·LAI`. It is now derived per canopy from the classical n = 3 bound, so the gate tightens
itself on open canopies instead of being set everywhere by its loosest case.

⚠ **`test_canopy.py`'s physics half is mostly about `intercepted_fraction`, which nothing
calls.** Those claims were NOT ported onto the dead function — that is precisely the trap
§5ad names. The live ones land on the quadrature instead, and the resolution (delete it or
wire it back) stays an S6 item.

### ⚠⚠ The roster was wrong about batch A's third file, and the correction is structural

**`test_gas_exchange.py` is not a `science.rs` file.** §5ad's batching table lands all of
batch A on "`science.rs` FvCB + canopy functions". Read from the Python side, that file's
15 tests are **flow-level stoichiometry** — `Allocation`'s O₂ leg, `MaintenanceRespiration`'s
closed loop, `GrowthRespiration`'s netted no-op — plus a sealed-season integration. Its Rust
surface is `flows.rs`, not `science.rs`.

That matters because §5ad also says the two kinds of work "must not be batched together",
and gives that as the reason batch F is deliberately late. So batch A was **mixed all
along** and the roster hid it. The correction, stated so the next reader inherits it rather
than rediscovering it:

* **Batch A is 98 tests over two surfaces**, 83 equation-level and 15 flow-level.
* **Batch F is still the batch that is not like the others** — its distinguishing property
  is not "flow-level" but "flow-level *and* no extracted functions exist to test instead".
  Batch A's flow half has a genuine alternative (the equations it composes are already in
  `science.rs`) and needed no production-code change; F's does not.
* The `owner` column in FINDING 5's table is wrong for `test_gas_exchange` in the same way
  it was wrong for everything else, but in the opposite direction: there it named `flows.rs`
  where the owner was `science.rs`, here §5ad's correction over-applied and named
  `science.rs` where the owner really is `flows.rs`.

### The second half: the gas-exchange third → `flows.rs` + `science.rs` (13 tests, 11 mutations)

Ten flow-level tests in a new `#[cfg(test)] mod tests` **inside `flows.rs`**, and three
equation tests for the chamber seam (`ci_from_co2_pool`, `oxygen_limitation_factor`) in
`science.rs`. The placement is not stylistic: exit-gate clause 1 measures with
`cargo test -p domains --lib`, and integration tests under `domains/tests/` fall out of that
binary while landing in the same one as the goldens and bands — the exact noise `--lib` was
chosen to exclude by construction.

**Five Python tests got no successor, and each absence is a decision rather than a
narrowing:**

| Python test | why no successor |
|---|---|
| ~~`test_allocation_balances_carbon_and_oxygen`~~ | ⚠ **WRONG — corrected below; it now HAS a successor.** The reason given was a step-level claim standing in for a per-flow one |
| ~~`test_maintenance_closed_balances_carbon_and_oxygen`~~ | ⚠ **WRONG — same correction** |
| `test_sealed_conserves_oxygen_exactly` | `assert_conserved` runs every step of every run; a completed sealed run already asserts it |
| `test_sealed_co2_o2_anti_correlate_at_pq1` | with no boundary O₂ stock, `2·(CO₂+O₂) = const` **forces** `ΔO₂ = −ΔCO₂` step for step — it is oxygen conservation restated |
| `test_maintenance_closed_emits_single_pool_leg` | `FlowResult::new` **rejects** a duplicate leg, so the withdraw+deposit pair it rules out is an `Err` in Rust, not a wrong flow |

⚠ **And one whose premise is false in the reference.**
`test_sealed_o2_stays_far_from_rationing` is the *"`f_O2` is deferred"* guard, and its own
docstring says so. `f_O2` is **live** here — `MaintenanceRespiration` and six soil flows all
call `oxygen_limitation_factor` — and the reference's sealed chamber depletes O₂ on purpose
(`system.rs::sealed_chamber_runs_well_fed` asserts the depletion and `rationed == 0`
together, which is the successor claim). The Python file's header prose describing the
deferral is stale against this tree and was **not ported**. *Read a ported file's header as
a dated document, not as a specification.*

### ⚠⚠ §5ae CORRECTED the same day: two of the five "no successor" reasons were wrong, and the audit found a hole

The table above was reviewed after the batch was committed, and it **overstated what had been
measured** in exactly the way this slice exists to prevent. Three fixes landed as a follow-up
(`cargo test -p domains --lib` 196 → 197, workspace 820 → 821, clippy clean, no golden moved).

**1. "Covered by the engine's machinery" was a step-level claim standing in for a per-flow
one — and the biosphere was the one domain that had dropped it.**

What runs every step is `assert_conserved`, which folds **state deltas** across every stock
*after every flow has been applied*. `assert_flow_balanced` is the **local** assertion: this
flow, on its own, moves no net CARBON or OXYGEN. The step-level fold cannot see an imbalance
another flow in the same step cancels, and where it does fire it says "the step drifted",
naming no flow.

The gap was found by **grepping for the assertion rather than reasoning about it**:
`crew`, `eclss`, `power` and `thermal` each call `assert_flow_balanced_default` in their own
in-src tests, and `assert_flow_balanced` appeared **nowhere in the biosphere, in the entire
crate**. So the two `balances_carbon_and_oxygen` tests were not claims the engine already
made — they were the ones the reference's largest domain was missing. They now have a real
successor, `every_gas_flow_balances_carbon_and_oxygen_leg_by_leg`, covering Allocation (open
and sealed), the sealed maintenance burn, and open-field growth respiration.
*Grep for the assertion before recording a claim as covered.*

The two run-level entries in the table (`test_sealed_conserves_oxygen_exactly`,
`test_sealed_co2_o2_anti_correlate_at_pq1`) stand as written — those genuinely are the
step-level claim, and the step-level claim is genuinely asserted every step.

**2. ⚠⚠ A test whose fixture zeroes a stock cannot see a denominator that wrongly includes
it — and mine did.**

`the_sealed_burn_is_split_in_proportion_to_organ_carbon` asserted leaf 0.6 / stem 0.2 / root
0.2 from a 3 : 1 : 1 fixture. The maintenance denominator is `leaf + stem + root`; grain does
**not** pay maintenance. But the fixture's storage organ held **0.0**, so
`leaf + stem + root` and `leaf + stem + root + storage` were the same number, and the test
agreed with both readings.

Measured, not argued: filling storage to 10 mol C and then making the denominator include it
reddens **1 test of 197 — this one, and nothing else in the binary.** Before the fix that
mutation reddened *nothing at all*. A maintenance respiration that charged the grain for the
upkeep of tissue it does not have was invisible to the whole reference. The test now fills
storage, and pins the burn *total* as well as the shares, because that is the same claim
about the same denominator read from the other side.

*The general form, and it is not the same as "test a non-trivial case": ask which stock each
expected number would be unchanged by, and fill exactly those. A zero in a fixture is a
silent case-merge.*

**3. The `f_O2` ratio is now swept rather than pinned at one pair.**

`burn(K)/burn(9K) == 5/9` is exactly derivable **only if** nothing else in the burn depends
on the O₂ amount — a premise the test's comment did not state and could not check. It now
asserts the whole curve: for every multiple `m` of `K` the factor is `m/(1+m)`, so the burn
must be `2m/(1+m)` times the burn at `K`, swept over `m ∈ {½, 2, 4, 9, 100, 2100}`. A single
ratio can be right by coincidence; a curve cannot. ⚠ Honest scope: this is a *coincidence*
argument, not a new measurement — no mutation was found that the pair missed and the sweep
catches, and none is claimed.

**⚠ What this correction says about the batch's own method.** All eleven of the original
mutations reddened a named test, and the batch read as finished. But a mutation battery
proves the tests are **reachable and sensitive**; it cannot prove an expected value is
**right**, because a test that encodes a misreading is sensitive to that misreading. Both
real defects here were found by *reading the fixture against the code* — asking which stock
each pinned number would be unchanged by — and one of them was found only because someone
asked what `biomass` meant. **A green battery is evidence about the instrument, not about the
arithmetic.**

### The mutation battery: 11 mutations, 11 named reds, and four caught by nothing else

`cargo test -p domains --lib --no-fail-fast`, Windows, baseline **196 passed / 0 failed**;
both files verified byte-exact against pristine copies afterwards. Harness and logs:
`M:\claud_projects\temp\s5-batch-a`.

⚠ The harness reverts by **copying a pristine snapshot**, never `git checkout` — the files
under mutation held uncommitted work, and `git checkout` would have deleted the batch. Same
trap S1 recorded; it was in the first draft of this harness and caught before it ran.

| # | mechanism broken | new test that bit it | other reds |
|---|---|---|---:|
| M1 | the `f_O2` throttle dropped from the sealed burn | `the_sealed_burn_is_throttled_by_f_o2_in_the_michaelis_ratio` (+ the partial-deficit case) | **0** |
| M2 | `covered = min(GASS, MRES)` → `max` | four of the new flow tests | 1 |
| M3 | organ split → equal thirds | `the_sealed_burn_is_split_in_proportion_to_organ_carbon` | 2 |
| M4 | `organ_total` forgets storage | `allocation_releases_one_oxygen_per_carbon_fixed_across_all_four_organs` | 20 |
| M5 | the O₂ leg becomes unconditional | `allocation_in_the_open_field_emits_no_oxygen_leg` | 7 |
| M6 | the sealed context falls back to the Ci forcing | `the_sealed_context_reads_ci_from_the_pool_and_not_the_forcing` | 10 |
| M7 | the Michaelis denominator `K+x` → `K+2x` | `oxygen_limitation_is_michaelis_and_half_saturates_at_k` (+ M1's test) | 1 |
| M8 | `ci_ratio` dropped (Ci becomes Ca) | `ci_from_a_finite_pool_is_the_mole_fraction_times_the_ci_ratio` | 6 |
| M9 | the negative-O₂ clamp removed | `a_negative_oxygen_amount_clamps_to_zero_rather_than_reversing_the_sign` | **0** |
| M10 | the open-field maintenance branch routed through the sealed one | `open_field_maintenance_draws_the_covered_part_from_the_atmosphere` | **0** |
| M11 | `GrowthRespiration` empty on both branches | `sealed_growth_respiration_is_an_empty_round_trip` | **0** |

⚠⚠ **M1 is the finding, and it is §5ad's M1c wearing a different mechanism.** Dropping the
O₂ self-limit from plant maintenance respiration — deleting a whole feedback, not perturbing
a coefficient — was run at **workspace scope** to price it honestly. It reddens
`every_cheap_station_golden_is_inside_its_measured_band`,
`every_cheap_station_golden_is_still_this_reference_s_output`,
`every_classified_domains_golden_is_inside_its_measured_band`,
`every_domains_golden_is_still_this_reference_s_output` — **four committed-byte and band
comparisons, and nothing else in 820 tests.** Not one science gate, not one behavioural
check, not one liveness floor. The mechanism was golden-only, exactly as the canopy
quadrature was, and for the same structural reason: a broken equation moves a trajectory,
and a snapshot notices that a number moved without ever having been about the mechanism.

### The design rule this batch adds: **mutate against the balance machinery, not with it**

Most of `test_gas_exchange.py`'s claims are stoichiometric identities, and in this engine a
stoichiometric identity is what `assert_flow_balanced` and `assert_conserved` already check.
A mutation that drops or sign-flips an O₂ leg unbalances OXYGEN, so its red comes from the
conservation machinery — which is golden-loan coverage under a more reassuring name.

The discriminating mutations are the ones that leave **every conserved quantity balanced**:

* a **magnitude** change that scales the whole transfer (M1 — organs, pool and O₂ all move
  by the same factor, so PQ = 1 still holds exactly and balance is untouched);
* a **distribution** change that redivides a fixed total (M3);
* a **branch** change that swaps one balanced leg set for another balanced leg set (M2, M10,
  M11);
* a **routing** change that reads the right number from the wrong source (M6).

Every test in the flow half is written against one of those four, and the module's own header
records which of the ported claims are balance-immune and which are balance-restated. The
three that are restatements are ported anyway — with the redundancy written down — because
their independent content is the organ *roster* and the branch *condition*, not the ratio.

**Applies forward to batch F**, whose subject is the same shape: the soil-carbon flows are
`{C, O}` transfers whose leg sums are forced. *Before writing a flow-level test, ask what it
still asserts once the balance check is removed. If the answer is "nothing", the test is a
second copy of `assert_conserved`.*

### What batch A leaves standing

* **Batches B–G unchanged in scope**, with the roster row for A corrected above.
* **`intercepted_fraction` still unresolved** (S6 item; clause 4 of the exit gate).
* **The exit gate's clause 3 — the by-name claim census — is not yet written.** It is a S5
  exit artefact, not a per-batch one, and the three `science.rs` seam tests are the first
  entries that must be marked as *additional* coverage rather than as successors: they have
  no Python ancestor in batch A's files at all.
* **No Python deleted.** All three files stay green and running until S6.

## §5af Stage 3 — S5 batch B BUILT, COMPLETE 2026-08-26: the timing batch, and a branch no scenario reaches

Batch B is the timing batch: `tests/test_phenology.py`, 90 tests over the cardinal-capped
degree-day rate, the two-phase DVS ramp, vernalization, photoperiod, drought acceleration,
the two aux accumulators and the config boundary.

**Totals.** `cargo test -p domains --lib` 197 → **221**; `cargo test --workspace` 821 → **845**.
Clippy clean at `--all-targets -D warnings` (it caught one thing — see the review pass).
**No golden byte moved, no band, no floor, no manifest** — asserted by
`git status --porcelain rust/data/` returning empty, not inferred from a green suite. The
The batch is tests plus **one production change, taken as an explicit decision rather than
slipped in** — the three injectable loader readers below.

### The before-battery, and it is worse than batch A's

Eight live mutations against `cargo test -p domains --lib` (197 tests). Harness and logs:
`M:\claud_projects\temp\s5-batch-b`.

| # | mechanism broken | red | of which **about the mutated mechanism** |
|---|---|---:|---:|
| B1 | uncap `daily_thermal_time` at `t_cap` | **0** | — |
| B2 | `development_stage` reproductive divisor → TSUM1 | **0** | — |
| B3 | drop `development_stage`'s 2.0 cap | **0** | — |
| B4 | flip `vernalization_day`'s upper ramp | **0** | — |
| B5 | drop `vernalization_factor`'s clamp | **0** | — |
| B6 | `photoperiod_factor` long-day → short-day | 3 | **0** |
| B7 | drop the photoperiod multiply in the accumulator | 3 | **0** |
| B8 | drop the vernalization multiply in the accumulator | 3 | **0** |

⚠ **Five of eight reddened nothing at all**, and the three that did reddened the *same three
tests* every time: `open_season_canopy_is_physical` (a peak-LAI band),
`the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` (a shading-regime check) and
`perennial_leaf_cycle_is_a_fixed_point` (a trajectory-convergence claim). Not one is about
photosynthesis timing, cold, or daylength. They redden because a broken development rate
moves a trajectory and a band somewhere else notices — §5ad's finding, reproduced on a
second batch with a different mechanism set.

⚠ **The instrument was checked before the reading was believed.** All eight logs carry a
`test result:` line with 197 collected, so none of the five zeroes is a silent build
failure — the trap that would read exactly like "the mechanism is covered".

### ⚠⚠ A second probe separated "untested" from "unreachable", and they are NOT the same defect

A zero-red mutation has two possible causes and the batch's response differs completely:
the branch runs and nothing checks its value (write a test), or the branch never runs at all
(a test can pin it, but only a *scenario* can exercise it). Replacing each branch body with a
`panic!` and re-running measures which:

| branch | tests that ENTER it |
|---|---:|
| `development_stage` reproductive phase | 23 |
| `development_stage`'s `DVS = 2` cap (raw value > 2) | 20 |
| `vernalization_day`'s upper ramp (8–12 °C) | 20 |
| `vernalization_factor`'s clamp (raw value outside [0, 1]) | 20 |
| **`daily_thermal_time`'s `t_cap` plateau** | **0** |

⚠⚠ **The `t_cap` plateau is entered by ZERO tests of the entire workspace, goldens
included** — the probe was re-run under `cargo test --workspace` and the run stayed green
with a `panic!` in that branch. No scenario in the tree is ever 30 °C warm. The cardinal cap
on degree-day accumulation is therefore not merely untested, it is *unexercised*: batch B
pins the branch as a function, and the remaining half — that no run ever needs it — is
recorded as a finding below rather than fixed inside a testing batch.

### Where the tests went, and why the batch spans four files

The roster row said "lands on `science.rs`". As with batch A's third file that is true of the
majority and not of the batch, and the split is by SUBJECT, not by convenience:

| file | tests | subject |
|---|---:|---|
| `science.rs` | 8 | the five rate laws as pure functions: the GDD cardinals, the two-phase DVS ramp, VERDAY's three segments and the source's own worked example, VERFUN's load-bearing clamp, PPFUN's long-day direction, and the memory-vs-no-memory contrast between the two vegetative modifiers |
| `flows.rs` | 6 | the two accumulators as PROCESSES: increment form, what each reads through `env`, the multiplicative composition, the complete arrest of an unvernalized qualitative cultivar, and the anthesis gate **on its boundary** |
| `system.rs` | 1 | the WIRING: `wssd: None` declines the drought modifier, on an off-default plot |
| `params.rs` | 9 | the config boundary: the committed block as the two cited "Winter Europe" rows, the three reader-level schema guards, and — after the decision below — the five that reach the semantic guards |

⚠ **The aux tests are in `flows.rs`, where the structs live, but two aux-level claims already
had successors in `system.rs`** (`wsfd_uses_wssg_and_is_not_gated_off_at_anthesis`,
`drought_acceleration_is_wired_into_the_accumulator_and_no_scenario_shows_it`). They were
NOT duplicated, and the split is written into both files so the census reads as one roster
rather than as a gap.

⚠ **The wiring test EVALUATES rather than inspecting fields.** Python asserts
`proc.drought is None`, because reaching into a built registry is what it can do; Rust cannot
downcast a `Box<dyn AuxProcess>` at all, and the alternative turned out to be the better
claim — build the season, evaluate the accumulator on a hand-built bone-dry root zone, and
assert the increment is the plain rate. A field can be right while the arithmetic that reads
it is wrong; the increment cannot.

### The after-battery: 14 mutations, 14 subjects

Every mutation now reddens a test whose subject **is** the mutated mechanism (the three
unrelated bands still redden alongside, and are not counted as coverage):

| # | mechanism broken | the test that is ABOUT it |
|---|---|---|
| B1 | uncap the GDD rate | `thermal_time_is_the_degree_day_rate_capped_at_both_cardinals` |
| B2 | reproductive divisor → TSUM1 | `development_stage_is_the_two_phase_tsum_ramp` |
| B3 | drop the `DVS = 2` cap | `development_stage_is_the_two_phase_tsum_ramp` |
| B4 | flip VERDAY's upper ramp | `vernalization_day_is_the_three_segment_cold_response` |
| B5 | drop VERFUN's clamp | `vernalization_factor_arrests_a_qualitative_cultivar…` (+ both arrest tests) |
| B6 | PPFUN long-day → short-day | `photoperiod_factor_is_the_long_day_response…` |
| B7 | drop the photoperiod multiply | `thermal_time_aux_multiplies_the_two_vegetative_modifiers` |
| B8 | drop the vernalization multiply | `thermal_time_aux_arrests_completely_when_unvernalized` |
| B9 | modifiers ADD instead of MULTIPLY | `thermal_time_aux_multiplies_the_two_vegetative_modifiers` |
| B10 | anthesis gate `< 1` → `<= 1` | `the_vegetative_modifiers_are_gated_off_at_and_after_anthesis` |
| B11 | daylength read as hours (drop `/3600`) | `thermal_time_aux_multiplies_the_two_vegetative_modifiers` |
| B12 | WSFD moved inside the vegetative branch | `wsfd_uses_wssg_and_is_not_gated_off_at_anthesis` (pre-existing) |
| B13 | wire the drought modifier unconditionally | `the_wiring_declines_the_drought_modifier_when_no_wssd_is_cited` |
| B14 | wiring hardcodes a 1 m² plot | `the_wiring_declines_the_drought_modifier_when_no_wssd_is_cited` |

⚠ **B10 and B11 are the two the port could plausibly get wrong and the Python file does not
test at all.** B10 is the gate's *boundary*: `DVS == 1.0` exactly at anthesis, and
`is_vegetative` tests `< 1.0`, so the modifiers switch off AT anthesis rather than after it —
the successor asserts the plain rate at `tsum_anthesis` and the complete arrest one degree-day
earlier, which is what makes it a claim about the gate and not about the weather. B11 is the
UNIT seam: the forcing carries daylength in seconds and the accumulator divides by 3600, so
feeding hours reads a near-zero day and clamps PPFUN to nothing. Neither is in
`test_phenology.py`; both are the reference's own hazards and are additional coverage, not
successors.

⚠ **One candidate mutation was dropped because it is not a bug.** Opening VERDAY's closed
boundaries (`<=`/`>=` → `<`/`>`) is arithmetically inert: at the base and at the ceiling both
ramps evaluate to exactly zero, so the two forms agree everywhere. Recorded so nobody adds a
test for it — the same shape as `captured_water`'s symmetric factors.

### What got NO successor, and why each absence is a decision

| Python test(s) | why no successor |
|---|---|
| `test_daily_thermal_time_rejects_inverted_band`, `test_development_stage_rejects_non_positive_sums`, `test_vernalization_day_rejects_ill_ordered_cardinals`, `test_vernalization_factor_rejects_bad_params`, `test_photoperiod_factor_rejects_bad_params` (9 tests) | the `ValueError`-raising input guards are omitted from `science.rs` **by the recorded Phase-7 decision in its own module header** — they never fire for the frozen scenarios and would force `Result` on hot rate laws. The ordering/positivity rules moved to the LOADER, which is where the successor claim lives (and where it is now blocked; see below) |
| `test_aux_process_satisfies_protocol`, `test_vernalization_aux_satisfies_protocol` | the trait is a compile-time obligation in Rust: a type that does not implement `AuxProcess` cannot be put in the aux vector. The compiler is the census, not a test |
| `test_constant_temperature_season_accumulates_to_rate_times_n_dt`, `test_derived_dvs_tracks_the_accumulator` | ENGINE claims (aux advances once per step at the step-entry snapshot, under **both** integrators) wearing a phenology costume. ⚠ **Grepped, not reasoned** — §5ae's correction was exactly this row's shape, so the successors were named before the row was written: `simcore/tests/aux_channel.rs` carries `a_constant_rate_process_accumulates_to_rate_times_n_dt_under_euler`, `..._under_rk4` and `all_four_rk4_stages_see_the_step_entry_aux`. The RK4 half is the non-obvious one — four flow evaluations per step, one aux advance — and it is covered |
| `test_potato_declines_wssd_because_the_source_has_no_potato_row` | ⚠ **a GAP, not a decision** — the Rust roster has no potato build at all (`params.rs` records its stage 2 as deferred), so the crop-specific half cannot be ported. The RULE it rests on (`wssd: None` declines the modifier) IS ported |
| `test_phen_loader_rejects_inverted_cardinal_band`, `test_phen_loader_rejects_non_positive_sum` (3 tests) | **now ported** — the decision above made them reachable |

### ⚠ The one production change, and it was DECIDED rather than assumed

`phenology()`'s `t_base < t_cap` assertion, `vernalization()`'s cardinal-ordering assertion
and the `tsum` positivity check all read `PHENOLOGY_YAML` through `include_str!`. The only
file they could ever see was the committed one, which is valid, so **they were inert**:
removing any of the three left `cargo test -p domains --lib` at 216 passed / 0 failed, against
a live control — declaring `t_base` in kelvin, which the committed file is not, reddened 29.
*A guard that cannot be handed a bad file is a comment.*

The three READER-level guards (wrong unit, unknown field, missing `source`) were already
reachable, because `ParamFile::parse` and `guarded_set` take text. The three SEMANTIC ones
needed text-injectable variants of the readers — the shape `allocation_from` already uses **in
the same file**. That is a production change inside a testing batch, which batch A's rule says
is a decision of its own; it was **put to the user in plain terms and answered "build it"**,
the alternative being five Python tests dying at S6 with no successor.

Shipped: `phenology_from`, `vernalization_from`, `photoperiod_from` and a shared
`phenology_block_from`, each public reader now a one-line call at `PHENOLOGY_YAML`. **No
committed load changed** — `--lib` stayed at 216 across the split before the five tests were
added. The after-battery says the guards now have teeth:

| guard dropped | red | the test that is ABOUT it |
|---|---:|---|
| `t_base < t_cap` | 1 | `an_inverted_phenology_cardinal_band_is_rejected` |
| `tsum > 0` (both) | 1 | `a_non_positive_thermal_sum_is_rejected` |
| vernalization cardinal ordering | 1 | `ill_ordered_vernalization_cardinals_are_rejected` |
| `vdsat > 0` | 1 | `a_bad_vernalization_sensitivity_or_saturation_is_rejected` |
| `vsen >= 0` | 1 | `a_bad_vernalization_sensitivity_or_saturation_is_rejected` |
| `cpp > 0` | 1 | `a_bad_photoperiod_pair_is_rejected` |
| `ppsen >= 0` | 1 | `a_bad_photoperiod_pair_is_rejected` |
| *control:* `t_base` declared in K | 35 | — (the live unit guard, unchanged) |

⚠ **Each rejection test also asserts the LEGAL boundary case**, because a guard tuned one
notch too tight forbids a real crop rather than a bad file: `vsen == 0` and `ppsen == 0` are
the day-neutral cultivar, which the tree ships, and both must still load.

### ⚠ The review pass caught three overclaims and one lint, and they are the batch's own failure mode

Written down because all four are the shape this slice exists to prevent — a comment that
says more than its test measures.

1. **The "engine owns it" row was reasoning, not grepping** — §5ae's correction, four hours
   old, reappearing. Discharged by naming the three `simcore` tests above.
2. **`thermal_time_aux_without_a_cited_wssd_ignores_a_bone_dry_root_zone`'s comment claimed
   it would catch an unconditionally-wired modifier.** It would not: the test hand-builds
   `drought: None`, so `build_plants` never runs, and B13's log confirms that mutation
   reddens the two `system.rs` tests and leaves this one green. What it actually guards is
   `drought_factor`'s `let-else` early return. The comment now says so.
3. **The three trailing ordering `assert!`s in the params test read as coverage of the
   loader's guard.** They are a RESTATEMENT of the rule against the committed values —
   deleting the guard leaves them green. Now labelled, so the census does not count them.
4. **Clippy rejected `assert!(1.0 - 0.09 * 16.0 < 0.0)`** as always-true. It was a comment
   doing work as an assertion; it is now written against the same `cpp`/`ppsen` bindings the
   test uses, which is the better claim anyway.
5. ⚠⚠ **The `rejects()` test helper suppressed the panic hook, and `set_hook` is
   PROCESS-GLOBAL.** Cargo runs these on parallel threads, so two concurrent calls interleave
   — A installs the no-op, B takes the *no-op* as its "previous", A restores the real hook,
   B restores the no-op — and every panic for the rest of the run prints nothing. It cannot
   cause a false pass; it silently destroys some OTHER test's failure message in some later
   run. Removed — the backtraces are noise, and noise is the correct price, which is what the
   `allocation_from` precedent had been doing all along. *The divergence from the precedent
   was the defect, and it was invisible because the tests still passed.*

### Findings recorded, not fixed

1. ⚠⚠ **The `t_cap` plateau is unexercised workspace-wide** (above). Either a scenario should
   reach it or the cap is decoration; both answers are science decisions, not testing ones.
2. ⚠ **A test-local constant shadows a real one with a DIFFERENT value.** Batch A's test
   module in `flows.rs` declares `const ROOTED_DEPTH: &str = "biosphere.rooted_depth"`, while
   the engine's `stocks::ROOTED_DEPTH` is the bare `"rooted_depth"`. It is harmless where it
   sits — those tests are self-consistent — but an aux test that inherited it would read an
   aux key nothing writes and pass on a `unwrap_or(0.0)` default. Batch B's block imports the
   real constants and says why, in the file.
3. ⚠ **Two doc comments are attached to the wrong item.** `science.rs`'s
   "Development stage `DVS ∈ [0, 2]`…" line sits on `root_zone_fraction` (and
   `development_stage` itself carries none), and `flows.rs`'s "`AuxProcess` advancing the
   `vernalization_days` accumulator" sits on `RootDepthExtension` (and
   `VernalizationAccumulation` carries none). Doc-only, no behaviour, deliberately not
   touched inside a testing batch.
4. ⚠ The Phase-7 header's rationale for omitting the input guards — *"they never fire for the
   frozen scenarios"* — is a claim about the roster as it stood then, and §5ad already flagged
   it as cheap to re-check. Finding 1 is the first evidence that a roster-dated claim about
   this file has actually drifted: the `t_cap` branch is now not merely unfired but
   unreachable.

### What batch B leaves standing

* **Batches C–G unchanged in scope.** C (water) is next by the roster.
* ⚠⚠ **The `t_cap` plateau is a SCIENCE question with no owner yet** — does a scenario
  need to get that hot, or is the cap decoration? Queued here as well as recorded in the
  findings, because a finding that lives only in a findings list is owned by nothing.
* **`intercepted_fraction` still unresolved** (S6 item; clause 4 of the exit gate).
* **The by-name claim census (clause 3) still unwritten** — now with batch B's own additional
  coverage (B10's gate boundary, B11's unit seam, the two `science.rs`/`params.rs` claims with
  no Python ancestor) to mark as *additional* rather than as successors.
* **No Python deleted.** `test_phenology.py` stays green and running until S6.

## §5ag Stage 3 — S5 batch C BUILT, COMPLETE 2026-08-26: the water batch, and six branches nothing in the tree can reach

Batch C is the water batch: `test_transpiration` 46, `test_soil_layers` 27,
`test_water_cycle` 17, `test_root_depth` 16 = **106 Python tests**. It lands on five
surfaces across **two crates**, and the second crate is a correction to the roster, not a
convenience — see "Where the tests went" below.

`cargo test -p domains --lib` **221 → 257**; `cargo test -p station --lib` **53 → 60**;
`cargo test --workspace --no-fail-fast` **845 → 888**; clippy clean at `--all-targets -D
warnings`. **No golden byte, band, floor or manifest moved** — asserted with
`git status --porcelain rust/data/` (empty), not inferred from a green suite.

The batch is tests plus **one production change**, taken as an application of batch B's
already-given answer rather than as a fresh decision. Harness and all 23 logs:
`M:\claud_projects\temp\s5-batch-c`.

### The before-battery: sixteen mutations, ONE caught by a test about its own mechanism

Baseline 221 passed / 0 failed. §5ad predicted this batch would come back *mixed* rather
than uniformly bare, because four of `science.rs`'s six already-tested functions are
water/root ones. It did, and the split is the finding.

| # | mechanism broken | red | of which **about the mutated mechanism** |
|---|---|---:|---:|
| M1 | `slope_svp` drops the `SVP_C` factor | **0** | — |
| M2 | Penman–Monteith drops the `(1 + r_s/r_a)` canopy term | **0** | — |
| M3 | Penman–Monteith drops the negative-energy clamp | **0** | — |
| M4 | `transpirable_capacity` drops `ground_area` | 3 | **1** |
| M5 | `FTSW` zero-capacity limb returns 1 instead of 0 | **0** | — |
| M6 | `WSFG` uncapped above 1 | 15 | **0** — every one a compensation-point or leaf-cycle gate |
| M7 | `WSFG` gains a hard wilting floor at 0.05 | 1 | **0** |
| M8 | `soil_water_stress` hardcodes unit area in the denominator | 2 | **0** |
| M9 | `root_zone_fraction` uncapped above 1 | **0** | — |
| M10 | `root_zone_fraction` drops its non-positive-depth guard | **0** | — |
| M11 | `resow_water_return` drops its zero-depth guard | **0** | — |
| M12 | **`Transpiration` drops `f_water` entirely** | 1 | **0** |
| M13 | `Irrigation` drops the deficit limb | 1 | **0** |
| M14 | condensation rate doubled | **0** | — |
| M15 | `Recycling` reads `soil_water` instead of `condensate` | 9 | **0** |
| M16 | the dry-subsoil stop tests `< 0` instead of `<= 0` | 1 | **1** |

**Eight of sixteen reddened nothing at all.** Of the eight that did, exactly two reddened
a test whose subject is the mutated mechanism, and M4's "1" is the ground-area pin batch
`soil-layers` wrote for a different call site.

⚠⚠ **M12 is the sharpest reading and deserves its own sentence.** Deleting the soil-water
stress factor from transpiration — a plant that transpires as if it were never
water-limited, a whole feedback removed rather than a coefficient perturbed — reddened
**one test in 221**, and that test is about drought-*accelerated* phenology. Same shape as
batch A's canopy quadrature and batch B's photoperiod, one mechanism over.

⚠ **M15 is the balance lesson from batch A, reproduced.** Making `Recycling` first-order in
`soil_water` instead of `condensate` keeps every leg balanced and every conserved quantity
conserved, so conservation cannot see it; it reddened nine tests, every one a chamber or
compensation-point gate. *A balanced mutation is invisible to the balance machinery by
construction — only the rate law itself can catch it.*

### ⚠⚠ The second probe again, and this time SIX branches are unreachable

Batch B's `panic!`-per-silent-branch probe, which separates "the branch runs and nothing
checks it" (write a test) from "the branch never runs" (only a *scenario* can reach it).
Nine probes on `--lib`; the three that fired were `root_zone_fraction`'s saturation limb
(20 tests), and both `Irrigation` limbs (7 each) — live but unasserted.

The other six fired in **zero tests of `--lib`**, and were re-run together under
`cargo test --workspace --no-fail-fast`, which **stayed fully green, goldens included**:

* `fraction_transpirable`'s zero-capacity limb,
* `root_zone_fraction`'s non-positive-depth guard,
* `resow_water_return`'s zero-old-depth guard,
* `resow_water_return`'s nothing-abandoned limb,
* `water_stress_factor`'s exactly-empty-zone limb,
* **`penman_monteith_transpiration`'s negative-energy clamp.**

#### The clamp's own stated rationale is false in this tree, and the reason is structural

`test_penman_monteith_clamps_negative_radiation_to_zero`'s comment justifies the clamp by
saying daily-average net radiation *"goes negative on short midwinter days (the winter-wheat
season overwinters)"*. Measured here it never does, and not because the winter is mild:
`weather::net_radiation` is **net SHORTWAVE only** — `(1 − α)·IRRAD/86400`, with no longwave
loss term — so it is non-negative for every non-negative irradiance, and
`vapor_pressure_deficit` is itself a `max(0, …)`. Both drivers of `λE` are non-negative at
every call site in the tree, so the clamp is unreachable **by construction**, not by luck.

The clamp is kept and pinned at the function's own contract (it is `pub`, and a longwave
term is the obvious next weather science), and the **unreachability is asserted over the
committed weather rather than left in a comment that could rot**. Adding a longwave term is
a science question, recorded rather than taken inside a testing batch.

*The general form, and it is batch B's with one turn more: a probe tells you a branch never
ran. Reading WHY it never ran is what turns a coverage gap into a finding — here the answer
was eight lines away in a different module, and it falsifies a sentence the Python test has
carried since Phase 1.*

### Where the tests went, and why the batch spans two crates

§5ad's roster row says batch C "lands on `science.rs` Penman-Monteith, stress, root zone".
That is true of the largest third and wrong about the rest — the same correction batch A
and batch B each had to make, for the third time. The split is by SUBJECT:

| surface | tests | subject |
|---|---:|---|
| `domains/src/biosphere/science.rs` | 11 | the equations: SVP + its analytic slope, the PM combination equation, `TTSW`/`FTSW`/`WSFG`, the composed stress, `FROOT1`, the re-sow return |
| `domains/src/biosphere/flows.rs` | 7 | the flows: `Transpiration`'s three factors, `Irrigation`'s two limbs, the two cycle flows' rate law |
| `domains/src/biosphere/params.rs` | 9 | the three water param files + their guards |
| `domains/src/biosphere/system.rs` | 9 | the season: the extension law, the flowering stop, the re-sow, the access gate, the deep-water rescue, the ring, sealed water conservation |
| `station/src/scenario.rs` + `harvest.rs` | 7 | the scenario **census**, and the past-anthesis injection |

⚠ **The station half is forced, not chosen.** The `SeasonScenario` roster is split across
two crates and `domains` cannot see `station`, so `station` is the only place both halves
are visible. A census that enumerated only the biosphere's four would miss the station's
four and the harvest injection — which is precisely the scenario the Python test was
written after.

### ⚠⚠ The census had to change SHAPE, and porting it as a list would have lost the claim

`test_every_scenarios_water_stores_are_geometric` enumerates by reflection (`dir(module)`),
and its own comment states why: *a hand-listed roster silently omits the scenario added
after it was written* (`coverage-roster-is-not-the-manifest`). Rust has no reflection, and a
literal array in the test would reproduce exactly the failure the original exists to
prevent — green while covering less, with nothing to say so.

The successor is a **source scan with two controls**, the shape
`params::tests::the_census_matches_the_directory_on_disk` already uses in this codebase:

1. `the_scenario_roster_matches_what_the_source_declares` scans the **production half** of
   `system.rs` and `scenario.rs` (everything before `#[cfg(test)]`) for both declaration
   shapes — `const NAME: SeasonScenario =` and `fn name() -> SeasonScenario {` — and
   asserts the checked roster equals the scan, as sets.
2. `a_whole_file_scan_finds_more_than_the_census_does` proves the scanner is not blind: the
   whole-file scan picks up the test modules' diagnostics, batch C's own
   `deep_water_scenario` among them.
3. `the_census_comparison_reddens_on_an_unlisted_scenario` proves the comparison bites.

Measured, not argued: declaring a new production `SeasonScenario` in `station/src/scenario.rs`
reddens **exactly one test of 60**, the census.

⚠ Cutting at `#[cfg(test)]` is load-bearing rather than tidy. Test modules declare
scenarios (the retired `WATER_BITING` and `N_LIMITED` copies, and this batch's own
diagnostics); those never build a shipped run, so they owe no identity — but a scan that
counted them would make the census un-satisfiable and it would be *loosened* rather than
fixed.

### The three Python scenarios with no Rust roster entry

`DEEP_WATER`, `DROUGHT` and the retired `WATER_BITING` do not exist on this side. The
diagnostics **declare their subject inline** — the shape
`nitrogen_limitation_is_wired_into_assimilation_and_no_scenario_shows_it` already uses in
`system.rs` — rather than being added to the production roster, where a new
`SeasonScenario` would stand in front of the freeze manifest for no reference gain.

**The clean control needed no production seam, and that was checked rather than assumed.**
The deep-water headline claim needs a run with *only* `RootZoneCapture` removed;
`Registry` does not lend out its owned flows, so the obvious reading is that Rust cannot
have the control. But `compartments()` is module-private and the test module is *inside*
that module, so the flow list is reachable before `Registry::new` closes over it. ⚠ The
naive control (`soil_extractable_water = 0`) is the one the Python file records as having
been **destroyed silently** by the geometry re-basing — `EXTR` now appears in `TTSW` as
well as in the transfer, so a zero kills the crop instead of isolating the transfer. *A
control that changes more than it claims is worse than no control.*

#### The cross-port reading the deep-water pin produced

Measured on this port, first, before looking at Python's bands: leaf **9.5775×**, grain
**5.8281×**. Python's current pins are `9.0 < r < 10.5` and `5.5 < r < 6.2` — bands it
arrived at through two independent re-measurements. The Rust numbers sit inside both. That
agreement is a **cross-port reading, not a copied literal**, and it is worth recording
because this ratio has moved twice for reasons unrelated to what it measures (WSFD, then
the depth-resolved canopy), each time with nothing red because the bound had slack.

### ⚠ The production change: three loader guards that could not be handed a bad file

`transpiration()`, `root_depth()` and `water_cycle()` all read their YAML through
`include_str!`, so their six guards could only ever see the committed file — which is valid.
**Measured inert exactly as batch B measured phenology's**: deleting the whole guard loop
from any of the three left `cargo test -p domains --lib` at **221 passed / 0 failed**,
against a live control (declaring `aerodynamic_resistance` in `min/m`) that reddened **25**.

`transpiration_from` / `root_depth_from` / `water_cycle_from` ship, the `allocation_from`
shape already in that file, and `--lib` was **unchanged at 239 across the split**. All three
guards now redden exactly the test about them.

⚠ **This was NOT re-asked, and the reason is recorded rather than assumed.** Batch B put
the identical question to the user in their own terms and was answered "build it"; three
files is a larger surface, not a different decision, and the alternative is the same — ten
Python rejection tests dying at S6 with no successor. What batch B's finding forbids is
letting *"recorded in three places"* stand in for *asking*; it does not require re-asking a
question answered the same day about the same shape. Stated here and in the commit as an
application of that answer, so it can be reversed on sight.

⚠ Every rejection test also pins the **LEGAL** boundary. `water_cycle`'s is the sharp one:
the guard is `require_non_negative`, not `require_positive`, and the asymmetry is a design
decision — **a zero rate is how a chamber with no condenser is declared**, which is every
open-field scenario in the tree, since the ring exists only in the sealed branch. A guard
tuned one notch tighter would forbid the frozen roster.

### The after-battery: 16 + 3 + 4, every one now caught by a test about itself

All sixteen mutations re-run against the finished batch, plus the three guard deletions and
four station-side mutations (a new unrostered scenario, a rostered scenario losing its
geometry, the harvest injection dropping its water re-derivation, and the harvest depth
ceasing to track the crop cap). **Every one reddens a test whose subject IS the mutated
mechanism.** Six of the 23 redden exactly one test in the whole binary.

⚠ **The instrument was checked before the readings were believed.** All 19 domain logs were
confirmed to carry a `test result:` line collecting **257**, and all four station logs
collecting **60** — a mutation that fails to COMPILE produces zero "FAILED" lines, which
reads identically to "nothing noticed", and this batch's before-half rests on eight such
zeroes. The tree was restored from pristine copies and verified byte-exact by sha-256 after
every battery, never with `git checkout --` (batch A's near-miss).

### Two pins that are the port's own hazards, with no Python ancestor

* **The anthesis boundary of the flowering stop.** `is_vegetative` tests `DVS < 1.0`, so at
  `thermal_time == tsum_anthesis` exactly the stop must already have fired. The Python test
  checks `+1.0` only. Same boundary batch B pinned for the anthesis gate.
* **The per-STEP extension ceiling.** The engine runs at `dt = ¼`, so the largest legal
  depth increment is a *quarter* of the daily rate. A test comparing against the daily rate
  would pass on a build that had dropped `dt` from the accumulator entirely — the
  days-vs-steps unit trap this repo has already been bitten by once.

### The one Python literal that was NOT copied, and why

`test_penman_monteith_pinned_value` asserts `6.158958394549651` and its own comment calls it
a *"pinned regression literal"* — a value read out of the tree, which S5's exit gate
(clause 2) rejects outright. The successor **hand-composes the combination equation in the
comment and asserts every intermediate separately** (`e_s = 2338.2813 Pa`,
`Δ = 144.7462 Pa/°C`, aero `= 24413.3 W/m²`, denominator `= 305.5462`, `λE = 174.6464 W/m²`,
`= 6.1590 mm/day`), which reproduces the same number while making it re-checkable without
running anything. The FAO-56 `e_s` table (0.6108 / 1.2280 / 2.3383 / 4.2431 kPa) ports as-is
— it is genuinely external — and the file's own honest scope note survives with it: the
slope literals are only *formula-consistent*, so the independent slope check is the finite
difference, not the table.

⚠ One bound was **re-pinned from the measurement rather than from the guess**: the
canopy-resistance term's contribution was written as `> 1.5×` and measured at **1.4432×**.
It is now two-sided on the measurement, so a change that *shrinks* the term is caught as
well as one that drops it.

### §5ag review — four corrections, and the one that could have falsified the batch's headline artefact

An audit of the batch after it was committed. As with batch A's, **none of the four could
have been found by either battery**: all 23 mutations reddened a test whose subject was the
mutated mechanism, and the batch read as finished.

`cargo test -p domains --lib` 257 → **259**, `-p station --lib` 60 → **61**, clippy clean, no
frozen value moved.

#### ⚠⚠ 1. The census scanned a HAND LIST OF FILES, which is the same defect one level up

`the_scenario_roster_matches_what_the_source_declares` named two paths outright:
`domains/src/biosphere/system.rs` and `station/src/scenario.rs`. Its two controls prove the
*scanner* works and the *comparison* bites — and neither can see a `SeasonScenario` declared
in a **third** module. So the census replaced a hand list of scenarios with a hand list of
files, while making a claim about the tree. That is precisely the failure the
reflection-based Python original exists to prevent, moved up one level and out of view of
every control the batch shipped.

The file set is now **discovered**: a walk of `rust/crates/*/src/**/*.rs`, with the two
files it currently finds asserted as a recorded measurement (the shape
`the_census_matches_the_directory_on_disk` uses against its directory) rather than as the
input. Measured: declaring a scenario in `station/src/greenhouse.rs` — a module the first
draft could not have looked at — now reddens exactly one test of 60, the census.

*The general form: when you replace a reflective enumeration with a scan, ask what the scan
itself is hand-fed. A control that proves the scanner works says nothing about whether the
scanner was pointed at everything.*

#### ⚠⚠ 2. The batch had no "deliberately NOT ported" list, which is §5ad's whole subject

Batches A and B each enumerate the tests that get no successor and why. Batch C shipped with
"what batch C leaves standing" (forward items) and **no per-test absence list** — so 77
Python test functions became 43 Rust tests with the difference unexplained, and an absence
reads as an oversight rather than as a decision. That is the exact failure mode S5 exists to
prevent, in the batch that exists to prevent it.

The list, with what each absence actually is:

| Python test | disposition |
|---|---|
| `test_water_stress_factor_rejects_non_positive_threshold` | **inherited port decision** (`science.rs`'s module header omits the `ValueError` input guards on hot rate laws) — but its PRECONDITION is now asserted, see below |
| `test_penman_monteith_rejects_non_positive_aerodynamic_resistance` | **successor exists, one layer down**: `transpiration_from` rejects a non-positive pair. The check moved from the function to the loader |
| `test_sealed_water_scoped_compartment_ledger_balances_every_step` | **no successor**: `compartment_boundary_ledger` has no Rust equivalent anywhere in the tree. Recorded as a gap, not as a decision |
| `test_transp_loader_round_trips_a_valid_file`, `test_transpiration_params_file_exists`, `test_loader_reads_committed_rates` | folded into the committed-value pins — `include_str!` makes "the file exists" and "it round-trips" the same assertion |
| `test_the_rate_is_exactly_zero_with_nothing_below` | covered by `a_dry_subsoil_stops_extension` (which already sweeps `<= 0`) plus M16 |
| `test_drought_declares_a_stratified_profile_deliberately`, `test_the_roster_this_covers_is_not_empty_and_includes_the_station` | folded into `a_dry_below_root_store_stops_extension_for_the_whole_season` and the census's own non-vacuity assertions |

Two of these produced work rather than a row:

**`wssg` is a SCENARIO field, and no guard in this port reaches it.** `water_stress_factor`
returns `1.0` whenever `ftsw >= threshold`, so a zero `wssg` reads even a bone-dry root zone
as perfectly unstressed — Python raises, Rust cannot without putting a `Result` on a rate law
called every step. The omission is the inherited Phase-7 decision, but §5ad had already
flagged its soft half: *"they never fire for the frozen scenarios"* is a claim about the
roster **as it stood when it was written**. `no_scenario_declares_an_input_the_omitted_guards
_would_have_rejected` now checks it on every scenario the census finds, at zero runtime cost.
Measured: declaring `wssg = 0` on the frozen scenario reddens it, alone in the station binary.

**The five-cycle ratchet claim had no successor at all.** The batch's re-sow pins were a
single `annual_reset` call and a two-year run asserting `returned > 0`; the Python claim is a
FIXED POINT over five cycles, *"the difference between a cycle and a ratchet, and no single
golden can show it"*. `the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years` ships,
with the transient's existence, size and direction pinned alongside the convergence, and
with the two stores' joint conservation across every cycle boundary — which is what says the
convergence is a redistribution rather than a leak. ⚠ Its honest scope is measured and
stated in the test: it catches a return that **stops**, not one that is the wrong **size** (a
`× 1.000001` drift converges to a *different* fixed point and leaves it green, and is caught
by the exact-value pins instead).

**And `RootZoneCapture` had no balance test**, which is batch A's own review finding one
mechanism over: the biosphere called `assert_flow_balanced_default` nowhere until batch A
added the gas case, and batch C added the water flows' — but not the capture's.
`the_root_zone_capture_is_a_balanced_internal_transfer` ships.

#### ⚠ 3. A guard's justification asserted something about the roster that is false

`water_cycle_from`'s doc comment said the non-negative guard exists because a zero rate is
how *"every OPEN-field scenario in the tree"* declares no condenser. It is not: the ring is
built only inside the `sealed` branch, so an open-field scenario **omits the two flows
entirely** rather than declaring zero rates, and nothing in the tree declares a zero (the
shipped file is 0.5/0.5). The guard's shape is the FILE's own rule — its header says *"A zero
rate is valid ...; negative is rejected"* — and that is what the comment now says, with the
correction recorded in place. Same species as batch A's overclaims: **a reason asserted about
a roster that does not hold**.

#### What this review says about the batch's method, again

Batch A's correction concluded that *a green mutation battery is evidence about the
INSTRUMENT, not about the arithmetic*. Batch C adds a second axis: **a battery is also
evidence about the mechanisms you chose to mutate, and says nothing about the ones you did
not port.** Three of the four items here are about *absence* — a file the scan was never
pointed at, tests with no successor and no recorded reason, a claim with no test. No
mutation of any shipped mechanism could surface any of them, because none of them is a
mechanism that is wrong; they are claims that are missing. *The instrument for absence is a
census, and the census has to be written down.*

### What batch C leaves standing

* **`intercepted_fraction` still unresolved** (S6 item; clause 4 of the exit gate).
* **The by-name claim census (clause 3) still unwritten** — now with batch C's additional
  no-ancestor pins (the anthesis boundary, the per-step ceiling, the PM unreachability
  assertion) to declare alongside batch B's two.
* **`daily_thermal_time`'s 30 °C cap** — batch B's unreachable branch, still owned by
  nothing.
* **The Penman–Monteith negative-energy clamp is unreachable, and the model has no longwave
  term.** New, and it is a science question rather than a testing one: either the clamp is
  dead weight or `net_radiation` is missing a term. Not decided here.
* **`flows.rs`'s test-local `ROOTED_DEPTH`** still shadows `stocks::ROOTED_DEPTH` with a
  different string (batch B's finding). Batch C's own fixtures use the engine's name
  explicitly and say so, but the shadow is untouched.
* **`compartment_boundary_ledger` has no Rust equivalent** anywhere in the tree, so
  `test_sealed_water_scoped_compartment_ledger_balances_every_step` has no successor. A
  gap found by the review's absence census, not a decision.
* **No Python deleted.** All four files stay green and running until S6.

## §5ah Stage 3 — S5 batch D BUILT, COMPLETE 2026-08-26: the spending batch, where conservation is blind by construction

Batch D is the carbon-spending batch: `test_allocation` 43, `test_respiration` 25,
`test_carbon_budget` 22, `test_stem_reserves` 22 = **112 Python tests**, of which **14 are
not batch D's subject at all** (see the roster correction below). It lands on three
surfaces in one crate.

`cargo test -p domains --lib` **259 → 282**; `cargo test --workspace --no-fail-fast`
**891 → 914**; clippy clean at `--all-targets -D warnings`. **No golden byte, band, floor
or manifest moved** — asserted with `git status --porcelain rust/data/` (empty), not
inferred from a green suite. Harness and all logs:
`M:\claud_projects\temp\s5-batch-d`.

The batch is tests plus **one production change**, taken as an application of batches B
and C's already-given answer rather than as a fresh decision (see "the loader split").

### ⚠⚠ What makes this batch different in KIND: it is the redistribution batch

Three of its four subjects are *redistributions*: the partition table splitting one
increment four ways, `fstr` moving part of the stem leg into shielded starch, the
maintenance shortfall burning organs in proportion, the reserve draining into the grain.
**Every one of those keeps each flow's legs summing exactly as they did**, so the
biosphere's strongest machinery — `assert_flow_balanced`, the conservation assertion on
every step, the boundary ledger — is blind to all of them by construction.

Batch C learned this once (its M15: a balanced mutation of `Recycling` was invisible to the
balance machinery). Here it is not an analogy, it is the default case, so the battery was
built around it: **eight of the fifteen mutations below are sum-preserving reshuffles.**

### The before-battery: fifteen mutations, THREE caught by a test about the mechanism

Baseline 259 passed / 0 failed. `bal` marks a mutation that preserves every leg sum.

| # | bal | mechanism broken | red | of which **about the mutated mechanism** |
|---|:-:|---|---:|---:|
| D1 | ● | partition table: swap leaf↔stem at the DVS-0 knot | 7 | **1** — and one more red is a *broken fixture*, see below |
| D2 | ● | `partition_fractions` reverses the interpolation weight | 2 | **0** |
| D3 | ● | `Allocation`'s `fstr` reserve split deposits nothing | 2 | **0** |
| D4 | ● | open-field maintenance burns organs EQUALLY, not in proportion | 1 | **0** |
| D5 | ● | `StemRemobilization` runs BACKWARDS, grain → reserve | **28** | **0** |
| D6 | ● | the same equal-split reshuffle, sealed limb | 4 | **1** |
| D7 | | `maintenance_respiration_flux` drops Q10 entirely | 3 | **0** |
| D8 | | `available_for_growth` drops its non-negative clamp | 21 | **0** |
| D9 | | `Allocation`'s DMI drops the growth efficiency `Yg` | 3 | **0** |
| D10 | | `GrowthRespiration`: `(1 − Yg)` → `Yg` | **0** | — |
| D11 | | maintenance `covered` uncapped by GASS | 7 | **1** |
| D12 | ● | `partition_fractions` extrapolates the TOP end from the FIRST row | 2 | **0** |
| D13 | | `StemRemobilization`'s cessation bound goes non-strict | **0** | — |
| D14 | | `Allocation`'s reserve FILL loses its cessation | **0** | — |
| D15 | ● | `partition` routes the grain share into the ROOT leg | 2 | **0** |

**Twelve of fifteen reddened nothing whose subject was the mutated mechanism**, and three
reddened nothing at all. Of the eight sum-preserving reshuffles, exactly two were caught by
something about them, and **both catches are value or leg-SHAPE pins rather than rate
laws**: D1's is `every_value_matches_the_generated_table` (C8's params census, which pins
the table's rows bit-for-bit) and D6's is batch A's
`the_sealed_burn_is_split_in_proportion_to_organ_carbon`.

⚠ **One of D1's seven reds is not a catch, and reading it as one would have inflated the
column.** `a_partition_row_that_does_not_sum_to_one_is_rejected` builds its broken file by
`replacen("fl: 0.55", …)`; D1 changes that literal, so the test's own
`assert_ne!("the substitution must apply")` fires. It reddened because its FIXTURE broke,
not because it saw the mutation — which is the self-protecting assertion doing exactly its
job, and exactly why the "of which" column has to be read from each test's body.

⚠⚠ **D5 is the sharpest reading in the battery.** Reversing the stem-reserve drain — the
grain feeding the stem instead of the stem feeding the grain, a whole mechanism running
backwards — reddens **28 tests in 259, and not one of them is about stem reserves.**
Twenty-eight reds carrying zero information about what broke is the same reading as a
golden red: *a number moved.* This mechanism has a 1,643-line Python file behind it and
shipped only on the user's explicit call, and in Rust it was guarded by nothing that names
it. D3 is the same finding from the other side: deleting the reserve's FORMATION entirely
reddened two tests, neither about reserves.

### ⚠⚠ Three mechanisms were guarded by the goldens and by NOTHING ELSE

D10, D13 and D14 each reddened **zero** tests of `-p domains --lib`. Applied together they
were re-run under `cargo test --workspace --no-fail-fast`, which came back **888 passed / 3
failed** — and all three failures are committed-byte comparisons.

Applied together is not an attribution, so each was then re-run ALONE against the three
golden binaries. The result is uniform and it is the point:

| mutation | golden reds, applied alone |
|---|---|
| D10 `GrowthRespiration`'s complement | `every_domains_golden_is_still_this_reference_s_output`, both tier-contract bands |
| D13 the drain's cessation | the same three |
| D14 the fill's cessation | the same three |
| *control: clean tree* | 14 passed, 0 failed |

**So three live mechanisms of the frozen biosphere were guarded by committed bytes and by
nothing else** — nothing behavioural, nothing named, nothing that would survive someone
regenerating the goldens. That is batch A's canopy-quadrature finding reproduced on three
more mechanisms, and D13/D14 are the pair whose param file argues about them at length (the
`FINISH DS = 2.` domain-boundary reasoning runs to a full paragraph in
`stem_reserves.yaml`, and nothing in the tree checked that the bound was strict).

⚠ **Why D10 is invisible in particular, read rather than guessed.** `GrowthRespiration`
moves carbon from `co2_atmos` to `co2_resp`. In the frozen open-field wiring both are
BOUNDARY stocks, so no organ, no chamber gas and no conserved quantity moves with it; in
the sealed wiring the two are the same stock and the flow is empty by construction. A
mutation of its coefficient therefore changes exactly one thing: numbers in a file.

### The silent-branch probe: eleven branches, all live, and one reached by a single test

Batch B's second instrument (`panic!` at the top of a branch, count the tests that fire),
run on all eleven branches of batch D's surfaces. **None is unreachable** — unlike batch
C's six — so every one is a *write a test* case rather than a *only a scenario can reach
it* case.

The one worth naming is **P9, `StemRemobilization`'s empty-reserve zero-flux limb, reached
by exactly ONE test in 282** — and that test is this batch's own
`the_remobilization_drains_the_reserve_into_the_grain_and_is_first_order`. Before batch D
it was reached by nothing. The other ten fire in 14–41 tests apiece.

### The roster correction, predicted BEFORE the port rather than found in review

§5ad's roster row says batch D lands on "`science.rs` partition, Q10, growth budget". That
is true of a quarter of it. **This is the fourth consecutive batch to correct that column**,
so this time the correction was written down before any Rust was, and it has two halves.

**Half one — 14 of the 112 tests are batch G's subject, not batch D's.**
`test_allocation.py` is two files in one: 29 tests about partitioning and its loader, and
**14 about senescence** (`senescence_flux`, the `Senescence` flow, mutual shading, and the
whole `senescence.yaml` loader block). Senescence is batch G's mechanism. They are **handed
forward by name** rather than ported here or dropped — batch G's roster row grows from 37
tests to 51.

**Half two — the surfaces.** Batch D's own 98 tests split by SUBJECT:

| surface | tests | subject |
|---|---:|---|
| `domains/src/biosphere/science.rs` | 8 | the equations: Q10, maintenance, the clamped budget, the partition table's interpolation, extrapolation, sum rule and split |
| `domains/src/biosphere/flows.rs` | 10 | the shared budget and its limitation, the three budget-coupled flows' legs, both halves of the stem reserve |
| `domains/src/biosphere/params.rs` | 5 | the three param files' REJECTIONS (the values were already pinned by C8's census) |

### The loader split: one production change, and it is the third instance of the same one

`respiration()` and `stem_reserves()` were the last two guarded loaders in `params.rs`
still hard-wired to their own `include_str!`, so their bound and unit guards were
unreachable from any test. Both are now split into a text-taking core
(`respiration_from`, `stem_reserves_from`) plus a thin wrapper — **the same refactor
batches B and C each made** for phenology, transpiration, root depth and the water cycle,
and it is not the extraction §5ad rules out. That one is about lifting equations out of
`Flow::demand` to make them unit-testable, which changes what the science is made of; this
changes nothing but who may call the reader, and the committed values are asserted
unchanged by the params census.

### ⚠ A guard asymmetry, found by an absence census and MEASURED before it was written up

`allocation.yaml` is the one frozen biosphere param file whose loader enforces **neither**
the provenance rule nor the field-set rule. Its schema is a LIST of rows rather than flat
value/unit/source scalars, so `allocation_from` reads the table through the raw node API
and never meets `guarded_map`, which is where both rules live. Probed before the finding was
written: stripping its `source:` and adding an unknown top-level key were **both accepted**.

It is not unguarded, and that is the half worth recording rather than filing as a gap: the
file's newline-normalized sha-256 is pinned in `docs/biosphere-reference.manifest.json`
under `param_files`, and since C7 the reference WRITES that manifest while
`tests/crossport/test_manifest_writer.py` compares the committed bytes. A provenance-only
edit is therefore caught — **as a stale manifest, not as a load error**. Two different
failures, two different fixes, and only one of them names the file.

`provenance_is_enforced_at_the_loader_for_two_files_and_at_the_manifest_for_the_third`
pins all of it, including the two mutations that LOAD — so if `allocation_from` is ever
routed through `guarded_map`, that assertion says so out loud instead of a guard quietly
appearing. What has no guard either way is a FUTURE list-shaped param file, which would
inherit this loader shape and be required to carry a source by nothing until it reached
the manifest census. **Recorded as an S6 item, not fixed inside a testing batch.**

### ⚠ A bound test written from the wrong end passes on every input except the two that define it

`the_respiration_bounds_are_rejected_each_at_its_own_shape` went RED on its first run, and
the reason is worth a sentence. Its first draft asserted a `[0, 1)` growth efficiency and a
legal zero remobilization rate — **the mirror image of both bounds**. `require_half_open`
is `(0, 1]` and says so in as many words in its own doc comment ("zero is a degenerate
model, one is lossless and legitimate"); the draft was written from the range's NAME, and
the helper was four files away in `config/`. Every input except `0.0` and `1.0` behaves
identically under the two readings, so a bound test written from the wrong end is green on
everything but the two values that are the point of having a bound.

The shipped test now asserts the shapes side by side and states why they differ: the
remobilization RATE is `(0, 1]` (draining the whole standing reserve in a day is
degenerate but legal), while the remobilizable FRACTION is open at BOTH ends (a stem that
diverts all of its growth to starch never builds structure at all).

### ⚠ The Python claim that has no Rust successor: the RK4 half

`test_the_reserve_closes_every_sealed_chamber_on_both_integrators` asserts closure under
**Euler and RK4**. Rust has `Rk4Integrator`, and `crew_run`, `eclss_run`, `power_run` and
`thermal_run` all use it — but **nothing in the Rust tree ever runs the BIOSPHERE under
RK4**. Measured by enumerating every `Rk4Integrator` user in `crates/*/src` and
`crates/*/tests`.

So the Euler half of that claim is covered structurally (the science gates and
`system.rs`'s season runs exercise the reserve-live wiring and assert conservation every
step — which is why D3 and D5 reddened so many of them), and the RK4 half has no successor
at all. It is not fixed here: running the biosphere under RK4 is a capability change with
its own consequences (under RK4+ a needed arbitration scale is a HARD ERROR, not a
backstop), and §5ad forbids slipping that into a testing batch. **Recorded as a successor
item.** It also sits next to a trap this repo has already logged twice — `rationed == 0`
under Euler is blind by construction, and the Euler/RK4 trap runs both ways.

### What batch D deliberately does NOT port, per test

Written as a list rather than as forward items, because batch C's review found that a batch
shipping only forward items lets an absence read as an oversight. 112 Python tests, 23 Rust
tests, and here is every difference.

| Python test(s) | disposition |
|---|---|
| the 14 senescence-subject tests of `test_allocation.py` | **handed to batch G by name** — its mechanism, its roster row grows 37 → 51 |
| `test_maintenance_respiration_maturity_seam_scales_linearly` | **no successor, and correctly so**: `maturity` is hard-coded to 1.0 in this port, and nothing in EITHER tree ever passes anything but the default. The seam was exercised by its own test and by nothing else — a finding about the reference it came from, not a gap in the port |
| `test_*_params_file_exists`, `test_*_loader_round_trips_a_valid_file`, `test_load_*_matches_committed_values` | folded into `include_str!` plus C8's params census — "the file exists", "it round-trips" and "its values are these" are one assertion once the file is compiled in |
| `test_growth_flow_is_carbon_balanced`, `test_maintenance_flow_is_carbon_balanced_both_regimes`, `test_allocation_is_carbon_balanced` | the engine's own machinery: `assert_conserved` runs every step of every run, and batch A recorded the same disposition for the gas flows |
| `test_context_storage_excluded_from_biomass` | **guarded by the TYPE, harder than by a test** — `CarbonContext` has no `storage_c` field, so `leaf_and_biomass` cannot include storage and no mutation short of adding a field can redden a test for it. Same disposition batch A gave `FlowResult::new` rejecting a duplicate leg. ⚠ The batch first shipped this as a ported claim whose docstring said `with_storage` made it falsifiable; measured in review, it did not — see §5ai |
| `test_maintenance_zero_biomass_is_inert` | covered by `maintenance_respiration_is_the_reference_rate_scaled_by_biomass_and_q10`'s exact zeros plus the `biomass > 0.0` branch probe (P10/P11) |
| `test_partition_fractions_empty_table_raises` | **inherited port decision**: `partition_fractions` indexes `table[0]` directly, so an empty table is a panic rather than a raised error. The loader refuses a table with fewer than two rows before the function can ever see one — pinned by `the_partition_tables_structural_rules_are_each_rejected_separately` |
| `test_resp_loader_rejects_a_missing_source` / `_an_unknown_field` for **allocation** | **no loader successor and that is a measured asymmetry**, see the guard-asymmetry section above; the manifest owns it instead |
| `test_the_reserve_closes_every_sealed_chamber_on_both_integrators` | Euler half covered structurally; **RK4 half has no successor**, see above |
| the ten design-record tests of `test_stem_reserves.py` (`test_the_SOURCED_form_*`, `test_OUR_reconstruction_*`, `test_NEITHER_form_*`, `test_the_partition_table_is_what_blocks_it_*`, `test_the_drain_rate_is_BIT_INERT_*`, `test_a_reserve_that_never_drains_moves_NITROGEN_*`, `test_the_trigger_is_OURS_*`, `test_the_fill_fraction_is_the_only_number_*`, `test_the_grain_gain_is_the_TRANSFER_*`, `test_the_frozen_harvest_index_is_LOW_*`) | **no successor, by nature.** These run candidate flow classes (`_GrowthFractionFill`, `_SnapshotFill`, `_ReserveShedding`) that exist only inside the test file and were never built, and they measure sensitivities to argue a decision that has already been taken. They are a DECISION RECORD written as executable tests; their content lives in `stem_reserves.yaml`'s header and `docs/plans/post-roadmap-stem-reserves.md`. ⚠ This is the largest single absence in the batch and it is a judgement, not an oversight: porting them would mean building two unbuilt models in Rust to re-refute them |
| `test_the_reserve_passes_every_manifest_liveness_floor_*`, `test_the_open_season_science_bands_survive_*`, `test_option_Bs_litter_C_to_N_identity_survives_*` | covered structurally by the science gates, which run the reserve-live wiring — measured, not assumed: D3 and D5 reddened those gates. **But structurally is not by name**, which is exactly what the by-name claim census (clause 3) exists to make visible |

### What batch D leaves standing

* **The by-name claim census (clause 3) is now deferred by FOUR consecutive batches**, and
  it is named here rather than deferred silently a fourth time. Its accumulated input:
  batch B's two no-ancestor pins, batch C's three, and batch D's **three new ones** — the
  goldens-only guard on `GrowthRespiration`'s complement and on both cessation bounds; the
  measured loader/manifest guard asymmetry on `allocation.yaml`; and the three structural
  coverings above, which are the census's own subject (a claim covered structurally but not
  by name is precisely a row that would go missing).
* **The biosphere is Euler-only in the Rust tree.** New, and the largest of the leftovers.
* **A future list-shaped param file would have no provenance guard at its loader.** S6.
* **`intercepted_fraction` still unresolved** (S6; clause 4 of the exit gate).
* **`daily_thermal_time`'s 30 °C cap** — batch B's unreachable branch, still owned by
  nothing.
* **The Penman–Monteith negative-energy clamp is unreachable and the model has no longwave
  term** — batch C's science question, untouched.
* **`compartment_boundary_ledger` has no Rust equivalent** — batch C's gap, untouched.
* **`flows.rs`'s test-local `ROOTED_DEPTH`** still shadows `stocks::ROOTED_DEPTH`.
* **No Python deleted.** All four files stay green (112 passed, re-measured) until S6.

### ⚠ One harness defect, found and fixed mid-batch

The battery's revert check hashed the file's **decoded text**, and `Path.read_text` /
`write_text` translate newlines on Windows — so it round-tripped CRLF → LF on two files and
its own sha-256 comparison could not see it. Caught by `git status`, not by the check that
existed to catch it. The harness now reads and writes BYTES, and the digest is over the
bytes on disk. *A byte-exactness check that normalizes before hashing is checking
something else.* Same species as this batch's other instrument findings, one level down: in
the harness rather than in the subject.

## §5ai Stage 3 — S5 batch D review, 2026-08-26: the after-battery asked the wrong question

Four findings, and the first of them is a red CI job that was already pushed. Same shape as
batch C's review section: each is something neither of the batch's two instruments could
have surfaced, because none of them is a mechanism that is wrong.

### ⚠⚠ 1. The Python-side gates were never run, and one of them was RED

Batch D's verification was `cargo test`, `cargo clippy`, the mutation battery, the branch
probes, `tests/test_context_budget.py` and the four batch-D Python files. `CLAUDE.md`'s
command list also carries `uv run ruff check .`, `uv run ruff format .` and
`uv run pyright`, and **none of them ran** — on either commit.

`uv run ruff check .` came back with **8 `E501` errors**, all in the ceiling commit's own
comment block, all already on `main`. That is a red CI Python job pushed twice over.

⚠ The near-miss inside the miss: `ruff format --check` **passes** on the same file. The
formatter does not re-wrap comments, so a comment block over the line limit is formatted
and lint-red at the same time, and checking the formatter would have said "clean". *Two
tools with adjacent names own different halves of the same rule.* This repo already has
`ci-python-job-red-on-linux` recorded — "local green ≠ CI green" — and this is the weaker
version of it: not local-green-CI-red, but **local never asked**.

### ⚠⚠ 2. Three of the batch's 23 new tests were reddened by NOTHING, and the after-battery could not see it

The after-battery asked, for each of fifteen mutations, *was this caught, and by a test
about it?* — and answered yes fifteen times. It never asked the transposed question: **was
each new TEST reached by anything at all?** Cross-referencing the hit lists against the 23
new names answers it in one pass, and three never appear — in the battery **or** in the
eleven branch probes.

That is the C4b finding (a gate can be inert BY CONSTRUCTION) arriving through a hole in
the instrument rather than in the subject. A targeted control battery was run on the three:

| control | red | which of the three fired |
|---|---:|---|
| A1 MRES maintains the LEAF, not the whole biomass | 5 | the budget recomposition |
| A2 canopy assimilation ignores the limitation | 3 | the ratio test |
| A3 `available_for_growth`'s two arguments transposed | 29 | the budget recomposition, the ratio test |
| B1 the nitrogen factor dropped from the limitation product | 1 | **NONE** |
| B2 the nitrogen denominator is the LEAF, not leaf+stem+root | 1 | **NONE** |
| C1 `Allocation`'s DMI scaled by a constant the growth flow does not share | 5 | **NONE** |
| C2 *(no-op control: an attribute only)* | 0 | none, correctly |

**Two of the three are real but narrower than their docstrings claimed**, and both now say
so. The budget recomposition is a **composition** check: it calls the same `science.rs`
entry points the flow does, so a wrong rate law moves both sides identically and it stays
green — which is exactly why the two mutations that broke `maintenance_respiration_flux`
and `available_for_growth` outright never touched it. What it owns is everything *between*
the functions (A1, A3). The ratio test is the same species one level out, with its own
measured limit: it sees a limitation that reaches only ONE flow (A2) and is blind to a
CONSTANT that reaches only one (C1), because a constant cancels in a ratio.

**The third was genuinely INERT on its central claim, and it is fixed rather than
re-described.** `the_limitation_is_water_times_nitrogen_and_excludes_the_storage_organ`
asserted `lim == f_water · f_n` on a state whose `plant_n` was 1.0 against 5 mol C — a
concentration a hundred times critical — so `f_N` saturated at exactly 1.0 and the claim
degenerated to `lim == f_water`. **Deleting the nitrogen factor from the product outright
reddened one test in 282, and it was not the test named for the product.**

Rewritten as `the_limitation_is_the_product_and_both_factors_actually_bite`, with the
stressed state DERIVED from the loaded thresholds rather than written as a literal, and
four operating points: neither factor biting, nitrogen alone, water alone, and both — the
last one being the only one that distinguishes a product from a `min` or a mean, since all
three agree at 0.5. Measured after: it now reddens on B1 **and** B2.

⚠ **The derivation had to be derived for a reason.** The first rewrite hardcoded
`n_residual = 0.001`, `n_critical = 0.002` — the **Python test fixture's** thresholds, not
the committed file's (0.005 / 0.015, folded) — and read `f_N = 1.0` for a state it had just
declared stressed. *A test that constructs a stressed state must construct it from the
numbers the code will actually use, not from the numbers the test it was ported from used.*

### ⚠ 3. A claim guarded by the TYPE, described as though guarded by the test

The storage half of that test asserted that adding grain does not change the maintained
biomass. `CarbonContext` has no `storage_c` field at all, so `leaf_and_biomass` **cannot**
include storage — the exclusion is enforced by the struct, not by a line that could be
wrong, and no mutation short of adding a field can redden it. The docstring said
`with_storage` was what made the claim falsifiable; measured, it was not: both `lim == 1.0`
assertions passed under either reading, because the nitrogen factor was saturated either
way.

Python's `test_context_storage_excluded_from_biomass` therefore moves to the disposition
table under batch A's existing heading — *guarded by the constructor, harder than by a
test* — which is the same disposition batch A gave `FlowResult::new` rejecting a duplicate
leg. Not a gap; a claim that a type already makes unfalsifiable.

### ⚠ 4. A long verification run overlapping a mutation battery reports the battery's tree

The first `tests/crossport` run came back **3 failed / 191 passed**, all three
`test_rust_reproduces_the_committed_golden_bytes`. It was started in the background and the
inertness controls were run while it was in flight, so the Rust tree it built from was
**mutated** for part of the run. Re-run alone: clean.

Cheap to state, easy to misread as a real golden regression, and it is a harness rule
rather than a finding: *a verification that builds the tree cannot share the tree with
anything that edits it.* Batch D's other harness defect (the byte-exactness check that
hashed decoded text) is the same species — the instrument, not the subject — and this is
the third in one batch.

### ⚠⚠ 5. The memory ceiling has TWO copies, and the raise edited one

The raise that made room for this batch's memory line went into
`tests/test_context_budget.py`. The gate also has a Rust mirror,
`rust/crates/repo_gates/tests/context_budget.rs`, built during this very slice — and it kept
the old `16_000`. It went red on the final full workspace run, on the **first memory line
the raise existed to make room for**, one commit after the raise.

Neither gate was wrong. There are two copies of one rule, and *a rule with two copies has
one that is stale* — a lesson already on the record here from a different subject, and the
ceiling ceremony has now become an instance of it. Both copies carry the same three bounds
as of this commit, and were controlled **together**: a 241 B hook, a 239 B hook and 40
padding rows produce the identical verdict and fire the identical bound on both sides; a
byte-exact revert returns both to green.

⚠ It is worth noting *why* the batch's own gates did not catch it earlier. The batch-D
workspace run (914 / 0) finished **before** the memory index line was written, and the
memory line was written after — so the sequence "run the gate, then edit the thing the gate
guards" hid it for exactly one commit. The general form is the same one this review keeps
finding: *a gate run before the change is not a gate run on the change.*

### What the review says about the method

Batch A: a green battery is evidence about the **instrument**, not the arithmetic.
Batch C: a battery is evidence only about the mechanisms you **chose to mutate**.
Batch D adds the transpose: **a battery is evidence about the mutations, and says nothing
about whether your new TESTS are reachable.** Both questions are answered from the same
run — the hit lists were already on disk — and only one of them was asked. The check is one
pass over the report: *every new test name must appear in at least one hit list, or it owes
a control that shows what it does catch.*

## §5aj Stage 3 — S5 batch E BUILT, COMPLETE 2026-08-26: the nitrogen batch, and seven mechanisms the goldens were guarding alone

Batch E is the nitrogen batch: `test_nitrogen` 37, `test_nitrogen_form` 15,
`test_nitrogen_throttle` 7 = **59 Python tests**. It lands on **four** surfaces in one
crate, and it is the first batch whose subject already carried direct Rust tests — so its
first job was a **subtraction**, not an addition.

`cargo test -p domains --lib` **282 → 298**; `cargo test --workspace --no-fail-fast`
**914 → 930**; clippy clean at `--all-targets -D warnings`. **No golden byte, band, floor
or manifest moved** — asserted with `git status --porcelain rust/data/` (empty). The
Python gates `uv run ruff check .`, `ruff format --check .` and `uv run pyright` all ran
and are clean (batch D's review finding 1, which this batch inherits as a checklist item
rather than as a lesson to relearn). Harness and all logs:
`M:\claud_projects\temp\s5-batch-e`.

The batch is tests plus **one production change**: the `nitrogen_from(text, name)` loader
split, the FOURTH instance of the one batches B, C and D each made, taken as an
application of the given answer rather than as a fresh decision.

### The subtraction that had to come first

`science.rs` already carried three Phase-7 tests over two of batch E's four functions —
`the_nitrogen_stress_ramp_is_linear_between_its_two_knots` (both knots, the interior
monotonicity, the linearity on a clean band, the zero-biomass guard) and
`soil_n_below_the_residual_shuts_uptake_off_entirely`. Batches A–D all started from
surfaces with no direct coverage; here a straight port would have written a second copy of
the `f_N` ramp and called it growth. **So `f_N`'s ramp gets no successor**, and the reason
is written into the block header rather than left as a gap in the count.

⚠ The subtraction is the right first move and it was still half wrong — measured, and
corrected below: the existing pin covers the ramp and is blind to the DENOMINATOR.

What the subtraction left standing is the batch's real subject: **`target_n_concentration`
— Greenwood's curve, the one function of the four with no direct test in either surface.**

### The before-battery: sixteen mutations, ELEVEN of which reddened nothing

Baseline 282 passed / 0 failed, `cargo test -p domains --lib`.

| # | mechanism broken | red | of which **about the mutated mechanism** |
|---|---|---:|---:|
| E1 | `target_n_concentration`: the PLATEAU removed, the curve extrapolated below 1 t/ha | **0** | — |
| E2 | `target_n_concentration`: the exponent's sign flipped | **0** | — |
| E3 | `soil_n_availability`: the hard-off boundary `<=` → `<` | **0** | — *equivalent mutant, see below* |
| E3b | `soil_n_availability`: the interior ramp INVERTED | **0** | — |
| E4 | `nitrogen_stress_factor`: `f_N` reads absolute N, not a concentration | 10 | **1** — and *not* the unit test, see below |
| E5 | `NitrogenUptake`: Greenwood's `W` collapses onto `f_N`'s denominator | **0** | — |
| E6 | `NitrogenUptake`: demand-limiting removed (the retired fixed-flux law) | 1 | **0** |
| E7 | `NitrogenUptake`: the deficit's non-negative clamp dropped | 1 | **0** |
| E8 | `NitrogenUptake`: the availability gate dropped from the capacity | 1 | **0** |
| E9 | `NitrogenUptake`: capacity stops scaling with ground area | **0** | — |
| E10 | `Fertilization`: the rate stops scaling with ground area | **0** | — |
| E11 | `NitrogenSenescence`: the remobilization `min` dropped | **0** | — |
| E12 | `NitrogenSenescence`: the shed concentration decoupled from the plant | **0** | — |
| E13 | `carried_nitrogen`: the donor pool's N:C normalization dropped | **0** | — |
| E14 | `LitterNitrogenTransfer`: the mineralized/stabilised N shares SWAPPED | **0** | — |
| E15 | `annual_reset`: the seedling N WINDFALL returns (total N still conserved) | **0** | — |

**Fifteen of sixteen reddened nothing whose subject was the mutated mechanism**, and
eleven reddened nothing at all.

⚠⚠ **The one direct catch is not the test the subtraction had just credited, and that is
the batch's second-sharpest reading.** E4 replaces `f_N`'s concentration
`plant_n / biomass_c` with the bare amount, and
`the_nitrogen_stress_ramp_is_linear_between_its_two_knots` — the Phase-7 pin that made
`f_N` "already covered" — **stayed green**, because every call it makes passes
`biomass_c = 1.0`. A denominator of one is the arithmetic identity of having no
denominator at all. The single catch was `flows::tests::the_limitation_is_the_product_and_
both_factors_actually_bite`, a flow-level pin one layer out, from batch D.

So the subtraction was right that `f_N`'s RAMP is covered and wrong that `f_N` is, and the
correction is a sixteenth test rather than a note: `the_stress_factor_reads_a_concentration_
and_not_an_amount` puts one amount of nitrogen against two biomasses. **This is the same
species of defect as the availability midpoint below — an existing pin evaluated at the one
point where the thing it is named for cannot be seen — and two independent instances in one
batch is a pattern, not a coincidence.** Both were found by mutation and neither by
reading.

### ⚠⚠ Seven mechanisms were guarded by committed bytes and by nothing else — and four by NOTHING

Each of the eleven zero-red mutations was then re-run ALONE against the golden and
tier-contract binaries (`--test golden_regression --test tier_contract`, whose clean-tree
control is 22 passed / 2 ignored). The split is the finding:

| guarded by the goldens ALONE | guarded by **nothing at all** |
|---|---|
| E1 Greenwood's domain bound | E3b the availability ramp's shape |
| E2 the exponent's sign | E9 the uptake's plot scaling |
| E5 the two denominators | E10 the fertilization's plot scaling |
| E11 the remobilization `min` | E12 the shed concentration's coupling |
| E13 the carried-N kernel | |
| E14 the N respired/stabilised split | |
| E15 the re-sow's nitrogen split | |

That is batch D's three-mechanism finding at more than twice the size, and the second
column is worse than the first: four live mechanisms whose mutation moves **no golden
byte**, so regenerating the goldens would not even record that they changed.

### The branch probe explains three of the four zeros, and only three

Batch B's instrument (a `panic!` at the top of a branch, count the tests that fire), run
on fifteen branches of batch E's surfaces:

* **E3 is an EQUIVALENT MUTANT, not a coverage hole.** At `soil_n == sn_residual` the `<=`
  returns 0 and the `<` falls through to `(0)/(crit−res) = 0`. Same number, both readings.
  Recorded rather than counted, and it is why E3b was written: a battery that scores an
  equivalent mutant as an uncaught one inflates its own column.
* **E12 is an equivalent mutant ON THE FROZEN ROSTER.** Probe P14 — the lean arm of
  `min(plant_n/biomass_c, n_residual)` — fires in **zero** tests: no scenario in the tree
  ever runs a plant below the residual concentration, so the `min` always selects the
  residual and replacing it with that constant changes nothing. E11 (dropping the `min`
  outright) does move, which is the consistent reading.
* **E9/E10 are batch C's ground-area finding on two more call sites.** Every frozen
  scenario is 1 m², so an area factor is invisible to the goldens, the tier bands and the
  cross-port comparison alike. `system.rs::capture_scales_with_ground_area_at_its_call_sites`
  already says this for water; nothing said it for nitrogen.
* **E3b is a coverage hole, and a specific one.** Probe P5 says the availability ramp's
  interior is reached by **one test in the whole binary** — the Phase-7 pin — and that
  test's only interior assertion is at the ramp's MIDPOINT. The midpoint is a fixed point
  of `x ↦ 1 − x`, so **a full inversion of the ramp is invisible to the only test that
  reaches it.** The Python side parametrizes six points including the quarter band; the
  port carried the midpoint over and lost the discriminating ones.

Two more probe readings worth keeping: `nitrogen_stress_factor`'s fully-stressed limb and
zero-biomass guard each fire in exactly ONE test (the Phase-7 pin), and
`NitrogenSenescence`'s zero-shed guard fires in none.

### ⚠ The Python test that is inert on its own subject, measured rather than inferred

`test_nitrogen_form.py::test_nitrogen_is_conserved_across_the_annual_reset` builds
`PERENNIAL_CHAMBER_SCENARIO` and drives it through **`run_season`** — the driver with no
reset hook — so it never crosses a reset at all. Measured: with the reset's litter leg
deleted outright, so that nitrogen is DESTROYED at every year boundary, **that test still
passes**. The mutation *is* caught — by `test_litter_pool_cn_is_TWO_regimes_...`, which
drives the same scenario through `run_perennial` and trips the engine's own conservation
gate. So the claim was held structurally, inside a test named for something else.

The sharp part is that the file already knew: `_litter_rows`' docstring says in as many
words that *"`resets` is not a knob — it is a property of the scenario, and getting it
wrong is what this module's correction was about."* **The correction was applied to the
helper and not to the test one function below it.** A correction is a claim about a file,
and it stops at the call sites someone remembered to visit.

The Rust successor asserts the **split** — the seedling inherits the parent's tissue
concentration, the remainder is the balancing residual into litter — and only then the
total, because the total is exactly what the windfall reading also satisfies.

### The roster correction — the FIFTH consecutive one, and the first to add a fourth surface

§5ad's row says batch E lands on "`science.rs` target N, uptake, stress". True of a third
of it. Batch E's 59 tests split by SUBJECT:

| surface | Rust tests | subject |
|---|---:|---|
| `domains/src/biosphere/science.rs` | 5 | Greenwood's curve (both branches, the crossing, the degenerate bound), the availability ramp off its symmetry point, and `f_N`'s denominator |
| `domains/src/biosphere/flows.rs` | 7 | `NitrogenUptake`'s two arms, its two denominators, the plot scaling, `Fertilization`, and both arms of the coupled shed |
| `domains/src/biosphere/params.rs` | 2 | every guard on `nitrogen.yaml`, and the kg N/kg DM → kg N/mol C fold |
| `domains/src/biosphere/system.rs` | 2 | the re-sow's nitrogen split; the roster fact that only the open field leaves the plateau |

### ⚠ Two Rust-side ADDITIONS, named as such

`nitrogen_from` enforces two rules with no Python counterpart, and both are now pinned:
`n_target_coefficient > n_critical` (without it Greenwood's plateau — the curve's maximum
— sits below the stress threshold, so `f_N < 1` from the first step at every crop mass),
and the EQUAL case of the concentration band, which is what separates the `<` the loader
writes from the `<=` it could have been written with. They belong in the by-name claim
census as additions rather than as ports.

### The transposed question, asked INSIDE the batch this time

Batch D's review found that a battery answers "was this mutation caught?" and never "was
each new TEST reached?". Cross-referencing batch E's 16 new names against the after-battery
hit lists: **twelve appear** — eleven under a mutation of their own mechanism, and one under a mutation of something else entirely (see the correction below). Four appear
nowhere, and all four are about things a mechanism mutation cannot touch — guards, a fold,
a derived constant, a degeneracy. Each was given a targeted control:

| control | red | which unreached test fired |
|---|---:|---|
| C1 the loader's ordered-band guard disabled | 1 | the guard census — **and only it** |
| C2 the kg DM per mol C fold INVERTED | 17 | the fold pin, the guard census, the straw C:N |
| C3 the non-positive domain bound PANICS instead of degenerating | 1 | the degeneracy pin — **and only it** |
| C4 the cited straw residual N 0.5 % → 0.6 % | 3 | the straw C:N, the fold pin |
| C5 *(no-op control: a comment only)* | **0** | none, correctly |

All four are live. C1 and C3 reddening exactly one test each is the strongest reading in
the table: those two tests are the *only* thing in the tree that sees those two rules.

### One candidate pin measured INERT before it was written

An assertion that the loader divides before it multiplies (`M_C / cf` then `× value`,
rather than `value × M_C / cf`) was drafted and then measured: at the committed values the
two orders are **bit-identical**, so the pin would have been green under both. It is not
shipped; the order stays a comment, and the measurement is recorded in the fold test's
docstring so nobody writes it again.

### What batch E deliberately does NOT port, per test

59 Python tests, 16 Rust tests. Written as a list, per batch C's review finding that a
batch shipping only forward items lets an absence read as an oversight.

| Python test(s) | disposition |
|---|---|
| `test_nitrogen_stress_factor_cardinal_values`, `_zero_or_negative_biomass_is_neutral` | **already pinned** by Phase 7's `the_nitrogen_stress_ramp_is_linear_between_its_two_knots` — both knots, the interior, the zero-biomass guard. ⚠ With ONE measured exception: that test evaluates at `biomass_c = 1.0` throughout and is therefore blind to the DENOMINATOR, which is exactly what makes `f_N` a concentration. `the_stress_factor_reads_a_concentration_and_not_an_amount` is that half, and only that half |
| `test_soil_n_availability_cardinal_values` | **partly** already pinned (`soil_n_below_the_residual_shuts_uptake_off_entirely`); the discriminating quarter points are the one thing that port lost, and they are what `the_availability_ramp_is_pinned_off_its_own_symmetry_point` restores |
| `test_soil_n_availability_rejects_inverted_band`, `test_nitrogen_stress_factor_rejects_inverted_band` | **no successor, by an inherited decision.** Neither Rust function raises — `science.rs`'s module header states the rule for the whole module (the `ValueError`-raising input guards are omitted; the behavioural clamps are kept exactly). ⚠ For `nitrogen_stress_factor` the loader owns the rejection and it is pinned. For `soil_n_availability` **nothing does**: `sn_residual`/`sn_critical` are SCENARIO fields, not param-file entries, and no scenario validator checks their order. Recorded as an S6 item, not fixed inside a testing batch |
| `test_target_rejects_a_non_positive_plateau_bound` | **split in two**: the loader's rejection is pinned in `params.rs`, and the function's DEGENERACY is pinned in `science.rs`, so the pair is visible from both ends |
| `test_uptake_is_nitrogen_balanced`, `test_fertilization_is_nitrogen_balanced` | the engine's own machinery — `assert_conserved` runs every step of every run. Same disposition batches A and D gave the gas and budget flows |
| `test_nitrogen_params_file_exists`, `test_nitrogen_loader_round_trips_a_valid_file` | folded into `include_str!` plus C8's params census: "the file exists" and "it round-trips" are one assertion once the file is compiled in |
| `test_committed_nitrogen_carbon_fraction_matches_canopy` | **already pinned**, and harder: `params.rs::the_two_carbon_fractions_agree` compares the two FOLDED constants rather than two YAML literals, so it also catches a fold that diverges without the file doing so |
| `test_uptake_scales_linearly_with_dt` | folded into the two-arm test's dt half and the fertilization dt pin; the capacity branch alone (which is all the Python test reaches) is the weaker of the two |
| `test_open_season_peaks_below_the_crossing_with_the_margin_pinned` | **already the reference's own gate** — `science_gates::open_season_peaks_below_the_greenwood_crossing`, since slice C4. The Python function is the checker's copy. What the gate does NOT assert is that the crossing is where the curve actually crosses, and `the_greenwood_target_meets_the_stress_threshold_at_the_crossing` is that other half |
| `test_open_season_peak_w_margin_to_the_crossing` | **no successor, and it is a judgement.** A two-sided characterization band on our own peak (`13.2 < peak_w < 13.6`, ratio `0.920–0.939`), re-measured FOUR times, whose own docstring argues it is deliberately not a gate. It has caught two real regressions, so this absence is the largest of the batch — recorded as an open item below rather than filed as covered |
| `test_shed_nitrogen_uses_the_same_carbon_flux_as_the_senescence_flow` | ported as `the_shed_nitrogen_is_the_senescing_carbon_at_the_remobilized_concentration`, which compares the N leg against `Senescence`'s own litter-carbon leg — plus the LEAN arm, which nothing in either tree reaches |
| `test_litter_pool_cn_is_TWO_regimes_set_by_which_event_fills_the_pool`, `test_the_pool_cn_IS_the_shed_ratio_and_the_deviation_is_the_N_FREE_SEED`, `test_the_free_mineralization_rate_no_longer_EXISTS_to_be_calibrated` | **handed to batch F by name.** Their subject is the decomposer chain's carried-N family (`carried_nitrogen`, `LitterNitrogenTransfer`, `MicrobialNitrogenRelease`, `HumusNitrogenRelease`) — `test_mineralization.py`'s surface, which is batch F's. E13 and E14 measure exactly how unguarded that family is today (goldens only), and that measurement is batch F's input, not batch E's to spend. Batch F's roster row grows from 97 tests to 100 |
| `test_shed_material_has_a_straw_like_carbon_to_nitrogen_ratio` | ported — it is the shed COMPOSITION, which is batch E's subject, unlike the pool ratios above |
| the seven tests of `test_nitrogen_throttle.py` | **no successor, by nature.** Option (D) was priced and NOT BUILT; the file is a decision record written as executable tests — PDF phrase extraction from a gitignored `sources/` (so every one of them `skip`s on CI already: `memory/pdf-pins-green-by-skip-on-ci.md`), absence assertions about a module that no longer exists, and a measurement of a pool ratio whose home is batch F. Its content lives in `docs/plans/post-roadmap-nitrogen-cycle-form.md`. Same disposition batch D gave `test_stem_reserves.py`'s ten design-record tests, and for the same reason: porting them would mean rebuilding a refused model in Rust to re-refute it |

### What batch E leaves standing

* **`soil_n_availability`'s band is ordered by nothing.** `sn_residual`/`sn_critical` are
  scenario fields; Python's function raised on an inverted band and Rust's returns a step
  function instead. **S6** — a scenario validator is a production change. ⚠ The BEHAVIOUR is
  pinned as of this batch's review; only the guard is deferred.
* **The Greenwood margin pin has no Rust successor.** The load-bearing half (the 14.4248
  crossing, and that the frozen crop stays under it) is the reference's own gate; the
  narrative band that has caught two regressions is not ported. Open.
* **The by-name claim census (clause 3) is now deferred by FIVE consecutive batches.** Its
  accumulated input grows by batch E's own: the seven goldens-only guards and the four
  no-guard-at-all mechanisms above, the two Rust-side loader additions, and one *claim held
  structurally inside a test named for something else* (the reset's nitrogen), which is the
  census's own subject stated as plainly as it will ever be stated.
* **`carried_nitrogen` and the three N legs of the decomposer chain are guarded by the
  goldens alone** — measured here, handed to batch F.
* **The biosphere is still Euler-only in the Rust tree** (batch D).
* **`intercepted_fraction` still unresolved** (S6; clause 4 of the exit gate).
* **`daily_thermal_time`'s 30 °C cap** — batch B's unreachable branch, still owned by nothing.
* **No Python deleted.** All three files stay green (59 passed, re-measured) until S6.

### ⚠ Batch D's harness defect, reproduced one layer over

Batch D found its own byte-exactness check normalizing newlines before hashing, and fixed it
by reading and writing BYTES. This batch then edited three docs with `Path.write_text`, which
on Windows translates every LF into a CRLF on the way out — so a two-line edit rewrote 214 line
endings in `docs/post-roadmap-log.md` and 118 in the memory index. Caught by `git`'s own
CRLF warning, not by anything of ours. *The rule batch D wrote for its digest is a rule about
every write, not about digests.*

### ⚠ The transposed question's own blind spot, found by the same review

"Does each new test appear in a hit list?" answers REACHABILITY, not whether the test
reddens for the reason its name gives. One of the twelve does not:
`only_the_open_field_crop_leaves_greenwoods_plateau` appears **only under E4** — `f_N`
reading an absolute amount — which has nothing to do with the plateau and reddens it purely
by moving a trajectory until a chamber's peak crosses the margin. **E1, removing the plateau
outright, left it green**, and that is not a defect: the test's load-bearing half is
`w < bound`, a claim about which SCENARIOS exist, which no mutation of a rate law can move.
So the honest count is **eleven reached by a mutation of their own mechanism, plus one whose
only reader is a mutation of something else** — recorded in the test's own docstring, because
this is exactly the "a number moved, wearing a reassuring name" reading §5ad's battery was
built to expose, and it does not stop being that when it appears in our column instead of
theirs.

### The inverted soil-N band: the guard is S6, but the BEHAVIOUR is pinned now

`soil_n_availability`'s band is ordered by nothing (below), and the S6 note said "unguarded"
without saying what the unguarded case does. It does something specific: the function
degenerates to a **step at `sn_residual`**, with the interior unreachable because the two
conditions overlap. That is now asserted, alongside the ramp — the same shape batch D used
for `allocation.yaml`'s two mutations that LOAD, so a validator appearing later says so out
loud instead of a guard quietly materializing. Measured independent of the ramp inversion by
construction, and confirmed against E3b.
