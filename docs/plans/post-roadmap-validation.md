# Post-roadmap: validation — the oracle match and the uncited params (bucket 3)

**Status: DIAGNOSIS COMPLETE, scope not yet chosen.** Third of the three sequenced
post-roadmap items (`post-roadmap-flow-registry-growth.md`, "The sequence"). Tier 1
(reuse) and Tier 2 (the grammar) are done; this is the one that establishes what is
**true**.

**No golden has moved and none may move before the scope decision below is taken.** The
diagnosis is measurement only — it reads the two committed PCSE-free fixtures and runs the
season. Nothing in `src/` has been touched.

## The charge

Two deferred debts, recorded in `CLAUDE.md`'s post-roadmap table:

1. **The Phase-1 quantitative oracle match**, deferred by user decision at Phase-1 Step 11
   and never run. Its tests are opt-in (`-m oracle`) and absent from the default gate.
2. **The 55 uncited params** — 55 of 57 carry
   `source: "TODO(cite) — provisional, literature-typical … pending validation gate"`.

## These are two mechanisms over disjoint param sets, not one job

The plan that sequenced this bucket framed it as "the deferred oracle match **and** the 55
uncited params", which reads as one homogeneous task. It is not:

| set | validated by | act |
|---|---|---|
| the potential-production crop params (`photosynthesis`, `canopy`, `phenology`, `allocation`, `respiration`, `senescence`, `transpiration`) | the WOFOST oracle trajectory | calibration — **citation and calibration are the same act** |
| the rest (`decomposition`, `mineralization`, `microbial_respiration`, `nitrogen`, `herbivory`; all 10 Power/Thermal/ECLSS params) | **no oracle exists** | literature citation against a primary source — a lighter, separate act |

Only the first set is driven by a trajectory match. Planning the 55 as homogeneous would
mis-size the work in both directions.

## The diagnosis (measured 2026-07-16)

*The measuring scripts were scratch (run outside the tree, PCSE-free, `src/` untouched);
their **numbers** are recorded below, which is the durable part. Promoting the diagnostic
into `tests/` — so the timing failure below cannot silently return — is part of scope (A).*

The pre-existing record said only: "the trajectory runs ~2 orders of magnitude below the
oracle", attributed to "uncalibrated placeholders + no vernalization" (`season.py` module
docstring; `tests/test_oracle_smoke.py`). That attribution is **incomplete**, and the
headline number (peak LAI ~0.09 vs ~6.3) understates a structural failure.

### Finding 1 — the canopy never bootstraps: a source-limited death spiral (DOMINANT)

Measured, per day:

| day | our leaf (mol C) | our LAI | our light interception | oracle LAI |
|---|---|---|---|---|
| 0 | 0.050 | 0.029 | **1.75 %** | 0.034 |
| 32 | 0.146 | 0.085 | **5.00 %** | 0.092 |
| 100 | 0.055 | 0.032 | 1.92 % | 0.239 |
| 212 | 0.022 | 0.013 | 0.79 % | **6.337** |

**Our canopy intercepts 1.75 % of incident light at sowing, peaks at 5.0 % on day 32, and
collapses. The oracle reaches 97.8 % interception at peak LAI.**

The mechanism, confirmed empirically rather than inferred: the seedling is source-limited
from day 1. At LAI ≈ 0.03 the Beer–Lambert interception is ~1.7 %, so gross assimilation
is tiny, so the daily structural increment to leaf (`fl · DMI`, with `fl = 0.55` — the
allocation table is *not* the problem here) is smaller than the 2 %/day leaf death rate
(`rdr_leaf`). Leaf shrinks → intercepts less light → fixes less carbon → shrinks faster.
The canopy can never get off the ground.

**The initial condition is not the problem** — our LAI₀ (0.029) and the oracle's (0.034)
agree. The difference is entirely in the growth dynamics.

The structural gap: a juvenile canopy-expansion phase that is temperature-driven rather
than assimilate-limited. This is standard crop physiology, available in the primary
sources already cited by `allocation.yaml` / `senescence.yaml` (Penning de Vries et al.
1989; van Keulen & Wolf 1986) — it must be sourced from those, **never** reverse-engineered
from PCSE (`docs/reuse-and-licenses.md`: PCSE is an offline oracle, cite the paper not the
tool).

### Finding 2 — phenology runs ~1.6x fast, and it is INDEPENDENT of finding 1

| stage | our day | oracle day | early by |
|---|---|---|---|
| DVS 0.5 | 47 | 193 | 146 d |
| DVS 1.0 (anthesis) | 138 | 217 | 79 d |
| DVS 2.0 (maturity) | 218 | 292 | 74 d |

Our winter wheat, sown 1 Oct, reaches anthesis in **mid-February**. Grounded in source, not
inferred: there is **no vernalization term anywhere in `src/`** — `phenology.py:48-54`
documents it as a deliberately deferred "second state accumulator" with a derived
`VERNFAC ∈ [0,1]`. DVS accumulates on thermal time alone, so it races through the winter
the oracle sits dormant through.

**DVS runs on thermal time, independent of biomass — so fixing phenology does not fix
finding 1, and vice versa.** The day-32 collapse happens deep in the vegetative phase; a
longer vegetative phase would only let it collapse more slowly. Two problems, changed one
at a time.

### Finding 3 — partitioning is roughly right; the earlier read was a confound

Sampled at **matched DVS** (dimensionless, conversion-free), not matched calendar day:

| DVS | organ | ours | oracle |
|---|---|---|---|
| 0.5 | leaf / stem / root / storage | 0.362 / 0.248 / **0.390** / 0.000 | 0.445 / 0.300 / **0.256** / 0.000 |
| 1.0 | leaf / stem / root / storage | 0.169 / 0.446 / **0.385** / 0.000 | 0.299 / 0.535 / **0.146** / 0.020 |
| 2.0 | leaf / stem / root / storage | 0.069 / 0.345 / 0.183 / **0.403** | 0.145 / 0.261 / 0.073 / **0.521** |

**A matched-*day* comparison is invalid here and produced a wrong answer.** At day ~305 our
plant has sat at DVS 2 for ~87 days (senescing at 2 %/day, filling grain) while the oracle
matured 13 days earlier; that comparison said "we over-allocate to storage, 0.69 vs 0.52".
At matched DVS the sign **reverses** — we *under*-allocate to storage (0.403 vs 0.521). The
first read was an artifact of finding 2. Recorded here because the confound is easy to
re-introduce and the corrected method (sample at matched phenological state) is the
reusable part.

What survives as a real signal: a **root-heavy bias** (0.390 vs 0.256 at DVS 0.5; 0.385 vs
0.146 at anthesis). Modest next to findings 1 and 2.

### The ranking

1. **The canopy deficit** — dominant, and it *worsens* at matched DVS (53x at DVS 0.5 →
   386x at anthesis → 436x at DVS 1.5), because the oracle's canopy expands while ours
   collapses. Structural.
2. **The phenology overrun** — independent; structural (vernalization).
3. **Param values** — real, but the *third* cause. No amount of tuning within literature
   ranges fixes a canopy that intercepts 1.75 % of light.

**The load-bearing consequence: the deferred "quantitative oracle match" is not a
calibration task.** It was sequenced as one. Two of its three causes are missing science,
not wrong numbers.

## What the existing tests do not catch

`tests/test_oracle_smoke.py::test_lai_is_unimodal_for_both` passes on a canopy that peaks
on **day 32 of ~305** and collapses before anthesis. `_is_unimodal` asks only for an
interior peak with ends below half-peak — it has **no timing teeth**. The suite therefore
records the magnitude gap but is blind to the timing failure underneath it, which is the
more diagnostic signal. Any increment should close this.

## Scope constraints (verified, not assumed)

- **The Rust port hardcodes biosphere param values** (`crates/domains/src/biosphere/biosphere_params.txt`,
  read via `include_str!`) — **but the file is generated** from the Python loaders
  (`uv run python tests/crossport/gen_biosphere_params.py`). A param recalibration is
  therefore a mechanical regeneration on the port side, not a hand-edit per number. **New
  science (vernalization, juvenile canopy expansion) is a code change and must be mirrored
  in `rust/crates/domains` by hand** to hold cross-port parity.
- **The station goldens cascade.** `station/{greenhouse,lighting,sealed,harvest}.py` import
  `build_season` from `domains.biosphere.season` — they *run* biosphere science rather than
  merely referencing its goldens. A biosphere recalibration moves station goldens too.
  ("Delegated" in the station manifest means delegated *science*, not insulation.)
- **Both halves of the work are a biosphere unfreeze** (`docs/biosphere-reference.md`, "The
  unfreeze discipline", steps 1–6): advisor review *before* regenerating anything, then
  goldens, then manifest, then provenance, then gates.
- **`git diff src/simcore/` must stay empty** — unconditionally, on every path below.

## The scope decision (OPEN — the user's call)

The diagnosis makes the options concrete. Recorded here so the choice is deliberate:

- **(A) Honest first increment.** Pin the gap as a *number with its cause* — the
  `test_bvad_validation.py` three-column precedent and the `lab.fit_order` "measure the
  known structural error" discipline the repo already uses. Give the timing failure teeth
  (finding 1 is currently invisible to the suite). Cite the highest-leverage params.
  Document vernalization + juvenile canopy expansion as known structural gaps, deferred
  with the diagnosis behind them. **Moves no golden**; no unfreeze.
- **(B) Full oracle match.** Implement vernalization + a juvenile canopy-expansion phase
  (clean-room from Penning de Vries / van Keulen), recalibrate the PP param set against the
  trajectory, mirror both into the Rust port by hand, regenerate the 7 frozen biosphere
  goldens + the cascaded station goldens under the unfreeze discipline. Large, multi-part,
  and the first post-roadmap work to add **science** rather than reach or expressiveness.
- **(C) The no-oracle citation half only.** Cite the 10 sibling params + the 5 non-PP
  biosphere files against primary sources. Independent of the oracle work, and the only
  part with no structural blocker.

The sequencing rationale recorded in the Tier-1 plan — deviation is legitimate "as long as
it is deliberate and documented" — makes (A) coherent: it converts an *undocumented* gap
into a documented one, which is precisely the standard the project already holds itself to.
