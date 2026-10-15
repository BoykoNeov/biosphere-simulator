# Post-roadmap: a cited air resistance for every sealed chamber — 7.5 s/m

**Opened 2026-10-15 on the user's call** (*"1"*, from a list of next items whose first was "the crop drinks
too little water: we now have a source for the air resistance"). The price is
`post-roadmap-chamber-air-resistance.md` (closed 2026-10-14 with 50 kept, because no value below 50 had a
source). The source arrived the same day, after the close: `post-roadmap-real-world-checks.md` §13e–§13j.

**This is a biosphere + station unfreeze and a science item.** The open field keeps 50.

## 1. The user's decisions (2026-10-15)

1. **Value: 7.5 s m⁻¹**, the cited number (over a judgement value such as 15, which would be WHAT-IF).
2. **Scope: every sealed chamber** (the price's Line S, 9 goldens), over lamp-lit only (Line L, 2 goldens).

The caveat, put to the user in the question and repeated by a side note they forwarded after answering:
the source's fans blew 1.7 m s⁻¹ over the crop, NASA's BPC 0.2–1.2, so in a slower-aired room 7.5 is a
**lower bound** on the resistance, not an estimate — the best-ventilated end of the range. The station's
own air speed is not known. The user's choice stands with that on the table.

## 2. The source and the number

Monje, O. & Bugbee, B. (2019), *Radiometric Method for Determining Canopy Stomatal Conductance in
Controlled Environments*, *Agronomy* 9(3):114, doi 10.3390/agronomy9030114 (open access, CC BY 4.0; on the
git-ignored shelf as `sources/monje_bugbee_2019_agronomy9_114.pdf`). p. 19, Discussion: *"The gA for the
wheat (5.5 mol m⁻² s⁻¹ or 7.5 s m⁻¹) … were within the range of typical aerodynamic conductances of field
crops (… 4–13 s m⁻¹)"*. Locus: two ~1 m² EGC-13 chambers at Utah State, HPS-lit, wheat at LAI > 15, fan
1.7 m s⁻¹ above the canopy (p. 5).

**The conversion, checked against the model's own air** (advisor, 2026-10-15). Penman–Monteith in
`science.rs` uses `AIR_DENSITY` 1.205 kg m⁻³ — sea-level air at 20 °C. The same molar conductance at
101.325 kPa: `P / (R T) / 5.5` = **7.56 s m⁻¹ at 20 °C, 7.53 at 21 °C**. So the paper's own 7.5 is the
value at the model's air; §13e's 6.4 is the same conductance at the paper's 86 kPa altitude, which no
model room has. Script: `W:\temp\claude\ra-adopt\pm75.py`.

## 3. Advisor review (2026-10-15), summarized

Ask the value and the scope first (done, §1); correct the "four goldens" I had told the user (the price
says 2 or 9). Settle 7.5 vs 6.4 on the model's air density (done, §2). Don't cherry-pick the pricing
branch — its environment hook was a pricing device. Write predictions before the run and don't carry them
over: 7.5 is below the lowest priced value (9), so re-check that watering keeps up, re-check "water only"
(a carbon move would be the headline), and expect the sealed station's throughput and condensate drain to
exceed the price's 9 cell. Ceremony: plan + log index row together; cite the paper as the value's source, with its locus
caveat (another chamber, faster air); regenerate goldens and manifests; if the re-sow cycle test goes
red again, measure and restate, never just widen the 1e-12; the canopy-resistance instrument reads the
scenario's value; record whether the TM scorecard opts in; the energy side is unmeasured, not zero; full
suite `--no-fail-fast`, then `--ignored`, below-normal priority.

## 4. The shape

* **A second param**, `chamber_aerodynamic_resistance` in `transpiration.yaml` (7.5 s/m, cited), beside
  the open field's `aerodynamic_resistance` (50, still `TODO(cite)`). Parameters are data; the value has a
  source, so it lives in the file that carries sources.
* **The scope is the scenario's existing `sealed` flag.** Every sealed build is a chamber; the open field
  is not. No new scenario field, so no toggle-roster line (the price's bool-shaped red does not arise).
  An authored sealed scenario gets the chamber value too — a sealed room is a room.
* **One selector**, `TranspirationParams::aerodynamic_resistance_for(sealed)`, read by
  `transpiration_flow` and by every by-hand instrument that rebuilds the flow (the canopy-resistance
  instrument, the `chamber_dryness` example), so no copy can read 50 for a sealed room.
* **Two commits** (the house rule: inert plumbing first). (A) the param, its loader line, its positivity
  guard and the selector, read by nothing — 20 of 20 goldens identical, the biosphere manifest moves only
  `transpiration.yaml`'s digest. (B) `transpiration_flow` reads the selector — the 9 goldens move.
* **The TM 102788 scorecard stays out** (an open build with the lamp's forcings swapped in, so the
  scenario-scoped value does not reach it): it pins no ratio, and its §13 tests already print the trial's
  water row at 7.5 (1.03 of the level). Opting it in would need a scope device on an open build. Recorded,
  not built; the user may ask for it.

## 5. The arithmetic at 7.5 (the price's §4 instrument, extended)

Penman–Monteith at full cover, `rs` 50:

| `ra` | lamp (TM point) | sunlit 10 °C / Rn 40 | 17 °C / Rn 100 | 22 °C / Rn 130 |
|---|---|---|---|---|
| 50 | 2.58 mm/d | 1.76 | 3.31 | 4.43 |
| 9 | 4.12 (×1.60) | 3.04 (×1.73) | 4.89 (×1.48) | 6.50 (×1.47) |
| 7.5 | 4.27 (×1.66) | 3.15 (×1.79) | 5.05 (×1.53) | 6.72 (×1.52) |

7.5 is ~4 % beyond 9 in demand. The highest daily demand, 6.72 mm/d, is under the watering's 8 mm/day
capacity (and the sunlit chambers never pass LAI ~1, where it is lower still — the price's §4 correction).

## 6. Predictions (written before any code)

| | prediction | confidence |
|---|---|---|
| A1 | commit (A) is inert: `regen_goldens` 20 of 20 identical; the biosphere manifest's only change is `transpiration.yaml`'s digest; station and authoring manifests unchanged | high |
| B1 | commit (B): `regen_goldens` rewrites exactly the price's nine — `sealed_chamber`, `perennial_chamber`, `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon` (biosphere) and `greenhouse`, `harvest`, `lighting`, `sealed_station` (station); `season_euler`, `drift_summary`, `sealed_energy_drift_summary` and the eight crop-free goldens identical | high |
| B2 | **water only**: no carbon, nitrogen, oxygen, energy, `thermal_time`, `rooted_depth`, `vernalization_days` or `soil_shade_lai` leaf moves in any of the nine | medium-high (held bit for bit at 9; 7.5 is 4 % more demand, still under capacity) |
| B3 | the three one-week runs move a little further than at 9 (`lighting` soil ~−0.18 %, condensate ~+0.42 %); the multi-year end states are snapshots and may swing either way — not predicted in sign | medium |
| B4 | root-zone outflow over the run (a probe, not a golden): `sealed_station` above the price's +77 % at 9 (~+80–90 %); the sunlit chambers +4 to +25 % | medium |
| B5 | reds before the goldens are regenerated: `golden_regression` (domains, station), the `tier_contract` checks; after regenerating and before the manifests: the manifest byte gates (domains, station) | high |
| B6 | the re-sow cycle test (`the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years`) goes red as at 9 (settles in two cycles); measured and restated, not widened | medium |
| B7 | the canopy-resistance instrument stays green, because it reads the selector (at 9 it went red only by reading the file's 50 against an overridden flow) | high |
| B8 | green: the TM scorecard (open build), `drought_window`, every carbon / energy / gas test, the conservation and determinism laws | high |

## 7. What "water only" cannot see

The model gives evaporation no heat — no station node and not `thermal.rs` carries latent heat — and the
dehumidifier draws no power for what it condenses. A real chamber transpiring ×1.5–1.8 more would cool its
leaves and its room and load its dehumidifier. So the energy side of this change is **unmeasured, not
zero**. **Measured 2026-10-15** (`post-roadmap-chamber-latent-heat.md` §5): the crop transpires at night with no heat to pay for it — ≥ 1.80 GJ over the run at 7.5, 0.67 GJ at 50 — so the change made an existing gap 2.7× larger rather than creating one.

---

## 8. Commit (A), measured (2026-10-15)

**A1 HELD.** `regen_goldens`: 20 of 20 identical. The biosphere manifest's only change is
`transpiration.yaml`'s digest; the station and authoring manifests did not move. One red, not predicted:
the claim census (`claim_census.rs`) requires every new test in `params.rs` to be declared, and pins the
declared count — one `A` row added, 83 → 84. Clippy clean. Commit `4f6d7dc`.

## 9. Commit (B), measured (2026-10-15) — graded

**B1 HELD.** `regen_goldens` report: exactly the nine predicted goldens change, 11 identical
(`season_euler`, `drift_summary`, `sealed_energy_drift_summary` among them).

**B2 HELD — water only, bit for bit.** Every value that differs in the nine is a water stock (soil,
subsoil, condensate) or the soil account's water aux (`top_soil_water`, `soil_evap_today`,
`soil_dry_days`). Script `W:\temp\claude\ra-adopt\diff.py`, output `diff.txt`.

| golden | soil | condensate | subsoil |
|---|---|---|---|
| `lighting` (7 days) | −0.171 % | +0.411 % | — |
| `greenhouse` (7 days) | −0.144 % | +0.303 % | — |
| `harvest` | −0.013 % | +0.192 % | — |
| `sealed_chamber` (3 yr) | +6.9 % | −11.9 % | — |
| `perennial_chamber` (5 yr) | +26.8 % | −24.4 % | +0.03 % |
| `perennial_long_horizon` (15 yr) | +28.0 % | −36.2 % | +1.4 % |
| `consumer_chamber` (5 yr) | −2.3 % | +4.3 % | −0.03 % |
| `consumer_long_horizon` (15 yr) | +27.2 % | −23.9 % | +0.01 % |
| `sealed_station` (4 yr) | −21.7 % | +21.5 % | +32.7 % |

**B3 HELD.** `lighting` −0.171 % soil / +0.411 % condensate against ~−0.18 / ~+0.42. The multi-year end
states swing in sign, as predicted they might: they are readings inside a watering cycle (next table).

**B4 — held for the station, the sunlit range exceeded by one chamber.** The price's probe
(`zz_ra_probe.rs`, run as an uncommitted example, then removed). Its before-run reproduces the price's
unset row to every printed digit. Root-zone outflow and run means, change against before:

| build | outflow | mean soil | mean subsoil | mean condensate |
|---|---|---|---|---|
| `sealed_station` (lamp, 4 yr) | **+86.7 %** (1860.6 → 3473.5 kg) | +0.4 % | +9.9 % | −15.1 % |
| `sealed_chamber` (3 yr, never re-sown) | **+26.7 %** | −12.1 % | +0.2 % | +26.7 % |
| `perennial_chamber` (5 yr) | +17.2 % | +1.6 % | −2.0 % | −1.2 % |
| `consumer_chamber` (5 yr) | +9.8 % | +2.0 % | +0.5 % | −3.2 % |

Predicted: the station ~+80–90 % (held); the sunlit chambers +4 to +25 % — `sealed_chamber`'s +26.7 % is
over the band, the hand estimate ran low.

⚠ **Not predicted, and against the price's reading:** the price measured the sealed station's mean
condensate falling **−8.5 / −16.3 / −39.2 %** at 30 / 15 / 9 and wrote it as a drain "growing with
`1/ra`". At 7.5 it is **−15.1 %** (subsoil +9.9 % against +24.7 % at 9). So the drain is not monotone in
the value. Whether the branch device and the adopted shape agree byte for byte at 7.5 is checked next
(§9a), so that this is read as the model's behaviour, not a difference between the two builds. Cause not
established.

### 9a. The two builds agree byte for byte (2026-10-15)

The price's branch (`wip/chamber-ra-price`, its environment hook set to `sealed` / 7.5), run in a temporary
worktree with its own target directory: `regen_goldens --write` rewrote 9, and `cmp` finds **all 21 golden
files byte-identical** to the adopted shape's. So the adopted selector is the priced change, and the
non-monotone condensate drain in §9 is the model's own behaviour at 7.5, not a difference between builds.

### 9b. The suite at 7.5 — B5–B8 graded (2026-10-15)

Full debug suite `--no-fail-fast` after the goldens were written, before the manifests: **3 red**, 85
binaries run. `--release -- --ignored`: green (the slow golden and band tests read the regenerated files).

**B5 HELD.** The two manifest byte gates (domains, station) red until regenerated. `--write-manifest`
then moved exactly the 5 biosphere and 4 station `golden_sha256` rows; authoring unchanged. (The
`golden_regression` and `tier_contract` reds did not appear because the goldens were rewritten before the
run.)

**B6 HELD, and a second red it hid.** `the_resow_makes_a_cycle_and_not_a_ratchet_over_five_years`: the
below-root store at the cycle starts reads 167.9026, 167.57357515255570, 167.57357515193095,
167.5735751519298 kg. It settles in two cycles: the second step is 3.7e-12 relative (above the test's
1e-12), the third 7e-15. Measured at five values over the same run:

| `r_a` | first step | second step | second / first |
|---|---|---|---|
| 50 | −1.20e-3 | −1.12e-13 | 9e-11 |
| 30 | −1.47e-3 | −2.17e-13 | 1.5e-10 |
| 15 | −1.77e-3 | −8.66e-13 | 4.9e-10 |
| 9 | −1.92e-3 | −2.83e-12 | 1.5e-9 |
| 7.5 | −1.96e-3 | −3.73e-12 | 1.9e-9 |

Restated, not widened: the fixed point is held to 1e-12 from the third cycle; a new check holds the second
step below 1e-8 of the first (measured 1.9e-9 — a ratchet steps by about the same amount every cycle); the
transient's 4e-3 bound is unchanged (measured 1.96e-3). The run is one year longer (six, `+ 1` on the
constant) so the fixed-point check still compares three cycles, as it did. The test keeps its name: the
census and five plans cite it.

**Not predicted: the soil total's check, hidden behind the first.** With the first check restated, the
test's "the two soil stores together are conserved" check went red: 186.60944615875857, …763, …664, …547 —
−1.66e-14 by cycle 4 against a flat 1e-14. Measured over 12 cycles (a probe test, removed): the
**chamber's full water total** (every stock by its water composition, 195 kg) falls in a straight line too,
**−5.8e-15 a cycle at 7.5 and −3.8e-15 at 50** — a rounding lean in the water path, the same sign every
cycle, present before this change and only faster with it. At 50 the flat 1e-14 would have been reached in
cycle 5, one past the test's horizon. Put to the user in plain words; **the user chose "allow a stated
rounding margin"**. The check now carries two claims at a budget of 2e-14 a cycle (~3× the faster measured
lean): NO LEAK on the full water total, and the soil pair RETURNS to the same total at each re-sow. The
lean itself is **not hunted — an open item**. The soil pair was never a closed set (vapour and condensate
hold water too), so its old "conserved" wording claimed more than it could.

⚠ **Correction to the price** (`post-roadmap-chamber-air-resistance.md` §7b): at 9 the same soil-total
check would have failed too (the probe: −1.83e-14 by cycle 4; at 15, −1.74e-14), unseen because the test
stopped at its first failing check.

**B7 HELD.** `canopy_resistance` green (it reads the selector). **B8 HELD.** Green: the TM scorecard (open
build, before §10's change), `drought_window`, every carbon / energy / gas test, the laws.

## 10. The TM 102788 scorecard opts in (the user, 2026-10-15)

A side note the user forwarded pointed out that §4's "stays out" left the very comparison this item was
picked for at the old 50. Asked, the user chose **"use 7.5"** (over keeping 50 or printing both): the
trial was a chamber, so the scored row reads the chamber value.

**The shape.** The scorecard hosts the trial on an OPEN build, so the scenario's `sealed` flag cannot carry
it. The scored row (`scorecard`) and its own instrument checks (non-limiting water and nitrogen, the books
see the night, the bit-identical rerun) build from the frozen params with the open field's `r_a` set to
`aerodynamic_resistance_for(true)` — the adopted chamber value, not a what-if. The §10–§13 instruments keep
running at the open field's 50, as their recorded numbers were taken; each prints the value it used.

**Predictions (before the code).**

| | prediction | confidence |
|---|---|---|
| T1 | W1 4.25–4.35 L m⁻² d⁻¹, **ratio ~1.02–1.04** of the 4.17 level (§13g's by-hand 4.303 at 7.5 light-only; the model sat 0.15 % from its by-hand at 50) | medium |
| T2 | U1, N1, N2, U2, C1, B1 and the end crop **bit-identical** to the row at 50: the crop is never water-limited, so no carbon path reads the water | high |
| T3 | the conditions check holds at 7.5: lowest FTSW stays above `wssg` | medium-high |
| T4 | W2 (whole-run mean) rises by about the W1 factor or a little less (the seedling weeks transpire little) | medium |

**Graded (2026-10-15)** — `scorecard_tm102788 -- --nocapture`, release:

| | at 50 | at 7.5 | TM | ratio | prediction |
|---|---|---|---|---|---|
| W1, TM days 25–80 (L m⁻² d⁻¹) | 2.6027 | **4.3409** | 4.17 | **1.041** (0.833 on the 16 m² of trays) | T1 **HELD** (4.25–4.35; ratio at the band's edge) |
| U1, N1, N2, U2, C1, B1, end crop | — | bit-identical | | unchanged | T2 **HELD**, now asserted |
| lowest FTSW | 0.4486 | 0.4461 | | above `wssg` | T3 **HELD** |
| W2, whole run | 2.7352 | **4.2549** | 4.50 | **0.946** | T4 **HELD** (×1.556 against W1's ×1.668) |

The model's W1 sits 0.9 % above §13g's light-only by-hand 4.303 at 7.5 (at 50 it sat 0.15 % below its own
by-hand) — the soil–crop energy split and the leaf-area resistance, not chased. T2 is now an assertion,
`the_chamber_air_moves_the_rows_water_and_nothing_else` (every CO₂ step and the end crop bit-identical to
the row at 50; W1 must rise). The row's water ratio goes **0.62 → 1.04**: at the measured chamber air
resistance the gap this item was picked for closes, with no lamp heat added. ⚠ That is the source's
best-case coupling (§2: its fan ran faster than the BPC's), so "closes" is an upper reading, not a match;
and a closed level is not a validated mechanism — the shape (P8) was not re-read.

## 11. Done (2026-10-15)

* Every sealed chamber reads 7.5 s m⁻¹; the open field keeps 50. 9 goldens moved, water only.
* The TM scored row reads 7.5: water ratio 0.62 → 1.04, nothing else moves.
* The energy side is **unmeasured, not zero** (§7).
* Open: the water path's rounding lean (§9b); the sealed station's non-monotone condensate drain (§9).
* **Every reader checked (advisor, 2026-10-15):** the one production builder of `Transpiration`
  (`system.rs`) takes the selector; every other direct read of `aerodynamic_resistance` is an open-field
  context (`DEFAULT_SCENARIO`, the scorecard's §10–§13 instruments, `soil_evaporation`'s S3), a
  self-consistent flow unit fixture, a param pin, or the frozen C1 control table. The only what-if on the
  key (the scorecard's R1) runs on the open build, so no what-if has gone silently inert.
* Evidence cited as `W:\temp\claude\ra-adopt\` (the golden diff script and output, before/after golden
  copies, the throughput probe, suite and scorecard logs) now lives in `sources\temp-evidence\ra-adopt\`
  (local, not in git), by the repo's convention for cited temp paths.
