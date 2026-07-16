"""Generate the authoring cross-port vectors the Rust `authoring`/`expr` port is pinned
against (Phase 9, Step 4a).

Two committed files, two parity surfaces (the plan's genuinely-new cross-port surface):

* **Parse-parity** (`parse_vectors.txt`, Tier-0 structural): a rate string lowers to the
  *same* AST in Python and Rust. Each accept case is `input <TAB> canonical-S-expr` (the
  S-expr rendered here, re-derived by the Rust parser + `render_sexpr` and asserted
  equal); each reject case is `reject <TAB> input` (both ports must error — the message
  text is deliberately NOT pinned). This is the sole Tier-0 parse-parity risk surface
  (`expr_parser`'s precedence/associativity + the deferred-grammar rejection).

* **Trajectory-parity** (`traj_vectors.txt`, Tier-1 bit-exact): an authored
  `DeclarativeFlow` built from a *parsed* rate string, run under Euler AND RK4, per-step
  hex-float. The **SelfDischarge re-expression anchor** (the frozen `SelfDischarge`'s
  `k·battery`, the plan's Step-2 anchor — no new golden) proves the VM+parser+integrator
  interplay is bit-identical; a **synthetic authored scenario** additionally exercises
  every remaining VM node in the *evaluated* path (`ForcingRef`/`StepN`/`Neg`/`+`/`-`),
  which SelfDischarge (only `*`) does not — the engine-vectors "synthetic scenario"
  discipline applied to authored flows. Transcendental-free ⇒ Tier-1 bit-exact.

The AST is plain data and the arithmetic is `+ - *` only, so decimals round-trip and the
grouping (`increment = rate*dt`, `coeff*increment`) is bit-identical across ports. The
Rust integration test (`rust/crates/authoring/tests/authoring_vectors.rs`) reads both
files and gates every row; `test_crossport.py` guards the files stay in sync with
`render()`.

Regenerate with::

    uv run python tests/crossport/gen_authoring_vectors.py
"""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

from authoring.expr_parser import parse_rate_expr
from domains.power.loader import load_self_discharge_params
from simcore import boundary
from simcore.environment import SourceResolver, constant
from simcore.events import Event
from simcore.expr import (
    BinOp,
    Const,
    DeclarativeFlow,
    Expr,
    ForcingRef,
    Monod,
    Neg,
    ParamRef,
    StepN,
    StockRef,
)
from simcore.flow import Flow
from simcore.ids import DomainId, FlowId, StockId, UnitLabel
from simcore.integrator import EulerIntegrator, Rk4Integrator, StepReport
from simcore.quantities import Quantity, StockKind
from simcore.registry import Registry
from simcore.state import State, Stock

_DATA_DIR = (
    Path(__file__).resolve().parents[2]
    / "rust"
    / "crates"
    / "authoring"
    / "tests"
    / "data"
)
PARSE_PATH = _DATA_DIR / "parse_vectors.txt"
TRAJ_PATH = _DATA_DIR / "traj_vectors.txt"


# --------------------------------------------------------------------------- #
# Canonical S-expression rendering — MUST match rust `authoring::sexpr`.        #
# --------------------------------------------------------------------------- #
def render_sexpr(node: Expr) -> str:
    """Render an AST to the canonical S-expr (the parse-parity diff form).

    `Const` renders through `float.hex()` (bit-exact — the Rust side reproduces the
    identical spelling via the hex-float codec), so a literal's parity is bit-exact.
    """
    if isinstance(node, Const):
        return f"(const {node.value.hex()})"
    if isinstance(node, StockRef):
        return f'(stock "{node.stock}")'
    if isinstance(node, ParamRef):
        return f'(param "{node.name}")'
    if isinstance(node, ForcingRef):
        return f'(forcing "{node.name}")'
    if isinstance(node, StepN):
        return "n"
    if isinstance(node, Neg):
        return f"(neg {render_sexpr(node.operand)})"
    if isinstance(node, BinOp):
        return f"({node.op} {render_sexpr(node.left)} {render_sexpr(node.right)})"
    if isinstance(node, Monod):
        return (
            f"(monod {render_sexpr(node.substrate)} "
            f"{render_sexpr(node.half_saturation)})"
        )
    raise TypeError(f"not an Expr node: {node!r}")  # pragma: no cover


# The accept cases — every node type + precedence/associativity + the exact-literal and
# dotted-id forms the parser must lower identically. The Rust parser re-derives each
# S-expr; a divergence is a parse-parity failure.
_ACCEPT_CASES: tuple[str, ...] = (
    # the anchor's rate, verbatim
    'param("self_discharge_rate") * stock("power.battery")',
    # every leaf form
    "n",
    "2.5",
    "1.0e-8",
    "-3.0",
    'stock("sim.a")',
    'param("k")',
    'forcing("par")',
    # precedence + associativity (the load-bearing parse-parity risk)
    "n - n - n",  # left-assoc: ((n - n) - n)
    "n + n * n",  # * tighter than +
    "n * n + n",
    "-n * n",  # unary minus tighter than *
    "(n + n) * n",  # parens override
    "n * (n + n)",
    "-(n + n)",  # unary minus over a parenthesised sum
    # a nested demand-shaped rate over refs (the shape a real authored flow uses)
    'param("k") * (stock("sim.a") - param("floor"))',
    'forcing("q") + n * param("g") - param("c")',
    # exact-literal parity: awkward decimals + exponents must round-trip bit-for-bit
    "0.1 + 0.2",
    "123456789.123456789",
    "1e10 * 2.0",
    # monod (Tier 2) — the arg ORDER is a frozen semantic choice, so it is pinned here
    # cross-port, not merely unit-tested per port.
    'monod(stock("sim.a"), param("k"))',
    # Vmax arrives through the already-frozen `*` — why no 3-arg form is needed.
    'param("vmax") * monod(stock("sim.a"), param("k"))',
    # full sub-expressions as args (not just refs), incl. precedence inside an arg
    'monod(stock("sim.a") - param("floor"), param("k") * 2.0)',
    "monod(-n, 1.0 + 2.0)",
    # nesting, and monod interacting with the surrounding precedence
    'monod(monod(n, param("a")), param("b"))',
    'n + monod(n, param("k")) * n',
)

# The reject cases — the deferred grammar + malformed input. BOTH ports must error; the
# message text is not a parity target (Tier-0 is accept→same-AST, reject→both-error).
_REJECT_CASES: tuple[str, ...] = (
    "n / n",  # division STILL deferred — monod guards its own denominator, so it
    # resolved x/0 internally without exposing the raw form
    "exp(n)",  # unknown function/identifier — the rest of the set is still deferred
    "sqrt(n)",
    "n ** 2",  # ** is two '*' tokens then a bad primary
    "   ",  # empty
    "",  # empty
    "(n + n",  # unbalanced
    "n + n)",  # unbalanced
    "n n",  # trailing junk
    "n @ n",  # stray char
    'stock("power.battery)',  # unterminated string
    'stock("")',  # empty ref arg
    "stock(n)",  # ref arg not a string
    "* n",  # leading binary op
    "n +",  # dangling binary op
    "dt",  # no dt token (grammar-enforced)
    # monod arity — both under- and over-application error, so the arity of the one
    # function form cannot be got wrong silently on either port
    "monod(n)",
    "monod(n, n, n)",
    "monod()",
    "monod(n n)",  # missing comma
    "monod(n,)",  # dangling comma
    "monod n, n",  # missing parens
    "monod(n, n",  # unbalanced
    # the comma is confined to the call — it is NOT a sequencing operator
    "n, n",
    ",",
)


# --------------------------------------------------------------------------- #
# Trajectory scenarios (authored DeclarativeFlows, transcendental-free).       #
# --------------------------------------------------------------------------- #
_POWER = DomainId("power")
_BOUNDARY = DomainId("boundary")
_SIM = DomainId("sim")

# Anchor: the frozen SelfDischarge k (same loader/value/bounds — the authored constant
# IS the frozen one), a −1/+1 ENERGY split battery → waste_heat.
_SD_K = load_self_discharge_params().self_discharge_rate
SD_DT = 3600.0
SD_STEPS = 24
SD_BATTERY0 = 1.0e7

# Synthetic authored scenario constants.
SYN_DT = 0.5
SYN_STEPS = 24
SYN_POOL0 = 100.0
SYN_Q = 2.0  # forcing "q"
SYN_G = 0.01
SYN_C = 0.5
SYN_K = 0.03


def _energy_stock(sid: str, domain: DomainId, amount: float, kind: StockKind) -> Stock:
    return Stock(StockId(sid), domain, Quantity.ENERGY, UnitLabel("J"), amount, kind)


def _self_discharge_traj() -> dict[str, list[_Row]]:
    battery = _energy_stock("power.battery", _POWER, SD_BATTERY0, StockKind.POOL)
    waste = _energy_stock("boundary.waste_heat", _BOUNDARY, 0.0, StockKind.BOUNDARY)
    stocks = {battery.id: battery, waste.id: waste}
    rate = parse_rate_expr('param("self_discharge_rate") * stock("power.battery")')
    flow = DeclarativeFlow(
        id=FlowId("power.self_discharge"),
        priority=0,
        rate=rate,
        stoichiometry=(
            (StockId("power.battery"), -1.0),
            (StockId("boundary.waste_heat"), 1.0),
        ),
        params=(("self_discharge_rate", _SD_K),),
    )
    return _run_both(stocks, [flow], SourceResolver(), SD_DT, SD_STEPS)


def _carbon_stock(sid: str, domain: DomainId, amount: float, kind: StockKind) -> Stock:
    return Stock(StockId(sid), domain, Quantity.CARBON, UnitLabel("mol"), amount, kind)


def _synthetic_traj() -> dict[str, list[_Row]]:
    """A well-fed authored scenario exercising every remaining VM node in evaluation.

    Two authored flows over one pool, both balanced by construction (integer ±1
    coeffs). The forced inflow's rate uses ForcingRef/StepN/Const/Mul/Add/Sub; the
    donor leak's rate uses Neg (over a parenthesised product). Forced source is
    unclamped (never rations); the leak is donor-controlled (RK4 ≢ Euler). Sized so the
    inflow rate stays positive for every n and the pool never empties — rationed == 0,
    events == ().
    """
    pool = _carbon_stock("sim.pool", _SIM, SYN_POOL0, StockKind.POOL)
    src = boundary.source(StockId("boundary.src"), Quantity.CARBON, 0.0)
    snk = boundary.sink(StockId("boundary.snk"), Quantity.CARBON, 0.0)
    stocks = {pool.id: pool, src.id: src, snk.id: snk}

    # rate = q + n*g - c  (forced; ForcingRef, StepN, Mul, Add, Sub, ParamRef, Const)
    forced_rate = parse_rate_expr('forcing("q") + n * param("g") - param("c")')
    forced = DeclarativeFlow(
        id=FlowId("sim.forced_in"),
        priority=0,
        rate=forced_rate,
        stoichiometry=(
            (StockId("boundary.src"), -1.0),
            (StockId("sim.pool"), 1.0),
        ),
        params=(("c", SYN_C), ("g", SYN_G)),
    )
    # rate = -(k * pool)  (donor-controlled; Neg over a product). Paired with a flipped
    # +1/−1 stoich so the physics is an ordinary leak pool → snk, but written with a
    # unary minus so the evaluated path covers Neg.
    leak_rate = parse_rate_expr('-(param("k") * stock("sim.pool"))')
    leak = DeclarativeFlow(
        id=FlowId("sim.leak"),
        priority=0,
        rate=leak_rate,
        stoichiometry=(
            (StockId("sim.pool"), 1.0),
            (StockId("boundary.snk"), -1.0),
        ),
        params=(("k", SYN_K),),
    )
    return _run_both(stocks, [forced, leak], _syn_resolver(), SYN_DT, SYN_STEPS)


def _syn_resolver() -> SourceResolver:
    return SourceResolver(forcings={"q": constant(SYN_Q)})


# Monod scenario constants (Tier 2). A saturating drain + a saturating forced inflow.
MON_DT = 0.5
MON_STEPS = 24
MON_POOL0 = 100.0
MON_VMAX = 2.0  # max drain rate
MON_K = 10.0  # drain half-saturation
MON_Q = 2.0  # forcing "q"
MON_KQ = 1.0  # inflow half-saturation
MON_VIN = 3.0  # max inflow rate


def _monod_traj() -> dict[str, list[_Row]]:
    """An authored scenario exercising ``Monod`` in the *evaluated* path.

    Tier-1 **bit-exact**: division is an IEEE-754 basic op (correctly-rounded,
    deterministic cross-port), not a libm transcendental — so this vector asserts
    bit-identity like the ``SelfDischarge`` anchor, NOT a tolerance band.

    The drain is the frozen ``f_O2`` story in miniature and is why ``monod`` earns its
    place: ``vmax · monod(pool, k)`` is **donor-controlled and self-limiting** — as the
    pool falls the rate smoothly shuts off, so positivity comes from *kinetics*, never
    the Euler backstop (``rationed == 0``). It is also nonlinear in the pool, so
    RK4 ≢ Euler and the two schemes are independent evidence. The inflow additionally
    covers a ``ForcingRef`` inside a monod arg and a monod on the *left* of a ``*``.

    Sized well-fed: inflow ``3.0 · 2/(2+1) = 2.0`` against a drain that starts at
    ``2.0 · 100/110 ≈ 1.82``, so the pool rises gently and never approaches zero.
    """
    pool = _carbon_stock("sim.pool", _SIM, MON_POOL0, StockKind.POOL)
    src = boundary.source(StockId("boundary.src"), Quantity.CARBON, 0.0)
    snk = boundary.sink(StockId("boundary.snk"), Quantity.CARBON, 0.0)
    stocks = {pool.id: pool, src.id: src, snk.id: snk}

    # rate = vmax * monod(pool, k) — saturating, donor-controlled, self-limiting.
    drain_rate = parse_rate_expr('param("vmax") * monod(stock("sim.pool"), param("k"))')
    drain = DeclarativeFlow(
        id=FlowId("sim.saturating_drain"),
        priority=0,
        rate=drain_rate,
        stoichiometry=(
            (StockId("sim.pool"), -1.0),
            (StockId("boundary.snk"), 1.0),
        ),
        params=(("k", MON_K), ("vmax", MON_VMAX)),
    )
    # rate = monod(q, kq) * vin — a monod over a ForcingRef, on the left of a `*`.
    inflow_rate = parse_rate_expr('monod(forcing("q"), param("kq")) * param("vin")')
    inflow = DeclarativeFlow(
        id=FlowId("sim.saturating_in"),
        priority=0,
        rate=inflow_rate,
        stoichiometry=(
            (StockId("boundary.src"), -1.0),
            (StockId("sim.pool"), 1.0),
        ),
        params=(("kq", MON_KQ), ("vin", MON_VIN)),
    )
    resolver = SourceResolver(forcings={"q": constant(MON_Q)})
    return _run_both(stocks, [drain, inflow], resolver, MON_DT, MON_STEPS)


# --------------------------------------------------------------------------- #
# Trajectory capture (mirrors gen_engine_vectors.py).                          #
# --------------------------------------------------------------------------- #
class _Row:
    def __init__(self, state: State, rationed: int, events: tuple[Event, ...]) -> None:
        self.state = state
        self.rationed = rationed
        self.events = events


def _run_one(
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


def _run_both(
    stocks: Mapping[StockId, Stock],
    flows: list[Flow],
    env: SourceResolver,
    dt: float,
    steps: int,
) -> dict[str, list[_Row]]:
    reg = Registry(list(flows), dict(stocks))
    euler = EulerIntegrator(reg)
    rk4 = Rk4Integrator(reg)
    return {
        "euler": _run_one(euler, State(0, dict(stocks), rng_seed=0), env, dt, steps),
        "rk4": _run_one(rk4, State(0, dict(stocks), rng_seed=0), env, dt, steps),
    }


# scenario -> {scheme -> rows}
def _collect() -> dict[str, dict[str, list[_Row]]]:
    return {
        "self_discharge": _self_discharge_traj(),
        "synthetic": _synthetic_traj(),
        "monod": _monod_traj(),
    }


_SCENARIOS = ("self_discharge", "synthetic", "monod")
_SCHEMES = ("euler", "rk4")


def render_parse() -> str:
    """The exact text of `parse_vectors.txt` (LF newlines)."""
    lines = [
        "# Authoring parse-parity vectors — "
        "GENERATED by tests/crossport/gen_authoring_vectors.py",
        "# accept lines: accept <TAB> input <TAB> canonical-sexpr",
        "# reject lines: reject <TAB> input   (both ports must error; msg not pinned)",
    ]
    for text in _ACCEPT_CASES:
        sexpr = render_sexpr(parse_rate_expr(text))
        lines.append(f"accept\t{text}\t{sexpr}")
    for text in _REJECT_CASES:
        lines.append(f"reject\t{text}")
    return "\n".join(lines) + "\n"


def render_traj() -> str:
    """The exact text of `traj_vectors.txt` (LF newlines)."""
    lines = [
        "# Authoring trajectory-parity vectors — "
        "GENERATED by tests/crossport/gen_authoring_vectors.py",
        "# stock lines: stock <TAB> scenario <TAB> scheme <TAB> n <TAB> stock_id "
        "<TAB> amount(float.hex)",
        "# meta lines:  meta  <TAB> scenario <TAB> scheme <TAB> n <TAB> rationed "
        "<TAB> n_events",
    ]
    trajectories = _collect()
    for scenario in _SCENARIOS:
        for scheme in _SCHEMES:
            rows = trajectories[scenario][scheme]
            for n, row in enumerate(rows):
                for sid in sorted(row.state.stocks):
                    amt = row.state.stocks[sid].amount
                    lines.append(
                        f"stock\t{scenario}\t{scheme}\t{n}\t{sid}\t{amt.hex()}"
                    )
                if n > 0:
                    lines.append(
                        f"meta\t{scenario}\t{scheme}\t{n}\t{row.rationed}\t"
                        f"{len(row.events)}"
                    )
    return "\n".join(lines) + "\n"


def main() -> int:
    _DATA_DIR.mkdir(parents=True, exist_ok=True)
    PARSE_PATH.write_text(render_parse(), encoding="utf-8", newline="\n")
    TRAJ_PATH.write_text(render_traj(), encoding="utf-8", newline="\n")
    # Sanity: the synthetic scenario must be well-fed (the "it ran clean" guard).
    trajectories = _collect()
    total_rationed = sum(
        row.rationed
        for scenario in _SCENARIOS
        for scheme in _SCHEMES
        for row in trajectories[scenario][scheme]
    )
    total_events = sum(
        len(row.events)
        for scenario in _SCENARIOS
        for scheme in _SCHEMES
        for row in trajectories[scenario][scheme]
    )
    print(
        f"wrote {len(_ACCEPT_CASES)} accept + {len(_REJECT_CASES)} reject parse cases "
        f"to {PARSE_PATH.name}; {len(_SCENARIOS)} trajectory scenarios "
        f"(rationed={total_rationed}, events={total_events}) to {TRAJ_PATH.name}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
