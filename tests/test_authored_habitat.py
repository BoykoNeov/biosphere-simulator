"""The first authored habitat — `scenarios/algae_habitat.yaml` (post-roadmap).

A fully closed carbon+oxygen habitat authored entirely in a scenario file: the crew
eats algae and exhales CO2; the algae photosynthesise it back into biomass and O2; the
crew breathes the O2; a decomposer remineralises the egested carbon. **Zero boundary
stocks** — every atom stays inside. Design + the steady-state/stability arithmetic:
``docs/plans/post-roadmap-authored-habitat.md``.

**What these tests do and do NOT claim.** The platform guarantees *conservation +
determinism only* (Phase-9 decision B); the flow LAWS here are authored kinetics and
therefore UNCALIBRATED — invented first-order forms, not literature-derived. So there
is **no golden and no manifest entry**: this scenario is a runtime artifact, never
reference (``docs/authoring-reference.md``, "authored != validated"). The fixed-point
test below is an **internal-consistency** check — that the authored graph does what its
own design arithmetic predicts — *not* a scientific validation.

This lives outside ``tests/authoring/scenarios/`` on purpose: those files are test
fixtures and cross-port anchors, whereas ``scenarios/`` is authored *content* — the
thing the platform exists to make possible.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from authoring.interpreter import load_scenario
from authoring.run import run_scenario
from simcore.ids import StockId
from simcore.state import State

HABITAT = str(Path(__file__).parent.parent / "scenarios" / "algae_habitat.yaml")

FOOD = StockId("crew.food_store")
CO2 = StockId("cabin.co2")
O2 = StockId("cabin.o2")
BIOMASS = StockId("algae.biomass")
FECES = StockId("waste.feces")

# The carbon-bearing stocks (each 1:1 carbon except cabin.co2, which the composition
# fold counts as 1 carbon + 2 oxygen).
CARBON_STOCKS = (FOOD, CO2, BIOMASS, FECES)

# The analytic fixed point the design solves for (plan: "The steady state"). Derived,
# not fitted: k_harv*B = q -> B=1000; k_photo*L*C = (k_resp+k_harv)*B -> C=500;
# k_dec*W = (1-f)*q -> W=100; F = C_total - C - B - W = 1900; O2 = 3300 - C = 2800.
FIXED_POINT = {
    FOOD: 1900.0,
    CO2: 500.0,
    BIOMASS: 1000.0,
    FECES: 100.0,
    O2: 2800.0,
}


@pytest.fixture(scope="module")
def run() -> tuple[list[State], int]:
    """Interpret + run the sealed year once (~1 s); shared by every test below."""
    states, rationed, _events = run_scenario(load_scenario(HABITAT))
    return states, rationed


def _carbon(state: State) -> float:
    return sum(state.stocks[s].amount for s in CARBON_STOCKS)


def _oxygen_atoms(state: State) -> float:
    # cabin.o2 is {oxygen:2} and cabin.co2 is {oxygen:2} — the fold that closes O.
    return 2.0 * (state.stocks[O2].amount + state.stocks[CO2].amount)


# --- what the platform actually guarantees: conservation + determinism -----


def test_carbon_closes_with_no_boundary_leg(run: tuple[list[State], int]) -> None:
    """CARBON is conserved across the whole sealed year, to float roundoff.

    The habitat declares **no boundary stock**, so this is closure in the strict
    sense: inputs = outputs = 0, hence total carbon must be invariant. The engine
    asserts this every step; here we pin the endpoints explicitly.
    """
    states, _ = run
    start = _carbon(states[0])
    assert start == pytest.approx(3500.0)
    for state in (states[len(states) // 2], states[-1]):
        assert _carbon(state) == pytest.approx(start, rel=1e-12)


def test_oxygen_closes_via_the_composition_fold(run: tuple[list[State], int]) -> None:
    """OXYGEN is conserved — and only because CO2 carries {carbon:1, oxygen:2}.

    Every oxygen atom lives in cabin.o2 or cabin.co2, so their folded sum is
    invariant (equivalently O2 + CO2 = const). Mutation-verified: stripping the
    composition off cabin.co2 makes the interpreter reject crew.respiration at build
    time ("authored stoichiometry is not balanced for OXYGEN") — the station/cabin.py
    finding, now enforced against an authored file.
    """
    states, _ = run
    start = _oxygen_atoms(states[0])
    assert start == pytest.approx(6600.0)
    assert _oxygen_atoms(states[-1]) == pytest.approx(start, rel=1e-12)


def test_run_is_deterministic() -> None:
    """Two independent interpret+run passes are bit-identical, stock for stock."""
    a, _, _ = run_scenario(load_scenario(HABITAT))
    b, _, _ = run_scenario(load_scenario(HABITAT))
    assert len(a) == len(b)
    for sa, sb in zip(a, b, strict=True):
        for stock_id, stock in sa.stocks.items():
            # Exact equality: determinism is bit-identity within a build.
            assert stock.amount == sb.stocks[stock_id].amount


def test_arbitration_backstop_never_fires(run: tuple[list[State], int]) -> None:
    """Positivity comes from the kinetics, not from rationing.

    Every rate is strictly positive and every k*dt <= 4.3e-3, so donor-controlled
    draws self-limit long before a stock could go negative. A firing here would mean
    the authored constants are inconsistent with dt — the failure mode the design
    arithmetic exists to rule out.
    """
    _, rationed = run
    assert rationed == 0


def test_no_stock_ever_goes_negative(run: tuple[list[State], int]) -> None:
    """No stock crosses zero anywhere in the year.

    ``waste.feces`` legitimately *starts* empty, so the universal law here is
    non-negativity, not positivity (see the sharper food-store check below).
    """
    states, _ = run
    for stock_id in (*CARBON_STOCKS, O2):
        low = min(state.stocks[stock_id].amount for state in states)
        assert low >= 0.0, f"{stock_id} hit {low}"


def test_the_crew_is_fed_for_the_whole_year(run: tuple[list[State], int]) -> None:
    """The food store never runs dry — the closure condition, checked.

    This is the sharp one. The crew's draw is FORCED (it reads a forcing, never a
    stock), so unlike every other flow here its positivity is by **sizing**, not
    structure: the culture must sustain a biomass that feeds it (k_harv*B_ss = q). If
    harvest under-fed the crew, food_store would fall monotonically and cross zero.
    It dips during the transient (the culture starts at a fifth of its steady-state
    biomass) and recovers.
    """
    states, _ = run
    food = [state.stocks[FOOD].amount for state in states]
    assert min(food) > 1500.0, f"food store dipped to {min(food)}"
    # It really does dip before recovering — otherwise this test could pass
    # vacuously on a scenario whose crew never eats.
    assert min(food) < food[0]
    assert food[-1] > min(food)


# --- the scenario earns its keep: the loop is LIVE, and it works -----------


def test_the_loop_is_live_not_static(run: tuple[list[State], int]) -> None:
    """The habitat visibly runs: the culture blooms while scrubbing the cabin air.

    The run starts deliberately off the fixed point. A habitat that "closes" because
    nothing moves would satisfy every conservation test above and prove nothing — so
    assert the transient is real and in the intended direction.
    """
    states, _ = run
    first, last = states[0], states[-1]

    # The culture establishes: ~5x growth.
    assert first.stocks[BIOMASS].amount == pytest.approx(200.0)
    assert last.stocks[BIOMASS].amount > 4.5 * first.stocks[BIOMASS].amount
    # ...drawing the cabin CO2 down ~2.6x...
    assert last.stocks[CO2].amount < 0.5 * first.stocks[CO2].amount
    # ...and driving O2 up (the atoms the CO2 gave up).
    assert last.stocks[O2].amount > first.stocks[O2].amount
    # The decomposer's pool fills from empty.
    assert first.stocks[FECES].amount == 0.0
    assert last.stocks[FECES].amount > 50.0


def test_converges_to_the_analytic_fixed_point(run: tuple[list[State], int]) -> None:
    """The sealed year lands on the fixed point the design solves for, to ~0.1 %.

    **Internal consistency, not validation** — it checks the authored graph behaves
    as its own arithmetic predicts (the ODE steady state + a stable Jacobian: trace
    -1.8e-6 < 0, det 6.0e-13 > 0, slowest mode tau ~26 d, so a 1-year run is ~14
    tau). It makes no claim that these kinetics describe a real algal habitat.
    """
    states, _ = run
    for stock_id, predicted in FIXED_POINT.items():
        assert states[-1].stocks[stock_id].amount == pytest.approx(predicted, rel=1e-3)


# --- the "authored != validated" marker -----------------------------------


def test_run_is_marked_uncalibrated() -> None:
    """The habitat carries has_authored_kinetics — the honest marker.

    Every flow law here is authored, so the marker MUST be set: it is what makes
    Godot banner the run UNCALIBRATED and the graph dump mark it. The marker being
    True is the platform working as designed (decision B), not a defect.
    """
    assert load_scenario(HABITAT).has_authored_kinetics is True
