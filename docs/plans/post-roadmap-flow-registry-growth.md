# Post-roadmap: growing the author-selectable flow registry (Tier 1)

**Status: COMPLETE** (2026-07-16). An **unfreeze** of the authoring platform, chosen by
the user. Sequenced first of three; see "The sequence" at the foot. The outcome block is
at the foot of this file — **read it before the plan body**, because four of the plan's
assumptions did not survive contact and the record of *how* is the useful part.

## The charge

Make the frozen Power / Thermal / ECLSS science **author-selectable**, so a scenario file
can compose it instead of inventing it. The roadmap's closing promise is that a scenario
defines "a habitat with its power budget, thermal limits, crew size, and ecosystem"; today
the registry exposes **three crew flows** and nothing else, so three of those four are
unreachable from an authored file.

`flow_registry.py`'s own docstring records the debt: *"Step 0 registers the standalone Crew
flows (the composition anchor). Later steps grow this to the rest of the frozen flow set."*
That never happened. This plan does it for the flows that fit; the biosphere does not fit
(see "Not in scope").

## The finding that reframes this plan — and the justification it kills

**55 of the project's 57 parameters are uncited placeholders.** Measured, not estimated:

| domain | uncited / total params |
|---|---|
| biosphere | 45 / 45 |
| power | 2 / 2 |
| thermal | 4 / 4 |
| eclss | 4 / 4 |
| **crew** | **0 / 2** — calibrated to NASA BVAD Table 3-31 (Phase 6, Step 9) |

Each carries `source: "TODO(cite) — provisional, literature-typical … pending validation
gate"`. The *equations* are cited primary literature (FvCB is Farquhar et al. 1980;
Stefan-Boltzmann is Incropera; the ECLSS loops are Seader/Ogata). The *numbers* are
plausible placeholders. **Structure is literature-derived; values are not calibrated.** The
one validation gate that would fix this — the Phase-1 quantitative oracle match — was
deferred by user decision and never ran; its tests are opt-in (`-m oracle`, needing the
PCSE dep group and network) and absent from the default gate.

Two consequences, both load-bearing:

1. **The "these flows are on firmer footing than the biosphere" argument for this plan is
   false and is hereby withdrawn.** Power/Thermal/ECLSS are *exactly* as uncited as the
   biosphere. The justification for Tier 1 is reuse and reach, not calibration.
2. **The "registering frozen flows would make the UNCALIBRATED marker lie" objection is
   rejected — it proves too much.** As a gate it would block *all* registration, including
   the three crew flows already registered, and would make this plan wait on bucket 3,
   contradicting the chosen sequence. It is dropped as a criterion. Registering these flows
   exposes *the same placeholder science the frozen station already runs on*; it degrades
   nothing that is not already so.

What survives is a **documentation** obligation, not a gate — see Step 5.

## The two axes (why the marker is not the issue)

`has_authored_kinetics` measures **who wrote the rate law** (authored vs frozen). It has
never measured **whether the science is validated**. Those are independent axes:

| axis | asks | fixed by |
|---|---|---|
| **shape** of the law | is the functional form right? | the registry (this plan) / the grammar (Tier 2) |
| **values** in the law | are the numbers right? | validation (bucket 3) |

The marker is accurate about its own axis. It is simply silent on the other one, and always
has been. A signal for the *values* axis is a **bucket-3 deliverable**: it cannot
discriminate until there is something to discriminate — today it would report "uncalibrated"
for everything except crew, which one sentence of prose already says. Deliberately not built
here (see "Not in scope").

## What Tier 1 registers

**Nine flow types.** Every one already matches the registry's shape —
`(id, priority, *wiring[, params])`, with `evaluate` reading only `snapshot`, `env`, `dt`.
No new logic, no `FlowTypeSpec` change, no interpreter change, no grammar change.

| authoring type name | cls | wiring_fields | param_set |
|---|---|---|---|
| `power.solar_charge` | `SolarCharge` | `solar_source`, `battery`, `waste_heat` | `charge` |
| `power.load_draw` | `LoadDraw` | `battery`, `waste_heat` | — |
| `power.self_discharge` | `SelfDischarge` | `battery`, `waste_heat` | `self_discharge` |
| `thermal.heat_input` | `HeatInput` | `heat_source`, `node` | — |
| `thermal.radiator_reject` | `RadiatorReject` | `node`, `space` | `thermal` |
| `eclss.crew_metabolism` | `CrewMetabolism` | `cabin_o2`, `cabin_co2`, `cabin_h2o`, `metabolic_o2_sink`, `metabolic_co2_source`, `metabolic_h2o_source` | — |
| `eclss.co2_scrubber` | `CO2Scrubber` | `cabin_co2`, `co2_removed` | `eclss` |
| `eclss.condenser` | `Condenser` | `cabin_h2o`, `humidity_condensate` | `eclss` |
| `eclss.o2_makeup` | `O2Makeup` | `o2_supply`, `cabin_o2` | `eclss` |

**Three param loaders.** `self_discharge` is already registered (Step-2's authored-kinetics
anchor); `crew` is already registered.

| name | loader fn | default file |
|---|---|---|
| `charge` | `load_charge_params` | `power/params/charge.yaml` |
| `thermal` | `load_thermal_params` | `thermal/params/radiator.yaml` |
| `eclss` | `load_eclss_params` | `eclss/params/eclss.yaml` |

## Hazards — verified, and each needs a plan step

1. **Forcing-binding: the frozen forced flows read hardcoded module constants, not wiring.**
   `SolarCharge`→`solar_power`, `LoadDraw`→`load_power`, `HeatInput`→`heat_load`,
   `CrewMetabolism`→`o2_consumption`/`co2_production`/`h2o_production`. This is the
   documented *"a forcing-bound frozen bundle cannot be prefixed"* boundary, which crew
   already hit. **Single-instance authoring works** (declare a forcing under the exact
   hardcoded name); **prefixed multi-instance is out** (forcing-name collision). Must be
   pinned by an anchor and stated in the reference doc — not discovered by an author.

2. **The frozen params carry implicit `dt` assumptions, and an author picks `dt`.** This is
   the sharpest hazard and it is new to authoring. ECLSS's `co2_scrub_rate = 1.0e-3 /s` is
   sized for `dt = 60 s` (`k·dt = 0.06 ≪ 1`); an author selecting `eclss.co2_scrubber` and
   running at `dt = 3600` gets `k·dt = 3.6 > 1` — the donor-controlled draw exceeds the
   stock and the Euler backstop fires or a stock goes negative. Thermal's `heat_capacity` is
   likewise sized so `τ ≈ 65` steps at `dt = 3600`. **The frozen flow is correct; the
   authored `dt` makes it wrong.** This is exactly the failure the algae-habitat design
   arithmetic caught (`post-roadmap-authored-habitat.md`), now reachable by an author who
   wrote no kinetics at all. Anchors must pin a safe `dt`; the reference doc must state the
   per-flow constraint.

3. **`RadiatorReject` is nonlinear (`T⁴`) and donor-controlled**, so RK4 ≢ Euler for it (a
   tolerance agreement, not bit-identity). The cross-port anchor must be tiered accordingly
   rather than assuming exactness.

## The steps

Each follows the unfreeze discipline in `docs/authoring-reference.md` ("The unfreeze
discipline", steps 1–6). `git diff src/simcore/` **must stay empty**; `git diff` of the
domain trees must stay empty (this plan adds no science and changes no frozen flow).

- **Step 1 — Register, Python side.** Add the 9 `FlowTypeSpec` entries + 3 `PARAM_LOADERS`
  entries to `src/authoring/flow_registry.py`. Mirror the frozen constructor signatures
  exactly. Confirm the completeness gate (`tests/test_authoring_freeze_manifest.py`) **fails**
  before the manifest is regenerated — that gate owns "added to the tree, exercised by
  nothing", and it should bite here. If it does not, the gate has a hole and that is a
  finding.
- **Step 2 — Register, Rust side.** The same 9 + 3 in
  `rust/crates/authoring/src/flow_registry.rs`. Phase 7 already ported these domains, so the
  classes exist; this is registry entries only. A Python-only change is a broken contract,
  not a half-done one.
- **Step 3 — Anchors.** New scenario fixtures under `tests/authoring/scenarios/` exercising
  each new type, wired single-instance with the hardcoded forcing names, at a `dt` inside
  the params' safe range. These are cross-port anchors: they must run identically on both
  ports under the tier contract (`docs/native-port-reference.md`). Include a **teeth** case
  proving the wiring-field check rejects a misnamed field, per the registry's existing
  discipline.
- **Step 4 — The `dt` hazard, pinned as a test.** A test that selects a donor-controlled
  frozen flow at an unsafe `dt` and asserts the failure is **loud** (backstop fires /
  positivity violated), not silent. If it is silent, that is a finding to surface, not to
  paper over. This is the step that converts hazard 2 from folklore into a gate.
- **Step 5 — Documentation (the "documented" half of deliberate deviation).** In
  `docs/authoring-reference.md`: the new flow types + loaders; the forcing-binding boundary
  (hazard 1) and the multi-instance exclusion; the per-flow `dt` constraints (hazard 2); and
  — plainly — **that these flows are frozen but *uncalibrated*: "no UNCALIBRATED banner"
  means "no authored kinetics", not "validated"; every non-crew param is a placeholder
  pending bucket 3.** Mirror the "calibration ≠ validation (the three columns)" framing
  already in `tests/test_bvad_validation.py` — the project knows how to say this well.
- **Step 6 — Regenerate the manifest.** `uv run python tests/test_authoring_freeze_manifest.py`,
  then **review the diff**: `flow_types` 3 → 12 and `param_loaders` 2 → 5 are the git-visible
  record of exactly what was unfrozen. Nothing else in the manifest may move.
- **Step 7 — Gates + provenance.** Full suite incl. `-m slow`; `ruff`; `pyright`;
  `cargo test`; `cargo clippy --all-targets -D warnings`. Update this doc with an Outcome
  block and `CLAUDE.md`'s post-roadmap table. Conventional Commit naming the unfreeze.

## Exit criteria

- The 9 types + 3 loaders are selectable from a scenario file **on both ports**, anchored.
- **The 20 frozen goldens are byte-identical.** Registration touches no science; any golden
  movement is a bug, not an expected consequence.
- `git diff src/simcore/` empty; no domain-side edit; the manifest diff shows exactly
  `flow_types` and `param_loaders` and nothing else.
- The forcing-binding boundary and the `dt` constraints are *documented and tested*, not
  latent.
- The reference doc states the uncalibrated status of the newly-reachable science.

## Not in scope (deliberate, each with its reason)

- **The biosphere (Tier 3).** Excluded for a **structural** reason, not a calibration one:
  `Allocation` takes `ctx: CarbonContext` — a composite bundling four param objects
  (`photo`, `canopy`, `resp`, `nitro`) with four stock ids — plus `pheno` and `alloc` on
  top. `FlowTypeSpec` offers a flat `wiring_fields` tuple and a single `param_set`; neither
  can express that. Beyond the shape, the biosphere needs the aux accumulator, the shared
  `co2_pool` feedback var, the two-rate master-day driver and `annual_reset` — all explicitly
  deferred ("the interpreter builds single-rate, no-reset graphs only"). It wants a
  *frozen-compartment include*, not flow-type entries. A phase of work, not a step.
- **A provenance / calibration marker.** A bucket-3 deliverable. It cannot discriminate
  before bucket 3 exists, and it is not the cheap add it appears to be: the loaders record
  `source` and then **discard** it (`_ValueUnitSource` validates it; `load_crew_params`
  returns floats only), so it needs either a loader change or an authoring-side param
  rescan, plus an FFI crossing for the Godot banner.
- **Grammar changes.** Tier 2. No new op is required by this plan.
- **Multi-instance / prefixed instances of these flows.** Blocked by hazard 1, which is a
  pre-existing documented boundary.
- **Recalibrating any param.** Bucket 3. This plan must not move a number.

## The sequence

1. **Tier 1 — this plan.** Expands what can be **reused**: frozen science becomes
   selectable. An unfreeze; mechanical, no new logic.
2. **Tier 2 — the grammar (`monod`).** Expands what can be **said**: saturation becomes
   expressible, which no amount of `+ − ×` can approximate. The highest-ceremony unfreeze —
   `docs/authoring-reference.md` mandates advisor review *before writing anything*, because
   a grammar op freezes a semantic choice cross-port. Two open questions to settle then, not
   now: **which** saturating form (`monod(S,K) = S/(S+K)` vs a 3-arg `Vmax` form — the
   algebra is shared by Michaelis-Menten, Monod and Holling type II), and **the degenerate
   case** — `monod(0,0)` is `0/0` → NaN, and the grammar cannot guarantee `K > 0` because
   `K` is an arbitrary expression. So `x/0` does not vanish for `monod`; it relocates. NaN
   crossing a hex-float golden contract is the thing to design against. Note `min(k·S, Vmax)`
   is *not* the cheap alternative: its kink is non-differentiable and destroys RK4's
   convergence order, and `rk4` is a frozen integrator name.
3. **Bucket 3 — validation.** Establishes what is **true**: the deferred Phase-1 quantitative
   oracle match, and the 55 uncited params. The matcher machinery already exists and is
   tested (`lab/oracle_match.py`, with a test proving it *rejects* an out-of-band candidate).
   Carries the calibration-signal deliverable dropped from Tier 1. Expect findings in the
   frozen biosphere and therefore the biosphere unfreeze discipline.

Rationale for this order (the user's, recorded): deviation from reality is legitimate —
interesting, educational and fun — **as long as it is deliberate and documented**. That
makes expressiveness and reach worth having before validation, and makes Step 5's honesty
obligation the price of taking them in that order.

---

# OUTCOME — COMPLETE (2026-07-16)

All seven steps landed. The nine flow types + three param loaders are author-selectable on
**both ports**, anchored, documented, and manifest-frozen. The exit criteria held exactly:
the 20 frozen goldens are byte-identical, `git diff src/simcore/` is empty, `git diff
src/domains/` is empty (no science touched), and the manifest diff shows `flow_types`
3 → 12 and `param_loaders` 2 → 5 **and nothing else**.

## The headline result

**An authored `eclss_cabin.yaml` reproduces the frozen `eclss_state.json` byte-for-byte —
on both ports.** This is the strongest claim the registry can carry and it was not in the
plan (the plan only asked for anchors "at a safe dt"). It is the `crew_mission.yaml`
precedent lifted to a domain with **three conserved quantities** and a **six-leg forced
flow**, across nine stocks the registry had to wire correctly — including the
unclamped-source vs clamped-sink split and the single-quantity cabin compositions — with
**no build-time stoichiometry check to catch a mistake** (that check runs for authored
`kinetics` flows only, never for frozen `type` flows). The Python side additionally proves
*structural* equality against `build_eclss(...)` (same `State`, same canonical flow tuple
incl. the `EclssParams` objects), which is stronger than byte-identity and localizes
failure.

What it proves: the registry **lowers correctly**. What it does not prove: that the ECLSS
numbers are right. They are not — see Step 5.

## What the plan got wrong (four assumptions, all corrected)

1. **"This is registry entries only" (Step 2) — false for 2 of 9.** `HeatInput` and
   `CrewMetabolism` had **private fields and no `pub fn new`**; every other one of the nine
   had a public constructor. Both are the *forced stand-ins that Phase-6 station coupling
   drops*, so nothing outside their own module had ever needed to build one. Two additive
   `pub fn new` constructors were added to `rust/crates/domains/src/{thermal,eclss}.rs`,
   following the in-repo `SelfDischarge::new` precedent ("Additive over `build_power`'s
   internal struct-literal construction, which stays untouched, so the frozen sibling
   goldens can't move"). **This is a port edit and is recorded as one**: no science, no
   arithmetic, no value moved; the struct-literal construction inside each builder is
   untouched, and the frozen goldens confirm it.
2. **"Anchors must run identically on both ports under the tier contract" (Step 3) — the
   harness had no tier support at all.** `test_rust_authoring_run_matches_python` was a
   bare `rust_final == python_final`, sound only because every existing anchor happened to
   be transcendental-free. `ANCHORS` gained a `float_tier` column and the run
   parametrization now filters to Tier 1.
3. **Hazard 3's "the cross-port anchor must be tiered accordingly" resolved by
   *exclusion*, not by a band.** The discriminating fact (advisor): the **graph dump is
   bit-exact for all nine**, because it renders authored literals via `float.hex()` and
   never calls `evaluate()` — the `powf` cannot reach it. And `RadiatorReject`'s runtime
   cross-port arithmetic is *already* frozen by `thermal_state.json`'s measured band, so an
   authored thermal run anchor would only re-prove pinned arithmetic. So: graph-dump anchor
   all nine, bit-exact run anchor the eight Tier-1 flows, exclude `thermal.radiator_reject`
   from the run comparison, and cover its param wiring with a Rust unit test (the one
   sliver the dump cannot see — params are not rendered in it). **No new band was minted**:
   a measured band is a frozen tolerance, and freezing one for a runtime-only authored
   artifact cuts against "authored ≠ validated".

   **The trap this avoided** (advisor, worth preserving): the fresh-vs-fresh `==` would
   have *passed* for thermal on any single machine, because Rust and CPython there share
   one libm. Asserting that pass would have silently labelled a `powf` flow Tier-1,
   contradicting `tiers.json`'s own rule ("classify by the ops the simulator EXECUTES") and
   breaking the moment a run compared across libms.
4. **Step 4's "assert the failure is loud" — it is SILENT.** See below.

## Step 4: the dt hazard is silent (measured, not reasoned)

The plan said to assert the failure is loud, and that "if it is silent, that is a finding
to surface, not to paper over". It is silent. At `dt = 3600` the ECLSS cabin **does not
raise**, **conserves every quantity every step**, completes with `rationed = 37`, and ends
with `cabin_o2` at `-1.4e-14` — *a cabin with no oxygen, reported as a successful run*. The
backstop scales the over-draw so nothing goes properly negative, which is exactly why
nothing raises; the only signal is the `rationed` count, and `states, _, _ =
run_scenario(built)` discards it.

Pinned by `tests/test_authoring_dt_hazard.py` (5 tests), which asserts the **actual**
behavior rather than the hoped-for one. Deliberately **not fixed**: a strict/raise mode is
a platform behavior change well outside a registration unfreeze, and surfacing `rationed`
prominently (a run summary, a Godot warning) is a capability-gap item with its own design.

Two mechanisms, not one — worth separating because they fail independently:

* the **donor-controlled** `k_scrub·dt = 3.6 > 1` (the plan's hazard 2), and
* the **forced** `crew_metabolism` O₂ draw of `0.004·3600 = 14.4 mol` against a 10 mol
  cabin — a well-fed-sizing failure that would survive any scrubber-rate recalibration.

**The composability corollary, which the plan missed:** the frozen sizings *disagree with
each other*. ECLSS is sized for `dt = 60`; Thermal's `heat_capacity` for `τ ≈ 65` steps at
`dt = 3600`. A scenario composing both must pick one `dt`, and only `dt ≤ ~60` is safe for
both. **There is no `dt` natural to both domains** — and an author cannot derive that from
any single flow's documentation.

## Findings the plan did not anticipate

* **`eclss.o2_makeup` reverses above its setpoint.** The frozen docstring says an
  above-setpoint venting clamp is "a deferred seam that never arises here" — true of every
  *frozen* scenario, but an author can wire `cabin_o2` above the 10.0 mol setpoint, at
  which point the rate goes negative and the flow silently vents cabin O₂ *back into the
  supply tank*. Measured: it conserves, does not ration, and converges to `o2_eq = 8.0`. So
  it is **mild** — documented, not gated. It is the same class as the `dt` hazard: **a
  frozen flow's safety argument is scoped to the frozen scenario data, and authoring is
  what escapes that scope.** That generalization is the most transferable thing Tier 1
  learned, and Tier 2 / bucket 3 should expect more of it.
* **Nothing owned "the declared `wiring_fields` match the frozen constructor".** The
  registry's docstring argues its duplication is "a stable, deliberately-curated public
  surface … not incidental drift" — but nothing checked the copy was *accurate*. The
  manifest gate owns completeness; the anchors own does-it-run; neither notices a
  `wiring_fields` entry naming a field that does not exist on a type no anchor exercises.
  Now gated by `test_spec_mirrors_the_frozen_constructor_exactly`, which derives each
  frozen class's real dataclass fields (so it also catches a rename on the frozen side).
* **The completeness gate has teeth, confirmed.** Step 1's check ran as specified: both
  `test_frozen_flow_type_registry_is_complete` and
  `test_frozen_param_loader_set_is_complete` **failed** before regeneration. Caveat worth
  recording (advisor): the gate is plain `manifest == live`, so **after** regeneration it
  passes even for a flow type with zero anchors. "Exercised by something" remains a
  discipline, not a gate.
* **`power_bus.yaml` is Tier-1 where its own frozen scenario is Tier-2.** The frozen power
  goldens are Tier-2 because of `math.sin` in the *half-sine solar schedule* — not in any
  flow. An authored file declares constant forcings only (a documented deferral), which
  removes Power's only transcendental. So the authored descendant is *more strictly gated*
  than the frozen reference it descends from. This is also why it gets no golden: it cannot
  reproduce a run it cannot express.
* **Registering a param loader opens *two* surfaces, and the second had no anchor** (found
  in review). A set is reachable as a frozen type's `param_set` *and* as an authored
  `kinetics` rate's `param("…")`. Only the second carries a cross-port hazard: Python
  derives the key names via `asdict()`, Rust **hardcodes** them in `kinetics_param_map`, so
  a typo there resolves in Python and fails only in Rust — the "Python-only registration is
  a broken contract" failure the Rust registry's own docstring warns about, and one the
  frozen-`type` anchors cannot see (they never touch that map). Closed by
  `param_sets_dsl.yaml`, a Tier-1 anchor reading **all nine** new key names, following the
  `self_discharge_dsl.yaml` precedent exactly. It is deliberately nonsense physics
  (a battery leaking at a rate summed from a radiator's emissivity and the temperature of
  deep space) — conservation-closed nonsense is what the platform permits, and the property
  under test is param resolution, not physics. It passed first try, so the Rust key names
  were right; the point is that this is now *proven* rather than assumed.
  `test_the_kinetics_anchor_reads_every_key_of_the_three_new_sets` guards the anchor itself,
  so adding a param to a frozen set cannot leave a key silently un-anchored.
* **Consciously accepted, not overlooked:** `thermal.heat_input` rides in the Tier-2 file,
  so its *run* parity is not bit-checked even though it would qualify for Tier 1 alone. A
  third scenario file to win that buys ~nothing — the graph dump already proves its wiring
  on both ports, and it is the same `env.get(name)·dt` shape as `power.load_draw`, which
  *is* bit-checked in `power_bus.yaml`.

## Step 5: the honesty obligation, discharged

`docs/authoring-reference.md` gained two sections. **"Frozen is not calibrated"** states the
55/57 finding plainly, tabulates the two axes (shape vs values), and says the thing most
open to misreading: **"no `UNCALIBRATED` banner" means "no authored kinetics", NOT
"validated"** — a scenario built entirely from frozen types carries no marker and is no more
validated than one that authored its own kinetics; it is merely not the author's fault.
**"The `dt` constraint"** tabulates the per-flow constraint, the frozen sizing, and where
each breaks, plus the composability constraint and the silent-failure finding. The
forcing-binding boundary entry was extended from three flows to nine (six of the new ones
are forcing-bound: single-instance authoring works, prefixed multi-instance does not).

## What this cost, and what it bought

Zero engine change, zero science change, zero param moved. Two additive Rust constructors.
The platform now reaches three of the roadmap's four promised scenario facets ("a habitat
with its power budget, thermal limits, crew size, and ecosystem") — the ecosystem is the one
still unreachable, for the structural reason in "Not in scope".

## The sequence, unchanged

Tier 1 (reuse) is done. **Tier 2** (the grammar — `monod`; what can be *said*) and **bucket
3** (validation; what is *true*) are unchanged and un-started; see "The sequence" above.
Tier 1 strengthens the case for bucket 3: it made *more* uncalibrated science reachable by
authors, so the gap between "selectable" and "trustworthy" is now wider — and the only thing
holding that line is a documentation section.
