"""Step-7 regression-snapshot gate: the golden minimal-consumer multi-year run.

Pins the Phase-3 Step-7 capstone — the perennial sealed chamber plus **one herbivore**
(``CONSUMER_CHAMBER_SCENARIO``, the season weather tiled ``CONSUMER_CHAMBER_YEARS×``,
Euler-daily, with :func:`season.run_perennial`'s annual phenology reset / re-sow) —
bit-exactly. The **final State** is serialized via the ``sim_io`` hex-float serializer
and byte-compared to a committed golden, so any bit change in the consumer output (a
flow
law — grazing / consumer respiration / mortality — a herbivory param, the reset, the
reduction order, or the consumer sizing) surfaces here. The fourth golden (open field
``test_regression_season.py``; sealed ``test_regression_sealed_season.py``; perennial
``test_regression_perennial_season.py``); the validation phenomena (consumer
persistence,
the leaf↓/CO₂↑ cascade, genuine closure, conservation, the per-compartment ledger,
determinism) are pinned behaviourally in ``test_consumer.py``.

Mirrors the perennial golden: full ``State`` via ``sim_io.dumps`` (hex-float, incl. the
aux ``thermal_time`` and every boundary stock), Euler only, regeneration a separate
explicit ``__main__`` action. The generator bakes in the **pre-golden closure gate**
(the
Step-4 rhythm): it asserts ``rationed == 0``, ``events == ()`` AND the carbon loss-sink
==
0.0 on this exact scenario — death (plant *and* consumer) routes to ``litter_carbon``,
never to the BOUNDARY loss-sink, so "genuinely closed" holds for *these* committed knobs
—
before the bytes can be pinned.

**Bit-stability caveat** (as for the other goldens): the season uses transcendentals
(``exp``/``pow``/``sin``) which IEEE-754 does not mandate correctly-rounded, so this
golden is bit-identical **within a build** (determinism #7; the determinism test
confirms)
but cross-platform last-ULP differences are tolerance territory. Regenerate (review the
diff) if the toolchain moves.
"""

import json

import sim_io
from config.paths import GOLDEN_DIR, WINTER_WHEAT_WEATHER
from domains.biosphere.season import (
    CONSUMER_CHAMBER_SCENARIO,
    CONSUMER_CHAMBER_YEARS,
    build_season,
    run_perennial,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, steps_for
from golden_platform import (
    assert_matches_golden,
    windows_golden_only,
    write_python_golden,
)
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

GOLDEN_PATH = GOLDEN_DIR / "consumer_chamber_state.json"

_WEATHER_FIXTURE = WINTER_WHEAT_WEATHER


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _final_state() -> State:
    """Run the canonical consumer multi-year season (Euler); return the final State.

    The single source of truth for the committed golden and the load-back test. Bakes in
    the **pre-golden closure gate**: the golden comes from a ``rationed == 0`` /
    no-extinction / loss-sink-empty trajectory by construction — the consumer's
    mortality
    routes to ``litter_carbon`` (in-system, decomposable), never to the BOUNDARY
    loss-sink, so "genuinely closed" holds for *these* committed knobs (the consumer is
    sized to persist while the plant still fills grain for ``annual_reset`` — the
    recoverable regime).
    """
    year = len(_weather())
    weather = _weather() * CONSUMER_CHAMBER_YEARS
    state, registry = build_season(CONSUMER_CHAMBER_SCENARIO)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        CONSUMER_CHAMBER_SCENARIO,
        weather_resolver(weather, CONSUMER_CHAMBER_SCENARIO),
        BIO_DT,
        steps_for(len(weather)),
        year=steps_for(year),
    )
    assert rationed == 0, "golden consumer run must be well-fed (no arbitration)"
    assert events == (), "golden consumer run must be extinction-free"
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states), (
        "golden consumer run must be genuinely closed (carbon loss-sink stays 0.0 — "
        "the consumer's death routes to litter, not the BOUNDARY loss-sink)"
    )
    return states[-1]


@windows_golden_only
def test_consumer_golden_matches_the_reference() -> None:
    """Python's consumer output against the **Rust-authored** golden.

    ⚠ This assertion used to be ``==`` on the bytes. It is one of the two the reference
    flip loosened (``golden_platform.PYTHON_DIVERGES``): Python's 5-yr consumer chamber
    differs from Rust's at 7 of 205 leaves, worst ~2 ULP. The comparison is still Tier-0
    exact on structure and every discrete field, and the numeric ceiling is 1000x
    tighter
    than this scenario's own Tier-2 band — but it can no longer see a reduction-order
    change. ⚠ That coverage did not vanish, it moved: the 15-yr horizon of *this same
    scenario* (``consumer_long_horizon_state.json``) is still byte-gated, and
    ``test_golden_provenance.py`` asserts it stays that way.
    """
    assert_matches_golden(GOLDEN_PATH, sim_io.dumps(_final_state()))


@windows_golden_only
def test_consumer_golden_loads_back() -> None:
    """The golden decodes through the core constructors and round-trips **byte-stably**.

    ⚠ Kept EXACT on purpose: both ends of a codec round trip are Python, so which engine
    wrote the bytes is irrelevant and the flip does not reach this assertion. Its other
    half — "…and equals what the live engine produces" — is
    :func:`test_consumer_golden_matches_the_reference`'s job, gated there under
    tolerance and deliberately not repeated here.

    ⚠⚠ **What that costs, measured rather than asserted.** The old form
    (``loads(text) == _final_state()``) caught *any* tampered value; this one catches
    only what fails to round-trip — a malformed literal, a non-canonical spelling, a
    moved key.
    A changed-but-valid hex float survives it. Negative controls on this pair:

    * gross value tamper → red at :func:`test_consumer_golden_matches_the_reference`;
    * **last-nibble tamper (~1.4e-15 relative) → GREEN on both.** Under the 1e-14
      ceiling
      it is by construction indistinguishable from the divergence the roster permits.

    That last row is the honest price of tolerance-gating this file, and it is **not** a
    hole in the suite: the byte-exact backstop moved to the side that now owns the
    bytes.
    ``tests/crossport/test_golden_provenance.py`` asserts Rust reproduces this golden
    exactly, with no exemptions — so a last-nibble edit is still red, on the reference
    side. ⚠ That census is Windows + ``cargo`` + ``slow``, i.e. local, not the CI job.
    """
    text = GOLDEN_PATH.read_text(encoding="utf-8")
    loaded = sim_io.loads(text)  # a tampered golden fails to load at all
    assert sim_io.dumps(loaded).encode("utf-8") == GOLDEN_PATH.read_bytes()


def _regenerate() -> None:
    """⚠ **Refuses.** This golden is authored by the Rust port, not by Python.

    Kept as a signpost rather than deleted: ``write_python_golden`` raises with the
    blessed path (``tests/crossport/regen_goldens_from_rust.py --write``), so someone
    reaching for the habitual ``uv run python tests/test_regression_consumer_season.py``
    is told where the reference lives instead of silently reverting it to the checker.
    """
    write_python_golden(GOLDEN_PATH, sim_io.dumps(_final_state()).encode("utf-8"))


if __name__ == "__main__":
    _regenerate()
