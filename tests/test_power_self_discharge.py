"""Phase-5 P5.5: the standalone Power self-discharge validation (energy closure).

Step 5 adds the Power domain's **first donor-controlled flow** — ``SelfDischarge``
(``battery → waste_heat``, ``leak = k·battery·dt``), the first-order standing shelf-loss
— as an **opt-in third flow** of ``build_power`` (default off, so ``BOUNDED_SOC``'s
golden and RK4 ≡ Euler bit-identity are untouched — see ``test_power_run.py`` /
``test_regression_power.py``). It reuses ``BOUNDED_SOC_SCENARIO`` **verbatim** (the
daily-balanced forced flows) so the leak is the *sole* driver of any departure from the
balanced baseline, over ``SELF_DISCHARGE_DAYS``.

**Why it earns its keep (the plan gates SelfDischarge on this).** The two existing Power
flows are *forced* (state-independent): the SOC has no restoring force / no attractor —
its boundedness is *constructed* by exact daily balance (``test_power_run.py``).
``SelfDischarge`` is the first flow that reads a **stock** (``leak ∝ battery``), so it
brings three genuinely new, non-vacuous properties:

* **The contraction test — the rigorous, magnitude-independent proof of the restoring
  force.** Two runs differing *only* in ``battery0`` with identical forcing: the forced
  terms cancel in the difference, so under Euler ``d_n ≡ B_A(n) − B_B(n) = d_0·(1 −
  k·dt)^n`` — an **exact geometric contraction** that even measures ``k`` back out. A
  forced-only system keeps ``d_n`` *constant* (no decay), so this cleanly
  *distinguishes* the donor-controlled flow from the forced ones. This holds at the
  realistic Li-ion rate (~2.6 %/month) where the SOC barely bends — the property is
  proved by the algebra, not by a visible convergence (no rate inflation; see
  ``params/self_discharge.yaml``).
* **``rationed == 0`` from kinetics, for *this* leg.** ``leak ∝ battery`` self-limits to
  0 as the battery empties (``k·dt < 1`` structural). *LoadDraw* is still forced, so the
  run's overall ``rationed == 0`` still leans on well-fed sizing — self-discharge does
  not change that; it makes only *its own* draw structural.
* **RK4 ≢ Euler.** The forced-only bit-identity (``k1 = k2 = k3 = k4``) is broken by the
  state-dependent leak, so the integrator cross-check becomes a real **tolerance
  agreement** (O(dt²) apart), not the algebraic identity ``BOUNDED_SOC`` had.

And the closure payload is unchanged: ENERGY is conserved **every step** over the
augmented ledger (the leak is a balanced 2-leg transfer, ``−leak + leak = 0``), and
``waste_heat`` stays **monotonic** (it now also receives the leak).

Pure-stdlib spine; both params load from the committed ``charge.yaml`` /
``self_discharge.yaml``.
"""

from dataclasses import replace

import pytest

from domains.power.loader import load_charge_params, load_self_discharge_params
from domains.power.scenario import (
    BOUNDED_SOC_SCENARIO,
    SELF_DISCHARGE_DAYS,
    PowerScenario,
)
from domains.power.stocks import BATTERY, SOLAR_SOURCE, WASTE_HEAT
from domains.power.system import build_power, power_resolver, run_power
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.state import State

_CHARGE = load_charge_params()
_SELF_DISCHARGE = load_self_discharge_params()
_SCENARIO = BOUNDED_SOC_SCENARIO
_STEPS = SELF_DISCHARGE_DAYS * _SCENARIO.steps_per_day
_DT = _SCENARIO.dt_seconds
_KDT = _SELF_DISCHARGE.self_discharge_rate * _DT  # the per-step contraction factor arg


def _run(
    scenario: PowerScenario = _SCENARIO,
    *,
    self_discharge: bool = True,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
) -> tuple[list[State], int, tuple]:
    sd = _SELF_DISCHARGE if self_discharge else None
    state, registry = build_power(_CHARGE, scenario, sd)
    resolver = power_resolver(_CHARGE, scenario)
    steps = SELF_DISCHARGE_DAYS * scenario.steps_per_day
    return run_power(
        integrator_cls(registry), state, resolver, scenario.dt_seconds, steps
    )


def _soc(states: list[State]) -> list[float]:
    return [s.stocks[BATTERY].amount for s in states]


@pytest.fixture(scope="module")
def leaky() -> tuple[list[State], int, tuple]:
    return _run()


# --- k·dt < 1 : the structural-positivity precondition -------------------------------
def test_self_discharge_step_factor_below_one() -> None:
    # The donor-controlled draw self-limits only while k·dt < 1 (the herbivory rate·dt<1
    # discipline). At the realistic rate this is ~3.6e-5 — deeply structural.
    assert 0.0 < _KDT < 1.0


# --- THE keep-earning proof: the exact geometric contraction -------------------------
def test_self_discharge_contracts_geometrically() -> None:
    # Two runs differing ONLY in battery0, identical forcing. The forced terms cancel in
    # the difference, so d_n = d_0·(1 − k·dt)^n EXACTLY (to fp round-off) under Euler —
    # the restoring force made visible, forcing-independent, measuring k back out. This
    # is the rigorous "earns its keep" proof at the realistic rate (SOC barely bends).
    other = replace(_SCENARIO, battery0=_SCENARIO.battery0 + 1.0e6)
    a, _, _ = _run()
    b, _, _ = _run(other)
    soc_a, soc_b = _soc(a), _soc(b)
    d0 = soc_b[0] - soc_a[0]
    assert d0 == pytest.approx(1.0e6)
    for n in range(_STEPS + 1):
        predicted = d0 * (1.0 - _KDT) ** n
        assert soc_b[n] - soc_a[n] == pytest.approx(predicted, rel=1e-9)
    # And the difference genuinely SHRANK (contraction, not a flat line): the restoring
    # force pulled the two trajectories together.
    assert abs(soc_b[-1] - soc_a[-1]) < abs(d0)


def test_forced_only_difference_is_constant() -> None:
    # The contrast that makes the contraction meaningful: with self-discharge OFF, the
    # two forced flows have NO restoring force, so the battery0 offset propagates
    # undecayed — d_n == d_0 for every n (bit-identical forced deltas). Donor-control is
    # exactly what turns this constant into a geometric contraction.
    other = replace(_SCENARIO, battery0=_SCENARIO.battery0 + 1.0e6)
    a, _, _ = _run(self_discharge=False)
    b, _, _ = _run(other, self_discharge=False)
    soc_a, soc_b = _soc(a), _soc(b)
    for n in range(_STEPS + 1):
        assert soc_b[n] - soc_a[n] == pytest.approx(1.0e6, rel=0.0, abs=1e-3)


# --- isolation: the leak is the sole driver of departure from the balanced baseline ---
def test_self_discharge_departs_the_balanced_baseline(
    leaky: tuple[list[State], int, tuple],
) -> None:
    # The forced part is daily-balanced (load_fraction=1), so WITHOUT the leak the SOC
    # returns to battery0 at each day boundary. WITH the leak it monotonically decays
    # below battery0 — the leak isolated as the sole conservation-preserving drift.
    leaky_states, _, _ = leaky
    balanced, _, _ = _run(self_discharge=False)
    spd = _SCENARIO.steps_per_day
    b0 = _SCENARIO.battery0
    day_soc = [
        leaky_states[d * spd].stocks[BATTERY].amount
        for d in range(SELF_DISCHARGE_DAYS + 1)
    ]
    # baseline returns to battery0 each day; leaky run strictly decays below it.
    for d in range(SELF_DISCHARGE_DAYS + 1):
        assert balanced[d * spd].stocks[BATTERY].amount == pytest.approx(b0, abs=1e-6)
    assert all(a > b for a, b in zip(day_soc, day_soc[1:], strict=False))  # decreasing
    assert b0 - day_soc[-1] > 1.0  # departed battery0 well above round-off (it "bit")


# --- the closure payload is preserved (the leak is a balanced 2-leg transfer) ---------
def test_self_discharge_energy_conserved_every_step(
    leaky: tuple[list[State], int, tuple],
) -> None:
    # Adding a donor-controlled flow does not break energy closure: the augmented ENERGY
    # ledger (solar_source + battery + waste_heat) still balances to round-off each step
    # (the leak's −leak + leak = 0). The Phase-5 payload, with the third flow.
    states, _, _ = leaky
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= 1e-6


def test_self_discharge_energy_total_is_invariant(
    leaky: tuple[list[State], int, tuple],
) -> None:
    # Integral form: the total ENERGY across all three stocks never leaves battery0 (the
    # leak moves joules battery → heat, it does not destroy them — "every joule named").
    states, _, _ = leaky
    for s in states:
        total = (
            s.stocks[SOLAR_SOURCE].amount
            + s.stocks[BATTERY].amount
            + s.stocks[WASTE_HEAT].amount
        )
        assert total == pytest.approx(_SCENARIO.battery0, abs=1e-4)


def test_self_discharge_waste_heat_monotonic(
    leaky: tuple[list[State], int, tuple],
) -> None:
    # waste_heat now also receives the leak, and still only ever receives — so it is
    # non-decreasing every step, strictly grown over the run (heat diagnostic).
    states, _, _ = leaky
    heat = [s.stocks[WASTE_HEAT].amount for s in states]
    assert all(b <= a for b, a in zip(heat, heat[1:], strict=False))
    assert heat[-1] > heat[0] > -1.0


# --- rationed == 0 / events == () (LoadDraw sizing + the leg self-limits) -------------
def test_self_discharge_never_rations(leaky: tuple[list[State], int, tuple]) -> None:
    # The battery stays well-fed (the leak decays it only ~1% over the horizon), so the
    # Euler backstop never fires. NOTE: this leans on LoadDraw's well-fed sizing; the
    # self-discharge LEG is separately structural (k·dt < 1), but that is not what keeps
    # the run unrationed here.
    _, rationed, _ = leaky
    assert rationed == 0


def test_self_discharge_no_events(leaky: tuple[list[State], int, tuple]) -> None:
    _, _, events = leaky
    assert events == ()


# --- the broken bit-identity: RK4 agrees with Euler only to TOLERANCE now -------------
def test_self_discharge_breaks_rk4_euler_bit_identity() -> None:
    # The state-dependent leak means the RK4 stage derivatives are no longer identical
    # (k1 ≠ k2), so RK4 ≢ Euler bit-for-bit — unlike the forced-only BOUNDED_SOC run
    # (test_power_run.py's bit-identity, which must stay). They agree to O(dt²) tol; the
    # tiny leak makes that gap small but nonzero.
    euler, _, _ = _run(integrator_cls=EulerIntegrator)
    rk4, _, _ = _run(integrator_cls=Rk4Integrator)
    e_final = euler[-1].stocks[BATTERY].amount
    r_final = rk4[-1].stocks[BATTERY].amount
    assert r_final != e_final  # NOT bit-identical (the identity is broken)
    assert r_final == pytest.approx(e_final, rel=1e-5)  # but agree to tolerance


# --- determinism ----------------------------------------------------------------------
def test_self_discharge_is_deterministic(
    leaky: tuple[list[State], int, tuple],
) -> None:
    states, rationed, events = leaky
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)
