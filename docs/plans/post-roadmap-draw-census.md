# Post-roadmap — the draw census (Step 2, slice 1)

**Opened 2026-09-30**, on the user's call *"step 2's first slice - work on it"*. This is slice 1
of Step 2 in `docs/plans/post-roadmap-review-2026-09-29.md` ("small air volumes against a big
step"). Record: `docs/log/draw-census.md`.

**Scope: measurement only.** Nothing frozen is touched, no option is priced, nothing is decided.
Slice 2 (pricing options A, B and C) and the decision point are later work. The user's standing
wish for option C (*"make plants take less as the CO₂ runs low"*) is noted and is **not** taken
here.

## 1. The question

The Euler backstop steps in when one step would take more out of a store than the store holds.
Every frozen run asserts that it never did (`rationed == 0`). That is a yes/no answer: a run
whose worst step takes 0.28 of a store reads the same as one whose worst step takes 0.76. The
review found three stores close to the edge by separate routes: the sealed jar's CO₂, the lab
leaf form breaking the jar, and the chamber's vapour store.

This slice asks, for every frozen run: **for each store that can be overdrawn, what is the most
any single step takes out of it, as a fraction of what the store holds when the step starts?**

## 2. What is built

* **One lab example, `rust/crates/station/examples/draw_census.rs`.** It sits in `station`
  because that is the lowest crate that can see every frozen run. It writes nothing and decides
  nothing.
* **One probe, shared by all three drivers.** Before each step it evaluates every flow on the
  step's own starting state, exactly as the integrator does, and adds up each store's
  withdrawals (every negative leg, in canonical flow order). The drivers:
  - the biosphere season loop, **with the re-sow applied before the probe** on the perennial
    runs. That is `run_season`'s own order: re-sow, then step. `readouts::step_draws` has no
    re-sow variant, deliberately, because its observer would see the state *before* the re-sow;
  - the single-rate loop (`domains::run`, `station::run_station`);
  - the station's two-rate day, through the lab `TwoRate` driver's observer, probing **both**
    sides: the plant quarter-day steps and the cabin's one-minute steps.
* `readouts::step_draws` (a pinned science gate uses it) and `examples/intraday_exchange.rs` are
  **not** edited.

### The roster: every golden file on disk (21)

| Golden file | Run | Driver |
|---|---|---|
| `season_euler_state.json` | open field, 1 season | season loop |
| `sealed_chamber_state.json` | sealed jar, 3 seasons, no re-sow | season loop |
| `perennial_chamber_state.json` | perennial chamber, 5 seasons | season loop + re-sow |
| `perennial_long_horizon_state.json` | the same, 15 seasons | season loop + re-sow |
| `consumer_chamber_state.json` | consumer chamber, 5 seasons | season loop + re-sow |
| `consumer_long_horizon_state.json` | the same, 15 seasons | season loop + re-sow |
| `drift_summary.json` | **the two 15-season runs above**, folded | (shares those runs) |
| `crew_state.json` | crew alone, 7 days at 1 h | single-rate |
| `eclss_state.json` | life support alone, at 60 s | single-rate |
| `power_state.json` | power alone, 7 days at 1 h | single-rate |
| `power_self_discharge_state.json` | power with self-discharge, 14 days | single-rate |
| `thermal_state.json` | thermal alone, at 1 h | single-rate |
| `cabin_gas_state.json` | crew + life support, at 60 s | single-rate |
| `water_recovery_state.json` | the cabin + water recovery, at 60 s | single-rate |
| `station_state.json` | power → thermal, 7 days | single-rate |
| `sealed_energy_drift_summary.json` | power → thermal, 15 years, folded | single-rate |
| `greenhouse_state.json` | plants ↔ cabin, 7 days | two-rate |
| `lighting_state.json` | power → plants, 7 days | two-rate |
| `harvest_state.json` | plants ↔ cabin + harvest, 7 days | two-rate |
| `sealed_station_state.json` | the whole sealed station, 4 years, re-sow | two-rate + re-sow |
| `state_snapshot.json` | **excluded**: a hand-written file-format fixture, not the output of any run | — |

The 5- and 15-season runs are measured separately, not read off one another.

### The table's columns

Per run, per side, per store: the **worst draw** (withdrawal ÷ amount held), the **step** and
**day** it happened on, the **flow with the largest withdrawal** on that step, the **mean draw**
over the steps that drew at all, and **how many steps took more than 0.1 and more than 0.5**.
The extra columns separate two kinds of store. One is drained by a fixed-rate flow, so it loses
the same fraction every step (mean close to worst). The other is emptied by demand set elsewhere,
like the crop's uptake (a worst far above the mean). Ranking both by the worst figure alone
would mix them.

## 3. Controls — each must hold before any number is read

1. **The end state is the golden's.** Each state-golden run's final state must byte-match the
   committed golden through `domains::goldens::compare`. This is what proves the census loop is
   the reference run and not a look-alike. It matters most for the re-sow order and the
   two-rate observer. For the two summary goldens (`drift_summary`,
   `sealed_energy_drift_summary`), the census run's final state must instead be bit-identical
   to the reference runner's own final state on the same run.
2. **The pinned jar figure is reproduced.** The sealed jar's CO₂ row must read 0.756662 at
   step 777, as `science_gates::margins::the_jars_tightest_co2_step_is_pinned_by_its_headroom`
   pins it. That gate runs the jar's own 3-season horizon, the same run as the golden, so the
   census must agree with it to the last digit.
3. **The probe sums what the step applied, on every step.** For every store, the amount held
   before, plus the probe's own sum of that step's legs (after the backstop's scaling, computed
   with `arbitration::min_scaling`), must equal the amount after, **bit for bit**. That holds
   unless the step fired an extinction, which the census counts and reports. This is the check
   that can fail on a run that never rations, where the next control cannot.
4. **The probe counts the firings the integrator counts.** Per side, over the run. ⚠ On every
   frozen run this reads `0 == 0`. So it is also run on **one case that really rations**: the
   lab leaf form in the sealed jar, which rationed in `docs/log/leaf-rust-remeasure.md`. There
   the count must be non-zero and must match.

## 4. Predictions — written 2026-09-30, before any census code ran

From the review (Step 2, "Predictions to write before slice 1"):

* **P1.** The tightest stores are the chamber CO₂ and the vapour store, in the sealed jar.
* **P2.** The open field has no store under pressure (its air is unlimited).
* **P3.** No nitrogen or soil-water store is within a factor of two of its limit (worst < 0.5).

Added before running, from the rates in the param files:

* **P4. Sealed jar CO₂: 0.757 at step 777** (control 2 restated as a prediction). The 3-season
  worst is in season 1.
* **P5. The fixed-rate stores sit at rate × step.** Condensate (recycling at 0.5/day): 0.125 on
  almost every step, mean ≈ worst. Vapour (condensation at 0.5/day, plus any excess above the
  75 % setting drawn whole): worst between 0.125 and 0.4, mean near 0.125. Cabin CO₂ on the
  cabin side (scrubber at 1e-3/s × 60 s): 0.06, flat. Cabin water on the cabin side (5e-4/s ×
  60 s): 0.03, flat.
* **P6. The perennial and consumer chambers are looser than the jar** on CO₂ (worst < 0.757).
  The jar is the one sized to starve.
* **P7. The station's plant side.** The review's predictions did not mention it, but a
  quarter-day crop step drawing on the small cabin air is exactly "a small store against a big
  step". Cabin CO₂ on the plant side: **≈ 0.16 (0.10–0.25)** on `greenhouse`, `harvest` and
  `sealed_station`. Basis: the crew-loop record's 14.09 m² crop asked for 2.23 times the
  standing air, so the frozen 1 m² crop asks for about 2.23 / 14.09 = 0.158 of it, which is the
  "6.31× headroom" the review quotes. Cabin O₂ on the plant side: < 0.01 (a large pool).
  `lighting`: the lamp and the crop share no stock, so its crop draws on no cabin store at all.
* **P8. The cabin side is not tight anywhere** apart from the fixed-rate figures in P5. Every
  other store's worst draw is < 0.05.
* **P9. Plant tissue is never close.** Leaf, stem, root and grain stores: worst < 0.1 in every
  run. They lose carbon to maintenance respiration at night and to senescence at a few percent a
  day.
* **P10. The standalone sibling domains** (crew, life support, power, thermal): worst < 0.2
  everywhere. Thermal is sized with its time constant far longer than the step, so its worst is
  < 0.05.
* **P11. Controls 1, 3 and 4 hold on the first run.** If one fails, that is a finding, and no
  number is read until it is understood.

## 5. Results

*(Filled in after the run.)*
