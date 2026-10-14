# Post-roadmap: checks against the real world (the 2026-09-29 review's Step 6)

**Opened 2026-10-06 on the user's call** (*"step 6"*, choosing it from the list of open work
after Step 3c slice 4 closed). The step as proposed: `post-roadmap-review-2026-09-29.md`, Step 6.
Its four slices: (1) a drought scenario in the lab, (2) choose the comparison data, a list for
the user, (3) a scorecard, lab-only, (4) a licensing check on each source.

**What this step is not.** It is not the PCSE oracle comparison (`post-roadmap-validation.md`,
bucket 3), which compares the crop to another *model*. This one compares runs to measured
*chambers*. Nothing here gates and nothing here moves a frozen value.

---

## 1. Advisor review (2026-10-06), summarized

* Read the existing validation plan first; extend rather than duplicate. (Done: it is the PCSE
  oracle work, a different comparison, so this is its own plan.)
* **Slice 1 may be partly done — measure before building.** The step's premise ("no scenario on
  the roster produces drought") was written from `log/leaf-rust-remeasure.md` finding 5, which is
  about the *lab* leaf form's drought factor, not the growth-stress one. Count which factors fire
  on the existing deep-water run first; what already fires is not a gap.
* Use the existing seam for a window: `Irrigation` reads its capacity from a forcing, and
  `perturbations::window_override` + `with_forcing` at 0 is a hard off (`flows.rs`, the
  irrigation doc). No new flow, lab-only, no golden.
* Predictions before the run; a control at the default 8 mm/day where every factor is exactly 1;
  one mutation that must turn the test red.
* Place the window knowingly: transpiration ignores leaf area, so an early drought stresses a
  seedling with water it would never use. Around the closed canopy / flowering the gap distorts
  least. A stated limit, not a fix.
* Slice 2 ends with a question to the user. Flag for every candidate trial whether its CO₂ was
  **held** (needs the controller the third direction plan holds at "not yet", its §2.2) or
  **free-running**, and the **crop class** against our winter wheat with a cold period.
* When slice 3 comes, label the water-use row as dominated by the known leaf-area gap up front.

---

## 2. Slice 1 — measured before designing (2026-10-06)

A throwaway probe (`W:\temp\claude\step6\zz_probe_drought.rs`, copied into `domains/tests` for
one run and removed) read the root zone's water fraction (`FTSW`) on every emitted state of one
season, and counted the steps below each threshold. Every drought factor in the crop is the same
`FTSW` against its own threshold (`science::soil_water_stress`): growth (`WSFG`) and transpiration
both at `wssg` = 0.30, development speed (`WSFD`) from the growth factor and `wssd` = 0.40, and the
lab leaf form's expansion (`WSFL`) at `LEAF_WSSL` = 0.40.

| run | lowest FTSW | steps below 0.30 (growth, transpiration, development) | steps below 0.40 (lab leaf form) | largest development factor | anthesis / maturity day |
|---|---|---|---|---|---|
| default, 8 mm/day | **0.9933** (day 0) | **0** | **0** | 1.0000 | 250 / 293 |
| deep water, 1 mm/day (`system.rs` tests) | 0.0810 (day 292) | 1278 (days 225–305) | 1434 (days 215–305) | 1.2919 | 247 / 283 |

**Finding — the step's premise is true of the frozen roster and false of the suite.** No frozen
scenario ever stresses the crop. But two test-only scenarios already do: the deep-water run above
(growth, transpiration and development all fire for ~80 days), and the manufactured dry run in
`drought_acceleration_is_wired_into_the_accumulator_and_no_scenario_shows_it`. So "drought is
unexercised" narrows to:

1. **No drought has an end.** Both test droughts are chronic — the water never comes back. Whether
   every factor returns to exactly 1 once watering resumes has never been run.
2. **The lab leaf form's drought factor has never fired** (finding 5 stands for it): no test runs
   that form on a dry soil.
3. ⚠ **Stale, left:** that test's name and doc ("Every scenario in the Rust roster holds
   `WSFG == 1`") were written before the deep-water test was ported; the deep-water run does not
   hold it. Same rule as the other stale notes — recorded here, the name left alone (Step 8).

## 3. Slice 1 — the design

A lab test file, `rust/crates/domains/tests/drought_window.rs`. The default wheat, one season,
with watering **cut to 0 over days 220–260** (steps `220·16` to `260·16`) and back to the
default 8 mm/day after, by `window_override` on `IRRIGATION_VAR`. No new flow, no new parameter,
no scenario field: the window lives in the forcing, as the perturbation harness's do.

**Why days 220–260.** The control flowers on day 250 and matures on day 293. The window spans the
last month before flowering and the first days after, when the canopy is near its largest — where
the leaf-area-blind transpiration (below) distorts least. It also leaves 33 days of watering
before maturity, so recovery can be seen inside the season.

**Known limit, stated not fixed:** transpiration is the full-cover Penman–Monteith rate times the
soil-water factor; it does not read leaf area (`flows.rs` `Transpiration`;
`post-roadmap-leaf-shedding.md` §12; `post-roadmap-room-temperature.md` §21). So the soil dries at a
full canopy's pace whatever the canopy is. The drought this test makes is the model's, not a
measured one.

**What it asserts** (an instrument, not a reference scenario):
* the control (no window) holds every factor at exactly 1 on every step;
* in the windowed run, growth/transpiration stress (`FTSW < 0.30`) fires inside the window and
  never before it;
* every factor is back to exactly 1 before maturity and stays there to the season's end;
* drought hastens development: the windowed run's thermal time at the season's end exceeds the
  control's;
* the books: 0 rationed, no events, a re-run bit-identical (conservation is asserted by the engine
  on every step, so a closed run is the conservation check);
* the lab leaf form (`LeafAreaForm::NodeEnvelope`) on the same window: its leaf factor
  (`FTSW < 0.40`) is below 1 on at least one step before flowering, when leaf expansion runs.

**Mutations, each must turn the test red (`--no-fail-fast`):**
* M1 — the cut value 0 → 8 (the window becomes a no-op): the "fires" assertion goes red.
* M2 — the window's end moved to the season's end (no recovery): the "back to 1" assertion goes red.

## 4. Slice 1 — predictions (written before the test exists)

Basis: the root zone holds `rooted_depth × 0.13 × 1000` kg, so at a rooted depth of 1.0–1.3 m
about 130–170 kg; late-spring demand on this weather is ~4–5.8 kg/day (the deep-water test's own
5.77 peak); stress starts after the zone loses 70 %.

* **D1 — onset.** FTSW first falls below 0.40 between 10 and 30 days after the cut (days
  230–250), and below 0.30 between 12 and 35 days after (days 232–255). Confidence medium: the
  water the roots capture as they deepen into the subsoil delays it, by an amount not estimated.
* **D2 — depth.** The lowest FTSW is between 0.10 and 0.29 — below 0.30 is D1; above 0.10 because
  below 0.30 the water loss itself slows in proportion.
* **D3 — recovery.** After watering resumes on day 260, FTSW is back at or above 0.30 within 15
  days (by day 275), and every factor is exactly 1.0 from then to day 305. Irrigation fills up
  to the deficit at 8 kg/day against a stressed demand below 5.
* **D4 — development.** The largest development factor is between 1.05 and 1.40 (1 + `wssd` is the
  ceiling). Flowering is 0–4 days earlier than the control's 250; maturity 0–8 days earlier than
  its 293.
* **D5 — grain at maturity.** Below the control's, at 0.60–0.97 of it. Confidence low: the model has
  no grain number, so a drought before flowering acts only through less carbon fixed and stored.
* **D6 — the books.** 0 rationed, no events, the re-run bit-identical.
* **D7 — the lab leaf form** fires its factor on at least one pre-flowering step, in the window.
* **D8 — the control** holds every factor at exactly 1 (already measured: lowest FTSW 0.9933; this
  is the test's own check, not a new prediction).
* **D9 — nothing frozen moves:** `regen_goldens` reports 20 of 20 identical; no manifest line.

## 5. Slice 1 BUILT (2026-10-06, lab-only) — the predictions graded

`rust/crates/domains/tests/drought_window.rs`, six tests, green on the first run. Nothing frozen
moved: no source file outside the new test changed.

| | predicted | measured | grade |
|---|---|---|---|
| D1 | below 0.40 on days 230–250; below 0.30 on days 232–255 | **day 249** and **day 254** (29 and 34 days after the cut) | HELD, both at the late edge |
| D2 | lowest FTSW 0.10–0.29 | **0.2093** | HELD |
| D3 | back at ≥ 0.30 by day 275; every factor exactly 1 from then to day 305 | last state below 0.30 on **day 263**, below 0.40 on **day 266**; exactly 1 from there to the end | HELD |
| D4 | development factor peak 1.05–1.40; flowering 0–4 days early; maturity 0–8 days early | **1.1209**; flowering **day 250 = control**; maturity **day 293 = control**; end thermal time 2038.22 vs 2028.37 | HELD (both shifts at 0) |
| D5 | grain at maturity 0.60–0.97 of the control's | 26.92 vs 27.95 mol C, **0.9631** | HELD, near the top edge |
| D6 | 0 rationed, no events, a re-run bit-identical | as predicted (each asserted) | HELD |
| D7 | the lab leaf form's factor below 1 on ≥ 1 pre-flowering step | **17 states**, all on day 249 | HELD, but weak (below) |
| D8 | the control holds every factor at 1 | lowest FTSW 0.9933 | HELD |
| D9 | 20 of 20 goldens identical | see §5a | — |

**Mutations (`--no-fail-fast`), both red as designed:** M1 (the cut 0 → 8 mm/day) turned 4 of 6
red — fires, recovers (no stressed state to recover from), hastens, the lab leaf form; M2 (the
window to the season's end) turned exactly the recovery test red.

**Finding — the root zone carries about a month of a full crop's water.** With the roots near
their depth, a full zone loses its first 70 % in 29–34 days at the full-cover rate, so a 40-day cut
stresses the crop only on its last 6–11 days. The window was placed to cover "the month before
flowering"; the stress it delivered fell **at and after flowering** (days 249–266 against
flowering on 250). That is why flowering and maturity did not move: the development factor ran
above 1 for only ~10 days. The test's assertions do not depend on where the stress falls, so the
window is left as built and the fact recorded.

**D7 is weak, stated so.** The lab leaf form's factor sits below 1 on one day before flowering. That
shows the drought reaches the factor's input; it does not show the form's leaf area moves because
of it. A window starting ~30 days earlier would give the form a real pre-flowering drought; not
done, because finding 5's mechanism stays parked.

### 5a. The gates

* **D9 HELD:** `regen_goldens` (report only) — 20 of 20 goldens identical, 0 would change.
* `cargo clippy --all-targets -- -D warnings`: clean.
* `cargo test --no-fail-fast`: **1339 passed, 1 failed, 6 ignored.** The one red was
  `repo_gates::every_plan_doc_is_indexed` — this plan had no index row yet, the expected
  bookkeeping. The suite ran BEFORE the doc edits; `repo_gates` was re-run after them (and after
  the direction plan's re-read marker moved) and is green.


---

## 6. Slice 2 — the comparison data: the search (2026-10-06)

**The shelf first.** `sources/` holds no closed-chamber trial. The project's own record holds two
pointers: BVAD Rev 2 Table 4-91's wheat row (nominal CO₂ uptake 77.00 g CO₂ m⁻² d⁻¹, used in
`log/chamber-scale.md`), a *design* value and not a trial; and NASA TM 102788, named a Step 6
target by `log/co2-uptake-source.md` and never opened.

**Opened this session: NASA TM 102788** — Wheeler, R.M. & Sager, J.C. (January 1990), *Carbon
Dioxide and Water Exchange Rates by a Wheat Crop in NASA's Biomass Production Chamber: Results
from an 86-Day Study (January to April 1989)*, Kennedy Space Center. NTRS 19900016137 (scanned;
every page below read off the page image). Now on the shelf:
`sources/wheeler_sager_1990_TM102788.pdf` (the shelf is git-ignored). A US-Government work, public
domain.

| quantity | value (page) |
|---|---|
| chamber | 112.6 m³ incl. air handling; ~20 m² of plants (64 trays × 0.25 m² plus gaps); leak < 10 % of volume per day (p. 2), 5–10 % (p. 10) |
| crop | *Triticum aestivum* cv **Yecora Rojo**, **hydroponic** — ⚠ "spring wheat" is NOT on the TM's pages; it is general knowledge of the cultivar, to be verified against a source before the crop decision leans on it (recirculating nutrient film), ~1500 plants m⁻² (p. 4, p. 9) |
| light | 20 h light / 4 h dark after 72 h dark at planting; 96 × 400 W HPS, dimmed as the crop grew to hold canopy-top PPF; **mean 534 µmol m⁻² s⁻¹** (± 42 between levels); 695 → 480 on day 28 (pp. 5, 11) |
| temperature, humidity | 20 °C constant to day 34, then 20 °C light / 16 °C dark; means 20.3 / 16.8 °C; RH 81 % / 82 % (p. 5) |
| CO₂ | **HELD at 1000 ppm** by injecting pure CO₂; means 1160 ± 238 ppm light, 1380 ± 310 dark; supply ran out briefly near days 50 and 60 (pp. 5, 11) |
| span | planted 19 Jan 1989; water off day 84; lights off day 86 (p. 6) |
| dark respiration | rose to ~13 µmol m⁻² s⁻¹ by ~day 20, then fell; **mean 7.2** (440 ppm per 4-h night); +65 % / +45 % for 16 → 20 °C at 5 / 10 weeks (p. 9) |
| net photosynthesis | **peak 27 µmol m⁻² s⁻¹ at day 25**; **mean 15** over days 10–84; low before day 15 from incomplete ground cover (pp. 9–10) |
| carbon balance | 21.6 mol CO₂ fixed per 20-h day, 2.1 respired per night: **19.5 mol CO₂ d⁻¹ net** for the stand (≈ 0.98 mol m⁻² d⁻¹, 43 g CO₂ m⁻² d⁻¹ — about 56 % of BVAD's nominal 77) (p. 10) |
| biomass | **~40 kg total dry biomass at harvest** (≈ 2.0 kg m⁻²); the gas-exchange estimate 44 kg (p. 10) |
| light response | stand uptake linear in PPF from 60 to 750 µmol m⁻² s⁻¹; light compensation point ~190 (p. 11) |
| CO₂ response | drawdown rate flat from 2200 to ~800 ppm, falling below 700–800; a deliberate drawdown from 2200 ppm on day 25 (Figs. 2–3) (pp. 7–8) |
| transpiration | condensate-measured; **peak 120 L d⁻¹ near day 25** "when ground cover was complete", then about constant to senescence; **mean ~90 L d⁻¹ = 4.5 L m⁻² d⁻¹** (pp. 12–13) |

⚠ **Context added to an earlier record, not a correction.** `log/co2-uptake-source.md` read Figs.
2–3 as what they are: one deliberate drawdown from 2200 ppm on day 25 "in the sealed 113 m³
chamber". (A first draft of this paragraph called that a mistake; re-reading the record showed it
was not.) What the record did not say: outside that test the trial **held CO₂ at 1000 ppm** (p. 5),
and "sealed" is approximate — 5–10 % of the air leaked a day (pp. 2, 10). Annotated there in
place. A usable CO₂-response curve; the trial as a whole is a held one.

**Not opened (named, with what blocks each):**

| candidate | what it is | access | why it matters |
|---|---|---|---|
| Bugbee & Salisbury 1988, *Plant Physiol.* 88:869–878 | wheat (hydroponic, 2000 plants m⁻²) at CO₂ **held at 1200 ppm**, PPF 400–2080, 16–20 h; harvests at 24, 45, 79 days; crop growth rate to 138 g m⁻² d⁻¹, grain 60 g m⁻² d⁻¹; harvest index 41–44 % (abstract only) | open on PMC (PMC1055676), read as abstract | the light-response row: the same crop across a 5× light range |
| Wheeler et al. 1996 and 2008, *Adv. Space Res.* (2008: 41:706–713) | the BPC's whole programme: five wheat, three soybean, five lettuce, four **potato** crops; biomass against daily light, radiation use 0.4–0.7 g per mol photons | Elsevier; seen only through a NASA slide deck (NTRS 20150022488), so far secondary | the potato row for our second species |
| Lunar-Mars Life Support Test Project Phase I (JSC, 1995; Edeen & Barta, JSC-33636) | one crew member 15 days in a closed chamber with 11.2 m² of wheat; CO₂ and O₂ held by changing the light and the CO₂ supply | NTRS candidates (19980233235), not opened | the only **crew + crop** row — the station's question, not the jar's |
| Gerbaud, André & Richaud 1988, *Physiol. Plant.* 73:471–478 | a closed wheat chamber over the life cycle, 80 plants m⁻²; transpiration peak ~9, mean 5–6 L m⁻² d⁻¹ (as quoted in TM 102788 p. 13) | Wiley, not opened | a second transpiration row at a lower plant density |
| Lunar Palace 1 (Beihang, 2014, 105 days, 3 crew) | wheat as the main O₂ source in a multi-crop closed system | papers not opened | multi-crop, crew; conditions probably held |

### 6a. The two filters, applied

* **CO₂ held or free-running.** The trials with a stated CO₂ control all held it: TM 102788 (read),
  Bugbee & Salisbury (abstract), LMLSTP Phase I (held by changing the **light** and the CO₂ supply —
  a controller of another kind). Gerbaud et al. and Lunar Palace: not known. A held chamber
  **should** map onto a **fixed leaf-internal CO₂ forcing** (`CI_VAR`, 250 ppm by default), which the
  open-field build reads — so the parked chamber controller (third direction plan §2.2) would not
  be forced for the held rows. ⚠ **Not measured, and no build hosts it yet:** the open-field build
  reads the outdoor weather (no lamp, no 20-h day, no held 20/16 °C); the lamp-lit station builds
  (`station::scenario::lighting_scenario`, `day_neutral_lighting_scenario` — lamp PAR, net radiation
  and daylength, a constant temperature, the day-neutral crop) all build a **sealed** biosphere,
  whose leaf CO₂ comes from the chamber's air. Slice 3 confirms or refutes the mapping by building
  the row. For LMLSTP "not forced" is not established. The jar has no counterpart found.
* **Crop class.** The wheat trials read grew **Yecora Rojo** (spring habit by general knowledge, see
  the table) **in nutrient solution** under 16–20 h light. The reference crop is a **winter** wheat that needs a cold period.
  A scorecard row therefore runs the crop with vernalization off — the same family as the
  day-neutral crop already built (`bucket3-day-neutral-crop`), whose own lesson applies: matching
  the *timing* with a crop from the same family is tautological, so the row reads rates and
  amounts, not dates. Hydroponic roots never dry; an irrigated soil held at FTSW ≈ 1 (measured in
  §2, the control's lowest 0.9933) is the closest the model has.
* **The water row is labelled now, before any run:** the model's transpiration is the full-cover
  rate from day 0, so it cannot show TM 102788's low early water use "before ground cover was
  complete". Any ratio on transpiration before ~day 25 is the known leaf-area gap, not a finding.

### 6b. Licensing (slice 4, as each source is listed)

TM 102788: a US-Government work, public domain; its numbers are facts cited to the page. Bugbee &
Salisbury 1988: free to read on PMC, publisher copyright; measurements cited to the paper are
facts (`docs/reuse-and-licenses.md`); no dataset file copied. The rest: not opened, so not
checked — each gets this check before a number from it is committed.

### 6c. Decisions owed to the user

1. **Which trial(s) to score first.** Recommended: TM 102788 alone first — opened, public domain,
   conditions complete, and it reports the components (night respiration, day uptake, water,
   biomass) the review asked for, not only yield. Then Bugbee & Salisbury 1988 for the light range.
2. **The crop for the row.** Recommended (once the cultivar's spring habit is checked against a
   source): the frozen wheat with vernalization off (spring habit),
   read on rates and amounts, not dates. The alternative — the winter crop given the cold period
   first — would compare a crop the trial did not grow.
3. **Whether to go after the unopened sources** (the BPC programme papers for potato; LMLSTP Phase
   I for crew + crop) now, or after the first row.

**ANSWERED 2026-10-07 (the user), all three as recommended:** (1) TM 102788 first; (2) the frozen
wheat with vernalization off, read on rates and amounts, not dates — after the cultivar's spring
habit is checked against a source; (3) the unopened sources after the first row.

**The cultivar's habit, checked (2026-10-07):** Golden State Grains' variety page — *"short-statured,
hard red spring wheat"*, developed by CIMMYT with Mexico's agriculture ministry, received in
California in 1970. A seed-company page, so a secondary source; the UC Davis Foundation Seed page it
appears to restate returned 403. Enough for the habit (spring, no cold requirement), which is what
the crop decision rests on.

---

## 7. Slice 3, row 1 — TM 102788: the design (2026-10-07, before code)

### 7a. Advisor review (2026-10-07), summarized

Split the readouts into **rates at a closed canopy** (comparable) and **season totals** (dominated by
how fast the canopy closes, which the model's seedling sets: 0.16 mol C m⁻² and no plant count,
against the trial's ~1500 plants m⁻²) — keep the frozen seedling and label the totals; resizing it
would be fitting. Book CO₂ flow by flow and prove the books close on the observed change. Prove
every outdoor forcing is replaced. Prove the crop is never short of water or nitrogen, and fix the
conditions — not the science — before reading any score. Check the unit conversion against the
TM's own arithmetic. Read night respiration on fully dark steps only; use the day-34 night-
temperature drop as a rate test. Fix the CO₂ input first and run two levels. Label the water row
twice (leaf-area-blind; a fixed surface resistance that does not close at high CO₂). Put it in
`station/tests` to reuse the lamp conversion. Assert only books and conditions; pin no ratio. The
advisor's own guesses (season totals < 1, water > 1, peak uptake nearest 1) are for checking, not
adopting — and on water this plan disagrees (§8, W1).

### 7b. What the model is given

`rust/crates/station/tests/scorecard_tm102788.rs`, lab-only. The frozen wheat, open build
(`DEFAULT_SCENARIO`, `sealed: false`), `vernalization: false`; every weather-derived forcing replaced:

| forcing | the trial (page) | the model |
|---|---|---|
| day 0 | planted; 72 h dark; then 20 h light (pp. 4–5) | model day 0 = **TM day 3**, lamp on from step 0; run 83 days, to TM day 86. Emergence is not modelled; the seedling is the frozen one |
| PAR | 695 µmol m⁻² s⁻¹ until day 28, then 480 (p. 11); study mean 534 (p. 5) | **695 to TM day 28, 480 after** (mean 552, +3 % on the page's 534 — stated); a 20-h top-hat centred on midday (`light_path::top_hat_window_mean`), lights 02:00–22:00 |
| daylength | 20 h | 72 000 s (the photoperiod factor is 1 above 16 h, so photoperiod sensitivity is inert) |
| net radiation | — | `station::lighting::lamp_net_radiation` of the window's PAR, on the same top-hat |
| temperature | 20 °C to day 34; then 20 light / 16 dark (p. 5) | 20 °C to TM day 34; then per step `16 + 4 × lit fraction` |
| VPD | RH 81 % light / 82 % dark (p. 5) | `es(T) × (1 − 0.81)` per step, from the step's temperature |
| CO₂ | held at 1000 ppm; light-period mean 1160 (p. 5) | `ci = 0.7 × Ca` (the frozen sealed `ci_ratio`): **Ca = 1160 → ci 812** as the row; Ca = 1000 → ci 700 as the sensitivity |
| water, nutrients | hydroponic, replenished daily / twice weekly (p. 4) | irrigation 8 mm/day (default); soil N default. Asserted non-limiting (below) |

The open build has **no soil carbon** (the litter–microbe–humus cascade is sealed-only), so its CO₂
exchange is the crop's alone — the trial's nutrient solution had no soil.

### 7c. The instrument's own checks (asserted; none is a score)

* **Weather gone:** every key `weather_forcings` supplies is replaced, except the three the scenario
  sets as constants (`ci`, `irrigation`, `fertilization`).
* **Books:** for every step, the CO₂ legs of every flow, evaluated on the step's state as the engine
  does, sum to the observed change in `boundary.co2_atmos`; 0 rationed; no events; a re-run
  bit-identical.
* **Conditions:** root-zone FTSW ≥ 0.30 on every step; the nitrogen factor = 1 on every step. If
  either fails, the condition is fixed (more water, fertilization) and recorded BEFORE any score is
  read.
* **Units:** the converter reproduces the TM's own arithmetic — 440 ppm × 4692 µmol/ppm ÷ 20 m² ÷
  14 400 s = 7.17 (TM: 7.2, p. 6 and abstract), and 15 µmol m⁻² s⁻¹ × 20 m² × 72 000 s = 21.6 mol (p. 10).

### 7d. The readouts

**Rates (comparable):**
* **U1** peak net daytime CO₂ uptake (fully lit steps), µmol m⁻² s⁻¹ — TM 27 at day 25.
* **N1** night respiration on fully dark steps — TM ~13 near day 20 (Fig. 4, text p. 9); mean 7.2.
* **N2** the day-34 night-temperature step: model night respiration on the last 20 °C night over the
  first 16 °C night — TM: 16 → 20 °C raised respiration 65 % at 5 weeks (Fig. 5, p. 9).
* **W1** transpiration at full cover (TM days 25–80), L m⁻² d⁻¹ — TM peak 6.0, "relatively constant"
  after (pp. 12–13).

**Totals (dominated by the starting seedling — labelled so):**
* **U2** mean net uptake over TM days 10–84 — TM 15 µmol m⁻² s⁻¹.
* **C1** net carbon fixed over TM days 10–84 — TM 19.5 mol CO₂ d⁻¹ × 75 d ÷ 20 m² = **73 mol C m⁻²**.
* **B1** plant carbon at TM day 86 — TM ~40 kg dry ÷ 20 m² = 2.0 kg m⁻² = **67 mol C m⁻²** on the
  TM's own CH₂O basis (p. 10), 75 at a 45 % carbon fraction.
* **W2** mean transpiration — TM 4.5 L m⁻² d⁻¹.

**Sensitivity:** **S1** C1 at ci 812 over ci 700.

## 8. Slice 3, row 1 — predictions (before the test exists)

Hand arithmetic behind the sure ones: maintenance respiration is 0.02 d⁻¹ × 2^((T − 25)/10) of
leaf + stem + root carbon, and at night the model has no assimilate, so it grows nothing — **night
respiration is maintenance alone**. Water: Penman–Monteith with the frozen 50 / 70 s m⁻¹ and the
lamp's PAR-only net radiation gives ~74 W m⁻² lit and ~31–36 dark at 480 µmol, ~92 lit at 695:
**2.4–2.9 mm d⁻¹**, whatever the canopy.

| | direction | range | confidence |
|---|---|---|---|
| U1 peak daytime uptake | below 27 (the model's canopy closes after the day-28 dimming) | 15–35 µmol m⁻² s⁻¹ (ratio 0.55–1.3) | low |
| N1 night respiration near day 20 | far below | ratio 0.01–0.15 | medium |
| N1 night respiration, season mean | below | ratio 0.15–0.6 | low |
| N2 night temperature step | below the TM's 1.65 | 1.20–1.32 (2^0.4 = 1.32, less ~2 days' growth) | medium-high |
| W1 transpiration at full cover | **below** (the advisor guessed above) | 2.3–3.0 L m⁻² d⁻¹ (ratio 0.38–0.5) | medium-high |
| W2 mean transpiration | below | ratio 0.5–0.67 (the model's rate is flat from day 0) | medium-high |
| U2 mean daytime uptake | below | ratio 0.3–0.9 | low |
| C1 net carbon fixed | below | 25–65 mol C m⁻² (ratio 0.35–0.9) | low |
| B1 plant carbon at day 86 | below | ratio 0.3–0.9 | low |
| S1 ci 812 / ci 700 on C1 | above 1, small | 1.005–1.05 (electron-transport-limited FvCB at high Ci is nearly flat: (Ci − Γ*)/(4Ci + 8Γ*) rises 2.4 %) | medium-high |
| conditions | FTSW ≥ 0.30 everywhere (8 mm/day against ≤ 3 drawn) | — | high |
| | nitrogen factor 1 everywhere | — | low |
| books | close to rounding on every step; 0 rationed; bit-identical | — | high |

**Why W1 disagrees with the advisor's guess.** The guess reasoned from stomata (the model's fixed
surface resistance does not close at high CO₂, which would push water use UP). The arithmetic
above says the energy term dominates: the model's lamp heats the crop with its PAR only, while an
HPS lamp's radiant output is mostly outside PAR, and the BPC moved its air hard. If W1 comes out
above 1, the arithmetic was wrong and that is the finding.

## 9. Slice 3, row 1 BUILT (2026-10-07, lab-only) — the scorecard, graded

`rust/crates/station/tests/scorecard_tm102788.rs` at commit `2be91a3`; read with
`cargo test -p station --test scorecard_tm102788 -- --nocapture`. Six tests, all instrument checks;
the scorecard itself is printed, never asserted. Nothing frozen moved.

### 9a. An instrument error, caught before any score was read

The first run printed night respiration as **exactly 0** and the night-temperature ratio as NaN.
Cause, in my instrument and not the model: the open build keeps the outside air in **two** boundary
stocks — `boundary.co2_atmos` supplies the crop's carbon, `boundary.co2_resp` receives what it
respires (`MaintenanceRespiration`'s open-field branch: "covered from the atmosphere, shortfall
from the organs", returned to `co2_resp`). The draft booked `co2_atmos` alone, so it counted gross
intake and no respiration at all. Fixed by booking and closing the books on the SUM of both stocks
(what the trial's analyser saw), with a new check that every day has a nonzero night and day. No
condition or parameter was changed. The draft's printed numbers are not scores and are not used.

### 9b. The scorecard (ci = 812 ppm)

| row | model | TM 102788 | ratio | kind |
|---|---|---|---|---|
| U1 peak daytime net uptake (µmol m⁻² s⁻¹) | **26.43**, TM day 24 | 27, day 25 | **0.98** | rate |
| N1 night respiration, TM day 20 | 1.93 | ~13 | 0.149 | rate |
| N1 night respiration, season mean | 3.44 | 7.2 | 0.478 | rate |
| N2 night respiration 20 °C / 16 °C (day 33 / 34) | 1.294 | 1.65 (Fig. 5, 5 weeks) | 0.784 | rate |
| W1 transpiration, TM days 25–80 (L m⁻² d⁻¹) | 2.376 | ~6.0 (peak; then roughly constant) — ⚠ the PEAK; the level is 4.17 (§11) | 0.396 (0.570 at the level) | rate |
| U2 mean daytime net uptake, days 10–84 | 17.33 | 15 | 1.156 | total |
| C1 net carbon fixed, days 10–84 (mol C m⁻²) | 90.26 | 73.1 | 1.235 | total |
| B1 plant carbon, TM day 86, roots in (mol C m⁻²) | 62.01 | 66.7 (CH₂O basis; 75 at 45 % C) | 0.930 (0.83) | total |
| B1 the same, roots out | 45.43 | 66.7 | 0.681 (0.61) | total |
| W2 mean transpiration | 2.508 | 4.5 | 0.557 | total |
| S1 C1 at ci 812 / ci 700 | 1.0308 | — | — | sensitivity |

Conditions: lowest root-zone fraction **0.9899**; nitrogen factor **1.0000** on every state. The
crop at TM day 86: DVS 1.70 (not mature — dates are not compared); leaf 8.95, stem 15.24, root
16.58, grain 19.42, stem reserve 1.82 mol C m⁻².

### 9c. The predictions, graded

| | predicted | measured | grade |
|---|---|---|---|
| U1 | below 27; 15–35 | 26.43 | HELD — but its stated reason ("closes after the dimming") was WRONG: the peak came on day 24, before it |
| N1 day 20 | 0.01–0.15 | 0.149 | HELD, at the edge |
| N1 mean | 0.15–0.6 | 0.478 | HELD |
| N2 | 1.20–1.32, below 1.65 | 1.294 | HELD |
| W1 | **below** (against the advisor's guess); 0.38–0.5 | 0.396 | HELD |
| W2 | 0.5–0.67 | 0.557 | HELD |
| U2 | below; 0.3–0.9 | 1.156 | **FAILED** (above) |
| C1 | below; 0.35–0.9 | 1.235 | **FAILED** (above) |
| B1 | below; 0.3–0.9 | 0.930 | direction HELD, range FAILED by 0.03 |
| S1 | 1.005–1.05 | 1.031 | HELD |
| conditions | water high confidence; nitrogen low | both non-limiting | HELD |
| books | close, 0 rationed, bit-identical | as predicted (asserted) | HELD |

**The count:** of the **10 predictions about the model**, 7 held, 2 failed (U2, C1 — above their
ranges), 1 held on direction and missed its range by 0.03 (B1, roots in). The last two rows are
the instrument's own checks, asserted by the test; they are not counted.

### 9d. What the row says

1. **The seedling was not the limit — the totals' label was wrong.** Both the advisor and this plan
   expected the season totals below 1 because the model starts from 0.16 mol C m⁻² against ~1500
   plants. The model's canopy closed on the trial's schedule: peak uptake on day 24 against day 25,
   at 0.98 of the trial's rate. So the totals are comparable rows after all, and the "seedling"
   label on them is withdrawn.
2. **Daytime uptake is close; the model fixes ~23 % more carbon over the season** — mostly because
   after the day-28 dimming its daytime uptake sits at 16–18 µmol m⁻² s⁻¹ (day 30: 18.0) where the
   trial dropped to 15.3 (p. 12) and declined with age, and partly because its nights cost half as
   much (next point).
3. **The model's crop barely respires at night.** At night it pays upkeep only (0.02 d⁻¹ × Q10 2 on
   leaf + stem + root); growth respiration happens only while it grows, i.e. in the light. The
   trial's stand respired at 13 µmol m⁻² s⁻¹ near day 20, nearly half its daytime net rate; the
   model 1.9. Its night also answers temperature more weakly (1.29 for 4 °C against the trial's
   1.65, a whole-stand Q10 near 3.5 against the model's 2.0). The daytime net hides this: the
   model's respiration is in its day numbers instead.
4. **Fixation is high (1.23) and a third of it is shed — measured.** Over the whole run the model
   fixed 92.26 mol C m⁻²: 61.85 became plant and **30.41 was shed to the open build's litter sink**
   (the books close to 6e-14). The trial's gap between gas exchange and harvest was ~10 % (p. 10).
   Whether standing biomass is "close" depends on what the TM's "about 40 kg of total biomass"
   (p. 10) covers, which it does not say: **0.93 with roots, 0.68 without** (roots are 27 % of the
   model's plant carbon). Both are reported; neither is chosen.
5. **Water: 0.40 at full cover, below as predicted against the advisor's guess. The cause is NOT
   tested.** Two candidates, neither isolated: (a) the model's lamp warms the crop with its PAR only
   (`lamp_net_radiation`, the 2026-10-05 choice; room-temperature plan §21), while the BPC's
   high-pressure sodium lamps radiate a great deal outside PAR; (b) the frozen resistances (air 50,
   surface 70 s m⁻¹) are the FAO grass-reference values outdoors, while the BPC blew air hard over
   the crop. **Sized:** to transpire 6 L m⁻² d⁻¹ at those resistances the crop needs ~198 W m⁻² of
   latent heat while lit, i.e. ~340 W m⁻² of net radiation — **about 4× the 81 W m⁻²** the lamp gives
   at 480 µmol. The lamp's extra radiation may be only part of that. Whether either candidate holds
   for the station's own lamp is not shown here. The leaf-area-blind rate shows too — the model
   drinks 2.9 L m⁻² d⁻¹ from day 5, when the trial's seedlings drank little.
6. **CO₂: 3 % between 1000 and 1160 ppm**, matching the trial's "decreased slightly from 1500 to
   800 ppm" (p. 8). The fixed-CO₂ forcing does what a held chamber does on this row: the parked
   controller was not needed.

**Not acted on.** No parameter moved; this is a diagnostic. Candidates it names, each the user's:
the night-respiration split (point 3) and the post-dimming uptake (point 2) are science questions;
the water gap (point 5) needs its two candidates separated — a lab what-if (net radiation scaled,
or the resistances lowered) would size each; not run.

---

## 10. The water gap, split (2026-10-07, the user's pick) — design and predictions, before code

**The user's pick:** *"Split the water gap"* — offered as "raise the lamp's heating, or lower the air
resistance, and see which one closes the gap". **The method changed, told to the user before
work:** the model's water use reads only the lamp's net radiation, VPD, temperature and two
resistances, and feeds nothing back while the root zone stays wet. So a sweep inside the model
would only re-run Penman–Monteith. The arithmetic sizes each candidate, sources judge whether
the size is physically plausible, and one small run checks only the instrument and the coupling.

### 10a. Advisor review (2026-10-07), summarized

Arithmetic first: as the air resistance goes to 0, the lit-period limit is `ρcp·VPD/(γ·rs)`, which
may already rule out the airflow candidate alone. The heating candidate needs a plausibility
source (the BPC construction papers the TM cites on p. 2; HPS radiant output outside PAR) — with a
stop rule if those are blocked. Check what the 6 L measures (condensate, p. 7) — open solution
surfaces may evaporate too. One run, WHAT-IF, pinning nothing: a resistance substitution must move
water use to the arithmetic's value (the substitution reaches `Transpiration`), and a scaled net
radiation must leave every carbon row bit-identical while FTSW stays ≥ 0.30. Write, before reading
any source, what each source would have to say to refute each candidate; "not refuted" is the most
a surviving candidate earns.

### 10b. The arithmetic (the model's own constants)

`ρcp` = 1.205 × 1013 = 1220.7 J m⁻³ K⁻¹; `γ` = 67 Pa K⁻¹; frozen `ra` = 50, `rs` = 70 s m⁻¹; VPD =
es(T) × 0.19 = 444 Pa at 20 °C, 345 Pa at 16 °C. (⚠ 2026-10-14: every size in this section is for 6.0, the
trial's PEAK, and the constant `rs` 70; §11 re-sizes them at the level, 4.17, and the adopted form.)
The trial's full-cover 6.0 L m⁻² d⁻¹ is a daily
mean latent flux of ~170 W m⁻²; with the dark hours at their own rate it needs **~198 W m⁻² while
lit**.

* **(b) airflow alone.** As `ra → 0`, `λE → ρcp·VPD/(γ·rs)`: **~116 W m⁻² lit, ~90 dark (16 °C)**, so
  **at most ~3.9 L m⁻² d⁻¹ with infinite airflow** — short of 6.0. Reaching 198 lit at `ra → 0`
  needs `rs` ≈ **41 s m⁻¹**, the leaves' pores MORE open than the frozen 70, where the expected
  direction at 1000+ ppm CO₂ is the opposite (partial stomatal closure — general knowledge, not
  sourced here).
* **(a) lamp heating alone**, frozen resistances: 198 W m⁻² lit needs net radiation ≈ **340 W m⁻²**,
  **~4.2×** the lamp's PAR-only 81 W m⁻² at 480 µmol.
* **(c) evaporation from the nutrient solution's open surfaces** inside the 6 L — not sizable from
  arithmetic.

### 10c. Predictions

| | prediction | what would refute it |
|---|---|---|
| A1 | (b) alone cannot close the gap: no `ra` gives more than ~3.9 L m⁻² d⁻¹ at `rs` = 70 | a source giving the BPC wheat canopy's `rs` ≤ ~41 s m⁻¹ at 1000+ ppm |
| A2 | (a) alone is implausible at ×4.2: an HPS lamp's radiation outside PAR adds well under 3× the PAR energy, so the canopy's net radiation stays under ~2.5× the PAR-only figure (low confidence; from general knowledge of HPS spectra) | a source putting the BPC canopy's net radiation near 340 W m⁻², or HPS non-PAR radiant output ≥ ~3× its PAR |
| A3 | the BPC's lamps sat behind barriers that absorbed part of their infrared (low confidence) | a construction source showing bare lamps over the canopy (strengthens (a)) |
| A4 | the TM gives no way to take solution-surface evaporation out of its 6 L | a page that separates it |
| A5 | so the gap is most likely (a) and (b) together, perhaps with (c); no single candidate is named the cause | — |
| R1 | run: `aerodynamic_resistance` 50 → 10 (WHAT-IF) moves W1 to the value Penman–Monteith gives on the same forcings, to ~1e-9 | a mismatch: the substitution does not reach the flow, or something else modulates it |
| R2 | run: net radiation × 4.2 (WHAT-IF) gives W1 ≈ 6.0 (5.8–6.2), every carbon readout bit-identical, FTSW ≥ 0.30 throughout (the demand ~6 against irrigation 8 mm/day) | carbon moves, or the soil dries — a coupling finding |

Stop rule for sources: the shelf, then NTRS for Prince et al. 1987, Sager et al. 1988, Prince & Knott
1989, then one search for HPS radiant output. Paywalled or blocked → recorded as blocked, and the
row stops at the arithmetic.

### 10d. Graded (2026-10-07)

**The run** (`scorecard_tm102788.rs`, three new tests, WHAT-IF, nothing pinned):

| | predicted | measured | grade |
|---|---|---|---|
| A1 (arithmetic, now a test) | no `ra` gives more than ~3.9 L m⁻² d⁻¹ at `rs` = 70 | W1 as `ra → 0`: **3.9486** (frozen `ra` 50: 2.3762) | HELD |
| R1 | `ra` 50 → 10 moves W1 to Penman–Monteith on the same forcings, ~1e-9 | model **3.249631** = by hand 3.249631 | HELD |
| R2 | net radiation × 4.2: W1 5.8–6.2; carbon bit-identical; FTSW ≥ 0.30 | W1 **6.0541**; every CO₂ step and the end crop bit-identical; lowest FTSW **0.9257** | HELD |

**The sources** (stop rule followed: the shelf held nothing; one NTRS report and one lamp paper
read off their page images; both now on the git-ignored shelf):

* **NASA TM 103494** — Wheeler, Mackowiak, Dreschel, Sager, Prince, Knott, Hinkle & Strayer (March
  1990), *System Development and Early Biological Tests in NASA's Biomass Production Chamber*
  (`sources/wheeler_et_al_1990_TM103494.pdf`; US-Government work, public domain). ⚠ Locus: the two
  canopy measurements below are from the **Phase II crop** (May–Aug 1988: 24-h light at 660 µmol,
  23 °C, 65 % RH), not TM 102788's Phase IV crop.
  * p. 4: shoots were confined to *"60 cm of vertical growing dimension between the culture trays
    and lamp barriers"* — the lamps sat behind barriers; their material and infrared absorption
    are not stated in the pages read.
  * p. 8: *"Air velocities as measured with a hot wire anemometer typically ranged from 0.2 to 1.2
    m s⁻¹, while infrared temperatures of plant shoots typically stayed within ± 1 °C of the
    surrounding air temperature."*
  * p. 8: Phase II condensate **6.1 L m⁻² d⁻¹**, "closely represented transpiration".
  * p. 14 (Phase IV, TM 102788's crop): condensate exceeded the water added, *"likely a result of
    residual water in the air handling system from supplementary humidification during the first
    2 weeks"*; after full ground cover transpiration *"remained relatively constant"*.
* **Tazawa (1999)**, *JARQ* 33:177–183, Table 2 (`sources/tazawa_1999_JARQ33_177.pdf`; JIRCAS,
  free to read; facts cited): a 360 W clear HPS lamp emits **39.0 %** of its input as visible
  radiation (380–780 nm). It gives no infrared share, so "how much an HPS lamp radiates outside
  PAR" stays unsized from this source.

| | prediction | what the sources say | grade |
|---|---|---|---|
| A1 | (b) airflow alone cannot close the gap | arithmetic holds **at the frozen `rs` = 70** — and only there (§10e). ~~and the premise "the BPC blew air hard" is not supported: 0.2–1.2 m s⁻¹ is gentle air, which raises `ra` if anything~~ (withdrawn: `ra` = 50 is a field wind-profile value; in a chamber the air moves through the canopy, so the chamber's air speed says nothing about it here) | HELD at `rs` = 70 ~~, strengthened~~ |
| A2 | (a) at ×4.2 is implausible; non-PAR radiation adds well under 3× | not answered: Tazawa gives the visible share only. The one leaf-temperature reading counts AGAINST radiation alone (§10e) | NOT TESTED |
| A3 | lamps behind barriers | barriers existed (p. 4); their absorption unknown | HELD on existence |
| A4 | the TM gives no way to separate solution-surface evaporation | none found; the only stated contamination is early humidifier water, outside the W1 window (days 25–80) | HELD |
| A5 | (a) and (b) together, no single cause | no single cause is shown; ~~supported, in the form below~~ the form below is withdrawn (§10e) | not shown |

⚠ **The two paragraphs below were WRONG and are struck, not reworded** (caught by advisor review
the same day, after commit `bb79817` had recorded them; §10e is the correction).

~~**The energy balance the sources allow (an inference, from the Phase II crop).** Leaves within
± 1 °C of the air means little sensible heat: at `ra` = 50 s m⁻¹, 1 K is `ρcp/ra` ≈ 24 W m⁻². So the
canopy's latent heat ≈ its net radiation ± ~24. Phase II's 6.1 L m⁻² d⁻¹ under continuous light is a
latent flux of ~173 W m⁻²; its 660 µmol of PAR is 144 W m⁻² radiant, 111 net of the 0.23 albedo. So
the canopy absorbed **at least ~1.3×** the energy of its PAR alone — extra lamp radiation was there,
but a modest multiple, not 4×.~~

~~**So the ×4.2 belongs to the model's split, not to the lamp alone.** At the frozen `ra` 50 / `rs` 70
the model's Penman–Monteith sends only about half of its net radiation into evaporation
(`Δ / (Δ + γ(1 + rs/ra))` ≈ 0.47 at 20 °C) and warms the leaf with the rest, while the chamber's
leaves sat at air temperature. The gap is therefore **both**: more radiant energy than PAR (a), and
a surface/air coupling in the model that does not put it into evaporation the way the chamber's
canopy did (a resistance question — but toward a lower `rs`/`ra` ratio, not "hard airflow"). Neither
is named the cause; "not refuted" is what (a) earns, and (b) in its "airflow" form is refuted.~~

### 10e. The correction (2026-10-07, advisor review of §10d)

⚠ *Later 2026-10-07 (`log/canopy-resistance.md`):* FAO-56 Ch. 2 has since been read first-hand. Eq. 5
is scoped by its source to *"dense full cover vegetation"*, and the 0.5 and the ~100 s m⁻¹ are stated
in its grass box — so the use below, at the model's full-cover LAI 6.2, is inside the source's scope;
the "general knowledge, not read on a page" caveat in point 3 is discharged.

1. **"About half into evaporation" confused the marginal coefficient with the split.** `Δ / (Δ +
   γ(1 + rs/ra))` ≈ 0.47 is what each EXTRA watt of net radiation adds to evaporation. At the
   model's operating point the split is **74 of 81 W m⁻² ≈ 0.91** (§8). At ×4.2 the leftover is
   ~340 − 198 ≈ 142 W m⁻² of sensible heat, which puts the model's leaf **~5.8 K above the air**
   (142 × 50 / 1220.7). The Phase II reading — shoots within ± 1 °C of the air — therefore counts
   **against** the radiation-only candidate, not for it.
2. **The "at least ~1.3× PAR" bound does not stand.** Its ± 24 W m⁻² per kelvin assumed `ra` = 50, the
   very value in question (at `ra` = 10 the same 1 K allows ± 122). And Phase II's 6.1 L m⁻² d⁻¹ is on
   ~8 m² of trays (TM 103494 p. 6: *"approximately 8 m² of growing area (32 0.25-m² trays)"*), while
   TM 102788 counts its 16 m² of trays as ~20 m² with the gaps (p. 2). On that basis Phase II is ~4.9
   L m⁻² d⁻¹ (~139 W m⁻²) and the bound falls to ~1.0×. **Extra lamp radiation is not established.**
3. **The resistance candidate was framed wrongly.** The frozen `rs` = 70 is uncited (`TODO(cite)`,
   "literature-typical ~50–100 s/m") — it is not an FAO value in this tree, though it equals FAO-56's
   grass reference. FAO-56's canopy form `rs = rl / (0.5·LAI)` (`rl` ≈ 100 s m⁻¹; general knowledge,
   not read on a page here) makes it a **canopy-scale** value that falls as leaf area rises. Measured
   (printed by `the_surface_resistance_a_full_canopy_would_have`): the model's mean **LAI over TM
   days 25–80 is 6.216**, so that form gives **`rs` ≈ 32.2 s m⁻¹**, and:
   * W1 at the frozen `ra` 50 rises **2.376 → 2.853** L m⁻² d⁻¹ — part of the gap;
   * the airflow ceiling (`ra → 0`) at that `rs` is **8.59** — ABOVE the trial's ~6.0. So once `rs`
     follows leaf area, resistances alone are no longer ruled out; A1 holds only at `rs` = 70.

   This is the record's existing gap — *transpiration ignores leaf area* — showing up at full cover,
   not a physiological stretch: it needs no lower stomatal resistance at high CO₂.
4. **What stands.** A1 (at `rs` = 70), R1, R2: firm. The one leaf-temperature reading (an earlier crop,
   a different area basis) counts against radiation alone. Extra lamp radiation: not established. A
   canopy resistance that falls with leaf area: a named mechanism tied to a recorded gap, which with a
   chamber `ra` below 50 (not sourced) could close the rest. **Neither is named the cause.**

**Not done, the user's call:** pricing a leaf-area-dependent canopy resistance as a science item (lab
first; adopting it would be an unfreeze). Whether the station's own lamp has the same gap is not
measured here — this row is an HPS chamber.

---

## 11. The water gap, re-sized (2026-10-14, the user's pick: *"1"*, the water gap, read-only)

**Found before any new run: the row's trial figure was the PEAK, not the level.** W1 averages the
model over TM days 25–80 and divided by **6.0 L m⁻² d⁻¹** — the text's *peak*, 120 L day⁻¹ "near 25
days" ÷ 20 m² (p. 12). The level over those days is on Fig. 10 (p. 27). The canopy-resistance plan
(§7a there) had already normalised that figure's shape by "~85" while labelling the level "~6.0".

**Fig. 10 digitized** (the page rendered at 200 dpi; filled "condensate produced" markers found as the
blobs that survive a 5-px disc erosion; axes calibrated on the plot frame, 0–100 days by 0–150 L day⁻¹;
the legend's own marker and a 1-px speck dropped; reading error ~±3 L day⁻¹ per point). Condensate,
not "water added": the TM says condensate "directly related to water transpired" after day 12 (p. 12).
Script and image: `W:\temp\claude\water-gap\fig\` (to be kept at `sources\temp-evidence\water-gap\`).

| TM day | 25.9 | 29.1 | 31.8 | 35.0 | 38.3 | 41.0 | 44.3 | 47.0 | 50.2 | 53.3 | 56.0 | 59.1 | 61.9 | 65.0 | 68.2 | 71.0 | 74.2 | 76.9 | 80.0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| L day⁻¹ | 95.2 | 97.0 | 81.6 | 86.1 | 91.3 | 91.3 | 79.8 | 74.1 | 73.0 | 76.0 | 81.3 | 86.7 | 87.6 | 81.0 | 88.1 | 86.6 | 75.9 | 74.1 | 76.6 |

Mean of the 19: **83.3 L day⁻¹ = 4.17 L m⁻² d⁻¹** on the TM's own 20 m² (the stand with the gaps
between trays, p. 2 and p. 13), **5.21** on the 16 m² of trays. Neither basis is chosen; 20 m² is the
row's, as it is the TM's for its own 4.5 average. Cross-check: the digitized points from day 11 on
average 83.7 L day⁻¹ where the text says "about 90" — the points read ~7 % under the text (a sampled
curve against a daily mean, or reading error); the corrected level could be ~4.2–4.5.

**Advisor (2026-10-14), summarized:** report this first; digitize condensate with an error; use the
model's season-mean row (W2, model 2.735 against the TM's own 4.5 = 0.61, already like-for-like) as the
control; give both area bases; re-size every §10 argument at the corrected target AND the adopted
surface-resistance form (Szeicz–Long, `rs` = 100 / min(LAI, 2) = 50 s m⁻¹ at full cover, adopted
2026-10-10) with predictions first — the airflow ceiling may now exceed the target, which would flip
§10's "airflow alone cannot close it"; strike the stale 0.40/0.43 figures in place.

**Side note offered to the user mid-work (2026-10-14), recorded:** the trial's high-pressure sodium
lamps radiate far more heat than light; if the gap is closed with lamp heat, the model may match that
one chamber and get the station's own lamp wrong. Ask which part of the gap is the trial's lamp type
and which is the model. This item changes nothing in the model.

### 11a. Predictions (hand arithmetic with the model's own constants, written before the code)

Operating point at full cover, 480 µmol, 20 °C lit: lamp net radiation 81 W m⁻², VPD 444 Pa, Δ ≈ 145
Pa K⁻¹, `ra` 50, `rs` 50. Lit λE ≈ (145·81 + 1220.7·444/50)/(145 + 67·2) ≈ 81 W m⁻²; dark ≈ 39. Daily ≈
2.61 L m⁻² d⁻¹ — the model's 2.603 (so at full cover the leaf already sits at air temperature).

| | prediction | confidence |
|---|---|---|
| Q1 | W1 = 2.603 / 4.17 = **0.62** (20 m²), **0.50** (16 m²); close to the control W2 = 0.61 | high (arithmetic) |
| Q2 | by hand at the adopted full-cover `rs` (mean LAI 6.2 → 50) and `ra` 50, Penman–Monteith on the forcings gives W1 within 2 % of the model's 2.603 (the residual is the soil's share of the energy and its evaporation) | high |
| Q3 | **the airflow ceiling flips:** as `ra → 0` the flux tends to `ρcp·VPD/(γ·rs)`, independent of radiation: ~162 W m⁻² at 20 °C, ~126 at 16 °C → W1 ceiling **5.3–5.7**, ABOVE 4.17 (and above 5.21) | high |
| Q4 | air coupling alone, lamp PAR-only: the `ra` that gives W1 = 4.17 is **6–12 s m⁻¹** (hand: ~8–9) | medium |
| Q5 | lamp heat alone at `ra` 50: net radiation **×2.2–2.4** of the PAR-only figure gives 4.17 (§10's ×4.2 was for 6.0 at `rs` 70) | medium |
| Q6 | leaf over air, lit steps, mean over days 25–80: Q5's case **+1.7 to +2.4 K**; Q4's case **−0.2 to −0.6 K** (the leaf cooler than the air); the row as is ~0 K | medium |
| Q7 | so on the one leaf-temperature reading on the shelf (TM 103494, Phase II crop: shoots within ±1 °C of the air) the air-coupling case fits and the lamp-heat case sits outside it — still a different crop and area basis, so neither is named the cause | — |

What would refute: Q2 off by more than 2 % means the instrument's by-hand path does not describe the
adopted flow and Q3–Q6 must be re-derived from the model, not the formula.

### 11b. Measured (2026-10-14) — graded

`scorecard_tm102788.rs`: the W1 row now divides by `TM_W1_LEVEL` = 4.17 (and prints the trays basis);
A1 is renamed to say it is about the constant resistance and checks against the level; a new printed
test, `the_gap_resized_at_the_adopted_form`, carries Q1–Q6. Nothing in the model changed; no golden,
manifest or param moved. Log `W:\temp\claude\water-gap\resize.txt` (kept at
`sources\temp-evidence\water-gap\`).

| | predicted | measured | grade |
|---|---|---|---|
| Q1 | 0.62 (20 m²), 0.50 (16 m²); near W2's 0.61 | **0.6241**, **0.4996**; W2 0.6078 | HELD |
| Q2 | by hand within 2 % of the model | 2.6067 against 2.6027 (0.15 %) | HELD |
| Q3 | airflow ceiling 5.3–5.7, above 4.17 | **5.528** | HELD — §10's "airflow alone cannot close it" holds only at the constant 70 s m⁻¹ |
| Q4 | `ra` 6–12 s m⁻¹ | **8.76** | HELD |
| Q5 | net radiation ×2.2–2.4 | **×2.24** | HELD |
| Q6 | lamp heat +1.7 to +2.4 K; air coupling −0.2 to −0.6 K; as is ~0 | **+2.06 K**; **−0.30 K**; +0.04 K | HELD |

On the trays basis (5.21) the same two candidates need `ra` 1.45 s m⁻¹ (leaf −0.08 K) or net radiation
×3.07 (leaf +3.4 K).
Fully dark steps carry **0.058** of W1 as is and **0.093** in the Q4 case: the leaf-area resistance has no
light term (`transpiration.yaml`: "the leaf-area forms do not close stomata in the dark"), so a crop
closing its pores at night would need a lower `ra` than 8.76.

**What the row now says.**

1. **The model's crop drinks about 0.6 of the trial's water at full cover, not 0.4.** The gap was over-
   stated by reading the trial's best day as its typical day. The full-cover rate (0.62) and the season
   mean (0.61) now agree.
2. **On the 20 m² basis, two candidates each close what is left, alone** (on the 16 m² of trays neither
   is plausible alone: almost no air resistance at all, or leaves 3.4 K over the air — so the area basis
   bears on the answer), and they point different ways for the station:
   * **Air coupling** (a model property): the model's air resistance is 50 s m⁻¹, uncited (`TODO(cite)`,
     "wind/height-dependent") and one value for every build — the station's chambers read the same
     `transpiration.yaml` (checked 2026-10-14: no station-side override). The trial's level needs ~9. A chamber fan pushing air through a dense canopy could plausibly
     give a much lower value, but no source on the shelf gives one. If this is the cause, the station's
     chambers carry the same gap, whatever their lamp.
   * **Lamp heat** (the trial's lamp type): ×2.24 the heat of the lamp's light alone. High-pressure sodium
     lamps radiate a lot outside the visible band; how much reached this canopy through the lamp
     barriers is not on the shelf. If this is the cause, it belongs to the trial's lamps, and an LED-lit
     station would not share it — matching it in the model would be the error the side note warns of.
3. **The one temperature reading leans toward air coupling.** Under lamp heat alone the leaves would
   run ~2 K above the air; under air coupling ~0.3 K below. TM 103494's earlier crop in the same
   chamber had shoots within ±1 °C of the air. That crop had other conditions and another area basis,
   so this leans, it does not decide. **Neither is named the cause.**

**What would separate them (the user's call):** a source for the chamber's canopy air resistance or its
leaf-to-air coupling (the BPC design papers named in §10c: Prince et al. 1987, Sager et al. 1988), or
one for the radiant heat that crossed the lamp barriers. Either is a read, not a build. Changing the
model's air resistance for chambers would be a science item and an unfreeze.

**Left unedited on purpose:** the 0.40 / 0.43 in the frozen contract's prose (`docs/station-reference.md`,
the 2026-10-10 entry) — correcting a contract's prose is the user's call. Every other line carrying the
old figure is marked in place, the log index's status cell for this item included (⚠ the first
version of this paragraph said that cell was gated to the log heading; only the row's FIRST cell is).

---

## 12. The water gap's two candidates, searched (2026-10-14, the user's pick: *"1"*, read-only)

**The question.** §11 left two candidates, each closing the 20 m² gap alone: the canopy's air coupling
(`ra` ≈ 8.8 s m⁻¹ instead of the uncited 50) or extra lamp heat (net radiation ×2.24 the lamp's PAR alone).
They point different ways for the station: air coupling would carry over; the sodium lamps' heat would not.
This item searches for a source that separates them. Nothing in the model, its tests, goldens or params moves.

### 12a. What would count — stated before any source was read (advisor, 2026-10-14)

| a source reporting… | air coupling (`ra` ≈ 8.8) | lamp heat (×2.24) |
|---|---|---|
| leaf (canopy) minus air temperature, best on the Phase IV crop itself | ≈ −0.3 K | ≈ +2 K |
| lit vs dark water use | both rise; the dark share rises | only lit rises; the dark share falls (0.058 of W1 now) |
| total radiation at the canopy, or the barrier's infrared transmission | — | needs ≈ 2.2× the PAR energy |
| a canopy aerodynamic / boundary-layer conductance | needs ≈ 9 s m⁻¹ | — |

**Air speed alone does not count** (§10d withdrew the 0.2–1.2 m s⁻¹ reading). The most a candidate can earn
here is "not refuted" or "leans"; naming a cause needs a measurement on the Phase IV crop. Stop rule as §10c:
the shelf, then NTRS for Prince et al. 1987, Sager et al. 1988, Prince & Knott 1989, then one search on the
lamp barrier. Paywalled or blocked → recorded as blocked.

### 12b. What was read

Page numbers below are PDF pages.

* **The shelf, read in full this time** (both TMs by text extraction, not page samples). Nothing new for
  the Phase IV crop: TM 102788 records condensate **once a day** (*"Condensate tank volumes for both the
  upper and lower air handling systems were recorded each day and then emptied"*, PDF p. 9), so **no
  lit/dark split exists** in the trial's record; and it gives no canopy temperature. TM 103494 adds only
  what §10d had (the ±1 °C, Phase II).
* **NASA TM-2003-211184** — Wheeler et al. (2003), *Crop Production for Advanced Life Support Systems —
  Observations From the Kennedy Space Center Breadboard Project* (NTRS 20030032422; US-Government work,
  public domain; on the git-ignored shelf as `sources/wheeler_et_al_2003_TM2003-211184.pdf`):
  * p. 14: *"Lamps were separated from the plants by clear glass or acrylic barriers."* — the barrier
    material, which §10d had not found.
  * p. 14: *"air circulation of about 400 m³ min⁻¹ provided by two 30-kilowatt (kW) fans"*; 96 × 400 W HPS
    lamps on dimming ballasts, *"about 2 kW of input electrical power per m² of growing area"*.
  * p. 30, p. 33 (Table 9): water use over a crop averaged 2–5 L m⁻² d⁻¹ (wheat 4.7), peaks 5–9 *"depending
    on the species and the water vapor pressure deficit"*; p. 35: plotted against daily PAR, *"no clear
    correlations existed; this was likely due to the more influential effects of different water vapor
    pressure deficits (VPDs) among the different studies."*
* **McCree (1984)**, *Crop Science* 24:816–819, doi 10.2135/cropsci1984.0011183x002400040043x — **the
  abstract only** (via Crossref; the paper is paywalled; facts cited, not copied; the abstract text kept at
  `sources/mccree_1984_abstract.txt`). Chambers with metal halide, high-pressure sodium and low-pressure
  sodium lamps, some with glass or plastic barriers:
  * PAR-to-shortwave (280–2800 nm) ratios *"generally at or above those found in natural sunlight (about
    0.5)"* — the shortwave is at most ~2× the PAR energy;
  * without a barrier (metal halide), PAR-to-total under half that, from the bulbs' and fixtures' thermal
    radiation; *"Leaf temperatures were as much as 16 °C above air temperature."*;
  * *"A plate glass barrier removed 75% of the thermal radiation, while transmitting 81% of the
    photosynthetic radiation and 74% of the shortwave radiation. With the barrier, leaf temperatures were
    close to air temperature."*; barriers *"essential"* with metal halide or HPS lamps.
* NTRS 20190027336 (2019, a history of the chamber): restates the glass-or-acrylic barriers; nothing else.

**Blocked:** Prince et al. 1987 (SAE 871437 — NTRS record 19880033873, no download); Sager et al. 1988 (ASAE
88-4018 — not on NTRS); Prince & Knott 1989 (NTRS record 19910030115, no download); Bugbee, Monje & Tanner
1996 (*Adv. Space Res.* 18:149 — sensors for canopy temperature and air velocity) and Monje & Bugbee 1996
(*Acta Hort.* 440:123) — USU Digital Commons records with no file; McCree 1984's full text (paywalled).
McCormack et al. 1994 (AIAA 94-4573, *"A method for modeling transpiration in CELSS"*) — not found on NTRS.

### 12c. The lamp candidate, sized against McCree (the model's own constants)

The model's lamp net radiation is `(1 − 0.23) · PAR / 4.57`: **80.9 W m⁻²** at 480 µmol. If every watt of
the lamp's shortwave were absorbed at the same 0.23 albedo (generous: canopies reflect more of the near
infrared than of PAR — general knowledge, not on a page here), McCree's ratios cap the shortwave reaching
the canopy at **1.83×** the PAR energy (≤ 2× at the bulb, through a glass barrier passing 74 % of the
shortwave and 81 % of the PAR) to **2.0×** (if his ratio was measured where the plants sat). So ×2.24 is
**more than all the shortwave**: the remaining **19–33 W m⁻²** would have to be heat radiation from or
through the barrier ~~— the part McCree found a glass barrier cuts by 75 %, leaving leaves near air
temperature~~ (struck: the 75 % cut does not make the remainder small — §12e). On the trays basis (×3.07)
the remainder is **86–100 W m⁻²**.

### 12d. Graded

| criterion | what was found | reads toward |
|---|---|---|
| leaf − air temperature, Phase IV | **none** — TM 102788 gives no canopy temperature | — (blocked) |
| leaf − air temperature, other | Phase II crop ±1 °C (TM 103494, §10d); McCree: with a glass barrier, leaves close to air | **leans against lamp heat alone** (+2 K predicted) — ⚠ at the model's `ra` only, not against extra radiation as such (§12e); does not separate air coupling from "lamp heat plus air coupling" |
| lit vs dark water use | **none** — condensate logged daily | — (blocked by the record's design) |
| total radiation / barrier IR | BPC: glass or acrylic barriers; McCree: shortwave ≤ ~2× PAR, glass removes 75 % of thermal | ~~**strains lamp heat alone**~~ **unsized by this source** (§12e): ×2.24 exceeds all the shortwave and needs 19–33 W m⁻² of residual heat radiation (§12c); one reading of the same abstract lets ~60 through |
| canopy conductance | **none** — the two Bugbee/Monje papers that may hold one are blocked | — |
| (not in the table) water use vs light vs VPD | TM-2003: no clear correlation with daily PAR; VPD *"likely"* more influential — across species and studies, the authors' inference | leans toward air coupling, weakly: a well-coupled canopy's water use follows VPD, a poorly coupled one's follows radiation (Penman–Monteith; the reading is ours) |

**What this item says.**

1. ~~**Lamp heat alone is strained, not refuted.**~~ **Lamp heat is neither ruled in nor out by this source
   (§12e).** By McCree's ratios the barrier-filtered shortwave cannot supply ×2.24 on its own; it would need
   every shortwave watt absorbed and ~20–33 W m⁻² of heat radiation besides~~, which a glass barrier is
   reported to cut to near-air leaf temperatures~~ — and the barrier may pass more than that. McCree's
   chambers are not the BPC, his HPS-specific numbers are not in the abstract, and the BPC's acrylic option
   is unread.
2. **Air coupling is not refuted, and leans slightly ahead** — on the leaf temperatures and on the BPC's
   water use tracking air dryness rather than light. But **no source gives the chamber's canopy air
   resistance**; the papers that may (Bugbee/Monje 1996) are blocked.
3. **The trial's own record cannot separate them**: it has neither a canopy temperature nor a lit/dark
   water split. A cause can only be named from another source or another trial.
4. **Most likely some of each** — some extra radiation through the barrier, a lower air resistance — which
   the model can only represent by both. **Neither is named the cause.**

**For the station:** if air coupling carries most of it, the station's lamp-lit chambers (which read the
same uncited `ra` = 50 from `transpiration.yaml`) under-transpire too. Changing that value is a science
item and an unfreeze — **not priced here; the user's call.** The next read that could decide it is one of
the blocked Bugbee/Monje papers (a library copy), or Bugbee & Salisbury's chamber work if it reports canopy
temperatures (unchecked).

### 12e. The correction (2026-10-14, advisor review of §12a–d, after commit `5290bf1`)

⚠ `5290bf1`'s headline (*"lamp heat alone strained"*) is wrong and stays in history, as `bb79817`'s did;
the struck text above is the record of it.

1. **The 75 % cut was read as making the barrier's leftover heat radiation small. The same abstract does
   not allow that.** For the metal-halide chamber without a barrier, PAR/shortwave ≈ 0.5 and PAR/total
   < ~0.25, so thermal radiation was **> ~2×** the PAR energy. A glass barrier leaving 25 % of it leaves
   **> ~0.5×** the bulb-side PAR — **> ~0.62×** the PAR at the canopy (÷ 0.81), **~65 W m⁻²** at 480 µmol, ~62
   absorbed at a leaf's ~0.95 thermal absorptance (general knowledge). That is **above** the 19–33 W m⁻²
   the candidate needs, not below.
2. **Two things the abstract does not say:** whether that ratio holds for HPS (it is given for metal halide
   only), and whether "thermal" is the excess over the room's own radiation or the absolute downward flux.
   Read as absolute, a 75 % cut is not physical (the barrier itself radiates at least at room temperature),
   so read either way the abstract **bounds the residual in neither direction**.
3. **So the radiation row is unsized by this source.** What remains against lamp heat alone is the leaves
   near air temperature (BPC Phase II ±1 °C; McCree's chambers with a barrier) — evidence against lamp heat
   **at the model's air resistance** (+2 K), not against extra radiation as such. If McCree's barrier
   chambers did carry ~60 W m⁻² of residual heat radiation with leaves near air temperature, their canopies
   were well coupled to the air — which reads toward **both** candidates acting together. That is our
   inference from an abstract, not a finding.
4. **What stands:** the trial's record cannot separate the candidates (12b); air coupling is not refuted;
   the BPC's water use tracked air dryness more than light (a weak lean toward coupling); the sources that
   would decide it are blocked. **Neither is named the cause.**

---

## 13. The water gap with the air resistance kept at 50 (2026-10-14, the user's pick: *"use 50. work on 2"*)

**The user's two calls.** The chamber air-resistance price (`post-roadmap-chamber-air-resistance.md`) is
closed: every room keeps the uncited `ra` = 50 s m⁻¹, nothing adopted. Then: work on the water gap — why the
model's crop drinks 0.62 of TM 102788's water at full cover (§11).

**What this item can and cannot deliver, stated first.** Keeping 50 is a decision about the model, not
evidence about the trial: air coupling stays a candidate cause there (§12). This item is a **diagnosis** and
changes nothing in the model, its goldens or its params. If air coupling is the cause, the station's chambers
keep their share of the gap whatever this finds.

### 13a. Advisor review (2026-10-14), summarized

Close the price first (done, commit `f13ad3c`). Then the check that decides what this item can end in: at the
kept `ra` 50, can any surface resistance `rs` ≥ 0 reach the trial's level? (Advisor's arithmetic: `rs` = 0
gives ~107 W m⁻² lit against ~130 needed — to be verified, not taken.) If not, the only lever left INSIDE the
model is extra radiation — the sodium-lamp heat §11's side note warned would fit this trial and get the LED
station wrong — and keeping 50 must not be read as pushing the diagnosis that way. Next the forcing side,
never checked: the TM's MEASURED temperatures (20.36 °C light, 16.8 °C dark) and humidities (81 % / 82 %)
against the row's set points (20 / 16 °C, 81 % both), and the PAR-to-energy factor (4.57 µmol J⁻¹ is
daylight's; a sodium lamp's is nearer 5). Predictions first. Then a FRESH source search — not the §12b blocked
list again: Bugbee & Salisbury 1988 (*Plant Physiol.* 88:869), full texts of the Bugbee/Monje work on Utah
State's own lab pages, any BPC wheat paper giving canopy-minus-air temperature — graded against §12a's table
as written, same stop rule.

### 13b. The forcings, checked against the TM (read before predicting)

TM 102788 PDF p. 5 (text extraction): *"For the first 34 days after planting, air temperature and humdity were
held constant at 20 °C and 80% relative. After 34 days, temperature set points were switched to 20 °C in the
light and 16 °C in the dark. … Throughout the study, temperatures averaged 20.36 °C (± 0.4 °C difference
between levels) for the 20-h light cycle, and 16.8 °C (± 0.5 °C) for the dark cycle. Humidities averaged 81% ±
4% and 82% ± 6% for the light and dark cycles."* And PPF *"throughout the study averaged 534 µmol m⁻² s⁻¹"* —
consistent with the row's 695 → 480 dimming on day 28 (time-weighted over TM days 3–86: ~545).

The row uses the set points (20.0 / 16.0 °C) and 81 % in both periods. ⚠ "Throughout the study" for the dark
mean cannot include days 3–33 at a constant 20 °C AND average 16.8 unless the later nights ran ~14.7 °C; the
reading taken here is that the dark mean is over the cool-night period, applied from day 34 only.

The lamp's light-only energy: `station::lighting::lamp_net_radiation` = `(1 − 0.23) · PAR / 4.57`, the
`4.57 µmol J⁻¹` being McCree's daylight factor (`weather.rs` `PAR_UMOL_PER_J`). A high-pressure sodium lamp's
400–700 nm photons carry less energy each on average (its output sits near 590 nm); its factor is ~4.9–5.0
µmol J⁻¹ — **general knowledge, not on a page here** (to look for in the §13d search). Used as a sensitivity
at 4.98, not a correction.

### 13c. Predictions (hand arithmetic, the §11a operating point; written before any code)

Lit: Δ 145 Pa K⁻¹, Rn 81 W m⁻², VPD 444 Pa, ρcp 1220.7, γ 67, `ra` 50, full-cover `rs` 50 (Szeicz–Long). Dark
(16 °C): Δ ~112, VPD 345, Rn 0.

| | prediction | confidence |
|---|---|---|
| K1 | **No `rs` ≥ 0 reaches 4.17 at `ra` 50.** `rs` → 0: lit λE (145·81 + 1220.7·444/50)/(145 + 67) ≈ 107 W m⁻² (×1.31 on today's 81), dark ≈ 47 (×1.38): W1 ceiling **3.3–3.5** L m⁻² d⁻¹, **0.79–0.84** of the level. On the trays basis (5.21) further still | high |
| K2 | so at `ra` 50 the gap closes in the model only with **extra net radiation** (Q5's ×2.24 stands) or `rs` below 0 — no physical leaf setting does it | high (follows K1) |
| K3 | the TM's measured temperatures and humidities (20.36 / 16.8 °C from day 34; 81 / 82 %) move W1 by **0 to +2 %**; the ratio stays **0.62–0.63** | medium |
| K4 | a sodium-lamp factor of 4.98 µmol J⁻¹ (Rn × 0.918) moves W1 **−3 to −5 %** (ratio ~**0.60**) and raises Q5's multiple from ×2.24 to **×2.4–2.5** of the lamp's (corrected) light-only energy | medium |
| K5 | together: ratio **0.59–0.62**. The forcing side does not explain the gap; checking it is the point, closing it is not expected | medium |

What would refute: K1 — the by-hand ceiling at `rs` → 0 reaching 4.17 (then a leaf-side cause stays open at
50). K3/K4 — a shift outside the stated band in either direction means the arithmetic misses a term, and the
by-hand path is re-derived before reading anything from it.

Printed, never asserted, in `rust/crates/station/tests/scorecard_tm102788.rs`. No model, golden, param or
manifest change.

### 13d. K1–K5 measured (2026-10-14) — graded

`the_gap_at_the_kept_air_resistance` (printed, never asserted). The by-hand path reproduces `w1_by_hand`
exactly at the set points (2.6067), 0.15 % from the model's 2.6027 as in §11b.

| | predicted | measured | grade |
|---|---|---|---|
| K1 | `rs` → 0 at `ra` 50: W1 3.3–3.5, 0.79–0.84 of 4.17 | **3.4414**, **0.825** (0.661 of 5.21) | HELD — **no leaf setting closes the gap at `ra` 50** |
| K2 | so only extra net radiation (or `rs` < 0) closes it in the model | follows from K1 | HELD |
| K3 | measured temperatures/RH: W1 0 to +2 %, ratio 0.62–0.63 | **+0.86 %**, **0.630**; net radiation for 4.17 ×2.21 | HELD |
| K4 | sodium factor 4.98: W1 −3 to −5 %, ratio ~0.60, multiple ×2.4–2.5 | **−3.98 %**, **0.599**, **×2.44** | HELD |
| K5 | together: ratio 0.59–0.62 | **0.604** (×2.41) | HELD |

**What the forcing side says:** the row's set points were close enough — the measured temperatures add under
1 %, and a sodium lamp's photon energy takes ~4 % away. The gap is not in the forcings, and with the air
resistance kept at 50 the model can only close it with lamp heat of ×2.4 its light-only energy.

### 13e. The fresh search (2026-10-14) — what was read

* **Bugbee & Salisbury 1988** (*Plant Physiol.* 88:869–878, PMC1055676): free to read, but every route
  (PMC, Europe PMC, USU Digital Commons — "not available here") answered with an automated bot check, not a
  file. **Blocked from here; the user can fetch it by hand.** Its PMC abstract and reference list say nothing
  on canopy temperature or water use.
* **Utah State lab pages** for the 1996 Bugbee/Monje papers: no file found (the CV only lists them).
* **BPC papers giving canopy-minus-air temperature:** one search found none (a 1994 contractor report gives
  BPC *air* temperatures only).
* **FOUND — Monje & Bugbee (2019)**, *Radiometric Method for Determining Canopy Stomatal Conductance in
  Controlled Environments*, *Agronomy* 9(3):114, doi 10.3390/agronomy9030114 — open access, CC BY 4.0, by the
  authors of the blocked 1996 papers, measuring the same quantity; on the git-ignored shelf as
  `sources/monje_bugbee_2019_agronomy9_114.pdf`. ⚠ **Locus: not the BPC and not TM 102788's crop.** Two ~1 m²
  EGC-13 chambers at Utah State (86 kPa, 1460 m), USU-Apogee wheat at 1100 plants m⁻², leaf area index > 15,
  18–35 days old, 1600 µmol m⁻² s⁻¹ from four 1000 W **high-pressure sodium** lamps above a glass box of
  chilled water (the "water filter"), 21 °C, RH 68–70 % lit, 20 h light. What it gives (PDF page numbers):
  * p. 5 (2.4): fan settings 0.8 / 1.7 / 2.3 m s⁻¹ above the canopy; most measurements at 1.7. (BPC: 0.2–1.2
    m s⁻¹, TM 103494 p. 8.)
  * p. 6 (2.6): PAR converted to energy *"assuming 5 µmol m⁻² s⁻¹ per W m⁻² for HPS lamps"* — an assumption
    in a peer-reviewed paper, which puts K4's 4.98 inside the range a working group uses; still not a
    measurement.
  * p. 7 (2.7): *"The PARabs to Rnet ratio was 83% of Rnet in a chamber with a water filter below the lamps,
    but was only 64% of Rnet when the water filter was removed"*; draining the filter *"added ~100 W m⁻² to
    Rnet, as the glass temperature … reached 45 °C"*. Evaporation from the covered solution: *"small (~2% of
    Rnet)"* (bears on §10c A4).
  * p. 19 (Discussion): *"The gA for the wheat (5.5 mol m⁻² s⁻¹ or 7.5 s m⁻¹) … were within the range of
    typical aerodynamic conductances of field crops (… 4–13 s m⁻¹)"*. ⚠ The paper's 7.5 s m⁻¹ converts at
    sea-level molar volume; at its own 86 kPa and 21 °C the same molar conductance is **6.4 s m⁻¹**. Both are
    carried.
  * Table 3 (p. 17): wheat at constant light and `gA`, CO₂ 390 → 1230 µmol mol⁻¹: canopy stomatal conductance
    2.15 → 1.03 mol m⁻² s⁻¹ (−52 %), transpiration 11.2 → 7.0 mmol m⁻² s⁻¹ (−37 %), latent heat 460 → 288 W
    m⁻², decoupling coefficient 0.65 → 0.39.
  * p. 13 (3.3, Fig. 6; the text is overprinted in the PDF, read as best it allows): canopy minus air rose
    linearly with light, **0 → +4 °C at the low fan setting and −1 → +2 °C at the high one** over 0–1700 µmol.

### 13f. The source's numbers carried into the row — predictions (before the run)

Transplanting is the weak step: another chamber, three times the light, a denser and shorter crop, faster
air. So these are *what the row would do if the BPC were like the USU chamber*, never a fit.

**Net radiation in the model's units.** The model's lamp net radiation is `0.77 · PAR/4.57`. The source's is
`PARabs / ratio`, with PARabs = capture × PAR/5 and capture ~0.90–0.95 (visible reflection 0.055, p. 6). So
the multiple on the model's figure is `1.187 · capture / ratio`: **×1.3–1.4 with the water filter** (ratio
0.83) and **×1.7–1.8 without** (0.64). The BPC's lamps sat behind plain glass or acrylic (§12b), nearer the
no-water case.

| | prediction | confidence |
|---|---|---|
| S1 | air coupling at the source's value, lamp light-only: `ra` 7.5 → W1 **4.3–4.6**; `ra` 6.4 → **4.5–4.8** — above the 4.17 level, below the 5.21 trays figure | medium |
| S2 | lamp heat at the source's ratios, `ra` 50: ×1.35 → W1 **2.9–3.1**; ×1.75 → **3.2–3.5** — short of 4.17 alone | medium |
| S3 | both (`ra` 6.4–7.5 with ×1.35–1.75): W1 **4.5–5.1**, between the two area bases; at low `ra` extra radiation adds little (the marginal share falls to ~0.2) | medium |
| S4 | leaf over air on lit steps at `ra` 7.5 with ×1.75: **−0.1 to +0.3 K** — inside the source's high-fan band at this light (−1 → +2 °C over 0–1700 µmol gives ~−0.2 K at 480) | low |

What would refute: S1 below 4.17 means the source's chamber value would NOT close the gap by itself; S2 at or
above 4.17 means the source's lamp heat alone would.

### 13g. S1–S4 measured (2026-10-14) — graded

`the_gap_at_a_measured_chamber_coupling` (printed, never asserted; the model's full-cover `rs` 50 throughout —
the source's own canopy stomatal resistance, ~34–40 s m⁻¹ at 1230 µmol mol⁻¹ but at 1600 µmol of light, is
not carried).

| case | W1 | of 4.17 (20 m²) | of 5.21 (trays) | leaf − air |
|---|---|---|---|---|
| `ra` 7.5, light only | 4.303 | **1.03** | 0.83 | −0.28 K |
| `ra` 6.4, light only | 4.432 | **1.06** | 0.85 | −0.26 K |
| `ra` 50, ×1.35 | 3.048 | 0.73 | 0.59 | +0.61 K |
| `ra` 50, ×1.75 | 3.552 | 0.85 | 0.68 | +1.26 K |
| `ra` 7.5, ×1.35 / ×1.75 | 4.489 / 4.702 | 1.08 / 1.13 | 0.86 / 0.90 | −0.14 / +0.02 K |
| `ra` 6.4, ×1.35 / ×1.75 | 4.599 / 4.789 | 1.10 / 1.15 | 0.88 / 0.92 | −0.13 / +0.01 K |

| | predicted | measured | grade |
|---|---|---|---|
| S1 | `ra` 7.5 → 4.3–4.6; 6.4 → 4.5–4.8; above 4.17, below 5.21 | 4.303; **4.432** | HELD on the claim (both above 4.17, below 5.21); the 6.4 value 0.07 under its band — the hand estimate ran high |
| S2 | ×1.35 → 2.9–3.1; ×1.75 → 3.2–3.5; short of 4.17 | 3.048; **3.552** | HELD on the claim; ×1.75 0.05 over its band |
| S3 | 4.5–5.1, between the two bases | 4.489–4.789 | HELD (the lowest case 0.01 under) |
| S4 | `ra` 7.5 ×1.75: −0.1 to +0.3 K | +0.020 K | HELD |

### 13h. What §13 says

1. **With the air resistance kept at 50, no leaf setting closes the gap (K1)** — even leaves with no
   resistance at all reach 0.83 of the trial. Inside the model only extra lamp heat (×2.4 the lamp's
   light-only energy, sodium-corrected) would. That is a statement about the model's choice, not the trial's
   cause.
2. **The forcings were not the gap (K3–K5):** the TM's measured temperatures add under 1 %; a sodium lamp's
   photon energy removes ~4 %. The ratio is 0.60–0.63 whichever is used.
3. ⚠ *Overstated — corrected in §13i–§13j ("reaches about the level", and "some of each"):* **A measured chamber value now exists, and it leans the trial's gap toward air coupling.** In an HPS-lit
   wheat chamber at Utah State (Monje & Bugbee 2019), the canopy's air resistance was **6.4–7.5 s m⁻¹**, and
   the lamps' extra heat put net radiation at **×1.3–1.8** the model's light-only figure. Carried into the row:
   the measured air coupling **alone closes the 20 m² level** (1.03–1.06); the measured lamp heat **alone does
   not** (0.73–0.85); both together give 1.08–1.15 of the 20 m² level and 0.86–0.92 of the trays figure. At the
   measured coupling the leaf sits within ±0.3 K of the air — inside the source's own high-fan band.
4. **Not named the cause.** The source is another chamber: three times the light, faster air (1.7 m s⁻¹
   against the BPC's 0.2–1.2), a shorter and far denser crop. The BPC's slower air would raise its `ra`, by an
   amount this source does not give. "Leans toward air coupling, with some lamp heat" is what it earns.
5. **For the station (the user's decision stands, and is not re-opened here):** the price closed on
   2026-10-14 noted that *"no value below 50 has a source"*. That is no longer quite true: a peer-reviewed
   chamber measurement puts a ventilated wheat chamber's canopy air resistance near 7 s m⁻¹, inside the 4–13 s m⁻¹
   it quotes for field crops (its ref. 46, unread here) — a range the model's 50 sits above (for the open field
   too; FAO-56's grass reference gives ~100 s m⁻¹ at 2 m s⁻¹ wind, so field values vary with crop height and
   wind, and nothing here bears on the open field). Whether that changes the
   decision is the user's call; adopting any value is still a science item and an unfreeze.

**Blocked and left:** Bugbee & Salisbury 1988 (bot check; free, fetchable by hand), the two 1996 papers (no
file). The trial's own record still cannot separate the candidates (§12b).

### 13i. The correction (2026-10-14, advisor review of §13d–h, after commit `67fa625`)

⚠ `67fa625`'s headline — the measured coupling *"closes it by air coupling alone"* — **overstates what §13g
shows**, and stays in history as `5290bf1`'s did. Three reasons:

1. **The 1.03–1.06 is inside the trial figure's own error.** §11 found the digitized points ~7 % under the
   TM's text and allowed a level of 4.2–4.5. At 4.5, `ra` 7.5 gives 0.96 and `ra` 6.4 gives 0.98: the
   measured coupling **reaches about the level**; it does not close it with room to spare.
2. **The source's air was faster than the BPC's.** Its 7.5 s m⁻¹ is at the 1.7 m s⁻¹ fan setting; the BPC ran
   0.2–1.2 m s⁻¹, at or under the source's LOW setting (0.8). Slower air raises `ra` by an amount the source
   does not give, so 7.5 is a lower bound for the BPC, not an estimate.
3. **S4's leaf-temperature check used the wrong band.** Against the source's LOW-fan line (0 → +4 °C over
   0–1700 µmol, ≈ **+1.1 K** at 480) — the one nearer the BPC's air — the lamp-heat-alone cases (+0.6 / +1.3 K)
   fit as well as the coupled ones (≈ 0 K). At BPC-like air the leaf temperatures do not lean toward coupling.

**What stands:** the CONTRAST — at the source's figures, its coupling reaches about the level while its lamp
heat alone reaches 0.73–0.85. **The check that sets the wording** (no wind-scaling law needed): solve for the
`ra` that reaches 4.17, and 4.5, at each lamp-heat case. If a combined case closes at an `ra` well above 7.5,
the honest line is §12d point 4's: *some of each, in any chamber slower than the source's.*

**Predictions** (interpolated from §13g and §11b's 8.76; written before the run):

| | `ra` reaching 4.17 | `ra` reaching 4.5 |
|---|---|---|
| R1 light only (×1.0) | 8.76 (§11b) | **5–6.5** |
| R2 water-filter heat (×1.35) | **10–13** | **7–8.5** |
| R3 glass-barrier heat (×1.75) | **13–18** | **9–11** |

So the prediction is: at the glass-barrier heat, a canopy coupled about **half as well** as the source's (ra
roughly twice 7.5) still reaches the 20 m² level. That is "some of each", not "coupling alone".

### 13j. R1–R3 measured (2026-10-14) — graded, and what §13 now says

`the_air_resistance_each_lamp_heat_needs` (printed, never asserted):

| | `ra` reaching 4.17 (predicted) | `ra` reaching 4.5 (predicted) | grade |
|---|---|---|---|
| R1 light only | **8.76** (8.76) | **5.86** (5–6.5) | HELD |
| R2 ×1.35 | **11.43** (10–13) | **7.39** (7–8.5) | HELD |
| R3 ×1.75 | **17.54** (13–18) | **10.53** (9–11) | HELD |

Leaf over air in every case: −0.30 to +0.23 K.

**What §13 says, corrected (replaces §13h point 3's "closes it by air coupling alone"):**

* **Firm:** at the kept `ra` 50 no leaf setting closes the gap (K1), and the forcings move it under 4 %
  (K3–K5).
* **The contrast:** at the source's own figures, its air coupling reaches **about** the trial's level (0.96–1.06
  across the level's 4.17–4.5 band), and its lamp heat alone reaches **0.73–0.85**.
* **Some of each:** with the source's glass-barrier lamp heat (×1.75 — the BPC's barriers were glass or
  acrylic), a canopy coupled about **half as well** as the source's (`ra` ~10.5–17.5 against its 6.4–7.5)
  still reaches the level. The BPC's slower air would couple its canopy less well than the source's by an
  unknown amount, so §12d point 4 stands as the honest line: **most likely some of each — a canopy much
  better coupled than the model's 50, plus some lamp heat. No cause named.**
* **For the model:** every combination that reaches the trial needs `ra` far below 50 (at most ~17.5 even
  with the source's full glass-barrier heat). Keeping 50 is the user's decision and stands; this is what it
  leaves unexplained.
