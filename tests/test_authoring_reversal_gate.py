"""The direction gate: ``run_scenario`` refuses a demand-controlled flow that reversed.

**What it closes.** ``eclss.o2_makeup`` is the registry's only demand-controlled flow:
its magnitude is ``k·(setpoint − stock)``, so above the setpoint it goes negative and
the flow drains the stock it is named for, back into its source. An author who wired
``cabin_o2`` above the frozen ``10.0 mol`` setpoint got ``−1.2 mol`` on step 1 and no
complaint from anything (``test_authoring_export_fidelity.py`` measured that silence).

**Why it needed a third gate rather than widening one of the two that exist.** They are
blind to it for *different structural reasons*, and neither is fixable:

* conservation is a **stoichiometry** check — the flow's two legs share one magnitude,
  so OXYGEN balances exactly whichever direction it points;
* ``RationedError`` is a **scarcity** check — the draw is proportional to the setpoint
  *error*, not to the stock, so it never over-draws and the backstop never fires.

Reversal is a **direction** defect. Neither existing gate measures direction, so a third
one is the honest shape rather than a stretched version of either.

**The reconciliation that made this buildable at all.** Three *frozen* scenarios
reverse **legitimately** — `greenhouse`, `harvest` and `sealed_station` wire this
regulator to the biosphere O₂ pool, where the crop out-produces the crew
(``docs/plans/post-roadmap-o2-makeup-reversal.md``). A gate that condemned them would be
wrong. It cannot reach them, for a reason that is asserted here rather than assumed: the
frozen scenarios are built directly in Python and never pass through ``interpret``, and
the biosphere is not in the flow-type registry at all, so **no authored file can put a
crop on the far side of the regulator**. Today every reversal an authored graph can
produce is a mis-wiring. ⚠ That premise expires if the biosphere ever becomes authorable
— pinned below so the expiry is noisy rather than silent.
"""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

import pytest

from authoring.errors import RationedError, ReversedFlowError
from authoring.flow_registry import FLOW_TYPES
from authoring.interpreter import interpret
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.ids import StockId

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"

SETPOINT = 10.0
"""The frozen ``o2_setpoint``. Restated for the arithmetic's legibility; the assertions
below read the value the gate actually used, out of its own message."""


def _makeup_only(
    dt: float, steps: int, cabin_o2_0: float, *, allow_unsafe_step: bool = False
) -> Any:
    """``eclss.o2_makeup`` alone, wired at ``cabin_o2_0`` — the export-fidelity rig.

    Reduced to the one flow on purpose: a full cabin raises for reasons that have
    nothing to do with direction, and a gate test that trips on the wrong error is a
    test of nothing. ``allow_unsafe_step`` is off by default here (unlike
    ``test_authoring_export_fidelity.py``, where every band it measures needs it) —
    this file's subject is the *direction* gate, which must be reachable at a perfectly
    safe step.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = dt
    raw["steps"] = steps
    raw["stocks"] = [
        s for s in raw["stocks"] if s["id"] in ("eclss.cabin_o2", "boundary.o2_supply")
    ]
    for stock in raw["stocks"]:
        if stock["id"] == "eclss.cabin_o2":
            stock["amount"] = cabin_o2_0
    raw["flows"] = [f for f in raw["flows"] if f["id"] == "eclss.o2_makeup"]
    raw.pop("forcings", None)
    return interpret(
        ScenarioSpec.model_validate(raw), allow_unsafe_step=allow_unsafe_step
    )


def test_a_reversed_run_now_raises_instead_of_returning_quietly() -> None:
    """The gate fires, and the message names the step: *when* is the diagnosis."""
    with pytest.raises(ReversedFlowError) as excinfo:
        run_scenario(_makeup_only(60.0, 4, 20.0))
    message = str(excinfo.value)
    assert "eclss.o2_makeup" in message
    assert "INITIAL state" in message, (
        "a crossing at t=0 is a wiring error and must be reported as one — an author "
        "told only 'it reversed' does not know whether to change the file or the graph"
    )
    assert "allow_reversal=True" in message, "the opt-out must be discoverable"


def test_the_run_at_the_setpoint_is_allowed_because_it_does_not_reverse() -> None:
    """Exactly *at* the setpoint the magnitude is 0, not negative. The gate is ``>``.

    This is the boundary the frozen ``eclss_cabin.yaml`` fixture sits on
    (``cabin_o2_0 == o2_setpoint``, the regulator idling), so a ``>=`` gate would have
    condemned the platform's own committed example. Pinned rather than left to the
    fixture passing by luck.
    """
    states, rationed, _events = run_scenario(_makeup_only(60.0, 4, SETPOINT))
    assert rationed == 0
    assert states[0].stocks[StockId("eclss.cabin_o2")].amount == SETPOINT


def test_below_the_setpoint_the_gate_is_silent() -> None:
    """The control: a correctly-wired makeup run is untouched by this work."""
    states, rationed, _events = run_scenario(_makeup_only(60.0, 20, 5.0))
    assert rationed == 0
    o2 = [s.stocks[StockId("eclss.cabin_o2")].amount for s in states]
    assert o2[-1] > o2[0], "the regulator should have been filling the cabin"
    assert all(x <= SETPOINT for x in o2), "and never overshooting its own target"


def test_allow_reversal_returns_the_trajectory_for_study() -> None:
    """The opt-out is a real escape hatch, not a way to make a scenario 'work'.

    Same contract as ``allow_rationing``: the run is still wrong, and you get to look
    at it. Asserting the reversal is genuinely present under the flag is what makes
    this different from asserting the flag merely suppresses an exception.
    """
    states, rationed, _events = run_scenario(
        _makeup_only(60.0, 4, 20.0), allow_reversal=True
    )
    o2 = [s.stocks[StockId("eclss.cabin_o2")].amount for s in states]
    supply = [s.stocks[StockId("boundary.o2_supply")].amount for s in states]
    assert o2[1] < o2[0], "the cabin should have LOST oxygen to a flow named 'makeup'"
    assert supply[1] > supply[0], "...and the tank should have gained it"
    assert rationed == 0, "and nothing else objected, which is the whole point"


def test_the_rationing_verdict_is_reported_before_the_direction_one() -> None:
    """Order matters: a rationed trajectory is already suspect, so blame ``dt`` first.

    A run that both rations and reverses would otherwise be reported as a wiring error,
    sending the author to edit a file whose real problem is the step size.

    ⚠ **Reaching this state at all needs the study hatch, and that is a finding, not a
    setup detail.** Multi-rate Step 5's build-time ``k·h < 1`` precondition refuses
    ``dt = 1200`` at *interpret* time, so an author cannot construct a run that both
    rations and reverses by accident — the ordering below only ever decides which
    verdict a **deliberate** study sees. Recorded because it says the two run-time gates
    overlap far less in practice than their descriptions suggest.
    """
    built = _makeup_only(1200.0, 8, 12.0, allow_unsafe_step=True)
    with pytest.raises(RationedError):
        run_scenario(built)


def test_exactly_one_registered_flow_type_claims_the_demand_controlled_shape() -> None:
    """The gate is data-driven off the registry, so the census is the gate's scope.

    If a second demand-controlled type is ever registered this goes red — which is the
    intent: the new type's setpoint pair has to be a decision someone made, not a field
    left at its default.
    """
    claimed = {
        name: spec.demand_controlled
        for name, spec in FLOW_TYPES.items()
        if spec.demand_controlled is not None
    }
    assert claimed == {"eclss.o2_makeup": ("cabin_o2", "o2_setpoint")}, claimed


def test_no_frozen_scenario_reaches_the_reversal_gate() -> None:
    """The reconciliation, asserted: the gate cannot condemn the frozen roster.

    Three frozen scenarios reverse **legitimately** (a crop out-producing the crew is
    correct P-control). They are safe from this gate for two independent reasons, and
    both are pinned because either one alone would be a thin argument:

    1. **They never pass through this layer** — the frozen builds are Python
       (``build_greenhouse`` and friends), and ``run_scenario`` only ever sees a
       ``BuiltScenario`` produced by ``interpret``.
    2. **An authored file could not reproduce their situation anyway** — the biosphere
       is absent from the flow-type registry, so there is no way to author a crop that
       fills the regulated stock.

    ⚠ Reason 2 is the one with an expiry date. Making the biosphere authorable would
    turn this test red, which is the point: the gate's premise would have to be
    re-decided rather than quietly outlived.
    """
    import station.greenhouse as greenhouse
    import station.sealed as sealed

    for module in (greenhouse, sealed):
        source = Path(module.__file__ or "").read_text(encoding="utf-8")
        assert "run_scenario" not in source, (
            f"{module.__name__} now routes through the authoring run harness; the "
            f"reversal gate would start judging a frozen scenario that reverses "
            f"legitimately. Re-read errors.ReversedFlowError before proceeding."
        )

    biosphere_types = [n for n in FLOW_TYPES if n.startswith("biosphere.")]
    assert biosphere_types == [], (
        f"the biosphere is now author-reachable ({biosphere_types}), so an authored "
        f"scenario can put a crop on the far side of the O2 regulator. The reversal "
        f"gate's premise — 'every authored reversal is a mis-wiring' — has expired and "
        f"must be re-decided, not merely re-passed."
    )
