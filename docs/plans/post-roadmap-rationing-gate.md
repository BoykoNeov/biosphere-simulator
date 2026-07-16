# Post-roadmap: the rationing gate — make the `dt` hazard loud

**Status: COMPLETE (2026-07-17).** Bucket 2 (capability gaps). The user chose it from the
post-roadmap menu; it is the item Tier 1 named and deferred, not a new idea.

**One line:** `authoring.run_scenario` now **raises** when the Euler arbitration backstop
fires, in both ports — closing the one surface that could detect a habitat asphyxiating
its crew and say nothing.

---

## The charge

Tier 1 created a hazard by registering frozen flow types for authoring, measured it, pinned
it, and deliberately left it unfixed with an explicit hand-off
(`docs/authoring-reference.md`, "The `dt` constraint"):

> Making it loud (a strict mode, a run summary, a Godot warning) is a capability-gap item
> with its own design — deliberately not folded into a registration unfreeze.

This is that item, and that design.

## The hazard (unchanged by this work — read this twice)

Every frozen rate constant was sized against the `dt` of *its own frozen scenario*, and
that sizing **is** the flow's positivity argument — but it lives in a YAML comment, and an
author picks `dt`. `eclss.co2_scrubber`'s `k_scrub·dt` is `0.06` at the frozen `dt = 60`
and **3.6** at `dt = 3600`: the donor-controlled draw demands 3.6x the entire CO₂ pool in
one step.

The arbitration backstop catches it — and *that is why it was invisible*. It scales the
over-draw so no stock goes properly negative. So the run:

* does not raise,
* **conserves every quantity, every step** (the ledger gate never trips),
* completes normally, `rationed = 37`,
* and ends with `cabin_o2` at `-1.4e-14` — **a cabin with no oxygen**.

The only signal was the count, and `states, _, _ = run_scenario(built)` — the natural call —
discards it. **The platform reported a successful, mass-conserving run of a habitat that
killed everyone aboard.**

## What was decided, and why it was not a judgement call

**Decision: raise by default** (user-selected, from `raise` / `warn-but-return` /
`RunResult`-struct-only).

The decisive argument is that **this is a consistency fix, not a new policy.** Every other
layer had already reached the verdict; `run_scenario` was the lone dissenter:

| layer | its verdict on `rationed > 0` |
|---|---|
| the goldens | assert `rationed == 0` — a failing gate |
| `simcore.integrator.StepReport` | its own docstring: *"a failing gate, not a warning"* |
| RK4 (`check_no_overdraw`) | **hard error** (`ArbitrationError`) on the identical condition |
| `station.objectives` | scores a rationed run `survived = False` |
| `authoring.run_scenario` | **returned an integer and hoped** |

**A counter-example was raised and then dissolved.** Two crossport tests assert
`rationed > 0` as *correct* behavior (a deep blackout rations `power.load_draw`), which
looked like proof that rationing is sometimes legitimate physics and must not raise. It is
not: those are the **station/Godot** path, and that path *already calls it failure* — the
`no_rationing` objective criterion, `survived = False`, *"a rationed station is a failure
even at the horizon"*. It reports rather than raises because a **player** should see the
failure; an **author** calling a library function gets an exception. Same verdict, two
idioms. Within the authoring path, every existing test asserts `rationed == 0` except the
hazard pin — so the blast radius was exactly one test and **zero existing content**.

## What shipped

* **`authoring.errors.RationedError`** (Python) / **`ErrorKind::Rationed`** (Rust). The
  message names the count, the `dt`, and the escape hatch, and points at the doc.
* **`run_scenario` raises**; `allow_rationing=True` (Rust:
  `run_scenario_allowing_rationing`) is the opt-in escape hatch — for *inspecting* a
  rationed run, never for making a scenario "work".
* **The pin flipped, not deleted** (`tests/test_authoring_dt_hazard.py`, 7 tests) — see
  below.
* **The Rust mirror, same increment** (user-selected over "Python now, Rust next"), + 3 new
  Rust tests. **Rust rations 37 times too** — an unplanned cross-port confirmation the
  hazard is identical on both sides, now pinned.
* **Doc updated** (`docs/authoring-reference.md`, "The `dt` constraint"): the section that
  told authors the failure is silent no longer says something false.
* **A stale pointer fixed**: `tests/authoring/scenarios/eclss_cabin.yaml` cited
  `tests/authoring/test_frozen_flow_dt_hazard.py` — a path that **has never existed**.

### Two design choices worth their own line

**`RationedError` is deliberately NOT an `AuthoringError`.** Both ports define that class
as what is decidable *from the file structure alone, before any step runs* (its module
docstring is explicit, and so is Rust's). Rationing is decidable only by running, and the
same file at a smaller `dt` is fine. Subclassing would have been convenient and would have
made both docstrings lies. Rust cannot express the split as a second type without widening
`run_scenario`'s error across 61 construction sites, so it carries the distinction as
`AuthoringError.kind` — match on it; the message text is explicitly not a parity target.

**The build-time `k·dt < 1` check was considered and rejected as *the* fix** (advisor
catch). It is seductive — it would fail before running, and the doc already tabulates the
per-flow constraints. But it is **partial**: it cannot see `eclss.crew_metabolism`, whose
failure is `forcing·dt > stock` (14.4 mol/step drawn from a 10 mol cabin — an *independent*
mechanism this file's own tests isolate), nor any coupled multi-flow dynamic. The runtime
`rationed` signal is the only **general** detector. And it would live in the **frozen
registry** — an unfreeze, which this scope was chosen to avoid.

## The exit criteria, held

* **Nothing unfrozen.** All three manifest gates green, **unregenerated**; no manifest
  file touched. `run_scenario`/`errors` are not in the authoring manifest (which freezes
  the grammar, VM, file schema, flow-type registry, param loaders) — the doc pre-authorized
  this surface by name.
* **No golden moved.** `git diff tests/regression/golden/` empty.
* **`git diff src/simcore/` empty.** The core was not touched; this is a boundary-layer
  behavior change.
* **Gates:** 1732 Python passed / 1 skipped (opt-in oracle), 97 crossport passed, all Rust
  green, `clippy --all-targets -D warnings` clean, ruff check + format clean, pyright 0.

## What this did NOT do — the sentence that keeps the record honest

**The silence is fixed. The hazard is not.** The physics, the params, and every frozen
sizing are untouched. `k_scrub·dt` is still `3.6` at `dt = 3600`; the cabin still
asphyxiates, and `test_the_underlying_hazard_is_UNCHANGED_only_its_silence_was_fixed`
asserts that it still does, at the same 37 firings and the same airless cabin, via the
escape hatch. **We made the failure loud. We did not make the scenario work.** A reader who
takes a green run as "the `dt` hazard is handled" has it exactly backwards — the
composability constraint (no `dt` is natural to both ECLSS and Thermal) is untouched, and
authors still must design around the table in the reference.

That distinction is why the pin was **flipped rather than deleted**, per the project's
`lab.fit_order` / `test_oracle_gap.py` idiom: the file still measures the known-wrong
behavior; what changed is that the harness now refuses to return it.

## Deferred, by name

* **A build-time `k·dt < 1` precondition per flow type** — early warning, but partial (see
  above) and a **registry unfreeze**. If ever taken: it complements the runtime gate, never
  replaces it.
* **Per-flow attribution in the error.** The message names the count and `dt`, not *which*
  flow rationed. `StepReport.rationed` is a bare count; widening it is a `simcore` change
  with a cross-port cascade. The station's inspection layer already surfaces `min_scale`
  per flow — that is where the shape would come from.
* **An author-facing run CLI / Godot banner.** There is no run CLI today (`station::sim`
  has no scenario-file dispatch — the standing bucket-2 item); `run_scenario` is the top of
  the author's stack, which is *why* the raise lives there.
* **`eclss.o2_makeup`'s unclamped reversal** — the *sibling* hazard in the same family
  (frozen scope escaped by authoring), still open: wire `cabin_o2` above the setpoint and
  the "makeup" flow silently vents cabin O₂ back into the tank. It conserves, it does not
  ration, **and so this gate does not catch it.** Documented in the reference; a real
  candidate for the next bucket-2 increment.

## The transferable lesson

**Conservation is not survival.** The backstop clamps at zero rather than going negative,
so a run can balance perfectly, every step, and still have emptied the one stock keeping
the crew alive. The every-step ledger gate — the project's strongest safety property, and
the one an authored graph leans on hardest — is *structurally incapable* of catching this
class of failure. An authored graph now needs **two** things to be sound: it balances,
**and** it never rationed.

This is the same shape as scope (A)'s finding one level down: *an explanation is only as
good as the measurement behind it.* Here: **a safety property is only as good as the
failure it can see**, and a guard that silently succeeds is indistinguishable from a system
that never needed it.
