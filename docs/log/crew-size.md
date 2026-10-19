## **The station's crew size** (the cabin CO₂ item's named successor — the station's crew breathes 13.3 crew members' carbon and drinks one's water, a size nobody chose)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-20** on the user's *"yes"* to taking the crew-size fix they chose on 2026-10-19. A plan with
predictions; no value, golden, manifest or `source:` string moved. Plan: `docs/plans/post-roadmap-crew-size.md`.

* **Today, measured.** `CABIN_GAS_SCENARIO` eats 4.0e-3 mol C/s and drinks 5.0e-5 kg/s. Against BVAD Table 3-31 that
  is 13.30 crew members' CO₂ (328.0 mol/d over 24.654) and 0.99 of one's water (4.32 kg/d over 4.366). Every station
  build inherits it. The sealed station eats 84 % of its 500 000 mol food store over 1220 days.
* **Four sizings, one in scope.** The standalone crew (`MISSION_SCENARIO`) is 1.66 / 0.59 / 3.09 (CO₂ / water / O₂);
  the ECLSS stand-in 10.5 / 0.59 / 12.4. `bioregenerative_station.yaml` calls the first *"one crew member"*,
  `lamp_greenhouse.yaml` the second *"a crew of one"*, and `crew_habitat_template.yaml` multiplies the first per
  crew member. Recorded, not touched: a science change and a re-labelling do not share a batch.
* **The form.** The food rate is BVAD's food-carbon total (24.654 + 1.332 = 25.986 mol C/CM-d), so no second copy of
  the loaded `f_resp` is typed into a constant; breathed out it is +0.028 % over BVAD's CO₂ (the rounding of 0.949).
  This departs on purpose from the science gate, which calibrates to the CO₂ load. Water re-derived from BVAD:
  +1.065 %.
* **A coverage hazard, found before building.** The two crew-only cabins start at 0 CO₂; at one crew member they reach
  the scrubber's target at step 712 of 900, so the scrubber would be tested on a fifth of the run and no golden would
  say so. Proposed: start them at the target, as every other cabin store starts at its own.
* **Predicted at one crew member** (before any measurement): food draw and fecal carbon ÷13.3; the cabin 12.568 →
  12.217 mol (1323 → 1286 ppm); sealed grain −0.2 % to −1.4 %; 15 of 20 goldens byte-identical. The crew's body heat
  is in no room's books, so nothing there moves.
* **Not closure.** One crew member still needs about 14 m² of crop; the bed is 1 m². The crew-coupled loop stays
  refused.
* **The crew count is the user's choice** (one recommended; four needs the water store measured; 13.3 with water to
  match keeps a number nobody chose).
