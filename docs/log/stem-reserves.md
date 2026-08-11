## **Stem-reserve remobilization** (the user's own question: does the stem feed the seed?)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DIAGNOSED AND PRICED 2026-08-10, NOT BUILT — read-only.**
`docs/plans/post-roadmap-stem-reserves.md`; probes `M:/temp/stem_reserves/`; **18 pins** in
`tests/test_stem_reserves.py` (2 slow). Teeth by **mutation**: Table 7's wheat row 0.40 →
barley's 0.30 takes **12 of 18** red. **THE STARTING FACT**: our stem gains **62 % AFTER
flowering** and is still gaining on the last day, and there is **no path at all** from stem
carbon to grain. ⚠ Three counts of that structure and only one is the quantity — **5** flows
*reference* `stem_c`, **3** ever emit a leg on it, and at any single mid-season step only
**2** do, because the maintenance draw is a **CONDITIONAL** door (deficit days only). My
first count was wrong and the pin measures all three over the whole trajectory. Against the
committed oracle fixture (HI ≈ 0.564, quoted in `allocation.py`'s own docstring) the frozen
crop's grain **fraction** is 0.84× and its grain **mass** 0.52× — *different quantities,
stated apart.* **THE SCIENCE WAS ON THE SHELF, in the book `allocation.py` ALREADY CITES**:
[A] §2.2.2 Table 7 (p. 46) gives **wheat 0.40** as the remobilizable fraction of stem weight
at flowering (**CABO unpublished**, and the caption says so; the same page says "there is
little data published"), and the "simple view" drains it at **0.1 d⁻¹**. ⚠ **Uncited ≠
self-disclaimed**: the book's *"chosen without an experimental basis"* attaches to the
**other** (L1Q) hypothesis, which carries the same numeral. **⚠⚠ THE FINDING, AND AN ADVISOR
CATCH CORRECTED ITS FRAMING BEFORE IT SHIPPED: the SOURCED form fixes the stem and
overshoots the grain, and the form that lands on the grain is ONE WE CONSTRUCTED.** §3.2.4's
growth fraction is what [A] **programs** (Listing 3 Lines 17, 35): stem shape **1.618× →
0.985×** (it stops gaining), harvest index **1.151×** the oracle. A one-shot fill at
flowering — read off Table 7's *caption* — lands on the harvest index (1.000×) and leaves
the stem still growing **+35 %**. My draft called these "two readings of the same source ...
both are [A]". **They are not**: every formation pointer in the book is one of **two
programmed models** (Listing 3's growth fraction; Listing 4's **sink-limited overflow**),
and Table 7 is cited *into the first* as the source of its parameter. **A data table is not
a model form** — established by extracting every `Listing 3`/`Listing 4` pointer and reading
its context. That makes the refusal **stronger**: the book's own form misses, and the form
that hits is ours. The **Greenwood precedent repeating** — reading the primary *dissolved*
the fork instead of balancing it. **⚠⚠ FINDING 2, THE STRUCTURAL ONE: the partition table is
the blocker and it is UNCITED.** `fs = 0.10` at DVS 2.0 with **flat extrapolation**, so 10 %
of every day's growth keeps going to the stem through grain fill and past maturity;
`allocation.yaml` flags the whole table `TODO(cite) — provisional`. **[A]'s trigger ("once
stems stop growing") is not merely unfireable here — it is a STATEMENT ABOUT [A]'s PARTITION
TABLE**, in which `fs` reaches zero. And [A]'s *other* form is blocked by a sibling gap: an
overflow fill needs a grain sink that can be **full**, and ours is `fo·DMI` with no
capacity. **Both of the book's forms are blocked by something structural we lack, one in the
table and one in the grain.** ⇒ the (C)/canopy-regulator shape a **third** time — with the
difference that **this mechanism is NOT inert**: it moves grain by **half** and passes every
gate. The refusal is *"what it rests on is uncited"*, not *"it does not work"*. **FINDING 3
— the provenance ranking of the three numbers is exactly INVERTED against what matters**:
the drain rate is **bit-inert on carbon** (checked at `to_bits()` over every stock at every
step: across 0.05/0.2/1.0 the *only* stocks differing are starch and grain — once carbon is
in the reserve it is already outside maintenance **and** senescence, and grain is too, so
the flow is a **rename**), the trigger is near-inert (peak LAI **bit-identical** across DVS
0.0–1.5 while standing starch varies 4.8×), and the **only** number that moves anything is
`fstr` — the one the book tabulates. ⚠ One measured exception: at `rate = 0` nitrogen moves,
because grain sits inside Greenwood's `W`, the **nitrogen target's** denominator. **FINDING
4 — the extra grain is the TRANSFER, not the exemptions**: strip both (starch pays
maintenance **and** is shed at `rdr_stem`) and grain is still **+49.6 %** of the full form's
+53.5 %; whole-season accounting shows the system total barely moves (133.95 → 136.78 mol C)
— a **redistribution**, not extra photosynthesis. **FINDING 5 — closure holds, everywhere,
and the controls reproduce the record first** (frozen trough **0.055175**, fixed point
**0.634352**, stem-only's failure **0.046065** — so the harness is known able to report a
failure before it reports a pass): `rationed == 0` on all four chambers at 5 and 15 yr,
**under RK4**, and on **`sealed_station`** (4 yr, grain 24.358 → 39.123) — a leg the
stem-only work recorded **unmeasured** because its biosphere gate failed first. All four
`perennial_long_horizon` manifest gates pass (trough **0.055977**, *above* frozen; liveness
**0.637424** > 0.55). ⚠ **Our reconstruction was measured too rather than deferred**
(advisor catch) — its **discrete one-shot switch** is precisely why RK4 could not be assumed
fine with it; it closes everywhere, and in the shedding-fed chambers leaves the CO₂ trough
**bit-identical to frozen** because the trough precedes the single fill. ⚠ `open_season`'s
bands **hold but the margins shrink** — peak LAI 5.4624 (86.5 % → **91.0 %** of the V-K&S
threshold), `W` 14.1516 (87.6 % → **98.1 %** of the 14.4248 crossing); at Table 7's **top**
row (0.50) it crosses. `n_limited` **keeps its regime** (0.178930/186 vs the recorded
0.175851/187) — option (A) deleted that knob, this does not. **Option (B)'s litter C:N
identity survives** (shedding-fed +1.4 %; reset-driven +18 %, *toward* real residue). ⚠
**The maintenance treatment is a FORK, named not buried**: starch-pays-maintenance **fails**
CO₂ stationarity while the exempt form passes — chosen on the independent argument that
`storage_c` is already exempt for the same stated reason, **not** because it goes green.
**FINDING 6 — it rescues ONE of stem-only's two closure legs** (floor 0.046065 →
**0.053127**, above 0.05) and **not** the other (stationarity still fails) ⚠ recorded as a
measurement, **not** a re-opening. ⚠⚠ **THE ORACLE HI IS NOT A TARGET AND WAS NOT USED AS
ONE**: two sampled variants land within parts-per-ten-thousand of it — **a coincidence of
where the sweep was sampled**, pinned with that label, and `fstr = 0.20` is **Sorghum's**
row. **THE SUCCESSOR IS THE PARTITION TABLE**, not this mechanism: a DVS-keyed `fs` reaching
zero, cited to a primary — with that in place [A]'s own trigger becomes fireable and the
choice of formation form stops being ours. Priced if taken later: 1 stock + 2 flows ⇒
`flow_set` + `param_files` + a new param file, every carbon golden, the station manifest,
`biosphere_params.txt`, the Rust mirror, crossport. Also standing and **named as
unmeasured**: whether a table whose `fs` reaches zero fixes the stem shape *by itself*, with
no reserve — a different change with a different sign on the harvest index. `git diff src/`
empty; nothing unfrozen; ruff + pyright clean.
