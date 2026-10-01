# Post-roadmap — a sealed chamber's crop transpires against the chamber's own air (the 2026-09-29 review's Step 3, slice 3b)

**Opened 2026-10-01** on the user's *"work on step 3"*, after 3a
(`docs/plans/post-roadmap-light-from-delivered-power.md`). Review plan:
`docs/plans/post-roadmap-review-2026-09-29.md`, Step 3, slice 3b.

**Predictions are written here before any code.** The build is then measured into a temp copy
of the goldens; **nothing frozen is written until the user has seen the measurement.** This is a
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

## 4. Results

None yet.
