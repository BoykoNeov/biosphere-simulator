# Post-roadmap Tier 2: the grammar — `monod`

**Status: COMPLETE** (2026-07-16). The outcome block is at the foot of this file — **read
it before the plan body**; the plan's central bet (that a frozen flow already decided
`monod`'s semantics) paid off, but two things it did not anticipate are the useful part.

An **unfreeze of the authoring grammar**, chosen by
the user; sequenced second of three in
[`post-roadmap-flow-registry-growth.md`](post-roadmap-flow-registry-growth.md) ("The
sequence"). This is the **highest-ceremony unfreeze the platform defines**:
`docs/authoring-reference.md`'s discipline mandates advisor review *before writing
anything*, because a grammar op freezes a semantic choice cross-port and freezing the
wrong answer is worse than deferring.

## The charge

Tier 1 expanded what can be **reused**. Tier 2 expands what can be **said**: saturation
becomes expressible. No amount of `+ − ×` approximates it — a saturating law is the one
shape the frozen grammar structurally cannot reach, and it is the single most common
functional form in the science this project models (Michaelis–Menten, Monod, Holling
type II all share the algebra).

## The discipline's precondition, satisfied — a frozen flow forces the definition

The grammar's own rule is that **each deferred op waits on a real frozen flow to force
its semantic definition** ("bespoke until a second instance justifies it", applied to the
grammar). For `monod` that flow exists, and it decides both open questions the Tier-1 doc
left for "then, not now":

**`src/domains/biosphere/chamber.py:90` — `oxygen_limitation_factor`**, frozen since
Phase 2 Step 7, cited to Davidson et al. 2012 (the Michaelis–Menten O₂-limitation form
for respiration):

```python
x_o2 = max(0.0, o2_mol) / air_mol   # ← argument preparation
denom = k_o2 + x_o2
if denom <= 0.0:                    # ← the kernel
    return 0.0                      #    "k_o2 == 0 and O₂ == 0: degenerate; no O₂ ⇒ no respiration"
return x_o2 / denom
```

It is used by three frozen flows (`microbial_respiration`, `herbivory`, and
`carbon_budget`'s maintenance-respiration shortfall). So `monod` is **not a semantics we
invent** — it is one the frozen science already chose and cited. That is the strongest
possible footing for a grammar unfreeze, and it is why Tier 2 is reachable now.

### Question 1 — which form? **2-arg, resolved by the frozen precedent.**

`monod(substrate, half_saturation) = S/(S+K)` — a dimensionless saturation factor, not a
3-arg `Vmax·S/(S+K)`.

The frozen flow settles it: `f_O2` is dimensionless and applied as `daily · f_O2 · dt`, so
`Vmax` arrives through the **already-frozen `*`**. A 3-arg form would freeze an
argument-order choice that buys nothing and cannot be un-chosen. Arg order is
`monod(substrate, half_saturation)`, matching both the frozen signature
(`oxygen_limitation_factor(o2_mol, *, k_o2)`) and Michaelis–Menten convention; it is a
**frozen semantic choice** and is pinned by a parse vector.

### Question 2 — the degenerate case? **`denom <= 0 → 0.0`, resolved by the frozen precedent.**

The Tier-1 doc flagged this as the sharp one: `monod(0,0)` is `0/0` → NaN, the grammar
cannot guarantee `K > 0` because `K` is an arbitrary expression, and so "`x/0` does not
vanish for `monod`; it relocates" — with NaN crossing a hex-float golden contract as the
thing to design against.

**The hazard dissolves rather than needing a design.** The frozen code already made this
exact call, with a documented rationale. Mirroring it makes `monod` a **total function**:
for all finite inputs it returns a finite float — never NaN, never ±inf, never raises. So
there is no NaN to cross the golden contract, and the Python-raise-vs-Rust-`inf` split
never arises because no raw `x/0` is ever reachable.

## What the advisor review changed (mandated by the discipline; recorded here)

The review corrected two of the three positions taken into it. Both are load-bearing:

1. **Separate the two clamps — mirror the kernel ONLY.** The frozen function does two
   distinct things and only one of them is `monod`:
   - `max(0.0, o2_mol)/air_mol` and the `ValueError` guards on `air_mol`/`k_o2`/finiteness
     are **argument preparation** — a depleting physical pool, a degenerate chamber. For an
     authored rate that layer *is the sub-expressions the author composed*.
   - `denom = …; if denom <= 0: return 0.0; return S/denom` is **the kernel**.

   The initial proposal baked `max(0.0, S)` into `monod`. That was **wrong**: a silent
   `max(0,·)` on the substrate would change `monod(stock("a") - stock("b"), k)`
   invisibly — precisely the silent-failure class the Tier-1 outcome doc identified as the
   thing to avoid. Off the natural domain the kernel stays total and produces
   conservation-closed nonsense the author owns, which is the platform's stated philosophy.

   **And do not raise**: the frozen code *returns 0* for the degenerate denom (it only
   raises in the arg-prep layer), and a raise inside `eval_expr` could fire at a transient
   RK4 stage state on a well-posed run.

2. **`monod` is Tier-1 bit-exact, not Tier-2.** The proposal half-assumed the function set
   moves an authored flow Tier-1 → Tier-2 (the reference doc says so). **It does not apply
   here:** division is an IEEE-754 *basic* operation — correctly-rounded and deterministic
   cross-port, exactly like `+ − *`. That rule covers libm calls (`RadiatorReject`'s `T⁴`
   via `pow`); `monod` is not in that class. The traj vector therefore asserts **bit-exact**
   and a failure is a finding, not a band to widen.

   On the natural domain (`S ≥ 0`, `K > 0`) the kernel is **bit-identical** to frozen
   `f_O2` — IEEE `+` is commutative, so `S+K` and the frozen `k_o2+x_o2` agree exactly.

3. **Ship `monod` without `/` — confirmed, and it is the *more* disciplined choice.**
   `monod` guards its own division, so it resolves the `x/0` cross-port question **by
   construction** and never exposes raw `x/0`. Bare `/` as an author-facing op would
   *reintroduce* the exact hazard the deferral protects, and no frozen flow forces it. `/`
   stays in the deferred table; the provenance note must say plainly that `monod` landed
   with `x/0` resolved *internally* while bare `/` remains open.

Review also directed: use the frozen `oxygen_limitation_factor` as a **unit-level oracle**
(Step 5); a **dedicated `Monod` node**, not a generic N-ary function framework (bespoke
until a second function justifies it); and reject-vector the comma's misuse.

**Why `monod` and not `min(k·S, Vmax)`** (one line, because it is the obvious cheap
alternative and it is wrong): `S/(S+K)` is C∞ on the natural domain so RK4's convergence
order survives, whereas a `min` kink is non-differentiable and destroys it — and `rk4` is
a frozen integrator name. The `denom <= 0 → 0` branch is a derivative discontinuity only
at the pathological boundary, which `K > 0` never reaches — same as frozen `f_O2`.

## The freeze-contract question this raises, answered explicitly

The unfreeze discipline's step 2 says: *"`git diff src/simcore/` **must stay empty** —
`simcore/expr.py` was the one sanctioned, one-time addition (decision A); a second core
edit is not an unfreeze, it is a new decision needing its own review."*

Adding a grammar op **necessarily edits `simcore/expr.py`** (the VM must evaluate the new
node). Read literally, that sentence forbids every grammar unfreeze — which contradicts
the same document's *"the grammar is **deliberately incomplete** … it is expected to
grow"* and *"Adding any of them is a **deliberate unfreeze** (grammar node/op sets move ⇒
the completeness gate fires)"*. Both cannot hold.

**The interpretation taken, stated rather than slipped:** the sentence forbids *new,
unsanctioned core mechanisms* — a second module, a new engine concept. Growing `expr.py`'s
node union **is** the sanctioned mechanism for the growth the contract explicitly
anticipates; the "one-time addition" was adding the *module*, not freezing its contents
forever. The manifest's `expr_nodes` set is what makes that growth git-visible and
gate-enforced, which is the whole point of freezing the node set rather than a code hash.

The invariant actually held, which is checkable and is the honest version of the rule:

- `git diff src/simcore/` touches **only `expr.py`**, and only **additively** (no existing
  node or op changes semantics);
- **the 20 frozen goldens stay byte-identical** — the proof the addition is inert for
  everything that does not use it.

## The steps

- **Step 1 — the Python VM.** `simcore/expr.py`: the `Monod` node + the kernel. Additive.
- **Step 2 — the parser.** `authoring/expr_parser.py`: a `,` token (a new grammar-surface
  token), `monod(expr, expr)` with an arity check, and the `render_rate_expr` inverse.
- **Step 3 — the other three Python AST walks.** See the hazard below; this is the step
  most likely to be under-done.
- **Step 4 — the Rust mirror.** `simcore/src/expr.rs` + `authoring/src/{expr_parser,
  sexpr,compose,interpreter,template}.rs`. A Python-only change is a broken contract.
- **Step 5 — the frozen oracle.** A unit test pinning the `monod` kernel bit-exactly
  against `chamber.oxygen_limitation_factor` across its natural domain. This is the
  `SelfDischarge` oracle pattern applied to the *primitive* — evidence that the grammar op
  IS the frozen science, not merely inspired by it.
- **Step 6 — the vectors.** Parse accepts (arg order, nesting, precedence interaction) +
  rejects (arity `monod(a)` / `monod(a,b,c)`, missing comma, bare top-level comma). A
  Tier-1 **bit-exact** traj scenario carrying a `monod` rate under Euler *and* RK4.
- **Step 7 — manifest, docs, gates.** Confirm the completeness gate **fails** before
  regeneration (`expr_nodes` 7 → 8); regenerate; review that the diff shows *only*
  `expr_nodes` + the two vector hashes. Move `monod` out of the deferred table. Full suite
  incl. `-m slow`, ruff, pyright, `cargo test`, `clippy -D warnings`.

## The hazard the plan found before writing (Step 3)

**Two of the four Python AST walks have a permissive fallback, so a new node is silently
mishandled rather than loudly rejected.** Found by reading, not by a failing test:

| walk | fallback | what a missing `Monod` branch does |
|---|---|---|
| `compose._prefix_expr_refs` | `return node` | **silently leaves `monod`'s `stock`/`forcing` refs unprefixed** in a prefixed bundle → wrong stock, or a resolve-time `KeyError` at step 1 |
| `interpreter._collect_refs` | implicit fall-through | **silently skips build-time referential validation** inside a `monod` subtree → an unknown param surfaces as a runtime `KeyError` instead of a clean `AuthoringError` |
| `template._eval` | `raise AuthoringError` | safe, but the **message would lie** ("stock/forcing/n are not available at build time" when the author wrote none of those) |
| `simcore.expr.eval_expr` | `raise TypeError` | safe |

The first is a silent-wrong-answer path. So Step 3 adds the `Monod` branches **and makes
the two permissive walks exhaustive**, so the *next* node addition cannot slip either.

**The cross-port asymmetry worth recording:** Rust's `match` on the `Expr` enum is
exhaustive *by construction* — adding `Expr::Monod` will fail to compile at every site
until handled. Python's isinstance chains have no such guard. The Rust port would have
caught what the Python reference silently permits, which inverts the usual direction of
the parity relationship.

**Templates stay `monod`-free (a deliberate, reversible deferral).** The template surface
is separately frozen ("only `Const`/`ParamRef`/`Neg`/`BinOp` are legal") and no frozen
flow forces a saturating *initial condition*; a rate-only op is also the established
pattern there (`stock`/`forcing`/`n` are already rate-only). So `monod` in a template is a
**precise** `AuthoringError`, not a lying one.

## Exit criteria

- `monod` is parseable, evaluable, prefixable and rejected-where-illegal **on both ports**,
  pinned by parse + traj vectors.
- **The 20 frozen goldens are byte-identical.** The grammar addition touches no science.
- `git diff src/simcore/` touches `expr.py` only, additively; the manifest diff shows
  `expr_nodes` + the vector hashes and nothing else.
- The `monod` kernel is proven **bit-exact against the frozen `f_O2`** it descends from.
- The traj vector asserts **Tier-1 bit-exactness**, not a tolerance band.

## Not in scope

- **Bare `/`.** Stays deferred: `monod` resolves `x/0` internally and no frozen flow forces
  raw division. Recorded in the deferred table with the reason updated.
- **The rest of the function set** (`exp ln pow sqrt abs min max clamp`) and bounded
  conditionals. Each still waits on its own forcing flow; `clamp`/`ifpos` still carry
  unresolved edge semantics, and the transcendentals still need the named-constant surface
  (σ as a module constant) that remains unresolved.
- **A generic function-call framework.** Bespoke `Monod` node until a second function
  justifies the abstraction.
- **`monod` in template expressions.** See above.
- **Authoring a biosphere flow.** Still structurally blocked (the `CarbonContext` reason in
  the Tier-1 doc). `monod` makes the *rate shape* expressible; it does not make the
  biosphere composable. The oracle is therefore taken at the **unit** level, not by
  authoring a frozen biosphere scenario.
- **Recalibrating anything.** Bucket 3.

---

# OUTCOME — COMPLETE (2026-07-16)

All seven steps landed. `monod` is parseable, evaluable, prefixable and
rejected-where-illegal on **both ports**, pinned by parse + traj vectors and by a
bit-exact oracle against the frozen flow it descends from. The exit criteria held: the 20
frozen goldens are byte-identical, `git diff src/domains/` is empty, `git diff
src/simcore/` touches **`expr.py` only** and only additively (the sole non-docstring
deletion is the `Expr = …` union line itself, extended with `| Monod`), and the manifest
diff shows `expr_nodes` 7 → 8 plus the two provenance hashes **and nothing else**.

## The headline result

**The grammar op IS the frozen science, proven bit-exactly — not merely modelled on it.**
`monod(x_O2, K_O2)` reproduces `chamber.oxygen_limitation_factor` bit-for-bit across its
frozen domain (70 cases: 10 O₂ amounts × 7 half-saturations, including the `k = 0`
limit-disabled setting and the frozen `1.5e-4` K_O2 scale). This is the `SelfDischarge`
re-expression pattern lifted from a *flow* to a **grammar primitive**, and it is the
strongest claim a grammar addition can carry: the op is not a plausible saturating
function, it is *the frozen, cited one*, and a drift is a test failure.

What it does **not** prove: that `K_O2 = 1.5e-4` is right. It is one of the 55 uncited
params. `monod` makes saturation *sayable*; bucket 3 still owns whether the numbers are
true.

## What the plan got right (the central bet)

The bet was that the discipline's precondition — *a real frozen flow must force the
semantic definition* — was already satisfied, and that this collapsed both open questions
rather than requiring a design. It held completely:

* **2-arg vs 3-arg** was decided by `f_O2` being dimensionless and applied as
  `daily · f_O2 · dt` — `Vmax` arrives through the frozen `*`.
* **The degenerate case** was decided by the frozen `if denom <= 0.0: return 0.0`. The
  Tier-1 doc's sharpest worry — *"NaN crossing a hex-float golden contract is the thing to
  design against"* — **dissolved**: the node is total, so there is no NaN to cross it. That
  worry was correct *given* its assumption that `monod` would have to invent its edge
  semantics. The assumption was wrong, and the fix was reading the frozen code.

**The transferable lesson:** the Tier-1 outcome generalized that "a frozen flow's safety
argument is scoped to the frozen scenario, and authoring is what escapes that scope." Tier
2 is the **mirror image** — a frozen flow's *semantic* choices are also scoped to it, and
the grammar can **inherit** them instead of re-deciding. Before designing an op's
semantics, grep the frozen tree for the op. The discipline already said this; what was new
is how much it saved.

## What the advisor review changed (the discipline mandates it; it earned its place)

Two of the three positions taken into review were **wrong**, and both would have shipped:

1. **The `max(0.0, S)` clamp.** The plan intended to bake the frozen function's substrate
   clamp into `monod`. The review separated *argument preparation* from *the kernel*: the
   clamp exists because a depleting physical pool leaves float dust, which is the author's
   sub-expression's problem, not the op's. A silent `max(0, ·)` would have made
   `monod(stock("a") - stock("b"), k)` quietly mean something else — shipping precisely
   the silent-failure class Tier 1 identified as the thing to avoid, *inside the op added
   to stop authors guessing*. Pinned by `test_monod_does_not_clamp_its_substrate`.
2. **The tier.** The plan half-assumed `monod` was Tier-2, reading across from the
   reference doc's "a transcendental moves an authored flow Tier-1 → Tier-2". Division is
   an IEEE-754 **basic** operation — correctly-rounded and deterministic cross-port. That
   rule covers *libm* calls (`RadiatorReject`'s `powf`), not `/`. Had this stood, the traj
   vector would have been given a tolerance band, **minting a frozen tolerance for
   arithmetic that is exactly reproducible** — the inverse of Tier-1's trap, which was
   labelling a `powf` flow Tier-1. Both are the same mistake in opposite directions:
   *classify by the ops actually executed*.

A third position — ship `monod` without `/` — was confirmed and sharpened: not a
compromise but the *more* disciplined choice, because `monod` guards its own denominator
and so resolves `x/0` internally while never exposing the raw form.

## Findings the plan did not anticipate

* **The parity harness silently ignored the new scenario.** `traj_vectors.txt` gained the
  `monod` rows and `load_expected()` dutifully read them — but the trajectory test
  **hardcodes** which scenarios it rebuilds, so the new trajectory was loaded, never
  checked, and the suite stayed **green**. *Adding rows to a vector file does not gate
  them.* This is the manifest gate's "added to the tree, exercised by nothing" hole, one
  level down in the *parity* harness — and unlike the manifest's version, nothing
  announced it. Closed by `no_vector_scenario_goes_unchecked`, which asserts set-equality
  between the scenarios in the file and the scenarios the harness rebuilds, so the next
  one cannot be added mutely.
* **Two of the four Python AST walks would have mishandled the new node silently** —
  found by reading before writing, which is why Step 3 existed at all.
  `compose._prefix_expr_refs` ended in `return node`, so a prefixed bundle's
  `monod(stock("o2"), …)` would have kept the **unprefixed** id (wrong stock, or a
  resolve-time `KeyError` at step 1); `interpreter._collect_refs` fell through, silently
  skipping build-time referential validation inside a monod subtree. Both are now
  exhaustive and raise on an unknown node, so the *next* grammar addition cannot slip.
* **Rust caught structurally what Python permits — the parity relationship inverted.**
  `cargo build` named all five `Expr` match sites as `non-exhaustive patterns:
  &Expr::Monod { .. } not covered` and refused to compile; all five were already written
  with explicit arms. Python's isinstance chains have no such guard. **The port would have
  caught what the reference silently allows** — the opposite of the usual direction. It is
  not a reference-authority violation (Rust found nothing *wrong* in Python; it simply
  cannot express the hazard), but it is the first time the port's type system did the
  reference's quality control, and it is why the Python walks are now exhaustive by hand.
* **A dead anchor is trivially bit-exact — found by nearly shipping one.** The
  `monod_dsl.yaml` file anchor (added as the advisor's optional defense-in-depth, then
  kept for a stronger reason: the **comma** is the first rate-grammar character that is
  also YAML-significant) was in its first draft *dimensionally wrong* — `k · monod(…)` is
  `1/s`, not the `J/s` the frozen `k · battery` produces, because the `battery` factor was
  dropped. Its rate was `1.3e-8 J/s` and it drained **1.6e-9** of the battery over 336
  steps. **The crossport suite passed it — 97 green.** Bit-exact cross-port equality of a
  flat line is trivially satisfiable, so the gate that "proves" an anchor says nothing
  about whether the anchor *does* anything. Caught by measuring the trajectory instead of
  trusting the comment describing it (which claimed the run "slides down the curve" —
  false). Now sized so the battery drains 1.0e7 → 1.5e6 and the monod factor genuinely
  traverses 0.667 → 0.226, with `rationed == 0` from the kinetic roll-off. **Generalizes
  past this anchor:** every parity gate in this repo answers "do the ports agree?", never
  "is there anything here to agree about?" — an inert fixture is invisible to all of them.
* **The manifest's `grammar_note` was stale by construction.** It enumerated the deferred
  set as `(exp ln pow sqrt abs min max clamp monod)` — a hand-written string, not a
  derived set, so landing `monod` left the manifest *actively wrong* while every gate
  passed. Updated by hand, which is the honest description: the manifest derives its
  *sets* from live sources but its *prose* is unowned. A future op must remember to edit
  it; nothing will fire if it does not.

## Deliberate scope calls, each reversible

* **Templates stay `monod`-free.** The build-time-legal node set is its own frozen surface,
  no frozen flow forces a saturating *initial condition*, and rate-only is the established
  pattern there (`stock`/`forcing`/`n` already are). Rejected **precisely** rather than by
  the pre-existing generic message, which would have told the author they wrote a
  `stock`/`forcing`/`n` they did not — a lying error.
* **A dedicated `Monod` node, not a function framework.** Bespoke until a second function
  justifies the abstraction; the manifest diff stays one line.
* **The oracle is at the unit level, not a scenario.** The biosphere remains structurally
  un-authorable (the `CarbonContext` reason). `monod` makes the rate *shape* expressible;
  it does not make the biosphere composable, and no attempt was made to force it.

## What this cost, and what it bought

One additive node in the core VM, one token in the grammar, zero science change, zero
param moved, zero golden shifted. The platform can now *say* the most common functional
form in its own domain — and says it in the frozen science's own words.

## The sequence

Tier 1 (reuse) and Tier 2 (expressiveness) are done. **Bucket 3 — validation** (what is
*true*: the deferred Phase-1 quantitative oracle match and the 55 uncited params) is
unchanged and un-started; see `post-roadmap-flow-registry-growth.md`, "The sequence".

Tier 2 sharpens the case for it in a way Tier 1 did not. Tier 1 widened the gap between
"selectable" and "trustworthy" by making more uncalibrated science reachable. Tier 2 makes
the gap **structural**: the very test proving `monod` is bit-exact against the frozen flow
is **silent on whether either is right**. The op is provably the frozen law; the frozen law
is provably nothing yet.

**And a late finding sharpens it further — the shape is reachable, the value is not.**
`monod`'s half-saturation has **no home in any registered param set**. The five loaders
expose `charge_efficiency`, the two crew fractions, the four ECLSS gains,
`self_discharge_rate`, and the four thermal properties — **not one half-saturation among
them**. The frozen `K_O2 = 1.5e-4` that motivated the whole op lives in the biosphere,
whose loaders are unregistered for the same structural reason its flows are. So an author
writing `monod` must supply `K` as a **literal** or repurpose an unrelated frozen constant
(or ship a `pack:`). Tier 2 made the frozen `f_O2`'s *shape* reachable and left its *value*
behind. That is not a bug — it is the calibration gap, now visible in the grammar itself.
