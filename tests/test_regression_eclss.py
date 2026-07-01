"""Regression-snapshot gate: the golden standalone ECLSS run (Step 6, multi-quantity).

Pins ``STEADY_STATE_SCENARIO`` (the clean cabin relaxing to its emergent per-species
steady states under the three ECLSS control loops, Euler, over ``STEADY_STATE_STEPS``)
bit-exactly. The **final State** is serialized via the ``sim_io`` hex-float serializer
and byte-compared to a committed golden, so any bit change in the ECLSS trajectory (a
flow law, a rate constant, the reduction order, ``eclss.yaml``, the scenario sizing)
surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the ECLSS domain's own regression pin, the sibling of the Power /
Thermal goldens (``power_state.json`` / ``thermal_state.json``); the seven frozen
reference goldens + two demo goldens + the two Power goldens + the Thermal golden are
untouched and byte-identical.

Mirrors ``test_regression_thermal`` (the additive-scenario discipline): full ``State``
via ``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action.
The generator bakes in a **pre-golden gate specific to ECLSS's purpose**: it asserts the
run is well-fed (``rationed == 0``), event-free (``events == ()``), that **all three
quantities (CARBON / OXYGEN / WATER) balance every step** (residual ≈ 0 — the
first-multi-quantity-sibling payload, the analogue of Thermal's "ENERGY closed"), and
that each species **actually reached its emergent steady state** (within a narrow band
of ``steady_state``) — so the golden is **impossible to regenerate from a degenerate
run** (an imbalance/never-converged regression fails the gate, not silently re-freezes
a broken trajectory; the "the control loops actually bit" analogue of the N-limited
``f_N`` gate).

**Bit-stability caveat** (as for the biosphere / Power / Thermal goldens): the flows use
plain float arithmetic (no transcendentals here — the ECLSS laws are polynomial), so
this golden is bit-identical **within a build**; cross-platform last-ULP differences
remain tolerance territory. Regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import sim_io
from domains.eclss.loader import load_eclss_params
from domains.eclss.scenario import STEADY_STATE_SCENARIO, STEADY_STATE_STEPS
from domains.eclss.stocks import CABIN_CO2, CABIN_H2O, CABIN_O2
from domains.eclss.system import build_eclss, eclss_resolver, run_eclss, steady_state
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "eclss_state.json"

_PARAMS = load_eclss_params()
_SCENARIO = STEADY_STATE_SCENARIO
_STEPS = STEADY_STATE_STEPS
_DT = _SCENARIO.dt_seconds
_SS = steady_state(_PARAMS, _SCENARIO)

_LEDGER_ABS_TOL = 1e-6
_EQ_BAND = 1e-6  # each species within this of its steady state (achieved << this)


def _final_state() -> State:
    """Run the canonical steady-state ECLSS run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / ``events ==
    ()`` trajectory in which **all three quantities balance every step** (residual ≈ 0 —
    the payload) and each species **reached its emergent steady state** (within
    ``_EQ_BAND``) — so a future imbalance/never-converged regression fails here rather
    than silently re-freezing a degenerate run.
    """
    state, registry = build_eclss(_PARAMS, _SCENARIO)
    resolver = eclss_resolver(_SCENARIO)
    states, rationed, events = run_eclss(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden ECLSS run must be well-fed (no arbitration)"
    assert events == (), "golden ECLSS run must be event-free (no POPULATION stock)"
    # The payload: every step's augmented ledger balances all three mass quantities.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for q in (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER):
            assert abs(ledger[q].residual) <= _LEDGER_ABS_TOL, (
                "golden ECLSS run must keep every quantity closed every step "
                "(an imbalanced trajectory must not be pinnable as this golden)"
            )
    # The emergent attractors actually formed: each species reached its steady-state
    # band (the control loops "bit"). A degenerate/never-converged run fails here.
    final = states[-1]
    assert abs(final.stocks[CABIN_CO2].amount - _SS.cabin_co2) < _EQ_BAND
    assert abs(final.stocks[CABIN_H2O].amount - _SS.cabin_h2o) < _EQ_BAND
    assert abs(final.stocks[CABIN_O2].amount - _SS.cabin_o2) < _EQ_BAND
    return final


def test_eclss_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the ECLSS
    # output fails here (within-build; see the caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_eclss_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (also the check
    # that sim_io serializes a three-mass-quantity ECLSS State cleanly).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed ECLSS golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_eclss.py

    Review the diff before committing: a change here means the ECLSS output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
