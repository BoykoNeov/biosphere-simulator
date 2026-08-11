# Root functional coupling — DIAGNOSED 2026-08-11, NOT BUILT (fork open)

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

⇒ **In the cited literature, root carbon is a CONSEQUENCE of stress, not a CAUSE of uptake.**
"Make root carbon buy something" is not a form the shelf offers. What the shelf offers is a
*rooting-depth state* that does real work on both water and nitrogen, plus a *stress →
allocation* rule that grows roots. Those two close a loop only if depth is tied to root
mass — and **that link is exactly the uncited one**, in both sources.

This is the same shape the project has now hit three times (the canopy regulator, stem
reserves, this): **the mechanism we want rests on a state variable we do not have.** The
difference here is that the missing state is cited, buildable, and does work — it just is
not the state the charge names.

## The fork — the user's call, not settled here

**(a) Cite the uptake ceiling and move it into the band.** Retires `nitrogen.yaml`'s last
`TODO(cite)`; measured scientifically inert (7 of 8 scenarios bit-identical, one stock at
1–2 ULPs); costs a full biosphere unfreeze ceremony for a zero-science change; does
**nothing** for roots. Small, clean, and honest about what it is.

**(b) Build rooting depth as a state.** Genuinely cited twice over, gates both water
(`TTSW`) and nitrogen (`SNAVL`), and makes depth-limited growth a real mechanism the model
currently lacks. Bigger: a new state variable, a new param file, both ports, a full
unfreeze. But it leaves `ROOT_C` **exactly as inert as it is today** — so it does not
discharge the wheat doc's successor, it replaces it.

**(c) Refuse, and record why.** The charge as written — make root *carbon* productive —
has no cited form on this shelf; building one would be inventing the link both primaries
decline to make. That is the move this project normally refuses.

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
