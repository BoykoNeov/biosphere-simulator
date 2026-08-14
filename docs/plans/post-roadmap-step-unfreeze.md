# The step unfreeze — the biosphere moves to `dt = ¼` (authorized 2026-08-14)

**Status: PLANNED 2026-08-14, IN PROGRESS. Authorized by the user** after Step 0's two
measurement axes (`docs/log/co2-controller.md`, `docs/log/step-sweep.md`). The sweep
recommended `dt = ½`; **the user chose `dt = ¼`** — the same ceremony, with 4.8× headroom
instead of 2.1×, so the next mechanism added to the tree probably does not force a repeat.

This is the largest unfreeze this project has run. It follows the discipline in
`docs/biosphere-reference.md` §"The unfreeze discipline", and it is deliberately **split into
two commits** so the golden diff is attributable.

---

## 0. Why, in one paragraph

In a sealed chamber the crop draws the air's CO₂ down over the season. Below the CO₂
compensation point (`Γ*/ci_ratio = 61.07 ppm`) assimilation is exactly zero — a hard floor.
At the shipped step the model drives the sealed and perennial chambers **below that floor**
(57.89 and 56.03 ppm) and keeps fixing carbon anyway. It is a truncation error, not a
threshold crossing: the observable *converges* as the step shrinks (57.89 → 75.06 → 75.82 →
76.03 against an RK4 limit of 76.29). At `dt = ¼` every sealed scenario clears, at every CO₂
enrichment level, with `k·h` ≈ 0.18–0.21.

**The price, all three numbers** (sealed chamber, Euler, against the converged limit): the
season-low CO₂ error goes **−24 % → −0.6 %**, peak leaf carbon **+4.0 % → +0.7 %**, harvest
**−0.7 % → −0.1 %**.

---

## 1. ⚠ The step change is a FORCING-INDEX change, not a constant edit

**This is the part that makes it a design job rather than a search-and-replace.** Every
forcing in this tree is a pure function of the integer step count `n`, never of `n·dt`
(Step 0 axis 1, scope finding 1, audited at all five sites). So simply passing `dt = 0.25`
and `steps = 4·len(weather)` would feed the crop **the entire season's weather in the first
quarter-year** and then clamp on the final day for the remaining three quarters. The probe
sidestepped this by tiling the weather list; production cannot tile at 75 call sites.

Four things are indexed by `n` and each needs a rule:

| site | today | needs |
|---|---|---|
| `season.py:_table` (5 weather forcings + 3 constants) | `values[min(n, last)]` | index physical time |
| `run_perennial(..., year=)` — the annual reset | `len(weather)`, a step count | scale with the step |
| `biosphere/perturbations.py:window_override` | `start <= n < end` | scale with the step |
| `station/perturbations.py` | same | already writes `3 * _SPD` — the house pattern |
| ⚠ **leg-reconstruction helpers in ~34 test sites** | `resolver.bind(before, 1.0)` / `flow.evaluate(before, bound, 1.0)` | must use the same `dt` the engine used |

⚠ **That last row is a FIFTH class, found while routing on 2026-08-14 and not in the
original audit.** Several tests re-derive a step's flow legs by hand to check the ledger,
and they hard-code `dt = 1.0`. At `dt = ¼` a reconstruction at `1.0` disagrees with the
engine by 4× on every leg. They are **deliberately left for the step flip to expose**: the
ledger assertions fail loudly and name themselves, which is a better list than one I would
guess at across 34 sites, and a site that stays green is a site whose `1.0` was a genuine
unit-test constant rather than a season's step.

⚠ **"~34" is a LITERAL-SEARCH number and is wrong in both directions — advisor, 2026-08-14.**
It came from grepping the literal `1.0`, which is the same failure mode as the name search
that missed `year_len` in commit 1. It over-counts (most `.bind(state, 1.0)` hits are ordinary
unit tests that construct their own env at their own `dt` and never touch a run trajectory)
**and** it under-counts in a way a literal grep can never see: several sites already pass
`scenario.bio_dt` rather than `1.0` (`test_lighting_run.py:313,323`,
`test_crew_coupled_loop.py:487`, `test_greenhouse_run.py:198`, `test_harvest_run.py:268`,
`test_station_perturbations.py:284`) and so **follow the flip automatically** — invisible to a
`1.0` search, and correct already. Enumerate by **callee** — every `.bind(` and `.evaluate(`,
then read the `dt` argument — exactly the rule commit 1 paid for. The count is not load-bearing
either way: this class fails loudly, so the flip's own red list is the authority.

### ⚠ The SIXTH class — a period in DAYS used against a STEP index (fix in commit 1)

Found 2026-08-14 while auditing the routing's *arguments*, and **not findable by any test
at `dt = 1`**: a season length is `len(_weather())` — a **day** count — and it is used to
slice, modulo or size a **step**-indexed trajectory. Today the two numbers coincide, so
the whole suite is blind to it by construction.

**This class is more dangerous than the fifth, and must NOT be left for the flip.** The leg
helpers hard-code a literal and fail loudly naming themselves. A re-sow period four times
too short instead produces a *plausible* run — a crop re-sown quarterly, a 60-entry
"per-year" summary vector where the golden holds 15 — that a loose assertion can still
pass. It would then be regenerated into a golden and become the frozen reference. That is
precisely the failure the two-commit split exists to prevent, so it is fixed **here**,
where it is provably inert.

⚠ **The first enumeration keyed on the NAME (`year`, `_YEAR`, `year_steps`) and was
incomplete** — it cannot see a day count called anything else. Re-run **by use site**,
name-blind, before trusting any inventory:

```
rg -nE "states?\[[^]]*[*%][^]]*\]" tests/ src/     # arithmetic trajectory subscripts
rg -nE "\b[nikdy]\s*%\s*" tests/ src/              # modulo against a counter
rg -nE "(_DAYS|_days|len\(_?[Ww]eather)" tests/ src/   # every day-ish constant
```

That is what surfaced `test_regression_sealed_station.py:132` and the cross-port fold,
neither of which contains the string `year` as a name.

⚠ **The name-blind greps were still not enough, and the reason is worth keeping.** A
*third* pass — enumerating **callers of the function** rather than occurrences of a name —
found `tests/test_senescence_form.py`, which the first two passes both missed: its constant
is called `year_len`, matching neither `_YEAR` nor `year = len(`, and one of its three uses
**inlines `len(_weather())` directly into the call** with no local to grep for at all. The
rule that actually works: for a unit bug, enumerate **every call site of the function whose
parameter carries the unit** (`rg "year_summaries\("`), then read each argument. A name can
be anything; the callee cannot.

| # | class | sites | fix |
|---|---|---|---|
| A | `year_summaries(states, Y, …)` on a **biosphere** trajectory | `test_decade_stability` ×9, `test_regression_long_horizon` ×6, `test_stem_reserves` ×3, `test_senescence_form` ×3 (found only on the third pass) | `steps_for(Y)` |
| B | `n % Y == 0` in a hand-rolled reset closure | `test_soil_fractionation:529`, `test_stem_reserves:526` | `steps_for(Y)` |
| C | `i % Y == 0` over a trajectory index | `test_consumer:500,534`, `test_compartment_ledger:132` | `steps_for(Y)` |
| D | trajectory slicing `states[y*Y …]` | `test_perennial_chamber` ×5, `test_consumer:442`, `test_soil_layers` ×3, `test_compartment_ledger:213` | `steps_for(Y)` |
| E | conservation ceiling `N · BALANCE_ATOL` | `test_biosphere_stress:130`, `test_decade_stability:96` | `steps_for(Y)·years` — the bound is genuinely per-step, so it *should* grow 4× |
| F | the cross-port fold segmenting a Rust per-step series | `tests/crossport/test_crossport.py:576` | `steps_for(raw["season_days"])` |

**⚠ Sites that look identical and must NOT be converted** — the distinction is real and
worth stating, because converting them would be the same bug pointing the other way:

* **`run_master_day` returns one state per master DAY** (`driver.py`: `states.append`
  is outside the sub-step loop). So the station trajectory is **day-indexed** while the
  biosphere `run_season` trajectory is **step-indexed**. `test_sealed_station_landmine:156`,
  `test_sealed_station_stability:264` and `test_regression_sealed_station:132` index it with
  `season_days` and are **correct as written**. ⚠ This is a **design constraint on ceremony
  item 1**: when `slow_steps_per_day` lands, `states` must stay one entry per master day, or
  those three tests and the sealed-station goldens change *shape*.
* Every `_STEPS = <X>_DAYS * scenario.steps_per_day` in the power / crew / thermal tests —
  that is the **fast** domains' own steps-per-day, which `BIO_DT` does not touch.
* `test_soil_fractionation:1523-1524` (`assert 807 % year_steps == 197`, `502 % … == 197`)
  are **measurement pins, not unit conversions.** Wrapping them would make them false at
  `dt = ¼` for a reason unrelated to the science. Left alone and marked expected-red for
  commit 2, which re-measures them.

**Fixed by hand, file by file — deliberately NOT by a second rewriter.** A day count used
as a step count is *syntactically identical* to a correct step count, so no regex can
discriminate; and the first rewriter's failure mode was to fail open on a name collision
and hide exactly the sites that then went red. A script that gets 28 of 30 right and
silently wrong-converts 2 would cost commit 2 its attributability.

**⚠ `dt = 1` green does not verify this class** — it only proves inertness, which is not
the property in question. The discriminating check is a throwaway run with the constants
flipped locally (see §5 item 0).

**The chosen design:** `_table` indexes **physical time** —
`values[min(int(n * dt), last)]`. At `dt = 1.0` that is `values[min(n, last)]`
character-for-character, and `n * 0.25` is exact in binary so `int()` truncates cleanly.
Then `dt`, steps-per-day and steps-per-year go behind one small helper in the biosphere
domain, and the call sites are routed through it so they read as **physical time** rather
than step counts.

## 2. The two-commit split (the reason for it)

**Commit 1 — the indexing change, at `dt = 1.0`, with EVERY GOLDEN BIT-IDENTICAL.**
`_table` moves to physical-time indexing, the helper lands, the call sites are routed through
it, the perturbation windows are expressed in days. The step itself does **not** move. The
exit criterion is that the full suite passes with **no golden regenerated**.

⚠ **The routing pass went red on 2026-08-14, and the cause was the helper's NAME.** The
helper was first called `steps`, which is the most natural local name in this suite
(`steps = len(weather)`, `def _run(..., steps: int = ...)`). At every such site the local
shadowed the import and the call raised `'int' object is not callable` — **31 errors + 26
failures, all one cause.** Renamed to `steps_for`; the ~30 locals were left alone, because
renaming *them* would be churn that leaves the same trap armed for the next test written.
Two things came out of it worth keeping:

* The collision was **loud here only because those lines run on every pass.** A shadowed
  call on a rarely-taken branch fails just as silently as any other name error waiting for
  its branch. The fix is the un-collidable name, not vigilance — recorded in `step.py`'s
  docstring so it is not "simplified" back.
* The routing script had **failed open on exactly this**: in the four files with a local
  `steps`, it saw the name already bound and skipped adding the import, so those files
  called a helper they never imported. A tool that resolves a name collision by declining
  to import is not being careful, it is hiding the collision — and the four sites it hid
  were the four the suite then failed on.
* Chasing it surfaced a real naming lie: those locals were `len(weather)`, a **day** count
  called `steps`. Renamed to `days`, and a stale `# season length in steps` comment
  corrected to days. That conflation is the whole reason this commit exists.

⚠ **Commit 1 is a FORM change that moves nothing in the manifest** — the `WSFD` shape that
`docs/biosphere-reference.md` calls out explicitly as *the unfreeze nothing catches*. Both
automatic gates are blind to it, so the ceremony on commit 1 is honor-system and is followed
deliberately, not waited for.

**Commit 2 — the step moves.** One edit to the helper's constants, then regenerate. Because
commit 1 proved bit-identical, **every byte of commit 2's golden diff is attributable to the
step**, which is exactly what this project's *"predict the golden diff before regenerating"*
discipline needs.

## 3. ⚠ What is NOT in this ceremony

The direction plan proposed one ceremony carrying the step **plus** the parked leaf mechanism
**plus** a chamber-CO₂ science band. **The leaf merge is deliberately excluded.** The user
authorized a decision about *the step*; and the plan's own condition for merging the leaf
branch is that its evidence base be **re-measured at the shipping step** — every number that
got it accepted (the Greenwood gate clearing by 5.2 %, thickness inside real wheat's range,
`rationed == 0`, the `WSFL` leverage cut) was measured at Euler `dt = 1`, and that measurement
has not been done at `dt = ¼`. Bundling it would also make the golden diff unreadable.

Land the step, regenerate, verify. **Then** put the leaf question with its evidence
re-measured. Same authorization, two attributable regenerations.

The chamber-CO₂ science band is a small addition and may ride along **after** the step lands
and its readings are known — a band written before the regeneration would be a prediction, not
a contract.

## 4. Predicted golden diff — WRITTEN BEFORE REGENERATING

From the sweep (`docs/plans/post-roadmap-step-sweep.md` §1–2), at Euler `dt = ¼`:

| observable | now | predicted | direction |
|---|---|---|---|
| sealed season-low CO₂ | 57.89 ppm | ~75.8 ppm | **up ~18 ppm** |
| perennial season-low CO₂ | 56.03 | ~75.5 | up ~19 |
| consumer season-low CO₂ | 73.29 | ~74.4 | up ~1 |
| water_biting season-low CO₂ | 87.96 | ~95.7 | up ~8 |
| sealed peak leaf C | 0.9215 | ~0.8923 | **down ~3 %** |
| sealed harvest | 0.7189 | ~0.7234 | up ~0.6 % |
| open field peak leaf C | 9.3023 | ~9.4889 | up ~2 % |
| open field harvest | 33.7142 | ~34.1489 | up ~1.3 % |
| unclamped margin (sealed) | 1.3072 | ~5.4574 | **up ~4×** |
| `rationed` | 0 | **0** | unchanged — asserted |
| extinction events | () | **()** | unchanged — asserted |

**Anything outside this table is a finding, not a nuisance.** In particular `rationed` and
`events` are *assertions*, not predictions: if either moves, stop.

⚠ **This table predicts VALUES; the sixth class would move SHAPES.**
`tests/regression/golden/drift_summary.json` holds 15-element arrays for a 15-year horizon
(`perennial.peak_leaf`, `consumer.peak_leaf`, `consumer.consumer_carbon`). An unconverted
`year_summaries` caller at `dt = ¼` returns `(len(states)-1) // days` = **60** entries, each
covering a quarter-year, and `is_period_2` may well still classify the 4×-resampled cycle.
So: **every array length in every regenerated golden stays identical, and `horizon_years`
stays 15.** A length change is not a value moving — it is a missed conversion.

### 4b. The STATION half — written 2026-08-14, before any constant moved

⚠ **The table above is entirely biosphere observables.** Four station goldens move with this
change and had **no written prediction at all**, which would have forfeited exactly the
attributability commit 1 bought. Advisor finding; this subsection is the repair, and it is a
**gate**: nothing is regenerated until it is written.

**(i) The `n` pin — exact, and the sharpest test in this ceremony.** Every state golden pins
the integer step counter. `n` counts *slow steps*, so the flip multiplies it by exactly 4 —
no science, no truncation error, no judgement. **A wrong `n` is a missed conversion, full
stop.** Enumerated from disk (not from the 7-scenario roster — see the "coverage roster ≠
manifest" lesson):

| golden | now | predicted | why |
|---|---|---|---|
| `season_euler_state` | 305 | **1220** | 305 d × 4 |
| `n_limited_state` | 305 | **1220** | 305 d × 4 |
| `water_biting_state` | 305 | **1220** | 305 d × 4 |
| `sealed_chamber_state` | 915 | **3660** | 3 yr |
| `consumer_chamber_state` | 1525 | **6100** | 5 yr |
| `perennial_chamber_state` | 1525 | **6100** | 5 yr |
| `consumer_long_horizon_state` | 4575 | **18300** | 15 yr |
| `perennial_long_horizon_state` | 4575 | **18300** | 15 yr |
| `greenhouse_state` | 7 | **28** | 7 master days × 4 slow steps |
| `harvest_state` | 7 | **28** | 7 master days × 4 |
| `lighting_state` | 7 | **28** | 7 master days × 4 |
| `sealed_station_state` | 1220 | **4880** | 4 yr of master days × 4 |

**(ii) The goldens that must stay BYTE-IDENTICAL.** No biosphere in them, so the step cannot
reach them. If any of these moves, the flip leaked somewhere it had no business being:
`cabin_gas_state`, `crew_state`, `eclss_state`, `power_state`, `power_self_discharge_state`,
`thermal_state`, `station_state`, `water_recovery_state`, `demo_euler_state`,
`demo_rk4_state`, `state_snapshot`, `sealed_energy_drift_summary`.

**(iii) A structural prediction with teeth — the lighting seam's Power half.**
`src/station/lighting.py` states that **Power and the biosphere share no stock**; the lamp is
driven by a constant forcing, not by anything the plant does. So in `lighting_state.json` the
biosphere stocks move and `power.battery`, `boundary.waste_heat` and `boundary.light_used`
must come back **bit-identical**. If a Power stock moves, "coupled only by a forcing schedule"
was wrong, which is a **seam finding**, not a step finding.

**(iv) Values — directional, with the honest label on each.** These are *derived* from the
biosphere anchors above, not separately measured; the derivation is stated so a miss is
diagnosable rather than merely surprising.

| observable | expectation | reasoning |
|---|---|---|
| greenhouse / harvest / lighting biosphere stocks | **small, < ~1 %** | 7-day runs. The compensation-point crossing is a season-scale effect; over 7 days only local Euler truncation error shows. |
| greenhouse cabin gas (shared stocks) | small, same sign as its biosphere leg | in the greenhouse the biosphere's gas pools **are** the cabin air |
| sealed-station biosphere CO₂ | **up**, order the biosphere rows (~+18 ppm scale) | same mechanism, same 4-yr scale as `sealed_chamber` |
| sealed-station peak organic C | **down a few %** | tracks sealed peak leaf C, down ~3 % |
| sealed-station crew / ECLSS / power stocks | small | coupled to the biosphere only through the gas and food seams, and buffered |
| `rationed`, extinction `events` | **0 and `()`** | ⚠ **assertions, not predictions** |

**(iv-b) Two pre-commit checks, both discharged 2026-08-14 (advisor).**

* ⚠ **Item 7's CI discharge reasoned only about the BIOSPHERE goldens.** The four station
  goldens now carry moved transcendental biosphere values too, so the same reasoning has to
  be *checked*, not assumed, for them. It holds: `test_regression_sealed_station.py`,
  `test_regression_greenhouse.py`, `test_regression_lighting.py` and
  `test_regression_harvest.py` all carry `windows_golden_only`. Regenerating on Windows
  will not redden CI on either side.
* **No manifest horizon is a step count.** All seven `docs/biosphere-reference.manifest.json`
  scenario rows record `years` (1/3/5/15), so no horizon field moves with the step — only
  `dt_days` and the per-golden `golden_sha256`s. (`n_limited` and `water_biting` are goldens
  on disk with no manifest row at all — the "coverage roster ≠ manifest" asymmetry, which is
  why §4b(i) is enumerated from disk.)

**(v) The shape assertions, restated for the station.** `states` holds one entry per **master
day**, and `slow_steps_per_day` does not change that (`driver.py` keeps `states.append` in the
master-day loop). So **every station trajectory length is unchanged** and every day-indexed
slice in the station tests stays correct. A station trajectory that got 4× longer means the
`states.append` moved inside the slow loop — a driver bug, not a step effect.

### 4c. ✅ The prediction, scored against the regeneration (2026-08-14)

Written before any constant moved, scored after. **Every structural prediction held; nothing
in (i)–(iii) or (v) needed revision.**

* **(i) `n` — 12 of 12 exact.** 1220 / 1220 / 1220 / 3660 / 6100 / 6100 / 18300 / 18300 for
  the biosphere goldens, 28 / 28 / 28 / 4880 for the station ones. No judgement was involved
  and none was needed: a single wrong `n` would have been a missed conversion, and there was
  none.
* **(ii) The 12 byte-identical goldens — none moved.** `git status` over
  `tests/regression/golden/` lists exactly the 12 expected files and no others. The step did
  not leak into the sibling-domain goldens.
* **(iii) The lighting seam's Power half — bit-identical, as claimed.** Comparing stock-by-stock
  against `HEAD`: every `biosphere.*` stock moved, and `power.battery`,
  `boundary.waste_heat` and `boundary.light_used` are **unchanged to the last bit**. This is
  the sharpest single result in the ceremony — it independently confirms the seam really is
  coupled by a forcing schedule only, which until now was a claim in a docstring that nothing
  tested. (`biosphere.storage_c` is also unchanged: a 7-day seedling has not begun filling
  storage, so it sits at its initial value in both runs.)
* **(v) Shapes — no line added or removed anywhere.** `git diff --stat` reports
  **279 insertions and 279 deletions** across the 12 files: every change is a value replacing
  a value. That is an independent, whole-tree proof that no array changed length, stronger
  than checking the three `drift_summary` arrays by hand. `horizon_years` is still 15, the
  three arrays are still 15 long, and both `is_period_2` flags are still `false` (they were
  `false` before — the 4×-resampling worry in §4 never arose because the segmentation was
  converted).

**The VALUE rows, measured independently of the suite** (`temp/step-unfreeze/probe_co2.py`,
so the two assertion rows are read on their own rather than buried among suite failures):

| row | predicted | measured | verdict |
|---|---|---|---|
| **sealed season-low CO₂** | ~75.8 ppm | **76.82 ppm** | ✅ **clears 61.07 — the point of the ceremony** |
| sealed peak leaf C | ~0.8923 | **0.892261** | ✅ on the nose |
| open field peak leaf C | ~9.4889 | **9.488895** | ✅ exact to 6 s.f. |
| perennial decade CO₂ trough | ~75.5 | **75.48** | ✅ (probe, §5 item 0) |
| `rationed` | **0** — assertion | **0** | ✅ |
| extinction `events` | **()** — assertion | **()** | ✅ |

⚠ **One discrepancy, recorded rather than smoothed.** The sealed season-low came in at
**76.82 ppm**, about 1 ppm *above* §4's predicted 75.8 — and above the **76.29** that
`step.py`'s docstring quotes as the *RK4 limit* the refinement sequence converges toward. A
Euler run at a finite step should sit below that limit, not above it, so one of the two
numbers is measured on a different subject. The likely explanation is that the sweep's figure
predates several mechanisms that have since landed (stem reserves, soil layers, root
coupling), so its converged limit is a limit for a tree that no longer exists — but that is an
*explanation*, not a measurement, and it is not being treated as one.

#### ⚠⚠ RESOLVED 2026-08-14 — and the explanation above was WRONG

Measured, not argued (`temp/step-unfreeze-repin/probe_sequence.py`, both configurations, one
tree, nothing touched): **the tree never changed.** The sweep's table reproduces **cell for
cell** — sealed `57.8925 / 75.0588 / 75.8185 / 76.0339`, perennial
`56.0299 / 74.9148 / 75.4757 / 75.6516` — on today's code.

The real cause is the **run**, not the tree. `season.run_perennial` applies `annual_reset`
**unconditionally**; it never asks whether the scenario is perennial (`season.py:609-612`).
`sweep_biosphere.py` drove every scenario through it, while the sealed chamber's golden
(`test_regression_sealed_season.py:60`) uses plain `run_season`. So the sweep's sealed rows
are a **re-sown** sealed chamber — a run no golden performs — and 76.82 is the **no-re-sow**
reading. Separated, each sequence converges monotonically from below to its own RK4 limit
(~76.29 with re-sow, ~77.11 without): the paradox was the comparison.

⚠ **The consequence is bigger than the docstring.** In its own configuration the sealed
chamber reads **75.75 ppm at `dt = 1` — it never crossed 61.07 at all.** The chambers that did
cross are **perennial (56.03)** and consumer, both of which genuinely re-sow. So this
ceremony's headline named the wrong scenario, and the pair "57.9 → 76.82" compares two
different kinds of run; the honest pairs are **56.03 → 75.48** (perennial) and
**75.75 → 76.82** (sealed). **The step move and its authorisation stand** — the perennial
crossing is real and is the thing the ceremony fixed.

⚠ **Three lessons, and the first one is the cheap one that was skipped.** (1) The wrong
explanation was *already refuted by evidence in hand*: sealed peak leaf agreed to 6 s.f.
between the sweep and this ceremony, which a changed tree cannot do. A first explanation that
fits is not a measurement. (2) A helper named for one scenario class that silently applies to
all of them will be called on all of them; `run_perennial` should arguably assert its
scenario. (3) The error propagated into the *recommended fix* — the successor `science_band`
was proposed as *"the sealed chamber's season-low stays above the compensation point"*, aimed
at the one scenario that never had the problem, where it would have passed immediately.
**A guard inherits the locus of the diagnosis that motivated it.**

⚠ **Not checked:** the `57.9 ppm` figure predates the sweep (it headlines
`log/co2-enrichment-margin.md` and the CO₂-controller work), so whether those measured through
the same unconditional re-sow is **unknown** and is named as an open question, not asserted.

### 4d. What the PORT caught that the reference did not (2026-08-14)

The purity rule says the port has no reference authority — a Rust run that surfaces a Python
bug is an unfreeze-discipline **finding**, never a native-side fix. It happened twice here,
and both were fixed in Python first and mirrored.

**(1) A sixth-class site the commit-1 sweep MISSED.** `tests/test_station_perturbations.py`
had `_START, _END = 2, 7  # window (master days)`, and the perturbation forcings key on `n`.
At `dt = ¼` that reads as **days 0.5–1.75 instead of days 2–7** — a 4× shorter window in a
different part of the run. Commit 1's three-pass sweep covered the biosphere's
`test_perturbations.py` and **not this station sibling**; the caller-of-`year_summaries`
method that caught the last biosphere site does not generalise to a *different* callee. The
Rust port's `o2_leak_is_absorbed_by_makeup_effort` went red on it. ⚠ **The lesson is about
the sweep, not the site: "enumerate by callee" has to be repeated for EVERY callee that
carries a unit** — `year_summaries`, `with_station_leak`, `with_crew_load_spike`,
`with_lighting_failure`, `run_perennial`'s reset period — not just the one that produced the
lesson. Both ports' `with_station_leak` docstrings also asserted *"`n` is the day count, so
the window activates on whole master days"*; corrected.

**(2) A test whose tolerance was arbitrary, kept by measuring instead of loosening.**
`o2_leak_is_absorbed_by_makeup_effort` claims the two pools *"fail differently"*: a CARBON
leak changes the biology, an O₂ leak is absorbed by the demand-controlled makeup and leaves
the plant alone. "Leaves the plant alone" was pinned as `rel_tol = 1e-6`, and after the
window fix it still failed — at `1.55e-5`, in **both ports identically**, which is itself the
evidence that this is the reference's behaviour and not a port defect.

It is a real and explainable effect: per master day the slow domain now takes four steps
before any fast makeup runs, so the plant sees the intra-day O₂ drawdown at four
progressively lower levels instead of one, and `f_O2` self-limitation responds slightly
differently. Magnitude: **0.0015 %**.

⚠ **Loosening `1e-6` to `1e-4` would have been weakening a test to make it pass**, which this
project forbids. Measured the contrast instead: the CARBON leak moves biomass **16.6 %**, the
O₂ leak **0.0015 %** — a factor of **10715**. The assertion is now that ratio, in both ports.
That is **strictly stronger** than what it replaced: the old absolute form also passed if the
carbon leak did nothing at all, and the new one does not. *An absolute tolerance standing in
for a contrast is a pin waiting to break on the first scale change; write the contrast.*

**(3) Six `n`-is-the-day-count assumptions in the consumer layers**, all in the Godot bridge,
session, palette and save/load parity tests (`session.n() == master_days`). Fixed by
converting, and the save/load helper now takes an explicit `steps_per_unit` because
`step_n` counts **master days** while `n` counts **slow steps**. Also `FastForwardTo(n)`
targets a step count while `step_n` takes master days — the worker-parity test passed the
same number to both. ⚠ **The public `fast_forward_to(n)` Godot API is documented in steps and
is unchanged; a front-end that means "day 5" must convert.** Flagged, not redesigned: the API
semantics are a Phase-8 consumer question, not part of this ceremony.

**(4) One clippy lint, allowed with a reason.** `slow_steps_per_day` pushed `session::Mode`
from 376 to 384 bytes, crossing `large_enum_variant`. A process holds exactly one live
session, so the waste is ~200 bytes; the suggested fix boxes a resolver on the
parity-critical stepping path. Allowed with the rationale written at the site.

## 5. The rest of the ceremony, in order

0. ✅ **The discriminating check — RUN 2026-08-14, PASSED.** `BIO_DT`/`STEPS_PER_DAY` were
   flipped locally and **uncommitted**, the affected files run, and the failures read for
   *kind* rather than count. Then the constants were reverted and commit 1 landed clean.
   This is the check the whole sixth-class fix rests on: commit 1's "provably inert"
   claim is true and *uninformative*, because inert at the old step is not the property
   the sixth class needs — *correct in the new unit* is, and no `dt = 1` run can show it.

   **What was measured at `dt = ¼`:**
   * A 3-year perennial run gives **3661 states** (= 3·305·4 + 1) and `year_summaries`
     returns **3 entries, not 12** — the segmentation still means a year. `rationed = 0`,
     `events = ()`.
   * `test_perennial_chamber.py` passes **in full**, including the re-sow timing pins
     (`dvs(states[y·YEAR]) ≈ 2.0` pre-reset, `< 0.1` one step after) — the annual reset
     fires **once** per year at the right step, which is the failure this class existed
     to prevent.
   * Across the affected files the **only** failures were the two expected kinds:
     the deferred **fifth class** (`test_compartment_ledger` /`test_consumer`
     `..._ledger_balances_every_step` — a leg reconstructed at `1.0` against a `0.25`
     engine step, residual 5.4e-3 vs a 1e-12 tolerance, failing loudly and naming
     itself), and **value pins moving in the predicted direction**.
   * ⚠ **A §4 prediction was hit on the nose**: the perennial decade CO₂ trough pin moved
     `0.0559766 → 0.0754757`, i.e. 55.98 → 75.48 ppm against a predicted ~75.5.
   * **Zero shape failures and zero unit failures.** No summaries vector was 4× long; no
     reset fired quarterly.

   ⚠ **A commit-2 finding, recorded now so it is not mistaken for a regression later:**
   `test_decade_stability.py::test_the_co2_floor_fires_on_the_buffer_not_on_the_carbon_supply`
   fails at `¼`. That is the *point* of the unfreeze — the CO₂ floor stops being reached —
   so it is a science finding to be argued past in writing during commit 2, not a pin to
   re-tune. Two sibling decade pins move with it.

1. ⚠ **`run_master_day`** must take `STEPS_PER_DAY` slow substeps per master day
   (`src/station/driver.py`) — engine code, and the cost Step 0 found that the direction
   plan never priced. **Confirmed 2026-08-14: this IS a station-contract unfreeze too.**
   `run_master_day` is not itself named in `docs/station-reference.manifest.json`, but the
   manifest's `numerics_note` reads *"Sealed reference: biosphere-slow **dt=1 day** +
   everything-fast dt=60 s"* — the step is written into the station contract in prose, so
   it moves with this change.
   ⚠ **The commit-1 indexing change makes this driver edit much smaller than feared.** The
   driver's stated reason for one slow step per day was *"so `n` stays the day count the
   day-indexed weather resolver reads"* (`driver.py:44-46`, and again at 109-112). That is
   now obsolete: the resolver indexes `int(n · dt)`, so `n` may be four times the day count
   and still read the right weather row. The `fast_dt · steps_per_day == 86400` guard is
   about the **fast** domain and is untouched. Add a `slow_steps_per_day` keyword defaulting
   to `1` so existing callers stay byte-identical, and update the two stale comments.
2. **Regenerate the affected goldens**, each through its own explicit `__main__` action, and
   review the byte diff against §4.
3. **Regenerate both manifests** and review the diff — the git-visible record of what moved.
   ✅ **Good news, found 2026-08-14: the step change is NOT honor-system.**
   `docs/biosphere-reference.manifest.json` records `dt_days: 1.0`, and
   `tests/test_freeze_manifest.py:444` asserts it **against a hard-coded literal**
   (`assert manifest["dt_days"] == 1.0`). So the step move fails loudly and the contract is
   updated deliberately — unlike commit 1, which nothing catches. Keep that assertion a
   **literal**; rewriting it to compare against `BIO_DT` would make the contract auto-follow
   the code, which is the opposite of a freeze.

   ⚠ **But the STATION half of the same statement is honor-system — checked 2026-08-14.**
   The two contracts are asymmetric and it would be easy to assume the biosphere's loud gate
   covers both. It does not. `docs/station-reference.manifest.json` records the step **only**
   in `numerics_note` prose (*"biosphere-slow dt=1 day"*), and that string is a hard-coded
   literal inside the **generator** (`tests/test_station_freeze_manifest.py:286`), so the
   generated manifest and the on-disk manifest agree with each other whatever `bio_dt` says.
   Flipping `bio_dt` reddens **nothing** there. The note reads *"dt per scenario (enforced by
   goldens, no importable constant)"* — true, but goldens enforce the *values*, and the prose
   is what a reader takes the contract to say. This is the "freeze's prose half is ungated"
   lesson again: **edit that literal deliberately, by hand, as part of the ceremony.**
4. **Report the science gates** — every `science_bands` and `liveness_floors` reading. ⚠ A
   band failure is a **blocking finding to be argued past in writing**, never a number to
   re-tune; and a band *passing* is not an endorsement (`open_season` sits 3.8 % above the LAI
   lower bound — a tight margin, not comfort).
5. **Cross-port tiers** — 7 biosphere + 4 station goldens move, so `tests/crossport/tiers.json`
   bands need **re-measuring**, not just re-running.
6. **The Rust mirror** — the port has no reference authority; it mirrors the rule. The
   `_table` analogue is `rust/crates/domains/src/biosphere/system.rs::table_schedule`
   (mirrored in `6861e2b`, bit-identical at `dt = 1`); the step constant needs the same
   treatment when it moves. ⚠ **Item 6 named only the biosphere step constant and that was
   short** (advisor): `rust/crates/station/src/scenario.rs` carried four independent
   `bio_dt: 1.0` literals, the Rust `driver`/`session` needed the same `slow_steps_per_day`
   split as Python (`session` shares `advance_one_master_day` with the runner by the Phase-8
   parity discipline, so it moves with it), and `sealed_reset_hook`'s period needed the same
   days→steps conversion. All bound to `domains::biosphere::{BIO_DT, STEPS_PER_DAY}` rather
   than re-declared.

   ⚠ **A conflation this ceremony ACCIDENTALLY CREATED, caught by advisor and fixed the same
   day.** The Rust side already had a `steps_for`, and it took **years** where Python's takes
   **days**. Both were arithmetically right, and mirroring the step into the Rust one left the
   *same name meaning two different units across the two ports* — the exact defect this whole
   ceremony exists to remove, re-introduced by the fix for it. Renamed to `steps_for_years`,
   with a real `steps_for(days)` added alongside matching Python's unit exactly. The lesson is
   narrow and worth keeping: **a cross-port pair must agree on the unit of a shared name, and
   a rename in one port is not a mirror unless the unit mirrors too.**
7. ~~⚠ **The CI hazard**~~ ✅ **DISCHARGED 2026-08-14 — no new work.** `tests/golden_platform.py`
   already gates every transcendental golden behind `windows_golden_only`
   (`skipif sys.platform != "win32"`), precisely because hex-float goldens are byte-exact
   only on their generation platform. All the biosphere goldens are transcendental and
   therefore already gated, so regenerating on Windows will not redden CI. The genuinely
   cross-platform gate is the cross-port tolerance band — which item 5 re-measures anyway.
8. **Gates**: full suite including `-m slow`, `ruff`, `pyright`, `cargo test`, `cargo clippy`.
   ⚠ `¼` is **4× raw simulation work**, so the full-suite upper bound is ~25 min, not the
   ~12 min quoted for `½`. Use targeted regeneration during the work and one full run at the
   end.

## 6. Invariants that hold throughout

* `git diff src/simcore/` **stays empty, unconditionally** — there is no unfreeze path that
  edits the core. The step is a domain-side instantiation choice.
* The arbitration backstop stays Euler-only; `rationed == 0` stays asserted on golden runs.
* No test is weakened or deleted to make this pass. A test that goes red is a finding.
* Conventional Commits naming the unfreeze; commit and push to `main`.
