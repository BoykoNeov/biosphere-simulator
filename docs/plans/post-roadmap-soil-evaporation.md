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

⚠ **The event rows below are superseded** by §11a (re-run 2026-10-08 under the settled watering and deep soil).

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

⚠ **The event rows below are superseded** by §11a (re-run 2026-10-08 under the settled watering and deep soil).

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

---

## 10. Re-pricing all three for adoption (the user, 2026-10-07)

**DECIDED (the user): re-price all three together** — the Szeicz–Long canopy resistance, soil
evaporation and watering in events (over resistance + soil with daily watering, and over stopping at
the lab). ⚠ Watering in events in the reference is its own design change: every frozen run waters
continuously today.

**The floor: OFF, my choice on the user's delegation** (*"Choose what is closer to reality"*).
Reason: the model already gives the soil its own energy share, `exp(−0.5·ETLAI)` of the light, never
zero; the book's floor stands in for energy its formula misses outdoors (diffuse sky light,
advection). A sealed lamp-lit chamber has neither, so a fixed 1.5 mm/day would evaporate water with
no energy to pay for it (S3 had to exclude the floor for that reason), and it does not dim with the
lamp. Kept as a switch.

**The candidate reference:** `rs = 100 / min(LAI, 2.0)`; soil evaporation two-stage with the energy
split, the 150 mm top layer capped at silt-loam saturation, no floor; watering on FAO-56's trigger
(`p = 0.55`), the whole deficit per event.

**Precondition built first (on main, frozen-identical):** the station's `sealed_reset_hook` now calls
`annual_reset_with` with the params the season was built from (`params::biosphere()`), which equals
`annual_reset` on every frozen build and re-sows the lab values the plain reset refuses. Proven by the
golden report and the station suite including its ignored tests (§10a).

### 10a. The precondition, proven

`regen_goldens` (report): 20 of 20 identical. `cargo test --release -p station -- --include-ignored`:
241 passed, 0 failed (the sealed station's golden, its band, both session resumes and the
`chamber_walls` trajectory pins among them).

### 10b. The biosphere twins — and a ratchet in the deep store (2026-10-07)

Twins of the six biosphere goldens under all three forms (`W:\temp\claude\step6\reprice\zz_twins3.rs`,
throwaway; each control equal to its committed golden byte for byte; twins in
`W:\temp\claude\step6\reprice\twins\`). Every twin differs, and adds **five aux keys**
(`top_soil_water`, `soil_dry_days`, `soil_shade_lai`, `soil_evap_today`, `soil_potential_today`);
the open season also adds the stock `boundary.soil_evaporation`. Water stocks move; **carbon does not**.

⚠ **Not predicted: the rooted depth moves in the four re-sown goldens** — perennial 5 y 1.300 → **0.332 m**,
consumer 15 y 1.300 → **0.156 m**. Traced (a throwaway probe, `zz_roots.rs`), perennial chamber:

| | day 0 | 120 | 300 (maturity) | 310 (re-sown) | 400 | 500 |
|---|---|---|---|---|---|---|
| frozen — depth / root-zone fill / water below the roots (kg) | 0.15 / 1.00 / 175.5 | 1.30 / 0.99 / 26.0 | 1.30 / 0.95 / 26.0 | 0.23 / 1.04 / 158.5 | 1.18 / 1.00 / 39.4 | 1.30 / 0.97 / 24.2 |
| all three | 0.15 / 1.00 / 175.5 | 1.30 / 0.61 / 26.0 | 1.30 / 0.53 / 26.0 | 0.23 / 0.48 / **91.4** | 0.94 / 0.65 / **0.0** | 0.94 / 0.48 / **0.0** |

**The mechanism — a redistribution, invisible to conservation:** the deepening roots capture the
water below them (the frozen mechanism); at each re-sow the abandoned zone returns its CURRENT water
to the deep store (`resow_water_return`, a fraction of the zone's water), and under events the zone
sits near half full, so it returns about half as much; and an event fills the zone only to its drained
upper limit, so `Drainage` (which moves only water above it) never recharges the deep store. The deep
store therefore empties season by season; once it is empty the roots stop deepening (Soltani &
Sinclair Box 14.1, `If WSTORG = 0`), so the zone ends shallow. The crop is never stressed (fill ≥ 0.44),
so carbon does not move — but the soil profile ratchets.

### 10c. Fixing the ratchet: watering a little over full (the user, 2026-10-07)

**DECIDED (the user): water a bit over full** (over refilling the whole profile, and over accepting
the drain). **Source, read first-hand:** FAO Irrigation Water Management Training Manual No. 4,
*Irrigation Scheduling* (Brouwer, Prins & Heibloem), Ch. 3: *"Not all water which is applied to the
field can indeed be used by the plants. Part of the water is lost through deep percolation and
runoff"*; `d_gross = d_net · 100 / ea`; Annex 1 Table 8, indicative field application efficiencies:
surface 60 %, sprinkler 75 %, drip 90 %. (FAO-56 Ch. 8 itself asks the NET depth not to exceed the
depletion, to avoid percolation — Eq. 88 and its scheduling paragraph — so it gives no excess.)
**The user's values: the open field sprinkler 75 %, the sealed chambers drip 90 %.**

**The mapping:** each event applies `deficit / ea` into the root zone (in a sealed chamber,
`min(condensate, deficit / ea)`); the model's own `Drainage` (Soltani & Sinclair 14.11–14.12, 30 %
a day of the excess above the drained upper limit, into the unbounded deep store `WSTORG`) carries
the extra below the roots — no new path. A sealed chamber cannot lose runoff, so all of its loss
percolates.

**Predictions:** R1 — the deep store no longer empties: the re-sown chambers' rooted depth at the
end of their goldens returns to ~1.30 m (as frozen), not 0.33 / 0.16; R2 — carbon still identical;
R3 — the open field draws more from its water source (gross 1/0.75 of net).

**Measured (the root trace over five seasons, perennial chamber, all three forms with the efficiencies):**
the deep store still empties — 175.5 → 91.4 (re-sow 1) → 0.0 (day 400) → 50.8 → 23.7 → 19.5 → **1.0 kg**
(frozen ~24–159), and the rooted depth ends at **0.50 m** (frozen 1.30; without the over-watering 0.33).
**R1 FAILED:** the over-application slows the drain, it does not stop it. Two reasons: the excess
drains only while the zone is above its upper limit, and the crop and soil pull it back below within
about a day, so little percolates; and at each re-sow the abandoned deep soil keeps the root zone's
fill at harvest (~0.5 under events, ~1.0 frozen), so it returns half as much. ~~In a rain-free chamber watered only around the roots, a slowly drying deep profile may be mostly
real physics~~ (withdrawn, §10d: deep drying is plausible, but WHERE the roots stop is the single
deep store credited at full capacity — `0.233 + 91.36 / 130 = 0.936 m`, exactly the stopping depth).

### 10d. Advisor review, the book's rule, and same-day percolation (2026-10-07)

**Advisor review, summarized:** the stopping depth is not dry soil — `rooted depth at re-sow + deep
store / (EXTR · 1000)` predicts it exactly — so it is the model's ONE undifferentiated deep store,
credited at full capacity per metre of new root, not physics; check the book before saying more. The
watering mapping recharged less than its sources assume: both FAO sources take the loss the SAME
day, so route `gross − net` straight below the roots (a correction of the mapping, not a new user
decision). ⚠ **Stale, labelled:** the six twins in `W:\temp\claude\step6\reprice\twins\` and the
event rows of §8a / §9a were produced before the efficiency change (`bc0bed8`); they predict nothing
now and are regenerated once the watering settles. And the price has grown: three forms became five
lab changes.

**The book, read:** Soltani & Sinclair Eqn 14.12 and p. 175: `EWAT = min(GRTD · EXTR, WSTORG)` — new
root depth is credited at FULL capacity, capped by the single deep store `WSTORG`; `WSTORG = WSTORG +
DRAIN − EWAT`. The model follows the book exactly. The book models ONE season; carried across
re-sowings, its single deep store with full-capacity credit is what stops the roots early. A
limitation of the book's single-season soil model applied over many seasons — not established physics.

**Same-day percolation, built** (`EventIrrigation` / `EventRecycling` now put `net` into the root zone
and `gross − net` into the deep store in the event step). The root trace, perennial chamber:

| day | 0 | 120 | 300 | 310 (re-sown) | 400 | 615 | 920 | 1225 | 1500 |
|---|---|---|---|---|---|---|---|---|---|
| depth (m) | 0.15 | 1.30 | 1.30 | 0.23 | **1.18** | 0.23 | 0.23 | 0.23 | **1.15** |
| water below the roots (kg) | 175.5 | 26.0 | 64.0 | **142.1** | 18.5 | 113.0 | 78.3 | 100.0 | 6.2 |
| frozen, the same | 175.5 / 1.30 … | | | 158.5 | 1.18 / 39.4 | 158.5 | 158.5 | 158.5 | 1.30 / 24.2 |

**R1 mostly held now:** the deep store holds near the frozen run's (season 2's roots reach 1.18 m,
as frozen), with a slower residual decline (1.15 m against 1.30 after five seasons).

### 10e. New roots credited with the deep soil's actual wetness (the user, 2026-10-07)

**DECIDED (the user): credit new roots with the deep soil's actual wetness, not full capacity; accept
the slow residual drying** that remains. Same-day percolation (§10d) is committed first, on its own.

**The form — MINE, on the user's instruction; NOT in the book.** A fourth lab switch, a sibling of
the watering form (`DeepSoilCredit`; the loader keeps the book's `FullCapacity`, and its code path
computes no ratio at all, so the frozen runs stay bit-identical). Under `ActualWetness`, the water a
metre of new root captures is the book's scaled by how wet the deep store is:

```
EWAT = GRTD · EXTR · ρ · A · min(1, WSTORG / ((SOLDEP − DEPORT) · EXTR · ρ · A))
```

— the deep store spread evenly over the unrooted soil, which is how the book itself seeds it (Eqn
14.28, `WSTORG = (SOLDEP − DEPORT)·EXTR·ρ·A·MAI`, a uniform fraction `MAI`). It reduces to the book's
Eqn 14.10 exactly when the deep store is at capacity — the book's single-season case. Capped at 1:
over-watering can push the store past its own capacity (Eqn 14.12 has no outflow), and the credit
must never exceed the book's. Only the water side changes; the rooting RATE and its four stops are
untouched (`extension_rate` stays the single source).

**Predictions (before code):**

* **P1 (exact, tests the build): capture leaves the deep soil's wetness unchanged.** Taking
  `Δd · W/(S−d)` from `W` over `S−d` leaves `W/(k(S−d))` the same. So the dry-subsoil stop (`WSTORG =
  0`) essentially never fires, and **the roots reach 1.30 m every season by construction** — rooted
  depth stops being the readout. **The new readout:** the deep store's wetness at each re-sowing, and
  the root zone's fill once the roots are at full depth.
* **P2: where the wetness drifts.** Only two things move it: drainage and percolation push it UP;
  a re-sowing mixes the abandoned zone back in at its harvest fill (~0.5 under events). So under events
  the deep wetness should settle somewhere between the harvest fill and 1 — the residual drying, now
  a measured fraction, not a stopping depth.
* **P3: equal to the book while the deep store is full.** Every first season starts at MAI = 1, and
  under continuous watering the drainage keeps the 0.2 m below 1.30 m at exactly 26.0 kg = capacity
  (§10b). So the new form ALONE, continuous watering: the three never-re-sown goldens and every first
  season equal the book to rounding; after the first re-sowing the deep store is 158.5 / 164.7 kg =
  0.962 full, so season 2's new roots get ~4 % less water per metre.
* **P4: the depth trace under all forms + percolation + this:** as frozen (1.18 m at day 400; **1.30 m
  at day 1500**, against 1.15 now), because the depth rate reads the root zone's stress, and events
  trigger at FTSW 0.45, above `wssg` 0.30.
* **P5: carbon may move now — down, if at all.** Less water per metre of new root makes the deep part
  of the zone drier: events come sooner (more percolation — a feedback that recharges the deep store),
  and any drought response reading the zone fill above the trigger could bite. Expected small.

### 10f. BUILT lab-only and measured — graded (2026-10-07)

Built as §10e describes (`science::DeepSoilCredit`, `deep_soil_wetness`, `lab::with_deep_credit`;
`RootZoneCapture` reads `p.water.deep_credit`). New unit test: a deep store seeded half full keeps
its wetness at 0.5 on every capture step and the roots reach the cap, while the book's credit empties
it and stops the roots at `0.15 + W/(EXTR·ρ·A)`. Throwaway probes (`W:\temp\claude\rootcredit\
zz_rootcredit.rs`, `zz_rootcredit15.rs`; outputs `probe.log`, `probe15.log`), perennial chamber.

**Fifteen years, at each harvest** (deep wetness = the store below the roots over its capacity,
just after the re-sowing):

| | roots, every season | deep wetness after re-sowing | water over the 0.2 m layer's capacity at harvest | air + condensate | peak grain s1 → s14 |
|---|---|---|---|---|---|
| frozen | 1.30 m | 0.956 | 0 | 8.4 kg | 0.7127 → 0.5185 |
| all forms, book credit | 1.30 → **0.90 m** | 0.87 → 0.50 → **~0.44** | 38 → 0 | 30 → ~104 kg | identical |
| all forms, actual wetness | **1.30 m, all 15** | 0.87 → 0.76 → **0.69** (flat from s9) | 38 → **~19, flat** | 30 → ~65 kg | identical |

**Graded:**

* **P1 HELD (exact):** within a season the wetness stays put under capture (0.880 at days 310 and 400);
  the roots reach 1.30 m in all fifteen seasons.
* **P2 HELD:** it moves only up within a season (percolation) and is mixed down at each re-sowing; it
  settles at **0.69** — between the harvest fill and 1 — and stops falling. This is the slow residual
  drying the user accepted: **a level, not a ratchet** (0.956 frozen → 0.69).
* **P3 HELD:** the form alone, continuous watering: season 1 identical to the book (deep wetness 1.000
  throughout); after the first re-sowing 0.964 vs 0.962, and the re-sow wetness 0.957 either way.
* **P4 HELD:** 1.184 m at day 400 as frozen; 1.30 m at day 1500 (1.15 under the book's credit).
* **P5: carbon did NOT move** — peak grain identical to four digits in all three, every season. The
  crop is never water-stressed under either credit (the events trigger above `wssg`).

**Found, not acted on:** the 0.2 m below the full-depth roots holds ~19 kg ABOVE its capacity at every
harvest (Eqn 14.12's store has no outflow, and the credit is capped at 1, so percolated water waits
there until the next re-sowing mixes it up). It is bounded — flat from season 9, and the chamber's free
water does not drain into it (~65 kg held) — so it is a resting place, not a leak. In a real chamber
that water would pond at the bottom of the container. Left as is; the user's to reopen.

**Advisor review of §10f, summarized (2026-10-07):** both commits sound; but (1) **the 69 % level is
partly the overfilled layer's water** — each re-sowing mixes the ~19 kg held above the bottom layer's
capacity back into the deep store; without it the level is ≈ **0.58** (`(0.693·164.7 − 19.3)/164.7`).
The roots result does not depend on it (it holds by construction). The overfill now bears on the number
the user accepted, so it is the user's decision, not a footnote. (2) Measured on one scenario, not the
worst — the consumer chamber was the old ratchet's worst case (0.156 m); run it. (3) Push and watch
Linux CI.

**The consumer chamber, fifteen years** (`W:\temp\claude\rootcredit\probe15_consumer.log`): roots
**1.30 m in all fifteen seasons** (book credit: 1.30 → ~0.70–0.80 m); the deep wetness after re-sowing
swings **0.68–0.95** with no trend (it follows the root zone's fill at harvest, i.e. how recently the
last event came), ≈ 0.57–0.83 without the overfill; the overfill 16–22 kg, bounded; peak grain identical
in all three. **P1–P4 hold here too.**

**Corrected statement of the accepted drying:** the deep soil settles at roughly 0.6–0.7 of full after
each re-sowing (perennial 0.69, consumer 0.68–0.95), of which ~0.1 is water held above the bottom
layer's capacity; ≈ 0.58 without it.

### 10g. The deep overfill drains and is recycled (the user, 2026-10-08)

**DECIDED (the user): *"Let it drain and be recycled."*** The ~19 kg held above the bottom layer's
capacity (§10f) leaves the soil and returns to the water the crop is watered from.

**The form — MINE, on the user's instruction; NOT in the book** (Eqn 14.12 gives `WSTORG` no outflow).
A fifth lab switch, a sibling of `DeepSoilCredit` (`DeepOverflow`; the loader keeps the book's `Held`,
and on `Held` no flow is registered at all, so the frozen flow set — and every golden — is untouched).
Under `Recycled`, one new flow `SubsoilOverflow`:

```
excess = WSTORG − (SOLDEP − DEPORT) · EXTR · ρ · A          (the store's own capacity, as §10e)
OVERFLOW = max(0, excess) · DRAINF · dt                      (DRAINF = the scenario's 0.3 / day)
WATER  subsoil_water → condensate (sealed)  |  → water_source (open field)
```

* **The rate is the book's own** Eqn 14.11 drainage factor — the rule [F] applies to water above the
  root zone's upper limit, applied one store down. Not same-day: the root zone's drainage is not
  same-day either, and a pulse of event loss then leaves over ~3 days (e-folding 1/0.3).
* **The destination is the watering reservoir:** in a sealed chamber the condensate store that
  `EventRecycling` draws from (a container's drain collected back to the drip tank); in the open field
  the irrigation source boundary (drainage collected and returned to supply — our mapping, changeable;
  the open field is single-season and barely reaches the overflow).
* **Stays capped at 1:** `deep_soil_wetness` keeps its clamp (a pulse may sit above capacity for a few
  days before it drains).
* Bounded by construction: overflow ≤ 0.3·dt·excess < excess, and capture takes at most the capacity
  of the newly explored slice, so the two outflows never overdraw the store (no rationing).

**Predictions (before code), all lab forms + actual-wetness credit + this, fifteen seasons:**

* **Q1 (tests the build):** at each harvest the water below full-depth roots sits within a few kg of
  its 25.97 kg capacity (residue of the last event's pulse only), not 16–38 kg over.
* **Q2 (the number the user accepted):** the deep wetness after each re-sowing falls from 0.69 to
  ≈ **0.58** in the perennial chamber (the advisor's `(0.693·164.7 − 19.3)/164.7`), minus a little
  feedback: a drier deep store credits new roots less water, events come sooner, more percolates — and
  that percolation now leaves rather than accumulating. Consumer chamber: ≈ 0.57–0.83, no trend.
* **Q3:** free water (air + condensate) rises by about what no longer sits in the soil: perennial
  ~65 → ~85 kg.
* **Q4:** roots still reach 1.30 m every season (by construction, §10e P1).
* **Q5:** carbon identical to four digits (events still trigger above `wssg`).
* **Q6:** the frozen path bit-identical (20/20 goldens); the switch off registers nothing.

### 10h. BUILT lab-only and measured — graded (2026-10-08)

Built as §10g (`science::DeepOverflow`, `flows::SubsoilOverflow`, `lab::with_deep_overflow`; registered
in `system.rs` only under `Recycled`). Two new unit tests: the flow on constructed stores (nothing at or
below capacity; a 0.3 share of the EXCESS, times `dt`, into the reservoir; a deeper root leaves less room;
full-depth roots make the whole store overflow; the donor clamp at `DRAINF = 5`), and the switch (the
book's `Held` registers no overflow flow; `Recycled` returns to the condensate in the perennial chamber
and to the irrigation source in the open field). ⚠ Checked first (advisor): lab flows never enter a
frozen manifest's flow list (those are written from the frozen builds), and the station builds its
greenhouse through the same builder and filters flows only by named ids, so the flow reaches it.

Throwaway probe `W:\temp\claude\overflow\zz_overflow15.rs` (output `probe15.log`), fifteen seasons, all
lab forms + actual-wetness credit, with the overflow `Held` (§10f) and `Recycled`:

| | over capacity at harvest | over capacity, worst step | deep wetness after re-sowing | free water (air + condensate), late seasons | roots | peak grain |
|---|---|---|---|---|---|---|
| perennial, held | 19–38 kg | 38.0 kg | 0.67–0.94, last six mean **0.684** | ~65 kg | 1.30 m | — |
| perennial, **recycled** | 0.00–0.16 kg (one season 7.8) | 9.3 kg | 0.55–0.98, last six mean **0.626** | ~70–79 kg | 1.30 m, all 15 | identical |
| consumer, held | 16–32 kg | 31.8 kg | 0.68–0.95, last six mean 0.813 | 12–65 kg | 1.30 m | — |
| consumer, **recycled** | 0.00–0.05 kg (two seasons 0.5, 1.5) | 9.3 kg | 0.55–0.84, last six mean **0.689** | 31–90 kg | 1.30 m, all 15 | identical |

Rationing 0 in all six runs (two outflows now draw on the deep store in one step).

**Graded:**

* **Q1 HELD:** the deep store sits at its capacity (25.97 kg) at harvest; what remains above it is the
  tail of an event's pulse, gone in days (worst step 9.3 kg, against 38).
* **Q2 PARTLY:** the level fell — perennial 0.684 → **0.626** — but less than the predicted ≈ 0.58, and
  the feedback I predicted (drier deep soil → more percolation → lower still) had the WRONG sign. Consumer
  within the predicted 0.57–0.83, no trend. **Why — measured, see "The tank caps every deep watering"
  below:** the 19 kg drained into the condensate is spent on the next waterings, so the root zone is
  fuller after each event and returns more water to the deep store at each re-sowing.
* **Q3 PARTLY:** free water rose by ~9 kg, not ~20 — the rest is in the fuller root zone (water conserved:
  held 45.2 + 84.8 + 64.9 = 194.9 kg; recycled 26.0 + 94.9 + 74.1 = 195.0 kg).
* **Q4 HELD:** roots 1.30 m in all fifteen seasons, both chambers.
* **Q5 HELD:** peak grain identical to four digits, every season, both chambers.
* **Q6 HELD:** `regen_goldens` report 20 of 20 identical.

**The accepted drying, restated:** with the overfill drained and recycled the deep soil settles at about
**0.6–0.7 of full after each re-sowing** (perennial mean 0.63, consumer 0.69), now with no stranded water
in it. That is the honest number the user's decision was after.

**Advisor review of §10h, summarized (2026-10-08):** the build, tests, census and frozen check are
sound; but my first explanation of Q2/Q3 ("about half the drained water came back into the root zone")
was asserted from single harvest snapshots that swing 79–157 kg with event timing. Its sharper candidate,
from the totals: a sealed chamber holds ~195 kg; refilling a 1.30 m root zone from the trigger (0.45)
needs ~93 kg net, ~103 kg gross at drip 90 %, but the air and condensate together hold only ~74–93 kg —
so a deep watering may never deliver the whole deficit. Count it.

**The tank caps every deep watering — measured** (throwaway `W:\temp\claude\overflow\zz_capped.rs`,
output `capped.log`; an event = one step where the zone gains and the condensate loses > 3 kg; capped =
the gross wanted, `deficit / 0.90`, exceeds the condensate held). Fifteen seasons:

| | events, roots at full depth | capped by the condensate | zone fill after, mean (min) | events, shallower roots | capped |
|---|---|---|---|---|---|
| perennial, held | 52 | **52** | 0.910 (0.773) | 13 | 0 |
| perennial, recycled | 48 | **48** | 0.955 (0.943) | 22 | 0 |
| consumer, held | 43 | **43** | 0.918 (0.808) | 10 | 0 |
| consumer, recycled | 42 | **41** | 0.956 (0.941) | 17 | 0 |

* **Every watering of a full-depth root zone in a sealed chamber is limited by the tank**, held or
  recycled; none while the roots are shallow. The fill after an event is whatever water is not in the
  deep store or the air — so moving the 19 kg out of the deep store raises every deep refill (0.91 →
  0.955), which is the Q2/Q3 mechanism.
* ⚠ **For the pricing:** §10's candidate reference says *"the whole deficit per event"*. In the sealed
  chambers that is physically impossible at full depth: the chamber holds too little free water. The
  crop is never stressed by it (carbon identical), but any pricing that reads event sizes or the
  watering rule must state that the sealed chambers water from a short tank.

---

## 11. The stale comparisons re-run under the settled forms, then the price (the user, 2026-10-08)

**Asked (the user):** *"re-run the out-of-date comparison runs under the final settings, then estimate
together what making the three changes official would cost."* The three changes are Szeicz–Long, soil
evaporation and watering in events — but watering in events needs the two deep-soil fixes the user
chose (§10e, §10g), or the root ratchet of §10b returns. **So the candidate is five loader settings:**
`rs_form = SzeiczLong`, `soil_evap = TwoStage { floor: false }`, `watering = Fao56Trigger` (its
application efficiencies — sprinkler 0.75, drip 0.90 — and same-day percolation come with it, in
`system.rs`'s two watering builders), `deep_credit = ActualWetness`, `deep_overflow = Recycled`.

### 11a. The event rows of §8a / §9a, re-run (they supersede those rows)

`measurement()` in `domains/tests/soil_evaporation.rs` now runs its event rows under the settled deep
forms (a `settled` helper; the frozen, S–L-alone and daily rows unchanged and reproduced to the digit).
Log: `W:\temp\claude\threeforms\measure.log`.

| run | config | water out (kg/yr) | of it soil | at LAI < 0.5 | Stage I share | events | lowest FTSW | dead-crop vapour / frozen | dead-crop soil / frozen crop water | peak top fill |
|---|---|---|---|---|---|---|---|---|---|---|
| default (1 y) | + soil, events | 536.26 | 33.63 | 38.20 | 0.204 | 5 | 0.448 | — | 0.744 | 2.65 |
| | + soil + floor, events | 592.70 | 88.57 | 38.20 | 0.180 | 6 | 0.449 | — | 0.744 | 2.65 |
| sealed chamber (3 y) | + soil, events | 156.67 | 38.66 | 37.88 | 0.091 | 5 | 0.449 | 0.176 | 0.011 | 2.65 |
| | + soil + floor, events | 157.32 | 39.32 | 38.20 | 0.088 | 5 | 0.449 | 0.176 | 0.011 | 2.65 |
| perennial (5 y) | + soil, events | 351.49 | 109.48 | 123.61 | 0.204 | **23** | 0.448 | 0.999 | 0.438 | 2.65 |
| | + soil + floor, events | 355.18 | 113.17 | 126.93 | 0.212 | 24 | 0.445 | 0.999 | 0.486 | 2.65 |
| consumer (5 y) | + soil, events | 298.57 | 102.20 | 145.07 | 0.174 | **21** | 0.447 | 0.992 | 0.424 | 2.65 |
| | + soil + floor, events | 299.41 | 103.07 | 143.38 | 0.177 | 21 | 0.447 | 0.994 | 0.418 | 2.65 |

Carbon bit-identical to frozen in every row. `drought_window` under events: first stressed day 244
(floor 240), as §9a.

**What changed against §9a:**

* **The open field: nothing** (single season, the deep store full throughout, so the credit equals the
  book's; the efficiency changes only what the source supplies, not what leaves the root zone).
* **The re-sown chambers water far less often** — perennial 63 → **23** events in five years, consumer
  25 → 21 — and lose a quarter less water (perennial 481.79 → 351.49 kg/yr; consumer 333.34 → 298.57).
  The roots now reach 1.30 m every season, so the root zone is deep and each watering (capped by the
  tank, §10h) refills a large store: fewer, larger events, and a surface that is wet less often (the
  perennial Stage I share 0.416 → 0.204).
* **The soil replaces more of the dead crop's phantom water:** perennial 0.220 → **0.438**, consumer
  0.209 → **0.424** of what the frozen dead crop transpired over its dead days; the air holds (0.999,
  0.992).
* **The never-re-sown sealed chamber is unchanged in kind:** its dead phase still dries the air (0.176).
  Re-sowing, not watering, is what keeps a sealed chamber's air.
* The floor still adds almost nothing in the chambers (≤ 1 %): the switch stays off as decided.

### 11b. The biosphere twins, regenerated

`W:\temp\claude\threeforms\zz_twins5.rs` (throwaway; staged into `domains/tests/` for one run and
removed). Each CONTROL equals its committed golden byte for byte; twins in
`W:\temp\claude\threeforms\twins\`; the diff (`twins_diff.txt`) by key:

| golden | moves (end state) | new keys |
|---|---|---|
| `season_euler` | soil water −41.7 %; vapour sink −14.4 % (transpired); water source drawn −20.6 %; subsoil +0.5 % | the 5 soil aux keys; stock `boundary.soil_evaporation` (33.63 kg) |
| `sealed_chamber` | condensate 8.15 → 52.33 kg; soil −27.4 %; **air vapour 0.25 → 2.0e-6 kg** | the 5 soil aux keys |
| `perennial_chamber` / long horizon | condensate 8.15 → 76.10 / 69.63 kg; soil −42.9 % / −38.9 %; subsoil +7.2 % (to its capacity, 25.97) | the 5 soil aux keys |
| `consumer_chamber` / long horizon | condensate 8.14 → 60.87 / 48.78 kg; soil −33.6 % / −26.2 %; subsoil +7.7 % / +7.9 % | the 5 soil aux keys |

**Water only:** no carbon, nitrogen, oxygen stock or `rooted_depth` / `thermal_time` /
`vernalization_days` moves in any twin. Water moves from the soil to the chamber's free water
(the root zone sits between the trigger and full instead of near full).

### 11c. Predictions for the branch run (written before it)

Method as the canopy pricing (`post-roadmap-canopy-resistance.md` §9–§10): a local branch
`wip/three-forms-price` from main, the five loader lines only, `regen_goldens` in REPORT mode, the full
suite `--no-fail-fast`, the ignored tests in release; then a throwaway `--write` to count the manifest
reds; then back to main. Never pushed.

| | prediction |
|---|---|
| V1 | `regen_goldens` report: the six biosphere goldens + `lighting`, `greenhouse`, `harvest`, `sealed_station` would change (10); each biosphere one equals its §11b twin byte for byte. Unchanged: `drift_summary` (leaf carbon only), `sealed_energy_drift_summary`, the eight crop-free goldens, `state_snapshot.json` |
| V2 | the four station goldens: water stocks move and the 5 soil aux keys appear; no `boundary.soil_evaporation` (sealed). Carbon: **uncertain** — the station's chambers may hold less free water than the biosphere's, and a capped watering could stress the crop; classified after |
| V3 | the station registries carry `SoilSurfaceAccount`, `SubsoilOverflow` once each and the event watering under its old id (`biosphere.recycling`), counted on the branch |
| V4 | red by design: `golden_regression` and `tier_contract` on the moved goldens; the domains manifest writer **even in report mode** (new flow/aux types enter the frozen registries — unlike the canopy price, where only values moved); canopy P1; scorecard R1; soil-evaporation S1; the deep-water rescue's grain pin; the five-year re-sow cycle test; any test asserting the loader keeps the book's credit or `Held` |
| V5 | possibly red, classified after: tests pinning a water stock or humidity in a run with a crop; the watering-trigger and air-split tests (the station's own trigger now meets a biosphere that waters in events) |
| V6 | green: every carbon, energy and gas test; the conservation and determinism laws; `chamber_walls` trajectory pins |

### 11d. The price, measured on a local branch (2026-10-08) — graded

Branch `wip/three-forms-price` (local, never pushed): `196f875` the five loader lines; `3563e06` a second
line the run forced (below). Logs: `W:\temp\claude\threeforms\` (`regen_report.log`, `suite.log`,
`ignored.log`, `suite_after_write.log`, `station_diff.txt`, `manifest_diff.txt`). Main untouched.

**Forced before anything ran — a cost the twins could not show:** `regen_goldens` panicked at once. The
golden producer re-sows the perennial chambers through the plain `annual_reset`, which REFUSES a state
carrying the soil account (built that way so a lab state could never be re-sown wrongly). The branch
routed `run_perennial` through `annual_reset_with(.., &params::biosphere())` to price the rest — which
itself reddens `param_funnel` (a second production param load). An adoption has to choose the real
route: let the plain reset reset the soil values from the one frozen param load.

| | predicted | measured | grade |
|---|---|---|---|
| V1 | 10 goldens change; biosphere ones equal their twins | **20 run, 10 would change** — the predicted ten; all six biosphere goldens equal their §11b twins **byte for byte** | HELD |
| V2 | station goldens: water only, 5 aux keys; carbon uncertain | water only; the 5 aux keys; **no carbon value moves** in any of the four | HELD (carbon: unchanged) |
| V3 | each station registry carries the new flows once, the watering under its old id | greenhouse, lighting, harvest, sealed station: `biosphere.recycling = EventRecycling`, `SubsoilOverflow` and `SoilSurfaceAccount` exactly once each; the cabin / power / fast registries none | HELD |
| V4 | the reds by design | all the named ones red; the domains manifest writer red in report mode as predicted; after `--write` the station's too | HELD |
| V5 | possibly red | the watering-trigger and air-split tests stayed **green** | none fired |
| V6 | carbon, energy, gas, laws, `chamber_walls` green | green; both sealed session resumes green | HELD |

**Suite:** 1337 passed, **24 failed**, 8 ignored (debug); ignored in release: 6 passed, 2 failed (the
sealed station's golden and band). After the throwaway `--write`: 21 failed (the four golden/band reds
go, the station manifest writer joins). Then discarded (`git checkout`), main restored.

**The 24 + 2 reds, classified:**

| class | tests | what an adoption does |
|---|---|---|
| goldens and bands (6) | `golden_regression` ×2 + the expensive sealed station, `tier_contract` ×2 + the expensive band | regenerate (all four band reds cleared on the throwaway `--write` alone — measured; the real band cost is the cross-port line below) |
| manifests (2) | domains + station `manifest_writer` | regenerate: 10 `golden_sha256` rows; `flow_set` −`Irrigation` −`Recycling` +`EventIrrigation` +`EventRecycling` +`SubsoilOverflow`; `aux_set` +`SoilSurfaceAccount` (26 lines) |
| "the old form through the switch is frozen" (4) | canopy P1 and its exact-by-hand check, soil-evaporation S1, scorecard R1 | restate against the new loader (the by-hand check lacks the soil split) |
| tests of the BOOK's deep store that took the loader as "the book" (6) | dry store stops roots, clean vs naive deep-water controls, the clamp on an emptying store, the actual-wetness credit, the overflow switch, the deep-water grain rescue (1.0 vs 1.22) | pin the old forms these tests assume — the credit and overflow, and likely the watering too (unverified: event watering's same-day percolation also wets the below-root store a "dry subsoil" test relies on); with the new credit the roots reach 1.30 m by construction |
| the re-sow route (4) | three tests calling the plain `annual_reset` on a loader state + `param_funnel` | the design item above |
| cycle-flow ring check (1) | `biosphere.recycling` returns no legs at a state where no event is due | construct the state below the trigger |
| re-sow fixed point (1) | `the_resow_makes_a_cycle_and_not_a_ratchet…`: the below-root store 125.7, 126.2, 134.6, 95.9 | restate: under events each cycle depends on when the last watering fell — it cycles, it does not settle |
| **behaviour findings, not yet explained (3)** | `drought_acceleration…`: the 1 + WSSD bound on season thermal time exceeded on the manufactured dry chamber; lab knockout: dropping root capture no longer kills the perennial runs; lab non-finite check: an infinite param now fails at a new aux (`soil_shade_lai`) and is reported as an error, not "dead" | each needs a look before adoption |

**What moves (end states):**

| golden | moves |
|---|---|
| `season_euler` | soil −41.7 %; transpired −14.4 %; drawn from the source −20.6 %; +`boundary.soil_evaporation` 33.6 kg |
| `sealed_chamber` | condensate 8.2 → 52.3 kg; soil −27 %; **air vapour 0.25 → 2e-6 kg** (the dead-crop years, §8c, unchanged in kind) |
| `perennial_chamber` / long | condensate 8.2 → 76.1 / 69.6 kg; soil −43 % / −39 %; subsoil +7 % (to capacity) |
| `consumer_chamber` / long | condensate 8.1 → 60.9 / 48.8 kg; soil −34 % / −26 %; subsoil +8 % |
| `greenhouse` (7 d) | condensate −59 %; vapour −63 %; soil +13 % |
| `harvest` (7 d) | condensate −66 %; vapour −73 %; soil +2.6 % |
| `lighting` (7 d) | condensate −78 %; soil +10 % |
| `sealed_station` | condensate 5.5 → 71.6 kg; soil −38 %; subsoil −15 % |
| the other ten | identical |

**Water only:** no carbon, nitrogen, oxygen, energy, root depth or development value moves anywhere.

**⚠ The headline for the decision: the package does NOT fix the defect that opened this item in one
golden.** `sealed_chamber_state.json` (never re-sown) would be frozen with **2e-6 kg of air vapour against
0.25** — the success test S4 still fails under events (0.176, §11a). The mechanism, read in the code: the
soil's evaporation is capped by the top-layer account (`Transpiration::split`, `room = top.min(..)`, and
Stage I needs > 1 mm in it), and the account's ONLY inflow is the watering flow's legs into the root zone
(`SoilSurfaceAccount`, `inflow`; a re-sowing also re-seeds it). Under events a chamber whose deep root zone
never reaches the trigger is never watered, so once the top empties its soil evaporation is **zero for
good** — the dry air comes from the model having no upward supply into the top layer (capillary rise),
not from a soil that has physically dried. The account ends at exactly 0 in all ten moved goldens; the
share of steps with an empty top was not measured. Under daily watering S4 held (0.999), because the
top is refilled every step.

**Ceremony the run cannot show:**

1. **Constants into param files with their sources** (the reference may not hard-code coefficients):
   `100` s/m and the leaf-area cap (Szeicz–Long), `0.55` (FAO-56), `0.75` / `0.90` (FAO Manual 4), soil
   albedo `0.12` and extinction `0.5` (Soltani & Sinclair), the top layer `0.15` m, its wet threshold,
   the Stage I fill `0.5`, silt loam `SAT − DUL` (Table 13.1). Each edited file's `param_files` row
   regenerates. The floor `1.5` mm/day stays lab-only.
2. **Lab forms kept or retired:** the constant 70, the book's full-capacity credit and `Held` become the
   old forms — kept as lab switches (the deep-store tests above need them) or retired.
3. **Prose:** each contract's dated entry; "every frozen run waters continuously" and "transpiration
   does not read the canopy" become false; §10's "the whole deficit per event" restated — the sealed
   chambers water from a short tank (§10h).
4. **The separate-air lab option** (`air_split.rs`): its `TriggeredWatering` does not feed the top-layer
   account — an open lab gap, not in any golden.
5. **The cross-port tolerance contract (`docs/native-port-reference.md`) is part of the unfreeze, and
   UNMEASURED.** The soil path adds new transcendental sites to all ten moved goldens — `sqrt` in Stage
   II (`stage_two_factor`) and `exp` in the soil's shade — which `rust/data/tiers.json`'s
   `_transcendental_sites` and each golden's `transcendentals` list do not name. On Windows the bands
   demand byte-exactness, so local green says nothing about Linux; measuring them needs the branch run
   on Linux CI (a push — the user's call).
6. **Potato** adopts with wheat (§11 of the canopy plan, decided); its tests stayed green.

## 12. The top-layer cap is OUR departure from the book — a switch, measured (the user, 2026-10-08)

**Asked (the user, on the recommendation after §11d):** fix the price's headline — the never-re-sown
sealed chamber's air still dries — before anything is adopted. The recommendation was "an upward
supply into the top layer (capillary rise)". **Re-read first-hand, that framing was wrong:**

* The book leaves capillary rise out on purpose: *"capillary rise of water may occur if there is a high
  water table in the soil, but it is not considered here"* (p. 171). A chamber has no water table, so a
  capillary-rise form would be uncited (WHAT-IF).
* **The book's own program never caps the soil's evaporation at the top layer's water** (Box 14.1,
  p. 187): `SEVP` is set from `EOS` and the Stage II factor alone; `ATSW1 = ATSW1 + … − TR1 − SEVP`,
  then `If ATSW1 < 0 Then ATSW1 = 0`; and `ATSW = ATSW + … − TR − SEVP` takes it from the whole root
  zone. Stage II is *defined* as the regime after the top layer has dried (p. 181), so Stage II's
  √-decline IS the book's supply from below; the account only floors at 0.
* Our `Transpiration::split` caps it: `room = top.min(..)`. Introduced with the build (`09db450`);
  §4b's "all of the soil evaporation [comes out of the top account]" was read as a cap. **Not a user
  decision and not a fix for a measured problem — a mapping choice made at build time.**

**Advisor review (2026-10-08), summarized:** drop capillary rise; this is correcting a departure, not
adding science. Do not claim it fixes S4: the book's Stage II factor `√(d+1) − √d` decays toward zero,
so a never-watered chamber still decays by construction, only slower. Compute (a) the flux that holds
the air, (b) the uncapped Stage II flux over the dead years, (c) whether the slow drain reaches the
trigger in the run (a watering would restart Stage I — the feedback may decide S4). Build it as a
switch beside the cap, both in one table. Afterwards re-price §11d: every event row moves.

**A second departure, logged, NOT changed here:** the book starts and re-starts `DYSE` at **1**
(`DYSE = 1` at initialisation, Box 14.1 p. 185; `If (RAIN + IRGW) > WETWAT Then DYSE = 1`), so its first
Stage II day runs at `√2 − 1 ≈ 0.41` of `EOS`; ours resets `soil_dry_days` to 0, a first day at
**1.0**. Its own item.

**The switch:** `SoilEvaporationForm::TwoStage { floor, supply }`, `SoilSupply::TopLayer` (the cap,
what §8–§11 measured) or `SoilSupply::RootZone` (the book's program: capped only by the root zone's
water after the crop's share). The account is unchanged (it already floors at 0).

### 12a. Predictions (written before the build)

Numbers: the sealed chamber is 1 m², its condenser takes `0.5 · min(v, target)` per day
(`condensation_rate` 0.5), the target ≈ 0.25 kg.

* **(a)** to hold the air at its target needs ≈ **0.125 kg/day** of evaporation.
* **(b)** the soil's wet-surface potential in that chamber ≈ 1.07 kg/day (§9a's daily-watering row,
  392.40 kg/yr at a Stage I share of 1.000). Over the dead years `d` runs into the hundreds, the factor
  `≈ 1/(2√d)` ≈ 0.05 → 0.02, so the uncapped flux is ≈ **0.02–0.05 kg/day** — below (a). The air
  settles near `flux / 0.5` ≈ 0.04–0.1 kg.
* **(c)** the cumulative Stage II drain over ~2 dead years ≈ `EOS · √730` ≈ 29 kg against a 1.3 m root
  zone's ≈ 93 kg to the trigger. Whether it reaches the trigger depends on where the zone stands when
  the crop dies (not known before the run): **not predicted**; if it fires, the air recovers in pulses.

| | prediction |
|---|---|
| P1 | sealed chamber, events: the dead-crop vapour ratio **rises** from 0.176 but stays **well below 0.9** — S4 still fails; the end-state vapour rises from 2e-6 kg to ~0.04–0.1 kg |
| P2 | every event row: soil evaporation and water out of the root zone **rise**; Stage I share unchanged or lower; events equal or more; lowest FTSW still ≈ the trigger (0.45) |
| P3 | carbon bit-identical to frozen in every row (the trigger at 0.45 sits above the stress threshold) |
| P4 | daily-watering rows: essentially unchanged (the top is refilled every step, so the cap rarely binds) |
| P5 | `drought_window` under events: the first stressed day comes **earlier** than 244 |
| P6 | the four instrument tests stay green on the cap (they keep `TopLayer`) |

### 12b. Measured (2026-10-08) — graded

`measurement()` gained the row `S-L+soil, ev, book` (`SoilSupply::RootZone`, floor off, events, the
settled deep forms) and an end-of-run air-vapour column. Log: `W:\temp\claude\caprise\measure.log`.
Every other row reproduced §11a to the digit.

| run | cap | water out (kg/yr) | of it soil | Stage I share | events | lowest FTSW | carbon = frozen | dead-crop vapour / frozen | dead-crop soil / frozen crop water | end vapour (kg) |
|---|---|---|---|---|---|---|---|---|---|---|
| default (1 y) | top layer | 536.26 | 33.63 | 0.204 | 5 | 0.448 | yes | — | 0.744 | — |
| | **book** | 548.51 | 45.83 | 0.200 | 5 | 0.448 | yes | — | 0.810 | — |
| sealed chamber (3 y) | top layer | 156.67 | 38.66 | 0.091 | 5 | 0.449 | yes | 0.176 | 0.011 | 1.95e-6 |
| | **book** | 187.94 | 70.74 | 0.157 | 6 | 0.450 | yes | **0.605** | 0.040 | **0.125** |
| perennial (5 y) | top layer | 351.49 | 109.48 | 0.204 | 23 | 0.448 | yes | 0.999 | 0.438 | 0.2505 |
| | **book** | 419.00 | 176.93 | 0.223 | 26 | 0.447 | yes | 0.999 | 0.447 | 0.2505 |
| consumer (5 y) | top layer | 298.57 | 102.20 | 0.174 | 21 | 0.447 | yes | 0.992 | 0.424 | 0.5010 |
| | **book** | 381.78 | 185.49 | 0.209 | 26 | 0.447 | yes | 0.996 | 0.443 | 0.5010 |

`drought_window` under events: first stressed day 244 (cap) → **241** (book).

| | predicted | measured | grade |
|---|---|---|---|
| P1 | dead-crop vapour rises from 0.176, stays well below 0.9; end vapour ~0.04–0.1 kg | **0.605**; end vapour **0.125 kg** (half the 0.25 target) | HELD in kind; the end vapour **above** the range (≈ 0.06 kg/day of evaporation, not 0.02–0.05) |
| P2 | soil and water out rise in every event row; Stage I share unchanged or lower; events equal or more; lowest FTSW ≈ 0.45 | soil +36 % / +83 % / +62 % / +81 %; events 5/6/26/26 against 5/5/23/21; FTSW ≈ 0.45 — HELD; **the Stage I share ROSE** in three of four (sealed 0.091 → 0.157): faster drying → more watering events → the top wetted more often | HELD except the Stage I share |
| P3 | carbon bit-identical everywhere | identical in every row | HELD |
| P4 | daily-watering rows essentially unchanged | **not measured** — no daily row runs under the book's supply | NOT MEASURED |
| P5 | drought's first stressed day earlier than 244 | 241 | HELD |
| P6 | the instrument tests stay green on the cap | green; full suite 1361 passed, 0 failed, 8 ignored; clippy clean (logs `suite.log`, `clippy.log` beside the measurement) | HELD |

**What it says.** Following the book's program instead of our cap roughly doubles the soil's share of
the water use in the chambers and lifts the never-re-sown chamber's dead-crop air from 18 % to 61 % of
the frozen model's. **It does not make S4 pass.** Where the remaining gap comes from: §12c. The bigger
soil loss is paid for with more watering, not with carbon.

### 12c. When the waterings fell — the attribution checked (advisor, 2026-10-08)

**Advisor review of §12b, summarized:** "the remaining gap is the book's own Stage II decline" was an
untested attribution: the extra watering (5 → 6) and the end vapour above the predicted range both fit
a watering IN the dead years, which would make part of 0.605 a pulse, not the tail. Probe it. Also: the
local `wip/three-forms-price` branch no longer compiles over this commit (its loader line has no
`supply`) — re-pricing means choosing `supply` there, which is the user's adoption decision, not a
default.

**Probe** (`probe_book_supply_sealed_events`, `#[ignore]`d; log `W:\temp\claude\caprise\probe.log`),
the sealed chamber under the book's supply:

* Waterings on days 125, 191, 219, 246, 276, **339** — all in the first year. The last falls as the crop
  dies (LAI 0.17 on day 330, 0.08 on day 360); it re-wets the top (14.2 kg on day 360), so the dead
  phase **opens with ~50 days of Stage I** — part of the 0.605 mean.
* After that, **no watering in the dead years**: the dry clock runs unbroken from ~day 410 to the end
  (502.9 days), FTSW drifts 0.88 → 0.63, never near the 0.45 trigger. (c): the drain does NOT reach the
  trigger in this run.
* The air in the dead years **swings with the time of year** rather than settling: 0.23 kg (day 540),
  **0.010 kg (day 720)**, back to 0.12 kg (day 900). The end vapour 0.125 kg is on the upswing, not a
  steady tail — P1's "above the range" is a seasonal reading. Which forcing drives the swing (light,
  temperature, the target) was not separated.

**So:** the dead years' trend IS the book's Stage II decline (no watering intervenes); the mean 0.605 is
lifted by the death-time watering's Stage I start; and the air still falls to ~4 % of its target at the
season's low point. The success test fails on the Stage II tail.

## 13. The price under the book's supply — both caps side by side (2026-10-08)

**Asked (the user, "go with your recommendation"):** re-price §11d with the soil's evaporation drawing on
the whole root zone (§12's `SoilSupply::RootZone`, the book's program) instead of our top-layer cap. **This
is evidence for the choice, not the choice:** which supply an adoption carries stays the user's call (§12c).

**Method.** `wip/three-forms-price` rebased onto `1a8c86e` (clean), its loader line given a `supply`.
Control first: with `supply: TopLayer` a throwaway `regen_goldens --write` reproduced all 21 files of §11d's
`W:\temp\claude\threeforms\branch_goldens\` **byte for byte** — the rebase added nothing. Then `RootZone`:
`regen_goldens --write` (throwaway, restored), the full suite `--no-fail-fast` (debug), the ignored tests in
release. Branch commit `3cfa280` (local, never pushed). Logs and both golden sets:
`W:\temp\claude\booksupply\` (`regen_rootzone.log`, `diff_cap_vs_book.txt`, `diff_main_vs_book.txt`,
`suite.log`, `ignored.log`, `fail_book.txt`; predictions written before the run in `predictions.md`).

### 13a. Graded

| | predicted | measured | grade |
|---|---|---|---|
| B1 | the same 10 goldens change against main | the same 10; the other 10 identical | HELD |
| B2 | against the cap's price all 10 move again, water only | all 10 move; only soil water, condensate, vapour, the soil aux keys and `boundary.soil_evaporation` | HELD |
| B3 | `sealed_chamber` (915 d, the §12c probe's horizon): air vapour ≈ 0.125 kg | **0.124996 kg** (cap 1.95e-6; frozen 0.2505) | HELD |
| B4 | biosphere: no carbon / N / O₂ / root depth / development value moves against main | none moves | HELD |
| B5 | station goldens: carbon UNCERTAIN (a tank-capped watering could stress the crop) | **no carbon value moves** in any of the four; the lowest tank level over the run was not traced (end states only) | carbon: unchanged |
| B6 | soil water lower than the cap's price in every moved golden; sealed condensate/vapour higher | lower in 8 of 10; **higher** in `consumer_chamber` (+0.9 %) and `sealed_station` (+5.5 %, condensate −13 %, and its below-root store **+19 %**: 19.7 → 23.5 kg, ~1 % above main where the cap left it 15 % below) — **not explained**; end states only, the watering timing not traced | PARTLY |
| B7 | the same 24 + 2 reds by name, same classes, no new red | **identical lists** (`comm` empty both ways); the ignored run 7 passed (the §12c probe now among them), 2 failed — the same two; the three unexplained reds fail with the **same messages** (WSFD bound; the knockout's `[]`; `soil_shade_lai` inf); the re-sow cycle test's below-root store reads 144.7, 154.1, 101.3, 123.9 (cap: 125.7, 126.2, 134.6, 95.9); the grain rescue still 1.2199 | HELD |

### 13b. Both prices, end states against main

| golden | top-layer cap (§11d) | book's root-zone supply |
|---|---|---|
| `season_euler` | soil −41.7 %; soil evaporation 33.6 kg | soil −48.9 %; soil evaporation **45.9 kg** (+36 %) |
| `sealed_chamber` | air vapour **2e-6** kg; condensate 52.3 | air vapour **0.125** kg (half of frozen 0.25); condensate 62.5 |
| `perennial_chamber` / long | condensate 76.1 / 69.6 | condensate 88.3 / 74.1 |
| `consumer_chamber` / long | condensate 60.9 / 48.8 | condensate 59.9 / 68.6 |
| `greenhouse` / `harvest` (7 d) | air vapour 0.70 / 0.51 (frozen 1.90) | air vapour **1.21 / 1.08** |
| `lighting` (7 d) | condensate 0.88 (frozen 3.96) | condensate 4.37 |
| `sealed_station` | condensate 71.6; soil 100.4 | condensate 62.3; soil 105.9 |

Water only in both; every carbon, nitrogen, oxygen and energy value identical to main in both.

**What it says for the choice.** The two caps cost the same ceremony: the same ten goldens, the same 24 + 2
reds in the same classes, the same three unexplained reds. The book's supply moves water further from the
frozen model in the open field and the long chambers, but keeps the never-re-sown sealed chamber's air at half the frozen model's where the cap leaves essentially none
(the success test is defined against frozen, so that comparison is fair). The 7-day station chambers' air is
also nearer frozen, but part of their drop is the leaf-area resistance working on seedlings — not a merit
either way. It does not make
the never-re-sown chamber's success test pass (§12b–c). Nothing in the run favours the cap; the cap is our
build-time departure (§12), the root-zone supply is the cited program.

**Still owed whichever is chosen:** the §11d design item (the plain re-sow must reset the soil values from the
one frozen param load), the reds' restatements, the ceremony list after §11d, and the unmeasured cross-port
bands (a Linux CI push). The book's `DYSE = 1` start (§12) stays its own item, not folded in.

## 14. DECIDED: the whole root zone supplies the soil's evaporation (the user, 2026-10-09)

**The user:** *"Strive for what is closer to reality."* Asked which supply an adoption carries (§13), this
picks **`SoilSupply::RootZone`**, the book's program.

**Why it is the closer one, physically (not only "the book's"):** once a real soil's surface dries,
evaporation does not stop. It slows, because water keeps moving up from the moist soil under the dry
crust, and Stage II's √-time decline is the book's description of exactly that supply from below. The
top-layer cap stops evaporation dead the moment a 150 mm *bookkeeping* account reaches zero while the
soil beneath it is still ~63 % full (§12c's probe). No physical process does that. Here the cited program
and physical realism point the same way.

**What this decision is NOT:** it is not the go-ahead to adopt. Regenerating goldens and manifests,
restating the 26 reds, the param-file ceremony and the Linux CI push for the cross-port bands all
wait for the user's explicit go (advisor, 2026-10-09). "Closer to reality" is a rule for choosing between
forms. It does not waive the unfreeze ceremony.

**Not decided by this principle:**

* **The book's `DYSE = 1` start (§12).** Neither start is shown to be more realistic. It is one
  convention against another, so it stays its own item.
* **A realism lead found while recording this, not acted on:** the sealed chamber's condenser keeps
  drawing vapour out *below* its humidity setting (`science::condensed_vapour_kg`:
  `(v − target)⁺ + rate · min(v, target) · dt`). `water_cycle.yaml` already records that *"a real
  dehumidifier would not; recorded, not changed."* That draw, not the soil, may be what pulls the
  never-re-sown chamber's dead-phase air down to ~4 % of its target (§12c) while the soil below is
  still moist. If so, the success test S4 that "still fails" is partly a condenser artefact, and
  reading it as a mark against either supply would be wrong. The condenser is frozen science (every
  sealed golden), so this would be its own unfreeze item, the user's call. It is not folded into this
  adoption.

**Owed next, in order:**

1. Trace B6 (§13a): the sealed station's below-root store ends 19 % wetter under this supply. This is
   read-only: when the waterings fell, where the drained water landed. Do it before any golden moves.
2. The old cap's fate (ceremony item 2): keep `TopLayer` as a lab switch (the four instrument tests
   pin it) or retire it.
3. Then the adoption list of §11d/§13, on the user's go.

⚠ **§14's condenser lead, CONFIRMED and FIXED 2026-10-09** (`post-roadmap-chamber-dehumidifier.md`, a biosphere
unfreeze on the user's *"Fix the dehumidifier. Then step 1"*). The condenser now removes only the excess over
its setting. In this plan's measurement the never-re-sown chamber's dead-crop air went **0.605 → 0.981** under
the chosen root-zone supply (S4 passes) and **0.176 → 0.704** under the cap (still fails). **So §12b's sealed
row, §12c's air readings and §13's sealed numbers (`sealed_chamber` 0.125 kg, `sealed_station`) are STALE:
they were measured with the old condenser.** Their soil, water-out and event columns are unaffected (the
measurement re-ran them to the digit).

## 15. B6 explained: the sealed station's below-root store is not wetter, it is read at a different moment (2026-10-09)

**Method.** `wip/three-forms-price` rebased onto the dehumidifier fix (`f1ba3a3`). Control: `git range-diff`
shows the branch's three commits **byte-identical** (`=`) before and after. §13's golden control could not be
repeated, because main moved. A probe (`station/examples/probe_b6.rs`, untracked, kept with the logs) ran
`sealed_station` on the rebased branch under each supply (the loader's `supply` line flipped for the cap run
and restored) and on main. It printed every day's water stores, a per-season summary, the run means, and each
watering against the condensate tank. Logs: `W:\temp\claude\dehumidifier\` (`b6_book.log`, `b6_cap.log`,
`b6_main.log`).

**First, re-measured after the fix** (the condenser fills the tank that caps the waterings, so it could have
moved B6). End-state below-root store: root zone **24.39 kg**, cap **20.75 kg**, main **23.24 kg**. The gap is
+17.5 % (was +19 %), so the fix did not remove it.

| | cap | root zone | main (frozen forms) |
|---|---|---|---|
| below-root store, **mean over the run** | **78.70 kg** | **78.80 kg** | 97.91 kg |
| root zone, mean | 63.80 | 64.30 | 90.76 |
| condensate, mean | 50.12 | 49.51 | 3.94 |
| waterings / tank-capped | 22 / 13 | 23 / 12 | 0 / 0 |
| below-root store at each 305-day chunk's low (kg) | 25.88, 21.12, 24.97, 18.70 | 24.88, 19.68, 23.69, 24.39 | 23.15–23.24 |

**What it says.** Over the run the two supplies keep the same deep store, to 0.1 %. The store swings from
~20 to ~160 kg within every season: re-sowing and the waterings' losses fill it, and the descending roots
draw it down. The run ends ~80 days into a new crop, with the roots at 1.27 m of 1.30. So the end value is
the trough, and the trough's depth depends on when that season's last waterings fell (cap days 1181 and 1204;
root zone 1196). Within ONE supply, the troughs differ by as much as the two supplies do at the end (cap
18.70–25.88). **B6 is a snapshot of a cycling store, not a wetter one.** The tank capped about half the
waterings under both supplies (the "wanted" column is approximate: the deficit to full at the day-before
depth, over 0.90), so a capping difference does not explain it either. Against main the package does lower
the deep store's mean by ~20 %: the accepted residual drying of §10f–h, and not new.

**Grade of §13's B6 row:** PARTLY → explained. Nothing in the price changes.

## 16. ADOPTION — the user's go (2026-10-10)

**The user:** *"Step 2"*, the adoption of §11d/§13 under §14's root-zone supply, which waited on an explicit go.
The package is the five loader settings of §11 (`rs_form = SzeiczLong`, `soil_evap = TwoStage { floor: false,
supply: RootZone }`, `watering = Fao56Trigger`, `deep_credit = ActualWetness`, `deep_overflow = Recycled`) with
their application efficiencies and same-day percolation. Potato adopts with wheat. A biosphere **and** station
unfreeze.

**Advisor review before any code (2026-10-10), summarized:** (1) re-measure the reds first: the 24 + 2 list and
§13's sealed numbers predate the dehumidifier fix, and §15 re-ran only the probe; (2) the three unexplained reds
BLOCK the flip: each gets a read-only probe and a recorded cause before any test is restated; (3) the re-sow
route threads the params the run was built from. A reset that loads the frozen params itself is the escape
`param_funnel` exists to catch (an override built through `build_season_with` would be re-sown with frozen
values), and `params::biosphere()` is wheat; (4) build in two commits, inert plumbing first; (5) the cross-port
bands are measured only on Linux CI. The old cap's fate is not a blocker: the deep-store tests already need the
old forms as switches.

### 16a. The reds re-measured after the dehumidifier fix

`wip/three-forms-price` sits on `f1ba3a3`, and `git diff f1ba3a3 main -- rust/` is empty, so the branch is main
plus the three pricing commits. Full suite `--no-fail-fast` (debug): **1336 passed, 25 failed, 9 ignored**. The
25 are §13's 24 **by name** (`comm` against `sources/temp-evidence/booksupply/fail_book.txt`: nothing dropped,
nothing new from the package) plus one unrelated red,
`repo_gates::every_memory_index_line_names_a_file_and_vice_versa` (below). Ignored, release: **7 passed, 2
failed**, the sealed station's golden and band, as §13a. **§13's red list stands.** Logs:
`W:\temp\claude\water-adoption\` (`suite.log`, `ignored.log`, `fail_now.txt`).

⚠ **The unrelated red, not this batch's:** the memory-index gate reads one link per index line, and only lines
that start `- [`. The user's memory index was compressed on 2026-10-08 so that finished series share a line, so
39 indexed notes read as unindexed. It runs only where the profile exists (never on CI). Either the lines split
again or the scanner reads every link on a line: the user's call, recorded and left.

### 16b. The three unexplained reds: each has a cause

Probe: `domains/tests/zz_adoption_probe.rs` (untracked, run on the branch and on main, then deleted). Logs
`probe_branch.log`, `probe_knockout.log`, `probe_drought_main.log` in the same folder.

1. **`drought_acceleration…`, "WSFD exceeded its 1 + WSSD bound": the bound was never a season law.** The
   manufactured dry chamber's season thermal time, wssd 0.40 against off: **2842.27 / 2028.37 = 1.40126** on the
   branch, **1.14781** on main. Of the steps whose accelerated increment exceeds 1.4× the off run's, **136 of 136**
   are steps where the accelerated crop is past anthesis and the off crop is still vegetative. Past anthesis the
   accumulator drops the vernalization and photoperiod factors (both < 1, `ThermalTimeAccumulation`), so a crop
   that flowers first gains more than `1 + wssd` per step against one that has not. Per step, in the same phase,
   the factor never exceeds `1 + wssd`. Why it shows now: on main the chamber is fed continuously by recycling
   (FTSW 0.04–0.29 all season, 72 watering steps), so drought acceleration bites mildly and the crop flowers 7
   days early. Under events the tank is empty, there is one watering, FTSW sits at 0–0.05, the factor runs at its
   ceiling, and the crop flowers **20 days** early (day 230.5 against 250.25). **No bug.** Restatement: the bound
   is asserted per step, over the steps where both runs are in the same phase, which is what Eqn 15.8 states.
2. **The knockout: dropping root capture no longer kills the perennial runs, because the water has a second
   route.** Perennial chamber, five years, capture dropped: under events and under continuous watering alike the
   crop completes with **the same grain as the intact run** (peak 0.7127 → 0.5853 kg C by season, identical to the
   digit). With **both** `biosphere.root_zone_capture` and `biosphere.subsoil_overflow` dropped it dies at the
   third re-sow under either watering (`seed bank too small`); with the old `Held` overflow and the book's credit,
   dropping capture alone kills it, as before. The mechanism: as the roots deepen, the store below them is spread
   over less soil, so its water rises above its own capacity; `Recycled` drains that excess to the tank and the
   watering returns it to the root zone. The deep water reaches the crop with or without capture. **No bug: the
   knockout no longer isolates "the roots reach the deep water".** Restatement: the knockout drops both flows.
3. **The non-finite check: an infinite param now fails at the build, earlier and louder.** `carbon_fraction = 0`
   makes `sla_per_mol_c` infinite; the soil account's starting `soil_shade_lai` is the seedling's leaf area, so
   the infinity is now in a STORED value and `State::new` refuses it (`State.aux["soil_shade_lai"] is not finite:
   inf`), returned as a setup error before the report's own guard runs. Nothing is printed and nothing is hidden.
   Restatement: the test asserts the refusal, and the report's fold guard keeps a witness that still reaches it
   (a param that is infinite only in a fold).

### 16c. The old forms stay as lab switches

The constant 70 s/m, `SoilEvaporationForm::Off` and the `TopLayer` cap, `Continuous` watering, the book's
full-capacity credit and the `Held` overflow all stay, as lab switches. The six deep-store tests of §11d pin the
book's own deep store through them, and the four instrument tests pin `TopLayer`; retiring them would cost those
comparisons. This answers §14's step 2.

### 16d. The design: two commits

**Commit A: plumbing; no golden moves.**

* **The re-sow route.** Once the loader carries the soil account every reference state stores it, so the plain
  `annual_reset` (which refuses such a state) would refuse every reference run. It is retired; the re-sow takes
  the params the run was built from. `run_perennial` takes `&BiosphereParams`, and every production caller
  (`run_perennial_final`, the golden producer, `ulp_probe`, `emit_trajectory`, the station's sealed re-sow hook)
  loads the frozen params **once** and hands the same object to the build and to the re-sow. `param_funnel`'s
  blessed site moves with it if the single load has to sit in a named helper.
* **The constants into param files, with their sources** (the reference may not hard-code coefficients):
  Szeicz–Long's leaf resistance 100 s/m and Teh's threshold LAI 4.0 into `transpiration.yaml`; the soil's albedo
  0.12 and extinction 0.5, the top layer's 0.15 m, the 1 mm wet threshold, the Stage I FTSW 0.5 and silt loam's
  SAT 0.433 / DUL 0.218 into a new `soil_evaporation.yaml`; FAO-56's 0.55 and the two application efficiencies
  0.75 / 0.90 into a new `watering.yaml`. The station's separate-air option reads the same 0.55 from the domains
  param instead of its own copy (`air_split::FAO56_WHEAT_DEPLETION_FRACTION`). The floor's 1.5 mm/day and FAO's
  full-cover 0.5 stay code: their forms stay lab-only.
* **Gate:** `regen_goldens` reports every golden identical; the manifests change only in `param_files`; the full
  suite and the ignored tests pass (bar the memory-index red of §16a).

**Commit B: the flip.** The five loader lines and `supply: RootZone`; the goldens and both manifests regenerated;
`rust/data/tiers.json`'s `_transcendental_sites` and each moved golden's `transcendentals` list given the soil
path's `sqrt` (Stage II) and `exp` (the soil's shade); each red restated by its §11d class, with §16b's causes for
the three; dated entries in the biosphere and station contract docs; every "LAB-ONLY / the loader's value"
comment on the adopted forms flipped. Built fresh on main, not cherry-picked from the pricing commits. Then a
push, and the cross-port bands read on Linux CI.

**Not in this batch:** the book's `DYSE = 1` start (§12, its own item); the separate-air option's watering not
feeding the top-layer account (§11d ceremony 4, a lab gap); the memory-index gate (§16a).

### 16e. Commit A, built (2026-10-10)

**One change of plan from §16d:** the coefficients went into the two existing files, not two new ones.
The soil's nine go into `transpiration.yaml`, beside Szeicz–Long's two: the soil–crop energy split is
computed inside the transpiration flow, so it is one process. The watering three go into `water_cycle.yaml`,
whose header now says the sprinkler efficiency is the open field's. This keeps the census at 15 files (a count
pinned in several places) and changes no file list.

**What landed.**
* `annual_reset` and `run_perennial` (no params) are retired. `annual_reset_with` and `run_perennial_with`
  are the only routes. `run_perennial_final` takes `&BiosphereParams`, so the spine still has one production
  param load. The golden producer, `ulp_probe` and `emit_trajectory` load the frozen params once per run and
  hand the same object to the build and to the re-sow. Three tests that pinned the retired reset's refusal
  were restated to their surviving claim: the re-sow resets a stored leaf area, adds none to a state that
  stores none, and resets the soil values.
* **The station's sealed re-sow hook is NOT threaded, on purpose.** The station builds its season through
  `build_season` (frozen wheat) and its hook loads the same frozen wheat params. It has no override seam, so
  there is nothing to escape, and threading would change the signatures of the hook's six callers (palette,
  `godot_bridge`, the lamp shed, three examples) for no gain. Recorded, not done.
* Twelve constants are now params, loaded with bounds: `leaf_stomatal_resistance`, `threshold_lai`,
  `soil_albedo`, `soil_shade_extinction`, `top_layer_depth`, `top_layer_wet`, `stage_one_ftsw`,
  `top_layer_saturation`, `top_layer_drained_upper_limit` (`transpiration.yaml`); `depletion_fraction`,
  `sprinkler_efficiency`, `drip_efficiency` (`water_cycle.yaml`). The science functions take them as
  arguments. The station's separate-air option reads `depletion_fraction` instead of its own copy.
  A new test pins each value against the constant it replaced (bit for bit) and reaches each new bound.
  Control: with the `top_layer_wet` bound deleted, it goes red.

**Gate, measured.** `regen_goldens`: 20 of 20 identical, after the re-sow slice and again after the
coefficients. The biosphere manifest's only change: the digests of `transpiration.yaml` and
`water_cycle.yaml`. Full suite: 1360 passed, 1 failed (the memory-index red of §16a), 9 ignored. Clippy clean. Ignored, release:
9 passed, 0 failed (the sealed station's golden, band and both session resumes among them).
Logs: `W:\temp\claude\water-adoption\` (`regenA1.log`, `regenA.log`, `suiteA2.log`, `clippyA2.log`,
`mutant.log`, `ignoredA.log`).

⚠ **A slip, caught by the tests and fixed:** the edit scripts wrote Windows line endings, and a first suite
run had six loader tests red because their text-substitution helpers search for `\n`. Every touched file
was converted back to LF before the run above.

### 16f. Commit B, predictions (written before the loader lines change)

Built on main over commit A, so the re-sow needs no pricing hack. The basis: §13 (the book's supply) and
§16a's re-measure; the sealed rows of §13 are stale (the dehumidifier fix), so their numbers are re-predicted
from §14's note.

| | prediction |
|---|---|
| F1 | `regen_goldens` report: the same 10 goldens change as §13 (`season_euler`, `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`, `greenhouse`, `harvest`, `lighting`, `sealed_station`); the other 10 identical, `drift_summary` and `sealed_energy_drift_summary` among them (leaf carbon and energy do not move) |
| F2 | water only in all 10: no carbon, nitrogen, oxygen, energy, `rooted_depth`, `thermal_time` or `vernalization_days` value moves; the biosphere goldens gain the five soil aux keys; `season_euler` gains `boundary.soil_evaporation` |
| F3 | the six biosphere goldens equal §13's root-zone set byte for byte **except** the three sealed-family runs the dehumidifier fix touched (`sealed_chamber`, `perennial_*`, `consumer_*`), whose water stocks differ; `season_euler` (open field, no condenser) **equals** §13's byte for byte |
| F4 | `sealed_chamber`: end air vapour within 10 % of the frozen 0.2505 kg (§14's note: the dead-crop air ratio 0.981 under the fixed condenser), not §13's stale 0.125 |
| F5 | debug suite reds before any restatement: §16a's 24 minus the four commit A already discharged (`param_funnel`, the two refusal tests' successors, `the_params_aware_resow…` retired) and minus the two re-sow tests that called the plain reset (`the_resow_returns_the_abandoned_fraction…`, `the_resow_splits_the_parents_nitrogen…`), which now re-sow with params: **18**, plus the unrelated memory-index red. The ignored run: the sealed station's golden and band red |
| F6 | after `--write` and both manifests regenerated: the golden, band and manifest reds clear; the station manifest moves its `golden_sha256` rows and its flow/aux sets as §11d listed |
| F7 | the three §16b reds fail with §16b's messages, and their restatements pass |

Not predicted: the cross-port bands on Linux (measured only by CI after the push).

### 16g. Commit B, built and graded (2026-10-10)

| | predicted | measured | grade |
|---|---|---|---|
| F1 | the same 10 goldens change; the other 10 identical | exactly those 10 (`regen_goldens` report); `drift_summary`, `sealed_energy_drift_summary` and the eight crop-free goldens identical | HELD |
| F2 | water only; the five soil aux keys; `boundary.soil_evaporation` in `season_euler` | no non-water stock and no existing aux key moved in any of the 10; the five keys in all 10; the new stock in `season_euler` only (45.87 kg) | HELD |
| F3 | biosphere goldens equal §13's root-zone set except the sealed family; `season_euler` equal | `season_euler` byte-identical to §13's; the five sealed-family files differ (the dehumidifier fix) | HELD |
| F4 | `sealed_chamber` air vapour within 10 % of 0.2505 kg | **0.250476 kg**, unchanged from before the flip | HELD |
| F5 | 18 debug reds + the memory red; ignored: the sealed station's golden and band | **20** flip reds + the memory red + one census red; the ignored tests were not run before regenerating | MISCOUNTED — see below |
| F6 | after `--write` and both manifests: golden, band and manifest reds clear; station manifest moves its `golden_sha256` rows | cleared; the biosphere manifest: 6 `golden_sha256` rows, `flow_set` −`Irrigation` −`Recycling` +`EventIrrigation` +`EventRecycling` +`SubsoilOverflow` (24), `aux_set` +`SoilSurfaceAccount` (4); the station manifest: its 4 `golden_sha256` rows only | HELD |
| F7 | the three §16b reds fail with §16b's messages; their restatements pass | the same messages; restated, all three pass | HELD |

**F5, read honestly.** The arithmetic was wrong, not the model: §16a's 24 minus the four commit A discharged
(`param_funnel`, `the_params_aware_resow…` retired, the two re-sow tests that had called the plain reset) is
**20**, and I also subtracted the two refusal tests' successors, which were never on the list. The census red
was **commit A's**, not the flip's: the new loader test was added after commit A's suite run and carries no
census row, so `20fe5b7` went to `main` with `every_test_in_an_s5_surface_is_claimed_or_declared` red. Fixed on
its own (`0f025e9`, the A-row count 82 → 83) before the flip was committed. The lesson: re-run the suite after
the last edit, not before it.

**The 20 reds, restated by class** (§11d's table, §16b's causes):

* **Goldens and bands (4 + 2 ignored), manifests (2):** regenerated.
* **The switch identities (4):** `canopy_resistance` P1, `soil_evaporation` S1 → *the loader's forms through the
  switches are the reference run*; `canopy_resistance` P2 and the scorecard's R1 → run with the soil's
  evaporation off (and R1 the constant resistance), because their by-hand formulas have no energy split and
  their subject is the resistance's reach. The measurement's "frozen" row is now `pre_adoption()`, all five old
  forms, so it is still the model the forms were priced against.
* **The book's own deep store (7):** the dry store, the rescue and its two controls, the actual-wetness credit,
  the overflow's two sides, the clamp, the re-sow fixed point — each pins the five old forms through
  `pre_adoption_params()`. Under the reference none of the conditions can be built: event watering wets the
  store below the roots with every watering's loss and refills the whole deficit (so `irrigation_mm_day` is only
  an on/off switch), and the re-sown chamber has no fixed point to converge to.
* **The ring (1):** recycling is evaluated at a third state below the trigger, and its sinks include the
  store below the roots (the drip's loss).
* **§16b's three:** the drought bound per step in the same phase (mutation control: scaling the factor ×1.2
  reddens it at step 928); the knockout drops both routes; the infinite param asserts the build's refusal and
  keeps the fold guard's witness under `Off`.
* **One source-scan guard** (`the_knockout_helper_goes_through_the_lab_seam`) followed the helper's rename to
  `trace_without_flow_with`.

**Gate.** Debug suite after the last edit: 1361 passed, 1 failed (the memory-index red), 9 ignored. Ignored,
release: 9 passed, 0 failed (the sealed station's golden and band, both session resumes). Clippy clean. Logs: `W:\temp\claude\water-adoption\` (`regenB.log`, `regenB_write.log`, `golden_diff.txt`,
`suiteB0.log`, `suiteB1.log`, `suiteB2.log`, `ignoredB.log`, `clippyB.log`, `mutant2.log`).

**Owed:** the cross-port bands on Linux CI for the ten moved goldens (the push after this commit).

### 16h. After the push: Linux CI, the user's decision, and what the adoption did to the real-world check (2026-10-10)

**Linux CI on `448671c` (run 38042136840): one red, and not a band.** Every golden and band test in the
default `cargo test` passed on Linux; the steps after it (the ignored sealed-station golden and band, and
clippy) were skipped by the red, so they are read on the next push. The one red was
`tier_sensitivity::the_biosphere_band_sits_above_the_measured_sensitivity`. That test nudges the PAR
seam by ±1 ULP, measures how far the 15-year perennial run moves, and checks the reading three ways:
non-zero, within 10× of the Python instrument's 3.520e-15, and below the 1e-11 band.

| reading | Windows | Linux |
|---|---|---|
| before the switch (`0f025e9`) | 2.291e-15 (worst leaf `stocks[10]`) | passed (value not printed) |
| after the switch (`448671c`) | 3.965e-15 (worst leaf `biosphere.stem_c`) | **3.569e-14** (same leaf) — 1.4 % past the 3.520e-14 edge |

So the switch moved the carbon's last-digit sensitivity on both platforms. No golden's carbon moved,
but a perturbed run's carbon now reads the water forms somewhere (a path not traced). Why Linux reads 9×
Windows was **not** measured: no Linux toolchain is installed locally, and CI runs only on a push to
main or a pull request.

**Advisor (2026-10-10), summarized:** tell the user before going further; don't move the reference
figure or widen the window; this check justifies the band, so how to re-anchor it is the user's call;
measure Windows before and after, then Linux both ways, before proposing a fix.

**DECIDED (the user), asked how to get the Linux numbers:** *"I don't care about Linux, drop its tests
if needed."* Done narrowly: the test still runs on Linux (claims 1 and 3), and only claim 2, the reach
check against the Python figure, runs on Windows alone and says so on Linux. Not compiled out
(`#[cfg]`): a gate that vanishes is the shape `goldens.rs` warns against. ⚠ **The gap this leaves:** on
Linux the biosphere band's basis is now "non-zero and below 1e-11", not "the same order as the
instrument it re-measures". The Python figure describes a model before these forms, so on any platform
the window's premise is gone; a re-anchoring is owed if this check is to mean "the same dynamics" again.

**The real-world check, read after the switch** (`station/tests/scorecard_tm102788.rs`, TM 102788 at
812 ppm): water use over TM days 25–80 **2.38 → 2.60 L m⁻² d⁻¹, 0.40 → 0.43 of the trial's ~6.0**;
mean water use 0.61 of the trial's; peak daytime uptake unchanged at 0.979. The leaf-area resistance
lets the dense canopy transpire a little more (rs 70 → 50 s/m at full cover); the gap is mostly still
there. (The scorecard's row is a nutrient-solution crop held above the stress threshold, so the soil's
evaporation and the event watering hardly touch it.)

**The separate-air gap is now live** (§11d ceremony 4). The station's separate-air option waters from
the crew's supply with its own `TriggeredWatering`, which the soil's top-layer account does not see: its
inflow is the biosphere's own watering. While soil evaporation was lab-only this was latent; now every
separate-air run with its own watering on has a top layer refilled only by the biosphere's event
watering, so its Stage I restarts less often than the water delivered would warrant. Measured once:
`cargo run --release -p station --example watering` completes (the 22 °C chamber watered 11.0 kg from
the crew's store over a season; the weather chamber 5.0 kg). ⚠ The separate-air numbers already in the
records (`post-roadmap-room-temperature.md` §13–§18) are
**pre-adoption**. The fix (feed the account from whatever waters the root zone) is offered as a follow-on.

**The pricing branch** `wip/three-forms-price` (three local commits, never pushed) is superseded by
`20fe5b7` and `448671c`; deleting it is the user's call.
