"""Penman–Monteith transpiration + root uptake (Phase-1 Step 7; the first WATER flows).

The first **WATER**-currency process (P1, single-currency): potential transpiration
from weather via the Penman–Monteith combination equation, made *actual* by a soil-
water stress factor, plus the irrigation input that refills the soil-water pool. Two
flows over a single ``soil_water`` POOL — and **no ``plant_water`` pool**:

* **Root uptake ≡ transpiration (the 3→2 deviation from the plan inventory).** The
  foundation inventory lists root uptake and transpiration as separate flows through a
  ``plant_water`` intermediate. Step 7 collapses them: in a single-bucket model the
  soil-water state *is* the water state (the WOFOST ``TRA`` convention — transpiration
  is both the uptake and the loss), and the "how much the roots can extract" physics
  lives in :func:`water_stress_factor`, not a separate leg. A ``plant_water`` pool would
  be written by uptake and drained by transpiration but **read by nothing** — no
  Phase-1 limiter reads *plant* water (``f_water`` reads *soil* water availability), so
  it would be a stock with no consumer. Contrast nitrogen (Step 10), where a
  ``plant_n`` pool *is* justified (leaf-N concentration drives ``f_N``).

* **Transpiration** — ``soil_water -> vapor_sink``, balanced in WATER:

      ``actual = penman_monteith_transpiration(weather) · water_stress_factor(soil)``

  (mm day⁻¹ → kg day⁻¹ via ground area). As ``soil_water → wilting`` the stress factor
  → 0 and transpiration shuts off, so positivity is **structural** (the WATER analogue
  of Step 5's ``Γ*`` clamp / Step 6's ``max(0, …)`` — P3).

* **Irrigation** — ``water_source -> soil_water``, balanced in WATER: a scheduled
  supply (mm day⁻¹ forcing → kg day⁻¹) that refills the depleting pool.

**Penman–Monteith (Monteith 1965).** The combination equation, in consistent SI:

    ``λE = [Δ·(Rn − G) + ρ_a·c_p·VPD/r_a] / [Δ + γ·(1 + r_s/r_a)]``   (W m⁻²)

then ``λE / λ_vap · 86400 → mm day⁻¹``. ``Δ`` is the slope of the saturation-vapour
curve, ``γ`` the psychrometric constant, ``r_a``/``r_s`` the aerodynamic/surface
resistances. **The input interface is deliberately bounded:** weather (``Rn``, ``VPD``,
``T``) is forcing read via ``env.get``; ``r_a``/``r_s`` are crop params (a fixed pair —
deriving ``r_a`` from wind + canopy height + roughness is a Step-11 refinement that
would balloon the forcing set); the psychrometric constants are universal module-level
values (FAO-56), not crop coefficients. WOFOST uses Penman + a crop factor rather than
full PM — reconciling the two formulations is Step-11 behavioral tuning, not a bend in
this clean-room physics.

**Area basis (P4).** At water density 1000 kg m⁻³, **1 mm depth over 1 m² = 1 kg**, so
the absolute daily leg is ``kg day⁻¹ = T[mm day⁻¹] · ground_area[m²]`` directly — the
per-area-rate × ``ground_area`` convention (the WATER mirror of FvCB's µmol→mol factor).
``ground_area`` and the soil-water stress thresholds are scenario data (flow fields),
not crop params.

Pure stdlib only. Citations: Monteith, J.L. (1965), "Evaporation and environment",
Symp. Soc. Exp. Biol. 19:205–234; Penman, H.L. (1948), "Natural evaporation from open
water, bare soil and grass", Proc. R. Soc. Lond. A 193:120–145; Allen, R.G., Pereira,
L.S., Raes, D. & Smith, M. (1998), "Crop evapotranspiration — Guidelines for computing
crop water requirements", FAO Irrigation and Drainage Paper 56 (the operational SVP /
slope forms and the standard psychrometric constants); Tetens, O. (1930), "Über einige
meteorologische Begriffe", Z. Geophys. 6:297–309 (the saturation-vapour form).
"""

import math
from dataclasses import dataclass

from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State

# --- saturation-vapour-pressure constants (Tetens / FAO-56 eq. 11) -----------
# e_s(T) = SVP_A · exp(SVP_B · T / (T + SVP_C))  [Pa], with T in °C. FAO-56 states it
# in kPa (0.6108 kPa); SVP_A = 610.8 Pa is the same coefficient in Pa.
SVP_A: float = 610.8  # Pa
SVP_B: float = 17.27  # dimensionless
SVP_C: float = 237.3  # °C

# --- universal psychrometric constants (FAO-56 standard values, sea level) ---
# These are physics, not crop coefficients (cf. the "parameters are data" rule, which
# governs crop coefficients) — module-level like ``photosynthesis.MICROMOL_TO_MOL``.
GAMMA_PSYCHROMETRIC: float = 67.0  # γ, Pa °C⁻¹ (≈0.067 kPa °C⁻¹ at 101.3 kPa)
AIR_DENSITY: float = 1.205  # ρ_a, kg m⁻³ (~20 °C)
AIR_SPECIFIC_HEAT: float = 1013.0  # c_p, J kg⁻¹ °C⁻¹ (FAO-56)
LATENT_HEAT_VAPORIZATION: float = 2.45e6  # λ_vap, J kg⁻¹ (~20 °C, FAO-56)
SECONDS_PER_DAY: float = 86400.0

# Area-conversion identity: at water density 1000 kg m⁻³, 1 mm of depth over 1 m² is
# exactly 1 kg, so mm day⁻¹ · ground_area[m²] = kg day⁻¹ with no scaling factor.


@dataclass(frozen=True)
class TranspirationParams:
    """Loader-produced Penman–Monteith crop parameters in core-ready form.

    Mirrors ``RespirationParams``/``PhotosynthesisParams``: declared data, no magic
    numbers in the physics. The aerodynamic and surface resistances are held fixed
    (deriving ``r_a`` from wind + canopy height is a Step-11 refinement). Values are
    provisional literature-typical placeholders pending the Step-11 validation gate
    (see ``params/transpiration.yaml``).
    """

    aerodynamic_resistance: float  # r_a, s m⁻¹ (canopy ↔ atmosphere transfer)
    surface_resistance: float  # r_s, s m⁻¹ (bulk stomatal/canopy resistance)


def saturation_vapor_pressure(temp_c: float) -> float:
    """Saturation vapour pressure ``e_s = A·exp(B·T/(T+C))`` (Pa; Tetens / FAO-56).

    The Tetens form with FAO-56 coefficients (``A=610.8 Pa, B=17.27, C=237.3 °C``).
    At 20 °C ≈ 2338 Pa (FAO-56 table: 2.339 kPa).
    """
    return SVP_A * math.exp(SVP_B * temp_c / (temp_c + SVP_C))


def slope_svp(temp_c: float) -> float:
    """Slope of the saturation-vapour curve ``Δ = B·C·e_s/(T+C)²`` (Pa °C⁻¹).

    The analytic derivative of :func:`saturation_vapor_pressure` — sharing the same
    ``B``/``C`` constants rather than re-introducing FAO-56's pre-multiplied ``4098``.
    At 20 °C ≈ 144.7 Pa °C⁻¹ (FAO-56 table: 0.145 kPa °C⁻¹).
    """
    return SVP_B * SVP_C * saturation_vapor_pressure(temp_c) / (temp_c + SVP_C) ** 2


def penman_monteith_transpiration(
    net_radiation: float,
    vpd: float,
    temp_c: float,
    *,
    aerodynamic_resistance: float,
    surface_resistance: float,
    soil_heat_flux: float = 0.0,
) -> float:
    """Potential transpiration ``mm day⁻¹`` from the PM combination equation (Monteith).

    ``net_radiation`` (``Rn``) is the daily-average net radiation (W m⁻²), ``vpd`` the
    vapour-pressure deficit (Pa), ``temp_c`` the air temperature (°C, driving ``Δ``).
    ``soil_heat_flux`` (``G``) defaults to 0 (negligible at the daily step). Computes
    the latent-heat flux ``λE`` (W m⁻²)::

        λE = [Δ·(Rn − G) + ρ_a·c_p·VPD/r_a] / [Δ + γ·(1 + r_s/r_a)]

    then converts to a depth rate via ``λE / λ_vap · 86400`` (W m⁻² → kg s⁻¹ m⁻² →
    mm s⁻¹ at ρ_water = 1000 kg m⁻³ → mm day⁻¹). Raises ``ValueError`` for a
    non-positive ``aerodynamic_resistance`` (a zero would divide by zero).

    The result is clamped to ``>= 0`` (the demand-side analogue of Step 5's ``Γ*``
    and Step 6's ``max(0, …)`` clamps). ``λE`` goes negative only when ``Rn − G < 0``
    — daily-average net radiation is negative on short midwinter days (and the
    winter-wheat season overwinters), where a negative *potential* would flip this
    sink into a deposit into ``soil_water`` and a withdrawal from the clamped vapour
    sink (dew/condensation is not modelled in Phase 1). Clamping keeps transpiration
    one-directional; ``f_water`` separately closes the *supply* side (soil depletion).
    """
    if not aerodynamic_resistance > 0.0:
        raise ValueError(
            f"aerodynamic_resistance must be > 0 s/m, got {aerodynamic_resistance!r}"
        )
    delta = slope_svp(temp_c)
    available_energy = net_radiation - soil_heat_flux
    aerodynamic_term = AIR_DENSITY * AIR_SPECIFIC_HEAT * vpd / aerodynamic_resistance
    denominator = delta + GAMMA_PSYCHROMETRIC * (
        1.0 + surface_resistance / aerodynamic_resistance
    )
    latent_flux = (delta * available_energy + aerodynamic_term) / denominator
    return max(0.0, latent_flux / LATENT_HEAT_VAPORIZATION * SECONDS_PER_DAY)


def water_stress_factor(
    soil_water: float, *, sw_wilting: float, sw_critical: float
) -> float:
    """Soil-water stress factor ``f_water ∈ [0, 1]`` (available-water fraction).

    Linear between the wilting point and a critical point: 0 at/below ``sw_wilting``,
    a ramp to 1 over ``[sw_wilting, sw_critical]``, and 1 at/above ``sw_critical``. As
    ``soil_water → sw_wilting`` transpiration shuts off (structural positivity, P3).
    The thresholds are scenario/soil data (rooting depth + soil type), passed as call
    args like ``ground_area`` — not crop params. Raises ``ValueError`` if the band is
    not strictly positive (``sw_wilting < sw_critical``).
    """
    if not sw_wilting < sw_critical:
        raise ValueError(
            f"require sw_wilting < sw_critical, got ({sw_wilting!r}, {sw_critical!r})"
        )
    if soil_water <= sw_wilting:
        return 0.0
    if soil_water >= sw_critical:
        return 1.0
    return (soil_water - sw_wilting) / (sw_critical - sw_wilting)


@dataclass(frozen=True)
class Transpiration:
    """WATER flow ``soil_water -> vapor_sink`` (Penman–Monteith; balanced in water, P1).

    Reads net radiation, VPD, and air temperature as scalar drivers through ``env.get``
    (forcing or shared stock — the flow cannot tell, #16). The potential PM rate is made
    actual by :func:`water_stress_factor` on the step-entry ``soil_water`` amount, so
    the flow self-limits as the pool depletes. ``flux = potential · f_water ·
    ground_area · dt`` (mm day⁻¹ · m² = kg day⁻¹ via the density identity) — dt-linear
    (the daily rate is dt-independent), so the RK4 increment-form contract holds. The
    ``sw_wilting``/``sw_critical`` thresholds and ``ground_area`` are scenario data.
    """

    id: FlowId
    priority: int
    soil_water: StockId
    vapor_sink: StockId
    rn_var: str
    vpd_var: str
    temp_var: str
    params: TranspirationParams
    ground_area: float
    sw_wilting: float
    sw_critical: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        net_radiation = env.get(self.rn_var)
        vpd = env.get(self.vpd_var)
        temp_c = env.get(self.temp_var)
        soil_water = snapshot.stocks[self.soil_water].amount
        potential = penman_monteith_transpiration(
            net_radiation,
            vpd,
            temp_c,
            aerodynamic_resistance=self.params.aerodynamic_resistance,
            surface_resistance=self.params.surface_resistance,
        )
        f_water = water_stress_factor(
            soil_water, sw_wilting=self.sw_wilting, sw_critical=self.sw_critical
        )
        daily_kg = potential * f_water * self.ground_area
        flux = daily_kg * dt
        return FlowResult(
            legs=(Leg(self.soil_water, -flux), Leg(self.vapor_sink, flux))
        )


@dataclass(frozen=True)
class Irrigation:
    """WATER flow ``water_source -> soil_water`` (scheduled supply; balanced, P1).

    Reads an irrigation depth rate (mm day⁻¹) as a scalar driver through ``env.get``
    (a forcing schedule). ``flux = rate · ground_area · dt`` (mm day⁻¹ · m² = kg day⁻¹
    via the density identity) — dt-linear. Refills the depleting ``soil_water`` POOL
    from an unclamped boundary supply, so the season's water balance closes (#13).
    """

    id: FlowId
    priority: int
    water_source: StockId
    soil_water: StockId
    irrigation_var: str
    ground_area: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        rate_mm_day = env.get(self.irrigation_var)
        daily_kg = rate_mm_day * self.ground_area
        flux = daily_kg * dt
        return FlowResult(
            legs=(Leg(self.water_source, -flux), Leg(self.soil_water, flux))
        )
