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

**Re-pricing all three, opened 2026-10-07** (plan §10). The station's re-sow hook now uses
`annual_reset_with` (frozen-identical: 20/20 goldens, station suite 241 passed). The floor left
off — closer to reality in a lamp-lit chamber, the user's delegation.

* Found while preparing the twins: event watering slowly empties the store below the roots. Each
  re-sow returns the root zone's water at its harvest fill (half full under events), and watering
  only to full never recharges below. Roots stopped at 0.94 m in season 2, 0.16 m after 15 years.
* The stopping depth is the book's single deep store credited at full capacity per metre of new
  root (`EWAT = min(GRTD·EXTR, WSTORG)`, a single-season model), not dry soil — predicted exactly.
* The user chose to water a little over full: FAO Manual 4's field application efficiency
  (sprinkler 75 % in the field, drip 90 % in chambers), the loss percolating the same day, as
  both FAO sources assume. The deep store then holds near the frozen run's (roots 1.18 m in
  season 2, as frozen; 1.15 against 1.30 after five seasons).
* The earlier twins and event numbers predate this and are stale; regenerated before any flip.

**Same-day percolation committed (`cb3a305`); new roots credited with the deep soil's actual wetness,
built lab-only (2026-10-07,** plan §10e–f). The user's two decisions: credit new roots by how wet the
deep store really is (`min(1, WSTORG / capacity below the roots)` — our mapping, not in the book; equal
to the book while the store is full), and accept the slow residual drying.

* Over fifteen seasons in the perennial chamber with all lab forms: roots reach 1.30 m every season
  (the book's credit: shrinking to 0.90 m); the deep soil settles at 69 % full after each re-sowing,
  flat from season 9 (frozen 96 %; the book's credit ~44 %). The drying the user accepted is a level,
  not a ratchet.
* Carbon identical to four digits under all three. Frozen goldens untouched: the book's path computes
  no ratio.
* Bounded quirk recorded, not fixed: ~19 kg sits above the bottom layer's capacity at each harvest (the
  book's deep store has no outflow).
* Still owed before a flip: regenerate the stale twins and the §8a/§9a event rows under the settled
  watering + credit.
* ⚠ Corrected after review: the 69 % includes ~19 kg held above the bottom layer's capacity, mixed back
  in at each re-sowing — ≈ 58 % without it. The consumer chamber (the old ratchet's worst case): roots
  1.30 m in all fifteen seasons; deep wetness 68–95 % with no trend; grain identical. Whether to let that
  overfill drain away is the user's open decision.

**The deep overfill drains and is recycled, built lab-only (2026-10-08,** plan §10g–h). The user's
decision: *"Let it drain and be recycled."* A fifth lab switch (`DeepOverflow`; the loader keeps the
book's `Held`, which registers nothing, so the frozen path is untouched — 20 of 20 goldens identical):
water above the deep store's own capacity drains at the book's 30 % a day back to the water the crop is
watered from — the condensate in a sealed chamber, the irrigation source in the open field (our mapping).

* Fifteen seasons, both chambers: the deep store sits at its capacity at harvest (was 16–38 kg over);
  roots 1.30 m every season; grain identical to four digits; nothing rationed.
* The deep wetness after re-sowing settles at ~0.63 (perennial) and ~0.69 (consumer), with no stranded
  water in it — less of a drop than the predicted 0.58, because about half the drained water came back
  into the root zone through the watering (water conserved to 0.1 kg). The predicted feedback had the
  wrong sign.
