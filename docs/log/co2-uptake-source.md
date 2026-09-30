## **A source for "plants take less as the CO₂ runs low"** (the 2026-09-29 review's Step 2, option C — the biology was already built; what is missing is inside the step, and that half has a published scheme)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**SEARCHED 2026-09-30 on the user's call** (*"2"* — *"just find the source for C"*). A
literature search only: no code, nothing frozen touched, no option priced. Plan:
`docs/plans/post-roadmap-co2-uptake-source.md`. ⚠ Its two predictions were stated in the
conversation before the search, but the plan file was written after it and committed with the
results — unlike the draw census, whose predictions were committed first (`6aa7329`).
Downloads kept outside the repo, under `W:\temp\claude\co2-drawdown\`.

## Finding 1 — the crop already takes less as the CO₂ runs low; C is about the inside of a step

Uptake is FvCB, and both of its limbs carry `(Ci − Γ*)`, so uptake falls to zero at the
compensation point, read off the live pool. What is missing: the rate is read at the step's
start and held for the whole quarter-day. The review's wording of C (*"computed against the air
it would leave behind"*) is a change to how the step is taken, not a new biology formula. So
"find the source" had two halves — a biology law (a) and a numerical scheme (b).

## Finding 2 — (a) biology: a measured curve, not a new law

**Our shelf: nothing** (P1 held). `Wheeler` appears only as the 1993 CO₂-setpoint citation.

**The web — Wheeler, R.M. & Sager, J.C. (1990), *Carbon dioxide and water exchange rates by a
wheat crop in NASA's Biomass Production Chamber: results from an 86-day study (January to April
1989)*, NASA TM 102788, Kennedy Space Center** (NTRS 19900016137; open PDF). A 20 m² wheat stand
in the sealed 113 m³ chamber, at ~25 days after planting, CO₂ raised to 2200 ppm and left to draw
down:

* **Fig. 2 (p. 19):** chamber CO₂ against time, ~2400 → ~130 ppm over ~9 hours. The pull is
  nearly steady down to ~800 ppm and then tails off, flattening well above zero.
* **Fig. 3 (p. 20):** stand net CO₂ uptake (µmol m⁻² s⁻¹, derived from Fig. 2's slope) against
  chamber CO₂: about 20–23 above ~900 ppm, falling steeply below ~500, near zero by ~115 ppm.
  Fig. 3's caption (p. 16): *"Photosynthetic rate increased rapidly from 115 ppm up to about
  700-800 ppm, and then increased slowly"*; the text (p. 7) derives it as *"the 1st derivative
  of a polynomial fit to Fig. 2"*. (Page numbers are the report's printed ones.)
* Locus check: **chamber** CO₂ (Ca), **whole stand** (net, so stand respiration is inside it),
  **one chamber and one crop**, fitted with a polynomial — a curve, not a rate law.

So P2 half held: no new law. The published description of a stand in a sealed room is the
stand's uptake–against–CO₂ curve, and the model computes that curve mechanistically (FvCB
through the canopy layers) already. What the report *is*: **a validation target** for the
review's Step 6 — replay the chamber's area, volume and light in the lab and compare with
Figs. 2–3. Not done here.

Side reading, also no law: Campbell, Sage, Kocacinar & Way (2005), *Global Change Biology*
11:1956–1967 — growth at low CO₂; it records leaf uptake as a straight line in chamber CO₂ from
50 to 270 ppm (p. 1959), which is P2's near-compensation slope, and a whole-plant compensation
point (~65 ppm) above the leaf's (~50).

## Finding 3 — (b) numerics: a candidate scheme, and it is NOT C

**Burchard, H., Deleersnijder, E. & Meister, A. (2003), *A high-order conservative Patankar-type
discretisation for stiff systems of production–destruction equations*, Applied Numerical
Mathematics 47(1):1–30, doi:10.1016/S0168-9274(03)00101-6.** Built for biogeochemical models:
unconditionally positive and conservative at any step size. Its first-order member (MPE)
*"multipl[ies] the destruction terms with weights that comprise yᵢⁿ⁺¹ as a factor"*.

⚠ **That is not C.** (Corrected 2026-09-30 after review; the first commit said "C's wording
exactly".) MPE still reads the rate at the step's **start** and scales it by
end-pool ÷ start-pool. For the crop's draw, `C₁ = C₀ / (1 + h·A(C₀)/C₀)`: the implied uptake
`A(C₀)·C₁/C₀` reaches zero only as the pool reaches **zero**, while FvCB's reaches zero at the
**compensation point**. So a big step can still carry the pool below the compensation point —
it guarantees a non-negative pool, not a right amount (the "a clamp hides a wrong amount"
lesson, in a new place). C taken literally — uptake *"computed against the air it would leave
behind"* — is **backward (implicit) Euler** on that flow: solve `C₁ = C₀ − h·A(C₁)`. For a
single-donor flow with `A` rising in `C`, that is one monotone scalar solve, and `A(Γ) = 0`
keeps `C₁` at or above the compensation point. Backward Euler is textbook; no source was
sought for it here.

⚠ Read from Kopecz & Meister (2018 preprint arXiv:1703.05052, §1, Definition 1.4 and eqs. 5–7),
which restates the class and attributes MPE to that 2003 paper. The 2003 paper itself is
paywalled (one more search found no open copy; ResearchGate is request-only) and was **not**
read. It is a **candidate known only from a restatement**, not a source.

What the restatement says that matters here:

* *"Weighting only the destruction terms will result in a non-conservative scheme"* — the
  production terms are weighted by their **donor's** end-of-step ratio too, and each step is an
  N×N **linear solve** over every stock. So MPE is an **integrator**, not a change inside one
  flow. It would replace the frozen "Euler, dt = ¼" key → a biosphere unfreeze.
* **Where it would live — probably `simcore`, which Step 2 forbids editing.** Checked, not
  built: `simcore::integrator`'s `Scheme` trait is private, and every biosphere call site is
  typed `&EulerIntegrator` (`domains/src/biosphere/system.rs`). A new scheme either edits
  `simcore` or re-assembles the step (arbitration, extinction, the conservation gate, aux) from
  `simcore`'s public parts inside `domains` — whether that is possible is **open**. The
  flow-local backward-Euler form above needs neither: it lives inside one flow's `evaluate`.
* It needs flows written as pairwise donor→receiver transfers. A flow with one donor (the crop's
  CO₂ draw) maps cleanly; a flow with two donors does not obviously — **open**.
* It is first order: it stops a pool going negative, it does not make the step accurate. The
  review's slice 3 test (converge on a much finer step) still judges it.

**A cheaper cousin, NOT the published scheme:** scale a draining flow's whole leg set by
`1 / (1 + h·rate / pool)` inside the flow. That keeps every flow balanced and fits the existing
"arbitration scales the whole flow" idiom with no `simcore` edit — but it is not MPE (respiration's same-step
return to the pool is not seen, the same blind spot the arbitration backstop has), it has no
citation of its own, and it would be WHAT-IF until shown to converge.

## Predictions, graded

| | Prediction | Result |
|---|---|---|
| P1 | The shelf holds no drawdown form | **Held** |
| P2 | Only the near-compensation slope; C is purely numerics | **Half held** (below) |

P2: no new law, and C is numerics — but a measured whole-stand drawdown curve turned up, a
validation target P2 did not foresee.

## What it changes

* **Option C needs no biology beyond FvCB; it is a question of how the step is taken**, which
  puts C and B on the same footing for slice 2. Two forms: literal C (backward Euler inside the
  crop's flow — respects the compensation point, lives in `domains`), and MPE (a whole-system
  scheme, known only from a restatement, probably a `simcore` edit, and does **not** respect
  the compensation point). Neither is built or priced.
* **Step 6 gains a target:** NASA TM 102788 Figs. 2–3 (wheat, 113 m³, 20 m²).
* ⚠ Tuning trap, unchanged: either form can never overdraw, so it would silence the jar's 0.757
  and the leaf mechanism's 1.15 **by construction**. Silence proves nothing; only convergence does
  (`docs/log/leaf-expansion.md` finding 9).
