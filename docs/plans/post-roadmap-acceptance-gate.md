# The acceptance gate — what the frozen contract can actually refute

**Status: DIAGNOSED 2026-08-09, read-only. No `src/` change, no golden moved, no
manifest touched, nothing unfrozen.** Probes in `M:/claud_projects/temp/acceptance_gate/`;
pins in `tests/test_acceptance_gate.py` (46 cases, 8 of them `slow`).

The chamber-scale diagnosis measured a great deal, and then closed on one claim it stated
rather than measured:

> The defect is not the chamber's size. It is that the frozen contract uses this rig's
> closure gate (`rationed == 0`) as the acceptance test for FIELD-scale plant science.
> […] `open_season` carries no **carbon-scarcity** gate at all: its CO₂ source is an
> unclamped boundary stock, so a carbon rationing assertion is unfalsifiable there.

Three separate pieces of work — scope (A) finding 11, canopy-regulator finding 4, and
(C) finding 8 — bottomed out on that collision from different sides. This document
measures it instead of restating it.

---

## Method

**The metric is the gate's own arithmetic, not one invented here.**
`simcore.arbitration._scale_factors` computes, per clamped stock,
`scale_s = available_s / demand_s`, and the backstop fires when it drops below 1. The
minimum of that ratio over a run is therefore exactly *how close the gate came to firing
on that stock*, in the gate's own units, with 1.0 the firing point by construction. Tied
to the shipped code by `test_the_margin_is_the_gates_own_scale_factor`, not by prose.

**Each scenario is driven the way its own golden drives it.** The runners are imported
from the committed `test_regression_*` modules rather than re-derived — the nitrogen
work had to correct a `run_season`-vs-`run_perennial` mix-up twice, and re-deriving 19
runners is that hazard by construction.

**The roster comes from the manifests, not from a list typed here.** 7 biosphere + 13
station = 20, of which 19 have a trajectory (`drift_summary` is the stability signature
of two runs already in the roster; named and pinned as derived rather than left absent).
Three prior findings in this project were *a scenario list checked against its own length
instead of against the frozen set*; a census is exactly that shape.

⚠ **The gate fires per registry CALL, not per simulated step** — checked because the
answer could have moved finding 4. `greenhouse`, `harvest`, `lighting` and
`sealed_station` step one `State` through **two registries**, so `min_scaling` runs many
times per simulated day (the fast cabin/power registry) plus once (the slow biosphere
registry), each call seeing only its own flows' demand. A recorded margin is therefore
the tightest *call* — exactly what the backstop protects — and **not** headroom against a
day's total draw. Six stocks including `carbon_pool` are demanded by *both* registries in
`greenhouse`/`harvest`; measured, the two registries' minima there **coincide** (16.667
either way), so no census number is affected. `lighting` shares nothing across its
registries and is the control. `sealed_station` is the one case where they differ, and it
matters — see finding 4.

**Classification.** Per stock:

| class | meaning |
|---|---|
| `impossible:boundary` | `unclamped` — arbitration *skips* it (decision #13). No gate can exist here at any value. |
| `impossible:never-withdrawn` | clamped, but nothing ever draws from it. |
| `rate-determined` | the margin is flat over the whole run ⇒ demand ∝ stock ⇒ margin ≡ `1/(Σkᵢ·dt)`: a property of the timestep, not of scarcity. |
| `live` | trajectory-dependent. |

⚠ **The flatness test is a relative-spread test (≤ 1e-12), and exact equality was wrong.**
`margin = x/(k·x)` is algebraically `1/k` but not bit-stable across different `x`:
`water_vapor` lands on exactly 2.0 at every step only because its rate 0.5 is
binary-exact, while `litter_carbon`'s 0.011 is not and its margin wobbles by **one ULP**
(spread 2.2e-16). The first draft classified by `min == max` and thereby filed the
decomposer's litter pool as a *scarcity* gate on a rounding artefact — the exact mistake
the census exists to avoid. The cut sits ~3.7 orders above the observed ULP wobble
(2.2e-16) and ~6.7 orders below the smallest genuine variation measured
(`microbial_carbon`, 4.6e-06) — a ~10.4-order gap with the cut inside it, so its exact
placement is not load-bearing.

⚠ **The converse is not claimed**: a varying margin is not thereby a scarcity margin.
`microbial_carbon` varies only because `f_O2` does.

---

## The census

Tightest live gate per frozen scenario (minimum of `available/demand` over the run):

| scenario | tightest live stock | margin |
|---|---|---|
| `perennial_chamber` | `biosphere.carbon_pool` | **1.126** |
| `perennial_long_horizon` | `biosphere.carbon_pool` | **1.126** |
| `sealed_chamber` | `biosphere.carbon_pool` | **1.491** |
| `consumer_chamber` | `biosphere.carbon_pool` | **1.802** |
| `consumer_long_horizon` | `biosphere.carbon_pool` | **1.802** |
| `sealed_station` | `biosphere.carbon_pool` | 5.218 |
| `power_self_discharge` | `power.battery` | 11.086 |
| `power_bounded_soc` / `station_heat_closure` / `sealed_energy_drift` | `power.battery` | 11.295 |
| `lighting` | `biosphere.carbon_pool` | 14.443 |
| `greenhouse` / `harvest` | `biosphere.carbon_pool` | 16.667 |
| `eclss_steady_state` | `eclss.cabin_o2` | 33.333 |
| `cabin_gas` / `water_recovery` | `eclss.cabin_o2` | 35.573 |
| `open_season` | `biosphere.leaf_c` | 42.507 |
| `thermal_equilibrium` | `thermal.node` | 257.681 |
| `crew_mission` | `crew.food_store` | 388.556 |

---

## Finding 1 — the empty cell, and it is sharper than "unfalsifiable"

In `open_season` the crop's carbon source is `boundary.co2_atmos`. It is `unclamped`, so
arbitration skips it by design — **and it holds 0.0 mol C for the entire run**, because a
BOUNDARY source is a ledger entry, not a reservoir. Every clamped CARBON stock in the
scenario is either the plant's own tissue (`leaf_c`, `stem_c`, `root_c`) or a
never-withdrawn sink.

So `assert rationed == 0` in `tests/test_regression_season.py` is true by construction
with respect to carbon. It reports that no flow out-ran a *tissue* pool — an integration
overshoot check on a state variable — and never that the crop was well fed. The season
golden's own comment calls this line "the Step-11 `rationed == 0` invariant"; the
invariant is real, it is simply not about carbon supply.

**The obvious fix is refused, for two reasons, and the second is the interesting one.**

1. An open field genuinely *has* an unclamped atmosphere. Modelling it as finite would
   model a different system — the tempting "give `open_season` a CO₂ pool" is not a fix
   but a scenario substitution.
2. Even a finite pool would not create a gate, because open-field assimilation does not
   read the pool at all: `sealed=False` keeps the Phase-1 **constant `ci` forcing**, and
   only `sealed=True` swaps in `chamber.ci_from_co2_pool`. A gate needs both a clamped
   stock *and* a draw that depends on it.

---

## Finding 2 — the qualifier holds, and the reasons are different per currency

The committed sentence says no **carbon**-scarcity gate. Measured rather than inherited,
because letting that widen into "no scarcity gate" is this repo's most-repeated failure:

* `biosphere.soil_water` — live, min margin **189.24**. So water never comes within two
  orders of the gate in the frozen open-field run. (The documented drought cascade is a
  *perturbation* scenario, outside the frozen roster.)
* `biosphere.soil_n` — live, min margin **126,238.75**. Nominal by construction: the
  scale is the documented non-physical one (`soil_n0 = 100` against `sn_critical = 50`).

Two different reasons — carbon's is *structural* (no clamped source exists), water's and
nitrogen's are *quantitative* (the source exists and is slack by 2 and 5 orders) — and
they are recorded apart.

⚠ Note the design point underneath all three: scarcity in this model is *meant* to act
through the limitation factors (`f_water`, `f_N`, `f_O2`), not through the backstop.
`simcore/arbitration.py` says so in its first paragraph — "a *rare numerical guard*, not
the ecological mechanism". A biting drought throttles assimilation without ever moving
`rationed`.

---

## Finding 3 — on donor-controlled stocks the gate is a `dt` check, not a scarcity check

Where total demand is proportional to the stock, the margin is identically `1/(Σkᵢ·dt)`.
Measured, with the constant identified in each case:

| stock | margin | `1/margin` | the constant |
|---|---|---|---|
| `biosphere.water_vapor` | 2.0 | 0.5 | `condensation_rate` |
| `biosphere.condensate` | 2.0 | 0.5 | `recycling_rate` |
| `biosphere.litter_carbon` | 90.909 | 0.011 | `decomposition_rate` |
| `biosphere.litter_n` | 90.909 | 0.011 | `decomposition_rate` (option (B): the same flux) |
| `eclss.cabin_co2` | 16.667 | 0.06 | `co2_scrub_rate · dt` |
| `eclss.cabin_h2o` | 33.333 | 0.03 | `condense_rate · dt` |

Such a stock can only ration when `k·dt > 1`. So on these stocks `rationed` answers *"is
my timestep safe?"*, not *"is the resource short?"* — a real question, and not the one the
contract reads it as answering. Two param files say as much themselves: `water_cycle.yaml`
("rate·dt < 1 keeps the backstop off") and `eclss.yaml` (which prints both products, 0.06
and 0.03).

⚠ **Scope, corrected from a draft that overreached.** The multi-rate work's build-time
`k·h < 1` precondition lives in `authoring/interpreter._effective_step` and fires when an
**authored scenario file** is interpreted. The frozen scenarios are built directly in
Python (`build_season`, `build_eclss`, …) and never pass through `interpret` — checked,
not assumed. So the draft's "the run-time gate is the same inequality reached the
expensive way" is true **for authored content only**; on the frozen roster this is the
*only* check of that inequality. The finding is unchanged — the quantity being checked is
the timestep, not scarcity — but it is not a redundancy here.

**A corroboration from an unrelated direction.** `microbial_n`'s margin is *bit-identical*
to `microbial_carbon`'s in every sealed scenario, and `litter_n`'s matches
`litter_carbon`'s to 1 ULP. That is option (B)'s core identity — both currencies leaving
on the same flux — falling out of a measurement that was not looking for it.

---

## Finding 4 — exactly one stock in the roster is a binding gate

Rank every live margin across all 20 frozen scenarios. **The six smallest are the same
stock**, `biosphere.carbon_pool`, in the six scenarios that seal one: 1.126, 1.126,
1.491, 1.802, 1.802, 5.218. The first margin on any other stock is `sealed_chamber`'s
`o2_pool` at 8.944 — and that chamber is documented as deliberately O₂-poor, so even the
runner-up is a chamber property. The tightest gate outside the biosphere is
`power.battery` at 11.086.

⚠ Stated as a **rank**, with no threshold. An earlier draft asserted "every margin below
9.0", a cut chosen by eye that lands between the 6th and 7th entries — and it was off by
one, because the runner-up is 8.94. A number invented to separate two measurements is the
fitted comparison this project refuses; the ranking needs no cut.

⚠ **"live" is load-bearing in that sentence, and the unqualified version is false.** Rank
*all* classes and the 6th tightest is the water-cycle pair at 2.0, sitting between the 5th
live entry (1.802) and the 6th (5.218). The claim holds only after the rate-determined
exclusion — which is argued above and is sound, but this is precisely the shape this
project has watched five times: the careful sentence stays put and the paraphrase travels.
So the exclusion is itself pinned (the raw 6th entry is asserted, by value and by class),
and the test carries `LIVE` in its **name**, a name being the most-quoted paraphrase there
is.

**Tripling the horizon does not tighten the gate.** `perennial_long_horizon`'s margin is
`0x1.20430fa48d229p+0` — *bit-identical* to the 5-year `perennial_chamber`, and likewise
for the consumer pair. `test_chamber_scale.py` pinned that the long-horizon goldens reuse
the same scenario objects, so the *inventory* is bit-identical by construction at t=0;
the *margin* is not a t=0 property and had to be run. It comes back identical: the
minimum is reached inside the first five years. The horizon lengthens the run, not the
jar.

**So the chamber-scale claim is confirmed with a number.** The acceptance test that every
biosphere science change has actually been judged by is one stock in one rig, with 11–80 %
headroom — while the field-scale scenario the science is *for* is 42× clear of its
tightest gate, and that gate is on a tissue pool.

**`sealed_station`'s 5.218 is a genuine plant-side draw, and that was checked rather than
assumed.** It is a multi-registry scenario, so the row would not have belonged in the
ordering claim at all if its minimum had come from the fast cabin registry. Measured: the
**biosphere** registry's daily call produces 5.218, while the cabin registry's minimum on
that same stock is 16.667 — the `1/(k·dt)` value the `greenhouse`/`harvest` rows sit at.
So it measures the same quantity as the standalone chambers' 1.126 / 1.491 / 1.802.

The station-coupled chamber being 4.6× slacker than `perennial` — the same jar under the
P6.3 biosphere↔cabin seam — makes the binding-ness *look* like a property of isolation
rather than of chambers as such. ⚠ That causal half is an inference: the margin was
measured and attributed to a registry, but the mechanism was not isolated, and no pin
asserts it.

---

## Finding 5 — the contract has no plausibility column at all

A manifest scenario entry carries `scenario`, `golden`, `golden_sha256` (and, for the
biosphere, `years`). There is no field for "and the crop must be physical". So the frozen
acceptance set is:

> {golden bytes, `rationed == 0`, no extinction, conservation, determinism}

— every one of which is a property of the **run**, and none of which is a property of the
**science**.

Plausibility readings *do* exist in the tree: `test_senescence_form.py` pins
`5 < peak LAI < 8` for `open_season` against real wheat, and `test_nitrogen_form.py` pins
the 14.4248 t/ha Greenwood crossing. Both are literature-backed and both were written by
post-roadmap diagnoses. But neither is reachable from a manifest — asserted by grepping
both manifests for those filenames, not left to prose — so neither can fail an unfreeze
ceremony, and both were authored *after* the science they judge. They are records, not
gates.

---

## Finding 6 — the two gates disagree, and adjudicating that is not this work's call

Measured facts, from work already committed. **Two clean disagreements, and one case that
looks like a third and is not:**

* **(C)'s full DVS form**: `test_euler_reports_no_rationing_and_that_is_the_trap` pins
  `rationed == 0` on `perennial_chamber` under **Euler**, and Euler at `dt=1` *is* the
  frozen biosphere configuration — while `test_the_primarys_form_takes_the_canopy_
  unphysical_on_either_table` pins `open_season`'s peak LAI above **15** (16.40) against
  the ~5–8 band. The closure gate, run as frozen, passes it; the plausibility reading
  refuses it. What actually caught it was an RK4 run, which is not the crop's frozen
  integrator.
* **The canopy regulator**: **bit-identically inert** — at `to_bits()` over every stock at
  every step, pinned across the 6 scenarios of `test_senescence_form._ROSTER` — so the
  closure gate is silent, while it takes that same canopy from 16.40 to **6.24**, i.e.
  flips the plausibility verdict.
  ⚠ Both bullets cite the **pins**, not the summary. A draft of this section wrote "in all
  8 scenarios" for both, inherited from the `CLAUDE.md` status rows; the committed
  assertions cover 6 rows and 1 row respectively. The conclusion is unchanged — `perennial`
  is exactly where (C) died under RK4, so the Euler-clean pin *is* the disagreement — but
  the count came from a paraphrase, which is the failure this repo has recorded three
  times under a different name.
* **Stem-only senescence is NOT a disagreement case, and an earlier draft of this
  document said it was.** `perennial` goes `rationed` 0→1 under Euler, so the closure
  gate refuses it; peak LAI falls 5.191 → **4.985**, which moves *away* from the
  V-K&S shading threshold (6.0) but lands **0.3 % outside the ~5–8 realism band on the
  low side**. Both gates refuse it. The draft called that an "improved" canopy reading by
  silently swapping which band was meant — the mass-vs-area conflation
  `test_senescence_form.py` already pins as having bitten this repo twice, committed here
  a third time and caught by reading the pin instead of the summary. A 0.3 % margin is in
  any case too thin to carry a verdict either way.

⚠ **Deliberately not adjudicated here.** Which gate is authoritative is a *contract*
question. Using a criterion authored in this document to reverse a measured refusal would
be the co-adaptation shape this project has refused three times — the consumer-chamber 2×,
the DPM/RPM labile re-read, and ruling B. The finding is that they disagree; the decision
is the user's.

---

## Priced, not proposed — the crew-coupled route

The chamber-scale diagnosis named soil-carbon pool fractionation as *the* seam, with a
measured obstacle, and left the crew-coupled route standing as an aside. In the census's
own unit — days of one m² of wheat's CO₂ uptake, [BVAD] Table 4-91's 77.00 g CO₂/m²·d =
1.7496 mol C/m²·d:

* the sealed chamber's whole carbon inventory: **2.01 days**;
* `crew.food_store` in `greenhouse` / `harvest`: 4,000 mol C = **2,286 days ≈ 6.26 years**.

And the loop back into it exists in the tree already: `station.harvest` runs
`storage_c → food_store`, donor-controlled.

⚠ **What that does not show.** Those scenarios run 7 days with a seedling, are
station-side, and are outside the biosphere contract. No field-scale crop has ever been
run against that store. The inventory is available; the demonstration is not — and
building it is *authoring*, which under the pivot owes conservation + determinism, not
scientific endorsement. It is recorded here so the price is not re-derived.

---

## What this work does NOT claim

* Not that `rationed == 0` is a bad gate. It is a numerical backstop and it does that job
  correctly everywhere in the roster; `simcore/arbitration.py` has always said so. The
  finding is about the *duty it has been put to*.
* Not that the sealed chambers should be changed. Chamber-scale refuted resizing on
  independent [BVAD] grounds; nothing here reopens it.
* Not that a plausibility gate should be added to the frozen contract. Promoting a
  diagnostic into a contract is an unfreeze-shaped decision, and the bands available were
  authored after the science — that is worth saying out loud before anyone promotes one.
* Not a verdict on any refused change. See finding 6.

---

## Pins

`tests/test_acceptance_gate.py` — read-only, no fixture, nothing unfrozen:

1. the metric is `arbitration._scale_factors`' own `scale_s`, on a synthetic case that
   also checks an unclamped source imposes no constraint;
2. the roster is exactly the two manifests' scenario sets, with `drift_summary` named as
   derived from runs that are themselves in the roster;
3. `open_season`'s carbon source is unclamped **and** holds 0.0, and its only live carbon
   gates are the three tissue pools;
4. the "finite CO₂ pool" fix would not gate either — `sealed=False` keeps constant-`ci`
   forcing and builds no `carbon_pool`;
5. water and nitrogen are live-but-slack in `open_season` (2 and 5 orders), pinned
   separately from carbon so the qualifier cannot be dropped;
6. six first-order stocks' margins equal `1/(k·dt)` with the constant named;
7. the microbial and litter N/C margin identities (option (B), seen from outside);
8. the census table, per scenario, by stock and value;
9. the rank claim: the six tightest **live** margins in the roster are all
   `biosphere.carbon_pool`, the 7th is `sealed_chamber.o2_pool` at 8.944 — plus the raw
   all-class 6th entry (the water-cycle pair at 2.0, rate-determined), so the exclusion
   the claim depends on is a measured fact rather than an invisible filter;
10. the gate fires per registry **call**, the shared stocks in `greenhouse`/`harvest`
    agree across registries, `lighting` shares none, and `sealed_station`'s binding 5.218
    comes from the **biosphere** registry (cabin: 16.667) — without which that row would
    not belong in (9);
11. tripling the horizon leaves the margin bit-identical;
12. every frozen run stays above the firing point (the metric's consistency check against
    the gate it measures);
13. a manifest scenario entry has no plausibility field, and the bands that exist are
    named by no manifest;
14. the crew-coupled inventory in days-of-wheat, with the BVAD constant imported from
    `test_chamber_scale.py` rather than re-derived.

**A runtime cost — measured, then mostly fixed, and the fix is the finding.** The census
cannot reuse the session-scoped `sealed_tier2_run` trajectory: it needs the *arbitration
calls*, not the states, so Tier-2 must be re-run under the recorder.

The first version spread four roster-wide slow claims across four test functions.
`_CACHE` is per *process* and xdist's `--dist load` spreads tests across workers, so that
meant **up to four Tier-2 recomputations** — measured, on a clean back-to-back pair at
`-n 12`: **22m34s** for the whole suite. `docs/test-suite-runtime.md` had already closed
both routes to worker affinity (a collection-hook group is silently dropped; `loadgroup`
doubled the full run), so the remaining lever was *fewer tests that need it*: the four
claims were merged into one function, each with a labelled assertion message so a failure
still says which claim broke. Post-merge, the same command runs in **6m47s** — a **3.3×**
saving from deleting no coverage at all, only test *boundaries*.

⚠ The pair above is quoted as a ratio from two back-to-back clean runs, per
`docs/test-suite-runtime.md`'s own rule; it is deliberately **not** compared against that
document's 4m33s baseline, which was measured on another day.

Two things worth carrying: a per-process cache is a *per-worker* cache under xdist, so
"cached" does not mean "computed once"; and the number of expensive **tests** — not the
number of expensive scenarios — is what sets the bill.

Dropping `sealed_station` entirely would have been cheaper still, and would also have
removed one of the six rows finding 4 rests on. That cost was taken deliberately.
