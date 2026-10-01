# Post-roadmap — a sealed chamber's crop transpires against the chamber's own air (the 2026-09-29 review's Step 3, slice 3b)

**Opened 2026-10-01** on the user's *"work on step 3"*, after 3a
(`docs/plans/post-roadmap-light-from-delivered-power.md`). Review plan:
`docs/plans/post-roadmap-review-2026-09-29.md`, Step 3, slice 3b.

**BUILT AND MEASURED 2026-10-01, NOT FROZEN** (§4). The chamber reading is in the code behind
`science::VpdRead`; the loader still sets `Weather`, so every committed golden still holds and the
suite is green. **Freezing it is the user's decision**: one loader line, the 9 goldens'
regeneration, the manifests, and one test's bound (§4.4). This is a
biosphere unfreeze (`docs/biosphere-reference.md`, "The unfreeze discipline"): the frozen
transpiration flow's dryness input changes in every sealed build.

## 1. The problem

Sealed transpiration (`Transpiration` with `saturation: Some`, `domains/src/biosphere/flows.rs`)
reads its vapour-pressure deficit — how dry the air is — from the **weather file**
(`weather::vapor_pressure_deficit(temp, vap_hpa)`, outdoor vapour pressure). The chamber's own air
has been held at BVAD's 75 % humidity since 2026-09-29 (`log/vapour-step-artefact.md`), so the
crop's water loss answers to air it is not in. Same shape as CO₂ and O₂ before they were coupled.

**Already priced** (2026-09-29, at the then ¼-day step, `log/vapour-step-artefact.md`): at 75 %
the chamber's own deficit summed over a run is **1.50× the weather's**, and the potential
transpiration it drives **1.21× (+21 %)**. Precedent for the form: BVAD Rev 2 §4.14, Eqn 4-23
(Monje 1998) takes the deficit from the chamber's relative humidity.

## 2. The form — no new number

The chamber's actual vapour pressure follows from its vapour store by the identity the model
already uses for every gas in a room (`p_i / P_std = n_i / n_ref`, `science::saturation_vapour_kg`):

    e_a = v · P_std / (n_ref · M_H2O)          VPD_chamber = max(0, e_s(T) − e_a)

`v` = the chamber's `water_vapor` at the start of the step (kg), `n_ref` = the room's reference
fill (`chamber_air_capacity_mol`), `e_s(T)` = FAO-56 saturation pressure at the step's
temperature (already cited), `P_std` = 101 325 Pa. The `T/T_ref` term dropped from every gas is
dropped here too (stated in `saturation_vapour_kg`). At the humidity target the deficit is
`(1 − 0.75) · e_s(T)`.

**Scope:** sealed builds only (`saturation: Some`). The open field keeps the weather's figure.
`net_radiation` (the energy term) stays the weather file's, in the lamp-lit builds as well —
found in 3a, recorded, not changed here (it is its own question: a lamp's radiation is not the
sun's).

**One helper, not a second copy:** `science::chamber_vapour_pressure_pa(v, n_ref)` beside
`saturation_vapour_kg`, so the two halves of the identity live together.

**Plumbing, by precedent** (`science::Co2Read`, Step 2): a field never loaded from the file,
set by the loader to the new reference value, which `domains::lab` can flip back — so the
retired reading stays runnable and the control "flipped back = today's goldens, bit for bit" is
a test, not a claim.

## 3. Predictions (written 2026-10-01, before the code)

1. **Which goldens move: exactly the 9 that carry a sealed chamber** — `sealed_chamber`,
   `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`,
   `greenhouse`, `harvest`, `lighting`, `sealed_station`. The open-field and plant-free goldens
   are byte-identical. `sealed_energy_drift_summary` is unchanged (latent heat is not modelled,
   so no energy stock reads transpiration). `drift_summary` — not predicted; read on the day.
2. **Potential transpiration rises about +21 %** summed over a run in the biosphere chambers, as
   priced at ¼ day; the 1/16 step should not move the ratio much, because the deficit at the
   start of a step is set by the humidity target, not the step. Measured, not assumed.
3. **Water stress: predicted NONE.** Irrigation is demand-driven (refills the root zone up to its
   capacity, at most 8 mm/day in the default scenarios), so the stress factor drops below 1 only
   if a day's transpiration outruns 8 mm/day long enough to drain the root zone past its
   threshold, or if a sealed build's water source runs short. If the prediction holds, the stress
   factor stays **exactly 1** and **no carbon, nitrogen, O₂ or energy value moves in any
   golden** — only water stocks (`soil_water`, `subsoil_water`, `water_vapor`, `condensate` and
   the water stores that feed irrigation). **Checked before code, by reading:** the stress factor
   is a hard threshold, exactly 1.0 at `FTSW ≥ wssg` (`science::water_stress_factor`); the drought
   factor is `(1 − wsfg)·wssd + 1`, exactly 1 at `wsfg = 1`; root extension reads the same stress
   factor plus a "subsoil empty" stop. So carbon can move only if the stress factor falls below 1
   at some step or the subsoil reaches zero where it did not before. **Both are measured
   directly** (lowest stress factor, lowest drought factor, steps below 1, subsoil minimum, per
   scenario) — not inferred from carbon moving.
4. **`water_vapor` barely moves**: on steps the crop can fill, the air ends at the target either
   way; the extra transpiration goes to `condensate`.
5. **Manifests:** the 9 `golden_sha256` rows move (biosphere 5, station 4), plus
   `drift_summary`'s (it folds the sealed 15-year perennial and consumer runs, so any change to
   those trajectories moves its drift figures). The flow set and param hashes do not (no file
   text changes; the new field is never loaded). `rust/data/tiers.json` is untouched unless the
   tier checks go red (a crossed band is re-measured under the native-port contract, not
   re-tuned) — predicted not to.
6. **The condenser's shortcut becomes load-bearing.** The condenser keeps a first-order draw
   *below* the 75 % setting (`science::condensed_vapour_kg`), recorded as harmless because "it
   only matters on steps the plants cannot fill" (`log/vapour-step-artefact.md`). Reading the
   chamber's air, the crop now transpires harder on exactly those steps. Measured: how many steps
   start below the target, the lowest relative humidity at a step start, and the share of the
   extra transpiration that falls on those steps. Predicted: few such steps (night and seedling
   steps), lowest RH well under 75 %, a small share of the extra.
7. **The station's crew water loop does not move.** The station wires neither the chamber's
   `water_vapor` nor its `condensate` into the cabin (the crew condenser works on `cabin_h2o`),
   so crew and ECLSS stores are bit-identical in the 4 station goldens.
8. **Tests:** flow tests that build a *sealed* `Transpiration` and pin its flux from a `vpd`
   forcing go red (the sealed flow no longer reads that forcing) and are re-posed by hand against
   the chamber's air; the open-field transpiration tests and the per-step humidity bounds in
   `tests/atmosphere.rs` stay green. New: the identity helper's tests (at the target the deficit
   is `0.25·e_s(T)`; above saturation it is 0), and the control — the weather reading flipped
   back reproduces today's goldens bit for bit.

## 4. Results — measured 2026-10-01, nothing frozen

**Built:** `science::chamber_vapour_pressure_pa` and `science::chamber_vpd_pa` (the identity's two
directions, one place), `science::VpdRead { Weather, Chamber }` on `WaterCycleParams` (never
loaded; the loader sets `Weather` until the freeze), sealed `Transpiration` reading the chamber
under `Chamber`, `lab::biosphere_with_vpd_read`, the measurement example `chamber_dryness`.
Tests: the identity inverts saturation at three temperatures and three room sizes; the sealed
flow at the target, at and above saturation matches Penman–Monteith by hand at the chamber's
deficit. The two older split tests are pinned to `Weather` with the reason written beside them.

### 4.1 The control — HELD

Loader set to `Weather`, `regen_goldens` in report mode: **20 of 20 goldens identical.**

### 4.2 The goldens under `Chamber` — regenerated into `W:\temp\claude\cd\golden`, never the tree

| golden | `condensate` | `soil_water` | `subsoil_water` |
|---|---|---|---|
| `sealed_chamber` | −2.24 % | +0.12 % | — |
| `perennial_chamber`, `perennial_long_horizon` | −2.24 % | +0.90 % | −4.93 % |
| `consumer_chamber`, `consumer_long_horizon` | −2.24 % | +0.90 % | −4.97 % |
| `sealed_station` | −2.26 % | +1.22 % | −6.99 % |
| `greenhouse` (7 days) | +31.98 % | −3.48 % | — |
| `harvest` (7 days) | +31.98 % | −0.63 % | — |
| `lighting` (7 days) | +26.43 % | −2.81 % | — |

**Nothing else moved in any golden**: no carbon, nitrogen, O₂, energy, crew or ECLSS value, and
not `water_vapor` (the air ends a filled step at the target either way). The end-of-run water
stores are snapshots of a cycling ring, so their signs are not the transpiration's: the 7-day
seedling runs end with more condensed, the year-long runs at a different point of the cycle.

### 4.3 The measurement (`cargo run --release -q -p domains --example chamber_dryness`)

| chamber | potential transpiration, chamber ÷ weather | lowest stress factor (both) | steps starting below 75 % | lowest RH after day 1 | extra transpiration on those steps |
|---|---|---|---|---|---|
| `sealed_chamber` (3 yr) | **1.2055** | **1.000000**, 0 steps below 1 | 466 of 14 641 | 0.538 | 5.6 % |
| `perennial_chamber` (5 yr) | **1.2055** | **1.000000**, 0 | 776 of 24 401 | 0.538 | 5.6 % |
| `consumer_chamber` (5 yr) | **1.2055** | **1.000000**, 0 | 777 of 24 401 | 0.538 | 5.6 % |

The lowest subsoil water is 25.966 kg under both readings (never near empty). The very first step
starts at RH 0: the chambers are sown with empty air (`water_vapor0 = 0`).

### 4.4 Predictions, graded

| # | prediction | measured | verdict |
|---|---|---|---|
| 1 | exactly the 9 sealed goldens move | the 9, and only them | **held** |
| 1b | `drift_summary` moves | **identical** | **missed**, and explained: it folds **carbon only** (each year's peak leaf carbon in the perennial and consumer chambers, and the consumer's year-end carbon), not the per-quantity mass drift the prediction assumed. No carbon moved, so it could not — a 15-year confirmation of row 3 |
| 2 | potential transpiration about +21 % | +20.55 % in all three chambers | **held** |
| 3 | no water stress; no carbon, N, O₂ or energy value moves | **measured directly in the 3 biosphere chambers**: stress factor exactly 1 at every step. **In the 4 station runs it is inferred**, from no carbon value moving — `sealed_station` is the run whose subsoil water moved most (−7 %), and its stress factor was not probed | **held** (station: inferred) |
| 4 | `water_vapor` barely moves | unchanged at the end of every run | **held** |
| 5 | 9 `golden_sha256` rows move; tiers untouched | not run yet (nothing written) | open |
| 6 | the condenser's below-target draw: few steps, small share | 3.2 % of steps, 5.6 % of the extra, lowest RH 0.54 after day 1 | **held** |
| 7 | crew and ECLSS stores unmoved | unmoved | **held** |
| 8 | the sealed flow tests pinned to the forcing need re-posing | the two were re-posed before the run (pinned to `Weather`) | held |
| 8b | (checked after the advisor asked) a test or perturbation made hollow | no test, perturbation, precision probe or science gate names the dryness input (`VPD_VAR`, `"vpd"`, `vap_hpa`), so none is silently emptied | none found |
| — | **not predicted** | `the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years` goes red under `Chamber` | **missed** |

**The miss that needs a decision.** That test checks the perennial chamber's yearly re-sow: one
transient year, then the same subsoil water every year (to 1e-12), the transient pointing
upward, and the transient **smaller than 1e-3**. Under `Chamber` everything holds except the
last: the transient is **1.203e-3** (it was 3.39e-4 under `Weather`), because the extra
transpiration leaves the first year's root zone further from its steady state (160.67 kg against a
settled 162.41, where `Weather` had 160.49 against 160.98). The cycle still settles: 167.71978719215,
…213, …213. The 1e-3 was set at about 3× the old transient; nothing in its doc ties it to a
physical limit. Raising it is a bound moved to fit a change, which this repository treats as the
user's call, never the agent's.

### 4.5 What freezing it would take

1. The loader line `vpd_read: VpdRead::Weather` → `Chamber`.
2. `regen_goldens -- --write`: the 9 goldens above.
3. The biosphere and station manifests' 9 `golden_sha256` rows (`manifest_writer`), and the
   reference docs' amendment blocks.
4. The re-sow test's transient bound — the user's decision (§4.4).
5. Run the tier checks after the write (prediction 5).

### 4.6 The freeze — taken by the user 2026-10-01; its predictions, written before `--write`

The user's *"freeze it"*, and for the re-sow bound *"Raise to 0.004"* (about 3× the measured
transient, the rule that set 1e-3). Predictions for the write, corrected by §4.4's 1b:

1. `regen_goldens` (report mode, loader at `Chamber`) reports **exactly the 9** sealed goldens
   changed; the other 11 identical.
2. Each written golden is **byte-identical** to its copy measured in `W:\temp\claude\cd\golden`
   (§4.2): same build, deterministic, so the frozen bytes are the bytes §4.2 describes.
3. Manifests: **exactly 9 `golden_sha256` rows** move — biosphere 5 (`sealed_chamber`,
   `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`),
   station 4 (`greenhouse`, `harvest`, `lighting`, `sealed_station`). **Not** `drift_summary`
   (§4.4 1b: it folds carbon only) and not `sealed_energy_drift_summary`. No `param_files`,
   `flow_set` or `aux_set` entry; no authoring manifest byte.
4. `rust/data/tiers.json` untouched; no tier-2 band crossed. `git diff rust/crates/simcore/` empty.
5. **Reach:** the authoring platform cannot build a `Transpiration` (no authored scenario or
   flow type names it, grepped), so authored habitats do not move. The Godot palette's
   `greenhouse` and `sealed` sessions are built by the reference loader and **do** read the
   chamber's air from the freeze on.
