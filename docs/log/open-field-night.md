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

**STEP 2 BUILT lab-only and MEASURED, 2026-10-17** (plan §7), on the user's *"step 2"*; asked how far
the air's cycle should reach, the user chose **the open field and the jars** (the jars read their own
air, so a cycle on the weather file's deficit alone would never reach them). The fixture gained daily
`TMIN` / `TMAX` from NASA POWER (`ecb418d`, inert: the same download reproduced the three committed
columns bit for bit). The form (`93eec5a`): Teh (2006) §2.7's temperature curve through the observed
minimum and maximum, each step its window's exact mean, vapour pressure held at the day's;
`system::WeatherAir` supplies it, `science::AirTemperatureRead::WithinDay` makes every water reader take
it (transpiration, soil evaporation, the jar's own deficit, humidity bound and dehumidifier, the soil
account) while growth — and the roots' capture — keep the daily mean; lamp-lit station rooms build on
`lighting::room_params`. Tests `domains/tests/open_field_air.rs` (one records every flow's reads; its
teeth checked). Probe `domains/examples/open_field_air.rs`.

Light + air against the daily mean: water **−2.0 %** in the open field (step 1 alone −11.2 %), **−20 to
−21 %** in the jars (step 1 −25 to −33 %); **carbon bit-identical everywhere**; the jars condense at
night. The open field's sign depends on the curve's placement (shifted onto the observed mean: +2.4 %).
Flipped in a scratch worktree: the same 8 goldens as step 1, water only; station goldens identical; 19
test reds, none a defect in the form (2 manifests, 6 old-loader pins, 2 step-0, 4 mixed test builds, 4
held-room test builds, 1 moved drought ratio). **Adoption, and the placement, are the user's.**

**Lessons (step 2).**
* **A by-hand multiplier needs the weight it multiplies.** The prediction that light + air would end
  *above* the daily mean took the drying-air term as half the daylit water and multiplied it by the
  radiation-weighted 1.85×; the model's daylit water rose 1.14×. Missed on sign; the cause is not traced.
  The air-only row, predicted from the 24-h mean (1.000×), hit.
* **The flipped tree found a leak no golden could.** A station lab variant rebuilt its soil account from
  the loader's params, so under the flip it read the outdoor air inside a sealed room — silently, 1.3 g.
  The fix made the rooms *drop* the outdoor names, so a missed reader now fails instead of reading the
  outdoors. A guard that turns a silent wrong value into an error is worth more than one more pin.
* **A placement choice can be larger than the effect it places.** The 4.4-point swing between the curve
  as printed and the curve shifted onto the observed mean is bigger than the open field's net −2.0 %.

**ADOPTED 2026-10-17** (plan §9), on the user's *"Adopt, curve as printed"*: the loader takes the sun's
path and the within-day air for every water read. Inert plumbing first (`d79b99a`: every comparison
names its forms; held-room test builds read their room; reach tests read the crop at noon), then the
loader lines. 8 goldens (6 biosphere, 2 station — `greenhouse`, `harvest`), water only, **byte for byte
the flipped tree's**; both probes reproduce their saved runs; the lamp-lit rooms, `drift_summary` and the
TM 102788 scorecard unchanged; 5 pins restated (the retired forms keep theirs by name); the room guard
pinned (`station/tests/room_air.rs`); the deep-water drought ratio 1.0294 → 1.0930 recorded as moved.
**Opened:** the afternoon air reaches 2.03 kPa, so Sinclair & Amir's 1.75 kPa drought termination is now
within reach on this weather. ⚠ **WITHDRAWN the same day** (plan §9d): the table had been misread on
2026-10-06 — on the page, 1.75 kPa pairs with FTSW < 0.00 (a state neither the book's program nor this
model can hold) and the other two rows need VPD > 2.20, which no reading of this weather reaches (the
book's daily VPD peaks at 1.82, the afternoon at 2.03–2.10). The rule cannot fire here; nothing was built.

* **Lesson: read a table off the PAGE, not a text extract.** The extract had run the rows together, the
  2026-10-06 transcription paired the wrong VPD with each FTSW, and two records then carried it — a
  second error rode on it (an afternoon peak compared with a threshold the book defines on a daily
  weighted VPD).

* **Lesson: a flipped tree can be the prediction.** Measured in a scratch worktree before the decision,
  the adoption's golden diff was then reproduced line for line — the ceremony's "predict the diff" was a
  `cmp`, not an estimate.

**A4's miss traced, 2026-10-18** (plan §10–11; `domains/examples/open_field_air_terms.rs`). The
open field's light + air was predicted to end +10 to +35 % over the daily mean and ended −2.0 %; the miss
sits in the lit steps' crop multiplier, 1.146 against the estimate's ~1.42. Evaluating the real flow's
crop leg on every state of the run with the air swapped (the light-only side reproduces the separate run
to the gram, 445.223 kg) splits it exactly: the drying-air share of the daytime water is **0.30, not the
assumed 0.5**; the dryness ratio is **1.43, not 1.85** — the estimate weighted the afternoon's deficit by
the sun, but the drying-air term runs on open-stomata time, not light (uniform weight: 1.85 → 1.51), the
water is carried by the late, drier months (→ 1.48), and 0.05 is the script against the model (0.009 of
it the step's mean temperature, the rest not traced); the warmer afternoon's Δ is +0.02, upward. **The
estimate was wrong, not the model; nothing moved.** §7d's first candidate (the canopy's own resistance)
is a real but minor part: it cancels within a step, but decides which steps carry the season's water —
under the old constant resistance the share is 0.34, not 0.30.

* **Lesson: weight a term by what it multiplies.** The radiation weight belonged to the radiation term;
  borrowing it for the drying-air one doubled-down on the afternoon and dropped the cool morning.
* **Lesson: a same-state decomposition earns its reading only after it reproduces the separate run.**
  Here it did exactly — because the crop leg never sees the two runs' different soil water.
