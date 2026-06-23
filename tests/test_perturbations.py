"""Phase-3 Step-6 (P3.5): the perturbation harness + representative perturbations.

The harness (``domains/biosphere/perturbations.py``) composes a perturbation **onto**
the already-assembled ``(state, registry, resolver)`` — a scenario-layer intervention,
not a core change (``git diff src/simcore/`` empty; all three existing goldens stay
byte-identical; **no new golden** — a perturbation is a behavioural demonstration, the
Step-5 "diagnostics, no golden" precedent). Three representative perturbations, each a
**cascade, no cascade code** with **conservation + ``rationed == 0``** held through it:

1. **drought** (open field — the only scenario with irrigation to cut): cut irrigation
   over a window → ``soil_water`` drains across the stress band → ``f_water < 1`` →
   assimilation falls → biomass below baseline. Baseline ``f_water ≡ 1`` (no spurious
   baseline stress), so the dip is wholly the perturbation's.
2. **lighting_failure** (perennial chamber): PAR → 0 over a within-year window →
   gross assimilation → 0 → growth stalls, the chamber carbon pool stops drawing down so
   **chamber CO₂ rises**.
3. **atmospheric_leak** (perennial chamber): a windowed first-order ``carbon_pool →
   leak_sink`` leak → the pool drains, ``Ci`` collapses, and the chamber is **no
   longer closed** (the leak-sink accounts the vented mass — conservation holds with
   it explicit; closure does not).

**Scenario assignment is asymmetric — by design.** Drought can only live on the open
field (sealed dropped irrigation in Step 3 for genuine water closure); lighting + leak
target the closed chamber, where the per-compartment-ledger-under-perturbation story
matters. Lighting + leak run a **single season** (``run_season``, 1 year): the annual
reset first fires at ``n = 305``, so a 305-step run equals ``run_perennial``
and never resets — the cascade is fully visible in year 1 and reset-under-ledger is
already covered by Step 5's ``test_compartment_ledger``. The recoverable-regime boundary
(a *severe* perturbation that starves the seed bank → the reset raises) is pinned
separately by the characterization test, the only ``run_perennial`` use here.

**Cascade asserts are direction-only** (the Step-4/5 anti-flakiness rule — never a
magnitude or a day index; per-stock, never ``State == State``), each comparing a
**perturbed** run to a **baseline** run. **Leg reconstruction binds the PERTURBED
resolver** (the one carrying the zeroed schedule / ``LEAK_VAR``) — binding the baseline
would disagree on exactly the perturbed steps (the design's flagged bug). Determinism
re-runs stand in for the absent golden.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.chamber import ci_from_co2_pool
from domains.biosphere.compartments import (
    ATMOSPHERE,
    PLANTS,
    compartment_boundary_ledger,
    expected_extinction_residuals,
)
from domains.biosphere.perturbations import (
    LEAK_SINK,
    with_atmospheric_leak,
    with_drought,
    with_lighting_failure,
)
from domains.biosphere.scenario import (
    DROUGHT_SCENARIO,
    PERENNIAL_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_YEARS,
)
from domains.biosphere.season import (
    CARBON_POOL,
    LEAF_C,
    ROOT_C,
    SOIL_WATER,
    STEM_C,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.transpiration import water_stress_factor
from simcore.boundary import BOUNDARY_DOMAIN, loss_sink
from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


_YEAR = len(_weather())  # season length in steps (305)

# Perturbation windows (integer steps — #14 forcing seam). Lighting + leak target the
# sealed perennial chamber, so their window stays WITHIN year 1 (the recoverable regime:
# grain refills before the n=305 reset). Drought is open-field (no reset to starve), so
# its window is unconstrained — widened so the irrigation cut pushes soil_water clearly
# across the stress band for an unambiguous f_water dip (a thin graze of the threshold
# would be fragile).
_START = 30
_END = 80  # lighting + leak (within year 1, recoverable)
_DROUGHT_END = 130  # drought (open field, deeper drawdown)
_K_LEAK = 0.05  # first-order leak rate (k·dt = 0.05 < 1 ⇒ rationed == 0 structural)

# The per-quantity ledger / conservation tol table Step 5 pins (a flat 1e-7 is far too
# loose for the O(1) CARBON amounts — it would hide a real misapplication).
TOL = {
    Quantity.CARBON: 1e-12,
    Quantity.OXYGEN: 1e-11,
    Quantity.WATER: 1e-7,
    Quantity.NITROGEN: 1e-9,
}


# --- shared helpers ----------------------------------------------------------
def _biomass(state: State) -> float:
    """Vegetative carbon (leaf + stem + root) — the cascade's biomass signal."""
    return (
        state.stocks[LEAF_C].amount
        + state.stocks[STEM_C].amount
        + state.stocks[ROOT_C].amount
    )


def _total(state: State, quantity: Quantity) -> float:
    """Folded total of ``quantity`` over ALL stocks (boundaries/leak-sink incl.)."""
    return sum(
        st.amount * st.composition.get(quantity, 0.0) for st in state.stocks.values()
    )


def _assert_conserved(states: list[State]) -> None:
    """Every quantity conserved on every stored step, within the per-quantity tol.

    Sums over **all** stocks (leak-sink and loss-sinks included), so a leak that vents
    mass to its boundary sink still conserves the *total* — only the chamber interior's
    closure breaks, not conservation.
    """
    for quantity, abs_tol in TOL.items():
        q0 = _total(states[0], quantity)
        for s in states:
            assert math.isclose(
                _total(s, quantity), q0, rel_tol=0.0, abs_tol=abs_tol
            ), (quantity, _total(s, quantity), q0)


def _reconstruct_legs(
    registry: Registry, resolver: SourceResolver, before: State
) -> list[FlowResult]:
    """One ``evaluate`` per flow at ``before`` — the applied legs (Euler, rationed=0).

    Binds the resolver to the **same** snapshot it evaluates against (#16, mirroring the
    engine). ``resolver`` MUST be the perturbed one (carrying the zeroed schedule /
    ``LEAK_VAR``), or the reconstructed legs disagree with the applied legs on the
    perturbed steps (the design's flagged bug).
    """
    bound = resolver.bind(before, 1.0)
    return [flow.evaluate(before, bound, 1.0) for flow in registry.flows]


def _assert_ledger_balances_every_step(
    states: list[State],
    registry: Registry,
    resolver: SourceResolver,
    *,
    track: tuple[DomainId, Quantity],
) -> None:
    """Ledger balances every step; assert the ``track`` flux is non-vacuous.

    The run is extinction-free (``events == ()``, asserted by the caller), so the
    correction term :func:`expected_extinction_residuals` returns is ``{}`` and the raw
    residual must be ≈ 0 within tol for every ``(domain, quantity)`` — boundary included
    (no whitelist). ``track`` is a ``(domain, quantity)`` whose flux must be seen
    at least once (non-vacuity: the residual check exercises real crossing flux).
    """
    track_domain, track_quantity = track
    saw_track = False
    for i in range(len(states) - 1):
        before = states[i]
        legs = _reconstruct_legs(registry, resolver, before)
        ledger = compartment_boundary_ledger(before, states[i + 1], legs)
        for entry in ledger:
            assert abs(entry.residual) <= TOL[entry.quantity], (i, entry)
            if (
                entry.domain == track_domain
                and entry.quantity is track_quantity
                and (entry.crossing_in > 0.0 or entry.crossing_out > 0.0)
            ):
                saw_track = True
    assert saw_track, (track, "tracked crossing flux never observed — vacuous")


# --- drought (open field): cut irrigation → f_water < 1 → biomass falls ---------------
@pytest.fixture(scope="module")
def drought() -> tuple[list[State], list[State]]:
    """Baseline vs irrigation-cut open-field runs (the water-lean DROUGHT_SCENARIO)."""
    weather = _weather()
    state, registry = build_season(DROUGHT_SCENARIO)
    base_resolver = weather_resolver(weather, DROUGHT_SCENARIO)
    base_states, base_rationed, base_events = run_season(
        EulerIntegrator(registry), state, base_resolver, 1.0, _YEAR
    )
    drought_resolver = with_drought(base_resolver, start=_START, end=_DROUGHT_END)
    drought_states, drought_rationed, drought_events = run_season(
        EulerIntegrator(registry), state, drought_resolver, 1.0, _YEAR
    )
    assert base_rationed == 0 and drought_rationed == 0
    assert base_events == () and drought_events == ()
    return base_states, drought_states


def _fwater(state: State) -> float:
    return water_stress_factor(
        state.stocks[SOIL_WATER].amount,
        sw_wilting=DROUGHT_SCENARIO.sw_wilting,
        sw_critical=DROUGHT_SCENARIO.sw_critical,
    )


def test_drought_baseline_unstressed_then_cut_stresses(
    drought: tuple[list[State], list[State]],
) -> None:
    # The clean contrast: baseline f_water ≡ 1 over the WHOLE season (the scenario is
    # sized to stay at/above sw_critical with irrigation on — no spurious baseline
    # stress), yet cutting irrigation drives f_water strictly below 1 within the window
    # (soil_water drains across the band). Direction-only — never the 0.5 magnitude.
    base_states, drought_states = drought
    assert all(_fwater(s) == 1.0 for s in base_states)
    assert min(_fwater(drought_states[i]) for i in range(_START, _DROUGHT_END)) < 1.0


def test_drought_suppresses_biomass(
    drought: tuple[list[State], list[State]],
) -> None:
    # The cascade's payoff: end-of-season vegetative biomass is strictly below baseline
    # (f_water < 1 cuts limitation = f_water·f_N into gross assimilation). Per-stock
    # comparison, never State == State.
    base_states, drought_states = drought
    assert _biomass(drought_states[-1]) < _biomass(base_states[-1])


def test_drought_conserves_and_is_deterministic(
    drought: tuple[list[State], list[State]],
) -> None:
    # Conservation holds through the perturbation (drought changes a forcing value —
    # the ledger structure is untouched), and the perturbed run is bit-identical on a
    # re-run (the no-golden insurance). Drought gets NO ledger assertion (design: only
    # lighting + leak — drought adds no cross-compartment flux to audit).
    _, drought_states = drought
    _assert_conserved(drought_states)
    state, registry = build_season(DROUGHT_SCENARIO)
    resolver = with_drought(
        weather_resolver(_weather(), DROUGHT_SCENARIO), start=_START, end=_DROUGHT_END
    )
    rerun, _, _ = run_season(EulerIntegrator(registry), state, resolver, 1.0, _YEAR)
    assert rerun[-1] == drought_states[-1]


# --- lighting_failure (perennial chamber): PAR → 0 → growth stalls, CO₂ rises ---
@pytest.fixture(scope="module")
def lighting() -> tuple[list[State], list[State], Registry, SourceResolver]:
    """Baseline vs PAR-blackout single-season runs + the perturbed registry/resolver."""
    weather = _weather()
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    base_resolver = weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO)
    base_states, base_rationed, base_events = run_season(
        EulerIntegrator(registry), state, base_resolver, 1.0, _YEAR
    )
    lf_resolver = with_lighting_failure(base_resolver, start=_START, end=_END)
    lf_states, lf_rationed, lf_events = run_season(
        EulerIntegrator(registry), state, lf_resolver, 1.0, _YEAR
    )
    assert base_rationed == 0 and lf_rationed == 0
    assert (
        base_events == () and lf_events == ()
    )  # no extinction ⇒ ledger needs no corr.
    return base_states, lf_states, registry, lf_resolver


def test_lighting_failure_stalls_growth_and_raises_co2(
    lighting: tuple[list[State], list[State], Registry, SourceResolver],
) -> None:
    # The two-pronged signature within the window: biomass strictly BELOW baseline
    # (growth halted) AND the carbon pool strictly ABOVE baseline (photosynthesis
    # stopped drawing it down, so CO₂ rises) — the emergent cascade, no cascade code.
    base_states, lf_states, _, _ = lighting
    assert _biomass(lf_states[_END]) < _biomass(base_states[_END])
    assert lf_states[_END].stocks[CARBON_POOL].amount > (
        base_states[_END].stocks[CARBON_POOL].amount
    )


def test_lighting_failure_conserves_and_ledger_balances(
    lighting: tuple[list[State], list[State], Registry, SourceResolver],
) -> None:
    # Four-quantity conservation every step + the per-compartment ledger balances every
    # step under the perturbation (legs reconstructed against the PERTURBED resolver —
    # the zeroed PAR schedule). CARBON crossing flux is non-vacuous (photosynthesis /
    # respiration still cross the plants↔atmosphere boundary outside the window).
    _, lf_states, registry, lf_resolver = lighting
    _assert_conserved(lf_states)
    _assert_ledger_balances_every_step(
        lf_states, registry, lf_resolver, track=(ATMOSPHERE, Quantity.CARBON)
    )


def test_lighting_failure_is_deterministic(
    lighting: tuple[list[State], list[State], Registry, SourceResolver],
) -> None:
    _, lf_states, _, _ = lighting
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    resolver = with_lighting_failure(
        weather_resolver(_weather(), PERENNIAL_CHAMBER_SCENARIO), start=_START, end=_END
    )
    rerun, _, _ = run_season(EulerIntegrator(registry), state, resolver, 1.0, _YEAR)
    assert rerun[-1] == lf_states[-1]


# --- atmospheric_leak (perennial chamber): pool → leak_sink → Ci collapses ---
@pytest.fixture(scope="module")
def leak() -> tuple[list[State], list[State], Registry, SourceResolver]:
    """Baseline vs windowed-leak single-season runs + the leaked registry/resolver."""
    weather = _weather()
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    base_resolver = weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO)
    base_states, base_rationed, base_events = run_season(
        EulerIntegrator(registry), state, base_resolver, 1.0, _YEAR
    )
    leak_state, leak_registry, leak_resolver = with_atmospheric_leak(
        state,
        registry,
        base_resolver,
        pool=CARBON_POOL,
        k_leak=_K_LEAK,
        start=_START,
        end=_END,
    )
    leak_states, leak_rationed, leak_events = run_season(
        EulerIntegrator(leak_registry), leak_state, leak_resolver, 1.0, _YEAR
    )
    assert base_rationed == 0 and leak_rationed == 0
    assert (
        base_events == () and leak_events == ()
    )  # no extinction ⇒ ledger needs no corr
    return base_states, leak_states, leak_registry, leak_resolver


def _ci(state: State) -> float:
    return ci_from_co2_pool(
        state.stocks[CARBON_POOL].amount,
        air_mol=PERENNIAL_CHAMBER_SCENARIO.chamber_air_mol,
        ci_ratio=PERENNIAL_CHAMBER_SCENARIO.ci_ratio,
    )


def test_atmospheric_leak_opens_chamber_and_collapses_ci(
    leak: tuple[list[State], list[State], Registry, SourceResolver],
) -> None:
    # The chamber opens: leak-sink strictly accumulates across the window (mass leaves
    # to the boundary — the chamber is no longer closed), and within the window both the
    # carbon pool and the derived Ci are strictly below baseline (the draw-down → weaker
    # FvCB cascade). Direction-only, per-stock.
    base_states, leak_states, _, _ = leak
    assert leak_states[_END].stocks[LEAK_SINK].amount > (
        leak_states[_START].stocks[LEAK_SINK].amount
    )
    assert leak_states[_END].stocks[CARBON_POOL].amount < (
        base_states[_END].stocks[CARBON_POOL].amount
    )
    assert _ci(leak_states[_END]) < _ci(base_states[_END])


def test_atmospheric_leak_conserves_with_sink_explicit_and_ledger_balances(
    leak: tuple[list[State], list[State], Registry, SourceResolver],
) -> None:
    # Conservation STILL holds with the leak-sink explicit (do NOT assert closure /
    # loss_sink == 0 — the leak legitimately moves mass to the boundary; _total sums all
    # stocks incl. the leak-sink, so the TOTAL is conserved). The per-compartment ledger
    # balances every step against the PERTURBED resolver, and the ATMOSPHERE→boundary
    # flux is reported (the tracked, non-vacuous crossing — the debuggability payoff).
    _, leak_states, leak_registry, leak_resolver = leak
    _assert_conserved(leak_states)
    _assert_ledger_balances_every_step(
        leak_states,
        leak_registry,
        leak_resolver,
        track=(BOUNDARY_DOMAIN, Quantity.CARBON),
    )


def test_atmospheric_leak_is_deterministic(
    leak: tuple[list[State], list[State], Registry, SourceResolver],
) -> None:
    _, leak_states, _, _ = leak
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    leak_state, leak_registry, leak_resolver = with_atmospheric_leak(
        state,
        registry,
        weather_resolver(_weather(), PERENNIAL_CHAMBER_SCENARIO),
        pool=CARBON_POOL,
        k_leak=_K_LEAK,
        start=_START,
        end=_END,
    )
    rerun, _, _ = run_season(
        EulerIntegrator(leak_registry), leak_state, leak_resolver, 1.0, _YEAR
    )
    assert rerun[-1] == leak_states[-1]


# --- the deferred multi-extinction live-order agreement (a hand-built test) ---
# Discharges the `ea901d4` forward-pointer: the helper-fold accumulation test
# pins the HELPER's fold, deferring "the live float-order agreement is to be verified
# [in Step 6] against the real ledger". No shipped perturbation drives multi-extinction
# (organs asymptote toward 0 without crossing threshold), so this is a hand-built
# deterministic two-extinction step through the REAL integrator (the Step-5 single-
# extinction idiom, extended to two with DISTINCT residuals so the accumulation order is
# genuinely exercised).
_POP_A = StockId("test.pop_a")
_POP_B = StockId("test.pop_b")
_PLANT_POOL = StockId("test.plant_c")
_ATMOS_POOL = StockId("test.atmos_c")


def _carbon(
    sid: StockId,
    domain: DomainId,
    amount: float,
    *,
    kind: StockKind,
    threshold: float = 0.0,
) -> Stock:
    return Stock(
        id=sid,
        domain=domain,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=kind,
        extinction_threshold=threshold,
    )


class _CrossingFlow:
    """A fixed, balanced CARBON transfer ``src -> dst`` across two compartments."""

    def __init__(self, fid: FlowId, src: StockId, dst: StockId, amount: float) -> None:
        self._id, self.src, self.dst, self.amount = fid, src, dst, amount

    @property
    def id(self) -> FlowId:
        return self._id

    @property
    def priority(self) -> int:
        return 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        x = self.amount * dt
        return FlowResult(legs=(Leg(self.src, -x), Leg(self.dst, x)))


def test_handbuilt_two_extinction_live_order_agreement() -> None:
    # Two below-threshold POPULATION organs in the SAME (compartment, quantity) extinct
    # in ONE real integrator step, with DISTINCT residuals (0.002 ≠ 0.003) — a genuine
    # multi-extinction fold (two distinct values, not one doubled). The integrator
    # orders events by sorted stock-id and buckets the loss-sink in that same order; the
    # helper folds the integrator's OWN report.events (never a hand-built list), so the
    # LIVE ledger and the helper must agree: the corrected residual is ~0 everywhere
    # (+0.005 raw on PLANTS, −0.005 on boundary). (With two values IEEE addition
    # commutes, so this is the live-agreement check the ea901d4 pointer earmarked, not
    # order-sensitivity per se.)
    ls = loss_sink(Quantity.CARBON)
    pop_a = _carbon(_POP_A, PLANTS, 0.002, kind=StockKind.POPULATION, threshold=0.01)
    pop_b = _carbon(_POP_B, PLANTS, 0.003, kind=StockKind.POPULATION, threshold=0.01)
    plant_pool = _carbon(_PLANT_POOL, PLANTS, 10.0, kind=StockKind.POOL)
    atmos_pool = _carbon(_ATMOS_POOL, ATMOSPHERE, 0.0, kind=StockKind.POOL)
    stocks = {s.id: s for s in (pop_a, pop_b, plant_pool, atmos_pool, ls)}
    before = State(n=0, stocks=stocks, rng_seed=0)
    flow = _CrossingFlow(FlowId("test.cross"), _PLANT_POOL, _ATMOS_POOL, 1.0)
    registry = Registry([flow], stocks)

    report = EulerIntegrator(registry).step_report(before, SourceResolver(), 1.0)
    after = report.state

    # Both organs snapped this step; the leg-reconstruction precondition holds.
    assert report.rationed == 0
    assert len(report.events) == 2
    assert sorted(e.residual for e in report.events) == [0.002, 0.003]

    bound = SourceResolver().bind(before, 1.0)
    results = [f.evaluate(before, bound, 1.0) for f in registry.flows]
    ledger = compartment_boundary_ledger(before, after, results)
    by = {(e.domain, e.quantity): e for e in ledger}
    # Raw: +Σr on the shared organ compartment, −Σr on boundary; crossing target clean.
    assert by[(PLANTS, Quantity.CARBON)].residual == pytest.approx(0.005)
    assert by[(BOUNDARY_DOMAIN, Quantity.CARBON)].residual == pytest.approx(-0.005)
    assert abs(by[(ATMOSPHERE, Quantity.CARBON)].residual) < 1e-12

    # The helper folds the integrator's report.events; the correction zeroes each entry.
    expected = expected_extinction_residuals(before, report.events)
    for e in ledger:
        corrected = e.residual - expected.get((e.domain, e.quantity), 0.0)
        assert abs(corrected) < 1e-12, (e, expected)


# --- the recoverable-regime boundary (characterization, NOT a shipped perturbation) ---
def test_severe_perturbation_trips_the_reset_seed_bank_guard() -> None:
    # The regime boundary (a probe finding): a SEVERE/permanent perturbation suppresses
    # photosynthesis enough that the grain never refills, so at the year-1 reset
    # annual_reset raises (the P3.4 closure caveat: the seedling must come from the
    # in-system seed bank). A full-year PAR blackout over the FIRST year, run across TWO
    # years (so the n=305 reset actually fires) → ValueError. This locks the
    # closure-caveat × perturbation regime as a deliberate boundary, not a cascade.
    weather = _weather() * PERENNIAL_CHAMBER_YEARS
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    severe = with_lighting_failure(
        weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO), start=10, end=_YEAR
    )
    with pytest.raises(ValueError, match="seed bank too small"):
        run_perennial(
            EulerIntegrator(registry),
            state,
            PERENNIAL_CHAMBER_SCENARIO,
            severe,
            1.0,
            2 * _YEAR,
            year=_YEAR,
        )
