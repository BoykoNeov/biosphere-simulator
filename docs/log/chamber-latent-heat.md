## **The heat the chamber's water carries, estimated** (the air-resistance adoption's open energy item — at 7.5 the crop's water takes about the whole lamp's heat, and the night it transpires through is a night the room's heat books never see)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-15** on the user's *"Estimate it first"*, after being told the item as first offered was
wrong: the model gives evaporation no heat, so its own energy books measure zero by construction. Lab-only;
nothing frozen moved. Plan, predictions committed before the run, and grading:
`docs/plans/post-roadmap-chamber-latent-heat.md`. Instrument: `rust/crates/station/examples/chamber_latent_heat.rs`,
run on main (7.5) and on `4f6d7dc` (50); every control held on both (no rationing, legs bit-exact, golden
byte-exact).

* **Where the heat would go:** BVAD p. 145 — one condensing heat exchanger takes sensible and latent heat
  into the same coolant loop. So evaporation only splits the cooler's total, **until the water's heat
  exceeds that total**.
* **At 50 it did not:** the crop's water carried 43 W against the cooler's 78 W over the run (warm phase:
  0.53 of the lamp). The adoption's "water only" held for the energy totals.
* **At 7.5 it does:** 80.7 W against 77.9 W; in the warm phase 1.01 of the lamp. A coil condensing that
  water removes 0.29 GJ more than the room receives — about 20× everything the model's heater gives.
* **Night transpiration tips it:** in the dark the crop transpires at 92 % of its lit rate at 7.5 (66 % at
  50) — the stomata never close, and the room's held 75 % air keeps the dryness term working. 1.8 GJ, six
  times the shortfall.
* **And the room cannot see the night:** its lamp heat is a daily average on every minute, so the heater
  fired 421 minutes in 1220 days. Two predictions (P6, P7) were refuted by that, not by the water.
* **Missed:** P1 (mean power) on the horizon — 1220 days, not four years.
* **Not decided — three follow-ons, each its own call:** night-time stomatal closure (a lead, the ASCE-EWRI
  2005 hourly night resistance, is not on the shelf); lamp heat switched with the light; latent heat in the
  room's books (worth taking only after the other two).
