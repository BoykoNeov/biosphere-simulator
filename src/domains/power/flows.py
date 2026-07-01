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

  * **SelfDischarge** — ``battery → waste_heat``, the first-order standing leak
    ``leak = k·battery·dt`` (J). A donor-controlled (first-order-in-SOC) drain: the
    stored charge slowly bleeds to heat even with no load. **Two legs** (``−leak`` from
    the battery, ``+leak`` to ``waste_heat``) balance ENERGY exactly (``−leak + leak =
    0``); a real cell's shelf-loss, textbook. Unlike the two forced flows this reads a
    **stock**, so it is the Power domain's **first donor-controlled flow** — see below.

**Positivity: ``SelfDischarge``'s leg is structural, ``LoadDraw``'s is not.** The two
forced flows (``SolarCharge``/``LoadDraw``) are driven by ``env`` power, not by a stock,
so a constant load *can* over-draw an empty battery → the Euler backstop would fire; the
standalone validation keeps ``LoadDraw``'s positivity by **well-fed sizing** (the
battery never empties — the Phase-1 "kept non-limiting" pattern), landed with the
scenario in Step 3/4. ``SelfDischarge`` is different: its draw is ``k·battery·dt``, **∝
the donor's own start-of-step amount**, so it self-limits to 0 as the battery empties —
positivity is **structural** (``k·dt < 1`` keeps the backstop unfired), the
``Decomposition`` / ``Grazing`` donor-controlled idiom. It is also the Power domain's
**first restoring force** (the ``−k·battery`` term): the SOC gains a genuine attractor —
two runs from different initial SOC **converge** (their difference contracts
geometrically, ``d_n = d_0·(1 − k·dt)^n``), where the two forced flows alone leave that
difference constant. Because it reads a stock, it also **breaks the RK4 ≡ Euler
bit-identity** the forced-only system had (that identity survives only while every flow
is state-independent). A donor-controlled *load* and a brownout perturbation remain
documented seams.

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


@dataclass(frozen=True)
class SelfDischargeParams:
    """Loader-produced self-discharge parameter: the first-order shelf-loss rate.

    A provisional literature-typical placeholder pending the validation gate (see
    ``params/self_discharge.yaml``). ``self_discharge_rate`` (k, 1/s) ≥ 0: the fraction
    of stored charge that leaks to heat per second. 0 disables the leak (inert, a valid
    "ideal cell" — the herbivory "zero rate is valid" precedent); negative is rejected
    at the loader. Positivity of the flow is structural for ``k·dt < 1``
    (donor-controlled, ∝ the battery's own amount).
    """

    # k, self-discharge rate battery → waste_heat (1/s). NOT per-day: Power's natural
    # time unit is seconds (energy J, power W = J/s), unlike the biosphere's /day.
    self_discharge_rate: float


def self_discharge_flux(battery_joules: float, *, self_discharge_rate: float) -> float:
    """Instantaneous self-discharge leak ``self_discharge_rate · battery`` (W = J/s).

    First-order donor-controlled (the ``Decomposition`` / ``Grazing`` form), so it → 0
    as the battery → 0 (positivity is structural). The leaked charge degrades to heat
    (the :class:`SelfDischarge` flow); a flow multiplies by ``dt`` for the per-step
    joule increment (the increment-form contract). At ``battery = 0``/``k = 0`` it is 0.
    """
    return self_discharge_rate * battery_joules


@dataclass(frozen=True)
class SelfDischarge:
    """ENERGY flow ``battery → waste_heat`` — the first-order standing leak (P5.5).

    Bleeds ``self_discharge_flux(battery, k)·dt`` of stored charge to heat each step — a
    real cell's shelf-loss, degrading useful electrical energy to waste heat with no
    load present. Two legs use the same magnitude ``leak`` ⇒ ENERGY balances exactly
    (``−leak + leak = 0``). **Donor-controlled** (``leak ∝ battery``, the first Power
    flow to read a stock), so positivity is structural (``k·dt < 1``) and the SOC gains
    a restoring force (a stable attractor — two runs from different SOC converge).
    Because it reads a stock it breaks the forced-only RK4 ≡ Euler bit-identity. ``flux
    = rate·dt`` is dt-linear (RK4-order-safe, Phase-6-multi-rate-safe).
    """

    id: FlowId
    priority: int
    battery: StockId
    waste_heat: StockId
    params: SelfDischargeParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        leak = (
            self_discharge_flux(
                snapshot.stocks[self.battery].amount,
                self_discharge_rate=self.params.self_discharge_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.battery, -leak),
                Leg(self.waste_heat, leak),
            )
        )
