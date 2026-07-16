# Post-roadmap — the first authored habitat (`algae_habitat`)

**Status: COMPLETE (2026-07-16).** The roadmap is complete through Phase 9, its last
phase. This is the first work past it, chosen by the user (2026-07-16) from four
candidates: *use the platform*, close capability gaps, the Phase-1 oracle match, or
harden.

**Deliverables:** `scenarios/algae_habitat.yaml` (the habitat — a new top-level
`scenarios/` tree for authored *content*, distinct from `tests/authoring/scenarios/`
fixtures) + `tests/test_authored_habitat.py` (9 tests). See *Outcome* at the bottom.

## Goal

Author a genuinely new **fully closed** habitat as a scenario file — no engine code, no
Python, no Rust. A crew eats algae and exhales CO₂; the algae photosynthesise that CO₂
back into biomass and O₂; the crew breathes the O₂; a decomposer remineralises the crew's
egested carbon back to CO₂. **Carbon and oxygen both close with zero boundary stocks.**

## Why this, and why now

The roadmap's closing lines are the whole charge:

> New stations and ecosystems are authored, not programmed.
> The simulator is not really about plants. It is about closure of matter — and energy — cycles.

Nine phases built the platform. Nothing has yet *used* it for its stated purpose: every
scenario under `tests/authoring/scenarios/` is a test fixture or cross-port anchor
(`docs/authoring-reference.md` says so explicitly). This is the payoff, and it is
**runtime-only** — no freeze contract moves.

## The constraint that shaped the design (advisor)

> Can every intended rate be written as sums/products of stocks, params, forcings and
> constants (no division, no transcendentals)? If yes → runtime-only, author it, no
> unfreeze. If a flow genuinely needs `x/(x+k)` → stop and surface *that specific op* to
> the user as its own decision; don't absorb it into this task.

**Answer: yes.** Every rate below is a product of non-negative stocks, frozen params,
forcings and constants. `binary_ops` is frozen at `{+, −, ×}`; nothing here needs
division, `monod`, `pow` or a named constant. **This design does not touch the deferred
grammar.** Had it needed to, the correct move was to stop and ask.

But expressible ≠ stable. A second advisor pass caught that the balance arithmetic proves
only conservation, and that the first draft would have **tripped its own gates**. The
numerics below are the fix.

### Three verified facts that shaped the design

1. **`params` is optional** (`FlowSpec.params: str | ParamPackRef | None`) and `Const` is a
   grammar node — an authored rate may carry inline constants with no param loader.
2. **A negative rate silently reverses a flow.** `DeclarativeFlow.evaluate` forms
   `increment = rate·dt`, legs are `coeff·increment`; a sign flip swaps donor and
   recipient. Conservation would survive, positivity would not — and here positivity must
   come from kinetics (the arbitration backstop is Euler-only and asserted-unfired).
   **Therefore: one flow per physical process, every rate strictly positive by
   construction.** No logistic `rN − cN²` (it flips sign at `N > r/c`); death is its own flow.
3. **Stoichiometry cannot reference a param** (`dict[str, float]`, literal floats only).
   This drives the two-flow carbon split below.

## The design

### The representation decision this rests on

Oxygen closes **only** via the composition annotation Phase 6 discovered
(`src/station/cabin.py`): CO₂ is `{carbon: 1, oxygen: 2}`, O₂ is `{oxygen: 2}`. The ledger
*refuses* a decoupled pure-carbon CO₂ — a real gate, not decoration. `StockSpec.composition`
exposes exactly this to an author.

### Run config

`integrator: euler` · `dt: 3600.0` (1 h) · `steps: 8760` (**one sealed year**) · `rng_seed: 0`.

### Stocks — POOL throughout, no boundary stock at all

| id | quantity | kind | composition | initial |
|---|---|---|---|---|
| `cabin.co2` | carbon | pool | `{carbon:1, oxygen:2}` | 1300.0 |
| `cabin.o2` | oxygen | pool | `{oxygen:2}` | 2000.0 |
| `algae.biomass` | carbon | pool | (1:1 default) | 200.0 |
| `crew.food_store` | carbon | pool | (1:1 default) | 2000.0 |
| `waste.feces` | carbon | pool | (1:1 default) | 0.0 |

`algae.biomass` is a **POOL, not POPULATION** (advisor): POPULATION pulls in
extinction→loss-sink routing that would need a declared loss sink, and this is a bulk
algal culture, not a discrete population. Biomass never approaches zero here anyway.

### Forcings

`crew_food_intake: 5.0e-4` (mol C/s, the frozen crew figure at `crew_count = 1`) ·
`light: 1.0` (a dimensionless multiplier — an always-lit photobioreactor).

### Flows — five, every rate strictly positive, all in `+ − ×`

**1. `crew.respiration`** — `params: crew`
```
rate: param("respired_carbon_fraction") * forcing("crew_food_intake")
stoich: crew.food_store −1, cabin.co2 +1, cabin.o2 −1
```
CARBON `−1 + 1 = 0` · OXYGEN `+1×2 − 1×2 = 0`. RQ = 1, mirroring frozen
`station.flows.CrewRespiration`.

**2. `crew.egestion`** — `params: crew`
```
rate: forcing("crew_food_intake") - param("respired_carbon_fraction") * forcing("crew_food_intake")
stoich: crew.food_store −1, waste.feces +1
```
CARBON `−1 + 1 = 0`. Positive because `f_resp ∈ [0,1]` (frozen loader bound).

> **Why two flows, not one four-legged flow.** Stoichiometry takes literal floats only, so
> a merged `CrewRespiration` would hardcode `0.949`/`0.051` as coefficients — silently
> duplicating a BVAD-calibrated frozen value and drifting if it is ever recalibrated.
> Splitting the carbon split into *two flows whose rates carry the fraction* keeps every
> coefficient an exact integer **and** sources `f_resp` from the frozen `crew` loader via
> `param()` — the `self_discharge_dsl` precedent ("the authored rate constant is the frozen
> one"). It is also truer to the invariant that a flow is one atomic transfer.

**3. `algae.photosynthesis`**
```
rate: 1.2e-6 * forcing("light") * stock("cabin.co2")
stoich: cabin.co2 −1, algae.biomass +1, cabin.o2 +1
```
CARBON `−1 + 1 = 0` · OXYGEN `−1×2 + 1×2 = 0`. **First-order in CO₂ only — deliberately
*not* ∝ biomass**, matching the frozen `Photosynthesis` idiom (advisor). A `×biomass`
factor would make the per-step CO₂ draw-fraction `k·light·biomass·dt` grow with biomass —
`≈18` at the biomass the crew requires and `dt=3600`, i.e. CO₂ negative in one step. With
this form `k_eff = k·light` is **constant**: `k_eff·dt = 4.3e-3 ≪ 1` unconditionally, the
donor-controlled self-limiting condition holds, and the system is linear. (It also removes
the zero-biomass-can't-grow trap.)

**4. `algae.respiration`**
```
rate: 1.0e-7 * stock("algae.biomass")
stoich: algae.biomass −1, cabin.co2 +1, cabin.o2 −1
```
CARBON `−1 + 1 = 0` · OXYGEN `+1×2 − 1×2 = 0`. `k·dt = 3.6e-4 ≪ 1`.

**5. `algae.harvest`** — closes the food loop
```
rate: 5.0e-7 * stock("algae.biomass")
stoich: algae.biomass −1, crew.food_store +1
```
CARBON `−1 + 1 = 0`. `k·dt = 1.8e-3 ≪ 1`.

**6. `waste.decomposition`** — closes the carbon loop completely
```
rate: 2.55e-7 * stock("waste.feces")
stoich: waste.feces −1, cabin.co2 +1, cabin.o2 −1
```
CARBON `−1 + 1 = 0` · OXYGEN `+1×2 − 1×2 = 0`. `k·dt = 9.2e-4 ≪ 1`. **This flow is what
makes carbon close** — without it, egested carbon accumulates in a dead-end sink and the
habitat slowly starves. Its addition is why the scenario needs no boundary stock at all.

### The steady state — why these constants (advisor #1/#3: state the condition, don't hope)

With `F` food, `C` cabin CO₂, `B` biomass, `W` feces, `O` cabin O₂, and
`q = 5e-4` (forced crew draw), `f = 0.949`:

```
dF/dt = k_harv·B − q
dB/dt = k_photo·L·C − (k_resp + k_harv)·B
dC/dt = −k_photo·L·C + f·q + k_resp·B + k_dec·W
dW/dt = (1−f)·q − k_dec·W
dO/dt = k_photo·L·C − f·q − k_resp·B − k_dec·W
```

Carbon conservation checks: `dF + dB + dC + dW = −q + f·q + (1−f)·q = 0` ✓.

**The closure condition the crew's forced draw imposes** (advisor #1 — the crew is FORCED,
so positivity is by *sizing*, not structure): `k_harv·B_ss = q`. The constants are chosen
to satisfy it and every other stationarity condition simultaneously:

| condition | gives |
|---|---|
| `k_harv·B = q` → `5e-7 × 1000 = 5e-4` | `B_ss = 1000` |
| `k_photo·L·C = (k_resp+k_harv)·B` → `1.2e-6 × 500 = 6e-4` | `C_ss = 500` |
| `k_dec·W = (1−f)·q` → `2.55e-7 × 100 = 2.55e-5` | `W_ss = 100` |
| `F_ss = C_total − C_ss − B_ss − W_ss` | `F` absorbs the slack |

All five derivatives vanish exactly at `(F, C, B, W) = (1900, 500, 1000, 100)`. O₂ is not
independent: `dO/dt = −dC/dt`, so **`O₂ + CO₂ = const`** (atom conservation), giving
`O_ss = 3300 − 500 = 2800`.

**Stability** (the `(B, C)` subsystem; `W` decouples with `τ = 1/k_dec ≈ 45 d`):

```
J = [ −(k_resp+k_harv)   k_photo·L ]  = [ −6.0e-7   1.2e-6 ]
    [      k_resp       −k_photo·L ]    [  1.0e-7  −1.2e-6 ]
trace = −1.8e-6 < 0 ;  det = 6.0e-13 > 0  ⇒ both eigenvalues have negative real part
λ₁ = −4.4e-7 (τ ≈ 26 d)  ·  λ₂ = −1.36e-6 (τ ≈ 8.5 d)
```

Stable, real, non-oscillatory. The fastest mode has `dt/τ = 4.9e-3 ≪ 1`, so Euler at
`dt = 3600` is comfortable — no stiffness, no overshoot. A one-year run is `≈ 14 τ` of the
slow mode: convergence completes well inside the horizon.

### Deliberately started OFF the fixed point

ICs are `(F, C, B, W) = (2000, 1300, 200, 0)` — `C_total = 3500`, so the run relaxes to
`(1900, 500, 1000, 100)`. This makes gate #4 real: **the algae bloom 5× (200 → 1000) while
drawing the cabin CO₂ down 1300 → 500 and driving O₂ up 2000 → 2800.** A habitat that
"closes" because nothing moves would prove nothing; this one visibly scrubs its own air as
the culture establishes, then holds.

### What closes, and what does not (state honestly; do not overclaim)

- **CARBON closes completely**: `food → CO₂ → biomass → food`, with the egested branch
  `food → feces → CO₂` remineralised. **No boundary leg.**
- **OXYGEN closes completely**: `cabin.o2 ⇄ cabin.co2`. **No boundary leg.**
- **WATER is out of scope** — the frozen `crew.water_balance` drains to boundary sinks and
  would add an open leg that teaches nothing here. Deliberate omission.
- Closure in the **augmented / atom-conservation** sense, per `station/cabin.py`'s own
  honesty note. A *runtime artifact*, never reference.

## What is frozen vs authored (the "authored ≠ validated" ledger)

- **Frozen and reused:** the `crew` param set (`respired_carbon_fraction = 0.949`,
  BVAD-calibrated) via `param()`; the engine, integrators, conservation gate, composition
  fold.
- **Authored (uncalibrated):** all six flow *laws* and the five inline rate constants. The
  algae/decomposer kinetics are **invented, not literature-derived** — plausible
  first-order forms, chosen to place the fixed point somewhere physically sensible. The
  crew's `q = 5e-4` mirrors the frozen crew scenario figure.
- **Consequence:** `has_authored_kinetics = True`; Godot banners it UNCALIBRATED; the graph
  dump marks it. **This is the platform working as designed** — the decision-B asymmetry
  demonstrated end-to-end on real content for the first time, not a shortfall.
- No golden, no manifest entry, no calibration claim, no place in any reference.

## Non-goals / fences

- **No unfreeze.** `git diff src/` must come back empty — this adds a scenario file and a
  test. Not the registry, not the grammar, not a manifest.
- **Not a golden.** Authored artifacts never become reference (decision B).
- **No `includes`.** A single flat file: bundle composition over a *shared* stock is a
  documented deferral (`docs/authoring-reference.md` — "Shared-stock composition"), and
  this habitat shares `cabin.co2`/`cabin.o2` across its halves by design. One file
  sidesteps it; factoring into bundles would hit that deferral and is **not** attempted.
- **Not the frozen crew flows.** `crew.food_metabolism` is carbon-only; wiring it into a
  `{carbon:1, oxygen:2}` pool fabricates oxygen and the ledger refuses it — exactly what
  Phase 6 hand-coded around. The atom-coupled merge is not in the registry, so respiration
  is authored. A real, honest platform limitation, recorded below rather than worked around.

## Gates

Not automatic — the constants above were tuned to satisfy #3 and #4 *simultaneously*
(advisor: they are in tension).

1. **Conservation** — asserted every step by the engine. Carbon and oxygen balancing over a
   sealed year *is* the closure claim; a bad stoichiometry surfaces as `ConservationError`.
2. **Determinism** — two runs bit-identical.
3. **Arbitration backstop unfired** — count == 0 (every `k·dt ≤ 4.3e-3`).
4. **The loop is live** — biomass ≈ 5×, CO₂ drawn down ≈ 2.6×, O₂ up; final state within
   tolerance of the analytic fixed point `(1900, 500, 1000, 100, 2800)`. This test earns
   the scenario its keep: it checks the habitat *works*, not merely that it balances.
5. **`has_authored_kinetics` is True** — the marker is honest.
6. `uv run pytest`, `ruff`, `pyright` green; `git diff src/` empty.

## Findings this design surfaced (each a real platform gap)

1. **The flow registry is crew-only.** `FLOW_TYPES` = 3 standalone crew flows; the frozen
   biosphere / power / thermal / eclss science is **not author-selectable**. An authored
   ecosystem must invent its kinetics rather than compose frozen, calibrated laws. The
   `flow_registry.py` docstring's "Later steps grow this to the rest of the frozen flow
   set" **did not happen** — the manifest confirms 3. This is the biggest gap between the
   roadmap's promise ("a scenario can define a habitat with its power budget, thermal
   limits, crew size, and ecosystem") and the platform as built.
2. **Stoichiometry cannot reference a param**, forcing either literal duplication of
   calibrated values or the flow-splitting workaround used here.
3. Growing the registry is an **unfreeze** (a manifest key), so both are follow-on user
   decisions — *not* absorbed into this task.

## Outcome — COMPLETE (2026-07-16)

**The habitat runs and closes.** One sealed year (8760 Euler steps, `dt = 3600`), ~1 s.
The run relaxes from its off-equilibrium start onto the analytic fixed point to **five
significant figures** — the design arithmetic predicted the behaviour before the code ran:

| stock | initial | final | predicted |
|---|---|---|---|
| `crew.food_store` | 2000 | 1900.093 | 1900 |
| `cabin.co2` | 1300 | 499.986 | 500 |
| `algae.biomass` | 200 | 999.953 | 1000 |
| `waste.feces` | 0 | 99.968 | 100 |
| `cabin.o2` | 2000 | 2800.014 | 2800 |

- **Carbon drift over the year: `+8.2e-12` mol** on a 3500 mol total (~2e-15 relative —
  pure float roundoff). **Oxygen drift: `-3.5e-11`** atoms on 6600. Both with **zero
  boundary stocks**: closure in the strict sense, inputs = outputs = 0.
- **Arbitration backstop fired 0 times**; 0 extinction events; no stock went negative.
  The food store dipped to 1825.7 during the bloom and recovered — the forced-crew sizing
  condition held.
- All six gates green. `git diff src/` **empty** — the purity invariant holds; this is a
  pure consumer of the frozen platform.

### Mutation-verified (a passing test proves nothing until you've seen it fail)

Stripping the `{carbon:1, oxygen:2}` composition off `cabin.co2` — the annotation the
whole oxygen loop rests on — is **rejected at build time, before a single step**:

```
AuthoringError: flow 'crew.respiration': authored stoichiometry is not balanced
for OXYGEN (Σ coeff·composition = -2.0, tolerance 3.0e-09);
an authored flow must conserve every quantity
```

This is the `station/cabin.py` finding ("the ledger *refuses* the decoupled version") now
enforced against an *authored* file, and it is the platform's conservation guarantee
demonstrated as a real, load-bearing gate rather than a claim.

### What the advisor changed (both passes were load-bearing)

1. **"Stop manufacturing walls."** The first pass caught me framing the crew-only registry
   and the `{+,−,×}` grammar as blockers. They are not: `+ − ×` covers all mass-action and
   polynomial kinetics, and the user asked to *author a scenario*, not to reuse frozen
   biosphere flows. The task was doable, runtime-only, all along.
2. **"Expressible ≠ stable."** The second pass caught that balance arithmetic proves only
   conservation. The **first draft would have tripped its own gates**: photosynthesis
   `∝ biomass` makes the per-step CO₂ draw-fraction `k·light·biomass·dt ≈ 18` at the
   biomass the crew requires — CO₂ negative in one step. Dropping the biomass factor
   (matching the frozen idiom) made the system linear and unconditionally stable, and
   forced the steady-state/Jacobian arithmetic that now pins every constant. Also: POOL
   over POPULATION (no extinction/loss-sink machinery), and the forced-crew sizing
   condition stated rather than hoped for.

Adding the **decomposer** (my call, following from #2's rethink) is what closed carbon
completely and removed the last boundary stock.

### Deliberately NOT done

- No unfreeze of anything: not the registry, not the grammar, not a manifest.
- No golden, no manifest entry, no cross-port anchor — authored artifacts never become
  reference (decision B). The Rust port is untouched.
- The deferred grammar (division/`monod`/`pow`/named constants) stayed deferred: **nothing
  here needed it**, so nothing forced a semantic choice. Per the freeze doc, an op joins
  only when a real flow forces its definition — this habitat did not.
- Water, and any `includes`/bundle factoring (would hit the shared-stock deferral).

### The honest limitation

This habitat's science is **invented**. It demonstrates the *platform*, not calibrated
ecology: `has_authored_kinetics = True`, Godot banners it UNCALIBRATED. That is the point —
the "authored ≠ validated" asymmetry demonstrated end-to-end on real content for the first
time. Making an authored habitat *scientifically* credible needs finding #1 (a registry
carrying the frozen, calibrated biosphere flows), which is an unfreeze and a user decision.
