## **A cited air resistance for every sealed chamber** (the price's successor — Monje & Bugbee 2019's measured 7.5 s/m, found after the price closed, adopted for every sealed room; the open field keeps 50)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-15** on the user's call (*"1"*). A biosphere + station unfreeze and a science item. Plan,
predictions committed before code, and grading: `docs/plans/post-roadmap-chamber-air-resistance-adopted.md`.

* **The user's choices:** the cited 7.5 s m⁻¹ (not a judgement value), for every sealed chamber (9 goldens
  in the price), not lamp-lit only. The open field keeps the uncited 50.
* **The caveat, on the table when they chose:** the source's fans were faster than NASA's chamber's, so
  in a slower-aired room 7.5 is the best-ventilated end of the range, a lower bound on the resistance.
* **The number is the model's air's:** the paper's 5.5 mol m⁻² s⁻¹ converts to 7.56 s m⁻¹ at the
  model's sea-level air density, so 7.5 and not the 6.4 of the paper's altitude.

**ADOPTED 2026-10-15**, in two commits: inert plumbing (`chamber_aerodynamic_resistance` in
`transpiration.yaml`, the selector `aerodynamic_resistance_for(sealed)`; no golden moved), then the flow
reads it.

* **What moved:** exactly the 9 predicted goldens (5 biosphere, 4 station), water only, bit for bit — no
  carbon, nitrogen, oxygen or energy value anywhere. The price's branch at 7.5 gives all 21 golden files
  byte-identical, so the adopted shape is the priced change.
* **The NASA comparison:** on the user's call the TM 102788 scored row reads the chamber value. Its water
  use goes from 0.62 to 1.04 of the trial's; nothing else in the row moves (now asserted). The source's
  coupling is a best case, so "the gap closes" is an upper reading, not a match.
* **What the suite found:** the re-sow cycle settles in two cycles, not one (predicted; restated, not
  widened, and the test runs a sixth year to keep comparing three). Hidden behind it, not predicted: the
  chamber's whole water total drifts down by rounding, the same sign every cycle — present at 50, faster
  at 7.5. The user chose a stated rounding budget (2e-14 a cycle) over hunting it first; **the lean is an
  open item.** The price's record missed this check at 9 and carries a correction.
* **Not measured:** the energy side — evaporation's heat and the dehumidifier's load — is unmeasured, not
  zero. Also open: the sealed station's condensate drain is not monotone in the value (cause not found).

