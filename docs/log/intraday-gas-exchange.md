## **Plants and cabin trade gas within the day** (the 2026-09-29 review's first step, lab slices only — and the rationing count ROSE as the starvation eased)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**MEASURED 2026-09-29 on the user's call** (*"start implementing"* the 2026-09-29 review
proposal). Lab slices 1, 2, 3 and 5 of its first step; the adopt-or-not decision is the user's
and this item stops in front of it. Plan, predictions (committed before any code, `be9cd13`),
grading and the full tables: `docs/plans/post-roadmap-intraday-gas-exchange.md`.
**20 of 20 goldens identical; no manifest byte moved; the reference day is unedited.**

## What was built

* `station::driver::TwoRate` — a lab-only second day order beside the reference
  `advance_one_master_day`. It runs a master day slow-first (the reference) or interleaved
  (one plant quarter-day, then that quarter's 360 cabin minutes, four times), reports rationing
  **split by side**, and hands every step's before/after to an observer. Nothing in the
  reference, the session or the bridge calls it. An uneven split is refused.
* `rust/crates/station/tests/day_order.rs` — seven controls: slow-first is the reference bit
  for bit (greenhouse, harvest ring, a re-sow); no plants means no change; a scenario whose sides
  share no stock cannot tell the orders apart; one whose sides share the air can; the refusal.
  Five mutations, each caught by its intended test.
* `rust/crates/station/examples/intraday_exchange.rs` — the measurement. Every plant step is
  re-evaluated and handed to the backstop's own scale factors, so each firing is named; the
  probe's count must equal the driver's. The carbon books must close.

## Controls

* Slow-first equals the reference over the **full 4-year sealed station with its real re-sow**.
* Area scaling on the standalone sealed chamber is exact in Rust too: worst deviation
  **1.3e-14** over 3,660 steps at 187.45 m², aux identical. So the crew-loop record's sizing
  lever survives the port, the quarter-day step and the humidity setting.
* The carbon books close to 8e-11 mol; the crew's exhalation matches food × respired fraction.

## Findings

**1. ⚠⚠ The schedule starves the crop through CONCENTRATION, not rationing, so the rationing
count went UP as the starvation eased (predicted: down ≥ 90 %; measured 22 → 72).** At one crew
member's area (14.09 m²) under the reference order, the crop starts its afternoon quarter-days
at **1.89 and 1.47 mol** of CO₂ against the scrubber's 3.80: photosynthesis follows the
concentration down, demand falls with supply, and the backstop seldom fires. Interleaved, every
plant step starts at **exactly 3.7960** (the scrubber closes a quarter-day gap completely), the
crop asks for more, and the backstop fires more. A rationing count cannot measure this harm;
the crew-loop record already said `rationed == 0` cannot measure closure.

**2. At one crew member's area, interleaving removes the schedule's share of the starvation.**
Per m² the crop reaches **98.8 %** of the 1 m² crop's peak carbon (reference order: 57 %).
Closure share 0.64 % → 1.26 %.

**3. At the whole crew's area (187.45 m²) it helps and does not solve**: 18 % of the 1 m²
crop per m² (reference order 7.4 %), closure ×3.77 (predicted ×2–4). The predicted 11× draw
**failed** (7.26×) because its premise scaled an unstarved crop. What remains is the air's
size or the scrubber's rate, not the order.

**4. ⚠ The harvest ring moves far more than predicted** (grain store +37.5 %, humus +19 % in
7 days; predicted < 0.1 %). Isolated: the harvest flow carries the grain half, feces-to-litter
the soil half, independently. The two sides share **two more stocks than the air** —
my prediction reasoned about the air alone. Why each is this large is not isolated.

**5. The review's golden count was wrong on arrival**: `lighting` cannot move (lamp and crop
share no stock; measured bit-identical) and `sealed_energy_drift` never runs on this driver.
Adoption would move **three** goldens: `greenhouse` (≤ 4.7e-4), `sealed_station` (crop carbon
+1.2 %), `harvest` (finding 4).

**6.** The crew-loop record's Python-era numbers are not today's (187.45 m², reference order:
466 firings and LAI 0.439 today, 282 and 0.2772 then) — recorded side by side, not joined.

**Recommendation to the user: adopt** (the plan's §7) — a station unfreeze moving three goldens.
