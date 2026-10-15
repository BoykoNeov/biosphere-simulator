## **The heat the chamber's water carries, estimated** (the air-resistance adoption's open energy item — the crop transpires at night with no heat to pay for it, at the old value and the new; 7.5 made it 2.7× larger)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-15** on the user's *"Estimate it first"*, after being told the item as first offered was
wrong: the model gives evaporation no heat, so its own energy books measure zero by construction. Lab-only;
nothing frozen moved. Plan, predictions committed before the run, and grading:
`docs/plans/post-roadmap-chamber-latent-heat.md`. Instrument: `rust/crates/station/examples/chamber_latent_heat.rs`,
run on main (7.5) and on `4f6d7dc` (50); every control held on both (no rationing, legs bit-exact, golden
byte-exact).

* **Where the heat would go:** BVAD p. 145 — one condensing heat exchanger takes sensible and latent heat
  into the same coolant loop. So evaporation only splits the cooler's work, **as long as there is heat to
  split in that hour**: a held room stores almost none (0.053 K per minute of full lamp).
* **The finding:** the model's crop keeps transpiring at night — 92 % of its lit rate at 7.5, 66 % at 50 —
  because the stomata never close (the surface resistance reads leaf area, not light; checked in the code)
  and the room holds 75 % air. With the lamp off, as in any real room, the dark hours alone need **1.80 GJ
  at 7.5 and 0.67 GJ at 50** over 1220 days from somewhere. The new value made it 2.7× larger; it did not
  create it.
* **The netted figure, kept as the most favourable reading:** over the whole run at 7.5 the water carries
  80.7 W against the cooler's 77.9 W (0.29 GJ short); at 50, 43 W (a surplus). ⚠ A first draft headlined this
  as "50 held, 7.5 breaks" — an averaging artefact, corrected before reporting.
* **The room cannot see the night:** its lamp heat is a daily average on every minute, so the heater fired
  421 minutes in 1220 days. Two predictions (P6, P7) were refuted by that, not by the water.
* **Missed:** P1 (mean power) on the horizon — 1220 days, not four years.
* **Not decided — three follow-ons, each its own call:** night-time stomatal closure (a lead, the ASCE-EWRI
  2005 hourly night resistance, is not on the shelf); lamp heat switched with the light; latent heat in the
  room's books (worth taking only after the other two). Recommended first: night-time closure, the dominant term.
  **Night-time closure ADOPTED 2026-10-15** (`log/night-stomata.md`): the dark-hours shortfall is ~0.31 GJ
  now, not 1.80. **The room's lamp heat switched with the light ADOPTED 2026-10-15** (`log/room-lamp-heat.md`):
  the heater now fires on the warm nights; 0.10 GJ of dark-window water heat is not covered. The third,
  **latent heat in the room's books, ADOPTED 2026-10-15** (`log/latent-heat-books.md`): the heater 0.31 →
  1.34 GJ, by day too; all three follow-ons are adopted.
