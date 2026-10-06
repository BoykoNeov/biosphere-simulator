# Post-roadmap — the plants' temperature in a station room (the 2026-09-29 review's Step 3, slice 3c — a design note, no code)

**Opened 2026-10-01** on the user's agreement to the recommendation *"the Step 3c design note
next"*. Review plan: `docs/plans/post-roadmap-review-2026-09-29.md`, Step 3, slice 3c, which says
*"first a design note, not a build … decided again once the note exists."* **This file is that
note. Nothing in `rust/` changed.** The decisions in §9 are the user's.

**DECIDED 2026-10-01 (§10):** form B in a dedicated plant chamber (shared air, its own
temperature), the winter wheat with a cited cold period, re-sow on maturity, 22 °C.
**EXTENDED 2026-10-03 (§11):** separate air for crew and plants as a lab-only *option* (shared air
stays the default), joined by a fan that carries gas and heat; the cabin gets its own heat store.

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
  60.4 W = **378.7 W** (`sealed::sealed_node_heat`; lamp η = 2.5 / `PAR_UMOL_PER_J` 4.57 = 0.547).
* **Where that puts it:** 167.424 K by the closed form. **Measured** in the frozen
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
*Moves* (grepped for `sealed_bio_resolver`, `src/`, `examples/`, `tests/`, `godot_bridge`): the
`sealed_station` golden (`goldens.rs`), the Godot sealed session (`palette.rs`, `godot_bridge`),
the lab lamp shed (`examples/lamp_shed.rs`, `tests/lamp_shed.rs`), the `draw_census` and
`intraday_exchange` examples, and the sealed cases in `tests/{day_order, perturbations,
session_parity, session_save_load}.rs`. *Does not move:* the perturbed brown-out golden
(`emit_perturbed_brownout` builds the heat-closure `build_station`, which has no plants), any biosphere golden, heat closure,
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
| `sealed_station` (golden; Godot sealed session; lab `lamp_shed`; the §5 A list, grepped) | yes | **in** | the only build with plants *and* a station heat model |
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
2. **Crop alone (day-neutral, weather temperature kept):** under the 16 h lamp the daylength
   factor is already exactly 1 (16 h ≥ `cpp` 16 h), so in this station the swap removes **only**
   the cold requirement. Control: vernalization off with photoperiod left on must be bit-identical
   to the day-neutral build here. Predicted: earlier maturity than today's crop in the same Dutch
   season. The size of the change is to be measured, not guessed.
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

## 9. Decisions — the user's (ANSWERED 2026-10-01 — see §10)

1. **The temperature form.** A (held constant) now, B (can fail) later on top, C refused.
   *Recommended: A.*
2. **The crop.** A warm room arrests the frozen winter wheat. Options: the existing day-neutral
   crop; a cold treatment in the room's schedule (needs its own source); or stop and keep the
   weather. *Recommended: day-neutral.*
3. **The calendar.** Re-sow when the crop matures, or keep 305 days. *No recommendation until slice
   3 shows how the standing crop behaves.*
4. **The setpoint value.** 22 °C (BVAD cabin nominal, a direct citation for a room) or 23 °C (BVAD's
   crop table, another model's wheat). *Recommended: 22 °C, with 23 °C recorded as contrast.*

## 10. DECIDED 2026-10-01 — the user's answers, and the build they set

The user answered §9 on 2026-10-01: *"1. B 2. add a cold period 3. re sow as soon as the crop
matures 4. 22"*. Every recommendation in §9 except the setpoint was declined, so §5's "A first"
and §8's day-neutral slices are **superseded**.

| # | Decision | Taken |
|---|---|---|
| 1 | Form | **B** — a held room that can fail (a heat store, a capacity-limited exchanger to the node) |
| 2 | Crop | **winter wheat kept**, with a **cold period** in the room's schedule; needs its own source |
| 3 | Calendar | **re-sow on maturity** (the 305-day calendar goes) |
| 4 | Setpoint | **22 °C** (BVAD Table 4-73 cabin nominal) — ⚠ see the note below the conflict |

**A conflict found after the answers, and its resolution (asked, answered 2026-10-01).** The
crew and the plants share one atmosphere (`sealed.rs`: crew respiration draws `O2_POOL` and emits
into `CARBON_POOL`). The cited vernalization window ends at 12 °C (optimum 0–8 °C), and BVAD's
cabin *lower* limit is 18 °C (Table 4-73). A cold period in "the room" would therefore hold the
crew below their cited range for weeks per crop. Options put: a plant chamber; chill seed before
sowing; a cold cabin; drop the cold (day-neutral). **The user chose the plant chamber.** So:

* **"The room" in B is a dedicated plant growth chamber** with its own held temperature. It shares
  the gas pools with the crew cabin (ducted air), as today.
* This is cheap here, checked against the code: the crew reads **no** temperature (`crew.rs` has
  no temperature input; its rates are fixed), and the plants' vapour store is **already** separate
  from the crew's. The station never names `biosphere.water_vapor` anywhere in `station/src`; the
  crew's `WaterBalance` and the ECLSS `Condenser` touch only `eclss.cabin_h2o`, and the sealed
  build's biosphere registry is `build_season` verbatim. The cabin's own temperature stays
  unmodelled, as today.
* So crew body heat does **not** enter the chamber's heat books. It stays out of the books, the gap
  §2 already lists.
* ⚠ **The 22 °C citation weakened with this choice.** §9 recommended 22 °C as BVAD's *cabin*
  nominal, "a direct citation for a room". The room is now a plant chamber, not the crew cabin, so
  Table 4-73 applies to it **by analogy only** — the same standing (a class citation) as Table
  4-111's 23 °C crop-room figure. The value stays 22 °C (the user's choice); the citation's kind is
  what changed.

**The cold period's source — searched 2026-10-01, NOT yet bound.** ⚠ The figures below are **per a
search engine's summary; the pages were NOT opened**, and the summary mixed several results, so
which paper says which figure is unverified (the "4 °C, 16 h, 100–150 µmol" sentence may be the
PMC high-throughput protocol's, PMC12690896, not the preprint's). Opening and pinning each to its
page **blocks slice 3**. Controlled-environment practice as summarized: 2–6 °C for 6–10 weeks (most often 4 °C), plants in a refrigerated cabinet under
16 h light at 100–150 µmol m⁻² s⁻¹ (the speed-vernalisation preprint, bioRxiv
10.1101/2021.12.01.470717, quoting the "normal" treatment); an alternative of 10 °C for 6 weeks
(Zheng et al. 2023, *Plant Breeding*, doi 10.1111/pbr.13074). Rules for binding it:
* **The duration and temperature come from the source, never from the model.** The model's own
  numbers (VDSAT 50 chill-days at 1 per day in 0–8 °C) say 50 days at 4 °C saturates. Choosing 50
  days *because* the model saturates there is calibration in disguise. Model readings at 4 °C, as
  **predictions only**: 6 weeks → 42 chill-days → factor 0.736; 8 weeks → factor 1; 10 weeks → 1.
* ⚠ A recorded mismatch, not a tuning target: at 10 °C the model credits 0.5 chill-day per day, so
  Zheng's 6 weeks at 10 °C gives 21 → factor ≈ 0.04. The model and that paper disagree.
* Locus check owed on the page itself (the preprint is a secondary statement of "normal" practice).
  If no sound source binds, the cold period is **WHAT-IF, lab-only** (`docs/param-file-conventions.md`)
  and the reference build stops at slice 2. The user is told before slice 3 is built either way.

**The cold period's source — pages OPENED 2026-10-03 (user: "yes, alongside").** Read on the
page itself, via the open PMC full text:
* **Dhakal, Sandro & Gutiérrez (2025)**, "Optimized protocol for high-throughput vernalization with
  speed breeding in winter wheat", *Plant Methods*, doi 10.1186/s13007-025-01473-7 (PMC12690896).
  Methods: *"After emergence, the plants were subjected to two vernalizing treatments for four
  weeks."* *"For the normal vernalization (NTV) treatment, the plants were maintained at 4 °C with
  a 16-hour light and 8-hour dark photoperiod"*, at 100–150 µmol m⁻² s⁻¹ (fluorescent). The HTV
  arm: 10 °C, 22 h light, 400–500 µmol. Result: *"four of the twelve genotypes reached heading
  (ZGS 59) earlier in NTV, whereas the other eight genotypes did not significantly differ"*.
  Background: *"A temperature between 0 and 8 °C is considered an optimum vernalization
  temperature for winter wheat"*; *"the effective vernalization period is reported to be between
  30 and 45 days for most genotypes"* — ⚠ that sentence cites Vavilov (1951) [22], a secondary
  locus, not this paper's own measurement.
  **Locus:** a primary, controlled-environment protocol on **growing plants** under a 16 h light —
  the closest match to a plant chamber found. ⚠ Its 12 genotypes are not our cultivar class
  (Soltani & Sinclair's "Wheat / Winter Europe", VDSAT 50).
* **Wu et al. (2025)**, *Int. J. Mol. Sci.* 26:4280, doi 10.3390/ijms26094280 (PMC12072254):
  *"Germinated seeds vernalized at 4 °C … for 4 weeks before sowing"*. ⚠ Locus: a **seed**
  treatment before sowing — the option §10 set aside (chill seed), not a chamber phase. Contrast only.
  ⚠ **Cultivar class, read on the page:** *"twelve hard red winter wheat genotypes"* — US
  cultivars (Freeman, Arapahoe, TAM 113, SD Andes, Warthog …) plus Wisconsin lines; Table S1 (their
  vernalization types) is not in the full text. US Great Plains winter wheats, **not** the model's
  "Winter Europe" class, and no unvernalized control. So 0.274 is a mismatch in **what the paper
  studied**, not a disagreement with it.
* **Cha et al. (2022)**, "Speed vernalization to accelerate generation advance in winter cereal
  crops", *Molecular Plant* 15(8):1300–1309, doi 10.1016/j.molp.2022.06.012 — **opened** as the
  author-accepted manuscript (White Rose eprint 188442, CC-BY-NC-ND; local copy
  `W:\temp\claude\cold-period\cha2022_aam.pdf`, manuscript lines 40–42 and 75–77): *"To date, the
  agronomic and academic standard cereal vernalization protocol entails 6–10 weeks at a low
  temperature of 2–6°C under short-day (8-h-light: 16-h-dark) photoperiod, where the lighting
  conditions are low light intensity"* (citing Luo & He 2020; Xu & Chong 2018; Dixon et al. 2019;
  Kim et al. 2009). Their own barley "regular vernalization" arm: *"6°C for 6 weeks under
  8-h-light:16-h-dark"*; their lab vernalization light: 100 µmol m⁻² s⁻¹.
  **Locus:** a statement of **standard practice** for winter cereals generally (secondary — it
  cites the four), on growing plants. Not cultivar-specific.
* **Still NOT opened** (Wiley HTTP 403): Zheng et al. 2023 (10 °C, 6 weeks); Li et al. 2013,
  *Plant J.* (3 vs 6 weeks at 4 °C, two cultivars). Unverified; not to be bound.

**The rule for choosing, written BEFORE any choice (advisor, 2026-10-03).** The 6–10-week figures
happen to straddle the model's 50-day saturation; choosing them *for that reason* is the
calibration in disguise this note forbids. The criteria, in order: (1) opened on the page;
(2) growing plants in a chamber, not a seed treatment; (3) the cultivar class nearest the model's
"Winter Europe" (a strong, all-or-nothing requirement). Dhakal passes 1–2 and fails 3; Cha passes
1–2 and is class-general on 3; Wu fails 2. Model readings at 4 °C stay predictions only:
Cha's range 6–10 weeks → **factor 0.736–1**.

**⚠ Both opened protocols also fix the LIGHT, and the station's lamp is far off it.** Dhakal:
100–150 µmol m⁻² s⁻¹ under 16 h; Cha: "low light intensity" under **8 h**. The lamp gives
200 W × 2.5 µmol/J over 1 m² ≈ **500 µmol m⁻² s⁻¹ for 16 h**. Taking a protocol's temperature and
duration but not its light is a partial match — and the light matters in a cold chamber
(photosynthesis continues; the lamp's heat is cooling load). A user decision at slice 3: dim the
lamp to the protocol during the cold phase, or keep it and record the mismatch.
* **Model readings at 4 °C, predictions only** (1 chill-day per day; factor `1 − 0.033·(50 − d)`):
  Dhakal's 4 weeks → 28 → **factor 0.274**, held at that value for the rest of the vegetative
  phase once the chamber warms; Vavilov's 30–45 days → 0.34–0.84; 6 weeks → 0.736; 8 weeks → 1.
  ⚠ So the one opened protocol leaves the model's cultivar developing at about a quarter speed —
  a **source/model disagreement about the cultivar**, not a value to tune. Which figure binds is
  the user's call.

**Re-sow on maturity is a hook change, not a driver change.** `sealed_reset_hook` is handed the
current `State`, so it can fire on a state condition (development complete) instead of
`n % season_steps`.

### The slices — one cause per golden diff, predictions committed before each one's code

1. **Lab check, no golden.** Winter wheat in the sealed station with `TEMP_VAR` held at 22 °C from
   sowing, the rest of the sealed build unchanged. Predicted:
   * no chill-day ever accrues (`vernalization_days` stays exactly 0), the factor is exactly 0, so
     `thermal_time` stays exactly 0 and DVS stays 0;
   * so grain stays exactly 0 (the partition table's storage column is 0 before anthesis, and the
     scenario sows with `storage_c0` = 0), while leaf, stem and root **keep growing** — a crop of
     vegetative biomass with no seed;
   * so the first re-sow, at day 305, **hard-fails** with *"seed bank too small to re-sow"*
     (`reset_crop`: grain 0 < seedling 0.16). The check runs one season and asserts that failure
     at the boundary, rather than reading it as a bug.

   This is the control for slice 3 (it shows the cold period, not the warm room, is what lets the
   crop develop).

   **RESULT 2026-10-01 — all three predictions held.** Lab check
   `rust/crates/station/tests/warm_room_arrest.rs` (4 tests, ≈ 40 s; no golden, no `src/` change):
   * chill-days and thermal time are exactly 0 on every day of the warm season;
   * grain is exactly 0 on every day; the vegetative crop reaches **29.91** (leaf + stem + root)
     against the seedling's 0.16 — more than the plain season's 18.64 vegetative, which also filled
     40.05 of grain;
   * a two-season warm run fails at the boundary with *"seed bank too small to re-sow — storage_c
     0.0"*.
   * Control: the plain season vernalizes, develops and fills grain (so the zeros are not a reading
     that is always zero). Liveness: with the room at 8 °C instead, all three warm tests go red.
   * Finding, not a prediction: the sown state (day 0) carries no `vernalization_days` entry at
     all. Cause, read in the code: `build_sealed_station` assembles its own starting entries
     (`thermal_time`, `rooted_depth`) and omits it, while `build_season` and the re-sow both seed
     it to 0. The thermal-time flow reads a missing entry as 0 (`flows.rs`, `unwrap_or(0.0)`), so
     no value moves today. It is the same class of omission as the `rooted_depth` one fixed
     2026-08-12 (the comment at that site), so the original sowing and a re-sowing start with
     different entry sets — relevant to slice 4. Not fixed here: seeding it could change exported
     bytes, which is a golden question, not a lab one. The test reads an absent entry as 0 on day
     0 only.
2. **The chamber heat store (B), the plants NOT yet reading it.** A station unfreeze with the full
   ceremony. Into the chamber: the lamp's waste heat **and** its light leg (re-pointed from
   `boundary.light_used`; only the sugar-fixed part should leave as chemical energy — to be
   priced). Out: a controlled exchanger to `thermal.node`, up to a capacity. New numbers are
   labelled DESIGN, not tuned. Predicted: every biosphere stock bit-identical; the node runs warmer
   (more heat reaches it: the light leg, ≈ 73 W averaged) by a closed-form amount computed before
   code; nominal chamber = setpoint. A perturbation (exchanger or radiator fault) shows the chamber
   drifting — the reason B was chosen over A.
3. **The plants read the chamber, with the cited cold period.** `TEMP_VAR` in `sealed_bio_resolver`
   becomes the chamber's temperature; the setpoint schedule is cold (source's value) for the
   source's duration after each sowing, then 22 °C. One science change: reading the room cannot be
   split from the cold period without arresting the crop (slice 1 is that split, as a lab check).
   The 305-day calendar is **kept** in this slice. Predictions owed, not guessed: condensation —
   saturation at 4 °C is ≈ 0.81 kPa against ≈ 2.64 at 22, so the plants' vapour store condenses on
   entering the cold phase; transpiration — §8 item 3's ×1.96 drier air at 22 °C; carbon — sign not
   predicted; time to maturity ≈ cold period + (1850 − thermal time accrued while cold)/22 days.
4. **Re-sow on maturity.** The hook fires on development complete; each new crop gets its own cold
   period. Rough count: a cycle of ≈ 120–150 days gives ≈ 8–10 crops over the 1220-day horizon
   (measured, not this estimate, is what gets recorded). ⚠ **Risk to measure:** `annual_reset_with`
   hard-errors when the grain is smaller than the next seedling (`system.rs`, "seed bank too small
   to re-sow"). Over ~9 crops in a row, one poor crop ends the run. Whether it does is a prediction
   recorded before the run.

Out of scope still: net radiation (§6, a possible "3d"); crew body heat; the cabin's temperature.
(⚠ The last two move **in** scope for the separate-air option only — §11.)

## 11. Separate air as an option — asked and DECIDED 2026-10-03

The user, on reading §10: *"you said that the crew and the plants share the same chamber - this
may be possible, but i want also to have the option that this is not the case. Also - gas
exchange between chambers (also heat flow with the gas exchange, fans)"*.

**What §10 actually set:** a separate *temperature*, the *air* shared — one CO₂ pool and one O₂ pool
(`sealed.rs`: `CrewRespiration` writes `CARBON_POOL`/`O2_POOL`; the scrubber and O₂ makeup act on
the same two). The ask is a second topology: two air spaces joined by a fan.

**An oddity the split resolves (recorded, NOT fixed under shared air).** The shared-air build
already carries **two** water-vapour stores — the plants' (`biosphere.water_vapor`, 3b) and the
crew's (`eclss.cabin_h2o`) — in what is described as one air. That is physically consistent only
once the air is split.

### Advisor review (2026-10-03), summarized

1. Shared air stays the default and **bit-identical**; separate air is a new option. An option is
   not free: a new flow type and new cabin gas stocks trip the completeness gates
   (`manifest_writer.rs`, `science_gates.rs`), so it is **lab-only first** (the slice-1 /
   lamp-shed pattern) or a full unfreeze.
2. **The fan moves concentration, not mole fraction.** Each species — **including the inert
   fill** — moves at `Q · (n_i/capacity_a − n_i/capacity_b)`. Driven by live mole fractions, a
   pressure difference (a breach on one side) would never move air: the same denominator trap the
   atmosphere build nearly fell into (`docs/log/atmosphere.md`). ⚠ Do not "simplify" this later.
3. **The cabin keeps its 9500 mol; the plant chamber gets its own capacity.** `eclss.yaml`'s
   `o2_setpoint` is an absolute 1995 mol that holds only for a 9500-mol cabin (its own TRIGGER
   note), and the mole-fraction fix is deferred behind an authoring grammar change. Checked
   2026-10-03: the file's other three values are first-order rates (1/s) and a gain, none an
   absolute inventory, so the cabin keeping 9500 mol trips nothing.
4. **The load-bearing risk is the plants' CO₂ supply** (measured below — it is real).
5. The fan's heat is three terms: **sensible** `Q·c_p·(T_cabin − T_chamber)`; the **fan's own
   power** (a battery load that becomes heat in the air — the energy books move); **latent** heat
   in the moving vapour (§2 already lists latent heat as untracked — stays out unless chosen).
   Predict condensation when warm cabin air meets a 4 °C chamber.
6. One cause per change: split the air (gases only, one temperature) first; then the heat stores;
   then the fan's heat.
7. The fan rate needs a source; the chamber volume too. Neither is to be quoted from memory.

### Decisions — the user's (2026-10-03)

| # | Question | Taken |
|---|---|---|
| 1 | How separate air fits | **an option, lab-only first**; shared air stays the default, every golden unchanged |
| 2 | The cabin's temperature | **its own heat store** (crew body heat, equipment, the fan) — the recommendation (held at 22 °C) was **declined** |
| 3 | Order | **air split first** (gases only, one temperature), then heat stores, then the fan's heat |
| 4 | Cold-period source search | **alongside**, now (it still blocks slice 3) |

### Sources searched (BVAD Rev 2, local text extract `W:\temp\claude\bvad\bvad.txt`) — page images NOT yet read

* **No inter-room ventilation rate found.** Table 4-73's "Ventilation" row (printed p. 153) is an
  air **speed** in the cabin, m/s — 0.076 / 0.15 / 0.6096 lower / nominal / upper as the extract
  lays it out — not a volume exchanged between rooms. A locus mismatch for `Q`; usable only as
  contrast. An ISS inter-module ventilation figure is to be searched in NASA ECLSS documents next;
  failing that, `Q` is DESIGN.
* **Table 4-88, "Plant Growth Chamber Equivalent System Mass per Growing Area"** (printed p. 170,
  Drysdale 1999b; the extract's columns are garbled — **page image owed before anything binds**):
  shoot zone **0.67 m³ per m²** of growing area, root zone 0.11; shoot zone **power 0.3 kW/m²**,
  and footnote 183: *"Power consumption and thermal control within the shoot zone reflect fans for
  gas movement."* So a cited (class) figure exists for both the chamber's air volume and the fan's
  power, per m² of crop. Cross-checked against the table's own Total row: volume 0.67 + 0.11
  (root zone) + 0.25 (lamps) = **1.03**, the printed total; power 0.3 + 0.14 + 2.1 + 0.075 ≈
  **2.6**, the printed total — so the extract's column reading is self-consistent (the page image
  is still owed). ⚠ An equivalent-system-mass table's volume is the **allocated** space, hardware
  included, so the free air is **at most** this — the CO₂ finding below is a lower bound on the
  problem.

### Measured / derived — the CO₂ supply problem is real

The sealed station's crop is **1 m²** (`ground_area` default, `system.rs:153`, untouched by
`greenhouse_bio_scenario`). At Table 4-88's 0.67 m³/m², 22 °C and 101.325 kPa (n/V = P/RT =
41.29 mol/m³), the plant chamber holds **27.66 mol of air** — **343× less than the cabin's 9500** —
and, at the cabin's starting CO₂ fraction (3.796/9500), **0.011 mol of CO₂**.

The draw census (`docs/log/draw-census.md`) measured the crop's worst single slow-step CO₂ draw on
the shared cabin air at **0.078 of the pool**. Scaled by 343 at equal concentration — **an
estimate, not a measurement** — the same draw is **≈ 27× the chamber's whole CO₂** in one
(⚠ **SUPERSEDED by §13's measurement: ≈ 6.7×.** The 0.078 was the census's quarter-day-step figure;
at today's 1/16 step the worst ratio is 0.0194.)
1.5-hour step. ⚠ The 0.078 is the draw over the cabin CO₂ **at that worst step** (set by crew and
scrubber), not over the starting 3.796 mol, so the rescale has an unstated denominator: the
conclusion holds (under separate air the chamber sits *below* the cabin, so the ratio is if
anything larger), the number does not. **2a's committed prediction uses absolute moles drawn per
slow step, read from the sealed-station trajectory, over the chamber's CO₂.** The fan would resupply continuously in reality, but the crop is evaluated on the
slow step (1/16 day) while the fan runs on the fast one (60 s), and arbitration counts no
same-step inflow. So a chamber sized by the source would starve by construction of the *step*, not
of the physics. (Option C, built 2026-09-30, makes uptake fall as CO₂ runs low, so it would read as
a stunted crop rather than a hard error — which is worse, being quiet.)

**This is the first thing the air-split slice measures, and a decision it will owe the user** —
not decided here. Directions, unpriced: a larger chamber (DESIGN; a size picked to make the crop
behave is calibration); the crop's gas exchange on the fast step; the fan's resupply folded into
the slow step. Predictions owed before code: the chamber's CO₂ against one slow step's draw, and the
steady chamber-CO₂ deficit ≈ crop draw / `Q`; and the Euler bound `Q·60 s / 27.66 mol ≪ 1` — at
27.66 mol, any `Q` above ≈ 0.46 mol/s (≈ 0.67 m³/min) breaks it, so the same small chamber also
caps the fan.

### The revised slice list (supersedes §10's numbering from slice 2 on)

* **2a — the air split, lab-only.** A topology switch; under "separate", the cabin gets its own CO₂,
  O₂, inert fill (9500 mol capacity) and keeps `eclss.cabin_h2o`; crew, scrubber, makeup and
  condenser re-point to it; the plants keep the biosphere's pools; one exchange flow moves all
  species by concentration difference. One temperature, no heat. Default (shared) bit-identical.
* **2b — the chamber heat store** (§10's slice 2), on the default build: the reference path.
* **2c — the cabin heat store**, separate-air option only (lab): crew body heat (BVAD Table 3-31,
  12.426 MJ/CM-day ≈ 143.8 W), equipment heat, a capacity-limited exchanger to `thermal.node`.
* **2d — the fan's heat**, separate-air option only (lab): sensible + fan power; latent out unless
  chosen.
* **3, 4** as §10 (the cold period, re-sow on maturity).

## 12. The cold period — DECIDED 2026-10-03

The user, on the two slice-3 questions: *"1. use standard 2. dim it"*; then, asked to resolve the
range and the day length: **8 h and dim**, **the middle: 8 weeks at 4 °C**.

| # | Decision | Taken | Source / standing |
|---|---|---|---|
| 1 | Which figure binds | **the standard protocol**, Cha et al. 2022: 6–10 weeks at 2–6 °C | opened (AAM, lines 40–42); a practice statement, class-general |
| 2 | The point in the range | **8 weeks (56 days) at 4 °C** — the centre of both ranges, chosen without the model | the 6- and 10-week ends run as lab sensitivity checks |
| 3 | Light in the cold phase | **dimmed, and the day shortened to 8 h** — the protocol's short day in full | Cha: *"short-day (8-h-light: 16-h-dark) … low light intensity"* |
| 4 | Dim level | **100 µmol m⁻² s⁻¹** — Cha's own vernalization light (AAM line 384), the low end of Dhakal's 100–150 | the standard sentence states no number; this is the same paper's figure |

Model readings, **predictions only**: 56 chill-days at 4 °C → CUMVER ≥ VDSAT 50 → **factor 1**
(the ends: 6 weeks → 0.736, 10 weeks → 1). The 8 h day sits below the photoperiod parameter `cpp`
(16 h), so the daylength factor is **< 1 during the cold phase** — its value from the cited Eqn 7.6
form is to be computed before code, not guessed. The 16 h, full-power lamp returns when the chamber
warms to 22 °C. Consequences to price in slice 3: lamp energy and heat fall in the cold phase
(≈ 100/500 of the light for 8/16 of the hours, ≈ 1/10 of the daily light); photosynthesis falls
with it; the cold phase's thermal time is small at 4 °C. ⚠ The cold phase is now a **schedule on
two things** — the chamber setpoint and the lamp — driven from the same sowing clock, which slice 4
(re-sow on maturity) must restart per crop.

## 13. Slice 2a — the measured baseline and the predictions, committed before code (2026-10-03)

**Instrument:** `rust/crates/station/examples/air_split_baseline.rs` (lab, writes nothing). It runs
the frozen sealed station through the two-rate observer and reads the crop's gross CO₂ withdrawal
per slow step in absolute moles (after option C's end-of-step solve), plus the fast side's books
on the same pool. **Control: the end state is byte-exact against `sealed_station_state.json`**, so
the loop is the reference run. Output kept at `W:\temp\claude\air_split_baseline.txt`.

**Measured (shared air, 1220 days, rationed 0 / 0):**

| Quantity | Value |
|---|---|
| worst single slow step, crop CO₂ draw | **0.073583 mol** from a pool of 3.795788 (ratio **0.0194**), day 1120.75 |
| mean draw over the 14 344 slow steps that drew | **0.036129 mol** |
| crop gross draw, peak day / run mean | **0.774** / **0.425 mol/day** |
| crew CO₂ emission (fast net + scrubbed) | **327.97 mol/day** (= food intake 4e-3 mol/s × 86 400 × the respired fraction) |
| shared CO₂ pool at slow-step starts | **3.79575 – 3.79617 mol** — pinned by the scrubber at P/k |

⚠ **Correction to §11.** The draw census's 0.078 was measured at the **quarter-day** step (it
predates the 1/16 step, 2026-09-30); at 1/16 the worst ratio is 0.0194. So §11's "≈ 27×" rescaled
a stale figure. Measured instead: **0.0736 / 0.01105 = 6.66×** the chamber's CO₂ on the worst step,
**3.27×** on the mean drawing step. The conclusion stands, smaller.

⚠ **Context worth stating plainly:** the 1 m² crop takes **0.13 %** of the crew's CO₂ (0.425 of
328 mol/day). The crop is a token against this crew; nothing about the cabin's air depends on it.

**The setup 2a builds (lab-only):** a `separate` air topology. The cabin gets its own CO₂, O₂ and
inert fill at **9500 mol** capacity and keeps `eclss.cabin_h2o`; the crew, scrubber, O₂ makeup and
condenser re-point to it. The plants keep the biosphere's pools, at a chamber capacity of
**27.66 mol** (BVAD Table 4-88's 0.67 m³/m² × 1 m² × P/RT at 22 °C, 101.325 kPa = 41.29 mol/m³;
an upper bound on free air). One fan flow, on the fast step, moves **every** species by
`Q · (n_i/cap_cabin − n_i/cap_chamber)` — CO₂, O₂, the inert fill and vapour. Both rooms start at
the cabin's composition. One temperature, no heat. `Q` is DESIGN (no source found, §11).

**Predictions (graded after the build):**

1. **The default (shared) build is untouched:** every golden byte-identical; the lab option adds
   no golden and no manifest row.
2. **Conservation holds** on the separate build every step (the fan is internally balanced per
   species), and total moles per room move only by the fan's net, which is zero while both rooms
   sit at reference pressure.
3. **The crop starves by the step at the BVAD chamber size.** Option C caps a slow step's uptake
   below the chamber's CO₂ at the step's start (no same-step inflow is counted), so with the fan
   refilling the chamber to at most the cabin's concentration between slow steps, the crop's
   gross draw is ≤ ≈ (16 h / 1.5 h) × 0.01105 ≈ **0.118 mol/day** plus what soil respiration adds
   inside the chamber — against **0.425** mean and **0.774** peak on shared air. Predicted: crop
   gross CO₂ draw falls to **well under half** of shared air, and the crop ends lighter. How much
   lighter is not predicted.
4. **The physics, not the step, would have fed it.** A continuous crop at the worst-step rate
   (1.36e-5 mol/s) keeps the chamber at ≥ half the cabin's CO₂ concentration if
   `Q ≥ 2U/x_cabin` = **0.068 mol/s** (≈ 0.10 m³/min). The fan's own Euler limit on the 60 s step
   is `Q·dt/27.66 < 1` → **Q < 0.461 mol/s** (≈ 0.67 m³/min). A window exists, so prediction 3's
   starvation is the slow step's, not a fan too weak to keep up. The build runs inside the window.
5. **Control — a big chamber recovers shared air.** At a chamber capacity of 9500 mol (the cabin's
   own size) with a fast fan, the crop's draw returns to within a few percent of shared air.
   ("A few percent" is a bound to be graded, not a value to fit.)
6. **The cabin barely notices:** its CO₂ stays within ±0.01 % of 3.796 mol in every case,
   because the crop is 0.13 % of the crew's CO₂.

**The decision this will owe the user** (after the build, with prediction 3 graded): how the
plants' gas exchange meets the fan — crop gas exchange on the fast step; the fan's resupply
inside the slow step; or a larger chamber (DESIGN). Not decided here.

### 13a. Water — measured, and predictions added before code (advisor, 2026-10-03)

The advisor's point: the 343× shrink that starves CO₂ also hits vapour, and the fan's vapour leg
opens a new water path that conservation cannot see (a *redistribution*). The instrument was
extended (same file, same byte-exact control) and re-run:

| Quantity (shared air, 1220 days) | Value |
|---|---|
| the shared room's relative humidity at slow-step starts | **≈ 0.75 on nearly every step** (10-day means 0.71–0.76; day 0 0.54) — pinned at the humidity target already |
| worst one-step transpiration / a 27.66-mol chamber's humidity target | **0.4219 kg / 0.006876 kg = 61×**; mean over 19 520 transpiring steps **28×** |
| the cabin's vapour per mol of air vs a chamber at its target | **7.1e-6 vs 0.8e-4 – 3.3e-4 kg/mol**; cabin wetter on **0 of 19 520** steps |

Read in the code: sealed transpiration is **bounded at the source** — `to_air = min(flux,
headroom)`, the rest straight to `biosphere.condensate` (`flows.rs`, `Transpiration::evaluate`);
condensate recycles to the soil (`Recycling`). So a small room cannot overflow and cannot error.

⚠ **A finding about the cabin, recorded, not acted on:** the cabin's air sits at **≈ 1.5 %
relative humidity** (7.1e-6 kg/mol against ≈ 4.7e-4 saturated at 22 °C). It is `eclss.yaml`'s
DESIGN `condense_rate` (τ ≈ 2000 s, chosen for solver stability, "not calibrated") holding the
crew's vapour at `P/k`. BVAD's cabin range is far wetter. Shared air never showed it, because the
crew's vapour store and the plants' were never compared; the split compares them.

**Predictions — water (W) and the added controls:**

* **W1 — fan vapour leg OFF, small chamber:** the chamber sits at its humidity target, as the
  shared room already does, so the deficit the crop reads is the same and **transpiration per unit
  canopy is unchanged**; only the split moves — `to_air` falls to ≈ the small room's headroom, the
  rest goes to condensate. Water stress stays exactly 1 (3b's finding) unless the canopy changes.
* **W2 — direction, vapour leg ON:** the fan moves water **one way only, chamber → cabin, in every
  season** (the cabin is drier on every measured step; the direction cannot flip while the cabin's
  condenser holds it at 1.5 %). The path: chamber vapour → `eclss.cabin_h2o` → ECLSS condenser →
  recovered water → the crew's water store. **Nothing returns it to the plants** — the sealed build
  irrigates only from its own recycled condensate.
* **W3 — size, vapour leg ON:** the fan's refill time `cap/Q` (≈ 138 s at Q = 0.2 mol/s) is ≪ the
  slow step (5400 s), so each slow step starts with the chamber's vapour near the **cabin's** level,
  ≈ 1.5 % RH. The crop then reads nearly the **full** saturation deficit (vs 25 % of it at the
  target) — **potential transpiration up ≈ 3–4×** (Penman–Monteith's deficit term; not exact —
  the radiation term does not scale). The water exported to the crew is bounded per slow step by
  the small room's headroom: **≈ 16 × 0.002–0.010 kg ≈ 0.04–0.16 kg/day**, i.e. **≈ 10–50 kg over a
  305-day season** out of a plant loop holding ≈ 195 kg of soil water (19.5 root zone + 175.5
  below). Whether the soil reaches water stress inside a season is **not predicted** — measured.
* **W4 — control:** vapour leg OFF vs ON separates the water effect from the CO₂ effect. Prediction
  3 (CO₂ starvation) is graded on the **vapour-OFF** run, so a lighter crop is not double-caused.
  Each run records per step whether transpiration's source bound binds and the water-stress factor.

**The bounds made falsifiable:**

* Prediction 5's "within a few percent" is **5 %**: at chamber capacity 9500 mol with a fast fan,
  the crop's season gross CO₂ draw is within **±5 %** of shared air's.
* **Fan-rate control (new):** at Q = **0.1** and **0.4** mol/s (both inside the 0.068–0.461
  window; refill times ≈ 277 s and ≈ 69 s, both ≪ 5400 s) the crop's season gross CO₂ draw differs
  by **< 5 %**. Near-equal draw is what shows the starvation is the slow step's, not the fan's.
* Prediction 1 is graded by `regen_goldens` (report-only, every golden unchanged), the full
  `cargo test --no-fail-fast` and `cargo clippy --all-targets -- -D warnings` — not by the sealed
  golden alone (the Godot session and session-parity tests call the same builder).

**Build discipline (advisor):** the fast flow list is **extracted** from `build_sealed_station`
(one copy, the public signature kept, the shared build calling it with `CARBON_POOL`/`O2_POOL`),
never duplicated. The fan is a lab flow in `station/src`, registered in nothing frozen — the
`SheddingLamp` precedent; the full suite confirms the gates do not flag it.

### 13b. Slice 2a BUILT (lab only) — the predictions graded (2026-10-03)

**What landed.** `rust/crates/station/src/air_split.rs` (LAB-ONLY: the cabin's own CO₂ / O₂ /
inert fill, the `AirExchange` fan, `build_split_station`); `sealed.rs`'s fast flow list
**extracted** into `sealed_fast_flows` (one copy; the shared build calls it with the biosphere's
pools); the season instrument `rust/crates/station/examples/air_split.rs` (output
`W:\temp\claude\air_split_run1.txt`); lab pins `rust/crates/station/tests/air_split.rs` (90-day
runs of the same claims). Fan `Q` = 0.2 mol/s (DESIGN, inside the window).

**One season (305 days), every case rationed 0 / 0:**

| Case | Crop gross CO₂ (mol) | Plant C at end | Chamber RH | Transpired (kg) | To the air | Exported to cabin |
|---|---|---|---|---|---|---|
| shared | 129.556 | 58.689 | 0.7481 | 711.25 | 284.70 | — |
| split, Q 0.2, vapour off | 22.519 (**0.174**) | 8.429 (**0.144**) | 0.7502 | 708.21 (0.996) | 0.83 | — |
| split, Q 0.1, vapour off | 22.519 | 8.429 | 0.7502 | 708.21 | 0.83 | — |
| split, Q 0.4, vapour off | 22.519 | 8.429 | 0.7502 | 708.21 | 0.83 | — |
| split, Q 0.2, vapour **on** | 22.519 | 8.429 | **0.0333** | 1704.54 (**2.40×**) | 23.33 | **23.30 kg** (78 fast steps ran back) |
| big chamber 9500 mol, Q 10 | 129.489 (**0.999**) | 58.654 (0.999) | 0.7481 | 711.25 | 284.70 | — |

**Graded:**

| # | Prediction | Result |
|---|---|---|
| 1 | default build untouched | **HELD** — `regen_goldens`: 20 of 20 identical; `cargo test --no-fail-fast` 1252 passed, 1 red that was **not** this build (my own MEMORY.md hook past the 240 B per-line gate — shortened, gate green); `clippy --all-targets -D warnings` clean |
| 2a | conservation every step | **HELD** — the driver asserts it per sub-step; every run completed |
| 2b | each room's total moves only by the fan's net, zero at reference pressure | ✗ **FALSIFIED as stated** (graded after the advisor's review, `examples/air_split.rs` run 2, `W:\temp\claude\air_split_run2.txt`): the rooms do not sit at reference — the chamber's total gas reaches **+1.96 %** over its reference air (the vapour, which the dry inert charge leaves out; the **shared** room reads the same 1.96e-2, so this predates the split), the cabin drifts **≤ 2.5e-4** (scrubber, makeup, crew). The fan carried **0 mol** of inert gas all season: both rooms start at the same inert concentration and nothing else moves inert, so the fan has nothing to even out. It evens out a pressure difference only once one room's inert changes (a breach) |
| 3 | crop starves by the step, well under half | **HELD** — 0.174 of shared CO₂ (0.074 mol/day, under the ≈ 0.118 ceiling); plant carbon 0.144 |
| 4 | a fan window exists; starvation is the step's | **HELD** via the fan-rate control |
| fan-rate | Q 0.1 vs 0.4 within 5 % | **HELD** — equal to 5 figures (22.5186 / 22.5187) |
| 5 | big chamber within ±5 % of shared | **HELD** — 0.999 |
| 6 | cabin CO₂ within ±0.01 % of 3.796 | ✗ **FALSIFIED** — 3.7907–3.8012 (±0.14 %). The bound came from the shared pool read at **slow-step** starts (3.79575–3.79617); read at every fast step the shared pool itself swings 3.731–3.833 (±1.7 %). The split cabin is **steadier** than the shared pool, not as steady as claimed |
| W1 | vapour off: same humidity, same transpiration | **HELD** — RH 0.7502 vs 0.7481; transpiration 0.996; `to_air` 0.83 vs 284.7 kg. ⚠ The promised per-step water-stress record was **not built** (the instrument records where the source bound binds, not the stress factor); W4's identical plant carbon is the stronger evidence and stands in for it |
| W2 | one way only, chamber → cabin, every season | ✗ **FALSIFIED in the absolute** — chamber → cabin on all but **78** of 439 200 fan steps; the seasonal direction held |
| W3 | export ≈ 10–50 kg/season, ≈ 0.04–0.16 kg/day; transpiration up ≈ 3–4× | export **HELD** (23.30 kg, 0.076 kg/day); transpiration ✗ **MISSED LOW** — **2.40×** |
| W4 | vapour on/off attributes the effects | **HELD** — plant carbon identical (8.429) on and off: within a season the drain does not touch the crop, so prediction 3 is CO₂ alone |

**Tally: 9 held, 4 failed** (2b, 6, W2 in the absolute, W3's transpiration).

**Liveness of the lab pins (mutations, `--no-fail-fast`, each restored and `cmp`-checked):**
vapour never crossing → the drain pin goes red; the crew breathing the chamber's pools (no
separation) → the starvation pin goes red; the chamber left at the cabin's size → **no growth pin
went red** (its crop starves either way, from a low concentration instead of the step), so a fifth
pin was added, `each_room_is_charged_at_its_own_size`, which that mutation turns red.

**Findings, recorded:**

* ⚠ **Transpiration does not follow the canopy.** A crop with 0.144 of the carbon transpires 0.996
  as much. Penman–Monteith here uses fixed resistances per ground area, so transpiration is set by
  the weather and the room, not by leaf area. It was invisible while every run grew a full crop.
* **The water drain is exact bookkeeping:** plant water 195.000 → 171.705 kg (−23.295, the fan's
  export); the crew's store +20.965, the rest in the cabin vapour, recovery buffer and brine.
* The cabin's ≈ 1.5 % relative humidity (§13a) is what drives the drain: the dehumidifier's DESIGN
  rate, now visible.

**Decisions owed to the user (not taken):**

1. **How the plants' gas exchange meets the fan** — the starvation is the slow step's. A chamber
   stops starving only above ≈ **184 mol ≈ 4.5 m³ per m² of crop** (the worst step's 0.0736 mol at
   the cabin's CO₂ concentration) — 6.7× BVAD's 0.67 m³, so the unread page image cannot overturn
   this. Each remedy's price: crop gas exchange on the fast step changes the **frozen biosphere
   step**; folding the fan's resupply into the crop's end-of-step CO₂ solve couples a biosphere
   flow to a station flow; a larger chamber is a DESIGN size picked so the crop behaves —
   calibration, unless a source sets it.
2. **A water return path** — with vapour crossing, the plants' water ends up in the crew's store
   and nothing brings it back (a real station would irrigate from it).
3. **The cabin's humidity** — `eclss.yaml`'s DESIGN condenser rate holds the cabin at ≈ 1.5 % RH,
   far below BVAD's cabin range.

## 14. The three decisions TAKEN (user, 2026-10-03) — and the order

The user, on §13b's three: *"Calculate the plants' gas exchange minute by minute. This changes
the frozen plant science — do it"*; *"A way to get water back to the plants, for example watering
them from the crew's supply, as a real station would — do it"*; *"The crew cabin's humidity: about
1.5 % today, far below the handbook's range for a cabin — fix it."*

**Order (advisor, 2026-10-03): humidity → gas exchange → watering, three batches, three commits.**
Watering is sized against the drain the other two leave, so it goes last. Humidity first: small,
cited, and its golden diff is predictable. The gas exchange regenerates the same station goldens
again, so it is kept in a commit of its own so each diff has one cause.

## 15. The cabin's humidity — a station unfreeze (predictions committed before code)

**Source, read as the page image** (`W:\temp\claude\bvad\p63-077.png`): BVAD Rev 2 **Table 4-1,
"Typical Steady-State Values for Vehicle Atmospheres", printed p. 63**, row *Relative Humidity, %*:
**25** (lower, ref. 9 = NASA Std. 3001 Vol 2 Rev A 2015) / **40** (nominal, ref. 10 = *Typical
ISS*) / **75** (upper, ref. 9). The text extract scrambles this table's columns (its temperature
row lands on the wrong line); the image is the authority.

**What is wrong today.** `Condenser` removes `k·h2o` — first-order on the WHOLE cabin vapour — so
the cabin settles at `P/k` = **0.0675 kg** in every station golden (P = 3.375e-5 kg/s of crew
humidity, k = 5e-4 /s). That is ≈ 1.5 % RH of a 9500-mol cabin at 22 °C. It is not a humidity
anyone chose: it is the crew's output divided by a solver-stability rate.

**The form.** A dehumidifier that acts only above its setpoint:
`R = k · max(0, h2o − h2o_setpoint)`. Same first-order rate `k` (still DESIGN, unchanged), now on
the EXCESS. The steady state becomes `h2o_eq = h2o_setpoint + P/k`. One-sided because a condensing
heat exchanger cannot humidify. ⚠ The biosphere's `science::condensed_vapour_kg` is NOT reused: it
draws first-order *below* its target too (its own recorded scope line), so it would leave the cabin
at `P/k` — exactly the bug.

**The value.** `humidity_setpoint` = **1.7863 kg**, an absolute inventory derived the way
`o2_setpoint` was: 0.40 × saturation at **22 °C** (295.15 K, BVAD Table 4-73's cabin nominal, the
temperature this note already decided) × the cabin's **9500 mol** × M_H2O, using the model's own
FAO-56 curve (`weather::saturation_vapor_pressure`, 2643.93 Pa → 4.46579 kg saturated). A test
pins the derivation. ⚠ **TRIGGER**, as on `o2_setpoint`: the value encodes the cabin's air
(9500 mol) and a temperature the cabin does not hold as a state. When the cabin gets its own heat
store (§11, the user's choice), the setpoint must become a relative humidity that reads it.

**Starting vapour: every scenario starts AT the setpoint** (`cabin_h2o_0` 0 → 1.7863), the
`cabin_o2_0` rule ("every scenario that sets this must move WITH the setpoint").

**Predictions (graded after the build):**

* **H1 — the shift.** With `e = h2o − setpoint`, the new law is `ė = P − k·e`, `e(0) = 0`: exactly
  the OLD law in `h2o`. So the condenser's flux is the old one up to the rounding of `h2o −
  setpoint`, and in every golden **`eclss.cabin_h2o` rises by the setpoint** (0.0675 → ≈ 1.8538 kg;
  the standalone `eclss` golden 0.04 → ≈ 1.8263) while **condensate, `crew.water_store`,
  `eclss.recovered_water` and `boundary.brine` move only in the last few bits** (relative change
  ≤ 1e-12). Not bit-identical: the subtraction rounds.
* **H2 — the plants do not notice.** In shared air the crop reads its own `biosphere.water_vapor`,
  never `cabin_h2o`, so **every carbon, O₂, nitrogen and biosphere-water value is byte-identical**
  in `greenhouse`, `harvest`, `sealed_station` (and `lighting` does not build a cabin condenser at
  all — to be confirmed by its golden not moving).
* **H3 — which goldens move.** Exactly the six that hold `eclss.cabin_h2o`: `cabin_gas`, `eclss`,
  `greenhouse`, `harvest`, `sealed_station`, `water_recovery`. `sealed_energy_drift_summary` and
  every biosphere golden: unchanged. No rationing, no events.
* **H4 — the manifest.** `eclss.yaml`'s hash and the six `golden_sha256` rows change; no flow id,
  no flow type, no seam. The new param is a new row in the param set. The authoring flow registry's
  condenser entry keeps `rate_params: [condense_rate]` (the setpoint is a target, not a rate) and is
  not `demand_controlled` (one-sided: it cannot reverse).
* **H5 — the split build (lab).** With the cabin at 40 % instead of 1.5 %, the fan's vapour
  difference shrinks, so the season's drain chamber → cabin falls well below 23.3 kg — but stays
  positive, because the chamber's own target is 75 % (> 40 %). How far it falls is measured.

**Recorded, not fixed:** shared air now holds two humidities in one room — the crew's vapour stock
at ≈ 41 % and the plants' at their 75 % target. They were two stocks before; now the mismatch is
visible in the numbers, not just in the wiring.

### 15a. The cabin's humidity BUILT — the predictions graded (2026-10-03)

**What landed.** `eclss.yaml` gains `humidity_setpoint` = 1.7863 kg (BVAD Table 4-1, 40 %
nominal; derivation pinned by `the_humidity_setpoint_is_forty_percent_of_the_cabins_saturation`);
`Condenser` draws `k_cond·max(0, cabin_h2o − humidity_setpoint)`; every `cabin_h2o_0` and every
authored fixture's cabin vapour start at the setpoint (`eclss_cabin.yaml`,
`eclss_multirate_cabin.yaml`, `eclss_thermal_habitat.yaml`, `scenarios/bioregenerative_station.yaml`);
the authoring `eclss` param set carries the key. Six goldens regenerated, the station manifest
rewritten, unfreeze logged in `docs/station-reference.md` and `docs/authoring-reference.md`.

| # | Prediction | Result |
|---|---|---|
| H1 | `cabin_h2o` +setpoint; the crew's water books move only in the last bits (≤ 1e-12) | **HELD** — `cabin_h2o` 0.0675 → 1.8538 (`eclss` 0.04 → 1.8263); condensate, `water_store`, `recovered_water`, `brine` ≤ **3.4e-14** relative; `sealed_station`'s `water_store` byte-identical |
| H2 | every plant-side value byte-identical | **HELD** — no carbon, O₂, N or biosphere-water byte moved in any golden |
| H3 | exactly six goldens move | **HELD** — `cabin_gas`, `eclss`, `water_recovery`, `greenhouse`, `harvest`, `sealed_station`; `lighting` and the drift summary unchanged |
| H4 | `eclss.yaml` hash + six `golden_sha256`; the new param a new row | **HALF** — the hash and six rows moved, nothing else; ✗ there is **no param-name row** to move: the station manifest records param files by hash only, so a new param shows up as a file digest and nothing more |
| H5 | split build: the drain shrinks but stays chamber → cabin | ✗ **FALSIFIED — it reversed.** Over a season (`W:\temp\claude\air_split_run4.txt`) the plants GAIN 2.08 kg (195.000 → 197.083) and the crew's store loses 1.88 kg; the chamber's mean RH rises 0.75 → **0.91**; transpiration falls to 0.933 of shared. The chamber runs at the weather's temperature (≈ 16 °C mean), so at 75 % it holds less vapour per mol than a 40 %, 22 °C cabin (1.95e-4 kg/mol) |

**Tally: 3 held, 1 half, 1 failed.**

**Tests that changed, and why each is an unfreeze rather than a weakening:**

* `condense_flux_is_first_order` → `…_in_the_excess`; new `condense_flux_is_zero_at_and_below_the_setpoint`
  (the one-sided half) and a loader refusal of a non-positive setpoint.
* `eclss_run.rs`: the steady state is `setpoint + P/k`; the integral invariant and the
  monotone rise now start from the setpoint (same claims, new start), and assert the start IS
  the setpoint.
* Authoring's unsafe-step demonstration re-measured **38 → 28** firings: the condenser's law
  alone removed the ten (changing it with the fixture untouched already gave 28). At `k·h = 1.8`
  it overshoots below the setpoint and stops instead of emptying the cabin.
* The lab pin `vapour_crossing_drains_the_plants_water_into_the_crews` became
  `…_moves_water_between_the_plants_and_the_crew`, asserting the measured direction (cabin →
  plants over 90 days: +1.39 kg) with a note that a 22 °C chamber should flip it back.
* **New:** `the_multirate_eclss_anchor_runs_its_condenser`. That fixture's teeth are its condenser,
  and a dry start would have left it idle all day (the crew adds ≈ 1.73 kg < 1.7863): a dead
  anchor with nothing red. Its Python run-match died with S6, so nothing ran its trajectory at all.
  Measured 2.8387e-2 kg above the setpoint, the header's "~2.84e-02", now pinned.

**⚠ The finding the fix makes visible — shared air (advisor's correction of my first reading).**
The shared room holds TWO vapour stocks, each held by its own condenser, and their targets ADD:
the plants' 75 % of the weather's saturation plus the crew's 40 % of 22 °C's. Even at one common
22 °C that is ≈ 116 % of saturation. Measured over one season, both counted against saturation
at the plants' temperature: above 1 on **4 876 of 4 880** plant steps, max **2.88×**; before the
fix (the shift makes the old crew vapour exactly `cabin_h2o − setpoint`, from the same run):
**8** steps, max **1.22×**. So the frozen 2026-09-23 note "can exceed saturation by at most ~3 %"
was already wrong, by 22 %. ⚠ The shared room was never at 1.5 %: it was ≈ 76 % (the plants' 75
plus the crew's 1.5) — only the crew's own stock read 1.5 %. **Nothing reads the sum** (searched:
the condenser, the crew water balance, the lab fan, the builders, the Godot palette's crew-only
`cabin_gas` session — no reader adds the two, no display shows a room humidity, the breach readers
do not touch `cabin_h2o`), so no simulated value changes. The fix is right as built where the cabin
has its own air (`cabin_gas`, `water_recovery`, standalone ECLSS, the lab split). **The user's
call:** keep two stocks (recorded flaw), or give shared air one vapour stock held by one condenser —
a separate station change that moves either the plants' air (drier, more transpiration) or where
the crew's water ends up (in the plants' soil).

## 16. The plants' gas exchange minute by minute (predictions committed before code, 2026-10-03)

**The form (advisor, 2026-10-03).** The crop's three carbon-budget flows — `Allocation`,
`GrowthRespiration`, `MaintenanceRespiration` — move **together and unchanged** from the plant
step (1/16 day) onto the cabin's 60 s step. They share one gross assimilation by design, so none
moves alone. No new carbon stock: a sugar buffer between the two steps would be a new mechanism
with no source, and would turn the sealed build's dropped CO₂ round trip into real gross fluxes.
An adapter, `station::gas_exchange::OnFastStep`, wraps each flow:

* **the forcing window** — after the plant step `state.n` has already advanced, so the fast steps
  that follow it lie in window `n − 1`; the adapter evaluates every forcing at `(n − 1, plant dt)`.
  Getting this wrong would shift the light by 90 minutes, silently. **Light keeps the plant
  step's resolution** (the window's mean PAR, every minute of it): this change moves the CO₂
  timing and nothing else. Resolving light per minute is a different change, not taken.
* **the step unit** — the inner flow is handed `dt = 60 / 86400` days;
* **shared reads** — the CO₂ pool and soil water are read live from the minute's snapshot, so
  `Allocation`'s bit-equality guard (pool variable vs the stock it draws) still holds;
* **no double count** — the three are removed from the plant registry (`Registry::into_parts`);
* **die-off and the safety net** — `substep` runs arbitration and the extinction pass exactly as
  `step_report` does (read in `simcore::integrator`); it skips only the aux and `n`, which these
  flows never write.
* It refuses a day whose fast steps do not follow ONE plant step each (`steps_per_day` must be a
  multiple of `bio_steps_per_day`): the lamp scenarios' grouping of 2 plant steps per 3 power
  hours would make "the window this fast step lies in" ambiguous.

⚠ Two reads shift by one plant window: development stage (thermal time) and the stress factors
are read after the plant step that opens the window, not before it. 90 minutes of thermal time.

**Rollout.** A builder option, measured lab-only on the separate-air build and on shared air,
then adopted into the reference shared build as its OWN commit (the user authorized the unfreeze).
Cost measured before adopting.

**Predictions (graded after the build):**

* **G1 — shared air barely moves.** Same light, a 9500-mol room: over one season the crop's gross
  CO₂ draw and its end carbon are within **±1 %** of the plant-step run (the step studies measured
  ≈ 0.1 % on harvest from 1/16 to 1/64 once light is held, and light is held here).
* **G2 — the separate-air crop stops starving.** At the BVAD chamber (27.66 mol) and Q = 0.2 mol/s,
  the season's crop CO₂ draw rises from **0.174** of shared to **between 0.75 and 0.98** of it. The
  basis: at steady state the chamber sits `U/Q` below the cabin's concentration; at the shared
  run's peak draw (0.774 mol/day ≈ 9e-6 mol/s) that is 4.5e-5 mol/mol under the cabin's 4.0e-4,
  i.e. the crop sees ≈ 89 % of the cabin's CO₂ at peak and more the rest of the season.
* **G3 — the fan rate now MATTERS** (the signature of the plant step having been the limit): draw
  at Q 0.1 < Q 0.2 < Q 0.4, and **Q 0.1 / Q 0.4 ≤ 0.97**. If they are still equal to five figures,
  the plant step is still the limit and the build is wrong.
* **G4 — the controls hold:** the 9500-mol chamber with Q = 10 stays within ±5 % of shared air
  (per-minute on both); vapour off vs on still leaves plant carbon unchanged within a season.
* **G5 — books:** every run conserves every step, rations 0 / 0, raises no events.
* **G6 — the window pin:** a test holds the adapter's PAR on the minute after a plant step equal to
  that plant step's own PAR, on a step where the lamp's window changes (so `n` vs `n − 1` differ).
* **G7 — cost:** measured, not predicted. A season of the split build at the plant step took
  ≈ 15 s (release). Reported before adoption; if it threatens the suite's run time it goes to the
  user.

**Scope, stated:** transpiration stays on the plant step. Moving it too would send most of the
crop's 700–1700 kg of water a season through the fan — the next question, not this one.

### 16a. Gas exchange on the minute step BUILT (lab option) — the predictions graded (2026-10-03)

**What landed.** `rust/crates/station/src/gas_exchange.rs` (`OnFastStep`, `PlantWindow`,
`split_carbon_budget`, `gas_exchange_on_fast_step`, `require_one_plant_step_per_group`);
`AirSplit.gas_exchange` selects the step on the separate-air build; the season instrument
`examples/air_split.rs` counts the crop's CO₂ draw on whichever side it is taken (output
`W:\temp\claude\air_split_run5.txt`); pins in `tests/gas_exchange.rs` (the three traps) and
`tests/air_split.rs` (90-day season claims). Nothing in the reference calls it yet.

**One season (305 days), "of shared" = the frozen plant-step shared run:**

| Case | Crop gross CO₂ | Plant C at end | cabin CO₂ range |
|---|---|---|---|
| shared, plant step (frozen) | 129.556 (1.000) | 58.689 (1.000) | 3.731–3.833 |
| shared, minute | 130.456 (**1.007**) | 59.114 (**1.007**) | 3.782–3.813 |
| split 27.66 mol, Q 0.1, minute | 111.861 (0.863) | 49.484 (0.843) | — |
| split 27.66 mol, Q 0.2, minute | 119.238 (**0.920**) | 53.263 (**0.908**) | — |
| split 27.66 mol, Q 0.4, minute | 122.964 (0.949) | 55.184 (0.940) | — |
| split Q 0.2, minute, vapour on | 119.238 (0.920) | 53.263 (0.908) | — |
| big 9500 mol, Q 10, minute | 130.325 (1.006) | 59.047 (1.006) | — |
| (split Q 0.2, plant step, for contrast) | 22.519 (0.174) | 8.429 (0.144) | — |

| # | Prediction | Result |
|---|---|---|
| G1 | shared air within ±1 % | **HELD** — +0.7 % draw and plant carbon (up, the direction a finer step on growth gives). The shared pool's minute-to-minute swing narrows to a third (3.731–3.833 → 3.782–3.813): the crop's draw is now spread through the 90 minutes instead of landing at once |
| G2 | split draw 0.75–0.98 of shared | **HELD** — 0.920 (plant carbon 0.908) |
| G3 | fan rate matters; Q0.1/Q0.4 ≤ 0.97 | **HELD** — 0.863 < 0.920 < 0.949; ratio **0.910** (plant carbon 0.843 / 0.940 = 0.897) |
| G4 | big chamber within ±5 %; vapour on/off leaves plant C unchanged | **HELD** — 0.999 of minute-shared; 53.263 on and off |
| G5 | conservation, 0 / 0 rationing, no events | **HELD** for conservation and rationing on every season run; events are asserted empty by the 90-day pins (the season instrument does not count them) |
| G6 | the window pin | **HELD** — `the_fast_step_reads_the_light_of_the_plant_window_it_lies_in`, plus the flow-level pin. Mutations, `--no-fail-fast`, each restored and `cmp`-checked: reading window `n` → 2 red; `dt` left in seconds → 1 red; the three left on the plant step too → 1 red |
| G7 | cost | a season with the observer (which re-evaluates the three each minute): **≈ 26 s vs ≈ 18 s** on the plant step. The reference's own cost is measured at adoption (§17) |

**Tally: 7 held.** At 90 days (the pins): Q 0.1 / 0.2 / 0.4 → 0.892 / 0.939 / 0.963 of shared air's
plant carbon; the 9500-mol chamber 1.008.

### 16b. The adapter REDESIGNED before adoption — the plant step records its window (2026-10-03)

**Found while listing adoption's callers:** the first adapter copied a plant-side resolver in at
BUILD time. Every run that changes the plant's inputs after building — the perturbation suite's
forcing wrappers, `warm_room_arrest`'s held 22 °C, the lab's lamp shedding (`rewire_for_shedding`
wraps every plant-step flow and aux, and nothing on the fast side) — would have changed phenology
and water and left photosynthesis on the unchanged copy. Conserving perfectly, no red.

**Advisor (2026-10-03), taken:** the plant step RECORDS the two forcings the carbon budget reads
(PAR, temperature) for its own window, from its own environment, into the state's aux
(`station::gas_exchange::PlantWindowRecorder`, keys `station.plant_window.par` / `.temp`); the
minute step reads them back and errors on any other read. The window is then right by
construction (the `n − 1` read is gone), and whatever changes the plant step's environment is
what gets recorded — `LampLitAux` scales PAR for aux reads (read in `lamp_shed.rs`), so shedding
reaches photosynthesis too. ⚠ The aux channel is additive, so the record is `old + (X − old)`:
measured **exact on all 96** of three days' window values.

**Checks:** `examples/air_split.rs` re-run (`W:\temp\claude\air_split_run6.txt`): **identical to
§16a's table in every printed digit**. New pins: the plant-side light switched off ⇒ the crop
fixes nothing on the minute step; a plant-side 22 °C is what is recorded. Mutations,
`--no-fail-fast`, each restored and `cmp`-checked: the recorder reading a build-time copy → 2 red
(the light and temperature pins); `dt` in seconds → 2 red; no recorder → 4 red; the budget left
on the plant step too → 2 red. Full suite 1270 passed, 0 failed, 339 s wall.

## 17. ADOPTION into the reference — a station unfreeze (predictions committed before code, 2026-10-03)

**What changes.** `build_sealed_station` and `build_greenhouse` (with plants) take the crop's gas
exchange on the minute step. Both gain an `_at(…, GasExchangeStep)` form; the plain names mean
`Minute`. So after this the reference has **two forms**, by where the crop's air is:
the minute step wherever the cabin's flows act on the crop's air (`greenhouse`, `harvest`,
`sealed_station`, and the Godot `greenhouse` / `sealed` sessions, which call the same builders);
the plant step for the lit-room `lighting` scenario (24 power hours cannot pair one-to-one with
16 plant steps — refused, not approximated) and for every standalone biosphere scenario (no fast
step exists). `TwoRate::validate` refuses the retired slow-first day on a minute build (every
minute would read the day's last window). Lab instruments that reproduce plant-step records
(`day_order`'s slow-first tests, `draw_census`, `intraday_exchange`, `air_split_baseline`, and
`build_split_station`'s own base) are pointed at `PlantStep` builds.

**Predictions (graded after the build):**

* **A1 — which goldens move:** exactly `greenhouse`, `harvest`, `sealed_station`. Unchanged:
  `lighting`, `sealed_energy_drift_summary` (energy books do not read the crop), and every other
  station and biosphere golden.
* **A2 — what moves inside them:** carbon, O₂ and nitrogen stocks and the crew-side gas
  boundaries (`co2_removed`, `o2_supply`). **Every water stock and every pre-existing aux value is
  byte-identical** — transpiration does not read the canopy (§13b's finding), and nothing in the
  water loop, phenology or root depth reads carbon. The aux block gains two keys,
  `station.plant_window.par` / `.temp`.
* **A3 — how much:** `sealed_station` (4 seasons): grain and total plant carbon within **±1.5 %**;
  `greenhouse` / `harvest` (7-day seedlings): plant carbon within **±1 %**.
* **A4 — books:** 0 rationing, 0 events in every golden; the tier contract tests pass.
* **A5 — the manifest:** the three `golden_sha256` rows; the station `flow_set` gains
  `Allocation`, `GrowthRespiration`, `MaintenanceRespiration` (the sealed FAST registry now holds
  them). No aux set moves: the recorder sits in the plant registry, which the station manifest
  excludes — so `PlantWindowRecorder` is a station-side aux type that **no freeze record lists**.
  Stated, not hidden. Two prose claims become false and are corrected: `freeze_manifest.rs`'s
  "the slow registry is excluded so no biosphere flow leaks in", and `station-reference.md`'s
  "biosphere delegated, `build_season` verbatim" for these three scenarios.
* **A6 — cost:** the `sealed_station` golden 1 m 57 s → **≤ 2 m 45 s**; the full suite ≤ 1.3 × its
  339 s.
* **A7 — lab pins with numbers:** pinned values in `warm_room_arrest`, `lamp_shed` and
  `perturbations` may move; each red is read, explained and re-measured, never loosened.

### 17a. ADOPTED — the predictions graded (2026-10-03)

| # | Prediction | Result |
|---|---|---|
| A1 | exactly `greenhouse`, `harvest`, `sealed_station` move | **HELD** — `regen_goldens`: 3 of 20 changed |
| A2 | water stocks and pre-existing aux byte-identical; two aux keys added | **HELD** — no water byte, no `thermal_time` / `vernalization_days` / `rooted_depth` byte moved in any of the three; `station.plant_window.par` / `.temp` appear |
| A3 | `sealed_station` ±1.5 %, `greenhouse` / `harvest` ±1 % | **HELD after a fix** — sealed plant carbon **+0.75 %**, grain **+0.79 %**; greenhouse **+0.29 %**; harvest **+0.07 %**. ✗ On the first run harvest read **−17.6 %**, its grain and stem reserve at 0: `build_harvest` reused the greenhouse's plant registry and DISCARDED its fast one to rebuild it with `Harvest` — taking the moved carbon budget with it, so the crop grew nothing (and conserved). The prediction is what caught it. Fixed (a plant-step base, the minute step applied onto the rebuilt registry); new pin `every_crop_build_steps_the_carbon_budget_exactly_once`, which the re-introduced bug turns red |
| A4 | 0 rationing, 0 events; tier contract passes | **HELD** |
| A5 | 3 `golden_sha256` rows; `flow_set` + the three types; no aux set change | **HELD** exactly (manifest diff: 3 types, 3 rows, nothing else) |
| A6 | sealed golden ≤ 2 m 45 s; suite ≤ 1.3 × 339 s | **HELD** — all 20 goldens regenerate in **120 s**; the full suite **354 s** (1.04×) |
| A7 | lab pins with numbers may move | two reds, both in `tests/perturbations.rs`: "regulator erasure" (`ΔC ≈ 0` to 1e-6 at the day's end) read 1.3e-6 and 3.6e-6. Explained: a crop exchanging every minute shifts a regulated pool by its own flux over the regulator's rate, `ΔS/k`; the old claim held only because a 90-minute pulse had been erased by the day's end. Restated with the offset taken out and the SAME 1e-6 bound: residuals **1.8e-9, 6.0e-10 (CO₂), 6.3e-14 (O₂)**. `warm_room_arrest` and `lamp_shed` stayed green |

**Tally: 7 of 7 held, one only after the bug it found was fixed.**

**The reference now has two forms, by where the crop's air is** (§17): minute-step gas exchange
in `greenhouse`, `harvest`, `sealed_station` (and the Godot sessions built from them);
plant-step in `lighting` and every standalone biosphere scenario.

`examples/air_split_baseline.rs`, re-pointed at a plant-step build with `run_sealed` as its
control (byte-exact), reproduces §13's CO₂ figures exactly (0.073583 / 0.036129 / 0.773765 /
0.424778). Its water line now reads the cabin wetter than a target-held 27.66-mol chamber on
**11 968 of 19 520** plant steps (§13a: 0) — the humidity fix, the same reversal as §15a's H5.
Output `W:\temp\claude\air_split_baseline_run2.txt`.

## 18. Watering the plants from the crew's supply (lab, separate air; predictions before code, 2026-10-03)

**Why it is lab-only.** In shared air the plants' water loop is closed by construction (their
vapour stock and condensate never meet the crew's); only the separate-air build moves water
between the two, through the fan.

**What changed since §13b asked for it.** The cabin now holds 40 % RH (§15). With the plant
chamber still at the WEATHER's temperature the flow REVERSED: over a season the plants gain
2.08 kg from the cabin (§15a, H5). A drain returns once the chamber is held at 22 °C — the
decided setpoint (§10), not yet built as a room with heat books; here it is a forcing override on
the plant side, as `warm_room_arrest` does. At one 22 °C the chamber's 75 % target holds
3.5e-4 kg of vapour per mol of air against the cabin's 1.95e-4, so the fan (Q = 0.2 mol/s) can
carry up to ≈ 3.2e-5 kg/s ≈ **2.7 kg/day** out of a chamber held at its target — more than a
season's soil water (195 kg) over 305 days.

**The form — no new science.** The open field's own `Irrigation` flow ([F] Eqn 14.8:
demand-driven, `min(capacity · A · dt, max(0, TTSW − ATSW))`), on the plant step, with its
source re-pointed from the weather's boundary at the crew's `crew.water_store`. Capacity: the
scenario's `irrigation_mm_day` = 8 mm/day (8 kg/day on 1 m²). Id `station.watering`; an
`AirSplit.watering` switch. It only tops the ROOT ZONE up: water already below the roots is the
subsoil's, as in the field.

**Predictions (one season, 305 days, Q = 0.2, vapour crossing, minute gas exchange):**

* **R1 — the weather-temperature chamber never needs it:** watering delivers **0 kg** (the root
  zone stays at capacity while the cabin feeds the chamber); every other number is §16a's.
* **R2 — a 22 °C chamber without watering drains:** the plants lose **≥ 50 kg** to the cabin over
  the season, and the root zone reaches water stress at some point (`f_water < 1`), so the crop
  ends lighter than R3.
* **R3 — a 22 °C chamber with watering:** the root zone is held near capacity; watering delivers
  within **±10 %** of what the fan exports; the plants' water ends within **±2 %** of where it
  started; plant carbon ≥ R2's.
* **R4 — the crew pays the recovery loss, not the water:** the exported water reaches the crew's
  store through the condenser and the recovery processor (`recovery_efficiency` 0.9), so with
  watering the crew's store falls by ≈ **10 % of the water cycled**, not by all of it.
* **R5 — books:** conservation every step, 0 / 0 rationing, no events.

### 18a. Watering BUILT as predicted — the predictions graded, and mostly FAILED (2026-10-03)

`AirSplit.watering` adds the field's `Irrigation` (source `crew.water_store`, 8 mm/day cap) to
the plant step; instrument `rust/crates/station/examples/watering.rs`, output
`W:\temp\claude\watering_run1.txt`. One season, separate air, Q 0.2, vapour crossing, minute
gas exchange:

| Chamber | Watering | Watered (kg) | Fan export (kg) | Plants' water 195.000 → | soil / subsoil at end | Crew store Δ | Brine Δ | Water stress | Plant C |
|---|---|---|---|---|---|---|---|---|---|
| weather | off | 0 | −2.083 | 197.083 | 159.2 / 28.3 | −133.74 | +131.54 | never | 53.263 |
| weather | on | **43.256** | −2.083 | **240.339** | 169.0 / 61.8 | −177.00 | +131.54 | never | 53.263 |
| 22 °C | off | 0 | **21.248** | 173.752 | 133.3 / 25.9 | −112.74 | +133.87 | never | 28.314 |
| 22 °C | on | **51.178** | 21.248 | **224.930** | 169.1 / 41.2 | −163.92 | +133.87 | never | 28.314 |

| # | Prediction | Result |
|---|---|---|
| R1 | weather chamber: watering delivers 0 | ✗ **FAILED** — 43.3 kg. The field rule refills the root zone to capacity every plant step its own transpiration dips it, AHEAD of the chamber's recycled condensate; the condensate then arrives on a full zone and drains below it. So watering pre-empts the plants' own loop and adds the crew's water to it: the plants' water ends **+45 kg** |
| R2 | 22 °C, no watering: drain ≥ 50 kg and water stress | ✗ **FAILED** — 21.2 kg and no stress. The drain is capped by the PLANT STEP again: transpiration (still on it) puts at most the chamber's headroom into the air each 90 minutes and condenses the rest inside, and with the fan holding the chamber near the cabin's level that headroom is ≈ (3.5e-4 − 1.95e-4) kg/mol × 27.66 mol ≈ 0.0043 kg a step × 16 × 305 ≈ 21 kg. The same step artefact as the CO₂ starvation, on the water side |
| R3 | 22 °C + watering: delivered ±10 % of export, plants' water ±2 % | ✗ **FAILED** — delivered 51.2 vs 21.2 exported; the plants' water ends **+15 %** (R1's mechanism) |
| R4 | crew store falls ≈ 10 % of cycled water | ✗ **FAILED** — the crew paid all 51.2 kg; brine is identical with and without watering, because the extra water stayed in the plants' loop and never cycled back |
| R5 | books | **HELD** — 0 / 0 rationing, no events, every run |

**Tally: 1 held, 4 failed.** The 22 °C crop's lower carbon is the warm room's arrest (§9, slice 1),
not water: no run reached water stress.

**What the failures say.** (1) A field's top-up rule is the wrong controller for a closed room whose
own condensate already waters the crop: it fills ahead of the loop and inflates it. (2) The drain the
watering was asked to answer is itself bounded by the plant step's transpiration-into-air rule — the
next question §16 named (transpiration on the minute step).

### 18b. Watering REDESIGNED: on a depletion trigger (advisor, 2026-10-03; predictions before code)

**The source of WHEN.** FAO-56 (Allen et al. 1998, *Crop evapotranspiration*, FAO Irrigation and
Drainage Paper 56), **Table 22**, "Ranges of maximum effective rooting depth (Zr), and soil water
depletion fraction for no stress (p), for common crops" (fetched from fao.org, chapter 8,
2026-10-03): spring wheat **p = 0.55**, winter wheat **p = 0.55**; *"The fraction of TAW that a
crop can extract from the root zone without suffering water stress is the readily available soil
water: RAW = p TAW."* Taken as the irrigation trigger: water when depletion reaches RAW, i.e.
when the root zone's fill `FTSW = ATSW / TTSW` falls below **1 − p = 0.45**. ⚠ Locus: TAW is
FAO's field-capacity-to-wilting-point store and `TTSW` is [F]'s transpirable store — the same
quantity by definition, read here as a class citation. The check the advisor set: 0.45 sits
**above** the model crop's own stress onset (`wssg` 0.30), so it waters before the crop suffers.
**HOW MUCH** stays [F] Eqn 14.8's refill (`min(capacity · A · dt, TTSW − ATSW)`). Without a latch,
a refill capped at 0.5 kg per plant step stops as soon as FTSW is back over 0.45, so the zone is
HELD at the trigger rather than refilled to the top — stated, not hidden.

**Measured before predicting** (`W:\temp\claude\watering_run2.txt`, the §18a runs with the root
zone's lowest fill added): without watering the zone never falls below **0.8184** (weather chamber)
or **0.6623** (22 °C).

**Predictions:**

* **T1 — no season run needs it:** in all four runs watering delivers **exactly 0 kg**, and each
  watering-on run's end state is **byte-identical** to its watering-off run.
* **T2 — it fires when the zone IS low** (pin a): a root zone started below the trigger is watered
  from `crew.water_store` (the store falls by exactly what the soil gains), the step after the zone
  is back over 0.45 it stops.
* **T3 — above the trigger it gives exactly 0** (pin b).
* **T4 — liveness:** the always-refill rule (§18a's) turns pin b red; an inverted comparison turns
  pin a red.

### 18c. Triggered watering BUILT (lab, off by default) — graded (2026-10-03)

`station::air_split::TriggeredWatering` gates the field's refill (`Irrigation`, crew-sourced) by
FAO-56's trigger, `FTSW < 1 − 0.55`; `FAO56_WHEAT_DEPLETION_FRACTION` carries the citation. Output
`W:\temp\claude\watering_run3.txt`.

| # | Prediction | Result |
|---|---|---|
| T1 | 0 kg in all four seasons; on == off byte for byte | **HELD** — 0.000 kg in every run; weather and 22 °C end states byte-identical on vs off |
| T2 | fires below the trigger, store → soil kg for kg, stops once over 0.45 | **HELD** — `watering_fires_below_the_trigger_and_stops_above_it` (from 0.30 it waters several steps, each leg pair equal and opposite, and stops over 0.45) |
| T3 | exactly nothing above the trigger | **HELD** — no legs at 0.46, 0.66, 1.0 |
| T4 | always-refill reddens pin b; inverted reddens pin a | **HELD, stronger** — each mutation turns BOTH pins red |

**Tally: 4 held.** So watering now does what the user asked of it — water goes back to the plants
from the crew's supply when, and only when, their root zone runs down — and in this build as it
stands it never has to: the most the season drains is 21 kg, against a root zone that never falls
below 66 % full. **That 21 kg is the plant step's**, not the room's (§18a, R2): transpiration
still puts at most the chamber's headroom into its air each 90 minutes. Moving transpiration to the
minute step is the next decision, and the user's.

### 17b. Follow-up checks after adoption (advisor, 2026-10-03)

* **Lamp shedding reaches photosynthesis — by a run, not only by reading.** `tests/lamp_shed.rs`
  builds with `build_sealed_station` (minute step since §17) and stayed green:
  `a_blackout_sheds_the_lamp_and_the_crop_feels_it` and
  `cutting_only_the_lamps_power_darkens_the_lab_crop` both assert the crop grows LESS. On the
  minute build the recorded window PAR is the only light the carbon budget reads, so those pins
  now cover the recorder's lamp-lit path.
* **`examples/draw_census.rs` re-run after adoption** (`W:\temp\claude\draw_census_after_adoption.txt`):
  control 1 byte-exact on all 18 goldens it runs, the three adopted ones included. ⚠ Its control 4
  reports FAILED — *"lab_leaf_jar: control 4 case does not ration — it proves nothing"* — and so
  does the pre-session commit `5661ded` (`W:\temp\claude\draw_census_pre_session.txt`): the lab
  leaf jar stopped rationing at the 1/16 step (2026-09-30), so the control's chosen case went
  stale then. Not caused here; recorded, not fixed.
* **Recompiled, NOT re-run:** `examples/intraday_exchange.rs` and `examples/lamp_shed.rs`. Their
  recorded numbers date from plant-step builds (`intraday_exchange` is now pinned to them).
* `docs/station-reference.md`'s "the `aux_set` is empty — the station carries no non-conserved
  accumulator" was false after §17 (it carries `PlantWindowRecorder`); reworded to the real reason.

### 17c. §17b's stale checks cleared — and a fourth they hid (2026-10-04)

Outputs in `W:\temp\claude\cleanup-2026-10-04\`. ⚠ **Cause, stated:** only item 3's breakage
came from the adoption (§17). Controls 2 and 4 of the census went stale at the 1/16 step and
option C (2026-09-30); `lamp_shed` was un-run, not stale. A sweep of `station/examples/` for the
slow-first day finds `intraday_exchange` the only one, its builders now all plant-step.

**1. `examples/draw_census.rs` control 4 re-pointed.** Its rationing case is now the one
`tests/leaf_form.rs` already uses for the same job: the frozen jar with its room shrunk to a tenth
(air, CO₂ and O₂ scaled), in the EXPLICIT CO₂ form (`Co2Read::StartOfStep`) — under the
reference's option C it does not ration, so it could prove nothing. One constant names the case
for both the roster and the verdict, and a full (unfiltered) run that does not reach it fails.
* **Two probes agree:** the census reads **205 firings** (probe 205 = integrator 205) and a
  tightest CO₂ step of **1.488053** of the pool (step 3348, `allocation`) — the figures
  `leaf_form.rs` measured with `readouts::step_draws`, a separate probe. Control 3 holds on every
  step (`draw_census_squeezed.txt`).
* **It can fail** (each mutation run, then restored and `cmp`-checked): the case under option C →
  *"does not ration — it proves nothing"*; the probe's count off by one → *"probe firings 206 vs
  integrator 205 (⚠ DISAGREE)"*; the case's roster key renamed, full run → *"control 4 case is
  not in the roster"* (`draw_census_breakC.txt`).
* **A FOURTH stale check, found by the full run: control 2.** The tool carries its own copy of the
  jar's pinned tightest CO₂ step, and it still held the quarter-day figure (0.756662 at step 777).
  The gate it copies (`science_gates::margins::JAR_CO2_STEP_DRAW`) was re-pinned for the 1/16 step
  and again for option C on 2026-09-30, to **0.165230 at step 3108**. Every run since printed
  *"⚠ NOT reproduced"* — and the verdict still read "all held", because control 2 was printed but
  never counted (the pre-session and after-adoption outputs both show it). Fixed: the copy carries
  the gate's figure, and control 2 now joins the verdict (a full run that does not reach the jar
  fails too). Checked: reproduced on the jar; with the old figure put back → *"FAILED — sealed_jar:
  control 2"*. Two copies, one stale — and a printed check nobody counts is not a check.
* **Full run** (`draw_census_full2.txt`): **controls: all held** — control 1 byte-exact on all 20
  runs it checks, control 2 reproduced, control 3 bit for bit, control 4 agrees everywhere and the
  squeezed jar rations. `cargo clippy --all-targets -D warnings` clean; `cargo test --no-fail-fast`
  **1275 passed, 0 failed** (the harvest golden among them, so `build_harvest_at` left the
  reference path byte-identical).

**2. `examples/lamp_shed.rs` re-run on the minute build** — predictions written before reading
(`lamp_shed_predictions.md`), **6 of 6 held**:

| run | crop (mol C), plant step (§7 of the lamp plan) | minute step | rationed | battery day 8 (J) |
|---|---|---|---|---|
| plain, calm | 0.323944 | 0.325813 (+0.58 %) | 0 | 5.7840e7 |
| plain, blackout | 0.323944 | 0.325813 — **= calm, bit for bit** | 5140 | 1.8144e4 |
| lab, calm | 0.323944 | 0.325813 | 0 | 5.7840e7 |
| lab, blackout | 0.221738 (−31.6 %) | 0.222449 (**−31.7 %**) | 0 | 1.8219e7 |

The lamp is still shed at day 4.625 (the group delivering 1/6 of nominal) and never comes back on.
The plain build still cannot feel the blackout: the minute-step gas exchange reads the recorded
window light, which the plain lamp never darkens; and since the crop ends bit-identical to the
calm run, none of the backstop's 5140 firings can have scaled its carbon budget (inferred from
that bit-identity, not counted per flow).

**3. `examples/intraday_exchange.rs` was BROKEN, not merely un-run.** It panicked in its harvest
section: *"the slow-first day cannot run a minute-step gas exchange"*. §17b's "pinned to the plant
step" was true of its greenhouse and sealed builds, but adoption had hard-wired
`station::harvest::build_harvest` to the minute step with no plant-step variant. Fixed the way
the other two builders already are: `build_harvest_at(…, GasExchangeStep)`, with `build_harvest`
calling it with `Minute` (the reference path unchanged; the harvest golden re-checked byte for
byte by the suite). Re-run, exit 0, 5 min 23 s (`intraday_exchange_rerun.txt`):
* control 1 (full 4-year sealed station, interleaved == the reference runner, bit for bit): **true**;
* every station run rations 0 / 0 with no events, in both day orders.
* ⚠ **It does NOT reproduce its record's numbers, and this is no longer expected to.** The record
  (`docs/log/intraday-gas-exchange.md`, 2026-09-29) was made at the quarter-day step; since then the
  1/16 step, option C, Step 3b's sealed transpiration and the 40 % cabin humidity all moved the
  plant-step tree it runs. Finding 4's SHAPE holds (harvest flow carries the grain half, feces→litter
  the soil half, independently); its sizes moved: grain store +0.375 → **+0.483**, microbes +0.121 →
  **+0.151**, humus +0.191 → **+0.237** (both seams on, 7 days). The instrument is a plant-step tool
  on today's science, not a check of the minute build.

## 19. Two decisions TAKEN (user, 2026-10-03)

1. **Shared air's two moisture stocks: KEEP two, record the flaw.** The crew's `cabin_h2o` (held
   at 40 %) and the plants' `water_vapor` (held at 75 %) stay separate stocks in the one shared
   room; counted together they read above saturation (§15a). Nothing reads the sum; it stays a
   documented flaw of shared air, not fixed.
2. **Leaf water loss (transpiration) on the minute step: LAB FIRST, then decide.** Build it on the
   separate-air lab build, measure, and bring the main-build decision back to the user — the path
   the gas exchange took (§16 → §17).

## 20. Leaf water loss on the minute step — lab first (predictions committed before code, 2026-10-05)

**What changes (lab only).** On the separate-air build, the plants' `Transpiration` and the plant
chamber's `Condensation` move **together and unchanged** from the plant step onto the cabin's 60 s
step — a new `AirSplit.transpiration` switch, default the plant step. Why the pair: transpiration's
split between air and condensate counts the condenser's same-step draw
(`headroom = target − v + condensed`), so it is only consistent when the two share a step.
`Recycling` (condensate → soil), root-zone capture, drainage and the watering trigger stay on the
plant step. The reference is not touched.

**Reads.** `Transpiration` reads temperature, net radiation, the chamber's own vapour (`VpdRead::Chamber`,
checked in `params.rs`; the weather's deficit is never read), soil water and the rooted depth;
`Condensation` reads temperature. Temperature is already recorded by the plant step
(`PlantWindowRecorder`). Net radiation is recorded by a **second, lab-only recorder** on its own
key, `station.plant_window.net_radiation` — not added to the reference recorder, which would put a
new aux key into three goldens. The minute step refuses any other read. Stocks and aux are read live.

**Refusals, at build time.** Water on the minute step with gas exchange on the plant step (the
temperature record would not exist); a plant registry missing either flow (they move together or
not at all).

**Found while reading, recorded, not changed.** The sealed crop's net radiation is the weather
file's **outdoor daily value**, not the lamp's (`weather_forcings`; only PAR is replaced by the
lamp in `sealed_bio_resolver`). Daily temperature and net radiation are per-day tables, so a
window's recorded value is exact for every minute in it. ⚠ *Superseded 2026-10-05 (§21):* net
radiation is now the lamp's per-window value. That is still constant within a window, which is
all the recorder needs, but it is no longer a per-day table.

**Measured before predicting** (`W:\temp\claude\minute-transpiration\crossover.txt`, a throwaway
tool, deleted). The cabin holds 1.8803e-4 kg of vapour per mol of air. A chamber at its 75 % target
holds more than that above **≈ 12 °C**: the weather is above it on **127 of 305** days. Held AT its
target every minute, the fan (Q 0.2) would carry **113.8 kg out** on those days and **157.7 kg in**
on the others; at 22 °C, **867.1 kg out**. Penman–Monteith at the target: 708.0 kg a season
(weather), 1012.9 kg (22 °C).
⚠ **The fan sees only 70 % of that gradient on the minute step.** It exchanges `Q·dt = 12` mol a
minute of a 27.66-mol chamber (`k = 0.434`). All flows read the minute's start, so after
transpiration refills to the target the fan has already drawn on the start value. The chamber
settles at `(target − cabin-equivalent)/(1 + k)` above the cabin's level, and the fan moves
**1/(1 + k) = 0.697** of the full-gradient figure. This is explicit Euler meeting a fast fan, and the
CO₂ exchange has it already. Stated, not changed.

**Predictions (one season, BVAD chamber, Q 0.2, vapour crossing, minute gas exchange; "the
plant-step run" = §18c's same case):**

* **M1 — the weather chamber now GAINS, by tens of kg.** The fan's net vapour flow is cabin →
  plants by **20–45 kg** (basis 0.697 × (113.8 − 157.7) = −30.6; the plant-step run: 2.08 kg).
  Watering delivers 0. Plant carbon within ±1 % of 53.263.
* **M2 — the 22 °C chamber without watering DRAINS and STRESSES** (retests §18a's R2). The plants
  lose **≥ 100 kg** of their 195 kg (the plant-step run: 21.2). The crop is water-stressed
  (`f_water < 1`) on **≥ 20 %** of the season's minutes (plant-step run: never). Plant carbon below
  28.314 by **more than 1 %**.
* **M3 — with watering on, watering does the work it was built for.** It delivers **300–650 kg**
  (§18c: 0); the fan exports **400–650 kg** (0.697 × 867.1 = 604, less where transpiration falls
  short); the root zone's lowest fill stays **≥ 0.40** (held at the 0.45 trigger, since the 8 kg/day
  cap is far above a ≈ 2 kg/day drain); **no minute** is stressed. Plant carbon within ±2 % of
  28.314.
* **M4 — the crew pays the recovery loss.** Exported vapour reaches the crew's store through the
  cabin condenser and the recovery processor (wired in `sealed_fast_flows`). So brine rises over
  the plant-step run's 133.873 kg by **0.10 × (export − 21.248) ± 10 %**.
* **M5 — season transpiration** (counted on whichever step runs it): weather chamber **650–900 kg**;
  22 °C with watering **1000–1500 kg** (the chamber sits below its target, so the air is drier
  than the 1012.9 basis).
* **M6 — control, vapour not crossing:** the plants' water stays 195.000 kg to 1e-9. Season
  transpiration on the minute step is within **±2 %** of the plant-step run's, and plant carbon
  within ±0.5 %. Total transpired is not capped by the air's headroom (the overflow goes straight to
  condensate), so only its timing changes.
* **M7 — books:** conservation every step, 0 / 0 rationing, no events, in every run.
* **M8 — the reference is untouched:** `regen_goldens` reports 0 of 20 changed; full
  `cargo test --no-fail-fast` and clippy green.
* **M9 — pins turn red when broken** (mutations `--no-fail-fast`, each restored and `cmp`-checked):
  * the minute after a plant step reads that step's net radiation, at a day boundary where it
    changes;
  * `dt` left in seconds turns a pin red;
  * the pair left on the plant step too (double count) turns a pin red;
  * no net-radiation recorder is an error;
  * a plant-side 22 °C reaches minute-step condensation;
  * the refused combinations error.
* **M10 — cost:** measured. Expected within 1.3× of the ≈ 26 s minute-gas-exchange season.

### 20a. Built (lab) — graded, on the OUTDOOR-radiation tree (2026-10-05)

**What landed.** `station::air_split::{water_on_fast_step, NetRadiationRecorder, WATER_LOSS_FLOWS,
WATER_LOSS_WINDOW}` and the `AirSplit.transpiration` switch; `OnFastStep::reading` (the window's
variable list is now a field; `OnFastStep::new` keeps the carbon budget's list, so the reference
path is unchanged). Pins in `rust/crates/station/tests/minute_transpiration.rs`. Instrument
`rust/crates/station/examples/minute_transpiration.rs`, output
`W:\temp\claude\minute-transpiration\run1.txt`.

⚠ **Every number below was measured with the crop's net radiation taken from the OUTDOOR weather
file** (the §20 finding). The user ruled that wrong the same day (§21), so this table records
the move's behaviour on the old input. It is not a forecast of the fixed tree. After §21, §20 is
re-predicted from scratch, not re-graded.

| Case (one season) | Watered | Fan → cabin | Transpired | Plants' water 195 → | Lowest root-zone fill | Stressed minutes | Plant C | Brine Δ |
|---|---|---|---|---|---|---|---|---|
| weather, plant step (§18c control) | 0 | −2.083 | 663.6 | 197.083 | 0.8184 | 0 % | 53.263 | +131.540 |
| weather, minute, watering off | 0 | **−55.117** | 671.8 | 250.117 | 0.6490 | 0 % | 53.263 | +126.234 |
| weather, minute, watering on | 0 | −55.117 | 671.8 | 250.117 | 0.6490 | 0 % | 53.263 | +126.234 |
| 22 °C, plant step (§18c control) | 0 | +21.248 | 1735.1 | 173.753 | 0.6623 | 0 % | 28.314 | +133.873 |
| 22 °C, minute, watering off | 0 | **+169.085** | 257.8 | **25.915** | 0.0001 | **81.4 %** | **0.058** | +148.657 |
| 22 °C, minute, watering on | **477.000** | **+562.168** | 1225.1 | 109.832 | 0.4489 | 0 % | 28.314 | +187.959 |
| weather, plant step, no vapour crossing | 0 | 0 | 708.21 | 195.0000 | 0.8492 | 0 % | 53.263 | +131.748 |
| weather, minute, no vapour crossing | 0 | 0 | 708.03 | 195.0000 | 0.8507 | 0 % | 53.263 | +131.748 |

Both controls reproduce §18c's figures to every printed digit. Every run rationed 0 / 0 with no
events.

| # | Prediction | Result |
|---|---|---|
| M1 | weather chamber gains 20–45 kg; watering 0; plant C ±1 % | ✗ **FAILED on size** — gains **55.1 kg** (direction held; watering 0 and plant C 53.263 held). Candidate cause, an ESTIMATE and not measured: the basis took the cabin at exactly its 40 % setting, but the cabin's proportional condenser holds it above that by crew output ÷ rate = 2e-5 / 5e-4 = 0.04 kg (+2.2 %). Over the season at the 0.697 coupling that is ≈ −16 kg, giving ≈ −47 kg; ≈ 8 kg is left unexplained |
| M2 | 22 °C, no watering: loses ≥ 100 kg, stressed ≥ 20 % of minutes, plant C below 28.314 by > 1 % | **HELD, harder than predicted** — loses 169.1 kg, stressed 81.4 %, and the crop is **dead of drought** (0.058 mol C). §18a's failed R2 was the plant step's cap, as §18a said |
| M3 | watering 300–650 kg, export 400–650 kg, fill ≥ 0.40, no stressed minute, plant C ±2 % | **HELD** — 477.0 / 562.2 kg, 0.4489, none, 28.314 (to every printed digit) |
| M4 | brine +0.10 × (export − 21.248) ± 10 % over 133.873 | **HELD** — predicted +54.09, measured +54.086 |
| M5 | transpiration 650–900 (weather), 1000–1500 (22 °C, watering) | **HELD** — 671.8, 1225.1 |
| M6 | no vapour crossing: water 195.000, transpiration ±2 %, plant C ±0.5 % | **HELD** — 195.0000, −0.03 %, 53.263 |
| M7 | books | **HELD** — 0 / 0, no events, every run |
| M8 | reference untouched | **HELD** — `regen_goldens`: **20 of 20 identical**; clippy clean; `cargo test --release --no-fail-fast` **1280 passed, 0 failed**, 4 ignored (1275 + the 5 new pins) |
| M9 | pins turn red when broken | **HELD** — five mutations, each run `--no-fail-fast` on the pin file, restored and `cmp`-checked: `dt` in seconds → 2 red; no net-radiation recorder → 4 red; the recorder also writing temperature (the additive double) → 2 red; the refusal of plant-step gas exchange removed → 1 red; only transpiration moved, its count check off → 4 red |
| M10 | cost | **not cleanly measured** — runs took 25–36 s against 23 s for the plant-step control, but a test build ran alongside. Open |

Found on the way: at 22 °C the plant-step build transpired **1735 kg**, the minute build 1225 kg.
On the plant step, the fan pulls the chamber toward the cabin's dryness for 90 minutes between
transpiration steps. Transpiration then reads that drier air.

## 21. The lamp-lit crop's net radiation comes from the LAMP — a station unfreeze (user, 2026-10-05; predictions before code)

**The defect (found in §20, ruled on by the user the same day: "fix this, in its current state, it
doesnt make sense").** In every lamp-lit build the crop's PAR is the lamp's, but transpiration's
net radiation was still the weather file's outdoor daily value. A sealed crop lost more water on
sunny outdoor days, and a lamp blackout left its water loss untouched.

**The form — no new science, no new parameter.** Net radiation while lit = `(1 − α) ×` the lamp's
radiant flux at the crop, and 0 while dark:
* the radiant flux is the window's lamp PAR × `PAR_PHOTON_ENERGY_J_PER_UMOL` (McCree's 1/4.57 J per
  µmol, the same conversion the lamp's own energy split uses, so all the lamp's radiant output is
  PAR, as in its energy books);
* `α` is FAO-56's 0.23, the SAME constant the outdoor form `weather::net_radiation` uses. **The
  user's choice (2026-10-05)**, over holding the fix for a PAR-specific canopy reflectance.
  ⚠ Locus: 0.23 is broadband sunlight on a grass reference. Leaves reflect less of an all-PAR
  lamp's light (no PAR-specific value could be read on a page we can reach: Penning de Vries 1989
  and Goudriaan & van Laar 1994 are not open; the open review Liu et al. 2021, *Plant Physiol.*
  186:977, says only "very low reflectance"). So this probably UNDER-counts the absorbed energy by
  up to ≈ 20 %. Recorded, replaceable.
* It follows the lamp's own top-hat path (`lamp_light_path`), so it is per window and it follows
  any dimming (the decided cold-phase 100 µmol).
* Still left out, as outdoors: net long-wave, the lamp's waste heat (it goes to the thermal node),
  and canopy size (transpiration ignores leaf area, §13b).

**Where.** `sealed_bio_resolver` (the sealed station, its Godot session, the palette) and
`lighting_bio_resolver`. The two lab paths that darken the lamp must darken this too:
`lamp_shed`'s `LampLitEnv` (scales PAR by the delivered share → scales net radiation too) and
`perturbations::with_lighting_failure` (PAR → 0 in the window → net radiation → 0 too). The
sunlit builds (`greenhouse`, `harvest`, every biosphere scenario) keep the weather's value.
`authoring` has no lamp.

**Measured before predicting** (`W:\temp\claude\minute-transpiration\zz_scratch_lamp_rn.rs`,
throwaway): lamp PAR 500 µmol m⁻² s⁻¹ → 109.41 W m⁻² radiant → **84.25 W m⁻² while lit**, 56.16
as a daily mean (16 h). The outdoor daily mean over a season is 88.00. Penman–Monteith summed at the
chamber's 75 % target: one season 708.0 → 557.2 kg (**× 0.787**); the lighting scenario's 7 days
15.8 → 14.6 kg (**× 0.924**). The sealed station today never stresses its crop: lowest root-zone
fill **0.7805** over the 4-season golden (stress onset 0.30).

**Predictions:**
* **L1 — which goldens move:** exactly `sealed_station_state` and `lighting_state` (2 of 20).
  Unchanged: `greenhouse`, `harvest`, `sealed_energy_drift_summary`, all 7 biosphere goldens, the rest.
* **L2 — what moves inside them: water only.** Soil, subsoil, condensate and vapour stocks move.
  Every carbon, O₂ and nitrogen stock is byte-identical, as are the crew's and the energy stocks and
  every aux value (`thermal_time`, `vernalization_days`, `rooted_depth`,
  `station.plant_window.*`). The basis: no step reaches water stress (fill ≥ 0.78 with the higher
  outdoor draw), and nothing on the carbon side reads water except through the stress factor
  (allocation, leaf expansion, drought-accelerated development), which stays exactly 1. The precedent
  is Step 3b's chamber dryness: water only.
* **L3 — size:** the sealed station's transpiration over its golden's horizon falls by **21 % ± 5
  points**, the lighting scenario's by **8 % ± 4 points**. Measured by an observer run on the two
  builds before and after.
* **L4 — books:** 0 rationing, no events, the tier-contract tests pass after the regeneration.
* **L5 — the manifest:** exactly the two `golden_sha256` rows. `param_files` is unchanged, because
  no parameter is added (the constant is reused).
* **L6 — the blackout, which is what the user asked about:** in the lab's lamp-shedding blackout,
  the crop transpires LESS than in the calm run over the same days (before this change, the same).
  In `with_lighting_failure` the window's net radiation reads 0. Each is pinned, and each pin turns
  red with the change reverted.
* **L7 — the outdoor form is untouched:** `weather::net_radiation` is bit-identical (refactored to
  share one `(1 − α)` helper with the lamp form). Held by the 7 biosphere goldens, plus a pin.

### 21a. Built — graded (2026-10-05)

**What landed.** `domains::biosphere::weather::net_shortwave` (the outdoor `net_radiation` now
calls it, same operations); `station::lighting::{lamp_net_radiation, lamp_net_radiation_path}`,
inserted as `net_radiation` by `sealed_bio_resolver` and `lighting_bio_resolver`;
`lamp_shed::LampLitEnv` scales it with PAR; `perturbations::with_lighting_failure` zeroes it with
PAR. New pins: `rust/crates/station/tests/lamp_net_radiation.rs` (4) and
`tests/lamp_shed.rs::a_blackout_also_cuts_the_crops_water_loss`. L3's measuring tool
(`W:\temp\claude\minute-transpiration\zz_scratch_lamp_rn.rs`, output `lamp_rn_L3.txt`)
reproduces both goldens' final soil water before and after, to every digit.

| # | Prediction | Result |
|---|---|---|
| L1 | exactly `sealed_station_state` and `lighting_state` move | **HELD** — `regen_goldens`: 2 of 20 |
| L2 | water stocks only | **HELD** — `lighting`: condensate 4.1904 → 3.9639, soil water 30.2475 → 30.4741; `sealed_station`: condensate 8.0323 → 4.3257, soil water 161.2299 → 165.5191, subsoil 23.3583 → 22.7757. Every other stock and every aux value is byte-identical. The chamber's vapour is unchanged too: it ends at its target either way |
| L3 | transpiration −21 ± 5 % (sealed), −8 ± 4 % (lighting) | **HELD** — sealed 2841.40 → 2248.55 kg over 1220 days (**−20.9 %**); lighting 16.012 → 14.810 kg (**−7.5 %**) |
| L4 | books; tier tests pass | **HELD** — 0 / 0 rationing; clippy clean; `cargo test --release --no-fail-fast` **1285 passed, 0 failed**, 4 ignored (1280 + 5 new), the tier-contract tests among them. The Godot parity job runs on CI only |
| L5 | exactly the two `golden_sha256` rows | **HELD** — manifest diff is those two lines |
| L6 | a blackout cuts water loss, pinned, and each pin red when reverted | **HELD** — four reversions, each run `--no-fail-fast`, restored and `cmp`-checked: sealed resolver back on outdoor → 2 red (its pin and the lighting-failure pin); lighting resolver back → 1 red; lamp shedding scaling PAR only → 1 red (the blackout pin); lighting failure leaving net radiation → 1 red |
| L7 | the outdoor form is bit-identical | **HELD** — all 7 biosphere goldens and the sunlit `greenhouse` / `harvest` identical; `the_sunlit_greenhouse_keeps_the_weathers_net_radiation` compares bits |

**Not done, and the next decisions (the user's).**
* The **chamber's temperature** is also the outdoor weather's in every lamp-lit build: the same
  kind of defect, already queued as the chamber heat store (slice 2b). Kept out of this batch.
* **§20's lab numbers were measured on the outdoor input.** They are re-predicted from scratch
  on this tree before §20's adoption question goes to the user.
* ⚠ **So were EVERY separate-air water figure before this.** The split build reads
  `sealed_bio_resolver`. That covers §13b's 23.3 kg drain, §15a's +2.08 kg reversal, §18a's and
  §18c's 21.2 kg drain and 0.66 lowest fill, and the instruments that print them
  (`examples/air_split.rs`, `examples/watering.rs`). None was re-run here. Read them as measured
  on the outdoor input. The carbon figures (§13b, §16a) do not read net radiation.
  **Re-measured in §22 (2026-10-05): every conclusion still standing holds** — the weather
  chamber gains 2.131 kg, the 22 °C chamber drains 21.248 kg byte for byte, watering 0 kg.

## 22. The separate-air water figures re-measured on the lamp's net radiation (predictions before the run, 2026-10-05)

§21a left every separate-air WATER figure measured on the outdoor input. This batch re-runs them
and changes no simulation code. **Method (advisor, 2026-10-05):** the same instrument runs on two
trees, `57688da` (the commit before §21's fix; a git worktree under `W:\temp\claude\netrad-remeasure\pre`)
and HEAD, so every difference has one cause. The pre-fix tree is first checked to REPRODUCE the
record, so nothing else between the record and the fix is mistaken for it.

**Superseded, so not re-graded.** §13b's 23.3 kg drain (reversed by the cabin humidity, §15a) and
its 0.144 starvation (ended by the minute-step gas exchange, §17a). Re-running them would grade
numbers no conclusion rests on.

**Still standing, and graded here:**
1. the weather-temperature chamber GAINS ≈ 2.08 kg a season from the cabin (§15a H5, §18a row 1);
2. the 22 °C chamber drains ≈ 21.2 kg a season to the cabin, and that drain is the PLANT STEP's
   cap — each 90-minute step sends the chamber's air only its headroom (§18a R2, §18c);
3. the root zone never falls below 0.8184 / 0.6623 full, so triggered watering delivers 0 kg and
   on/off end states are byte-identical (§18c T1);
4. plant carbon 53.263 / 28.314, untouched by water.

### 22a. Instruments — one repaired, one extended

* ⚠ **`examples/air_split.rs` had crashed since the 2026-10-03 adoption (`49adbc6`).** Its shared
  comparison built the station with `build_sealed_station` — whose default became the minute
  step — then wrapped the minute step on again, which found no carbon-budget flows left and
  panicked. Worse, before the panic its plain `shared` row had silently become the MINUTE build
  (130.4561 mol instead of 129.5564). `cargo test` compiles examples but runs none, and §17c's
  sweep of stale instruments did not include it. Repaired: the case asks
  `build_sealed_station_at(…, gas)` for its step. **No recorded figure came from the broken
  form:** its newest output, `W:\temp\claude\air_split_run6.txt` (written 18:41 on 2026-10-03),
  predates `49adbc6` (19:17), and nothing after §16b quotes this instrument.
* `examples/watering.rs` now also prints each season's transpiration, the part that reached the
  chamber's air, and the plant steps where the headroom held some back (the cap claim 2 rests
  on). Advisor's point: the cap cannot be argued from the season's MEAN transpiration — with the
  lamp's net radiation at 0 in the dark, night steps run on the vapour-deficit term alone.

### 22b. The pre-fix baseline (tree `57688da`, output `W:\temp\claude\netrad-remeasure\`)

* `watering` (unmodified) reproduces `W:\temp\claude\watering_run3.txt` **byte for byte**; the
  extended instrument prints the same figures plus the new counts (`watering_pre2.txt`).
* The repaired `air_split` reproduces `W:\temp\claude\air_split_run6.txt` **value for value**
  (column spacing only differs) — `air_split_pre2.txt`.
* New, pre-fix: transpiration **663.623 kg** (weather) / **1735.062 kg** (22 °C), to the air
  **3.217 / 22.076 kg**, the cap holding some back on **4880 of 4880** plant steps in both.

### 22c. Predictions (committed before the HEAD run)

The mechanism they rest on: a capped step sends the air exactly its headroom, `target − v +
condensed`, a function of the chamber's vapour and temperature alone — not of the leaves' flux
(`flows.rs`, `Transpiration::evaluate`). Net radiation reaches nothing but that flux. So while
every step stays capped, the air side, the fan, the cabin and the crew cannot see the fix.

* **N1 — the cap still binds.** 22 °C: capped on **4880 of 4880** (night flux from the
  vapour-deficit term is ≈ 0.02–0.1 kg a step against a ≈ 0.004 kg headroom). Weather chamber:
  capped on **≥ 4636 (95 %)**; it is the one at risk, its air sitting at ≈ 0.91 RH, where a
  cold dark step's flux is smallest.
* **N2 — the cabin side cannot see it.** If N1 holds at 4880 in a case: fan export, to-air,
  crew-store Δ and brine Δ are **byte-identical** to pre-fix in that case (−2.083 / 21.248 kg
  export). If the weather chamber has uncapped steps: its export moves by under 0.5 kg and stays
  cabin → plants.
* **N3 — plant carbon byte-identical** in all four runs (no run reaches water stress either way;
  §21a L2 saw the same in the goldens).
* **N4 — transpiration falls:** weather **−15 to −25 %**, 22 °C **−10 to −25 %** (the sealed
  golden fell 20.9 %); the fall moves from condensate into the soil, with the plants' total water
  unchanged where N2 holds.
* **N5 — the root zone's lowest fill RISES** in both (less drawn), so watering still delivers
  **0 kg** and on/off stay byte-identical.
* **N6 — the repaired `air_split` at HEAD:** carbon columns byte-identical to pre-fix; the
  vapour-on rows move only in transpiration, as N1–N4.

So the prediction is that **all four standing conclusions stand**; the only numbers to restate
are transpiration and where the plants' water sits (soil versus condensate).

### 22d. Graded (2026-10-05) — all four standing conclusions STAND

Outputs in `W:\temp\claude\netrad-remeasure\`: `watering_pre2.txt` / `watering_head.txt`,
`air_split_pre2.txt` / `air_split_head.txt`. The byte claims were checked by dumping each run's
four end states from both trees (a temporary line in the instrument, removed; the dumping
re-runs printed the same tables) and comparing all 38 stocks and every aux value.

| Case | Transpired pre → HEAD (kg) | Capped steps pre → HEAD | Fan export (kg) | Lowest fill | End stocks that differ |
|---|---|---|---|---|---|
| weather chamber | 663.623 → **512.784** (−22.7 %) | 4880 → **4424** | −2.083 → **−2.131** | 0.8184 → 0.8356 | 5 of 38, all water: soil, subsoil, condensate, crew store, brine |
| 22 °C chamber | 1735.062 → **1563.655** (−9.88 %) | 4880 → **4880** | 21.248 → 21.248 | 0.6623 → 0.6796 | 2 of 38: soil water, condensate |

| # | Prediction | Result |
|---|---|---|
| N1 | 22 °C capped 4880/4880; weather ≥ 4636 | **HALF** — 22 °C **HELD** (4880); weather ✗ **FAILED**, **4424** (90.7 %): 456 steps where the leaves' flux is under the chamber's headroom. Which steps those are (dark, cold, humid was the guess) was **not measured** |
| N2 | 22 °C cabin side byte-identical; weather export moves < 0.5 kg, still cabin → plants | **HELD** — 22 °C: chamber vapour, cabin vapour, crew store, brine byte-identical; only the plants' soil water and condensate differ. Weather: export moves **0.048 kg**, still cabin → plants (crew store −133.741 → −133.784, brine +131.540 → +131.535) |
| N3 | plant carbon byte-identical, all four | **HELD** — no carbon stock and no aux value differs in any run |
| N4 | weather −15 to −25 %, 22 °C −10 to −25 %; the fall lands in the soil | **HALF** — weather **HELD** (−22.7 %); 22 °C ✗ **MISSED** by a hair, **−9.88 %**. The fall lands in the soil (22 °C: soil +4.509 kg, condensate the same amount lower, plants' total unchanged; weather: soil +3.408, subsoil +0.379, total +0.048 = the export's move) |
| N5 | lowest fill rises; watering 0; on/off byte-identical | **HELD** |
| N6 | `air_split` at HEAD: carbon columns byte-identical; vapour-on rows move as N1–N4 | **HALF** — carbon **HELD**: every co2 / plant C / cabin CO₂ column identical to pre-fix. The vapour-on row moves as N1–N4 graded, so it carries N1's miss: to air 3.217 → 3.169, capped 4880 → 4424, plant and crew water move too — under the "transpiration only" N1 at 4880 would have given, ✗ **FAILED as stated**. Unpredicted but in line: the vapour-OFF split's transpiration 708.207 → 557.383 with the cap 4880 → 4877, and the shared rows 711.247 → 563.077 (−20.8 % in one season; the 1220-day golden fell 20.9 %) |

**Tally: 3 held, 3 half.** The two misses are both the size of the night-time flux: I put it
higher than it is, so more weather-chamber steps fall under the cap than predicted, and the warm
chamber, where the vapour deficit dominates, fell a little less.

**What the earlier conclusions now read as:**
1. **The weather-temperature chamber still gains from the cabin: 2.131 kg** a season (was 2.083).
2. **The 22 °C chamber still drains 21.248 kg** to the cabin, byte for byte, and the plant step's
   cap is what sets it on every step. §18a R2's mechanism stands.
3. **Watering still delivers 0 kg**; the root zone's lowest fill is 0.8356 / 0.6796 (was
   0.8184 / 0.6623) — more margin above the 0.45 trigger, not less.
4. **Plant carbon unchanged**: 53.263 / 28.314.
5. **Restated:** the separate-air transpiration totals (512.784 / 1563.655 kg) and where the
   plants' water sits. In the weather chamber the cap no longer sets every step, so its gain is
   now partly the leaves' own flux.

⚠ **Not covered here:** §20's minute-step transpiration numbers (the next item, re-predicted from
scratch before its adoption question). §15a's shared-room "above saturation on 4 876 of 4 880"
now reads **4 875** at HEAD (`air_split_head.txt`); max unchanged at 2.8798.

## 23. Slice 2b — the plant chamber's heat store: design, priced before code (2026-10-05)

The user, offered the minute-step adoption question or this: *"2"* (this). §10's slice 2 / §11's
2b, on the default build, a station unfreeze. **The plants do NOT read the chamber yet** (that is
slice 3, with the cold period). No code in this section.

### 23a. Advisor review (2026-10-05), summarized

1. **Two golden diffs, not one.** 2b-i: the chamber store inserted as a pass-through on the
   lamp's *waste-heat* leg only (lamp → chamber → exchanger → node); the light leg stays on
   `boundary.light_used`. Every non-energy stock byte-identical (dump all stocks, as §22d); the
   node *not* necessarily byte-identical (the exchanger lags, the arithmetic path differs) —
   predict the size, not zero. 2b-ii: the light leg re-pointed into the chamber; this is where
   the node warms, and `sealed_node_heat` (whose doc says the radiant leg "leaves as PAR, not to
   the node") must start the node at its new equilibrium.
2. **"Nominal chamber = setpoint" is not a prediction a lagged controller can meet.** Pick the
   control law, then commit the steady offset in closed form. The sealed station's lamp draw is
   the daily **average** (`lighting_average_power`), so the chamber gets a constant input and no
   day/night swing although the crop's light switches — recorded, not fixed here.
3. **Check that B delivers the failure the user chose it for.** With no heater and no heat-loss
   path, a dead lamp leaves the chamber at its temperature forever; a radiator fault reaches the
   chamber only if the exchanger is gated on the node being colder, and then after ~a month.
   These are the user's calls.
4. **Price the sugar-fixed share of the light** from the golden, with a cited energy content.
5. **Look for sources** for the chamber's heat capacity and the exchanger's capacity before
   labelling them DESIGN (BVAD Table 4-88's page image, owed since §11).
6. **What breaks:** `lamp_shed` counts a step as lit when light reaches `boundary.light_used`
   (`lamp_shed.rs:359`) — after 2b-ii it never does; `air_split` inherits the chamber through
   `sealed_fast_flows`; Godot may carry a stock list; manifest + completeness gates.
   `tier1_node_is_period_1_fixed_point` reads the heat-closure run and should not move.

### 23b. Measured and sourced

**BVAD Table 4-88, page image READ** (PDF page index 183, printed p. 170;
`W:\temp\claude\chamber-heat\bvad_t4-88.png`). §11's extract reading holds. Rows (mass kg/m²,
volume m³/m², power kW/m², thermal control kW/m²): Crops 20.0 / – / – / –; Shoot Zone 3.6 / 0.67 /
0.3 / 0.3; Root Zone Water and Nutrients 36.8 / 0.11 / 0.14 / 0.14; Lamps 22.9 / 0.25 / 2.1 / 2.1;
Ballasts 8.4 / TBD / 0.075 / 0.075; Mechanization 4.1; Secondary Structure 5.7; Total 101.5 /
1.03 / 2.6 / 2.6. Footnote 183: *"Power consumption and thermal control within the shoot zone
reflect fans for gas movement."* (Drysdale 1999b.)
* **The table books every watt as heat to reject — the lamps' 2.1 kW/m² included.** Thermal
  control equals power, row by row. A class citation for sending the lamp's light into the
  chamber as heat, and for **sizing the exchanger to the chamber's installed power**.
* It gives **no** specific heat and **no** controller rate.

**The sugar-fixed share of the light — priced from the frozen golden.** Energy content: α-D-glucose
ΔcH°solid = **−2805.0 ± 1.3 kJ/mol** (NIST Chemistry WebBook SRD 69, CAS 492-62-6, condensed-phase
data; Ponomarev & Migarskaya 1960, reanalysed by Cox & Pilcher 1970; opened 2026-10-05) →
**467.5 kJ per mol C**. In `sealed_station_state.json` (n = 19 520, 1220 days) the organic carbon
standing at the end (leaf, stem, root, grain, stem reserve, litter, humus, microbes) is
**117.80 mol** — an upper bound on what the run stored chemically, since its start is not
subtracted and respiration in the chamber returns the rest as heat. 117.80 × 467.5 kJ =
**55.1 MJ** against `boundary.light_used` = **7.688 GJ**: **≤ 0.72 %**. (The per-season reading,
the whole standing crop at a harvest — 59.71 mol — against one season's light, 1.922 GJ, is
1.45 %.) **So all the light goes to the chamber as heat; the overcount is recorded, ≤ 0.72 %.**
Coupling the fast `Lamp` to the crop's assimilation would be a cross-domain seam; not taken.

**The chamber's heat capacity — no source fixes it; a plausibility anchor, not a derivation.**
Air alone (27.66 mol × 29.1 J/mol·K ≈ 0.80 kJ/K) heats **9.9 K per minute step** at the lamp's
133.3 W: unusable. Anchor: Table 4-88's root-zone water and nutrients, 36.8 kg/m², taken as water
(NIST WebBook, liquid water Shomate fit, Chase 1998: Cp = 75.375 J/mol·K at 298.15 K =
4 184 J/kg·K) → 1.54e5 J/K for our 1 m². **Proposed: C_ch = 1.5e5 J/K, DESIGN** — the
`radiator.yaml` precedent ("a post-hoc plausibility anchor … NOT a derivation"). One minute step's
full input over it: 133.3 × 60 / 1.5e5 = **0.053 K** — the Euler bound holds.

**The exchanger's capacity — a class sizing rule, a DESIGN number.** Table 4-88 sizes thermal
control to the installed power; ours is the lamp's nameplate **200 W** (fans unmodelled). Margin
over the averaged input 133.3 W: 1.5×.

**The node, closed form** (`equilibrium_temperature`; today's heat recomputed from the golden's
end node, 167.4238 K → 378.703 W):
* 2b-i: the same heat reaches the node → **167.4238 K**, unchanged in steady state.
* 2b-ii: + the light leg 72.939 W → 451.643 W → **174.961 K (−98.19 °C), +7.537 K.**
* The node's own relaxation time today: 12.8 days.

**The failure directions — what B can and cannot show, priced:**
* **Exchanger fault** (capacity → 0): the chamber heats at 133.3 / 1.5e5 = **3.2 K per hour**
  (waste heat alone, 2b-i: 1.45 K/h). The hot direction works with no new number.
* **Radiator total loss:** the node gains 451.6 W / 1e7 J/K = **3.90 K per day**, and passes
  22 °C after **≈ 31 days** (2b-i: 3.27 K/day, ≈ 39 days). It reaches the chamber **only if** the
  exchanger refuses to move heat into a warmer node (the second law: no heat pump is modelled).
* **Lamp off** (failure, shedding): with no heater and no heat-loss path the chamber **holds its
  temperature indefinitely** — B shows nothing in the cold direction. A loss path needs a wall
  conductance (DESIGN) toward something: the node (−98 °C in this model — a chamber leaking to it
  would freeze and need a heater) or the cabin (no temperature in the default build).

**The control law — the precedent is the cabin condenser:** first-order toward a setpoint,
one-sided, a DESIGN rate (`eclss.yaml` `condense_rate` 5e-4 /s, "a solver-stability choice").
Proposed: removed per step = `min(capacity·dt, k·(Q − Q_set)·dt)` when `Q > Q_set`, else 0.
Proportional, so the chamber settles **above** the setpoint by `input / (k·C_ch)`:

| k (1/s) | τ | k·dt | offset, 2b-i (60.39 W) | offset, 2b-ii (133.33 W) |
|---|---|---|---|---|
| 5e-4 (the condenser's) | 33 min | 0.03 | 0.81 K | 1.78 K |
| 1/600 | 10 min | 0.1 | 0.24 K | **0.53 K** |
| 1/60 | 1 min | 1.0 | 0.024 K | 0.053 K (at the Euler edge) |

The chamber starts at its own closed-form steady state (setpoint + offset), so a nominal run is
flat from step 0. The stock is referenced to absolute zero (`T = Q / C_ch`), not to `T_space`.

### 23c. Decisions owed to the user (asked 2026-10-05)

1. **A dead lamp:** keep the chamber at its temperature (hot direction only), or add a wall loss
   (toward what) — *recommended: hot only, revisit with the cabin heat store (2c).*
2. **The radiator:** the exchanger works only while the station structure is colder than the
   chamber — *recommended: yes (physics; costs nothing nominally; a radiator loss reaches the
   chamber after ≈ 31 days).*
3. **How tightly the chamber is held:** *recommended: τ = 10 min, 0.53 °C above 22 °C.*

Not asked, defaulted (say so to override): all the light becomes chamber heat (≤ 0.72 %
overcount); C_ch = 1.5e5 J/K and capacity = 200 W, both DESIGN with the anchors above.

**ANSWERED 2026-10-05 (verbatim):**
1. Dead lamp: *"walls loose heat to what is near them, the outside should be modelled - it can be
   environment, it can be inside the station, it can be in space . also the oustide may be hotter
   or cooler"* — the recommendation (hot direction only) was **declined**.
2. Radiator: *"Yes, physical (Recommended)"* — the exchanger moves heat only into a colder node.
3. Control: *"1-minute response"* — k = 1/60 s⁻¹, k·dt = 1.0 on the 60 s step: the controller
   removes the **whole** snapshot excess each step (capped by capacity). Steady offset 0.024 K
   (2b-i) / 0.053 K (2b-ii) above 22 °C. ⚠ `k·dt > 1` would overshoot and oscillate on a coarser
   step, silently — so the build **hard-errors** on `k·dt > 1`.

**Advisor on the answers (2026-10-05), summarized.** 2b-i and 2b-ii stand unchanged (they do not
depend on what the chamber sits next to). The walls are a **third** golden diff, **2b-iii**. Before
asking again: price each surroundings option (wall flow across a range of insulation, sources
searched first), and say whether the lamp's 133 W alone holds 22 °C against the loss — if not, a
heater is not optional. Questions owed: a heater on the battery (a new load, a new DESIGN
capacity); what the reference chamber sits in (§10 put it **inside the station**, so "inside" is
the consistent default — the cabin held at 22 °C, cited, or the node, whose −98 °C is not
physical; or wait for 2c's cabin store); ship one surroundings in the reference and the others
lab-only. Mechanics: a two-signed boundary for the surroundings (check simcore already has one —
its diff stays empty); a test either side of `T_node = T_ch`, since the node is referenced to
`T_space` and the chamber to 0 K; predict the bounded dip `wall·dt/C` when wall and exchanger act
on the same step.

**Checked:** simcore's `boundary::source(.., unclamped = true)` already takes both signs
(`boundary.o2_supply` ends negative in the golden), so a two-signed surroundings reservoir needs
no simcore change.

### 23d. 2b-iii — the walls, priced (2026-10-05)

**The wall's conductance — a CLASS citation.** BVAD Table 4-50, "Frozen Food Storage on a Property
per Frozen-Food-Mass Basis" (page image READ, PDF page index 125,
`W:\temp\claude\chamber-heat\bvad_t4-50.png`): **1/R_S = 0.28 / 0.32 / 0.32 × 10⁻³ kW/m²·K** (low /
nominal / high, Ewert 2002) — the composite wall resistance of an ISS-class cooled cabinet *"through
the cabinet wall accounting for insulation, door seals, and any other pathways"* (text, printed
p. 111). Locus: a freezer cabinet inside a cabin, not a plant chamber — class only. Taken: **U =
0.30 W/m²·K** (mid of the range). The chamber's wall area is DESIGN geometry: a 1 m × 1 m box of
Table 4-88's shoot + root volume (0.78 m³ → 0.78 m tall) = **5.12 m²** → **UA = 1.536 W/K** (cited
range 1.43–1.64). Time constant `C_ch/UA` = **27 h**.
* BVAD p. 144: *"Passive thermal control … generally takes the form of insulation and resistive
  heaters"*, and footnote 161: inside a pressurized cabin, conduction/convection dominate and
  radiant exchange is often neglected. A cited precedent for a heater, and for a linear wall law
  inside the station.

**Each surroundings, with the chamber at 22.05 °C and the lamp's 133.3 W (2b-ii):**

| Surroundings | Law | Normal wall flow | Lamp alone holds 22 °C? | Lamp dead |
|---|---|---|---|---|
| **Cabin**, held at a setting (22 °C BVAD nominal; settable hotter or cooler) | UA·ΔT | **0.08 W** out | yes | stays ≈ 22 °C (nothing to see at a 22 °C cabin) |
| **Outdoor weather** (the Dutch file, −1.8 to 22.2 °C) | UA·ΔT | 17.5 W mean, 36.6 W max out | yes | cools toward outdoors, 27 h time constant |
| **Space** (insulation blanket, radiative to 2.7 K) | ε*σA(T⁴−T_s⁴) | 3.1 / 22 / 66 W at ε* = 0.0014 / 0.01 / 0.03 | yes | cools, radiatively |
| **Station structure** (the node, −98 °C) | UA·ΔT | **185 W** out | **NO** — settles at −11.4 °C without a heater | — |

* ε* has **no source**: NASA/TP-1999-209263 (Finckenor & Dooling, opened) gives layer
  emittance < 0.04 and "each reflector will reflect 90 to 99 percent", not a blanket's effective
  emittance. 0.0014 is the textbook ideal for 15 layers at 0.04 (form only); real blankets are
  worse by an unsourced factor.
* The station-structure row is driven by the node's −98 °C, which §2 already says is not physical.
* With the cabin at 22 °C the wall matters in slice 3's **cold phase**: a 4 °C chamber gains
  1.536 × 18 = **27.7 W** from the cabin.
* Where the heat goes: to a boundary for cabin / outdoors / space (it leaves the station's energy
  books — for the cabin, a recorded gap until 2c gives the cabin a store); to the node for the
  structure.

**A heater** (a battery load, resistive, DESIGN capacity) matters only where the lamp cannot hold
the chamber: always for the structure, and with the lamp dead for outdoors and space.

**ANSWERED 2026-10-05 (verbatim):**
1. Reference surroundings: *"since we have most data from experiments, when plants grow on natural
   light, maybe the standard should stay as outdoor weather, becasue we will have most data to
   calibrate. but the model should be able to run realistically (even if not proven) all
   variants."* — **outdoor weather** in the reference (the recommendation, the cabin, declined).
2. The other three: *"Yes, as experimental (Recommended)"* — cabin (held, settable hotter or
   cooler), space (radiative; ε* unsourced → WHAT-IF), station structure: **lab options**.
3. Heater: *"Yes"* — a resistive heater on the battery, DESIGN capacity.

⚠ **Flagged to the user, to be re-asked before slice 3:** walls facing the weather do not make
the plants feel the weather — the chamber is still held at its setting, so the walls only change
how hard the heater and cooler work. What decides calibration against field data is **which
temperature the plants read**: today the weather; slice 3 (as planned) the held chamber.

### 23e. 2b-i — predictions, committed before code (2026-10-05)

**What is built.** A chamber heat store `thermal.chamber` (ENERGY POOL, `T = Q / C_ch`, 0 K
reference); the `Lamp`'s waste-heat leg re-pointed node → chamber; one new flow
`station.chamber_cooling` (`ChamberCooling`): chamber → node,
`min(capacity·dt, (Q − C_ch·T_set)·dt/τ)` when `Q > C_ch·T_set` **and** `T_node < T_ch`, else
0. New station param file `chamber.yaml`: `heat_capacity` 1.5e5 J/K (DESIGN, the water anchor),
`cooling_capacity` 200 W (DESIGN, Table 4-88's class sizing rule), `response_time` τ = 60 s
(DESIGN, the user's "1-minute response"), `setpoint` 295.15 K (BVAD Table 4-73, by analogy; the
user's 22 °C). The build **hard-errors** when the fast step `dt > τ`. The chamber starts at its
closed-form steady state `C_ch·T_set + w·dt`, `w` = the lamp's averaged waste heat. No walls, no
heater, light leg unchanged (2b-ii, 2b-iii).

**Predictions (the frozen `sealed_station` golden, regenerated):**

| # | Prediction |
|---|---|
| P1 | Every **non-energy** stock and **every aux value** in the end state is **byte-identical** to HEAD's golden. Of the energy stocks, `power.battery`, `boundary.solar_source` and `boundary.light_used` are byte-identical too (the lamp draws the same; its light leg is untouched) |
| P2 | `thermal.chamber` starts at `C_ch·T_set + w·dt` with w = 60.39387 W: **T = 295.174158 K** (offset `w·dt/C_ch` = 0.024158 K), and stays there to within **1e-9 K** on every step (deadbeat: `dt/τ` = 1.0 exactly, so each step removes the whole excess) |
| P3 | `thermal.node` is **NOT predicted byte-identical**: its increment now arrives through the cooler (`(Q − Q_set)·1.0`, equal to `w·dt` within a few ULP of Q ≈ 4.4e7 J) and in a different sum. End-state node within **1e-9 relative** of HEAD's 1.6472e9 J (T = 167.4238 K to 6 figures); `boundary.space` within 1e-9 relative |
| P4 | Energy conservation holds every step; `rationed == 0` (the cooler's draw is ≤ 12 kJ against a 4.4e7 J store) |
| P5 | The second-law gate never binds in nominal running (node ≈ 167 K ≪ 295 K): a test either side of `T_node = T_ch` (reference points differ: node from `T_space`, chamber from 0 K) — heat moves just below, none just above |
| P6 | Unmoved: every biosphere golden, `lighting`, heat closure, `sealed_energy_drift_summary`, the perturbed brown-out, `tier1_node_is_period_1_fixed_point` (all read runs without the chamber) |
| P7 | Manifest diff: + flow type `ChamberCooling`, + param file `chamber.yaml` and its four params, `sealed_station`'s `golden_sha256` row; nothing else |
| P8 | The lab builds inheriting `sealed_fast_flows` (`air_split`, `lamp_shed`, the Godot sealed session): carbon and water unchanged; `lamp_shed`'s rule-off bit-identity test stays green (the light leg is untouched in 2b-i) |
| P9 | A cooler fault (capacity → 0, lab test): the chamber warms at `w/C_ch` = **1.45 K per hour**, and the node, losing that input, relaxes toward the colder equilibrium of 317.6 W ≈ **160.3 K** on its 12.8-day time scale |

P9's 160.3 K is computed with the closed form `equilibrium_temperature` from 378.703 − 60.394 W.

### 23f. 2b-i BUILT — graded (2026-10-05)

Built as §23e describes: `rust/crates/station/src/chamber.rs` (the stock, the flow, the
build guard), `rust/crates/station/params/chamber.yaml`, the sealed builder wiring
(`sealed.rs`; `air_split` passes the same params). Golden regenerated; the station manifest
regenerated. HEAD's golden kept for the comparison at
`W:\temp\claude\chamber-heat\sealed_head.json`.

| # | Result |
|---|---|
| P1 | **HELD, and stronger** — every stock present in both goldens is byte-identical, energy included; the only difference is the added `thermal.chamber`. Every aux value identical; `n` = 19 520 both |
| P2 | **HELD, exactly** — the end-state chamber is `295.1741575492341 K`, the same bits as its start `C_ch·T_set + w·τ`; the lab test asserts ≤ 1e-9 K on all 61 days of its nominal run |
| P3 | ✗ **MISSED, on the safe side** — predicted "not byte-identical, within 1e-9"; measured **byte-identical**, `thermal.node` and `boundary.space` both. The reading offered (not tested): the cooler hands the node the same bits the lamp's leg used to, in the same position among the node's legs. |
| P4 | **HELD** — the golden run asserts `rationed == 0` and no events |
| P5 | **HELD** — `chamber::tests::heat_moves_only_into_a_colder_node`: heat moves with the node 1e-6 K below the chamber, none at equality, 1e-6 K above or 50 K above |
| P6 | **HELD** — `regen_goldens` report: 19 of 20 identical, `sealed_station_state.json` alone changed |
| P7 | **HELD** — manifest diff is three lines: + `ChamberCooling` in the flow set, + `chamber.yaml`'s digest, `sealed_station`'s `golden_sha256`. (The manifest carries a file digest, not a per-parameter list, so "its four params" appear only through that digest.) |
| P8 | **HELD, by running the instruments, not only the suite** (advisor: green tests are not "carbon and water unchanged", and nothing runs examples — §22's crash). The seven examples that build the sealed station (`air_split`, `air_split_baseline`, `draw_census`, `intraday_exchange`, `lamp_shed`, `watering`, `minute_transpiration`) ran on `1ae2c93` (the commit before) and on `14101a4`; outputs in `W:\temp\claude\chamber-heat\examples\`. Byte-identical: `air_split_baseline`, `lamp_shed`, `watering`. The rest differ **only** in run times, and in `draw_census` a new `thermal.chamber` row (worst draw 0.000082) and `intraday_exchange` counting "19 stocks bit-identical" instead of 18 — the added stock. `draw_census` control 1 reads `sealed_station_state.json: byte-exact` against the new golden. `air_split`'s output equals the §22 record `W:\temp\claude\netrad-remeasure\air_split_head.txt` apart from timing lines. The suite: green apart from the stale-manifest gate before regeneration; `air_split`, `lamp_shed` (rule-off bit-identity included), session parity and save/load pass. CI on `14101a4`: green |
| P9 | **HELD** — `tests/chamber_heat.rs`, 60 days, the cooler scaled to 0: the chamber's first-day rise equals `w·86400/C_ch` = 34.787 K to 1e-9 relative; the node falls on every day and ends within 0.15 K of the closed-form **160.308 K**; every non-energy stock and every aux value byte-identical to the nominal run on all 61 days |

**The cross-port tier contract** (`rust/data/tiers.json`) carries a band per **golden**, not a
stock list, so `thermal.chamber` is compared with every other stock of `sealed_station`; no
action. **Prose made false by the change, corrected** (advisor): `sealed.rs`'s module header,
the `Lamp` docs in `flows.rs`, and `docs/station-reference.md`'s flow list and param-file
count. `sealed_node_heat`'s doc ("the radiant leg leaves as PAR, not to the node") is still
true and changes in 2b-ii.

**Tally: 8 held, 1 missed (P3, conservative).** Liveness: re-pointing the lamp's heat leg back
to the node (the pre-2b-i wiring) turns `tests/chamber_heat.rs` red at P2's assertion
(`nominal chamber drifted: 295.15 vs 295.1741575492341`); restored.

⚠ **What 2b-i shows and does not.** With no walls, a dead cooler heats the chamber without limit
(≈ +2 090 K over the 60-day test). That is the absence of 2b-iii's heat-loss path, not a
prediction about a real chamber. Next: 2b-ii (the light leg into the chamber; the node warms to
174.961 K), predictions first.

### 23g. 2b-ii — the lamp's light into the chamber: design and predictions, before code (2026-10-05)

**What is built.**
* **The lamp's whole draw becomes chamber heat.** simcore rejects two legs on one stock
  (`FlowResult::new`: "a flow must net…"), so `Lamp` with its light and heat targets equal
  emits **one** leg, `+draw`, to that target (`battery −draw`). Physically Table 4-88's
  "thermal control = power"; the ≤ 0.72 % stored as sugar is the recorded overcount (§23b).
  `Lamp` with two different targets (the `lighting` build) is untouched.
* **`boundary.light_used` leaves the sealed build** (a stock that would read 0 forever is a
  false record). The `lighting` build keeps its own.
* **`sealed_node_heat`** counts the whole lamp draw (its doc's "the radiant leg leaves as PAR,
  not to the node" becomes false and is rewritten), so the node starts at its new equilibrium.
  The chamber's input is the whole averaged draw; `chamber_heat0` takes it.
* **`lamp_shed`'s lit detector** read `boundary.light_used`. Replaced: a fast step counts as lit
  when the lamp flow, evaluated on the step's starting state, draws from the battery (the
  `SheddingLamp` zeroes every leg when shed; a failed lamp draws 0). The caveat it already
  carried stands: a lamp cut by the arbitration backstop would count as lit; the tests assert
  `rationed == 0` where they rely on it.

**Predictions** (closed forms from `equilibrium_temperature` and the 2b-i golden):

| # | Prediction |
|---|---|
| Q1 | Every non-energy stock and every aux value **byte-identical** to the 2b-i golden; `power.battery` and `boundary.solar_source` byte-identical (the lamp draws the same) |
| Q2 | `boundary.light_used` gone from the golden; `thermal.chamber` starts and stays at **295.203333 K** (`T_set + 133.333·60/1.5e5`), within 1e-9 K |
| Q3 | `thermal.node` starts and holds at **174.961 K** (451.643 W), +7.537 K; end state within 1e-6 K of the closed form |
| Q4 | `boundary.space` at the end = the 2b-i value + the 2b-i `light_used` (39.918 + 7.688 = **47.607 GJ**), within 1e-6 relative |
| Q5 | Goldens: only `sealed_station` moves (`lighting` byte-identical — its `Lamp` keeps two targets); manifest diff = that golden's hash only (no flow or param added) |
| Q6 | `lamp_shed`: the rule-off build reproduces the plain sealed run bit for bit (the detector counts every step lit, so the ratio stays exactly 1.0); its rule-on tests keep their verdicts; the blackout test's `light_used` comparison is rewritten to the battery-drawn lit-step count (the same claim: the lamp ran less) |
| Q7 | A dead cooler (`tests/chamber_heat.rs`, updated to the whole draw): the chamber warms **76.8 K a day**; the node relaxes toward the **same 160.308 K** as in 2b-i (it loses the whole lamp either way); crop byte-identical |
| Q8 | `tier1_node_is_period_1_fixed_point` and every heat-closure figure unmoved (they read the heat-closure run) |
| Q9 | The seven sealed examples: identical to 2b-i apart from timings, except `lamp_shed`'s printed `light_used` column, which is replaced |

### 23h. 2b-ii BUILT — graded (2026-10-05)

Built as §23g describes (`flows.rs`: `Lamp` nets its legs when both targets coincide;
`sealed.rs`: both targets `thermal.chamber`, `boundary.light_used` dropped, `sealed_node_heat` and
the chamber's start on the whole draw via `chamber_heat_input_w`; `lamp_shed.rs`: the
`lamp_draws` detector). 2b-i's golden kept at `W:\temp\claude\chamber-heat\sealed_2bi.json`.

| # | Result |
|---|---|
| Q1 | **HELD** — only three stocks differ from 2b-i's golden, all energy: `thermal.node`, `thermal.chamber`, `boundary.space`. `power.battery`, `boundary.solar_source`, every other stock and every aux value byte-identical |
| Q2 | **HELD** — `boundary.light_used` gone; the chamber ends at **295.2033333333333 K** (start = end) |
| Q3 | **HELD** — the node ends at **174.96093576109791 K**, the closed form's printed digits exactly |
| Q4 | **HELD** — `boundary.space` 47 606 755 251.94 J against the predicted 47 606 755 251.08 J: **1.8e-11 relative** |
| Q5 | **HELD** — `regen_goldens`: 19 of 20 identical (`lighting` among them); manifest diff = `sealed_station`'s `golden_sha256` only |
| Q6 | **HELD** — the full suite's `lamp_shed` tests pass with the new detector: the rule-off build reproduces the plain sealed run bit for bit, the rule-on verdicts stand, and the blackout test now asserts the lab delivered less light in all (the sum of its delivery log) |
| Q7 | **HALF** — the rates held: the dead-cooler chamber warms **76.8 K a day** (asserted to 1e-9) toward the same **160.308 K** node target, crop byte-identical. ✗ The test's fixed **0.15 K** "near the target after 60 days" band, sized in 2b-i for a 7.1 K starting gap, was carried over **without re-deriving** it: from 174.96 K the node ends **0.2095 K** above the target, and the suite went red. Replaced by a derived bound (for `T ≥ T_eq`, `T⁴ − T_eq⁴ ≥ 4T_eq³(T − T_eq)`, so the gap shrinks at least as fast as `exp(−t/τ_eq)`, τ_eq = 14.57 days at 160.3 K): bound **0.2387 K**, measured 0.2095 K, and no overshoot |
| Q8 | **HELD** — `tier1_node_is_period_1_fixed_point` and the heat-closure figures pass unchanged (the suite) |
| Q9 | **HELD** — the seven examples on this tree against 2b-i's outputs (`W:\temp\claude\chamber-heat\examples_2bii\` vs `…\examples\head_*.txt`), timings stripped: `air_split`, `air_split_baseline`, `watering`, `minute_transpiration` identical; `draw_census` differs only in the energy rows (`thermal.chamber`'s worst draw 0.000082 → 0.000181, `thermal.node` 0.000014 → 0.000016); `intraday_exchange` counts 18 bit-identical stocks instead of 19 (`light_used` gone); `lamp_shed` differs **only** in the replaced last column — crop, battery, rationing and the "first shed in group 74 (day 4.625), share 0.1667, dark to the end" line are identical. Cross-check of the new detector: the blackout run's old light ratio 2.9212e7 / 5.0416e7 = **0.5794** = its new mean delivered share |

**Tally: 8 held, 1 half (Q7: the rates held, a carried-over test band did not).**

**§2's first "gap in the heat books" is closed:** the lamp's light no longer leaves the station;
it heats the chamber, and the node carries it to the radiator (+72.94 W, +7.54 K).

### 23i. 2b-iii — the walls and the heater: design and predictions, before code (2026-10-05)

**What is built.**
* **`ChamberWall`** (`station.chamber_wall`): chamber ↔ its surroundings, either sign. Conductive
  `UA·(T_ch − T_sur)` or radiative `ε*·σ·A·(T_ch⁴ − T_sur⁴)`. Surroundings, the user's four:
  * **outdoor weather — the reference**: `T_sur` = the weather file's daily temperature (the one
    the plants read), °C + 273.15, into a two-signed boundary `boundary.chamber_surroundings`;
  * **cabin** (lab): a held temperature, settable hotter or cooler, same boundary;
  * **space** (lab): radiative to `T_space` into `boundary.space`, ε* **WHAT-IF** (no source,
    §23d) — a lab constant, never in `chamber.yaml`;
  * **station structure** (lab): conductive to `thermal.node`.
* **`ChamberHeater`** (`station.chamber_heater`): battery → chamber, the cooler's mirror —
  `min(heater_capacity·dt, (C_ch·T_set − Q)·dt/τ)` while below the setpoint. Resistive, so every
  joule drawn is heat.
* **`chamber.yaml` gains three:** `wall_conductance` **0.30 W/m²·K** (CITED, class: BVAD Table 4-50
  1/R_S 0.28–0.32, a freezer cabinet wall, §23d), `wall_area` **5.12 m²** (DESIGN: a 1 m × 1 m
  box of Table 4-88's 0.78 m³), `heater_capacity` **200 W** (DESIGN: symmetric with the cooler;
  must exceed the structure option's steady 44.8 W, below).
* **⚠ The time base.** The fast operator keeps the **slow** step count `n` (16 a day) while it
  runs at `dt = 60 s`, so the weather table's own indexing (`floor(n·dt)` with `dt` in days)
  would read the wrong day there. The outdoor temperature is read as `floor(n·bio_dt)` — the
  same day the plants read — and a test pins it at day boundaries.

**Predictions — the reference golden** (an independent Python re-simulation of chamber + node on
the 60 s step over the 1220-day weather, `W:\temp\claude\chamber-heat\`; not the Rust code):

| # | Prediction |
|---|---|
| R1 | Every non-energy stock and every aux value **byte-identical** to 2b-ii; `power.battery` and `boundary.solar_source` **byte-identical** — the heater never fires (the lamp's 133.3 W exceeds the largest wall loss, 36.6 W), so it emits zero legs every step. Walls and heater share one golden diff **because** the heater is inert to the bit there |
| R2 | `boundary.chamber_surroundings` ends at **+1.8417e9 J** (17.47 W mean out), within 1e-4 relative |
| R3 | The node is no longer a fixed point: from 174.961 K it ends at **174.141 K**; its daily range **172.17–174.89 K**, mean **173.25 K**; end within 0.01 K |
| R4 | The chamber varies with the weather: **295.1887–295.2034 K**, ends 295.1999 K (each step `T_set + (133.33 W − wall)·60/C_ch`) |
| R5 | `boundary.space` = 2b-ii's − 1.8335e9 J (the wall's heat leaves through the walls, not the radiator, less the node's stored change), within 1e-4 relative |
| R6 | Goldens: only `sealed_station` moves. Manifest: + `ChamberWall`, + `ChamberHeater` (flow set), `chamber.yaml`'s digest, the golden hash |
| R7 | `tier1_node_is_period_1_fixed_point` and heat closure unmoved; the seven sealed examples identical apart from timings and energy rows |

**Predictions — lab tests:**

| # | Prediction |
|---|---|
| L1 | **The heater fires** (reference build, lamp failed over the coldest 5 days, 113–117, mean 0.04 °C): it draws **1.4574e7 J** (mean 33.7 W) within 1e-3; at the window's end the battery is **+4.3026e7 J** above the nominal run (the lamp's 5.76e7 J not drawn, less the heater); the chamber dips at most **0.0146 K** below 22 °C (`wall·dt/C_ch`) and never further. Control: the same failure with the heater's capacity at 0 — the chamber falls toward outdoors (liveness: the dip bound goes red), and the crop is byte-identical to the heated run (the plants read no chamber temperature) |
| L2 | **Cabin** at 18 °C: 6.22 W out; at 27 °C: **7.59 W in** — the cooler takes it, the chamber stays held. The flow's sign and size pinned at both, away from the zero-flow point |
| L3 | **Space**, ε* = 0.01 (WHAT-IF): 22.05 W out at 295.2 K |
| L4 | **Station structure**, 1220 days: the heater runs continuously at **44.8 W**; the node settles near **179.15 K**; the battery, which nominally falls to 5.946e9 J (the sealed power budget pays life support, not the lamp — §6 of the lamp-shed record), ends near **1.22e9 J** — above zero, so no rationing within the horizon, barely |
| L5 | The time base: the outdoor temperature read on the fast step equals the plants' for the same slow `n`, at the first and last minute of a day |

### 23j. 2b-iii BUILT — graded (2026-10-05)

Built as §23i describes: `ChamberWall`, `ChamberHeater` and `ChamberSurroundings` in
`rust/crates/station/src/chamber.rs`; `build_sealed_station_in` (the surroundings option) and
`outdoor_temperature` (the time base) in `sealed.rs`; three parameters in `chamber.yaml`; lab
tests `rust/crates/station/tests/chamber_walls.rs` (four, plus two full-horizon ones behind
`--ignored`, run for this grading). 2b-ii's golden kept at
`W:\temp\claude\chamber-heat\sealed_2bii.json`; run outputs in
`W:\temp\claude\chamber-heat\run_2biii\`.

| # | Result |
|---|---|
| R1 | **HELD** — against 2b-ii's golden only `thermal.node`, `thermal.chamber`, `boundary.space` differ, plus the added `boundary.chamber_surroundings`; `power.battery`, `boundary.solar_source`, every other stock and every aux value byte-identical — the heater never fired |
| R2 | **HELD** — `boundary.chamber_surroundings` **1.841730e9 J** against 1.841731e9 (5.4e-7) |
| R3 | **HELD** — node end **174.14080 K** (predicted 174.14086); daily min **172.1720**, max **174.8881**, mean **173.24720** against 172.169 / 174.889 / 173.247 |
| R4 | **HELD, to 7 figures** — chamber end **295.1999296 K**; daily range **295.1886868–295.2034234 K**, the re-simulation's exactly |
| R5 | **HELD** — `boundary.space` fell **1.833528e9 J** against 1.833530e9 (1.1e-6) |
| R6 | **HELD** — `regen_goldens`: only `sealed_station`; manifest diff = + `ChamberHeater`, + `ChamberWall`, `chamber.yaml`'s digest, the golden hash |
| R7 | **HALF** — the seven examples against 2b-ii's outputs (`W:\temp\claude\chamber-heat\run_2biii\` vs `…\examples_2bii\`), timings stripped: `air_split`, `air_split_baseline`, `watering`, `minute_transpiration` identical; `draw_census` differs in the energy rows only (`thermal.chamber` worst 0.000195, on day 197.9). ✗ Two differences I did not foresee: **(1)** `lamp_shed`'s blackout run ends with **3.9 MJ less battery** (1.8219e7 → 1.4355e7 J; crop, shed timing and shares identical): once the lamp is shed the chamber cools and the **heater draws on the very battery the shedding protects** — about a fifth of what was left. A finding about the design, recorded, not changed here — **the user then chose to shed the heater with the lamp (§23k), built in §23l–§23m: the 3.9 MJ is back.** **(2)** `intraday_exchange`'s day-order comparison now moves 3 energy stocks: the walls read the outdoor temperature by `n`, and the fast minutes after plant step `k` carry `n = k + 1`, so under the retired slow-first order they read the next day's weather all day, under the reference order for the last 90 minutes of each day — the one-plant-step lead every fast-side forcing keyed on `n` already has (L1's lamp window). Documented at `outdoor_temperature`; not special-cased |
| L1 | **HELD** — the heater drew **1.45653e7 J** (predicted 1.4574e7, −0.06 %); battery vs nominal **+4.30347e7 J** (4.3026e7, +0.02 %); chamber within 0.015 K under 22 °C on every day-end inside the window; without the heater it fell below 280 K; crop byte-identical between the two. ⚠ **Found, not predicted — an instrument lag:** the fast operator runs after its group's plant step has advanced `n`, so a lamp window keyed `[113·16, 118·16)` goes dark in the **last 90 minutes of day 112** and comes back 90 minutes before day 118 ends. The test first compared at `states[113]` and went red on its own control; it now compares at `states[112]` and asserts the lag is where it says (a red there means the lag moved). The same one-plant-step lag the lamp-shed record carries for the crop's light |
| L2 | **HELD** — cabin 18 °C: **6.2221 W** out; 27 °C: **7.5934 W in**; chamber held (295.201 / 295.206 K) |
| L3 | **HELD** — space, ε* = 0.01: **22.047 W** at 295.2 K (unit test) |
| L4 | **HELD** — 120 days on the structure: node **179.1486 K** (179.151), heater **44.817 W** (44.84); the full 1220 days: battery ends **1.2163e9 J** (≈ 1.219e9), no rationing |
| L5 | **HELD** — the walls read the plants' day at the first and last plant step of seven days, bit for bit. Liveness: indexing the table by the fast `dt` instead turns it red at day 1 |

**Tally: R1–R6 held, R7 half (two unforeseen example differences); L1–L5 held, L1 with an instrument lag found on the way.**

**Not predicted — an existing test went red.** `tests/chamber_heat.rs` (2b-i/2b-ii) asserted a
flat nominal chamber and a linear dead-cooler rise; with walls the nominal chamber follows the
weather (R4 itself says so) and a dead cooler's chamber climbs toward its walls' equilibrium.
The roster in §23i should have listed it. Fixed by switching the walls **off** in that file (scaled
by 0, as its cooler is), so it keeps isolating the cooler; the walls have their own file.

**What B now shows, end to end.** Hot direction: a dead cooler heats the chamber (toward
`T_out + 133.3 W / UA` ≈ 87 K above outdoors with walls). Cold direction: a dead lamp on cold
days pulls the chamber toward outdoors at a 27 h time constant, and the battery heater holds it.
A radiator loss reaches it after ≈ 31 days through the cooler's second-law gate.

### 23k. Two decisions TAKEN (user, 2026-10-05)

1. **The heater when the lamp is shed:** *"Cut it with the lamp"* — the heater is an
   interruptible load, like the lamp (the shedding rule's source, Chung & Mazzocco, splits loads
   into interruptible and uninterruptible). Owed: the lab's lamp shedding sheds the heater too,
   predictions first. The reference has no shedding, so no golden moves.
2. **Which temperature the plants read in slice 3:** *"The chamber (as planned)"* — over keeping
   the outdoor weather for calibration and over a switchable option. Slice 3 (the plants read
   the chamber, with the cited cold period) is next after item 1, with its own design and
   predictions.

### 23l. The heater shed with the lamp (lab) — design and predictions, before code (2026-10-05)

§23k item 1. Lab only: the reference has no shedding, so nothing frozen moves.

**What is built.** `station::lamp_shed`'s switch-off wrapper covers **both** interruptible loads,
`Lamp` and `station.chamber_heater`, at the same reserve and on the same battery reading;
`rewire_for_shedding` refuses a fast registry missing either. The wrapper is renamed
`SheddingLamp` → `SheddingLoad` (nothing in code pins the old name). The lit detector and the
crop's light are untouched — the heater gives heat, not light.

**Which reading of the decision this is.** The heater is cut by the **battery rule**, not by the
lamp being dark: it is an interruptible load *like* the lamp (Chung & Mazzocco's split), not a
load slaved to it. So a lamp that fails with a healthy battery (`with_lamp_power_cut`) still has
its heater, which holds the chamber.

**Predictions — `tests/lamp_shed.rs` fixture (1.5e8 J battery, solar ×0 over days 2–5, 8 days):**

| # | Prediction |
|---|---|
| H1 | The lab blackout run equals, **in every stock and aux value on every day, bit for bit**, the same lab run with the heater disabled (wrapped at health 0). Why exactly: while the lamp is lit it holds the chamber above its setpoint, so the heater gives zero legs; the lamp and the heater are shed on the same step (same reading, same reserve); and on this station a shed lamp never comes back (§6 of the lamp-shed record), so the heater never restores |
| H2 | Liveness of H1: the dark run's chamber ends **more than 3 K** below its 22 °C setpoint (estimate ≈ 8–9 K: the unshed heater drew 3.9 MJ over the 3.375 dark days, 13.4 W mean, which at the walls' 1.536 W/K means outdoors averaged ≈ 13 °C, and 3.375 days is 3 of the walls' 27 h time constants) — so an unshed heater would have drawn |
| H3 | The `lamp_shed` example's "lab shed, blackout" battery returns from **1.4355e7** to **1.8219e7 J**, its value before the walls (2b-ii); crop, first-shed group and share unchanged |
| H4 | The battery rule, not the lamp: with only the lamp's power cut over days 2–5 and the frozen battery (never near the reserve), the heater **draws** — the battery ends lower than the same run with the heater disabled, by ≈ **3.5 MJ** (13.4 W × 3 days; ±20 %, the window is earlier in the season than H2's) |
| H5 | Every existing test in `tests/lamp_shed.rs` and `tests/chamber_walls.rs` stays green unchanged (`chamber_walls` never calls the rewire; the lamp-cut run never crosses the reserve; no test pins the blackout battery). `regen_goldens` report-only: nothing moves |
| H6 | Mutation: wrapping the lamp alone turns H1's test red (it would differ by the 3.9 MJ) |

**Untested, recorded (advisor):** on a station whose battery *does* recover, switching back on
brings up to 200 W of re-warming heat along with the lamp (the chamber cooled while dark), and
the rule has no latch — so the restore can push the battery straight back under the reserve and
chatter. The sealed fixture cannot reach it; whoever builds a recovering battery owns that check.

### 23m. The heater shed with the lamp — BUILT (lab), graded (2026-10-05)

Built as §23l describes: `SheddingLoad` and `INTERRUPTIBLE_LOADS` in
`rust/crates/station/src/lamp_shed.rs`; two tests in `rust/crates/station/tests/lamp_shed.rs`
(`a_shed_heater_is_the_heaterless_run_bit_for_bit`, `a_lamp_failure_with_a_healthy_battery_keeps_its_heater`).
Outputs in `W:\temp\claude\heater-shed\`.

| # | Result |
|---|---|
| H1 | **HELD** — the blackout run equals the heater-disabled run in every stock and aux value on all 9 day-ends, and in the shed log, bit for bit |
| H2 | **HELD** — the dark chamber ends at **286.348 K, 8.80 K** below its setpoint (estimate ≈ 8–9 K) |
| H3 | **HELD** — the example's "lab shed, blackout" battery is **1.8219e7 J** again (was 1.4355e7); crop −31.725 %, first shed in group 74, share 0.5794 — all as before |
| H4 | **HELD** — with only the lamp's power cut and a healthy battery, the heater drew **3.566e6 J** (predicted 3.5e6 ±20 %, +1.9 %) |
| H5 | **HELD** — the nine existing `lamp_shed` tests and the four `chamber_walls` tests green unchanged; `regen_goldens` report-only: 20 of 20 identical; clippy clean; full suite **1306 passed, 0 failed** (`--no-fail-fast`) |
| H6 | **HELD** — exempting the heater from the rule turned exactly H1's test red, the other ten green |

**Tally: H1–H6 held.** One harness slip on the way, not a finding: the first H1 test reused
`assert_same_run`, which strips the lab's bookkeeping slot from one side only (it compares a lab
run against a plain one), so it went red at day 0 on identical runs; it now compares the two lab
runs directly.

## 24. Slice 3 — the plants read the chamber, with the cited cold period: design and predictions, before code (2026-10-05)

§23k item 2, the user's call: the plants read **the chamber**. §10's slice 3, with §12's cold
period (8 weeks at 4 °C, the lamp dimmed to 100 µmol m⁻² s⁻¹ for an 8 h day). The 305-day
calendar is **kept** (slice 4 re-sows on maturity). **No code in this section.** A station
unfreeze; the biosphere is untouched (the plants' formulas do not change, only what they read).

### 24a. Advisor review (2026-10-05), summarized

1. **The mechanism is a station-side environment wrapper** (the `OnFastStep` / `LampLitFlow`
   precedent). Wrap inside `build_sealed_station_in`, **after** `gas_exchange_on_fast_step`, so the
   plant-window recorder is wrapped too and every caller inherits it. Keep the inner `type_name`
   and forward `priority()` (as `OnFastStep` does — **not** `LampLitFlow`'s rename: the manifest's
   flow set is keyed on type names). **Remove `TEMP_VAR` from `sealed_bio_resolver`** so an
   unwrapped reader errors loudly, and refuse a lab-inserted one rather than ignore it. The
   minute-step carbon budget reading the plant step's recorded value is fine for this slice. State
   the start values (the re-simulation assumes them).
2. **The roster: days 0–55 are where the lab fixtures live.** Record each sealed fixture's start
   and horizon before running; classify all-cold / crosses day 56 / warm only. Controls that hold
   only because the plants read no chamber (`chamber_walls` L1, `chamber_heat` P9) **invert by
   design** — rewrite them as the opposite prediction with a direction; they become this slice's
   liveness. Per fixture whose subject is not the cold: opt out (no cold phase) or re-derive, with
   the reason written down — **not all opted out to stay green**.
3. **Split the golden diff in two.** The dimmed lamp and 8 h day are separable from reading the
   chamber; the cold setpoint is not (slice 1 shows the warm chamber arrests the crop).
4. Commit the predictions from the validated instrument, conditioned where they depend on water.
5. Strengthen "re-sow cannot fail" beyond a grep — done, §24e.

**Second pass, on the written section (same day), summarized:** (1) opting every `lamp_shed`
test out of the cold would leave §24i's delivered-share fix with no liveness — keep the rule-off
test in the cold, opt out only the heater and (in 3b) reserve-crossing tests; (2) say where the
program lives, so opting out is one edit and cannot be half-done — one scenario field; (3) A11
missed `chamber.yaml`, whose header 3a makes false; (4) write the overheated-crop test's
prediction and fallback now; optional: check the perturbation tests for magnitude thresholds.
All five taken (§24c, §24f A11, §24h, §24i).

### 24b. How the plants read the chamber

* **`PlantsReadChamber`** (station-side): wraps each plant-step flow and aux process. Its
  environment answers `TEMP_VAR` as `Q_chamber / C_ch − 273.15` (°C) from the snapshot it is
  evaluated on, and delegates every other variable. Inner `type_name`, `id` and `priority`
  forwarded, so no manifest key moves for the wrapper.
* **Where.** Once, in `build_sealed_station_in`, on the plant registry **after**
  `gas_exchange_on_fast_step` — so the `PlantWindowRecorder` records the chamber, and the
  minute-step carbon budget (`OnFastStep`) reads that recorded value through its window: the
  chamber at the opening of each 90-minute window, the resolution the light already has. Live
  minute-by-minute temperature for the carbon budget is a possible later refinement; it differs
  only while the chamber moves (a warm-up, a fault).
* **`TEMP_VAR` leaves `sealed_bio_resolver`.** The walls keep reading the weather through
  `outdoor_temperature` (unchanged — it builds its own table). **The guard:** the wrapper's
  environment first asks the inner environment for `TEMP_VAR`; if the inner one answers, it
  **errors** ("the plants read the chamber; a temperature wired on the plant side would be
  silently ignored"). This turns three silent failures loud: a lab resolver that holds the plants
  at 22 °C (`warm_room_arrest`, the `watering` and `minute_transpiration` examples and tests); an
  unwrapped reader; and **a build that moves flows after wrapping** — `air_split` builds a
  plant-step station and moves the carbon budget itself (`air_split.rs:419`), so a wrapped flow
  inside `OnFastStep` would meet `WindowEnv`, which answers `TEMP_VAR` → error, rather than
  silently reading the chamber live while the reference reads the window.
* **No new stock, no new aux.** Saved sessions keep loading (the stock set is unchanged).

### 24c. The cold program

* **One home: a `SealedStationScenario` field** (advisor, second pass), beside `lamp_power_w` and
  `photoperiod_hours`, defaulting from `cold_period.yaml`. Every reader takes it from there: the
  fast setpoint and lamp forcings (`sealed_fast_resolver`), the plant light, net radiation and
  daylength (`sealed_bio_resolver`, 3b), the chamber's start (`chamber_heat0`), and the shedding
  driver's nominal draw (§24i). **Opting a fixture out of the cold is one scenario edit**
  (`cold_days: 0`), so a fixture cannot end up opted out on the plant side and not the fast side.
  Slice 4 swaps the program's clock for a sowing clock in the state, in the same place.
* **`cold_period.yaml`** (new station param file), one source for all four: Cha et al. 2022
  (*Molecular Plant* 15:1300, the AAM opened 2026-10-03): `cold_setpoint` **277.15 K** and
  `cold_days` **56** (the standard 6–10 weeks at 2–6 °C, AAM lines 40–42 — the centre, the
  user's choice §12); `cold_par` **100 µmol m⁻² s⁻¹** (their vernalization light, AAM line 384)
  and `cold_photoperiod_hours` **8** (the standard's short day). 3a carries the first two, 3b adds
  the other two.
* **The clock.** The cold phase is the first `cold_days` of each season:
  `floor(n·bio_dt) mod season_days < cold_days`. With the calendar kept, the season start **is**
  the sowing. ⚠ Slice 4 (re-sow on maturity) must move this clock into the state — recorded here so
  it is not rediscovered.
* **Fast side:** `ChamberCooling` and `ChamberHeater` read their setpoint from a new fast forcing
  (the program), not from `chamber.yaml`'s constant, which stays as the warm value (295.15 K). 3b:
  `LAMP_POWER_VAR` follows the program — the cold-phase draw is `cold_par · area / photon_efficacy`
  = 100 / 2.5 = **40 W**, × 8/24 = **13.33 W** averaged. ⚠ DESIGN assumption: a dimmed lamp keeps
  its photon efficacy (2.5 µmol/J) — no source read for dimming.
* **Plant side (3b):** `PAR_VAR`, `RN_VAR` and `DAYLENGTH_VAR` switch to the dim, 8 h lamp on the
  same clock (`lamp_light_path` / `lamp_net_radiation_path` with the cold values).
* **⚠ The time base** (§23i/§23j): the fast side keys on `n`, which leads the plants by one plant
  step. So the chamber starts to warm (or cool) in the **last 90 minutes** of the day before the
  plants' day changes. Kept, as for the walls — not special-cased. Consequences predicted below:
  the plants' first reading on day 56 is mid-warm-up, and the golden's last 90 minutes read
  day 1220 (≡ day 0 of a fifth season), so the end state's chamber is **mid-cool-down**.
* **Start values.** The chamber starts at day 0's setpoint and lamp:
  `C_ch·T_cold + lamp_avg(day 0)·τ`. The node start (`sealed_node_heat`) is **unchanged** (the full
  lamp; it has not been an equilibrium since the walls). The re-simulation assumes exactly this.

### 24d. The split — two golden diffs, one cause each

* **3a — the plants read the chamber, with the cold setpoint; the lamp unchanged** (full, 16 h,
  all year). The cooler's 200 W holds 4 °C against the lamp's 133.3 W plus the walls' ≈ 11 W
  inflow (outdoors is 5.4–16.4 °C over days 0–55, always warmer than the chamber).
* **3b — the cold phase's lamp:** 100 µmol, 8 h. The daylength factor 0.28
  (`1 − 0.09·(16 − 8)`, Eqn 7.6) and the ≈ 2.3 GJ battery change belong to this diff alone.

### 24e. The instrument, and its controls

`W:\temp\claude\slice3\resim.py` and `endstate.py` — an independent Python re-simulation (not the
Rust code) of the chamber, node and battery on the 60 s step over 1220 days, with the plants'
clock (thermal time + chill-days, Eqns 7.4/8.3/8.6/7.6) read off the chamber at each plant-step
opening. Outputs beside it (`ref.txt`, `slice3a_56.txt`, `slice3_56.txt`, `endstate.txt`).
**Controls, run in `ref` mode against the frozen golden:**
* the heat books: node end **174.14080 K**, daily min/max/mean **172.1720 / 174.8881 / 173.24720**,
  chamber end **295.1999296 K**, surroundings **1.841730e9 J** — §23j's graded values to the
  printed digits; `boundary.space` **45 773 227 032.2669 J** against the golden's
  45 773 227 032.266945 (16 digits);
* the crop's clock: thermal time **2490.6033461672773** and chill-days **143.37249999999898** —
  the golden's end-state aux **bit for bit**. So drought-hastened development (WSSD 0.40,
  configured for this crop) never fired in today's run.

**"Re-sow cannot fail" — checked by listing every writer of `storage_c`, not by a grep pattern.**
`Allocation` (partition fraction × `available_for_growth`, which is `max(0, ·)`: ≥ 0),
`StemReserveRemobilization` (rate × reserve ≥ 0), `Harvest` (station; **off** in the reference,
`with_harvest = false`), and `reset_crop` (zeroes it at the re-sow). `organ_stock` builds it as a
POPULATION with extinction threshold **0**, so the die-off pass cannot zero it. Maintenance burns
leaf, stem and root only. So within a season grain never falls; the crop matures on day ≈ 139
(below), long before the day-305 re-sow.

### 24f. Predictions — 3a (the `sealed_station` golden, regenerated)

| # | Prediction |
|---|---|
| A1 | `boundary.solar_source` **byte-identical** to the 2b-iii golden. `power.battery` = golden − the heater: **5.939020485e9 J** (5.9456e9 − 6.5795e6), within 1e-6 relative. Every other stock moves (the plants read a different temperature) — magnitudes **not predicted** except below |
| A2 | The heater fires **once per season, at the warm-up**: **1.6449e6 J** each, 6.5795e6 J in all, within 1e-3. (Until now it never fired in the reference.) It fires nowhere else: the cold phase's walls bring heat in |
| A3 | The chamber: cold phase **4.053–4.061 °C**; warm phase as 2b-iii's (to 295.2034 K); the warm-up is ≈ 2.25 h (the plants' first reading on day 56 is **≈ 16.0 °C**, then 22.045); the re-sow cool-down ≈ 11 h (the lamp's 133 W against the cooler's 200 W; plant readings on day 305 ≈ 19.4, 12.3, 6.3 °C at 0, 4.5 and 9 h, then 4.06). End state **292.5662711 K** (mid-cool-down: the last 90 minutes read day 1220), within 1e-6 K |
| A4 | The node: end **174.18129 K**; daily min / max / mean **172.1755 / 176.2189 / 173.74488**, within 0.01 K (the walls bring heat in during the cold phase, so the node runs warmer then) |
| A5 | `boundary.chamber_surroundings` **1.308367e9 J**, `boundary.space` **46 310 460 294.6 J**, within 1e-5 relative |
| A6 | Chill-days at the end **55.712671868** (the plants' first readings after the day-305 re-sow are above 12 °C while the chamber cools). **Exact to 1e-9 relative whatever the crop does** — vernalization reads only the temperature, and the chamber's heat books do not read the crop |
| A7 | Thermal time at the end **5573.6508642** — **if the crop is never water-stressed**. Any excess means drought-hastened development fired: that is the detector for the open water question (§8 item 3), not a miss to tune away |
| A8 | Flowering / maturity (days after sowing, unstressed): season 1 **102.0625 / 136.0625**, seasons 2–4 **102.125 / 136.125**. Today's crop: 219.25 / 266.31. So ≈ 117 days earlier, then ≈ 169 days standing until the calendar re-sows |
| A9 | The run completes: every re-sow finds grain ≥ the seedling's 0.16 (§24e). `rationed == 0`, no events |
| A10 | Water: during each cold phase the plants' vapour store ends every plant step at or under the 4.06 °C ceiling, **1.035 kg** (75 % of saturation, 9500 mol); in the warm phase under **3.359 kg**. Up to ≈ 2.3 kg condenses in the hours after each re-sow (if the store was at its warm ceiling). Transpiration and the crop's carbon: **direction not predicted** (hotter, drier air in the warm phase against a crop that matures 117 days earlier) |
| A11 | Goldens: only `sealed_station` moves (19 of 20 identical). Manifest: + `cold_period.yaml`'s digest, **`chamber.yaml`'s digest** (its header is edited, §24i), the golden hash; **no** flow-set change (the wrapper keeps type names) |

### 24g. Predictions — 3b (on top of 3a)

| # | Prediction |
|---|---|
| B1 | `boundary.solar_source` byte-identical. `power.battery` = the 2b-iii golden + the lamp saved − the heater: **8.261426e9 J** (5.9456e9 + 2.322432e9 − 6.6060e6), within 1e-6 relative |
| B2 | Heater **1.6515e6 J** per warm-up (6.6060e6 in all), within 1e-3 |
| B3 | Chamber: cold phase **4.005–4.013 °C**; the plants' first reading on day 56 **≈ 15.97 °C**; the re-sow cool-down ≈ 4 h (plants read ≈ 15.2 °C at the first step of day 305, then 4.01). End state **288.3622829 K**, within 1e-6 K |
| B4 | Node: end **174.18129 K**; daily min / max / mean **163.1421 / 174.3095 / 171.44761** (the dim lamp sends 120 W less heat through the cooler for 56 days a season), within 0.01 K |
| B5 | `boundary.chamber_surroundings` **1.305818e9 J**; `boundary.space` **43 991 226 753.0 J**, within 1e-5 relative |
| B6 | Chill-days at the end **55.926093595** (exact to 1e-9, as A6) |
| B7 | Thermal time at the end **5512.7606791**, if unstressed (A7's detector). The cold phase contributes only **23.62 °C·day** (4 °C × the daylength factor 0.28 × the vernalization factor, which is 0 until ≈ 19.7 chill-days) |
| B8 | Flowering / maturity, unstressed: **104.875 / 138.875** days after sowing, every season. Sensitivity (lab, not golden): 6 weeks → 109.25 / 143.31 (the factor stays 0.736 all season); 10 weeks → 118.19 / 152.19 |
| B9 | Vapour ceiling in the cold phase **1.031 kg** (4.011 °C); A9, A10's other parts and A11 unchanged (`cold_period.yaml`'s digest moves again) |

### 24h. The roster — every sealed fixture, before running (start day 0 unless noted)

| Fixture | Window | Class | Expected | Decision (why) |
|---|---|---|---|---|
| `goldens.rs` `sealed_station` | 1220 d | all phases | §24f / §24g | regenerate |
| `tests/chamber_heat.rs` | 60 d, walls off | crosses 56 | the flat-chamber claim fails; P9's "crop byte-identical under a dead cooler" **inverts**; with walls off a dead cooler drives the chamber without bound, and the plants now read it (thousands of K — outside every plant formula's range; `q10^(ΔT/10)` stays finite, the crop's maintenance does not stay plausible) | **opt out of the cold** (its subject is the cooler); keep the day-1 rate; P9's crop claim **inverted to a direction over day 1** (the overheated crop's leaf + stem + root below nominal). **Predicted for the 60-day run:** finite (every plant temperature term is a power `q10^(ΔT/10)`, at most ≈ 10¹⁷⁶ at +4 600 K; saturation vapour pressure is bounded) but **`rationed > 0` within the first few days** (maintenance doubles every 10 K and soon asks for more than the organs hold). So the test records the rationing as found, not `== 0`. **Fallback, written now:** if the run hard-errors before day 60, the 60-day node claim moves to a walls-on dead-cooler run (the chamber then bounded ≈ 87 K above outdoors), its node target re-derived with the instrument before running |
| `tests/chamber_walls.rs` L5 | 7 d | cold | premise gone (the plants no longer read the weather) | re-point: the walls read the day the plants' `floor(n·bio_dt)` names, from the weather table |
| `chamber_walls` L1 | days 113–117 | warm (after a warm-up) | heater figures unchanged in the window (both runs share the day-56 warm-up); "crop identical with and without the heater" **inverts** | **invert, with directions**: without the heater the plants read a colder chamber → **less thermal time**, and (dark lamp, no photosynthesis) **less maintenance burn → more leaf + stem + root** than the heated run |
| `chamber_walls` L2 | 3 d | cold | the wall law's sign flips: the chamber is at 4 °C, so the 18 °C cabin sends heat **in** (≈ 21.4 W), the 27 °C cabin ≈ 35.3 W in | **re-derive** in closed form (the subject — the wall law either way — is testable at the cold setpoint) |
| `chamber_walls` L4 (+ its full-horizon twin, ignored) | 120 d / 1220 d | crosses 56 | the cold chamber faces a −98 °C node: the heater runs harder (3b) or softer (3a) in the cold phase; the 120-day node and last-30-day heater figures shift by the cold phase's tail | **re-derive** with the instrument extended to the structure surroundings, before running |
| `tests/lamp_shed.rs` — the heater tests (`a_shed_heater_is_the_heaterless_run_bit_for_bit` = H1/H2, `a_lamp_failure_with_a_healthy_battery_keeps_its_heater` = H4) | 8 d, blackout 2–5 | cold | they lose their subject: the walls bring heat **in**, so the heater never fires (H1 trivially true, H2's −3 K and H4's 3.5 MJ fail) | **opt out of the cold** in 3a (their subject is the heater of a warm chamber) |
| `lamp_shed` — the reserve-crossing tests (`a_blackout_sheds_the_lamp_and_the_crop_feels_it`, `a_shed_lamp_on_the_sealed_station_never_comes_back`, `a_blackout_also_cuts_the_crops_water_loss`, `cutting_only_the_lamps_power_darkens_the_lab_crop`, `the_small_battery_alone_does_not_reach_the_reserve`, `the_plain_crop_does_not_feel_a_blackout`) | 8 d | cold | **3a: unchanged subject** — the lamp is unchanged and the heater idle, so the battery and the shed timing (group 74) are as before; crop figures move. **3b:** the dim lamp's 13 W may never bring the battery to the reserve | 3a: **keep in the cold**, re-derive crop figures; 3b: **opt out** (their subject is shedding the full lamp) |
| `lamp_shed` — `rule_off_the_lab_wiring_is_the_plain_sealed_run_bit_for_bit`, `rule_on_with_the_frozen_battery_nothing_sheds_and_nothing_moves`, `a_run_resumed_from_a_saved_day_is_the_continuous_run_bit_for_bit` | 8 d | cold | hold | **keep in the cold, both slices.** In 3b the rule-off test is the **liveness of §24i's delivered-share fix**: with the bug, the cold phase's dim lamp reads 10 % delivered, the lab crop is darkened twice, and the rule-off run differs from the plain one |
| `tests/perturbations.rs` matter | 8 d, window 2–7 | cold | the sign claims (biomass down, effort up, battery spared) and `assert_returns_to_setpoint` (an exact regulator identity, `rel < 1e-6`, independent of temperature) predicted to **hold**. ⚠ One comparative magnitude, the O₂-leak test's `|carbon| > |O₂|`, is at risk: at 4 °C the crop's fluxes shrink | **keep in the cold** (their subject is the response, not the temperature); if the comparison goes red, that is a finding about the cold crop — re-derived, not loosened |
| `tests/air_split.rs` | 90 d | crosses 56 | builds through `build_sealed_station_at` (plant step) and moves the carbon budget itself → the §24b guard **errors**; its ratios (0.144, 0.999, …) shift | apply the wrapper after its own move; **keep the cold** (both arms run the same program); **re-derive** its figures |
| `tests/gas_exchange.rs` | units | — | `a_change_to_the_plant_side_temperature_is_what_gets_recorded` inserts `TEMP_VAR` → refused | re-express: a change to the **chamber** is what gets recorded |
| `tests/minute_transpiration.rs` | units | — | `plant_resolver(held_22)` inserts `TEMP_VAR` → refused | re-express through a chamber held at 22 °C |
| `tests/lamp_net_radiation.rs` | spot days | cold days, in 3b | the sealed crop's net radiation is the **dim** lamp's in the cold phase (3b) | 3b: **re-derive**, and assert both phases |
| `tests/warm_room_arrest.rs` | 1 season | — | its 22 °C resolver is refused | re-express as **a program with no cold phase** (`cold_days` 0); it stays the control for the cold period; its 8 °C liveness becomes a cold program held all season |
| `tests/session_parity.rs`, `session_save_load.rs`, `day_order.rs` | short + ignored full | all | self-consistency (one build two ways) — **hold** | keep; run the ignored ones for grading |
| examples `air_split`, `air_split_baseline` | season | crosses 56 | `bound.get(TEMP_VAR)` → **crash** (intended) | re-point their temperature readout at the chamber |
| examples `watering`, `minute_transpiration` | season | — | 22 °C inserts refused | re-express via a warm chamber; their records were made at 22.0 °C, a held chamber reads 22.05 °C → small shifts, recorded |
| examples `draw_census`, `intraday_exchange`, `lamp_shed` | various | all | `draw_census`'s byte-exact control passes against the regenerated golden; `lamp_shed` as its tests | run all seven before and after, as §23f |
| Godot sealed session (`palette.rs`, `godot_bridge`) | — | — | inherits the build; no stock-set change, so saves still load | none |

### 24i. Traps found while designing

* **`run_shedding`'s delivered share** (`lamp_shed.rs`) divides the lamp's draw by
  `lighting_average_power(scenario)` — the constant full lamp. In 3b's cold phase a healthy dim
  lamp would read **10 % delivered**, and `LampLitEnv` would darken the crop a second time. 3b must
  take the nominal from the lamp program (the unperturbed schedule). Lab only; silent otherwise.
* `with_lighting_failure` and `with_lamp_power_cut` **wrap** the existing schedule
  (`window_override(base, …)`), so they compose with the program. Checked, no change.
* `build_sealed_station` still omits `vernalization_days` at the first sowing (slice 1's finding):
  read as 0, inert, unfixed here — seeding it would add a second cause to the diff.
* **Prose 3a makes false — corrected in 3a's commit, not left:** `chamber.yaml`'s header ("THE
  PLANTS DO NOT READ THIS TEMPERATURE YET"; `setpoint` as *the* held temperature — it becomes the
  warm-phase one), so its digest moves (A11; the `lamp.yaml` keep-it-stale precedent was a
  comment on a file this change does not otherwise touch); `chamber.rs`'s module doc ("The plants
  do not read this temperature yet"); `sealed.rs`'s module doc ("the biosphere registry is
  `build_season` verbatim" — now wrapped); `docs/station-reference.md`'s description of what the
  sealed station's plants read.

### 24j. Decisions owed to the user

1. **Two golden changes (3a, then 3b), or one?** *Recommended: two* (advisor) — the dim lamp moves
   the battery by 2.3 GJ and the crop's clock through the 0.28 daylength factor, and in one diff
   neither could be told apart from the chamber reading.
2. Defaulted unless overridden: the dimmed lamp keeps its efficiency (40 W for 100 µmol, DESIGN);
   the cold phase keyed to the calendar until slice 4; the minute-step photosynthesis reads the
   chamber at each 90-minute window's opening; the per-fixture choices in §24h.

**ANSWERED 2026-10-05 (the user):** (1) *"two changes — ok"*: 3a, then 3b. (2) The dimmed lamp's
efficiency: *"it depends on the type of lamp; if we assume it is an LED dimmed by PWM, then it
keeps its efficiency."* So the lamp is a **PWM-dimmed LED** and keeps 2.5 µmol/J when dimmed — still
**DESIGN** (the user's reason, the physics of PWM: the LED runs at its rated current for a shorter
share of each cycle), not a citation; no source read. It matters in 3b only. The other defaults
stand.

**Checked before code (advisor, 2026-10-05):** the sealed plant resolver's only other
weather-derived key is `VPD_VAR` (the outdoor deficit). The reference reads the chamber's own
(`VpdRead::Chamber`, `biosphere/params.rs`), so it is unread there; left in place and recorded,
not removed (a second cause). No display / Godot code reads `"temp"`.

### 24k. Slice 3a BUILT (2026-10-05, a station unfreeze) — the predictions graded

**What landed.** `station/params/cold_period.yaml` (new: `cold_setpoint` 277.15 K, `cold_days` 56,
both CITED [Cha 2022], class); `SealedStationScenario::cold` (`ColdProgram`) with **one clock**,
`is_cold_on_step` / `chamber_setpoint_on_step` (`scenario.rs`); `ChamberParams::setpoint` renamed
`warm_setpoint` and `cooling()` / `heating()` / `chamber_heat0()` given the setpoint as an argument
(advisor: every old reader became a compile error — three, all in tests); the cooler and heater
read the fast forcing `CHAMBER_SETPOINT_VAR` (no fallback: a resolver without it errors);
`chamber::plants_read_chamber` (`PlantsReadChamberFlow` / `PlantsReadChamberAux`, type names, ids
and priority kept) applied **last** in `build_sealed_station_in`; `build_sealed_station_unread`
(public) for a build that moves plant flows itself (`air_split`, `tests/gas_exchange.rs`), which
wraps after its moves; `TEMP_VAR` removed from `sealed_bio_resolver`. Prose 3a made false,
corrected: `chamber.yaml`'s header, `chamber.rs` / `sealed.rs` module docs, `station-reference.md`
(ten param files).

**Instrument:** the §24e re-simulation, plus a temporary day-by-day example (not committed;
grader `W:\temp\claude\slice3a\grade.py`, output `grade.txt`). The golden diff:
`W:\temp\claude\slice3a\compare_golden.py`.

| # | Prediction | Measured | Grade |
|---|---|---|---|
| A1 | `solar_source` byte-identical; battery 5.939020485e9 | identical; **5.939020484818e9** (3e-11) | HELD |
| A2 | heater 1.6449e6 J per warm-up, 6.5795e6 in all | **1.64488e6** each season; **6.57950e6** | HELD |
| A3 | cold 4.053–4.061 °C; warm to 295.2034 K; day 305's first readings ≈ 19.4 °C; end 292.5662711 K | day-ends **4.0542–4.0608 °C**; warm **295.1886868–295.2034234**; day-305 end 19.41 °C; end **292.5662711** | HELD |
| A4 | node end 174.18129; daily 172.1755 / 176.2189 / 173.74488 | **174.18129**; **172.1755 / 176.2189 / 173.74488** | HELD (to the printed digit) |
| A5 | surroundings 1.308367e9; space 46 310 460 294.6 | **1.3083665e9** (−4e-7); **46 310 460 294.59** | HELD |
| A6 | chill-days 55.712671868 | **55.712671868320** | HELD |
| A7 | thermal time 5573.6508642 if unstressed | **5573.6508641726** (5e-12) | HELD — **the drought detector stayed silent: the crop is never water-stressed** |
| A8 | flowering / maturity 102.06 / 136.06 (season 1), 102.125 / 136.125 after | first day-end past each: **103 / 137** in all four seasons (day resolution; A7 pins the clock) | HELD |
| A9 | completes; re-sow finds grain; rationed 0, no events | rationed **0**, events **0**; grain at the re-sows 37.70 / 37.87 / 37.87 mol | HELD |
| A10 | cold vapour ≤ 1.035 kg; warm ≤ 3.359 | cold max **1.0347**; warm max **3.3603** | **MISSED as written — the prediction's own rounding.** 3.359 was the ceiling at 22.05 °C (3.35956, truncated); the warm chamber reaches 22.0534 °C, whose ceiling is **3.36026** — the measured maximum sits on it. The cold-phase figure, computed the same way at 4.0608 °C, is 1.03466. No mechanism miss; recorded, not re-fitted |
| A11 | only `sealed_station` moves; manifest + `cold_period.yaml`, `chamber.yaml`, golden hash; no flow-set change | **19 of 20 identical**; manifest diff exactly those three lines | HELD |

The end state: 10 of 36 stocks byte-identical (the crew's books, `solar_source`, the inert fill);
the crop much smaller at the end (leaf 2.30 → 0.068 mol C: it matures 117 days earlier and stands
169 days); grain 40.37 → 37.87 mol; the plants' recorded window temperature 16.51 → 22.05 °C.

**The roster (§24h), graded on the first full run (`--no-fail-fast`, 21 reds).** Every red matched
the roster in cause except four, recorded:

1. **`chamber_walls` L1** — the roster said the heater figures were unchanged because "both runs
   share the day-56 warm-up". **Wrong:** the heaterless run's heater was scaled 0 for the WHOLE
   run, so it skipped the warm-up and the two batteries differed before the window. Fixed by
   cutting that heater from day 100 (`HEATER_CUT_FROM`); then the window figures held unchanged
   (14.565 MJ against 14.574 predicted in 2b-iii) and the inversion held: unheated, thermal time
   1365.5 against 1451.7, leaf + stem + root 14.28 against 13.53 mol.
2. **`lamp_shed::the_plain_crop_does_not_feel_a_blackout`** (and the plain half of
   `cutting_only_the_lamps_power_darkens_the_lab_crop`) — the roster expected "crop figures move"
   only in the lab runs. **The plain crop now feels a blackout through the ROOM**: the unshed lamp
   empties the small battery on day 6, the backstop rations the lamp (5 140 times), less lamp heat
   reaches the chamber, and the crop reads a colder room from day 7 (diagnosed with a temporary
   per-stock diff). Re-expressed: the plain crop's recorded LIGHT is bit-identical (the defect
   the test pins), and the chamber ends colder. Renamed `..._does_not_see_a_blackout`.
3. **`tests/gas_exchange.rs` (three more) and `minute_transpiration.rs` (two)** — the roster's
   search found tests that INSERT a plant-side temperature, not tests that READ one off the plant
   resolver to compute an expected value. Five did. Re-expressed against the chamber's
   `Q / C − 273.15`; the raw flows the minute-step pins compare against now come from
   `build_season` (unwrapped). New pin `a_wrap_before_the_move_is_refused` (the guard on the
   order the advisor named).
4. **`chamber_walls` L4 (120 days) stayed green** where the roster said "re-derive". Checked, not
   assumed: the node's relaxation time at 179 K is ≈ 10.5 days and the cold phase ends 64 days
   before day 120, so its tail is gone (e⁻⁶). Its full-horizon twin was re-derived with the
   instrument extended to the structure (`W:\temp\claude\slice3a\resim_structure.py`, run before
   the test): battery ≈ **1.6767e9** (control without the cold: 1.2163e9 against the test's old
   measured 1.219e9).

Also as found: `chamber_heat`'s dead-cooler run did not hard-error (no fallback needed); it
rationed **878** times over 60 days, and the overheated crop's leaf + stem + root was 0.093 against
0.173 mol after day 1. `lamp_shed` H1/H2 **passed** in the cold, meaninglessly (H2 compared a 4 °C
chamber against the warm setpoint) — opted out as the roster said, after which H2 reads 8.8 K and
H4 3.57 MJ, as in §23m. `air_split`'s vapour pin went red as its own comment foresaw; per phase it
holds both ways: cold phase plants **+2.16 kg** (crew store −1.94), warm phase **−2.38 kg** (crew
+2.14) — the two nearly cancel over 90 days.

**The slow (ignored) station tests, run:** all pass. `chamber_walls` L4's full horizon: battery
**1.676675658e9** against the instrument's 1.676675985e9 (2e-7); R3/R4: node **172.17546 /
176.21889 / 173.74488**, chamber day-ends 277.2042–295.2034 K with 8 between the bands (one warm-up,
one cool-down per season), end 292.5662711 K. Full suite and clippy green.

**The soil carbon halved — UNPREDICTED, then measured (same day, after the commit; the user
asked why).** §24f predicted no crop or soil magnitudes, and the golden diff showed humus 24.60 →
12.91, litter 23.20 → 6.95, microbial 10.29 → 4.33 mol C. Instrument: a temporary example (not
committed) counting carbon INTO the litter by source and season, and soil respiration, for the
new reference and for the pre-3a reference rebuilt from today's code (no cold period, plants
unwrapped and reading the weather). **Controls: both runs' end states equal their goldens byte
for byte; litter closure exact in both** (observed net = stock change).

| Per full season (mol C) | pre-3a | 3a |
|---|---|---|
| shed by the living crop (`Senescence` → litter) | 60.34 | 21.93 |
| dumped at the re-sow (residue + grain − seed) | 59.55 | 41.44 |
| **into the soil, in all** (= the crop's net production) | **119.9** | **63.4** (0.53×) |
| soil respiration (settled seasons) | 113.3 | 59.9 |
| peak crop carbon | 59.18 | 44.05 |

**Cause, measured:** the soil's income halved. The soil's own processes read no temperature
(`Decomposition`, `HumusDecomposition`, `MicrobialRespiration` carry no `temp_var` — read in
`flows.rs`), so its first-order pools settle in proportion to the input: humus 0.525× against an
income of 0.53×. Litter fell further (0.30×) by TIMING: the old crop shed late in the season, so
its leaves were still litter at the day-1220 snapshot; the new one sheds early and they have
decomposed. Grain fell only 6 % (it is filled late, and matures with the crop).
**Why the crop produces less — the phases, measured** (a second temporary instrument splitting
each season's net production = Δ crop carbon + what it shed; same controls, both end states
byte-equal to their goldens; totals 119.9 and 63.4 as above). Season 2, mol C:

| Phase | pre-3a | 3a |
|---|---|---|
| days 0–55 | 13.33 (0.238/d; outdoors) | **0.47** (0.008/d; 4 °C, a seedling) |
| warm, before maturity | 87.49 over 210 d (0.417/d) | 39.44 over **80 d** (**0.493/d**) |
| after maturity | 19.07 over 39 d (0.489/d) | 23.46 over 169 d (0.139/d, dying back) |

⚠ **This CORRECTS a claim made first** (in the reply to the user and here): that production per
growing day was "about the same, so fewer growing days" — a soft denominator, which counted the
56 cold days as growing days (advisor). Measured, the 3a crop grows **18 % faster** per warm day
before maturity (22 °C against the weather's cooler days), but almost nothing in the cold weeks
and for 80 warm days instead of 210; the shorter window outweighs the faster rate. Still not an
experiment that removes a cause — a decomposition of where the carbon was made.

**The golden's other unpredicted moves, read against the same causes** (from the code, not
isolated): `rooted_depth` 1.300 → 1.257 m — roots stop at flowering (`science::extension_rate`),
which now comes at ≈ day 102 with 56 cold days in it, short of the 1.3 m reached before;
`subsoil_water` +23 % — water below the roots stays out of reach as roots stop shallower;
`water_vapor` +41 % and `condensate` +27 % — a warmer chamber holds more vapour (A10's ceiling)
and cycles more through the air; `soil_water` −4.5 %; `plant_n` −34 % — the end-state crop is a
smaller, dying-back one, and its nitrogen follows its tissue. None of them points to another
cause. **Lesson:** a prediction table that
grades only the quantities it names cannot see a large move elsewhere in the same golden — the
diff of every stock should be read against the prediction BEFORE `--write`, not after.

**Not removed (recorded):** `VPD_VAR` (outdoor deficit) stays in the sealed plant resolver, unread
under `VpdRead::Chamber`. **Next:** 3b, the cold phase's dimmed 8 h lamp (§24g), on top of this.


### 24l. Slice 3b — before switching the lamp: the plumbing's control, the roster revised, the predictions completed (2026-10-05)

**Advisor review before code (2026-10-05), summarized.** (1) Build in two stages: the plumbing
first with the cold lamp set EQUAL to the warm one (500 µmol, 16 h), which must reproduce 3a byte
for byte, then the switch — the only way to show the three forcings turned into schedules add no
second cause. (2) Rename the lamp's average-draw helpers so every reader re-decides between the
FULL lamp (the node's start, two warm-window tests) and the PROGRAM at step `n` (the fast draw,
the chamber's start, the shedding nominal). (3) List every plant-side reader of light, net
radiation and daylength first. (4) Check no caller passes a modified lamp efficacy. (5) See the
§24i fix go red once. (6) The `lamp_shed` reserve-crossing tests: a cold period with the FULL lamp
keeps them in 3a's world exactly, where `cold.days = 0` would warm the chamber and wake the heater
(a second cause). (7) Write the missing predictions — identical stocks, the crop's and soil's
direction, `rationed == 0` at risk — and diff every golden stock before `--write`. (8) Grade the
vapour ceiling at the MEASURED cold temperature (A10's miss). (9) Provenance: the 100 µmol has a
locus problem (below). All taken.

**What the build does.** `ColdProgram` gains `par` and `photoperiod_hours` (`cold_period.yaml`:
`cold_par`, `cold_photoperiod_hours`). One clock, four new readers of it on
`SealedStationScenario`: `full_lamp_par`, `lamp_par_on_step`, `photoperiod_hours_on_step`,
`lamp_power_on_step` (cold: `par · ground_area / photon_efficacy`, the PWM DESIGN) and
`lamp_average_power_on_step`. Plant side: `PAR`, `RN` and `daylength` switch on the clock. Fast
side: `lamp_power` follows the program. The chamber starts at the program's step-0 lamp. Renamed
so each reader re-decided: `lighting_average_power` → `full_lamp_average_power`,
`chamber_heat_input_w` → `full_lamp_heat_input_w` (the node's start and the two warm-window tests
keep the full lamp). `run_shedding`'s nominal is the program's draw at the same `before.n` as the
draw it is divided into (§24i). Readers checked: `daylength` feeds only the development clock
(`ThermalTimeAccumulation`); `PAR` only photosynthesis (and the window recorder); `RN` only
transpiration. No test or example passes a modified `photon_efficacy`; the fast side reads
`lamp.yaml`'s, written into its doc comment.

**Stage 1, measured (cold lamp = warm lamp):** `regen_goldens` **20 of 20 identical**; the full
suite green except `station::manifest_writer` (the yaml's digest — expected). The plumbing is
inert.

**The 100 µmol — locus, read on the page (`W:\temp\claude\cold-period\cha2022.txt`).** Line 384's
100 µmol m⁻² s⁻¹ is Cha et al.'s OWN speed-vernalization light (22-h-light at 8 °C, one institute,
RDA Korea), not the standard protocol's: that sentence (lines 40–42) gives no number, only "low
light intensity", and their Supplementary Fig. 3B records 900 and 1300 µmol at the other two
institutes during vernalization. So the value is CITED with a ⚠ LOCUS (a different protocol's
light), the user's §12 choice standing. The 8 h is the standard protocol's own (lines 40–42).

**Roster revisions for 3b (deviations from §24h, with reasons):**

| Fixture | §24h said | Now | Why |
|---|---|---|---|
| `lamp_shed` reserve-crossing six | opt out (`cold.days = 0`) | a cold period with the **full** lamp (`cold.par` = the full PAR, 16 h) | the warm opt-out wakes the heater (shed with the lamp) — a second cause; the full-lamp cold period is stage 1's world, so these runs are 3a's **bit for bit** by construction |
| `lamp_shed` rule-off / frozen battery / resume | keep in the dim cold | as §24h | the rule-off test is the §24i fix's liveness: mutated back to the constant nominal it must go **red** (a healthy dim lamp reads 10 % delivered) |
| `warm_room_arrest` | — | unchanged | its warm run has no cold period (3b cannot reach it); its plain run is the reference, which still vernalizes and fills grain |

**Re-derived before running** (the instrument `W:\temp\claude\slice3b\resim.py`, §24e's extended
to the structure; controls in 3a mode reproduce the figures those tests pass with today: node at
day 120 **179.14648 K**, heater **44.8425 W**, full-horizon battery **1.676676e9 J**):

| # | Fixture | 3b prediction |
|---|---|---|
| C1 | `chamber_walls` L2 (cold, 3 d), closed form with `P` = 13.333 W | **277.1639 K / −21.483 W** (18 °C cabin), **277.1695 K / −35.298 W** (27 °C) — out of the test's 2e-3 W band, so re-pinned |
| C2 | `chamber_walls` L4, 120 d | node **179.14647 K**, heater **44.8426 W**: unchanged at the test's resolution (the cold phase's tail is gone by day 120, as in 3a) |
| C3 | L4 full horizon (ignored) | battery **1.680820e9 J** (the dim lamp's 2.32 GJ saved is spent again by the heater against the cold node; net +4.1 MJ) |
| C4 | R3/R4 (ignored) | node **163.1421 / 174.3095 / 171.44761** (= B4); chamber day-ends cold band **277.1562–277.1628 K**, warm **295.1887–295.2034 K**, **8** between the bands (4 warm-ups, 4 cool-downs at ≈ 288.4 K), end **288.3622829 K** |
| C5 | `lamp_net_radiation` | cold days 0 and 40: **0.77 × 100 / 4.57 = 16.849 W m⁻²**; warm days 150 and 250: 84.245; the failure test's recovered day 6 is cold, so dim |
| C6 | `perturbations` matter (8 d, cold, now dim) | the signs hold (biomass down under a carbon leak and a lighting failure, UP under an O₂ leak; effort up; battery spared). The O₂ test's `|carbon| > |O₂|` is at risk again, less light, smaller fluxes — if red, a finding about the dim cold crop, re-derived not loosened |

**The golden (`sealed_station`), beyond §24g's B1–B9:**

| # | Prediction |
|---|---|
| B10 | **The same 10 stocks byte-identical to the 3a golden** (`chamber_inert`, `brine`, `fecal_waste`, `loss.carbon`, `n_source`, `solar_source`, `food_store`, `water_store`, `cabin_h2o`, `recovered_water`): none reads the crop, the lamp or the chamber. `thermal.chamber` = 1.5e5 × 288.3622829 = **43 254 342.4 J** (B3) |
| B11 | Direction, not magnitude: the cold phase's net production falls below 3a's **0.47 mol C** per season — one-tenth the photons (a fifth the PAR for half the hours), the same 4 °C maintenance — and **may go negative**. The crop then enters the warm phase smaller, so its season production is **below 3a's 63.4 mol C** (partly offset by 2.75 more warm days to maturity, B8); soil income falls, so `humus_carbon`, `litter_carbon` and `microbial_carbon` end **below** 3a's 12.91 / 6.95 / 4.33; `storage_c` **≤** 3a's 37.87 |
| B12 | ⚠ **A9 (`rationed == 0`, no extinction events) is at risk**, stated before running: a seedling at 4 °C under 100 µmol for 8 h may not cover its own upkeep for 56 days. If the backstop fires, that is a finding about the cold protocol's light on THIS crop — recorded, not tuned |
| B13 | B9's vapour ceiling is graded at the **measured** cold maximum, not at 4.011 °C |
| B14 | Manifest: `cold_period.yaml`'s digest and the golden hash only; no flow-set change; 19 of 20 goldens identical |

### 24m. Slice 3b BUILT (2026-10-06, a station unfreeze) — the predictions graded

**What landed.** `cold_period.yaml` + `cold_par` 100 µmol m⁻² s⁻¹ (CITED, ⚠ LOCUS — manuscript
lines 318–319 are Cha's own speed-vernalization arms; the standard sentence names no number) and
`cold_photoperiod_hours` 8 (CITED, the standard protocol's short day); `ColdProgram { par,
photoperiod_hours }`; the four lamp readers on the one clock (§24l); the plant resolver's `PAR`,
`RN` and `daylength` and the fast `lamp_power` on the program; the chamber's start at the
program's step-0 lamp; `full_lamp_average_power` / `full_lamp_heat_input_w` (renamed: the node's
start keeps the full lamp); `run_shedding`'s nominal from the program. Unit pin
`the_cold_lamp_is_forty_watts_for_eight_hours`.

**Instrument:** `W:\temp\claude\slice3b\resim.py` (§24e's, extended to the structure) for the
predictions; a temporary day-by-day example (not committed; output
`W:\temp\claude\slice3b\grade.txt`) and `W:\temp\claude\slice3a\compare_golden.py` for the grades.

| # | Prediction | Measured | Grade |
|---|---|---|---|
| B1 | battery 8.261426e9 | **8.261426027e9** (3e-9); `solar_source` identical | HELD |
| B2 | heater 1.6515e6 J per warm-up, 6.6060e6 in all | **6.605973e6** in all (= 2b-iii + lamp saved − battery); every season's battery change identical (−2.934643e9), so **1.65149e6** each | HELD |
| B3 | cold 4.005–4.013 °C; day 56's first reading ≈ 15.97; day 305's ≈ 15.2; end 288.3622829 K | day-ends **4.0062–4.0128 °C**; day-end 56 **15.965**, 305 **15.206**; end **288.3622829** | HELD |
| B4 | node end 174.18129; daily 163.1421 / 174.3095 / 171.44761 | **174.18129**; **163.1421 / 174.3095 / 171.44761** | HELD (to the printed digit) |
| B5 | surroundings 1.305818e9; space 43 991 226 753.0 | **1.3058179e9**; **43 991 226 753.017** (the instrument's own 43 991 226 753.017456 to every digit) | HELD |
| B6 | chill-days 55.926093595 | **55.926093595** | HELD |
| B7 | thermal time 5512.7606791 if unstressed | **5512.7606791** | HELD — the crop is never water-stressed |
| B8 | flowering / maturity 104.875 / 138.875 | first day-end past each: **105 / 139**, all four seasons | HELD |
| B9 | cold vapour ceiling 1.031 kg (4.011 °C) | settled cold days' max **1.031171 kg** — the ceiling at the MEASURED warmest settled cold reading, 4.0128 °C, is 1.031171 (A10's lesson applied). One re-sow day-end reads 1.031181, mid-way through condensing the warm air's excess | HELD |
| B10 | the same 10 stocks identical to 3a; `thermal.chamber` 43 254 342.4 J | the 10 identical, **plus 2 unpredicted** (`condensate`, `water_vapor`, below); chamber **43 254 342.43** | HELD, with an unpredicted extra |
| B11 | cold-phase production below 3a's 0.47 mol C, maybe negative; season production, soil pools and grain below 3a | the seedling **loses** carbon: 0.160 → **0.077** mol C over the 56 dim days; humus 12.91 → **7.54**, litter 6.95 → **3.93**, microbial 4.33 → **2.43**, grain 37.87 → **24.26**; peak crop 44.05 → **28.43** | HELD (direction; magnitude not predicted) |
| B12 | ⚠ A9 (`rationed == 0`, no events) at risk | rationed **0**, events **0** (the golden asserts both) | HELD — the risk did not fire |
| B13 | the vapour ceiling graded at the measured temperature | done (B9) | — |
| B14 | manifest: `cold_period.yaml` + golden hash; 19 of 20 identical | exactly those two lines; **19 of 20** | HELD |
| C1 | L2: −21.483 / −35.298 W | **−21.4826** (18 °C cabin) | HELD |
| C2 | L4 120 d unchanged (179.1465 K, 44.84 W) | green, unchanged | HELD |
| C5 | net radiation dim 16.849 W m⁻² on cold days | re-pinned, green | HELD |
| C6 | perturbation signs hold; `|carbon| > |O₂|` at risk | all green, the comparison included | HELD |

**Unpredicted, explained (`condensate` and `water_vapor` byte-identical to 3a).** The plants'
water ring reads the crop nowhere: transpiration (`Transpiration`, Penman–Monteith) reads the
lamp's net radiation, the chamber's temperature and vapour, and the soil's water stress — not the
canopy (a recorded scope line since §21) — and the crop is never water-stressed (B7). The
warm-phase chamber is history-free: the deadbeat cooler (dt = τ) resets it every minute. So in
every warm phase the ring runs on bit-identical inputs in 3a and 3b, and the condensate's
first-order return contracts any difference by 0.96875 a plant step — e^−126 over the 249 warm
days, far below one ulp. The two runs land on the same bits. Read in the code, not isolated.

**The roster, graded on the first full run (`--no-fail-fast`).** Eight reds in five binaries:
`chamber_walls` L2 (C1), `lamp_net_radiation` ×2 (C5), the manifest (B14) — predicted; and three
NOT predicted, recorded:

1. **`gas_exchange::a_change_to_the_plant_side_light_reaches_the_minute_step`** — "the lit crop
   did not grow: −0.0042 mol C" over two days. The same cause as B11: under the dim cold lamp the
   lit seedling shrinks too. The pin was `lit > 0`; its subject is that a plant-side light change
   reaches the minute step, which a copied resolver would erase as `lit == dark`. Re-expressed
   `lit > dark` (and `dark <= 0` kept).
2. **`air_split` — the BVAD chamber's starvation and the fan control** — measured in the dim world
   at day 90: the BVAD chamber at **0.513** of shared air (the pin: under 0.5), the fan's 0.1/0.4
   ratio **0.973** (the pin: under 0.97). §24l's roster missed this file for 3b (it listed only
   3a's figures). Cause: the crop enters the warm phase smaller and at day 90 has had 34 warm days
   — it is not hungry enough for the chamber's CO₂ supply to bind. Moved, like `lamp_shed`'s
   reserve-crossing six, to the cold period with the full lamp (3a's world exactly, by stage 1);
   the dim-world figures are this record. ⚠ So under the reference's own cold program the
   separate-air starvation is shallower at day 90. Over the whole season the `air_split` example
   (run in the dim world, below) still starves it: 0.183 of shared air's plant carbon (3a 0.141).

**The §24i fix's liveness:** with `run_shedding`'s nominal mutated back to the constant full
lamp, `rule_off_the_lab_wiring_is_the_plain_sealed_run_bit_for_bit` and
`rule_on_with_the_frozen_battery_nothing_sheds_and_nothing_moves` both go **red**; restored,
green.

**Why the grain fell 36 % — measured, not isolated.** The same day-by-day example run with the
cold period's lamp at full power (3a's world, by stage 1): crop carbon at the end of the cold
weeks **0.424** (3a) against **0.077** mol C (3b), at day 90 **9.76** against **1.93**, peak
**44.43** against **28.41**, grain **37.70** against **24.24** (season 1). The 3b crop enters the
warm weeks 5.5 times smaller and never closes the gap before maturity; 2.75 more warm days (B8)
do not make it up.

**The slow (ignored) station tests, run:** all pass — L4's full horizon battery
**1.680819683e9** J (C3: 1.680820e9); R3/R4 node **163.14207 / 174.30953 / 171.44761**, chamber
day-ends 277.1562020–295.2034234 K with **8** between the bands, end 288.3622829 (C4); the sealed
golden, session parity, resume and tier band tests green. Full suite and clippy green.

**The seven examples, run after — graded against 3a's outputs** (`W:\temp\claude\slice3a\ex_*.txt`
against `W:\temp\claude\slice3b\ex_*.txt`; second review, advisor): all complete. Separate air,
whole season: the BVAD chamber's crop **0.183** of shared air (3a 0.141), still starved; with the
gas exchange on the minute step **0.904** (3a 0.886); shared-air plant carbon 40.42 → **25.71** mol.
`intraday_exchange`: the minute step's gain over the plant step **+1.75 %** (3a +2.45 %); peak leaf
area **1.67** (3a 3.42). Water (`watering`, `minute_transpiration`): season transpiration 1309.7 →
**1283.4** kg; watering on the minute step 369.5 → **367.0** kg; no water stress anywhere (FTSW min
0.84 on the plant step), as before. `air_split_baseline`: the worst plant-step CO₂ draw 0.0174 →
**0.0129** of the pool. `draw_census`'s control 1 is byte-exact against
the regenerated golden. `lamp_shed` (the reference's own cold program, the SMALL battery): under
the dim lamp's 13 W the blackout never brings the battery to the reserve (lowest 6.24e7 J against
2.61e7), so nothing sheds and all four runs' crops are identical — §24h's 3b warning, measured.
That is why the tests moved to the full-lamp cold period.

**Not removed (recorded):** `VPD_VAR` still in the sealed plant resolver, unread. **Next:** slice 4,
re-sow on maturity (the clock moves into the state, §24c).

**Why the seedling shrinks — MEASURED (second review, advisor: the first wording, "cannot keep
itself under the dim light", was asserted).** A temporary example (not committed) replayed
season 1's 56 cold days in the reference's step order and, before every step, evaluated each flow
on the same snapshot, booking its carbon by flow type. Control: the booked legs close on the
observed change EXACTLY (crop −0.082915651, litter −1.327087291), and day 56's crop equals the
reference run's (0.077084). The same run with the cold period's lamp at full power (3a's world)
reproduces 3a's day-56 crop (0.423597).

| 56 cold days, mol C | 3b, dim | 3a's world, full lamp |
|---|---|---|
| growth into the organs (`Allocation`) | **+0.0089** | +0.4691 |
| maintenance respiration | −0.0185 | −0.0165 |
| leaves, stems, roots shed to litter (`Senescence`) | **−0.0734** | −0.1889 |
| change | **−0.0829** | +0.2636 |

So the loss is mostly SHEDDING, not respiration: the crop's net production (growth − upkeep) is
only slightly negative (**−0.0096**), while it sheds 0.073 — split by organ (same instrument,
same closure): **leaf 0.0338, root 0.0327, stem 0.0069** (3a's world: 0.1153 / 0.0636 / 0.0100).
⚠ First reported to the user as "leaves"; roots shed nearly as much (corrected 2026-10-06). `Senescence` is a constant relative
death rate (`rdr_leaf` × leaf, plus the shading term) and **reads no temperature** — at 4 °C the
seedling sheds leaf at its 22 °C daily rate while its growth all but stops. ⚠ A CANDIDATE MODEL
GAP, recorded, not acted on: in thermal-time crop models a leaf's life is counted in degree-days,
so a cold leaf lives longer. Not measured further here; it matters for slice 4 and for the 6- / 10-
week sensitivity checks. The leaf never nears die-off: lowest **0.0162** mol C, falling ≈ 15 % a
week, against an extinction threshold of **0** (the organ is a population that cannot be zeroed).

**DECIDED 2026-10-06 (the user), on the 100 µmol's locus and its cost:** *"Keep 100, study
shedding"* — the cold-week light stays at 100 µmol m⁻² s⁻¹ (§12's choice stands, the locus warning
recorded), and the next item is the leaf-shedding rule that reads no temperature (sources first,
predictions before code), ahead of slice 4.

⚠ **Forward pointer (2026-10-06), §24m:** the seedling's cold-week loss (0.160 → 0.077 mol C) was
the flat tissue shedding; after the leaf-shedding unfreeze (`post-roadmap-leaf-shedding.md`) the
seedling ends the cold weeks at 0.146 mol C and grain at maturity is 14.58 mol C (9.51). §24m's
numbers are a dated record of the tree before it.

## 25. Slice 4 — re-sow on maturity: design and stage 1's predictions, before code (2026-10-06)

The user's "go" (2026-10-06) on the recommendation: open slice 4's design, predictions first, the
plumbing before the switch (the slice-3 shape). Decision 3 of §10 (*"re sow as soon as the crop
matures"*) is already the user's; nothing here re-asks it.

### 25a. Advisor review (2026-10-06), summarized

1. **The clock is written by the STATION's hook, not `annual_reset`/`reset_crop`** — those also
   re-sow the perennial biosphere runs, so an entry written there would land in the biosphere
   goldens (a second unfreeze). Seed it at the sealed build too (the missing day-0
   `vernalization_days`, §10 slice 1, is the precedent not to repeat).
2. **Stage 1 is not "nothing frozen moves"** — the goldens serialize `aux`, so
   `sealed_station_state.json` gains one key. (The recommendation given to the user said
   otherwise; corrected in the reply.) Predict exactly one new key, diff with it stripped.
3. **Stage 1's bit-identity cannot see a reader left on the calendar** — while the re-sow stays on
   the calendar the state clock and the calendar agree everywhere. Two guards: an off-calendar
   sowing test (both sides, all program variables), and **deleting** the n-only
   `is_cold_on_step` family so every calendar reader is a compile error.
4. **No simcore change** without checking the freeze — checked: both the biosphere and station
   unfreeze steps require `git diff rust/crates/simcore/` empty. So the station-side wrapper
   pattern (`ChamberEnv`, with its guard), its order against the other wrappers written down
   and tested.
5. **Stage 2 timing, predicted**: the hook runs once per master day, before the plant steps; the
   fast side's 90-minute lead into each cold phase disappears; stage 1 KEEPS `mod season_days`
   (so the lead's bytes reproduce), stage 2 DROPS it (else `season_days` survives as a hidden
   second clock); maturity through `science::development_stage`; the threshold on
   `SealedStationScenario`, so the hook's signature is unchanged for `godot_bridge`, `palette`
   and the session tests.
6. **Seed bank**: grain at maturity ~14.6 vs the seedling's 0.16 — check any `with_harvest`
   fixture against "seed bank too small".

### 25b. Why the readers cannot simply read the state

A forcing is `Fn(n, dt)` — it cannot see the state (`simcore::environment::Schedule`), and
simcore is outside both unfreeze paths. Five variables follow the cold program: on the plant
step `par`, `net_radiation`, `daylength`; on the fast step `lamp_power` and `chamber_setpoint`.
Three options were weighed:

* **A state-aware forcing in simcore** — refused: a simcore byte change, outside the freeze.
* **The wrapper owns the program's schedules** (the resolver carries none) — refused: the
  perturbations (`with_lighting_failure`, the lab's `with_lamp_power_cut`) compose schedules
  *in the resolver*; they could no longer reach the program.
* **CHOSEN — twins in the resolver, selected by a wrapper.** Each program variable `v` is
  carried as two plain schedules, `v@warm` and `v@cold`; a wrapper on every flow and aux process
  answers `v` with one of them, chosen by the state's sowing clock. **The plain name `v` is in
  no sealed resolver**, so a reader that escaped the wrapper errors at its first read ("unknown
  env var") instead of silently reading one phase. The guard: the wrapper **refuses** an inner
  environment that answers plain `v` (a forcing that would otherwise be silently overridden),
  as `ChamberEnv` refuses a plant-side `temp`.

### 25c. Stage 1 — the plumbing (calendar kept)

* **The clock**: aux `station.sown_step`, the slow step count `n` at the current crop's sowing
  (an integer held in an f64; exact below 2⁵³). Seeded `0` by the sealed build; set to `n` by the
  sealed re-sow hook, on the state the reset returns (`sealed_reset_hook` wraps `annual_reset`).
  Missing → a hard error, never 0.
* **Cold or warm**: `floor((n − sown)·bio_dt) mod season_days < cold.days`. In stage 1 every
  sowing is a multiple of `season_days · 16`, so this equals today's `floor(n·bio_dt) mod
  season_days` exactly (an integer difference, times 1/16, floored), including the fast side's
  one-plant-step lead at each season start. `n < sown` is refused.
* **The wrapper**: `SowingClockFlow` / `SowingClockAux` (type names, ids, priorities kept — the
  station manifest's flow set is keyed on type names). Plant side answers `par`, `net_radiation`,
  `daylength`; fast side answers `lamp_power`, `chamber_setpoint`.
* **Its order** against the other wrappers — it must be the OUTERMOST (applied last), so that
  the environment it builds is the one every inner wrapper reads through:
  * `LampLitEnv` (lab lamp shed) scales `par`/`net_radiation` as read from *its* inner
    environment; with the clock outside, `LampLit` asks the clock for `par` and scales the
    selected phase. With the clock inside, the clock would ask `LampLit` for `par@cold`, which
    `LampLit` does not scale — the shed lamp would silently stop dimming the crop. Tested.
  * `ChamberEnv` answers only `temp` — disjoint, order free.
  * `OnFastStep` (gas exchange, the lab's water loss) ignores the outer environment entirely
    (it reads the recorded window and shared stocks), so the fast-side clock around it is inert;
    the window **recorder** is a plant-step aux process and is wrapped, so the window records
    the selected `par`.
* **The perturbations**: `map_forcing` (station) applies its change to `v`, or to both twins when
  the resolver carries them, and refuses a resolver carrying both `v` and a twin.
  `run_shedding`'s direct read of the `lamp_power` schedule and its nominal draw read the program
  through the state.
* **The n-only API is deleted**: `is_cold_on_step`, `chamber_setpoint_on_step`,
  `lamp_par_on_step`, `photoperiod_hours_on_step`, `lamp_power_on_step`,
  `lamp_average_power_on_step` → state-taking forms, plus a steps-since-sowing form for the
  chamber's start (`chamber_heat0`, sown at 0).

**Predictions (stage 1):**

| # | Prediction |
|---|---|
| P1 | Every golden byte-identical but `sealed_station_state.json`, which gains exactly ONE aux key, `station.sown_step` = 14640 (the day-915 re-sow, 915 × 16) = `0x1.c980000000000p+13`; with that line removed, the file is byte-identical to today's |
| P2 | Station manifest: only `sealed_station`'s golden hash moves; flow set, aux set (empty — no aux PROCESS writes the clock), params unchanged. Biosphere manifest, authoring manifest, `tiers.json`: untouched. `git diff rust/crates/simcore/` empty; `rust/crates/domains/` untouched |
| P3 | Off-calendar sowing (a state sown at day 10): all five variables shift their cold window by exactly 10 days — plant side read through the wrapped registry, fast side likewise |
| P4 | A sealed resolver carrying plain `par` beside the wrapper → refused at the first read; a sealed registry run WITHOUT the wrapper → errors at the first read (unknown `par`) |
| P5 | `with_lighting_failure` and `with_lamp_power_cut` tests stay green; a mutation mapping only the warm twin turns at least one red (they run from day 0, i.e. inside the cold phase) |
| P6 | Session save/load and two-rate parity tests unchanged and green (the clock is state) |
| P7 | `cargo test` + `clippy -D warnings` green with the n-only API gone; every former reader re-pointed (listed in the outcome) |

### 25d. Stage 2 — the switch (written now; re-checked against stage 1 before its code)

* The hook re-sows when `development_stage(thermal_time) ≥ 2` (the threshold's phenology on
  `SealedStationScenario`), at the first master-day start after maturity; the calendar condition
  goes. The clock drops `mod season_days`. `season_days` stays the weather's tiling period and the
  horizon's unit (`days() = years · season_days = 1220`), and nothing else.
* **Predicted**, from B8 (maturity 138.875 days after sowing, every season): maturity falls in
  day 138, the re-sow at the start of day **139** after each sowing → re-sows on days 139, 278,
  417, 556, 695, 834, 973, 1112 (8 re-sows, 9 crops; the 9th sown on day 1112 stands 108 days at
  the horizon — past flowering, 104.875, not mature). A ±1-day drift in any one cycle shifts the
  rest; a measured mismatch is recorded, not tuned.
* The fast side's 90-minute lead at a re-sow disappears (the setpoint and lamp switch at the
  day's start, with the plants); the lead at the warm switch (day 56 after sowing) stays.
* Every re-sow finds grain far above the seedling (~14.6 vs 0.16, unstressed); the
  `with_harvest` fixtures are checked against "seed bank too small" before the golden.
* The lab shedding driver's "inside one season" guard becomes "inside one crop" (it runs no
  hook) — re-decided in stage 2.
* The golden moves; its numbers are predicted from a lab run of stage 2 before the regeneration.

### 25e. Stage 1 BUILT (2026-10-06, a station unfreeze) — the predictions graded

| # | Prediction | Measured | Grade |
|---|---|---|---|
| P1 | one new aux key in `sealed_station_state.json`, `0x1.c980000000000p+13`, every other byte identical | `regen_goldens`: 19 of 20 identical, `sealed_station` CHANGED; the diff is exactly one added line, `"station.sown_step": "0x1.c980000000000p+13"` | HELD |
| P2 | manifest: only that golden's hash | `dump_station_inventory --write-manifest`: one line, `sealed_station`'s `golden_sha256`; `simcore/` and `domains/` untouched (`git status`) | HELD |
| P3 | sown on day 10 → every program variable's cold window shifts 10 days, both sides | `tests/sowing_clock.rs`: all five, through the wrapper, on days 60 / 65 / 66 of a day-10 sowing (cold / cold / warm; each pair of twins differs, asserted), and day 60 of a day-0 sowing warm; on the REAL build the window recorder records the cold `par` and the lamp draws the cold power from the battery | HELD |
| P4 | a plain program forcing beside the clock refused; a registry off the clock fails at its first read | both, `tests/sowing_clock.rs` (Validation "silently overridden"; an unknown program variable) | HELD |
| P5 | perturbation tests green; a warm-twin-only mutation reddens ≥ 1 | green; the mutation (map `@warm`, leave `@cold`) reddened **3**: `a_lighting_failure_darkens_the_crops_net_radiation_too`, `cutting_only_the_lamps_power_darkens_the_lab_crop`, `lighting_failure_stalls_growth_and_spares_battery`. Restored | HELD |
| P6 | session save/load and parity unchanged | green, unedited | HELD |
| P7 | full suite and clippy green with the n-only API deleted | `cargo test --workspace`: 1333 passed, 0 failed (6 ignored, run separately); `clippy --all-targets -D warnings` clean | HELD |

**Found by the guard, not predicted.** The first suite run had 24 reds, every one the clock's
refusal of a plain `net_radiation` / `daylength_s` (value 67.29, 40541.1): the sealed plant
resolver starts from `weather_forcings`, which carries the weather's own `par`, `net_radiation` and
`daylength`, and slices 3b/§21 had replaced them by **overwriting** the plain names. With twins
the weather's values stayed behind under the plain names, where an unwrapped reader would have
read the WEATHER silently. The resolver now removes all three, as it already removed `temp`.
This is the guard doing the one thing it exists for.

**The former calendar readers, re-pointed** (the compiler's list, once the n-only API was
deleted): the lab shedding driver's lamp draw and nominal (`lamp_shed.rs`, now
`sowing::program_value` and `scenario.phase`); `tests/lamp_net_radiation.rs`. And the readers
the compiler could NOT list — direct reads of a plain program variable from a sealed resolver,
which fail at run time — found by running: `tests/gas_exchange.rs` (three tests),
`tests/minute_transpiration.rs` (one). The hand-assembled builds (`air_split`, the
`shedding_station` example, the gas-exchange tests' minute build) now call
`sealed::wrap_last`, which applies the chamber wrapper and the clock together, so neither can be
applied without the other.

**The wrapper order, as built.** The lab lamp shed rewires a build that is already on the clock,
so its `LampLitEnv` sits OUTSIDE the clock, not inside as §25c planned, and is asked for the
twins; it now dims `par@cold`/`par@warm` (`sowing::untwin`). Tested
(`the_shed_lamp_dims_the_twin_the_clock_selects`: half delivery records half the cold `par`).

**Stale, left:** `cold_period.yaml`'s header still names `is_cold_on_step` and "slice 4 must move
this clock into the state". A comment edit in a param file moves its manifest digest (an
unfreeze of its own); corrected the next time that file's entry moves for a real reason, the
`senescence.yaml` precedent.

### 25f. Stage 2 — the predictions, completed before code (2026-10-06)

**Advisor review of stage 1 / before stage 2 (2026-10-06), summarized.** Stage 1 complete, every
instrument able to fail. Before stage-2 code, add what the record already implies: (1) grain per
crop falls by about two thirds — the frozen crop has 14.58 mol C at maturity but 41.02 on the
day-305 re-sow eve, so ≈ 65 % forms in the ≈ 166 days AFTER its own maturity, which re-sowing at
maturity removes; book which flow fills it before naming it, and report it as a finding, not a
re-ask; (2) the cold weeks more than double (9 sowings × 56 = 504 days against 224) — battery,
heater and gas-regulator directions; (3) the end state changes character (a crop 108 days old,
not one 305 days old); (4) list the tests red by design; (5) seed bank — every `with_harvest`
run that crosses day 139; (6) the threshold from the crop's own phenology, and a check that the
hook fires at the first master-day start after the biosphere's DVS reaches 2. If the lab twin
shows a seed-bank failure or grain far from ≈ 14.6, stop and report rather than regenerate.

**Checked before code.** Seed bank: no sealed build with `with_harvest = true` is RUN across a
re-sow (`palette`, `godot_bridge`, the golden and every sealed session pass `false`; the one
`true` sealed build, `tests/gas_exchange.rs`, inspects its registries and never steps). The
threshold: `build_season` builds every sealed crop from `params::biosphere()`, whose `pheno` is
`params::phenology()` — the scenario carries that.

**The predictions (stage 2), the `sealed_station` golden:**

| # | Prediction |
|---|---|
| Q1 | Re-sows at the starts of days **139, 278, 417, 556, 695, 834, 973, 1112** (B8: maturity 138.875 days after sowing, so the first day-start with DVS = 2 is day 139 after it); 9 crops; the end state's `station.sown_step` = 1112 × 16 = **17792** (`0x1.1600000000000p+14`). The hook fires on the first master-day start after the biosphere's own DVS reaches 2, never earlier, never later (checked day by day in the lab twin). A ±1-day drift in a cycle is recorded, not tuned |
| Q2 | Grain at each re-sow **≈ 14.6–15 mol C** (crop 1: 14.58 at maturity plus ≤ 2 plant steps of fill), every crop well above the 0.16 seedling — no "seed bank too small". Later crops not predicted beyond "the same order" (their soil and air differ). **Grain per crop falls ≈ 64 %** against the frozen 41.02 |
| Q3 | The post-maturity grain is booked flow by flow on the frozen tree (crop 1, days 139–305) before the record names its source. Expected, not asserted: `Allocation` partitioning to storage at DVS capped at 2 |
| Q4 | The END state is a crop **108 days** after sowing: past flowering (104.875) and filling, so `storage_c` small (**< 5 mol C**, against today's 41.0 on the 305-day crop); `thermal_time` ≈ 1100 + 3.1 × 22 ≈ **1170 (1150–1190)**; `vernalization_days` ≈ **55.9** (every crop sees the same program); DVS between 1 and 1.1 |
| Q5 | Cold days **504** (9 × 56) against 224. Dim-lamp days +280 at (133.33 − 13.33) W averaged = **+2.90 GJ** of lamp not drawn; less the heater's extra cold-week draw. The battery ends **2.4–2.9 GJ fuller** than today's 8.261e9 J |
| Q6 | The station's heat store (node) ends and averages **colder** (less lamp heat reaches it through the cooler) |
| Q7 | Crop carbon over the horizon: more warm days with a YOUNG canopy (≈ 716 warm days, all before maturity, at the pre-maturity rate) against today's 4 × 249 with ≈ 169 a crop past maturity (dying back). Net fixation over the horizon **up**, so the scrubber's `co2_removed` and the O₂ makeup's draw **down** — ⚠ low confidence: the re-sown residue (8 crops' litter instead of 3) feeds decomposition |
| Q8 | Every other golden byte-identical (only `sealed_station` runs the sealed hook); manifest: that golden's hash only |

**Tests red by design (listed before running):**

* `scenario::cold_clock_tests::the_cold_clock_counts_whole_days_from_the_sowing` — asserts cold
  again at `season` steps after a sowing (the dropped `mod season_days`). Restated: cold through
  day 55 after a sowing, warm ever after (also at `season`).
* `warm_room_arrest::warm_room_first_resow_fails_for_want_of_seed` — a crop held at 22 °C never
  reaches DVS 2, so the hook never fires and the "seed bank too small" refusal is unreachable
  from the warm room; the warm crop stands. Restated to that (no re-sow over two seasons, grain
  0 throughout); the refusal itself stays covered in `domains` (`reset_crop`'s own tests).
* Not red but **made inert**, so re-pointed: `session_save_load::sealed_resume_across_a_season_
  boundary_is_bit_identical` saves at day 304 and resumes past 305 — no re-sow there any more.
  Re-pointed to the first maturity re-sow (day 139), with an assertion that the sowing clock
  moved inside the window (so it cannot go inert silently again).
* The lab shedding driver (`run_shedding`) runs no hook and refuses horizons past
  `season_days`. Changed to run the sealed re-sow hook as the reference driver does, and the
  guard goes; every lab shedding test runs 8–10 days, so nothing moves.

### 25g. Stage 2 BUILT (2026-10-06, a station unfreeze) — the predictions graded

The hook re-sows on `development_stage ≥ 2` from `scenario.pheno` (`params::phenology()`, the
file the crop is built from), and the clock lost `mod season_days`. Instrument: the lab twin
`station/examples/resow_on_maturity.rs` (committed; it reads the committed golden for the
calendar column), then `regen_goldens --write`.

| # | Prediction | Measured | Grade |
|---|---|---|---|
| Q1 | re-sows at the starts of days 139, 278, 417, 556, 695, 834, 973, 1112; end `sown_step` 17792; at the first day-start with DVS 2 | exactly those eight days; DVS 2.0000 on each re-sow's day-start and 1.9743–1.9750 on the one before; golden `station.sown_step` = `0x1.1600000000000p+14` | HELD |
| Q2 | grain at each re-sow ≈ 14.6–15, never near the seedling; per crop −64 % against 41.02 | **14.5804–14.5889** (crop 1 14.5804, the frozen crop's maturity value to every digit); −64.4 %. The "+ ≤ 2 plant steps of fill" in the prediction was wrong — the 14.58 on record was already the day-139 day-start value — so the range's floor sits 0.02 above the measurement | HELD (the range's floor 0.02 high) |
| Q3 | the post-maturity grain booked by flow before it is named | a temporary instrument (not committed; booked legs close on the observed change exactly) on the frozen tree, crop 1: before maturity `Allocation` +9.531 and `StemRemobilization` +5.049 = 14.580; days 139–305 **`Allocation` +26.415, nothing else** — the crop keeps photosynthesizing and, its development stage capped at 2, partitions the new growth to grain | HELD |
| Q4 | end crop 108 days old, past flowering; `storage_c` < 5; thermal time 1150–1190 (≈ 1170); chill-days ≈ 55.9; DVS 1–1.1 | 108 days; grain **0.896**; thermal time **1169.84**; chill-days **55.863**; DVS **1.093** | HELD |
| Q5 | cold days 504; battery +2.4–2.9 GJ over 8.261e9 | **504**; battery **1.11505e10 J, +2.889 GJ** | HELD |
| Q6 | the node ends and averages colder | ends **174.0307 K** against 174.1813 (−0.151 K); the average not measured (the calendar run's trajectory is no longer reproducible from this tree) | HELD (end only) |
| Q7 | net fixation up, so `co2_removed` and the O₂ makeup DOWN (low confidence) | both **UP** by 8.870 mol (`co2_removed` 400061.75 → 400070.62; O₂ supplied likewise) | **FAILED** |
| Q8 | every other golden byte-identical; manifest by that golden's hash only | `regen_goldens`: 19 of 20 identical; the manifest's one line | HELD |

**Why Q7 failed, measured.** The scrubber removes whatever carbon the station does not hold, so
over a run it moves by exactly the change in the carbon the biosphere holds at the END — not by
the carbon fixed along the way, which is what the prediction reasoned from. At the end the
biosphere holds **8.870 mol C less** (73.976 → 65.106; the scrubber's +8.870 to the last digit):
the frozen end crop's 41.0 mol of grain is gone (−40.12), only partly offset by the soil, fed by
eight crops' residues instead of three (humus +5.03, litter +8.12, microbes +4.60), and a young
crop's tissues (+13.5).

**The end state changed character, as predicted**: a crop 108 days old and filling, not one 305
days old standing since day 139 of its life. Also measured: soil carbon at the end is up by
roughly two thirds (humus 13.22 → 18.25, litter 7.34 → 15.46, microbes 4.46 → 9.06).

**The tests named red by design** were restated before the run, so none was observed red:
`the_cold_clock_counts_whole_days_from_the_sowing` (warm at `season`, the wrap gone);
`warm_room_arrest`'s re-sow refusal, now `warm_room_is_never_resown_because_it_never_matures`
(two seasons, sowing 0, grain 0 throughout); `session_save_load`'s season-boundary resume,
re-pointed to day 139 with an assertion that the sowing clock moved inside its window; the lab
shedding driver now runs the hook. ⚠ **One more, NOT in §25f's list, found by reading the test
after the predictions were committed and before running:** `warm_room_arrest`'s control
(`control_the_plain_season_develops_and_fills_grain`) read the plain run's LAST day, which is
now a third crop 27 days old with no grain; it reads the whole season now (and that the plain
run IS re-sown). New: `the_hook_resows_at_maturity_and_not_before` (one ulp of thermal time
short of DVS 2 → no re-sow; at it → re-sown, its sowing its own step, thermal time 0; the
threshold equal to `params::phenology()`'s). Suite: 1334 passed, 0 failed.

⚠ **Two reds NOT predicted, in the ignored full-horizon tests** (`chamber_walls.rs`, run with
`--ignored` after the suite was green — the roster in §25f did not list the trajectory pins).
Both moved in the predicted DIRECTIONS (Q5, Q6); their sizes were pins from independent
re-simulations of the calendar run, now re-pinned to this tree's measurement (labelled so in
the tests — a regression pin, not a physics check):
* the heater-on-the-structure battery: 1.6808e9 → **2.2602e9 J**;
* the reference node's day-end min / max / mean: 163.1421 / 174.3095 / 171.44761 →
  **162.2719 / 174.2496 / 169.1984 K** — which also grades Q6's AVERAGE half: 2.25 K colder,
  HELD;
* and, NOT predicted in direction either, the chamber's **cold band widens** from
  277.1562–277.1628 to **277.1517–277.1673 K**: 152 of the 504 cold day-ends sat outside the old
  band, every one within 0.0043 K of it. Cause, read in the setup: the cold weeks now fall in
  every part of the weather year (they used to start on the same weather day each season), so
  the chamber's walls face different outdoor air while it is held cold. Physically negligible;
  recorded because a pin moved. Transition day-ends: 9 over 9 crops (≤ 2 per crop). The run ends
  WARM at 295.1999296 K (it ended mid-cool-down at 288.36 before).

**Stale, left:** `cold_period.yaml`'s header still gives the clock as `floor(n·bio_dt) mod
season_days` — now wrong twice over (the wrap is gone too). Same rule as in §25e.

**Gaps and successors, recorded, not acted on.** (1) The model keeps filling grain after
maturity (the crop is never told it is finished — the `FINISH` line is a domain boundary, not a
cessation rule, on record); with the re-sow at maturity it no longer runs, but any run that
keeps a mature crop standing still shows it. (2) The re-sow is at day resolution (the hook runs
at master-day starts), up to one day after maturity; 0.125 day here on B8's maturity at
138.875 days after sowing.
