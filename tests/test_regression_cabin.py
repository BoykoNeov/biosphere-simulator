"""Regression-snapshot gate: the golden Crew ↔ ECLSS cabin gas-loop run (P6.2).

Pins ``CABIN_GAS_SCENARIO`` (the real crew respiring into / breathing from the ECLSS
cabin, Euler, over ``CABIN_GAS_STEPS``) bit-exactly. The **final coupled State** is
serialized via the ``sim_io`` hex-float serializer and byte-compared to a committed
golden, so any bit change in the coupled trajectory (the ``CrewRespiration`` law, the
composition wiring, the ECLSS control coefficients, the crew split fractions, the
scenario sizing, the reduction order) surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the station's own Step-2 regression pin, the sibling of the Step-1
``station_state.json`` heat golden; the seven frozen reference goldens + two demo
goldens + the four sibling goldens (two Power, one Thermal, one ECLSS, one Crew) + the
Step-1 station golden are untouched and byte-identical (the cabin is a *separate*
assembly — zero domain change, zero core change).

Mirrors ``test_regression_station`` / ``test_regression_eclss`` (the additive-scenario
discipline): full ``State`` via ``sim_io.dumps``, Euler only, regeneration a separate
explicit ``__main__`` action. The generator bakes in a **pre-golden gate specific to
Step 2's purpose**: it asserts the run is well-fed (``rationed == 0``), event-free
(``events == ()``), that **all three quantities (CARBON / OXYGEN / WATER) balance every
step** (residual ≈ 0 — the gas-closure payload), that **O₂ was genuinely drawn from the
cabin** (``cabin_o2`` ended below the setpoint), and that **each species reached its
steady state** — so the golden is **impossible to regenerate from a degenerate run** (an
imbalance / an undrawn cabin / a non-converged trajectory fails the gate, not silently
re-freezes; the "the gas loop actually closed OXYGEN" analogue of the N-limited ``f_N``
gate). The composition annotation being load-bearing (the decoupled version raising
``ConservationError``) is pinned in ``test_cabin_run``.

**Bit-stability caveat** (as for the biosphere / Power / Thermal / ECLSS / station
goldens): the flows use only +−×÷ (no transcendental), so this golden is bit-identical
within a build; regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import pytest

import sim_io
from domains.crew.loader import load_crew_params
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import CABIN_CO2, CABIN_H2O, CABIN_O2
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.cabin import build_cabin, cabin_resolver, cabin_steady_state
from station.scenario import CABIN_GAS_SCENARIO, CABIN_GAS_STEPS
from station.system import run_station

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "cabin_gas_state.json"

_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_SCENARIO = CABIN_GAS_SCENARIO
_DT = _SCENARIO.dt_seconds
_STEPS = CABIN_GAS_STEPS
_SS = cabin_steady_state(_CREW, _ECLSS, _SCENARIO)

_LEDGER_ABS_TOL = 1e-9
_EQ_BAND = 1e-3
_MASS_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER)


def _final_state() -> State:
    """Run the canonical cabin gas-loop run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` /
    ``events == ()`` trajectory in which **all three quantities balance every step**
    (residual ≈ 0 — gas closure), **O₂ was drawn from the cabin** (``cabin_o2`` below
    the setpoint), and **each species reached its steady state** — so a future imbalance
    / undrawn-cabin / non-converged regression fails here rather than silently
    re-freezing a degenerate run.
    """
    state, registry = build_cabin(_CREW, _ECLSS, _SCENARIO)
    resolver = cabin_resolver(_SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden cabin run must be well-fed (no arbitration)"
    assert events == (), "golden cabin run must be event-free (no POPULATION stock)"
    # The payload: every step's CARBON / OXYGEN / WATER ledgers balance to round-off.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in _MASS_QUANTITIES:
            assert abs(ledger[quantity].residual) <= _LEDGER_ABS_TOL, (
                "golden cabin run must keep every quantity closed every step — an"
                " imbalanced trajectory must not be pinnable as this golden"
            )
    final = states[-1]
    # O₂ genuinely drawn from the cabin (the crew breathes cabin air, not a store).
    assert final.stocks[CABIN_O2].amount < _ECLSS.o2_setpoint, (
        "golden cabin run must draw cabin O₂ below the setpoint"
    )
    # Each species reached its emergent steady state (a non-converged run fails here).
    assert final.stocks[CABIN_O2].amount == pytest.approx(_SS.cabin_o2, abs=_EQ_BAND)
    assert final.stocks[CABIN_CO2].amount == pytest.approx(_SS.cabin_co2, abs=_EQ_BAND)
    assert final.stocks[CABIN_H2O].amount == pytest.approx(_SS.cabin_h2o, abs=_EQ_BAND)
    return final


def test_cabin_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the coupled
    # output fails here (within-build; the flows are transcendental-free).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_cabin_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (also the check
    # that sim_io serializes a combined multi-quantity, composition-carrying State
    # cleanly).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed cabin golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_cabin.py

    Review the diff before committing: a change here means the coupled output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
