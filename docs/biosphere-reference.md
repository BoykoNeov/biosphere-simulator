# The biosphere reference (frozen) — Phase 4, P4.3

The biosphere is the project's **first domain and its reference domain**. Phase 4 froze it:
it is no longer a moving target. Phase 5 sibling domains (power / thermal / atmosphere-ECLSS
/ crew) are verified standalone against their own references and then against *this* one; the
eventual Rust port ports it **verbatim** (roadmap line 7: *"We port a stable multi-domain
engine, not an evolving one"*). This file is the **freeze contract** — what is frozen, the
evidence the freeze rests on, and the **unfreeze discipline** for ever changing a frozen item.

This is the Phase-0 *"freeze the engine architecture before scientific complexity appears"*
discipline applied one level up — to the biosphere **science**. It is **boundary-side docs +
a manifest only**: `git diff src/simcore/` stays **empty**, unconditionally.

Its machine-readable companion is **`docs/biosphere-reference.manifest.json`** (generated;
see *The manifest* below). The plan of record is
[`docs/plans/phase-4-closed-biosphere.md`](plans/phase-4-closed-biosphere.md).

## What "frozen" means (and what it does NOT)

**Frozen** = the items below are the committed reference. A change to any of them is an
**unfreeze event** that must follow the discipline at the bottom of this file — it is not an
ordinary edit. Freezing is a *process* discipline, **not a code lock**: nothing in the source
forbids editing a param file; the goldens + the manifest gate make an undocumented change
*fail CI*, which is what gives the freeze teeth.

**Frozen ≠ finished-forever.** A reference can be deliberately revised (a calibration pass, a
new trophic level in a later phase). The discipline only insists the revision be *documented,
reviewed, and re-captured* — not silent.

**Not part of the biosphere reference (scoped out, by name):**
- The **Phase-0 engine-skeleton demo** — `domains/biosphere/flows.py` (the trivial
  `Photosynthesis`/`Respiration`/`Harvest` transfers, *no real biology*) and its
  `params/demo.yaml`. These exercise the *engine* (RK4 vs Euler, the conservation gate). The
  manifest excludes `demo.yaml` explicitly.
  ⚠ **Their two goldens were DELETED on 2026-08-18** (`demo_euler_state.json`,
  `demo_rk4_state.json`; C6 of the reference flip — `docs/log/reference-flip.md`), because
  the reference has no `build_demo` and no contract required them. `state_snapshot.json`
  stays: it is a hand-authored `sim_io` fixture the reference *reads*. So `demo.yaml` is now
  frozen by **nothing** — deliberate, and recorded here rather than left to be discovered.
- **No new science.** Phase 4 added no flow, no trophic level, no coupled
  (Lotka-Volterra/Holling) dynamics — those were deferred at the Phase-3 capstone. The freeze
  captures the closed biosphere **as Phase 3 left it**.
- **The two additive dormant-machinery scenarios** — `N_LIMITED_SCENARIO` (open field,
  `f_N` driven below 1 by N-dilution) and `WATER_BITING_SCENARIO` (sealed chamber, the closed
  water cycle's `f_water` driven below 1). Added *after* the freeze (the Phase-5 sequencing
  decision) to flush the never-run-hot `f_N` and sealed-`f_water` limiter integrations before
  Phase 5, they were **deliberately NON-frozen**: scenario *data* only (no new flow / aux /
  param), their own goldens and tests, and **not** in the manifest. Adding them left all seven
  frozen goldens byte-identical — that byte-identity was the proof the reference did not move.
  ⚠ **RETIRED 2026-08-18** (C6 of the reference flip; `docs/log/reference-flip.md`). Both
  scenarios, both goldens and their five test files are deleted. The reference carries no such
  scenarios, so keeping them would have meant a checker asserting science the reference cannot
  run. **Nothing in the manifest moved** — neither name ever appeared in it, which is what
  made the retirement an ordinary deletion rather than an unfreeze.
  ⚠ **The limiters they existed to exercise are still exercised, in the reference.** `f_N`
  and its uptake shutoff now have manufactured-condition pins in `rust/crates/domains`
  (`science.rs::the_nitrogen_stress_ramp_is_linear_between_its_two_knots`,
  `system.rs::nitrogen_limitation_is_wired_into_assimilation_and_no_scenario_shows_it`), and
  the water side already had one. Before the deletion, `nitrogen_stress_factor` had **zero**
  test callers in the reference — so this doc's "never-run-hot" concern was live, and the
  successor landed first for exactly that reason.

## The frozen surface

The manifest is the authoritative, machine-checked list. This section is the human-readable
account.

### Locked integrator + dt — **Euler, `dt = ¼ day`**

The biosphere runs the **forward-Euler** integrator at a **quarter-day** step (`t = n·dt`,
integer step count; `n` counts *steps*, so it is 4× the day count). Euler was **locked by
probe, with evidence** (P4.1, Step 1): both closed scenarios were run Euler *and* RK4 to 15 yr
and structurally agreed (both stationary, both closed, same period class); the 100k-step
stress (Step 3, 328 yr) confirmed no slow drift. RK4 ships in `simcore` but the biosphere does
**not** use it — crop physiology is daily-integrated and the daily canopy flux is not
RK4-refinable.

⚠ **The step and the day are now different numbers, and that is the whole point.** The step
lives in **one** place, `src/domains/biosphere/step.py` (`BIO_DT`, `STEPS_PER_DAY`,
`steps_for`), and every run length, reset period and perturbation window is expressed in
**days** and converted. The weather table stays **one row per physical day** at any step —
`season._table` indexes `int(n · dt)`, not `n` — and must never be tiled to match
`STEPS_PER_DAY`. The manifest records `dt_days`, `tests/test_freeze_manifest.py` asserts it
against a **hard-coded literal** (deliberately *not* against `BIO_DT` — a contract that
imports its value from the code auto-follows the code), and the goldens enforce the values.

#### ✅ RESOLVED 2026-08-14 (was a KNOWN DEVIATION recorded 2026-08-13): the CO₂ compensation-point crossing

The `dt = 1` reference **fixed carbon at concentrations where its own FvCB kinetics say it
fixes none**: assimilation is `max(0, min(Ac, Aj))` and both branches carry `(Ci − Γ*)`, so
with `Γ* = 42.75` and `ci_ratio = 0.7` the crop cannot draw the chamber below
`Ca = 61.07 ppm` — and the **perennial chamber's** season-low sat at **56.03 ppm**
(consumer likewise). It was a **truncation error, not a threshold crossing**: the withdrawal
was computed at the start-of-step concentration and applied for a whole day, so a step
starting above the shutoff and ending below it never re-evaluated.

**Measured at the shipped `dt = ¼`: the perennial chamber's season-low was `75.48 ppm`**
— clear of the 61.07 shutoff, so the reference no longer fixes carbon below its own
compensation point. (⚠ That reading is the step ceremony's, 2026-08-14. Two later
unfreezes moved it to **70.25 ppm**, still clear; the current five are tabulated in the
band section below. The verdict this sentence records is unchanged — the margin it was
measured with is not.) Authorized by the user (*"quarter the step"*, over `½`, which also clears
it: `¼` leaves 4.8× headroom to the arbitration bound where `½` leaves 2.1×, so the next
mechanism added probably does not force a second ceremony). Ceremony:
[`plans/post-roadmap-step-unfreeze.md`](plans/post-roadmap-step-unfreeze.md).

##### ⚠ CORRECTED 2026-08-14 — this entry named the SEALED chamber, and the sealed chamber never crossed

This section, and every write-up of the ceremony, headlined *"the sealed chamber's season-low
sat at 57.9 ppm"* and paired it with *"76.82 ppm at `dt = ¼`"*. **The pairing compares two
different runs, and the 57.9 belongs to neither the sealed chamber's contract nor its
golden.** A prose-only correction: no golden, threshold, scenario or manifest value moves.

`season.run_perennial` applies `annual_reset` **unconditionally** — it never asks whether the
scenario is a perennial one — and the 2026-08-13 step sweep drove *every* scenario through
it. The sealed chamber's golden (`tests/test_regression_sealed_season.py`) uses plain
`run_season` and re-sows never. Re-measured on today's tree:

| scenario, in **its own golden's** configuration | `dt = 1` | `dt = ¼` | crossed at `dt = 1`? |
|---|---|---|---|
| `sealed_chamber` (no re-sow) | **75.75** | **76.82** | **no — never below 61.07** |
| `perennial_chamber` (re-sows) | **56.03** | **75.48** | **yes** |
| `consumer_chamber` (re-sows) | 73.29 | 74.42 | no (nearest, 20 % clear) |

and the sweep's 57.89 is the sealed chamber's **third re-sown season**, a run no golden and
no contract performs.

⚠ **The step move stands; only the locus was wrong.** The perennial and consumer chambers
re-sow, their goldens run that way, and the perennial crossing is real — a 8.3 % crossing of
a hard shutoff, sustained. Two corroborations that this is a mislabelling and not model
drift: the sweep's table reproduces **cell for cell** on today's tree, and today's `dt = 1`
`run_season` reproduces the pre-unfreeze committed golden **bit-exactly**
(`biosphere.carbon_pool = 2.35678018024373`).

**Both figures the old note insisted be quoted together, restated:** the tail statistic that
was **26 % low** (56.03 against a converged ~75.75 on the perennial chamber) is the one this
change fixes; the **headline outputs that moved ~3 %** moved as predicted (sealed peak leaf
carbon 0.9215 → 0.8923, −3.2 %). Neither alone was an honest summary of the defect, and
neither alone is an honest summary of the repair.

⚠ **Two things this does NOT claim.** (a) `Γ*` remains a `TODO(cite)` entry in
`photosynthesis.yaml`, so the *crossing* was robust but the *number* 61.07 is still
provisional — clearing a provisional threshold is a weaker result than clearing a cited one.
(b) ~~The sweep's convergence sequence is stale because the tree gained mechanisms.~~
**WITHDRAWN 2026-08-14 — that explanation was wrong.** The sequence
(`57.9 → 75.1 → 75.8 → 76.0` against a limit of `76.29`) is arithmetically correct and
reproduces exactly today; it is the **re-sown** sealed run's sequence, and the 76.82 it was
compared against is the **no-re-sow** run's. Two configurations, not two trees. Separated,
each converges monotonically from below to its own RK4 limit — which is what dissolves the
apparent paradox. Both sequences are tabulated in `src/domains/biosphere/step.py`. What
remains true is the *instruction*: **do not quote a convergence figure without saying which
run it belongs to.**

The historical record of the deviation while it stood — how it was found, the enrichment
sweep, and the headroom measurement — is
[`log/co2-enrichment-margin.md`](log/co2-enrichment-margin.md) and
[`log/allocation-headroom.md`](log/allocation-headroom.md). ⚠ Read those as **dated**, and
⚠ **also as unverified on this point**: they headline the same `57.9 ppm` figure and predate
the sweep, so whether they measured the sealed chamber through the same unconditional re-sow
has **not been checked**. Named as an open question, not asserted either way.

⚠ **The gap that let this ship for a year, worth more than the fix.** It was never a guard
failure: arbitration rationed zero times, every golden gate was green, and every
`science_band` passed. **A `science_bands` entry of the right shape — *"every sealed
scenario's season-low CO₂ stays above the compensation point"* — would have caught it on day
one.** The reason it did not exist is that the bands were written to describe what the model
*does*, not to cross-check the model against its own kinetics. That band is now writable (the
perennial chamber passes at 75.48 ppm against 61.07, the sealed at 76.82) and adding it is the
natural successor to this ceremony; it is deliberately **not** bundled here, because a band
written in the same change that makes it pass is a restatement of the run, not a contract on
it.

⚠ **CORRECTED 2026-08-14, and this correction is the sharpest thing on this page.** The
sentence above named *"the sealed chamber's season-low CO₂"* — **the one scenario that never
crossed.** As originally written, the proposed guard would have passed on day one and caught
nothing. The scenario-specific phrasing came from the same locus error as everything else in
this entry, and it survived into the recommendation for the *fix*. **A guard inherits the
locus of the diagnosis that motivated it; if the diagnosis names the wrong subject, the guard
is aimed at it too.** Hence the band is written over the whole sealed roster rather than one
scenario, which is also the form that does not need the diagnosis to have been right.

#### ✅ THE BAND LANDED 2026-08-14, one ceremony later, over all five chamber scenarios

`science_bands` gains five entries — `sealed_chamber`, `perennial_chamber`,
`consumer_chamber`, `perennial_long_horizon`, `consumer_long_horizon` — each asserting
*season-low chamber CO₂ > `Γ*/ci_ratio`*, at
[`tests/test_co2_compensation_band.py`](../tests/test_co2_compensation_band.py). **A
schema-free unfreeze: five manifest entries, no value moved, no golden regenerated, no
`src/` change** (`git diff src/` empty). Measured at the shipped step:

| scenario | driver | season-low CO₂ | margin |
|---|---|---|---|
| `sealed_chamber` | `run_season`, 3 yr | 76.8196 ppm | 1.2579× |
| `perennial_chamber` | `run_perennial`, 5 yr | 75.4757 | 1.2359× |
| **`consumer_chamber`** | `run_perennial`, 5 yr | **74.4210** | **1.2186×** |
| `perennial_long_horizon` | `run_perennial`, 15 yr | 75.4757 | 1.2359× |
| `consumer_long_horizon` | `run_perennial`, 15 yr | 74.4210 | 1.2186× |

⚠ **The tightest of the five was never measured by any of the work that argued about
it.** Every write-up of this defect quoted the sealed chamber (75.75 / 76.82) or the
perennial one (56.03 / 75.48). The **consumer chamber sits below both**, and no record
carried its number until the band enumerated the roster. The probes that drove the step
decision swept the scenarios the *argument* was about; a band is about the *roster*, and
the two are not the same list. **Enumerate the contract's own subjects, not the ones the
discussion happens to have named** — which is this entry's locus lesson a second time, in
a form the correction above does not cover.

##### ⚠⚠ EVERY NUMBER IN THE TABLE ABOVE WENT STALE THE SAME DAY IT WAS WRITTEN — including which row is the tightest

The table and the docstrings that quote it landed in `4d7fdfd`. **Six commits later the
same day**, the within-day light path (`a0ef98b`) moved all five values 4–7 %
**downward**, and the layered canopy moved them a little further the next day. Re-measured
on `cc44b41`, with the band's own helper, each scenario driven the way its own golden
drives it:

| scenario | driver | season-low CO₂ | margin | vs the table above |
|---|---|---|---|---|
| `sealed_chamber` | `run_season`, 3 yr | 71.4358 ppm | 1.1697× | −7.0 % |
| **`perennial_chamber`** | `run_perennial`, 5 yr | **70.2526** | **1.1503×** | −6.9 % |
| `consumer_chamber` | `run_perennial`, 5 yr | 73.3386 | 1.2009× | −1.5 % |
| **`perennial_long_horizon`** | `run_perennial`, 15 yr | **70.2526** | **1.1503×** | −6.9 % |
| `consumer_long_horizon` | `run_perennial`, 15 yr | 73.3386 | 1.2009× | −1.5 % |

⚠ **The ranking inverted, which makes the paragraph above true in its lesson and wrong in
its subject.** *"The consumer chamber sits below both"* held for six commits: the consumer
chamber is now the **loosest** of the five and the **perennial** chamber is the tightest.
The two builds that moved it both act through canopy *closure* — a diurnal light curve and
then a depth-resolved canopy — and the consumer chamber is the one whose crop the crew's
CO₂ keeps furthest from closing, so it lost the least. The instruction — *enumerate the
roster, not the discussion* — is unaffected, and is why this re-measurement enumerated all
five rather than the two that had been argued about.

⚠⚠ **The sharpest part: the pin written for exactly this event DID fire, in the same file,
and the prose beside it was left standing anyway.**
`test_the_five_margins_are_pinned_not_merely_positive` exists because *"an inequality that
passes tells you nothing about how nearly it failed"*; the light path took it red and
`a0ef98b` re-pinned it (1.2579 → 1.1671, 1.2359 → 1.1543, 1.2186 → 1.2086) **and updated
its own comment to say so**. The docstrings four lines above it, and this document's table,
quote the superseded values to this day. So the gap is not that the tree lacked a guard —
the guard worked, on the first unfreeze after it was written. **A number in prose is not
guarded by a pin on the same quantity ten lines away**; re-pinning is a mechanical
consequence of a red test, and re-reading the prose is not.

⚠ **And the 15-year rows still equal the 5-year rows exactly**, so the property those two
entries were added to check ("the long horizon adds no new low") survived a change that
moved every value. That is worth more than the values: it is the *shape* of the run, and
it reproduced across an unfreeze that touched the canopy's whole light path.

⚠ **This is the third instance in two days**, and the third is the one that shows it is
structural: `cc44b41` fixed two `liveness_floors` source strings describing a superseded
equilibrium, and in the same commit a cross-port sensitivity probe that had gone vacuous.
All three passed while being false, because **the manifest gate compares marker ↔ manifest,
never marker ↔ reality**, and the freeze's prose half is ungated by design. The common
shape is not "a missing test" — in this instance the test existed and fired. It is that
**a value written into prose acquires no owner**, while a value written into an assertion
acquires one the moment it goes red.

⚠ **The horizon was checked rather than assumed, because it could have been false.** The
humification split pushed the chamber settling transient to ~35 yr, past every frozen
horizon, so "green on the golden" does not by itself mean "green at equilibrium". Run to
50 yr, both re-sowing chambers take their **global** minimum inside the frozen horizon
(perennial year 2, consumer year 5) and rise monotonically to a settled attractor
(75.84 / 75.06 ppm). The band's worst case is what the golden already runs.

⚠ **`Γ*`'s citation gap is discharged as a *risk*, not as a debt.** It is still
`TODO(cite)`, and caveat (a) above stands. What is new is that the gap is measured
harmless: the only route to the same quantity on the shelf — Teh eq. 6.19,
`Γ* = O₂/(2·τ)` with `τ = 2600` at 25 °C (Table 6.2) — gives **57.69 ppm, *below* the
shipped 61.07**. So the shipped floor is the conservative one and closing the citation
can only widen every margin above. That is asserted as a test, not as a sentence
(`test_the_shipped_floor_is_the_conservative_one_against_the_cited_route`). ⚠ It is **not**
a licence to swap the value: Teh's companion constants (`Kc` 300, `Ko` 300 mmol/mol)
disagree with ours (404.9 / 278.4), so the two are different parameterizations, and the
comparison is legitimate only because it moves the bound in the harder direction.

⚠ **The bound is derived at run time, never typed.** The literal `61.07` appears in the
test file only as a *tripwire* on the params (`test_the_floor_is_where_the_frozen_params_
put_it`), so a silent re-value of `Γ*` goes red instead of quietly moving five bounds at
once. The band itself is one-sided (`>`) on purpose — it must survive the next
mechanism's golden movement without being re-pinned — with the five margins pinned
loosely and separately, because *an inequality that passes says nothing about how nearly
it failed*.

### The flow set + the aux processes

The flow classes assembled across the canonical scenarios — the frozen flow taxonomy. The
manifest's `flow_set` is **derived from freshly assembled registries** (the union over the open
field + the three chambers), never hand-listed, so a flow added to any compartment builder is
caught by the completeness gate even if no golden exercises it. As frozen, the set is the 23
classes spanning the producer (allocation, the two respirations, senescence, transpiration,
**stem-reserve remobilization**, nitrogen uptake/senescence, the forcing-driven
irrigation/fertilization), the decomposer
(decomposition, microbial respiration, humus decomposition, and the three carried-nitrogen
legs), the water cycle (condensation, recycling, root-zone capture), and the consumer
(grazing, consumer respiration, consumer mortality), plus the soil's `Drainage`.

⚠ **23 since 2026-08-12: `StemRemobilization` was ADDED** by the stem-reserve build
(`docs/plans/post-roadmap-stem-reserves.md`), together with **one new stock**,
`stem_reserve_c`. ⚠ The count also absorbs a correction: this line read **21** while the
manifest held **22**, because `Drainage` was added earlier and the prose was not updated.
The manifest gate equates the manifest with the tree and **this document is not a side of
that comparison** — the standing gap, hit again.

⚠ **21 since 2026-08-11: `RootZoneCapture` was ADDED** by the soil-layers build
(`docs/plans/post-roadmap-soil-layers.md`), together with **one new stock**,
`subsoil_water`. It is the water side of rooted depth — [F] Soltani & Sinclair's `EWAT`
(Eqn 14.10), the transfer that makes water the roots have just reached available to them.
Unlike the depth gate on nitrogen, **this one is not inert**: it moved `soil_water` on 10
of the 12 goldens it touched (`harvest` and `water_biting` gained the stock but moved no
amount). What it did *not* move is any carbon, nitrogen or oxygen amount, anywhere, at any
horizon — a prediction written down before regeneration and checked against the diff.

⚠ **20, and the decomposer's shape is the reason** (post-roadmap, the humification split,
2026-08-10 — `docs/plans/post-roadmap-cue-humification.md`). Two classes were *added*
(`HumusDecomposition`, `HumusNitrogenRelease`) and two existing ones changed **currency**:
`Decomposition` was single-currency CARBON since Phase-2 Step 4 and is now CARBON+OXYGEN,
because the split gives it a CO₂ leg and the composition gate forces the O₂ draw that comes
with it. A currency change is invisible to `flow_set` — it freezes class *names* — which is
why it is written down here.

**Gross carbon assimilation is not a flow** (and not an aux): it is a recomputed *quantity*
inside the shared `CarbonContext` budget — the `GrossAssimilation` flow was *dissolved* in the
Phase-1 Step-11 buffer rewiring — entering the system through the `Allocation` flow's
`co2_atmos → organs` leg. So there is no `Photosynthesis`/`GrossAssimilation` class in
`flow_set`; that science is frozen via `Allocation`. The manifest also freezes the
**`aux_set`** (the registries' non-conserved accumulators, derived symmetrically from the
public `registry.aux_processes`) — the thermal-time / DVS accumulator that drives
allocation, (since post-roadmap scope (B) increment 1) the **vernalization-days**
accumulator that gates it, and (since 2026-08-11) the **rooted-depth** accumulator — so a
future aux process added but wired into no golden is caught too. (See `flow_set` /
`aux_set` in the manifest for the exact lists.)

⚠ **What `aux_set` does NOT catch: a change to what an existing accumulator DOES.** It
freezes the set of accumulator *classes*, so adding a third rate multiplier inside
`ThermalTimeAccumulation` (`WSFD`, 2026-08-12) leaves it untouched. That build changed the
frozen phenology science and moved nothing in this manifest at all — see the warning above
the unfreeze log.

⚠ **`aux_set` GREW 2 → 3 on 2026-08-11: `RootDepthExtension` was ADDED** by the root
functional coupling (`docs/plans/post-roadmap-root-functional-coupling.md`). It advances
rooted depth by [E]'s own law and gates `NitrogenUptake`'s supply term by the fraction of
the reference soil layer the roots have reached.

**Three things about this unfreeze are unusual enough to state here rather than only in
the plan doc:**

1. **It changed no value.** All 12 affected goldens differ by exactly one added `aux` key;
   not one stock amount moved, on any scenario, at any horizon. The manifest's golden
   hashes moved because the files gained a key, not because the science did.
2. **It was BUILT OVER A MEASURED REFUSAL, at the user's explicit direction.** The
   mechanism is bit-identically inert on the entire frozen roster — nitrogen uptake is
   demand-bound on every step of every scenario, and this gate shrinks supply. That was
   measured *before* the build, the work was recorded as refused on the
   canopy-regulator precedent, and the user overruled the refusal. The record says so in
   both directions; nobody should later discover the inertness and conclude it was missed.
3. **No golden can catch its removal.** Deleting the aux process or the gate factor leaves
   every golden green. `aux_set` catches the process disappearing; everything else is
   pinned in `tests/test_root_depth.py`, whose assertions are mutation-verified for
   exactly that reason.

### The param files — 15 clean-room biosphere param files

⚠ **15 since 2026-08-12: `stem_reserves.yaml` was ADDED** by the stem-reserve build —
four numbers whose provenance ranking is **inverted against what they do**. The only one
that moves much (`remobilizable_fraction` = 0.40) is the one [E] tabulates (Table 7,
wheat) and it is **CABO unpublished** data; the drain rate is uncited and measured
**bit-inert on carbon**; the trigger is **ours** ([E]'s "once stems stop growing" cannot
fire in this tree); and `cessation_dvs` is [E]'s own program boundary (`FINISH DS = 2.`)
rather than a value chosen inside it. It is plant-side, so it joined the **crop-param-set
vocabulary** (9 → 10 names), and **potato does not override it — it switches the mechanism
off**, because [E] Table 7 gives potato a range where wheat gets a point value.

⚠ **14 since 2026-08-11: `root_depth.yaml` was ADDED** by the root functional coupling —
two values, both first-hand from [E] Table 25 p. 137's "Wheat winter" row (Gregory et al.,
1978), read off the page image because that table's text layer column-collapses. It is
plant-side, so it joined the **crop-param-set vocabulary** (8 → 9 names) and potato
**overrides** it from Table 25's own potato row (Vos & Groenwold, 1986). Neither wheat
value is one of the table's flagged estimates.

⚠ **13 since 2026-08-10: `humification.yaml` was ADDED** by the humification split
(`docs/plans/post-roadmap-cue-humification.md`) — three CO₂ fractions that partition every
decomposer carbon flux, plus the slow-SOM pool's own rate, all four first-hand from CENTURY
(Parton et al. 1987). It is the first param file added since the freeze, and it exists
because the frozen form was asserting values **off the end of the source's own functions**:
a litter CO₂ fraction of 0.0 against a measured 0.45–0.55, and `Es = 1.0` where eq. [6]
`Es = 0.85 − 0.68·T` cannot exceed 0.85 at any texture. That is the shape bucket-3 scope C
found for the decomposer *rates*, one level down — the citation covered the **rate** and
never covered **where the decayed carbon goes**.

⚠ **It was 12, not 13, before that: `mineralization.yaml` was RETIRED**, and it is the first param *file* this
project has removed rather than re-valued. Both rates it ever held were discharged by
**form** changes rather than by finding citations — `n_senescence_rate` when N shedding
became coupled to the senescing carbon (option (A)), and `mineralization_rate` when the
return leg became microbe-mediated and stoichiometric (option (B)). With no parameter left,
the file went with them; its provenance record — five rounds of negative retrieval results —
is archived verbatim at `docs/retired/mineralization.yaml`, because a stale *negative* result
suppresses the next search and is the more expensive thing to lose.

`src/domains/biosphere/params/*.yaml` minus `demo.yaml`: `canopy`, `photosynthesis`,
`respiration`, `transpiration`, `phenology`, `allocation`, `senescence`, `nitrogen`
(Phase-1 producer); `decomposition`, `microbial_respiration` (Phase-2 decomposer — and,
since option (B), the N return loop too: its legs carry no rate of their own);
`water_cycle` (Phase-3 water closure); `herbivory` (Phase-3 consumer). Each is clean-room from primary literature
([`docs/param-file-conventions.md`](param-file-conventions.md),
[`docs/reuse-and-licenses.md`](reuse-and-licenses.md)) — **never** the unlicensed WOFOST YAML
or PCSE source. The manifest records a newline-normalized sha-256 of each as **provenance**.

> **Documented finding (carried, not hidden) — UPDATED by scope (B) increment 1
> (2026-07-20).** This note read *"phenology lacks vernalization, so the trajectory runs
> ~2 orders below the PCSE oracle"* until increment 1 added **vernalization** and
> **photoperiod** (clean-room from Soltani & Sinclair 2012; `phenology.yaml` grew 4→12
> params, all cited; a second aux accumulator was added). The magnitude gap **closed to
> ~1.22×** (peak LAI 5.19 vs the oracle's 6.34) and the canopy now bootstraps — with **no
> canopy science written**, because the "structural" canopy collapse was downstream of the
> phenology error (`docs/plans/post-roadmap-oracle-match.md`, findings). The **residual is
> now cause 3 — param values**: the `tsum` phase partition (reproductive phase too short).
> **Ceremony 2 (2026-07-20) investigated it and moved NO value** — both `tsum` values are
> already literature-centred (now cited to Penning de Vries 1989), and the oracle's longer
> grain fill is a *different cultivar*, so matching it would leave the cited range
> (backfitting, forbidden by ruling B — **the oracle is a diagnostic, never a fit target**).
> Only two `source:` strings changed; see the Unfreeze log. The frozen
> reference remains the **machinery** — balanced flows, the conservation gate, `rationed ==
> 0`, determinism, the emergent limit cycle (now a period-1 fixed point; see below) — with
> two real sciences added and a residual documented, not a validated oracle match.

### The driving forcing

The canonical scenarios are driven by the committed raw-weather fixture
`tests/oracle/winter_wheat_weather.json` (NASAPower facts; read as JSON, never via PCSE).
Tiling it `Y×` gives the multi-year horizons. Recorded in the manifest under `forcing`.

#### ⚠ UNFROZEN 2026-08-14: PAR varies **within** the day — the light path

The fixture is unchanged and its hash has not moved, but **the shape of the day has**. Until
now the day's radiation reached the crop as one daytime-mean PAR held flat across the whole
day, and the only thing in the model that knew the sun sets was the factor `× daylength_s`
inside the assimilation aggregator. Now `PAR_VAR` is a **within-day schedule**
(`domains/biosphere/light_path.py`): the cited sinusoidal path ([E], *"The path of radiation
intensity during the day is assumed to be sinusoidal"*), handed to the forcing seam as the
**analytic mean over each step's window**, and the aggregator integrates over one day of
seconds instead of over the photoperiod.

- **The day's photon dose is conserved exactly, at any step size** — `peak = (π/2)·mean` is
  the value that makes it so, and the window means are an exact partition of one integral.
  No parameter is introduced, no radiation created, `IRRAD` untouched. This is a
  redistribution, not a recalibration, and `tests/test_light_path.py` pins it.
- **The manifest gains `forcing.light_path`**, a sampled fingerprint of the shape — and it is
  the one hash in this contract that is **compared**, not merely recorded. The shape can be
  changed without touching any file the other hashes cover, so provenance alone would have
  left a hole the gate could not see.
- `daylength_s` **stops** being photosynthesis's integration window and **stays** the
  photoperiod signal phenology reads. Two readers became one.
- Why the step-window mean rather than the instantaneous value: measured, both. Sampling the
  sinusoid at the step-entry instant is sampling luck at this step (−8 % of the day's carbon
  on one day of the year, **+4 %** on another, and exactly **zero** at `dt = 1`). The window
  mean is monotone and dose-conserving. Full table:
  [`plans/post-roadmap-gross-net-gas-exchange.md`](plans/post-roadmap-gross-net-gas-exchange.md)
  finding 10.

**What it buys, and it is the reason it was built:** at PAR = 0 gross assimilation is exactly
0, so `MaintenanceRespiration`'s biomass-burning *shortfall* branch — built in Phase 2,
cited, conservation-balanced, and **never once executed** in this project's history, because
a day-averaged PAR made `GASS > MRES` at every step of every scenario — now runs. The sealed
chamber's CO₂ rises through the night and falls through the day, with O₂ its exact mirror at
PQ = 1. **No flow, stock or parameter was added to get it**; the gate was the forcing.

#### ✅ RESOLVED 2026-08-15 (was a KNOWN DEVIATION recorded 2026-08-14): the canopy floor is cleared by the step, not by the science

`science_bands.open_season` requires `5.0 < peak LAI < 8.0` ("real wheat peaks at ~5–8").
Under the light path the reference read **5.3806 at the shipped `dt = ¼` — inside the band**
— while the observable was still moving **15 %** between `¼` and `1/32`, with a converged
value of **4.7132, below the floor**. The band's own arithmetic was used throughout (its
test's `_run` / `_peak_lai`, not a lookalike probe), on 08-14 and again on 08-15.

| | `dt = ¼` (shipped) | `dt = ⅛` | `dt = 1/16` | `dt = 1/32` |
|---|---|---|---|---|
| before the light path | 5.5719 | 5.5896 | 5.5984 | 5.6028 |
| after the light path (the deviation) | **5.3806 PASS** | 4.8598 FAIL | 4.7278 FAIL | 4.7132 FAIL |
| **after the layered canopy + SLA anchor** | **6.0228 PASS** | 5.5699 PASS | 5.4406 PASS | **5.4273 PASS** |

⇒ **The deviation is retired: the band now clears at every step in the sweep, converged
value included.** The successor question this section named — *"the tree is short of a growth
mechanism it was previously compensating for with a flat sun"* — was answered, and the
answer was two things at once: a **sourced leaf-thickness constant** (`specific_leaf_area`
22.0 → 23.53 m²/kg, +7.0 %, Penning de Vries Table 19 "Wheat, winter") and a **depth-resolved
canopy** that lowers assimilation further. See the 2026-08-15 entry in the unfreeze log.

⚠ Two honest qualifications, neither of which reopens the deviation. (a) The **step
sensitivity is not gone** — the shipped `dt = ¼` still reads 11 % above the converged value,
so the shipped number remains the loosest point of the sweep. What changed is that the whole
sweep now sits inside the band, so no reading of the step turns the band red. (b) The floor
was cleared by a **provenance** move as much as by a mechanism, which is exactly what the
predecessor's finding 4 predicted would have the leverage here (peak LAI amplifies a uniform
parameter error ~3.5×). That cuts both ways: the two surviving `TODO(cite)` literals under
the same amplifier (`extinction_coef`, `carbon_fraction`) can move this observable just as
far, and are flagged in `canopy.yaml`'s header as the highest-leverage work left.

**The honest reading is that the band was clearing against a diurnally biased (high)
assimilation.** The loss is the concavity of the FvCB light response, not the new night
respiration: a control at the *same daily carbon* but with dark steps (a top-hat at the
daytime mean) costs only **1.1 %** of peak LAI, while the concavity costs **14.9 %** — a ~1 %
pointwise loss compounding four-fold because it lands on the phase that sets the canopy.
One candidate explanation is eliminated by measurement: the diagnosed-but-unbuilt canopy
regulator is a **5 %/day loss above LAI 6**, so it can only push the canopy further down.

⇒ **This is recorded, not tuned.** Retuning the bound so the change fits is the
co-adaptation this contract has refused three times. What the deviation says is that the
tree is now short of a growth mechanism it was previously compensating for with a flat sun —
the successor question, and it is not answered here.

⚠ **The other gates do NOT all move away from danger, and an earlier draft of this section
said they did.** Two open, two narrow:

| gate | before | after | reading |
|---|---|---|---|
| Greenwood peak-W crossing (14.4248 t/ha) | 2.20 % margin | **4.75 %** | opens |
| CO₂ compensation point, all five chambers | 1.219–1.258× floor | **1.154–1.209×** | narrows, all clear |
| perennial converged peak-leaf, `max(tail) > 0.55` | 0.612211 (+11.3 %) | **0.603679 (+9.8 %)** | narrows |
| perennial annual-min chamber CO₂, floor 0.05 | 0.075476 (+51.0 %) | **0.071036 (+42.1 %)** | narrows |
| the two peak-leaf non-collapsing floors + consumer year-end carbon | ≫ | **+1048 % / +1195 % / +5343 %** | nowhere near |

⚠ **The perennial peak-leaf series is the one to watch, and not at the bound the manifest
states.** `max(tail)` clears 0.55 by 9.8 %, but the tail *declines* and its settled last
value is **0.567715 — 3.2 % over the floor**, down from 4.9 % at the step change and 8.0 %
before it. **Three consecutive moves in the same direction, from unrelated causes.** A
liveness floor anchored on a measured equilibrium is being approached, not held; whoever
next raises or lowers assimilation should read this line before assuming room.

### The canonical scenarios + their goldens

Phase 4 invents **no new scenario** (P4.2 — capture, not invention). The reference is the four
Phase-3 goldens re-affirmed + the three Phase-4 long-horizon artifacts:

| Scenario | Knobs | Years | Golden |
| --- | --- | --- | --- |
| open season | `DEFAULT_SCENARIO` (open field) | 1 | `season_euler_state.json` |
| sealed chamber | `SEALED_CHAMBER_SCENARIO` | 3 | `sealed_chamber_state.json` |
| perennial chamber | `PERENNIAL_CHAMBER_SCENARIO` | 5 | `perennial_chamber_state.json` |
| consumer chamber | `CONSUMER_CHAMBER_SCENARIO` | 5 | `consumer_chamber_state.json` |
| perennial (long-horizon) | `PERENNIAL_CHAMBER_SCENARIO` | 15 | `perennial_long_horizon_state.json` |
| consumer (long-horizon) | `CONSUMER_CHAMBER_SCENARIO` | 15 | `consumer_long_horizon_state.json` |
| drift summary | both long-horizon runs (stability signature) | 15 | `drift_summary.json` |

The long-horizon length is `LONG_HORIZON_YEARS = 15` — a single importable constant
(`scenario.py`) shared by the long-horizon golden, the decade probe, and this manifest, so the
frozen horizon cannot drift. Each golden is a hex-float byte snapshot via `sim_io` (the
`drift_summary` is the per-year peak-`leaf_c` / year-end `consumer_carbon` vectors + the period
class). They are bit-identical **within a build**; the season uses transcendentals
(`exp`/`pow`/`sin`), so cross-platform last-ULP differences are **tolerance territory** (the
cross-port concern), not a freeze violation.

## The evidence the freeze rests on

The freeze is earned by Phase 4 Steps 1–4 (full detail + measured numbers in the plan):
- **Conservation holds over decade-scale runs.** Total CARBON/OXYGEN/NITROGEN/WATER stay under
  the structural ceiling (`≤ N·BALANCE_ATOL`) and the round-off-scale slope detector (no
  systematic growth) at 15 yr and at the 100k-step (328 yr) stress — mass-drift slope flat at
  machine-ε, deterministic round-off, not a leak.
- **The emergent limit cycle is stationary** (bounded, non-amplifying, non-collapsing) the
  whole horizon. ⚠ Both chambers are now **period-1 fixed points** (scope (B) increment 1):
  the perennial's old **period-2** cycle was a property of the *broken canopy regime*, and
  closing the canopy (vernalization + photoperiod) flattened the year-to-year return map
  below unit gain, so the 2-cycle lost stability and converged upward — measured, either
  phenology term alone suffices (`docs/plans/post-roadmap-oracle-match.md`). The consumer was
  always period-1 (the herbivore damps the producer oscillation).
- **Closure carried every step:** `rationed == 0` (kinetics, not the Euler backstop),
  `events == ()` (no extinction), carbon loss-sink `0.0` (death routes to litter) — on every
  one of the 100,040 stress steps, both scenarios.
- **The reference integrator + dt are locked** (Euler, `dt = 1`) — RK4 cross-check retired the
  escalation preconditions and structurally agreed, so Step 2 (escalation) was skipped.

Tests of record: `tests/test_decade_stability.py`, `tests/test_biosphere_stress.py`
(marked-slow), `tests/test_drift.py`, `tests/test_regression_long_horizon.py`, and the four
Phase-3 scenario regression tests.

## The manifest

`docs/biosphere-reference.manifest.json` is the machine-readable surface, **generated** by
`tests/test_freeze_manifest.py` (`uv run python tests/test_freeze_manifest.py`). It names the
integrator + dt, the horizon, the derived flow set + aux set, the param files (+ provenance
hashes), the forcing (+ hash), and each scenario → golden (+ hash).

### ⚠⚠ Who produces it — MIXED AUTHORITY since 2026-08-16 (reference-flip slice 6)

The keys the **Rust reference tree** can produce are now read from it, by shelling
`cargo run --example dump_biosphere_inventory`; the rest are still the checker's. **The file
says which is which, per key, in its own `_authority` block** — so a reader cannot mistake a
Python-retained field for a Rust-derived one. Two consequences:

- **Regeneration needs `cargo`.** The gates do not: nothing in `test_freeze_manifest.py`
  shells cargo, and the base suite stays offline-clean. The cargo-side gates (is the manifest
  *stale* against the reference tree; does the frozen `dt_days` literal still match the
  reference's `BIO_DT`) live in `tests/crossport/test_inventory_parity.py`.
- **The completeness gates changed meaning without changing their arithmetic.**
  `set(manifest["flow_set"]) == set(_flow_set())` used to say *the manifest froze everything
  Python has*; it now says *Python still matches the reference*. A failure there is a **Python**
  drift and is **not** fixed by regenerating.

⚠ **"The manifest is Rust-anchored" was the wrong summary until slice C4 (2026-08-18), and
this paragraph is the record of what changed.** It read: *"By content most of it is still
Python's — `science_bands` + `liveness_floors` alone are about half the file and are a static
census of pytest markers, with no Rust referent while the science gates are pytest-side."*
The subordinate clause was the load-bearing one, and C4 removed its premise: the 13 biosphere
science gates are declared in `rust/crates/domains/src/biosphere/science_gates.rs`, where a
macro emits the roster row and the `#[test]` that executes it as **one declaration**, so the
census is derived from the tree in Rust without there being anything to parse. What is still
the checker's, per key, is in the table below — read it rather than a summary sentence.

| Key | Producer | Why |
|---|---|---|
| `flow_set`, `aux_set` | **Rust** | the union of `type_name()` over the four canonical builds |
| `forcing.light_path` | **Rust** | its samples; **measured** byte-identical to Python's before re-anchoring, because this key is gated exactly rather than tolerance-bound |
| `long_horizon_years`, `scenarios.*.years` | **Rust** | the reference tree's horizon constants |
| `scenarios.*.golden_sha256` | **Rust** (6 of 7) | the golden is the reference's own output |
| `scenarios.drift_summary.golden_sha256` | **Python** | ⚠ one run, two authors: `drift.py`'s fold of the *same* 15-yr trajectory whose final state Rust authors. The fold is the artifact |
| `param_files` | **Rust** (since slice C8) | ⚠ the *rules* re-anchored, not the digits: the census is now the set the reference LOADS (a compile-time `include_str!` list) and the digest is `config::provenance`. The 15 values are **author-neutral** — both sides hash the same file the same way — so the ceremony moved none of them |
| `forcing.weather_fixture` / `weather_sha256` | **Python** | ⚠ the reason changed in slice C9 (2026-08-17) and the old one is now false: the port no longer reads a file *generated from* this fixture — it reads **this fixture**, with a compile-time `include_str!`. It stays Python's because `include_str!` takes a literal, so the reference knows the fixture's **bytes and not its name**; a Rust-authored filename would be a hand-typed duplicate of the include path, a literal dressed as a derivation. The *bytes* half is now cross-checked anyway — see the `weather_sha256` note below |
| `science_bands`, `liveness_floors` | **Rust** (since slice C4) | ⚠ the *claims* re-anchored, not the values: all 13 `quantity`/`bound`/`source` strings are byte-identical to the Python census's and every verdict was measured identical on both ports first — only the 13 `locus` strings moved. The **key set** is still this manifest's own hand roster (which scenarios get an entry, and which get an explicitly empty list meaning "measured, none"), and a Rust gate naming a scenario outside it **raises** during regeneration rather than being filtered away. ⚠ Two markers did not move: `crew_mission` and `sealed_station` are *station* keys whose referents the reference does not carry yet — slice C4b |
| `integrator`, `dt_days` | **hand** | the two deliberate anti-derived literals (below) |
| `scenarios.*.scenario` / `.golden` | **hand** | a human label; a filename |

⚠ **`golden_sha256` is now the one hash class that IS compared** against the files on disk.
Slice 5 measured the hole it closes: regenerating a frozen golden desynchronised this manifest
with **every manifest gate green**. Deliberately *goldens only* — the param hashes stay
provenance, because they are hand-edited files whose values the goldens already enforce,
while a golden is machine-generated and **is** the value.

⚠ **`weather_sha256` left that provenance-only class in slice C9** (2026-08-17), because C9
gave it a second side to be compared *against*. The reference now embeds the fixture with
`include_str!`, so it can hash the text it **compiled in**; the checker hashes the file it
finds **on disk**; `test_the_weather_hash_matches_the_reference_tree` fails if the two ever
stop being the same bytes. This is the `locked_dt_days` shape — the reference emits the value
**to be checked and never spliced**, so the key does not re-anchor. It guards a scheduled
hazard rather than a hypothetical one: C9's `include_str!` reaches out of the Rust tree into
the Python one, and the relocation slice will move that file. A control confirmed the teeth —
one digit changed in one `TEMP` value reddens exactly this test.

⚠ **The two anti-derived literals are unchanged, and `dt_days` gained the half it was
missing.** Neither is imported from the code — a contract field that imports its own value
auto-follows the code, and the 2026-08-14 step move became a ceremony only because that
literal went red. What slice 6 added is the *other* direction: the frozen `dt_days` is now
checked against the **reference tree's** `BIO_DT`, so moving Rust's step without the ceremony
is red rather than silent. `integrator` did **not** get the same treatment: there is no
importable scheme name on either side, so a `"EulerIntegrator"` string typed into the Rust
dump would be a second hand-written literal checked against the first — which reads like a
gate and is none. It stays enforced by the goldens.

**What the manifest gate checks vs. what the goldens check** — the division is deliberate:
- **The scenario goldens own *values*.** Any value change to a frozen param file, a flow law,
  the integrator/dt, or the weather fixture already moves a committed golden and fails its
  byte-compare. The manifest does **not** re-assert that (it would be redundant, and a raw
  byte hash of hand-edited YAML is not reproducible under `autocrlf`). The manifest's hashes
  are **provenance only** — a re-derivable, newline-normalized record of *which content* was
  frozen, regenerated on a deliberate unfreeze.
- **The manifest gate owns *completeness*** — the one thing the goldens are blind to: a param
  file, flow class, or aux process added to the tree but wired into no golden. The gate asserts
  the frozen *sets* (param files, flow classes, aux classes) against the live tree and the
  horizon against its constant — and a teeth test confirms it actually fails on an unfrozen
  file. A new-but-unfrozen param/flow/aux fails the gate; that is the signal to either freeze it
  (an unfreeze) or remove it.
- **`science_bands` + `liveness_floors` own the *science*** — added 2026-08-09, and the reason
  they exist is that until then **nothing did**. The frozen acceptance set was {golden bytes,
  `rationed == 0`, no extinction, conservation, determinism}: every one a property of the RUN.
  Assertions about the *science* — that `open_season`'s canopy is a real wheat canopy, that the
  closed chamber's CO₂ attractor has not collapsed — sat in test files reachable from no
  manifest, so none could fail an unfreeze ceremony. Both fields are **derived** rather than
  hand-listed, so they carry the same completeness teeth as the flow set — and ⚠ **how** they
  are derived changed in slice C4 while that requirement did not. Python met it by parsing
  `science_gate` decorators out of the test tree with `ast` (`tests/science_gates.py`); the
  reference meets it by making the roster row and the test **one declaration**
  (`rust/crates/domains/src/biosphere/science_gates.rs`), so an unexercised entry is a compile
  error rather than something a meta-test has to hunt for textually. The Python census
  survives for the two *station* gates only, until slice C4b.

  ⚠ **A deleted claim is a different failure from an unexercised row, and only one of them is
  structural.** The macro makes the second impossible; the first is caught by the manifest
  comparison in `tests/crossport/test_inventory_parity.py` — measured, not assumed: deleting a
  gate declaration leaves the whole Rust suite **green** (the test simply ceases to exist) and
  turns that gate red naming the missing scenario.

  **The two are deliberately separate names, because they are claims of different strength.** A
  `science_bands` bound comes from **outside this repo** (real wheat's ~5–8 LAI, Van Keulen &
  Seligman's 6.0 shading threshold, Greenwood's 14.4248 t/ha crossing, BVAD's RQ). A
  `liveness_floors` bound was **tuned to our own calibration** — the perennial plant floor moved
  `> 1.0` → `> 0.9` when the decomposer calibration shrank the plant ~19 % — so it guards
  *continuity with the current calibration*, **not** physical plausibility. Freezing a floor
  under the bands' name would say "the frozen tree passes a bound the frozen tree set".

## The unfreeze discipline

Changing **any** frozen item — a param value, a flow, a scenario knob, the integrator or dt,
the horizon, or adding a new param/flow to the domain — is an **unfreeze**. The procedure (the
Phase-1 PCSE/clean-room provenance rigor, applied to our own reference):

1. **Justify + review.** Write down *why* (a calibration source, a new process, a bug). For a
   science or numerical change, get it **advisor-reviewed** before regenerating anything — the
   project's standing rhythm.
2. **Make the change** boundary-side. `git diff src/simcore/` **must stay empty** —
   unconditionally. (Even an RK4 escalation is a domain-side instantiation choice; there is no
   unfreeze path that edits `simcore/`.)
3. **Regenerate the affected goldens** and **review the byte diff** — a change there means the
   trajectory moved, which is the point. ⚠ **Six of this contract's seven goldens are the Rust
   port's output** since the reference flip, and the blessed path for them is
   `uv run python tests/crossport/regen_goldens_from_rust.py --write`; the per-module
   `__main__` in `tests/test_regression_*.py` **refuses** to write one (`golden_platform.
   write_python_golden`). `drift_summary.json` is the exception — it is Python's fold and its
   own `__main__` is still right.
4. **Regenerate the manifest** (`uv run python tests/test_freeze_manifest.py`) and review its
   diff — the changed hashes / flow set / param set are the git-visible record of exactly what
   was unfrozen. ⚠ **This step now needs `cargo`**: since 2026-08-16 the manifest reads the
   Rust reference tree for every key its `_authority` block marks `rust`. So regenerating on a
   box without a Rust toolchain fails loudly rather than writing a Python-derived manifest.
5. **Report the science gates.** State the change's readings against the scenario's
   `science_bands` and `liveness_floors`. A band failure is a **blocking finding** that must be
   argued past in writing, not a number to re-tune — retuning a bound so a change fits is the
   co-adaptation shape this project has refused (the consumer-chamber 2×, the DPM/RPM labile
   re-read, ruling B). ⚠ And the converse is not licensed either: a band **passing** is not an
   endorsement. `open_season` sits **3.8 %** above the LAI lower bound and **12 %** below the
   Greenwood crossing — these are tight margins, not comfort.
6. **Record provenance.** Update this file and the Phase-4 plan with what changed and why (a
   calibration cites its primary source per `docs/param-file-conventions.md`).
7. **Re-run the gates:** full suite (incl. `-m slow` for the stress), `ruff`, `pyright`; commit
   with a Conventional Commit that names the unfreeze.

⚠ **"An undocumented unfreeze fails CI by construction" is NOT true in general, and the
2026-08-12 `WSFD` build is the counter-example.** The claim used to read that way here. It
holds for an unfreeze that moves a frozen golden or changes a frozen *set* — but a **form**
change can do neither: `WSFD` added a third multiplier to the thermal-time rate, and because
it is the exact multiplicative identity wherever water does not limit, it moved no frozen
golden, no `flow_set`, no `aux_set`, no `param_files` entry, and therefore **not one byte of
the manifest**. Both automatic gates were blind to it. This joins the provenance-only edit
CLAUDE.md already warns about as a second door into the same room: the ceremony below is
honor-system for such a change, so follow it deliberately rather than waiting for a red test.

### Unfreeze log

⚠ **Every entry below is a DATED RECORD, true of the day it was written, and is not
maintained afterwards.** Entries measured against scenarios that no longer exist keep their
numbers: `n_limited` and `water_biting` were retired on 2026-08-18 (see the scope section
above), so a present-tense sentence naming them — "the golden that moved", "one of only two
runs where water limits", a golden count of 25 — describes the tree **as it was at that
entry's date**. Rewriting them would falsify the measurement; only the *scope* statements at
the top of this doc, which are live claims, are kept current.

- **2026-08-18 — the SCIENCE-GATE CENSUS re-anchors to the reference (a LOCUS-only unfreeze;
  13 `locus` strings and two `_authority` notes moved, and not one recorded claim).**
  Reference-flip slice C4. `science_bands` + `liveness_floors` were the largest
  Python-authored block of any contract — about half this manifest by content — and this
  doc's own producer table said they had "no Rust referent". They now come from
  `rust/crates/domains/src/biosphere/science_gates.rs`.

  **What moved and what did not.** All 13 `quantity`/`bound`/`source` strings are
  byte-identical to the Python census's; the diff is exactly 13 `locus` strings plus the two
  `_authority` entries. Every gate's verdict, and every quantity it reads, was measured
  identical on both ports **before** the port was written (§5j of
  `docs/plans/post-roadmap-reference-flip.md`: 39 keys, 37 byte-identical, the two that differ
  being the same 4 values slice C5 already owns at 7 ULP, with every band's margin orders
  above them). The gate values were re-measured after the port and match that probe to 17
  significant digits, in the **debug** profile the permanent gates run in.

  **Why a census could change language at all.** The requirement was never "parse the tree" —
  it was **derived, never hand-listed**. Python met it with an `ast` walk over
  `@pytest.mark.science_gate` decorators; Rust has no such introspection, so a table plus
  tests would have been a hand-maintained roster. The macro makes the roster row and the
  `#[test]` **one declaration**, which is stronger: an unexercised entry is a compile error
  rather than something a meta-test hunts textually.

  ⚠ **One parametrized Python test carried TWO markers.** In the reference the row *is* the
  test, so it became two tests with two loci — same claims, same numbers, one more locus
  string. That is the whole of why the diff is 13 loci and not 12.

  ⚠ **Two of the 15 gates did NOT move, and it is a split rather than a deferral.**
  `crew_mission` and `sealed_station` are *station*-manifest keys whose referents the
  reference does not carry (the RQ helper; `predicted_equilibrium_temperature`). Slices 6–8
  re-anchored one manifest per slice on purpose. They are slice **C4b**, scheduled.

  ⚠ **A near-miss worth the whole entry: the first regeneration wrote MOJIBAKE into this
  contract and nothing was red.** The reference emits UTF-8; `subprocess.run(text=True)`
  decoded the pipe with the Windows locale's cp1252, so `—` was frozen as `â€"` and `Γ` as
  `Î"`. Every gate stayed green, because the manifest and the checker agreed — the corruption
  happened on the way in. It was caught by **predicting the diff before regenerating**
  (13 loci, two notes, zero value changes) and finding 37 changed lines. Both readers now pin
  `encoding="utf-8"`; losing it on one side is red in the crossport parity gate, and losing it
  on **both** — which compares equal — is caught by
  `test_the_frozen_claim_text_survived_the_pipe`, which asserts the characters themselves.
  *Every byte this dump emitted was ASCII until this slice, which is why the pipe's encoding
  had never mattered.*

  ⚠ **A pin elsewhere went red, and that was the pin working.**
  `test_acceptance_gate.py::test_the_plausibility_bands_are_now_named_by_a_manifest` asserted
  that the manifest names `test_senescence_form` and `test_nitrogen_form` — the Python files
  the loci pointed at when the standing was granted. It was **re-pointed at the new loci, not
  relaxed**, which is the treatment its own docstring records for the pin it replaced.

  **Negative controls, each reddening a different line and nothing else:** delete a gate
  declaration → the Rust suite stays **green** (the test ceases to exist) and the crossport
  parity gate reddens naming the missing scenario, which is why the manifest comparison and
  not the macro is what catches a *deleted claim*; drop one sample from one pre-reduced series
  → **exactly one** gate red, the annual-summary count (the CO₂ band does not notice, because
  its minimum is not at the end); drop the observer's initial state → all 13 red; point a gate
  at a scenario outside the roster → regeneration **raises** and writes nothing; re-mark a
  biosphere test in Python → the two census gates red from both directions.

- **2026-08-17 — `allocation.yaml` is REFORMATTED out of YAML flow style (a FORMAT-only
  unfreeze; no value moved, and nothing could have caught it).** Reference-flip slice C1, the
  slice that moves param loading into the Rust reference. The partition table was written as
  `- {dvs: 0.0, fl: 0.55, …}`; it is now block style, one key per line. **Only whitespace and
  line breaks changed — every number is character-for-character what it was.**

  **Why a data file had to change at all.** The reference's YAML reader (`crates/config/src/yaml.rs`,
  hand-rolled over a documented closed subset since Phase 9) **excludes flow style by design**.
  So the closed subset this project froze for *authored* files did not cover **this project's own
  param files** — measured before the slice was designed, and the deciding number is that flow
  style appears in **exactly two files** (`allocation.yaml` and the potato override) and in **zero**
  authored scenarios. Widening a frozen grammar to accommodate two data files it was never asked
  about was the larger change and the wrong one.

  **The value gate, which is what makes "format-only" checkable:** `gen_biosphere_params.py`
  reproduces `biosphere_params.txt` **byte-for-byte** after the reformat, so no frozen number
  moved anywhere. **The manifest diff was predicted before regenerating and held exactly:** one
  line, `param_files["allocation.yaml"]`'s sha-256. No golden hash moved; no golden was re-run.
  `crops/potato/allocation.yaml` was reformatted the same way and is **invisible to this
  contract** — `_frozen_param_files()` is a non-recursive glob minus `demo.yaml`.

  ⚠ **This is the provenance-only shape again, so the ceremony was run deliberately** (advisor
  review → regenerate the manifest as the git-visible record → this entry): the `param_files`
  hashes are **recorded and never compared**, so the reformat turns nothing red.

  ⚠⚠ **The control found a real defect in the frozen reader, and it is worth reading twice.**
  A negative control asserting *"flow style is rejected, not silently mis-parsed"* **failed** —
  the reader rejected flow style as a mapping **value** (`a: {b: 1}`) and silently mis-parsed it
  as a **sequence item**: `- {dvs: 0.0, fl: 0.55}` has a `key:` head, so the mapping path yielded
  the key `"{dvs"` with the value `"0.0, fl: 0.55}"` and **no error at all**. The test named
  `flow_style_is_rejected` had covered only the value form for its whole life — i.e. it missed the
  one form this repository's own param files were written in. Fixed on the key side with the
  excluded-leader set shared between both guards, the regression case added **to that test**
  rather than a crate away, and confirmed inert on every existing file (all three authoring
  integration binaries green). *A test that names a behaviour is not evidence it covers the case
  that matters.*

- **2026-08-16 — the manifest is RE-ANCHORED to the Rust reference (a PRODUCER unfreeze; no
  frozen value moved).** Reference-flip slice 6. This is the first entry in this log that
  changes **who writes the contract** rather than what it says: `flow_set`, `aux_set`,
  `forcing.light_path`, `long_horizon_years` and every `scenarios.*.years` are now read from
  the Rust tree, and the file carries an `_authority` block naming the producer of every key
  (table above). **The diff was predicted before regenerating and held exactly: the only
  changes are the new `_authority` block and the `_comment`.** Not one frozen value moved —
  the sets were already identical (slice 3), and the light-path fingerprint was *measured*
  identical, sample for sample, **before** re-anchoring, because that key is gated exactly and
  a prediction would not have been evidence.

  **What makes this an unfreeze rather than a refactor**, since nothing scientific moved: the
  contract's *source of authority* changed, and a future reader who assumed the whole file was
  Python-derived would draw wrong conclusions from it. Also landed with it: `golden_sha256` is
  now compared against the files on disk (closing the hole slice 5 measured, where regenerating
  a frozen golden desynchronised this manifest with every manifest gate green), and the frozen
  `dt_days` literal is now checked against the reference tree's `BIO_DT`.

  ⚠ **The re-anchoring is partial by content, and that is stated rather than glossed:**
  `param_files` (until slice 9 decides who loads the YAML), the weather fixture + its hash, the
  `drift_summary` golden's hash, and the whole `science_bands` / `liveness_floors` census —
  roughly half the file — remain Python's, each with its reason written beside it in the
  manifest itself. *(`param_files` left that list on 2026-08-17; see the C8 entry below.
  The weather fixture is still on it, but its **reason** was replaced the same day — see C9.)*

- **2026-08-17 — `param_files` RE-ANCHORED TO THE REFERENCE (slice C8 of the flip). Not one
  hash moved, and the reason is the finding.** The key's 15 digits are **author-neutral by
  construction**: both trees compute a newline-normalized sha-256 of the same file under the
  same rule, so *"`param_files` is now Rust's"* is the wrong summary and the ceremony was
  predicted to be value-free before it was run (it was). What re-anchored is a pair of
  **rules**:

  1. **The census** — the manifest now names the files the reference *loads*
     (`domains::biosphere::params::param_files`, a compile-time `include_str!` list) instead of
     the files a **glob of a Python package directory** finds. The difference is directional and
     it is the point: a param file added to the tree and wired into no loader used to *enter*
     the frozen surface; now it drops out of it and the gate says so. The 15-of-20 rule survives
     with both of its exclusion reasons intact (four `crops/potato/*.yaml` by **non-recursion**,
     `demo.yaml` by **name**), asserted Rust-side against the directory — and a free negative
     control ships with it, because a recursive walk picks the potato overrides up and **two of
     them share a basename with a frozen file**.
  2. **The normalization** — `config::provenance`, a hand-rolled sha-256 (every engine crate is
     zero-dep by charter) over LF-normalized text. ⚠ That rule is load-bearing **today**, and
     not for the reason one would guess: `git ls-files --eol` shows the index is LF on **all 24**
     param files, but the working-tree copy of `senescence.yaml` on the development box is
     **CRLF** — and `include_str!` embeds the working tree. Measured: the un-normalized digest
     for that one file differs from the frozen value, so without normalization the reference
     would emit different hashes on that box and on Linux CI. "autocrlf converts on checkout"
     would have been the wrong story; it would have hit all 24.

  Python's `_frozen_param_files()` and `_normalized_sha256()` are **retained with their meaning
  inverted** — the identical assertions now ask *has the checker drifted from the contract?* —
  which is the treatment slices 6 and 8 gave the flow set and the authoring rosters. Deleting
  them would have thrown away the only thing that says the two rules still agree.

  ⚠ Prerequisite: **slice C1**, which moved the YAML loaders into the reference. Before C1 this
  key genuinely had no referent, and the three slices that recorded so were right for their day.

- **2026-08-17 — the weather path moved into the reference (slice C9 of the flip). NOT an
  unfreeze: no key re-anchored and no value moved; this entry exists because two sentences
  elsewhere in this document became false.** The reference used to read
  `weather_facts.txt`, a hex-float table that `tests/crossport/gen_biosphere_weather.py`
  lowered out of the JSON fixture. It now reads `tests/oracle/winter_wheat_weather.json`
  itself, through a closed-subset JSON reader and an ISO-date→day-of-year helper in the
  `config` crate; the generator, the table and the Python sync gate are deleted.

  **Bit-neutral, and measured before any Rust was written**: all 916 values parse to identical
  bit patterns, because `f64::from_str` and CPython's `float()` both round correctly. The two
  sentences repaired are the authority-table row for `forcing.weather_fixture` (which said the
  port reads a file *generated from* the fixture) and the "goldens only" note above (which
  listed the weather hash as never compared). Both are corrected in place.

  ⚠ **The half this contract cannot check is the calendar, not the floats — and that is a fact
  about the committed data, not about the code.** The fixture runs 2006-10-01 → 2007-08-01:
  neither year is a leap year and the span contains no 29 February, so a wrong leap rule
  produces byte-identical output on **every row of the fixture and every golden in the tree**.
  Measured with the rule broken to the naive `year % 4 == 0`: the whole `domains` suite came
  back **48 passed / 0 failed**, including the bit-for-bit control against the old table and
  every season run. Only hand-computed unit tests carrying 1900 and 2000 — dates the fixture
  cannot reach — catch it, and `config::date` carries them with that reason in its header.

- **2026-08-15 — `canopy.carbon_fraction` BOUND TO A CITATION (provenance only; one hash, no
  value, no golden).** The honour-system ceremony from `CLAUDE.md`: advisor review →
  regenerate the manifest as the git-visible record → this entry. **Nothing could go red, by
  construction** — the per-file sha-256 is recorded and never compared, and 0.45 did not move.

  **Why it is an unfreeze at all.** `carbon_fraction` was one of two `TODO(cite)` literals the
  layered-canopy build named as *"the highest-leverage provenance work left on this
  observable"*. It is now cited to Raimanova et al. (2024) — measured wheat C at 45.05 % in
  grain and 45.66 % in straw, the paper's own words being that these are *"near to the value
  of 45 % used for the calculation of C content in plant dry mass"*.

  ⚠ **It was never missing.** `nitrogen.yaml` has carried that citation, for the identical
  value, since the 2026-07-16 citation round — under a **MUST-EQUAL** constraint with
  `canopy.yaml` that both files document. And `crops/potato/canopy.yaml` has stated since
  2026-08-11 that *"the reference value IS cited, but to a measurement of wheat grain/straw"*
  — describing this file, which still said `TODO(cite)`. **A crop override was the record that
  the reference was sourced.** Fourth instance of the shape (`canopy-regulator-diagnosed`,
  `stem-reserve-form-is-on-the-shelf`, and the SLA anchor in `test_potato_crop.py` one day
  ago): *check your own shelf before treating a value as unsourced.*

  ⚠ **What is genuinely new here, and is not a copy:** the *basis*. `nitrogen.yaml` applies the
  fraction whole-plant and records a root delta ([C] measures roots at 34.9 %, ~10 points below
  shoots, overstating root C by ~29 %). This file applies it to **leaf blade**, so the straw
  figure — the above-ground vegetative measurement — is the nearer of the paper's two shoot
  readings and the root delta does **not** transfer. The same citation binds *more* tightly
  here than where it was first written down, and copying the source string verbatim would have
  imported a caveat that does not apply.

  **Left open, deliberately:** `extinction_coef` is now the file's only `TODO(cite)`, and the
  shelf holds **three disagreeing readings** of it (0.60 / 0.65 / 0.68), which is a value
  question rather than a retrieval one. Measured, priced and left for the user in
  [`plans/post-roadmap-canopy-provenance.md`](plans/post-roadmap-canopy-provenance.md).

- **2026-08-15 — THE LAYERED CANOPY + THE LEAF-THICKNESS ANCHOR (11 goldens, both manifests,
  one science band RESTATED, and the native port).**
  `docs/plans/post-roadmap-canopy-magnitude.md` §7b; full record `docs/log/layered-canopy.md`.
  Authorized by the user on that plan's open decision (*"build the layered canopy anyway as
  honest physics (it's coherent alongside the thickness fix, and that combination is the only
  one where it doesn't eat the margin)"*), and twice more as its price emerged.

  **Why.** The predecessor pass refused a depth-resolved canopy *as a fix* — it moves peak LAI
  the wrong way — while stating plainly that it is the better physics. The user took it as
  physics rather than as a fix, paired with a sourced leaf-area constant so the science band
  keeps its margin.

  **What changed, three mechanisms.** (i) `photosynthesis.canopy_assimilation` integrates over
  canopy depth with Goudriaan's 3-point Gaussian (abscissae `0.5 ± 0.5·√0.6`, weights
  `5/18, 8/18, 5/18`), absorbed PAR at depth `L` being `k·I₀·exp(−k·L)`; the big-leaf
  aggregator is gone. Both halves of the Jensen bias are now closed. (ii) `canopy.yaml`'s
  `specific_leaf_area` **22.0 → 23.53 m²/kg**, bound to Penning de Vries et al. (1989) Table 19
  p. 100, "Wheat, winter". ⚠ This retired a `TODO(cite)` **and moved the value +7.0 %** — a
  calibration inside the goldens, not the provenance-only shape. (iii) A new
  `allocation.mutual_shading_rate` and two cited `senescence.yaml` params (`shade_rate = 0.05
  /day`, `lai_threshold = 6.0`) — Van Keulen & Seligman 1987 via Penning de Vries p. 101.
  No flow or stock was added or removed; `git diff src/simcore/` stayed empty.

  **The band restatement, and why it is not a re-tuning.** (i)+(ii) took `open_season`'s peak
  LAI to **6.0228**, comfortably inside "5.0 < peak < 8.0" but 0.38 % over the *separate*
  sourced check `peak LAI < 6.0`. Rather than move 6.0 to fit, the mechanism that threshold was
  standing in for was **built**, and the band restated as **"peak < 6.0 OR the 5 %/day
  mutual-shading loss is MODELLED"**. ⚠ The loss is currently **inert** — 6.0228 either way,
  bit-identically — and the test name and manifest source say so. The regime is now represented
  rather than avoided.

  **What moved.** All 7 biosphere goldens and the 4 plant-bearing station goldens; the
  plant-free station goldens are byte-identical, which is the structural check that nothing
  leaked. `canopy.yaml` + `senescence.yaml` hashes; `biosphere_params.txt` +3 lines.

  **The gate report — every band AND every liveness floor, as the ceremony requires.**
  **All 15 science gates GREEN.** Five CO₂ compensation-point bands (sealed / perennial /
  consumer chambers and both long horizons) pass. Four frozen liveness floors pass
  (`non_collapsing(0.05)` peak-leaf on both long horizons, `non_collapsing(5e-4)` consumer
  biomass, `max(tail) > 0.55` perennial fixed point). `open_season`'s three bands pass — the
  physical-canopy band, the restated mutual-shading band, and the Greenwood crossing. The RQ
  structural prediction and the Tier-1 period-1 fixed point pass. ⚠ Two acceptance-gate facts
  moved and are recorded rather than smoothed: the station's plant-side margin **loosened
  11.8868 → 12.2894** (the plant still binds, but the gap to the cabin's 16.6667 narrowed and
  this pair has crossed twice in two days), and the 50-year perennial peak-leaf attractor now
  settles at **0.5437, below the 0.55 the floor is anchored on** — that figure is a probe, not
  a manifest bound, all four frozen 15-year floors still pass, and the assertion was inverted
  with its reasoning rather than the floor moved.

- **2026-08-14 — THE WITHIN-DAY LIGHT PATH: the plant breathes (13 goldens, both manifests,
  and the native port).** `docs/plans/post-roadmap-gross-net-gas-exchange.md`. Authorized by
  the user (*"the plants MUST emit oxygen at least minute by minute and consume co2 … it is
  imperative, it is what reality is"*), and then, on being told the within-day light curve
  was separable, *"it should be possible for the light Input to vary within the day"*.

  **Why.** The charge asks for a mechanism the tree **already had**: the sealed branch of
  `MaintenanceRespiration` burns biomass, returns CO₂ to the shared pool and consumes O₂ at
  PQ = 1, cited and conservation-balanced since Phase 2 — and it had **never executed**,
  because a day-averaged PAR makes `GASS > MRES` at every step of every scenario. The defect
  was a forcing that made an existing mechanism unreachable, so the fix is a forcing, not a
  flow. ⚠ Third instance of that shape (`canopy-regulator-diagnosed`,
  `stem-reserve-form-is-on-the-shelf`): **check whether the mechanism is already in the tree
  and merely unreachable before designing a replacement.**

  **What changed.** A new `domains/biosphere/light_path.py` (the two window means, sinusoid
  and lamp top-hat); `photosynthesis.daily_canopy_assimilation` → `canopy_assimilation`, its
  `daylength_s` argument becoming a `window_s` the daily budget passes one day of seconds to;
  `CarbonContext` loses its `daylength_var`; `season.weather_resolver` wires the sinusoidal
  path and the two lamp seams (`station/lighting.py`, `station/sealed.py`) the top-hat. **No
  flow, stock or parameter was added or removed** — `flow_set`, `aux_set`, `param_files` and
  `dt_days` are all unmoved in the manifest diff. `git diff src/simcore/` stayed empty.

  **What moved.** All 7 biosphere goldens and the 4 plant-bearing station goldens (the
  plant-free ones — cabin, crew, demo, ECLSS, power, thermal, station, water recovery — are
  byte-identical, which is the structural check that the change did not leak). The manifest
  gains `forcing.light_path`, the first **compared** hash in a `forcing` block that was
  otherwise provenance-only.

  **The gate report — every band AND every liveness floor, as the ceremony requires.**
  All five CO₂ compensation-point bands clear (sealed 71.28 ppm, perennial 70.49, consumer
  73.81, against 61.07) though every margin tightens ~4–7 %; the Greenwood peak-W crossing
  gets *safer* (2.20 % → 4.75 %). ⚠ **All five liveness floors pass and two of them
  narrowed**: perennial's converged peak-leaf `max(tail) > 0.55` reads 0.603679 (+9.8 %,
  was +11.3 %) and its annual-min chamber CO₂ 0.071036 (+42.1 %, was +51.0 %), while the
  two peak-leaf non-collapsing floors and the consumer's year-end carbon sit at +1048 %,
  +1195 % and +5343 % — nowhere near. `rationed == 0` and no extinction events on every
  scenario at every step measured. **One deviation is recorded rather than fixed**: the
  peak-LAI band passes at the shipped step (5.3806) and its converged value (4.7132) is below
  the floor — see the known-deviation section above, with the control that attributes the
  loss to the FvCB light response's concavity rather than to the new night respiration.

  ⚠ **Two prose claims were measured false while doing this, both of them ungated:**
  `lighting.py`'s *"the only runtime consumer of `daylength_s` is photosynthesis"* (there
  were three readers; the photoperiod-sensitive phenology path was added after that sentence
  was written) and `MaintenanceRespiration`'s *"at the PP fill `f_O2` ≈ 1"* (measured 0.854
  — and 0.847 on the committed tree, so the dip is the provisioned O₂ fill's, not the light
  path's). Both corrected in place. A claim about a branch that never runs is never wrong
  where anyone can see it.

- **2026-08-14 — THE INTEGRATION STEP: `dt = 1 day` → `dt = ¼ day` (13 goldens, both the
  biosphere and station manifests, and the native port).**
  `docs/plans/post-roadmap-step-unfreeze.md`. Authorized by the user (*"quarter the step"*).
  This is the first unfreeze of a **numerics** item rather than a science item, and the
  first to move the station contract as well.

  **Why.** The `dt = 1` reference drew the **perennial** chamber's CO₂ down to **56.03 ppm**
  and kept fixing carbon there, below the `61.07 ppm` compensation point its own FvCB kinetics
  make a hard shutoff — a truncation error, not biology. At `¼` its season-low **was
  75.48 ppm on the day of this ceremony, and reads 70.25 ppm on today's tree** (the light
  path and the layered canopy both moved it; the band section above tabulates all five and
  says why the prose lagged). See the resolved-deviation section above for both magnitudes
  and the caveats.
  ⚠ This entry read *"the sealed chamber ... 57.9 ppm ... 76.82 ppm"* until **2026-08-14**;
  corrected there — the sealed chamber never crossed in its own configuration, and the pair of
  numbers came from two different runs.

  **What changed.** `BIO_DT` / `STEPS_PER_DAY` in `src/domains/biosphere/step.py`, which is
  now the single place the step lives; the station's three scenarios bind `bio_dt` /
  `bio_steps_per_day` to those constants instead of declaring their own `1.0`;
  `run_master_day` takes `slow_steps_per_day` slow sub-steps per master day (with a new
  `slow_dt · slow_steps_per_day == 1 day` guard mirroring the fast one). `git diff
  src/simcore/` stayed empty throughout.

  ⚠ **The step change was the small part; the UNIT change was the work.** Every run length,
  reset period and perturbation window had to move from days to steps, and *no test at
  `dt = 1` can tell a correct conversion from a wrong one* — the two are the same integer.
  So it landed in two commits: the routing first, proved **byte-identical** on the full
  suite with no golden regenerated, then the step, so every byte of the golden diff is
  attributable to the step alone. Correctness of the routing was established by a separate
  discriminating probe at `¼`, not by the suite.

  **Verification, predicted before regenerating** (plan §4/§4b, scored in §4c): all 12
  goldens' step counter `n` hit its predicted value exactly; the 12 goldens with no
  biosphere in them stayed byte-identical; `git diff --stat` came back 279 insertions and
  279 deletions, so no array anywhere changed length; and in the lighting seam every
  biosphere stock moved while `power.battery`, `boundary.waste_heat` and
  `boundary.light_used` came back **bit-identical** — testing, for the first time, the
  docstring claim that that seam is coupled by a forcing schedule only. `rationed == 0` and
  `events == ()` throughout.

  **A SECOND MANIFEST EDIT, 2026-08-14, during the re-pinning that followed.** The
  `liveness_floors` justification for `perennial_long_horizon` cites its probe **by value**,
  and both values moved: the anchor `0.0736681` (1.47× the floor) → **`0.0758448`** (1.52×),
  and the witness *"the jar shrunk `0.8x` at fixed composition trips it at `0.0481100`"* →
  **`0.65x` … `0.0492366`**. ⚠ **The `0.05` floor itself did NOT move, deliberately.** The
  chamber's whole per-year CO₂ trough series rose ~35 % at the finer step, so the floor is a
  third further away than it was and the old `0.8x`/`0.7x` jars no longer reach it; the guard
  was **re-run rather than re-tuned**, sweeping the shrink factor to find where the crossing
  now is (between `0.68x` at 0.0514049, passing, and `0.65x` at 0.0492366, tripping) and
  leaving the bound alone. Re-anchoring a floor upward every time the reference rises is how a
  floor becomes a restatement of the current run. The guard keeps its teeth and they are
  blunter: the jar must now be shrunk by about a third where a fifth used to do.

  ⚠ **What this unfreeze did to the ACCEPTANCE GATE, recorded because it weakens it.** The
  census in `tests/test_acceptance_gate.py` measures margins in **steps** (`stock / demand per
  call`), so quartering the step multiplied every biosphere margin by ~4 while every row driven
  by an untouched registry — power, thermal, ECLSS, crew, nine of nineteen — stayed
  bit-identical. On `sealed_station` that crossed a boundary: its binding call was the plant's
  draw on the shared CO₂ pool (5.0232), and at `¼` that rose to **19.0209**, above the ECLSS
  scrubber's unchanged `1/(k·dt) = 16.667`. **Same stock, different registry — the station's
  `rationed == 0` no longer answers a question about the plant**, and the census's "the six
  smallest live margins are `carbon_pool` in the six sealed scenarios" is now five. Not
  repaired: choosing a step to keep a gate would be backwards. The natural successor is still
  the `science_band` named below — and it must be written for the **perennial and consumer**
  chambers, not the sealed one.

- **2026-08-12 — stem-reserve remobilization, and its CESSATION at maturity (+1 flow,
  +1 stock, +1 param file, 13 goldens, the biosphere manifest).**
  `docs/plans/post-roadmap-stem-reserves.md`. The mechanism was **diagnosed and refused on
  2026-08-10** and is built here on the user's explicit call — a provenance judgement that
  was theirs to make, the same shape as the root-coupling build.

  **What changed.** A new POOL stock `stem_reserve_c` holds the stem's shielded starch.
  `Allocation` **splits its own stem leg** — `fstr` of it is deposited as starch instead of
  structural stem ([E] §3.2.4 p. 93, Listing 3 Line 17), which is a re-routing of one
  deposit rather than a fifth sink, so the CO₂ and O₂ legs are untouched and the flow still
  balances by construction. A new flow `StemRemobilization` draws the reserve into the
  grain at `0.1 d⁻¹` (Listing 3 Line 35). `annual_reset` dumps whatever is left to litter
  with the rest of the dead plant. `params/stem_reserves.yaml` carries the numbers;
  `SeasonScenario.stem_reserves` switches the mechanism per crop and **potato turns it
  off**, because [E] Table 7 gives potato a *range* ("0.2-0.4") where it gives wheat a
  single 0.4 — picking inside someone else's range is our number wearing their name.

  ⚠ **`SEALED_CHAMBER_SCENARIO.litter_carbon0` was re-sized 3.0 → 3.5**, because the extra
  carbon fixed released enough O₂ to bottom the pool at **5.08 %** of its fill against a
  scenario whose whole purpose is a ≥ 95 % depletion — the diagnostic it exists to show was
  abolished. The sweep behind the new value is recorded in `scenario.py` beside it.

  **The cessation (the second half, and the user's second question — "the stem should stop
  feeding the seed at some point").** As first built, neither half of the mechanism ever
  stopped: the drain fired at every step from anthesis onward and the fill kept diverting
  starch for as long as the partition table fed the stem. Measured, the consequence was
  smaller than it sounds — the reserve peaks at anthesis and **drains 91 %** by season's
  end — but **3–7 % of the transfer happened after the crop was physiologically dead**, and
  `sealed_chamber` (which never re-sows) ran **two whole years** past maturity doing it.
  Both halves now stop at `cessation_dvs = 2.0`.

  ⚠ **That bound is [E]'s own, and it must be read at its exact strength.** Listing 3 —
  the module whose Lines 17/35 *are* this mechanism — ends at **Line 114** with
  `FINISH DS = 2., CELVN = 3.`, and the prose says it twice (§3.1.4 p. 81, §3.4.2 p. 105).
  But `FINISH` is a **run-control** statement: [E] does not say remobilization ceases at
  maturity, it says its program **has no state there**. So this is the source's *domain
  boundary*, and using it is a decision **not to extrapolate a form past the program that
  defines it** — never a cited cessation rule. Our tree has no `FINISH`; its DVS merely
  caps at 2.0 and the season keeps stepping, which is why the question arises here and not
  in [E].

  **Science gates, reported.** `open_season`'s two outside-sourced bands **hold, with
  shrinking margins**: peak LAI **5.4624** (inside 5–8, and 91.0 % of the Van Keulen &
  Seligman 6.0 threshold, up from 86.5 %), Greenwood's `W` **14.1457 t/ha** (98.1 % of the
  14.4248 crossing, up from 87.6 %). ⚠ At Table 7's **top** row (sugar-cane, 0.50) `W`
  crosses — pinned, so the clearance is a measured fact rather than an impression. All four
  `perennial_long_horizon` liveness gates pass: CO₂ trough **0.056030** (above both the
  0.05 floor and the frozen 0.055175), stationarity, the leaf floor, and the fixed point
  **0.637384** > 0.55. `rationed == 0` and no extinction events on every chamber, at 5 and
  15 years, and under RK4.

  **The golden diff was PREDICTED BEFORE REGENERATION, twice, and both predictions held.**
  For the cessation the prediction was: the 13 goldens the build already moved, **no
  fourteenth**, and nothing non-wheat. Measured: **10** moved and 3 did not — and the three
  that held (`greenhouse`, `lighting`, `harvest`) are the 7-day station runs, which end 287
  days short of maturity, so the window provably cannot reach them. Potato is bit-identical
  because its reserve is off.

  ⚠ **Two counts in this document were stale BEFORE this work and are corrected here**:
  the flow set read "21" while the manifest held **22** (`Drainage` was added by the
  soil-water re-basing and the prose was not updated), and the param count is now **15**.
  The manifest gate compares the manifest against the tree; **this prose is not a side of
  that comparison**, which is exactly the gap already recorded under "the freeze's prose
  half is ungated".

- **2026-08-12 — `WSFD`, drought-accelerated phenology ([F] Eqn 15.8; NO manifest movement,
  NO frozen golden movement, one non-frozen golden).**
  `docs/plans/post-roadmap-water-stress-curves.md`. The second of the two successors the
  soil-water re-basing named. **The first unfreeze in this series that NOTHING AUTOMATIC
  CATCHES** — see the warning above.

  **What changed.** `ThermalTimeAccumulation` gained a third optional rate multiplier,
  `WSFD = (1 − WSFG)·WSSD + 1`, applied to the daily temperature unit. Drought hastens
  development, so unlike its two neighbours (`verfun`, `ppfun`) it is **not** a `[0, 1]`
  limitation factor, and unlike them it is **not phase-gated** — [F] Box 16.2 gates it on
  `CTU > tuEMR` only, and our accumulator starts at emergence, so it runs through grain
  filling. `SeasonScenario` gained `wssd: float | None = 0.40` ([F] Table 15.1, wheat, off
  the same page render `wssg` came from); `POTATO_SCENARIO` sets it `None` because
  **Table 15.1 has no potato row** and populates the coefficient for only two of its ten
  crops — an absence in the source, not a modelling preference.

  ⚠ **The record that named this successor mis-filed it as a threshold** ("`WSSD`
  (phenology, 0.40)", under a heading about "different **thresholds**"). Table 15.1's caption
  calls it "a **coefficient** of phenological development response to drought", and Eqn 15.8
  is driven by `WSFG` — which the tree already computes. So the build needed no new `FTSW`
  call site and no second threshold, and was materially smaller than its recorded price.

  **Why the frozen roster does not move, measured rather than argued.** `WSFD(1) = 1`
  exactly, and every one of the 7 frozen scenarios holds `WSFG ≡ 1` — the driest,
  `drought`, bottoms at `FTSW = 0.7039` against `wssg = 0.30`. Instrumenting
  `water_stress_factor` across every golden-bearing run on the frozen tree found exactly
  **two** runs where water limits at all: `water_biting` (min `WSFG` 0.1667) and
  `deep_water` (0.2677). Re-run with the coefficient live, every frozen scenario is
  **bit-identical in every stock and every aux**.

  **Science gates (step 5).** Unchanged by construction: no frozen scenario's trajectory
  moved a bit, so every `science_bands` reading and every `liveness_floors` reading is
  exactly what it was, including `open_season`'s tight 3.8 % / 12 % margins.

  **The one golden that moved** is `water_biting_state.json`, which is **not** in this
  manifest. 14 stocks moved, 6 are bit-identical, and the split is the finding: **every
  WATER stock and `rooted_depth` came through untouched**, because potential transpiration
  is a Penman–Monteith function of weather (not of leaf area) and the roots had already
  stopped on the dry-subsoil gate by day 12. Carbon: leaf −13.6 %, root −7.9 %, stem
  +4.2 %, **grain +33.2 %** — faster development means anthesis arrives sooner, so less of
  a water-limited season goes into canopy and more into filling grain. Drought escape is
  what `WSSD > 0` encodes. `rationed == 0`, `events == ()`, loss-sink empty.

  **The feedback loop was measured, not reasoned away.** `WSFD` speeds DVS → root extension
  hits its `DVS ≥ 1` stop earlier → shallower zone → lower `FTSW` → larger `WSFD`. Bounded
  by `1 + WSSD`, and measured inert on both live runs even at an absurd `WSSD = 1.50`,
  because root growth has already stopped for a *different* cited reason long before
  anthesis (day 12 vs 251 on `water_biting`; day 107 vs 251 on `deep_water`). That is a
  fact about these two scenarios, not a general safety property.

  **`WSSL` was REFUSED, and the refusal got stronger.** See the plan doc: [F] Box 16.2
  applies `WSFL` to its node-driven leaf-area branch and deliberately **not** to the
  carbon-driven `GLAI = GLF·SLA`, whose dry matter already carries `WSFG`. Our canopy is
  only ever that second branch, so the factor would double-count. The successor is a
  sink-limited leaf-expansion phase (which would require LAI as a state variable, reversing
  the "LAI is derived, not stored" lock), not a missing multiply.

- **2026-08-12 — the soil water regime re-based on geometry (+1 flow, `flow_set` 21 → 22,
  12 goldens, both manifests, both ports).**
  `docs/plans/post-roadmap-soil-water-rebasing.md`. The successor the soil-layers build
  named as "the real finding", taken on the user's call. **The largest unfreeze in this
  series, and the only one so far that changes what a frozen number MEANS rather than what
  it is.**

  **The defect.** `soil_water0 = 1000` kg over 1 m² is 1000 mm of extractable water, which
  at `EXTR = 0.13` requires a **7.7 m soil column**. The root-zone bucket was not
  dimensionally a soil profile, and the stress thresholds beside it (`sw_wilting = 20`,
  `sw_critical = 60` **kg**) were absolute masses calibrated against that impossible
  bucket.

  **What replaced it, all from [F] Soltani & Sinclair Ch. 13–15:**

  | piece | equation |
  |---|---|
  | `soil_water0` 1000 → **19.5** kg | `ATSW = DEPORT · EXTR · ρ · A · MAI` (14.26) |
  | `subsoil_water0` 195 → **175.5** kg | `WSTORG = IPATSW − ATSW` (14.27/14.28) |
  | `sw_wilting`/`sw_critical` → **`wssg = 0.30`** | `FTSW = ATSW/TTSW` (14.6/14.7), `WSFG = min(1, FTSW/WSSG)` (15.3), Table 15.1 wheat |
  | new `Drainage` flow | `DRAIN = (ATSW − TTSW)·DRAINF` (14.11) into `subsoil_water` (14.12) |
  | `Irrigation` demand-driven | `IRGW = min(capacity, TTSW − ATSW)` (14.8) |
  | `soil_moisture_index` (MAI), `drainage_factor` (DRAINF) | new scenario fields |

  ⚠ **`subsoil_water0 = 195` was WRONG, and its pin held the wrong identity.** 195 kg is
  [F]'s `IPATSW` — the *whole* profile — where 14.28 makes `WSTORG` the profile **minus**
  the root zone. The shipped default double-counted the root zone's 19.5 kg, and
  `tests/test_soil_layers.py` pinned that formula. Defensible only while `soil_water0` was
  not geometric; the re-basing removed the excuse, so value and pin moved together.

  **The measured result: only water moved.** Predicted before regeneration and confirmed
  over **all 25 goldens on disk** — `season_euler`, `sealed_chamber`, `n_limited`,
  `drought`, `perennial_chamber`, `consumer_chamber`, both long horizons, `greenhouse`,
  `lighting`, `harvest` and `sealed_station` move `soil_water` / `subsoil_water` /
  `water_source` and **nothing else**. Not one carbon, nitrogen or oxygen amount, at any
  horizon. The reason is structural: every frozen scenario is a *potential production*
  scenario, so `FTSW` never falls below 0.79 and `WSFG ≡ 1` exactly as `f_water ≡ 1` did.
  The two forms agree wherever water does not limit, which is everywhere the freeze looks.

  ⚠ **`water_biting` is the exception, and it was re-declared rather than preserved.** Its
  bite was `soil_water0 = 50` kg "inside the (20, 60) band" — a band that no longer exists.
  It now declares `soil_moisture_index = 0.05`, chosen against its own written contract (a
  sustained bite, never fully wilted, crop alive, loop conserved) and swept, not fitted to
  a golden. Leaf C peak 0.8299 → 0.7621, storage C 0.2610 → 0.2452. **Its dry-subsoil
  override is retired** — under geometry the subsoil scales with the same MAI, so it no
  longer abolishes the stress it was protecting, and keeping it would kill the crop.

  ⚠ **The geometry re-basing and the `FTSW` conversion were filed as two successors and
  are ONE mechanism.** Geometry alone kills every sealed chamber outright: a sealed chamber
  has no water inlet, so `soil_water ≤ sw_wilting` is an **absorbing state** (no
  transpiration → no vapour → no condensate → no recycling → no change), and the geometric
  19.5 kg misses the 20 kg escape by 0.5 kg. Measured, not reasoned.

  ⚠ **Three defects surfaced that the change did not cause, only exposed:**
  (1) all three station builders (`greenhouse`/`lighting`/`sealed.py`) seeded their aux
  **without** `rooted_depth`, silently starting the crop at depth 0 — invisible while the
  depth gate was inert, fatal once stress divides by `TTSW = depth · EXTR · ρ · A`;
  (2) `harvest` injects a 1.3 m root system but inherited the 0.15 m zone's water, i.e.
  `FTSW = 0.115` on day 0 for a grain-filling crop — the depth and the water are two halves
  of one declaration and are now derived together;
  (3) `test_soil_fractionation.reset_variant` hand-copied the re-sow rule under a comment
  claiming it mirrored `season.annual_reset`, so when the rule changed the copy did not —
  the durable fix is `season.resow_water_return`, one function with two callers.

  ⚠ **The re-sow return was re-derived, not re-tuned.** It returned the abandoned column
  *at the drained upper limit* (149.58 kg) clamped to what was held — fine against a 1150 kg
  store, more than the entire store once the store is 19.5–169 kg, at which point its clamp
  fired every re-sow and handed the whole root zone to the subsoil. It is now the abandoned
  **fraction** of the water, which preserves `FTSW` exactly across a re-sow, needs no clamp,
  and equals the old form at the drained upper limit. Measured: one transient cycle, then a
  fixed point held to round-off over eight years.

  ⚠ **`Drainage` is bit-identically inert on the entire frozen roster** — with irrigation
  demand-driven the zone is never over-filled, so `DRAINF` 0.3 and 0.0 give identical
  states. That is physically correct (you cannot drain what was never over-applied) and it
  means **no golden can catch this flow's removal**; its pins are unit-level and
  mutation-verified, like `root_depth`'s.

  **What was deliberately NOT built, each a named successor:** `WSSL` (leaf-area expansion,
  0.40) and `WSSD` (phenology, 0.40) — we have no water-gated leaf-expansion or
  drought-accelerated development term for them to attach to; runoff and soil evaporation;
  and making `DROUGHT` actually bite (now a one-field change, but it would move a golden's
  science for a reason outside this charge).

  **Cascade:** biosphere manifest (`flow_set` 21 → 22, golden hashes); station manifest
  (golden hashes); 12 goldens; `src/station/{greenhouse,lighting,sealed,harvest}.py`; the
  Rust mirror; the acceptance-gate census (three rows change tightest stock to
  `soil_water`, and two threshold-shaped claims were **dropped rather than re-tuned** —
  see below).

  ⚠ **Two acceptance-gate claims died and were replaced by rank + exact values, not by
  looser thresholds.** `water`'s slack in `open_season` fell 189.24 → **9.31** (a margin of
  189× was never a fact about safety, it was a fact about a bucket that could not exist);
  and `carbon_pool > 4 × runner-up` became 3.98× on two chambers. Loosening either bound
  after seeing the measurement is the fitted comparison `test_acceptance_gate.py` refuses
  in its own words, so both were replaced by the rank plus an exactly-pinned runner-up —
  strictly stronger, since a threshold only catches changes bigger than its slack.

- **2026-08-11 — soil layers: the below-root store (+1 flow, +1 stock, 12 goldens, both
  manifests).** `docs/plans/post-roadmap-soil-layers.md`. The successor the root-depth
  build named, and the first unfreeze in this series that **moves a value**.

  `subsoil_water` (`WSTORG`) holds extractable water that is physically present below the
  rooted depth and currently unreachable; `RootZoneCapture` (`EWAT`, [F] Eqn 14.10) moves
  it into `soil_water` as the roots arrive. Three cited additions came with it: the soil's
  own rooting cap (`SOLDEP` — **discharging a ceiling `root_depth.yaml` had recorded as
  deferred**), the extractable-water fraction `EXTR` = 0.13 ([F] Ch. 13), and a **cited
  sowing rooting depth** replacing an uncited `0.0` ([F] Ch. 14: *"normally between 150 to
  400 mm"*).

  **The resolution was a citation, not a compromise.** This work had been priced as "the
  largest single piece the post-roadmap record has considered" on the assumption that
  layers meant an N-layer discretization. [F] opens its soil-water chapter by settling
  that: *"a two-layered soil or even a one-layer soil seems satisfactory (Robertson and
  Fukai, 1994)"*, and specifies the two stores as the root zone and the water below it.

  **The diff was PREDICTED BEFORE REGENERATION and the prediction held.** Written down:
  `soil_water` up by the season's capture, `subsoil_water` down by the same, `f_water`
  exactly `1.0` throughout, therefore **every carbon / nitrogen / oxygen stock
  bit-identical at every horizon**. Measured after: 12 goldens changed and the only things
  that moved anywhere were `soil_water`, the new `subsoil_water`, and `rooted_depth`. Both
  drift-stability summaries are **byte-identical**.

  ⚠ **Two scenarios declare a DRY subsoil, deliberately.** (⚠ **One, since 2026-08-12** —
  `water_biting`'s override was retired by the re-basing; see the entry at the top.)
  `water_biting` and `drought` are *defined* as water-lean, so a hidden reservoir would contradict their construction —
  and the measurement is the reason it matters: with the default profile the drought
  perturbation's cascade is not weakened but **abolished** (`f_water` never leaves 1.0;
  end vegetative carbon 33.61 → 33.28 instead of 33.61 → 12.68). That is the mechanism
  working, not an artefact: a crop that can root into wet subsoil is drought-defended.

  ⚠ **The re-sow water return is OURS**, not [F]'s (it is single-season and silent).
  Without it, one-way capture would ratchet the profile into the root zone over a 15-year
  chamber run. Pinned as a five-cycle identity, not as a single number.

  **What was deliberately NOT built, each a named successor:** the `FTSW = ATSW/TTSW`
  stress conversion, drainage/runoff/soil evaporation, and — the real finding —
  **re-deriving `soil_water0` from geometry.** Our root-zone bucket is not dimensionally a
  soil profile: 1000 kg over 1 m² is 1000 mm of extractable water, which at `EXTR = 0.13`
  needs a 7.7 m column. Deriving it would collapse the store to ~19.5 kg at sowing, below
  `sw_critical`, and make **every frozen scenario water-stressed**. That is a re-basing of
  the whole water regime and the verdict is the user's.

  ⚠ **ALL THREE WERE TAKEN ON 2026-08-12 — see the entry at the top of this log.** Two of
  the three turned out to be **one mechanism**, and the price recorded here was wrong in
  both directions: the open field costs ~2 % of yield, and every *sealed* chamber dies
  outright. Also corrected there: the `subsoil_water0` default above is [F]'s `IPATSW`,
  not `WSTORG`, so it double-counted the root zone.

  **Cascade:** biosphere manifest (`flow_set` 20 → 21, 7 golden hashes); station manifest
  (golden hashes); 12 goldens; the Rust mirror; `tests/test_soil_layers.py` (14 pins, all
  mutation-verified against 7 deliberately broken variants).

- **2026-08-11 — rooted depth: a third aux accumulator (+1 param file, `aux_set` 2 → 3, 12
  goldens).** `docs/plans/post-roadmap-root-functional-coupling.md`. ⚠ **This entry was
  written retroactively on 2026-08-11** — the build shipped without one, which is the
  `freeze-prose-half-is-ungated` failure mode showing up in the log itself: the manifest
  gate compares manifest against tree and has no opinion about this file.

  `RootDepthExtension` advances rooted depth by [E]'s law and gates `NitrogenUptake`'s
  supply by the fraction of the reference soil layer the roots have reached. It changed
  **no value** (12 goldens differ by exactly one added `aux` key), it is bit-identically
  inert on the entire frozen roster, and **that was measured before a line was written**:
  the work was recorded as REFUSED on the canopy-regulator precedent and the user
  overruled the refusal. Nobody should later find the inertness and conclude it was
  missed. Its pins are unit-level and mutation-verified because no golden can catch its
  removal.

- **2026-08-10 — the decade CO₂ guard re-anchored: one `liveness_floors` entry's `bound` and
  `source`, and nothing else.** `docs/plans/post-roadmap-co2-guard-reanchor.md`. No value, no
  golden, no param hash, no `flow_set`/`param_files` change — the smallest unfreeze this
  contract has taken, and it is a **tightening**.

  The floor on `perennial_long_horizon`'s annual minimum CO₂ pool skipped the first
  `_TRANSIENT = 2` years. Measured on the current tree, that window is **inert**: the
  whole-run minimum is 0.055175 (year 1) = **1.103×** the 0.05 floor and no year dips below
  it, so the slice constrained nothing on the reference and constrained only candidate
  changes. It is removed; `non_collapsing(whole)` implies `non_collapsing(sliced)`, so the
  teeth cannot decrease.

  The bound's justification moved from a 15-year reading to the trough's **measured
  attractor** (0.0732912, 1.47× the floor, converged; the CUE build's own idiom for the
  `> 0.55` floor), with the deepest year of a 50-year run pinned as lying **inside** the
  frozen horizon. The comment the window carried — *"dips to ~0.039 … before settling to
  ~0.055"* — was pre-split prose the CUE build's four-guard restatement missed; it and two
  further sites that quoted pre-split CO₂ numbers are corrected.

  ⚠ The guard was also measured **not** to detect what it claimed: slowing the recycling loop
  — the drain mechanism itself — moves the trough the *wrong way* (0.055175 → 0.057797 at
  half the microbial rate), as does starting the chamber CO₂-poor. It is a **buffer**-vs-peak-
  demand guard, and the lever that trips it is jar size at fixed composition. That negative
  result is committed as a test, and it is what gives the "the level check catches what
  `is_stationary` is blind to" claim a witness that is not a candidate science change.

- **2026-08-10 — the humification split (a CUE): +1 param file, +2 flows, +2 stocks, 6
  biosphere goldens, and 4 restated guards.** `docs/plans/post-roadmap-cue-humification.md`.
  The seam the soil-fractionation diagnosis named as its own replacement. Every decomposer
  carbon flux is now partitioned between CO₂ and the pool the remainder stabilises into,
  at CENTURY's own constants (Parton et al. 1987, first-hand): litter → CO₂ 0.45 + active
  SOM; active SOM → CO₂ 0.85 (`Es` at `T = 0`) + slow SOM; slow SOM → CO₂ 0.55 + active
  SOM, with `K6 = 0.0038/week`. Nitrogen follows the same partition, so no N rate enters
  anywhere.

  **Why it was an unfreeze rather than a refinement:** the frozen form was asserting values
  **off the end of the source's own functions** — a litter CO₂ fraction of 0.0 against a
  measured 0.45–0.55, and `Es = 1.0` where eq. [6] cannot exceed 0.85 at any texture. The
  citation covered the decomposer **rates** and never covered **where the decayed carbon
  goes**, because the partition was not a parameter.

  **Science gates, as step 5 requires.** `open_season`'s three `science_bands` are
  **untouched** — an open-field build carries no litter, microbial or humus stock, and its
  golden hash did not move. Of the `liveness_floors`: the two `non_collapsing` floors and
  the consumer floor pass; the **converged peak-leaf floor FAILED** at 0.634 against
  `> 0.9`, and this is the blocking finding, argued rather than re-tuned. The split does
  not destabilise the chamber — it lengthens the settling transient from ~3 years to ~35,
  past the frozen 15-year horizon. The attractor is real and was **measured** (0.594984 at
  ~year 45, now its own test). The floor is therefore re-anchored on that equilibrium
  rather than on the horizon's reading, at **0.55** — 2.2× the recorded 0.253 dead
  baseline. ⚠ This is the **second** time that floor has moved for a smaller plant
  (1.0 → 0.9 at the decomposer calibration); its manifest `source` records the whole chain
  so the pattern is visible rather than buried.

  Three structural pins were restated for the same reason and **none was re-tuned to a
  looser amplitude**: each was replaced by the claim still true at the frozen horizon
  (monotone + decelerating), which still fails on the failure mode the original guarded.

  **Cascade:** biosphere manifest (`flow_set` 18 → 20, `param_files` 12 → 13, 6 golden
  hashes, 1 liveness bound); station manifest (4 golden hashes); 10 goldens; the Rust
  mirror and the crossport tier.

- **2026-08-09 — the science assertions get contract standing (a SCHEMA unfreeze; two new
  manifest fields, NO value, golden, param or `src/` change).**
  `docs/plans/post-roadmap-acceptance-gate-standing.md`. The adjudication of the acceptance-gate
  diagnosis's finding 6, which that work deliberately left to the user. Added `science_bands` +
  `liveness_floors`, derived from `science_gate` markers via `ast`.

  **The reframe that unblocked it:** finding 6 read as "the two gates disagree". Read off the
  *pins*, they never overlap — closure binds on the chambers and is **structurally empty** for
  carbon on `open_season`; the bands exist on `open_season` and cannot exist on a 52 g DM/m²
  carbon-limited rig. Every "disagreement" was two verdicts on **different scenarios**,
  aggregated by the reader. So promoting a band on `open_season` cannot reverse a measured
  closure refusal — the co-adaptation objection needs a verdict to overrule, and the cell is
  empty.

  **Three exclusions, each measured rather than argued.** (1) *Margin-ratio and doc-staleness
  pins* — `peak_w / 14.4248 > 0.85` fails when prose drifts, and `0.80 < peak/6.0 < 0.92` fails
  when the margin **improves**; two committed tests were **split** so a marked test carries only
  its gate. (2) *Diagnosis pins about refused forms* (`peak > 15.0` for (C)) — not the tree as
  frozen. (3) *Calibration identities* — most of `test_bvad_validation.py` asserts quantities
  the crew params were fitted to, which its own docstring already calls true "**by
  construction**"; only the RQ structural prediction survives.

  ⚠ **A correction found while deriving this:** a draft argued the field was vacuous because the
  golden already freezes peak LAI. It does not — `season_euler_state.json` is a single
  **endpoint** (`n = 305`). The goldens constrain a trajectory only at its last step, so every
  mid-run quantity was unfrozen.

- **2026-07-27 — the nitrogen-cycle FORM change (a FORM unfreeze; one uncitable param
  RETIRED, three cited ones added; every carbon trajectory byte-identical).**
  `docs/plans/post-roadmap-nitrogen-cycle-form.md`. The named successor to the decomposer
  calibration, and the one place the biosphere was **structurally** non-physical rather than
  merely uncalibrated: **nothing tied the nitrogen leaving the plant to the carbon it was part
  of**, so litter C:N was the unconstrained ratio of two independent first-order rates —
  measured at **0.004** in-run (≈ 1 C : 246 N) against wheat straw's ~80.

  **What changed (two halves of one change — they cannot ship apart):**
  * `mineralization.NitrogenSenescence` is now **N:C-coupled**: `shed_N =
    min(plant_n/biomass_c, n_residual_per_mol_c) · shed_C`, driven by the same per-organ flux
    `allocation.Senescence` sends to litter. **`n_senescence_rate` is GONE** — a bare 1/day
    rate that five rounds of the citation scope established *no primary source publishes*
    (retrieval "exhausted, not blocked"), and the project's highest clean-room risk. It was
    discharged by **changing the form, not by finding a citation**: the coupled form's
    parameter is a tissue N concentration, and `n_residual` was *already cited* to Van Hecke
    et al. (2020) for exactly that quantity ("N left after N remobilization to the grain").
  * `nitrogen.NitrogenUptake` is now **demand-deficit** (`min(target·biomass − plant_n,
    capacity·availability)`), the seam the Phase-1 docstring named. The target is **Greenwood
    et al. (1990) eqn (6)**, read first-hand: `%N = 5.697·W^−0.5` for `W > 1 t/ha`, and
    **constant at 5.697 % below** — the paper's own statement, with a mechanism (exponential
    growth ⇒ constant %N) and an Ågren 1985 citation, not our interpolation. Three new cited
    params in `nitrogen.yaml`.
  * `season.annual_reset` now resets **nitrogen** too (it was carbon-only, leaving the
    seedling an N *windfall* its own docstring called "harmless only while `f_N ≡ 1`"). The
    seed keeps the parent's concentration; the remainder dies to litter as a balancing
    residual, so NITROGEN is conserved exactly.
  * `SeasonScenario.plant_n0` 0.5 → **2.43e-4 kg** (scenario data, no cited value moved). The
    old IC was **2055× the target concentration** — an artefact of the fixed-flux law, where
    nothing consumed `plant_n` against a target — and it does not self-correct downward,
    because a plant above target has zero deficit.

  **What moved, and what did not.** 10 goldens moved and **every one of them moved only in its
  NITROGEN stocks** (`plant_n`, `soil_n`, and `litter_n` where sealed); every CARBON amount is
  byte-identical, and `drift_summary.json` — a carbon-side stability signature — regenerated
  **unchanged**. `n_limited` is byte-identical for a structural reason: it is open-field, so it
  builds no N-shedding flow at all. Cascade: 6 biosphere goldens + `water_biting` + 3 station
  goldens (`greenhouse`/`harvest`/`lighting`) + `sealed_station` ⇒ **both** manifests (2 param
  hashes + 6 golden hashes here; golden hashes there). Rust mirrored by hand; `src/simcore/`
  diff empty.

  ⚠ **The margin behind "`f_N ≡ 1`" collapsed by ~2.5 orders even though the conclusion held.**
  Capacity-uptake pinned `plant_n` at `max_uptake_capacity / n_senescence_rate`, ~1000×
  critical. Demand-deficit fills to the *target*, so the plant now sits at **3.8× critical on
  the plateau and ~1.07× at `open_season`'s peak** (12.633 t/ha, where Greenwood gives 1.60 %
  against a 1.50 % critical). The curve crosses `n_critical` at **14.42 t/ha** — 88 % of the
  way — so anything that grows the open-field crop ~15 % moves a frozen golden. Pinned in
  `tests/test_nitrogen_form.py`, deliberately not left in prose. Note `nitrogen.yaml` has
  recorded this arithmetic since the citation scope's round 2 ("ours equals the curve only at
  W ≈ 14.44 t/ha") as a *delta*; the form change makes it a *mechanism*.

  ⚠ **All three reasons the decomposer calibration gave for NOT moving
  `mineralization_rate` are now moot — because the parameter no longer exists.** Option (A)
  falsified two of them (it was not "behaviorally inert" — it set the litter pool's C:N — and
  N and C were no longer "uncoupled"), leaving only **pool identity**: Stanford & Smith
  measured soil organic N₀, ours is fresh residue N. Option (B) then retired the parameter
  outright, and a parameter that does not exist cannot be mis-anchored to the wrong pool. The
  question was not *answered* — no value was chosen from a cited band — it was **dissolved**.

  **The litter pool C:N law, as it now stands.** Under the retired direct `Mineralization`,
  nitrogen left the litter pool at a free `mineralization_rate` while carbon left it at
  `decomposition_rate`, so the pool's ratio was pushed ~2.7× away from its input's and the
  quasi-steady law was `(shed C:N) × (k_min/k_decomp)` = `90 × 2.727` = 245.5. The
  microbe-mediated legs carry N on the **same** first-order carbon flux, so the pushing factor
  is exactly **1** and the pool converges on the ratio of the material fed into it:

      pool C:N  →  shed C:N  =  carbon_fraction / n_residual  =  90

  Measured, the shedding-fed chambers sit at **98.7–100.6** at peak `litter_n` and
  `sealed_chamber` **ends at 90.6** — within 0.7 % of the shed ratio. That is not merely a
  smaller error: the litter pool's C:N stopped being an accident of two unrelated rate
  constants and became a function of the *composition of the material that fell in*, both of
  whose numbers are cited.

  ⚠ **The residual above 90 is the N-FREE SEED, not a "pulsed transient"** (corrected
  2026-07-27 after an advisor catch, then measured). With both currencies draining on the
  same flux, `d(C/N)/dt = 0` — the ratio is exactly invariant between pulses, so pulsing
  *cannot* move it; that mechanism belonged to the retired differential-drain form. The
  chambers seed `litter_carbon0 = 3.0` mol C with **no `litter_n0` counterpart** (C:N = ∞),
  a seam the (A) record already named, and with the seed removed the pool C:N equals the
  shed ratio **to 1.4e-15 relative at every step** — an identity, not a band. So the
  **model's** litter pool C:N is `carbon_fraction / n_residual` exactly = 90, i.e. **1.125×**
  wheat straw rather than 1.25×, and the committed scenarios' excess is a known unphysical
  IC that decays at `decomposition_rate` (hence `sealed_chamber` at 3 yr ends at 90.6 while
  `water_biting` at 1 yr still reads 98.6). Pinned in `tests/test_nitrogen_form.py`; the
  committed-scenario bounds are labelled **scenario facts, not model facts**.

  ⚠ **Three previously-recorded claims are retired here, and none of them was WRONG** — each
  was a true measurement of a form that no longer exists, so they are resolved rather than
  corrected: the 245.5 law (its `k_min` is gone); "a shedding-fed pool runs N-poor at 0.71–0.78
  of the law" (it ran N-poor *because* N drained 2.7× faster than C); and "the end-of-run
  snapshot is inflated ~2.4× and horizon-dependent over an order of magnitude" — the inflation
  **was** the differential drain, so with equal drains there is none. The horizon-dependence
  that the earlier anti-regression pin existed to guard against is gone **at its source**,
  which is why that pin was replaced by its inverse (`end/peak` must now be ≈ 1) rather than
  merely relaxed.

  ⚠ **What this does NOT claim.** The decomposer cluster's **carbon** rates are untouched and
  still run at the fast edge of their literature ranges (`decomposition_rate` 4.0/yr, Olson's
  fastest ecosystem), and the litter pool's C:N now inherits whatever that carbon rate is.
  The honest statement is that the N cycle no longer contributes a *separate* uncited rate —
  **not** that the decomposer side is now fully cited.

  ⚠ **The second regime is untouched, and that is the point of calling it two regimes.** A
  reset-driven chamber's pool is filled by the **annual dump**, whose C:N is set by the dying
  plant rather than by any rate, so (B) barely moves it (10.9 → 10.0, 9.9 → 9.1). "Peak
  `litter_n`" still names two different events — the seasonal senescence maximum in a
  shedding-fed chamber, versus the dump one step past a year boundary in a reset-driven one.
  All of it is pinned in `tests/test_nitrogen_form.py`, each scenario driven the way its own
  golden drives it.

  ⚠ **A recorded limitation, not an oversight:** shedding at the residual concentration means a
  senescing plant **retains** most of its nitrogen while its denominator collapses, so tissue
  concentration rises without bound as biomass → 0 (measured ~110× target in the 3-year
  chamber, ~6e6× in the 5-year perennial). Harmless for carbon (`f_N` saturates at 1) and N is
  conserved exactly, but it is the **one-pool** model showing through: real remobilized N goes
  to *grain*, and there is a single whole-plant pool that cannot represent that. Related: the
  chambers seed `litter_carbon0` with **no `litter_n0` counterpart**, which inflates their pool
  C:N further — a deferred seam, now named.

- **2026-07-21 — scope (B) decomposer calibration (a VALUE unfreeze; two carbon-side
  rates moved).** The scope-C diagnosis (the decomposer cluster runs fast vs the primary
  literature) drove a calibration of the two **carbon-side** rates from above-range to
  top-of-range: `decomposition_rate` 0.02 → **0.011**/day (7.3 → 4.0/yr; Olson 1963's
  fastest ecosystem, near Zhang 2008's 293-litter max) and `microbial_respiration_rate`
  0.05 → **0.016**/day (18.25 → 5.84/yr; the CENTURY/CLM5 active-SOM range). Both land at
  the **fast edge**, forced by chamber closure: central literature values starve the
  recycled-CO₂ loop and crash annual re-sow (measured; RothC-BIO 0.66/yr is infeasible at
  any litter size — resizing overshoots into rationing). So "runs fast" is **reduced, not
  resolved**, and the residual is documented (real residue like wheat straw decays nearer
  Zhang's median; the strict RothC microbial-biomass reading is ~8.8× below ours). The
  micro re-anchoring (active-SOM over strict microbial-biomass) is deliberate and
  recorded, not a relabel to excuse the old value. `mineralization_rate` was
  **investigated and deliberately NOT moved** — its cited range is the wrong pool (soil
  N₀ vs fresh residue N), the model's N scale is non-physical, and the rate is
  behaviorally inert; the real gap is the missing immobilization **form** (a documented
  deferred seam), recorded in its `source:`.
  ⚠ **All three of those grounds are now spent, and by 2026-07-27 the parameter itself is
  gone** — option (A) falsified "inert" and "non-physical scale", and option (B) retired
  the parameter (and its file) outright, so the pool-identity objection has nothing left
  to attach to. The `source:` this sentence points at now lives in
  `docs/retired/mineralization.yaml`. Note also that the "missing immobilization form"
  named here turned out to be **unavailable**, not merely unbuilt: our `microbial_carbon`
  is a *transit* pool (CUE = 1.0), so the textbook homeostatic microbial C:N would demand
  90–152× the litter N present. What shipped is microbe-mediated N **transit**; the
  immobilization seam stays open with a *measured obstacle* rather than a deferral. **6 frozen goldens regenerated** (sealed,
  perennial, consumer, both long-horizon, drift_summary) + water_biting + 4 station
  goldens (greenhouse/harvest/lighting/sealed_station); the manifest's 3 param hashes
  (incl. mineralization's comment-only edit) + 6 golden hashes moved. **Period class held
  period-1** (Tier-0 exact — no flip); the closed-chamber plant shrinks ~19% (perennial
  fixed point 1.222 → 0.994, still robustly alive), so the `test_biosphere_stress`
  `>1.0` guard was updated to `>0.9` with the reason in place. Crossport sensitivity
  6.47e-15 ≪ band 1e-11. Advisor-reviewed before regeneration; hand-mirrored into Rust
  (`biosphere_params.txt`, two hexfloats). `git diff src/simcore/` empty. Full record +
  the resize measurement + the honest ruling-B framing:
  `docs/plans/post-roadmap-decomposer-calibration.md`.

- **2026-07-20 — scope (B) increment 1: vernalization + photoperiod.** Two clean-room
  sciences (Soltani & Sinclair 2012, Ch. 8 Eqn 8.3/8.6 and Ch. 7 Eqn 7.6) added to
  `phenology.py` as a second aux accumulator + two vegetative-phase rate multipliers.
  `phenology.yaml` grew 4→12 params (all **cited**, not `TODO(cite)`); `aux_set` grew
  `{ThermalTimeAccumulation}` → `{…, VernalizationAccumulation}`. The **CONSUMER chamber
  was enlarged 2×** (a coupled scenario-data change — the healthier plant over-drew its
  CO₂ pool; SEALED/PERENNIAL kept their sizing). **12 goldens regenerated** + the manifest;
  the perennial period class moved period-2 → period-1 (a broken-canopy artifact
  dissolved). Advisor-reviewed before regeneration; hand-mirrored into Rust (which surfaced
  a genuine cross-port reset bug). `git diff src/simcore/` empty. Full record + the four
  findings: `docs/plans/post-roadmap-oracle-match.md`. Recalibration of the `tsum` residual
  is deferred (scope-B ceremony 2), oracle-as-diagnostic-only.

- **2026-07-20 — scope (B) ceremony 2: `tsum` citation (a provenance-only unfreeze; NO
  value moved).** The recalibration premise was **falsified**: both thermal sums are
  already literature-centred (Penning de Vries et al. 1989 Tables 12 & 15, first-hand off
  the page images — `tsum_maturity = 750` is dead-centre of the winter-wheat range
  [727, 784] °C·day; `tsum_anthesis = 1100` ∈ [1026, 1333]). The oracle's implied
  TSUM2 (~1207) is a longer-grain-fill **cultivar**, not our error, and matching it would
  leave the cited range (backfitting, forbidden by ruling B) — and is calendar-impossible
  at our anthesis anyway. So the two `tsum` `source:` strings were retired from
  `TODO(cite)` to [E] Penning de Vries 1989; **`phenology.yaml`'s manifest hash moved,
  nothing else** — no golden, no `src/`, no Rust value (`biosphere_params.txt`
  byte-identical), station/authoring manifests untouched. This is the scope-(C) shape: a
  provenance edit no golden catches, so the ceremony (advisor review → regenerate the
  manifest as the git-visible record → provenance) was run deliberately. Full record +
  the double-modulation exploration + the "why not both / day-neutral habitat crop"
  resolution: `docs/plans/post-roadmap-oracle-match.md` ("Ceremony 2 — DONE").

## Phase-5 handoff

The biosphere is frozen as **THE reference**. Phase 5 builds sibling domains (power / thermal /
atmosphere-ECLSS / crew), each verified **standalone against its own references first**, then
against this frozen biosphere — never the reverse (a sibling does not get to move the
reference). The reference moves only through the unfreeze discipline above.
