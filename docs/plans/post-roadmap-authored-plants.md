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
