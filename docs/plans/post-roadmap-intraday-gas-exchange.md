# Post-roadmap — plants and cabin trade gas within the day

**Opened 2026-09-29**, on the user's call *"start implementing"* the 2026-09-29 review proposal.
This is that proposal's first step. Record: `docs/log/intraday-gas-exchange.md`. The crew-loop
diagnosis this step answers: `docs/plans/post-roadmap-crew-coupled-loop.md` §9.

**Scope of this session: the lab slices only (1, 2, 3 and 5).** Nothing frozen is touched. The
adopt-or-not decision (slice 4) is the user's, and this document stops in front of it.

## 1. The defect

The station's two-rate driver runs a whole master day of plant steps, then a whole day of
cabin steps (`rust/crates/station/src/driver.rs`, `advance_one_master_day`):

```
today:     P P P P  c c c ... c        (4 plant quarter-days, then 1440 one-minute cabin steps)
proposed:  P c..c   P c..c   P c..c   P c..c     (360 cabin steps after each plant step)
```

So all four of the crop's quarter-day steps draw on the cabin CO₂ as it stood at dawn. The crew's
exhalation for that day arrives only afterwards. The scrubber holds the cabin pool at an amount
(about 3.8 mol C) that is a small fraction of what the crew exhales in a day, so a crop sized to
the crew is rationed by the schedule, not by the air.

## 2. What is built

* **A second day order, lab-only, in `driver.rs`.** One function runs a master day in either
  order: slow-first (today's) or interleaved. It reports rationing split by side (plant and
  cabin), because the reference driver sums the two into one number and the crew-loop record
  showed that sum cannot say which side rationed. An observer sees each step's before and after
  state, tagged plant, cabin or re-sow, so the lab can keep books per side.
* **The reference function is not edited.** No runner, session or bridge calls the new one.
* **Refusals.** The interleaved order requires the cabin's steps per day to divide evenly by the
  plant's; it refuses otherwise. Both orders keep the reference's two day-length guards.
* **The lab example `station/examples/intraday_exchange.rs`** does the measuring (slices 2, 3, 5).

## 3. Controls — each must hold before any measurement is read

1. **The slow-first path is the reference, bit for bit.** Same final state (hex-float snapshot),
   same rationing total, same events, on the greenhouse (7 days) and on the sealed station across a
   re-sow. Otherwise the lab is measuring its own copy.
2. **No plants, no change.** Interleaved with an empty plant registry must match the reference
   bit for bit. This measures the step-counter risk (the cabin now sees the counter move four
   times a day) instead of assuming it away. Also checked by reading: no cabin-side flow reads the
   step count, and every cabin-side forcing in the four plant-bearing scenarios is a constant.
3. **The interleaved order is not a no-op.** On the greenhouse it must differ from the reference.
4. **The area-scaling transform, rebuilt in Rust.** Scaling the ground area and every extensive
   starting amount (air included) by A on the standalone sealed chamber must reproduce A times the
   unscaled run, every stock at every step, to round-off. The crew-loop record measured this in
   Python (1.7e-14). If it fails here, that is a finding and slice 3 stops until it is understood.
   For the station runs the **air is not scaled** (the crew-loop record, finding 8(c)): the air
   fields are `chamber_air_capacity_mol`, `chamber_co2_mol0`, `chamber_o2_mol0`, `water_vapor0`
   and `condensate0`.
5. **The rationing probe names what the integrator counted.** For each plant step, the lab
   re-evaluates the plant flows and computes the backstop's own scale factors. The probe's count
   must equal the driver's plant-side count, step by step.
6. **The carbon books close.** Over a run, the cabin CO₂ pool's change must equal the plant-side
   change plus the cabin-side change plus any re-sow change, to round-off. The crew's exhalation,
   derived as the cabin-side change plus what the scrubber removed, must match the respired
   fraction times the food eaten.

## 4. Measurements

* **Slice 2 — the frozen roster.** The four scenarios that run on the two-rate driver and carry a
  plant: `greenhouse`, `lighting`, `harvest` (7 days each) and `sealed_station` (4 years). For
  each, every stock at the end of the run, reference order against interleaved.
* **Slice 3 — the crew-sized crops.** The sealed station at **14.09 m²** (one crew member's crop
  by BVAD) and **187.45 m²** (the station's whole crew). Sized as the crew-loop record did it:
  ground area and every extensive plant and soil amount × A, not the air; lamp × A, solar × A and
  battery × A; one season; harvest on. Both drivers, **re-measured today** — the record's
  numbers came from deleted Python, before the quarter-day step and the humidity setting.
  Reported per driver: plant-side and cabin-side rationing, the stock that forces each plant-side
  firing, the worst single-step draw per stock, peak leaf area, and the season's crop carbon.
* **Slice 5 — the closure readout.** Per run: what the crew exhaled, what the scrubber removed,
  what the plant side drew from the air net, and what harvest returned to the food store. The
  closure share is the plant side's net draw over the crew's exhalation.

## 5. Predictions — written 2026-09-29, before any code ran

**Which frozen results can move if interleaving is adopted:**

* **Correction to the review proposal.** It lists five goldens that would move. Two cannot:
  * `lighting` — the lamp and the crop share no stock; the only link is the lamp schedule, a
    forcing. Interleaving must leave it **bit-identical**.
  * `sealed_energy_drift` — it has no plant and runs on the single-rate driver, not this one.
* **Sixteen of the twenty goldens never touch the two-rate driver** and cannot move: the seven
  biosphere ones, the five sibling-domain ones (`crew`, `eclss`, `power`,
  `power_self_discharge`, `thermal`), and `cabin_gas`, `water_recovery`, `station`,
  `sealed_energy_drift`. Only `greenhouse`, `lighting`, `harvest` and `sealed_station` do.
* So adoption would move **three** goldens: `greenhouse`, `harvest`, `sealed_station`.

**How much they move:**

* `greenhouse` and `harvest` are 7-day runs of a seedling. The plant draws little, so plant stocks
  move by less than 1 part in 1,000. The cabin pool ends each day back near the scrubber's level
  in both orders, so cabin stocks move by less than that.
* `sealed_station` (1 m², 4 years): **the crop gets more carbon, not less.** Today its lit
  quarter-days after the first see a pool the earlier ones have drawn down; interleaved, each sees
  a refilled pool. Predicted size: season crop carbon up by **0.3 % to 5 %**. Cabin-side stocks
  (battery, heat, water) move by less than 1 part in 10,000.

**The crew-sized crops (slice 3):**

* The lamp is on 16 hours centred on midday, so the four quarter-days get 2, 6, 6 and 2 hours of
  light. The two middle ones carry 6/16 of the day's photosynthesis each.
* Scaling the crew-loop record's per-area daily **net** gain (its demand figure), the heaviest
  quarter-day asks for about **0.84×** the refilled pool at 14.09 m² and **11×** at 187.45 m².
  Net gain is a lower bound on what the crop draws, so both are floors.
* The scrubber's time constant is about 17 minutes against a 6-hour window, so under
  interleaving each plant step sees the pool fully re-settled.
* **14.09 m²:** reference order rations on most lit days. Interleaved, rationing **falls by at
  least 90 %**. Whether it reaches zero is not predicted — 0.84× is a floor, not an estimate.
* **187.45 m²:** reference order rations; interleaved still rations on most lit days, because
  11× cannot be closed by a factor-of-four refill. The stock forcing it is the cabin CO₂ pool.
* **Cabin-side rationing is 0** in every run, both orders (the power is sized with the lamp).
* **Closure share:** 1 m², well under 1 % in both orders. At 187.45 m², interleaving raises it by
  a factor between 2 and 4 — the crop gets up to four draws a day instead of one.

**The step-counter risk:** control 2 passes, and so does the no-plant run.

## 6. Grading — measured 2026-09-29

Command: `cargo run --release -q -p station --example intraday_exchange` (sections `roster`,
`sealed`, `similarity`, `crew`; none runs all). Output kept at `W:\temp\claude\intraday\`.

**Tree state after the build:** `cargo test --workspace --no-fail-fast`, Windows: **1219 passed,
0 failed, 4 ignored** (1212 + the 7 new controls), exit 0; clippy `-D warnings` clean;
`regen_goldens` report: **20 of 20 run, 0 would change.**

### 6.1 Controls — all held

| control | result |
|---|---|
| 1. slow-first is the reference | bit-identical on the greenhouse, the harvest ring, a 3-day sealed run with a re-sow (tests), **and the full 4-year sealed station with its real re-sow** (the lab) |
| 2. no plants, no change | bit-identical; no cabin-side flow reads the step count (grepped) |
| 3. the order is not a no-op | the greenhouse differs |
| 4. area scaling is exact | worst stock deviation from A× **1.04e-14** (A = 14.09) and **1.32e-14** (A = 187.45) over 3,660 steps; aux identical to the bit |
| 5. the probe counts what the backstop counts | equal in every run |
| 6. the carbon books close | pool budget residual ≤ 8.2e-11 mol; crew exhalation matches the respired fraction × food eaten to ≤ 4e-12 |

**The controls themselves were mutation-checked** (`tests/day_order.rs`): five mutations — the
interleaved order run slow-first, the slow-first order run cabin-first, the uneven-split refusal
disabled, the re-sow not reported to the observer, one fast step too many per plant step — each
turned its intended test red. ⚠ My first attempt at the refusal mutation was malformed (operator
precedence left the check live), so its green meant nothing; it was redone and went red.

### 6.2 Predictions

| prediction | measured | grade |
|---|---|---|
| `lighting` bit-identical | 25 of 25 stocks bit-identical | **held** |
| `sealed_energy_drift` and 15 other goldens cannot move | by construction (they do not run on this driver) | **held** |
| `greenhouse` plant stocks move < 1e-3 | largest 4.7e-4 (stem reserve); cabin stocks 1.6e-8 | **held** |
| `harvest` plant stocks move < 1e-3 | grain store **+37.5 %**, humus **+19 %**, microbes **+12 %** | **FALSIFIED** |
| `sealed_station` crop C up 0.3–5 % | **+1.23 %**, up | **held** |
| `sealed_station` cabin-side stocks < 1e-4 | all below 2.3e-5 | **held** |
| 14.09 m²: heaviest step asks ≥ 0.84× the refilled pool | 1.10× | **held** (a floor) |
| 187.45 m²: heaviest step asks ≥ 11× | **7.26×** | **FALSIFIED** — see finding 3 |
| 14.09 m²: rationing falls ≥ 90 % | **rises**, 22 → 72 | **FALSIFIED** — see finding 1 |
| 187.45 m²: still rations, on the CO₂ pool | 943 firings, every one on `carbon_pool` | **held** |
| cabin-side rationing 0 everywhere | 0 in all 10 runs | **held** |
| 1 m² closure share well under 1 % | 0.030 % (4 y), 0.089 % (1 season) | **held** |
| 187.45 m² closure share ×2 to ×4 | 0.613 % → 2.313 %, **×3.77** | **held** |
| step-counter risk empty | controls 2 and 3 | **held** |

### 6.3 Findings

**1. The schedule starves the crop through CONCENTRATION, not through rationing — so the
rationing count went UP as the starvation eased.** Measured, not argued: the lab records the
cabin CO₂ each plant step starts from, by quarter of the day.

| run | quarter 0 | quarter 1 | quarter 2 | quarter 3 | peak LAI | peak crop C per m² |
|---|---|---|---|---|---|---|
| 1 m², slow-first | 3.7960 | 3.7828 | 3.6471 | 3.5151 | 5.467 | 33.60 |
| 1 m², interleaved | 3.7960 | 3.7960 | 3.7960 | 3.7960 | 5.571 | 34.18 |
| 14.09 m², slow-first | 3.7960 | 3.4799 | **1.8946** | **1.4654** | 3.119 | 19.13 |
| 14.09 m², interleaved | 3.7960 | 3.7960 | 3.7960 | 3.7960 | **5.498** | **33.78** |
| 187.45 m², slow-first | 3.7960 | 2.5465 | 1.1613 | 1.1640 | 0.439 | 2.50 |
| 187.45 m², interleaved | 3.7960 | 3.7960 | 3.7960 | 3.7960 | 1.096 | 6.20 |

(One season, harvest on, CO₂ in mol.) Under slow-first the crop's afternoon steps see the pool
at half and two-fifths of the scrubber's level; photosynthesis follows the concentration down,
so demand falls with supply and the backstop rarely fires (22 times). Interleaved, every step
sees the pool re-settled at **exactly 3.7960** — the scrubber closes a quarter-day gap
completely — so the crop asks for more, and when it asks for more than the pool holds the
backstop fires (72 times). **A rationing count cannot measure this starvation**: the crew-loop
record showed `rationed == 0` cannot measure closure; here the count moved in the opposite
direction to the harm.

**2. At one crew member's area, interleaving removes the schedule's starvation almost entirely.**
Per square metre the 14.09 m² crop reaches **98.8 %** of the 1 m² crop's peak carbon and
**98.7 %** of its leaf area (slow-first: 57 %). Closure share doubles, 0.64 % → 1.26 %.

**3. At the whole crew's area, it helps and does not solve.** The crop reaches **18 %** of the
1 m² crop's peak carbon per m² (slow-first: 7.4 %); closure share ×3.77. The 11× prediction
failed because its premise did: it scaled an **unstarved** crop's demand, and this crop is
starved to a fifth, so it asks for less (7.26×). Its heaviest step still asks for **7.26×** the
pool it starts from, and that pool is the scrubber's equilibrium **amount** (3.796 mol), which
does not depend on the room's volume. **Which lever would relieve it is NOT measured**: no run
here varied the scrubber rate, the air or the step. ⚠ Not "a bigger room": the crew-loop record
(finding 8(c)) measured that more air at the same scrubbed amount is a LOWER concentration and a
worse crop. The plan's own risk note applies — a finer interleave by hand is not the next move;
this is the squeeze the review's Step 2 measures.

**4. The harvest ring moves because the two sides share two more stocks than the air.**
Isolated by switching each seam off (7 days, relative change, slow-first → interleaved):

| harvest flow | feces → litter | grain store | microbes | humus |
|---|---|---|---|---|
| off | off | +3.4e-4 | +3.1e-5 | +2.4e-5 |
| **on** | off | **+0.402** | +3.1e-5 | +2.4e-5 |
| off | **on** | −0.011 | **+0.121** | **+0.191** |
| on | on | +0.375 | +0.121 | +0.191 |

Each seam carries one half and the soil half is unchanged by the harvest flow. Both are
cabin-side flows that write a plant-side stock (harvest drains `storage_c`; the crew's feces
land in `litter_carbon`), so the order decides what the plant sees there too. My prediction
reasoned only about the air. ⚠ **Why** each moves this much in 7 days is not isolated: the
harvest scenario starts past anthesis with a 119-mol litter pile, and no run varied either.

**5. Two checks added on the advisor's review, both clean.**

* **The three goldens adoption would move keep their own gates under interleaving.** Each golden
  producer asserts zero rationing and zero events. Measured interleaved: `greenhouse` 0 / 0,
  `harvest` 0 / 0, `sealed_station` (4 y) 0 / 0. Adoption is a regeneration, not a gate change.
* **Oxygen is not the route.** The cabin O₂ each plant step starts from differs between the
  orders by at most **0.13 %** (1993.1 vs 1995.7 mol at 187.45 m²), against CO₂ differing by up
  to **3.3×**. The starvation runs through CO₂.

**6. The crew-loop record's numbers are not today's.** At 187.45 m² slow-first, today's tree
gives 466 plant-side firings and peak LAI 0.439; the record (deleted Python, a one-day step)
gave 282 and 0.2772. Recorded side by side, not joined: the step, the humidity setting and the
oxygen form have all changed since.

## 7. The decision this stops in front of

**Adopt interleaving into the reference, or keep it lab-only?** Adoption is a station unfreeze:
the ceremony, the reference function replaced (not kept as a switch), the session switched in the
same commit, and the moved goldens regenerated.

**What adoption would move, measured:** three goldens. `greenhouse` by at most 4.7e-4;
`sealed_station` by +1.2 % in crop carbon; `harvest` by **+37.5 %** in its grain store and
+19 % in its humus (finding 4). Seventeen cannot move, `lighting` included.

**Recommendation: adopt.** The review's condition was "if the large crop stops being starved by
the schedule". Measured, the schedule's share of the starvation is removed at one crew member's
area (98.8 % of the 1 m² crop, which starts every step at the scrubber's level) and more than
halved at the whole crew's; what remains at 187 m² is a draw of 7.26× the scrubbed pool, and
which lever relieves it is not measured (finding 3). The interleaved order is also the one that matches
the physics — the crew breathes out while the plants take CO₂ in — and it costs no new number.
The one thing to weigh is the `harvest` golden's large move, which is the same effect on two
stocks the old order hid.

## 8. ADOPTED by the user, 2026-09-30 — the unfreeze

**The user's call, 2026-09-30:** *"1. adopt"*. A station unfreeze under
`docs/station-reference.md` §"The unfreeze discipline". Advisor-reviewed before any code.

### 8.1 The design, as reviewed

* `advance_one_master_day` takes the interleaved body: one plant step, then
  `steps_per_day / slow_steps_per_day` cabin steps, repeated. The runners and the session call
  that one function, so the session and the Godot bridge follow with no edit of their own.
* **An uneven split is refused** where the two day-length guards already live (`run_master_day`
  and `SimSession::two_rate`), because the per-day function does not validate and an integer
  division would silently drop cabin time. Every caller today passes 1440 or 24 cabin steps
  against 4 plant steps. **Authored scenarios cannot reach this driver**: nothing in
  `rust/crates/authoring` calls it, and the bridge reaches it only through the fixed palette.
* **§7's "delete the lab section" is NOT followed, on review.** Deleting `TwoRate` would orphan
  `examples/intraday_exchange.rs` — the only thing that reproduces findings 1–4 — and the
  controls comparing the orders. §7's objection was to a *switch in the reference*; a lab-only
  arm that re-runs the retired order for the record is not that. `DayOrder::SlowFirst` becomes
  "the retired order, lab-only", and the bit-identity control now says **interleaved** equals
  the reference.

### 8.2 Predictions — written 2026-09-30, before any code changed

From the lab's own runs on the unedited tree (`intraday_exchange -- roster` and `-- sealed`):

1. **Structure.** Of the 21 golden files, exactly **3** change: `greenhouse_state.json`,
   `harvest_state.json`, `sealed_station_state.json`. The other 18 are byte-identical —
   `lighting_state.json` and `sealed_energy_drift_summary.json` included.
2. **Values — exact, not approximate.** Each new golden is the lab's interleaved final state bit
   for bit, because the new reference day performs the lab's interleaved arithmetic operation for
   operation. Recognisable figures:
   * `greenhouse`: 16 stocks move, the largest `stem_reserve_c` +4.691e-4; crop C
     0.215763 → 0.215797.
   * `harvest`: 17 stocks move; `storage_c` +37.53 %, `humus_carbon` +19.05 %,
     `microbial_carbon` +12.12 %; crop C 0.187205 → 0.187629.
   * `sealed_station` (4 y): 17 stocks move; crop C 60.811979 → 61.557289 (+1.2256 %).
3. **Gates.** All three keep 0 rationing and 0 events (measured interleaved, finding 5); the
   sealed station's one science claim (the thermal node does not collapse) holds; session
   parity and the bridge's save/load round trip hold, since they call the same function.
4. **Manifest.** Exactly three `golden_sha256` rows move, plus `numerics_note`, whose prose is
   edited by hand to name the order (it is the manifest's record of the station's numerics and
   today does not say the order at all). No param, flow, aux or claim row moves. The biosphere
   and authoring manifests do not move.
5. **Tier-2 band basis.** The greenhouse ±1-ULP sensitivity is re-measured by
   `station/tests/tier_sensitivity.rs` on the new order; predicted to stay within 10× of the
   recorded 2.8e-16 and below the 1e-11 band. The band's written argument ("regulators hold the
   pools at setpoint between the once-daily biosphere lumps") becomes truer, not weaker: the
   lumps are now quarter-day ones with the regulator between each.
6. **No simcore byte moves** (`git diff rust/crates/simcore/` empty).

A row that moves unpredicted, or a gate that reddens, is a finding — not a diff to accept.

### 8.3 Grading — measured 2026-09-30

| # | Prediction | Result |
|---|---|---|
| 1 | exactly 3 of the goldens change | **held**: `regen_goldens` reported 20 run, 3 changed (`greenhouse`, `harvest`, `sealed_station`); 17 byte-identical, `lighting` and `sealed_energy_drift_summary` included |
| 2 | each new golden is the lab's interleaved end state, bit for bit | **held to every printed digit**: greenhouse 16 stocks moved, `stem_reserve_c` +4.6912e-4; harvest 17, `storage_c` +37.532 %, `humus_carbon` +19.049 %, `microbial_carbon` +12.123 %; sealed station 17, `stem_reserve_c` +1.5002 %, `storage_c` +1.3349 %. No non-stock field moved. The three bit-identity controls in `tests/day_order.rs` (greenhouse, harvest ring, a re-sow) now hold the *interleaved* lab arm to the reference and pass |
| 3 | gates hold | **held**: full `cargo test --release --no-fail-fast`, 74 test binaries, 0 failed; session parity and the bridge's save/load pass unedited; the ignored long runs are graded in §8.4 |
| 4 | manifest: 3 `golden_sha256` rows + `numerics_note` | **held**: exactly those 4 lines; biosphere and authoring manifests unmoved |
| 5 | greenhouse ±1-ULP sensitivity within 10× of 2.8e-16, below 1e-11 | **held**: 3.490512e-16 (was 2.762079e-16, ratio 1.26); recorded in `rust/data/tiers.json` `measured_2026_09_30_interleaved_day` |
| 6 | no `simcore` byte moves | **held**: `git diff rust/crates/simcore/` empty |

**The refusal.** `fast_steps_per_slow_step` is the one place the split is checked; the per-day
function calls it (so no path can drop cabin time), and `run_master_day` and
`SimSession::two_rate` call it up front. A new test drives all three with 1440 cabin steps over
7 plant steps and reads the refusal from each. No shipped scenario is refused; authored files
cannot reach this driver.

**Prose that named the old order, all corrected:** the driver's module doc, the session's
docs (one also still called `n` a day count — a leftover of the one-day step), `greenhouse.rs`,
`palette.rs`, the bridge's `time_control.rs`, `godot/greenhouse_smoke.gd`, `ulp_probe.rs`, the
native-port doc and `tiers.json`'s sealed-station evidence ("once-daily biosphere lumps": they
are quarter-day lumps with the regulators between each now, which only strengthens that
argument), and `docs/station-reference.md`'s numerics paragraph.

### 8.4 The long runs — measured 2026-09-30

`cargo test --release -- --ignored`, 4 of 4 pass: the sealed-station golden is still the
reference's output (88 s, 1.3 M sub-steps); a session stepped over the full sealed horizon
matches `run_master_day` bit for bit; a resume across a sealed season boundary is
bit-identical; every expensive station golden sits inside its measured band. The lab example's
full-length control now reads *interleaved == reference bit for bit: true* over the 4-year
sealed station with its real re-sow.
