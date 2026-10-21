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
