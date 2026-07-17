"""Multi-rate authoring, Step 1: the identity, measured through the authoring layer.

**Why this file exists before any schema change.** The plan's golden-preservation
argument is that multi-rate authoring need not move a single golden, because
``multirate_step`` has a bit-exact identity path: ``n_sub == 1`` + an **empty** slow
registry reproduces the single-rate ``step``. ``simcore`` already pins that
(``tests/test_multirate.py::test_all_fast_nsub1_reproduces_single_rate_bitwise``) — but
on a **synthetic** registry. Nothing proved it through the **authoring** layer, on a
real authored graph, against a real frozen golden. The advisor ruled that gap
*blocking*: if it is not byte-exact here, the knob design is moot and the phase stops.
It is byte-exact; this file is that proof, promoted from probe to pin.

**Step 3 demoted this from load-bearing to corroborating — which is a promotion for the
phase.** The title above used to end "the identity the whole phase rests on", and at
Step 1 that was exactly right: the goldens were to be preserved *because* the identity
held. Step 2's three-registry design changed the mechanism, and Step 3's harness made it
real: :func:`authoring.run.run_scenario` **branches** on ``BuiltScenario.is_multirate``
and drives a single-rate scenario down the pre-multi-rate loop verbatim, so the 25
goldens are preserved **by construction** — they never reach the driver at all
(``test_authoring_multirate_run.py::test_a_single_rate_scenario_never_touches_the_driver``
is that guarantee's pin). Nothing here weakened; what it carries got lighter. It is now
defence-in-depth on the driver's faithfulness, not the fact the phase stands or falls
on.

**Why this file still drives ``multirate_step`` by hand — and must.** The note here used
to instruct Step 3 to re-point the byte-identity pin through ``run_scenario``. That
instruction is **superseded**: it was written before the branch existed, and the branch
makes it impossible. ``is_multirate`` is *false* at ``n_sub=1`` with an empty slow set,
so a re-pointed test would exercise the **single-rate path** — not a weaker test of the
driver but a test of the wrong thing, and (sharing its golden oracle with
``test_authoring_frozen_flows.py``) a duplicate of that file, with the
driver-faithfulness check lost. The driver at ``n_sub=1`` is reachable **only** from
here. The through-harness coverage the note was reaching for does exist — at
``n_sub=60``, where the driver actually runs: ``test_authoring_multirate_run.py``.

**The identity carries an UNSTATED precondition: no aux processes.** ``step_report``
advances ``State.aux``; ``multirate_step`` deliberately never does (P2, *"Aux ×
multi-rate is out of scope"*). So the two agree only on an aux-free registry. That holds
today for every authored graph — ``interpret`` calls ``Registry(flows, stocks)`` and
never wires ``aux_processes`` — and it is pinned below, because it is the precondition
that silently becomes false the day the biosphere (the one aux-bearing domain) is made
authorable.

**Transcendental discipline** (the cross-libm trap): every assertion below is an exact
``float.hex()`` / bytes comparison of two runs of the **same** arithmetic, or of a run
against a committed golden. No closed form is asserted against.
"""

import copy
from pathlib import Path
from typing import Any

import pytest

import sim_io
from authoring.interpreter import BuiltScenario, interpret, load_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator
from simcore.multirate import Split, multirate_step
from simcore.registry import Registry
from simcore.state import State

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"


def _run_single_rate(built: BuiltScenario) -> tuple[list[State], int]:
    """The reference: today's single-rate path, ``step_report`` per step."""
    integrator = EulerIntegrator(built.registry)
    state = built.state
    states = [state]
    rationed = 0
    for _ in range(built.steps):
        report = integrator.step_report(state, built.resolver, built.dt)
        state = report.state
        states.append(state)
        rationed += report.rationed
    return states, rationed


def _run_multirate(
    built: BuiltScenario,
    n_sub: int,
    slow_ids: tuple[str, ...] = (),
    split: Split = Split.STRANG,
) -> tuple[list[State], int]:
    """Drive ``multirate_step`` over the authored graph, partitioned by flow id.

    Two disjoint registries over **one** stock dict (N3). ``slow_ids`` empty ⇒ the
    identity path. This is by hand what the Step-2 interpreter will do from the
    per-flow ``rate:`` key.
    """
    flows = list(built.registry.flows)
    slow = EulerIntegrator(
        Registry([f for f in flows if f.id in slow_ids], built.state.stocks)
    )
    fast = EulerIntegrator(
        Registry([f for f in flows if f.id not in slow_ids], built.state.stocks)
    )
    state = built.state
    states = [state]
    rationed = 0
    for _ in range(built.steps):
        report = multirate_step(
            slow, fast, state, built.resolver, built.dt, n_sub, split=split
        )
        state = report.state
        states.append(state)
        rationed += report.rationed
    return states, rationed


def _assert_bit_identical(a: list[State], b: list[State]) -> None:
    """Exact equality over **every stock of every step**, by ``float.hex()``.

    Comparing only the headline stock is how the first pass of this probe fooled itself:
    it read "bit-identical" off flows that never touch ``cabin_o2``. Compare everything.
    """
    assert len(a) == len(b)
    for i, (sa, sb) in enumerate(zip(a, b, strict=True)):
        assert sa.n == sb.n, f"step {i}: n {sa.n} != {sb.n}"
        assert set(sa.stocks) == set(sb.stocks), f"step {i}: stock sets differ"
        for sid in sorted(sa.stocks):
            xa, xb = sa.stocks[sid].amount, sb.stocks[sid].amount
            assert xa.hex() == xb.hex(), (
                f"step {i} {sid}: {xa.hex()} != {xb.hex()} (delta {xb - xa:+.3e})"
            )


# ---------------------------------------------------------------------------
# 1. THE IDENTITY — the blocking fact, on the real authored graph
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("split", [Split.STRANG, Split.LIE])
def test_nsub1_empty_slow_is_bit_identical_to_single_rate(split: Split) -> None:
    # The golden-preservation argument, measured through the authoring layer: 900 Euler
    # steps of the Tier-1 ECLSS anchor (4 flows, 3 conserved quantities, one six-leg
    # forced flow), every stock of every step compared by hex.
    #
    # BOTH splits hold, and that is not luck: with an empty slow set Strang's two
    # `slow(dt/2)` halves are no-ops and Lie's one `slow(dt)` is a no-op, so each
    # reduces to the single full fast sub-step — which IS the single-rate step.
    built = load_scenario(str(ECLSS_YAML))
    single, r_single = _run_single_rate(built)
    multi, r_multi = _run_multirate(load_scenario(str(ECLSS_YAML)), 1, (), split)
    assert r_single == r_multi == 0
    _assert_bit_identical(single, multi)


def test_nsub1_reproduces_the_frozen_eclss_golden_bytes() -> None:
    # The identity's strongest form and the one the plan actually needs: the multi-rate
    # driver at n_sub=1 reproduces the *frozen* golden BYTE-FOR-BYTE — not merely
    # "agrees with the single-rate run". Same oracle test_authoring_frozen_flows.py
    # uses; no new golden is minted, which is the point.
    built = load_scenario(str(ECLSS_YAML))
    states, rationed = _run_multirate(built, 1, ())
    assert rationed == 0
    produced = sim_io.dumps(states[-1]).encode("utf-8")
    assert produced == (GOLDEN_DIR / "eclss_state.json").read_bytes()


def test_the_identity_precondition_no_authored_graph_has_aux() -> None:
    # THE UNSTATED PRECONDITION. step_report advances aux; multirate_step never does
    # (P2). The identity above therefore holds only because the authoring layer cannot
    # express aux at all: `interpret` calls Registry(flows, stocks) and never passes
    # aux_processes. This pin is the tripwire — the day the biosphere (the one
    # aux-bearing domain, deferred from the registry for exactly this family of reasons)
    # becomes authorable, this turns red BEFORE aux silently freezes under multi-rate.
    built = load_scenario(str(ECLSS_YAML))
    assert built.registry.aux_processes == ()
    assert built.state.aux == {}


# ---------------------------------------------------------------------------
# 2. THE MIRROR — why `n_sub=1` with a slow set must be refused, not honoured
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "slow_id",
    [
        "eclss.co2_scrubber",  # donor-controlled
        "eclss.o2_makeup",  # demand-controlled
        "eclss.crew_metabolism",  # FORCED — the counterintuitive one
    ],
)
def test_nsub1_with_a_slow_flow_perturbs_the_trajectory(slow_id: str) -> None:
    # A partition at n_sub=1 buys NO rate separation and NO perf win — and it is not
    # inert. It perturbs via TWO mechanisms: the slow flow's own two-half-step
    # discretization ((1−k·dt/2)² ≠ (1−k·dt)), and — the dominant one — the COUPLING,
    # since fast flows now read slow-updated stocks mid-step (N5 prices this as the
    # splitting error).
    #
    # The forced row is why this is parametrized over all three shapes. Two intuitive
    # hypotheses are false, and this pins both dead:
    #   "only COUPLED flows perturb"   -> false: the scrubber perturbs its own stocks.
    #   "a FORCED flow splits exactly" -> false where it matters: crew_metabolism's own
    #                                     legs do split to roundoff, yet the cabin moves
    #                                     ~1e-01 because the fast flows see a
    #                                     half-metabolised cabin.
    # Hence the Step-2 validator rule: n_sub=1 + non-empty slow is an AuthoringError.
    reference, _ = _run_single_rate(load_scenario(str(ECLSS_YAML)))
    partitioned, _ = _run_multirate(load_scenario(str(ECLSS_YAML)), 1, (slow_id,))
    differing = [
        sid
        for sid in sorted(reference[-1].stocks)
        if reference[-1].stocks[sid].amount.hex()
        != partitioned[-1].stocks[sid].amount.hex()
    ]
    assert differing, (
        f"slow={slow_id!r} at n_sub=1 left every stock bit-identical — if this is now "
        f"true, the 'a partition at n_sub=1 silently moves the answer' rationale for "
        f"refusing it needs re-deriving, not deleting"
    )


# ---------------------------------------------------------------------------
# 3. THE PAYOFF — the phase's whole value proposition, at the dt that breaks the cabin
# ---------------------------------------------------------------------------

O2_EQ = 8.0  # o2_setpoint − Con_o2/k_makeup = 10 − 0.004/0.002 — the truth
UNSAFE_DT = 3600.0  # k_makeup·dt = 7.2, k_scrub·dt = 3.6 — the rates' breaking point
FROZEN_DT = 60.0  # the anchor's own sizing: k_makeup·dt = 0.12
HOURS = 24  # the horizon both sides of the payoff comparison cover
CABIN_O2 = StockId("eclss.cabin_o2")  # a NewType over str — a bare literal is not a key


def _at_dt(dt: float, steps: int) -> BuiltScenario:
    """The ECLSS anchor re-interpreted at an overridden ``dt``/``steps``.

    Mirrors ``test_authoring_dt_hazard._build_at`` — the anchor file itself is never
    edited; only its run config is overridden, so the graph under test stays the frozen
    one.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = dt
    raw["steps"] = steps
    return interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)


def _at_unsafe_dt(steps: int = HOURS) -> BuiltScenario:
    """The ECLSS anchor at the master dt that wrecks it single-rate."""
    return _at_dt(UNSAFE_DT, steps)


def test_single_rate_at_the_unsafe_dt_is_broken() -> None:
    # The baseline the payoff is measured against. At k·dt = 7.2 the controller diverges
    # and the backstop fires. (This lands at 72.0 where test_authoring_dt_hazard.py
    # records ~0 at the same dt — no contradiction: that file samples at 15 steps, this
    # at 24, and an oscillating divergence is simply at a different point.)
    _states, rationed = _run_single_rate(_at_unsafe_dt())
    assert rationed > 0


@pytest.mark.parametrize(
    ("n_sub", "safe"),
    [
        (2, False),  # effective k·dt = 3.60 — STILL unsafe
        (10, True),  # effective k·dt = 0.72
        (60, True),  # effective k·dt = 0.12 — the frozen sizing
    ],
)
def test_substepping_rescues_the_cabin_only_when_the_EFFECTIVE_step_is_safe(
    n_sub: int, safe: bool
) -> None:
    # THE PHASE IN ONE TEST, and the argument for the build-time precondition.
    #
    # n_sub=60 lands on the truth while the master step — what NEIGHBOURS see — stays at
    # one hour. That is the export-fidelity charge answered.
    #
    # n_sub=2 does NOT. Multi-rate splits dt into dt/n_sub, so an author who "switches
    # multi-rate on" and picks a too-small n_sub gets the IDENTICAL hazard one level
    # down: the knob reads as safety and is not. Multi-rate is the performance enabler,
    # not the hazard closer — hence `rate_params` + the k·(dt/n_sub) < 1 build check.
    states, rationed = _run_multirate(_at_unsafe_dt(), n_sub)
    final = states[-1].stocks[CABIN_O2].amount
    if safe:
        assert rationed == 0
        assert final == pytest.approx(O2_EQ, abs=1e-6)
    else:
        assert final != pytest.approx(O2_EQ, abs=1e-6)


def test_the_coarse_export_is_the_same_answer_as_the_fine_run() -> None:
    # The sharpest statement of the fix, and the one worth reading twice:
    #
    #   master dt=3600 + n_sub=60  ==  single-rate dt=60,
    #
    # over the same 24 h of simulated time — while EXPORTING 60x less often. Both sides
    # take 1440 sub-steps of 60 s; they differ only in how often a master step commits
    # (i.e. how often a neighbouring domain reads the stock). That is precisely the
    # composability the reference calls impossible ("there is no dt natural to both
    # domains"), and precisely the user's charge: the hourly export is now the value the
    # 60 s run actually produces.
    coarse, _ = _run_multirate(_at_dt(UNSAFE_DT, HOURS), 60)
    fine, _ = _run_single_rate(_at_dt(FROZEN_DT, HOURS * 60))
    # NOT bit-identical step-for-step: the two have different step COUNTS by
    # construction (24 master commits vs 1440), and Strang's no-op slow halves still
    # re-enter the arithmetic. The claim is that the ENDPOINTS agree — which is exactly
    # what a neighbouring domain reads.
    a = coarse[-1].stocks[CABIN_O2].amount
    b = fine[-1].stocks[CABIN_O2].amount
    assert a == pytest.approx(b, abs=1e-12), (
        f"coarse-export endpoint {a!r} != fine-run endpoint {b!r}"
    )
