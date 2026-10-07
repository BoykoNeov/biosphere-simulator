# Post-roadmap: bare-soil evaporation — lab build, then priced together with the canopy resistance

**Opened 2026-10-07 on the user's call** (*"Soil evaporation first"*), after pricing the leaf-area
canopy resistance (`post-roadmap-canopy-resistance.md` §10–§11): under that form a never-re-sown
sealed chamber's air dries out while its crop is dead, because the model has no evaporation from
bare soil — the frozen constant resistance had been standing in for it. The user's order: build soil
evaporation in the lab, then re-price it and Szeicz–Long together. **Nothing frozen or adopted.**

## 1. Advisor review (2026-10-07), summarized

The choice is how a form fits the model's existing water and energy, not its source alone. Bolted
onto the crop's Penman–Monteith without an energy split, a soil term spends the same radiation twice
at partial cover — worst in the sealed chambers (LAI ≈ 1), and nothing in the books would catch it.
Read the 1.5 mm/day floor on the page and price it as its own line. Name every mapping decision now:
the top layer, the dry-stage clock, the rewetting rule, the radiation input (no double albedo), the
vapour's destination in sealed builds and every flow list it must join. Teh's two-source form needs
soil data the model lacks. State the success test before building; the NASA trial does not apply
(hydroponic trays, no soil).

## 2. The source, read first-hand

Soltani & Sinclair (2012), *Modeling Physiology of Crop Development, Growth and Yield*, Ch. 14 (on
the shelf; book pp. 172, 180–181, 184 read off the page images):

* p. 180, Eqns 14.15–14.18: *"potential soil evaporation from a bare, wet soil each day (EOS, mm
  day⁻¹)"* = `SRAD × (1 − SALB) × EXP(−KET × ETLAI) × DELT / (DELT + 0.68)`, converted by 239/583
  (energy → mm); `SALB` *"commonly … close to 0.12"*; `KET` ~0.5, the canopy extinction coefficient
  for global radiation. *"A simplified Penman equation."*
* p. 180, Eqn 14.16: `ETLAI = LAI` before beginning seed growth, then *"kept constant at the value of
  LAI at beginning seed growth (BSGLAI) because senesced leaves, attached to the plant or fallen on
  the soil surface, still shade the soil"*.
* p. 180: *"Since some radiation always provides energy for soil evaporation, a minimum value of EOS
  is used. Generally, it is assumed the minimum EOS is 1.5 mm (Amir and Sinclair, 1991)."* p. 184:
  *"minimum soil evaporation when the soil is covered by crop (EOSMIN, 1.5 mm day⁻¹)"*. The book's
  program applies it only *"If PET > EOSMIN And EOS < EOSMIN"*.
* pp. 180–181, Eqn 14.19 (Amir & Sinclair 1991): Stage I, `SEVP = EOS`, while the top layer holds
  water (`ATSW1 > 1`) and the profile is not dry (`FTSW > 0.5`); Stage II,
  `SEVP = EOS × ((DYSE + 1)^0.5 − DYSE^0.5)`, `DYSE` the days since Stage II began; back to Stage I
  only after rain or irrigation over 10 mm (`WETWAT`).
* p. 172: the top layer is *"usually 150 to 600 mm"* deep.

## 3. Decisions TAKEN (the user, 2026-10-07)

1. **Soltani & Sinclair's two-stage form WITH an energy split**: the soil gets `exp(−KET·ETLAI)` of
   the radiation, the crop the rest — so crop transpiration changes too and both are re-measured
   (over the form bolted on without a split, and over Teh's two-source form).
2. **The 1.5 mm/day floor as a switch, measured on and off.**
3. **The wet stage decided by the top layer's own water** (wet when it holds water and the profile is
   above half full) — a deliberate departure from the book's 10 mm rewetting rule, chosen because the
   model waters continuously in small amounts.

## 4. Mapping decisions (mine, stated; each changeable)

* **The top layer is the root zone's own water fraction.** The model already declares water uniform
  through its root zone (the re-sow's returned water). So `ATSW1 = FTSW × TTSW1`, and `ATSW1 > 1 mm`
  holds whenever `FTSW > 1/(DEP1·EXTR)` — 0.05 at the book's shallowest 150 mm, 0.013 at 600 mm —
  which the user's other condition, `FTSW > 0.5`, always implies. **So the stage test is `FTSW >
  0.5`, and `DEP1` drops out:** no new stock, no value picked inside the book's range. Evaporated
  water leaves the root-zone store (`soil_water`).
* **The dry-stage clock** `DYSE` is a new lab aux value: it counts days (in steps × dt) while `FTSW ≤
  0.5` and resets to 0 when the zone is above half full again.
* **ETLAI's hold** at beginning seed growth is a second lab aux value. "Beginning seed growth" maps to
  the first step the model's own partition table gives grain a share.
* **Radiation input.** The soil reads the same net-radiation forcing the crop reads (`RN_VAR` —
  weather outdoors, the lamp's in lamp-lit and sealed builds). That forcing is already net of FAO's
  0.23 albedo, so the incident value is recovered as `RN / (1 − 0.23)` and the soil applies its own
  0.12 once — no double albedo.
* **The split.** The crop's Penman–Monteith radiation term reads `RN × (1 − exp(−KET·ETLAI))`; its
  aerodynamic (VPD) term is unchanged. (The book's soil term has no VPD term.)
* **Constants.** `DELT/(DELT + 0.68)` uses the model's own slope and psychrometric constant (67 Pa
  K⁻¹ against the book's 0.68 mbar K⁻¹ = 68 Pa K⁻¹); energy → water by the model's 2.45 MJ kg⁻¹.
* **The floor** (switch on): `EOS = max(EOS, 1.5 mm/day)` only when the bare-soil potential (without
  the canopy factor) exceeds 1.5, as the book's program does.
* **Sealed builds.** The soil's vapour goes to the chamber's air through the same saturation cap and
  condensate split as transpiration.
* **Reach.** Built in `build_season_with`, so the biosphere scenarios and every station build that
  calls `build_season` reach it. The station's lab-only air-split and minute-step paths move flows by
  id lists; whether they move the new flow is checked and stated at build time.

## 5. The success test (written before building)

* In the never-re-sown sealed chamber, while its crop is dead (LAI < 0.1, years 2–3), the air no
  longer collapses under Szeicz–Long + soil evaporation, and the water comes from the soil, not the
  dead crop.
* The re-sown chambers, the station and the open field are measured, not assumed. Outdoors, soil
  evaporation should restore much of the winter water use the canopy form removed (FAO-56's
  partial-cover sentence).
* An energy readout: per step, the soil's evaporation and the crop's radiation-driven transpiration
  together stay within the net radiation (the crop's VPD-driven part is reported separately).

Predictions, built before any code, follow in §6 once the build's surface is fixed.

---

## 3b. Advisor review of §4 (2026-10-07), summarized — and the user's fourth decision

The review found that §4's "the top layer drops out" also removed the drying stage: with the top
layer read as the whole root zone, Stage II needs the zone at or below half full, which a watered run
almost never reaches (default 0.99, chambers ~0.84, station ≥ ~0.45), so evaporation would run at its
full potential from up to 1.3 m of soil. Put to the user. Also: two flows filling one chamber's air
overshoot its humidity cap (the cabin-humidity lesson) — build soil evaporation inside
`Transpiration`; give the crop its GREEN leaf area and the soil the held one; the book's Stage II
factor and floor are daily quantities against a 1/16-day step; every new state value must be reset at
every re-sow path and must be finite.

**DECIDED 2026-10-07 (the user): a thin top layer, 150 mm** — the shallow end of the book's 150–600 mm,
the model's own rooting depth at emergence (`rooted_depth0`), over another depth and over the whole
root zone.

## 4b. The design, revised (supersedes §4 where they differ)

**The top layer is the book's own overlapping account, not a second store of water.** Soltani &
Sinclair track two balances: the TOTAL root zone (Eqn 14.5, which already contains the top layer) and
the top layer (Eqn 14.2), fed by the same watering, with its own drainage downward (`DRAIN1`), its own
share of transpiration (`TR1`) and all of the soil evaporation. So:

* `soil_water` stays the whole root-zone store, and none of its ten readers changes.
* A new lab aux value, `top_soil_water` (`ATSW1`, kg on the ground area), is advanced each step by an
  aux process from the SAME pure functions the flows use on the same start-of-step snapshot (aux and
  flows are evaluated on one snapshot — `simcore::integrator`): `+` the irrigation or the sealed
  recycling inflow, `− DRAIN1 = (ATSW1 − TTSW1)·DRAINF` above capacity (the model's `drainage_factor`),
  `− TR1` (all of the transpiration while rooted depth ≤ 0.15 m; else `TR · min(1, FTSW1/WSSG)`, the
  book's program), `− SEVP`; floored at 0. `TTSW1 = 0.15 m · EXTR · 1000 · area`.
* **The stage (the user's rule):** Stage I while `ATSW1 > 1 mm` and `FTSW > 0.5`; Stage II otherwise.
* **The dry-stage clock** `soil_dry_days` (aux): + dt per step in Stage II, reset to 0 in Stage I. The
  Stage II factor is the book's per-DAY amount, `√(d+1) − √d` with `d = ⌊soil_dry_days⌋`, applied as a
  rate through day d — so a day's steps sum to the book's daily value.
* **The held leaf area** `soil_shade_lai` (aux): equals green LAI until the partition table first gives
  grain a share, then holds. Finite throughout (it starts at the seedling's LAI).
* **One flow, one cap.** Soil evaporation is computed inside `Transpiration`, so the sealed chamber's
  saturation cap and condensate split see the SUM, and every path that moves the flow by id moves both.
  Open builds give the soil its own sink leg so the books separate. A pure function computes the
  crop/soil split for readouts.
* **The energy split:** the crop's radiation term reads `RN · (1 − exp(−KET · LAI_green))`, the soil
  `RN/(1 − 0.23) · (1 − 0.12) · exp(−KET · soil_shade_lai)`. Since `soil_shade_lai ≥ LAI_green`, the
  two shares never exceed 1; the energy dead leaves intercept goes to neither. `KET` = 0.5 (the book's,
  for global radiation) in every build — in lamp-lit builds the crop's own PAR extinction is 0.60; the
  book's value is kept and the difference recorded.
* **The floor (switch):** met as a daily total — aux accumulators for the day's soil evaporation and
  its bare-soil potential; on the day's last step, if the potential exceeded 1.5 mm and the evaporated
  total is below it, the shortfall is added (only while the top layer holds it).
* **Every re-sow path** (`annual_reset_with`; the station's `sealed_reset_hook`) resets the three
  values: `top_soil_water` to the uniform share of the zone's water, `soil_dry_days` to 0,
  `soil_shade_lai` to the new seedling's LAI. A plain reset refuses a state carrying them (the lab leaf
  form's precedent).
* `weather::net_radiation` is checked before dividing by 0.77; if it carries a long-wave term, the
  incident shortwave is read from the weather's own shortwave forcing instead.
  **Checked:** `weather::net_radiation` is `(1 − 0.23) × shortwave` exactly (no long-wave term), and the
  lamp's is the same `net_shortwave` of its radiant PAR — so `RN / 0.77` is the incident value in every
  build.

## 6. Predictions (before any code)

Hand arithmetic behind the sizes: bare wet soil under full outdoor sun at ~100 W m⁻² mean incident
gives `100 · 0.88 · Δ/(Δ+γ)` ≈ 55 W m⁻² ≈ **1.9 mm/day** at 20 °C (Δ/(Δ+γ) ≈ 0.63); the frozen crop's
full-cover rate on the same weather is ~2 mm/day. Under LAI 1 the soil keeps 61 %, under LAI 6 5 %.

| | prediction | confidence |
|---|---|---|
| S1 | with soil evaporation off and the constant resistance, the new code path is bit-identical: every state of a frozen run, and `regen_goldens` 20 of 20 | high |
| S2 | a day of constant forcing at dt = 1/16 sums to the book's daily value — Stage I `EOS`, Stage II `EOS·(√(d+1) − √d)` — to 1e-12; the floor adds exactly the day's shortfall to 1.5 mm | high |
| S3 | energy: on every step the soil's evaporation and the crop's radiation-driven transpiration together use at most the net radiation (latent-heat equivalent); the crop's VPD-driven part reported apart | high |
| S4 | **the success test** — in the never-re-sown sealed chamber, over its dead-crop steps (LAI < 0.1), the mean chamber vapour under Szeicz–Long + soil (floor off) is **0.7–1.5× the frozen run's** (Szeicz–Long alone: ~0); refuted below 0.5 | medium |
| S5 | open-field winter (LAI < 0.5) water loss, frozen 95.4 kg / Szeicz–Long alone 30.5: with soil evaporation **65–130 kg** — most of it now from the soil | low-medium |
| S6 | carbon bit-identical in the default season and the three sealed chambers (no run reaches FTSW < 0.30) | medium |
| S7 | the open field's top layer is in Stage I on > 90 % of steps (it is watered daily); a sealed chamber's, fed continuously by recycling, likewise | medium |
| S8 | the floor (on) adds 80–160 kg to the open field's season evaporation (under the closed canopy, `exp(−0.5·6) = 5 %`, the floor lifts ~0.15 to 1.5 mm/day for ~100 summer days); < 10 % in the chambers, whose canopies never close | low |
| S9 | drought runs dry sooner: `drought_window`'s first stressed day earlier than Szeicz–Long alone's 250 | medium |
| S10 | every re-sow resets the three values (asserted after `annual_reset_with` and in the station's hook) | high |

## 7. Watering in events (the user, 2026-10-07)

**The problem, raised by a side note and confirmed:** the model waters continuously — the open
field is topped up to full every step (`Irrigation`, deficit-driven), the sealed chambers get a
steady trickle of recycled condensate — and the weather fixture has no rain (`WeatherRow` carries
temperature, radiation and vapour pressure only). So the 150 mm top layer is refilled every day and
the drying stage would appear only in drought runs. Real fields and chambers are watered in events.

**DECIDED (the user): FAO-56's depletion trigger** (over a fixed interval with an unsourced dose, and
over keeping daily top-ups). FAO-56 Table 22, wheat `p = 0.55` (`RAW = p·TAW`), already used by the
station's lab watering (`station::air_split::FAO56_WHEAT_DEPLETION_FRACTION`).

**The design (lab-only, a third switch, `WateringForm`):**
* **Open field:** `Irrigation` waters only on a step that starts with the root zone at or below
  `FTSW = 1 − p = 0.45`, and then applies the whole deficit to full in that step — FAO-56's
  scheduling applies the net depth `Dr` at once. The 8 mm/day system capacity does not cap an event
  (stated: an event is not a daily rate).
* **Sealed chambers:** the condensate store is the reservoir. `Recycling` moves water only at a
  trigger step, `min(condensate, deficit)`, instead of a steady fraction.
* Unlike `station::air_split::TriggeredWatering`, which holds the zone AT the trigger with capped
  top-ups (no latch), these are events: the zone refills to full and then dries for days.
* The top-layer account reads the same `Irrigation`/`Recycling` instance, so it sees the events.
* The trigger (0.45) sits above the crop's stress threshold (`wssg` 0.30), so the crop should never
  be stressed by the schedule itself — carbon is predicted unchanged; water timing changes.

---

## 8. BUILT lab-only and measured (2026-10-07) — graded

**Built (nothing frozen moved):** `science::SoilEvaporationForm` (`Off` — the loader's —
`TwoStage { floor }`), `science::WateringForm` (`Continuous` — the loader's — `Fao56Trigger`), the
book's constants beside them; `flows::SoilEvapRead`, `WaterSplit` and `Transpiration::split` (one
computation, used by the flow and the account), one sealed cap helper (`sealed_legs`) for both
branches; `flows::SoilSurfaceAccount` (the top layer's account and the clocks);
`flows::EventIrrigation` / `EventRecycling` (watering in events, lab wrappers, a zero capacity still
a hard off); one builder each for the water flow and the two watering flows, so the account's
instances are built as the season's are; `annual_reset_with` resets the five new values, the plain
`annual_reset` refuses them; `lab::biosphere_with_soil_evaporation`. Test:
`rust/crates/domains/tests/soil_evaporation.rs` (four instrument checks; the measurement
`#[ignore]`d, run with `--ignored --release`). Found by its own check while building: the plain
reset's refusal had not been applied by the first edit — the S10 test went red and caught it.

### 8a. The measurement (per year of the run)

| scenario | config | water out of the root zone | of it soil | at LAI < 0.5 | Stage I share | watering events | lowest FTSW | carbon = frozen | dead-crop vapour / frozen |
|---|---|---|---|---|---|---|---|---|---|
| default (1 y) | frozen | 587.29 | 0 | 95.40 | — | 0 | 0.993 | — | — |
| | Szeicz–Long alone | 559.57 | 0 | 30.52 | — | 0 | 0.998 | yes | — |
| | S–L + soil, daily watering | 651.42 | 148.51 | 91.36 | 1.000 | 0 | 0.994 | yes | — |
| | S–L + soil, events | 538.61 | 35.98 | 38.20 | 0.241 | 5 | 0.448 | yes | — |
| | S–L + soil + floor, events | 613.47 | 110.84 | 38.20 | 0.229 | 6 | 0.449 | yes | — |
| sealed chamber (3 y, never re-sown) | frozen | 708.03 | 0 | 532.25 | — | 0 | 0.843 | — | 1.000 |
| | S–L alone | 180.22 | 0 | 38.95 | — | 0 | 0.952 | yes | 0.195 |
| | S–L + soil, daily | 510.54 | 392.40 | 315.74 | 1.000 | 0 | 0.867 | yes | **0.999** |
| | S–L + soil, events | 167.79 | 49.87 | 43.89 | 0.087 | 5 | 0.450 | yes | **0.204** |
| | S–L + soil + floor, events | 166.46 | 48.57 | 42.30 | 0.093 | 5 | 0.450 | yes | 0.195 |
| perennial chamber (5 y) | frozen | 707.99 | 0 | 319.95 | — | 0 | 0.843 | — | 1.000 |
| | S–L alone | 404.99 | 0 | 117.97 | — | 0 | 0.955 | yes | 0.993 |
| | S–L + soil, daily | 701.89 | 459.14 | 252.38 | 1.000 | 0 | 0.867 | yes | 1.000 |
| | S–L + soil, events | 446.34 | 204.31 | 137.88 | 0.333 | 46 | 0.447 | yes | 0.998 |
| | S–L + soil + floor, events | 476.90 | 234.11 | 150.68 | 0.392 | 57 | 0.443 | yes | 0.997 |
| consumer chamber (5 y) | frozen | 708.00 | 0 | 398.23 | — | 0 | 0.836 | — | 1.000 |
| | S–L alone | 346.30 | 0 | 129.01 | — | 0 | 0.957 | yes | 0.941 |
| | S–L + soil, daily | 684.20 | 487.46 | 319.86 | 1.000 | 0 | 0.861 | yes | 0.999 |
| | S–L + soil, events | 448.39 | 250.68 | 188.94 | 0.396 | 62 | 0.442 | yes | 0.941 |
| | S–L + soil + floor, events | 464.05 | 266.52 | 198.80 | 0.430 | 72 | 0.441 | yes | 0.957 |

`drought_window`'s cut (days 220–260), first day below FTSW 0.30: frozen 254; S–L alone 250; + soil
daily 251; + soil, events **243**; + soil + floor, events **237**.

### 8b. The predictions, graded

| | predicted | measured | grade |
|---|---|---|---|
| S1 | the frozen forms through the switches are the frozen run, bit for bit | asserted (default and sealed, every stock and aux value) | HELD (golden report: §8d) |
| S2 | a day sums to the book's daily value; the floor tops each eligible day to 1.5 mm | 63 floor-eligible days, 0 short; the Stage II factor constant within a day and equal to `√(d+1) − √d` | HELD |
| S3 | soil + crop radiation terms ≤ the net radiation | worst 0.7196 | HELD |
| S4 | **the success test:** dead-crop vapour 0.7–1.5× frozen (refuted < 0.5) | daily watering **0.999 — HELD**; watering in events **0.204 — FAILED** | split by watering |
| S5 | open-field winter water 65–130 kg | daily 91.36 (HELD); events 38.20 (FAILED) | split by watering |
| S6 | carbon bit-identical in the four runs | yes in every config, events included (lowest FTSW ≥ 0.44 > 0.30) | HELD |
| S7 | the top layer in Stage I on > 90 % of steps | daily watering 100 % (HELD — the prediction assumed it); events 9–43 % | HELD for the watering it assumed |
| S8 | the floor adds 80–160 kg to the open field, < 10 % in the chambers | open field +74.9 kg (just below); sealed −1 %, perennial **+15 %**, consumer +6 % | FAILED narrowly on both |
| S9 | drought runs dry sooner than S–L alone's day 250 | events 243 / 237 (HELD); daily 251 (FAILED by one day) | mostly held |
| S10 | every re-sow resets the values; the plain reset refuses | asserted (after a first-edit miss the test itself caught) | HELD |

**The count (corrected 2026-10-07, advisor review):** of 10 predictions **6 held** (S1, S2, S3, S6, S7,
S10), 1 mostly held (S9), 1 failed (S8), 2 split by watering (S4, S5).

### 8c. What it says

1. **With daily top-ups the surface never dries** (Stage I 100 % everywhere), and soil evaporation then
   holds the dead-crop chamber's air at 0.999 of frozen — ~~it replaces the dead crop's phantom
   transpiration almost one for one~~ (withdrawn, §9: the vapour readout tops out at the humidity
   setpoint; over the dead-crop days the soil supplies **0.56** of the frozen dead crop's water).
2. **With watering in events the surface dries between waterings** (Stage I 9–43 % of steps), drought
   bites a week or more sooner, and the water cycled through the chambers falls by a third or more.
3. **But the never-re-sown chamber's dead phase dries out again under events (0.20):** its root zone
   is DEEP (the dead crop's roots reached 1.30 m — the golden's own `rooted_depth`), the soil only evaporates from the 150 mm top, so the
   zone never falls to the 0.45 trigger — nothing waters it, the surface stays dry, and Stage II
   decays as √t. Physically that is a fallow chamber with a dry crust: plausible, and not the
   "phantom transpiration" artefact, but it means the success test (S4) passes only with daily
   watering. The re-sown chambers keep their air under every config (0.94–1.00).
4. **Carbon never moves** in any configuration measured: the crop is never stressed, events included.
5. **The floor matters only with a crop:** +75 kg/yr in the open field; nothing in the fallow chamber
   (dry surface, Stage II).

### 8d. The gates

* `regen_goldens` (report): **20 of 20 identical, 0 would change** — S1 in full.
* `cargo clippy --all-targets -- -D warnings`: clean (two `is_multiple_of` rewrites, same arithmetic,
  made after the suite started).
* `cargo test --no-fail-fast`: **1357 passed, 0 failed, 8 ignored**.

## 9. The top layer overfilled under events — measured, decided, predictions (2026-10-07)

**Advisor review of §8, summarized:** an event puts the whole root-zone deficit (~90 kg) into a top
account of 19.5 kg capacity, and only the 30 %/day drainage brings it down — the book also caps the
top layer at saturation (`WSAT1 = DEP1·SAT`, the excess running off). Check the overshoot first. The
success readout saturates at the humidity setpoint (0.999 means "at the cap"), so add soil
evaporation over the frozen dead crop's transpiration; strike "almost one for one" (392 against 708).
The §8b tally is wrong: 6 held, 1 mostly, 1 failed, 2 split. The station cannot run the soil form
until a lab re-sow hook uses `annual_reset_with`; under events its recycling becomes
`EventRecycling`, and the lab air-split's `TriggeredWatering` does not feed the account. And the
soil-shading LAI of a never-re-sown crop stays held at its seed-growth value (~1.0) through its dead
years, keeping ~39 % of the light off the fallow soil — the book's hold covers one season.

**Measured (a throwaway probe, `W:\temp\claude\step6\soil\zz_overshoot.rs`), top-layer capacity 19.5
kg:** daily watering — peak 1.00× (default, sealed), 1.10× (perennial); events — **peak 4.78×**,
31.9 days above capacity in the default season (16.4 above 2×), 36.2 in the 3-year sealed chamber,
165.8 in the 5-year perennial. **So §8's event rows are inflated.**

**DECIDED (the user): the book's limit, silt loam** (over silty clay, passing straight through, and
labelling the rows). Soltani & Sinclair Table 13.1, silt loam: `SAT` 0.433, `DUL` 0.218, `EXTR` 0.132
(the model's 0.13). Above capacity the top drains at the model's `drainage_factor` (the book's
`DRAINF`, 0.3); above saturation, `DEP1·(SAT − DUL)` = 32.25 mm over capacity (**2.65×**), the account
stops counting — that water is already in the root zone below, so no stock moves.

**Predictions:**
* T1: under events the peak fill is ≤ 2.65× capacity in every run; daily watering unchanged.
* T2: under events, soil evaporation and the Stage I share fall below §8's (the inflated top had
  room it no longer has); carbon still identical.
* T3: the never-re-sown chamber's dead-phase air stays dry under events (its trigger still never fires).
* T4: the four instrument tests stay green.

### 9a. Measured with the cap — graded

| run | config | water out (kg/yr) | of it soil | Stage I share | events | dead-crop vapour / frozen | dead-crop soil / frozen crop water | peak top fill |
|---|---|---|---|---|---|---|---|---|
| default | + soil, daily | 651.42 | 148.51 | 1.000 | 0 | — | 0.961 | 1.00 |
| | + soil, events | 536.26 | 33.63 | 0.204 | 5 | — | 0.744 | **2.65** |
| | + soil + floor, events | 592.70 | 88.57 | 0.180 | 6 | — | 0.744 | 2.65 |
| sealed chamber (3 y) | + soil, daily | 510.54 | 392.40 | 1.000 | 0 | 0.999 | **0.559** | 1.00 |
| | + soil, events | 150.45 | 32.44 | 0.061 | 4 | 0.176 | **0.011** | 2.65 |
| | + soil + floor, events | 150.55 | 32.54 | 0.061 | 4 | 0.176 | 0.011 | 2.65 |
| perennial (5 y) | + soil, daily | 701.89 | 459.14 | 1.000 | 0 | 1.000 | 0.657 | 1.10 |
| | + soil, events | 481.79 | 239.05 | 0.416 | 63 | 0.998 | 0.220 | 2.65 |
| | + soil + floor, events | 485.97 | 243.19 | 0.420 | 66 | 0.998 | 0.277 | 2.65 |
| consumer (5 y) | + soil, daily | 684.20 | 487.46 | 1.000 | 0 | 0.999 | 0.618 | 1.10 |
| | + soil, events | 333.34 | 136.27 | 0.173 | 25 | 0.936 | 0.209 | 2.65 |
| | + soil + floor, events | 389.11 | 191.26 | 0.275 | 41 | 0.935 | 0.179 | 2.65 |

(Frozen and Szeicz–Long-alone rows unchanged from §8a; carbon bit-identical to frozen in every row.)
`drought_window` under events: first stressed day **244** (floor: **240**), against §8's 243 / 237.
Instrument tests green; S2 now 49 floor-eligible days, 0 short; S3 worst 0.7196.

| | predicted | measured | grade |
|---|---|---|---|
| T1 | peak fill ≤ 2.65× under events; daily unchanged | 2.65 in every event run; daily 1.00 / 1.10 as before | HELD |
| T2 | under events, soil evaporation and Stage I share fall below §8's; carbon identical | default 35.98 → 33.63 and 0.241 → 0.204; sealed 49.87 → 32.44, 0.087 → 0.061; consumer 250.68 → 136.27, 0.396 → 0.173 — HELD; **perennial ROSE** 204.31 → 239.05 and 0.333 → 0.416 (63 events against 46: with less room in the top, the zone reaches the trigger more often); carbon identical everywhere | HELD except the perennial chamber |
| T3 | the never-re-sown chamber's dead phase stays dry under events | 0.176 | HELD |
| T4 | the instrument tests stay green | green | HELD |

**What the new readout says.** The 0.999 of §8 was the humidity cap: even under daily watering the
soil supplies only **0.56–0.96** of what the frozen model's dead or sparse crop "transpired" over its
dead-crop days, and under events **0.01–0.74**. Soil evaporation does not replace the phantom water
use one for one anywhere; it replaces part of it.
