## **The live-O₂ FvCB form is ADOPTED** (the previous slice removed this one's cost: the station's flagship golden moves 0.058 %, where the same flip moved it 60–75 % a day earlier — and an oxygen leak now makes the crop grow BETTER)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-07**, on the user's *"slice 2 (adopting the live-oxygen form) — do it."*
Slice 2 of two; slice 1 is `log/o2-setpoint-cited.md`. The form itself was built and gated on
2026-09-06 (`log/o2-form-built.md`); this item is only the choice of which built form the
reference selects, plus everything that choice touches.

Plan, every prediction, and their scoring: `docs/plans/post-roadmap-o2-form-adoption.md`
§13 (written before any run) and §14 (measured). Unfreeze entries:
`docs/biosphere-reference.md` and `docs/station-reference.md`, both dated 2026-09-07.

**What changed, in one line:** `rust/crates/domains/src/biosphere/params.rs:473`,
`o2_form: O2Form::Constant → O2Form::LivePool`. Ten goldens, 21 manifest lines on the
biosphere side and 4 on the station's, five re-posed science gates, and one station
perturbation claim that inverted.

---

## 1. The headline is that the previous slice removed this one's cost

Measured on 2026-09-06, **before** the cabin oxygen setpoint was cited, the same flip moved
`sealed_station`'s plant and soil carbon by **60–75 %**. Measured after: **+0.058 %**.

| golden | pre-slice-1 | post-slice-1 (shipped) |
|---|---|---|
| `sealed_station`, largest carbon move | +74.9 % | **+0.058 %** |
| `greenhouse` | +89.5 % | **+0.046 %** |
| `harvest` | +72.1 % | **+0.049 %** |
| `sealed_chamber` (the jar) `o2_pool` | +363.3 % | **+363.3163 %** |
| `sealed_chamber` `leaf_c` | −44.8 % | **−44.7829 %** |

The cause was never the oxygen science. `greenhouse_bio_scenario` put 10 mol of O₂ into
9500 mol of cabin air — **1.05 mmol/mol**, half a percent of a breathable atmosphere — and
nothing in the tree had ever divided the oxygen pool by the air, so the crop read a constant
210 regardless. Adoption is the first thing that ever compared them. Slice 1 moved the
setpoint and the charge to 1995 mol, and the station now sits at **209.800 mmol/mol** against
the frozen constant 210.

⚠ **So the station's three sealed assemblies changed role: they are CONTROLS on this form
now, not its largest consumers.** The control count went from two to five, and §2b's stop rule
— *if a control moves past 2 %, the form is wired wrong and adoption stops* — got three more
subjects. All five came in under 0.1 %.

⚠ **And the jar is the anchor that says slice 1 reached what it was supposed to and nothing
else.** `sealed_chamber` is a biosphere scenario; slice 1 touched `eclss.yaml` and
`station/src/scenario.rs`, neither reachable from it. It reproduced the pre-slice-1 probe to
four significant figures on both rows, and the five pointwise margins reproduced it to **all
six decimals** — read off the pin's own failure output on this tree, not transcribed, because
a green pin at a 2 % tolerance would only have said "within 2 %".

**Taking the two changes in one diff would have made them indistinguishable.** The 12–25 %
soil movement slice 1 measured would have been inside this slice's diff and unattributable.

---

## 2. The band is re-posed POINTWISE — a different assertion, and a HARDER one

The five `..._stays_above_the_compensation_point` gates compared one fold of a run against one
constant, `Γ*/ci_ratio = 61.071429 ppm`. They now compare two series pointwise:
`CO₂(t) > Γ*(t)/ci_ratio` with `Γ*(t) = 42.75·O₂(t)/210`.

Nothing in the new bound was chosen to make a scenario pass — **the old bound is the special
case at constant O₂**, so the four scenarios sitting at ~210 keep the bound they already had
to within a tenth of a percent. That is what distinguishes this from the three re-tunings this
project has refused (`log/canopy-magnitude-diagnosed.md`).

⚠ **On a sealed chamber the pointwise band is harder, not looser, and the plan's §4 had
concluded the opposite.** Photosynthetic quotient 1 makes CO₂ and O₂ anticorrelated, so the
jar's oxygen — and therefore its floor — is at its **highest** at the instant its CO₂ is at
its lowest. A pointwise band on a sealed run evaluates itself at the physically hardest
instant the run contains, automatically. §4 had quoted the ratio at the run's *end* (×685) and
called the band vacuous; the binding instant is step **779 of 3661** — 3 seasons × 305 days
÷ `dt = ¼`, plus the initial sample — and the true minimum is **×10.674948**. *A number read at
the wrong instant lies.*

⚠ That denominator was written as "a 1000-step run" in the first draft of this file, from
nothing, and corrected on review. It changes no conclusion, which is exactly why it is the kind
of number that survives: **an unsourced figure in prose acquires no owner**, and this section is
about a paragraph whose premises nobody ran.

Measured minima: jar **×10.674948**, the four controls **×1.150381 / ×1.200661**.

---

## 3. ⚠ A retired check nearly shipped wearing a green tick — and my own plan argued for it

`check_bound_literals` requires a bound's numeric literal to appear in **executable** text, and
it scans the whole file. `61.07` was distinctive enough that only its own tripwire supplied it.
The pointwise threshold is `1.0`, which any line of a 1200-line file satisfies.

§13c recorded this as a real loss and concluded it was **not repairable inside the bound**,
because the physical threshold *is* one and writing a measured margin there would make the
frozen contract a second, tighter copy of the margins pin — which that pin's own docstring
refuses.

**The paragraph presented two options and there were three.** The bound now reads

> `min over t of CO₂(t)/(Γ*(t)/ci_ratio) > 1.0; Γ*(210 mmol/mol)/ci_ratio = 61.07 ppm`

carrying its **anchor**: the frozen params' own derived value, the number the old bound already
carried, supplied by `the_floor_is_where_the_frozen_params_put_it` alone. Not a measured
margin, so the objection that killed the idea never applied to it.

⚠ **The shape, and it is the transferable one: a well-argued paragraph explaining why
something CANNOT be fixed is the easiest place in a plan for a missing option to hide.** The
prediction discipline catches a wrong number by running it. It does not catch an argument whose
premises were never enumerated — nothing runs an argument. Caught in review, one step before
the manifests were regenerated; a step later it would have cost two regenerations.

---

## 4. ⚠⚠ THE FINDING — an oxygen leak now makes the crop grow BETTER

`station::perturbations::o2_leak_is_absorbed_by_makeup_effort` asserted that the O₂ pool is
*defended*: `O2Makeup` is demand-controlled, so a leak surfaces as makeup **effort** and *"the
plant is essentially UNTOUCHED"* — measured at a **10715×** contrast (a carbon leak moved
biomass 16.6 %, an oxygen leak 0.0015 %).

Under the adopted form the contrast is **under 2×**: carbon −18.4 %, oxygen **+9.8 %**.

⚠ **The old claim was true, and it was a claim about the CROP BEING BLIND, not about the
controller.** `photosynthesis.o2` was a constant, so the oxygen pool was pure ECLSS
bookkeeping and no amount of leaking could reach the biology. Adoption gives the leak a second
path — the pool drives `Γ*` and the Rubisco denominator — and the regulator, which defends the
*level*, cannot defend the crop from the excursion on the way there. The ECLSS signature
(supply works harder, the sink fills) is unchanged; what retired is "orders quieter".

⚠ **The sign is the part worth reading twice: the crop ends AHEAD.** Less oxygen is less
photorespiration. A perturbation that reads as damage on the ECLSS side reads as a **yield
increase** on the biology side, and nothing in the model was arranged to produce that — it is
FvCB's own oxygen terms arriving. The re-posed test asserts the **signs** of both leaks rather
than a magnitude, because a sign is what a mis-sensed coupling breaks and no conservation or
arbitration check can see it.

⚠ **And this was invisible in the golden diff.** `sealed_station` moves 0.058 % under
adoption and 9.8 % under a leak on the same tree. **A form can be nearly inert on the frozen
roster and loud under perturbation** — the roster is a set of nominal runs, and only the
perturbation suite asks what happens off them.

---

## 5. Two tests would have stayed GREEN on retired claims, which is worse than red

Both read the loaded params object, and the live substitution happens per step **inside the
flow**, so neither noticed the reference had stopped using the number it pinned:

* `the_floor_is_where_the_frozen_params_put_it` — re-posed to pin what 61.07 actually is now,
  the floor **at the reference oxygen**, plus the link that makes it load-bearing: at
  `x_O₂ = photo.o2` the live form reproduces the frozen `Γ*` exactly, so 61.07 is the anchor
  the pointwise floor scales from rather than a number sitting in a file.
* `the_shipped_floor_is_the_conservative_one_against_the_cited_route` — re-posed with the
  assertion that makes its **citation** survive: both parameterizations are linear in O₂ (ours
  by construction, Teh eq. 6.19's by derivation), so their ratio does not depend on the oxygen
  and the conservativeness ordering is **scale-invariant**. Checked at the jar's depleted
  oxygen, not argued at 210 and hoped for.

Same class as `log/science-gate-census-in-rust.md`: *a DELETED claim leaves the suite green.*

---

## 6. The predicted red set: 5 of 5, plus 5 that were not predicted

Predicted and confirmed: the jar's band gate, the margins pin (which printed `0.119443` —
`7.294541 ppm` against the old constant floor, exactly the plan's number), the `o2_form`
loader identity control, and both `golden_regression` targets.

Missed:

1. **`manifest_writer` did NOT redden at the flip.** §2d called it *"the one automatic gate"*
   and §13d predicted it; the manifest's `golden_sha256` hashes the golden **file on disk**,
   which no build had rewritten yet. The gate is real but fires at **regeneration**, one step
   later than assumed. *It is what catches a half-done job; it is not what tells you a flip
   reaches the goldens.*
2–3. **Both `tier_contract` targets** — the cross-port tolerance bands read the same goldens.
   §13d listed `golden_regression` and stopped: *enumerate the readers of a file, not the one
   you thought of.*
4. **`a_live_oxygen_column_is_built_through_the_seam_...`** asserted the **baseline** column
   has a floor. Re-posed by swapping which form is the variant — asking for a `LivePool`
   variant today compares the reference against itself and passes while measuring nothing.
5. **The perturbation claim** — §4 above.

⚠ **`cargo clippy` was not run at builds A, B or C** — three `cargo test` runs and no lint,
against a CLAUDE.md that lists both as the reference's own gates. The warning was real and was
named in review before it was run (`folds::min_ppm` left imported and unread by the
`measured()` rewrite). **A gate you do not run is indistinguishable from a gate that passes.**

---

## 7. Two things adoption newly owed, and one it REFUSED

* **The fold's own witness.** `min_compensation_ratio` is the only reader of the pointwise
  claim, so a fold dividing by the wrong air inventory would give a plausible number, a green
  suite and a contract asserting nothing. The control is the collapse: under forced-`Constant`
  params it must reproduce `min_ppm/floor_ppm` **exactly**. Plus its anti-vacuity sibling — a
  fold that *ignored* the oxygen series would pass the identity too, so it is separately
  asserted that the answer moves under `LivePool`.
* **`open_season` falls through to the constant INSIDE the reference now.** The form rides the
  params object but the value it reads is a stock, and the open field has none. Correct physics
  for a field breathing the atmosphere — and byte-for-byte identical to forgetting to wire the
  form. While `Constant` was the reference nothing depended on telling those apart. Both halves
  are now asserted separately: the run is unmoved by the form, **and** the reason is that it
  carries no oxygen stock. `log/o2-form-built.md`'s *a half-switch cannot guard itself*,
  arriving at the reference.
* **The station-side band is REFUSED, and the reason is not the ceremony.** The station census
  carries no CO₂ band (`plan §2d`), so the cabin oxygen the crop now reads is asserted by
  nothing. A gate for it was designed and refused **on measurement**: `O2Makeup` holds the pool
  at 209.800 mmol/mol, any defensible band around 210 leaves ~50× of headroom, and every input
  that could move it already moves four goldens and a param hash. **It would have been inert by
  construction** — `log/station-science-claims-in-rust.md`'s failure, where three controls
  failed before one bit. A recorded gap is a better artifact than a green row that cannot go
  red. (It is also a third claim in a census whose own test calls a third claim a widening.)

---

## 8. What is left open

* The **station has no CO₂ band and no cabin-oxygen band**, by the decision in §7. Closing it
  needs a subject with a reachable falsifier — a perturbed run rather than a regulated nominal
  one. §4 suggests where: the perturbation suite is where this form is loud.
* `lab/report.rs` prints `n/a` for the compensation floor in **every** reference column now
  (§13f, re-read after the runs and kept). The cell is a refusal and the refusal is still true;
  the quantity that replaces it lives in `margins::PINNED` and the five gates, so it is moved
  rather than lost.
* The **ppO₂ restatement** named in §10c of the plan is still the honest successor to slice 1's
  mole-count setpoint. Adoption does not depend on it — `V` and `T` are constants here, so the
  two differ by a fixed factor — and slice 1 wrote the trigger into the param's own `source:`
  string: if an ECLSS is ever wired to a cabin whose air inventory is not 9500 mol, the value
  must become a fraction and the flow must read the air.
