"""Regression-snapshot gate: the golden crew water-recovery run (P6.4).

Pins ``WATER_RECOVERY_SCENARIO`` (the crew water loop closed by the ``recovered_water``
buffer + the station-owned ``WaterRecovery`` flow, Euler, over ``WATER_RECOVERY_STEPS``)
bit-exactly. The **final coupled State** is serialized via the ``sim_io`` hex-float
serializer and byte-compared to a committed golden, so any bit change in the coupled
trajectory (the ``WaterRecovery`` law, the re-pointed Condenser/urine sinks, the
recovery params, the crew split fractions, the ECLSS coefficients, the sizing, the
reduction order) surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the station's own Step-4 regression pin, the sibling of the Step-1
``station_state.json`` heat golden and the Step-2 ``cabin_gas_state.json`` gas golden;
the seven frozen reference goldens + two demo goldens + the sibling/station goldens are
untouched and byte-identical (Step 4 is a *separate* assembly — zero domain change, zero
core change).

Mirrors ``test_regression_cabin`` (the additive-scenario discipline): full ``State`` via
``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action. The
generator bakes in a **pre-golden gate specific to Step 4's purpose**: it asserts the
run is well-fed (``rationed == 0``), event-free (``events == ()``), that **all three
quantities (CARBON / OXYGEN / WATER) balance every step** (the closure payload), that
``water_store`` **regenerated** (ended above the open-loop η_w = 0 baseline — the "it
bit" check, the analogue of the cabin "O₂ drawn below the setpoint" gate), and that the
two WATER pools **reached their steady states** — so the golden is **impossible to
regenerate from a degenerate run** (an imbalance / a non-regenerating store / a
non-converged trajectory fails the gate, not silently re-freezes).

**Bit-stability caveat** (as for the biosphere / Power / Thermal / ECLSS / station /
cabin goldens): the flows use only +−×÷ (no transcendental), so this golden is
bit-identical within a build; regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import pytest

import sim_io
from domains.crew.loader import load_crew_params
from domains.crew.stocks import WATER_STORE
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import CABIN_H2O
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.flows import WaterRecoveryParams
from station.loader import load_water_recovery_params
from station.scenario import WATER_RECOVERY_SCENARIO, WATER_RECOVERY_STEPS
from station.system import run_station
from station.water import (
    RECOVERED_WATER,
    build_water_recovery,
    water_recovery_resolver,
    water_recovery_steady_state,
)

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "water_recovery_state.json"

_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_WR = load_water_recovery_params()
_SCENARIO = WATER_RECOVERY_SCENARIO
_DT = _SCENARIO.dt_seconds
_STEPS = WATER_RECOVERY_STEPS
_SS = water_recovery_steady_state(_CREW, _ECLSS, _WR, _SCENARIO)

_LEDGER_ABS_TOL = 1e-9
_EQ_BAND = 1e-3
_MASS_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER)


def _run(recovery_efficiency: float) -> list[State]:
    params = WaterRecoveryParams(
        recovery_rate=_WR.recovery_rate, recovery_efficiency=recovery_efficiency
    )
    state, registry = build_water_recovery(_CREW, _ECLSS, params, _SCENARIO)
    resolver = water_recovery_resolver(_SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden water-recovery run must be well-fed (no arbitration)"
    assert events == (), "golden water-recovery run must be event-free"
    return states


def _final_state() -> State:
    """Run the canonical water-recovery run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: every step's CARBON / OXYGEN / WATER ledgers balance
    (closure), the ``water_store`` regenerated above the open-loop (η_w = 0) baseline
    (the "it bit" check), and the two WATER pools reached their steady states — a future
    imbalance / non-regenerating / non-converged regression fails here, not re-freezes.
    """
    states = _run(_WR.recovery_efficiency)
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in _MASS_QUANTITIES:
            assert abs(ledger[quantity].residual) <= _LEDGER_ABS_TOL, (
                "golden water-recovery run must keep every quantity closed every step —"
                " an imbalanced trajectory must not be pinnable as this golden"
            )
    final = states[-1]
    # The store regenerated: it ended materially above the open-loop (η_w = 0) baseline.
    baseline_final = _run(0.0)[-1]
    assert (
        final.stocks[WATER_STORE].amount > baseline_final.stocks[WATER_STORE].amount
    ), "golden run must regenerate water_store above the open-loop baseline"
    # The two WATER pools reached their steady states (a non-converged run fails here).
    assert final.stocks[CABIN_H2O].amount == pytest.approx(_SS.cabin_h2o, abs=_EQ_BAND)
    assert final.stocks[RECOVERED_WATER].amount == pytest.approx(
        _SS.recovered_water, abs=_EQ_BAND
    )
    return final


def test_water_recovery_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the coupled
    # output fails here (within-build; the flows are transcendental-free).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_water_recovery_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State.
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed water-recovery golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_water_recovery.py

    Review the diff before committing: a change here means the coupled output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
