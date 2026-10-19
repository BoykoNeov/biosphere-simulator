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
* **The form.** The food rate is computed at run time, `N × BVAD's CO₂ load ÷ the loaded f_resp`, as the science
  gate's cabin is, so `f_resp` exists once. ⚠ A constant food-carbon total (24.654 + 1.332 = 25.986 mol C/CM-d) was
  the first draft and was withdrawn on review the same day: its 1.332 carries the same 50 % feces-carbon assumption
  that made `f_resp` 0.949, so it is a hidden second copy. Water re-derived from BVAD: +1.065 %.
* **A coverage hazard, found before building.** The two crew-only cabins start at 0 CO₂; at one crew member they reach
  the scrubber's target at step 712 of 900, so the scrubber would be tested on a fifth of the run and no golden would
  say so. Proposed: start them at the target, as every other cabin store starts at its own.
* **Predicted at one crew member** (before any measurement): food draw and fecal carbon ÷13.3; the cabin 12.568 →
  12.217 mol (1323 → 1286 ppm); the sealed station's plant stocks about −0.5 % (its golden holds no grain; the
  soil-water split may move freely, its total within 1 %); 15 of 20 goldens byte-identical. The authored bed does not
  read the station's crew (read in `bed.rs`'s build path). The crew's body heat is in no room's books.
* **Not closure.** One crew member still needs about 14 m² of crop; the bed is 1 m². The crew-coupled loop stays
  refused.
* **The crew count is the user's choice** (one recommended; four needs the water store measured; 13.3 with water to
  match keeps a number nobody chose).
