"""Phase-3 Step-5 (P3.1 ledger discharge): the FULL per-compartment boundary ledger.

Step 1 built the per-compartment boundary ledger and proved it on a hand transfer
(``test_compartments``); Step 3 ran it **live but WATER-scoped**
(``test_water_cycle``), explicitly deferring the full-ledger + extinction-exception
handling "to Step 5". This is that discharge — **no new science, no behavior change,
no new golden** (a diagnostics + verification step on the already-built ecosystem):

1. **Perennial full ledger, every step / quantity / compartment.** For the committed
   ``PERENNIAL_CHAMBER_SCENARIO`` (the full closed 4-leaf ecosystem) the apply-integrity
   identity ``net crossing flux == stored delta`` holds within a per-quantity tol on
   **every** step — including the four annual-reset boundary steps — for **every**
   ``(domain, quantity)`` the ledger returns, **boundary included** (no whitelist). The
   genuinely-closed run is extinction-free (``events == ()``), so the residual is ~0
   with **no** correction term: the headline proof that the four-compartment split is
   apply-correct.
2. **The extinction exception** — a balanced *non-flow* change the legs cannot see (the
   organ's compartment gains ``+r``, ``boundary`` gains ``-r``) — discharged by a
   **hand-built deterministic** one-step case (the sealed run does not go extinct over
   its horizon — probed — so the optional sealed full-ledger test is dropped, as
   designed) plus a pure unit test of the ``expected_extinction_residuals`` helper.
3. **The emergent cross-compartment demonstration** — carbon genuinely *cycles*
   through every active leaf (both crossing directions, robust gate) and the
   reset -> litter -> decomposition -> CO2 -> regrowth cascade shows up as a
   direction-only draw-down-then-recover of the chamber CO2 pool (loose narrative, no
   magnitude/timing — heeding ``test_perennial``'s "do NOT assert period-matching").

**Leg reconstruction is test-side** (no ``simcore`` change — ``git diff src/simcore/``
stays empty): under Euler + ``rationed == 0`` one ``flow.evaluate`` at the
start-of-step state equals the applied legs (the Step-3 precedent, generalized). The
annual reset is a pure, schedule-known transform, so the before-step state is
re-derived with ``annual_reset`` at each boundary — **mirroring ``run_perennial``'s
predicate verbatim** so it cannot drift — putting the reset *outside* the transition
(the ledger then sees only flow legs).

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
from pathlib import Path

import pytest

from domains.biosphere.compartments import (
    ATMOSPHERE,
    PLANTS,
    SOIL,
    compartment_boundary_ledger,
    expected_extinction_residuals,
)
from domains.biosphere.season import (
    CARBON_POOL,
    PERENNIAL_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_YEARS,
    SeasonScenario,
    annual_reset,
    build_season,
    run_perennial,
    weather_resolver,
)
from simcore.boundary import BOUNDARY_DOMAIN, loss_sink
from simcore.environment import Environment, SourceResolver
from simcore.events import ExtinctionEvent
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# The per-quantity tol table test_perennial already pins (a flat 1e-7 is far too loose
# for the O(1) CARBON amounts — it would hide a real misapplication).
TOL = {
    Quantity.CARBON: 1e-12,
    Quantity.OXYGEN: 1e-11,
    Quantity.WATER: 1e-7,
    Quantity.NITROGEN: 1e-9,
}


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


@pytest.fixture(scope="module")
def perennial() -> tuple:
    """The canonical perennial run + the registry/resolver for leg reconstruction.

    ``StepReport`` does not expose flow legs, so the ledger tests reconstruct them:
    under Euler + ``rationed == 0`` one ``evaluate`` at the (reset-aware) start-of-step
    state equals the applied legs. Module-scoped so the multi-year run executes once.
    """
    year = len(_weather())
    weather = _weather() * PERENNIAL_CHAMBER_YEARS
    steps = len(weather)
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        PERENNIAL_CHAMBER_SCENARIO,
        resolver,
        1.0,
        steps,
        year=year,
    )
    return (
        states,
        registry,
        resolver,
        PERENNIAL_CHAMBER_SCENARIO,
        rationed,
        events,
        year,
    )


def _before_step(
    states: list[State], i: int, year: int, scenario: SeasonScenario
) -> State:
    """The state the engine actually stepped from at transition ``i -> i+1``.

    ``run_season`` records the **pre-reset** trajectory, so on a reset boundary the
    engine stepped from ``annual_reset(states[i])``, not ``states[i]``. This re-derives
    it with the **verbatim** ``run_perennial`` predicate (``i > 0 and i % year == 0``)
    so the two cannot drift; the reset's non-flow carbon redistribution then lies
    *outside* the transition and the ledger sees only flow legs.
    """
    if i > 0 and i % year == 0:
        return annual_reset(states[i], scenario)
    return states[i]


def _legs(
    registry: Registry, resolver: SourceResolver, before_step: State
) -> list[FlowResult]:
    """The step's applied legs, reconstructed by one evaluate at ``before_step``.

    Binds the resolver to the **same** snapshot it evaluates against (the #16 seam,
    mirroring the engine); sound as the applied legs only when ``rationed == 0`` (no
    arbitration scaling, so ``_reduce(scaled) == _reduce(raw)``).
    """
    bound = resolver.bind(before_step, 1.0)
    return [flow.evaluate(before_step, bound, 1.0) for flow in registry.flows]


# --- headline: full per-compartment ledger, every step / quantity / domain ----------
def test_perennial_full_ledger_balances_every_step(perennial: tuple) -> None:
    # The deferred half of P3.1: apply-integrity (net crossing == stored delta) holds
    # within the per-quantity tol on EVERY step — incl. the four reset-boundary steps
    # (the post-reset reconstruction is the part most likely to have a bug) — for EVERY
    # (domain, quantity) the ledger returns, boundary INCLUDED (no whitelist: an
    # unexpected nonzero anywhere is exactly what this catches). The closed run is
    # extinction-free, so there is zero non-flow correction term.
    states, registry, resolver, scenario, rationed, events, year = perennial
    assert rationed == 0  # Euler-only precondition (no arbitration scaling)
    assert events == ()  # genuinely closed, so no extinction exception this run
    saw = {Quantity.CARBON: False, Quantity.OXYGEN: False, Quantity.WATER: False}
    for i in range(len(states) - 1):
        before_step = _before_step(states, i, year, scenario)
        ledger = compartment_boundary_ledger(
            before_step, states[i + 1], _legs(registry, resolver, before_step)
        )
        for entry in ledger:
            assert abs(entry.residual) <= TOL[entry.quantity], (i, entry)
            if entry.quantity in saw and (
                entry.crossing_in > 0.0 or entry.crossing_out > 0.0
            ):
                saw[entry.quantity] = True
    # Non-vacuity (the Step-3 saw_crossing precedent, per quantity): the residual check
    # exercises real crossing flux, not a frozen run that passes trivially.
    assert all(saw.values()), saw


# --- emergent cross-compartment dynamics: carbon genuinely cycles (robust gate) -----
def test_perennial_cross_compartment_carbon_cycles(perennial: tuple) -> None:
    # The "emergent cross-compartment dynamics" claim as a robust check: over the run
    # each active leaf has BOTH summed crossing_in > 0 AND summed crossing_out > 0 for
    # CARBON: carbon cycles through every leaf (photosynthesis draws atmosphere->plants;
    # respiration + the reset push plants->atmosphere/soil; decomposition pushes
    # soil->atmosphere), not a one-way drain. Summed/direction-only, never a per-step
    # magnitude (the period-2 cycle is in transient at year 5; no period-matching).
    states, registry, resolver, scenario, _, _, year = perennial
    crossing_in = {PLANTS: 0.0, SOIL: 0.0, ATMOSPHERE: 0.0}
    crossing_out = {PLANTS: 0.0, SOIL: 0.0, ATMOSPHERE: 0.0}
    for i in range(len(states) - 1):
        before_step = _before_step(states, i, year, scenario)
        ledger = compartment_boundary_ledger(
            before_step, states[i + 1], _legs(registry, resolver, before_step)
        )
        for entry in ledger:
            if entry.quantity is Quantity.CARBON and entry.domain in crossing_in:
                crossing_in[entry.domain] += entry.crossing_in
                crossing_out[entry.domain] += entry.crossing_out
    for leaf in (PLANTS, SOIL, ATMOSPHERE):
        assert crossing_in[leaf] > 0.0, (leaf, "no carbon crossed IN")
        assert crossing_out[leaf] > 0.0, (leaf, "no carbon crossed OUT")


def test_perennial_atmosphere_carbon_draws_down_then_recovers(perennial: tuple) -> None:
    # The reset -> litter -> decomposition -> CO2 -> regrowth cascade, "for free", as a
    # DIRECTION-ONLY narrative (never a magnitude or a day index): within every year the
    # chamber CO2 pool draws DOWN below its year-start value (photosynthesis), then
    # RECOVERS above that minimum before the reset (decomposition refuels it).
    # Both facts are bit-deterministic across years (the period-2 shape: a deep mid-year
    # dip some years, a shallow early dip others) keeps the direction universal.
    states, *_, year = perennial
    carbon_pool = [s.stocks[CARBON_POOL].amount for s in states]
    for y in range(PERENNIAL_CHAMBER_YEARS):
        segment = carbon_pool[y * year : (y + 1) * year + 1]
        trough = min(segment)
        assert trough < segment[0]  # drawn down within the growing year
        assert segment[-1] > trough  # recovered before the next reset


# --- the extinction exception: pure helper unit tests -------------------------------
def test_expected_extinction_residuals_books_organ_and_boundary() -> None:
    # The correction the full ledger adds on an extinction step: +r to the organ's
    # compartment (where the snapped POPULATION mass vanished, unaccounted by any leg)
    # and -r to boundary (where the loss-sink gained it, also legless). Maps the event
    # by the extinct stock's domain (read from `before`) and quantity.
    organ = StockId("bio.pop")
    before = State(
        n=0, stocks={organ: _population(organ, PLANTS, 0.42, threshold=0.5)}, rng_seed=0
    )
    event = ExtinctionEvent(n=1, stock=organ, quantity=Quantity.CARBON, residual=0.3)
    assert expected_extinction_residuals(before, [event]) == {
        (PLANTS, Quantity.CARBON): 0.3,
        (BOUNDARY_DOMAIN, Quantity.CARBON): -0.3,
    }


def test_expected_extinction_residuals_empty_events_is_empty() -> None:
    # No extinctions, so no correction (the clean step: the raw ledger residual stands).
    assert expected_extinction_residuals(State(n=0, stocks={}, rng_seed=0), []) == {}


def test_expected_extinction_residuals_accumulates_within_a_compartment() -> None:
    # Two POPULATION stocks in the SAME (compartment, quantity) extinct in one step: the
    # helper sums their residuals (+ to the shared organ compartment, - to boundary). It
    # folds events in tuple order; on a real step that order is the integrator's sorted
    # stock-id order, which also drives its loss-sink bucketing, so the helper and the
    # ledger agree. Multi-extinction under perturbations is a Step-6 path — the live
    # float-order agreement is to be verified there against the real ledger.
    a, b = StockId("bio.pop_a"), StockId("bio.pop_b")
    before = State(
        n=0,
        stocks={
            a: _population(a, PLANTS, 0.2, threshold=0.5),
            b: _population(b, PLANTS, 0.3, threshold=0.5),
        },
        rng_seed=0,
    )
    events = (
        ExtinctionEvent(n=1, stock=a, quantity=Quantity.CARBON, residual=0.2),
        ExtinctionEvent(n=1, stock=b, quantity=Quantity.CARBON, residual=0.3),
    )
    assert expected_extinction_residuals(before, events) == {
        (PLANTS, Quantity.CARBON): pytest.approx(0.5),
        (BOUNDARY_DOMAIN, Quantity.CARBON): pytest.approx(-0.5),
    }


# --- the extinction exception: hand-built deterministic discharge (PRIMARY) ----------
_POP = StockId("test.pop")
_PLANT_POOL = StockId("test.plant_c")
_ATMOS_POOL = StockId("test.atmos_c")


def _carbon(sid: StockId, domain: DomainId, amount: float) -> Stock:
    """A 1:1 CARBON POOL stock (composition defaults to ``{CARBON: 1.0}``)."""
    return Stock(
        id=sid,
        domain=domain,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


def _population(
    sid: StockId, domain: DomainId, amount: float, *, threshold: float
) -> Stock:
    """A POPULATION CARBON stock (extinction-eligible, decision #6)."""
    return Stock(
        id=sid,
        domain=domain,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=threshold,
    )


class _CrossingFlow:
    """A fixed, balanced CARBON transfer ``src -> dst`` (one clean crossing flow).

    ``src``/``dst`` live in different compartments, so the ledger classifies it as a
    crossing flow; the withdrawal stays well under ``src``'s amount so the Euler
    backstop never fires (``rationed == 0``, the leg-reconstruction precondition).
    """

    def __init__(self, fid: FlowId, src: StockId, dst: StockId, amount: float) -> None:
        self._id = fid
        self.src = src
        self.dst = dst
        self.amount = amount

    @property
    def id(self) -> FlowId:
        return self._id

    @property
    def priority(self) -> int:
        return 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        x = self.amount * dt
        return FlowResult(legs=(Leg(self.src, -x), Leg(self.dst, x)))


def test_handbuilt_extinction_step_residual_and_correction() -> None:
    # The deterministic discharge of the Step-1 forward-pointer (not "hope the sealed
    # run dies"): a two-compartment state with one below-threshold POPULATION organ in
    # PLANTS and one clean crossing flow PLANTS->ATMOSPHERE, stepped once (Euler). The
    # organ snaps to 0 and its residual routes to the boundary loss-sink — a balanced
    # NON-FLOW change the legs cannot see. The RAW ledger residual is +r on PLANTS and
    # -r on boundary (ATMOSPHERE, the clean crossing target, stays ~0), and the helper
    # zeroes the exception exactly — leaving every compartment balanced.
    ls = loss_sink(Quantity.CARBON)
    pop = _population(_POP, PLANTS, 0.001, threshold=0.01)
    plant_pool = _carbon(_PLANT_POOL, PLANTS, 10.0)
    atmos_pool = _carbon(_ATMOS_POOL, ATMOSPHERE, 0.0)
    stocks = {s.id: s for s in (pop, plant_pool, atmos_pool, ls)}
    before = State(n=0, stocks=stocks, rng_seed=0)
    flow = _CrossingFlow(FlowId("test.cross"), _PLANT_POOL, _ATMOS_POOL, 1.0)
    registry = Registry([flow], stocks)

    report = EulerIntegrator(registry).step_report(before, SourceResolver(), 1.0)
    after = report.state

    # Exactly one extinction; r is the snapped (pre-snap) amount, routed to the sink.
    assert report.rationed == 0  # the leg-reconstruction precondition holds
    assert len(report.events) == 1
    r = report.events[0].residual
    assert r == 0.001
    assert after.stocks[_POP].amount == 0.0
    assert after.stocks[ls.id].amount == pytest.approx(r)

    # Reconstruct the legs (Euler) and run the RAW ledger over before -> after.
    bound = SourceResolver().bind(before, 1.0)
    results = [f.evaluate(before, bound, 1.0) for f in registry.flows]
    ledger = compartment_boundary_ledger(before, after, results)
    by = {(e.domain, e.quantity): e for e in ledger}

    # The exception: +r on the organ compartment, -r on boundary; crossing target clean.
    assert by[(PLANTS, Quantity.CARBON)].residual == pytest.approx(r)
    assert by[(BOUNDARY_DOMAIN, Quantity.CARBON)].residual == pytest.approx(-r)
    assert abs(by[(ATMOSPHERE, Quantity.CARBON)].residual) < 1e-12

    # The helper books that correction, so the corrected residual is ~0 everywhere.
    expected = expected_extinction_residuals(before, report.events)
    for e in ledger:
        corrected = e.residual - expected.get((e.domain, e.quantity), 0.0)
        assert abs(corrected) < 1e-12, (e, expected)
