"""The second authored habitat — `scenarios/bioregenerative_station.yaml`.

A bioregenerative life-support station in which **CARBON, OXYGEN and WATER each close
with zero boundary stocks** while **ENERGY is deliberately open** (4000 W in from the
sun, 4000 W out to space). Design + the full steady-state/Jacobian arithmetic:
``docs/plans/post-roadmap-bioregenerative-station.md``.

**What is new here, against `tests/test_authored_habitat.py`.** The first habitat
invented all six of its flow laws and closed with a finding: the flow registry was
crew-only, so an authored ecosystem *had* to invent its kinetics. The registry has since
grown 3 → 12, the grammar gained ``monod``, and authoring gained a coupling cadence.
This habitat is the first authored content to use any of it: **7 of its 17 flows are
frozen types**, wired so that the calibrated equipment *recycles* (the scrubber's
``co2_removed`` feeds the bioreactor, the regulator's ``o2_supply`` is a tank
photosynthesis refills) instead of discarding.

**What these tests do and do NOT claim.** The platform guarantees *conservation +
determinism only* (Phase-9 decision B). The ten ``kinetics`` laws here are authored and
therefore UNCALIBRATED, and the frozen types buy frozen **form**, not endorsement —
every ECLSS / power / thermal value they read is a ``DESIGN`` placeholder whose own
param file says no source *can* fix it (``flow_registry.py``: "Registered ≠
calibrated"). So there is **no golden and no manifest entry**: a runtime artifact, never
reference. The fixed-point tests below are **internal-consistency** checks — that the
authored graph does what its own arithmetic predicts — *not* scientific validation.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from authoring.interpreter import load_scenario
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from domains.thermal.flows import temperature
from simcore.ids import StockId
from simcore.state import State

STATION = str(
    Path(__file__).parent.parent / "scenarios" / "bioregenerative_station.yaml"
)

FOOD = StockId("crew.food_store")
CABIN_CO2 = StockId("cabin.co2")
CABIN_O2 = StockId("cabin.o2")
HUMIDITY = StockId("cabin.humidity")
FEED = StockId("bioreactor.co2_feed")
BIOMASS = StockId("bioreactor.biomass")
O2_TANK = StockId("store.o2_tank")
WATER = StockId("crew.water_store")
FECES = StockId("waste.feces")
URINE = StockId("waste.urine")
CONDENSATE = StockId("water.condensate")
BATTERY = StockId("power.battery")
NODE = StockId("thermal.node")
SOLAR = StockId("boundary.solar_source")
SPACE = StockId("boundary.space")

# The three matter books, each with NO boundary member — that absence IS the
# closure claim.
CARBON_STOCKS = (FOOD, CABIN_CO2, FEED, BIOMASS, FECES)
OXYGEN_STOCKS = (CABIN_O2, O2_TANK, CABIN_CO2, FEED)  # each folds to 2 oxygen atoms
WATER_STOCKS = (WATER, HUMIDITY, CONDENSATE, URINE)
# ENERGY is the one book that DOES include boundaries — it is open by construction.
ENERGY_STOCKS = (BATTERY, NODE, SOLAR, SPACE)

INTERIOR = CARBON_STOCKS + OXYGEN_STOCKS[:2] + WATER_STOCKS + (BATTERY, NODE)

# The analytic fixed point the design solves for. DERIVED, NOT FITTED — see the plan
# doc's "The steady state":
#     k_h*B     = q          -> B  = 1000
#     k_scrub*Cc = f*q       -> Cc = 0.4745
#     k_d*W     = (1-f)*q    -> W  = 12.75
#     V_max*phi = (k_h+k_r)*B -> phi = 0.6 -> Cn = K*phi/(1-phi) = 450
#     Oc        = S* - f*q/k_makeup           = 9.76275
# The two slack stocks (crew.food_store, store.o2_tank) close their own books.
FIXED_POINT = {
    FOOD: 736.7755,
    CABIN_CO2: 0.4745,
    FEED: 450.0,
    BIOMASS: 1000.0,
    FECES: 12.75,
    CABIN_O2: 9.76275,
    O2_TANK: 147.76275,
    HUMIDITY: 0.0405,
    CONDENSATE: 2.025,
    URINE: 4.875,
    WATER: 53.0595,
    NODE: 2.99127e9,
}

SOLAR_POWER_W = 4000.0
RUN_SECONDS = 8760 * 3600.0
O2_SETPOINT = 10.0  # the frozen eclss value the regulator drives toward


def _spec() -> ScenarioSpec:
    """The validated file as *written*, before interpretation.

    ``load_scenario`` returns a ``BuiltScenario`` (stocks already lowered, flows already
    resolved to frozen classes), which is the wrong altitude for the structural tests
    below: they are about what the scenario file **declares** — which flows are frozen
    type selections, where the frozen wiring fields point, which stocks are boundary.
    """
    return ScenarioSpec.model_validate(load_yaml(STATION))


@pytest.fixture(scope="module")
def run() -> tuple[list[State], int]:
    """Interpret + run the sealed year once (~9 s); shared by every test below."""
    states, rationed, _events = run_scenario(load_scenario(STATION))
    return states, rationed


def _carbon(state: State) -> float:
    return sum(state.stocks[s].amount for s in CARBON_STOCKS)


def _oxygen_atoms(state: State) -> float:
    # cabin.o2 / store.o2_tank are {oxygen:2}; cabin.co2 / bioreactor.co2_feed are
    # {carbon:1, oxygen:2}. Every oxygen atom in the station lives in one of the four.
    return 2.0 * sum(state.stocks[s].amount for s in OXYGEN_STOCKS)


def _water(state: State) -> float:
    return sum(state.stocks[s].amount for s in WATER_STOCKS)


def _energy(state: State) -> float:
    return sum(state.stocks[s].amount for s in ENERGY_STOCKS)


# --- what the platform actually guarantees: conservation + determinism -----


def test_carbon_closes_with_no_boundary_leg(run: tuple[list[State], int]) -> None:
    """CARBON is conserved across the sealed year, to float roundoff.

    No carbon-bearing stock is a boundary, so this is closure in the strict sense:
    inputs = outputs = 0, hence total carbon must be invariant. Every mole the crew
    exhales reaches the culture only by passing through the FROZEN scrubber, whose
    ``co2_removed`` field is wired to an interior pool rather than a sink.
    """
    states, _ = run
    start = _carbon(states[0])
    assert start == pytest.approx(2200.0)
    for state in (states[len(states) // 2], states[-1]):
        assert _carbon(state) == pytest.approx(start, rel=1e-11)


def test_oxygen_closes_via_the_composition_fold(run: tuple[list[State], int]) -> None:
    """OXYGEN is conserved — only because both CO2 pools carry the composition fold.

    The fold is what ties the crew's respiration legs together across two quantities. It
    also survives the FROZEN regulator sitting inside the loop: ``eclss.o2_makeup`` is a
    two-leg one-magnitude transfer between two stocks that fold identically, so it is
    oxygen-neutral automatically — the leg-shape rule the design rests on.
    """
    states, _ = run
    start = _oxygen_atoms(states[0])
    assert start == pytest.approx(1216.0)
    for state in (states[len(states) // 2], states[-1]):
        assert _oxygen_atoms(state) == pytest.approx(start, rel=1e-11)


def test_water_closes_with_no_boundary_leg(run: tuple[list[State], int]) -> None:
    """WATER is conserved — the quantity the first authored habitat put out of scope.

    ``store -> humidity -> condensate -> store`` and ``store -> urine -> store``, with
    the FROZEN condenser's ``humidity_condensate`` field wired to an interior pool.
    Recovery is 100 % here, an idealisation stated plainly in the scenario header.
    """
    states, _ = run
    start = _water(states[0])
    assert start == pytest.approx(60.0)
    for state in (states[len(states) // 2], states[-1]):
        assert _water(state) == pytest.approx(start, rel=1e-11)


def test_energy_balances_but_is_deliberately_open(run: tuple[list[State], int]) -> None:
    """ENERGY balances *including* its boundaries: Inputs = Outputs + dStored.

    Matter closes; energy cannot, and claiming otherwise would be wrong — a station runs
    on the difference between a 5800 K source and a 2.7 K sink. So the assertion here is
    the augmented one, and the two boundary stocks are load-bearing rather than an
    embarrassment. The sun's ledger is exactly ``solar_power * elapsed``.
    """
    states, _ = run
    first, last = states[0], states[-1]
    assert _energy(last) == pytest.approx(_energy(first), rel=1e-12)

    taken_in = -last.stocks[SOLAR].amount
    rejected = last.stocks[SPACE].amount
    stored = (last.stocks[NODE].amount - first.stocks[NODE].amount) + (
        last.stocks[BATTERY].amount - first.stocks[BATTERY].amount
    )
    assert taken_in == pytest.approx(SOLAR_POWER_W * RUN_SECONDS, rel=1e-12)
    assert taken_in == pytest.approx(rejected + stored, rel=1e-11)
    # And it really is open: both boundary legs are large, not incidental.
    assert taken_in > 1e11
    assert rejected > 1e11


def test_run_is_deterministic(run: tuple[list[State], int]) -> None:
    """A second independent interpret+run pass is bit-identical, stock for stock."""
    states, _ = run
    again, _rationed, _events = run_scenario(load_scenario(STATION))
    assert len(again) == len(states)
    for sa, sb in zip(states, again, strict=True):
        for stock_id, stock in sa.stocks.items():
            # Exact equality: determinism is bit-identity within a build.
            assert stock.amount == sb.stocks[stock_id].amount


def test_arbitration_backstop_never_fires(run: tuple[list[State], int]) -> None:
    """Positivity comes from the kinetics and the sizing, not from rationing.

    ``run_scenario`` now *raises* ``RationedError``, so a completed run is already the
    gate; asserted explicitly anyway. Every donor-controlled ``k*h`` is <= 0.60 (the
    frozen O2 regulator, which sets ``n_sub = 12``), and the one flow with no ``k*h``
    guarantee to lean on — ``thermal.radiator_reject``'s T^4 law — is covered here by
    measurement rather than by argument.
    """
    _, rationed = run
    assert rationed == 0


def test_no_stock_ever_goes_negative(run: tuple[list[State], int]) -> None:
    """No interior stock crosses zero anywhere in the year.

    ``boundary.solar_source`` is excluded: it is an ``unclamped`` source whose
    negative-going amount is pure ledger bookkeeping (the integral of energy taken in).
    Several interior pools legitimately *start* empty, so the universal law is
    non-negativity, not positivity.
    """
    states, _ = run
    for stock_id in INTERIOR:
        low = min(state.stocks[stock_id].amount for state in states)
        assert low >= 0.0, f"{stock_id} hit {low}"


def test_the_crew_is_fed_and_watered_for_the_whole_year(
    run: tuple[list[State], int],
) -> None:
    """Neither consumable store runs dry — the closure conditions, checked.

    Both crew draws are FORCED (they read forcings, never a stock), so unlike every
    donor-controlled flow their positivity is by **sizing**, not structure: the culture
    must sustain a biomass that feeds the crew (``k_harv * B* = q``) and the processors
    must return the water the crew loses. If harvest under-fed the crew, ``food_store``
    would fall without limit and cross zero.
    """
    states, _ = run
    food = [state.stocks[FOOD].amount for state in states]
    water = [state.stocks[WATER].amount for state in states]
    # Carbon migrates out of the larder and into the standing culture + feed tank, then
    # the loop holds: a large but bounded decline that flattens, not a slide to zero.
    assert min(food) > 700.0, f"food store dipped to {min(food)}"
    assert min(food) < 0.5 * food[0]
    assert min(water) > 50.0, f"water store dipped to {min(water)}"
    # The last tenth of the year moves the larder by well under 1 % — it has flattened.
    tail = food[len(food) * 9 // 10]
    assert abs(food[-1] - tail) / tail < 0.01


def test_the_o2_regulator_never_points_backwards(run: tuple[list[State], int]) -> None:
    """``cabin.o2`` stays strictly below the setpoint — the direction gate, as a bound.

    ``eclss.o2_makeup`` is the registry's only demand-controlled type: above the
    setpoint the frozen law goes negative and the flow reverses, which neither
    conservation nor rationing can see (``run_scenario`` raises ``ReversedFlowError``,
    and that is the only catch there has ever been). It does not fire here **by
    construction**, not by luck: the equilibrium sits below the setpoint by
    ``f*q/k_makeup`` — strictly positive because the crew always breathes — the cabin
    starts below it, and the per-sub-step map is a contraction with factor ``1 - k*h =
    0.4 > 0``, so it approaches from below without overshoot. Photosynthesis fills the
    *tank*; this regulator is the only path from tank to cabin.
    """
    states, _ = run
    o2 = [state.stocks[CABIN_O2].amount for state in states]
    assert max(o2) < O2_SETPOINT
    # ...and it is a live regulator, not an idle one: it lifts the cabin 8.0 -> ~9.76.
    assert o2[0] == pytest.approx(8.0)
    assert o2[-1] == pytest.approx(O2_SETPOINT - 0.23725, rel=1e-9)
    assert all(b >= a for a, b in zip(o2[:-1], o2[1:], strict=True)), "not monotone"


# --- the scenario earns its keep: every loop is LIVE, and they all work -----


def test_every_loop_is_live_not_static(run: tuple[list[State], int]) -> None:
    """All four books visibly run. A habitat that closes because nothing moves proves
    nothing, so the run starts off the fixed point in every one of them at once.
    """
    states, _ = run
    first, last = states[0], states[-1]

    # The culture establishes ~5x while the feed tank fills ~4.5x behind the scrubber...
    assert last.stocks[BIOMASS].amount > 4.5 * first.stocks[BIOMASS].amount
    assert last.stocks[FEED].amount > 4.0 * first.stocks[FEED].amount
    # ...paid for out of the O2 buffer, which is the same carbon seen from the
    # other book.
    assert last.stocks[O2_TANK].amount < 0.4 * first.stocks[O2_TANK].amount
    # The waste compartments charge from empty and hold.
    assert first.stocks[FECES].amount == 0.0
    assert last.stocks[FECES].amount > 10.0
    # The water loop charges from empty into its three transit pools.
    assert first.stocks[CONDENSATE].amount == 0.0
    assert last.stocks[CONDENSATE].amount > 1.5
    assert last.stocks[URINE].amount > 4.0
    assert last.stocks[WATER].amount < first.stocks[WATER].amount
    # The thermal node warms from far below equilibrium to a real station temperature.
    assert temperature(
        first.stocks[NODE].amount, heat_capacity=1.0e7, space_temperature=2.7
    ) == pytest.approx(102.7, rel=1e-6)
    assert temperature(
        last.stocks[NODE].amount, heat_capacity=1.0e7, space_temperature=2.7
    ) == pytest.approx(301.83, rel=1e-4)


def test_the_monod_factor_traverses_its_curve(run: tuple[list[State], int]) -> None:
    """The saturating factor actually moves, 0.250 -> 0.600.

    The ``monod_dsl.yaml`` lesson, applied to content: an earlier draft of that anchor
    drained 1.6e-9 of its battery and the cross-port suite passed anyway — **a dead
    anchor is trivially bit-exact**, and a saturating rate pinned at one value for 8760
    steps exercises the op without exercising the curve. Here the feed tank starts at a
    third of its half-saturation and settles at one and a half times it.
    """
    states, _ = run
    half_saturation = 300.0  # the bare inline literal — see the next test

    def factor(state: State) -> float:
        s = state.stocks[FEED].amount
        return s / (s + half_saturation)

    assert factor(states[0]) == pytest.approx(0.250, rel=1e-9)
    assert factor(states[-1]) == pytest.approx(0.600, rel=1e-3)


def test_converges_to_the_analytic_fixed_point(run: tuple[list[State], int]) -> None:
    """The sealed year lands on the fixed point the design solves for, to ~0.5 %.

    **Internal consistency, not validation** — it checks the authored graph behaves as
    its own arithmetic predicts (the ODE steady state plus a stable Jacobian: trace
    -1.13e-6 < 0, det 2.67e-13 > 0, discriminant +2.18e-13 > 0, so REAL eigenvalues and
    no ringing; slowest tau ~34.7 d, and a year is ~10.5 tau). It makes no claim that
    these kinetics describe a real bioregenerative station.

    **The tolerance is 1 % and that is not slack to hide in.** The residual here is not
    the transient (the fastest-settling pools are 63-315 tau in by the end and still sit
    off the continuous value) — it is the multi-rate **export offset**, which the next
    test derives in closed form and pins to 14 significant figures on the three pools
    where it is largest. This test is the coarse "did the whole graph land where the
    algebra said"; that one is the sharp instrument.
    """
    states, _ = run
    for stock_id, predicted in FIXED_POINT.items():
        assert states[-1].stocks[stock_id].amount == pytest.approx(predicted, rel=1e-2)


def test_the_multirate_export_offset_is_exact(run: tuple[list[State], int]) -> None:
    """**The analytic fixed point is not what a multi-rate run exports** — and by how
    much is closed-form.

    For a pool fed only by FAST flows and drained by ONE slow first-order flow, Strang
    puts the fast block *between* the slow set's two half-steps, so the inflow takes
    only one half-step of decay before the export point while the standing amount takes
    two::

        X_exported = X_continuous * 2a/(1 + a),      a = 1 - k*(dt/2)

    That is a property of the splitting, not an inaccuracy to be tightened away: the
    three pools of that shape sit 0.18 %, 0.18 % and 0.91 % below their continuous
    steady states, and the formula predicts all three to 14 significant figures. Pinned
    here because a tolerance-only check would let a genuinely mis-lowered partition hide
    inside the slack (``eclss_multirate_cabin.yaml``'s point: a mis-driven partition
    does not drift, it lands somewhere else).
    """
    states, _ = run
    slow_half_step = 3600.0 / 2.0
    cases = (
        (FECES, 2.0e-6, 12.75),
        (URINE, 2.0e-6, 4.875),
        (CONDENSATE, 1.0e-5, 2.025),
    )
    for stock_id, k, continuous in cases:
        a = 1.0 - k * slow_half_step
        exported = continuous * 2.0 * a / (1.0 + a)
        assert states[-1].stocks[stock_id].amount == pytest.approx(exported, rel=1e-11)
        # ...and the offset is real, i.e. this test is not vacuous.
        assert exported < continuous


def test_the_battery_is_sized_not_relaxed(run: tuple[list[State], int]) -> None:
    """The energy budget balances by construction, and holds for the whole year.

    ``eta_c*P_solar - P_load - k_sd*B = 0.95*4000 - 3790 - 1.0e-8*1.0e+9 = 0``. This is
    a **sizing** claim, not an attractor: the battery's restoring time constant is
    ``1/k_sd = 3.2 yr``, far longer than the run, so within one year it is an
    accumulator. A mis-sized load would show up here as a monotone drift, not as an
    equilibrium somewhere else.
    """
    states, _ = run
    charge = [state.stocks[BATTERY].amount for state in states]
    assert max(abs(c - 1.0e9) for c in charge) / 1.0e9 < 1e-9


# --- the file is what it says it is: composed, not re-invented -------------


def test_it_actually_composes_the_frozen_registry() -> None:
    """7 of the 17 flows are frozen ``type`` selections, spanning three sibling domains.

    This is the whole point of the file — the first authored *content* to compose the
    grown registry rather than invent every law — so it is asserted structurally.
    Rewriting any of these as authored ``kinetics`` would keep every conservation test
    above green while quietly undoing what the scenario exists to demonstrate.
    """
    spec = _spec()
    frozen = {f.id: f.type for f in spec.flows if f.type is not None}
    authored = [f.id for f in spec.flows if f.kinetics is not None]
    assert frozen == {
        "eclss.co2_scrubber": "eclss.co2_scrubber",
        "eclss.condenser": "eclss.condenser",
        "eclss.o2_makeup": "eclss.o2_makeup",
        "power.solar_charge": "power.solar_charge",
        "power.load_draw": "power.load_draw",
        "power.self_discharge": "power.self_discharge",
        "thermal.radiator_reject": "thermal.radiator_reject",
    }
    assert len(authored) == 10
    assert len(spec.flows) == 17


def test_the_frozen_equipment_recycles_instead_of_discarding() -> None:
    """No matter-bearing stock in this station is a boundary — the structural claim.

    Three frozen wiring fields are *named* for boundaries (``co2_removed``,
    ``humidity_condensate``, ``o2_supply``) and all three point at interior pools. The
    interpreter permits it because a wiring field is validated by NAME only
    (``interpreter._build_flow``); there is no ``kind`` constraint anywhere on the path.
    That single fact is what turns calibrated life-support equipment into recycling
    machinery, and it is why carbon, oxygen and water can close at all.
    """
    spec = _spec()
    boundary = {s.id: s.quantity for s in spec.stocks if s.kind == "boundary"}
    assert boundary == {
        "boundary.solar_source": "energy",
        "boundary.space": "energy",
    }, "a matter book acquired a boundary leg — closure is no longer strict"

    wiring = {f.id: f.wiring for f in spec.flows if f.type is not None}
    assert wiring["eclss.co2_scrubber"]["co2_removed"] == "bioreactor.co2_feed"
    assert wiring["eclss.condenser"]["humidity_condensate"] == "water.condensate"
    assert wiring["eclss.o2_makeup"]["o2_supply"] == "store.o2_tank"
    # The Power<->Thermal seam, authored: dissipation lands on the radiator node.
    for flow in ("power.solar_charge", "power.load_draw", "power.self_discharge"):
        assert wiring[flow]["waste_heat"] == "thermal.node"


def test_it_declares_a_coupling_cadence() -> None:
    """The partition is physical: the cabin-air set is fast, everything slower is slow.

    ``n_sub = 12`` is set by the tightest fast constraint, the frozen O2 regulator's
    ``k_makeup * h < 1`` (h = 300 s gives 0.60). The slow set steps at ``dt/2 = 1800
    s``, **not** ``dt/n_sub`` — the multi-rate Step-5 finding, and the reason every slow
    constant above is checked against 1800 rather than 300.
    """
    spec = _spec()
    assert spec.n_sub == 12
    assert spec.dt == 3600.0
    slow = {f.id for f in spec.flows if f.rate_class == "slow"}
    fast = {f.id for f in spec.flows if f.rate_class != "slow"}
    assert fast == {
        "crew.respiration",
        "crew.egestion",
        "crew.perspiration",
        "crew.urination",
        "eclss.co2_scrubber",
        "eclss.condenser",
        "eclss.o2_makeup",
    }
    assert len(slow) == 10

    # Seven stocks are genuinely shared ACROSS the rate-class boundary, so Strang's
    # operators do not commute here — unlike `eclss_multirate_cabin.yaml`, whose header
    # is candid that its own partition is "a fixture device, not a sizing claim" and
    # which had to manufacture a single such stock to have anything to prove. Computed
    # from the file rather than listed, so re-classing a flow cannot leave a stale
    # expectation behind.
    def touched(flow: object) -> set[str]:
        spec_flow = flow  # narrow once; a flow is either kinetics or a frozen type
        if getattr(spec_flow, "kinetics", None) is not None:
            return set(spec_flow.kinetics.stoichiometry)  # type: ignore[attr-defined]
        return set(spec_flow.wiring.values())  # type: ignore[attr-defined]

    fast_stocks: set[str] = set()
    slow_stocks: set[str] = set()
    for flow in spec.flows:
        (slow_stocks if flow.rate_class == "slow" else fast_stocks).update(
            touched(flow)
        )
    assert fast_stocks & slow_stocks == {
        "crew.food_store",
        "crew.water_store",
        "bioreactor.co2_feed",
        "store.o2_tank",
        "waste.feces",
        "waste.urine",
        "water.condensate",
    }


# --- the "authored != validated" marker -----------------------------------


def test_run_is_marked_uncalibrated() -> None:
    """The station carries ``has_authored_kinetics`` — the honest marker.

    Ten of the seventeen flow laws are authored, so the marker MUST be set: it is what
    makes Godot banner the run UNCALIBRATED and the graph dump mark it. Composing seven
    frozen types does **not** buy the file out of it, and would not even if all
    seventeen were frozen — "frozen" means the *form* is literature-derived, not that
    the values are validated (``flow_registry.py``: "Registered != calibrated").
    """
    assert load_scenario(STATION).has_authored_kinetics is True
