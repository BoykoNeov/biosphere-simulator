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

### Locked integrator + dt — **Euler, `dt = 1.0 day`**

The biosphere runs the **forward-Euler** integrator at a **one-day** step (`t = n·dt`, integer
step count). This was **locked by probe, with evidence** (P4.1, Step 1): both closed scenarios
were run Euler *and* RK4 to 15 yr and structurally agreed (both stationary, both closed, same
period class); the 100k-step stress (Step 3, 328 yr) confirmed no slow drift. RK4 ships in
`simcore` but the biosphere does **not** use it — crop physiology is daily-integrated and the
daily canopy flux is not RK4-refinable. The integrator + dt have **no importable constant**
(each regression run helper selects `EulerIntegrator(...)` and `dt = 1.0` inline); they are
**documented** in the manifest and **enforced by the goldens** — an integrator or dt switch
moves every committed golden.

### The flow set + the aux processes

The flow classes assembled across the canonical scenarios — the frozen flow taxonomy. The
manifest's `flow_set` is **derived from freshly assembled registries** (the union over the open
field + the three chambers), never hand-listed, so a flow added to any compartment builder is
caught by the completeness gate even if no golden exercises it. As frozen, the set is the 17
classes spanning the producer (allocation, the two respirations, senescence, transpiration,
nitrogen uptake/senescence, the forcing-driven irrigation/fertilization), the decomposer
(decomposition, microbial respiration, mineralization), the water cycle (condensation,
recycling), and the consumer (grazing, consumer respiration, consumer mortality).

**Gross carbon assimilation is not a flow** (and not an aux): it is a recomputed *quantity*
inside the shared `CarbonContext` budget — the `GrossAssimilation` flow was *dissolved* in the
Phase-1 Step-11 buffer rewiring — entering the system through the `Allocation` flow's
`co2_atmos → organs` leg. So there is no `Photosynthesis`/`GrossAssimilation` class in
`flow_set`; that science is frozen via `Allocation`. The manifest also freezes the
**`aux_set`** (the registries' non-conserved accumulators, derived symmetrically from the
public `registry.aux_processes`) — the thermal-time / DVS accumulator that drives
allocation, and (since post-roadmap scope (B) increment 1) the **vernalization-days**
accumulator that gates it — so a future aux process added but wired into no golden is caught
too. (See `flow_set` / `aux_set` in the manifest for the exact lists.)

### The param files — 12 clean-room biosphere param files

⚠ **12, not 13: `mineralization.yaml` was RETIRED**, and it is the first param *file* this
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
5. **Record provenance.** Update this file and the Phase-4 plan with what changed and why (a
   calibration cites its primary source per `docs/param-file-conventions.md`).
6. **Re-run the gates:** full suite (incl. `-m slow` for the stress), `ruff`, `pyright`; commit
   with a Conventional Commit that names the unfreeze.

An undocumented unfreeze fails CI by construction (a moved golden, or the completeness gate),
so the discipline is enforced, not merely requested.

### Unfreeze log

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
