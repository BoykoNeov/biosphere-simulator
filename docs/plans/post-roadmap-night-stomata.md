# Stomata that close in the dark — a lab-only form first

**Opened 2026-10-15** on the user's choice *"Night stomata closing"*, the first of the three follow-ons
the latent-heat estimate left open (`post-roadmap-chamber-latent-heat.md` §5b). **This doc builds a
lab-only form and measures it. It is not the adoption** — adopting it is a biosphere unfreeze and a
separate call for the user, on the numbers in §6.

## 0. Why

The model's crop transpires in the dark at 92 % of its lit rate at the chamber's 7.5 s/m air resistance
(66 % at 50; `log/chamber-latent-heat.md`). The surface resistance is Szeicz–Long (Teh Eq. 4.80) on
FAO-56's constant leaf value `rl = 100 s/m` — **a composite**: Teh's own leaf resistance is the
light-dependent Jarvis form (Eq. 4.79), for which the shelf has no wheat coefficients, so the stand-in
carried no light term and the stomata never close. `transpiration.yaml` says so in its own source text.

## 1. Sources

### 1a. On the shelf

* **Teh (2006) Eq. 4.79, p. 97** (page read as an image), after Jarvis (1976):
  `r_st = (a₁ + I_PAR) / (a₂ · I_PAR)`. As `I_PAR → 0` it diverges: **complete closure in the dark, no
  floor.** Coefficients are given for sunflower (11.2550, 0.0051) and maize (2.8927, 0.0040) only.
  Unusable as the night value: zero night transpiration contradicts every wheat measurement below.
* **Monje & Bugbee (2019) p. 16** (§3.6–3.7, the source of the 7.5): *"In the dark, … canopy
  transpiration was small"* — in their wheat chamber, no number given. Supports the direction only.
* Penning de Vries et al., Wheeler TMs, BVAD, Soltani & Sinclair: nothing on night conductance
  (searched: nocturnal, night-time stomata/conductance/transpiration, dark-period water).

### 1b. Fetched 2026-10-15

* **[N1] McAusland, L., Acevedo-Siaca, L.G., Pinto, R.S., Pinto, F., Molero, G., Garatuza-Payan, J.,
  Reynolds, M.P., Murchie, E.H. & Yepez, E.A. (2023)**, "Night-time warming in the field reduces
  nocturnal stomatal conductance and grain yield but does not alter daytime physiological responses",
  *New Phytologist* 239(5):1622–1636, doi 10.1111/nph.19075, **CC BY 4.0**. Full text saved as
  `sources/mcausland_2023_nph19075_PMC10952344.xml` (Europe PMC). Twelve irrigated spring-wheat
  genotypes, field, Ciudad Obregón (Mexico), three seasons; flag leaves, porometer and LI-6400.
  * **Table 3** (verbatim): *"Mean nocturnal conductance (g_sn) as a percentage of g_s at booting or
    heating under control or nocturnally heated plots"* — Booting / Control **12.30 ± 4.54**; Heat
    9.38 ± 4.62; Heading / Control **19.40 ± 9.01**; Heat 17.2 ± 7.65.
  * **Table 3 is the gas-exchange data** (Methods, verbatim): *"Measurements of A, g_s, R_d and g_sn
    were taken using infrared gas analysers (LI‐6400‐XT; Li‐Cor)"*, day at 1800 µmol m⁻² s⁻¹ PPFD,
    26 °C leaf, 1.2 kPa VPD; *"At night, leaf temperature was 16°C (±1°C), while leaf VPD and C_a were
    kept the same as during the day"*; *"across all 3 yr"*, n = 6. **VPD held equal day and night**,
    so the ratio is stomatal alone — what Penman–Monteith needs, since it carries VPD and temperature
    itself. The reason Table 3 is preferred over Table 2.
  * Table 2 (porometer, two seasons, ambient leaf VPD): control 16.8 % (booting), 32.7 % (heading).
  * **When in the night:** *"30 min after sunset (c. 19:00 h) for a period of 1–2 h before dewfall"*
    — early night only. The paper notes g_sn *"exists throughout the night and predawn"*; how it
    moves through a night is not in this source.
  * Introduction: *"Nocturnal conductance typically accounts for 5–18% of all daytime water loss in
    wheat (Caird et al., 2007; McAusland et al., 2021)"*.
* **[N2] McAusland et al. (2021)**, *New Phytologist* 232:162–175, doi 10.1111/nph.17563 — **abstract
  only** (full text 403 from every route tried): eight wheat genotypes, well watered, *"lifetime g_sn
  achieved rates of 5.7–18.9% of g_s"*, declining with age for most genotypes.
* **[N3] ASCE-EWRI (2005) standardized reference ET, hourly form** — read through Snyder, R.L.,
  "Equations for Estimating ETos and ETrs with Hourly Weather Data" (UC Davis, 2002, after ASCE-EWRI
  2004), not the standard itself: short grass `rs = 50 s/m` *"during daylight (when Rn > 0)"*,
  `200 s/m` *"during the night (when Rn ≤ 0)"* (night conductance 25 % of day); tall alfalfa 30 / 200
  (15 %). **LOCUS: reference surfaces, not wheat** — used here for the day/night RULE, not the value.

**Locus of the value.** [N1] is leaf-level, field wheat at night temperatures 8–12 °C; the model's
chamber is a held 22 °C canopy at 75 % RH. A canopy ratio is taken equal to the leaf ratio — exact
under Szeicz–Long, where `rc = rst / L` is linear in leaf conductance. Night warming LOWERED the ratio
in [N1]; a warm chamber night is outside its range, which is stated, not corrected for.

**The basis is the RATIO, not [N1]'s absolute night conductance** (advisor, 2026-10-15). The model's
day leaf resistance is FAO's class value 100 s/m; dividing by f gives night leaf resistances of 813 s/m
(f 0.123) and 515 s/m (f 0.194). [N1]'s own porometer means (Table 2, control: g_sn 0.048 and 0.063
mol m⁻² s⁻¹) are ~870 and ~660 s/m at ~41.6 mol m⁻³ air. Booting agrees; at heading the ratio route
opens the night stomata ~25 % more than the paper's absolute value. The ratio is kept because it
carries the model's own day leaf onto the night and does not graft a field leaf's magnitude onto
FAO's class value; the heading gap is stated as the cost.

## 2. The form

`NightStomata { Open, Closing(f) }` on the transpiration flow's canopy read; the loader sets `Open`
(today's behaviour, bit-identical).

* **`Closing(f)`**: on a step whose net radiation is `≤ 0`, the canopy surface resistance is the day
  value divided by `f`, the night-to-day conductance ratio. Otherwise unchanged. **The rule is
  ASCE's** ([N3], `Rn > 0` is day), applied to the model's own day resistance.
* **Partly lit steps count as day** — ASCE's rule taken literally. At the 1/16 step a 16-h photoperiod
  centred on midday (lights 04:00–20:00) leaves 1.5 of its 8 dark hours inside partly lit windows, so
  about a fifth of the dark time keeps open stomata; the 20-h TM row, 1 h of its 4. Stated as a bias
  toward MORE night water. A lit-fraction weighting is the alternative; it needs a lit-fraction signal
  the flow does not have, and is not built.
* **Where:** inside `flows.rs` `CanopyRead`, which both transpiration paths call. The shared
  `science::canopy_surface_resistance` is **unchanged**: its four other readers (the TM scorecard's
  diagnostics, `chamber_dryness`, `soil_evaporation` and `canopy_resistance` tests) ask for the
  daytime value and keep getting it. Each is re-read at build time; any that integrates water over
  nights is a finding.
* **Numbers as cited consts in `science.rs`** (the Q10 / leaf-form precedent): a new YAML key would
  join the frozen param census.
* **Outdoors it cannot fire:** the open field's net radiation is a daily mean
  (`weather::net_radiation`), and the weather file's lowest IRRAD is 4.2e5 J m⁻² d⁻¹ (> 0, measured).
* **Carried on `TranspirationParams`** beside `rs_form` (never loaded from the file; the loader sets
  `Open`), read by the ONE production builder (`system.rs` `transpiration_flow`) into `CanopyRead`.
  Every other `Transpiration { … }` is a `flows.rs` unit test. The lab minute-step transpiration
  re-evaluates the same flow, so it goes through `CanopyRead` too.
* **How the lab runs reach the station:** the sealed station builds its crop from the frozen params
  (`build_season`, no injection seam). Rather than add one, the measurement runs in a scratch
  worktree under `W:\temp\claude` with the loader's default flipped — the route the air-resistance
  pricing took. That also prices adoption directly: `regen_goldens` reports which goldens move.

## 2a. Which goldens can move (predicted from each build's net-radiation forcing)

| golden | net radiation it reads | can `Closing` fire? |
|---|---|---|
| `lighting_state`, `sealed_station_state`, `sealed_energy_drift_summary` | the lamp's top-hat (`lighting::lamp_net_radiation_path`, `sealed::sealed_bio_resolver`) | **yes** |
| `greenhouse_state`, `harvest_state` | the weather's daily mean (`greenhouse_bio_resolver` → `weather_resolver`) | no |
| the 7 biosphere goldens (`season_euler`, `state_snapshot`, `sealed_chamber`, the two consumer and two perennial) and `drift_summary` (the consumer/perennial jars' drift) | the weather's daily mean, sealed jars included | no |
| `station`, `cabin_gas`, `crew`, `eclss`, `power`, `power_self_discharge`, `thermal`, `water_recovery` | no crop | no |

⚠ This corrects the latent-heat plan's §5b (*"it would move water in every sealed golden"*): the
biosphere crate's sealed jars read the outdoor daily mean and never see a dark step. Only the
lamp-lit station builds can move.

## 3. Values priced

The ratio is the user's call. Priced: **f = 0.123** ([N1] booting control) and **f = 0.194** ([N1]
heading control). [N2]'s 5.7 % and [N1] Table 2's 32.7 % bound it.

## 4. Predictions (written before any code ran)

* **P1** Every frozen build bit-identical under `Open`; goldens 20/20.
* **P2** Under `Closing` as the default, `regen_goldens` reports exactly the three §2a "yes" goldens
  moved and the other 17 identical. The open season is also asserted bit-identical in a test.
* **P3** Crop transpiration on FULLY DARK windows at the chamber's 7.5 falls about 4–6×. Penman–
  Monteith at `Rn = 0`, 22 °C (Δ ≈ 162 Pa/K, γ = 67), full canopy (rs 50): the denominator
  `Δ + γ(1 + rs/ra)` goes from ~680 to ~2500 (f 0.194) or ~3900 (f 0.123), so 116 W becomes ~31 W
  or ~20 W — if the chamber's night VPD holds (checked in the run). Partly lit windows are unchanged
  by construction; their dark part is reported separately as the rule's bias.
* **P4** The ≥ 1.80 GJ dark-hours heat shortfall (`log/chamber-latent-heat.md`) falls by about the
  same factor, to ~0.3–0.5 GJ.
* **P5** The TM 102788 water ratio (1.04) **falls**, by roughly the dark share of the trial's water
  times ~0.8. The trial ran 4 dark hours in 24 at 16.8 °C and 82 % RH, so the dark share is small;
  guess 1.04 → 0.95–1.00. A fall below 0.95 is a finding about that guess.
* **P6** The sealed builds' carbon moves only through soil water (less drawn at night, so less stress
  if any), so carbon readouts move little; if a carbon readout moves more than 1 %, that is a finding.

## 5. Measurements

1. P1, P2: the goldens + `regen_goldens` report on the frozen tree; the open season run under
   `Closing` compared bit for bit.
2. TM row water ratio under each f.
3. The chamber latent probe (`station/examples/chamber_latent_heat.rs`) under each f: dark-window
   latent, the shortfall.
4. Sealed station runs: total transpiration, condensate, carbon readouts against `Open`.

## 6. Results

*(to be filled from the runs)*
