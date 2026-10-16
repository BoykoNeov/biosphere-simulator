## **The open field gets a night** (the lit-share adoption's staged item — the weather's net radiation was a 24-h mean, > 0 on every step, so the open field and the weather-lit jars never had a dark step and their stomata never closed)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-16** on the user's *"Open field night"*. Asked whether the air should get a day/night
cycle too, the user chose **"light first, air next"**: step 1 shapes the net radiation, lab-only, and
measures it; step 2 gives the air a day/night dryness cycle (re-fetched daily min/max temperature, water
only) **before any adoption**. Plan, form, predictions written before any run:
`docs/plans/post-roadmap-open-field-night.md`.

**STEP 1 BUILT lab-only, 2026-10-16.** `system::WeatherRadiation { DailyMean, SunPath }`, through
`weather_forcings_with` / `weather_resolver_with`; `weather_forcings` keeps `DailyMean`. `SunPath` puts
the day's net radiation on the half-sine PAR already follows (`half_sine_window_mean` at
`Rn_24h · 86400 / D`, the same day length row), so each step's Rn is a fixed multiple of its PAR and the
day's total is kept; `lit_fraction` is the step's daylit share (`top_hat_window_mean` over the same span).
No new number, no new citation ([E]'s shape; ASCE-EWRI only for the `Rn > 0` rule). Tests:
`domains/tests/open_field_night.rs` (the loader's forcing to the bit; each day's dose to 1e-12, Rn / PAR
one constant, Rn, PAR and φ zero together, two split steps a day; a midnight step reaches the flow as
dark; a season runs). Probe: `domains/examples/open_field_night.rs`. Goldens unchanged.

**MEASURED 2026-10-16** (plan §5; the goldens and the suite in a scratch worktree with the loader
flipped). Water drawn from the root zone, crop and soil: the open field −11.2 % over its season (the crop
−11.4 %, the soil −8.9 %); the weather-lit jars −32.7 % (sealed, 3 yr), −28.4 % (perennial, 5 yr),
−25.4 % (consumer, 5 yr). **Carbon bit-identical in all four**: the crop's water stress never binds in
either form. 8 of 20 goldens change, the 8 predicted; every changed value is water or a soil-evaporation
clock; `lighting_state`, `sealed_station_state` and `drift_summary` byte-identical. No lab room-heat pin
moved. 5 test claims went red, all on the premise "the open field has no night" — 3 that state it, 2
that read the open field's step 0 (midnight) as lit.

**Lessons.**
* **The jars move two to three times the open field, and the first-written cause was half the story.**
  It was recorded as "their air resistance, 7.5 s/m against 50" before any run; the experiment removing
  it (the sealed builds at 50) took back ~75 % of the sealed chamber's excess and only ~40 % of the
  perennial and consumer chambers'. The rest is not identified (the jars' own-air deficit, the humidity
  cap, recycling are candidates). A cause written from arithmetic earns its run before it is a lesson.
* **"Nearly linear" was right about the radiation and silent about the soil.** The soil's potential is
  radiation-only and its daily sum is unchanged, yet it evaporated 8.9 % less: not the top layer's cap
  (the capped evaporation is equal in both), so the rate's timing inside a day — the Stage I / II switch
  or the 1.5 mm floor. Not traced.
* **A test's step 0 is a time of day.** Two tests read the open field at step 0 and called it a lit step;
  under a real day step 0 is midnight. A premise the forcing used to make true for every step is now true
  for some.
