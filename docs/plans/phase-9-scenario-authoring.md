# Phase 9 — Scenario Authoring & Modding (the model becomes a platform)

**Status: IN PROGRESS — Steps 0–3 + Step 4a + Step 4b COMPLETE (composition + parameter packs + the
bounded kinetics DSL + templates + the Rust VM/parser port with parse- & trajectory-parity + the Rust
runtime scenario-FILE interpreter); Steps 5–7 JIT.** Step 4 was SPLIT (advisor-recommended,
USER-CONFIRMED): 4a ported the AST/VM + rate-expr parser to Rust with the two cross-port parity
surfaces (parse-parity Tier-0 + SelfDischarge trajectory-parity Tier-1, Euler+RK4) at **zero YAML
dependency**; **4b (COMPLETE) added the runtime scenario-file layer** — decision E resolved
USER-CONFIRMED to a **hand-rolled closed-subset YAML parser** (one grammar owned on both ports, no
crate), the schema/interpreter calling the frozen constructors + Step-3 template boundary-eval, and
file-level parse-parity (byte-identity vs `crew_state.json` + Rust≡Python run + a canonical
structural graph dump); **parameter packs deferred in the Rust port**. Step 5 (Godot loads a file)
is unblocked. Decision D (the bounded-closed-grammar reading) was **CONFIRMED
by the user** (2026-07-13) and Step 2 landed: the pure-stdlib expression VM + `DeclarativeFlow`
entered `simcore` as the deliberate one-time additive break, proven faithful by re-expressing the
frozen `SelfDischarge` bit-identically (Euler + RK4). Step 2 shipped only the unambiguous arithmetic
core; division, the function set, and a named-constant surface are deferred until a real flow forces
each (see the Step 2 entry). **Step 3 (templates)** added `parameters` + boundary-time
parameter-expressions (`param('crew_count') * base`) that lower to literals at interpret time — the
"one template, many habitats" knob, anchored by re-expressing the frozen Crew as a `crew_count`
template (byte-identical at 1.0, 4× at 4.0); it **deliberately amends decision A** (the boundary now
does `+ − ×`, a new Step-4 cross-port surface). Plan of record for turning the frozen
multi-domain engine into an **authoring platform**: new stations, ecosystems, species, and
domains are declared in **data**, not programmed. Pre-plan orientation complete and
advisor-reviewed; two load-bearing scope decisions **USER-CONFIRMED** (see "Confirmed scope
decisions"), and one sub-fork inside them **flagged NEEDS-CONFIRM** (the kinetics-DSL shape —
see decision D). Steps 0–2 are designed concretely here; Steps 3–7 get just-in-time design as
each prior step nails the constraints (the Phase-5/6/7/8 discipline).

The roadmap's charge (lines 374–end):

> **Goal:** *Turn the engine from a model into a platform.*
> **Create:** declarative scenario files · parameter packs · species definitions · domain
> definitions · station and ecosystem templates.
> Scenarios now span domains: a scenario can define a habitat with its power budget, thermal
> limits, crew size, and ecosystem — **without touching engine code**.
> **Why:** New stations and ecosystems are **authored, not programmed**. The simulator is not
> really about plants — it is about **closure of matter and energy cycles.**

## Goal

Ship a declarative authoring layer so a scenario file can define a whole habitat — power
budget, thermal limits, crew size, and ecosystem — that both the **Python reference** and the
**frozen Rust core** build and run, and that the **shipped Godot game loads at runtime**. The
engine still computes all domain logic; authoring only *selects, parametrizes, wires, and
(within a bounded grammar) shapes the kinetics of* the frozen primitives.

## Confirmed scope decisions (USER-CONFIRMED)

1. **Scope = composition + parametrization + bounded new kinetics (roadmap reading "B").** A
   scenario file may not only pick existing frozen `Flow` types, parametrize them (params /
   param-packs), set ICs, and wire which shared stock each points at (that is the composition
   subset, "reading A") — it may **also author new kinetics** for a flow via a declarative
   expression. A *species* = a flow set + a param-pack + a stock template; a *domain* = a named
   bundle of stock + flow definitions over a quantity set. (User chose the more ambitious
   reading over the pure-composition one the advisor recommended; the design below is what
   makes it coexist with the non-negotiable invariants — see decision D.)

2. **The frozen Rust core parses scenario files at runtime.** The Rust *boundary* layer gains a
   real scenario-file parser, so the shipped game (and modders) load new station/scenario/
   species/domain files at runtime — the "author, not program" payoff. Consequence: a new Rust
   parse dependency **and** a new cross-port **parse-parity** surface (Rust-parse(file) ≡
   Python-parse(file) ≡ same graph). Chosen over the Phase-7 Option-C "author in Python, lower
   to a hex-float bundle Rust reads" precedent, which cannot load arbitrary player/modder files.

## The load-bearing decisions (the things the plan must nail)

### A. Authoring is boundary code; per-step evaluation is a pure-stdlib engine primitive.

The scenario→(`State`, `Registry`, resolver) **interpreter** lives in the boundary layer (like
`config/` / `sim_io/`): it reads YAML, validates a pydantic schema, and calls the **existing
frozen constructors**. It does **no float math**, so the engine stays pure and the parity risk
is purely *structural* (same graph, same ids, same param values, same reduction op-order). The
one thing that must run *inside* the frozen engine — the per-step evaluation of an authored
kinetics expression — is handled by a small **expression VM over a plain-data AST** (decision
D). The AST is plain data (stdlib only), so the VM is a **stdlib addition to `simcore`**: the
first deliberate core extension since the biosphere/station freeze, added **once** like a new
integrator, never per-scenario. Parsing YAML→AST stays in the boundary; only AST→f64 evaluation
is core.

### B. The safety spine (the legality argument — inherited from Phase 8, amended for kinetics).

- **Authored artifacts are runtime objects, never frozen, never reference.** The 20-golden set
  and both freeze manifests (`docs/biosphere-reference.*`, `docs/station-reference.*`) **do not
  grow**. Phase 9 authors no new *reference* science; it re-composes and re-shapes frozen
  primitives at runtime.
- **Conservation makes composition safe** (a bad wiring surfaces as a runtime
  `ConservationError`, never a silent fix) — the Phase-8 "composed station is a runtime object"
  argument, one level up.
- **Conservation does NOT make new kinetics scientifically valid** (a conservation-closed
  expression can still be physical nonsense). So authored kinetics carry an explicit
  **"authored ≠ validated"** status: no calibration claim, no golden, no manifest entry, and a
  surfaced-as-such flag in the display projection. Scientific validity remains a human authoring
  responsibility; the engine only guarantees *conservation* and *determinism*.

### C. Balance-by-construction for authored flows (why novel kinetics still can't break closure).

A declarative flow is authored as a **rate expression × a fixed stoichiometry vector**, never as
raw per-stock deltas. The interpreter emits `rate × coeff` for each leg, so `Σ legs = 0` per
quantity **structurally** — exactly the way every frozen flow is balanced (the "flows return
structured per-stock legs, arbitration scales the whole flow" invariant). The author controls
only the scalar *rate* (an expression over stocks/forcings/params) and the integer/rational
stoichiometric coefficients; they *cannot* express an unbalanced flow. Conservation-every-step
is then a redundant backstop, not the primary guard — the grammar makes imbalance
inexpressible.

### D. The kinetics DSL is a bounded, closed, deterministic grammar — **NOT** an arbitrary evaluator. **(USER-CONFIRMED 2026-07-13; Step 2 built on it.)**

This is the decision that lets USER-CONFIRMED scope (1) coexist with the non-negotiables. The
expression grammar is a **fixed, finite, closed set** of primitives:

- arithmetic `+ − × ÷`, unary `−`;
- a closed function set: `exp ln pow sqrt min max clamp abs` + saturation/`monod(x,k)` +
  bounded conditionals (`ifpos`, piecewise) — chosen to cover the frozen flows' own math
  (FvCB `exp`/`sqrt`, Stefan-Boltzmann `pow`, first-order `k·x`, demand `k·(setpoint−x)`);
- reads: stock amounts (by id), forcings (by name), params (by name), `dt`, `n`;
- **no** user-defined functions, recursion, loops, I/O, or unbounded operators.

Both Python and Rust implement the **same AST evaluator with the same op-order** ⇒ bit-exact
within a build, and cross-port tolerance-gated exactly like every other transcendental (Phase-7
3-tier contract: Tier-1 bit-exact for a transcendental-free authored flow, Tier-2 measured band
for one that touches `exp`/`pow`/etc.). The VM is a *mechanical* port, like the integrator —
so "core stays pure stdlib / the Rust port stays mechanical / determinism is bit-identical
within a build" all survive. **An arbitrary code/expression evaluator is explicitly rejected** —
it would break all three at once and defeat the freeze.

> **NEEDS-CONFIRM:** the bounded-closed-grammar reading of scope (1). If the user actually wants
> unbounded authored math (a general expression language / plugin code), that is a different,
> much larger project with no bit-exact-parity or purity guarantee, and this plan does not cover
> it. The steps below assume the bounded grammar.

### E. The cross-port parse-parity surface (new in this phase).

Runtime Rust parsing (decision 2) adds a surface Phase 7 deliberately avoided: **Rust must parse
the same file into the same graph as Python.** Contract:

- **Tier-0 structural exact:** same stock-id set, same flow-id set + priorities, same AST
  structure, same param values (decimal → f64 round-trips identically across correct-rounding
  parsers), same reduction op-order.
- **Tier-1 / Tier-2 trajectory:** as decision D, reusing the `tests/crossport/` harness +
  `tiers.json` discipline.
- **The Rust YAML dependency is a real choice** (Phase 7 chose Option-C hex-floats *precisely to
  avoid* a Rust YAML dep; runtime-parse now forces one). Decide in Step 4 between a vetted YAML
  crate and a hand-rolled closed-subset parser, and confirm the YAML-1.1 `1.0e7`-is-a-string
  and decimal-rounding hazards the Phase-7 notes raised are handled (canonical numeric spelling
  in the schema).

## Non-goals (explicit scope fence)

- **No new *reference* science, no calibration, no new *frozen* domains/flows/scenarios.** The
  Python reference + Rust port stay frozen; authored artifacts are runtime-only (decision B). A
  surfaced discrepancy in a frozen primitive is an **unfreeze-discipline finding**, never a
  silent authoring-side "fix."
- **No unbounded / plugin kinetics** (decision D) — the grammar is closed and finite.
- **No netcode / server infrastructure / WASM** (still Phase-8's fence; runtime-parse is a
  local file-load, not a network protocol).
- **No authoring UI editor** beyond "load a file" — Godot *loads and runs* authored files
  (Step 5); a graphical scenario *editor* is out of scope (a later phase if wanted).

## Step spine (concrete-first; JIT the rest — this project's rhythm)

- **Step 0 — the scenario-file format + Python interpreter (composition subset only). COMPLETE.**
  New additive boundary package `src/authoring/` (`schema.py` pydantic scenario schema · `flow_registry.py`
  the author-selectable frozen-flow surface + named param loaders · `interpreter.py` lowers a validated
  `ScenarioSpec` → `(State, Registry, resolver)` by calling the frozen constructors, no float math ·
  `run.py` single-rate/no-reset harness · `errors.py`). **Anchor = the frozen standalone Crew, chosen
  because it has ZERO derived ICs** (every value is plain scenario data): `tests/authoring/scenarios/
  crew_mission.yaml` re-expresses `MISSION_SCENARIO`. Proven three ways, strongest first (advisor):
  (1) **structural equality** — `interpret(...).state == build_crew(...)[0]` and
  `registry.flows == expected.flows` (frozen flow dataclasses incl. `params: CrewParams` are equatable),
  the failure-localizing primary gate; (2) **loaded values == frozen `CrewScenario`** — the pyyaml
  YAML-1.1 numeric-string guard (dotless `1e-3` parses as a *string*; every float carries a decimal
  point + a pydantic-`float` coercion backstop); (3) **byte-identity** — the interpreted 168-step Euler
  run reproduces `crew_state.json` byte-for-byte (no new golden; the frozen golden is the oracle).
  Plus the **decision-B safety teeth**: `crew_broken_wiring.yaml` (a CARBON withdrawal wired at an
  OXYGEN stock) interprets cleanly then raises `ConservationError` on step 1 — bad wiring surfaced,
  never silently fixed. **The flow-type registry is explicit, not introspected** (a `StockId` is a
  `str` alias — field-type introspection can't tell a wiring field from any str field; and the registry
  *is* the author-selectable contract Step 7 freezes). Params reuse the frozen `load_crew_params` loader
  by name ⇒ the param contribution is byte-identical *trivially*; what byte-identity genuinely tests is
  wiring + ICs + forcings (inline/override packs are Step 1). 7 tests. **Zero core + zero domain change**
  (`git diff src/{simcore,domains}/` empty — purely additive boundary + tests + YAML); the VM entering
  `simcore` is Step 2's deliberate one-time break, not now. All 20 frozen goldens byte-identical.
  > **Derived ICs surfaced & deferred (advisor).** The plan's original literal `station.yaml` anchor is
  > NOT declaratively expressible — `station`'s `node0` = f(Power's mean dissipation), the greenhouse's
  > `chamber_co2_mol0` = f(crew fractions), etc. Crew is the Step-0 anchor *because* it has none. Before
  > a coupled scenario (station/greenhouse) can be authored, the **derived-IC mechanism** must be decided:
  > precompute-and-inline (author lowers the derived value to a literal) vs letting it bleed into the DSL
  > (a computed-IC expression). That is a later sub-step (likely Step 3 templates), NOT Step 0.
- **Step 1 — parameter packs (data-only). COMPLETE.** A **parameter pack is a param file the
  *frozen* loader reads** (`load_crew_params(path=…)`) — the tightest constraint is "don't bypass
  the frozen bounds/exact-unit/pydantic validation," and a pack-is-a-param-file satisfies it for
  free (advisor). `schema.FlowSpec.params` now accepts a bare **string** (the flow type's frozen
  default set — the Step-0 form) OR a `ParamPackRef` (`{pack: <path>}`); the loader is tied to the
  flow type via `FlowTypeSpec.param_set` (a fixed fact of the class), and `flow_registry.load_param_set`
  dispatches default-vs-pack. **Pack paths resolve relative to the scenario file's directory**
  (`interpret(spec, base_dir=…)`, threaded from `load_scenario`) so a modder scenario+pack bundle
  relocates intact (Step-5 forward-fit). **Step-0's byte-identity gate doesn't transfer** (a changed
  pack diverges by design) — split into (a) a **no-op pack** (restates the frozen 0.949/0.675, only
  `source:` differs — recorded-not-parsed) reproducing `crew_state.json` **byte-for-byte** (the
  faithfulness gate) + (b) a **changed "cultivar" pack** (`respired_carbon_fraction` 0.949→0.80) whose
  effect is *reconstructed*: less `exhaled_co2`, more `fecal_waste` in the exact 0.80/0.20 split,
  `food_store` depletion + the whole WATER/OXYGEN side **bit-identical** (the forced intake `q` is
  independent of the split), CARBON conserved every step — the `n_limited`/`water_biting`
  reconstruct-the-factor "it bit" gate. Plus a **bad-bound pack** (`resp` 1.5) → `ValueError` from the
  frozen loader (packs reuse the guards, never route around them). **Full-file packs only** — no
  partial-merge (would merge at the YAML-dict level *before* the loader, never `dataclasses.replace`
  after), no multi-domain bundling (earns its keep at templates/species) — both deferred; inline params
  (values in the scenario file) also deferred (needs a from-dict loader surface `_CrewSchema` doesn't
  expose). 5 tests. **Zero core + zero domain change** (`git diff src/{simcore,domains}/` empty). Full
  suite green; all 20 frozen goldens byte-identical.
- **Step 2 — the bounded kinetics DSL (decision D), Python side. COMPLETE
  (decision D CONFIRMED by the user; the deliberate one-time additive `simcore` break landed).**
  New additive `src/simcore/expr.py` — the plain-data AST (`Const`/`StockRef`/`ParamRef`/
  `ForcingRef`/`StepN`/`Neg`/`BinOp`), a pure-stdlib `eval_expr`, and `DeclarativeFlow`
  (rate-expr × stoichiometry, balanced by construction — decision C). The **only** `simcore`
  change is this new file (`git diff --stat src/simcore/` empty; the "zero core change since
  freeze" streak broken **once, on purpose**, like adding an integrator). The Python parser is
  `src/authoring/expr_parser.py` (a tiny recursive-descent infix parser, explicitly-pinned
  precedence/associativity — the sole Tier-0 parse-parity surface Step 4 mirrors); `schema.py`
  gains `KineticsSpec` + a `type`-xor-`kinetics` `FlowSpec` union; the interpreter builds the
  `DeclarativeFlow`, validating referential integrity (`param`/`stock` reads) and
  **balance-by-construction** against the stock compositions (relative tolerance, mirroring
  `assert_flow_balanced` — exact for integer coeffs, tolerance-backed for fractional splits).
  **Acceptance = re-express the frozen `SelfDischarge` (`k·battery`, transcendental-free ⇒
  Tier-1)** as a `kinetics` YAML flow (`self_discharge_dsl.yaml`, reusing the frozen
  `self_discharge` param set so `k` is the frozen value) and prove the interpreted run is
  **bit-identical to the frozen constructor's trajectory, per step, under BOTH Euler and RK4**
  (RK4 nontrivial — `SelfDischarge` is donor-controlled ⇒ RK4 ≢ Euler); the frozen flow is the
  oracle (no new golden). 39 tests (VM op-for-op + `DeclarativeFlow` + parser + the safety spine
  + the anchor). **Advisor-driven scope calls (locked so Step 7's freeze inherits them):**
  (1) **no `dt` token in the rate grammar** — the rate is the instantaneous (dt-independent)
  rate and `DeclarativeFlow` supplies the single `× dt`, so RK4-order-safety is *structural*, not
  a documented hope; `n` stays (dt-independent); (2) **ship only the unambiguous arithmetic core
  (`+ − ×`, unary `−`, refs) + defer** division (`/0` is a Python-raise-vs-Rust-`inf` cross-port
  choice) **and the whole function set** (`exp ln pow sqrt abs min max clamp monod` + bounded
  conditionals — `monod`/`clamp`/`ifpos` each carry a real semantic choice) until a real flow
  forces the definition (the `_DemoSchema` "bespoke until a second instance" discipline, applied
  to the grammar); (3) a faithful **transcendental** anchor additionally needs a **named-constant**
  surface (Stefan-Boltzmann σ is a CODATA *module constant*, not a param), unresolved here —
  `SelfDischarge` dodges it; both (2)/(3) are flagged so Step 4's Rust port does not assume the
  grammar is complete. `BuiltScenario.has_authored_kinetics` is the "authored ≠ validated" marker
  (decision B; display-surfacing is a later step). **Zero domain change**; full suite (incl.
  `-m slow`) + ruff + pyright green; all 20 frozen goldens byte-identical (no regen). The
  load-bearing "the VM is faithful" proof, and the (B) analogue of Step 0's anchor.
- **Step 3 — templates. COMPLETE.** Parametrized scenario files (a habitat template
  instantiated with a **crew size**, the roadmap's own example) on top of Steps 0–2. New
  additive `src/authoring/template.py` + a `parameters:` block on `ScenarioSpec` + a
  `float | str` widening of the two numeric fields (`StockSpec.amount`, `ForcingSpec.const`)
  + an `overrides=` arg threaded through `interpret`/`load_scenario`. **The mechanism:** a
  template declares named scalar `parameters` (with defaults); a stock `amount` / forcing
  `const` may be a **bounded-grammar expression** over them (`param('crew_count') * 1000.0`);
  an instantiation supplies `overrides`, and the interpreter **evaluates those expressions
  to literals at build time** (`resolve_parameters` + `eval_numeric_field`), which the frozen
  constructors then receive unchanged. **This deliberately amends decision A** (Steps 0–2 did
  *no* boundary float math ⇒ "parity is purely structural"): boundary-eval `+ − ×` is now a
  **new cross-port surface** Step 4's Rust interpreter must match — benign (IEEE-deterministic,
  decimals round-trip), stated not slipped, load-bearing for Step 7's freeze. **The grammar is
  reused, the context is not** (advisor #5): template expressions parse with the *same*
  `expr_parser` (so Step 4 mirrors one parser, precedence pinned in one place) but evaluate at
  build time where no `State`/`env`/`n` exists — only `Const`/`ParamRef`/`Neg`/`BinOp` are
  legal, `param('…')` resolves against the **template-parameter** namespace (a documented
  *overload* of the kinetics-DSL `param`, disjoint context), and a `stock`/`forcing`/`n` ref is
  an `AuthoringError`; the `+ − ×` op-order mirrors `simcore.expr.eval_expr` so a boundary
  literal is bit-identical to what the engine VM would compute (one op-order, both ports).
  **Anchor = the frozen Crew re-expressed as a template** (`crew_habitat_template.yaml`,
  parametrized by `crew_count`; `crew_mission.yaml` untouched so Steps 0/1 keep their goldens):
  (1) **faithfulness → byte-identity** — at `crew_count = 1.0` (default AND explicit override)
  `1.0 * base == base` **exactly** ⇒ reproduces `crew_state.json` byte-for-byte (the whole
  template→resolve→evaluate→interpret→run path validated against the frozen golden); (2) **the
  knob is load-bearing ("it bit")** — at `crew_count = 4.0` every stock ≈ 4× (reconstruct to
  `rel_tol=1e-12` — accumulate-then-scale ≠ scale-then-accumulate in fp, the
  `n_limited`/`water_biting` discipline), CARBON/WATER/OXYGEN conserved every step,
  `rationed == 0`/`events == ()`, and the final state is **not** byte-identical to the golden
  (a non-scaling knob could not move it — the Step-1 changed-pack analogue). 12 tests (2
  byte-identity + 3 the-4×-gate + 7 referential-integrity/unit). **Zero core + zero domain
  change** (`git diff src/{simcore,domains}/` empty — purely additive boundary + tests + YAML);
  all 20 frozen goldens byte-identical (no regen).
  > **The derived-IC fork is resolved *in principle*, not *handled* (advisor #3).** The
  > mechanism *is* the fork's resolution: a computed expression, evaluated in the **boundary**,
  > lowered to a literal before the engine (a hybrid of the plan's "precompute-and-inline" vs
  > "DSL bleed" fork — DSL grammar, boundary timing). But the `crew_count` anchor needs only
  > template params + multiplication. Making the greenhouse `chamber_co2_mol0 =
  > f_resp·food_intake/k_scrub` expressible needs **two** still-deferred things: **division**
  > (boundary-only-cheap — a template `/0` is a build-time `AuthoringError`, deterministic in
  > both ports; it does NOT reopen the engine-VM `/0` choice Step 2 deferred) **and reading a
  > loaded flow-param into the template namespace** (`f_resp` from `crew.yaml`). Both are the
  > flagged **next increment**. The **simulation-derived** tier (station `node0` = f(run Power,
  > take mean)) needs running a sub-sim — genuinely larger, deferred further.
  > **File composition/includes** (merging fragments) is out of Step 3's parametrization scope —
  > deferred to Step 6 (species/domain bundles).
- **Step 4 — the Rust interpreter + parse-parity.** SPLIT (advisor-recommended,
  USER-CONFIRMED) into **4a — the crux (no YAML dep)** and **4b — the file parser
  (decision E, folds into Step 5)**. The genuinely-new cross-port surface of this phase —
  rate-grammar parse-parity + authored-flow trajectory parity — is provable with **zero
  YAML dependency**: the trajectory anchor builds a `DeclarativeFlow` in Rust directly
  from a *parsed rate string* (the `gen_engine_vectors` inline-flow discipline), and
  parse-parity tests rate *strings*, not files. So the heaviest, most precedent-loaded
  decision (E, the Rust YAML dep — Phase-7 chose Option-C hex-floats *precisely to avoid*
  one, and `serde_yaml` is deprecated) is separated out and does not gate the crux.
  - **Step 4a — the VM + parser port + the two parity surfaces. COMPLETE.** Two new Rust
    modules, **zero YAML dep**: (1) `rust/crates/simcore/src/expr.rs` — the mechanical
    mirror of the Phase-9 Step-2 `simcore.expr` extension (the `Expr` enum
    `Const`/`StockRef`/`ParamRef`/`ForcingRef`/`StepN`/`Neg`/`BinOp`, `eval_expr`,
    `DeclarativeFlow` implementing the `Flow` trait — `increment = rate*dt`, `coeff*increment`
    per leg, op-order char-for-char); (2) a new boundary crate `rust/crates/authoring`
    (depends only on `simcore`) with `expr_parser.rs` (the recursive-descent parser, the
    **sole Tier-0 parse-parity surface** — precedence/associativity pinned identically to
    the Python `authoring.expr_parser`) + `sexpr.rs` (the canonical S-expr renderer the
    parity gate diffs). **The deferred grammar is deferred in both ports (do NOT complete
    it):** the `BinaryOp` enum literally cannot represent `/`, and `/`/unknown-idents/`**`
    are *rejected* exactly as Python raises `AuthoringError` — the "helpful Rust dev adds
    all the ops" trap structurally foreclosed. **Two parity surfaces**, both against
    committed generated-vector files (the `gen_engine_vectors` discipline — `src/authoring`
    + `simcore.expr` ARE the reference, no external anchor; `test_crossport.py` guards
    in-sync): (i) **parse-parity** (Tier-0 structural) — 20 accept cases (every node type +
    precedence/associativity + exact-literal round-trip: `Const` renders through the
    hex-float codec so a literal's parity is *bit-exact*) each lowering to the identical
    canonical S-expr in both ports, + 16 reject cases (both ports must error; **message
    text NOT pinned** — Tier-0 is accept→same-AST, reject→both-error); (ii)
    **trajectory-parity** (Tier-1 bit-exact) — the frozen `SelfDischarge` re-expression
    anchor (`k·battery`, the plan's Step-2 anchor, **no new golden** — and it gives two
    free checks: Rust-DSL == Python-DSL AND Rust-DSL == the already-ported Rust
    `SelfDischarge`) + a **synthetic authored scenario** additionally exercising every
    remaining VM node in the *evaluated* path (`ForcingRef`/`StepN`/`Neg`/`+`/`-`, which the
    Mul-only SelfDischarge misses — the "synthetic scenario" engine-vectors discipline),
    both run under **Euler AND RK4** (RK4 nontrivial — donor-controlled ⇒ RK4 ≢ Euler) and
    gated per-step bit-exact via the hex-float codec (`rationed==0`/`events==()` asserted).
    `simcore` gains 17 expr unit tests; `authoring` 14 parser + 2 vector-gated integration
    tests. **Zero core + zero domain change** (`git diff src/` empty — the whole crux is
    under `rust/` + `tests/crossport/`, and `simcore.expr` was already extended in Step 2);
    `cargo test` + `clippy -D warnings` green; full Python suite incl. `-m slow` + ruff +
    pyright green; all 20 frozen goldens byte-identical (no regen). The template
    **boundary-eval** surface (Step-3's `param('crew_count')*base`) is *interpreter* scope,
    so it defers with 4b, not the crux.
  - **Step 4b — the Rust YAML/schema/interpreter (decision E). COMPLETE.** Runtime
    scenario-**file** parsing landed entirely under `rust/crates/authoring/` +
    `tests/crossport/` (`git diff src/` empty; the one `rust/crates/domains/src/crew.rs`
    touch is two additive `pub fn new` constructors for `OxygenConsumption`/`FoodMetabolism`
    — the Phase-8 "re-pointed sibling flows got `pub fn new`" idiom, under `rust/`, golden
    unaffected). **Decision E resolved USER-CONFIRMED = hand-rolled closed-subset YAML
    parser** over a vetted crate (`serde_yaml` is deprecated, but the load-bearing reason is
    the **parse-parity boundary**: a crate forces reconciling *two* independent YAML-1.1
    impls against pyyaml — the `1.0e7`-is-a-string hazard, decimal edges — whereas a
    documented closed subset collapses that to **one grammar owned on both ports**, the
    Option-C/hex-float/sexpr ethos). Modules: `yaml.rs` (indent-based reader over the
    documented subset — block maps, `- ` list-of-maps, quoted/bare scalars, `#` comments;
    the YAML-1.1 numeric rule — a float needs a `.` and a signed exponent — pinned in
    `is_yaml_number`, so a dotless `1e-3` is a *string* like pyyaml; anchors/flow-style/
    tags/multiline **excluded and rejected**, not silently mis-parsed), `schema.rs`
    (`ScenarioSpec` mirror + `extra="forbid"` + the `type`-xor-`kinetics` validation +
    the `number|expr` union typed via bare-vs-quoted), `flow_registry.rs` (the three crew
    types → frozen constructors + named param sets from the Option-C constants),
    `template.rs` (Step-3 build-time `+ − ×` boundary-eval, op-order mirroring
    `simcore.expr`), `interpreter.rs` (`build_stock`/`build_flow`/balance-by-construction +
    referential-integrity checks/`load_scenario`), `run.rs` (euler/rk4 final-state),
    `graph_dump.rs` (the canonical structural dump). **The reframe that shrank it
    (advisor): the crew engine is already Tier-1 bit-exact in Rust (Phase-7), and the
    interpreter does no trajectory math — so `crew_mission → crew_state.json` byte-identity
    is near-automatic once the Rust interpreter builds the same graph; literal parity rides
    on `f64::from_str` being correctly-rounded (since 1.55, like Python `float()`).**
    **Parameter packs are DEFERRED in the Rust port** (advisor-endorsed scope call): all
    three anchors use *named default* sets (`params: crew`/`self_discharge`) mapped to the
    existing Option-C constants; a `{pack: …}` reference is a clean interpret-time error
    (a Rust pack reader would re-run the frozen bounds/unit validation on an arbitrary
    file, which no anchor exercises and Step 5 does not need) — this bounds decision E's
    "param packs" parity line. A **non-zero priority on a frozen flow type** is likewise
    rejected loudly (the frozen crew constructors carry no priority field, shared with the
    station callers; no anchor uses one; a `kinetics` flow honors priority fully).
    **Consequence (advisor): reject-parity is NOT a cross-port surface in 4b** (unlike
    4a's reject→both-error rate strings) — because the Rust port deliberately rejects a
    *superset* of what Python accepts (a `{pack: …}` ref + a non-zero frozen-flow priority
    both *succeed* in Python, *error* in Rust), the Rust reject tests prove "the
    interpreter never silently mis-builds," and accept→same-graph is the whole parity
    story, carried by the byte-identity/run-match/graph-dump gates. **The
    file-level parse-parity gates (advisor: the `.yaml` IS the shared artifact — no vector
    file):** (1) **byte-identity** — the Rust interpreter's run of `crew_mission` and the
    `crew_habitat_template` at its default (`crew_count=1.0` ⇒ `1.0*base==base` exactly)
    reproduces the FROZEN `crew_state.json` byte-for-byte (transcendental-free ⇒ Tier-1,
    platform-independent); (2) **Rust run ≡ Python run** for every anchor incl. `@4.0` +
    `self_discharge_dsl` (Rust-parse ≡ Python-parse ≡ same trajectory, via `sim_io`
    round-trip); (3) **structural graph-dump parity** — a canonical dump (id-sorted stocks
    with hex-float amounts, flow ids + **priorities**, forcing constants) rendered
    *identically* by both ports (the fact a final-state snapshot is blind to — priorities,
    present-but-inert flows, the bit-exact Step-3 boundary-eval of `param('crew_count')*const`
    on the `@4.0` case). All three run on the crossport CI job (`skipif cargo`), Tier-1 so
    glibc-vs-UCRT-golden holds. **Tests:** 12 Rust unit (`yaml.rs`) + 10 Rust integration
    (`scenario_files.rs`, incl. the reject cases — unknown type, type-xor-kinetics,
    unbalanced stoichiometry, unknown-stock ref, frozen-priority + pack deferrals) + 12
    Python crossport (4 graph-dump + 4 run-matches + 2 golden + the 2 in-sync 4a).
    **Zero core + zero frozen-golden change** (`git diff src/` empty; 20 goldens
    byte-identical, no regen); `cargo test` (workspace) + `clippy --all-targets` +
    ruff/format/pyright green. Blocks Step 5 (Godot loads a file), not 4a.
- **Step 5 — Godot loads a file at runtime (JIT).** The `godot_bridge` gains a "build session
  from a scenario file path" entry; the "author, not program" payoff. Cross-boundary parity
  smoke: the file-loaded session (through the actual cdylib) == the headless-built one, the
  Phase-8 FTZ/DAZ discipline.
- **Step 6 — species / domain definitions (JIT).** A *species* = flow-set + param-pack + stock
  template; a *domain* = a named stock+flow bundle over a quantity set — authored bundles built
  on Steps 2/4, composing into scenarios.
- **Step 7 — the format/grammar freeze doc (JIT).** The Phase-4/6/8 freeze-contract analogue,
  one level up: freeze the **DSL grammar + file schema + the VM** (so mods authored against them
  stay stable), *not* new goldens. `docs/authoring-reference.md` + completeness gate on the
  grammar/schema surface.

## Open questions / risks to resolve in review

- **The kinetics-DSL sub-fork (decision D) is RESOLVED** — USER-CONFIRMED (2026-07-13) as the
  bounded closed grammar; Step 2 built the VM on it.
- **The deferred grammar surface (Step 2 scope calls).** Division (`/0` cross-port choice), the
  closed function set (`exp ln pow sqrt abs min max clamp monod` + bounded conditionals — each
  ambiguous op needs a real flow to fix its definition), and a **named-constant** surface (σ for a
  Stefan-Boltzmann re-expression) are all deferred; Step 4 (Rust port) must NOT assume the grammar
  is complete, and each op joins when a real frozen flow forces its semantics.
- **Rust YAML dependency** (decision E) — crate vs hand-rolled subset; the deprecated-`serde_yaml`
  + YAML-1.1 numeric hazards from the Phase-7 notes.
- **Where the VM lives** — a stdlib `simcore` addition breaks the "zero core change since freeze"
  streak *deliberately and once*; confirm that framing (a one-time engine primitive, not
  per-scenario code) is acceptable, and that it enters the freeze manifest as a frozen VM.
- **"Authored ≠ validated" surfacing** — how prominently the display / CLI marks an
  authored-kinetics run as uncalibrated (decision B).
- **Scientific-validity ownership** — with (B), conservation-closed nonsense is authorable; the
  plan guarantees closure + determinism only. Confirm that is the intended contract.
