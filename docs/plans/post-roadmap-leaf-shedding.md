# Tissue shedding that reads no temperature — a design note (2026-10-06)

**Status: OPENED 2026-10-06 as a design note, no code.** The user's call after Step 3c slice 3b
(`post-roadmap-room-temperature.md` §24m, "DECIDED 2026-10-06": *"Keep 100, study shedding"*).
Any change to `Senescence` is a **biosphere unfreeze** (the frozen science, `docs/biosphere-
reference.md`): it moves every biosphere golden and the station's. Nothing is built until the
user picks.

## 1. Why this is open

Slice 3b dimmed the cold weeks' lamp, and the seedling lost half its carbon over the 56 cold days
(0.160 → 0.077 mol C). Booked flow by flow (a temporary example replaying the reference step order;
the booked legs close on the observed change exactly):

| 56 cold days, mol C | 3b, dim lamp | 3a's world, full lamp |
|---|---|---|
| growth into the organs (`Allocation`) | +0.0089 | +0.4691 |
| maintenance respiration | −0.0185 | −0.0165 |
| shed, **leaf** | −0.0338 | −0.1153 |
| shed, **root** | −0.0327 | −0.0636 |
| shed, **stem** | −0.0069 | −0.0100 |
| change | −0.0829 | +0.2636 |

So the cold seedling is not starving — growth and upkeep nearly cancel (−0.0096) — it is
**shedding**: `Senescence` removes a fixed fraction of each organ per day (`rdr_leaf` 0.02,
`rdr_root` 0.01, `rdr_stem` 0.005, all `TODO(cite)`), reading no temperature and no development
stage. At 4 °C tissue dies as fast as at 22 °C. ⚠ First reported to the user as "leaves"; roots shed
nearly as much (corrected the same day).

## 2. What the shelf says (searched 2026-10-06 under several headings — "senesc", "leaf death",
"death rate", "life span", "longevity", "turnover", "freez" — across every PDF in `sources/`)

| Source (on the shelf) | Locus | Before flowering | After flowering / seed growth |
|---|---|---|---|
| Penning de Vries et al. 1989, §3.2.6 p. 95 + Listing 5 p. 212 (already `senescence.yaml` [A]) | rice IR36 crop file; potential production | leaf and root death **0** below DS 1.0; "except for their reserves, stems do not lose weight" | DS-keyed rates (leaf 0.007 → 0.012 /day, root 0.011 → 0.010) |
| Soltani & Sinclair 2012, Box 9.1 + Eqn 9.7, pp. 105–109 (already cited [C] for phenology) | generic grain crops, **non-limiting N** (Ch. 9's stated assumption) | **"No leaf senescence"** from emergence to beginning seed growth | LAI falls linearly to 0 at maturity, paced in **temperature units** (DTU) |
| Teh, *Introduction to Mathematical Modeling of Crop Growth*, Eqn 7.17 pp. 155–156, after Goudriaan & van Laar 1994 (SUCROS) | generic; Teh's model | leaf death from age **0** before flowering | `DVR / max(0.1, 2 − DVS)` — paced by the **development rate** (temperature-driven) |
| Teh, p. 158 (line "no root turnover or root death") | Teh's model | no root death | no root death |
| Shading: Soltani Eqn 9.11 / Teh Eqn 7.18 (Goudriaan & van Laar: ≤ 0.03 /day above LAI 4–5); Van Keulen & Seligman via Penning de Vries p. 101 (5 %/day above LAI 6, **built** 2026-08-15) | canopy closure | a seedling never reaches it | — |
| Freezing: Soltani Eqn 9.13 | min temperature below 0 °C | never fires at 4 °C | — |
| Soltani Ch. 17 (N mobilization drives leaf senescence) | N-limited crops; vegetative senescence only when N uptake is very limited | — | seed-fill N mobilization |

**All three crop-model books agree: no age-related leaf death before flowering, and what comes
after is paced by temperature or development.** None supports a daily rate during vegetative
growth. Our flat `rdr_leaf` was recorded in July as standing in for canopy regulation the tree did
not then have (`senescence.yaml`, the (C) diagnosis).

## 3. The July refusal, and which of its blockers are still gates (2026-10-06)

The DS-keyed form ("(C)") was priced and refused 2026-07-27/28 (`post-roadmap-nitrogen-cycle-
form.md`, "THE (C) DIAGNOSIS"; `senescence.yaml` header). Its blockers, re-checked against today's
tree — a refusal is dated to its state, and so is each gate it failed:

| July blocker | A gate today? |
|---|---|
| canopy peak LAI 16.4 without vegetative leaf death | **discharged** — the shading rule (5 %/day above LAI 6) was BUILT 2026-08-15 |
| `perennial` hard-errors under RK4 | **not a reference gate** — the biosphere is frozen at Euler; RK4 appears only in lab tests (`tests/leaf_form.rs`) |
| `perennial`'s decade CO₂ floor 0.05 | **yes** — `decade_min_carbon_pool_stationary` (`science_gates.rs`, the per-year minimum of the CO₂ pool, `non_collapsing(floor=0.05)`; its window was removed 2026-08-10, a tightening). ⚠ First cited here as `science_gates.rs:566`, which is the sibling LEAF floor (`perennial_decade_leaf_cycle_is_stationary_and_alive`); both floors exist (corrected 2026-10-06) |
| Greenwood biomass cap 14.4248 t/ha | **yes** — `tests/mutual_shading_tolerance.rs` |
| `rationed == 0` in every golden | **yes** |

The tree has changed since July in ways that bear on all of these: the step (`dt` 1/16), the
specific leaf area re-cited, the shading rule, temperature kinetics, the live-O₂ form, the soil
layers. None of July's numbers can be quoted as today's.

## 4. The options (for the user)

* **A — the sources' form.** No leaf or root death before flowering; after it, paced by
  development (Teh/SUCROS's `DVR/(2 − DVS)`, or Soltani's linear decline in temperature units);
  stems per Penning de Vries (no structural death). CITED. A biosphere unfreeze; it hits the
  carbon-limited chambers that refused (C) in July.
* **B — the flat rates converted to degree-days.** Keep today's magnitudes but count them per
  °C·day (so 4 °C sheds about a fifth as fast as 22 °C). ⚠ UNCITED: the conversion needs a
  reference temperature, which would be fitted — WHAT-IF, lab-only by the project's rules.
* **C — leave it, recorded.** The cold-week loss stays; slice 4 goes ahead.

## 5. The measurement plan (owed before the user can pick A or B; nothing frozen moves)

Through the existing lab seam (`domains::lab::mechanism`, `build_season_replacing` — one assembly
body), never by editing the frozen `senescence.yaml`. Predictions written before each run.

1. **Control:** the frozen form through the seam reproduces every golden bit for bit.
2. **A**, organ by organ (leaf, root, stem separately, then together — July's stem-only lesson:
   the organs do not move together), and **B** (WHAT-IF), on every biosphere golden scenario plus
   the sealed station, reporting the chambers and the station in **separate columns** (the
   station's shared air is not CO₂-limited, so it will look like an easy win while the jar and
   `perennial` pay).
3. Against today's live gates: the CO₂ floor, the Greenwood cap, `rationed == 0`, and the peak LAI
   band.

**One prediction already written:** with no vegetative shedding, the dim seedling ends the cold
weeks near **0.150 mol C** (0.160 − 0.0096), not 0.077.

## 6. Decisions owed to the user

1. Run the measurement plan (§5)? It means a lab flow for A (and B), predictions, and runs across
   every biosphere scenario — a sizeable piece of lab work, no frozen change.
2. If yes: A only, or A and B (B is WHAT-IF, lab-only, and cannot become the reference uncited)?

**ANSWERED 2026-10-06 (the user):** *"Do what is closer to reality."* Read as **A, measured** — the
sources' form is the cited description of real crops; B rests on a fitted reference temperature.
B is not run. The measurement plan (§5) proceeds for A, lab-only, predictions first.

## 7. A, made concrete — and the predictions, written before the lab flow exists (2026-10-06)

**The form (one per organ, so the organs can be measured apart — July's stem lesson):**

* **Leaf — Teh Eqn 7.17 (after Goudriaan & van Laar 1994):** age death 0 while DVS < 1; after
  anthesis `DVR / max(0.1, 2 − DVS)`, with `DVR = daily_thermal_time(T) / tsum_maturity` (the
  tree's own reproductive clock). Integrated, leaf then falls LINEARLY in development toward
  maturity — Soltani & Sinclair Eqn 9.7's shape, so two sources give one curve. The shading term
  (`shade_rate` above `lai_threshold`, Van Keulen & Seligman, built) is added unchanged. ⚠ DVR
  omits the drought hastening factor (1 whenever the crop is unstressed, as in every run measured
  so far) — recorded.
* **Root — Penning de Vries:** 0 while DVS < 1; `rdr_root` (0.01) after — the frozen value, within
  ~10 % of the source's post-anthesis 0.010–0.011 (rice). (Teh models no root death at all.)
* **Stem — Penning de Vries:** "except for their reserves, stems do not lose weight" → 0. July
  refused stem-only on `perennial`'s closure; measured separately here.
* **Nitrogen shedding** (`NitrogenSenescence`, sealed scenarios) carries the same rates — "one
  physical event, two legs" — so the lab replaces BOTH flows with one rate law. The open field has
  no nitrogen flow; its cells come from a carbon-only replacement (the harness marks a scenario
  n/a when a target is missing, so the two compositions are merged cell by cell, open field from
  one, chambers from the other).

**Columns:** frozen; **FLAT** (the lab flows at today's rates — the control); **L** (leaf);
**LR** (leaf + root); **LRS** (leaf + root + stem). Instrument: `lab::report` (the value/science
switch harness), `--long`, plus a station example for the sealed station.

**Predictions:**

| # | Prediction |
|---|---|
| S0 | FLAT reproduces the frozen column **bit for bit** in every cell, rationed and events included (the control; it is arithmetic, not science) |
| S1 | open field, L: peak LAI **rises** from 6.0228 but the shading term holds it in **6.0–7.0**; peak W **rises** and crosses the Greenwood point 14.4248 t/ha |
| S2 | every chamber's season-low CO₂ **falls** under L, further under LR and LRS (standing tissue holds carbon the air would have had) |
| S3 | perennial long horizon: the converged peak leaf **rises**; its CO₂ low **falls**; whether it stays above the decade CO₂ floor (0.05 mol) and rationed stays 0 is **not predicted** — the July blocker, the question this measures. LRS (stem) is the likeliest to break it (July: rationed 0 → 1 at dt = 1) |
| S4 | station, the 3b cold seedling at day 56 (0.077 mol C under the frozen form): L ≈ **0.11**, LR ≈ **0.14**, LRS ≈ **0.150** (= 0.160 − 0.0096, the cold weeks' net production) |

## 8. Measured (2026-10-06) — the predictions graded

**Built, lab-only:** `domains::lab::shedding` (one rate law, two flows — carbon and its nitrogen
twin — switchable per organ; a second cited leaf reading added after the first results,
`LeafShedding::PenningDeVries`, Listing 5's `LLVT` table); `tests/shedding_form.rs` (5: the
control on the open field and the jar, the law by hand, Penning de Vries' table by hand, the
reach); examples `domains/shedding_switch` (the lab report, `--long`) and
`station/shedding_station` (the sealed station, full horizon). Outputs in
`W:\temp\claude\shedding\`. Two temporary examples (not committed) booked the open field flow by
flow and applied the science-gate formulas to the lab runs.

Columns: frozen; FLAT (control); L (Teh's leaf); LR (+ root after anthesis); LRS (+ no stem death);
LRS-PdV (LRS with Penning de Vries' milder leaf table after anthesis).

| # | Prediction | Measured | Grade |
|---|---|---|---|
| S0 | FLAT = frozen bit for bit | every report cell; the station's END STATE equal; the two tests | HELD |
| S1 | open field (L): peak LAI rises, within 6.0–7.0; peak W rises past 14.4248 | LAI **5.44 → 6.18** (the "from 6.0228" in §7 was a stale figure — 5.44 is today's); peak W **falls** 12.97 → 10.05 t/ha | LAI HELD; **peak W MISSED, wrong sign** (below) |
| S2 | chambers' CO₂ low falls under L, further under LR, LRS | falls under L (jar 9.66 → 3.45, perennial 74.9 → 66.8, consumer 75.5 → 70.8 ppm); LR and LRS fall **slightly less** than L | first half HELD; "further" MISSED |
| S3 | perennial long: peak leaf rises, CO₂ low falls; floor / rationing not predicted | peak leaf **+81 %**; CO₂ low −10.8 %; every gate formula passes for every form: rationed 0, compensation ratio 1.22 → **1.09** (perennial), 1.24 → 1.16 (consumer), cycles stationary and alive | HELD; **July's blocker does not reproduce** |
| S4 | station cold seedling at day 56: L ≈ 0.11, LR ≈ 0.14, LRS ≈ 0.150 | **0.1168 / 0.1460 / 0.1521** (frozen 0.0771) | HELD |

**S1's miss, explained (booked flow by flow, open field):** the L crop grows MORE (allocation 100
vs 86 mol C) and is bigger at anthesis (leaf 9.5 vs 8.2, stem 10.0 vs 7.1, root 10.7 vs 6.6), but
Teh's rate takes the leaves to ~0 by maturity (0.05 mol C left vs the frozen 4.6), so grain fill
runs on a dying canopy: grain 33.4 → 24.2 mol C, and peak W comes 18 days earlier. July's
measurement used Penning de Vries' milder post-anthesis table, which is why it pointed up.

**Unpredicted — the after-flowering half decides the yield, and the two cited readings bracket it:**

| | frozen | L / LRS (Teh, Soltani) | LRS-PdV | WOFOST oracle* |
|---|---|---|---|---|
| open field peak LAI | 5.44 | 6.18 | 6.18 | 6.34 |
| open field leaf at maturity (mol C) | 4.63 | 0.05 | 6.76 | LAI 0 |
| open field grain (t/ha, 0.45 C) | 8.9 | 6.5 / 6.2 | 9.6 | 11.5 |
| open field peak W (t/ha; Greenwood cap 14.4248) | 12.97 | 10.05 / 10.62 | **15.67 — over the cap** | (TAGP 20.4) |
| station grain per season (mol C) | 24.26 | 11.15 / 10.86 | **61.17** | — |
| station soil humus at the end (mol C) | 7.54 | 6.02 / 5.28 | 17.05 | — |
| chambers: all gate formulas | pass | pass | pass | — |

\* `tests/oracle/winter_wheat_reference.json` — WOFOST 7.2 potential production, same site and
season; PCSE's OUTPUT, a diagnostic (EUPL; never ported).

**What it means.** The part all three books agree on — **no age-related shedding before
flowering** — is what the cold seedling needed, and it passes every gate in every scenario. After
flowering the two cited readings disagree, and this model's grain fill is sensitive enough that
one halves the station's grain while the other multiplies it 2.5× and breaks the frozen Greenwood
cap. Against WOFOST, Teh/Soltani gets the canopy's SHAPE right (leaves gone at maturity) and the
yield further off (6.5 vs 11.5 t/ha); Penning de Vries the reverse. That points at a second gap,
recorded, not measured further: this model fills grain slowly and leans on green leaves through
fill to do it (WOFOST has 5.7 t/ha of grain by DVS 1.27).

## 9. Second review (advisor, 2026-10-06) — a confound, two corrections, and the price

**Advisor, summarized:** (1) grain may be counted AFTER maturity — the green-leaf forms keep
filling it while the crop stands; measure grain on the day DVS reaches 2; (2) the "CO₂ floor"
wording was wrong as corrected mid-run — find the real CO₂ guard; (3) "passes every gate" is
wider than what was run — run the full suite on a throwaway build and count the reds; (4) read the
existing WOFOST record before calling slow grain fill a gap, and confirm the weather is the same;
(5) split the decision in two halves, and measure "the agreed half alone" before offering it.

**(1) The confound is real, and it flips the station's grain.** Grain on the first day-end past
maturity (`tt ≥ tsum_anthesis + tsum_maturity`) beside grain at the season's end:

| mol C | open field at maturity | open field at end | station at maturity (day 139) | station on the re-sow eve (day 305) |
|---|---|---|---|---|
| frozen | 25.91 | 33.40 | **9.51** | 24.26 |
| L (Teh leaf) | 24.09 | 24.20 | **11.14** | 11.15 |
| LRS (Teh, all organs) | 23.20 | 23.26 | **10.85** | 10.86 |
| LRS-PdV | 28.05 | 35.93 | **15.04** | 61.17 |
| BEFORE (agreed half only, new) | 27.95 | 35.30 | **14.58** | 41.02 |

At maturity the forms sit within −10 % / +8 % of frozen in the open field, and in the station
**every** source form gives MORE grain than frozen (+14 % to +58 %). The "station grain halves"
of §8 was the frozen crop filling grain for 166 days after maturity with green leaves — which a
real crop does not do, and which slice 4 (re-sow on maturity) removes. §8's grain rows are read
through this table from now on.

**(2) Both 0.05 floors exist.** `decade_min_carbon_pool_stationary` is the CO₂ one (per-year
minimum of the pool) and `perennial_decade_leaf_cycle_is_stationary_and_alive` the leaf one. The
census below ran both.

**(3) The price, counted** — `BEFORE` (the agreed half) wired into `Senescence` and
`NitrogenSenescence` on a throwaway build (`DVS < 1` → no leaf, no root shedding; nothing else),
`cargo test --release --no-fail-fast` plus `-p domains -p station -- --ignored`, then reverted
(`git status` clean on `flows.rs`). **29 reds in 11 binaries:**

* **Every plant science gate that judges survival PASSES** — the five compensation-point bands,
  the decade CO₂ floor, both decade leaf cycles, the consumer cycle.
* **One science gate goes red:** `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided`
  asserts the chambers' peak LAI stays below 1.0 ("carbon-limited by design and cannot reach the
  regime"); the jar reaches **1.024**. The source's threshold is 6.0, so the claim the 1.0 proxies
  for survives; the proxy does not.
* **The pinned margins:** the five compensation margins (the jar 14.15 → 5.00), the jar's
  tightest CO₂ step (0.165 → 0.342 of the pool — "rising is the jar closing on rationing").
* **Every golden** (domains, cheap and expensive station) and its tier band — expected of any
  carbon change.
* **The flat form's own unit tests** (3) and the lab's two FLAT controls — expected (the frozen
  flow changed under them).
* **Characterization pins measured on the flat form:** `mutual_shading_tolerance` (6),
  `value_switch_run` (3), `gas_composition_perturbations` (3), `leaf_form` (2),
  `temperature_kinetics` (1), `reaching_the_below_root_store…` (1). Each records a number of
  today's tree; adoption would re-derive each, as 3a did its roster.

**(4) WOFOST.** Same weather (NASA POWER, 52° N 5° E, 2006-10-01 → 2007-08-01, PCSE 6.0.13 — both
files' provenance). But the yield comparison is NOT like-for-like: `post-roadmap-oracle-match.md`
("ceremony 2") already found that the oracle's longer grain fill is a **different cultivar** and
the phase partition calendar-impossible to close. So §8's "second gap: slow grain fill" is
**withdrawn** — what the oracle supports is the canopy SHAPE (peak LAI 6.34; no leaf at
maturity), not a yield target.

**(5) The decision, in two halves:**

* **Before flowering — agreed by all three sources.** No age-related leaf or root shedding. It is
  what the cold seedling needed (0.077 → 0.146 mol C at day 56), every survival gate passes, and
  grain at maturity rises (station +53 %, open field +8 %). Price: 29 reds, one of them a science
  proxy (the jar's leaf area crosses 1.0), the rest pins to re-derive.
* **After flowering — disputed.** Teh / Soltani (leaves gone at maturity, WOFOST's shape) vs
  Penning de Vries' table (rice; leaves stay green). Entangled with the fixed re-sow calendar,
  which lets a green crop keep filling grain for months after maturity. Recommended: take it
  AFTER slice 4 (re-sow on maturity), which removes that confound.

**DECIDED 2026-10-06 (the user): "Adopt agreed half."** No age-related leaf or root shedding
before flowering becomes the frozen science — a biosphere unfreeze, predictions first, the 29 reds
re-derived one by one. The after-flowering question waits for slice 4.

## 10. The unfreeze — design, predictions and roster, before code (2026-10-06)

**Step 1 of the ceremony (`docs/biosphere-reference.md`, "The unfreeze discipline").**

**Why.** Three primary sources on the shelf (§2) agree that a crop sheds no tissue from age before
it flowers; the frozen flat rates (all `TODO(cite)`) shed it from day one. In the cold weeks of
the station's protocol that loss halves the seedling; measured in the lab (§8–§9) the agreed half
removes it and every survival gate still passes.

**The form — the smallest change that is the agreed half, and nothing else:**
* `Senescence` and `NitrogenSenescence` gain the development clock they need — the thermal-time
  aux key and `tsum_anthesis` / `tsum_maturity` from `phenology.yaml` (no new param) — and while
  `DVS < 1` the **age** part of the leaf rate and the root rate are 0. The mutual-shading term
  (V-K&S, built) is unchanged at every stage; the stem rate is unchanged (stem-only was refused in
  July on its own measurement and is not part of the agreed half); after anthesis every rate is
  today's flat one.
* Exactly the lab's `SheddingForm::BEFORE_ANTHESIS_ONLY` arithmetic.
* **Scope:** every crop that loads `senescence.yaml` — winter wheat and the potato, which shares
  the file. ⚠ LOCUS: Penning de Vries states the DS-keyed form for crops generally (§3.2.6);
  Soltani's Box 9.1 for grain crops; the potato is a tuber crop. Potato has no golden; its tests
  passed on the throwaway build.
* `senescence.yaml`: the three rates' `source` text says they apply AFTER anthesis and cites the
  zero before it; the header's (C) diagnosis is annotated; the stale `shade_rate` note the third
  direction plan's §4 waits on is corrected in the same edit.

**Predictions (written before the build):**

| # | Prediction |
|---|---|
| U1 | **Every regenerated golden equals the lab twin's run bit for bit** — the build is `BEFORE_ANTHESIS_ONLY`'s arithmetic, so the lab's report values for that column and `shedding_station`'s BEFORE end state are the new goldens' values exactly. After the build, the lab's `BEFORE_ANTHESIS_ONLY` through the seam reproduces the frozen run (the control moves from FLAT to it) |
| U2 | Goldens that move: the biosphere goldens with a vegetative crop (`season_euler`, `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`) and the station goldens carrying the crop (`sealed_station`; `greenhouse`, `harvest`, `lighting`, `station` if their crop is vegetative within their horizon). `drift_summary` and the non-crop goldens (crew, eclss, power ×2, thermal, cabin_gas, water_recovery, sealed_energy_drift) byte-identical |
| U3 | Biosphere manifest: `senescence.yaml`'s digest and the moved goldens' hashes; **no** flow-set, aux-set or param-set change (no new param, type names unchanged). Station manifest: the moved station goldens' hashes only |
| U4 | Readings (the throwaway build / lab): open field peak LAI ≈ 6.18, peak W below the Greenwood cap; chambers' compensation ratios ≈ 5.0 / 1.10 / 1.17 (jar / perennial / consumer); decade CO₂ floor and both leaf cycles pass; sealed station: the cold seedling ≈ 0.146 mol C at day 56, grain at maturity ≈ 14.58 mol C, 0 rationing |
| U5 | ⚠ **One science band fails as written:** `the_vks_mutual_shading_regime_is_modelled_not_merely_avoided` asserts the chambers' peak LAI < 1.0 and the jar reaches **1.024**. Argued here, not re-tuned: the 1.0 was never the source's bound — the source's threshold is 6.0 (V-K&S) and the claim the 1.0 stood for, "the chambers are carbon-limited and cannot reach the regime", holds with a 5.9× margin. Proposed: the gate asserts the claim (chambers < the source's 6.0 threshold) and records the measured 1.024; ⚠ that IS a bound change on a frozen gate, so it is the user's call, put to them with this argument |

**The roster — the 29 reds of the throwaway build, each with its planned handling:**

| Test(s) | Planned handling |
|---|---|
| 3 goldens + 3 tier bands (domains + station `golden_regression`, `tier_contract`) | regenerate; the tier bands' basis re-measured if a band is per-golden |
| `the_five_margins_are_pinned_not_merely_positive`, `the_jars_tightest_co2_step_is_pinned_by_its_headroom` | re-pin with the movement and its direction recorded (the jar's CO₂ margin 14.15 → 5.00; its tightest step 0.165 → 0.342 of the pool — the jar closer to rationing, stated) |
| `the_vks_mutual_shading_regime…` + `mutual_shading_tolerance::the_chamber_half_of_the_gate` | U5 — the user's decision |
| `flows::tests` ×3 (the flat form's unit tests) | re-express: before anthesis the age rates are 0, after it the frozen rates; the shading half unchanged |
| `shedding_form` FLAT controls ×2 | the control moves to `BEFORE_ANTHESIS_ONLY` (U1); FLAT becomes the "pre-2026-10-06 form" column |
| `mutual_shading_tolerance` ×5, `temperature_kinetics` ×1, `value_switch_run` ×3, `gas_composition_perturbations` ×3, `leaf_form` ×1, `system::reaching_the_below_root_store…` ×1 | each read for what it claims; re-derived on the new tree if the claim survives with new numbers, re-expressed if its premise was the flat form, recorded either way — **never loosened to pass** |

### 10a. Advisor review of §10 (2026-10-06, the ceremony's step 1), summarized — and what it changed

(1) Classify the 15 "read each" reds NOW, from the throwaway build's messages, into premise / new
number / claim fails — a failing science claim is a second blocking finding. (2) U1 and U4 used
borrowed numbers: run the lab report with the agreed-half column and save it, and set its peak W
against 14.4248. (3) Find every construction of the two flows (authoring, station, Godot) and
what the manifest records per flow. (4) U2 hedged on "vegetative within their horizon" and was
confident about `drift_summary` without checking. (5) Put U5 to the user now; check the 1.0's
history first. Fold-ins: the potato's pass used wheat's sums — re-run it on the real build; make a
missing development clock loud; list every prose the change makes false; add a station
unfreeze-log entry.

**(3) Constructions:** `Senescence` / `NitrogenSenescence` are built only in
`biosphere/system.rs` (`build_season_with`) and in `flows.rs`'s own unit tests; the authoring
manifest names neither; the biosphere manifest records flow TYPE NAMES, the science-gate bound
TEXTS and the golden hashes — the 1.0 lives inside the VKS gate's check, not in its recorded bound
text ("peak < 6.0 OR the 5%/day mutual-shading loss is MODELLED").

**(4) U2 corrected:** `drift_summary.json` folds the perennial and consumer chambers' leaf series
(`domains::goldens::drift_summary`) — it **moves**. `state_snapshot.json` is an engine fixture
(`simcore::snapshot`) with no crop — unchanged. Every crop golden starts at DVS 0, so every one
moves; the exact list comes from a report-only regeneration, diffed stock by stock before
`--write`.

**(5) The 1.0's history:** `docs/log/mutual-shading-tolerance.md` FINDING 5 — the chambers' "< 1.0"
is the project's own statement that they are "carbon-limited by design and cannot reach the
shading regime at all", never a sourced bound; it was found to be the second DETECTOR of a
`specific_leaf_area` error (crossed between ×2.5 and ×3.5). **DECIDED 2026-10-06 (the user):
"Restate to the source's 6.0."** Both copies move together (`science_gates.rs` and
`tests/mutual_shading_tolerance.rs::the_chamber_half_of_the_gate`); the detector role is lost,
recorded. Paired cost stated to the user: the jar's tightest single CO₂ draw 0.165 → 0.342 of
the pool, never rationing.

**(1) The 15, classified from their messages on the throwaway build:**

| Test | Message (throwaway build) | Class |
|---|---|---|
| VKS gate; `the_chamber_half_of_the_gate` | jar peak LAI 1.024 ≥ 1.0; "still inside the bound at ×2.5 — 2.567" | **claim (proxy) fails** → U5, decided |
| `the_loss_is_inert_on_both_observables…` | "the frozen canopy must stay under 6.0 — 6.177" | premise was the flat form: the shading loss is now LIVE at frozen params |
| `the_loss_delays_the_biomass_cap…`, `the_loss_roughly_doubles…` | "the frozen rung must be under 14.4248 / 8" (the loss-OFF ladder) | premise: without vegetative shedding the frozen canopy needs the loss to stay inside its bands |
| `the_loss_is_one_sided…` | the ×0.682 low rung reaches 6.04 vs 7.02 | premise: the low rung no longer sits below the threshold — a lower rung is needed |
| `the_peak_w_crest_is_light_saturation…` | "a 25 % cut in k must cost the frozen canopy real biomass — 0.031" | premise: the frozen canopy is now near light saturation, where k matters little |
| `temperature_kinetics::the_mutual_shading_step_is_what_caps…` | cardinal 6.18 → **14.82** with the loss off | premise: the loss is no longer inert in the frozen tree — the canopy is regulated by the cited shading rule, as July's (C) diagnosis predicted ("the flat `rdr_leaf` has been standing in for canopy regulation") |
| `value_switch_run` ×3 | peak leaf at k 0.55 / frozen / 0.65: 9.850 / 9.836 / 9.836 | premise: the harness's probe (higher k → higher peak leaf) is flat in a shading-capped canopy; a monotone probe is needed |
| `gas_composition_perturbations`: the vent, the reversal | at f = 0.8 the jar's low is 3.50 vs 3.42 baseline | new number: the depletion reversal now begins above f = 0.8 |
| `gas_composition_perturbations`: halving O₂ | "the healthy jar is not oxygen-limited at its trough" | new number, the statement inverts: the jar is further from anoxia |
| `leaf_form::the_form_no_longer_rations…` | the lab jar's tightest step 0.678 (pinned 0.3128) | new number |
| `system::reaching_the_below_root_store…` | "the canopy rescue moved: 1.029" | new number |

No science claim other than the U5 proxy fails; the Greenwood gate itself passed (peak W 13.88
t/ha < 14.4248 on the throwaway build).

**(2) The lab twin's own numbers** (`shedding_switch --long`, column `BEFORE (agreed half)`, run
on the pre-change tree; saved `W:\temp\claude\shedding\lab_twin_report.txt`) — **U1/U4 restated:
after the build, the frozen column of the same report must print exactly these values**, and the
lab's `BEFORE_ANTHESIS_ONLY` through the seam must equal the frozen run bit for bit:

| readout | today | the agreed half (= the new frozen) |
|---|---|---|
| open field peak LAI | 5.440614 | **6.177401** |
| open field peak W, t/ha (Greenwood cap 14.4248) | 12.966529 | **13.888379** — under the cap by 3.7 % |
| jar season-low CO₂, ppm | 9.661059 | **3.423283** |
| perennial chamber season-low CO₂, ppm | 74.861477 | **67.059152** |
| consumer chamber season-low CO₂, ppm | 75.544857 | **71.312478** |
| perennial long horizon, converged peak leaf, mol C | 0.587917 | **1.029832** |
| rationed / events, every run | 0 / 0 | **0 / 0** |
| sealed station (`shedding_station`): cold seedling at day 56; grain at maturity | 0.0771; 9.51 | **0.1460; 14.58** |

## 11. The build, PARKED — the drought response it removes (2026-10-06)

**Built, on branch `wip/leaf-shedding-adoption` (not on main):** the form in `Senescence` /
`NitrogenSenescence` (`age_shedding_rates`; a missing development clock refused); 10 goldens
regenerated — U1 **held to every printed digit** (the frozen column of `shedding_switch` after
the build equals the lab twin's before it, all eight readouts; the station's seedling 0.1460 and
grain at maturity 14.58); `senescence.yaml` provenance; the VKS chamber bound restated to 6.0
with its twin (measured: no chamber reaches 6.0 up to ×3.5 SLA); the margins re-pinned (all five
FELL; the jar's tightest step 0.165 → 0.342); the claim census. U2's corrected list held except
`harvest`, byte-identical — its crop starts past anthesis (`thermal_time0`).

**What stopped it — a roster item mis-classified in §10a as "new number":**
`system::reaching_the_below_root_store_is_what_saves_the_deep_water_crop`, the headline water
claim. With the deep store vs without:

| | before | the agreed half |
|---|---|---|
| peak leaf carbon ratio | 11.8× | **1.03×** (9.84 vs 9.56 mol C) |
| grain ratio | 7.7× | **1.22×** (12.52 vs 10.26) |

The droughted control used to collapse (peak LAI ≈ 0.5): its growth could not replace leaves dying
at 2 %/day — the cold seedling's spiral. Without the flat rate, drought slows growth and kills
nothing. **So the flat `rdr_leaf` was also standing in for drought-driven leaf death, which the
tree does not have.** Penning de Vries et al. (1989) §4.3.4 p. 141: "Severe water stress can lead
to progressive death and removal of leaf area… Van Keulen (1982) reduces biomass of wheat and
grasslands by 0.1–0.2 d⁻¹ when water stress exceeds a certain level" — a LEAD, not read at source.

**DECIDED 2026-10-06 (the user): "Pause; add drought shedding first."** The agreed half and a
cited drought-driven leaf death are to be adopted together, so the drought response is not lost.
Next: the drought-shedding study, sources first.

⚠ **Lesson:** §10a classified 13 reds from their messages without reading the SIZE of each move;
a two-sided pin failing at 1.03 against 11.1..13.0 was a claim collapsing, not a number drifting.

## 12. The drought question, measured — the precondition cannot be met from the shelf (2026-10-06)

**The shelf, searched for drought-driven leaf death.** Soltani & Sinclair Ch. 15–16 (their
water-limited model): drought slows leaf expansion (WSFL — refused here 2026-08-12 on the source's
own reason: it applies to a node-driven branch this canopy does not have, and would double-count
WSFG) and growth (WSFG, built), hastens development (WSFD, built), and **kills no leaf before seed
growth even under drought**; the only drought kill is crop TERMINATION after Sinclair & Amir
(1996), their Table 15.4 (FTSW < 0.10 with VPD > 2.20 kPa for 3 days; FTSW < 0.02 with VPD > 1.75
for 1; FTSW ≤ 0 with VPD > 2.20 for 1). Penning de Vries §4.3.4 p. 141 names Van Keulen (1982):
biomass reduced 0.1–0.2 d⁻¹ "when water stress exceeds a certain level" — not on the shelf, a
LEAD. Teh: nothing.

**Measured in the deep-water scenario** (a temporary probe, not committed; frozen-flat vs the
agreed half, each with and without the deep store):

| | frozen, with deep store | frozen, without | agreed half, with | agreed half, without |
|---|---|---|---|---|
| peak leaf carbon (mol C) | 5.672 | 0.480 | 9.836 | 9.555 |
| grain at MATURITY (mol C, day) | 7.42 (283) | 0.85 (276) | 10.09 (283) | 6.35 (276) |
| grain at the season's end | 9.60 | 1.24 | 12.52 | 10.26 |
| growth-stress factor WSFG: median / min / days < 0.5 | 1.00 / 0.27 / 62 | 0.81 / 0.26 / 100 | same as frozen | same as frozen |
| max VPD over the season | 0.75 kPa | 0.75 | 0.75 | 0.75 |

* **Termination can never fire here:** the Dutch weather's VPD never passes 0.75 kPa; the rule
  needs 1.75–2.20.
* **The soil water does not depend on the crop.** FTSW and VPD are IDENTICAL between the two
  forms, though the crops differ ~20× in leaf: transpiration does not read the canopy (§21 of the
  room-temperature plan, and slice 3b's water-ring finding). So the control's drought from day 11
  — October, a seedling — is set by weather and irrigation, not by what a seedling uses; a real
  seedling would barely dry the soil. Both forms are judged against a drought the model invents.
  **A second gap**, separate from shedding.
* **The rescue at maturity:** 8.7× grain (frozen) → **1.59×** (agreed half). Removing the flat
  rate is what removes the control's collapse; whether that collapse was REALISTIC is not shown
  either way — the control sits at a median WSFG 0.81 with 100 days under half rate, real stress.

⚠ **Owning the earlier framing.** §11 said "the flat rate was also standing in for drought-driven
leaf death" as a finding, and the user's pause rests on it. What is measured is narrower: the flat
rate produces a collapse in this scenario, and no cited mechanism on the shelf reproduces it.

**The decision returns to the user** (options in the reply): resume the parked adoption with the
deep-water claim re-expressed and two gaps recorded; stay paused for Van Keulen (1982); stay paused
to make transpiration read the canopy first (a separate, larger unfreeze); or abandon.

**DECIDED 2026-10-06 (the user): "Resume the adoption."** The parked build (branch
`wip/leaf-shedding-adoption`) is brought back onto main; the deep-water claim is restated at its
measured size; two gaps are recorded (no sourced drought leaf death; crop-blind water use).

## 13. ADOPTED (2026-10-06, a biosphere unfreeze) — graded

| # | Prediction | Measured | Grade |
|---|---|---|---|
| U1 | the new frozen run = the lab twin, bit for bit | every readout of `shedding_switch` (frozen after = agreed half before, 8 of 8 to every printed digit); the station's seedling 0.1460 and grain at maturity 14.58, its twin's end state = frozen; the lab control (`tests/shedding_form.rs`) bit-identical on the open field and the jar | HELD |
| U2 | (corrected in §10a) every crop golden + `drift_summary` move; non-crop goldens identical | 10 move; `harvest` **byte-identical** (its crop starts past anthesis, `thermal_time0`) | HELD but for `harvest` |
| U3 | manifests: `senescence.yaml` + the moved goldens' hashes, no set change | exactly that: 1 digest + 7 biosphere + 3 station golden hashes | HELD |
| U4 | the readings | as the lab twin (U1); rationed 0 everywhere | HELD |
| U5 | the VKS chamber bound fails at 1.0 | jar 1.024; restated to 6.0 by the user; no chamber reaches 6.0 up to ×3.5 SLA | HELD (decided) |

**The roster, done** (each with its measured numbers in the test's own doc): the margins re-pinned
(all five fell); the jar's tightest step re-pinned (rose); the VKS gate and its twin restated;
the flat form's unit tests moved past anthesis and the zero before it pinned; a missing clock
refused (new test); the lab control moved to the agreed half; the deep-water claim re-stated
(collapsed, two gaps); the Q10 2×2 re-stated (the loss now caps both forms); the value-switch probe
moved to the jar (the open field's peak leaf is flat in k under the cap); the shading ladder
re-measured (inverted — LAI ceiling ×1.616 on / ×0.719 off; the biomass cap unreachable with the
loss on, its crest 14.186 at ×3; the one-sided rung moved to ×0.3); the room-size tests inverted
(a smaller room reads a higher low at every size; the explicit form's backstop now fires below
0.60, was 0.17); the jar's O₂ trough factor 0.606 → 0.758; the lab leaf form's jar draws
re-pinned (0.678 explicit, 0.382 under C); the claim census +2.

**Gaps recorded, not acted on:** (1) no sourced drought-driven leaf death (Van Keulen 1982 a lead);
(2) crop-blind water use — transpiration does not read the canopy, so a seedling dries the soil as
a full canopy would; (3) the after-anthesis form, disputed between the sources, waits for slice 4.

