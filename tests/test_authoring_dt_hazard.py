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

**THE FINDING THIS FILE PINS: the failure is SILENT.** Measured, not assumed (the plan
required settling this empirically rather than by reasoning). At ``dt = 3600`` the run

  * does **not** raise,
  * **conserves** every quantity every step (the ledger gate never trips),
  * completes with ``rationed = 37``,
  * and leaves ``cabin_o2`` at ``-1.4e-14`` — **a cabin with no oxygen**.

The arbitration backstop does its job: it scales the over-draw so no stock goes properly
negative, which is exactly why nothing is raised. The *only* signal is the ``rationed``
count returned from :func:`authoring.run.run_scenario` — and a caller who writes
``states, _, _ = run_scenario(built)`` (the natural way to call it) discards it. So the
platform reports a successful, conserving run of a habitat that asphyxiated its crew.

That is a **flagged finding, not a defect to paper over** — and deliberately NOT fixed
here. Adding a strict/raise mode would be a platform behavior change well outside a
registration unfreeze, and surfacing ``rationed`` more prominently (a run summary, the
Godot banner) is a capability-gap item with its own design. What Tier 1 owes is that the
hazard is *pinned and documented* rather than latent: this file is the gate, and
``docs/authoring-reference.md`` ("The dt constraint") is the author-facing warning.

The **composability** corollary, which is why this matters beyond one scenario: the
frozen sizings disagree with each other. ECLSS is sized for ``dt = 60``; Thermal's
``heat_capacity`` is sized so ``tau ~ 65 steps`` at ``dt = 3600``. A scenario composing
both must pick ONE ``dt``, and only ``dt <= ~60`` is safe for both (a smaller dt only
ever helps Thermal). There is no ``dt`` that is natural for both domains.
"""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

import pytest

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


def _run_at(dt: float, steps: int) -> tuple[list, int, tuple]:
    """Interpret the ECLSS anchor with ``dt``/``steps`` overridden, and run it.

    Mutates a parsed copy of the committed anchor rather than shipping a second,
    near-duplicate scenario file: the *only* difference that matters is ``dt``, and
    keeping one file makes that unmissable.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = dt
    raw["steps"] = steps
    built = interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)
    return run_scenario(built)


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


def test_at_an_unsafe_dt_the_failure_is_silent_not_loud() -> None:
    # THE FINDING. Everything about this run says "success" except the one integer a
    # caller is free to discard.
    states, rationed, events = _run_at(UNSAFE_DT, 15)

    # It does not raise (the call above completing IS that assertion) and it conserves —
    # the every-step ledger gate lives inside step_report, so a completed run proves it.
    assert events == ()

    # The backstop fired: the ONLY signal that anything went wrong.
    assert rationed > 0, (
        "expected the Euler backstop to ration the over-draw at k*dt = 3.6; "
        "if this is 0 the hazard's mechanism has changed and the docs are stale"
    )

    # And here is what it silently permitted: the cabin oxygen is gone. The backstop
    # clamps at zero (its job), so this is ~0 from below by float roundoff, not a
    # properly negative stock — which is precisely why nothing raised.
    cabin_o2_final = states[-1].stocks["eclss.cabin_o2"].amount
    assert cabin_o2_final == pytest.approx(0.0, abs=1e-9)

    # Sharpen it: the crew started with a full 10 mol cabin and ended with none of it.
    assert states[0].stocks["eclss.cabin_o2"].amount == pytest.approx(10.0)


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
