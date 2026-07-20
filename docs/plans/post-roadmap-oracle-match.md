# Post-roadmap: the full oracle match — bucket 3 scope (B)

**Status: IN PROGRESS. Increment 1 (vernalization) started 2026-07-20.**

The scope-(B) option recorded in [`post-roadmap-validation.md`](post-roadmap-validation.md)
("The scope decision"), taken by the user after scopes (A) and (C) closed. That plan's
diagnosis is the input to this one and is **not restated here** — read its findings 1–3
first. This doc owns only what (A) left open: the science, the decisions, and the outcome.

**This is a phase, not a step.** It is the first post-roadmap work to add **science**
rather than reach or expressiveness, and the first to run the biosphere unfreeze
discipline for a *structural* change rather than a param edit.

## The charge, restated in the form the diagnosis left it

Scope (A) measured the oracle gap and ranked its causes. Two are missing science, one is
param values:

1. **The canopy never bootstraps** (DOMINANT) — 1.75 % light interception at sowing vs
   the oracle's 97.8 % at peak; LAI peaks day 32 of ~305 and collapses before anthesis.
   A source-limited death spiral.
2. **Phenology runs ~1.6× fast** — anthesis day 138 vs the oracle's 217, **79 days
   early**. There is no vernalization term anywhere in `src/`.
3. **Param values** — real, but the *third* cause.

(1) and (2) are **independent**: DVS runs on thermal time, which is independent of
biomass. So they are changed one at a time.

## The two decisions taken up front (2026-07-20)

Both were put to the user before any science was written, on the advisor's argument that
starting calibration against an undefined target is how recalibration chases noise
forever.

> **⚠ DECISION 1 WAS SUPERSEDED ON THE SAME DAY — see "The bar, decided" below.** It is
> kept here unedited because the *reasoning* is what licensed the two increments, and
> because the route by which its trigger fired is itself a finding: the bar was deferred
> "until increment 2 measures the residual", **increment 2 then dissolved**, and the
> residual arrived anyway from a direction nobody planned.

### Decision 1 — the acceptance bar is DEFERRED until increment 2 has measured the residual

**There is no acceptance bar in the tree, and this was verified rather than assumed.**
`tests/test_oracle_match.py` tests the *helper* (`lab/oracle_match.py`) on synthetic
series — it never compares our trajectory to the oracle. The **only** live comparison in
the whole tree is `tests/test_oracle_smoke.py:120`, `nrmse(oracle, our) < 0.5`, which its
own module docstring calls a deliberately loose band.

That matters more than a missing number, because Phase 1 recorded a *ruling* that scope
(B)'s third increment would collide with
([`phase-1-single-producer.md`](phase-1-single-producer.md), Step-11 design):

> **Oracle match = QUALITATIVE / loose smoke check; the tight quantitative gate is
> [deferred]** … *clean-room forbids backfitting to WOFOST* … the observed gap is
> recorded as a **finding, not tuned away**.

So "then recalibration", as the `CLAUDE.md` status row words scope (B), is in tension
with a ruling still on the books. **The user chose to defer the bar until the residual
gap after both structural fixes is a measured number** — this project's own
"measure before you rank causes" discipline (scope (A)'s transferable finding) applied to
its own scope decision. The phase is therefore **2 increments + an open decision**, not
3 increments.

**Consequence for increments 1 and 2: no param is moved to chase the oracle.** Both
increments add structure and cite their params to primary literature. Whatever gap
remains is re-measured and *then* the calibration question is put to the user with a
number behind it.

### Decision 2 — vernalization first, canopy second

The advisor's ordering, confirmed by the user: vernalization is smaller, is an
**already-documented seam** with a known published form, and its precondition is now
measured (below). The canopy fix is dominant but is a **regime switch, not a param** —
new growth logic touching allocation/season, and the half most at risk of accidentally
reinventing PCSE.

Each science lands as its **own** unfreeze increment — own advisor review, own golden
regeneration, own manifest bump. A combined regeneration would mix two effects and make
it impossible to attribute what moved. (The multi-rate phase's 7-step staging is the
house precedent.) Because matched-DVS comparison is conversion-free, the canopy fix stays
validatable independent of phenology timing, so the order is a risk choice, not a
dependency.

---

# Increment 1 — vernalization

## The precondition, measured before implementing (not assumed)

Vernalization is a no-op without a cold winter. The advisor flagged this as a five-minute
check that could invalidate the whole science. Measured against the committed fixture
`tests/oracle/winter_wheat_weather.json` (305 days from 2006-10-01):

| quantity | value |
|---|---|
| temperature range | −1.8 … 22.2 °C |
| days in the vernalization-effective −1…8 °C window | **105** |
| days below 5 °C | 38 |
| day 60–180 mean | 5.4 … 7.1 °C |

**The precondition holds with room to spare.** This is a real winter, and winter wheat
saturates its vernalization requirement at ~50 vernalization-days.

## The source, and why this one

**Soltani, A. & Sinclair, T.R. (2012), *Modeling Physiology of Crop Development, Growth
and Yield*, CABI, Wallingford. ISBN 978-1-84593-970-0. Chapter 8, "Phenology —
Vernalization", pp. 90–93.**

Retrieved from `sources/` (already on the shelf — one of the ~15 scope-B references
stocked during scope (C) round 2). Three candidate sources were checked by extraction;
Penning de Vries 1989 and Teh 2016 mention vernalization **twice each** and give no
formulation, while Soltani & Sinclair carries a full chapter (64 hits) with equations, a
cardinal-temperature figure, and a **parameter table by cultivar**. It is a modeling
monograph presenting published equations with primary attributions (Ritchie 1991; Roberts
& Summerfield 1987; Porter & Gawith 1999) — **not** PCSE source and not the unlicensed
WOFOST param YAML, so it satisfies `docs/reuse-and-licenses.md`.

Extraction quality was verified before the numbers were trusted (scope (C) round 6's
lesson — extraction can silently destroy exactly the digits at issue): **0 U+FFFD
characters**, 33 947 digits recovered, and both equations render as readable text rather
than as figure images.

## The formulation (transcribed, with equation numbers)

**Eqn 8.3 — vernalization day.** A 3-segment linear response with four cardinal
temperatures: base `TBV`, lower optimum `TP1V`, upper optimum `TP2V`, ceiling `TCV`:

```
VERDAY = 0                              if TMP <= TBV
       = (TMP − TBV) / (TP1V − TBV)     if TBV < TMP < TP1V
       = 1                              if TP1V <= TMP <= TP2V
       = (TCV − TMP) / (TCV − TP2V)     if TP2V < TMP < TCV
       = 0                              if TMP >= TCV
```

**Cumulative:** `CUMVER_i = CUMVER_{i−1} + VERDAY` (p. 92).

**Eqn 8.6 — the vernalization function**, from cumulative vernalization days `CUMVER`,
the saturation requirement `VDSAT`, and a sensitivity coefficient `vsen`:

```
verfun = 1 − vsen · (VDSAT − CUMVER_i)    if CUMVER_i < VDSAT
       = 1                                if CUMVER_i >= VDSAT
```

`verfun ∈ [0, 1]` and multiplies the development rate (Eqn 8.2), **fixed at 1 during
phases when the crop is not sensitive** (p. 91: wheat is insensitive sowing→emergence and
from terminal spikelet→maturity).

### The parameter values, and the ranges around them

**Cardinal temperatures — wheat, Fig. 8.1 (p. 91):** temperatures below −1 °C or above
12 °C do not contribute; 0–8 °C is the full-effect optimum. So `TBV = −1`, `TP1V = 0`,
`TP2V = 8`, `TCV = 12`. Consistent with the book's cited optimum of 0–8 °C (Ritchie 1991).

*The literature range, recorded per the calibration discipline:* Porter & Gawith (1999),
reviewing the wheat literature, report −1.3 / 3.8 / 6.0 / 15.7 °C, and APSIM uses 0–15 °C
peaking at 2 °C (Robertson et al. 2002b; Keating et al. 2003). Fig. 8.1 is taken as
primary because it is the formulation's own wheat parameterization; the alternatives are
recorded so that a later calibration knows the sanctioned range rather than inventing one.

**Response parameters — Table 8.1 (p. 93), row "Wheat / Winter Europe":**
`vsen = 0.033`, `VDSAT = 50`. The winter-Europe row is the correct one for a winter wheat
sown 1 October on European weather. `VDSAT = 50` is independently the book's headline
wheat value (Ritchie 1991, p. 92); Baloch et al. (2003) report 60–70 for Pacific Northwest
wheats, and the table's 18 rows span `vsen` 0.0012–0.04.

Note what these values imply: at `CUMVER = 0`, `verfun = 1 − 0.033·50 = −0.65`, clamped
to 0. Winter Europe wheat is therefore **qualitative** in the book's own terminology
(Fig. 8.2) — development is *fully arrested* until ~19.7 vernalization-days accumulate.
That is a property of the cited parameterization, not a modeling choice of ours.

## The feasibility number, computed before writing code

The formulation above, run against the committed weather fixture:

| quantity | value |
|---|---|
| total CUMVER over the season | 143.4 vernalization-days |
| `verfun` first exceeds 0 | day **57** |
| CUMVER saturates (≥ 50) | day **91** |
| days of complete arrest (`verfun == 0`) | **57** |
| days of partial development (`0 < verfun < 1`) | 34 |

**57 days of complete developmental arrest plus 34 partial days lands at roughly 74 days
of delay, against a measured gap of 79 days.** Vernalization alone plausibly closes nearly
the whole phenology overrun — which is what finding 2 predicted and is the reason this
increment is worth its ceremony.

⚠ This is a *prediction from the forcing alone*, not a result. It ignores the feedback
that a longer vegetative phase has on everything downstream. The measured outcome is
recorded below once the implementation runs.

## Design — how it maps onto our channel

**The seam already exists and needs no API change.** `phenology.py`'s module docstring
documented vernalization as a deferred "second state accumulator" with a derived
`VERNFAC ∈ [0,1]`, and `ThermalTimeAccumulation.evaluate(snapshot, env, dt)` already
carries `snapshot` for exactly this purpose. P2 names the channel "non-conserved scalar
accumulator**s**" (plural), so a second accumulator is an **extension of the channel, not
a violation of it**.

Four pieces:

1. `vernalization_day(temp_c, ...)` — Eqn 8.3, a pure function beside
   `daily_thermal_time`.
2. `vernalization_factor(cum_verday, ...)` — Eqn 8.6, clamped to `[0, 1]`.
3. `VernalizationAccumulation` — a second `AuxProcess` writing the `vernalization_days`
   accumulator in increment form, exactly mirroring `ThermalTimeAccumulation`.
4. `ThermalTimeAccumulation` becomes vernalization-aware: it reads both accumulators off
   `snapshot.aux`, derives the current DVS, and multiplies its rate by `verfun` **only in
   the vegetative phase**.

**Why the factor multiplies the thermal-time increment.** The book applies `verfun` to
the *development rate* (Eqn 8.2). Our DVS is derived from `thermal_time` rather than
integrated directly, so the thermal-time increment is the equivalent lever — scaling it
scales DVS's rate of advance identically. This is a faithful re-expression of Eqn 8.2 in
our channel's idiom, recorded because the two forms are not obviously the same thing.

### Three deliberate simplifications, each forced by the available forcing data

Recorded as decisions rather than silently taken:

* **Crown temperature ≈ air temperature.** The book prescribes crown temperature `Tcr`
  because the growing point sits below the soil surface, and notes that soil surface
  temperature *is* similar to air temperature except where **snow cover** causes the two
  to diverge (p. 92). The fixture carries no snow or precipitation variable, so the
  divergence term is unrepresentable — and the fixture's minimum is −1.8 °C, so persistent
  snow cover is not implied either.
* **De-vernalization is not implemented.** Eqn 8.5 reduces CUMVER when `CUMVER < 10` and
  **TMAX** > 30 °C. The fixture carries daily-*mean* `TEMP` only — there is no TMAX — so
  the term is **unimplementable**, not merely omitted. It is also **inert on this
  weather**: the seasonal maximum daily mean is 22.2 °C, so the `> 30 °C` trigger could
  not fire even if TMAX were available.
* **The sensitive window is gated at DVS < 1 (anthesis), not terminal spikelet.** The book
  ends wheat's vernalization sensitivity at terminal spikelet (Ritchie 1991) or anthesis
  (Wang & Engel 1998) — it cites **both**. Our model has no terminal-spikelet stage; DVS
  runs 0 → 1 → 2 over emergence → anthesis → maturity. Gating at DVS < 1 adopts the
  Wang & Engel end point, which is the one our stage set can express exactly. Once CUMVER
  saturates on day 91 the factor is 1 for the rest of the vegetative phase anyway, so on
  *this* weather the two end points are indistinguishable — but the choice is recorded
  because another weather series could separate them.

## Exit criteria for increment 1

- [ ] The science is clean-room from Soltani & Sinclair Ch. 8 with equation-level cites;
      **never** reverse-engineered from PCSE.
- [ ] New params carry real citations, not `TODO(cite)` — this increment *retires* debt.
- [ ] **No param is moved to chase the oracle** (decision 1).
- [ ] `git diff src/simcore/` empty.
- [ ] Hand-mirrored into `rust/crates/domains`; cross-port parity holds.
- [ ] The full unfreeze ceremony run: advisor review *before* regenerating, then the 7
      biosphere goldens + the cascaded station goldens, then the manifest, then
      provenance, then the gates.
- [ ] `tests/test_oracle_gap.py` updated to the new measured numbers — **it pins
      known-wrong behavior, so fixing a gap turns it red on purpose**. Update the numbers
      and the prose; never delete the test.

---

# OUTCOME — increment 1 (measured 2026-07-20, BEFORE the unfreeze ceremony)

> ## ⛔ READ THIS FIRST — the endpoint match below is a FALSE POSITIVE
>
> Vernalization moves our anthesis from 79 days early to 3 days late. **That match is
> achieved by a mechanism the oracle does not have**, and the increment is therefore
> **NOT accepted**: no golden was regenerated and the ceremony was stopped at step 1.
>
> The oracle (WOFOST 7.2 `Winter_wheat_101`) **does not arrest development in winter** —
> it runs a **photoperiod**-modulated rate. Evidence in "The check that stopped the
> ceremony" below. Everything in "What it was aimed at" is arithmetically correct and
> **scientifically misattributed**; the internal-coupling finding after it survives
> intact, because it never referenced the oracle.

**The science landed, and it did something the diagnosis said it would not — and then a
review check showed it landed for the wrong reason.**

## What it was aimed at: the phenology overrun, essentially closed

| stage | oracle | ours BEFORE | ours AFTER | error before → after |
|---|---|---|---|---|
| DVS 0.5 | 193 | 47 | 170 | −146 d → **−23 d** |
| DVS 1.0 (anthesis) | 217 | 138 | **220** | −79 d → **+3 d** |
| DVS 2.0 (maturity) | 292 | 218 | 267 | −74 d → **−25 d** |

**Anthesis lands within 3 days of the oracle**, from 79 days early. The prediction made
from the forcing alone (~74 days of delay against a 79-day gap) was accurate. Cumulative
vernalization over the season is 143.4 days against a 50-day requirement.

The residual is a *shape* error, not a rate error: the vegetative phase is still slightly
short (−23 d at DVS 0.5) and grain fill still runs fast (anthesis +3 d → maturity −25 d,
so our reproductive phase is 47 days against the oracle's 75). That is a `tsum_maturity`
question — i.e. **cause 3, param values** — and it is deliberately not touched here
(decision 1).

## ⚠ THE FINDING: scope (A)'s independence claim is FALSIFIED, and the canopy gap fell ~10×

Scope (A) finding 2 states, in bold:

> **DVS runs on thermal time, independent of biomass — so fixing phenology does not fix
> finding 1, and vice versa.**

**This half survives the check below, and is stated deliberately WITHOUT reference to the
oracle** — it is a before/after on our own model, so it holds whatever the oracle's
mechanism turns out to be. (The oracle-relative version, "the peak-LAI gap fell 43.4× →
4.4×", is *not* used: it imports a target whose validity the next section puts in
question. Advisor catch.)

Measured, changing **only** phenology and **no param value**:

| canopy metric (ours) | BEFORE | AFTER | change |
|---|---|---|---|
| peak LAI | 0.146 (day 32) | **1.441 (day 254)** | **9.9×** |
| peak light interception | 5.00 % | **57.9 %** | 11.6× |
| leaf at season end, as fraction of peak | < 20 % | **40 %** | spiral gone |

**The dominant cause fell by an order of magnitude as a side effect of fixing the second
cause.** `tests/test_oracle_gap.py::test_method_the_death_spiral_mechanism` now **fails**:
leaf ends the season at 40 % of its peak, not below 20 %. *The death spiral does not
happen any more.*

**Why the claim was wrong, and the shape of the error.** The premise is true — DVS is
computed from thermal time and never reads biomass. The conclusion does not follow,
because **the dependency is one-directional, and "and vice versa" asserts it is
symmetric**. Biomass does not affect DVS, but DVS strongly affects biomass: `Allocation`
reads DVS to interpolate the partition fractions `fl/fs/fr/fo`. With DVS racing, the crop
left the high-`fl` vegetative phase around day 47, leaf allocation fell below the 2 %/day
leaf death rate, and *that* started the spiral. Holding it in the vegetative phase for
170 days keeps `fl` high, and the spiral never starts.

So the source-limited death spiral was **substantially a consequence of the phenology
overrun**, not an independent structural gap. Scope (A) ranked the canopy first and
phenology second; the ranking is right about *magnitude* and wrong about *causal order*.

**This is the meta-finding's eleventh instance, and the first where the falsified claim
is a scientific one rather than a doc/status claim.** Its distinctive mechanism, worth
carrying: **an asymmetric fact was written down as a symmetric claim.** "A does not
depend on B" was extended to "so they are independent" — and the extension is the part
that was never measured. The tell is the phrase *"and vice versa"* doing unearned work
after a correctly-argued one-way statement. Scope (A) could not have caught it without
running exactly this experiment, but it could have *scoped* the claim to the direction it
had evidence for.

**Consequence for increment 2 — flagged, but NOT yet actionable.** It is tempting to
conclude "increment 2 is now much smaller"; that conclusion is **contingent on the
phenology fix being the right one**, and the next section shows it is not. What can be
said oracle-independently is that *a* phenology slowdown, whatever its mechanism, largely
dissolves the death spiral by keeping `fl` high — so increment 2's brief must be
**re-derived after the phenology mechanism is settled**, never inherited from scope (A).
The surviving defect also looks different in kind (our LAI now peaks 34 days *after* our
own anthesis, where wheat should peak at or just before it and decline through grain
fill) — a senescence/allocation *timing* question rather than a failure to bootstrap.

## The check that stopped the ceremony (advisor, at the pre-regeneration review)

The advisor asked a question I had raised in passing and walked past: **the oracle's crop
identity.** An early note in this session read *"the bundled oracle is
lintul3_springwheat … but the season is winter wheat sown 1 Oct"*, and it was not chased.
Spring wheat has no cold requirement; LINTUL3 often carries no vernalization at all. If
the oracle did not model dormancy, a 57-day arrest matching its anthesis date would be
**the right answer for the wrong physics** — precisely what clean-room exists to prevent.

**Half of that dissolved on inspection, and the dangerous half did not.** The fixture's
`provenance` block — which had been visible all along and never opened — says
`model_variant: Wofost72_PP`, `variety_name: Winter_wheat_101`, sown 2006-10-01 at
lat 52. So it *is* full WOFOST 7.2 winter wheat, not LINTUL3 spring wheat. **But the crop
label was never the question; the mechanism was**, and WOFOST 7.2 carries vernalization
only when enabled.

### The oracle does not arrest — measured

| window | oracle mean dDVS/day |
|---|---|
| winter, days 60–150 | **0.00163** (nonzero) |
| spring, days 180–215 | 0.01674 |

The oracle's DVS climbs *through* the cold window (0.0302 → 0.1770), with only 24
zero-advance days in 217. Its late anthesis is a **rate** effect, not an arrest.

### And the rate it runs is photoperiod-modulated — the discriminating test

Dividing the oracle's daily DVS advance by our *unvernalized* degree-day rate isolates a
multiplier with the temperature effect removed. It correlates with **daylength at
lat 52 at r = 0.972**, rising monotonically 0.02 → 1.71 from the shortest days to the
longest.

That correlation alone would be suggestive (daylength and temperature co-vary
seasonally), so the discriminating test is what the two hypotheses do **after the cold
requirement is satisfied**:

| day | daylength (h) | our vernalization factor | oracle's implied multiplier |
|---|---|---|---|
| 100 | 7.81 | **1.000** (saturated) | 0.200 |
| 140 | 9.84 | **1.000** | 0.641 |
| 180 | 12.55 | **1.000** | 1.241 |
| 210 | 14.53 | **1.000** | 1.711 |

**CUMVER saturates on day 91. A vernalization mechanism would be flat from there — the
factor is pinned at 1 and cannot rise again. The oracle's multiplier instead keeps
climbing 0.20 → 1.71, tracking daylength the whole way.** Vernalization cannot produce
that shape; photoperiod can and does.

**Conclusion: the oracle's missing science is PHOTOPERIOD, not vernalization** — and
photoperiod is the *other* deferred seam `phenology.py` has documented since Phase 1
("a pure astronomical function (latitude + day-of-year) read via ``env.get`` — a
development-rate multiplier with **no accumulator**"). `DAYLENGTH_VAR` already exists as a
forcing.

### What this costs, and what it does not

**The two terms are substitutes for hitting the anthesis date, not complements.**
Vernalization already lands anthesis at +3 d; adding photoperiod on top would overshoot
far later. So this is a genuine fork, not an "add the other one too".

**It does not invalidate the code.** The implementation is faithful to its source (it
reproduces [C]'s worked example exactly), the params are honestly cited, and real winter
wheat genuinely *does* require vernalization — WOFOST's `Winter_wheat_101` is itself a
simplification of the physiology. What is falsified is only the claim that **this** term
is what reconciles us with **this** oracle.

**The lesson, which is the transferable part:** an endpoint match is not evidence of a
mechanism. Two different sciences moved anthesis by ~76 days from opposite premises
(total arrest vs. proportional slowing) and one of them agreed with the target to 3 days.
The diagnosis in scope (A) named "no vernalization" as the cause of finding 2 from
`grep`-level evidence — *there is no vernalization term in `src/`* — which is true, and
**says nothing about whether the oracle has one**. The check that settles it is
comparing the *shape* of the target's trajectory, not its endpoints, and it costs one
query against data that was already loaded.

## What did NOT change

* **No param value was moved to chase the oracle** (decision 1 held). The only param edits
  are the six *new* vernalization params, each cited on entry.
* `git diff src/simcore/` — empty.
* Phenology's own deferred seam list keeps **photoperiod**, untouched.

## The provenance risk, recorded rather than glossed

The source ([C] Soltani & Sinclair 2012 Ch. 8) is a **secondary source quoting
primaries** — the exact chain shape scope (C) spent six rounds on, whose record stands at
**2 sound : 1 fabricated**. What is verified **first-hand** is that our implementation
reproduces [C]'s own worked example on p. 91 exactly (5 days at 7 °C → 5 VERDAY; at 10 °C
or −0.5 °C → 2.5 VERDAY), and that the extraction was clean (0 U+FFFD over 33 947 digits).
What is **not** verified is the locus check that caught Dunn 2011: Ritchie (1991), Jones
et al. (2003) and Porter & Gawith (1999) are not in `sources/` and were not opened. Dated
residual risk, 2026-07-20 — and per scope (C) round 6, *that is a fact about this
afternoon's shelf, not a property of the literature*.

---

# The user's decision, and increment 1 as it actually shipped

Put to the user as a three-way fork (photoperiod instead / both / vernalization only).
**The user chose BOTH, explicitly accepting that "the oracle stops being usable as the
phenology target".** Physiological realism over the oracle match: real winter wheat
requires vernalization *and* is photoperiod-sensitive, and WOFOST's `Winter_wheat_101` —
which models only the latter — is itself a simplification of the physiology.

So increment 1 ships **two** terms, not one. The photoperiod half:

**Source: [C] Ch. 7, Eqn 7.6 (the long-day form), p. 78; Table 7.2 p. 84.** Wheat is a
long-day plant — development toward flowering slows below a critical photoperiod:

```
ppfun = 1 − ppsen · (CPP − PP)   if PP < CPP     (clamped to [0, 1] — the source is
      = 1                        if PP >= CPP     explicit that a negative becomes 0)
```

Params from Table 7.2 row **"Wheat / Winter Europe"** — the *same cultivar class* as the
Table 8.1 vernalization row, so the two terms are consistently parameterized:
`CPP = 16 h`, `ppsen = 0.09 h⁻¹`. Range recorded, not invented: Major & Kiniry (1991) give
CPP ≈ 17.7 h for long-day crops including wheat, Ritchie (1991) 19 h; Table 7.2's seven
wheat rows span CPP 14–17 and ppsen 0.09–0.17.

**It needed no new infrastructure.** `weather.daylength_seconds` (FAO-56) and the
`daylength_s` forcing already existed and were already wired into the season resolver.
A pleasant emergent consequence: `lighting.py` drives `daylength_s` from
`scenario.photoperiod_hours`, so a **station greenhouse's lamp schedule now controls
flowering** — physically correct, and free.

Unlike vernalization, photoperiod is an **instantaneous** driver with no accumulator — it
adds no aux state. That difference is precisely what the discriminating test above turned
on.

**Its sensitivity window carries the same approximation as vernalization's, and that is
now stated symmetrically.** The source ends wheat's photoperiod sensitivity at *terminal
spikelet* (Ritchie 1991), which is before anthesis; we gate at `DVS < 1` because our stage
set has no terminal-spikelet point. So the slowdown is applied slightly longer than
prescribed — the identical trade already documented for the cold requirement. It was
written up for vernalization only and left implicit for photoperiod (advisor catch);
recorded in `phenology.yaml` for both now, because a stage set that gained a
terminal-spikelet point would narrow **both** windows, not one.

## The measured result — and the second surprise

| stage | oracle | ORIGINAL | vernalization only | **vern + photoperiod** |
|---|---|---|---|---|
| DVS 0.5 | 193 | 47 | 170 | **211** |
| DVS 1.0 (anthesis) | 217 | 138 | 220 | **251** |
| DVS 2.0 (maturity) | 292 | 218 | 267 | **294** |

Anthesis overshoots to +34 d, exactly as predicted when the fork was put to the user
(the two terms are substitutes for hitting that date). **Maturity, however, lands on
day 294 against the oracle's 292.**

### ⚠ THE SECOND FINDING: the canopy gap closed too — with NO canopy science at all

| canopy metric | oracle | ORIGINAL | vern only | **vern + photoperiod** |
|---|---|---|---|---|
| peak LAI | 6.337 (d 212) | 0.146 (d 32) | 1.441 (d 254) | **5.191 (d 263)** |
| peak light interception | 97.8 % | 5.00 % | 57.9 % | **95.56 %** |
| peak-LAI gap | — | **43.4×** | 4.4× | **1.22×** |

**Scope (A)'s dominant, structural, "the canopy can never get off the ground" failure is
essentially gone — and not one line of canopy science was written, nor one param value
moved.** Light interception went from 5 % to 95.56 % against the oracle's 97.8 %.

**Increment 2 — "implement a juvenile canopy-expansion phase" — now looks unnecessary.**
That was the *dominant* item in scope (A)'s ranking and the larger half of scope (B) as
scoped. It appears to have been, in its entirety, a **downstream consequence of the
phenology error**: a crop that races through its vegetative phase leaves the high-`fl`
allocation region before it can build a canopy, and the 2 %/day leaf death rate then eats
what little it built. Fix the development rate and the "source-limited death spiral"
never starts.

This is the same one-directional-coupling lesson as the first finding, now with its full
consequence visible: **scope (A) ranked the canopy first because it measured magnitudes,
and magnitude is not causal order.** The measurement was right; the *ranking* invited an
inference the measurement did not support.

### What is left is genuinely cause 3 — param values

The residual is now a **phase-partition** error, which is exactly a `tsum` question:

| phase | ours | oracle |
|---|---|---|
| emergence → anthesis | 251 d | 217 d |
| anthesis → maturity | **43 d** | **75 d** |

Our reproductive phase is too short and our vegetative phase too long — i.e.
`tsum_anthesis` is too high relative to `tsum_maturity`. Both are `TODO(cite)`
placeholders (`1100` / `750`) and both are **deliberately untouched** under decision 1.
Peak LAI also still arrives 51 days after the oracle's.

**So the phase has landed where the diagnosis said it could not: two structural fixes in,
and what remains is calibration.** The acceptance-bar question the user deferred is now
live, with a number behind it.

## ⚠ THE THIRD FINDING: the perennial period-2 limit cycle was an artifact, and it is gone

The advisor's ceremony warning — *"a 57-day arrest can dominate a short-horizon chamber
run, and perennial/long-horizon now re-vernalize every cycle; conservation will not catch
a plant that never develops"* — paid off, though not where expected. The full suite came
back **693 passed, 1 failed**, and the failure is
`test_stress_perennial_period_2_sustained`: the perennial chamber's **period-2 limit
cycle collapsed to a fixed point**.

That test pinned a *documented structural property* — "a genuine period-2 limit cycle for
all 328 years, 320 yr of strict alternation, gap ~0.07 ~= 28 % of scale". So the question
is whether the new science broke the model or corrected it.

**Measured, isolating each term over 16 years of the sealed perennial chamber:**

| config | peak leaf | max adjacent gap |
|---|---|---|
| **baseline** (both inert) | 0.2530 | 7.157e-02 (**28.28 %** of scale) |
| vernalization only | 1.0171 | 4.44e-16 |
| photoperiod only | 1.0795 | 0.0 |
| both (shipped) | 1.2215 | 1.55e-15 |

Two things follow, and the first validates the second:

1. **The baseline row reproduces the committed test's own numbers exactly** ("gap ~0.07,
   ~28 % of scale" → 7.157e-02, 28.28 %). The harness is measuring the same thing the
   test was, so the comparison is sound rather than merely suggestive.
2. **Either term alone collapses the cycle**, and the system converges **upward** — peak
   leaf 0.253 → 1.222, **~4.8×**. A degenerate fixed point (a plant that never develops)
   would go the other way.

**The first hypothesis was photoperiod entrainment — measured and REJECTED.** Daylength
is a strict annual function, so "photoperiod entrains the free-running oscillation" is an
attractive and textbook-shaped story. It is also wrong: vernalization alone, which has no
annual phase structure at all, kills the cycle just as completely. Recorded because it
was *drafted as the explanation* before the isolating run, and only the fourth row of
that table stopped it — the same "plausible, confidently stated, wrong" failure mode
scope (A) recorded twice.

**The mechanism that survives** is canopy closure flattening the year-to-year return map.
At baseline the starved canopy sits at ~5 % light interception, where Beer–Lambert is
still nearly **linear** in LAI, so a good year begets a bad one with loop gain > 1.
Slowing development lets the canopy close (~95.6 % interception), where Beer–Lambert
**saturates**: a change in starting leaf barely moves intercepted light, the map's slope
at the fixed point drops below 1, and the 2-cycle loses stability. This is the same
damping story as the consumer chamber, which was already period-1 — there a herbivore
supplies the damping instead of light saturation. (Stated as the *supported* reading: the
interception numbers are measured, the return-map slope argument is reasoning from them.)

**So the period-2 cycle was an artifact of the broken canopy regime, not a property of
the perennial chamber** — a third documented behavior that turns out to have been
downstream of the phenology error. The test is **flipped, not weakened**: it still pins a
discrete structural property over the full horizon, still fails on a period break, and
gains a `max(tail) > 1.0` assertion so that a degenerate fixed point at a dead plant
cannot pass where the oscillating baseline used to. The module docstring's period-2
paragraph is corrected in place with the supersession marked.

---

# The bar, decided (2026-07-20) — and the Phase-1 tension resolved

Decision 1 deferred the acceptance bar until "the residual after both structural fixes is
a measured number". **That trigger fired by an unplanned route**: increment 2 (the canopy)
dissolved, so the measurement that was supposed to come *from* it never happened — but the
residual it was meant to expose showed up anyway, as the phase-partition error above
(reproductive phase 43 d vs 75 d). The advisor flagged that the condition was satisfied
and that re-deferring it a second time would let the user's own selected option — which
read *"forces the deferred bar decision NOW"* — pass silently.

**Two rulings, both the user's:**

### Ruling A — the ceremony is STAGED

Freeze the **structural science alone** now; the `tsum` recalibration becomes its own
later increment with its own ceremony. Costs a second station cascade, and buys the thing
the staged discipline exists for: **each golden diff is attributable to exactly one
cause.** Provenance does not regress in the meantime — the `tsum` params remain the *same*
`TODO(cite)` placeholders already sitting in the frozen tree, while the six new
vernalization + two new photoperiod params all arrive **cited**.

### Ruling B — the bar is LITERATURE RANGES ONLY; the oracle is a diagnostic, never a target

This retires the tension that has been open since Phase 1. The books said *"clean-room
forbids backfitting to WOFOST"* while the scope-B row said *"then recalibration"*, and
those could not both govern. The resolution:

> **A param may move only within a cited literature range. The oracle is read as a
> diagnostic and is never fitted to. Whatever gap remains is recorded as a finding.**

This is not a compromise between the two; it is the consistent position both were reaching
for, and it now also follows from the user's earlier ruling that **the oracle is not the
phenology target** — an instrument you have declared unfit as a target cannot coherently
be restored as a fitting objective. `test_oracle_gap.py`'s existing idiom (pin the gap as
a measured number with its cause) is exactly the right shape for it and needs no change of
philosophy, only new numbers.

**This is also the honest reading of what just happened.** The vernalization endpoint match
was a **false positive that an endpoint bar would have rewarded** — anthesis within 3 days,
by a mechanism the target does not possess. A bar that scores endpoint agreement would have
graded the wrong physics as success. Ruling B removes the incentive entirely rather than
trying to build a cleverer scorer.

---

# ⛔ THE CEREMONY IS BLOCKED — the frozen scenarios were co-adapted to the broken plant

**This is the fourth finding, it was nearly missed, and it changes the size of the
increment.**

The advisor blocked the ceremony on a process point: the earlier suite run used `-x`, so
it **stopped at the first failure** and everything collected after it never executed. The
phenology fixture and the stress test had been fixed *in isolation*, and no clean full run
existed. Entering a 20-golden regeneration with an unknown red set would conflate "this
golden moved because the science moved" with "this test needed flipping".

Full `-m "not slow"` run, no `-x`: **32 failed, 1849 passed, 37 errors.**

## The casualty set, triaged

**Expected — these ARE the unfreeze** (fixed by regenerating): all 9 `test_regression_*`
season/station goldens, the 6 `test_regression_long_horizon` errors, and
`test_freeze_manifest::test_frozen_aux_set_is_complete` (a new aux process exists).

**Expected re-pins** (they pin known-wrong behavior on purpose): 6 in `test_oracle_gap.py`,
2 in `test_oracle_smoke.py`.

**NOT expected — six failures, and they reduce to TWO root causes:**

### Root cause A — the sealed chambers' CO₂ pool is now the binding constraint

The plant is ~5× larger. The sealed chambers were sized when it was starved.

* `test_consumer::test_consumer_never_rations` — **`rationed == 1`**. Traced to step 196:
  `biosphere.allocation` demands **0.2381 mol C** from a `carbon_pool` holding **0.1852** —
  a **1.29× over-draw**, so the Euler backstop fires. Project policy is `rationed == 0`,
  and bucket 2 made this a hard error.
* `test_decade_stability::test_decade_run_is_deterministic` — `ArbitrationError`,
  `scale_f = 0.9506 < 1`. The **same over-draw**, promoted to a hard error because RK4
  requires positivity from the kinetics rather than the backstop.
* `test_consumer::test_per_compartment_ledger_balances_every_step` — residual −0.0528 in
  `biosphere.atmosphere` CARBON **at step 196**. Same event; the ledger sees the scaled
  draw.
* `test_water_biting::test_water_biting_cascade_vs_ample` — **the direction inverted**:
  the water-*stressed* run now reaches a HIGHER peak vegetative biomass (2.8151) than the
  ample baseline (2.7057).

  That last one looks like a bug and is not. Measured: the **ample** run drives
  `carbon_pool` *lower* (min **0.0730** vs **0.0792**, from the same 0.3570 start). It
  grows faster, exhausts the sealed CO₂ sooner, and is carbon-limited earlier. **The
  binding constraint has switched from water to carbon**, so relieving water stress no
  longer buys biomass. The test isolates `f_water` on the assumption that water is what
  binds — an assumption that was true of the starved plant and is now false.

### Root cause B — delayed phenology moved the productive phase past short test horizons

* `test_sealed_chamber::test_sealed_producer_recovers_o2_after_trough` — O₂ declines to
  ~1.3e-29 at the **last** step of that fixture's horizon and never recovers, so
  `max(o2[trough:])` is a one-element list. (In the full-length sealed season O₂ *does*
  recover: trough 0.026 at step 137, ending at 2.236 — so this is a **horizon** artifact,
  not a broken loop.)
* `test_chamber::test_sealed_assimilation_rises_then_declines` — assimilation no longer
  collapses below 1 % of peak (0.00229 vs a 0.000284 threshold). The healthier plant
  simply does not crash.

## Why this is a scope escalation, not a long list of test edits

Every one of these tests is **correct about the model it was written against**. What
changed is that several frozen **scenario knobs** — the sealed chambers' CO₂ sizing above
all — were implicitly calibrated to a plant that could never grow. The scenarios and the
broken plant were **co-adapted**, and fixing only one half exposes the seam.

That means the honest fix is not "regenerate the goldens". It is a **decision about the
scenarios**, and scenario knobs are themselves frozen items:

* **Re-size the sealed chambers** (raise the CO₂/air content) so a correctly-developing
  plant is not carbon-starved. Physically principled — a chamber sized for a 5× larger
  plant — but it moves frozen scenario data *and* makes the golden diff reflect two
  changes at once, which is what ruling A's staging exists to avoid.
* **Accept carbon limitation as the chambers' new physics** and re-pin the behavioral
  tests (including flipping the water-biting direction assertion, with the measurement
  above as its justification). Cheaper and changes no scenario data — but it freezes a
  chamber whose plant is CO₂-starved, and it does **not** resolve the `rationed == 1` /
  `ArbitrationError`, which are policy violations rather than re-pinnable expectations.
* **Split the increment** — land the science with the scenarios re-sized as part of the
  same unfreeze, treating "the chambers were sized for a broken plant" as a finding of
  this increment rather than a follow-on.

**This is a user decision and the ceremony stops here.** No golden has been regenerated;
`git diff tests/regression/golden/` is empty.

## The transferable lesson

**A model and its scenarios co-evolve, and a scenario can silently encode a bug's
consequences.** Nothing in the freeze contract records *why* the chamber holds the CO₂ it
holds; the number was chosen when it was sufficient, and it was sufficient only because
the plant could not grow. Correcting the plant made a previously-slack constraint bind —
and the tell was not the rationing (which is loud) but the **water-biting inversion**,
which reads as a physics bug and is actually a *constraint-order change*. The generalizable
check: when a fix makes a subsystem substantially bigger or faster, ask which constraints
were slack **because** of the bug, not merely which tests fail.

## The re-sizing investigation (the user chose "re-size in this increment")

The chamber CO₂/air was sized when the plant was starved. Deriving the fix rather than
tuning it, and the derivation itself is a finding.

### `chamber_air_mol` feeds TWO mole fractions, not one (advisor catch)

The first instinct was **carbon-only** scaling — raise `chamber_air_mol` + `chamber_co2_mol0`
together (holding `Ci0 = ci_ratio·co2_mol0/air_mol·1e6 = 250`), leave O₂ alone. That is
**wrong**, and the reason is that `air_mol` is the denominator of *both* intensive
variables the chamber exposes:

* `Ca = co2_mol / air_mol · 1e6` → Ci (photosynthesis), and
* `x_O2 = o2_mol / air_mol` → the `oxygen_limitation_factor` (f_O2) that throttles
  maintenance respiration (`chamber.py:36`, `carbon_budget.py:405`).

Carbon-only scaling (air 1000→2000, O₂ held at its value) would **halve x_O2** and
silently change respiration physics — freezing a chamber whose gas is no longer air. So
the correct move is **full uniform scaling**: air, CO₂, *and each scenario's own O₂* by
the same factor, which holds **both** mole fractions invariant. A genuinely bigger
chamber with the same gas.

### The scale-invariance, and why ×2 (not "derived from demand")

Above a ~1.5× exhaustion threshold the carbon pool draws down to a **scale-invariant
~0.20 of its start** — the FvCB Ci-shutoff pins Ci to a fixed fraction toward Γ*
regardless of absolute size, which is the design's own "Ci falls ~5×" intent. So the
number is **not** derived from cumulative carbon demand (the draw-down is scale-free); ×2
is simply the **smallest round factor past the threshold, with ~2× peak-draw headroom**.
The genuinely derived result is the scale-invariance itself.

### Which scenarios actually break (measured; RK4 verified per advisor)

| scenario | Euler rationed x1 | RK4 x1 | RK4 x2 |
|---|---|---|---|
| SEALED | 0 | — (one-shot) | — |
| PERENNIAL | 0 | **OK** | OK |
| CONSUMER | **1** (step 196) | **ArbitrationError** (scale_f 0.9506) | **OK** |

**Only the CONSUMER chamber breaks** — under *both* integrators — and ×2 clears both. The
mechanism is physical: the herbivore grazes leaf, the plant regrows by drawing *more* from
the CO₂ pool, so carbon throughput (and peak draw) is higher than the herbivore-free
perennial. Perennial and SEALED do not ration at all.

### ⚠ THE OPEN SUB-DECISION — uniform ×2 weakens SEALED's O₂-depletion drama

This is the one piece I will **not** bake into a frozen golden without a ruling, because
it trades off two frozen scenarios' scientific purposes:

Uniform ×2 holds x_O2 invariant, but SEALED's **O₂ draws down only to 42 % of its start
(vs 1.3 % at x1)**. SEALED exists to stage O₂ depletion for the f_O2 limiter test, and
that depletion is driven by microbial respiration of a **fixed litter pile**
(`litter_carbon0 = 3.0`). Doubling the O₂ pool without doubling the depletion driver makes
the same absolute draw a shallower *fraction*. **SEALED's CO₂ pool, O₂ pool, and litter
pile were hand-tuned as a coupled set for the starved plant** — the co-adaptation finding,
one level deeper — and they do not cleanly rescale by a single factor.

Three ways to resolve it, none free:

* **(i) Uniform ×2 everywhere, re-tune SEALED's O₂/litter** so the depletion drama
  survives with a healthy plant. Most physically coherent ("one bigger chamber"), but
  re-tunes a second coupled scenario and moves the most goldens.
* **(ii) Consumer-specific enlargement** — give only the CONSUMER chamber the bigger CO₂
  reserve, leave SEALED and PERENNIAL at their frozen sizing. Most surgical (SEALED's O₂
  drama untouched, fewest goldens moved), and physically defensible (the herbivore raises
  carbon demand). Cost: the consumer chamber is no longer literally "the perennial chamber
  + one herbivore" — a narrative the scenarios currently lean on.
* **(iii) Uniform ×2 and accept the shallower SEALED depletion**, re-pinning the f_O2
  test if it still exercises the limiter. Cheapest; risks the f_O2 test becoming vacuous.

**SEALED does not ration and needs no carbon fix on its own** — it is dragged in *only*
because it shares the default CO₂/air. That fact is what makes (ii) coherent.

## Still to do for increment 1

Ceremony **blocked** pending the sub-decision above (uniform+retune / consumer-specific /
uniform+accept), then: regenerate the affected goldens, re-triage the full suite at the
chosen sizing (advisor: re-run `-m "not slow"` with no `-x` *after* re-sizing, not off the
pre-resizing triage), regenerate the manifest, record provenance, run all gates; compile
and anchor the Rust mirror. Remaining: full-suite degeneracy review (advisor: a 57-day arrest
can dominate a short-horizon chamber run, and perennial/long-horizon now re-vernalize
every cycle — conservation will not catch a plant that simply never develops), the Rust
hand-mirror, then goldens → manifest → provenance → gates.
