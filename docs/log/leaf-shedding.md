## **Tissue shedding that reads no temperature** (opened from Step 3c slice 3b — the cold seedling loses half its carbon to fixed daily shedding rates the books on the shelf do not support)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-06** as a design note only, on the user's choice after Step 3c slice 3b: *"Keep
100, study shedding"*. No code. Note, sources and the measurement plan:
`docs/plans/post-roadmap-leaf-shedding.md`.

* Slice 3b's dim cold weeks cost the seedling half its carbon. Booked flow by flow, it is not
  starving — growth and upkeep nearly cancel — it is shedding: 0.073 mol C in 56 days, leaf 0.034,
  root 0.033, stem 0.007. The model removes a fixed share of each organ every day (0.02, 0.01 and
  0.005 a day, none cited), whatever the temperature or the crop's stage.
* Three crop-model books on the shelf agree that a crop sheds no leaf from age before it flowers,
  and that what comes after is paced by temperature or development: Penning de Vries et al. 1989,
  Soltani & Sinclair 2012 (Box 9.1, Eqn 9.7), and Teh after Goudriaan & van Laar 1994 (Eqn 7.17).
  Teh models no root death at all; Penning de Vries none before flowering, and no stem death.
* The same form was refused in July because it broke the sealed chambers. That refusal is dated:
  of its four blockers, the canopy one was discharged by the shading rule built in August, the
  RK4 one is not a reference gate, and two are still gates (the long-run chamber's CO₂ floor and
  the Greenwood biomass cap). (The CO₂ floor was first cited at the wrong line — that line is its
  sibling leaf floor; both exist. Corrected the same day.) The tree has changed enough since that none of July's numbers
  stand.
* Options for the user: A, the sources' form (cited; a biosphere unfreeze that must be measured
  against the chambers); B, today's rates counted per degree-day (uncited, so lab-only); C, leave
  it recorded. A measurement plan, run through the existing lab seam with predictions first, is
  owed before A or B can be chosen.

**MEASURED 2026-10-06, lab only** (note §7–§8), on the user's *"do what is closer to reality"*,
read as: measure the books' form. A lab copy of the shedding rule, switchable organ by organ,
reproduces today's runs bit for bit at today's rates (the control), then runs the books' form
across every plant scenario and the sealed station.
* The part all three books agree on — no age-related shedding before flowering — is what the
  cold seedling needed: it ends the cold weeks at 0.152 mol C instead of 0.077, as predicted.
* July's refusal does not reproduce: no rationing anywhere, and every chamber passes every
  gate formula, though the long-run chamber's CO₂ margin thins (1.22 → 1.09 of its floor).
* After flowering the two cited readings disagree, and this model's grain fill is sensitive to
  it. Teh / Soltani take the leaves to zero by maturity: the canopy looks like WOFOST's, but grain
  falls (open field 8.9 → 6.2 t/ha, the station's per season 24 → 11 mol C). Penning de Vries'
  milder table keeps leaves green: grain rises (open field 9.6 t/ha, the station's 61 mol C) and
  the open field breaks the frozen Greenwood biomass cap. WOFOST, the offline comparison, has
  11.5 t/ha with no leaves left — so the model fills grain too slowly, a second gap, recorded.
* My prediction that the books' form would RAISE the open field's biomass had the wrong sign
  (it was read off July's measurement, which used the milder table); recorded, not re-fitted.

**SECOND REVIEW 2026-10-06** (note §9) — and two of the findings above were wrong.
* The station's "grain halves" was counted on the re-sow eve, 166 days after maturity, while the
  frozen crop kept green leaves and kept filling grain. Counted on the day the crop matures, every
  source form gives MORE grain than today's (+14 % to +58 %); in the open field they sit within
  −10 % / +8 %. Re-sowing on maturity (slice 4) removes that standing period.
* "Slow grain fill, a second gap" is withdrawn: the WOFOST comparison is a different cultivar, a
  finding already on record (the oracle-match plan, "ceremony 2").
* The price of the part all sources agree on (no shedding before flowering), counted on a
  throwaway build: every survival gate passes, including the decade CO₂ floor; 29 tests go red —
  the goldens, the pinned margins, the flat form's own tests, a set of measured pins, and one
  science proxy (the sealed jar's leaf area reaches 1.02 against a 1.0 bound standing in for the
  source's 6.0).
* Recommended to the user: the agreed half is the candidate; the after-flowering half waits for
  slice 4.

**BUILT AND PARKED 2026-10-06** (note §10–§11). The user chose to adopt the agreed half, and to
restate the chambers' leaf-area bound to the source's 6.0. Built on its own branch, the change
reproduced the lab's predictions exactly. Then one test showed what else the flat rate had been
doing: a crop short of water used to lose its canopy (leaves dying faster than drought let them
grow back), and reaching deep water rescued it twelvefold; without the flat rate the droughted
crop keeps its leaves and the rescue all but vanishes (1.03× canopy, 1.22× grain). The model has
no drought-driven leaf death of its own — the flat rate stood in for it. The user paused the
adoption to add a cited drought shedding first, so both land together. The build waits on the
branch `wip/leaf-shedding-adoption`.

**ADOPTED 2026-10-06 (a biosphere unfreeze)** (note §13), on the user's "resume". A crop now sheds
no leaf or root from age before it flowers — what all three books say. The change landed exactly
as its lab twin predicted, to every printed digit. Every survival test passes; the closed
chambers' CO₂ margins fell (the jar's most) and were re-pinned with that direction stated. The
open field's canopy now reaches the size where the cited shading rule takes over, which is what
July's diagnosis said the flat rate had been hiding. Three findings came with it and are recorded
in the tests: the deep-water rescue all but vanished; a smaller sealed jar now reads healthier at
every size; the chambers' leaf-area bound was restated to the source's 6.0 by the user. The
station's cold-week seedling no longer loses half its carbon (0.146 mol C at the end of the cold
weeks), and its grain at maturity rises from 9.5 to 14.6 mol C. Two gaps stay open: no sourced
drought leaf death, and water use that does not read the crop.
