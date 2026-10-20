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
* **The user's answers (2026-10-20): four crew members, BVAD water in the same change, the crew-only cabins started
  at the target.** Four crew members drink 122 kg in the 7-day crop cabins' 50 kg store, so it becomes 200 kg
  (today's margin). Predictions at four were committed before measuring (`5edcd6e`).
* **MEASURED, lab-only** (`station/examples/crew_size_lab.rs`; plan §10). All five frozen sides reproduce their
  goldens' bytes; every switched run `rationed 0`, `events 0`. Every predicted number held but one: the sealed station's
  food store 373 222.598, CO₂ scrubbed 120 216.8, crew water 17 869.0 (the store holds), brine 2130.6, the cabin
  12.303 mol (1295 ppm) and no day ending below the target (day ends, not steps). Sealed grain peak −0.33 to
  −0.35 % in each 305-day year; re-sow days identical. **The miss: the final leaf, −0.06 % against about −0.4 %.**
  A draft blamed the final crop's stage; tested, the leaf at that age moves −0.06 to −0.07 % after all eight
  sowings, so it is the leaf's own low sensitivity at that age, not chance. Why is not measured.
* **Three unpredicted moves, explained.** Cabin humidity +11.1 %: the condenser's offset scales with the crew's
  humidity, the same mechanism as the CO₂ offset, left out of the table; ⚠ it deepens the recorded defect that
  the shared air's two vapour stocks are never summed (116 % → 121 % of saturation). The harvest build's soil
  carbon −69.9 %: that build sends the feces into its soil. The sealed station's energy about 0.01 %: by
  elimination the plant room's heater (four flows on the battery; solar, load and lamp read fixed forcings).
* **Adoption is the user's call**: a station unfreeze, five goldens and the scenario's form.
