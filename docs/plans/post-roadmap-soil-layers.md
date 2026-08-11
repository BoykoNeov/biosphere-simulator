# Soil layers — the water side of root depth (the successor the root build named)

**Charge.** `post-roadmap-root-functional-coupling.md` closed with one live successor:

> What is now genuinely available, and was not before, is the **water** coupling: `TTSW =
> DEPORT · EXTR` attaches to the accumulator this build added. It remains unbuilt and needs
> the single soil-water pool split into layers — a structural change, priced above, not
> attempted.

The user took it, with a framing that turns out to be the design's own default: *"in a
station, the soil will be artificially watered so that there will be no much deeper soil
layers that the plants have to reach — water will be available in all layers. But we want
our biosphere simulator to be able to simulate a whole lot of scenarios and conditions."*
So the well-watered profile is the **default**, and the dry-topped profile is the
**scenario** — which is exactly how the frozen roster and the new diagnostic split below.

## The resolution is CITED, not a compromise

The price doc assumed "layers" meant an N-layer discretization. [F] says otherwise, in the
sentence that opens its own soil-water chapter:

> "for models attempting to simulate crop growth and yield as is the objective of this
> book, a **two-layered soil or even a one-layer soil seems satisfactory** (Robertson and
> Fukai, 1994)." — [F] Ch. 14

and then specifies the two stores it means: **the root zone** (which grows) and **the water
stored below it** (`WSTORG`). So "we did not build N layers" is a design choice with a
citation behind it, not a shortcut. The mechanism the charge asks for — *water that is
physically present but currently unreachable* — is `WSTORG`, and the reachability event is
one equation:

```
EWAT = min(GRTD · EXTR, WSTORG)          [F] Eqn 14.10
```

`EWAT` ("the amount of water becoming available to the crop due to root growth") is a
transfer from the below-root store into the root zone, driven by the depth increment the
**existing** `rooted_depth` accumulator already computes. Nothing about the depth law
changes; it acquires a second consumer.

## What is built

| piece | source | note |
|---|---|---|
| `subsoil_water` POOL (`WSTORG`) | [F] Eqn 14.12, Fig. 14.2 | in-system soil water below the rooted depth |
| `RootZoneCapture` flow (`EWAT`) | [F] Eqn 14.10 | `subsoil_water → soil_water`, rate `GRTD·EXTR·area`, clamped to the donor |
| `WSTORG == 0 ⇒ GRTD = 0` | [F] Box 14.1 | roots do not extend into dry soil |
| `soil_depth` (`SOLDEP`) caps rooted depth | [F] Box 14.1; [E] Listing 7 L33 | **discharges a deferral `root_depth.yaml` already names** |
| `soil_extractable_water` (`EXTR`) | [F] Ch. 13 | 0.13 mm mm⁻¹, cited first-hand |
| `rooted_depth0` at sowing | [F] Ch. 14 | replaces our **uncited 0.0** with a cited 150–400 mm range |
| re-sow returns the abandoned zone's water | ours — [F] is single-season | symmetric with capture; see below |

## What is NOT built, and why — each is a named successor, not an oversight

* **The `FTSW` stress conversion.** [F]'s stress driver is `FTSW = ATSW / TTSW` with
  `TTSW = DEPORT · EXTR`; ours is an absolute-kg ramp between `sw_wilting` and
  `sw_critical`. Converting is a **second, orthogonal form change**: deeper roots raise
  `FTSW`'s numerator *and* denominator together, so it is not the reachability mechanism
  this charge is about, and it would move `water_biting`. **Named successor.**
* **Drainage (14.11), runoff (14.13), soil evaporation.** `DRAIN` is `WSTORG`'s only
  *input* in [F], so omitting it makes the below-root store one-way within a season. That
  is honest for a station bed and it is what the re-sow return exists to keep from
  ratcheting. Building drainage would bite the frozen reference hard — `soil_water0 = 1000`
  kg over 1 m² is ~7.7× the transpirable water a 1.5 m profile can hold, so a drainage rule
  would drain most of it on day 1 and change every frozen trajectory.
* **Re-partitioning `soil_water0` by geometry** ([F] Eqns 14.26–14.28: `ATSW = DEPORT ·
  EXTR · MAI`). ⚠ **This is the real finding of the build, recorded rather than acted on:**
  our root-zone bucket is dimensionally not a soil profile. 1000 kg over 1 m² is 1000 mm of
  extractable water, which at `EXTR = 0.13` would need a 7.7 m soil column. Deriving the
  root-zone store from depth instead of declaring it would collapse it to ~19.5 kg at
  sowing — below `sw_critical = 60` — and make **every frozen scenario water-stressed**.
  That is a re-basing of the whole water regime, not this mechanism. **Named successor,
  and the verdict is the user's.**

## The three decisions

### 1. The frozen scenarios get a WET subsoil

Forced by [F]'s own `If WSTORG = 0 Then GRTD = 0`: a zero subsoil freezes the frozen crop's
roots, which is botanically wrong *and* moves a golden — paying a golden move to get a worse
answer. A moist profile is also the user's own station framing. The default is
`subsoil_water0 = 195 kg`, which is not a free number: it is `soil_depth 1.5 m × EXTR 0.13 ×
1000 kg m⁻³ × 1 m²`, i.e. **the profile at the drained upper limit** — the potential-
production condition every frozen scenario is already built on. A pin holds the identity.

### 2. `water_biting` keeps its bite — `subsoil_water0 = 0.0`

`water_biting` is the one frozen-adjacent scenario whose soil water sits *inside* the stress
band, so it is the one place capture would feed back (capture → `f_water` → depth →
capture). A default 195 kg subsoil would pump 2.34 kg/day into a 50 kg chamber and **abolish
the water stress the scenario exists to exercise**. So it declares a chamber that is lean
*throughout*: no extractable water below the bed either.

⚠ That choice is only benign **because of decision 3.** Under our old uncited
`rooted_depth = 0` at sowing, a dry subsoil would freeze depth at 0, making `FROOT1 = 0` and
nitrogen uptake identically zero — turning a deliberately water-only scenario into an
N-limited one. A cited sowing depth removes the trap.

### 3. Sowing rooting depth becomes cited data

[F] Ch. 14: *"The value of DEPORT at crop emergence must be provided to the model. It is
normally between 150 to 400 mm depending on crop species and soil conditions."* We took
`0.0`, which no source supports and which only ever worked because the depth gate was
inert. `rooted_depth0 = 0.15 m` is the bottom of that cited range — the cautious end, since
a shallower start is a *tighter* gate. It applies at sowing **and at re-sow** (a re-sown
crop starts with the root system a sown crop has).

### The re-sow return — ours, and stated as ours

[F] is single-season and silent. Our chambers re-sow for up to 15 years, and without a
return leg every re-sow ratchets more of the profile permanently into the root zone. Water
in soil does not move when a plant dies: if the root zone shrinks from `d` to
`rooted_depth0`, the abandoned column's transpirable water is again below the root zone.

```
return = min(soil_water, (depth − rooted_depth0) · EXTR · 1000 · ground_area)
```

— exactly the inverse of the cumulative unclamped capture, so a full season is a closed
cycle. It lives in `annual_reset`, which is already inside `conservation.assert_conserved`,
so the transfer is gated, not silent.

## The predicted golden diff (written BEFORE regeneration)

* `soil_water` **up** by the season's cumulative capture; `subsoil_water` **down** by the
  same amount; total water conserved.
* `f_water` exactly `1.0` on every frozen scenario throughout — it is a clamp branch
  returning the literal `1.0`, not a ramp evaluation, so no ULP leaks in.
* Therefore **every carbon / nitrogen / O₂ stock bit-identical**, at every horizon.
* `rooted_depth` moves (the sowing depth), and the cap is reached ~8 days earlier.
* `water_biting`: only `rooted_depth` moves; its water story is untouched.

Predicted-vs-actual is recorded in "Outcome" below. A prediction that survives contact is
the strongest check available here, since no golden can catch the mechanism itself.

## Sources

* **[F]** Soltani, A. & Sinclair, T.R., *Modeling Physiology of Crop Development, Growth and
  Yield*, CABI. **Ch. 13** (DUL/LL/EXTR; the 0.13 mm mm⁻¹ value), **Ch. 14** (the two-store
  soil water balance, Eqns 14.2–14.12 and 14.25–14.28, Table 14.1, Box 14.1's VBA listing —
  the authoritative order of operations).
* **[E]** Penning de Vries, F.W.T. et al. (1989), *Simulation of Ecophysiological Processes
  of Growth in Several Annual Crops*, Simulation Monographs 29. Listing 7 L33 — the
  shallowest-of-soil-and-crop rooting cap this build discharges.

---

## Outcome — BUILT 2026-08-11

### The prediction held, exactly

Written before regeneration, checked after, over **all 25 goldens on disk** (not the
manifest's 7 — that roster error bit the previous build twice):

| golden | stocks added | stocks moved | aux moved |
|---|---|---|---|
| `season_euler` (open, 1 yr) | `subsoil_water` | `soil_water` | `rooted_depth` |
| `sealed_chamber` (3 yr) | `subsoil_water` | `soil_water` | `rooted_depth` |
| `perennial_chamber` (5 yr) | `subsoil_water` | `soil_water` | `rooted_depth` |
| `consumer_chamber` (5 yr) | `subsoil_water` | `soil_water` | `rooted_depth` |
| `perennial_long_horizon` (15 yr) | `subsoil_water` | `soil_water` | `rooted_depth` |
| `consumer_long_horizon` (15 yr) | `subsoil_water` | `soil_water` | `rooted_depth` |
| `n_limited`, `sealed_station` | `subsoil_water` | `soil_water` | `rooted_depth` |
| `greenhouse`, `lighting` | `subsoil_water` | `soil_water` | — |
| `harvest` | `subsoil_water` | — | — |
| `water_biting` | `subsoil_water` | — | `rooted_depth` |
| `drift_summary`, `sealed_energy_drift_summary` | **byte-identical** | | |

**Not one carbon, nitrogen or oxygen amount moved, on any scenario, at any horizon.** The
`soil_water` shift is `149.5806 kg` on every full-season run, against the geometric
prediction `(1.30062 − 0.15) × 0.13 × 1000 = 149.58` — the capture, to the digit.

Two results worth more than the table:

* **The 15-year runs land on the same `soil_water` as the 5-year runs** (1133.678079 kg).
  A season is an exactly closed cycle: capture and re-sow return cancel. That is the
  ratchet argument verified rather than reasoned.
* **Sealed-chamber water conserves over four stocks**: `soil_water + subsoil_water +
  water_vapor + condensate = 1195.0000000000173` after 15 years (= 1000 + 195), a drift of
  1.7e-11 in line with the pre-existing round-off.

`harvest` moves nothing because its crop starts past anthesis at the 1.3 m cap, so the
extension rate — and therefore the capture — is zero. That is the two consumers of the
shared rate agreeing, visible in a golden.

### The measured effect — and the confounder that had to be removed

`DEEP_WATER_SCENARIO`: irrigation cut at sowing, 350 kg in the root zone, the default
195 kg below. Against a control that switches off **only the water transfer**
(`soil_extractable_water = 0`, so rooted depth grows identically):

| run | peak leaf C | grain C |
|---|---|---|
| capture on | **8.8398** | 3.6927 |
| control (`EXTR = 0`) | 3.5345 | 0.0000 |

A 2.5× canopy, and the difference between setting grain and setting none. The peak canopy
equals the **fully irrigated** reference season's 8.8398 to all printed figures: reaching
the subsoil is worth as much here as a boundary supply.

⚠ **The naive control (`subsoil_water0 = 0`) gives the same numbers but does not license
the claim** — it removes the water *and* freezes rooted depth, so it moves the nitrogen
gate too. The two controls agree stock-for-stock except `soil_n` at **1 ULP** (rel.
1.4e-16), which is the measurement that says the effect is water. This project has had a
causal attribution come back at 39 % before (`asserted-attributions-rot`); the experiment
that removes the cause is cheap and was run.

### Three things the build found that were not in the plan

1. **A dry subsoil is not a free choice — it is gated by the sowing depth.** Under our old
   uncited `rooted_depth = 0`, [F]'s `WSTORG = 0 ⇒ GRTD = 0` freezes depth at 0, making
   `FROOT1 = 0` and nitrogen uptake **identically zero**. So `water_biting`'s lean profile
   is survivable only because decision 3 shipped. Two cited mechanisms composed into a
   trap, and the escape was also a citation.
2. **The default profile abolishes the drought cascade, it does not weaken it.** With
   195 kg below, the irrigation-cut window never leaves `f_water = 1.0` (soil water bottoms
   at 149.4 kg, well above the 60 kg threshold) and end vegetative carbon goes 33.61 →
   33.28 instead of 33.61 → **12.68**. `DROUGHT_SCENARIO` therefore declares a dry subsoil,
   with the number recorded rather than the decision merely asserted.
3. **A test helper was re-listing the aux keys instead of copying them.**
   `test_soil_fractionation.build_variant` built its state with a literal
   `{THERMAL_TIME: 0, VERNALIZATION_DAYS: 0}` — silently omitting `rooted_depth`, harmless
   while that accumulator started at 0 and a real divergence the moment it did not. The
   file's own docstring already warned that *"carrying the aux across is load-bearing"*;
   the bug had simply moved one level up. Now copied from `build_season`.

### The exit state

* **12 goldens changed**, both drift summaries byte-identical; `rationed == 0` and
  `events == ()` everywhere, so the donor clamp never reaches the arbitration backstop.
* **Biosphere manifest**: `flow_set` 20 → 21 (`RootZoneCapture`), golden hashes refreshed.
  Station manifest refreshed. `param_files` **unchanged** — `EXTR`, `SOLDEP`,
  `subsoil_water0` and the sowing depth are scenario/soil data, like `sw_wilting` and
  `ground_area`, not crop params.
* **Both ports**: the Rust mirror carries the same single `extension_rate`, the same donor
  clamp, and the same re-sow return. `cargo test` green, `cargo clippy` clean.
* **14 new pins** in `tests/test_soil_layers.py`, **all mutation-verified** — seven
  deliberately broken variants (drop the donor clamp, drop the dry-subsoil gate, drop the
  soil cap, drop the re-sow return, reset depth to 0, use a flat ungated rate, restore the
  uncited sowing depth) each turn the intended pin red.
* **4 new Rust pins**, added after the port was measured to be **blind to its own
  transcription**. `cargo test` green proves Rust's tests pass; it is not parity, and the
  mirror carries three hand-written pieces. Measured: dropping the donor clamp, dropping
  the dry-subsoil stop, dropping the soil cap, using a unit `ground_area` in the re-sow
  return, and dropping `ground_area` from the capture call **all left the entire Rust
  suite green**. Each is now caught. Two of them need a plot that is not 1 m² and a store
  that empties — neither of which any frozen scenario provides, so the tests construct
  them.
* ⚠ **One flagged hazard turned out not to exist, and that is recorded so nobody re-adds
  a test for it.** A transposed `soil_extractable_water`/`ground_area` at a call site was
  raised as a risk invisible on 1 m² plots. It is invisible on *every* plot: the two are
  symmetric factors of a product, so transposing them is arithmetically identical for all
  inputs. Checked by mutation rather than argued. What a caller *can* get wrong is
  dropping a factor, and that is what the non-unit-area pin catches.
* One extensive-IC pin (`test_crew_coupled_loop`) caught `subsoil_water0` missing from the
  area-scaling similarity transform — the pin doing exactly its job on a new field.

### What this does and does not discharge

It discharges the water coupling the root-depth build named, and the soil rooting cap
`root_depth.yaml` recorded as deferred. It does **not** discharge, and each is now the
named successor in its own right:

* **`FTSW` as the stress driver** — the faithful form, orthogonal to reachability.
* **Drainage / runoff / soil evaporation** — `DRAIN` is `WSTORG`'s only input in [F].
* **Re-deriving the root-zone store from geometry** — the largest of the three, and the one
  that would change the frozen reference's science rather than add to it. The verdict is
  the user's, not ours.
