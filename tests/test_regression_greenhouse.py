"""Regression-snapshot gate: the golden biosphere ↔ cabin greenhouse run (P6.3).

Pins ``GREENHOUSE_SCENARIO`` (the frozen sealed biosphere breathing the crew's
cabin air, Euler, over ``days`` master steps of the two-rate driver) bit-exactly —
the **final State** serialized via the ``sim_io`` hex-float serializer and
byte-compared to a committed golden. Any bit change in the coupled trajectory (a
biosphere flow, a cabin flow, the reverse-seam wiring, the master-step driver,
``crew.yaml`` / ``eclss.yaml`` / a crop param, the reduction order, the scenario
sizing) surfaces here.

**This is an additive, NON-frozen golden** — not in
``docs/biosphere-reference.manifest.json`` (that manifest is the frozen
*biosphere* reference; this run re-wires the frozen biosphere *beside* its
freeze). It is the Step-3 station assembly's own regression pin, the
``test_regression_power`` / cabin-golden analogue; the seven frozen reference
goldens + two demo + the earlier station goldens are untouched and byte-identical.

**The pre-golden gate bakes in Step 3's purpose.** The generator asserts the run
is well-fed (``rationed == 0``), event-free (``events == ()``), that **every
quantity closes across every master day** (the combined-ledger payload), and — the
"it bit" check — that the **plant fixes net carbon** (biosphere organic carbon
grows: the plant is a net CO₂ sink, so by the RQ = 1 offload identity it removed
CO₂ the scrubber didn't and released O₂ the makeup didn't). A degenerate run —
plant inert, or pushed into net- source by a longer horizon — fails the gate and
is **unpinnable**, rather than silently re-freezing.

**Scope (by design).** The pinned state sits at a day boundary, where the fast
ECLSS scrubber has relaxed ``CARBON_POOL`` / ``O2_POOL`` back to their regulator
setpoints — so those two pool *values* do not distinguish with/without plants
(they are setpoints, not evidence). The plant's signature is in the pinned
biosphere biomass/litter stocks + the cumulative ``co2_removed`` / ``o2_supply``
reservoirs. The explicit with/without offload booleans + the three-way
conservation identity live in ``test_greenhouse_run.py`` (the pinned-state +
separate-behavioral division).

**Bit-stability caveat** (as for the biosphere/Power goldens): the biosphere
weather conversions + FvCB use ``math`` transcendentals (not IEEE-754
correctly-rounded), so this golden is bit-identical **within a build**;
cross-platform last-ULP differences are tolerance territory. Regenerate (review
the diff) if the toolchain moves.
"""

import json
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
from domains.crew.loader import load_crew_params
from domains.eclss.loader import load_eclss_params
from golden_platform import windows_golden_only
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator
from simcore.state import State
from station.greenhouse import (
    build_greenhouse,
    greenhouse_bio_resolver,
    greenhouse_cabin_resolver,
    run_greenhouse,
)
from station.scenario import GREENHOUSE_SCENARIO

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
GOLDEN_PATH = GOLDEN_DIR / "greenhouse_state.json"
_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_SC = GREENHOUSE_SCENARIO
_BIO_C = (LEAF_C, STEM_C, ROOT_C, STORAGE_C, LITTER_CARBON, MICROBIAL_CARBON)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical greenhouse (Euler, with plants); return the final State.

    The single source of truth for the committed golden and the load-back test.
    Bakes in the **pre-golden gate**: ``rationed == 0`` / ``events == ()``, every
    conserved quantity balances across every master day (the combined-ledger
    closure payload), and the plant fixes net carbon (biosphere organic carbon
    grows — the offload "it bit" check) — so a future imbalance/inert-plant
    regression fails here rather than re- freezing a degenerate run.
    """
    state, bio_reg, cabin_reg = build_greenhouse(_CREW, _ECLSS, _SC, with_plants=True)
    states, rationed, events = run_greenhouse(
        EulerIntegrator(bio_reg),
        EulerIntegrator(cabin_reg),
        state,
        greenhouse_bio_resolver(_weather(), _SC),
        greenhouse_cabin_resolver(_SC),
        _SC,
    )
    assert rationed == 0, "golden greenhouse run must be well-fed (no arbitration)"
    assert events == (), "golden greenhouse run must be event-free"
    for before, after in zip(states, states[1:], strict=False):
        for ql in compute_ledger(before, after):
            assert abs(ql.residual) <= 1e-6, (
                f"golden greenhouse run must keep {ql.quantity} closed every master day"
                " — an imbalanced trajectory must not be pinnable as this golden"
            )
    gained = sum(states[-1].stocks[s].amount for s in _BIO_C) - sum(
        states[0].stocks[s].amount for s in _BIO_C
    )
    assert gained > 0.0, (
        "golden greenhouse run must fix net carbon (the plant offloads life support) —"
        " a net-source run must be unpinnable"
    )
    return states[-1]


@windows_golden_only
def test_greenhouse_golden_bytes_match() -> None:
    expected = sim_io.dumps(_final_state()).encode("utf-8")
    assert expected == GOLDEN_PATH.read_bytes()


@windows_golden_only
def test_greenhouse_golden_loads_back() -> None:
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state()


def _regenerate() -> None:
    """Rewrite the committed greenhouse golden from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_greenhouse.py

    Review the diff before committing: a change here means the coupled output
    moved.
    """
    GOLDEN_PATH.write_bytes(sim_io.dumps(_final_state()).encode("utf-8"))
    print(f"wrote {GOLDEN_PATH}")


if __name__ == "__main__":
    _regenerate()
