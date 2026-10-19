# The station's crew size — one crew member, defined once

**Opened 2026-10-20** on the user's *"yes"* to taking the crew-size fix, the item they chose on
2026-10-19 as the one after the cabin CO₂ level (`post-roadmap-co2-level.md` §9, *"1300, option 1"*).
**This doc is the plan and its predictions. Nothing is built and nothing frozen has moved.**

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
(3.796e-3 / 2e-3 = 1.898 mol). Read with `W:\temp\claude\crew-size\peek.py`, kept until the item
closes.

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

The crew flows are forced: nothing stops a store running dry except its starting size.

1. **One crew member. Recommended.** The smallest change, and the one the water already describes.
   Every store stays fed: the sealed station eats 31 703 mol of its 500 000. The crop is still
   about 14× too small to feed this one person — see §7.
2. **Four** (the ISS's usual complement, and `science_gates::bvad::N_CREW`). Food 126 812 mol of
   500 000, fine. Water through the crew is 21 080 kg over 1220 days against a 20 000 kg store; the
   sealed build recycles urine back into that store (it ends today at 19 473), so it should hold,
   but that is a measurement owed, not a fact.
3. **Keep 13.3 and raise the water to match.** 70 000 kg through the crew; the water store and
   possibly recovery need re-sizing. It keeps a number nobody chose.

## 4. The form — one crew member, from BVAD, with no second copy of a loaded value

* **The food rate is BVAD's food-carbon total, not CO₂ ÷ `f_resp`.** `f_resp` is a loaded param
  (`crew.yaml`); typing `24.654 / 0.949 / 86400` into a `const` would make a second, unwatched copy
  of it ("two copies, one stale"). The crew eats carbon = CO₂ carbon + fecal carbon = 24.654 +
  1.332 = **25.986 mol C/CM-d**, 3.007657e-4 mol/s, which is the same balance `crew.yaml` derives
  `f_resp` from. Breathed out, `0.949 × 25.986` = 24.661 mol/d, +0.028 % over BVAD's 24.654: the
  rounding of `f_resp` to three figures. ⚠ **This departs from the science gate on purpose.** The
  gate (`science_gates.rs`, `bvad::cabin_scenario`) calibrates to the CO₂ load, so its O₂ headline
  is BVAD's number exactly; a scenario *eats*, and the split is the param's job.
* **The BVAD constants move out of `#[cfg(test)] mod bvad`** into the scenario's own module (or a
  shared one both read), so the gate and the scenario share one copy. A crew count `N` multiplies
  them. Whether `CabinScenario` carries `N` or only the derived rates is a build detail.
* **Water re-derived from BVAD too: 4.366 kg/d = 5.053241e-5 kg/s, +1.065 %.** Recommended, so "one
  crew member" is defined in one place for both quantities. It moves every water stock the crew
  touches by about 1 %. If the user prefers, it can stay 5.0e-5 bit-identical and be its own
  item; the predictions below give both.

## 5. ⚠ The crew-only cabins start with no CO₂ — at one person the scrubber is barely tested

`cabin_gas` and `water_recovery` start `cabin_co2_0 = 0.0` and run 900 one-minute steps. Today the
crew reaches the 12.1885 mol target in about 54 steps. **At one crew member it takes 42 703 s, step
712 of 900**, so the scrubber works only in the last fifth of the run. The goldens would not
notice. Proposed: **start both cabins at the target** (as their O₂ and H₂O, and every crop-sharing
cabin's CO₂, already start at theirs). That is its own move and is predicted separately below.

## 6. Predictions, written before any measurement (one crew member)

Arithmetic: `W:\temp\claude\crew-size\predict.py`. "Offset" = the crew's breathed-out CO₂
(2.854267e-4 mol/s) over the scrubber's rate 1e-2, or over the O₂ makeup's 2e-3.

**`cabin_gas`, `water_recovery` (900 × 60 s):**

| Stock | Today | Predicted, start 0 | Start at target |
|---|---|---|---|
| `crew.food_store` | 784.000 | **983.7586** | same |
| `boundary.fecal_waste` | 11.016 | **0.82831** | same |
| `eclss.cabin_co2` | 12.568102 | **12.2170** | 12.2170 |
| `boundary.co2_removed` | 192.416 | **≈ 3.196** | ≈ 15.384 |
| `eclss.cabin_o2` | 1993.102 | **≈ 1994.857** | same |
| `boundary.o2_supply` | −203.086 | **≈ −15.27** | same |
| `crew.water_store` (`cabin_gas`) | 17.300 | 17.300 unchanged / **17.27125** with BVAD water | same |

**`greenhouse`, `harvest` (7 days):** food store 1580.800 → **3818.097**; fecal 123.379 → **9.277**;
the cabin (`biosphere.carbon_pool`) 12.568105 → **≈ 12.2170** (1323 → 1286 ppm). The seedlings
move by under 1 %, all smaller (lower CO₂).

**`sealed_station` (1220 days):** food store 78 368.000 → **468 296.884**; fecal 21 503.232 →
**1616.859**; the cabin ≈ 1286 ppm. **The crop: grain peak −0.2 % to −1.4 %, most likely about
−0.5 %.** That is the local slope of `post-roadmap-co2-level.md` §5's curve at 1300 ppm, which
lies between its 400→1284 slope and its 1284→3951 one. Transpiration moves down with it, by
less. The crew's breathing (2.85e-4 mol/s) stays about 10× the crop's peak lit uptake, so **the
cabin never falls below the target** and the one-sided scrubber never idles. The lab run will
count the steps where it does.

**Every other golden (15 of 20) byte-identical**: the standalone crew, ECLSS and biosphere
goldens do not read the station's scenario. The authored bed (`authoring/src/bed.rs`) starts from
`sealed_station_scenario()` but runs its own authored crew; whether it reads the station's crew
rates is to be confirmed by the lab run, not assumed.

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
   rates are scenario data, so the example passes its own `CabinScenario`. It measures one and
   four crew members, BVAD water on and off, and the start at the target, against §6.
2. **The user's adoption choice**, on the measured numbers.
3. **Adoption** as a station unfreeze: save the lab's switched states as bytes, regenerate, `cmp`.
