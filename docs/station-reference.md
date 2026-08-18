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
the derived flow set + aux set, the eight param files (+ provenance hashes), each scenario
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
2. **Make the change** boundary-side. `git diff src/simcore/` **must stay empty** and
   `src/domains/` changes are domain-side data/citation edits only (a sibling param is a
   Phase-5 domain param, not a `simcore/` change).
3. **Regenerate the affected goldens**, each via its own explicit `__main__` action, and
   **review the byte diff** — a change there means the trajectory moved, which is the point.
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
  and caught by `tests/crossport/test_manifest_writer.py`'s byte comparison.

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
  and the flow keeps drawing the daily average. The two halves of one lamp differ on
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
