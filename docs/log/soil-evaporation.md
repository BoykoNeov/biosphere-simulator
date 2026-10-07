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

**Watering in events DECIDED 2026-10-07 (the user):** FAO-56's depletion trigger (`p = 0.55`) —
nothing until the root zone has used 55 % of its water, then a full refill; the sealed chambers'
condensate is the reservoir. Raised because daily top-ups would keep the top layer always wet.

**BUILT lab-only and measured 2026-10-07** (plan §8). Three switches, all off in the loader; the
frozen path is bit-identical (asserted; 20 of 20 goldens).

* With daily top-ups the surface never dries (Stage I 100 %), and the dead-crop chamber's air sits
  at 0.999 of frozen — its humidity cap; the soil supplies only 0.56 of the dead crop's phantom water.
* With watering in events the surface dries between waterings (Stage I 9–43 % of steps), drought
  bites a week sooner, and the chambers cycle a third or more less water.
* Under events the never-re-sown chamber's air dries again while its crop is dead (0.20): its root
  zone is 1.30 m deep and loses water only from the top 150 mm, so it never reaches the trigger — a
  fallow chamber with a dry crust. The re-sown chambers keep their air (0.94–1.00).
* Carbon never moved in any configuration. Of 10 predictions 6 held, 1 mostly, 1 failed, 2 split by
  watering (corrected; first recorded as 7 held). A test caught a reset refusal the first edit missed.

**The top layer overfilled under events, CAPPED 2026-10-07** (plan §9). An event refilled the root
zone into a 19.5 kg top account, peaking at 4.8× its capacity. The user chose the book's saturation
limit for silt loam (Soltani & Sinclair Table 13.1, up to 2.65×). Measured with the cap:

* Peak fill 2.65× in every event run; event-watered soil evaporation and wet share fell in three
  runs and ROSE in the perennial chamber (more waterings with less room on top).
* Under daily watering the soil supplies 0.56–0.96 of the frozen model's dead or sparse crop's
  phantom water over its dead days; under events 0.01–0.74. It replaces part of it, nowhere all.
* Still owed before any station pricing: a lab re-sow hook (the station's uses the plain reset,
  which refuses these values), event recycling in the station, and the lab air-split watering,
  which does not feed the top-layer account.
