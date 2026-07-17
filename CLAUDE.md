# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from commit 1;
biosphere is the first domain. Python is the canonical reference ("laboratory");
a native Rust core and a Godot front-end consume it. End goal (reached): a
science-credible Godot station sim that runs the *same* simulation headless.

**The roadmap (`roadmap_extracted.txt`) is COMPLETE through Phase 9 — its last
phase.** The simulator "is not really about plants; it is about closure of matter
— and energy — cycles" (roadmap).

## Where the detail lives (this file is a map, not a record)

- **`docs/plans/phase-*.md`** — the living record. Each carries its phase's
  design *and* per-step `COMPLETE`/`RESOLVED` outcome blocks (findings, the
  advisor calls, what was deferred and why). **This is the source of truth for
  what happened.**
- **`docs/*-reference.md` (+ `.manifest.json`)** — the freeze contracts.
- **`roadmap_extracted.txt`** — the original charge.
- Reference material: `docs/bvad-reference.md` (NASA BVAD Table 3-31 — the
  primary source behind the crew params), `docs/param-file-conventions.md`,
  `docs/perf-baseline.md`, `docs/reuse-and-licenses.md`.
- Do not restate any of it here. See "Working style".

## Phase status

| Phase | Topic | Status |
|---|---|---|
| 0 | Engine skeleton | COMPLETE |
| 0.5 | Numerical foundations | COMPLETE |
| 1 | Single producer | COMPLETE — quantitative oracle match DEFERRED (user decision) |
| 2 | Closed chamber (producer + decomposer) | COMPLETE |
| 3 | Modular biosphere / consumers | COMPLETE |
| 4 | Closed biosphere | COMPLETE — **biosphere FROZEN as the reference** |
| 5 | Sibling domains (power/thermal/eclss/crew) | COMPLETE |
| 6 | Station integration (cross-domain coupling) | COMPLETE — **station FROZEN as the multi-domain reference** |
| 7 | Native core (the Rust port) | COMPLETE |
| 8 | Godot front-end | COMPLETE |
| 9 | Scenario authoring & modding | COMPLETE — **the author-facing platform FROZEN** |

Each row's detail is in `docs/plans/phase-<n>-*.md`.

**Post-roadmap** (the roadmap has no Phase 10 — work past it is chosen, not scheduled):

| Work | Status |
|---|---|
| The first authored habitat (`scenarios/algae_habitat.yaml`) | COMPLETE — `docs/plans/post-roadmap-authored-habitat.md` |
| Tier 1: grow the flow registry (9 Power/Thermal/ECLSS flows + 3 loaders) | COMPLETE — the authoring platform **unfrozen** (`flow_types` 3→12, `param_loaders` 2→5); `docs/plans/post-roadmap-flow-registry-growth.md` |
| Tier 2: the grammar — `monod` | COMPLETE — the **grammar unfrozen** (`expr_nodes` 7→8); saturation is now sayable. Forced by the frozen `chamber.oxygen_limitation_factor`, whose kernel it mirrors bit-exactly; bare `/` stays deferred. `docs/plans/post-roadmap-grammar-monod.md` |
| Bucket 3 scope (A): validation — diagnose + pin the oracle gap | COMPLETE — **no golden moved, nothing unfrozen**. The gap is **structural, not merely uncalibrated**: the canopy never bootstraps (1.75 % light interception at sowing vs the oracle's 97.8 %; LAI peaks day 32 of ~305 and collapses *before* anthesis) and phenology runs ~1.6x fast (no vernalization) — two *independent* missing sciences; param values are only the **third** cause. So the deferred "quantitative oracle match" is **not a calibration task**. Pinned by `tests/test_oracle_gap.py`; `docs/plans/post-roadmap-validation.md` |
| Bucket 3 scope (B): the full oracle match (vernalization + juvenile canopy expansion, then recalibration) | OPEN, not started — a **phase, not a step**: new science ×**4** (clean-room, hand-mirrored into Rust), then the 7 frozen biosphere goldens + **cascaded station goldens** under the biosphere unfreeze discipline. **Scope (C) grew it twice**: the decomposer rates are a third piece, and round 2 added a fourth — `n_senescence_rate` is a bare constant where its own cited source makes the rate a **function of development stage** (zero before anthesis), so it is a *form* gap that recalibration cannot absorb. Round 2 also stocked the shelf: ~15 unopened scope-(B) references (Soltani & Sinclair, Teh, Luo & Smith) now sit in `sources/` |
| Bucket 2 (cont.): the export-fidelity hazard — **the one rationing cannot see** | **DOC+PINS COMPLETE; the fix is PLANNED, not started.** Scope premise was false *again* (cf. scope C): the "open" o2_makeup venting hazard was **already documented in 3 places and the prose was right** — just never measured. Measuring it found a worse, invisible one. `eclss.o2_makeup` is the registry's **only demand-controlled** flow: its draw ∝ the *setpoint error*, not the stock, so near the setpoint it **never over-draws and the backstop never fires**. Every other row is donor-controlled (draw ∝ stock), over-draws at `k·dt>1`, and **is** caught. So: `dt=900` exports `12 → 8.4 → 11.28 → 8.976` around a 10.0 setpoint with **`rationed = 0`**, conserving, endpoint *correct*; `dt=1000` swings **12↔4 forever** while `o2_supply` drains past −800 mol, reported clean. The full cabin is protected **only by coincidence** (`k_scrub·dt=1` and `k_makeup·dt=2` both land at `dt=1000`). **`k·dt < 1` STAYS OPERATIVE** — an advisor catch killed a draft that relaxed it to the textbook stability bound `< 2`: `< 2` answers "does the solver diverge", `< 1` answers "is the export usable by a neighbour"; we couple domains, so `< 1` governs. **No clamp** (symmetry IS the restoring force). Not a bug in the flow — `dx/dt = k(S−x)` cannot oscillate; this is explicit Euler failing at too large a step, a **solver** property (pinned: first-order convergence + the deadbeat prediction hit exactly). Nothing unfrozen, no golden moved. `tests/test_authoring_export_fidelity.py` (12 pins) |
| Bucket 2 (cont.): **multi-rate authoring** — the chosen fix | **IN PROGRESS — Steps 1–5 of 7 done; the authoring platform is UNFROZEN and the manifest has moved.** `docs/plans/post-roadmap-multirate-authoring.md`. An author picks a **coupling cadence**, not a global `dt`; each flow sub-steps at the `dt` its own rate constant demands. **Step 1** pinned the fact the phase rested on: `n_sub=1` + an empty slow set reproduces the frozen ECLSS golden **byte-for-byte** — measured, not inferred. **Step 2** landed the knob (`ScenarioSpec.n_sub`, `FlowSpec.rate_class`, the disjoint slow/fast registries on `BuiltScenario`). **Step 3** made it drive: `run_scenario` at master `dt=3600` + `n_sub=60` lands on the truth (`cabin_o2 == 8.0`, `rationed == 0`) while exporting **hourly** — the user's charge answered end-to-end; no golden moved, manifest untouched (Step 3 adds no schema field/integrator/flow type). Rulings worth carrying: the partition check lives in the **interpreter, not a pydantic validator**, because a *bundle* may contribute `rate_class: slow` and a schema-level check runs before `apply_includes` would ever see it; the **manifest regenerates at each step that moves the surface**, never batched to a final ceremony (the gate is plain equality, so batching disables it for the steps in between); and **the branch, not the identity, preserves the goldens** — `run_scenario` routes single-rate scenarios down the pre-multi-rate loop **verbatim**, so the 25 goldens never reach the driver. That demoted Step 1's identity from *load-bearing* to *corroborating*, and **superseded its own instruction** to re-point the identity pin through `run_scenario`: `is_multirate` is false at `n_sub=1`, so the re-pointed test would drive the **single-rate path** — the wrong path, and a duplicate of `test_authoring_frozen_flows.py`. A leaked branch would have kept **every golden green** while silently resting all 25 on the identity; pinned by `test_a_single_rate_scenario_never_touches_the_driver` (monkeypatched to raise). **Step 4** built the scenario the reference calls **impossible** ("there is no `dt` natural to both domains") and the sentence is now measured false: `eclss_thermal_habitat.yaml` — master `dt=3600`, ECLSS fast `n_sub=60`, Thermal slow — runs `rationed == 0` with the cabin at `o2_eq` and the node warming 102.7→277.4 K. The constraint had **two halves and both are escaped**: the shared `dt` is *unsafe* (single-rate `dt=3600`: 840 firings, cabin **72.0** vs truth 8.0 — diverged, not drifted) and the safe shared `dt` is *wasteful* (`dt=60`: clean, but **20160** Thermal evals for a `τ` of ~65 steps). **The payoff is 30×, NOT the 60× this plan predicted** — an advisor catch *before* the measurement: 60 is the cadence ratio, but **Strang steps the slow set at `dt/2`, twice per master step**, so `20160/(336×2) = 30.0`; the missing factor of two is Strang's bill for the order/safety choice, and Lie would realize 60× at a lower order. Honest whole-run number: wall improves only **2.31×** (multi-rate saves the *slow* domain's work, and here the slow domain is the cheap one) — so the win lands where the slow set is expensive, i.e. the biosphere, the domain multi-rate cannot reach yet. What Step 4 does **not** prove: the two domains share no stock (no *quantity*, even), so the Strang operators commute exactly and **no coupling fidelity is exercised** — **forced by the registry, not chosen** (no ECLSS flow carries a heat leg; the cross-rate boundary and the cross-stock boundary never overlap, since coupling lives *within* a domain or across *same-timescale* domains). Pinned as an assertion, not a caveat, so a future coupling registry addition goes red. What **is** new: the **first non-empty slow set** ever driven through `run_scenario`. A Step-2 ruling also reached further than Step 2 knew: **"the same graph, single-rate" is not `n_sub=1`** — the interpreter refuses `n_sub=1` with a non-empty slow set, so going single-rate means dropping the `rate_class` keys too (the refusal's own message says exactly that, on the first author who needed it). **`simcore.multirate` already exists and is proven** (Phase 0.5) — this is a *consumer* phase; `git diff src/simcore/` must stay empty (it does; the aux tripwire therefore lives in `run.py`, multi-rate branch only — and *is testable there*, since `interpret` can't express aux). Rejected alternatives, priced: the **implicit integrator** (the instinctive "rework the maths") is the repo's largest unfreeze **and does not meet the bar** — backward Euler gives 10.61 vs truth 8.33 at `dt=1800`; it buys *sane*, not *near*. **Multi-rate is the performance enabler, NOT the hazard closer** (advisor): an unsafe **effective sub-step** is the same hazard one level down. `n_sub=2` at `dt=3600` *does* raise through the harness — but that is **luck of shape** (the backstop sees the donor-controlled scrubber; `o2_makeup` stays invisible). **Step 5 CLOSED it** — and its headline is that **this plan's own formula was the bug**. The plan (and the sentence just above) specified `k·(dt/n_sub) < 1`; that is right for the **fast** set and **false-PASSES the slow set in the UNSAFE direction**, because Strang steps the slow set at **`dt/2`** regardless of `n_sub` (measured: scrubber slow at `dt=3600`/`n_sub=60` — formula reports `k·h = 0.06` ✓, truth is **1.8**, 24 rationings, `cabin_co2` → **0.0**). **It is the same Strang fact that made Step 4's predicted 60× a measured 30×** — one blind spot (*reasoning about `n_sub` as though it governed both rate classes*), two wrong claims, one of them a **safety predicate**; the formula was even written *after* Step 4's catch. Three cases, named once in `interpreter._effective_step`: single-rate → `dt`, fast → `dt/n_sub`, slow → `dt/2`. The `dt/2` divisor is **coupled to `run._SPLIT` and pinned, not commented** (advisor) — Lie steps the slow set at full `dt`, so a flipped split would loosen the check 2× silently. **The behavior change is real and observable**: the `k·dt` family moved from run-time `RationedError` to **build-time `AuthoringError`**, which broke **22 committed pins across 6 files** — and *that* is the evidence it works, since those tests could no longer construct their own subject (`allow_unsafe_step=True` is the study hatch, the `allow_rationing` idiom; **both are needed and neither implies the other** — one opens the build, one the run). **Build time is the locus for the PACK, not the convenience** (advisor): a param pack can inflate a gain past every frozen guard (unit ✓, bound `>0` ✓ — a gain has no `dt`-independent ceiling), and a pack's values exist only *after* `interpret` resolves them, so `run_scenario` **structurally cannot see it**; with `o2_makeup` demand-controlled, that pack would otherwise export an oscillating cabin with `rationed == 0` and no gate complaining. **A manifest hole nearly shipped**: adding `rate_params` to `FlowTypeSpec` without adding it to the manifest test's `_flow_types()` derivation leaves the equality gate **green** while the field is never frozen — an equality gate is blind to a field absent from *both* sides (the scope-C "provenance-only edit nothing catches" shape); fixed atomically + given teeth from outside the derivation. Honest scope unchanged: **"the platform catches the `k·dt` family", never "your dt is safe"** — `radiator_reject` (`τ ≫ dt`; **"≫" is not a predicate**), `crew_metabolism` (state-dependent) and authored `kinetics` (decision B) declare `rate_params=()`, a ruling not an oversight, and **neither gate subsumes the other**. **Step 6 inherits an OPEN DECISION** (advisor, post-Step-5): the check sits in `interpret`, **upstream of the deliberate library-vs-interactive split** — `godot_bridge::build_session_from_file` builds via `authoring::load_scenario`, so mirroring it into Rust makes the *interactive* path refuse an unsafe scenario at build and **the player never sees the cabin die**, silently narrowing the designed *"library caller → exception; interactive session → visible diagnostic"* (which was scoped to **rationing**, the only gate that existed when written). No tension today (Rust unmirrored). Both readings defensible — the bridge *already* refuses rk4 at build vs "watch the cabin die" **is** the study-the-unsafe-run case the hatch exists for — so **decide it, don't inherit it**. **The meta-finding: that is the THIRD doc claim this phase falsified** (Step 4's "no `dt` natural to both domains", Step 5's "still deferred", now "does not raise — deliberately"): **a phase that expands what the platform CAN do systematically invalidates the sentences written when it could not** — so on finishing a step, grep the reference doc for claims about the limitation just removed, including the sections you did *not* touch. All three marked in place; Step 7 owns the rewrite |
| Bucket 2: the rationing gate — make the `dt` hazard loud | COMPLETE — **nothing unfrozen, no golden moved**. `authoring.run_scenario` now **raises** `RationedError` (Rust: `ErrorKind::Rationed`) when the Euler backstop fires; `allow_rationing=True` / `run_scenario_allowing_rationing` is the escape hatch. This is the item Tier 1 named and deferred, and it is a **consistency fix, not a new policy**: the goldens, `StepReport`'s own docstring ("a failing gate, not a warning"), RK4's hard error, and `station.objectives` (`survived = False`) had *all* already called rationing failure — `run_scenario` was the lone surface that detected it and returned an integer. The apparent counter-example (blackouts legitimately ration `load_draw`) **dissolved**: that is the station path, which already scores it a *lost game* — same verdict, player idiom vs author idiom. **The silence is fixed; the hazard is NOT** — the cabin still asphyxiates at `dt=3600` (37 firings, both ports — a free cross-port confirmation, now pinned). `docs/plans/post-roadmap-rationing-gate.md` |
| Bucket 3 scope (C): cite the no-oracle params | COMPLETE — **a partial discharge, and a second structural finding**. The count was **29**, not 15 (the "5" were *files*, holding 13 params; +2 in `water_cycle.yaml`, which scope (A)'s 7-PP/5-non-PP split omitted entirely; +4 station folded in). Outcome: **8 CITED, 14 DESIGN, 7 still TODO(cite) + a measured finding** — because half the values have no citation to find (a *form* citation never licenses a *value*; citing a sizing choice IS the fabrication failure mode). **The headline is not the citations:** the **decomposer side runs fast** (litter decay ~1.5× above a 293-value global max; mineralization ~3.9× the Stanford & Smith mean; microbial respiration **~2.5–28×**) — the mirror of scope (A)'s canopy collapse, on the *return* side of the loop, and a new scope-(B) candidate. Also found: a **cross-kingdom param copy** (herbivory `o2_half_saturation` mirrored from the microbial file, ~4× sharper than animal physiology) and **two unverified citations** in the frozen tree (Dunn 2011 / Divya 2009 — paywalled, never opened). No golden moved; 26/26 artifacts byte-identical. **ROUND 2 (2026-07-16): the user supplied 10 of the 12 retrieval-list sources; the open items were re-run against the primaries.** Round 1's *inferences* held to the digit (Davidson's Km: predicted "~1200×/~3 orders/~150 µM/~58 %", actual **1210×/3.08/149 µM/58 %**), but its **flagship number moved**: Parton 1987 gives CENTURY's active-SOM max at 7.3/yr, so microbial respiration is **~2.5×** against *that* anchor — while RothC's BIO (0.66/yr, now primary) keeps 28×. **The two lineages disagree ~11× on what "the microbial pool" means, and that spread is the finding** — both recorded, because taking the flattering 2.5× is the re-anchoring trap this scope already refused. Retrieval list now **3 items, not 12** (Stanford & Smith, Dunn, Divya — all three still unread, so the miscitation risk survives). `docs/plans/post-roadmap-citation.md` |

## The freeze contracts (four; each has an unfreeze discipline — follow it)

Changing anything a manifest names is an **unfreeze event**, not a refactor —
each doc spells out the ceremony. The three manifests have a paired gate
(`tests/test_freeze_manifest.py` biosphere, `test_station_freeze_manifest.py`,
`test_authoring_freeze_manifest.py`) that owns **completeness** (something added
but exercised by nothing); the goldens own **values**.

**The gap between the two (scope C, 2026-07-16): a provenance-only edit is an unfreeze that
NOTHING CATCHES.** The manifests' per-file sha-256 is **provenance, not an assertion** — no
test compares it. So editing only a param's `source:` (which the loaders *record but never
parse*, and which the Rust generator never emits) moves the recorded hash, keeps every golden
byte-identical, and turns **nothing** red. It is still an unfreeze by each doc's own
definition — the ceremony is simply **honor-system there**. That cuts both ways: mechanically
trivial, and *because* nothing catches a skipped discipline, follow it deliberately
(advisor review → regenerate the manifest as the git-visible record → document).

| Contract | Freezes | Doc |
|---|---|---|
| Biosphere | the reference science: Euler/`dt=1`, 17 flows, aux, 13 params, 7 scenarios→goldens | `docs/biosphere-reference.md` + manifest |
| Station | the multi-domain assembly: sibling flows/params, the 4 seams, 13 scenarios→goldens (biosphere **delegated**) | `docs/station-reference.md` + manifest |
| Native port | the cross-port **tolerance** contract (not code): the 3 tiers + measured bands | `docs/native-port-reference.md` |
| Authoring | the author-facing **platform**: grammar, file schema, VM node/op set, flow-type registry | `docs/authoring-reference.md` + manifest |

`docs/phase-8-reference.md` is deliberately a doc with **no** manifest (Phase 8
added a consumer and changed no science).

## Layout

- `src/simcore/` — the pure engine. `src/domains/{biosphere,power,thermal,eclss,crew}/`
  — no domain imports another. `src/station/` — the assembly layer that owns all
  cross-domain wiring. `src/authoring/` — the declarative scenario platform
  (boundary code). `src/{sim_io,config,lab}/` — boundary.
- `rust/crates/{simcore,domains,station,authoring,godot_bridge}/` — the native port.
- `godot/` — the front-end (a subdir, so Godot's importer never scans the tree).
- `scenarios/` — authored **content** (runtime artifacts, never reference). Distinct
  from `tests/authoring/scenarios/`, which are fixtures / cross-port anchors.
- `tests/regression/golden/` — 25 golden files; the **20** in
  `tests/crossport/tiers.json` (7 biosphere + 13 station) carry the cross-port
  tier contract.

## Purity invariants (the exit criterion of every port/consumer phase)

- **`git diff src/` must come back empty** for Phase 7/8/9 work. The Rust port,
  the Godot front-end, and the authoring platform are *consumers* — they never
  edit the Python reference to suit themselves.
- **`gdext` appears in `rust/crates/godot_bridge` and nowhere else.** Engine
  crates carry no Godot types.
- **The port has NO reference authority.** A Rust/Godot run that surfaces a
  Python bug is an unfreeze-discipline finding, never a silent native-side fix.
- **"Authored ≠ validated."** Authored artifacts are runtime-only and never
  frozen; they get conservation + determinism, not scientific endorsement.

## Non-negotiable invariants (the things that are easy to get wrong)

- **Core is pure.** `simcore/` imports **stdlib only — zero third-party deps**
  (no numpy/pint/yaml/json/plotting/UI/net). Boundary stuff lives in `sim_io/`
  and `config/`. This keeps the Rust port mechanical.
- **Flows return structured per-stock legs, never a net delta.** A flow is an
  atomic stoichiometric transfer; arbitration scales the *whole flow*.
- **Every flow is internally balanced.** The "outside" is explicit BOUNDARY
  reservoir stocks; `Inputs = Outputs + ΔStored` where Inputs/Outputs are
  boundary deltas. Conservation is asserted every step — a failure is a bug.
- **Determinism:** bit-identical within a build. Time is an **integer step count**
  (`t = n*dt`, never `t += dt`). **Canonical (flow-id) order on every reduction**
  (demand sum, scaling, delta sum). Cross-port (Rust) is tolerance-gated.
- **Arbitration backstop is Euler-only and rare.** It runs always, counts
  firings; golden runs assert the count == 0. Under RK4+, a needed scale is a
  **hard error** (positivity comes from kinetics).
- **Extinction conserves mass:** POPULATION stock below threshold → 0 with the
  residual routed to the loss-sink. POOL stocks are never zeroed-with-loss.
- **RNG** is a counter-based, keyed, pure-Python generator in `simcore`, keyed by
  `(seed, key, n)` so draws are order-independent. No sequential-state RNG.
- **Units** validated at the boundary (pint, in `config/`); the core stores plain
  floats + a canonical-unit label.
- **Parameters are data** (YAML + pydantic schema). No hardcoded coefficients.

## Reuse & licensing (see docs/reuse-and-licenses.md)

- Reimplement science from **primary literature**; cite the paper, not PCSE.
- **PCSE is EUPL (copyleft): offline validation oracle only, never ported or
  imported.** The WOFOST param YAML repo has no license — don't copy it.
- Project's own license is **BNCL-1.0** (Boyko Non-Commercial License v1.0) —
  free to use/modify for non-commercial purposes; commercial use requires
  separate written permission from the copyright holder.

## Commands

```
uv sync                 # install/lock deps
uv run pytest           # tests (pytest + hypothesis)
uv run ruff check .     # lint
uv run ruff format .    # format
uv run pyright          # types
```

`cargo test` + `cargo clippy --all-targets -D warnings` in `rust/`.
Markers: `-m slow` (opt-out, ~9 min), `-m oracle` (opt-in).

## Testing

- Prefer **test-first** for engine invariants. Use **property-based** tests
  (hypothesis) for universal laws: conservation, non-negativity, order-independence.
- Golden/regression snapshots use **hex-float** for exact comparison.
- Never weaken or delete a test to make it pass; fix the code or flag the gap.

## Working style

- Plan before non-trivial work; keep `docs/plans/*` updated as living docs.
- Keep this file lean. Put detail in `docs/`, not here.
- Repo etiquette: branch before committing; Conventional Commits.
  (Commits keep the harness-required `Co-Authored-By: Claude` trailer.)
