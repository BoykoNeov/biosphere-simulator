# Perturbation suite, biosphere side — and what it measures about the missing atmosphere

**Written 2026-09-08**, on the user's *"start deliberately breaking things and check the
model still responds sensibly — i agree, go with it"*, then narrowed by the user's own
question (*"how is it that only oxygen leaks, and not air?"*) to the gas-composition axis.

Named in records as **the perturbation-suite plan**. The direction plan is referred to as
*"the third direction plan"*, never by filename.

**Course chosen by the user 2026-09-08: A now, B next session.**

* **A — this plan.** Probes only. Diagnostics, no golden, no manifest, no unfreeze.
* **B — the next session's work, DECIDED not proposed.** Give the habitat a real
  atmosphere: total gas as a stock, pressure as a state, the leaf reading partial
  pressures rather than mole fractions over a constant. That is a frozen-science change
  with the full ceremony, and A exists partly to be the instrument that measures it.

---

## 1. What was wrong with my framing, twice

**Correction 1 (mine, caught by the advisor).** I told the user this was *"genuinely new
ground."* It is not. `rust/crates/station/tests/perturbations.rs` is 495 lines and 10 tests;
five composers live in `rust/crates/station/src/perturbations.rs` and the generic primitives
(`window_override`, `with_forcing`, `LeakFlow`) in
`rust/crates/domains/src/biosphere/perturbations.rs`. The harness exists.

**What is actually missing** is measured, not felt: `grep -rln "perturbations::"` over
`rust/crates` returns seven files and **none of them is in `domains/tests/`**. Every
perturbation in this project is a *station cascade* — energy (brownout, radiator) and matter
(leak, crew spike, lighting). Not one of them perturbs the plant science. So the work is
**extend, biosphere-side**, which is cheaper than "build a suite" and is the half that the
O₂ inversion actually implicated.

**Correction 2 (mine, caught by writing this section).** I told the user that a physically
correct hull breach *"would change nothing at all here"* because the model reads mole
fractions. That is right about the **bookkeeping** and wrong as stated about the **leaf**.
Partial pressure is mole fraction × total pressure, so halving the total pressure and halving
both mole fractions are *the same thing at the leaf*. The model can therefore express a
depressurization's effect on photosynthesis perfectly well — by dropping the fractions. What
it cannot do is **arrive** there on its own, because nothing conserves total gas.

The precise statement, which §2 is built on:

> The biology reads `co2_mol / chamber_air_mol` and `o2_mol / chamber_air_mol`.
> `chamber_air_mol` is a **constant carried on the flow** (`chamber_air_mol: Option<f64>`,
> `rust/crates/domains/src/biosphere/flows.rs:65`), set to **9500.0** at
> `rust/crates/station/src/scenario.rs:91`. There is no total-gas stock, no nitrogen, no
> pressure, anywhere in the tree.

⚠ This is not news to the params — `eclss.yaml`'s `o2_setpoint` source string already
carries the trigger: *"this value silently encodes the ONE cabin air inventory within
O2Makeup's reach (9500 mol) … If an ECLSS is ever wired to a cabin whose air is not 9500
mol, this MUST become a mole fraction and the flow MUST read the air."* B trips exactly
that trigger. It was written 2026-09-06 and is two days old at the time of writing.

⚠ And the tree already knows the invariance, in a comment nobody connected to this:
`consumer_chamber_scenario` was enlarged 2× and its note says *"All three gas quantities
scale by the same factor so Ci0 (250) and x_O2 (0.21) both stay invariant."* That is E1
below, applied by hand as a sizing choice, with its invariance asserted in prose and
**checked by nothing**.

## 2. The selection rule — why these probes and not "what can we break"

The advisor's reframe, taken: the finding that earned this work was not *"we lack tests"*.
It was **a true claim went false and the nominal roster could not see it**
(`o2_leak_is_absorbed_by_makeup_effort` asserted the crop was untouched at a 10715×
contrast; correct while `photosynthesis.o2` was a constant, wrong the moment the pool got a
second path). So the criterion is:

> **Which settled verdict rests on the word "inert", and does its subject have a route into
> the biology that the nominal roster keeps quiet?**

⚠ **Reconciliation a fresh reader will need.** The third direction plan's §3 is a
do-not-re-propose list, and the canopy regulator sits on it as *"inert on the chambers"*.
This plan **re-proposes no mechanism**. §3 forbids bringing the mechanisms back; §2.3 of the
same document explicitly licenses asking whether a *verdict* is scoped to an observable the
nominal roster silences — *"that roster is not the model's behaviour space."* Those are
different questions and this plan asks only the second.

## 3. The three probes

The axis is **gas composition**, because that is where the O₂ inversion happened and where B
is going. Each is a contrast against an unperturbed baseline, direction-only.

### E0 — the arithmetic, with no run at all

Pure-function tests on `rust/crates/domains/src/biosphere/science.rs`:

* `light_limited_rate(ci, j, gamma) = j·(Ci − Γ*) / (4Ci + 8Γ*)` is **homogeneous of degree
  zero** in `(Ci, Γ*)`: scale both by `f` and top and bottom scale by `f`. So the
  light-limited branch is **exactly blind to a proportional change in both gases.**
* `rubisco_limited_rate = Vcmax·(Ci − Γ*) / (Ci + Kc(1 + O/Ko))` is **not**: the numerator
  scales by `f`, the denominator does not, because `Kc` is a constant term. So the
  Rubisco-limited branch **loses** as `f` falls.

This is the mathematical fingerprint of the whole question, it costs nothing, and it is
checkable exactly rather than by a run whose co-limitation hides which branch answered.

### E1 — the bookkeeping-correct vent (all three scale)

Scale `chamber_co2_mol0`, `chamber_o2_mol0` **and** `chamber_air_mol` by a common `f`.
Composition is unchanged by construction; the chamber is simply smaller.

### E2 — the depressurization as the leaf sees it (gases only)

Scale the two gas pools, leave `chamber_air_mol` fixed. Both mole fractions fall by `f` —
arithmetically identical, at the leaf, to the total pressure falling by `f` at fixed
composition. This is the probe that answers the user's question.

### E3 — the same under regulators (station), IF the harness allows it cheaply

`with_station_leak` takes **one** pool and one sink whose id is a single constant
(`LEAK_SINK`), so leaking two pools collides on the sink id. A two-gas leak needs per-pool
sinks. Deferred by default; see §6.

## 4. PREDICTIONS — written before any run, to be scored in §5

**Numbered so a miss cannot be quietly reworded. §5 is written only after the runs.**

1. **E0-a:** `light_limited_rate` is invariant under `(Ci, Γ*) → (f·Ci, f·Γ*)` to within
   floating-point rounding (≤ 1e-12 relative), for every `f` tried. *Not* bit-identical —
   numerator and denominator each round — so the tolerance is a real one and stated as such.
2. **E0-b:** `rubisco_limited_rate` under the same scaling is **strictly decreasing in
   `f`** downward from 1.0, i.e. `f < 1` strictly lowers it. Monotone, not merely different.
3. **E1:** with all three scaled by `f = 0.5`, the **initial** assimilation rate is unchanged
   (composition is identical), but the **season** is not: the same plant draws on half the
   absolute pool, so depletion is faster and the season-low CO₂ ends **lower** (in ppm) than
   baseline. Direction: season-low ppm falls. Confidence: high on the mechanism, moderate on
   whether the frozen jar has enough season left for it to show.
4. **E2:** with the two gases scaled by `f = 0.5` and air fixed, the crop ends **below**
   baseline — because the jar spends part of its season Rubisco-limited, and E0-b bites
   there. ⚠ **This is the prediction I am least sure of and it is the interesting one.**
   The competing outcome is near-invariance, which would mean the jar is light-limited
   essentially throughout and that halving its atmosphere is nearly free — a strange and
   important thing to learn, and the reason this probe is worth running rather than reasoning
   about. **Either result is a finding; neither is a failure.**
5. **E2 vs the O₂-only leak:** the two must differ in **sign** on biomass. The O₂-only leak
   raises the crop (measured, 2026-09-07: +9.8 % on the station). E2 drops O₂ *and* CO₂, and
   the CO₂ term is the one the crop is actually short of, so E2 must not come out positive.
   If E2 also raises the crop, the CO₂ path is weaker than the O₂ path at these levels — a
   result that would directly contradict `carbon_leak_lowers_biomass_and_scrubber_effort`'s
   ordering and would need chasing, not recording.
6. **Falsifier reachability:** every new whole-run assert can be made to fail by flipping the
   sign of the scaling (`f > 1` instead of `f < 1`). Any assert that stays green under both
   is inert and gets deleted or re-posed in this same batch, not filed as a follow-up.

## 5. MEASURED — written after the runs

Suite: `rust/crates/domains/tests/gas_composition_perturbations.rs`, **11 tests, green**.

### Scoring the six predictions

| # | prediction | verdict |
|---|---|---|
| 1 | `Aj` invariant under proportional scaling, ≤ 1e-12 rel | ✅ **HELD** exactly as stated |
| 2 | `Ac` strictly monotone in the scale | ✅ **HELD**, and the `Kc = 0` control confirms the term is the cause |
| 3 | E1 season-low CO₂ falls as the room shrinks | ⚠ **HELD ONLY IN THE WELL-POSED REGIME**, and the exception is the batch's second finding |
| 4 | E2 lowers the crop | ✅ **HELD** — −6.70 % at `f = 0.5`, monotone over 0.25–1.5, never rationing |
| 5 | E2 must not raise the crop as an O₂-only leak does | ✅ **HELD**, and the *reason* was wrong — see finding 3 |
| 6 | every whole-run assert reversible by `f > 1` | ✅ **HELD**; both directions are asserted in-test |

### Finding 1 — the gated observable REVERSES past the rationing cliff

| E1 factor | backstop firings | season-low CO₂ (ppm) | peak crop |
|---|---|---|---|
| ×1.0 | 0 | 7.29454 | 0.409954 |
| ×0.9 | 0 | 6.96120 | 0.407324 |
| ×0.8 | 0 | 5.70008 | 0.404716 |
| ×0.75 | **2** | 5.07224 | 0.403419 |
| ×0.6 | **13** | 6.20512 | 0.399593 |
| ×0.5 | **28** | **7.43317** | 0.397040 |

`season-low chamber CO₂` is the quantity the biosphere contract gates the jar on. It falls
correctly until the arbitration backstop starts firing, and then **comes back up** — a jar
broken enough to ration at ×0.5 reads *above its own healthy baseline*. **Peak biomass stays
monotone across the same range**, which makes it the honest observable off the nominal roster.

⚠ The backstop is not misbehaving; the golden runs assert its firing count is 0 exactly so the
reference never sits here. What is new is that the observable's **direction** is unsafe outside
that assumption. That is a property of the metric, not of the run, and nothing recorded it.

### Finding 2 — the self-limiting feedback is what keeps the jar well-posed

E1 rations at `f = 0.75`. E2 does **not** ration at `f = 0.25`, holding a third as much carbon
in absolute moles. Dropping the mole fraction lowers `Ci`, so demand falls with supply — the
chamber self-limits. E1 holds the fraction, and therefore the appetite, while removing the
buffer the appetite draws on.

⚠ Same mechanism as the CO₂-controller result the third direction plan records under its §2.2
(holding the chamber at its starting CO₂ measured four times worse than letting it deplete,
because the controller removes the feedback) — **reached by an unrelated route.** Two
independent arrivals at one mechanism is the strongest evidence in this batch.

### Finding 3 — the oxygen sign is scoped to REGULATION, and my first explanation was wrong

Halving O₂ alone: **jar −1.45 %, big perennial chamber −0.86 %.** Both **negative** — the
opposite of the station's +9.8 % under an O₂ leak.

Oxygen enters twice: at the leaf (`Γ*`, the Rubisco denominator) and at the soil
(`oxygen_limitation_factor`, `x/(K+x)`, `K = 1e-4 mol/mol`, on decomposition and microbial
respiration). In a sealed chamber the decomposers **are** the CO₂ supply. The station's cabin
is **defended** by `O2Makeup` at 209.8 mmol/mol, so its supply path never bites and only the
leaf path answers.

⚠⚠ **A control refuted my first reading inside this batch.** I wrote *"the jar's oxygen is
scarce, the big chamber's is not, so the big chamber should invert"* — the big chamber went
**down too**, at a trajectory-minimum soil factor of 0.999040 against 0.999521, a 0.048 %
throttle against a 0.86 % crop loss. **The distinguishing feature is regulated vs unregulated,
not big vs small.** The big chamber's route is 18× its throttle and is **NOT explained**;
compounding over its perennial years is the obvious candidate and has not been run. It is
recorded as open in the test's own docstring rather than guessed at.

⚠ And in the jar the two gas cuts **compound rather than fight**: −4.95 % (CO₂ alone), −1.45 %
(O₂ alone), −6.70 % together. The "assimilation traded against photorespiration" intuition —
which is what prompted this whole axis — is a *station* intuition and is false of the chambers.

### Finding 4 — the jar is one halving from anoxia, and the model takes it gracefully

At `f = 0.5` the jar's O₂ bottoms at **5.1e-15 mol** against 1.5e-1 on the baseline — a factor
of 3e13 — and the soil factor collapses with it. Across 3661 steps the stock is **never
negative**, the backstop **never fires**, no event is raised. First-order donor control means
the draw vanishes with the pool, so it reaches zero smoothly instead of overshooting into a
clamp: the positivity contract behaving exactly as its docstring claims, at the first place in
this batch that actually tested it.

The healthy jar's own trough sits at a soil factor of **0.606** — already deep in the limiting
region. Its oxygen charge is not a margin; it is the edge.

### The mutation battery

Five mutations to `science.rs`, each applied alone, run with `--no-fail-fast`, reverted after:

| mutation | tests reddened |
|---|---|
| M1 `Kc` term folded into `Ci` (Ac made homogeneous) | 7 of 9 |
| M2 `Aj` denominator `+1.0` (homogeneity broken) | 1 — **only** the `Aj` invariance test |
| M3 `Ci` ignores `air_mol` (hardcoded 9500) | 3 — the air-reading tests |
| M4 `Γ*` stops scaling with O₂ | 3 |
| M5 soil O₂ factor forced to `1.0` | 5 |

Every test reddened under at least one mutation **except** `the_gas_perturbations_are_
deterministic`, which is **inert by nature**: a deterministic mutation cannot make a
determinism check fail, and only a nondeterminism-introducing change could. Stated rather than
shrugged at.

⚠ **One honest gap.** The O₂ *sign* assert has no demonstrated direct falsifier. Every
mutation that changes the oxygen path enough to flip it (M5, and a sharper M6 freezing the soil
factor at the charge fraction) breaks well-posedness *first*, so the test reddens on its
backstop guard rather than on its sign. The sign's evidence is instead the **positive control**
— the same probe run on a second scenario, which produced finding 3. That is weaker than a
mutation and is recorded as such.

## 6. Scope, and what is deliberately not here

* **No golden, no manifest, no unfreeze.** These are diagnostics — the module header states
  the precedent twice and names determinism re-runs as the insurance. That is what makes A
  cheap, and it is the reason A is separable from B at all.
* **No new science gate.** ⚠ Closing the third direction plan's *"the station census carries
  no CO₂ or cabin-oxygen band"* would mean adding a row to `station::science_gates!`, and
  that row lands in `docs/station-reference.manifest.json` — **an unfreeze with a ceremony**,
  not a free consequence. The advisor's *"no manifest"* and its *"take the census gap first"*
  are in tension, and the tension is resolved in favour of the cheap half: A builds the
  **subject** (a perturbed run where cabin gas actually deviates); promoting it to a census
  row is a separate slice with its own ceremony, and it is better taken **after** B, when the
  quantity it would freeze is the one B leaves behind rather than the one B replaces.
* **E3 deferred** unless E1/E2 land with time to spare. Per-pool leak sinks are a harness
  change to a shared composer, and a shared composer touched for a probe is how a diagnostic
  turns into a refactor.
* **Not proposed, and named so it is not mistaken for an omission:** PAR/`Vcmax` branch
  crossing and a temperature excursion. Both are real and both are the advisor's next two;
  they are the natural second batch and they are better run once E0 has said which branch
  binds where, because that is the fact they both turn on.

## 7. Filing

Index line + pointer row + `docs/log/perturbation-suite.md` + a memory file. Nothing in
`CLAUDE.md`. **The plan doc lands in the same commit as the first probe**, so it never needs
an exemption paragraph in the log — that file is a graveyard of exemptions that outlived
their premise, one of them false for seventeen days.

⚠ **The re-read that comes due.** Landing this appends a row to the record table, and
`repo_gates` asserts the third direction plan's re-read marker names the table's **last**
row. That owes an actual re-read — strike §2.3's two named non-decisions (this plan is the
answer to one of them), record B as decided — not a marker bump. The gate cannot tell the
difference; the diff reader can.
