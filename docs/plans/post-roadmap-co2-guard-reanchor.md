# The decade CO₂ guard, re-anchored

**Status: COMPLETE (2026-08-10).** A `liveness_floors` **bound-text + source** unfreeze of
the biosphere manifest. No value, no golden, no param hash, no `src/` change.

The contract question the stem-only re-price left open was *"does `transient=2` fit a tree
whose settling transient the CUE build measured at ~35 years?"* — with the trap stated in
the same breath: `transient=3` clears the floor and `5` clears stationarity, and picking a
window because the subject goes green is the consumer-chamber-2× / DPM-RPM / ruling-B /
fractionation-seed-sweep shape, refused four times.

**It was not answered by choosing a window. It was answered by measuring that the window
does no work.**

The discriminator used throughout: *would this change's justification survive if stem-only
were deleted from the record?* Every claim below does.

---

## What shipped

`tests/test_decade_stability.py::test_decade_min_carbon_pool_stationary`

* the floor's `[_TRANSIENT:]` slice is **removed** — `non_collapsing(summaries, floor=0.05)`;
* `transient=_TRANSIENT` **stays** in the paired `is_stationary` call, deliberately (finding 5);
* the floor is anchored on a **measured attractor** by a new beyond-horizon test, not on a
  15-year reading;
* the guard's stale mechanism comment is replaced by what was measured (finding 3);
* the manifest entry's `bound` drops *"past the sow-in transient"* and its `source` is
  rewritten off the pre-CUE stem-only measurement it carried.

Two new slow tests: `test_the_chamber_co2_trough_has_an_attractor_beyond_the_frozen_horizon`
(the anchor) and `test_the_co2_floor_fires_on_the_buffer_not_on_the_carbon_supply` (the
committed negative result). `tests/test_senescence_form.py`'s mirror of the guard follows
it; `tests/test_biosphere_stress.py` gains one annotation.

---

## FINDING 1 — the window was INERT on the frozen tree, in both assertions

Measured, perennial/Euler at the frozen 15-year horizon:

```
[0.075830, 0.055175, 0.073733, 0.063636, 0.060115, 0.061131, 0.064465, 0.071173,
 0.075611, 0.075121, 0.074335, 0.073871, 0.073602, 0.073451, 0.073367]
```

* whole-run minimum **0.055175** at year 1 = **1.103×** the 0.05 floor;
* **no year of the run dips below the floor**, so `non_collapsing(summaries, 0.05)` and
  `non_collapsing(summaries[2:], 0.05)` are **both True**;
* `is_stationary` is True at `transient` = **0, 1 and 2** alike.

So the slice constrained nothing on the reference and constrained only candidate changes.
**A window that is inert on the frozen tree and load-bearing only on candidates is the one
shape a frozen contract's guard must not have** — it is a knob whose entire effect is on
things the contract is being used to judge.

Removing it is a **strict tightening**: `non_collapsing(whole)` implies
`non_collapsing(sliced)`, so the teeth cannot decrease. That is a proof, not a measurement,
and it is why no "does the new guard still catch what the old one caught" run was needed.

## FINDING 2 — the comment the window carried was describing a tree that no longer exists

The slice was added by the scope-B decomposer calibration under:

> the year-2 CO2 minimum dips to ~0.039 during soil establishment before settling to ~0.055

Both numbers are **pre-humification-split**. On the current tree nothing dips below 0.05 and
the trough settles at **0.0733**, not 0.055. The CUE build restated four guards (the two
decade-stability fixed-point pins, the two `test_biosphere_stress` twins, `sealed_station`'s
biomass gate) and **this one was not among them** — so its assertions kept passing while its
justification rotted, the repo's most-logged shape.

⚠ The sweep was run rather than assumed. Three live sites quoted pre-CUE CO₂ numbers as
present tense: this comment, `test_senescence_form.py`'s docstring (which *validated itself
against* this comment), and `test_biosphere_stress.py`'s `CO2min 0.039` parenthetical. All
three are corrected in place, originals kept where the sentence is explicitly about an
earlier era. Plan-doc tables recording what was measured **then** are left alone — those are
the record, not stale prose.

## FINDING 3 — the guard does not detect what its comment said, and the refutation is the teeth

The comment claimed the guard shows *"closure is not slowly draining the atmosphere into
biomass"*. So the natural teeth check is to starve the loop. **Both obvious levers make the
trough SHALLOWER:**

| mutation | whole-run min CO₂ | vs frozen | floor |
|---|---|---|---|
| frozen | 0.055175 | — | pass |
| `microbial_respiration_rate` ÷2 (the drain mechanism itself) | **0.057797** | higher | pass |
| `chamber_co2_mol0` −20 % | **0.058757** | higher | pass |
| jar ×0.8 at fixed composition | **0.044941** | lower | **FAIL** |
| jar ×0.7 at fixed composition | **0.045871** | lower | **FAIL** (stationarity passes) |

Everything downstream self-limits: less carbon reaching the plant grows a smaller plant,
which draws less. Slowing the recycling rate fourfold moves the trough the *wrong way*.

⇒ this is **not a carbon-supply guard**. It tracks the **buffer against peak demand** — the
same `biosphere.carbon_pool` the acceptance-gate census found binding in all six sealed
scenarios, and the chamber-scale diagnosis's *"the atmosphere is a buffer of hours"* showing
up as a committed assertion. Reached independently for the sixth time.

The negative result is **committed as a test**, not left in a probe: a reader reaching for
the recycling rate to check the guard's teeth gets a green bar and concludes it is toothless.
`docs/retired/mineralization.yaml` exists because a stale negative result suppresses the next
search; a **counterintuitive** one suppresses it hardest.

⚠ The jar-shrink witness matters for one specific reason: at 0.7× the floor fires **while
`is_stationary` passes** — the "clean attractor in the wrong place" failure the level check
exists for. That claim previously rested on stem-only, i.e. on the very change the guard's
verdict was being used to refuse. It now rests on a mutation that is not a candidate science
change at all.

## FINDING 4 — the anchor: a measured attractor, and the worst year is inside the horizon

Run to 50 years: the per-year trough converges to **0.07329124**, flat to six decimals over
the last eight years, = **1.466×** the floor. The floor is now anchored below a *measured*
attractor rather than beside a passing 15-year reading — the CUE build's own idiom for the
`> 0.55` liveness floor, applied to its sibling.

And the claim that licenses keeping the gate at 15 years: **the deepest year of the fifty is
year 1** — inside the frozen horizon. Without this, removing the `[_TRANSIENT:]` slice could
have traded one blind spot for another.

## FINDING 5 — `_TRANSIENT` was NOT touched globally, and the measurement is why

`is_stationary` at `transient` = 0 / 1 / 2:

| pin | 0 | 1 | 2 |
|---|---|---|---|
| perennial min CO₂ | True | True | True |
| perennial peak leaf | **False** | **False** | True |
| consumer peak leaf | **False** | **False** | True |
| consumer year-end carbon | True | True | True |

The window is load-bearing for the two leaf pins and inert for this one. Removing it *here*
too was drafted and **dropped**: this series' binding same-phase diff (0.013618, **90 % of
bound**) sits at index 2 and is **not dropped by `transient=2` anyway**, so removal buys an
*identical* constraint while spending the remaining 10 % of headroom. Inertness justified
removing the slice that was hiding a candidate's failure; nothing hides behind this one, and
`_TRANSIENT` keeps meaning one thing across all four of its uses.

## FINDING 6 — the consequence for stem-only, stated and deliberately not leaned on

Stem-only's verdict is **unchanged**: min CO₂ **0.046065** at year 2, failing the floor
inside *and* outside the removed window, and failing stationarity at fixed indices 2 and 3.

⚠ This is a fact about stem-only, **not evidence the re-anchor is right**. The justification
is finding 1 (inertness on the control) plus finding 3 (an independent teeth witness), and it
stands whether stem-only passes or fails. Re-deciding stem-only inside the commit that moved
its guard is the shape this project refuses; the contract question it poses — *is a deeper
sow-in transient with a healthier attractor a failure?* — is now sharper (the frozen tree's
own transient reaches 1.103× the floor, stem-only's 0.921×) and is still the user's.

---

## The science-gate report (unfreeze discipline step 5)

* `science_bands`: **structurally untouched.** No band is defined on `perennial_long_horizon`;
  `open_season`'s three are unaffected and its golden hash did not move.
* `liveness_floors`: the perennial peak-leaf floor (`0.05`), the converged fixed point
  (`> 0.55`) and the consumer floors all pass unchanged. The one entry that moved is this
  guard's, and it moved by **tightening** — the recorded bound now describes a check over the
  whole run rather than over a window.
* No band or floor was re-tuned to accommodate anything. No bound value changed.

## Verification

* `uv run python tests/test_freeze_manifest.py` ⇒ **exactly 2 lines** of the biosphere
  manifest move (`bound`, `source` of one `liveness_floors` entry). No param hash, no golden
  hash, no `flow_set`/`param_files` membership change. The station manifest is untouched
  (`grep -l` confirmed it names no `perennial_long_horizon` gate).
* **Teeth verified by mutation, not by a green bar:** dropping the `science_gate` marker takes
  `test_frozen_science_gates_are_complete` **and**
  `test_science_gate_bounds_name_a_literal_present_at_their_locus` red.
* `git diff src/` empty. Probes: `M:/claud_projects/temp/co2_guard/`.
