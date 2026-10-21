# Post-roadmap — the big station (opened 2026-10-21)

**Status: Step 1 MEASURED (2026-10-21, §3a).** The 200 m² station is CO₂-starved, as predicted; its
crop grows half as much per m², and CO₂ is the whole cause (measured). One inert plumbing change landed
for it (§3b); nothing else changes the reference. Step 2 is a choice for the user (§4).

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

## 3a. Step 1 — MEASURED (2026-10-21)

The lab: `rust/crates/station/examples/big_station.rs` (1 m² and 200 m² side by side, 1220 days, four
crew). Everything sized per m² of crop is ×a: crop and soil starts, the plants' tank (it starts empty
at both sizes: the reference's tank starts at 0, so the advisor's feared tank-vs-store mismatch does
not arise), the lamp, the chamber's heat capacity, cooler and heater, its walls by `chamber.yaml`'s box
rule (`2a + 4·0.78·√a`: 444 m² at 200 m², not ×200's 1 024). The room's air (9 500 mol), its CO₂ and
O₂ targets, the crew and their stores are the reference's. **Control:** the sizing at a = 1, through the
reference driver, reproduces `sealed_station_state.json` byte for byte (`--control`).

**Two stand-ins, neither bound.** Outside electricity is a lab flow that tops the battery back up each
minute from `boundary.grid`; heat rejection is the radiator and the structure's heat capacity ×a. The
structure ran at 100–129 K (1 m²: 162–176 K), so the cooler was **never** blocked by a warmer
structure; cooler-at-cap and heater-at-cap minutes are the same at both sizes (~1 770 and ~2 030 — the
setpoint's jumps at each re-sow and cold-period end), and the chamber held 4.00–22.07 °C (1 m²:
4.00–22.06). Rationed: 0 and 0 at both sizes — so the starvation below is the crop's kinetics, not the
arbitration backstop.

**The predictions:**

| | Predicted | Measured | |
|---|---|---|---|
| B1 | crew 98.6 mol CO₂/day | 98.62 mol/day | held (by construction) |
| B2 | an unstarved crop would fix ~70 mol C/day | did not occur — the crop is starved (B3) | — |
| B3 | CO₂-starved; closure share 20–50 %; crop/m² below 1 m² | starved: room CO₂ falls to **158 ppm** in lit hours (1 m²: never below 1 290). Grain 9.6–11.6 mol C/m² per crop (1 m²: 22.4), peak crop C 21.6–26.4 (48.2) — **43–55 %**. Closure share: crop 1 **36.7 %**, then 19.2, 9.9, 6.0, 4.4, 3.6, 3.2, 2.9 %; whole run **8.4 %** | starvation and crop held; the share held for crop 1 only — **missed** over the run (below) |
| B4 | O₂ never climbs without bound | 20.96–21.01 % | held |
| B5 | ~300 kg/day through the plants (~42 t per crop); the crew store does not run dry | **466 kg/day** (2.33 kg/m²/day — the same per m² as the 1 m² crop's 2.33, though it grows half as much); ~65 t per crop. Crew store 20 → 16.2 t (1 m²: → 17.8 t — the reference itself loses ~2.2 t to brine over 1220 days); reserve top-ups 1.5 t in all | store held; throughput **missed, 1.55× low** |
| B6 | lamp ~640 kWh/day; the room holds its setpoint | lamp 640 kWh/day in the warm phase (40 kW × 16 h, by construction); **430 kWh/day** averaged over the run (the cold period's dim lamp, 56 days of each crop); setpoint held | held |

**Why the closure share falls (measured, not asserted).** The lab books every crop cycle's carbon: the
ecosystem's net draw = the crop's change + the soil's change (litter, microbes, humus, consumers, the
litter sink), with a residual of 0.000 mol in every cycle. At each re-sow the old crop goes into the
soil (crop −4 372, soil +4 073 at the first), and the soil breathes it back out: soil carbon climbs
1 300 → 7 482 mol, its gain per crop falling 2 099 → 366. With the harvest off, nothing leaves, so in
the long run everything the crop fixes comes back and the net draw tends to zero — at 1 m² the same
(0.4 % → 0.0 %). **The share of the crew's CO₂ the crop holds at its peak is 31–38 % per crop and does
not fall** — that is the crop's capacity; turning it into closure needs the harvest to leave the room
(the food loop, §4 item 2).

**Is it the CO₂? (the attribution check, `--crew 800`).** 200 m² with 800 crew — every m² gets the 1 m²
run's CO₂ — over the first crop: grain **24.1** mol C/m², peak crop C **52.7** — back at (above) the 1 m²
run's 22.4 and 48.2. The room's CO₂ ran 2 733–3 883 ppm (the 9 500 mol room's scrubber cannot keep 800
crew at its target), hence the overshoot. So the half-sized crop at four crew is the CO₂ and nothing else
the sizing moved.

**An engine limit met on the way.** The first stand-in for outside electricity was a battery of 1e14 J.
Both runs (1 m² too) failed the every-step conservation check on day one: the check allows 1e-9 of the
step's largest transfer (`simcore::conservation`, `tol = atol + rtol·scale`), and a stock that large
rounds by ~0.016 J. The same bound applies to every stock that keeps a running total, the boundaries
included: at 200 m² the heat sent to space reaches ~1.9e12 J over 1220 days (ULP ~2e-4 J against a
minute's tolerance of ~1e-3 J) — inside the bound for this run, near it for a bigger or longer one. A
station "bigger than Biosphere 2" running for decades will meet it; not a change for this step, a
finding for Step 4.

**The crew's water store drains in the reference too** (20 → 17.8 t at 1 m² over 1220 days): the
recycler's losses. The approved 98 % recycler is folded into the adoption (§1).

## 3b. The plumbing Step 1 needed (landed, inert)

The sealed build read `chamber.yaml` inside itself at seven places, and its scrubber and O₂ makeup took
their targets on a `const` room size beside a second copy in the scenario, so no lab could size the
chamber. Now `SealedStationScenario::chamber` carries the chamber's coefficients (defaulting to the
file's) and the regulators read the scenario's `chamber_air_capacity_mol`. All 20 goldens identical;
`tests/chamber_sizing.rs` checks that a distinctive value reaches every reader (each reverted reader
turns a test red — measured). The user's rule from this step: parameters are inputs, not files re-read
inside a build (`CLAUDE.md`, invariants).

**Step 2 is the user's choice** (§4). What Step 1 says about the order: the crop can hold a third of
the crew's carbon but is starved to half its growth, and what it fixes comes back because nothing
leaves. Item 1 (CO₂ to the plants) and item 2 (the food loop) are both needed for closure; neither
alone reaches it.

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
