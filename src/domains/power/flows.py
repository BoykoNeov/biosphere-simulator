"""Power flows: energy-balanced multi-leg transfers with explicit heat closure (P5.2).

The two flows of the standalone power core, both internally balanced in ``ENERGY``
(``Σ legs == 0``, asserted every step now that ENERGY is conserved — Phase 5 / P5.1).
"Every joule named" is **structural**: a lossy transfer is a *multi-leg* flow that books
the degraded fraction as heat, never a hidden net loss.

  * **SolarCharge** — ``solar_source → battery (+η_c) + waste_heat (+(1−η_c))``. The
    forced supply ``X = env.get(solar_power)·dt`` (W·s = J) splits into the stored
    fraction ``η_c·X`` (into the battery) and the charge-conversion loss ``(1−η_c)·X``
    (named as heat). **Always three legs** (the decomposition "emit the structural legs
    even at zero amount" convention) — at ``η_c = 1`` the heat leg is exactly 0 (the
    "collapses to 2" case), at night (``X = 0``) all three legs are 0 (a no-op step).
  * **LoadDraw** — ``battery → waste_heat``, the forced dissipative load
    ``Y = env.get(load_power)·dt`` (W·s = J). A resistive/compute load: **100 % →
    heat**, the cleanest "every joule named" demonstration. A single magnitude ``Y``
    in both legs ⇒ balances exactly. (Useful work leaving the system — a pump moving
    fluid → a ``useful_work`` BOUNDARY sink — is the documented split-seam, deferred;
    standalone Power makes heat the star.)

**Energy degrades; the first law still holds.** LoadDraw moving ``Y`` from battery to
waste_heat is **first-law-conserved** (joules balance) — the "loss" is *exergy*, not
joules. That degradation is the monotonic heat-generated diagnostic (the ``waste_heat``
amount), **not** a second conserved quantity. ``charge_efficiency`` (η_c) is therefore a
**one-way charge efficiency** — the conversion loss *into* the battery — **not** a
round-trip efficiency. Because discharge here is **joule-lossless**, the *modeled*
round-trip *equals* η_c (charge is the only joule-losing leg); a realistic charge-leg
η_c ≈ 0.95 thus makes the modeled round-trip optimistic relative to a real
~0.90-round-trip cell — the discharge-side conversion loss is a deferred seam (see
``params/charge.yaml``).

**Positivity / ``rationed == 0`` is a sizing discipline, not structural here.** Both
flows are **forced** (driven by ``env`` power, not first-order in a stock), so a
constant load *can* over-draw an empty battery → the Euler backstop would fire. The
standalone validation keeps ``rationed == 0`` by **well-fed sizing** (the battery never
empties — the Phase-1 "kept non-limiting" pattern), landed with the scenario in Step
3/4. A
donor-controlled (first-order-in-SOC) load and a brownout perturbation are documented
seams. A first-order ``SelfDischarge`` (``battery → waste_heat``, ``k·battery·dt``,
donor-controlled so it self-limits to 0) is the third, optional flow — added only if it
earns its keep in the validation scenario.

Pure stdlib only. Citation: battery charge/round-trip efficiency and the
electricity-degrades-to-heat first law are textbook (clean-room, cited in
``params/charge.yaml``); the η_c value is a provisional ``TODO(cite)`` placeholder
pending the validation gate.
"""

from dataclasses import dataclass

from domains.power.stocks import LOAD_POWER_VAR, SOLAR_POWER_VAR
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class ChargeParams:
    """Loader-produced charge parameter: the one-way charge efficiency.

    A provisional literature-typical placeholder pending the validation gate (see
    ``params/charge.yaml``). ``charge_efficiency`` ∈ (0, 1]: the fraction of supplied
    energy that reaches the battery, the remainder named as heat. 1 = lossless charging
    (the heat leg collapses to 0); the loader rejects 0 (a battery that stores nothing)
    and out-of-range values.
    """

    # η_c, one-way charge efficiency (dimensionless): stored / supplied. NOT round-trip.
    charge_efficiency: float


def charge_split(
    supply_joules: float, *, charge_efficiency: float
) -> tuple[float, float]:
    """Split supplied energy into ``(stored, lost_to_heat)`` (J), summing to the input.

    ``stored = η_c · supply``; ``lost = (1 − η_c) · supply``. Plain, deterministic
    arithmetic — the ~1e-15 round-off in ``stored + lost`` vs ``supply`` is covered by
    ``assert_flow_balanced``'s relative tolerance, exactly as for every other flow (the
    invariant is determinism, not bit-exact ``Σ == 0``). At ``η_c = 1`` ``lost`` is
    exactly 0; at ``supply = 0`` both are 0.
    """
    stored = charge_efficiency * supply_joules
    lost = (1.0 - charge_efficiency) * supply_joules
    return stored, lost


@dataclass(frozen=True)
class SolarCharge:
    """ENERGY flow ``solar_source → battery (+η_c) + waste_heat (+(1−η_c))`` (P5.2).

    The forced solar supply ``X = env.get(solar_power)·dt`` (W·s = J) is withdrawn from
    the unclamped ``solar_source`` boundary and split: ``η_c·X`` stored in the battery,
    ``(1−η_c)·X`` named as charge-conversion heat. Three legs balance ENERGY exactly in
    intent (``−X + η_c·X + (1−η_c)·X = 0``). Always three legs (zero-amount at η_c=1 /
    night); ``flux = rate·dt`` is dt-linear (RK4-order-safe, Phase-6-multi-rate-safe).
    """

    id: FlowId
    priority: int
    solar_source: StockId
    battery: StockId
    waste_heat: StockId
    params: ChargeParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        supply = env.get(SOLAR_POWER_VAR) * dt
        stored, lost = charge_split(
            supply, charge_efficiency=self.params.charge_efficiency
        )
        return FlowResult(
            legs=(
                Leg(self.solar_source, -supply),
                Leg(self.battery, stored),
                Leg(self.waste_heat, lost),
            )
        )


@dataclass(frozen=True)
class LoadDraw:
    """ENERGY flow ``battery → waste_heat`` — the dissipative load (P5.2).

    Withdraws the forced demand ``Y = env.get(load_power)·dt`` (W·s = J) from the
    battery and deposits **all** of it as heat (100 % dissipative — a resistive/compute
    load, the cleanest "every joule named"). A single magnitude ``Y`` in both legs ⇒
    ENERGY balances exactly (``−Y + Y = 0``). Forced (not donor-controlled), so
    positivity is a sizing discipline (well-fed battery), not structural — see the
    module docstring. ``flux = rate·dt`` is dt-linear.
    """

    id: FlowId
    priority: int
    battery: StockId
    waste_heat: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        draw = env.get(LOAD_POWER_VAR) * dt
        return FlowResult(
            legs=(
                Leg(self.battery, -draw),
                Leg(self.waste_heat, draw),
            )
        )
