## **Potato stage 2 — the Rust habitat mirror** (the deferred half, taken)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md).
> The plan of record is `docs/plans/post-roadmap-potato-crop.md`; stage 1's outcome is
> [`potato-crop.md`](potato-crop.md), whose own title reads *"(stage 1 of 2)"*.

**COMPLETE 2026-09-06.** The second species is now something the **reference** can run.
Stage 1 (2026-08-11) built the crop in Python and measured it against an offline WOFOST
oracle; the reference flip inverted which language spells the reference and S6 deleted the
Python checker, so **that half no longer exists** — `params::potato` + `potato_scenario` are
the potato's only live form, and `rust/crates/domains/tests/potato_crop.rs` (10 tests) is its
only gate. The four `crops/potato/*.yaml` files had crossed with slice S1 and been read by
nothing for three weeks.

**The seam is the one that was already there.** No new assembly path, no crop branch inside
`build_season`: `build_season_with(scenario, &params)` is the value-switch seam built for the
lab, and a crop is exactly what it takes. So the whole build is *a params constructor and a
scenario constructor*, and `tests/param_funnel.rs` keeps it honest — `potato` joins the loader
roster that gate derives from the tree, so a spine module reaching for it goes red. **A second
crop is a caller's choice, never a branch inside the assembly.**

**A crop is the plant-side param set, partitioned.** Overridden: `allocation`, `canopy`,
`phenology`, `root_depth`. Shared: photosynthesis, respiration, transpiration, senescence,
stem reserves, nitrogen, decomposition, microbial respiration, humification, water cycle,
herbivory — each pinned bit-identical to the reference crop's, so *"potato shares wheat's
photosynthesis"* is an assertion rather than a comment.

## THE PIN HAD TO BE FIELD-LEVEL, AND READING THE FILES IS WHAT SHOWED IT

The obvious shape — *overridden file ⇒ every value differs, shared file ⇒ every value
identical* — is **red or vacuous on this crop**. Potato's `canopy.yaml` overrides the *file*
but carries wheat's `extinction_coef` (0.6) and `carbon_fraction` (0.45) **unchanged and on
purpose**, each labelled *"SHARED, not potato"* in its own `source:` string; only
`specific_leaf_area` moves (33.33 vs 22.0). A file-level assertion would have failed on those
two, and the repair a hurried session would reach for is to weaken it until it asserts nothing.
⚠ **And the `identical` half is the load-bearing one** — *"an override differs"* is what a
reader already assumes; *"this override carries the reference value on purpose"* is the claim
that rots silently, when someone later "fixes" a shared placeholder into a fabricated
species-specific number.

⚠ **One of those fields is unreadable from the structs.** `CanopyParams` keeps only
`sla_per_mol_c`, with `carbon_fraction` already folded into it (`sla · M_C / carbon_fraction`),
so the claim stage 1 flagged — *`carbon_fraction` agreement now spans a crop boundary*, because
potato overrides `canopy.yaml` and does **not** override `nitrogen.yaml` — cannot be checked at
the struct level at all. It is checked against the file text, through `config::ParamFile`.

## "INERT FOR POTATO" IS A CLAIM ABOUT THE SCENARIO, NOT ABOUT THE FILE

Potato's `phenology.yaml` carries eight vernalization/photoperiod fields at wheat's values,
each sourced *"INERT for potato — never read."* That is true only because `potato_scenario`
sets `vernalization: false` and `photoperiod: false` — **a scenario fact wearing a parameter
file's clothes**, and this repo's standing lesson is that a claim of inertness needs a ladder.

Two decisions follow, and both are the opposite of the cheap one:

* **All three phenology structs are loaded from the POTATO file** (`pheno`, `vern`,
  `photoperiod`), not just the first. Loading `vern`/`photoperiod` from wheat's file would make
  those eight source lines decoration — the fields would be unread because nothing pointed at
  them, rather than because the crop is day-neutral. Read this way the sentence is falsifiable.
* **The mutations are absurd, not marginal**, and the assertion is on bits: `vsen` at 100× the
  reference, `cpp` outside any real day. Plus **two controls**, because a silence proves
  nothing on its own: `t_base` (a field the same file's thermal-time loader reads every step)
  must move the run, and each mutated params object must differ from the unmutated one — that
  second one is what separates *"inert in the run"* from *"the seam forgot to reload `vern`"*,
  and without it the test would have passed just as happily while measuring nothing.

## THE CEREMONY THAT WAS NOT OWED — AND THE ONE SENTENCE THAT DIED

The census docstring in `params.rs` gave the reason the four overrides are excluded as
*"the port has no potato build (its stage 2 is deferred), so it loads none of them."* **This
build kills that sentence**, and it is a *freeze contract's own stated exclusion rationale*,
which is exactly the shape that costs an unfreeze ceremony (advisor review → regeneration →
documentation) in this repo.

⚠ **It did not, and the check was one grep.** The sentence lives only in Rust doc comments
(`params.rs` twice, `system.rs` once). The manifest's own `_authority` prose says only *"two
different exclusion reasons (four `crops/potato` overrides by non-recursion, `demo.yaml` by
name)"* — which stays true. No manifest byte asserts the retired reason, so the correction
rides free in the build commit. **The lesson is the pricing, not the outcome:** *check which
side of the freeze a stale sentence is actually on before paying for a ceremony*, because the
cost of guessing wrong is a whole regeneration nobody needed.

**What replaces it is the rule that was always doing the work.** `param_files` is the set the
reference loads **as the frozen reference crop**; non-recursion is the *mechanism* of the
exclusion and *authored ≠ validated* is its *reason*. Freezing a set the project simultaneously
calls unvalidated would be incoherent, and a test now says so: the four embedded texts must not
appear in the census, compared as **texts** rather than basenames, because all four basenames
collide with frozen ones and a name check would pass while looking at the wrong file.

## AN EMBED DOES NOT ENTER THE CENSUS — AND THE FALSE BELIEF WAS IN THE TEST THAT AVOIDED IT

`params.rs`'s own `potato_overrides_the_rooting_habit_rather_than_sharing_wheats` read its
override **off disk**, through a `std::fs::read_to_string` and a `Box::leak`, on the stated
grounds that an `include_str!` *"would quietly add a file to `param_files()` and therefore to
the freeze manifest."*

**It would not.** `param_files` is a hand-written list; nothing in the tree follows an embed
into it. The avoidance was real work — a filesystem read and a leak in a unit test — spent on a
hazard that did not exist, and it had stood since slice C8. This commit adds **four** embeds and
the manifest is byte-identical, which is the measurement rather than the argument. The test now
uses the const and the belief is retired in place.

*The general shape, and it is not new here: a hazard avoided is never measured, so the belief
that motivated the avoidance is the one nothing ever checks.*

## POTATO IS THE FIRST SCENARIO IN THE RUST TREE TO DECLINE STEM RESERVES

Measured before it was written: `stem_reserves: false` appeared **nowhere** in `rust/crates/`,
and `wssd: None` only inside one unit test's inline scenario. Every one of `potato_scenario`'s
four switches is a **declined** modifier rather than a tuned value — day-neutral by [E] Table
12's own "–" legend, no `WSSD` because [F] Table 15.1 has no potato row, no stem reserves
because [E] Table 7 gives potato a *range* (0.2–0.4) where wheat gets a single 0.4 and picking
inside someone else's range is our number wearing their name.

So this build is also **that branch's first exercise by a run**, and the gate asserts the
absence rather than the flag: `StemRemobilization` is in the reference crop's type set, absent
from potato's, back when the flag is flipped on the same params, and the potato build's type set
is a **subset** of the reference crop's — it only ever subtracts.

⚠ **Deliberately not a fifth canonical build.** `freeze_manifest::inventory` unions its
`flow_set`/`aux_set` over `DEFAULT_SCENARIO` and the three chambers; `potato_scenario` is not
added to that union and must not be. A crop that is authored-not-validated has no business
widening a frozen contract.

## THE ONE THING THAT WENT RED, AND IT WAS A GATE TWO CRATES AWAY

The build was written against the biosphere crate, and the first full-workspace run turned
**`station`** red: `scenario::water_geometry_tests::the_scenario_roster_matches_what_the_source_declares`.
That census walks every `.rs` file under `rust/crates/*/src`, finds every `SeasonScenario`
declaration in both crates' production halves, and demands the roster name all of them —
because `domains` cannot see `station`, so `station` is the only place both halves are
visible. `potato_scenario` appeared in the scan and not in the roster, exactly as designed.

⚠ **The reason it is owed matters more than the fix.** `potato_scenario` inherits its entire
plot from `DEFAULT_SCENARIO` and overrides only crop switches, so both soil-water identities
([F] Eqns 14.26–14.28) hold **by inheritance** — and that file's own header records why that
is not a reason to skip it: *correct-by-inheritance is not covered, it is untested*, the
lesson a `harvest` scenario paid for when it overrode rooting depth while inheriting the
shallow zone's water and nothing went red until a golden moved.

⚠ **And it nearly did not get read.** The first full run was piped to `tail`, which swallowed
cargo's exit code — the harness reported success while the tail itself ended
`error: 1 target failed: -p station --lib`. `--no-fail-fast` did its job and the run continued
past the red; the *pipe* is what hid it. This repo's standing warning is that a truncated
battery reports fewer reds than there are; **a piped one can report none at all.** The re-run
captures the exit status explicitly.

## The gap note that was true, closed rather than deleted

`system.rs`'s `the_wiring_declines_the_drought_modifier_when_no_wssd_is_cited` carried
*"⚠ Potato has no Rust successor and that is a GAP, not a decision."* It was accurate the day it
was written and this build closes it. The note is **rewritten to say so**, not removed: the test
keeps the portable half (the rule that `wssd: None` declines the modifier, on an off-default
plot no scenario in the tree supplies) and hands the crop-specific half to `tests/potato_crop.rs`.

## What did NOT move

**Nothing frozen.** 7 biosphere + 13 station goldens byte-identical, both manifests
byte-identical, the census still 15 files, `docs/*-reference.md` untouched. The whole build is
additive: four `include_str!`s, one params constructor, one scenario constructor, one test file.

⚠ **And this crop is still not validated.** It is wired into no golden and named in no manifest.
What is gated is that it *resolves as a set*, *reaches the run*, and *conserves mass, runs
deterministically and is never rationed* — including in the sealed chamber, which is the trap
stage 1 flagged (a different canopy and a different partition table in a jar sized for the
reference crop; `rationed == 0` says the backstop never had to intervene). The measured
disagreements with WOFOST — first tuber carbon on day 7 against day 46, peak LAI 2.79× low —
are stage 1's record and remain a **finding, not a calibration target**.

## Honest residuals

* The **oracle is gone with Python**, so stage 1's diagnostic table cannot be re-run. It is a
  dated measurement of a crop whose numbers have not moved, not a live check.
* `extinction_coef` is still carried from wheat and still recorded as *plausibly biased LOW*
  for a broad-leaved planophile canopy (~0.8–1.0 vs 0.6). Unchanged by this build, and the
  field-level pin now asserts the carry-over rather than hiding it.
* The **four-year-old warm-window over-run** stands: our cardinal-cap form holds flat above 18
  °C where [E]'s response declines, so warm weather over-accumulates development. A model-form
  gap, recorded in the file, never closed by moving the value.
* `potato_with` leaks one short string per mutation (`Box::leak`), because the `*_from` seams
  take a `&'static str`. Bounded, test-only, and the alternative — hand-building the struct —
  is precisely what would bypass the schema, unit guard and bounds the seam exists to keep in
  the path.

`docs/plans/post-roadmap-potato-crop.md`
