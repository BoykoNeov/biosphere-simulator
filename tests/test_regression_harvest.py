"""Regression-snapshot gate: the golden biomass/food-loop run (P6.6).

Pins ``HARVEST_SCENARIO`` (the reproductive greenhouse plant filling grain that the
station-owned ``Harvest`` flow drains into the crew ``food_store``, Euler, over the
embedded greenhouse horizon) bit-exactly. The **final coupled State** is serialized via
the ``sim_io`` hex-float serializer and byte-compared to a committed golden, so any bit
change in the coupled trajectory (the ``Harvest`` law, the harvest rate, the
``thermal_time0`` phenology start, the reused greenhouse gas loop, the crew/ECLSS
coefficients, the sizing, the reduction order) surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference). It is the station's own Step-6 regression pin, the sibling of the Step-1
``station_state.json`` / Step-2 ``cabin_gas_state.json`` / Step-3
``greenhouse_state.json`` / Step-4 ``water_recovery_state.json`` / Step-5
``lighting_state.json`` goldens; every frozen reference golden + demo golden +
sibling/station golden stays untouched and byte-identical (Step 6 is a *separate*
assembly — zero domain change, zero core change).

Mirrors ``test_regression_water_recovery`` (the additive-scenario discipline): full
``State`` via ``sim_io.dumps``, Euler only, regeneration a separate explicit
``__main__`` action. The generator bakes in a **pre-golden gate specific to Step 6's
purpose**: the run is well-fed (``rationed == 0``), event-free (``events == ()``), **all
three mass quantities (CARBON / OXYGEN / WATER) balance every master day** (the closure
payload), the ``Harvest`` **actually moved carbon** — ``food_store`` ended above the
no-harvest baseline and ``storage_c`` below it (the "it bit" check, the analogue of the
water-recovery "regenerated above the open-loop baseline" gate) — and the two-way
identity (``Δfood_store = Δstorage_c``) holds. So the golden is **impossible to
regenerate from a degenerate run** (an imbalance / a non-harvesting run / a broken
identity fails the gate, not silently re-freezes).

**Bit-stability caveat** (as for the biosphere / Power / Thermal / ECLSS / station /
cabin / water-recovery / lighting goldens): the coupled flows use only +−×÷ (no
transcendental), so this golden is bit-identical within a build; regenerate (review the
diff) if the toolchain moves.
"""

import json
from pathlib import Path

import sim_io
from domains.biosphere.stocks import STORAGE_C
from domains.crew.loader import load_crew_params
from domains.crew.stocks import FOOD_STORE
from domains.eclss.loader import load_eclss_params
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.harvest import (
    build_harvest,
    harvest_bio_resolver,
    harvest_cabin_resolver,
    run_harvest,
)
from station.loader import load_harvest_params
from station.scenario import HARVEST_SCENARIO

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "harvest_state.json"
_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_HARVEST = load_harvest_params()
_SCENARIO = HARVEST_SCENARIO

_LEDGER_ABS_TOL = 1e-6
_IDENTITY_ABS_TOL = 1e-7
_MASS_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(*, with_harvest: bool) -> list[State]:
    state, bio_reg, cabin_reg = build_harvest(
        _CREW, _ECLSS, _HARVEST, _SCENARIO, with_harvest=with_harvest
    )
    states, rationed, events = run_harvest(
        EulerIntegrator(bio_reg),
        EulerIntegrator(cabin_reg),
        state,
        harvest_bio_resolver(_weather(), _SCENARIO),
        harvest_cabin_resolver(_SCENARIO),
        _SCENARIO,
    )
    assert rationed == 0, "golden harvest run must be well-fed (no arbitration)"
    assert events == (), "golden harvest run must be event-free"
    return states


def _final_state() -> State:
    """Run the canonical harvest run (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: every master day's CARBON / OXYGEN / WATER ledgers balance
    (closure), the ``Harvest`` regenerated ``food_store`` above the no-harvest baseline
    and drained ``storage_c`` below it (the "it bit" check), and the two-way identity
    holds — a future imbalance / a non-harvesting run / a broken identity fails here,
    not re-freezes.
    """
    states = _run(with_harvest=True)
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in _MASS_QUANTITIES:
            assert abs(ledger[quantity].residual) <= _LEDGER_ABS_TOL, (
                "golden harvest run must keep every quantity closed every master day —"
                " an imbalanced trajectory must not be pinnable as this golden"
            )
    final = states[-1]
    baseline_final = _run(with_harvest=False)[-1]
    # The harvest bit: food regenerated above baseline, grain drained below it.
    d_food = final.stocks[FOOD_STORE].amount - baseline_final.stocks[FOOD_STORE].amount
    d_storage = baseline_final.stocks[STORAGE_C].amount - final.stocks[STORAGE_C].amount
    assert d_food > 0.0, (
        "golden run must regenerate food_store above the no-harvest baseline"
    )
    assert d_storage > 0.0, "golden run must drain grain below the no-harvest baseline"
    # The two-way identity: food gained == grain removed (a pure CARBON transfer).
    assert abs(d_food - d_storage) <= _IDENTITY_ABS_TOL, (
        "golden run's harvest must satisfy Δfood_store = Δstorage_c (a CARBON transfer)"
    )
    return final


def test_harvest_golden_bytes_match() -> None:
    # Byte-exact compare against the committed golden — any bit change in the coupled
    # output fails here (within-build; the flows are transcendental-free).
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


def test_harvest_golden_loads_back() -> None:
    # The committed golden round-trips back to the exact final State.
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed harvest golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_harvest.py

    Review the diff before committing: a change here means the coupled output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
