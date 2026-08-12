## **Allocation headroom** — the leaf blocker re-diagnosed as a step-size bound, not a science defect

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record, with every table, is
> [`../plans/post-roadmap-allocation-headroom.md`](../plans/post-roadmap-allocation-headroom.md).
> Written new rather than migrated, so there is no pre-split table cell behind it.

**DIAGNOSED 2026-08-12, NOTHING BUILT.** Taken the moment `leaf-expansion.md` finding 10
handed a verdict back (*"complete, correct, fully measured and unshippable … NO route on the
shelf"*). The question asked was deliberately **not** about leaves: with the user's standing
direction being *"move closer and closer to reality"*, the prerequisite nobody had measured
is whether the frozen tree passes RK4 **by margin or by construction** — because if by
margin, the leaf mechanism is not special and the whole direction is blocked rather than one
mechanism. Answerable on frozen `main` alone, and answered there: probes
`M:/claud_projects/temp/allocation-headroom/`, `src/` untouched throughout, the parked branch
`leaf-expansion-blocked` (`cb668f6`) never merged.

**FINDING 1 — THE MEASURE IS ARBITRATION'S OWN ARITHMETIC, AND THE FIRST ATTEMPT AT IT WAS A
PROXY THAT READ AS THE CHECK.** `_scale_factors` forms `amount / demand` per clamped stock
and clamps at 1; the **unclamped** minimum over a run is the exact distance to the wall
(`> 1` clean, `< 1` raises under RK4, silently rationed under Euler). ⚠ The first probe used
`available / co2_pool` — *"what fraction of everything left does this day want"* — which
**exceeds 1 on runs that ration zero times**, because a day's available carbon is not the
allocation flow's demand on the pool. Superseded and kept on the record: a proxy that reads
as the check is worse than no proxy.

**FINDING 2 — THE FROZEN TREE IS NOT ON A KNIFE EDGE, AND THE HEADROOM IS WILDLY
NON-UNIFORM.** Under RK4 at `dt = 1`: open field **9.135**, sealed chamber **1.375**,
perennial **1.391**. Passing by construction in a field; passing by ~37 % of margin in a jar.
⚠ `consumer_chamber`'s 2.000 is **not a carbon number** — its binding stock at nominal CO₂ is
`water_vapor` at *exactly* 2.0000 under both integrators, a structural ratio rather than a
headroom measurement; quoting it as carbon headroom would be false.

**FINDING 3 — SIZE IS NOT THE TRIGGER, AND THE DECISIVE SWEEP IS THE ONE THAT WAS FIRST READ
BACKWARDS.** Photosynthetic capacity ×2, seedling ×4, specific leaf area ×1.5, and chamber
CO₂ ×4 all run clean under RK4 — and CO₂ ×4 grows a crop **30–70 % larger than the leaf
mechanism ever grew**, with *more* headroom than the frozen tree has. ⚠ `chamber_air_mol` was
swept as a carbon knob and is not one: air is the **denominator** in
`Ci = ci_ratio·co2_mol/air_mol·1e6`, so scaling it *dilutes* the CO₂ — it is the
carbon-poorer control, and the run that looked like "bigger chamber, still fine" was
measuring the opposite of the claim attached to it. **Nor is the aux accumulator the
trigger:** reproducing *only* that property on the frozen tree (leaf carbon cached per step
inside the carbon budget, biomass live) is clean on all three chambers and **Euler
bit-identical**, the sanity check that the patch is a no-op where it must be.

**FINDING 4 — THE BREAKING STEP SAYS THE OPPOSITE OF THE INTUITIVE STORY.** At the failing
day the leaf build asks for more carbon **than the entire pool holds** — and it has *less*
leaf carbon there than the frozen run had. Sink-limited expansion **defers** early canopy
build-up, so the pool is ~3× fuller, `Ci` is high, and assimilation runs about an order of
magnitude faster. The daily rate is computed at the **start-of-step** concentration and
applied for a whole day, so within-day draw-down is never resolved: **the jar's carbon
turnover time has fallen below `dt`.** Both breaks land on **the same step, 501**, in two
differently-configured chambers — a *day* signature, not a *size* signature, and that
coincidence is what pointed at the step in the first place.

**FINDING 5 — IT CLEARS AT A FINER STEP WITH NOTHING RETUNED, AND HALF OF THAT SENTENCE IS AN
ARITHMETIC IDENTITY.** The parked branch runs **clean** at `dt = 1/2` and `1/4` on both
chambers that broke; no parameter moved, no form changed. ⚠ But demand is `rate · dt` against
a stock that has barely moved, so `margin ∝ 1/dt` is near-tautological — `consumer_chamber`
reproduces 2.000 → 4.000 → 8.000 **exactly** — and *any* over-draw of this kind clears at a
small enough step. ⚠⚠ **A RECORD THAT STOPPED THERE WOULD BE REPEATING `leaf-expansion.md`
FINDING 9 WITH THE SIGN FLIPPED.** That finding's durable content was *a guard that is blind
by construction cannot be the evidence that a mechanism is safe* — `rationed == 0` under
Euler proved nothing. **Shrinking `dt` until `check_no_overdraw` goes quiet is the same
error.** Silence is not endorsement in either direction.

**FINDING 6 — WHAT THE CONTROL ESTABLISHES, WHICH IS THE PART THAT IS NOT ARITHMETIC.** Run
on frozen `main`, the same harness, no leaf mechanism: **(a)** the frozen science is *already
converged* at `dt = 1` under RK4 — a 4× refinement moves peak leaf carbon by **0.06 %** — so
refining the step moves the **margin** without moving the **answer**, i.e. the margin at
`dt = 1` is an artefact of the withdrawal arithmetic, not a statement about the biology.
⚠ **(b)** under **Euler, the integrator that ships**, the same refinement moves peak leaf
carbon **3.2 %**: the shipped step carries ~3 % truncation error and every biosphere golden
has it baked in. Not a defect — `Euler / dt = 1` is deliberate frozen contract — but it had
never been written down, and it is the number anyone re-opening the step contract needs.
**(c)** the open field's 9× against the chambers' 1.4× means `dt = 1` is not marginal in
general; it is marginal **in a sealed jar**, which is the configuration the station runs and
the one `chamber-scale-diagnosed` already measured at ~2 days of one crop's carbon.

**FINDING 7 — ⚠ THIS TREE ALREADY ENFORCES THIS BOUND IN THREE PLACES, AND THE CARBON POOL IS
THE ONE PLACE IT CANNOT BE WRITTEN.** For a withdrawal roughly proportional to its stock,
`margin ≈ 1/(k·h)` — the **reciprocal of the authoring platform's `k·h` precondition** (good
near the wall here, since `Ci` is exactly linear in the pool and assimilation near-linear in
`Ci` at low concentration; independently corroborated by the clean `1/dt` scaling above). The
same bound appears as a **build-time check** in `authoring/interpreter.py`, as rate constants
**chosen** so `k·dt = 0.06 < 1` in `eclss.yaml` and `water_recovery.yaml`, and as
`remobilization_rate ∈ (0, 1]` in `loader.py`, whose docstring cites it by name. Measured
`k·h`: frozen sealed **0.727**, perennial **0.719**, open field **0.109**; leaf branch sealed
**1.055**, perennial **1.034** — **over the bound the rest of the tree enforces.** The
biosphere's chamber carbon pool is the one withdrawal whose `k` is **emergent** rather than a
parameter (PAR × LAI × temperature × `Ci` × the stress factors), so no load-time bound could
ever have been written and none was. **So the leaf mechanism did not discover a new failure
mode — it pushed the one stock with no `k·h` precondition past the precondition.**

**FINDING 8 — ⚠⚠ A LIVE FRAGILITY IN THE SHIPPED CONFIGURATION, WITH NOTHING TO DO WITH
LEAVES.** Under **Euler** the sealed chamber loses roughly a fifth of its headroom per
doubling of chamber CO₂ — 1.307 → 1.146 → **1.044** at ×4 (perennial 1.553 → 1.295 →
**1.079**). Four per cent of room. **"Raise the chamber CO₂" is one of the most obvious early
realism moves available and it is the move that spends this margin fastest**, so it carries
its own successor rather than an aside on a leaf record. ⚠ Note the direction: at ×4 the
*shipped* integrator is **thinner** than RK4 (1.044 vs 1.490), which was not expected.

**THE VERDICT, AND WHAT IT DOES NOT DISCHARGE.** Three routes are priced in the plan doc so
that naming one does not make it the default. **(A) Refuse the leaf mechanism** — the
standing recommendation, and **no longer supported by its own argument**, which was "the only
measured route to green is a forbidden retune"; that premise is falsified. It survives only
as a scope decision. **(B) A finer biosphere step** — measured, works, moves nothing
scientific, and is the most expensive item here: `Euler / dt = 1` is the *first* item in the
biosphere freeze, the station freeze **delegates** biosphere scenarios so they move with it,
and `native-port-reference.md` freezes **measured** cross-port tolerance bands — **three
contracts, not one**, plus a Rust parity re-measure and 2–4× the step count. ⚠ That third
one is the interesting entry: the doc **never names a step**, so its dependence on `dt = 1`
is entirely implicit (the bands are measurements taken on goldens that run at `dt = 1`
because the *biosphere* contract says so). Nothing in it would go red. **A contract whose
dependence on the thing being unfrozen is unstated is the one left out of the price** — and
the first draft of this record asserted the doc said so, which it does not. ⚠ And the
existing multi-rate machinery is **NOT** a cheap version of it: `simcore.multirate` **freezes
aux by design** (`station/greenhouse.py` says so in its own docstring, which is why the
station driver hand-rolls an operator split), so phenology would stop advancing, and
`rate_class` is an *authoring* field whose slow class steps at `dt/2` regardless of `n_sub`.
**(C) Positivity from kinetics** — arbitration's own contract instruction, keeps `dt = 1`,
touches no integrator contract, **and is unmeasured**: the pool dependence already exists via
`Ci` and is simply not steep enough, so this is a saturating form whose half-saturation
constant must come from a source. ⚠ Tuned instead, it is `a-clamp-hides-a-wrong-amount` in a
new costume — *worse* than the bound retune finding 9 refused, because it would look like
mechanism. ⚠⚠ **AND THE LEAF EVIDENCE BASE HAS NEVER BEEN MEASURED AT THE STEP THAT WOULD
SHIP IT.** What was measured at `dt = 1/2` is the **guard**. The evidence that got the
mechanism accepted — the Greenwood gate clearing by 5.2 %, thickness inside real wheat's
range, the node branch deciding ~39–45 % of days, rationing 0, the `WSFL` leverage cut — is
**all Euler at `dt = 1`**, and finding 6(b) gives a concrete reason to expect it to move. So
the leaf mechanism's standing is **"not refuted, route identified, evidence base pending
re-measurement"** — *not* "unblocked, ships as-is". Successors named: the `dt = 1` contract
decision for a sealed chamber (independent of leaves — the control is entirely on frozen
`main`); route C's shelf search for a *cited* supply-limited assimilation form; the Euler CO₂
fragility on its own; and the leaf evidence base re-measured, gated on the first two.
