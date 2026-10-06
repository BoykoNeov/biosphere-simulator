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
