# Phase 5 — Sibling Domains (Power first; energy joins the conserved set)

**Status: IN PROGRESS — Steps 1–4 (Power) + Step 5 (Thermal) COMPLETE; Atmosphere-ECLSS / Crew
remain forward-pointers.** The load-bearing decision (energy closure, P5.1) and the standalone
Power domain (P5.2–P5.5) are designed in full and advisor-reviewed below; **Thermal is now the
second complete standalone sibling** (Step 5 — full design in the "Step 5 — full design" section
below; advisor-reviewed before any code). Atmosphere-ECLSS / Crew are forward-pointers, each to be
designed just-in-time (the Phase-1/2/3/4 rhythm).

**Step 5 (Thermal) — COMPLETE.** New `src/domains/thermal/` (zero core change): a forced heat input
→ an in-system thermal node (POOL, sensible heat J with a **derived temperature** `T = T_space +
Q/C`) → a **nonlinear Stefan-Boltzmann radiator** (`R = εσA(T⁴ − T_space⁴)·dt`) rejecting to the
**permanent** `boundary.space`. The genuinely-new machinery over Power (temperature, heat capacity,
a nonlinear restoring force) yields a **real emergent equilibrium temperature** `T_eq` — a genuine
attractor, unlike Power's *constructed* daily balance. The load-bearing implementation constraint
(advisor): T⁴ trades `SelfDischarge`'s clean structural positivity for **sizing-dependent** positivity
— structural at the floor (`R → 0` as `Q → 0`, the node referenced to `T_space`) + `τ = C/(4εσA·T_eq³)
>> dt` near equilibrium (`rationed == 0` the `LoadDraw` way). 39 tests (22 flow + 15 run + 2 golden);
additive **NON-frozen** golden. `boundary.space` is a permanent boundary Thermal cannot move inward —
**Phase 6 rewires Power's dissipation legs to feed `thermal.node`** (standalone builds the receiver).
Zero core change (`git diff src/simcore/` empty); full suite (incl. `-m slow`) + ruff + pyright green
(**1132 passed**); seven frozen + two demo + two Power goldens byte-identical (no regen). **Step 1 (P5.1a) — the isolated ENERGY-assert flip — is DONE** (ENERGY joined
`ASSERTED_QUANTITIES`, proven inert; a follow-up swept the last stale decision-#8 prose so the
Power commit's `git diff src/simcore/` is empty). **Step 2 (P5.2 core) — the standalone Power
flows — is DONE** (new `src/domains/power/`: `power.battery` POOL + `boundary.solar_source`/
`boundary.waste_heat`; `SolarCharge` 3-leg heat-named + `LoadDraw` dissipative; `charge.yaml`
one-way η_c; 18 per-flow tests; zero core change; seven frozen + two demo goldens byte-identical).
**Step 3 (P5.3) — the standalone run harness + bounded-SOC validation — is DONE** (new
`scenario.py` + `system.py`: `build_power`/`power_resolver`/`run_power`, the `season.py` analogue
**minus** the reset hook; a half-sine day/night solar schedule; the **load derived for exact daily
energy balance** — `load_w = load_fraction·η_c·(Σ_day solar)/steps_per_day`, since the *forced*
flows make SOC a restoring-force-free accumulator where only exact balance is bounded (the
advisor's physics correction — option A, not a hand-tuned constant); 14 validation tests:
ENERGY conserved every step, `rationed == 0`, `events == ()`, day-over-day SOC return, material
SOC swing, monotonic heat, determinism + RK4≡Euler + registration-order independence; **zero core
change**, seven frozen + two demo goldens byte-identical).
**Step 4 (P5.4) — the hex-float Power golden capture — is DONE** (`tests/test_regression_power.py`
+ `tests/regression/golden/power_state.json`): pins `BOUNDED_SOC_SCENARIO`'s 7-day final `State`
via `sim_io.dumps`, the additive-`n_limited`-golden discipline (byte-match + load-back + separate
`__main__` regen; **additive, NON-frozen** — the Power domain's own regression pin, not in the
biosphere manifest). The **pre-golden gate** bakes in Power's purpose — `rationed == 0`,
`events == ()`, the **per-step ENERGY ledger balances** (residual ≤ 1e-6 — the energy-closure
payload, the analogue of the N-limited "`f_N` actually bit" gate: an imbalanced run is
**unpinnable**), material SOC swing, and day-boundary return to `battery0` (Step-3's exact
tolerances, so legitimate 7-day round-off doesn't fail regen). A day-boundary final state is
deliberately blind to intra-day *shape* (that coverage — interior minimum, half-sine, monotonic
heat, RK4≡Euler — stays in `test_power_run.py`; the biosphere pinned-State + separate-behavioral
division, no drift-summary analogue needed). Within-build bit-stability (`math.sin` transcendental
caveat, as for the biosphere goldens). **Zero core change** (`git diff src/simcore/` empty); full
suite (incl. `-m slow`) + ruff + pyright green (1069 passed); **seven frozen + two demo goldens
byte-identical** (no regen). The biosphere stays FROZEN (`docs/biosphere-reference.md`); Phase 5
builds *beside* it, never moves it. **Next: Step 5** — the forward-pointer siblings
(Thermal / Atmosphere-ECLSS / Crew), each designed just-in-time; cross-domain coupling is Phase 6.**

Phases 0, 0.5, 1, 2, 3, and 4 are complete and regression-pinned
(`docs/plans/phase-{0-engine-skeleton,0.5-numerical-foundations,1-single-producer,2-closed-chamber,3-modular-biosphere,4-closed-biosphere}.md`).
Phase 4 froze the biosphere as **THE reference** (`docs/biosphere-reference.md` + manifest).
Phase 5 proves the engine carries **more than biology**.

## Relationship to the roadmap

**Goal (roadmap lines 313–325):** *Prove the engine carries more than biology. New science
enters Python first — so do new domains. Build (each alone, each with stocks, flows, params,
tests, docs before integration): Power, Thermal, Atmosphere/ECLSS, Crew.*
**Rule:** *Each domain is verified standalone against its own references before it touches the
biosphere.* **Exit criteria:** *Each domain passes conservation and determinism alone.*

Phase 5 is **breadth, not depth**: four sibling domains, each the same stock-and-flow pattern
the biosphere proved, each registered into the **same** registry / integrator / ledger (roadmap
lines 102–112: *one integrator, one ledger*). **No cross-domain coupling yet** — coupling
through shared stocks is **Phase 6** (station integration). A Phase-5 domain is developed and
verified **alone**, then left ready to register.

**Sequencing — Power first (the memory decision).** Power is the natural first sibling because
it forces the one structural change the roadmap has anticipated since Phase 0 but deferred:
**energy joins the conserved set** (roadmap lines 6, 46–51: *"Energy joins the conserved set,
with explicit heat closure… every electrical draw must name where its energy went."* Line 318:
*"Energy enters the ledger in practice."*). Until now `ENERGY` was a tracked-but-**balance-exempt**
quantity (decision #8). Power is where it stops being exempt. The other three siblings (Thermal,
Atmosphere/ECLSS, Crew) reuse the energy-closure machinery Power lands and add no comparably
load-bearing core decision — so they are sketched here and designed when reached.

## The central reframing — Phase 5's load-bearing work is **energy closure**, not "a new domain"

Adding a domain is, by now, routine: the biosphere is **five** registered compartments
(`atmosphere` / `soil` / `plants` / `water` / `consumers`) assembled by `build_season`, each its
own stocks + flows + params + tests. A sibling *domain* is the **same pattern at station scale**
(roadmap line 288: *"a biosphere subsystem and a station domain are the same pattern at different
scales"*). The registry, the source-resolver `Environment`, the every-step conservation gate, the
arbitration backstop, Euler/RK4, the golden discipline — **all of it generalizes directly**.

So the one genuinely new thing in Phase 5 is **making `ENERGY` a conserved, every-step-asserted
quantity with explicit heat closure**. That is the load-bearing decision (P5.1), and the Power
domain (P5.2) is its first concrete carrier. Everything else is the established rhythm.

## Locked decisions (Phase 5)

### P5.1 — `ENERGY` joins the asserted conserved set; "closed" means the **augmented system balances**, with **explicit heat closure** ("every joule named; usefulness is not conserved"). *(The load-bearing decision — designed in full; advisor-reviewed before any process step.)*

Two parts: a one-line **enabling edit** (trivial), and the **energy+heat topology** it enables
(the actual design — the advisor's correction: *the flip is not the design*).

#### (a) The enabling edit — flip `ENERGY` into `ASSERTED_QUANTITIES`

`simcore/quantities.py` currently defines
`ASSERTED_QUANTITIES = frozenset(Quantity) - {Quantity.ENERGY}` (decision #8: *"energy closure
is Phase 5/6; here energy is a diagnostic only"*). Phase 5 **removes the `ENERGY` exclusion** so
the every-step gate (`conservation.assert_conserved`) and the flow-balance check
(`flow.assert_flow_balanced`) balance `ENERGY` like the four mass quantities. `canonical_unit[ENERGY]`
is already `J`; no unit work.

**This is the one sanctioned `simcore/` edit in Phase 5.** Frame it correctly: the
*"`git diff src/simcore/` empty — unconditionally"* rule was the **biosphere-freeze** invariant
(`docs/biosphere-reference.md`), **not** a global ban on ever touching core. This edit (i) changes
a core *frozenset* that the roadmap always intended to change at this phase (lines 6, 318), (ii)
touches **no biosphere-frozen surface** (no biosphere stock carries `ENERGY` — see the
golden-safety proof below), and (iii) is the literal meaning of *"energy enters the ledger in
practice."* It is sanctioned, scoped, and isolated — not a freeze violation.

**Golden-safety — proven, not assumed (the advisor's blocker, resolved).** The gate **skips** a
quantity absent from the state (`conservation.py`: `if ql is None: continue`) but **raises** if a
*present* `ENERGY` leg/stock fails to balance. So the flip is a no-op **iff** no existing run has a
present-but-unbalanced `ENERGY` flow. Two suspects, both cleared:
- **The biosphere (7 frozen goldens):** has **no `ENERGY` stock** — light / net-radiation enter as
  **MJ forcing read through `env.get`** (a scalar rate multiplier), never as a tracked stock or a
  leg. No `ENERGY` ledger entry exists → the gate skips it → every biosphere golden is
  **byte-identical**.
- **The Phase-0 demo (`flows.py` / `demo.py` / `demo.yaml`) — the real suspect:** **does** carry an
  `ENERGY` stock — `boundary.source(LIGHT, Quantity.ENERGY, 1.0)` (`demo.py`). But **no leg ever
  touches it**: `Photosynthesis` reads `light` from `env` as a scalar multiplier, not a consumed
  leg. So `boundary.light`'s amount is constant and its per-step delta is **always exactly 0** →
  `residual == 0` → passes trivially. The flip makes the demo's existing `for q in
  ASSERTED_QUANTITIES` conservation check (`test_biosphere_demo.py`) *also* assert `ENERGY` — which
  passes and **strengthens** the test. `demo_euler/rk4_state.json` are byte-identical.

  **The empirical proof is the full suite** (incl. `-m slow`), not the 7 goldens alone — run it
  after the flip and confirm green + zero golden churn. (Reasoning says safe by construction; the
  suite is the receipt.)

**Edit surface beyond the one line — the now-stale decision-#8 prose.** "`ENERGY` is
balance-exempt / closure is Phase 5/6" is asserted as present-tense **fact** in `quantities.py`,
`flow.py` (module docstring + `assert_flow_balanced`), `conservation.py` (`compute_ledger` +
`assert_conserved`), `boundary.py` (`loss_sinks` docstring), and `demo.py` (the `light` comment).
All become **wrong** on the flip and are part of the edit — reworded to: *`ENERGY` is now a
conserved, asserted quantity (Phase 5); it was exempt through Phase 4.* `per_quantity_residual` /
`compute_ledger` already *report* `ENERGY` (they always did), so only the **prose** and the
**asserted-set membership** change, not the ledger arithmetic.

**`loss_sinks()` default.** Its default arg is `ASSERTED_QUANTITIES`, so post-flip the default
*would* build an `ENERGY` loss-sink. `ENERGY` has **no `POPULATION` stock** (extinction routes
biomass carbon only), so it needs none. **No production caller uses the default** — `build_season`
and `build_demo` both pass explicit `{Quantity.CARBON}`; only `test_boundary.py` calls
`loss_sinks()` bare, and it asserts `set(sinks) == {loss_sink_id(q) for q in ASSERTED_QUANTITIES}`
— a self-tracking tautology that stays green. Leave the default as-is (it remains correct: "one
loss-sink per asserted quantity"); Power's `ENERGY` POOL/BOUNDARY stocks simply never use one.

#### (b) The energy + heat topology — what "every joule named" actually means *(the design)*

Energy obeys `Inputs = Outputs + ΔStored` like any mass quantity (roadmap line 47). The new
content is **heat closure**: energy **degrades** — every electrical draw deposits its energy as
heat (minus any useful work / mass flow carried away). A model whose joules balance while **heat
appears from nowhere** is wrong (roadmap line 51). So:

- **Every Power flow is internally balanced in `ENERGY`** (`Σ legs == 0`), exactly as biosphere
  flows balance carbon. A lossy transfer is a **multi-leg** flow that books the loss as heat. The
  canonical example — a **battery charge with round-trip inefficiency** is **3 legs**:
  `bus −100 J, battery +95 J, waste_heat +5 J` (η = 0.95), summing to 0. The 5 J did not vanish; it
  is **named** as heat. This is the structural enforcement of "every joule named."
- **"Usefulness is not conserved"** is a **diagnostic**, not a second ledger: joules balance
  (the gate), but a **monotonic heat-generated accumulator** records the cumulative degraded
  energy (roadmap line 50). The `waste_heat` boundary sink's *amount* **is** that accumulator —
  it only ever receives, so it is monotonic **by construction** (the biosphere loss-sink-as-free-
  diagnostic precedent).

**Standalone "closed" = the augmented system (stocks + boundary reservoirs) balances** (decision
#13) — **not** "no energy crosses the boundary." Heat leaving to a **`waste_heat` BOUNDARY sink**
is a **legitimate Output**, the same way Phase-1/2 biosphere transpiration drained to a
`vapor_sink` BOUNDARY before Phase 3 closed the water cycle. This is what makes asserting `ENERGY`
in *standalone* Power **correct rather than premature**: the ledger balances over the augmented
system every step, even though heat physically leaves.

**The boundary moves inward later — the explicit progression (the real content of "designed in
full").** Standalone Power dumps heat to `boundary.waste_heat`. The **Thermal** domain (a later
Phase-5 sibling, or Phase 6) **moves that boundary inward** into a real heat **stock** + radiator
rejection — exactly as Phase 3 closed
`vapor_sink → water_vapor → condensate → soil_water` for water. Power's `waste_heat` BOUNDARY is
the **seam** Thermal will sever. Stating this is what licenses the standalone design: we are not
ignoring heat, we are routing it to an explicit boundary that a sibling domain later owns.

#### (c) Energy vs **electricity** — one conserved quantity (joules), two **forms** (the distinction made explicit)

**Electricity and heat are different *forms* of energy, not different *conserved quantities*.**
This distinction is load-bearing and must be explicit — but it lives at the **stock/flow level**,
**not** in the `Quantity` set. The reason is structural, not stylistic:

- The conservation gate asserts `Σ legs == 0` **per `Quantity`, independently**
  (`conservation.assert_conserved`). A joule of **electricity** turning into a joule of **heat** is
  genuine **transmutation of form** — `battery(electrical) → waste_heat(thermal)`. If `ELECTRICAL`
  and `THERMAL` were *separate* `Quantity` members, that load flow would post `−100` to the
  electrical residual and `+100` to the thermal residual — **neither balances**, and the gate would
  raise `ConservationError` **every step**. A per-quantity ledger **cannot** balance a conversion
  across two of its quantities. So electricity and heat **must** share one conserved `Quantity` for
  the ledger to be coherent.
- This is exactly the roadmap's design (lines 46–51): *"Energy obeys `Inputs = Outputs + ΔStored`
  like any other quantity. But energy **degrades**… **Track joules in the ledger. Track
  heat-generated as a monotonic diagnostic.**"* **One** conserved currency (joules); the
  electricity→heat degradation is a **diagnostic**, because **usefulness (exergy) is not
  conserved** — a joule of electricity can do work a joule of waste heat cannot, yet both are joules
  and the first law conserves their sum.

**So the distinction is carried, explicitly, by:**
1. **Which stock holds the joules — the form is a property of the stock, made plain in its id/role.**
   `power.battery` and `boundary.solar_source` hold **electrical** energy; `boundary.waste_heat`
   holds **thermal** energy. The same `Quantity.ENERGY` (J), different **form**. (Phase 6's
   cross-domain seam — *"power fills electrical energy; biosphere draws it for lighting"* — is a
   **shared electrical stock**; the shared stock *is* the electrical-form marker, the same way the
   shared CO₂ stock is the carbon interface.)
2. **The degradation is a *named flow leg*, not a hidden loss.** `LoadDraw:
   battery(electrical) → waste_heat(thermal)` **is** "electrical energy degrading to heat"; the
   lossy `SolarCharge`'s 3rd leg is "charge-conversion loss degrading to heat." Every form-change is
   a visible, balanced leg.
3. **The monotonic heat-generated accumulator** (the `waste_heat` amount) measures **cumulative
   degradation** — the quantitative statement of "usefulness is not conserved."

**Deliberately NOT done now (a flagged seam, not an omission):** a formal `EnergyForm`
(`ELECTRICAL` / `THERMAL` / …) **label on `Stock`** would make the form machine-queryable
(diagnostics, UI later). It is **deferred** — it is a `simcore` surface addition with no flow or
test that yet needs to *discriminate* on it (the role is already unambiguous from the stock id), and
this codebase rejects speculative generality until a consumer exists. If Thermal or the Phase-6
UI needs to partition energy by form, that is the moment to add the label — with a consumer, not on
spec. **Until then: energy is one conserved quantity; electricity vs heat is read off the stock.**

### P5.2 — The standalone Power domain: solar (forced) → battery (POOL) → load + losses → heat (BOUNDARY). *(Designed in full; the first carrier of P5.1.)*

A minimal, **standalone, genuinely energy-conserving** Power domain — the Power analogue of the
Phase-1 single-producer season: it ships the **machinery** (energy-balanced multi-leg flows, the
every-step gate now covering `ENERGY`, `rationed == 0` by construction, determinism, a golden), not
a calibrated power-system model.

**Stocks (energy currency = J; all `ENERGY`):**
- `boundary.solar_source` — **BOUNDARY**, `unclamped=True`. The electrical supply (panel
  conversion folded into the forcing — incident sunlight is **not** a tracked stock, mirroring the
  biosphere treating light as forcing). Its (negative-going) amount is cumulative supply
  bookkeeping (#13). *Nuclear/grid input is the same shape with a flat schedule — a documented
  seam, not built.*
- `power.battery` — **POOL**. Stored electrical energy (state of charge). Arbitration may throttle
  draws against it; it is the one stock the backstop guards.
- `boundary.waste_heat` — **BOUNDARY sink**. Receives every degraded joule (battery round-trip
  loss, the dissipative load, optional self-discharge). Monotonic ⇒ **free heat-generated
  diagnostic** (roadmap line 50). The seam Thermal later moves inward.

**No pass-through "bus" POOL.** A near-zero pass-through pool **cannot source a flow** under the
arbitration backstop (it scales withdrawals against the *start-of-step* amount — the Step-11
`plant_c`-buffer lesson, in memory). So solar deposits **directly into the battery** and loads draw
**directly from the battery** — the battery **is** the bus + storage. This avoids re-importing a
known trap.

**Flows (all increment-form `flux = rate·dt`, `rate` in **W**, dt-independent ⇒ RK4-order-safe,
Phase-6-multi-rate-safe):**
- `SolarCharge`: `solar_source → battery (+η_c) + waste_heat (+(1−η_c))`, magnitude
  `env.get("solar_power")·dt`. Charge-efficiency loss named as heat (3 legs; η_c = 1 collapses it
  to 2). Solar power is a **day/night forcing schedule** (the analogue of the weather table).
- `LoadDraw`: `battery → waste_heat`, a **dissipative load** (100 % → heat — a resistive/compute
  load; this is the cleanest "every joule named" demonstration). Magnitude `env.get("load_power")·dt`
  (forced demand). *Useful work leaving the system (a pump moving fluid → a `useful_work` BOUNDARY
  sink) is the documented split-seam, deferred — standalone Power makes **heat** the star.*
- `SelfDischarge` (**LANDED, P5.5**): `battery → waste_heat`, first-order `k·battery·dt` — a
  standing leak; donor-controlled so it self-limits to 0 as the battery empties. It **earned its
  keep**: as the Power domain's **first donor-controlled flow** it brings three properties the two
  forced flows cannot — (1) `rationed == 0` structural for *its own* leg (`k·dt < 1`; LoadDraw still
  leans on sizing), (2) the domain's **first restoring force** → a stable SOC attractor, proved
  **magnitude-independently** by a two-run contraction test (`d_n = d_0·(1 − k·dt)^n` exactly;
  forced-only keeps `d_n` constant), and (3) it **breaks** the forced-only RK4 ≡ Euler bit-identity
  (now a tolerance agreement). Realistic Li-ion `k ≈ 1e-8/s` (~2.6 %/month; NOT inflated). Added as
  an **opt-in third flow** of `build_power` (`self_discharge_params=None` default ⇒ the `BOUNDED_SOC`
  golden + bit-identity untouched); reuses `BOUNDED_SOC_SCENARIO` verbatim so the leak is the *sole*
  drift driver. Own behavioral test (`test_power_self_discharge.py`) + own additive **non-frozen**
  golden (`power_self_discharge_state.json`). Zero core change; seven frozen + two demo + the
  two-flow Power golden byte-identical.

**`rationed == 0` from kinetics — the sizing discipline (carried from the biosphere).** A *forced
constant* load can over-draw an empty battery → the Euler backstop fires (Euler-only, allowed, but
the golden asserts `count == 0`). Two ways to keep `rationed == 0`, mirroring the biosphere:
- **Well-fed sizing (primary):** size the validation scenario (solar amplitude, battery capacity,
  load) so the battery **never empties** — bounded SOC oscillation, demand always met. This is the
  Phase-1 "PP plot kept non-limiting" pattern, and it is the advisor's validation target.
- **Donor-controlled load (alternative/seam):** make `LoadDraw` first-order in battery SOC, which
  self-limits to 0 — structural positivity, the biosphere first-order idiom. Realism trade-off (a
  real load is a constant-W demand, not SOC-proportional); the Step design picks. A **brownout**
  scenario (forced load > supply, battery empties, backstop engages / rationing bites) is the Power
  analogue of drought/extinction — a **later perturbation step**, not the baseline golden.

**dt / time unit — seconds (SI; documented, not implicit).** Energy is J, power is W = J/s, so
Power's natural currency is **seconds**, unlike the biosphere's implicit `dt = 1 day`. A day/night
solar cycle wants a sub-day step to resolve it (e.g. `dt = 3600 s`, 24 steps/day — the exact value
is a Step-design choice, **documented** like the biosphere's daily step). Increment-form flows keep
determinism + RK4 order and **do not paint Phase 6 into a corner**: Phase-6 multi-rate sub-steps the
fast power domain a fixed, seeded count inside one slow biosphere step (roadmap lines 146–149) — an
increment-form `rate·dt` flow sub-steps cleanly.

**The standalone validation target (gives "passes conservation + determinism alone" teeth).**
Forced solar **day/night** + a forced load the battery can always meet → a **bounded battery-SOC
oscillation** (charge by day, discharge by night), `ENERGY` **conserved every step** (the augmented
ledger balances — `Δsolar + Δbattery + Δwaste_heat ≈ 0` at round-off), `rationed == 0`,
heat-generated **monotonically increasing**. This is the Power analogue of the biosphere's emergent
limit cycle — an emergent diurnal SOC cycle with no control code, the same `Inputs = Outputs +
ΔStored` machinery. Pinned as the Power golden (hex-float `State` snapshot, the biosphere
discipline).

### P5.3 — Each sibling domain is verified **standalone** first; **no cross-domain coupling in Phase 5** (that is Phase 6).

Roadmap line 323: *each domain is verified standalone against its own references before it touches
the biosphere.* Phase 5 stops at **standalone**. The shared-stock coupling (power fills electrical
energy; biosphere draws it for lighting — roadmap lines 131–135) is **Phase 6**. A Phase-5 domain
is left **ready to register** (its stocks/flows/params/tests/docs complete), not yet wired to a
sibling. This keeps each domain's conservation + determinism proof **clean and attributable** — a
cross-domain bug cannot hide in a standalone green.

### Carried Phase-0/0.5/1/2/3/4 invariants that constrain Phase 5
- **Core pure** (stdlib only). The **one** sanctioned `simcore/` edit is the `ASSERTED_QUANTITIES`
  flip (+ its stale-comment cleanup) — a frozenset membership change the roadmap always intended at
  this phase. No other core edit. **The biosphere-frozen surface is untouched** (`docs/biosphere-
  reference.md` unfreeze discipline does **not** trigger — no biosphere param/flow/scenario/golden
  moves; proven byte-identical).
- **Flows return structured per-stock legs, internally balanced** — now in `ENERGY` too (the 3-leg
  lossy-charge pattern).
- **Conservation asserted every step** — now covering `ENERGY`. **Determinism** bit-identical;
  integer step count (`t = n·dt`, never `+=`); canonical (flow-id) order on every reduction.
- **Arbitration backstop Euler-only, rare**; golden asserts `rationed == 0` (positivity from
  kinetics / well-fed sizing). Under RK4+ a needed scale is a hard error.
- **Parameters are data** (YAML + schema); no hardcoded coefficients — Power's efficiencies /
  capacities / rates are param files, its scenario amounts are scenario data (the
  `loader.py` + `scenario.py` split).
- **Reuse/licensing** (`docs/reuse-and-licenses.md`): Power science is clean-room from primary
  literature; cite the source. (Battery round-trip efficiency, solar diurnal profiles, etc. are
  textbook — cite, don't copy.)

## Scope

### In scope (Phase 5)
- **Energy closure (P5.1):** `ENERGY` joins `ASSERTED_QUANTITIES`; stale decision-#8 prose updated;
  full-suite proof that all 7 frozen goldens + the demo goldens are byte-identical / green.
- **A standalone Power domain (P5.2):** stocks, energy-balanced multi-leg flows (solar charge with
  named heat loss, dissipative load), param file(s), a `SeasonScenario`-analogue scenario, the
  day/night forcing resolver, a hex-float golden, conservation + determinism + `rationed == 0`
  tests. Verified **standalone**.
- **The forward-pointer plan** for Thermal / Atmosphere-ECLSS / Crew (sketched below; each designed
  just-in-time when reached).

### Explicitly deferred (do NOT build in Phase 5)
- **No cross-domain coupling** (shared-stock wiring, cascades) — **Phase 6** (station integration).
- **No Rust port** — after station integration (roadmap line 7).
- **No biosphere change** — it is frozen. Any need to touch it follows the unfreeze discipline and
  is out of Phase-5 scope.
- **No calibration / validation-against-life-support-references** (NASA BVAD / BioSim) — that is
  **Phase 6** (integrated consumption/production validation). Phase 5 ships **verified machinery**,
  not calibrated numbers (the biosphere precedent — `TODO(cite)` placeholders, machinery frozen).
- **Useful-work / mass-flow energy split** on loads — a documented seam; standalone Power loads are
  dissipative (100 % → heat).

## Step sequence (sketched; each designed just-in-time, P5.1 first)

1. **Energy joins the conserved set (P5.1a) — the isolated core flip. — COMPLETE.** Flipped
   `ENERGY` into `ASSERTED_QUANTITIES`; updated the stale decision-#8 prose across
   `quantities.py` / `flow.py` / `conservation.py` / `boundary.py` / `demo.py` (a follow-up commit
   swept three more leftovers — `flow.py` `per_quantity_residual` + the Phase-0 demo `flows.py`);
   full suite (incl. `-m slow`) + ruff + pyright green; **zero golden churn** (all 7 frozen + both
   demo goldens byte-identical). A **measurement step before any Power stock exists** — it proved
   "flipping changes nothing for existing runs," isolating the one risky edit (the Phase-4-Step-1
   "measure before capture" rhythm). No new golden; no Power code.
2. **Power stocks + the energy-balanced flows (P5.2 core). — COMPLETE.** New `src/domains/power/`
   (zero core change): `power.battery` POOL, `boundary.solar_source` (unclamped source) +
   `boundary.waste_heat` (monotonic sink); `SolarCharge` (**always 3 legs**, heat-named —
   `solar_source → battery(+η_c) + waste_heat(+(1−η_c))`; η_c=1 → heat leg exactly 0) + `LoadDraw`
   (2-leg dissipative, 100 % → heat). Both **forced** (`solar_power`/`load_power` W via `env`, ×dt →
   J; increment-form, dt-linear); positivity is a **sizing discipline**, not structural. One param,
   `charge.yaml` `charge_efficiency` η_c ∈ (0,1] — **ONE-WAY charge** (discharge is joule-lossless;
   the discharge loss is exergy → the heat diagnostic), so the **modeled round-trip = η_c**
   (the √-decomposition does not apply to a lossless-discharge model). 18 per-flow tests: balance
   via `assert_flow_balanced` (**the Step-2 gate** — a full conservation-gate run is Step 3), leg
   structure, dt-linearity, zero-input no-op, loader bounds. **Capacity is NOT a param** (POOL stocks
   have no upper clamp → sizing/scenario data, Step 3). Clean-room + cited. Seven frozen + two demo
   goldens byte-identical.
3. **The Power scenario + day/night resolver + standalone run harness. — COMPLETE.** New
   `scenario.py` (`PowerScenario` — `battery0`/`solar_peak_w`/`load_fraction`/`daylight_hours`/
   `dt_seconds`/`steps_per_day`; `BOUNDED_SOC_SCENARIO` + `BOUNDED_SOC_DAYS = 7`) + `system.py`
   (`build_power` — three stocks + two flows, **no loss-sinks** as Power has no POPULATION stock;
   `solar_schedule` — a half-sine over the daylight window, the weather-table analogue *computed*;
   `power_resolver`; `run_power` — the `run_season` analogue **minus** the reset hook). **The
   load-bearing physics call (advisor):** both flows are *forced* (state-independent), so SOC is a
   restoring-force-free accumulator with **no attractor** — boundedness is **not emergent**, it is
   *constructed* by **exact daily energy balance**. So the load is **derived**, not hand-tuned:
   `power_resolver` computes `load_w = load_fraction·η_c·(Σ_day solar)/steps_per_day` from the
   discrete daily solar sum + the loaded η_c (`load_fraction = 1` ⇒ exact balance ⇒ bounded
   periodic SOC; `>1` is the free brownout knob). This is the one place a Power resolver reads a
   flow param — Power's load is intrinsically η_c-coupled. `dt = 3600 s`, 24 steps/day. Probe:
   swing ≈ 72 % of `battery0`, min-SOC 11.3× a step's draw (`rationed == 0` structural),
   day-over-day drift 1e-7 J (the exact-balance round-off). **14 validation tests** (the non-vacuous
   payload checks the advisor named): per-step ENERGY ledger residual ≈ 0 + the integral
   total-ENERGY invariant; `rationed == 0`; `events == ()`; **day-over-day SOC return** (true only
   under exact balance); material swing with min > 0; interior (morning-crossover) daily minimum;
   monotonic `waste_heat`; the balance identity; determinism; **RK4 ≡ Euler bit-for-bit** (forced
   flows ⇒ k1=k2=k3=k4 — framed as the identity, not robustness); registration-order independence.
   **Zero core change** (`git diff src/simcore/` empty); seven frozen + two demo goldens
   byte-identical (no regen).
4. **Standalone validation + golden capture (P5.4) — COMPLETE.** Pinned the hex-float Power golden
   (`tests/test_regression_power.py` + `regression/golden/power_state.json`): `BOUNDED_SOC_SCENARIO`'s
   7-day final `State` via `sim_io.dumps`, byte-match + load-back + separate `__main__` regen (the
   additive-`n_limited` discipline; additive/NON-frozen). Pre-golden gate = `rationed == 0` /
   `events == ()` / **per-step ENERGY ledger balances** (residual ≤ 1e-6 — the closure payload, so an
   imbalanced run is unpinnable) / material SOC swing / day-boundary return to `battery0` (Step-3's
   exact tolerances). Step 3 already landed the behavioral assertions; Step 4 adds only the pinned
   golden + regen. **Zero core change**; full suite (incl. `-m slow`) + ruff + pyright green (1069
   passed); seven frozen + two demo goldens byte-identical.
5. **Thermal (COMPLETE)** — the second sibling domain, designed just-in-time (full design below).
   New `src/domains/thermal/` (zero core change): `thermal.node` POOL (sensible heat J, referenced
   to `T_space` so `Q = C·(T − T_space) ≥ 0`) + `boundary.heat_source` (unclamped) +
   `boundary.space` (monotonic sink). Flows: **`HeatInput`** (2-leg forced, `heat_source → node`,
   heat→heat lossless — no charge-loss leg unlike Power's `SolarCharge`) + **`RadiatorReject`**
   (2-leg **donor-controlled nonlinear** Stefan-Boltzmann `R = εσA(T⁴ − T_space⁴)·dt`,
   `node → boundary.space`). `radiator.yaml` (ε / area / heat-capacity / T_space; σ is a CODATA
   module constant, **not** a param). The genuinely-new machinery over Power: **temperature**
   (`T = T_space + Q/C`, a derived readout — not a stock/aux), **heat capacity**, and a **nonlinear
   restoring force** → a *real* **emergent equilibrium temperature** `T_eq` (a genuine attractor,
   vs Power's *constructed* daily balance). Positivity is **structural at the floor** (`R → 0` as
   `Q → 0`) and **by sizing near equilibrium** (`τ = C/(4εσA·T_eq³) >> dt` — the load-bearing
   constraint, `rationed == 0` the `LoadDraw` way, not a structural `k·dt < 1` claim). Resolver is a
   plain constant `heat_load` (the radiator is the balance — **no** derived load, unlike Power's
   `balanced_load_w`). Validation (`EQUILIBRIUM_SCENARIO`, cold node warming to `T_eq ≈ 280.9 K`,
   τ ≈ 65 steps, 720-step / ~11τ horizon): ENERGY conserved every step, `rationed == 0`,
   `events == ()`, monotonic `space` sink, **T converges to `T_eq`**, **two-run monotone
   contraction** (nonlinear, not geometric — the no-radiator contrast keeps the difference
   *constant*), **RK4 ≢ Euler** (tolerance agreement — nonlinear donor-controlled), determinism,
   registration-order independence. Additive **NON-frozen** golden (`thermal_state.json` +
   pre-golden gate = ENERGY closed / `rationed == 0` / **reached equilibrium**). **`boundary.space`
   is a permanent, true boundary** — heat leaves to deep space forever; standalone Thermal does NOT
   "close" anything (unlike Phase-3 water). Power's `waste_heat` was a *temporary seam*; Thermal
   reveals the "somewhere" = an in-system node + radiator, and **Phase 6 rewires Power's dissipation
   legs to feed `thermal.node`** (the inward move — a Phase-6 wiring act; standalone builds the
   *receiver*). Zero core change (`git diff src/simcore/` empty); seven frozen + two demo + two Power
   goldens byte-identical.
6. **(Forward-pointer) Atmosphere-ECLSS / Crew** — each a sibling domain on the same pattern,
   designed just-in-time:
   - **Atmosphere / ECLSS** — O₂, CO₂, water vapor, pressure; mixing, scrubbing, condensation. Pure
     mass-quantity domain (no new core decision — reuses the frozen mass machinery).
   - **Crew** — O₂ intake, CO₂ output, water/food consumption, waste; **forced schedules at first**
     (roadmap line 321). The eventual biosphere coupling (crew CO₂ ↔ plant uptake) is **Phase 6**.

## Step 1 — full design: the isolated ENERGY-assert flip (P5.1a)

*The de-risk that gates Phase 5's energy work — a **measurement**, designed in full and
advisor-reviewed before any Power code (the Phase-1/2/3/4 rhythm). The change is small; the
**discipline** is the point: isolate the one risky core edit, prove it inert, then build Power on a
proven-conserved energy ledger.*

### Deliverables (one core edit + a verification, no new golden, no Power code)
1. **`simcore/quantities.py`** — `ASSERTED_QUANTITIES = frozenset(Quantity)` (drop the
   `- {Quantity.ENERGY}`). Rebase the surrounding comment from *"ENERGY is exempt (decision #8:
   energy closure is Phase 5/6)"* to *"all five quantities are asserted; ENERGY joined the conserved
   set in Phase 5 (was exempt through Phase 4)."* Keep `BALANCE_ATOL/RTOL` as-is.
2. **Stale decision-#8 prose** in `flow.py` (module docstring + `assert_flow_balanced` docstring),
   `conservation.py` (`compute_ledger` + `assert_conserved`), `boundary.py` (`loss_sinks` docstring
   line *"ENERGY gets no loss-sink (it is balance-exempt)"* → *"ENERGY is asserted but has no
   POPULATION stock, so it needs no loss-sink"*), `demo.py` (the `boundary.light` comment *"ENERGY is
   balance-exempt, #8"* → *"ENERGY is now asserted; this stock's delta is always 0, so it conserves
   trivially"*). **Prose only** — no arithmetic change (`per_quantity_residual`/`compute_ledger`
   already report ENERGY).
3. **Verification (no new test file needed; reuse the suite):** run `uv run pytest` (full, incl.
   `-m slow`), `uv run ruff check . && uv run ruff format --check .`, `uv run pyright`. **Acceptance:**
   green; **zero golden bytes changed** (`git status` shows only the edited `.py` files — no
   `tests/regression/golden/*` churn). The demo's `for q in ASSERTED_QUANTITIES` conservation loop
   (`test_biosphere_demo.py`) now also asserts ENERGY on the demo and passes — the strengthening is
   the receipt. *(Optionally add one explicit assertion that `Quantity.ENERGY in ASSERTED_QUANTITIES`
   to pin the intent, but the suite already covers behavior.)*

### Why this is golden-safe (restated as the acceptance argument)
- **Biosphere:** no `ENERGY` stock ⇒ gate skips ⇒ 7 goldens byte-identical.
- **Demo:** one `ENERGY` stock with permanently-zero delta ⇒ `residual == 0` ⇒ passes ⇒ 2 goldens
  byte-identical; the demo's conservation test strengthens (now covers ENERGY) and stays green.
- **The full suite is the empirical proof**, not the reasoning alone (the advisor's blocker, closed).

## Step 5 — full design: the Thermal sibling domain (COMPLETE)

*The second standalone sibling, designed just-in-time and advisor-reviewed before any code (the
Phase-1/2/3/4 rhythm). Thermal is the domain that reveals **where Power's `waste_heat` "somewhere"
actually is** — it builds the in-system receiver (a thermal node + a radiator) that Phase 6 wires
Power's dissipation into. Scoped to Thermal **alone** (a P5.2–P5.5-sized effort); Atmosphere-ECLSS /
Crew stay forward-pointers.*

### The load-bearing design fork — the radiation law (advisor-decided: T⁴, not linear)

A linear rejection `R = k·node` would be `SelfDischarge` with the sink renamed — **no new content**.
The **Stefan-Boltzmann `R = εσA(T⁴ − T_space⁴)`** law is (i) the physically correct rejection mode in
**vacuum** (radiation is the *only* mode with no medium), and (ii) genuinely-new machinery:
**temperature** (`T = T_space + Q/C`, the first non-J derived readout — a pure function of the node
amount, computed at evaluate-time, **not** a stock/aux/ledger entry), a **heat capacity**, and a
**nonlinear restoring force**. It yields a **real emergent equilibrium temperature** `T_eq` where
`εσA(T_eq⁴ − T_space⁴) = heat_load` — a *genuine* attractor, contrast Power's `BOUNDED_SOC` whose
boundedness was *constructed* by an exactly-balanced derived load. Because the radiator **is** the
restoring force, **any** constant `heat_load` lands at a unique stable `T_eq`, so the resolver is a
plain constant — **no** `balanced_load_w` analogue (the one place Power had to derive its load).

### The three T⁴ consequences the Power/SelfDischarge pattern-match would miss (advisor)

1. **Positivity is NOT structural like `k·dt < 1`.** Radiated ∝ `T⁴`, not ∝ `Q`, so there is no
   single-param guarantee. Two parts: (a) **at the floor — structural**: the node is **referenced to
   `T_space`** (`Q = C·(T − T_space) ≥ 0`), so `R → 0` as `Q → 0` (the `T⁴ − T_space⁴` term vanishes)
   and the radiator cannot pull `Q` negative (a *warm* reference would break this); (b) **near
   equilibrium — by sizing**: the real risk is **Euler overshoot**, so the relaxation time
   `τ = C/(4εσA·T_eq³)` must be `>> dt`. The scenario sizes the heat capacity `C` so `τ/dt` is tens of
   steps (`τ ≈ 65` steps at `dt = 3600 s`). `rationed == 0` holds **by sizing** (framed like Power's
   `LoadDraw`), **not** by a structural claim like `SelfDischarge`. **This is the load-bearing
   implementation constraint — `τ >> dt` was nailed before capturing the golden.**
2. **Contraction is monotone, not geometric.** `SelfDischarge`'s exact `d_n = d_0·(1−k·dt)^n` is a
   *linear* law; `T⁴` is nonlinear, so two runs' difference contracts **monotonically** (asymptotically
   geometric near `T_eq`). The test asserts monotone contraction + strong final shrinkage, **not** the
   exact formula. The no-radiator contrast (forced `HeatInput` alone keeps the difference *constant*)
   isolates the radiator as the restoring force.
3. **`boundary.space` is a permanent, true boundary — standalone Thermal closes nothing.** Heat leaves
   to deep space **forever** (you cannot move *that* inward). Power's `waste_heat` was a *temporary
   seam*; Thermal reveals the "somewhere" = an in-system thermal POOL + a radiator to the permanent
   space boundary. The inward move is a **Phase-6 wiring act** (rewire Power's dissipation legs to feed
   `thermal.node`); standalone Thermal builds the *receiver*. "Closed" is decision #13's *augmented*
   sense (node + boundary reservoirs balance every step, even though heat physically leaves).

### Deliverables (all additive; zero core change)
- **`src/domains/thermal/`** — `stocks.py` (`thermal.node` POOL + `boundary.heat_source` +
  `boundary.space`), `flows.py` (`HeatInput` 2-leg forced + `RadiatorReject` 2-leg donor-controlled;
  `temperature` / `radiated_power` rate laws; **`STEFAN_BOLTZMANN` σ as a CODATA module constant** —
  a universal physical constant with provenance, the `drift.py` "documented constant, not YAML"
  discipline, **not** a param), `loader.py` + `params/radiator.yaml` (ε / radiator_area / heat_capacity
  / space_temperature — exact-string unit-guarded, absolute **K** for the T⁴ law), `scenario.py`
  (`EQUILIBRIUM_SCENARIO` — cold node, constant `heat_load`, sized `τ >> dt`), `system.py`
  (`build_thermal` / `thermal_resolver` / `run_thermal` + closed-form `equilibrium_temperature` /
  `relaxation_time`).
- **Tests (39):** `test_thermal_flows.py` (22 — rate laws, leg balance, dt-linearity, floor self-limit,
  loader bounds/units), `test_thermal_run.py` (15 — ENERGY conserved every step, `rationed == 0`,
  `events == ()`, `τ >> dt`, emergent `T_eq` + balance identity, monotone warming, two-run contraction
  + no-radiator contrast, monotonic `space` sink, determinism, **RK4 ≢ Euler tolerance agreement**,
  registration-order independence), `test_regression_thermal.py` (2 — additive **NON-frozen** golden
  `thermal_state.json` with a pre-golden gate: ENERGY closed / `rationed == 0` / **reached
  equilibrium** — an unconverged/imbalanced run is unpinnable).
- **Acceptance (met):** `git diff src/simcore/` empty; full suite (incl. `-m slow`) + ruff + pyright
  green (1132 passed); seven frozen + two demo + two Power goldens byte-identical (no regen).

## Exit criteria (Phase 5 — "the engine carries more than biology")
- **`ENERGY` is a conserved, every-step-asserted quantity** with explicit heat closure; the flip is
  proven inert for all existing runs (7 frozen + 2 demo goldens byte-identical; full suite green).
- **A standalone Power domain passes conservation and determinism alone:** the augmented `ENERGY`
  ledger balances every step; bounded non-collapsing battery-SOC oscillation; `rationed == 0`,
  `events == ()`; monotonic heat-generated; a hex-float golden pinned; registration-order
  independence.
- **Every electrical draw names its heat** (structurally — lossy flows are multi-leg, heat-booked).
- `git diff src/simcore/` limited to the **one sanctioned** `ASSERTED_QUANTITIES` flip (+ stale-
  comment cleanup); the **biosphere-frozen surface is untouched** (no unfreeze).
- Full suite green; ruff + pyright clean.
- **Thermal is COMPLETE** — a standalone sibling that passes conservation + determinism alone
  (emergent equilibrium temperature, `rationed == 0` by `τ >> dt` sizing, monotone contraction,
  RK4 ≢ Euler tolerance agreement, a hex-float golden). It builds the in-system receiver for Power's
  heat; **Phase 6 moves the `waste_heat` BOUNDARY inward** by rewiring Power's dissipation legs into
  `thermal.node` (the energy analogue of the water-cycle closure). **Atmosphere-ECLSS / Crew** remain
  forward-pointers, each designed just-in-time when reached. **Cross-domain coupling is Phase 6.**
