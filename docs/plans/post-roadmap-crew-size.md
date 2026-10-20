# The station's crew size — one crew member, defined once

**Opened 2026-10-20** on the user's *"yes"* to taking the crew-size fix, the item they chose on
2026-10-19 as the one after the cabin CO₂ level (`post-roadmap-co2-level.md` §9, *"1300, option 1"*).
**This doc is the plan, its predictions, and (§12) the adoption of four crew members on 2026-10-20.**

## 1. What today is, measured

The station's crew is one set of numbers, `station::scenario::CABIN_GAS_SCENARIO`, and every
station build inherits its two intake rates (`GREENHOUSE_CABIN_SCENARIO`,
`SEALED_STATION_CABIN_SCENARIO` and `WATER_RECOVERY_SCENARIO` change only the stores). Against
BVAD Table 3-31 (`docs/bvad-reference.md`; per crew member per day: 24.654 mol CO₂, 1.332 mol fecal
carbon, 4.366 kg modelled water):

| Quantity | Station rate | Per day | Crew members' worth |
|---|---|---|---|
| food carbon eaten | `food_intake_rate` 4.0e-3 mol/s | 345.6 mol C | — |
| CO₂ breathed out (× `f_resp` 0.949) | — | 328.0 mol | **13.30** |
| water drunk (humidity + urine) | `water_intake_rate` 5.0e-5 kg/s | 4.32 kg | **0.99** |
| O₂ breathed in (RQ = 1, `CrewRespiration`) | — | 328.0 mol | 11.7 (vs BVAD 27.97 mol) |

So the station's crew breathes like 13.3 people and drinks like one. The 13.3 was first found by
`post-roadmap-crew-coupled-loop.md` §3, which priced the area the crop would need (187 m² against
the bed's 1 m²) and refused the loop; nobody chose the number. It is also why the scrubber's
*level* had to be cited rather than sized from hardware (`post-roadmap-co2-level.md` §3).

**Measured in the goldens (2026-10-20):** the sealed station eats 421 632 of its 500 000 mol food
store over its 1220 days (84 %), leaving 78 368.000 (`sealed_station_state.json`); the crew-only
cabin ends at 12.568102 mol CO₂ and 1993.102 mol O₂ (`cabin_gas_state.json`): the target plus the
crew's output over the scrubber's rate (3.796e-3 / 1e-2 = 0.380 mol) and the O₂ makeup's
(3.796e-3 / 2e-3 = 1.898 mol). Read with `W:\Claude_projects\space-station\sources\temp-evidence\crew-size\peek.py`
(git-ignored, like the rest of `sources\`).

## 2. ⚠ The tree holds FOUR "one crew member" sizings, not one — this item fixes ONE

| Where | CO₂ / carbon | Water | O₂ | Called |
|---|---|---|---|---|
| **station** (`CABIN_GAS_SCENARIO`; 5 station goldens) | 13.3 | 0.99 | 11.7 | — |
| standalone crew (`domains::crew::MISSION_SCENARIO`; `crew_state.json`) | 1.66 | 0.59 | 3.09 | — |
| standalone ECLSS stand-in (`STEADY_STATE_SCENARIO`; `eclss_state.json`) | 10.5 | 0.59 (humidity only) | 12.4 | — |
| authored files | the crew domain's or ECLSS's rates | | | *"one crew member"* / *"a crew of one"* |

The authored labels are false on both sides: `scenarios/bioregenerative_station.yaml` calls the
crew domain's 5e-4 / 3e-5 *"one crew member"*; `rust/data/scenarios/crew_habitat_template.yaml`
multiplies them by `crew_count`; `scenarios/lamp_greenhouse.yaml` calls the ECLSS stand-in's rates
*"a crew of one"*. **This item scopes to the station's five crew goldens** (`cabin_gas`, `water_recovery`, `greenhouse`, `harvest`, `sealed_station`). The other three sizings are
recorded here as findings and not touched: a separate item each if the user wants them, because a
science change and a re-labelling do not share a batch (`CLAUDE.md`, "Posture").

## 3. The user's choice — how many crew members

⚠ **The numbering below is this plan's, not a choice the user made.** Their 2026-10-19 *"option 1"* chose the
crew-size fix as the next item; it picked no crew count.

The crew flows are forced: nothing stops a store running dry except its starting size.

1. **One crew member. Recommended.** The smallest change, and the one the water already describes.
   Every store stays fed: the sealed station eats 31 694 mol of its 500 000. The crop is still
   about 14× too small to feed this one person — see §7.
2. **Four** (the ISS's usual complement, and `science_gates::bvad::N_CREW`). Food 126 777 mol of
   500 000, fine. Water through the crew is 21 080 kg over 1220 days against a 20 000 kg store; the
   sealed build recycles urine back into that store (it ends today at 19 473), so it should hold,
   but that is a measurement owed, not a fact.
3. **Keep 13.3 and raise the water to match.** 70 000 kg through the crew; the water store and
   possibly recovery need re-sizing. It keeps a number nobody chose.

## 4. The form — one crew member, from BVAD, with one copy of each value

* **The food rate is computed at run time: `N × BVAD's CO₂ load ÷ the loaded f_resp`** (24.654 mol/CM-d
  ÷ 0.949 = 3.006826e-4 mol C/s for one crew member). Breathed out, it is BVAD's CO₂ load exactly,
  as the science gate's cabin is (`science_gates.rs`, `bvad::cabin_scenario`), and `f_resp` exists
  once, in `crew.yaml`. So `CabinScenario` carries the crew count instead of a food rate, and the
  station's crew resolvers take the loaded crew params (their builders mostly do already).
  ⚠ **Rejected: a constant food-carbon total** (24.654 + 1.332 = 25.986 mol C/CM-d, +0.028 % CO₂). It
  looks free of `f_resp` but is not: its 1.332 rests on the same 50 % feces-carbon assumption that
  made `f_resp` 0.949, so editing `f_resp` would leave the constant silently disagreeing. That is why
  the science gate's docstring rejects it too. Typing `24.654 / 0.949` into a `const` is rejected for
  the plain reason: a second copy of a loaded value.
* **The BVAD constants move out of `#[cfg(test)] mod bvad`** into a module both the gate and the
  scenario read, so there is one copy of each.
* **Water re-derived from BVAD too: 4.366 kg/d = 5.053241e-5 kg/s, +1.065 %.** Recommended, so "one
  crew member" is defined in one place for both quantities. It involves no param (the water split
  is applied after), and it moves every water stock the crew touches by about 1 %. It can instead
  stay 5.0e-5 bit-identical and be its own item; the predictions below give both.

## 5. ⚠ The crew-only cabins start with no CO₂ — at one person the scrubber is barely tested

`cabin_gas` and `water_recovery` start `cabin_co2_0 = 0.0` and run 900 one-minute steps. Today the
crew reaches the 12.1885 mol target in about 54 steps. **At one crew member it takes 42 715 s, step
712 of 900**, so the scrubber works only in the last fifth of the run. The goldens would not
notice. Proposed: **start both cabins at the target** (as their O₂ and H₂O, and every crop-sharing
cabin's CO₂, already start at theirs). That is its own move and is predicted separately below.

## 6. Predictions, written before any measurement (one crew member)

Arithmetic: `W:\Claude_projects\space-station\sources\temp-evidence\crew-size\predict.py`. "Offset" = the crew's breathed-out
CO₂ (2.853478e-4 mol/s) over the scrubber's rate 1e-2, or over the O₂ makeup's 2e-3.

**`cabin_gas`, `water_recovery` (900 × 60 s):**

| Stock | Today | Predicted, start 0 | Start at target |
|---|---|---|---|
| `crew.food_store` | 784.000 | **983.7631** | same |
| `boundary.fecal_waste` | 11.016 | **0.82808** | same |
| `eclss.cabin_co2` | 12.568102 | **12.2170** | 12.2170 |
| `boundary.co2_removed` | 192.416 | **≈ 3.192** | ≈ 15.380 |
| `eclss.cabin_o2` | 1993.102 | **≈ 1994.857** | same |
| `boundary.o2_supply` | −203.086 | **≈ −15.27** | same |
| `crew.water_store`, `cabin_gas` | 17.300 | unchanged; **17.27125** with BVAD water | same |
| `crew.water_store`, `water_recovery` | 19.624250 | unchanged; **≈ 19.60** with BVAD water (recovery returns part of the extra) | same |

**`greenhouse` (7 days):** food store 1580.800 → **3818.147**; fecal 123.379 → **9.2745**; the cabin
(`biosphere.carbon_pool`) 12.568105 → **≈ 12.2170** (1323 → 1286 ppm). The seedlings move by under
1 %, all smaller (lower CO₂). **`harvest`:** the same, except that its food store also carries the
harvest's small return (+0.0215 mol today), so it ends a little above 3818.147, not on it.

**`sealed_station` (1220 days).** The golden is the state on day 1220, the last day of the fourth
season; it holds no grain (`storage_c` 0). Predicted final stocks:

| Stock | Today | Predicted |
|---|---|---|
| `crew.food_store` | 78 368.000 | **468 305.650** |
| `boundary.fecal_waste` | 21 503.232 | **1616.412** |
| `biosphere.carbon_pool` | 12.568517 | **≈ 12.217** (plus the crop's own offset, as today's 0.0004) |
| `biosphere.o2_pool` | 1993.099927 | **≈ 1994.857** (the same) |
| `biosphere.leaf_c`, `stem_c`, `stem_reserve_c`, `root_c` | 9.526, 5.412, 4.018, 7.138 | each **lower, by about 0.5 %** (between 0.2 and 1.5 %) |
| `boundary.co2_removed` | 400 033.1 | **≈ 30 000** (the crew's 30 078 breathed out, less the crop's net uptake) |
| `boundary.o2_supply` | −400 031.6 | **≈ −30 000** |

The ½ % comes from a measured local slope: the CO₂ item's lab run read grain +49.4 % at 1323 ppm
(`post-roadmap-co2-level.md` §9) and its price read +48.6 % at 1284 ppm (§5), about 0.5 % for 39 ppm.
The lab example will also read each year's grain peak, as that item's price did: **about −0.5 %**.
⚠ **The water split between `soil_water`, `subsoil_water` and `condensate` may move a lot while
their total holds**, as it did in the CO₂ item (−77 % / +70 %, sum unchanged): the depletion-
triggered watering fires on different steps once the crop changes at all. Predicted: the three-way
total within 1 %, the split unpredicted. The crew's breathing (2.85e-4 mol/s) stays about 10× the
crop's peak lit uptake, so **the cabin never falls below the target** and the one-sided scrubber
never idles; the lab run will count the steps where it does. ⚠ *It counted day ends, not steps
(§10): the claim is checked at 1221 day boundaries, not at every minute.*

**Every other golden (15 of 20) byte-identical**: the standalone crew, ECLSS and biosphere
goldens do not read the station's scenario. **The authored bed does not read the station's crew,
settled by reading the code (2026-10-20):** `authoring/src/bed.rs` starts from
`sealed_station_scenario()` but builds only through `station::plant_chamber` and
`station::sealed::sealed_bio_resolver`, and neither reads `food_intake_rate`,
`water_intake_rate` or the cabin scenario.

**Non-golden tests likely to go red or change their subject** (to read at the lab step, before
adopting):

* `station/tests/perturbations.rs`, the carbon leak (`K_LEAK` 1e-3 /s on the cabin's CO₂). The leak
  far outruns one crew member's breathing, so during the window the cabin falls to about 0.29 mol
  (≈ 30 ppm), against about 3.8 mol (≈ 400 ppm) today. The crop starves much harder there. Its
  assertions are one-sided and should still hold; `rationed == 0` is the line at risk.
  `assert_returns_to_setpoint` (crew spike, lighting failure) assumes the scrubber is working at the
  day's end; it should stay true, since the crew still out-breathes the crop.
* `station/src/science_gates.rs`, `bvad::cabin_scenario`: it sets its own rates, but it inherits
  `cabin_co2_0` through `..CABIN_GAS_SCENARIO`, so starting the cabins at the target changes the
  gate's run too.
* `tier_sensitivity`, `station/tests/air_split.rs`, the `godot_bridge` `cross_boundary` /
  `time_control` tests and the session tests build these scenarios; whether any pins a crew-sized
  number is read off the full suite's reds.

**Not in the model, so cannot move:** the crew's body heat (BVAD 143.8 W per crew member) enters
no room's heat books; grepped `rust/crates` for metabolic heat. Nothing here changes RQ = 1, so
the known O₂ miss (−11.8 % against BVAD) is unchanged.

## 7. Not this item

* **Food closure.** One crew member needs about 14 m² of crop to be fed and the bed has 1 m²
  (24.654 ÷ 1.7496 mol C/m²-d, `post-roadmap-crew-coupled-loop.md` §3). This fix makes the crew
  consistent with itself; it does not make the station feed itself. The crew-coupled loop stays
  refused.
* The three other crew sizings of §2.
* The scrubber's level (adopted, 0.13 kPa) and rate (`DESIGN`) are unchanged. At one crew member
  the cabin sits 3 ppm above the target instead of 40.

## 8. Build order

1. **Lab first.** A station example (`co2_target_lab.rs`'s pattern) that runs each of the five
   goldens' builds twice: frozen, and with the cabin scenario's rates set to `N` × BVAD. It first
   checks that its frozen side reproduces each golden's bytes. No library switch is needed: the
   rates are scenario data, so the example passes its own `CabinScenario` (at the lab stage it
   sets the rate itself; the run-time form of §4 is built at adoption). It measures one and
   four crew members, BVAD water on and off, and the start at the target, against §6.
2. **The user's adoption choice**, on the measured numbers.
3. **Adoption** as a station unfreeze: save the lab's switched states as bytes, regenerate, `cmp`.

## 9. The user's answers (2026-10-20), and the predictions at four crew members

**Four crew members; water re-derived from BVAD in the same change; the two crew-only cabins
started at the scrubber's target.** One consequence, taken as the obvious default and said so to
the user: the 7-day greenhouse and harvest cabins hold 50 kg of crew water, and four crew members
drink 122.25 kg in 7 days, so the store would run dry on day 3 (the flows are forced). It becomes
**200 kg**, today's margin kept (50 kg for 30.24 drunk is 1.65×; 200 for 122.25 is 1.64×).

Rates: food 1.2027304e-3 mol C/s (computed from the loaded `f_resp`), breathed out 1.1413911e-3
mol/s (4 × 24.654 mol/d exactly), water 2.0212963e-4 kg/s (4.0426× today). The CO₂ offset over the
target is 0.1141 mol (1295 ppm, −28 ppm from today), the O₂ offset 0.5707 mol. Written before the
measurement, by the same arithmetic as §6:

| Golden | Stock | Today | Predicted (N = 4) |
|---|---|---|---|
| `cabin_gas`, `water_recovery` | `crew.food_store` | 784.000 | **935.0526** |
| | `boundary.fecal_waste` | 11.016 | **3.31232** |
| | `eclss.cabin_co2` (started at 12.188502) | 12.568102 | **12.30264** |
| | `boundary.co2_removed` | 192.416 | **≈ 61.52** |
| | `eclss.cabin_o2` | 1993.102 | **≈ 1994.429** |
| | `boundary.o2_supply` | −203.086 | **≈ −61.06** |
| `cabin_gas` | `crew.water_store` | 17.300 | **9.0850** |
| `water_recovery` | `crew.water_store` | 19.624250 | **≈ 18.48** (today's net loss × 4.0426, if recovery is linear in throughput) |
| `greenhouse`, `harvest` | `crew.food_store` | 1580.800 | **3272.589** (harvest a little above) |
| | `boundary.fecal_waste` | 123.379 | **37.098** |
| | `crew.water_store` (now from 200) | 19.760 | **77.752** |
| | `biosphere.carbon_pool` | 12.568105 | **≈ 12.3026** |
| `sealed_station` | `crew.food_store` | 78 368.000 | **373 222.598** |
| | `boundary.fecal_waste` | 21 503.232 | **6465.647** |
| | `boundary.co2_removed` | 400 033.1 | **≈ 120 220** (120 312 breathed out, less the crop and soil's net uptake, ~95 today) |
| | `boundary.brine` | 527.028 | **≈ 2131** (today brine is 10.0 % of the crew's water, the recovery's 0.9) |
| | `crew.water_store` | 19 472.854 | **≈ 17 869** — the store holds, about 11 % down over four years |
| | `biosphere.carbon_pool` | 12.568517 | **≈ 12.303** |
| | `biosphere.o2_pool` | 1993.099927 | **≈ 1994.43** |
| | `leaf_c`, `stem_c`, `stem_reserve_c`, `root_c` | | each **about −0.4 %** (−28 ppm at §6's ~0.5 % per 39 ppm) |
| | yearly grain peak (read by the lab, not in the golden) | | **about −0.4 %** |

The soil-water split (`soil_water`, `subsoil_water`, `condensate`) may move freely; their total
within 1 %. The cabin stays above its target at every day's end (the crew breathes ~40× the crop's
peak lit uptake). 15 of 20 goldens byte-identical. The carbon leak test's cabin falls to about
1.14 mol during the leak (≈ 120 ppm), against ≈ 400 ppm today.

## 10. Measured against §9 (2026-10-20) — lab only, nothing frozen moved

`cargo run --release -q -p station --example crew_size_lab` (from `rust/`, below-normal priority;
`station/examples/crew_size_lab.rs`). Raw output:
`sources/temp-evidence/crew-size/crew-size-lab-n4.txt` (all five; re-run after the review below
added the re-sow days and the same-age leaf, with identical numbers). **All five frozen sides reproduce their golden's
bytes.** Every switched run: `rationed 0`, `events 0`.

**Every predicted number held but one, the final leaf (below).** To the printed figures: food stores 935.0526 / 3272.589 /
373 222.598; fecal 3.31232 / 37.098 / 6465.647; the cabin 12.30264 (crop cabins 12.3026, sealed
12.303); `co2_removed` 61.521 and sealed 120 216.8 (predicted ≈ 120 220); `cabin_o2` 1994.4293;
`o2_supply` −61.064; crew water 9.0850, 18.481 (predicted ≈ 18.48), 77.752, sealed 17 869.0
(≈ 17 869); brine 2130.56 (≈ 2131). No day ends with the cabin below its target (0 of 8, 8, 1221).
The soil-water total is unchanged to the fourth decimal while its split moved freely, as flagged
(condensate +55 %, subsoil −9.7 %, soil −5.3 %). The sealed grain peak, read per 305-day year
by the lab (each year holds about two crops): **−0.33 % to −0.35 %** in all four years (predicted about −0.4 %). Re-sow days identical
(141, 281, … 1121).

**The plant stocks, against "each about −0.4 %" (band 0.2–1.5 %):** root −0.59 %, stem reserve
−0.52 %, stem −0.50 %, **leaf −0.06 %, outside the band: a MISS** of size, not of sign. A first
draft of this section explained it as the final crop's stage (re-sown day 1121, 99 days old). **That
was tested and is wrong as stated:** the lab reads the leaf at age 99 days after every sowing, and it
moves −0.058 % to −0.071 % all eight times. So it is not one day's chance: at that age the leaf is
about six times less sensitive to the cabin's CO₂ than the other organs. Why is not measured.

**Three moves not predicted, each explained:**

* **The cabin's humidity, +11.1 % (1.8538 → 2.0592 kg), in every build.** The condenser holds the
  vapour above its target by the crew's humidity over its rate, and that output is ×4.04. Same
  mechanism as the CO₂ and O₂ offsets; it was left out of the table, not out of the model.
  ⚠ **It deepens a recorded defect.** The sealed station's cabin and plant chamber share one air,
  but their two vapour stocks are held separately and nothing reads their sum (`docs/log/` cabin
  humidity record). Saturation for the 9500 mol at 22 °C is 4.4658 kg (40 % RH = 1.7863). The sum
  is 3.3464 + 1.8538 = 5.2002 kg today (116 % of saturation) and 3.3464 + 2.0592 = 5.4056 kg at
  four crew members (121 %). Not this item's to fix; it does not block adoption.
* **The harvest build's soil carbon, −69.9 %** (litter, microbes, humus). That build sends the
  crew's feces into the soil (`build_harvest(…, close_feces = true)`), so the soil's carbon input
  is the fecal carbon, ÷3.33 like it.
* **The sealed station's energy, by about 0.01 %** (battery +1.2 MJ over four years, the node and
  the two boundaries with it). **By elimination, the plant room's heater.** Four flows are built on
  the battery (`sealed.rs`'s `SolarCharge` and `LoadDraw`; `plant_chamber::chamber_flows`'s `Lamp`
  and `ChamberHeater` through the power port); the latent-heat coil, the cooler and water recovery
  are not. `SolarCharge` and `LoadDraw` read constant forcings, the `Lamp` reads only its program
  (`env.get(LAMP_POWER_VAR)`), and the re-sow days that set the program are identical. That leaves
  the heater. *Why* the heater's work moved (the crop's water use, through the chamber's heat books)
  is not measured.

**Unchanged and not run:** the 15 other goldens (none reads the station's crew); the non-golden
tests of §6, read at adoption. **Adoption is the user's call**, a station unfreeze: five goldens,
the station manifest's hashes, and the scenario's form (§4: the crew count and the loaded `f_resp`,
the BVAD constants in one module, the 200 kg crop-cabin store, the crew-only cabins started at
the target).

## 11. Trial run against the whole suite (2026-10-20) — undone afterwards, nothing committed

The four crew members' values were put into `station::scenario` by hand (the constants as the lab
computes them, `CABIN_GAS_SCENARIO.cabin_co2_0 = 12.188502343942758`, the crop cabins' store 200 kg),
the whole suite run, and the file restored (`git status` clean). Setting the start on the shared
constant is the lab's change: only the two crew-only builds read `cabin_co2_0` (`cabin.rs`, `water.rs`;
grepped), and the lab sets it for exactly those. The point: find what fails that is
**not** a saved result moving, before the user decides.

**`cargo test --no-fail-fast`: four targets red.** Three are the saved results moving, as they must:

* `station --test golden_regression` (cheap goldens): stops at `cabin_gas_state.json` line 7.
* `station --test tier_contract` (cheap bands): `cabin_gas` is bit-exact tier, 192.416 against 61.521.
* `godot_bridge --test cross_boundary`: the two tests that compare the bridge's run with the
  `cabin_gas` golden (same line 7).

**The fourth is not a saved result.** `station --test air_split`,
`on_the_minute_step_the_chamber_is_fed_and_the_fan_rate_matters`: the fan's 0.1 / 0.4 ratio is
0.8599 / 0.8808 = 0.976, against the pin's `< 0.97`. Its other two claims held (the chamber fed,
the fan's order). This file already keeps the old cabin (the first-order scrubber and the old start,
`post-roadmap-co2-level.md` §10) because its subject is the CO₂ supply to a **hungry** crop.
**Cause, measured:** with the four crew members' water and start level kept and only the old food
rate (4.0e-3) put back in the file's `scenario()`, all ten of its tests pass. So the smaller crew's
CO₂ output fed the crop less, and the fan stopped being what limits it. Proposed, the CO₂ item's
precedent: the file keeps the old food rate as it keeps the old scrubber, said in its doc comment.
Loosening 0.97 would weaken the pin. **The user's call.**

**Passed, including every test §6 named as at risk:** `perturbations.rs` (the carbon leak still
`rationed == 0`), `science_gates`, `tier_sensitivity`, the session tests, the other `cross_boundary`
tests. 103 test targets ran.

**`cargo test --no-fail-fast -- --ignored`: two targets red, both expected.** The sealed station's
golden (`the_sealed_station_golden_is_still_this_reference_s_output`, line 3 `rooted_depth`) and its
band (`every_expensive_station_golden_is_inside_its_measured_band`, worst 3.76 at `stocks[29]`).
Every other slow test passed, the 10-minute soil-evaporation and the chamber-wall runs among them.

**So adoption costs:** five goldens and their bands (expected), the bridge's two `cabin_gas`
comparisons (they follow the golden), and one decision on the fan test.

## 12. ADOPTED (2026-10-20) — four BVAD crew members, a station unfreeze

The user's words: *"adopt the four crew, about the test - do as you recommend"*. So the fan test keeps the old
crew CO₂ load (§11's proposal), never a looser 0.97. Advisor-reviewed before code: two commits, so that the
expression the lab measured is the one adopted. Both arithmetic orders in the tree (the lab's
`n × 1.085 × 1000 ÷ 44.009` and the gate's `n × (1.085 × 1000 ÷ 44.009)`) give the same bits at n = 4, since
multiplying by 4 is exact. The split settled that question rather than relied on the argument.

1. **The refactor commit (no byte moved).** The BVAD values moved from `science_gates.rs`'s test-only module
   to `station::bvad`, with `food_intake_rate(n, f_resp)` and `water_intake_rate(n)` defined once. The gate
   and the lab read them; the lab gained `CREW_SAVE=<dir>`. The lab's output was **identical** to
   `crew-size-lab-n4.txt`, all five frozen sides reproduced their goldens, clippy was clean and the
   science gates passed (checked on that commit's own tree).
2. **The saved states.** That run wrote the five switched final states in the goldens' own bytes.
3. **The unfreeze.** `CabinScenario` carries `crew_count` (`STATION_CREW_COUNT` = 4) in place of its two
   rates; `CabinScenario::food_intake_rate(&CrewParams)` and `water_intake_rate()` compute them; the five crew
   resolvers (`cabin`, `water_recovery`, `greenhouse_cabin`, `harvest_cabin`, `sealed_fast`) take the loaded
   `CrewParams`, which every caller passes (the one it built with, or `domains::params::crew()` where it held
   none). `CABIN_GAS_SCENARIO.cabin_co2_0` is the target, pinned in `the_cabin_starts_at_its_targets`;
   `GREENHOUSE_CABIN_SCENARIO.water_store0` is 200. The science gate's validation cabin names `crew_count: 4`.

**Measured:**
* `regen_goldens -- --write` rewrote exactly the five (`cabin_gas`, `water_recovery`, `greenhouse`, `harvest`,
  `sealed_station`). **Each is `cmp`-identical to its saved lab state.** The other 15 are identical.
* The manifest moved exactly as predicted: the 5 `golden_sha256` rows and nothing else.
* `git diff rust/crates/simcore/` is empty.
* `cargo test --workspace --no-fail-fast`: 1456 passed, 0 failed, the bridge's two `cabin_gas` comparisons
  among them.
* **`air_split.rs`: all ten pass** with the crew count that eats the old 4.0e-3 under the loaded `f_resp`
  (`OLD_FOOD_INTAKE_RATE × f_resp × 86 400 ÷ BVAD's CO₂ mol`, about 13.3). ⚠ This is not quite §11's trial.
  The trial kept four crew members' water with the old food rate; the adopted form has one count for both,
  so this file's crew drinks 13.3 crew members' water. It was re-run under that, not assumed. The pin
  prints no ratio, so the figure is not recorded, only that it is under 0.97.
* `cargo clippy --all-targets --workspace -- -D warnings`: clean. `cargo test --workspace --no-fail-fast --
  --ignored` (the slow tests, which no CI runs): 12 passed, 0 failed.

**Left as they were:**
* The three other sizings of §2.
* The pre-adoption lab tools (`co2_target_lab`, `co2_level_price`, `co2_water_lab`). They were only re-typed
  to compile, and their frozen side is now the four-crew station.
* `crew_size_lab.rs` is flipped. Its frozen side is now four crew members and its default switch one (`CREW_N`).
  The old 13.3-CO₂ / 0.99-water crew cannot be written as a crew count, so what the tool measured before
  adoption is the tool at the refactor commit.
