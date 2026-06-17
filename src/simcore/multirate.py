"""Multi-rate sub-stepping: operator splitting over disjoint flow sets (N2–N5).

The Phase-0.5 multi-rate driver. One **master** step of size ``dt`` advances the
integer clock by exactly 1 (``State.n -> n+1``); a *fast* flow set sub-steps
``n_sub`` times **inside** that master step while a *slow* flow set steps at the
master rate. This is the efficiency feature of the roadmap's "domains step at
different rates" — without paying the fast ``dt`` on the slow domain.

**Why this is pure core (and RK45 is not).** Multi-rate **preserves** decision #14
(time is an integer step count, ``t = n·dt``): the fast sub-steps are *internal* —
amounts are perturbed, ``n`` is unchanged — exactly how RK4 stage states already
keep ``n``. The driver only ever advances ``n`` by 1, once, at the master-step
commit. So it lives in pure-stdlib ``simcore`` (decision **N2**), unlike the
adaptive RK45 oracle (variable ``dt``, ``lab/``).

**The partition is per-flow, not per-domain (N3).** ``slow`` and ``fast`` are two
integrators over **disjoint flow registries that share one stock dict**. A
cross-domain flow has no single-domain rate, so the rate-class is a property of the
*flow*, assigned by the scenario assembler; the driver takes the two pre-built
integrators and does not infer the partition. The fast and slow sets meet only
through the shared stocks they read/write.

**Operator splitting; Strang by default (N4).** Within one master ``dt`` the slow
and fast operators are *sequenced*, not applied simultaneously. **Lie** splitting
(slow-full then fast-full) is globally 1st-order even when each operator is
sub-integrated with RK4 — the split caps the composite order. **Strang** (the
default) is the symmetric half-slow / full-fast / half-slow composition, 2nd-order
*provided both operators are RK4* (``min(splitting_order, sub-integrator orders)``;
a Euler operator silently collapses Strang back to 1st order — the exact
order-reduction this phase exists to catch). Strang's 2 is a hard ceiling: the
operators' non-commutativity is an O(``dt²``) term no sub-integration removes, so
multi-rate is an **efficiency** trade that *costs* accuracy versus single-rate RK4
(order 4). Higher-order multi-rate is deferred.

**Atomicity is per-operator (N5).** Phase-0 atomicity (#11 — no flow reads a stock
another flow mutated in the same step) holds **within** each sub-operation (all slow
flows see one snapshot; each fast sub-step sees its own), but **not across** the
slow/fast split: fast flows deliberately read slow-updated stocks. That coupling is
the point of multi-rate, and is priced as the O(``dt²``) (Strang) splitting error.

**Conservation (N4/N5).** Every sub-operation is internally balanced (a scaled
balanced delta is still balanced; extinction's loss-sink routing is a balanced
non-flow change), so the composite conserves exactly by linearity. The conservation
gate is asserted **once**, at the master-step boundary (``before`` = entry state,
``after`` = post-commit) — the belt-and-suspenders engine-bug tripwire. Sub-steps
(``Substepper.substep``) deliberately *skip* the per-operation conservation assert,
so an unbalanced injected sub-delta surfaces **here**, at the boundary.

**Determinism (#7/#15).** Canonical flow-id order holds within each registry;
``n_sub`` is fixed; the master step is bit-identical and registration-order
-independent within each registry.

Pure stdlib only.
"""

import enum
from dataclasses import replace

from simcore import conservation
from simcore.environment import SourceResolver
from simcore.events import Event
from simcore.integrator import StepReport, Substepper
from simcore.state import State


class Split(enum.Enum):
    """The operator-splitting scheme for one master step (decision N4).

    ``STRANG`` (default, gated path) — symmetric ``slow(dt/2)`` → ``fast`` →
    ``slow(dt/2)``; 2nd-order *iff both operators are RK4*. ``LIE`` (fallback /
    comparison) — ``slow(dt)`` → ``fast``; 1st-order regardless of sub-integrator.
    """

    STRANG = "strang"
    LIE = "lie"


def multirate_step(
    slow: Substepper,
    fast: Substepper,
    state: State,
    env: SourceResolver,
    dt: float,
    n_sub: int,
    *,
    split: Split = Split.STRANG,
) -> StepReport:
    """One master multi-rate step: advance ``State.n`` by exactly 1 (decisions N2–N5).

    ``slow`` and ``fast`` are integrators over **disjoint flow registries sharing one
    stock dict** (N3). The fast set sub-steps ``n_sub`` times at ``dt/n_sub`` inside
    the master step; the slow set steps at the master rate (split into two ``dt/2``
    halves under Strang). Every sub-operation is an amounts-only ``substep`` (``n``
    kept); the **single** ``n -> n+1`` commit and the **composite** conservation gate
    are owned here (sub-steps skip the per-operation assert, so an unbalanced
    sub-delta trips this boundary gate).

    Returns a ``StepReport`` like the single-rate ``step_report``: the produced
    ``State`` (``n -> n+1``), the aggregated extinction ``events`` (re-stamped to the
    produced ``n``, matching the ``ExtinctionEvent.n`` convention), and ``rationed``
    summed over all sub-operations (the Euler-backstop firing count; always 0 when
    both operators are RK4 — RK4 hard-errors on over-draw).

    ``n_sub`` must be ``>= 1`` (``ValueError`` otherwise). With ``n_sub == 1`` and an
    **empty** slow registry, a Strang master step reproduces the single-rate ``step``
    bit-for-bit (the slow halves are no-ops; the one full fast sub-step *is* the
    single-rate step).
    """
    if n_sub < 1:
        raise ValueError(f"n_sub must be >= 1, got {n_sub}")

    # A flat ordered op-list — the splitting sequence made self-documenting. Strang:
    # half-slow / n_sub × fast / half-slow. Lie: full-slow / n_sub × fast. Each entry
    # is one amounts-only sub-operation at its own step size.
    fast_ops = [(fast, dt / n_sub)] * n_sub
    if split is Split.STRANG:
        ops = [(slow, dt / 2.0), *fast_ops, (slow, dt / 2.0)]
    elif split is Split.LIE:
        ops = [(slow, dt), *fast_ops]
    else:  # defensive: a future Split member must be wired above, not silently run
        raise ValueError(f"unknown split scheme {split!r}")

    before = state
    events: list[Event] = []
    rationed = 0
    cur = state
    for stepper, h in ops:
        report = stepper.substep(cur, env, h)
        cur = report.state
        events.extend(report.events)
        rationed += report.rationed

    # The single master-step commit (N2): n -> n+1 once, over the post-split amounts.
    committed = replace(cur, n=before.n + 1)
    # The composite conservation gate (N4/N5): asserted once, here, over the whole
    # master step. Sub-steps skipped it, so this is the load-bearing tripwire.
    conservation.assert_conserved(before, committed)
    # Re-stamp events to the produced state's n: sub-steps keep n at before.n, but an
    # event belongs to the master step the driver is producing (ExtinctionEvent.n is
    # "the n of the State the integrator is producing").
    stamped = tuple(replace(event, n=committed.n) for event in events)
    return StepReport(state=committed, events=stamped, rationed=rationed)
