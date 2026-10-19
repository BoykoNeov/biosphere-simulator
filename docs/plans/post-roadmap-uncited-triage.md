# Uncited-value triage — Step 5 of the 2026-09-29 review

**Opened and built 2026-10-19** on the user's *"1, 2 and 3 — do them"* (the triage table was item 3).
A measurement, not a change: no value, golden, manifest or `source:` string moved. Two lab examples
were added; they write nothing.

The review asked for *"each of the 60 markers, which readout it moves most, and by how much for a
±10 % change … 'the five that matter'"*. This file is that table, with the recorded range of each
value beside its reading, because ±10 % ranks **sensitivity**, not **error**.

## 1. The roster — 45 values, and why not 60

**60 was a count of the text `TODO(cite)`, not of values** (60 at `b4ed41f` across 23 files). It
counts a file's header line, a value's `source:` and a comment as three, and it misses every value
marked `DESIGN`, which is uncited by declaration. The rule used here: **a frozen value whose own
`source:` names no primary source for its number.** Read off every param file on 2026-10-19:

| Class | Count | Values |
|---|---|---|
| `TODO(cite)` open | 23 | photosynthesis ×8 (vcmax, jmax, quantum_yield, theta, t_min, t_opt_lo, t_opt_hi, t_max); respiration ×5 (maintenance_coef, q10, t_ref, growth_efficiency, o2_half_saturation); phenology t_base, t_cap; transpiration aerodynamic_resistance, surface_resistance; nitrogen max_uptake_capacity; herbivory o2_half_saturation; senescence rdr_leaf, rdr_stem, rdr_root; allocation partition_table |
| `DESIGN` (a choice, not a literature value) | 18 | herbivory grazing, respiration, mortality; water_cycle recycling_rate; eclss co2_scrub_rate, condense_rate, o2_makeup_gain; self_discharge_rate; radiator area, heat_capacity; harvest_rate; water_recovery rate, efficiency; chamber heat_capacity, cooling_capacity, response_time, wall_area, heater_capacity |
| `CALIBRATED` (inside a cited range, chosen to close) | 2 | decomposition_rate, microbial_respiration_rate |
| `OURS` | 1 | stem_reserves trigger_dvs |
| A book value stated bare | 1 | stem_reserves remobilization_rate |

Excluded: the potato overrides (no frozen run grows potatoes) and values with a primary source whose
page was read. **40 of the 45 were measured; the chamber's five `DESIGN` values were not** (§5).

## 2. Method

```
cargo run --release -q -p domains --example uncited_triage                  # 30 values + 4 shares, ~40 min
cargo run --release -q -p domains --example uncited_triage -- --recorded    # 7 rows at their ranges
cargo run --release -q -p station --example uncited_switch                  # 9 rows, ~25 min
```

(from `rust/`, at below-normal priority; a value name after `--` runs that row alone.)

* **Plant side** (`domains/examples/uncited_triage.rs`): each value through the lab value switch
  (`lab::biosphere_with`) at ×0.9 and ×1.1, or **±1 °C** for a temperature (10 % of `t_base = 0` is
  no step). The partition table runs as its four organ shares, each scaled at every knot with the other
  three compensated. Read: every gated readout of the switch's report (`lab::report::SPECS`, 15-year
  rows included), and the four largest final-stock moves over the four frozen season runs. A control
  refuses a row whose substitution at the typed frozen value does not rebuild the frozen params bit
  for bit.
* **Station side** (`station/examples/uncited_switch.rs`): the equipment structs are handed to their
  builders as values, so a copy with one field scaled is the substitution — the goldens' own builders
  and runners. Each row runs on a build that **reads** it: eclss and water recovery on the sealed
  station; `harvest_rate` on the harvest golden's build; the radiator on the 15-year heat-closure run,
  read as the station's one science gate reads it; self-discharge on its power run.
* **At the recorded size.** Where a value's own `source:` records a range or error wider than ±10 %,
  it runs again at that size (`RECORDED` in the plant tool, the `recorded` column in the station tool).
* **"Margin used"** is printed only for the three readouts with a plain-number bound: peak LAI
  (5 < x < 8), peak W (< 14.4248 t/ha), the converged peak-leaf fixed point (> 0.55). The CO₂
  season-low rows have no plain bound (their floor reads "n/a" under the live-O₂ form), so they
  carry a move only. Over 100 % means the bound is crossed.

⚠ **Nearly every large "margin used" is one gate: peak W, with 3.9 % headroom** (13.888379 against
14.4248). That gate is a **tripwire for the nitrogen-stressed branch**, set to fire before the
earliest measured bite (15.068 t/ha) — crossing it means the open field reached the branch a guard
watches, not that the yield is wrong. Anything that raises yield 4 % trips it.

⚠ **The final-stock column is evidence of reach, never of size.** It prints only the top four moves,
and the water buffers (condensate, soil and subsoil water) crowd carbon out — they swing −94 % to
+75 % on almost every row, because a small change shifts when the watering trigger fires. A "+0.000 %"
there after the water rows is truncation, not a measurement. The re-sowing chambers also end at an
arbitrary point in their crop cycle, so a chamber's final leaf carbon at +280 % is where the run
stopped. An all-zero top four is still evidence that nothing moved.

## 3. Results — plant side (±10 %, or ±1 °C)

Ranked by the largest move in a gated readout. "Range" is what the value's own `source:` records.

| # | Value | Largest gated move | Margin used (peak W unless named) | Range or known error |
|---|---|---|---|---|
| 1 | partition: leaf share fl | **+10.2 %** fixed point (×1.1) | 22 % of the fixed point's (×0.9) | fitted; backfill refused, leaf direction exhausted |
| 2 | growth_efficiency 0.75 | **−9.2 %** peak W | **230 % — CROSSES at ×1.1** | 0.7–0.85 |
| 3 | quantum_yield 0.3 | **−8.8 %** peak W | **216 % — CROSSES at ×1.1** | "~0.3", none |
| 4 | vcmax 100 | **+8.3 %** jar's CO₂ low | 0 | 80–120 |
| 5 | q10 2 | +7.0 % jar's CO₂ low | 47 % | "≈ 2" |
| 6 | maintenance_coef 0.02 | +6.1 % jar's CO₂ low | 63 % | 0.015–0.03 |
| 7 | partition: root share fr | −5.0 % fixed point | 69 % | as fl |
| 8 | t_opt_lo 15 °C | −5.0 % jar's CO₂ low | 37 % | none |
| 9 | t_base 0 °C | −4.8 % peak W | **103 % — CROSSES at −1 °C** | none |
| 10 | t_ref 25 °C | +4.3 % jar's CO₂ low | 42 % | "~25" |
| 11 | partition: stem share fs | −3.0 % peak W | 72 % | as fl |
| 12 | partition: grain share fo | +2.9 % peak W | 76 % | as fl |
| 13 | theta 0.7 | +2.9 % peak W | 74 % | none |
| 14 | jmax 180 | −2.4 % peak W | 52 % | 1.5–2 × Vcmax |
| 15 | decomposition_rate 0.011 | +2.1 % jar's CO₂ low | 4 % of the fixed point's | at the cited range's fast edge, by design |
| 16 | rdr_leaf 0.02 | −1.8 % peak W | 44 % | ~1.7× its cited table |
| 17 | microbial_respiration_rate | −0.9 % fixed point | 2 % | calibrated |
| 18 | rdr_stem 0.005 | +0.5 % peak W | 14 % | the cited source says stems lose none |
| 19 | t_min 0 °C | −0.4 % jar's CO₂ low | 8 % | none |
| 20 | herbivory grazing_rate | −0.4 % consumer CO₂ low | — | consumer biomass ±7 % |
| 21 | remobilization_rate 0.1 | −0.3 % peak W | 6 % | a book value |
| 22 | rdr_root 0.01 | −0.15 % peak W | 4 % | within ~10 % of its table |
| 23 | trigger_dvs 1.0 | −0.10 % peak W | 2 % | ours |
| 24 | herbivory respiration_rate | −0.06 % | — | consumer biomass ±8–9 % |
| 25 | respiration o2_half_saturation | +0.01 % | 0 | none |
| 26 | herbivory mortality_rate | +0.01 % | — | consumer biomass ±5 % |

**No gated readout moved — two different reasons:**

* **Nothing reads it in any frozen build.** `surface_resistance` (70): the frozen leaf-area form
  (`SzeiczLong`) ignores the constant, which only the `Constant` form reads. `recycling_rate` (0.5):
  the frozen watering trigger (`EventRecycling`) never reads it. **Citing either would assert
  nothing.** Removing either is an unfreeze and is not proposed here.
* **Read, but not reached by today's frozen runs.** `t_opt_hi`, `t_max`, `t_cap` (no frozen run is
  that warm); `aerodynamic_resistance` (open field only, where only soil water moves: −3.4 % at
  ×0.9); `max_uptake_capacity` (uptake never hits the cap); herbivory `o2_half_saturation` (consumer
  biomass ±0.004 %). A warmer scenario would wake the first three — dated to the 2026-10-19 roster.

## 4. Results — at each value's recorded range

| Value | Probe | What it moved |
|---|---|---|
| maintenance_coef | 0.015 / 0.03 | peak W +6.3 % (**CROSSES, 162 %**) / −10.8 %; jar's CO₂ low −15 % / **+32 %** |
| growth_efficiency | 0.7 / 0.85 | peak W −6.1 % / +11.8 % (**CROSSES, 305 %**), peak LAI +3.3 % |
| rdr_leaf | 0.012 (its cited table's late value) | peak W +5.8 % (**CROSSES, 150 %**) |
| vcmax | 80 / 120 | jar's CO₂ low **+18.8 % / −12.1 %**; nothing near a bound |
| jmax | 150 / 200 | peak W −4.3 % / +2.2 % (57 %) |
| aerodynamic_resistance | 30 / 100 | open-field soil water −18 % / +20 %; no gated readout |
| max_uptake_capacity | ÷6 (0.00025) | **nothing, to every printed digit** — the ~6× error is inert in every frozen run |

## 5. Results — station side

| Value | ±10 % | At its recorded error |
|---|---|---|
| **co2_scrub_rate** 1e-3 | no gate reads it; chamber CO₂ run-minimum −9 % (×1.1); water buffers swing | **÷8 (ISS CDRA's cycle): the chamber's final CO₂ +700 %, roots +136 %; ÷30: CO₂ +2894 %, roots +154 %.** No rationing |
| recovery_rate 1e-3 | only its own buffer, inversely (−9 / +11 %) | ÷100: the crew's water −0.02 %; ÷1000: −0.23 % (the buffer holds the rest) |
| harvest_rate 1e-5 (harvest build) | the grain store −8 / +10 % | ÷70: grain store +432 %; the crew's food −0.001 % over the golden's 7 days |
| recovery_efficiency 0.9 | the crew's water ±2.4 % | — |
| radiator_area 10 | the station gate's quantity ∓2.4–2.7 %; **6–7 % of its margin** | — |
| condense_rate | cabin vapour ±0.4 % | — |
| o2_makeup_gain | ±0.01 % | — |
| self_discharge_rate | battery ±0.12 % | — |
| radiator heat_capacity | the gate quantity ±0.003 % (the stored joules scale with it, by definition) | — |

**Not measured: the chamber's five `DESIGN` values** (heat_capacity, cooling_capacity, response_time,
wall_area, heater_capacity). `sealed.rs` and `plant_chamber.rs` read `params::chamber()` inside the
build, so there is no value to hand in. **Offered, not built:** pass the chamber params into the
builder as the eclss and recovery structs already are — a refactor of the reference with no value
change, after which `uncited_switch` gains five rows.

⚠ The sealed station builds with the harvester off (`with_harvest = false`, its golden's scope), so
the tool's first run measured `harvest_rate` there and read 0.000 % everywhere. The harvest golden
is the build that reads it; the row now runs there.

## 6. The shortlist — the five worth chasing

1. **The scrubber rate (`co2_scrub_rate`).** The largest known error with the largest measured
   consequence: at the real equipment's cadence the chamber ends with 8–30× the CO₂ and roughly
   double the plant tissue. ⚠ No citation can fix it: the form is wrong (real CO₂ removal is a
   cyclic bed swing, not a first-order drain), so chasing it is a **form and sizing decision**, not a
   page to find. BVAD is on the shelf (`sources/`) and carries equipment figures.
2. **Maintenance respiration (`maintenance_coef`).** Its own recorded range alone crosses the yield
   tripwire and moves the jar's CO₂ low by −15 % to +32 %, the largest plant-side swing measured.
3. **Growth efficiency (`growth_efficiency`).** +10 % crosses the tripwire; the top of its recorded
   range crosses it 3×.
4. **Quantum yield (`quantum_yield`).** +10 % crosses the tripwire, and no range is recorded at all.
5. **Leaf death rate (`rdr_leaf`).** The value its own cited table gives crosses the tripwire. This
   one is already the heaviest finding in its file; the triage confirms it is not idle.

**Next in line:** `t_base` (−1 °C crosses), `vcmax` (the largest CO₂ move over its range, no bound
near), `q10`. The partition table ranks first at ±10 % but its direction is closed on record (backfill
refused; leaf direction exhausted) — it is listed, not proposed.

**Safe to stop chasing for the frozen runs:** the two values nothing reads; the hot-end temperatures;
`max_uptake_capacity` (inert even at its recorded 6× error); water recovery (even at ÷1000 the crew
loses 0.23 % of its water); the harvest rate (grain buffer only over 7 days); the O₂ half-saturations;
condense_rate, o2_makeup_gain, self_discharge_rate.

## 7. Found while measuring

* **The fixed-point gate's `source` string quotes a stale reading.** It says the 15-year value is
  0.578137 and clears 0.55 by 5.1 %; the leaf-shedding unfreeze (2026-10-06) moved it to 1.029832
  (`post-roadmap-leaf-shedding.md`), clearing by 87 %. The string is a gate source: a correction is a
  ceremony, not a clean-up.
* **Two uncited values are read by no frozen build** (§3). A citation search on either is wasted.
* **The Bernacchi check is unchanged**: still owed, still needs the PDF placed in `sources/` by hand.
  None of its three constants is in this roster (they are cited, to the literature's reproduction).

## 8. Checked

Both examples are lint-clean under `cargo clippy -- -D warnings`; every run above completed with no
rationing and no failure. Results in §9.

## 9. Results (2026-10-19, at below-normal priority, from `rust/`)

* `cargo test --workspace --no-fail-fast`: **1447 passed, 0 failed, 12 ignored** over 103 test binaries.
* `cargo test --release --workspace --no-fail-fast -- --ignored`: **12 passed, 0 failed** — the twelve
  slow tests the run above skips.
* `cargo clippy --workspace --all-targets -- -D warnings`: clean.

No golden, manifest or param file changed, so the byte and manifest gates read the committed tree as before.
