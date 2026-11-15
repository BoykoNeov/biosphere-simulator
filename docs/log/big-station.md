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
* **Step 4 PLANNED 2026-10-21** (plan §10; the user's choices on §9a's four). One change at a time:
  (a) 3c run on to day 3000 as the baseline; 4a gas tanks charged at the start and refilled from the
  air (CO₂ 60 000 mol, O₂ 11 000 mol, both inputs; the O₂ side becomes a real tank with a floor —
  today it is an outside source), predictions G1–G6; then 4b a top-up dose, 4c drainage and a marsh.
  Ammonia dropped as a loss (the user: it comes back to the soil with the water).
* **Step 4 (a) and 4a MEASURED 2026-10-21** (plan §10, §10.2; 3000 days each). (a) 3c run on: the
  harvest reaches 89 % of 2b per m² by crop 20, still rising; imports never stop (74 693 mol C); soil
  solution 54 g N/m². 4a, CO₂ tank charged with 60 000 mol and a real O₂ tank of 11 000: crop 1 at
  97.5 % (G1), imports 18 300 mol C and 13.75 kg N (G2), the O₂ tank mirrors the CO₂ tank to 1.0000 and
  never empties (G4), no O₂ bought. G3 missed: the CO₂ tank never ran short (lowest 2 364 mol on day
  2939) — I over-counted the soil's gain. The farm is now nitrogen-limited (G5): the soil solution
  sits at 0.12 g/m²; ~500 stressed plant steps a crop in the growing weeks, factor ≥ 0.88 (a twin
  checked; the printed 0.003 is a re-sown seedling's inherited residual) — little growth lost, but the
  grain gets 48 % of USDA's nitrogen. Nitrogen parks in soil organic matter (7.4 kg) and the urine
  tank. "Feeds the crew" counts calories, not protein. The user chose a fertilizer store first, a
  legume crop later.
* **Step 4b MEASURED 2026-10-21** (plan §10.3, §10.4). The user chose full grain protein; it tops out
  at the crop's own ceiling, **87 %** of USDA's nitrogen (the frozen uptake leaves out the grain; 2c:
  85.8 %). A top-up to 1.97 g N/m², urine in EcoSanRes' window and fertilizer at any time (Linquist et
  al. 1992), reaches it: the same crop as at 10 g/m² to the printed digit (P2 missed high — capped
  steps are caught up). Protein 53 → **94 g** per person-day (BVAD 122), grain carbon 98 %. Two loops
  stay open: ~1.9 kg fertilizer a crop goes on after day 93 while the urine tank gains ~2.3 kg a crop
  (39.5 kg by day 3000); and with no imports the CO₂ tank empties on day 1534 — the soil's gain and
  a 4.6 % food surplus — after which the farm is carbon-limited at 88–90 % and imports return.
* **Step 4b-2 MEASURED 2026-11-10** (plan §10.5, §10.6; the user chose nitrogen first, urine at any
  time, then carbon, then the marsh). `--urine-any-time` sets aside EcoSanRes' day-93 cutoff (a
  hygiene rule; wheat is threshed and cooked — a reading, not a citation). The crop, diet and carbon
  books are 4b's to every printed digit (Q4); the CO₂ tank still empties on day 1534. Fertilizer
  over 3000 days **43.7 → 13.2 kg**, urine tank at the end 39.5 → 9.0 kg. It does not reach zero: in
  crops 2–11 the urine is all used, and the books show the fertilizer pays for the two stocks still
  growing, the soil's organic matter and the uneaten food store (equal by construction, not a
  measurement). From crop 12, carbon-short, the crew eat the surplus down, the fertilizer stops and
  the tank climbs again (+0.4 kg a crop). Q1/Q2 missed high: the soil ends every crop above its
  level, so late urine (1.81 kg at each crop's end) collects; storage time unmeasured.
* **Step 4c MEASURED 2026-11-15** (plan §10.7, §10.8; the user chose (b), wastes burned back to CO₂
  as CEEF did, plus a bigger CO₂ charge and a one-year food reserve, grain above it burned).
  `--burn-wastes`: straw (leaves, stems, stem reserve; the roots stay), feces and the food above the
  reserve go to a burner — carbon × 0.936 to the CO₂ tank, 0.064 kept as char (Tako et al. Fig. 2(b),
  "Deposition" 3.3 of 51.9 mol C/day), nitrogen to N₂ (BVAD Eq. 4-1). Over 3000 days at 98 000 mol
  CO₂: the tank never runs short and the crop stays full size (4b-2's emptied on day 1534); lowest
  22 012 (char 0: 38 693), so the bigger charge was needed only with the char. The soil still gains
  (roots and shed leaves, +344 by crop 20; B1 missed); the tank's lasting leak is the char (832 a
  crop) — whether CEEF's deposition is a lasting sink is unread. Nitrogen opens: 2.6 kg a crop to N₂,
  fertilizer ~3.0 kg a crop (66.5 kg in all). The food rule fires only in a 5-day twin (the year is
  not reached in eight years).
* **Step 4c-2 MEASURED 2026-11-15** (plan §10.9, §10.10; the user: "work on those" — the leak, the
  char, the burned nitrogen). ⚠ Correction: CEEF's "45 % leak" (Tako et al. Fig. 2(b)) is an
  ESTIMATED remainder; Fig. 3's measured curves return ~86 % of what reached the processor, the rest
  the deposition dumped at cleaning (38 mol). `--burn-co2-leak 0.45` as an upper bound: the CO₂ tank
  dry day 1257, the crop at 58 %, and — not predicted — the O₂ tank empties (day 2443): every leaked
  carbon takes two oxygen atoms; the room reaches 0 % O₂. Char 0.116 (the measured stream): never
  short, lowest 8 460 mol. Nitrogen at Takahashi & Ohya 1986's Ru catalyst (half denitrified):
  fertilizer 66.5 → 40.3 kg over 3000 days, nothing else moves; a legume later (the user). The
  leak run is unrationed to day 2380: the 58 % crop is the science, the rationing starts with the O₂
  tank empty. The user chose char 0.116 (dumped, borrowed for straw) and the catalyst share kept on.
