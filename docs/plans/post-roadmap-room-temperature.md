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
window's recorded value is exact for every minute in it.

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
