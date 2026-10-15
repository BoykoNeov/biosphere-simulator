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
