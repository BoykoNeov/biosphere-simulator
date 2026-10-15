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
  centred on midday (lights 04:00–20:00) leaves 2 of its 8 dark hours inside partly lit windows
  (03:00–04:30 and 19:30–21:00, 1 h dark each), so a quarter of the dark time keeps open stomata; the
  20-h TM row, 1 h of its 4. ⚠ *First written "1.5 of 8", corrected from the windows' edges before
  the results were read.* Stated as a bias
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

## 6. Results (2026-10-15)

**How it ran.** The lab form committed on main (`829b7f1`, loader `Open`); the probe's vapour readout
added (`039a51a`). Baseline on main; then a scratch worktree of `039a51a` with the loader line flipped
to `Closing(NIGHT_CONDUCTANCE_RATIO_BOOTING)`, then `…_HEADING`. Per value: `chamber_latent_heat`
(the sealed station, 1220 days), the TM scorecard, `regen_goldens` (report). Every control held in
every run: no rationing (slow 0, fast 0), legs re-applied with 0 mismatches, water books closing to
≤ 6e-12 kg; the baseline probe's golden byte-exact.

| | Open (today) | Closing 0.123 (booting) | Closing 0.194 (heading) |
|---|---|---|---|
| warm dark windows, crop latent | 116.19 W | **19.75 W** | **30.20 W** |
| crop dark/lit rate, warm | 0.925 | 0.157 | 0.240 |
| dark-window heat with nothing to pay for it (lamp off at night) | 1.80 GJ | **0.31 GJ** | **0.47 GJ** |
| water through the station's chamber, 1220 days | 3473 kg | 2832 kg (−18.5 %) | 2918 kg (−16.0 %) |
| chamber vapour, warm dark windows (mean) | 3.3332 kg | 3.3379 kg | 3.3316 kg |
| TM 102788 W1 (water, days 25–80) | 1.041 | **0.958** | **0.967** |
| TM carbon rows (C1, B1, U1, N1) | — | identical | identical |
| goldens moved | — | 2 | 2 |

**Graded.**

* **P1 held**: on main under `Open`, the full suite green, goldens unchanged.
* **P2 MISSED, one of three.** Predicted three goldens; two moved (`lighting_state`,
  `sealed_station_state`). `sealed_energy_drift_summary` is identical: it holds only
  `horizon_years`, `is_stationary` and `node_peak_temp_k`, and the model gives evaporation no heat,
  so a water-only change cannot reach it. The prediction asked *"can the switch fire in this build"*,
  not *"does this golden record anything the switch moves"*. The other 17 identical, as predicted.
* **P3 held**: 116 W → 19.75 / 30.20 W against the predicted ~20 / ~31. The night air held its
  vapour (3.333 → 3.338 / 3.332 kg), so the drop is not offset by a drier night.
* **P4 held**: 1.80 GJ → 0.31 / 0.47 GJ against the predicted ~0.3–0.5.
* **P5 held**: TM W1 1.041 → 0.958 / 0.967 against 0.95–1.00. Carbon untouched (the trial row is
  never water-limited).
* **P6 held, more strongly than stated**: in both moved goldens only water moved — condensate, soil
  water, subsoil water and the soil-surface counters (`top_soil_water`, `soil_dry_days`,
  `soil_evap_today`). No carbon, nitrogen, oxygen or energy value. The sealed station's end snapshot
  shifts ~51 kg from condensate to soil water; it is one day's state in a watering cycle, not a
  run total (the run total is the row above).

**The rule's bias, estimated (not run).** The partly lit windows count as day, so their 2 dark hours
per warm day keep open stomata. At the measured open dark rate (116.19 W) those hours carry ~0.60 GJ
over the 716 warm days; closing them as the dark windows close would remove another ~0.50 GJ
(booting) or ~0.44 GJ (heading) — about as much again as the dark windows have left. A lit-fraction
rule is the form that would; it needs a lit-fraction signal the flow does not have.

**The four other readers of `science::canopy_surface_resistance`, re-read (as §2 promised).**

* `station/tests/scorecard_tm102788.rs` — **integrates nights at the day value.** `w1_by_hand` and
  `w1_by_hand_with` sum every step of a TM day, dark ones included, at one daytime `rs`; the §11 Q2
  "model / hand" line and the §13 rows would drift on adoption. All print, none asserts — except R1
  (`a_what_if_air_resistance_reaches_the_water_flow`), which runs the CONSTANT form, where the rule is
  not read, so it stays exact. On adoption: apply the rule in the by-hand path, or say the rows are
  daytime-resistance arithmetic.
* `domains/tests/canopy_resistance.rs` (flow = by hand to 1e-12) — biosphere scenarios on the weather's
  daily mean: never a dark step, unaffected.
* `domains/tests/soil_evaporation.rs` S3 (crop + soil radiation term ≤ net radiation) — a closed stoma
  only lowers the crop term; unaffected.
* `domains/examples/chamber_dryness.rs` — reads the file's constant 70 s/m on a biosphere jar (daily-mean
  light); unaffected.

**The suite with closing adopted** (the worktree at `Closing(0.194)`, its two goldens rewritten,
`--release --no-fail-fast`): three reds, **no behaviour pin among them** —
`the_loader_keeps_the_stomata_open` (by design), `station` `manifest_writer`'s
`the_committed_manifest_is_what_the_reference_writes` (the two golden hashes, the ceremony's
regeneration), and `closing_reaches_the_flow_on_a_dark_step_by_exactly_the_rule`, an instrument fault:
it took the loader's params as its open baseline, so under adoption the heading case compared with
itself. Fixed on main the same day (the baseline is now `NightStomata::Open` by name).

**What adoption would cost.** A biosphere unfreeze (the loader line and the two consts' citation
standing in a frozen form), two station goldens, water only, and the station manifest regenerated. The TM water match goes from 1.04 to
0.96–0.97 (3–4 % under, where it was 4 % over), and the trial's own record cannot say how much of
its water was night water (`post-roadmap-real-world-checks.md` §12d).
