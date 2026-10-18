# Authoring can reach plants — Step 7 of the 2026-09-29 review

**Opened 2026-10-18** on the user's *"1 and 2"* (Steps 4 and 7 of the review proposal written
2026-09-29). This doc is Step 7: 7a (run what is authored) is BUILT; 7b (a crop bed as one
authoring block) is a design with its open questions put to the user, nothing built.

## 1. 7a — run what is authored: BUILT

**The premise, re-checked on the day: true.** Nothing in `rust/crates` loaded either file in the
repo's `scenarios/` (`algae_habitat.yaml`, `bioregenerative_station.yaml`); the authoring tests read
the fixtures in `rust/data/scenarios/` only.

**The test:** `rust/crates/authoring/tests/authored_content.rs`. It enumerates `scenarios/*.yaml` on
disk (never a hand list; a second test fails if fewer than two are found) and holds each file to what
the platform promises an authored file — no value is pinned, so "authored is not validated" stays
true:

* it **builds** (schema, interpreter, the build-time `k·h < 1` check);
* it **runs to its own horizon** under `run_scenario`'s gates — conservation every step, **no
  rationing**, no demand-controlled flow pointing backwards. Rationing counts as a failure on
  purpose: it is the platform's own default for an authored run, and a clamped stock is an emptied
  one;
* it **repeats**: two runs end bit-identical.

Both files run their full year (8760 steps); the test takes ~7 s.

### What it found on its first run

**`bioregenerative_station.yaml` was broken, not merely stale.** The review plan expected one false
*sentence*; the run found a false *habitat*. On 2026-09-06 the frozen `o2_setpoint` moved from the
design value 10.0 mol to BVAD's 21 % of a 9500 mol cabin, 1995.0 mol. This habitat's cabin started
at 8.0 mol. So for six weeks its oxygen regulator, chasing 1995 mol, drained the whole 500-mol buffer
tank into the cabin every hour: the backstop fired **122,640 times** in the year (14 per hour), and
the run still conserved every quantity. Nothing went red because nothing ran it.

**The fix, sized rather than tuned.** The regulator's steady offset below its setpoint is
`f·q/k_makeup = 0.23725` mol — set by the crew's draw and the rate constant, not by the setpoint.
So the cabin is moved to start 2.0 mol below the new setpoint (1993.0), exactly as 8.0 sat below
10.0, and every other book in the file runs unchanged. Measured on the fixed file: rationed 0, no
reversal, cabin O₂ 1994.76272 after the first hour and 1994.76275 (= 1995 − 0.23725) from the
second on, rising monotonically; the O₂ tank ends at 148.198, the file's own stated "500 → 148.2";
biomass 1000.0 and feed 449.6 (its "5×" and "4.5×"); the thermal node at its stated
Q_eq = 2.99127e9 J.

The file's reasoning was corrected in place (the setpoint, the fixed point, the start, a dated note
on the re-size), plus one more false sentence the read turned up: *"every ECLSS value is a DESIGN
placeholder"*, which two citations (2026-09-06, 2026-10-03) had falsified.

**`algae_habitat.yaml` was sound.** It runs clean and ends at its stated fixed point (food 1900.1,
CO₂ 500.0, biomass 1000.0, feces 100.0, O₂ 2800.0).

**Teeth:** the test is red on the unfixed file (the 122,640 firings above are its message).

## 2. 7b — a crop bed as one authoring block: the design, and the questions

**What it is.** One new building block in the authoring list: *a planted bed of crop X, area A, in
this air, this water, this soil*. Underneath it calls the reference's own plant builder
(`domains::biosphere::system::build_season`) — the science is written once, in Rust, and the
author composes it. Growing the formula language until it can spell photosynthesis is ruled out
(the science would then live in two places).

**What was checked.** The crates allow it without a cycle: `authoring` already depends on `domains`,
which owns the plant builder. The station's assembly (its greenhouse builder and its day driver with
re-sow hooks) lives in `station`, which `authoring` does not depend on today; depending on it would
not create a cycle either.

**Three things the authoring runner cannot do today, each a decision:**

1. **The plant side keeps counters and re-sows.** The plants carry running totals that are not
   matter (development stage, sowing clock) and a yearly re-sow. The authoring runner has no reset
   hook, and its multi-rate path *refuses* any graph with such counters (by design: it would freeze
   them silently). So either the runner learns the station's day driver, or a bed runs on the
   single-rate path only.
2. **Which of the bed's stocks an author may connect.** The bed brings its own air, soil water,
   litter and plants. An author will want to connect the bed's air to their cabin and its water to
   their tanks. Options run from "air and water only" (the rest stays private, so the science's
   assumptions about its own soil and plants hold) to "every stock".
3. **How the two step sizes are declared.** The plants step at 1/16 day; cabin equipment at minutes.
   The station already solves this with its own interleave (review Step 1). An author either
   inherits that fixed schedule or declares it.

**Absorbed, as the review plan said it should be:** the parked **O₂-setpoint authoring decision**
(the third direction plan, B's slice 4). The frozen setpoint silently assumes a 9500 mol cabin and
reaches every authored regulator. That is exactly what broke the habitat above. The honest version
is that an author declares the cabin's air and the regulator reads a mole fraction of it. That is an
authoring grammar change and an unfreeze of its own.

**This is an authoring unfreeze** (the block list grows, the grammar may), so it is built only on
the user's answers.

## 3. The user's answers (2026-10-18)

Asked in plain words with a recommendation each; the user took the **most author freedom** on the
first three and the recommendation on the fourth:

1. **Re-sow: the author writes it.** The file declares its own sowing calendar; the runner carries
   the plants' counters and applies the author's re-sows. (Not the recommendation, which was to reuse
   the station's day driver.)
2. **Wiring: every stock.** An author may connect any of a bed's stocks — air, water, soil, litter,
   plants — to their own. (Not the recommendation, "air and water only".) ⚠ This lets an author build
   arrangements the science was never checked against; "authored is not validated" is what covers it,
   and the authored-content test's promises (conservation, no rationing, repeatable) still apply.
3. **Step sizes: the author declares them**, checked against the existing build-time safety limits.
   (Not the recommendation, "fixed, inherited".)
4. **O₂ target: a share of the author's own cabin air.** The author declares the cabin's air; the
   regulator aims at 21 % of it. A separate authoring unfreeze, built with the crop bed.

The slices and the ceremony follow in §4, written before anything is built.

**A fifth answer, asked after the design was reviewed: the bed is the station's LAMP-LIT chamber**
(lamp light, the chamber's heat store and temperature), the recommendation, on the user's "prefer
closer to reality". Not the weather-lit jar.

**A correction to question 3 as it was put.** It promised that author-declared steps would be
"checked against the existing safety limits". The build-time `k·h < 1` check covers only the
equipment flows that carry rate constants, **not the plant flows**. For a bed, the checks that
actually exist are: whole steps per day, one plant step per group of cabin steps, and rationing as
a run-time failure. The plant science is measured at the 1/16-day step only; another step is
allowed and unexamined. The user was told this before the slices below were written.

## 4. The slices, written before anything is built

Each its own commit, its own predicted diff, full tests `--no-fail-fast`, `--ignored` before done.

1. **The O₂ target as a share — the station half.** `eclss.yaml`'s `o2_setpoint` (1995.0 mol) is
   cited as a mole fraction of 0.21 times the 9500 mol the station's cabin holds. Store the
   fraction, compute the moles where the cabin's air is known. **Predict:** bit-neutral (to be
   verified: `0.21 * 9500.0 == 1995.0` in f64), no golden moves, `eclss.yaml`'s `values`
   fingerprint and its key's name move. First: every reader of `o2_setpoint`, and whether the
   authoring side reads a second copy (the hex-float bundle).
2. **The O₂ target as a share — the authoring half** (an authoring unfreeze). Every authored use of
   `eclss.o2_makeup` must declare its cabin's air; **no default** (a default is the silent
   mis-sizing this removes). The direction gate reads the same per-scenario target the flow aims
   at, tested with a cabin that is not 9500. Every fixture in `rust/data/scenarios/` and both
   content files using the regulator gain the declaration.
3. **A two-rate runner path for files with a bed** — the station's master-day driver, which carries
   the plants' counters and takes a reset hook. Files without a bed keep today's path verbatim,
   proven by a bit-level snapshot of every fixture's and content file's final state before and
   after (the authored-content test pins no values, so it cannot see drift). The step-safety
   check's slow-step divisor is re-examined for a bed file (Strang's `dt/2` is not the plant
   step). `godot_bridge`'s wrap of a loaded scenario either runs a bed file on this path or
   refuses it.
4. **The bed block — the lamp-lit chamber.** One bed per file (the plant flows hard-code their
   stock ids; a second is a load error). "Every stock" wiring = an author stock **becomes** a bed
   stock (the bed's flows cannot be re-pointed, the author's can): quantity and composition must
   match, an aliased author stock may not carry its own `amount:`, all other ids must be
   disjoint. The starting state, counters included, is the reference builder's **verbatim** (a
   hand-rebuilt counter set is how ROOTED_DEPTH once started at 0); the starting-counters gate is
   mirrored for authored beds. Gas exchange on the cabin's minute step where the bed's air is
   shared with cabin equipment (the reference's form since 2026-10-03).
5. **The author's re-sow calendar + one example greenhouse in `scenarios/`.** Re-sow through
   `annual_reset_with` with the params the run was built from (overrides included; the
   `param_funnel` gate's hazard), only on master-day boundaries through the driver's checked reset
   hook. The example's horizon is chosen by measuring its run time (the content test runs every
   file twice). Then the authoring manifest and `docs/authoring-reference.md`.

## 5. Slice 1 BUILT 2026-10-18 — the O₂ target as a share, the station half

`eclss.yaml`'s `o2_setpoint: 1995.0 mol` became `o2_fraction: 0.21 dimensionless` (its source kept,
re-led with what the file now stores). `EclssParams::o2_setpoint_mol(cabin_air_mol)` builds the
target; `O2Makeup::new` takes the cabin's air and has no default. Every station build passes
`station::scenario::STATION_CABIN_AIR_MOL` (9500; `air_split::CABIN_AIR_MOL` now names it); the
standalone ECLSS scenario carries `cabin_air_mol: 9500.0`. The authoring side passes a named,
temporary `flow_registry::IMPLICIT_CABIN_AIR_MOL` (9500) — the defect, named — until slice 2.

**Predicted:** bit-neutral; no golden; the station manifest's `eclss.yaml` entry only.
**Measured:** `regen_goldens` 0 of 20 would change; the biosphere and authoring manifests unchanged;
the station manifest's diff is `param_files["eclss.yaml"]`, both fingerprints (the value and its
source both moved). `0.21 * 9500.0 == 1995.0` exactly, asserted in the loader's test. The loader
bounds the fraction to (0, 1], tested both sides.

**Found, not built:** `humidity_setpoint` (1.7863 kg) carries the same hidden 9500 mol — it is BVAD's
40 % RH *in the station's cabin*. An authored cabin of another size inherits it the same way. It is
not in the user's answer (which named the O₂ target), so it is recorded here, not changed.

**Found, for slice 2:** two fixtures still start their cabin at the long-gone 10.0
(`eclss_multirate_cabin.yaml`, `eclss_thermal_habitat.yaml`, "== the frozen o2_setpoint"), and one
reads `param("o2_setpoint")` inside an authored rate (`param_sets_dsl.yaml`) — so the authoring
param map's key is part of what slice 2 decides.

## 6. Slice 2 BUILT 2026-10-18 — the O₂ target as a share, the authoring half

`FlowSpec` gains `cabin_air_mol` (number or template expression). Required on `eclss.o2_makeup`,
refused on every other flow (frozen or kinetics), no default; a non-positive or non-finite value is
refused. The flow aims at `o2_fraction · cabin_air_mol`, and the direction gate reads the same value
(`flow_registry::demand_setpoint`) — it used to read the station's 1995 mol for every cabin.
`flow_registry::IMPLICIT_CABIN_AIR_MOL` is deleted. Each flow type's spec carries `needs_cabin_air`,
and the authoring manifest records it. The `eclss` kinetics param set exposes `o2_fraction` in place
of `o2_setpoint` (a kinetics rate has no cabin to take a fraction of).

**Files.** `eclss_cabin.yaml` and `scenarios/bioregenerative_station.yaml` declare 9500 (both already
sat at 1995, so they run as before). `eclss_multirate_cabin.yaml` and `eclss_thermal_habitat.yaml`
declare **47.61904761904762**, for which `0.21 × air == 10.0` exactly: the setpoint both files' own
text was designed around ("10.0 → o2_eq 8.0"). From 2026-09-06 until now they had been silently
judged against 1995 — a second and third authored file the frozen move had broken in meaning, found
by making the assumption explicit. `param_sets_dsl.yaml` reads `o2_fraction`.

**Predicted:** the authoring manifest moves by `schema_fields.FlowSpec` (+`cabin_air_mol`) and every
`flow_types` entry (+`needs_cabin_air`); no other manifest; no golden (no golden is authored).
**Measured:** exactly that diff.

**Tests added** (`authoring/tests/scenario_files.rs`): a regulator with no air, zero air or negative
air is refused at build; the field on a scrubber or a kinetics flow is refused; the direction gate
judges a 100-mol cabin against 21 mol (and reports that setpoint), while the station's air leaves the
anchor at its setpoint; a `param(...)` expression for the air builds and runs.

## 7. Slices 3–5 re-ordered before building (advisor review, 2026-10-18)

§4's slices 3 and 4 cannot be tested apart (a runner path for bed files has no bed file to run), and the
riskiest part — pulling the lamp-lit chamber out of the sealed station — deserves its own proof. So:

3. **Extract the chamber (station only).** `build_sealed_station_unread`'s plant chamber (the season, the
   chamber heat store, `Lamp` / `ChamberCooling` / `ChamberWall` / `ChamberHeater`, the latent-heat books,
   the gas-exchange move, the last wrappers) becomes its own builder, and the sealed station is rebuilt on
   it. **Proof:** `regen_goldens` 0 of 20, all three manifests unchanged, the lab separate-air build green.
   **First, measured:** whether the station's hand-built starting counters equal `build_season_with`'s own
   (they are seeded from `params::biosphere()` beside plants built from `room_params()`); if not, the
   difference is a finding, recorded, not fixed inside a bit-identical refactor.
4. **The runner path + the bed block, together.** Configured by the existing `SealedStationScenario` (an
   allow-listed set of its fields), not a parallel struct. Interior stocks (plants, soil, chamber air and
   heat) at fixed ids an author may wire to directly — listed in the manifest; outward connections (the
   lamp's and heater's power, the cooler's heat sink, the walls' surroundings) are wiring fields onto the
   author's own stocks, so a habitat never holds two batteries. The walls' surroundings are required (no
   default). A regulator wired to the bed's O₂ must declare the bed's air. The bed's air stays the
   reference 9500 mol unless the user says otherwise: the humidity target hides that size.
5. **Re-sow calendar + an example greenhouse**, as §4 slice 5.

## 8. Slice 3 BUILT 2026-10-18 — the lamp-lit chamber, extracted

`station::plant_chamber` holds the chamber as parts a habitat assembles: `chamber_plants` (the season
built for a lamp-lit room, its stocks, its plant registry and **the builder's own starting values** plus
the sowing clock), `chamber_start_stocks` (the chamber's heat store at the program's day-0 steady state,
and the walls' boundary when they face the outdoors or a held cabin), `chamber_flows` (lamp, cooler,
walls, heater) and `chamber_latent_heat_flow` (the evaporation coil). Their outward connections are
`ChamberPorts { power, heat_sink, space }`; the sealed station passes `sealed::STATION_PORTS` (battery,
node, `boundary.space`). `sealed_fast_flows` keeps its output and order (the lab separate-air build
reuses it unchanged); `latent_heat::latent_heat_flow` is deleted (one builder now).

**The starting values, measured before the hand-built copy went.** The station seeded the soil account
from `params::biosphere()` beside plants built from `room_params()`. The two differ in one field (the
water's air-temperature read), which seeds nothing, so on the reference params the builder's values and
the copy are the same keys and values. **Under the lab leaf form they were not:** the builder seeds the
seedling's leaf area, the station's copy did not — a lab-only gap, now closed by taking the builder's.

**Proof:** `regen_goldens` 0 of 20 would change; all three manifests unchanged; the full suite and
clippy (below).

## 9. Slice 4 BUILT 2026-10-18 — the runner path and the bed block

**Station side.** `plant_chamber` gained `chamber_fast_forcings` (the lamp's draw, the walls' outdoor
temperature, the setpoint program — the sealed station's fast resolver now takes them from here),
`ResowRule { Maturity, EveryDays(n), Never }` with `chamber_reset_hook` (the sealed station's hook is
`Maturity`), and `assemble_chamber_habitat` — the sealed station's sequence (latent-heat books, gas
exchange on the fast step, last wrappers) around any habitat's stocks and flows, checking that the
habitat's ids are disjoint from the chamber's and that each port names an energy stock. The sealed
station does not call it (its fast registry keeps its own order, byte for byte); the parts are shared.
Proof of the refactor: 0 of 20 goldens; biosphere and station manifests unchanged.

**Authoring side.** `bed:` (schema), `crate::bed` (`bed_scenario`: the sealed station's scenario with
the allow-listed fields applied, `years` derived from the horizon so the weather never clamps;
`bed_stock_ids`; `interpret_bed`), `BuiltScenario.bed: Option<BedRun>`, and `run_bed` on
`advance_one_master_day`. The author's flows are built over the bed's stocks and theirs together, so
"every stock" is a wiring, not a rename. Refused: another crop; a `dt` that does not divide a day; a
horizon that is not whole days; rk4; `n_sub`/`rate_class: slow`; a port naming no stock or a non-energy
stock; `space` given or missing against the surroundings; `every_days: 0`; an author stock with a bed
id; an author forcing with a name the chamber reads; an O₂ regulator on the bed's air declaring any air
but 9500.

**Found while testing, kept as behaviour:** a re-sow draws its seed from the old crop's grain and
harvests it, so a calendar that re-sows a crop with no grain (three days in, or the morning after a
re-sow) is refused by `annual_reset` at that re-sow. The test pins the refusal and, with a seed bank
set, the calendar's timing.

**Slice 5 shrinks** to the example greenhouse: the re-sow calendar landed here.

**A layering decision, made and flagged (2026-10-18).** `simcore/tests/workspace_purity.rs` allowed
`authoring` to depend on `config`, `domains` and `simcore` only, and calls a new edge "an
unfreeze-shaped decision". The bed needs `authoring → station`. Taken: it points from the authoring
layer down to an engine crate — the layering's own direction (the reverse, `station → authoring`, stays
refused; the godot bridge's notes call that the inversion) — and closes no cycle. The alternative,
moving the station's chamber machinery (chamber heat, lamp, sowing clock, gas exchange, latent heat)
down into `domains`, is far larger and riskier. The rule's comment records why; the user was told and
may veto.

## 10. Slice 5 BUILT 2026-10-18 — the example greenhouse

`scenarios/lamp_greenhouse.yaml`: a crew of one (`eclss.crew_metabolism`) breathing the air of a
lamp-lit wheat bed — the first authored habitat with plants in it. The crew takes O₂ from the bed's
`biosphere.o2_pool` and adds CO₂ and water to its `carbon_pool` and `water_vapor`; the scrubber and the
O₂ regulator (declaring the bed's 9500 mol) hold the shared air; a solar array charges the battery the
lamp draws on; the chamber's cooler and the solar waste heat land on a node that radiates to space.
Every boundary paired with a bed pool carries that pool's composition, so each flow balances in carbon
and oxygen.

**Measured.** 14 days (the file's horizon, inside the cold period): rationed 0, no reversal, CO₂ held at
3.000 mol (the scrubber's `P/k` = 0.003/1e-3) and O₂ at 1993.0 (`0.21·9500 − 0.004/2e-3`) — both the
hand arithmetic. 300 days (`bed.rs`, `--ignored`, ~50 s release / ~170 s test build): rationed 0, no
reversal, the crop matures and is re-sown; last sowing at day 280 (the sealed station re-sows on days
139 and 278). The content test now runs three habitats in ~12 s.

**Step 7 is complete:** 7a (run what is authored) and 7b (a crop bed as one block), on the user's five
answers.
