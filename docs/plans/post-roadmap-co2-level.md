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
the air is known, exactly as `o2_setpoint_mol` and `humidity_setpoint_kg` already are. A
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
