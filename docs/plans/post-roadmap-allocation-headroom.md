# Allocation headroom — how much room does the frozen tree actually have?

**Taken 2026-08-12 on the user's call**, immediately after `docs/log/leaf-expansion.md`
finding 10 handed a verdict back: the sink-limited leaf-expansion mechanism is *"complete,
correct, fully measured and unshippable"*, blocked by `ArbitrationError` under RK4, with
*"NO route on the shelf"*.

Probes: `M:/claud_projects/temp/allocation-headroom/` (`probe.py` … `probe7.py`,
`RESULTS.md`). `probe5.py` is superseded — see §2.

The parked mechanism is on branch `leaf-expansion-blocked` (commit `cb668f6`, pushed);
`main` is clean and **nothing in this document changed a line of `src/`**.

---

## 1. The question, and why it is not a question about leaves

Finding 10 left the leaf mechanism refused-by-default: the only measured route to green was
a bound retune that `biosphere-reference.md` step 5 forbids. But the user's standing
direction is *"move closer and closer to reality"*, and that direction has a prerequisite
nobody had measured:

> **Is the frozen tree passing RK4 by *margin* or by *construction*?** If by margin, then
> the leaf mechanism is not special — every realism increment trips the same wall, and the
> whole direction is blocked rather than one mechanism.

That is answerable on the frozen tree alone, without the parked branch, and it is worth
more than the leaf verdict. It was measured first.

## 2. The measure — and the half of it that is arithmetic

`simcore.arbitration._scale_factors` forms `stock.amount / demand` for every clamped stock
and clamps the result at 1. Recording the **unclamped minimum** over a whole run gives the
exact distance to the wall:

    margin = min over (step, RK4 stage, clamped stock) of  amount / demand

`margin > 1` is clean; `margin < 1` raises under RK4 (`check_no_overdraw`) and is silently
rationed under Euler (`min_scaling`). `margin = 1.03` means a mechanism adding 3 % to any
single day's withdrawal breaks the run. `probe6.py`.

> ⚠ **An earlier proxy was wrong and is recorded rather than deleted.** `probe5.py` used
> `available / co2_pool` — "what fraction of everything left does this day want". It
> exceeds 1 on runs that ration zero times under Euler, because the day's *available carbon*
> is not the same object as the *allocation flow's demand on the pool*. A proxy that reads
> as the check is worse than no proxy. The measure above is arbitration's **own arithmetic**,
> not a reconstruction of it.

⚠ **And one property of this number is an identity, not a result.** Demand is formed as
`rate · dt` against the start-of-step amount, so halving `dt` halves the withdrawal against
a stock that has barely moved: `margin ∝ 1/dt` is very close to tautological, and §6 measures
it to four decimals. **Any** over-draw of this kind clears at a small enough step. What is
*not* tautological is the **level** of the number and whether the trajectory moves when the
step is refined — those are §3 and §6.2.

## 3. The frozen tree has real headroom, and it is not uniform

RK4 unless stated, worst margin over the run, always on `biosphere.carbon_pool` unless noted:

| scenario | horizon | margin, dt = 1 | implied `k·h` |
|---|---|---|---|
| `open_field` | 1 yr | **9.135** | 0.109 |
| `sealed_chamber` | 3 yr | **1.375** | 0.727 |
| `perennial_chamber` | 3 yr | **1.391** | 0.719 |
| `consumer_chamber` | 3 yr | 2.000 *(on `water_vapor`, not carbon)* | 0.500 |

**So: passing by construction in the open field, passing by ~37 % of margin in a sealed
jar.** The `k·h` column is §7's reframing; read it there, not here.

⚠ **The `consumer_chamber` row is not a carbon row.** At nominal CO₂ its binding constraint
is `biosphere.water_vapor` at *exactly* 2.0000 under both integrators — a structural ratio
(one flow withdrawing half of a stock), not a headroom measurement. Raise the chamber CO₂
and the binding stock switches to carbon. Quoting 2.000 as "the consumer chamber's carbon
headroom" would be false; it is quoted here because it is what the run's minimum actually is.

### 3.1 The shipped integrator is the *thinner* one, which was not expected

Euler is what ships. Same runs, same horizon:

| scenario | Euler, x1 | Euler, x2 CO₂ | Euler, x4 CO₂ | RK4, x4 CO₂ |
|---|---|---|---|---|
| `sealed_chamber` | 1.307 | 1.146 | **1.044** | 1.490 |
| `perennial_chamber` | 1.553 | 1.295 | **1.079** | 1.420 |
| `consumer_chamber` | 2.000 *(water)* | 1.779 | 1.327 | 1.622 |

⚠⚠ **This is a live fragility in the shipped configuration and it has nothing to do with
leaves.** Under Euler the sealed chamber loses roughly a fifth of its headroom per doubling
of chamber CO₂, and at 4× it is at **1.044** — a 4 % increment on any one day's withdrawal
away from rationing. "Raise the chamber CO₂" is one of the most obvious early realism moves
available, and it is the move that spends this margin fastest. It gets its own successor in
§10; it must **not** be filed as an aside on a leaf record.

## 4. What does NOT consume the headroom

Peak leaf carbon (mol C), Euler, 3 yr, on frozen `main` — each knob swept alone, then the
same run re-checked under RK4 (`probe.py`, `probe2.py`):

| knob | range | sealed | perennial | consumer | RK4 |
|---|---|---|---|---|---|
| `chamber_co2_mol0` (carbon **supply**) | ×1 → ×4 | 0.9215 → **1.2104** | 0.8446 → **1.1282** | 0.9582 → **1.6237** | clean; margin *improves* |
| `vcmax`/`jmax` | ×1 → ×2 | 0.9215 → 0.9071 | 0.8446 → 0.8306 | 0.9582 → 0.9829 | clean |
| specific leaf area | ×1 → ×1.5 | 0.9215 → 0.7341 | 0.8446 → 0.6639 | 0.9582 → 0.8149 | clean |
| `leaf_c0` (seedling size) | ×1 → ×4 | 0.9215 → 0.8467 | 0.8446 → 0.7771 | 0.9582 → 0.8897 | clean |
| `chamber_air_mol` | ×1 → ×4 | 0.9215 → 0.7248 | 0.8446 → 0.8489 | 0.9582 → 0.4141 | clean |

⚠ **`chamber_air_mol` is not a carbon knob, and the first pass of this table treated it as
one.** `Ci = ci_ratio · co2_mol / air_mol · 1e6` — air is the **denominator**, so scaling it
*dilutes* the CO₂. It is the carbon-**poorer** control, which is why its numbers fall. The
supply knob is `chamber_co2_mol0`, and it is the decisive row:

> **A crop 30–70 % larger than the leaf mechanism ever grew runs clean under RK4, with more
> headroom than the frozen tree has.** Plant size is not the trigger.

### 4.1 Nor is "leaf area became an aux accumulator"

The leaf build made leaf area the fourth aux accumulator — advanced once per step at the
step-entry state and held constant across RK4's four stages, which was an obvious suspect.
`probe3.py` reproduces **only that property** on the frozen tree: leaf carbon is cached per
step number inside `CarbonContext._leaf_and_biomass` while biomass stays live. Clean on all
three chambers, and **Euler bit-identical** (0.9215 / 0.8446 / 0.9582) — the sanity check
that the patch is a no-op where it must be.

⚠ Scope, stated because it is narrower than it looks: what was frozen here is a *derived*
quantity, consistent with the stage states by construction. This clears the three
already-frozen accumulators and the leaf one; it does **not** clear a hypothetical
accumulator carrying a genuinely independent value.

## 5. What is actually happening at the breaking step

`probe4.py` instruments `CarbonContext.budget` at steps 498–502 on `perennial_chamber`,
both branches:

| | CO₂ pool at the stage | the day's available C | LAI used | demand / pool |
|---|---|---|---|---|
| frozen, n = 501 | 0.0773 | 0.0128 | 0.4683 | 0.17 |
| leaf build, n = 501 | 0.2009 | 0.2770 | 0.5326 | **1.38** |

**The leaf build's single day wants more carbon than the entire pool holds** — and *not*
because the crop is bigger. At that day it has **less** leaf carbon than the frozen run had.
The mechanism is the opposite of the intuitive one:

> Sink-limited expansion **defers** early canopy build-up. Deferred canopy means deferred
> draw-down, so at day 501 the pool is ~3× fuller, `Ci` is high, and assimilation runs about
> an order of magnitude faster than the frozen tree's at the same date. The daily rate is
> computed at the **start-of-step** concentration and applied for a whole day, so the
> within-day draw-down is never resolved.

**The jar's carbon turnover time has fallen below `dt`.** Both of the branch's breaks land
on **the same step, 501**, in two differently-configured chambers — a *day* signature, not a
*size* signature. That coincidence is what pointed at the step in the first place.

## 6. Halve the step and it clears

`probe7.py`: each weather day is simulated as `sub` sub-steps at `dt = 1/sub` (the day is
repeated `sub` times so the forcing total is unchanged, and the re-sow period scales with it).

**Parked branch, RK4, 3 yr:**

| scenario | dt = 1 | dt = 1/2 | dt = 1/4 |
|---|---|---|---|
| `sealed_chamber` | **BREAK**, 0.948 | clean, **2.066** | clean, **4.337** |
| `perennial_chamber` | **BREAK**, 0.967 | clean, **2.116** | clean, **4.486** |
| `consumer_chamber` | clean, 1.735 | clean, 3.576 | clean, 7.638 |

**Frozen `main`, RK4 — the control, same harness, no leaf mechanism:**

| scenario | horizon | dt = 1 | dt = 1/2 | dt = 1/4 | peak leaf C across the three |
|---|---|---|---|---|---|
| `open_field` | 1 yr | 9.135 | 18.435 | 37.054 | 9.6593 / 9.6041 / 9.5765 |
| `sealed_chamber` | 3 yr | 1.375 | 2.744 | 5.481 | 0.8867 / 0.8862 / 0.8862 |
| `perennial_chamber` | 3 yr | 1.391 | 2.776 | 5.547 | 0.8113 / 0.8121 / 0.8117 |
| `consumer_chamber` | 3 yr | 2.000 | 4.000 | 8.000 | 0.9433 / 0.9417 / 0.9409 |

**No parameter moved, no form changed, nothing was retuned.**

### 6.1 The honest reading of the first table

Per §2, `margin ∝ 1/dt` to within a per cent everywhere is the **identity**, not the finding
— `consumer_chamber` reproduces 2.000 → 4.000 → 8.000 exactly. So "it clears at dt = 1/2" on
its own is worth very little, and a record that stopped there would be inviting the reader
to mistake **a guard that stopped firing** for **a mechanism that was validated.**

⚠⚠ **That is `leaf-expansion.md` finding 9's own lesson, running in the opposite
direction.** Finding 9's durable content was: *a guard that is blind by construction cannot
be the evidence that a mechanism is safe* — `rationed == 0` under Euler proved nothing
because Euler's backstop cannot see an over-draw. Shrinking `dt` until `check_no_overdraw`
goes quiet is the **same error with the sign flipped**. Silence is not endorsement either way.

### 6.2 What the control actually establishes — this is the finding

Two facts from the frozen rows, and only these two are load-bearing:

1. **The frozen science is already converged at `dt = 1` under RK4.** A 4× refinement moves
   peak leaf carbon by 0.06 % (sealed 0.8867 → 0.8862) and less than 0.9 % anywhere. So
   refining the step moves the **safety margin** without moving the **answer** — the margin
   at `dt = 1` is a discretisation artefact of the withdrawal arithmetic, not a statement
   about the biology.
2. ⚠ **Under Euler — the integrator that ships — the same refinement moves peak leaf carbon
   0.9215 → 0.8923, 3.2 %.** The shipped step carries about 3 % truncation error, and every
   biosphere golden has that baked in. This is not a defect (Euler at `dt = 1` is frozen
   contract, chosen deliberately), but it is the number anyone re-opening the step contract
   has to know, and it had never been written down.

3. **The open field has 9× margin against the chambers' 1.4×.** `dt = 1` is not marginal in
   general. It is marginal **in a sealed chamber** — which is exactly the configuration the
   station runs, and exactly the configuration this project is about
   (`chamber-scale-diagnosed`: the jar holds about two days of one crop's carbon).

## 7. ⚠ THIS TREE ALREADY HAS THIS BOUND, IN THREE PLACES, AND THE CARBON POOL IS THE ONE PLACE IT CANNOT BE WRITTEN

The reframing that makes the numbers above legible in the repo's own vocabulary. For a
withdrawal roughly proportional to the stock it draws from, `demand ≈ k · amount · dt`, so

    margin = amount / demand ≈ 1 / (k · h)

— the margin is the **reciprocal of the authoring platform's `k·h` precondition.** That
approximation is good near the wall here, because `Ci` is exactly linear in the pool
(`ci_from_co2_pool`) and assimilation is near-linear in `Ci` at low concentration. And it is
independently corroborated: §6's clean `1/dt` scaling is precisely what `k·h` predicts.

The same bound already appears, three times, everywhere else in the tree:

| where | how it appears |
|---|---|
| `src/authoring/interpreter.py` | a **build-time** `k·h < 1` check that refuses the scenario, with a per-rate-class remedy |
| `eclss.yaml`, `water_recovery.yaml` | rate constants **chosen** so `k·dt = 0.06 < 1`, said out loud in the `source:` note |
| `loader.py` `load_stem_reserve_params` | `remobilization_rate ∈ (0, 1] day⁻¹`, bounded citing *"the `k·h < 1` precondition the multi-rate authoring work made a build-time check elsewhere in this tree"* |

**The biosphere's chamber carbon pool is the one withdrawal in the tree whose `k` is not a
parameter.** It is emergent — the product of PAR, LAI, temperature, `Ci`, and the stress
factors — so there is no load-time bound anyone could have written, and none was. Measured
`k·h` (RK4, `dt = 1`, the reciprocals in §3):

* frozen sealed **0.727**, perennial **0.719**, consumer **0.500**, open field **0.109**
* leaf branch sealed **1.055**, perennial **1.034** — **over the bound the rest of the tree
  enforces**

> **So the leaf mechanism did not discover a new failure mode. It pushed the one stock in
> the tree that has no `k·h` precondition past the precondition.** Every other domain either
> checks this at build time or picked its rate constant to satisfy it. The biosphere never
> could, and until now never had to.

⚠ The approximation has a limit worth stating: `k` here is not constant over a season, and
the identity is a local reading at the binding step, not a global property. It is a
**reframing that makes the number interpretable**, not a new measurement.

## 8. Three routes, priced — because naming one makes it the default

### Route A — refuse the leaf mechanism (the standing recommendation, now unsupported)

`leaf-expansion.md` finding 10 recommended refuse-and-revert on the grounds that the only
measured route to green was a forbidden retune. §6 falsifies the premise: a second route
exists and moves no parameter. **Route A is no longer supported by its own argument.** It
remains available as a *scope* decision (the mechanism is expensive and its evidence base
needs redoing — §9), but it can no longer be justified as "there is no way to run it."

### Route B — a finer biosphere step

Measured, works, moves nothing scientific (§6). It is also the **most expensive thing in
this document**, and the price is understated by anyone who says "it moves the goldens":

| what it touches | why |
|---|---|
| **Biosphere freeze** | `Euler / dt = 1` is the *first item* in `docs/biosphere-reference.md`; 7 scenarios → goldens |
| **Station freeze** | biosphere scenarios are *delegated* to the biosphere contract, so they move with it; 13 scenarios → goldens |
| **Native-port freeze** | `docs/native-port-reference.md` freezes **measured** Tier-2 tolerance bands per golden (`1e-11` biosphere, `1e-12` station); 20 goldens carry the cross-port tier contract |

⚠ **The native-port row is weaker than the first draft of this table claimed, and the
weakness is itself the point.** That doc **never names a step** — it tabulates bands per
golden and points at `tiers.json`. The dependence on `dt = 1` is entirely *implicit*: the
bands are measurements taken on runs of the frozen goldens, and those goldens run at
`dt = 1` because the biosphere contract says so. So the bands would have to be re-measured,
but nothing in that document would go red to tell you, and nothing in it says why. **A third
contract whose dependence on the thing being unfrozen is unstated is exactly the contract
that gets left out of an unfreeze price.** (The first draft here asserted "bands measured at
`dt = 1`" as though the doc said it. It does not; checked.)

**Three contracts, not one**, plus a re-measure of the Rust parity tiers and a performance
cost (2× or 4× the biosphere step count) that
`docs/perf-baseline.md` would need to absorb. ⚠ And §6.2(2) says the goldens would move for
a *second* reason beyond the step change: the shipped Euler trajectory itself is 3.2 % from
its refined limit, so this is not a pure re-freeze at the same numbers.

⚠ **The existing multi-rate machinery is NOT a cheap version of route B, and saying so would
be false.** This was checked because it was the first thing I assumed:

* `simcore.multirate` splits one shared master `dt` and composes `substep` only — it
  **freezes aux by design**. `src/station/greenhouse.py` says exactly this in its own
  docstring, which is *why* the station driver hand-rolls an operator split and keeps the
  biosphere at one step per day. Phenology would stop advancing.
* `rate_class` is an **authoring-platform** flow field whose slow class steps at `dt/2`
  regardless of `n_sub` (Strang), per `multirate-effective-step-is-per-rate-class`. Adopting
  it is not a surgical carve-out — every flow's effective step changes.

The only route measured here is a **globally finer biosphere step**.

### Route C — positivity from kinetics (NOT measured; the cheapest if it holds)

`simcore/arbitration.py`'s own contract says it: under a higher-order scheme, **positivity
must come from the kinetics, not from a clamp.** That is an instruction, and route C is the
one that follows it. §5 shows the dependence on the pool already exists — assimilation reads
`Ci`, `Ci` is linear in the pool — it is simply **not steep enough** for the whole-day
explicit integral to stay under the pool.

A saturating/supply-limited form on pool concentration would keep `dt = 1`, touch no
integrator contract, and move no golden the biosphere freeze does not already own. Against it:

* ⚠ **A half-saturation constant tuned so the guard goes quiet is `a-clamp-hides-a-wrong-amount`
  in a new costume** — worse than the `0.90` bound retune finding 9 refused, because it would
  *look* like mechanism. This route is only legitimate if the form and its constant come from
  a source, and that has **not** been checked. It is the first thing to check.
* The `monod` node exists in `src/simcore/expr.py` and the authoring grammar, **not** in the
  biosphere's compiled Python path — so "the tree can already say this" is true of the
  authoring platform and false of `carbon_budget.py`. This would be new Python in a frozen
  module.
* It changes the science, where route B does not. That cuts both ways: a real CO₂-depletion
  response in a sealed jar is *more* physical than an instantaneous one, which is arguably
  the point of the whole exercise — but it is a science claim needing a citation, not a
  discretisation fix.

**Not measured. Priced so that route B does not win by default.**

## 9. ⚠ WHAT THIS DOES NOT DISCHARGE

**The leaf mechanism's evidence base has never been measured at the step that would ship
it.** §6 measures the *guard* on the parked branch at `dt = 1/2` and `1/4`. The evidence
that got the mechanism accepted is a different set of objects, and all of it was measured
under **Euler at `dt = 1`**:

* the Greenwood biomass gate clearing by 5.2 % at the same phyllochron that failed it
* leaf thickness 1.03–1.18× nominal median, inside real wheat's range
* [F]'s node branch still deciding ~39–45 % of days (i.e. the sink limitation surviving)
* rationing 0 on all eleven scenarios
* the `WSFL` leverage cut of ~400× on `water_biting`

`probe7.py` does print peak leaf carbon for clean runs, so a single trajectory number exists
for the branch at the finer steps — but one number is not that list. **Re-measuring the list
at the shipping step is the first step of any leaf successor, not a formality**, and §6.2(2)
gives a concrete reason to expect movement: the frozen tree's own Euler trajectory shifts
3.2 % under the same refinement.

So the correct standing of the leaf mechanism after this work is **"not refuted, route
identified, evidence base pending re-measurement"** — not "unblocked, ships as-is."

## 10. Successors, named

1. **The `dt = 1` contract for a sealed chamber** (route B's decision). The measurement is
   done; what remains is a contract decision with a three-freeze price. ⚠ Independent of the
   leaf question — §6's control is entirely on frozen `main`.
2. **Route C's shelf search**, with the discriminator stated so the successor knows what a
   "yes" looks like: **does [E] or [F] give assimilation as a function of *ambient / pool*
   CO₂ with a cited half-saturation — as opposed to the Farquhar `A`–`Ci` curve we already
   have?** That distinction is the whole question. Our curve *is* saturating in `Ci`, but
   `Ci` is **linear** in the pool (`ci_from_co2_pool`), so the composition is near-linear at
   low pool and that is precisely why it is not steep enough. A form that saturates in the
   *pool* is a different object. If neither source separates them, **route C is refused on
   provenance and route B is the only route**, which materially simplifies the decision in
   (1) — so this search should run *before* (1) is decided, not after.
3. ⚠ **The Euler CO₂ fragility (§3.1), on its own.** Nothing to do with leaves: the shipped
   configuration at 4× chamber CO₂ sits at margin 1.044. Any work item that raises chamber
   CO₂ must measure this first. This is the successor most likely to be needed soonest,
   because raising CO₂ is the cheapest realism move on the board.
4. **The leaf evidence base re-measured at the shipping step** (§9) — gated on (1) or (2).
