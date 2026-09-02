# The O₂ the jar cannot see — the FvCB item's first form gap, measured

**Taken 2026-09-02**, the day the FvCB provenance item landed, as item 2 of the September
direction plan's recommended order: *"the `o2` measurement through the value switch —
minutes; decides whether the jar's bands are sensitive to a form the tree does not carry."*

It is a **measurement**, not a build. Nothing in `rust/crates/domains/params/` moved, no
golden moved, no manifest key moved, and no decision was taken.

| | Outcome |
|---|---|
| Is the jar's band sensitive to the form? | **YES** — and in the *opposite* direction to the one the gap's own wording implies |
| Building the oxygenation half alone | **REFUSED as a recommendation** — it cuts the jar's headroom above the floor by 43 % |
| Building both halves together | the band becomes ~10× *safer*, and thereby stops discriminating (§5) |
| The other two sealed chambers | **INERT by arithmetic**, not by measurement — they sit at 210.2 mmol/mol |
| The open field | the oxygenation term is **unreachable** there — but by only ~15 % of Vcmax (§3) |
| The temperature form (the plan's item 3) | **RE-PRICED** — *if* it cuts Vcmax at the cool end (unverified, §3), it crosses the branch boundary this item measured |
| The harness | **EXTENDED** — it could not spell the one column the recommendation rests on (§6) |

The record is `docs/log/o2-coupling-measured.md`.

---

## 1. What the jar actually breathes, against what its photosynthesis reads

`rubisco_limited_rate` is `Vcmax·(Ci − Γ*) / (Ci + Kc·(1 + O/Ko))`, and `O` is the frozen
`o2 = 210.0 mmol/mol` in `photosynthesis.yaml`. Every sealed scenario meanwhile carries
`biosphere.o2_pool` as a live stock. Read off `system.rs` and the goldens:

| scenario | air (mol) | O₂ charge (mol) | fraction at charge | O₂ at golden's end | fraction at end |
|---|---|---|---|---|---|
| `sealed_chamber` | 1000 | **2.0** | **2.0 mmol/mol** | **0.033185896524892 mol** | **0.033 mmol/mol** |
| `perennial_chamber` | 1000 | 210 | 210.0 mmol/mol | 210.24 mol | 210.24 mmol/mol |
| `consumer_chamber` | 2000 | 420 | 210.0 mmol/mol | 420.46 mol | 210.23 mmol/mol |

**The frozen constant is exactly right for two of the three sealed scenarios and wrong by
three orders of magnitude for the third** — 105× at the jar's charge, 6329× at its end. And
the third is `sealed_chamber`, the scenario that exists *to show O₂ depletion*: its own
docstring calls it "the O₂-poor sealed chamber" and its litter charge was re-sized in
August specifically to keep the pool bottoming below 5 % of its fill.

So the scenario built to demonstrate O₂ depletion is the one whose photosynthesis is
constitutionally unable to see the depletion.

## 2. The form moves TWO numbers, and only one of them is the gap as written

The direction plan's wording — *"`o2` is a constant; chamber O₂ is a stock"* — names the
oxygenation denominator. But O₂ enters FvCB twice:

1. the Rubisco denominator, `Kc·(1 + O/Ko)`;
2. the photorespiratory compensation point, `Γ* = 0.5·O/(S_c/o)`, i.e. **Γ* ∝ O₂**.

⚠ **That proportionality is a derivation from the standard FvCB relation, not a retrieved
number.** It is used here only to place the second column at a physically coherent value;
nothing in the tree moved on it. At the jar's charge it gives `Γ* = 42.75 × 2/210 =
0.4071`, and at the jar's end `42.75 × 0.033/210 = 0.006718`.

### The columns

All from `cargo run --release -q -p domains --example value_switch`, on the committed tree.
`o2=210` and `gamma_star=42.75` were run as **round-trip controls** and reproduced the
frozen column to every printed digit on all eight rows.

| column | `sealed_chamber` season-low CO₂ (ppm) | the floor `Γ*/ci_ratio` (ppm) | ratio |
|---|---|---|---|
| frozen | 71.435803 | 61.071429 | **1.1697** |
| `o2=2` alone (the gap as written) | 66.924275 (−6.3 %) | 61.071429 (unmoved) | **1.0958** |
| `gamma_star=0.4071` alone | 13.511831 | 0.581571 | 23.23 |
| **`o2=2 + gamma_star=0.4071`** (the form) | **7.183490** (−89.9 %) | **0.581571** | **12.35** |
| `o2=0.033 + gamma_star=0.006718` (at the end) | 6.574840 | 0.009597 | 685.1 |

**FINDING — the two halves move the band in opposite directions, and the floor's half wins
by an order of magnitude.** Headroom above the floor goes 10.364 ppm frozen → **5.853 ppm**
with the oxygenation half alone (a **43 % cut**), and → a ratio of 12.35 with both. Building
the half the gap's wording names, on its own, is the *only* way to make the jar's band
tighter; building the whole form makes it about ten times looser.

⚠ **A half-built form here is worse than the frozen constant it replaces.** That is the
finding, and it is not what the gap's own description would lead a reader to expect.

Also measured: `o2=2` and `o2=0.033` are near-identical on every row (66.924275 both, on
the jar). **The oxygenation term saturates by ~2 mmol/mol**, so `1 + O/Ko → 1` and there is
nothing left to vary. All of the form's time-dependence lives in Γ*, none of it in the
denominator the gap was written about.

## 3. The control — and it re-prices the temperature form

The obvious reading of §2 is "and the open field?" It does not move at all:

| perturbation | `open_season` peak LAI |
|---|---|
| `o2` 210 → 2 (a 43 % cut in the Rubisco denominator) | **+0.000 %** |
| `vcmax` +10 % | **+0.000 %** |
| `jmax` +10 % | +0.756 % |
| `quantum_yield` +10 % | +1.338 % |

`vcmax` and `o2` both live on `Ac`; `jmax` and `quantum_yield` both live on `Aj`. One family
is inert and the other is not, so **the open field runs on the light-limited branch and the
whole oxygenation term is unreachable there** — not damped, unreachable. The observable
itself is live (Γ* moves it +13 %), so this is a fact about the branch, not a clamped output.

⚠ **But the headroom is thin, and this is the part that matters for what comes next:**

| `vcmax` | 100 (frozen) | 90 | 80 | 70 | 60 | 50 |
|---|---|---|---|---|---|---|
| peak LAI | 6.022837 | 6.022837 | 5.981236 | 5.649397 | 4.890622 | 3.488340 |
| vs frozen | — | **+0.000 %** | −0.69 % | −6.20 % | −18.80 % | −42.08 % |

**The crossover is between 80 and 90 — about 15 % below the shipped Vcmax.** So "the Rubisco
branch is unreachable in the open field" is true of *this* Vcmax and has almost no margin.

The September plan's item 3 is the Arrhenius temperature form of Kc, Ko and Γ*, and the open
field runs 5–25 °C through a season. ⚠ **Whether that form also moves Vcmax is NOT established
here.** The cited paper is *understood* to give Vcmax a temperature response as well, but no
Vcmax Arrhenius parameters have been retrieved — that is the same class of unretrieved claim
as Γ* ∝ O₂ above, and the page check the FvCB item owes is the thing that would settle it.
What is measured is only the ladder below. **Conditionally, then:** *if* the form cuts Vcmax
by more than ~15 % at the cool end, it **moves the tree across this branch boundary** — which
means it does not merely "raise assimilation at 15–25 °C" as the plan predicts, but switches
which limitation is binding, and switches the oxygenation term this item just measured as
unreachable *into* reachability. That is a different and larger
claim than the plan carries, and it should be predicted before that build, not discovered.

Second-order, worth recording: `vcmax=50` also puts `perennial_long_horizon`'s converged
peak-leaf at **0.545058, below its 0.55 liveness floor**. The temperature form's cool-end
behaviour has a floor to clear as well as a band.

## 4. ⚠ What these columns do NOT say

The value switch substitutes **globally** — one value for every scenario. So:

* **`perennial_chamber` and `consumer_chamber` sit at 210.2 mmol/mol.** Their `o2=2` and
  coupled columns are counterfactuals, not consequences of the form. Under the form those
  two scenarios move by ~0.1 %, which is the +0.1 % drift already in the table in §1.
* **The `perennial_long_horizon` liveness-floor break at `gamma_star=0.4071` (0.578137 →
  0.417240, against a `> 0.55` bound) is a counterfactual too.** That scenario's Γ* does not
  move under the form. It must not be reported as a form consequence, and is recorded here
  only as a sensitivity.
* **Under the real form, exactly one gated row in the whole table can move:**
  `sealed_chamber / season-low chamber CO₂`.

## 5. The band this was checked against becomes uninformative under the form it recommends

`min > Γ*/ci_ratio` is written against a **constant** floor — that is what lets it be a
single number, 61.071429 ppm, computed once. Under the form, Γ* is a function of a live
stock, so the assertion becomes `CO₂(t) > Γ*(t)/ci_ratio` **pointwise in time**. That is a
different assertion, not a re-tuned one, and it is not what the five frozen
`..._stays_above_the_compensation_point` gates say.

And it would not discriminate: the jar's ratio goes 1.17 → 12.35 at charge and → 685 at the
end. A guard that passes by three orders of magnitude is not measuring anything.

**So the build this measurement supports would retire the usefulness of the guard that
motivated the measurement.** That is the "built, and inert on the chambers" shape this
project has hit before (the canopy regulator; the parked leaf mechanism). It is the build's
real cost, and it is named here rather than discovered after.

## 6. The harness could not spell the column the conclusion rests on

`report::compare` has always taken `Vec<(String, Vec<Substitution>)>` — a column may carry
any number of substitutions. But `examples/value_switch.rs` parsed every spec into
`vec![sub]`, one substitution per column. So the coupled column in §2 **could be argued
across two single-parameter columns and could not be measured**, and the difference between
those two things is the difference between evidence and arithmetic done by the reader.

Fixed by lifting the grammar out of the example binary into `domains::lab::parse_variants`:

* `,` still sweeps one target into one column per value;
* `+` joins several targets into **one** column — `o2=2.0+gamma_star=0.4071`;
* mixing `,` and `+` in one spec is **refused**, because `a=1,2+b=3` has two readings and a
  harness that picks one produces a table whose caption is wrong invisibly;
* a malformed part (`a=1+`) is an error rather than degrading to a single-substitution
  column, which would *read* as the coupled measurement and not be one.

The grammar was inline in an `examples/` binary until now, which means the one thing it
could get wrong was reachable by no test at all. Five tests were added; the mutation that
collapses a `+` column into two independent ones reddens two of them.

⚠ **A tooling change inside a science batch, named rather than slipped in** — the same shape
as the FvCB item's own FINDING 2, and for the same reason: the measurement could not be made
without it. It re-anchors nothing: no contract, golden, manifest key or param moved.

## 7. Recommendation — the build is the user's call

1. **Build both halves or neither.** The oxygenation half alone is the only version that
   tightens the jar's band, and it is the version the gap's wording invites.
2. **Re-pose the band first.** A time-varying Γ* makes `min > Γ*/ci_ratio` ill-posed as
   written; without a pointwise successor the build lands with no gate that can see it.
3. **Predict the branch crossing before the temperature form (§3), not after.** ~15 % of
   Vcmax headroom is what stands between the open field and a different limiting process.
4. Only `sealed_chamber` can move. This is one scenario's science, not the roster's.
