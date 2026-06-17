"""Phase-1 Step-2 tests: the non-conserved auxiliary channel (P2).

Exercises the load-bearing architecture step — ``State.aux``, the ``AuxProcess``
protocol (``simcore.auxiliary``), the ``Registry`` aux-process collection, and the
integrator's aux advance — against the locked P2 contract:

* **Euler accumulation** — a constant-rate aux process integrates to ``rate·n·dt``.
* **RK4-carry** — aux advances **once** per step under RK4 (not once per stage), and a
  flow reading aux sees the **step-entry** value at every one of the 4 RK4 stages
  (aux is kept across stages like ``n`` — only stock amounts perturb).
* **Conservation exclusion** — an aux-only change conserves trivially (aux is outside
  the ledger by construction); a deliberately unbalanced *stock* change still trips
  the gate; a model that advances aux *and* moves mass conserves every step.
* **Multi-rate placement guard** — ``substep`` leaves aux **untouched** while
  ``step_report`` advances it once (the pinned placement; aux × multi-rate is out of
  scope in Phase 1, single-rate).
* **Determinism / registration-order independence** — the cross-process per-name sum
  is canonical (``AuxId``-sorted, #15): shuffling the aux-process registration list
  yields a bit-identical accumulator even for an associativity-sensitive sum.
* **State.aux primitive** — empty immutable default, finiteness validation,
  read-only mapping, detaches from the caller dict.
* **Registry** — duplicate-``AuxId`` rejection and canonical id-sorted order.

Test-local aux processes / flows mirror the ``_DecayFlow`` / ``_Cascade`` precedent
(frozen dataclasses; the recording flow keeps a mutable ``list`` whose *reference* is
frozen but contents append — a deliberate test instrument).
"""

import dataclasses
import math
from collections.abc import Sequence

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore import boundary, conservation
from simcore.auxiliary import AuxId, AuxProcess
from simcore.environment import Environment, SourceResolver, constant
from simcore.flow import ConservationError, FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

BIO = DomainId("bio")


# --- test-local aux processes + flows --------------------------------------
@dataclasses.dataclass(frozen=True)
class _ConstRateAux:
    """Constant-rate accumulator: increment == ``rate·dt`` (increment form)."""

    id: AuxId
    name: str
    rate: float

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> dict[str, float]:
        return {self.name: self.rate * dt}


@dataclasses.dataclass(frozen=True)
class _ForcedAux:
    """Thermal-time-like: increment == ``env.get(var)·dt`` (reads forcing via #16)."""

    id: AuxId
    name: str
    var: str

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> dict[str, float]:
        return {self.name: env.get(self.var) * dt}


@dataclasses.dataclass(frozen=True)
class _AuxRecordingFlow:
    """No-op *balanced* flow (empty legs) that records the aux value it sees.

    Used to prove aux is constant across RK4 stages: ``evaluate`` is called once per
    stage, and ``seen`` collects the aux read each time. The ``list`` is a mutable
    test instrument — the dataclass is frozen (its *reference* cannot be reassigned),
    its contents append.
    """

    id: FlowId
    priority: int
    name: str
    seen: list[float]

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        self.seen.append(snapshot.aux.get(self.name, 0.0))
        return FlowResult(legs=())


@dataclasses.dataclass(frozen=True)
class _Decay:
    """``src -> boundary sink`` first-order decay (dt-linear, balanced carbon)."""

    id: FlowId
    priority: int
    src: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.sink, moved)))


def _pool(sid: str, amount: float) -> Stock:
    return Stock(
        id=StockId(sid),
        domain=BIO,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


# --- Euler / RK4 accumulation ----------------------------------------------
@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_constant_rate_accumulates_to_rate_times_n_dt(integrator_cls: type) -> None:
    # A constant-rate aux process integrates by explicit Euler to rate·n·dt under
    # BOTH schemes — aux advances exactly once per step, never sub-staged through RK4.
    rate, dt, steps = 2.5, 0.5, 7
    reg = Registry(
        flows=[], stocks={}, aux_processes=[_ConstRateAux(AuxId("tt"), "tt", rate)]
    )
    integ = integrator_cls(reg)
    state = State(n=0, stocks={}, rng_seed=0)
    for _ in range(steps):
        state = integ.step(state, SourceResolver(), dt)
    assert state.n == steps
    assert state.aux["tt"] == rate * dt * steps


def test_forced_aux_reads_env() -> None:
    # An aux process resolves forcing through the bound env (#16 seam): thermal time
    # accumulates the forced "temperature" each step.
    temp, dt, steps = 18.0, 1.0, 4
    reg = Registry(
        flows=[],
        stocks={},
        aux_processes=[_ForcedAux(AuxId("tt"), "thermal_time", "temp")],
    )
    integ = EulerIntegrator(reg)
    resolver = SourceResolver(forcings={"temp": constant(temp)})
    state = State(n=0, stocks={}, rng_seed=0)
    for _ in range(steps):
        state = integ.step(state, resolver, dt)
    assert state.aux["thermal_time"] == temp * dt * steps


# --- RK4-carry: a flow reading aux sees a within-step constant -------------
def test_rk4_stages_all_see_step_entry_aux() -> None:
    # Under RK4 the recording flow is evaluated 4× (one per stage); every read equals
    # the step-entry aux value, because RK4 stage states keep aux (only amounts
    # perturb). aux only advances at the n->n+1 commit, after the stages.
    name = "tt"
    rate, dt, v0 = 3.0, 0.5, 10.0
    seen: list[float] = []
    flow = _AuxRecordingFlow(FlowId("rec"), 0, name, seen)
    reg = Registry(
        flows=[flow], stocks={}, aux_processes=[_ConstRateAux(AuxId("tt"), name, rate)]
    )
    integ = Rk4Integrator(reg)
    state = State(n=0, stocks={}, rng_seed=0, aux={name: v0})

    nxt = integ.step(state, SourceResolver(), dt)
    # 4 stage evaluations, all reading the step-entry value v0 (constant within step).
    assert seen == [v0, v0, v0, v0]
    # aux advanced exactly once (by one Euler increment), after the stages.
    assert nxt.aux[name] == v0 + rate * dt

    # A second step: the 4 new reads equal the *new* step-entry value.
    seen.clear()
    integ.step(nxt, SourceResolver(), dt)
    assert seen == [v0 + rate * dt] * 4


# --- conservation exclusion ------------------------------------------------
def test_aux_only_change_conserves() -> None:
    # An aux-only delta (identical stocks, different aux) conserves trivially: the
    # ledger reasons over stocks only, so aux is invisible to it by construction.
    stocks = {s.id: s for s in [_pool("bio.c", 3.0)]}
    before = State(n=0, stocks=stocks, rng_seed=0, aux={"tt": 0.0})
    after = State(n=1, stocks=stocks, rng_seed=0, aux={"tt": 99.0})
    conservation.assert_conserved(before, after)  # must not raise


def test_aux_does_not_appear_in_ledger() -> None:
    # The per-quantity ledger is identical whether or not aux differs — aux carries
    # no conserved-quantity surface.
    stocks = {s.id: s for s in [_pool("bio.c", 3.0)]}
    before = State(n=0, stocks=stocks, rng_seed=0, aux={"tt": 0.0})
    after = State(n=1, stocks=stocks, rng_seed=0, aux={"tt": 99.0})
    ledger = conservation.compute_ledger(before, after)
    assert all(ql.residual == 0.0 for ql in ledger)


def test_unbalanced_stock_change_still_trips_gate_despite_aux() -> None:
    # Aux being outside the gate must not weaken it: an unbalanced *stock* change
    # (carbon created from nothing) still raises, regardless of any aux change.
    before = State(
        n=0,
        stocks={s.id: s for s in [_pool("bio.c", 3.0)]},
        rng_seed=0,
        aux={"tt": 0.0},
    )
    after = State(
        n=1,
        stocks={s.id: s for s in [_pool("bio.c", 5.0)]},  # +2 carbon, no counterparty
        rng_seed=0,
        aux={"tt": 1.0},
    )
    with pytest.raises(ConservationError, match="CARBON"):
        conservation.assert_conserved(before, after)


def test_stepping_with_aux_and_mass_flow_conserves_every_step() -> None:
    # A model that advances aux AND moves real mass passes the always-on every-step
    # gate (the integrator's _finalize tail) — aux advancing does not perturb the gate.
    src, sink = StockId("bio.c"), StockId("boundary.sink")
    stocks = {
        src: _pool("bio.c", 4.0),
        sink: boundary.sink(sink, Quantity.CARBON),
    }
    reg = Registry(
        flows=[_Decay(FlowId("decay"), 0, src, sink, 0.25)],
        stocks=stocks,
        aux_processes=[_ConstRateAux(AuxId("tt"), "tt", 2.0)],
    )
    integ = EulerIntegrator(reg)
    state = State(n=0, stocks=stocks, rng_seed=0)
    for _ in range(50):
        state = integ.step(state, SourceResolver(), 1.0)  # raises if any step breaks
    assert state.n == 50
    assert state.aux["tt"] == 2.0 * 50  # aux still accumulated alongside


# --- multi-rate placement guard --------------------------------------------
def test_substep_leaves_aux_untouched_while_step_report_advances_once() -> None:
    # The pinned placement (P2): substep (the multi-rate primitive) keeps aux; only
    # step_report advances it. Advancing aux in the shared substep path would advance
    # it n_sub× per master step — so aux × multi-rate is out of scope (single-rate P1).
    rate, dt, v0 = 4.0, 0.5, 1.0
    reg = Registry(
        flows=[], stocks={}, aux_processes=[_ConstRateAux(AuxId("tt"), "tt", rate)]
    )
    integ = EulerIntegrator(reg)
    state = State(n=0, stocks={}, rng_seed=0, aux={"tt": v0})
    resolver = SourceResolver()

    sub = integ.substep(state, resolver, dt)
    assert sub.state.aux["tt"] == v0  # untouched
    assert sub.state.n == state.n  # substep keeps n, too

    full = integ.step_report(state, resolver, dt)
    assert full.state.aux["tt"] == v0 + rate * dt  # advanced once
    assert full.state.n == state.n + 1


# --- determinism / registration-order independence -------------------------
# Three processes writing ONE shared accumulator name, with associativity-sensitive
# increments: the canonical (AuxId-sorted) sum ((0 + 1) + 1e16) - 1e16 == 0 loses the
# 1, whereas a non-sorting impl fed the reversed list would get 1 — so the sort is
# what the test discriminates. (Two processes would be vacuous: float + is commutative.)
_ASSOC = [
    _ConstRateAux(AuxId("a"), "acc", 1.0),
    _ConstRateAux(AuxId("b"), "acc", 1e16),
    _ConstRateAux(AuxId("c"), "acc", -1e16),
]


def _step_once(procs: Sequence[AuxProcess]) -> float:
    reg = Registry(flows=[], stocks={}, aux_processes=procs)
    out = EulerIntegrator(reg).step(
        State(n=0, stocks={}, rng_seed=0), SourceResolver(), 1.0
    )
    return out.aux["acc"]


_CANONICAL_ACC = _step_once(_ASSOC)


@given(order=st.permutations(range(len(_ASSOC))))
def test_aux_sum_is_registration_order_independent(order: tuple[int, ...]) -> None:
    shuffled = [_ASSOC[i] for i in order]
    assert _step_once(shuffled).hex() == _CANONICAL_ACC.hex()


def test_aux_sum_canonical_value_is_associativity_specific() -> None:
    # Pins the canonical order as the AuxId-sorted one (a + b + c, dropping the 1),
    # so the order-independence test above is a real discriminator, not trivially true.
    assert _CANONICAL_ACC == 0.0


def test_two_processes_write_distinct_names() -> None:
    reg = Registry(
        flows=[],
        stocks={},
        aux_processes=[
            _ConstRateAux(AuxId("a"), "alpha", 1.0),
            _ConstRateAux(AuxId("b"), "beta", 2.0),
        ],
    )
    out = EulerIntegrator(reg).step(
        State(n=0, stocks={}, rng_seed=0), SourceResolver(), 0.5
    )
    assert dict(out.aux) == {"alpha": 0.5, "beta": 1.0}


# --- State.aux primitive ---------------------------------------------------
def test_state_aux_defaults_empty_and_immutable() -> None:
    state = State(n=0, stocks={}, rng_seed=0)
    assert dict(state.aux) == {}
    with pytest.raises(TypeError):
        state.aux["x"] = 1.0  # type: ignore[index]


def test_state_aux_mapping_is_read_only() -> None:
    state = State(n=0, stocks={}, rng_seed=0, aux={"tt": 1.0})
    with pytest.raises(TypeError):
        state.aux["tt"] = 2.0  # type: ignore[index]


def test_state_detaches_from_caller_aux_dict() -> None:
    source = {"tt": 1.0}
    state = State(n=0, stocks={}, rng_seed=0, aux=source)
    source["late"] = 2.0  # mutate caller's dict afterwards
    assert "late" not in state.aux  # snapshot is unaffected


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_state_rejects_non_finite_aux(bad: float) -> None:
    with pytest.raises(ValueError, match="not finite"):
        State(n=0, stocks={}, rng_seed=0, aux={"tt": bad})


def test_state_aux_signed_zero_preserved() -> None:
    # isfinite-only validation (matching Stock.amount): -0.0 is finite and kept.
    state = State(n=0, stocks={}, rng_seed=0, aux={"z": -0.0})
    assert math.copysign(1.0, state.aux["z"]) == -1.0


# --- Registry aux discipline -----------------------------------------------
def test_registry_rejects_duplicate_aux_id() -> None:
    with pytest.raises(ValueError, match="duplicate AuxId"):
        Registry(
            flows=[],
            stocks={},
            aux_processes=[
                _ConstRateAux(AuxId("dup"), "a", 1.0),
                _ConstRateAux(AuxId("dup"), "b", 2.0),
            ],
        )


def test_registry_aux_processes_canonical_sorted() -> None:
    procs = [
        _ConstRateAux(AuxId("c"), "x", 1.0),
        _ConstRateAux(AuxId("a"), "x", 1.0),
        _ConstRateAux(AuxId("b"), "x", 1.0),
    ]
    reg = Registry(flows=[], stocks={}, aux_processes=procs)
    assert [p.id for p in reg.aux_processes] == [AuxId("a"), AuxId("b"), AuxId("c")]


def test_registry_aux_processes_default_empty() -> None:
    assert Registry(flows=[], stocks={}).aux_processes == ()


def test_aux_process_is_runtime_checkable() -> None:
    assert isinstance(_ConstRateAux(AuxId("a"), "x", 1.0), AuxProcess)
