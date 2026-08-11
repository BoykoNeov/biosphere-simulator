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

## Outcome

*(Pending the user's scope decision — see "The coupling" above. Nothing built yet; the
whole of the above is measurement on the frozen tree, `git diff src/` empty.)*
