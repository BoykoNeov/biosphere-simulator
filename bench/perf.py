"""Phase 0.5 Step 6 — performance baseline harness (NON-GATING).

An out-of-core measurement script (repo root, **outside** ``simcore``) that sweeps
the engine's per-step cost across **stock count**, **domain count**, and **scheme**
(Euler / RK4 / Strang+RK4 multi-rate), reporting **steps/sec** and **peak traced
Python heap** (``tracemalloc``). Stdlib only — no third-party deps — so it neither
touches the ``simcore`` purity gate nor adds a runtime dependency; it imports the
core only as a measurement target.

**Non-gating.** Absolute numbers are machine-dependent: this is a *regression
reference* committed to ``docs/perf-baseline.md``, not a pass/fail test. A future
phase may add a relative-regression check; Phase 0.5 only establishes the baseline.

**Scenario — a closed ring of CARBON ``POOL`` stocks.** ``N`` stocks
``s_0 … s_{N-1}`` with one first-order transfer flow per stock, ``s_i -> s_{i+1 mod
N}`` at rate ``k`` (each leg ``= k·dt·amount``). The ring is the load-bearing choice:
**exactly one outgoing flow per pool** caps per-stock demand at ``k·dt·s_i`` for *any*
``N``, so with ``k·dt`` small the system stays well-fed at every size — no Euler
rationing, no RK4 hard-error (a hub/star would pile ``N−1`` draws on the centre and
over-draw it). Closed (total mass conserved, no boundary reservoir) so the every-step
conservation gate passes; uniform initial amounts keep every RK4 stage amount positive.
Generalizes the closed A⇌B exchange of ``tests/test_stability.py`` (``N = 2`` is that
exchange) to arbitrary ``N``.

**Methodology.**
  * **steps/sec** is counted in **master steps** (``n -> n+1``) for *all* schemes, so
    they are directly comparable. Each cell is calibrated to a target wall-time then
    measured **best-of-R** (min time wins — filters OS-scheduling noise); ``gc`` is
    disabled across the timed interval.
  * **peak memory** is a **separate** run (never time a ``tracemalloc``-instrumented
    loop) reporting the high-water mark of the traced Python heap — *not* RSS. The
    scenario is built *inside* the traced region so the registry/flows/stocks count.
  * Each cell is sanity-checked (``rationed == 0``, no events) so a mis-scaled
    scenario fails loudly rather than producing meaningless numbers.

Run ``python bench/perf.py`` (via ``uv run``) to print the report; ``--out PATH`` to
write it; ``--quick`` for a fast subset.
"""

from __future__ import annotations

import argparse
import gc
import os
import platform
import subprocess
import sys
import time
import tracemalloc
from collections.abc import Callable
from dataclasses import dataclass
from datetime import UTC, datetime

from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import Rk4Integrator, StepReport
from simcore.multirate import multirate_step
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- scenario parameters ----------------------------------------------------
RATE = 0.5  # first-order transfer rate k
DT = 0.1  # k·dt = 0.05 per step — well-fed (no over-draw) at any N
AMOUNT = 1.0  # uniform initial amount per pool (total = N, conserved)
N_SUB = 2  # multi-rate fast sub-steps (order pinned in tests; here it only trims work)

Stepper = Callable[[State], StepReport]
StepperFactory = Callable[[dict[StockId, Stock], list["_Transfer"]], Stepper]


# --- test-local flow --------------------------------------------------------
@dataclass(frozen=True)
class _Transfer:
    """One ring edge ``src -> dst`` at first-order ``rate`` (leg ``= rate·src·dt``).

    Balanced (``Σ legs == 0``), so a ring of these conserves carbon with no boundary
    reservoir — mirrors ``tests/test_stability._Exchange``.
    """

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.dst, moved)))


# --- scenario builder -------------------------------------------------------
def _sid(i: int) -> StockId:
    return StockId(f"ring.s{i:05d}")


def _fid(i: int) -> FlowId:
    return FlowId(f"ring.f{i:05d}")


def build_ring(
    n_stocks: int, n_domains: int = 1
) -> tuple[dict[StockId, Stock], list[_Transfer]]:
    """A closed ring of ``n_stocks`` CARBON ``POOL`` stocks over ``n_domains`` domains.

    Stock ``i`` is assigned domain ``d{i % n_domains}`` (round-robin), so domain count
    is an independent axis from stock count. Flow ``i`` transfers ``s_i -> s_{i+1 mod
    n_stocks}``; when neighbours fall in different domains the flow is simply
    cross-domain (allowed). ``n_stocks`` must be ``>= 2`` (a 1-ring would be a
    self-loop — a duplicate leg on one stock, which ``FlowResult`` rejects).
    """
    if n_stocks < 2:
        raise ValueError(f"ring needs >= 2 stocks, got {n_stocks}")
    unit = canonical_unit(Quantity.CARBON)
    stocks: dict[StockId, Stock] = {}
    for i in range(n_stocks):
        sid = _sid(i)
        stocks[sid] = Stock(
            id=sid,
            domain=DomainId(f"d{i % n_domains}"),
            quantity=Quantity.CARBON,
            unit=unit,
            amount=AMOUNT,
            kind=StockKind.POOL,
        )
    flows = [
        _Transfer(_fid(i), 0, _sid(i), _sid((i + 1) % n_stocks), RATE)
        for i in range(n_stocks)
    ]
    return stocks, flows


# --- per-scheme steppers ----------------------------------------------------
# Each factory takes the run's stock dict + flow list and returns a
# `state -> StepReport` callable — one master step (n -> n+1) regardless of scheme.
def _euler(stocks: dict[StockId, Stock], flows: list[_Transfer]) -> Stepper:
    from simcore.integrator import EulerIntegrator

    integ = EulerIntegrator(Registry(flows, stocks))
    env = SourceResolver()
    return lambda s: integ.step_report(s, env, DT)


def _rk4(stocks: dict[StockId, Stock], flows: list[_Transfer]) -> Stepper:
    integ = Rk4Integrator(Registry(flows, stocks))
    env = SourceResolver()
    return lambda s: integ.step_report(s, env, DT)


def _multirate(stocks: dict[StockId, Stock], flows: list[_Transfer]) -> Stepper:
    # Disjoint fast/slow flow sets over the shared stock dict (N3), split by edge
    # parity. The split is a *nominal* partition (no real stiffness separation here),
    # so multi-rate is pure overhead at this scenario — the cost it trades accuracy
    # for only pays off when a genuinely fast domain can sub-step a slow one.
    fast_flows = [f for i, f in enumerate(flows) if i % 2 == 0]
    slow_flows = [f for i, f in enumerate(flows) if i % 2 == 1]
    fast = Rk4Integrator(Registry(fast_flows, stocks))
    slow = Rk4Integrator(Registry(slow_flows, stocks))
    env = SourceResolver()
    return lambda s: multirate_step(slow, fast, s, env, DT, N_SUB)


SCHEMES: dict[str, StepperFactory] = {
    "Euler": _euler,
    "RK4": _rk4,
    "Multi-rate": _multirate,
}


# --- measurement primitives -------------------------------------------------
def _sanity_check(make_stepper: StepperFactory, n_stocks: int, n_domains: int) -> None:
    """Run a few steps and assert the scenario is well-fed + conserving.

    The integrator already asserts conservation every step and RK4 hard-errors on an
    over-draw, so a broken scenario would raise here; the explicit ``rationed``/events
    asserts document intent and catch a silently-scaling Euler config.
    """
    stocks, flows = build_ring(n_stocks, n_domains)
    step = make_stepper(stocks, flows)
    state = State(n=0, stocks=stocks, rng_seed=0)
    for _ in range(5):
        report = step(state)
        if report.rationed != 0 or report.events:
            raise AssertionError(
                f"scenario not well-fed (N={n_stocks}, D={n_domains}): "
                f"rationed={report.rationed}, events={report.events}"
            )
        state = report.state


def _run_n(step: Stepper, state0: State, n: int) -> tuple[State, float]:
    """Drive ``step`` for ``n`` master steps; return ``(final_state, elapsed_s)``.

    Scalars only in the loop (no accumulation) — mirrors ``test_stability._run`` so we
    do not inflate timing/memory with a growing list of states.
    """
    state = state0
    t0 = time.perf_counter()
    for _ in range(n):
        state = step(state).state
    return state, time.perf_counter() - t0


def steps_per_sec(
    make_stepper: StepperFactory,
    n_stocks: int,
    n_domains: int,
    *,
    target_s: float,
    repeats: int,
) -> float:
    """Best-of-``repeats`` steps/sec for one cell, calibrated to ``target_s`` wall-time.

    Warms up, calibrates the step count to hit roughly ``target_s``, then takes the
    fastest of ``repeats`` timed runs (min time → max steps/sec). ``gc`` is disabled
    across each timed interval so a collection cycle cannot land inside a measurement.
    """
    stocks, flows = build_ring(n_stocks, n_domains)
    step = make_stepper(stocks, flows)
    state0 = State(n=0, stocks=stocks, rng_seed=0)

    # Warm up (caches/branch prediction; first-call allocation), then calibrate.
    warm, _ = _run_n(step, state0, 8)
    _, cal_t = _run_n(step, warm, 64)
    per_step = cal_t / 64 if cal_t > 0 else 1e-9
    n = max(64, int(target_s / per_step))

    best = float("inf")
    gc_was_enabled = gc.isenabled()
    gc.disable()
    try:
        for _ in range(repeats):
            _, elapsed = _run_n(step, state0, n)
            best = min(best, elapsed)
    finally:
        if gc_was_enabled:
            gc.enable()
    return n / best if best > 0 else float("inf")


def peak_heap_kib(
    make_stepper: StepperFactory, n_stocks: int, n_domains: int, *, steps: int
) -> float:
    """Peak traced Python heap (KiB) while building + stepping the scenario.

    A **separate** run from timing (tracemalloc perturbs timing). The scenario is built
    *inside* the traced region so the registry, flow objects, and stock dict all count
    toward the peak; old per-step ``State`` snapshots are GC'd, so the high-water mark
    is reached within a few steps and does not grow with ``steps``.
    """
    gc.collect()
    tracemalloc.start()
    stocks, flows = build_ring(n_stocks, n_domains)
    step = make_stepper(stocks, flows)
    state = State(n=0, stocks=stocks, rng_seed=0)
    for _ in range(steps):
        state = step(state).state
    _, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
    return peak / 1024.0


# --- sweep rows -------------------------------------------------------------
@dataclass(frozen=True)
class Cell:
    sps: float  # steps/sec (best of repeats)
    kib: float  # peak traced heap, KiB


def stock_sweep(
    sizes: list[int], *, target_s: float, repeats: int, mem_steps: int
) -> dict[int, dict[str, Cell]]:
    """For each ``N`` (single domain) and each scheme: steps/sec + peak heap."""
    rows: dict[int, dict[str, Cell]] = {}
    for n in sizes:
        rows[n] = {}
        for name, factory in SCHEMES.items():
            _sanity_check(factory, n, 1)
            sps = steps_per_sec(factory, n, 1, target_s=target_s, repeats=repeats)
            kib = peak_heap_kib(factory, n, 1, steps=mem_steps)
            rows[n][name] = Cell(sps=sps, kib=kib)
            print(f"  N={n:>4} {name:<10} {sps:>12,.0f} steps/s  {kib:>9,.1f} KiB")
    return rows


def domain_sweep(
    n_stocks: int, domain_counts: list[int], *, target_s: float, repeats: int
) -> dict[int, float]:
    """At fixed ``N`` (RK4), steps/sec vs domain count — expected flat (orthogonal)."""
    rows: dict[int, float] = {}
    for d in domain_counts:
        _sanity_check(_rk4, n_stocks, d)
        sps = steps_per_sec(_rk4, n_stocks, d, target_s=target_s, repeats=repeats)
        rows[d] = sps
        print(f"  domain D={d:>4} (N={n_stocks}, RK4) {sps:>12,.0f} steps/s")
    return rows


# --- report -----------------------------------------------------------------
def _git_commit() -> str:
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        return out.stdout.strip() or "unknown"
    except (subprocess.SubprocessError, OSError):
        return "unknown"


def _machine_block() -> list[str]:
    cpu = platform.processor() or platform.machine() or "unknown"
    return [
        f"- **Date (UTC):** {datetime.now(UTC):%Y-%m-%d %H:%M}",
        f"- **Commit:** `{_git_commit()}`",
        f"- **Platform:** {platform.platform()}",
        f"- **CPU:** {cpu} ({os.cpu_count()} logical cores)",
        f"- **Python:** {platform.python_implementation()} {platform.python_version()}",
    ]


def _table(header: list[str], rows: list[list[str]]) -> list[str]:
    sep = ["---"] * len(header)
    lines = ["| " + " | ".join(header) + " |", "| " + " | ".join(sep) + " |"]
    lines += ["| " + " | ".join(r) + " |" for r in rows]
    return lines


def render_report(
    stock_rows: dict[int, dict[str, Cell]],
    domain_n: int,
    domain_rows: dict[int, float],
    *,
    target_s: float,
    repeats: int,
    mem_steps: int,
) -> str:
    schemes = list(SCHEMES)
    out: list[str] = []
    out.append("# Performance baseline (Phase 0.5, Step 6)")
    out.append("")
    out.append(
        "**NON-GATING.** Absolute numbers are machine-dependent — this is a tracked "
        "*regression reference*, not a pass/fail gate. Regenerate with "
        "`uv run python bench/perf.py --out docs/perf-baseline.md`."
    )
    out.append("")
    out.extend(_machine_block())
    out.append("")
    out.append("## Method")
    out.append("")
    out.append(
        "Scenario: a **closed ring of `N` CARBON `POOL` stocks**, one first-order "
        "transfer flow per stock (`s_i -> s_{i+1 mod N}`, `k·dt = "
        f"{RATE * DT:g}`). One outgoing flow per pool caps per-stock demand, so the "
        "ring stays well-fed (no Euler rationing, no RK4 over-draw hard-error) and "
        "closed (carbon conserved with no boundary reservoir) at every `N`. `N = 2` "
        "is the closed A⇌B exchange of `tests/test_stability.py`. Each measured cell "
        "is sanity-checked `rationed == 0` / no events."
    )
    out.append("")
    out.append(
        "**steps/sec** counts **master steps** (`n -> n+1`) for every scheme, so the "
        "three are directly comparable; each cell is calibrated to ~"
        f"{target_s:g}s then measured best-of-{repeats} (min time), with `gc` disabled "
        "across the timed interval. **Peak heap** is the high-water mark of the traced "
        "Python heap (`tracemalloc`, **not** RSS), measured in a separate run "
        f"({mem_steps} steps) with the scenario built inside the traced region."
    )
    out.append("")

    # Table 1 — steps/sec vs stock count (also the scheme comparison).
    out.append("## Stock-count scaling — throughput (steps/sec)")
    out.append("")
    out.append(
        "Per-step cost is ~O(`N`) (one flow per stock); the three schemes also "
        "compare here. RK4 evaluates the flow set 4× per step (the per-step ratio to "
        "Euler grows toward 4× as fixed overhead is amortized at larger `N`); "
        "multi-rate is **slower still and is pure overhead in this scenario** — the "
        "fast/slow split is nominal (no stiffness separation), so it pays the "
        "splitting cost with none of the sub-stepping benefit it exists for (see "
        "`phase-0.5` N4)."
    )
    out.append("")
    sizes = sorted(stock_rows)
    rows1 = [
        [f"{n:,}", *[f"{stock_rows[n][s].sps:,.0f}" for s in schemes]] for n in sizes
    ]
    out.extend(_table(["N (stocks)", *[f"{s} (steps/s)" for s in schemes]], rows1))
    out.append("")

    # Table 2 — peak heap vs stock count.
    out.append("## Stock-count scaling — peak traced Python heap (KiB)")
    out.append("")
    out.append(
        "Memory scales ~O(`N`) (the stock dict, flow objects, and a few transient "
        "`State` snapshots); it does **not** grow with step count (old snapshots are "
        "GC'd, so the high-water mark is reached within a few steps)."
    )
    out.append("")
    rows2 = [
        [f"{n:,}", *[f"{stock_rows[n][s].kib:,.1f}" for s in schemes]] for n in sizes
    ]
    out.extend(_table(["N (stocks)", *[f"{s} (KiB)" for s in schemes]], rows2))
    out.append("")

    # Table 3 — domain count (orthogonal).
    out.append(f"## Domain-count scaling — throughput at fixed N = {domain_n:,} (RK4)")
    out.append("")
    out.append(
        "Domain count is **orthogonal** to per-step cost: the hot loop never iterates "
        "domains — the conservation ledger partitions by `StockKind` and `Quantity` "
        "(not domain), and `Registry.domain_index` is O(stocks) regardless of how many "
        "domains those stocks span. So throughput here is flat (within measurement "
        "noise); domain count affects only one-time construction, not the step."
    )
    out.append("")
    rows3 = [[f"{d:,}", f"{domain_rows[d]:,.0f}"] for d in sorted(domain_rows)]
    out.extend(_table(["D (domains)", "RK4 (steps/s)"], rows3))
    out.append("")
    return "\n".join(out)


# --- entry point ------------------------------------------------------------
def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out", metavar="PATH", help="write the markdown report to PATH (else stdout)"
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="fast subset (smaller sweep, shorter timing) for iteration",
    )
    args = parser.parse_args(argv)

    if args.quick:
        sizes = [2, 8, 32]
        domain_n, domain_counts = 32, [1, 2, 8, 32]
        target_s, repeats, mem_steps = 0.05, 2, 20
    else:
        sizes = [2, 8, 32, 128, 512]
        domain_n, domain_counts = 128, [1, 2, 4, 8, 32, 128]
        target_s, repeats, mem_steps = 0.15, 3, 50

    print("stock-count sweep (single domain):", file=sys.stderr)
    stock_rows = stock_sweep(
        sizes, target_s=target_s, repeats=repeats, mem_steps=mem_steps
    )
    print(f"domain-count sweep (N={domain_n}, RK4):", file=sys.stderr)
    domain_rows = domain_sweep(
        domain_n, domain_counts, target_s=target_s, repeats=repeats
    )

    report = render_report(
        stock_rows,
        domain_n,
        domain_rows,
        target_s=target_s,
        repeats=repeats,
        mem_steps=mem_steps,
    )
    if args.out:
        with open(args.out, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(report + "\n")
        print(f"wrote {args.out}", file=sys.stderr)
    else:
        # Bypass the console's locale codec (Windows cp1252 cannot encode the report's
        # `·`/`×`/`⇌`); the report is UTF-8 by construction.
        sys.stdout.buffer.write((report + "\n").encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
