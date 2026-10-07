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
