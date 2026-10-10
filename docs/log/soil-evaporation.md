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
  water in it — less of a drop than the predicted 0.58 (the predicted feedback had the wrong sign).
* Why, measured after review: **every watering of a full-depth root zone in a sealed chamber is capped by
  the condensate tank** (52/52, 48/48, 43/43, 41/42 over fifteen seasons; none while roots are shallow) —
  the chamber holds too little free water to refill a 1.30 m zone. The drained 19 kg is spent on those
  refills (fill after an event 0.91 → 0.955), so more returns to the deep soil at each re-sowing.
  ⚠ For the pricing: "the whole deficit per event" is impossible in the sealed chambers at full depth.

**The stale comparisons re-run and the price measured (2026-10-08,** plan §11). The "three changes" are
five loader settings (the two deep-soil fixes come with watering in events).

* Re-run under the settled watering: the re-sown chambers water about a third as often (perennial 63 → 23
  events in five years), the soil replaces ~0.43 of the dead crop's phantom water (was ~0.21); carbon
  untouched. The six biosphere twins regenerated.
* The price, on a local branch (`wip/three-forms-price`, never pushed): 10 of 20 goldens move, each
  biosphere one equal to its twin byte for byte; **water only** everywhere, station included. Suite 24 red +
  2 ignored red, all classified: 19 regenerate or restate by construction, 4 need a design choice (the
  plain re-sow refuses the soil state — found by the golden producer crashing before anything ran), 3 are
  unexplained behaviour changes in instrument tests. Manifests: 26 lines; ~11 constants owed to param files.

**The top-layer cap was ours, not the book's (2026-10-08,** plan §12). The proposed fix, "capillary rise into the
top layer", was dropped on reading the book: it leaves capillary rise out (no water table), and its own program
never caps the soil's evaporation at the top layer's water — Stage II draws from the whole root zone and the top
account only floors at 0. Our build had added that cap. Built as a lab switch (`SoilSupply::TopLayer` — the cap,
still the default — or `RootZone`, the book's) and measured under events:

* The never-re-sown chamber's dead-crop air: 18 % → **61 %** of the frozen model's, but **the success test still
  fails**. A probe: no watering falls in the dead years (the last, as the crop dies, opens them with ~50 wet
  days); after that the air swings with the season, down to ~4 % of its target at the low point — the book's
  Stage II tail.
* Soil evaporation rises 36–83 % in the event rows, paid for by more watering (5→6, 23→26, 21→26 events); carbon
  identical everywhere; the drought bites 3 days sooner.
* Logged, not changed: the book restarts its dry-day count at 1 (first dry day 0.41 of the potential), ours at 0
  (1.0).

**Re-priced under the book's supply (2026-10-08,** plan §13; the user: *"go with your recommendation"*). Evidence
for the supply choice, not the choice. The local branch rebased, a control first (the cap's price reproduced
byte for byte), then the book's supply in the loader:

* The same 10 goldens move, the same 24 + 2 reds by name and class, the three unexplained reds fail with the same
  messages — **the two caps cost the same ceremony.**
* Water only again, station included; no carbon value moves anywhere.
* The sealed chamber's end air: **0.125 kg** under the book's supply against 0.000002 kg under the cap (frozen
  0.25); the 7-day station chambers' air 1.1–1.2 kg against 0.5–0.7 (frozen 1.9). More soil water used in the
  open field (soil evaporation 33.6 → 45.9 kg).
* The choice is the user's; the recommendation is the book's supply (cited; the cap was ours).

**DECIDED 2026-10-09: the whole root zone supplies the soil's evaporation** (plan §14; the user: *"Strive for
what is closer to reality"*). A drying surface keeps evaporating, more slowly, from the moist soil beneath it;
the cap stopped it dead at an empty bookkeeping layer. Not yet the go-ahead to adopt. Owed first: trace why the
sealed station's below-root store ends 19 % wetter. Logged as a separate realism lead: the condenser draws
vapour below its humidity setting, which a real dehumidifier would not, and may be what dries the dead chamber's air.

**2026-10-09, after the dehumidifier fix** (plan §14–§15). The condenser drew vapour below its setting; fixed
as its own biosphere unfreeze (`log/chamber-dehumidifier.md`). The never-re-sown chamber's dead-crop air is now
0.981 of frozen under the root-zone supply (S4 passes) and 0.704 under the old cap. §13's sealed readings are
stale. **B6 explained:** over the run the sealed station's below-root store is the same under both supplies
(78.7 vs 78.8 kg mean). It swings ~20–160 kg each season, and the run ends at a trough whose depth depends on
when the last waterings fell. A snapshot, not a wetter store.

**ADOPTED 2026-10-10** on the user's *"Step 2"* (plan §16), in two commits. The plumbing first, with no
golden moving: twelve coefficients moved from code into `transpiration.yaml` and `water_cycle.yaml` with their
sources, and the params-free re-sow retired (a re-sow now takes the params the run was built from). Then the
switch: Szeicz–Long, the two-stage soil evaporation drawing on the whole root zone, watering in events, the
actual-wetness credit and the recycled overflow are the loader's. **10 goldens moved, water only** — the
sealed chambers keep 6–11× more water in the tank between waterings, the root zone sits lower, the air ends
where it did; the open field's soil evaporates 45.9 kg into its own sink. Every prediction held except a
miscount of the reds (I wrote 18; the arithmetic gave 20). The three reds that looked like findings had
ordinary causes: the drought bound was a per-step law stated as a season one; the recycled overflow is a
second route to the deep water; an infinite param is now refused at the build. ⚠ **Under event watering the
open field's `irrigation_mm_day` is only an on/off switch** — the book's deep-store tests, which need a crop
watered below its demand, a dry store or a fixed-point re-sow cycle, now pin the five old forms, kept as lab
switches. The cross-port bands for the ten moved goldens are read on Linux CI.

**After the push (plan §16h).** Linux CI: one red, the biosphere band's reach check (Linux read the
last-digit sensitivity 9× Windows, 1.4 % past a window around the retired Python figure; the band itself
held). The user: *"I don't care about Linux, drop its tests if needed"* — that one check runs on Windows
only now; why Linux reads higher was not measured. **The real-world check:** TM 102788's water use went
0.40 → 0.43 of the trial's; carbon uptake unchanged. **Live gap:** the separate-air option's own watering
does not feed the soil's top-layer account (fix offered).
Linux CI green on `a5332e8`, the moved goldens' bands included.

**The separate-air gap closed (plan §17, 2026-10-10).** The soil's top-layer account now counts every flow
that waters the root zone, not just the biosphere's own: the lab separate-air build hands it its watering
from the crew's store too. The reference is untouched (20 of 20 goldens identical, the one-source sum adds
in the same order). The gap was small in practice: that watering fires about 10 and 22 times a season in
0.5 kg steps, so at most 5 and 11 kg had been missing; the top layer is now dry on 3 and 2 fewer plant steps
of 4880. **Found on the way, not fixed (an unfreeze, the user's call):** the four station assemblies never
seed the soil account's starting values, so every station run's first season starts with an EMPTY top layer
and unshaded soil; the re-sow seeds them. Same day, separately, the user deleted the Linux CI job that ran
the reference's tests; the slow tests now run only by hand.
