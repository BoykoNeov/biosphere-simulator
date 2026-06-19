"""Nitrogen uptake + limitation (Phase-1 Step 10; the last of the seven processes).

The **NITROGEN**-currency process (P1, single-currency) and the structural mirror of
Step 7 (water): a depletable soil-N pool drained by a self-limiting uptake flow and
refilled by a supply flow, plus the ``f_N`` stress factor that Step 11 wires into the
photosynthesis ``Π fᵢ`` limiter (delivered + unit-tested standalone here, **not yet a
flow input** — exactly as Step 7 delivered :func:`water_stress_factor` without wiring
``f_water`` into :class:`GrossAssimilation`).

* **N uptake** — ``soil_n -> plant_n``, balanced in NITROGEN:

      ``actual = max_uptake_capacity · ground_area · soil_n_availability(soil_n)``

  (kg N m⁻² day⁻¹ → kg N day⁻¹ via ground area). As ``soil_n → sn_residual`` the
  availability factor → 0 and uptake shuts off, so positivity is **structural** (the
  NITROGEN analogue of transpiration's ``water_stress_factor`` → 0 at wilting, P3).

* **Fertilization** — ``n_source -> soil_n``, balanced in NITROGEN: a scheduled supply
  (kg N m⁻² day⁻¹ forcing → kg N day⁻¹) that refills the depleting pool — the
  ``Irrigation`` mirror.

**Uptake is a max *capacity* gated by availability, NOT plant demand (the fixed-flux
lock).** ``max_uptake_capacity`` ignores plant need by construction; N-limitation
arises by **dilution** (biomass outgrows the fixed supply → concentration falls →
``f_N`` drops) and by **soil depletion** (``soil_n_availability`` → 0). This keeps the
uptake flow reading **only ``soil_n``**, out of the biomass-read consistency web the
Step-11 transition checklist manages. WOFOST's demand-deficit (``target_conc·biomass −
plant_n``) is a documented, strictly-additive Step-11 refinement seam (the ``maturity``
seam shape), introduced where the web is already managed.

**The two stress factors split (vs Step 7's single double-duty function).** Step 7's
``water_stress_factor`` both limited transpiration and *was* ``f_water``. Here the two
roles read different stocks and so are two functions:

* :func:`soil_n_availability` (reads ``soil_n``) limits **uptake** (supply side); its
  thresholds are scenario/soil data (call-args like ``sw_wilting``/``sw_critical``).
* :func:`nitrogen_stress_factor` ``= f_N`` (reads ``plant_n`` + biomass) limits
  **photosynthesis** (plant status); the WOFOST critical-N-dilution idiom.

**Concentration in native currency units (the ``sla_per_mol_c`` precedent).**
``plant_n`` is kg N and biomass is mol C; leaf-N concentration is conventionally
kg N / kg DM. Rather than the pure core holding the molar mass / carbon fraction, the
**loader** pre-converts the residual/critical thresholds ``kg N/kg DM → kg N/mol C``
(``× M_C / carbon_fraction``, identical in form to ``sla_per_mol_c``), so this module
compares ``plant_n / biomass_c`` against plain-float thresholds. ``f_N`` is
**whole-plant** (one ``plant_n`` pool; leaf-specific N is deferred).

**Area basis (P4).** The per-area uptake capacity (kg N m⁻² day⁻¹) is multiplied by the
scenario ``ground_area`` (m²) inside ``evaluate`` to yield an absolute kg N day⁻¹ leg —
the canonical per-area-rate × ``ground_area`` convention (the NITROGEN mirror of FvCB's
µmol→mol and transpiration's mm→kg factors).

Pure stdlib only. Citations: the critical-N-dilution concept — Greenwood, D.J. et al.
(1990), "Decline in percentage N of C3 and C4 crops with increasing plant mass", Annals
of Botany 66:425–436; the soil-supply-gated uptake idiom — the WOFOST N-balance module
(reimplemented from the published model description, not the unlicensed param YAML).
"""

from dataclasses import dataclass

from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class NitrogenParams:
    """Loader-produced nitrogen crop parameters in core-ready form.

    Mirrors ``TranspirationParams``/``RespirationParams``: declared data, no magic
    numbers in the physics. The two concentration thresholds are **already folded to
    kg N per mol C** at the loader (from the conventional kg N/kg DM via the carbon
    fraction), so the core compares them directly against ``plant_n / biomass_c``.
    Values are provisional literature-typical placeholders pending the Step-11
    validation gate (see ``params/nitrogen.yaml``).
    """

    max_uptake_capacity: float  # max N uptake per ground area (kg N m⁻² day⁻¹)
    n_residual_per_mol_c: float  # plant-N conc at/below which f_N = 0 (kg N / mol C)
    n_critical_per_mol_c: float  # plant-N conc at/above which f_N = 1 (kg N / mol C)


def soil_n_availability(
    soil_n: float, *, sn_residual: float, sn_critical: float
) -> float:
    """Soil-N availability factor ``∈ [0, 1]`` that gates uptake (supply side).

    Linear between a residual (unextractable) amount and a critical amount: 0 at/below
    ``sn_residual``, a ramp to 1 over ``[sn_residual, sn_critical]``, and 1 at/above
    ``sn_critical``. As ``soil_n → sn_residual`` uptake shuts off (structural
    positivity, P3 — the NITROGEN analogue of :func:`water_stress_factor`). The
    thresholds are scenario/soil data (passed as call args like ``sw_wilting``), not
    crop params. Raises ``ValueError`` if the band is not strictly positive
    (``sn_residual < sn_critical``).
    """
    if not sn_residual < sn_critical:
        raise ValueError(
            f"require sn_residual < sn_critical, got ({sn_residual!r}, {sn_critical!r})"
        )
    if soil_n <= sn_residual:
        return 0.0
    if soil_n >= sn_critical:
        return 1.0
    return (soil_n - sn_residual) / (sn_critical - sn_residual)


def nitrogen_stress_factor(
    plant_n: float,
    biomass_c: float,
    *,
    n_residual_per_mol_c: float,
    n_critical_per_mol_c: float,
) -> float:
    """Plant-N stress factor ``f_N ∈ [0, 1]`` (the photosynthesis limiter; unwired).

    Linear in the whole-plant N concentration ``conc = plant_n / biomass_c`` (kg N per
    mol C): 0 at/below ``n_residual_per_mol_c``, a ramp to 1 over
    ``[n_residual_per_mol_c, n_critical_per_mol_c]``, and 1 at/above the critical
    concentration — the WOFOST critical-N-dilution idiom (a populated ``Π fᵢ`` limiter,
    Step 5's ``limitation=`` seam, wired at Step 11).

    Guards ``biomass_c <= 0`` → returns 1.0 (neutral: with no biomass there are no
    leaves, so photosynthesis is already 0 via the LAI=0 path — never a divide-by-zero).
    Raises ``ValueError`` if the band is not strictly positive
    (``n_residual_per_mol_c < n_critical_per_mol_c``).
    """
    if not n_residual_per_mol_c < n_critical_per_mol_c:
        raise ValueError(
            "require n_residual_per_mol_c < n_critical_per_mol_c, got "
            f"({n_residual_per_mol_c!r}, {n_critical_per_mol_c!r})"
        )
    if biomass_c <= 0.0:
        return 1.0
    conc = plant_n / biomass_c
    if conc <= n_residual_per_mol_c:
        return 0.0
    if conc >= n_critical_per_mol_c:
        return 1.0
    return (conc - n_residual_per_mol_c) / (n_critical_per_mol_c - n_residual_per_mol_c)


@dataclass(frozen=True)
class NitrogenUptake:
    """NITROGEN flow ``soil_n -> plant_n`` (capacity-gated; balanced in N, P1).

    The potential per-area uptake capacity is made actual by :func:`soil_n_availability`
    on the step-entry ``soil_n`` amount, so the flow self-limits as the pool depletes.
    ``flux = max_uptake_capacity · ground_area · availability · dt`` (kg N m⁻² day⁻¹ ·
    m² = kg N day⁻¹) — dt-linear (the daily rate is dt-independent), so the RK4
    increment-form contract holds. The ``sn_residual``/``sn_critical`` thresholds and
    ``ground_area`` are scenario data. ``plant_n`` is a POOL (an N reservoir, never
    zeroed-with-loss); it is **not** read here (capacity ignores plant need — the
    fixed-flux lock; demand-deficit is a Step-11 seam).
    """

    id: FlowId
    priority: int
    soil_n: StockId
    plant_n: StockId
    params: NitrogenParams
    ground_area: float
    sn_residual: float
    sn_critical: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        soil_n = snapshot.stocks[self.soil_n].amount
        availability = soil_n_availability(
            soil_n, sn_residual=self.sn_residual, sn_critical=self.sn_critical
        )
        daily_kg = self.params.max_uptake_capacity * self.ground_area * availability
        flux = daily_kg * dt
        return FlowResult(legs=(Leg(self.soil_n, -flux), Leg(self.plant_n, flux)))


@dataclass(frozen=True)
class Fertilization:
    """NITROGEN flow ``n_source -> soil_n`` (scheduled supply; balanced, P1).

    Reads an N-application rate (kg N m⁻² day⁻¹) as a scalar driver through ``env.get``
    (a forcing schedule). ``flux = rate · ground_area · dt`` (kg N m⁻² day⁻¹ · m² =
    kg N day⁻¹) — dt-linear. Refills the depleting ``soil_n`` POOL from an unclamped
    boundary supply, so the season's N balance closes (#13) — the ``Irrigation`` mirror.
    """

    id: FlowId
    priority: int
    n_source: StockId
    soil_n: StockId
    fertilization_var: str
    ground_area: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        rate_kg_m2_day = env.get(self.fertilization_var)
        daily_kg = rate_kg_m2_day * self.ground_area
        flux = daily_kg * dt
        return FlowResult(legs=(Leg(self.n_source, -flux), Leg(self.soil_n, flux)))
