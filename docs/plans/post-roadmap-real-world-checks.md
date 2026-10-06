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
