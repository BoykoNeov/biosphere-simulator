"""Thermal flows: heat input + the nonlinear Stefan-Boltzmann radiator (Step 5).

The two flows of the standalone Thermal core, both internally balanced in ``ENERGY``
(``Σ legs == 0``, asserted every step now that ENERGY is conserved — Phase 5 / P5.1):

  * **HeatInput** — ``heat_source → node``, the forced heat supply ``X =
    env.get(heat_load)·dt`` (W·s = J). Heat → heat, **no form change**, so unlike
    Power's ``SolarCharge`` (electrical → electrical + charge-loss-heat, 3 legs) there
    is **no conversion loss** — **two legs**, a single magnitude ``X`` ⇒ ENERGY balances
    exactly (``−X + X = 0``). *Forced* (reads ``env``, not a stock), the standalone
    stand-in for Power's dissipation feeding the node (Phase-6 coupling replaces it).
  * **RadiatorReject** — ``node → boundary.space``, the **nonlinear** heat rejection ``R
    = ε·σ·A·(T⁴ − T_space⁴)·dt`` (J), with ``T = T_space + node/C`` read off the node's
    own amount. **Donor-controlled** (reads the node stock), two legs use the same
    magnitude ``R`` ⇒ ENERGY balances exactly (``−R + R = 0``). This is the physically
    correct rejection mode in **vacuum** — radiation is the *only* mode with no medium
    (Stefan-Boltzmann; textbook, clean-room, cited below).

**Why the T⁴ radiator, not a linear leak (the genuinely-new machinery).** A linear ``R =
k·node`` would be ``SelfDischarge`` with the sink renamed — no new content. The
Stefan-Boltzmann ``T⁴`` law brings what Thermal exists to add: a **temperature** (the
first non-J derived readout), a **heat capacity**, and a **nonlinear restoring force**
that produces a *real* **emergent equilibrium temperature** ``T_eq`` where
``ε·σ·A·(T_eq⁴ − T_space⁴) = heat_load``. That is a *genuine* attractor — contrast
Power's ``BOUNDED_SOC``, whose boundedness was *constructed* by exact daily balance. Any
constant heat input yields a unique stable ``T_eq`` (radiated is monotone in ``T``), so
— unlike Power — the resolver needs **no** derived/tuned load (the radiator is the
balance).

**Positivity is sizing-dependent here, NOT structural like ``k·dt < 1``.** Radiated ∝
``T⁴``, not ∝ ``Q``, so there is no clean single-param positivity guarantee. Two parts
keep it (and both are honest about *how*):
  * **At the floor — structural.** Because the node is referenced to ``T_space`` (``Q =
    C·(T − T_space) ≥ 0``), ``R → 0`` as ``Q → 0`` (the ``T⁴ − T_space⁴`` driving term
    vanishes), so the radiator cannot pull the node below the floor.
  * **Near equilibrium — by sizing (the well-fed discipline, like Power's
    ``LoadDraw``).** The real risk is **Euler overshoot** when radiated is large: the
    relaxation time ``τ = C / (4·ε·σ·A·T_eq³)`` must be ``>> dt``. The scenario sizes
    the heat capacity ``C`` so ``τ/dt`` is tens of steps (see
    ``domains.thermal.scenario`` / ``system.relaxation_time``), which keeps a step's
    rejection ``≪`` the stored ``Q`` ⇒ ``rationed == 0``. This is a *sizing* claim,
    framed like ``LoadDraw`` — **not** a structural one like ``SelfDischarge``.

**RK4 ≢ Euler (a tolerance agreement, like ``SelfDischarge``).** The radiator reads a
stock and is nonlinear, so the RK4 stage derivatives differ (``k1 ≠ k2``) and the
forced-only bit-identity does not hold; the integrators agree to ``O(dt²)``.

Pure stdlib only. Citations: the Stefan-Boltzmann law (radiated flux ``∝ σ T⁴``) and the
grey-body emissivity ``ε`` are textbook radiative-heat-transfer physics (clean-room);
``σ`` is the CODATA physical constant below. Radiator emissivity / area / heat capacity
/ space temperature are ``params/radiator.yaml`` (provisional, ``TODO(cite)``).
"""

from dataclasses import dataclass

from domains.thermal.stocks import HEAT_LOAD_VAR
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State

# The Stefan-Boltzmann constant σ (W·m⁻²·K⁻⁴) — a **universal physical constant**, not a
# model coefficient, so it lives here as a named constant with provenance (the
# ``drift.py`` "documented constant with provenance, not config YAML" discipline; cf.
# ``BALANCE_ATOL`` as a ``simcore`` constant). The tunable radiative *properties*
# (emissivity, area) ARE params (``params/radiator.yaml``); σ is fixed by physics.
# Value: CODATA 2018 recommended value (exact since the 2019 SI redefinition fixed k_B,
# h, c): σ = 2π⁵k_B⁴ / (15 h³ c²) = 5.670374419e-8 W·m⁻²·K⁻⁴. Source: CODATA 2018
# (Tiesinga et al., Rev. Mod. Phys. 93, 025010, 2021).
STEFAN_BOLTZMANN: float = 5.670374419e-8


@dataclass(frozen=True)
class ThermalParams:
    """Loader-produced radiator parameters (the node's thermal + radiative properties).

    Provisional literature-typical placeholders pending the validation gate (see
    ``params/radiator.yaml``). All four are exact-string unit-guarded at the loader
    (none is a conserved-Quantity canonical unit, so none routes through pint — the
    ``ChargeParams`` discipline):

      * ``emissivity`` (ε, dimensionless) ∈ (0, 1]: the grey-body emissivity of the
        radiator surface. 1 = a black body; the loader rejects 0 (a surface that
        radiates nothing — no rejection path) and out-of-range values.
      * ``radiator_area`` (A, m²) > 0: the radiating area.
      * ``heat_capacity`` (C, J/K) > 0: the node's total heat capacity (thermal mass ×
        specific heat). Sets both the temperature readout ``T = T_space + Q/C`` and the
        relaxation time ``τ = C/(4εσA·T_eq³)`` — sized large enough that ``τ >> dt``
        keeps Euler from overshooting (the well-fed sizing discipline).
      * ``space_temperature`` (T_space, K) ≥ 0: the radiative sink temperature (deep
        space ≈ 2.7 K). Also the node's reference: ``Q = C·(T − T_space)``.
    """

    emissivity: float
    radiator_area: float
    heat_capacity: float
    space_temperature: float


def temperature(
    node_joules: float, *, heat_capacity: float, space_temperature: float
) -> float:
    """The node temperature (K), the derived readout ``T = T_space + Q/C``.

    A pure function of the node's stored sensible heat ``Q = node_joules`` — computed at
    evaluate-time, not a stock or an aux accumulator. ``Q = 0`` ⇒ ``T = T_space`` (the
    floor); positive ``Q`` warms it linearly in the heat capacity.
    """
    return space_temperature + node_joules / heat_capacity


def radiated_power(node_joules: float, *, params: ThermalParams) -> float:
    """The instantaneous radiated power ``ε·σ·A·(T⁴ − T_space⁴)`` (W = J/s).

    The Stefan-Boltzmann grey-body law with ``T = temperature(node_joules)``. Nonlinear
    and **donor-controlled** (rises steeply with ``T``), so it self-limits to 0 as the
    node cools to ``T_space`` (``Q → 0``) — structural positivity at the floor. A flow
    multiplies by ``dt`` for the per-step joule increment (the increment-form contract).
    At ``Q = 0`` it is exactly 0.
    """
    t = temperature(
        node_joules,
        heat_capacity=params.heat_capacity,
        space_temperature=params.space_temperature,
    )
    return (
        params.emissivity
        * STEFAN_BOLTZMANN
        * params.radiator_area
        * (t**4 - params.space_temperature**4)
    )


@dataclass(frozen=True)
class HeatInput:
    """ENERGY flow ``heat_source → node`` — the forced heat input (Step 5).

    Withdraws the forced supply ``X = env.get(heat_load)·dt`` (W·s = J) from the
    unclamped ``heat_source`` boundary and deposits **all** of it into the node (heat →
    heat, no form change ⇒ no loss leg). A single magnitude ``X`` in both legs ⇒ ENERGY
    balances exactly (``−X + X = 0``). Forced (not donor-controlled); ``flux = rate·dt``
    is dt-linear (RK4-order-safe, Phase-6-multi-rate-safe). Standalone this is the
    stand-in for Power's dissipation; Phase-6 coupling feeds the node from Power's legs.
    """

    id: FlowId
    priority: int
    heat_source: StockId
    node: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        supply = env.get(HEAT_LOAD_VAR) * dt
        return FlowResult(
            legs=(
                Leg(self.heat_source, -supply),
                Leg(self.node, supply),
            )
        )


@dataclass(frozen=True)
class RadiatorReject:
    """ENERGY flow ``node → boundary.space`` — the Stefan-Boltzmann radiator (Step 5).

    Rejects ``R = radiated_power(node)·dt = ε·σ·A·(T⁴ − T_space⁴)·dt`` (J) of stored
    heat to the permanent deep-space sink, with ``T`` read off the node's own amount.
    Two legs use the same magnitude ``R`` ⇒ ENERGY balances exactly (``−R + R = 0``).
    **Donor-controlled** (``R`` depends on the node stock), so it is the domain's
    restoring force — the SOC-analogue (heat) gains a genuine nonlinear attractor at the
    emergent equilibrium temperature ``T_eq``. Positivity is structural at the floor
    (``R → 0`` as ``Q → 0``) and by well-fed sizing near equilibrium (``τ >> dt``); see
    the module docstring. Because it reads a stock and is nonlinear, RK4 ≢ Euler (a
    tolerance agreement). ``flux = rate·dt`` is dt-linear.
    """

    id: FlowId
    priority: int
    node: StockId
    space: StockId
    params: ThermalParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        rejected = (
            radiated_power(snapshot.stocks[self.node].amount, params=self.params) * dt
        )
        return FlowResult(
            legs=(
                Leg(self.node, -rejected),
                Leg(self.space, rejected),
            )
        )
