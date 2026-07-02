"""Generate the engine cross-port trajectory vectors the Rust port is pinned against.

Phase-7 Step 2 (P7.2). The reference is the frozen ``simcore`` engine
(state/flow/arbitration/conservation/integrator/registry/environment/boundary/
events/aux/multirate). This script defines a **synthetic, transcendental-free**
scenario *on top of* that engine (its flows are pure ``+ - * /``), runs it under
Euler, RK4, and the multi-rate Strang driver, and writes a committed flat trajectory
file. The Rust port defines the **same** scenario (``tests/engine_vectors.rs``) and
gates every step **bit-exact** (Tier 1) against this file via the Step-0 hex-float
codec.

No external anchor is needed (unlike the RNG's published splitmix64): ``src/simcore``
*is* the reference here — proving Rust == Python is the whole goal, and the ~1300-test
Python suite grounds Python. ``test_crossport.py`` only checks this file stays in sync
with ``render()``.

Two scenarios:

* **A (rich, well-fed):** a forced inflow from an unclamped boundary source, a
  donor-controlled leak and transfer off a pool, a donor-controlled drain that takes a
  POPULATION stock to **extinction** (routing the residual to the loss-sink), and one
  aux process reading the pool. Run under euler / rk4 / multirate. Exercises canonical
  id-sorted reduction, the RK4 ⅙-combine, extinction loss-sink routing, the once-per-
  step aux increment, and (under multirate) ``substep`` keeping ``n`` with aux frozen.
* **B (rationing):** two forced withdrawals whose combined demand exceeds the pool, so
  the Euler min-scaling backstop fires (``rationed > 0``). Run under euler only; the
  RK4 ``ArbitrationError`` counterpart is a Rust unit test, not a trajectory.

**The synthetic flow arithmetic is written character-for-character identically here and
in Rust** (float ``*`` is not associative, so the grouping — e.g. ``(k * a) * dt`` — is
load-bearing for bit-exactness). Decimal literals parse to identical bits in both.

Regenerate with::

    uv run python tests/crossport/gen_engine_vectors.py
"""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

from simcore import boundary
from simcore.auxiliary import AuxId
from simcore.environment import Environment, SourceResolver, constant
from simcore.events import Event
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId, UnitLabel
from simcore.integrator import EulerIntegrator, Rk4Integrator, StepReport, Substepper
from simcore.multirate import Split, multirate_step
from simcore.quantities import Quantity, StockKind
from simcore.registry import Registry
from simcore.state import State, Stock

# The committed vector file the Rust integration test reads.
VECTORS_PATH = (
    Path(__file__).resolve().parents[2]
    / "rust"
    / "crates"
    / "simcore"
    / "tests"
    / "data"
    / "engine_vectors.txt"
)

_SIM = DomainId("sim")
_SIMB = DomainId("simb")

# --------------------------------------------------------------------------- #
# Shared numeric contract — decimal literals, identical in Python and Rust.    #
# --------------------------------------------------------------------------- #
DT = 0.5
INFLOW = 2.0  # forcing "inflow"
LEAK_K = 0.04
TRANSFER_K = 0.02
DRAIN_KP = 0.2
AUX_K = 0.01
STEPS_A = 20
N_SUB = 2

# Scenario B.
DT_B = 1.0
POOL_B0 = 10.0
DRAIN_B = 6.0  # each of two flows wants DRAIN_B * dt; combined demand > pool
STEPS_B = 2


# --------------------------------------------------------------------------- #
# Scenario A flows (transcendental-free; grouping mirrors Rust exactly).       #
# --------------------------------------------------------------------------- #
class ForcedIn:
    """Forced inflow ``boundary.src -> sim.a`` at a constant env rate."""

    id = FlowId("sim.forced_in")
    priority = 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        q = env.get("inflow")
        amt = q * dt
        return FlowResult(
            legs=(Leg(StockId("boundary.src"), -amt), Leg(StockId("sim.a"), amt))
        )


class Leak:
    """Donor-controlled leak ``sim.a -> boundary.snk`` (makes RK4 stages differ)."""

    id = FlowId("sim.leak")
    priority = 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        a = snapshot.stocks[StockId("sim.a")].amount
        flux = (LEAK_K * a) * dt
        return FlowResult(
            legs=(Leg(StockId("sim.a"), -flux), Leg(StockId("boundary.snk"), flux))
        )


class Transfer:
    """Donor-controlled transfer ``sim.a -> sim.b`` (a third flow touching sim.a)."""

    id = FlowId("sim.transfer")
    priority = 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        a = snapshot.stocks[StockId("sim.a")].amount
        flux = (TRANSFER_K * a) * dt
        return FlowResult(
            legs=(Leg(StockId("sim.a"), -flux), Leg(StockId("sim.b"), flux))
        )


class DrainP:
    """Donor-controlled drain ``sim.p -> boundary.snk`` — takes p to extinction."""

    id = FlowId("sim.drain_p")
    priority = 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        p = snapshot.stocks[StockId("sim.p")].amount
        flux = (DRAIN_KP * p) * dt
        return FlowResult(
            legs=(Leg(StockId("sim.p"), -flux), Leg(StockId("boundary.snk"), flux))
        )


class AuxThermal:
    """One aux accumulator ``thermal_time`` advanced by ``(AUX_K * a) * dt``."""

    id = AuxId("sim.thermal")

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        a = snapshot.stocks[StockId("sim.a")].amount
        inc = (AUX_K * a) * dt
        return {"thermal_time": inc}


def _scenario_a_stocks() -> dict[StockId, Stock]:
    stocks: dict[StockId, Stock] = {
        StockId("sim.a"): Stock(
            StockId("sim.a"),
            _SIM,
            Quantity.CARBON,
            UnitLabel("mol"),
            100.0,
            StockKind.POOL,
        ),
        StockId("sim.b"): Stock(
            StockId("sim.b"),
            _SIM,
            Quantity.CARBON,
            UnitLabel("mol"),
            0.0,
            StockKind.POOL,
        ),
        StockId("sim.p"): Stock(
            StockId("sim.p"),
            _SIM,
            Quantity.CARBON,
            UnitLabel("mol"),
            5.0,
            StockKind.POPULATION,
            extinction_threshold=1.0,
        ),
        StockId("boundary.src"): boundary.source(
            StockId("boundary.src"), Quantity.CARBON, 0.0
        ),
        StockId("boundary.snk"): boundary.sink(
            StockId("boundary.snk"), Quantity.CARBON, 0.0
        ),
    }
    ls = boundary.loss_sink(Quantity.CARBON)
    stocks[ls.id] = ls
    return stocks


def _scenario_a_state() -> State:
    # Initial aux carries thermal_time=7.0: euler/rk4 advance it; multirate freezes it
    # (substep leaves aux untouched) — the same initial state exercises both.
    return State(0, _scenario_a_stocks(), rng_seed=0, aux={"thermal_time": 7.0})


def _resolver_a() -> SourceResolver:
    return SourceResolver(forcings={"inflow": constant(INFLOW)})


# --------------------------------------------------------------------------- #
# Scenario B flows (forced over-withdrawal → Euler rationing).                 #
# --------------------------------------------------------------------------- #
class DrainForced:
    """A forced withdrawal ``simb.pool -> simb.snk`` of ``DRAIN_B * dt`` per step."""

    def __init__(self, fid: str) -> None:
        self.id = FlowId(fid)
        self.priority = 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        amt = DRAIN_B * dt
        return FlowResult(
            legs=(Leg(StockId("simb.pool"), -amt), Leg(StockId("simb.snk"), amt))
        )


def _scenario_b_stocks() -> dict[StockId, Stock]:
    return {
        StockId("simb.pool"): Stock(
            StockId("simb.pool"),
            _SIMB,
            Quantity.CARBON,
            UnitLabel("mol"),
            POOL_B0,
            StockKind.POOL,
        ),
        StockId("simb.snk"): boundary.sink(StockId("simb.snk"), Quantity.CARBON, 0.0),
    }


def _scenario_b_state() -> State:
    return State(0, _scenario_b_stocks(), rng_seed=0)


# --------------------------------------------------------------------------- #
# Trajectory capture                                                          #
# --------------------------------------------------------------------------- #
class _Row:
    """One captured step: the produced state plus its report diagnostics."""

    def __init__(self, state: State, rationed: int, events: tuple[Event, ...]) -> None:
        self.state = state
        self.rationed = rationed
        self.events = events


def _run_single(
    integrator: EulerIntegrator | Rk4Integrator,
    state: State,
    env: SourceResolver,
    dt: float,
    steps: int,
) -> list[_Row]:
    rows = [_Row(state, 0, ())]
    cur = state
    for _ in range(steps):
        report: StepReport = integrator.step_report(cur, env, dt)
        cur = report.state
        rows.append(_Row(cur, report.rationed, report.events))
    return rows


def _run_multirate(
    slow: Substepper,
    fast: Substepper,
    state: State,
    env: SourceResolver,
    dt: float,
    steps: int,
    n_sub: int,
) -> list[_Row]:
    rows = [_Row(state, 0, ())]
    cur = state
    for _ in range(steps):
        report = multirate_step(slow, fast, cur, env, dt, n_sub, split=Split.STRANG)
        cur = report.state
        rows.append(_Row(cur, report.rationed, report.events))
    return rows


def _collect() -> dict[str, list[_Row]]:
    env_a = _resolver_a()
    stocks_a = _scenario_a_stocks()

    # Euler / RK4: one registry with all flows + the aux process.
    all_flows = [ForcedIn(), Leak(), Transfer(), DrainP()]
    reg_full = Registry(all_flows, stocks_a, aux_processes=[AuxThermal()])
    euler = EulerIntegrator(reg_full)
    rk4 = Rk4Integrator(reg_full)

    # Multirate: disjoint slow/fast registries over the shared stocks, flows-only
    # (multirate never advances aux — so thermal_time stays frozen at 7.0).
    slow_reg = Registry([ForcedIn()], stocks_a)
    fast_reg = Registry([Leak(), Transfer(), DrainP()], stocks_a)
    slow = EulerIntegrator(slow_reg)
    fast = EulerIntegrator(fast_reg)

    # Scenario B (euler only).
    reg_b = Registry(
        [DrainForced("simb.drain1"), DrainForced("simb.drain2")], _scenario_b_stocks()
    )
    euler_b = EulerIntegrator(reg_b)

    return {
        "euler": _run_single(euler, _scenario_a_state(), env_a, DT, STEPS_A),
        "rk4": _run_single(rk4, _scenario_a_state(), env_a, DT, STEPS_A),
        "multirate": _run_multirate(
            slow, fast, _scenario_a_state(), env_a, DT, STEPS_A, N_SUB
        ),
        "ration": _run_single(
            euler_b, _scenario_b_state(), SourceResolver(), DT_B, STEPS_B
        ),
    }


# Scheme render order (fixed for a deterministic file).
_SCHEMES = ("euler", "rk4", "multirate", "ration")


def render() -> str:
    """The exact text of the committed vector file (LF newlines)."""
    lines = [
        "# Engine cross-port trajectory vectors — "
        "GENERATED by tests/crossport/gen_engine_vectors.py",
        "# stock lines: stock <TAB> scheme <TAB> n <TAB> stock_id <TAB> "
        "amount(float.hex)",
        "# aux lines:   aux   <TAB> scheme <TAB> n <TAB> name <TAB> value(float.hex)",
        "# meta lines:  meta  <TAB> scheme <TAB> n <TAB> rationed <TAB> n_events",
        "# event lines: event <TAB> scheme <TAB> n <TAB> stock <TAB> quantity <TAB> "
        "residual(float.hex)",
    ]
    trajectories = _collect()
    for scheme in _SCHEMES:
        rows = trajectories[scheme]
        for n, row in enumerate(rows):
            for sid in sorted(row.state.stocks):
                amt = row.state.stocks[sid].amount
                lines.append(f"stock\t{scheme}\t{n}\t{sid}\t{amt.hex()}")
            for name in sorted(row.state.aux):
                val = row.state.aux[name]
                lines.append(f"aux\t{scheme}\t{n}\t{name}\t{val.hex()}")
            if n > 0:
                lines.append(f"meta\t{scheme}\t{n}\t{row.rationed}\t{len(row.events)}")
                for ev in row.events:
                    lines.append(
                        f"event\t{scheme}\t{n}\t{ev.stock}\t{ev.quantity.value}\t"
                        f"{ev.residual.hex()}"
                    )
    return "\n".join(lines) + "\n"


def main() -> int:
    VECTORS_PATH.parent.mkdir(parents=True, exist_ok=True)
    VECTORS_PATH.write_text(render(), encoding="utf-8", newline="\n")
    trajectories = _collect()
    total_events = sum(
        len(row.events) for rows in trajectories.values() for row in rows
    )
    print(
        f"wrote engine vectors for {len(_SCHEMES)} schemes "
        f"({total_events} extinction events) to {VECTORS_PATH}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
