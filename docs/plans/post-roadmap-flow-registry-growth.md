# Post-roadmap: growing the author-selectable flow registry (Tier 1)

**Status: PLANNED** — not started. An **unfreeze** of the authoring platform, chosen by
the user 2026-07-16. Sequenced first of three; see "The sequence" at the foot.

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
