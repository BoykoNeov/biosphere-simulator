## **The chamber-scale diagnosis** (the upstream blocker with THREE witnesses)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED 2026-08-09, read-only — and the answer is that the CHAMBER is not the thing to
fix.** `docs/plans/post-roadmap-chamber-scale.md`; probes `M:/temp/chamber/`; 11 pins in
`tests/test_chamber_scale.py`. Scope (A) finding 11, canopy-regulator finding 4 and (C)
finding 8 all bottom out on one number, so it was measured instead of inferred. **THE
HEADLINE — the obvious fix is REFUTED, not merely unattractive, and by an engineering
reference rather than by taste.** [BVAD] Table 4-88 (p. 170, Drysdale 1999b) gives a flight
biomass-production chamber at **1.03 m³/m² of growing area** (shoot zone 0.67); ours is
`1000 mol / 1 m²` = **24.06 m³/m² at 20 °C**, i.e. **~23× MORE generous than the flight
design already**. Holding one field crop's standing carbon (56.03 mol C/m²) in that air at
its 357 ppm needs **157× the air = 3,665× BVAD**; at the *top* of [BVAD]'s cited plant
optimum (p. 130, "roughly 0.10 to 0.20 kPa (Wheeler et al. 1993)" = 987–1974 ppm) it still
needs **28× = 663× BVAD**. ⇒ **at any defensible spec the atmosphere is a buffer of HOURS**,
and the co-adaptation trap never has to be argued — resizing to rescue a refused science
*would* be backfitting, but it is independently **impossible**. The one legitimate
atmospheric knob is **2.8–5.5×**, cited, and not taken. **THE CENSUS** (each scenario driven
the way its own golden drives it): inventory **3.517 / 3.517 / 3.884 mol C** = **42.2 g
C/m²**, **85 % of it the seeded litter pile and 10 % the air**; at peak the plant holds
**55.8 / 63.9 / 67.0 %** of every carbon atom in the system. In cited units of demand
([BVAD] Table 4-91 wheat, CO₂ uptake 77.00 g/m²·d = **1.7496 mol C/m²·d**): the whole
inventory is **2.01 days** of one m² of wheat, the atmosphere alone **4.9 hours**, and —
against Table 3-31's 24.654 mol C/CM-d already first-hand in `docs/bvad-reference.md` —
**3.4 hours of ONE crewmember's exhalation**. That chamber is asked to run **3, 5 and 15
years closed**. ⚠ **THE ROSTER WAS CHECKED AGAINST THE MANIFEST, NOT AGAINST THE TABLE'S OWN
LENGTH — and the first draft was the THIRD instance of that shape** (advisor catch; cf.
(B)-finding 4's five rows vs seven frozen scenarios, (A)-finding 9's list checked against
its own length): three sealed rows, no `water_biting`, no 15-yr row under a sentence naming
15 years. **Measured, and three rows cover all four sealed chambers at every frozen horizon
BY CONSTRUCTION**: the long-horizon goldens **reuse the same scenario objects** (differing
only in `years`) and the inventory is a **t=0 property**, so it is **bit-identical** at 5
and 15 yr with the peak fraction unmoved (63.9 / 67.0 %, `rationed == 0` at both) — the
horizon lengthens the run, not the jar; and `water_biting` is `sealed=True,
litter_carbon0=3.0` with **every gas default**, i.e. the perennial chamber with
`soil_water0` 1000→50, so its inventory is perennial's **bit-exactly**
(`0x1.c22d0e5604189p+1`), plant peaking at **64.9 %**. Not a fourth jar — the same jar under
water stress. The same crop with an unbounded source reaches **1495 g DM/m², LAI 5.19** vs
the chamber's **52–70 g DM/m², LAI 0.51–0.63** — **~24× in mass, ~9× in leaf area**
(different quantities, not one ratio). ⚠ **A RETRIEVAL HAZARD MATERIALIZED AND THE VISUAL
CHANNEL CAUGHT IT — round 6's rotated-table finding taking a SECOND instance**: `pdftotext
-layout` scrambles Table 4-91's columns and files **Rice's** row (30.23/39.0/42) under the
name **Wheat** (true row 50.00/150.0/42/56.00/**77.00**/11.79). Both BVAD numbers were read
off 170 dpi page renders — including Table 4-88, whose extraction happened to be *correct*,
because by then extraction was proven untrustworthy for this document. **THE MECHANISM —
`stock = flux / k`, an IDENTITY, and with ONE pool you get one of them.** [RothC] §1.5
first-hand (`sources/RothC_guide_WIN.pdf`, the decomposer calibration's own source): DPM
**10.0**/yr, RPM **0.3**, BIO **0.66**, HUM **0.02**, IOM **inert**. Our
`decomposition_rate` 0.011/day = **4.015/yr** is a *decomposable-plant-material* rate; a
real soil keeps most of its carbon **13×–201× slower, plus a pool that never decomposes at
all**. [RothC]'s Hoosfield example: **33.8 t C/ha = 281.4 mol C/m²** at equilibrium on a
**1.70 t C/ha/yr = 14.15 mol C/m²·yr** input; our litter stock **3.0 mol C/m² = 0.36 t
C/ha** is **94× short** while our litter *flux* `k·C` = **12.04 mol C/m²·yr** is the **same
order**. ⚠ **The flux agreement is n=1 and is deliberately NOT written as a law** (advisor's
blocking catch): one soil, one crop, 1852 arable — and `litter_carbon0 = 3.0` was sized by
probe to make **O₂ depletion dramatic**, `decomposition_rate` recalibrated separately for
closure, so two free numbers landing 0.85× on one reference is **one point**, and calling it
"our flux is right" would be this project's own meta-finding again. **Only the ORDERING is
claimed and only the ordering is pinned** (same order of magnitude, never the ratio); the
94× stands on its own arithmetic. **This also explains a prior measurement whose recorded
cause was the SYMPTOM**: the decomposer calibration's finding 4 (litter ×5 rations
`perennial`, ×10/×20 explode) was attributed to "the chamber's O₂ headroom, **not the litter
size**" — the cause is that **at fixed `k`, stock and flux are the SAME KNOB** (Hoosfield's
stock at our `k` would return **~80× real soil respiration**), so there is no litter size
that raises the inventory without raising the draw. ⚠ **Sharper than "they missed it": that
paragraph writes the identity `flux = k·C_litter` down TWO SENTENCES before concluding
against it** — its opening steady-state argument ("annual CO₂ return is k-independent; k
only sets the standing pool size") is *correct* for a pool fed by a fixed input, but a
**seeded** pile is an initial condition draining at `k·C`, so the governing identity was the
one already in hand. **Annotated at its own site** (`post-roadmap-decomposer-calibration.md`
finding 4, original kept — the (C)-diagnosis precedent; correcting it only in the new doc is
round 4's error). **THE ANSWER TO "WHAT IS THE SEALED CHAMBER A MODEL OF?"** — it is **not a
BLSS analogue and cannot be made into one by sizing**: a real one balances **~14 m² of wheat
per crewmember** ([BVAD] 24.654 ÷ 1.7496); ours has **1 m² and no crew at all**. (No "at our
field rate" area figure is quoted — the only annual number available is peak **standing**
biomass, not annual **net** fixation; wrong-shaped denominator, advisor catch.) What it
legitimately **is**: a closed plant+soil **test rig**, and it does that honestly. ⇒ **THE
DEFECT IS NOT THE CHAMBER'S SIZE — it is that the frozen contract uses this rig's closure
gate (`rationed == 0`) as the ACCEPTANCE TEST for FIELD-scale plant science.** All three
witnesses are that one collision: a mechanism sized for a 56 mol C/m² crop, judged by
whether a 3.5 mol C jar survives it — while `open_season`, the only frozen scenario at field
scale and the one that grew *better* under two of the three refused changes ((C) full +47.8
%, stem-only +7.96 %; the regulator alone is inert there), carries **no CARBON-SCARCITY gate
at all** — its CO₂ source is an unclamped boundary stock, so a carbon rationing assertion is
unfalsifiable there. **THE SEAM, WITH A MEASURED OBSTACLE — NOT a recommendation** (the
(B)-diagnosis idiom, and the canopy-regulator row is the precedent for what a
zero-frozen-benefit build costs): **soil carbon pool fractionation** (any subset with ≥1
slow pool) is the shape that decouples stock from flux, its science is on the shelf and
first-hand, and it is what would make the **re-labelling this project twice refused** (the
DPM/RPM "labile fraction" re-read; `microbial_n`'s homeostatic C:N) unnecessary rather than
tempting. Priced: new stocks+flows ⇒ `flow_set` + `param_files`, every carbon golden, the
station manifest, `biosphere_params.txt`, the Rust mirror, crossport — and it *enables* a
bigger inventory rather than delivering one. Whoever takes it **names the invariant first**
(increment 1's consumer-chamber 2× was legitimate **because** it held Ci₀ = 250 and x_O₂ =
0.21 invariant — sized on an independent invariant, not on the goldens). Also standing: the
crew-coupled route already exists as `GREENHOUSE_BIO_SCENARIO` (9500 mol air at ~400 ppm, a
**4000 mol C** food store — `crew.food_store` is a CARBON pool, checked not assumed — a crew
respiring into it ⇒ **~1137×, three orders more carbon**), but it runs 7 days with a
seedling and is station-side and non-frozen. Both `open_season` W denominators reconciled to
the digit and pinned **together** (**14.954** t/ha incl. fibrous roots / **12.633** excl. —
the figure `CLAUDE.md` quotes), since conflating quantities that differ only by denominator
has bitten this repo twice. No value/golden/param/manifest moved; `git diff src/` empty;
nothing unfrozen.
