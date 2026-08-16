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
| 7 | **Re-anchor the station manifest** | 6 | freeze contract 2 | ceremony |
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
| `forcing.weather_fixture` / `weather_sha256` | **no** | a hash of the Python-side oracle fixture; Rust reads `weather_facts.txt`, generated from it — **identical shape to `param_files`** |
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

**⚠ `forcing.light_path` was the key most likely to be wrong, and it was MEASURED, not
predicted (advisor).** Unlike every hash beside it this one is **gated exactly** — CI
recomputes the fingerprint in glibc-CPython and compares strings — so re-anchoring it creates
a UCRT-Rust-vs-glibc-CPython pair that had never been measured, where one ULP in `cos` or one
character of difference between the two hex-float writers turns CI red for a non-science
reason (the failure mode slice 2 rejected `std::any::type_name` over). Measured first: Rust
reproduces **all twelve samples byte for byte**, hash unchanged. Had it not, the key would
have been declared Python-retained; inventing a normalization is the move this contract
refuses.

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

**Eight negative controls, each turning exactly one gate red on the intended assertion**, green
again after every revert: the two-direction rename pair above; a golden's bytes tampered
(hash gate); an unclassified field added to the manifest; an `_authority` pattern matching no
field; `drift_summary` reclassified as Rust-authored (caught against the roster); the reference
tree's `BIO_DT` moved to 0.5; its light-path peak factor moved to π/2.1; its
`LONG_HORIZON_YEARS` moved to 16 (which reached both the gate and three regenerated leaves);
and a frozen chamber horizon tampered, which only the new Python horizon-conformance gate sees.
⚠ One control run was **invalid and was re-run**: a stray command had corrupted the manifest's
JSON, so sixteen tests failed on a parse error rather than on the assertion under test —
*a control that turns the whole file red has measured nothing* (slice 5's "check the control's
own exit code", one layer up).

**Not done, and named rather than left implicit:** `param_files` and the weather fixture wait
for slice 9; the science-gate census has no route to the reference at all and is not a slice-9
question; and slices 7 and 8 inherit this same per-key exercise for their own manifests —
including the reading that the station case in `test_inventory_parity.py` is still a parity
check *because* slice 7 has not been taken.

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
