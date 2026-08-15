# The canopy's two uncited literals — the provenance work the layered canopy named

**Taken 2026-08-15**, the day after the layered-canopy unfreeze, on that build's own
closing line: *"NOT DONE: the two surviving `TODO(cite)` literals under the 3.5× amplifier
(`extinction_coef`, `carbon_fraction`) — now flagged in `canopy.yaml`'s header as the
highest-leverage provenance work left on this observable."*

The charge was one item. **It split into three, and only one of the three was a retrieval
problem.**

| | Outcome |
|---|---|
| `carbon_fraction` | **BUILT** — and it was never uncited; the citation was one file away |
| `extinction_coef` | **MEASURED, NOTHING BUILT** — three disagreeing readings on the shelf; the decision is the user's |
| The band prose | **CORRECTED** — a spin-off: five frozen values in prose had been superseded six commits after they were written |

---

## 1. Why this was the pick

The predecessor measured the peak-LAI band's subject as a **~3.5× amplifier**: a 5 %
error in leaf-area-per-carbon or in assimilation buys 18 % of peak LAI. Three of the
inputs that set it were provisional literals, which made *a ±2 % provenance disagreement
the whole distance to the bound*. That finding's own conclusion was that **a band which
amplifies every parameter error 3.5× cannot arbitrate between mechanisms** — so the
provenance is upstream of every remaining mechanism question on this observable.

⚠ **The 3.5× figure does not transfer to `k`, and this work is where that was established.**
It is the elasticity to a *uniform* perturbation of leaf area or assimilation. `k` enters
`absorbed_par = k·I₀·exp(−k·L·LAI)` — it raises absorption at the top of the canopy and
lowers penetration to the bottom, so its effect is **self-cancelling in the closed canopy**
and only linear in the open one. Measured below: +8.3 % on `k` buys **+0.8 %** of peak LAI
at the shipped step, not +30 %. A pre-registered elasticity is a claim about a knob, not
about a file.

---

## 2. `carbon_fraction` — BUILT, and the finding is where it was found

Full detail in the freeze doc's unfreeze log (2026-08-15). In brief:

- `nitrogen.yaml` has cited the identical 0.45 to **Raimanova et al. (2024)** since the
  2026-07-16 citation round, under a **MUST-EQUAL** constraint with `canopy.yaml` that both
  files document.
- `crops/potato/canopy.yaml` has said since 2026-08-11 that *"the reference value IS cited,
  but to a measurement of wheat grain/straw"* — describing `canopy.yaml`, which still read
  `TODO(cite)`. **A crop override was the record that the reference was sourced.**
- Fourth instance of *check your own shelf*, and the fourth where the stale artifact was
  **our own record** rather than the literature.
- What was genuinely new: the **basis**. `nitrogen.yaml` uses the fraction whole-plant and
  carries a root delta (roots 34.9 %, ~10 points below shoots). `canopy.yaml` uses it on
  **leaf blade**, so the straw figure (45.66 %) is the nearer measurement and that delta
  does not transfer. Copying the source string verbatim would have imported a caveat that
  does not apply.

Cost: one sha-256 in the manifest. No value, no golden, no `src/` logic.

---

## 3. `extinction_coef` — the shelf answered, three times, differently

All three verified **off page renders**, not the text layer (`docs/log/canopy-magnitude.md`
records why that matters for the Penning de Vries scan).

| Reading | Source | What it is |
|---|---|---|
| **0.60** | Penning de Vries et al. (1989) p. 36 — *"Its average extinction coefficient is about 0.6 for a canopy with erect leaves and 0.8 for one with horizontal leaves (Goudriaan, 1977)"* | An **architecture-class** value for PAR. Wheat is erectophile. Already source [B] of this file, and **Goudriaan is the author of the depth scheme shipped yesterday** |
| **0.65** | Soltani & Sinclair (2012) Table 10.1 p. 125, the "Wheat" row | A **crop-specific** value, for the *identical equation* (their Eqn 10.2, `FINT = 1 − exp(−KPAR·LAI)`) |
| **0.68** | ibid. Fig. 10.8(a), fitted to measured PAR interception in wheat cv. Zagros | A **measurement**, but explicitly *unpublished data* — corroboration, not a citation |

⚠ **The "wheat-specific" framing does not settle it by itself.** Table 10.1's own caption
lists *Penning de Vries et al., 1989* among its sources, so 0.65 is partly a **derived**
row rather than an independent wheat measurement; and 0.68 is unpublished. Meanwhile 0.60
is a first-hand statement about erectophile canopies attributed to Goudriaan (1977), whose
quadrature this tree now runs. **There is a coherence argument for 0.60 that has nothing to
do with which number is free**, and it must be stated as such — picking 0.60 *because* it
moves no golden is the co-adaptation this project has refused four times.

Teh (2013) was read too and gives no wheat value: it derives `k` from leaf angle and solar
elevation (`k = 0.5/sin β` for a spherical distribution), i.e. an *instantaneous* quantity,
where ours is a daily constant. Not a fourth reading.

### 3a. What each value does — measured on `cc44b41`, `src/` untouched

`extinction_coef` has **exactly one production consumer** (`photosynthesis.py:272`, via
`CanopyParams`), so a loader patch reaches everything. ⚠ That audit found a second thing:
**`intercepted_fraction` now has no caller in `src/` at all** — the layered canopy replaced
it, and it survives only in tests and the Rust mirror. It is the dead `exp` behind
`cc44b41`'s vacuous cross-port gate, still exported from the frozen surface.

⚠⚠ **That dead function is now load-bearing as a tripwire, which makes deleting it a
regression waiting to happen.** `cc44b41`'s repair has the ULP probe shim **both** modules
holding a Beer–Lambert `exp` — and one of the two is this function, which the carbon path no
longer calls. Deleting it as dead code (an entirely reasonable cleanup) would take the probe
back toward the vacuous state that fix was written for; the zero-sensitivity assertion added
in the same commit is what would catch it, and only if the *other* shim still perturbs.
**Any removal of `intercepted_fraction` must re-check `tests/crossport/measure_tier2_bands.py`
in the same change.**

**`open_season`** (the band's subject; band is `5.0 < peak LAI < 8.0`):

| `k` | peak LAI, `dt = ¼` shipped | peak LAI, `dt = 1/32` converged | LAI peak at DVS | harvest | rationed |
|---|---|---|---|---|---|
| 0.60 | 6.0228 | 5.4273 | 1.306 | 32.6995 | 0 |
| 0.65 | 6.0700 (+0.8 %) | 5.8233 (+7.3 %) | **0.909** | 33.0269 | 0 |
| 0.68 | 6.0586 (+0.6 %) | 6.0040 (+10.6 %) | **0.957** | 33.1159 | 0 |

⚠ **The shipped column barely moves, and the reason is a mechanism doing its job**: the
Van Keulen & Seligman 5 %/day mutual-shading loss above LAI 6 shipped yesterday **inert**
(0.2 days above the threshold). At `k = 0.65` it is above 6 for 1.8 days and is **clipping
the peak**. The mechanism that was forced into the tree by a threshold, and that the record
flagged as currently doing nothing, is the reason a bigger `k` cannot run away.

⚠ **The LAI peak moves 0.4 DVS earlier**, from *after* anthesis (1.306) to *before* it
(0.909) — a **second, independently pinned defect** moving the right way, corroborated by
two oracles of different model families that both peak before anthesis. The SLA
investigation predicted this direction from an unrelated knob.

**The five frozen chamber CO₂ bands** (floor 61.0714 ppm), measured with the band's own
helper, each scenario driven the way its own golden drives it:

| scenario | `k = 0.60` | `k = 0.65` | `k = 0.68` |
|---|---|---|---|
| `sealed_chamber` | 71.4358 (1.1697×) | 72.7554 (1.1913×) | 73.5044 (1.2036×) |
| `perennial_chamber` | 70.2526 (1.1503×) | 71.5936 (1.1723×) | 72.4023 (1.1855×) |
| `consumer_chamber` | 73.3386 (1.2009×) | 73.9088 (1.2102×) | 74.6714 (1.2227×) |

**Every band gets looser.** ⚠ **Read that with the next table, not on its own** — it is not
a safety margin improving, it is the chamber crop doing **less**.

**The liveness floors**, 15-year perennial chamber:

| | `k = 0.60` | `k = 0.65` |
|---|---|---|
| `max(tail)` peak leaf (floor **0.55**) | 0.578137 (**+5.12 %**) | 0.552202 (**+0.40 %**) |
| 15-yr final-year peak leaf | 0.550405 | **0.525448 — below the floor** |
| 50-yr attractor | 0.543748 | 0.519219 |

⚠⚠ **This is the finding that decides the item.** At `k = 0.65` the perennial liveness floor
clears by **0.40 %**, down from 5.12 %, and it clears *only because* `max(tail)` takes the
maximum over the tail window — the trajectory is monotonically declining and its final year
is already **below** the bound. It is, on this evidence, one small mechanism away from red.

⚠ **And "green" here means green *locally*.** This was measured on Windows in a worktree; CI
is Linux, and `ci-python-job-red-on-linux` records goldens minted on this box going red there
on libm ULP differences. **0.40 % is inside the band where that has bitten before** — which
is an argument against 0.65 that none of the other options carries, and it is a *risk*, not a
measurement: nothing has been run on CI at `k = 0.65`.

**And the two tables have one cause.** A larger `k` intercepts more light per unit leaf; in
the open field that is more carbon and a bigger canopy, and in a **sealed** chamber it is a
faster draw on a fixed carbon inventory, so the crop limits earlier and ends **smaller** —
which is exactly why the chamber's CO₂ minimum *rises*. **Reading the CO₂ bands alone would
have reported `k = 0.65` as uniformly safe.** The band improves because the plant shrinks.

`pytest tests/test_decade_stability.py` at `k = 0.65` in a worktree: **31 passed, 3 failed**
— all three failures exact-value characterization pins (the 50-yr attractor, the CO₂ trough
attractor, the CO₂-floor mutation witness), which any value move breaks. **No floor and no
band goes red.**

### 3b. The three options, priced

1. **Bind to 0.60** (Penning de Vries, erect-leaf canopy, Goudriaan 1977). Retires the last
   `TODO(cite)` in the file at the **provenance-only** price — one manifest hash, no golden.
   Coherent with the shipped depth quadrature. ⚠ Must be argued on the merits and recorded
   *with* the disagreement, never as "the free one".
2. **Bind to 0.65** (Soltani & Sinclair, wheat row, our exact equation). The full ceremony —
   13 goldens, both manifests, the Rust mirror, three characterization re-pins. Buys a
   converged canopy 7.3 % nearer the middle of its band and a **development-stage peak
   moving from after anthesis to before it**, which is a second pinned defect improving. ⚠
   Costs the perennial liveness floor almost all its clearance (5.12 % → 0.40 %).
3. **Bind to 0.60 and record the disagreement as a measured risk** — the shape used for `Γ*`.
   ⚠ **But not the same asymmetry, and the difference matters.** For `Γ*` the cited
   alternative sits *below* the shipped value, so one direction is conservative on every
   count. Here the two risks are **opposed**: on *provenance*, 0.60 may be ~8 % low against
   the crop-specific literature (our canopy intercepting less light than published wheat
   does); on *the gates*, 0.60 is the **conservative** side, because the alternatives are the
   ones that spend the perennial liveness floor's clearance down to 0.40 %. **There is no
   direction here that is safe on both axes**, which is precisely why this is a decision and
   not a lookup.

**Not recommended: 0.68.** Unpublished data cannot retire a `TODO(cite)` in a tree whose
rule is *cite the primary literature*; it belongs in the record as corroboration that the
published wheat row is not an outlier.

---

## 4. The spin-off — five frozen values in prose, superseded six commits after they were written

Shipped separately (`d8f5583`) because it is a correction to the previous two builds, not to
this one. The band table in the freeze doc and the docstrings in
`tests/test_co2_compensation_band.py` quoted 76.82 / 75.48 / 74.42 ppm; the tree reads
71.44 / 70.25 / 73.34. The **light path** moved them 4–7 % on the same day the band landed.

⚠ **The pin written for exactly this event fired, and the prose beside it was left anyway.**
`test_the_five_margins_are_pinned_not_merely_positive` went red at the light path and was
re-pinned in `a0ef98b`, *with its own comment updated to say so* — four lines below
docstrings that kept the superseded numbers. So the gap is not a missing guard: **a value in
an assertion acquires an owner the moment it goes red, and a value in prose acquires none.**

⚠ **The ranking inverted too**: the consumer chamber's docstring argues *"THE TIGHTEST OF THE
FIVE … enumerate the roster, not the discussion"* and it is now the **loosest**. Both builds
that moved these act through canopy **closure**, and the consumer chamber's crop is the one
the crew's CO₂ keeps furthest from closing, so it lost the least. The lesson survives; its
subject moved. **A ranking is a claim about a moment — re-derive it, never quote it.**

---

## 5. Status

- `carbon_fraction` — **BUILT** (`c8f6df5`).
- The band prose — **CORRECTED** (`d8f5583`).
- `extinction_coef` — **MEASURED; the decision is the user's and is untaken.**
- Probes: `M:/claud_projects/temp/canopy-k/` (`probe1_k_sweep.py`, `probe2_chamber_bands.py`,
  `probe3_floor_margins.py`, worktree `wt-k65`).
