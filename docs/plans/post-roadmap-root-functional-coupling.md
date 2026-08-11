# Root functional coupling — the NITROGEN half REFUSED 2026-08-11, NOT BUILT

⚠ **Scope of this verdict.** What is refused is **rooting depth gating NITROGEN**, measured
inert. The **water** coupling is a different shape and is **NOT refuted** — it was never
run, because it cannot be run without splitting the soil pool into layers. Do not read the
refusal as covering it; see "What is NOT refuted" below.

Read-only so far. No `src/`, param, golden or manifest change; `git diff src/` empty.
Probe scripts live outside the tree (`M:/claud_projects/temp/root-coupling/`); every
number below is reproducible from them.

## Why this was taken

`post-roadmap-wheat-partition-backfill.md` refused the cited winter-wheat partition table
and named its own successor:

> `ROOT_C` is read in exactly one place outside plumbing … **There is no uptake function.**
> Root carbon buys nothing … So carbon sent below ground is, in our model, **dead weight by
> construction.** … **Do not re-attempt this backfill until roots do work.**

The user chose that successor over three alternatives. The charge is therefore specific:
**make root carbon buy something**, so that an allocation table can be judged on whether
its root share earns its keep rather than on whether it was fitted to a canopy band.

## The obvious entry point, and why it is a dead end

`NitrogenUptake` (`nitrogen.py`) already has a capacity term, and its denominator is the
plot, not the plant:

```
capacity = max_uptake_capacity · ground_area · soil_n_availability(soil_n)
flux     = min(deficit, capacity) · dt
```

`max_uptake_capacity` is `nitrogen.yaml`'s **one remaining `TODO(cite)`** — and its own
source tag already calls it *"a non-binding NUMERICAL SAFETY CEILING, not an empirical
quantity"*, reasoned from `[D]` to be **~6× above** the fastest reported wheat N
accumulation. Replacing a per-area ceiling with a per-root-mass specific uptake rate looked
like the project's proven move: **retire a param by changing the FORM, not by hunting a
citation** (the shape that worked twice in `post-roadmap-nitrogen-cycle-form.md`).

Three measurements killed it in that shape.

### Measurement 1 — the capacity term is not the binding constraint anywhere

Every step of all nine biosphere scenarios, classified as capacity-bound
(`capacity < deficit`) or demand-bound:

| scenario | steps | capacity-bound | demand-bound | inactive | max(deficit/capacity) | peak root (mol C) |
|---|---|---|---|---|---|---|
| `default` (open_season) | 306 | **0** | 117 | 189 | 0.5281 | 8.809 |
| `sealed_chamber` | 916 | **0** | 77 | 839 | 0.1238 | 0.628 |
| `perennial_chamber` | 916 | **0** | 78 | 838 | 0.1315 | 0.658 |
| `consumer_chamber` | 916 | **0** | 66 | 850 | 0.2434 | 0.758 |
| `n_limited` | 306 | **0** | 0 | 306 | 0.0000 | 0.107 |
| `water_biting` | 306 | **0** | 69 | 237 | 0.1281 | 0.638 |
| `day_neutral` | 306 | **0** | 37 | 269 | 0.0143 | 0.147 |
| `potato` | 306 | **0** | 47 | 259 | 0.0295 | 0.261 |
| `drought` | 306 | **0** | 117 | 189 | 0.5281 | 8.809 |

**Zero capacity-bound steps on the whole roster.** `n_limited` is inactive throughout for a
different reason, worth recording: its soil N sits below `sn_residual`, so availability is
identically 0 and the *stress* there is pure dilution of a fixed reserve — the scenario
named for nitrogen limitation exercises the stress ramp, never the supply term.

### Measurement 2 — what a root-proportional rate would have to be

If capacity became `specific_rate · root_dry_mass · availability`, the rate at which the
flow stays non-binding is `max over steps of deficit / (root_DM · availability)`:

| scenario | kg N/kg root/day | mmol N/g root/day |
|---|---|---|
| `default` (open_season) | 0.02168 | **1.547** |
| `consumer_chamber` | 0.02471 | 1.764 |
| `perennial_chamber` | 0.01536 | 1.097 |
| `sealed_chamber` | 0.01520 | 1.085 |
| `water_biting` | 0.01690 | 1.207 |
| `potato` | 0.01088 | 0.777 |
| `day_neutral` | 0.00761 | 0.543 |

So the form is buildable and the required magnitudes are in a physiologically discussable
range. **What is missing is the citation** — and the shelf sweep below is what settled it.

### Measurement 3 — the cited band is scientifically INERT, and that is the sharp finding

`[F]` Soltani & Sinclair give the maximum-N-accumulation parameter first-hand, and it is
**per ground area, exactly like ours**:

> "Maximum rate of N accumulation is generally between **0.2 and 0.6 g N m⁻² day⁻¹** (Viets,
> 1965; Sinclair and Amir, 1992; Sinclair et al., 2003)."

That is 0.0002–0.0006 kg/m²/day against our frozen **0.0015** — an independent source
lineage confirming the file's own `[D]`-based "~6× too high", and landing essentially on
the 0.0003–0.0005 range the file had *guessed* for a realistic value.

Re-running every scenario with the ceiling moved into that band, comparing the **full final
state stock-by-stock in hex-float** (the comparison the goldens make):

| ceiling (kg/m²/day) | 0.0006 | 0.0005 | 0.0004 | 0.0003 | 0.0002 |
|---|---|---|---|---|---|
| `default` (open_season) | 1/14 moved | 1/14 | 1/14 | 1/14 | 2/14 |
| every other scenario | **bit-identical** | bit-identical | bit-identical | bit-identical | `consumer_chamber` 2/20 |

The one stock that moves in `open_season` across 0.0003–0.0006 is `soil_n`, at relative
**1.4e-16 to 2.1e-15** — one to a few last bits. Peak leaf carbon is unchanged to all 6
printed figures at every ceiling. Only at 0.0002, the very bottom of the band, does
anything real move (`plant_n` −5.4 %).

**Why it is inert, and the general lesson.** Uptake is *target-seeking*: the deficit is a
level (`target · biomass − plant_n`), and the flow closes it in one step at `dt = 1` (the
deadbeat case the file already documents). Capping the *rate* only makes the plant take
more days to reach the *same level*, and the season is far longer than the delay. So a rate
ceiling on a level-seeking flow is near-inert on any horizon long compared with the
approach time — which is why a parameter can sit 6× out of its literature band for the
project's whole history and never once be caught by a golden.

⚠ **Inert is not free.** Because the goldens compare hex-float exactly, the few-ULP `soil_n`
drift in `open_season` still **moves a golden**. So citing this parameter properly is a
biosphere **unfreeze with the full ceremony** (manifest, `biosphere_params.txt`, the Rust
mirror, the cross-port tier) for a change that alters no scientific quantity.

⚠ It is, however, **provably not calibration** — the objection `nitrogen.yaml` itself
raises against moving `n_residual` and `n_critical` ("changing it is calibration = scope
B"). You cannot be fitting an output that is unchanged to 15 significant figures. That
distinction is measured here, not argued.

## THE BLOCKER FOR THE ACTUAL CHARGE — two primaries, same verdict

Both sources on the shelf that model root function make **rooting DEPTH** the functional
variable, and **neither derives it from root biomass**.

`[F]` Soltani & Sinclair couple root depth into both resources:

```
FROOT1 = min(DEPORT / DEP1, 1)                  ratio of rooting depth to top-layer depth
SNAVL  = (NCON − 0.000001) · ATSW1 · 1000 · FROOT1        soil N available to the crop
TTSW   = DEPORT · EXTR                          total transpirable soil water in the root zone
```

…and then drives the depth itself by a **constant daily extension rate**:

```
DEPORT_i = DEPORT_{i−1} + GRTD          GRTD a crop constant (mm/day), gated to zero by
                                        phenology, zero dry-matter production, water stress,
                                        or hitting max effective/soil depth
```

`[E]` Penning de Vries independently does the same thing:

```
GZRT = GZRTC · WSERT · TERT              GZRTC = 0.03 m/day (a PARAMETER), scaled only by
                                         water-stress and temperature factors
```

**No root-mass term in either.** And `[E]` supplies the coupling in the *opposite*
direction — Brouwer's functional equilibrium, p. (§ water relations):

> "Carbohydrate partitioning between shoot and root under water stress is altered in favour
> of the root biomass. Brouwer (in de Wit et al., 1978) described the biological principle
> of the mechanism; **roots are formed in proportion to the demand from shoots for water.**
> … At higher stress levels during the vegetative phase, the share that goes to the roots
> increases by up to 50 % of the amount that otherwise would go to the shoot."

⇒ **Root carbon is decoupled from root FUNCTION on purpose, and `[E]` says so outright.**
This is not an omission either source made; it is a stated modelling principle with a
physical reason (p. 136):

> "The length of fibrous roots can vary enormously without much impact on root weight.
> Hence, **simulation of rooted depth occurs independently of the growth of root mass.**"

That single sentence is the answer to the charge. The wheat doc's finding — root carbon is
dead weight in our model — is **true**, but the remedy it implied (make root carbon buy
something) is **contradicted by the primary**: the quantity that does the work is root
*length/depth*, and length is not a function of weight. Care with the scope of this claim:
`[E]` decouples depth from mass, which is not the same as saying root carbon has no
function at all — the "consequence, not cause" reading above holds specifically along the
*depth* chain.

⚠ And root carbon is not merely useless, it is **costly**: `respiration.py` charges
maintenance on `Σ(leaf + stem + root)`, and senescence bleeds roots at `rdr_root =
0.01/day`. So allocation below ground is a pure sink — which is exactly the bias that let a
*fitted* root share pass the canopy band in the wheat backfill.

## Measurement 4 — THE VERDICT: a rooting-depth gate on nitrogen is INERT too

Before designing the build, the gate was measured rather than assumed. `NitrogenUptake`'s
availability was multiplied by `FROOT1 = min(rooted_depth / soil_layer_depth, 1)` with depth
on `[E]`'s trajectory (0.15 m at sowing, +0.02 m/day, capped), swept over layer depths of
0.2 / 0.5 / 1.0 m, alone and combined with the cited uptake ceiling. Full final state,
stock-by-stock, hex-float:

| case | result across all 8 scenarios |
|---|---|
| `FROOT1` gate only, layer 0.2 m | **BIT-IDENTICAL** |
| `FROOT1` gate only, layer 0.5 m | **BIT-IDENTICAL** |
| `FROOT1` gate only, layer 1.0 m | **BIT-IDENTICAL** |
| `FROOT1` + ceiling 0.0003 | identical to the ceiling acting alone — the gate adds **nothing** |
| `FROOT1` + ceiling 0.0002 | identical to the ceiling acting alone — the gate adds **nothing** |

**Zero effect, at every layer depth, alone or combined.** The two supply-side reductions do
not compound into a bite.

⚠ **COVERAGE CORRECTION — the first pass measured the wrong roster, and this is recorded
rather than quietly fixed.** Measurements 3 and 4 were first run on the eight scenario
*constants*, at horizons picked by hand (3 years for the chambers). The manifest freezes
something different: **seven scenario→golden pairs at specific horizons** — `open_season`
1 yr, `sealed_chamber` 3 yr, `perennial_chamber` **5**, `consumer_chamber` **5**,
`perennial_long_horizon` **15**, `consumer_long_horizon` **15**, plus `drift_summary` (a
stability signature over the two 15-year runs). So the original "bit-identical on all eight
scenarios" was a **coverage claim about a roster that is not the frozen one** — four of the
seven were run at the wrong horizon and the two 15-year runs not at all. Exactly the
`multirate-crossport-anchor-partition-parity` shape (*"the suite was measurably blind"*).

Re-run against the manifest's own roster and horizons, and comparing the **aux channel** as
well as the stocks:

| frozen scenario | horizon | `FROOT1` gate (0.5 m / 1.0 m, ± anthesis stop) | ceiling 0.0006 | ceiling 0.0003 | both |
|---|---|---|---|---|---|
| `open_season` | 1 yr | BIT-IDENTICAL | `soil_n` 1.4e-16 | `soil_n` 2.1e-15 | `soil_n` 2.1e-15 |
| `sealed_chamber` | 3 yr | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL |
| `perennial_chamber` | 5 yr | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL |
| `consumer_chamber` | 5 yr | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL |
| `perennial_long_horizon` | **15 yr** | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL |
| `consumer_long_horizon` | **15 yr** | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL | BIT-IDENTICAL |

`drift_summary` follows: it is a signature over the two 15-year runs, both bit-identical.
**The verdict survives the correction** — but it is now a measurement of the frozen roster
rather than of a roster that resembled it. The long horizons were the place it could have
failed, since the doc's own inertness argument is that the run is long relative to the
delay.

⚠ The first pass also declared an anthesis stop for the depth trajectory and **never applied
it** — `[E]`'s *"root growth generally stops around flowering"* was in the comment, not in
the code. The re-run applies it (depth frozen from day 150) and also runs the un-gated
control: **all three variants bit-identical**, so the verdict does not rest on which one ran.

**Why — and this is the structural finding of the whole exercise.** Measurement 1 showed
uptake is **demand-bound on every step of every scenario**. `FROOT1` and the ceiling both
shrink *supply*, and supply has ≥1.9× headroom at its tightest and ≥7.6× nearly everywhere.
Worse, the gate is anti-correlated with the need for it: **rooting depth and nitrogen demand
grow together** — depth is smallest when the plant is a seedling demanding almost nothing,
and has saturated (`FROOT1 = 1`) long before demand peaks around day 210. A depth gate can
never catch up with a demand that outruns it.

⇒ **No supply-side root coupling can bite in this model.** That is not a fact about our
scenarios; it is a fact about the flow's form.

## The verdict, against the project's own precedent

`post-roadmap-canopy-regulator.md` priced exactly this position and refused it:

> **Benefit on the frozen tree: exactly zero, bit-for-bit.** Adopting it alone would move
> nothing and cost a full cascade — its only value is as a *precondition* for a form the
> tree refuses.

This build is in the same position, and the same verdict follows: **rooting depth gating
nitrogen is NOT BUILT.** It would cost a new aux accumulator (`aux_set` 2 → 3, a biosphere
unfreeze), a new param file, both ports and the cross-port tier, to change **nothing,
bit-for-bit**. That is the fourth measurably-inert mechanism in this series (canopy
regulator, stem reserves' trigger, the uptake ceiling, this).

## What is NOT refuted — the water side, and its real price

Depth was measured inert **on nitrogen only**. The water coupling is a different shape and
is untested: `TTSW = DEPORT · EXTR` changes the *size of the accessible pool*, not the rate
of a demand-bound flow — and water stress genuinely bites in `water_biting` and `drought`.

But it cannot be done additively. Our soil water is a **single pool** with fixed
wilting/critical thresholds; deeper roots reach water that is physically present but
currently unreachable, so representing that needs the pool split into reachable and
unreachable parts — i.e. **soil layers**. That is a structural change to the frozen
biosphere reference, not a new mechanism alongside it, and both sources assume layers
throughout (`ZRTL`, `ATSW1`, the L2SS/L2SU modules).

⚠ Also still owed if that is ever taken: `[E]` Table 25 carries per-species rates and
maximum depths, but its text layer **column-collapses** (18 rate values against 15 species
labels; the maximum-depth column reads `ies) 1.8 1.8 1.0 OM …`) — the same failure mode as
`[E]` Table 18 and `[F]` Table 17.1, needing the page-image method. Two of the rate values
are flagged `*` = the source's own **estimate**, and the wheat rows may be among them.
Clean from body text, and enough to have run measurement 4: *"Rooted depth can increase at a
rate of 3–5 cm d⁻¹"*, *"Root growth generally stops around flowering"*, and that the
temperature effect is taken equal to photosynthesis's and the water-stress effect equal to
that of water uptake at the root tips.

## The fork that remains

**(a) Cite the uptake ceiling and move it into the band.** Retires `nitrogen.yaml`'s last
`TODO(cite)`; measured scientifically inert (7 of 8 scenarios bit-identical, one stock at
1–2 ULPs); costs a full biosphere unfreeze ceremony for a zero-science change; does
**nothing** for roots. Small, clean, and honest about what it is.

**(b) Soil layers, so depth can gate water.** The only remaining way to make roots do work.
A structural change to a frozen reference — the largest single piece of work the
post-roadmap record has considered — and its parameters need the page-image read.

**(c) Stop here.** The diagnosis is the deliverable: the charge is answered (root *carbon*
has no cited function, by the primary's own statement), the supply-side dead end is measured
rather than assumed, and the price of the one live successor is written down.

## Sources

* **[E]** Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M. & Bakema, A. (1989),
  *Simulation of Ecophysiological Processes of Growth in Several Annual Crops*, Simulation
  Monographs 29, PUDOC/IRRI. Rooted-depth definition; `GZRT = GZRTC·WSERT·TERT` with
  `GZRTC = 0.03`; Brouwer functional-equilibrium partitioning. Already first-hand for the
  potato params and the stem-reserve lead.
* **[F]** Soltani, A. & Sinclair, T.R., *Modeling Physiology of Crop Development, Growth and
  Yield*, CABI. Ch. 14 (`DEPORT`, `GRTD`, `TTSW`), Ch. 17 (`MXNUP`, the 0.2–0.6 g N/m²/day
  band, Table 17.1), Ch. 18 (`FROOT1`, `SNAVL`). ⚠ Table 17.1's wheat `MXNUP` cell
  **column-collapsed** in the text layer — same failure mode as `[E]` Table 18; a
  wheat-specific value needs a page-image read. The 0.2–0.6 band quoted above is body text
  and extracted cleanly.
* **[D]** Meng et al. (2013), PLoS ONE 8(7):e68783 — already cited in `nitrogen.yaml`; the
  2.44 kg N/ha/day peak period-average that first flagged the ceiling as ~6× high.
