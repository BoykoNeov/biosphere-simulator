# Adopting the live-O₂ FvCB form — the band re-posed, in writing, before any run

**The decision was taken by the user 2026-09-06** ("adopt the live-oxygen form"), against the
third direction plan's §2.3, which prices adoption and states its one precondition:

> ⚠ **The price is not the number, it is the guard.** The `min > Γ*/ci_ratio` band is written
> against a **constant** floor; a live Γ\* turns it into a pointwise claim — a different
> assertion, not a re-tuned one — and at the ratios this form produces (12 at charge, 685 at
> the end) it stops discriminating. **Adopting means re-posing that band first**, in writing,
> before a run.

This document is that re-posing. **Nothing has been run for it and nothing has been changed.**
Its whole value is that it is written before the flip, so the predicted red set below can fail
against the measurement rather than be assembled from it. The project has refused three
attempts to re-tune this family of bounds to fit a result (`log/canopy-magnitude.md`); writing
the prediction first is the only mechanical difference between re-posing and re-tuning.

Filed under the third direction plan. Exempt from the log's index while nothing is built —
see the exemption paragraph in `docs/post-roadmap-log.md`, **committed with this doc**, which
expires the moment the first slice lands.

---

⚠⚠ **MEASURED 2026-09-06, AFTER this document was committed — and it REFUTES two of its own
three findings.** §8 is the outcome and it is the part to read. Everything above it is left
**exactly as written before the run**, banners aside, because a prediction edited after the
fact is not a prediction. In short: §0's station finding **held and got much larger**; §4's
headline (*"the band cannot be re-posed into anything that binds"*) is **WRONG** — the
pointwise band's true minimum is ×10.67, not ×685; and §5's concern about `ci_ratio` has
**the wrong sign**. Two of my own conclusions did not survive the measurement they told me
they were owed.

---

## 0. ⚠ HEADLINE — the scope claim adoption was priced under is FALSE, and the correction is
## a station-side one

The third direction plan's §2.3 prices adoption as *"only `sealed_chamber` — the jar"*, with
*"the two big chambers move +0.140 % / +0.118 %"* and the open field bit-identical. **That is
a true statement about the biosphere lab's seven-scenario roster and a false statement about
the tree.** It was measured through `lab::report::SPECS`, which enumerates biosphere scenarios
only; the station's sealed assemblies were never in the instrument.

Enumerated from **disk** rather than from the record (`log/coverage-roster-is-not-the-manifest.md`:
*enumerate from disk*), by reading `biosphere.o2_pool` out of every golden and dividing by the
scenario's own `chamber_air_mol`:

| golden | O₂ at the golden's end | air (mol) | mmol/mol | frozen constant reads | predicted move |
|---|---|---|---|---|---|
| `sealed_chamber_state` | 0.033186 | 1000 | **0.033** | 210 | **huge** — measured, −89.8 % on the band row |
| `greenhouse_state` | 8.102000 | 9500 | **0.853** | 210 | **huge — NEVER MEASURED** |
| `harvest_state` | 8.102000 | 9500 | **0.853** | 210 | **huge — NEVER MEASURED** |
| `sealed_station_state` | 8.102000 | 9500 | **0.853** | 210 | **huge — NEVER MEASURED** |
| `perennial_chamber_state` | 210.2412 | 1000 | 210.2 | 210 | ~0.1 % (a control) |
| `perennial_long_horizon_state` | 210.2409 | 1000 | 210.2 | 210 | ~0.1 % (a control) |
| `consumer_chamber_state` | 420.4600 | 2000 | 210.2 | 210 | ~0.1 % (a control) |
| `consumer_long_horizon_state` | 420.4661 | 2000 | 210.2 | 210 | ~0.1 % (a control) |
| `lighting_state` | 210.1316 | 1000 | 210.1 | 210 | ~0.1 % (a control) |
| `drift_summary.json` | folds the two perennial/consumer runs | — | — | small, **non-zero** |
| `sealed_energy_drift_summary.json` | folds the sealed station | — | — | unknown, **non-zero** |
| `season_euler_state` | no O₂ pool | — | — | **bit-identical by construction** |

⚠ **The three station rows are charged LOWER than the jar is.** `greenhouse_bio_scenario`
puts `chamber_o2_mol0 = 10.0` into `chamber_air_mol = 9500.0` — **1.05 mmol/mol at charge**,
against the jar's 2.0. `sealed_station_bio_scenario` and the harvest seam inherit it. So the
three assemblies the station contract exists to freeze sit *deeper* in the regime this form
was built for than the one scenario the pricing named, and **no column has ever been measured
on them under `LivePool`** — the lab's oxygen-form harness (`lab::biosphere_with_o2_form`)
builds biosphere params and measures biosphere specs; it cannot reach a station assembly.

⚠ **And that charge is very likely a scenario-authoring artifact rather than a design.** A
cabin-sized air volume (9500 mol) carrying a chamber-sized oxygen fill (10 mol) is 0.4 % of a
breathable atmosphere. It was invisible for four phases because oxygen did not enter
photosynthesis; the frozen constant read 210 for it regardless. **Adoption makes that number
load-bearing science.** This is `log/o2-makeup-reversal-inside-the-freeze.md`'s shape exactly:
*a scope claim is dated to the roster that existed when it was written.*

**Consequence for this document:** §5 cannot be finished from the shelf. The station's three
sealed assemblies need a measurement before adoption is defensible, and §7 says what that
means for the decision.

---

## 1. What "adopting" is, mechanically

One line: `rust/crates/domains/src/biosphere/params.rs:473`, `o2_form: O2Form::Constant` →
`O2Form::LivePool`. Everything else in this document is consequence.

The form itself is already built, reviewed and gated (`log/o2-form-built.md`,
`docs/plans/post-roadmap-o2-form.md`). **Nothing about the science changes here.** What
changes is which of two built forms the reference selects, and therefore which numbers are
frozen.

---

## 2. The predicted red set — written before the run

⚠ **This is the list the run must reproduce.** An extra red is a finding; a missing red means
the flip did not reach what I think it reaches, which is the silent-baseline failure
`tests/o2_form.rs`'s own header names. Neither is licence to edit this list afterwards.

### 2a. Exactly one frozen science gate reddens

`band_gate` compares `folds::min_ppm(t)` against `folds::floor_ppm()`, and `floor_ppm` reads
`p.photo.gamma_star / ci_ratio` off the **loaded params object**. The substitution happens per
step *inside the flow*, so the loaded object still carries `gamma_star = 42.75` after the flip
and the floor still computes to **61.071429 ppm**. Against the measured `LivePool` minima
(`log/o2-form-built.md`'s finding-1 table):

| gate | min ppm under `LivePool` | vs 61.071429 | verdict |
|---|---|---|---|
| `sealed_chamber_stays_above_the_compensation_point` | 7.294541 | 0.119 × | **RED** |
| `perennial_chamber_stays_above_the_compensation_point` | 70.351292 | 1.152 × | green |
| `consumer_chamber_stays_above_the_compensation_point` | 73.425099 | 1.202 × | green |
| `perennial_long_horizon_stays_above_the_compensation_point` | ~70.35 | ~1.152 × | green |
| `consumer_long_horizon_stays_above_the_compensation_point` | ~73.43 | ~1.202 × | green |

**One of five.** And the one that reddens is the only one where the frozen floor is
scientifically wrong for the run it is applied to.

### 2b. The margin pin reddens on the same single row

`the_five_margins_are_where_the_pin_says` holds `PINNED` at a 2 % tolerance:

| scenario | pinned | predicted | move |
|---|---|---|---|
| `sealed_chamber` | 1.169709 | **0.119443** | **−89.8 % → RED** |
| `perennial_chamber` | 1.150335 | 1.151951 | +0.14 %, inside 2 % |
| `consumer_chamber` | 1.200866 | 1.202283 | +0.12 %, inside 2 % |
| `perennial_long_horizon` | 1.150335 | ~1.151951 | inside 2 % |
| `consumer_long_horizon` | 1.200866 | ~1.202283 | inside 2 % |

⚠ **The two controls staying inside a tolerance that was measured to fire** (the within-day
light path moved three of five past it) is the strongest available evidence that the flip is
scenario-shaped rather than global. If a control moves past 2 %, the form is wired wrong and
adoption stops.

### 2c. The lab's own identity control reddens, correctly

`tests/o2_form.rs::the_constant_form_is_the_frozen_run_on_every_measured_quantity` asserts
both `params::biosphere().photo.o2_form == O2Form::Constant` and that the loader's column
equals the forced-`Constant` column. Both halves invert on the flip. **This is the tripwire
the build asked for, firing exactly as designed** — it is re-posed, never deleted.

⚠ `the_sealed_chamber_moves_under_the_live_form` stays **green** and its meaning inverts: it
now compares the reference against a lab counterfactual rather than a lab alternative against
the reference. It still guards the same failure (a `LivePool` column that silently reads the
baseline back), so the assertion is right and the prose around it is not.

### 2d. The manifest reddens — and this is the one automatic gate

**Verified, not assumed:** `docs/biosphere-reference.manifest.json` carries the bound string
`"min > Γ*/ci_ratio (61.07 ppm)"` **five times**, under `science_bands`. So re-posing the
five bounds moves the manifest, and `rust/crates/domains/tests/manifest_writer.rs` compares
the committed file byte for byte. **This is not the WSFD counter-example**: unlike a pure form
change, this one cannot land half-done in silence.

⚠ The station manifest carries **no** copy of that string — checked. The station's two science
gates are the crew respiratory quotient and the sealed thermal fixed point, neither of which
is a CO₂ band. **So the station side has no automatic guard on this at all**, which is the
other half of §0's problem.

### 2e. Eleven of twenty-one goldens move

The nine `bio=Y` state files plus the two drift summaries from §0's table. `season_euler_state`
is bit-identical by construction (no O₂ pool). The ten non-biosphere goldens are untouched.

**Predicted `regen_goldens` report: 20 of 20 run, 11 would change.**

⚠ **MEASURED: 20 of 20 run, 10 would change** — see §8b. One over-prediction
(`sealed_energy_drift_summary.json`), and the 21st golden is **not** in the changed set.

⚠ `regen_goldens` covers 20; `rust/data/golden/` holds 21. That surplus is the known
`coverage-roster-is-not-the-manifest` gap, and it must be re-checked here rather than
assumed harmless: if the 21st is one of the eleven, it has no regeneration path in the tool
and adoption inherits `drift_summary`'s old defect — *a frozen golden no path can regenerate.*

### 2f. Two tests stay green and become STALE, which is worse than red

* `the_floor_is_where_the_frozen_params_put_it` pins `folds::floor_ppm() ≈ 61.07`. It reads
  the params object, which does not move, so it **stays green while pinning a number the
  reference no longer uses**.
* `the_shipped_floor_is_the_conservative_one_against_the_cited_route` compares Teh's
  `O₂/(2τ)` route against the same constant floor — and it computes Teh's oxygen from
  `photo.o2`, the **constant**, so it too stays green on an argument that no longer describes
  the reference.

Both must be re-posed with the band. A green test asserting a retired claim is the exact shape
`log/science-gate-census-in-rust.md` records: *a DELETED claim leaves the suite green.*

---

## 3. The band, re-posed

**The claim the five gates make today:**

> the season-low chamber CO₂ stays above `Γ*/ci_ratio`, where `Γ*` is the frozen 42.75 µmol/mol
> and `ci_ratio` the C3 set point — a single number, 61.071429 ppm, for every scenario and
> every instant.

**The claim they must make under adoption:**

> at **every step**, the chamber's CO₂ stays above that step's own compensation point:
> `CO₂(t) > Γ*(t)/ci_ratio` with `Γ*(t) = 42.75 · O₂(t)/210`.

This is a **different assertion, not a re-tuned one**, and the difference is worth naming
precisely, because it is what distinguishes this from the three refused re-tunings:

* the old bound compares one fold of the run against one constant;
* the new bound compares two series pointwise, and cannot be satisfied by a scenario merely
  being large;
* nothing in it was chosen to make any scenario pass. It is the same physics the old bound
  states, evaluated at the oxygen the run actually has instead of at the oxygen the atmosphere
  has.

⚠ **The old bound is a special case of the new one** — at constant O₂ they are identical — so
the four scenarios where the constant is right to 0.1 % keep, to within that, the bound they
already had. That is the argument for the re-posing being a restatement rather than a
weakening, **and it is exactly the argument that fails on the jar**, which is §4.

### 3a. A mechanical constraint on how the new bound is SPELLED

`check_bound_literals` requires every numeric literal in a `bound:` string to appear in
**executable** text at the gate's locus (comments stripped since C4b), and separately refuses
a bound with no number at all:

```rust
assert!(!literals.is_empty(), "a bound with no number is not a bound: {gate:?}");
```

So `"min > Γ*(t)/ci_ratio, pointwise"` **cannot be the bound string** — it would redden that
check. Whatever number the re-posed band carries must be a real, defensible threshold that
appears in executable code in `science_gates.rs`, exactly as `61.07` does today via
`the_floor_is_where_the_frozen_params_put_it`. **The gate machinery forbids re-posing this
band as an unquantified claim**, which is a constraint working in the right direction and is
why §4 cannot be skipped.

---

## 4. ⚠ REFUTED BY §8a — does the re-posed band still bind?

⚠⚠ **This section's conclusion is WRONG and the measurement it says it is owed is what
refuted it.** The pointwise band's true minimum on the jar is **×10.674948**, not ×685: the
×685 figure is the ratio at the run's END, and the binding instant is nowhere near the end.
The section is kept verbatim as the record of a conclusion drawn from a number taken at the
wrong instant. Read §8a instead.


A re-posed bound that cannot fail is not a bound. `log/co2-guard-reanchored.md` is the
precedent and it is unambiguous: *the window was measured inert and removed, not resized until
the subject passed.*

| scenario | O₂ regime | pointwise floor | headroom | binds? |
|---|---|---|---|---|
| `perennial_chamber` | 210.2 mmol/mol, flat | ≈ 61.13 ppm | ×1.151 | **yes** — essentially the bound it has today |
| `consumer_chamber` | 210.2 mmol/mol, flat | ≈ 61.13 ppm | ×1.202 | **yes** |
| the two long horizons | same | ≈ 61.13 ppm | same | **yes** |
| `sealed_chamber` (the jar) | 2.0 → 0.033 mmol/mol | 0.582 → **0.0097 ppm** | **×12 at charge, ×685 at the end** | **NO** |

The four controls keep a real bound. **The jar does not.** A guard that passes by a factor of
685 is not measuring anything about the jar; it is measuring that a positive number exceeds a
number approaching zero. Under `log/co2-guard-reanchored.md`'s own rule, the honest options
for the jar's row are **remove it** or **replace it with a bound that can fail** — never
"keep it, it passes".

⚠ **The ×12 / ×685 figures are the third direction plan's, and they are the ratio at the
CHARGE and the END, not at the minimum-CO₂ instant.** The true quantity — `min over t of
(CO₂(t) − Γ*(t)/ci_ratio)` — has never been measured. It must be, before this section is
final. I expect it to change nothing qualitative (both series fall together, and the jar's
oxygen falls faster than its CO₂), but that is a prediction, not a result, and it is written
here so it can fail.

### 4a. Candidate successor bounds for the jar, and why each is unsatisfying

1. **The pointwise band anyway, accepting it is loose.** Rejected — it is the "keep it, it
   passes" option, refused above.
2. **A depletion contract**: the jar's O₂ pool bottoms at ≥ 95 % depletion (the scenario's
   stated purpose, and the reason `litter_carbon0` was re-sized 3.0 → 3.5 in 2026-08-12).
   This is a real, failable claim about the jar — but it is a claim about **oxygen**, and it
   would leave the scenario with **no carbon band at all**, which is a coverage loss the
   manifest's completeness gate would happily accept.
3. **A ratio bound on the jar's CO₂ against its own charge** — fitted to 7.29 by construction.
   Refused: that is the co-adaptation shape.
4. **A validity-range bound on `ci_ratio`** — see §5. This is the honest one, and it is not a
   band on this scenario at all.

**I do not have a defensible successor bound for the jar's carbon row.** That is stated as a
finding rather than papered over, per the plan-of-record's own instruction for exactly this
outcome.

---

## 5. ⚠ REFUTED BY §8c — the assumption that inherits the load

⚠⚠ **The sign is backwards.** A fixed `Ci/Ca` **understates** the leaf's internal CO₂ at low
ambient CO₂, not overstates it, so the assumption makes the model **conservative** exactly
where I claimed it made it unsafe. Kept verbatim; read §8c.


The frozen 61.07 floor was doing a second job nobody had written down. **It was the only thing
standing between the model and a regime its other assumptions do not cover.**

Under adoption the jar keeps fixing carbon down to **7 ppm ambient CO₂**, because Γ\* has
collapsed with the oxygen. The oxygen chemistry in that statement is right — it is the whole
point of the form. But the model only *reaches* 7 ppm believing the leaf still has carbon to
work with, and it believes that because `ci_ratio = 0.7` is a **fixed constant**: the leaf's
internal CO₂ is always 70 % of the outside air, whatever the outside air is.

A real C3 leaf at 7 ppm ambient assimilates almost nothing regardless of Γ\*, because the
diffusion gradient driving CO₂ into the leaf has gone. FvCB with a fixed Ci/Ca set point
**cannot represent that**, and the constant floor was — accidentally — keeping the model out
of the range where the omission matters.

**So removing the floor's protection does not validate the run down to 7 ppm. It relocates the
unvalidated assumption from `Γ*` to `ci_ratio`.** The band was hiding it; adoption exposes it,
and exposing it is *better* — but only if it is then stated, not stepped over.

The same argument applies with more force to the three station assemblies in §0, which sit at
0.85 mmol/mol O₂ and have never been looked at at all.

⚠ Note what this is **not**: it is not an argument against the oxygen form, which is better
science than the constant it replaces on every scenario. It is an argument that adopting it as
the **reference** moves three or four frozen runs into a regime where a *different* frozen
assumption is out of range, and that this was not part of what adoption was priced at.

---

## 6. If adoption proceeds — the ceremony, in order

Recorded so it is not re-derived, and so that a decision to proceed does not also become a
decision to improvise. Steps 1–2 of `docs/biosphere-reference.md`'s unfreeze discipline are
this document and its review.

1. **Measure the station's three sealed assemblies first** (§0). Without this, adoption
   freezes numbers nobody has looked at. This is the one step that is not optional under any
   reading.
2. **Measure `min over t of (CO₂(t) − Γ*(t)/ci_ratio)` on all five banded scenarios** (§4's
   owed measurement), and check the prediction.
3. Flip `params.rs:473`. `git diff rust/crates/simcore/` **must stay empty**.
4. Run the suite `--no-fail-fast`; **check the red set against §2** before reading anything
   else. ⚠ `godot_bridge::cross_boundary` carries an unreproduced red at 595 s against a
   600 s limit and will rebuild again; if it fails the same way it is the same timeout
   question and evidence about nothing — but it must not be allowed to hide a real red.
5. Re-pose the five bounds and the two stale-green tests (§2f, §3a). ⚠ Re-pose, do not delete.
6. `regen_goldens` **report-only**, check "11 would change" against §2e, then `--write`, then
   `git diff -- rust/data/golden/` as the record.
7. Regenerate the biosphere manifest with a **predicted** diff (C4 wrote corrupted prose into
   this contract with every gate green; the prediction is what caught it).
8. The comment census — the flip falsifies at least: `params.rs:169` (*"the loader always sets
   `Constant`… so the goldens cannot move"*), `science.rs:179` (*"a LAB alternative and
   endorses nothing"*), `O2Form::Constant`'s own docstring (*"the frozen reference"*),
   `system.rs:338-339` (*"inert under the frozen `Constant`"*), `lab/mod.rs:139` (*"`Constant`
   is the reference and stays the reference"*), `flows.rs:88-92`, and `report.rs:315`'s
   `floor_ppm: None` reasoning, which is right for a lab alternative and wrong for the
   reference — the reference would print `n/a` for its own floor.
9. The normal three (index line, pointer row, `docs/log/o2-form-adopted.md`) + a memory file,
   **and delete this doc's exemption paragraph in the same commit**.
10. Re-read the third direction plan against the new last record row, strike §2.3 in place
    naming the record, move the `Re-read against the record's last row:` marker.

---

## 7. Recommendation — STOP and report, do not proceed on my own judgement

The user authorised **adoption with the band re-posed**. What §4 and §5 found is that the band
cannot be re-posed into anything that binds on the one scenario it exists for, and that the
protection it was silently giving is not replaceable from the shelf. That is materially
different from what adoption was priced at in the direction plan (*"one row of one scenario…
the price is not the number, it is the guard"*), and §0 makes it larger again: three station
goldens in the same regime, never measured, with no automatic gate on that side.

`log/root-coupling-refused.md`'s rule governs what I do with that: **my refusal is a
recommendation, not a verdict — report it.** So this document goes back to the user with three
courses, not one:

* **(a) Proceed anyway**, on the reading that better oxygen science is worth a looser jar band
  and a relocated assumption, with §6 run in full and §4a's option 2 (the depletion contract)
  replacing the jar's carbon row. Defensible; it is a decision about what the simulator is
  *for*, which is the user's, not mine.
* **(b) Measure first, decide after** — do §6 steps 1 and 2 only, which change no frozen byte
  and cost one run each, and re-take the decision with the station numbers in hand. **This is
  what I recommend**: the station measurement is owed under every course including (a), it is
  the cheapest thing on this list, and it is the only one that can still change the answer.
* **(c) Leave the form in the lab**, on the reading that a reference should not freeze runs in
  a regime where `ci_ratio` is out of range. Costs nothing and loses nothing: the form stays
  reachable as `science_switch -- o2form=live_pool`, exactly as the Q10 form does.

⚠ **None of the three is "adopt and quietly widen the band."** That option is closed by this
document existing.

---

## 8. MEASURED — what the runs said, 2026-09-06

Everything here was produced **after** the sections above were committed, by a scratch harness
that has since been deleted and a loader flip that was reverted: `git status` is clean and
**no frozen byte moved.** The harness added one series to `Trajectory` (`biosphere.o2_pool`,
which the pointwise band needs and which nothing sampled), flipped `params.rs:473` in the
working tree, ran, and was reverted in full.

⚠ **The harness validated itself against the record before anything new was read.** Under
`O2Form::Constant` it reproduced the frozen minima exactly — 71.435803 / 70.252606 /
73.338613 — and under `LivePool` it reproduced `log/o2-form-built.md`'s finding-1 column
exactly — 7.294541 / 70.351292 / 73.425099. Both columns to six decimals, so the numbers
below are this tree's, not a re-derivation that happens to look similar.

### 8a. ⚠ §4 is REFUTED — the pointwise band binds, at ×10.67

`min over t of CO₂(t)/(Γ*(t)/ci_ratio)` on the five banded scenarios, under `LivePool`:

| scenario | min ratio | at step | CO₂ there | live floor there | x_O₂ there |
|---|---|---|---|---|---|
| `sealed_chamber` | **×10.674948** | 779 | 7.294541 | 0.683333 | 2.349705 |
| `perennial_chamber` | ×1.150381 | 1999 | 70.351292 | 61.154791 | 210.286649 |
| `consumer_chamber` | ×1.200661 | 2043 | 73.425099 | 61.153897 | 210.283575 |
| `perennial_long_horizon` | ×1.150381 | 1999 | 70.351292 | 61.154791 | 210.286649 |
| `consumer_long_horizon` | ×1.200661 | 2043 | 73.425099 | 61.153897 | 210.283575 |

**×10.67, not ×685.** And the reason is a piece of physics §4 should have seen: in a sealed
chamber the two series are **anticorrelated** — photosynthetic quotient 1, so every mole of
CO₂ fixed releases a mole of O₂. The jar's oxygen is at its **highest** (2.349705 mmol/mol,
*above* its 2.0 charge) exactly when its CO₂ is at its lowest. So the live floor is at its
highest at the moment the band is evaluated.

⚠ **That inverts §4's argument rather than softening it.** A pointwise band on a sealed
chamber is not loose by construction — it is evaluated at the physically hardest instant the
run contains, automatically, which is more than the constant floor ever did. The ×685 figure
is the ratio at the run's **end**, hundreds of steps after the binding instant, when CO₂ has
recovered on respiration and the oxygen has drained to 0.153756. Quoting it as the band's
margin was reading a number at the wrong instant — the same defect as the record's own
*"a record's probe figures are not the tree's"*, one level down.

**So the re-posed band is a real bound on all five scenarios**, ×10.67 on the jar against
×1.15–1.20 on the four controls. Nine times looser, and not vacuous. §4a's search for a
successor bound was a search for a problem that does not exist; option 2 (the depletion
contract) is still worth having as a *companion*, but nothing forces it.

### 8b. §2's predicted red set — 10 of 11, and the surplus golden is out of scope

`regen_goldens` report-only after the flip: **20 of 20 run, 10 would change.** Predicted 11.

* The nine `bio=Y` state files and `drift_summary.json` changed, exactly as predicted.
* **`sealed_energy_drift_summary.json` did NOT change** — the one over-prediction. It folds
  `node_peak_temp_k` and `is_stationary` on the sealed station's thermal side, and the
  biosphere's carbon does not reach either.
* `season_euler_state.json` identical, as predicted by construction.
* ⚠ **§2e's open question is answered: the 21st golden (`state_snapshot.json`) carries no
  biosphere and is not in the changed set**, so adoption does *not* inherit `drift_summary`'s
  old defect of a frozen golden with no regeneration path.

### 8c. ⚠ §5 is REFUTED — the `ci_ratio` concern has the wrong sign

§5 argued that a fixed `Ci/Ca = 0.7` lets the model believe in assimilation a real leaf could
not achieve at 7 ppm ambient, because the diffusion gradient is gone. **The sign is
backwards**, and it follows from the definition rather than needing a run:

`Ci = Ca − A/g`. As `Ca` falls toward the compensation point, `A` falls with it — steeply,
because `A ∝ (Ci − Γ*)` — so the drawdown `A/g` falls **faster than `Ca` does**, and the real
`Ci/Ca` therefore **rises toward 1**. A real leaf at 7 ppm ambient sits at `Ci ≈ Ca`, not at
`0.7·Ca`.

So holding the ratio at 0.7 puts `Ci` **below** where a real leaf would hold it, which
**understates** assimilation and puts the ambient compensation point `Γ*/0.7` **above** the
true one (`Γ*/1.0`). The fixed ratio is the **conservative** assumption in exactly the regime
§5 said it was dangerous in — and that is the same shape
`the_shipped_floor_is_the_conservative_one_against_the_cited_route` already argues about the
floor's other parameterization, which §5 had read without recognising.

⚠ **This is `log/canopy-magnitude-diagnosed.md`'s shape** — *the plan's fix had the wrong sign
and our own docstring said so* — reproduced by me, in a document written to stop exactly this
kind of thing. It is recorded rather than quietly deleted because that is the whole discipline:
**a claim written into a fix is checked by nothing unless someone checks it.**

⚠ It is an **argument**, not a measurement: the model has no conductance term, so `g` is not a
quantity this tree carries and the inequality above cannot be read off a run. It is recorded at
the strength it has.

### 8d. ⚠ §0 HELD, and it is far larger than "huge" — the station's Tier-2 golden moves by two thirds

The one finding that survived, and the measurement made it the decisive one. Largest relative
moves per golden, frozen → adopted:

| golden | stock | frozen | adopted | move |
|---|---|---|---|---|
| `sealed_station_state` | `biosphere.humus_carbon` | 25.823 | 45.1658 | **+74.9 %** |
| `sealed_station_state` | `biosphere.microbial_carbon` | 12.3966 | 21.1391 | **+70.5 %** |
| `sealed_station_state` | `biosphere.storage_c` | 37.9945 | 63.6084 | **+67.4 %** |
| `sealed_station_state` | `biosphere.root_c` | 8.18673 | 13.6531 | **+66.8 %** |
| `sealed_station_state` | `biosphere.stem_c` | 11.2998 | 18.6024 | **+64.6 %** |
| `greenhouse_state` | `biosphere.stem_reserve_c` | 0.00314025 | 0.00595129 | **+89.5 %** |
| `greenhouse_state` | `biosphere.leaf_c` | 0.0820178 | 0.118064 | **+43.9 %** |
| `harvest_state` | `biosphere.storage_c` | 0.00234709 | 0.00403986 | **+72.1 %** |
| `sealed_chamber` (the jar) | `biosphere.o2_pool` | 0.0331859 | 0.153756 | **+363.3 %** |
| `sealed_chamber` (the jar) | `biosphere.leaf_c` | 8.75291e-8 | 4.8331e-8 | **−44.8 %** |
| `lighting_state` (control) | `biosphere.stem_reserve_c` | 0.0052749 | 0.00527374 | −0.022 % |
| `perennial_chamber` (control) | `biosphere.carbon_pool` | 0.115822 | 0.115935 | +0.098 % |

**The controls behave** — 0.02 % and 0.098 % maxima — so the flip is scenario-shaped and the
wiring is right. **And `sealed_station` is the fully-coupled multi-year station**, the Tier-2
golden this whole contract exists to freeze. Its plant and soil carbon rise by **two thirds**.

⚠ **The direction plan priced adoption at "one row of one scenario."** The measurement says it
is a **60–75 % move in the principal stocks of the station's flagship run**, plus a 45 % fall
in the jar's biomass, plus 44–90 % rises in the greenhouse and harvest seams. Nothing about
the form is wrong; what was wrong is that its consequences were measured on an instrument that
could not see the station.

⚠ **And the cause is the greenhouse charge, which is almost certainly an authoring artifact.**
`greenhouse_bio_scenario` puts 10 mol of O₂ into 9500 mol of cabin air — **1.05 mmol/mol,
0.5 % of a breathable atmosphere.** Under the frozen constant the crop read 210 regardless, so
nobody had to notice. Under adoption the crop reads 1.05, its oxygenation nearly vanishes, and
it grows two thirds more. **The 60–75 % is not the oxygen science arriving; it is an
unrealistic oxygen charge becoming visible**, and adopting the form would freeze that number
as reference science.

⚠ **Not yet checked, and it belongs to whoever takes course (a):** whether the cabin's O₂ is
*meant* to be the crew's breathable supply — the ECLSS seam writes `boundary.o2_supply`, and
`biosphere.o2_pool` ends at the same 8.102 mol in both the 28-step greenhouse run and the
4880-step sealed station, which is a fixed point worth understanding before re-charging it.

---

## 9. RECOMMENDATION — revised after §8, and the question for the user has changed

**§7's recommendation is withdrawn.** It rested on §4 and §5, both refuted. Two of its three
courses were built on my own errors and are not the choice any more.

What the measurement leaves is one clean fact and one genuine question.

**The clean fact: adopting the form is scientifically sound and its ceremony is unblocked.**
The band re-poses into a bound that binds on all five scenarios (§8a), evaluated at the
hardest instant each run contains; the `ci_ratio` assumption is conservative rather than
dangerous (§8c); the controls confirm the wiring (§8d); the red set is predictable and the
manifest gate will catch a half-done job (§2d). Nothing scientific stands in the way.

**The genuine question: the station's greenhouse air is not breathable, and adoption is what
makes that matter.** A cabin at 0.5 % oxygen is not a habitat anyone designed; it is a fill
value that never had to be right. Adoption would freeze a 60–75 % change in the station's
flagship golden that is **caused by that fill value**, not by the science being adopted.

So the choice is now about **order**, not about whether:

* **(a) Fix the greenhouse oxygen charge first, then adopt.** Set the station's sealed
  biospheres to a real cabin fraction (≈210 mmol/mol, i.e. ~1995 mol in 9500) — its own small
  unfreeze with its own goldens — and *then* adopt the form, at which point the station
  scenarios become controls that barely move and the jar is genuinely the only thing that
  changes, exactly as adoption was priced. **This is what I recommend.** It separates
  "a fill value was wrong" from "the oxygen science changed", which are two findings that
  should not share one golden diff.
* **(b) Adopt now, and accept the 60–75 % move** as the honest consequence of the scenario as
  authored. Defensible, and cheaper by one unfreeze — but it freezes a large change whose
  cause is a charge nobody defends, and it makes the two causes inseparable in the record.
* **(c) Adopt now and fix the charge afterwards**, in two unfreezes. Worst of both: the same
  large diff lands, and then most of it is undone.

⚠ **Whichever is chosen, the pieces adoption itself needs are now known and cheap**: §6's
ceremony stands with steps 1 and 2 struck (they are done — this section is their result), and
§4a's hunt for a successor bound is dropped.
