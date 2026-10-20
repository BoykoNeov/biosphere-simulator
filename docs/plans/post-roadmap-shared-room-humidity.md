# One moisture store for a shared room

**Opened 2026-10-20** on the user's *"3"* (the shared room's humidity over 100 %) and then *"1"* (one
moisture store, held at the room's own temperature). ⚠ **This reverses a decision the user took on
2026-10-03** (`post-roadmap-room-temperature.md` §19.1: *keep two stocks, record the flaw*); the user
was told so before choosing. The plan and its predictions (§1–§11), then the measurement; **ADOPTED
2026-10-20** (§12: 75 %, the sealed station's thermostat live).

## 1. What today is, measured (2026-10-20, on `75c0ca9`)

Three station builds put the crew and the plants in ONE room: `greenhouse`, `harvest` and
`sealed_station` (the crew's CO₂ and O₂ are the plants' `CARBON_POOL` / `O2_POOL`). Their water vapour
is not shared: the room holds two vapour stocks, each held by its own condenser.

| Stock | Who fills it | Held by | Target |
|---|---|---|---|
| `biosphere.water_vapor` | the plants' transpiration | the plants' `Condensation`, plant step, takes everything above target | 0.75 × saturation at the plants' temperature (`water_cycle.yaml`, BVAD p. 130) |
| `eclss.cabin_h2o` | the crew's `WaterBalance` (4 × 2.946 kg/d) | the ECLSS `Condenser`, minute step, first order `k·max(0, h − target)`, k = 5e-4 /s | 0.40 × saturation at a **stored 22 °C** (`eclss.yaml`, BVAD Table 4-1) |

Measured with a throwaway probe (since deleted; `station/examples/shared_room_humidity.rs`'s two-stock
case reproduces every number below to the printed digit, §7): one 305-day season
of the reference sealed station, the room's whole vapour (both stocks) against saturation at the plants'
temperature, at every plant-step start:

| Plant room | Steps | Room over saturation | Crew vapour ALONE over saturation | Room mean / max | Plants' share | Crew's share |
|---|---|---|---|---|---|---|
| warm, 14–22 °C | 2690 | 2670 | 0 | 1.208 / 1.993 | 0.747 | 0.461 |
| cold, 4–12 °C (the 8-week program after each sowing) | 2190 | **2190** | **2188** | **2.229** / 2.488 | 0.732 | **1.497** |

**Two causes, not one.**
1. *Two stocks in one room.* Their targets add: 75 % + 46 % (the crew's 40 % plus the condenser's
   standing offset `P/k` = 1.364e-4 / 5e-4 = 0.2728 kg; measured `cabin_h2o` 2.0592 − 1.7863 = 0.2729).
   That is the warm weeks' 121 %.
2. *The crew's target assumes 22 °C.* 45 % of the season the shared room is at 4 °C, where air holds
   about a third as much; the crew's 1.79 + 0.27 kg alone is 150 % of saturation. Merging the stocks
   without reading the room's temperature would leave this.

The record's "116 %" and "4 876 of 4 880 steps" (2026-10-03) predate the cold program and the four crew.

**Water flows, one season** (same probe): the plants evaporated 684.8 kg (2.25 kg/d); the crew breathes
and sweats 3594 kg (11.8 kg/d), five times the plants. Crew store 20 000 → 19 466.9 kg; brine 532.6 kg;
the plants' condensate 0 → 33.9 kg; soil 19.5 → 25.9; subsoil 175.5 → 134.1. Rationed 0 / 0.

**Nothing reads the room's sum today** (searched 2026-10-03 and unchanged): no flow, no display. So no
simulated number is wrong *because of* the flaw; the fix is what moves numbers.

## 2. The form

**F1. One vapour stock.** In a shared room the crew's humidity leg goes to `biosphere.water_vapor`;
`eclss.cabin_h2o` is not built. Done the way the gas pools already are: `sealed::CabinAir` gains an
`h2o` field (`WATER_VAPOR` for shared air, `CABIN_H2O` for the separate-air lab, which keeps its own
cabin vapour and must not move).

**F2. One target, at the room's temperature.** Both condensers hold the same target: the room's setting ×
saturation at the temperature the plants' WATER reads for that plant step (`system::water_air_vars`).
That is not the same variable in every build: the sealed station is a room (`lighting::room_form`, the
chamber's daily-mean read, the chamber's own temperature since slice 3a); `greenhouse` and `harvest` run
the default params, whose water reads the weather's **within-day** air (`AirTemperatureRead::WithinDay`,
`AIR_TEMP_WITHIN_DAY_VAR`). The fast side reads the plant window's recorded value of THAT variable (the
reference recorder, `gas_exchange::PlantWindowRecorder`, records `TEMP_VAR` only, so a build whose water
reads the within-day air needs that name recorded too), so the two condensers read ONE thermostat. Two
condensers with different targets on one stock would mean the drier one takes the other's water every
step.
⚠ *Amended after §8:* the measured live thermostat gives the two condensers DIFFERENT temperatures on
purpose (the minute one the chamber's, the plants' the window's), and the harm argued above did not
appear: over one season the two catches sum to the crew's emission plus 0.35 kg (crew condenser
3554.56 + plants' 39.92 = 3594.48 against 3594.13 kg breathed), so the plants lose ~0.35 kg a season
to the crew's side. One thermostat is not required; one target LAW (all of the excess) is.

**F3. The crew's condenser takes everything above the target, every minute** — the plants' frozen
`Condensation` law, instantiated a second time on the fast step, not the ECLSS first-order law. ⚠ Why,
from the numbers: the first-order law leaves a standing 0.27 kg above the target, and the plants'
`Condensation`, which takes ALL the excess, would sweep that up every 90 minutes. Estimated: a refill
`0.2728 × (1 − e^(−5400/2000))` = 0.254 kg per plant step, 4.1 kg/d — about a third of the crew's water
sent into the plants' soil loop each day, 5 t over the sealed run. With the instant law the room sits at
the target and each side keeps **most** of its own water: the crew's breath → `RECOVERED_WATER` (sealed)
/ `HUMIDITY_CONDENSATE` (greenhouse, harvest), as today; the plants' transpiration → `condensate`, as
today. **No re-plumbing of either water loop**, so the "where does the collected water go" choice put to
the user on 2026-10-20 mostly disappears. The ECLSS `k` (DESIGN, a solver-stability value) is not read
by a shared room.

**F3a. What still crosses — two leaks, quantified (advisor, 2026-10-20).** "Each side keeps its own" is
not exact. One store has no owner, so a leak here means: water the plants' condenser catches that, with
two stocks, the crew's would have caught.
* *The minute's residue.* The minute condenser takes the excess at the minute's start while the crew adds
  `4 × 2.946 / 1440` = 0.00818 kg during it, so every minute ends 0.00818 kg above the target, and the
  plants' `Condensation` takes that at the next plant step: 16 × 0.00818 = **0.131 kg/d, 39.9 kg a
  season, 160 kg over the 1220-day run** (of the crew's 3594 kg a season: 1.1 %).
* *The target falling between plant steps.* When the room cools, the plants' condenser (first in the
  window) takes the excess; when it warms, the crew's breath refills it. Measured on the reference
  sealed season (the probe): the 75 % target's drops between consecutive plant steps sum to **5.68 kg a
  season** (rises 5.68), nearly all at the two warm → cold switches, so ≤ 23 kg over 1220 days. In
  `greenhouse` / `harvest` the water reads the within-day air, so the target falls every evening:
  **estimated 1–2 kg/d, up to ~15 kg of the crew's 82 kg over 7 days** (an 8 → 20 °C day moves 75 % of
  saturation in 9500 mol between 1.36 and 2.96 kg). Not measured; the lab measures it.
* Accepted as **two coils in one room**, stated, not designed out: it is real — a room with two
  condensers at one setpoint splits what they catch by timing. The lab reads what each condenser
  catches directly, not from end stores.

**F4. Not in the Tier-1 builds.** `eclss`, `cabin_gas`, `water_recovery` keep their own cabin, their
stored-pressure target (no `exp`) and the ECLSS law: byte-identical. All three shared builds are Tier 2.

**F5. Heat.** The crew's vapour condensing on the room's coil books no heat: the crew's body heat,
latent part included, is in no room's books (a separate item, not this one). The plants' latent-heat
books (`latent_heat.rs`) are unchanged in form; they move only if transpiration moves.

**The setting — the one value choice, priced in the lab, decided by the user:**
* **75 %** — the plants' value already in the tree (BVAD p. 130: plants need *"about 75 %"*,
  *"at the high end for crew comfort"*; Table 4-1's upper crew limit). No new value.
* **40 %** — the crew's nominal (Table 4-1, *"Typical ISS"*). Both condensers at 40 %, so the plants'
  air is much drier and they transpire more. ONE copy reaching every reader: `water.humidity_setpoint`
  is read by `Transpiration`'s saturation split (`system.rs`, `VapourSaturation`) and by `Condensation`,
  both from the run's `BiosphereParams`; the shared room sets it on those params, as
  `lighting::room_form` sets a room's air read, and the minute condenser reads the same params. The
  biosphere's file keeps 0.75 for every chamber that is not shared.

## 3. How it is built — lab first

A switch on the shared builds, `RoomVapour::{TwoStocks, OneStock { setting }}`, default `TwoStocks` (the
reference, bit-identical), and a lab example `station/examples/shared_room_humidity.rs`: one season and
the full 1220 days of the sealed station, plus the 7-day `greenhouse` / `harvest`, each at `TwoStocks`,
`OneStock{0.75}`, `OneStock{0.40}`. The `TwoStocks` side must reproduce the goldens' bytes. Adoption —
the default flipping and the goldens regenerated — is the user's call, after the measurement.

## 4. Blast radius (searched)

* Goldens holding `eclss.cabin_h2o`: `cabin_gas`, `eclss`, `water_recovery` (own cabin; unchanged) and
  `greenhouse`, `harvest`, `sealed_station` (shared; the key disappears on adoption).
  `lighting` has no crew. ⚠ Corrected 2026-10-20 (§11): `sealed_energy_drift_summary` was listed here
  as reading the sealed run's energy; it runs `HEAT_CLOSURE_SCENARIO` (Power → Thermal, no plants, no
  crew, no vapour stock), so this item cannot reach it.
* Manifests: `cabin_h2o` appears only in the authoring manifest (the `eclss.condenser` wiring field) —
  untouched. The station manifest names no vapour stock; its golden hashes move on adoption, and its
  `flow_set` (type names across the station's registries — it already lists the biosphere's
  `Allocation`, moved onto the fast step) gains `Condensation`; `Condenser` stays (the Tier-1 builds).
* `rust/data/tiers.json`: the three goldens share the biosphere band 1e-11 (a ULP-sensitivity band);
  nothing there is keyed to `cabin_h2o` or crew water.
* Re-sowing (`annual_reset_with` → `reset_crop`) touches crop carbon, the stem reserve, litter and
  the soil account's aux — never `water_vapor`. So the crew's water in the plants' stock is neither
  created nor destroyed at a sowing.
* `CabinScenario::cabin_h2o_0` becomes unread in the shared builds at adoption (the separate-air lab
  still reads it). Decided at adoption, not left dead: dropped from the scenarios that only shared
  builds use, or the shared room's start set from it (the plants' `water_vapor0` is 0 today).
* `station::air_split` builds on `build_sealed_station_unread`; the separate-air room keeps its own
  `cabin_h2o` (F1), so `tests/air_split.rs` and its 0.97 fan pin must stay byte-identical.
* `palette.rs:79` names `CABIN_H2O` for the crew-only `cabin_gas` session: unaffected.
* An authored bed may already wire `eclss.condenser` onto the bed's `water_vapor` (`authoring/src/bed.rs`,
  checked against the bed's air): that is an author's own room, with its own condenser law, untouched here.

## 5. Predictions (committed before any code)

**`OneStock{0.75}`, sealed station, one season** (against §1's run):
1. The room never above saturation **at the room's real temperature** — the chamber's live heat-store
   temperature, read on EVERY minute, not the window value both condensers enforce (that check would
   prove itself). Predicted max vapour ÷ live saturation ≤ 0.85 over the season. The risk is the
   warm → cold switch: 75 % of the window-start saturation passes the live saturation if the chamber
   falls more than ~4.5 °C inside one 90-minute window; reported separately for the switch windows.
2. Plant water nearly unchanged: transpiration reads the room's vapour, which was the plants' 0.747
   (warm) / 0.732 (cold) and becomes ≈ 0.75 → evaporated total within −2 % … 0 of 684.8 kg. Soil,
   subsoil, condensate within 2 % (condensate plus F3a's ≤ 46 kg).
3. Crew water: the standing crew vapour (start 1.786, end 2.059 kg) no longer exists, and F3a's leaks
   move 39.9 + ≤ 5.7 kg a season to the plants' side → crew store end **45 ± 10 kg below** 19 466.9
   (less the brine the recovery would have taken of it, ~10 %); the plants' condensate that much above
   33.9. Read directly: what each condenser caught, per season.
4. Carbon (plant stocks, grain peaks, food store) within 0.1 %; re-sow days identical.
5. Energy within 0.01 % (only via transpiration's latent heat).
6. Rationed 0 / 0, events 0.

**`OneStock{0.40}`, same run:**
7. The room at ≈ 0.40 × saturation; the air's dryness rises from 25 % to 60 % short of saturation, so
   transpiration rises — predicted +30 % … +140 % (the canopy and air resistances damp the dryness
   term; the width is the honest uncertainty).
8. The plants' soil and subsoil are drawn faster, and the condensate pool is refilled more; plant
   carbon and grain may fall from water stress, predicted 0 … −10 %. If the subsoil empties within the
   season, that is a finding, not a tuning target.

**Across the suite (at adoption):**
9. Goldens: `greenhouse`, `harvest`, `sealed_station` move (water stocks; `cabin_h2o` gone);
   `sealed_energy_drift_summary` moves at ≤ 1e-4 relative or not at all; the other 17 byte-identical.
   ⚠ Superseded before any adoption run by §11's fresh prediction (the battery's 3.7e-4 already broke
   the ≤ 1e-4 premise, and the golden turned out to hold no room at all).
   In the 7-day `greenhouse` / `harvest` at 75 %: the crew's `humidity_condensate` sink 1–15 kg lower,
   the plants' `condensate` the same amount higher (F3a); no carbon value moves by more than 0.1 %.
   The station manifest: 3 golden hashes and `flow_set += Condensation`, nothing else.
10. `air_split` (all ten) byte-identical; no Tier-1 golden moves.

## 6. Not in this item

* The crew's body heat (sensible and latent) in a room's books — its own item.
* The three mislabelled crew sizings — its own item.
* A crew breathing a room that drops to 4 °C for 8 weeks: the model does not hold the crew's
  temperature, so this item holds the shared room's humidity at the temperature it HAS. Separate rooms
  (the separate-air lab) are the form that would answer it; not taken here.

## 7. Measured, lab-only (2026-10-20; `station/examples/shared_room_humidity.rs`)

The switch is `RoomVapour` on `GreenhouseScenario` / `SealedStationScenario`, default `TwoStocks`;
with it the suite's golden, separate-air, tier and builder-parity tests pass unchanged, and the lab's
two-stock runs reproduce the probe's §1 numbers to the printed digit. Every run below rationed 0 / 0.
"Caught" is read off each condenser's legs, evaluated on each step's own snapshot.

**Sealed station, one season, `OneStock { 0.75 }` (window thermostat) against two stocks:**

| # | Predicted | Measured | |
|---|---|---|---|
| 1 | never above real saturation; max ≤ 0.85 | **max 1.234, above 1 on 143 of 439 200 minutes**; the max falls in the 926 minutes whose live chamber temperature is > 1 °C off its window's (the warm ↔ cold switches) — that all 143 do was not counted on this run | ✗ |
| 2 | transpiration −2 … 0 % | −1.54 % (674.3 vs 684.8 kg) | ✓ |
| 2 | soil, subsoil, condensate within 2 % | soil +22.5 %, subsoil +19.0 %, condensate +41.1 % (+5.8, +25.5, +13.9 kg) | ✗ |
| 3 | crew store 45 ± 10 kg lower; ~10 % of it less brine | −40.5 kg; brine −4.5 kg | ✓ |
| 3 | plants' condenser catches the leaks, 39.9 + ≤ 5.7 kg | 44.5 kg against 5.66 (+38.9) | ✓ |
| 4 | carbon within 0.1 %; grain peaks | plant carbon +0.0000 %; peaks 22.440 / 22.447 (22.440 / 22.445) | ✓ |
| 5 | energy within 0.01 % | node −1.1e-5, chamber +5e-9, **battery +3.7e-4** | ✗ |

* **Miss 2 is a placement, not an amount.** The leak's 40 kg was predicted into the condensate; the
  plants' watering then carries most of it into the root zone and subsoil, which are small stores.
* **Miss 5:** the battery holds 0.04 % more — 10.5 kg less transpiration is ~26 MJ less evaporation
  heat for the chamber's heater to make up. The direction was predicted (only via transpiration);
  the size against a 0.01 % bound was not checked against the battery's own small denominator.
* **Miss 1 is the finding (§8).**

**`OneStock { 0.40 }`:** room max 0.658 of real saturation, never above 1 (✓ 7: ≈ 0.40 away from the
switches). Transpiration **+111.8 %** (1450.1 kg; predicted +30 … +140 %, ✓). ✗ 8: the plants' soil
was NOT drawn down — soil +1.4 %, subsoil +9.5 %, condensate +90 % — and plant carbon rose +0.17 %, grain
peaks 23.244 / 22.812 (+3.6 %, +1.6 %; predicted 0 … −10 %). The chamber's heater pays for the extra
evaporation: battery −9.9 %, node +1.4 %. Why grain rose is **not measured**; no cause is asserted.

**Greenhouse and harvest, 7 days** (the water reads the within-day air; the "real" temperature there
is that air, at the plant window's resolution, so a switch lag cannot show): at 0.75 the room never
passes 0.754 of saturation (two stocks: 1.78 max, above 1 on 9180 of 10 080 minutes); the crew's
`humidity_condensate` −3.09 / −3.10 kg and the plants' `condensate` +3.23 / +3.25 kg (predicted 1–15,
✓); plant carbon +0.0000 %. At 0.40: room ≤ 0.404, crew condensate −1.87 kg, condensate +3.5 / +3.3
kg, soil −2.4 % / −0.2 %, plant carbon +0.0000 %.
⚠ **The "never passes 0.754" check proved itself**: it read the same plant-window temperature the
condensers enforce. The within-day air is a continuous curve, evaluable over any minute, so §10 re-reads
it minute by minute — and the premise "a switch lag cannot show" was wrong.

## 8. The thermostat lags the room at the switches

Both condensers hold a target set at the plant window's temperature. When the chamber's program
switches 22 → 4 °C, the chamber cools inside a 90-minute window while the target stays at the
window's start: the room holds 75 % of a saturation the air no longer has. 143 minutes a season pass
saturation, up to 1.23×. The shared room's crew condenser runs every minute and the sealed build has the
room's live temperature (the chamber's heat store), so a fourth case reads it: `Thermostat::LiveChamber`
(the plants' `Condensation` law, through the plants' own chamber wrapper `chamber::reads_chamber`, on
the fast step). The greenhouses have no heat store and refuse it. Predicted before the run: room max
≤ 0.76 of real saturation (one minute's breath above the live target); the lag's water now goes the
other way at a cooling switch (the minute condenser takes the room's excess to the crew's recovery
before the plants' condenser sees it), so the plants' condenser catches less than the window case's
44.5 kg, and the crew store ends nearer the two-stock run.

**Measured (same day, same tool; the first three cases repeat to the digit):**

| Case | Room ÷ real saturation, max | Minutes above 1 | Plants' condenser | Crew store vs two stocks | Grain peaks |
|---|---|---|---|---|---|
| window @ 0.75 | 1.234 | 143 | 44.5 kg | −40.5 kg | 22.440 / 22.447 |
| **live @ 0.75** | **0.7601** | **0** | **39.9 kg** | **−36.5 kg** | 22.440 / 22.447 |
| window @ 0.40 | 0.658 | 0 | 60.4 kg | −38.6 kg | 23.244 / 22.812 |
| live @ 0.40 | 0.408 | 0 | 40.1 kg | −36.3 kg | 23.180 / 22.778 |

Every prediction for the live case held **but one, by 1e-4: the ceiling, 0.7601 against "≤ 0.76"**.
One minute's breath above a 4 °C target is 0.6 % of saturation (0.756); what carries it past 0.76 is
**not measured** on this run (a candidate, the chamber cooling inside the minute, is counted by the
`--full` run's "above setting + one minute's breath" readout, §9).
The plants' condenser now catches the minute's residue and nothing else — 39.9 kg, the §2 F3a figure to
the digit — so the switch leak is gone in both directions. Plant water, carbon and energy are the window
case's within its own digits (battery +3.77e-4, the same transpiration cause). At 0.40 the live
thermostat brings the room to its setting (0.408) where the window one reached 0.658 at the switches.

## 9. The 1220-day sealed run (`--full`, 2026-10-20)

All five cases, the whole run (1 756 800 minutes, eight full seasons and the start of a ninth);
rationed 0 / 0 in every case. Against two stocks:

| Case | Room ÷ real saturation, max | Minutes above 1 | Plants' condenser | Crew store | Battery, lowest | Grain peaks |
|---|---|---|---|---|---|---|
| two stocks | 2.554 | 1 747 351 | 22.9 kg | — | 1.025e10 J (51 % of start) | 22.437 … 22.446 |
| window @ 0.75 | 1.261 | 573 | 178.1 kg | −161.7 kg | 1.028e10 J | 22.437 … 22.448 |
| **live @ 0.75** | **0.7603** | **0** | **159.7 kg** | **−146.1 kg** | **1.028e10 J** | 22.437 … 22.448 |
| window @ 0.40 | 0.662 | 0 | 245.0 kg | −154.0 kg | **3.17e9 J (16 %)** | 22.476 … 23.385 |
| live @ 0.40 | 0.408 | 0 | 160.4 kg | −145.7 kg | **3.18e9 J (16 %)** | 22.595 … 23.425 |

* **Live @ 0.75 holds over the long run.** Never above saturation; the plants' condenser catches 159.7
  kg against §2 F3a's 160 kg; the crew store −146.1 kg = 4 × the season's −36.5. Soil +14.9 %, subsoil
  +27.0 % (the leak's placement, as in §7), battery +2.9e-3, node −9.8e-5, chamber +2e-7.
* **The 0.7601 ceiling miss (§8):** over the run the live room passed "setting + one minute's breath"
  on 33 504 minutes, and on **every one of them** the chamber was cooler than the minute before. That is
  what the candidate (the chamber cooling inside the minute, after the condenser set its target)
  requires; it supports the candidate, it does not isolate it.
* **The window thermostat's switch leak recurs every season**: 573 minutes above saturation (143 a season).
* **40 % is not a free choice: it drains the battery.** The extra evaporation is paid by the chamber's
  heater from the battery: the lowest charge over the run is 16 % of the starting 2.0e10 J against two
  stocks' 51 %, and the end charge is 69 % below the two-stock run's. Whether it would run out on a
  longer run is not measured, nor whether it had stopped falling: the lowest charge may be the run's
  end. Grain peaks rise again (up to +4.4 %); the cause is still not measured.
* **⚠ Open at 40 %: the crop is a third smaller on the run's last day.** Plant carbon on day 1220 is 34 %
  (window) / 41 % (live) below two stocks, while both 75 % cases sit at −0.002 % on the same day — so the
  day alone does not explain it. A later ninth sowing (maturity moved; P4's re-sow days were not logged)
  or slower growth would each produce it; neither is measured. It stands beside the grain rise and the
  battery drain. A rerun logging each re-sow step and each season's lowest battery charge would answer it.

## 10. The greenhouses, read minute by minute

`RealTemp::OutdoorAir` re-reads the weather's within-day air curve over each MINUTE (the plants' water
reads the same curve averaged over the plant window). Seven days, the same three cases:

| Build | two stocks | window @ 0.75 | window @ 0.40 |
|---|---|---|---|
| greenhouse: max, minutes above 1 | 1.829, 9180 of 10 080 | **0.821**, 0 | 0.439, 0 |
| harvest | 1.829, 9180 | **0.821**, 0 | 0.439, 0 |

The window thermostat lags the outdoor air too (0.821, not 0.754), but by a 7 % margin, not past
saturation. A live thermostat for the greenhouses is buildable (the curve is evaluable per minute,
which §8's "refuse it" premise missed) and is NOT built; at these numbers it is not needed to keep the
room below saturation. Water and carbon readouts repeat §7's to the digit.

## 11. A fresh prediction for `sealed_energy_drift_summary` (committed before any adoption run)

§5 P9 ("≤ 1e-4 or not at all") rested on the sealed run's energy. The golden is
`goldens::sealed_energy_drift`, the 15-year `HEAT_CLOSURE_SCENARIO` (Power → Thermal, built by
`build_station` with no crew, plants or vapour). **Prediction: byte-identical at adoption.** If it
moves, the adoption reached a build it should not, and that is a finding. So P9 becomes: `greenhouse`,
`harvest` and `sealed_station` move; the other 18 goldens are byte-identical.

**What the user decides (adoption):** the setting, 75 % or 40 % (40 % costs the sealed station's battery,
§9); and the sealed station's thermostat — live (never above saturation over 1220 days) or the window
(573 minutes above it, at the switches). The greenhouses stay on the window (max 0.821, §10) unless the
user asks for a live one.

## 12. ADOPTED 2026-10-20 — 75 %, the sealed station's thermostat live

**The user's answer:** *"75%, live"*. Live is the sealed station's only (the greenhouses have no
chamber heat store, §8); they keep the plant window's thermostat, as §11 said.

**The form, as built.** `room_vapour::shared_room(thermostat)` is the reference: one stock at the
plants' own setting, read from `water_cycle.yaml` (0.75, not a second copy). `greenhouse_scenario()` (and
so `harvest_scenario()`) takes `shared_room(Window)`, `sealed_station_scenario()` takes
`shared_room(LiveChamber)`. `TwoStocks` stays, as the lab's record of the form until today.

**Order (advisor-reviewed before code):**
1. Lab first: `shared_room_humidity -- --save <dir>` runs the three goldens' own builds; the two-stock
   side reproduced each committed golden's bytes, then the chosen room's final state was saved
   (committed alone, `c94bd63`).
2. The default flipped; the full suite run with `--no-fail-fast` to see every red before fixing any.
3. `regen_goldens` report: **exactly `greenhouse`, `harvest`, `sealed_station` CHANGED; the other 17
   identical**, `sealed_energy_drift_summary` included (§11's fresh prediction ✓). ⚠ §5 P9 and §11 said
   "the other 18": the roster runs 20 goldens (the 21st file in `rust/data/golden/` is not in it).
4. `--write`; each regenerated golden is **`cmp`-identical** to the lab's saved file.
5. The station manifest: the 3 `golden_sha256` rows and `flow_set` + `Condensation` — **exactly §4 / P9's
   prediction**. The authoring manifest did not move (its writer test passed).

**What the goldens moved** (old → new, final day):

| | greenhouse | harvest | sealed station |
|---|---|---|---|
| `eclss.cabin_h2o` | removed | removed | removed |
| crew condensate (`humidity_condensate` / crew store) | −3.09 kg | −3.10 kg | crew store −146.1 kg; brine −16.2 kg |
| plants' `condensate` | +3.23 kg | +3.25 kg | +134.4 kg |
| soil / subsoil | +0.58 % / — | +0.07 % / — | +14.9 % / +27.0 % |
| carbon | no bit | no bit | plant stocks within 0.003 % |
| energy | — | — | battery +0.29 %, node −9.8e-5, chamber +2.3e-7 |
| aux | + `station.plant_window.air_temp_within_day` (the recorder §2 F2 named) | the same | evaporation total −1.34 % |

Every row repeats the lab's §7–§9 readouts to their printed digits. **Not predicted, not traced:** the
sealed station's top soil layer on the last day, 8.8 → 28.5 kg (+223 %); the lab never printed it. The
watering's timing on that day is a candidate (the CO₂ item saw the final-day water split move with its
sum unchanged), not a measured cause.

**`cabin_h2o_0`:** kept, unread by the three shared builds (documented on the field). The crew-only cabins,
the two-stock form and the separate-air lab read it. No start value moved in this batch (the plants'
`water_vapor0` stays 0), so the saved states stayed the prediction.

**The reds, and what each became** (31 tests in 9 binaries, all read before any fix):
* `golden_regression`, `tier_contract`, `manifest_writer` — the reference moving; cleared by steps 4–5.
* `air_split.rs` (10) and `minute_transpiration.rs` (4) — `build_split_station` refused the new default.
  Separate air builds two rooms, so `split_scenario` now sets `TwoStocks` itself; the refusal now stands
  only for a one-store setting other than the plants' own, which separate air would drop. Then one
  `air_split` pin read against the one-store shared baseline: "vapour off holds the shared plants' water
  loop exactly" measured 195.0 against 237.5 kg — the crew's breath now reaches the shared plants' water,
  so the claim is about the two-stock room. The file's baseline (`scenario()`) is pinned to two stocks,
  beside its old crew load and old scrubber; all ten pass, the 0.97 fan pin untouched.
* `day_order.rs` (5), `gas_exchange.rs` (7), `latent_heat.rs` (1) — plant-step (or no-plant) builds
  refused: a one-store condenser reads the minute step's recorded window. Their subject is the step
  machinery, the same on either form, so their plant-step builds are pinned to two stocks;
  `every_crop_build_steps_the_carbon_budget_exactly_once` keeps its minute-step builds on the reference's
  one store.
* `chamber_walls.rs` — `walls_onto_the_station_structure_run_the_heater_continuously`: the node
  188.7014 → **188.5947 K**, the heater 139.193 → 139.163 W (inside its band already). Re-pinned as
  measured, band unchanged; neither size nor sign predicted (earlier re-pins moved the node down both when
  the plants evaporated less and when they evaporated more).
* Found only by the `--ignored` run, after the goldens: `chamber_walls.rs`'s 1220-day
  `the_reference_node_and_chamber_follow_the_weather` — node min / max / mean 162.4702 / 175.3236 /
  170.0607 → **162.4686 / 175.3114 / 170.0318 K** (max and mean past their 0.01 bands), the chamber's end
  295.1355085 → **295.1355778 K**; 9 day ends between the bands, as before. Re-pinned as measured, bands
  unchanged. Colder on the mean: the run's evaporation total is −1.34 %, and this test's own history has
  less evaporation → a colder node (2026-10-16). Size not predicted.
* Examples built on plant-step shared rooms (`air_split`, `air_split_baseline`, `intraday_exchange`) take
  two-stock scenarios, as their records were measured; nothing runs examples, so these were found by
  grep, not by a red.


## 13. One water reserve for the station (opened 2026-10-20)

**How it was asked.** The decision summary for §12 said that at 75 % "nothing else moved", leaving out
§2 F3a's ~40 kg a season of the crew's water reaching the plants' soil; the user was told. Asked "keep,
remove the drift, or undo", the user first chose *"Add the missing pipe: send the plants' collected water
to the crew's water recovery — do it"*. That pipe (the plants' `Condensation` booked to `RECOVERED_WATER`,
`room_vapour::Repointed`) was built lab-only and **replaced before any of its results was read**: the
user's next message was *"i want system that balanced water needs. also - i think it is reasonable for the
station to have a tank with xtra water to serve as a buffer for all needs"*. Its code was reverted, its
runs stopped, its saved bytes deleted.

**The user's three answers (2026-10-20):**
1. The reserve is the crew's existing clean-water tank (`crew.water_store`, 20 t). No new tank.
2. The plants' extra goes through the crew's recycler (`RECOVERED_WATER`, η 0.9) on its way there.
3. The plants' own tank (`biosphere.condensate`) keeps **one full watering**; above that it drains.

**The form (sealed station only):**
* *Top-up* (plant step): when the plants' own watering triggers (`EventRecycling`'s trigger, the same
  builder) and their tank holds less than the watering's gross `deficit / ea`, the rest is drawn from the
  crew's store, split as the plants' watering splits (`ea` to the root zone, the rest below it). Counted by
  the soil account (`soil_surface_account_with`), as the separate-air watering is.
* *Drain* (minute step, so it never shares a snapshot with the watering): everything in the plants' tank
  above the level goes to `RECOVERED_WATER`.
* *The level* — a DESIGN value derived from existing settings, no new number: `depletion_fraction ×
  transpirable_capacity(min(max_rooted_depth, soil_depth)) / drip_efficiency` = 0.55 × 1.3 m × 0.13 ×
  1000 kg m⁻³ × 1 m² / 0.9 = **103.3 kg** (one watering of the deepest root zone, trigger to full).
  ⚠ Amended after the probe below: first written from the soil's depth (1.5 m, 119.2 kg), but the roots
  stop at 1.3 m (`root_depth.yaml`), so no watering can be bigger than ~103.8 kg.
* Not the greenhouses (their crew water goes to a sink) and not the separate-air lab (it already waters
  from the crew's store).

**Measure before building (advisor, 2026-10-20).** The plants' tank held 33.9 kg after the first season
under two stocks (§1). If it rarely passes 119 kg, the drain returns little and the crew's breath is not
"returned", only capped; if the tank is rarely short, the top-up is insurance the reference never uses. On
today's reference, over 1220 days: what the top-up would supply, what the drain would return, the largest
watering delivered, the deepest roots, the plants' tank over time.

**The probe** (`station/examples/water_reserve.rs` as first written, today's reference, 1220 days, 9 crop
cycles; rationed 0 / 0). Every watering trigger (4 a cycle) found the plants' tank holding enough: **the
top-up would never fire**. The plants' water climbs every cycle — 216.6 kg at the first cycle's end, then
234.9, 253.1, … 357.6 kg: **+18.3 kg a cycle**, the crew's minute residue (0.131 kg/d × ~140 days). The
tank's peak passes one watering in every cycle from the second on (111.2, 129.3, 148.0, … 242.7 kg). So
the drain returns the drift, and the top-up is insurance the reference does not use. Largest watering
103.8 kg; roots 1.30 m (hence the level's amendment above).

**Predictions** for `CrewStore` against `Off` (today's reference), 1220 days, committed before the run:
* P1. The drain returns **130 ± 30 kg** in all (from the first or second cycle on); the crew store ends
  **+117 ± 27 kg** (η 0.9), brine **+13 ± 3 kg**.
* P2. The plants' water **levels off**: no cycle-to-cycle rise of its end value above 5 kg after the
  second cycle; it ends near the second cycle's end (~235 ± 20 kg) against `Off`'s 357.6.
* P3. The top-up: **0–30 kg** in all. With the tank's peaks clipped its troughs fall too, so a late
  watering in a cycle may find it short; no prediction of which.
* P4. Carbon within 0.1 %; grain peaks as `Off`'s to the third digit; rationed 0 / 0; re-sow days
  identical.
* P5. Goldens: **exactly `sealed_station` changes**. Station manifest: its `golden_sha256`, and `flow_set`
  gains **`ReserveTopUp` and `TankDrain`** (station flow types; the completeness gate may ask each to be
  exercised).
* P6. `chamber_walls`' pins move (less water in the plants' loop), size not predicted.

**Measured, lab-only** (`water_reserve` example, 1220 days, 9 crop cycles; both rationed 0 / 0):

| | predicted | measured | |
|---|---|---|---|
| P1 drain, in all | 130 ± 30 kg | **143.70 kg** (8.4 in cycle 0, then 18.4–19.2 a cycle) | ✓ |
| P1 crew store, end | +117 ± 27 kg | **+125.03 kg** | ✓ |
| P1 brine, end | +13 ± 3 kg | **+14.37 kg** | ✓ |
| P2 plants' water levels off | rise ≤ 5 kg a cycle | 208.874, 208.885, 208.907, 208.885, 208.906, 208.884, 208.909, 208.883 kg | ✓ |
| P2 … near the second cycle's end | ~235 ± 20 kg | **208.9 kg** | ✗ |
| P3 top-up, in all | 0–30 kg | **4.30 kg** (2 of 4 waterings a cycle find the tank short) | ✓ |
| P4 carbon; grain peaks | within 0.1 %; as `Off` | plant carbon +0.0000 %; grain peaks identical to 4 digits | ✓ |

P2's level missed: the tank's peak (111.2 kg) passed the level already in the first cycle, so the drain
started there and the plants' water settled at the first cycle's end less 7.7 kg, not the second's. The
soil and subsoil end identical to `Off`'s to 4 decimals: the water that leaves is the tank's surplus, which
no watering reached. The tank now swings 0.2–103.7 kg each cycle; its two last waterings are short by
0.1–1.0 kg, the top-up's whole work. `--save` wrote the sealed golden's predicted bytes
(to a temp folder, since deleted: its file is `cmp`-identical to the committed
`rust/data/golden/sealed_station_state.json`, §14); `Off` reproduced the committed golden first.

## 14. ADOPTED 2026-10-21 — the station's water reserve

**The flip:** `sealed_station_scenario()` takes `WaterReserve::CrewStore`. The full suite with
`--no-fail-fast` before any fix: **1458 passed, 3 red**, each read first:
* `manifest_writer` — the station manifest's `flow_set` gains `ReserveTopUp` and `TankDrain` (P5); cleared
  by regenerating it.
* `chamber_heat.rs` `a_dead_cooler_heats_the_chamber_cools_the_node_and_overheats_the_crop` — *"the hot
  room's crop never stopped evaporating"*. **A behaviour change, not predicted:** with the cooler dead the
  room overheats and the crop dies; without the reserve the plants' tank runs dry, the soil dries and
  evaporation stops (day ~15), and the test's node bound is derived from that day. With the reserve the
  crew's tank keeps watering the hot room's soil on its dryness trigger, as a real watering system would,
  so it never stops within 60 days. The file's subject is the node, so `warm_scenario()` is pinned to
  `Off` (⚠ doc), and the reserve's side is a test of its own,
  `a_dead_coolers_room_keeps_drawing_on_the_reserve` (still evaporating on day 60).
* `air_split.rs` `vapour_crossing_moves_water_between_the_plants_and_the_crew` — "vapour off holds the
  shared plants' loop exactly" read 195.0 kg against 204.5: the shared side's two-stock baseline now has
  its short waterings topped up from the crew's tank (two stocks leave the plants' tank shorter). A claim
  about two rooms that each keep their own water; the file's baseline is pinned to `Off` beside its
  `TwoStocks` (⚠ doc).

**After the fixes:** clippy clean; the full suite 1462 passed, 0 failed; `--ignored` 12 passed, 0 failed. **P6 missed:** no `chamber_walls` pin moved. Consistent with §13's table (soil and subsoil +0.0000: the reserve moves the plants' tank, not the water that evaporates), not traced further.
