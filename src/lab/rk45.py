"""Adaptive Dormand–Prince RK45 — the out-of-core validation oracle (Phase 0.5 Step 2).

*Realizes "Adaptive RK45 (lab only)" per decision N1.* This is **not** a
``simcore.Integrator`` and deliberately lives **outside** ``simcore``: error-controlled
adaptive stepping means a **variable ``dt``**, which breaks Phase-0 decision #14
(``t = n·dt``, integer step count). It does *not* break determinism — an adaptive step
is still a deterministic function of state — but because it cannot honor the integer
clock it is excluded from the integer-clock / bit-identical-across-ports contract and
is **not** added to the determinism gates. Its role is to generate a high-accuracy
reference trajectory for the convergence study (analogous to PCSE as an offline oracle,
never the shipped engine).

**Reuses the core's flow machinery (N1).** A ``Flow.evaluate(snapshot, env, dt)``
returns the *per-step increment* ``dt·rate(snapshot)`` (the increment-form contract,
``simcore.flow``). RK45 is a derivative method, so it recovers the bare derivative
``rate = leg / dt`` by evaluating each flow at ``dt = _RATE_DT = 1.0`` — at that
``dt`` the legs *are* the rates, with no division round-off.

**Load-bearing assumption (same one RK4 leans on).** ``rate = leg / dt`` is exact
**only if** ``rate`` is independent of ``dt`` (the legs scale linearly with ``dt``).
A flow that uses ``dt`` non-linearly still conserves mass but would make the recovered
derivative ``dt``-dependent and silently corrupt this reference — see the
``Flow.evaluate`` dt-linearity note in ``simcore.flow`` and the integrator's
increment-form contract.

**Autonomous scenarios only (Phase 0.5).** Forcing is evaluated by binding the
resolver to the stage state, whose integer ``n`` is the template's fixed ``n`` — so a
*time-varying* forcing schedule would not be re-evaluated at the sub-stage times
``(n+cᵢ)·dt``. Sub-stage time-varying forcing is explicitly deferred (Phase-0 step-5
note, carried into Phase 0.5); the Phase-0.5 oracle scenarios are autonomous, so this
is exact for them. Constant forcing and shared-stock coupling work unchanged.

**No FSAL, no arbitration/extinction.** All seven stages are evaluated every step
(FSAL's one-eval saving is not worth the accept/reject state-management bug surface in
a reference oracle). The Euler-only min-scaling backstop and the extinction pass are
*not* run: like RK4, positivity here is the kinetics' job, and the oracle's scenarios
are well-fed (non-arbitrating). Conservation is **not** asserted internally — it is
``dt``-independent and verified at the call site against rebuilt states via
``simcore.conservation.assert_conserved`` (see ``Trajectory.state_at``).

Pure stdlib (``math`` only) — ``lab`` carries none of the core's invariants, but there
is no reason to pull in a third-party dep for a hand-rolled RK45.
"""

import math
from collections.abc import Mapping
from dataclasses import dataclass, replace

from simcore.environment import SourceResolver
from simcore.ids import StockId
from simcore.registry import Registry
from simcore.state import State

# --- Dormand–Prince (DOPRI5) Butcher tableau --------------------------------
# The embedded 4(5) pair scipy's RK45 uses. ``_C`` are the stage nodes; ``_A`` is the
# strictly-lower-triangular stage-coefficient matrix (row ``i`` has ``i`` entries, for
# the ``i``-th stage ``yᵢ = y + dt·Σ_{j<i} A[i][j]·kⱼ``); ``_B`` are the 5th-order
# solution weights and ``_B_STAR`` the embedded 4th-order weights. ``_E = _B − _B_STAR``
# weights the per-step error estimate directly (so the estimate never differences two
# large nearby vectors ``y5 − y4``). Consistency (each ``Σ A[i] == C[i]``, ``Σ B == 1``,
# ``Σ B_STAR == 1``) is the discriminating control the tableau test pins — a transcribed
# coefficient is this module's likeliest defect.
_C: tuple[float, ...] = (0.0, 1 / 5, 3 / 10, 4 / 5, 8 / 9, 1.0, 1.0)
_A: tuple[tuple[float, ...], ...] = (
    (),
    (1 / 5,),
    (3 / 40, 9 / 40),
    (44 / 45, -56 / 15, 32 / 9),
    (19372 / 6561, -25360 / 2187, 64448 / 6561, -212 / 729),
    (9017 / 3168, -355 / 33, 46732 / 5247, 49 / 176, -5103 / 18656),
    (35 / 384, 0.0, 500 / 1113, 125 / 192, -2187 / 6784, 11 / 84),
)
_B: tuple[float, ...] = (
    35 / 384,
    0.0,
    500 / 1113,
    125 / 192,
    -2187 / 6784,
    11 / 84,
    0.0,
)
_B_STAR: tuple[float, ...] = (
    5179 / 57600,
    0.0,
    7571 / 16695,
    393 / 640,
    -92097 / 339200,
    187 / 2100,
    1 / 40,
)
_E: tuple[float, ...] = tuple(b - bs for b, bs in zip(_B, _B_STAR, strict=True))
_N_STAGES = 7

# The ``dt`` at which flows are evaluated to recover the bare derivative ``rate``.
# 1.0 makes ``leg / _RATE_DT == leg == rate`` exactly (no division round-off), valid
# under the dt-linearity contract (see the module docstring).
_RATE_DT = 1.0

# --- adaptive step-size controller constants (the scipy RK45 defaults) -------
# The embedded error estimate is O(dt⁵), and we advance the 5th-order solution (local
# extrapolation); the standard step exponent for this 4(5) pair is −1/(4+1) = −0.2.
_ERR_EXPONENT = -0.2
_SAFETY = 0.9
_MIN_FACTOR = 0.2
_MAX_FACTOR = 10.0
# A step whose adaptive ``dt`` shrinks below this (relative to the horizon) means the
# controller cannot meet the tolerance — a stiffness signal, and stiff/implicit solvers
# are deferred (the phase plan's reconciliation). Surface it loudly rather than spin.
_MIN_STEP_REL = 1e-13


@dataclass(frozen=True)
class Trajectory:
    """A sampled adaptive RK45 reference trajectory (one sample per accepted step).

    ``stock_ids`` is the canonical (sorted) stock layout. ``times`` are the accepted
    time points — ``times[0] == 0.0`` (the initial state) through ``times[-1] ≈ t_end``
    (the final step is clipped to land on the horizon). ``samples[i]`` is the amount of
    each stock at ``times[i]``. ``n_accepted`` / ``n_rejected`` are the controller's
    accepted and rejected step counts.

    The carried ``initial`` ``State`` is the template ``state_at`` rebuilds reference
    states from. **Its ``n`` is *not* a simulation step count** — RK45 breaks the
    integer clock by design (N1); ``state_at`` stamps the *sample index* as ``n`` purely
    so the rebuilt ``State`` is well-formed (the conservation gate ignores ``n``).
    """

    stock_ids: tuple[StockId, ...]
    times: tuple[float, ...]
    samples: tuple[Mapping[StockId, float], ...]
    n_accepted: int
    n_rejected: int
    initial: State

    def step_sizes(self) -> list[float]:
        """The accepted step sizes ``times[i+1] − times[i]`` (the adaptivity record)."""
        return [b - a for a, b in zip(self.times[:-1], self.times[1:], strict=True)]

    def final(self) -> dict[StockId, float]:
        """The endpoint amounts (the convergence-reference value)."""
        return dict(self.samples[-1])

    def state_at(self, i: int) -> State:
        """Rebuild the reference ``State`` at sample ``i`` (for a conservation check).

        Amounts come from ``samples[i]``; every other field (kind/quantity/unit/domain)
        from ``initial``. ``n`` is set to the (non-negative) sample index — a label, not
        a step count (see the class note). Used with
        ``simcore.conservation.assert_conserved`` to verify the oracle conserves mass
        (``dt``-independent), without coupling that assertion into the integrator.
        """
        idx = range(len(self.samples))[i]  # normalize negatives; IndexError if OOB
        amounts = self.samples[idx]
        stocks = {
            sid: replace(stock, amount=amounts[sid])
            for sid, stock in self.initial.stocks.items()
        }
        return replace(self.initial, n=idx, stocks=stocks)


def _with_amounts(template: State, amounts: Mapping[StockId, float]) -> State:
    """A stage ``State``: ``template`` with each stock's amount set from ``amounts``.

    ``Stock.__post_init__`` rejects a non-finite amount, so a diverging integration
    surfaces as a ``ValueError`` here rather than poisoning the reference with ``Inf``.
    """
    stocks = {
        sid: replace(stock, amount=amounts[sid])
        for sid, stock in template.stocks.items()
    }
    return replace(template, stocks=stocks)


def _derivative(
    registry: Registry,
    template: State,
    env: SourceResolver,
    amounts: Mapping[StockId, float],
) -> dict[StockId, float]:
    """The bare derivative ``dy/dt`` per stock at the state ``amounts``.

    Builds the stage state, binds the resolver to it (the #16 seam — same snapshot the
    flows read), evaluates every flow in canonical id-order, and reduces legs per stock
    recovering ``rate = leg / _RATE_DT``. Stocks untouched by any flow are simply absent
    (a trivial 0 derivative). Reductions run in canonical flow/leg order — not required
    for the oracle (it is exempt from bit-identical-across-ports) but free and on-brand.
    """
    stage = _with_amounts(template, amounts)
    bound = env.bind(stage, _RATE_DT)
    deriv: dict[StockId, float] = {}
    for flow in registry.flows:
        result = flow.evaluate(stage, bound, _RATE_DT)
        for leg in result.legs:
            deriv[leg.stock] = deriv.get(leg.stock, 0.0) + leg.amount / _RATE_DT
    return deriv


def _rk45_step(
    registry: Registry,
    template: State,
    env: SourceResolver,
    y: Mapping[StockId, float],
    dt: float,
) -> tuple[dict[StockId, float], dict[StockId, float]]:
    """One Dormand–Prince step from ``y`` over ``dt``.

    Returns ``(y_new, error)``: ``y_new`` is the 5th-order advance
    ``y + dt·Σ Bⱼ·kⱼ`` and ``error`` is the embedded estimate ``dt·Σ Eⱼ·kⱼ``
    (``E = B − B_STAR``), computed directly rather than as a difference of two
    near-equal vectors. Factored out so the embedded estimate's **5th order** can be
    pinned on synthetic single-step data (the tableau's discriminating control, the
    Step-1 ``fit_order`` analogue) independently of the adaptive driver.
    """
    stages: list[dict[StockId, float]] = [_derivative(registry, template, env, y)]
    for i in range(1, _N_STAGES):
        row = _A[i]
        yi = {
            sid: amt + dt * sum(row[j] * stages[j].get(sid, 0.0) for j in range(i))
            for sid, amt in y.items()
        }
        stages.append(_derivative(registry, template, env, yi))
    y_new = {
        sid: amt + dt * sum(_B[j] * stages[j].get(sid, 0.0) for j in range(_N_STAGES))
        for sid, amt in y.items()
    }
    error = {
        sid: dt * sum(_E[j] * stages[j].get(sid, 0.0) for j in range(_N_STAGES))
        for sid in y
    }
    return y_new, error


def _error_norm(
    y: Mapping[StockId, float],
    y_new: Mapping[StockId, float],
    error: Mapping[StockId, float],
    atol: float,
    rtol: float,
) -> float:
    """The RMS error norm ``sqrt(mean((errᵢ / scaleᵢ)²))`` (Hairer & Wanner).

    ``scaleᵢ = atol + rtol·max(|yᵢ|, |y_newᵢ|)`` is strictly positive (``atol > 0`` is
    required), so the norm cannot divide by zero. ``≤ 1`` ⇒ the step meets tolerance.
    """
    total = 0.0
    count = 0
    for sid, e in error.items():
        scale = atol + rtol * max(abs(y[sid]), abs(y_new[sid]))
        total += (e / scale) ** 2
        count += 1
    return math.sqrt(total / count) if count else 0.0


def _next_dt(dt: float, err_norm: float) -> float:
    """The controller's next step size from the current ``err_norm`` (clamped)."""
    if err_norm == 0.0:
        return dt * _MAX_FACTOR
    factor = _SAFETY * err_norm**_ERR_EXPONENT
    return dt * min(_MAX_FACTOR, max(_MIN_FACTOR, factor))


def rk45_trajectory(
    registry: Registry,
    state0: State,
    env: SourceResolver,
    t_end: float,
    *,
    atol: float,
    rtol: float,
    dt0: float,
    max_steps: int = 1_000_000,
) -> Trajectory:
    """Integrate ``registry`` from ``state0`` to ``t_end`` with adaptive RK45.

    Carries its **own** float ``t`` and adaptive ``dt`` (N1 — outside the integer
    clock). Starts at ``dt0``, accepting a step when its RMS error norm is ``≤ 1`` and
    growing/shrinking ``dt`` from the estimate; the final step is clipped to land on
    ``t_end`` exactly. ``atol`` and ``rtol`` must both be ``> 0`` (so the error scale is
    positive); ``dt0`` and ``t_end`` must be ``> 0``.

    Raises ``RuntimeError`` if the controller cannot reach ``t_end`` within
    ``max_steps`` or its step size underflows (``< _MIN_STEP_REL·t_end`` — a stiffness
    signal; stiff/implicit integration is deferred). Returns the sampled
    ``Trajectory`` (one sample per accepted step, plus the initial state).
    """
    if t_end <= 0.0:
        raise ValueError(f"t_end must be > 0, got {t_end!r}")
    if atol <= 0.0 or rtol <= 0.0:
        raise ValueError(
            f"atol and rtol must both be > 0 (the error scale must be positive); "
            f"got atol={atol!r}, rtol={rtol!r}"
        )
    if dt0 <= 0.0:
        raise ValueError(f"dt0 must be > 0, got {dt0!r}")

    stock_ids = tuple(sorted(state0.stocks))
    y: dict[StockId, float] = {sid: state0.stocks[sid].amount for sid in stock_ids}
    t = 0.0
    dt = min(dt0, t_end)
    min_step = _MIN_STEP_REL * t_end

    times: list[float] = [0.0]
    samples: list[Mapping[StockId, float]] = [dict(y)]
    n_accepted = 0
    n_rejected = 0
    steps = 0

    # Float horizon guard: once t is within a relative ulp-band of t_end, treat it as
    # reached (the clip lands the last accepted step there to within rounding).
    close = max(min_step, 1e-12 * t_end)
    while t_end - t > close:
        if steps >= max_steps:
            raise RuntimeError(
                f"RK45 did not reach t_end={t_end!r} within max_steps={max_steps} "
                f"(reached t={t!r}); raise max_steps or loosen the tolerance"
            )
        steps += 1
        dt = min(dt, t_end - t)  # never overshoot; the final step lands on t_end
        y_new, error = _rk45_step(registry, state0, env, y, dt)
        err_norm = _error_norm(y, y_new, error, atol, rtol)
        if err_norm <= 1.0:
            t += dt
            y = y_new
            times.append(t)
            samples.append(dict(y))
            n_accepted += 1
        else:
            n_rejected += 1
        dt = _next_dt(dt, err_norm)
        if dt < min_step:
            raise RuntimeError(
                f"RK45 step size underflowed (dt={dt!r} < {min_step!r}) near t={t!r}; "
                "the scenario looks stiff — stiff/implicit integration is deferred "
                "(see docs/plans/phase-0.5-numerical-foundations.md)"
            )

    return Trajectory(
        stock_ids=stock_ids,
        times=tuple(times),
        samples=tuple(samples),
        n_accepted=n_accepted,
        n_rejected=n_rejected,
        initial=state0,
    )
