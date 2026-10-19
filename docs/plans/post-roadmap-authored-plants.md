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

## 11. Checked after the build (advisor's end-of-work review, 2026-10-18)

* **The author-declared plant step, tried.** 8 and 32 steps a day run clean; **24 does not** — the
  lamp's lit share over a 1/24-day step rounds to 1.0000000000000004 and the station's own guard stops
  the run at the first lit step. The bed now refuses a plant step that is not a power of two (a day
  split into a power of two is exact). Pinned in `bed.rs`. **Owed, the user's call:** the real fix is
  in the station's lamp program (`lighting::lamp_light_path`), which every reference build reads — its
  own item, with the goldens' ceremony.
* **The direction gate is daily in a bed file** (`run::run_bed`), where the single-rate path checks
  every step: a regulator pushed past its target by midday photosynthesis and back by evening is not
  seen. Stated in the content test's header and the reference doc. The example is far from it (the
  crew draws ~345 mol O₂ a day against the bed's few).
* **Open, recorded:** `humidity_setpoint` hides the station's 9500 mol (in the third direction plan as
  well); crops other than wheat; a bed air other than 9500.

⚠ **Superseded 2026-10-18 by §12 and §13** (the first two bullets above, on the user's *"work on the
leftovers"*). The bullets are left as written.

## 12. The plant step: the lamp's window capped, any step the driver can interleave (2026-10-18)

**What the overshoot was, measured before anything changed** (an arithmetic probe over the
lamp-window function, 12 to 1440 steps a day, 100 years; kept, with its output, at
`sources/temp-evidence/lamp-probe/` — git-ignored, local, like the rest of `sources/`):

* **Time does not slip.** `(n · dt).trunc()` named the right day on every step at every count tried,
  and the time of day was never more than 6e-12 of a day (half a microsecond) off the exact one. The
  advisor's worry — that a step at midnight would read as the previous day's end and a 24-hour lamp
  would go dark there with no guard firing — **does not occur**: a 24-hour lamp read full light at
  every midnight.
* **The window's mean does.** At every count that is not a power of two, a wholly lit window read
  one rounding step off the lamp: above it (1.0000000000000004 at 24 — what the transpiration's
  lit-share guard refused) or below it (0.9999999999999998 at 3). At 16 and 32, never.
* **A reach limit, recorded, not built:** at 1440 plant steps a day (a plant step of one minute) the
  window check (`t0 + dt ≤ 1 + 1e-12`) starts failing after about 33 years, and the lamp program
  reads such a step as dark. No authored bed comes near it.

**The fix** (`domains::biosphere::light_path::top_hat_window_mean`): a window wholly inside the lit
span returns the lamp's value exactly, and a partly lit one is capped at it. At a power of two both
branches return the value the old line did, bit for bit — `regen_goldens` reports **0 of 20 would
change**. The bed's power-of-two refusal is lifted: any count the day driver can interleave with the
fast steps runs (it refuses a count sharing no factor with them); 0 is refused. Pinned:
`light_path`'s `a_wholly_lit_window_is_the_lamp_exactly_at_any_step` and `bed.rs`'s
`any_plant_step_the_driver_can_interleave_runs` (8, 12, 24, 32 and 48). **Both go red with the old
line restored** (checked). ⚠ A case "24 under a lamp that never goes off" was in the first commit and
was removed (advisor's review): the fixture's three days run on the cold program, whose day length
is `cold.photoperiod_hours`, which the bed's `photoperiod_hours` does not set — so it was the plain
24 run again. The wholly lit window is pinned by the unit test alone.

Not validated: the plant science is measured at 16 steps a day only.

## 13. The direction gate on every state a bed's day commits (2026-10-18)

`station::driver::advance_one_master_day_observed` shows a caller every state the day commits —
after each plant step and each fast sub-step — and does nothing else; `advance_one_master_day` is it
with an observer that does nothing, so every station run is the same operation for operation (and
`regen_goldens` reads 0 of 20). `run::run_bed` now judges each of those states and names the **fast**
step it was seen at — the same unit as the error's "of N steps", where until now it named the plant
step count against the fast total.

**Pinned by a case the old check passed** (`bed.rs`,
`the_direction_gate_sees_a_midday_excursion_a_day_end_check_misses`): the fixture with an O₂
regulator on the bed's air and no crew. The plants' daytime O₂ lifts the pool past the regulator's
target inside the lit day, and by midnight it is back under it. With the check restored to the day's
end the test goes red ("the gate did not see it"; checked).

**Found in review and fixed the same day:** the run's first plant step comes before any fast step, so
it was reported as step 0, which the error text reads as "the file starts the stock above the
setpoint — a wiring error". It is now step 1 (`run::bed_step`, unit-pinned). Unreachable by any file
today — under the cold program the first plant step is dark, so the plants draw O₂ down — but a
wrong diagnosis waiting for a lit first step.

## 14. The three that are left — each a formal change, priced, not built

Each touches a frozen contract, so each waits for the user.

* **Another crop (potato).** (**BUILT 2026-10-18, §17.**) The potato's parameters load (`params::potato()`), but the chamber
  builds its plants from the wheat's (`lighting::room_params`), and the chamber's cold program — eight
  weeks at 4 °C after each sowing — exists to vernalize wheat, which a potato does not need. So it is
  three pieces: the station's builders take the crop's parameters (no value moves for wheat), the cold
  program becomes the crop's, and `bed_crops` grows (an authoring-manifest regeneration). The potato
  itself is authored, not validated (its canopy reads 2.79× low, `docs/log/potato-crop.md`).
  **Medium.**
* **A bed air other than 9500 mol.** (**BUILT 2026-10-18, §16.**) It cannot come first: the condenser's `humidity_setpoint`
  (1.7863 kg) is 40 % humidity of exactly 9500 mol at 22 °C, so a smaller room would be held wetter
  (75 % at about 5070 mol, saturated at about 3800), a larger one drier. It needs the next item, then an `air_mol` field on the bed (an
  authoring unfreeze) and the O₂ regulator's `cabin_air_mol` check reading it.
* **`humidity_setpoint` as a relative humidity.** Store 0.40 and derive the kilograms from the room's
  air and temperature, as the O₂ target became a share (slices 1–2). The derived 1.78632 kg is not the
  written 1.7863, so the station goldens that read the condenser would move (expected water only, in
  the fifth figure — not measured), with the station's unfreeze ceremony. **Small to medium, and it unblocks the air volume.**

## 15. The cabin humidity target as a share — opened 2026-10-18 on the user's *"1"*

The third of §14, taken first because it unblocks the second. Advisor review before any code
(2026-10-18), whose points are folded in below.

**The change.** `eclss.yaml`'s `humidity_setpoint: 1.7863 kg` becomes two stored values:
`humidity_fraction: 0.40 dimensionless` (BVAD Table 4-1's nominal, the quantity the old source
already cited) and `saturation_vapour_pressure: <e_sat(22 °C)> Pa` — the model's own FAO-56 curve
read at BVAD Table 4-73's crew-cabin nominal 295.15 K (the figure's own locus;
`station/params/chamber.yaml` borrows the same number for the plant chamber by analogy), with a test
re-deriving it from the curve. The target in kilograms is built where the cabin's air is known,
`EclssParams::humidity_setpoint_kg(cabin_air_mol)`, and `Condenser::new` takes the cabin's air and
stores its target at construction, as `O2Makeup` does. **Renamed, not re-united:** a reader of the
old key now fails loudly rather than reading 0.40 as kilograms (none was found: no authored rate
reads `param("humidity_setpoint")`, and the authoring manifest records no param-set key names).
`water_cycle.yaml`'s `humidity_setpoint: 0.75` is the plant chamber's, already a fraction, and is
not touched.

**Why a pressure and not the temperature (advisor, second review).** Storing 295.15 K would put an
`exp` into every build of the condenser, and three goldens (`eclss`, `cabin_gas`, `water_recovery`)
are Tier 1 in `rust/data/tiers.json` — `transcendental_free: true`, last-bit agreement on any
platform. Either that claim goes silently false (the cross-platform run that would catch it was
deleted 2026-10-10) or the three rows move to Tier 2 with a measured band — a native-port unfreeze
stacked on a fifth-figure rounding move. Stored as a pressure, the target is
`fraction × e_sat / P_std × air × M_H2O`, multiply and divide only, so `eclss.rs`'s "all arithmetic is
+ − × ÷" and the tier rows stay true untouched. Nothing real is lost: the temperature was going to be a
stored constant either way.

**Half of the trigger, not all of it.** The old source's TRIGGER named two hidden inputs, the 9500 mol
and a cabin temperature the model does not hold. This closes the first. The second stays a stored,
cited constant, not a state: the cabin has no heat store, so the target cannot follow the room. The
new TRIGGER: when the cabin gets a heat store, the pressure becomes a live read of its temperature,
`exp` enters every step, and the three goldens above leave Tier 1 inside that unfreeze.

**Slice A — domains + station.** Every station build passes `STATION_CABIN_AIR_MOL`. The authoring
call site passes a named temporary constant until slice B. Every cabin that starts AT the target
(`STEADY_STATE_SCENARIO`, `CABIN_GAS_SCENARIO`, `eclss_cabin.yaml`, `eclss_multirate_cabin.yaml`,
`eclss_thermal_habitat.yaml`, `scenarios/bioregenerative_station.yaml`) moves to the derived value,
written as a round-trip literal with a test that it equals the derivation — the matched-start rule
(`EclssScenario::cabin_h2o_0`): otherwise each cabin starts about 1.6e-5 kg below its target and the
diff carries a start-up transient that is not this change.

**Predicted before running (slice A).** The goldens that record the cabin's vapour: `eclss_state`,
`cabin_gas_state`, `water_recovery_state`, `greenhouse_state`, `harvest_state`,
`sealed_station_state` — six of the twenty. In each, `cabin_h2o` sits higher by the constant
Δ = derived − 1.7863 ≈ +1.6e-5 kg at every recorded time (target and start shift together, so the
condenser's flux is unchanged up to rounding); every other water stock moves at most in its last
bits; every non-water stock is bit-identical; no other golden moves; `tier_contract` and
`tier_sensitivity` stay green in both crates. Any carbon, oxygen or energy move, or any seventh
golden, is a finding and stops the slice.

**Slice B — authoring (a user question first).** `eclss.condenser` requires `cabin_air_mol`, no
default. Open: `eclss_multirate_cabin.yaml` and `eclss_thermal_habitat.yaml` declare 47.619… mol for
their O₂ regulator (sized so its target is 10 mol). The same air for their condenser drops its target
from ≈ 1.79 to ≈ 0.009 kg; 9500 gives one cabin two sizes. The user's call.

### 15a. Slice A BUILT 2026-10-18 — measured against the prediction

`regen_goldens` (report, then `--write`): **6 of 20 would change — exactly the six named.** Leaf by leaf
against `HEAD`, each of the six differs in **one** number, `eclss.cabin_h2o`, by
+1.542826e-05 kg (= 1.786315428255099 − 1.7863); every other leaf of every golden is bit-identical —
condensate, `crew.water_store`, `recovered_water` and `brine` included, where the prediction allowed
last-bit moves. The shift argument holds exactly: start and target moved together, so `h2o − setpoint`
ran the same bits. Manifests: the station's diff is `eclss.yaml`'s two fingerprints and the six
`golden_sha256` rows; the biosphere and authoring manifests unchanged (the authoring manifest names no
param-set keys, so the `eclss` set's rename does not reach it).

**Follow-up (advisor's end review).** `CABIN_GAS_SCENARIO`'s two cabin starts are hand-written copies of
computed targets, reused by the greenhouse, harvest and sealed builds; now pinned to them
(`scenario.rs`'s `the_cabin_starts_at_its_targets`), so a target move goes red there, not as a start-up
jolt in a golden diff.

### 15b. Slice B's question, measured before it is asked (2026-10-18)

A throwaway probe (deleted) built the two 47.6-mol files with their condenser on that air and their
cabin starting at its target. The condenser behaves exactly as before — the vapour it holds above its
target is unchanged, 2.838710e-2 kg (multi-rate) and 4.000000e-2 kg (thermal) — because only the excess
drives it. What changes is the air: saturation in 47.6 mol at 22 °C is 0.022385 kg, so the cabins end at
**1.668× and 2.187× saturation**, where at 9500 mol they end at 0.406× and 0.409×. Nothing in the model
reads the sum against saturation. The crew's water output is sized for a real cabin; 47.6 mol is about
one cubic metre. The three options put to the user:

* **(a) 47.6 mol for both regulators** — one cabin size per file, the pinned behaviour kept, the air
  supersaturated (unchecked).
* **(b) 9500 mol for the condenser** — the water exactly as today; the file declares two cabin sizes.
* **(c) 9500 mol for both** — one consistent, realistic cabin; the files' O₂ story (10 → 8 mol) becomes
  1995 → 1993, which their headers and a pinned offset describe.

### 15c. Slice B — the user's answer and the build (2026-10-18)

**The user's answer: (c), 9500 mol for both** — one realistic cabin per file. So:

* `eclss.condenser` requires `cabin_air_mol` (its spec's `needs_cabin_air` becomes true), no default, and
  aims at `humidity_fraction` of that air's saturation; `IMPLICIT_CONDENSER_AIR_MOL` is deleted. The
  refusal of the field on other types names both types. The error for a missing air says which target
  is a share of it.
* Every authored condenser declares its air: `eclss_cabin.yaml` and `scenarios/bioregenerative_station.yaml`
  9500 (as their regulators); `eclss_multirate_cabin.yaml` and `eclss_thermal_habitat.yaml` 9500 for both
  regulators, their `cabin_o2` start moving 10.0 → 1995.0 (matched start); the inline fixture in
  `authoring/tests/multirate.rs` 9500.
* The two files' headers re-state their O₂ story (setpoint 1995, `o2_eq` 1993). Their prose figures for
  other cadences (a single-rate run exporting 0.0 mol, a master-step run ending at 72.0) were measured
  on the 10-mol design and are labelled so, not re-measured.

**Predicted before running.** The authoring manifest moves by exactly one value,
`flow_types["eclss.condenser"].needs_cabin_air` false → true; the biosphere and station manifests and all
20 goldens are unchanged (no golden is authored). Every file's water runs the same bits (9500 is the air
the condenser was already built on). In the two re-sized files the O₂ regulator's deviation from its
target runs the same (matched start; ⚠ NOT bit-for-bit, see the measurement below), so the condenser's pinned excess (2.8387096774195486e-2) holds
and every authored-content check stays green.

**Measured (slice B BUILT 2026-10-18).** The authoring manifest moved by exactly the one predicted value
(`flow_types["eclss.condenser"].needs_cabin_air` false → true); the biosphere and station manifests
unchanged; `regen_goldens` 0 of 20 would change. The authoring tests pass, the multi-rate anchor's pinned
condenser excess included, with the two re-sized files. New tests: a condenser with no, zero or negative
air is refused; a 950-mol cabin settles at a tenth of the station's target plus the crew's `P/k`.
**Measured after the advisor's end review — the thermal file was covered by no test.**
`authored_content.rs` sweeps `scenarios/`, not `rust/data/scenarios/`, and only the multi-rate file is
loaded by a test; so a throwaway probe ran both re-sized files, new and as the old 10-mol O₂ design. Both
pass `run_scenario`'s gates (no rationing, no reversal); `cabin_o2` ends at 1993.000000000001; the water
is bit-identical to the old design (excess 2.8387096774195486e-2 and 3.999999999999648e-2). The O₂
deviation from its target differs from the old design's by **9.0e-13** — the prediction's "same bits" was
wrong: 10 → 1995 crosses several powers of two, so `setpoint − cabin_o2` rounds on a coarser grid. Agreement
to rounding, as expected of the shift, not bit equality. **The thermal file still has no permanent test** —
recorded, not added.
**§15 is complete**, and with it §14's third item; the second (a bed air other than 9500 mol) is now
unblocked and still the user's call.

## 16. A bed air other than 9500 mol — BUILT 2026-10-18 on the user's *"1"*

§14's second item, unblocked by §15. An authoring unfreeze; nothing on the reference side moves.

**The design (advisor review before building).** `bed.air_mol`, **optional**, the reference's 9500
when absent — as `lamp_power_w` and `photoperiod_hours` default to the reference's, so
`lamp_greenhouse.yaml` keeps its bytes. Finite and > 0, no lower bound (none measured). The room is
**resized, not relabelled**: `SealedStationScenario::with_chamber_air` scales the starting CO₂ and O₂
by the ratio of the sizes, the inert fill is the rest of the room. That is the separate-air lab's
own resize, moved onto the scenario so the lab (`air_split::split_scenario`) and the bed call one
function; at the present size the ratio is exactly 1.0, so every charge keeps its bits. **The
regulator check grew a second case**: the condenser had the same hole the O₂ regulator was closed
against — since §15b it aims at a share of the air it declares, and a condenser on
`biosphere.water_vapor` declaring 950 on the 9500 bed built. Now an `eclss.o2_makeup` on the bed's
O₂ and an `eclss.condenser` on its vapour must each declare the bed's air.

**Predicted before running.** The authoring manifest moves by exactly one value,
`schema_fields.BedSpec += "air_mol"`; the biosphere and station manifests unchanged; `regen_goldens`
0 of 20; the default bed starts bit-identical to the sealed station's chamber (the existing test,
unedited).

**Measured.** All four as predicted: the manifest's one line, 0 of 20 goldens, the start test green.
New tests (`authoring/tests/bed.rs`): a 3000-mol bed starts at the reference's amounts × 3000/9500
exactly, runs clean, repeats, and ends differently from the 9500 bed; on it a regulator or a
condenser declaring 9500 is refused and one declaring 3000 builds; a condenser declaring 950 on the
default bed is refused; `air_mol` 0, −3000, inf and NaN are refused. Two mutations, each red under
`--no-fail-fast`: the bed ignoring `air_mol` (2 tests), the condenser check removed (1).

**How small rooms behave** (a throwaway probe, the crop-only fixture, deleted after): every size
from BVAD's 1 m² box (27.66 mol, `air_split::BVAD_CHAMBER_AIR_MOL`) to 30000 mol runs 30 days with
nothing rationed and no reversal. The CO₂ climbs, and faster the smaller the room — 399.6 ppm at the
start; after 30 days 17 072 at 27.66 mol, 5 026 at 100, 891 at 950, 449 at 9500: the chamber's
respiration (plants and soil — not separated) outruns the crop's uptake under the cold program's dim
lamp, and a small room has less air to dilute it. The leaf carbon is higher the smaller the room
(0.0551 vs 0.0509 at 9500), consistent with CO₂ fertilisation — not tested. Not a
fault the platform refuses; an author's habitat needs its own scrubber or crew to balance it.

**Left as it was:** the bed's crop (wheat only, §14's first item), and the thermal test file still
has no permanent test (§15c).

**Checked after the build (advisor's end review).** The survey of the room size's readers had been
read through truncated greps; re-run in full. Every fixed 9500 in `station/src` is on the cabin side
(`sealed_fast_flows`' condenser and O₂ regulator, `cabin.rs`, `water.rs`, the station scenario's
start values, the separate-air lab's cabin) — none on a bed's build path, which never calls
`sealed_fast_flows`. Every plant-side reader in `domains` takes the size from the `SeasonScenario` it
is built from; the literals there are in test modules. The causes in the probe paragraph were
worded as findings and were not tested; reworded.

## 17. A potato bed — opened 2026-10-18 on the user's *"1"*

§14's first item. Today the chamber is wheat at **five** places, and a bed that switched only some of
them would build, conserve, never ration and still be part wheat — every check the bed has would pass:

1. the season's params (`plant_chamber::chamber_plants`, `lighting::room_params()`);
2. the re-sow's params (`chamber_reset_hook`) — missed, every re-sow resets a potato with wheat's;
3. the maturity threshold (`SealedStationScenario::pheno`, set to `params::phenology()`), which the
   `Maturity` re-sow reads;
4. the season's crop switches (`vernalization`, `photoperiod`, `wssd`, `stem_reserves` —
   `potato_scenario`'s four);
5. the cold program (eight weeks at 4 °C after each sowing — wheat's vernalization).

**The design (advisor review before writing, 2026-10-18).** One value, `crop`, on
`SealedStationScenario`; the other four follow from it and cannot be set apart from it:
* `pheno` is **removed** as a field and derived (`plant_params().pheno`): it has one reader, and a
  stored copy is a second home that can disagree with the crop.
* `plant_params()` is the ROOM form over the crop's params: `room_params` becomes the room's
  setting applied to a base set (`DailyMean` air), not "wheat with the room's setting" — raw
  `potato()` would read the outdoor air names a room drops.
* `with_crop(crop)` sets the crop, the four switches and the cold program. The switches get one home
  in `domains` that `potato_scenario()` is rebuilt on (bit-identical), so the field crop and the bed's
  cannot drift. It takes a base, so the `SeasonScenario` census (which counts *declarations*,
  `fn name() -> SeasonScenario`) has nothing new to roster; `potato_scenario` stays rostered.
* The potato's cold program: none (`days = 0`) — the cold exists to vernalize wheat, and the
  potato's vernalization fields are wheat's values labelled *"INERT for potato — never read"*.
* `air_split.rs:408` (the separate-air lab's soil account) reads the scenario's `plant_params()`:
  identical for wheat, and a split potato scenario then grows potato throughout.
* `lighting::build_lighting` has its own scenario type and stays wheat.

**Slice 1 — the station side, bit-neutral.** Predicted: `regen_goldens` 0 of 20; all three manifests
byte-identical; the existing "default bed starts as the sealed station's chamber" test green,
unedited. New tests: the wheat scenario's derived phenology is `params::phenology()` exactly; a
potato scenario's params, phenology, switches and cold are the potato's.

**Slice 2 — the authoring unfreeze.** `BED_CROPS += "potato"`; `bed_scenario` calls `with_crop`;
the refusal text, the docs (`bed.rs`, `schema.rs`, `authoring-reference.md`) and the manifest's
`bed_crops` note and value — one manifest regeneration. Tests, written to tell a potato from a
part-wheat bed:
* day 0 has no cold — warm setpoint, full lamp;
* a `Maturity` bed is run past the potato's predicted maturity day (predicted from its
  temperature sums at 22 °C before running): the re-sow fires at the potato's development
  stage 2, not wheat's, and the re-sown crop is a potato (leaf area per leaf carbon reads the
  potato's specific leaf area, 33.33 vs wheat's 22.0);
* builds, runs clean, repeats bit-identically, ends differently from the wheat bed.
Three mutations, each red under `--no-fail-fast`: the re-sow left on wheat's params; the phenology
left as wheat's; the cold kept for the potato.

**Authored, not validated.** The potato's canopy reads 2.79× low against WOFOST
(`docs/log/potato-crop.md`); the 0.60 extinction-coefficient citation is a class that refutes it for
potato (`docs/log/extinction-coef-bound.md`); it has never run in a lamp-lit sealed room. An example
`scenarios/*.yaml` potato habitat is **offered to the user, not built**: every authored file runs its
full horizon in the content sweep.

### 17a. Slice 1 BUILT 2026-10-18 — the crop is one value on the station side

`station::scenario::Crop` (`Wheat`, `Potato`) is a field of `SealedStationScenario`; the `pheno` field
is gone, derived as `pheno()` from `plant_params()` (the crop's params through
`lighting::room_form`, which `room_params()` is now built on). `with_crop(Potato)` sets the crop, the
field potato's switches (`domains::biosphere::potato_switches`, which `potato_scenario()` is now built
on) and `cold.days = 0`. The chamber's season, its re-sow and the separate-air lab's soil account read
`plant_params()`. **Measured as predicted:** `regen_goldens` 0 of 20; no manifest moved; the full
suite green (103 test binaries), clippy clean. Pinned in `station/tests/chamber_crop.rs` (4 tests).

* ⚠ **The test's first draft was wrong, not the code:** the re-sow fires when the development stage
  reads 2, and with the potato's non-integer sums (314.29 + 733.33) that is a few ulps BELOW the sum of
  the sums. Wheat's 1100 + 750 hid it. The threshold is now found from the stage function itself.

### 17b. Slice 2 BUILT 2026-10-18 — the authoring unfreeze

`BED_CROPS` is `["wheat", "potato"]`; `bed_scenario` maps the name to `Crop` and calls `with_crop`;
the refusal text, `bed.rs`/`schema.rs` docs and `authoring-reference.md` (the grammar block, a bullet,
the unfreeze log) say so. **Manifest regenerated, measured as predicted:** `bed_crops` gains `potato`
and its note's text changes — 3 lines in, 2 out; nothing else.

**The maturity day, predicted before the first run.** The chamber holds 22 °C, above the potato's
18 °C cap, so its thermal time grows 18 − 7 = 11 °C·d a day; stage 2 (314.29 + 733.33 = 1047.62) falls
on day 95.2, so the re-sow is at the start of day 96, plant step 1536. **Measured: 1536**, 150 days,
nothing rationed, no reversal (`bed.rs`, `--ignored`, ~40 s).

**Five mutations, each red under `--no-fail-fast`** (station `chamber_crop` + `sowing_clock`,
authoring `bed`, ignored tests included):

| Mutation | Red |
|---|---|
| the re-sow resets with wheat's params | 1 (the station's hook test) |
| the maturity threshold is wheat's | 3 (incl. the 150-day bed run) |
| the potato keeps the cold program | 3 |
| the bed builds wheat whatever it names | 3 |
| the season is built from wheat's params | 3 |

* The first is caught only at the hook, and that is the right place: the plant registry is built once
  from the crop's params, so a re-sow's params reach only the seedling's starting values (its leaf area
  shading the soil), and no bed test pins those. The hook test compares them directly.
* **Offered, not built:** an example `scenarios/*.yaml` potato habitat (every authored file runs its
  full horizon in the content sweep).
  **BUILT 2026-10-19** (the user's *"1"*): `scenarios/potato_greenhouse.yaml`, the wheat example's
  wiring with `crop: potato` and its own header. ⚠ **The one thing it re-checked:** the wheat file's
  150 W solar line was set where its 14 days run under the cold program's dim lamp; a potato bed has no
  cold program, so the full lamp meets it from day 1. Measured: the battery ends 14 days at 2.0007e10 J
  and 150 days at 1.9937e10 J (−0.3 % from 2.0e10), nothing rationed, no reversal, re-sown at plant
  step 1536. CO₂ 3.00 mol and O₂ 1993.0 as in the wheat file. A slow test pins the season
  (`bed.rs`, `the_potato_example_resows_and_holds_its_battery_over_a_season`, ~40 s, `--ignored`); the
  content sweep now runs four habitats in ~13 s.
