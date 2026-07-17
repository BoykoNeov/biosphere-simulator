# Post-roadmap: multi-rate authoring — the author picks a coupling cadence, not a global `dt`

**Status: DESIGNED, not built.** A **phase, not a step** — an authoring unfreeze
(schema + interpreter + run harness), the Rust mirror, the freeze manifest, and the
cross-port tiers. **The user opened the unfreeze on 2026-07-17** and the design-level
advisor pass on the author-facing knob is **DONE** (below): the knob is decided, the
blocking identity is **measured bit-exact through the authoring layer**, and the payoff is
**measured**. No code is written and no frozen surface has moved. One decision is open —
whether the effective-sub-step precondition folds into this phase.

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
`n_sub` whose **effective sub-step** is unsafe — the identical hazard, one level down
(**measured**: `n_sub=2` at `dt=3600` gives `36.0` against a truth of `8.0`). The direct
closer is the **build-time precondition**, and multi-rate *changes what it must check*: the
**effective sub-step `dt/n_sub`**, never the master `dt`.

**→ So the precondition FOLDS INTO this phase (user decision, 2026-07-17): the whole `k·dt`
family.** See "The precondition" below. Without it this phase would ship a knob that reads
as safety while the hazard is unchanged.

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

## The author-facing knob — DECIDED (design pass, advisor + measured, 2026-07-17)

The partition is **per-flow, not per-domain** (decision N3): *"A cross-domain flow has no
single-domain rate, so the rate-class is a property of the flow, assigned by the scenario
assembler; the driver takes the two pre-built integrators and does not infer the
partition."* So `simcore` **refuses to guess**, and the authoring layer must ask.

### The shape: (a) per-flow `rate_class: fast | slow`, defaulting to `fast`

```yaml
name: eclss_thermal_habitat
integrator: euler
dt: 3600.0          # the COUPLING CADENCE — what neighbours see, not what ECLSS solves at
n_sub: 60           # the fast set sub-steps at dt/n_sub = 60 s

flows:
  - id: eclss.o2_makeup
    type: eclss.o2_makeup
    # rate_class: fast  <- the default; sub-steps at 60 s
  - id: thermal.radiator_reject
    type: thermal.radiator_reject
    rate_class: slow    # tau ~ 65 steps at dt=3600; stepping it 60x is pure waste
```

**Spelled `rate_class`, not `rate` (Step 2, user's call).** The design pass argued *where*
the knob lives and never weighed the spelling — so `rate:` was never actually decided, it
was assumed. A bare `rate:` would sit one nesting level above `kinetics.rate`, the rate
*law*, and a flow may carry both. `rate_class` also matches what the concept is called
everywhere else it appears (`interpreter._RATE_CLASSES`, the manifest's `rate_classes`),
so `rate:` was the lone inconsistency rather than the convention.

**(b) was rejected, and the bundle argument alone is decisive** (advisor). A top-level
`fast: [flow-id, …]` list is a list of **id references**, and `{bundle, prefix}` includes
**rewrite ids** (`<prefix>.<id>`) — so the list becomes a new rewrite surface in
`compose.apply_includes` that silently mis-fires the moment someone forgets it. Under (a)
the rate-class travels **with the flow**, inside the bundle, and prefixing cannot touch it
because it is a *property*, not a reference. (a) therefore has **zero referential-integrity
surface** and simply *is* N3. (c) re-introduces the inference N3 refused and cannot see
authored `kinetics`; (d) over-promises (`multirate_step` takes ONE `n_sub` for the fast set).

**Default `fast` is load-bearing, not a convenience.** `rate` defaulting to `fast` +
`n_sub` defaulting to `1` ⇒ empty slow set ⇒ **the bit-exact identity path**. A scenario
with no multi-rate keys lowers to today's trajectory *by construction*. Default `slow`
could not do this: with an empty fast set, Strang would run two `dt/2` half-steps, which is
**not** one full Euler step.

### `split` (Strang/Lie): NOT author-visible — pinned to Strang

Strang is the core's own default and carries the higher nominal order. **The justification
is order/safety, not performance** (advisor correction — I had it backwards): Lie is
actually *cheaper* on the slow set (1 slow evaluation per master step vs Strang's 2).
Strang additionally steps the slow set at `dt/2`, which is *safer* for the slow set's own
`k·dt`. Lie is documented in `simcore` as "fallback / comparison" — a **study** tool, not an
authoring choice — and our Euler flows collapse Strang to 1st order anyway, so exposing the
knob buys an author no order they can use. **Deferred by name:** author-visible `split`.

### Mixed integrators: NOT exposed

`multirate_step` takes two `Substepper`s and would accept `slow=rk4, fast=euler`. Both are
built from the scenario's single `integrator`. **Deferred by name:** per-rate-class
integrator.

### `n_sub = 1` with a NON-empty slow set: an `AuthoringError`

The advisor surfaced this as an unconsidered case; it is now **measured**, and the first
reading of the probe was wrong twice — worth recording, because the wrong readings are the
intuitive ones:

| `n_sub=1`, slow set | all-stock result |
|---|---|
| `[]` (the identity path) | **BIT-IDENTICAL** |
| `[co2_scrubber]` | DIFFERS — `cabin_co2 −4.57e-02` |
| `[o2_makeup]` | DIFFERS — `cabin_o2 +6.19e-02` |
| `[crew_metabolism]` (FORCED) | DIFFERS — `cabin_o2 +1.20e-01` |

So **every** non-empty slow partition at `n_sub=1` perturbs, via **two** mechanisms:
the slow flow's own two-half-step discretization (`(1−k·dt/2)² ≠ (1−k·dt)`), *and* — the
dominant one — the **coupling**: fast flows now read slow-updated stocks mid-step. The
forced row proves the second is independent of flow shape: `crew_metabolism`'s own legs
split exactly (its residual is roundoff, `−1.25e-12`), yet the cabin moves by `1.2e-01`
because the fast flows see a half-metabolised cabin.

Two hypotheses died here. "It only perturbs *coupled* flows" — false, the scrubber perturbs
its own stocks. "A *forced* flow splits exactly, so it is safe" — false in the only sense
that matters; its own legs do, the trajectory does not. (The first probe pass compared
**only `cabin_o2`** and read "bit-identical" off flows that never touch it — blind evidence
for a claim about "the trajectory". Compare every stock.)

A partition at `n_sub=1` therefore buys **no rate separation and no perf win** while
silently moving the answer: a misconfiguration, refused at build time, not honoured.

### The aux tripwire — future-proofing, not a present fix

`multirate_step` **never advances aux** (P2: *"Aux × multi-rate is out of scope"*), while
`step_report` does — so the identity claim carries an **unstated precondition**: *no aux
processes*. It holds today only because `interpret` calls `Registry(flows, stocks)` and
**never wires `aux_processes`** — the authoring layer cannot express aux at all, and the one
aux-bearing domain (biosphere) is deferred from the registry for exactly this family of
reasons. So the guard can never fire from authored input **today**; it is a tripwire for the
phase that makes the biosphere authorable, where aux would otherwise **silently freeze**.
Raise in the multi-rate run path if `registry.aux_processes` is non-empty, pointing at
simcore's P2 boundary.

### `RationedError`'s message

Says *"Reduce dt and re-run"*. Under multi-rate the honest advice is **"increase `n_sub` or
reduce `dt`"** — the message is part of the fix, since it is what an author reads at the
moment they hit the hazard.

## The precondition — DECIDED: the whole `k·dt` family (user, 2026-07-17)

Offered as minimal (`o2_makeup` alone — my recommendation and the advisor's) vs the whole
family vs defer. **The user chose the whole family**, on the uniformity argument: *"the
platform checks `k·dt` for any flow that declares a rate constant"* is a rule an author can
hold in their head; *"`o2_makeup` is special"* is trivia they will forget. It also moves the
error from **run time to build time** for the other three — the author learns before a long
run, not after.

A new **optional** `rate_params` field on `FlowTypeSpec` names which of a flow type's params
are first-order rate constants. The interpreter checks `k · (dt/n_sub) < 1` for each, at
build time, from the **pack-resolved** params object it already holds — so a param **pack**
that inflates a gain is caught too, which only a build check can do. Transcendental-free
(`+ − × <`) ⇒ the Rust mirror is byte-safe.

| flow type | `rate_params` | today at its frozen `dt` |
|---|---|---|
| `eclss.co2_scrubber` | `co2_scrub_rate` | `1e-3 · 60 = 0.06` |
| `eclss.condenser` | `condense_rate` | `5e-4 · 60 = 0.03` |
| `eclss.o2_makeup` | `o2_makeup_gain` | `2e-3 · 60 = 0.12` — **the one rationing cannot see** |
| `power.self_discharge` | `self_discharge_rate` | `1e-8 · 3600 = 3.6e-5` |

**The field names are the real ones, checked** — `k_scrub`/`k_cond` are only the docs'
shorthand; the dataclass fields are `co2_scrub_rate` / `condense_rate`. `EclssParams` also
carries **`o2_setpoint`, which is NOT a rate** and must never be checked — which is exactly
why `rate_params` is an explicit declaration and not "every float on the params object".

**What it honestly CANNOT cover — document, do not fake:**

* `thermal.radiator_reject` — the constraint is `τ = C/(4εσA·T_eq³) ≫ dt`. **"≫" is not a
  predicate**; making it one means inventing a safety factor the science does not supply.
* `eclss.crew_metabolism` — `forced draw < stock` is **state-dependent**, not param-only. A
  build check sees only the *initial* amount, so it is necessary-not-sufficient. Rationing
  catches it at run time; that stays its guard.
* **authored `kinetics`** — structurally uncheckable: the author wrote the rate law, so the
  platform cannot know its constant. This is exactly decision B's "authored ≠ validated"
  boundary, not a gap to be closed.

So the precondition's honest claim is **"the platform catches the `k·dt` family"**, never
"your `dt` is safe". The **general** precondition stays deferred — a research problem, not a
consumer-phase task.

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

What was **not** pinned is the identity at the **authoring** level: the simcore test proves
it on a synthetic registry, not that `authoring.run` driving `multirate_step` at `n_sub = 1`
reproduces the real authored graph. The advisor ruled this **blocking** — *"if it isn't
byte-exact through the authoring layer, the knob design is moot and the phase stops there"*
— so it was measured **before** any design was committed to:

> `eclss_cabin.yaml` (the Tier-1 ECLSS anchor, 900 steps, 4 flows, 3 quantities), single-rate
> vs `multirate_step` at `n_sub=1` with an empty slow registry, compared by `float.hex()`
> over **every stock of every step**: **BIT-IDENTICAL**, under **both** Strang *and* Lie.
> (`M:\claud_projects\temp\multirate-authoring\identity_probe.py`.)

So the golden-preservation argument is **measured, not inferred**, and Step 1 is now the
narrower job of *promoting the probe to a committed pin* rather than establishing the fact.

## The payoff — measured, and it does NOT close the hazard

The plan's central claim was inference until now: that the master `dt` which wrecks the
cabin is rescued by sub-stepping. Measured on the ECLSS anchor at master `dt = 3600`
(truth = `o2_eq` = 8.0):

| mode | effective `k·dt` | `rationed` | final `cabin_o2` |
|---|---|---|---|
| single-rate `dt=3600` | 7.20 | 60 | **72.000000** (diverged) |
| multi-rate `n_sub=2` | 3.60 | 95 | **36.000000** (still broken) |
| multi-rate `n_sub=10` | 0.72 | 0 | **8.000000** |
| multi-rate `n_sub=60` | 0.12 | 0 | **8.000000000000007** |
| single-rate `dt=60` (reference) | 0.12 | 0 | **8.000000000000007** |

**The bottom two rows are the phase in one line**: master `dt=3600` with `n_sub=60` lands on
the *same* value as `dt=60`, while exporting **hourly**. That is the user's charge answered.

**The `n_sub=2` row is the advisor's point made concrete, and it is the argument for the
precondition**: an author who "switches on multi-rate" and picks `n_sub=2` gets **36.0**
instead of **8.0**. Multi-rate hands out a knob that *looks* like safety; the effective
sub-step is the same hazard one level down.

(The single-rate `dt=3600` row reads **72.0**, not the `≈0` the reference records — no
contradiction: `test_authoring_dt_hazard` samples at **15** steps, this at **24**, and an
oscillating divergence is simply at a different point. Both are the same broken run.)

## Scope — every surface this touches

| surface | change | contract |
|---|---|---|
| `authoring/schema.py` | the partition + `n_sub` knob | **authoring unfreeze** |
| `authoring/interpreter.py` | build two disjoint registries over one stock dict | **authoring unfreeze** |
| `authoring/run.py` | drive `multirate_step`; sum `rationed` across sub-ops | **authoring unfreeze** |
| `authoring/flow_registry.py` | `rate_params` on `FlowTypeSpec` + 4 rows — the precondition **folds in** | **authoring unfreeze** |
| `docs/authoring-reference.manifest.json` | regenerate **at each step that moves the surface** (2: `schema_fields` + `rate_classes`; 5: `flow_types`), never batched to the end — the gate is plain equality and would sit red meanwhile | manifest |
| `rust/crates/authoring` | the hand-mirrored port | native-port tolerance contract |
| `tests/crossport/tiers.json` | a multi-rate anchor, if one is added | cross-port tiers |
| `simcore/` | **NONE** — `multirate_step` already exists and is proven | purity invariant |

`git diff src/simcore/` **must come back empty**: this phase is a *consumer* of the
Phase-0.5 driver. If it cannot be built without changing `simcore`, that is a finding to
surface, not a licence to edit the core.

## Steps (provisional — the advisor pass may reshape 2–3)

1. **Extend the identity pin to the authoring path.** ✅ **MEASURED** (see above:
   bit-identical over every stock of every step, both splits) — this step is now *promoting
   the probe to a committed pin*, not establishing the fact. Test-first, before any schema
   change.
2. **Advisor pass on the knob** ✅ **DONE** — (a) + default `fast`, top-level `n_sub`,
   Strang pinned, `n_sub=1`-with-slow refused, aux tripwire. **Then schema +
   interpreter** ✅ **DONE** — see "Step 2: COMPLETE" below.
3. **The run harness** — `multirate_step` per master step; `rationed` summed over
   sub-operations (its contract already aggregates); `RationedError` semantics preserved.
4. **The composability anchor**: an authored Thermal+ECLSS scenario at master `dt = 3600`
   with ECLSS fast (`n_sub = 60`) — the scenario the reference currently calls impossible.
   Conservation + determinism, `rationed == 0`, and the exported `cabin_o2` monotone.
5. **The effective-sub-step precondition** — **folded in** (user, the whole `k·dt` family):
   `rate_params` on `FlowTypeSpec`, checked as `k·(dt/n_sub) < 1` at build time. The three
   uncoverable shapes documented by name, not faked.
6. **Rust mirror**, hand-written, then the cross-port tier.
7. **The consolidated unfreeze ceremony** + the reference-doc narrative, per
   `docs/authoring-reference.md`, "The unfreeze discipline". **The manifest itself moves
   *incrementally*, not here** (advisor, Step 2): `test_frozen_schema_surface_is_complete`
   is plain equality against the live tree, so it goes red the *instant* a schema field
   lands — and this project commits each step to `main`. Deferring the regeneration to
   Step 7 would mean a red gate across Steps 3–6, which does not "save" the ceremony, it
   just **disables the gate** for the four steps most likely to drift. So each step
   regenerates the surface it moves (Step 2: `schema_fields` + `rate_classes`; Step 5:
   `flow_types`/`rate_params`), and Step 7 owns the *narrative*: the reference doc, the
   deferrals-by-name, and the consolidated record of what was unfrozen and why.

## Step 2: COMPLETE — the knob exists, and it is inert until asked for

**Nothing runs multi-rate yet.** `interpret` now *builds* the partition; no harness
consumes it (Step 3). `git diff src/simcore/` is **empty**, as the purity invariant
demands — this is a consumer phase.

**The surface, as decided:** `ScenarioSpec.n_sub: int = Field(default=1, ge=1)` and
`FlowSpec.rate: str = "fast"`. `BuiltScenario` gains `slow_registry` / `fast_registry`
(the disjoint N3 partition over the one shared stock dict), `n_sub`, and an
`is_multirate` property.

**Three registries, not two — and the third is the safety.** `registry` (the whole flow
set) is kept alongside the partition. That is what lets Step 3's harness run every
*existing* scenario down **today's single-rate code path verbatim**, instead of routing it
through `multirate_step` and *relying* on the `n_sub=1` identity. The identity is measured
(Step 1) — but "measured" and "load-bearing for all 25 goldens" are different risk
postures, and there is no reason to take the second when the first is free.
`is_multirate` is written in the robust form (`n_sub > 1 or slow non-empty`) even though
the refusal makes it equivalent to `n_sub > 1`: the equivalence is a *consequence of the
refusal*, not a property of multi-rate.

**The validation lives in the interpreter, not in a pydantic `model_validator` — and this
is not a style choice.** A `model_validator` on `ScenarioSpec` runs at *schema* time, and
`apply_includes` merges bundles at the *top of* `interpret`. A **bundle** may contribute
`rate: slow` flows, which a schema-level check would never see — it would lower a
partitioned scenario as single-rate, silently. Pinned both ways
(`test_a_bundle_contributed_slow_flow_is_seen` + the refusal reached *through* an
include). `n_sub`'s `ge=1` **is** schema-level, correctly: it is a shape constraint (cf.
`IncludeSpec.prefix`'s `min_length=1`), not a graph-level judgement.

**`n_sub > 1` with an EMPTY slow set is LEGAL, and this row nearly got refused.** It looks
like a misconfiguration ("multi-rate with nothing slow"), which is exactly the intuition
that would have deleted the phase's headline: it is *uniform sub-stepping* — the export
cadence decoupled from the solver step — and it is the configuration the measured payoff
runs on (Step 1's `_run_multirate(_at_unsafe_dt(), 60)` passes `slow_ids=()`). The four-row
legality matrix is pinned in `tests/test_authoring_multirate_partition.py`.

**The manifest moved, deliberately** (the unfreeze, git-visible): `schema_fields` gains
`FlowSpec.rate` + `ScenarioSpec.n_sub`, and a new **`rate_classes`** key records the legal
values — the gap `integrator_names` already fills for `integrator`, since `schema_fields`
records that `rate` *exists* but not what it may *say*. Derived from a live
`interpreter._RATE_CLASSES`, never transcribed. Unlike the flow-type registry (expected to
grow), this vocabulary is **closed at two by `multirate_step`'s own signature** — it takes
exactly two `Substepper`s, so a third class cannot appear without a `simcore` change.

**The key is `rate_class`, not `rate` — a naming hazard caught *before* the freeze.** The
first cut used `rate:`, following this plan's own YAML example, and flagged the collision
with `KineticsSpec.rate` (the rate *law*, one nesting level down on the same flow) three
separate times while doing so. The advisor named that for what it was: the design pass had
argued *where* the knob lives — per-flow property vs top-level id list, on the
referential-integrity argument — and **never weighed the spelling**, so `rate:` was
assumed, not decided, and three flags is not a resolution. The deciding argument turned out
not to be the collision at all but an inconsistency introduced by the work itself: the
concept was already named "rate class" in `interpreter._RATE_CLASSES` and in the manifest's
own `rate_classes` key, so the YAML was the *only* place spelling it `rate` — a manifest
reader would have seen `rate_classes: [fast, slow]` and had to infer it governed a field
called `rate`. Renamed while it was one unpushed commit; after the push it would have been
a full authoring unfreeze. The design pass's actual argument (a per-flow **property**, not
an id **reference**) is untouched by the spelling.

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
