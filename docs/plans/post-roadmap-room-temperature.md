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
