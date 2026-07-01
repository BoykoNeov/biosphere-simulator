"""Regression-snapshot gate: the golden coupled Power → Thermal station run (P6.1).

Pins ``HEAT_CLOSURE_SCENARIO`` (Power's daily-balanced microgrid feeding the Thermal
node, the node started at the equilibrium its mean dissipation implies, Euler, over
``HEAT_CLOSURE_DAYS``) bit-exactly. The **final combined State** is serialized via the
``sim_io`` hex-float serializer and byte-compared to a committed golden, so any bit
change in the coupled trajectory (a Power/Thermal flow law, the seam wiring, the σ
constant, the charge/radiator params, the reduction order, the scenario sizing) surfaces
here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the station's own Step-1 regression pin, the sibling of the Power /
Thermal domain goldens; the seven frozen reference goldens + two demo goldens + the four
sibling goldens (two Power, one Thermal, one ECLSS) + the Crew golden are untouched and
byte-identical (the station is a *separate* assembly — zero domain change, zero core
change).

Mirrors ``test_regression_thermal`` (the additive-scenario discipline): full ``State``
via ``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action.
The generator bakes in a **pre-golden gate specific to Step 1's purpose**: it asserts
the run is well-fed (``rationed == 0``), event-free (``events == ()``), that the
**per-step COMBINED ENERGY ledger balances** (residual ≈ 0 — the cross-domain
energy-closure payload), that the seam is **structural** (``boundary.waste_heat`` /
``boundary.heat_source`` absent — heat is not in a shadow sink), and that the node
**stayed at the dissipation-set equilibrium** (``T`` within a narrow band of the
predicted ``T_eq``) — so the golden is **impossible to regenerate from a degenerate
run** (an imbalance / a leaked shadow sink / a runaway node fails the gate, not silently
re-freezes a broken trajectory; the "the seam actually closed energy" analogue of the
N-limited ``f_N`` gate).

**Bit-stability caveat** (as for the biosphere / Power / Thermal goldens): the Power
``solar_schedule`` uses ``math.sin``/``math.pi`` and the radiator uses ``T⁴`` / a
fractional power, none IEEE-754-mandated correctly-rounded, so this golden is
bit-identical **within a build** but cross-platform last-ULP differences are tolerance
territory. Regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import sim_io
from domains.power.loader import load_charge_params
from domains.power.stocks import WASTE_HEAT
from domains.thermal.flows import temperature
from domains.thermal.loader import load_thermal_params
from domains.thermal.stocks import HEAT_SOURCE, NODE
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.scenario import HEAT_CLOSURE_DAYS, HEAT_CLOSURE_SCENARIO
from station.system import (
    build_station,
    predicted_equilibrium_temperature,
    run_station,
    station_resolver,
)

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "station_state.json"

_CHARGE = load_charge_params()
_THERMAL = load_thermal_params()
_SCENARIO = HEAT_CLOSURE_SCENARIO
_SPD = _SCENARIO.power.steps_per_day
_DT = _SCENARIO.power.dt_seconds
_STEPS = HEAT_CLOSURE_DAYS * _SPD
_T_EQ = predicted_equilibrium_temperature(_CHARGE, _THERMAL, _SCENARIO)

_LEDGER_ABS_TOL = 1e-6
# T stays within this of the mean-power T_eq (≈0.001 K achieved — started at
# equilibrium).
_EQ_BAND_K = 1.0


def _final_state() -> State:
    """Run the canonical heat-closure station run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / ``events ==
    ()`` trajectory in which the **per-step combined ENERGY ledger balances** (residual
    ≈ 0 — cross-domain energy closure), the seam is **structural** (no
    ``boundary.waste_heat`` / ``boundary.heat_source`` shadow sink), and the node
    **stayed at the dissipation-set equilibrium** (``T`` within ``_EQ_BAND_K`` of the
    predicted ``T_eq``) — so a future imbalance / shadow-sink / runaway regression fails
    here rather than silently re-freezing a degenerate run.
    """
    state, registry = build_station(_CHARGE, _THERMAL, _SCENARIO)
    resolver = station_resolver(_CHARGE, _SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden station run must be well-fed (no arbitration)"
    assert events == (), "golden station run must be event-free (no POPULATION stock)"
    # The seam is structural: dissipation was redirected into the node, not a shadow
    # sink.
    assert WASTE_HEAT not in states[0].stocks, (
        "golden station run must not carry a boundary.waste_heat shadow sink"
    )
    assert HEAT_SOURCE not in states[0].stocks, (
        "golden station run must not carry a boundary.heat_source forcing stand-in"
    )
    # The payload: every step's COMBINED ENERGY ledger balances to round-off.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= _LEDGER_ABS_TOL, (
            "golden station run must keep ENERGY closed every step (both domains) —"
            " an imbalanced trajectory must not be pinnable as this golden"
        )
    # The node stayed at the dissipation-set equilibrium (the radiator carried the real
    # load). A runaway / collapsed / mis-sized node fails here.
    t_final = temperature(
        states[-1].stocks[NODE].amount,
        heat_capacity=_THERMAL.heat_capacity,
        space_temperature=_THERMAL.space_temperature,
    )
    assert abs(t_final - _T_EQ) < _EQ_BAND_K, (
        "golden station run must hold the node at the dissipation-set equilibrium"
    )
    return states[-1]


def test_station_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the coupled
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_station_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (also the check
    # that sim_io serializes a combined two-domain pure-ENERGY State cleanly).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed station golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_station.py

    Review the diff before committing: a change here means the coupled output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
