## **Checks against the real world** (the 2026-09-29 review's Step 6 — a drought with an end, built lab-only; the comparison data searched, and the one trial opened held its CO₂)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-06** on the user's call (*"step 6"*), chosen from the open list after Step 3c
slice 4 closed. Plan, predictions and grades: `docs/plans/post-roadmap-real-world-checks.md`.
Nothing frozen moved.

**Slice 1 — a drought with an end, BUILT lab-only** (`rust/crates/domains/tests/drought_window.rs`).

* Measured first: the step's premise ("no scenario produces drought") is true of the frozen roster
  — the default season never drops below 99 % of its root-zone water — and false of the suite. The
  test-only deep-water run (1 mm/day) stresses growth, water use and development for ~80 days.
  What was missing: no drought had an END, and the lab leaf form's drought factor had never fired.
* Watering cut to 0 over days 220–260, then back to 8 mm/day. Nine predictions written first; all
  held, several at their edges. Stress from day 254 (34 days after the cut) to day 266; lowest
  root-zone fraction 0.2093; every factor exactly 1 again by day 266; development factor peak
  1.12; grain at maturity 0.963 of the control's; flowering and maturity unmoved.
* **Finding: the deep root zone holds about a month of a full crop's water**, so the cut's stress
  fell at and after flowering, not in the month before it as placed.
* Two mutations, both red as designed (the cut made a no-op: 4 of 6 red; no recovery: the recovery
  test red).
* Stated limits: transpiration ignores leaf area, so the drought is the model's; the lab leaf form's
  factor fired on only one pre-flowering day, which shows the input reaches it, not an effect.

**Slice 2 — the comparison data, SEARCHED; the choice is the user's.**

* **Opened: NASA TM 102788** (Wheeler & Sager 1990, the Biomass Production Chamber, 1989): 112.6 m³,
  20 m², wheat cv. Yecora Rojo (spring habit — general knowledge, NOT on the TM's pages) in
  nutrient solution, 20 h light at 534 µmol m⁻² s⁻¹, 20 / 16 °C,
  **CO₂ held at 1000 ppm**. Measured: night respiration mean 7.2 µmol m⁻² s⁻¹; day uptake peak 27 at
  day 25, mean 15; 19.5 mol CO₂ fixed per day net; ~40 kg dry biomass; transpiration peak 120, mean
  90 L/day (4.5 L m⁻² d⁻¹). Public domain; on the (git-ignored) shelf.
* **Context added to the record** (`log/co2-uptake-source.md`, annotated in place): its Figs. 2–3
  are, as that record said, one deliberate drawdown on day 25. What it did not say: the rest of the
  trial held CO₂ at 1000 ppm, and the "sealed" chamber leaked 5–10 % of its air a day.
* Named, unopened: Bugbee & Salisbury 1988 (light range, CO₂ held at 1200 ppm), the BPC programme
  papers (Wheeler 1996, 2008 — potato too), LMLSTP Phase I (1995, one crew member + 11.2 m² wheat),
  Gerbaud et al. 1988, Lunar Palace 1.
* **The trials with a stated CO₂ control all held it** — TM 102788 (read), Bugbee & Salisbury
  (abstract), LMLSTP Phase I (held by changing the LIGHT, a controller of another kind). Gerbaud
  and Lunar Palace: unknown. A held trial should map onto a fixed leaf-CO₂ forcing (`CI_VAR`),
  which the open-field build reads — but that build reads the outdoor weather, and the lamp-lit
  builds (`lighting_scenario`, `day_neutral_lighting_scenario`: lamp, set photoperiod, constant
  temperature) are all SEALED. No existing build hosts TM 102788 yet; slice 3 confirms the mapping.
  If it holds, the parked chamber controller is not forced for the held-CO₂ rows; for LMLSTP it is
  not established. The sealed jar has no counterpart found.
* The wheat trials read grew Yecora Rojo under long days; a scorecard row would run the crop with
  vernalization off and read rates and amounts, not dates.
* The water row is labelled before any run: early water use is the known leaf-area gap.
