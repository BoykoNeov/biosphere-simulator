# Post-roadmap — the second authored habitat (`bioregenerative_station`)

**Status: COMPLETE (2026-08-11).** Authored content, runtime-only. See *Outcome* at the
bottom — the run landed on the design arithmetic before any constant was touched. The first authored
habitat (`docs/plans/post-roadmap-authored-habitat.md`, 2026-07-16) closed carbon and
oxygen out of **six invented flow laws and nothing else**, and closed with three findings.
This file is the answer to its **finding #1**, which is worth quoting in full because it
is the charge this scenario exists to discharge:

> **The flow registry is crew-only.** `FLOW_TYPES` = 3 standalone crew flows; the frozen
> biosphere / power / thermal / eclss science is **not author-selectable**. An authored
> ecosystem must invent its kinetics rather than compose frozen, calibrated laws. […]
> This is the biggest gap between the roadmap's promise (*"a scenario can define a habitat
> with its power budget, thermal limits, crew size, and ecosystem"*) and the platform as
> built.

The registry has since grown 3 → 12 (`post-roadmap-flow-registry-growth.md`), the grammar
gained `monod` (`post-roadmap-grammar-monod.md`), and authoring gained a coupling cadence
(`post-roadmap-multirate-authoring.md`). **Nothing under `scenarios/` has used any of it.**
This habitat uses all three, and it is the first authored file with a power budget, thermal
limits, a crew and an ecosystem in one graph — the roadmap sentence, met.

## Deliverables

- `scenarios/bioregenerative_station.yaml` — the habitat (second file in the content tree).
- `tests/test_bioregenerative_station.py` — the gate set.
- This doc; a row in `docs/post-roadmap-log.md`; one line in the CLAUDE.md index.

**No golden, no manifest entry, no cross-port anchor, `git diff src/` empty.** Authored
artifacts never become reference (Phase-9 decision B), and `tests/crossport` anchors live
in `tests/authoring/scenarios/` — adding one there is a different, freeze-adjacent act.

## The collision this design had to resolve first (advisor)

Two facts already in the tree are in direct tension, and getting them backwards would have
produced a file that either cannot close or cannot compose:

1. `algae_habitat.yaml` closes **oxygen** only because `cabin.co2` carries
   `composition: {carbon: 1, oxygen: 2}`, and authored `kinetics` flows are
   stoichiometry-checked against that fold **at build time**.
2. `eclss_cabin.yaml` / `eclss_thermal_habitat.yaml` / `eclss_multirate_cabin.yaml` all warn
   that an ECLSS cabin's pools must be **un-annotated**, because frozen `type` flows get
   **no** build-time check and annotating `{carbon: 1, oxygen: 2}` breaks OXYGEN balance on
   `eclss.crew_metabolism` — surfacing as a runtime `ConservationError`.

Read together they look like "compose the registry" and "close oxygen" are mutually
exclusive. **They are not — the discriminator is leg shape, not frozen-vs-authored:**

> A frozen flow is composition-safe iff every one of its legs lands on stocks that fold
> the same way. A **two-leg, one-magnitude** transfer (`−R` here, `+R` there) balances
> *every* quantity automatically, whatever composition the two ends share. A flow whose
> legs cross composition classes with independent magnitudes does not.

So `eclss.co2_scrubber`, `eclss.condenser`, `eclss.o2_makeup`, `power.self_discharge` are
all safe against annotated pools; `eclss.crew_metabolism`, `crew.food_metabolism` and
`crew.oxygen_consumption` are not — each moves one quantity while leaving the *atoms that
travel with it* to a decoupled reservoir, which is exactly the Phase-6 seam. **This habitat
therefore composes the two-leg frozen equipment and authors the atom-coupled physiology**,
and that split is a finding, not a workaround (see "Findings" below).

### The second half: frozen flows terminate at boundary reservoirs

Every frozen ECLSS field is *named* for a boundary (`co2_removed`, `o2_supply`,
`humidity_condensate`), so a habitat built from them looks open by construction.

**Verified it is not.** `interpreter._build_flow` (`src/authoring/interpreter.py:568-575`)
validates only that the wiring **key names** match the flow type's declared fields, then
passes each value straight through as a `StockId`. There is no `kind` constraint anywhere
on the path. A wiring field named for a boundary may point at an interior `kind: pool`.

That single fact is what makes this habitat possible: **the scrubber stops discarding and
starts feeding.** `co2_removed` → `bioreactor.co2_feed`; `humidity_condensate` →
`water.condensate`; `o2_supply` → `store.o2_tank`. The frozen, calibrated equipment becomes
recycling machinery inside a closed loop, with no engine change of any kind.

## The design

A bioregenerative life-support station. The crew's exhaled CO₂ is captured by the **frozen
CDRA-style scrubber** and concentrated into a bioreactor feed tank; a photobioreactor fixes
that carbon into algal biomass and releases O₂ into a buffer tank; the **frozen proportional
O₂ regulator** meters the tank back into the cabin at its setpoint; harvest feeds the crew.
Humidity is captured by the **frozen condenser**, urine and condensate are recovered back to
the potable store. A solar array charges a battery through the **frozen charge-efficiency
split**, a constant load dissipates it, and every joule of waste heat lands on a **frozen
radiator** node that rejects it to space.

### Run config

`integrator: euler` · `dt: 3600.0` (the master cadence, 1 h) · `n_sub: 12` · `steps: 8760`
(**one sealed year**) · `rng_seed: 0`.

### The rate-class partition — an honest two-timescale separation

`eclss_multirate_cabin.yaml`'s header is candid that its partition is *"a fixture device,
not a sizing claim"* — ECLSS's four flows are all the same order, so it had to manufacture
one cross-boundary stock to have something to prove. **Here the separation is physical and
the boundary crossings are unavoidable:**

| class | effective step `h` | what is in it | why |
|---|---|---|---|
| **fast** | `dt/n_sub` = **300 s** | the three ECLSS loops + the four crew metabolic flows | cabin-air time constants are 500–2000 s; the crew flows are forced constants that feed/drain those same pools |
| **slow** | `dt/2` = **1800 s** (Strang, *not* `dt/n_sub`) | photosynthesis, algal respiration, harvest, remineralization, the two water processors, the three power flows, the radiator | biology relaxes over **weeks**; the radiator over **52 h**; the battery over **3.2 yr** |

**Seven stocks are shared across the rate-class boundary** — `crew.food_store`,
`bioreactor.co2_feed`, `store.o2_tank`, `waste.feces`, `crew.water_store`,
`water.condensate`, `waste.urine` — so Strang's operators genuinely do not commute here.
That is a property of the physics, not a device.

**Why the crew flows are fast even though they are forced.** Sub-stepping a forced flow is
free and exact (`rate·h·n_sub ≡ rate·dt` to roundoff). Classing them slow is what would
cost: `crew.respiration` alone injects `4.745e-4 × 1800 = 0.854` mol of CO₂ per half-step
into a cabin pool that holds **0.4745** mol at steady state — a 180 % kick into a pool
whose own time constant (1000 s) is *shorter* than the slow step. The trajectory would
survive (no negative stock, no rationing) and would export a violent sawtooth. This is the
export-fidelity hazard as a *partitioning* question, and it is why "which class" is a
modelling decision, not a performance knob.

### Stocks — 15, of which exactly **two are boundary**, both ENERGY

| id | quantity | kind | composition | initial | steady state |
|---|---|---|---|---|---|
| `cabin.co2` | carbon | pool | `{carbon:1, oxygen:2}` | 0.0 | 0.4745 |
| `cabin.o2` | oxygen | pool | `{oxygen:2}` | 8.0 | 9.76275 |
| `cabin.humidity` | water | pool | — | 0.0 | 0.0405 |
| `bioreactor.co2_feed` | carbon | pool | `{carbon:1, oxygen:2}` | 100.0 | 450.0 |
| `bioreactor.biomass` | carbon | pool | — | 200.0 | 1000.0 |
| `store.o2_tank` | oxygen | pool | `{oxygen:2}` | 500.0 | 147.76275 |
| `crew.food_store` | carbon | pool | — | 1900.0 | 736.7755 |
| `crew.water_store` | water | pool | — | 60.0 | 53.0595 |
| `waste.feces` | carbon | pool | — | 0.0 | 12.75 |
| `waste.urine` | water | pool | — | 0.0 | 4.875 |
| `water.condensate` | water | pool | — | 0.0 | 2.025 |
| `power.battery` | energy | pool | — | 1.0e+9 | 1.0e+9 (by sizing) |
| `thermal.node` | energy | pool | — | 1.0e+9 | 2.99127e+9 |
| `boundary.solar_source` | energy | **boundary** | — | 0.0 (`unclamped`) | −1.26144e+11 |
| `boundary.space` | energy | **boundary** | — | 0.0 | +1.24153e+11 |

All POOL, no POPULATION (the first habitat's reasoning: bulk cultures, not discrete
populations, and no loss-sink machinery is wanted).

### Flows — 17: **7 frozen types, 10 authored**

**Frozen (calibrated *form*, `TODO(cite)`-discharged-as-DESIGN *values*):**

| id | type | wired to | law |
|---|---|---|---|
| `eclss.co2_scrubber` | `eclss.co2_scrubber` | `cabin.co2 → bioreactor.co2_feed` | `k_scrub·C`, k = 1.0e-3 |
| `eclss.condenser` | `eclss.condenser` | `cabin.humidity → water.condensate` | `k_cond·H`, k = 5.0e-4 |
| `eclss.o2_makeup` | `eclss.o2_makeup` | `store.o2_tank → cabin.o2` | `k_m·(S* − O)`, k = 2.0e-3, S* = 10.0 |
| `power.solar_charge` | `power.solar_charge` | `boundary.solar_source → power.battery` (+`thermal.node`) | forced `solar_power`, η_c = 0.95 |
| `power.load_draw` | `power.load_draw` | `power.battery → thermal.node` | forced `load_power` |
| `power.self_discharge` | `power.self_discharge` | `power.battery → thermal.node` | `k_sd·B`, k = 1.0e-8 |
| `thermal.radiator_reject` | `thermal.radiator_reject` | `thermal.node → boundary.space` | `εσA(T⁴ − T_space⁴)` |

**The `waste_heat` wiring is the Phase-6 Power↔Thermal seam, authored.** All three power
flows point their `waste_heat` field at `thermal.node` rather than at a boundary sink, so
the station's dissipation lands on the radiator instead of vanishing. `thermal.heat_input`
is deliberately **not** used: with the seam wired, a forced heat load would be
double-counting.

**Authored (`kinetics`, uncalibrated — every rate strictly positive by construction,
one flow per physical process):**

| id | class | rate | stoichiometry |
|---|---|---|---|
| `crew.respiration` | fast | `param(respired_carbon_fraction) · forcing(crew_food_intake)` | `food −1, cabin.co2 +1, cabin.o2 −1` |
| `crew.egestion` | fast | `forcing(…) − param(…)·forcing(…)` | `food −1, feces +1` |
| `crew.perspiration` | fast | `param(insensible_water_fraction) · forcing(crew_water_intake)` | `water_store −1, humidity +1` |
| `crew.urination` | fast | `forcing(…) − param(…)·forcing(…)` | `water_store −1, urine +1` |
| `bioreactor.photosynthesis` | slow | `1.0e-3 · monod(stock(co2_feed), 300.0)` | `co2_feed −1, biomass +1, o2_tank +1` |
| `bioreactor.respiration` | slow | `1.0e-7 · stock(biomass)` | `biomass −1, co2_feed +1, o2_tank −1` |
| `bioreactor.harvest` | slow | `5.0e-7 · stock(biomass)` | `biomass −1, food +1` |
| `waste.remineralization` | slow | `2.0e-6 · stock(feces)` | `feces −1, co2_feed +1, o2_tank −1` |
| `water.recovery` | slow | `1.0e-5 · stock(condensate)` | `condensate −1, water_store +1` |
| `water.urine_processing` | slow | `2.0e-6 · stock(urine)` | `urine −1, water_store +1` |

**The two frozen crew fractions are read through `param()`, not transcribed** — the
`self_discharge_dsl` precedent, and the first habitat's reason: stoichiometry takes literal
floats only, so a merged four-leg respiration would hardcode `0.949`/`0.051` and silently
duplicate a BVAD-calibrated value. Splitting each metabolic fate into its own flow keeps
every coefficient an exact integer **and** sources the fraction from the frozen loader.

### Why `monod` — and why it is the *opposite* of a decoration here

The first habitat had to make photosynthesis first-order in CO₂ **only**, deliberately
dropping the biomass factor, because a `∝ biomass` law makes the per-step draw fraction
grow with the culture (measured `≈ 18` at steady-state biomass — CO₂ negative in one step).

A photobioreactor is genuinely **light-limited**, not substrate-limited: at any useful cell
density the productivity is set by the illuminated area, not by how much biomass is behind
it. The faithful law is therefore **zeroth-order** — `P = V_max` — and zeroth-order is
exactly the shape that *cannot* be made positivity-safe, because the draw does not know how
much substrate is left.

`monod` fixes precisely that, and this is the whole argument for the op:

```
P = V_max · S/(S + K)
    S ≫ K  →  P → V_max          (light-limited: the physics we want)
    S ≪ K  →  P → (V_max/K)·S    (first-order: donor-controlled, self-limiting at 0)
```

So the effective rate constant at depletion is `V_max/K = 1.0e-3/300 = 3.333e-6 /s`, giving
`k_eff·h = 6.0e-3 ≪ 1` at the slow step. **The saturating form restores structural
positivity to a law that has none.** The first habitat traded physics for safety; `monod`
means this one does not have to.

**The factor genuinely traverses its curve** (the `monod_dsl.yaml` dead-anchor lesson): the
feed tank starts at 100.0 mol (factor **0.250**) and settles at 450.0 (factor **0.600**).

**`K = 300.0` is a bare inline literal, and that is still a finding.** No registered param
loader exposes a half-saturation — `charge`/`crew`/`eclss`/`self_discharge`/`thermal`
between them offer an efficiency, two fractions, four ECLSS gains, a leak rate and four
radiator properties. The frozen `K_O2` that motivated `monod` lives in the biosphere, whose
loaders are unregistered. `monod_dsl.yaml` measured this in July; it is unchanged, and now
it bites authored *content* rather than a fixture.

### Forcings

`crew_food_intake: 5.0e-4` mol C/s · `crew_water_intake: 3.0e-5` kg/s — both the frozen
`CrewScenario` defaults at one crew member. `solar_power: 4000.0` W ·
`load_power: 3790.0` W (sized below).

## The steady state — solved before the run, not fitted after it

Write `q = 5.0e-4` (food C draw), `w = 3.0e-5` (water draw), `f = 0.949`, `g = 0.675`.

### Carbon — and why the scrubber is load-bearing

```
dF/dt  = k_h·B − q
dCc/dt = f·q − k_scrub·Cc
dCn/dt = k_scrub·Cc + k_r·B + k_d·W − V_max·φ(Cn)
dB/dt  = V_max·φ(Cn) − (k_h + k_r)·B
dW/dt  = (1−f)·q − k_d·W          with  φ(S) = S/(S+K)
```

Conservation check: `dF + dCc + dCn + dB + dW = −q + f·q + (1−f)·q = 0` ✓ — **no boundary
leg anywhere in the carbon book.**

| condition | gives |
|---|---|
| `k_h·B = q` → `5.0e-7 × 1000 = 5.0e-4` | **B\* = 1000.0** |
| `k_scrub·Cc = f·q` → `1.0e-3 × 0.4745 = 4.745e-4` | **Cc\* = 0.4745** |
| `k_d·W = (1−f)·q` → `2.0e-6 × 12.75 = 2.55e-5` | **W\* = 12.75** |
| `V_max·φ = (k_h+k_r)·B` → `1.0e-3 × 0.6 = 6.0e-4` | `φ* = 0.6` → **Cn\* = K·φ/(1−φ) = 450.0** |
| `F* = C_total − Cc − Cn − B − W` | **F\* = 2200 − 1463.2245 = 736.7755** |

Substituting back into `dCn/dt`: `4.745e-4 + 1.0e-4 + 2.55e-5 − 6.0e-4 = 0` ✓ — the two
recycle streams (algal maintenance respiration and waste remineralization) plus the scrubber
output exactly feed the reactor.

**Every mole of carbon the crew exhales reaches the algae only by passing through the frozen
scrubber**, and the scrubber's own frozen `k` is what sets the cabin's standing CO₂
inventory (`Cc* = f·q/k_scrub` — a *calibrated-form* equipment property setting a habitat
state variable). That is the sentence this whole file exists to be able to write.

### Oxygen — closes through the composition fold, exactly as before

Every O-bearing stock is diatomic-equivalent, so the conserved quantity is
`2·(Oc + Ot + Cc + Cn)`.

```
dOc/dt = k_m·(S* − Oc) − f·q
dOt/dt = V_max·φ(Cn) − k_r·B − k_d·W − k_m·(S* − Oc)
```

- `Oc* = S* − f·q/k_m = 10.0 − 4.745e-4/2.0e-3 = ` **9.76275** mol
- `dOt/dt` at steady state: `6.0e-4 − 1.0e-4 − 2.55e-5 − 4.745e-4 = 0` ✓
- `Ot* = 608.0 − 9.76275 − 0.4745 − 450.0 = ` **147.76275** mol

Note `Ot` is not independent: `Ot = 608 − Oc − Cc − Cn`, so the O₂ buffer draining
500 → 147.8 **is** the carbon filling the feed tank 100 → 450, seen from the other book.

### The direction gate, reasoned rather than hoped

`eclss.o2_makeup` is the registry's only demand-controlled type, so `ReversedFlowError`
fires if `cabin.o2` is ever found **above** `o2_setpoint = 10.0`. It is not:

- the equilibrium is **below** the setpoint by construction — `Oc* = S* − f·q/k_m = 9.76275`,
  and the offset `f·q/k_m` is strictly positive because the crew always breathes;
- `Oc` starts at 8.0, i.e. **below** it, and the per-sub-step map is
  `Oc ← Oc + 0.6·(10 − Oc) − 0.14235`, a contraction with factor `1 − k_m·h = 0.4 > 0` —
  so it approaches `9.76275` **monotonically from below and cannot overshoot**;
- nothing else adds O₂ to the cabin. Photosynthesis fills the *tank*, and the regulator is
  the only path from tank to cabin.

This is the `post-roadmap-o2-makeup-reversal.md` finding taken seriously: the reversal is
*not* author-only, it fires inside frozen runs, and the safe direction has to be argued from
the sign of the offset — not assumed because "the crew consumes O₂".

### Water — the quantity the first habitat put out of scope

```
H*  = g·w/k_cond = 2.025e-5/5.0e-4 = 0.0405 kg
Cd* = g·w/k_rec  = 2.025e-5/1.0e-5 = 2.025 kg
U*  = (1−g)·w/k_up = 9.75e-6/2.0e-6 = 4.875 kg
Ws* = 60.0 − 0.0405 − 2.025 − 4.875 = 53.0595 kg
```

Closes with **zero boundary stocks**: `store → humidity → condensate → store` and
`store → urine → store`. 6.94 kg of the 60 kg inventory (11.6 %) ends up in transit — the
loop's standing charge, filled from empty over the run.

**Honestly: recovery here is 100 %.** Real water processors recover ~85–93 %; a residual
brine reject would be an open leg. Full recovery is an idealisation, chosen so the water
book makes the same strict-closure claim as carbon and oxygen. It is authored, so it is
not a calibration claim either way.

### Energy — deliberately **open**, and that is the physically correct answer

The roadmap's charge is *"closure of matter — **and energy** — cycles"*. Matter closes.
Energy cannot, and a habitat that claimed otherwise would be wrong: a station is a heat
engine running on a temperature difference between a 5800 K source and a 2.7 K sink.

```
battery:  η_c·P_solar − P_load − k_sd·B = 0.95×4000 − 3790 − 1.0e-8×1.0e+9
                                         = 3800 − 3790 − 10 = 0     ⇒ B* = 1.0e+9 J
node in:  (1−η_c)·P_solar + P_load + k_sd·B = 200 + 3790 + 10 = 4000 W  ( = P_solar ✓ )
radiator: εσA·(T⁴ − T_space⁴) = 4000  with  εσA = 0.85 × 5.670374419e-8 × 10 = 4.81982e-7
          T_eq = (4000/4.81982e-7 + 2.7⁴)^¼ = 301.83 K
          Q_eq = C·(T_eq − T_space) = 1.0e+7 × 299.13 = 2.99127e+9 J
```

Every joule that enters leaves: **4000 W in from `boundary.solar_source`, 4000 W out to
`boundary.space`**, with the battery an exactly-balanced buffer in between. The two boundary
stocks in this file are both ENERGY, and their existence is the honest statement.

**The battery does not relax — it is sized.** `τ = 1/k_sd = 1.0e+8 s = 3.2 yr`, far longer
than the run, so within one year the battery is an *accumulator*, not a restoring stock.
`load_power = 3790.0` is chosen to put it exactly at its fixed point; asserting it holds is
a check on the **sizing**, not a demonstration of an attractor. The thermal node is the live
half of the energy story: it starts at 1.0e+9 J (**102.7 K**) and warms to 301.8 K with
`τ = C/(4εσA·T_eq³) = 1.886e+5 s ≈ 52 h`.

**What is NOT modelled, by declaration:** the energy half is **budgeted, not coupled**. The
photobioreactor's lamp power sits inside `load_power` as a number, but photosynthesis is not
gated on the battery — `V_max` is a constant, not a function of available power. Making the
biology draw on the bus is the obvious successor to this file and is deliberately not
attempted here (it would need an invented light-response law and would destroy the
analytic fixed point that makes the gates sharp).

## Stability — the `(B, Cn)` subsystem

`Cc`, `Oc` and `H` are fast-slaved (their own time constants are 300–2000 s against the
biology's weeks), and `W`, `Cd`, `U` decouple as first-order lags. The coupled core is
biomass against reactor feed, with `β = V_max·φ'(Cn*) = V_max·K/(Cn*+K)² =
1.0e-3 × 300/750² = 5.3333e-7`:

```
J = [ −(k_h + k_r)     β    ]  =  [ −6.0e-7    5.3333e-7 ]
    [      k_r        −β    ]     [  1.0e-7   −5.3333e-7 ]

trace = −1.13333e-6 < 0
det   = 3.2e-13 − 5.3333e-14 = 2.66667e-13 > 0        ⇒ stable
disc  = trace² − 4·det = 1.28444e-12 − 1.06667e-12 = 2.1778e-13 > 0   ⇒ REAL eigenvalues
λ₁ = −3.3333e-7  (τ = 3.00e+6 s ≈ 34.7 d)   λ₂ = −8.0e-7  (τ = 1.25e+6 s ≈ 14.5 d)
```

Stable, real, **non-oscillatory**. A one-year run is `≈ 10.5 τ` of the slow mode.

**This was a design choice, not luck.** The first draft made photosynthesis
`μ·B·φ(Cn)` — the textbook Monod *growth* law, proportional to biomass. Its Jacobian is

```
J = [ 0            μB·φ'  ]   trace = −5.3333e-7,  det = 2.6667e-13,
    [ k_r − μφ*   −μB·φ'  ]   disc  = −7.82e-13 < 0   ⇒ COMPLEX
```

— a damped oscillator with a **164-day period** and a 43-day decay: a substrate–consumer
ring that needs most of a year just to complete one cycle, from initial conditions far
outside the linear regime. Dropping the biomass factor (the light-limited form, which is
also the better physics for a dense reactor, and the frozen `Photosynthesis` idiom) makes
the eigenvalues real and the run converge. **The arithmetic caught this before the file was
written** — which is the whole reason the first habitat's plan does it this way too.

### Every step-size precondition, checked

| flow | class | `h` | `k·h` |
|---|---|---|---|
| `eclss.o2_makeup` | fast | 300 | 2.0e-3 × 300 = **0.60** ✓ (build-time checked) |
| `eclss.co2_scrubber` | fast | 300 | 1.0e-3 × 300 = **0.30** ✓ (build-time checked) |
| `eclss.condenser` | fast | 300 | 5.0e-4 × 300 = **0.15** ✓ (build-time checked) |
| `power.self_discharge` | slow | 1800 | 1.0e-8 × 1800 = **1.8e-5** ✓ (build-time checked) |
| `bioreactor.photosynthesis` | slow | 1800 | `V_max/K` × 1800 = **6.0e-3** ✓ (authored — the author's job) |
| `bioreactor.harvest` | slow | 1800 | 5.0e-7 × 1800 = **9.0e-4** ✓ |
| `bioreactor.respiration` | slow | 1800 | 1.0e-7 × 1800 = **1.8e-4** ✓ |
| `waste.remineralization` | slow | 1800 | 2.0e-6 × 1800 = **3.6e-3** ✓ |
| `water.recovery` | slow | 1800 | 1.0e-5 × 1800 = **1.8e-2** ✓ |
| `water.urine_processing` | slow | 1800 | 2.0e-6 × 1800 = **3.6e-3** ✓ |
| `thermal.radiator_reject` | slow | 1800 | `τ/h = 105` ✓ (no predicate exists — sizing) |

`n_sub = 12` is set by the tightest fast constraint, the regulator: `h < 1/k_m = 500 s`.
`n_sub = 8` (`h = 450`) would put it at 0.90, **legal but at 90 % of the bound for no gain**
— `eclss_multirate_cabin.yaml`'s own words, applied.

## What is frozen vs authored (the "authored ≠ validated" ledger)

- **Frozen and reused:** 7 flow *types* (their rate laws), 4 param sets (`eclss`, `charge`,
  `self_discharge`, `thermal`) through the frozen loaders and their bounds/unit guards, and
  the two BVAD-calibrated crew fractions via `param()`.
- **Frozen ≠ calibrated, and this file leans on that hard.** Of the frozen values, only
  `respired_carbon_fraction` (0.949) and `insensible_water_fraction` (0.675) are calibrated
  (NASA BVAD Table 3-31). Every ECLSS / power / thermal value is a `DESIGN` placeholder
  whose param file says outright that no source *can* fix it — `o2_setpoint` is "permanently
  un-bindable as written", and `co2_scrub_rate` is recorded as ~1–1.5 orders of magnitude
  fast against the real ISS CDRA cadence. Composing frozen types buys **frozen form**, not
  endorsement (`flow_registry.py`: "Registered ≠ calibrated").
- **Authored (uncalibrated):** all 10 `kinetics` laws and their 7 inline constants
  (`V_max`, `K`, `k_r`, `k_h`, `k_d`, `k_rec`, `k_up`), chosen to place the fixed point
  somewhere physically sensible. Not literature-derived.
- **Consequence:** `has_authored_kinetics = True`; Godot banners it UNCALIBRATED. No golden,
  no manifest entry, no calibration claim, no place in any reference.

## Gates

1. **Conservation** — asserted every step by the engine (and once per master step inside
   `multirate_step`). CARBON, OXYGEN and WATER each balance with **zero boundary legs**;
   ENERGY balances *including* its two boundary stocks (`Inputs = Outputs + ΔStored`).
2. **Determinism** — two interpret+run passes bit-identical.
3. **No rationing** — `run_scenario` now *raises* `RationedError`, so a completed run is
   itself the gate; asserted explicitly anyway.
4. **No reversal** — `run_scenario` raises `ReversedFlowError` if `cabin.o2` is ever found
   above the setpoint. Argued above; asserted as a trajectory bound.
5. **The loops are live** — biomass 5×, the feed tank 4.5×, the O₂ buffer drained 3.4×, the
   cabin O₂ regulated up from 8.0, the node warmed 102.7 K → 301.8 K, the water loop charged
   from empty. Final state within **1 %** of the analytic fixed point on all 13 interior
   stocks, with the residual dominated by the multi-rate export offset — which is *itself*
   pinned in closed form rather than left inside the tolerance (see the Outcome).
6. **The monod factor traverses** — 0.250 → 0.600, measured, not assumed.
7. **`has_authored_kinetics` is True.**
8. `uv run pytest`, `ruff`, `pyright` green; **`git diff src/` empty.**

## Findings this design surfaces

1. **The composition rule is about leg shape, not about frozen-vs-authored.** The three
   ECLSS fixtures all state "an authored cabin must not annotate composition" as though it
   were a property of frozen flows. It is a property of *multi-magnitude cross-quantity*
   flows. Two-leg one-magnitude frozen transfers compose freely with annotated pools, which
   is the entire basis of this habitat. The existing warnings are correct about their own
   subject (`eclss.crew_metabolism`) and over-general as written.
2. **A frozen flow's "boundary" wiring field is a name, not a constraint** — verified in
   `interpreter.py`. This is what lets equipment recycle instead of discard, and it is
   nowhere documented as an authoring capability.
3. **`monod` earns its unfreeze on content, not just on a fixture.** It converts a
   zeroth-order (light-limited) uptake law from structurally-unsafe to
   structurally-self-limiting. That is a stronger justification than the one in
   `post-roadmap-grammar-monod.md`, which mirrored a frozen kernel.
4. **Still no half-saturation param in any registered loader** (`monod_dsl.yaml` measured
   this in July). Any author reaching for `monod` writes `K` as a bare literal. Unchanged.
5. **The atom-coupled crew flows are still not composable.** `crew.food_metabolism` is
   carbon-only and `crew.oxygen_consumption` oxygen-only; wiring either into a
   `{carbon:1, oxygen:2}` pool fabricates or destroys oxygen. The registry has no
   atom-coupled respiration, so physiology stays authored. This is the *residue* of the
   first habitat's finding #1 — narrowed from "the registry is crew-only" to "the registry
   has no composition-aware metabolic flow", which is a much smaller and much more specific
   gap.
6. **The energy half is budgeted, not coupled** (see above). Naming it is the point;
   building it is a separate decision.

## Outcome — COMPLETE (2026-08-11)

**The station runs and closes.** One sealed year (8760 master steps at `dt = 3600`,
`n_sub = 12`; ~8.6 s), 18 tests green. The run relaxes from an off-equilibrium start in all
four books at once and lands on the analytic fixed point — **the design arithmetic
predicted the behaviour before the file was written, and no constant was tuned afterwards:**

| stock | initial | final | predicted |
|---|---|---|---|
| `crew.food_store` | 1900.0 | 737.249 | 736.7755 |
| `cabin.co2` | 0.0 | 0.474500 | 0.4745 |
| `bioreactor.co2_feed` | 100.0 | 449.565 | 450.0 |
| `bioreactor.biomass` | 200.0 | 999.984 | 1000.0 |
| `waste.feces` | 0.0 | 12.7270 | 12.75 |
| `cabin.o2` | 8.0 | 9.76275 | 9.76275 |
| `store.o2_tank` | 500.0 | 148.198 | 147.76275 |
| `cabin.humidity` | 0.0 | 0.0405000 | 0.0405 |
| `water.condensate` | 0.0 | 2.00661 | 2.025 |
| `waste.urine` | 0.0 | 4.86621 | 4.875 |
| `crew.water_store` | 60.0 | 53.0867 | 53.0595 |
| `power.battery` | 1.0e+9 | 1.0e+9 (drift **exactly 0**) | 1.0e+9 |
| `thermal.node` | 1.0e+9 (102.700 K) | 2.99126e+9 (**301.827 K**) | 2.99127e+9 (301.83 K) |

- **CARBON drift over the year: `+3.1e-12` relative** (3.1e-9 mol on 2200).
  **OXYGEN: `+4.3e-12`** (5.2e-9 atoms on 1216). **WATER: `−2.8e-14`** (1.7e-12 kg on 60).
  All three with **zero boundary stocks** — closure in the strict sense, inputs = outputs = 0.
- **ENERGY balances including its boundary:** 126.144 GJ taken in from the sun
  (`= 4000 W × 3.1536e7 s`, exact to 1e-12), 124.153 GJ rejected to space, 1.99126 GJ stored
  in the node. `Inputs = Outputs + ΔStored` ✓.
- **Arbitration backstop fired 0 times**; 0 extinction events; no interior stock went
  negative. The food store declined **monotonically** 1900 → 737 and flattened (< 1 % over
  the last tenth of the year), so the forced-crew sizing condition held with 737 mol to spare.
- **`cabin.o2` peaked at 9.76275**, i.e. never reached the 10.0 setpoint, and rose
  monotonically — the direction gate held exactly as argued, not by luck.
- **The monod factor traversed 0.250000 → 0.599768.**
- All eight gates green. `git diff src/` **empty**; `ruff`, `ruff format`, `pyright` clean.

### The one thing the design arithmetic did NOT predict — and it is a finding

Three pools settled a little below their continuous steady states: `waste.feces` −0.18 %,
`waste.urine` −0.18 %, `water.condensate` −0.91 %. That is **not** residual transient (those
pools are 63–315 τ in by the end) and **not** error. It is the **multi-rate export offset**,
and it is closed-form:

> For a pool fed only by FAST flows and drained by ONE slow first-order flow, Strang puts
> the fast block *between* the slow set's two half-steps, so the inflow takes only one
> half-step of decay before the export point while the standing amount takes two:
>
> ```
> X_exported = X_continuous · 2a/(1 + a),      a = 1 − k·(dt/2)
> ```

**Measured against all three pools, the formula is exact to 14 significant figures**
(rel. 2.5e-14, 8.0e-14, 6.6e-15). It is pinned in
`test_the_multirate_export_offset_is_exact` rather than absorbed into a tolerance, because
a tolerance-only check is exactly where a genuinely mis-lowered partition would hide —
`eclss_multirate_cabin.yaml`'s own point: a mis-driven partition does not drift, it lands
somewhere else. **Stated as a rule for future authored content: the analytic fixed point is
not what a multi-rate run exports, and the gap is derivable, so derive it.**

### Mutation-verified (five mutations, five different layers)

A passing test proves nothing until it has been seen to fail. Each mutation was run against
the real file and caught at a *different* layer of the platform:

| mutation | caught by | verdict |
|---|---|---|
| strip `{carbon:1, oxygen:2}` off `cabin.co2` | **build time**, `AuthoringError` | *"flow 'crew.respiration': authored stoichiometry is not balanced for OXYGEN (Σ coeff·composition = −2.0)"* |
| point the scrubber's `co2_removed` back at a boundary | **run time**, `ConservationError` | *"conservation violated for CARBON: residual −1.240 … (boundary_delta=0.0, stored_delta=−1.240)"* |
| start `cabin.o2` at 12.0, above the setpoint | **run time**, `ReversedFlowError` | *"…cabin.o2 reached 12.0 — above it, at the INITIAL state, so this is a wiring error"* |
| class `eclss.co2_scrubber` `slow` | **build time**, `AuthoringError` | *"…integrates the flow at dt/2 = 1800.0 (the slow set's Strang half-step…)"* — `k·h = 1.8 ≥ 1`, i.e. the exact false-PASS trap `_effective_step` documents |
| drop `monod`, leaving the bare zeroth-order `rate: '1.0e-3'` | **run time**, `RationedError` | **the backstop fired 8731 times out of 8760 master steps** |

**The last one is the sharp one.** The design argues that `monod` is what makes a
light-limited (zeroth-order) uptake law positivity-safe. Deleting the op and leaving
everything else identical turns a clean run into one that rations on 99.7 % of its steps.
That converts the justification for the op from an argument into a measurement.

### What the advisor changed

The advisor's pass before any file was written caught the **composition collision** — that
`algae_habitat.yaml`'s "annotate `cabin.co2` or oxygen cannot close" and the three ECLSS
fixtures' "never annotate a cabin pool" look mutually exclusive, and that committing to
either reading blindly produces a habitat that cannot close *or* cannot compose. Chasing
that down is what produced the **leg-shape rule** (finding 1), which is the load-bearing
idea of the whole file. It also named the discriminating check — *can a frozen flow's
boundary-named wiring field point at an interior pool?* — which turned out to be the single
fact everything else rests on (finding 2), and insisted the design arithmetic be written
before the run rather than after, which is what caught the ringing first draft.

### Deliberately NOT done

- **No unfreeze of anything:** not the registry, not the grammar, not a manifest, not a
  param file. `git diff src/` empty.
- **No golden, no manifest entry, no cross-port anchor.** Authored artifacts never become
  reference (decision B), and `tests/crossport/authoring_files.py::ANCHORS` draws from
  `tests/authoring/scenarios/` — adding one there is a freeze-adjacent act, not this task.
  (Note this file would be **Tier 2** if it ever were anchored: `thermal.radiator_reject`
  evaluates `T**4`, a transcendental site — `thermal_node.yaml`'s precedent.)
- **No `includes`/bundle factoring.** Shared-stock composition across bundles is still a
  documented deferral, and this station shares the cabin across every one of its halves.
- **No energy↔biology coupling.** `V_max` is a constant, not a function of bus power. Named
  as the successor, not attempted.
- **The clamped `store.o2_tank` path is not exercised** (advisor). The tank is a POOL rather
  than an `unclamped` boundary because it is a real finite inventory, and the frozen
  regulator withdraws from it — so arbitration *would* scale that draw if the tank ran low.
  But it drains monotonically 500 → 148.2 and never approaches zero, so **`rationed == 0`
  says nothing about that path**: the claim is structural reasoning, not a measurement.
  Sizing a run that actually empties the buffer is a different scenario and was not built.
  Recorded rather than left implied, because a header comment asserting a design intent
  reads as verified when it is not.
- **The deferred grammar stayed deferred:** nothing here needed bare division, `pow`, `exp`,
  a conditional or a named constant, so nothing forced a semantic choice.

### The honest limitation

The ten authored laws are **invented**, and the frozen values the seven composed types read
are `DESIGN` placeholders, not measurements — so this is a demonstration of the *platform*
and of *closure*, not of calibrated life support. What it does now demonstrate that nothing
did before: an author can take frozen, literature-derived flow **forms** off the registry
shelf, wire them into a closed loop rather than out to a sink, drive them at their own
cadence alongside biology that runs a thousand times slower, and have the engine hold three
conservation books to 1e-11 for a year — without touching a line of engine code.

