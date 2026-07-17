"""Multi-rate Step 5: the build-time ``k·h < 1`` precondition — the hazard closer.

Steps 1–4 built the multi-rate knob and proved it pays off. What they explicitly did NOT
do is close the hazard: *"multi-rate is the performance enabler, not the hazard closer"*
(advisor, recorded in ``docs/plans/post-roadmap-multirate-authoring.md`` ).
``multirate_step`` splits one master ``dt`` into ``dt/n_sub``, so an author who
"switches multi-rate on" and picks ``n_sub=2`` at ``dt=3600`` gets the identical hazard
one level down — measured, 36.0 against a truth of 8.0. **This file is the closer**: the
platform now refuses to *build* that scenario.

**The scope, stated honestly** (the plan's own words): the claim is *"the platform
catches the ``k·dt`` family"*, never *"your dt is safe"*. The family is the four
frozen flow types declaring
:attr:`~authoring.flow_registry.FlowTypeSpec.rate_params`. Three shapes are
uncoverable **by declaration rather than by omission**, and section 5 pins each so
that a future reader finds a decision rather than a gap.

**Why build time — the pack, not the convenience.** The obvious argument for a build
check is that an author learns before a long run rather than after. That is true and
secondary. The load-bearing argument is that a **parameter pack** may inflate a gain,
and a pack's values exist only *after* ``interpret`` resolves them: ``run_scenario``
receives an already-built flow and cannot see what the pack asked for. Section 3
measures this — a scenario at its own frozen, correct ``dt`` made unsafe purely by a
pack that passes every frozen guard.

**The user chose the whole ``k·dt`` family over the minimal ``o2_makeup``-only scope**
(2026-07-17), against both my recommendation and the advisor's, on the uniformity
argument:
*"the platform checks ``k·dt`` for any flow that declares a rate constant"* is a rule an
author can hold in their head; *"``o2_makeup`` is special"* is trivia they will forget.

**What this file corrects: the plan's own formula was wrong.** The plan specified
``k·(dt/n_sub) < 1`` for every flow. That is right for the fast set and for single-rate,
and **measurably wrong for the slow set** — a false PASS in the unsafe direction.
Section 2 owns that finding. It is the *same* Strang ``dt/2`` fact that turned Step 4's
predicted 60× Thermal saving into a measured 30×: the same blind spot has now bitten two
separate claims in this phase, from reasoning about ``n_sub`` as though it governed both
rate classes.
"""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

import pytest

from authoring.errors import AuthoringError
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS
from authoring.interpreter import _effective_step, interpret
from authoring.run import _SPLIT
from authoring.schema import ScenarioSpec
from config import load_yaml
from domains.eclss.loader import load_eclss_params
from domains.power.loader import load_self_discharge_params
from simcore.multirate import Split

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"

FROZEN_DT = 60.0  # the ECLSS anchor's own load-bearing sizing: k_makeup*dt = 0.12
UNSAFE_DT = 3600.0  # k_scrub*dt = 3.6, k_makeup*dt = 7.2

SCRUBBER = "eclss.co2_scrubber"
CONDENSER = "eclss.condenser"
MAKEUP = "eclss.o2_makeup"


def _raw(**edits: Any) -> dict[str, Any]:
    """The committed ECLSS anchor's raw YAML with run-config edits applied.

    The anchor **file** is never edited (the ``test_authoring_dt_hazard._build_at``
    discipline): only its in-memory dict, so the graph under test stays the committed
    one.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw.update(edits)
    return raw


def _build(
    *, allow_unsafe_step: bool = False, slow: tuple[str, ...] = (), **edits: Any
):
    raw = _raw(**edits)
    for flow in raw["flows"]:
        if flow["id"] in slow:
            flow["rate_class"] = "slow"
    return interpret(
        ScenarioSpec.model_validate(raw),
        base_dir=SCENARIO_DIR,
        allow_unsafe_step=allow_unsafe_step,
    )


# ---------------------------------------------------------------------------
# 1. THE DECLARATION — rate_params names real rates, and only rates
# ---------------------------------------------------------------------------


def test_every_declared_rate_param_exists_on_its_frozen_params_object() -> None:
    # The registry declares param names as STRINGS, so a renamed frozen field would
    # leave a dead entry here that silently checks nothing — `getattr` would raise at
    # build, but only for a scenario that happens to use that flow type. Read the real
    # objects off the frozen loaders instead of transcribing: this is the "the field
    # names are the real ones, checked" note in FlowTypeSpec.rate_params, made
    # executable.
    #
    # It also catches the inverse: a flow type declaring rate_params but no param_set (a
    # registry inconsistency — there would be no object to read the rate from).
    for name, spec in FLOW_TYPES.items():
        if not spec.rate_params:
            continue
        assert spec.param_set is not None, f"{name}: rate_params but no param_set"
        params_obj = PARAM_LOADERS[spec.param_set]()
        for param in spec.rate_params:
            assert hasattr(params_obj, param), f"{name}: no frozen field {param!r}"
            assert isinstance(getattr(params_obj, param), float)


def test_the_checked_family_is_exactly_the_four_first_order_rates() -> None:
    # The scope, pinned as a set. Growing it is a deliberate act (a new flow type with a
    # rate constant SHOULD land here); shrinking it silently un-checks a hazard, which
    # is what this assertion exists to prevent.
    declared = {
        name: tuple(spec.rate_params)
        for name, spec in FLOW_TYPES.items()
        if spec.rate_params
    }
    assert declared == {
        "eclss.co2_scrubber": ("co2_scrub_rate",),
        "eclss.condenser": ("condense_rate",),
        "eclss.o2_makeup": ("o2_makeup_gain",),
        "power.self_discharge": ("self_discharge_rate",),
    }


def test_o2_setpoint_is_NOT_checked_because_it_is_not_a_rate() -> None:
    # THE REASON rate_params is an explicit declaration and not "every float on the
    # params object". EclssParams carries o2_setpoint = 10.0 mol — a target INVENTORY.
    # Multiplying it by a step size is meaningless, and a naive "check every" would
    # 10.0 * 60 = 600 >= 1 and refuse the committed anchor at its own correct dt.
    #
    # The unit is the tell, and the frozen loader states it: the three rates are "1/s",
    # the setpoint is "mol". Only a 1/s quantity has a k*dt to bound.
    params = load_eclss_params()
    assert params.o2_setpoint == 10.0
    assert "o2_setpoint" not in FLOW_TYPES["eclss.o2_makeup"].rate_params
    # The anchor at its frozen dt builds cleanly — which it could not if o2_setpoint
    # were treated as a rate.
    assert _build(dt=FROZEN_DT).dt == FROZEN_DT


def test_the_frozen_rates_are_the_values_the_arithmetic_below_assumes() -> None:
    # The premise of every k*h number in this file, read from the frozen loaders rather
    # than transcribed. If a recalibration moves these, this is the test that says so —
    # and the rest of the file's expected values become suspect rather than silently
    # wrong.
    eclss = load_eclss_params()
    assert eclss.co2_scrub_rate == 1.0e-3
    assert eclss.condense_rate == 5.0e-4
    assert eclss.o2_makeup_gain == 2.0e-3
    assert load_self_discharge_params().self_discharge_rate == 1.0e-8


# ---------------------------------------------------------------------------
# 2. THE EFFECTIVE STEP — the plan's formula was wrong for the slow set
# ---------------------------------------------------------------------------


def test_the_effective_step_is_per_rate_class_not_dt_over_n_sub() -> None:
    # THE CORRECTION, as a table. The plan for this step said "the interpreter checks
    # k*(dt/n_sub) < 1" — one formula for every flow. There are THREE cases:
    #
    #   single-rate      -> dt          (the whole registry steps once per master step)
    #   multi-rate fast  -> dt/n_sub    (the point of the cadence knob)
    #   multi-rate slow  -> dt/2        (Strang's half-steps — INDEPENDENT of n_sub)
    #
    # dt/n_sub coincides with dt at n_sub=1, so single-rate is not really a separate
    # formula. The SLOW row is the one that genuinely differs, and it is the row the
    # plan missed.
    assert _effective_step(3600.0, 1, multirate=False, slow=False) == 3600.0
    assert _effective_step(3600.0, 60, multirate=True, slow=False) == 60.0
    assert _effective_step(3600.0, 60, multirate=True, slow=True) == 1800.0
    # n_sub does not touch the slow set — the same dt/2 at any cadence. This is the
    # whole content of the correction.
    assert _effective_step(3600.0, 2, multirate=True, slow=True) == 1800.0
    assert _effective_step(3600.0, 600, multirate=True, slow=True) == 1800.0


def test_the_slow_step_tracks_the_split_actually_used() -> None:
    # THE LATENT COUPLING, pinned rather than commented (advisor).
    # `_SLOW_STEP_DIVISOR = 2` is true only because the harness pins Strang. Under LIE
    # the slow set steps at the FULL dt (simcore/multirate.py: ops = [(slow, dt),
    # *fast_ops]), so the divisor would be 1 and a hardcoded 2 would make the check too
    # permissive by exactly 2x — silently, and in the unsafe direction.
    #
    # The interpreter cannot import `_SPLIT` (run.py imports the interpreter), so the
    # coupling is asserted here instead. If a future step exposes `split` to authors or
    # flips the default for a study, this goes red and names its own remedy.
    assert _SPLIT is Split.STRANG, (
        "authoring.run._SPLIT is no longer Strang — interpreter._SLOW_STEP_DIVISOR "
        "must change with it (Lie steps the slow set at the full dt, so it becomes "
        "1.0); leaving it at 2.0 would UNDER-state every slow flow's step by 2x"
    )


def test_a_slow_flow_is_judged_at_dt_over_2_not_the_plans_formula() -> None:
    # THE MEASURED FALSE PASS — the finding that forced the correction above, kept as a
    # regression pin because the plan's formula is the intuitive one and would be
    # reintroduced by anyone reading only the plan.
    #
    # Scrubber SLOW, master dt=3600, n_sub=60:
    #   the plan's formula:  k * (3600/60) = 1e-3 * 60   = 0.06  -> PASS
    #   the truth:           k * (3600/2)  = 1e-3 * 1800 = 1.8   -> UNSAFE
    # Measured consequence of believing the formula: the run rations 24x over 24 steps
    # and empties cabin_co2 to exactly 0.0 while reporting a "safe" k*h of 0.06.
    with pytest.raises(AuthoringError) as excinfo:
        _build(slow=(SCRUBBER,), n_sub=60, dt=UNSAFE_DT, steps=24)
    msg = str(excinfo.value)
    assert "1.8" in msg, f"the TRUE k*h must be named, got: {msg}"
    assert "0.06" not in msg, "the plan's (wrong) k*h must not be what the author reads"
    assert "dt/2=1800.0" in msg
    assert "NOT dt/n_sub" in msg  # names the trap explicitly


def test_the_same_flow_at_the_same_dt_is_fine_as_FAST() -> None:
    # The control that implicates the RATE CLASS rather than the flow, the graph or the
    # dt. Identical file, identical dt, identical n_sub, identical k — only `rate_class`
    # moves, and 1.8 becomes 0.06 because the fast set is what n_sub governs.
    assert _build(n_sub=60, dt=UNSAFE_DT).is_multirate is True
    with pytest.raises(AuthoringError):
        _build(slow=(SCRUBBER,), n_sub=60, dt=UNSAFE_DT)


def test_raising_n_sub_cannot_rescue_a_slow_flow_and_the_message_says_so() -> None:
    # The remedy an author will reach for first, pinned as NOT working — at any cadence,
    # to a thousand. A message that said "increase n_sub" here would send them in a
    # circle and cost them the trust they need to believe the next diagnostic.
    for n_sub in (2, 60, 600, 1000):
        with pytest.raises(AuthoringError) as excinfo:
            _build(slow=(SCRUBBER,), n_sub=n_sub, dt=UNSAFE_DT)
        assert "REGARDLESS of n_sub" in str(excinfo.value)
        assert "re-class this flow 'fast'" in str(excinfo.value)


# ---------------------------------------------------------------------------
# 3. THE PACK — the load-bearing reason the check is at BUILD time
# ---------------------------------------------------------------------------


def test_a_pack_that_inflates_a_gain_is_caught_and_only_a_build_check_can_see_it() -> (
    None
):
    # THE ARGUMENT FOR THE LOCUS. The scenario is the committed anchor at its OWN
    # frozen, correct, load-bearing dt = 60 — the file is untouched and right. The pack
    # alone makes it unsafe: o2_makeup_gain 2e-3 -> 2e-2, so k*dt goes 0.12 -> 1.2.
    #
    # Every guard that existed before this step says yes: the frozen loader reads the
    # pack, the exact-unit guard passes ("1/s"), the bound check passes (a gain needs
    # only > 0 — there IS no dt-independent "too big" for a gain, which is exactly why
    # this cannot be a loader bound). And `run_scenario` cannot see it either: it
    # receives a constructed flow, long after the pack has been resolved into an
    # anonymous float.
    #
    # So this hazard is visible at exactly one point in the pipeline. `interpret` is it.
    raw = _raw(dt=FROZEN_DT)
    for flow in raw["flows"]:
        if flow["id"] == MAKEUP:
            flow["params"] = {"pack": "packs/eclss_hot_makeup.yaml"}
    with pytest.raises(AuthoringError) as excinfo:
        interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)
    msg = str(excinfo.value)
    assert "o2_makeup_gain" in msg
    assert "0.02" in msg, f"the PACK's inflated k must be named, got: {msg}"
    assert "1.2" in msg  # k*h at the file's own frozen dt


def test_the_inflating_pack_passes_every_frozen_guard_it_should() -> None:
    # The other half of the test above, and the one that makes it mean something: the
    # pack is not malformed. If the frozen loader rejected it, the build check would be
    # redundant rather than load-bearing. It loads, and the value it produces is the
    # inflated one — a legal param set that happens to be unusable at this dt.
    params = load_eclss_params(SCENARIO_DIR / "packs" / "eclss_hot_makeup.yaml")
    assert params.o2_makeup_gain == 2.0e-2  # 10x the frozen value, and perfectly valid
    assert params.o2_setpoint == 10.0
    # It is unsafe only RELATIVE TO A STEP — at a small enough dt the same pack is fine,
    # which is why this can never be a loader bound. 2e-2 * 10 = 0.2 < 1.
    raw = _raw(dt=10.0, steps=2)
    for flow in raw["flows"]:
        if flow["id"] == MAKEUP:
            flow["params"] = {"pack": "packs/eclss_hot_makeup.yaml"}
    assert interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR).dt == 10.0


# ---------------------------------------------------------------------------
# 4. THE BOUND AND THE HATCH
# ---------------------------------------------------------------------------


def test_the_bound_is_strictly_less_than_one_not_less_than_two() -> None:
    # k*h == 1 exactly is REFUSED. This is the deadbeat case (12 -> 10 -> 10 -> 10),
    # which is not unstable at all — it converges in one step. It is refused anyway, and
    # the reason is the export-fidelity ruling this phase rests on: k*h < 2 answers
    # "does the solver diverge"; k*h < 1 answers "is the export usable by a neighbour".
    # We couple domains, so < 1 governs. See
    # docs/plans/post-roadmap-multirate-authoring.md.
    #
    # k_makeup = 2e-3, so dt = 500 gives exactly 1.0.
    with pytest.raises(AuthoringError, match="o2_makeup_gain"):
        _build(dt=500.0, steps=2)
    # And just below it builds — the bound is where it says it is, not approximately.
    assert _build(dt=499.0, steps=2).dt == 499.0


def test_the_hatch_opens_the_build_and_nothing_else() -> None:
    # `allow_unsafe_step=True` is the `run_scenario(allow_rationing=True)` idiom: for
    # STUDYING an unsafe run, never for making a scenario work. It does not make the
    # step safe — it makes the platform stop objecting, and the scenario it hands back
    # is exactly the broken one that was asked for.
    built = _build(dt=UNSAFE_DT, steps=15, allow_unsafe_step=True)
    assert built.dt == UNSAFE_DT
    # The hatch is not a mode: the very next build without it refuses again.
    with pytest.raises(AuthoringError):
        _build(dt=UNSAFE_DT, steps=15)


def test_the_committed_scenarios_all_pass_the_precondition() -> None:
    # NO GOLDEN MOVED, and this is the assertion behind that claim rather than a hope.
    # The precondition is a refusal, so the risk it carries is refusing something that
    # already worked. Every committed scenario builds on the author's default path — no
    # hatch.
    #
    # Scenario files only: `tests/authoring/scenarios/*.yaml` includes bundles and param
    # packs, which are not scenarios and fail schema validation by design.
    built_count = 0
    for path in sorted(SCENARIO_DIR.glob("*.yaml")):
        try:
            spec = ScenarioSpec.model_validate(load_yaml(str(path)))
        except Exception:
            continue  # a bundle/pack/fixture, not a scenario
        try:
            interpret(spec, base_dir=path.parent)
        except AuthoringError as exc:
            if "rate constant" in str(exc):
                pytest.fail(f"{path.name} no longer builds: {exc}")
            continue  # a deliberately-broken fixture (crew_broken_wiring, bad packs)
        except Exception:
            continue
        built_count += 1
    assert built_count >= 8, (
        f"expected the committed anchors to build, got {built_count}"
    )


# ---------------------------------------------------------------------------
# 5. THE HONEST SCOPE — the three shapes this CANNOT cover, by declaration
# ---------------------------------------------------------------------------


def test_the_uncoverable_shapes_are_declared_empty_not_forgotten() -> None:
    # "Document, do not fake" (the plan). Each of these has a real constraint that a
    # build-time param check cannot express, and declaring rate_params=() is the record
    # of that decision. A future reader finds a ruling here, not an oversight:
    #
    #  * thermal.radiator_reject — tau = C/(4*eps*sigma*A*T_eq^3) >> dt. ">>" IS NOT A
    #    PREDICATE. Making it one means inventing a safety factor the science does not
    #    supply, and a fabricated threshold that reads as a guarantee is worse than an
    #    honest gap.
    #  * eclss.crew_metabolism — "forced draw < stock" is STATE-dependent. A build check
    #    sees only the INITIAL amount, so it is necessary-not-sufficient; rationing
    #    catches it at run time, and that stays its guard. This is why the run-time
    #    gate was not removed when this one landed: neither subsumes the other.
    assert FLOW_TYPES["thermal.radiator_reject"].rate_params == ()
    assert FLOW_TYPES["eclss.crew_metabolism"].rate_params == ()
    # Both still build at a dt where a first-order flow of the same k would be refused —
    # the honest consequence of the gap, not a bug.
    assert FLOW_TYPES["thermal.radiator_reject"].param_set == "thermal"


def test_authored_kinetics_are_structurally_uncheckable_and_that_is_decision_B() -> (
    None
):
    # The third uncoverable shape, and the only one that is a BOUNDARY rather than a
    # limitation. The author wrote the rate law, so the platform cannot know which of
    # its params (if any) is a first-order constant — `param('k') * stock('x')` and
    # `param('k') + stock('x')` are the same shape to the interpreter.
    #
    # This is exactly decision B's "authored != validated" line, not a gap to be closed:
    # an authored kinetics flow gets conservation + determinism, never scientific
    # endorsement. `rate_params` lives on FlowTypeSpec — the FROZEN-type registry — and
    # a kinetics flow has no FlowTypeSpec at all, so the check skips it by construction
    # rather than by a special case.
    raw = _raw(dt=UNSAFE_DT, steps=2)
    # Re-express the scrubber's own frozen law as authored kinetics at the unsafe dt:
    # the identical arithmetic the precondition refuses above, now invisible to it.
    raw["flows"] = [
        {
            "id": "authored.scrub",
            "priority": 0,
            "params": "eclss",
            "kinetics": {
                "rate": "param('co2_scrub_rate') * stock('eclss.cabin_co2')",
                "stoichiometry": {
                    "eclss.cabin_co2": -1.0,
                    "boundary.co2_removed": 1.0,
                },
            },
        }
    ]
    raw["stocks"] = [
        s
        for s in raw["stocks"]
        if s["id"] in ("eclss.cabin_co2", "boundary.co2_removed")
    ]
    raw.pop("forcings", None)
    # It BUILDS — same k, same dt, same k*h = 3.6 the frozen type is refused for.
    built = interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)
    assert built.has_authored_kinetics is True
    assert built.dt == UNSAFE_DT
