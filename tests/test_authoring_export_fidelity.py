"""Post-roadmap bucket 2: ``rationed == 0`` does not mean the EXPORT is right.

**The finding this file pins, and why it is not the ``dt`` hazard.**
``tests/test_authoring_dt_hazard.py`` pins the *donor-controlled* failure: at
``k_scrub·dt > 1`` the scrub draw exceeds the whole stock, the Euler backstop rations,
and (post-fix) ``run_scenario`` raises. That hazard is **caught**.

This file pins the *demand-controlled* one, which is **not caught, by construction**:

  * ``eclss.o2_makeup`` is the only DEMAND-controlled flow in the registry. Its draw is
    proportional to the **setpoint error** ``(o2_setpoint − cabin_o2)``, not to the
    stock. Near the setpoint that draw is SMALL, so it never over-draws ``cabin_o2``
    and **the backstop never fires**.
  * A donor-controlled flow's draw is proportional to **the stock itself**, so
    ``k·dt > 1`` demands more than exists — an over-draw the backstop MUST scale.

That asymmetry is the whole finding. Rationing detects **large excursions that
over-draw**. It is structurally blind to **near-setpoint oscillation that stays
positive** — which is precisely the case that corrupts a cross-domain export.

**Why an oscillating export is a failure even though it converges.** For
``1 ≤ k_makeup·dt < 2`` the controller still reaches the correct equilibrium: the
ENDPOINT is right. But the INTERMEDIATE values oscillate around the setpoint
(measured at ``dt = 900``: ``12 → 8.4 → 11.3 → 8.98 → 10.8``), and in a coupled station
those intermediates are **exported to other domains every step**. A neighbour reading
``cabin_o2`` sees oxygen sloshing ±20 % that the real cabin never does. Converging to
the right answer eventually does not license exporting wrong answers on the way.

**This is why the frozen ``k·dt < 1`` bound in docs/authoring-reference.md is the
OPERATIVE one and must not be relaxed to the textbook stability bound ``k·dt < 2``.**
``< 2`` is the correct bound for *"does the solver diverge"*. ``< 1`` is the correct
bound for *"is the exported trajectory monotone and therefore usable by a neighbour"*.
This project couples domains, so ``< 1`` governs. See that doc's ``dt`` table.

**Transcendental discipline** (the cross-libm trap, ``test_oracle_gap.py``): every pin
below asserts on the **Euler recurrence**, which is ``+ − ×`` only and therefore
bit-identical across ports and platforms. The closed-form ``exp`` solution appears
ONLY as a loosely-toleranced sanity reference, never as an exact assertion.
"""

import copy
import math
from pathlib import Path
from typing import Any

import pytest

from authoring.errors import RationedError
from authoring.interpreter import interpret
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.ids import StockId

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"

# The frozen eclss.yaml values, restated so the arithmetic below is legible. The tests
# that depend on them read the loader; these are for the comments' benefit.
K_MAKEUP = 2.0e-3  # o2_makeup_gain (1/s)  — DEMAND-controlled
K_SCRUB = 1.0e-3  # co2_scrub_rate (1/s)  — DONOR-controlled
SETPOINT = 10.0  # o2_setpoint (mol)
CON_O2 = 0.004  # forced crew o2_consumption (mol/s), scenario data

# o2_eq = o2_setpoint − Con_o2/k_makeup. Pure arithmetic — no transcendental.
O2_EQ = SETPOINT - CON_O2 / K_MAKEUP  # == 8.0

# The four regimes of k_makeup·dt, as dt (seconds).
DT_MONOTONE = 60.0  # k·dt = 0.12  — the frozen dt; monotone, export-clean
DT_OSCILLATING = 900.0  # k·dt = 1.80  — converges, but exports oscillation
DT_PERPETUAL = 1000.0  # k·dt = 2.00  — never converges; undamped forever
DT_DIVERGENT = 1200.0  # k·dt = 2.40  — diverges until something over-draws


def _build(raw: dict[str, Any]) -> Any:
    return interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)


def _full_cabin(dt: float, steps: int, cabin_o2_0: float) -> Any:
    """The committed ECLSS anchor with dt/steps/initial-O2 overridden.

    Mutates a parsed copy rather than shipping near-duplicate scenario files — the
    ``test_authoring_dt_hazard._build_at`` idiom, extended with the initial amount
    because *starting above the setpoint* is the case this file exists to reach.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = dt
    raw["steps"] = steps
    for stock in raw["stocks"]:
        if stock["id"] == "eclss.cabin_o2":
            stock["amount"] = cabin_o2_0
    return _build(raw)


def _makeup_only(dt: float, steps: int, cabin_o2_0: float) -> Any:
    """``eclss.o2_makeup`` ALONE — no scrubber, no condenser, no crew.

    Isolating the controller is what removes the *coincidental* protection: the
    scrubber's own ``k_scrub·dt > 1`` gate happens to fire near the same dt, so a full
    cabin often raises for a reason that has nothing to do with the makeup loop. An
    author is free to register this flow on its own, and then no gate stands between
    them and the oscillation.
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
    return _build(raw)


def _o2(states: list) -> list[float]:
    return [s.stocks["eclss.cabin_o2"].amount for s in states]


def _supply(states: list) -> list[float]:
    return [s.stocks["boundary.o2_supply"].amount for s in states]


# --------------------------------------------------------------------------------------
# 1. The reversal — asserted in prose in THREE places, measured in none until now.
# --------------------------------------------------------------------------------------


def test_above_the_setpoint_the_flow_reverses_exactly_as_documented() -> None:
    """The "deferred venting seam" is real, and the prose describing it is correct.

    ``domains/eclss/flows.py`` ("an above-setpoint venting clamp is a deferred seam"),
    ``authoring/flow_registry.py`` ("above the setpoint the frozen law goes NEGATIVE")
    and ``docs/authoring-reference.md`` ("the flow silently reverses") all claim this.
    None of them measured it. It reverses, it conserves, and it does not ration.
    """
    states, rationed, _events = run_scenario(_makeup_only(DT_MONOTONE, 4, 20.0))
    o2, supply = _o2(states), _supply(states)

    # Step 1: S = k·dt·(setpoint − cabin_o2) = 0.12 · (10 − 20) = −1.2 mol. Exact
    # arithmetic (+ − ×), so this is an equality, not an approximation.
    assert o2[1] - o2[0] == pytest.approx(-1.2, rel=1e-12)
    assert supply[1] - supply[0] == pytest.approx(+1.2, rel=1e-12)

    # The direction is genuinely reversed: cabin → tank, the opposite of "makeup".
    assert supply[1] > supply[0], "O2 flowed cabin → tank"
    assert o2[1] < o2[0], "the cabin LOST oxygen to a flow named 'makeup'"

    # And nothing anywhere objects.
    assert rationed == 0


def test_the_reversal_conserves_which_is_why_no_gate_sees_it() -> None:
    """Reversal is invisible to every existing gate: OXYGEN balances to the last digit.

    This is the transferable half of the finding. The conservation ledger cannot
    distinguish "the regulator supplied O2" from "the regulator took O2 back" — both
    balance. Conservation is a *stoichiometry* check, never a *plausibility* one.
    """
    states, rationed, _events = run_scenario(_makeup_only(DT_MONOTONE, 20, 20.0))
    o2, supply = _o2(states), _supply(states)
    total_0 = o2[0] + supply[0]
    for i in range(len(o2)):
        assert o2[i] + supply[i] == pytest.approx(total_0, rel=1e-12)
    assert rationed == 0


# --------------------------------------------------------------------------------------
# 2. The user's principle: the ANSWER is dt-independent. Only the path is not.
# --------------------------------------------------------------------------------------


def test_the_equilibrium_is_dt_independent_across_a_500x_span() -> None:
    """Every stable dt reaches the SAME equilibrium — dt cancels at the fixed point.

    ``0 = k·(S − x*) − Con  ⇒  x* = S − Con/k``. There is no ``dt`` in the answer, and
    this is why "a coarse dt must not change the result" is a *correct* principle about
    the equilibrium even though it cannot hold for the trajectory.
    """
    duration = 3600.0 * 20  # 20 h — long enough for every dt below to converge
    for dt in (1.0, 10.0, 60.0, 120.0, 300.0):
        steps = int(duration / dt)
        states, rationed, _events = run_scenario(_full_cabin(dt, steps, 20.0))
        assert rationed == 0
        # Converged to o2_eq = 8.0 regardless of dt, to within 1e-9 mol.
        assert _o2(states)[-1] == pytest.approx(O2_EQ, abs=1e-9), f"dt={dt}"


def test_a_forced_flow_is_exactly_dt_invariant() -> None:
    """The dt-invariant half the project already has — and the reason it is different.

    ``eclss.crew_metabolism`` is FORCED: ``flux = rate·dt``, so over T seconds it moves
    ``rate·dt·(T/dt) = rate·T`` at **every** dt. A forced flow genuinely satisfies "big
    dt == small dt". A stock-reading flow cannot, which is exactly what the frozen
    docstrings mean by "because it reads a stock, RK4 ≢ Euler".
    """
    duration = 3600.0 * 20
    expected = CON_O2 * duration  # 288.0 mol — pure arithmetic
    for dt in (1.0, 60.0, 300.0):
        steps = int(duration / dt)
        states, _r, _e = run_scenario(_full_cabin(dt, steps, 20.0))
        sink = states[-1].stocks[StockId("boundary.metabolic_o2_sink")].amount
        assert sink == pytest.approx(expected, rel=1e-9), f"dt={dt}"


# --------------------------------------------------------------------------------------
# 3. THE FINDING: the oscillating band exports garbage, and rationing cannot see it.
# --------------------------------------------------------------------------------------


def test_below_the_monotonicity_bound_the_export_is_monotone() -> None:
    """``k·dt < 1``: the exported trajectory falls smoothly. This is the safe regime."""
    assert K_MAKEUP * DT_MONOTONE < 1.0
    states, rationed, _events = run_scenario(_makeup_only(DT_MONOTONE, 40, 20.0))
    o2 = _o2(states)
    assert rationed == 0
    # Strictly decreasing, every step, all the way down. Nothing to corrupt a neighbour.
    for a, b in zip(o2[:-1], o2[1:], strict=True):
        assert b < a, "monotone approach from above"
    assert min(o2) >= SETPOINT, "never overshoots below the setpoint"


def test_the_oscillating_band_exports_NON_monotone_intermediates() -> None:
    """``1 ≤ k·dt < 2``: converges to the right answer, exports the wrong path.

    **This is the pin the whole file exists for.** The endpoint is correct, so an
    endpoint-only check (and ``rationed``, and conservation) all pass — while the
    exported intermediates oscillate across the setpoint every single step.
    """
    assert 1.0 <= K_MAKEUP * DT_OSCILLATING < 2.0
    states, rationed, _events = run_scenario(_makeup_only(DT_OSCILLATING, 12, 12.0))
    o2 = _o2(states)

    # Nothing complains.
    assert rationed == 0

    # But the export crosses the setpoint back and forth rather than approaching it.
    # Measured: 12 → 8.4 → 11.28 → 8.976 → 10.8 ... (exact, + − × only).
    assert o2[1] == pytest.approx(8.4, rel=1e-12)
    assert o2[2] == pytest.approx(11.28, rel=1e-12)
    assert o2[3] == pytest.approx(8.976, rel=1e-12)

    # The formal statement: the step sign CHANGES — a monotone approach never does.
    signs = [(b - a) > 0 for a, b in zip(o2[:-1], o2[1:], strict=True)]
    assert any(signs) and not all(signs), "oscillates: both up-steps and down-steps"

    # And it really does straddle the setpoint, which is what a neighbour reads.
    assert min(o2) < SETPOINT < max(o2)


def test_the_oscillating_band_still_converges_which_is_why_it_hides() -> None:
    """The band is dangerous *because* it looks fine by every check we have.

    Endpoint correct + conserving + ``rationed == 0``. Only the intermediates are wrong,
    and no gate in the project reads intermediates.
    """
    states, rationed, _events = run_scenario(_makeup_only(DT_OSCILLATING, 200, 12.0))
    o2 = _o2(states)
    assert rationed == 0
    assert o2[-1] == pytest.approx(SETPOINT, abs=1e-6), "endpoint is RIGHT"


def test_at_exactly_two_the_oscillation_never_decays() -> None:
    """``k·dt == 2``: ``error → (1−2)·error = −error``. Undamped, forever.

    The sharpest case: not a transient, not a divergence — a permanent ±2 mol square
    wave about the setpoint, conserving, unrationed, and reported as a clean run.
    """
    assert K_MAKEUP * DT_PERPETUAL == 2.0
    states, rationed, _events = run_scenario(_makeup_only(DT_PERPETUAL, 200, 12.0))
    o2 = _o2(states)
    assert rationed == 0
    # Still swinging at full amplitude after 200 steps — 12 ↔ 8, exactly.
    assert o2[0] == pytest.approx(12.0, rel=1e-12)
    assert o2[1] == pytest.approx(8.0, rel=1e-12)
    assert o2[-2] == pytest.approx(8.0, rel=1e-12)
    assert o2[-1] == pytest.approx(12.0, rel=1e-12)
    assert max(o2[-20:]) - min(o2[-20:]) == pytest.approx(4.0, rel=1e-12)


def test_the_divergent_band_eventually_over_draws_and_IS_caught() -> None:
    """``k·dt > 2``: diverges until an excursion over-draws — then rationing fires.

    The boundary of what the gate can see: rationing catches this one, but only
    *after* the divergence grows large enough to demand more than the stock holds. It
    is the amplitude that trips it, never the oscillation.
    """
    assert K_MAKEUP * DT_DIVERGENT > 2.0
    with pytest.raises(RationedError):
        run_scenario(_makeup_only(DT_DIVERGENT, 8, 12.0))


# --------------------------------------------------------------------------------------
# 4. The mechanism: WHY donor-controlled is caught and demand-controlled is not.
# --------------------------------------------------------------------------------------


def test_donor_controlled_over_draws_and_is_caught_demand_controlled_is_not() -> None:
    """The asymmetry, stated as a test — the reason this is not the ``dt`` hazard.

    A DONOR-controlled draw is ``k·dt·stock``: at ``k·dt > 1`` it demands more than the
    whole stock, every time, so the backstop MUST fire. A DEMAND-controlled draw is
    ``k·dt·(setpoint − stock)``: near the setpoint it is small however large ``k·dt``
    gets, so it never over-draws and the backstop never fires.
    """
    # Donor-controlled (the scrubber) past ITS bound: caught.
    dt_scrub_unsafe = 1200.0
    assert K_SCRUB * dt_scrub_unsafe > 1.0
    with pytest.raises(RationedError):
        run_scenario(_full_cabin(dt_scrub_unsafe, 8, 10.0))

    # Demand-controlled (the makeup) past ITS monotonicity bound, at a dt where the
    # scrubber is still SAFE (k_scrub·dt = 0.9 < 1): NOT caught.
    assert K_SCRUB * DT_OSCILLATING < 1.0 < K_MAKEUP * DT_OSCILLATING
    _states, rationed, _events = run_scenario(_makeup_only(DT_OSCILLATING, 12, 12.0))
    assert rationed == 0, "the oscillation is invisible to the gate"


def test_the_full_cabin_is_protected_only_by_coincidence() -> None:
    """``k_scrub·dt = 1`` and ``k_makeup·dt = 2`` both land at ``dt = 1000``.

    So a full cabin usually raises (via the SCRUBBER) before the makeup loop's own
    cliff can be observed — protection the makeup flow does not own and an author does
    not inherit. At ``dt = 1000`` exactly, the scrub draw *equals* the stock rather than
    exceeding it, the coincidence fails, and the cabin oscillates while reporting
    success.
    """
    assert K_SCRUB * DT_PERPETUAL == 1.0
    assert K_MAKEUP * DT_PERPETUAL == 2.0

    states, rationed, _events = run_scenario(_full_cabin(DT_PERPETUAL, 200, 12.0))
    o2 = _o2(states)
    assert rationed == 0, "the full cabin does NOT raise at dt = 1000"
    # An 8 mol square wave against a 10 mol setpoint, forever, reported as a clean run.
    assert max(o2[-20:]) - min(o2[-20:]) == pytest.approx(8.0, rel=1e-12)


# --------------------------------------------------------------------------------------
# 5. The export criterion, stated against the closed form (LOOSE — transcendental).
# --------------------------------------------------------------------------------------


def test_convergence_is_first_order_in_dt_which_is_what_correct_looks_like() -> None:
    """Halving dt halves the error: the signature of a correct first-order scheme.

    Guards the claim that the oscillation is a SOLVER property, not broken physics —
    a broken model does not track its own closed-form solution with textbook O(dt)
    error. ``exp`` appears here only as a loosely-toleranced reference (the cross-libm
    discipline of ``test_oracle_gap.py``): the assertion is on the error RATIO, which
    is what carries the meaning.
    """

    def analytic(t: float) -> float:
        return O2_EQ + (20.0 - O2_EQ) * math.exp(-K_MAKEUP * t)

    t_probe = 1800.0
    errors = {}
    for dt in (10.0, 20.0, 40.0):
        states, _r, _e = run_scenario(_full_cabin(dt, int(t_probe / dt), 20.0))
        errors[dt] = abs(_o2(states)[-1] - analytic(t_probe))

    # First-order ⇒ error roughly doubles as dt doubles. Loose (transcendental ref).
    assert errors[20.0] / errors[10.0] == pytest.approx(2.0, rel=0.05)
    assert errors[40.0] / errors[20.0] == pytest.approx(2.0, rel=0.05)
