"""``eclss.o2_makeup`` runs BACKWARDS inside the frozen roster — not only for authors.

**What this file corrects.** Three committed loci say the O₂ regulator's above-setpoint
reversal is a seam the frozen scenarios never reach:

* ``src/domains/eclss/flows.py`` — "a deferred seam that never arises here";
* ``docs/authoring-reference.md`` — the same quote, glossed "**true of every
  frozen scenario**, but an author can wire ``cabin_o2`` above the ``10.0 mol``
  setpoint";
* ``src/authoring/flow_registry.py`` — "reachable **by an author** who wires
  ``cabin_o2`` above the frozen 10.0 mol setpoint".

All three scope the reversal to *authored* content. Measured, that is **false of the
frozen roster**: it fires in every frozen scenario that couples the regulator to a
photosynthesising crop. It is nobody's bug — ``docs/authoring-reference.md`` decides the
question the other way and the decision stands ("**Do not 'fix' this with a clamp** …
the reversal is correct P-control", because ``o2_eq = o2_setpoint − Con_o2/k_makeup`` is
an attractor from *both* sides only while the controller stays linear). What was wrong
was the **scope claim**, and the frozen goldens have carried the reversal all along.

**The mechanism — why the split is standalone-vs-coupled and not a coincidence.**
``build_eclss`` / ``build_cabin`` / ``build_water_recovery`` wire ``O2Makeup`` to a
cabin stock whose only O₂ source is the regulator itself, starting *at* the setpoint
under a consuming crew — so the error ``(o2_setpoint − cabin_o2)`` can never go
negative. ``build_greenhouse`` / ``build_sealed_station`` wire it to the
**biosphere's** ``O2_POOL`` (``station/greenhouse.py``, ``station/sealed.py`` — "the
seam: crew breathes the biosphere O₂ pool"), which has a second, larger source the
regulator does not model: the plant. Photosynthesis overshoots the setpoint and the
regulator pushes the excess back into ``boundary.o2_supply``. So the reversal is not
an edge case reached by extreme wiring — it is what this model does whenever plants
out-produce the crew, and the seam that makes it reachable is P6.3's, not an
author's.

⚠ **The goldens are blind to all of this — measured, not inferred.** Clamping
``makeup_flux`` leaves every plant-coupled golden **byte-identical**, so these frozen
snapshots neither pin the reversal nor refute it: they freeze the *endpoint*, and these
are single mid-run calls. That is why the finding needed a file of its own rather than
an existing gate, and why a clamp would cost no golden cascade anywhere in the roster
(the refusal is physical, not a price).

**Magnitudes, and why they are pinned as bands rather than values.** The excursions are
small and rare, and **not one magnitude**: ``greenhouse``/``harvest`` overshoot by
0.081 % (1 call in 10 080; 3 in 30 240), ``sealed_station`` by 0.0030 % (1 in
1 756 800) — a 27× spread. They are also downstream of the biosphere's ``math``
transcendentals, so an
exact pin would be a cross-libm trap (``docs/perf-baseline.md``'s neighbours; the CI-is-
Linux/box-is-Windows split). Both the *existence* of the reversal and the peak's order
are far above ULP noise, so this file asserts those and leaves the exact counts to
``docs/plans/post-roadmap-o2-makeup-reversal.md``, where a stale number is harmless.

The standalone side is pinned **exactly**: those trajectories start at ``cabin_o2_0 =
10.0`` and only decrease, so their maximum is the initial value — ``+ − ×`` arithmetic
with no transcendental in it.
"""

from __future__ import annotations

from collections.abc import Callable, Iterator
from contextlib import contextmanager

import domains.eclss.flows as eclss_flows
from domains.eclss.loader import load_eclss_params

SETPOINT = load_eclss_params().o2_setpoint
"""The frozen 10.0 mol target, read from the param file rather than restated."""


@contextmanager
def _recording() -> Iterator[list[tuple[float, float]]]:
    """Record every ``(cabin_o2, flux)`` ``O2Makeup`` asks ``makeup_flux`` for.

    Patching the module-level function (which ``O2Makeup.evaluate`` looks up by name at
    call time) records **every** evaluation, including the ones inside a multi-rate
    sub-step that never commits a state — which a trajectory scan over committed states
    would miss. That matters here: the excursions are single-call events.
    """
    calls: list[tuple[float, float]] = []
    real = eclss_flows.makeup_flux

    def recorder(cabin_o2: float, **kw: float) -> float:
        flux = real(cabin_o2, **kw)
        calls.append((cabin_o2, flux))
        return flux

    eclss_flows.makeup_flux = recorder  # type: ignore[assignment]
    try:
        yield calls
    finally:
        eclss_flows.makeup_flux = real  # type: ignore[assignment]


def _standalone_runners() -> dict[str, Callable[[], object]]:
    """The three frozen scenarios whose regulator sees a cabin stock, not the crop."""
    import test_regression_cabin as cabin
    import test_regression_eclss as eclss
    import test_regression_water_recovery as water_recovery

    return {
        "eclss_steady_state": eclss._final_state,
        "cabin_gas": cabin._final_state,
        "water_recovery": water_recovery._final_state,
    }


def _plant_coupled_runners() -> dict[str, Callable[[], object]]:
    """The frozen scenarios whose regulator is wired to the biosphere O₂ pool.

    ``sealed_station`` is the third and reverses too, but is a Tier-2 run (minutes)
    and is deliberately left uncovered — see
    ``test_the_uncovered_third_scenario_is_named_rather_than_left_to_omission``.
    These two run in ~5 s and carry the finding on their own.
    """
    import test_regression_greenhouse as greenhouse
    import test_regression_harvest as harvest

    return {"greenhouse": greenhouse._final_state, "harvest": harvest._final_state}


def test_the_regulator_never_reverses_in_the_standalone_cabins() -> None:
    """The half of the frozen prose that is TRUE — and it is the control.

    Without this the plant-coupled result below would not identify the seam as the
    cause: "it reverses somewhere in the roster" is compatible with the regulator
    simply being unstable. It reverses in exactly the scenarios that grow a crop.
    """
    for name, run in _standalone_runners().items():
        with _recording() as calls:
            run()
        assert calls, f"{name} exercises no O2Makeup at all — the control is vacuous"
        peak = max(o2 for o2, _ in calls)
        assert peak == SETPOINT, (
            f"{name}: cabin_o2 peaked at {peak!r}, not the {SETPOINT!r} setpoint it "
            f"starts at — this scenario's O₂ has gained a source it did not have"
        )
        assert all(flux >= 0.0 for _, flux in calls), (
            f"{name}: the regulator ran backwards in a standalone cabin"
        )


def test_the_regulator_does_reverse_in_the_frozen_plant_coupled_scenarios() -> None:
    """The finding: "a deferred seam that never arises here" is false of the roster.

    Asserted as existence + an order-of-magnitude band, never as an exact count — the
    excursion is downstream of the biosphere's transcendentals (see the module
    docstring).
    """
    for name, run in _plant_coupled_runners().items():
        with _recording() as calls:
            run()
        reversed_calls = [(o2, f) for o2, f in calls if f < 0.0]
        assert reversed_calls, (
            f"{name}: no reversal found. Either the seam changed or the crop stopped "
            f"out-producing the crew — both are findings, and both make the frozen "
            f"prose in domains/eclss/flows.py true again. Re-measure before deleting."
        )
        peak = max(o2 for o2, _ in calls)
        assert SETPOINT < peak < SETPOINT * 1.01, (
            f"{name}: peak O₂ pool {peak!r} is outside the measured band — the "
            f"excursion was ~0.08 % over the {SETPOINT!r} setpoint when pinned"
        )


def test_the_reversal_is_the_biosphere_seam_not_a_cabin_stock() -> None:
    """The mechanism, structurally: the coupled builds hand it the crop's own pool.

    Pinned so the finding cannot be re-read as "the greenhouse cabin happens to
    run rich". The wiring is the cause, and it is P6.3's seam, not an author's.
    """
    from domains.biosphere.stocks import O2_POOL
    from domains.crew.loader import load_crew_params
    from domains.eclss.stocks import CABIN_O2
    from domains.eclss.system import O2_MAKEUP
    from station.greenhouse import build_greenhouse
    from station.scenario import GREENHOUSE_SCENARIO

    _state, _bio, cabin_reg = build_greenhouse(
        load_crew_params(), load_eclss_params(), GREENHOUSE_SCENARIO, with_plants=True
    )
    (makeup,) = [f for f in cabin_reg.flows if f.id == O2_MAKEUP]
    assert makeup.cabin_o2 == O2_POOL, (  # type: ignore[attr-defined]
        "the greenhouse regulator is no longer wired to the biosphere O₂ pool; the "
        "reversal finding's stated mechanism is stale"
    )
    assert makeup.cabin_o2 != CABIN_O2, (  # type: ignore[attr-defined]
        "the greenhouse regulator now watches a cabin stock — re-measure the finding"
    )


def test_the_uncovered_third_scenario_is_named_rather_than_left_to_omission() -> None:
    """``sealed_station`` reverses too, and is deliberately NOT pinned. Say so.

    Measured once (2026-08-11): **1 reversal in 1 756 800 calls**, peak 10.000304.
    It is not pinned because recording the flux requires re-running the trajectory —
    ``run_tier2`` is not cached at the function level (the session fixture caches
    *states*, and states cannot show a single mid-run call) — so a pin would add a
    fresh ~3 min Tier-2 run to the suite. ``post-roadmap-acceptance-gate.md`` measured
    exactly that cost and fought it down 22m34s → 6m47s, establishing the rule this
    defers to: **the number of expensive *tests*, not of expensive scenarios, sets the
    bill.** The finding is fully carried by ``greenhouse`` + ``harvest`` above, which
    run in ~5 s.

    This test asserts nothing about the run. It exists so the gap is a **stated**
    omission rather than a silent one, and so the helper it would have used is still
    proved importable — a "we chose not to cover this" note whose subject has been
    renamed away is worse than no note.
    """
    import sealed_tier2_helper

    assert callable(sealed_tier2_helper.run_tier2), (
        "the uncovered scenario's runner has moved; re-check whether the reversal "
        "measurement recorded in docs/plans/post-roadmap-o2-makeup-reversal.md still "
        "describes a scenario that exists"
    )
