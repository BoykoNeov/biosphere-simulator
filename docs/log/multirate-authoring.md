## Bucket 2 (cont.): **multi-rate authoring** — the chosen fix

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE — all 7 steps; the authoring platform is RE-FROZEN with the multi-rate surface in
it** (`schema`: `n_sub` + `rate_class`; `rate_classes: [fast, slow]`; `flow_types`:
`rate_params`), both ports, no golden moved, `src/` untouched.
`docs/plans/post-roadmap-multirate-authoring.md`. An author picks a **coupling cadence**,
not a global `dt`; each flow sub-steps at the `dt` its own rate constant demands. **Step 1**
pinned the fact the phase rested on: `n_sub=1` + an empty slow set reproduces the frozen
ECLSS golden **byte-for-byte** — measured, not inferred. **Step 2** landed the knob
(`ScenarioSpec.n_sub`, `FlowSpec.rate_class`, the disjoint slow/fast registries on
`BuiltScenario`). **Step 3** made it drive: `run_scenario` at master `dt=3600` + `n_sub=60`
lands on the truth (`cabin_o2 == 8.0`, `rationed == 0`) while exporting **hourly** — the
user's charge answered end-to-end; no golden moved, manifest untouched (Step 3 adds no
schema field/integrator/flow type). Rulings worth carrying: the partition check lives in the
**interpreter, not a pydantic validator**, because a *bundle* may contribute `rate_class:
slow` and a schema-level check runs before `apply_includes` would ever see it; the
**manifest regenerates at each step that moves the surface**, never batched to a final
ceremony (the gate is plain equality, so batching disables it for the steps in between); and
**the branch, not the identity, preserves the goldens** — `run_scenario` routes single-rate
scenarios down the pre-multi-rate loop **verbatim**, so the 25 goldens never reach the
driver. That demoted Step 1's identity from *load-bearing* to *corroborating*, and
**superseded its own instruction** to re-point the identity pin through `run_scenario`:
`is_multirate` is false at `n_sub=1`, so the re-pointed test would drive the **single-rate
path** — the wrong path, and a duplicate of `test_authoring_frozen_flows.py`. A leaked
branch would have kept **every golden green** while silently resting all 25 on the identity;
pinned by `test_a_single_rate_scenario_never_touches_the_driver` (monkeypatched to raise).
**Step 4** built the scenario the reference calls **impossible** ("there is no `dt` natural
to both domains") and the sentence is now measured false: `eclss_thermal_habitat.yaml` —
master `dt=3600`, ECLSS fast `n_sub=60`, Thermal slow — runs `rationed == 0` with the cabin
at `o2_eq` and the node warming 102.7→277.4 K. The constraint had **two halves and both are
escaped**: the shared `dt` is *unsafe* (single-rate `dt=3600`: 840 firings, cabin **72.0**
vs truth 8.0 — diverged, not drifted) and the safe shared `dt` is *wasteful* (`dt=60`:
clean, but **20160** Thermal evals for a `τ` of ~65 steps). **The payoff is 30×, NOT the 60×
this plan predicted** — an advisor catch *before* the measurement: 60 is the cadence ratio,
but **Strang steps the slow set at `dt/2`, twice per master step**, so `20160/(336×2) =
30.0`; the missing factor of two is Strang's bill for the order/safety choice, and Lie would
realize 60× at a lower order. Honest whole-run number: wall improves only **2.31×**
(multi-rate saves the *slow* domain's work, and here the slow domain is the cheap one) — so
the win lands where the slow set is expensive, i.e. the biosphere, the domain multi-rate
cannot reach yet. What Step 4 does **not** prove: the two domains share no stock (no
*quantity*, even), so the Strang operators commute exactly and **no coupling fidelity is
exercised** — **forced by the registry, not chosen** (no ECLSS flow carries a heat leg; the
cross-rate boundary and the cross-stock boundary never overlap, since coupling lives
*within* a domain or across *same-timescale* domains). Pinned as an assertion, not a caveat,
so a future coupling registry addition goes red. What **is** new: the **first non-empty slow
set** ever driven through `run_scenario`. A Step-2 ruling also reached further than Step 2
knew: **"the same graph, single-rate" is not `n_sub=1`** — the interpreter refuses `n_sub=1`
with a non-empty slow set, so going single-rate means dropping the `rate_class` keys too
(the refusal's own message says exactly that, on the first author who needed it).
**`simcore.multirate` already exists and is proven** (Phase 0.5) — this is a *consumer*
phase; `git diff src/simcore/` must stay empty (it does; the aux tripwire therefore lives in
`run.py`, multi-rate branch only — and *is testable there*, since `interpret` can't express
aux). Rejected alternatives, priced: the **implicit integrator** (the instinctive "rework
the maths") is the repo's largest unfreeze **and does not meet the bar** — backward Euler
gives 10.61 vs truth 8.33 at `dt=1800`; it buys *sane*, not *near*. **Multi-rate is the
performance enabler, NOT the hazard closer** (advisor): an unsafe **effective sub-step** is
the same hazard one level down. `n_sub=2` at `dt=3600` *does* raise through the harness —
but that is **luck of shape** (the backstop sees the donor-controlled scrubber; `o2_makeup`
stays invisible). **Step 5 CLOSED it** — and its headline is that **this plan's own formula
was the bug**. The plan (and the sentence just above) specified `k·(dt/n_sub) < 1`; that is
right for the **fast** set and **false-PASSES the slow set in the UNSAFE direction**,
because Strang steps the slow set at **`dt/2`** regardless of `n_sub` (measured: scrubber
slow at `dt=3600`/`n_sub=60` — formula reports `k·h = 0.06` ✓, truth is **1.8**, 24
rationings, `cabin_co2` → **0.0**). **It is the same Strang fact that made Step 4's
predicted 60× a measured 30×** — one blind spot (*reasoning about `n_sub` as though it
governed both rate classes*), two wrong claims, one of them a **safety predicate**; the
formula was even written *after* Step 4's catch. Three cases, named once in
`interpreter._effective_step`: single-rate → `dt`, fast → `dt/n_sub`, slow → `dt/2`. The
`dt/2` divisor is **coupled to `run._SPLIT` and pinned, not commented** (advisor) — Lie
steps the slow set at full `dt`, so a flipped split would loosen the check 2× silently.
**The behavior change is real and observable**: the `k·dt` family moved from run-time
`RationedError` to **build-time `AuthoringError`**, which broke **22 committed pins across 6
files** — and *that* is the evidence it works, since those tests could no longer construct
their own subject (`allow_unsafe_step=True` is the study hatch, the `allow_rationing` idiom;
**both are needed and neither implies the other** — one opens the build, one the run).
**Build time is the locus for the PACK, not the convenience** (advisor): a param pack can
inflate a gain past every frozen guard (unit ✓, bound `>0` ✓ — a gain has no
`dt`-independent ceiling), and a pack's values exist only *after* `interpret` resolves them,
so `run_scenario` **structurally cannot see it**; with `o2_makeup` demand-controlled, that
pack would otherwise export an oscillating cabin with `rationed == 0` and no gate
complaining. **A manifest hole nearly shipped**: adding `rate_params` to `FlowTypeSpec`
without adding it to the manifest test's `_flow_types()` derivation leaves the equality gate
**green** while the field is never frozen — an equality gate is blind to a field absent from
*both* sides (the scope-C "provenance-only edit nothing catches" shape); fixed atomically +
given teeth from outside the derivation. Honest scope unchanged: **"the platform catches the
`k·dt` family", never "your dt is safe"** — `radiator_reject` (`τ ≫ dt`; **"≫" is not a
predicate**), `crew_metabolism` (state-dependent) and authored `kinetics` (decision B)
declare `rate_params=()`, a ruling not an oversight, and **neither gate subsumes the
other**. **Step 6 mirrored it into Rust and the port is level** —
schema/registry/interpreter **and the `multirate_step` driver**; `git diff src/` empty, no
golden moved, the **manifest did not move** (it freezes the *Python* surface; Step 6 adds
none). Scope was set by a checked fact — **and the fact was right while the rule drawn from
it was wrong, which Step 6b then fixed**: `eclss_thermal_habitat` appears in zero crossport
files and is Tier-2 `T**4` anyway ⇒ graph-dump-only, so no trajectory vector was minted for
*it*; that quietly became "no multi-rate anchor is possible", which does not follow — the
Tier-2 obstacle is the *radiator*, not multi-rate. The driver was mirrored anyway because
otherwise Rust **builds what it cannot run**, and `run.rs` ignoring `n_sub` would silently
single-rate it at the master cadence — *the same file meaning different things on the two
ports*. **THE FINDING: the mirror carries the RULE but not the RATIONALE.** Step 5's
load-bearing reason for build-time was the **pack** — and **packs are deferred in Rust**, so
`frozen_rate_value` reads the *frozen* `k` (a `Box<dyn Flow>` exposes no params accessor).
Sound only while that deferral holds: **the day packs land in Rust it becomes a false PASS
in the unsafe direction** — the Step-5 shape again, invisible for exactly the flow the check
exists for. **A deferral in one port became a safety precondition in the other**; pinned,
not commented (`pack_deferral_is_what_makes_the_frozen_rate_read_sound`). The precondition's
unique-over-rationing value therefore **narrows in Rust to exactly `eclss.o2_makeup`**.
**THE OPEN DECISION WAS DECIDED BY THE USER — the bridge PASSES the hatch** (against the
implementer's *and* advisor's "refuse"): `build_session_from_file` builds via
`load_scenario_allowing_unsafe_step`, so the documented split stands literally and a player
still watches the cabin die. **The cost, recorded not glossed**: that is now the **one
surface where the `k·dt` family is unguarded**, and for demand-controlled `o2_makeup` an
unsafe gain reaches a session with **every diagnostic clean**. The losing argument was
strong and is preserved: the bridge already maps `AuthoringError` → `SimError::Validation`
**which the UI renders** (so refusing created no *silence*), and *"watch the cabin die" is
factually wrong for what the precondition intercepts* — `k·h` yields a **meaningless** run
(72.0, 9× too much O₂), not a death; the genuine die cases are state-dependent
(`crew_metabolism`, `rate_params=()`) and are **not** refused. A **second, different**
bridge question fell out of the driver mirror: a multi-rate file is now refused parallel to
rk4 (no multi-rate session exists) — **not** on precondition grounds. The **graph dump grew
`n_sub` + rate class before any anchor needs them** (Step-5's lesson applied *forward*: an
equality gate is blind to a field absent from both sides, so a dump omitting the partition
would diff green while the ports lowered *different* partitions); rendered unconditionally,
read off the **built partition** not the spec. The routing branch is pinned **behaviorally**
— Rust has no monkeypatch, so **aux is the observable** (`step_report` advances it,
`multirate_step` never does), which detects the *consequence* rather than the call. **THE
NEAR-MISS (advisor, at the "done" call): the driver I argued hardest to mirror had ZERO
trajectory coverage and the suite was GREEN** — every multi-rate test either built only or
was refused *before* a step ran, so nothing executed `run_multirate`; `panic!("REACHED")` at
its top left everything passing. **Clippy is green as long as *production* calls it —
reachable ≠ exercised**, and a port phase's tests skew to *rejection* cases, which never
reach the happy path. **When the argument for mirroring is "otherwise the port builds what
it cannot run", the test that settles it is the one that RUNS it** — so: `panic!()` at the
top of what you just ported and re-run; if nothing goes red, nothing runs it. Closed by a
bit-identity pin (`n_sub=60` at master `dt=3600` == single-rate `dt=60`, `to_bits()` every
stock) + a non-empty-slow-set run (**the FIRST place `dt/2` is *stepped* rather than
asserted as a constant** — "only" until the compose-gap row below added a second).
**Self-inflicted hazard**: a bare `cargo fmt` reformatted the **whole** Rust tree (rustfmt
1.8 vs an older format), touching frozen simcore — and **there is no CI fmt gate** to want
it. Fully reverted. On a repo with a frozen core a formatter run is **an unreviewed edit to
files the purity invariant forbids touching**: use `rustfmt <file>`, never bare `cargo fmt`.
**The meta-finding: that is the THIRD doc claim this phase falsified** (Step 4's "no `dt`
natural to both domains", Step 5's "still deferred", Step 6's "does not raise —
deliberately"): **a phase that expands what the platform CAN do systematically invalidates
the sentences written when it could not** — so on finishing a step, grep the reference doc
for claims about the limitation just removed, including the sections you did *not* touch.
All three marked in place; Step 7 owns the rewrite. **Step 6b closed the OTHER half of Step
6's charge ("then the cross-port tier") — and the near-miss review had surfaced TWO gaps,
not one**: the driver's zero trajectory coverage (fixed then) and the **absent cross-port
anchor** (fixed now), so "Step 6 COMPLETE" was true of the *mirror* and premature for the
*step*. **THE HOLE, measured not argued: with Rust mutated to lower an all-fast partition —
the two ports meaning different things by the same file — the entire pre-anchor crossport
suite stayed GREEN (33 passed).** Nothing in it was sensitive to a partition because nothing
in it *had* one; the graph dump's inert `n_sub`/rate-class columns were necessary and
nowhere near sufficient (**a field both ports render identically at `n_sub 1` proves nothing
about `n_sub 30`**). The anchor: `eclss_multirate_cabin.yaml` (master `dt=1800`, `n_sub=30`,
`eclss.condenser` slow at `dt/2=900` ⇒ `k·h=0.45`), 15 anchors now. **Pure ECLSS
deliberately — the Tier-2 obstacle was the RADIATOR, not multi-rate**: Tier-1 ⇒ *both* gates
bite, and the second is the one that matters, since **the dump structurally cannot see a
mis-DRIVEN partition, only a mis-RENDERED one** (forcing Rust's driver to `Split::Lie`
leaves the dump green, run red). **The teeth are the SHARED STOCK**: `eclss.cabin_h2o` is
touched by fast `crew_metabolism` + slow `condenser` — the first anchor where the cross-rate
and cross-stock boundaries overlap, so Strang does not commute and dropping one `rate_class:
slow` key moves `cabin_h2o` **2.8387e-02 → 4.0e-02 (~29 %)** with `cabin_o2`/`cabin_co2`
bit-identical. `eclss_thermal_habitat` **could not have done this** (disjoint quantities ⇒
splitting error exactly zero ⇒ its run gate passes whether or not the ports agree) — Step 4
pinned that disjointness; 6b needed its opposite and had to author it. The partition is a
**fixture device, not a sizing claim** (ECLSS's four flows are one order apart; the
condenser is merely slowest — the `param_sets_dsl.yaml` "deliberately nonsense physics"
precedent), and the ~29 % is discretization error, not a better answer: **the gap IS the
signal**. Honest scope: both mutations are *also* caught by Rust's own Step-6 pins (2 red /
1 red), so the anchor is **not** the sole guard — its unique value is comparing the ports
**to each other at all**, so a divergence self-consistent on each side (a default, a
vocabulary, an `includes` interaction) is caught here or nowhere. **`compose.rs`'s
"`rate_class` survives prefixing" was left UNANCHORED** (the file declares no `includes`) —
named then, **CLOSED now** by `two_batteries_multirate.yaml` +
`tests/test_authoring_multirate_compose.py` + 2 Rust pins + a 16th cross-port row. Step 6b's
hole one level down, and **measured the same way**: with `compose.rs` hardcoded to `"fast"`,
the 34 pre-existing Rust authoring tests **all pass**. Here the **dump is the load-bearing
half** — the mirror image of 6b, because a dropped `rate_class` is a mis-**BUILT** partition
(which the dump renders, off `slow_registry` membership) not a mis-**DRIVEN** one. The
drafted rationale was wrong and was corrected by mutation, not argument: *dropping* the
field is unreachable on both ports (Python `model_copy` carries unnamed fields structurally;
a Rust struct literal will not compile without it) — the reachable bug is a **wrong value**
or a **pre-`apply_includes` partition**, both landing on an empty slow set, which at `n_sub
≥ 2` builds **silently**. The 6b lesson was applied rather than repeated: an inline fast
flow drains the SLOW instance's battery so the rate-class and shared-stock boundaries
overlap, else the disjoint instances would commute exactly and the run gate would pass on a
port that dropped the key entirely — but the honest delta is **~1.1e-7** (`k·dt` = 3.6e-5
here vs 6b's 0.45/29 %), live only because Tier-1 is bit-exact. **THE META-FINDING TAKES A
FOURTH INSTANCE, and it is the phase's own**: 6b falsified a claim Step 6 wrote *in this
very document, one section up*. So "grep the reference doc for claims about the limitation
you removed" is not enough — **the claim to re-read is the one you wrote yourself in the
step that removed it**: a scope decision recorded as a fact ("X appears in zero crossport
files") outlives the reasoning that made it right ("…*and it is Tier-2, which is why*"), and
the fact reads like the rule. **Step 7 closed it: the ceremony + the narrative, doc-only** —
and the manifest regenerated **byte-identical**, which is Step 2's "regenerate at each step,
never batch" ruling *confirmed* rather than a formality (a diff here would have meant four
steps of gate-blind drift). Two sweeps, and **the forward one found the hole nothing can
gate**: `n_sub`/`rate_class`/`rate_classes`/`rate_params` were frozen in the manifest,
mirrored on both ports and anchored — and described in **ZERO prose**, with every gate
correctly green, because **`test_frozen_schema_surface_is_complete` equates the manifest
against the live tree and the reference doc is not one of the two sides**. Step 5's lesson
one turn further (blind to a field absent from both sides → blind to anything that is not a
side at all); the manifest's `reference_doc` key is a **pointer, not an assertion** — the
scope-C "provenance, not a gate" shape again. A doc-coverage gate is **named, not built** (a
*mention* is not a *description*; a green light for empty prose is worse than an honest
honor-system half). The backward sweep took the meta-finding to a **FIFTH** instance — *"the
interpreter builds single-rate, no-reset graphs only"*, in a section the phase never
touched, which every prior sweep walked past because they followed the **hazard**, not the
**concept**. **And it was only HALF false**: its "two-rate master-day driver" is the
**biosphere's** daily/annual reset cadence, *not* `multirate_step` — two mechanisms sharing
a word, so "multi-rate is authorable now" would have swapped a stale claim for a **false**
one, i.e. the sweep committing the very failure it exists to prevent. The four `⚠
SUPERSEDED`/`✅ RESOLVED` boxes are **gone, not stacked** — resolving them into present-tense
prose is what reserving the narrative was *for*; a fifth box is the opposite of owning the
rewrite. The discipline's literal *"update the Phase-9 plan"* was **deliberately not
followed**: post-roadmap work updates the reference + its own plan (the Tier-1/`monod`
precedent), and the wording predates post-roadmap plans existing — the same stale-sentence
shape, in the discipline itself
