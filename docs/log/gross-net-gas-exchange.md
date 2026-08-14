## **The within-day light path — the plant breathes** (the user's charge; a mechanism that was already in the tree and could not fire)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md).
> The fuller record is the plan doc this row names:
> [`../plans/post-roadmap-gross-net-gas-exchange.md`](../plans/post-roadmap-gross-net-gas-exchange.md).

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**BUILT 2026-08-14 — and the mechanism the charge asked for was already in the tree,
cited and conservation-balanced, unable to fire.** The user, shown that the two-rate
driver delivers the plant's whole day of gas exchange in lumps: *"the plants MUST emit
oxygen at least minute by minute and consume co2 … it is imperative, it is what reality
is"*, and then *"it should be possible for the light Input to vary within the day"*.
**FINDING 1 — THE DEFECT WAS A FORCING THAT MADE AN EXISTING MECHANISM UNREACHABLE.**
`MaintenanceRespiration`'s sealed branch burns biomass, returns CO₂ to the shared pool and
consumes O₂ at PQ = 1 — built in Phase 2, cited, throttled, balanced — and had **never
executed in this project's history**, because it is gated on `shortfall = MRES − GASS > 0`
and a day-averaged PAR makes daily `GASS` exceed `MRES` 20–200× at every step of every
scenario. So the fix is a forcing, not a flow: **no flow, stock or parameter was added or
removed** (`flow_set`, `aux_set`, `param_files`, `dt_days` all unmoved in the manifest
diff). ⚠ **Third instance of the shape** (`canopy-regulator-diagnosed`,
`stem-reserve-form-is-on-the-shelf`): *check whether the mechanism is already in the tree
and merely unreachable before designing a replacement* — here one grep of the enabling
condition would have found it before a design existed, and the plan's first draft had
already specified a new stock, a new flow and a new free parameter to replace it.
**FINDING 2 — ⚠⚠ THE SINUSOID HAS TWO RENDERINGS AND THE PLAN'S IS UNSHIPPABLE.** A
forcing schedule is a function of the integer step and is piecewise-constant across it
(#14), so a within-day path arrives either as the instantaneous value at the step-entry
instant or as the analytic **mean over the step's window**. At the shipped `dt = ¼` the
instantaneous form is *sampling luck*: **0.916** of today's daily carbon at a 13.3 h day,
**1.039** at a 16.5 h day — an 8 % loss and a 4 % gain from the same code — and **exactly
zero** at `dt = 1`, where the single sample lands at midnight. Its golden diff would
record the sampling grid rather than the science. The **window mean conserves the day's
photon dose exactly at any step** (the window means are a partition of one integral) and
converges monotonically to the 60 s answer. ⇒ window mean shipped; the instantaneous form
recorded as **measured-and-refused**, not as a road not taken. **FINDING 3 — ⚠⚠ A FAST
SUB-STEP CANNOT SEE A LIGHT CURVE, WHICH GATES THE REST OF THE CHARGE.**
`Substepper.substep` **keeps `State.n`** and `BoundEnvironment.get` resolves a forcing as
`schedule(snapshot.n, dt)`, so every fast sub-step inside one master step is handed an
identical `(n, dt)` and therefore an identical PAR. The plan's stage 2 — move the gas
flows into the fast registry — would hand the light curve to the one operator structurally
unable to see it. The fix is boundary-side (one resolver per sub-step index, chosen by the
driver) and must **not** be a `Schedule` signature change in `simcore` (the frozen Phase-0
interface; it would turn a boundary unfreeze into a core one). ⚠ **And stage 2 is not the
cheap route the plan priced**: the gas legs are produced by the three flows that *are* the
carbon budget, so "gas fast, growth slow" is not a partition of this tree — minute-rate gas
means 1440 evaluations/day of the most expensive flows against 4 today, while the whole
biosphere at `dt = 1/32` costs **8×** (measured: a 3-year sealed season 0.60 s → 5.92 s).
A finer step for everything is cheaper than a fast lane for the expensive part. **FINDING
4 — ⚠⚠ THE CONTROL: THE CANOPY LOSS IS THE CONCAVITY, NOT THE NIGHT.** The change does two
things at once and they must not be reported as one, so a third forcing was run — a
**top-hat at the daytime mean**, whose daily gross assimilation is the committed tree's and
which differs from it only by having dark steps. `open_season` peak LAI, converged
(`dt = 1/32`): baseline **5.6028**, top-hat **5.5414**, sine **4.7132**. ⇒ night
respiration costs **1.1 %** of the canopy (near-inert, and its apparent *gain* at coarse
steps is a straddling-window artifact that shrinks with the step); the concavity costs
**14.9 %**. ⚠ That is a ~1 % pointwise loss at peak canopy **compounding four-fold**,
because the early-season loss (3.5 %) lands on the phase that *sets* peak canopy
(`wheat-partition-backfill-refused`) — **the plan's own pointwise multiplier was never a
bound on the season, and quoting it as one would have under-priced this change by an order
of magnitude.** **FINDING 5 — THE SCIENCE GATE: THE PEAK-LAI BAND HOLDS AT THE SHIPPED STEP
AND FAILS AT EVERY FINER ONE.** `5.0 < peak LAI < 8.0` reads **5.3806 PASS** at `dt = ¼`
and **4.7132 FAIL** converged, with the observable still moving **15 %** between the two
(the baseline moved 0.6 %). ⚠ Measured with the band's **own** `_run`/`_peak_lai` after an
advisor challenge that the finding was about to be written from a differently-configured
probe — they reproduce it cell for cell, but the check was the point
(`acceptance-gate-diagnosed`). Of the plan's two pre-registered readings one is
**eliminated by measurement**: the canopy regulator is a 5 %/day **loss** above LAI 6, so
it cannot fix a canopy that fell to 4.71. What remains: **the band was clearing against a
diurnally biased assimilation, and what the light path exposes is a canopy this tree cannot
grow.** Recorded as a KNOWN DEVIATION in `docs/biosphere-reference.md`, not tuned. Every
other gate moved *away* from danger (Greenwood margin 2.20 % → 4.75 %; all five CO₂
compensation-point bands clear). **The user's call, taken on the measurements: finish at
the shipped step.** **FINDING 6 — THE CHARGE'S OBSERVABLE, AND IT IS ALREADY CONVERGED.**
Sealed chamber, day 200: CO₂ climbs every dark step and falls every lit one, O₂ its exact
negative at PQ = 1 — and the diurnal swing is **0.010690 mol at `dt = ¼` against 0.010799
at `dt = 1/32`**. ⚠ The season's *canopy* needs a finer step; the season's *breathing* does
not. ⚠ **One of this work's own claims was withdrawn by measurement**: the night branch is
gated on `MRES > GASS`, **not** on a fully dark step — dim dawn and dusk steps cross it too
— so the "133 blind days at `dt = ¼`" count is a lower bound on when the mechanism runs,
not a measure of it. **FINDING 7 — TWO UNGATED PROSE CLAIMS MEASURED FALSE, BOTH ABOUT
THINGS NOTHING RAN.** `lighting.py`'s *"the only runtime consumer of `daylength_s` is
photosynthesis"* named one reader where there were three (the photoperiod-sensitive
phenology path was added three phases later — `o2-makeup-reversal-inside-the-freeze`
exactly), and `MaintenanceRespiration`'s *"at the PP fill `f_O2` ≈ 1"* is **0.854** — with
the control saying it is not ours (**0.847** on the committed tree, so the dip is the
provisioned O₂ fill's). ⚠ **A claim about a branch that never runs is never wrong where
anyone can see it.** **WHAT THE CEREMONY COST**: 13 of 25 goldens (every plant-bearing one;
the eight plant-free ones byte-identical, which is the structural check that the change
reached only what it should), both manifests, both reference docs, the Rust mirror (all 101
cross-port tier tests green **without touching a band**), and **99 red tests re-pinned** —
of which four were not value moves and are recorded as such: the nitrogen-stress bite
**vanished** in both places it was measured (a smaller crop dilutes its N less), stem-only
and Table 7's top row **stopped crossing** the Greenwood tripwire, the stem-only/frozen CO₂
attractor ordering **flipped a second time** (retired as an ordering, asserted as
indistinguishability — a sign that flips on its own carries no information), and the
`lighting` scenario's **binding gate changed stock** (soil water → carbon pool: the lamp's
dark hours are now real). ⚠ **The manifest gained `forcing.light_path`**, a sampled
fingerprint of the day's *shape* and the first **compared** hash in a `forcing` block that
was otherwise provenance-only — the shape can change without touching any file the other
hashes cover, which is the "field absent from both sides ⇒ gate green" blindness recorded
in `multirate-effective-step-is-per-rate-class`. **WHAT IS NOT BUILT, DECLARED**: stage 0
(the time-unit refactor) and stage 2 (rate-classing the gas) — finding 3 says why they are
not preconditions of this half and re-prices stage 2. **So "minute by minute" is NOT
delivered**; what is delivered is a real day/night cycle at quarter-day granularity, with
the diurnal amplitude already converged.
