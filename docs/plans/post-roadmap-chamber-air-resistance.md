# Post-roadmap: pricing a chamber air resistance — what a lower `ra` would move

**Opened 2026-10-14 on the user's call** (*"2"*: price changing the air-resistance value), chosen
after Step 6's water-gap search (`post-roadmap-real-world-checks.md` §11–§12): at the trial's level
(4.17 L m⁻² d⁻¹, 20 m² basis) the model's crop drinks 0.62 of TM 102788's water, and a canopy air
resistance `ra` of ~8.8 s m⁻¹ instead of the uncited 50 would close it alone (§11b Q4).

**This is a PRICE, not an adoption.** Nothing on main changes; the run is on a local branch, never
pushed. Adopting any value would be a biosphere + station unfreeze, and a science item.

## 1. The user's decisions (2026-10-14)

1. **Values: a spread, 50 → 30 → 15 → 9 s m⁻¹** (over the fitted ~9 alone, and over stopping to look
   for a source). The spread shows the cost curve without choosing a number. ⚠ No value below 50 has a
   source: 9 is FITTED to TM 102788 (it would pass that row by construction), and 30 / 15 are points on
   a curve, not citations. Adopting one would need either a source or the user setting aside the rule
   that tuned values stay lab-only (`docs/param-file-conventions.md`, WHAT-IF).
2. **Scope: the open field keeps 50**; the chamber value applies to enclosed rooms. Priced as **two
   lines**:
   * **Line L — lamp-lit chambers only** (the user's first pick): the `lighting` and `sealed_station`
     builds (and the lab builds derived from them: day-neutral lighting, separate air).
   * **Line S — every sealed chamber** (added on the user's pick *"price both lines"*, after the scenarios
     turned out to fall in three groups, not two): Line L plus the five weather-lit sealed builds —
     the sealed, perennial and consumer chambers, the greenhouse and the harvest.

## 2. Advisor review (2026-10-14), summarized

Ask the value and the scope before any code (done, §1). Inventory every reader of `ra` before
predicting. Re-check the station's heat books for latent heat (the 2026-10-07 check predates the
2026-10-10 water-forms adoption). Do NOT carry "water only" over from the leaf-area pricing: that
change LOWERED water use; this one RAISES it (~1.6× at full cover), so any build whose watering
cannot keep up could cross the stress thresholds and move carbon — predict it per build. Same method
as the leaf-area pricing (`post-roadmap-canopy-resistance.md` §9–§10): predictions first, a local
branch, `regen_goldens` in report mode, the full suite `--no-fail-fast` plus `--ignored`, every red
classified, a price table and no recommendation.

## 3. Checked before predicting

* **One reader.** `ra` reaches the model in exactly one place: `Transpiration`'s
  `aerodynamic_resistance` (`domains/src/biosphere/flows.rs`, both the soil-split path and the plain
  path), built by `transpiration_flow` in `system.rs` from `p.transp`. The soil-evaporation potential
  does not read it; no leaf temperature is modelled (the §11 leaf-over-air figures are the instrument's).
* **No latent heat in the station** (re-checked 2026-10-14): no `station/src/*.rs` and not
  `domains/src/thermal.rs` carries a latent or evaporative term. More transpiration cannot cool or warm
  any node, so the plants' temperature cannot move through this route.
* **The station's builders call `build_season(&scenario.bio)`**, which reads `params::biosphere()`
  itself; their own `params::biosphere()` calls seed only the soil account. So a scope must be carried
  by the **scenario**, not by the params a builder passes. This matches the physics: `ra` is a property
  of the room's air movement (the param file itself says "wind/height-dependent"), like `ground_area`.
* **Every lamp-lit build is sealed** (`lighting_bio_scenario`, `day_neutral_lighting_bio_scenario`, the
  sealed station's), so Line S contains Line L.
* **The open-build TM scorecard is outside both lines**: `scorecard_tm102788.rs` hosts the held-CO₂
  trial on an OPEN build with the lamp's forcings swapped in, so a scenario-scoped value does not reach
  it. Its W1 row at each value is the §11 instrument's (`ra` substituted through the what-if path).
* **Watering:** event watering (adopted 2026-10-10) refills at FAO-56's depletion fraction 0.55 (FTSW
  0.45) with a capacity of 8 mm/day; sealed chambers water from their condensate (`Recycling`).
  Development speeds up below FTSW 0.40 (`wssd`), growth and transpiration slow below 0.30 (`wssg`).

## 4. The arithmetic (the model's constants; `W:\temp\claude\air-res-price\pm.py`)

Penman–Monteith at full cover, `rs` 50 (Szeicz–Long at LAI ≥ 2), demand per day:

| `ra` | lamp (TM point: 20 h at 20 °C, Rn 81, VPD 444; 4 h dark at 16 °C) | sunlit sealed, air at 75 % RH: 10 °C / Rn 40 | 17 °C / Rn 100 | 22 °C / Rn 130 |
|---|---|---|---|---|
| 50 | 2.58 mm/d (×1.00) | 1.76 | 3.31 | 4.43 |
| 30 | 2.99 (×1.16) | 2.13 (×1.21) | 3.74 (×1.13) | 4.97 (×1.12) |
| 15 | 3.64 (×1.41) | 2.68 (×1.52) | 4.41 (×1.33) | 5.84 (×1.32) |
| 9 | 4.12 (×1.60) | 3.04 (×1.73) | 4.89 (×1.48) | 6.50 (×1.47) |

The highest daily demand in the table, 6.5 mm/day at `ra` 9 on a warm bright day, is under the
watering's 8 mm/day capacity. A sealed chamber's air holding near its 75 % humidity target limits the
deficit further (more transpiration → humid air → condensed by the dehumidifier, not a drier leaf).

## 5. Predictions (written before the branch exists)

| | prediction | confidence |
|---|---|---|
| P1 | the branch with no override is inert: `regen_goldens` reports 20 of 20 identical | high |
| P2 | Line L at 30, 15 and 9: exactly `lighting_state` and `sealed_station_state` would change; the other 18 identical | high |
| P3 | Line S at 30, 15 and 9: those two plus `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`, `greenhouse`, `harvest` — **nine**; `season_euler`, `drift_summary`, `sealed_energy_drift_summary` and the eight crop-free goldens identical | high |
| P4 | **water only, in every moved golden at every value**: water stocks (soil, subsoil, condensate, vapour) and the soil account's water aux move; no carbon, nitrogen, oxygen, energy or phenology value moves — because watering keeps FTSW above 0.40 everywhere (§4: peak demand under capacity) | medium — the claim the advisor said not to assume; a carbon move refutes it and is the headline |
| P5 | the size grows with `1/ra`: every stock that moves at 30 moves further at 15 and further again at 9, same sign | medium (end states are snapshots inside a watering cycle, so a single stock can be non-monotone) |
| P6 | reds by design on the branch at Line S / 9: `golden_regression` (domains and station) and the tier bands on the nine moved goldens; the sealed station's golden in `--ignored`. The manifest byte gates stay **green** (report mode does not rewrite the files — the 2026-10-07 lesson) | high |
| P7 | possibly red, classified after: tests pinning water, humidity or condensate in a sealed or lamp-lit run (chamber dryness, vapour, dehumidifier, watering, minute-step transpiration, separate air, lamp shed, the re-sow cycle test) | — |
| P8 | green: the TM scorecard (outside both lines), every carbon / energy / gas test, `drought_window`, the conservation and determinism laws | high |
| P9 | Line L's reds are a subset of Line S's at the same value | high |

## 6. The branch (throwaway)

`wip/chamber-ra-price`, local, never pushed. A scenario field `lamp_lit` (true on the three lamp-lit
scenario builders) and a pricing hook in `transpiration_flow` that reads two environment variables —
`PRICE_RA_SCOPE` (`lamp` | `sealed`) and `PRICE_RA_VALUE` — so one build prices all six cells. An
environment read is NOT an adoption shape (parameters are data); it exists only so the price runs
without six recompiles. Logs: `W:\temp\claude\air-res-price\`.
