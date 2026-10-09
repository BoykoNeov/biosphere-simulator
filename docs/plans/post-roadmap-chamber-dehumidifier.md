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

## 6. BUILT and adopted — the predictions graded (2026-10-09)

**What landed.** `science::condensed_vapour_kg(v, target) = max(0, v − target)`; `water.condensation_rate`
deleted from `water_cycle.yaml`, `biosphere_params.txt`, the loader, `Condensation`, `VapourSaturation` and both
builders (no other crate built either struct with it). The loader refuses the key if it comes back. Eight
goldens regenerated (`regen_goldens --write`), both manifests rewritten. Logs: `W:\temp\claude\dehumidifier\`
(`regen_report.log`, `golden_diff.txt` with the old goldens beside it, `suite.log`, `measure.log`).

| | predicted | measured | grade |
|---|---|---|---|
| D1 | 9 goldens move, 12 identical | **8 move**: the five biosphere chambers, `greenhouse`, `harvest`, `sealed_station`. **`lighting` is byte-identical.** Its lamp is off 8 h a day (16 h photoperiod), so a dark period exists; why the old draw left no trace there was **not traced**. The other 12 identical as predicted (`cabin_gas`, `eclss`, `season_euler`, both drift summaries among them). | MISSED by one |
| D2 | water only | water only: `condensate`, `soil_water`, `subsoil_water`. No carbon, N, O₂, energy, temperature, root-depth or development byte moved. `water_vapor` ends every run exactly where it did (at the target). | HELD |
| D3 | soil wetter or equal, condensate lower or equal; under 1 % | **Size HELD:** the five biosphere chambers move only at rounding (≤ 5e-14 relative). `greenhouse` / `harvest` (7 days): condensate **−0.25 %**, soil **+0.04 % / +0.007 %**. `sealed_station`: soil −4e-10, below-root +2.8e-9, condensate −1e-13. **Direction:** held where anything moved materially. `sealed_station`'s soil is the wrong sign, but at 4e-10, which is noise-scale. | HELD in size; direction where material |
| D4 | `water_cycle.yaml`'s digest + 5 + 4 `golden_sha256` rows | digest + **5** (biosphere) + **3** (station; `lighting` did not move). Nothing else. | HELD (with D1's correction) |
| D5 | reds: goldens, bands, the two manifest writers, the condenser's tests, the loader tests | goldens (both crates), the cheap station band (`greenhouse` 2.5e-3 against a 1e-11 band, which clears on the rewrite, measured), the domains manifest writer. The condenser and loader tests were restated in the same change, so only two lib tests fell: the saturation test (my own new assertion missed a rounding tolerance) and **one unpredicted one**, `the_three_cycle_flows_carry_only_water_in_ring_order`, which evaluated all three flows at one state below the target, where the condenser now draws nothing. Restated: the condenser is evaluated at a second state above the target. Also red: the plan-index gate (this file, procedural). The memory-index gate was **already red and unrelated**: the 2026-10-08 compression put several memory links on one index line, and the gate reads one link per line. | HELD + 1 unpredicted restatement |
| D6 | the dead chamber's air rises under both supplies, the cap below the root zone, neither to zero | Never-re-sown sealed chamber, events, dead-crop vapour against frozen: **cap 0.176 → 0.704** (end vapour 2e-6 → **0.073 kg**); **root zone 0.605 → 0.981** (end vapour 0.125 → **0.2505 kg**, the target). S-L alone (no soil evaporation): 0.719. Every other row of the measurement unchanged. Carbon equal to frozen in every row. | HELD |

**What it says.** The old draw was nearly invisible wherever a living crop fed the air: the biosphere goldens
moved only in rounding, and that is the algebra of §5. It was decisive where nothing fed the air. In the
lab's dead chamber the dehumidifier caused most of the missing air: about two-thirds of the gap under the old cap (0.176 → 0.704), and nearly all of it (0.605 → 0.981) under the root-zone supply. **Under the chosen root-zone supply the
success test S4 now passes (0.981).** Under the old top-layer cap it still fails (0.704). So the test still
tells the two supplies apart, and the cap's failure is now the soil's own.

**Restated tests (each an unfreeze, not a weakening):** `the_two_cycle_flows_are_first_order_in_their_own_donor_pool`
→ `the_two_cycle_flows_read_their_own_donor_pool` (the condenser's first-order claim retired and replaced by its
opposite: nothing below the setting, exactly the excess above it; the claim census's seven rows renamed with
it). The saturation and filling-step tests now pin the new headroom, `max(0, target − v)`. The loader tests
moved from the deleted rate to `recycling_rate`, and one asserts the deleted key is refused. The ring-order test
was restated as above.

**Owed:** §13 of `post-roadmap-soil-evaporation.md` is stale for the sealed rows; step 1 (B6) runs on the
rebased branch.

**Corrections after the build (advisor, 2026-10-09):**
* §2's warning that S4 "may stop telling the two soil supplies apart" was wrong, and D6's reasoning was
  right. Under the cap, the dead air falls only when a cooler step lowers the setting and is never
  refilled, so it ends at 0.704 and still fails.
* The claim census still mapped `test_condensation_flux_is_first_order_in_vapor` as `ported` to a test
  that now asserts the opposite. Moved to `retired-subject` with a dated header note. The six recycling
  rows beside it stay `ported`. The census pins no per-disposition count except `open`, so no literal moved.
* Prose outside the crates was grepped (`docs/` live sections, `godot/`, `scenarios/`, the param
  conventions): nothing describes the old draw except dated entries, which stay as written. One code
  comment in `system.rs` (the ring's positivity) was updated.
