# Post-roadmap — the plants' temperature in a station room (the 2026-09-29 review's Step 3, slice 3c — a design note, no code)

**Opened 2026-10-01** on the user's agreement to the recommendation *"the Step 3c design note
next"*. Review plan: `docs/plans/post-roadmap-review-2026-09-29.md`, Step 3, slice 3c, which says
*"first a design note, not a build … decided again once the note exists."* **This file is that
note. Nothing in `rust/` changed.** The decisions in §9 are the user's.

## 1. What the sealed station's plants read today

| Input | Source today | Value |
|---|---|---|
| Temperature | the weather file (`weather_forcings`, `TEMP_VAR`) | a Dutch season, 1 Oct → 1 Aug: **−1.8 to 22.2 °C, mean 10.67 °C** (305 rows) |
| Net radiation (transpiration's energy term) | the weather file, `(1 − 0.23)·IRRAD/86400` | outdoor net shortwave, **mean 88.0 W/m², 3.7–243.8** |
| Air dryness | the chamber's own vapour store (3b, frozen 2026-10-01) | but computed at the **weather's** temperature |
| PAR, daylength | the lamp (`sealed_bio_resolver`) | 200 W, 16 h |

Temperature is read at **eight** sites in the biosphere, found by listing every
`env.get(&self.temp_var)` in `domains/src/biosphere/flows.rs`, not by name: photosynthesis and
maintenance respiration (`CarbonContext`), `Transpiration` (Penman–Monteith, the humidity target
and 3b's chamber deficit), `Condensation` (the humidity target), `ThermalTimeAccumulation`
(development), `VernalizationAccumulation`, `RootDepthExtension`, `RootZoneCapture`. A room
temperature moves all eight at once. That is the review plan's *"every temperature-driven process
moves at once"*, now as a list.

## 2. The station's heat model is not a room — measured

`thermal.node` is one heat store with three numbers (`domains/params/thermal/radiator.yaml`):
ε = 0.85 (CITED, a coating range), area A = 10 m² (DESIGN), heat capacity C = 1e7 J/K (DESIGN,
chosen so Euler stays stable). The radiator rejects `εσA(T⁴ − T_space⁴)`. Whatever heat goes in,
the node settles where the radiator balances it.

* **Heat into the sealed station's node:** charge loss 15.9 W + the load 302.4 W + lamp waste heat
  61.0 W = **379.2 W** (`sealed::sealed_node_heat`).
* **Where that puts it:** 167.5 K by the closed form. **Measured** in the frozen
  `rust/data/golden/sealed_station_state.json` end state: `thermal.node` = 1.6472e9 J, so
  **T = 167.42 K = −105.73 °C.**
* ⚠ `docs/station-reference.md`'s *"T_eq ≈ 160 K"* is the **heat-closure** scenario (no lamp, no
  plants), a different run. The two numbers are not the same claim.

So wiring the plants to the node as it stands puts them at −106 °C. The node stands for
"structure + coolant + water stores", and the temperature it reaches is set by a radiator area
that `radiator.yaml` itself says no source can fix. To land at 295 K with today's load the area
would have to be **≈ 1.04 m²**. Choosing a design number so that a temperature comes out right
is calibration in disguise (§5, option C).

**Gaps in the heat books** (recorded here; none is changed by this note):
* The lamp's light leg (η ≈ 0.547 of its draw, ≈ 73 W averaged over the day) goes to
  `boundary.light_used` and **leaves the station**. In a real room, light absorbed by leaves and
  walls becomes heat. Only the few percent fixed as sugar does not.
* Crew body heat (BVAD Table 3-31: 12.426 MJ per crew member per day ≈ 143.8 W) is not in the
  books. This was already listed in `docs/bvad-reference.md`.
* The latent heat of transpiration and condensation is not tracked. In a sealed room, heat
  carried off the leaves as vapour is released again at the condenser. It moves *where* heat is
  rejected, not how much.

## 3. What a station does instead — sourced

* BVAD Rev 2 (NASA/TP-2015-218570, Feb 2022), printed p. 6: *"The Thermal Interface is responsible
  for maintaining cabin temperature and humidity … within appropriate bounds and for collection
  and removal of the collected waste heat."* So a cabin's temperature is a **held** quantity, like
  the CO₂, O₂ and humidity this model's ECLSS already holds.
* BVAD **Table 4-73, "Crew Cabin Thermal Ranges", printed p. 153** (read as the page image,
  2026-10-01): air temperature **291.15 / 295.15 / 300.15 K** (lower / nominal / upper; 18 / 22 /
  27 °C). Nominal is attributed to Thirsk et al. (2009), the limits to NASA HIDH (2014).
* BVAD **Table 4-111, printed p. 188** (page image, 2026-10-01): **wheat 23 °C light and dark, 20 h
  photoperiod**; white potato 20 / 16 °C, 12 h. In the text: *"some key plants, such as wheat and
  potatoes, are most productive at temperatures below the standard crew comfort zone."*
* ⚠ **Locus.** Table 4-111's wheat belongs to BVAD's own crop model (the Modified Energy Cascade),
  run under a 20 h photoperiod. Ours is winter-wheat physiology under a 16 h lamp. So 23 °C is at
  most a **class** citation, a crop-room figure from another model. The direct citation for *a
  station room* is the cabin nominal, 295.15 K. Recommended: 22 °C as the value, 23 °C as contrast.

## 4. What a warm room really costs: the crop and the calendar

Swapping the temperature is nearly free. `lighting.rs` already overrides `TEMP_VAR` with a
constant (`habitat_temp_c`), and `sealed_bio_resolver` just lacks the line. What it costs is the
crop. Checked against the function, not against a comment:

* Vernalization (`phenology.yaml`, [C] Eqn 8.3) accrues chill-days only **below 12 °C**
  (`t_ceiling_v`). Its factor (Eqn 8.6) is `1 − vsen·(VDSAT − CUMVER)` with vsen = 0.033 and
  VDSAT = 50. The product is 1.65 > 1, so the cultivar is *qualitative*: the factor is **exactly 0
  until ≈ 19.7 chill-days have accrued**, and it multiplies development in the whole vegetative
  phase.
* **So any room held at 12 °C or warmer arrests the frozen winter wheat before flowering,
  forever.** Today it vernalizes off the Dutch winter: the frozen end state carries
  `vernalization_days` = 143.37.
* The day-neutral crop already exists (`vernalization: false, photoperiod: false`, the cited
  winter-wheat params with both gates removed). Precedent: `day_neutral_lighting_scenario`, a
  lamp-lit room at 20 °C, authored content with no golden.

**The calendar.** The sealed station re-sows on a fixed **305-day** calendar (`annual_reset` via
`slow_reset`). The crop's thermal sums total 1100 + 750 = **1850 °C·day**. At 22 °C that is
**≈ 84 days to maturity**, so ≈ 220 days a year with a finished crop standing in the room until the
calendar re-sows it. Today's crop uses the whole Dutch season. This is likely the **largest golden
mover** of anything here, and it is a choice: re-sow on maturity, or keep the calendar.

## 5. Three forms for the temperature itself

**A — a held room, constant.** In the sealed station, `TEMP_VAR` becomes a constant setpoint
(the `habitat_temp_c` precedent, added to `sealed_bio_resolver`). The heat model and the energy
books are untouched, and the plants never feel a heat fault.
*Moves:* the `sealed_station` golden, the Godot sealed session, and the lab runs built on the
sealed station. *Does not move:* any biosphere golden, heat closure,
`sealed_energy_drift_summary`, or the node gate. Both of the last two read the heat-closure run
(`science_gates::runs::node_peak_temps` builds `HEAT_CLOSURE_SCENARIO`), not the sealed station.

**B — a held room that can fail.** A room-air heat store on the fast step. Into it goes all of
the lamp (light included), crew heat and equipment heat. A controlled exchanger moves heat from
the room to the node, up to a capacity, and the radiator is unchanged. In nominal runs room =
setpoint, so B equals A. Under a radiator or power fault the room drifts and the plants feel it.
*Costs:* new DESIGN numbers (exchanger capacity, the room's heat capacity — air alone is
9500 mol × 29.1 J/mol·K ≈ 276 kJ/K, far too small to stand alone), a new energy flow set (a
station unfreeze), the lamp's light leg re-pointed (the energy ledger moves), and a change in what
`thermal.node` means. **B is A plus more, so A first loses nothing.**

**C — a free-floating room.** No controller, so the temperature falls out of the heat balance.
It would be set by a DESIGN radiator area (−106 °C at 10 m², 22 °C at ≈ 1 m²). Every number in
the chain is DESIGN. **Recommend against.**

## 6. Out of scope, by name: net radiation

The evaporation energy term is still outdoor sunlight under a lamp. This was found in 3a and
deferred in 3b. Under the lamp the canopy gets ≈ 109 W/m² of light for 16 h (≈ 73 W/m² averaged
over the day). The weather's figure has a similar mean, 88, but swings 66-fold over the season.
**Not taken in 3c:** it is the *light* path, and changing it together with temperature would put
two causes in one golden diff. Candidate as its own item (a "3d"), the user's call.

## 7. The roster — every sealed build that reads the weather's temperature

| Build | Heat model? | In 3c? | Why |
|---|---|---|---|
| `sealed_station` (golden; Godot sealed session; lab `lamp_shed`, perturbed brown-out) | yes | **in** | the only build with plants *and* a station heat model |
| `greenhouse`, `harvest` | no node | out | cabin-coupled; no station heat model to hold a room. A setpoint there is a separate choice |
| `lighting` (golden, `habitat_temp_c: None`) | waste heat to a boundary | out | frozen 7-day lamp check; already has the override hook |
| biosphere `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon` | none | out | the biosphere's own jars, with no station around them |
| open-field `season` | none | out | not sealed; the weather *is* its climate |
| `day_neutral_lighting_scenario` (authored) | — | out | already 20 °C constant |

## 8. Predictions owed before any code — one slice, one cause

The crop swap and the temperature swap are **separate science changes**. One golden diff
carrying both explains neither. Proposed order, each slice measured on its own:

1. **Temperature alone, winter wheat kept (a check, lab only):** at 22 °C the crop never reaches
   flowering. No chill-days ever accrue, so the factor is 0 from sowing, development never
   starts (thermal time stays 0), and grain is 0. This is the §4 claim tested by running it.
2. **Crop alone (day-neutral, weather temperature kept):** development is no longer gated by cold
   or daylength. Predicted: earlier maturity than today's crop in the same Dutch season. The size
   of the change is to be measured, not guessed.
3. **Room setpoint (form A) on the day-neutral crop, calendar kept:** maturity ≈ 84 days after
   sowing, then standing until day 305.
   * Water: at 22 °C saturation pressure is 2.64 kPa against the season mean 1.35, so the chamber's
     dryness roughly **doubles (×1.96)** on top of 3b's +20.6 %. 3b's *"no crop reaches water
     stress"* no longer holds by argument. Which store binds first (soil water or the chamber's
     vapour store) is to be measured. No guess is recorded as a prediction.
   * Carbon: maintenance respiration and photosynthesis both rise with temperature. The sign of
     the net change is not predicted.
4. **The calendar (if chosen):** re-sow on maturity. Over the 4 × 305 = 1220-day horizon, ≈ 14
   crops instead of 4.

## 9. Decisions — the user's

1. **The temperature form.** A (held constant) now, B (can fail) later on top, C refused.
   *Recommended: A.*
2. **The crop.** A warm room arrests the frozen winter wheat. Options: the existing day-neutral
   crop; a cold treatment in the room's schedule (needs its own source); or stop and keep the
   weather. *Recommended: day-neutral.*
3. **The calendar.** Re-sow when the crop matures, or keep 305 days. *No recommendation until slice
   3 shows how the standing crop behaves.*
4. **The setpoint value.** 22 °C (BVAD cabin nominal, a direct citation for a room) or 23 °C (BVAD's
   crop table, another model's wheat). *Recommended: 22 °C, with 23 °C recorded as contrast.*
