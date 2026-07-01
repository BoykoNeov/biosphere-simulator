"""Regression-snapshot gate: the golden standalone Crew run (Step 7, net-consumer).

Pins ``MISSION_SCENARIO`` (the provisioned crew depleting its finite stores under the
three forced metabolic flows, Euler, over ``MISSION_DAYS × steps_per_day`` steps)
bit-exactly. The **final State** is serialized via the ``sim_io`` hex-float serializer
and byte-compared to a committed golden, so any bit change in the Crew trajectory (a
flow law, a split fraction, the reduction order, ``crew.yaml``, the scenario sizing)
surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the Crew domain's own regression pin, the sibling of the Power /
Thermal / ECLSS goldens (``power_state.json`` / ``thermal_state.json`` /
``eclss_state.json``); the seven frozen reference goldens + two demo goldens + the two
Power goldens + the Thermal golden + the ECLSS golden are untouched and byte-identical.

Mirrors ``test_regression_eclss`` (the additive-scenario discipline): full ``State`` via
``sim_io.dumps``, Euler only, regeneration a separate explicit ``__main__`` action. The
generator bakes in a **pre-golden gate specific to Crew's purpose**: it asserts the run
is well-fed (``rationed == 0``), event-free (``events == ()``), that **all three
quantities (CARBON / OXYGEN / WATER) balance every step** (residual ≈ 0 — the
multi-quantity payload, the analogue of ECLSS's "three quantities closed"), and that the
mission **actually ran the stores down materially** (each store between 60 % and 80 % of
its initial inventory — the "it bit" check) — so the golden is **impossible to
regenerate from a degenerate run** (a zero-intake mission that leaves the stores full,
or an over-long one that empties them and rations, fails the gate; the analogue of the
N-limited ``f_N`` gate).

**Bit-stability caveat** (as for the biosphere / Power / Thermal / ECLSS goldens): the
flows use plain float arithmetic (no transcendentals — the Crew laws are linear splits),
so this golden is bit-identical **within a build**; cross-platform last-ULP differences
remain tolerance territory. Regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import sim_io
from domains.crew.loader import load_crew_params
from domains.crew.scenario import MISSION_DAYS, MISSION_SCENARIO
from domains.crew.stocks import FOOD_STORE, O2_STORE, WATER_STORE
from domains.crew.system import build_crew, crew_resolver, run_crew
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "crew_state.json"

_PARAMS = load_crew_params()
_SCENARIO = MISSION_SCENARIO
_STEPS = MISSION_DAYS * _SCENARIO.steps_per_day
_DT = _SCENARIO.dt_seconds

_LEDGER_ABS_TOL = 1e-6


def _final_state() -> State:
    """Run the canonical mission Crew run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: the golden comes from a ``rationed == 0`` / ``events ==
    ()`` trajectory in which **all three quantities balance every step** (residual ≈ 0 —
    the payload) and the mission **materially depleted** every store (each between 60 %
    and 80 % of its initial inventory — the stores ran down but stayed well-fed) — so a
    future imbalance / degenerate-mission regression fails here rather than silently
    re-freezing a broken trajectory.
    """
    state, registry = build_crew(_PARAMS, _SCENARIO)
    resolver = crew_resolver(_SCENARIO)
    states, rationed, events = run_crew(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert rationed == 0, "golden Crew run must be well-fed (no arbitration)"
    assert events == (), "golden Crew run must be event-free (no POPULATION stock)"
    # The payload: every step's augmented ledger balances all three mass quantities.
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for q in (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER):
            assert abs(ledger[q].residual) <= _LEDGER_ABS_TOL, (
                "golden Crew run must keep every quantity closed every step "
                "(an imbalanced trajectory must not be pinnable as this golden)"
            )
    # The mission actually ran: each store materially depleted yet stayed well-fed. A
    # degenerate (zero-intake / over-long) run fails here.
    final = states[-1]
    for stock, initial in (
        (FOOD_STORE, _SCENARIO.food_store0),
        (WATER_STORE, _SCENARIO.water_store0),
        (O2_STORE, _SCENARIO.o2_store0),
    ):
        assert 0.6 * initial < final.stocks[stock].amount < 0.8 * initial
    return final


def test_crew_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the Crew
    # output fails here (within-build; see the caveat in the module doc).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_crew_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State (also the check
    # that sim_io serializes a three-mass-quantity Crew State cleanly).
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed Crew golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_crew.py

    Review the diff before committing: a change here means the Crew output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
