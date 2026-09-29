# The parked leaf mechanism, re-measured in Rust — a LAB-ONLY port

**Opened 2026-09-29** on the user's *"go with your recommendation"*, where the recommendation
was the **re-measuring step only** of the third direction plan's §2.1 (KEEP, as a new Rust work
item, 2026-09-23). This doc is that step. It is **not** the adoption.

## 0. The price, corrected before building

The recommendation called re-measuring *"cheap, changes nothing"*. **Half of that was false:**
the mechanism does not exist in Rust — the branches `leaf-expansion-blocked` /
`leaf-expansion-rebase` are Python (`src/`, deleted by S6) — so re-measuring means **porting it
far enough to run**. Said to the user before building (advisor call 1). What stays true is
the other half: the port is **lab-only**, and nothing frozen moves.

## 1. Route — a lab FORM, on the `KineticsForm` / `O2Form` precedent

* **The seven numbers are cited Rust `const`s in `science.rs`, not a param file.** Measured
  before choosing: `params::tests::the_census_matches_the_directory_on_disk` equates
  `param_files()` with a non-recursive `read_dir` of `params/biosphere/`, and the manifest
  hashes that set — so a `leaf_area.yaml` there would be a manifest entry, i.e. an unfreeze.
  The Q10 lab form's numbers (`TEH_Q10_*`) live as consts in `science.rs` for exactly this
  reason; this copies it. `plant_density = 300` rides with them — [F] files it as management
  data, which is a scenario field on adoption, not here.
* **`LeafAreaForm { Derived, NodeEnvelope }` on `CanopyParams`**; the loader sets `Derived`.
  Reachable only through `lab::biosphere_with_leaf_form` and
  `science_switch -- leafform=node_envelope`.
* **Under `Derived` nothing is constructed**: no aux process, no `leaf_area_index` aux key,
  `CarbonContext::leaf_area_aux = None`. That is what keeps the frozen build bit-identical, and
  the manifest's `aux_set` byte comparison is what proves `LeafAreaExpansion` never appears in
  a canonical build.
* The mechanism is the branch's, re-implemented from its source (`leaf_area.py`), rule for
  rule: node branch below `tu_tlm` (analytic derivative, `rate·dt`), carbon branch at/after it
  (recomputes `Allocation`'s leaf leg), `DLAI = rdr_leaf·LAI`, then the [E] envelope as an
  **absolute projection on the state** (deliberately `dt`-dependent — a bound on a state, not a
  rate law).

## 2. The five places a straight port would corrupt the measurement (advisor call 1)

1. **The readout.** `readouts::peak_lai` derives LAI from leaf carbon, so under the form it
   would read the wrong quantity. The trajectory now samples the **canopy LAI the budget
   actually reads** (the aux when present, else derived), and the thickness ratio
   `state ÷ derived` is reported beside it.
2. **The aux key is seeded only under the form** (the branch seeded it always, which here would
   move every golden that dumps aux).
3. **The re-sow must reset it** — omitted on the branch it rationed 85× on
   `consumer_long_horizon`. `annual_reset` has no params, and reaching for `params::canopy()`
   inside it is the step-time escape `param_funnel.rs` exists to catch. So: `annual_reset`
   **refuses** a state carrying a stored leaf area, and a params-aware
   `annual_reset_with` / `run_perennial_with` resets it to the seedling's derived area. Every
   frozen path keeps the old signature and could not silently skip the reset.
4. **Mutual shading keeps reading derived LAI** — the branch's choice, mirrored and **recorded
   as a decision**: it was written when `shade_rate` was bit-inert, and the layered canopy has
   since made it live on exactly the chambers the case rests on.
5. **`dt` contract** — analytic derivative for the node rate; projection for the clamp.

## 3. Predictions, written BEFORE the first run

| # | Claim | Predicted |
|---|---|---|
| P1 | Frozen build | **bit-identical**: `regen_goldens` (report-only) clean, `cargo test` green, no manifest byte moves |
| P2 | Step 0 under the form | LAI state == derived exactly (seeded from the same expression) |
| P3 | `open_season` peak canopy LAI | within **±2 %** of frozen (Aug: −0.26 % at `dt=¼`, on a tree since re-layered) |
| P4 | The chambers' peak canopy LAI | **1.15–1.35×** frozen (Aug: 1.27–1.32×) |
| P5 | Rationing, Euler, all six runs | **0** |
| P6 | Leaf thickness (state ÷ derived) | max ≤ **1.1765 × 1.01** (one ¼-step's overshoot), min ≥ **0.667** |
| P7 | Chamber CO₂ margins | tighten, by **< 10 %** of margin (Aug: ~4 %) |
| P8 | RK4, decade horizon, `dt=¼`, perennial chamber | **clean** (Aug: 34 passed at `dt=¼`) — recorded as a fact, not as evidence of safety |
| P9 | `WSFL` (leaf drought factor, threshold 0.40) | **never below 1 on the roster** — `n_limited` and `water_biting` were retired by C6 and every Rust scenario holds `WSFG == 1`; if it never fires, **drought response is UNMEASURED**, said plainly |

Compared against **today's frozen tree**, never against the 2026-08-14 numbers, which measured a
tree that no longer exists.

## 4. Outcome — MEASURED 2026-09-29; four predictions falsified, the decision stays the user's

Instrument: `cargo run --release -q -p domains --example leaf_remeasure -- --rk4` (committed,
so the evidence base is no longer probe scripts outside the repo). Gates:
`rust/crates/domains/tests/leaf_form.rs` (11 tests).

| run | peak LAI frozen → lab | peak W (t/ha, a CEILING) | CO₂ ratio (a FLOOR, >1) | rationed, Euler | RK4 |
|---|---|---|---|---|---|
| `open_season` | 6.0228 → 5.7044 (**−5.3 %**) | 13.3791 → 13.2050 | — | 0 / 0 | clean / clean |
| `sealed_chamber` (jar) | ⚠ *rationed run — not quoted* | | | 0 / **5** | clean / **raises, step 773** |
| `perennial_chamber` | 0.4928 → 0.6360 (**1.291×**) | 0.3211 → 0.3270 | 1.1504 → 1.1000 | 0 / 0 | clean / clean |
| `consumer_chamber` | 0.5848 → 0.7622 (**1.303×**) | 0.3437 → 0.3728 | 1.2007 → 1.1397 | 0 / 0 | clean / clean |
| both 15-yr horizons | identical to their 5-yr rows | | | 0 / 0 | clean / clean |

The jar's lab row is withheld on the harness's own rule — *a rationed run's numbers are not the
model's answer*. Its result is the rationing and the RK4 raise: `scale_f = 0.978` on flow #0,
step 773 = **day 193 of season 1**. August's RK4 break (`log/leaf-expansion.md` finding 9) was
at day 197 of season 1; that is recorded as a **coincidence of timing, not an explanation**.

**The predictions, graded as written:**

* **P1 HELD.** `regen_goldens` report-only: 20 of 20 identical. Every pre-existing test green;
  no manifest byte moved (`manifest_writer` green).
* **P2 HELD** — pinned by `the_form_seeds_the_seedlings_derived_area_and_wires_the_process`.
* **P3 FALSIFIED** — −5.3 %, not ±2 %. The frozen tree's own `open_season` peak LAI is now
  6.02 (it was 5.46 in August); the mechanism no longer converges toward inert there.
* **P4 HELD** on the two unrationed chambers (1.291×, 1.303×).
* **P5 FALSIFIED** — the jar rations 5 times under Euler (frozen: 0). Pinned as `> 0` by
  `the_form_rations_the_sealed_jar_and_the_frozen_tree_does_not`, not `== 5`: a marginal
  firing count can move by libm ULPs on CI's Linux box; the zero control cannot.
* **P6 FALSIFIED AS WRITTEN.** The sampled ratio reaches **0.628–0.643** against a 0.667 floor
  and 1.185–1.187 against a 1.176 ceiling. What holds is the bound **at step entry**, exactly:
  every extreme is the bound divided by that step's leaf-carbon change (min at day 10.75, leaf
  carbon +3.6–6.1 % that step; 0.6667 / 1.0362 = 0.6434). The August record discussed only the
  ceiling-side overshoot; the floor side is larger because seedling growth per step is. The
  gate now asserts the step-entry form, and a mutation dropping the floor clamp reddens it.
* **P7 FALSIFIED.** "< 10 % of margin", and the gate is a FLOOR, so the margin is `ratio − 1`:
  perennial 0.150 → 0.100 (**−33 %**), consumer 0.201 → 0.140 (**−30 %**). ⚠ The plan's own
  sentence mixed two quantities — August's "~4 %" was a change in the RATIO — and grading
  against the one that passes would have been the regrade this project keeps refusing.
* **P8 FALSIFIED** on the jar (above); clean on the other five, and every frozen run clean.
  ⚠ August's *"the RK4 blocker is gone at `dt = ¼`"* does **not** hold on today's tree.
* **P9 HELD** — `WSFL < 1` on **0** steps of any run. **The drought response of this mechanism
  is UNMEASURED**: the two scenarios that exercised it were retired by C6, and it was the reason
  the mechanism was built.

**Directions, stated per gate** (the misreading `leaf-mechanism-converging-to-inert` records):
Greenwood peak W is a CEILING, so `open_season` 13.38 → 13.21 is *more* headroom; the chambers'
W rises but sits far below it. The CO₂ ratio is a FLOOR, so lower is *less* headroom.

**What this does NOT establish, and what was deliberately not run:**

* **No cause for the jar is claimed.** Two things that bear on it changed since August — the
  live-O₂ form was adopted (it moved the jar's observable ~10×), and mutual shading, which this
  form leaves on DERIVED LAI (§2.4), became live with the layered canopy. Naming either needs a
  control (e.g. the leaf form over `O2Form::Constant`); the decision does not need it.
* **No retune was tried.** Narrowing the envelope or refining the step until the jar goes quiet
  is the retune `log/leaf-expansion.md` finding 9 refused.
* Mutual shading on the stored LAI (§2.4) is unmeasured.

**Standing:** not adopted; the lab form stays reachable (`science_switch -- leafform=node_envelope`)
and moves nothing frozen. Whether the rewrite continues is the user's decision.

## 5. The jar control — why does the sealed jar break? (opened 2026-09-29, user: *"the next step is the control run that finds out why the jar fails"*)

§4 withheld a cause. This section is the control that §4 said naming one would need. It
changes no number of the mechanism and adopts nothing; it toggles forms that already exist.

**Order (advisor call, 2026-09-29): name the failure first, then let it pick the control.**

1. **Name it.** Per Euler step, evaluate every flow at the step-entry state and read
   `arbitration::scale_factors` — the same computation the backstop applies. For each firing:
   the flow, the stock it overdraws (summed demand vs amount), stored and derived LAI, the
   chamber's CO₂ and O₂. For the RK4 raise: `registry.flows()[i].id()` names "flow #i" exactly.
   ⚠ RK4 fails at a *stage*, so an entry-state probe can read all factors = 1 on step 773; the
   flow index is exact, the limiting stock comes from the Euler side.
2. **The O₂ control, as a full 2×2** — leaf form {`Derived`, `NodeEnvelope`} × O₂ form
   {`LivePool` (the loader's, since 2026-09-07), `Constant`}. The frozen×Constant cell is
   measured, not assumed, or the control is uninterpretable. Graded 0 vs `> 0`, not by count.
3. **Mutual shading, by measurement:** max of derived AND stored LAI on the lab jar vs the
   threshold 6.
4. The named suspects are not the list: the habitat atmosphere and vapour saturation also
   changed sealed chambers after August. If step 1's stock fits neither suspect, it picks.

**Predictions, written BEFORE the first run:**

| # | Claim | Predicted |
|---|---|---|
| J1 | The overdrawn stock | the chamber **CO₂ pool** (`biosphere.carbon_pool`), drawn by assimilation: the jar holds ~2 days of carbon and the form grows a bigger canopy. Alternative I would not be surprised by: the **O₂ pool** (2 mol at start), drawn by respiration |
| J2 | The five Euler firings | one cluster, near step 773 (day ~193 of season 1), the same flow as RK4's flow #0 |
| J3 | frozen × `Constant` | **0** firings, RK4 clean (else the control means nothing) |
| J4 | lab × `Constant` | **0** firings, RK4 clean — i.e. an **interaction**: the jar breaks only with both the leaf form and live O₂. Low confidence; the O₂ form moved the jar ~10×, which is why it is my lead |
| J5 | Jar LAI, derived and stored | both stay **below 6** all run → mutual shading measured uninvolved |

### 5a. Outcome — MEASURED 2026-09-29; all five predictions held, and the jar was already close

Instrument: `cargo run --release -q -p domains --example jar_control` (committed). The probe's
own firing count is asserted equal to the integrator's `rationed` every step, and its jar
minimum under the reference reproduces `log/o2-form-adopted.md`'s **7.294541 ppm** exactly.

| cell (leaf × O₂) | Euler firings | CO₂ pool min (mol, room 1000) | tightest step's CO₂ draw ÷ held (step) | max LAI derived / stored | RK4 |
|---|---|---|---|---|---|
| frozen × `LivePool` (the reference) | 0 | 7.29e-3 | **0.757** (777) | 0.443 / — | clean |
| frozen × `Constant` | 0 | 7.14e-2 | 0.284 (734) | 0.543 / — | clean |
| lab × `LivePool` | **5** | 3.38e-3 | **1.147** (777) | 0.548 / 0.572 | raises, step 773 |
| lab × `Constant` | 0 | 6.79e-2 | 0.235 (758) | 0.601 / 0.674 | clean |

The five firings, all `biosphere.allocation` overdrawing `biosphere.carbon_pool`: steps 773,
777, 781, 785, 789 — days 193.25–197.25 of season 1, **once a day at the same quarter-day**,
f = 0.986 / 0.872 / 0.897 / 0.980 / 0.970. RK4's "flow #0" resolves to `biosphere.allocation`.
At every firing the stored leaf area sat at the envelope's **ceiling** (0.5587 / 0.4728 = 1.18×
what its carbon implies).

**Graded as written:** J1 HELD (CO₂ pool, by allocation). J2 HELD (one cluster, 773–789, same
flow as RK4). J3 HELD. J4 HELD — **an interaction**: neither the leaf form nor live O₂ alone
rations the jar. J5 HELD — LAI peaks at 0.60 derived / 0.67 stored against a threshold of 6;
mutual shading is measured uninvolved in the jar.

**What the cause is, as far as this control reaches:**

* Live O₂ removed the jar's kinetic floor. The jar starts with 2 mol O₂ in 1000 (0.2 %), so
  under `LivePool` there is almost no photorespiration and the crop pulls CO₂ to **~7 ppm**;
  under `Constant` it stops near **~70 ppm**, and a step never draws more than 28 % of the pool.
* That leaves the **reference** jar drawing **76 %** of its CO₂ in one step at step 777 — and
  the lab form's worst step is the **same step**, drawing 115 %. The leaf form does not create
  the squeeze; it tips it, **from both sides**. At step 777 (the probe's `squeeze` rows):

  | cell | CO₂ demand (mol) | CO₂ held (mol) | ratio |
  |---|---|---|---|
  | frozen × `LivePool` | 1.2302e-2 | 1.6258e-2 | 0.757 |
  | lab × `LivePool` | 1.4140e-2 (**+15 %**) | 1.2333e-2 (**−24 %**) | 1.147 |

  So the crop asks for somewhat more *and* the pool arrives lower — the lab jar is already
  drawn down 20 steps earlier (held 1.68e-2 vs 2.23e-2 at step 757). By logs, the lower pool is
  about two thirds of the ratio's rise. ⚠ Corrected in review (advisor, 2026-09-29): the first
  draft said *"~50 % more demand"*, dividing two ratios whose denominators differ — the
  ratio-vs-quantity mix this record's own P7 grading warned about.
* ⚠ **Not established:** that the extra leaf area (the ceiling-sitting stored LAI) is the part
  of the form that adds the demand or draws the pool down early. It is what the probe *saw* at
  every firing, not a control.

**⚠ THE FINDING THAT OUTLIVES THE LEAF FORM: on the reference run, step 777 withdraws 76 % of
the CO₂ it starts with, and no gate measures that distance.** What was searched, since
*"nothing watches it"* was first written unsearched (advisor, 2026-09-29) and this project has
retracted that sentence once before:

* **The jar's trough IS pinned.** `margins::the_five_margins_are_pinned_not_merely_positive`
  (`science_gates.rs`) holds `sealed_chamber` at **×10.674948** within a 2 % relative
  tolerance, so a change that moves the jar's CO₂ minimum reddens it. But the pinned quantity is
  the distance to the **compensation floor**, which live O₂ put near 0.7 ppm — a different
  limit from the one that fires here.
* **`rationed == 0`** is the only gate on overdraw, and it is binary: it reads the same at 0.28
  as at 0.76.
* **Grepped `rust/crates`** for `scale_factors` (two callers: `station/src/inspection.rs`, a
  display read, and this probe) and for any headroom / draw / demand assertion on
  `biosphere.carbon_pool`: **none bounds one step's draw.**

So a re-pin of the ×10.67 margin after some future change would read as "still ten times clear"
while the step margin could be gone. ⚠ What is NOT claimed: that any change adding ~32 % demand
would ration the jar — added demand also lowers the pool earlier and the crop's demand falls as
CO₂ falls, so the threshold is not that arithmetic. Recorded, not acted on — **whether the step
margin should be pinned is the user's call.**

**ANSWERED 2026-09-29 — PINNED** (user: *"Add a test that tracks how close the jar's worst step
comes to running out of CO2"*): `science_gates::margins::the_jars_tightest_co2_step_is_pinned_by_its_headroom`,
0.756662 at step 777, tolerance ±2 % of the HEADROOM (`1 − draw`), instrument `readouts::step_draws`.
Detail: `log/leaf-rust-remeasure.md`, last section.

**Standing:** unchanged. The lab form is still not adopted, nothing frozen moved, and whether
the rewrite continues is the user's decision — now with a named cause for the jar.

## 6. Which part of the leaf form squeezes the jar? (opened 2026-09-29, user: *"continue work on the leaf mechanism"*)

§5a named the interaction (leaf form × live O₂) and left one thing open: whether the extra leaf
area — the stored area sitting 1.18× above what its carbon implies at every firing — is what
adds the demand and draws the pool down early. This section is that diagnostic. **It changes no
number of the mechanism, adopts nothing, and is not a search for a fix.**

⚠⚠ **Written before any result: nothing measured here licenses narrowing the envelope.** The
one whole-run cell below caps the stored area at the carbon-implied value — which is,
structurally, the retune `log/leaf-expansion.md` finding 9 refused. It is run as a CONTROL, to
locate the squeeze; if it quiets the jar, that is a fact about where the squeeze lives, not a
candidate. The envelope's numbers are [E]'s, cited, and stay.

**What each outcome would change** (the decision is continue / park / refuse; adoption and
retune are both off the table, so a cell earns its run only if an outcome moves that decision):

* If the squeeze is the **late extra area** (removing it after the leaf-growth cutoff quiets the
  jar): the break is a property of how the form couples to the jar's near-empty CO₂ late in the
  season — a continue-case question about the form's senescence/area coupling, which is the
  part the branch mirrored without re-deciding (§2.4's shading choice lives there too).
* If the **early head start alone** still rations (the cap after the cutoff does not quiet it):
  the break is the seedling phase growing a bigger plant that empties the pool sooner — i.e. any
  form that speeds early growth breaks this jar. That leans toward park: the jar's live-O₂ margin
  (0.757, now pinned) is the binding fact, not this form.

**Instrument:** `examples/jar_squeeze.rs`, one season (all five firings are in season 1), Euler,
plus RK4 status on the capped cell. The cap is a wrapper around the leaf-area process built in
the example only (same aux id); the probe's firing count is asserted equal to the integrator's.

1. **Timeline, at fixed checkpoints** (the leaf-growth cutoff crossing, anthesis, steps 757 and
   777, and each firing): stored vs carbon-implied area, **whether the envelope clamp bound this
   step and from which side**, lab vs frozen CO₂ and O₂ pools.
2. **Local split at step 777** (demand only — the numerator, never demand ÷ held): the lab
   state's CO₂ demand, then with the stored area swapped to carbon-implied, then additionally
   the CO₂ pool swapped to the frozen run's value, then the O₂ pool too; the frozen state's
   demand is the residual's anchor. Reported in **both swap orders** for area vs CO₂, because
   assimilation is not additive in them.
3. **Whole-run cell "capped after the cutoff":** from the first step at/after the leaf-growth
   cutoff, the stored area is held ≤ carbon-implied. The head start's extra carbon and early
   drawdown stay; the late extra area goes. The one-step area drop at the cutoff is printed.

**Predictions, written BEFORE the first run:**

| # | Claim | Predicted |
|---|---|---|
| Q1 | Which side the clamp binds from late in season 1 (steps 757–789) | the **ceiling, pressing DOWN**: after the cutoff nothing grows the excess, so a ratio pinned at 1.176 means leaf carbon is falling faster than the area decays (advisor, 2026-09-29 — I had described it to the user as area "held at" the ceiling, which reads as reached from below) |
| Q2 | When the excess area is made | before the cutoff (the early seedling phase); at the cutoff crossing the ratio is already ≥ 1.1 |
| Q3 | Area swap at step 777 | removes **most** of the lab's demand excess over frozen: canopy interception at LAI ≈ 0.5 is near-linear, so 1.18× area ≈ +15 % interception, about the whole +15 % |
| Q4 | Adding the CO₂ swap | **raises** demand (the lab pool is 24 % lower; at ~10 ppm assimilation is near-linear in CO₂) — so after both swaps the lab state asks for MORE than frozen, the bigger plant's residue |
| Q5 | O₂ swap | small (< 5 % of demand): both jars sit near 0.2 % O₂ |
| Q6 | "Capped after the cutoff" | **still rations** (low confidence): the pool arrives 24 % lower at step 777 on the head start alone, and the frozen jar already draws 0.757 |

### 6a. Outcome — MEASURED 2026-09-29; the squeeze is in the SEEDLING phase, and my one whole-run cell could not see it

Instrument: `cargo run --release -q -p domains --example jar_squeeze` (committed). Its step-777
values reproduce §5a's exactly (demand 1.4140e-2 / 1.2302e-2, held 1.2333e-2 / 1.6258e-2), and
its firing count is asserted equal to the integrator's every step.

**⚠ The design's premise was wrong, and the run said so in its first line.** The leaf-growth
cutoff (`tuEMRTLM`, 724 °C·d) falls at **step 902, day 225.5** — thirty days AFTER the squeeze
(days 193–197). Both the "capped after the cutoff" cell and Q1's reasoning assumed the squeeze
came after it (so did the advisor's review of the design). **The cell is uninformative by
construction**: the cap never engages before the firings, which come out identical (5, RK4 at
773). It is not graded as "Q6 held".

**The timeline** (lab ÷ frozen unless marked):

| day | stored ÷ carbon-implied area | clamp | CO₂ pool, lab / frozen (mol) | plant C, lab vs frozen |
|---|---|---|---|---|
| 150.0 | 1.181 | ceiling | 1.53 / 0.833 | |
| 162.5 | 0.951 | – | 1.27 / 0.410 | **−36 %** |
| 168.75 | 0.792 | – | 0.889 / 0.0129 | |
| 175.0 | 0.786 | – | 0.698 / 0.0208 | −16 % |
| 181.25 | 0.721 | – | 0.0328 / 0.0225 | |
| 189.25 | 1.067 | – | 0.0168 / 0.0223 | **+21 %** |
| 192.25–197.25 | 1.18 | **ceiling, every step** | 0.0149→0.0103 / 0.0191→0.0142 | +18.5 % (day 194.25) |

Clamp census over season 1: before the cutoff, ceiling 143 steps, floor 132, free 627; after
it, free 318.

**What happens, as far as this measures:**

1. **Early, the lab crop is SMALLER — from the first weeks, and mostly in CARBON.** ⚠ Corrected
   in review (advisor, 2026-09-29): the first draft said *"the seedling rule holds area below its
   carbon (0.72–0.79, days 169–181), so it intercepts less and grows slower"*. That compared the
   lab's area with its OWN carbon, not with the reference crop's, and the window it cited comes
   AFTER the deficit it was meant to explain (−36 % plant carbon already at day 162.5). With the
   reference's area added (`froz LAI`, `area/fr`, `leafC/fr` columns): at day 100 the lab canopy
   is **0.44×** the reference's, of which **0.55 is leaf carbon** and 0.79 thickness. The floor
   (thickest leaf) binds intermittently, about twice a day, from **day 2.75 to day 79.75** — the
   seedling rule would have held area lower still. That fits an early thick-leaf phase
   compounding into a carbon deficit over ~80 days; **it is not tested as the cause** (it would
   take a control). The frozen crop empties the jar's CO₂ by ~day 169; the lab crop leaves it
   there until ~day 181.
2. **At the squeeze, it is BIGGER** (+18.5 % plant carbon, +30 % leaf carbon at step 777). The
   jar's total carbon is identical in both (4.017 mol); the difference is WHERE it sits — the
   frozen crop had already shed more to the soil (2.42 vs 2.13 mol) — consistent with it having
   grown earlier, not tested as the cause.
3. **During the squeeze, leaf carbon is FALLING** (carbon-implied area 0.478 → 0.452 over days
   192–197) in a CO₂-starved jar, while the seedling rule — which reads thermal time, not carbon
   — keeps pushing area UP. Only the thin-leaf ceiling (1.18×) holds it.

**The split at step 777 (CO₂ demand, mol per step):**

| state | demand | vs frozen |
|---|---|---|
| lab | 1.4140e-2 | +14.9 % |
| lab, area → carbon-implied | 1.1492e-2 | −6.6 % |
| lab, CO₂ pool → frozen's | 1.9772e-2 | +60.7 % |
| lab, area AND CO₂ swapped | 1.6256e-2 | +32.1 % |
| … AND O₂ swapped | 1.6258e-2 | +32.2 % |
| frozen | 1.2302e-2 | — |

Area effect −18.7 % (at the lab's CO₂) / −17.8 % (at frozen's); CO₂ effect +39.8 % / +41.5 %;
interaction 1.012 — the two nearly multiply. The residual +32 % matches the +30 % leaf carbon.
**With the extra area removed alone, step 777 would draw 0.93 of its pool** (1.1492e-2 ÷
1.2333e-2): locally, the ceiling-held area is what tips the step past 1. ⚠ That is a one-step
counterfactual, not a run: it does not say a run without the excess would stay unrationed,
since the excess also shapes the pool's path into step 777.

**Graded as written:**

* **Q1 — the SIDE held, the REASON falsified.** The ceiling binds on every step 769–789 and it
  presses down on a rising area as leaf carbon falls. But it is not "after the cutoff, nothing
  grows the excess": the seedling rule is still growing it.
* **Q2 — FALSIFIED as a story.** "Before the cutoff" is trivially true (everything is), and the
  ratio at the cutoff is 1.179 ≥ 1.1. But there is no early head start: the lab crop is BEHIND
  early, and the squeeze-time excess is made in days 187–192 (ratio 0.975 → 1.18).
* **Q3 HELD** — the area swap removes 18.7 %, more than the whole +14.9 % excess.
* **Q4 HELD** — the CO₂ swap raises demand ~40 %; with both swapped the lab asks +32 % more.
* **Q5 HELD** — O₂ swap +0.01 %.
* **Q6 — UNINFORMATIVE BY CONSTRUCTION** (above).

**What this means for the decision, in the terms §6 set:** neither branch fits as written — the
squeeze is neither "late extra area after the cutoff" nor "an early head start". It is the
**seedling rule running on thermal time while the crop is carbon-starved**: area keeps rising
as leaf carbon falls, and the envelope's ceiling is the only coupling between them in that
window. **The open question — does [F]'s seedling phase limit area by carbon supply? — is ANSWERED BY
OUR OWN RECORD, not by a source check** (searched 2026-09-29 on the advisor's prompt, before
calling it a PDF job; the shelf lesson, again). `log/leaf-expansion.md`:

* **Finding 2:** [F] Ch. 9 is scoped to *"non-limiting water and nutrients"* and has no
  mechanism by which the atmosphere runs out of carbon — **below the cutoff there is no carbon
  feedback whatsoever.** The port omitted nothing; the gap is [F]'s.
* **Finding 3:** the one carbon-supply rule [F] mentions (a per-day `min(node, carbon)` rate,
  p. 103, describing Boote et al. 1998) was built and **ratchets into a death spiral**.
* **Findings 4–5:** the [E] envelope was chosen **deliberately as the stand-in** for that missing
  feedback. So what §6a measured is that stand-in's ceiling (1.18×) binding in the one window
  where carbon is scarcest.
* **Finding 10:** [E]'s own thickness mixture, integrated on the four carbon-limited runs, never
  goes thinner than nominal (area ≤ carbon-implied) — the lab jar's 1.18× is outside what it
  reaches there. ⚠ And a ceiling derived from that was **already priced as landing inside the
  refused retune window** (1.058× from `open_season`); it is recorded here as context, not as a
  route.
* **Finding 11:** August's jar wall was the jar's carbon turning over faster than one step — the
  quantity the new pin now watches (0.757 of the pool per step on the reference).

**Standing:** unchanged — lab-only, nothing frozen moved, no retune, nothing adopted. Continue /
park / refuse is still the user's decision.
