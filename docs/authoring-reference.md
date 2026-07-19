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
| `ScenarioSpec` | the whole file: run config (`name`/`integrator`/`dt`/`steps`/`rng_seed`/`n_sub`) + `includes` + `parameters` + `stocks` + `flows` + `forcings` |
| `BundleSpec` | a reusable domain/species bundle: `parameters`/`stocks`/`flows`/`forcings` — **no run config, no nested `includes`** (both rejected by `extra="forbid"`) |
| `IncludeSpec` | the `{bundle, prefix}` namespaced include form |
| `StockSpec` | one stock: id/domain/quantity/kind/amount + optional `composition`/`unclamped`/`extinction_threshold` |
| `FlowSpec` | one flow: a frozen `type` + `wiring` **xor** authored `kinetics`, + `priority`/`params`/`rate_class` |
| `KineticsSpec` | `rate` (the grammar above) × `stoichiometry` |
| `ForcingSpec` | `const` — **constant forcings only** |
| `ParamPackRef` | `{pack: …}` — an alternate param file read by the *same* frozen loader |

`integrator` is one of the frozen names `euler` / `rk4`; `rate_class` is one of `fast` /
`slow`. **Both vocabularies are frozen separately from the field that carries them**, by the
manifest's `integrator_names` and `rate_classes` keys — `schema_fields` records that a field
*exists*, never what it may *say*, so a silently-added third value would move no frozen set
without them. The `rate_classes` pair is **closed at two by `multirate_step`'s own
signature** (it takes exactly two `Substepper`s), unlike the flow-type registry, which is
expected to grow.

**Recorded schema relaxation (Phase-9 Step 6b):** `stocks` and `flows` are **optional** on both
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
class + its **exact `wiring` field set** + its param set + its `rate_params`. The manifest
freezes all four per entry: **the wiring names are as much the contract as the type name** —
renaming one breaks every scenario file that names it.

The registry is **explicit, not introspected, by design**: a `StockId` is a `str` alias at
runtime, so field-type introspection cannot tell a wiring field from any other string
field — and more to the point, this registry *is* the authoring contract, a deliberately
curated public surface. It is **expected to grow** (Phase-9 Step 0 registered the three
standalone Crew flows as the composition anchor); each addition is an unfreeze.

The post-roadmap **Tier-1 unfreeze** grew it to **twelve**, adding the nine standalone
Power / Thermal / ECLSS flows (`docs/plans/post-roadmap-flow-registry-growth.md`). The
`FORCED` column is the fact you most need and cannot see from the type name — it decides
how positivity holds, and whether the flow is multi-instanceable:

| type | wiring fields | param set | driver | `rate_params` |
|---|---|---|---|---|
| `crew.oxygen_consumption` | `o2_store`, `o2_consumed` | — | FORCED `crew_o2_intake` | — |
| `crew.food_metabolism` | `food_store`, `exhaled_co2`, `fecal_waste` | `crew` | FORCED `crew_food_intake` | — |
| `crew.water_balance` | `water_store`, `crew_humidity`, `urine` | `crew` | FORCED `crew_water_intake` | — |
| `power.solar_charge` | `solar_source`, `battery`, `waste_heat` | `charge` | FORCED `solar_power` | — |
| `power.load_draw` | `battery`, `waste_heat` | — | FORCED `load_power` | — |
| `power.self_discharge` | `battery`, `waste_heat` | `self_discharge` | donor (`k·battery`) | `self_discharge_rate` |
| `thermal.heat_input` | `heat_source`, `node` | — | FORCED `heat_load` | — |
| `thermal.radiator_reject` | `node`, `space` | `thermal` | donor, **nonlinear `T⁴`** | — (`τ ≫ dt` is not a predicate) |
| `eclss.crew_metabolism` | `cabin_o2`, `cabin_co2`, `cabin_h2o`, `metabolic_o2_sink`, `metabolic_co2_source`, `metabolic_h2o_source` | — | FORCED ×3 (`o2_consumption`, `co2_production`, `h2o_production`) | — (state-dependent) |
| `eclss.co2_scrubber` | `cabin_co2`, `co2_removed` | `eclss` | donor (`k_scrub·cabin_co2`) | `co2_scrub_rate` |
| `eclss.condenser` | `cabin_h2o`, `humidity_condensate` | `eclss` | donor (`k_cond·cabin_h2o`) | `condense_rate` |
| `eclss.o2_makeup` | `o2_supply`, `cabin_o2` | `eclss` | demand (`k·(setpoint − cabin_o2)`) | `o2_makeup_gain` |

**`rate_params` names the first-order rate constants** — the `k` in a `dx/dt = k·x` or
`k·(S−x)` leg, in `1/s` — and is the subset the build-time `k·h < 1` precondition reads (see
*The `dt` constraint*). An **empty** cell is a **declaration, not an omission**: a param-free
flow has no `k` to read, and the two annotated rows are uncoverable for reasons the
`rate_params` docstring records. It is an explicit list rather than "every float on the
params object" because `EclssParams` also carries **`o2_setpoint`, which is an inventory in
mol, not a rate**, and checking it would be meaningless.

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
| `eclss.co2_scrubber` | `k_scrub·dt < 1` | `dt = 60` → `0.06` | `dt = 3600` → **3.6** | **build** + rationing |
| `eclss.condenser` | `k_cond·dt < 1` | `dt = 60` → `0.03` | `dt = 3600` → **1.8** | **build** + rationing |
| `eclss.o2_makeup` | `k_makeup·dt < 1` | `dt = 60` → `0.12` | `dt = 3600` → **7.2** | **build ONLY** — rationing sees nothing in `1 ≤ k·dt < 2`; see below |
| `eclss.crew_metabolism` | forced draw < stock | `0.004·60 = 0.24` of 10 mol | `0.004·3600 = 14.4` of 10 mol | rationing (**state-dependent — no build check possible**) |
| `power.self_discharge` | `k·dt < 1` | `dt = 3600` → `3.6e-5` | `dt ≈ 1e8` s (~3 yr) | **build** + rationing |
| `thermal.radiator_reject` | `τ = C/(4εσA·T_eq³) ≫ dt` | `dt = 3600` → `τ ≈ 65` steps | a much larger `dt` overshoots | — (**"≫" is not a predicate**) |

**The "build" cells are the precondition**, and it is the platform's answer to the row it
names: the four rows whose constraint is a first-order `k` declare it as `rate_params` in the
registry, and `interpret` refuses `k·h ≥ 1` **before any step runs**. **`h` is the *effective*
step, not the file's `dt`** — `dt` single-rate, `dt/n_sub` for a **fast** flow, and **`dt/2`
for a `rate_class: slow` one** (Strang's half-step, *independent of `n_sub`*; see
*Multi-rate*, below, for what those keys mean). The last is the trap: a slow flow's step is
**not** `dt/n_sub`, and treating it as such reports a safe `k·h` for an unsafe flow. The two
rows without a "build" cell are uncoverable **by declaration, not by omission** — see the
`rate_params` docstring. Escape hatch for studying an unsafe run:
`interpret(..., allow_unsafe_step=True)` (Rust: `interpret_allowing_unsafe_step`).

**The precondition's honest claim is "the platform catches the `k·dt` family", never "your
`dt` is safe".** It reads a **declared** first-order rate constant and compares it against the
step that flow is actually integrated at. It cannot see `thermal.radiator_reject`'s `τ ≫ dt`,
`eclss.crew_metabolism`'s state-dependent over-draw, or an authored `kinetics` rate — the last
by construction, since `rate_params` lives on `FlowTypeSpec` and a kinetics flow has none.
**Neither gate subsumes the other**, which is why rationing was not retired when this landed:
the build check sees the param **pack** and the demand-controlled flow; rationing sees the
state-dependent over-draw. The **general** precondition stays deferred — a research problem,
not a consumer-phase task.

**It is at build time for the pack, not for the convenience.** The obvious argument — an
author learns before a long run — is true and secondary. The load-bearing one: a
`{pack: …}` may inflate a gain past every frozen guard (unit ✓, bound `> 0` ✓ — **a gain has
no `dt`-independent ceiling**, which is precisely why this can never be a loader bound), and
a pack's values exist only *after* `interpret` resolves them, so `run_scenario` — which
receives an already-built flow — **structurally cannot see it**. `packs/eclss_hot_makeup.yaml`
measures exactly that: the committed ECLSS anchor, at its own frozen and correct `dt = 60`,
taken from `k·dt = 0.12` to **1.2** purely by a pack. Because `o2_makeup` is
demand-controlled, without the build check that pack exports an oscillating cabin with
`rationed == 0` and **no gate anywhere reporting a problem**.

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
composing both **at a single `dt`** must pick one, and only **`dt ≤ ~60`** is safe for both
(a smaller `dt` only ever helps Thermal's overshoot margin). **There is no single `dt`
natural to both domains — which is why the author no longer has to pick one.** This
paragraph used to end "there is no `dt` natural to both domains" full stop, and read as a
statement about the platform rather than about single-rate integration; multi-rate falsified
it (see *Multi-rate*, below, and `tests/authoring/scenarios/eclss_thermal_habitat.yaml` —
the file this sentence called impossible). Both halves of the bind are measured, and
multi-rate escapes both: the shared `dt` is **unsafe** (single-rate `dt = 3600`: 840
rationings, `cabin_o2` = **72.0** against a truth of 8.0) *and* the safe shared `dt` is
**wasteful** (single-rate `dt = 60`: clean, and **20160** Thermal evaluations for a `τ` of
~65 steps).

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

**The build-time precondition this section used to defer is the "build" column above** — it
landed in the multi-rate unfreeze (2026-07-17) and is documented at the table rather than
here. The deferral's own two predictions both held (it is a *partial* detector; it is the
*only* protection for `eclss.o2_makeup`), and its one guess was **wrong in the unsafe
direction**: it specified the effective sub-step as `dt/n_sub` for every flow, which
false-PASSES a **slow** flow — see *Multi-rate*, below.

**Still deferred, by name** (the rest of "make it loud", which the rationing gate did not
do): **per-flow attribution** in the error — the message names the count and `dt`, not
*which* flow rationed, because `StepReport.rationed` is a bare count and widening it is a
`simcore` change — and an author-facing **run CLI** / Godot banner (there is no run CLI
today; `run_scenario` is the top of the **library** run path, which is why the raise lives
there).

**The other way to run an authored file** is `godot_bridge`'s `build_session_from_file`,
which bypasses `run_scenario` and so does **not** raise — deliberately. Read that as
**"does not raise *on rationing*, and does not refuse an unsafe `k·h` step either"**: the
asymmetry was designed when rationing was the only gate, and the `k·h` precondition lives in
`interpret`, *upstream of the split*, so this path had to choose. **It passes the study
hatch** (`authoring::load_scenario_allowing_unsafe_step`), which keeps the split literally
true. It is not silent either: `rationed` is in the observation projection,
`SimSession.total_rationed()` is exposed to GDScript, and `objectives_json` scores a rationed
session `no_rationing = false` → `survived = false`. **Library caller → exception;
interactive session → visible diagnostic + objective failure.** A player should watch the
cabin die; an author calling a function gets an exception.

**Stated plainly, because this one has a cost: `build_session_from_file` is the one surface
where the `k·dt` family is unguarded.** For the demand-controlled `eclss.o2_makeup` — where
the build check is the *only* detector there has ever been (see the next section) — an unsafe
makeup gain reaches a session with **every diagnostic reading clean**: `rationed` stays 0,
`no_rationing` stays true, `survived` stays true, and the cabin oscillates. That is
consistent with what a session *is* ("authored ≠ validated" — for watching, not for
vouching), **and it is a real hole**. An author who wants a verdict calls `run_scenario`.

**The case for refusing instead was strong and is recorded, not buried** — it was the
recommendation of both the implementer and the advisor, and two of its points remain true:
this path already maps every `AuthoringError` → `SimError::Validation` **which the UI
renders**, so refusing would have created no *silence*, only an earlier diagnostic; and
*"watch the cabin die"* is **factually wrong for what the precondition intercepts** — the
`k·h` family produces a **meaningless** run (measured: `72.0`, **nine times too much** O₂,
oscillating), not a death. The genuine "cabin dies" cases are the **state-dependent** ones
(`eclss.crew_metabolism`'s forced over-draw), which declare `rate_params = ()`, are **not**
refused by the precondition, and still run and die here exactly as documented. The decision
went the other way on the hatch's own stated purpose — studying an unsafe run *is* what it
exists for — and a maintainer reopening this should read both halves.

**Separately, and NOT on precondition grounds**: this path also refuses a **multi-rate**
file (`n_sub`/`rate_class`), parallel to the rk4 refusal and for the same reason —
`SimSession::single_rate` cannot drive `multirate_step`, and silently single-rating the file
would run it at the master cadence the author chose *because* it is unsafe un-sub-stepped.
The alternative to refusing was never honouring it; it was a silent cross-port divergence.

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

**The build-time precondition is what stands here, and in the band that matters it is the
*only* thing that does.** `eclss.o2_makeup` declares `rate_params = ("o2_makeup_gain",)` and
**is** checked at build. For the `1 ≤ k·dt < 2` rows that is not an *earlier* catch but the
**sole** one — they export `rationed = 0`, so no run-time gate ever fires. (Read that scoped
to the band, not to the whole table: the `2.40` row **diverges**, and a divergence grows
until it *does* over-draw, which is why rationing finally catches that one at `rationed = 2`.
The uncaught case is the **quiet** one, not the violent one.) The bound is the right one for
the donor-controlled rows *and* for this one, but **for opposite reasons** — over-draw there,
export fidelity here — and a precondition that had skipped `o2_makeup` as "already covered by
rationing" would have been wrong about the only row that needed it.

**The `1 ≤ k·dt < 2` rows in the table above can no longer be built without
`allow_unsafe_step=True`.** `tests/test_authoring_export_fidelity.py` — the file that
*measured* them — passes it deliberately, and that is the precondition working: the only
remaining way into this band is to say so out loud.

**None of this is a bug in the frozen flow.** The continuous law `dx/dt = k(S − x)` is
unconditionally stable — it decays to `S` from anywhere and cannot oscillate. The
oscillation is **explicit Euler failing to track a stable equation at too large a step**: a
solver property, not a physics one. The model reproduces its own closed-form solution with
textbook first-order error (halve `dt`, halve the error — pinned), and at `k·dt = 1` it hits
the analytic *deadbeat* prediction exactly (`12 → 10 → 10 → 10`). Reworking the kinetics
would be fixing the wrong layer.

### Multi-rate — the author picks a coupling cadence, not a global `dt` (post-roadmap)

The escape from the constraint two sections up. **`dt` stops being "the step everything is
solved at" and becomes the *coupling cadence* — what neighbouring domains see** — while each
flow sub-steps at the `dt` its own rate constant demands. Plan of record:
[`docs/plans/post-roadmap-multirate-authoring.md`](plans/post-roadmap-multirate-authoring.md).

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

The knob is **two schema fields** (`ScenarioSpec.n_sub`, `FlowSpec.rate_class`) driving
`simcore.multirate.multirate_step`, which was built in Phase 0.5 and is **consumed, not
changed**, here — `git diff src/simcore/` is empty for the whole unfreeze.

**The partition is per-flow, not per-domain**, and that is `multirate_step`'s own ruling
inherited rather than re-litigated: *"a cross-domain flow has no single-domain rate, so the
rate-class is a property of the flow, assigned by the scenario assembler; the driver takes
the two pre-built integrators and does not infer the partition."* The core **refuses to
guess**, so the authoring layer must ask. A per-flow *property* also has **zero
referential-integrity surface**: the rate-class travels *with* the flow into a bundle, and
`{bundle, prefix}` id-rewriting cannot touch it — which a top-level `fast: [flow-id, …]`
list of **id references** could not have claimed.

That last sentence was, for four commits, **asserted in three places and checked in none**:
both ports carried it as a comment on the line that copies the field, and no anchor file
declared a rate class *and* an `includes`. It is now anchored on both ports and compared
across them (`two_batteries_multirate.yaml` — a bundle-declared `rate_class: slow` must
reach the built partition under its **namespaced** id). The failure it guards is silent
rather than loud: whether the class is lost in the rewrite or the partition is computed
*before* `apply_includes`, the result is an **empty slow set**, which at `n_sub ≥ 2` builds
and runs clean. The graph dump is the primary gate here — it reads each flow's class off
`slow_registry` membership, so a mis-**built** partition is exactly what it renders (the
mirror of a mis-**driven** one, which it structurally cannot see).

**The frozen semantic choices** — each one, like `monod`'s, is a decision a reader can
otherwise only recover from a test:

| choice | resolution | because |
|---|---|---|
| `rate_class` default | **`fast`** | fast + `n_sub = 1` ⇒ an **empty slow set** ⇒ the bit-exact identity path (below). Default `slow` could not do this: with an empty *fast* set, Strang runs two `dt/2` half-steps, which is **not** one Euler step |
| the operator split | **Strang, not author-visible** | the core's own default and the higher nominal order. Lie is *cheaper* on the slow set (1 slow eval per master step vs 2) — the justification is **order/safety, not performance**, and Strang's `dt/2` is the *safer* slow step |
| `n_sub = 1` + a non-empty slow set | **`AuthoringError`** | it buys no rate separation and no perf win **while still moving the answer** (measured: `+1.2e-01` on the cabin). A misconfiguration, refused rather than honoured |
| the integrator per rate class | **one, from `integrator`** | `multirate_step` would accept `slow=rk4, fast=euler`; exposing it is deferred by name |

**The effective step is per-rate-class, and this is the trap the whole surface turns on:**

| case | the step that flow is integrated at |
|---|---|
| single-rate | `dt` |
| multi-rate, **fast** | `dt/n_sub` |
| multi-rate, **slow** | **`dt/2`** — Strang's half-step, ***independent of `n_sub`*** |

`n_sub` governs the **fast** set only. A slow flow steps at `dt/2` whether `n_sub` is 2, 60
or 1000, so reading its step as `dt/n_sub` reports a safe number for an unsafe flow —
measured at `k·h = 0.06` where the truth is **1.8**, with 24 rationings and `cabin_co2`
emptied to exactly 0.0. `interpreter._effective_step` is the one place the three cases are
named; the `dt/2` divisor **tracks the pinned split by assertion, not by comment**, because
Lie would step the slow set at the full `dt` and quietly loosen the check 2×.

**The identity path — why this unfreeze moved no golden.** `n_sub` defaults to 1 and
`rate_class` to `fast` ⇒ an empty slow set ⇒ `multirate_step` reproduces the single-rate
`step` **bit-for-bit** (`simcore`'s own contract, measured through the authoring layer over
every stock of every step, under both splits). But **the goldens rest on the branch, not on
that identity**: `run_scenario` routes on `is_multirate`, so a scenario that declares no
cadence takes the **pre-multi-rate loop verbatim** and never reaches the driver — pinned by
monkeypatching `multirate_step` to raise (Rust, which has no monkeypatch, pins the same
branch *behaviorally* via aux). The identity is corroborating; the branch is load-bearing.

**What it resolves, measured** (`tests/test_authoring_multirate_run.py`):

| mode, on the ECLSS anchor at master `dt = 3600` (truth = 8.0) | verdict | final `cabin_o2` |
|---|---|---|
| single-rate `dt = 3600` | **refused at build**; rations under the hatch | diverges — **72.0** |
| multi-rate `n_sub = 60` | `rationed == 0` | **8.000000000000007** |
| single-rate `dt = 60` (the reference run) | `rationed == 0` | **8.000000000000007** |

The bottom two rows are the point in one line: master `dt = 3600` with `n_sub = 60` lands on
the *same* value as `dt = 60`, **while exporting hourly**. The top row is the contrast that
gives them meaning — *the identical file at the identical `dt`, minus the `n_sub` knob* — and
its direction is worth reading, because the intuitive word for it is wrong: **72.0 is nine
times too much oxygen, not too little.** The hazard is not "the cabin suffocates", it is
"the number is meaningless" — `k·dt = 7.2` makes the update map alternate and grow, so the
*sign* of the error is an accident of where the oscillation is sampled.

`eclss_thermal_habitat.yaml` is the composition this reference called impossible: ECLSS
fast, Thermal slow, `rationed == 0`, the cabin at `o2_eq` and the node warming 102.70 →
277.44 K against `T_eq ≈ 280.9` (`tests/test_authoring_multirate_composability.py`, which
also pins the same 72.0 divergence on the *composed* graph, at 840 rationings over 336
steps — the same broken map, sampled elsewhere).

**What it does NOT do — three, and the first is the one to read:**

* **Multi-rate is the performance enabler, NOT the hazard closer.** An unsafe **effective
  sub-step** is the identical hazard one level down (`n_sub = 2` at `dt = 3600` gives 36.0
  against a truth of 8.0). The build-time precondition is the direct closer; that `n_sub = 2`
  now fails to *build* is the precondition's doing, not the knob's.
* **It costs accuracy versus single-rate RK4.** Strang is 2nd-order *only if both operators
  are RK4*, and **a Euler operator silently collapses Strang back to 1st order** — our frozen
  flows are Euler. The operators' non-commutativity is an O(`dt²`) term no sub-integration
  removes. It is worth it here only because the alternative at a coarse `dt` is not a *less
  accurate* value but a **meaningless** one.
* **It is not a licence to raise `dt`.** `k·dt < 1` stays operative on the **effective**
  sub-step, for the export-fidelity reason above.

**The saving is real but narrower than the cadence ratio suggests.** Thermal's evaluations
drop **30×**, not the 60× the ratio implies — Strang steps the slow set at `dt/2`, *twice*
per master step, so `20160 / (336 × 2) = 30.0` exactly. And the honest whole-run number is
**2.31×**: multi-rate saves the *slow* domain's work, and in that anchor the slow domain is
the cheap one. The large wall win lands where the slow set is **expensive** — the biosphere
— which is precisely the domain the registry cannot yet reach.

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

The contract is earned by Phase-9 Steps 0–6c and, since, by each post-roadmap unfreeze
(Tier 1's registry growth, Tier 2's `monod`, multi-rate) — full detail + measured results in
each plan of record:

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
- **The multi-rate partition is cross-port** — `eclss_multirate_cabin.yaml` (multi-rate
  Step 6b) is the one anchor that declares a coupling cadence, so it is the only place the
  two ports are compared on a **partition** rather than on a single-rate graph. It is pure
  ECLSS *deliberately*: the obvious candidate (`eclss_thermal_habitat.yaml`) is Tier-2
  (`T**4`) and could only ever be graph-dump-covered, and **the graph dump cannot see a
  mis-*driven* partition — only a mis-*rendered* one** (measured: forcing the Rust driver
  to split Lie leaves the dump green and turns the run red). Its teeth are the fast/slow
  **shared stock** (`eclss.cabin_h2o`): the Strang operators do not commute, so dropping
  its single `rate_class: slow` key moves the trajectory ~29 %. Before it existed, mutating
  the Rust interpreter to lower an all-fast partition left **the whole crossport suite
  green**. The partition there is a fixture device, not a sizing claim — see the file's
  header and "Frozen is not calibrated".
- **The `k·h` precondition refuses the right things, and nothing else** — the risk a
  *refusal* carries is refusing something that already worked, so
  `test_the_committed_scenarios_all_pass_the_precondition` asserts every committed scenario
  still builds on the author's default path, with no hatch. That is the assertion behind "no
  golden moved". The bound is `< 1` and not `≤ 1` by measurement (`k·h` exactly 1.0 — the
  deadbeat case — is **refused**; one `dt` unit below it builds), and the effective-step
  formula has teeth: reverting `_effective_step` to a flat `dt/n_sub` turns **5 red across 2
  files** on the Python side and 3 on the Rust side. Its behavior change is *itself*
  evidence — 22 committed pins across six files asserted the run-time rationing verdict and
  **could no longer construct their own subject** once the error moved to build time.
- **The interpreter is faithful** — the fifteen crossport anchor runs
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
- **The interpreter builds no-reset graphs only** — the biosphere's **two-rate master-day
  driver** and the `annual_reset` hook are not authorable. This bullet used to open "builds
  single-rate, no-reset graphs only", which the multi-rate unfreeze falsified — but **only
  half of it**, and the two "rates" here are different mechanisms that happen to share a
  word. `multirate_step`'s fast/slow **operator split** is now authorable (see *Multi-rate*);
  the biosphere's **master-day driver** — a daily/annual cadence with a reset hook — is a
  different thing entirely and stays deferred, with the rest of the biosphere.
- **Author-visible `split`** — `multirate_step` takes Strang **or** Lie; the harness pins
  Strang, and exposing the choice would buy an author no order they can use (Euler operators
  collapse Strang to 1st order anyway) while loosening the slow set's `k·h` check by exactly
  2× — Lie steps the slow set at the full `dt`. It is a **study** tool, per `simcore`'s own
  "fallback / comparison" framing, not an authoring knob.
- **Per-rate-class integrator** — `multirate_step` would accept `slow=rk4, fast=euler`; both
  substeppers are built from the scenario's single `integrator`.
- **The general `dt` precondition** — `k·h < 1` covers the *declared first-order rate*
  family and nothing wider. `τ ≫ dt`, state-dependent forced draws, and authored `kinetics`
  are uncoverable for the reasons recorded at *The `dt` constraint*; a general one is a
  research problem, not a consumer-phase task.
- **Parameter packs on the Rust port — and this one is a cross-port *soundness*
  precondition, not a feature gap.** `ParamsSpec::Pack` is a Rust error (a Phase-9 ruling),
  so Rust's `k·h` check reads the **frozen** constant (`flow_registry::frozen_rate_value`)
  where Python reads the **pack-resolved** value off the built flow. That is sound **only
  while the deferral holds**: the day packs land in Rust, `frozen_rate_value` becomes a
  **false PASS in the unsafe direction** — reporting the frozen `k` while the flow runs the
  pack's inflated one — and it would be invisible for exactly `eclss.o2_makeup`, the flow
  the check uniquely exists for. Pinned rather than commented
  (`pack_deferral_is_what_makes_the_frozen_rate_read_sound`). **A deferral in one port is
  load-bearing for a safety check in the other**; adding packs to Rust means fixing the read
  in the same commit.
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
