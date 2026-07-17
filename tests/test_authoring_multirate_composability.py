"""Multi-rate authoring, Step 4: the composability anchor — the impossible scenario.

``docs/authoring-reference.md`` ("The dt constraint") says of composing ECLSS with
Thermal: *"A scenario composing both must pick one dt, and only dt <= ~60 is safe for
both. **There is no dt natural to both domains.**"* ``eclss_thermal_habitat.yaml`` is
that scenario and this file measures that the sentence is false — once an author picks a
**coupling cadence** rather than a global ``dt``, both domains run at the ``dt`` their
own rate constants demand, out of one file at one export cadence.

**The constraint has two halves and both must be shown**, because either alone is
unconvincing: the shared ``dt`` is *unsafe* (single-rate ``dt=3600`` rations), and the
safe shared ``dt`` is *wasteful* (single-rate ``dt=60`` makes Thermal pay 20160 steps to
resolve a ``tau`` of ~65 steps). Multi-rate escapes both at once.

**What this anchor is NOT.** The two domains share no stock — no ECLSS flow type carries
a heat leg, so there is no ECLSS->Thermal flow to write. That is **forced by the
registry, not chosen**: coupling lives within a domain (the shared cabin) or across
same-timescale domains (Power<->Thermal, both slow), so the cross-rate-class boundary
and the cross-stock boundary never overlap. The fast and slow operators here act on
disjoint quantities, so Strang's non-commutativity term is exactly zero and **no
coupling fidelity is exercised**. That is not a gap: it is pinned twice already, by
``test_authoring_multirate_partition.py`` (a slow flow sharing stocks with fast flows
DOES perturb them: +1.2e-01 on the cabin) and by simcore's ``test_multirate.py``.

**What IS new**: the first **non-empty slow set** ever driven through ``run_scenario``.
Every Step-3 multi-rate run declared ``n_sub`` with an *empty* slow set, so
``_run_multirate``'s slow sub-integrator had never held a flow from any authored file.
"""

import copy
import dataclasses
import itertools
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any

import pytest

from authoring.errors import AuthoringError, RationedError
from authoring.interpreter import BuiltScenario, interpret
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from domains.thermal.flows import RadiatorReject
from simcore.ids import StockId
from simcore.state import State

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
HABITAT_YAML = SCENARIO_DIR / "eclss_thermal_habitat.yaml"

CABIN_O2 = StockId("eclss.cabin_o2")
NODE = StockId("thermal.node")

O2_EQ = 8.0
"""o2_setpoint - Con_o2/k_makeup = 10 - 0.004/0.002 — the dt-independent truth."""

MASTER_DT = 3600.0
"""The coupling cadence. Single-rate this is unsafe for ECLSS (k_scrub*dt = 3.6 > 1)."""

SAFE_SHARED_DT = 60.0
"""The only dt safe for BOTH domains single-rate — the reference's own '<= ~60'."""

MASTER_STEPS = 336
"""14 days ~ 5.2 tau of Thermal's relaxation (tau ~ 65 steps at dt=3600)."""

SHARED_DT_STEPS = 20160
"""The same 14 days at the safe shared dt (336 * 60) — what Thermal used to pay."""

T_SPACE = 2.7
HEAT_CAPACITY = 1.0e7
T_EQ = 280.9
"""Emergent, not a param: eps*sigma*A*(T_eq^4 - T_space^4) = heat_load = 3000 W."""


def _temperature(node_joules: float) -> float:
    """The derived readout T = T_space + Q/C (thermal/flows.py's own formula)."""
    return T_SPACE + node_joules / HEAT_CAPACITY


def _build(*, single_rate: bool = False, **overrides: Any) -> BuiltScenario:
    """The habitat anchor re-interpreted at an overridden run config.

    The anchor **file** is never edited (the ``test_authoring_dt_hazard._build_at``
    discipline): only its in-memory dict, so the graph under test stays the committed
    one and only its cadence moves.

    ``single_rate=True`` drops the partition as well as ``n_sub`` — see
    ``test_going_single_rate_means_dropping_the_partition_not_just_n_sub`` for why
    that is not merely a convenience.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(HABITAT_YAML)))
    if single_rate:
        raw["n_sub"] = 1
        for flow in raw["flows"]:
            flow.pop("rate_class", None)
    raw.update(overrides)
    return interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)


@contextmanager
def _counting_radiator_evals() -> Iterator[list[int]]:
    """Count ``RadiatorReject.evaluate`` calls — the Thermal work actually done.

    Patched on the **class**, not the instance: the frozen flow dataclasses forbid
    ``setattr``. Counting evaluations rather than timing is deliberate — the ratio below
    is an exact integer fact about the driver, where a wall-clock assertion would be a
    flake.
    """
    calls = [0]
    original = RadiatorReject.evaluate

    def counting(self: RadiatorReject, *args: Any, **kwargs: Any) -> Any:
        calls[0] += 1
        return original(self, *args, **kwargs)

    RadiatorReject.evaluate = counting  # type: ignore[method-assign]
    try:
        yield calls
    finally:
        RadiatorReject.evaluate = original  # type: ignore[method-assign]


def _amounts(states: list[State], stock: StockId) -> list[float]:
    return [s.stocks[stock].amount for s in states]


def _run(built: BuiltScenario) -> tuple[list[State], int]:
    states, rationed, _events = run_scenario(built)
    return states, rationed


# The two expensive runs are module-scoped: 336 master steps * 60 sub-steps and 20160
# single-rate steps. Read-only trajectories, so sharing them across tests is safe.


@pytest.fixture(scope="module")
def multirate() -> tuple[list[State], int, int]:
    """The anchor as committed: master dt=3600, n_sub=60, Thermal slow."""
    with _counting_radiator_evals() as calls:
        states, rationed = _run(_build())
    return states, rationed, calls[0]


@pytest.fixture(scope="module")
def single_rate_at_safe_dt() -> tuple[list[State], int, int]:
    """The same graph at the only dt safe for both domains — the wasteful escape."""
    with _counting_radiator_evals() as calls:
        states, rationed = _run(
            _build(single_rate=True, dt=SAFE_SHARED_DT, steps=SHARED_DT_STEPS)
        )
    return states, rationed, calls[0]


# ---------------------------------------------------------------------------
# 1. THE PARTITION — the shape the reference calls impossible, as built
# ---------------------------------------------------------------------------


def test_the_habitat_lowers_to_a_two_cadence_partition() -> None:
    # The file declares one dt and two cadences. Assert the partition is what the header
    # claims BEFORE trusting any trajectory off it: a scenario that silently lowered
    # Thermal to `fast` would still run clean and would prove nothing about multi-rate.
    built = _build()
    assert built.is_multirate is True
    assert built.n_sub == 60
    assert built.dt == MASTER_DT
    assert sorted(str(f.id) for f in built.slow_registry.flows) == [
        "thermal.heat_input",
        "thermal.radiator_reject",
    ]
    assert sorted(str(f.id) for f in built.fast_registry.flows) == [
        "eclss.co2_scrubber",
        "eclss.condenser",
        "eclss.crew_metabolism",
        "eclss.o2_makeup",
    ]


def test_the_two_rate_classes_share_no_stock() -> None:
    # THE HONESTY PIN, and it is an assertion rather than a comment because the claim it
    # bounds is what this file may NOT conclude. Disjoint stocks => the Strang operators
    # commute exactly => zero splitting error => this anchor exercises the CADENCE
    # decoupling and NOT coupling fidelity.
    #
    # It is disjoint by FORCE, not by choice: no ECLSS flow type carries a heat leg, so
    # no ECLSS->Thermal flow can be written. If a future registry addition couples them,
    # this test goes red — which is the correct moment to re-read the paragraph above,
    # because the anchor would then be proving strictly more than it claims.
    built = _build()

    def wired_stocks(registry: Any) -> set[str]:
        # Read the wiring off the frozen dataclasses rather than off evaluated legs:
        # `built.resolver` is a SourceResolver (a binding source), not the bound
        # Environment `evaluate` wants, and the wiring is static anyway. Per
        # FlowTypeSpec, every frozen constructor is (id, priority, *wiring[, params]) —
        # the discipline test_authoring_frozen_flows.py pins — so the wiring is exactly
        # the remaining fields. `params` is a dataclass, not a StockId.
        return {
            str(getattr(flow, field.name))
            for flow in registry.flows
            for field in dataclasses.fields(flow)
            if field.name not in {"id", "priority", "params"}
        }

    slow_stocks = wired_stocks(built.slow_registry)
    fast_stocks = wired_stocks(built.fast_registry)
    assert slow_stocks == {"boundary.heat_source", "thermal.node", "boundary.space"}
    assert len(fast_stocks) == 9
    assert slow_stocks & fast_stocks == set()


# ---------------------------------------------------------------------------
# 2. THE ANCHOR RUNS — conservation, determinism, rationed == 0
# ---------------------------------------------------------------------------


def test_the_composed_scenario_runs_clean_at_the_master_cadence(
    multirate: tuple[list[State], int, int],
) -> None:
    # The headline. Conservation is proved by COMPLETION, not by an assertion here: the
    # every-step per-quantity gate runs inside multirate_step, so a mis-balanced leg
    # would have raised ConservationError rather than reached this line (four
    # independent books — oxygen, carbon, water, energy).
    #
    # `rationed == 0` is MEASURED and not reasoned, and the slow set is why: Thermal's
    # T^4 law has no k*dt < 1 guarantee to lean on, and Strang steps it at dt/2 = 1800
    # s.
    states, rationed, _evals = multirate
    assert rationed == 0
    # One commit per MASTER step — sub-steps are internal and never commit. This is what
    # makes the export hourly, which is the whole charge.
    assert len(states) == MASTER_STEPS + 1
    assert states[-1].n == MASTER_STEPS


def test_the_anchor_is_deterministic() -> None:
    # Bit-identical within a build, over EVERY stock of EVERY step — not just the two
    # the other tests read. float.hex() rather than ==, per the golden discipline.
    first, _ = _run(_build())
    second, _ = _run(_build())
    assert [{str(k): v.amount.hex() for k, v in s.stocks.items()} for s in first] == [
        {str(k): v.amount.hex() for k, v in s.stocks.items()} for s in second
    ]


# ---------------------------------------------------------------------------
# 3. BOTH DOMAINS LAND — each at its own cadence, out of one file
# ---------------------------------------------------------------------------


def test_the_cabin_holds_at_o2_eq_while_the_file_exports_hourly(
    multirate: tuple[list[State], int, int],
) -> None:
    # The fast side. o2 tau = 1/k_makeup = 500 s, so the cabin reaches o2_eq inside the
    # FIRST master hour (3600 s = 7.2 tau) and holds. That is export fidelity stated as
    # a trajectory: a neighbour sampling this file hourly sees 8.0 — against the 0.0 mol
    # a single-rate dt=1800 run exports (test_authoring_export_fidelity.py).
    states, _rationed, _evals = multirate
    o2 = _amounts(states, CABIN_O2)
    assert o2[0] == 10.0  # starts AT the setpoint; the regulator idles
    assert o2[1] == pytest.approx(
        O2_EQ, abs=1e-2
    )  # already there after one master step
    assert o2[-1] == pytest.approx(O2_EQ, abs=1e-9)
    # Monotone non-increasing: the crew draw pulls the cabin down to o2_eq and the
    # regulator never overshoots it. Measured as exact (no tolerance) — the approach is
    # from above and the fixed point is approached, never crossed.
    assert all(later <= earlier for earlier, later in itertools.pairwise(o2))


def test_the_thermal_node_warms_to_equilibrium_on_the_slow_set(
    multirate: tuple[list[State], int, int],
) -> None:
    # The slow side, and the sizing note it rests on: a run that "equilibrates" because
    # it STARTED at equilibrium proves nothing (thermal_node.yaml's own warning). So pin
    # the distance travelled, not just the endpoint.
    states, _rationed, _evals = multirate
    node = _amounts(states, NODE)
    assert _temperature(node[0]) == pytest.approx(102.7, abs=0.1)  # far below T_eq
    assert _temperature(node[-1]) == pytest.approx(277.4, abs=0.1)  # ~5.2 tau in
    assert _temperature(node[-1]) < T_EQ  # approached from below, never overshot
    # Monotone increasing: 3000 W in against ~54 W radiated at 102.7 K, closing on the
    # T^4 balance at T_eq. An overshoot would show up here before it showed up in
    # `rationed`, since the backstop only sees an over-draw large enough to empty the
    # node.
    assert all(later >= earlier for earlier, later in itertools.pairwise(node))


# ---------------------------------------------------------------------------
# 4. THE CONSTRAINT'S TWO HALVES — what multi-rate escapes
# ---------------------------------------------------------------------------


def test_the_shared_dt_is_unsafe_the_first_half_of_the_constraint() -> None:
    # Half one: the same graph, the same master dt, minus the partition. k_scrub*3600 =
    # 3.6 > 1, the scrubber over-draws, and the run is a RationedError. This is what
    # "only dt <= ~60 is safe for both" means from the ECLSS side.
    with pytest.raises(RationedError):
        run_scenario(_build(single_rate=True))


def test_the_unsafe_shared_dt_wrecks_the_cabin_not_merely_rations() -> None:
    # What the refusal above is PROTECTING against — the number, not the exception. 72.0
    # mol against a truth of 8.0: the demand-controlled regulator has DIVERGED, not
    # drifted.
    #
    # Read the direction carefully, because the intuitive word for it is the wrong one:
    # 72.0 is NINE TIMES TOO MUCH oxygen, not too little. The dt hazard is not
    # "the cabin suffocates" — it is "the number is meaningless", and its sign is an
    # accident of where the oscillation is sampled. k_makeup*dt = 7.2 makes the update
    # map o2 -> -6.2*o2 + 57.6, whose |ratio| > 1: it alternates and grows. The
    # reference's separately-measured "-1.4e-14" endpoint (test_authoring_dt_hazard, 15
    # steps) and this 72.0 (336 steps) are the SAME broken map at different phases. An
    # author who reads "asphyxiation" learns the wrong lesson and will accept a run that
    # happens to land high.
    #
    # Cross-check worth keeping: 840 firings over 336 steps is 2.5/step, exactly the
    # rate the plan's Step-3 table measured on the bare ECLSS anchor (60 firings / 24
    # steps). Adding the Thermal half changed neither the firing rate nor the endpoint,
    # which is itself evidence the two rate classes do not interact here.
    states, rationed, _events = run_scenario(
        _build(single_rate=True), allow_rationing=True
    )
    assert rationed == 840
    assert states[-1].stocks[CABIN_O2].amount == pytest.approx(72.0, abs=1e-6)
    assert states[-1].stocks[CABIN_O2].amount > O2_EQ  # ABOVE the truth, not below


def test_the_safe_shared_dt_is_wasteful_the_second_half(
    single_rate_at_safe_dt: tuple[list[State], int, int],
) -> None:
    # Half two, and the half that is easy to forget: dt=60 is not "the answer that
    # already worked". It IS safe — rationed == 0, the cabin lands — and it makes
    # Thermal resolve a tau of ~65 steps in 20160 of them. The reference calls dt <= ~60
    # the escape; this is its price.
    states, rationed, evals = single_rate_at_safe_dt
    assert rationed == 0
    assert states[-1].stocks[CABIN_O2].amount == pytest.approx(O2_EQ, abs=1e-9)
    assert evals == SHARED_DT_STEPS


# ---------------------------------------------------------------------------
# 5. THE PAYOFF — Thermal stops paying ECLSS's dt
# ---------------------------------------------------------------------------


def test_thermal_stops_paying_eclss_dt_and_the_factor_is_30_not_60(
    multirate: tuple[list[State], int, int],
    single_rate_at_safe_dt: tuple[list[State], int, int],
) -> None:
    # THE MEASURED PAYOFF, and the number is NOT the one the plan predicted. The cadence
    # ratio is 3600/60 = 60, and the plan says "Thermal is forced to pay 60x the steps
    # it needs" — but the realized saving is 30x, because **Strang steps the slow set at
    # dt/2, twice per master step**. 336 * 2 = 672 against 20160.
    #
    # The missing factor of two is not an error, it is Strang's documented slow-set cost
    # (authoring/run.py, `_SPLIT`): Lie would evaluate the slow set ONCE per master step
    # and realize the full 60x, at a lower nominal order and a coarser slow-set step.
    # The split was pinned to Strang on order/safety grounds, and this is the bill for
    # it.
    _s, _r, multirate_evals = multirate
    _s60, _r60, single_rate_evals = single_rate_at_safe_dt
    assert multirate_evals == 2 * MASTER_STEPS == 672  # two half-steps per master step
    assert single_rate_evals == SHARED_DT_STEPS == 20160
    assert single_rate_evals / multirate_evals == 30.0
    # Named explicitly so the reconciliation cannot be lost: the cadence ratio is 60 and
    # the realized saving is half of it, by construction rather than by accident.
    assert SAFE_SHARED_DT * multirate_evals / MASTER_STEPS == 2.0 * SAFE_SHARED_DT


def test_the_cheap_thermal_run_agrees_with_the_expensive_one(
    multirate: tuple[list[State], int, int],
    single_rate_at_safe_dt: tuple[list[State], int, int],
) -> None:
    # What the 30x BUYS is only worth having if the cheap run is still right — otherwise
    # it is the dt hazard wearing a new hat. Measured: the cabin agrees to the last bit,
    # and the node to 0.014 %.
    #
    # Those two are different claims and the difference is the point. The cabin is
    # BIT-IDENTICAL because the fast set is integrated at 60 s either way and the
    # operators are disjoint, so multi-rate reproduces the ECLSS trajectory exactly
    # rather than approximately. The node DIFFERS because it is genuinely stepped
    # coarser (1800 s vs 60 s) — that residual is Euler's discretization error on T^4
    # across a 30x step, and it is 0.04 K. That smallness is not luck; it is `tau >> dt`
    # doing its job, and it is exactly why Thermal never needed the fine step.
    states, _r, _e = multirate
    states60, _r60, _e60 = single_rate_at_safe_dt

    assert states[-1].stocks[CABIN_O2].amount.hex() == (
        states60[-1].stocks[CABIN_O2].amount.hex()
    )

    node_multirate = states[-1].stocks[NODE].amount
    node_single = states60[-1].stocks[NODE].amount
    assert node_multirate == pytest.approx(node_single, rel=2e-4)
    assert abs(_temperature(node_multirate) - _temperature(node_single)) < 0.1


# ---------------------------------------------------------------------------
# 6. GOING BACK — the refusal that shapes what "single-rate" even means here
# ---------------------------------------------------------------------------


def test_going_single_rate_means_dropping_the_partition_not_just_n_sub() -> None:
    # Found while building this anchor, and worth a pin: you cannot make this file
    # single-rate by setting n_sub=1. Step 2 REFUSES n_sub=1 with a non-empty slow set —
    # it buys no rate separation, no perf win, and still moves the answer (the slow set
    # splits into two dt/2 halves and the fast flows read slow-updated stocks mid-step).
    #
    # So the `single_rate=True` helper above must strip `rate_class` too, and every
    # "same graph single-rate" contrast in this file rests on that. The refusal's own
    # message says so ("drop the 'rate_class: slow' key(s) to run single-rate"), which
    # is the message doing its job.
    with pytest.raises(AuthoringError, match="n_sub=1 with a non-empty slow set"):
        _build(n_sub=1)


def test_the_partition_is_inert_without_a_cadence_only_if_dropped() -> None:
    # The complement of the refusal: stripping rate_class gives a legal single-rate
    # scenario over the WHOLE registry — six flows, one cadence — which is what the two
    # contrasts above actually run. Pinned so those contrasts cannot silently degrade
    # into running some other graph than the anchor's.
    built = _build(single_rate=True)
    assert built.is_multirate is False
    assert len(built.registry.flows) == 6
    assert not built.slow_registry.flows
