# The live-O₂ FvCB form — **both halves**, built lab-only

**Decided 2026-09-06 by the user: "so build both."** That answers the question the
measurement left (`docs/log/o2-coupling-measured.md`: *"build both halves or neither"*) and
answers only that question. **Adoption is a separate decision and is not taken here** — this
build follows the shape the Q10 temperature form took: a cited alternative, reachable from the
lab, with the loader always selecting the frozen reference. No unfreeze, no golden, no
manifest key, no band, no floor.

Plan of record for this item. Filed under the September direction plan (§2.3.2 there, and its
decision list).

---

## 1. The gap, in one paragraph

Photosynthesis in this tree reads oxygen from a **constant**: `o2 = 210 mmol/mol`, the
atmosphere's mole fraction. Three of the frozen scenarios are **sealed chambers** that carry
O₂ as a live stock, and one of them — `sealed_chamber`, "the jar" — runs its O₂ down to
**0.033 mmol/mol** by the end of its golden. So the crop's Rubisco is oxygenating against an
atmosphere it consumed weeks earlier, by a factor of **6329×** at the end. For the other two
chambers the constant is right to 0.1 %, which is why this was invisible for nine phases.

## 2. Both halves, and why half is worse than none

O₂ enters FvCB in two places:

1. the **Rubisco denominator** `Kc·(1 + O/Ko)` — the half the gap's own wording named;
2. **`Γ* = 0.5·O/S_c/o`**, the CO₂ compensation point — so `Γ* ∝ O₂`.

⚠ That proportionality is a **derivation** from the standard FvCB relation and from [D]
eq. 6.19 (`Γ* = O/(2·τ)` with `O` constant), **not** a retrieved number. It is labelled as a
derivation everywhere it appears, exactly as `TEH_Q10_GAMMA_STAR` is.

The two halves move the jar's band in **opposite directions** (measured 2026-09-02):

| what | jar season-low CO₂ / floor |
|---|---|
| frozen (`o2 = 210`) | 71.435803 / 61.071429 = **1.1697** |
| denominator half alone (`o2 = 2`) | 66.924275 / 61.071429 = **1.0958** (headroom −43 %) |
| both halves (`o2 = 2`, `Γ*` scaled) | 7.183490 / 0.581571 = **12.35** |

So **a half-built form is worse than the frozen constant** — it tightens a band that the full
form loosens tenfold. The denominator also **saturates by ~2 mmol/mol**, so all of the form's
time-dependence lives in Γ* and none in the term the gap was written about. This is why the
decision was posed as both-or-neither, and it is why this plan builds both.

## 3. Shape of the build

**The form rides the params object, like `KineticsForm`.** A new `O2Form` enum with
`Constant` (the frozen reference, `#[default]`) and `LivePool`; one field
`PhotosynthesisParams::o2_form`, never loaded from a file, always `Constant` out of the
loader. Same reasoning as the temperature form's: three flows hold a `CarbonContext` and all
three call `budget()`, so a per-flow replacement would make a step whose growth respiration is
computed off frozen assimilation and whose allocation is not — internally inconsistent, and
plausible-looking in a report.

**But the value cannot ride the params object, because it is a STOCK.** `CarbonContext` gains
`o2_pool_var: Option<String>`, wired the same all-or-nothing way as the existing sealed triple
(`co2_pool_var` / `chamber_air_mol` / `ci_ratio`). At each step `budget()` resolves the live
mole fraction from the step-entry snapshot and hands `canopy_assimilation` a params object
whose `o2` and `gamma_star` carry it. Under `Constant` it hands over `self.photo` unchanged,
so every operation downstream is the frozen one and the goldens cannot move.

⚠ **The two forms compose and must commute.** `Γ*` is scaled by `O/O_ref` here and by a Q10
there; both are multiplicative, so the composition is order-independent. That is a property to
pin, not to assume — a test asserts it, because if it ever stops holding, a table with both
switches thrown means nothing.

## 4. ⚠ The three things this build must not get wrong

### 4a. The frozen guard is NOT re-posed, and that is deliberate

`min > Γ*/ci_ratio (61.07 ppm)` is written against a **constant** floor. Under this form Γ*
tracks a live stock, so the claim becomes `CO₂(t) > Γ*(t)/ci_ratio` **pointwise** — a
different assertion, not a re-tuned one, and not what the five frozen
`..._stays_above_the_compensation_point` gates say. **The guard is left exactly as it is.**
Re-posing it would be an unfreeze of a frozen band in a build whose whole claim is that it
unfreezes nothing. What this build owes instead is that the report must not *print* the stale
floor as though it applied — see 4b.

### 4b. The report must refuse to compare against a floor the form dissolved

The lab report has a row *"chamber CO₂ compensation point (ppm) — the floor the CO₂ rows are
read against"*, and it is computed from the params object's `gamma_star`. Under `LivePool`
that object still holds 42.75, because the substitution happens per step inside the flow — so
the row would print **61.071429** and a reader would divide by it. That is the exact failure
this project keeps recording: a number in prose acquiring no owner.

The fix uses machinery the report already has. A column measured under `LivePool` marks that
row **not applicable**, with the reason printed in the cell — the same treatment a knockout
gets for a flow a scenario does not contain. *"A question that scenario cannot answer"*
becomes *"a question this form cannot answer with a constant."*

### 4c. The controls are the two big chambers, and the open field is inert BY CONSTRUCTION

⚠ This is the correction the measurement's own record insisted on. The **value switch**
substitutes `o2` globally, so its columns for `perennial_chamber` and `consumer_chamber` were
counterfactuals rather than form consequences. The **form** is scenario-shaped: it reads a
stock that only sealed builds have.

* `open_season` has no O₂ pool, so it keeps the atmospheric constant and its rows must be
  **bit-identical**. That is *by construction*, and the record must say so — it is not
  evidence the form is small.
* `perennial_chamber` and `consumer_chamber` sit at ~210.2 mmol/mol and must move by
  **~0.1 % or less**. These are the real controls: a form that moves them is wired wrong.
* `sealed_chamber` is the **only** row where the science can move. Predict its direction
  before running.

## 5. Prediction, written before the run

1. `open_season` rows: **0.000 %**, all of them, by construction (no O₂ stock).
2. `perennial_chamber` / `consumer_chamber` season-low CO₂: move, but **< 0.5 %**.
3. `sealed_chamber` season-low CO₂: **falls well below the frozen 71.44 ppm**, because
   Γ* collapses toward zero and the crop keeps fixing carbon far past the point the frozen
   floor calls its compensation point. Direction: down. Magnitude: order 10 ppm or below.
   ⚠ **A number to fail against, written before the run.** The 2026-09-02 coupled column
   (`o2=2 + gamma_star=0.4071`, the jar's CHARGE fraction) read **7.183490 ppm**. That column
   was a *global* substitution and is therefore **not** this form — but at the charge fraction
   the two should be close. **If the form lands far from ~7 ppm, the wiring is wrong, not the
   science.** This is the only external cross-check available, and it is written here so it
   can fail.
4. `perennial_long_horizon` peak-leaf fixed point: **unchanged to ~0.1 %** — its O₂ never
   depletes. ⚠ Not the 0.417240 the value switch produced at `gamma_star=0.4071`; that column
   was a counterfactual, and reproducing it here would mean the form is wired globally.
5. `regen_goldens`: **20 of 20, 0 would change.**

Anything else is a finding to chase, not a number to accept.

## 6. Gates this build owes

* the loader default is `Constant`, and flipping it reddens something (the bit-identity
  control's shape);
* `Constant` reproduces the frozen tree **bit for bit** on all measured quantities;
* `LivePool` at a mole fraction of exactly 210 reproduces `Constant` bit for bit — the
  identity control, the analogue of `ScaledMechanism` at 1.0;
* the two forms **commute** (§3);
* `Γ*` scales **linearly** with the mole fraction, and the Rubisco denominator **saturates**;
* the open field is unreachable by the form, asserted rather than observed;
* a mutation that makes `LivePool` return the `Constant` answer reddens at least one test;
* ⚠ **the mutation that matters is aimed at the Γ\* half, not the denominator half.** The
  denominator **saturates by ~2 mmol/mol** — measured 2026-09-02, `o2=2` and `o2=0.033` agree
  to six figures — so at the jar's charge AND at its end a denominator-only form and the full
  form differ almost not at all *in the denominator term*. Dropping Γ\* should be loud;
  dropping the denominator may be **invisible on every gated row**. If it is, that is a
  **finding to record, not a gap to paper over**: it would be the honest version of "both
  halves or neither", and the shape `s5-batch-f` already named — *a redundant guard has no
  mutation that reddens*;
* ⚠ **the silent-baseline guard lives in the test file, because it cannot live in the code.**
  [`oxygen_at`] falls through to the frozen params when a scenario has no O₂ stock, and the
  open field legitimately needs that. So a caller who forgot to wire `o2_pool_var` would get a
  clean `LivePool` run that reads the baseline back — exactly the failure `biosphere_with`'s
  header names. The test file therefore asserts that `sealed_chamber` under `LivePool`
  **MOVES**. A column that reads unchanged where it must move is the one thing this design
  cannot distinguish from correct wiring on its own.

## 7. What this does NOT do

- It does not adopt the form. The reference stays `Constant`.
- It does not re-pose the CO₂ compensation guard (4a).
- It does not touch `respiration.yaml`'s `o2_half_saturation`, which is a different O₂
  coupling (microbial), already live and already reading the stock.
- It does not answer whether a habitat whose crop sees its own O₂ is the right realism move.
  That is the adoption decision, and it arrives **after** these numbers.
