# The station reference (frozen) — Phase 6, P6.10

Phase 6 integrated the five domains — the frozen biosphere plus the four Phase-5 siblings
(power / thermal / eclss / crew) — into one coupled station, closing matter **and** energy
through shared stocks. Step 10 freezes that **whole assembly** as the project's
**multi-domain reference**: the stable engine Phase 7's native (Rust) port targets verbatim
(roadmap line 7: *"We port a stable multi-domain engine, not an evolving one"*). This file
is the station **freeze contract** — what is frozen, the evidence the freeze rests on, and
the **unfreeze discipline** for ever changing a frozen item.

It is the [`docs/biosphere-reference.md`](biosphere-reference.md) discipline one assembly
level up. Like it, this is **boundary-side docs + a manifest only**: `git diff
src/simcore/` stays **empty** and `src/domains/` is **untouched**, unconditionally. Its
machine-readable companion is **`docs/station-reference.manifest.json`** (generated; see
*The manifest* below). The plan of record is
[`docs/plans/phase-6-station-integration.md`](plans/phase-6-station-integration.md).

## Whole-assembly scope — and the biosphere delegation

Step 10 freezes the **whole integrated station**: the Phase-5 siblings' flow classes +
param files, the four station-owned seams + three station params, and the 13
station/sibling scenarios → goldens. The **biosphere is delegated** — it was frozen in
Phase 4, so this reference **references** `docs/biosphere-reference.manifest.json` (the
manifest's `delegates_to` field) rather than re-freezing it. A change to a biosphere item
follows *its* unfreeze discipline; a change to a sibling or station item follows *this*
one.

**Why whole-assembly, not station-layer-only** (advisor-reviewed, user-confirmed). A
station-layer-only freeze (owning just the four seams + three params) would leave the
sibling flows and params changeable with **no unfreeze ceremony — in exactly the layer
Phase 7 ports**. That is a silent-change hole. Freezing the siblings closes it. The
sibling multi-domain evidence already exists: the Tier-2 3-year sealed run
(conservation + longevity across all five domains) and the Step-9 NASA BVAD crew
validation.

**Frozen ≠ calibrated (the "frozen-but-illustrative" caveat).** Freezing an item does
**not** claim it is calibrated — only that changing it is a documented, reviewed,
re-captured event. Several frozen coefficients are deliberately **illustrative**, carried
as such, not hidden:
- The **ECLSS** rate-constants (`k_scrub` / `k_cond` / `k_makeup`, `o2_setpoint`) and the
  **station** `harvest_rate` / `recovery_rate` / `recovery_efficiency` / `photon_efficacy`
  are illustrative sizing — BVAD publishes no first-order τ, only steady-state throughput
  (which the closure checks validate). Step 9 explicitly kept them illustrative.
- The **crew** physiology fractions (`respired_carbon_fraction` = 0.949,
  `insensible_water_fraction` = 0.675) **are** literature-bound (NASA/TP-2015-218570 Rev 2
  Table 3-31 + Rose et al. 2015; Step 9). The one structural residual — RQ = 1 forces
  crew O₂ consumption ~11.8 % below BVAD — is *measured and pinned*
  (`tests/test_bvad_validation.py`), not a freeze omission.
This mirrors the biosphere, which froze uncalibrated `TODO(cite)` crop params behind a
documented-finding note. A calibration pass is a future, deliberate unfreeze.

## What "frozen" means (and what it does NOT)

**Frozen** = the items below are the committed reference. A change to any of them is an
**unfreeze event** that must follow the discipline at the bottom of this file. Freezing is
a *process* discipline, **not a code lock**: nothing forbids editing a param file; the
goldens + the manifest gate make an undocumented change *fail CI*, which is what gives the
freeze teeth.

**The terminological transition Step 10 makes.** Through Steps 1–9 the 13 station/sibling
goldens were "**additive NON-frozen**" (the Power-domain golden discipline — pinned, but
freely regenerable, as Step 9 did for six of them). Step 10 **promotes them to the frozen
station reference**: regenerating one now is an **unfreeze event** with ceremony, not a
casual `__main__` re-run. (The whole-station golden *capture* itself was already done —
Step 7's `sealed_station_state.json` + `sealed_energy_drift_summary.json` are it; Step 10
adds no new golden, only the contract that freezes them.)

## The frozen surface

The manifest is the authoritative, machine-checked list. This section is the
human-readable account.

### Locked integrator — **Euler everywhere**; dt per scenario

Every station/sibling scenario runs **forward-Euler** (`t = n·dt`, integer step count).
The dt varies by scenario and is **not** an importable constant (each run helper selects
it inline), so the manifest *documents* `integrator = "EulerIntegrator"` + a per-scenario
note and the **goldens enforce** it (an integrator or dt switch moves every committed
golden). The **sealed reference** is two-rate: biosphere-slow **`dt = ¼` day, four slow
sub-steps per master day** + everything-fast **`dt = 60 s`** (ECLSS's binding
`k_scrub·dt < 1`), stepped by `station.driver.run_master_day`. The Tier-1 energy loop is
single-rate **`dt = 3600 s`** (`station.system.run_station`, where `n` advances so the
diurnal SOC swing + the SB radiator's emergent `T_eq` attractor are expressible). The
biosphere carries its own Euler/`dt` lock (its manifest); the station does **not** re-declare
it — `bio_dt` / `bio_steps_per_day` bind to `domains.biosphere.step`.

⚠ **Two things about `n` that were true here until 2026-08-14 and are not any more.**
(a) **`n` is NOT the master-day count** — it is the slow domain's *step* count, so under the
sealed reference it is 4× the day count. Any calendar computed from `n` (a re-sow period, a
perturbation window) must be converted with `steps_for`; three call sites and two docstrings
in this assembly asserted the old identity and were corrected. (b) **`states` is still one
entry per master day** — `slow_steps_per_day` did not change that, so station trajectories
stay **day-indexed** while biosphere trajectories are **step-indexed**. That asymmetry is
load-bearing: slicing a station trajectory works in days, and "fixing" those sites to use
steps introduces the bug it looks like it removes.

### The flow set — 16 sibling + station flow classes (derived)

The frozen flow taxonomy of the coupled station, **derived from freshly assembled
registries** (never hand-listed): the union over the four standalone sibling registries
(`build_power` with `SelfDischarge`, `build_thermal`, `build_eclss`, `build_crew`) **and**
the maximal sealed **fast** registry (`build_sealed_station(..., with_harvest=True)`), so a
flow wired into any sibling or the station assembly is caught even if no golden exercises
it. The 16 classes:

- **power** — `SolarCharge`, `LoadDraw`, `SelfDischarge`
- **thermal** — `HeatInput`, `RadiatorReject`
- **eclss** — `CrewMetabolism`, `CO2Scrubber`, `Condenser`, `O2Makeup`
- **crew** — `OxygenConsumption`, `FoodMetabolism`, `WaterBalance`
- **station seams** — `CrewRespiration`, `WaterRecovery`, `Lamp`, `Harvest`

The five *dropped* stand-ins (`HeatInput`, `CrewMetabolism`, `OxygenConsumption`,
`FoodMetabolism`, `SelfDischarge`) exist only in the **standalone** sibling builds — pinned
by the standalone sibling goldens — which is why the derivation unions those, not only the
coupled fast registry. The biosphere's slow registry is **never** included (delegated), so
no biosphere flow (`Allocation` / `MicrobialRespiration` / …) appears here. The `aux_set`
is empty — the siblings + station carry no non-conserved accumulator (the biosphere's
`ThermalTimeAccumulation` lives in the delegated slow registry) — but the *set* is frozen
so a future aux is caught.

### The eight param files

`src/domains/{power,thermal,eclss,crew}/params/*.yaml` + `src/station/params/*.yaml`:
`charge`, `self_discharge` (power); `radiator` (thermal); `eclss` (eclss); `crew` (crew);
`water_recovery`, `lamp`, `harvest` (station). Each is clean-room from primary literature
or illustrative sizing per the frozen-but-illustrative caveat above; the manifest records a
newline-normalized sha-256 of each as **provenance**. Biosphere param files are **not**
recorded here (delegated).

### The 13 scenarios + their goldens

Step 10 invents **no new scenario** and adds **no new golden** — it pins the surface Steps
1–9 built:

| Scenario | Step | Golden |
| --- | --- | --- |
| `BOUNDED_SOC_SCENARIO` (Power) | P5.2–4 | `power_state.json` |
| `SELF_DISCHARGE` (Power + leak) | P5.5 | `power_self_discharge_state.json` |
| `EQUILIBRIUM_SCENARIO` (Thermal) | P5 (thermal) | `thermal_state.json` |
| `STEADY_STATE_SCENARIO` (ECLSS) | P5 (eclss) | `eclss_state.json` |
| `MISSION_SCENARIO` (Crew) | P5 (crew) | `crew_state.json` |
| `HEAT_CLOSURE_SCENARIO` (Power→Thermal) | P6.1 | `station_state.json` |
| `CABIN_GAS_SCENARIO` (crew↔ECLSS) | P6.2 | `cabin_gas_state.json` |
| `GREENHOUSE_SCENARIO` (biosphere↔cabin) | P6.3 | `greenhouse_state.json` |
| `WATER_RECOVERY_SCENARIO` | P6.4 | `water_recovery_state.json` |
| `LIGHTING_SCENARIO` (Power→biosphere) | P6.5 | `lighting_state.json` |
| `HARVEST_SCENARIO` (biomass→food) | P6.6 | `harvest_state.json` |
| `SEALED_STATION_SCENARIO` (Tier-2, 4 yr) | P6.7 | `sealed_station_state.json` |
| `HEAT_CLOSURE_SCENARIO` 15-yr (Tier-1) | P6.7 | `sealed_energy_drift_summary.json` |

The two sealed horizons are importable constants (`SEALED_STATION_YEARS = 4`,
`SEALED_ENERGY_YEARS = 15`, `station/scenario.py`) recorded in the manifest and asserted
against those constants, so the frozen horizons cannot drift. Each golden is a hex-float
byte snapshot via `sim_io` (the energy drift-summary is the per-year peak-node-temperature
vector + the period class). They are bit-identical **within a build**; the coupled runs use
transcendentals (`exp`/`pow`/`sin` in weather / FvCB / the SB radiator), so cross-platform
last-ULP differences are **tolerance territory** (the cross-port concern), not a freeze
violation.

### Not part of the station reference (scoped out, by name)

- The **frozen biosphere** — **delegated**, not excluded: it is frozen by
  `docs/biosphere-reference.manifest.json` (the manifest's `delegates_to`).
- The **Phase-0 engine-skeleton demo** goldens (`demo_euler_state.json`,
  `demo_rk4_state.json`, `state_snapshot.json`) — no real science, frozen by their own
  Phase-0 goldens.
- The two **NON-frozen biosphere stress scenarios** (`n_limited_state.json`,
  `water_biting_state.json`) — deliberately non-frozen scenario *data* (the biosphere doc
  scopes them out too).
- The **cross-domain perturbation harness** (`src/station/perturbations.py`) — diagnostics,
  **no golden** (the Phase-3 `perturbations.py` precedent; determinism re-runs are the
  insurance). Its `ScaledFlow` is perturbation-only, so it is deliberately **not** in the
  frozen flow set.

## The evidence the freeze rests on

The freeze is earned by Phase 6 Steps 1–9 (full detail + measured numbers in the plan):
- **Conservation holds every step, every quantity + ENERGY, across the whole assembly.** The
  Tier-2 sealed run (~3 yr, ~1.3 M sub-steps) asserts the combined ledger after **every**
  fast sub-step; relative day-boundary drift is flat at round-off for CARBON / OXYGEN /
  WATER / NITROGEN **and** ENERGY.
- **Energy earns a genuine subsystem attractor** (Tier 1): the SB radiator node settles to
  a period-1 fixed point at the dissipation-set `T_eq ≈ 160 K`, SOC daily-periodic, ENERGY
  drift flat over 15 yr.
- **Matter earns conservation + regulated-pool stationarity + a period-1 plant** (Tier 2):
  the ECLSS / recovery loops hold CO₂/O₂/H₂O at setpoints; the pinned-CO₂ coupled biosphere
  is period-1 with a converging decomposer pool. Whole-system matter stationarity is
  **deferred** (stores drain, feces open) — a characterization, not a closed ecosystem.
- **Cross-domain cascades emerge with no cascade code** (Step 8): brownout / radiator
  failure / leak / crew spike / lighting failure propagate through shared stocks alone; the
  station regulators erase the naive pool-level signature (the signature is regulator
  *effort* + sinks).
- **Integrated crew metabolism is validated against NASA BVAD** (Step 9): the one un-tuned
  output (RQ) is pinned; the ~11.8 % O₂ residual is measured, not hidden.

Tests of record: `tests/test_sealed_station_stability.py` (Tier 1 + Tier 2, marked-slow),
`tests/test_sealed_station_landmine.py` (Tier 3), `tests/test_regression_sealed_station.py`,
`tests/test_station_perturbations.py`, `tests/test_bvad_validation.py`, and each step's
`test_*_run.py` + `test_regression_*.py`.

## The manifest

`docs/station-reference.manifest.json` is the machine-readable surface, **generated** by
`tests/test_station_freeze_manifest.py` (`uv run python
tests/test_station_freeze_manifest.py`). It names the integrator, the two sealed horizons,
the derived flow set + aux set, the eight param files (+ provenance hashes), each scenario
→ golden (+ hash), and the `delegates_to` pointer to the biosphere manifest.

**What the manifest gate checks vs. what the goldens check** — the division is deliberate
(the biosphere manifest's exact split):
- **The scenario goldens own *values*.** Any value change to a frozen param, a flow law, or
  the integrator/dt already moves a committed golden and fails its byte-compare. The
  manifest does not re-assert that; its hashes are **provenance only**, regenerated on a
  deliberate unfreeze.
- **The manifest gate owns *completeness*** — the one thing the goldens are blind to: a
  param file, flow class, or aux process added to the frozen tree but wired into no golden.
  The gate asserts the frozen *sets* against the live tree (and a teeth test confirms it
  fails on an unfrozen file). A new-but-unfrozen param/flow/aux fails the gate; that is the
  signal to either freeze it (an unfreeze) or remove it.
- **`science_bands` + `liveness_floors` own the *science*** — added 2026-08-09; see
  `docs/biosphere-reference.md` for what the two names mean and why they are kept apart, and
  `docs/plans/post-roadmap-acceptance-gate-standing.md` for the inclusion rule.

  ⚠ **On the station side the measured result is mostly EMPTY, and that is the finding rather
  than a gap.** **11 of the 13** station scenarios carry no outside-sourced bound at all —
  established mechanically, by there being no module-level sourced constant in any station
  run-test. Only `crew_mission` has a band (BVAD Table 3-31's RQ) and only `sealed_station` has
  a floor (the thermal node must not collapse toward `T_space`). Freezing the emptiness is the
  point: the absence is now a recorded claim that a future band cannot be added around silently,
  instead of an unexamined assumption. Every roster scenario gets an **explicit empty list** —
  an absent key and a deliberately-empty one are different claims.

## The unfreeze discipline

Changing **any** frozen station/sibling item — a param value, a flow, a scenario knob, the
integrator/dt, a sealed horizon, or adding a new param/flow — is an **unfreeze**. (A
biosphere change follows *its* discipline instead.) The procedure:

1. **Justify + review.** Write down *why* (a calibration source, a new process, a bug). For
   a science or numerical change, get it **advisor-reviewed** before regenerating anything.
2. **Make the change** boundary-side. `git diff src/simcore/` **must stay empty** and
   `src/domains/` changes are domain-side data/citation edits only (a sibling param is a
   Phase-5 domain param, not a `simcore/` change).
3. **Regenerate the affected goldens**, each via its own explicit `__main__` action, and
   **review the byte diff** — a change there means the trajectory moved, which is the point.
4. **Regenerate the manifest** (`uv run python tests/test_station_freeze_manifest.py`) and
   review its diff — the changed hashes / flow set / param set are the git-visible record of
   exactly what was unfrozen.
5. **Record provenance.** Update this file and the Phase-6 plan with what changed and why (a
   calibration cites its primary source per `docs/param-file-conventions.md`).
6. **Re-run the gates:** full suite (incl. `-m slow` for the sealed stability), `ruff`,
   `pyright`; commit with a Conventional Commit that names the unfreeze.

An undocumented unfreeze fails CI by construction (a moved golden, or the completeness
gate), so the discipline is enforced, not merely requested.

### Unfreeze log

- **2026-08-14 — the biosphere's integration step moves to `¼` day (biosphere-delegated;
  4 station goldens + `numerics_note` + `run_master_day`).**
  `docs/plans/post-roadmap-step-unfreeze.md`. Authorized by the user. The science reason is
  entirely biosphere-side (see its reference's resolved-deviation section); what makes this a
  **station** unfreeze is that the driver had to learn to sub-step the slow domain, and the
  contract states the step in prose.

  **What changed.** `run_master_day` (and its Rust mirror, and the Phase-8 session that
  shares `advance_one_master_day` with it) takes `slow_steps_per_day`, defaulting to `1` so
  the change was provably inert on its own — 272 station tests, including the byte-exact
  goldens, passed before the step moved. A `slow_dt · slow_steps_per_day == 1 day` guard was
  added, the symmetric partner of the existing `fast_dt · steps_per_day == 86400 s`. The
  three scenarios' `bio_dt: 1.0` literals now bind to `domains.biosphere.step`, so the
  station cannot desync from the biosphere's step. `sealed_reset`'s period converted from
  days to steps.

  ⚠ **The re-sow period was correct by ACCIDENT, not by design.** `n % season_days` with
  `n = 4·day` still fires on the right days only because 305 is odd; at `season_days = 304`
  the same line would re-sow **four times a year**. Converted deliberately so the
  correctness does not rest on a coprimality nobody had written down.

  ⚠ **`numerics_note` is honor-system and this is the record that it was maintained by
  hand.** The string lives as a literal in the manifest *generator*, compared only against a
  manifest generated from that same literal — so flipping `bio_dt` reddens nothing here. The
  biosphere side is different (`dt_days` is asserted against a hard-coded number and failed
  loudly, as designed). Do not assume the loud gate on that side covers this one.

  **Verification.** The four station goldens' step counter went 7 → 28 (greenhouse, harvest,
  lighting) and 1220 → 4880 (sealed station), each exactly as predicted before regenerating;
  the eight biosphere-free station goldens are **byte-identical**, and
  `sealed_energy_drift_summary` regenerated bit-for-bit identical. Every station trajectory
  length is unchanged, because `states` still appends once per master day.

- **2026-08-11 — the soil-layers cascade (biosphere-delegated; 4 station goldens, no
  station-side science).** `docs/plans/post-roadmap-soil-layers.md`. The biosphere gained a
  `subsoil_water` stock and a `RootZoneCapture` flow (its own manifest carries them); the
  four station scenarios that embed a biosphere — `greenhouse`, `harvest`, `lighting`,
  `sealed_station` — regenerated. The biosphere-free goldens
  (crew/eclss/cabin/water_recovery/power/thermal/station/sealed_energy) are
  **byte-identical**, and so is `sealed_energy_drift_summary`.

  ⚠ **The station-side check that mattered is the WATER LOOP.** Three tests summed the
  biosphere's internal ring as `soil_water + water_vapor + condensate`; the below-root
  store is in-system soil water crossing no boundary, so leaving it out reads a conserved
  transfer as a leak. All three (Python `greenhouse`/`lighting`, Rust
  `day_neutral_lighting`) now sum four stocks. **`harvest` moved no amount at all** — its
  crop starts past anthesis at the rooting cap, so the extension rate and therefore the
  capture are zero. `delegates_to` biosphere.

- **2026-08-09 — the science assertions get contract standing (a SCHEMA unfreeze; NO value,
  golden, param or `src/` change).** `docs/plans/post-roadmap-acceptance-gate-standing.md`.
  Added `science_bands` + `liveness_floors`, derived from `science_gate` markers. Station-side
  content is `crew_mission`'s BVAD RQ band and `sealed_station`'s node floor; the other 11
  scenarios are explicitly empty, which is a measured result — see the manifest section above.

- **2026-07-21 — scope (B) decomposer-calibration cascade (biosphere-delegated values +
  a sealed horizon).** The biosphere unfreeze (decomposer rates 0.02→0.011 / 0.05→0.016;
  see `docs/biosphere-reference.md`) cascaded to the four station scenarios that embed a
  **sealed** biosphere: `greenhouse`, `harvest`, `lighting`, `sealed_station` goldens
  regenerated (the biosphere-free goldens — crew/eclss/cabin/water_recovery/power/thermal/
  station/sealed_energy — are byte-identical). **`SEALED_STATION_YEARS` moved 3 → 4**: the
  calibration enlarged the biosphere soil-pool equilibria ~2–3×, so the sealed_station's
  **year-1 soil-establishment spin-up** (the `annual_reset` plant-dump, ~60 mol C into
  litter) now spans a full year; 4 seasons give the biomass watch two settled post-spin-up
  same-phase diffs, and the pre-golden gate + `test_sealed_station_stability` skip the
  spin-up via `is_stationary(transient=1)` (bound unchanged at 1.0 — a documented spin-up
  skip, not a relaxed amplitude bound). 4 is also the max `rationed==0` horizon (year 5
  rations, year 6 collapses — both **measured** pre-existing and rate-independent: OLD and
  NEW rates both ration at year 5 with the identical count, so the calibration lengthened
  the soil-settling transient, not the stable window). The manifest's `sealed_station_years` + the four station
  golden hashes moved; `delegates_to` biosphere. Advisor-reviewed. Full record:
  `docs/plans/post-roadmap-decomposer-calibration.md`.

## Phase-7 handoff

The station is frozen as **THE multi-domain reference**. Phase 7's native (Rust) port
targets this frozen assembly — the biosphere (its own manifest) + the four siblings + the
station seams — porting it verbatim, tolerance-gated cross-port (the transcendental
last-ULP caveat). The reference moves only through the unfreeze discipline above.
