"""The non-conserved auxiliary channel (Phase-1 P2): scalar accumulators.

Some simulation state *evolves* but is **not** a conserved quantity, so it cannot
ride the flow → reduce → apply path or pass the every-step conservation gate. The
canonical Phase-1 example is **thermal time** (°C·day): an integrator-advanced
scalar with no balanced counterparty. Development stage, leaf area, etc. are
**derived** from thermal time / leaf carbon, not stored — so the channel stays a
thin set of accumulators, not a general state soup.

An ``AuxProcess`` is the parallel of a ``Flow``: it ``evaluate``s against the
step-entry snapshot and returns **per-accumulator-name increments** in the same
*increment form* as ``Flow`` (``dt·rate(snapshot)``, not a bare rate). But unlike a
flow it is **single-valued and un-balanced** — there is no conserved counterparty,
so there is no balance check, and the integrator advances aux **outside** the
conservation gate (see ``simcore.conservation``: the ledger reasons only over
``State.stocks``, so aux is invisible to it by construction).

**Numerics (P2/P3).** Aux is advanced by **one explicit-Euler evaluation at the
step-entry snapshot**, independent of the stock integrator's scheme, and is **never
sub-staged through RK4**: RK4 stage states keep aux unchanged (only stock amounts
perturb), exactly as they keep the integer ``n`` — so a flow that *reads* aux sees a
within-step constant. The reduction over processes is in **canonical id order**
(``AuxId``-sorted), summing per accumulator name (decision #15): two processes may
contribute to the same name, and that cross-process float sum must be deterministic.

**Module name.** This is the "aux channel" the plan names, but the file is
``auxiliary.py`` rather than ``aux.py`` because ``AUX`` is a reserved Windows device
name — ``aux.py`` cannot be committed/cloned on Windows (git's Win32 file APIs
resolve it to the device), and an eventual Rust ``aux.rs`` would hit the same wall.
The Python identifiers (``AuxProcess``, ``AuxId``, the ``State.aux`` field) are
unaffected — only the filename is reserved.

Pure stdlib only.
"""

from collections.abc import Mapping
from typing import NewType, Protocol, runtime_checkable

from simcore.environment import Environment
from simcore.state import State

# A stable, canonical-sortable identifier for an aux *process* (distinct from the
# accumulator *names* it writes). Plain string at runtime; distinct under the
# checker — mirrors ``FlowId``. Keep ASCII so Python's str sort matches the future
# Rust UTF-8 byte sort (cross-port determinism).
AuxId = NewType("AuxId", str)


@runtime_checkable
class AuxProcess(Protocol):
    """A pure, deterministic rate for one or more non-conserved accumulators.

    ``evaluate`` reads the snapshot/env only, never mutates, and is deterministic
    in its inputs (mirroring ``Flow.evaluate``). It returns a mapping from
    accumulator **name** to its **per-step increment** ``dt·rate(snapshot)`` — the
    same increment-form contract as ``Flow`` (so ``rate`` must be independent of
    ``dt``; a process that uses ``dt`` non-linearly still accumulates but its
    "rate" is no longer a clean per-time quantity). There is **no balance check**:
    aux is non-conserved by definition.

    ``id`` is **read-only** (a property) so frozen process implementations — the
    expected immutable shape — satisfy the protocol. It identifies the *process*
    for registration dedup and canonical iteration order; it is **not** an
    accumulator name (a process may write several names, and several processes may
    write one shared name — the integrator sums those contributions).
    """

    @property
    def id(self) -> AuxId: ...

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]: ...
