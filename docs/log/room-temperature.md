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

**Slice 1 BUILT 2026-10-01 (lab only, no golden).** With the plants held at 22 °C, the frozen
winter wheat never accrues a chill-day, so it never develops and never sets grain. It grows
29.91 of leaf, stem and root instead, and the first re-sow refuses with "seed bank too small".
All three predictions were committed before the code and all held. The control (the plain
season develops) and a liveness check (all three go red at 8 °C) both pass. Check:
`rust/crates/station/tests/warm_room_arrest.rs`. Slice 2 is a station unfreeze and was not started.

**EXTENDED 2026-10-03** (note §11). The user asked for separate air as an *option*: crew and plants
in two air spaces joined by a fan that carries gas and heat. Decided: an option, **lab-only first**
(shared air stays the default, every golden unchanged); the cabin gets **its own heat store**
(the held-22 °C recommendation was declined); the air split comes first, gases only; the cold-period
source search runs alongside. Found while sizing it: BVAD Table 4-88 (p. 170, page image not yet
read) gives a plant chamber 0.67 m³ of air per m² of crop and 0.3 kW/m² of fan power. For the
station's 1 m² crop that is 27.66 mol of air, 343× smaller than the cabin, and the crop's measured
worst slow-step CO₂ draw scales to ≈ 27× the chamber's whole CO₂ (⚠ corrected the same day to
≈ 6.7×, measured — the 0.078 it rescaled was a quarter-day-step figure). The fan resupplies on the 60 s
step but the crop draws on the 1.5-hour one, so a source-sized chamber starves by construction of
the step. Measured first in slice 2a; the remedy is the user's call. No inter-room ventilation rate
found in BVAD (its "Ventilation" row is an air speed).

**Cold period DECIDED 2026-10-03** (note §12). The standard protocol binds (Cha et al. 2022, opened
as the accepted manuscript): **8 weeks at 4 °C**, the centre of its 6–10 weeks at 2–6 °C, with the
lamp dimmed to 100 µmol m⁻² s⁻¹ and the day cut to 8 h for the cold phase. The 6- and 10-week ends
become lab sensitivity checks. The model's wheat reads a full vernalization (56 chill-days ≥ 50).

**Slice 2a BUILT 2026-10-03 (lab only, no golden)** (note §13–§13b). Separate air is
`station::air_split`: the cabin's own CO₂/O₂/inert fill at 9500 mol, the plants in a BVAD-sized
27.66-mol chamber, and a fan moving every species by its concentration difference (Q = 0.2 mol/s,
DESIGN). The fast flow list was extracted from the sealed builder, not copied; all 20 goldens are
unchanged. Over one season the crop draws **0.174** of shared air's CO₂ and ends with **0.144** of
its carbon — starved by the 1.5-hour plant step, not by the fan (0.1 and 0.4 mol/s give the same
numbers to five figures; a cabin-sized chamber gives 0.999). With vapour crossing, **23.3 kg** of
the plants' water drains into the crew's store in a season, because the cabin sits at ≈ 1.5 %
humidity. Four predictions failed and are recorded (9 held): the rooms' total gas is not held at
reference (the chamber's vapour adds 1.96 %, as on shared air); the cabin's CO₂ moved ±0.14 %, not ±0.01 %;
water flowed back on 78 of 439 200 fan steps; transpiration rose 2.4×, not 3–4×. Found on the way:
transpiration ignores canopy size (a crop with 14 % of the carbon transpires 99.6 % as much).
Three decisions are the user's: how the plant step meets the fan, a water return path, the cabin's
humidity.

**Cabin humidity FIXED 2026-10-03 (a station unfreeze)** (note §14–§15a). The user took all three
of slice 2a's decisions; the order is humidity, gas exchange, watering. The cabin's condenser drew
on all its vapour and held every cabin at ≈ 1.5 % relative humidity. It now holds BVAD Table 4-1's
nominal **40 %** (printed p. 63, page image; 25–75 % range), derived to 1.7863 kg at 22 °C in the
9500-mol cabin, and acts only above it. Six goldens moved: the cabin's vapour by exactly +1.7863 kg,
the crew's water books in the last few bits, no plant-side byte. Graded: 3 held, 1 half (the
manifest has no param-name row), 1 failed. The failure: in the lab's separate air, water now flows
cabin → plants (+2.08 kg a season), because the plant chamber still runs at the weather's
temperature. Found, recorded, not fixed (the user's call): shared air holds two vapour stocks whose
targets add, so the room counted whole is above saturation on 4 876 of 4 880 plant steps (8 before).
Nothing reads that sum. Also found: the multi-rate ECLSS authoring anchor would have gone dead under
a dry start, and nothing ran its trajectory since S6; a test now does.
