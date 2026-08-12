## **The remaining water-stress curves** — `WSFD` BUILT, `WSFL` REFUSED

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per
> work item. The fuller record is the plan doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT + REFUSED 2026-08-12** — the two successors the soil-water re-basing named, taken
together on the user's call. `docs/plans/post-roadmap-water-stress-curves.md`; probes
`M:/claud_projects/temp/water-stress-curves/`. [F] applies its water-deficit factor to
**four** processes; after the re-basing we carried **one**. **FINDING 1 — THE RECORD THAT
NAMED THE SUCCESSOR MIS-FILED IT, AND THE MIS-FILING WAS THE PRICE.** It wrote *"`WSSL`
(leaf-area expansion, 0.40) and `WSSD` (phenology, 0.40) — [F] applies the deficit factor
to four processes **with different thresholds**"*. `WSSD` is not a threshold. Table 15.1's
own caption reads *"Threshold FTSW for leaf area development (WSSL) and growth (WSSG), **and
a coefficient of phenological development response to drought (WSSD)**"*, and Eqn 15.8 is
`WSFD = (1 − WSFG)·WSSD + 1` — driven by `WSFG`, which the tree already computes. So the
build needed **no new `FTSW` call site, no new threshold, no second comparison**: one scalar
and a factor already in hand. A word in a successor's own description priced it as a
mechanism it is not; **correcting a price downward is a finding, and nothing but re-reading
the caption produces it.** The same page render also shows `WSSD` populated for only **two
of the table's ten crops** (wheat 0.40, chickpea 0.40), which [F] explains in its own words
— *"the scientific basis and a procedure to measure WSSD need to be sought"*. **FINDING 2 —
`WSFL` REFUSED, and the refusal moved from being about US to being about the SOURCE.** The
recorded blocker was *"we have no water-gated leaf-expansion term for it to attach to"* — a
claim about our tree, and therefore erodable by building one. [F] Box 16.2 computes daily
LAI increase in three phenological branches and applies `WSFL` to **exactly one**: the
node-driven `GLAI = ((PLA2−PLA1)·PDEN/10000)·WSFL` (sink-limited, leaf area from main-stem
node number through an allometric power law, *independent of dry matter* — drought there
gives less area for the same mass, i.e. thicker leaves). It applies `WSFL` **not at all** to
`GLAI = GLF·SLA`, the carbon-driven branch, because that dry matter already carries `WSFG`
through `RUE = IRUE·TCFRUE·WSFG`. **Our canopy is only ever that second branch** (`LAI =
leaf_carbon·sla/area`, the "LAI is derived, not stored" lock), and the mapping is
term-for-term: `RUE·WSFG → DDMP → GLF → GLAI` onto `limitation → DMI → FL(DVS) → leaf_C →
LAI`. So our leaf-area drought response is not missing — **it is present, complete, and it
is the response [F] itself specifies for a model of this shape**; adding `WSFL` would
double-count the deficit on the one branch the source deliberately leaves unscaled. The
successor is therefore a **sink-limited leaf-expansion phase** (node accumulator, `PHYL`,
the `PLACON`/`PLAPOW` allometry, a `tuTLM` boundary, and — the expensive part — leaf area as
a STATE variable, since `WSFL` scales an expansion rate and the area it withholds must stay
withheld), **never "the missing `WSFL` multiply"**. ⚠ Applying it to standing `LAI` instead
would make the canopy shrink and regrow with soil water, which is wilting/rolling — a
different, real mechanism and not this citation. Refusal reported as a recommendation, per
this record's standing rule; the verdict is the user's. **FINDING 3 — MEASURE WHAT CATCHES
IT BEFORE BUILDING IT, AND THE ANSWER WAS ALMOST NOTHING.** `WSFD ≡ 1` wherever `WSFG ≡ 1`,
so the whole question is which runs ever see stress. Instrumenting `water_stress_factor` —
the single point every consumer reaches — and replaying each regression module's own
final-state builder on the frozen tree found **exactly two** water-limited runs in the
entire tree: `water_biting` (min `WSFG` 0.1667) and `deep_water` (0.2677). Every frozen
scenario holds `WSFG ≡ 1`; the driest of them, `drought`, bottoms at `FTSW = 0.7039` against
`wssg = 0.30`. Confirmed by running it: **all 7 frozen scenarios bit-identical in every
stock and every aux.** **FINDING 4 — THE FIRST UNFREEZE IN THIS SERIES THAT NOTHING
AUTOMATIC CATCHES.** No new flow (`flow_set` stays 22), no new accumulator (`aux_set` stays
3 — `aux_set` freezes accumulator *classes*, and this changes what an existing one *does*),
no param file, no frozen golden. **The manifest does not move by one byte**, verified by
regenerating it. Both automatic gates are blind; the only gate is
`water_biting_state.json`, which is not in the manifest. `docs/biosphere-reference.md` said
*"an undocumented unfreeze fails CI by construction"* — **that sentence was false and is now
corrected in place**, with this build named as the counter-example. It is a second door into
the room CLAUDE.md's provenance-only warning describes: a **form** change can be invisible to
every gate when its new factor is the multiplicative identity everywhere the freeze looks.
⚠ Regenerating the manifest also surfaced, incidentally, that **the committed manifest is not
byte-reproducible from its own generator** (escaped vs literal em-dashes; equal as parsed
JSON, which is why nothing tests it). Reverted, not committed. **FINDING 5 — THE POSITIVE
FEEDBACK IS REAL, BOUNDED, AND INERT HERE FOR A REASON WORTH NAMING.** `WSFD` speeds DVS →
root extension hits its `DVS ≥ 1` stop earlier → shallower zone → smaller `TTSW` → lower
`FTSW` → larger `WSFD`. Measured rather than reasoned away, including at an absurd
`WSSD = 1.50`: on both live runs the rooted-depth trajectory is untouched and min `FTSW`
does not move, because **root growth had already stopped for a different cited reason long
before anthesis** — day 12 vs 251 on `water_biting` (the dry-subsoil gate), day 107 vs 251
on `deep_water` (the crop's own depth cap, at DVS 0.0442). A statement about these two
scenarios, not a safety property; the bound `1 + WSSD` is what stops it running away.
**SHIPPED:** `phenology.drought_development_factor` + `DroughtDevelopmentParams`;
`ThermalTimeAccumulation` gains a **third** rate multiplier that breaks both patterns its
neighbours set — it **may exceed 1** (drought hastens development; `verfun`/`ppfun` are
`[0,1]` limitation factors) and it is **not phase-gated** ([F] gates it on `CTU > tuEMR`
only and our accumulator starts at emergence, so it runs through grain filling);
`SeasonScenario.wssd: float | None = 0.40`, with `POTATO_SCENARIO` at `None` **because [F]
has no potato row** — an absence in the source, not a preference. The `wssd < −1` rejection
is cited, not defensive: below it development runs backwards, which [F] forbids in the same
words it uses for photoperiod. **THE ONE GOLDEN THAT MOVED**, `water_biting`, splits 14
moved / 6 bit-identical, and the split is the finding: **every water stock AND `rooted_depth`
came through untouched**, because potential transpiration is a Penman–Monteith function of
*weather* rather than of leaf area, so a changed canopy does not change the water draw.
Carbon: leaf −13.6 %, root −7.9 %, stem +4.2 %, **grain +33.2 %** — faster development brings
anthesis sooner, so less of a water-limited season goes into canopy and more into filling
grain. **Drought escape is what `WSSD > 0` encodes**, and it reads as a *benefit* in grain
precisely because the scenario is water-limited. **FINDING 6 — TWO PINS I WROTE WERE
TAUTOLOGICAL, AND ONLY MUTATION FOUND IT.** Seven Python and five Rust mutations were run;
**two Python and two Rust breaks initially passed.** (a) `assert built.ground_area ==
DEFAULT_SCENARIO.ground_area` is `1.0 == 1.0`, so a hardcoded `1.0` in the wiring passed —
the same 1 m² blindness the soil-layers build recorded on the Rust side, re-entered on the
Python side; the pin now constructs a 3.5 m² plot with off-default `EXTR` and `wssg`.
(b) Every `WSFD` test built its aux process **directly**, so wiring the modifier
unconditionally in `plants.py` — the break that hands potato a coefficient [F] declines to
publish — passed the entire file; the pin now walks the registry `build_season` produced.
⚠ **A pin that reads a default asserts nothing.** (c) On the Rust side a season-level pin
asserting only DIRECTION (`accelerated > off`) plus a bound stayed green when `WSFD` was
given the wrong threshold and when it was gated to the vegetative phase — a stressed season
is mostly vegetative, so it still accelerates, just by the wrong amount. **A direction
assertion is not a value assertion, and neither is a bound**; a second Rust pin now calls
`evaluate` directly with an exact expected value, past anthesis. The mutation harness also
**measures** the claim written into those docstrings: with the new pins skipped, **four of
five breaks leave the rest of the Rust suite green**, including replacing Eqn 15.8 with a
constant `1.0` and deleting the multiply outright. **FINDING 7 — a moved probe value with
two different causes on one day.** `test_soil_fractionation` and `test_stem_reserves` both
pin a `water_biting` CO₂ trough: 0.085006 → 0.088509 (the re-basing re-declared the
scenario) → **0.093346** (this build). Every *claim* those tests make was re-measured and
still holds; `sealed_chamber`'s trough moved on neither occasion, which is the control.
**EXIT:** `flow_set` 22 (unchanged), `aux_set` 3 (unchanged), `param_files` 14 (unchanged),
**manifest unchanged**, 1 non-frozen golden, both ports, 2321 Python tests + 21 Rust test
binaries + all 101 cross-port parity checks green, `git diff src/simcore/` empty. **NOT
built, each a named successor:** the sink-limited leaf-expansion phase that `WSFL` attaches
to (finding 2); `WSFN`, the nitrogen deficit factor ([F] Ch. 17 — the fourth of the four,
and the only one still unexamined); runoff and soil evaporation; and making `DROUGHT`
actually bite, still a one-field change and still outside every charge so far.
