"""Regression-snapshot gate: the golden standalone Thermal run (Step 5, energy closure).

Pins ``EQUILIBRIUM_SCENARIO`` (the cold node warming to its emergent equilibrium
temperature under Stefan-Boltzmann radiation, Euler, over ``EQUILIBRIUM_STEPS``)
bit-exactly. The **final State** is serialized via the ``sim_io`` hex-float serializer
and byte-compared to a committed golden, so any bit change in the Thermal trajectory (a
flow law, the σ constant, the radiator params, the reduction order, ``radiator.yaml``,
the scenario sizing) surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the Thermal domain's own regression pin, the sibling of the Power
golden (``power_state.json``); the seven frozen reference goldens + two demo goldens +
the two Power goldens are untouched and byte-identical.

Mirrors ``test_regression_power`` (the additive-scenario discipline): full ``State`` via
``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action. The
generator bakes in a **pre-golden gate specific to Thermal's purpose**: it asserts the
run is well-fed (``rationed == 0`` — the ``τ >> dt`` sizing held), event-free (``events
== ()``), that the **per-step ENERGY ledger balances** (residual ≈ 0 — the Phase-5
energy-closure payload), and that the node **actually reached the emergent equilibrium**
(``T`` within a narrow band of ``T_eq``, having climbed most of the way from the floor)
— so the golden is **impossible to regenerate from a degenerate run** (an
imbalance/never-converged regression fails the gate, not silently re-freezes a broken
trajectory; the "the radiator actually bit" analogue of the N-limited ``f_N`` gate).

**Bit-stability caveat** (as for the biosphere / Power goldens): ``radiated_power`` uses
``**`` (``T⁴``) and a fractional power in ``equilibrium_temperature``, which IEEE-754
does not mandate correctly-rounded, so this golden is bit-identical **within a build**
but cross-platform last-ULP differences are tolerance territory. Regenerate (review the
diff) if the toolchain moves.
"""

from pathlib import Path

import sim_io
from domains.thermal.flows import temperature
from domains.thermal.loader import load_thermal_params
from domains.thermal.scenario import EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS
from domains.thermal.stocks import NODE
from domains.thermal.system import (
    build_thermal,
    equilibrium_temperature,
    run_thermal,
    thermal_resolver,
)
from golden_platform import windows_golden_only
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "thermal_state.json"

_PARAMS = load_thermal_params()
_SCENARIO = EQUILIBRIUM_SCENARIO
_STEPS = EQUILIBRIUM_STEPS
_DT = _SCENARIO.dt_seconds
_T_EQ = equilibrium_temperature(_PARAMS, _SCENARIO)

_LEDGER_ABS_TOL = 1e-6
# T_final within this of T_eq (≈ 0.04 K achieved; comfortable margin for round-off).
_EQ_BAND_K = 0.5


def _final_state() -> State:
    """Run the canonical equilibrium Thermal run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / ``events ==
    ()`` trajectory in which the **per-step ENERGY ledger balances** (residual ≈ 0 —
    energy closure) and the node **reached the emergent equilibrium** (``T`` within
    ``_EQ_BAND_K`` of ``T_eq``, having warmed most of the way from the floor) — so a
    future imbalance/never-converged regression fails here rather than silently
    re-freezing a degenerate run.
    """
    state, registry = build_thermal(_PARAMS, _SCENARIO)
    resolver = thermal_resolver(_SCENARIO)
    states, rationed, events = run_thermal(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, (
        "golden Thermal run must be well-fed (τ >> dt, no arbitration)"
    )
    assert events == (), "golden Thermal run must be event-free (no POPULATION stock)"
    # The Phase-5 payload: every step's augmented ENERGY ledger balances to round-off.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= _LEDGER_ABS_TOL, (
            "golden Thermal run must keep ENERGY closed every step (energy closure) —"
            " an imbalanced trajectory must not be pinnable as this golden"
        )
    # The emergent attractor actually formed: T reached T_eq's band, having climbed most
    # of the way from the floor (the radiator "bit"). A degenerate/never-converged run
    # (wrong sizing, broken radiator) fails here.
    t_final = temperature(
        states[-1].stocks[NODE].amount,
        heat_capacity=_PARAMS.heat_capacity,
        space_temperature=_PARAMS.space_temperature,
    )
    assert abs(t_final - _T_EQ) < _EQ_BAND_K, (
        "golden Thermal run must reach the emergent equilibrium temperature"
    )
    assert t_final - _PARAMS.space_temperature > 0.9 * (
        _T_EQ - _PARAMS.space_temperature
    ), "golden Thermal run must have warmed most of the way from the floor"
    return states[-1]


@windows_golden_only
def test_thermal_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the Thermal
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_thermal_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (also the check
    # that sim_io serializes a pure-ENERGY Thermal State cleanly — no POPULATION/mass).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed Thermal golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_thermal.py

    Review the diff before committing: a change here means the Thermal output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
