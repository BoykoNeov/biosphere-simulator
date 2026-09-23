# Post-roadmap — the chamber's water vapour obeys saturation

**Opened 2026-09-23**, on the user's *"go with your recommendation"* (the recommendation: the
first of the three successors `log/atmosphere.md` named, the biggest known physical defect in
the tree). Predecessor: `docs/log/atmosphere.md`, finding 2.

## 1. The defect

In a sealed chamber, transpiration moves water `soil_water → water_vapor` at a rate set by the
**weather's** VPD, and `Condensation` removes it first-order at `0.5/day`. Nothing asks whether
the air can hold it. The steady state is `v = T/k`, so the vapour load is set by the plot alone:
all three frozen chambers peak at the same 536.995 mol in rooms of 1000 and 2000 mol — wet
pressure 1.537 against a saturation-implied ≈1.023.

## 2. Measured before designing (probe under `W:\temp\claude\vapour_probe`)

| chamber | room (mol) | saturation range (mol) | max one-step transpiration (mol) | steps above saturation |
|---|---|---|---|---|
| sealed | 1000 | 5.28 … 26.41 | 80.13 | 3660 / 3661 |
| perennial | 1000 | 5.28 … 26.41 | 80.13 | 6100 / 6101 |
| consumer | 2000 | 10.57 … 52.83 | 80.13 | 6099 / 6101 |

**One ¼-day step transpires up to 4.85× the whole room's saturation capacity.** That settles the
design:

* A fix inside `Condensation` alone cannot bound the vapour — it sees only start-of-step vapour
  and overshoots by one step's transpiration (the jar would still peak near 1.08).
* Raising `condensation_rate` to keep up needs `k ≈ 10–16/day`, `k·dt ≈ 3–4`: Euler-infeasible,
  the backstop would ration.
* **The bound has to be at the source.**

## 3. Design

**Saturation ceiling (no new number).** `n_sat(T) = e_s(T) / P_std · n_ref`, where `e_s` is the
already-cited FAO-56 Tetens curve (`weather::saturation_vapor_pressure`), `n_ref` the chamber's
existing `chamber_air_capacity_mol`, and `P_std = 101 325 Pa` the standard atmosphere — a
**definition** (10th CGPM, 1954, Resolution 4), not a calibration. Relative-humidity ceiling
**1** — plain saturation; no humidity setpoint is invented.

⚠ Deliberately dropped: `n_ref` is the room's fill at reference *pressure* and the model has no
chamber temperature state, so the conversion ignores the `T/T_ref` factor (≈7 % over the
weather's range). The model's V and T are constants everywhere else; this matches them.

**Transpiration (sealed only) splits its sink.** Flux `F` is unchanged. The part the air can hold,
`clamp(cap − v, 0, F)`, goes to `water_vapor`; the rest goes to `condensate` in the same step —
dew on the condenser and walls. Three legs, balanced; the vapour leg is never negative.
The open field keeps its two legs to `boundary.vapor`, byte-for-byte.

**Condensation gains an above-saturation term.** `max(0, v − cap) + k·dt·min(v, cap)` —
the whole excess (a temperature drop lowering the cap), plus the existing engineered condenser on
the saturated remainder. Withdraws at most `v` while `k·dt < 1`.

**Invariant:** at the end of every step, vapour ≤ saturation *at the temperature that step ran
at*. (A day boundary can lower the next cap; the next step's condensation removes the excess.)

**Not built — named successor:** the plants still read the **weather's** VPD, not the chamber's
own humidity. Coupling it cannot bound the vapour by itself (Penman–Monteith's radiation term
transpires at VPD 0, and the weather VPD floors at 0), so it does not block this; it is the same
"reads something other than the chamber's air" shape the atmosphere work found.

## 4. Prediction — written before any code

**Golden files (21 on disk):**

* **Change: the 9 sealed goldens** — `sealed_chamber`, `perennial_chamber`,
  `perennial_long_horizon`, `consumer_chamber`, `consumer_long_horizon` (biosphere);
  `greenhouse`, `lighting`, `harvest`, `sealed_station` (station). Water stocks move:
  `water_vapor` falls to at most saturation (≤ ~0.5 kg in a 1000-mol room), `condensate` and
  the soil stores absorb the difference, and the irrigation/drainage boundaries move with them.
* **Byte-identical: the other 12**, including `season_euler_state.json` (open field, which holds
  `boundary.vapor`), `cabin_gas`/`eclss` (the station's own humidity, not the biosphere's), and
  both drift summaries.
* **Carbon / O₂ / N / consumer stocks: predicted UNCHANGED in the three biosphere chambers,** on a
  measurement: the water-stress factor is exactly 1 at every step of all three runs under both
  `wssg` and `wssd`, with soil water 18.7 … 168 kg. ⚠ **One reader is unmeasured** — the root
  extension's dry-subsoil stop reads `subsoil_water`, which drainage feeds. If it binds anywhere,
  the biology moves through rooted depth → N uptake. The four **station** chambers are
  unmeasured on every reader. If biology moves, that is a finding to explain, not to absorb.

**Tests:** the tripwire goes red on **both** assertions (peak above 1.023; room-independence),
and is deleted rather than re-thresholded. The replacement guard fails with the split removed.

## 5. Outcome — BUILT 2026-09-23; the golden predictions held, TWO others did not

Record: `docs/log/vapour-saturation.md`.

* **Goldens:** 9 changed, 11 identical — file for file as predicted (20 are regenerated; the
  21st, `state_snapshot.json`, is not a reference output — §4's "other 12" miscounted it).
* **Only water moved:** `water_vapor`, `condensate`, `soil_water`, and in six files
  `subsoil_water`. No carbon/O₂/N/consumer value in any of the 9 — the two unmeasured branches
  (dry-subsoil root stop; the four station chambers) did not bind.
* **Bound:** vapour ≤ saturation at the step's temperature on every step of all three chambers;
  `rationed == 0`; wet pressure peak 1.537 → 1.0235; the 2000-mol chamber's peak vapour is
  exactly twice the jar's.
* **Manifests:** 9 golden hashes + `water_cycle.yaml` (header rewritten), nothing else.
* ⚠ **MISSED — "the irrigation/drainage boundaries move with them".** No boundary stock moved in
  any of the 9 files; the water was redistributed entirely among the four in-chamber stores.
  Not investigated — the prediction over-reached, and the golden diff is what says so.
* ⚠ **MISSED — "the tripwire goes red on BOTH assertions".** Its `peak > 1.023` half stayed green
  on the fixed model (a 20 °C ceiling; the weather is warmer); only its room-scaling half fired.
  Deleted and replaced as its text instructed.
