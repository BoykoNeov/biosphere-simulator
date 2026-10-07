## **Bare-soil evaporation** (opened from the canopy-resistance price — with a leaf-area resistance a dead crop stops transpiring, and the model has no soil evaporation to take its place)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-07** on the user's call (*"Soil evaporation first"*), after the canopy-resistance
price showed a never-re-sown sealed chamber's air drying out while its crop is dead. To be built in
the lab and then priced together with that resistance. Plan: `docs/plans/post-roadmap-soil-evaporation.md`.

* Source, read first-hand: Soltani & Sinclair (2012) Ch. 14 — potential bare-soil evaporation from
  the radiation reaching the soil (a simplified Penman, extinction 0.5, soil albedo 0.12), a
  1.5 mm/day floor (Amir & Sinclair 1991), and a two-stage wet/drying rule.
* Decided by the user: the form with an energy split between soil and crop; the floor as a switch,
  measured on and off; the wet stage read from the soil's own water rather than the book's 10 mm
  rewetting rule.
* A mapping that removes a choice: the model already treats root-zone water as uniform, so the
  book's top-layer test reduces to "the root zone is above half full", and the top-layer depth
  (150–600 mm in the book) drops out.
