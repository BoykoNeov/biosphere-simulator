# Post-roadmap — a source for "plants take less as the CO₂ runs low" (Step 2, option C)

**Opened 2026-09-30**, on the user's call *"2"* — the second of four offered next items: *"just
find the source for C"*. Option C of Step 2 in `docs/plans/post-roadmap-review-2026-09-29.md`
(*"make the draw self-limiting"*), which the user wants built regardless (*"this should be
implemented at some point for sure"*). The review's rule decides which side of the line C lands
on: a published form, or it is WHAT-IF and lab-only. Record: `docs/log/co2-uptake-source.md`.

**Scope: a literature search.** No code, nothing frozen touched, no option priced.

## 1. The question, re-framed before searching

The crop **already** takes less as the CO₂ runs low. Its uptake is FvCB
(`domains/src/biosphere/science.rs`, `rubisco_limited_rate` / `light_limited_rate`), both limbs
carrying a `(Ci − Γ*)` factor, so uptake falls to zero at the compensation point; `Ci` is read
off the live pool (`ci_from_co2_pool`). What is missing is **inside a step**: the rate is read
at the step's start and held for the whole quarter-day, so the crop keeps drawing at the rate the
full pool set. The review's wording of C — *"computed against the air it would leave behind"* —
is a change to how the step is calculated. So the search has two halves:

* **(a) Biology** — a published law for how a canopy's uptake falls as a sealed room's CO₂ is
  drawn down. What the user asked for.
* **(b) Numerics** — a published method for taking a draining flow over a step against the end
  state. A numerical-methods source, not a plant paper.

A usable (a) source must say which concentration it is on (chamber `Ca` or leaf `Ci` — the model
reads `Ci` through a fixed ratio 0.7), whether it is per leaf or per stand, and whether it is a
rate law or a curve fitted to one chamber.

## 2. Predictions — stated before the search (the advisor's, 2026-09-30)

⚠ Stated in the conversation before searching; this file was written after the search and
committed with the results, so the commit history does not show them first.

* P1: our own shelf holds no drawdown form.
* P2: the literature gives only the straight line near the compensation point (uptake ∝
  `Ci − Γ*`), which is the slope of the FvCB already built and so adds nothing; C is then
  purely a numerics question.

## 3. Results

Graded and written out in `docs/log/co2-uptake-source.md`. In short: P1 held. P2 half held — no
new *law* was found, but the search found a **measured whole-stand drawdown curve** (NASA KSC,
Wheeler & Sager 1990), which is a validation target, not a form. Half (b) found a candidate,
the modified Patankar–Euler scheme (Burchard, Deleersnijder & Meister 2003, unread), which keeps
pools non-negative but is **not** C: C taken literally is backward Euler on the crop's flow.

## 4. What this does not do

* It does not build C, and does not choose between A, B and C (Step 2 slices 2–4).
* It does not compare the model with the NASA curve — that is Step 6 work, and needs the
  chamber's geometry replayed in the lab.
* It does not read Burchard et al. 2003 itself (paywalled); the scheme is taken from an open
  paper co-written by one of its authors (Meister) that restates and attributes it. Read the original before
  any reference use.
