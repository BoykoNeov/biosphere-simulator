# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from commit 1;
biosphere is the first domain. Python is the canonical reference ("laboratory");
a Rust core + Godot front-end come much later. End goal: a science-credible Godot
station sim that runs the *same* simulation headless.

**Source of truth for current work:** Phases 0, 0.5, 1, and 2 are **complete**
(`docs/plans/phase-{0-engine-skeleton,0.5-numerical-foundations,1-single-producer,2-closed-chamber}.md`).
**Phase 3 — the subsystem hierarchy / multi-compartment structure — is COMPLETE (exits)**
(`docs/plans/phase-3-modular-biosphere.md` — Steps 1–7 COMPLETE (hierarchy representation +
reusable compartment builders: `season.py` split into `scenario`/`stocks`/`atmosphere`/`soil`/`plants`/`water`;
water cycle closed, P3.3 — `soil_water`→`water_vapor`→`condensate`→`soil_water`, sealed now
closed for all four quantities; P3.4 closure-preserving mortality + annual reset — `annual_reset`
driver transform + `PERENNIAL_CHAMBER_SCENARIO` → sustained multi-year oscillation, death routes
to litter not the loss-sink; P3.1 ledger discharge / Step 5 — per-compartment boundary ledger
balances every step/quantity/compartment on the perennial run, extinction exception via the
`expected_extinction_residuals` helper, all diagnostics-only with no behavior change;
P3.5 perturbation harness / Step 6 — `perturbations.py` composes drought/lighting-failure/
atmospheric-leak onto the assembled inputs outside `build_season`, each a cascade-for-free
with conservation + `rationed == 0` + per-compartment ledger balanced under the perturbed
resolver; zero core change, three goldens byte-identical, no new golden;
Step 7 minimal consumer — a fifth leaf `biosphere.consumers` + `herbivory.py` (first-order
`Grazing`/`ConsumerRespiration`/`ConsumerMortality`, the decomposer pattern one trophic level
up), `CONSUMER_CHAMBER_SCENARIO`, fourth golden; consumer persists, genuinely closed,
`rationed == 0`, leaf↓/CO₂↑ cascade, per-compartment ledger balanced incl. CONSUMERS; zero
core change, three producer-only goldens byte-identical);
**Phase 3 exits. Phase 4 — decade-scale stability + freeze-as-reference — is COMPLETE
(the biosphere is FROZEN AS THE REFERENCE; exits → Phase 5 sibling domains)**
(`docs/plans/phase-4-closed-biosphere.md`): **Steps 1–5 COMPLETE.** Step 1 (P4.1) —
`domains/biosphere/drift.py` (pure-stdlib drift instrument: `total_quantity` promoting the
`_total` fold + three axes — mass-drift ceiling/detector, `is_stationary`/`non_collapsing`
stationarity split, `is_period_2`) + `test_drift.py` + `test_decade_stability.py`; both closed
scenarios probed Euler **and** RK4 to 15 yr → **Euler LOCKED, with evidence** (drift jitters at
√N round-off; cycle bounded/non-amplifying/non-collapsing — perennial settles to a **period-2**
cycle, consumer to a **period-1 fixed point** (herbivore damps the producer oscillation, measured
not assumed); closure held; RK4 cross-check retired the preconditions & structurally agrees → Step
2 escalation skipped). Step 3 (100k-step stress) — `test_biosphere_stress.py` (marked-slow,
streaming-chunked, bit-identical to a continuous run): both closed scenarios Euler-daily to **328
yr (100,040 steps)**, the real slow-drift detector → **EULER HOLDS, NO DRIFT** (mass-drift slope
flat at machine-ε over the 22×-longer run, both detector bounds span both horizons; period class
sustained the full horizon — perennial period-2, consumer period-1; closure held every step);
zero core change, four Phase-3 goldens untouched. Step 4 (P4.2, golden capture) —
`test_regression_long_horizon.py` pins the closed biosphere at the **decade-scale horizon**
(`LONG_HORIZON_YEARS = 15`, new shared `scenario.py` constant): 15-yr perennial+consumer
final-`State` hex goldens + the **drift-summary golden** (per-year cycle summaries + period
class — the stability signature; mass-drift round-off deliberately NOT pinned, it's noise).
Pre-golden closure gate + load-back + `__main__` regen mirror the existing discipline; the
four Phase-3 goldens re-affirmed byte-identical (Step 2 skipped → no regen); zero core change.
Step 5 (P4.3, freeze contract) — `docs/biosphere-reference.md` (freeze contract +
**unfreeze discipline**) + `docs/biosphere-reference.manifest.json` (generated: locked
**Euler/`dt=1`**, `LONG_HORIZON_YEARS=15`, the **17-class flow set + the aux set, both
derived from assembled registries** — not hand-listed; gross assimilation is a recomputed
*quantity* in the carbon budget folded into `Allocation`, NOT an aux — the one aux is the
thermal-time/DVS accumulator, frozen via `aux_set`; the 13 clean-room param files + 7
scenarios→goldens, newline-normalized sha-256 **provenance**) + `tests/test_freeze_manifest.py`
(the **completeness** gate + a teeth test — frozen *sets* vs the live tree, NOT byte-rehash:
value enforcement stays with the scenario goldens, the gate owns only what they're blind to —
a param/flow/aux added but wired into no golden; rationale: every param feeds a golden so a
value change already moves one, and a raw hash is non-reproducible under `autocrlf`). Phase-0
demo (`flows.py`/`demo.yaml`) scoped out by name. Seven goldens byte-identical (no regen);
zero core change (`aux_processes` is a public read-only property). **Phase 4 EXITS.**
**Pre-Phase-5 "cheap middle" — two additive dormant-machinery scenarios — COMPLETE
(additive, NON-frozen; flushed the `f_N` + sealed-`f_water` limiter integrations):**
`N_LIMITED_SCENARIO` (open field — tiny fixed `plant_n0` + uptake off via `soil_n0` below
`sn_residual` → N-limitation **by dilution**, `f_N`↓ to ~0.55) and `WATER_BITING_SCENARIO`
(sealed — `soil_water0=50` inside the `(sw_wilting, sw_critical)` band → the closed water
cycle bites, `f_water`↓ to ~0.50, loop still conserved). Each: scenario *data* only (no new
flow/aux/param), reconstruct-the-factor behavioral test (`test_{n_limited,water_biting}.py`)
+ a cascade-vs-baseline isolating the limiter, a regression golden whose pre-golden gate
asserts the factor *actually bit* (a non-biting run can't re-pin it). The `soil_n_availability`
*middle* ramp stays an integrated never-run-hot path (can't co-bite arbitration-free with
dilution; unit-tested in `test_nitrogen.py`). Zero core change; **seven frozen goldens
byte-identical** (no regen) — proof the reference didn't move; not in the freeze manifest
(see `docs/biosphere-reference.md`).
**Phase 5 — sibling domains (power / thermal / atmosphere-ECLSS / crew), Power first — IN
PROGRESS** (`docs/plans/phase-5-sibling-domains.md`; plan advisor-reviewed). The load-bearing
decision is **energy closure (P5.1)**: `ENERGY` *joins the conserved set* (was balance-exempt
through Phase 4, decision #8). **Energy is ONE conserved quantity (joules); electricity vs
heat is a *form* distinction carried by the stock, NOT a separate `Quantity`** — a per-quantity
ledger can't balance a conversion (`battery→waste_heat`) across two of its quantities; "every
joule named" is the 3-leg lossy-flow pattern, "usefulness is not conserved" is the monotonic
heat-generated diagnostic (roadmap 46–51). Standalone Power dumps heat to a `boundary.waste_heat`
sink; **Thermal moves that boundary inward later** (the water-cycle-closure analogue for energy).
**Step 1 (P5.1a) COMPLETE — the isolated `ASSERTED_QUANTITIES` flip**: `frozenset(Quantity)`
now includes `ENERGY`; stale decision-#8 prose updated across `quantities.py`/`flow.py`/
`conservation.py`/`boundary.py`/`demo.py`; three inverse-teeth unit tests flipped (ENERGY
imbalance now *caught*). **Proven inert for existing runs** — the biosphere has no `ENERGY`
stock and the Phase-0 demo's `boundary.light` has a permanently-zero delta, so **all seven
frozen goldens + both demo goldens are byte-identical** (full suite green, 1035 passed). The one
sanctioned `simcore/` edit (a frozenset the roadmap always intended to flip here, line 318 —
**not** a biosphere-freeze violation; no frozen surface moved). A **Step-1 follow-up** commit
then swept the last stale decision-#8 prose (`flow.py` `per_quantity_residual` + the Phase-0
demo `flows.py`), comment-only, demo goldens byte-identical — so the Power feature commit's
`git diff src/simcore/` is **empty** (a Phase-5 exit criterion).
**Step 2 (P5.2 core) COMPLETE — the standalone Power flows (first carrier of energy closure)**:
new `src/domains/power/` package (`stocks.py`/`flows.py`/`loader.py`/`params/charge.yaml`), all
ENERGY (J), **zero core change**. Stocks: `power.battery` POOL + `boundary.solar_source`
(unclamped source) + `boundary.waste_heat` (monotonic sink). Flows: **`SolarCharge`** (3-leg,
heat-named — `solar_source → battery(+η_c) + waste_heat(+(1−η_c))`, **always 3 legs**;
η_c=1 → heat leg exactly 0) + **`LoadDraw`** (2-leg dissipative — `battery → waste_heat`, 100%
→ heat). Both **forced** (read `solar_power`/`load_power` W from `env`, ×dt → J — increment-form,
dt-linear); positivity is a **sizing discipline** (well-fed battery), not structural — brownout
is a later step. One param: `charge_efficiency` η_c ∈ (0,1] (the biosphere value/unit/source +
exact-string-unit-guard loader discipline). **`charge_efficiency` is ONE-WAY charge** — discharge
is joule-lossless (the discharge "loss" is *exergy*, tracked as the heat diagnostic), so the
**modeled round-trip = η_c** (0.95 = optimistic vs a real ~0.90 cell; discharge-side loss deferred).
18 per-flow tests (balance via `assert_flow_balanced` — the Step-2 gate; leg structure; dt-linearity;
zero-input no-op; loader bounds). **Capacity is NOT a param** (POOL stocks have no upper clamp →
sizing/scenario data, Step 3). Full suite green; **seven frozen + two demo goldens byte-identical**.
**Step 3 (P5.3) COMPLETE — the standalone run harness + bounded-SOC validation**: new
`scenario.py` (`PowerScenario`; `BOUNDED_SOC_SCENARIO` + `BOUNDED_SOC_DAYS=7`) + `system.py`
(`build_power` — three stocks + two flows, **no loss-sinks** as Power has no POPULATION stock;
`solar_schedule` — a half-sine over the daylight window, the weather-table analogue *computed*;
`power_resolver`; `run_power` — the `season.py` `run_season` analogue **minus** the reset hook),
**zero core change**. **The load is DERIVED for exact daily energy balance, not hand-tuned** — both
flows are *forced* (state-independent), so SOC is a restoring-force-free accumulator with **no
attractor**, and only **exact** daily balance is bounded (advisor physics call — option A over a
probe-sized constant, whose qualitative-regime precedent doesn't transfer to a balance condition):
`power_resolver` computes `load_w = load_fraction·η_c·(Σ_day solar)/steps_per_day` from the discrete
daily solar sum + loaded η_c (`load_fraction=1` ⇒ balance ⇒ bounded periodic SOC; `>1` = free
brownout knob; the **one** place a Power resolver reads a flow param — load is intrinsically
η_c-coupled). `dt=3600 s`, 24 steps/day. Probe: swing ≈ 72% of `battery0`, min-SOC 11.3× a step's
draw (`rationed==0` structural), day-over-day drift 1e-7 J. **14 validation tests** (the non-vacuous
payload): per-step ENERGY ledger residual ≈ 0 + integral total-ENERGY invariant; `rationed==0`;
`events==()`; **day-over-day SOC return** (true only under exact balance); material swing min>0;
interior morning-crossover minimum; monotonic `waste_heat`; balance identity; determinism; **RK4 ≡
Euler bit-for-bit** (forced ⇒ k1=k2=k3=k4, framed as the identity); registration-order independence.
Seven frozen + two demo goldens byte-identical (no regen).
**Step 4 (P5.4) COMPLETE — the hex-float Power golden capture**: `tests/test_regression_power.py` +
`tests/regression/golden/power_state.json` pin `BOUNDED_SOC_SCENARIO`'s 7-day final `State` via
`sim_io.dumps` (byte-match + load-back + separate `__main__` regen — the additive-`n_limited` golden
discipline; **additive, NON-frozen** — the Power domain's own regression pin, NOT in the biosphere
manifest). The **pre-golden gate** bakes Power's purpose in: `rationed == 0`, `events == ()`, the
**per-step ENERGY ledger balances** (residual ≤ 1e-6 — the energy-closure payload, the analogue of
the N-limited "`f_N` actually bit" gate: an imbalanced run is **unpinnable**), material SOC swing,
day-boundary return to `battery0` (Step-3's exact tolerances so legitimate 7-day round-off doesn't
fail regen). A day-boundary final state is deliberately blind to intra-day *shape* — that coverage
(interior minimum, half-sine, monotonic heat, RK4≡Euler) stays in `test_power_run.py` (the biosphere
pinned-State + separate-behavioral division; no drift-summary analogue needed). Within-build
bit-stability only (`math.sin` transcendental caveat). **Zero core change** (`git diff src/simcore/`
empty); full suite incl. `-m slow` + ruff + pyright green (**1069 passed**); **seven frozen + two
demo goldens byte-identical** (no regen). **The standalone Power domain (P5.2–P5.4) is now
complete.**
**P5.5 COMPLETE — `SelfDischarge`, the first donor-controlled Power flow (it earned its
keep)** (an add-on to the standalone Power domain, not a renumbered plan step): `battery →
waste_heat`, first-order `leak = k·battery·dt` (2-leg, ENERGY-balanced), an
**opt-in third flow** of `build_power` (`self_discharge_params: SelfDischargeParams | None = None`;
default `None` ⇒ the two-flow `BOUNDED_SOC` golden + RK4≡Euler bit-identity **untouched**). Reuses
`BOUNDED_SOC_SCENARIO` **verbatim** (`SELF_DISCHARGE_DAYS=14`) so the leak is the *sole* driver of
departure from the daily-balanced baseline (which returns to `battery0`; the leaky run monotone-
decays below it). New `params/self_discharge.yaml` (`self_discharge_rate` k, unit `1/s`, `k ≥ 0`,
realistic Li-ion `1e-8/s` ≈ 2.6 %/month — **NOT inflated**; loader reuses the generic
`_ValueUnitSource`). **Why it earns its keep** (the plan gates SelfDischarge on this): unlike the
two *forced* flows it reads a **stock**, so (1) `rationed==0` is structural for *its own* leg
(`k·dt<1`; LoadDraw still leans on sizing — not overclaimed), (2) it is the domain's **first
restoring force** → a stable SOC attractor, proved **magnitude-independently** by a two-run
**contraction test** (`d_n = d_0·(1−k·dt)^n` exactly to fp tol; forced-only keeps `d_n` constant —
the clean distinguisher), and (3) it **breaks** the forced-only RK4≡Euler bit-identity (now a
tolerance agreement). Energy still closed every step (the leak is `−leak+leak=0`); heat still
monotonic. Tests: SelfDischarge flow unit tests + self-discharge loader in `test_power_flows.py`;
behavioral `test_power_self_discharge.py` (contraction, forced-only contrast, baseline isolation,
closure, broken bit-identity); additive **non-frozen** golden `test_regression_power_self_discharge`
+ `power_self_discharge_state.json` (pre-golden gate: `rationed==0`, ENERGY closed, SOC *departs*
`battery0` — the "it bit" check; **not** the two-flow golden's "returns to `battery0`"). **Zero core
change** (`git diff src/simcore/` empty); full suite incl. `-m slow` + ruff + pyright green
(**1093 passed**); **seven frozen + two demo + the two-flow Power golden byte-identical** (no regen).
**Step 5 (Thermal) COMPLETE — the second standalone sibling; the first nonlinear attractor + the
receiver Phase 6 moves Power's `waste_heat` into**: new `src/domains/thermal/` (zero core change) —
`thermal.node` POOL (sensible heat J, **referenced to `T_space`** so `Q = C·(T − T_space) ≥ 0`) +
`boundary.heat_source` (unclamped) + `boundary.space` (monotonic sink). Flows: **`HeatInput`**
(2-leg forced, heat→heat lossless — no charge-loss leg unlike `SolarCharge`) + **`RadiatorReject`**
(2-leg **donor-controlled nonlinear** Stefan-Boltzmann `R = εσA(T⁴ − T_space⁴)·dt`). `radiator.yaml`
(ε / radiator_area / heat_capacity / space_temperature, exact-string K-guarded; **σ is a CODATA
module constant, NOT a param** — a universal physical constant with provenance, the `drift.py`
discipline). Genuinely-new machinery over Power: **temperature** (`T = T_space + Q/C`, a derived
evaluate-time readout — **not** a stock/aux/ledger entry), **heat capacity**, and a **nonlinear
restoring force** → a *real* **emergent equilibrium temperature** `T_eq ≈ 280.9 K` (a genuine
attractor, vs Power's *constructed* daily balance). **The load-bearing constraint (advisor): T⁴
trades `SelfDischarge`'s structural `k·dt<1` positivity for sizing-dependent positivity** — structural
at the floor (`R → 0` as `Q → 0`, the `T_space` reference) + `τ = C/(4εσA·T_eq³) >> dt` near
equilibrium (`τ ≈ 65` steps; `rationed == 0` the `LoadDraw` way, **not** structural). Resolver is a
plain constant `heat_load` — **no** `balanced_load_w` (the radiator IS the balance). 39 tests (22
flow + 15 run + 2 golden): ENERGY conserved every step, `rationed == 0`, `events == ()`, monotonic
`space` sink, **T converges to `T_eq`**, **two-run monotone contraction** (nonlinear, not geometric;
no-radiator contrast keeps the difference constant), **RK4 ≢ Euler** (tolerance agreement),
determinism, registration-order independence; additive **NON-frozen** golden `thermal_state.json`
(pre-golden gate: ENERGY closed / `rationed == 0` / **reached equilibrium**). **`boundary.space` is a
permanent, true boundary — standalone Thermal closes nothing** (heat leaves to deep space forever,
unlike Phase-3 water); **Phase 6 rewires Power's dissipation legs to feed `thermal.node`** (the inward
move — standalone builds the receiver). **Zero core change** (`git diff src/simcore/` empty); full
suite incl. `-m slow` + ruff + pyright green (**1132 passed**); **seven frozen + two demo + two Power
goldens byte-identical** (no regen).
**Step 6 (Atmosphere/ECLSS) COMPLETE — the third standalone sibling; the first multi-quantity one;
the cabin-air receiver Phase 6 wires Crew + biosphere into**: new `src/domains/eclss/` (zero core
change) — three **single-quantity** cabin POOLs `eclss.cabin_o2`/`cabin_co2`/`cabin_h2o` (OXYGEN/
CARBON/WATER — the first sibling touching >1 conserved quantity; all three already-asserted mass
quantities, so **no new core decision**) + six boundary reservoirs (`o2_supply`, `co2_removed`,
`humidity_condensate`, and the three `metabolic_*` crew-seam reservoirs). Flows: **`CrewMetabolism`**
(the forced **multi-quantity** crew/Phase-6 seam — one flow, six legs across three quantities, each
balanced independently: O₂ out of cabin, CO₂/H₂O into cabin) + three ECLSS control loops —
**`CO2Scrubber`**/**`Condenser`** (first-order donor-controlled, `SelfDischarge` pattern, structural
`k·dt<1`) + **`O2Makeup`** (**demand-controlled** toward a setpoint `k·(o2_setpoint−cabin_o2)·dt` —
the advisor's fix so O₂ doesn't recapitulate Power's constructed-balance problem; a restoring force
with **no readout**). `eclss.yaml` (3 rates 1/s + `o2_setpoint` mol; exact-string guarded;
**illustrative `TODO(cite)`, deliberately NOT NASA BVAD/BioSim** — calibration is Phase 6). **Scope
discipline (advisor):** pressure readouts / N₂ diluent / composition stocks (CO₂={CARBON:1,OXYGEN:2})
are **deferred seams** — no standalone flow needs them. **The payload is the 3-quantity every-step
gate** (the "it bit" check, the ENERGY-closed analogue). **Honest novelty:** linear ⇒ **geometric**
contraction (reuse `d_n=d_0·(1−k·dt)^n`, per species — **not** Thermal's nonlinear one); RK4 ≢ Euler
(tolerance agreement). Each species reaches an emergent steady state (`co2_eq=P/k_scrub`,
`h2o_eq=P/k_cond`, `o2_eq=o2_setpoint−Con/k_makeup` → 3.0 mol / 0.04 kg / 8.0 mol). **The
atom-conservation seam** (honest analogue of Thermal's permanent `boundary.space`): each quantity
balances over its augmented system, but the crew seam does **not** tie the atoms together (real
respiration binds inhaled O₂ into exhaled CO₂/H₂O) — three decoupled boundary reservoirs, closed by
**Phase-6 crew coupling + composition stocks**. 41 tests (23 flow + 16 run + 2 golden) incl. the
per-species geometric-contraction + no-control contrast; additive **NON-frozen** golden
`eclss_state.json` (pre-golden gate: 3-quantity closure / `rationed==0` / reached steady state).
**Zero core change** (`git diff src/simcore/` empty); full suite incl. `-m slow` + ruff + pyright
green (**1173 passed**); **seven frozen + two demo + two Power + one Thermal golden byte-identical**
(no regen).
**Step 7 (Crew) COMPLETE — the fourth and last sibling; the first net-consumer / open-loop one;
PHASE 5 EXITS**: new `src/domains/crew/` (zero core change) — three **finite provisioned-store**
POOLs `crew.food_store` (CARBON) / `crew.water_store` (WATER) / `crew.o2_store` (OXYGEN) drawn down
by three **forced** metabolic flows + five monotonic boundary output sinks. Flows: **`OxygenConsumption`**
(2-leg, `o2_store → crew_o2_consumed`) + **`FoodMetabolism`** (3-leg **split** `food_store → exhaled_co2
(f_resp) + fecal_waste (1−f_resp)`) + **`WaterBalance`** (3-leg **split** `water_store → crew_humidity
(f_ins) + urine (1−f_ins)`) — the two splits are `SolarCharge`'s η-split applied to a **mass** quantity
(`carbon_split`/`water_split` mirror `charge_split`). `crew.yaml` (two dimensionless split fractions,
exact-string guarded, illustrative `TODO(cite)` — NOT NASA BVAD; the intake **rates** are scenario data).
**The load-bearing framing (advisor): Crew is the first NET-CONSUMER / open-loop sibling** — all flows
**forced** (read rates, never a store), so no restoring force / no attractor; the stores just **run down**
(`store(n)=store0−n·rate·dt`), and *that incompleteness is the argument for Phase-6 closure*. **The
splits are justified because each output routes to a DIFFERENT Phase-6 destination** (CO₂→cabin-air vs
feces→solid-waste; humidity→cabin-air vs urine→water-recovery), NOT "the first mass split". **Crew is
the real version of ECLSS's forced `CrewMetabolism` stand-in** — Phase 6 deletes that stand-in and wires
Crew's outputs into the cabin (CO₂→`cabin_co2`, humidity→`cabin_h2o`, O₂←`cabin_o2`, a *subset*; urine/
feces/O₂-consumed route elsewhere). Multi-quantity like ECLSS (CARBON/OXYGEN/WATER conserved every step
— the payload); positivity by **well-fed sizing** (`rationed==0` because each store's endurance
`store0/rate` exceeds the mission — the `LoadDraw` way, never a store-availability clamp); **forced ⇒ RK4
≡ Euler bit-identical REVIVED** (the symmetric bookend to ECLSS/Thermal, which broke it — framed as the
identity). Integral invariant cleaner than ECLSS (the store holds the inventory → **carbon total==food0**,
no negative-going source). No POPULATION stock (crew count is scenario data) ⇒ `events==()`; the atom-level
stoichiometry (`C_food+O₂→CO₂+H₂O`) + composition stocks are **deferred seams**. Validation
(`MISSION_SCENARIO`, 7-day provisioned mission, `dt=3600 s`, 168 steps, each store → ≈70%): 3-quantity
every-step closure, `rationed==0`, `events==()`, monotone depletion + closed-form `depletion_times`,
monotonic sinks, RK4 ≡ Euler bit-identity, determinism, registration-order independence; additive
**NON-frozen** golden `crew_state.json` (pre-golden gate: 3-quantity closure / `rationed==0` / **material
depletion** — the "it bit" check). **32 tests** (18 flow + 12 run + 2 golden). **Zero core
change** (`git diff src/simcore/` empty); full suite incl. `-m slow` + ruff + pyright green (**1205
passed**, 1 oracle skip); **all thirteen existing goldens byte-identical** (seven frozen + two demo + two
Power + one Thermal + one ECLSS; no regen). **Phase 5 EXITS → Phase 6 (station integration / cross-domain
coupling).**
**Phase 6 — station integration / cross-domain coupling — IN PROGRESS**
(`docs/plans/phase-6-station-integration.md`; plan advisor-reviewed). **Step 1 (P6.1) COMPLETE — the
`src/station/` assembly layer, proven on Power → Thermal heat closure**: new `src/station/`
(`scenario.py`/`system.py` — the layer that imports both siblings and owns the wiring; **no domain imports
another**). The seam: Power's dissipation legs redirected from `boundary.waste_heat` into `thermal.node` by
passing `thermal.node`'s id where `SolarCharge`/`LoadDraw` took `waste_heat`; Thermal's forced `HeatInput`
stand-in dropped (Power's dissipation *is* the input now) ⇒ `boundary.waste_heat`/`boundary.heat_source`
**absent** from the station state (the redirection is structural, not a shadow sink); `RadiatorReject` rejects
the **real** load to deep space. `build_station`/`station_resolver`/`run_station` — the harness every later
step reuses (`station_resolver` == `power_resolver`; multi-resolver merging deferred). Single-quantity
(ENERGY): combined ledger balances every step over `solar_source+battery+node+space` (the payload). **Node's
initial heat DERIVED from Power's actual dissipation** (`equilibrium_node_heat`→`mean_dissipated_power`→reused
Thermal `equilibrium_temperature`; all solar → heat in daily balance ⇒ mean ≈316 W ⇒ `T_eq≈160.1 K`), not a
hand-set `heat_load`. **The two-start convergence test is the non-circular core** (advisor): two `node0`
(`0.5/1.5·Q_eq`) under identical Power forcing contract to one band over ~3 τ (τ≈14.6 d) — radiator alone
governs the difference (no-radiator contrast keeps it constant) ⇒ equilibrium set by dissipation *independent
of IC* (start-at-`Q_eq` alone only shows stability). Node band within ~1 K of the **mean-power** `T_eq` (true
attractor slightly below by the T⁴-convexity offset — honest, not pinned exact). Corroboration:
`battery`+`solar_source` **bit-identical to standalone Power** (coupling is pure sink re-wiring — donor
unperturbed, verified step-by-step); per-day `ΔSpace` ≈ Power's per-day heat gen (real load, quantitatively);
RK4≢Euler on the nonlinear node but bit-identical on the forced battery; determinism; registration-order
independence. **16 tests** (14 run + 2 golden); additive **NON-frozen** golden `station_state.json`
(pre-golden gate: `rationed==0`/`events==()`/combined ENERGY closed every step/no shadow sink/node at the
dissipation-set equilibrium). **Zero core change** (`git diff src/simcore/` empty) + **zero domain change**
(`src/domains/` untouched); full suite incl. `-m slow` + ruff + pyright green (**1221 passed**); **all
fourteen existing goldens byte-identical** (seven frozen + two demo + two Power + one Thermal + one ECLSS +
one Crew; no regen).
**Step 2 (P6.2) COMPLETE — the Crew ↔ ECLSS cabin gas loop; OXYGEN closes via composition CO₂ + a merged
respiration flow**: new `src/station/cabin.py` (second assembly) + `src/station/flows.py` (`CrewRespiration`,
the first station-owned flow) + a `CabinScenario`. The seam drops ECLSS's forced `CrewMetabolism` stand-in +
its `metabolic_*` reservoirs and the crew `o2_store`/`OxygenConsumption`; the real crew breathes cabin air via
the merged **`CrewRespiration`** (`food_store + cabin_o2 → cabin_co2 + fecal_waste` — the `MicrobialRespiration`
PQ=1 template, **forced**, 4-leg, O₂-leg magnitude = `respired` since only metabolized carbon draws O₂) + crew
`WaterBalance` (`water_store → cabin_h2o + urine`); ECLSS's `CO2Scrubber`/`Condenser`/`O2Makeup` carry over. **Every
CO₂ stock in the loop is composition `{C:1,O:2}`** (`cabin_co2` AND `co2_removed`), every O₂ stock `{O:2}`
(`cabin_o2` AND `o2_supply`) — built **inline in the station** (the `boundary`/`eclss` constructors take no
composition arg; extending them = core change), **zero core change**. **Two non-vacuous gates** (per-quantity
balance is trivial): (1) the **decoupled** (pure-carbon `cabin_co2`) build raises `ConservationError` for OXYGEN
on the **first step** (balance is evaluation-time — a one-step run, not construction) — composition is
load-bearing, the "it bit" gate; (2) **O₂ genuinely drawn from the cabin** — `cabin_o2` starts at the setpoint,
pulled below to `o2_eq = setpoint − f_resp·food/k_makeup` (8.3 mol). **RQ=1 baked in by PQ=1** (O₂ consumption =
CO₂ production in one flow; realistic RQ≈0.75 needs metabolic-water machinery — deferred, matching the biosphere).
**Closure is augmented/atom-conservation sense, NOT a closed cycle** (O₂ still from `o2_supply`, CO₂ still to
`co2_removed` — the `boundary.space` analogue; the recycled cycle is Step 3). WATER stays decoupled (metabolic
water ignored — scope boundary). Cabin reaches emergent steady states (`cabin_steady_state`); crew **stores run
down** (forced, open-loop — argument for Steps 4/6), well-fed (`rationed==0`). Forced stores **RK4≡Euler
bit-identical**, state-dependent cabin species **RK4≢Euler** (mid-transient). **dt=60 s** (ECLSS's binding
`k·dt<1`); reuses `crew.yaml`+`eclss.yaml` verbatim. **16 tests** (14 run + 2 golden); additive **NON-frozen**
golden `cabin_gas_state.json` (pre-golden gate: 3-quantity closure / `rationed==0` / O₂ below setpoint / reached
steady state). **Zero core change** (`git diff src/simcore/` empty) + **zero domain change** (`src/domains/`
untouched; `CabinScenario` additive in `station/scenario.py`); full suite incl. `-m slow` + ruff + pyright green
(**1237 passed**); **all fifteen existing goldens byte-identical** (seven frozen + two demo + two Power + one
Thermal + one ECLSS + one Crew + the Step-1 station; no regen).
**Step 3 (P6.3) COMPLETE — the biosphere ↔ cabin greenhouse; the emergent crew↔plant CO₂/O₂ feedback; plants
offload life support via a net-fixation conservation identity**: new `src/station/greenhouse.py` (third assembly)
+ `GreenhouseScenario`. **The seam is REVERSED from the plan's first framing** (advisor-reviewed): the naive
"point the biosphere's `ChamberWiring` at the cabin's CO₂/O₂ ids" is **blocked** — only `plants` consumes the
wiring, while `soil.MicrobialRespiration` (built for EVERY sealed chamber) + `consumers.ConsumerRespiration` read
`CARBON_POOL`/`O2_POOL` from the **catalog, hardcoded**, so re-pointing the wiring redirects only plant gas.
Instead **keep the biosphere's `CARBON_POOL` (`{C:1,O:2}`) / `O2_POOL` (`{O:2}`) as the shared cabin air and
re-point the CABIN's five all-parameterised flows** (`CrewRespiration`/`CO2Scrubber`/`O2Makeup`/`Condenser`/
`WaterBalance`) at those ids — re-point the side that CAN be. Reuses `build_season(sealed)` **wholesale**
(build_atmosphere included, CARBON loss-sink included, default `sealed` wiring + default `{CO2_POOL_VAR:
CARBON_POOL}` Ci map all unchanged); physically correct (plants + microbes + crew breathe one cabin-air stock).
**A bespoke two-rate master-step driver, NOT `simcore.multirate`** (advisor-reviewed): the biosphere is
structurally `dt=1` **day** (weather indexed by `n`), the cabin `dt=60 s` **per second** (`k_scrub·dt<1`) — two
different time UNITS, which `multirate_step` can't bridge (one shared master `dt` split as `dt/n_sub`) AND it
composes `substep` only, which by design freezes the biosphere's `thermal_time` aux (phenology). The driver does
the operator split by hand: per day, cabin `substep(dt=60)`×1440 (keeps `n`, conservation asserted after EACH
substep — the every-step teeth) then biosphere `step_report(dt=1)`×1 (advances aux AND `n`, so `n` stays the day
count and the frozen `weather_resolver` is reused unchanged). Two disjoint registries over one shared stock dict
+ two integrators; all public methods ⇒ zero core change. **The payload is a net-fixation CONSERVATION IDENTITY,
not a cabin-pool shift** (advisor-reviewed, empirical): the fast scrubber (τ≈1000 s, 86 τ/day) fully relaxes
`CARBON_POOL`/`O2_POOL` back to their regulator setpoints between the once-daily biosphere lumps, so the
regulated pools are IDENTICAL (to fp) at every day boundary — the plant's effect is *erased from the pool* and
*conserved into* (a) biosphere biomass and (b) reduced ECLSS work. So the "it bit" gate is: the plant fixes net
carbon (`bio_organic_C` grows), the scrubber removes LESS CO₂ (`co2_removed_with < co2_removed_no`), the makeup
supplies LESS O₂ (`o2_supply_with > o2_supply_no`), and the three agree to tolerance (`Δco2_removed ≈ bio_gain ≈
Δo2_supply`, RQ=1; cancellation floor ~1e-10 ⇒ `atol=1e-8`, not bit-exact; booleans carry the sign, an un-biting
net-source run flips them). Step 2's composition-failure gate does NOT re-run (`{C:1,O:2}` is frozen inside
`build_atmosphere`; no new composition requirement). Illustrative scale (crew ~3400× the 1 m² seedling);
calibration deferred to Step 9. The biosphere is **Euler-locked by its freeze** ⇒ an Euler run (no RK4
cross-check). Crew stores re-sized for the multi-DAY horizon (draw is `rate·time`, dt-independent; `rationed==0`
by well-fed sizing); biosphere internal water + N loops still close (not coupled to the cabin — Steps 4/6). 11
tests (9 run + 2 golden); additive **NON-frozen** golden `greenhouse_state.json` (pre-golden gate: `rationed==0`
/ `events==()` / every quantity closed every master day / plant fixes net carbon). **Zero core change** (`git
diff src/simcore/` empty) + **zero domain change** (`src/domains/` untouched; `GreenhouseScenario` additive in
`station/scenario.py`); full suite incl. `-m slow` + ruff + pyright green (**1248 passed**); **all sixteen
existing goldens byte-identical** (seven frozen + two demo + two Power + one Thermal + one ECLSS + one Crew + the
Step-1 station + the Step-2 cabin-gas; no regen).
**Step 4 (P6.4) COMPLETE — the crew water-recovery loop; the crew's finite `water_store` becomes REGENERATIVE;
built on the CABIN, not the greenhouse**: new `src/station/water.py` + the **first station-owned params**
(`src/station/params/water_recovery.yaml` + `src/station/loader.py`). The seam re-points the Step-2 cabin's two
WATER disposal sinks (`humidity_condensate` / `urine`) into a new `recovered_water` buffer POOL (the crew analogue
of the biosphere's `condensate` — the ECLSS `Condenser` product + the crew urine collect there), and a
station-owned **`WaterRecovery`** flow (`recovered_water → water_store (+η_w) + brine (+(1−η_w))`, 3-leg, the
`SolarCharge`/`carbon_split` η-split on WATER, **donor-controlled** `k_rec`) returns the recovered fraction to
`crew.water_store`, venting only the unrecoverable remainder to a `brine` sink. So the store's net drain drops
from the full intake to `(1−η_w)·intake` — **regenerative up to the recovery efficiency**, fully closed only at
η_w = 1 (`brine` the honest remaining WATER boundary, the Thermal `boundary.space` analogue). `water_recovery.yaml`
(`recovery_rate` k_rec 1/s ≥ 0 structural `k·dt<1`; `recovery_efficiency` η_w dimensionless ∈ [0,1]; exact-string
guarded, illustrative `TODO(cite)` — NOT NASA/ISS numbers). **Zero domain / zero core change** (assembly-level id
re-pointing — the `Condenser`/`WaterBalance` flow classes untouched; a buffer pool + a new flow, NOT a split at
the condenser, which would be a domain change). **Scope decision (advisor): closure ≠ humidity unification** —
the plan's "crew humidity + **biosphere transpiration** → cabin_h2o" over-reached; coupling biosphere
transpiration into the cabin is a **fidelity refinement, NOT a closure requirement**, and is **deferred** (Step 7).
The biosphere's internal water ring is **already closed/sealed independently**
(`test_biosphere_internal_water_loop_closed`); the crew loop closes independently the moment recovery is added —
so station WATER conserves as (closed biosphere ring) + (crew loop closed up to brine). Built on the **cabin** (not
the greenhouse) so the biosphere — **Euler-locked by its freeze** — stays out of the assembly and the **RK4 ≢
Euler cross-check runs**: recovery makes `water_store` **state-dependent** (inflow ∝ the buffer level), **breaking**
the forced RK4 ≡ Euler bit-identity the cabin stores had (the "it earned its keep" signal, the `SelfDischarge`
analogue), while the forced `food_store` stays bit-identical. **The payload is a conservation identity** (the
Step-3 offload analogue): the `recovered_water` dynamics + the forced intake are both **independent of η_w** (η_w
only splits the *output*), so the water returned to the store equals **exactly** η_w × the water the open-loop
(η_w=0) baseline sends to `brine` — `water_store_with − water_store_without ≈ η_w·brine_without` (~1e-13). The "it
bit" gate is with-vs-without recovery (η_w=0 reproduces the open-loop drain, same topology) + the identity; the two
WATER pools reach emergent steady states (`cabin_h2o → f_ins·intake/k_cond`, `recovered_water → intake/k_rec`);
WATER's total is invariant (`brine` the only terminal WATER sink); `rationed==0` (structural + well-fed);
`events==()`. **17 tests** (15 run + 2 golden, the run set incl. the pre-golden gate); additive **NON-frozen**
golden `water_recovery_state.json` (pre-golden gate: 3-quantity closed every step / `rationed==0` / `water_store`
regenerated above the η_w=0 baseline — the "it bit" check / reached WATER steady states). **Zero core change**
(`git diff src/simcore/` empty) + **zero domain change** (`src/domains/` untouched); full suite incl. `-m slow` +
ruff + pyright green (**1265 passed**, 1 oracle skip); **all seventeen existing goldens byte-identical** (seven
frozen + two demo + two Power + one Thermal + one ECLSS + one Crew + the Step-1 station + the Step-2 cabin-gas + the
Step-3 greenhouse; no regen — `water_recovery_state.json` is the eighteenth).
**Step 5 (P6.5) COMPLETE — Power → biosphere lighting; the phase's one NON-shared-stock coupling; energy
enters biology**: new `src/station/lighting.py` + the station-owned **`Lamp`** flow (`station/flows.py`) +
`lamp.yaml` (second station-owned param). The seam: a grow lamp `power.battery → light_used + waste_heat`
(3-leg SolarCharge η-split, **forced**) whose electrical draw ALSO sets the biosphere's `par` **forcing**
(`PAR = photon_efficacy·lamp_power_w/ground_area`), replacing the weather-table PAR. **Power and the
biosphere share NO stock** (finding #3 / #16) — the lamp-draw schedule is the whole interface, feeding both
the ENERGY ledger (this flow) and the PAR forcing (a value the frozen biosphere reads; a flow can't tell
forcing from a shared stock). **The daylength coupling is the correctness crux** (advisor): `incident_par`
returns a daytime-mean flux and FvCB re-multiplies by `daylength_s` (dose = PAR × daylength), so **both**
`PAR_VAR` and `DAYLENGTH_VAR` come from the lamp (`daylength_s = photoperiod_hours·3600`) — verified the ONLY
runtime `daylength_s` consumer is photosynthesis. **Scope (advisor-endorsed deviation): Power + biosphere
only; the `waste_heat` leg → `boundary.waste_heat`, NOT `thermal.node`** (the plan's parenthetical would only
re-test Step-1's node seam; inward move deferred to the sealed-station step — "boundary now, inward later").
**One lamp param, the ENERGY split DERIVED**: `lamp.yaml` carries only `photon_efficacy` (µmol/J, illustrative
2.5, Kusuma/Bugbee 2020); `η_lamp = photon_efficacy/PAR_UMOL_PER_J` via the inverse of the biosphere's own
McCree constant (a σ/CODATA-style module constant, not a param) — efficacy and radiant fraction are two
accountings of ONE device, consistent by derivation; loader guards `∈ (0, PAR_UMOL_PER_J]` (η_lamp = 1
ceiling). **The frozen-`n` fast domain forces a daily-average lamp draw** (`substep` keeps `n`, so a within-day
top-hat isn't an `n`-schedule; the biosphere carries the photoperiod internally via `daylength_s`, so Power
draws the constant `lamp_power_w·photoperiod/24` — daily energy exact, only intra-day instantaneous power
smeared; PAR uses the on-window intensity). **The two-rate driver is EXTRACTED** to `src/station/driver.py`
(`run_master_day` generalizes `run_greenhouse`'s body — slow domain once/day via `step_report`, fast domain
×`steps_per_day` via `substep` + per-substep conservation assert; `run_greenhouse` refactored to a thin
wrapper, its golden byte-identical; `run_lighting` the second instance). **Minimal Power** (battery POOL + Lamp
only — no SolarCharge/LoadDraw; the battery is a finite energy store draining, the Crew-store pattern).
**Payload — the signed "it bit" gate (Euler-only, biosphere frozen)**: lamp-on ⇒ `bio_organic_C` grows (+0.11
mol/7 days), lamp-off (PAR = 0) ⇒ declines (respiration only); PAR factor reconstructed; ENERGY closed every
step (battery drains by exactly `lamp_power_w·photoperiod·3600·days`; `light_used`/`waste_heat` name the
η-split); biosphere internal water/N loops still close; `rationed == 0`, `events == ()`; Power⊥biosphere stock
sets disjoint. For Step 8: a schedule-derived PAR won't see brownout rationing automatically (flagged). **26
tests** (unit: `lamp_energy_split`/`Lamp` legs+balance/loader bounds; run: the gate + determinism); additive
**NON-frozen** golden `lighting_state.json` (pre-golden gate: every quantity closed / battery drained by the
lamp energy / lit grew while dark declined). **Zero core change** (`git diff src/simcore/` empty) + **zero
domain change** (`src/domains/` untouched); full suite incl. `-m slow` + ruff + pyright green (**1291 passed**,
1 oracle skip); **all eighteen existing goldens byte-identical** (seven frozen + two demo + two Power + one
Thermal + one ECLSS + one Crew + Step-1 station + Step-2 cabin-gas + Step-3 greenhouse + Step-4 water-recovery;
no regen — `lighting_state.json` is the nineteenth).
**Step 6 (P6.6) COMPLETE — the biomass/food loop; the trophic CARBON ring closes; crew food becomes
regenerative** (landed as TWO advisor-flagged increments/commits over the Step-3 greenhouse). New
`src/station/harvest.py` + `station/flows.py:Harvest`+`HarvestParams` + `params/harvest.yaml` +
`loader.py:load_harvest_params` + `scenario.py:HarvestScenario`; **zero core + zero domain change**
(`git diff src/{simcore,domains}/` empty). **SEAM 1 — the `Harvest` flow** (`storage_c → food_store`,
donor-controlled `k·storage_c·dt`, 2-leg single-currency `{CARBON:1}` transfer, in the cabin/fast
registry): the CARBON twin of Step-4 `WaterRecovery`, one trophic level — the biosphere's grain drains
into the crew's finite `food_store`, so the open-loop store becomes **regenerative** up to the harvest.
**The reproductive-plant precondition** (`storage_c` fills only post-anthesis, `FO>0` needs `DVS>1`):
`HarvestScenario.thermal_time0=1300` (DVS 1.27) starts the biosphere phenology past anthesis via a
**station-level `State`-aux injection** in `build_harvest` (the station owns the greenhouse State's aux
dict ⇒ `SeasonScenario` untouched, zero domain change). Go/no-go grain-fill spike + coupled `k_harvest`
probe (`harvest_rate=1e-5`/s, `k·dt=6e-4`) passed the recommended path: grain settles to a **positive
quasi-steady** (day-boundary min ~7e-4–1.4e-3 mol, ~89% of the ~1.3e-2 mol/7-day fill captured). Payload
= the **two-way identity** `Δfood_store = Δstorage_c = cumulative harvest` to ~1.8e-9 (the ~1580-mol
food-store cancellation floor; signal 7 orders above) — **exact because grain fill is identical
with/without harvest** (verified before building, advisor-flagged: only `annual_reset` reads `storage_c`
and doesn't fire ≤7d; `Allocation`'s `FO·DMI` is independent of `storage_c`'s level; `CrewRespiration`
is forced; `Harvest` touches neither `CARBON_POOL` nor a photosynthesis input). Driver is **slow-first**
(biosphere refills grain, then 1440 cabin substeps drain it ⇒ day-boundary snapshot is the intra-day
*minimum*). **SEAM 2 — feces → litter re-point** (`close_feces`, closes the ring feces → litter →
microbes → CO₂): wired via a default-preserving `fecal_waste_target: StockId = FECAL_WASTE` param on
`build_greenhouse`/`_cabin_flows` (drops the `FECAL_WASTE` sink when re-pointed — station-layer change,
zero domain/core) + a `close_feces: bool` knob on `build_harvest` (default `True`). A **regime spike
(advisor-flagged) INVERTED the design's `Δlitter ≈ feces` identity**: the design's ~3400× domination
premise is right (feces ≈363 mol vs seedling litter 0.0135), but at `x_O2 ≈ 10/9500` the microbes are
**active, not throttled** (litter→~342 mol, microbial biomass→~20 mol, consuming ~21 mol) ⇒ the seam-2
gate is **per-quantity closure + `FECAL_WASTE` sink absent + litter-grows-materially (with/without) +
`rationed==0` + `events==()`**, NOT a three-way identity. Both `fecal_waste`/`litter_carbon` verified
pure `{CARBON:1}`. Finding: closing feces perturbs grain/food only at **fp round-off** (rel ~1e-15) —
microbial CO₂ enters the shared `CARBON_POOL` but the scrubber holds it at setpoint (Step-3
regulator-erasure), so the seams are near-orthogonal, not bit-identical. The ~3400× magnitude mismatch =
a **direction+conservation demo, not a self-sufficient loop** (crew draws ~345 mol C/day vs plant fills
~1.6e-3/day; food still net-depletes; calibration deferred to Step 9). Euler-only (biosphere frozen).
Additive **NON-frozen** golden `harvest_state.json` (closed ring; regenerated in seam 2). **33 tests**
(12 run + 2 golden after seam 1; +3 seam-2 run tests). Full suite incl. `-m slow` + ruff + pyright green
(**1305 passed**, 1 oracle skip); **all nineteen pre-Step-6 goldens byte-identical** (`harvest_state.json`
is the twentieth).
**Step 7 (P6.7) COMPLETE — the sealed station: multi-year matter + energy stability (the Phase-4
analogue at station scale); PHASE 6's stability capstone**: new `src/station/sealed.py`
(`build_sealed_station` composing **every** Phase-6 seam over one shared stock dict + two registries
— biosphere-slow + everything-fast ~11 flows, waste-heat legs → `thermal.node` the Step-1 inward
move) + `SealedStationScenario` + a `slow_reset` hook added to `station.driver.run_master_day` (the
`annual_reset` re-sow machinery the ≤7-day greenhouse/lighting/harvest runs never fired — a plumbing
gap, **not** a physics finding; the discriminator advisor-flagged) + shared
`tests/sealed_tier2_helper.py` + a session-scoped `sealed_tier2_run` conftest fixture. **Zero core +
zero domain change** (`git diff src/{simcore,domains}/` empty; the driver/scenario edits are
station-layer). **The thesis splits in two (never conflated):** (A) integration + longevity — the
NEW thing — the full assembly *sustains* every-quantity + ENERGY conservation to round-off over many
annual cycles; (B) physical stationarity is a per-subsystem *characterization*, not a whole-station
claim (energy earns a real attractor; matter earns conservation + regulated-pool stationarity + the
period-1 plant; whole-system matter stationarity **deferred** — stores drain, feces open). **The
load-bearing spike PASSED:** the coupled biosphere under **pinned CO₂** (scrubber-held Ci≈258, the
regime the freeze never validated) is **period-1** (grain-at-re-sow byte-identical every year — the
constant Ci removes the CO₂-pool feedback that drove Phase-4's period-2) with a **converging**
decomposer pool (peak total-organic-C 29.10→29.196→29.196, diffs shrinking ~450×), re-sows cleanly,
`rationed==0`. **Scope calls (spike-measured, advisor-endorsed):** harvest **DROPPED from Tier 2**
(drains `storage_c` to 0.011 < the 0.16 seed bank → starves the re-sow; its food-loop conservation
is pinned in Step 6); `close_feces=False` (the litter loop is the one *unregulated* one). Power runs
**constant daily-average** solar/load in the fast lane (`substep` freezes `n` ⇒ the diurnal shape is
inexpressible — the Step-5 lamp-average precedent; the diurnal SOC swing + node attractor are Tier
1's job). **Two advisor-flagged test-design fixes:** (1) `drift.py`'s *absolute* bounds do NOT
transfer (station stocks 1e0–1e10; OXYGEN's `o2_supply` reaches ~2e5 while its conserved total is
~27) — normalize by the **max single-stock magnitude** (`quantity_scale`), NOT `total(0)`, giving
horizon-invariant relative drift ~1e-11 abs / ~1e-14 slope; (2) the biomass watch uses **total
organic C** (not peak leaf_c, which hides the moving decomposer) and asserts the year-over-year diffs
**SHRINK** (genuine convergence — stronger than `is_stationary`'s non-amplifying clause, which passes
a linear ramp). **Three-tier delivery: Tier 1** (energy decade — `run_station` 15 yr diurnal; node a
**period-1 fixed point** at `T_eq≈160.08 K`, SOC daily-periodic, ENERGY relative drift flat) + a
**drift-summary golden**; **Tier 2** (3-season combined-ledger run, ~915 days × 1440 sub-steps ≈ 1.3 M
sub-steps ~3 min; every-quantity + ENERGY relative drift flat on the day-boundary trace, biomass
bounded/converging, regulated pools stationary, `rationed==0`, feces boundary open) + a **final-State
golden**; **Tier 3** (assertion-only landmine — `close_feces=True`: litter quasi-steady but
`microbial_carbon` grows unbounded and **`rationed>0` from day 25**; the *same* `is_stationary(bound=
1.0)` that passes Tier 2 (~0.09 diffs) FAILS the landmine (~1e4 diffs) — the symmetric discriminator
earning the instrument its keep). **15 sealed tests** (Tier-1 + Tier-2 stability, Tier-3 landmine,
2 regression goldens). Two additive **NON-frozen** goldens (`sealed_station_state.json`,
`sealed_energy_drift_summary.json`) — **NOT in the freeze manifest**. Full suite incl. `-m slow` +
ruff + pyright green; **all twenty existing goldens byte-identical** (no regen). NEXT: Step 8 (P6.8)
— the cross-domain perturbation harness (cascades, no cascade code).
Roadmap `roadmap_extracted.txt`. Reuse/licensing rules: `docs/reuse-and-licenses.md`.

## Non-negotiable invariants (the things that are easy to get wrong)

- **Core is pure.** `simcore/` imports **stdlib only — zero third-party deps**
  (no numpy/pint/yaml/json/plotting/UI/net). Boundary stuff lives in `sim_io/`
  and `config/`. This keeps the Rust port mechanical.
- **Flows return structured per-stock legs, never a net delta.** A flow is an
  atomic stoichiometric transfer; arbitration scales the *whole flow*.
- **Every flow is internally balanced.** The "outside" is explicit BOUNDARY
  reservoir stocks; `Inputs = Outputs + ΔStored` where Inputs/Outputs are
  boundary deltas. Conservation is asserted every step — a failure is a bug.
- **Determinism:** bit-identical within a build. Time is an **integer step count**
  (`t = n*dt`, never `t += dt`). **Canonical (flow-id) order on every reduction**
  (demand sum, scaling, delta sum). Cross-port (Rust) is tolerance-gated.
- **Arbitration backstop is Euler-only and rare.** It runs always, counts
  firings; golden runs assert the count == 0. Under RK4+, a needed scale is a
  **hard error** (positivity comes from kinetics).
- **Extinction conserves mass:** POPULATION stock below threshold → 0 with the
  residual routed to the loss-sink. POOL stocks are never zeroed-with-loss.
- **RNG** is a counter-based, keyed, pure-Python generator in `simcore`, keyed by
  `(seed, key, n)` so draws are order-independent. No sequential-state RNG.
- **Units** validated at the boundary (pint, in `config/`); the core stores plain
  floats + a canonical-unit label.
- **Parameters are data** (YAML + pydantic schema). No hardcoded coefficients.

## Reuse & licensing (see docs/reuse-and-licenses.md)

- Reimplement science from **primary literature**; cite the paper, not PCSE.
- **PCSE is EUPL (copyleft): offline validation oracle only, never ported or
  imported.** The WOFOST param YAML repo has no license — don't copy it.
- Project's own license is **BNCL-1.0** (Boyko Non-Commercial License v1.0) —
  free to use/modify for non-commercial purposes; commercial use requires
  separate written permission from the copyright holder.

## Commands (once the skeleton exists)

```
uv sync                 # install/lock deps
uv run pytest           # tests (pytest + hypothesis)
uv run ruff check .     # lint
uv run ruff format .    # format
uv run pyright          # types
```

## Testing

- Prefer **test-first** for engine invariants. Use **property-based** tests
  (hypothesis) for universal laws: conservation, non-negativity, order-independence.
- Golden/regression snapshots use **hex-float** for exact comparison.
- Never weaken or delete a test to make it pass; fix the code or flag the gap.

## Working style

- Plan before non-trivial work; keep `docs/plans/*` updated as living docs.
- Keep this file lean. Put detail in `docs/`, not here.
- Repo etiquette: branch before committing; Conventional Commits.
  (Commits keep the harness-required `Co-Authored-By: Claude` trailer.)
