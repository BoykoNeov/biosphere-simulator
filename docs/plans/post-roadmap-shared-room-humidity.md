# One moisture store for a shared room

**Opened 2026-10-20** on the user's *"3"* (the shared room's humidity over 100 %) and then *"1"* (one
moisture store, held at the room's own temperature). ⚠ **This reverses a decision the user took on
2026-10-03** (`post-roadmap-room-temperature.md` §19.1: *keep two stocks, record the flaw*); the user
was told so before choosing. **This doc is the plan and its predictions. Nothing is built.**

## 1. What today is, measured (2026-10-20, on `75c0ca9`)

Three station builds put the crew and the plants in ONE room: `greenhouse`, `harvest` and
`sealed_station` (the crew's CO₂ and O₂ are the plants' `CARBON_POOL` / `O2_POOL`). Their water vapour
is not shared: the room holds two vapour stocks, each held by its own condenser.

| Stock | Who fills it | Held by | Target |
|---|---|---|---|
| `biosphere.water_vapor` | the plants' transpiration | the plants' `Condensation`, plant step, takes everything above target | 0.75 × saturation at the plants' temperature (`water_cycle.yaml`, BVAD p. 130) |
| `eclss.cabin_h2o` | the crew's `WaterBalance` (4 × 2.946 kg/d) | the ECLSS `Condenser`, minute step, first order `k·max(0, h − target)`, k = 5e-4 /s | 0.40 × saturation at a **stored 22 °C** (`eclss.yaml`, BVAD Table 4-1) |

Measured with a throwaway probe (`W:\temp\claude\humidity_probe\`, outside the repo): one 305-day season
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
  `lighting` has no crew. `sealed_energy_drift_summary` reads the sealed run's energy.
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
