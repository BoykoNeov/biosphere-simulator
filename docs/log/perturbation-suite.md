## **The perturbation suite reaches the biosphere** (and the oxygen sign it was built on is scoped to REGULATION, not to the science — both unregulated chambers invert the station's)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**BUILT 2026-09-08**, on the user's *"start deliberately breaking things and check the model
still responds sensibly — i agree, go with it"*, narrowed the same session by the user's own
question — *"how is it that only oxygen leaks, and not air? … what about leaking air — less
pressure, less oxygen, less co2 too"* — which is what put the batch on the gas-composition
axis rather than on a breadth-first sweep.

Plan, predictions (written before any run) and their scoring:
`docs/plans/post-roadmap-perturbation-suite.md` §4 and §5.
Suite: `rust/crates/domains/tests/gas_composition_perturbations.rs`, **11 tests**.

**Diagnostics, no golden, no manifest, no unfreeze** — the perturbation modules' own
precedent. Nothing frozen moved.

---

## 1. Two of my own framings were wrong, and the corrections shaped the work

**"Genuinely new ground."** It was not. `rust/crates/station/tests/perturbations.rs` is 495
lines and 10 tests, with five composers and the generic primitives already in place. What was
missing is narrower and measurable: `grep -rln "perturbations::"` returns seven files and
**none under `domains/tests/`**. Every perturbation in this project was a *station cascade*;
not one perturbed the plant science. The work was **extend, biosphere-side**, which is cheaper
than what was proposed and is the half the O₂ inversion actually implicated.

**"A correct hull breach would change nothing here."** Right about the bookkeeping, wrong as
stated about the leaf. Partial pressure is mole fraction × total pressure, so halving the total
and halving both fractions are *the same thing at the leaf*. The model can express a
depressurization's effect on photosynthesis; what it cannot do is **arrive** there.

⚠ **The structural fact behind both, and it is the reason B exists:** the biology reads
`co2_mol / chamber_air_mol` and `o2_mol / chamber_air_mol`, and `chamber_air_mol` is a
**constant carried on the flow** (`flows.rs:65`, 9500.0 at `station/src/scenario.rs:91`).
There is no total-gas stock, no nitrogen and no pressure anywhere in the tree. **Nothing in
this model can lose an atmosphere.**

⚠ `eclss.yaml`'s `o2_setpoint` source string had already written the trigger two days earlier
— *"If an ECLSS is ever wired to a cabin whose air is not 9500 mol, this MUST become a mole
fraction and the flow MUST read the air."* And `consumer_chamber_scenario`'s note asserts the
scale-invariance in prose (*"All three gas quantities scale by the same factor so Ci0 and x_O2
both stay invariant"*) with **nothing checking it**. The gap was documented twice and tested
never.

---

## 2. What was built

Three probes on one axis, each a direction-only contrast against an unperturbed baseline.

* **E0 — the arithmetic, no run.** `Aj = J(Ci−Γ*)/(4Ci+8Γ*)` is **homogeneous of degree zero**
  in `(Ci, Γ*)`: the light-limited branch is *exactly* blind to a proportional change in both
  gases, and therefore to a pressure change at fixed composition. `Ac` is **not**, because `Kc`
  is a constant term in its denominator — asserted as strict monotonicity, with a `Kc = 0`
  control that removes the effect.
* **E1 — the bookkeeping-correct vent.** All three gas quantities scaled: same composition,
  smaller room.
* **E2 — the depressurization as the leaf feels it.** The two gases scaled, the air held: both
  fractions fall together.

**Deferred with a reason, not dropped:** a two-gas *station* leak. `with_station_leak` takes
one pool and one sink whose id is a single constant, so two pools collide on the sink; per-pool
sinks are a change to a shared composer, and a shared composer touched for a probe is how a
diagnostic becomes a refactor.

---

## 3. Four findings, and the batch predicted one and a half of them

**Predictions 1, 2, 4 and 6 held exactly. Prediction 3 held only inside a regime the plan did
not know was there. Prediction 5 held and its stated reason was wrong.**

### 3a. ⚠⚠ The gated observable REVERSES past the rationing cliff

| E1 factor | backstop firings | season-low CO₂ (ppm) | peak crop |
|---|---|---|---|
| ×1.0 | 0 | 7.29454 | 0.409954 |
| ×0.8 | 0 | 5.70008 | 0.404716 |
| ×0.75 | **2** | 5.07224 | 0.403419 |
| ×0.5 | **28** | **7.43317** | 0.397040 |

`season-low chamber CO₂` is the quantity the biosphere contract gates the jar on. It falls
correctly as the room shrinks — until the arbitration backstop fires, at which point the
throttled withdrawal stops drawing the pool down and **the minimum comes back up**. At ×0.5 a
jar rationing 28 times reads *above its own healthy baseline*.

**Peak biomass stays monotone across the same four rooms**, which makes it the honest
observable off the nominal roster; the pair is pinned together, because one of them says the
metric misled rather than that the run went strange.

⚠ The backstop is not misbehaving, and the golden runs assert its firing count is 0 exactly so
the reference never sits here. What is new is that the observable's **direction** is not safe
outside that assumption — *a property of the metric, not of the run*, and nothing in the tree
recorded it.

### 3b. The self-limiting feedback is what keeps the jar well-posed

E1 rations at `f = 0.75`. E2 does **not** ration at `f = 0.25`, holding a third as much carbon
in absolute moles. Lowering the mole fraction lowers `Ci`, so demand falls with supply — the
chamber self-limits. E1 holds the fraction, and therefore the appetite, while removing the
buffer that appetite draws on.

⚠ This is the **same mechanism** as the CO₂-controller result the third direction plan records
(holding the chamber at its starting CO₂ measured four times worse than letting it deplete,
because a controller removes the feedback), reached by an unrelated route — shrink the room
instead of regulating it. Two independent arrivals at one mechanism is the strongest evidence
in the batch.

### 3c. ⚠⚠ The oxygen sign is scoped to REGULATION — and a control refuted my first reading

Halving O₂ alone: **jar −1.45 %, big perennial chamber −0.86 %.** Both negative — the opposite
of the station's **+9.8 %** under an O₂ leak, measured the day before.

Oxygen enters the model **twice**: at the **leaf** through `Γ*` and the Rubisco denominator
(the photorespiration relief), and at the **soil** through `oxygen_limitation_factor`, a
Michaelis–Menten `x/(K+x)` with `K = 1e-4 mol/mol`, applied to decomposition and microbial
respiration. In a sealed chamber the decomposers **are** the CO₂ supply. The station's cabin is
**defended** by `O2Makeup` at 209.8 mmol/mol, so its supply path never bites and only the leaf
path is left to answer.

⚠ **The first explanation was refuted inside this batch by its own control.** I wrote that the
jar's oxygen is scarce and the big chamber's is not, so the big chamber should show the
station's sign. It went **down too** — at a trajectory-minimum soil factor of 0.999040 against
0.999521, a **0.048 % throttle against a 0.86 % crop loss**. The distinguishing feature is
**regulated vs unregulated, not big vs small.** The big chamber's route is 18× its throttle and
is **NOT explained**; perennial compounding is the obvious candidate and has not been run. It
is recorded as open in the test's own docstring rather than guessed at.

⚠ **In the jar the two gas cuts compound rather than fight:** −4.95 % (CO₂ alone), −1.45 % (O₂
alone), −6.70 % together — near-additive. The "assimilation traded against photorespiration"
intuition, which is what prompted this whole axis, is a **station** intuition and is false of
the chambers. *This is the O₂ adoption's own lesson — a finding is scoped to the observable it
was measured on — arriving one level up: scoped to the SCENARIO.*

### 3d. The jar is one halving from anoxia, and the model takes it gracefully

At `f = 0.5` the jar's O₂ bottoms at **5.1e-15 mol** against 1.5e-1 on the baseline — a factor
of 3e13 — and the soil factor collapses with it: decomposition, the sealed jar's entire CO₂
supply, stops.

Both halves are pinned:

* **The fragility.** The healthy jar's own trough already sits at a soil factor of **0.606**,
  deep in the limiting region. Its oxygen charge is not a margin; it is the edge.
* **The robustness, and it is the better news.** Across 3661 steps the stock is **never
  negative**, the backstop **never fires**, no event is raised. First-order donor control means
  the draw vanishes with the pool, so the model reaches zero smoothly rather than overshooting
  into a clamp — the positivity contract behaving exactly as its docstring claims, at the first
  place in this batch that actually tested it.

---

## 4. The mutation battery, and the one gap it did not close

Five mutations to `science.rs`, each applied alone, `--no-fail-fast`, reverted after:

| mutation | reddened |
|---|---|
| M1 `Kc` term folded into `Ci` | 7 of 9 |
| M2 `Aj` denominator `+1.0` | 1 — **only** the invariance test |
| M3 `Ci` ignores `air_mol` | 3 — the air-reading tests |
| M4 `Γ*` stops scaling with O₂ | 3 |
| M5 soil O₂ factor forced to `1.0` | 5 |

Every test reddened under at least one mutation **except** `the_gas_perturbations_are_
deterministic`, which is **inert by nature**: only a nondeterminism-introducing change could
redden a determinism check. Stated rather than shrugged at.

⚠ **The honest gap: the O₂ sign assert has no demonstrated direct falsifier.** Every mutation
that changes the oxygen path enough to flip it (M5, and a sharper M6 freezing the soil factor
at the charge fraction) breaks **well-posedness first**, so the test reddens on its backstop
guard rather than on its sign. Its evidence is a **positive control** instead — the same probe
on a second scenario, which is what produced 3c. That is weaker than a mutation and is recorded
as such rather than counted as coverage.

---

## 5. What this leaves

**B is DECIDED, not proposed** — the user's *"ok A now, but immediately after that (next
session) B"*. Give the habitat a real atmosphere: total gas as a stock, pressure as state, the
leaf reading partial pressures instead of fractions over a constant. That is a frozen-science
change carrying the full ceremony, and this batch exists partly to be the instrument that
measures it. It trips `eclss.yaml`'s own written trigger.

**Named and not taken**, so neither is mistaken for an omission: the PAR/`Vcmax` branch
crossing and a temperature excursion — the natural second batch, better run once E0 has said
which branch binds where, since that is the fact both turn on. And the two-gas station leak
(§2).

⚠ **Not closed, and deliberately:** the third direction plan's *"the station census carries no
CO₂ or cabin-oxygen band"*. Closing it means a row in `station::science_gates!`, which lands in
`docs/station-reference.manifest.json` — **an unfreeze with a ceremony**, not a free
consequence of having built a perturbed subject. Better taken after B, when the quantity it
would freeze is the one B leaves behind rather than the one B replaces.
