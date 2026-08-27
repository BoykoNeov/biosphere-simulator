# A value-switch harness — "source A vs source B, what changes?" as a command

> ## ⚠⚠ STATUS 2026-08-27 — THE SEAM IS GONE AGAIN. Read this before the 2026-08-15 block below.
>
> **The reference flip finished today (S6, `9b87d09`), which is the event this plan was
> sequenced behind** — the user, 2026-08-16: *"after the switch to Rust, work will continue
> on the universal harness, that permits easy toggle of parameters and science."* That
> instruction widens the subject from values to **science**, and it is now unblocked.
>
> ⚠⚠ **And the 2026-08-15 block below is FALSE as of `4f7168e`.** `src/config/overrides.py`
> and its 19 tests were deleted with the other 271 Python files; `grep overrides
> rust/crates/config/src/` finds nothing. **The seam is NOT built.** The remaining scope is
> therefore *both* halves in Rust — the injection seam **and** the reporting layer — not
> "just the reporting layer" as that block says. Nothing of the Python build survives except
> its findings.
>
> **What DOES survive the language change, unchanged:** §2 (register vs switch), §3 (the
> harness writes nothing), §6 (the five reporting requirements — none of them was about
> Python), §7 (the shim-a-dead-path failure mode, which Rust does not fix), §9 (the first
> target). **What is superseded:** §4's placement, §5's whole option set, §10's exit criteria.
> Each is marked in place below.
>
> **Still out of scope, unchanged:** the `extinction_coef` decision (still the user's; the
> analysis still favours 0.65), its ceremony, and any touch of the perennial liveness floor.
> The harness regenerates the evidence; it does not take the decision.

> ## ⚠ STATUS 2026-08-15 — SUPERSEDED BY THE BLOCK ABOVE; kept because its findings survive its tree. (As written: "THE SEAM IS BUILT; THE HARNESS IS NOT. Read this before §5.")
>
> The user chose **Option C (the clean seam)** over Option A. It shipped in `666670a` as
> `src/config/overrides.py` + the one-line hook in `config.loader.load_yaml`, with 19
> tests (`tests/test_param_overrides.py`). Full suite **2425 passed / 5 skipped**, up
> exactly the 19 new tests; every golden, both manifests and every science gate green.
>
> ⚠ **§5's price for C was wrong and is now measured.** It claimed an unfreeze question
> via the manifests' completeness half; that half is defined over the param-file set, the
> flow-class set, the horizon constant and file existence, and a boundary hook is in none
> of them. **Nothing in either manifest moved.** The paragraph is kept below as written
> because the correction is the point — see also `memory/asserted-attributions-rot`.
>
> ⚠ **§5's recommendation ("A first, C only if A proves fragile") is SUPERSEDED.** Do not
> follow it. C is the shipped route.
>
> **What is still to build (the actual remaining scope):** §8's runner and comparison
> mode — take a set of substitutions and a scenario, run, and report the readouts with
> §6's five requirements honoured. The seam is the foundation; the reporting layer is the
> deliverable. §9's first target is unchanged.
>
> **⚠ The `extinction_coef` decision is STILL OPEN AND UNTAKEN.** The user delegated the
> choice ("go with the extinction coefficient of your choosing") and the analysis favours
> **0.65** — crop-specific, our exact equation, canopy peak toward mid-band, LAI peak
> moving before anthesis, and the liveness-floor objection dissolved on examination (that
> floor is a self-calibrated continuity tripwire whose own 50-yr subject already sits
> below it at 0.60). **Nothing was changed**: the ceremony did not start, `canopy.yaml`
> still reads 0.6 with its `TODO(cite)`. ⚠ Before shipping 0.65, settle the one live
> operational risk — the perennial liveness floor clears by **0.40 %** and that was
> measured on Windows while CI is Linux, inside the band where libm ULP differences have
> taken locally-green results red. Name that exposure in the commit *before* pushing, and
> do **not** re-anchor the floor in the same batch (that would be indistinguishable from
> the co-adaptation this tree has refused four times).

**PLANNED 2026-08-15, the seam BUILT (see status above), the harness not.** Forward-looking; exempt from
the log index per the paragraph in `docs/post-roadmap-log.md` (call it *"the value-switch
plan"* in any record file — naming it by filename there turns the parity check red).

Successor to the canopy-provenance work (`post-roadmap-canopy-provenance.md`), which
left the `extinction_coef` decision **open and untaken**. This plan does not take it.

## 1. Why — measured, not theoretical

Answering one parameter question in one session cost **three hand-written probes**, all
thrown away:

| probe | what it did |
|---|---|
| `probe1_k_sweep.py` | patched `plants.load_canopy_params`, `dataclasses.replace(..., extinction_coef=k)`, re-ran the season, read peak LAI / harvest / DVS-at-peak |
| `probe2_chamber_bands.py` | drove the CO₂ band's own `_season_low_ppm` helper at each `k`, with cache clearing |
| `probe3_floor_margins.py` | read the liveness floors and their clearances at each `k` |

Three instances of one tool in one sitting — and that is the small number. ⚠ **Counted
across the tree rather than asserted: 42 distinct probe scripts are named in 16 plan
docs**, from `phase-2-closed-chamber.md`'s `bench/probe_multiyear.py` through
`probe1_sizing.py … probe10_which_stock.py` (crew-coupled loop), `probe.py … probe7.py`
(allocation headroom), and the three above. Nearly all live under
`M:/claud_projects/temp/`, outside the repo, and are gone.

**The tool is rebuilt every time a question is asked and deleted every time it is
answered** — 42 times. This plan was first drafted claiming "at least three more times";
measuring it moved the number by an order of magnitude, which is itself the argument.

**The gap this fills is specific.** The tree already has:

* a **documentation** answer to "which numbers are soft" — the three provenance classes
  (CITED / DESIGN / TODO(cite)) in `docs/param-file-conventions.md`, with `source:`
  presence loader-enforced per param file;
* a **finding** answer — the habit of pinning a disagreement as a test
  (`test_gap_2_the_headline_the_two_sources_disagree_on_tuber_onset`,
  `test_the_two_source_tables_disagree_by_an_order_but_agree_on_the_form`,
  `test_cited_and_fitted_partitions_agree_BY_CONSTRUCTION_so_it_is_no_evidence`).

It has **no experiment answer**. Asking "what would source B's value do?" costs a
hand-written patch script — and a hand-written patch script is one careless edit away from
being a real change to the tree.

## 2. The distinction this is built on (the user's, 2026-08-15)

> *"a central place to switch between critical model values — this is a great basis for
> experimentation. Switching this for that results in such and such consequences."*

A **register** is a list you read; it decays, because nothing forces it to stay true. A
**switch** is a thing you run; it cannot decay, because you re-run it. A previous session
argued against the first and, by conflating them, accidentally argued against the second.

⚠ **A central register remains out of scope and is still the wrong build** — it would
relocate information that already lives in enforced places, and
`memory/context-budget-relocation-is-not-a-discipline` is exactly that lesson. Prose and
references stay where they are (param `source:` tags, plan docs, pinned tests); what this
adds is the *reproducible command*, so a future A-vs-B table is regenerated rather than
quoted.

## 3. The safety property that is the whole point

**The harness overrides in memory and writes nothing.** It reads the frozen YAML,
substitutes values for the duration of one run, and touches no file. Therefore:

* no param YAML is edited → no per-file sha-256 moves in either manifest;
* no golden is regenerated;
* no `science_bands` / `liveness_floors` entry moves;
* `git diff src/` stays empty outside `src/lab/`. ⚠ 2026-08-27: `src/` no longer exists;
  the same property in Rust is that the harness moves no file under `rust/data/` and no
  committed manifest — see §10.

**Experiments become cheap precisely because they cannot accidentally become commitments.**
A harness that wrote YAML would be a calibration tool wearing an experiment's name, and
"changing the number is calibration, a separate act" is already the convention doc's rule.

## 4. Placement

⚠⚠ **SUPERSEDED 2026-08-27 — `src/lab/` does not exist.** Kept for the *rule* it states,
which does survive: the harness lives outside the frozen core and may depend on the domains,
never the reverse.

**The Rust constraint that decides this, and it closes one obvious option:** the harness has
to reach *up* — run scenarios, read each science gate's bound and margin — so it **cannot**
live in `rust/crates/config`, which by invariant sits *below* `domains` and may not reach into
the engine. The precedent the tree already uses for "a tool that drives a run and reports" is
S6's regeneration path: `cargo run --release -q -p station --example regen_goldens`. So the
live candidates are an **example under `station`** (matches the precedent exactly) or a small
crate *above* `station`. **Not decided here** — take it at build time, with §7's proof-the-
override-is-live requirement as the deciding criterion rather than tidiness.

*(As written 2026-08-15:)* `src/lab/`. It already holds `oracle_match.py`, `convergence.py`,
`rk45.py` — the offline-study area, outside the frozen core and outside the port's mirror
obligation. `simcore` purity is untouched: `lab/` may import domains, never the reverse.

## 5. THE OPEN DESIGN DECISION — the injection seam

⚠⚠ **SUPERSEDED 2026-08-27 in its entirety — every symbol below is deleted code.** Options A,
B and C are all about Python modules that no longer exist, and the "the user chose C" verdict
died with the file it shipped. **The seam has to be measured again against the Rust loader
(`rust/crates/config`) before it can be designed** — where params are read, whether a
pre-loaded param object can be handed in, and what the equivalent of "patch the use site, not
the definition site" is when the compiler resolves the call. Do not port A/B/C by analogy.

⚠ **What DOES carry over, because it is language-independent:** the Option-A cost —
*intercepting the definition site when the run reaches a different path silently no-ops* — is
the same defect as §7, and Rust's type system does not catch it. It is the thing to measure
first, not the thing to assume away.

⚠ **And the widened subject changes this section's shape.** The user's 2026-08-16 wording is
*"parameters **and science**"*. Substituting a number and substituting a *mechanism* are not
the same seam, and this section was written when only the first existed. Price them as two,
and say so, before building either.

*(The 2026-08-15 measurement follows, kept as the record of how the Python side worked.)*

**Measured 2026-08-15: there is no injection point.** `build_season(scenario)` takes only a
scenario; `_carbon_context` and `build_plants` call `crop_param_set(scenario.crop)` and then
`load_*_params(crop.paths[...])` **internally** (`src/domains/biosphere/plants.py:99-120`,
`:136-146`). Pre-loaded param objects cannot be passed in. Two routes:

### Option A — patch the loaders at their use site (RECOMMENDED to start)

What the three probes did. Wrap `plants.load_canopy_params` (etc.) so it calls through and
applies `dataclasses.replace`. **Zero `src/` change outside `lab/`; proven to work three
times already.**

Costs, stated rather than discovered later:
* the patch must target the **use** site (`plants.load_canopy_params`), not the definition
  site (`loader.load_canopy_params`) — patching the latter silently no-ops;
* the harness must know which module imported which loader, so the mapping is data the
  harness carries and that data can go stale when an import moves.

### Option C — add a narrow injection seam in `src/`

Let the scenario or `build_season` accept pre-loaded params. Cleaner and immune to both
costs above, but it changes shared code under `src/`.

⚠ **Its price is NOT established — this was asserted while drafting and is unverified.**
The claim was that an added public argument falls under the freeze manifests' completeness
half ("something added but exercised by nothing"), making it an unfreeze question rather
than a refactor. **That half is defined over the params/flows/aux the manifest names**, and
whether an optional keyword argument on `build_season` is in that set at all was never
checked. **Verify before choosing C; do not inherit this paragraph as a finding.** (The
shape it would otherwise repeat is `memory/asserted-attributions-rot`.)

**Still not recommended first** — take it only if A proves fragile in practice.

*(Option B — writing temporary crop directories to disk — is rejected: `CROPS_DIR` is a
module constant under `src/`, so this would write into the repo tree, losing §3.)*

## 5R. The Rust seam, MEASURED 2026-08-27 (before any design)

Nothing here is carried over from §5 by analogy; each line was read out of the tree today.

### The finding that changes the shape of the problem: **there is no runtime load to hook**

All **23** frozen param YAMLs are `include_str!`-ed into the binary at compile time — 15
biosphere (`crates/domains/src/biosphere/params.rs`), 5 sibling (`crates/domains/src/params.rs`),
3 station (`crates/station/src/params.rs`). Python's seam was one line inside a *runtime*
function (`config.loader.load_yaml`); the Rust reference has no such function to hook, and
`grep read_to_string` finds no production path that reads a param file. **So "intercept the
loader" is not available, and §5's options A/B/C are not merely stale — their whole premise
is.**

### What is available instead, and it is STRICTLY BETTER than what Python had

`biosphere/params.rs` exposes **17 `pub fn <loader>_from(text, name)`** entry points beside
the 17 zero-argument loaders. They are the *same* code path: same hand-rolled reader, same
exact-string unit guard, same frozen bounds, same two boundary folds. So an override in Rust
is naturally expressed as **modified YAML text**, and it is *validated on the way in*.

⚠ This is a real gain over the deleted Python harness, and worth stating as one: that one
substituted with `dataclasses.replace` on an already-constructed object, which **bypassed
the schema, the unit guard and the bounds**. An out-of-range experimental value would have
run silently. Through `_from`, it cannot.

### The biosphere has exactly ONE production param load

`crates/domains/src/biosphere/system.rs:873` — `let p = params::biosphere();` inside
`build_season`. **Every other `params::…()` call in the biosphere is inside `#[cfg(test)]`**
(checked by line number against each file's `#[cfg(test)]`: `flows.rs` 1637/1638/3792/4882 vs
1548; `science.rs` 2445 vs 539; `science_gates.rs` 1246 vs 93; `system.rs` 1972/2086/2223 vs
1169). And everything downstream **already takes `&BiosphereParams`** — `compartments(scenario, &p)`
threads it the whole way.

So the injection point is `build_season_with(scenario, &BiosphereParams)` with the existing
`build_season` delegating to it — additive, one funnel, and behaviour-neutral by construction
rather than by measurement. `season_setup` / `run_season_final` / `run_perennial_final` need
the same `_with` pass-through; weather is a separate resolver and is not touched.

⚠ **This single funnel is what makes §7's failure mode small HERE, and the argument does not
travel** — see the next paragraph. Note the funnel is a fact about today's tree, not a
guarantee: a future flow that calls `params::canopy()` at step time would silently escape the
override. That is a gate the harness owes itself, not a risk to accept.

### ⚠⚠ The sibling domains and the station are the OPPOSITE shape

* their `*_from` functions are **private** (`crates/domains/src/params.rs:74-128`) — no
  validated text entry point exists at all;
* their params are loaded at **~15 scattered production sites** with no funnel:
  `station/src/builder.rs:277-279`, `station/src/goldens.rs`, `domains/src/goldens.rs`,
  `authoring/src/flow_registry.rs:314-376`.

**A biosphere-shaped seam does not generalise to them**, and a harness that quietly covers
only the biosphere while presenting itself as universal is §7 wearing a different hat. Say
which half is covered, in the output, or cover both deliberately. §9's first target is
biosphere-only, so this is a scope statement, not a blocker.

### ⚠⚠ The REPORTING half is harder than the seam, and it is what decides §4

§8 asks for *"every `science_gate` bound with its margin and its distance-from-degenerate"*.
Measured: **no non-test binary can obtain a margin today.**

* `science_gates!` emits each gate's `check:` body into `#[cfg(test)] mod gate_tests`, and the
  runs those bodies read into `#[cfg(test)] mod runs`;
* `GATES` *is* public at ordinary compile time — but it carries `bound` as a **`&'static str`**,
  a human-readable claim, not an evaluator. There is nothing to call.

Three routes, none free, **and this is the decision to take next**:

1. **The harness is a test, not an example.** It then sees the gate bodies — but "one command"
   becomes `cargo test`-shaped, and a test that prints a table is an odd thing.
2. **Lift the gate bodies and `runs` out of `#[cfg(test)]` into library code.** Cleanest to
   consume, but it edits the file every science claim in the biosphere is filed in, and the
   macro's whole design point is that the row and the assertion are one declaration — worth
   re-reading `science_gates.rs`'s header before touching.
3. **The harness re-derives the quantities itself.** Rejected on sight unless something forces
   it: *a rule with two copies has one that is stale* is this repo's most-repeated lesson, and
   a margin computed twice is exactly that.


### The route, DECIDED 2026-08-27 by reading `mod runs` and `mod folds`

**Narrowed route 2 — lift the FIXTURE, not the census.** The two `#[cfg(test)]` modules
behind the gates turn out to be exactly what the harness needs, and neither is a claim:

* `mod runs` is a pure fixture — it builds a `Trajectory` (per-step `leaf_c` / `stem_c` /
  `storage_c` / `carbon_pool` / `consumer_c`, plus `rationed`, `events`, `years`) by calling
  `season_setup` + `run_season` / `run_perennial`, **all of which are already public non-test**.
  Its only assertion is the observer-sample-count sanity check. Its `OnceLock` caching is a
  fixture's analogue, not a claim.
* `mod folds` is pure arithmetic over a `Trajectory` — `peak_lai`, `min_ppm`, `peak_w`,
  `floor_ppm`, `t_per_ha`, the segment folds. No bounds, no asserts except two anti-vacuity
  guards.
* each gate body is then two lines: `let peak = folds::peak_lai(t);` then the `assert!` that
  compares it to the bound.

So the harness can have **every quantity a gate reads, computed by the same code the gate
uses**, without touching one gate declaration and without a second copy of any arithmetic.
Route 3 is avoided outright and route 1's ugliness is unnecessary. The macro's "the row and
the assertion are one declaration" invariant is untouched — what moves is the fixture beneath
it.

⚠ **The runs must become param-aware, and that is what ties this half to the seam.**
`runs::trajectory` reaches the frozen params through `season_setup → build_season →
params::biosphere()`. The gates must keep doing exactly that (a gate is a claim about the
frozen tree). The harness needs the same fixture driven with substituted params, so the lifted
function takes `&BiosphereParams` and the cached frozen accessors pass `params::biosphere()`.
⚠ And the override runs must **not** share the `OnceLock` cells — a cached frozen trajectory
returned for a variant run is §7's silent no-op with a different cause.

### ⚠ What this route does NOT give, stated now rather than discovered in the output

**A numeric margin.** `ScienceGate::bound` is a human-readable `&'static str`
(`"5.0 < peak < 8.0"`, `"non_collapsing(floor=5e-4)"`), not an evaluator, and parsing it would
be a fragile second copy of the census — the thing route 3 was rejected for. So the first cut
reports, per gate: the **authority label** (`science_bands` vs `liveness_floors`, §6.2), the
recorded bound **as written**, and the **measured quantity** under baseline and each variant,
with the movement between them. That satisfies §6.3 (opposed movement is visible), §6.4 (no
stored ranking — the table is re-derived every run) and §6.5 (a null result is a row reading
zero). §6.1's distance-from-degenerate is carried where a degenerate baseline is on record
rather than invented per gate.

**Say this in the output itself.** A table that shows quantities while the reader assumes it
shows pass/fail is a worse failure than not printing it.

## 6. Requirements earned by the canopy-provenance session

Each of these exists because reporting *without* it produced a wrong read on 2026-08-15.

1. **Report distance-from-degenerate, never clearance-vs-bound alone.** "The perennial
   liveness floor's clearance falls 5.12 % → 0.40 %" reads as a plant nearly dying. Against
   the measured stunted-regime baseline (0.253, `test_senescence_form.py:2382`) the plant
   moves **2.29× → 2.18×**. The first number alone misled this session's own recommendation.
   Report both, always.
2. **Label each gate's authority in the output.** `science_gates.py:44-47` keeps
   `science_bands` (bound from outside the repo) and `liveness_floors` (tuned to our own
   calibration) deliberately apart: *"merging two claims of different strength under one
   name is this project's recorded failure mode."* A flip list that does not say which kind
   flipped commits exactly that merge.
3. **Opposed movement is a first-class result, not a footnote.** At `k = 0.65` all five
   chamber CO₂ bands loosen while the liveness floor tightens, *for one reason*. Reading
   either family alone gives the wrong answer. The report must surface the opposition.
4. **Never store a ranking; re-derive it.** "The tightest of the five" inverted in six
   commits (`d8f5583`).
5. **State what did NOT move.** The 3.5× uniform-perturbation amplifier did not transfer to
   this knob (+8.3 % on `k` → +0.8 % of shipped peak LAI). A null result is the finding.

## 7. ⚠ The failure mode this harness is structurally prone to

**A probe that shims a path nothing calls proves nothing.** This tree has already shipped
that defect: `cc44b41` repaired a cross-port gate that had gone vacuous because its ULP
probe shimmed a dead `exp` — and the audit behind `post-roadmap-canopy-provenance.md` found
`intercepted_fraction` now has **no caller in `src/` at all**, surviving only in tests and
the Rust mirror.

A value-switch harness has exactly this failure mode by construction: patch the wrong
symbol, get a clean run, read the baseline, and report "no effect" as a finding.

**Requirement: the harness must prove the substitution took effect, or fail loudly.**
Minimum: assert the run's outputs differ from baseline for a value known to matter, and
refuse to report "no change" without having verified the override was live. A silent no-op
must be impossible, not merely unlikely.

## 8. Scope

⚠ **2026-08-27: read every `src/` path below as its Rust home per §4, and add the seam itself
to the "In" list — it is no longer built.** The out-list is unchanged and still binding.

**In:**
* one module in `src/lab/` — take a set of `{param: value}` substitutions plus a scenario,
  run, return the readouts (canopy peak, DVS at peak, harvest, chamber CO₂ lows, every
  `science_gate` bound with its margin *and* its distance-from-degenerate, rationing count);
* a comparison mode — baseline vs N variants, tabulated;
* the live-override self-check of §7;
* tests for the module.

**Out:**
* a central register (§2);
* changing **any** parameter value, in YAML or otherwise;
* the `extinction_coef` decision — still open, still the user's;
* the 0.65 ceremony (13 goldens, both manifests, Rust mirror, 3 characterization re-pins);
* touching the perennial liveness floor. It is *misread*, not broken — moving 0.55 is the
  co-adaptation refused four times, and its own comment says so.

## 9. First target

Reproduce the canopy-provenance A/B table as one command: `k ∈ {0.60, 0.65, 0.68}` against
the open-field season and the five chamber gates.

⚠ **The harness must not be read as taking the decision.** It regenerates the evidence the
decision was already priced on; the choice between Penning de Vries (0.60, coherent with the
Goudriaan quadrature shipped 2026-08-15) and Soltani & Sinclair (0.65, wheat-specific, our
exact equation) stays open.

**Then the second layer:** whichever value ships, pin the disagreement as a test — three
sources, three values, two not independent, and the measured consequence. The harness
produces the number; the pinned test freezes the finding. They are complements, not
alternatives, and §1 lists the three precedents.

## 10. Exit criteria

⚠⚠ **RE-EXPRESSED 2026-08-27 — the 2026-08-15 list gated a deleted tree.** Three of its five
criteria named Python tooling that no longer runs, so as written the list was unmeetable, not
merely stale.

* No param YAML under `rust/data/`, no golden, no committed manifest and no science-gate bound
  moves. (`git status` clean outside the harness's own files after a full harness run.)
* The §9 table regenerable by **one command**, on a tree with `extinction_coef` unchanged.
* A test proving a **mis-targeted override is detected**, not silently reported as
  no-change — §7, and the criterion that outranks the rest.
* `cargo test` green + `cargo clippy --all-targets -D warnings`.
* `simcore` purity untouched — the zero-third-party-deps rule, already gated by S4's structure
  half; point at that gate rather than re-asserting it here.

*(As written 2026-08-15:)* `git diff src/` empty outside `src/lab/`; no manifest hash, golden,
or gate bound moved. · The §9 table regenerable by one command. · A test proving a mis-targeted
override is detected. · Suite green (`uv run pytest -n 12`), `ruff`, `pyright`. · `simcore`
imports unchanged (stdlib only).
