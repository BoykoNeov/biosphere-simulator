# The cabin's CO₂ level — the scrubber, re-read as a level (2026-10-19)

**Opened 2026-10-19** on the user's *"1"*: the first of the uncited-value triage's shortlist
(`post-roadmap-uncited-triage.md` §6, item 1 — "the scrubber rate"). A design note and a price,
lab-only: **no value, golden, manifest or `source:` string moved.** One lab example was added
(`rust/crates/station/examples/co2_level_price.rs`); it writes nothing. The choice of level is the
user's (§7).

## 1. What today is, measured

The cabin's CO₂ is set by **nothing anyone chose.** The scrubber is first-order
(`removed = k · cabin_co2`), so the cabin settles where the crew's output equals the removal:
`n = P / k = 3.796e-3 / 1e-3 = 3.796 mol`, **399.6 ppm** of the 9500 mol cabin — Earth's outdoor
air. `k = 1e-3 /s` was picked for solver stability (`eclss.yaml`: "k·dt = 0.06 < 1"), and the
greenhouse scenario is filled *at* that fixed point (`chamber_co2_mol0 = 3.796`, the
crew-coupled-loop record's PIN 5). The crop shares this air, so the crop's CO₂ is the same accident.

Real cabins hold far more. **BVAD Rev 2 Table 4-1, printed p. 63, read as the page image**
(`sources/temp-evidence/bvad/p63-077.png`; the extracted text scrambles this table's columns):

| Row | Lower | Nominal | Upper | Reference |
|---|---|---|---|---|
| p[CO₂] for Plants (kPa) | 0.04 (Earth normal) | **0.13** (accepted optimum for plant growth) | 3.4 (NASA Std. 3001) | |
| p[CO₂] for Crew (kPa) | 0.267 (Law 2010) | **0.507** (Law 2010; footnote 83: "may be reduced to 0.267 kPa nominal 24-hour average in future NASA-STD-3001") | 1.01 (Law 2010) | |

And the text of p. 62: *"ISS has recently adopted a lower maximum value for CO₂ of 0.40 kPa as
per directive from Mission Control"*; p. 62 also lists the symptoms from 0.307 kPa up. The plant
chapter (text line ~7832 of `bvad.txt`): the plant optimum is *"roughly 0.10 to 0.20 kPa (Wheeler,
et al., 1993)"*, and *"plants do appear to have reduced productivity at very high partial
pressures of carbon dioxide that are considered within the normal range for crew (up to about
1.0 kPa)"*.

At 101.325 kPa, 0.13 kPa is 1283 ppm, 0.40 kPa is 3948 ppm, 0.507 kPa is 5004 ppm. Today's
400 ppm is **below BVAD's lowest crew value by 6.7×** and sits at the plants' *lower* bound.

## 2. ⚠ BVAD carries no scrubber equipment figures

The 2026-09-29 review (Step 5, item 2) said *"BVAD is on the shelf and carries equipment figures"*.
**Rev 2 does not, for CO₂ removal**: grepped `bvad.txt` for CDRA, four-bed, molecular sieve,
swing bed, amine, LiOH, "carbon dioxide removal" — the only hits are the EVA suit's swing bed
(§ on EVA consumables) and a trade-study word list. The one CDRA figure in the tree is still
`eclss.yaml`'s [C] (Watanabe et al. 2015, NTRS 20150016500): 34.7 m³/h airflow, 144-min cycle.

## 3. Why the level, not the hardware

The first instinct — size `k` from the real machine — carries a hidden sizing into the result:

* For a flow-through bed, removal ≈ airflow × concentration × capture efficiency, which **is**
  first-order in the cabin's concentration: `k = Q·η / V`. The cabin's volume is not a model
  quantity; at 22 °C and 101.325 kPa, 9500 mol is `nRT/P` ≈ **230 m³** (our arithmetic). One CDRA
  then gives `k ≈ 34.7 / 3600 / 230 ≈ 4.2e-5 /s` before `η` — about 24× slower than today.
* But the cabin's level is `P / k`, and **this station's crew is 13.3 crew-members' worth of
  carbon** (`post-roadmap-crew-coupled-loop.md` §3: `f_resp · food_intake_rate · 86400 / 24.654`)
  — while it drinks one crew-member's water. At one CDRA the cabin settles near 90 mol, **~0.96
  kPa**, just under BVAD's 1.01 upper bound. That number would be a fact about the crew's
  oversizing, not about scrubbers. This repo has already paid for one such hidden sizing: the
  1995 mol O₂ target that silently encoded the 9500 mol cabin (`eclss.yaml`, `o2_fraction`'s
  history).
* ⚠ **The triage's "÷8" row was the wrong arithmetic.** It took the CDRA's 144-min bed cycle as
  "~8× our τ" and scaled `k` by 1/8. A swing period is not a removal time constant; the airflow
  estimate above gives ~1/24 before efficiency. The ÷8 and ÷30 rows remain correct as
  *measurements at those rates* (they are the 3200 and 12 000 ppm points); their *label* is wrong.

**So the cited quantity is the level, not the machine.** This is the move the repo has made twice:
`o2_setpoint` (mol) became `o2_fraction`, and the condenser draws only the excess above
`humidity_fraction` of the cabin's air. The scrubber's rate stays a `DESIGN` value, but it then
only sets how fast the cabin returns to its level, not where the level is.

## 4. The bed-swing ripple — estimated, not built

`eclss.yaml` says *"no source CAN fix it — real CDRA is cyclic swing adsorption, not first-order
decay"*. Whether the cycle matters is a size question: one CDRA turns the cabin's air over in
`V / Q ≈ 230 / 34.7 ≈ 6.6 h`, against a 72-min half-cycle. A **worst-case** bound — removal
stopping entirely for a half-cycle and doubling for the next — swings the cabin by
`P · 4320 s / 2 ≈ 8.2 mol` at this crew: ±17 % of the 0.507 kPa level, ±67 % of the 0.13 kPa one.
A real CDRA always has one bed adsorbing, so the real swing is a fraction of this; the bound says
only that a ripple model is **not obviously idle at the plant level** and is not worth building
before a level is chosen. Recorded, not built.

## 5. The price — four levels, the sealed station, four years

`cargo run --release -q -p station --example co2_level_price` (from `rust/`, at below-normal
priority, 2026-10-19). Each run uses the frozen first-order form with `k = P / n_target` and the
cabin started at `n_target`. That is not the form proposed in §6. At steady state both hold the
same level, so the crop's response is **expected** to carry over. That is unmeasured: the
one-sided form stops entirely when the crop pulls the cabin below target, where the first-order
form only slows. At this crew's 328 mol/day against the crop's ~0.6 that should be negligible,
and it is owed a measurement at the lab build. Raw output:
`sources/temp-evidence/co2-level/co2-level-price.txt`. **Baseline reproduced:** the "today" run
scrubbed 400 073.8 mol C, and the frozen golden's final `boundary.co2_removed` is
400 073.759 (`rust/data/golden/sealed_station_state.json`), so the tool runs the golden's build. Readings are day-boundary values;
plant C (leaf + stem + root + grain) and grain are each year's peak, in mol C on the 1 m² bed.

| Level | Cabin CO₂ (ppm, yr 1–4 mean) | Plant C peak | Grain peak | vs today |
|---|---|---|---|---|
| today (`k = 1e-3`, nobody's choice) | 400 | 31.74 | 15.07 | — |
| **0.13 kPa**, plants' nominal | 1284 | 48.16 | 22.40 | plant **+51.7 %**, grain **+48.6 %** |
| **0.40 kPa**, ISS's operating cap | 3951 | 53.40 | 24.93 | plant +68.3 %, grain +65.4 % |
| **0.507 kPa**, crew nominal | 5007 | 53.96 | 25.21 | plant +70.0 %, grain +67.2 % |

All four: `rationed 0`, `events 0`, no failure. The four years agree with one another to the
fourth figure. The scrubbed carbon is the same within 0.014 % (~400 000 mol C). The crew sets the
throughput, and the crop's share of it is tiny: the station's 13.3 crew-members against 1 m² of
wheat, the 187× of the crew-coupled-loop record.

**What the numbers say:**

* **Most of the gain comes in the first step.** 400 → 1284 ppm buys +49 % grain. Tripling again
  to 3951 buys another +11 points, and 5007 two more. The crop model saturates. Its
  photosynthesis (FvCB) has no high-CO₂ penalty; grepped `domains/src/biosphere` for a triose-
  phosphate limit, inhibition and "elevated": nothing.
* **The model has been checked at one level, 1000 ppm, and nowhere above it.** The one real-world
  check of the crop's gas exchange (`log/real-world-checks.md`, TM 102788) held CO₂ at 1000 ppm,
  on the open-field build with a fixed leaf CO₂, not this sealed station. Uptake read 0.98 of the
  measured value. BVAD's own text says real plants
  lose productivity at CO₂ levels inside the crew range, up to about 1.0 kPa. The model cannot
  show that loss. At 0.40 and 0.507 kPa the extra +17–19 points over 0.13 kPa are therefore the
  model's extrapolation, and the true sign may be the opposite.
* **Only the crop was priced.** Water and every other stock were not read here. The goldens that
  share cabin air with a crop (the sealed station, greenhouse and harvest) would all move, and by
  this much in carbon. The station's other goldens move under the form change in §6, whatever
  the level.

## 6. The form, if a level is adopted

Proposed, following the condenser's precedent: **`eclss.yaml` stores a cited CO₂ share
(`co2_fraction`). The scrubber removes only what is above that share of the cabin's air:**
`removed = k · max(0, cabin_co2 − co2_fraction · air) · dt`. The target in mol is computed where
the air is known, exactly as `o2_setpoint_mol` and `humidity_setpoint_kg` already are. (⚠ **As
built, it is stored as the pressure, `co2_partial_pressure: 130.0 Pa`, not as a fraction** (§10):
the unit Table 4-1 prints, and the lab's `130 / 101 325 · air` bit for bit, where a decimal
fraction would have moved the last bits of every crop-sharing golden.) A
standalone cabin with no air of its own reads `STATION_CABIN_AIR_MOL`, as the humidity target
does. `max` already sits in the condenser under the Tier-1 claim (`+ − × ÷` and comparisons, no
transcendental), so the three Tier-1 goldens keep their tier.

⚠ **A one-sided proportional remover holds above its target, not at it.** At steady state the
crew's output equals `k · (n − target)`, so `n = target + P/k`. At the frozen `k = 1e-3` that
offset is **3.796 mol, a full 400 ppm**: today's whole level would sit on top of the cited
one. So the form change carries a **rate change**. At `k = 1e-2 /s` (`k·dt = 0.6 < 1` at the
60-s cabin step) the offset is 0.38 mol, **40 ppm**: 3 % above 0.13 kPa, 0.8 % above 0.507 kPa.
The rate stays `DESIGN`: it sets how tightly the level is held, not where it is. ⚠ **The authored
multirate path already breaks this bound at today's rate**, and a ×10 rate makes it worse.
`authoring/src/interpreter.rs` (`effective_step`) records that with the scrubber classed slow at
a 3600-s master step, it steps at 1800 s, where `k·h = 1.8` at today's `k`. That run rations and
empties the cabin's CO₂. At `k = 1e-2`, `k·h` is 18. Any authored file that classes the scrubber
slow must class it fast, or the build must refuse it. Check this before building.

What the change reaches (reach list, not yet a build):

* `domains/src/eclss.rs` (`scrub_flux`, `CO2Scrubber`, `EclssParams`, the in-source tests);
  `eclss.yaml`; `params.rs`; `sibling_params.txt`.
* Station builders `cabin.rs`, `water.rs`, `sealed.rs`; the greenhouse start value
  (`chamber_co2_mol0 = 3.796`, which must move to the new level or the run opens on a transient);
  the perturbation tests that assert a return to `P/k` (`perturbations.rs`
  `assert_returns_to_setpoint`).
* The authoring registry (`flow_registry.rs`), since a renamed param is an authoring unfreeze.
  Also the authored files that write `co2_scrub_rate`: `scenarios/bioregenerative_station.yaml`,
  `rust/data/scenarios/param_sets_dsl.yaml` (×3), `rust/data/scenarios/packs/eclss_hot_makeup.yaml`.
* Goldens: every station golden that reads the scrubber, with the station unfreeze ceremony. The
  station and authoring manifests are regenerated.

**Build order if adopted:** lab-only behind a switch first, as the night-stomata and latent-heat
items were. Write the predicted golden moves first, then measure them, then adopt.

## 7. The user's choice

**Which CO₂ level should the station's cabin hold?** Plants and crew share one air here, so one
number serves both.

1. **0.13 kPa (≈1280 ppm), the plants' nominal. Recommended.** It is cited. It is near the one level
   (1000 ppm) where the crop model has been checked against a real chamber. It is safe for
   the crew: below BVAD's lowest crew value, so the scrubber works harder than a real cabin's,
   which is allowed. Grain +49 %.
2. **0.40 kPa (≈3950 ppm), what the ISS runs to.** Realistic for the crew side. The crop's extra
   gain over option 1 is extrapolation the model cannot check, and BVAD says the real effect there
   is a loss.
3. **0.507 kPa (≈5000 ppm), BVAD's crew nominal.** The same as 2, a little further out. Footnote
   83 says it may be lowered to 0.267 kPa.
4. **Keep today's 400 ppm** and re-label it honestly: `DESIGN`, "Earth outdoor air, below every
   cited cabin value". Nothing moves.

Not priced: 0.267 kPa (≈2630 ppm, the crew lower bound and the possible future nominal). It lies
between rows 2 and 3 of §5's table, where the curve is already flattening. Measure it before choosing it.

## 8. Not this item

* **§2.2 of the third direction plan (the chamber CO₂ setpoint controller) is NOT decided by
  this.** That controller *adds* CO₂ to plant-only chambers that run out, and its two-sided form
  vents carbon. This item concerns the crew cabin's *remover*: it can only take CO₂ out, and the
  only change is the level it stops at.
* The crew's 13.3× carbon sizing is not touched; it is the reason the level must be cited rather
  than derived from hardware (§3), not a thing this item fixes.

## 9. The user's answer, and the lab build (2026-10-19)

**The user chose 0.13 kPa, and the crew-size fix as the next item** (*"1300, option 1"*; the crew
question is in §8 and the record). The crew stays 13.3 crew-members' worth of carbon for this item.
Order matters: with the level held by a target, correcting the crew later cannot starve the crop. At
today's first-order rate, a one-crew-member load would settle the cabin at about 30 ppm.

**Built lab-only (stage 1, inert plumbing plus a switch):**

* `domains::eclss::ScrubForm` — `FirstOrder` (the loader's, frozen) or `AboveTarget {
  partial_pressure_pa }`, a field on `EclssParams` that `eclss.yaml` does not carry.
* `CO2Scrubber::with_cabin_air(air)`: the three station builders (`cabin.rs`, `water.rs`,
  `sealed.rs`) now tell the scrubber the station's 9500 mol cabin; the standalone build tells it
  its scenario's air. Under `FirstOrder` the air is never read (a flow-level test pins this; the
  goldens pin it end to end). Under `AboveTarget` with no air, the flow refuses at its first step.
* `domains::lab::with_scrubber_above_target(p, CO2_TARGET_PA = 130.0, ABOVE_TARGET_SCRUB_RATE =
  1e-2)`.
* Not touched: `eclss.yaml`, the authoring registry, every scenario. An authored file reads the
  loader's params, so it cannot reach the switch.

### Predictions, written before the measurement

The six goldens that read the ECLSS params are `eclss` (domains), `cabin_gas`, `water_recovery`,
`greenhouse`, `harvest` and `sealed_station`. `sealed_energy_drift` builds Power + Thermal only and
cannot move. In the lab runs the crop-sharing scenarios start at the new target
(`chamber_co2_mol0 = 12.188`); the crew-only cabins keep their frozen start of 0.

1. **`eclss`, `cabin_gas`, `water_recovery` (crew-only cabins):** only `cabin_co2` and
   `boundary.co2_removed` move; every other stock is bit-identical, because no other flow reads
   the cabin's CO₂. The final `cabin_co2` is target + P/k: 12.188 + 0.300 = **12.488** for
   `eclss` (P = 0.003), and 12.188 + 0.380 = **12.568** for the other two (P = 3.796e-3). Starting
   from 0, the cabin fills in about 68 steps and then settles with a 100-s time constant, well
   inside the 900-step horizon. `co2_removed` falls by the rise in the final stock: ~9.49 and
   ~8.77 mol.
2. **`greenhouse`, `harvest` (7 days, crop shares the cabin):** `carbon_pool` reads ~12.57. The
   plant carbon pools rise, by less than the sealed station's +52 % (a seedling, or a crop past
   flowering, over 7 days). O₂ moves through the regulator, since the crop makes more. Water moves
   only as far as leaf area does: small, and possibly zero in `harvest`, whose crop is past
   flowering and sheds leaves rather than grows them.
3. **`sealed_station`:** within 1 % of §5's first-order price (plant C +51.7 %, grain +48.6 %),
   since both forms hold the same level to 40 ppm. Water and the chamber's heat move with the
   larger canopy; how much is not predicted.

### Measured against the predictions (2026-10-19)

`cargo run --release -q -p station --example co2_target_lab` (raw:
`sources/temp-evidence/co2-level/co2-target-lab.txt`). On each of the six builds the frozen side
reproduced its committed golden **byte for byte**, before any comparison. Every switched run had
`rationed 0`, `events 0`.

| Golden | Stocks moved | Graded |
|---|---|---|
| `eclss` | 2 of 9: `cabin_co2` 3.0 → **12.488502**, `co2_removed` −9.488 | **1 HELD exactly** |
| `cabin_gas` | 2 of 10: `cabin_co2` 3.796 → **12.568102**, `co2_removed` −8.772 | **1 HELD exactly** |
| `water_recovery` | 2 of 10, the same two numbers as `cabin_gas` | **1 HELD exactly** |
| `greenhouse` | 19 of 30: `carbon_pool` 12.568; leaf +26 %, root +15 %, stem +8 %; condensate +0.32 %, soil water −0.15 %; O₂ supply −0.002 % | **2 HELD.** One move not predicted: `chamber_inert` −0.112 % |
| `harvest` | 21 of 29: grain (`storage_c`) +47 %, stem +13 %, leaf +9 %; condensate +0.16 %; `crew.food_store` +0.0004 % | **2 HELD, except** the water: predicted "possibly zero", measured +0.16 % condensate. Small, but not zero: the crop past flowering still grew leaf (+9 %) |
| `sealed_station` | 26 of 36 (final day) | **3 HELD** on the crop, by the like-for-like readout below. Water: see below |

**The `chamber_inert` move was not predicted, and it follows from construction.** The chamber's inert
gas is charged as "the rest of the air" (`system::chamber_inert_charge_mol`: capacity − CO₂ − O₂),
stored in kg. Starting the cabin 8.392 mol higher in CO₂ charges 8.392 mol less N₂ = 0.2351 kg, the
measured 210.134228 → 209.899126. The chamber still starts at exactly reference pressure. This is
part of moving the start value, in every crop-sharing build. **At adoption it goes into the
predicted set.**

**Prediction 3, the crop, measured like for like** (`co2_level_price -- above-target`, raw:
`.../co2-level-price-above-target.txt`): the cabin holds **1323 ppm**, the target's 1283 plus the
predicted 40-ppm standoff. Plant C peaks +52.5 % and grain +49.4 % on every year, against §5's
first-order +51.7 % and +48.6 %: 0.5 % apart, inside the 1 % predicted. The final-day stocks in the
table read higher (root +103 %, leaf +78 %). That is a different readout: one mid-season day,
where a crop that started faster is further ahead than at its yearly peak.

**The sealed station's water — what moved and what did not.** On the final day, condensate
86.01 → 19.52 (−77 %) and soil water 79.66 → 135.51 (+70 %), with subsoil +41 %. **Their sum is
unchanged to six decimals** (191.653614 both sides), and `water_vapor` is unchanged. The chamber's
water is closed, and only its split between the condensate store and the soil differs on the final
day. The reference waters a sealed chamber's soil from its condensate in events. Since the
water-forms adoption (2026-10-10, `post-roadmap-soil-evaporation.md` §16), `EventRecycling`
fires when the root zone falls to FTSW = 1 − p = 0.45 (`water.yaml`, FAO-56 Table 22). A soil that
fills in pulses and drains between them has a phase. The larger canopy drains it faster, so the
pulses shift, and a single final day lands at a different point in the cycle. That is the reading
the conserved sum supports. ⚠ It is a reading, not a trace: a day-by-day series of the two stocks
would confirm it, and that is owed with the transpiration total below.
⚠ **Not measured: the transpiration total over the run**, the readout that would say how much more
water the larger crop moves. The final state cannot show it. Owed before adoption, and the
station's water readouts in the TM 102788 scorecard (`real-world-checks.md`) should be re-read with it.

**Also moved in `sealed_station`, not predicted in size:** battery −0.51 %, `boundary.space`
+0.12 %, the thermal node +0.04 %, the chamber's surroundings −0.07 %. The energy books move with
the crop, presumably through the chamber's heat (lamp, cooling, latent). The path was not traced.

### The water total, measured (2026-10-19)

The readout owed above. `cargo run --release -q -p station --example co2_water_lab` (from `rust/`,
below-normal priority, 455 s; raw: `sources/temp-evidence/co2-level/co2-water-lab.txt`, day series:
`.../co2-water-series.csv`). The tool runs the sealed station's four years twice through the lab's
two-rate driver in the reference's interleaved order. It **asserts** that the frozen side's final
state is `sealed_station_state.json` byte for byte before printing anything. Every flow of both
sides is re-evaluated on each step's start state, and its legs into and out of the chamber's four
water stores (soil, subsoil, vapour, condensate) are summed by flow and by year.

**The instrument closes.** For each store and each side, delivered − drawn + the re-sow hook's
moves equals final − initial to within 1.5e-11 kg. The chamber holds exactly 195.000000 kg of
water on every day of both runs, so no water crosses to the cabin or the crew's loop. Both runs
had `rationed (0, 0)` and no events.

⚠ **`biosphere.transpiration` is the crop's AND the soil's evaporation**, one leg out of the root
zone since the water-forms adoption (2026-10-10). The closure above verifies their SUM (+10.8 %).
The tool splits them by the soil account's daily counter (`soil_evap_today`, restarted on each
day's first plant step). That account carries its own instance of the same flow, built by the same
builder, and the sealed build's wrappers (`plants_read_chamber`, `on_sowing_clock`) wrap aux
processes as they wrap flows, so both read the same chamber. The tool checks on every plant step
that the counter's share lies in `[0, the flow's soil leg]`: **0 steps outside, both sides.**

| Four years, kg | Frozen (400 ppm) | 0.13 kPa (1323 ppm) | Change |
|---|---|---|---|
| **Crop transpiration** | 2308.9 | 2625.8 | **+13.7 %** (yearly +12.8 to +14.7 %) |
| Soil evaporation | 285.2 | 247.6 | −13.2 % (the larger canopy shades the soil) |
| Both, out of the root zone | 2594.1 | 2873.4 | +10.8 % |
| Watering, condensate → soil | 2641.5 | 3006.5 | +13.8 % (33 → 39 watering days) |
| Condensation from the air | 23.3 | 22.8 | −2.1 % |

**Read against the crop:** the yearly peak of plant carbon rises +52.5 %, the four years' crop
water +13.7 % (different readouts: a peak stock against a flow total; their ratio is indicative
only). The model's water flows read no CO₂: `flows.rs`'s transpiration and canopy code carries no
CO₂ or `ci` input (grepped), so the extra water is the larger canopy's, not a change in the leaf.
BVAD's own canopy model (Rev 2 p. 187, the `gS` formula under Eq. 4-26, after Monje 1998; read from the extracted
text, not the page image) makes stomatal conductance proportional to net photosynthesis over the
CO₂ concentration, so a real canopy at this level would transpire less per unit of growth than
this model does. If anything, the model overstates the rise.

**The bigger crop is never short of water.** On every plant step of both runs the water-stress
factor is 1 (0 stressed steps; minimum 1.000000). Most waterings take all the condensate held (23
watering steps frozen, 24 switched), so the watering is supply-limited, but the FAO-56 trigger
(FTSW 0.45) sits above the stress threshold and the soil is refilled before the crop feels it.
That holds at today's level too; the higher level does not change it.

**The phase reading is confirmed by the day series.** The yearly mean soil water agrees between
sides to 1–4 % (62.8 / 63.0 / 61.4 / 66.8 against 63.6 / 62.4 / 63.7 / 68.7 kg), and the
condensate means scatter both ways (43 / 46 / 65 / 52 against 38 / 52 / 58 / 45). The switched
run waters on day 1215, four days before the end (soil 68.4 → 144.7, condensate 84.2 → 1.1). The
frozen run last watered on day 1205, 14 days before. That alone makes the final day's −77 % / +70 %
split.

**Also seen, already in the frozen run and not this item's:** the re-sow hook moves root-zone
water into the subsoil (768 kg over four years frozen, 810 switched), as a new seedling's shallow
roots reset the root zone. Recorded only.

**The TM 102788 scorecard cannot move.** It builds the open field with leaf CO₂ as a fixed
forcing (`station/tests/scorecard_tm102788.rs`, no ECLSS or sealed build anywhere in it), so the
scrubber cannot reach it. It is also the one real-chamber check of the crop's water, and it was
run at 1000 ppm, close to the adopted level: water 0.928 of measured (the figure since the lit-share
adoption, `log/lit-share-stomata.md`, quoted from the record, not re-run here).

**What adoption would cost** (from §6, unchanged): the station unfreeze ceremony; `eclss.yaml`
gains the cited `co2_fraction` (130 Pa of 101.325 kPa) and the 1e-2 rate; `params.rs`,
`sibling_params.txt`; the authoring registry and the three authored files (five entries) that set
`co2_scrub_rate`; six goldens (`eclss`, `cabin_gas`, `water_recovery`, `greenhouse`, `harvest`,
`sealed_station`) regenerated against the predictions already measured here, with the inert-charge
move now among them; the station and authoring manifests regenerated.

**The authored scrubbers, checked (resolves §6's "check this before building").** The authoring
build already refuses any first-order rate with `k·h ≥ 1` (`interpreter.rs`,
`check_rate_preconditions`, reading `effective_step`, slow set at `dt/2`); `effective_step`'s
1800-s note is that guard's history, not a live file. No authored file classes the scrubber slow.
Four of the five files that place one step it at 60 s (`k·h = 0.6` at 1e-2). The fifth,
`scenarios/bioregenerative_station.yaml`, reads `params: eclss` and steps its fast set at
`3600 / 12 = 300 s`: at 1e-2 that is `k·h = 3`, so **adoption would make that file refuse to
build** until its `n_sub` rises to 60 (60 s). That is an authored-content edit, in the adoption's
batch, with its tests re-run.

## 10. ADOPTED (2026-10-19) — slice A, the station

**On the user's word** (*"1"*, after the summary of §9). The station unfreeze, in the order the
advisor set: the lab's results saved as bytes first, then the reference changed, then the goldens
regenerated and compared against the saved bytes.

**What changed.** `eclss.yaml` gains `co2_partial_pressure: 130.0 Pa` (BVAD Table 4-1, plants'
nominal; stored as the pressure the table prints, not as the fraction §6 proposed — see the note
there) and `co2_scrub_rate` goes 1e-3 → 1e-2 with a new `DESIGN` source. `ScrubForm::AboveTarget`
is the loader's form; `FirstOrder` survives as the lab switch
`domains::lab::with_first_order_scrubber(p, FIRST_ORDER_SCRUB_RATE)`. `CO2Scrubber::new` takes the
cabin's air with no default (the condenser's and the regulator's rule), so the old "refuses at its
first step without air" path is gone, not tested. `greenhouse_bio_scenario`'s `chamber_co2_mol0`
starts at the target, 12.188502343942758 mol, pinned against `co2_target_mol` in
`the_cabin_starts_at_its_targets`. `sibling_params.txt`'s `co2_scrub_rate` row is retired by hand
(11 → 10), `o2_setpoint`'s precedent. The authoring side builds every scrubber against a named,
temporary `IMPLICIT_SCRUBBER_AIR_MOL = 9500` until slice B. `scenarios/bioregenerative_station.yaml`
moves to `n_sub: 60` (60-s fast step, `k·h = 0.6`), as §9 said it must.

**Measured against the lab, byte for byte.** Before the change, `co2_target_lab` dumped the six
switched final states (under `W:\temp\claude\co2-adopt\lab\`, deleted at the end of the batch).
After it, `regen_goldens --write` rewrote exactly six goldens (`eclss`, `cabin_gas`,
`water_recovery`, `greenhouse`, `harvest`, `sealed_station`), the other 14 identical, and **each of
the six is `cmp`-identical to its dump.** So §9's tables are the adopted reference's moves, not an
approximation of them. Station manifest, predicted and measured: `eclss.yaml`'s two fingerprints and
the six `golden_sha256` rows, nothing else. Biosphere and authoring manifests unchanged.

**The lab tools swapped sides.** `co2_level_price`, `co2_target_lab` and `co2_water_lab` now run the
loader's form as "frozen" (checked against the new goldens) and the pre-adoption first-order form
as the switched side, started at its old fixed point 3.796 mol. Nothing runs examples, so this was
done by hand; they compile under `clippy --all-targets`.

**Tests that went red (12 of 1452), each re-stated, none loosened:**

* `domains/tests/eclss_run.rs` (3). The steady state is `target + P/k`, not `P/k`. The CO₂
  contraction is linear only above the target, so both runs now start at or above it (the
  condenser test's arrangement), and a new test pins the one-sided half: below the target an
  offset carries undecayed and nothing is scrubbed. RK4 against Euler: both reach the exact fixed
  point to the last bit by step 900, so the "identity breaks" check moved from the final step to
  the trajectory.
* `authoring/tests/multirate.rs` (4). Fixture numbers at the old rate: the slow scrubber's true
  `k·h` is 18 (was 1.8), the deadbeat boundary is `dt = 100` (was 1000), and the probe cabin
  starts at 20 mol, above the 12.19 mol target, because below it the scrubber is idle and "bit
  identical" would compare two runs doing nothing.
* `authoring/tests/scenario_files.rs` (3). `k·h = 36` at `dt = 3600` (was 3.6); the rationing
  count at that step re-measured 28 → 19, the condenser's 2026-10-03 case again (a one-sided law
  overshoots below its target and stops rather than clamping at zero). The O₂ "airless cabin"
  assertion, which says the mechanism is the same one, still holds.
* `station/tests/chamber_walls.rs` (1). A measured pin, re-pinned as at every earlier adoption:
  node 188.7373 → 188.7014 K, heater 139.788 → 139.193 W. The tolerances are unchanged.
* `station/tests/air_split.rs` (1), **not re-pinned.** At 1283 ppm a fan-fed small chamber is far
  less hungry: the fan's 0.1 / 0.4 ratio reads 0.990 (fans 0.1 / 0.2 / 0.4 → 0.984 / 0.990 / 0.993
  of shared air's plant carbon at 90 days), past the pin's 0.97. Loosening the line to fit would
  weaken it (a side note raised exactly this). The file's subject is CO₂ supply to a hungry crop,
  measured at 400 ppm, so the whole file now runs at that level through the lab switch and the old
  start, and keeps 0.97. The separate-air option's behaviour at the adopted level is the reading
  above, recorded and not pinned.

`station/tests/perturbations.rs`, which the advisor flagged (does a spike push the cabin below the
target, where the one-sided form cannot restore it?), stayed green for the right reason: the
crew's 3.8e-3 mol/s keeps the cabin above the target through every perturbation, so the scrubber
is linear there and `Δ(crop flux)/k` is still the offset.

**The slow tests (`cargo test -- --ignored`), 3 red of 12, each handled:**

* `station/tests/chamber_walls.rs`, two four-year measured pins, re-pinned as at every earlier
  adoption: the reference node's mean 170.0109 → 170.0607 K (min / max inside their 0.01 K), the
  chamber's end 295.1355095 → 295.1355085 K; the station-structure battery now runs out on day 1023
  (band 1041–1061 → 1013–1033), 774 543 fast-side rationings, 7 day ends off the band (≤ 18). Earlier
  and warmer, presumably the larger crop's evaporation paid by the heater; not traced.
* `authoring/tests/bed.rs`, the potato example's battery: −0.3 % over 150 days at 400 ppm, −1.7 % at
  the adopted level, against the test's 1 % line. **The user chose to re-size the file, not the
  line** (*"Raise its solar supply"*). Measured with a throwaway probe outside the tree: 150 W
  reproduces the test's −1.674 %; 175 W −0.135 %; 180 W +0.173 %. `scenarios/potato_greenhouse.yaml`
  now reads `solar_power: 175.0`; its 14-day run ends +0.17 %, nothing rationed, re-sown at the same
  step. The wheat example keeps 150 W: its own 300-day slow test stayed green.

Final gate: `cargo clippy --all-targets -- -D warnings` clean; `cargo test` 1453 passed, 0 failed;
`cargo test -- --ignored` 12 of 12 after the above.

**Slice B (next):** `eclss.co2_scrubber` gains `needs_cabin_air: true`; every authored scrubber
declares `cabin_air_mol` (six files: `eclss_cabin`, `eclss_multirate_cabin`,
`eclss_thermal_habitat`, `bioregenerative_station`, `lamp_greenhouse`, `potato_greenhouse`);
`kinetics_param_map` exposes `co2_partial_pressure`; `IMPLICIT_SCRUBBER_AIR_MOL` is deleted; the
authoring manifest is regenerated. Then the crew-size fix (§9, the user's "option 1").

**Follow-up checks after the push (advisor's list).** The swapped `co2_target_lab` was run once:
all six frozen sides reproduce the new goldens' bytes, and its old-form side lands exactly on the
pre-adoption values (cabin CO₂ 3.0 / 3.796 mol, `chamber_inert` 210.134228 kg), with the same moved
counts as §9 (raw: `sources/temp-evidence/co2-level/co2-target-lab-after-adoption.txt`). The
authored files' measured prose was re-read with the throwaway probe (its source and outputs kept in
the same folder: `probe-main.rs.txt`, `potato-solar-150d.txt`, `potato-solar-14d.txt`,
`bioregen-year.txt`): in `bioregenerative_station.yaml` the bloom (5×), feed tank (4.5×), monod
factor (0.250 → 0.600) and regulator (1994.76 mol) read as before, and the O₂ buffer now drains
500 → 136.4 (was 148.2), corrected there. `lamp_greenhouse.yaml` quotes no measured CO₂ or battery
figure. The `air_split` example notes that it now runs at a different level from its test.
