## **The plants' temperature in a station room** (the 2026-09-29 review's Step 3, slice 3c — a design note: a held room is cheap, and what it costs is the crop and the calendar)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-01** as a design note only, on the user's agreement to *"the Step 3c design note
next"*. No code. Note, sources and predictions: `docs/plans/post-roadmap-room-temperature.md`.

* The station's heat store is not a room. Measured in the frozen `sealed_station` end state, it
  sits at **167.42 K (−105.73 °C)**, set by a DESIGN radiator area. Wiring the plants to it as it
  stands would put them at −106 °C.
* BVAD Rev 2 gives the room, read from the page images: cabin air 291.15 / **295.15** / 300.15 K
  (Table 4-73, p. 153); wheat 23 °C under a 20 h photoperiod (Table 4-111, p. 188). The second is
  another model's wheat, so it is at most a class citation.
* A warm room arrests the frozen winter wheat. Its chill-days accrue only below 12 °C, and it
  does not develop at all until about 19.7 have accrued. The day-neutral crop exists. At 22 °C it
  would mature in ≈ 84 days against a 305-day re-sow calendar.
* Forms: A, a held constant (recommended); B, a held room that can fail, which is A plus heat
  books; C, a free-floating room, recommended against. Net radiation is out of scope by name.
* Four decisions are the user's (form, crop, calendar, setpoint value). Nothing in `rust/`
  changed.

**DECIDED 2026-10-01** (note §10). The user chose form **B**, the winter wheat with a **cold
period**, **re-sow on maturity**, and **22 °C**. Asked next, because crew and plants share one
atmosphere and a cold period sits far below the crew's cited 18 °C floor, the user chose a
**dedicated plant chamber** with its own temperature and shared air. That is cheap here: the crew
reads no temperature, and the plants' vapour store is already separate from the crew's. The cold
period's length and temperature must come from a source, not from the model's own saturation
point; candidates were found by search but none is bound yet. Four slices, none built.
