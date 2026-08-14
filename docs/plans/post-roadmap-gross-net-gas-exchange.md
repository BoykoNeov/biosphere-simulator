# Gross/net gas exchange — the plant breathes continuously

**Status: stages 3 + 1 BUILT AND SHIPPED 2026-08-14 — the ceremony is COMPLETE.** 13
goldens, both manifests, both reference docs, the Rust mirror and 99 re-pinned tests; full
suite green (2404 passed). Stages 0 + 2 are **not built** (finding 11), so "minute by
minute" is not delivered — a real day/night cycle at quarter-day granularity is. The build
section at the foot of this file is the measurement record, taken **before** any golden
moved; ⚠ its finding 14 headline was corrected once already (the band passes at the
shipped step and fails converged), and the gate report was corrected again after shipping
— *the liveness floors were omitted from it, and two of them narrowed*. See
`docs/biosphere-reference.md`.

## Charge

The user, after being shown that the two-rate driver delivers the plant's whole day of gas
exchange in lumps *before* the crew breathes for 1440 minutes:

> "the plants MUST emit oxygen at least minute by minute and consume co2, it would be great
> if there is a mechanism to simulate their growth on a larger scale, but it is imperative,
> it is what reality is."

and, on being told the within-day light curve was a separable item:

> "it should be possible for the light Input to vary within the day"

Scope authorized: **stage 1 + stage 0 + stage 2** (mechanism, time-unit refactor, fast gas
exchange). Stage 3 (the light curve) was *promoted into* that scope by finding 2 below — it
is a precondition of stage 2, not an enhancement.

Standing context: the user's prior instruction that the project should build the mechanisms
reality has, rather than refuse cited science for want of them
(`post-roadmap-wheat-partition-backfill.md`, `post-roadmap-root-functional-coupling.md`).

---

## FINDING 1 — the charge is a FORM change, not a cadence change

⚠ **The granularity complaint is downstream of a gross/net collapse, and sub-stepping alone
would not discharge it.**

`carbon_budget.CarbonContext.budget()` computes `(GASS, MRES, available)` — gross canopy
assimilation and maintenance respiration, separately and correctly. `Allocation.evaluate`
then collapses them: it takes `available`, applies growth efficiency to get the structural
increment `DMI`, and sources **both gas legs from `organ_total`** — the summed *organ* legs,
i.e. the net structural increment:

```python
organ_total = leaf_leg + stem_leg + root_leg + storage_leg
legs = [Leg(self.co2_atmos, -organ_total), Leg(self.ctx.leaf_c, leaf_leg), ...]
...
legs.append(Leg(self.o2_pool, organ_total))   # PQ = 1
```

So the only carbon that ever crosses a gas boundary from the plant is carbon that ended up
in a **structural organ**. Consequences, in ascending order of seriousness:

1. **When `DMI` is zero, no gas moves at all.** Not "a little"; none.
2. **The plant never respires into the air.** A real plant at night consumes O₂ and emits
   CO₂. Ours emits and consumes strictly in proportion to growth, so its night-time gas
   exchange is whatever the day's net says, smeared across the step — and if growth has
   stopped (maturity, stress, darkness within a coarse step), it is nothing.
3. **Sub-stepping this flow 1440× yields 1440 small net-growth events.** Smoother, still not
   breathing. **The cadence fix does not reach the defect.**

⇒ The charge lands in *mechanism* work, not in the step-unification work. This is the
finding to lead with, and it means the user's two asks (more mechanisms; finer gas exchange)
are one ask.

> **⚠ AMENDED by finding 5 (same day, after advisor review + probe).** The three consequences
> above are all correct and all still hold. The *cause* attributed to them here is not: the
> netting of `GASS` against `MRES` on one well-mixed pool is physically sound, and the real
> cause is that a day-averaged PAR makes `GASS > MRES` at **every** step, so the night branch
> of `MaintenanceRespiration` — which already exists, already returns CO₂ to the shared pool
> and already consumes O₂ — is unreachable. **Read finding 5 before acting on this one.**
> Kept unedited as the record of a diagnosis that was right about the symptom and wrong about
> the mechanism, which is the shape `asserted-attributions-rot` warns about.

## FINDING 2 — ⚠⚠ THE WITHIN-DAY LIGHT CURVE IS A PRECONDITION OF STAGE 2, NOT AN OPTION

`photosynthesis.daily_canopy_assimilation` is instantaneous FvCB times **one** daily factor:

```python
return canopy_rate * daylength_s * ground_area * MICROMOL_TO_MOL * f_temp * limitation
```

`× daylength_s` is the **only** thing in the model that knows the sun sets. `weather.incident_par`
supplies a **daytime-mean** PAR (`0.5 · IRRAD / daylength_s · 4.57`), one value per day from
a `_table` schedule.

Sub-step that flow per minute against the same constant PAR and the day's assimilation
becomes `canopy_rate × 86400 s` instead of `canopy_rate × daylength_s`:

> **at a 12 h day, exactly 2× the carbon, fixed at full rate through the night.**

Stage 2 without a light curve is not incomplete — it is **physically wrong in a new way that
the current coarse model is not**. The user's instinct arrived at this independently.

## FINDING 3 — the form is CITED, on a shelf already first-hand for six rows

`sources/Simulation Of Ecophysiological Processes Of Growth In -- Penning De Vries et al.`
([E]), §on canopy photosynthesis:

> "A pioneer model by de Wit (1965) has been improved and expanded (Goudriaan, 1977, 1986;
> de Wit et al., 1978; Spitters, 1986; Spitters et al., 1986) and because of its versatility
> and documentation its approach is followed in this book. … It uses a specific way of
> integrating the instantaneous rate of leaf photosynthesis in time (three points between
> noon and sunset, times two, assuming the morning to be equal to the afternoon) and in space
> (three depths in the canopy). … **The path of radiation intensity during the day is assumed
> to be sinusoidal.**"

And the *direction* of the resulting change, stated by the source as an exercise answer (T6):

> "The increase in canopy photosynthesis per unit increase of radiation decreases
> continuously (Figure 17). **Splitting an amount of radiation in unequal portions over a day
> leads to a lower daily total.**"

⇒ **The predicted golden movement is cited, not inferred: assimilation goes DOWN.** Our
current constant-mean-PAR evaluation is the "equal portions" case; a sinusoidal path is the
"unequal portions" case; the source says the latter totals less. This is the same concavity
the code's own docstring already confesses:

> "**Provisional high-bias (Jensen).** `Ag` is concave in PAR, so this mean-PAR big-leaf
> overestimates the true intra-canopy/diurnal integral. Closing that gap is the Step-11
> Gaussian, which extends *this* function additively."

**⚠ And sub-stepping makes the deferred Gaussian UNNECESSARY.** Goudriaan's three-point
scheme is a *fast approximation to an integral* for models that take one step per day. A
model that steps the light curve per minute performs that integral **directly and more
accurately**, with no quadrature scheme to add. The Step-11 deferral is discharged by a
different route than the one it named — record it as such rather than leaving it open.

## FINDING 5 — ⚠⚠ THE NIGHT-RESPIRATION MECHANISM IS ALREADY BUILT AND CITED. IT CANNOT FIRE.

**Advisor review raised a blocker — "respired CO₂ goes to a sink, not to the air" — and it is
FALSE for the sealed chamber**, which is the case the charge is about. `stocks.py:260`:

```python
resp_sink=CARBON_POOL if sealed else CO2_RESP
```

In a sealed chamber the respiration "sink" **is** the shared carbon pool. And
`MaintenanceRespiration.evaluate` already carries a fully-developed sealed branch: it drops
the `covered` maintenance as a net-zero CO₂→CO₂ round trip on the single pool, burns the
`shortfall` out of leaf/stem/root in proportion to biomass, returns that carbon **to the
pool**, consumes O₂ at PQ=1, and throttles the whole thing by an O₂ half-saturation factor so
it shuts off smoothly on a depleting pool.

**That is exactly the mechanism the charge asks for. It is already written, already cited,
and it never runs.** It is gated on `shortfall = MRES − min(GASS, MRES) > 0`, and with a
day-averaged PAR the daily `GASS` exceeds `MRES` by 20–200×, so `shortfall` is **identically
zero at every step of every frozen scenario** (measured, probe Q2 below).

⇒ **The defect is not a missing mechanism. It is a forcing that makes an existing mechanism
unreachable.** Same shape as `canopy-regulator-diagnosed` ("blocked on a MISSING science" was
false the day it was written) and `stem-reserve-form-is-on-the-shelf`. **Check whether the
mechanism exists and is merely unreachable before designing a replacement.**

### ⇒ The design collapses. The labile pool is NOT required.

With an intra-day light path and the carbon flows rate-classed fast:

| | `GASS` | `covered` / `shortfall` | net pool effect | net O₂ |
|---|---|---|---|---|
| **day sub-step** | > 0, > `MRES` | `MRES` / 0 | `Allocation` draws `DMI` | `+DMI` |
| **night sub-step** | **0** | 0 / **`MRES`** | organs burn → **CO₂ returns to pool** | **−`MRES`** |

The chamber then *rises* in CO₂ and *falls* in O₂ overnight and reverses by day — the diurnal
signature the charge demands — **with no new stock and no new flow.**

⚠ **And the netting that finding 1 called a "collapse" is physically CORRECT for a well-mixed
pool.** Simultaneous uptake of `GASS` and release of `MRES` into one pool *is* a net transfer
of `GASS − MRES`; dropping the round trip loses nothing a well-mixed pool could observe.
Finding 1's three consequences survive intact, but their cause is re-attributed: they follow
from **`GASS` never being zero**, not from the netting. Revised accordingly — see finding 1's
amendment note.

The labile pool remains **optional**, needed only if partitioning must stay strictly daily
while gas runs fast. Decide it on measurement, not in advance; it is no longer on the
critical path, and dropping it removes a new stock, a new flow, and a new free parameter from
the blast radius.

## FINDING 6 — the science-gate risk is ~1 %, not ~3.8 % (MEASURED)

Probe: `M:/claud_projects/temp/gasexchange/probe_lightcurve.py` (read-only, frozen params,
`git diff src/` empty). Half-sine at `peak = (π/2)·mean`, integrated at 60 s against today's
constant-mean evaluation.

| LAI | 0.5 | 1.0 | 2.0 | 3.0 | **5.191** | 8.0 |
|---|---|---|---|---|---|---|
| sine/flat | 0.964 | 0.970 | 0.977 | 0.983 | **0.989** | 0.993 |

**The loss shrinks as the canopy closes** — at high LAI the mean absorbed PAR *per leaf* is
lower, so FvCB is operating on a more nearly linear stretch and there is less concavity to
lose. At `open_season`'s pinned peak LAI 5.191 the multiplier is **0.989 — a 1.1 % loss
against 3.68 % of band headroom.** Direction confirms [E] T6.

⚠ **This is an INSTANTANEOUS multiplier, not the season outcome, and it must not be quoted as
one.** The worst loss (3.6 %) lands at low LAI — i.e. during the early **compounding** phase,
which `wheat-partition-backfill-refused` measured as the phase that *sets* peak canopy
("diversions after DVS ≈ 0.6 are nearly free"). A 3.6 % early loss can compound to more than
1.1 % at peak. **The band is likely to hold and is NOT yet proven to.** The compounded figure
comes from running the season, which is a build, not a probe.

Probe Q2, same run — daily `shortfall` is **0.00000 in every case** (daily `GASS` 0.45–1.01
vs `MRES` 0.005–0.05), while night-only `MRES` is **0.0016–0.0156 mol C**, strictly positive.
⇒ finding 5's gate is confirmed shut today and confirmed opened by the light path.

## FINDING 4 — every piece needed is already in the tree

| Need | Already present | Cite |
|---|---|---|
| Instantaneous canopy rate | `daily_canopy_assimilation` minus its `× daylength_s` | [A] Farquhar 1980 |
| Solar geometry | `weather.daylength_seconds` — declination, sunset hour angle | FAO-56 eq. 24/25/34 |
| A half-sine daylight schedule | `power/system.py:156` ships one | in-tree precedent |
| Gross vs. maintenance split | `CarbonContext.budget()` returns both, then discards the split | — |
| Fast/slow rate classing | `simcore.multirate`, `station.driver.run_master_day` | — |

**No new photosynthesis science is written by this plan.** The kernel is instantaneous and
cited; the daily wrapper is one multiplication; the geometry that says when it is dark is
already computed for daylength.

---

## Design

> ⚠ **REVISED after findings 5 + 6.** The original stage 1 (a new `labile_c` stock and a new
> `GrossAssimilation` flow) is **struck**. It is not required: the mechanism exists, and the
> light path plus rate-classing reaches it. The struck design is preserved at the foot of
> this document under "Struck: the labile-pool design", with why it was dropped — per
> `docs/context-budget.md` rule on refusals, a dropped design is recorded, not deleted.

### Stage 1 (revised) — no new stock, no new flow

Nothing to build. Findings 5 + 6 establish that `MaintenanceRespiration`'s sealed branch
already does what the charge asks, and that stages 3 + 2 are what make it reachable. The work
that remains is **verifying** it does so:

- pin that `shortfall > 0` for every night sub-step and `== 0` for every day sub-step;
- pin the chamber's diurnal CO₂ and O₂ swing (sign and magnitude), which is the observable
  the charge is actually about and which **no current test asserts**;
- pin that the `f_O2` throttle stays ≈ 1 at the PP fill (its deferral, `scenario.py` Step 3,
  is inherited and must be re-measured now that the branch actually runs).

### Stage 3 (folded in, per finding 2) — the sinusoidal light path

Replace the daily-constant PAR forcing with a within-day schedule:

```
PAR(t) = PAR_peak · sin(π · (t − sunrise) / daylength)   for sunrise ≤ t < sunset,  else 0
```

**The daily total is conserved by construction.** A half-sine integrates to `2/π · peak ·
daylength`, so setting `PAR_peak = (π/2) · PAR_daytime_mean` reproduces the same day's PAR
energy that `weather.incident_par` already computes. ⇒ **No new parameter, no new energy, no
recalibration of IRRAD.** The *only* thing that changes is the distribution — which is
exactly the concavity the source and our own docstring both name.

This is the same shape `power/system.py` already ships for solar, and its sunrise/sunset come
from the declination geometry `weather.py` already computes for daylength.

### Stage 0 — the time-unit refactor (do before stage 2)

Convert the remaining step-indexed quantities (the perturbation window bounds in
`domains/biosphere/perturbations.py` and `station/perturbations.py`) to physical days.
**Moves no golden** — perturbation runs carry no golden by standing decision. Rationale and
the four-bug precedent: `docs/log/step-unfreeze.md` ("three passes, each failing
differently — and a fourth found later"; the Rust port caught the one the audit missed).

⚠ Per that same record: **provably inert ≠ provably correct.** Each conversion needs its own
discriminating probe (flip the constant, run, read the failure *kind*, revert). A green suite
is not evidence here.

### Stage 2 — rate-class the gas exchange

Gas flows fast (per-minute, or whatever the cabin runs at); partitioning and phenology slow.
Requires teaching the biosphere side of `run_master_day` a two-registry split of its own,
which is `src/station/` and `src/domains/biosphere/` — **boundary code, a legitimate unfreeze
target.** ⚠ `git diff src/simcore/` still must come back empty.

---

## Hazards to settle DURING design, not after

1. **⚠ Same-step inflow vs. the arbitration backstop.** The recorded rule
   (`arbitration-no-same-step-inflows`) is that the backstop scales withdrawals against the
   **start-of-step** amount. A labile pool filled by assimilation and drained by respiration
   *in the same sub-step* is exactly that shape. **Mitigation to verify, not assume:** keep
   maintenance respiration drawing on **biomass** (as it does today) rather than on the
   labile pool, which removes the same-step dependency entirely.
2. **Rationing must stay at zero.** The frozen goldens assert `rationed == 0`. A labile pool
   that empties mid-day would ration. Sizing/ordering must be measured, not argued.
3. **The `f_O2` deferral.** Plant respiration currently needs no O₂ self-limitation because
   the O₂ pool is vastly larger than the fluxes (`scenario.py` Step-3 note). Making
   respiration continuous does not change the totals, so the deferral should hold — **verify
   the O₂ minimum across a season rather than inheriting the claim.**
4. **Extinction / positivity under a finer step.** The step-unfreeze precedent: quartering
   the step changed which guards were live. Re-read the liveness floors.
5. **Cross-port.** Every changed flow is hand-mirrored into Rust and the tier bands re-measured.
   The biosphere is already Tier 2 (transcendental); adding `sin` to the PAR path keeps it there.
6. **⚠ SCOPE, must be decided in this doc, not discovered: the 7 standalone biosphere goldens
   have no fast path.** `run_master_day` is the *station* driver; standalone runs go through
   `season.run_season`, single-rate, with no fast registry. Left as-is, stage 2 gives the same
   domain continuous gas exchange inside a station and lumped exchange standalone — the exact
   split the step sweep called *"no longer honest"* (`post-roadmap-step-sweep.md`, scope
   finding 2). **Decision required:** either `run_season` gains a two-rate path, or stage 2 is
   station-only and `docs/biosphere-reference.md` says so in the freeze text. Recommendation:
   `run_season` gains the path — the light curve (stage 3) moves the standalone goldens
   regardless, so they are already being regenerated.
7. **⚠ Days-vs-seconds at the registry boundary.** The driver requires `fast_dt ·
   steps_per_day == 86400` **seconds** while biosphere rate laws are **per-day**. Moving a
   biosphere flow into the fast registry hands it a `dt` in the wrong unit — precisely the
   hazard that bit the step unfreeze four times, where three successive audit passes each
   missed some and the Rust port caught the last (`docs/log/step-unfreeze.md`). **Plan a
   discriminating probe** (flip the unit, run, read the failure *kind*, revert); a green suite
   is not evidence.
8. **Labile-pool drain cadence — only if the pool is ever reinstated.** With a sinusoidal
   path, assimilation is zero all night; a pool drained per slow step (¼ day) is empty
   overnight and anything drawing on it rations. Moot under the revised design (no pool);
   recorded so a future reinstatement does not rediscover it.
9. **Manifest completeness for the forcing.** The biosphere manifest names a `forcing` set.
   An intra-day PAR schedule changes forcing *wiring* while `weather_sha256` is unchanged —
   check the manifest gains an entry naming the schedule shape, or this reproduces the
   "field absent from both sides ⇒ gate green" blindness recorded in
   `multirate-effective-step-is-per-rate-class`.

## Blast radius (the unfreeze this is)

| What moves | Detail |
|---|---|
| ~~`flow_set`~~ | **unchanged** — findings 5+6 struck the new flow |
| ~~Stock set~~ | **unchanged** — no `labile_c` |
| Forcing | PAR schedule becomes intra-day; `forcing.weather_sha256` unchanged (same IRRAD) |
| Biosphere goldens | **all 7** |
| Station goldens | the greenhouse-bearing ones: `greenhouse`, `harvest`, `lighting`, `sealed_station`, `sealed_energy_drift` |
| Manifests | biosphere + station |
| Ports | Rust mirror of every changed flow; tier bands re-measured |

**Predicted direction and magnitude, MEASURED before the build
(`soil-layers-built`: predict the golden diff before regenerating).** Assimilation falls, per
[E] T6 and probe finding 6: the instantaneous multiplier is **0.989 at peak LAI** and **0.964
at LAI 0.5**. `open_season` peak LAI sits at 5.191 against a contract band of 5.0–8.0 —
**3.68 % of headroom.** The pointwise loss at peak (1.1 %) fits; the early-season loss
(3.6 %) lands on the compounding phase that *sets* peak canopy, so **the compounded outcome
is not bounded by either number and is the thing to measure first on a real season run.**

A **second, offsetting** movement, predicted with its sign: night respiration returning
`MRES` to the pool (finding 5) should **raise** season-low chamber CO₂ — the quantity the
step unfreeze cared about (`sealed` 75.06 ppm against the 61.07 ppm compensation floor). Both
signs to be read off the regenerated goldens; **neither may be used to excuse the other**
without being measured separately.

⚠ If the LAI band breaks it is a science-gate failure to be argued in writing, **not** tuned
past: retuning a bound so a change fits is the co-adaptation shape this project has refused
three times (`science-gates-contract-standing`, `soil-fractionation-refused`,
`bucket3-scope-b-decomposer-calibration` ruling B). The two honest readings if it does: the
band was fitted against a biased assimilation and should be re-derived from its source, or
the missing canopy regulator (`canopy-regulator-diagnosed`) has become load-bearing.

⇒ **This is the plan's biggest live risk, and it is known before a line is written.** If the
band breaks, the honest readings are (a) the band was fitted to a biased assimilation and
should be re-derived from its source, or (b) the missing canopy regulator
(`canopy-regulator-diagnosed`) is now load-bearing. Both are findings; neither is a knob.

## Ceremony (docs/biosphere-reference.md, "The unfreeze discipline")

1. Justify + **advisor review before regenerating anything** — this document.
2. Change boundary-side; `git diff src/simcore/` empty.
3. Regenerate affected goldens via each `__main__`; review the byte diff.
4. Regenerate both manifests; review.
5. **Report the science gates** — every band and liveness floor, pass or fail, in writing.
6. Record provenance ([E] for the sinusoidal path and the direction; FAO-56 for geometry).
7. Full suite incl. `-m slow`, `ruff`, `pyright`, `cargo test`, `cargo clippy`; Conventional
   Commit naming the unfreeze.

---

## Struck: the labile-pool design (recorded, not deleted)

The first draft of this plan (2026-08-14, before the advisor review and probe) proposed a new
`labile_c` stock and a new `GrossAssimilation` flow, splitting gross uptake from partitioning
so the two could be rate-classed apart:

| Flow | Legs | Rate class |
|---|---|---|
| `GrossAssimilation` (new) | `co2_atmos → labile_c`, `+O₂` at PQ=1, sized by `GASS` | fast |
| `MaintenanceRespiration` (re-pointed) | biomass/labile → resp sink, `−O₂` | fast |
| `Allocation` (gas legs removed) | `labile_c → organs`, DVS-partitioned | slow |
| `GrowthRespiration` (re-pointed) | the `Yg` cost on the allocated increment | slow |

**Why it was struck:** finding 5. The design was built on finding 1's attribution — that the
gross/net collapse was the defect — and that attribution was wrong. Netting `GASS` against
`MRES` on one well-mixed pool is physically correct; the pool cannot observe the round trip.
The real gate was the forcing. Once the light path opens that gate, the existing
`MaintenanceRespiration` sealed branch supplies the night-time CO₂ release and O₂ draw with
no new stock, no new flow, and no new free parameter.

**Keep for:** if partitioning must later be held to a strictly daily cadence while gas
exchange runs fast, this is the structure that does it — with hazard 8 (drain cadence) and
the carry-over question ([E]'s family does not carry assimilate across days; carrying it
introduces an uncited rate constant into a tree that already has 55) attached.

**The transferable lesson, and it is the third instance:** `canopy-regulator-diagnosed`,
`stem-reserve-form-is-on-the-shelf`, and now this. **Before designing a mechanism, check
whether it is already in the tree and merely unreachable.** Here the mechanism was not only
present but cited, throttled, and conservation-balanced — and one grep of its enabling
condition (`shortfall > 0`, never true) would have found it before a design existed.

---

# SCOPE BROADENED (same day) — the light path must be AUTHORABLE

## Charge addendum

> "we should be able to simulate different scenarios - closed biosphere on earth - earth
> light curve (varying by distance from equator and season), biosphere on an alien world,
> station in deep space or underground with synthetic lighting and custom curve."

and, when told the author-facing half was a separate contract:

> "most things should be authorable. this doesn't refuse the role of validated goldens."

⇒ The authoring unfreeze is **IN**, and the user has explicitly reaffirmed the project's
standing `authored ≠ validated` split rather than asking to weaken it: the platform grows,
the frozen reference stays the validated core. No conflict to resolve.

## FINDING 7 — the four worlds split three ways, and two of them are nearly free

| The user's case | What it needs | Where |
|---|---|---|
| Earth, varying by latitude + season | `weather.daylength_seconds` **already takes latitude**; the Earth constants `0.409` (obliquity, rad) and `365` (year length) become parameters | biosphere unfreeze |
| Alien world | the *same* parameters, different values — obliquity, year length, **day length** | biosphere unfreeze |
| Deep space / underground, synthetic lighting | **already ships** — `station/lighting.py` overrides PAR *and* daylength from the lamp, with a golden. Missing only the within-day **top-hat**, which its own docstring records as inexpressible | biosphere unfreeze (the same light-path work makes it expressible) |
| Custom author-supplied curve | `ForcingSpec` must grow past `const` | **authoring unfreeze** |

`authoring/schema.py:167`: *"A forcing schedule. Step 0 = constant forcings only (`const`).
Computed schedules (the Power half-sine, biosphere weather) are a later step."* ⇒ the fourth
case is the platform's own deferred item, now taken.

⚠ **Two ceremonies, not one.** The biosphere unfreeze and the authoring unfreeze have
different justifications, different manifests and different gate reports. Run them as two
commits with two ceremonies even though they land in one batch — a single merged ceremony
cannot cleanly report either contract's gates.

## FINDING 8 — ⚠ A STALE SCOPE CLAIM IN THE LIGHTING SEAM. `daylength_s` HAS THREE READERS, NOT ONE.

`station/lighting.py` states, as the basis for its design:

> "The only runtime consumer of `daylength_s` is photosynthesis (phenology / transpiration /
> net-radiation do not read it), so 'day = lamp photoperiod' is consistent everywhere it is
> read."

**Measured false.** `grep` of every reader:

| Reader | Use |
|---|---|
| `carbon_budget.py:246` | the photosynthesis integration window |
| `photosynthesis.py:172,201,219` | the `× daylength_s` daily multiplier |
| **`phenology.py:455-458`** | **the photoperiod response** — `env.get(self.daylength_var) / 3600.0`, wired at `plants.py:334` whenever `scenario.photoperiod` is set |

The photoperiod-sensitive development path (`bucket3-scope-b-increment1`) was added *after*
that sentence was written and made it stale — the `o2-makeup-reversal-inside-the-freeze`
shape exactly: **a scope claim is dated to the roster that existed when it was written.**

### ⇒ DECISION: `daylength_s` survives as a pure photoperiod signal.

- It **stops** being photosynthesis's integration window (sub-stepping does that integral).
- It **stays** the photoperiod that phenology reads, and the lamp keeps setting it — so the
  grow-lamp still drives flowering, which is the behaviour `lighting.py` intends and which
  deleting the var would have silently broken.

Correct the stale sentence in `lighting.py` as part of this work.

## FINDING 9 — PAR = 0 is safe, and has exactly ONE consumer (measured)

Before making PAR zero for half of every day — which has never happened in this project's
history — every reader was enumerated and the zero case run:

- Readers of `par`: **one** (`carbon_budget.py:242` → `daily_canopy_assimilation`). The other
  hits are the schedule *wiring* (`season.py`, `sealed.py`) and the perturbation wrappers.
- `gross_leaf_assimilation(ci=250, par=0)` → **0.0**; `daily_canopy_assimilation(par=0)` →
  **0.0** at LAI 0.5 and 5.191. No raise, no NaN, no negative.

⇒ finding 5's night branch activates without a division-by-zero or a negative-assimilation
path anywhere. ⚠ Note the guard that makes this true is `lai == 0 → return 0.0` plus
`mean_absorbed_par = incident_par · f_int / lai`; a future refactor that removes either
re-opens the question.

## Design — the light path becomes a first-class, parameterized object

Replace "PAR is a daily scalar" with a **light path** evaluated at time-within-day, in kinds:

| kind | params | serves |
|---|---|---|
| `constant` | intensity | back-compat; today's behaviour |
| `orbital` | latitude, **obliquity**, **year_length**, **day_length**, peak intensity | Earth (defaults) and any other world |
| `photoperiod` (top-hat) | on-intensity, on-window start + duration | grow lamps, underground, deep space |
| `table` | an author-supplied within-day curve | the custom case |

`orbital` generalises the existing FAO-56 geometry: `decl = obliquity · sin(2π · doy /
year_length − phase)` with Earth = `obliquity 0.409 rad`, `year_length 365`, `day_length
86400 s` reproducing `weather.daylength_seconds` **exactly**, so the Earth default is a
refactor and only the non-Earth values are new.

⚠ **`day_length` is the one that reaches the driver.** `run_master_day` hard-requires
`fast_dt · steps_per_day == 86400`. A world with a non-24 h day breaks that constant. Price
it as part of stage 0 (the time-unit refactor) — the master "day" becomes the world's
rotation period, not the literal number 86400. **This is the deepest structural reach of the
whole plan and must not be discovered late.**

---

# BUILD, 2026-08-14 — stages 3 + 1 built and MEASURED; nothing regenerated

**Status: `src/` changed, every golden still the committed one.** The measurements below
were all taken before a byte of golden was regenerated (`soil-layers-built`: predict the
diff first). Stage 0 and stage 2 are **not** built — see finding 11 for why they are not
preconditions of this half, and why stage 2 is a bigger object than the plan priced.

## FINDING 10 — ⚠ THE SINUSOID HAS TWO RENDERINGS AND THE PLAN'S IS UNSHIPPABLE

A forcing schedule is a function of the integer step and is piecewise-constant across it
(#14), so "PAR follows a half-sine" must be handed over as **either** the instantaneous
value at the step-entry instant (the plan's form, and what probe 1 integrated at 60 s)
**or** the analytic mean of the sine over the step window. At the step this project ships
they are not close. Ratio of the day's gross assimilation to today's, at peak LAI:

| form | `dt=1` | `dt=¼` (shipped) | `dt=⅛` | `dt=1/96` |
|---|---|---|---|---|
| instantaneous, 13.3 h day | **0.0000** | 0.9160 | 0.9914 | 0.9900 |
| instantaneous, 16.5 h day | **0.0000** | **1.0385** | 0.9934 | 0.9897 |
| window mean, 13.3 h day | 1.0183 | 0.9965 | 0.9917 | 0.9899 |
| window mean, 16.5 h day | 1.0129 | 0.9935 | 0.9913 | 0.9897 |

The instantaneous form is **sampling luck**: an 8 % loss on one day of the year and a
4 % *gain* on another, from the same code, and at `dt = 1` it returns exactly zero
because the one sample lands at midnight. Its golden diff would record the grid, not the
science. The window mean conserves the day's photon dose **exactly at any step** (the
window means are a partition of the same integral) and converges monotonically to the
60 s answer. ⇒ **the window mean is what shipped**, and the instantaneous form is
recorded as measured-and-refused rather than as a road not taken.

⚠ **The Jensen correction is therefore step-limited.** At `dt = ¼` about half of the
converged pointwise loss at peak canopy is resolved. That is a fact about the step, not
about the science, and it must not be reported as "the cited direction did not appear".

## FINDING 11 — ⚠⚠ A FAST SUB-STEP CANNOT SEE A LIGHT CURVE. STAGE 2 IS GATED ON THIS.

`Substepper.substep` advances amounts and **keeps `State.n`** (that is what makes the
single `n -> n+1` commit the driver's), and `BoundEnvironment.get` resolves a forcing as
`schedule(snapshot.n, dt)`. So **every fast sub-step inside one master step is handed an
identical `(n, dt)` pair** and therefore an identical PAR. Moving the gas flows into the
fast registry — stage 2, exactly as the plan describes it — would hand the light curve to
the one operator structurally unable to see it.

The fix is boundary-side and cheap in principle (build `steps_per_day` resolvers, one per
sub-step index, and let the driver pick), and it must **not** be a `Schedule` signature
change in `simcore/environment.py` — that is the frozen Phase-0 interface, and it would
turn a boundary unfreeze into a core one. Advisor-confirmed 2026-08-14.

⇒ **Stage 3 needs none of it**, and that is why this half shipped alone: both run paths
already advance `n` through `step_report` (standalone `run_season`; the station's
biosphere as the *slow* domain), so a within-day schedule reading `t = n·dt` works today.
Hazards 6 (`run_season` has no fast path) and 7 (days-vs-seconds at the registry
boundary) belong to stage 2, not here, and stage 0 is not a precondition of anything
built here.

⚠ **And stage 2 is not the cheap route the plan assumed.** The gas legs are produced by
the three flows that *are* the carbon budget (`Allocation`, `GrowthRespiration`,
`MaintenanceRespiration`, sharing one `CarbonContext.budget()`), so "gas fast, growth
slow" is not a partition of this tree — rate-classing gas at one minute means evaluating
the most expensive flows 1440×/day against 4 today. Measured for comparison: the whole
biosphere at `dt = 1/32` costs **8×** (a 3-year sealed season, 0.60 s → 5.92 s). A finer
step for everything is cheaper than a fast lane for the expensive part.

## FINDING 12 — the measured before/after (Euler, committed weather, nothing regenerated)

Baseline is the committed tree run from a worktree at `82d965c`; both sides use the same
probe. `open_season` peak W is in t/ha (the Greenwood tripwire's own unit, crossing
14.4248); `min ppm` is the CO₂ compensation-point band's own quantity (floor 61.07).

| | `dt=¼` | `dt=⅛` | `dt=1/16` | `dt=1/32` |
|---|---|---|---|---|
| **open_season peak LAI** — baseline | 5.5719 | 5.5896 | 5.5984 | 5.6028 |
| — light path | **5.3806** | **4.8598** | **4.7278** | **4.7132** |
| **open_season peak W (t/ha)** — baseline | 14.1077 | 14.1350 | 14.1481 | 14.1546 |
| — light path | 13.7402 | 13.1906 | 13.0220 | 13.0000 |
| **sealed min CO₂ (ppm)** — baseline | 76.820 | 76.962 | 77.036 | 77.074 |
| — light path | 71.278 | 72.615 | 74.098 | 74.674 |
| **perennial min CO₂** — baseline / light path | 75.476 / 70.492 | 75.652 / 72.120 | 75.703 / 73.461 | 75.730 / 74.363 |
| **consumer min CO₂** — baseline / light path | 74.421 / 73.813 | 74.536 / 74.268 | 74.593 / 74.664 | 74.619 / 74.869 |

`rationed == 0` and no extinction events everywhere, at every step, on both sides.

## FINDING 13 — ⚠⚠ THE CONTROL: THE CANOPY LOSS IS THE CONCAVITY, NOT THE NIGHT

The light path changes two things at once and they must not be reported as one. The
control is a **top-hat at the daytime mean** — the committed tree's own PAR value,
confined to the daylight window — whose daily gross assimilation is the committed tree's
and which differs from it *only* by having dark steps. `open_season` peak LAI:

| | `dt=¼` | `dt=⅛` | `dt=1/32` |
|---|---|---|---|
| baseline (flat, no night) | 5.5719 | 5.5896 | 5.6028 |
| **top-hat** (same daily carbon, **with** night) | 5.9878 | 5.7451 | 5.5414 |
| sine (night **and** concavity) | 5.3806 | 4.8598 | 4.7132 |

**Night respiration is very nearly inert on the canopy** — at the converged step it costs
1.1 % of peak LAI (5.6028 → 5.5414); its apparent *gain* at coarse steps is the
straddling-window artifact, and it shrinks as the step does, which is how you tell the
two apart. **The concavity is the whole effect**: 5.5414 → 4.7132, **−14.9 %**, converged.

⚠ This is the compounding the plan pre-registered ("a 3.6 % early loss can compound to
more than 1.1 % at peak") and it compounds **four-fold**: a ~1 % pointwise loss at peak
canopy and ~3.5 % early becomes ~15 % of peak LAI, because the early loss lands on the
phase that *sets* the canopy (`wheat-partition-backfill-refused`). ⇒ **the pointwise
multiplier of finding 6 was never a bound on the season, and quoting it as one would have
under-priced this change by an order of magnitude.**

## FINDING 14 — ⚠⚠ THE SCIENCE GATE: THE PEAK-LAI BAND HOLDS AT THE SHIPPED STEP AND FAILS AT EVERY FINER ONE

⚠ **Measured with the band's OWN arithmetic, after an advisor challenge that this finding
was about to be written from a differently-configured probe.** The band's locus
(`tests/test_senescence_form.py::test_frozen_open_season_canopy_is_physical`) has its own
`_run` / `_peak_lai`; those two helpers were imported and re-run at each step rather than
trusted to agree with the probe. **They reproduce it cell for cell** — so the numbers
below are the gate's, not a lookalike. (`acceptance-gate-diagnosed`: measure a gate with
its own arithmetic. The check was cheap and it could have gone the other way.)

`science_bands.open_season` holds `5.0 < peak LAI < 8.0` ("real wheat peaks at ~5–8 LAI")
and `peak < 6.0` (Van Keulen & Seligman's mutual-shading threshold):

| | `dt=¼` (shipped) | `dt=⅛` | `dt=1/16` | `dt=1/32` |
|---|---|---|---|---|
| baseline | 5.5719 PASS | 5.5896 PASS | 5.5984 PASS | 5.6028 PASS |
| light path | **5.3806 PASS** | 4.8598 **FAIL** | 4.7278 **FAIL** | 4.7132 **FAIL** |

**At the shipped step the gate is green, and the suite confirms it** (both band tests
pass on the working tree). But the pass is worth exactly what the step is worth: the
observable is still moving **15 %** between `¼` and `1/32`, while the baseline's moved
0.6 %. ⇒ the honest statement is **not** "the band holds"; it is *the band holds at the
step we ship and the converged answer is below its floor*, which is the
documented-allowance shape (2026-08-13), not a red gate — **and not a clean bill either**.

**This is the plan's own biggest live risk, arriving where it said it would.** Of the two
pre-registered readings, one is **eliminated by measurement**: the canopy regulator
(`canopy-regulator-diagnosed`) is a **5 %/day leaf-area LOSS above LAI 6** — it can only
push a canopy down, and cannot be the fix for one that fell to 4.71, which is further from
its threshold than before. What remains is the other reading: **the band was clearing
against a diurnally biased (high) assimilation, and what the light path exposes is a
canopy this tree cannot grow** — a missing growth mechanism, not a wrong light path.
Retuning the bound so the change fits is the co-adaptation shape this project has refused
three times, and it is not on the table.

⚠ Note which way the other tripwires move — all of them *away* from danger. The Greenwood
peak-W crossing (14.4248) goes from **2.20 %** of margin to **4.75 %** at `dt = ¼` and
**9.88 %** at `dt = 1/32`; every CO₂ compensation-point band still clears its floor at
every step. So no gate is red today, and the one that is *converging* toward red is the
canopy floor.

## FINDING 15 — THE CHARGE'S OWN OBSERVABLE, MEASURED: THE CHAMBER BREATHES

Sealed chamber, day 200, `dt = 1/32` — the night branch running for the first time in
this project's history (CO₂ mol in the shared pool, O₂ its exact mirror at PQ = 1):

```
  hour       PAR    CO2 (mol)        dCO2     O2 (mol)         dO2
  0.00      0.00     0.085769  +0.0007469     2.271231  -0.0007469
  3.75      0.00     0.089502  +0.0007459     2.267498  -0.0007459
  6.00    459.05     0.090957  -0.0007004     2.266043  +0.0007004
 12.00   1544.85     0.084359  -0.0006651     2.272641  +0.0006651
 18.00    202.87     0.080158  +0.0002239     2.276842  -0.0002239
 23.25      0.00     0.084831  +0.0007458     2.272169  -0.0007458
```

CO₂ climbs all night and falls all day; O₂ is its exact negative. ⚠ **And the diurnal
swing is already essentially converged at the shipped step**: 0.010690 mol at `dt = ¼`
against 0.010799 at `dt = 1/32` (day 200). The season's *canopy* needs a finer step; the
season's *breathing* does not.

⚠ **One correction to this document's own finding-5 framing, found by measuring**: the
night branch is gated on `shortfall = MRES − GASS > 0`, **not** on a fully dark step. Dim
steps at dawn and dusk cross it too. So the earlier count of "fully dark steps per season"
(55 % of night hours at `dt = ¼`, with 133 days having none at all) is a **lower** bound
on when the mechanism runs, not a measure of it — at `dt = ¼`, day 200 has no dark step
and still breathes. The claim "the light path is necessary but not sufficient at `¼`" was
too strong and is withdrawn here rather than left standing in the module note.
