# The heat evaporation takes from the room — in the room's books, a lab switch first

**Opened 2026-10-15** on the user's *"yes"* to the third of the three follow-ons the latent-heat
estimate left (`post-roadmap-chamber-latent-heat.md` §5b), after night stomata
(`post-roadmap-night-stomata.md`) and the room's lamp heat switched with the light
(`post-roadmap-room-lamp-heat.md`), both adopted the same day. **This doc builds a lab switch and
measures it.** Adoption is a station unfreeze; it is taken here only if the measurement leaves one
sensible form, and put to the user in plain words if it shows a fork (§5).

## 0. Why

The plants' water costs no heat in the model. The transpiration flow moves water from the root zone
into the chamber air (and, past the humidity setting, straight into the condensate), and the chamber's
heat store (`chamber.rs`, `thermal.chamber`) never hears of it. Real evaporation takes
λ = 2.45 MJ per kg from the room, and a condensing coil gives it back into the coolant
(BVAD p. 145–146, quoted in the estimate's §1). While the cooler is running the two are one coil and
the station's thermal node receives the same total either way; where the cooler is idle — the dark
hours, the heater on — the room has a heat bill nothing pays. The estimate measured it with the lamp's
heat averaged over the day; the two follow-ons since made it measurable per window: the crop closes
its stomata in the dark, and the room's lamp heat is now off at night.

## 1. The form

**Evaporation draws λ·E from the chamber's heat store, minute by minute across the plant window that
moved the water, and the same heat goes to the thermal node** (the condensing coil, BVAD's one coolant
loop). Station-side only; the biosphere's `Transpiration` is not touched (its own chambers have no heat
books, and editing it would make this a biosphere unfreeze as well).

* **Why not with the water.** The water moves once per plant step (90 minutes). A window's evaporation
  is about 0.7 MJ in the warm phase (~135 W × 5400 s), against the chamber's 1.5e5 J/K: drawn at once it
  would drop the room ~5 K at every plant step and the heater would answer an artefact of the step.
* **The window.** The plant step records its window's evaporation rate (kg/s) into the state's aux (a
  plant-step aux process — the `PlantWindowRecorder` pattern of `gas_exchange.rs`; the aux channel is
  additive, so it writes the increment). The minutes after it lie in that window
  (`driver::day_groups`, one plant step per group on the sealed station — required, as the gas exchange
  requires it), so the fast flow reads the rate the window recorded. There is no `n − 1` to get wrong,
  and nothing is keyed on `n` (the lamp's 90-minutes-early trap).
* **The same water, not a copy of the law.** The recorder evaluates **the registry's own transpiration
  flow** (one instance shared between the registry and the recorder), on the plant step's snapshot and
  environment, and sums its leg on `SOIL_WATER` — what the flow moves, crop and soil together. No second
  builder and no second set of coefficients. Whatever wraps the plant step's environment (the chamber's
  temperature, the sowing clock's twins, the lab's lamp shedding) wraps the recorder too.
* **λ** is the model's own (`science.rs`, the one Penman–Monteith turns radiation into water with), so
  the heat and the water books agree. It is private there; the station takes it through a `pub` read,
  not a second literal.
* **The fast flow** `ChamberLatentHeat` (id `station.chamber_latent_heat`): `thermal.chamber →
  thermal.node`, `λ · rate · dt` J per minute. Over a window's 90 minutes that is λ × the window's
  evaporated water, to rounding.

### 1a. Where the vapour's heat lives — the simple form, and its bound

The exact form would hold the air's latent heat in its own energy stock (λ × the air's vapour, debited
by condensation). This build takes the **simple form**: the node is credited when the water evaporates,
not when it condenses. Water that stays in the air has had its heat booked to the coil early. The error
is bounded by λ × the air's vapour: at 22 °C and 75 % in 9500 mol of air that is ~3.3 kg, ~8 MJ, and
the swing between the cold period's air and the warm one's is ~2.3 kg (~6 MJ) — against a booked
quantity of order 1–9 GJ over the run. The measurement reports λ × the vapour's change over each phase
and over the run, so the bound is a number, not a claim.

### 1b. What is not in scope

* **The crew's moisture.** The cabin's own `Condenser` takes the crew's water (`CABIN_H2O`) into the
  recovery buffer; the crew are not in the plant chamber's heat books, so their latent heat is not
  either. Unchanged.
* **The cooler and the condenser sharing one capacity.** On BVAD's reading the condensing coil and the
  cooler are one heat exchanger, so sensible and latent heat both count against its 200 W. This build
  does not cap the latent heat; the measurement counts the minutes where cooler + latent exceed 200 W.
  ⚠ (a side note, 2026-10-15, before any run): the latent path is therefore an **uncapped** second
  route from the room to the node, so a fall in the cooler's own saturated minutes is not evidence of
  relief — the shared-coil count is the figure that says whether the coil keeps up (Q4).
* **The coil's temperature.** Condensation needs a coil colder than the dew point; the model condenses
  whatever is above the humidity setting regardless. The node is credited whatever its temperature; the
  measurement counts minutes where the node is warmer than the chamber (the cooler's own gate).
* **The sunlit chambers and the 7-day station builds** have no heat books (the estimate's §2).

## 2. Every reader

| Reader | Today | Under the switch | Why |
|---|---|---|---|
| `sealed::sealed_fast_flows` (the fast registry) | no latent heat | `ChamberLatentHeat` added | the change |
| the plant registry | `biosphere.transpiration` | the same flow, shared with the recorder | the recorder must read the moved water |
| plant-step aux | `station.plant_window` … | + the evaporation recorder | the window's rate |
| `chamber::chamber_heat0` (the chamber's start) | steady state of the first window's lamp draw (0, dark) | **to measure**: the first window's latent is the soil's evaporation in the dark | the start assumes a constant input; if the first window's latent is not ~0 the start shifts by `latent·τ / C` |
| `sealed::sealed_node_heat` (the node's start) | a daily-mean equilibrium | kept | the coil hands the node what the room loses; the daily mean is unchanged to the vapour bound |
| `air_split.rs` (lab separate air), `water_on_fast_step` (lab minute transpiration) | move transpiration onto the fast step | **refused** by the switch | the recorder would read no plant-step transpiration and the heat would go silently to zero; the switch errors if the transpiration flow is not on the plant step |
| `lamp_shed.rs` (lab) | wraps every plant-step flow and aux | composes | the recorder and the flow are wrapped alike |
| `perturbations.rs` | perturb resolvers and stocks | composes | nothing keyed on the new names |
| `session.rs` (save/load), `godot_bridge` | carry the state's aux | inherit | a new aux key and a new fast flow; the parity tests say whether they carry them |
| the station manifest (`flow_set`, `aux_set`) and its gate | — | **at adoption** | a new flow type; the paired gate owns completeness |
| `chamber_latent_heat` (the probe) | measures the missing term | measures the booked term | its closure check should pass except where the heater saturates |

## 3. Hazards (each checked by the measurement)

* **The heater saturates.** The reference room's heater (200 W) answers the walls and now the night's
  evaporation. With the walls on the **station structure** (a lab surroundings) it already carries
  182 W of 200 in the dark; the night's latent heat may saturate it, and the room would cool below its
  setpoint. All four surroundings are run.
* **The battery** pays the heater's extra work: its minimum and `rationed` are reported.
* **The plants read the room.** Carbon and water move a little, not only energy (the lamp item's
  lesson 3).
* **Rationing breaks the identity.** The recorder sees the evaluated flow, not the applied one; under
  rationing the two differ. Every reference run asserts `rationed == 0`.

## 4. Predictions (written before any code ran)

**Run 0, today's tree** (`chamber_latent_heat`, the switched lamp and night stomata; controls clean,
golden byte-exact): 2841.3 kg evaporated over 1220 days, λE **6.961 GJ** (66.0 W mean; the soil 11.4 %).
Warm dark windows: crop 19.6 W, **0.303 GJ**, all of it unfunded by the lamp. Warm lit windows: latent
140.4 W against the lamp's 177.8 W; 1079 of them (the partly lit ones) short of heat by **0.518 GJ**.
Cold windows: ~0.02 GJ short. Heater 0.306 GJ (warm 257 040 min, cold 38 310); cooler 8.496 GJ, at
capacity 1800 min warm / 1776 cold; walls net 0.636 GJ out; the room's warm minimum −11.18 K (the
warm-up after each cold period). Vapour at plant-step starts: 1.02–1.03 kg cold, 3.33–3.34 kg warm.
Battery ends 1.0865e10 J.

~~⚠ Within a window the lamp's draw and the latent draw are both uniform (the lamp's top-hat mean; the
window's rate), so a window's per-minute deficit is its mean deficit: the run-0 "unfunded" sums are the
heater's new work exactly, not a lower bound — before the walls and the plants answer the changed room.~~
⚠ *Struck after the run (advisor, 2026-10-15):* the probe's "heat in" counts the walls only when they
bring heat **in**; a lit window whose walls lose heat counts no loss, so its "unfunded" sum leaves the
walls' losses out. The sums were a **lower bound** after all — most of Q2's miss (§7), and the same fact as
the heater running by day.

| | prediction | confidence |
|---|---|---|
| Q1 | window identity: heat drawn over each window = λ × its moved water, ≤ 1e-12 relative; booked total = λ × evaporated (6.96 GJ at run 0's water, within Q6's move) | high |
| Q2 | the heater's total rises by **0.7–1.0 GJ** (0.303 dark + 0.518 partly lit + ~0.02 cold, ≈ 0.84), to **1.0–1.3 GJ**; warm heater minutes rise from 257 040 to **~330 000–370 000** (the partly lit windows' minutes join the dark ones) | medium-high |
| Q3 | the cooler removes **2.2–2.6 GJ** (the room's balance: lamp 8.83 + heater − walls 0.64 − λE 6.96) | medium-high |
| Q4 | **the shared coil** (the side note of 2026-10-15): cooler + latent above 200 W on **about as many minutes as the cooler alone saturated at run 0** (1000–3000 warm), since one coil's load is the lamp less the walls either way. The cooler alone saturates on **< 100** warm minutes — ⚠ that drop is the uncapped latent path (§1b), **not** relief, and is not reported as one | medium |
| Q5 | in the reference surroundings the heater never saturates outside the warm-up after each cold period (the night needs ~40 W, a partly lit window ≤ ~90 W + the walls); the warm minimum dips **11.2–11.8 K** | medium |
| Q6 | the plants read the room: lit minutes ~0.05 K cooler, dark ~0.02 K (the deadbeat offset `input·τ/C` with ~140 W less input by day); every carbon stock moves **≤ 0.1 %**, soil water and condensate **≤ 0.3 %** | medium |
| Q7 | the battery ends lower by **0.6–1.0 GJ** (the heater's rise, on the battery); `rationed == 0` | medium |
| Q8 | the node is colder than the room on every minute heat is booked to it (it sits near 174 K): **0** minutes | high |
| Q9 | the simple form's bound: vapour 1.0–3.4 kg, λ × range **5–6 MJ** — under 1 % of the heater's rise | high |
| Q10 | the first window is dark and evaporates almost nothing: latent **< 1 W**, so the chamber's start stays (a shift `< 1 W × τ / C` = 4e-4 K) | medium |
| Q11 | **the structure surroundings** (the heater already at 182 W of 200 in the dark): the heater **saturates** on many warm minutes (thousands); the room falls below its setpoint by **0.5–5 K** at those times; the battery, 1.48e9 J at the end today, ends **0.4–1.0e9 J**, above zero | low-medium |
| Q12 | the cabin (18 °C) and space surroundings: no heater saturation outside the warm-ups | low |
| Q13 | `regen_goldens` with the switch wired in: only `sealed_station_state` moves (carbon, water, energy); the other 19 and the biosphere's byte-identical — the `pub` λ moves nothing | high |

## 5. When to stop and ask

Adopt with the full ceremony if one form is the sensible one. Stop and put it to the user first if the
heater saturates in the reference surroundings, if anything rations or sheds, if the simple form's
bound turns out not small against the booked heat, or if a scope question has no answer on physics.

## 6. Measurements

0. **Baseline**: `chamber_latent_heat` on today's tree (the switched lamp, night stomata) — the dark
   windows' latent, the heater and cooler books. Log `W:\temp\claude\latent-heat-build\baseline.txt`.
1. **The lab switch** (`station::sealed::with_latent_heat`, lab-only; the loader unchanged): the window
   identity (heat drawn over a window = λ × the window's moved water) and the off-path byte identity
   as tests.
2. `chamber_latent_heat -- --latent` (the sealed station, 1220 days): the chamber's books (lamp, walls,
   heater, cooler, latent; heater, cooler-saturated and heater-saturated minutes; cooler + latent over
   200 W), the chamber's temperature range, the battery's minimum, the vapour bound, the end state
   against today's golden.
3. The four surroundings, heater books only.
4. A scratch worktree with the switch wired in: `regen_goldens` (report) and the suite.

## 7. Measured (2026-10-15)

**The lab switch** (`station::latent_heat::with_latent_heat`; `pub` λ in `science.rs`; the lab
`water_on_fast_step` refuses a build carrying the recorder) and its tests,
`rust/crates/station/tests/latent_heat.rs`: the window identity over four days, the switch's own pieces,
the two refusals, a fresh build only. **Liveness:** the heat × (1 + 1e-9) turns the identity red.

**The probe** (`chamber_latent_heat -- --latent [--surroundings=…]`, which also gained the re-sow days and
the crop at each maturity). Logs: `W:\temp\claude\latent-heat-build\` — `base3_outdoor.txt` /
`latent3_outdoor.txt` (the reference walls, without / with), `base_structure.txt`,
`latent_{structure,cabin,space}.txt`.

| | measured | predicted | grade |
|---|---|---|---|
| Q1 window identity | max relative **5.6e-15** over 19 504 windows; booked = λE = **6.948 GJ** | ≤ 1e-12 | **held** |
| Q2 heater | **1.340 GJ** (+1.034); warm minutes **584 549** | 1.0–1.3 (+0.7–1.0); ~330–370 k | **missed high**: the heater also runs by DAY (below) |
| Q3 cooler | **2.586 GJ** (cold 0.946, warm 1.640) | 2.2–2.6 | held, at the edge |
| Q4 shared coil | cooler + latent > 200 W: warm **1816** min (cooler alone at run 0: 1800), cold 720; the cooler alone: warm **0**, cold 1770 | ≈ run 0's saturation; cooler alone < 100 | **held** — the cooler's warm saturation vanished and the coil's did not: the side note was right |
| Q5 heater saturation, reference walls | warm 1237 min, cold 810 (run 0 **1250 / 810**: the warm-ups); warm minimum **−10.89 K** (−11.18) | none beyond the warm-ups; 11.2–11.8 K | held; the dip **missed** (shallower) |
| Q6 the plants | **each season one day longer** (re-sown day 140, 280 … 1120; run 0 139, 278 … 1112); grain at maturity **+3.4 %** in every season (14.576 → 15.070 mol C), leaf −1.7 %, stem +0.5 %, root +0.2 %; water −0.19 %; the day-1220 end state is **8 days earlier in its season** (storage 0.895 → 0, leaf −18 %) | carbon ≤ 0.1 %, water ≤ 0.3 % | **MISSED** (mechanism below) |
| Q7 battery | **−1.034 GJ** (1.0865e10 → 9.831e9); `rationed` 0 | −0.6–1.0 GJ | missed narrowly high (Q2's heater) |
| Q8 node colder | 0 minutes | 0 | held |
| Q9 vapour bound | 0.000–3.363 kg: the air **starts dry** (0 kg at step 0), so λ × range **8.24 MJ**; 0.8 % of the heater's rise | 5–6 MJ; < 1 % | the range missed (the dry start), the conclusion held |
| Q10 first window | 1.26e-3 kg, dark, **0.57 W** | < 1 W | held: the chamber's start stays |
| Q11 structure walls | the heater saturates **547 113** warm min; the room falls to **−49.4 K**; **the battery empties** (1.8e4 J) and the fast side **rations 763 338 times** | saturates; −0.5–5 K; battery 0.4–1.0e9 J above zero | **REFUTED** |
| Q12 cabin, space | heater saturated only at the warm-ups (warm 1143 / 1431 min, cold 810); no rationing; heater 0.98 / 1.92 GJ | none beyond the warm-ups | held |

**Why the heater runs by day (Q2).** The lit crop's latent power averages 140 W against the lamp's 178 W
over a lit window, but with a grown canopy it exceeds the lamp less the walls, so the room needs heat with
the lamp on (heater 37.7 W mean over its warm minutes, latent 106 W over them). Penman–Monteith's
air-dryness term does it: the dehumidifier holds the air at 75 %, and with `ra` 7.5 the dry air drives
transpiration beyond what the light pays for — the energy comes out of the room's air, and the heater
replaces it. This is the "dehumidify, then reheat" cost a controlled-environment room's HVAC pays.

**Why the season is a day longer (Q6).** The room's temperature by day falls by the deadbeat offset of
~140 W less input (`input·τ/C`: ~0.07 K above the setpoint → ~0 or just below), a ~0.3 % cut in the crop's
thermal time; ripening is checked at day ends, so the season crosses into one more whole day, and that day
is grain fill (+3.4 %). The size is a step of the day-end check, not a smooth response to 0.05 K: a run
whose maturity sat further from a day boundary would have moved by 0 days.

**Why the structure walls empty the battery (Q11).** They lose ~184 W to the node around the clock. The
lamp's daytime surplus paid part of that; evaporation now takes the surplus, so the heater must find the
evaporation's whole heat as well — ~66 W more on average, ~6.3 GJ over the run (heater 9.69 → 16.03 GJ)
against a battery that ended 1.48 GJ above zero. Not a fault of the form: that lab room cannot afford its
own transpiration. Its slow test (`chamber_walls.rs`,
`the_structures_heater_does_not_empty_the_battery_within_the_horizon`) asserts the opposite and would go
red at adoption.

### 7a. The season's extra day, measured (advisor, 2026-10-15)

The eight seasons are one event repeated (each re-sow resets the crop), so the cause was measured rather
than argued. The probe now prints, at each re-sow, the thermal time past maturity and the last day's gain,
and the room's mean temperature against its setpoint by window class (`base4_outdoor.txt`,
`latent4_outdoor.txt`).

| | run 0 | latent heat booked |
|---|---|---|
| room, warm lit windows | +0.064 K | **+0.008 K** |
| room, warm dark windows | −0.032 K | −0.040 K |
| room, cold windows | +0.045 / +0.018 K | +0.045 / +0.015 K |
| overshoot past maturity at the re-sow | **2.2–2.8 °C·d** of a 22.05 °C·d day (0.10–0.13 of it) | 20.6–21.1 °C·d (0.94–0.96 of the day) |

**Run 0's crop matured with 0.10–0.13 of its last day to spare.** The cooler lit room (−0.056 K over two
thirds of the warm windows) costs the season ~3 °C·d, more than that margin: the crop misses day 139 by
~1 °C·d and matures in the first ~6 % of day 140, so it gets one more whole day of grain fill (+0.49 mol
C, +3.4 %). The +3.4 % is a **step of the once-a-day maturity check**, not a smooth response: a crop sitting
more than ~0.15 day from the boundary would have moved by 0 days, and the smooth effect of ~0.15 day of
slower development is a fraction of one day's fill.

### 7b. The price of adoption, measured in a scratch worktree (§6 step 4)

The switch wired into `build_sealed_station_in` (after `wrap_last`), a worktree at `cd37dd6`; logs
`W:\temp\claude\latent-heat-build\wt_{regen,suite,ignored}.txt`.

**Q13 held:** `regen_goldens` changes `sealed_station_state` only; the other 19 identical.

**The reds** (17; none yet restated):

| test | why | kind |
|---|---|---|
| `latent_heat.rs` × 3 | written for a reference without the heat (a second recorder; the refusal now fires on the reference build) | rewrite: the switch becomes the lab-off one |
| `manifest_writer` | a new flow type, `ChamberLatentHeat` | regenerate (the ceremony) |
| `golden_regression` (ignored), `tier_contract` (ignored, aux key count 11 vs 12) | the golden | regenerate |
| `session_save_load` resume across a season (ignored) | pins the re-sow at day 139; it is now 140 (§7a) | restate the pin; the resume itself held |
| `chamber_heat` dead cooler | the nominal chamber sits 5e-4 K off its old fixed bound | derive the band |
| `chamber_walls` cabin, dead lamp, structure node, reference node (ignored) | energy identities lack the latent term; pins | restate: add the term, re-pin |
| `chamber_walls` structure battery (ignored) | **the battery empties; 763 338 rationed steps** | **a decision** (§8) |
| `lamp_shed` × 2 | an energy identity lacks the latent term; a dark day end is no longer exactly at the setpoint | restate |
| `air_split` × 2 | the separate-air builds assemble their own registries and **silently lack the heat**, so they are compared with a reference that has it | **a decision** (§8) |

`minute_transpiration.rs` and `scorecard_tm102788.rs` stayed green (they do not build through
`build_sealed_station_in`).

**Wiring for adoption (advisor):** inside the reference build, not through the lab wrapper — its `n = 0`
guard and seeded key would make any state saved before the change error at its first minute. A
pre-adoption save needs an explicit answer (seed the key at load, or refuse with a clear message).

## 8. Adoption (2026-10-15)

**The user's answers** (all four as recommended): **adopt**; the structure walls' slow test **restated to
the new truth** (the battery runs out, on a measured day); the **separate-air build gets the heat** (its
water stays on the plant step) and the **minute-water lab option opts out** with a note; the coil's shared
200 W **documented, not built** (§1b; the shared-coil count stays in the probe).

### 8a. What changes

* **Inside the reference build** (`build_sealed_station_unread`, so every sealed build and the
  separate-air build inherit it), not through a lab wrapper: the transpiration flow shared with the
  recorder, the recorder appended to the plant aux, `EVAPORATION_RATE_AUX` seeded at 0; and
  `ChamberLatentHeat` in `sealed_fast_flows` (the one canonical fast list, which the separate-air build
  re-uses with its own cabin air).
* **The lab off switch** `station::latent_heat::without_latent_heat`: removes the recorder, the fast flow
  and the seeded key (the shared flow stays: it delegates, so it computes what the plain flow computes).
  The lab `with_latent_heat` goes.
* **Minute water** (`air_split::build_split_station` with `transpiration: Minute`) applies the off switch
  before moving the water, and says so; `water_on_fast_step` keeps refusing a build that carries the
  recorder, so nothing reaches the minute step silently.
* **A save from before the change** carries no rate key. Loading keeps the session's registries; the
  first plant step after a group boundary records the key (the additive channel starts it from 0), so a
  save at a group boundary resumes; one taken between a plant step and its minutes errors at the first
  minute with the flow's own message.
* The station manifest's `flow_set` gains `ChamberLatentHeat`; `sealed_station_state` regenerated.

### 8b. Predictions (written before the build changes)

| | prediction |
|---|---|
| A1 | `regen_goldens`: `sealed_station_state` only, and its bytes **identical to the §7b worktree's** (the wiring moves the recorder inside the plants' wrappers rather than sharing the wrapped flow; both see the same environment) |
| A2 | the off switch on the adopted tree reproduces the **pre-adoption** golden byte-exact |
| A3 | the reds of §7b, with two changes: `latent_heat.rs` rewritten for the default; the separate-air pair (`air_split.rs`) **turn green** without a restatement once the split build has the heat on both sides of its comparison — unless its minute variant moves the water, in which case it stays red and is restated as minute water without the heat |
| A4 | the structure walls' battery runs out between days **900 and 1220** (§7: it ends at 1.8e4 J; the heater needs ~6.3 GJ more against a 1.48 GJ margin) |
| A5 | anything else that builds through `build_sealed_station_unread` (the TM 102788 scorecard, the sowing-clock and gas-exchange tests, the shedding example) moves only as §7 did: water ≤ 0.3 %, and season-dated pins by one day |

### 8c. Graded (2026-10-15)

| | measured | grade |
|---|---|---|
| A1 | the adopted `sealed_station_state` **byte-identical** to the §7b worktree's (`cmp`), before the ledger below | **held** |
| A2 | `chamber_latent_heat -- --no-latent` on the adopted tree, against the still-committed pre-adoption golden: **byte-exact** | **held** |
| A3 | the separate-air pair turned green unrestated (its minute variant moves the gas exchange, not the water); the other reds as §7b, plus two `minute_transpiration.rs` tests §7b could not see (their builds go through `build_sealed_station_unread`, which the trial did not wire) | held, with two unforeseen reds |
| A4 | the structure walls' battery runs out **by day 1025** (rationing 763 338; 410 saturated nights before it) | **held** (900–1220) |
| A5 | the TM 102788 scorecard, sowing-clock and gas-exchange tests green; season-dated pins moved one day (`session_save_load`'s first re-sow 139 → 140) | held |

**One change after the predictions:** a running total of the evaporated water
(`station::latent_heat::EVAPORATED_TOTAL_AUX`), written by the recorder beside the rate. At a day end
every recorded window's heat has been drawn, so the heat booked to date is λ × the total — a ledger any
state can be read against. It was added because three energy identities (`chamber_walls` L1,
`lamp_shed` H4, `chamber_heat`'s dead cooler) needed the span's evaporation from states alone; it adds a
second aux key to the golden (A1's identity was checked before it). Its own test: the minute legs summed
over four days equal λ × the total to 1e-12.

**The reds, restated** (none deleted, none widened without a derivation):

* `latent_heat.rs` — rewritten for the default: the build carries the books, the lab switch takes
  exactly them out, minute water needs them off, a keyless save resumes; the window identity and the
  ledger.
* `chamber_walls.rs` — the run helper is the reference day observed (`TwoRate`, interleaved) so it sums
  the latent legs; L1's heater identity gains the evaporation (walls 14.5 MJ + evaporation 12.0 MJ +
  store = the battery gap) and its held band one evaporation step; L2's closed form takes the lamp less
  the run's mean evaporation (4.6 W); L4 (120 days) re-pinned (node 189.2987 K, heater 136.919 W) and
  the room **not held** at its day 120 end (0.9526 K below: the heater saturated earlier that night);
  the reference node re-pinned (162.4912 / 176.3676 / 170.4416 K, ends 295.1354351 K), its band one
  evaporation step wider per state (9 day ends outside, as before); the structure horizon restated to
  the battery running out (the user's answer).
* `chamber_heat.rs` — the nominal room sits one evaporation step below its start at each dark day end
  (derived); the dead cooler's day-2 rise is the lamp less the evaporation (day 1 carries ~20 kJ of the
  healthy heater from the dark hours); its node bound starts where the dead crop stops evaporating (day
  11); the nominal node's lamp-only derivation is held on the books-off run, and the books-on node sits
  1.79 K above it, inside the 14.6 K the evaporation could pay for.
* `lamp_shed.rs` — H4's identity gains the evaporation from the ledger; the two rooms' dark day ends
  differ by exactly their evaporation steps.
* `minute_transpiration.rs` — the minute form carries no books (fast side +2 −1); the half-pair refusal
  switches the books off first.
* `session_save_load.rs` — the first re-sow is day 140 (§7a).

**A standing caution (a side note, 2026-10-15):** the reference crop now matures ~6 % into its last day,
closer to the line than before (11 %). A later change warming the room by ~0.03 K could take the day back
and move grain ~−3.4 % with no mechanism behind it. Read a ~3 % grain move against the re-sow days first.
