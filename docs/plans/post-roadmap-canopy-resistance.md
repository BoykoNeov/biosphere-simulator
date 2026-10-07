# Post-roadmap: a canopy surface resistance that reads leaf area — lab measurement first

**Opened 2026-10-07 on the user's call** (*"Leaf-area resistance"*), chosen after Step 6's water-gap
split (`post-roadmap-real-world-checks.md` §10e), which found that the crop's frozen surface
resistance is a constant 70 s m⁻¹ whatever its leaf area — the record's standing gap *"transpiration
ignores leaf area"* (`post-roadmap-leaf-shedding.md` §12; `post-roadmap-room-temperature.md` §21).

**The order the user set:** find a source for the form, measure it in the lab first, then decide.
Adopting a form would move frozen values — an unfreeze with its own ceremony, not part of this
item. **Nothing here is frozen or adopted.**

---

## 1. Advisor review (2026-10-07), summarized

Re-read the FAO-56 text first-hand rather than through a summary. The two sources differ in TWO
places (below the plateau by a factor of 2, and at full cover), and plugging FAO's leaf value into
Teh's form is a composite of two sources. Teh's leaf resistance is the Jarvis light-dependent one,
which closes stomata in the dark; wheat coefficients are not on the shelf, so that part is left out
and the dark share of water use is reported. Teh's threshold LAI is his own 4.0, never derived from
the model's peak LAI. Lab-only, through the existing form-switch pattern, with the values in lab
code and not in `transpiration.yaml`. Read LAI through the crop's own leaf-area read. Find every
place the flow is built. Measure where the form bites (droughts, the sealed chambers), not on
unstressed runs where carbon cannot move. Enumerate the water readers before predicting which
goldens a later adoption would move. Name the open field's wetter early soil as a consequence, not
a win. Use TM 102788 for the curve's SHAPE, not its level. Instrument checks and one mutation.

## 2. The sources, read first-hand

**FAO-56** (Allen, Pereira, Raes & Smith 1998, *Crop evapotranspiration*, FAO Irrigation and
Drainage Paper 56), Chapter 2, read from the raw page `https://www.fao.org/4/x0490e/x0490e06.htm`
(downloaded 2026-10-07; FAO's site, free to read; facts cited, no text copied into code):

* Eq. 5, `rs = rl / LAI_active`, is introduced as *"An acceptable approximation to a much more
  complex relation of the surface resistance of **dense full cover vegetation**"*. The same paragraph:
  *"Where the vegetation does not completely cover the soil, the resistance factor should indeed
  include the effects of the evaporation from the soil surface."* **So FAO's form is scoped to full
  cover; it says nothing licensing its use on a seedling.**
* `rl` *"bulk stomatal resistance of the well-illuminated leaf"*; *"This resistance is crop specific
  and differs among crop varieties and crop management. It usually increases as the crop ages and
  begins to ripen. There is, however, a lack of consolidated information on changes in rl over time
  for the different crops."*
* Box 5 (the grass reference): *"LAI_active = 0.5 LAI which takes into consideration the fact that
  generally only the upper half of dense clipped grass is actively contributing"*; *"The stomatal
  resistance, rl, of a single leaf has a value of about 100 s m⁻¹ under well-watered conditions"*;
  `LAI = 24 h` for clipped grass at h = 0.12 m → LAI 2.88 → `rs` = 70 s m⁻¹. **The 0.5 and the 100
  are stated in the grass box.** (The frozen `rs` = 70 is uncited — `TODO(cite)`, "literature-typical
  ~50–100" — and only equals FAO's grass value.)

**Teh** (*Introduction to Mathematical Modeling of Crop Growth*, on the shelf), §4.6, book p. 97–98,
read off the page image: Eq. 4.80 after Szeicz & Long (1969) — canopy resistance
`rc = rst / L` for `L ≤ 0.5 Lcr`, and `rc = rst / (0.5 Lcr)` for `L > 0.5 Lcr`, *"where Lcr is the
threshold leaf area index, often taken as the maximum L the plant can achieve. In this book, we will
take Lcr as 4.0, a typical maximum leaf area index."* `rst` is the single-leaf stomatal resistance of
Eq. 4.79 (Jarvis 1976), `rst = (a1 + I_PAR)/(a2 · I_PAR)`, with coefficients for sunflower and maize
only — none for wheat.

## 3. The two lab forms

| | FAO-56 Eq. 5 + Box 5 | Teh Eq. 4.80 (Szeicz & Long) |
|---|---|---|
| form | `rs = 100 / (0.5 · LAI)` | `rs = 100 / min(LAI, 0.5 · 4.0)` |
| at LAI 0.03 (seedling) | 6 667 s m⁻¹ — outside FAO's stated scope | 3 333 |
| at LAI 1.0 (a sealed chamber's peak) | 200 | 100 |
| at LAI 2.86 / 1.43 | 70 (= frozen) | 70 at LAI 1.43 |
| at LAI 6.2 (full cover) | 32 | 50 (the plateau from LAI 2.0) |
| its leaf value | FAO's 100 (the grass box) | FAO's 100 — **a composite**: Teh's own `rst` is the light-dependent Jarvis form, which is left out (no wheat coefficients) |

Both forms are lab-only. The frozen constant form stays the loader's value. The values (100, 0.5,
4.0) live in lab code, not in `transpiration.yaml` — a param-file edit would force a manifest
regeneration. At LAI 0 both give an infinite resistance and so zero transpiration (the
Penman–Monteith denominator diverges; the flux is exactly 0, finite).

**Where it is built:** one production site, `domains::biosphere::system` (`Transpiration { … }` in
`build_season_with`); the station's air split and minute-step paths move that same flow by id, so a
form carried in the flow travels with it. LAI is read the way the crop reads it (derived from leaf
carbon, or the lab leaf form's state).

## 4. Measured before predicting (2026-10-07)

A throwaway probe (`W:\temp\claude\step6\zz_probe_lai.rs`, run once, not committed) read LAI over
one season under the frozen build:

| scenario | LAI day 0 / 30 / 90 / 150 / 200 / 250 / 300 | peak | share of steps with LAI < 2.86 (FAO `rs` > 70) / < 1.43 (Teh `rs` > 70) / < 0.5 |
|---|---|---|---|
| default open field | 0.03 / 0.20 / 0.40 / 0.67 / 5.33 / 5.99 / 3.40 | 6.18 | 0.61 / 0.57 / 0.41 |
| deep water (1 mm/day) | same to day 200; 5.75 / 1.92 | 6.18 | 0.68 / 0.57 / 0.41 |
| sealed chamber | 0.03 / 0.33 / 0.60 / 0.89 / 1.00 / 1.02 / 0.35 | 1.02 | 1.00 / 1.00 / 0.26 |
| perennial chamber | … / 0.87 at day 250 | 0.88 | 1.00 / 1.00 / 0.46 |
| consumer chamber | … / 0.69 at day 250 | 0.77 | 1.00 / 1.00 / 0.63 |

(The probe's potato line used wheat's leaf constant and is discarded.) **The sealed chambers never
reach LAI 1.43, so under either form their resistance exceeds 70 on every step.**

## 5. Predictions (before the lab form exists)

| | prediction | confidence |
|---|---|---|
| P1 | the frozen constant form, run through the new switch, is bit-identical: `regen_goldens` 20 of 20 identical | high |
| P2 | each form reaches the flow: a run's transpiration equals Penman–Monteith at that form's `rs` on the same state, to ~1e-9 (the R1 pattern) | high |
| P3 | **sealed chambers:** season transpiration falls under both — FAO by 45–75 %, Teh by 20–50 % (their `rs` sits at ≥ 200 / ≥ 100 against 70) | medium |
| P4 | **open-field default:** winter transpiration (LAI < 0.5, 41 % of steps) falls by > 80 % under both; summer (LAI 5–6) rises — FAO ~+15–30 %, Teh ~+5–15 %; the season total falls by 0–25 %, and drainage rises | low |
| P5 | carbon in the default and the sealed chambers stays bit-identical — they never reach water stress, so only water moves | medium (needs each run's lowest FTSW ≥ 0.30 asserted) |
| P6 | **deep-water rescue** (pinned: canopy 1.0294×, grain at the season's end 1.2199×): the control's invented seedling drought shrinks, so the grain ratio falls toward 1 (1.00–1.18) under both; the canopy ratio stays 1.00–1.06 | low |
| P7 | **`drought_window.rs`**: at LAI ~5.5 the full-cover crop drinks more, so stress (FTSW < 0.30) starts earlier than day 254 — FAO by 3–10 days, Teh by 1–6 | low-medium |
| P8 | **TM 102788 row:** W1 rises from 2.376 to ~2.8–2.9 (FAO) and ~2.5–2.6 (Teh); neither reaches the trial's ~6. The early water use falls toward the trial's low pre-cover values | medium |
| P9 | the dark steps carry ~8 % of the row's daily water use under the frozen form — the share a light-dependent `rst` would act on, left out here | medium |
| P10 | a mutation forcing each form back to a constant 70 turns the "reads LAI" assertion red | high |

**What would refute the motive.** If P5 fails — carbon moves in an unstressed run — the form has a
coupling not understood here, and that is the finding.

## 6. Decisions owed to the user, after the measurement

1. Which form, if any, to price for adoption (FAO's scope is full cover; Teh's is the only one that
   addresses a seedling, with a composite leaf value).
2. A constant leaf resistance, or the light-dependent Jarvis form (a WHAT-IF without wheat
   coefficients).
3. Teh's threshold LAI (his 4.0) or a cited crop maximum.
4. Whether the open field's bare-soil evaporation, which today's full-cover rate stands in for, is
   taken up (Teh §4.7 carries a soil form) — a separate item.
