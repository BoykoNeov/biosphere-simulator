# Post-roadmap — the biosphere moves to a 1/16-day step, and the crop's CO₂ uptake is taken against the air it leaves (Step 2, slice 4)

**Opened 2026-09-30** on the user's decision for the review's Step 2, slice 4:

> *"1. A finer step everywhere (1/16 day …) 3. Option C only guarantees the plant never takes
> more CO₂ than the air holds … 1 + 3. But ensure the tests runtime would not quadruple"* —
> and then: *"i want the test suite to be roughly the same time as now"*.

Evidence the choice rests on: `docs/log/step-options.md` (the options priced against a 1/256-day
answer), `docs/log/step-cause.md` (the jar's loss is the light's time resolution; a 1/16 step is
the better shape on the open field and on the jar's worst CO₂ error), `docs/log/co2-uptake-source.md`
(C is backward Euler on the crop's flow; no published biology form is missing).

This is an **unfreeze** of the biosphere contract (`docs/biosphere-reference.md` freezes
Euler/`dt = ¼`) and of the station contract's plant step. It follows the discipline in
`docs/biosphere-reference.md` §"The unfreeze discipline". `simcore` is not edited.

## Three commits, in this order, each attributable

| Slice | What | Goldens | Why this order |
|---|---|---|---|
| 0 | Optimise the simulation crates in the dev/test profile | **none** (must be byte-neutral) | buys the runtime the step will spend; proven neutral before anything moves |
| 1 | `BIO_DT` ¼ → 1/16, `STEPS_PER_DAY` 4 → 16 | 11 predicted | every byte of this diff is the step's |
| 2 | C: the crop's uptake read against the end-of-step pool | sealed/chamber goldens predicted | every byte of this diff is C's |

The step and C are not one batch: C's diff must be readable on its own, and the step's price
must be known before C spends more.

## Slice 0 — the runtime budget

**Measured before planning (2026-09-30, this box, test run time excluding compilation):** the
whole suite takes **445 s**; `godot_bridge`'s `cross_boundary` (which drives the real Godot) is
185 s of it. With every crate optimised the suite passes identically (1231 of 1231) in **206 s**,
and everything outside `cross_boundary` goes from 221 s to 66 s.

That first measurement optimised the Godot binding library too, and its child `cargo` builds
inherited the setting through the environment. The committable form is narrower: per-package
`opt-level = 3` for `simcore`, `domains`, `station`, `authoring`, `config` only, written into
`rust/Cargo.toml`, so the child builds `cross_boundary` spawns see it too and the heavy `godot`
dependency stays unoptimised. Measured in two scratch worktrees (fresh build, incremental build
after touching one `domains` file, test run): see §Results.

Why it is byte-neutral, and how that is checked rather than assumed: Rust does no floating-point
contraction or reassociation at any opt level, and the goldens are already regenerated in
`--release` and compared in debug — so both profiles already produce the same bytes. The check is
the full suite green with no golden regenerated, plus `regen_goldens` (report only) reporting no
change. `debug-assertions` and `overflow-checks` stay on (they are separate profile keys).

**The price, stated for the user:** a slower first build and slower incremental rebuilds of the
simulation crates, and a debugger stepping through optimised code in those crates.

## Slice 1 — the step

### The change

`domains::biosphere::{BIO_DT, STEPS_PER_DAY}`: `0.25 / 4` → `0.0625 / 16`. Everything that
counts plant steps derives from these (the 2026-08-14 ceremony converted the days-vs-steps unit
everywhere). 1/16 is a power of two, so `n · dt` stays exact; the cabin's 1440 one-minute steps
per day split into 16 plant steps of 90 exactly (`driver::fast_steps_per_slow_step` refuses an
uneven split).

The hand-typed literals that must change by hand, on purpose: the manifest's `dt_days`
(`domains/src/freeze_manifest.rs`, and the assertion in `domains/tests/manifest_writer.rs`), and
the station `numerics_note` prose ("dt=1/4 day, 4 slow sub-steps per master day, each followed by
its quarter of …"), which nothing checks.

**The step experiments stay at four steps a day; the older lab instruments follow the shipped
step.** `step_options` and `step_cause` (examples and tests) already pass `4` explicitly
everywhere they mean "the shipped run", so they stay re-runnable as dated records with no edit.
The five older instruments (`jar_squeeze`, `jar_control`, `leaf_remeasure`, `intraday_exchange`,
`draw_census`) read `BIO_DT` and `season_steps()`/`steps_for_years` together, so they measure
*the shipped model* consistently at whatever step it has; their printed numbers are dated in
their own log records. They are not pinned. (Corrected before the flip, on an advisor check:
this paragraph first said all lab experiments would be pinned to four.) `draw_census` is the
instrument P6 is read with.

**The sweep, done before the flip (2026-09-30).** Every `BIO_DT`/`STEPS_PER_DAY`/`steps_for*`/
`season_steps` use in `domains`, `station`, `authoring`, `godot_bridge` was classed. None is read
inside a flow or a parameter conversion. Each is a horizon count, a test passing `dt` in, or a
test ceiling already written as `rate · BIO_DT` (so it follows the step). `params.rs`'s
`k · BIO_DT < 1` stability checks loosen by 4×. The Godot scripts (`godot/*.gd`) and the authored
scenarios (`scenarios/*.yaml`, `rust/data/scenarios/`) carry no biosphere step: the Godot
scripts' `24`s are the power domain's hourly step, and the authored scenarios set their own
`dt`. Present-tense prose saying "quarter-day"/"dt = ¼" is reworded; prose that records a
measurement dated at ¼ is left as history. The param YAML comments are **not** reworded: those
files are hashed into the manifest.

### Manifest predictions (P9, written before the flip)

* **Biosphere manifest:** exactly one value moves, `"dt_days": 0.25` → `0.0625`, by hand in the
  writer. The anti-derived literal appears three times in `domains/tests/manifest_writer.rs`
  (lines 51, 216, 229) and all three change with it. No param-file hash moves (no YAML edited);
  the light-path fingerprint (`raw_light_path_samples`, a fixed quarter-day sampling grid of the
  light function, not a step) does not move.
* **Station manifest:** exactly one value moves, `numerics_note`, to: *"Euler everywhere; dt per
  scenario (enforced by goldens, no importable constant). Sealed reference: biosphere-slow dt=1/16
  day, 16 slow sub-steps per master day, each followed by its sixteenth of the everything-fast
  dt=60 s sub-steps (90 of them; interleaved since 2026-09-30, slow-first before); Tier-1 energy
  single-rate dt=3600 s."* Nothing checks this prose.
* **Authoring manifest:** byte-identical. Its only step mention is the key name `dt` in the
  scenario schema; the flow-type registry is read off each type's spec, which carries no step.
  So slice 1 is **not** an authoring unfreeze.
* **`rust/data/tiers.json`:** unchanged unless P7 fires (a measured tier-2 band crossed); a crossing
  goes back to the user, not into this file.

### The sweep before any value is read

A unit sweep over every phrasing, not one grep: "quarter-day", "quarter day", "¼", "1/4",
"0.25", "360 cabin minutes", "six hours", "6 h", "its quarter", "×4", "four plant/slow/steps",
bare `4`/`/ 4` next to a step, in Rust, `.gd`, JSON and the contract docs. Then the flip, then the
whole suite `--no-fail-fast`, and every red is classed before any golden is regenerated:
(a) a step count that should now be a day count — convert to days; (b) a pin that measured the
step (a per-step margin) — convert to days (`margin · dt`); (c) a science claim that moved —
recorded as a finding, never re-tuned; (d) a band or liveness failure — **blocking, back to the
user**.

### Predictions (written before the flip)

* **P1 — structure.** Exactly 11 goldens move: the 10 plant-bearing state goldens
  (`season_euler`, `sealed_chamber`, `perennial_chamber`, `consumer_chamber`,
  `perennial_long_horizon`, `consumer_long_horizon`, `greenhouse`, `harvest`, `lighting`,
  `sealed_station`) and `drift_summary`. Each state golden's `n` is **exactly 4×**
  (1220 → 4880, 3660 → 14640, 6100 → 24400, 18300 → 73200, 28 → 112, 4880 → 19520). No array
  changes length. The 10 others (`cabin_gas`, `crew`, `eclss`, `power`, `power_self_discharge`,
  `station_state`, `thermal`, `water_recovery`, `sealed_energy_drift_summary`,
  `state_snapshot`) are byte-identical.
* **P2 — the open field loses ~10 % of peak leaf area.** The lab measured `open_season`'s peak
  LAI +11 % above the 1/256 answer at ¼ and +0.3 % at 1/16, so 6.02 → about 5.4–5.5. It stays
  inside the 5–8 science band.
* **P3 — ⚠ the mutual-shading loss stops acting in the open field.** Its threshold is LAI 6.0; the
  shipped canopy peaks at 6.02, just over it, since the 2026-08-15 leaf-area citation. At 1/16 the
  canopy should never reach 6.0, so the loss goes inert and
  `the_loss_is_inert_on_peak_lai_and_live_on_peak_w_at_the_frozen_params` goes red on its "live on
  peak W" half. That is a finding (the regime was reached through the step's canopy bias), not a
  test to loosen. The science gate `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided`
  still passes (it accepts "below 6.0").
* **P4 — harvests rise.** Open field ~+2 % (lab: −2.1 % → −0.01 % of the answer). Sealed jar's
  harvest ~+6 % (lab: −7.1 % → −1.4 %) — scoped to the lab's one-season jar; the frozen
  `sealed_chamber` golden is a 3-year run whose state is set by year 1.
* **P5 — no guard fires.** Rationed stays 0 on every golden run; every science band and liveness
  floor holds.
* **P6 — per-step margins roughly quarter.** The jar's tightest single-step draw (0.757) drops to
  about 0.19–0.25; pins written per step go red and are converted to days.
* **P7 — cross-port tier-2 sensitivity** may grow with four times as many steps; whether any
  measured band is crossed is not predicted. A crossing is re-measured under the native-port
  contract, not widened.
* **P8 — cost.** Biosphere-only runs cost ~4×; station runs less than 4× (the cabin's steps do not
  change). With slice 0 the whole suite stays within ±15 % of 445 s.

## Slice 2 — C

Predictions and the solver's cost are written into this file **before** slice 2's code, after
slice 1's numbers are in. Fixed now, from the review of the lab build:

* **Scope: crop only** — backward Euler on the allocation flow's uptake, `X = C₀ − U(X)`. The
  scope that also counts same-step returns can take more than the start-of-step pool, which is
  exactly what the user's promise ("never takes more CO₂ than the air holds") excludes.
* **Inside the allocation flow**, not a wrapper with its own `type_name`: the authoring contract
  freezes the flow-type registry, and a numerics change must not become an authoring unfreeze.
  (To be confirmed against the authoring manifest before code.)
* **One solve per step, a bracketed fast root-finder** (not the lab's bisection to machine
  precision, ~50–60 crop evaluations a step), deterministic, with the lab's checks kept: the
  bracket must bracket and the residual must meet its tolerance, or the step is an error.
* It reaches the station too: the station's cabin air *is* the biosphere's `carbon_pool`.

## Results

### Slice 0 — done 2026-09-30

Two scratch worktrees of `d219872`, default target directories (so `cross_boundary`'s child
builds and Godot's fixed library path are the real ones), measured back to back on this box:

| | as shipped | 5 crates at `opt-level = 3` |
|---|---|---|
| fresh build (`cargo test --no-run`) | 64 s | 87 s |
| rebuild after touching one `domains` file | 8 s | 9 s |
| whole suite, run | **496 s** | **110 s** |
| `cross_boundary` alone, rerun | 191 s | 45 s |
| tests passed | 1231 of 1231 | 1231 of 1231 |

The 496 s is the same suite as the planning measurement's 445 s on a busier box; the ratio is the
number. `cross_boundary` gains the most because its child builds of the simulation crates are
now optimised too, which the first (environment-variable) measurement had done by accident and
this form does on purpose.

Byte-neutrality: every golden the default suite compares passed unregenerated, and the four
ignored tests (`cargo test -- --ignored`, CI's second job) passed too, the sealed station golden
among them: 90 s, the two-rate full horizon 179 s, sealed resume 59 s, the expensive-golden
band 83 s — **411 s** in all, against **1649 s** unoptimised (357, 670, 269, 353), measured
afterwards in the as-shipped worktree with nothing else running. `regen_goldens` was not run for this slice: it runs in `--release`, which the change
does not touch, so it could not have seen it. `clippy -D warnings` clean.

### Slice 1 — done 2026-09-30

The first whole-suite run at 1/16 gave 25 reds (`--no-fail-fast`). Every one was classed before
any golden was regenerated.

**Two decisions went back to the user**, asked together:

* *The lamp scenarios could not run.* `lighting` and `day_neutral_lighting` step their power side
  hourly (24 a day), which does not divide by 16 plant steps, and the driver refused uneven
  splits. The user chose **"Loosen the even-split rule"** over the recommended half-hour power
  step. `driver::day_groups` now runs the day in `gcd(fast, slow)` equal groups: 1440/16 is 16
  groups of 1 plant step + 90 minutes (the adopted interleaved day, operation for operation);
  24/16 is 8 groups of 2 plant steps + 3 power hours. Counts with no common factor (1440/7) are
  still refused, because their only equal grouping is the retired slow-first order. The lamp
  scenario's battery came out **bit-identical** (the lamp and crop share no stock), as the
  advisor predicted before the question was asked.
* *The mutual-shading loss lost its last reach.* The science gate
  `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` asserted that the open field
  itself crosses LAI 6.0, and at 1/16 it peaks at 5.4406. The user chose **"Test it on a pushed
  run"**: the gate now runs the open field with `specific_leaf_area` ×1.1 and asserts the canopy
  enters the regime with the loss off (6.8832) and that the loss holds it visibly lower (6.0797).

**Findings (class c: a measured claim moved; each restated at its new value, the old one kept
in a dated comment, none loosened):**

1. *The mutual-shading loss becomes a ceiling.* With leaves 5–30 % thinner the loss holds peak
   LAI at 6.01–6.19, where without it the canopy reaches 6.2–9.5. On the ladder the LAI
   ceiling's absorption held (×2.058 against ×1.181, factor 1.74; was 1.77), the biomass cap's
   fell (×2.526 against ×1.210, factor 2.1; was 3.3), and the crest now overshoots the cap by
   0.97 % (was 0.13 %). "Nearly unfalsifiable in this direction" was partly the coarse step.
2. *The jar's rationing threshold moved from 75–80 % of its room to 16–17 %*, roughly with the
   step's per-step draw. The season-low CO₂ still reverses as the room shrinks (low point near
   half the room, above the healthy jar's by a quarter), but now in runs with **zero**
   rationing. The quarter-day record attributed the reversal to the backstop; that attribution
   does not survive the finer step. The depressurised jar still never rations (down to 2 %).
3. *The lab leaf form no longer rations the jar* (tightest step 0.3128 of the pool; it was
   1.15). The "jar breaks" reading that holds the leaf form back was taken at a retired step.
   Reopening it is the user's call.
4. *All five compensation-point margins rose*: jar 10.675 → 11.665, perennial 1.150 → 1.190,
   consumer 1.201 → 1.218.
5. *The jar's tightest step* is 0.200940 of the pool (step 3108), at the same moment of the
   season as the quarter-day step's 0.756662 (step 777). Per day it rose, 3.03 → 3.22. That pin
   now holds both the draw and the headroom to 2 %, because 2 % of a 0.80 headroom alone would
   have been an 8 % blind spot on the draw.
6. *The deep-water rescue grew* (leaf ×9.58 → ×11.83, grain ×5.83 → ×7.74) because the
   droughted control does worse at the finer step (peak LAI −31 %, grain −28 %) while the
   rescued crop loses less (−15 %, −4.5 %).

**Predictions graded.**

| # | predicted | measured | |
|---|---|---|---|
| P1 | 11 goldens, `n` exactly 4×, no length change, 9 identical | exactly so; but `lighting` could not run until the driver rule changed | half |
| P2 | open-field peak LAI 5.4–5.5, inside 5–8 | 5.4406 | held |
| P3 | the loss goes inert; the science gate still passes via "below 6.0" | inert on **both** observables; the gate went red on its reach check | half |
| P4 | open-field harvest ~+2 %, jar ~+6 % | +2.15 %; sealed-chamber golden +6.2 % (other chambers +8.3–8.8 %, sealed station +4.9 %) | held |
| P5 | no guard fires, every band and floor holds | rationed 0 on every golden; every science gate green | held |
| P6 | tightest draw 0.19–0.25 | 0.2009 per step; per day it rose | held, with the per-day caveat |
| P7 | tier-2 bands: not predicted | no band crossed (`tier_contract`, `tier_sensitivity` green) | — |
| P8 | whole suite within ±15 % of 445 s | **254 s** (the same suite measured 496 s before slice 0); ignored tests 417 s (411 s at ¼ optimised, 1649 s before slice 0) | refuted, the good way |
| P9 | one value moves in each of the two manifests; authoring and `tiers.json` untouched | `dt_days` and `numerics_note` as written, authoring and `tiers.json` untouched; **missed** the `golden_sha256` rows (7 biosphere, 4 station) that follow every moved golden | half |

Before the flip the advisor also flagged that the plan had no manifest prediction and asked for
the sweep inside flows; both were done and committed first (`70d2b37`).

## Slice 2 — C, predictions and cost (written 2026-09-30, after slice 1's numbers, before code)

### The design, fixed before code

* **A form field, the O₂ form's shape.** `PhotosynthesisParams` gains `co2_read`, which CO₂ the
  crop's growth reads: `StartOfStep` (Euler, the shipped form until now) or `EndOfStep` (C). The
  loader sets `EndOfStep`; `domains::lab` can flip it back so the dated lab records (which ran
  the explicit form) stay re-runnable. Never loaded from a param file. Only `Allocation` acts
  on it; growth and maintenance respiration read the start of the step (the lab's crop-only
  scope). No new `type_name`, so the authoring registry does not move.
* **The equation.** `X = C₀ − U(X)`, where `U(X)` is the allocation flow's draw on the chamber
  pool when the pool variable reads `X`. `U` does not fall as `X` rises (more CO₂, more
  assimilation), so `h(X) = X − C₀ + U(X)` rises and has one root.
* **The solve, one per step, bracketed.** The explicit draw `U(C₀)` is evaluated first (it is
  the step's own evaluation today). If it is 0 — every night step — `X = C₀` and the result is
  the explicit one, bit for bit. Otherwise the root lies in `[max(0, C₀ − U(C₀)), C₀]`: `h ≤ 0`
  at the low end (checked, an error if not) and `h = U(C₀) > 0` at the high end. Illinois
  false position inside that bracket to `|h| ≤ 1e-12·C₀`, at most 100 iterations, else an
  error. **The low end is returned**, where `h ≤ 0`, so `U(X) ≤ C₀ − X ≤ C₀` holds exactly, not
  to rounding: the crop never takes more CO₂ than the start of the step holds.
* **Where it acts.** Only where the scenario is sealed and the pool variable is wired (the jar,
  the chambers, and the station's cabin, which *is* the biosphere carbon pool). The open field
  has no pool: unchanged by construction.

### Predictions

* **Q1 — structure.** Moving: the five chamber goldens (`sealed_chamber`, `perennial_chamber`,
  `consumer_chamber`, and the two long horizons), `drift_summary` (its folds read the
  chambers), and the four station goldens with a crop (`greenhouse`, `harvest`, `lighting`,
  `sealed_station`): **10**. Byte-identical: `season_euler` (open field) and the nine
  plant-free station goldens. `n` and lengths unchanged. Manifests: only those 10 goldens'
  `golden_sha256` rows (6 biosphere, 4 station); no `dt_days`, no param hash, authoring
  untouched.
* **Q2 — small, and in C's direction.** The lab found C and Euler converging as the step shrinks
  (34 ppm apart at ¼, 2.7 at 1/64), both leaving more CO₂ in the air than the fine answer and
  growing less. At 1/16: every chamber's grain moves by **less than 1 %, down**; season-low CO₂
  and the five compensation margins move **up**, by less than 3 %.
* **Q3 — the station's seedlings barely move.** The 7-day goldens (`greenhouse`, `harvest`,
  `lighting`) draw at most 0.078 of the cabin's CO₂ a step (draw census), so they move in
  the 4th significant figure or later; `sealed_station` (4 years) less than 1 %.
* **Q4 — no guard fires, by construction.** Rationing 0 on every golden; the jar's tightest
  step (read by `step_draws`, which evaluates the flows as the step does) falls a little under
  0.2009, to 0.19–0.20.
* **Q5 — cost.** Night steps cost nothing extra. A day step costs the explicit evaluation plus
  2–5 Illinois evaluations. On the jar, fewer than 3 extra allocation evaluations per step on
  average, measured. The whole suite stays under 300 s (254 s after slice 1).
* **Q6 — the lab.** Lab tests that compare against "the shipped run" at four steps a day will
  see C where they expect Euler; where one goes red it is pointed at `StartOfStep`, the form it
  measured, and nothing in its numbers is re-pinned.

### Amendments before code (the advisor's review of the above, 2026-09-30)

* **Design: the solve ends when the bracket cannot shrink, not at a tolerance.** A tolerance stop
  lands anywhere inside the tolerance depending on the path, so a 1-ULP change in an input could
  move `X` by up to `1e-12·C₀`, which the tier-2 sensitivity tests (bands ~1e-11 over thousands
  of steps, and on Linux CI) would read as divergence. So: Illinois steps, a midpoint whenever
  the Illinois point is not strictly inside `(lo, hi)`, stop when no float lies strictly
  between `lo` and `hi`, error at an iteration cap. `|h(lo)| ≤ 1e-12·C₀` stays, as a check after
  the solve. Not a change of prediction.
* **Q4's "by construction", checked rather than assumed.** In a sealed build maintenance
  respiration only *returns* CO₂ to the pool (its covered part is a dropped round trip, its
  shortfall burns organs) and growth respiration is empty; nothing else in the slow step
  withdraws from the carbon pool. So the allocation flow's draw is the only withdrawal the
  backstop sums, and bounding it bounds the step.
* **Q7 — the tests whose subject C removes.** With C in the reference, "the jar rations" cannot
  happen, so these do not move by a number; their behaviour is gone:
  `gas_composition_perturbations.rs` (the 0.17/0.16 threshold bracket and E1 at 0.1) and
  `leaf_form.rs`'s tenth-size-jar control. Each is restated on `StartOfStep` (the explicit
  form's finding stays recorded) and paired with an assertion that the same squeeze rations
  **zero** times under `EndOfStep`. The promise itself becomes a test: the tenth-size jar and the
  2 % jar (205 and 1403 firings explicit) ration 0 times with C. If that fails, it is a finding.
* **Q6 strengthened.** `step_options` and `step_cause` (examples and tests) are pointed at
  `StartOfStep` explicitly whatever their colour, because under `EndOfStep` the lab's C would
  wrap a flow that already solves. Both examples are re-run and one headline number from each
  record must reproduce (the jar's −7.14 % harvest at ¼ against the 1/256-day answer).
* **Q5 graded on evaluations**, not wall-clock alone.

### Slice 2 — done 2026-09-30

Built as designed and amended: `science::Co2Read` on `PhotosynthesisParams` (loader:
`EndOfStep`), `flows::end_of_step_pool` (Illinois inside `[max(0, C₀ − U(C₀)), C₀]`, exact
bracket stop, low end returned), acting only in `Allocation` and only where a chamber pool is
wired. The lab step experiments (`step_options`, `step_cause`, examples and tests) pinned to
`StartOfStep`; both examples re-run and reproduce their records (jar harvest at ¼ against the
1/256-day answer −7.143 %, at ⅛ −2.509 %, at 1/16 −1.420 %; the light-cause cells −7.143 /
−7.185 / −0.023; the lab's C at ¼ −7.465 %).

**Two changes during implementation, before any golden was written:**

* *The low end of the bracket can read a few ULPs above zero*: at `lo = C₀ − U(C₀)` the exact `h`
  is `U(lo) − U(C₀) ≤ 0`, but the computed one carries the rounding of `(C₀ − U) − C₀ + U`
  (seen: 4e-17 to 1e-16). `lo` is nudged down by that excess, at most 8 times and only while it
  is below `1e-12·C₀`; anything larger stays the monotonicity error.
* *The exact stop cost 19.5 evaluations on a daytime step*, no better than the lab's bisection:
  Illinois closes from one side while the far end lags. A one-evaluation probe of the
  neighbouring float, taken once an end is within rounding of the root, closes the bracket.
  Measured on the jar (3 years) and the perennial chamber (1 year): **5.3 and 6.0 evaluations
  per daytime step, 1.35 and 2.84 per step** (night steps cost none; 10,925 of the jar's 14,640
  steps are idle), and C costs 3–5 % of a chamber run's wall time. The season lows were
  unchanged to the printed digits by the probe.

**Findings.**

1. *The promise holds as a test.* The jar at a tenth and a fiftieth of its room rations 205 and
   1403 times explicitly and **0 times under C** (`the_crop_never_takes_more_co2_than_the_air_holds`).
2. *C lifts the jar's CO₂ low point most*, because that is where the explicit step overdrew most:
   season low 7.969 → 9.661 ppm, compensation margin 11.665 → 14.152 (+21 %), tightest step
   0.2009 → 0.1652 of the pool (per day 3.22 → 2.64). The other chambers' margins rose
   +2.9 % and +1.5 %.
3. *Harvests barely move*: chamber grain −0.02 % to −0.09 %, `sealed_station` −0.88 %.
4. *The season-low reversal survives C, with no rationing anywhere*: lowest at 70 % of the
   room (9.501), back above the healthy jar by half the room.
5. *The lab leaf form's tightest jar step* is 0.2324 under C (0.3128 explicit).

**Predictions graded.**

| # | predicted | measured | |
|---|---|---|---|
| Q1 | 10 goldens move, `season_euler` and the 9 plant-free station goldens identical; only their 10 `golden_sha256` rows | exactly so | held |
| Q2 | grain down < 1 %; season lows and margins up < 3 % | grain −0.02 to −0.09 %; perennial +2.9 %, consumer +1.5 %, **jar +21 %** | half |
| Q3 | seedlings in the 4th significant figure; `sealed_station` < 1 % | `greenhouse`, `harvest` yes; **`lighting` in the 3rd** (−0.28 % leaf carbon; its chamber holds 0.23 mol); `sealed_station` −0.88 % | half |
| Q4 | rationing 0; tightest step 0.19–0.20 | rationing 0 everywhere; **0.1652** | half |
| Q5 | 2–5 evaluations per daytime step, < 3 per step, suite < 300 s | first build **19.5**; with the closing probe 5.3–6.0 per daytime step, 1.35–2.84 per step; suite **251 s**, ignored tests 339 s | half |
| Q6/Q7 | the step experiments pinned and reproducing; the rationing tests restated with a C pair | done; **missed** one test whose fixture overdraws (`the_sealed_context_reads_ci_from_the_pool_and_not_the_forcing`, restated on the explicit form with a C bound) and the claim census's four new `A` rows | half |

**Whole-batch cost.** Test suite 496 s → 251 s; the four ignored tests 1649 s → 339 s. The user's
"roughly the same time as now" is met with room to spare.
