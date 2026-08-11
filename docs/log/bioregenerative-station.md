## **The second authored habitat — `scenarios/bioregenerative_station.yaml`** (the first habitat's finding #1, discharged)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE 2026-08-11 — runtime-only; nothing unfrozen, no golden, no manifest entry, `git
diff src/` empty.** `docs/plans/post-roadmap-bioregenerative-station.md`; 18 tests in
`tests/test_bioregenerative_station.py`. **The charge.** `scenarios/` had held exactly one
file since 2026-07-16, and that one closed with a finding: *“the flow registry is crew-only
… an authored ecosystem must invent its kinetics rather than compose frozen, calibrated
laws. This is the biggest gap between the roadmap's promise (‘a scenario can define a
habitat with its power budget, thermal limits, crew size, and ecosystem’) and the platform
as built.”* The registry has since grown 3→12, the grammar gained `monod`, authoring gained
a coupling cadence — and **no authored content had used any of it**. This station uses all
three: **7 of its 17 flows are frozen types** across ECLSS/power/thermal, and it is the
first authored file with a power budget, thermal limits, a crew and an ecosystem in one
graph. **What it closes.** CARBON, OXYGEN and WATER each close with **zero boundary stocks**
(drifts +3.1e-12, +4.3e-12, −2.8e-14 relative over a sealed year of 8760 master steps);
ENERGY is **deliberately open** — 126.144 GJ in from the sun, 124.153 GJ out to space, 1.99
GJ stored in the node — because a station is a heat engine and any other claim would be
wrong. Water is the quantity the first habitat put out of scope. **THE FINDING THAT MADE IT
POSSIBLE: a frozen flow's “boundary” wiring field is a NAME, not a constraint.**
`interpreter._build_flow` validates only that the wiring KEY NAMES match the flow type's
declared fields and passes each value through as a plain `StockId` — there is no `kind`
check anywhere on the path. So `co2_removed` → an interior pool, `humidity_condensate` → an
interior pool, `o2_supply` → a tank photosynthesis refills: **the calibrated equipment stops
discarding and starts recycling**, with no engine change. Nowhere documented as an authoring
capability. **THE SECOND FINDING: the composition rule is about LEG SHAPE, not about
frozen-vs-authored.** `algae_habitat.yaml` needs `{carbon:1, oxygen:2}` on its CO₂ pool or
oxygen cannot close; all three ECLSS fixtures warn that an authored cabin must NOT annotate
composition. Read together those look mutually exclusive, and the advisor's pre-work pass is
what caught the collision. They are not: a **two-leg one-magnitude** transfer (−R here, +R
there) balances every quantity automatically whatever composition its ends share, while a
flow whose legs cross composition classes with INDEPENDENT magnitudes does not. So the
scrubber / condenser / regulator compose freely with annotated pools;
`eclss.crew_metabolism` and `crew.food_metabolism` cannot. The existing warnings are correct
about their own subject and **over-general as written**. **THE THIRD FINDING — `monod` earns
its unfreeze on content.** A photobioreactor is light-limited, so the faithful law is
ZEROTH-order (`P = V_max`) — and zeroth-order is exactly the shape that cannot be made
positivity-safe, because the draw does not know how much substrate is left. `monod` gives
`V_max·S/(S+K)`: → `V_max` at saturation (the physics) and → `(V_max/K)·S` at depletion
(donor-controlled, self-limiting at 0). **The saturating form RESTORES structural positivity
to a law that has none** — the first habitat had to trade the physics for the safety (it
dropped the ∝biomass factor because `k·light·B·dt` reached ~18); this one does not.
Measured, not argued: deleting the op and leaving everything else identical makes the
backstop fire **8731 times out of 8760 master steps**. A stronger justification than
`post-roadmap-grammar-monod.md`'s, which mirrored a frozen kernel. **THE FOURTH FINDING —
the analytic fixed point is NOT what a multi-rate run exports, and the gap is closed-form.**
Three pools settled 0.18–0.91 % below their continuous steady states, and that is neither
transient (they are 63–315 τ in) nor error: for a pool fed only by FAST flows and drained by
ONE slow first-order flow, Strang puts the fast block *between* the slow set's two
half-steps, so the inflow takes one half-step of decay before the export point while the
standing amount takes two — `X_exported = X_continuous · 2a/(1+a)`, `a = 1 − k·(dt/2)`.
**Exact to 14 significant figures on all three** (2.5e-14, 8.0e-14, 6.6e-15). Pinned as its
own test rather than absorbed into a tolerance, because a tolerance is precisely where a
mis-lowered partition would hide (`eclss_multirate_cabin.yaml`'s point: a mis-driven
partition does not drift, it lands somewhere else). **The partition is physics, not a
fixture device.** That fixture's header is candid that its own split is *“a fixture device,
not a sizing claim”* — ECLSS's four flows are all the same order, so it had to manufacture
one cross-boundary stock. Here cabin air (500–2000 s) vs biology (weeks) vs radiator (52 h)
vs battery (3.2 yr) is real, and **seven** stocks genuinely cross the rate-class boundary.
The crew's forced flows are classed FAST on purpose: sub-stepping a forced flow is exact and
free, whereas classing `crew.respiration` slow injects 0.854 mol per half-step into a cabin
pool holding 0.4745 — a 180 % kick into a pool whose own τ (1000 s) is *shorter* than the
slow step. It would not error; it would export a sawtooth. **Which rate class a flow is in
is a MODELLING decision, not a performance knob.** **The design arithmetic caught a blow-up
before the file was written** (the first-habitat discipline, repeated): the textbook Monod
*growth* law `μ·B·φ(S)` gives complex eigenvalues — a substrate–consumer ring with a
**164-day period** and a 43-day decay, i.e. most of a year to complete one cycle from
initial conditions far outside the linear regime. The light-limited form makes them real
(trace −1.13e-6, det 2.67e-13, disc +2.18e-13; τ = 34.7 d and 14.5 d), and a one-year run is
~10.5 τ. **Mutation-verified at five different layers** (a passing test proves nothing until
seen to fail): stripping the CO₂ composition → build-time `AuthoringError` on OXYGEN;
re-pointing the scrubber at a boundary → run-time `ConservationError`; starting `cabin.o2`
above the setpoint → `ReversedFlowError` at the INITIAL state; classing the scrubber `slow`
→ build-time rate precondition (`k·1800 = 1.8 ≥ 1`, the exact false-PASS trap
`_effective_step` documents); dropping `monod` → `RationedError` ×8731. **Residue of the
first habitat's finding #1:** it narrows from *“the registry is crew-only”* to *“the
registry has no composition-aware metabolic flow”* — `crew.food_metabolism` is carbon-only
and `crew.oxygen_consumption` oxygen-only, so wiring either into a `{carbon:1, oxygen:2}`
pool fabricates or destroys oxygen. Physiology stays authored. Also unchanged from July:
**no registered loader exposes a half-saturation**, so `K = 300.0` is a bare inline literal
(`monod_dsl.yaml` measured this; still true, now on real content). **Honest limitations:**
the ten authored laws are invented and the frozen values the seven composed types read are
`DESIGN` placeholders, not measurements — *registered ≠ calibrated*, and
`has_authored_kinetics = True` regardless; water recovery is an idealised 100 % (real
processors reject a brine); and **the energy half is budgeted, not coupled** — the reactor's
lamp power sits inside `load_power` as a number, but `V_max` is a constant, not a function
of bus power. That coupling is the named successor, deliberately not attempted (it needs an
invented light-response law and would destroy the analytic fixed point the gates rest on).
