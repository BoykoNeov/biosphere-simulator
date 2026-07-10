"""Regression-snapshot gate: the golden standalone Power run (P5.4, energy closure).

Pins ``BOUNDED_SOC_SCENARIO`` (the daily-balanced microgrid, Euler, over
``BOUNDED_SOC_DAYS``) bit-exactly — the standalone Power validation run whose battery
charges by day, discharges by night, and returns to the same SOC each day. The **final
State** is serialized via the ``sim_io`` hex-float serializer and byte-compared to a
committed golden, so any bit change in the Power trajectory (a flow law, the derived
load, the solar schedule, the reduction order, ``charge.yaml``, the scenario sizing)
surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the Power domain's own regression pin, the P5.2/P5.3 analogue of the
biosphere scenario goldens; the seven frozen reference goldens + two demo goldens are
untouched and byte-identical.

Mirrors ``test_regression_n_limited_season.py`` (the additive-scenario discipline): full
``State`` via ``sim_io.dumps``, Euler only, regeneration a separate explicit
``__main__`` action. The generator bakes in a **pre-golden gate specific to Power's
purpose**: it asserts the run is well-fed (``rationed == 0``), event-free
(``events == ()``), that the **per-step ENERGY ledger balances** (residual ≈ 0 — the
Phase-5 energy-closure payload, the analogue of the N-limited gate's "``f_N`` actually
bit": a run whose ledger did *not* balance must be **unpinnable**), and that the SOC
actually swings *and* returns to
``battery0`` at every day boundary (the bounded-daily-cycle property that only holds
under exact balance) — so the golden is **impossible to regenerate from a degenerate
run** (a drift/imbalance regression fails the gate, not silently re-freezes a broken
trajectory).

**Scope (by design):** a final-state-at-a-day-boundary snapshot is blind to intra-day
*shape* changes that preserve daily energy (``solar_source``/``waste_heat`` cumulatives
track totals; the battery returns to ``battery0``). That shape coverage — the interior
morning minimum, the half-sine schedule, the monotonic heat, RK4 ≡ Euler — lives in
``test_power_run.py`` (the biosphere division: a pinned final State *plus* separate
behavioral tests). This golden pins the endpoint; it does not replace those.

**Bit-stability caveat** (as for the biosphere goldens): ``solar_schedule`` uses
``math.sin``/``math.pi``, which IEEE-754 does not mandate correctly-rounded, so this
golden is bit-identical **within a build** but cross-platform last-ULP differences are
tolerance territory. Regenerate (review the diff) if the toolchain moves.
"""

import math
from pathlib import Path

import sim_io
from domains.power.loader import load_charge_params
from domains.power.scenario import BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO
from domains.power.stocks import BATTERY
from domains.power.system import build_power, power_resolver, run_power
from golden_platform import windows_golden_only
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "power_state.json"

_CHARGE = load_charge_params()
_SCENARIO = BOUNDED_SOC_SCENARIO
_STEPS = BOUNDED_SOC_DAYS * _SCENARIO.steps_per_day
_DT = _SCENARIO.dt_seconds

# Step-3's exact tolerances (test_power_run.py): the day-boundary SOC returns to
# battery0 to round-off, and each per-step ENERGY ledger residual is ≈ 0. The gate must
# match — not exceed — the property Step 3 validated, or legitimate 7-day round-off
# drift (the final state sits at n = BOUNDED_SOC_DAYS·steps_per_day) fails regeneration.
_LEDGER_ABS_TOL = 1e-6
_SOC_RETURN_ABS_TOL = 1e-6


def _final_state() -> State:
    """Run the canonical bounded-SOC Power run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes
    in the **pre-golden gate**: the golden comes from a ``rationed == 0`` /
    ``events == ()`` trajectory in which the **per-step ENERGY ledger balances**
    (residual ≈ 0 — energy closure), the SOC materially swings, and the SOC returns to
    ``battery0`` at every day boundary — so a future imbalance/drift regression fails
    here rather than silently re-freezing a degenerate run.
    """
    state, registry = build_power(_CHARGE, _SCENARIO)
    resolver = power_resolver(_CHARGE, _SCENARIO)
    states, rationed, events = run_power(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden Power run must be well-fed (no arbitration)"
    assert events == (), "golden Power run must be event-free (no POPULATION stock)"
    # The Phase-5 payload: every step's augmented ENERGY ledger balances to round-off.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= _LEDGER_ABS_TOL, (
            "golden Power run must keep ENERGY closed every step (energy closure) —"
            " an imbalanced trajectory must not be pinnable as this golden"
        )
    # A genuine charge/discharge swing (not a flat line) that returns each day boundary.
    soc = [s.stocks[BATTERY].amount for s in states]
    assert max(soc) - min(soc) > 0.1 * _SCENARIO.battery0, (
        "golden Power run must have a material SOC swing (charge/discharge)"
    )
    spd = _SCENARIO.steps_per_day
    for day in range(BOUNDED_SOC_DAYS + 1):
        assert math.isclose(
            states[day * spd].stocks[BATTERY].amount,
            _SCENARIO.battery0,
            rel_tol=1e-9,
            abs_tol=_SOC_RETURN_ABS_TOL,
        ), "golden Power run must return to battery0 each day (bounded daily cycle)"
    return states[-1]


@windows_golden_only
def test_power_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the Power
    # output fails here (within-build; see the transcendental caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_power_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (also the check
    # that sim_io serializes a pure-ENERGY Power State cleanly — no POPULATION/mass).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed Power golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_power.py

    Review the diff before committing: a change here means the Power output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
