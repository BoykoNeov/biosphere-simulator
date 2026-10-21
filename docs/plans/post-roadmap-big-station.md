# Post-roadmap — the big station (opened 2026-10-21)

**Status: Step 1 MEASURED (2026-10-21, §3a).** The 200 m² station is CO₂-starved, as predicted; its
crop grows half as much per m², and CO₂ is the whole cause (measured). One inert plumbing change landed
for it (§3b); nothing else changes the reference. **Step 2a MEASURED (§5a):** the crew's CO₂ stored and
injected in lit hours (after CEEF) brings the crop back to 99.4 % per m². **Step 2b MEASURED (§6a):** the
harvest feeds 30 % of the crew; the O₂ the station still buys is exactly the CO₂ left in the tank. **Step 2c
MEASURED (§7a):** nitrogen goes round through grain, crew and urine; the grain gets 86 % of
USDA's nitrogen; the books close, but the 20 t soil hides the loop. **Step 3a MEASURED (§8a):** with a
soil of real size (grams of nitrogen per m², after Soltani & Sinclair Ch. 18) the crop is never short;
the imported food's nitrogen fills the soil solution to ~5 300 mg/L at drained-full in four years (~7 800
at the driest end of a watering cycle), and of [F]'s three losses only ammonia from urine can run here,
taking at most a tenth of it. **Step 3c PLANNED (§9, the user's choice, ahead of 3b):** a 700 m² farm
that feeds the crew, imported food kept as a reserve eaten only when the harvest runs short, and urine
stored in a tank and dosed to the soil as the roots take it (EcoSanRes' timing window); predictions
F1–F6 written, nothing built.

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

## 5. Step 2a — the crew's CO₂ kept for the plants (the user's choice, 2026-10-21)

The user chose §4 items 1 then 2: *"Send the crew's CO₂ to the plants. Store what the air scrubber
removes and release it when the lights are on, using published designs. This fixes the half-size
crop."* — then the food loop. Both are built in the lab (`examples/big_station.rs`) first, item 1
measured and recorded before item 2 starts (advisor, 2026-10-21). Nothing in the reference changes.

**The published designs.**
- **CEEF, Rokkasho (Japan), 2005–07** — two crew, two goats and 150 m² of crops in sealed modules:
  *"Increased carbon dioxide (CO2) in the AHM [crew] atmosphere by respiration of crew and goats was
  separated and supplied back to the air in the PCM [plants]"*. Fig. 1, read off the rendered page:
  the CO₂ line runs separator → reservoir → reservoir → mass-flow controller → plant module; the
  waste processors' CO₂ joins the same reservoir. Tako, Y., Masuda, T., Tsuga, S. et al., *Major
  achievements of material circulation — closed habitation experiments using Closed Ecology
  Experiment Facilities*, JAXA Space Utilization Symposium (JAXA repository SA6000061006, 2017).
  A companion SAE paper (2006-01-2076, abstract only) says the recovered CO₂ held the growth
  chambers' CO₂ *"at a constant level during light periods"*. This is the design built.
- **NASA's Biomass Production Chamber** (contrast only): CO₂ held at 1000–1200 ppm in the light
  and *"allowed to accumulate from plant respiration during the dark cycles"* — the room as its
  own store. With crew in the same air that raises their CO₂ at night, so it is not built. Wheeler,
  R.M., *NASA's Controlled Environment Agriculture Testing for Space Habitats* (NTRS 20140017323).
  The same paper's sizing, for the readouts: *"about 20-25 m2 of crops could supply the O2 for one
  human, while about 50 m2 would be required for food"*.

**The build (lab).** The scrubber's outlet becomes a CO₂ store (a tank, starting empty, no size
limit on the first run so that its peak says what tank is needed) instead of `boundary.co2_removed`;
an injector moves CO₂ from the store into the room while the lamp is on, `k · (target − room CO₂)`,
no more than the store holds. Target and gain are the scrubber's own (0.13 kPa; 1e-2 /s), so the
scrubber acts only above the target and the injector only below it. The gain is DESIGN, like the
scrubber's: CEEF's mass-flow controller is not published as a rate.

**Predictions (written before the run; 200 m², four crew, 1220 days, harvest still off):**
- **C1 — "this fixes the half-size crop" (the user's claim, tested).** The crew's 98.6 mol/day
  exceeds what an unstarved crop fixes on average (~70 mol C/day, B2), and the cold period (56 days
  of a seedling under a dim lamp) banks ~5 000 mol before the crop's peak asks for more than the
  crew breathes out. So the store does not run dry: grain and peak crop per m² come back to ≥ 90 %
  of the 1 m² crop's (≥ 20 and ≥ 43 mol C/m²), every crop.
- **C2.** In lit hours the room holds near the target (no lit minute below 1 000 ppm, against
  158 ppm in Step 1) except while the store is empty, which C1 says does not happen.
- **C3 — the store keeps growing.** Nothing leaves (vented 0) and the harvest is still off, so
  the soil breathes each crop back (§3a) and the crew's carbon, imported from the food store,
  piles up in the tank: tens of thousands of mol by day 1220 (~100 mol/day less what the soil keeps).
  This is not closure — closure needs the food loop (item 2).
- **C4 — oxygen.** The crop's net O₂ over the run tends to zero for the same reason, so the O₂ tank
  supplies roughly the crew's whole need, as in Step 1 (within 10 %).

## 5a. Step 2a — MEASURED (2026-10-21)

`examples/big_station.rs -- --co2-store` (200 m², four crew, 1220 days; the 1 m² reference beside it
as the yardstick). Rationed 0 and 0. `--control` still reproduces `sealed_station_state.json` byte
for byte. The readouts now book the crew's exhalation from the crew's own flow, not from the food
store's fall (which a harvest would refill), and count the store's change as not a draw.

| | Predicted | Measured | |
|---|---|---|---|
| C1 | crop per m² back to ≥ 90 % of 1 m²'s; the store never runs dry | grain **22.30–22.32** mol C/m² per crop (1 m²: 22.44), peak crop **47.96–47.99** (48.24) — **99.4 %**, every crop; lit minutes the store ran short: **0** | **held — the half-size crop is fixed** |
| C2 | no lit minute below 1 000 ppm | lit-hour room CO₂ **1 255–1 596 ppm, mean 1 284** (Step 1: down to 158); 0 minutes below 1 000 | held |
| C3 | nothing leaves; the store piles up tens of thousands of mol | vented **0**; store **101 948 mol** at its peak (4.5 t of CO₂ — ~6 m³ as liquid), growing ~13 000 mol per crop from the third crop on; 84.3 % of everything the crew breathed out is in the tank at the end | held |
| C4 | O₂ tank supplies the crew's whole need within 10 % | the tank gave **101 457 mol** against 120 216 at 1 m² — **84 %** | **missed**: the soil keeps carbon (below) |

**What the crop does with the CO₂.** The share of the crew's carbon the crop holds at its peak rose
from 31–38 % (Step 1) to **69.5 %**, every crop; on its best days the crop drew twice the crew's
output (best day 200.8 % in crop 1, 130 % later — the store paying out). Closure share over the
run **15.7 %** (Step 1: 8.4 %), and the cycle books still close to 0.000 mol.

**Why C4 missed.** I predicted the crop's net draw would tend to zero, as at 1 m². It does — the crop
change per cycle is ±3 mol after crop 1 — but the **soil** keeps a share of each crop: soil carbon
2 765 → 14 306 mol, gaining ~700–900 mol per crop by the end. That carbon is O₂ the crew did not
have to buy: the O₂ tank's saving (16 %) equals the closure share (15.7 %). The soil's gain is
falling crop to crop, so this saving shrinks toward zero over decades without a harvest.

**Water through the plants is unchanged by the crop's size:** 465 kg/day, 2.33 kg/m²/day, the same
as at half the crop (Step 1) and at 1 m². So transpiration here is set by the lamp and the air, not
by the leaf mass — B5's miss (1.55× low) was not about growth.

**What it leaves.** The CO₂ loop is closed in the sense that nothing is thrown away, but the carbon
ends in a tank, not in food: the crop is re-sown into the soil and the crew still eats the imported
store. That is item 2 (the food loop, §6).

## 6. Step 2b — the food loop (lab, on top of the CO₂ store)

**The build (lab, `--food-loop`, which turns the CO₂ store on too).**
- **A batch harvest at maturity.** When the crop matures, the grain minus the next crop's seed goes
  to the crew's food store, then the usual re-sow runs. Not the existing `Harvest` flow: its own
  param file calls its continuous draw (τ ≈ 1.16 days) the wrong form, and draining the grain would
  leave the re-sow no seed. Real crops are harvested once, at maturity (BVAD Table 4-89: wheat 75–90
  days after planting; NASA's BPC wheat *"harvested at physiological maturity"*, Wheeler).
- **Inedible parts to the soil** — straw, leaves and roots. Already what the re-sow does
  (`reset_crop` puts everything but the seed into the litter); recorded, not rebuilt. The soil's
  decomposers are the compost; CEEF burned its inedible parts and feces and recovered the CO₂
  instead, a faster path to the same tank.
- **Feces to the soil** (`close_feces`, an existing switch: the crew's fecal carbon goes into the
  litter instead of a waste boundary).
- **Urine and nitrogen: a gap, not built.** The crew have no nitrogen books and the grain carries
  none, so harvesting removes no nitrogen and urine has none to give back. (Corrected after the run:
  the sealed station adds no fertilizer — its rate is 0; the crop's nitrogen comes from a soil pool of
  100 kg N per m², recorded as non-physical in `post-roadmap-nitrogen-cycle-form.md` — real soils hold
  ~0.3.) Closing nitrogen needs nitrogen in the food and the crew first.

**Predictions (written before the run; 200 m², four crew, 1220 days, store on):**
- **D1 — food share ~30 %.** Grain 22.3 mol C/m² less ~0.16 of seed, × 200 m² ≈ 4 430 mol per
  140-day crop ≈ 32 mol C/day, against the crew's 103.9 (98.6 ÷ 0.949). So the crop feeds **~30 %** of
  the crew and the food store falls ~72 mol/day instead of 104. A sizing limit, not a bug: this soil
  wheat under this lamp grows ~22 mol C of grain per m² per 140 days; NASA's estimate is ~50 m² of
  crops per person for food (Wheeler), CEEF fed 2 crew ~90 % from 150 m² of 23 crops.
- **D2 — closure share rises to ~45 %** (2a: 15.7 %): the harvested grain is carbon the plants fixed
  and the crew eat instead of imported food; the O₂ tank's supply falls with it, to **~55 %** of the
  1 m² run's.
- **D3 — the tank grows slower**, ~8 000–9 500 mol per crop instead of ~13 000: the imported food
  falls by the harvest.
- **D4 — the crop is unchanged** (grain and peak crop per m² within 1 % of 2a's): the seed is kept
  and the soil still gets the straw and roots.

## 6a. Step 2b — MEASURED (2026-10-21)

`examples/big_station.rs -- --food-loop` (200 m², four crew, 1220 days, the CO₂ store on). Rationed 0
and 0. Each cycle's carbon books — crop, soil, harvest leaving, feces arriving — close to 0.000 mol.

| | Predicted | Measured | |
|---|---|---|---|
| D1 | the crop feeds ~30 % of the crew; the food store falls ~72 mol/day | **30.4 %** of the crew's food per crop (4 429 mol C per 140-day crop, 22.15 mol C/m²); **28.0 %** over the run (the last, unfinished crop is not harvested); the store falls **74.9** mol/day (104 without the loop) | held |
| D2 | closure share ~45 %; the O₂ tank ~55 % of the 1 m² run's | closure share **36.4 %** over the run, settling to ~30 % per crop; the O₂ tank gave **76 475 mol — 64 %** of the 1 m² run's 120 216 | **missed** (below) |
| D3 | the CO₂ tank grows ~8 000–9 500 mol per crop | 7 052, 8 841, 9 338, 9 485, then **9 550, 9 594, 9 630** — still rising; peak **77 403 mol** (3.4 t CO₂; 2a: 101 948) | inside the range for crops 2–4, **above it** from crop 5, still rising |
| D4 | the crop is unchanged | grain 22.30–22.31, peak crop 47.96–47.98 mol C/m² (2a: 22.30–22.32, 47.96–47.99) | held |

**Why D2 missed.** I added the harvest's share to Step 2a's 15.7 %, but the two are not additive: in
2a, most of the closure was the **soil** keeping part of the grain it was given. Eaten grain is grain
the soil never gets, so the soil now gains less — soil carbon ends at 10 320 mol (2a: 14 306), its
gain per crop falling 3 068 → 488. What is left is the harvest itself: 4 429 mol per crop is 32 % of
the crew's 13 806 mol of exhaled CO₂, and the closure share settles there.

**The O₂ the station buys is the CO₂ in the tank.** The O₂ tank's supply over the run (76 474.6 mol)
equals the CO₂ store's final contents (76 475.4 mol) to within 1 mol: every CO₂ molecule the plants
do not take back holds the O₂ the crew breathed into it. So the remaining open part of the loop is
one number — the tank's growth, ~68 mol/day (= the imported food) — and closing it needs either more
food from the plants (more crop, more light, more productive crops) or turning that CO₂ back into O₂
(a CO₂ reduction system, as ISS's Sabatier path does, or more plants than food needs).

**Sizing, against the real world.** NASA's estimate is ~20–25 m² of crops per person for O₂ and ~50 m²
for food (Wheeler, NTRS 20140017323); CEEF fed two crew ~90 % from 150 m² of 23 crops. Here 200 m²
of this soil wheat feeds 1.2 crew-equivalents (30 % of four) — ~165 m² per person. Whether to close
the gap with area, light, crops or CO₂ reduction is the user's choice; nothing here was tuned.

**Nitrogen.** The inedible parts already return their nitrogen: the re-sow moves the old plants'
nitrogen into the soil's litter, which the soil mineralizes back. The grain, the crew and urine carry
none, so the harvest takes no nitrogen out. No fertilizer is added (the sealed station's rate is 0):
the crop draws on a soil pool of **100 kg N per m²** (20 t at 200 m²), already recorded as non-physical
(`post-roadmap-nitrogen-cycle-form.md`; real soils ~0.3 kg N/m² in all). Closing it means nitrogen in
the grain and food, crew nitrogen books, and urine back to the soil — and a soil small enough for the
loop to matter (§7).

**Water through the plants** is unchanged (465 kg/day, 2.33 kg/m²/day): the crop is the same crop.

## 7. Step 2c — nitrogen through the food (approved by the user, 2026-10-21)

The user, on hearing the food loop closes carbon but not nitrogen: *"the food should have a nitrogen
content. as well as other elements and microelements, that is the final goal - fullest cycle
simulation of energy, elements, nutrients"*; then *"ok i agree"* to nitrogen first. Phosphorus,
potassium and trace elements are a later choice: each is a new quantity in the frozen core.

**What the model has (read 2026-10-21).** Nitrogen already cycles inside the soil and plants: the
plant's one nitrogen pool (`plant_n`, held against the leaves', stems' and roots' carbon — not the
grain's), shedding to litter, the decomposers mineralizing it back to `soil_n` (the plant-available
pool). Nothing leaves: no leaching, denitrification or volatilization, and the sealed station adds no
fertilizer (rate 0). The soil starts at 100 kg N per m² — non-physical, recorded; real soils hold
grams of available nitrogen per m². The crew and the grain carry no nitrogen.

**The build (lab, `--nitrogen`, on top of `--food-loop`).**
- **Food nitrogen**, a stock beside the food store. The imported food carries BVAD's menu nitrogen:
  Table 4-61 (resupplied menus at 12.71 MJ/CM-d) gives 122–125 g protein per crew-day; 122 g ÷ 6.25 =
  **19.5 g N per crew-day**, over the model's own 25.97 mol C per crew-day. (BVAD's caveat: the table
  compares menus and is not absolute values.)
- **Grain nitrogen at harvest**: wheat grain is **2.49 % N of dry mass** — USDA FoodData Central
  168890, *Wheat, hard red winter*: 12.61 g protein and 13.1 g water per 100 g, protein = N × 5.83.
  Taken from the plant's nitrogen, but never below what the straw keeps: the cited residual 0.5 % of
  its dry mass (`nitrogen.yaml` `n_residual`) plus the seed's share. The model's nitrogen target leaves
  the grain out, so this cap may bind: a finding if it does, not a thing to tune.
- **The crew** eat nitrogen with their carbon at the food store's average N:C and pass it all (an adult
  is in balance): **88 % in urine** to the soil's available nitrogen, **12 % in feces** with the fecal
  carbon to the litter — Jönsson & Vinnerås 2004, as tabulated in EcoSanRes Fact Sheet 6 (2005):
  urine 85–90 % of excreted N in every country listed. Contrast: BVAD Table 4-34's feces protein
  (4.5 g/CM-d ≈ 0.7 g N) would be ~4 %. The urine's water still goes to water recovery; separating its
  nitrogen to the soil is a simplification (EcoSanRes applies the urine itself).

**Predictions (written before the run; 200 m², four crew, 1220 days, store and food loop on):**
- **N1.** The crew eat **78.1 g N/day** (4 × 19.5) while the store is all imported food.
- **N2 — the cap binds.** Unrestricted, the grain would take 4 429 mol C × 0.0267 kg DM/mol C ×
  2.49 % = **2.94 kg N per crop**. The plant holds its nitrogen against its vegetative carbon only, so
  at maturity I expect it to hold less than grain + straw residual: the grain gets **50–70 %** of
  USDA's nitrogen (1.5–2.0 kg per crop).
- **N3.** So the harvest covers **15–20 %** of the crew's nitrogen (27 % uncapped), below its 30 %
  share of their carbon.
- **N4 — nitrogen piles up in the soil**, as CO₂ in the tank: the imported food's nitrogen arrives
  in urine and feces and nothing removes it — ~60–65 g/day, **~75 kg over the run**, against a soil
  pool of 20 t. The crop cannot feel it.
- **N5.** Carbon: grain and peak crop within 1 % of Step 2b's (the soil's nitrogen is ~10⁴× what the
  crop needs); the nitrogen books close to 0 every crop.

## 7a. Step 2c — MEASURED (2026-10-21)

`examples/big_station.rs -- --nitrogen` (200 m², four crew, 1220 days, store and food loop on).
Rationed 0 and 0. Each cycle's nitrogen books — plant, soil (mineral, litter, microbes, humus) and
food — sum to 0 within 3·10⁻⁷ kg. Run 2026-10-21, ~13 min.

| | Predicted | Measured | |
|---|---|---|---|
| N1 | the crew eat 78.1 g N/day | **78.1** in crop 1, falling to **76.7** by the last: the store's mix now holds grain, with less nitrogen per carbon than the menu (0.57 against 0.75 g N per mol C) | held |
| N2 | the cap binds; the grain gets 50–70 % of USDA's 2.94 kg N per crop | the cap binds, but at **85.8 %**: 2.52 kg of 2.94, every crop. The plant holds **3.21 kg N** at maturity (≈2.3 % of its leaf, stem and root dry mass); the straw keeps its 0.5 % (0.69 kg) and the grain gets the rest | **missed** (high) |
| N3 | the harvest covers 15–20 % of the crew's nitrogen | **23 %** per crop (2.52 of ~10.9 kg); **21 %** over the run (20.2 of 94.4 kg) | **missed** (follows N2) |
| N4 | ~75 kg of nitrogen piles up in the soil | **71.0 kg** into the soil (8.2–8.4 kg per crop), 3.2 kg in the growing crop at the end, the food store down 74.2 kg; 20 t of soil nitrogen does not notice it | held |
| N5 | carbon within 1 % of 2b; nitrogen books close | carbon **identical** to 2b to the printed digit (harvests, CO₂ store 76 475.4 mol, O₂ bought 76 474.6); nitrogen books close every crop | held |

**Why N2 missed.** I guessed the plant would hold its nitrogen thinly because its target leaves the
grain out. It does not: at maturity the leaves, stems and roots hold ~2.3 % N, so the straw can give
up most of it and still keep its cited 0.5 %. What binds is that residual, not a shortage. Here 2.52
of 3.21 kg is 79 % of the plant's nitrogen in the grain, roots included. Field wheat: 57–83 % of the
plant's nitrogen in the grain across 15 durum cultivars (Desai & Bhatia 1978, *Euphytica* 27:561–566,
doi 10.1007/BF00043182 — abstract only, read via a search summary of the Indian Academy of
Sciences repository record; full text not read), and roots hold 10–20 % of a spring wheat's nitrogen
at maturity (Andersson & Johansson 2006, *J. Agron. Crop Sci.* 192:121–131, abstract). The model's
79 % counts the roots; most field figures count only the plant above ground, and on that basis the
model's share would be higher — at or above the top of the field range. It follows from the straw
keeping only its cited 0.5 %; not tuned.

**What the books show, and what they cannot.** Nitrogen now goes round the whole loop: soil → plant →
grain → food → crew → urine (88 %) and feces (12 %) → soil, and the inedible parts → litter → soil.
Nothing is lost, so the imported food's nitrogen piles up in the soil, as its carbon piles up as CO₂
in the tank. But the soil starts at 100 kg N per m² (20 t; non-physical, recorded): the 71 kg moved is
0.35 % of it, so the crop cannot feel the loop. Making nitrogen limit the crop needs a soil of real
size (grams of available N per m²) and real loss paths (leaching, denitrification, ammonia from urine)
— each a sourced choice, not made here. Phosphorus, potassium and trace elements each need a new
quantity in the frozen core.

## 8. Step 3 — a soil of real size, and its nitrogen losses (the user's choice, 2026-10-21)

The user chose "A" from the list after Step 2c: a soil holding grams of available nitrogen per m², and
the real ways nitrogen leaves a soil. Lab only (`big_station.rs`), as 2a–2c.

**The source: [F] Chapter 18, "Soil Nitrogen Balance"** (Soltani & Sinclair, *Modeling Physiology of
Crop Development, Growth and Yield*, pp. 239–252; PDF in `sources\`) — the same book the model's soil
water already follows (Ch. 13–14). It gives the soil's starting nitrogen from routine soil data (Eqns
18.19–18.24, with a worked example), what the roots can reach (Eqn 18.5), and three losses:
ammonia gas (18.13, Table 18.1), leaching with drained water (18.14) and denitrification (18.16–18.18).
Its mineralization (Stanford & Smith, 18.6–18.11) is **not** taken: the model already mineralizes
through its litter → microbes → humus chain, and two forms for one process would double-count.

**Read before designing (2026-10-21).**
- *The surplus.* The crew's urine brings ~0.34 g N per m² per day to the soil (88 % of 78 g/day over
  200 m²); the crop takes ~0.115 (3.2 kg per 140-day crop). Most of the crew's nitrogen comes from the
  imported food, so a soil of real size will **fill**, not starve: Step 2c put 71 kg into the soil,
  ~350 g N/m², against [F]'s example soil of 3.9 g N/m² in solution. The question is whether the
  losses keep up, not whether the crop runs short.
- *Which losses can run here.* A 300-day run (200 m², `--nitrogen`): the root zone **never drains** —
  `biosphere.drainage` moves 0 kg; the station waters up to the drained-full level and no higher. The
  only water leaving the soil is the overflow from below the roots, 39.7 kg/day, into the plants' tank,
  which waters the plants again. So, by [F]'s own rules: **leaching** (from the top layer, with its
  drainage) is 0, and what leaves below the roots comes back with the watering — a loop, not a loss;
  **denitrification** needs the top layer wetter than drained (FTSW1 > 1), which never happens. Neither
  is built. The one loss that runs is **ammonia from urine**.
- *Where the gas goes.* The room's air has no nitrogen stock. Ammonia in a station is removed by the
  air's trace-contaminant control; a loss to a boundary is an open leak in the books, and is shown as one.

**Step 3a — the soil of real size, no losses (`--real-soil`, on top of `--nitrogen`).**
- *The soil*: [F] Table 13.1's **silt loam** (bulk density 1.37 g/cm³; water at the lower limit 0.086,
  drained upper limit 0.218, extractable 0.132 m³/m³ — the model's 0.13 is the same soil). Nitrogen in
  the model's reference layer, 0.30 m (`soil_layer_depth`): 411 kg of soil per m² (Eqn 18.19). Solution
  nitrogen at [F]'s worked-example concentrations, 15 mg/kg nitrate and 1.8 mg/kg ammonium ([F]: "little
  error results from general estimates"): **1.97 g N/m²** (Eqns 18.22–18.23), **0.39 kg** at 200 m²,
  replacing 20 t.
- *What the roots can reach*, replacing the availability band (1 and 50 kg/m², design values sized for
  the 100 kg soil, which would switch uptake off at grams): Eqn 18.5, the nitrogen dissolved in the
  layer's plant-available water, `(NCON − 1 mg/L) × ATSW1 × FROOT1`, with NCON the layer's nitrogen over
  all its water (lower limit + available, Eqn 18.24). Uptake per day is capped at it ([F] Ch. 17:
  "if NUP is calculated to be greater than SNAVL, it is adjusted at SNAVL"), applied per step as a
  daily amount × dt. FROOT1 is the model's existing root-zone fraction (Eqn 18.4, already built). The
  model's one root-zone water store has no layers: the 0.30 m layer is taken to be as wet as the root
  zone (recorded; [F] notes its own top-layer choice "is not required").
- *Left out, on purpose*: the soil's organic matter. A silt loam holds 1.07 % carbon and 0.110 %
  nitrogen ([F] Table 18.2): ~450 g of organic N and ~4.4 kg of carbon per m² in the 0.30 m layer — at
  200 m², ~73 000 mol C, which breathes out CO₂ as it decays (in a sealed room that is Biosphere 2's
  story). Adding it is its own choice, with its own carbon prediction; 3a keeps the carbon books equal
  to 2c's so that the nitrogen change is read alone.

**Predictions (written before the run; 200 m², four crew, 1220 days, store, food loop and nitrogen on):**
- **R1 — the soil fills.** Solution nitrogen rises by ~0.25–0.30 g/m² per day after the first weeks
  (Step 2c's ~60 g/day into the soil, less what sits in litter): ~35–45 g/m² by the first harvest,
  **~250–350 g/m² by day 1220**. As a concentration in the layer's water at drained-full (65 mm),
  ~30 mg/L at the start, **~600 mg/L by the first harvest and thousands by the end** — past both
  reference points [F] uses (200 mg/L stops its mineralization; 400 mg/L caps its denitrification).
  The model has no salt damage, so the crop cannot show what that would do.
- **R2 — the crop is not short of nitrogen, except perhaps in the first crop's first weeks.** At sowing
  the roots reach ~0.6 g N/m² per day (half the layer, at 30 mg/L), above the crop's average need.
  Grain and peak crop within 1 % of 2c; plant nitrogen at maturity 3.21 kg ± 1 % from the second crop.
- **R3.** Carbon identical to 2c to the printed digit, unless R2's early weeks bind (then within 1 %).
  Nitrogen books close every crop.

**Step 3b — ammonia from urine (after 3a, measured).** [F] Eqn 18.13: a share VOLF of each application
leaves as ammonia. Urine is mostly urea (the first constituent of BVAD's urine ersatz; BVAD: urine was
vented on Shuttle "to avoid the breakdown of urea to ammonia"), so Table 18.1's **urea** row; the
chamber's 75 % humidity is its **humid** column. Read off the page (p. 245, Delgado
et al. 2010 after Meisinger & Randall 1991), urea, humid: **10 % surface-applied, 2 % incorporated**
(sub-humid 15 / 3, dry 25 / 5). EcoSanRes advises incorporating urine "to avoid … the loss of ammonia".
Which one is the user's choice, and so is where the ammonia goes (removed from the air = lost; or
scrubbed back into the plants' water).

## 8a. Step 3a — MEASURED (2026-10-21)

`examples/big_station.rs -- --real-soil` (200 m², four crew, 1220 days, store, food loop and nitrogen
on). Rationed 0 and 0. Nitrogen books close every crop (within 5·10⁻⁹ kg). Run 2026-10-21, ~13 min.

**The cap is shown to fire.** A throwaway run with the floor raised from 1 to 1 000 mg/L (250 days,
not kept) starves the first crop — grain 0.39 mol C/m² instead of 22.3, the uptake capped in 1 328
plant steps, 22 kg of nitrogen held back — so R2's 0 counts a live cap, not a dead one. With the real
floor, the closest the cap came: the roots could reach at least **9.2×** what the crop wanted (first
crop), 24× in the second, 162× in the last. Outputs: `sources\temp-evidence\real-soil\` (`n300.txt` the
2c run §8's drainage figures come from, `rs1220b.txt` this run, `floor1e-3.txt` the throwaway).

| | Predicted | Measured | |
|---|---|---|---|
| R1 | solution N ~35–45 g/m² by the first harvest, ~250–350 g/m² by day 1220; ~600 mg/L by the first harvest, thousands by the end (both at drained-full, 65 mm) | **34.8 g/m²** at the first harvest (from 1.97), rising ~0.29 g/m² per day every crop after; **341.9 g/m²** (68.4 kg) at day 1220. At drained-full (64.8 kg of water per m²): **536 mg/L** at the first harvest, **5 276 mg/L** at the end. The highest at any day's end, at the dry end of a watering cycle (the same nitrogen in less water): 777 and 7 809 mg/L | held |
| R2 | not short of nitrogen, except perhaps in the first crop's first weeks | the roots' reach **never** capped the uptake — 0 plant steps in all nine cycles. The first crop spends its first 8 weeks in the cold, needing almost nothing, while urine fills the soil | held (the "perhaps" did not happen) |
| R3 | carbon identical to 2c to the printed digit | **identical**: harvests, CO₂ store 76 475.4 mol, O₂ bought 76 474.6, plant nitrogen at maturity 3.213 kg | held |

**What it shows.** With a soil of real size, the nitrogen the crew bring in from imported food has
nowhere to go. The crop takes about a third of what urine delivers, so the soil solution climbs past
every reference point [F] uses — 200 mg/L, where its mineralization stops, in the first crop; 400
mg/L, where its denitrification is capped, soon after — to ~5 300 mg/L at drained-full by the fourth
year (~7 800 at the dry end of a watering cycle). The model
has no salt damage, so the crop grows as if nothing were wrong; a real crop would not. The nitrogen
that 2c hid in a 20 t soil is now in plain sight.

**What 3b can and cannot do.** The one loss that runs here, ammonia from urine, takes 2 % or 10 % of
it (Table 18.1, humid; incorporated or surface-applied): 1.4 or 6.9 g N/day of the ~69 in urine. The
soil would still gain ~0.26–0.29 g/m² per day. [F]'s other two losses are closed by the station's own
watering (no drainage from the root zone, never waterlogged). So **none of [F] Ch. 18's three losses
balances this loop** under this station's watering (2026-10-21) — losses outside that chapter, such as
denitrification in soil that is not saturated, were not looked for. What would: less imported nitrogen (more of the crew's food grown here), nitrogen taken
out on purpose (urine kept back or its nitrogen stored, as the CO₂ is), or watering that drains — each
a design choice for the user, not a soil property.

## 9. Step 3c — a farm that feeds the crew, the diet's switch, and a urine store (the user's choice, 2026-10-21)

The user, on 3a's finding, chose this ahead of 3b (ammonia, §8): *"The solution is - a bigger "soil" -
bigger crop, a marsh with enough size. The crew initially starts eating imported food. nitrogen goes
into the soil (or to the marsh and then transported manually to the soil by the crew). the crew starts
eating grown food - nitrogen reaches e relative balans"*. A store for the surplus came from a side
note (the nitrogen already piled up never leaves a sealed station on its own; watering, rain and
scrubbing move it in a circle) and was approved: *"yes"*. Also approved, for later steps: watering and
artificial spray or rain (*"yes, absolutely"*), and a marsh that holds the wastewater. Lab only
(`big_station.rs`), on top of `--real-soil` so that a nitrogen shortage can show.

**Why the pile should shrink.** An adult keeps no nitrogen, so a crew eating grown food returns to the
soil what the crop took: the loop is closed. Only imported food brings new nitrogen. In 3a the crew ate
~70 % imported food for all 1 220 days (the crop fed 30 %), and the store's mix kept them on it: the
crew eat at the store's average, and a 5·10⁵ mol store (~13 years of food) dilutes each harvest.

**The build (lab, `--feed-crew`, on top of `--real-soil`).**
- **The farm: 700 m²**, the round area above what feeds four crew. 2b measured 22.15 mol C of grain
  per m² per 140-day crop and the crew eat 103.9 mol C/day, so 658 m² feeds them exactly; 700 m² gives
  ~107 % (15 500 against 14 550 mol C per crop). Everything else sized per m² scales as in §3a.
- **Grown food first, imported food as a reserve.** The harvest goes to the crew's food store, as in
  2b. The imported food becomes its own reserve stock (the reference's 5·10⁵ mol C, with its
  nitrogen at BVAD's menu ratio), moved into the food store **only when the store would run short** —
  carbon and nitrogen together, at the imported ratio, so the crew's average reads the mix truly. The
  crew start on the reserve: there is no harvest before day ~140.
- **Urine to a tank, not the soil.** The crew's urine nitrogen (88 %) goes to a urine store; feces still
  go to the litter (12 %). EcoSanRes Fact Sheet 6 (read in full): urine *"should be stored for between
  one and six months"* before use (hygiene; the store's age is not modelled — after the first month
  the tank always holds more than a month's urine, a readout).
- **Dosed to replace what the roots take, in the cited window.** Each step, while the tank holds it,
  the dose equals the roots' uptake that step (the soil's solution nitrogen held at its level), but
  only from sowing to two-thirds of the season: EcoSanRes, urine is *"best utilized if … applied prior
  to sowing and up until two-thirds of the period between sowing and harvest"*, *"the last application
  … at least one month prior to harvesting"*. Two-thirds of the 140-day cycle is day 93 after sowing,
  47 days before harvest. Outside the window the crop draws on the soil alone. This is "on demand"
  as approved; the guideline's own dose is a fixed rate (below), kept as the cross-check.
- **The cited cross-check and limit.** EcoSanRes' rule of thumb: *"apply the urine produced by one
  person during one day … to one square metre of land per growing season"*, and *"the maximum
  application rate before risking toxic effects is at least four times the dose above"*. Here one
  person-day of urine holds **17.2 g N** on the imported menu (19.5 × 0.88) and **~13.0 g N** on an
  all-wheat diet (2c's grain, 0.569 g N per mol C × 25.97 mol C per crew-day × 0.88). So the rule's dose
  is **13–17 g N/m² per crop**, its toxic limit **~52–69 g N/m² per crop**. This replaces 3a's bare
  mg/L as the readout for "too salty". (3a's urine landed ~0.34 g/m²/day, ~48 g/m² per crop — near that
  limit before counting what had piled up.)
- **Not built here:** ammonia (3b; incorporating the urine, as EcoSanRes advises, is the 2 % case),
  the marsh, draining or spray watering — each its own later step.

**Carbon decides the nitrogen (advisor, 2026-10-21).** A sealed station can hold only the carbon it is
given. At 700 m² a crop at its peak holds ~33 600 mol C (48 per m², §6a), and the soil gains carbon
every crop (2b: 3 068 falling to 488 mol per crop at 200 m², ~26 000 over the run at 700 m²). The crew
breathe out ~13 800 mol per crop. That extra carbon can come only from imported food the crew eat —
and each mol of it brings 0.75 g of nitrogen. So the imports do not stop when the first harvest could
cover the diet; they stop when the crop and the soil have filled with carbon.
**The alternative, named, not built:** CEEF bought CO₂ from outside (*"CO2 addition from external
source"*, 23.5 mol C/day in its 2007 run; Tako et al., Fig. 2b, read in full). That builds the farm's
carbon with no nitrogen attached — a lever on the pile's size, and the user's choice if this run shows
the imports are large.

**Predictions (written before the run; 700 m², four crew, 1220 days, CO₂ store, food loop, nitrogen
and real soil on):**
- **F1 — the first crops are short of carbon.** Before the first harvest only the crew's ~13 800 mol and
  the soil's breath feed the crop. Grain per m²: crop 1 **40–65 %** of 2b's 22.15 mol C/m², crop 2
  **70–95 %**, from crop 3 or 4 within **5 %**.
- **F2 — imports are the carbon the farm fills with.** Imported food eaten: all of the first ~140 days,
  then each crop's shortfall — **40 000–70 000 mol C** over the run (about 100–170 days of the whole
  crew's food), most of it before day ~420. From crop 4 on, imported food is **under 10 %** of the diet;
  over the run the crop feeds **65–85 %** (2b: 28 %).
- **F3 — the imported nitrogen: 30–53 kg** (F2 × 0.75 g/mol; 3a: ~74 kg eaten). Where it ends: the
  standing crop (up to ~11 kg at maturity), the soil's litter and humus (a few kg), the soil solution
  (held near its 1.4 kg start), and **the rest in the urine tank, 15–35 kg at day 1220**, flat or nearly
  so after the ramp. The tank holds the pile; the soil does not.
- **F4 — the soil stays a soil.** Solution nitrogen between **0.5× and 2×** its 1.97 g/m² start all run
  (3a: to 342 g/m²). Doses per crop **8–14 g N/m²** — the same size as EcoSanRes' person-day rule
  (13–17), and far under its toxic limit (52–69).
- **F5 — late-season nitrogen may bind.** With no dose in the last 47 days, the roots' reach may cap
  uptake in some steps during grain fill (3a: never). If it does, grain within 3 % of F1's carbon-limited
  value; the count is reported, not tuned.
- **F6.** Rationed 0 and 0; carbon and nitrogen books close every crop.

**Readouts.** Per crop: grain per m² and as a share of the crew's food; imported food eaten (mol C,
kg N); the urine tank's contents and inflow; doses per m² against the person-day rule and its 4×
limit; solution nitrogen min/max (g/m², and mg/L at drained-full); capped uptake steps; carbon in crop,
soil, food store and CO₂ tank. Over the run: where the imported nitrogen ended.

**The documented ways to send ammonia back to the crops** (asked by the user; for 3b, not built here).
Read in full: EcoSanRes (incorporate urine to avoid ammonia loss) and Tako (CEEF). The rest from
search summaries, **full text not read**:
- *Wet scrubbers.* Livestock barns spray slightly acidic water. Melse & Ogink (2005, *Trans. ASAE*): acid
  scrubbers removed 40–100 % of ammonia, 96 % on average; with sulfuric acid the result is an
  ammonium-sulfate solution used as fertilizer. A 2016 study of slightly acidic spray scrubbers
  (*Front. Environ. Sci. Eng.* 10(6)): the captured ammonium *"should be an excellent fertilizer"*.
- *Condensate.* The ISS removes ammonia partly by its dissolving in the humidity condensate (NASA
  NTRS 20180004945; ISS condensate 20–42 mg/L as N, National Academies). This model's chamber condensate
  already returns to the plants' water.
- *Leaves.* Above a small compensation point leaves take up ammonia from the air (Farquhar et al. 1980,
  *Plant Physiol.* 66:710–714).
- *Never forming it.* MELiSSA's nitrifying compartment turns urine ammonium into nitrate for the plants;
  Lunar Palace 1 recovered ~20.5 % of urine nitrogen in its 105-day run; Biosphere 2 sent its wastewater
  through a constructed marsh into the crop irrigation (Nelson et al. 1999, *Ecol. Eng.*).

## 9a. Step 3c — MEASURED (2026-10-21)

`examples/big_station.rs -- --feed-crew` (700 m², four crew, 1220 days; CO₂ store, food loop, nitrogen
and real soil on). Rationed 0 and 0; carbon books close to 0.000 mol and nitrogen books to ~1e-10 kg
every crop. `--control` still reproduces `sealed_station_state.json` byte for byte, and `--real-soil`
reproduces 3a's first crop to the printed digit (the refactor moved nothing). Outputs:
`sources\temp-evidence\feed-crew\` (`fc1220.txt` this run; `d160c.txt` its first 160 days with the
dose-rule counter; `twin700.txt` 160 days at 700 m² with urine straight to the soil, as 3a; `rs160.txt`
the 3a check; `rich700.txt` 160 days at 700 m² on the reference's 100 kg N/m² soil, never capped).

Per crop: harvest per m² (share of 2b's 22.15), the harvest's share of the crew's food, imported food
eaten, the soil solution at the crop's end:

| crop | harvest mol C/m² | of 2b | feeds the crew | imported mol C | soil solution g N/m² |
|---|---|---|---|---|---|
| 1 | 5.96 | 27 % | 28.7 % | 14 548 | 0.31 (lowest 0.06) |
| 2 | 8.46 | 38 % | 40.7 % | 10 378 | 3.92 |
| 3 | 10.60 | 48 % | 51.0 % | 8 626 | 9.31 |
| 4 | 12.45 | 56 % | 59.9 % | 7 128 | 15.45 |
| 5 | 14.20 | 64 % | 68.3 % | 5 837 | 21.19 |
| 6 | 15.66 | 71 % | 75.3 % | 4 609 | 25.99 |
| 7 | 16.74 | 76 % | 80.6 % | 3 587 | 29.88 |
| 8 | 17.48 | 79 % | 84.1 % | 2 829 | 32.93 |
| (9, day 100 of 140) | — | — | — | 0 | 34.77 |

| | Predicted | Measured | |
|---|---|---|---|
| F1 | crop 1 at 40–65 % of 2b, crop 2 at 70–95 %, within 5 % from crop 3 or 4 | **27 %, 38 %**, then a slow climb: **79 %** at crop 8, still rising | **missed** |
| F2 | imports 40 000–70 000 mol C, most before day ~420; under 10 % of the diet from crop 4; the crop feeds 65–85 % over the run | **57 541 mol C** (554 crew-days of food), 58 % of it in the first three crops; **49 %** of the diet at crop 4, 19 % at crop 8; the crop fed **56 %** over the run | total held; the timing **missed** |
| F3 | imported N 30–53 kg: standing crop ≤ ~11, litter and humus a few, solution near its 1.4 kg start, **15–35 kg in the tank** | **43.2 kg**: plant +11.3, litter/microbes/humus +7.4, food store +1.2, **soil solution +23.0** (1.38 → 24.34), **tank 0.41** | total held; where it went **missed** |
| F4 | solution 0.5×–2× its start all run; doses 8–14 g N/m² per crop, far under the 52–69 toxic limit | solution **0.03× to 17.6×** (0.06 to 34.8 g/m²); doses **9.2–13.3 g/m²**, under the limit | doses held; the soil **missed** |
| F5 | late-season nitrogen may bind; if so, grain within 3 % of the carbon-limited value | crop 1: capped in **621** plant steps (soil drawn to 0.06 g/m² after dosing stops at day 93); crop 2: 12; then **0**. Crop 1's grain is **identical** to the urine-to-soil twin and to a run on the reference's nitrogen-rich soil, which is never capped (6.117 mol C/m² all three): carbon, not nitrogen, set it | held |
| F6 | rationed 0 and 0; books close every crop | held | held |

**"Capped" is not "stressed" (a side agent asked, 2026-10-21, whether a nitrogen shortage can slow
this crop at all).** It can: 3a's floor test (§8a) cut crop 1's grain from 22.3 to 0.39 mol C/m². The
stress (`nitrogen_stress_factor`) scales photosynthesis only below the critical concentration, 4.0·10⁻⁴
kg N per mol C (~1.5 % of dry mass); the cap counter instead compares the uptake with what the roots
ask for, the higher dilution-curve target. Crop 1 ended at 7.687 kg N in ~11 500 mol C — **1.66× the
critical** — because a carbon-starved crop is small. The grain is identical across 7.7, 8.5 and 8.8 kg
of plant nitrogen, so the stress was 1 in all three runs: the shortage fell inside the range where the
model's crop does not feel it. A per-step stress readout was not added.

**Why F1 and F2 missed: the farm fills by a fixed share of what is missing, not all at once.** Each
crop's extra carbon is only the imported food the crew ate to cover the gap, and the soil keeps more of
it than the crop gains (1.1× the crop's gain in crop 2, 2.4× by crop 8; soil carbon +5 487 mol in crop 2, falling to +2 001 in crop 8; 30 146 by
day 1120). So the gap in the diet shrinks by about a fifth per crop — 71 → 59 → 49 → 40 → 32 → 25 → 19 %,
ratio 0.78–0.83 — a slow exponential, not a fill. I had budgeted the carbon right (the imports' total
held) but assumed the crop would take it first. At that ratio, ~95 % takes about six more crops
(~2.3 years). The crop is still short of CO₂: the tank ran dry in **19 %** of lit minutes (194 808 of
1 045 440), room CO₂ down to the 100 ppm floor, mean 1 059 ppm. The O₂ bought (10 893 mol) is again the
CO₂ left in the tank at the end (~10 950 mol, 9.1 % of the crew's breath), as in 2b.

**Why F3 and F4 missed: the dose rule ignores the soil's own supply.** The dose equals what the roots
take, but the soil solution also gains nitrogen from everything else (the "rest" column — all but doses
and uptake, consistent with the old crop's straw, shed leaves and feces breaking down: **+4 to +8 kg per
crop** after crop 1). The roots take most of their nitrogen inside the dosing window,
so the dose empties the tank every window (lowest 4 g); the soil's own release plus all the urine is
more than the crop takes, and the excess — the imported nitrogen — lands in the soil solution, not the
tank. The side agent (2026-10-21) warned that holding the soil by dosing would make F4 pass by
construction; it failed instead, through exactly the term it said to watch. The soil's gain per crop
follows the imports down (+3.8, +4.3, +4.0, +3.4, +2.7, +2.1 kg from crop 3), so it would level off
when the imports stop. That is the user's "relative balance", reached slowly; the pile is 34.8 g/m²
(**537 mg/L** at drained-full, 864 at the dry end of a watering cycle) where 3a reached 342 g/m² and
5 300 mg/L — a tenth, but still past [F]'s 200 mg/L (mineralization stops) and 400 mg/L reference points.

**The dose rule's own flaw, measured.** Dosing "what the roots took" holds the soil where it is and can
never raise it. In crop 1 the tank could have covered the shortfall in **28** of the 621 capped steps
(12 in crop 2, then none); the rest of the cap came after day 93, when EcoSanRes says to stop. It cost
no grain (the twin and the never-capped run). Not changed: the rule is the one approved.

**Unpredicted: the urine is not stored a month.** EcoSanRes advises storing urine one to six months.
The tank is emptied every window, so the urine made inside a window reaches the soil within days; only
the ~47 days made after each window (~3.2 kg) waits for the next sowing.

**Next (the user's choice):** (a) let the run go on — the farm is still filling and the imports still
falling; (b) CEEF's bought CO₂ (§9) to fill the farm's carbon without imported nitrogen; (c) a dose rule
that tops the soil solution up to a set level from the tank, which would keep the surplus in the tank as
F3 expected (⚠ not "the uptake less the soil's release": that holds the soil flat in the window and lets
it fall after day 93 — ~3.7 kg taken and ~1.9 kg released over crop 7's last 47 days, −1.7 kg a season
against a 1.38 kg start — so it would starve the crop; advisor, 2026-10-21); (d) 3b's ammonia, the marsh, or draining.
