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
  the squeeze; it is the ~50 % more demand at the squeeze's tightest instant that tips it.
* ⚠ **Not established:** that the extra leaf area (the ceiling-sitting stored LAI) is the part
  of the form that adds the demand. It is what the probe *saw* at every firing, not a control.

**⚠ THE FINDING THAT OUTLIVES THE LEAF FORM: the reference jar's rationing headroom is 1.32×,
and nothing watches it.** The jar's science gate reads **×10.67** above its compensation floor
(`log/o2-form-adopted.md` §2) — true, and a distance to a *different* limit. The limit that
fires here is one Euler step overdrawing a small pool, and the only gate on it is
`rationed == 0`, which is binary: it reads the same at 0.28 as at 0.76. Since 2026-09-07 any
change that adds ~32 % to the crop's CO₂ demand at day 194 of the jar rations it, whatever that
change is for. Recorded, not acted on — **whether that headroom should be pinned is the user's
call.**

**Standing:** unchanged. The lab form is still not adopted, nothing frozen moved, and whether
the rewrite continues is the user's decision — now with a named cause for the jar.
