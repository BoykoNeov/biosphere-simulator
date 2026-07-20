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

### The param files — 13 clean-room biosphere param files (phenology grew, scope B inc. 1)

`src/domains/biosphere/params/*.yaml` minus `demo.yaml`: `canopy`, `photosynthesis`,
`respiration`, `transpiration`, `phenology`, `allocation`, `senescence`, `nitrogen`
(Phase-1 producer); `decomposition`, `microbial_respiration`, `mineralization` (Phase-2
decomposer + N return loop); `water_cycle` (Phase-3 water closure); `herbivory` (Phase-3
consumer). Each is clean-room from primary literature
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
> now cause 3 — param values**: the `tsum` phase partition (reproductive phase too short),
> whose recalibration is the deferred scope-B ceremony 2, to be moved only within cited
> literature ranges (**the oracle is a diagnostic, never a fit target**). The frozen
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

## Phase-5 handoff

The biosphere is frozen as **THE reference**. Phase 5 builds sibling domains (power / thermal /
atmosphere-ECLSS / crew), each verified **standalone against its own references first**, then
against this frozen biosphere — never the reverse (a sibling does not get to move the
reference). The reference moves only through the unfreeze discipline above.
