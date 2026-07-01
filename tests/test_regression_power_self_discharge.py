"""Regression-snapshot gate: the golden self-discharge Power run (P5.5, energy closure).

Pins the **self-discharge** Power run bit-exactly: ``BOUNDED_SOC_SCENARIO`` (the
daily-balanced microgrid, Euler) with the opt-in donor-controlled ``SelfDischarge`` flow
enabled, over ``SELF_DISCHARGE_DAYS``. Because the forced part is daily-balanced, the
leak is the sole driver of the SOC's monotone decay below ``battery0``. The **final
State** is serialized via ``sim_io`` (hex-float) and byte-compared to a committed
golden, so any bit change in the leaky trajectory (the leak law,
``self_discharge.yaml``'s ``k``, the forced flows, the reduction order, the scenario
sizing) surfaces here.

**Additive, NON-frozen golden** — not in ``docs/biosphere-reference.manifest.json`` (the
frozen *biosphere* reference), and separate from ``power_state.json`` (the two-flow
``BOUNDED_SOC`` golden, which stays byte-identical — this run does not touch it). The
Power domain's own regression pin for the third flow, the ``test_regression_power`` /
additive-``n_limited`` discipline: full ``State`` via ``sim_io.dumps``, Euler only,
regeneration a separate explicit ``__main__`` action.

**Pre-golden gate — the SelfDischarge purpose baked in.** The generator asserts the run
is well-fed (``rationed == 0``), event-free (``events == ()``), that the **per-step
ENERGY ledger balances** (residual ≈ 0 — energy closure survives the third flow), and —
the "it bit" check, the analogue of the N-limited gate — that the SOC **departs
``battery0`` monotonically** at every day boundary (a run where the leak did *nothing*
would return to ``battery0`` like the two-flow golden, so it must be **unpinnable** as
this golden). It deliberately does **not** copy the two-flow golden's "returns to
``battery0`` each day" assertion — that is false here by design (the leak decays it).

**Scope (by design):** a day-boundary final-state snapshot pins the endpoint; the
*properties* (the exact geometric contraction, the forced-only contrast, energy closed
every step, monotonic heat, RK4 ≢ Euler bit-identity) live in
``test_power_self_discharge.py``. This golden guards the bytes; those tests guard the
physics.

**Bit-stability caveat** (as for the biosphere/BOUNDED_SOC goldens): ``solar_schedule``
uses ``math.sin``/``math.pi`` (not correctly-rounded per IEEE-754), so this golden is
bit-identical **within a build**; regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import sim_io
from domains.power.loader import load_charge_params, load_self_discharge_params
from domains.power.scenario import BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS
from domains.power.stocks import BATTERY
from domains.power.system import build_power, power_resolver, run_power
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "power_self_discharge_state.json"

_CHARGE = load_charge_params()
_SELF_DISCHARGE = load_self_discharge_params()
_SCENARIO = BOUNDED_SOC_SCENARIO
_STEPS = SELF_DISCHARGE_DAYS * _SCENARIO.steps_per_day
_DT = _SCENARIO.dt_seconds

_LEDGER_ABS_TOL = 1e-6


def _final_state() -> State:
    """Run the canonical self-discharge Power run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / ``events ==
    ()`` trajectory in which the **per-step ENERGY ledger balances** (residual ≈ 0 —
    energy closure survives the third flow) and the SOC **departs ``battery0``**
    at every day boundary (the leak actually bit — a leak-free run would return to
    ``battery0`` and must not be pinnable here).
    """
    state, registry = build_power(_CHARGE, _SCENARIO, _SELF_DISCHARGE)
    resolver = power_resolver(_CHARGE, _SCENARIO)
    states, rationed, events = run_power(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden self-discharge run must be well-fed (no arbitration)"
    assert events == (), "golden self-discharge run must be event-free"
    # The Phase-5 payload survives the third flow: every step's ENERGY ledger balances.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= _LEDGER_ABS_TOL, (
            "golden self-discharge run must keep ENERGY closed every step —"
            " an imbalanced trajectory must not be pinnable as this golden"
        )
    # The "it bit" gate: the SOC departs battery0 monotonically each day boundary (a
    # leak-free baseline would return to battery0 — so a no-op leak is unpinnable here).
    spd = _SCENARIO.steps_per_day
    b0 = _SCENARIO.battery0
    day_soc = [
        states[d * spd].stocks[BATTERY].amount for d in range(SELF_DISCHARGE_DAYS + 1)
    ]
    assert all(a > b for a, b in zip(day_soc, day_soc[1:], strict=False)), (
        "golden self-discharge run's day-boundary SOC must strictly decay (leak active)"
    )
    assert b0 - day_soc[-1] > 1.0, (
        "golden self-discharge run must depart battery0 (the leak must actually bit)"
    )
    return states[-1]


def test_power_self_discharge_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change fails here
    # (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_power_self_discharge_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State.
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed self-discharge golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_power_self_discharge.py

    Review the diff before committing: a change here means the Power output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
