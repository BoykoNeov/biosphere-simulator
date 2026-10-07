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
| W1 transpiration, TM days 25–80 (L m⁻² d⁻¹) | 2.376 | ~6.0 (peak; then roughly constant) | 0.396 | rate |
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
es(T) × 0.19 = 444 Pa at 20 °C, 345 Pa at 16 °C. The trial's full-cover 6.0 L m⁻² d⁻¹ is a daily
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
| A1 | (b) airflow alone cannot close the gap | arithmetic holds; and the premise "the BPC blew air hard" is **not supported**: 0.2–1.2 m s⁻¹ is gentle air, which raises `ra` if anything | HELD, strengthened |
| A2 | (a) at ×4.2 is implausible; non-PAR radiation adds well under 3× | **not answered by the infrared figure** (Tazawa gives visible only). But see the energy balance below: the ×4.2 is the MODEL's need, not the lamp's | NOT TESTED as written; reframed |
| A3 | lamps behind barriers | barriers existed (p. 4); their absorption unknown | HELD on existence |
| A4 | the TM gives no way to separate solution-surface evaporation | none found; the only stated contamination is early humidifier water, outside the W1 window (days 25–80) | HELD |
| A5 | (a) and (b) together, no single cause | supported, in the form below | HELD |

**The energy balance the sources allow (an inference, from the Phase II crop).** Leaves within
± 1 °C of the air means little sensible heat: at `ra` = 50 s m⁻¹, 1 K is `ρcp/ra` ≈ 24 W m⁻². So the
canopy's latent heat ≈ its net radiation ± ~24. Phase II's 6.1 L m⁻² d⁻¹ under continuous light is a
latent flux of ~173 W m⁻²; its 660 µmol of PAR is 144 W m⁻² radiant, 111 net of the 0.23 albedo. So
the canopy absorbed **at least ~1.3×** the energy of its PAR alone — extra lamp radiation was there,
but a modest multiple, not 4×.

**So the ×4.2 belongs to the model's split, not to the lamp alone.** At the frozen `ra` 50 / `rs` 70
the model's Penman–Monteith sends only about half of its net radiation into evaporation
(`Δ / (Δ + γ(1 + rs/ra))` ≈ 0.47 at 20 °C) and warms the leaf with the rest, while the chamber's
leaves sat at air temperature. The gap is therefore **both**: more radiant energy than PAR (a), and
a surface/air coupling in the model that does not put it into evaporation the way the chamber's
canopy did (a resistance question — but toward a lower `rs`/`ra` ratio, not "hard airflow"). Neither
is named the cause; "not refuted" is what (a) earns, and (b) in its "airflow" form is refuted.

**Not done, the user's call:** whether the station's own lamp (a PWM-dimmed LED in the record,
mostly PAR) has the same gap is not measured here — this row is an HPS chamber. A lower `rs` at high
CO₂ would go against the expected stomatal response, so a resistance change would need a source of
its own.
