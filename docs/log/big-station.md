## **The big station** (the user's restated aim — the reference station's 1 m² bed carries under 1 % of its four crew's air; the floor is now 4 crew and 200 m² of crops, the aim a whole ecosystem bigger than Biosphere 2)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-21** on the user's restatement of the project (*"we implement a system - plant models,
growth models, physical and engineering systems and process ... then we combine them at different
scales ... the aim is simulation of relatively big closed and open biospheres"*) and, on learning the
bed's share, *"4 people and 200 square meters of crops will be the lower limit that we work on. we aim
for a whole ecosystem, bigger than biosphere-2"*. A plan; nothing built, no value, golden, manifest or
`source:` string moved. Plan: `docs/plans/post-roadmap-big-station.md`.

* **Why the station was 1 m²:** Phase 6 built it as a conservation demo and deferred sizing; the
  sizing step calibrated the crew and never resized the farm; later work refined the rig. Recorded
  since 2026-08-09, never raised as the main problem.
* **Scope (user):** open biospheres are future spin-offs; light and power are choices — outside
  electricity first, then sunlight through glass, then solar and fission power. The big station is
  adopted in ONE unfreeze, the approved 98 % recycler folded into it (advisor).
* **Step 1, next:** a lab at 200 m² and 4 crew on outside electricity, predictions B1–B6 written in the
  plan §3 (headline: CO₂-starved, closure share 20–50 %).
* **Step 1 MEASURED 2026-10-21** (plan §3a; lab `examples/big_station.rs`, 1 m² and 200 m², 1220 days,
  four crew). At 200 m² the room's CO₂ falls to 158 ppm in lit hours and the crop grows 43–55 % of the
  1 m² crop per m²; with 800 crew (the 1 m² CO₂ per m²) it grows back — CO₂ is the whole cause.
* **Closure share** 36.7 % in the first crop, 8.4 % over the run: with the harvest off, each re-sow puts
  the crop into the soil and the soil breathes it out (carbon books close to 0.000 mol). The crop holds
  31–38 % of the crew's carbon at its peak, every crop: the capacity, not yet closure.
* **Missed:** B5's water, 1.55× low (466 kg/day, 2.33 kg/m²/day at both sizes). Stand-ins never bound
  (cooler never blocked, rationed 0). Engine limit found: the every-step conservation tolerance
  (1e-9 of the step's transfer) cannot absorb rounding on a stock of ~1e14 J; a long big run nears it.
* **Plumbing, inert:** the chamber's coefficients and the room's air size are scenario inputs now
  (20 goldens identical, `tests/chamber_sizing.rs`). Step 2 is the user's choice.
* **Step 2a MEASURED 2026-10-21** (plan §5–§5a; lab `--co2-store`): the scrubber's CO₂ goes to a store
  and is injected back while the lamp is on, after Japan's CEEF (separator → reservoir → flow
  controller; Tako et al.). The 200 m² crop grows back to **99.4 %** of the 1 m² crop per m², every
  crop; lit-hour CO₂ never below 1 255 ppm (Step 1: 158); nothing vented. The tank holds **102 000 mol**
  (4.5 t CO₂) by day 1220 and keeps growing — the carbon has nowhere to go without the food loop.
  **Missed:** the O₂ tank still supplies 84 % of the crew's need (predicted ~100 %): the soil keeps a
  falling share of each crop. Water through plants 2.33 kg/m²/day whatever the crop's size.
* **Step 2b MEASURED 2026-10-21** (plan §6–§6a; lab `--food-loop`): the grain less the next seed goes
  to the crew's food at maturity (a batch harvest, not the continuous `Harvest` flow); straw and roots
  to the soil (the re-sow already did); feces to the soil. The crop feeds **30.4 %** of the four crew
  per crop (~165 m² per person; NASA estimates ~50); the food store falls 75 mol/day instead of 104.
  **Missed:** closure share 36.4 % (predicted ~45): eaten grain is grain the soil never keeps, so the
  shares do not add. The O₂ the station buys equals the CO₂ piling up in the tank, to 1 mol. Nitrogen
  closes only through the inedible parts; grain, crew and urine carry none (a gap, not built). No
  fertilizer is added; the soil's 100 kg N/m² (non-physical, recorded) is what the crop draws on.
* **Step 2c MEASURED 2026-10-21** (plan §7–§7a; lab `--nitrogen`): food carries nitrogen — the imported
  menu at BVAD's 19.5 g N per crew-day, the grain at USDA's 2.49 % of dry mass, taken from the plant
  but never below the straw's cited 0.5 %; the crew pass it all, 88 % in urine to the soil's available
  pool and 12 % in feces to its litter (Jönsson & Vinnerås 2004). The books close every crop (within
  3·10⁻⁷ kg). The grain gets **86 %** of USDA's nitrogen (predicted 50–70 %: the plant holds more than
  I guessed; the straw residual is what binds); the harvest covers **21 %** of the crew's nitrogen over
  the run; **71 kg** piles up in the soil, which at 20 t (non-physical, recorded) cannot feel it.
  Carbon identical to 2b. A real-sized soil and loss paths are the user's sourced choice; P, K and
  trace elements need new quantities in the frozen core.
* **Step 3a MEASURED 2026-10-21** (plan §8–§8a; lab `--real-soil`): a soil of real size after Soltani &
  Sinclair Ch. 18 — a silt loam's solution nitrogen at sowing, **1.97 g N/m²** instead of 100 kg, and
  the roots' reach as the nitrogen dissolved in the plant-available water (Eqn 18.5) instead of the
  1–50 kg/m² availability band. The crop is **never** short (0 capped steps); carbon identical to 2c.
  The imported food's nitrogen fills the soil: **342 g/m²**, **~5 300 mg/L** at drained-full (~7 800 at
  the dry end of a watering cycle) by day 1220, past every reference point the book uses; the model has
  no salt damage to show it. Of the book's three losses, only ammonia from urine can run here (2 % or
  10 %, Table 18.1 read off the page): the station never drains the root zone and is never waterlogged.
  None of the three balances the loop (losses outside Ch. 18 not looked for) — the fix is a design
  choice (grow more of the food, take nitrogen out on purpose, or drain). The cap is shown live: a
  floor of 1 000 mg/L starves the first crop.
* **Step 3c PLANNED 2026-10-21** (plan §9; the user's choice, ahead of 3b's ammonia): 700 m² so the
  harvest can feed the crew; imported food a reserve, eaten only when the harvest runs short; urine to a
  tank, dosed to the soil as the roots take it, from sowing to two-thirds of the season (EcoSanRes);
  EcoSanRes' person-day dose and its 4× toxic limit as the "too salty" readout. Carbon decides the
  nitrogen (advisor): the farm and soil fill with carbon only from imported food, so imports — and their
  nitrogen — run until they are full; CEEF's bought CO₂ is the named alternative. Predictions F1–F6;
  the user also approved spray watering / rain and a marsh for later steps.
* **Step 3c MEASURED 2026-10-21** (plan §9a; lab `--feed-crew`, 700 m², 1220 days). Rationed 0, every
  book closes. The farm fills slowly: harvest 27 % of 2b's per m² in crop 1, 38 % in crop 2, 79 % by
  crop 8 and still rising — each crop's extra carbon is only the imported food eaten to cover the gap,
  and the soil keeps more than the crop gains, so the gap shrinks by about a fifth per crop (F1, F2's
  timing missed; the imports' total, 57 541 mol C, held). The crop fed 56 % of the crew over four years,
  84 % in crop 8. Imported nitrogen 43 kg (held), but it went to the soil solution (+23 kg, to
  34.8 g/m², 537 mg/L at drained-full — a tenth of 3a), not the tank (0.4 kg; F3, F4 missed): the dose
  equals the roots' uptake and ignores everything else that feeds the soil solution (+4–8 kg a crop,
  consistent with the old crop's straw breaking down), so the tank empties every window. The soil's
  gain falls with the imports. Late-season nitrogen binds in crop 1 only and costs no grain (a
  urine-to-soil twin and a never-capped soil are identical). The urine is dosed within days, not
  stored the one to six months EcoSanRes advises. Next choices are the user's.
