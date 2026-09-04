# The temperature form of the FvCB kinetics — the science switch's first scientific pair

**Written 2026-09-04, before any code.** The item is §2.3.1 of the September direction plan,
ranked there as *"the highest-leverage item in this list, and the least predictable"* and
carrying its own instruction: **predict the crossing before building, not after.** This doc
is that prediction, written and committed against the frozen baselines below, so the record
can be read against it rather than around it.

**Standing:** lab-only. No frozen param file changes, no golden moves, no unfreeze. The form
is selected on the params object the one funnel already threads, and the frozen selection is
the existing code path verbatim.

---

## 1. What the tree does now, and what the alternative is

The tree carries the twelve `photosynthesis.yaml` constants **at 25 °C** and multiplies the
**whole** assimilation rate — Rubisco-limited and light-limited alike — by one
piecewise-linear cardinal-temperature factor `f_temp(T)`, the WOFOST AMAX/TMPFTB idiom
(source `[B]`). That is the tree's entire temperature treatment of photosynthesis.

The alternative is on the shelf and always has been: **Teh (2006), ch. 6**, already in
`sources/` and already cited in `science_gates.rs` as `TEH_SPECIFICITY_FACTOR`. Teh gives
each kinetic constant its own **Q10** response (Table 6.2, book p. 130, eq. 6.28–6.29
`ξ(T) = ξ₂₅·Q10^((T−25)/10)`) and gives the light branch its own response through a
temperature-dependent quantum efficiency (eq. 6.23).

| symbol | Teh's 25 °C value | unit | Q10 |
|---|---|---|---|
| Kc | 300 | µmol/mol | 2.1 |
| Ko | 300 000 | µmol/mol | 1.2 |
| τ (CO₂/O₂ specificity) | 2600 | µmol/µmol | 0.57 |
| Vcmax | 200 | µmol m⁻² s⁻¹ | 2.4 |

`e_m = 0.081 − 0.000053·T_f − 0.000019·T_f²` (eq. 6.23, after Ehleringer & Björkman 1977).
Γ* is not tabulated: it comes from `Γ* = O/(2τ)` (eq. 6.19), so **Γ*'s Q10 is `1/0.57 =
1.754` — a derivation from Teh's own two equations, not a retrieved number**, and it is
labelled as one everywhere it appears.

⚠ **Both the table and eq. 6.23 were read off rendered page IMAGES, not off the PDF text
layer.** The text layer is mangled on those pages: it gives Ko's unit as `mol mol⁻¹` (the
page says µmol mol⁻¹) and eq. 6.23's coefficients as `0.0000537 / 0.0000197` (the page says
`0.000053 / 0.000019`). The FvCB provenance item was burned binding constants without the
page in hand; this one had the page.

---

## 2. The form built, stated exactly

`PhotosynthesisParams` gains one non-YAML field naming **which temperature form its constants
are read under**. Two values:

* **`Cardinal`** — the frozen tree, the existing body verbatim. Bit-identical goldens by
  construction, which is the only reason this is not an unfreeze.
* **`Q10Teh`** — Teh's Q10s applied to **our** 25 °C anchors, Teh's `e_m` shape applied to
  our `quantum_yield` as a **ratio** (`α·e_m(T)/e_m(25)`, because Teh's 0.06 mol CO₂/mol
  photon and our 0.3 mol e⁻/mol photon are different bases — the shape transfers, the value
  does not), and **no** whole-rate multiplier.

**Why the anchors stay ours.** Changing the form and the values in one column measures
neither. This is the standard A/B discipline, but here it collides with a warning the tree
already carries, and the collision is recorded rather than resolved quietly:

> `science_gates.rs`, `the_shipped_floor_is_the_conservative_one_against_the_cited_route`:
> *"Teh's companion constants disagree with ours, so the two are different parameterizations
> and mixing them would be the co-adaptation this project refuses."*

That warning is about swapping a **value** between parameterizations. Taking a **response
shape** from one and leaving every 25 °C value alone is a different act — but it is still a
mixing, so §3 prices it: the prediction table carries a `teh_pure` column that runs Teh's own
25 °C values *and* his Q10s, and the gap between it and the hybrid is the size of the
objection.

**Why the params object and not a flow replacement.** Three flows hold a `CarbonContext` and
each calls `budget()` — `Allocation` (`flows.rs:181`), `GrowthRespiration` (`:247`),
`MaintenanceRespiration` (`:280`). Replacing one gives a step where growth respiration is
computed off the frozen assimilation and allocation off the new one — internally inconsistent
and perfectly plausible-looking in a report. Replacing all three means reconstructing their
contexts, i.e. a **second assembly body**, which is the exact defect `lab/mechanism.rs`'s
header says that module exists to prevent. Riding the form on `BiosphereParams` uses the one
funnel `tests/param_funnel.rs` already gates, and all three flows follow with no composition.

Checked before writing: neither `tests/manifest_writer.rs` nor `science_gates.rs` enumerates
`PhotosynthesisParams`'s fields, so a code-only field moves no manifest.

---

## 3. The prediction — leaf-level, decomposed, written before the season ran

`predict.py` (kept at `M:\claud_projects\temp\fvcb-temp\predict.py`) evaluates the leaf
rate laws at the **real** `open_season` forcing — Ci = 250, the sinusoidal within-day PAR at
`dt = ¼`, the committed 305-day temperature series — at fixed LAI, with no season run.

Season: 1220 steps, temperature min −1.80, 10th pct 4.59, **mean 10.67**, 90th pct 17.58,
max 22.20 °C. **The whole season is below 25 °C**, so every Q10 factor is a reduction and
every cardinal factor is < 1. PAR in lit steps: median 168, 90th pct 981 µmol m⁻² s⁻¹.

**Which branch binds is the whole story, and it was not what was assumed.** In the frozen
tree the light-limited branch binds **99 % of lit steps** (10 of 876 are Rubisco-bound at
LAI 3). So a change to the Rubisco constants is nearly invisible and a change to the light
branch is nearly everything — the reverse of how §2.3.1 and §2.1 item 4 were written, both
of which reasoned from a Vcmax ladder.

Season-integrated canopy gross rate at fixed LAI, as a ratio to frozen:

| LAI | delete `f_temp` only | Teh Q10s under `f_temp` | **the built form** | form, Jmax×`f_temp` | Teh whole |
|---|---|---|---|---|---|
| 1.0 | 1.249 | 1.084 | **1.436** | 1.391 | 1.740 |
| 3.0 | 1.243 | 1.150 | **1.561** | 1.511 | 1.746 |
| 6.0 | 1.241 | 1.163 | **1.580** | 1.536 | 1.748 |

Read at LAI 3, in log terms: **deleting the whole-rate multiplier is 49 % of the effect,
Teh's kinetics 31 %, Teh's quantum-yield shape 20 %.** So the deletion is the largest single
term but **under half** — the form is not merely the multiplier's removal wearing a citation.

Three consequences fall out of the same table:

1. **The flat-Jmax gap is small, and measured rather than caveated.** Giving Jmax the
   cardinal multiplier moves the answer 1.561 → 1.511, about **3 %**. The branch is
   quantum-yield-limited (absorbed PAR ~40 µmol at mid-depth), so Jmax is nearly inert and
   its missing temperature response cannot be the story.
2. **The mixing is worth about 12 %.** Teh's own values give 1.746 against the hybrid's
   1.561, and the difference is almost entirely **Vcmax**: Teh's 200 against our 100. Under
   the hybrid, Vcmax becomes binding on **9.1 %** of lit steps where the frozen tree has it
   binding on 1.1 % — so **the form promotes our provisional, uncited `vcmax = 100` into a
   load-bearing position.** That is a finding about the form, and it is the strongest
   argument for the Bernacchi/Wullschleger retrievals still owed in §2.2.
3. **A live Γ* dissolves the CO₂ floor the five margin gates are read against.** The floor is
   `Γ*/ci_ratio`, frozen at **61.071 ppm**. Under the form it sweeps the season:

   | T (°C) | −1.80 | 4.59 | 10.67 | 17.58 | 22.20 |
   |---|---|---|---|---|---|
   | floor (ppm) | 13.54 | 19.39 | 27.29 | 40.24 | **52.18** |

   It is **below 61.07 everywhere**, so the recorded bound stops being a bound and becomes,
   at best, the season's worst case. This is exactly the hazard `log/o2-coupling-measured.md`
   recorded for a live-O₂ Γ*, arriving by a different door: *a pointwise floor is a different
   assertion, not a re-tuned one.*

### 3.1 The gate predictions — written down, numbers and all

Frozen baselines, measured 2026-09-04 through the lab report:

| quantity | frozen | bound as recorded |
|---|---|---|
| `open_season` peak LAI | 6.022837 | `5.0 < peak < 8.0`; and `peak < 6.0` OR mutual shading modelled |
| `open_season` peak W excl. fibrous roots (t/ha) | 13.379084 | `< 14.4248` |
| `sealed_chamber` season-low CO₂ (ppm) | 71.435803 | `> 61.07` |
| `perennial_chamber` season-low CO₂ (ppm) | 70.252606 | `> 61.07` |
| `consumer_chamber` season-low CO₂ (ppm) | 73.338613 | `> 61.07` |
| `perennial_long_horizon` converged peak-leaf (mol C) | 0.578137 | `> 0.55` |
| `perennial_long_horizon` season-low CO₂ (ppm) | 70.252606 | `> 61.07` |
| `consumer_long_horizon` season-low CO₂ (ppm) | 73.338613 | `> 61.07` |

**Predictions.** Committed before the run; the point estimate is what the record is scored
against, the range is the honest uncertainty.

1. **peak LAI: ~8.0, range 7.3–8.8 → the `< 8.0` ceiling is AT OR OVER, call it RED.**
   +56 % carbon at fixed LAI, damped two ways: the canopy saturates with LAI (the frozen
   integral only gains 19 % from LAI 4 to 6), and the 5 %/day mutual-shading loss above
   LAI 6 gets substantially more to bite on than the sliver it currently sees at 6.0228.
   ⚠ **A mechanism that has barely ever fired becomes load-bearing** — that alone is worth
   the run.
2. **peak W: ~17.4, range 15.5–19.0 → the 14.4248 cap is BREACHED, confidently.** It needs
   only +7.8 % and the supply moves +56 %. This is the prediction I am most sure of.
3. **perennial converged peak-leaf: ~0.70, range 0.62–0.80 → floor gains clearance, GREEN.**
   More carbon raises a fixed point that is 5.1 % above its floor today.
4. **The five CO₂ minima: FALL, point estimate ~62 ppm for `sealed_chamber`, range 50–68 →
   at least one crosses the recorded 61.07 floor.** A bigger canopy draws the chamber down
   harder. ⚠ **And if one does, the red is uninterpretable as science**, because the floor it
   crossed is the *constant* one and the form's own floor at that moment is 20–50 ppm. The
   correct outcome there is *"the gate must be re-posed"*, not *"the form fails"*.
5. **Cross-cutting: the form is NOT inert anywhere.** §2.3.1 predicted the chambers might be,
   on the grounds they sit near 20 °C. They do not — every scenario reads the same weather
   series, so the chambers see the same 10.7 °C mean as the field. That premise in the
   direction plan is wrong and this doc supersedes it.

**What would falsify the whole approach**: if the built form's `open_season` peak LAI comes
back within 1 % of 6.0228, the params object is not reaching the rate law and the run is a
mis-target, not a finding — the failure `tests/param_funnel.rs` exists to make impossible.

---

## 4. What this item does NOT do

* It takes no decision and endorses no form. It regenerates evidence, the standing every
  `lab` item has had since the value switch.
* It moves no frozen value, no param file, no golden, and no manifest.
* It does **not** discharge the Bernacchi page check (§2.2). If anything it raises that
  retrieval's value again: the form makes Vcmax matter nine times more often than it did.
* It does not re-pose the CO₂ floor gate. Prediction 4 says that question is coming; posing
  a pointwise floor is its own item and its own decision.
