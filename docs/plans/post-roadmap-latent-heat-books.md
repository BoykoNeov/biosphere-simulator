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

⚠ Within a window the lamp's draw and the latent draw are both uniform (the lamp's top-hat mean; the
window's rate), so a window's per-minute deficit is its mean deficit: the run-0 "unfunded" sums are the
heater's new work exactly, not a lower bound — before the walls and the plants answer the changed room.

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
