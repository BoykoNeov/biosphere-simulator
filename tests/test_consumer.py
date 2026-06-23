"""Phase-3 Step-7 tests: the minimal consumer (one herbivore, the trophic pattern).

The optional stretch step — ``CONSUMER_CHAMBER_SCENARIO`` is the perennial sealed
chamber
(``run_perennial``, annual reset) plus **one herbivore** composed onto the *same* closed
ecosystem: grazing (``leaf_c → consumer_carbon``), consumer respiration
(``consumer_carbon + O₂ → carbon_pool``), and mortality (``consumer_carbon →
litter_carbon`` — death-to-litter, P3.4 / #6, never the loss-sink). It is the codebase's
existing minimal consumer (decomposition + microbial respiration — microbes eating
*dead*
litter) lifted **one trophic level** (eating *live* leaf), so the three flows mirror
``Decomposition`` / ``MicrobialRespiration`` / ``Senescence``.

Validates Step 7 with **zero** ``simcore`` changes (the flows + builder + scenario + a
herbivory param file are all under ``domains/biosphere/``):

1. **The three flows are balanced** (unit tests): grazing and mortality are
   single-currency CARBON transfers (Σ legs = 0); consumer respiration balances CARBON
   **and** OXYGEN in one flow (PQ=1, the composition fold). Mortality routes to the
   in-system ``litter_carbon`` POOL, never the BOUNDARY loss-sink.
2. **The trophic pattern composes under conservation / closure / ``rationed == 0``**
   (the run): the consumer **persists** (``consumer_carbon`` > 0 every step, a
   non-trivial
   peak), the carbon loss-sink stays **0.0** and ``events == ()`` (genuine closure —
   death
   routes to litter), all four quantities conserved every step incl. across the resets,
   ``rationed == 0`` (structural — first-order donor control), and the run is
   bit-identical
   on a re-run.
3. **The emergent cascade vs a no-consumer baseline** (direction-only, the Step-4/5/6
   anti-flakiness rule): the consumer is a carbon *recycler* — grazing lowers
   ``leaf_c``,
   the consumer respires the bulk back so ``carbon_pool`` *rises* (the "CO₂ up"
   signature),
   and ``consumer_carbon`` > 0. (litter is net-*lower*, not higher — the probe finding:
   the
   consumer shunts carbon to CO₂ faster than the senescence path; death-to-litter is
   real
   as a *flux*, asserted via the per-compartment ledger, not as a higher stock.)
4. **The per-compartment boundary ledger balances every step incl. the CONSUMERS leaf**
   (the Step-5/6 machinery, reset-aware leg reconstruction): every ``(domain,
   quantity)``
   residual ≈ 0, and CONSUMERS shows **both** crossing directions for CARBON (in from
   grazing, out to respiration + mortality) — the cross-compartment trophic cycling.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from dataclasses import replace
from pathlib import Path

import pytest

from domains.biosphere.compartments import (
    CONSUMERS,
    compartment_boundary_ledger,
    expected_extinction_residuals,
)
from domains.biosphere.herbivory import (
    ConsumerMortality,
    ConsumerRespiration,
    Grazing,
    HerbivoryParams,
)
from domains.biosphere.season import (
    CARBON_POOL,
    CONSUMER_CARBON,
    CONSUMER_CHAMBER_SCENARIO,
    CONSUMER_CHAMBER_YEARS,
    LEAF_C,
    LITTER_CARBON,
    O2_POOL,
    build_season,
    run_perennial,
    weather_resolver,
)
from simcore.boundary import loss_sink_id
from simcore.environment import SourceResolver
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


_YEAR = len(_weather())
# A representative within-year-1 growing-season index for the direction-only cascade
# comparison (mid-season — leaf substantial, the grazed gap clear; the Step-6 @80
# idiom).
_PROBE_DAY = 80

# Per-quantity conservation / ledger tolerances (the perennial-chamber table).
_TOL = {
    Quantity.CARBON: 1e-12,
    Quantity.OXYGEN: 1e-11,
    Quantity.WATER: 1e-7,
    Quantity.NITROGEN: 1e-9,
}


# --- the three flows are balanced (unit tests) ------------------------------
# Minimal hand-built states (the microbial-respiration / decomposition idiom), so the
# rate-law + balance are isolated from the season's full assembly.

_LEAF = LEAF_C
_CONS = CONSUMER_CARBON
_POOL = CARBON_POOL
_O2 = O2_POOL
_LITTER = LITTER_CARBON


def _env(state: State, dt: float):
    # The consumer flows read no forcing (donor-controlled on their stocks); a trivial
    # bound resolver suffices.
    return SourceResolver(forcings={}).bind(state, dt)


def _carbon_pop(sid, amount: float) -> Stock:
    carbon = canonical_unit(Quantity.CARBON)
    return Stock(
        id=sid,
        domain=CONSUMERS,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=0.0,
    )


def _carbon_pool(sid, amount: float) -> Stock:
    carbon = canonical_unit(Quantity.CARBON)
    return Stock(
        id=sid,
        domain=CONSUMERS,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=amount,
        kind=StockKind.POOL,
    )


def _co2_pool(sid, amount: float) -> Stock:
    return Stock(
        id=sid,
        domain=CONSUMERS,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
        composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
    )


def _o2_pool(sid, amount: float) -> Stock:
    return Stock(
        id=sid,
        domain=CONSUMERS,
        quantity=Quantity.OXYGEN,
        unit=canonical_unit(Quantity.OXYGEN),
        amount=amount,
        kind=StockKind.POOL,
        composition={Quantity.OXYGEN: 2.0},
    )


_PARAMS = HerbivoryParams(
    grazing_rate=0.1,
    respiration_rate=0.05,
    mortality_rate=0.02,
    o2_half_saturation=0.0,  # f_O2 ≡ 1 for O₂ > 0 — isolate the base flux in unit tests
)


def test_grazing_transfers_leaf_to_consumer_balanced() -> None:
    # leaf_c → consumer_carbon: a single-currency CARBON transfer (Σ legs = 0), the
    # Decomposition pattern one trophic level up (eating live tissue).
    state = State(
        n=0,
        stocks={s.id: s for s in (_carbon_pop(_LEAF, 0.4), _carbon_pop(_CONS, 0.02))},
        rng_seed=0,
    )
    flow = Grazing(
        FlowId("biosphere.grazing"),
        0,
        leaf_c=_LEAF,
        consumer_carbon=_CONS,
        params=_PARAMS,
    )
    result = flow.evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    grazed = 0.1 * 0.4
    assert math.isclose(legs[_LEAF], -grazed, rel_tol=1e-12)
    assert math.isclose(legs[_CONS], grazed, rel_tol=1e-12)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.CARBON}


def test_grazing_self_limits_at_zero_leaf() -> None:
    # No standing leaf ⇒ zero intake (donor-controlled positivity — rationed == 0 is
    # structural).
    state = State(
        n=0,
        stocks={s.id: s for s in (_carbon_pop(_LEAF, 0.0), _carbon_pop(_CONS, 0.02))},
        rng_seed=0,
    )
    flow = Grazing(
        FlowId("biosphere.grazing"),
        0,
        leaf_c=_LEAF,
        consumer_carbon=_CONS,
        params=_PARAMS,
    )
    assert all(
        leg.amount == 0.0 for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
    )


def test_consumer_respiration_balances_carbon_and_oxygen() -> None:
    # consumer_carbon + O₂ → CO₂ pool: ONE flow balances CARBON *and* OXYGEN (PQ=1, the
    # composition fold) — the MicrobialRespiration shape. Three legs, no source==sink.
    state = State(
        n=0,
        stocks={
            s.id: s
            for s in (
                _carbon_pop(_CONS, 0.03),
                _co2_pool(_POOL, 2.0),
                _o2_pool(_O2, 200.0),
            )
        },
        rng_seed=0,
    )
    flow = ConsumerRespiration(
        FlowId("biosphere.consumer_respiration"),
        0,
        consumer_carbon=_CONS,
        co2_pool=_POOL,
        o2_pool=_O2,
        params=_PARAMS,
        air_mol=1000.0,
    )
    result = flow.evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    respired = 0.05 * 0.03
    assert math.isclose(legs[_CONS], -respired, rel_tol=1e-12)
    assert math.isclose(legs[_POOL], respired, rel_tol=1e-12)
    assert math.isclose(legs[_O2], -respired, rel_tol=1e-12)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {
        Quantity.CARBON,
        Quantity.OXYGEN,
    }


def test_consumer_mortality_routes_to_litter_not_loss_sink() -> None:
    # consumer_carbon → litter_carbon: death-to-litter (P3.4 / #6). A single-currency
    # CARBON transfer to the in-system POOL — the carcass is decomposable, the loss-sink
    # is the numerical guard only.
    state = State(
        n=0,
        stocks={
            s.id: s for s in (_carbon_pop(_CONS, 0.03), _carbon_pool(_LITTER, 1.0))
        },
        rng_seed=0,
    )
    flow = ConsumerMortality(
        FlowId("biosphere.consumer_mortality"),
        0,
        consumer_carbon=_CONS,
        litter_carbon=_LITTER,
        params=_PARAMS,
    )
    result = flow.evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    died = 0.02 * 0.03
    assert math.isclose(legs[_CONS], -died, rel_tol=1e-12)
    assert math.isclose(legs[_LITTER], died, rel_tol=1e-12)  # to the litter POOL
    assert _LITTER == LITTER_CARBON  # the in-system pool, not loss_sink_id(CARBON)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.CARBON}


def test_consumer_respiration_self_limits_and_dt_linear() -> None:
    # No standing consumer ⇒ zero gas exchange; flux = daily·dt (dt-linear, RK4-ready).
    empty = State(
        n=0,
        stocks={
            s.id: s
            for s in (
                _carbon_pop(_CONS, 0.0),
                _co2_pool(_POOL, 2.0),
                _o2_pool(_O2, 200.0),
            )
        },
        rng_seed=0,
    )
    flow = ConsumerRespiration(
        FlowId("biosphere.consumer_respiration"),
        0,
        consumer_carbon=_CONS,
        co2_pool=_POOL,
        o2_pool=_O2,
        params=_PARAMS,
        air_mol=1000.0,
    )
    assert all(
        leg.amount == 0.0 for leg in flow.evaluate(empty, _env(empty, 1.0), 1.0).legs
    )
    state = State(
        n=0,
        stocks={
            s.id: s
            for s in (
                _carbon_pop(_CONS, 0.03),
                _co2_pool(_POOL, 2.0),
                _o2_pool(_O2, 200.0),
            )
        },
        rng_seed=0,
    )
    half = {
        leg.stock: leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
    }
    full = {
        leg.stock: leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
    }
    assert math.isclose(full[_POOL], 2.0 * half[_POOL], rel_tol=1e-12)


# --- the run: the trophic pattern composes (closure / conservation / rationed) ------


def _run(scenario):
    weather = _weather() * CONSUMER_CHAMBER_YEARS
    state, registry = build_season(scenario)
    resolver = weather_resolver(weather, scenario)
    states, rationed, events = run_perennial(
        EulerIntegrator(registry),
        state,
        scenario,
        resolver,
        1.0,
        len(weather),
        year=_YEAR,
    )
    return states, rationed, events, registry, resolver


@pytest.fixture(scope="module")
def consumer_run():
    return _run(CONSUMER_CHAMBER_SCENARIO)


@pytest.fixture(scope="module")
def baseline_run():
    # The SAME scenario minus the consumer (== the perennial chamber): the cascade
    # reference. consumer=False keeps the consumers leaf empty (byte-identical to the
    # perennial golden trajectory).
    return _run(replace(CONSUMER_CHAMBER_SCENARIO, consumer=False))


def _total(state: State, q: Quantity) -> float:
    return sum(s.amount * s.composition.get(q, 0.0) for s in state.stocks.values())


def test_consumer_persists_and_is_nontrivial(consumer_run) -> None:
    # The herbivore is alive every step after sowing (donor-controlled grazing keeps
    # refilling it while leaf > 0) and reaches a non-trivial peak — not a vacuous trace.
    states, _, _, _, _ = consumer_run
    biomass = [s.stocks[CONSUMER_CARBON].amount for s in states]
    assert all(b > 0.0 for b in biomass[1:])  # alive every step after t=0
    assert max(biomass) > 0.02  # a non-trivial standing biomass (probe: ~0.034)


def test_consumer_genuinely_closed(consumer_run) -> None:
    # The Step-7 closure headline: the consumer's death routes to the in-system
    # litter_carbon POOL (and is respired to the chamber CO₂ pool), never to the
    # BOUNDARY
    # loss-sink — so the chamber stays genuinely closed even with a trophic level added.
    states, _, events, _, _ = consumer_run
    assert events == ()
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states)


def test_consumer_never_rations(consumer_run) -> None:
    # rationed == 0 is structural: every consumer flow is first-order donor-controlled
    # (k·pool, self-limiting → 0), summed with the producer withdrawals on leaf below 1.
    _, rationed, _, _, _ = consumer_run
    assert rationed == 0


@pytest.mark.parametrize(
    ("quantity", "abs_tol"),
    [
        (Quantity.CARBON, 1e-12),
        (Quantity.OXYGEN, 1e-11),
        (Quantity.WATER, 1e-7),
        (Quantity.NITROGEN, 1e-9),
    ],
)
def test_consumer_conserves_every_quantity(consumer_run, quantity, abs_tol) -> None:
    # All four quantities conserved on every stored step incl. across the resets — the
    # consumer's three flows are each balanced, so adding the trophic level changes no
    # total (grazing/mortality are pure CARBON transfers; respiration balances C+O at
    # PQ=1).
    states, _, _, _, _ = consumer_run
    q0 = _total(states[0], quantity)
    for s in states:
        assert math.isclose(_total(s, quantity), q0, rel_tol=0.0, abs_tol=abs_tol)


def test_consumer_run_is_deterministic(consumer_run) -> None:
    # Bit-identical on a re-run (the golden's premise; the consumer flows are pure).
    states, rationed, events, _, _ = consumer_run
    states2, rationed2, events2, _, _ = _run(CONSUMER_CHAMBER_SCENARIO)
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_annual_reset_leaves_the_consumer_untouched(consumer_run) -> None:
    # annual_reset is plant-only: the herbivore persists across the re-sow (it is not
    # culled at harvest). consumer_carbon is continuous across each year boundary (the
    # reset only redistributes plant organ/grain carbon to litter + seedling).
    from domains.biosphere.season import annual_reset

    states, _, _, _, _ = consumer_run
    for y in range(1, CONSUMER_CHAMBER_YEARS):
        before = states[y * _YEAR]
        after = annual_reset(before, CONSUMER_CHAMBER_SCENARIO)
        assert (
            after.stocks[CONSUMER_CARBON].amount
            == before.stocks[CONSUMER_CARBON].amount
        )


# --- the emergent cascade vs the no-consumer baseline (direction-only) --------------


def test_cascade_leaf_down_co2_up_vs_baseline(consumer_run, baseline_run) -> None:
    # Direction-only (never State == State, never a magnitude/day index): at a
    # representative within-year-1 index the consumer run has LOWER leaf (grazed),
    # HIGHER
    # carbon_pool (the consumer respires carbon back that photosynthesis would otherwise
    # keep drawn down — the "CO₂ rises" signature, here from a respiring consumer), and
    # a
    # live consumer. The cascade is emergent — no cascade code.
    cons, _, _, _, _ = consumer_run
    base, _, _, _, _ = baseline_run
    assert (
        cons[_PROBE_DAY].stocks[LEAF_C].amount < base[_PROBE_DAY].stocks[LEAF_C].amount
    )
    assert (
        cons[_PROBE_DAY].stocks[CARBON_POOL].amount
        > base[_PROBE_DAY].stocks[CARBON_POOL].amount
    )
    assert cons[_PROBE_DAY].stocks[CONSUMER_CARBON].amount > 0.0


# --- the per-compartment boundary ledger (Step-5/6 machinery, CONSUMERS leaf) -------


def _legs(before_step: State, registry, resolver):
    """The step's applied legs, reconstructed by one evaluate at ``before_step``.

    Euler + ``rationed == 0`` ⇒ one evaluation at the (reset-aware) start-of-step state
    equals the applied legs (the Step-5/6 precedent). Binds the resolver to the SAME
    snapshot it evaluates against (the #16 seam).
    """
    bound = resolver.bind(before_step, 1.0)
    return [flow.evaluate(before_step, bound, 1.0) for flow in registry.flows]


def test_per_compartment_ledger_balances_every_step(consumer_run) -> None:
    # The architecture proof under a 5th leaf: for EVERY step / quantity / compartment
    # (incl. CONSUMERS and boundary) the apply-integrity residual is ≈ 0 (events == () ⇒
    # zero extinction correction). Reset-aware reconstruction mirrors run_perennial's
    # predicate verbatim so the reset sits OUTSIDE the transition the ledger sees.
    from domains.biosphere.season import annual_reset

    states, _, events, registry, resolver = consumer_run
    assert events == ()
    saw_consumers = False
    for i in range(len(states) - 1):
        before = (
            annual_reset(states[i], CONSUMER_CHAMBER_SCENARIO)
            if (i > 0 and i % _YEAR == 0)
            else states[i]
        )
        after = states[i + 1]
        ledger = compartment_boundary_ledger(
            before, after, _legs(before, registry, resolver)
        )
        step_events = [e for e in events if e.n == before.n]
        expected = expected_extinction_residuals(before, step_events)
        for entry in ledger:
            corr = expected.get((entry.domain, entry.quantity), 0.0)
            assert abs(entry.residual - corr) <= _TOL[entry.quantity], (
                f"step {i} {entry.domain} {entry.quantity.name}: "
                f"residual {entry.residual} vs correction {corr}"
            )
            if entry.domain == CONSUMERS:
                saw_consumers = True
    assert saw_consumers  # non-vacuity: the CONSUMERS leaf was actually in the ledger


def test_consumers_leaf_cycles_carbon_both_directions(consumer_run) -> None:
    # The cross-compartment trophic cycling (the reporting payoff): over the run the
    # CONSUMERS leaf has BOTH summed crossing_in > 0 (grazing draws plants→consumers)
    # AND
    # summed crossing_out > 0 (consumer respiration → atmosphere + mortality → soil) for
    # CARBON. Death-to-litter is real as this OUT flux (not as a higher litter stock).
    from domains.biosphere.season import annual_reset

    states, _, _, registry, resolver = consumer_run
    crossing_in = 0.0
    crossing_out = 0.0
    for i in range(len(states) - 1):
        before = (
            annual_reset(states[i], CONSUMER_CHAMBER_SCENARIO)
            if (i > 0 and i % _YEAR == 0)
            else states[i]
        )
        ledger = compartment_boundary_ledger(
            before, states[i + 1], _legs(before, registry, resolver)
        )
        for entry in ledger:
            if entry.domain == CONSUMERS and entry.quantity == Quantity.CARBON:
                crossing_in += entry.crossing_in
                crossing_out += entry.crossing_out
    assert crossing_in > 0.0  # grazing in (plants → consumers)
    assert crossing_out > 0.0  # respiration + mortality out (→ atmosphere / soil)
