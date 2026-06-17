"""``lab`` — the project's out-of-core "laboratory": validation / analysis tooling.

This package sits **outside** ``simcore`` and is deliberately exempt from the core's
determinism / integer-clock (``t = n*dt``, decision #14) invariants. It hosts things
that analyse or validate the engine but are not part of the shipped deterministic
core: the convergence-order fitter (Phase 0.5 Step 1) and, later, the adaptive RK45
reference oracle (Step 2, which carries its own float ``t`` and so cannot live in the
core). See ``docs/plans/phase-0.5-numerical-foundations.md`` (decisions N1/N2).
"""
