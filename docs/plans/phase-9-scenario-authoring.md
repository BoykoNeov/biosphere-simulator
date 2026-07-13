# Phase 9 — Scenario Authoring & Modding (the model becomes a platform)

**Status: DRAFT — PLANNED, not started.** Plan of record for turning the frozen
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

### D. The kinetics DSL is a bounded, closed, deterministic grammar — **NOT** an arbitrary evaluator. **(NEEDS-CONFIRM sub-fork)**

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

- **Step 0 — the scenario-file format + Python interpreter (composition subset only).** Define
  the YAML schema (pydantic) for the *composition* reading (existing flow types + params/
  param-pack refs + ICs + wiring), and the Python interpreter that builds
  `(State, Registry, resolver)` from it by calling the frozen constructors. **Acceptance = the
  byte-identity anchor:** a hand-written `station.yaml` re-expressing an existing frozen scenario
  reproduces its golden **byte-for-byte** (`declarative(station.yaml) == build_station ==
  golden`) — the interpreter proven faithful, needing no new golden. (No Python composition/
  palette analogue exists today — Phase-8 built `assemble`/`build_scenario` only in Rust — so
  Step 0 *creates* the Python composition layer. De-risks the whole format before any DSL.)
- **Step 1 — parameter packs (data-only).** A param-pack file that bundles/overrides existing
  YAML params, referenced by a scenario file. Nearly free — a subset of the Step-0 format;
  proves the "cultivar = param pack" species primitive cheaply.
- **Step 2 — the bounded kinetics DSL (decision D), Python side.** The closed expression grammar
  + the pure-stdlib AST/VM in `simcore` + a `DeclarativeFlow` (rate-expr × stoichiometry,
  balanced by construction — decision C) + the Python parser in the boundary. **Acceptance =
  re-express ONE frozen flow as a DSL flow** (e.g. `SelfDischarge = k·battery`,
  transcendental-free ⇒ Tier-1) and prove the DSL run is **bit-identical** to the frozen
  constructor's trajectory. The load-bearing "the VM is faithful" proof, and the (B) analogue of
  Step 0's anchor.
- **Step 3 — templates (JIT).** Parametrized / partial / composable scenario files (a habitat
  template instantiated with a power budget + crew size) on top of Steps 0–2.
- **Step 4 — the Rust interpreter + parse-parity (JIT).** Rust YAML parser (boundary, decision
  E) + the AST/VM ported to Rust `simcore` + parse-parity (Tier-0) + cross-port trajectory
  parity (Tier-1/2). The genuinely-new cross-port surface of this phase.
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

- **The kinetics-DSL sub-fork (decision D) is NEEDS-CONFIRM.** Everything downstream assumes a
  bounded closed grammar; an unbounded evaluator is a different plan.
- **Rust YAML dependency** (decision E) — crate vs hand-rolled subset; the deprecated-`serde_yaml`
  + YAML-1.1 numeric hazards from the Phase-7 notes.
- **Where the VM lives** — a stdlib `simcore` addition breaks the "zero core change since freeze"
  streak *deliberately and once*; confirm that framing (a one-time engine primitive, not
  per-scenario code) is acceptable, and that it enters the freeze manifest as a frozen VM.
- **"Authored ≠ validated" surfacing** — how prominently the display / CLI marks an
  authored-kinetics run as uncalibrated (decision B).
- **Scientific-validity ownership** — with (B), conservation-closed nonsense is authorable; the
  plan guarantees closure + determinism only. Confirm that is the intended contract.
