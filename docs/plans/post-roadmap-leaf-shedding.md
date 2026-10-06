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
