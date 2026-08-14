"""The biosphere's integration step — one place, so it is one edit.

Before 2026-08-14 the step was the literal ``1.0`` written at ~75 call sites, and the
season length was the literal ``len(weather)``. That is fine while the step *is* a day —
the two numbers coincide — and it silently conflates them the moment it is not. This
module separates them: :data:`BIO_DT` is the integration step, :data:`STEPS_PER_DAY` is
how many of those fit in a physical day, and callers ask for **days** and get steps.

**Why the step is moving off a day (the unfreeze of 2026-08-14).** In a sealed chamber
the crop draws the air's CO₂ down over the season, and below the CO₂ compensation point
(``Γ*/ci_ratio = 61.07 ppm``) assimilation is exactly zero — a hard floor. At a one-day
step the model drove the sealed and perennial chambers *below* that floor (57.89 and
56.03 ppm) and kept fixing carbon anyway. It is a **truncation error, not a threshold
crossing**: the observable converges as the step shrinks (57.89 → 75.06 → 75.82 → 76.03
against an RK4 limit of 76.29), which is a statement about the answer rather than about
the guard. Measured in `docs/plans/post-roadmap-step-sweep.md`; the ceremony is
`docs/plans/post-roadmap-step-unfreeze.md`; the frozen contract is
`docs/biosphere-reference.md`.

⚠ **Changing anything here is an unfreeze**, not a tuning knob — it moves every
biosphere golden and the cross-port tier bands with them. Follow the discipline in
`docs/biosphere-reference.md`.

⚠ **The step and the weather table are coupled.** ``season._table`` indexes
``int(n · dt)``, so the weather list stays **one row per physical day** at any step and
must never be tiled to match ``STEPS_PER_DAY``. Tiling it would double-count the
refinement.

Pure stdlib — this is domain-side data, and ``simcore`` neither imports nor knows it.
"""

from __future__ import annotations

# The integration step, in days.
#
# ``¼`` over ``½`` is the user's call, taken 2026-08-14: both clear the compensation
# point everywhere measured, but ``¼`` leaves 4.8× of headroom to the arbitration bound
# where ``½`` leaves 2.1×, so the next mechanism added to the tree probably does not
# force a second ceremony.
#
# The routing and the physical-time weather indexing landed FIRST, at the old step, and
# were proved byte-identical there. So every byte of this change's golden diff is
# attributable to the step alone — which is the whole reason for the two-commit split.
BIO_DT: float = 0.25

# How many integration steps make one physical day. Kept as an ``int`` (not derived by
# ``1 / BIO_DT``) because it indexes lists and sizes loops, and because the pair being
# stated twice makes the invariant below checkable rather than assumed.
STEPS_PER_DAY: int = 4

# The two must agree, and ``BIO_DT`` must be a negative power of two so that ``n · dt``
# is exact in binary and ``season._table``'s ``int()`` truncation has no round-off edge.
assert STEPS_PER_DAY * BIO_DT == 1.0, "BIO_DT and STEPS_PER_DAY disagree"
assert STEPS_PER_DAY & (STEPS_PER_DAY - 1) == 0, "STEPS_PER_DAY must be a power of two"


def steps_for(days: int) -> int:
    """Integration steps in ``days`` physical days — the call sites' unit conversion.

    Use this anywhere a run length, a reset period or a perturbation window is
    expressed: ``run_perennial(..., steps=steps_for(len(weather)),
    year=steps_for(len(one_season)))``. The point is that the call reads in **days**,
    which is what the scenario means, and stays correct when the step moves.

    ⚠ **Do not rename this back to ``steps``.** It was ``steps`` for one afternoon on
    2026-08-14 and the routing pass went red in 57 places: ``steps`` is the most natural
    local name in this suite (``steps = len(weather)``, ``def _run(..., steps: int =
    ...)``), so at every such site the local shadowed the import and the call raised
    ``'int' object is not callable``. The collision was loud *there* because those lines
    run on every suite pass — but a shadowed call on a rarely-taken branch would not be,
    so the fix is the un-collidable name, not vigilance.
    """
    return days * STEPS_PER_DAY


def day_of(n: int) -> int:
    """The physical day a step index falls in — the inverse of :func:`steps_for`.

    For indexing a trajectory by day rather than by step: ``states[steps_for(d)]`` is
    the state at the start of day ``d``, and ``day_of(n)`` says which day step ``n``
    is in.
    Matches ``season._table``'s ``int(n · dt)`` exactly, by construction.
    """
    return n // STEPS_PER_DAY
