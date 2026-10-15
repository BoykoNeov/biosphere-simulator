# Post-roadmap: a cited air resistance for every sealed chamber — 7.5 s/m

**Opened 2026-10-15 on the user's call** (*"1"*, from a list of next items whose first was "the crop drinks
too little water: we now have a source for the air resistance"). The price is
`post-roadmap-chamber-air-resistance.md` (closed 2026-10-14 with 50 kept, because no value below 50 had a
source). The source arrived the same day, after the close: `post-roadmap-real-world-checks.md` §13e–§13j.

**This is a biosphere + station unfreeze and a science item.** The open field keeps 50.

## 1. The user's decisions (2026-10-15)

1. **Value: 7.5 s m⁻¹**, the cited number (over a judgement value such as 15, which would be WHAT-IF).
2. **Scope: every sealed chamber** (the price's Line S, 9 goldens), over lamp-lit only (Line L, 2 goldens).

The caveat, put to the user in the question and repeated by a side note they forwarded after answering:
the source's fans blew 1.7 m s⁻¹ over the crop, NASA's BPC 0.2–1.2, so in a slower-aired room 7.5 is a
**lower bound** on the resistance, not an estimate — the best-ventilated end of the range. The station's
own air speed is not known. The user's choice stands with that on the table.

## 2. The source and the number

Monje, O. & Bugbee, B. (2019), *Radiometric Method for Determining Canopy Stomatal Conductance in
Controlled Environments*, *Agronomy* 9(3):114, doi 10.3390/agronomy9030114 (open access, CC BY 4.0; on the
git-ignored shelf as `sources/monje_bugbee_2019_agronomy9_114.pdf`). p. 19, Discussion: *"The gA for the
wheat (5.5 mol m⁻² s⁻¹ or 7.5 s m⁻¹) … were within the range of typical aerodynamic conductances of field
crops (… 4–13 s m⁻¹)"*. Locus: two ~1 m² EGC-13 chambers at Utah State, HPS-lit, wheat at LAI > 15, fan
1.7 m s⁻¹ above the canopy (p. 5).

**The conversion, checked against the model's own air** (advisor, 2026-10-15). Penman–Monteith in
`science.rs` uses `AIR_DENSITY` 1.205 kg m⁻³ — sea-level air at 20 °C. The same molar conductance at
101.325 kPa: `P / (R T) / 5.5` = **7.56 s m⁻¹ at 20 °C, 7.53 at 21 °C**. So the paper's own 7.5 is the
value at the model's air; §13e's 6.4 is the same conductance at the paper's 86 kPa altitude, which no
model room has. Script: `W:\temp\claude\ra-adopt\pm75.py`.

## 3. Advisor review (2026-10-15), summarized

Ask the value and the scope first (done, §1); correct the "four goldens" I had told the user (the price
says 2 or 9). Settle 7.5 vs 6.4 on the model's air density (done, §2). Don't cherry-pick the pricing
branch — its environment hook was a pricing device. Write predictions before the run and don't carry them
over: 7.5 is below the lowest priced value (9), so re-check that watering keeps up, re-check "water only"
(a carbon move would be the headline), and expect the sealed station's throughput and condensate drain to
exceed the price's 9 cell. Ceremony: plan + log index row together; cite the paper as the value's source, with its locus
caveat (another chamber, faster air); regenerate goldens and manifests; if the re-sow cycle test goes
red again, measure and restate, never just widen the 1e-12; the canopy-resistance instrument reads the
scenario's value; record whether the TM scorecard opts in; the energy side is unmeasured, not zero; full
suite `--no-fail-fast`, then `--ignored`, below-normal priority.

## 4. The shape

* **A second param**, `chamber_aerodynamic_resistance` in `transpiration.yaml` (7.5 s/m, cited), beside
  the open field's `aerodynamic_resistance` (50, still `TODO(cite)`). Parameters are data; the value has a
  source, so it lives in the file that carries sources.
* **The scope is the scenario's existing `sealed` flag.** Every sealed build is a chamber; the open field
  is not. No new scenario field, so no toggle-roster line (the price's bool-shaped red does not arise).
  An authored sealed scenario gets the chamber value too — a sealed room is a room.
* **One selector**, `TranspirationParams::aerodynamic_resistance_for(sealed)`, read by
  `transpiration_flow` and by every by-hand instrument that rebuilds the flow (the canopy-resistance
  instrument, the `chamber_dryness` example), so no copy can read 50 for a sealed room.
* **Two commits** (the house rule: inert plumbing first). (A) the param, its loader line, its positivity
  guard and the selector, read by nothing — 20 of 20 goldens identical, the biosphere manifest moves only
  `transpiration.yaml`'s digest. (B) `transpiration_flow` reads the selector — the 9 goldens move.
* **The TM 102788 scorecard stays out** (an open build with the lamp's forcings swapped in, so the
  scenario-scoped value does not reach it): it pins no ratio, and its §13 tests already print the trial's
  water row at 7.5 (1.03 of the level). Opting it in would need a scope device on an open build. Recorded,
  not built; the user may ask for it.

## 5. The arithmetic at 7.5 (the price's §4 instrument, extended)

Penman–Monteith at full cover, `rs` 50:

| `ra` | lamp (TM point) | sunlit 10 °C / Rn 40 | 17 °C / Rn 100 | 22 °C / Rn 130 |
|---|---|---|---|---|
| 50 | 2.58 mm/d | 1.76 | 3.31 | 4.43 |
| 9 | 4.12 (×1.60) | 3.04 (×1.73) | 4.89 (×1.48) | 6.50 (×1.47) |
| 7.5 | 4.27 (×1.66) | 3.15 (×1.79) | 5.05 (×1.53) | 6.72 (×1.52) |

7.5 is ~4 % beyond 9 in demand. The highest daily demand, 6.72 mm/d, is under the watering's 8 mm/day
capacity (and the sunlit chambers never pass LAI ~1, where it is lower still — the price's §4 correction).

## 6. Predictions (written before any code)

| | prediction | confidence |
|---|---|---|
| A1 | commit (A) is inert: `regen_goldens` 20 of 20 identical; the biosphere manifest's only change is `transpiration.yaml`'s digest; station and authoring manifests unchanged | high |
| B1 | commit (B): `regen_goldens` rewrites exactly the price's nine — `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon` (biosphere) and `greenhouse`, `harvest`, `lighting`, `sealed_station` (station); `season_euler`, `drift_summary`, `sealed_energy_drift_summary` and the eight crop-free goldens identical | high |
| B2 | **water only**: no carbon, nitrogen, oxygen, energy, `thermal_time`, `rooted_depth`, `vernalization_days` or `soil_shade_lai` leaf moves in any of the nine | medium-high (held bit for bit at 9; 7.5 is 4 % more demand, still under capacity) |
| B3 | the three one-week runs move a little further than at 9 (`lighting` soil ~−0.18 %, condensate ~+0.42 %); the multi-year end states are snapshots and may swing either way — not predicted in sign | medium |
| B4 | root-zone outflow over the run (a probe, not a golden): `sealed_station` above the price's +77 % at 9 (~+80–90 %); the sunlit chambers +4 to +25 % | medium |
| B5 | reds before the goldens are regenerated: `golden_regression` (domains, station), the `tier_contract` checks; after regenerating and before the manifests: the manifest byte gates (domains, station) | high |
| B6 | the re-sow cycle test (`the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years`) goes red as at 9 (settles in two cycles); measured and restated, not widened | medium |
| B7 | the canopy-resistance instrument stays green, because it reads the selector (at 9 it went red only by reading the file's 50 against an overridden flow) | high |
| B8 | green: the TM scorecard (open build), `drought_window`, every carbon / energy / gas test, the conservation and determinism laws | high |

## 7. What "water only" cannot see

The model gives evaporation no heat — no station node and not `thermal.rs` carries latent heat — and the
dehumidifier draws no power for what it condenses. A real chamber transpiring ×1.5–1.8 more would cool its
leaves and its room and load its dehumidifier. So the energy side of this change is **unmeasured, not
zero**.
