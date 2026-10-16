# Stomata that close for the dark part of a partly lit step — a lab-only form first

**Opened 2026-10-16** on the user's *"1"* — the first item of the "what is next" list: the dark
minutes inside partly lit plant steps, which the night-stomata rule counts as day
(`post-roadmap-night-stomata.md` §2, "partly lit steps count as day"; its §6 estimated the bias at
~0.50 GJ of dark-hour latent heat at the booting ratio, not run). **This doc builds a lab-only form and
measures it. It is not the adoption** — adopting it moves station goldens (an unfreeze) and is the
user's call, on the numbers in §5.

## 0. Why

The night rule (ASCE-EWRI's hourly `Rn > 0` is day) is applied per plant step, and a plant step is
1.5 h. A lamp-lit crop's net radiation is the lamp's **top-hat mean** over the step
(`station::lighting::lamp_light_path`), so a step the lamp is on for part of reads a dimmer light all
the way through, counts as lit, and keeps the stomata open for its dark minutes. At the reference's
16-h day (lights 04:00–20:00) the steps 03:00–04:30 and 19:30–21:00 are lit for a third each, so 2 of
the 8 dark hours a warm day keep open stomata. The cold program's 8-h day (08:00–16:00) has two steps
lit for two thirds; the TM 102788 row's 20-h day (02:00–22:00) has two at two thirds, 1 of its 4 dark
hours.

## 1. The form

A partly lit step with lit share φ is φ of lit time and (1 − φ) of dark time. The crop's water is
their time-weighted sum:

`E = φ · PM(Rn / φ, rs_day) + (1 − φ) · PM(0, rs_day / f)`

`Rn` is the step's mean the flow already reads, so `Rn / φ` is the lamp-on value. Two real
Penman–Monteith calls (its `max(0)` clamp makes it linear in Rn only where the result is ≥ 0; no
shortcut is taken). **Not** a resistance averaged by φ: that is a different number with no
time-split basis.

* **φ = 1** → `PM(Rn, rs_day)`; **Rn ≤ 0** → `PM(Rn, rs_day / f)` — today's rule, the same calls, so
  every fully lit and fully dark step is **bit-identical**. Only partly lit steps can move.
* **The `Rn ≤ 0 → dark` guard stays first**: the lamp shed (`station::lamp_shed`) scales the light and
  net radiation by the lamp's power share and not φ, so a fully shed lamp gives φ > 0 with Rn = 0.
* **φ > 0 ⇔ Rn > 0** otherwise; Rn > 0 with φ ≤ 0 is a wiring error, refused.
* **Only the crop's term splits.** Under the two-stage soil evaporation the crop reads
  `Rn · crop_radiation_share`, which the split carries; the soil's evaporation is not stomatal and
  keeps reading the step's mean.
* **Under `NightStomata::Open` the split is not taken** (both halves would read `rs_day`, and the
  identity is exact only up to rounding).

### ⚠ The form's own bias: closure is instant (a side note, 2026-10-16, taken)

The split shuts the pores the minute the lamp goes off and opens them the minute it comes on. The
hours it changes are exactly the ones beside the switch (20:00–21:00 after the lamp goes off,
03:00–04:00 before it comes on), where that is least true: stomata take tens of minutes to close after
dark and often begin opening before dawn, and the night ratio itself ([N1] Table 3) was measured
30 min after sunset, not at the moment of darkness. So the binary rule overstates these hours' water
and the split **understates** it: **the measured saving is an upper bound**, the truth between the two.
The same bias is in the adopted dark-step rule, but its hours are mostly far from the switch. A lagged
form needs a cited closure / opening time constant for wheat, which the shelf does not hold; not built,
and not guessed. Carried into the adoption decision.

### 1a. Where φ comes from

A new forcing, `lit_fraction`, supplied by every lamp-lit build as
`top_hat_window_mean(t, dt, 1.0, photoperiod_s)` — **the same function, at the same photoperiod, that
shapes the light**, so the two cannot disagree. Not re-derived inside the flow from the day length
and the step time: that would be a second copy of the day's shape.

* **The switch:** `TranspirationParams.night_rule: NightRule { NetRadiation, LitShare }`, a form
  selector (never a YAML key). The loader keeps `NetRadiation`. Under `LitShare` the canopy read names
  the `lit_fraction` forcing, and a build that does not supply it **fails at its first step** — no
  silent fallback to the binary rule.
* **Supplied by:** `lighting_bio_resolver` (the lighting golden's build), the sealed station's
  resolver (as cold/warm twins, with `lit_fraction` added to `PLANT_PROGRAM_VARS` so the sowing clock
  picks the phase), the TM 102788 scorecard's forcing closure (its `lit(n, dt)` already is φ). Supplied
  always, read only under `LitShare`: no golden reads it under the loader.
* **Not supplied: the open field and the weather-lit jars.** Their net radiation is a daily mean
  (`weather::net_radiation`, > 0 every day), so neither rule ever fires there. Under `LitShare` they
  would fail loudly. At adoption the loader is shared, so they need either φ = 1 (exact against their
  own daily-mean light) or a per-build choice — **decided at adoption, not here**.
* **The minute water path** (`air_split::water_on_fast_step`, lab-only) reads the plant step's
  recorded temperature and net radiation; under `LitShare` its fast environment has no `lit_fraction`,
  so it fails loudly. Not wired: at the minute step the binary rule is already nearly exact, and the
  path is lab-only. Recorded as an adoption item.

### 1b. The by-hand check

`scorecard_tm102788.rs`'s by-hand water (`w1_by_hand_at` and its sibling) applies the night rule; it
takes the split too, so its model / hand line does not drift on a flip. Its resolver supplies the
lit share as its own `lit(n, dt)`. ⚠ From the TM day its nights cool, the trial's temperature and
humidity are blended by the lit share inside a step too; the split reads the step's one blended
temperature and deficit for both halves (as the model does everywhere), so its two halves are not the
trial's lit and dark air. Stated, not corrected — the step reads one temperature.

## 2. Readers re-read

* `CanopyRead::surface_resistance` → replaced by one crop-potential method both transpiration paths
  call (`Transpiration::split` and `evaluate`).
* `science::night_surface_resistance` keeps its contract (the binary rule) for its other callers.
* The latent-heat books record the flow's moved water per window; they follow without change.
* The probe `examples/chamber_latent_heat.rs` was to gain `--lit-share`. ⚠ *Not built:* the sealed
  station builds from the loader (`build_season(&scenario.bio)`, and its re-sow hook reads
  `params::biosphere()`), so no lab switch reaches it. The switch was measured as night stomata was —
  the loader flipped in a scratch worktree (§5).

## 3. Predictions (written before any code ran)

Run 0 is the probe on today's tree (books adopted); its numbers are recorded in §5 before the switch
is run.

| | prediction | confidence |
|---|---|---|
| L1 | φ ∈ {0, 1} steps bit-identical under the switch (unit test); the frozen loader moves **no golden** (`regen_goldens` report) | high |
| L2 | every lamp-lit resolver: `Rn == φ · Rn_on` and `Rn == 0 ⇔ φ == 0` on every plant step of a day, both phases | high |
| L3 | the warm partly lit steps are φ = 1/3, the cold ones 2/3; 2 per day each | high |
| L4 | under `LitShare`, the sealed station's crop water over 1220 days falls by **~200 kg (−5 to −8 %)**: per warm partly lit step, (1 − φ)·1.5 h = 1 h at the open-minus-closed dark rate (~116 − 20 W) × 1432 steps ≈ 0.50 GJ ≈ 203 kg; somewhat under, since a drier night air raises the deficit | medium |
| L5 | the booked latent heat falls by **0.40–0.50 GJ**; the heater, 1.34 GJ today, falls by most of that (the dark minutes of a partly lit step have no lamp heat): **0.85–1.05 GJ** | medium |
| L6 | the cold phase moves < 1 % of L4 (its crop's dark rate is 0.15 W) | high |
| L7 | TM 102788 water 0.958 → **0.93–0.955** (1 of 4 dark hours, at a 2/3 lit share), carbon unchanged; model / hand stays ≈ 0.997 | medium |
| L8 | with the loader flipped (scratch worktree): **2 goldens move** — `lighting_state` (water only) and `sealed_station_state` (water and the chamber's heat/power, since the books turn water into heat) — the same two night stomata moved | medium-high |
| L9 | carbon unchanged in both, **unless** a re-sow day moves: the reference crop matures ~6 % into its last day (the books plan's standing caution), so a room warmer by ~0.03 K can step grain ~3 %. A ~3 % grain move is read against the re-sow days first | medium |

## 4. Steps

1. The form and its unit tests (L1, the guard, the refusal).
2. The forcing in the three builds; the resolver test (L2, L3).
3. The probe flag; run 0 and the switch (L4–L6).
4. The TM row (L7).
5. The loader flipped in a scratch worktree, `regen_goldens` report (L8, L9).
6. Grade, record, and hand the adoption decision to the user.

## 5. Results (2026-10-16)

**How it was run.** Steps 1–2 in the tree: the form, the forcing, 8 new tests (6 in
`domains/tests/lit_share_stomata.rs`, 2 in `station/tests/lit_share_forcing.rs`), all green. Steps 3–5
in a scratch worktree of `2af27e8` plus this work, with two scratch-only edits: the loader set to
`LitShare`, and `weather_forcings` supplying `lit_fraction = 1` (the weather's light is a daily mean, so
each of its steps is wholly lit — exact, and one of the two adoption options in §1a). Run 0 is the same
probe on the tree as committed. The cited outputs are kept at
`W:\Claude_projects\space-station\sources\temp-evidence\lit-share-build\` (git-ignored, local).

### 5a. The sealed station, 1220 days (`chamber_latent_heat`; controls clean in both: rationed 0, legs re-applied 0 mismatches; the books' window identity ≤ 5.5e-15 relative)

| | run 0 (binary) | lit share | change |
|---|---|---|---|
| water evaporated (kg) | 2835.91 | 2594.09 | **−241.8 (−8.5 %)** |
| of it the crop (kg) | 2509.2 | 2309.0 | **−200.2 (−8.0 %)** |
| of it the soil (kg) | 326.7 | 285.1 | −41.6 (−12.7 %) |
| booked latent heat (GJ) | 6.948 | 6.356 | −0.592 |
| heater (GJ) | 1.340 | 0.872 | −0.468 |
| cooler (GJ) | 2.586 | 2.710 | +0.124 |
| battery, end (J) | 9.831e9 | 1.0299e10 | +4.8 % |
| warm lit windows: crop / soil W | 125.45 / 14.85 | 114.90 / 13.57 | |
| cold lit windows: crop / soil W | 1.20 / 6.80 | 1.08 / 4.23 | |
| crop dark / lit rate, warm | 0.157 | 0.171 | |
| the room, warm lit, mean T − setpoint (K) | +0.0079 | +0.0126 | |
| re-sow days | 140 … 1120 | 140 … 1120 | none |
| grain at maturity (mol C, first) | 15.0653 | 15.0685 | +0.02 % |

**The soil's 42 kg was not predicted.** The soil's evaporation is not stomatal and reads the step's mean
either way, so the split cannot move it directly. Most likely route, not traced: a crop drinking less
trips the watering trigger (FAO-56 depletion, p = 0.55) less often, so the surface is dry more often
(Stage II). The end snapshot shows the dry clock running (`soil_dry_days` 0 → 6.25, the top layer 26.5
→ 0 kg) — one day's state, not a run total.

### 5b. TM 102788 (the scored row and the by-hand check, flipped)

W1 **0.9582 → 0.9283** (3.996 → 3.871 L m⁻² d⁻¹); W2 0.875 → 0.849; U1, C1, B1 identical to every
printed digit; model / hand 0.9967 → 0.9962.

### 5c. The goldens (`regen_goldens`, flipped): 20 run, **2 change**, 18 byte-identical

* `lighting_state` — water only: two water stocks and `top_soil_water`.
* `sealed_station_state` — water (soil −45 %, condensate ×6.9, subsoil −27 %: the end of a watering
  cycle, not run totals; the run total is `window_evaporation.total_kg` −8.5 %), energy (battery
  +4.8 %, space −1.1 %, the node −0.5 %), and **carbon and nitrogen at 1–2e-4** (leaf −2.0e-4, root
  −2.0e-4, humus −1.3e-4) through `thermal_time` +1.9e-4: the room's warmth, not the water.
* Every open-field and weather-lit golden byte-identical under `lit_fraction = 1` — the φ = 1 identity
  at golden scale.

### 5d. Graded

| | | |
|---|---|---|
| L1 | φ ∈ {0, 1} bit-identical; the frozen loader moves no golden | **HIT** (unit tests; §5c's 18; the committed tree's own suite) |
| L2 | `Rn == φ·Rn_on`, `Rn == 0 ⇔ φ == 0` | **HIT** (both builds, both phases) |
| L3 | warm φ = 1/3, cold 2/3, two each | **HIT** |
| L4 | crop water ~200 kg (−5 to −8 %), somewhat under | **HIT on the amount** (−200.2 kg, −8.0 %, at the edge); "somewhat under" did not show |
| L5 | latent −0.40 to −0.50 GJ; heater 0.85–1.05 GJ | **latent MISSED** (−0.59: the crop's own −0.49 is inside; the soil's unpredicted 0.10 is not); **heater HIT** (0.872) |
| L6 | cold < 1 % of L4 | **HIT** (~0.8 kg) |
| L7 | TM water 0.93–0.955; carbon unchanged; model / hand ≈ 0.997 | **water MISSED low** (0.928): the TM canopy is dense (LAI ~6.2), so an open dark hour loses ~5× a closed one — re-derived after the run, not re-fitted; carbon and model / hand **HIT** |
| L8 | the same 2 goldens | **HIT** |
| L9 | carbon unchanged unless a re-sow day moves | **PARTLY MISSED**: no re-sow day moved, but carbon moved at 1–2e-4 through the room's temperature — the latent-heat books carry a water change into the room's heat, and the room's heat into the crop's clock |

### 5d′. A test-runtime regression, caught after the commit (advisor review, 2026-10-16)

`ff84e7f`'s by-hand TM helper read the loader (`params::biosphere()`, which parses every param file)
on **every step**; the bisections call the by-hand sums hundreds of times, so `scorecard_tm102788`
went from 0.66–2.52 s to **922 s** in the test profile (1891 s in release). Fixed in the follow-up: the
rule is read once per test binary (`loader_night_rule`, a `OnceLock`); measured 1.87–2.00 s, inside
the old file's spread on the same machine. No value changed.

### 5e. What it says, and what is left for the decision

The binary rule left about **a quarter of the dark time with open stomata** (§0), and closing it takes
another **8 % off the sealed crop's water** and **0.47 GJ off the heater** over the run — a bit under half
the night-stomata adoption's step (the station's chamber water −18.5 % there, over three times the
dark hours; measured from different baselines, so not compared per hour). Carbon is untouched except
through the room's warmth.

⚠ **An upper bound** (§1, the side note): closure here is instant, at the hours nearest the switch.
The real figure lies between run 0 and this.

**Owed to an adoption, if taken:** (1) the open field and weather-lit builds need φ = 1 supplied (as the
scratch build did) or a per-build choice; (2) the minute water path (lab) fails under the switch — *measured* in the scratch
worktree: `minute_transpiration.rs`'s one-day run stops at its first fast step with *"a plant flow on the
fast step reads "lit_fraction", which is not in its recorded window"* — and would need the lit share
recorded or an explicit refusal; (3) the unfreeze ceremony for the 2 station goldens and
their manifest rows; (4) the soil's 42 kg traced before it is described as the watering trigger's.

## 6. Adoption (opened 2026-10-16)

**The user's answers** (2026-10-16): **adopt**; the minute water path **records the lit share** (as
recommended); and for the open field and the weather-lit builds **a real day/night share**, against the
recommendation of φ = 1 — then, asked how to take it, **stage it** (recommended). So:

* **This adoption** takes the split for every lamp-lit build. The weather's builds get **φ = 1 for now**:
  exactly what they do today (their net radiation is a daily mean, > 0 every step, so the binary rule
  never fired there either), recorded as a **temporary step toward the user's choice, not a decision
  against it**.
* **The next item** is *"the open field gets a night"*, lab-only first. It is not a forcing line: the
  built rule refuses `Rn > 0` with φ = 0 (§1), so a real night needs the weather's net radiation given a
  within-day shape — and since its temperature and deficit are daily means too, whether they need a
  daily cycle as well is that item's first scoping question (a night with daytime air may be a
  half-built form). It is a **biosphere** unfreeze (likely all 7 biosphere goldens and every open-field
  station golden), so it is priced in the lab before any decision. Starting source: ASCE-EWRI's hourly
  method.

**⚠ §1a was wrong about the minute path** (advisor, 2026-10-16). It said the binary rule is "already
nearly exact" there. It is not: `water_on_fast_step` reads `WATER_LOSS_WINDOW = [TEMP_VAR, RN_VAR]`,
the plant step's **recorded** window, so every minute of a 1.5-h step reads the step's mean net
radiation and carries the reference's partly lit bias unchanged.

### 6a. What changes

* **The loader line** (`params.rs`): `night_rule: NightRule::LitShare`. No YAML key, flow, aux or param
  set changes.
* **The weather** (`system::weather_forcings`, a frozen biosphere file): supplies `lit_fraction = 1`, a
  constant, with the staging stated beside it. No biosphere manifest key reads the forcing set (its
  `forcing` block hashes the light path and the weather fixture), so the manifest cannot move; the
  biosphere goldens are the pin.
* **The minute path** (`air_split`): `NetRadiationRecorder` also records `window_key(LIT_FRACTION_VAR)`
  (the same additive pattern; it is the only writer of that key), and `WATER_LOSS_WINDOW` gains
  `LIT_FRACTION_VAR`, so each minute splits as its plant step does.
* **Every build that reaches the flow** supplies φ: the lamp builds (`lighting_bio_resolver`, the sealed
  station's twins, the TM scorecard) already; everything else builds on `weather_forcings` (the
  greenhouse, the open fields, the weather-lit jars, and through them the authoring and Godot
  sessions). The lamp-failure perturbation zeroes the net radiation and not φ: the `Rn ≤ 0` guard
  makes that step dark.
* **Tests** — restated, none deleted. `the_loader_keeps_the_binary_rule` → pins `LitShare`.
  `a_build_without_the_lit_share_fails_under_the_switch` → the weather now supplies φ, so it takes the
  forcing **out** of the resolver to keep its claim (no silent fallback), and a new assertion pins the
  open field under the adopted loader bit-identical to the binary rule over its first days. The reach
  test's baseline was named `NetRadiation` in its own commit first (`eaa8f16`). `minute_transpiration.rs`:
  the recorder test checks the lit-share key too, and its raw-flow window resolver serves φ.
* **Wording**: the "lab-only" comments on `NightRule`, `LIT_FRACTION_VAR`, the lamp forcing, the flow
  field, `lab::with_night_rule` and the loader say what the loader now sets.
* **Goldens and manifests**: `lighting_state`, `sealed_station_state`; their two station-manifest rows.
* **The probe** (`examples/chamber_latent_heat.rs`) gains a watering / drying-stage count, for the
  soil's 42 kg (§6c).

### 6b. Predictions (written before the loader line changed)

| | prediction |
|---|---|
| B1 | `regen_goldens` (report): exactly `lighting_state` and `sealed_station_state` differ; the other 18 identical, the 7 biosphere goldens and `sealed_energy_drift_summary` among them — the scratch worktree's §5c, which made the same two edits |
| B2 | `lighting_state` moves in water keys only; `sealed_station_state` in water, energy, and carbon / nitrogen at 1–2e-4 through `thermal_time` (§5c) — **not** "water only" |
| B3 | the probe reproduces §5a's flipped column to its printed digits (evaporated 2594.09 kg, heater 0.872 GJ, latent 6.356 GJ) |
| B4 | TM 102788: W1 **0.9283**, W2 0.849, model / hand 0.9962; U1, C1, B1 unchanged; `scorecard_tm102788` still ~2 s |
| B5 | manifests: station REWRITTEN in exactly the two `golden_sha256` rows; biosphere and authoring `unchanged` |
| B6 | before regeneration the reds are: the two restated tests' old forms (gone once restated), `golden_regression` and `tier_contract` (both on `lighting_state`), and in `--ignored` the slow test(s) reading `sealed_station_state`. `manifest_writer` stays green until the goldens are rewritten (it hashes the committed files), then goes red until the manifest is. After: full suite, `--ignored`, clippy all green |
| B7 | the minute path runs under the adopted loader; its tests green after the stated restatement; no golden reads it |
| B8 | no science gate moves: no biosphere band or floor reads a lamp-lit build, and the weather builds are bit-identical |

### 6c. The soil's 42 kg (traced before any golden moves)

Hypothesis (§5a, not yet shown): the crop drinks less, so the root zone reaches the FAO-56 trigger less
often, the sealed station's watering (`EventRecycling`, condensate to soil) fires fewer times, and the
surface spends more plant steps drying (Stage II) than wet (Stage I). **Prediction:** fewer trigger
events and less water delivered under the split (roughly the −11.6 % fall in recycled water, §5a's legs
2689 → 2377 kg); more drying steps; and most of the −41.6 kg in the **wet-surface** stage's total (fewer
wet steps), not a lower drying-stage rate. If instead the events are unchanged and the fall is in the
drying stage, the hypothesis is wrong and is recorded as such.

**Traced 2026-10-16** (the probe's new watering / drying-stage count, run on the tree before the
loader line changed and after it; the after run reproduces §5a's flipped probe line for line — B3):

| | binary | split | change |
|---|---|---|---|
| watering events (trigger steps whose recycling moved water) | 37 | 33 | −4 |
| water into the root zone (kg) | 2689.18 | 2377.31 | −11.6 % |
| plant steps with a wet surface / drying | 11 840 / 7 680 | 8 434 / 11 086 | wet −29 % |
| soil evaporation, wet surface (kg) | 244.43 | 214.37 | **−30.1** |
| soil evaporation, drying (kg) | 82.12 | 70.80 | −11.3 |

**The hypothesis holds**, with one correction. The crop drinks less, the root zone reaches the trigger 4
times fewer, the surface is wet for 29 % fewer steps, and **three quarters of the fall (30 of 41 kg) is
the wet stage's**. The other quarter is the drying stage's, and there the prediction's "not a lower
drying-stage rate" was wrong: the drying steps are 44 % more, but the spells are longer, so more of them
sit deep in Stage II, where the book's day factor `√(d+1) − √d` (Eqn 14.19, `science.rs`) falls with the
dry days — per drying step 0.0107 → 0.0064 kg. (These stage sums total 41.4 kg against §5a's 41.6: the stage split is read from
the soil-evaporation day counter at each plant step, §5a's from the end-of-run legs.)

### 6d. Graded (2026-10-16)

* **B1 held.** `regen_goldens`: exactly `lighting_state` and `sealed_station_state` CHANGED; 18 identical,
  the 7 biosphere goldens and `sealed_energy_drift_summary` among them.
* **B2 held.** `lighting_state`: two water stocks and `top_soil_water`, nothing else. `sealed_station_state`:
  soil water −44.6 %, condensate ×6.90, subsoil −26.9 %, battery +4.76 %, space −1.07 %, node −0.49 %;
  leaf −2.0e-4, root −1.9e-4, humus C −1.2e-4, humus N −1.9e-4; `thermal_time` +1.9e-4 — §5c's figures.
* **B3 held.** The probe on the adopted tree is the scratch worktree's flipped run line for line (the new
  watering line aside).
* **B4 held.** W1 **0.9283** (3.8712), W2 0.8493, model / hand 0.9962; U1, C1, B1 unchanged;
  `scorecard_tm102788` 2.35 s (3.6 s with `--nocapture`).
* **B5 held.** Station REWRITTEN in the two `golden_sha256` rows; biosphere and authoring `unchanged`.
* **B6 MISSED.** Before regeneration 5 reds, not the 2 named (`golden_regression`, `tier_contract`, both
  hit). Unpredicted: (1) `scorecard_tm102788::every_weather_forcing_is_the_chambers` — the weather's new
  `lit_fraction` was not in the trial chamber's replaced list (the chamber already supplied its own; it is
  now listed — the test did its job); (2) `chamber_walls` L4 (node 188.7373 K, heater 139.788 W) and its
  day-end band: the room is **held again** at day 120's end, 0.07642 K below the setpoint inside one
  step's loss (was 0.9526 K) — restated; (3) `chamber_heat`'s dead-cooler run: its collapsing leaf now
  reaches the extinction snap on step 93 (day 6), residual −4.0e-205 mol — restated so the working
  cooler's run still asserts no events and the dead one's may lose only crop carbon. In `--ignored`, 2
  reds, both re-pinned to measured values: the structure's battery runs out by **day 1051** (was 1025),
  8 day ends off the band (was 410); the reference node **162.4715 / 175.3180 / 170.0109 K**, the room
  ends **295.1355095 K**. The two restated tests' old forms were gone before the run (restated first).
  After: suite **1398 passed / 0 failed** (`--release --no-fail-fast`), `--ignored` **9 / 0**, clippy clean.
* **B7 held.** `minute_transpiration.rs` 5 / 5 with the lit share recorded and served.
* **B8 held.** No science gate went red.

**Lessons.**
* **A heat-carrying change moves every lab room that heat reaches.** Four of the five unpredicted reds
  are the room's heat (walls, node, the dead cooler's crop), reached through the latent-heat books — the
  same reach §5d's L9 found for carbon. Predicting "the same 2 goldens" was right; predicting the reds
  from the goldens alone was not: the lab pins read rooms no golden records.
* **A list-of-replaced-keys test catches a forcing added upstream.** The TM chamber's own lit share had
  been *inserted after* the replaced list, so the new weather key went unlisted; the test named it.
