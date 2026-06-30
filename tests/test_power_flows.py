"""Phase-5 P5.2 tests: the standalone Power flows (ENERGY only; energy closure).

Step 2 (P5.2 core) adds the Power domain's two energy-balanced flows — the first
carriers of the Phase-5 energy-closure decision (P5.1: ENERGY joined the asserted
conserved set). Both balance ENERGY (``Σ legs == 0``); "every joule named" is
structural (the degraded fraction is a visible heat leg, never a hidden net loss):

* **SolarCharge** — ``solar_source → battery (+η_c) + waste_heat (+(1−η_c))``. The
  forced supply splits into the stored fraction and the charge-conversion loss (heat).
* **LoadDraw** — ``battery → waste_heat``, the 100 %-dissipative forced load (the
  cleanest "every joule named").

Three layers, mirroring ``test_decomposition`` (the established per-flow discipline):

* **Rate law** — ``charge_split`` is ``(η·X, (1−η)·X)`` summing to ``X`` (→ (0, 0) at
  ``X = 0``; loss → 0 at ``η = 1``).
* **Flow level** — each flow's legs balance ENERGY and touch no other quantity, are
  dt-linear (``flux = rate·dt`` — the increment-form contract, RK4-order-safe), and
  self-limit to zero legs on zero input.
* **Loader** — the committed η_c, with out-of-range and bad-unit rejection.

No scenario / golden / conservation-gate run here — those are Step 3/4 (the
``assert_flow_balanced`` per-flow check is the Step-2 gate). Pure-stdlib spine.
"""

import math
from pathlib import Path

import pytest

from domains.power.flows import ChargeParams, LoadDraw, SolarCharge, charge_split
from domains.power.loader import load_charge_params
from domains.power.stocks import (
    BATTERY,
    LOAD_POWER_VAR,
    SOLAR_POWER_VAR,
    SOLAR_SOURCE,
    WASTE_HEAT,
    battery_stock,
)
from simcore import boundary
from simcore.environment import BoundEnvironment, SourceResolver, constant
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId
from simcore.quantities import Quantity
from simcore.state import State


# --- shared fixtures ---------------------------------------------------------
def _state(*, battery: float = 1_000.0) -> State:
    """A State with the three Power stocks (battery POOL + the two boundaries)."""
    stocks = {
        s.id: s
        for s in (
            battery_stock(battery),
            boundary.source(SOLAR_SOURCE, Quantity.ENERGY, 0.0),
            boundary.sink(WASTE_HEAT, Quantity.ENERGY, 0.0),
        )
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _env(
    state: State, dt: float, *, solar_power: float = 0.0, load_power: float = 0.0
) -> BoundEnvironment:
    """Bind constant solar/load forcing schedules to ``state`` + ``dt`` (#16)."""
    return SourceResolver(
        forcings={
            SOLAR_POWER_VAR: constant(solar_power),
            LOAD_POWER_VAR: constant(load_power),
        }
    ).bind(state, dt)


def _solar_charge(eta: float = 0.95) -> SolarCharge:
    return SolarCharge(
        FlowId("power.solar_charge"),
        0,
        solar_source=SOLAR_SOURCE,
        battery=BATTERY,
        waste_heat=WASTE_HEAT,
        params=ChargeParams(charge_efficiency=eta),
    )


def _load_draw() -> LoadDraw:
    return LoadDraw(
        FlowId("power.load_draw"), 0, battery=BATTERY, waste_heat=WASTE_HEAT
    )


# --- rate law: charge_split --------------------------------------------------
def test_charge_split_is_the_efficiency_partition() -> None:
    # stored = η·X, lost = (1−η)·X — a hand value + the partition.
    stored, lost = charge_split(100.0, charge_efficiency=0.95)
    assert math.isclose(stored, 95.0, rel_tol=1e-12)
    assert math.isclose(lost, 5.0, rel_tol=1e-12)


def test_charge_split_sums_to_supply() -> None:
    # The two fractions sum back to the input (energy is named, not lost).
    stored, lost = charge_split(137.0, charge_efficiency=0.88)
    assert math.isclose(stored + lost, 137.0, rel_tol=1e-12)


def test_charge_split_lossless_at_eta_one() -> None:
    # η = 1: all supply stored, the heat fraction is exactly 0 (collapses to 2 legs).
    stored, lost = charge_split(100.0, charge_efficiency=1.0)
    assert stored == 100.0
    assert lost == 0.0


def test_charge_split_zero_supply() -> None:
    # No supply ⇒ nothing stored, nothing lost (night / self-limit).
    assert charge_split(0.0, charge_efficiency=0.95) == (0.0, 0.0)


# --- SolarCharge -------------------------------------------------------------
def test_solar_charge_splits_supply_into_battery_and_heat() -> None:
    # The supply leaves the source (−X) and lands as stored (+η·X) + heat (+(1−η)·X).
    state = _state()
    legs = {
        leg.stock: leg.amount
        for leg in _solar_charge(0.95)
        .evaluate(state, _env(state, 1.0, solar_power=100.0), 1.0)
        .legs
    }
    assert math.isclose(legs[SOLAR_SOURCE], -100.0, rel_tol=1e-12)  # withdrawn
    assert math.isclose(legs[BATTERY], 95.0, rel_tol=1e-12)  # stored
    assert math.isclose(legs[WASTE_HEAT], 5.0, rel_tol=1e-12)  # charge loss → heat


def test_solar_charge_balances_energy_only() -> None:
    # The 3-leg lossy flow balances ENERGY and touches no other quantity.
    state = _state()
    result = _solar_charge(0.95).evaluate(
        state, _env(state, 1.0, solar_power=100.0), 1.0
    )
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.ENERGY}


def test_solar_charge_always_three_legs() -> None:
    # The structural-legs convention (decomposition's "emit even at zero"): three legs
    # at η<1, at η=1 (heat leg 0), and at night (all 0) — never a variable leg count.
    state = _state()
    for eta, solar in ((0.95, 100.0), (1.0, 100.0), (0.95, 0.0)):
        legs = (
            _solar_charge(eta)
            .evaluate(state, _env(state, 1.0, solar_power=solar), 1.0)
            .legs
        )
        assert len(legs) == 3


def test_solar_charge_lossless_at_eta_one() -> None:
    # η = 1: the heat leg is exactly 0 and the battery receives the full supply.
    state = _state()
    legs = {
        leg.stock: leg.amount
        for leg in _solar_charge(1.0)
        .evaluate(state, _env(state, 1.0, solar_power=100.0), 1.0)
        .legs
    }
    assert legs[WASTE_HEAT] == 0.0
    assert math.isclose(legs[BATTERY], 100.0, rel_tol=1e-12)


def test_solar_charge_night_is_noop() -> None:
    # No solar supply ⇒ three zero-amount legs (a clean no-op step, dt-independent).
    state = _state()
    legs = (
        _solar_charge(0.95).evaluate(state, _env(state, 1.0, solar_power=0.0), 1.0).legs
    )
    assert all(leg.amount == 0.0 for leg in legs)


def test_solar_charge_is_dt_linear() -> None:
    # flux = rate·dt — the increment-form contract (RK4 order; Phase-6 multi-rate-safe).
    state = _state()
    half = {
        leg.stock: leg.amount
        for leg in _solar_charge(0.95)
        .evaluate(state, _env(state, 0.5, solar_power=100.0), 0.5)
        .legs
    }
    full = {
        leg.stock: leg.amount
        for leg in _solar_charge(0.95)
        .evaluate(state, _env(state, 1.0, solar_power=100.0), 1.0)
        .legs
    }
    assert math.isclose(full[BATTERY], 2.0 * half[BATTERY], rel_tol=1e-12)
    assert math.isclose(full[WASTE_HEAT], 2.0 * half[WASTE_HEAT], rel_tol=1e-12)


# --- LoadDraw ----------------------------------------------------------------
def test_load_draw_dissipates_battery_to_heat() -> None:
    # 100 % dissipative: the draw leaves the battery (−Y) and lands as heat (+Y).
    state = _state()
    legs = {
        leg.stock: leg.amount
        for leg in _load_draw()
        .evaluate(state, _env(state, 1.0, load_power=40.0), 1.0)
        .legs
    }
    assert math.isclose(legs[BATTERY], -40.0, rel_tol=1e-12)
    assert math.isclose(legs[WASTE_HEAT], 40.0, rel_tol=1e-12)


def test_load_draw_balances_energy_only() -> None:
    # Single magnitude in both legs ⇒ ENERGY balances exactly; no other quantity moved.
    state = _state()
    result = _load_draw().evaluate(state, _env(state, 1.0, load_power=40.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.ENERGY}


def test_load_draw_zero_load_is_noop() -> None:
    # No demand ⇒ zero-amount legs (the dt-independent self-limit).
    state = _state()
    legs = _load_draw().evaluate(state, _env(state, 1.0, load_power=0.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


def test_load_draw_is_dt_linear() -> None:
    # flux = rate·dt — the increment-form contract.
    state = _state()
    half = {
        leg.stock: leg.amount
        for leg in _load_draw()
        .evaluate(state, _env(state, 0.5, load_power=40.0), 0.5)
        .legs
    }
    full = {
        leg.stock: leg.amount
        for leg in _load_draw()
        .evaluate(state, _env(state, 1.0, load_power=40.0), 1.0)
        .legs
    }
    assert math.isclose(full[WASTE_HEAT], 2.0 * half[WASTE_HEAT], rel_tol=1e-12)


# --- loader ------------------------------------------------------------------
def test_loader_reads_committed_efficiency() -> None:
    assert load_charge_params().charge_efficiency == 0.95


def _write_charge(tmp_path: Path, *, value: float, unit: str = "dimensionless") -> Path:
    bad = tmp_path / "charge.yaml"
    bad.write_text(
        "name: power\nprocess: charge\nparameters:\n"
        f"  charge_efficiency:\n    value: {value}\n    unit: {unit!r}\n"
        '    source: "test"\n',
        encoding="utf-8",
    )
    return bad


def test_loader_rejects_zero_efficiency(tmp_path: Path) -> None:
    # 0 = a battery that stores nothing — meaningless; (0, 1] is required.
    with pytest.raises(ValueError, match="charge_efficiency must be in"):
        load_charge_params(_write_charge(tmp_path, value=0.0))


def test_loader_rejects_above_one_efficiency(tmp_path: Path) -> None:
    # > 1 would create energy on charge.
    with pytest.raises(ValueError, match="charge_efficiency must be in"):
        load_charge_params(_write_charge(tmp_path, value=1.5))


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="must be declared in"):
        load_charge_params(_write_charge(tmp_path, value=0.95, unit="J"))
