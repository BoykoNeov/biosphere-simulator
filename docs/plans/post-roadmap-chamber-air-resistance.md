# Post-roadmap: pricing a chamber air resistance — what a lower `ra` would move

**Opened 2026-10-14 on the user's call** (*"2"*: price changing the air-resistance value), chosen
after Step 6's water-gap search (`post-roadmap-real-world-checks.md` §11–§12): at the trial's level
(4.17 L m⁻² d⁻¹, 20 m² basis) the model's crop drinks 0.62 of TM 102788's water, and a canopy air
resistance `ra` of ~8.8 s m⁻¹ instead of the uncited 50 would close it alone (§11b Q4).

**This is a PRICE, not an adoption.** Nothing on main changes; the run is on a local branch, never
pushed. Adopting any value would be a biosphere + station unfreeze, and a science item.

## 1. The user's decisions (2026-10-14)

1. **Values: a spread, 50 → 30 → 15 → 9 s m⁻¹** (over the fitted ~9 alone, and over stopping to look
   for a source). The spread shows the cost curve without choosing a number. ⚠ No value below 50 has a
   source: 9 is FITTED to TM 102788 (it would pass that row by construction), and 30 / 15 are points on
   a curve, not citations. ⚠ *(Later 2026-10-14: a peer-reviewed measurement now exists — Monje & Bugbee 2019, ~6.4–7.5 s m⁻¹ in a faster-ventilated HPS wheat chamber; `post-roadmap-real-world-checks.md` §13e–§13i. The decision to keep 50 stands.)* Adopting one would need either a source or the user setting aside the rule
   that tuned values stay lab-only (`docs/param-file-conventions.md`, WHAT-IF).
2. **Scope: the open field keeps 50**; the chamber value applies to enclosed rooms. Priced as **two
   lines**:
   * **Line L — lamp-lit chambers only** (the user's first pick): the `lighting` and `sealed_station`
     builds (and the lab builds derived from them: day-neutral lighting, separate air).
   * **Line S — every sealed chamber** (added on the user's pick *"price both lines"*, after the scenarios
     turned out to fall in three groups, not two): Line L plus the five weather-lit sealed builds —
     the sealed, perennial and consumer chambers, the greenhouse and the harvest.

## 2. Advisor review (2026-10-14), summarized

Ask the value and the scope before any code (done, §1). Inventory every reader of `ra` before
predicting. Re-check the station's heat books for latent heat (the 2026-10-07 check predates the
2026-10-10 water-forms adoption). Do NOT carry "water only" over from the leaf-area pricing: that
change LOWERED water use; this one RAISES it (~1.6× at full cover), so any build whose watering
cannot keep up could cross the stress thresholds and move carbon — predict it per build. Same method
as the leaf-area pricing (`post-roadmap-canopy-resistance.md` §9–§10): predictions first, a local
branch, `regen_goldens` in report mode, the full suite `--no-fail-fast` plus `--ignored`, every red
classified, a price table and no recommendation.

## 3. Checked before predicting

* **One reader.** `ra` reaches the model in exactly one place: `Transpiration`'s
  `aerodynamic_resistance` (`domains/src/biosphere/flows.rs`, both the soil-split path and the plain
  path), built by `transpiration_flow` in `system.rs` from `p.transp`. The soil-evaporation potential
  does not read it; no leaf temperature is modelled (the §11 leaf-over-air figures are the instrument's).
* **No latent heat in the station** (re-checked 2026-10-14): no `station/src/*.rs` and not
  `domains/src/thermal.rs` carries a latent or evaporative term. More transpiration cannot cool or warm
  any node, so the plants' temperature cannot move through this route.
* **The station's builders call `build_season(&scenario.bio)`**, which reads `params::biosphere()`
  itself; their own `params::biosphere()` calls seed only the soil account. So a scope must be carried
  by the **scenario**, not by the params a builder passes. This matches the physics: `ra` is a property
  of the room's air movement (the param file itself says "wind/height-dependent"), like `ground_area`.
* **Every lamp-lit build is sealed** (`lighting_bio_scenario`, `day_neutral_lighting_bio_scenario`, the
  sealed station's), so Line S contains Line L.
* **The open-build TM scorecard is outside both lines**: `scorecard_tm102788.rs` hosts the held-CO₂
  trial on an OPEN build with the lamp's forcings swapped in, so a scenario-scoped value does not reach
  it. Its W1 row at each value is the §11 instrument's (`ra` substituted through the what-if path).
* **Watering:** event watering (adopted 2026-10-10) refills at FAO-56's depletion fraction 0.55 (FTSW
  0.45) with a capacity of 8 mm/day; sealed chambers water from their condensate (`Recycling`).
  Development speeds up below FTSW 0.40 (`wssd`), growth and transpiration slow below 0.30 (`wssg`).

## 4. The arithmetic (the model's constants; `W:\temp\claude\air-res-price\pm.py`, kept at `sources\temp-evidence\air-res-price\`)

Penman–Monteith at full cover, `rs` 50 (Szeicz–Long at LAI ≥ 2), demand per day:

| `ra` | lamp (TM point: 20 h at 20 °C, Rn 81, VPD 444; 4 h dark at 16 °C) | sunlit sealed, air at 75 % RH: 10 °C / Rn 40 | 17 °C / Rn 100 | 22 °C / Rn 130 |
|---|---|---|---|---|
| 50 | 2.58 mm/d (×1.00) | 1.76 | 3.31 | 4.43 |
| 30 | 2.99 (×1.16) | 2.13 (×1.21) | 3.74 (×1.13) | 4.97 (×1.12) |
| 15 | 3.64 (×1.41) | 2.68 (×1.52) | 4.41 (×1.33) | 5.84 (×1.32) |
| 9 | 4.12 (×1.60) | 3.04 (×1.73) | 4.89 (×1.48) | 6.50 (×1.47) |

The highest daily demand in the table, 6.5 mm/day at `ra` 9 on a warm bright day, is under the
watering's 8 mm/day capacity. A sealed chamber's air holding near its 75 % humidity target limits the
deficit further (more transpiration → humid air → condensed by the dehumidifier, not a drier leaf).

⚠ *Corrected 2026-10-14, after the run (advisor review):* the sunlit columns assume FULL cover, which
those chambers never reach — the canopy-resistance record measured their LAI never passing 1.02. At
LAI 1.0 (Szeicz–Long `rs` 100, crop share `1 − e^−0.5` of Rn) the same days give **×1.15–1.19 at 30,
×1.35–1.42 at 15, ×1.47–1.55 at 9**, and the highest demand falls to **3.55 mm/day**; at LAI 0.5, ×1.11–1.13 /
×1.22–1.25 / ×1.28–1.31. The ratios barely move and the watering headroom only grows, so P4's reasoning
stands, but its stated basis was wrong. The lamp column is unaffected (§11 of the real-world-checks plan
checked it against the model to 0.15 %). §8 measures the actual throughput.

## 5. Predictions (written before the branch exists)

| | prediction | confidence |
|---|---|---|
| P1 | the branch with no override is inert: `regen_goldens` reports 20 of 20 identical | high |
| P2 | Line L at 30, 15 and 9: exactly `lighting_state` and `sealed_station_state` would change; the other 18 identical | high |
| P3 | Line S at 30, 15 and 9: those two plus `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon`, `greenhouse`, `harvest` — **nine**; `season_euler`, `drift_summary`, `sealed_energy_drift_summary` and the eight crop-free goldens identical | high |
| P4 | **water only, in every moved golden at every value**: water stocks (soil, subsoil, condensate, vapour) and the soil account's water aux move; no carbon, nitrogen, oxygen, energy or phenology value moves — because watering keeps FTSW above 0.40 everywhere (§4: peak demand under capacity) | medium — the claim the advisor said not to assume; a carbon move refutes it and is the headline |
| P5 | the size grows with `1/ra`: every stock that moves at 30 moves further at 15 and further again at 9, same sign | medium (end states are snapshots inside a watering cycle, so a single stock can be non-monotone) |
| P6 | reds by design on the branch at Line S / 9: `golden_regression` (domains and station) and the tier bands on the nine moved goldens; the sealed station's golden in `--ignored`. The manifest byte gates stay **green** (report mode does not rewrite the files — the 2026-10-07 lesson) | high |
| P7 | possibly red, classified after: tests pinning water, humidity or condensate in a sealed or lamp-lit run (chamber dryness, vapour, dehumidifier, watering, minute-step transpiration, separate air, lamp shed, the re-sow cycle test) | — |
| P8 | green: the TM scorecard (outside both lines), every carbon / energy / gas test, `drought_window`, the conservation and determinism laws | high |
| P9 | Line L's reds are a subset of Line S's at the same value | high |

## 6. The branch (throwaway)

`wip/chamber-ra-price`, local, never pushed. A scenario field `lamp_lit` (true on the three lamp-lit
scenario builders) and a pricing hook in `transpiration_flow` that reads two environment variables —
`PRICE_RA_SCOPE` (`lamp` | `sealed`) and `PRICE_RA_VALUE` — so one build prices all six cells. An
environment read is NOT an adoption shape (parameters are data); it exists only so the price runs
without six recompiles. Logs: `W:\temp\claude\air-res-price\` (scripts, comparison and logs kept at
`sources\temp-evidence\air-res-price\`; the per-cell golden copies were deleted — the branch regenerates them).

---

## 7. The price, measured on the local branch (2026-10-14) — graded

Branch `wip/chamber-ra-price` (local, never pushed; commit `101aa1a` on top of `d82cf12`). Each cell:
`regen_goldens -- --write` with the two variables set, the 20 files copied to
`W:\temp\claude\air-res-price\<cell>\`, then `rust/data/golden` restored from git (the tree was clean
after the seventh cell). Per-leaf comparison against the branch's own unset run: `diff.py` → `diff.txt`
in the same folder. Suites at `ra` 9: the full debug suite `--no-fail-fast` for each line, and `--ignored
--release` for Line S.

### 7a. The predictions, graded

| | predicted | measured | grade |
|---|---|---|---|
| P1 | unset: 20 of 20 identical | **0 rewritten** | HELD |
| P2 | Line L: exactly `lighting` and `sealed_station` | **2 rewritten**, those two, at 30, 15 and 9 | HELD |
| P3 | Line S: those two plus the seven sunlit sealed builds; the other eleven identical | **9 rewritten**, the predicted nine, at every value | HELD |
| P4 | water only, everywhere | **held bit for bit**: in all 6 cells the only leaves that differ are water stocks (soil, subsoil, condensate) and the soil account's water aux (`top_soil_water`, `soil_evap_today`, `soil_dry_days`). No carbon, nitrogen, oxygen, energy, `thermal_time`, `rooted_depth`, `vernalization_days` or `soil_shade_lai` leaf moves. Even at 9 no build's watering falls behind | HELD |
| P5 | the size grows with `1/ra`, same sign | held for the three short runs (`lighting` −0.08 / −0.14 / −0.17 % soil; `greenhouse`, `harvest` likewise). **Refuted for every multi-year sealed run**: the end states swing with sign changes — the sealed station's soil +36 % at 30, +18 % at 15, −17 % at 9; the perennial long horizon's condensate −25 %, +5 %, −75 % | REFUTED (for snapshots of long runs). ~~The end state is a snapshot inside a watering cycle~~ — first written as the cause, untested; §8 measured it: true for the re-sown chambers' soil, **false** for the sealed station's subsoil and condensate (a steady shift) and the never-re-sown chamber |
| P6 | reds by design: the goldens' regression and band checks; manifest byte gates green | domains + station `golden_regression`, both `tier_contract` checks (cheap and the `--ignored` expensive) red; **manifest byte gates green** | HELD |
| P7 | possibly red: water-pinning sealed / lamp tests | **none of them** (chamber dryness, vapour, dehumidifier, watering, minute-step, separate air, lamp shed all green). Red instead: the re-sow cycle test (7b) and the canopy-resistance instrument (7c) | none fired as listed |
| P8 | green: the TM scorecard, carbon / energy / gas, `drought_window`, the laws | green | HELD |
| P9 | Line L's reds ⊆ Line S's | L: station golden + band, plus the two branch artifacts (7c) — all in S's list | HELD |

Suite at 9, Line S: 8 red (83 binaries green). Line L: 4 red. `--ignored`, Line S: the sealed station's
golden and band red, every other slow test green.

### 7b. The one behaviour red: the re-sow cycle settles in two cycles, not one

`the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years` (Line S, 9): the perennial chamber's
below-root store at each cycle's start reads **167.904, 167.582, 167.582 (−4.7e-10), 167.582 (−1.2e-12)**.
The test asks every cycle after the first to sit within 1e-12 of the second; the third is 4.7e-10 away.
So it still converges — by ~400× a cycle — and the frozen claim "one transient cycle, then a fixed point
to round-off" becomes "two cycles". Not run at 30 or 15.

⚠ **Correction (2026-10-15, the adoption's §9b).** Not the one behaviour red: the same test's next check —
the two soil stores' total at each re-sow, held to a flat 1e-14 — would have gone red at 9 too (−1.83e-14
by cycle 4; at 15, −1.74e-14), unseen because the test stopped at its first failing check. It is a rounding
lean in the water path, present at 50 and faster at a lower `r_a`
(`docs/plans/post-roadmap-chamber-air-resistance-adopted.md` §9b).

### 7c. Two reds belong to the branch, not to the price

* `scenario_flag_types::the_toggle_roster_is_every_bool_field_of_the_scenario` — the branch's new
  `lamp_lit` bool is not on the roster. An adoption carrying the scope as a scenario bool owes that line;
  one carrying a scenario VALUE (`ra` per room) would not hit this gate.
* `canopy_resistance::each_lab_form_reaches_the_water_flow_exactly` — its by-hand side reads the param
  file's 50 while the sealed flow reads the override. An adoption would make it read the scenario's value.
* (On main, not the branch: `repo_gates::every_plan_doc_is_indexed` was red from `d82cf12` until this
  plan's log row landed — the plan was committed before its index line.)

### 7d. The price table

**What moves** (end state against the unset run, %). ⚠ For the multi-year runs these end states are not
the size of the change: they say *that* the water moves, not *how much*. Run means and throughput: §8.

| golden | line | 30 | 15 | 9 |
|---|---|---|---|---|
| `lighting` (7 days) | L, S | soil −0.08, condensate +0.19 | −0.14 / +0.33 | −0.17 / +0.40 |
| `sealed_station` (4 yr) | L, S | soil +36, condensate −61, subsoil +36 | +18 / −36 / +36 | −17 / +15 / +33 |
| `greenhouse` (7 days) | S | soil −0.07, condensate +0.14 | −0.12 / +0.25 | −0.14 / +0.29 |
| `harvest` | S | soil −0.006, condensate +0.09 | −0.010 / +0.16 | −0.012 / +0.19 |
| `sealed_chamber` (3 yr) | S | soil −2.1, condensate +3.7 | −3.4 / +5.8 | −27 / +47 |
| `perennial_chamber` (5 yr) | S | soil +72, condensate −68 | +36 / −33 | +11 / −10 |
| `perennial_long_horizon` (15 yr) | S | soil +19, condensate −25 | −4.1 / +5.3 | +55 / −75 |
| `consumer_chamber` (5 yr) | S | soil −9.1, condensate +17 | −22 / +40 | +8.0 / −15 |
| `consumer_long_horizon` (15 yr) | S | soil +50, condensate −44 | +37 / −33 | +29 / −25 |
| the other 11 (incl. the open field) | — | identical | identical | identical |

**How much more water flows** — not in any golden; **measured in §8**: the sealed station +20 / +54 / +77 %,
the sunlit chambers +4 to +19 % at any value.

**Ceremony an adoption would add** (beyond the run):
1. **A value.** None below 50 has a source. ⚠ *(Later 2026-10-14: a peer-reviewed measurement now exists — Monje & Bugbee 2019, ~6.4–7.5 s m⁻¹ in a faster-ventilated HPS wheat chamber; `post-roadmap-real-world-checks.md` §13e–§13i. The decision to keep 50 stands.)* 9 is fitted to TM 102788 **on its 20 m² basis only**: on the
   16 m² of trays the same match needs ~1.5 s m⁻¹ (§11b of the real-world-checks plan), which no chamber
   plausibly has, so a better match to the trial is not evidence for the value. Adopting 30, 15 or 9
   needs a source or the user setting aside the WHAT-IF rule.
2. **A shape.** `ra` becomes a property of the room (scenario data, like `ground_area`) or a second param
   (`chamber_aerodynamic_resistance`) with a scope rule — either changes a schema and the biosphere
   manifest's `transpiration.yaml` / scenario rows.
3. **Manifest rows:** 2 (Line L) or 9 (Line S) `golden_sha256` rows across the biosphere and station
   manifests; the contracts' dated entries.
4. **Tests restated:** the re-sow cycle test (two cycles, at 9); the canopy instrument reads the scenario's
   value; a bool shape adds a roster line.
5. **The TM scorecard sits outside both lines** (an open build). Adopting would leave it reading 50 unless
   it opts in, and if it opts in at 9 its water row passes by construction.
6. **Potato** inherits the wheat transpiration file — no golden holds it.
7. **What "water only" cannot see.** The model gives evaporation no heat: no station node and not
   `thermal.rs` carries latent heat (§3), and the dehumidifier draws no power for what it condenses. In a
   real chamber up to ×1.8 the transpiration would cool the leaves and the room and load the dehumidifier.
   So the energy side of this price is **unmeasured, not zero** — "nothing but water moves" means nothing
   the model can see moves.

Nothing adopted; main keeps `ra` 50 everywhere. The branch is kept, local.

---

## 8. Run means and throughput (2026-10-14, advisor review of §7)

**The advisor's two objections** (summarized): the sealed station's subsoil (+36 / +36 / +33 %) does not
fit "a snapshot inside a watering cycle" — same sign, same size at every value is a shift — and that
cause was asserted, never tested; and §4's sunlit arithmetic assumed a canopy those chambers never
reach (corrected in §4). Measure run means and the root zone's outflow on the branch before calling it.

**The probe** (`zz_ra_probe.rs`, committed to the branch only; copy in `sources\temp-evidence\air-res-price\`):
the crop's `Transpiration` flow wrapped in a recorder that sums its root-zone leg — the crop's
transpiration **plus the soil's evaporation**, which leave the root zone together — and a per-step
observer for the means (daily states for the sealed station). Line S. Every run wrapped exactly one flow,
rationed 0. Log: `probe.txt`.

**Outflow from the root zone, kg over the run** (change against unset):

| build | unset | 30 | 15 | 9 |
|---|---|---|---|---|
| `sealed_station` (lamp, 4 yr) | 1860.6 | 2236.7 (**+20 %**) | 2858.8 (**+54 %**) | 3287.1 (**+77 %**) |
| `sealed_chamber` (3 yr, never re-sown) | 563.5 | 649.2 (+15 %) | 648.3 (+15 %) | 668.2 (+19 %) |
| `perennial_chamber` (5 yr) | 2095.0 | 2224.6 (+6 %) | 2383.7 (+14 %) | 2410.0 (+15 %) |
| `consumer_chamber` (5 yr) | 1907.8 | 1990.9 (+4 %) | 2016.8 (+6 %) | 2097.9 (+10 %) |

The sunlit chambers move far less than the arithmetic's ×1.3–1.55: their crop is small or dead much of
the run, and the soil's evaporation — which does not read `ra` — is part of the outflow. The lamp-lit
station, a closed canopy under a lamp, moves more than the §4 lamp column (×1.77 against ×1.60): its
operating point is not TM's. Not decomposed further.

**Run means** (change against unset):

| build | soil 30 / 15 / 9 | subsoil 30 / 15 / 9 | condensate 30 / 15 / 9 |
|---|---|---|---|
| `sealed_station` | +0.2 / +0.7 / +2.4 % | **+5.7 / +10.6 / +24.7 %** | **−8.5 / −16.3 / −39.2 %** |
| `perennial_chamber` | +3.7 / +2.3 / +3.4 % | −3.8 / −4.0 / −3.2 % | −3.2 / −0.8 / −3.1 % |
| `consumer_chamber` | +1.2 / +1.7 / +2.3 % | −2.5 / +6.8 / +1.1 % | −0.2 / −6.6 / −3.9 % |
| `sealed_chamber` | −16.4 / −2.1 / −16.7 % | +0.1 / +0.1 / +0.1 % | +36 / +4.6 / +37 % |

**Graded, the cause first written in §7:**
* **The re-sown chambers: the snapshot explanation holds.** Their run-mean soil moves 1–4 % where the end
  states swung up to 72 %.
* **The sealed station: it does not.** Water moves steadily out of the condensate and into the deep store,
  growing with `1/ra` — the shift the advisor read from the subsoil. Its soil's end-state swing (+36 /
  +18 / −17 %) is a snapshot (means +0.2 to +2.4 %). Why the condensate drains into the deep store is
  **not established** (a lead, untested: more watering events, each overfilling the root zone into the
  subsoil).
* **The never-re-sown sealed chamber: neither.** Its means swing too (soil −16 / −2 / −17 %), so its split
  between soil and condensate is sensitive to the value in a way a snapshot does not explain. Not
  attributed.

**What the price now says about size:** a chamber value would send +4 to +19 % more water through the
sunlit sealed chambers and +20 to +77 % through the lamp-lit station, with the station's condensate store
running down by up to 39 % on average. Carbon and every other quantity: unchanged (§7a P4), within what the
model can see (§7d point 7).

---

## 9. The user's decision (2026-10-14): keep 50 — CLOSED, nothing adopted

**The user's call:** *"1. use 50"*. Every room keeps the uncited `ra` = 50 s m⁻¹; no chamber value is
adopted, no param, golden or manifest moves. The price above stands as the record of what a lower value
would have cost. The local branch `wip/chamber-ra-price` was never pushed and is left as it is until the
user says whether to delete it.

⚠ **What keeping 50 does and does not say.** It is a decision about the model, not evidence about TM
102788: the trial's water gap (`post-roadmap-real-world-checks.md` §11–§12) stays open, and air coupling
stays a candidate cause there. It does mean the station's lamp-lit chambers keep whatever share of that gap
air coupling would have explained. The water gap's next round is §13 of that plan.
