## **Stomata that close for the dark part of a partly lit step** (the night-stomata rule's stated bias — a plant step the lamp is on for part of counts as day, so a quarter of the dark time kept open stomata)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-16** on the user's *"1"* (the first item of a "what is next" list). Plan, form,
predictions written before any run: `docs/plans/post-roadmap-lit-share-stomata.md`.

**BUILT lab-only, 2026-10-16.** `science::NightRule { NetRadiation, LitShare }` on
`TranspirationParams` (the loader's `NetRadiation`); `science::step_crop_transpiration` does the split:
`φ · PM(s·Rn/φ, rs_day) + (1 − φ) · PM(0, rs_day/f)`, day and night told by the step's own `Rn`, not the
crop's share `s`. `Rn ≤ 0` stays dark whatever φ says (a shed or failed lamp darkens the light, not φ);
`Rn > 0` with φ ≤ 0, or φ > 1, is refused. φ is `stocks::LIT_FRACTION_VAR`, supplied by
`lighting_bio_resolver`, the sealed station (as cold/warm twins, in `PLANT_PROGRAM_VARS`) and the TM
scorecard, each from the light's own top-hat. Switch: `domains::lab::with_night_rule`. Tests:
`domains/tests/lit_share_stomata.rs` (the form's edges bit-exact, the split's drop to 1e-12, the
refusal, the loader pin, an open field failing under the switch, the reach into `Transpiration`) and
`station/tests/lit_share_forcing.rs` (`Rn == φ·Rn_on`, `Rn == 0 ⇔ φ == 0`, warm φ = 1/3, cold 2/3).
Goldens unchanged.

**MEASURED 2026-10-16** (plan §5), in a scratch worktree with the loader flipped and the weather
supplying φ = 1. Over the sealed station's 1220 days: evaporation 2835.9 → 2594.1 kg (the crop −200.2,
the soil −41.6); latent heat 6.948 → 6.356 GJ; heater 1.340 → 0.872 GJ; cooler +0.12 GJ; battery
+4.8 %; re-sow days unchanged, grain +0.02 %. TM 102788: water 0.958 → 0.928, carbon unchanged, model /
hand 0.9962. Goldens: 2 of 20 change (`lighting_state` water only; `sealed_station_state` water,
energy, and carbon / nitrogen at 1–2e-4 through `thermal_time`); the 18 others byte-identical under
φ = 1. Predictions: 6 hit, L5's latent heat and L7's TM water missed (the soil's unpredicted share;
a dense TM canopy), L9 partly.

**Lessons.**
* **A water change now reaches the crop's clock.** Since the latent-heat books, water is heat in the
  room, and the room's heat is the crop's thermal time: a stomatal form moved carbon at 1e-4 with no
  carbon mechanism in it. Predict a water form's carbon as "through the room", not "unchanged".
* **The soil answered a crop-only change.** A form that cannot touch the soil's evaporation moved it
  −13 %, most likely through the watering trigger. A closed loop has no "only".
* **Instant closure is an upper bound** (a side note the user relayed): the hours this changes are the
  ones beside the switch, where stomata lag most. Carried into the decision, not modelled.

**A runtime regression, fixed the same day:** the commit's by-hand TM helper read the loader on every
step, taking `scorecard_tm102788` from under 3 s to 922 s; the follow-up reads it once (1.9 s). A
loader call inside a per-step closure is a cost no gate times.

**Owed to an adoption:** φ for the open field and weather-lit builds (φ = 1, or per build); the minute
water path (lab) fails under the switch (measured: it names the missing `lit_fraction`); the unfreeze
ceremony for 2 goldens; the soil's 42 kg traced.
