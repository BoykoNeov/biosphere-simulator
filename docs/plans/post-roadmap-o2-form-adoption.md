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

⚠ **`6b6129f`'s commit subject — *"the band CANNOT be re-posed into anything that binds on the
jar"* — is FALSE, refuted by §8a.** It is named here because a commit subject cannot be
rewritten once pushed, and `git log --oneline` otherwise shows a confident wrong claim whose
only correction is in the *next* subject. The pointer belongs at the false claim.

⚠ **DECIDED 2026-09-06: the user chose course (B) — fix the setting first, then adopt.**
§11 is slice 1's plan and predictions, written before anything was flipped. Adoption itself
(§6's ceremony, with §8a's re-posed band) is slice 2 and has not been taken.

⚠ **And §9 is superseded by §10**, which is a third refutation — of the recommendation §9
itself made. The oxygen pool the station runs on is **regulated to a setpoint**, so §9(a)'s
"re-charge the scenario" is a no-op a controller undoes. **Read §10 for the courses.**

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

## 9. ⚠ SUPERSEDED BY §10 — RECOMMENDATION revised after §8

⚠⚠ **Its recommended course (a) is a NO-OP** — the station's oxygen pool is held at an ECLSS
setpoint, so re-charging the scenario's initial fill is undone by a controller. Kept verbatim
as the record of a prescription written against a mechanism I had not identified. Read §10.

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

---

## 10. The oxygen pool HAS a writer — and §9(a) is a no-op. What the charge actually is.

§8d flagged the identical `biosphere.o2_pool = 8.102000000000007` across a 28-step greenhouse
run and a 4880-step sealed station as *"a fixed point worth understanding before re-charging
it."* It is understood now, and understanding it **invalidates §9(a) as written.**

### 10a. It is a regulated setpoint, not an untouched fill

`rust/crates/station/src/sealed.rs:190` and `greenhouse.rs:111` re-point the ECLSS at the
biosphere's own pool — *"the greenhouse seam: crew breathes the bio O₂"* — so `O2_POOL` has
three writers: photosynthesis adds to it, crew respiration draws on it, and `O2Makeup`
regulates it. `O2Makeup` is a proportional controller, `S = k·(o2_setpoint − cabin_o2)`, and
it is **two-sided** (that reversal is itself inside the station freeze,
`log/o2-makeup-reversal.md`). Its steady state is `o2_setpoint − consumption/k`, which is why
both runs sit at the same 8.102 whatever their horizon: they are both at the controller's
offset, reached early and held.

⚠ **So §9(a) — "set the station's sealed biospheres to ~1995 mol in 9500" — does nothing.**
The scenario's `chamber_o2_mol0` is an *initial* charge; the regulator pulls it back to its
setpoint within a few hundred steps and would run in reverse to do it. **Recommending it was
prescribing against a mechanism I had not identified.** The advisor caught it on exactly that
ground before it reached the user.

### 10b. What the charge actually is — two coherent conventions that were never reconciled

`eclss.yaml`'s `o2_setpoint` is **10.0 mol**, and `greenhouse_bio_scenario` sets
`chamber_o2_mol0 = 10.0` to match it. Both are self-consistent. The problem is the *other*
number in the same scenario:

| quantity | charge | ÷ `chamber_air_mol = 9500` | reads as | realistic? |
|---|---|---|---|---|
| CO₂ | 3.796 mol | 3.9958e-4 | **399.6 ppm** | **yes** — a real cabin |
| O₂ | 10.0 mol | 1.0526e-3 | **1.05 mmol/mol** | **no** — 0.5 % of breathable |

**The CO₂ charge was set against the air inventory; the O₂ charge was set against a controller
setpoint.** Two different bases, both defensible on their own terms, and **nothing in the tree
ever divided the oxygen pool by the air** — so the incoherence could not surface. `Ci` has
been computed from CO₂/air since Phase 2; O₂ was a constant 210 until the form was built.
**Adoption is the first thing that ever compares them.**

### 10c. ⚠ The param file PREDICTED this, in its own `source:` string

`eclss.yaml`'s `o2_setpoint` carries, and has carried since it was written:

> DESIGN — sizing choice, not a literature value: target cabin O₂ inventory. **PERMANENTLY
> un-bindable as written** — real systems regulate O₂ **PARTIAL PRESSURE**, never a mole
> count, so the units themselves foreclose citation (**restating it as a ppO₂ target would be
> a model change, not a citation edit**). Not calibrated.

That is this finding, written in advance, by whoever wrote the param. What it could not know
is *when* the mole-count convention would stop being harmless. **Adoption is that moment**:
the setpoint stops being an isolated inventory target and becomes a number the crop's
photosynthesis divides by an air volume.

⚠ And the honest fix is the one that string already names — a ppO₂ target, a cabin volume, and
`n = pV/RT` — which it correctly calls **a model change**. At a 21 kPa ppO₂ in a 101.3 kPa
cabin, a 9500 mol atmosphere holds ≈ **1995 mol** of O₂, ~200× the current setpoint. That is
a frozen **station** param moving by two orders of magnitude, which reaches the ECLSS, crew
and cabin-gas goldens as well as the three sealed assemblies. **It is a bigger unfreeze than
adoption is**, and it is not something to fold into adoption's ceremony.

### 10d. What this does to the courses

**§9's three courses are replaced.** (a) is a no-op as written; the real (a) is much larger
than described.

* **(A) Adopt now, and record the oxygen setpoint as a KNOWN, named limitation.** The form is
  right, the band re-poses (§8a), the controls behave (§8d), the manifest gate catches a
  half-done job. The 60–75 % move on the station's flagship golden is then frozen with a
  written statement of what causes it — a cabin oxygen inventory the param file itself calls
  un-bindable — and the ppO₂ restatement becomes the named successor item. **This is what I
  now recommend.** It is honest, it is bounded, and it does not hold a correct piece of
  science hostage to a much larger model change.
* **(B) Do the ppO₂ restatement first, then adopt.** Scientifically the tidiest order and it
  keeps the two causes in separate golden diffs. But it is a two-order-of-magnitude move in a
  frozen station param, touching the ECLSS, crew and cabin-gas contracts, and it needs its own
  citation work (a ppO₂ requirement) and its own cabin-volume decision. **A much bigger job
  than the thing it is unblocking**, and adoption would wait on it.
* **(C) Leave the form in the lab.** Still available, still costs nothing — the form stays
  reachable as `science_switch -- o2form=live_pool`, exactly as the Q10 form does. Weaker than
  it looked in §7, because §8a and §8c removed both scientific objections to adopting.

⚠ **What is NOT on the list, and was on §9's:** re-charging `chamber_o2_mol0`. §10a rules it
out — a controller undoes it.

---

## 11. SLICE 1 — the cabin oxygen setpoint, restated from a cited atmosphere

**Chosen by the user 2026-09-06: "fix the setting first."** This section is the plan and the
predictions, written **before** anything is flipped, for the same reason §2 was: the discipline
has now caught three of my own errors in a row.

### 11a. The design — a value-and-provenance move, NOT the model change the param file feared

`eclss.yaml`'s `o2_setpoint` says of itself that it is *"PERMANENTLY un-bindable as written"*
because it is a mole count and real systems regulate partial pressure. **That is true of the
units and false of this model**, and the reason is that `V` and `T` are constants here — there
is no volume state and no cabin thermodynamics — so a mole target and a partial-pressure
target differ by a fixed factor, and regulating one *is* regulating the other. **The
un-bindability was the derivation being absent, not the model form.**

⚠ **The structural alternative was considered and rejected on measurement, not taste.** Making
the setpoint a mole *fraction* that the flow multiplies by the cabin's air is the tidier shape,
and it is what I was advised to build. It is wrong here: within `O2Makeup`'s reach there is
exactly **one** air inventory (9500 mol, the greenhouse group) and **four scenarios with no air
inventory at all** — `eclss` standalone, `cabin_gas`, `water_recovery`, `crew` never convert a
pool to a concentration. The fraction design would force inventing a cabin size for four
scenarios that do not have the concept, which is scope growth of exactly the kind this document
declined for `chamber_air_mol`'s provenance. **Where the setpoint is actually used, the two
designs differ by a constant.**

**What is owed instead is the trigger, in writing, in the `source:` string:** if an ECLSS is
ever wired to a cabin whose air inventory is not 9500 mol, this value must become a fraction
and the flow must read the air. That converts a latent coupling into a stated condition, which
is the difference between a documented simplification and a hidden one.

### 11b. The number, and the choice of atmosphere is SCIENTIFIC

From **BVAD Rev 2 (NASA/TP-2015-218570), §4.1.1 "Design Values for Atmospheric Systems",
p. 61**, verbatim:

> ISS EVA operations originate from **21% oxygen and 101.3 kPa (14.7 psia) of pressure**, with
> a prebreathe period at 70.3 kPa (10.2 psia).

`n_O2 = 0.21 × 9500 mol = **1995.0 mol**`.

⚠ **Which atmosphere is a modelling decision with a consequence, and it is stated as one.**
The same page and `Table 4-1` (p. 63) give *three* nominal total pressures — **"101 or 70.3 or
56.5"** kPa — and the reduced-pressure exploration option runs **34 % oxygen at 57 kPa**
(Norcross 2013) so that lungs see an Earth-like ppO₂ at lower total pressure. **The crop reads
the mole fraction**, so 34 % would give a materially different photosynthetic and microbial
response than 21 %. This habitat is modelled as **ISS-like, sea-level-equivalent**, and that is
a choice, not a lookup.

⚠ **A weak internal-consistency reading supports it and is not presented as a derivation:** the
same cabin carries CO₂ at 399.6 ppm — Earth-ambient — and BVAD's own `p[CO₂] for Plants` lower
bound is *"0.04 kPa (4) Earth normal"*, which at 101.3 kPa is 395 ppm. A ppm is dimensionless
and fixes no total pressure, so this cannot select the atmosphere; it is the reading that
leaves the tree self-consistent.

⚠ **Cross-check between two loci, not a second derivation:** `Table 4-1`'s *"p[O₂] for Crew;
nominal no impairment"* is **21.2 kPa** nominal (ref (2) NASA HIDH 2014), which over 101.3 kPa
is 20.93 % — within 0.4 % of the 21 % cabin composition. Those are **different quantities** (a
crew physiological requirement vs a cabin composition) and the agreement is a consistency
check. The value taken is the composition, because the composition is what this model needs.

### 11c. ⚠ The bad number was NEVER inert — the soil microbes have been reading it all along

The oxygen form is not the only thing that reads this pool. `MaintenanceRespiration` (and five
sibling flows) throttle by `f_O2 = x/(k + x)` with `x = o2_mol/air_mol` and
`k = o2_half_saturation = 1e-4` (`respiration.yaml`), and that coupling is **live and frozen**,
not lab-only.

| cabin | x_O₂ | `f_O2` |
|---|---|---|
| greenhouse / harvest / sealed_station, **today** | 8.529e-4 | **0.8951** |
| greenhouse / harvest / sealed_station, **fixed** | 0.21 | **0.99952** |
| perennial / consumer chambers (correctly charged) | 0.2102 | 0.99952 |
| `sealed_chamber` (the jar — designed near-anoxia) | 3.32e-5 | 0.2492 |

**So the station's soil respiration has been throttled to 89.5 % of its unlimited rate for four
phases, by an oxygen inventory 200× too low, while every correctly-charged chamber sat at
99.95 %.** The defect was never waiting for adoption to matter — adoption is only what made it
*visible*. This is the honest reason the fix comes first.

### 11d. The edit set — five numbers that move together

They move together or the runs acquire a start-up transient that is not the science:

1. `rust/crates/domains/params/eclss/eclss.yaml` — `o2_setpoint` 10.0 → **1995.0**, with the
   citation and the trigger condition in `source:`. ⚠ This file is **hashed in the station
   manifest**, so `station/tests/manifest_writer.rs` is the automatic gate.
2. `station/src/scenario.rs::greenhouse_bio_scenario` — `chamber_o2_mol0` 10.0 → 1995.0.
3. `sealed_station_bio_scenario` — inherits (2) via `..greenhouse_bio_scenario()`; **verify**.
4. `station/src/scenario.rs::CABIN_GAS_SCENARIO` — `cabin_o2_0` 10.0 → 1995.0
   (`WATER_RECOVERY_SCENARIO = CABIN_GAS_SCENARIO`, so it follows).
5. `domains/src/eclss.rs::STEADY_STATE_SCENARIO` — `cabin_o2_0` 10.0 → 1995.0.

Plus prose that becomes false: `EclssScenario::cabin_o2_0`'s *"starts at the setpoint"* stays
true only if all five move; the `HAND` const in `eclss.rs`'s test module restates the yaml.

### 11e. ⚠ A BLOCKER of the drift_summary class — and it is the one real cost

`domains::params::tests::every_value_matches_the_generated_table` asserts **bit equality** of
twelve loaded params against `src/sibling_params.txt`, a hex-float table whose own header reads
*"GENERATED, do not edit. Source of truth: the frozen Python loaders"* and whose regeneration
line is `uv run python tests/crossport/gen_sibling_params.py` — **a file S6 deleted.** The
directory does not exist.

So moving `o2_setpoint` reddens a control that **cannot be regenerated**. Three options:

* **Hand-edit the hex float. REFUSED.** The file asserts its values came from Python's loaders.
  Writing a number Python never produced makes the file lie, and it is a control precisely
  because nobody edits it.
* **Retire the whole table.** Its purpose — proving slice C1's re-anchoring bit-neutral — is
  discharged, and Python is gone so it can never be regenerated for any future move. But this
  orphans the surviving evidence for eleven params that have *not* moved.
* **Retire the `o2_setpoint` ROW only**, dropping it from `pairs` and the count assertion
  12 → 11, with the reason recorded in the file. **This is what I will do.** The eleven
  unmoved params keep their bit-level control; the moved one keeps its manifest hash and its
  goldens, which is adequate. ⚠ The count assertion going 12 → 11 is a real, if small,
  weakening and is called that rather than presented as a tidy-up.

### 11f. Predicted golden set, written before the run

**Predicted: 6 of 20 change** — `eclss_state`, `cabin_gas_state`, `water_recovery_state`,
`greenhouse_state`, `harvest_state`, `sealed_station_state`. Unchanged: `crew_state` (no
ECLSS), `station_state` (no oxygen stocks), `lighting_state`, the four chambers,
`season_euler_state`, `drift_summary` (folds chambers with no ECLSS), and
`sealed_energy_drift_summary` (thermal folds only — it did not move for the oxygen form
either).

⚠ **A sharp prediction that can fail.** For the three scenarios with **no biosphere writing the
pool** — `eclss`, `cabin_gas`, `water_recovery` — the regulator's dynamics are in the deviation
`e = setpoint − cabin_o2`, and **both** the setpoint and the initial condition shift by the same
+1985.0. So `e(t)` is unchanged, and therefore:

* `eclss.cabin_o2` / `biosphere.o2_pool` shift by **exactly +1985.0**, and
* **`boundary.o2_supply` and `boundary.co2_removed` are BIT-IDENTICAL.**

If the supply moves on those three, the shift is not a pure translation and something else
reads the pool. For the three station assemblies it is **not** a pure translation — the
biosphere both writes the pool and reads it through `f_O2` (§11c) — so their biomass moves too:
**direction predicted DOWN**, because a less-throttled maintenance respiration burns more.
Magnitude predicted **small** — single-digit percent — and in any case far below the 60–75 %
that adoption itself produces.

### 11g. Predicted red set

1. `every_value_matches_the_generated_table` — RED, §11e, no regeneration path. **The one
   finding this slice buys.**
2. `station/tests/manifest_writer.rs` — RED until the station manifest is regenerated (the
   `eclss.yaml` sha-256 moves). The automatic gate, confirmed present.
3. The six golden comparisons — RED until regenerated.
4. `domains/tests/eclss_run.rs`'s steady-state check — **GREEN**: it asserts the *relation*
   `o2_eq = o2_setpoint − Con/k`, not a value. A pin on a relation survives a value move; that
   is the whole reason to write pins that way.
5. `station`'s `crew_mission` science gate (`bvad_o2_per_cm_per_day`) — **GREEN**: at steady
   state the supply flux equals the crew's consumption whatever the setpoint. ⚠ Verify rather
   than reason — it also asserts `rationed == 0` and no extinction events, and 900 steps must
   still converge from the new initial condition. A 200× larger setpoint gives the regulator
   more headroom, so both should improve; "should" is what the last three refutations were
   built on.
6. `domains/src/params.rs`'s loader tests — **GREEN**: they build synthetic YAML inline and
   never read the committed file.

---

## 12. SLICE 1 MEASURED — the golden prediction was exact, the arithmetic one was not

Run after §11 was committed. Scored against §11f/§11g line by line, including the misses.

### 12a. ✅ The golden set — exact

**Predicted 6 of 20; measured 6 of 20, and the same six**: `eclss_state`, `cabin_gas_state`,
`water_recovery_state`, `greenhouse_state`, `harvest_state`, `sealed_station_state`. Every
scenario predicted unchanged was `identical`, including the two drift summaries, `crew_state`,
`station_state`, `lighting_state`, the four chambers and `season_euler_state`.

### 12b. ✅ The translation — exactly +1985.0, on all three pools

`1995.0 − 10.0 = 1985.0`, and the shift is that number to the last bit:

| golden | stock | before | after | Δ |
|---|---|---|---|---|
| `eclss_state` | `eclss.cabin_o2` | 8.0 | 1993.0 | **+1985** |
| `cabin_gas_state` | `eclss.cabin_o2` | 8.102 | 1993.102 | **+1985** |
| `water_recovery_state` | `eclss.cabin_o2` | 8.102 | 1993.102 | **+1985** |
| `greenhouse` / `harvest` / `sealed_station` | `biosphere.o2_pool` | 8.102 | 1993.102 | **+1985** |

And on the three runs with no biosphere writing the pool, **only 2 of 9–10 stocks moved at
all** — the pool and its boundary source. The controller offset survives verbatim: 1995 − 2.0
for the standalone (`Con/k = 0.004/2e-3`), 1995 − 1.898 for the crewed cabins.

### 12c. ⚠ MISS — `boundary.o2_supply` is NOT bit-identical, and the reason is arithmetic

§11f predicted it in bold: *"`boundary.o2_supply` and `boundary.co2_removed` are
BIT-IDENTICAL."* Measured: `co2_removed` is, `o2_supply` is not. It moves by **7.24e-11** on
the standalone and **6.20e-11** on the two cabins — a **relative** change of 3.4e-13.

**The physics was right and the arithmetic was wrong.** The regulator's dynamics live in
`e = setpoint − cabin_o2`, and translating both by the same amount leaves `e(t)` invariant —
*in real arithmetic*. In `f64` it does not: subtracting two numbers near 1995 rounds
differently from subtracting two numbers near 10, so `e` is preserved only to about 1e-13
relative, and the supply is `∫k·e dt`. The residual is the signature of that cancellation, and
its size is the right size for it.

⚠ **Worth keeping because it is a fact about the model, not this edit:** the O₂ regulator is
now solving for a small deviation on top of a large inventory, where before both were small.
It is numerically less well-conditioned at 1995 mol than at 10 — harmlessly so at 3e-13, but
that is the direction, and a future cabin an order of magnitude larger would push it further.
**A prediction of bit-identity across a translation is a claim about real arithmetic; the
machine does not make it.**

### 12d. ⚠ MISS — two unpredicted reds, and I checked the thing I was looking at

Predicted red: the goldens, the station manifest gate, and
`every_value_matches_the_generated_table`. All three fired. **Two more did, unpredicted:**

* **`params::tests::eclss_loader_reads_the_committed_params`** — §11g item 6 said the loader
  tests *"build synthetic YAML inline and never read the committed file."* True of the others,
  **false of this one**, which calls `eclss()` and asserts `o2_setpoint == 10.0`. Re-posed to
  1995.0: it is the right place for the committed value to be pinned, and it fired correctly.
* **`o2_makeup_adds_toward_setpoint_and_dt_linear`** and **`o2_makeup_idle_at_setpoint`** —
  flow-level tests that build a state at the old setpoint (`state(8.0, …)` against a literal
  10.0, and `state(10.0, …)`).

⚠ **How I missed the pair is the same defect as §8a's ×685.** I searched for tests pinning the
setpoint, found `makeup_flux_is_proportional_to_the_deficit` and `makeup_flux_zero_at_setpoint`
— which pass their arguments *explicitly* and therefore stay green — reasoned correctly about
those, and wrote the conclusion as though it covered the family. The flow-level pair sits one
screen further down the same file. **A search that finds an instance of what it is looking for
stops looking.** Third time in this document.

⚠ **Both were re-posed rather than re-valued**, and that is a strengthening: they now read
`HAND.o2_setpoint − deficit` and `HAND.o2_setpoint`, because their claims are *"proportional to
the deficit"* and *"idle at the setpoint"* — neither is a claim about where the setpoint sits.
The setpoint's value is pinned once, in the loader test, which is where it belongs. The old
form pinned two things in one assertion and went red for the wrong one.

### 12e. ✅ The relation pins survived — which is why they are written that way

* `domains/tests/eclss_run.rs` asserts `o2_eq = o2_setpoint − Con/k` and is **green**. A pin on
  a relation survives a value move; a pin on a value would have been a second red with an
  argument owed. Predicted green, and green.
* The station's `crew_mission` science gate is **green**: at steady state the supply flux equals
  the crew's consumption whatever the setpoint, `rationed == 0` still holds, and 900 steps still
  converge from the new initial condition — which was the part §11g said to verify rather than
  reason about.
* `every_value_matches_the_generated_table` is **green** after the row retirement (§11e).

### 12f. ⚠ MISS — the manifest diff was 7 lines, not 1

§11d predicted the station manifest would move *"the `eclss.yaml` sha-256"*. It moved **seven
hashes**: that one **plus the `golden_sha256` of all six changed goldens**, which the manifest
carries per scenario. Benign, internally consistent, and caught only because the ceremony asks
for the prediction — no `flow_set`, `aux_set`, scenario or science band moved, which is the
part that would have mattered.

### 12g. The science — plants down as predicted, soil larger than predicted

Direction predicted **DOWN** for biomass, magnitude **single-digit percent**. Plants: correct
on both. Soil: correct on nothing.

| run | leaf | stem | root | storage | humus C | microbial C | litter C |
|---|---|---|---|---|---|---|---|
| `sealed_station` | −0.047 % | −0.047 % | −0.051 % | −0.034 % | −1.20 % | **−12.77 %** | **−12.08 %** |
| `greenhouse` | −0.335 % | −0.335 % | −0.335 % | — | **+23.94 %** | **+10.86 %** | — |
| `harvest` | — | — | — | — | **+24.57 %** | **+11.21 %** | — |

**The plants barely notice and the soil moves by a quarter.** That is the right shape: `f_O2`
throttles *respiration*, and the microbes are the ones respiring — 0.8951 → 0.99952, a 11.7 %
release of the throttle, which lands almost exactly on the microbial carbon in the two short
runs (+10.9 %, +11.2 %). The plants lose only the small maintenance-respiration share the same
factor gates.

⚠ **The two directions are not a contradiction, they are horizon.** `greenhouse` and `harvest`
are 28-step runs: faster decomposition has built humus and microbial biomass and has not yet
drawn the litter down. `sealed_station` is 4880 steps: the same faster decomposition has run to
its consequence, so litter and microbial standing stock are **lower** while humus has been
processed through. A rate change raises a pool early and lowers it late; reading either horizon
alone would have given the wrong sign for the other.

⚠ **And this is the whole justification for slice 1 being first.** These numbers are what the
bad oxygen inventory was already costing, with the frozen constant still in place and adoption
not taken. Had adoption landed first, this 12–25 % soil movement would have been inside the
60–75 % diff and indistinguishable from the oxygen science.
