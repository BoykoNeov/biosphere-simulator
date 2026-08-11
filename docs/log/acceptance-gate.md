## **The acceptance gate** (the chamber-scale diagnosis's own conclusion, measured)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED 2026-08-09, read-only — the collision three separate diagnoses bottomed out on,
now a number.** `docs/plans/post-roadmap-acceptance-gate.md`; probes
`M:/temp/acceptance_gate/`; 46 pins in `tests/test_acceptance_gate.py` (8 slow). **The
metric is the gate's OWN arithmetic, not one invented here**: `arbitration._scale_factors`
computes `scale_s = available_s/demand_s` and fires below 1, so the run's minimum of that
ratio *is* "how close the backstop came", with 1.0 the firing point by construction (tied to
the shipped code by a synthetic pin, not by prose). Roster from the **two manifests**
(7+13=20, of which 19 have a trajectory — `drift_summary` is named as derived, not omitted),
each scenario driven by its **own committed golden runner** rather than a re-derived one.
**FINDING 1 — the empty cell, sharper than "unfalsifiable": `open_season`'s carbon source
`boundary.co2_atmos` is `unclamped` AND HOLDS 0.0 mol C FOR THE WHOLE RUN** (a BOUNDARY
source is a ledger entry, not a reservoir), so `assert rationed == 0` in the season golden
is true by construction w.r.t. carbon — it reports that no flow out-ran a *tissue* pool,
never that the crop was fed. ⚠ **The obvious fix is refused TWICE**: an open field genuinely
*has* an unclamped atmosphere (so a finite pool is a scenario substitution, not a fix),
**and** it would not gate anyway — `sealed=False` keeps the Phase-1 **constant `ci`
forcing**, so nothing reads the pool; a gate needs a clamped stock *and* a draw that depends
on it. **FINDING 2 — the qualifier HELD, measured not inherited** (the committed sentence
says no *carbon*-scarcity gate, and letting that widen is this repo's most-repeated shape):
`soil_water` **189.24**, `soil_n` **126,238.75** — live but slack by 2 and 5 orders, i.e.
carbon's reason is *structural* and theirs is *quantitative*, recorded apart. Underneath:
scarcity is *designed* to act through `f_water`/`f_N`/`f_O2`, and `arbitration.py`'s first
paragraph says so — a biting drought never moves `rationed`. **FINDING 3 — on
donor-controlled stocks the gate is a `dt` CHECK, not a scarcity check**: margin ≡
`1/(Σk·dt)`, measured with the constant named — `water_vapor`/`condensate` **2.0**=1/0.5,
`litter_carbon`/`litter_n` **90.909**=1/0.011 (`decomposition_rate`), `cabin_co2`
**16.667**=1/0.06, `cabin_h2o` **33.333**=1/0.03 — so there `rationed` answers **"is my
timestep safe?"**, not "is the resource short?". ⚠ **Scope, corrected from a draft that
overreached**: the multi-rate build-time `k·h<1` precondition lives in
`authoring.interpreter._effective_step` and fires on an **authored file**; the frozen
scenarios are built directly in Python and never reach `interpret` (checked), so on the
frozen roster this is the *only* check of that inequality — the quantity checked is still
the timestep, but it is **not a redundancy here**. Two param files say it themselves
(`water_cycle.yaml`: "rate·dt < 1 keeps the backstop off"; `eclss.yaml` prints both
products). ⚠ **The flatness test is a RELATIVE-SPREAD test and exact equality was WRONG**:
`x/(k·x)` is `1/k` algebraically but not bit-stable across `x` — `water_vapor` lands on
exactly 2.0 only because 0.5 is **binary-exact**, while `litter_carbon`'s 0.011 wobbles by
**one ULP** ⇒ `min == max` files the decomposer's litter pool as a *scarcity* gate on a
rounding artefact, the exact error the census exists to avoid. **Free corroboration of
option (B)**: `microbial_n`'s margin is **bit-identical** to `microbial_carbon`'s and
`litter_n`'s matches `litter_carbon`'s to 1 ULP — (B)'s "both currencies leave on the same
flux" falling out of a measurement not looking for it. **FINDING 4 — EXACTLY ONE STOCK IN
THE ROSTER IS A BINDING GATE.** Rank every live margin across all 20: **the six smallest are
all `biosphere.carbon_pool`** (1.126, 1.126, 1.491, 1.802, 1.802, 5.218), and the first
margin on any other stock is `sealed_chamber`'s `o2_pool` at **8.944** — a chamber
documented as deliberately O₂-poor, so even the runner-up is a chamber property; tightest
outside the biosphere is `power.battery` **11.086**. ⚠ Stated as a **rank with NO
threshold**: my draft asserted "every margin below 9.0", a cut chosen by eye that lands
*between* the 6th and 7th — **and it was off by one**, because the runner-up is 8.94; a
number invented to separate two measurements is the fitted comparison this project refuses.
⚠⚠ **"LIVE" is LOAD-BEARING and the unqualified sentence is FALSE** (advisor): rank *all*
classes and the 6th is the **water-cycle pair at 2.0**, between the 5th live entry (1.802)
and the 6th (5.218) — the meta-finding's shape exactly, so the **exclusion itself is
pinned** (raw 6th asserted by value AND class) and the test carries `LIVE` in its **name**,
a name being the most-quoted paraphrase there is. ⚠⚠ **The gate fires per registry CALL, not
per step** — checked because it could have moved this finding:
`greenhouse`/`harvest`/`lighting`/`sealed_station` step one `State` through **two**
registries and **six stocks incl. `carbon_pool` are drawn by both**, so a margin is the
tightest *call*, not a day's headroom. greenhouse/harvest's two registries **coincide**
(16.667 either way), `lighting` shares none (control), and **`sealed_station`'s 5.218 comes
from the BIOSPHERE registry** (cabin's own min on that stock: 16.667) ⇒ a genuine plant-side
draw, the same quantity the standalone chambers measure — **had it come from the fast side
that row would not belong in the ordering claim at all**. **Tripling the horizon does not
tighten it** — `perennial_long_horizon` is `0x1.20430fa48d229p+0`, *bit-identical* to the
5-yr run (the margin is **not** a t=0 property and had to be run, unlike chamber-scale's
inventory): the minimum is reached inside the first five years. ⇒ **the chamber-scale claim
is confirmed with a number**: the acceptance test every biosphere science change has
actually been judged by is **one stock in one rig at 11–80 % headroom**, while the
field-scale scenario the science is *for* sits **42× clear**, on a tissue pool.
`sealed_station`'s **5.218** (the same jar under the P6.3 cabin coupling, 4.6× slacker)
*suggests* the binding-ness is a property of **isolation** rather than of chambers — ⚠
recorded as an inference, since the margin was measured and the mechanism was not isolated.
**FINDING 5 — the contract has NO plausibility column**: a manifest scenario entry carries
only `scenario`/`golden`/`golden_sha256`(/`years`), so the frozen acceptance set is {golden
bytes, `rationed == 0`, no extinction, conservation, determinism} — **every one a property
of the RUN, none a property of the SCIENCE**. Bands *do* exist (`test_senescence_form.py`'s
`5 < peak LAI < 8`, `test_nitrogen_form.py`'s 14.4248 t/ha crossing) but are **named by no
manifest** (grepped, not asserted) and were authored *after* the science they judge ⇒
records, not gates. **FINDING 6 — THE TWO GATES DISAGREE, and adjudicating is deliberately
NOT this work's call**: (C)'s full form is pinned `rationed == 0` on `perennial` under
**Euler** — Euler at `dt=1` *is* the frozen configuration, and `perennial` is exactly where
(C) died under RK4 — while its peak LAI is pinned above 15 (**16.40**) vs the ~5–8 band
(what caught it was RK4, *not* the crop's frozen integrator); the canopy regulator is
**bit-identically inert** at `to_bits()` across `_ROSTER`'s 6 scenarios (gate silent) while
flipping that canopy 16.40→**6.24**. ⚠ **Both cite the PINS, not this table**: my draft
wrote "in all 8" for both, inherited from the status rows above, where the committed
assertions cover **6** and **1** — the conclusion is unchanged, but the count came from a
paraphrase. ⚠ **Stem-only is NOT a third disagreement and my draft said it was**: the gate
refuses it (`perennial` 0→1) and peak LAI **4.985** lands **0.3 % OUTSIDE the 5–8 band on
the low side** — both refuse. The draft called that an "improved" canopy reading by silently
swapping *which band was meant*, the mass-vs-area conflation `test_senescence_form.py`
already pins as having bitten this repo twice, committed a third time and caught by
**reading the pin instead of the summary**. Which gate is authoritative is a **contract**
question — using a criterion authored here to reverse a measured refusal is the
consumer-chamber-2× / DPM-RPM / ruling-B shape. **Priced, not proposed — the crew-coupled
route** (the chamber-scale seam's standing alternative), in the census's own unit: the
sealed jar is **2.01 days** of one m² of wheat ([BVAD] 1.7496 mol C/m²·d, imported from
`test_chamber_scale.py` not re-derived), `crew.food_store`'s 4000 mol C is **2,286 days ≈
6.26 YEARS**, and the loop back exists (`station.harvest`: `storage_c → food_store`). ⚠ But
those scenarios run **7 days with a seedling**, are station-side and non-frozen — the
inventory is available, **the demonstration is not**, and building it is *authoring*. **A
RUNTIME COST, MEASURED THEN MOSTLY FIXED — and a per-process cache is a PER-WORKER cache**:
the census can't reuse the session-scoped `sealed_tier2_run` (it needs the *arbitration
calls*, not the states), and the first version spread 4 roster-wide slow claims across 4
test functions ⇒ under `--dist load`, **up to 4 Tier-2 recomputations** — measured clean at
`-n 12`: **22m34s**. Both routes to worker affinity were already closed by
`docs/test-suite-runtime.md`, so the lever is **fewer tests that need it**: merged into one
function with a labelled message per claim ⇒ **6m47s**, **3.3×**, deleting no coverage, only
test *boundaries*. (Ratio from a back-to-back pair; deliberately **not** compared to that
doc's 4m33s baseline from another day.) No value/golden/param/manifest moved; `git diff
src/` empty; nothing unfrozen; full suite **2099 passed**.
