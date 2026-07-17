# Post-roadmap: multi-rate authoring — the author picks a coupling cadence, not a global `dt`

**Status: IN PROGRESS — Steps 1–5 of 7 done.** A **phase, not a step** — an authoring
unfreeze (schema + interpreter + run harness), the Rust mirror, the freeze manifest, and
the cross-port tiers. **The user opened the unfreeze on 2026-07-17.** The knob is
decided, built (Step 2), **drives** (Step 3), has **paid off the phase's stated
motivation** (Step 4 — the scenario `docs/authoring-reference.md` calls *impossible* is
authored, committed and green), and **the hazard it does not itself close is now closed**
(Step 5: the build-time `k·h < 1` precondition). No golden has moved and `src/simcore/` is
untouched. Remaining: the Rust mirror (6) and the unfreeze ceremony (7).

**Step 5 corrected a formula this document specified.** The precondition is **not**
`k·(dt/n_sub) < 1` for every flow: the slow set steps at **`dt/2`** under Strang,
independent of `n_sub`, so that formula **false-PASSES** an unsafe slow flow (measured:
reports 0.06 where the truth is 1.8). It is the *same* Strang fact that made Step 4's
predicted 60× a measured 30×. The wrong formula is left standing where it was written and
corrected at the point of use — the Step-4 precedent.

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
**effective sub-step**, never the master `dt`.

**→ So the precondition FOLDS INTO this phase (user decision, 2026-07-17): the whole `k·dt`
family.** See "The precondition" below. Without it this phase would ship a knob that reads
as safety while the hazard is unchanged.

**✅ CLOSED in Step 5** — `n_sub=2` at `dt=3600` no longer builds. Read the paragraph above
with one correction it did not know: *"the effective sub-step `dt/n_sub`"* is right for the
**fast** set only. The slow set's effective sub-step is **`dt/2`**, and writing `dt/n_sub`
there is exactly the false-PASS Step 5 measured.

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
are first-order rate constants. The interpreter checks `k · h < 1` for each, at build time,
from the **pack-resolved** params object it already holds — so a param **pack** that
inflates a gain is caught too, which only a build check can do. Transcendental-free
(`+ − × <`) ⇒ the Rust mirror is byte-safe.

> **CORRECTED IN STEP 5.** This paragraph originally read `k · (dt/n_sub) < 1`. `h` is the
> **per-rate-class effective step** — `dt` single-rate, `dt/n_sub` fast, **`dt/2` slow** —
> and `dt/n_sub` for a slow flow is a **false PASS in the unsafe direction** (measured:
> 0.06 reported, 1.8 actual, 24 rationings, `cabin_co2 → 0.0`). See "Step 5: COMPLETE".

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
| `authoring/flow_registry.py` | ✅ `rate_params` on `FlowTypeSpec` + 4 rows — the precondition **folds in** | **authoring unfreeze** |
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
3. **The run harness** ✅ **DONE** — see "Step 3: COMPLETE" below.
4. **The composability anchor** ✅ **DONE** — see "Step 4: COMPLETE" below.
5. **The effective-sub-step precondition** ✅ **DONE** — see "Step 5: COMPLETE" below.
   (As written this step said "checked as `k·(dt/n_sub) < 1`". **That formula is wrong for
   the slow set and the step corrected it** — left standing here rather than edited away,
   the Step-4 `60×`/`30×` precedent.)
6. **Rust mirror**, hand-written, then the cross-port tier.

   **⚠ Step 6 carries an OPEN DECISION it must not inherit silently** (advisor, after
   Step 5). The precondition lives in `interpret` — which is **upstream of the deliberate
   library-vs-interactive split**. `godot_bridge::build_session_from_file` builds through
   `authoring::load_scenario` (`rust/crates/godot_bridge/src/lib.rs:190`), so **mirroring
   the precondition makes the interactive path refuse an unsafe scenario at build**, and
   the player never sees the cabin die. That silently narrows a *designed* asymmetry:
   `docs/authoring-reference.md` says this path "does **not** raise — deliberately"
   (*"Library caller → exception; interactive session → visible diagnostic + objective
   failure"*), which was scoped to **rationing**, the only gate that existed when it was
   written. **No tension today** — the Rust interpreter is unmirrored, so the shipped
   bridge is unchanged; this is a Step-6 landmine, not a Step-5 bug.

   The call is genuinely open and the precedent cuts both ways:
   * **For refusing**: `build_session_from_file` *already* rejects rk4 files at build
     ("single-rate Euler only"), so build-time refusal is not foreign to the session path,
     and an unsafe `k·h` is likewise decidable from the file — the same class as rk4.
   * **For the hatch**: *"a player should watch the cabin die"* **is** a
     study-the-unsafe-run case, which is precisely what `allow_unsafe_step=True` exists
     for. Passing it in the bridge is the reading consistent with the hatch's own purpose,
     and preserves the documented split intact.

   Either is defensible; **inheriting one by accident is not**. Marked in the reference
   doc in place — the **third** instance of this phase's recurring pattern (a doc claim
   whose scope silently narrowed), after Step 4's "no `dt` natural to both domains" and
   Step 5's "still deferred" precondition paragraph. That it is the third is the finding:
   *this phase keeps invalidating sentences written before the platform could do the thing
   they deny*, and the reference doc is where they accumulate. Step 7 owns the rewrite.
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

## Step 3: COMPLETE — the knob drives, and the goldens are safe by construction

**The phase's headline is now reachable from the surface an author calls.** Master
`dt=3600` + `n_sub=60` through `run_scenario` lands on `cabin_o2 == 8.0` with
`rationed == 0`, at 24 master commits — the export-fidelity charge answered end-to-end.
`git diff src/simcore/` is **empty**; the freeze manifest did **not** move (Step 3 adds
no schema field, integrator name, or flow type — the surface it touches is `run.py`,
which the manifest does not name); 1832 tests green, no golden moved.

**The branch, not the identity, is what preserves the goldens — and Step 3 is where that
became true.** `run_scenario` routes on `is_multirate`: a declared cadence goes to
`multirate_step`, **everything else takes the pre-multi-rate loop verbatim**. Step 1's
identity means routing *everything* through the driver would also work — which is exactly
why the branch needed a pin of its own. Had it leaked, **every golden would have stayed
green** while all 25 silently came to rest on the `n_sub=1` identity holding forever;
the leak would surface only years later, as a future `simcore` driver change moving 25
files at once with no cause attached. `test_a_single_rate_scenario_never_touches_the_driver`
monkeypatches `multirate_step` to raise and runs a single-rate scenario through the
harness. Keeping the third registry (Step 2) only buys the safety if something asserts
the harness actually uses it.

**A Step-1 instruction was superseded, and the sequence is the finding** (cf. the
`rate_class` naming catch, same flavour). `test_authoring_multirate_identity.py`'s header
told Step 3 to *"re-point the byte-identity pin through `run_scenario`"* — and the
advisor independently repeated it as scoped-but-unnamed work. **Both were wrong, for a
reason that did not exist when the note was written**: Step 2's branch makes
`is_multirate` *false* at `n_sub=1`, so a re-pointed test would drive the **single-rate
path**. Not a weaker test of the driver — a test of the wrong path, and one that (sharing
its golden oracle with `test_authoring_frozen_flows.py`) would collapse into a duplicate
of that file while losing the driver-faithfulness check entirely. The driver at `n_sub=1`
is reachable **only** by calling it directly. The note's *spirit* — exercise the driver
through the harness — is satisfied at `n_sub=60`, where the driver actually runs.
**The identity is thereby demoted from load-bearing to corroborating**, which is a
promotion for the phase: golden preservation moved from *measured* to *by construction*.

**The aux tripwire lives in `run.py`, multi-rate branch only.** `step_report` advances
aux; `multirate_step` never does (P2) — so routing an aux-bearing graph through the
driver freezes every accumulator **silently**, and the conservation gate structurally
cannot see it (aux is non-conserved by definition). Three rulings: it is **not** in
`multirate_step`, because `simcore` is frozen and this is a consumer phase; it is **not**
on the single-rate path, because `step_report` handles aux correctly and refusing it
there would ban a working shape; and it is an `AuthoringError` despite firing at run
time, because it is decidable from the graph's structure alone and is raised before any
step runs. It also **cannot be reached from any authored file** — `interpret` never wires
`aux_processes` — which is a second, independent argument for `run.py`: a hand-built
`BuiltScenario` can reach it there, so the guard is *testable*. In `interpret` it would
have been unreachable and untestable both.

**`RationedError`'s message is conditional, and that was a real catch.** *"Increase
`n_sub` or reduce `dt`"* is honest on the multi-rate path and **wrong** on the
single-rate one — there is no `n_sub` to raise, and naming it sends an author hunting for
a key their scenario does not have. The multi-rate variant also reports the **effective
sub-step** `dt/n_sub` rather than the master `dt`, which is no longer the step any flow
was integrated at: quoting it would name the one number that is *not* the cause. It
further warns that **the slow set steps at `dt/2` regardless of `n_sub`** under Strang, so
raising `n_sub` will not rescue a slow flow's over-draw — re-class it fast, or reduce `dt`.

**Still not the hazard closer, and the harness says so.** `n_sub=2` at `dt=3600` raises
`RationedError` through the harness — but that is **luck of shape, not the gate working**:
what the backstop sees is the *donor-controlled* scrubber's over-draw. `o2_makeup` is
demand-controlled and its near-setpoint oscillation stays invisible to rationing at any
`dt` (`test_authoring_export_fidelity.py`). Step 5's build-time `k·(dt/n_sub) < 1` check
remains the direct closer; this run-time catch is not a substitute.

## Step 4: COMPLETE — the impossible scenario is authored, and it cost Thermal 30× less

**The sentence is falsified.** `docs/authoring-reference.md` says: *"ECLSS is sized for
`dt = 60`; Thermal for `dt = 3600`. A scenario composing both must pick one `dt`, and only
`dt ≤ ~60` is safe for both. **There is no `dt` natural to both domains.**"*
`tests/authoring/scenarios/eclss_thermal_habitat.yaml` is that scenario: master `dt=3600`,
ECLSS fast at `n_sub=60`, Thermal slow. `rationed == 0`, the cabin holds `o2_eq` and the
node warms 102.70 K → 277.44 K against `T_eq ≈ 280.9`, both monotone, bit-identical across
runs. 13 pins in `tests/test_authoring_multirate_composability.py`. `git diff src/` empty;
**the manifest did not move** (Step 4 adds no schema field, integrator name, or flow type).

**The constraint had two halves, and pinning only one would have been the easy mistake.**
Either alone is unconvincing — so both are measured, on the *same graph*:

| the graph | | |
|---|---|---|
| single-rate at the master `dt=3600` | **`RationedError`**, 840 firings | the shared `dt` is **unsafe** |
| single-rate at `dt=60` (the reference's own escape) | clean, `rationed == 0` — and **20160** Thermal evals | the safe shared `dt` is **wasteful** |
| **multi-rate, master `dt=3600`, `n_sub=60`** | clean, **672** Thermal evals | escapes **both** |

The unsafe row is worth reading past the exception: with `allow_rationing=True` it ends at
`cabin_o2 = 72.0` against a truth of `8.0` — the regulator **diverged**, it did not drift.
**And the direction matters, because the intuitive word for it is wrong**: 72.0 is *nine
times too much* oxygen, not too little. The hazard is not "the cabin suffocates", it is
"the number is meaningless" — `k_makeup·dt = 7.2` makes the update map
`o2 → −6.2·o2 + 57.6`, which alternates and grows, so the *sign* of the error is an
accident of where the oscillation is sampled. The reference's `−1.4e-14` (15 steps) and
this `72.0` (336 steps) are the **same broken map at different phases**. An author who
learns "asphyxiation" will accept a run that happens to land high; pinned as
`amount > O2_EQ` alongside the value, so the direction cannot quietly drop out.
A free cross-check fell out: 840 firings / 336 steps = **2.5 per step**, exactly the rate
the Step-3 table measured on the *bare* ECLSS anchor (60 / 24). Adding the Thermal half
changed neither the firing rate nor the endpoint — itself evidence the two rate classes do
not interact here.

**The payoff is 30×, NOT the 60× this plan predicted — and the missing factor of two is
Strang's bill, not an error.** The advisor caught this *before* the measurement, which is
the only reason the number was checked rather than parroted: this plan says above that
*"Thermal is forced to pay 60× the steps it needs"*, and 60 is the cadence ratio
(`3600/60`). But **Strang steps the slow set at `dt/2`, twice per master step** — so
Thermal's realized evals are `336 × 2 = 672`, and `20160 / 672 = 30.0` exactly. Lie would
realize the full 60× (one slow evaluation per master step) at a lower nominal order and a
coarser slow-set step; the split was pinned to Strang on order/safety grounds in Step 2,
and 30× rather than 60× is what that decision costs. **The 60× claim above is left standing
as written and corrected here rather than edited away** — the cadence ratio *is* 60; what
is 30 is the saving.

**And the honest whole-run number is smaller still: wall clock improves 2.31×, not 30×.**
Multi-rate saves the **slow domain's** work, and in this anchor the slow domain is the cheap
one — ECLSS's 20160 fast sub-steps still happen and dominate (measured: 1.31 s vs 3.02 s).
The 30× is a real, exact, integer fact about Thermal's evaluations; it is not a claim about
the run. The wall win would be large where the *slow* set is the expensive one — which is
precisely the biosphere (17 flows + aux), and precisely the domain multi-rate cannot reach
yet. Not asserted in a test: an eval count is deterministic, a wall clock is a flake.

**What the anchor does NOT prove, and the reason is structural.** The two domains share no
stock — they share no *quantity* (oxygen/carbon/water vs energy; four independent
conservation books). So the Strang operators commute **exactly**, the splitting error is
zero, and **no coupling fidelity is exercised**. This is **forced by the registry, not
chosen**: no ECLSS flow type carries a heat leg, so there is no ECLSS→Thermal flow to
write. In the frozen registry the cross-rate-class boundary and the cross-stock boundary
**never overlap** — coupling lives *within* a domain (the shared cabin) or across
*same-timescale* domains (Power↔Thermal, both slow). Manufacturing a coupled cross-rate
case would mean declaring Power "fast" against its `1e-8` self-discharge rate: an
artificial scenario asserting an artificial fact. It is not a gap, because coupling
fidelity is pinned twice already and neither pin needs this file —
`test_authoring_multirate_partition.py` (a slow flow sharing stocks with fast flows *does*
perturb them: `+1.2e-01` on the cabin, and a *forced* one at that) and simcore's own
`test_multirate.py`. The disjointness is an **assertion**
(`test_the_two_rate_classes_share_no_stock`), not a comment: if a future registry addition
couples the domains, it goes red — the correct moment to re-read this paragraph, because
the anchor would then be proving strictly more than it claims.

**What IS new: the first non-empty slow set ever driven through `run_scenario`.** Every
Step-3 multi-rate run declared `n_sub` with an **empty** slow set (uniform sub-stepping),
so `_run_multirate`'s slow sub-integrator had never held a flow from any authored file.
Step 4 gives it two — and `rationed == 0` for the slow set is measured rather than reasoned
precisely because Thermal's `T⁴` law has **no `k·dt < 1` guarantee** to lean on.

**A Step-2 ruling reached further than Step 2 knew: "the same graph, single-rate" is not
`n_sub=1`.** Building the contrasts hit `AuthoringError` at *build* time — the interpreter
**refuses** `n_sub=1` with a non-empty slow set (it buys no rate separation and still moves
the answer). So going single-rate means **dropping the `rate_class: slow` keys too**, not
just the cadence. Both contrast rows above rest on that, so it is pinned
(`test_going_single_rate_means_dropping_the_partition_not_just_n_sub`) rather than left as
a helper detail. The refusal's own message already said so — *"drop the 'rate_class: slow'
key(s) to run single-rate"* — which is the Step-2 message doing exactly the job it was
written for, on the first author who needed it.

**The cheap Thermal run is still right, and the two ways it agrees are different claims.**
Against the expensive `dt=60` run: `cabin_o2` is **bit-identical** (`float.hex()`), the node
agrees to **0.014 %** (`0.04 K`). The cabin is exact because the fast set is integrated at
60 s either way and the operators are disjoint — multi-rate *reproduces* the ECLSS
trajectory rather than approximating it. The node differs because it is genuinely stepped
coarser (1800 s vs 60 s); that residual is Euler's discretization error on `T⁴` across a
30× step, and its smallness is not luck — it is `τ ≫ dt` doing its job, which is why
Thermal never needed the fine step.

**Tier-2, and left for Step 6 by precedent.** `thermal.radiator_reject` evaluates `T**4`
(Rust: `powf(4.0)`), so this file is **not** bit-exact cross-port. `thermal_node.yaml`'s
precedent applies when the cross-port question is taken up: exclude from the bit-exact run
parametrization, cover by the **graph dump** (which never calls `evaluate()`). Recorded
here, not solved here.

## Step 5: COMPLETE — the hazard is closed, and the plan's own formula was the bug

**The direct closer landed.** `interpret` now refuses a scenario whose step is too large
for a declared first-order rate: `k·h < 1`, checked at build time from the pack-resolved
params. `n_sub=2` at `dt=3600` — the case that made *"multi-rate is the performance
enabler, NOT the hazard closer"* true — **no longer builds**. `git diff src/simcore/` is
empty; **no golden moved**; the manifest moved deliberately (`flow_types` gains
`rate_params`). `tests/test_authoring_rate_precondition.py` (16 pins) + 22 migrated across
six files.

**THE FINDING: this plan specified the check, and the formula it specified is measurably
wrong.** The scope section above says *"The interpreter checks `k · (dt/n_sub) < 1` for
each"* — one formula for every flow. There are **three** cases, and the slow one differs:

| case | effective step `h` |
|---|---|
| single-rate | `dt` |
| multi-rate, **fast** | `dt/n_sub` |
| multi-rate, **slow** | **`dt/2`** — Strang's half-step, *independent of `n_sub`* |

Measured, before any code was written: `eclss.co2_scrubber` classed **slow** at master
`dt=3600`, `n_sub=60` — the plan's formula reports `k·h = 1e-3 · 60 = 0.06` and **PASSES**;
the flow truly steps at 1800 s (`k·h = 1.8`), rations **24** times over 24 steps, and
empties `cabin_co2` to **exactly 0.0**. **A false PASS in the unsafe direction is worse
than no check, because it reads as a guarantee.**

**And it is the SAME Strang fact that turned Step 4's predicted 60× into a measured 30×.**
That is the finding behind the finding: one blind spot — *reasoning about `n_sub` as though
it governed both rate classes* — has now produced two wrong claims in this phase, a
performance number and a **safety** predicate. Step 4 caught its instance and did not
generalize it; the formula in this very document was written after that catch and still
carried the error. `interpreter._effective_step` is now the one place the three cases are
named, and `test_the_effective_step_is_per_rate_class_not_dt_over_n_sub` +
`test_a_slow_flow_is_judged_at_dt_over_2_not_the_plans_formula` pin it. Mutation-checked:
reverting `_effective_step` to the plan's formula turns **5 tests red across 2 files**.

**The `dt/2` divisor is coupled to `_SPLIT`, and the coupling is pinned, not commented**
(advisor). `dt/2` is true only because the harness pins Strang; **Lie steps the slow set at
the full `dt`**, which would make the divisor too permissive by exactly 2× — silently, in
the unsafe direction. `interpreter` cannot import `run._SPLIT` (`run` imports `interpreter`),
so `test_the_slow_step_tracks_the_split_actually_used` asserts `run._SPLIT is Split.STRANG`
and names its own remedy. Exposing author-visible `split` now goes red *here*.

**The behavior change, stated plainly: the `k·dt` family moves from run-time
`RationedError` to build-time `AuthoringError`.** That is intended ("moves the error to
build time" — the user's own reason for choosing the whole family), and it is **observable**:
22 committed pins across six files asserted the run-time verdict and had to migrate. The
migration is itself the evidence the closer works — *those tests could no longer construct
their own subject*.

**`allow_unsafe_step=True` on `interpret`/`load_scenario` is the escape hatch**, the
`run_scenario(allow_rationing=True)` idiom, for **studying** an unsafe run — never for
making a scenario work. `test_authoring_export_fidelity.py` is the case that proves it must
exist: that file's whole subject is the oscillating band the bound excludes, so post-Step-5
it cannot build its own scenarios without saying so explicitly. **Both hatches are needed
and neither implies the other** — `allow_unsafe_step` opens the *build*, `allow_rationing`
opens the *run*. Two gates at two stages; the verbosity is the feature.

**Build time is the locus for the PACK, not for the convenience** (advisor). The obvious
argument — an author learns before a long run — is true and *secondary*. The load-bearing
one: a **param pack may inflate a gain**, and a pack's values exist only *after* `interpret`
resolves them, so `run_scenario` (which receives an already-built flow) **structurally
cannot see it**. `packs/eclss_hot_makeup.yaml` measures it: the committed anchor at its own
frozen, correct `dt=60`, made unsafe purely by a pack that passes **every** frozen guard
(unit exact-string ✓, bound `> 0` ✓ — a gain has no `dt`-independent upper bound, which is
precisely why this can never be a loader bound). `k·dt` goes `0.12 → 1.2`, and because
`o2_makeup` is demand-controlled, **without the build check this exports an oscillating
cabin with `rationed == 0` and no gate anywhere reporting a problem.**

**A manifest hole nearly shipped, and the equality gate could not have caught it**
(advisor). Adding `rate_params` to `FlowTypeSpec` is *not enough*:
`test_frozen_flow_type_registry_is_complete` compares the manifest against `_flow_types()`,
so had that derivation omitted the field, **both sides would omit it, the gate would stay
green, and `rate_params` would never be frozen at all** — a field governing which scenarios
the platform *refuses to build*, unfrozen and unnoticed. An equality gate is blind to a
field absent from both things it equates. This is the scope-C *"a provenance-only edit is an
unfreeze that NOTHING CATCHES"* shape, one level up. Fixed atomically (field + derivation +
regenerate) and given teeth from outside the derivation:
`test_the_manifest_actually_records_the_rate_params`.

**The honest scope is unchanged and now executable**: *"the platform catches the `k·dt`
family"*, **never** *"your dt is safe"*. The three uncoverable shapes are declared
`rate_params=()` — a ruling, not an oversight — and pinned:
`thermal.radiator_reject` (`τ ≫ dt`; **"≫" is not a predicate**, and inventing a safety
factor the science does not supply would be a *fabricated* guarantee), `eclss.crew_metabolism`
(`forced draw < stock` is **state-dependent**; a build check sees only the initial amount),
and **authored `kinetics`** (the author wrote the law — decision B's "authored ≠ validated"
boundary; `rate_params` lives on `FlowTypeSpec`, which a kinetics flow does not have, so the
check skips it *by construction* rather than by a special case). **Neither gate subsumes the
other**, which is why the run-time one was not removed: the build check sees the pack and the
demand-controlled flow; rationing sees the state-dependent over-draw.

**A test's own example turned out to be unsafe** — `test_a_true_partition_at_n_sub_gt_1_builds`
partitioned the **scrubber** slow at `dt=3600`. It now uses the **condenser**, which is the
*only* ECLSS flow that can legally be slow at that cadence (`k_cond = 5e-4`, half the
scrubber's ⇒ `5e-4 · 1800 = 0.9 < 1`). Same graph, same `dt`, same `n_sub` — only `k`
differs, and it decides membership of the slow set.

**Every committed scenario still builds on the author's default path** (no hatch), pinned
by `test_the_committed_scenarios_all_pass_the_precondition` — the assertion behind "no
golden moved", since the risk a *refusal* carries is refusing something that already worked.

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

Step 4 adds (probes under `M:\claud_projects\temp\multirate-step4\`, pinned in
`tests/test_authoring_multirate_composability.py`):

| claim | measured |
|---|---|
| the "impossible" composition runs | master `dt=3600` + `n_sub=60`, Thermal slow: `rationed = 0` |
| the shared `dt` is unsafe | single-rate `dt=3600`: **840** firings, `cabin_o2` = **72.0** vs truth 8.0 |
| the safe shared `dt` is wasteful | single-rate `dt=60`: clean, and **20160** Thermal evals |
| Thermal's saving is 30×, not 60× | `20160 / (336×2) = 30.0` exactly — Strang's `dt/2` slow set |
| the whole-run saving is far smaller | wall **1.31 s** vs **3.02 s** = 2.31× (the fast set dominates) |
| the cheap Thermal run is still right | `cabin_o2` **bit-identical**; node within **0.014 %** (0.04 K) |
| the firing rate is unchanged by composing | 840/336 = **2.5/step** — the Step-3 bare anchor's 60/24 |

Step 5 adds (probes under `M:\claud_projects\temp\multirate-step5\`, pinned in
`tests/test_authoring_rate_precondition.py`):

| claim | measured |
|---|---|
| **this plan's own formula false-PASSES a slow flow** | scrubber slow, `dt=3600`, `n_sub=60`: formula says `k·h = 0.06` ✓; truth is **1.8** — 24 rationings, `cabin_co2` → **0.0** |
| the slow set's step ignores `n_sub` entirely | `dt/2 = 1800` at `n_sub` = 2, 60, 600 **and** 1000 |
| the same flow at the same `dt` is fine as **fast** | `1.8 → 0.06`; only `rate_class` moved |
| a **pack** defeats every pre-Step-5 guard | `eclss_hot_makeup.yaml` loads clean (unit ✓, bound `> 0` ✓) and takes the anchor's own `dt=60` from `k·dt = 0.12` to **1.2** |
| the pack hazard is invisible to `run_scenario` | `o2_makeup` is demand-controlled ⇒ `rationed == 0` at any `dt` |
| the bound is `< 1`, not `≤ 1` | `dt=500` (`k·h` **exactly 1.0**, the deadbeat case) is **refused**; `dt=499` builds |
| the condenser is the only ECLSS flow that may be slow at `dt=3600` | `5e-4 · 1800 = 0.9 < 1` vs the scrubber's `1.8` |
| no committed scenario is refused | all build with **no hatch** — the "no golden moved" assertion |
| the pins have teeth | reverting `_effective_step` to the plan's formula ⇒ **5 red across 2 files** |
