"""Maintenance + growth respiration rate laws (Phase-1 Step 6; carbon sinks).

The counterpart to Step 5's gross assimilation: the two respiratory carbon losses that
turn the gross flux ``GASS`` into the net structural increment. This module holds the
**rate laws** (pure functions); the assembled flows that book them moved to
``domains.biosphere.carbon_budget`` at Step 11 (the buffer dissolution — they now source
from ``co2_atmos`` + the organ pools, not a ``plant_c`` pool).

* **Maintenance respiration** — the carbon cost of keeping existing tissue alive,
  proportional to standing biomass and rising with temperature (a ``Q10`` response):

      ``MRES = maintenance_coef · biomass · Q10^((T − T_ref)/10)``   (mol C day⁻¹)

  Self-limiting in biomass (→ 0 as biomass → 0), so positivity is structural.

* **Growth respiration** — the conversion loss when assimilate is built into
  structural tissue. The **maintenance-first** paradigm (McCree 1970; Penning de
  Vries et al. 1974; Thornley 1970; this is also how the WOFOST oracle budgets
  carbon — ``ASRC = GPHOT − MRES``, then ``× CVF``): growth respiration acts on the
  assimilate **remaining after maintenance**, not on gross:

      ``GRES = (1 − Yg) · max(0, GASS − MRES)``                      (mol C day⁻¹)

  where ``Yg`` is the carbon growth-conversion efficiency (mol structural C retained
  per mol available C). The ``max(0, …)`` clamp is **load-bearing** — the Step-6
  analogue of Step 5's ``Γ*`` clamp: when maintenance exceeds assimilation there is
  no growth, hence no growth respiration. With allocation depositing ``DMI =
  Yg·max(0, GASS − MRES)``, the structural increment matches the cited budget.

:func:`available_for_growth` (the shared ``max(0, GASS − MRES)``) is consumed by the
Step-11 :class:`~domains.biosphere.carbon_budget.GrowthRespiration` and
:class:`~domains.biosphere.carbon_budget.Allocation` so they agree on the carbon budget
by construction (the one genuine cross-flow drift hazard).

**Deferred seams.** WOFOST scales maintenance *down* as tissue matures (a development-
stage / senescence factor); the ``maturity`` argument (default 1.0) is the seam. The
``GASS < MRES`` carbon-deficit case (cold/dark overwintering) shrinks biomass — in the
dissolved-buffer model the maintenance shortfall is drawn from the organ pools (see
``carbon_budget.MaintenanceRespiration``); dormancy / vernalization is a refinement.

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
    biomass: float, temp_c: float, *, params: RespirationParams, maturity: float = 1.0
) -> float:
    """Daily maintenance respiration ``m_ref · biomass · Q10 · maturity`` (mol C day⁻¹).

    Proportional to standing ``biomass`` (mol C; ``Σ(leaf + stem + root)`` in the
    Step-11 season) and to the :func:`q10_factor` response. ``maturity`` ∈ [0, 1] is the
    deferred development / senescence down-scaling seam (default 1.0). Returns 0 at
    ``biomass = 0`` — self-limiting, so positivity is structural (no backstop
    dependence; P3).
    """
    return (
        params.maintenance_coef
        * biomass
        * q10_factor(temp_c, q10=params.q10, t_ref=params.t_ref)
        * maturity
    )


def available_for_growth(gross: float, maintenance: float) -> float:
    """Assimilate available for growth after maintenance: ``max(0, GASS − MRES)``.

    The maintenance-first carbon budget (McCree–de Vries–Thornley; the WOFOST
    ``ASRC = GPHOT − MRES`` invariant): the assimilate left once maintenance is paid,
    clamped at 0 (no negative growth). This single expression is **shared** by
    :func:`growth_respiration_flux` (``GRES = (1−Yg)·available``) and Step-9 allocation
    (``DMI = Yg·available``) so the growth-respiration and allocation flows agree on the
    same budget *by construction* — recomputing it independently would risk a 3-way
    budget drift (assimilation/growth-resp/allocation), the one genuine cross-flow
    hazard.
    """
    return max(0.0, gross - maintenance)


def growth_respiration_flux(
    gross: float, maintenance: float, *, growth_efficiency: float
) -> float:
    """Daily growth respiration ``(1 − Yg) · max(0, GASS − MRES)`` (mol C day⁻¹).

    The maintenance-first conversion loss (McCree–de Vries–Thornley): growth
    respiration acts on the assimilate left **after** maintenance
    (:func:`available_for_growth`). The ``max(0, …)`` clamp keeps the *sink* flow a sink
    — when ``MRES ≥ GASS`` there is no growth, so growth respiration is 0 rather than a
    (carbon-creating) negative withdrawal.
    """
    return (1.0 - growth_efficiency) * available_for_growth(gross, maintenance)
