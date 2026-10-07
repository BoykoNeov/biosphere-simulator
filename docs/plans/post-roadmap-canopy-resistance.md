# Post-roadmap: a canopy surface resistance that reads leaf area — lab measurement first

**Opened 2026-10-07 on the user's call** (*"Leaf-area resistance"*), chosen after Step 6's water-gap
split (`post-roadmap-real-world-checks.md` §10e), which found that the crop's frozen surface
resistance is a constant 70 s m⁻¹ whatever its leaf area — the record's standing gap *"transpiration
ignores leaf area"* (`post-roadmap-leaf-shedding.md` §12; `post-roadmap-room-temperature.md` §21).

**The order the user set:** find a source for the form, measure it in the lab first, then decide.
Adopting a form would move frozen values — an unfreeze with its own ceremony, not part of this
item. **Nothing here is frozen or adopted.**

---

## 1. Advisor review (2026-10-07), summarized

Re-read the FAO-56 text first-hand rather than through a summary. The two sources differ in TWO
places (below the plateau by a factor of 2, and at full cover), and plugging FAO's leaf value into
Teh's form is a composite of two sources. Teh's leaf resistance is the Jarvis light-dependent one,
which closes stomata in the dark; wheat coefficients are not on the shelf, so that part is left out
and the dark share of water use is reported. Teh's threshold LAI is his own 4.0, never derived from
the model's peak LAI. Lab-only, through the existing form-switch pattern, with the values in lab
code and not in `transpiration.yaml`. Read LAI through the crop's own leaf-area read. Find every
place the flow is built. Measure where the form bites (droughts, the sealed chambers), not on
unstressed runs where carbon cannot move. Enumerate the water readers before predicting which
goldens a later adoption would move. Name the open field's wetter early soil as a consequence, not
a win. Use TM 102788 for the curve's SHAPE, not its level. Instrument checks and one mutation.

## 2. The sources, read first-hand

**FAO-56** (Allen, Pereira, Raes & Smith 1998, *Crop evapotranspiration*, FAO Irrigation and
Drainage Paper 56), Chapter 2, read from the raw page `https://www.fao.org/4/x0490e/x0490e06.htm`
(downloaded 2026-10-07; FAO's site, free to read; facts cited, no text copied into code):

* Eq. 5, `rs = rl / LAI_active`, is introduced as *"An acceptable approximation to a much more
  complex relation of the surface resistance of **dense full cover vegetation**"*. The same paragraph:
  *"Where the vegetation does not completely cover the soil, the resistance factor should indeed
  include the effects of the evaporation from the soil surface."* **So FAO's form is scoped to full
  cover; it says nothing licensing its use on a seedling.**
* `rl` *"bulk stomatal resistance of the well-illuminated leaf"*; *"This resistance is crop specific
  and differs among crop varieties and crop management. It usually increases as the crop ages and
  begins to ripen. There is, however, a lack of consolidated information on changes in rl over time
  for the different crops."*
* Box 5 (the grass reference): *"LAI_active = 0.5 LAI which takes into consideration the fact that
  generally only the upper half of dense clipped grass is actively contributing"*; *"The stomatal
  resistance, rl, of a single leaf has a value of about 100 s m⁻¹ under well-watered conditions"*;
  `LAI = 24 h` for clipped grass at h = 0.12 m → LAI 2.88 → `rs` = 70 s m⁻¹. **The 0.5 and the 100
  are stated in the grass box.** (The frozen `rs` = 70 is uncited — `TODO(cite)`, "literature-typical
  ~50–100" — and only equals FAO's grass value.)

**Teh** (*Introduction to Mathematical Modeling of Crop Growth*, on the shelf), §4.6, book p. 97–98,
read off the page image: Eq. 4.80 after Szeicz & Long (1969) — canopy resistance
`rc = rst / L` for `L ≤ 0.5 Lcr`, and `rc = rst / (0.5 Lcr)` for `L > 0.5 Lcr`, *"where Lcr is the
threshold leaf area index, often taken as the maximum L the plant can achieve. In this book, we will
take Lcr as 4.0, a typical maximum leaf area index."* `rst` is the single-leaf stomatal resistance of
Eq. 4.79 (Jarvis 1976), `rst = (a1 + I_PAR)/(a2 · I_PAR)`, with coefficients for sunflower and maize
only — none for wheat.

## 3. The two lab forms

| | FAO-56 Eq. 5 + Box 5 | Teh Eq. 4.80 (Szeicz & Long) |
|---|---|---|
| form | `rs = 100 / (0.5 · LAI)` | `rs = 100 / min(LAI, 0.5 · 4.0)` |
| at LAI 0.03 (seedling) | 6 667 s m⁻¹ — outside FAO's stated scope | 3 333 |
| at LAI 1.0 (a sealed chamber's peak) | 200 | 100 |
| at LAI 2.86 / 1.43 | 70 (= frozen) | 70 at LAI 1.43 |
| at LAI 6.2 (full cover) | 32 | 50 (the plateau from LAI 2.0) |
| its leaf value | FAO's 100 (the grass box) | FAO's 100 — **a composite**: Teh's own `rst` is the light-dependent Jarvis form, which is left out (no wheat coefficients) |

Both forms are lab-only. The frozen constant form stays the loader's value. The values (100, 0.5,
4.0) live in lab code, not in `transpiration.yaml` — a param-file edit would force a manifest
regeneration. At LAI 0 both give an infinite resistance and so zero transpiration (the
Penman–Monteith denominator diverges; the flux is exactly 0, finite).

**Where it is built:** one production site, `domains::biosphere::system` (`Transpiration { … }` in
`build_season_with`); the station's air split and minute-step paths move that same flow by id, so a
form carried in the flow travels with it. LAI is read the way the crop reads it (derived from leaf
carbon, or the lab leaf form's state).

## 4. Measured before predicting (2026-10-07)

A throwaway probe (`W:\temp\claude\step6\zz_probe_lai.rs`, run once, not committed) read LAI over
one season under the frozen build:

| scenario | LAI day 0 / 30 / 90 / 150 / 200 / 250 / 300 | peak | share of steps with LAI < 2.86 (FAO `rs` > 70) / < 1.43 (Teh `rs` > 70) / < 0.5 |
|---|---|---|---|
| default open field | 0.03 / 0.20 / 0.40 / 0.67 / 5.33 / 5.99 / 3.40 | 6.18 | 0.61 / 0.57 / 0.41 |
| deep water (1 mm/day) | same to day 200; 5.75 / 1.92 | 6.18 | 0.68 / 0.57 / 0.41 |
| sealed chamber | 0.03 / 0.33 / 0.60 / 0.89 / 1.00 / 1.02 / 0.35 | 1.02 | 1.00 / 1.00 / 0.26 |
| perennial chamber | … / 0.87 at day 250 | 0.88 | 1.00 / 1.00 / 0.46 |
| consumer chamber | … / 0.69 at day 250 | 0.77 | 1.00 / 1.00 / 0.63 |

(The probe's potato line used wheat's leaf constant and is discarded.) **The sealed chambers never
reach LAI 1.43, so under either form their resistance exceeds 70 on every step.**

## 5. Predictions (before the lab form exists)

| | prediction | confidence |
|---|---|---|
| P1 | the frozen constant form, run through the new switch, is bit-identical: `regen_goldens` 20 of 20 identical | high |
| P2 | each form reaches the flow: a run's transpiration equals Penman–Monteith at that form's `rs` on the same state, to ~1e-9 (the R1 pattern) | high |
| P3 | **sealed chambers:** season transpiration falls under both — FAO by 45–75 %, Teh by 20–50 % (their `rs` sits at ≥ 200 / ≥ 100 against 70) | medium |
| P4 | **open-field default:** winter transpiration (LAI < 0.5, 41 % of steps) falls by > 80 % under both; summer (LAI 5–6) rises — FAO ~+15–30 %, Teh ~+5–15 %; the season total falls by 0–25 %, and drainage rises | low |
| P5 | carbon in the default and the sealed chambers stays bit-identical — they never reach water stress, so only water moves | medium (needs each run's lowest FTSW ≥ 0.30 asserted) |
| P6 | **deep-water rescue** (pinned: canopy 1.0294×, grain at the season's end 1.2199×): the control's invented seedling drought shrinks, so the grain ratio falls toward 1 (1.00–1.18) under both; the canopy ratio stays 1.00–1.06 | low |
| P7 | **`drought_window.rs`**: at LAI ~5.5 the full-cover crop drinks more, so stress (FTSW < 0.30) starts earlier than day 254 — FAO by 3–10 days, Teh by 1–6 | low-medium |
| P8 | **TM 102788 row:** W1 rises from 2.376 to ~2.8–2.9 (FAO) and ~2.5–2.6 (Teh); neither reaches the trial's ~6. The early water use falls toward the trial's low pre-cover values | medium |
| P9 | the dark steps carry ~8 % of the row's daily water use under the frozen form — the share a light-dependent `rst` would act on, left out here | medium |
| P10 | a mutation forcing each form back to a constant 70 turns the "reads LAI" assertion red | high |

**What would refute the motive.** If P5 fails — carbon moves in an unstressed run — the form has a
coupling not understood here, and that is the finding.

## 6. Decisions owed to the user, after the measurement

1. Which form, if any, to price for adoption (FAO's scope is full cover; Teh's is the only one that
   addresses a seedling, with a composite leaf value).
2. A constant leaf resistance, or the light-dependent Jarvis form (a WHAT-IF without wheat
   coefficients).
3. Teh's threshold LAI (his 4.0) or a cited crop maximum.
4. Whether the open field's bare-soil evaporation, which today's full-cover rate stands in for, is
   taken up (Teh §4.7 carries a soil form) — a separate item.

---

## 7. BUILT lab-only and measured (2026-10-07) — the predictions graded

**What was built (nothing frozen moved):** `science::SurfaceResistanceForm` (`Constant` — the
loader's value — `FaoFullCover`, `SzeiczLong`) and `science::canopy_surface_resistance`, with the
three lab constants beside it (`LEAF_STOMATAL_RESISTANCE` 100, `FAO_ACTIVE_LAI_FRACTION` 0.5,
`TEH_THRESHOLD_LAI` 4.0); `TranspirationParams::rs_form`, never loaded from the file;
`flows::CanopyRead`, which `Transpiration` carries only under a lab form (`None` on every frozen
build, so the frozen path is the old code path); `lab::biosphere_with_rs_form`. Tests:
`rust/crates/domains/tests/canopy_resistance.rs` (two instrument checks; the 24-season measurement
`#[ignore]`d, run with `--ignored --release`) and, in `station/tests/scorecard_tm102788.rs`,
`the_rows_water_curve_under_the_canopy_resistance_forms` (printed).

### 7a. The measurement

| scenario | form | season transpiration (kg) | at LAI < 0.5 | at LAI ≥ 4 | irrigated | lowest FTSW | end carbon = frozen |
|---|---|---|---|---|---|---|---|
| default | Constant | 587.29 | 95.40 | 351.05 | 586.98 | 0.9933 | — |
| | FAO | 555.41 (−5.4 %) | 17.37 (−82 %) | 421.18 (+20 %) | 555.10 | 0.9975 | yes |
| | Szeicz–Long | 559.57 (−4.7 %) | 30.52 (−68 %) | 388.85 (+11 %) | 559.24 | 0.9977 | yes |
| deep water 1 mm/d | Constant | 420.80 | 95.40 | 188.30 | 265.75 | 0.0810 | — |
| | FAO | 326.18 | 17.37 | 200.26 | 172.53 | 0.0767 | no (stressed) |
| | Szeicz–Long | 348.84 | 30.52 | 185.50 | 192.59 | 0.0720 | no (stressed) |
| sealed chamber | Constant / FAO / S–L | 708.21 / 334.60 (−53 %) / 504.19 (−29 %) | | | | 0.84 / 0.97 / 0.95 | yes |
| perennial chamber | Constant / FAO / S–L | 708.21 / 290.24 (−59 %) / 448.48 (−37 %) | | | | 0.84 / 0.97 / 0.95 | yes |
| consumer chamber | Constant / FAO / S–L | 708.25 / 248.14 (−65 %) / 393.58 (−44 %) | | | | 0.84 / 0.97 / 0.96 | yes |

Drainage was **0.00** in every run; the open field's irrigation is deficit-driven, so it tracks
transpiration (586.98 against 587.29). The sealed chambers are watered by their own condensate ring,
not irrigation.

| deep-water rescue (with the below-root store / without) | canopy | grain at the season's end |
|---|---|---|
| Constant (pinned) | 1.0294 (9.836 / 9.555) | 1.2199 (12.515 / 10.260) |
| FAO | **1.0000** (9.836 / 9.836) | 1.0360 (11.069 / 10.685) |
| Szeicz–Long | 1.0037 (9.836 / 9.799) | 1.0885 (10.894 / 10.008) |

| `drought_window` (cut days 220–260) | first day FTSW < 0.30 | lowest FTSW |
|---|---|---|
| Constant | 254 | 0.2093 |
| FAO | 247 | 0.1177 |
| Szeicz–Long | 250 | 0.1567 |

**TM 102788, the water curve** — each day's use over its own mean for TM days 25–80, against Fig. 10
(p. 27) "water added", read off the page image (± ~0.1):

| TM day | 5 | 8 | 11 | 14 | 20 | 26 | 40 | 60 | 80 | W1 (L m⁻² d⁻¹) |
|---|---|---|---|---|---|---|---|---|---|---|
| Fig. 10 / ~85 | 0.21 | 0.47 | 0.71 | 1.18 | — | 1.29 | 0.82–1.06 | | | ~6.0 (level) |
| Constant | 1.21 | 1.21 | 1.21 | 1.21 | 1.21 | 1.21 | 0.99 | 0.99 | 0.99 | 2.376 |
| FAO | 0.08 | 0.21 | 0.44 | 0.73 | 1.10 | 1.22 | 0.99 | 0.98 | 0.98 | 2.853 |
| Szeicz–Long | **0.17** | **0.40** | **0.74** | **1.06** | 1.21 | **1.21** | 0.99 | 0.99 | 0.99 | 2.607 |
| model LAI | 0.06 | 0.18 | 0.47 | 1.12 | 3.68 | 6.37 | 6.54 | 5.98 | 5.89 | |

(The model's step from 1.21 to 0.99 between days 26 and 40 is the trial's day-28 lamp dimming,
695 → 480 µmol; the trial's own curve falls over the same days.) Under the frozen form, fully dark
steps carry **5.7 %** of the row's water and partly lit steps 11.0 %.

### 7b. The predictions, graded

| | predicted | measured | grade |
|---|---|---|---|
| P1 | the constant form through the switch is bit-identical; `regen_goldens` 20/20 | every state of the default and sealed runs bit-identical (asserted); golden report: see §7d | HELD (assert); §7d |
| P2 | each form reaches the flow to ~1e-9 | to 1e-12, open field and sealed chamber, and each form moved the resistance (asserted) | HELD |
| P3 | sealed chambers: FAO −45–75 %, S–L −20–50 % | FAO −53 / −59 / −65 %; S–L −29 / −37 / −44 % | HELD |
| P4 | default: winter use −80 %+ under both; summer FAO +15–30 %, S–L +5–15 %; season −0–25 %; drainage rises | winter FAO −82 % (HELD), S–L **−68 % (FAILED)**; summer +20 / +11 % (HELD); season −5.4 / −4.7 % (HELD); drainage **0 throughout — the irrigation absorbs it (FAILED, the mechanism was wrong)** | mixed |
| P5 | carbon bit-identical in the unstressed runs | default and all three chambers: yes | HELD |
| P6 | deep-water grain ratio 1.00–1.18; canopy 1.00–1.06 | grain 1.036 / 1.089; canopy 1.000 / 1.004 | HELD on the numbers; **the reasoning was WRONG for the grain** (§7c point 2: the stored crop lost grain to the summer draw; the seedling drought explains the canopy only) |
| P7 | drought window earlier: FAO 3–10 days, S–L 1–6 | 7 / 4 days | HELD |
| P8 | W1 FAO ~2.8–2.9, S–L ~2.5–2.6; early use falls toward the trial's | 2.853 (HELD); 2.607 (**0.007 over**); early use falls under both (HELD) | mostly held |
| P9 | dark steps ~8 % of the water | 5.7 % fully dark (+ 11.0 % partly lit) | FAILED (low) |
| P10 | forcing the form back to constant in the flow turns the exactness check red | red, as predicted; restored | HELD |

### 7c. What it says

1. **Both forms are clean couplings:** water moves, carbon does not, until a drought makes the
   water matter. Then the shape of the season changes in two directions at once. The seedling
   barely drinks, so the invented early drought goes. The closed canopy drinks more, so a summer
   drought comes sooner and deeper (`drought_window` 7 or 4 days earlier; the deep-water crop's
   grain WITH its deep store falls 12.52 → 11.07 / 10.89).
2. **The deep-water rescue collapses to ~1 — for two different reasons.** ~~the rescue the frozen pin
   records was mostly the seedling drought the constant resistance invents~~ (withdrawn the same
   day, advisor review: the grain numbers contradict it). **The CANOPY gap was the seedling
   drought:** the store-less crop's peak leaf rises 9.555 → 9.836 / 9.799 and meets the stored crop's.
   **The GRAIN gap closes mostly from the other side:** the crop WITH the store loses grain (12.515 →
   11.069 / 10.894, its closed canopy drinking more in summer), while the store-less crop gains only
   under FAO (10.260 → 10.685) and LOSES under Szeicz–Long (→ 10.008). So the grain ratio now mostly
   reflects the summer draw, not the seedling.
3. **The three biosphere sealed chambers drink 29–65 % less** — their canopies never close, so their
   resistance sits above 70 on every step; their water stocks would move in an adoption. **The
   station's sealed builds were not run under either form — not measured.**
4. **The trial's early water curve fits Szeicz–Long better** (0.17 / 0.40 / 0.74 / 1.06 against
   0.21 / 0.47 / 0.71 / 1.18), while FAO's rises too slowly and the frozen constant is flat from day
   5. ⚠ **Only two of those points are clean:** the trays sat under acrylic lids for the first 120 h
   to hold humidity (TM 102788 p. 4), which suppresses water use whatever the leaf area, so day 5 is
   not clean; and 12 of 64 trays were reseeded on day 6 (p. 5), which touches day 8. On the clean days
   11 and 14 Szeicz–Long is still the closer (0.74 / 1.06 against 0.71 / 1.18; FAO 0.44 / 0.73). And the
   fit tests the form TOGETHER WITH the model's own LAI curve, with Teh's `Lcr` = 4 setting where it
   flattens. One trial, one figure read by eye: suggestive, not decisive. Neither form closes the
   LEVEL (2.6–2.9 against ~6), which stays the air-coupling question of the Step 6 record.
5. **FAO's form runs outside its source's scope below full cover**, and it is the one that fits the
   early curve worse. Szeicz–Long is the form whose source addresses low LAI — with a composite leaf
   value and no light response. ~~(worth ~6 % of the water here, in the dark)~~ **The light
   response's effect is unsized:** fully dark steps carry 5.7 % of the row's water and partly lit
   steps 11.0 %, and without wheat coefficients the lit-step Jarvis value could sit above or below
   100, so 5.7–17 % of the water falls where it would act.

### 7d. The gates

* `regen_goldens` (report only): **20 of 20 goldens identical, 0 would change** — P1 HELD in full.
* `cargo clippy --all-targets -- -D warnings`: clean.
* `cargo test --no-fail-fast`: **1354 passed, 0 failed, 6 ignored** (run before the measurement was
  marked `#[ignore]`, so it ran once inside the suite; it is run on demand from here on).
* `repo_gates` re-run after the doc edits.

---

## 8. Decisions TAKEN (the user, 2026-10-07) — price, not yet adopt

1. **Price Teh / Szeicz & Long for adoption** (over FAO-56 and over keeping both lab-only).
2. **A constant leaf resistance, 100 s m⁻¹** (FAO-56 Box 5), over the light-dependent Jarvis form.
3. **Teh's threshold LAI, 4.0**, over searching for a cited wheat maximum.

So the candidate reference form is `rc = 100 / min(LAI, 2.0)` s m⁻¹. Pricing it means measuring what
an adoption would move — every golden, every pinned test, every gate — with predictions first.
Adoption itself is a further decision and a biosphere + station unfreeze, not taken here.

---

## 9. Pricing Szeicz–Long for adoption — predictions before the flip (2026-10-07)

### 9a. Advisor review (2026-10-07), summarized

Enumerate all 20 goldens from the report, not a word search. Check the station's heat books for
evaporative cooling before predicting "water only". Decide potato rather than extend the user's
wheat decision to it silently. Make exact predictions from lab twins, the leaf-shedding pattern.
List the reds-by-design before running and classify every red after. Run the flip on a local branch
— one loader line, `regen_goldens` in REPORT mode only, the full suite, the ignored tests — never on
main. List the ceremony costs the run cannot show. Hand the user a price table, not a
recommendation.

### 9b. Checked before predicting

* **No evaporative heat in the station:** `station/src/chamber.rs` and every `station/src/*.rs` carry
  no latent-heat or transpiration term. Lower water use cannot warm the plant chamber, so the plants'
  temperature (which they read since slice 3a) cannot move through this route.
* **The station's watering trigger** refills at FAO-56's depletion fraction 0.55 (`watering-on-a-
  trigger`), so the root zone sits near FTSW ≥ 0.45, above `wssg` 0.30: no stress route.
* **Potato inherits the wheat transpiration params** (`params::potato()` is `..biosphere()`), so a
  loader flip would move potato too. No golden holds potato; its tests in
  `domains/tests/potato_crop.rs` would. **Priced as its own line; the user's decision covered wheat.**

### 9c. The lab twins — exact predictions

A throwaway probe (`W:\temp\claude\step6\twins\zz_twins.rs`, run once, not committed) mirrored the
producer of every golden that holds a crop. For each, a CONTROL run with the frozen-form
`Transpiration` swapped in reproduced the committed golden **byte for byte** (proving the mirror); then
the same run with the Szeicz–Long flow swapped in wrote the TWIN to `W:\temp\claude\step6\twins\`.
The station twins swap the flow by id before `wrap_last`, as `examples/shedding_station.rs` does.

**What each twin moves against its committed golden** (stock ids whose amounts differ; every other
stock and every aux value byte-identical):

| golden | moves |
|---|---|
| `season_euler_state` | `soil_water`, `boundary.vapor_sink`, `boundary.water_source` |
| `sealed_chamber_state` | `condensate`, `soil_water`, `water_vapor` |
| `perennial_chamber_state`, `perennial_long_horizon_state` | `condensate`, `soil_water`, `subsoil_water` |
| `consumer_chamber_state`, `consumer_long_horizon_state` | `condensate`, `soil_water`, `subsoil_water` |
| `lighting_state` | `condensate`, `soil_water` |
| `greenhouse_state`, `harvest_state` | `condensate`, `soil_water`, `water_vapor` |
| `sealed_station_state` | `condensate`, `soil_water`, `subsoil_water` (its transpiration sits on the plant step) |

**Water only, everywhere.** No carbon, nitrogen, oxygen, energy or aux value moves in any twin.

**Predicted unchanged (the other ten):** `drift_summary.json` (it folds leaf-carbon trajectories only,
and carbon does not move), `sealed_energy_drift_summary.json` (`build_station`, no crop), and the eight
crop-free goldens (`cabin_gas`, `crew`, `eclss`, `power`, `power_self_discharge`, `thermal`,
`water_recovery`, `station`) plus the untiered `state_snapshot.json`.

### 9d. Predictions for the branch run

| | prediction |
|---|---|
| Q1 | `regen_goldens` (report): exactly the ten goldens of §9c "would change"; each regenerated text equals its twin **byte for byte**; the other ten (and `state_snapshot.json`) identical |
| Q2 | red by design: `canopy_resistance.rs` P1 (the loader no longer returns `Constant`); the scorecard's R1 (`biosphere_what_if` inherits the new default, `w1_by_hand` still assumes 70); `golden_regression` on the ten moved goldens; the manifest byte gates on their hashes; the deep-water rescue's grain pin in `system.rs` (lab 1.0885 against a band set at 1.22) |
| Q3 | possibly red, classified after: any test pinning a water stock, a humidity or a transpiration total in a run with a crop (the chamber-dryness, vapour, watering, minute-step, air-split and lamp tests); `potato_crop.rs` where it pins water |
| Q4 | green: every carbon, energy and gas test; `drought_window.rs` (its assertions do not pin days); the conservation and determinism laws |
| Q5 | ignored tests (`--ignored --release`): `chamber_walls.rs` trajectory pins green — they read the node temperature and the battery, which do not read water |

### 9e. Ceremony costs the run cannot show

1. **The numbers become param data.** The reference may not hard-code coefficients, so `100` (FAO-56
   Box 5) and `4.0` (Teh p. 98) move into `transpiration.yaml` with their sources — the file's
   digest changes, so the biosphere manifest's `param_files` row regenerates (C7).
2. **The uncited 70** stops being read by the reference: kept for the lab `Constant` form, or retired
   (a schema change). The user's call.
3. **Manifest lines:** the ten moved goldens' `golden_sha256` rows (biosphere and station manifests)
   plus the `transpiration.yaml` row.
4. **Prose:** each contract's dated-change entry; the station contract's own sentence *"transpiration
   does not read the canopy"* (`docs/station-reference.md`) becomes false; the record's standing gap
   "transpiration ignores leaf area" closes for the crop (not for bare-soil evaporation).
5. **Potato:** adopt for both crops, or keep potato on the constant (a `transp` override in
   `params::potato()`).

---

## 10. The price, measured on a local branch (2026-10-07) — graded

Branch `wip/canopy-resistance-price` (local, never pushed; one commit, `b5d20b5`, the loader line
`rs_form: SurfaceResistanceForm::SzeiczLong`). Main keeps the constant form throughout. Logs:
`W:\temp\claude\step6\price\`.

### 10a. The predictions, graded

| | predicted | measured | grade |
|---|---|---|---|
| Q1 | exactly ten goldens change, each equal to its twin byte for byte; the other ten identical | `regen_goldens` report: **20 run, 10 would change** — the predicted ten; a probe calling each producer on the branch: **all ten equal their twins byte for byte** | HELD exactly |
| Q2 | red by design: canopy P1, scorecard R1, the goldens' regression and band checks, the manifest byte gates, the deep-water grain pin | P1, R1, `golden_regression` (domains + station cheap + the expensive sealed station), `tier_contract` (all three), the deep-water pin (*"1.0885 (measured 1.2199)"*) red. **The manifest byte gates stayed green** — they hash the committed files, which a report-only run does not rewrite | HELD except the manifest gates (wrong: they go red only after `--write`) |
| Q3 | possibly red: tests pinning water in a run with a crop | **none red** — no chamber-dryness, vapour, watering, minute-step, air-split, lamp or potato test moved | none fired |
| Q4 | carbon, energy, gas tests, `drought_window`, the laws green | green | HELD |
| Q5 | `chamber_walls` trajectory pins green | green (heater battery; node and chamber temperatures), plus both session resumes | HELD |
| — | **NOT predicted** | `the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years` red: the perennial chamber's below-root store no longer settles after one cycle | a finding (10b) |

Suite on the branch: **1345 passed, 8 failed** (debug); ignored, release: 5 passed, 2 failed (the
sealed station's golden and band). Every red classified: 7 are goldens / bands / instrument checks
that an adoption regenerates or restates by construction; 1 is a behaviour change.

### 10b. The one unpredicted red, measured

On the branch, the perennial chamber's below-root store at each cycle's start, 15 cycles (a
throwaway probe, `W:\temp\claude\step6\price\zz_subsoil.rs`): 172.242, 172.329, 172.444, 172.537,
172.610, … 172.851 — rising by 0.087, 0.115, 0.092, 0.073 … 0.008 kg per cycle, the increment
shrinking ~0.8× a cycle. **It converges** (to ~172.9 kg), not a ratchet: water migrates slowly from
the condensate (3.18 → 2.44 kg) into the subsoil because the sparse crop drinks less. The frozen claim
"one transient cycle, then a fixed point held to round-off" becomes "a transient of ~4 cycles". Water
is conserved throughout (asserted each step).

### 10c. The price table

| golden | what moves (end state) |
|---|---|
| `season_euler` (open field) | transpired −27.7 kg (−4.72 %) and irrigated −27.7 kg — the same water; soil −0.02 % |
| `sealed_chamber` | condensate 8.15 → **3.0e-6**; water vapour 0.25 → **2.9e-6** kg; soil +8.40 kg (+5.2 %) |
| `perennial_chamber` / long horizon | condensate −66 % / −69 %; soil +2.6 % / +2.7 %; subsoil +4.9 % / +5.0 % |
| `consumer_chamber` / long horizon | condensate −75 % / −78 %; soil +2.9 % / +3.0 %; subsoil +5.5 % / +5.7 % |
| `lighting` | condensate −88 %; soil +11.5 % |
| `greenhouse` | condensate −84 %; vapour −57 %; soil +16.2 % |
| `harvest` | condensate −87 %; vapour −69 %; soil +3.1 % |
| `sealed_station` | condensate +9.1 %; subsoil +9.5 %; soil −1.7 % (the tier band reports 9.46 % at its worst leaf) |
| the other ten | identical |

**Water only:** no carbon, nitrogen, oxygen, energy or aux value moves in any golden.

**A gap this exposes — must be weighed before any adoption.** In the 3-year sealed chamber the air
ends with **2.9e-6 kg of water vapour (against 0.25 frozen) — effectively none** over a wet soil: the crop has senesced (LAI → 0, so its
resistance → ∞ and it transpires nothing), and the model has **no bare-soil evaporation** — the
frozen constant resistance had been letting a dead or absent canopy keep "transpiring", standing in
for it. An adoption without a soil-evaporation term gives sealed chambers bone-dry air whenever the
crop is small or gone. (Teh §4.7, on the same page as Eq. 4.80, carries a soil form; FAO-56's own
scope sentence says partial cover "should indeed include the effects of the evaporation from the soil
surface".)

**Ceremony, beyond the run (§9e):** `100` and `4.0` into `transpiration.yaml` with their sources (the
file's digest changes → its manifest row regenerates); the uncited 70 kept for the lab form or
retired; ten `golden_sha256` rows across the biosphere and station manifests; the contracts' dated
entries and the station contract's sentence "transpiration does not read the canopy"; the deep-water
pin and the re-sow cycle test restated at their measured values; potato — adopt for both crops or
keep potato on the constant.
