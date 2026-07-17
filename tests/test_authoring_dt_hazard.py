"""Tier-1 Step 4: the frozen params carry implicit ``dt`` assumptions — and an author
picks ``dt``.

**This is the hazard that registration CREATED**, and it is the sharpest one in the
Tier-1 unfreeze (``docs/plans/post-roadmap-flow-registry-growth.md``). Every frozen rate
constant was sized against the ``dt`` of *its own frozen scenario*, and that sizing is
part of the flow's positivity argument — but it lives in a YAML comment, not the code.
Selecting the flow type from a scenario file hands the author the ``dt`` knob with no
guard attached:

  * ``eclss.co2_scrubber``'s ``k_scrub = 1.0e-3 /s`` is sized for ``dt = 60 s``
    (``k·dt = 0.06``). At ``dt = 3600`` it is ``3.6 > 1``: the donor-controlled draw
    demands 3.6x the entire stock in one step. ``eclss.condenser`` is the same at 1.8.
  * ``eclss.crew_metabolism`` is *forced*, so its positivity was only ever WELL-FED
    SIZING. At ``dt = 3600`` its O2 draw is ``0.004 * 3600 = 14.4 mol`` per step against
    a 10 mol cabin — the draw exceeds the whole stock, independent of the scrubber.

**The frozen flow is correct; the authored ``dt`` makes it wrong.** No flow here is
buggy, no param is wrong, and nothing needs fixing in ``domains/`` — the frozen sizing
argument is simply scoped to the frozen scenario, and authoring is what escapes that
scope. It is reachable by an author who wrote **no kinetics at all**, and whose scenario
therefore carries **no ``UNCALIBRATED`` banner** (that marker tracks *who wrote the rate
law*, never whether a run is sound).

**THE FINDING THIS FILE PINNED: the failure was SILENT.** Measured, not assumed (the
plan required settling this empirically rather than by reasoning). At ``dt = 3600`` the
run

  * did **not** raise,
  * **conserved** every quantity every step (the ledger gate never trips),
  * completed with ``rationed = 37``,
  * and left ``cabin_o2`` at ``-1.4e-14`` — **a cabin with no oxygen**.

The arbitration backstop does its job: it scales the over-draw so no stock goes properly
negative, which is exactly why nothing was raised. The *only* signal was the
``rationed`` count returned from :func:`authoring.run.run_scenario` — and a caller who
writes
``states, _, _ = run_scenario(built)`` (the natural way to call it) discards it. So the
platform reported a successful, conserving run of a habitat that asphyxiated its crew.

**UPDATE (post-roadmap): the SILENCE is fixed; the HAZARD is not.** ``run_scenario`` now
raises :class:`authoring.errors.RationedError` on ``total_rationed > 0``. Read the
distinction carefully, because this file is the thing that keeps it honest:

  * **What changed** — the *harness* refuses to hand back a rationed trajectory. That
    reaches the verdict the rest of the project had already reached everywhere else (the
    goldens assert ``rationed == 0``; ``StepReport`` calls a nonzero count "a failing
    gate, not a warning"; RK4 hard-errors on the identical condition;
    ``station.objectives`` scores a rationed run ``survived = False``). It is a
    consistency fix, not a new policy.
  * **What did NOT change** — the physics, the params, the frozen sizings, or the fact
    that ``k_scrub·dt = 3.6`` at ``dt = 3600``. The cabin still asphyxiates; you can
    still watch it do so with ``allow_rationing=True``, and the tests below still assert
    that it does, at the same numbers as before. **We made the failure loud. We did not
    make the scenario work** — and a reader who takes a green run here as "the dt hazard
    is handled" has misread it exactly backwards.

The composability constraint below is likewise untouched: an author must still pick a
``dt``, and there is still no ``dt`` natural to both ECLSS and Thermal. What they get
now is an exception instead of a corpse.

The **composability** corollary, which is why this matters beyond one scenario: the
frozen sizings disagree with each other. ECLSS is sized for ``dt = 60``; Thermal's
``heat_capacity`` is sized so ``tau ~ 65 steps`` at ``dt = 3600``. A scenario composing
both must pick ONE ``dt``, and only ``dt <= ~60`` is safe for both (a smaller dt only
ever helps Thermal). There is no ``dt`` that is natural for both domains.

``docs/authoring-reference.md`` ("The dt constraint") remains the author-facing warning.
"""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

import pytest

from authoring.errors import AuthoringError, RationedError
from authoring.interpreter import interpret
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"

# The frozen eclss.yaml rates, restated so the arithmetic below is legible. Sourced from
# the loader in the test that uses them — never hardcoded as the source of truth.
FROZEN_DT = 60.0  # EclssScenario.dt_seconds — what the rates are sized for
UNSAFE_DT = 3600.0  # 1 h — k_scrub * dt = 3.6 > 1


def _build_at(dt: float, steps: int) -> Any:
    """Interpret the ECLSS anchor with ``dt``/``steps`` overridden.

    Mutates a parsed copy of the committed anchor rather than shipping a second,
    near-duplicate scenario file: the *only* difference that matters is ``dt``, and
    keeping one file makes that unmissable.

    ``allow_unsafe_step=True`` because multi-rate Step 5 added a **build-time**
    ``k·h < 1`` precondition, which refuses ``UNSAFE_DT`` before a single step runs. The
    hazard this file documents is therefore now caught **twice, at two different
    stages**, and the tests below still exercise the *run-time* gate deliberately:
    disabling the build check is what keeps them pointed at the ``RationedError`` they
    were written to pin, instead of silently becoming duplicate tests of the build
    check. ``test_the_build_now_refuses_ the_unsafe_dt_before_any_step_runs`` covers the
    new stage.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = dt
    raw["steps"] = steps
    return interpret(
        ScenarioSpec.model_validate(raw),
        base_dir=SCENARIO_DIR,
        allow_unsafe_step=True,
    )


def _run_at(
    dt: float, steps: int, *, allow_rationing: bool = False
) -> tuple[list, int, tuple]:
    """Build at ``dt`` and run; ``allow_rationing`` opts back in to pre-fix behavior.

    The default is the *author's* path (raises on rationing); the tests that inspect the
    asphyxiated trajectory pass ``allow_rationing=True`` deliberately — that is the
    escape hatch doing its one job, and using it here is not a workaround but the point.
    """
    return run_scenario(_build_at(dt, steps), allow_rationing=allow_rationing)


def test_the_frozen_rates_are_sized_for_the_frozen_dt() -> None:
    # The premise, read from the frozen loader (not transcribed): at dt = 60 both
    # donor-controlled ECLSS rates satisfy the structural-positivity condition k*dt < 1,
    # and at dt = 3600 both violate it. If a future recalibration moves these rates,
    # is the test that notices the hazard's arithmetic changed.
    from domains.eclss.loader import load_eclss_params

    p = load_eclss_params()
    assert p.co2_scrub_rate * FROZEN_DT == pytest.approx(0.06)
    assert p.condense_rate * FROZEN_DT == pytest.approx(0.03)
    assert p.co2_scrub_rate * UNSAFE_DT == pytest.approx(3.6)
    assert p.condense_rate * UNSAFE_DT == pytest.approx(1.8)
    assert p.co2_scrub_rate * FROZEN_DT < 1.0 < p.co2_scrub_rate * UNSAFE_DT


def test_at_the_frozen_dt_the_backstop_never_fires() -> None:
    # The control. Same graph, same frozen params, the sized dt: positivity holds
    # (structurally for CO2/H2O, by well-fed sizing for O2) and nothing is rationed.
    # Without this, the unsafe-dt assertion below would not implicate dt.
    _states, rationed, events = _run_at(FROZEN_DT, 900)
    assert rationed == 0
    assert events == ()


def test_the_build_now_refuses_the_unsafe_dt_before_any_step_runs() -> None:
    # THE SECOND GATE (multi-rate Step 5), one stage earlier than the one below. The
    # author's natural call is `interpret` WITHOUT the study hatch, and it now refuses:
    # k_scrub * 3600 = 3.6 >= 1 is decidable from the params + dt alone, so there is no
    # reason to make an author spend a long run to learn it.
    #
    # This is why every other test in this file passes allow_unsafe_step=True: the
    # run-time gate they pin is now UNREACHABLE by an author's default path. Both gates
    # are kept because they catch different populations — this one sees a param PACK
    # that inflates a gain (visible only after interpretation) and the demand-controlled
    # o2_makeup that rationing structurally cannot see; the run-time one sees the
    # state-dependent over-draws (crew_metabolism's forced draw) that no build check can
    # decide. Neither subsumes the other.
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = UNSAFE_DT
    raw["steps"] = 15
    with pytest.raises(AuthoringError) as excinfo:
        interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)

    msg = str(excinfo.value)
    assert "co2_scrub_rate" in msg, f"the offending param must be named, got: {msg}"
    assert "3.6" in msg, f"the k*h value must be named, got: {msg}"
    assert "allow_unsafe_step" in msg, "the study hatch must be discoverable"


def test_at_an_unsafe_dt_the_run_now_raises_instead_of_returning() -> None:
    # THE GATE (the post-roadmap fix). The author's natural call — no escape hatch — now
    # refuses to hand back the trajectory. This is the assertion that would have saved
    # the fictional crew: it fails LOUDLY at the surface an author actually touches.
    with pytest.raises(RationedError) as excinfo:
        _run_at(UNSAFE_DT, 15)

    # The message must carry the two things an author needs to act: how bad, and which
    # knob. Asserted because a diagnostic nobody can act on is barely better than the
    # silence it replaced.
    msg = str(excinfo.value)
    assert "37" in msg, f"the count must be named, got: {msg}"
    assert "dt" in msg and "3600" in msg, (
        f"the offending knob must be named, got: {msg}"
    )
    assert "allow_rationing" in msg, (
        "the escape hatch must be discoverable from the error"
    )


def test_the_underlying_hazard_is_UNCHANGED_only_its_silence_was_fixed() -> None:
    # THE FINDING, PRESERVED. The raise above is a messenger; this asserts the message
    # is still true. Everything the pre-fix version of this file measured still holds:
    # the physics was never touched, and `allow_rationing=True` shows it unchanged.
    #
    # Keep this test. If a future dt-guard, recalibration, or kinetics change makes the
    # cabin survive at dt=3600, THIS is the test that should turn red and be rewritten
    # with new numbers — not quietly deleted because "the hazard is fixed now".
    states, rationed, events = _run_at(UNSAFE_DT, 15, allow_rationing=True)

    # Still conserves every step (the ledger gate lives in step_report, so completing
    # proves it) and still emits no events. Conservation was never the problem.
    assert events == ()

    # Still exactly 37 backstop firings — the same number the docs quote.
    assert rationed == 37, (
        "expected the Euler backstop to ration 37x at k*dt = 3.6; if this changed, the "
        "hazard's mechanism moved and docs/authoring-reference.md + this file are stale"
    )

    # And the cabin is still airless. The backstop clamps at zero (its job), so this is
    # ~0 from below by float roundoff, not a properly negative stock — which is
    # precisely why nothing raised before, and why conservation alone could never have
    # caught it.
    cabin_o2_final = states[-1].stocks["eclss.cabin_o2"].amount
    assert cabin_o2_final == pytest.approx(0.0, abs=1e-9)

    # Sharpen it: the crew started with a full 10 mol cabin and ended with none of it.
    assert states[0].stocks["eclss.cabin_o2"].amount == pytest.approx(10.0)


def test_the_escape_hatch_is_the_only_way_to_get_a_rationed_trajectory() -> None:
    # The escape hatch must be *deliberate*: same scenario, same dt, opposite outcomes,
    # differing only by the opt-in. This pins that the default is the safe one — a
    # regression that flipped the default would leave the test above green (it opts in)
    # and only this test would notice.
    with pytest.raises(RationedError):
        run_scenario(_build_at(UNSAFE_DT, 15))
    _states, rationed, _events = run_scenario(
        _build_at(UNSAFE_DT, 15), allow_rationing=True
    )
    assert rationed == 37


def test_the_forced_draw_alone_exceeds_the_cabin_at_the_unsafe_dt() -> None:
    # The second, independent mechanism — worth separating, because it is NOT the k*dt<1
    # story and survives any scrubber-rate recalibration. crew_metabolism is FORCED,
    # so its positivity was only ever well-fed sizing; at dt = 3600 a single step's O2
    # draw outruns the entire cabin inventory. Pure arithmetic on the committed anchor.
    raw: dict[str, Any] = load_yaml(str(ECLSS_YAML))
    o2_rate = raw["forcings"]["o2_consumption"]["const"]
    cabin_o2_0 = next(s["amount"] for s in raw["stocks"] if s["id"] == "eclss.cabin_o2")
    assert o2_rate * FROZEN_DT < cabin_o2_0  # 0.24 mol of a 10 mol cabin — fine
    assert o2_rate * UNSAFE_DT > cabin_o2_0  # 14.4 mol of a 10 mol cabin — impossible


def test_the_frozen_sizings_disagree_so_there_is_no_universal_dt() -> None:
    # The composability constraint an author most needs and cannot derive: ECLSS's rates
    # want dt <= ~60, while Thermal's heat_capacity is sized so tau ~ 65 steps at
    # dt = 3600. Composing both domains forces one dt, and only the small
    # one is safe (a smaller dt only ever helps Thermal's Euler-overshoot margin).
    from domains.eclss.loader import load_eclss_params
    from domains.thermal.loader import load_thermal_params
    from domains.thermal.system import relaxation_time

    tau = relaxation_time(load_thermal_params())
    assert tau / 3600.0 > 10.0  # Thermal is comfortable at dt = 3600 (tau ~ 65 steps)

    # ...but ECLSS is not, at that dt. The two frozen sizings are incompatible.
    assert load_eclss_params().co2_scrub_rate * 3600.0 > 1.0

    # At ECLSS's dt, Thermal is even safer — so dt <= 60 is the composable choice.
    assert tau / 60.0 > 10.0
