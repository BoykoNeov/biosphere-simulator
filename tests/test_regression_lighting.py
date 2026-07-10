"""Regression-snapshot gate: the golden Power → biosphere lighting run (P6.5).

Pins ``LIGHTING_SCENARIO`` (the sealed biosphere lit by a battery-powered grow lamp,
Euler, over ``days`` master steps of the two-rate driver) bit-exactly — the **final
State** serialized via the ``sim_io`` hex-float serializer and byte-compared to a
committed golden. Any bit change in the coupled trajectory (a biosphere flow, the Lamp
flow, the PAR/daylength-from-lamp wiring, the master-step driver, ``lamp.yaml`` / a crop
param, the reduction order, the scenario sizing) surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen *biosphere*
reference; this run re-wires the frozen biosphere *beside* its freeze, PAR now computed
from the lamp instead of a weather table). It is the Step-5 station assembly's own
regression pin, the ``test_regression_greenhouse`` analogue; the seven frozen reference
goldens + two demo + the earlier station goldens are untouched and byte-identical (zero
domain change, zero core change).

**The pre-golden gate bakes in Step 5's purpose.** The generator asserts the run is
well-fed (``rationed == 0``), event-free (``events == ()``), that **every quantity
closes across every master day** (the combined-ledger payload), that the **battery
drained by exactly the lamp's daily energy** (ENERGY closure, quantitatively — the
``light_used`` + ``waste_heat`` names it), and — the signed "it bit" check — that the
**plant fixed net carbon under the lamp** (biosphere organic carbon grew, which it does
*only* because PAR came from the lamp: the dark baseline declines). A degenerate run
(imbalance, a dead lamp, an inert plant) fails the gate and is **unpinnable**, rather
than silently re-freezing.

**Bit-stability caveat** (as for the biosphere / greenhouse goldens): the biosphere
weather conversions + FvCB use ``math`` transcendentals, so this golden is bit-identical
**within a build**; regenerate (review the diff) if the toolchain moves.
"""

from pathlib import Path

import sim_io
from domains.biosphere.stocks import (
    LEAF_C,
    LITTER_CARBON,
    MICROBIAL_CARBON,
    ROOT_C,
    STEM_C,
    STORAGE_C,
)
from domains.power.stocks import BATTERY, WASTE_HEAT
from golden_platform import windows_golden_only
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.state import State
from station.flows import PAR_PHOTON_ENERGY_J_PER_UMOL
from station.lighting import (
    LIGHT_USED,
    build_lighting,
    lighting_bio_resolver,
    lighting_power_resolver,
    run_lighting,
)
from station.loader import load_lamp_params
from station.scenario import LIGHTING_SCENARIO

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "lighting_state.json"
_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

_LP = load_lamp_params()
_SC = LIGHTING_SCENARIO
_BIO_C = (LEAF_C, STEM_C, ROOT_C, STORAGE_C, LITTER_CARBON, MICROBIAL_CARBON)


def _weather() -> list[dict[str, float | str]]:
    import json

    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(*, with_lamp: bool) -> list[State]:
    state, bio_reg, power_reg = build_lighting(_LP, _SC, with_lamp=with_lamp)
    states, rationed, events = run_lighting(
        EulerIntegrator(bio_reg),
        EulerIntegrator(power_reg),
        state,
        lighting_bio_resolver(_weather(), _LP, _SC, with_lamp=with_lamp),
        lighting_power_resolver(_SC),
        _SC,
    )
    assert rationed == 0, "golden lighting run must be well-fed (no arbitration)"
    assert events == (), "golden lighting run must be event-free"
    return states


def _final_state() -> State:
    """Run the canonical lighting run (Euler, lamp on); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden gate**: every master day's ledgers balance (closure), the battery
    drained by exactly the lamp's daily energy (ENERGY named), and the plant fixed net
    carbon under the lamp while the dark baseline declined (the signed "it bit" check) —
    a future imbalance / dead-lamp / inert-plant regression fails here, not re-freezes.
    """
    states = _run(with_lamp=True)
    for before, after in zip(states, states[1:], strict=False):
        for ql in compute_ledger(before, after):
            assert abs(ql.residual) <= 1e-6, (
                f"golden lighting run must keep {ql.quantity} closed every master day —"
                " an imbalanced trajectory must not be pinnable as this golden"
            )
    initial, final = states[0], states[-1]
    # ENERGY named: the battery drained exactly the lamp's daily energy × days, split
    # into light_used + waste_heat.
    drawn = _SC.lamp_power_w * _SC.photoperiod_hours * 3600.0 * _SC.days
    assert (
        abs((initial.stocks[BATTERY].amount - final.stocks[BATTERY].amount) - drawn)
        <= 1.0
    ), "golden lighting run must drain the battery by the lamp's daily energy"
    eta = _LP.photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL
    assert abs(final.stocks[LIGHT_USED].amount - eta * drawn) <= 1.0
    assert abs(final.stocks[WASTE_HEAT].amount - (1.0 - eta) * drawn) <= 1.0
    # The signed "it bit": lit plant grew, dark baseline declined.
    grown = sum(final.stocks[s].amount for s in _BIO_C) - sum(
        initial.stocks[s].amount for s in _BIO_C
    )
    assert grown > 0.0, "golden lighting run must fix net carbon under the lamp"
    dark = _run(with_lamp=False)
    dark_change = sum(dark[-1].stocks[s].amount for s in _BIO_C) - sum(
        dark[0].stocks[s].amount for s in _BIO_C
    )
    assert dark_change < 0.0, (
        "the dark baseline must lose carbon — the lamp is what makes the plant a sink"
    )
    return final


@windows_golden_only
def test_lighting_golden_bytes_match() -> None:
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_lighting_golden_loads_back() -> None:
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed lighting golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_lighting.py

    Review the diff before committing: a change here means the coupled output moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
