## Bucket 2 (cont.): the export-fidelity hazard — **the one rationing cannot see**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE — doc, pins, AND the fix.** ⚠ This row read *"the fix is PLANNED, not started"*
for four commits after it shipped: the closer is the **build-time `k·h < 1` precondition**
landed by multi-rate **Step 5** (the row below), which the multi-rate phase *absorbed*
rather than merely enabling — multi-rate is the performance enabler, the precondition is the
hazard closer, and this row was written before either existed. **The meta-finding's sixth
instance, and the first one in `CLAUDE.md` itself**: the status table is exactly the kind of
prose no gate watches (cf. "the freeze's prose half is ungated"), so a row can stay false
indefinitely with a green suite. On finishing a step, re-read the **status row** for the
bucket it closed, not only the reference doc. Scope premise was false *again* (cf. scope C):
the "open" o2_makeup venting hazard was **already documented in 3 places and the prose was
right** — just never measured. Measuring it found a worse, invisible one. `eclss.o2_makeup`
is the registry's only flow with a **SIGNED demand error**: its draw ∝ the *setpoint error*,
not the stock, so near the setpoint it **never over-draws and the backstop never fires**. ⚠
**This row said "the only demand-controlled flow" flat until 2026-07-27, when the N-cycle
form change made `biosphere.nitrogen_uptake` demand-controlled too** (draw ∝ `target·biomass
− plant_n`, not ∝ `plant_n`) — a cross-bucket claim the N-cycle work had to sweep, having
just applied the same lesson to `mineralization.py`'s "~1000×" sentence. **The invariant
that survives is SIGNEDNESS, and it is what the hazard actually needs**: `o2_makeup`'s error
changes sign through the setpoint, so it oscillates; the N deficit **clamps at 0** (`max(0,
…)`), so there is no restoring force to overshoot against and `k·h = 1` at the frozen `dt=1`
is *deadbeat*, not marginal (pinned:
`test_demand_limited_uptake_is_dt_linear_and_deadbeat_at_dt_1`). So the export-fidelity
finding is intact — but "demand-controlled ⇒ invisible to the backstop" is now the wrong
predicate to quote; "signed error" is the right one. Every other row is donor-controlled
(draw ∝ stock), over-draws at `k·dt>1`, and **is** caught. So: `dt=900` exports `12 → 8.4 →
11.28 → 8.976` around a 10.0 setpoint with **`rationed = 0`**, conserving, endpoint
*correct*; `dt=1000` swings **12↔4 forever** while `o2_supply` drains past −800 mol,
reported clean. The full cabin is protected **only by coincidence** (`k_scrub·dt=1` and
`k_makeup·dt=2` both land at `dt=1000`). **`k·dt < 1` STAYS OPERATIVE** — an advisor catch
killed a draft that relaxed it to the textbook stability bound `< 2`: `< 2` answers "does
the solver diverge", `< 1` answers "is the export usable by a neighbour"; we couple domains,
so `< 1` governs. **No clamp** (symmetry IS the restoring force). Not a bug in the flow —
`dx/dt = k(S−x)` cannot oscillate; this is explicit Euler failing at too large a step, a
**solver** property (pinned: first-order convergence + the deadbeat prediction hit exactly).
Nothing unfrozen, no golden moved. `tests/test_authoring_export_fidelity.py` (12 pins)
