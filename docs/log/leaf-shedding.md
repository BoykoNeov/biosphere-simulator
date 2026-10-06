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
  the Greenwood biomass cap). The tree has changed enough since that none of July's numbers
  stand.
* Options for the user: A, the sources' form (cited; a biosphere unfreeze that must be measured
  against the chambers); B, today's rates counted per degree-day (uncited, so lab-only); C, leave
  it recorded. A measurement plan, run through the existing lab seam with predictions first, is
  owed before A or B can be chosen.
