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
  `params/demo.yaml`. These exercise the *engine* (RK4 vs Euler, the conservation gate) and
  are frozen separately by their own Phase-0 goldens (`demo_euler_state.json`,
  `demo_rk4_state.json`, `state_snapshot.json`). The manifest excludes `demo.yaml` explicitly.
- **No new science.** Phase 4 added no flow, no trophic level, no coupled
  (Lotka-Volterra/Holling) dynamics — those were deferred at the Phase-3 capstone. The freeze
  captures the closed biosphere **as Phase 3 left it**.
- **The two additive dormant-machinery scenarios** — `N_LIMITED_SCENARIO` (open field,
  `f_N` driven below 1 by N-dilution) and `WATER_BITING_SCENARIO` (sealed chamber, the closed
  water cycle's `f_water` driven below 1) — added *after* the freeze (the Phase-5 sequencing
  decision) to flush the never-run-hot `f_N` and sealed-`f_water` limiter integrations before
  Phase 5. They are **deliberately NON-frozen**: scenario *data* only (no new flow / aux /
  param), their own goldens (`n_limited_state.json`, `water_biting_state.json`,
  `test_{n_limited,water_biting}.py` + the two `test_regression_*` gates), and **not** in the
  manifest. Adding them left all seven frozen goldens byte-identical — that byte-identity is
  the proof the reference did not move. A future maintainer should read these as intentional
  stress scenarios, not a freeze omission.

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
`Ca = 61.07 ppm` — and the sealed chamber's season-low sat at **57.9 ppm**. It was a
**truncation error, not a threshold crossing**: the withdrawal was computed at the
start-of-step concentration and applied for a whole day, so a step starting above the shutoff
and ending below it never re-evaluated.

**Measured at the shipped `dt = ¼`: the sealed chamber's season-low is `76.82 ppm`** —
clear of the 61.07 shutoff, so the reference no longer fixes carbon below its own
compensation point. Authorized by the user (*"quarter the step"*, over `½`, which also clears
it: `¼` leaves 4.8× headroom to the arbitration bound where `½` leaves 2.1×, so the next
mechanism added probably does not force a second ceremony). Ceremony:
[`plans/post-roadmap-step-unfreeze.md`](plans/post-roadmap-step-unfreeze.md).

**Both figures the old note insisted be quoted together, restated:** the tail statistic that
was **24 % low** (57.9 against a converged ~76) is the one this change fixes; the **headline
outputs that moved ~3 %** moved as predicted (sealed peak leaf carbon 0.9215 → 0.8923, −3.2 %).
Neither alone was an honest summary of the defect, and neither alone is an honest summary of
the repair.

⚠ **Two things this does NOT claim.** (a) `Γ*` remains a `TODO(cite)` entry in
`photosynthesis.yaml`, so the *crossing* was robust but the *number* 61.07 is still
provisional — clearing a provisional threshold is a weaker result than clearing a cited one.
(b) The convergence sequence and RK4 limit quoted in the 2026-08-13 sweep (`57.9 → 75.1 →
75.8 → 76.0` against a limit of `76.29`) are **stale**: the measured 76.82 sits *above* that
limit, which a finite-step Euler run should not, so the sweep's limit was measured on a tree
that has since gained mechanisms (stem reserves, soil layers, root coupling). Re-measuring it
is a refinement study and is **not** part of this ceremony — do not re-quote those numbers as
current.

The historical record of the deviation while it stood — how it was found, the enrichment
sweep, and the headroom measurement — is
[`log/co2-enrichment-margin.md`](log/co2-enrichment-margin.md) and
[`log/allocation-headroom.md`](log/allocation-headroom.md). ⚠ Read those as **dated**: they
describe the `dt = 1` tree and their convergence numbers are superseded per (b) above.

⚠ **The gap that let this ship for a year, worth more than the fix.** It was never a guard
failure: arbitration rationed zero times, every golden gate was green, and every
`science_band` passed. **A `science_bands` entry of the right shape — *"the sealed chamber's
season-low CO₂ stays above the compensation point"* — would have caught it on day one.** The
reason it did not exist is that the bands were written to describe what the model *does*, not
to cross-check the model against its own kinetics. That band is now writable (it passes at
76.82 ppm against 61.07) and adding it is the natural successor to this ceremony; it is
deliberately **not** bundled here, because a band written in the same change that makes it
pass is a restatement of the run, not a contract on it.

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
  manifest, so none could fail an unfreeze ceremony. Both fields are **derived** from
  `science_gate` markers in the test tree (`tests/science_gates.py`, statically via `ast`), so
  they carry the same completeness teeth as the flow set.

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
3. **Regenerate the affected goldens**, each via its own explicit `__main__` action
   (`tests/test_regression_*.py`, `tests/test_regression_long_horizon.py`), and **review the
   byte diff** — a change there means the trajectory moved, which is the point.
4. **Regenerate the manifest** (`uv run python tests/test_freeze_manifest.py`) and review its
   diff — the changed hashes / flow set / param set are the git-visible record of exactly what
   was unfrozen.
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

- **2026-08-14 — THE INTEGRATION STEP: `dt = 1 day` → `dt = ¼ day` (13 goldens, both the
  biosphere and station manifests, and the native port).**
  `docs/plans/post-roadmap-step-unfreeze.md`. Authorized by the user (*"quarter the step"*).
  This is the first unfreeze of a **numerics** item rather than a science item, and the
  first to move the station contract as well.

  **Why.** The `dt = 1` reference drew the sealed chamber's CO₂ down to **57.9 ppm** and kept
  fixing carbon there, below the `61.07 ppm` compensation point its own FvCB kinetics make a
  hard shutoff — a truncation error, not biology. At `¼` the season-low is **76.82 ppm**.
  See the resolved-deviation section above for both magnitudes and the two caveats.

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
