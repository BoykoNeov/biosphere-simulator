# Re-deriving the soil water store from geometry (the soil-layers build's own finding)

**Charge.** `post-roadmap-soil-layers.md` closed by naming three successors and flagging
one of them as the real finding of that build, with the verdict left to the user:

> **Re-partitioning `soil_water0` by geometry** ([F] Eqns 14.26–14.28: `ATSW = DEPORT ·
> EXTR · MAI`). ⚠ **This is the real finding of the build, recorded rather than acted on:**
> our root-zone bucket is dimensionally not a soil profile. 1000 kg over 1 m² is 1000 mm of
> extractable water, which at `EXTR = 0.13` would need a 7.7 m soil column. Deriving the
> root-zone store from depth instead of declaring it would collapse it to ~19.5 kg at
> sowing — below `sw_critical = 60` — and make **every frozen scenario water-stressed**.
> That is a re-basing of the whole water regime, not this mechanism. **Named successor,
> and the verdict is the user's.**

The user took it on 2026-08-12, quoting that paragraph.

## Status: DIAGNOSED — the recorded price was wrong in both directions, measured

The successor was recorded as "every frozen scenario water-stressed". It was measured
before any design work, on the frozen tree, by substituting the geometric store and
running. **The prediction is wrong on both halves of the roster, in opposite directions.**

| scenario | as shipped | geometry (`soil_water0 = 19.5`) | verdict |
|---|---|---|---|
| `DEFAULT` (open, irrigated) | `f_water ≡ 1`, leaf C peak 8.8398, storage C 22.4135 | 19 stressed days of 306 (one at exactly 0), peak **7.0905**, storage **21.8961** | survives: −20 % canopy, **−2 % yield** |
| `N_LIMITED` (open) | peak 0.1165, storage 0.0854 | peak **0.1174**, storage **0.0859** | survives; the N gate dominates, water is noise |
| `DROUGHT` (open, irrigation cut in a window) | `f_water ≡ 1`, peak 8.8398 | 33 stressed days, peak **6.4110**, storage **21.4607** | survives, and the scenario finally bites |
| `SEALED_CHAMBER` | peak 0.8015, storage 0.3101, depth 1.3006 | **peak 0.0500, storage 0.0000, depth 0.1500, `f_water = 0` on all 306 days** | **DEAD, and locked** |
| `WATER_BITING` (sealed) | `f_water` 0.50–1.0, peak 0.8299 | **identical death lock** | **DEAD, and locked** |

So the honest price is not "stressed": it is **fatal on every sealed chamber and cheap on
every open one**.

### Why the sealed chambers lock, and why nothing releases them

`soil.build_soil` drops `Irrigation` and the `water_source` boundary entirely when
`sealed=True` (that is the P3.3 genuine-closure decision). So a sealed chamber's
`soil_water` has **no external inflow at all** — its only inflow is `Recycling`, i.e.
water the plant itself transpired and the condenser recovered.

That makes the wilting point an **absorbing state**:

```
soil_water ≤ sw_wilting  ⇒  f_water = 0  ⇒  transpiration = 0
                         ⇒  no vapour ⇒ no condensate ⇒ no recycling
                         ⇒  soil_water unchanged  ⇒  f_water = 0  (for ever)
```

and `f_water` also multiplies `root_depth.extension_rate`, so rooted depth freezes at
`rooted_depth0` and `RootZoneCapture` moves nothing — the 175.5 kg sitting below the root
zone is permanently unreachable. Measured: `soil_water` holds at **exactly** 19.5000 for
all 306 days, depth at exactly 0.1500.

The geometric store misses the escape by **0.5 kg**: `rooted_depth0 · EXTR · ρ · area =
0.15 × 0.13 × 1000 × 1 = 19.5`, against `sw_wilting = 20.0`.

⚠ This is the **same trap shape** the soil-layers build already recorded as its finding
(1) — two cited mechanisms composing into a stop the crop cannot escape — entered through
a different door. There it was `WSTORG = 0 ⇒ GRTD = 0` against an uncited sowing depth of
0; here it is the wilting clamp against a geometric store. The lesson repeats: **a gate
whose input is downstream of its own output is an absorbing state, and only running it
shows that.**

The open-field scenarios escape only because irrigation (2 mm day⁻¹, i.e. 2 kg day⁻¹ into
a 19.5 kg bucket) refills the store from a boundary regardless of `f_water`. That is why
the roster splits so sharply.

## The coupling: this is one mechanism with the FTSW successor, not two

The soil-layers plan filed the geometry re-basing and the `FTSW` stress conversion as two
independent successors. **They are not independent, and the measurement above is the
proof.**

The reason is dimensional. Our stress driver is an **absolute-kg** ramp between
`sw_wilting = 20` and `sw_critical = 60` kg. Those two numbers are only meaningful against
a store of a particular size; they were chosen against a 1000 kg store. [F]'s driver is a
**fraction**:

```
TTSW = DEPORT · EXTR                    [F] Eqn 14.6
FTSW = ATSW / TTSW                      [F] Eqn 14.7
WSFG = 1 if FTSW ≥ WSSG else FTSW/WSSG  [F] Eqn 15.3 (WSFL likewise, on WSSL)
```

and the initialization is `ATSW = DEPORT · EXTR · MAI` (14.26), so

```
FTSW₀ = (DEPORT · EXTR · MAI) / (DEPORT · EXTR) = MAI
```

**— the sowing stress is `MAI`, independent of depth.** At the drained upper limit
(`MAI = 1`, the potential-production condition every frozen scenario is already built on)
the crop starts unstressed no matter how shallow the root zone, and stress appears only
from actual drying. The 19.5 kg store is not "nearly wilting"; it is **full**, and it only
reads as wilting because it is being compared against thresholds calibrated for a store
51× larger.

### Measured: `FTSW` does not merely soften the lock, it removes it — and preserves the science

The loop shape *survives* the conversion on paper (`WSFG → 0 ⇒ no transpiration ⇒ no
recycling ⇒ `FTSW` unchanged`), so whether the trap moves to an unreachable corner or
stays reachable is a question only a run answers. It was run: the stress factor patched to
[F]'s `WSFG = min(1, FTSW/WSSG)` with `TTSW = rooted_depth · EXTR · ρ · area` read from the
**step-entry** aux (exact under Euler `dt = 1`, since every flow evaluates against the
step-entry snapshot), `MAI = 1`, against the shipped tree.

**Every frozen end-state stock is reproduced exactly except the two water stocks:**

| scenario | what moves, shipped → geometry + `FTSW` |
|---|---|
| `DEFAULT` | `soil_water` 1172.2936 → 191.7936, `subsoil_water` 45.4194 → 25.9194. **Nothing else, at all.** |
| `SEALED_CHAMBER` | `soil_water` 1133.6781 → 153.1781, `subsoil_water` 45.4194 → 25.9194. **Nothing else.** |
| `N_LIMITED` | the same two, the same amounts. **Nothing else.** |
| `DROUGHT` | `soil_water` 92.7130 → 42.2130. **Nothing else.** |
| `WATER_BITING` | 18 stocks move; leaf C 0.2172 → 0.1730, storage C 0.2610 → 0.3164, `FTSW` dips to 0.1722 for 45 days |

Both water deltas are exactly the declaration that was removed: `1172.2936 − 191.7936 =
980.5 = 1000 − 19.5`, and the subsoil moves by exactly the 19.5 kg the double-count had
added. **Not one carbon, nitrogen or oxygen amount moves on four of the five scenarios.**

That is not luck and it is not a weak test. Every frozen scenario is a *potential
production* scenario: at `MAI = 1` the crop starts at the drained upper limit, `FTSW` never
falls below 0.79 on the sealed chamber or 0.9957 on the open field, both far above
`WSSG = 0.30`, so `WSFG ≡ 1` — exactly as `f_water ≡ 1` today. The two forms agree wherever
water does not limit, which is everywhere the freeze looks. `WATER_BITING` is the one
scenario built to make water limit, and it is the one whose science moves.

⚠ **`FTSW > 1` is reachable and was measured:** 2.21 on `DEFAULT`, **11.51** on `DROUGHT`.
Both irrigation and recycling pour into a now-bounded bucket with nothing to stop them, so
`soil_water` stops meaning `ATSW` — it can hold 11× the water the root zone can transpire.
It is harmless to the carbon today only because `WSFG` clamps at 1. This is the capacity
constraint below, and the measurement says it is required for the water numbers to mean
anything, not merely for tidiness.

So the three branches are:

1. **Geometry alone.** Arithmetically runnable, and it kills every sealed chamber. Not
   shippable.
2. **Geometry alone, with `rooted_depth0` raised to 0.40 m** (the *top* of the same cited
   [F] range we already quote the 0.15 from) → `ATSW₀ = 52 kg`, inside the band,
   `f_water₀ = 0.80`. Measured: open field survives at peak 8.7537 / storage 22.4037,
   near-untouched. **This is the trap branch:** it runs, it looks calibrated, and the
   stress number it produces is a depth-derived store compared against depth-independent
   thresholds — i.e. meaningless. Recorded so nobody reaches for it as the cheap option.
3. **Geometry + `FTSW`.** The coherent one, and measured above to be **science-preserving
   on the whole frozen roster bar the one scenario built to bite**. `MAI` replaces
   `soil_water0`; the ramp thresholds become `WSSL`/`WSSG` fractions from [F] Table 15.1.
   Still needs a capacity rule — see below.

## What branch 3 requires, enumerated with its equation

| piece | source | size |
|---|---|---|
| `ATSW₀ = DEPORT · EXTR · MAI`; `IPATSW = SOLDEP · EXTR · MAI`; `WSTORG₀ = IPATSW − ATSW₀` | [F] 14.26–14.28 | scenario fields + builders; **no new flow**; moves every golden |
| `TTSW = DEPORT · EXTR`, `FTSW = ATSW/TTSW`, `WSFG`/`WSFL` | [F] 14.6, 14.7, 15.3 | replaces `water_stress_factor`'s signature at **three** call sites; moves every golden |
| a capacity constraint, or `FTSW > 1` is reachable | [F] 14.8 *or* 14.11 | see below |

**The capacity constraint is the open scope question.** Once the root zone is bounded at
`TTSW`, an unconstrained inflow can push `ATSW` past it. There are three inflows to
`soil_water` and they are not alike:

* `RootZoneCapture` is **self-consistent** — it raises `ATSW` and `TTSW` by the same
  `GRTD · EXTR`, so it cannot lift `FTSW` above 1.
* `Irrigation` (open field only) is unconstrained. [F]'s own answer is **Eqn 14.8**,
  `IRGW = TTSW − ATSW` — irrigate to the drained upper limit — which replaces our fixed
  2 mm day⁻¹ with a cited rule and needs **no new stock or flow**.
* `Recycling` (sealed only) is unconstrained and has no [F] counterpart at all ([F] is a
  field model; there is no condenser). This is where `FTSW > 1` will actually appear, and
  it is **ours to decide**.

The heavier alternative is drainage, [F] Eqn 14.11: `DRAIN = (ATSW − TTSW) · DRAINF` when
`ATSW > TTSW`, with `DRAINF` from Table 14.2 (0.1–0.6 by soil and depth). That is a new
flow, a new boundary sink and a new param — `flow_set` 21 → 22 — and it **swallows a
successor that is currently filed separately**. Check whether 14.8 plus a sealed-side rule
suffices before reaching for it.

## Three things to check before building, not after

1. **`subsoil_water0` currently double-counts the root zone.** The shipped default is
   `soil_depth · EXTR · ρ · area = 195 kg`, which is [F]'s **`IPATSW`** (Eqn 14.27), the
   *whole* profile. [F] Eqn 14.28 is `WSTORG = IPATSW − ATSW`, i.e. 195 − 19.5 = **175.5**.
   `tests/test_soil_layers.py:89` pins the identity `subsoil_water0 == captured_water(
   soil_depth, …)` — so the pin holds a formula [F] does not have. It is defensible only
   because the current `soil_water0` is not geometric at all, so there is no `ATSW` to
   subtract; the re-basing removes that excuse and must fix both.
2. **`f_water` has three consumers, and [F] does not use one factor for all three:**
   assimilation (`carbon_budget.py:202`), root extension (`root_depth.py:178`), and actual
   transpiration. [F] Box 14.1 carries `WSFL` (leaf area), `WSFG` (growth), and Ch. 15's
   NTR (transpiration) as *different* curves on the same `FTSW`. ⚠ Our root-extension
   factor is **[E]'s, not [F]'s, and is explicitly cited** ([E] p. 137: *"The effect of
   water stress on the rate of increase in rooted depth is supposed to equal that of water
   uptake"*) — [F]'s own `GRTD` has no continuous water factor at all, only the discrete
   gates `CTU < tuBRG`, `CTU > tuTRG`, `DDMP = 0`, `DEPORT ≥ SOLDEP`, `DEPORT ≥ MEED`,
   `WSTORG = 0` (Box 14.1). So the two sources must be kept apart deliberately, and
   rescaling `f_water` silently rescales [E]'s citation.
3. **[F] Table 15.1 is column-scrambled by `pdftotext`** (crop names stack in one column,
   values in another) — so it was read off a **page render** (PDF page 210 = printed
   p. 195, `pdftoppm -r 160`), the way `test_chamber_scale.py` and
   `test_soil_fractionation.py` already do for exactly this reason. **RESOLVED
   2026-08-12** — the render gives, for **wheat**:

   | | `WSSL` (leaf area) | `WSSG` (growth/transpiration) | `WSSD` (phenology) |
   |---|---|---|---|
   | Wheat | **0.40** | **0.30** | **0.40** |

   The scrambled extraction happened to put the right numbers on the wheat row, which is
   exactly the kind of coincidence that would have made a wrong pin look verified. ⚠ The
   literal `0..25` in the text output is **not** an extraction artefact — it is a typo in
   the **printed table** (soybean's `WSSG`), visible in the render. Do not "fix" it
   silently if soybean is ever added.

   The same render settles the consumer question in check 2 above: p. 195 states the
   deficit factor is applied to **four** processes — `WSFG` growth/transpiration, `WSFL`
   leaf area, `WSFD` phenology, `WSFN` nitrogen (Ch. 17) — with *different* thresholds
   per process. Our tree applies **one** factor to three consumers, so the mapping is a
   decision to make deliberately, not a transcription.

## Sources

* **[F]** Soltani, A. & Sinclair, T.R., *Modeling Physiology of Crop Development, Growth
  and Yield*, CABI. **Ch. 13** (`EXTR` = 0.13 mm mm⁻¹), **Ch. 14** (Eqns 14.2–14.13 and
  14.25–14.28, Table 14.2, Box 14.1's VBA listing — the authoritative order of
  operations), **Ch. 15** (Eqn 15.3, Table 15.1, Fig. 15.1 — the `FTSW` response).
* **[E]** Penning de Vries, F.W.T. et al. (1989), *Simulation of Ecophysiological
  Processes of Growth in Several Annual Crops*, Simulation Monographs 29. p. 136–137,
  Listing 7 L34 — the root-extension water factor, which is **not** [F]'s.

---

# The build — scope chosen 2026-08-12: **branch 3 + drainage**

The user took the widest branch and supplied the design constraint that settles it:

> *"yes, the station can have a water reservoir, of course. the drainage can be turned on
> or off with at least a valve."*

**[F] agrees, and it is cited — so the reservoir needs no new stock.** The plan above
priced drainage as "a new flow, a new boundary sink and a new param". The boundary sink is
wrong: [F] does **not** drain to a sink.

> "For the root layer, not all the drained water (DRAIN) below the root layer may be
> considered a water loss. All or part of the drained water to deeper soil may be exploited
> later by the crop due to root growth." — [F] Ch. 14, p. 176
>
> `WSTORGᵢ = WSTORGᵢ₋₁ + DRAIN − EWAT` — **Eqn 14.12**
>
> "Final total drainage will be the amount of WSTORG at the end of simulation (growing
> season). That is, unused, drained water below the root zone."

Drainage's destination is `WSTORG` — **our existing `subsoil_water` POOL.** So the flow is
`soil_water → subsoil_water`, the exact inverse of `RootZoneCapture`, entirely in-system,
crossing no boundary: conservation is structural rather than asserted, and the user's
reservoir is the store the previous build already added. It also closes something the
soil-layers build recorded as a known one-way-ness: *"`DRAIN` is `WSTORG`'s only input in
[F], so omitting it makes the below-root store one-way within a season."* The store becomes
two-way, which is what [F] always had.

**The valve is `DRAINF`.** [F] Eqn 14.11 is `DRAIN = (ATSW − TTSW) · DRAINF` when
`ATSW > TTSW`, else 0 — so `drainage_factor = 0.0` **is** a shut valve, exactly, with no
branch of our own invention. No `bool`, no scenario flag: the parameter the source already
has does the job.

### ⚠ Drainage does NOT let irrigation alone — the first draft of this section was wrong

It said drainage "removes the need to touch irrigation at all", reasoning that excess
irrigation would simply drain. **Measured, that is false and expensive:** with the store
physically sized and `DRAINF = 0.3`, the flat 2 mm day⁻¹ leaves the reference open season
at `FTSW` **0.17** at its worst and costs **38 % of the yield** (storage C 22.4135 →
13.8450), with 204 kg of the season's irrigation ending up below the root zone.

The cause is not drainage misbehaving; it is that the flat schedule was **never sized
against demand**. It only looked adequate because the old bucket held 1000 kg — a
seven-metre soil column's worth of buffer. Peak measured demand is **5.7744 kg day⁻¹**
against a 2 kg day⁻¹ supply. Our own scenario comment calls this run *"water (PP,
non-limiting)"*; with a real bucket, that declaration is simply no longer true.

So irrigation becomes **demand-driven, capped by a declared system capacity** — which is
[F]'s two stated options composed, not an invention of ours:

> "One possibility is a fixed amount of water at each irrigation, **which may be defined by
> the capacity of the irrigation system**. Another possibility is to add sufficient water to
> return the root layer to a specific level, e.g. the drained upper limit. … `IRGW = TTSW −
> ATSW`" — [F] Ch. 14, **Eqn 14.8**

```
IRGW = min( irrigation_mm_day · area · dt ,  max(0, TTSW − ATSW) )
```

`irrigation_mm_day` stops being a *rate applied* and becomes a **capacity available**, so
the frozen default rises **2.0 → 8.0 mm day⁻¹** — chosen as a round number above the
measured 5.7744 peak (1.39× headroom), and **pinned to never bind on the reference season**,
which is what makes "potential production" a checkable declaration rather than a label.
Season water use actually *falls*, 610 → **582.44 kg**: demand-driven irrigation is more
frugal in total while needing a higher peak.

⚠ **Consequence to record now, because no golden will catch it later: once irrigation is
demand-driven, drainage is bit-identically INERT on the entire frozen roster.** `DRAINF`
0.3 and 0.0 give identical states everywhere. That is physically correct — you cannot drain
water you never over-applied — and it puts drainage in the same category as the root-depth
gate: a real mechanism that **no regression test can see**. Its pins must therefore be
unit-level and mutation-verified, and at least one scenario must actually over-water, or
the flow is decoration. Same for the Rust mirror, which was measured blind to five separate
mutations last build.

## `DRAINF` — the value, and a units defect in the source's own table

Table 14.2 (page render, PDF p. 192 = printed p. 177) gives `DRAINF` by soil texture and
depth: silty clay 0.3/0.2/0.1, silty loam 0.4/0.3/0.2, sandy loam 0.5/0.5/0.4, sand
0.6/0.5/0.4 — at `SOLDEP` **210 / 150 / 60**, captioned **mm**.

⚠ **That column cannot be a profile depth in millimetres, and [F] itself is the witness.**
(Stated that way deliberately: what is proven is that the *mm reading is impossible*, not
that the caption is a typo — DSSAT profiles are often specified per **horizon**, and
210/150/60 could be horizon thicknesses, which would make the caption right and only the
"profile depth" reading wrong. Either way the `DRAINF` pick below is unaffected.) As
millimetres
those soils are 6–21 cm deep, while the same chapter puts `DEPORT` at emergence at
**150–400 mm** and wheat's maximum extraction depth `MEED` at **1200 mm** (Table 14.1), and
Box 14.1 carries `If DEPORT >= SOLDEP Then GRTD = 0`. A 60 mm soil would stop a crop before
it emerged, and a 210 mm soil would make `MEED = 1200` unreachable on every crop in the
book. Read as **centimetres** the column is 2.1 m / 1.5 m / 0.6 m — ordinary profile depths,
all consistent with `MEED`. So the values are cm.

Our `soil_depth = 1.5 m` **is the middle row exactly**. Texture is fixed by a choice already
made: `EXTR = 0.13` is [F] Ch. 13's value "for many agricultural soils **except sandy
soils**", so we are a non-sandy agricultural soil → **silty loam at 1.5 m → `DRAINF = 0.3`**.

This is recorded rather than quietly worked around, because it is a **locus** defect of
exactly the kind `bucket3-scope-c-citation` warns about: the number is right, the unit
printed beside it is not, and only reading a *second* place in the same book shows it.

## The stress form: one factor, threshold 0.30, three consumers each justified separately

[F] p. 195 applies a deficit factor to four processes with different thresholds. Our tree
has three consumers of one `f_water`, and the mapping is **not** a free choice:

| our consumer | [F]'s process | factor | threshold |
|---|---|---|---|
| assimilation (`carbon_budget.limitation`) | "Growth, or specifically transpiration/dry matter accumulation" | `WSFG` | `WSSG` = **0.30** |
| transpiration (`Transpiration.evaluate`) | ⚠ **ours by analogy, NOT [F]'s** — see below | `WSFG` | 0.30 |
| root extension (`root_depth.extension_rate`) | not [F]'s at all — **[E] p. 137**: *"The effect of water stress on the rate of increase in rooted depth is supposed to equal that of water uptake"* | `WSFG`, **because [E] says "equal to"** | 0.30 |

So all three take `WSFG = min(1, FTSW/0.30)` ([F] Eqn 15.3), and the third does so *because*
[E]'s sentence pins it to the transpiration factor — the citation is preserved rather than
silently rescaled, which was check 2's hazard.

⚠ **The transpiration arm is ours, and the earlier draft of this table overstated its
warrant.** [F] p. 195's "Growth, or specifically transpiration/dry matter accumulation
(WSFG)" is *not* an endorsement of multiplying a potential rate by `WSFG`: in Box 14.1 [F]
computes `TR = DDMP · VPD / TEC`, i.e. transpiration is derived **from** dry-matter
production, which already carries `WSFG` — [F] never multiplies a Penman–Monteith potential
by a stress factor. Ours does (`transpiration.py`, `daily_kg = potential · f_water ·
ground_area`), and has since Phase 1. That predates this work and is not changed by it; what
changes is only the *shape* of the factor. Recorded as **ours by analogy** so a later reader
does not mistake it for a transcription.

**Also pinned, because the measurement harness papered over it:** the three consumers read
`soil_water` from two different places — `Transpiration` from `snapshot.stocks`,
`carbon_budget.limitation` through `env.get` — and must divide by a `TTSW` built from the
**same** step-entry rooted depth, or they silently disagree about `FTSW` inside one step.
The prototype used one shared depth cell and so could not have caught a disagreement. One
pin steps once and asserts all three agree.

**`WSFL` (leaf area, 0.40) and `WSFD` (phenology, 0.40) are NOT built**, and that is a real
gap rather than a simplification: we have no water-gated leaf-expansion or
phenology-slowdown term for them to attach to. Named successors, not oversights.

## What changes, piece by piece

| piece | source | shape |
|---|---|---|
| `soil_moisture_index` (`MAI`) ∈ [0,1] | [F] 14.25–14.28 | new scenario field, default **1.0** (drained upper limit — the potential-production condition the frozen roster is already built on) |
| `soil_water0` default 1000 → **19.5** | `ATSW = DEPORT · EXTR · ρ · A · MAI` (14.26) | value change + a pin holding the identity |
| `subsoil_water0` default 195 → **175.5** | `WSTORG = IPATSW − ATSW = (SOLDEP − DEPORT) · EXTR · ρ · A · MAI` (14.27/14.28) | fixes finding 5's double-count; **rewrites the pin at `test_soil_layers.py:89`**, which currently holds `IPATSW` |
| `TTSW`, `FTSW`, `WSFG` | 14.6, 14.7, 15.3 | `transpiration.water_stress_factor` changes signature; three call sites gain the rooted-depth aux + `EXTR`/`ground_area` |
| `sw_wilting` / `sw_critical` **retired** | — | they are the miscalibrated absolute band; replaced by `wssg` = 0.30 |
| `Drainage` flow | 14.11 + 14.12 | `soil_water → subsoil_water`, `(ATSW − TTSW)·DRAINF`, donor-clamped; `flow_set` **21 → 22** |
| `drainage_factor` (`DRAINF`) | Table 14.2, read as cm | new scenario field, default **0.3**; `0.0` is the shut valve |
| `Irrigation` becomes demand-driven | 14.8 + [F]'s "capacity of the irrigation system" | `IRGW = min(cap·A·dt, max(0, TTSW − ATSW))`; `irrigation_mm_day` **2.0 → 8.0**, reinterpreted rate → capacity, pinned non-binding |

## The whole design, measured end-to-end before a line of it was written

Geometry + `FTSW` + demand-driven irrigation + drainage, against the shipped tree, full
end-state stock-by-stock comparison:

| scenario | stocks that move | `rationed` |
|---|---|---|
| `DEFAULT` | `soil_water` 1172.2936 → 164.2348, `subsoil_water` 45.4194 → 25.9194, `water_source` −610.0 → −582.4413 | 0 |
| `SEALED_CHAMBER` | `soil_water` 1133.6781 → 153.1781, `subsoil_water` 45.4194 → 25.9194 | 0 |
| `N_LIMITED` | the same three as `DEFAULT`, the same amounts | 0 |
| `DROUGHT` | `soil_water` 92.7130 → 14.6542, `water_source` −610.0 → −582.4413 | 0 |

**Only water moves. Not one carbon, nitrogen or oxygen amount, on any of them.** A complete
dimensional re-basing of the water regime that leaves the frozen biology bit-identical.

**The arbitration backstop never fires** — `rationed == 0` across 15 probe runs including
`MAI` values low enough to kill the crop. That was the one blocking question (the old
absolute band shut transpiration off *hard* at 20 kg, which was the structural positivity
guarantee; `WSFG = FTSW/WSSG` only reaches zero at `FTSW = 0`, so the shutoff becomes
asymptotic and could in principle let a step overdraw with drainage competing for the same
donor). Measured, it does not. Checked, not reasoned.

## Hazards to measure, not assume (each has bitten this project before)

1. **`DROUGHT` and `WATER_BITING` declare `subsoil_water0 = 0.0` on purpose**, to freeze
   rooted depth via `WSTORG = 0 ⇒ GRTD = 0`. **MEASURED, and it fires:** with flat
   irrigation + drainage, `DROUGHT`'s rooted depth goes 0.1500 → **1.3072** — drainage
   fills the subsoil the scenario declared empty and un-freezes the depth. ⚠ It is
   **demand-driven irrigation, not drainage, that resolves this**: there is then no excess
   to drain, the subsoil stays at 0.0000, and depth stays at 0.1500 exactly as declared. So
   the two changes are load-bearing *for each other*, which is a third argument that this
   was never separable into small pieces.
2. **`WATER_BITING`'s bite is declared as `soil_water0 = 50` inside a (20, 60) kg band that
   is being deleted**, so it must be re-declared. **Target written before the number was
   picked**, from the scenario's own existing contract (`tests/test_water_biting.py`): a
   sustained bite (`min f < 0.5`, >30 days below it), **never fully wilted** (`0 < f ≤ 1`
   throughout), the crop alive, and the closed water loop still conserved. Swept `MAI` from
   0.10 down to 0.02 against that target: **`MAI = 0.05`** — `FTSW` 0.0500–0.3190, so
   `WSFG` bottoms at 0.167, leaf C peak 0.7621 and storage C 0.2452 against the shipped
   0.8299 / 0.2610, total loop water conserved 9.7500 → 9.750000 exactly, `rationed = 0`.
   ⚠ **And its dry-subsoil special case is retired, on a measurement.** That override
   existed because *"a default 195 kg subsoil would pump 2.34 kg/day into a 50 kg chamber
   and abolish the water stress"*. Under geometry the subsoil scales with the same `MAI`
   (8.775 kg, not 195), so it no longer abolishes anything — measured `FTSW` stays ≤ 0.319
   with the subsoil present. Keeping `subsoil_water0 = 0` would instead **kill** the crop at
   every `MAI` tried (leaf C 0.0500, storage C 0.0000): a sealed chamber holding 1.95 kg of
   total water grows nothing. So the special case goes, and with it the depth-freezing trap
   the soil-layers build had to work around.
3. **`DROUGHT` is left exactly as declared, and it still does not bite** — `FTSW` bottoms at
   0.7039, well above `WSSG = 0.30`, so storage C stays 22.4135. That is **not new**: the
   soil-layers build already recorded that the default profile *abolishes* the drought
   cascade rather than weakening it. The re-basing neither fixes nor worsens it. Making the
   scenario live up to its name is now a one-field change (a low `MAI`) but it would move a
   golden's *science* for a reason outside this charge — **named successor, not taken
   here.**
4. **The re-sow return and the 15-year closed cycle.** The soil-layers build's strongest
   result was that 15-year and 5-year runs land on the same `soil_water` — capture and
   re-sow return cancel exactly. Drainage adds a second downward path; re-measure that
   identity rather than trusting it.
5. **Predict the golden diff before regenerating** (`soil-layers-built`). The measured
   prediction is: on `DEFAULT`/`SEALED_CHAMBER`/`N_LIMITED`/`DROUGHT`, only `soil_water` and
   `subsoil_water` move under the geometry+`FTSW` half — drainage's additional effect is
   *not* yet measured and must be predicted separately.
6. **The Rust mirror is blind to its own transcription** unless the pins construct the
   conditions (measured last build: five separate mutations left the entire Rust suite
   green). Drainage's donor clamp and the `ATSW > TTSW` branch both need Rust pins that
   actually reach them.

## Sequencing

Python is the reference and this moves goldens and manifests, so **Python first, Rust
mirrors** (`rust-primary-pivot`: *"Moves a golden/manifest? Yes → Python"*). Blast radius is
measured and contained: `sw_wilting`/`sw_critical`/`soil_water0` appear **only** in the
biosphere domain and its tests plus three Rust biosphere files — no `src/station/`, no
`src/authoring/`, no authored `scenarios/*.yaml`, no `godot/`.

## The predicted golden diff — WRITTEN BEFORE REGENERATION

`soil-layers-built` earned this discipline the hard way, so the prediction goes in first
and the actual goes in the Outcome. Predicted over **all 25 goldens on disk**, not the
manifest's 7 (that roster error has bitten twice):

1. **`season_euler`, `n_limited`** (open, 1 yr): exactly three stocks move —
   `soil_water` 1172.2936 → **164.2348**, `subsoil_water` 45.4194 → **25.9194**,
   `water_source` −610.0 → **−582.4413**. Every carbon, nitrogen and oxygen amount
   **bit-identical**.
2. **`sealed_chamber`** (3 yr): two stocks — `soil_water` 1133.6781 → **153.1781**,
   `subsoil_water` 45.4194 → **25.9194**. No `water_source` (sealed has none). C/N/O
   bit-identical.
3. **`perennial_chamber`, `consumer_chamber`, `perennial_long_horizon`,
   `consumer_long_horizon`, `sealed_station`, `greenhouse`, `lighting`, `harvest`**:
   water stocks only, C/N/O bit-identical. ⚠ **Predicted, not measured** — the
   measurements above are single-season; the multi-year runs re-sow, and the re-sow
   return now shares the root-zone boundary with drainage. If a carbon amount moves on
   any of these, the re-sow/drainage interaction is the suspect and the prediction has
   failed, which is the point of writing it down.
4. **`water_biting`**: many stocks move — it is the one scenario re-declared
   (`soil_moisture_index = 0.05`). Leaf C peak 0.8299 → **0.7621**, storage C 0.2610 →
   **0.2452**.
5. **`drift_summary`, `sealed_energy_drift_summary`**: ⚠ **expected to MOVE**, unlike
   the soil-layers build where they stayed byte-identical. They summarise conservation
   drift, and the water totals themselves changed (1195 → 195 kg in the sealed chamber),
   so the round-off floor moves with them. A drift summary that came back *identical*
   would be the surprising result here.
6. Non-biosphere goldens (`power`, `thermal`, `eclss`, `crew`, `cabin_gas`,
   `power_self_discharge`, `demo_euler`, `demo_rk4`, `state_snapshot`,
   `water_recovery`): **untouched**. The blast radius was measured and contains no
   `src/station/`, `src/authoring/` or sibling-domain code.

## Outcome — BUILT 2026-08-12

### The prediction held, and the one place it failed was the one it named

Checked over **all 25 goldens on disk**. Predictions 1, 2, 5 and 6 held exactly; prediction
3 (the multi-year re-sowing runs) is the one that carried the caveat *"if a carbon amount
moves on any of these, the re-sow/drainage interaction is the suspect and the prediction
has failed"* — and it did fail, twice, for two different reasons, both of which the caveat
pointed straight at. Fixing both restored it:

* the **re-sow water return** was `min(captured_water(abandoned), soil_water)`, i.e. the
  abandoned column at the drained upper limit (149.58 kg). Against a 1150 kg store that was
  a rounding error; against a 19.5–169 kg store it exceeds the whole store, so its clamp
  fired on **every** re-sow and handed the entire root zone to the subsoil. Measured
  consequence: the 4-year sealed station made **no grain at all**. Re-derived as the
  abandoned **fraction** of the held water — preserves `FTSW` exactly across a re-sow, needs
  no clamp, and equals the old form at the drained upper limit, so it *generalises* the
  cited geometry rather than departing from it.
* `test_soil_fractionation.reset_variant` **hand-copied** that rule under a comment reading
  "Mirrors `season.annual_reset`". When the rule changed the copy did not, so every variant
  run was a control against a tree that no longer existed. The durable fix is
  `season.resow_water_return`: one function, two callers.

Final golden diff, after both: **only `soil_water`, `subsoil_water` and `water_source`
move, on every scenario except `water_biting`** — which was deliberately re-declared. Not
one carbon, nitrogen or oxygen amount, at any horizon.

⚠ **Prediction 5 was simply FALSE, and "false in our favour" is still a miss.** It said the
drift summaries would move, reasoning that they "summarise conservation drift, and the water
totals themselves changed". They came back **byte-identical**, and the reason is that the
premise was wrong about what those files contain: `drift_summary` holds `peak_leaf`,
`consumer_carbon` and `is_period_2`; `sealed_energy_drift_summary` holds `node_peak_temp_k`
and `is_stationary`. **There is no water in either.** So their byte-identity is not
independent confirmation of anything — it is the *same* statement as "no carbon moved",
restated. Recorded because the temptation is to bank a surprise-free result as extra
evidence, and here it was a duplicate of evidence already counted.

### Three defects the change exposed rather than caused

1. **All three station builders seeded their aux without `rooted_depth`** (both ports), so
   the station's crop silently started at depth 0. Invisible while the depth gate was inert;
   fatal once stress divides by `TTSW = depth · EXTR · ρ · A` (crop dead on day 1, root zone
   drained into the subsoil). `build_season` had seeded it correctly since the root-depth
   build — the station assembly simply never mirrored that.
2. **`harvest` injects a 1.3 m root system but inherited the 0.15 m zone's water**, i.e.
   `FTSW = 0.115` on day 0 for a grain-filling crop. Grain ended 79 % low before the fix.
   The depth and the water are two halves of one declaration and are now derived together.
3. **`DEEP_WATER` was not viable at zero irrigation** — and the reason is the sharpest
   single consequence of sizing the soil honestly: a crop that roots to 1.3 m over 1 m² can
   ever reach `1.3 × 0.13 × 1000 = 169 kg`, against a measured **582 kg** season demand. No
   soil depth fixes that. Its old `soil_water0 = 350` hid it — 350 kg in a 0.15 m root zone
   is 2.7 m of extractable water in a 15 cm layer. It now declares a *limited* supply
   (1 mm day⁻¹) and the mechanism it exists to show comes out **stronger**: 15× the canopy
   against the control, where the old declaration gave 2.5×.

   ⚠ **That 1 mm day⁻¹ was CHOSEN AFTER a sweep (0 → 4), and the asymmetry with the two
   acceptance-gate bounds refused on the same day is deliberate, not an inconsistency.** An
   acceptance *bound* asserts the tree is safe; picking it after seeing the measurement
   makes it assert only that the tree passes a bound the tree set, which is why both were
   dropped for rank-plus-exact-value. A *diagnostic scenario* has the opposite job — it
   exists to put a mechanism where it can be seen — so choosing the operating point that
   exposes it is the point, provided the sweep is recorded (it is: capacity 2 mm day⁻¹ and
   above makes the subsoil irrelevant, 0 makes the season unwinnable, and the whole range
   is in the plan). What would be illegitimate is quoting the 15× as a property of the
   *model* rather than of this scenario at this capacity.

⚠ **That control had to be rebuilt too, and silently.** It was `soil_extractable_water = 0`,
justified as removing the water transfer while leaving depth to grow. `EXTR` now appears in
**two** places — the transfer *and* `TTSW` — so zeroing it kills the crop outright rather
than isolating the transfer. The control is now "drop the `RootZoneCapture` flow from the
registry". A control that changes more than it claims is worse than none, and this one would
have kept passing while measuring something else.

### Two acceptance-gate claims died and were replaced by rank + exact values

Not by looser thresholds — `test_acceptance_gate.py` refuses fitted cuts in its own words,
and both of these would have been exactly that:

* **water's slack in `open_season` fell 189.24 → 9.31.** A margin of 189× was never a fact
  about safety; it was a fact about a bucket that could not exist. `soil_water` is now the
  tightest live gate in `open_season`, `greenhouse` and `lighting`.
* **`carbon_pool > 4 × runner-up` became 3.98×** on two chambers. Dropped in favour of the
  rank plus an **exactly pinned** runner-up — strictly stronger, since a threshold only
  catches changes bigger than its slack. The roster-wide runner-up has now changed identity
  twice (`o2_pool` → `power.battery` → `soil_water`); the retired "even the runner-up is a
  chamber property" corollary was **not** restored just because it drifted back to being
  true.

### The exit state

* **12 goldens changed**; both drift summaries byte-identical; `rationed == 0` and
  `events == ()` everywhere, so neither drainage nor the softer stress shutoff ever reaches
  the arbitration backstop.
* **Biosphere manifest**: `flow_set` 21 → 22 (`Drainage`), golden hashes. Station manifest
  refreshed. `param_files` **unchanged** — `wssg`, `MAI` and `DRAINF` are scenario/soil data,
  like `EXTR` and `ground_area` before them.
* **Both ports**, and the port carries the rule not the rationale: the same `WSFG` form, the
  same demand-driven irrigation, the same drainage destination, the same fractional re-sow
  return (via a shared `resow_water_return` on each side, because the hand-copy hazard is
  what bit here).

  ⚠ **And that fix re-creates the same hazard across the port boundary, which is accepted
  rather than solved.** `resow_water_return` now exists twice — once in Python, once in
  Rust — which is exactly the two-copies-of-one-rule shape that caused finding 9. It is
  unavoidable in a port and it is not fully covered: cross-port parity would catch a
  divergence that moves a golden, but **not** one that only shows on a scenario neither
  port runs. Each side pins the rule behaviourally (redistribution + `FTSW` preserved), and
  that is the mitigation, not a proof. Named here so the residual is visible rather than
  looking closed.
* **Rust pins for `Drainage`, mutation-verified**, because the flow is bit-identically inert
  on every scenario and `cargo test` green would otherwise prove nothing about it. Four
  mutations — drain a share of the whole store instead of the excess, drop `DRAINF`, drop
  `ground_area` from the capacity, drop the donor clamp — each turn a pin red. Its pins
  construct a non-unit plot and an over-filled zone, neither of which any scenario provides.
* **A ROSTER-WIDE geometry pin, added because the existing ones were correct-by-
  inheritance.** The two identities held on every scenario, but only three had a pin —
  `DEFAULT`, `water_biting`, `drought`. Everything else inherited the defaults, and
  *correct by inheritance is not covered, it is untested*: the moment a scenario overrode
  `rooted_depth0` or `soil_depth` without moving its stores, nothing went red. That is not
  hypothetical — it is precisely what `harvest` did. The new pin enumerates every
  `SeasonScenario` from the **modules** rather than a hand-list
  (`coverage-roster-is-not-the-manifest`), covers the station's four as well as the
  biosphere's, and names `DROUGHT` as the one stratified exemption. Mutation-verified:
  overriding `rooted_depth0` on `potato`, or `soil_depth` on `day_neutral`, each turns it
  red.
* **2290+ Python tests, the full Rust suite, and all 101 cross-port parity checks green.**

### What this does and does not discharge

It discharges the geometry re-basing, the `FTSW` conversion and drainage — the three
successors the soil-layers build named, two of which turned out to be one mechanism. It does
**not** discharge, and each is a named successor:

* **`WSSL` (leaf-area expansion, 0.40) and `WSSD` (phenology, 0.40)** — [F] applies the
  deficit factor to four processes with different thresholds; we carry one, because we have
  no water-gated leaf-expansion or drought-accelerated development term for the others to
  attach to. That is a real gap, not a simplification.
* **Runoff and soil evaporation** — the remainder of [F] Ch. 14's removals.
* **Making `DROUGHT` actually bite.** It still does not (`FTSW` bottoms at 0.7039 against
  `wssg = 0.30`), which is not new — the soil-layers build already recorded that the
  reachable subsoil *abolishes* that cascade. It is now a one-field change, but it would
  move a golden's science for a reason outside this charge.
