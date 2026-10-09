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

⚠ **Since 2026-10-03 the delegation has one station-owned exception: WHEN the crop's carbon
budget is stepped.** Wherever the cabin's flows act on the crop's air (`greenhouse`, `harvest`,
`sealed_station`), the station takes the crop's three carbon-budget flows (`Allocation`,
`GrowthRespiration`, `MaintenanceRespiration`) out of `build_season`'s registry and steps them,
unchanged, on its fast (60 s) registry; the plant step records the light and temperature they
read (`station::gas_exchange`). The flows' science stays the biosphere's and delegated; the
STEP they are taken on is this contract's. `lighting` and every standalone biosphere scenario
keep the plant step. The registry is no longer `build_season`'s verbatim in those three.

⚠ **Since 2026-10-05 the two lamp seams also own the crop's NET RADIATION** (`sealed`,
`lighting`): `(1 − 0.23) ×` the lamp's radiant PAR while lit and 0 in the dark, where before
they replaced PAR and day length but left transpiration on the weather file's outdoor value.
The formula's science stays the biosphere's (`weather::net_shortwave`); which light it is fed is
this contract's. See the unfreeze log.

⚠ **Since 2026-10-05 (slice 3a) the sealed station also owns the crop's TEMPERATURE.** Every
plant-step flow and aux process of `sealed_station` is wrapped to read `temp` from the plant
chamber (`thermal.chamber`, `Q / C − 273.15`; `station::chamber::plants_read_chamber`), and the
chamber runs the cold program of `cold_period.yaml`. The sealed plant resolver carries no
temperature, and a plant-side one is refused. Since slice 3b the same program dims the lamp in
the cold period (`cold_par` 100 µmol m⁻² s⁻¹ for `cold_photoperiod_hours` 8, a 40 W draw): the
crop's PAR, net radiation and day length and the lamp's draw all follow it. The
temperature-dependent science stays the
biosphere's; which temperature it reads is this contract's. The chamber's walls still read the
weather file's daily temperature (the outdoors they face). `greenhouse` and `harvest` still feed
their plants the weather's temperature; `lighting` the weather's, or its scenario's
`habitat_temp_c` constant when set (`lighting_bio_resolver`).

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
golden). The **sealed reference** is two-rate: biosphere-slow **`dt = 1/16` day, sixteen slow
sub-steps per master day** + everything-fast **`dt = 60 s`** (ECLSS's binding
`k_scrub·dt < 1`), stepped by `station.driver.run_master_day` **interleaved**: each slow
sub-step is followed by its sixteenth of the day's fast sub-steps (90), so the cabin refills
the shared air between the plant's draws (¼ day and 360 until the 2026-09-30 step change; where
the two counts do not divide evenly the day runs in equal groups, `driver::day_groups`, see the
log). It ran **slow-first** (all the slow sub-steps,
then all 1440 fast ones) until the 2026-09-30 unfreeze logged below. The Tier-1 energy loop is
single-rate **`dt = 3600 s`** (`station.system.run_station`, where `n` advances so the
diurnal SOC swing + the SB radiator's emergent `T_eq` attractor are expressible). The
biosphere carries its own Euler/`dt` lock (its manifest); the station does **not** re-declare
it — `bio_dt` / `bio_steps_per_day` bind to `domains::biosphere::{BIO_DT, STEPS_PER_DAY}`.

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
- **station seams** — `CrewRespiration`, `WaterRecovery`, `Lamp`, `Harvest`, `ChamberCooling`,
  `ChamberWall`, `ChamberHeater`
  (since 2026-10-05: the plant chamber's cooler, chamber → `thermal.node`; the sealed `Lamp`'s
  whole draw — waste heat and light — goes to `thermal.chamber`, and the sealed build has no
  `boundary.light_used`; since 2b-iii the chamber's walls exchange heat with the outdoor
  weather through `boundary.chamber_surroundings`, and a battery heater holds its floor)

The five *dropped* stand-ins (`HeatInput`, `CrewMetabolism`, `OxygenConsumption`,
`FoodMetabolism`, `SelfDischarge`) exist only in the **standalone** sibling builds — pinned
by the standalone sibling goldens — which is why the derivation unions those, not only the
coupled fast registry. The biosphere's slow registry is **never** included (delegated), so
no biosphere flow stepped on the plant step (`MicrobialRespiration` / …) appears here. ⚠ Since
2026-10-03 three biosphere types DO: `Allocation`, `GrowthRespiration` and
`MaintenanceRespiration` are stepped on the sealed station's fast registry (the exception
above). The plant-step `PlantWindowRecorder` that feeds them is in the excluded slow registry,
so no freeze record lists it. The `aux_set`
is empty because every aux process lives in the excluded slow registry — the biosphere's
`ThermalTimeAccumulation` and, since 2026-10-03, the station's own `PlantWindowRecorder` —
not because the station carries none (it carries that one). The *set* is frozen so a future
aux on a fast registry is caught.

### The ten param files

`rust/crates/domains/params/{power,thermal,eclss,crew}/*.yaml` + `rust/crates/station/params/*.yaml` (both moved out of the Python packages in Stage-3 slice S1, 2026-08-18; a pure rename — the manifest keys on basenames, so not one hash moved):
`charge`, `self_discharge` (power); `radiator` (thermal); `eclss` (eclss); `crew` (crew);
`water_recovery`, `lamp`, `harvest`, `chamber`, `cold_period` (station; `chamber` and `cold_period` added 2026-10-05). Each is clean-room from primary literature
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
- The **Phase-0 engine-skeleton demo** — no real science.
  ⚠ Its two goldens (`demo_euler_state.json`, `demo_rk4_state.json`) were **deleted
  2026-08-18** (C6 of the reference flip). `state_snapshot.json` stays — a hand-authored
  `sim_io` fixture the reference *reads*, not a run.
- The two **NON-frozen biosphere stress scenarios** (`n_limited`, `water_biting`) — scenario
  *data*, scoped out by the biosphere doc too.
  ⚠ **RETIRED 2026-08-18** with their goldens (same slice). Neither name ever appeared in
  this manifest, so nothing frozen here moved.
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
the derived flow set + aux set, the ten param files (+ provenance hashes), each scenario
→ golden (+ hash), and the `delegates_to` pointer to the biosphere manifest.

⚠⚠ **Since 2026-08-16 (slice 7 of the reference flip) this file has MIXED AUTHORITY, and
regenerating it needs `cargo`.** The keys the Rust reference tree can produce — `flow_set`,
`aux_set`, `sealed_station_years`, `sealed_energy_years` — are read out of it by shelling
`cargo run --example dump_station_inventory`; the rest is still the checker's or
hand-written. **The manifest states this itself**, per key, in its own `_authority` block,
which is the thing to read before assuming any field is Rust-derived.

⚠⚠ **`science_bands` + `liveness_floors` re-anchored to Rust on 2026-08-18 (slice C4b), and
the paragraph that stood here said they could not.** It read *"a static census of pytest
markers with no Rust referent"*, and named the referents the reference was missing: the RQ
helper and `predicted_equilibrium_temperature`. **The second half of that was already
false when it was written** — `predicted_equilibrium_temperature`, the drift folds and the
15-yr energy run were all in `rust/crates/station` — so C4b came in under its own estimate.
This contract's two claims are declared in `rust/crates/station/src/science_gates.rs` now
(the same exported `science_gates!` macro the biosphere's 13 use, in a second table because
a gate lives with the runs it reads and these read `station` types). Only the two `locus`
strings moved; `quantity`/`bound`/`source` are byte-identical, and the Python test bodies
stay as the checker's conformance half. ⚠ `sealed_energy_drift`'s golden hash is still a
Python-side fold of a raw Rust series.

⚠ **`param_files` joined the Rust half on 2026-08-17 (slice C8)** — what
re-anchored there is the *census* rule (the eight files the reference **loads**, not a glob
over six Python package directories) and the *normalization* rule, since the eight digits are
author-neutral either way; the log entry below carries the detail and the two things it
newly asserts. Two consequences a reader will otherwise miss:

- **The completeness gates changed meaning without changing their arithmetic.** They used
  to say *the manifest froze everything Python has*; they now say *Python still matches the
  reference*, and a failure is a **checker** drift. The completeness question itself moved
  to `tests/crossport/test_inventory_parity.py`, which compares the committed manifest
  against a freshly built Rust tree.
- **`sealed_energy_years` is `LONG_HORIZON_YEARS` in the reference tree** — the same
  constant the biosphere manifest freezes. Moving the decade horizon is one reference-side
  edit that unfreezes *two* contracts.

⚠ **What slice 7 deliberately did NOT close: the `numerics_note` steps are still ungated
prose.** That string carries the station's dt values as hand-maintained English, and
nothing compares it to anything (the manifest generator's own literal is the only thing it
is checked against, so the two agree whatever the code does). The reference tree *does*
have referents for those numbers — the sealed scenario's `bio_dt`/`cabin_dt` and the energy
scenario's `power_dt` — so the biosphere's `dt_days` treatment is buildable here. It needs
a structured manifest key that does not exist, and adding one **widens the frozen surface**,
which is its own unfreeze with its own ceremony rather than a rider on a re-anchoring. The
hole is recorded in that key's `_authority` entry rather than left implicit.

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
2. **Make the change** boundary-side. `git diff rust/crates/simcore/` **must stay empty**
   and `rust/crates/domains/` changes are domain-side data/citation edits only (a sibling
   param is a Phase-5 domain param, not a `simcore` change). ⚠ This step named `src/simcore/`
   and `src/domains/` until 2026-08-27; S6 deleted both, so the check it prescribed had no
   subject.
3. **Regenerate the affected goldens** — from `rust/`, `cargo run --release -q -p station
   --example regen_goldens -- --write` (or `--only <substring>` for one) — and **review the
   byte diff**: a change there means the trajectory moved, which is the point. ⚠ This step
   said *"each via its own explicit `__main__` action"* until 2026-08-27; those were the
   Python regeneration mains S6 deleted, and the blessed path is `station::regen` (S6 build
   item 2). Reporting is the default; `--write` is explicit, and every candidate is
   validated against its declared shape before **any** byte is written.
4. **Regenerate the manifest** — from `rust/`, `cargo run --example
   dump_station_inventory -- --write-manifest` — and review its diff: the changed hashes /
   flow set / param set are the git-visible record of exactly what was unfrozen. ⚠ **The
   command changed on 2026-08-18 (C7's station half)**; it was `uv run python
   tests/test_station_freeze_manifest.py`, which now has no writer at all and is a checker
   only. The step has needed a Rust toolchain since 2026-08-16 and now *is* the Rust
   toolchain. **Predict the diff before running it** — a re-anchored key that
   moves when you expected it not to is a finding, not a diff to accept.
5. **Record provenance.** Update this file and the Phase-6 plan with what changed and why (a
   calibration cites its primary source per `docs/param-file-conventions.md`).
6. **Re-run the gates:** full suite (incl. `-m slow` for the sealed stability), `ruff`,
   `pyright`; commit with a Conventional Commit that names the unfreeze.

An undocumented unfreeze fails CI by construction (a moved golden, or the completeness
gate), so the discipline is enforced, not merely requested.

### Unfreeze log

- **2026-10-09 — the biosphere's chamber condenser draws nothing below its setting, delegated: 3
  station goldens move (`greenhouse`, `harvest`, `sealed_station`) and their 3 `golden_sha256` rows
  follow. No station flow, param, seam or claim changed.** The biosphere entry of the same date has the
  form; plan `docs/plans/post-roadmap-chamber-dehumidifier.md`. `greenhouse` / `harvest`: condensate
  −0.25 %, soil +0.04 % / +0.007 %; `sealed_station` only at ~1e-9. Water only. `lighting` was predicted to
  move and is byte-identical (not traced). The cabin's own condenser (2026-10-03) is a different one-sided
  form and is untouched; `cabin_gas`, `eclss` and `water_recovery` are byte-identical.
- **2026-10-06 — the crop is re-sown when it MATURES, slice 4 stage 2: 1 golden moves
  (`sealed_station`).** `docs/plans/post-roadmap-room-temperature.md` §25f (predictions,
  committed before code) and §25g (graded). The sealed re-sow hook fires on the first master-day
  start with the crop's development stage at 2 (`SealedStationScenario::pheno`, the
  `phenology.yaml` the crop is built from), no longer on the 305-day calendar; the cold program
  runs the first 56 days after each sowing and never recurs within a crop (`mod season_days`
  gone). `season_days` is now only the weather's tiling period and the horizon's unit (1220
  days). **No flow-set or param change.** Measured: re-sows at days 139, 278 … 1112 (9 crops,
  every one maturing 139 days after its sowing); each re-sows from **14.58 mol C** of grain
  (the calendar's crop had 41.02 — 64 % of it formed by `Allocation` AFTER maturity, measured);
  504 cold days instead of 224, so the battery ends **+2.889 GJ** (1.11505e10 J) and the node
  0.151 K colder at the end, 2.25 K colder on average (169.1984 K); the chamber's cold hold
  is 0.004 K looser (the cold weeks now meet every part of the weather year); the end crop is 108 days old (grain 0.896, thermal time 1169.84). The CO₂
  scrubber and the O₂ makeup each work 8.870 mol harder over the horizon (predicted the other
  way — the biosphere ends holding 8.870 mol C less: the end crop's grain gone, the soil up by
  eight crops' residues). No rationing, no events. The other 19 goldens byte-identical; the
  manifest moves by the golden's hash.
- **2026-10-06 — the cold program reads the crop's own sowing, slice 4 stage 1: one aux key
  added to 1 golden (`sealed_station`); every stock bit-identical.**
  `docs/plans/post-roadmap-room-temperature.md` §25 (design and predictions, committed before
  code) and §25e (graded). The state carries `station.sown_step`, the slow step of the standing
  crop's sowing — seeded by the sealed build, rewritten by the sealed re-sow hook (not by
  `annual_reset`, which the biosphere's own runs share). The five variables the cold program
  drives (`par`, `net_radiation`, `daylength_s`; `lamp_power`, `chamber_setpoint`) are carried in
  the sealed resolvers as two phase twins (`v@cold`, `v@warm`) and selected by a wrapper on every
  flow and aux process (`station::sowing`; type names, ids and priorities kept, so **no flow-set
  change**); the plain names are in no sealed resolver, and the wrapper refuses one beside its
  twins. The re-sow stays on the calendar here, so the clock keeps `mod season_days` and agrees
  with the calendar on every step: the golden gains exactly the one line
  `"station.sown_step": "0x1.c980000000000p+13"` (day 915), as predicted, and the manifest moves by
  that golden's hash alone. `simcore/` and `domains/` untouched. Stage 2 (re-sow on maturity) is
  the switch.
- **2026-10-06 — the biosphere's tissue shedding, delegated: 3 station goldens move
  (`sealed_station`, `greenhouse`, `lighting`); `harvest` byte-identical (its crop starts past
  anthesis).** No station code or param changed; the biosphere's `Senescence` / `NitrogenSenescence`
  shed no leaf or root from age before anthesis (`docs/biosphere-reference.md`, same date;
  `docs/plans/post-roadmap-leaf-shedding.md`). Sealed station: the cold weeks' seedling ends at
  0.1460 mol C (0.0771 before — the 3b loss was the flat shedding), grain at maturity 14.58 mol C
  (9.51), grain on the re-sow eve 41.02 (24.26); the energy books, the water ring and the
  development clock are byte-identical (19 of 36 stocks). Predicted to the printed digit by the lab
  twin (`station/examples/shedding_station`).
- **2026-10-06 — the cold period's dimmed lamp, slice 3b: two keys added to a station PARAMETER
  FILE; 1 golden moves (`sealed_station`).** `docs/plans/post-roadmap-room-temperature.md` §24g /
  §24l (predictions, committed before the switch) and §24m (graded). `cold_period.yaml` gains
  `cold_par` 100 µmol m⁻² s⁻¹ (CITED with a ⚠ LOCUS: Cha et al. 2022's own speed-vernalization
  light, not the standard protocol's, which names none) and `cold_photoperiod_hours` 8 (CITED,
  the standard protocol's short day). In the cold period the crop's PAR, net radiation and day
  length and the lamp's draw follow the program; the lamp keeps its efficacy when dimmed (40 W,
  a PWM-dimmed LED — DESIGN, the user's). **No flow-set change.** Built in two stages: the
  plumbing with the cold lamp set to the full one reproduced all 20 goldens byte for byte, then
  the switch. Graded against the independent re-simulation: battery **8.261426e9 J** (+2.32 GJ
  of lamp saved, less the heater), the end chamber 288.3622829 K, the node's daily 163.1421 /
  174.3095 / 171.44761 K, chill-days 55.926093595 and thermal time 5512.7606791 (the crop is
  never water-stressed); flowering / maturity 105 / 139 days after sowing (from 103 / 137). The
  seedling LOSES half its carbon in the dim cold weeks (⚠ since 2026-10-06 it no longer does —
  0.146 mol C; the loss was the flat tissue shedding, see the entry above) (0.160 → 0.077 mol C) and grain falls
  36 % (37.87 → 24.26 mol C); no rationing, no events. The other 19 goldens are byte-identical;
  the manifest moves by `cold_period.yaml`'s digest and the golden hash.
- **2026-10-05 — the plants read the chamber, with the cited cold period, slice 3a: a station
  PARAMETER FILE added and the plant chamber's setpoint made a program; 1 golden moves
  (`sealed_station`).** `docs/plans/post-roadmap-room-temperature.md` §24 (design and predictions,
  committed before code) and §24k (graded). `station/params/cold_period.yaml`: `cold_setpoint`
  277.15 K and `cold_days` 56 (CITED, class: Cha et al. 2022's standard winter-cereal protocol,
  6–10 weeks at 2–6 °C; the centre is the user's choice). The chamber is held at 4 °C for the
  first 56 days of each season, then at `chamber.yaml`'s 22 °C (`setpoint`, now the WARM one;
  the Rust field is `warm_setpoint`); the cooler and heater read the fast forcing
  `chamber_setpoint`. Every plant-step flow and aux process is wrapped to read `temp` from the
  chamber (`Q/C − 273.15`), type names kept, so **no flow-set change**; the plant resolver
  carries no temperature, and a plant-side one is refused. The lamp is unchanged (full, 16 h; the
  dimmed cold-phase lamp is 3b). Predicted with an independent re-simulation and graded: the
  battery 5.939020485e9 J (3e-11), the heater 1.6449e6 J at each warm-up, the node's daily
  172.1755 / 176.2189 / 173.74488 K, the end chamber 292.5662711 K (mid-cool-down), chill-days
  55.712671868 and thermal time 5573.6508642 (the crop is never water-stressed) — all held; the
  warm-phase vapour ceiling was mis-rounded in the prediction (3.359 vs 3.3603, the ceiling at
  the measured 22.0534 °C). The crop flowers ≈ 102 days after sowing instead of 219. The other 19
  goldens are byte-identical; the manifest moves by the new file's digest, `chamber.yaml`'s
  (header) and the golden hash. ⚠ **Unpredicted, measured after the commit (§24k):** the
  golden's soil carbon roughly halved (humus 24.6 → 12.9, litter 23.2 → 6.9 mol C) — the soil's
  own processes read no temperature, but its income fell from 120 to 63 mol C a season as the
  crop's season shortened; the crop's nitrogen, rooting depth and the water's split between
  soil, subsoil and condensate moved too (§24k says why).

- **2026-10-05 — the plant chamber's walls and heater, slice 2b-iii: two FLOWS and three
  PARAMETERS added; 1 golden moves (`sealed_station`), energy books only.** The third of three
  diffs (`docs/plans/post-roadmap-room-temperature.md` §23i–§23j), on the user's design: walls
  that lose heat to surroundings that may be hotter or cooler, a heater on the battery, the
  reference walls facing the outdoor weather (the user's choice, for calibration against field
  data); a held cabin, space and the station structure are lab options. `ChamberWall`
  (`UA·ΔT` against the weather file's daily temperature, into a two-signed
  `boundary.chamber_surroundings`), `ChamberHeater` (the cooler's mirror, below the setpoint).
  `chamber.yaml`: `wall_conductance` 0.30 W/m²·K (CITED, class: BVAD Table 4-50's freezer-cabinet
  1/R_S 0.28–0.32), `wall_area` 5.12 m² (DESIGN), `heater_capacity` 200 W (DESIGN). The
  outdoor temperature is read on the plants' day (`floor(n·bio_dt)`; the fast operator keeps
  the slow `n`), pinned at day boundaries. Predicted first (an independent re-simulation),
  graded: every non-energy stock, every aux value and `power.battery` byte-identical (the
  heater never fires in the reference — the lamp's 133 W exceeds the walls' largest 36.6 W);
  the walls carry 1.8417e9 J outdoors (5.4e-7 of the prediction); the node is no longer a
  fixed point (daily 172.17–174.89 K, ends 174.141 K); the chamber 295.1887–295.2034 K to 7
  figures. The heater has its own lab test that fires it (a dead lamp on the coldest days).
  Unforeseen, recorded: in the lab's lamp shedding the heater draws on the battery the
  shedding protects (3.9 MJ in its blackout run).
- **2026-10-05 — the plant chamber's heat store, slice 2b-ii: the lamp's LIGHT heats the
  chamber too; 1 golden moves (`sealed_station`): the node warms 7.54 K, `boundary.light_used`
  leaves the sealed build. No parameter, no flow type added.** The second of three diffs
  (`docs/plans/post-roadmap-room-temperature.md` §23g–§23h). The light leg used to leave the
  station at `boundary.light_used`; in a room, light absorbed by leaves and walls is heat (BVAD
  Table 4-88 books every chamber watt as heat to reject; the ≤ 0.72 % fixed as sugar is the
  recorded overcount). `Lamp` with both targets equal now emits one netted leg (simcore rejects
  two legs on one stock); the `lighting` build, with two targets, is byte-identical.
  `sealed_node_heat` counts the whole lamp draw, so the node starts and holds at **174.961 K**
  (451.64 W; was 167.42 K). Predictions committed first; graded 8 held, 1 half: every
  non-energy stock and aux value byte-identical, the chamber at 295.2033 K, `boundary.space`
  within 1.8e-11 of the prediction. The half: the dead-cooler lab test's fixed band, sized for
  2b-i's starting gap, was not re-derived and went red; replaced by a derived exponential bound.
  The lab's lamp shedding now detects "lit" from the lamp's own draw.
- **2026-10-05 — the plant chamber's heat store, slice 2b-i: a station PARAMETER FILE and a
  FLOW are added; 1 golden moves (`sealed_station`) by one added stock and nothing else.** The
  user's form B (*"a held room that can fail"*, 2026-10-01), the first of three diffs
  (`docs/plans/post-roadmap-room-temperature.md` §23). The grow lamp's waste heat, which went
  straight to `thermal.node`, now goes into `thermal.chamber` (`T = Q / C_ch`, from 0 K), and a
  new flow `station.chamber_cooling` (`ChamberCooling`) hands it to the node: first-order
  toward the setpoint, capped, and only into a colder node (the second law). New
  `station/params/chamber.yaml`: `setpoint` 295.15 K (BVAD Table 4-73, by analogy),
  `heat_capacity` 1.5e5 J/K (DESIGN; anchor: Table 4-88's 36.8 kg/m² of root-zone water at
  NIST's 4184 J/kg·K), `cooling_capacity` 200 W (DESIGN; Table 4-88 sizes thermal control to
  installed power), `response_time` 60 s (DESIGN, the user's "1-minute response"); the build
  refuses `dt > τ`. Advisor-reviewed before code; predictions committed first (§23e), graded
  in §23f, 8 of 9 held: **every pre-existing stock and aux value is byte-identical**, the node
  and `boundary.space` included (predicted only within 1e-9), and the chamber holds
  295.1741575 K to the bit. The station census gains its ninth param file. The plants do not
  read the chamber yet. Lab: a dead cooler heats it 34.79 K a day and the node relaxes to the
  closed-form 160.31 K, with the crop byte-identical (`tests/chamber_heat.rs`).
- **2026-10-05 — a lamp-lit crop's net radiation is the lamp's, not the weather file's
  outdoor value; 2 station goldens move (`sealed_station`, `lighting`), 2 `golden_sha256`
  rows follow. No parameter added, no flow added, no `simcore` byte changed.** The user's
  ruling (*"fix this, in its current state, it doesnt make sense"*) on a finding of the minute-
  step transpiration lab (`docs/plans/post-roadmap-room-temperature.md` §20): every lamp-lit
  build replaced the crop's PAR with the lamp's but left transpiration on outdoor net radiation,
  so a sealed crop lost more water on sunny days outside and a dark lamp left its water loss
  untouched. Now `(1 − 0.23) ×` the lamp's radiant PAR while lit (500 µmol m⁻² s⁻¹ → 84.25
  W m⁻²), 0 in the dark, on the lamp's own top-hat (`station::lighting::lamp_net_radiation`).
  The 0.23 is FAO-56's broadband grass albedo, the same constant the outdoor form uses, **the
  user's choice** over holding the fix for a PAR-specific canopy reflectance; it probably
  under-counts what leaves absorb of an all-PAR lamp by up to ≈ 20 %. Predictions committed
  first (§21), graded in §21a: **water stocks only** — condensate, root-zone and below-root
  soil water; every carbon, O₂, nitrogen, crew and energy byte and every aux value is unchanged
  (the crop is never water-stressed). Transpiration over the goldens' horizons: `sealed_station`
  **−20.9 %**, `lighting` **−7.5 %**. The lab's lamp shedding and `with_lighting_failure`
  darken it too. Sunlit builds (`greenhouse`, `harvest`, every biosphere scenario) are
  byte-identical.
- **2026-10-03 — the crop's gas exchange is stepped on the cabin's minute step; 3 station
  goldens move, 3 `golden_sha256` rows follow, and the station `flow_set` gains `Allocation`,
  `GrowthRespiration`, `MaintenanceRespiration`. No `simcore` byte changed; no biosphere formula
  changed.** The user's decision (*"calculate the plants' gas exchange minute by minute. This
  changes the frozen plant science — do it"*), after the lab separate-air build measured that the
  plant step starves a small chamber by construction (a 27.66-mol chamber's crop drew 0.174 of
  shared air's CO₂, equal at any fan rate). Advisor-reviewed twice before code; designed, built
  lab-only, redesigned and adopted in four commits with predictions committed first
  (`docs/plans/post-roadmap-room-temperature.md` §16–§17a). `greenhouse` plant carbon +0.29 %,
  `harvest` +0.07 %, `sealed_station` +0.75 % (grain +0.79 % over 4 years). **Every water stock
  and every pre-existing aux value is byte-identical** (transpiration does not read the canopy);
  the aux block gains `station.plant_window.par` / `.temp`. 0 rationing, 0 events; the tier
  contract tests pass. Found by a missed prediction: `build_harvest` discarded the greenhouse's
  fast registry and with it the moved carbon budget (its crop grew nothing, −17.6 %); fixed, and
  `every_crop_build_steps_the_carbon_budget_exactly_once` now holds every crop build to it. The
  perturbation suite's "regulator erasure" claim is restated: the pools return to the baseline
  once the crop's own flux offset (ΔS / k) is taken out, residual ≤ 1.8e-9 against the unchanged
  1e-6 bound (it was ΔC ≈ 0, which held only while the crop drew a 90-minute pulse).

- **2026-10-03 — the cabin's condenser holds BVAD's 40 % relative humidity; 6 station goldens
  move (`eclss.cabin_h2o` by +1.7863 kg, the crew's water books in the last few bits) and
  `eclss.yaml`'s hash and 6 `golden_sha256` rows follow. One new sibling param; no flow id,
  flow type, seam or claim changed.** `Condenser` drew `k_cond·cabin_h2o`, first-order on all
  of the cabin's vapour, so every cabin settled at `P/k` = 0.0675 kg, ≈ 1.5 % RH of a 9500-mol
  cabin at 22 °C: a value set by a solver-stability rate. It now draws
  `k_cond·max(0, cabin_h2o − humidity_setpoint)`, one-sided, with `humidity_setpoint` = 1.7863 kg
  derived from BVAD Rev 2 Table 4-1 (printed p. 63, page image) — 40 % nominal ("Typical ISS"),
  range 25–75 % — at 22 °C in 9500 mol, with the `o2_setpoint` TRIGGER (it encodes the cabin's
  air and a temperature the cabin does not hold). Every scenario's `cabin_h2o_0` moved 0 → the
  setpoint (the `cabin_o2_0` rule). Advisor-reviewed before code; predictions committed first
  (`docs/plans/post-roadmap-room-temperature.md` §15, graded in §15a). `cabin_gas`, `eclss`,
  `water_recovery`, `greenhouse`, `harvest` and `sealed_station` moved; the shift argument
  (`e = h2o − setpoint` obeys the old law) predicted the diff: `cabin_h2o` +1.7863 exactly, and
  condensate / `crew.water_store` / `recovered_water` / `brine` by ≤ 3.4e-14 relative. **Not one
  plant-side byte moved**, and `lighting`, `sealed_energy_drift_summary` and every biosphere
  golden are unchanged. 0 rationing, 0 events, no tier-2 band crossed.
  ⚠ **What this makes visible in SHARED air, recorded and not fixed (the user's call):** the
  shared room holds two vapour stocks, each held by its own condenser — the plants' at 75 % of
  the weather's saturation, the crew's at 40 % of 22 °C's — and their targets ADD. Counted
  together over one season (`examples/air_split.rs`, `W:\temp\claude\air_split_run4.txt`) the
  room is above saturation at the plants' temperature on **4 876 of 4 880** plant steps, up to
  **2.88×**; before this change, **8** steps, up to **1.22×**. Even at one common 22 °C the two
  targets sum to ≈ 116 %. No flow reads the sum (nothing reacts to it, and no display shows a
  room humidity), so no simulated value changes; it is a bookkeeping flaw of two stocks in one
  room, which the crew-only cabin, the standalone ECLSS cabin and the lab's separate air do not
  have.


- **2026-10-01 — the biosphere's crop transpires against the chamber's own air; 4 station
  goldens move (water stocks only) and 4 `golden_sha256` rows follow. No station flow, param,
  seam or claim changed.** Delegated: `docs/biosphere-reference.md`'s 2026-10-01 entry is the
  record. `greenhouse`, `harvest` and `lighting` (7 days) moved `condensate` +26 % to +32 % and
  `soil_water` −0.6 % to −3.5 %; `sealed_station` moved `condensate` −2.26 %, `soil_water`
  +1.22 %, `subsoil_water` −6.99 %. No carbon, O₂, N, crew or ECLSS value moved;
  `sealed_energy_drift_summary.json` unchanged. ⚠ In `sealed_station` the water-stress factor
  was not probed directly; that it stays 1 is inferred from no carbon value moving. The Godot
  palette's `greenhouse` and `sealed` sessions read the new form. Every station golden keeps 0
  rationing and 0 events; no tier-2 band was crossed.

- **2026-09-30 — option C reaches the station: the crop's growth reads the cabin CO₂ the plant
  step leaves. 4 station goldens (`greenhouse`, `harvest`, `lighting`, `sealed_station`) and
  their `golden_sha256` rows; no station flow, param, seam or `simcore` byte changed.** The
  biosphere's C entry of the same day (see its log). The station's cabin air *is* the biosphere
  carbon pool, so the plant step's draw is solved against the cabin air it leaves before the
  cabin's 90 minutes run. The 7-day seedling goldens move in the 4th significant figure
  (`lighting`, whose chamber holds 0.23 mol of CO₂, in the 3rd); `sealed_station`'s grain
  −0.88 % over 4 years. Every station golden keeps 0 rationing and 0 events; no tier-2 band
  was crossed.

- **2026-09-30 — the plant step moves to 1/16 day, and the master day may run in equal
  GROUPS. 4 station goldens move (`greenhouse`, `harvest`, `lighting`, `sealed_station`), their
  4 `golden_sha256` rows and `numerics_note` follow. No flow, param, seam, claim or `simcore`
  byte changed.** The biosphere's step unfreeze of the same day (see its log); plan and grading:
  `docs/plans/post-roadmap-step-sixteenth.md`. The sealed reference now interleaves each 1/16-day
  plant step with its 90 cabin minutes. **The even-split refusal logged just below became
  unrunnable**: the lamp scenarios step power hourly, and 24 does not divide by 16. On the
  user's call (*"Loosen the even-split rule"*, over a half-hour power step) `driver::day_groups`
  replaced `fast_steps_per_slow_step`: the day runs in `gcd(fast, slow)` equal groups of slow
  steps then fast ones, which is the adopted interleaving whenever the counts divide (every
  cabin scenario) and 8 × (2 plant steps + 3 power hours) for the lamps. Counts with no common
  factor are still refused, because their only equal grouping is the retired slow-first day.
  `lighting`'s battery is bit-identical (lamp and crop share no stock); its crop moved with the
  step. Every station golden keeps 0 rationing and 0 events; no tier-2 band was crossed.

- **2026-09-30 — the master day is INTERLEAVED: each plant quarter-day is followed by its 360
  cabin minutes, where it had been all four plant steps then all 1440 minutes. 3 station
  goldens move, 3 `golden_sha256` rows and `numerics_note` follow. No flow, param, seam, claim
  or `simcore` byte changed.** The user's call (*"adopt"*), on the lab measurement in
  `docs/log/intraday-gas-exchange.md`; plan, predictions (committed before any code) and
  grading: `docs/plans/post-roadmap-intraday-gas-exchange.md` §8.
  **Why.** Slow-first let all four plant steps draw on the cabin CO₂ as it stood at dawn, with
  the crew's exhalation for the day arriving only afterwards, so a crop sized to the crew was
  starved by the schedule: at one crew member's area it reached 57 % of the 1 m² crop's growth
  per m², and 98.8 % interleaved. Interleaved is also the order that matches the physics.
  **What moved.** `greenhouse` (16 stocks, largest +4.7e-4), `harvest` (grain store +37.5 %,
  humus +19.0 %, microbes +12.1 % — the harvest flow and the feces-to-litter seam are two more
  stocks the sides share besides the air), `sealed_station` (crop carbon +1.23 % over 4 years).
  All three keep 0 rationing and 0 events. `lighting` cannot move (lamp and crop share no
  stock) and did not.
  **The new refusal.** `driver::fast_steps_per_slow_step` refuses a cabin-step count that does
  not divide evenly by the plant-step count, in the per-day function, `run_master_day` and
  `SimSession::two_rate`. No shipped scenario trips it; authored scenarios cannot reach this
  driver.
  **The retired order is kept, lab-only** (`driver::DayOrder::SlowFirst`), so the record that
  retired it can be re-run; the plan's §7 said to delete it and §8 says why it was not.
  Tier-2 basis re-measured: greenhouse ±1-ULP sensitivity 2.76e-16 → 3.49e-16, band 1e-11.

- **2026-09-29 — the biosphere's condenser holds 75 % relative humidity; 4 station goldens
  move (water stocks only) and 4 `golden_sha256` rows follow. No station flow, param, seam or
  claim changed.**
  Delegated: `docs/biosphere-reference.md`'s 2026-09-29 entry is the record. `greenhouse`,
  `harvest`, `lighting` and `sealed_station` re-ran. In each, `biosphere.water_vapor` moved
  ×0.84372–0.84377, with `condensate` and `soil_water` taking up the difference (and in
  `sealed_station` 6e-4 kg of `subsoil_water`); no carbon, O₂ or N value moved.
  `sealed_energy_drift_summary.json` unchanged. ⚠ The 2026-09-23 entry's per-STOCK caveat
  stands: the crew's `eclss.cabin_h2o` still sits outside the biosphere's target.

- **2026-09-23 — the biosphere bounds chamber water vapour by saturation; 4 station
  goldens move (water stocks only) and 4 `golden_sha256` rows follow. No station flow,
  param, seam or claim changed.**
  Delegated: `docs/biosphere-reference.md`'s 2026-09-23 entry is the record. `greenhouse`,
  `harvest`, `lighting` and `sealed_station` re-ran; in each only `biosphere.water_vapor`,
  `condensate`, `soil_water` (and in `sealed_station` 1e-4 kg of `subsoil_water`) moved — no
  carbon, O₂ or N value, which the prediction had flagged as unmeasured for these four and
  which held. `sealed_energy_drift_summary.json` unchanged.

  ⚠ **The bound is per STOCK, not per room.** The crew's humidity is a separate
  `eclss.cabin_h2o` in the same cabin air, held by the ECLSS condenser at 3.75 mol at the end of
  every run against a 115–219 mol saturation cap — so the cabin's total water can exceed
  saturation by at most ~3 %. This entry claims the biosphere's vapour is bounded, not the
  cabin's humidity.
  ⚠ **Superseded 2026-10-03, and it was already wrong.** Measured over one season counting both
  stocks against saturation at the plants' temperature, the room exceeded saturation on 8 of
  4 880 plant steps, by up to 22 % (not ~3 %). Since the cabin's condenser holds 40 % RH (the
  entry above), it does so on 4 876 of 4 880, up to 2.88×.

- **2026-09-07 — the biosphere adopts the LIVE-O₂ FvCB form; 4 station goldens move by
  ≤ 0.058 % and 4 `golden_sha256` rows follow. No station flow, param, seam or claim
  changed.**
  The delegated half: this contract freezes the multi-domain assembly and delegates the
  biosphere's science to `docs/biosphere-reference.md`, whose 2026-09-07 entry is the record.
  `greenhouse`, `harvest`, `lighting` and `sealed_station` re-ran; `sealed_energy_drift_
  summary.json` did **not** change, because it folds thermal quantities the biosphere's carbon
  does not reach.

  ⚠ **The size of this is the previous slice's doing, and the contrast is the point.**
  Measured before the cabin oxygen setpoint was cited (10 → 1995 mol, 2026-09-06), the same
  flip moved `sealed_station`'s plant and soil carbon by **60–75 %** — the crop was reading
  1.05 mmol/mol of O₂ where a cabin holds ~210. Measured after: **+0.058 %**. The station's
  three sealed assemblies are now *controls* on the biosphere's form rather than its largest
  consumers, and taking the two changes in one diff would have made them indistinguishable.

  ⚠ **A KNOWN GAP this slice did not close: the station census still carries no CO₂ band.**
  Its two claims are the crew respiratory quotient and the thermal fixed point, so the
  biosphere's five pointwise compensation-point gates have no station-side counterpart, and
  the cabin's oxygen mole fraction — which the crop now reads — is asserted by nothing here.
  A gate for it was designed and **refused on measurement**: `O2Makeup` holds the pool at
  209.800 mmol/mol against any defensible band with ~50× of headroom, and everything that
  could move it already moves four goldens and a param hash, so the row would have been inert
  by construction. Recorded as a gap rather than filed as a green claim.

- **2026-08-18 — the MANIFEST WRITER moves to the reference (C7's station half; a
  PROSE-only diff — three `_authority`/`_comment` rows, no hash, set, claim or horizon).**
  Until this slice the file was *authored* by the reference key by key (slices 3, 7, C8,
  C4b) and **written** by `tests/test_station_freeze_manifest.py`, which shelled the
  reference's dump, spliced its keys into its own and serialized the result. That module
  is a **checker only** now, with no `__main__`. Regeneration is step 4 of the ceremony
  above: from `rust/`, `cargo run --example dump_station_inventory -- --write-manifest`.
  It reproduced this file **byte-identical on the first run**.

  ⚠ Moving the writer is **authority-neutral by construction**: `_authority` records who
  produced the *value*, not who ran the digest or wrote the file. The precedent is in the
  block itself — `scenarios/*/golden_sha256` has read `rust` since slice 4 while *Python*
  computed the digest.

  ⚠⚠ **The trap this slice sets is only PARTLY visible, which is worse than the
  biosphere's.** `numerics_note` is hand-maintained prose naming three integration steps,
  and the writer now lives in the crate that owns all three. Measured: splicing `bio_dt`
  renders `dt=0.25 day` against the written `dt=1/4 day` and the regeneration gate
  reddens; splicing `cabin_dt` or `power_dt` renders `60` and `3600`, **byte-identical**
  to what the sentence already says, because Rust prints `60.0_f64` as `60`. So two of
  three would auto-follow the code with the regeneration diff seeing nothing — and this
  contract has no structured step key to compare against, unlike the biosphere's
  `dt_days`. Run end to end: the spliced regeneration printed *unchanged*. The guard is
  `rust/crates/station/tests/manifest_writer.rs`, which reads the writer's own source and
  requires the emission site to be a quoted literal naming none of the three.

  ⚠ **Adding a structured `dt` key is refused for the third time**, on the same ground:
  it widens the frozen surface and is its own ceremony, not a rider on a re-anchoring.

  ⚠ **Deleting the writer opened a hole and closed one.** The scenario roster
  (`name -> label, golden`) lived in the checker *and was written from it*, so nothing
  held it; the two fields at risk are exactly those `_authority` marks `hand`, which no
  gate can re-derive. `test_the_frozen_roster_is_the_references` closes it. What the move
  *closed* is a hand edit to the committed manifest, invisible to every gate before now
  and caught by the byte comparison in `rust/crates/station/tests/manifest_writer.rs`
  (`tests/crossport/test_manifest_writer.py` until S2 moved it and S6 retired it).

  **Verification.** `cargo test` + `cargo clippy --all-targets -D warnings`; `ruff`,
  `pyright`, the Python suite and the crossport suite green. Controls: hand-edited
  manifest → red; drifted roster label → red alone; moved golden → red; an aux process
  wired into a canonical build → the regenerated manifest **gains the name** and the
  checker's aux gate reddens (the substitute for a rename control this empty axis cannot
  run); the `numerics_note` splice → manifest unchanged, source-text guard red.

- **2026-08-18 — the two SCIENCE CLAIMS re-anchor to the reference (slice C4b of the flip; a
  LOCUS-only unfreeze — no bound, quantity, source, hash, set or golden moved).** The
  biosphere's 13 gates moved in slice C4 and these two were split off, correctly: a gate
  lives with the runs it reads, and the BVAD respiratory-quotient prediction reads the
  coupled cabin while the thermal node's floor reads the 15-yr Power→Thermal decade —
  `station` types, in a crate that depends on `domains` rather than the reverse. They are
  declared in `rust/crates/station/src/science_gates.rs` now, a **second table** invoking the
  same `science_gates!` macro (exported for this) with its own `source_file`.

  **The diff was predicted before regenerating and came back as predicted:** two `locus`
  strings, plus the two `_authority` rows moving `python` → `rust` with their prose. Nothing
  else in the file changed.

  ⚠ **The prose this doc carried named referents the reference was missing — and the naming
  was already false.** `predicted_equilibrium_temperature`, the `year_summaries` /
  `same_phase_diffs` / `is_stationary` / `non_collapsing` folds and the 15-yr energy run all
  existed in `rust/crates/station` when the sentence was written. So C4b came in under its
  own estimate, and the estimate's expiry condition never fired because nothing re-reads a
  present-tense claim about the tree.

  ⚠⚠ **The first regeneration silently dropped ELEVEN keys, and the prediction is what
  caught it.** The reference's *dump* emits only scenarios that carry a claim — deliberately,
  because which scenarios get a key is this manifest's hand-authored roster and a program
  that invented keys would claim authority over a set it cannot see. The Python census it
  replaced filled every roster key with `[]`. Splicing the dump's shape straight through
  therefore deleted the eleven empty lists, which on this contract are *the frozen claim*:
  11 of 13 station scenarios carry no outside-sourced bound, and `[]` says "measured, none"
  where an absent key says nothing. `_filed_under_the_roster` fills the roster around the
  reference's claims and **raises** on a claim naming a scenario outside it.

  ⚠ **A control that only became necessary the day the data changed.** The checker reads the
  reference's dump through `subprocess.run(text=True)` with no `encoding=`, i.e. the Windows
  locale — the exact mechanism that froze cp1252 mojibake into the biosphere contract in
  slice C4 with every gate green. The pin was added to the crossport reader then and *not*
  here, correctly: nothing this dump emitted was above ASCII. C4b is the first slice to send
  an em dash through it (`self — the node must not collapse toward T_space`). Pinned.

  **Verification.** `cargo test` + `cargo clippy --all-targets -D warnings`; `ruff`,
  `pyright`, the Python suite and the crossport suite green. Controls: the assertion carrying
  a recorded literal deleted → the reference's bound-literal check red, the gate itself
  green; a `science_gate` marker re-added in `tests/` → the census-exhausted gate red.
  Measured: the node's annual peaks sit at 160.12 K against the frozen floor of 100.0 (1.6×
  clearance), and the RQ gate's own numbers are unchanged from the Python body it mirrors.

- **2026-08-17 — `param_files` RE-ANCHORED TO THE REFERENCE (slice C8 of the flip). Not one
  hash moved, and that is the finding, not a relief.** The eight digits are **author-neutral
  by construction** — both trees compute a newline-normalized sha-256 of the same file under
  the same rule — so *"`param_files` is now Rust's"* is the wrong summary and the diff was
  predicted value-free before the ceremony was run. What re-anchored is the **census** (the
  eight files the reference *loads*: `domains::params::param_files` for power × 2, thermal,
  eclss and crew, plus `station::params::param_files` for `water_recovery` / `lamp` /
  `harvest` — compile-time `include_str!` entries, not a glob over six Python package
  directories) and the **normalization** (`config::provenance`; a hand-rolled sha-256, because
  every engine crate is zero-dep by charter).

  ⚠ **No exclusion rule on this side, and the asymmetry with the biosphere's 15-of-20 is
  stated per side deliberately.** These six directories hold nothing but frozen files. A reader
  who generalises the harder rule here will look for exclusions that do not exist.

  ⚠ **Newly asserted, and nothing had checked it before: every basename is unique across the
  six directories.** This key is basename-**keyed**, so a name appearing in two of them would
  silently collapse two files into one entry — Python's `_param_paths()` *documents*
  uniqueness and its dict would quietly keep whichever directory it read last.

  Python's `_param_paths()` and `_normalized_sha256()` are **retained with their meaning
  inverted**, as conformance checks on the checker — the treatment slice 7 gave the flow set.
  Prerequisite: **slice C1**, which moved the YAML loaders into the reference.

- **2026-08-16 — the reference flip's slice 7: this manifest is now produced from the Rust
  tree, and NO frozen value moved.** `docs/plans/post-roadmap-reference-flip.md`. Authorized
  by the user (target state B: Rust canonical, Python the checker). The whole diff is the new
  `_authority` block and the `_comment`; the flow set is the same 16 names, `aux_set` the
  same `[]`, the horizons the same 15/4, every hash unmoved — measured by running the dump
  *before* regenerating, and predicted in writing first.

  **What actually changed is the producer, not the contents**, which is why the evidence is
  a *pair* of controls rather than a green suite: renaming a flow in **Rust** moves the
  manifest and reddens the Python gate; renaming the **Python** class leaves the manifest
  byte-identical and reddens the same gate. Either alone proves nothing. Also landed:
  `golden_sha256` is now compared against the files on disk (the desync hole slice 5
  measured), the two sealed horizons are checked for staleness against the reference tree,
  and every field of the file declares its own producer.

- **2026-08-14 — the biosphere's within-day light path (biosphere-delegated; 4 station
  goldens, no station code).** `docs/plans/post-roadmap-gross-net-gas-exchange.md`.
  Authorized by the user (*"the plants MUST emit oxygen at least minute by minute"*). The
  science is entirely biosphere-side — see its reference's forcing section — and this
  contract moves for two reasons only.

  **What changed here.** The two **lamp** seams (`station/lighting.py`, `station/sealed.py`)
  stop handing the crop a constant PAR paired with a photoperiod-length integration window,
  and hand it the lamp's within-day **top-hat** instead. The daily photon dose is the same
  number; what changes is that the lamp's dark hours are now hours the crop respires
  through. ⚠ The lamp's **ENERGY** half is deliberately unchanged: the Power domain is the
  *fast* operator and `substep` freezes `n`, so a within-day shape is not expressible there
  and the flow keeps drawing the daily average. ⚠ *Partly outdated 2026-09-30:* since the
  master day was interleaved, the fast operator sees `n` take four values a day, so a
  **quarter-day** step shape is now expressible there (a minute-resolution one still is not).
  Nothing uses that: the lamp's energy half still draws the daily average, and the `lighting`
  golden was measured byte-identical across the change. The two halves of one lamp differ on
  purpose, and that asymmetry is now stated in `lighting.py` rather than implied.

  **What moved.** `greenhouse`, `harvest`, `lighting`, `sealed_station` (+ its energy-drift
  summary) — every station golden that carries a plant. The eight plant-free goldens are
  byte-identical. No station flow, stock, seam or param changed; `station_flow_set` and
  `params` are unmoved in the manifest diff.

  ⚠ **A stale scope claim in `lighting.py` was measured false and rewritten**: *"the only
  runtime consumer of `daylength_s` is photosynthesis"* named one reader when there were
  three — the photoperiod-sensitive phenology path was added three phases after that
  sentence was written. Same shape as `o2-makeup-reversal-inside-the-freeze`: **a scope
  claim is dated to the roster that existed when it was written.**

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
  ⚠ *"Derived from `science_gate` markers"* stopped being true on **2026-08-18 (slice C4b)**:
  the two claims are declared in `rust/crates/station/src/science_gates.rs`, the markers are
  gone from `tests/`, and the pytest-marker census is now asserted **empty**. The 11 empty
  lists are unchanged and still the measured result.

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
