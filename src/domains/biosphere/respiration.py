"""Maintenance + growth respiration (Phase-1 Step 6; the carbon **sink** flows).

The counterpart to Step 5's gross assimilation. Step 5 deposits *gross* assimilated
carbon into the (provisional) ``plant_c`` pool; Step 6 books the two respiratory
carbon losses that turn that gross flux into the net structural increment, each an
explicit balanced carbon flow ``plant_c -> boundary.co2`` (P1, single-currency):

* **Maintenance respiration** — the carbon cost of keeping existing tissue alive,
  proportional to standing biomass and rising with temperature (a ``Q10`` response):

      ``MRES = maintenance_coef · plant_c · Q10^((T − T_ref)/10)``   (mol C day⁻¹)

  Self-limiting in ``plant_c`` (→ 0 as biomass → 0), so positivity is structural.

* **Growth respiration** — the conversion loss when assimilate is built into
  structural tissue. The **maintenance-first** paradigm (McCree 1970; Penning de
  Vries et al. 1974; Thornley 1970; this is also how the WOFOST oracle budgets
  carbon — ``ASRC = GPHOT − MRES``, then ``× CVF``): growth respiration acts on the
  assimilate **remaining after maintenance**, not on gross:

      ``GRES = (1 − Yg) · max(0, GASS − MRES)``                      (mol C day⁻¹)

  where ``Yg`` is the carbon growth-conversion efficiency (mol structural C retained
  per mol available C). The ``max(0, …)`` clamp is **load-bearing** — the Step-6
  analogue of Step 5's ``Γ*`` clamp: when maintenance exceeds assimilation there is
  no growth, hence no growth respiration, and the flow never flips into a *deposit*
  (carbon creation). The net ``plant_c`` change from the three flows is then
  ``GASS − MRES − GRES = Yg·(GASS − MRES)`` — the structural increment, matching the
  cited budget.

**Why ``GrowthRespiration`` recomputes ``GASS`` and ``MRES``.** Flows ``evaluate``
independently against the step-entry snapshot — a flow cannot read another flow's
result before arbitration — so a flux-coupled quantity must recompute its inputs.
``GASS`` is recomputed via the Step-5 :func:`daily_canopy_assimilation` seam; ``MRES``
via the **same** :func:`maintenance_respiration_flux` that ``MaintenanceRespiration``
uses, so the two flows can never drift on the maintenance value (the one genuine DRY
hazard). The long field list on ``GrowthRespiration`` is the inherent cost of a
flux-coupled quantity in an independent-flow engine, not a smell.

**Deferred seams (documented, like the f_water/f_N seams in Step 5).** WOFOST scales
maintenance *down* as tissue matures (a development-stage / senescence factor); that
lands with phenology (Step 8) and the Step-11 oracle tuning. Standalone Step 6 is
plain ``Q10·biomass`` with the multiplier seam (``maturity`` argument, default 1.0)
in place, so Step 11 is a coefficient change, not a structural one.

**Known provisional behavior — winter biomass decline.** When ``GASS < MRES`` (dark,
cold) the net ``plant_c`` change is ``−(MRES − GASS) < 0``: standing biomass shrinks.
This is physically real for a respiring plant in carbon deficit; dormancy /
vernalization is the Step-11 refinement. It is the Step-6 analogue of Step 5's
documented big-leaf high-bias — a known provisional behavior, not a bug.

Pure stdlib only. Citations: McCree, K.J. (1970), "An equation for the rate of
respiration of white clover plants grown under controlled conditions", in *Prediction
and Measurement of Photosynthetic Productivity*, PUDOC, Wageningen, 221–229;
Penning de Vries, F.W.T., Brunsting, A.H.M. & van Laar, H.H. (1974), "Products,
requirements and efficiency of biosynthesis: a quantitative approach", J. Theor.
Biol. 45:339–377; Thornley, J.H.M. (1970), "Respiration, growth and maintenance in
plants", Nature 227:304–305; Amthor, J.S. (2000), "The McCree–de Wit–Penning de
Vries–Thornley respiration paradigms: 30 years later", Ann. Bot. 86:1–20.
"""

from dataclasses import dataclass

from domains.biosphere.canopy import CanopyParams, leaf_area_index
from domains.biosphere.photosynthesis import (
    PhotosynthesisParams,
    daily_canopy_assimilation,
)
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class RespirationParams:
    """Loader-produced respiration parameters in core-ready form.

    Mirrors ``PhotosynthesisParams``/``CanopyParams``: declared data, no magic
    numbers in the physics. All values are provisional literature-typical
    placeholders pending the Step-11 validation gate (see ``params/respiration.yaml``).
    Rates are expressed on a **carbon basis** (mol C per mol C biomass), so the
    pure physics never holds the kg-DM⇄mol-C carbon fraction (the Step-1 lock).
    """

    maintenance_coef: float  # m_ref, mol C respired / mol C biomass / day at t_ref
    q10: float  # Q10 temperature sensitivity of maintenance (dimensionless)
    t_ref: float  # reference temperature for the Q10 response (°C)
    growth_efficiency: float  # Yg, mol structural C / mol available C (0 < Yg ≤ 1)


def q10_factor(temp_c: float, *, q10: float, t_ref: float) -> float:
    """Q10 temperature multiplier ``q10^((T − T_ref)/10)`` (dimensionless).

    The classic exponential temperature response of maintenance respiration: the
    rate multiplies by ``q10`` for every 10 °C above ``t_ref`` (and divides below).
    Unlike the FvCB cardinal-temperature factor, this is **unbounded above** — that
    is correct: maintenance respiration keeps rising with temperature.
    """
    return q10 ** ((temp_c - t_ref) / 10.0)


def maintenance_respiration_flux(
    plant_c: float, temp_c: float, *, params: RespirationParams, maturity: float = 1.0
) -> float:
    """Daily maintenance respiration ``m_ref · plant_c · Q10 · maturity`` (mol C day⁻¹).

    Proportional to standing biomass ``plant_c`` (mol C) and to the
    :func:`q10_factor` temperature response. ``maturity`` ∈ [0, 1] is the deferred
    development-stage / senescence down-scaling seam (default 1.0 in Step 6; populated
    at Step 8/11). Returns 0 at ``plant_c = 0`` — self-limiting, so positivity is
    structural (no backstop dependence; P3).
    """
    return (
        params.maintenance_coef
        * plant_c
        * q10_factor(temp_c, q10=params.q10, t_ref=params.t_ref)
        * maturity
    )


def growth_respiration_flux(
    gross: float, maintenance: float, *, growth_efficiency: float
) -> float:
    """Daily growth respiration ``(1 − Yg) · max(0, GASS − MRES)`` (mol C day⁻¹).

    The maintenance-first conversion loss (McCree–de Vries–Thornley): growth
    respiration acts on the assimilate left **after** maintenance. The ``max(0, …)``
    clamp keeps the *sink* flow a sink — when ``MRES ≥ GASS`` there is no growth, so
    growth respiration is 0 rather than a (carbon-creating) negative withdrawal.
    """
    return (1.0 - growth_efficiency) * max(0.0, gross - maintenance)


@dataclass(frozen=True)
class MaintenanceRespiration:
    """CARBON sink flow ``plant_c -> co2_sink`` (maintenance; balanced in carbon, P1).

    Reads air temperature as a scalar driver through ``env.get`` (forcing or shared
    stock — the flow cannot tell, #16). ``flux = maintenance_respiration_flux(...)·dt``
    — dt-linear (the daily rate is dt-independent), so the RK4 increment-form contract
    holds. Respires into the **same** ``boundary.co2`` reservoir gross assimilation
    draws from (open/unclamped in the Phase-1 single-producer system).
    """

    id: FlowId
    priority: int
    plant_c: StockId
    co2_sink: StockId
    temp_var: str
    params: RespirationParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        temp_c = env.get(self.temp_var)
        plant_c = snapshot.stocks[self.plant_c].amount
        daily = maintenance_respiration_flux(plant_c, temp_c, params=self.params)
        flux = daily * dt
        return FlowResult(legs=(Leg(self.plant_c, -flux), Leg(self.co2_sink, flux)))


@dataclass(frozen=True)
class GrowthRespiration:
    """CARBON sink flow ``plant_c -> co2_sink`` (growth conversion loss, balanced, P1).

    Flux-coupled to assimilation and maintenance, so it **recomputes** both from the
    step-entry snapshot (flows cannot read each other's results pre-arbitration):
    ``GASS`` via the Step-5 :func:`daily_canopy_assimilation`, ``MRES`` via the same
    :func:`maintenance_respiration_flux` ``MaintenanceRespiration`` uses (no drift).
    Holds the photosynthesis + canopy + respiration params, the scenario
    ``ground_area``, and the same forcing-var names as ``GrossAssimilation`` — the
    inherent cost of a flux-coupled quantity in an independent-flow engine.

    ``flux = growth_respiration_flux(GASS, MRES, Yg)·dt`` — dt-linear (the daily rate
    is dt-independent; the ``max(0, …)`` clamp is on a daily rate, not a dt-gate).
    """

    id: FlowId
    priority: int
    plant_c: StockId
    co2_sink: StockId
    par_var: str
    ci_var: str
    temp_var: str
    daylength_var: str
    photo: PhotosynthesisParams
    canopy: CanopyParams
    resp: RespirationParams
    ground_area: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        incident_par = env.get(self.par_var)
        ci = env.get(self.ci_var)
        temp_c = env.get(self.temp_var)
        daylength_s = env.get(self.daylength_var)
        leaf_carbon = snapshot.stocks[self.plant_c].amount
        lai = leaf_area_index(
            leaf_carbon,
            sla_per_mol_c=self.canopy.sla_per_mol_c,
            ground_area=self.ground_area,
        )
        gross = daily_canopy_assimilation(
            incident_par,
            lai,
            ci,
            temp_c,
            daylength_s,
            params=self.photo,
            canopy=self.canopy,
            ground_area=self.ground_area,
            limitation=1.0,
        )
        maintenance = maintenance_respiration_flux(
            leaf_carbon, temp_c, params=self.resp
        )
        daily = growth_respiration_flux(
            gross, maintenance, growth_efficiency=self.resp.growth_efficiency
        )
        flux = daily * dt
        return FlowResult(legs=(Leg(self.plant_c, -flux), Leg(self.co2_sink, flux)))
