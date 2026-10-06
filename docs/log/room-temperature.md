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

**Gas exchange minute by minute ADOPTED 2026-10-03 (a station unfreeze)** (note §16–§17a). On the
user's call, the crop's three carbon-budget flows are stepped on the cabin's 60 s step instead of
the 90-minute plant step, unchanged, reading the light and temperature the plant step records for
its window. Lab first: the separate-air build's crop went from 0.174 to **0.920** of shared air's
CO₂ uptake, and the fan rate now matters (0.863 / 0.920 / 0.949). Redesigned once before adoption
(a resolver copied at build time would have hidden every plant-side perturbation from
photosynthesis). Adopted where the cabin's flows act on the crop's air: 3 goldens moved
(greenhouse +0.29 %, harvest +0.07 %, sealed +0.75 % plant carbon), no water byte. Graded 7 of 7;
one held only after the bug it caught was fixed (harvest discarded the moved flows). The
perturbation suite's "pools return to setpoint" claim now takes out the crop's own flux offset.

**Watering from the crew's supply BUILT 2026-10-03 (lab only, off by default)** (note §18–§18c).
The first form — the field's root-zone refill, sourced from the crew's store — failed 4 of 5
predictions: it watered plants that were not short (+30–45 kg of crew water a season into the
plants' loop, ahead of their own recycled condensate). Redesigned on FAO-56 Table 22's depletion
trigger (wheat p = 0.55: water when the root zone falls below 45 % full, above the crop's 30 %
stress onset): 4 of 4 held. Over a season it delivers 0 kg in every case (on and off byte-identical)
and fires, kg for kg from the crew's store, when the zone is set low on purpose. The drain it guards
against is capped at ≈ 21 kg a season by the plant step's transpiration rule — the next decision.

**Stale checks after the minute-step adoption, cleared 2026-10-04** (note §17c). Three measuring
tools were left un-run after the adoption; running them found one the adoption broke
(`intraday_exchange`) and two census checks that had gone stale earlier, at the 1/16 step and
option C on 2026-09-30 — not caused by the adoption. The
lamp-shedding numbers hold on the minute build (6 of 6 predictions; crop +0.58 %, shedding still
costs it 31.7 %). `intraday_exchange` had been crashing: adoption wired the harvest build to the
minute step with no plant-step option; `build_harvest_at` added, reference path unchanged. The draw
census's control 4 now uses the tenth-size jar (205 firings, matching an independent probe), and its
control 2 still carried the jar pin from before the 1/16 step — printed "NOT reproduced" on every
run while the verdict read "all held", because it was never counted. Both now count, and each was
shown to turn red when broken.

**Leaf water loss on the minute step BUILT 2026-10-05 (lab only, off by default)** (note §20–§20a).
On the user's "lab first", transpiration and the plant chamber's condenser move together onto the
60 s step in the separate-air build (a second recorder holds the window's net radiation, so the
reference's goldens did not move). 8 of 9 graded predictions held. The fan now drains or feeds the
chamber at a full gradient, scaled by 0.697 because it swaps 43 % of the small chamber's air a
minute. At 22 °C without watering the crop dies of drought (169 kg lost, stressed 81 % of minutes);
with watering it delivers 477 kg and the crew pays only the recovery loss (brine +54.09 kg,
predicted to the digit). Missed: the weather-temperature chamber gains 55 kg, not 20–45. A
candidate cause, estimated and not measured: the cabin sits 0.04 kg above its 40 % setting.
⚠ All measured on the outdoor input that §21 then fixed; to be re-predicted, not re-graded.

**A lamp-lit crop's net radiation from the LAMP: a station unfreeze, 2026-10-05** (note §21–§21a).
Found while building §20: every lamp-lit build gave the crop the lamp's PAR but the weather file's
OUTDOOR net radiation, so a sealed crop lost more water on sunny days outside, and a dark lamp left
its water loss untouched. The user: *"fix this, in its current state, it doesnt make sense."* Now
`(1 − 0.23) ×` the lamp's radiant PAR while lit (84.25 W m⁻²), 0 in the dark; the 0.23 is
FAO-56's albedo, the user's choice over waiting for a PAR-specific reflectance no reachable page
gives. 7 of 7 predictions held: 2 goldens move (`sealed_station`, `lighting`), **water stocks
only**; transpiration −20.9 % and −7.5 %; lamp shedding and the lighting failure darken it too,
each pinned. ⚠ Every earlier separate-air WATER figure (§13b, §15a, §18) was measured on the
outdoor input and was not re-run. Still open, the same kind of defect: the chamber's temperature
follows the outdoor weather too (the queued chamber heat store).

**The separate-air water figures re-measured, 2026-10-05** (note §22). The same instruments ran
on the commit before the lamp fix and on the fixed tree, so any difference has one cause. The
pre-fix tree first reproduced the record exactly. All four conclusions still standing hold.
The weather-temperature chamber still gains from the cabin (2.131 kg a season, was 2.083). The
22 °C chamber still drains 21.248 kg, byte for byte, with the plant step's cap setting it on every
step. Watering still delivers 0 kg, with more margin (lowest fill 0.836 / 0.680). Plant carbon is
unchanged. Leaf water loss in the split build falls 22.7 % and 9.9 %. Graded 3 held, 3 half:
night-time leaf water loss is smaller than predicted, so in the weather chamber 456 of 4 880
steps now fall under the cap. Found on the way: `examples/air_split.rs` had crashed since the
minute-step adoption, and its "shared" row had silently become the minute build before the crash.
Nothing runs examples, and the 2026-10-04 sweep of stale instruments missed it. Repaired.

**The plant chamber's heat store, slice 2b-i BUILT 2026-10-05 (a station unfreeze)** (note
§23–§23f). The user took the heat store over the minute-step adoption question, then decided its
design: walls that lose heat to their surroundings, which may be hotter or cooler (*"it can be
environment, it can be inside the station, it can be in space"*); the reference chamber's walls
face the outdoor weather, the cabin, space and the station structure are lab options; a heater
on the battery; the cooler works only into a colder node; a 1-minute response. Priced first: at
most 0.72 % of the lamp's light is stored as sugar (NIST's glucose combustion on the golden's
organic carbon), so all the light becomes chamber heat; BVAD Table 4-88 books every chamber
watt as heat to remove; Table 4-50's freezer-wall conductance gives the walls a class figure.
Three diffs, one cause each. **2b-i**: the lamp's waste heat passes through the chamber to the
node. Every existing stock in the reference run is byte-identical, the node included (predicted
only within 1e-9); 8 of 9 predictions held. A dead cooler heats the chamber 34.8 K a day with
the crop untouched. ⚠ Flagged for slice 3: walls facing the weather do not make the plants feel
the weather; which temperature the plants read is the calibration question.

**Slice 2b-ii BUILT 2026-10-05 (a station unfreeze)** (note §23g–§23h). The lamp's light now
heats the chamber too, instead of leaving the station: the station's heat store warms 7.5 K, to
174.96 K, exactly the closed form; nothing outside the energy books moves. Graded 8 held, 1
half: a lab test's tolerance, sized for 2b-i, was carried over without re-deriving it and went
red; replaced by a derived bound. The lab's lamp shedding detects "lit" from the lamp's own draw,
since its old detector's stock is gone.
Removing `boundary.light_used` from the sealed build: no Godot script, authored scenario or
runtime file names it (searched every non-Rust file). An older saved sealed session (without
`thermal.chamber`, with `boundary.light_used`) is refused at load with a "stock-id set does not
match" error (`SimSession::load_state`'s set-equality guard — read, not run); no migration is
offered. `lamp.yaml`'s header still draws the two-target `Lamp` form, which the `lighting` build
uses; left as is (editing it would move its frozen digest for a comment).

**Slice 2b-iii BUILT 2026-10-05 (a station unfreeze)** (note §23i–§23j). The chamber has walls
and a battery heater. In the reference the walls face the outdoor weather (the user's choice);
a held cabin, space and the station structure are lab options. Nothing outside the energy books
moved, the battery included: the heater never fires there, because the lamp's 133 W outweighs
the walls' largest 37 W loss. The station's heat store now follows the seasons (172.2–174.9 K).
Its lab test fires the heater (a dead lamp on the coldest days: 14.6 MJ, as predicted). Found on
the way: fast-side forcings keyed on the step count lead the plants by 90 minutes (a lamp window
on "day 113" goes dark in the last group of day 112); and in the lab's lamp shedding the heater
spends 3.9 MJ of the battery the shedding was protecting. An existing test that assumed a flat
chamber went red, unpredicted; it now runs with the walls off.

Saved sealed sessions from before 2b-iii also lack `boundary.chamber_surroundings`, so the same
`load_state` set-equality guard refuses them with the same error (read, not run).

**The heater shed with the lamp, BUILT 2026-10-05 (lab only)** (note §23k–§23m). The user's
call: the heater is an interruptible load, so the lab's lamp shedding switches it off too, on
the same battery reading. The blackout run is now the run with no heater at all, bit for bit,
and its battery is back at 1.8219e7 J; the chamber, unheated, ends 8.8 K below 22 °C. A lamp
that fails while the battery is healthy keeps its heater (3.57 MJ over three days). Six
predictions, six held; nothing frozen moved. Unbuilt and recorded: on a station whose battery
recovers, switching back on brings up to 200 W of re-warming with the lamp, and the rule has no
latch, so the restore could chatter.

**Slice 3a BUILT 2026-10-05 (a station unfreeze)** (note §24, graded in §24k). The plants read the
chamber, and the chamber runs the cited cold period: 4 °C for the first 8 weeks of each season,
22 °C after (Cha et al. 2022's standard protocol; `cold_period.yaml`). The lamp is unchanged —
dimming it for the cold weeks is 3b, a separate golden change by the user's choice. Eleven
predictions, made with an independent re-simulation before the code: ten held, most to the printed
digit; the eleventh (the warm-phase humidity ceiling) was mis-rounded in the prediction itself.
The crop now flowers about 102 days after sowing instead of 219 and is never short of water; the
heater fires once a season, at the warm-up. Found on the way: in the plain build a blackout now
reaches the crop through the room (an empty battery rations the lamp, so its heat stops); five
tests read the plants' temperature off their inputs, which the plan's search missed; and the
order matters — the plants must be wrapped to read the chamber after any flow is moved onto the
minute step, and a test now refuses the wrong order. Saved sessions keep loading (no stock added).
Unpredicted and measured after the commit (§24k): the soil's carbon roughly halved (humus 24.6 →
12.9 mol C). The soil itself did not change — its processes read no temperature — but its income
did: the crop, maturing 117 days earlier, hands the soil 63 mol C a season instead of 120, and the
soil's pools follow their income. Split by phase, the new crop grows 18 % faster per warm day
than the old, but almost nothing in its 8 cold weeks and for only 80 warm days before maturity
instead of 210 — so it makes about half the carbon. (A first reading, that production per
growing day was "about the same", counted the cold weeks as growing days and was corrected.)

**Slice 3b BUILT 2026-10-06 (a station unfreeze)** (note §24l predictions, graded in §24m). In
the 8 cold weeks the lamp is dimmed to 100 µmol m⁻² s⁻¹ for an 8-hour day (Cha et al. 2022: the
8 hours are the standard protocol's; the 100 is the paper's own vernalization light from a
different, faster protocol — the standard one says only "low light", and the paper's other two
institutes used 900 and 1300, so it is cited with that warning). The lamp keeps its efficiency
when dimmed (a PWM-dimmed LED, the user's design call): 40 W instead of 200. Built in two
stages: first the wiring with the cold-week light set equal to the normal light, which
reproduced every saved reference result byte for byte, then the switch. Every prediction held,
most to the printed digit: the battery ends 2.3 GJ fuller less the heater's 6.6 MJ; the crop
flowers 105 and matures 139 days after sowing (from 103 / 137). Found: the seedling LOSES half its
carbon in those 8 weeks (0.160 → 0.077 mol C), so it starts the warm weeks 5.5 times smaller, and
grain falls 36 % and the soil's carbon about 40 % (directions predicted, sizes not). Measured
afterwards, carbon booked flow by flow: the loss is mostly tissue SHED (0.073 mol C — leaves
0.034, roots 0.033, stems 0.007; first reported as "leaves", corrected the same day), not
breathing — growth and upkeep nearly cancel (−0.010). The model's shedding is a fixed daily
fraction per organ that reads no temperature, so at 4 °C tissue dies as fast as at 22 °C: a
candidate model gap, recorded; the user chose to study it next. No rationing. Two water stocks
came out bit-identical to 3a,
unpredicted and explained: the plants' water use does not read the crop's size, and the
warm-phase chamber forgets its past every minute, so the water ring converged to the same bits.
Three tests the roster missed went red, all by the seedling being smaller: a two-day test that
asked the lit crop to grow (now: lit above dark), and two separate-air tests at day 90 whose
crop is no longer hungry enough (moved to a cold period with the full lamp; the dim-world
figures recorded).

⚠ **Forward pointer (2026-10-06):** the cold-week seedling loss recorded under slice 3b above was
the biosphere's flat tissue shedding, not the dim light. Since the leaf-shedding unfreeze the
seedling ends the cold weeks at 0.146 mol C (`docs/log/leaf-shedding.md`).

**Slice 4, stage 1 — BUILT 2026-10-06 (a station unfreeze, plumbing only)** (plan §25). Re-sowing
when the crop matures means the chamber's cold weeks must count from each sowing, not from a
fixed calendar. Stage 1 moves that clock into the simulation's state — the step the standing crop
was sown on — while the re-sow itself stays on the calendar, so nothing can move but the record of
the clock: the sealed station's saved result gained exactly the one predicted line, every amount
bit-identical. The engine's core was not touched (both freeze rules forbid it): each of the five
lamp-and-chamber inputs the cold program drives is carried twice, a cold copy and a warm copy,
and a wrapper around every plant and station process picks one by the crop's own sowing. Because
the calendar and the new clock agree on every step of every saved run, matching them proves
nothing about the readers; a crop sown on day 10 does — its cold weeks moved 10 days on both
sides, in a test. Found by the wrapper's own refusal: the weather file's outdoor light, net
radiation and day length had been sitting under the plain names, overwritten until now; with the
two copies they would have been read silently by anything outside the wrapper. They are removed.
Every prediction held. Stage 2 — the re-sow fires on maturity — is next.

**Slice 4, stage 2 — BUILT 2026-10-06 (a station unfreeze): the crop is re-sown when it matures**
(plan §25f–§25g). The user's decision of 2026-10-01, built. Every crop now matures 139 days after it is
sown, and is re-sown the next morning: nine crops over the 1220-day run instead of four, and the
cold weeks run after every sowing — 504 cold days instead of 224. Eight predictions were written
before the code; seven held, most to the printed digit (the re-sow days exactly; the last crop's
thermal time 1169.8 against ≈ 1170). The big finding, predicted from the record and then booked
flow by flow: each crop now re-sows from 14.6 mol C of grain instead of 41 — because 64 % of
the old crop's grain formed AFTER it had matured. The model never tells a crop it is finished:
past maturity it keeps photosynthesizing and puts the new growth into grain. Re-sowing at
maturity removes that. The battery ends 2.9 GJ fuller (the dim lamp of the extra cold weeks).
The one failed prediction: the CO₂ scrubber and O₂ supply work slightly MORE (8.9 mol over
1220 days), not less — the scrubber moves with the carbon the station holds at the end, and the
end now holds a young crop instead of a big ripe one, partly offset by soil richer from eight
crops' residues. No rationing, no events; the 19 other saved results did not move. Two long tests (run
separately) had pinned numbers from the old calendar run and went red; both moved the predicted
way (more battery, a colder station) and were re-pinned, and one showed the chamber's cold hold a
few thousandths of a degree looser — the cold weeks now meet every season's weather.
