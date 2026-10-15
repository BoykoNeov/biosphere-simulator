# The heat the chamber's water carries — an estimate, lab-only

**Opened 2026-10-15** on the user's choice *"Estimate it first"*, from the air-resistance adoption's
open item (`post-roadmap-chamber-air-resistance-adopted.md` §7: *"the energy side of this change is
unmeasured, not zero"*). **Nothing here changes the model.** One probe, run on the tree before and after
the adoption, writes nothing and moves no golden.

## 0. The price, corrected before building

The item was offered to the user as "a small lab measurement that changes no results". Half of that
was wrong and was said so before building: the model gives evaporation **no heat at all** (no station
node and not `thermal.rs` carries latent heat, and the chamber dehumidifier draws no power), so
measuring the model's own energy books returns exactly zero. What can be measured is the size of the
missing term, from the model's own water flows, against the heat the chamber's books do carry.

## 1. What the shelf says about where the heat goes

BVAD (NASA/TP-2015-218570 Rev2, p. 145, "Internal Thermal Control System"): *"Cabin atmospheric
thermal loads are removed by the gas-liquid heat exchanger through two approaches. Sensible heat is
released from cabin atmospheric gases by convection to the gas-liquid heat exchanger. Latent heat is
released by condensing water vapor … removing both humidity and thermal energy"* — into the same
internal coolant loop (p. 146, Figure 4-5: "Condensing Heat Exchangers" → ITCS).

**Read for this model** (a reading of NASA's architecture, not something the model can test): when the
chamber's cooler is running, it and the condenser are one coil. Evaporation moves heat from the cooler's
column to the condenser's, and the station node receives the same total. So "water only" holds for the
energy books **on those steps**. It fails where the cooler is not running:

* **Heater-on steps.** The coil still condenses, so λ × the condensed water leaves the room, and the
  heater — on the battery — has to put it back. The model charges nothing.
* **Lamp-off steps.** The room is held at 22 °C and its dehumidifier keeps the air at or below 75 %, so
  the air stays dry at night and Penman–Monteith's air-dryness term keeps the crop transpiring. With no
  light, that water's heat comes from nowhere in the model.

**Electricity: none found.** BVAD prices thermal control as mass per kW of heat (Tables 3-5, 3-6), the
same for sensible and latent heat; its only coefficient of performance (Table 4-50) is a −22 °C food
freezer's, the wrong locus. And the model's cooler draws no power, so a figure would have nothing to
sit beside.

## 2. Scope

The **sealed station** only — the one room with a heat store, a cooler, walls and a heater (`chamber.rs`).
The sunlit sealed chambers (`sealed_chamber`, `perennial_*`, `consumer_*`) and the 7-day station builds
(`lighting`, `greenhouse`, `harvest`) have no heat books, so there is nothing to set a latent term against.

## 3. The instrument

`rust/crates/station/examples/chamber_latent_heat.rs`, through the lab two-rate runner (`TwoRate`, the
draw census's route). Every step, every flow is evaluated on the step's own start (as the census does),
and:

* **evaporation** = the transpiration flow's draw on `SOIL_WATER` (kg; the flow carries the crop's and the
  soil's together in a sealed build); the soil's share from the `soil_evap_today` aux's increment;
* **condensed** = every leg into `CONDENSATE` from the transpiration flow and the condensation flow;
* **chamber heat** = every energy leg on `thermal.chamber`, by flow (lamp, walls, heater, cooler);
* **latent power** = λ × evaporation / step, λ = 2.45e6 J/kg (`science.rs`), held across the plant step's
  minutes; a plant step is **dark** when its net radiation reads 0;
* **closure check**: a plant step fails when λE over its window exceeds the heat the room received in the
  same window (lamp + walls inward + heater).

**Controls** (the draw census's): `rationed == 0` on both sides; re-applying each step's legs gives the
`after` state bit for bit; the end state is byte-exact to the committed `sealed_station` golden of the tree
it runs on. The **50 run** is the same file in a worktree at `4f6d7dc` (the commit before the adoption,
whose golden is the 50 run's).

## 4. Predictions (written before any run)

| | prediction | confidence |
|---|---|---|
| P1 | mean latent power over the run: **45–70 W** at 7.5, **25–38 W** at 50 (the price's root-zone outflow, 3473.5 / 1860.6 kg over 4 yr, is ≤ 67 / 36 W if all of it evaporated) | medium |
| P2 | evaporation is **85–100 %** of the price's root-zone outflow (the rest drainage, which carries no heat) | low |
| P3 | the soil's share of evaporation is **5–25 %** | low |
| P4 | crop transpiration rate, dark steps over lit steps (warm phase): **0.7–0.95** at 7.5, **0.4–0.6** at 50 (Penman–Monteith at 22 °C, 75 %, lamp Rn ~84 W/m², rs 50: 161/181 and 55/101) | medium |
| P5 | lit steps: latent over lamp heat **0.4–0.9** at 7.5, **0.25–0.6** at 50 | medium |
| P6 | the closure check fails on **most dark steps with a grown canopy**, at both values; more failures at 7.5 than at 50; at 7.5 ≥ 50 % of all dark steps fail | medium |
| P7 | heater-on steps are mostly dark steps against cold outdoor weather; the latent during them is **not small** outside the cold period (≥ 30 W mean at 7.5) and small inside it (a seedling under a dim lamp, < 10 W) | low-medium |
| P8 | water books: evaporated − condensed − Δvapour = 0 to rounding | high |
| P9 | controls: `rationed == 0`, legs bit-exact, end state byte-exact, on both trees | high |

## 5. Measured (2026-10-15) — graded

Both trees ran the same probe file: main (7.5) and a worktree at `4f6d7dc` (50). Logs:
`W:\temp\claude\latent-heat\run75.txt`, `run50.txt` (kept at `sources\temp-evidence\latent-heat\`).
The run is **1220 days**, not the four years the predictions assumed (that slip is P1's miss below).

**P9 HELD, both trees:** `rationed` 0 on both sides; 0 steps where the re-applied legs miss the `after`
state; the end state byte-exact to the tree's own `sealed_station` golden; the transpiration flow read on
19 520 plant steps (1220 × 16). **P8 HELD:** evaporated − condensed − Δvapour = 3.0e-12 kg (7.5),
−3.2e-12 kg (50). The chamber's heat legs are **identical to the joule** at 50 and 7.5 — the model has no
coupling, as §0 said.

| | measured, 7.5 | measured, 50 | predicted | grade |
|---|---|---|---|---|
| P1 mean latent power | **80.7 W** (3473.5 kg) | **43.3 W** (1860.6 kg) | 45–70 / 25–38 W | **MISSED high, both** — the arithmetic divided by 1460 days; the run is 1220 |
| P2 evaporation / root-zone outflow | 100.00 % | 100.00 % | 85–100 % | held — `drainage` moved 0 kg; the price's outflow figures were evaporation exactly |
| P3 soil's share | 9.8 % | 16.8 % | 5–25 % | held |
| P4 crop rate dark / lit, warm | **0.925** | **0.660** | 0.7–0.95 / 0.4–0.6 | held / missed narrowly high |
| P5 lit latent / lamp heat | **1.058** | **0.603** | 0.4–0.9 / 0.25–0.6 | missed high / missed narrowly high |
| P6 window closure | 7465 of 11 456 warm windows (dark 1766 of 2864) | **0** | most dark windows, both values | **refuted at 50; and the check is confounded** (below) |
| P7 heater windows | 421 warm minutes in 1220 days, latent 7.1 W over them | 421, 6.8 W | not small outside the cold period | **refuted** |

**What confounds P6 and P7: the room's lamp heat is a daily average.** The fast side's `Lamp` reads the
lamp's averaged draw (133.33 W warm, 13.3–13.6 W cold) on every minute, lit or dark; only the plant side's
light and net radiation switch. So the room is never dark to its heat books, the heater almost never fires
(the averaged lamp outweighs the walls at night), and a per-window closure test compares the night's
transpiration with daytime heat. The meaningful closure is over a phase or the run.

### 5a. The estimate

**The warm phase** (the crop grown), latent power against lamp heat. ⚠ A "lit" window is any plant step
with net radiation above 0, so the two partly lit steps of each 16-hour day count as lit: three quarters of
the windows, not two thirds. That biases the lit rates slightly low and the dark/lit ratio slightly high;
no conclusion turns on it.

| | 7.5 | 50 |
|---|---|---|
| crop + soil latent, lit windows | 141.1 W | 80.4 W |
| crop latent, dark windows | 116.2 W | 43.6 W |
| **warm-phase mean latent** | **134.9 W** | **71.2 W** |
| lamp heat into the room (averaged) | 133.3 W | 133.3 W |
| **latent / lamp** | **1.01** | **0.53** |

**Over the run**, the room's books (identical on both trees): lamp 83.8 W in, heater 0.14 W in, walls
6.0 W out, cooler **77.9 W** out (93 % of the lamp). Latent: **80.7 W** at 7.5, 43.3 W at 50.

**What it says, on BVAD's one-coil reading (§1).** ⚠ *Corrected before reporting (advisor, 2026-10-15):* a
first draft read the run totals as "at 50 the energy totals held, at 7.5 they break". That nets windows short
of heat against windows with heat to spare — across the night and the day, and the cold phase against the
warm — and a held room cannot bank heat: its store is 1.5e5 J/K (0.053 K per minute of full lamp), and the
cooler removes any surplus at once. The run totals are kept below as the netted figure only.

1. **The model's night transpiration has no heat to pay for it, at either value.** The lamp is off at night
   in any real room (the model averages its heat over the day, item 4). The dark windows alone take
   **1.80 GJ at 7.5** (116.2 W × 2864 windows × 5400 s) and **0.67 GJ at 50** (43.6 W) — a lower bound on
   what a held room would have to find, since the walls lose heat at night too. The new value made it
   **2.7×** larger; it did not create it.
2. **Even with the averaged lamp**, windows short of heat add up to **1.66 GJ at 7.5** (7465 of 11 456 warm
   windows) and **0** at 50 — the probe's un-netted sum.
3. **The netted figure:** a coil condensing the run's water removes ≥ 8.50 GJ (3470 kg × λ) at 7.5 where the
   model's cooler removes 8.21 GJ — 0.29 GJ short; at 50 it is 4.55 GJ against 8.21 GJ, a surplus. This is
   the most favourable reading, and only it separates the two values.
4. **Why the crop transpires at night.** At 7.5 its dark rate is 92 % of its lit rate (66 % at 50).
   Penman–Monteith reads the room's held 75 % air at 22 °C, and the surface resistance (Szeicz–Long,
   `science::canopy_surface_resistance`) takes the leaf area and constants only — checked in the code
   (`flows.rs` `CanopyRead::surface_resistance`), nothing about light. So the stomata never close. ⚠ A lead,
   **not on the shelf**: the ASCE-EWRI (2005) hourly reference equation uses a larger night-time surface
   resistance than its daytime one, for this reason (general knowledge — unread, uncited here).
5. **The room's lamp heat being a daily average** hides all of this from the heater: the night that would
   need heating never exists in the room's books.

**Not measured, and why it matters:** whether the TM 102788 trial's water use (which 7.5 now matches,
1.04) includes night transpiration at this rate. The trial logged condensate daily (`post-roadmap-real-world-checks.md`
§12d: "lit vs dark water use — none"), so it cannot say.

### 5b. What this item does not decide

Three possible follow-ons, each its own decision, none started:

* **Night-time stomatal closure** — a larger dark-hours `rs`, from a source to be found (the ASCE lead).
  A biosphere unfreeze; it would move water in every sealed golden and likely the TM ratio.
  ⚠ *Corrected 2026-10-15* (`post-roadmap-night-stomata.md` §2a, §6): only the lamp-lit station builds
  ever see a dark step — the biosphere jars read the outdoor daily mean — so two goldens move, not every
  sealed one; the TM ratio did move (1.04 → 0.96–0.97).
  **ADOPTED 2026-10-15** at the booting ratio (0.123; `post-roadmap-night-stomata.md` §7): the
  dark-hours heat with nothing to pay for it, once the lamp is off at night, is now **~0.31 GJ, not
  1.80** (§6 there; the partly lit windows' dark minutes still count as day, ~0.5 GJ more, estimated).
* **The room's lamp heat switched with the light** (on/off rather than a daily average) — a station
  unfreeze; it moves the chamber's temperature and heater books, not water.
* **Latent heat in the room's books** (evaporation draws from the chamber store, condensation into the
  cooler's path) — the §0 build. Only worth taking after the two above, since today it would book a deficit
  that is night transpiration under an averaged lamp.
