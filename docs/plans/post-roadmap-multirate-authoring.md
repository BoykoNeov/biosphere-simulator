# Post-roadmap: multi-rate authoring — the author picks a coupling cadence, not a global `dt`

**Status: PLANNED, not started.** A **phase, not a step** — an authoring unfreeze
(schema + interpreter + run harness), the Rust mirror, the freeze manifest, and the
cross-port tiers. Nothing below has been built. The design-level advisor pass on the
author-facing knob is **owed before the first frozen-surface change**.

Predecessors: `post-roadmap-flow-registry-growth.md` (Tier 1 created the `dt` hazard by
registering the flows), `post-roadmap-rationing-gate.md` (made the *donor-controlled* half
loud), and commit `51c8c11` (measured the *demand-controlled* half — the one rationing
cannot see).

## The charge, in the user's own framing

> "big dt should produce the same results as a small dt … if we calculate every half an
> hour and at half an hour it is 0 or 5 or 10, or anything not near the calculated value
> which we get at 1 second dt, then this value is **exported at half an hour to other
> systems, and we get a completely different behavior at other systems**. Refuse other dt —
> maybe, maybe other solutions. We must take into account multiple factors, **performance is
> one of them**."

That is the correct requirement, and the guess in it was exact: measured, `dt = 1800`
exports **0.0000 mol** where truth is **8.3279**.

Two halves, and they need different answers:

* **The equilibrium already satisfies it.** `dt` cancels at the fixed point
  (`0 = k(S−x*) − Con ⇒ x* = S − Con/k`), and it is measured across a 500× span of `dt`
  landing on 8.0000000000 (`test_the_equilibrium_is_dt_independent_across_a_500x_span`).
  Forced flows are *exactly* `dt`-invariant. The principle holds where it can.
* **The trajectory cannot satisfy it, and no rework makes it.** This is discretization
  error, universal to every explicit integrator. The pins show textbook first-order
  convergence (halve `dt`, halve the error) and the analytic deadbeat prediction hit
  exactly at `k·dt = 1` — *the model reproduces its own closed form*. The maths is sound;
  the step is too big.

**Multi-rate is how you get both**: a coarse coupling cadence for cheap export to
neighbours, with each flow sub-stepping internally at the `dt` its own rate constant
demands.

## Why NOT the two alternatives (both were considered and priced)

* **Implicit / A-stable integrator (backward Euler).** The instinctive "rework the maths".
  It is the largest unfreeze in the repo (all four contracts, Newton iteration for the
  nonlinear biosphere flows, **every golden moves**) — and **it does not meet the bar**.
  Measured at `dt = 1800`: backward Euler gives **10.61 mol** against a truth of **8.33**
  (27 % off). It buys *sane* (never 0, never negative, monotone), not *near*. Largest price
  in the project for the smaller half of the goal.
* **Rework the flow kinetics / clamp `makeup_flux`.** Fixing a solver property in the
  physics layer. The continuous law `dx/dt = k(S−x)` is unconditionally stable and cannot
  oscillate; there is nothing wrong with it to fix. A clamp additionally destroys the
  attractor (`o2_eq` is an attractor from both sides *because* the controller is linear).

## What this DOES resolve

1. **The composability constraint the reference calls unsolvable.**
   `docs/authoring-reference.md`: *"ECLSS is sized for `dt = 60`; Thermal for `dt = 3600`. A
   scenario composing both must pick one `dt`, and only `dt ≤ ~60` is safe for both. **There
   is no `dt` natural to both domains**."* With multi-rate there is: master `dt = 3600`,
   Thermal slow (1 step), ECLSS fast (`n_sub = 60` → 60 s sub-steps). Today Thermal is
   forced to pay 60× the steps it needs, purely to keep ECLSS safe.
2. **Export fidelity at the coupling boundary** — the charge above.
3. **Performance, measured, not assumed.** Cost is purely proportional to step count
   (µs/step is flat at ~60–96 µs across a 3600× `dt` span; build excluded, 10 simulated
   days). `dt = 1` costs **83 s**; `dt = 3600` costs **0.017 s** — **4815×**. So paying the
   fast `dt` on the slow domain is a real, large, avoidable cost. That is exactly what
   `simcore.multirate` exists to avoid: *"without paying the fast `dt` on the slow domain."*

## What this does NOT resolve — read before treating it as the hazard fix

**Multi-rate is the performance enabler, not the hazard closer** (advisor, this session).
`multirate_step` splits one master `dt` into `dt/n_sub`. An author can still choose an
`n_sub` whose **effective sub-step** is unsafe — the identical hazard, one level down. The
direct closer remains the deferred **build-time precondition**, and multi-rate *changes what
it must check*: the **effective sub-step `dt/n_sub`**, never the master `dt`. The two are
complementary; shipping multi-rate does not retire the guard, it complicates it.

**It costs accuracy versus single-rate RK4, and the honest framing matters.** From
`simcore/multirate.py`'s own contract: Lie is globally 1st-order; Strang is 2nd-order *only
if both operators are RK4*, and **"a Euler operator silently collapses Strang back to 1st
order"**. Our frozen flows are Euler. The operators' non-commutativity is an O(`dt²`) term
no sub-integration removes, so **multi-rate is an efficiency trade that costs accuracy
versus single-rate RK4 (order 4)**. It is worth it here only because the alternative at a
coarse `dt` is not a *less accurate* value but a *meaningless* one (0 mol, 18 mol).

**It is not a licence to raise `dt`.** `k·dt < 1` stays operative on the **effective**
sub-step, for the export-fidelity reason (see the reference's "the hazard rationing cannot
see").

## The design question owed to the advisor — the author-facing knob

The partition is **per-flow, not per-domain** (decision N3): *"A cross-domain flow has no
single-domain rate, so the rate-class is a property of the flow, assigned by the scenario
assembler; the driver takes the two pre-built integrators and does not infer the
partition."* So `simcore` **refuses to guess**, and the authoring layer must ask. Candidate
shapes, none chosen:

* **(a) Per-flow `rate: fast | slow`** + a top-level `n_sub`. Closest to the core's own
  model; most explicit; verbose.
* **(b) Top-level `fast: [flow-id, …]` + `n_sub`.** One place to look; the partition reads
  as one decision rather than N scattered ones.
* **(c) Derive it** from each flow type's registered rate constant + the master `dt`. Most
  convenient, **most dangerous**: it re-introduces the inference N3 deliberately refused,
  and it cannot see authored `kinetics` flows (whose rate law the author wrote).
* **(d) Per-flow `substeps: N`** — sugar over (a)+`n_sub`, but `multirate_step` takes ONE
  `n_sub` for the whole fast set, so this would over-promise.

Open, for the advisor pass: **(b) vs (a)**; whether `split` (Strang/Lie) is author-visible
or pinned to Strang; and whether `n_sub` is validated against the effective-sub-step bound
at build time (which folds the deferred precondition into this phase rather than after it).

## The golden-preservation argument — why this need not move a single golden

`multirate_step`'s own contract: *"With `n_sub == 1` and an **empty** slow registry, a
Strang master step reproduces the single-rate `step` **bit-for-bit**."*

So the migration has a **bit-exact identity path**: default `n_sub = 1`, everything fast,
slow empty ⇒ byte-identical to today. Every existing authored scenario, every golden, and
both cross-port tiers hold **by construction, not by re-baselining**. A scenario opts into
multi-rate only by declaring a partition.

**And unlike the reversal prose, this claim is already measured** —
`tests/test_multirate.py::test_all_fast_nsub1_reproduces_single_rate_bitwise`, parametrised
over **both** Euler and RK4, green. (Checked rather than assumed: the first draft of this
plan asserted it was an unmeasured docstring claim, which was the very error this session
exists to correct. Check the premise.)

What is **not** yet pinned is the identity at the **authoring** level: the simcore test
proves it on a synthetic registry, not that `authoring.run` driving `multirate_step` at
`n_sub = 1` reproduces `eclss_state.json`. That is a narrow, real gap and it is Step 1 —
but it is an *extension* of an existing proof, not a load-bearing unknown.

## Scope — every surface this touches

| surface | change | contract |
|---|---|---|
| `authoring/schema.py` | the partition + `n_sub` knob | **authoring unfreeze** |
| `authoring/interpreter.py` | build two disjoint registries over one stock dict | **authoring unfreeze** |
| `authoring/run.py` | drive `multirate_step`; sum `rationed` across sub-ops | **authoring unfreeze** |
| `authoring/flow_registry.py` | only if the precondition folds in | **authoring unfreeze** |
| `docs/authoring-reference.manifest.json` | regenerate; the git-visible record | manifest |
| `rust/crates/authoring` | the hand-mirrored port | native-port tolerance contract |
| `tests/crossport/tiers.json` | a multi-rate anchor, if one is added | cross-port tiers |
| `simcore/` | **NONE** — `multirate_step` already exists and is proven | purity invariant |

`git diff src/simcore/` **must come back empty**: this phase is a *consumer* of the
Phase-0.5 driver. If it cannot be built without changing `simcore`, that is a finding to
surface, not a licence to edit the core.

## Steps (provisional — the advisor pass may reshape 2–3)

1. **Extend the identity pin to the authoring path.** `simcore` already proves
   `n_sub = 1` + empty slow ⇒ bit-identical, on both integrators. Carry it up: `authoring.run`
   driving `multirate_step` at `n_sub = 1` must reproduce `eclss_state.json` byte-for-byte
   on the committed ECLSS anchor. Test-first, before any schema change. If this fails, the
   de-risking argument is gone and the phase stops here.
2. **Advisor pass on the knob** ((a)–(d) above), then schema + interpreter.
3. **The run harness** — `multirate_step` per master step; `rationed` summed over
   sub-operations (its contract already aggregates); `RationedError` semantics preserved.
4. **The composability anchor**: an authored Thermal+ECLSS scenario at master `dt = 3600`
   with ECLSS fast (`n_sub = 60`) — the scenario the reference currently calls impossible.
   Conservation + determinism, `rationed == 0`, and the exported `cabin_o2` monotone.
5. **The effective-sub-step precondition** (or an explicit deferral, recorded by name).
6. **Rust mirror**, hand-written, then the cross-port tier.
7. **Regenerate the authoring manifest**; document; unfreeze ceremony per
   `docs/authoring-reference.md`, "The unfreeze discipline".

## The measurements this rests on

All from this session; probes under `M:\claud_projects\temp\o2-makeup-probe\`, findings
pinned in `tests/test_authoring_export_fidelity.py` (12 pins, green).

| claim | measured |
|---|---|
| the coarse export is not "a bit off", it is wrong | `dt=1800` exports **0.0 mol**; truth **8.33** |
| the equilibrium is `dt`-independent | 8.0000000000 across `dt` 1 → 500 |
| forced flows are exactly `dt`-invariant | `metabolic_o2_sink` = 288.0 mol at `dt` 1 and 60 |
| convergence is textbook first-order | err ×2 as `dt` ×2, to three digits |
| the maths is sound, not broken | deadbeat at `k·dt=1` hit exactly: `12 → 10 → 10 → 10` |
| cost is purely per-step | ~60–96 µs/step flat; `dt=1` **83 s** vs `dt=3600` **0.017 s** |
| backward Euler does not meet the bar | **10.61** vs truth **8.33** at `dt=1800` (27 % off) |
| the oscillating band is invisible | `dt=900`: `12 → 8.4 → 11.28 → 8.976`, `rationed = 0` |
