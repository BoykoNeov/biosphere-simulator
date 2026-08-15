# The canopy this tree cannot grow — the named candidate, measured and refused

**Taken 2026-08-15 on the user's call**, from the direction plan's §6b ("OPENED 2026-08-14
by the light path"), which called it *"the biggest open science item in the queue"*.

**DIAGNOSED 2026-08-15, NOTHING BUILT. `git diff src/` is empty, no golden regenerated, no
manifest touched, no parameter moved.** Every measurement below is a monkeypatch in a
throw-away probe under `M:/claud_projects/temp/canopy-layers/` — the frozen tree is the
control in every table, and every control reproduces its inherited anchor to the printed
digit before any candidate number is believed.

The decision this pass exists to inform is the user's and is **untaken**: see
[§7](#7-the-decision-and-it-is-the-users).

---

## 1. The charge, and the premise it rested on

The within-day light path (`post-roadmap-gross-net-gas-exchange.md`) gave the plant an
honest sun and cost it a canopy: `open_season`'s peak leaf-area index reads **5.3806** at
the shipped step and **4.7132** converged, against a `science_bands` bound of
`5.0 < peak < 8.0` sourced as *"real wheat peaks at ~5–8 LAI"*. It passes today only
because the observable is still moving 15 % between `dt = ¼` and `dt = 1/32`. That is
carried as a **known deviation** in `docs/biosphere-reference.md`, not tuned.

§6b eliminated three fixes by measurement (the canopy regulator — it is a *loss* above
LAI 6; the parked leaf mechanism — it sits *below* the frozen tree everywhere; re-tuning
the bound — refused three times in this project) and named one candidate that was not:

> the *intra-canopy* half of the Jensen bias is still open (this work closed only the
> diurnal half — a sunlit/shaded or multi-layer canopy is the cited next step **and can
> move canopy assimilation *up***).

⚠ **That clause is false, and the tree has said so since Phase 1.** `photosynthesis.py`'s
own module docstring, unchanged since Step 5:

> ``Ag`` is concave in PAR (saturating ``J``, then ``min``), so evaluating at a mean PAR
> **overestimates** the true integral (Jensen) — exactly why WOFOST does the
> intra-canopy/diurnal Gaussian.

Concave means Jensen runs one way only: `E[f(x)] ≤ f(E[x])`. Resolving the canopy into
depths **redistributes the same photons** onto a concave response and can only lower the
sum. The plan's clause did not survive being read next to the function it was about.

**So the first thing measured was the SIGN**, before any design — the discipline
`acceptance-gate-diagnosed` and `leaf-mechanism-converging-to-inert` both name: measure
the thing, do not rank it from its description.

---

## 2. Finding 1 — the named candidate is refuted, open loop and closed loop

**Open loop** (`probe1_sign.py`): wrap `canopy_assimilation` on a real `open_season` run at
the shipped step, recomputing each call as a 100-layer depth integral of the *same*
Beer–Lambert profile (absorbed PAR per unit leaf area at cumulative depth `L` is
`k·I₀·exp(−k·L)`, whose integral over `[0, LAI]` is exactly the `I₀·(1 − exp(−k·LAI))` the
big leaf absorbs — pure redistribution, no photon created or destroyed).

| reading | value |
|---|---|
| control peak LAI | **5.3806**, `rationed = 0` — the inherited anchor, to the digit |
| lit calls | 2598 of 3660 |
| season-integrated layered-100 / big-leaf | **0.943252** |
| season-integrated 3-point Gaussian / big-leaf | **0.930028** |
| per-call ratio, min … max | 0.883767 … 1.000000 |
| **calls with ratio > 1** | **0** — the pre-registered prediction, held exactly |

**Closed loop** (`probe2_closed_loop.py`): run the season *with* the layered aggregator, so
the loss feeds back through allocation into leaf carbon.

| step | big-leaf (frozen) | layered-100 | 3-point Gaussian |
|---|---|---|---|
| `dt = ¼` (shipped) | **5.3806** | 5.0314 | 4.9634 |
| `dt = 1/32` (converged) | **4.7132** | 4.4169 | 4.3757 |

Both controls reproduce their inherited anchors exactly. **The candidate moves peak LAI
down ~6.5 % at both steps** — at the converged step, from 4.71 to 4.42, further below the
5.0 floor, and at the shipped step it eats four fifths of the margin the band is passing
on.

⚠ **REFUSED as a fix for the canopy.** Not refused as *science*: a depth-resolved canopy is
more faithful than a big leaf, and the tree's own docstring has called the big leaf a
known high-bias for nine phases. What is refuted is the plan's claim about its **sign**.

## 3. Finding 2 — the cited cheap scheme is worse than the expensive one

The 3-point Gaussian is not a stand-in the project invented; it is what WOFOST and [A]
actually do, and it is the scheme §6b was pointing at. It comes in **below** the 100-layer
integral at both steps (4.9634 vs 5.0314; 4.3757 vs 4.4169) and below it open-loop
(0.9300 vs 0.9433).

So the quadrature error and the physics error **point the same way**. A reader who assumed
the cheap scheme was the conservative choice — the usual reason to reach for it — would
have been wrong by about a fifth of the total correction. Worth writing down because the
project reaches for cited cheap schemes routinely.

## 4. Finding 3 — it would NOT have taken a liveness floor red, and the reason generalises

§6b's own warning was that a mechanism lowering assimilation "may take the floor red",
perennial's converged peak-leaf sitting **3.2 %** over its bound on its settled value. That
is worth checking even for a refused mechanism, because "wrong direction **and** it breaks
a gate" is a materially stronger finding than "wrong direction".

**It is the weaker one.** `probe3_routes.py` measures the layering ratio against canopy
size, and `probe5_floors.py` runs `perennial_long_horizon` (15 yr, Euler, shipped step)
under the 3-point Gaussian — the *worst* of the schemes, so it bounds the others:

| LAI | layered / big-leaf | where |
|---|---|---|
| 0.07 | 0.999948 | chamber |
| 0.63 | 0.995965 | chamber |
| 1.00 | 0.990403 | field |
| 4.70 | 0.917150 | field |
| 6.00 | 0.900402 | field |

| floor | frozen | 3-point layered | bound |
|---|---|---|---|
| perennial `max(tail)` peak leaf | **0.603679** (+9.76 %) | 0.603540 (+9.73 %) | `> 0.55` |
| perennial annual-min chamber CO₂ | 0.0727724 (+45.54 %) | 0.0727769 (+45.55 %) | `> 0.05` |

The control reproduces the light path's re-pinned 0.603679 exactly. The candidate moves it
by **0.023 %** and moves the CO₂ floor *up* in the seventh digit.

⚠ **The reason is structural and it is the canopy regulator's reason, verbatim.** The
correction scales with canopy **closure**; every chamber peaks at LAI 0.07–0.63, where the
light profile has barely begun to decay, so there is nothing to resolve. A mechanism about
mutual shading is inert in a habitat whose canopy never closes — measured for the regulator
in 2026-07-27, and now measured again for a completely different mechanism arriving from a
completely different direction. **The damage is confined to `open_season`'s band.**

## 5. Finding 4 — the band's subject is a 3.5× amplifier, which limits what any band here can prove

`probe3_routes.py`, closed loop at the converged step, where clearing the floor costs
**+6.1 %** of peak LAI:

| knob | ×1.05 | ×1.10 | ×1.20 |
|---|---|---|---|
| specific leaf area | 5.5619 (**+18.0 %**) | 6.3481 (+34.7 %) | 7.9117 (+67.9 %) |
| gross assimilation | 5.5190 (**+17.1 %**) | 6.2737 (+33.1 %) | 7.7586 (+64.6 %) |

`rationed = 0` throughout. A 5 % nudge to either knob buys ~18 % of peak LAI: the canopy
**compounds at ~3.5×**, because more leaf area intercepts more light, which fixes more
carbon, which grows more leaf. Clearing the 6.1 % gap therefore costs about **+1.8 %** of
either quantity.

⚠ **This is the finding that reframes the whole item.** Three of the inputs that set peak
LAI are `TODO(cite)` provisional literals — `specific_leaf_area: 22.0`,
`extinction_coef: 0.6`, and the FvCB `vcmax`/`jmax` pair. At 3.5× amplification, a ±2 %
disagreement about any one of them is the entire distance to the bound. **A band whose
subject amplifies every parameter error 3.5× cannot arbitrate between mechanisms**; it can
only say the tree is in the right decade. Reading the 4.71 as evidence *about a mechanism*
— which is what "the canopy this tree cannot grow" claims — is reading it past its
resolution.

That does not make the deviation acceptable. It makes it a **provenance** finding rather
than a mechanism finding, which is a different queue.

## 6. Finding 5 — the SLA route has BOTH signs, and which one you get is the undocumented content of a `TODO(cite)`

§6b's second unmeasured candidate: *"the specific-leaf-area constant has no DVS keying
anywhere … which makes LAI strictly linear in leaf carbon"*. Every SLA table in the
literature **declines** with development (young leaves are thin, mature leaves are thick),
so keying the constant is one mechanism — but it is ambiguous about something the param
file never says: **what the frozen 22.0 m²/kg IS.** Nothing in `canopy.yaml` beyond
`"TODO(cite) — provisional, literature-typical"`.

`probe4_sla.py`, converged step, two crude linear ramps — **neither is a citation**; they
are a sensitivity test of the *form*, run deliberately before any source hunt so a source
hunt is not spent on a dead route:

| reading | peak LAI | DVS at peak | vs control |
|---|---|---|---|
| control — constant 22.0 | **4.7132** | **1.373** | — |
| early-anchored: 22.0 is the young value, table 22.0 → 15.0 | **3.0446** | 1.307 | **−35.4 %** |
| late-anchored: 22.0 is the mature value, table 29.0 → 22.0 | **8.2391** | **0.958** | **+74.8 %** |

`rationed = 0` in all three.

⚠ **The same mechanism spans 3.04 to 8.24 — a 2.7× range — on a question the parameter
file does not answer.** One reading is worse than the refused layered canopy; the other
clears the floor and overshoots the 8.0 ceiling. So "key SLA to development" is not a
proposal until the anchor is sourced, and the anchor is a **citation** question about a
provisional literal, exactly as finding 4 predicted the real blocker would be.

### 6b. And the late-anchored reading moves a *second*, independently-pinned defect the right way

Peak LAI on the frozen tree occurs at **DVS 1.373** — well *after* anthesis. Real wheat
peaks at or just before it, and this is not our reading of the literature: it is already
pinned in the suite as a known gap against **two independent oracles of different model
families**, `tests/test_oracle_gap_spring_wheat.py::test_gap_lai_peaks_after_anthesis_not_before`
(LINTUL3 peaks day 72 against its own anthesis at day 74; ours peaks ~13 days after ours)
with the same direction recorded against the WOFOST winter-wheat oracle.

Under the late-anchored ramp the peak moves to **DVS 0.958 — before anthesis**, where both
oracles put it.

⚠ **That was not what the probe was aimed at.** One candidate moving two symptoms that were
recorded separately, one of them by an oracle the candidate knows nothing about, is the
kind of corroboration this project counts — and it is the opposite of the fitted-table
shape (`wheat-partition-backfill-refused`), where a table passed a band *because* it had
been fitted to it. It is still only a **direction**: the ramp is crude, uncited, and
overshoots the ceiling.

---

## 7. The decision, and it is the user's

**Nothing is built and nothing should be built out of this pass without a call**, because
the two live options are not both "improvements":

1. **Build the layered canopy anyway, as fidelity.** It is more faithful physics and the
   tree has confessed the bias since Step 5. Price: it takes `open_season`'s peak-LAI band
   from marginal-pass (5.3806) to **5.0314 — a 0.6 % margin** at the shipped step, and
   deepens the converged failure to 4.4169. The liveness floors are unaffected (finding 3).
   ⚠ Shipping a mechanism that reds or nearly-reds a *sourced* band is a call that belongs
   upstream; re-tuning the bound to fit has been refused three times here.
2. **Take the SLA anchor as a citation question.** Finding 6 says the mechanism is real and
   the sign is a provenance question, and finding 4 says provenance is where the leverage
   actually is at 3.5× amplification. Cost: one targeted retrieval against [A] Penning de
   Vries (on the shelf) for a winter-wheat SLA-vs-development table, then an unfreeze that
   moves goldens if it lands. ⚠ It is **not** a free win — the late-anchored ramp overshoots
   the band's ceiling, so a sourced table has to land inside a window, and the anchor
   decides a 2.7× swing.
3. **Neither — leave the deviation documented.** It already is. The band still passes at
   the shipped step.

⚠ Options 1 and 2 are **independent**: the layered canopy costs ~6.5 % of peak LAI and the
SLA anchor can move it ±35–75 %, so building both is coherent — the honest physics plus a
sourced leaf-area constant — and is the only combination in which the layered canopy does
not eat the band's margin. It is also the largest.

## 8. What is NOT claimed

- **Not claimed: that the SLA route works.** Two uncited ramps bracket the sign. No table
  has been retrieved, no value proposed, no golden run against a sourced form.
- **Not claimed: that the layered canopy is wrong physics.** It is refused as a *fix*, on
  its sign. As physics it is better than what ships.
- **Not claimed: that 4.71 proves a missing mechanism.** Finding 4 says the observable
  cannot carry that weight at 3.5× amplification against three provisional literals.
- **Not measured: the partition table**, §6b's third candidate. It is a live suspect (it
  was fitted against the biased assimilation) and this pass did not touch it.
- **Not measured: `open_season`'s other gates under either candidate.** Only the two
  perennial liveness floors were run (finding 3); the CO₂ compensation-point bands and the
  Greenwood crossing were not, because nothing is proposed for shipping.

## 9. Probes

`M:/claud_projects/temp/canopy-layers/` — `probe1_sign.py` (open-loop sign/size),
`probe2_closed_loop.py` (closed loop, both steps, both schemes), `probe3_routes.py`
(elasticity + ratio-vs-LAI), `probe4_sla.py` (the two anchorings + DVS at peak),
`probe5_floors.py` (perennial liveness floors under the layered canopy). Every one carries
its frozen control in the same run.
