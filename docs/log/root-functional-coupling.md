## **Root functional coupling** (the wheat refusal's named successor)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**REFUSED on the measurement, then BUILT on the user's call, 2026-08-11.** ⚠ **The order is
the finding.** The charge as written (*make root CARBON buy something*) is refused **by the
primary itself**; the fallback (rooting depth gating nitrogen) was measured
**bit-identically inert on the whole frozen roster** and recorded NOT BUILT on the
canopy-regulator precedent; **the user overruled that refusal and directed the build**,
which shipped as a third aux accumulator, two cited params, both ports, and a biosphere
unfreeze that **moved no value** (12 goldens differ by exactly one added `aux` key; not one
stock amount moved, at any horizon). Anyone who later finds the inertness has found a
**documented choice, not an oversight**. **WHAT THE BUILD ADDED:** `rooted_depth` advanced
by [E] p.137's own law (`GZRT = GZRTC·WSERT·TERT`, zero at DVS>=1 per *"root growth
generally stops around flowering"*, zero at the cap), gating `NitrogenUptake`'s supply by
`FROOT1 = min(depth/soil_layer_depth, 1)` ([F] Soltani & Sinclair). `f_temp`/`f_water` are
**the tree's existing functions** — [E] says outright to reuse the photosynthesis
temperature response and the water-uptake stress response, so **no response curve was
invented**. Params from **[E] Table 25 p.137 read off the PAGE IMAGE** (its text layer
column-collapses — the third table in this project to need that method): winter wheat
**0.018 m/d, 1.3 m** (Gregory et al. 1978); potato **overrides** from Table 25's own potato
row (Vos & Groenwold 1986, 0.014 m/d, 0.8-1.0 m → **0.9 recorded as a midpoint-of-range
reading, not a transcription**). ⚠ Table 25's two `*  estimate` flags are **Sugar-beet and
Tulip, NOT wheat**, so the headline param is data. `soil_layer_depth = 0.30 m` is **DESIGN
and unciteable by construction** ([F]'s DEP1 is the evaporation layer of a layered soil
model we lack); measured inert at 0.2/0.5/1.0 m, with the re-open condition written down
(*if uptake ever becomes supply-bound*). **THE ONE REAL BUG THE BUILD FOUND WAS IN THE
BUILD:** regenerating goldens moved two amounts in `harvest_state.json` (`plant_n` **−12.3
%**), because `HARVEST_SCENARIO` fast-forwards the crop **past anthesis** to get a
grain-filling plant in 7 days — and the new law stops rooting at flowering, so that crop was
**a grain-filling plant that never grew roots and never could**, taking up no nitrogen at
all. Fixed as an INITIAL CONDITION (`HarvestScenario.rooted_depth0`, the exact sibling of
the `thermal_time0` that creates the situation), not a tolerance: a scenario that starts a
crop mid-life must state the crop's mid-life state. ⚠ The fix also happens to restore a
purely additive diff, and that is **named as a coincidence rather than leaned on** — the
argument is botanical and would stand if the golden had stayed moved; picking an IC because
it quiets a golden is the refused shape, and the reasoning is in the field's own comment so
it can be audited. ⚠ **SECOND COVERAGE LESSON IN ONE EXERCISE:** `harvest` is a *station*
golden, outside the biosphere manifest's seven, so the probe roster never touched it —
**after the roster had already been corrected once** for using hand-picked horizons instead
of the manifest's (5-yr chambers, two 15-yr runs). Twice, the answer to *"did you measure
everything?"* was *"no, and the miss is where the effect was."* **EXIT STATE:** crop
vocabulary 8→9 and both the vocabulary pin and potato's partition pin went red and were
updated **deliberately with the reasoning in the test** — which is what those pins are for;
`aux_set` 2→3; both manifests refreshed; Rust mirror carries the same law and the same **cap
form** (a *rate* cut-off, not an increment clamp, so the aux channel's dt-independence
contract holds — carried to the port as a RULE, not re-decided there); **15 new pins, all
mutation-verified** (deleting the gate factor, ignoring the flowering stop, flattening the
rate, dropping the re-sow reset, removing the aux process each turn exactly one test red) —
**they exist because no golden can catch any of those mutations**; 2248 passed,
ruff/pyright/cargo clean. **WHAT IT DOES NOT DISCHARGE:** `ROOT_C` still buys nothing and by
[E] p.136 it should not — that question is **closed by citation, not deferred**. What is
newly available is the **water** coupling (`TTSW = DEPORT·EXTR` attaches to this
accumulator), still unbuilt: it needs the soil-water pool split into layers. **ORIGINAL
DIAGNOSIS FOLLOWS.** **DIAGNOSED 2026-08-11 — read-only; `git diff src/` empty, no golden,
param or manifest moved.** `docs/plans/post-roadmap-root-functional-coupling.md`. **The
charge:** the wheat partition backfill refused a cited table and named its own successor —
*"`ROOT_C` … buys nothing … dead weight by construction. **Do not re-attempt until roots do
work.**"* The user chose it over three alternatives. **THE PRIMARY ANSWERS THE CHARGE
OUTRIGHT, AND IT IS A REFUSAL:** [E] p. 136 — *"The length of fibrous roots can vary
enormously without much impact on root weight. Hence, **simulation of rooted depth occurs
independently of the growth of root mass**."* Root carbon is decoupled from root function
**on purpose**, with a stated physical reason, in a source already first-hand for five other
rows. So the wheat doc's finding stands (root carbon is dead weight in our model) while the
remedy it implied — make root *carbon* productive — is **contradicted by the source**.
Confirmed independently: [F] Soltani & Sinclair drive `DEPORT_i = DEPORT_{i−1} + GRTD` from
a crop constant; [E] drives `GZRT = GZRTC·WSERT·TERT` from `GZRTC = 0.03 m/day` — **no
root-mass term in either**, and [E] supplies the coupling in the *opposite* direction
(Brouwer: water stress shifts up to +50 % of shoot allocation to roots). **FOUR
MEASUREMENTS, EACH KILLING A CHEAPER OPTION.** (1) The uptake capacity term is the binding
constraint on **zero steps of all nine scenarios** — every active step is demand-bound;
`n_limited` never exercises the supply term at all (availability ≡ 0; its stress is pure
dilution of a fixed reserve, which is worth knowing about the scenario named for N
limitation). (2) A root-proportional rate would need 0.54–1.76 mmol N/g root/day —
buildable, but uncited. (3) **THE SHARP ONE — the cited band is scientifically INERT.** [F]
gives the parameter first-hand *per ground area, exactly like ours*: *"Maximum rate of N
accumulation is generally between **0.2 and 0.6 g N m⁻² day⁻¹**"* — 0.0002–0.0006 kg/m²/day
against our frozen **0.0015**, an independent lineage confirming the file's own [D]-based
"~6× too high" and landing on the 0.0003–0.0005 the file had *guessed*. Moving it anywhere
into that band leaves **7 of 8 scenarios bit-identical**; `open_season` moves exactly one
stock (`soil_n`) at rel **1.4e-16–2.1e-15**, one to a few last bits. **Why:** uptake is
*target-seeking* — the deficit is a level and the flow is deadbeat at `dt = 1`, so capping
the *rate* only delays reaching the *same level*, and the season is far longer than the
delay. ⇒ **a parameter sat 6× outside its literature band for the project's whole history
and no golden could ever have caught it.** ⚠ Inert is **not free**: hex-float goldens make
even the ULP drift an unfreeze. ⚠ But it is **provably not calibration** — the very
objection `nitrogen.yaml` raises against moving `n_residual`/`n_critical` — because you
cannot be fitting an output unchanged to 15 significant figures. (4) **THE VERDICT — the
depth gate is inert too.** `FROOT1 = min(depth/layer, 1)` on availability, depth on [E]'s
trajectory, swept over layer depths 0.2/0.5/1.0 m, alone **and** combined with the cited
ceiling: **BIT-IDENTICAL** in every case; combining adds nothing over the ceiling alone. ⚠
**COVERAGE CORRECTED, recorded not quietly fixed:** the first pass measured eight scenario
*constants* at hand-picked horizons, which is **not the frozen roster** — the manifest
freezes seven scenario→golden pairs at specific horizons (`perennial_chamber` and
`consumer_chamber` at **5** yr, not 3; plus `perennial_long_horizon`,
`consumer_long_horizon` at **15** yr and `drift_summary`, none of which were run at all).
Re-run against the manifest's own roster and horizons, comparing the **aux channel** as well
as the stocks: **bit-identical everywhere**, including both 15-year runs — the place it
could have failed, since the inertness argument is that the run is long relative to the
delay. Same shape as `multirate-crossport-anchor-partition-parity` (*"the suite was
measurably blind"*). The first pass also declared [E]'s "root growth stops around flowering"
stop and **never applied it**; the re-run applies it and runs the un-gated control — all
three variants identical, so the verdict does not rest on which ran. **The cause is the
flow's FORM, not the scenarios:** both mechanisms shrink *supply*, supply has ≥1.9× headroom
at its tightest, and the gate is **anti-correlated with the need for it** — rooting depth
and N demand grow together, so depth saturates (`FROOT1 = 1`) long before demand peaks
around day 210. ⇒ **no supply-side root coupling can bite in this model.** **NOT BUILT**, on
`post-roadmap-canopy-regulator.md`'s own precedent quoted in the doc (*"benefit on the
frozen tree: exactly zero, bit-for-bit … would move nothing and cost a full cascade"*) — it
would cost `aux_set` 2→3 (a biosphere unfreeze), a new param file, both ports and the
cross-port tier, to change nothing. The fourth measurably-inert mechanism in this series.
**Corrected in passing:** roots **do** carry maintenance respiration (`Σ(leaf+stem+root)`)
and senesce at `rdr_root = 0.01/day`, so below-ground allocation is a *costly sink*, not
free — which sharpens rather than softens why a **fitted** root share could pass the canopy
band. **NOT refuted, and priced:** the *water* coupling (`TTSW = DEPORT·EXTR`) changes pool
**size** rather than a demand-bound **rate**, and water stress genuinely bites in
`water_biting`/`drought` — but our soil water is a single pool, so it needs the pool split
into reachable/unreachable, i.e. **soil layers**: a structural change to the frozen
reference, which both sources assume throughout. ⚠ Still owed if ever taken: [E] Table 25's
per-species rates and maximum depths **column-collapse** in the text layer (18 rate values
against 15 species labels; the depth column reads `ies) 1.8 1.8 1.0 OM …`) — the third table
in this project to need the page-image method — and two rate values are the source's own
flagged **estimates**, possibly the wheat rows. Clean from body text and enough to run
measurement 4: *"Rooted depth can increase at a rate of 3–5 cm d⁻¹"* and *"Root growth
generally stops around flowering."*
