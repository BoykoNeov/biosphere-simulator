"""Multi-rate authoring, Step 6: the Python side of the multi-rate cross-port anchor.

``eclss_multirate_cabin.yaml`` is the first file in
``tests/crossport/authoring_files.py`` :data:`ANCHORS` to declare a coupling cadence.
The cross-port gates themselves live in ``tests/crossport/test_crossport.py``
(graph-dump parity + run parity, both of which bite for it, because the file is
Tier-1). **This module owns what those gates cannot say**: that the anchor is worth
anchoring at all.

**Why that is a real question and not ceremony.** A cross-port equality gate compares
two ports to each other, so it is fully satisfied by two ports agreeing on *nothing
interesting*. Two failure modes this module exists to exclude:

* **A dead anchor is trivially bit-exact** — the ``monod_dsl.yaml`` lesson (an earlier
  draft of that file was inert and the whole crossport suite passed anyway). So the
  pools must be shown to *move*, to their known analytic steady states.
* **An inert partition is trivially bit-exact too** — the failure mode one level up,
  and the one specific to this anchor. If the slow flow shared no stock with the fast
  set, the Strang operators would commute, the partition would not change the
  trajectory, and the run-parity gate would pass whether or not the two ports lowered
  the same partition. That is exactly the shape of ``eclss_thermal_habitat.yaml``
  (disjoint quantities ⇒ splitting error exactly zero) and precisely why *it* is not
  the anchor.

So the load-bearing test here is
:func:`test_the_partition_is_load_bearing_not_decorative`: drop the anchor's single
``rate_class: slow`` key — change nothing else, not ``dt``, not ``n_sub``, not a param
— and the trajectory moves ~29 %. That is the measurement that converts "both ports
render the same rate class" into "both ports *drive* the same partition".

**What this anchor does and does not uniquely guard (measured, Step 6).** Two crude
port mutations — Rust lowering an all-fast partition, and Rust's driver splitting Lie
where the constant says Strang — are each caught by Rust's *own* Step-6 pins as well as
by the new gates, so this anchor is not the sole guard against them. What it uniquely
adds is that the two ports are compared **to each other at all**: with the partition
destroyed on the Rust side, the entire *pre-anchor* crossport suite stayed green (33
passed), because nothing in it was sensitive to a partition. A divergence that is
self-consistent on each port — a default, a vocabulary, an interaction with
``includes`` — is caught here or nowhere.

**A red in the crossport gates is a PORT finding, not a reason to retune this file.**
"""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any

import pytest

from authoring.flow_registry import FLOW_TYPES
from authoring.interpreter import BuiltScenario, _effective_step, interpret
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from domains.eclss.loader import load_eclss_params
from simcore.ids import StockId
from simcore.state import State

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ANCHOR_YAML = SCENARIO_DIR / "eclss_multirate_cabin.yaml"

CABIN_O2 = StockId("eclss.cabin_o2")
CABIN_CO2 = StockId("eclss.cabin_co2")
CABIN_H2O = StockId("eclss.cabin_h2o")

CONDENSER = "eclss.condenser"

MASTER_DT = 1800.0
"""The coupling cadence — what a neighbour would see, not what ECLSS solves at."""

N_SUB = 30
"""Fast h = dt/n_sub = 60 s: exactly eclss_cabin.yaml's load-bearing dt."""

FAST_STEP = 60.0
SLOW_STEP = 900.0
"""Strang steps the slow set at dt/2, **regardless of n_sub** — multi-rate Step 5's
finding, and the point on which this plan's own original formula was wrong."""

# The dt-independent analytic truths (domains/eclss/system.py's steady_state).
O2_EQ = 8.0
"""o2_setpoint - Con_o2/k_makeup = 10 - 0.004/0.002."""
CO2_EQ = 3.0
"""P_co2/k_scrub = 0.003/1.0e-3."""
H2O_EQ_CONTINUOUS = 0.04
"""P_h2o/k_cond = 2.0e-5/5.0e-4 — what a well-resolved condenser converges to."""

H2O_SLOW_SET = 0.02838709677419354
"""MEASURED, not derived: cabin_h2o's steady state with the condenser in the SLOW set.

Strang runs the slow operator twice per master step at h = dt/2 = 900 s, so the discrete
condenser removes (1 - k·h)² = 0.3025 of the pool per master step where a well-resolved
one would remove e^(-k·dt) = 0.4066 — it over-draws, and the pool settles ~29 % below
:data:`H2O_EQ_CONTINUOUS`. **That is bad physics and it is the whole point** (see the
anchor's own header): the gap is what makes a mis-lowered partition visible.
"""


def _build(*, condenser_fast: bool = False) -> BuiltScenario:
    """The anchor as committed, or with its one partition key dropped.

    The anchor **file** is never edited (the ``_build_at`` /
    ``test_authoring_multirate_composability`` discipline): only its in-memory dict, so
    the graph under test stays the committed one.

    ``condenser_fast=True`` drops ``rate_class: slow`` and **nothing else** — same
    ``dt``, same ``n_sub``, same params, still multi-rate. That one-key isolation is
    what makes the contrast a measurement of the partition rather than of the cadence.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ANCHOR_YAML)))
    if condenser_fast:
        for flow in raw["flows"]:
            if flow["id"] == CONDENSER:
                flow.pop("rate_class")
    return interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)


@pytest.fixture(scope="module")
def anchor_run() -> tuple[list[State], int]:
    """The anchor exactly as the crossport gates run it. Read-only ⇒ module-scoped."""
    states, rationed, events = run_scenario(_build())
    assert events == (), events
    return states, rationed


# ---------------------------------------------------------------------------
# 1. The anchor lowers to the partition the crossport gates think they compare
# ---------------------------------------------------------------------------


def test_the_anchor_lowers_to_a_one_flow_slow_set() -> None:
    built = _build()
    assert built.is_multirate
    assert built.n_sub == N_SUB
    assert built.dt == MASTER_DT
    assert [str(flow.id) for flow in built.slow_registry.flows] == [CONDENSER]
    assert [str(flow.id) for flow in built.fast_registry.flows] == [
        "eclss.co2_scrubber",
        "eclss.crew_metabolism",
        "eclss.o2_makeup",
    ]


def test_both_rate_classes_clear_the_build_time_precondition() -> None:
    """The anchor must pass the Step-5 ``k·h < 1`` check on the *author's* path — with
    no ``allow_unsafe_step`` hatch — on both classes at once. ``_build()`` not raising
    is that proof; the numbers are spelled out here because a cross-port disagreement
    about *them* would surface as one port refusing to build a file the other runs, and
    that is a failure mode the parity gates report only as a crash.

    ``_effective_step``'s own three-case semantics are pinned by
    ``test_authoring_rate_precondition.py``; what is asserted here is this **anchor's
    sizing** against them — and against the **frozen** ``eclss.yaml`` rather than
    transcribed literals, so a param edit that pushed a class over the bound surfaces
    here as a margin failure rather than as a mystery build error in the crossport run.
    """
    assert _effective_step(MASTER_DT, N_SUB, multirate=True, slow=False) == FAST_STEP
    assert _effective_step(MASTER_DT, N_SUB, multirate=True, slow=True) == SLOW_STEP

    p = load_eclss_params()
    fast_margins = {
        "co2_scrubber": p.co2_scrub_rate * FAST_STEP,
        "o2_makeup": p.o2_makeup_gain * FAST_STEP,
    }
    slow_margin = p.condense_rate * SLOW_STEP
    assert fast_margins == pytest.approx({"co2_scrubber": 0.06, "o2_makeup": 0.12})
    assert slow_margin == pytest.approx(0.45)
    assert all(m < 1.0 for m in (*fast_margins.values(), slow_margin))


# ---------------------------------------------------------------------------
# 2. The anchor is not dead — a dead anchor is trivially bit-exact
# ---------------------------------------------------------------------------


def test_the_anchor_runs_clean(anchor_run: tuple[list[State], int]) -> None:
    """``rationed == 0`` and no events — what the Rust ``emit_authored`` example asserts
    before it prints, so a regression here breaks the crossport run gate as a *crash*
    rather than a diff."""
    _states, rationed = anchor_run
    assert rationed == 0


def test_every_pool_moves_and_lands_on_its_analytic_steady_state(
    anchor_run: tuple[list[State], int],
) -> None:
    """48 master steps = 1 day ≈ 43 τ of the slowest loop. Each pool must be *away* from
    where it started as well as *at* the truth — an anchor that equilibrates because it
    started equilibrated proves nothing (thermal_node.yaml's own warning)."""
    states, _rationed = anchor_run
    initial, final = states[0], states[-1]

    for stock, expected in (
        (CABIN_O2, O2_EQ),
        (CABIN_CO2, CO2_EQ),
        (CABIN_H2O, H2O_SLOW_SET),
    ):
        assert initial.stocks[stock].amount != pytest.approx(expected, rel=1e-3), (
            f"{stock} starts at its steady state — this pool proves nothing"
        )
        assert final.stocks[stock].amount == pytest.approx(expected, rel=1e-9), stock

    # The two well-resolved loops (k·h ≤ 0.12) land on the dt-independent truth; only
    # the slow-set one is displaced — the anchor's story in one assertion.
    assert final.stocks[CABIN_H2O].amount != pytest.approx(H2O_EQ_CONTINUOUS, rel=1e-3)


# ---------------------------------------------------------------------------
# 3. THE TEETH — the partition changes the trajectory, so parity means something
# ---------------------------------------------------------------------------


def test_cabin_h2o_is_the_one_stock_shared_across_the_rate_class_boundary() -> None:
    """The anchor's teeth rest on this, so it is an assertion, not a comment.

    If the slow flow shared no stock with the fast set, the Strang operators would
    commute exactly, the partition would not move the trajectory, and the crossport run
    gate would pass whether or not the two ports agreed on it (that is
    ``eclss_thermal_habitat.yaml``'s situation — disjoint quantities). A future edit
    that breaks the sharing must go red here rather than silently gut the anchor.
    """
    built = _build()
    fast_stocks = {sid for flow in built.fast_registry.flows for sid in _wired(flow)}
    slow_stocks = {sid for flow in built.slow_registry.flows for sid in _wired(flow)}
    shared = fast_stocks & slow_stocks
    assert shared == {CABIN_H2O}, (
        f"the anchor's teeth are the fast/slow shared stock; got {sorted(shared)}"
    )


def test_the_partition_is_load_bearing_not_decorative() -> None:
    """**The measurement this module exists for.** Drop the anchor's single
    ``rate_class: slow`` key — same dt, same n_sub, same params, still multi-rate — and
    cabin_h2o moves from ~2.84e-02 to 4.00e-02, ~29 %.

    So a port that mis-classified this flow, threaded ``n_sub`` differently, or split
    the slow set at ``dt`` instead of ``dt/2`` does not drift by an ULP; it lands
    somewhere else entirely and ``test_rust_authoring_run_matches_python`` goes red.
    Without this,
    "the two ports render the same rate class" would be the *only* thing the crossport
    suite knew about the partition — a claim about a string, not about a driver.
    """
    slow_states, slow_rationed, _ = run_scenario(_build())
    fast_states, fast_rationed, _ = run_scenario(_build(condenser_fast=True))
    assert slow_rationed == 0 and fast_rationed == 0

    slow_h2o = slow_states[-1].stocks[CABIN_H2O].amount
    fast_h2o = fast_states[-1].stocks[CABIN_H2O].amount

    assert slow_h2o == pytest.approx(H2O_SLOW_SET, rel=1e-9)
    assert fast_h2o == pytest.approx(H2O_EQ_CONTINUOUS, rel=1e-9)
    assert abs(slow_h2o - fast_h2o) / fast_h2o == pytest.approx(0.29, abs=0.01)

    # The control: the one key moves the H2O loop and NOTHING else. If dropping it also
    # moved O2/CO2, the contrast above would be measuring some side effect instead.
    for stock in (CABIN_O2, CABIN_CO2):
        assert (
            slow_states[-1].stocks[stock].amount == fast_states[-1].stocks[stock].amount
        ), stock


def test_dropping_the_key_empties_the_slow_set_rather_than_reordering_it() -> None:
    """The contrast's premise: ``condenser_fast=True`` produces a graph that is still
    multi-rate (n_sub = 30) with an **empty** slow set — i.e. uniform sub-stepping at
    h = 60. So the 29 % above is the partition's doing and not a cadence change."""
    built = _build(condenser_fast=True)
    assert built.is_multirate, "n_sub=30 alone still routes through the driver"
    assert list(built.slow_registry.flows) == []
    assert len(built.fast_registry.flows) == 4


def _wired(flow: object) -> set[StockId]:
    """The stock ids a frozen flow is wired to, read off its dataclass fields.

    :data:`~authoring.flow_registry.FLOW_TYPES` already names exactly the constructor
    fields that take a ``StockId`` (``wiring_fields``) — the registry's own definition
    of what a flow touches, so this asks the source of truth rather than inferring from
    an evaluation. **Python-only, deliberately**: on the Rust side a ``Box<dyn Flow>``
    exposes no such accessor, which is why the cross-port graph dump carries no wiring
    at all, and why the sharing this test pins is trajectory-covered there instead.
    """
    spec = FLOW_TYPES[_type_of(flow)]
    return {StockId(getattr(flow, field)) for field in spec.wiring_fields}


def _type_of(flow: object) -> str:
    """The registry type name whose ``cls`` built this flow (lowering, reversed)."""
    matches = [name for name, spec in FLOW_TYPES.items() if spec.cls is type(flow)]
    assert len(matches) == 1, (type(flow), matches)
    return matches[0]
