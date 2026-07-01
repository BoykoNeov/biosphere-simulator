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
**Next: Step 5** — the forward-pointer siblings (Thermal / Atmosphere-ECLSS / Crew), each designed
just-in-time. Cross-domain coupling is **Phase 6**.
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
