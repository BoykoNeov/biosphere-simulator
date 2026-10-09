# Post-roadmap: the plant chamber's dehumidifier stops drying the air below its setting

**Opened 2026-10-09 on the user's call** (*"Fix the dehumidifier. Then step 1"*), after the soil-evaporation
supply decision (`post-roadmap-soil-evaporation.md` §14) turned up the lead. A **biosphere unfreeze**:
`Condensation` and `water_cycle.yaml` are frozen science, and the station's sealed goldens delegate to them.

## 1. What is wrong

`science::condensed_vapour_kg` removes `max(0, v − target) + rate·dt·min(v, target)` each step: the whole
excess above the humidity setting, **plus** a first-order draw (`water.condensation_rate` 0.5/day) on the
vapour *below* it. A real dehumidifier switches off below its setting. The file says so itself
(`water_cycle.yaml`: *"It still runs below the setting, which a real dehumidifier would not; recorded, not
changed"*), and the cabin's condenser was fixed for the same fault on 2026-10-03
(`post-roadmap-room-temperature.md` §15). In a chamber whose crop no longer supplies vapour, this draw
empties the air: the lab's never-re-sown sealed chamber falls to ~4 % of its target while its soil is
still ~63 % full (`post-roadmap-soil-evaporation.md` §12c).

## 2. Advisor review (2026-10-09), summarized

Use `max(0, v − target)`: drop the below-target term and nothing else. Do **not** copy the cabin's
first-order draw on the excess: here the excess is already removed whole, and slowing it would be a
second, unasked change that can leave the air above saturation after a cooling step. Then
`condensation_rate` has no job. Remove it everywhere (yaml, the `biosphere_params.txt` dump, the
loader and its tests, the fields on `Condensation` / `VapourSaturation`, the lab split and the
examples), and fix every piece of prose that names it or the flaw. Four checks before predicting
anything (§3). The precedent order: plan, predictions committed, code, regenerate, grade, dated entries
in both reference docs. After the fix, re-run the lab `measurement()`. With nothing drawing below the
setting, the dead chamber's air can only gain, so the success test S4 may stop telling the two soil
supplies apart; a pass is not evidence for the root-zone choice. Then step 1 (B6): rebase
`wip/three-forms-price` onto the fix, re-run its control, trace there; §13's sealed rows go stale.

## 3. The four checks, done

1. **Nothing reads the chamber's water as heat.** No latent-heat term in `station/src`. `LATENT_HEAT_VAPORIZATION`
   appears only inside the biosphere's evaporation formulas, which turn energy into water, not water into a
   heat-store leg. So a water change cannot reach carbon through the chamber's temperature.
2. **The manifests record param files by hash, not by name.** `docs/biosphere-reference.manifest.json`'s
   `param_files` carries `water_cycle.yaml` as one digest; no key names `condensation_rate`. Removing it moves
   one hash, no name row.
3. **Every sealed chamber starts with empty air** (`water_vapor0 = 0.0`, `system.rs`). At `v = 0` both laws
   draw nothing, so the opening steps agree.
4. **Water stress in the frozen sealed runs: not checked by reading.** Transpiration reads the chamber's air
   (Step 3b, frozen 2026-10-01), so a changed vapour changes transpiration and the soil. Whether the crop is ever
   below its stress threshold there decides whether carbon can move. Left to the run (prediction D2 says how).

## 4. The change

* `condensed_vapour_kg(v, target) = max(0, v − target)`. Its second reader, sealed transpiration's headroom
  `target − v + condensed`, becomes `max(0, target − v)` above and below the setting alike: the air fills to
  the setting, and only the excess condenses.
* `water.condensation_rate` deleted: from `water_cycle.yaml`, `biosphere_params.txt`, the loader, the
  `Condensation` / `VapourSaturation` fields and every builder. An uncited DESIGN number leaves the model.
  `recycling_rate`'s source string loses its "matched to `condensation_rate`" clause.
* The warnings that record the flaw (`water_cycle.yaml`, `science.rs`, `flows.rs`, `eclss.rs:125`) are
  restated as history.

## 5. Predictions (committed before code)

The algebra: at `v = target` the old law moved `rate·dt·target` into the condensate and then refilled the
air from the crop's flux `F`. The condensate's inflow was `F` either way, and the air ended at the target
either way. **So wherever `F ≥ rate·dt·target` the two laws agree up to rounding.** They differ where the crop
puts out less than that (night, a dead or absent crop), where the old law let the air sag below its setting.

| | prediction |
|---|---|
| D1 | **9 goldens move:** the five biosphere chambers (`sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`) and four station ones (`greenhouse`, `harvest`, `lighting`, `sealed_station`). **The other 12 are byte-identical**, including `cabin_gas` and `eclss` (the cabin's own condenser), `season_euler` (open field) and both drift summaries. |
| D2 | **Water only:** no carbon, nitrogen, oxygen, energy, temperature, root-depth or development value moves. If one does, the crop reached water stress through the chamber's dryness (check 4), and that is a finding, not noise. |
| D3 | **Direction:** the air no longer sags at night, so the air is drier less often, the crop transpires a little less, the soil ends **wetter or equal** and the condensate **lower or equal** in each moved golden. Small: under 1 % on soil water. (A guess at the size, not derived.) |
| D4 | **Manifests:** `water_cycle.yaml`'s digest plus 5 `golden_sha256` rows (biosphere) and 4 (station). No flow id, flow type, aux key or seam changes. |
| D5 | **Reds:** the 9 goldens and their cross-port bands, the two `manifest_writer`s, the condenser's own unit tests and the loader tests that name the rate. Each is a restatement against the new law. A red anywhere else is a finding. |
| D6 | **Lab (`soil_evaporation::measurement`, re-run on main after the fix):** the never-re-sown sealed chamber's dead-phase air **rises under both soil supplies**. Under the top-layer cap it stops being refilled once the top empties, so it should track the coldest setting seen since (a cooler day lowers the setting and condenses the excess; a warmer one does not refill it). It should end **lower than under the root-zone supply**, so S4 may still tell them apart, but neither drains toward zero. |

Logs: `W:\temp\claude\dehumidifier\`.
