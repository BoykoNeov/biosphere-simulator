# Post-roadmap — the big station (opened 2026-10-21)

**Status: PLAN. Step 1 (a measuring lab) not started.** Nothing here changes the reference.

## 1. Why this exists

The user's statement of the whole project (2026-10-21): component models as close to reality as is
reasonable, at reasonable speed, combined at different scales, to simulate relatively big closed and
open biospheres. On learning that the reference station's 1 m² wheat bed carries well under 1 % of its
four crew's air (`docs/plans/post-roadmap-intraday-gas-exchange.md` §6: 0.030 % over 4 years), the
user set the floor: *"4 people and 200 square meters of crops will be the lower limit that we work on.
we aim for a whole ecosystem, bigger than biosphere-2"*.

How the station came to be 1 m²: Phase 6 wired the per-m² crop to the crew as "a direction +
conservation demo, not a self-sufficient loop" and deferred crew-vs-plant sizing to its Step 9
(`docs/plans/phase-6-station-integration.md`, the harvest step's "Honest scope" note). Step 9 calibrated the crew to
BVAD and did not resize the farm; the station froze as "illustrative". The mismatch was recorded
again on 2026-08-09 (`docs/plans/post-roadmap-chamber-scale.md`) as a fact about the rig, never as a
reason to change direction, and post-roadmap work kept refining the rig.

**What carries over:** the crop model works per m², so its science (night stomata, watering trigger,
evaporation, soil layers, lamp heat) holds at any bed size. **What will not:** every room-level number
(air buffers, humidity, heat, CO₂ pools) is measured on a 1 m² room and must be measured again.

**Scope decisions (user, 2026-10-21):**
- Open biospheres (an outdoor farm, a greenhouse trading air with outside, a natural landscape) are
  **future spin-offs** building on this science — not built here.
- Light and power are **choices**, in this order: (1) outside electricity (a research facility on
  Earth — power is not a limit, only metered) for the first test; (2) sunlight through glass (the
  Biosphere 2 path: glazing, overheating, seasons); (3) lamps on solar panels and lamps on a fission
  reactor. (A radioisotope generator gives ~100 W; 40 kW would take hundreds, so "nuclear" means a
  fission reactor.)
- **One adoption, not many** (advisor, 2026-10-21): build the big station as a lab scenario and adopt
  it in ONE unfreeze. The approved 98 % water recycler (NASA 2023, already cited in
  `water_recovery.yaml`) is folded into that adoption instead of moving the goldens twice.

## 2. What is sized for 1 m² today (read from the code, 2026-10-21)

| Part | Today | At 200 m², if left alone |
|---|---|---|
| Lamp | `lamp_power_w` 200 W (PAR = efficacy·power/area) | PAR ÷ 200: the crop is in the dark |
| Chamber cooler / heater | `chamber.yaml` 200 W / 200 W, DESIGN, "sized to the installed power" | 40 kW of lamp heat against 200 W: overheats |
| Chamber heat capacity, walls | 1.5e5 J/K (root-zone water of 1 m²), 5.12 m² of wall | wrong by ~200× |
| Power | solar 1 kW peak, battery 2e10 J | lamps (~27 kW average) empty the battery in ~9 days |
| Room air | shared room 9 500 mol ≈ 230 m³ (crew + bed) | 1.15 m³ per m² of crop — BVAD's flight-chamber density, not a Biosphere 2 room (~16 m³/m²) |
| CO₂ | scrubber removes everything above 0.13 kPa to `boundary.co2_removed` | the night's CO₂ is thrown away; the day's crop can draw only what the crew breathes out |
| O₂ | `O2Makeup` is two-sided (gain·(setpoint − amount)): removes a surplus to the supply | holds — a plant surplus goes back to the tank |
| Food | `Harvest` exists, off in the sealed build (`with_harvest`, `close_feces` default off) | the crew eats a 500 kg store; nothing returns |
| Water reserve | working level 103.28 kg per m² of bed | 20.7 t — more than the crew's whole 20 t store |
| Soil | 1.5 m deep | 300 m³ of soil — a design question (Biosphere 2's farm; NASA used hydroponics) |
| Crew stores | food 500 kg, water 20 t | unchanged by area |

The existing scaling helper, `examples/intraday_exchange.rs::scale_area`, scales the crop's starting
amounts and optionally the chamber's air, and nothing in this table's lamp / thermal / power / reserve
rows.

## 3. Step 1 — measure a 200 m² station (a lab, no reference change)

One new example: the sealed station at 200 m² and 4 crew, outside electricity, everything in §2 that
is a per-m² sizing scaled with area (lamp, cooler, heater, heat capacity, wall area, reserve level,
crop starts), and the rest left as today (crew, stores, CO₂ target, room air as one choice to vary).
"Outside electricity" for the lab = a battery large enough never to bind, reported as energy drawn
(a stand-in, recorded as such; a real grid source is a build item if adopted).

**Run speed is not a question here**: area is one number on scalar stocks, so 200 m² costs what 1 m²
costs. Speed is measured when rooms are composed (Step 4).

**Predictions (written before any run):**
- **B1.** Crew output: 4 × 24.654 = 98.6 mol CO₂ per day; O₂ use 4 × 27.97 = 111.9 mol per day (BVAD
  Table 3-31, by construction).
- **B2.** An unstarved crop at today's per-m² rate (~48 mol C per ~136-day crop, ~0.35 mol C per m² per
  day) would fix ~70 mol C per day at 200 m² — ~70 % of B1 averaged over a crop, far above 100 % at
  peak, near 0 for the seedling.
- **B3.** It will be starved instead: the scrubber's pool at 0.13 kPa holds a few mol, the lit crop at
  peak asks several times the crew's hourly output, so in lit hours uptake is capped near the crew's
  exhalation rate, and the night's CO₂ is scrubbed away. Closure share over the run **20–50 %**, crop
  carbon per m² **below** the 1 m² crop's.
- **B4.** O₂ never climbs without bound: the regulator returns any surplus to the supply tank.
- **B5.** Water through the plants ~200× the 1 m² run (~209 kg per crop per m² → ~42 t per crop, ~300
  kg per day); the room's condensers carry it; the crew's store does not run dry over 1220 days.
- **B6.** Lamp energy ~640 kWh per day; the room holds its setpoint with the cooler scaled; rationing
  counts reported, not used as the verdict (`rationed == 0` cannot measure starvation).

**Readouts:** closure share (plant net carbon draw ÷ crew exhalation), daily and per crop; crop carbon
per m² against the 1 m² run; CO₂ and O₂ in the room (min/max); scrubbed and made-up totals; water
through plants, crew store, reserve top-ups; lamp energy and room temperature range; rationing.

## 4. After Step 1 (each its own choice, ordered by what Step 1 shows)

1. **CO₂ to the plants** (if B3 holds): keep the crew's CO₂ for the crop — store the scrubbed CO₂ and
   release it in lit hours, and/or a plant-aware target. Cited designs only.
2. **The food loop**: the harvest feeds the crew (turn on and re-measure `Harvest`), inedible biomass
   to compost, urine and waste to the plants' nutrients. Then the crew's food store becomes a buffer,
   not the diet.
3. **Room design at scale**: room air per m², soil depth vs hydroponics, chamber geometry — each a
   sourced choice for the user.
4. **Rooms as connectable units**: several farm rooms, a crew habitat, a wastewater marsh, linked by
   air and water — the shape a station "bigger than Biosphere 2" needs. Run speed is measured here.
5. **Sunlight through glass**, then solar and fission power, as choices.
6. **Adopt** the big station in one unfreeze (with the 98 % recycler); the 1 m² station stays as a
   test rig if anything still needs it.
