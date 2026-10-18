## **Authoring can reach plants** (Step 7 of the 2026-09-29 review — nothing ran the authored scenarios, and the authoring blocks hold no plant flow)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-18** on the user's *"1 and 2"* — Steps 4 and 7 of the review proposal written
2026-09-29. This is Step 7. Plan and record: `docs/plans/post-roadmap-authored-plants.md`.

**7a BUILT.** `rust/crates/authoring/tests/authored_content.rs` enumerates `scenarios/*.yaml` on disk
and holds each to the platform's promise for an authored file: it builds, runs its full horizon under
`run_scenario` (conservation every step, rationing and reversal are failures), and two runs end
bit-identical. Not a golden; nothing is pinned. ~7 s for both files' full years.

**Its first run found a broken habitat, not a stale sentence.** `bioregenerative_station.yaml` was
written against the design `o2_setpoint` of 10.0 mol with an 8-mol cabin. The 2026-09-06 citation moved
the setpoint to 1995.0 and nothing ran the file: for six weeks the regulator drained the 500-mol tank into
the cabin every hour — **122,640 backstop firings a year**, every step conserving. Fixed by sizing, not
tuning: the regulator's offset (0.23725 mol) is set by the crew's draw and the rate constant, not the
setpoint, so the cabin now starts at 1993.0 and every other book runs as written. Re-measured on the fixed
file: rationed 0, O₂ 1994.76275 from hour 2, tank 500 → 148.198 (the file's stated 148.2), biomass and
feed at their stated 5× and 4.5×, the thermal node at its stated Q_eq. `algae_habitat.yaml` was sound and
ends at its stated fixed point. A second false sentence came out: "every ECLSS value is a DESIGN
placeholder", falsified by two citations.

**7b DESIGNED, not built** — three questions for the user (the plants' counters and re-sow, which of a
bed's stocks an author may connect, how the two step sizes are declared), and the parked O₂-setpoint
authoring decision, which is what broke the habitat above.

* **Lesson: the unvalidated half breaks silently.** A frozen value move is a ceremony on the reference
  side and a free fall on the authored side. A test that only RUNS the content — no values — caught a
  six-week-old failure on its first run.
* **Lesson: run before you read.** The plan expected one false sentence; reading would have fixed the
  sentence and left a habitat that empties its oxygen tank 14 times an hour.

**SLICE 1 of 7b BUILT 2026-10-18 — the O₂ target is stored as a mole fraction.** `eclss.yaml`'s
`o2_setpoint: 1995.0 mol` became `o2_fraction: 0.21`; the target in mol is built per cabin
(`EclssParams::o2_setpoint_mol`), and the regulator's constructor takes the cabin's air with no
default. The station passes its 9500 mol; the authoring side a named, temporary 9500 until slice 2.
Predicted and measured bit-neutral: 0 of 20 goldens move, the station manifest's `eclss.yaml` entry
is the whole manifest diff. Found: `humidity_setpoint` hides the same 9500 mol (recorded, not changed —
outside the user's answer); two fixtures still start at the long-gone 10.0.

**SLICE 2 BUILT 2026-10-18 — an authored O₂ regulator declares its cabin's air.** `cabin_air_mol`,
required on `eclss.o2_makeup`, refused elsewhere, no default; the direction gate reads the same
per-flow setpoint. The temporary 9500 is deleted. An authoring unfreeze: the manifest gained the field
and each flow type's `needs_cabin_air`, as predicted. Declaring it exposed two more fixtures the
2026-09-06 move had broken in meaning (`eclss_multirate_cabin`, `eclss_thermal_habitat`, designed
around a 10-mol target): they declare 47.61904761904762 mol, for which 0.21 × air is exactly 10.0.

**SLICE 3 BUILT 2026-10-18 — the lamp-lit chamber, extracted.** `station::plant_chamber` holds the sealed
station's plant chamber as parts with outward `ChamberPorts` (power, heat sink, space); the sealed station
is rebuilt on them. Proof: 0 of 20 goldens, all three manifests unchanged, full suite and clippy green.
The station's hand-built starting values were measured equal to the builder's on the reference before the
copy was deleted; under the lab leaf form the copy lacked the seedling's leaf area (a lab-only gap, closed).

**SLICE 4 BUILT 2026-10-18 — a scenario may carry a crop bed.** `bed:` builds the sealed station's
lamp-lit chamber (its own builder) into an authored habitat and runs it on the station's day driver.
Every bed stock is wireable by the author's flows; three ports connect the chamber to the author's
battery, heat store and (walls facing space) radiative sink; the walls' surroundings and the re-sow
rule (`maturity`, `never`, `{every_days: N}`) are required. An authoring unfreeze (`bed_crops`,
`bed_stocks`, three spec models); the station's fast forcings and re-sow hook moved onto
`plant_chamber`, 0 of 20 goldens. Pinned: the bed's starting state equals the sealed station's, stock
for stock. Found: a re-sow needs grain from the old crop, so a calendar can be refused by the science.
A layering edge `authoring → station` was added to the purity gate, with its reason, and flagged.

**SLICE 5 BUILT 2026-10-18 — STEP 7 COMPLETE.** `scenarios/lamp_greenhouse.yaml`, the first authored
habitat with plants: a crew of one breathing a lamp-lit wheat bed's own air, with a scrubber, an O₂
regulator on the bed's 9500 mol, solar power and a radiator. 14 days clean (CO₂ and O₂ at the hand
arithmetic's values); 300 days clean with a maturity re-sow (`--ignored`).

* **Lesson: an assumption made explicit finds its own casualties.** Requiring the cabin's air exposed two
  more files the 2026-09-06 setpoint move had broken in meaning; running the content found the third.
* **Lesson: a layering gate is a question, not a wall.** The bed needed `authoring → station`; the gate
  called any new edge a decision, so it was taken in the open, with its reason in the rule.

**TWO LEFTOVERS FIXED 2026-10-18** (plan §12–§14, on the user's *"work on the leftovers"*). The lamp's
window mean is now the lamp exactly when the window is wholly lit and capped at it when partly lit: at
a step that is not a power of two of a day it had read a rounding step off, which the transpiration's
lit-share guard refused at 24. An arithmetic probe first showed the time itself does not slip (right
day on every step, 100 years, 12 to 1440 steps a day). Bit-identical at a power of two (0 of 20
goldens); the bed's power-of-two refusal lifted. And the bed's direction gate now judges every state
the day commits, through an observer on the station's day driver, where it judged each day's end:
pinned by a regulator that runs backwards at midday and is back under its target by midnight. The
other three (another crop, another air volume, the humidity setpoint as a share) are priced and wait
for the user.

* **Lesson: probe the arithmetic before choosing between "clamp" and "constraint".** The advisor's
  case for keeping the power-of-two rule — time slipping across midnight — was a real risk and a
  wrong prediction; seconds of arithmetic told the two apart, and the cap then fixed a case the guard
  never saw (a window reading *under* the lamp).

**THE HUMIDITY TARGET AS A SHARE, SLICE A BUILT 2026-10-18** (plan §15, on the user's *"1"* — the third
leftover, first because it unblocks a bed air other than 9500 mol). `eclss.yaml`'s `humidity_setpoint:
1.7863 kg` became `humidity_fraction: 0.40` and `saturation_vapour_pressure: 2643.931192210576 Pa` (the
model's curve at BVAD's 22 °C crew-cabin nominal); `Condenser::new` takes the cabin's air and builds
its target there, as the O₂ regulator does. Predicted before running: six named goldens, `cabin_h2o`
up by a constant, nothing else. Measured: exactly six, one number each, +1.542826e-05 kg, every other
leaf bit-identical. Station manifest: `eclss.yaml`'s fingerprints and six golden hashes. Authoring
keeps a named temporary 9500 until slice B, which waits on a user question (two test files declare a
47.6-mol cabin for their O₂ regulator).

* **Lesson: store the input in the form that keeps the tier claim true.** Storing 22 °C would have put
  an `exp` into every condenser build and silently falsified three goldens' "last-bit equal on any
  platform" (the check that would catch it was deleted with the Linux job). Stored as the pressure at
  22 °C, the target is multiply and divide; the temperature was a constant either way. Check
  `rust/data/tiers.json` before adding a transcendental anywhere a Tier-1 golden builds.

**SLICE B BUILT 2026-10-18** (plan §15b–§15c). Asked first, measured before asking: the two test files
that declared a 47.6-mol cabin for their O₂ regulator would keep a working condenser on that air, but
their air would end 1.7× and 2.2× past saturation. The user chose one realistic 9500-mol cabin for both
regulators. `eclss.condenser` now requires `cabin_air_mol`, no default; the temporary 9500 is deleted.
Authoring manifest: `needs_cabin_air` on the condenser, nothing else; 0 of 20 goldens.

* **Lesson: measure a question's premise before putting it to the user.** The first framing ("a small
  air overturns how the files were designed") was wrong — the condenser only acts on the excess, so its
  behaviour survived; the real cost was supersaturated air nothing checks, and that is what the user
  decided on.

**A BED AIR OTHER THAN 9500 MOL BUILT 2026-10-18** (plan §16, on the user's *"1"* — §14's second
leftover, unblocked by §15). `bed.air_mol`, optional, the reference's 9500 when absent. The room is
resized, not relabelled: its starting CO₂ and O₂ scale with the air, by the separate-air lab's own
resize moved onto the scenario (`SealedStationScenario::with_chamber_air`) so both call one function.
The bed's regulator check now covers the condenser on its vapour as well as the O₂ regulator on its
O₂ — the condenser had had the same hole since §15b. Predicted and measured: the authoring manifest
moves by `BedSpec += "air_mol"` and nothing else, 0 of 20 goldens, the default bed bit-identical to
the station's chamber. Probe: every size from BVAD's 27.66-mol box to 30000 mol runs a crop-only
30 days clean; the small rooms' CO₂ climbs (17 072 ppm at 27.66 mol), the soil out-breathing a
crop under the cold program's dim lamp.

* **Lesson: when a check is closed for one regulator, look for its twins.** The O₂ check was written
  when only the O₂ target was a share of the air; §15b made the condenser's a share too, and nothing
  re-asked which flows the bed's check should cover. A flow-type property (`needs_cabin_air`) had
  grown a second member, and a rule keyed on one type name did not follow it.
