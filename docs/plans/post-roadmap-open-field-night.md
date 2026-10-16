# The open field gets a night — the light first, the air next

**Opened 2026-10-16** on the user's *"Open field night"*, the item the lit-share adoption left owed
(`post-roadmap-lit-share-stomata.md` §6: the weather supplies `lit_fraction = 1` as a staging step).
Asked how far to take it, the user chose **"light first, air next"**: step 1 shapes the weather's net
radiation within the day, lab-only, and measures it; step 2, **before any adoption**, gives the air a
day/night dryness cycle from re-fetched daily min/max temperature (water only, not growth). Adopting
step 1 alone is not on the table: it would be a half-built form (below, §4).

## 0. Why

The weather fixture holds one value per day. `weather::net_radiation` is its **24-h mean**, net
shortwave only (`(1 − α)·Rs`), so it is > 0 on every step and the open field never has a night: the
night rule (`Rn ≤ 0` is dark, ASCE-EWRI's hourly rule) never fires there, and its stomata stay open
through every night. PAR already follows the sun (`sine_light_path`, a half-sine over the astronomical
day, [E] Penning de Vries); the net radiation is the one light-driven forcing that does not.

## 1. The form (step 1)

`system::WeatherRadiation { DailyMean, SunPath }`, chosen by `weather_forcings_with` /
`weather_resolver_with`. `weather_forcings` keeps `DailyMean`: the loader, every golden and every
station build are unchanged.

* **`SunPath`:** the step's net radiation is `half_sine_window_mean(t0, dt, Rn_24h · 86400 / D, D)` —
  the function, the day length row `D` and the solar-noon centring PAR already uses. Since both PAR
  and this net radiation are fixed fractions of the same daily shortwave, **each step's Rn is a fixed
  multiple of its PAR**, and the day's total is kept exactly (the half-sine's dose property).
* **`lit_fraction`** is the step's daylit share, `top_hat_window_mean(t0, dt, 1, D)` — the same span.
  So PAR > 0 ⇔ Rn > 0 ⇔ φ > 0 on every step, by construction, and a sunrise or sunset step is split by
  the adopted lit-share rule exactly as a lamp's switch step is.
* A day with `D = 0` is refused (its radiation would have nowhere to go). Not reachable at 52° N.
* **Nothing new is cited and no number is new.** The shape is [E]'s, already on the shelf for PAR.
  ASCE-EWRI is only the source of the `Rn > 0` day rule; it works from measured hourly radiation and
  is **not** cited for the shape.
* **What it leaves out, stated:** the net radiation stays shortwave only, so a night step is `Rn = 0`,
  not the negative net longwave a real night has. The night rule reads `Rn ≤ 0` either way; the
  missing longwave is a bias in the night's energy term (a real night's Rn < 0 would draw the
  Penman–Monteith radiation term below zero, clamped at the flux), not in which steps are dark.

### What can move, and what cannot

Penman–Monteith is linear in Rn at a fixed resistance, temperature and deficit (outside its `max(0)`
clamp). Temperature and deficit stay daily means in step 1, so spreading the same daily Rn over a
half-sine is, on its own, nearly a no-op for a day's water. What moves it:

1. **The night steps' stomata close** (`rs_day / f`, f = 0.123): the drying-air term drops on every
   dark step. This is the change.
2. **Sunrise and sunset steps** are split by the lit share (2 a day).
3. **Non-linear readers:** the two-stage soil evaporation (its stage clock and daily cap), the sealed
   weather-lit jars' humidity cap, the per-step supply caps — the timing of the day's water moves.

## 2. Predictions (written before any run)

| | prediction | confidence |
|---|---|---|
| N1 | the loader moves **no golden** (`DailyMean` is the old forcing to the bit) | high |
| N2 | under `SunPath`, every day of the tiled season: Σ Rn·dt = the daily mean to ~1e-12 relative; PAR > 0 ⇔ Rn > 0 ⇔ φ > 0; Rn / PAR one constant `(1 − α) / (0.5 · 4.57)` to rounding | high |
| N3 | φ = 1 on steps wholly inside the day, 0 wholly outside; two partly lit steps a day (sunrise, sunset) on almost every day | high |
| N4 | the open field's season crop water (`vapor_sink`) **falls 7–15 %**: summer nights are ~⅓ of the day, the drying-air term is about half of the crop's open-stomata water and closes to ~⅓ of itself (by hand, 17 °C, 0.7 kPa, ra 50, rs ~60) — ⅓ × ⅔ × ½ ≈ 11 % | medium-low |
| N5 | the soil's evaporation moves **< 5 %**, sign not predicted (same daily Rn; only its timing moves, plus any watering-trigger knock-on) | low |
| N6 | carbon moves only through water stress: grain within **±2 %**, and bit-identical if the irrigated field's stress never binds in either run | medium |
| N7 | the weather-lit sealed jars' crop water falls in the same direction; their carbon (isolation-limited) moves < 1 % | medium-low |
| N8 | with the loader flipped (scratch worktree): the goldens built on the weather move — `season_euler`, `harvest`, `greenhouse`, `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`, and `drift_summary` if carbon moves; `lighting_state` and `sealed_station_state` **byte-identical** (their builds replace both forcings with the lamp's) | medium-high |
| N9 | no lab room-heat pin moves (they are lamp-lit rooms); any lab pin built on the greenhouse or a weather-lit jar may | medium |

## 3. Steps

1. The form, and its tests (N1–N3) — `domains/tests/open_field_night.rs`.
2. The probe: the open field and the weather-lit jars, both forms (N4–N7).
3. The loader flipped in a scratch worktree under `W:\temp\claude`, `regen_goldens` report and the
   test suite (N8, N9).
4. Grade, record. **No adoption decision here** — step 2 (the air) comes first.

## 4. Step 2, the air (scoped, not built)

With the deficit a daily mean, closing the stomata at night while keeping the daytime air as damp as
the night's **biases water low**: the deficit is understated in exactly the hours the stomata are
open, and the saturation curve's curvature adds to it. The fixture holds no daily min/max
temperature (checked 2026-10-16: the three NASAPower files in the tree hold TEMP, IRRAD, VAP only;
`tests/oracle/runner.py` keeps just those three), though the source provides them. Step 2 re-fetches
the fixture with TMIN/TMAX (a frozen input: the manifest's `weather_sha256` moves), gives the deficit a
within-day cycle from a cited diurnal temperature form, and feeds it to the water reads only.

⚠ **The re-fetch must not move the existing three columns** (advisor, 2026-10-16). NASA POWER revises its
archive between versions, so a fresh download of the same 2006–07 days can return different TEMP, IRRAD
and VAP — and every golden would then move for a reason unrelated to the min/max. Take only the new
TMIN/TMAX columns, check the three committed columns value for value against the new download, and record
the service version. Also check how the provider forms the daily TEMP — the mean of min and max, or a 24-h
mean — since that decides where a within-day curve sits on the existing value.

## 5. Results (2026-10-16)

**How it was run.** Step 1 in the tree: the form (`system::WeatherRadiation`,
`weather_forcings_with`, `weather_resolver_with`) and 4 tests (`domains/tests/open_field_night.rs`),
green. The probe `domains/examples/open_field_night.rs` runs each weather-lit biosphere golden's build
under both forms from the loader's params. The goldens and the suite were run in a scratch worktree of
`ce7a19d` plus this work, with one scratch-only edit: `weather_forcings` / `weather_resolver` set to
`SunPath`. The cited outputs are kept at
`W:\Claude_projects\space-station\sources\temp-evidence\open-field-night\` (git-ignored, local).

### 5a. The probe (rationed 0 and no events in every run)

| build | water drawn, daily mean (kg) | sun path (kg) | change | of it at night (sun path) | carbon |
|---|---|---|---|---|---|
| open field, 1 season | 548.51 | 487.00 | **−61.5 (−11.2 %)** | 20.1 kg over 2180 dark steps (4.1 %) | bit-identical |
| sealed chamber, 3 yr | 713.75 | 480.07 | **−233.7 (−32.7 %)** | 4.9 % | bit-identical |
| perennial chamber, 5 yr | 2454.35 | 1757.84 | **−696.5 (−28.4 %)** | 4.0 % | bit-identical |
| consumer chamber, 5 yr | 2094.98 | 1563.43 | **−531.6 (−25.4 %)** | 3.4 % | bit-identical |

"Drawn" is the crop and the soil together, from the root zone. The open field's own sinks: the crop
502.63 → 445.22 kg (**−11.4 %**), the soil 45.87 → 41.78 kg (**−8.9 %**). The watering source ends
4.6 kg lower (−465.7 → −470.2 kg) — a **net**: the deep overflow returns water to it (`DeepOverflow::Recycled`),
so this is not "more irrigation". The end-of-run soil water (+71 %) is one point in the watering cycle,
not a season total.

**Why the jars move more than the open field — measured in part.** First written as "their air
resistance is 7.5 s/m against 50", unmeasured; then tested by substituting 50 for the sealed builds'
`chamber_aerodynamic_resistance` (the probe's `--sealed-ra 50`, through the value-switch harness):

| build | change at 7.5 s/m (loader) | at 50 s/m | the open field |
|---|---|---|---|
| sealed chamber, 3 yr | −32.7 % | −16.6 % | −11.2 % |
| perennial chamber, 5 yr | −28.4 % | −21.2 % | |
| consumer chamber, 5 yr | −25.4 % | −19.9 % | |

The air resistance carries **~75 %** of the sealed chamber's excess over the open field and **~40 %** of
the perennial and consumer chambers'. The rest is something else the jars have and the field does not —
candidates: they read the deficit from their own air (`VpdRead::Chamber`), which a crop drinking less
dries further; the humidity cap; recycled water. **Not identified.**

**The soil's −4.1 kg is not the top layer's cap:** the evaporation on steps that emptied the top layer
is the same in both (10.28 vs 10.31 kg). Radiation-only, the soil's daily potential is unchanged
(`soil_potential_today` equal to 1 ulp), so the loss is in the rate: the Stage I / II switch inside a
day, or the 1.5 mm daily floor. **Not traced further.**

### 5b. The goldens (`regen_goldens`, flipped): 20 run, **8 change**, 12 byte-identical

`season_euler`, `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`,
`consumer_long_horizon`, `greenhouse`, `harvest` — exactly N8's list. **Every changed value is water:**
23 water stock amounts and the soil-evaporation clocks (`soil_dry_days`, `soil_evap_today`,
`top_soil_water`; `soil_potential_today` at 1 ulp, a sum taken in a different order). No carbon,
nitrogen, thermal-time or rooted-depth value moved; `drift_summary` (carbon) is identical, as are
`lighting_state` and `sealed_station_state`.

### 5c. The suite, flipped (`cargo test --no-fail-fast`, then `-- --ignored`)

Reds outside Godot: **7**.

* **2 manifest writers** (`domains`, `station`) — the goldens were rewritten in the worktree, and each
  manifest names its goldens' bytes. Owed by any adoption, not a finding.
* **3 claims that state "the open field has no night"**, red as they should be:
  `open_field_night::the_loader_keeps_the_daily_mean` (this item's own pin),
  `lit_share_stomata::the_weather_is_wholly_lit_and_moves_nothing` (the staging step's pin),
  `night_stomata::the_open_field_never_sees_a_dark_step`.
* **2 claims that assumed it, unpredicted:** `night_stomata::closing_reaches_the_flow_on_a_dark_step_by_exactly_the_rule`
  ("Closing(0.123) moved a lit step") and `canopy_resistance::each_lab_form_reaches_the_water_flow_exactly`
  (its by-hand water omits the night rule). Both read the open field at **step 0, which is midnight**
  under the sun's path. An adoption restates their baselines (a lit step chosen by its light, not by
  its index); neither is a defect in the form.

The 17 `godot_bridge::cross_boundary` reds are the **environment, not the flip**: the worktree's
Godot project could not find its extension library (`res://simcore.gdextension` — the worktree built
into a redirected target directory). Not measured under the flip; both sides of those checks run the
same core. The 6 slow (`--ignored`) tests: **all pass** flipped. No lab room-heat pin moved.

### 5d. Graded

| | | |
|---|---|---|
| N1 | the loader moves no golden | **HIT** (the committed tree's suite and goldens) |
| N2 | dose kept to 1e-12, zero together, Rn / PAR one constant | **HIT** (test, every day of the season) |
| N3 | φ ∈ {0, 1} away from sunrise and sunset, two split steps a day | **HIT** (≤ 2 every day, exactly 2 on ≥ 95 % of days) |
| N4 | open-field crop water −7 to −15 % | **HIT** (−11.4 %) |
| N5 | the soil < 5 % | **MISSED** (−8.9 %; the rate's timing inside the day, not the cap; not traced) |
| N6 | carbon only through water stress, ±2 %, bit-identical if stress never binds | **HIT** (bit-identical) |
| N7 | the jars' water falls, carbon < 1 % | **HIT on direction and carbon** (bit-identical); the size, −25 to −33 %, was not predicted |
| N8 | the 8 named goldens move; the lamp builds byte-identical | **HIT** (exactly those 8; `drift_summary` identical, carbon unmoved) |
| N9 | no lab room-heat pin moves | **HIT** — but 2 test premises ("step 0 is lit") went red unpredicted |

## 6. What it says, and what is next

Giving the open field a night removes about a ninth of its season's water and a quarter to a third of
the weather-lit jars', and touches no carbon: the crop's water stress never binds in any of them. But
the air stays at its daily-mean dryness, so the hours the stomata are open read the day's average
deficit rather than the afternoon's — the water above is **likely low, so the saving likely overstated**
(argued, §4, not measured), not the answer. **Next: step 2, the air** — re-fetch the fixture with TMIN/TMAX (a frozen input), choose a
cited within-day temperature form, feed its deficit to the water reads, and measure the two steps
together before any adoption. The user's to open.
