"""The Environment source resolver: the Protocol plus its concrete backends.

Step 3 built **only** the ``Environment`` Protocol (``get(var) -> float``) — the
interface ``Flow.evaluate`` needs in its signature. Step 5 adds the concrete
backends behind it, without touching the frozen interface.

A flow calls ``env.get(var)`` and **cannot tell** whether the value came from a
forcing schedule (evaluated at ``t = n*dt``, integer ``n`` — decision #14) or a
sibling domain's shared stock (read from the *same* immutable snapshot the flows
read — decision #16). That indistinguishability is the point: identical domain
code runs both standalone (forcing) and coupled (shared stock).

The binding model (the crux): ``get(var)`` takes only ``var``, so all per-step
context — *which* snapshot, *which* ``dt`` — is **bound into** the object first.
``SourceResolver`` holds the build-once wiring (two disjoint var maps); its
``bind(snapshot, dt)`` returns a lightweight ``BoundEnvironment`` for one
derivative evaluation. The integrator (step 6) rebinds per evaluation (Euler:
once; RK4: per stage) and **must bind to the same snapshot it passes to
``flow.evaluate``** — that is the mechanism that makes #16 hold.

Pure stdlib only.
"""

import math
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from types import MappingProxyType
from typing import Protocol, runtime_checkable

from simcore.ids import StockId
from simcore.state import State


@runtime_checkable
class Environment(Protocol):
    """Resolves an environment variable name to a scalar value.

    The caller cannot distinguish a forcing-schedule source from a shared-stock
    source — both are just ``get(var) -> float`` (decision #16).
    """

    def get(self, var: str) -> float: ...


# A forcing schedule: a pure function of the integer step ``n`` and ``dt``.
# Passing the integer ``n`` *and* ``dt`` (rather than a precomputed float ``t``)
# keeps the integer visible (#14), so both step-index schedules ("every k steps")
# and wall-time schedules (evaluated at ``t = n*dt``) are expressible drift-free.
Schedule = Callable[[int, float], float]


def constant(value: float) -> Schedule:
    """A forcing ``Schedule`` that returns ``value`` at every step.

    Covers the Phase-0 demo's constant forcing (solar, fixed boundary levels).
    ``value`` is validated finite *here*, at wiring time, so a bad constant fails
    loudly at construction rather than at the first ``get``.
    """
    if not math.isfinite(value):
        raise ValueError(f"constant forcing value is not finite: {value!r}")

    def schedule(n: int, dt: float) -> float:
        return value

    return schedule


class SourceResolver:
    """Build-once env wiring: ``var -> forcing schedule`` OR ``var -> shared stock``.

    Mirrors ``Registry``: immutable structural config (the two var maps are wrapped
    in ``MappingProxyType``). The two namespaces are **disjoint** — a var is forcing
    *xor* shared; an overlap is a wiring bug rejected at construction (a structural
    property of the wiring itself, knowable at build, independent of any stock set).

    Referential integrity is *resolve-time*: a ``shared`` var pointing at a missing
    stock, or an unknown ``var``, surfaces as a ``KeyError`` at the first ``get``
    (n = 0 — loud and immediate, never a deep-run time bomb), not a construction
    check. (A build-time target check is one set-difference if step 10 ever wants
    belt-and-suspenders; it is omitted here so the resolver stays pure wiring.)
    """

    def __init__(
        self,
        forcings: Mapping[str, Schedule] | None = None,
        shared: Mapping[str, StockId] | None = None,
    ) -> None:
        forcing_map = dict(forcings or {})
        shared_map = dict(shared or {})
        overlap = forcing_map.keys() & shared_map.keys()
        if overlap:
            raise ValueError(
                "env var(s) wired as both forcing and shared stock: "
                f"{sorted(overlap)!r} (a var is forcing xor shared, decision #16)"
            )
        self._forcings: Mapping[str, Schedule] = MappingProxyType(forcing_map)
        self._shared: Mapping[str, StockId] = MappingProxyType(shared_map)

    @property
    def forcings(self) -> Mapping[str, Schedule]:
        """Read-only ``var -> Schedule`` forcing wiring."""
        return self._forcings

    @property
    def shared(self) -> Mapping[str, StockId]:
        """Read-only ``var -> StockId`` shared-stock wiring."""
        return self._shared

    def bind(self, snapshot: State, dt: float) -> "BoundEnvironment":
        """Bind to one snapshot + ``dt`` for a single derivative evaluation.

        The integrator must pass the **same** ``snapshot`` to ``flow.evaluate`` so
        a flow's direct reads and its ``env.get`` shared reads stay consistent
        (#16). The bound view is lightweight — it holds references, copies nothing.
        """
        return BoundEnvironment(self, snapshot, dt)


@dataclass(frozen=True, eq=False)
class BoundEnvironment:
    """An ``Environment`` bound to one snapshot + ``dt`` for a single evaluation.

    Resolves a forcing var via its schedule at the bound snapshot's integer ``n``
    (``t = n*dt``, #14), and a shared var by reading the bound snapshot's stock
    amount (#16 — the same immutable snapshot the flows read). Both branches draw
    on the *one* bound snapshot, so forcing-time and shared reads are mutually
    consistent by construction. The caller cannot tell which branch answered.
    """

    resolver: SourceResolver
    snapshot: State
    dt: float

    def get(self, var: str) -> float:
        forcings = self.resolver.forcings
        if var in forcings:
            value = forcings[var](self.snapshot.n, self.dt)
            # Stock amounts are already finite (Stock.__post_init__); only a
            # forcing schedule can introduce NaN/Inf, so guard it here rather than
            # let it poison a downstream leg.
            if not math.isfinite(value):
                raise ValueError(
                    f"forcing schedule for env var {var!r} returned non-finite "
                    f"value: {value!r}"
                )
            return value
        shared = self.resolver.shared
        if var in shared:
            # Reads the bound snapshot (#16). A KeyError here means the shared var
            # points at a missing stock — referential integrity, resolve-time by
            # design (consistent with the flow-leg apply path).
            return self.snapshot.stocks[shared[var]].amount
        raise KeyError(
            f"unknown env var {var!r} (wired as neither forcing nor shared stock)"
        )
