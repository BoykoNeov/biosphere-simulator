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
