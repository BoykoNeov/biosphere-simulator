# The eight hurdles — a plan from the 2026-09-29 science review

**Written 2026-09-29, on the user's call** (*"write a detailed plan for those 8 steps"*), after
a read-through review of the engine, the plant science, the station assembly, the authoring
layer and the record. **Nothing here is built. Nothing here is decided.** Each step ends with
the decision it needs from the user; no step starts without one.

**Standing of this document.** It is a *proposal*, and it does **not** supersede the third
direction plan, which stays the live forward-looking plan and keeps its re-read gate. Where
this plan touches an item that plan already carries (the humidity coupling, the cabin gas
band, the Bernacchi page check, the authoring decision behind the oxygen setpoint), it says
so and defers to that plan's pricing. When a step here is taken, it earns the normal three
(index line, pointer row, record file) and its own `post-roadmap-*.md` plan with predictions
written first; this document is then struck at that step, not extended.

**How every step is run — the house rules, restated once so the steps need not repeat them:**

- Predictions are written down **before** the run, and graded after.
- A lab measurement comes before any change to the reference.
- A change that moves a frozen value follows the unfreeze ceremony of the contract that owns
  it: advisor review, regenerate as the git-visible record, document.
- A science item and a re-anchoring slice are never taken in one batch.
- A mutation battery runs with `--no-fail-fast`.
- No value is tuned to make a gate pass. A WHAT-IF result stays in the lab.

---

## 0. The order, and why

| Order | Step | Kind | Moves frozen values? | Size |
|---|---|---|---|---|
| 1st | Step 1 — plants and cabin trade gas within the day | science (coupling) | yes — station goldens | medium |
| 2nd | Step 2 — small air volumes against a big step | science (numerics) | measurement first; maybe | medium–large |
| 3rd | Step 3 — plants live in the station's climate | science (coupling) | yes — station goldens | large, three slices |
| any time | Step 4 — cheaper change | tooling (re-anchoring) | hashes only, no value | small–medium |
| any time | Step 7 — authoring can reach plants | platform | authoring manifest | medium |
| any time | Step 8 — stale record clean-up | housekeeping | no (frozen files skipped) | small |
| on demand | Step 5 — sources for numbers | provenance | hash per item | per item |
| after 1 and 3 | Step 6 — checks against the real world | validation | no (lab-only) | medium |

**Why 1 before 3.** Step 1 changes *when* the two halves of the station see each other. Every
later coupling (light, humidity, temperature) rides on that timing, so measuring them first
would measure them on a schedule about to change.

**Why 2 sits between.** Step 1 refills the air between plant steps, which eases the squeeze
in a *station*. It does nothing for a *standalone* sealed chamber, and Step 3's humidity slice
is priced at +21 % water use, which makes fast small stores busier. Step 2's measurement slice
should be read before Step 3 is sized.

**Why 6 waits.** A scorecard against real chambers is only worth reading once the plants
breathe the chamber's air at the chamber's light and temperature. Before that, a mismatch
cannot be told apart from the missing coupling.

**Steps 4, 7 and 8 are independent** of the science thread and of each other. Step 4 is worth
doing early because it lowers the price of every ceremony after it.

---

## Step 1 — plants and cabin trade gas within the day

**TAKEN 2026-09-29, lab slices only** → `docs/plans/post-roadmap-intraday-gas-exchange.md`
(predictions, controls, results). ⚠ Its predictions correct this step's golden count: `lighting`
cannot move (lamp and crop share no stock) and `sealed_energy_drift` does not run on this driver.

### The problem, measured

`rust/crates/station/src/driver.rs`, `advance_one_master_day`: per day, the plant side takes
**all** of its steps (four quarter-days), and only then does the cabin side take its whole
day of small steps. `docs/log/crew-coupled-loop.md` measured the consequence:

- the crew breathes out **327.974 mol C per day**, 86 times the 3.796 mol the cabin air holds;
- the crop draws from the air as it stands at the start of the day, before any of that arrives;
- one crew member's worth of crop (14.09 m²) already asks for 2.23 times the standing air;
- so the loop is 0.023 % closed and the scrubber does the rest.

### The change

Interleave the two sides inside the day. For each of the day's plant steps: one plant step,
then the cabin's share of small steps for the same stretch of time.

```
today:     P P P P  c c c c c c ... (whole day)
proposed:  P c..c   P c..c   P c..c   P c..c      (a quarter of the cabin's steps each)
```

### Slices

1. **Lab first — a second driver, nothing frozen touched.** Add
   `advance_one_master_day_interleaved` beside the existing function, reachable only from a
   lab example. Requires `steps_per_day` to divide evenly by `slow_steps_per_day`; refuse
   otherwise.
2. **Measure on the frozen station scenarios** that carry a greenhouse (`greenhouse`,
   `lighting`, `harvest`, `sealed_station`, `sealed_energy_drift`): every stock, end of run,
   old driver against new.
3. **Measure the case the old driver could not run**: the crew-loop record's 14.09 m² and
   187.45 m² crops, with lamp and power sized as that record's finding 8 did.
4. **Decision point** (below). If adopted: the ceremony on the station contract, the old
   function deleted rather than kept as a switch, goldens regenerated.
5. **A closure readout** as a lab report: fraction of the crew's carbon that passed through the
   crop rather than the scrubber. The record showed `rationed == 0` cannot be that measure.

### Predictions to write before slice 2

- Frozen 1 m² station scenarios move by a small amount (the crop there has 6.31× headroom).
- Cabin-only goldens (`cabin_gas`, `water_recovery`) do not move at all — they have no plant.
- At 14.09 m² the rationing count falls sharply; whether it reaches zero is **not** predicted.
- Peak cabin CO₂ within a day changes shape: the scrubber and the crop now compete.

### Known risks

- **The step counter.** The cabin side runs with the counter frozen; the plant side advances
  it. Interleaved, the cabin sees the counter change four times a day. Any cabin forcing that
  is looked up by step count must be checked. The sealed station feeds the cabin daily-average
  constants, so none is expected — *expected, not measured*.
- **The re-sow hook** is consulted once per day at a plant-step boundary. It stays at the
  start of the day; verify the perennial reset still conserves.
- **Session parity.** `station::session` steps day by day through the same function. It must
  switch in the same commit, or incremental play and a full run stop agreeing.
- **Interleaving is still a splitting.** The crop still sees a quarter-day-old cabin. If slice
  3 shows a quarter-day is too coarse for a large crop, the next move is Step 2's, not a
  finer interleave by hand.

### Done when

The five greenhouse-bearing goldens are regenerated under ceremony, the session parity tests
are green, and the closure readout exists with a number for the frozen station and for the
BVAD-sized one.

### Decision needed

**Adopt interleaving into the reference, or keep it lab-only?** Adoption moves five station
goldens. Recommendation: adopt, if slice 3 shows the large crop stops being starved by the
schedule.

**DECIDED 2026-09-30: ADOPTED** (the user: *"adopt"*). Built the same day as a station unfreeze;
it moved **three** goldens, not five (`lighting` cannot move and `sealed_energy_drift` never runs
on this driver). Record: `docs/log/intraday-gas-exchange-adopted.md`; plan §8 of
`docs/plans/post-roadmap-intraday-gas-exchange.md`. Step 2 is next in this plan's order.

---

## Step 2 — small air volumes against a big step

**Slice 1 TAKEN 2026-09-30, measurement only** → `docs/plans/post-roadmap-draw-census.md`
(predictions, controls, the table). Only CO₂ pools drawn by the crop come near a limit; every
other store is rate × step or the weather. ⚠ It corrects this step's framing twice: the vapour
store's worst draw is set by a cold night, not by the step, and the 7-day station goldens hold
seedlings, so the station crop's pull on the cabin air (0.078) is visible only on the 4-year
sealed station. Slices 2–4 are not started.

**Slices 2 and 3 TAKEN 2026-09-30, lab only** → `docs/plans/post-roadmap-step-options.md`,
record `docs/log/step-options.md`. All three options were built and scored against a 1/256-day
answer. ⚠ **The shipped step leaves the sealed jar's final harvest 7.1 % short** (the open field
2.1 %). Only a finer step moves that (⅛ day −2.5 %, 1/16 −1.4 %). C, in both scopes, prevents
overdrawing but leaves the harvest where it was: the error is not the crop's in-step draw. B is
dominated by uniform refinement at equal cost. Cause not identified. **Slice 4, the decision, is
the user's.** The user's call, 2026-09-30: *"Go with your recommendation"* — find the cause, then
the step. The cause → `docs/plans/post-roadmap-step-cause.md`; it reports back before any
unfreeze. **Measured the same day** (`docs/log/step-cause.md`): the 7.1 % is the light's time
resolution, not the step — a quarter-day step with the light resolved inside the budget flows
reaches the jar's harvest (−0.02 %) at 1.7–4× the shipped cost. That is a new option for slice 4,
but not a dominant one: on the open field and on the jar's worst CO₂ error a 1/16 step does
better (the stepping of the state carries real error there). Slice 4 is still the user's.

**Slice 4 TAKEN AND BUILT 2026-09-30** on the user's *"1 + 3. But ensure the tests runtime would
not quadruple"* → `docs/plans/post-roadmap-step-sixteenth.md`, record `docs/log/step-sixteenth.md`.
The biosphere step is 1/16 day, and in a sealed build the crop's growth reads the CO₂ the step
leaves (C), so it can never take more than the air holds (tested: squeezed jars that ration 205
and 1403 times explicitly ration 0 times). The simulation crates were optimised in test builds
first, so the test suite went from 445–496 s to about 250 s. **Step 2 is done.** What it leaves
open: the lab leaf form no longer rations the jar at 1/16 (a re-measurement, the user's call),
and the mutual-shading loss is now reached only on a pushed run.

### The problem, measured

- The reference sealed jar's tightest step withdraws **0.757** of the CO₂ it starts with
  (`science_gates::margins::the_jars_tightest_co2_step_is_pinned_by_its_headroom`).
- The lab leaf mechanism pushes that to 1.15 and the jar breaks under both stepping methods
  (`docs/log/leaf-rust-remeasure.md`).
- The chamber's humidity was found to be set by the step size, and the vapour store's margin
  is fixed by rate × step (`docs/log/vapour-step-artefact.md`).
- The step was already cut once, 1 day → ¼ day, under a full ceremony.

The pattern: a store that turns over in hours is advanced by a rule that looks once every six
hours. Each new mechanism spends some of what headroom is left.

### Constraint

`simcore` is not edited — the freeze documents give no path for it. Everything below lives in
`domains` or `station`, using what `simcore` already offers (sub-steps that keep the counter).

### Slices

1. **Measure — the draw census.** Extend `readouts::step_draws` into a lab report over the
   whole roster: for every store that can be overdrawn, the worst single-step draw as a
   fraction of what it holds, and the step it happens on. One table. *This slice is the one to
   read before sizing Step 3.*
2. **Price three ways of fixing it, in the lab, on that table:**
   - **A — a uniformly finer step** (⅛ day). Simplest; doubles run time; a ceremony that moves
     every biosphere golden; buys a fixed factor and no more.
   - **B — sub-divide only when needed.** A driver in `domains` that, when the census rule
     says a step would draw more than a set fraction of a store, takes that step as several
     shorter ones. The rule reads only the state, so the run stays repeatable. The open design
     question is the phenology counters, which advance once per full step: they must advance
     by the same total whichever way the step is cut.
   - **C — make the draw self-limiting.** The crop's CO₂ uptake is computed against the air it
     would leave behind, not the air it found. This is a change of form to the science and
     needs a source for the form, or it is WHAT-IF.
     ⚠ **The user, 2026-09-30: *"C — make plants take less as the CO₂ runs low — this should
     be implemented at some point for sure."*** So C is not only a candidate to price: it is
     wanted regardless of which option Step 2 picks for the step itself. The source-or-WHAT-IF
     rule above still decides which side of the line it lands on — find a published form for
     uptake against a depleting pool first; failing that, it is lab-only and tagged.
     ⚠ **Searched 2026-09-30** (`docs/log/co2-uptake-source.md`): the biology is already built
     (FvCB falls to zero at the compensation point); what C adds is inside the step, so C and B
     are both "how the step is taken". Literal C is backward Euler on the crop's flow (lives in
     `domains`). The published candidate found, modified Patankar–Euler (2003, read only via a
     restatement), is a whole-system integrator — probably a `simcore` edit — and does not
     respect the compensation point, so it is not C.
3. **Judge by convergence, not by silence.** For each option: does the result approach what a
   much finer uniform step gives? A quiet safety net is not the test.
4. **Decision point.**

### Predictions to write before slice 1

- The tightest stores are the chamber CO₂ and the vapour store, in the sealed jar.
- The open field has no store under pressure (its air is unlimited).
- No nitrogen or soil-water store is within a factor of two of its limit.

### Known risks

- Option B changes the number of evaluations per day, so **every** affected golden moves even
  where nothing was tight, unless the rule's threshold is never met on that run. That must be
  measured per scenario, not assumed.
- Option B must refuse, not ration, if the shortest allowed sub-step still overdraws.
- Do not narrow a bound or refine the step *until the jar goes quiet* — that is tuning to a
  gate, refused in `docs/log/leaf-expansion.md` finding 9.

### Decision needed

**Which option, if any.** Recommendation: run slices 1–3, then choose. If forced to guess now:
B, because it is the only one that scales with how hard a store is being pulled.

---

## Step 3 — plants live in the station's climate

### The problem

In a sealed station the plants now breathe the station's CO₂ and oxygen. Three other inputs
still come from outside it:

| Input | Where it comes from today | Consequence |
|---|---|---|
| Light | a fixed scenario setting, `lamp_power_w` (`station/src/lighting.rs`) | a power shortage does not dim the crop — measured bit-identical in the crew-loop record |
| Air dryness | the weather file's outdoor figure | plants do not feel the chamber's humidity |
| Temperature | the weather file, or a constant | the station's heat model never reaches the plants |

### Slice 3a — light from delivered power

- **Change:** the light the crop receives is computed from the energy the lamp flow actually
  delivered, not from the scenario's nameplate.
- **How:** the lamp already deposits its light energy into a store (`boundary.light_used`).
  Make the plant's light input a function of what the lamp drew this step. With Step 1 in
  place the plant step and the lamp's steps alternate, so the plant reads the previous
  quarter-day's delivery.
- **Prediction:** on every frozen scenario the battery never runs short, so delivered equals
  nameplate and **no golden moves**. The perturbed brown-out run is where it shows.
- **Risk:** a one-quarter-day lag between lamp and leaf. State it in the record.

### Slice 3b — air dryness from the chamber's own humidity

- Already priced in the third direction plan (the successor named by the vapour items): the
  chamber's own dryness summed over a run is **1.5×** the weather's, potential water use
  **+21 %**. BVAD's crop model reads chamber humidity, a precedent for the form.
- **Change:** in a sealed chamber, the transpiration flow reads dryness computed from the
  vapour store and the air temperature; the open field keeps the weather's figure.
- **Prediction to write first:** which crops, if any, enter water stress at +21 %. The earlier
  "would not notice" was measured at today's rates.
- **Depends on Step 2's census** for the vapour and soil-water stores.

### Slice 3c — temperature from the station's heat model

- The largest of the three, and the least ready. The heat model is one lump with a heat
  capacity, referenced to deep space. Plants need *air* temperature in a *room*.
- **First a design note, not a build:** what stock carries the greenhouse air's heat, what
  moves heat between it and the station's lump, and what the lamp's waste heat and the
  plants' evaporation do to it. Evaporation cools; that term does not exist in the model yet.
- **Then a lab form** with the weather temperature replaced by the room's.
- **Risk:** every temperature-driven process moves at once — development speed, breathing,
  photosynthesis, evaporation. Expect the largest golden movement of any step here.

### Decision needed

**Take 3a, 3b, 3c in that order, or stop after any of them?** Recommendation: 3a and 3b as
builds; 3c as a design note first, decided again once the note exists.

---

## Step 4 — cheaper change

### The problem

- A comment-only edit in a parameter file changes its fingerprint in the manifest, which makes
  it a formal change. So comments known to be false are left in place
  (`senescence.yaml`, `self_discharge.yaml`).
- Eleven reference files contain results of functions like `exp` and `sin`, whose last digit
  differs between Windows and Linux. They can only be regenerated on Windows.

### Slice 4a — fingerprint the meaning, not the text

- **Today:** the fingerprint is taken over the file's text with line endings normalised
  (`config::provenance::normalized_sha256`).
- **Change:** take two fingerprints per parameter file:
  1. **values** — the parsed names, numbers and units, in a fixed order;
  2. **sources** — the `source:` strings, in the same order.
  Comments and layout enter neither.
- **Effect:** a value edit and a citation edit are still formal changes and still visible,
  separately. A comment edit is free.
- **This is itself a re-anchoring ceremony**, once, on all three manifests. No value moves.
- **Control:** after the change, edit a comment → manifest unchanged; edit a source → one
  fingerprint moves; edit a value → the other moves and a golden goes red.
- **Then** correct the known stale comments in one ordinary commit.

### Slice 4b — regeneration off Windows

- **Option 1 — record and live with it.** Already the state. Costs nothing, blocks a Linux-only
  contributor.
- **Option 2 — the project's own `exp`, `sin`, `sqrt`-free maths.** Hand-written versions of
  the handful of functions the science calls, giving identical digits everywhere. Moves all
  eleven files once. Keeps the no-outside-code rule. The cost is writing and proving them.
- **Option 3 — compare those eleven within a stated tolerance everywhere.** Cheapest to build,
  but gives up exact comparison on the platform that has it today.

### Decision needed

**4a: yes or no. 4b: option 1, 2 or 3.** Recommendation: 4a yes; 4b option 1 for now, option 2
only if a second development machine becomes real.

---

## Step 5 — sources for numbers

### The state

- **60** citation-needed markers across **23** parameter files (counted 2026-09-29).
- Every equipment value — scrubber, condenser, oxygen regulator, battery, radiator — is a
  design placeholder. The scrubber rate is recorded as roughly 10–30 times too fast.
- One paper is blocked: Bernacchi et al. (2001), needed to confirm three photosynthesis
  constants.

### The rule already in force, kept

Do not reopen the citation bucket wholesale. Take a number when a finding leans on it, and
open the shelf (`sources/`) before pricing any retrieval as blocked.

### What this plan adds

1. **A triage table, built once**: each of the 60 markers, which readout it moves most, and
   by how much for a ±10 % change. The lab's value switch produces the numbers. This turns
   "60 unknowns" into "the five that matter".
2. **Equipment first among equals after Step 1.** Once the crop competes with the scrubber for
   CO₂ within the day, the scrubber's rate stops being a placeholder nobody feels. BVAD is on
   the shelf and carries equipment figures; read the page images, not the extracted text.
3. **The Bernacchi check** stays exactly as the third direction plan states it: the PDF placed
   in `sources/` by hand, ten minutes, and if a digit differs, record it and move nothing.

### Decision needed

**Build the triage table?** And, from the user only: **the Bernacchi PDF.**

---

## Step 6 — checks against the real world

### The problem

The frozen results prove that nothing changed. The plausibility bands prove that outputs are
believable. Neither compares a run to a measured closed chamber. The plant runs on one weather
year at one site. No scenario on the roster produces drought, so the drought responses are
unexercised (`docs/log/leaf-rust-remeasure.md`, finding 5).

### Slices

1. **A drought scenario in the lab roster.** Same crop, irrigation withheld over a stated
   window. Asserts only that the drought factors fire and that matter is conserved. Lab-only:
   it is an instrument, not a reference scenario.
2. **Choose the comparison data — a shelf-first search, then a list for the user.** Candidates
   are published closed-chamber crop trials with gas exchange records (NASA's Biomass
   Production Chamber wheat and potato trials; the Russian and Chinese closed-habitat
   experiments). The record already names several in `docs/log/chamber-scale.md`. What is
   needed from each: chamber volume, planted area, light level and hours, temperature, CO₂
   setpoint, and measured yield, water use and gas exchange.
3. **A scorecard, lab-only.** For each trial, a scenario set to its stated conditions, and a
   table: measured, simulated, ratio. No pass mark. It is a diagnostic, in the way the PCSE
   comparison is — it informs, it does not gate.
4. **Licensing check** on each data source before any number is committed, per
   `docs/reuse-and-licenses.md`. Published measurements cited to their paper are facts; a
   dataset file may carry its own terms.

### Known risks

- A trial at controlled CO₂ needs a CO₂ controller, which the third direction plan holds as
  "not yet". The scorecard may force that decision.
- A good score can come from two errors cancelling. Report component readouts (light caught,
  carbon fixed, water used), not yield alone.

### Decision needed

**Build slice 1 now; approve the search in slice 2?** Recommendation: yes to both; slice 3
after Steps 1 and 3.

---

## Step 7 — authoring can reach plants

### The problem

- The authoring layer's list of ready-made building blocks has 12 entries: crew, power, heat,
  cabin equipment. **None is a plant flow** (`rust/crates/authoring/src/flow_registry.rs`).
- Its formula language has add, subtract, multiply and one saturation function. No division,
  no exponential, no minimum or maximum. The plant science cannot be written in it.
- No test runs the two files in `scenarios/`. One of them carries a statement about its own
  behaviour that stopped being true on 2026-09-06, and nothing noticed.

### Slice 7a — run what is authored

- A test that loads and runs every file in `scenarios/` for a short horizon and asserts only
  what the platform promises: it builds, matter is conserved, the run is repeatable.
- Not a golden, not a manifest entry. "Authored is not validated" stays true.
- Then correct the false statement in `scenarios/bioregenerative_station.yaml`.

### Slice 7b — a crop bed as one block

- **Not** growing the formula language until it can spell photosynthesis. That would put the
  reference science in two places.
- Instead, one composite building block: *a planted bed of crop X, area A, in this air, this
  water, this soil*. Underneath, it calls the same builder the reference uses
  (`build_season`), pointed at the author's stocks.
- **Open design questions, to settle in the step's own plan:**
  - the plant side has counters that advance with the step, and a yearly re-sow; the
    authoring runner must carry both;
  - which of the bed's stocks the author may wire to their own, and which stay private;
  - how the two step sizes are declared (this is the same interleave as Step 1).
- **This is an authoring unfreeze** (the block list grows), and it should absorb the
  oxygen-setpoint authoring decision the third direction plan has parked, since both change
  what an author can say about a cabin.

### Decision needed

**7a: yes or no. 7b: is an authored greenhouse wanted at all?** Recommendation: 7a yes, now;
7b after Step 1, so the block is built on the final schedule.

---

## Step 8 — stale record clean-up

### The state, counted 2026-09-29

- 648 lines in 87 Rust source files mention Python; 106 of them, in 19 files, are inside
  `simcore`.
- The three contract documents mention Python or `.py` files on 151 lines between them.
- `rust/crates/domains/src/biosphere/mod.rs` says of the step size: *"If the two ever
  disagree, Python is right by definition."* Python was deleted on 2026-08-27.

### What is and is not touched

| Where | Action |
|---|---|
| Comments in `domains`, `station`, `authoring`, `config`, `godot_bridge` | rewrite where a comment states a rule that is now false; leave history that is labelled as history |
| Comments in `simcore` | **not touched** — no path edits that crate |
| Parameter files | **not touched** until Step 4a makes comments free; then corrected |
| Contract documents | pointers to deleted files corrected; dated unfreeze-log entries left as written |
| Dated records in `docs/log/` | **never edited** — they are records |

### Slices

1. **Sort the 542 non-`simcore` lines** into: false rule, harmless history, pointer to a
   deleted file. Only the first and third are edited. The sort is a list in the step's record,
   so the edit can be checked against it.
2. **Edit, comments only.** The check that nothing else changed: the full test suite, and
   `regen_goldens` reporting 0 of 20 would change.
3. **One page per domain, "what this is today"**, generated from the code where possible
   (stock list, flow list, parameter files), so a newcomer has an entry point that is not the
   history.

### Decision needed

**Yes or no.** Recommendation: yes, after Step 4a so parameter files can be included in the
same pass.

---

## What this plan deliberately leaves out

- **The CO₂ setpoint controller** and **the parked leaf mechanism**: both are open decisions in
  the third direction plan and stay there. Step 2 bears on the leaf mechanism (it breaks the
  jar on exactly the squeeze Step 2 measures); that is a reason to read Step 2's census before
  deciding it, not a reason to move the decision here.
- **The product track** (the Godot front-end): dormant, untouched by any step.
- **New crops, new domains**: none is a hurdle; each is work the hurdles make cheaper.

## Test state on the day of writing

`cargo test --workspace --no-fail-fast`, Windows, 2026-09-29, on `ff3b3ee`: **1212 passed,
0 failed, 4 ignored**, across 73 result lines, exit code 0.
