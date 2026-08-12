## **Re-deriving the soil water store from geometry** (the soil-layers build's own named finding, taken and BUILT)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-12 — the recorded price was wrong in BOTH directions, and only running it
showed that.** `docs/plans/post-roadmap-soil-water-rebasing.md`; probes
`M:/claud_projects/temp/soil-rebasing/`. The soil-layers build named this successor and
priced it as *"every frozen scenario water-stressed"*, verdict left to the user; the user
took it on that description. **Measured on the frozen tree before any design work, that
description is wrong on both halves of the roster, in opposite directions.** **FINDING 1 —
the open field survives cheaply.** Substituting the geometric store (`rooted_depth0 · EXTR ·
ρ · area = 0.15 × 0.13 × 1000 × 1 = 19.5` kg) costs `DEFAULT` 19 stressed days of 306, leaf
C peak 8.8398 → **7.0905** (−20 %) and storage C 22.4135 → **21.8961** (**−2 %**); `N_LIMITED`
moves by noise (the N gate dominates); `DROUGHT` goes 8.8398 → 6.4110 and finally bites.
**FINDING 2 — every sealed chamber DIES, and is LOCKED.** `SEALED_CHAMBER` and
`WATER_BITING` both hold `soil_water` at **exactly** 19.5000 for all 306 days, `f_water = 0`
throughout, depth frozen at 0.1500, leaf C **0.0500** (the seed) and storage C **0.0000**.
The cause is structural and was not in the plan: `soil.build_soil` **drops `Irrigation` and
the `water_source` boundary entirely when `sealed=True`** (the P3.3 genuine-closure
decision), so a sealed chamber's only inflow is `Recycling` — water the plant itself
transpired. That makes the wilting point an **absorbing state**: `soil_water ≤ sw_wilting ⇒
f_water = 0 ⇒ no transpiration ⇒ no vapour ⇒ no condensate ⇒ no recycling ⇒ soil_water
unchanged`, and since `f_water` also multiplies `root_depth.extension_rate`, depth freezes
and `RootZoneCapture` can never reach the 175.5 kg below. **It misses the escape by 0.5 kg**
(19.5 against `sw_wilting = 20.0`). ⚠ This is the **same trap shape** the soil-layers build
recorded as its own finding (1) — two cited mechanisms composing into a stop the crop cannot
escape — entered through a different door; a gate whose input is downstream of its own
output is an absorbing state, and no amount of reading finds it. **FINDING 3 — the
re-basing and the `FTSW` successor are ONE mechanism, not two, and the frozen science
survives their union intact.** The soil-layers plan filed them separately. They are
dimensionally inseparable: our stress driver is an **absolute-kg** ramp (`sw_wilting = 20`,
`sw_critical = 60`) calibrated against a 1000 kg store, while [F]'s is a **fraction**,
`FTSW = ATSW/TTSW` with `TTSW = DEPORT · EXTR` (Eqns 14.6/14.7), initialized `ATSW = DEPORT ·
EXTR · MAI` (14.26) — so `FTSW₀ = MAI` **independent of depth**. A small store that is full
reads as full, not as nearly-dead. Measured with the stress factor patched to [F]'s
`WSFG = min(1, FTSW/WSSG)` (Eqn 15.3) and `TTSW` read from the **step-entry** aux (exact
under Euler `dt = 1`): **not one carbon, nitrogen or oxygen amount moves on four of the five
scenarios** — `DEFAULT`, `SEALED_CHAMBER`, `N_LIMITED` and `DROUGHT` reproduce every stock
exactly and move **only** `soil_water` and `subsoil_water`, by exactly the over-declaration
removed (1172.2936 → 191.7936 = 1000 − 19.5; the subsoil by exactly 19.5). Only
`WATER_BITING`, the one scenario built to make water limit, changes its science (18 stocks;
`FTSW` dips to 0.1722 for 45 days). Not luck: every frozen scenario is a *potential
production* scenario, `FTSW` never falls below 0.79 sealed or 0.9957 open against
`WSSG = 0.30`, so `WSFG ≡ 1` exactly as `f_water ≡ 1` does today — **the two forms agree
wherever water does not limit, which is everywhere the freeze looks.** **FINDING 4 — a
capacity constraint is required, measured not assumed.** With the root zone bounded, nothing
stops irrigation refilling past it: `FTSW` reaches **2.21** on `DEFAULT` and **11.51** on
`DROUGHT`, i.e. the store holds 11× the water it can transpire and `soil_water` stops
meaning `ATSW`. Harmless to carbon only because `WSFG` clamps at 1. `RootZoneCapture` is
self-consistent (it raises `ATSW` and `TTSW` by the same `GRTD·EXTR`) and sealed `Recycling`
never exceeded 1.0 in measurement, so **irrigation is the sole overfill source** — [F]'s own
Eqn 14.8 (`IRGW = TTSW − ATSW`, irrigate to the drained upper limit) closes it with no new
stock or flow; drainage (Eqn 14.11 + Table 14.2's `DRAINF`) is the heavier alternative that
adds a flow, a boundary sink and a param and absorbs a separately-filed successor. **FINDING
5 — `subsoil_water0` currently double-counts the root zone.** The shipped default 195 kg is
`soil_depth · EXTR · ρ · area`, which is [F]'s **`IPATSW`** (Eqn 14.27, the *whole* profile),
where 14.28 gives `WSTORG = IPATSW − ATSW = 175.5`. `tests/test_soil_layers.py:89` pins the
identity, so **the pin holds a formula [F] does not have**; it is defensible only because
today's `soil_water0` is not geometric at all, so there is no `ATSW` to subtract — the
re-basing removes that excuse and must fix pin and value together, in the same commit rather
than as its own ceremony. **FINDING 6 — one factor, four processes.** [F] p. 195 (page
render) applies the deficit factor to **four** processes with **different** thresholds —
`WSFG` growth/transpiration, `WSFL` leaf area, `WSFD` phenology, `WSFN` nitrogen (Ch. 17) —
while our tree applies **one** `f_water` to three consumers, so the mapping is a decision to
be made deliberately. ⚠ And the root-extension consumer is **[E]'s citation, not [F]'s**:
[E] p. 137 says *"The effect of water stress on the rate of increase in rooted depth is
supposed to equal that of water uptake"*, whereas [F]'s own `GRTD` (Box 14.1) carries **no**
continuous water factor at all — only the discrete gates `CTU < tuBRG`, `CTU > tuTRG`,
`DDMP = 0`, `DEPORT ≥ SOLDEP`, `DEPORT ≥ MEED`, `WSTORG = 0`. Rescaling `f_water` silently
rescales [E]'s citation, so the two sources must be kept apart on purpose. **The thresholds
were read off a PAGE RENDER, not the text extraction** (PDF p. 210 = printed p. 195): wheat
`WSSL = 0.40`, `WSSG = 0.30`, `WSSD = 0.40`. `pdftotext` scrambles Table 15.1's columns and
**happened to land the right numbers on the wheat row** — exactly the coincidence that makes
a wrong pin look verified. ⚠ The literal `0..25` in the extraction is a typo in the
**printed** table (soybean's `WSSG`), not an extraction artefact; do not silently "fix" it if
soybean is ever added. **THE BRANCH THAT LOOKS CHEAP AND IS A TRAP, recorded so nobody
reaches for it:** raising `rooted_depth0` to 0.40 m — the *top* of the same cited [F] range
the 0.15 comes from — gives `ATSW₀ = 52` kg, inside the band, `f_water₀ = 0.80`, and the open
field survives at peak 8.7537 / storage 22.4037, near-untouched. It runs, it looks
calibrated, and the stress number it produces is a depth-derived store compared against
depth-independent thresholds, i.e. **meaningless**. **EXIT STATE: nothing built, `git diff
src/` empty** — the whole of the above is measurement on the frozen tree. The remaining
decision is scope (geometry + `FTSW` + Eqn 14.8, versus additionally building drainage), and
it is the **user's**, because every branch is a full unfreeze of the biosphere reference: it
moves goldens, both manifests, and the Rust mirror.
 **THE BUILD, on the user's call for the widest
scope ("yes, the station can have a water reservoir, of course. the drainage can be turned
on or off with at least a valve").** ⚠ **[F] AGREES WITH THE USER, WITH A CITATION, AND IT
MADE THE BUILD SMALLER.** Drainage was priced as "a new flow, a new boundary sink and a new
param"; the sink is wrong — [F] Eqn 14.12 is `WSTORG = WSTORG + DRAIN − EWAT`, and p. 176
spells out why (*"not all the drained water below the root layer may be considered a water
loss. All or part of the drained water to deeper soil may be exploited later by the crop due
to root growth"*). So drainage is `soil_water → subsoil_water`, the exact inverse of
`RootZoneCapture`, entirely in-system, crossing no boundary: conservation is **structural**
rather than asserted, the user's reservoir is the store the previous build already added,
and the below-root store — one-way within a season until now — becomes the two-way store
[F] always had. **The valve is `DRAINF`**, [F]'s own parameter: `drainage_factor = 0.0` is a
shut valve exactly, with no boolean and no branch of ours. **SHIPPED:** `soil_water0`
1000 → **19.5** kg (14.26), `subsoil_water0` 195 → **175.5** kg (14.27/14.28),
`sw_wilting`/`sw_critical` → **`wssg = 0.30`** (14.6/14.7/15.3, Table 15.1 wheat off a page
render), a new `Drainage` flow (`flow_set` 21 → 22), `Irrigation` demand-driven
(`IRGW = min(capacity, TTSW − ATSW)`, 14.8), and two new scenario fields
(`soil_moisture_index`, `drainage_factor`). **THE HEADLINE: ONLY WATER MOVED.** Predicted
before regeneration over all 25 goldens and confirmed — every scenario moves `soil_water` /
`subsoil_water` / `water_source` and **nothing else**; not one carbon, nitrogen or oxygen
amount at any horizon; both drift summaries **byte-identical**; `rationed == 0` and
`events == ()` everywhere. Only `water_biting` moves its science, and it is the one scenario
deliberately re-declared (`soil_moisture_index = 0.05`, chosen against its **own written
contract** — sustained bite, never fully wilted, crop alive, loop conserved — then swept
0.10 → 0.02, not fitted to a golden; leaf C 0.8299 → 0.7621). **FINDING 7 — drainage does
NOT let irrigation alone, and the first draft of the design said it did.** With the store
physically sized and `DRAINF = 0.3`, a flat 2 mm/day leaves the reference season at
`FTSW` 0.17 and costs **38 % of the yield**, with 204 kg draining below the root zone —
because the flat schedule was never sized against demand (peak **5.7744 kg/day**) and only
looked adequate against a 7.7 m bucket. Demand-driven irrigation restores the frozen science
exactly and **uses less water in total** (610 → 582.44 kg). ⚠ Consequence recorded because
nothing else will catch it: once irrigation is demand-driven, **`Drainage` is
bit-identically inert on the entire frozen roster** — `DRAINF` 0.3 and 0.0 give identical
states — so no golden protects it, exactly the position `root_depth` is in. Its pins are
unit-level and **mutation-verified on BOTH ports**; the four Rust mutations (drain a share
of the whole store instead of the excess, drop `DRAINF`, drop `ground_area` from the
capacity, drop the donor clamp) each turn a pin red, and the pins have to *construct* a
non-unit plot and an over-filled zone because no scenario provides either. **FINDING 8 — the
re-sow return was written against a store that could not run out.** It returned the
abandoned column *at the drained upper limit* (149.58 kg) clamped to what was held: a
rounding error against 1150 kg, more than the entire store at 19.5–169 kg, so its clamp
fired on **every** re-sow and handed the whole root zone to the subsoil. Measured: the
4-year sealed station made **no grain at all** and `annual_reset` raised "seed bank too
small to re-sow". Re-derived as the abandoned **fraction** of the held water — preserves
`FTSW` exactly across a re-sow, needs no clamp, and equals the old form at the drained upper
limit, so it generalises the cited geometry rather than departing from it. Measured over
eight cycles: one transient year, then a fixed point held to round-off (7e-14). ⚠ The old
comment had already *anticipated* the shortfall ("the root zone may hold less than its
geometry allows") and **clamped instead of re-deriving** — a clamp that turns a wrong amount
into a survivable one hides the wrongness until the store shrinks. **FINDING 9 — a test
helper had hand-copied that rule under a comment claiming it mirrored the tree.**
`test_soil_fractionation.reset_variant` kept the old water return when the rule changed, so
every variant run was a control against a tree that no longer existed — caught only because
a pinned CO₂ trough moved 1.2 %. This is the same shape that file's own docstring warns
about, one level up again; the durable fix is `season.resow_water_return`, **one function
with two callers**, mirrored on the Rust side for the same reason. **FINDING 10 — three
defects the change EXPOSED rather than caused.** (a) All three station builders, **on both
ports**, seeded their aux without `rooted_depth`, silently starting the station's crop at
depth 0 — invisible while the depth gate was inert, fatal once stress divides by
`TTSW = depth · EXTR · ρ · A`. (b) `harvest` injects a 1.3 m root system but inherited the
0.15 m zone's water, i.e. `FTSW = 0.115` on day 0 for a grain-filling crop; grain ended 79 %
low. The depth and the water are two halves of one declaration and are now derived together.
(c) `DEEP_WATER` is not viable at zero irrigation, and the reason is the sharpest single
consequence of sizing the soil honestly: **a crop rooting to 1.3 m over 1 m² can ever reach
169 kg of extractable water against a 582 kg season demand.** No soil depth fixes that (the
CROP cap binds). Its old `soil_water0 = 350` hid it — 2.7 m of extractable water in a 15 cm
layer. It now declares a limited supply and the mechanism it exists to show comes out
**stronger**: 15× the canopy against the control, where the old declaration gave 2.5×.
⚠ **And that control had to be rebuilt, silently:** `soil_extractable_water = 0` was the
"clean" control, but `EXTR` now appears in **two** places — the transfer *and* `TTSW` — so
zeroing it kills the crop outright instead of isolating the transfer. It is now "drop the
`RootZoneCapture` flow from the registry". A control that changes more than it claims is
worse than none, and this one would have kept passing while measuring something else.
**FINDING 11 — two acceptance-gate claims died and were replaced by RANK PLUS EXACT VALUES,
not by looser thresholds.** Water's slack in `open_season` fell **189.24 → 9.31** (a margin
of 189× was never a fact about safety, it was a fact about a bucket that could not exist),
and `carbon_pool > 4 × runner-up` became **3.98×** on two chambers. `test_acceptance_gate.py`
refuses fitted cuts in its own words, so both bounds were dropped for the rank plus an
exactly-pinned runner-up — strictly stronger, since a threshold only catches changes bigger
than its slack. The roster-wide runner-up has now changed identity twice (`o2_pool` →
`power.battery` → `soil_water`), and the retired "even the runner-up is a chamber property"
corollary was **not** restored just because it drifted back to being true. **A UNITS DEFECT
IN THE SOURCE, recorded rather than worked around:** [F] Table 14.2's `SOLDEP` column
(210/150/60, captioned **mm**) cannot be a profile depth in millimetres — the same book puts
`DEPORT` at emergence at 150–400 mm and wheat's `MEED` at 1200 mm, and Box 14.1 stops root
growth at `DEPORT >= SOLDEP`, so a 60 mm soil would stop a crop before it emerged. Read as
cm the column is 2.1/1.5/0.6 m and our 1.5 m is the middle row exactly → silty loam →
`DRAINF = 0.3`. (Possibly horizon thicknesses rather than a typo; the pick is unaffected
either way. What is *proven* is that the mm reading is impossible, not that the caption is
wrong.) **EXIT:** `flow_set` 21 → 22, `param_files` **unchanged** (`wssg`/`MAI`/`DRAINF` are
scenario/soil data), 12 goldens, both manifests, both ports, **2290 Python tests + the full
Rust suite + all 101 cross-port parity checks green**. **NOT built, each a named successor:**
`WSSL` (leaf-area expansion, 0.40) and `WSSD` (phenology, 0.40) — [F] applies the deficit
factor to **four** processes with different thresholds and we carry one, because there is no
water-gated leaf-expansion or drought-accelerated development term for the others to attach
to; runoff and soil evaporation; and making `DROUGHT` actually bite (`FTSW` bottoms at 0.7039
— not new, the soil-layers build already recorded that the reachable subsoil *abolishes* that
cascade; now a one-field change, but it would move a golden's science for a reason outside
this charge).
