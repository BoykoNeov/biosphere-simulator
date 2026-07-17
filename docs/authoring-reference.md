# The authoring reference (frozen) — Phase 9, P9.7

Phase 9 turns the model into a **platform**: a scenario is *authored* in a declarative
file rather than programmed in Python or Rust. This file is the **freeze contract** for
the author-facing surface of that platform — what is frozen, the evidence the freeze
rests on, and the **unfreeze discipline** for ever changing a frozen item.

It is the Phase-4/6 freeze discipline applied **one level up**. Those contracts freeze
*science* — params, flow laws, scenarios → goldens
([`docs/biosphere-reference.md`](biosphere-reference.md),
[`docs/station-reference.md`](station-reference.md)). This one freezes the **grammar,
schema, and registry an author writes against**. The distinction matters: a mod authored
today keeps working tomorrow only if the *format* holds still, independently of whether
the science it composes is later recalibrated.

Its machine-readable companion is **`docs/authoring-reference.manifest.json`**
(generated; see *The manifest* below). The plan of record is
[`docs/plans/phase-9-scenario-authoring.md`](plans/phase-9-scenario-authoring.md).

## What "frozen" means (and what it does NOT)

**Frozen** = the items below are the committed contract. A change to any of them is an
**unfreeze event** that must follow the discipline at the bottom of this file. As
everywhere in this project, freezing is a *process* discipline, **not a code lock**:
nothing forbids editing `schema.py`. The completeness gate + the cross-port vectors make
an undocumented change *fail CI*, which is what gives the freeze teeth.

**Frozen ≠ finished.** The grammar in particular is **deliberately incomplete** (below) —
it is expected to grow. The discipline only insists each addition be *justified,
reviewed, and recorded*, never slipped in.

### Authored artifacts are NOT frozen — "authored ≠ validated" (decision B)

This is the load-bearing asymmetry of the whole phase. The **platform** is frozen; the
**scenarios authored on it are not**, and never become reference:

- an authored scenario is a **runtime object** — no golden, no manifest entry, no
  calibration claim, no place in any reference;
- the platform guarantees **conservation + determinism only**. Balance is structural
  (decision C: a flow is `rate × stoichiometry`, so a balanced coefficient vector stays
  balanced at every step for any rate), and the every-step conservation gate is a
  redundant backstop. **Scientific validity is the author's responsibility** —
  conservation-closed nonsense is authorable, by design;
- a run that used any authored kinetics carries `has_authored_kinetics` (rendered in the
  structural graph dump) — the uncalibrated marker. *How loudly each consumer surfaces it
  is not frozen here*: Godot reads it over the FFI and banners the run (see *Documented
  boundaries*), the CLI shows it in the graph dump, and a future consumer picks its own
  volume. What this contract fixes is that the marker is **available and honest**, not how
  it is drawn.

The scenario files under `tests/authoring/scenarios/` are consequently **test fixtures
and cross-port anchors, not a frozen scenario library**. Their byte-identity to
`crew_state.json` is evidence that the interpreter is faithful — it does not make them
reference.

### Delegated, not re-frozen

- **Param values.** An authored file reaches frozen param values only through the named
  loaders (`crew`, `self_discharge`, `charge`, `thermal`, `eclss`). Those files are frozen
  by [`docs/station-reference.manifest.json`](station-reference.manifest.json), named here
  via `delegates_to` and **never re-hashed** — the same pointer discipline by which the
  station manifest delegates the biosphere. One owner per frozen artifact. Delegation is
  about *ownership*, not endorsement: the station manifest freezes what those values **are**,
  not that they are **right** — see *Frozen is not calibrated*.
- **All science.** Flow laws, integrators, the engine's conservation/arbitration
  machinery: frozen by the biosphere/station/native-port contracts. This reference
  freezes only the *selection surface* over them.

## The frozen surface

The manifest is the authoritative, machine-checked list. This section is the
human-readable account.

### The bounded kinetics grammar (decision D)

A `kinetics` flow's `rate` is an expression in a **bounded, closed** grammar — a fixed,
finite primitive set with no user functions, recursion, loops, or I/O. Precedence and
associativity are **explicitly pinned** (`src/authoring/expr_parser.py`, mirrored
char-for-char by `rust/crates/authoring/src/expr_parser.rs`):

```
expr    := term  (("+" | "-") term)*         # left-associative
term    := factor ("*" factor)*              # left-associative
factor  := "-" factor | primary              # unary minus binds tighter than "*"
primary := number
         | "stock"   "(" string ")"
         | "param"   "(" string ")"
         | "forcing" "(" string ")"
         | "monod"   "(" expr "," expr ")"
         | "n"
         | "(" expr ")"
```

A `stock`/`param`/`forcing` argument is a **quoted string**, so any dotted id
(`power.battery`) is expressible without an identifier sub-grammar. `number` is standard
decimal/float syntax parsed by *this* parser, so the YAML-1.1 dotless-`1e-3`-is-a-string
hazard does not apply inside a rate.

`monod` is the sole **function** form (the post-roadmap Tier-2 unfreeze — see *Saturating
kinetics* below). Unlike the ref keywords it takes two **full sub-expressions**, and its
`,` is the grammar's only comma — legal nowhere else, so a bare `a, b` at top level is a
parse error rather than a sequencing operator.

**There is no `dt` token, by construction.** Exposing `dt` would let an author write
`f(dt)·dt` — non-linear in `dt` — silently forfeiting RK4 order. The rate is the
**instantaneous** (`dt`-independent) rate and `DeclarativeFlow` supplies the single
`× dt`, so **RK4-order-safety is structural, not a documented hope**. `n` stays readable
(`dt`-independent, safe).

### The grammar is DELIBERATELY INCOMPLETE — read this before assuming

Freezing this subset **does not** freeze a complete grammar. Each deferred op is waiting
on a **real frozen flow to force its semantic definition** (the "bespoke until a second
instance justifies it" discipline, applied to the grammar):

| Deferred | Why it is not shipped |
| --- | --- |
| `/` (bare division) | IEEE-unambiguous *except* at `x/0` — Python raises `ZeroDivisionError`, Rust f64 yields `inf`. A **cross-port semantic choice**, and **still open**: `monod` (the first divider) landed *without* lifting it, because `monod` guards its own denominator and so resolves `x/0` **internally**, never exposing the raw form. Shipping bare `/` would re-introduce exactly the hazard this defers, and no frozen flow forces it. |
| `exp ln pow sqrt abs min max clamp` | `clamp`/`ifpos` still carry a real definitional choice (inclusive bounds? `>0` vs `≥0`?). A transcendental also moves an authored flow from Tier-1 to Tier-2 under the [cross-port contract](native-port-reference.md) — note this does **not** apply to `monod`, whose division is an IEEE *basic* op. |
| bounded conditionals | Same — the comparison's edge semantics need a real flow. |
| named constants | A faithful Stefan-Boltzmann re-expression needs σ as a **module constant, not a param** (the `domains.thermal` / CODATA discipline). Unresolved. |

Adding any of them is a **deliberate unfreeze** (grammar node/op sets move ⇒ the
completeness gate fires), landing on **both ports** with new parse vectors. **`monod` is
the worked example** — it left this table by that exact route; see below.

### Saturating kinetics — `monod(substrate, half_saturation)` (post-roadmap Tier 2)

`monod(S, K) = S/(S+K)` — the one shape `+ − ×` cannot approximate, and the most common
functional form in the science this project models (Michaelis-Menten, Monod and Holling
type II share this algebra). Plan of record:
[`docs/plans/post-roadmap-grammar-monod.md`](plans/post-roadmap-grammar-monod.md).

**A frozen flow forced the definition — this is not a semantics the grammar invented.**
The op is the kernel of `domains.biosphere.chamber.oxygen_limitation_factor` (frozen
Phase-2 Step 7, cited to Davidson et al. 2012, used by three frozen flows), so both
choices the grammar had to make were *already made, and cited*:

| choice | resolution | because |
|---|---|---|
| 2-arg vs 3-arg `Vmax·S/(S+K)` | **2-arg, dimensionless** | frozen `f_O2` is dimensionless and applied as `daily · f_O2 · dt` — `Vmax` arrives through the already-frozen `*`, so a 3-arg form would freeze an argument order for nothing |
| `monod(0,0)` = `0/0` → NaN? | **`denom <= 0` → `0.0`** | the frozen line, verbatim ("no O₂ ⇒ no respiration") |

Arg order is `monod(substrate, half_saturation)` — a **frozen semantic choice**, pinned by
a parse vector, matching the frozen signature and MM convention.

Three properties an author can rely on:

- **It is total.** Every finite input returns a finite float: never NaN, never ±inf, never
  raising. So `0/0` cannot reach a hex-float golden, and the Python-raise-vs-Rust-`inf`
  split is unreachable. A negative denominator returns `0` rather than sign-flipping to a
  positive-looking factor.
- **It is Tier-1 bit-exact**, not Tier-2. Division is an IEEE-754 *basic* operation
  (correctly-rounded, deterministic cross-port), unlike the libm transcendentals. On the
  natural domain (`S ≥ 0`, `K > 0`) it is **bit-identical to the frozen `f_O2`** —
  gated by `tests/test_authoring_monod.py`, the `SelfDischarge`-oracle pattern applied to
  a grammar primitive.
- **It is RK4-order-safe.** `S/(S+K)` is C∞ on the natural domain. The obvious cheap
  alternative `min(k·S, Vmax)` is *not* equivalent: its kink is non-differentiable and
  destroys RK4's convergence order, and `rk4` is a frozen integrator name.

**What it does NOT do: clamp your substrate.** Only the frozen *kernel* is mirrored; the
frozen function's `max(0.0, o2_mol)/air_mol` is **argument preparation** for a depleting
physical pool, and in an authored rate that layer is *the sub-expressions you composed*. A
silent `max(0, ·)` would make `monod(stock("a") - stock("b"), k)` quietly mean something
else. If you need the clamp, write it.

> **`monod` is still bound by *Frozen is not calibrated*, from the other side.** It makes a
> saturating law *sayable*; the `K` you put in it is yours, and the platform will not tell
> you it is wrong. A rate using `monod` is authored kinetics, so it *does* raise
> `has_authored_kinetics` — unlike a frozen `type`, which raises nothing and is no more
> validated.

### The VM — `simcore/expr.py` (the one-time core addition, decision A)

The AST + evaluator + `DeclarativeFlow` live in `simcore` because an authored rate is
evaluated **per step, inside the integrator** (once per Euler step, per stage under RK4),
so it must be pure stdlib and deterministic like every other flow. *Parsing* text → AST
is a one-time **boundary** act and stays in `src/authoring`. This module is the
**one deliberate, one-time extension of the frozen core** since the biosphere/station
freeze — purely additive (every frozen golden stayed byte-identical), a single engine
primitive like adding an integrator, never per-scenario code.

**The VM is frozen by its grammar surface — the node union + the op set — not by a hash
of `expr.py`.** A code hash would add reformat/lint noise without a real gate, and the
VM's *behavior* is already pinned bit-exactly by `traj_vectors.txt`. The node set and op
set are what an author writes against, so those are what the manifest freezes.

### The scenario file schema

Eight pydantic spec models (`src/authoring/schema.py`), every one `extra="forbid"` — so
**each model's field set is exactly the legal key set** of an authored file, and a typo'd
key is a schema error, not a silent drop. The manifest freezes each model's fields.

| Model | Role |
| --- | --- |
| `ScenarioSpec` | the whole file: run config (`name`/`integrator`/`dt`/`steps`/`rng_seed`) + `includes` + `parameters` + `stocks` + `flows` + `forcings` |
| `BundleSpec` | a reusable domain/species bundle: `parameters`/`stocks`/`flows`/`forcings` — **no run config, no nested `includes`** (both rejected by `extra="forbid"`) |
| `IncludeSpec` | the `{bundle, prefix}` namespaced include form |
| `StockSpec` | one stock: id/domain/quantity/kind/amount + optional `composition`/`unclamped`/`extinction_threshold` |
| `FlowSpec` | one flow: a frozen `type` + `wiring` **xor** authored `kinetics`, + `priority`/`params` |
| `KineticsSpec` | `rate` (the grammar above) × `stoichiometry` |
| `ForcingSpec` | `const` — **constant forcings only** |
| `ParamPackRef` | `{pack: …}` — an alternate param file read by the *same* frozen loader |

`integrator` is one of the frozen names `euler` / `rk4`.

**Recorded schema relaxation (Step 6b):** `stocks` and `flows` are **optional** on both
`ScenarioSpec` and `BundleSpec` — a scenario composed purely from includes declares
neither inline. This was a required→optional change on both ports; it is frozen in that
relaxed form.

### The composition grammar (`includes`)

A scenario is *composed* from reusable bundles rather than re-declared inline. The merge
semantics are **cross-port** and frozen:

- **Order:** included bundles first (in `includes` order), then the scenario's own inline
  declarations. *(Unobservable in any serialized output — `sim_io` and the graph dump
  both id-sort — so it is pinned by a unit test on the list `apply_includes` returns,
  before canonicalization, on both ports.)*
- **No silent override:** a duplicate stock id, flow id, forcing key, or parameter name
  across any two sources is an `AuthoringError`.
- **Flat, one level deep:** a bundle carries no `includes` of its own.
- **Run config lives only in the top-level scenario.**
- **`overrides=` reaches a bundle-declared parameter** through the merge.

**Multi-instance namespacing (`{bundle, prefix}`).** A `prefix` namespaces every id the
bundle *declares* — stock id, flow id, forcing key → `<prefix>.<id>` — and every
*reference* to them: `wiring` values, `stoichiometry` keys, and the `stock(…)`/`forcing(…)`
refs inside a `kinetics` rate. The rate rewrite is a **structural AST walk**, not a string
scan; the prefixed AST is re-emitted by `render_rate_expr` (the parser's inverse). This is
what lets the **same** bundle compose twice.

> **`param(…)` is never prefixed.** In a rate it names a *frozen* param set, which two
> instances correctly **share**. Bundle-**parameter** namespacing is deferred.

**The `render_rate_expr` spelling is a per-port internal detail, NOT a cross-port
contract.** The graph dump omits rate strings and a trajectory depends only on the AST, so
the contract is *per-port round-trip stability* (`parse(render(node)) == node`) — Python
renders a `Const` via `repr`, Rust via `f64::Display`; both round-trip on their own port.

### The author-selectable flow-type registry

`FLOW_TYPES` (`src/authoring/flow_registry.py`) maps an authoring type name → the frozen
class + its **exact `wiring` field set** + its param set. The manifest freezes all three
per entry: **the wiring names are as much the contract as the type name** — renaming one
breaks every scenario file that names it.

The registry is **explicit, not introspected, by design**: a `StockId` is a `str` alias at
runtime, so field-type introspection cannot tell a wiring field from any other string
field — and more to the point, this registry *is* the authoring contract, a deliberately
curated public surface. It is **expected to grow** (Step 0 registered the three standalone
Crew flows as the composition anchor); each addition is an unfreeze.

The post-roadmap **Tier-1 unfreeze** grew it to **twelve**, adding the nine standalone
Power / Thermal / ECLSS flows (`docs/plans/post-roadmap-flow-registry-growth.md`). The
`FORCED` column is the fact you most need and cannot see from the type name — it decides
how positivity holds, and whether the flow is multi-instanceable:

| type | wiring fields | param set | driver |
|---|---|---|---|
| `crew.oxygen_consumption` | `o2_store`, `o2_consumed` | — | FORCED `crew_o2_intake` |
| `crew.food_metabolism` | `food_store`, `exhaled_co2`, `fecal_waste` | `crew` | FORCED `crew_food_intake` |
| `crew.water_balance` | `water_store`, `crew_humidity`, `urine` | `crew` | FORCED `crew_water_intake` |
| `power.solar_charge` | `solar_source`, `battery`, `waste_heat` | `charge` | FORCED `solar_power` |
| `power.load_draw` | `battery`, `waste_heat` | — | FORCED `load_power` |
| `power.self_discharge` | `battery`, `waste_heat` | `self_discharge` | donor (`k·battery`) |
| `thermal.heat_input` | `heat_source`, `node` | — | FORCED `heat_load` |
| `thermal.radiator_reject` | `node`, `space` | `thermal` | donor, **nonlinear `T⁴`** |
| `eclss.crew_metabolism` | `cabin_o2`, `cabin_co2`, `cabin_h2o`, `metabolic_o2_sink`, `metabolic_co2_source`, `metabolic_h2o_source` | — | FORCED ×3 (`o2_consumption`, `co2_production`, `h2o_production`) |
| `eclss.co2_scrubber` | `cabin_co2`, `co2_removed` | `eclss` | donor (`k_scrub·cabin_co2`) |
| `eclss.condenser` | `cabin_h2o`, `humidity_condensate` | `eclss` | donor (`k_cond·cabin_h2o`) |
| `eclss.o2_makeup` | `o2_supply`, `cabin_o2` | `eclss` | demand (`k·(setpoint − cabin_o2)`) |

**The biosphere is absent for a structural reason, not a calibration one.** `Allocation`
takes a composite `ctx: CarbonContext` (four param objects + four stock ids) plus `pheno`
and `alloc` — which a flat `wiring_fields` tuple and a single `param_set` cannot express.
It also needs the aux accumulator, the shared `co2_pool` feedback var, and the two-rate
master-day driver, all deferred. It wants a *frozen-compartment include*, not flow-type
entries. That is a phase of work, not a step.

`PARAM_LOADERS` names the frozen loaders a file may reach: `crew`, `self_discharge`,
`charge`, `thermal`, `eclss`. A **parameter pack** is read by the *same frozen loader*, so
a pack's values pass the frozen schema/bounds/unit validation — **a pack is a param file,
not a way around the guards**. Because every loader returns a flat dataclass of floats,
each set is reachable *both* as a frozen type's `param_set` and as a `kinetics` rate's
`param("…")` source — so an authored rate may read η_c, the radiator properties or the
ECLSS gains and get the **frozen** value through the **frozen** guards.

### Frozen is not calibrated — read this before trusting a number

The registry decides **which rate laws** an author may select. It says nothing about
whether those laws' **numbers** are right, and for these nine flows they are not known to
be. This is the single most misreadable fact on this page, so it is stated plainly:

> **55 of the project's 57 parameters are uncited placeholders.** Every param reachable
> through the loaders above carries `source: "TODO(cite) — provisional … pending
> validation gate"` **except `crew`'s two**, which are calibrated to NASA BVAD Table 3-31.
> `eclss.yaml` says so in as many words: *"deliberately NOT NASA BVAD / BioSim life-support
> numbers"*.

The *equations* are honest, cited primary literature (Stefan–Boltzmann is Incropera; the
ECLSS loops are Seader/Ogata; the FvCB biosphere is Farquhar et al. 1980). The *values* are
plausible guesses. **Structure is literature-derived; values are not calibrated.** The gate
that would settle them — the deferred Phase-1 quantitative oracle match — has never run,
and its tests are opt-in (`-m oracle`) and absent from the default gate.

Two consequences an author must hold onto:

1. **No `UNCALIBRATED` banner means "no authored kinetics" — NOT "validated."**
   `has_authored_kinetics` measures **who wrote the rate law**, never whether the science
   is validated. Those are independent axes, and the marker is silent on the second one:

   | axis | asks | fixed by |
   |---|---|---|
   | **shape** of the law | is the functional form right? | the registry / the grammar |
   | **values** in the law | are the numbers right? | validation (not yet done) |

   A scenario built entirely from frozen types carries **no** marker and is **no more
   validated** than one that authored its own kinetics. It is merely *not the author's
   fault*. A calibration signal for the values axis is future work: it cannot discriminate
   until there is something to discriminate — today it would report "uncalibrated" for
   everything except crew, which this section already says in prose.

2. **Selecting a frozen flow buys you reuse and conservation, not credibility.** That is
   still worth a great deal: the law's *shape* is literature-derived and cross-port
   pinned, and you inherit the frozen bounds/unit guards. Just don't read the absence of a
   warning as an endorsement.

Deviating from reality is **legitimate** here — interesting, educational, even fun — *as
long as it is deliberate and documented*. This section is the "documented" half.

### The `dt` constraint — the frozen params assume one, and you pick it

**This is the sharpest hazard on the platform, and registration is what created it.** Every
frozen rate constant was sized against the `dt` of *its own frozen scenario*, and that
sizing is part of the flow's positivity argument — but it lives in a YAML comment, not in
the code. Selecting a flow type hands you the `dt` knob with no guard attached.

| flow | constraint | frozen sizing | breaks at | caught by |
|---|---|---|---|---|
| `eclss.co2_scrubber` | `k_scrub·dt < 1` | `dt = 60` → `0.06` | `dt = 3600` → **3.6** | rationing |
| `eclss.condenser` | `k_cond·dt < 1` | `dt = 60` → `0.03` | `dt = 3600` → **1.8** | rationing |
| `eclss.o2_makeup` | `k_makeup·dt < 1` | `dt = 60` → `0.12` | `dt = 3600` → **7.2** | **nothing in `1 ≤ k·dt < 2`** — see below |
| `eclss.crew_metabolism` | forced draw < stock | `0.004·60 = 0.24` of 10 mol | `0.004·3600 = 14.4` of 10 mol | rationing |
| `power.self_discharge` | `k·dt < 1` | `dt = 3600` → `3.6e-5` | `dt ≈ 1e8` s (~3 yr) | rationing |
| `thermal.radiator_reject` | `τ = C/(4εσA·T_eq³) ≫ dt` | `dt = 3600` → `τ ≈ 65` steps | a much larger `dt` overshoots | — |

**The `o2_makeup` row is not like the others, and its `< 1` means something different.**
Every other row is **donor-controlled** or forced: the draw is `k·dt·stock`, so at `k·dt > 1`
it demands more than the whole stock, the backstop must scale it, and `run_scenario` raises.
Those bounds are *enforced*. `eclss.o2_makeup` is the registry's only **demand-controlled**
flow — its draw is `k·dt·(setpoint − stock)`, proportional to the **error**, not the stock.
Near the setpoint that draw is small no matter how large `k·dt` is, so it **never over-draws
and the backstop never fires**. Its `< 1` is honoured by the author or not at all.

Read that "caught by" cell precisely — it is scoped to a *band*, not to the row's tabulated
`dt`. At the tabulated `dt = 3600` (`k·dt = 7.2`) the controller **diverges**, and a
divergence grows until it *does* over-draw, so rationing catches that one (measured:
`rationed = 4`). What nothing catches is the **`1 ≤ k·dt < 2` band** (`dt ≈ 500–1000`), where
the oscillation stays near the setpoint, never over-draws, and converges to the right
endpoint. It is the *amplitude* that trips the gate, never the oscillation — which is why
the uncaught case is the *quiet* one, not the violent one.

**The composability constraint** falls straight out of that table, and it is not derivable
from any single flow: **ECLSS is sized for `dt = 60`; Thermal for `dt = 3600`.** A scenario
composing both must pick one `dt`, and only **`dt ≤ ~60`** is safe for both (a smaller `dt`
only ever helps Thermal's overshoot margin). There is no `dt` natural to both domains.

**The failure WAS silent — it now raises.** Measured, not assumed
(`tests/test_authoring_dt_hazard.py`). At `dt = 3600` the ECLSS cabin did **not** raise,
**conserved** every quantity every step, completed with `rationed = 37` — and ended with
`cabin_o2` at `-1.4e-14`. *The cabin had no oxygen and the platform reported success.* The
arbitration backstop scales the over-draw so nothing goes properly negative, which is
exactly why nothing was raised; the only signal was the `rationed` count from
`run_scenario`, and `states, _, _ = run_scenario(built)` throws it away.

`run_scenario` now raises **`RationedError`** (Rust: `AuthoringError` with
`kind = ErrorKind::Rationed`) whenever `total_rationed > 0`, in both ports. The escape
hatch is `allow_rationing=True` (Rust: `run_scenario_allowing_rationing`) — for
*inspecting* a rationed run, never for making a scenario "work".

**Read the scope of that fix precisely, because it is easy to over-read:**

* **The silence is fixed. The hazard is not.** The physics, the params, and every frozen
  sizing are untouched; `k_scrub·dt` is still `3.6` at `dt = 3600` and the cabin still
  asphyxiates — you can still watch it with the escape hatch, and both ports' tests still
  assert that it does, at the same numbers. **You get an exception instead of a corpse.**
  The table above is still the thing you must design around.
* **It is a consistency fix, not a new policy.** Every other layer had already reached
  this verdict: the goldens assert `rationed == 0`; `simcore.integrator.StepReport` calls
  a nonzero count *"a failing gate, not a warning"*; RK4 hard-errors on the identical
  condition (`ArbitrationError`); `station.objectives` scores a rationed run
  `survived = False`. `run_scenario` was the lone surface that detected it and said
  nothing.
* **`RationedError` is deliberately not an `AuthoringError`.** That class is defined as
  what is decidable *from the file structure alone, before any step runs* — and the same
  file at a smaller `dt` is fine. Nothing about rationing is structural.
* **Conservation was never going to catch this**, which is the transferable lesson: the
  backstop clamps at zero rather than going negative, so a run can balance perfectly and
  still have emptied the stock that keeps the crew alive. *Mass conservation is not
  survival.* An authored graph now needs **two** things to be sound — it balances, **and**
  it never rationed.

None of this is a bug in the frozen flows. Each is correct at the `dt` it was sized for;
the frozen sizing argument is simply scoped to the frozen scenario, and **authoring is what
escapes that scope**.

**Still deferred, by name** (the rest of "make it loud", which this did not do): a
**build-time** `k·dt < 1` precondition per flow type — it would catch the scrubber/condenser
*before* running rather than after, but it is only a partial detector (it cannot see
`eclss.crew_metabolism`, whose failure is `forcing·dt > stock`, not `k·dt > 1`, nor any
coupled multi-flow dynamic), and it would live in the **frozen registry**, so it is an
unfreeze with its own ceremony. **It must cover `eclss.o2_makeup`, where it would be the
*only* protection** rather than an earlier one — see "the hazard rationing cannot see"
below; reading that row as "already covered by rationing" is the mistake to avoid. And if
multi-rate is ever authorable, the precondition must check the **effective sub-step**
(`dt/n_sub`), never the master `dt`. Also deferred: **per-flow attribution** in the error (the
message names the count and `dt`, not *which* flow rationed — `StepReport.rationed` is a
bare count, and widening it is a `simcore` change), and an author-facing **run CLI** /
Godot banner (there is no run CLI today; `run_scenario` is the top of the **library** run
path, which is why the raise lives there).

**The other way to run an authored file** is `godot_bridge`'s `build_session_from_file`,
which bypasses `run_scenario` and so does **not** raise — deliberately. It is not silent
either: `rationed` is in the observation projection, `SimSession.total_rationed()` is
exposed to GDScript, and `objectives_json` scores a rationed session
`no_rationing = false` → `survived = false`. **Library caller → exception; interactive
session → visible diagnostic + objective failure.** A player should watch the cabin die;
an author calling a function gets an exception.

### `eclss.o2_makeup` — the hazard rationing cannot see (post-roadmap, measured)

The same scoping bites `eclss.o2_makeup`, and **not mildly**. This section used to call it
mild; measuring it (`tests/test_authoring_export_fidelity.py`) showed otherwise, and the
reason it is worse is the reason it is *invisible*.

**The reversal is real, and it is the harmless half.** `eclss.o2_makeup` is a *linear,
unclamped* proportional controller. Its frozen docstring notes an above-setpoint venting
clamp is "a deferred seam that never arises here" — true of every frozen scenario, but an
author can wire `cabin_o2` above the `10.0 mol` setpoint, at which point the rate goes
negative and the flow **reverses**, venting cabin O₂ back into the supply tank (measured:
wired at `20.0`, step 1 moves `−1.2 mol` cabin → tank). It conserves, it does not ration;
it is simply not what "makeup" suggests. **Do not "fix" this with a clamp**: the symmetry
IS the restoring force — `o2_eq = o2_setpoint − Con_o2/k_makeup` is an attractor from both
sides only because the controller is linear, and clamping would trade a clean geometric
contraction for a piecewise nonlinearity. The reversal is correct P-control.

**The harmful half is the oscillation, and `k·dt < 1` is what stands between you and it.**
The textbook stability bound of a proportional controller is `k·dt < 2`, not `< 1` — so it
is tempting to read the table's `< 1` as over-conservative and relax it. **Do not.** The
band between them is where this platform gets hurt:

| `k_makeup·dt` | dt | exported `cabin_o2` | rationed | verdict |
|---|---|---|---|---|
| `0.12` | 60 | `20 → 18.6 → 17.3 …` monotone | 0 | the frozen dt; export-clean |
| `1.80` | 900 | `12 → 8.4 → 11.28 → 8.976 → 10.8` | **0** | converges — **exports oscillation** |
| `2.00` | 1000 | `12 → 8 → 12 → 8 …` **forever** | **0** | undamped; never converges |
| `2.40` | 1200 | `12 → 7.2 → 13.9 → 4.51 → 17.7 → 0` | 2 | diverges → finally caught |

`1 ≤ k·dt < 2` reaches the **correct equilibrium** — so an endpoint check passes,
conservation passes, and `rationed == 0`. Only the *intermediates* are wrong. In a coupled
station those intermediates are **exported to neighbouring domains every step**, and a
neighbour reading `cabin_o2` sees oxygen sloshing ±20 % that no real cabin does. Converging
to the right answer eventually does not license exporting wrong answers on the way — which
is why **`k·dt < 1` (monotonicity) is the operative bound here, not `k·dt < 2` (stability)**.
`< 2` answers "does the solver diverge". `< 1` answers "is the exported trajectory usable by
a neighbour". This project couples domains, so `< 1` governs.

**Why no gate sees it — the transferable part.** Rationing detects **large excursions that
over-draw**. It is structurally blind to **near-setpoint oscillation that stays positive**,
because a demand-controlled draw shrinks as it approaches the setpoint. The `dt` fix above
taught *mass conservation is not survival*; this teaches the next one:
**`rationed == 0` does not mean the exported trajectory is right.**

**The full cabin is protected only by coincidence** — `k_scrub·dt = 1` and `k_makeup·dt = 2`
both land at `dt = 1000`, so the *scrubber's* gate usually fires first and hides the makeup
loop's own cliff. At `dt = 1000` exactly the scrub draw *equals* the stock rather than
exceeding it, nothing rations, and the measured cabin swings **12 ↔ 4 mol forever** while
`o2_supply` drains past **−800 mol** — reported as a clean, conserving, unrationed run. An
author who registers `eclss.o2_makeup` *without* a scrubber never had that coincidence at all.

**This corrects the deferred build-time precondition named above:** a `k·dt < 1` check is
the right bound for the donor-controlled rows *and* for `o2_makeup` — but for opposite
reasons (over-draw vs export fidelity), and only `o2_makeup`'s is otherwise unenforced. A
precondition that skipped `o2_makeup` as "already covered by rationing" would be wrong.

**None of this is a bug in the frozen flow.** The continuous law `dx/dt = k(S − x)` is
unconditionally stable — it decays to `S` from anywhere and cannot oscillate. The
oscillation is **explicit Euler failing to track a stable equation at too large a step**: a
solver property, not a physics one. The model reproduces its own closed-form solution with
textbook first-order error (halve `dt`, halve the error — pinned), and at `k·dt = 1` it hits
the analytic *deadbeat* prediction exactly (`12 → 10 → 10 → 10`). Reworking the kinetics
would be fixing the wrong layer.

### Templates — the boundary's arithmetic (decision A, as amended)

A `StockSpec.amount` or `ForcingSpec.const` may be a **template expression** over the
scenario's declared `parameters` (`param('crew_count') * 1000.0`), lowered to a literal at
interpret time. It reuses the *same* `expr_parser` (so precedence is pinned in one place)
but evaluates at build time, where no `State`/`env`/`n` exists — only
`Const`/`ParamRef`/`Neg`/`BinOp`.

This **amends decision A**, which originally held that the boundary does no float math:
boundary-eval `+ − ×` is a **cross-port surface**, stated rather than slipped. It is
benign (IEEE-deterministic; decimals round-trip through any correct-rounding parser) and
proven bit-exact by the hex-float graph dump.

## The evidence the freeze rests on

The contract is earned by Steps 0–6c (full detail + measured results in the plan):

- **Grammar semantics are cross-port pinned** — `parse_vectors.txt`: 26 accept cases
  render an **identical canonical S-expr** on both ports; 25 reject cases error on both
  (the *message* is deliberately not pinned). A precedence or associativity change moves
  a rendering and fails. Since Tier 2 this also pins `monod`'s **arg order** and its
  arity/comma rejections — the comma is legal nowhere but inside the call.
- **The VM's arithmetic is cross-port bit-exact** — `traj_vectors.txt`: the frozen
  `SelfDischarge` re-expressed as an authored flow is bit-identical to the frozen
  constructor's trajectory, per step, under **Euler *and* RK4** (RK4 is nontrivial —
  `SelfDischarge` is donor-controlled ⇒ RK4 ≢ Euler). The frozen flow is the oracle; no
  new golden was invented. Tier 2 added a third scenario carrying `monod` in the
  *evaluated* path — a saturating, donor-controlled drain that self-limits so
  `rationed == 0` comes from **kinetics, not the backstop** (the frozen `f_O2` story in
  miniature), bit-exact on both ports under both schemes.
- **The `monod` op IS the frozen science** — `tests/test_authoring_monod.py` pins the
  kernel **bit-exact** against `chamber.oxygen_limitation_factor` across its frozen
  domain, and pins totality (no finite input yields NaN/±inf) by measurement rather than
  argument. This is the `SelfDischarge` oracle pattern applied to a *grammar primitive*:
  the evidence that the op is the frozen law, not merely inspired by it.
- **`monod` survives the file path too** — `monod_dsl.yaml` is the deliberate twin of
  `self_discharge_dsl.yaml` (same battery, same frozen param set, same ±1 split) with the
  rate made saturating, so the diff between them is exactly what Tier 2 added. It exists
  because the traj vectors build a `DeclarativeFlow` *programmatically*, leaving one real
  surface uncovered: the **comma** is the first rate-grammar character that is also
  YAML-significant. It is sized so the battery actually drains (the monod factor slides
  0.667 → 0.226) — an earlier draft was inert and the crossport suite passed anyway,
  which is worth remembering: **a dead anchor is trivially bit-exact.**
- **The interpreter is faithful** — the fourteen crossport anchor runs
  (`tests/crossport/authoring_files.py::ANCHORS`, the authoritative live list), including
  an authored crew run **byte-identical to the frozen `crew_state.json`** — via a bare
  file, via a template at its default, and via a single-bundle `include` — and, since
  Tier 1, an authored ECLSS cabin **byte-identical to the frozen `eclss_state.json`** on
  *both* ports. That last one carries the registry's weight: it takes the byte-identity
  claim from one single-quantity domain to a three-quantity one whose six-leg forced flow
  balances CARBON, OXYGEN and WATER independently, across nine stocks the registry had to
  wire correctly (including the unclamped-source vs clamped-sink split and the
  single-quantity cabin compositions) with **no build-time stoichiometry check to catch a
  mistake** — that check runs for authored `kinetics` flows only, never for frozen `type`
  flows, so an error would surface only as a runtime `ConservationError`.
- **The registry mirrors the frozen constructors** — `test_authoring_frozen_flows.py`
  derives each frozen class's *actual* dataclass fields and asserts every `FlowTypeSpec`
  declares exactly `(id, priority, *wiring[, params])`. Added by Tier 1 because nothing
  owned it: the manifest gate owns completeness, the anchors own does-it-run, and neither
  notices a `wiring_fields` entry naming a field that does not exist on a type no anchor
  happens to exercise.
- **The safety spine has teeth** — a mis-wired flow (e.g. a carbon leg pointed at an
  oxygen stock) interprets cleanly and then raises `ConservationError` on step 1: bad
  wiring is **surfaced, never silently fixed**.
- **Composition is cross-port** — the two-bundle merge, the mixed include+inline anchor,
  and the same-bundle-twice namespaced anchor (each half bit-identical to a frozen
  single-instance run) all match across ports.

Tests of record: `tests/test_authoring_{crew,param_packs,kinetics,templates,compose}.py`,
`tests/crossport/test_crossport.py`, `rust/crates/authoring/tests/`.

## The manifest

`docs/authoring-reference.manifest.json` is the machine-readable surface, **generated** by
`tests/test_authoring_freeze_manifest.py`
(`uv run python tests/test_authoring_freeze_manifest.py`). Every frozen set in it is
**derived from its live single source of truth, never hand-listed** — the discipline both
prior manifests rest on.

**What the gate checks vs. what the vectors + anchors check** — the division is deliberate,
and mirrors the biosphere's:

- **The vectors + anchors own *semantics and values*.** `parse_vectors.txt` owns the
  grammar's meaning, `traj_vectors.txt` owns the VM's arithmetic, the anchors own the
  interpreter, and the station manifest owns the param values. The gate re-asserts none of
  it. Its two provenance hashes (of the vector files) are a re-derivable record of *which
  cases were frozen*, regenerated on a deliberate unfreeze — **not** assertions.
- **The gate owns *completeness*** — the one thing the vectors and anchors are blind to: a
  grammar node, a binary op, a schema field, a whole spec model, a flow type, or a param
  loader **added to the live tree but exercised by nothing**. This is the biosphere's
  "added a flow, wired into no golden" hole, one authoring level up. A teeth test on the
  flow registry and another on the schema scan confirm the gate actually fails on a
  phantom.

**Cross-port boundary, stated honestly.** The manifest freezes the **Python** surface of
record. The Rust mirror is gated by the parse/traj vectors + the anchors, not by this gate
— a Python schema field added with no Rust mirror is caught only once an anchor exercises
it. Parity surfaces own cross-port fidelity; this gate owns single-port completeness.

## Documented boundaries (deferred, by name — not omissions)

A future maintainer should read these as intentional scope, each surfaced when it was hit:

- **A forcing-bound frozen flow cannot be prefixed — and Tier 1 widened this from three
  flows to nine.** The frozen crew flows read `crew_o2_intake` etc. from a **hardcoded
  module constant**, not through wiring, so they cannot find a namespaced forcing key — a
  prefixed crew include fails at resolve time. (The greenhouse `CARBON_POOL` analogue:
  re-point the side that *can* be.) Kinetics / disjoint bundles namespace cleanly, which is
  why the multi-instance anchor is two batteries.

  Six of the nine flows Tier 1 registered are forcing-bound the same way, so the same
  boundary applies to each: `power.solar_charge` (`solar_power`), `power.load_draw`
  (`load_power`), `thermal.heat_input` (`heat_load`), and `eclss.crew_metabolism` (all
  three of `o2_consumption` / `co2_production` / `h2o_production`). **Single-instance
  authoring works** — declare a forcing under the exact hardcoded name, as
  `power_bus.yaml` / `thermal_node.yaml` / `eclss_cabin.yaml` do — but a second, prefixed
  instance collides on the forcing key. Two batteries are multi-instanceable because
  `power.self_discharge` is donor-controlled and reads a *stock*; two solar arrays are not.
  Renaming a forcing key is not a build error: it surfaces as a resolve-time `KeyError` at
  step 1.
- **Bundle-parameter namespacing** — two prefixed instances of a param-bearing bundle
  collide on the parameter name. The honest boundary; the only param-bearing bundle (crew)
  is un-multi-instanceable for the forcing reason anyway.
- **Parameter packs inside an included bundle** — a bundle pack would have to resolve
  against *the bundle's* directory (per-flow source-dir threading). A `{pack: …}` on a
  bundle flow is a clean `AuthoringError`; top-level scenario flows resolve packs fine.
- **Shared-stock composition** — two bundles pointing at one shared stock (the Phase-6
  cabin sharing, hand-coded in `station/`) is a larger deferral.
- **Nested includes** — rejected; composition is flat, one level deep.
- **The interpreter builds single-rate, no-reset graphs only** — the two-rate master-day
  driver and the `annual_reset` hook are not authorable.
- **Computed forcing schedules** (the Power half-sine, biosphere weather) — `const` only.
- **Derived initial conditions** — the *simulation-derived* tier (the station's `node0` =
  run Power, take the mean) needs running a sub-sim; deferred further than the
  arithmetic-derived tier templates cover.
- **"Authored ≠ validated" surfacing** — *the Godot half is now BUILT* (the recorded Step-7
  follow-up, done 2026-07-16). `SimSession.has_authored_kinetics()` exposes the marker across
  the FFI boundary and `from_file_dashboard` shows an UNCALIBRATED banner when it is set;
  `tests/crossport/test_godot_from_file.py` gates **both** halves — the marker crossing the FFI
  (including that the flag **clears** when the same session is rebuilt into a palette scenario:
  it is per-session state, not per-file) and the banner *itself* being drawn, driven through the
  real widget on a kinetics-bearing file. Consumer-side display only: no
  frozen surface moved (the marker is an interpreter *output*, named by no manifest key), the
  science is untouched, and no golden shifted. **Still deferred:** the CLI surfaces the marker
  only through `dump_graph`, because `station::sim` has no scenario-file dispatch (its own
  deferral, below-adjacent); an authored file still declares no display hints.

## The unfreeze discipline

Changing **any** frozen item — a grammar node or op, a schema field, a spec model, a flow
type or its wiring names, a param loader, or the composition semantics — is an
**unfreeze**. The procedure (the biosphere/station discipline, applied to the platform):

1. **Justify + review.** Write down *why*. For a **grammar** change especially, get it
   **advisor-reviewed before writing anything** — each deferred op carries an unresolved
   cross-port semantic choice (`x/0` is the canonical example), and freezing the wrong
   answer is worse than deferring.
2. **Make the change** boundary-side wherever possible. `git diff src/simcore/` **must
   stay empty** — `simcore/expr.py` was the *one* sanctioned, one-time addition (decision
   A); a second core edit is not an unfreeze, it is a new decision needing its own review.
3. **Land it on BOTH ports.** The grammar/schema is a cross-port contract: a Python-only
   change is a broken contract, not a half-done one. Add the parse/traj vectors or the
   anchor that pins the new surface.
4. **Regenerate the manifest** (`uv run python tests/test_authoring_freeze_manifest.py`)
   and review its diff — the changed sets are the git-visible record of exactly what was
   unfrozen.
5. **Record provenance.** Update this file and the Phase-9 plan with what changed and why
   (a deferred op joining moves it out of the *deliberately incomplete* table above).
6. **Re-run the gates:** the full suite (incl. `-m slow`), `ruff`, `pyright`, `cargo test`,
   `clippy -D warnings`; confirm the **20 frozen goldens stay byte-identical** (an
   authoring change that moves a science golden is a bug, not an unfreeze); commit with a
   Conventional Commit that names the unfreeze.

An undocumented unfreeze fails CI by construction (the completeness gate, or a moved
vector/anchor), so the discipline is enforced, not merely requested.
