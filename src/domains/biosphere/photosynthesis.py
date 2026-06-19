"""FvCB photosynthesis (Phase-1 Step 5; Farquhar, von Caemmerer & Berry 1980).

The first carbon **source** process: gross CO₂ assimilation into plant carbon. Two
layers, split deliberately so the citable leaf-level physics is exactly hand-checkable
and the canopy/diurnal aggregation (the part WOFOST does with a Gaussian integration)
is an isolated, additively-extendable seam:

* **Instantaneous leaf-level FvCB** — pure rate laws, per unit *leaf* area
  (µmol CO₂ m⁻² s⁻¹), checked against independent literals:
    - Rubisco-limited      ``Ac = Vcmax·(Ci − Γ*) / (Ci + Kc·(1 + O/Ko))``
    - Electron transport   ``θ·J² − (I₂ + Jmax)·J + I₂·Jmax = 0`` (smaller root),
                           ``I₂ = α·absorbed_par`` — a non-rectangular hyperbola.
    - Light/RuBP-limited   ``Aj = J·(Ci − Γ*) / (4·Ci + 8·Γ*)``
    - Gross leaf rate      ``Ag = max(0, min(Ac, Aj))``.
  ``Ac``/``Aj`` are gross-of-dark-respiration but **net-of-photorespiration** (the
  ``(Ci − Γ*)`` factor books the photorespiratory CO₂ release), so deposited carbon is
  honest gross assimilation with no hidden mass leak. The ``max(0, …)`` clamp is
  load-bearing for a *source* flow: at ``Ci ≤ Γ*`` the ``(Ci − Γ*)`` factor would flip
  ``Ag`` negative and turn the source into a withdrawal — clamp to 0 instead (P3:
  "assimilation → 0 as CO₂/light → 0").

* **Provisional canopy + diurnal aggregator** ``daily_canopy_assimilation`` — a
  **big-leaf** at daily-/canopy-mean PAR. ``Ag`` is concave in PAR (saturating ``J``,
  then ``min``), so a mean-PAR estimate **overestimates** the true daily integral
  (Jensen) — exactly why WOFOST does the intra-canopy/diurnal Gaussian. That Gaussian
  is the Step-11 refinement; ``daily_canopy_assimilation`` is the seam it extends
  *additively* (same signature). The expected high-bias is documented, not assumed
  away.

**Temperature (the WOFOST TMPFTB idiom).** Photosynthesis is strongly temperature-
limited; FvCB at a single reference temperature would assimilate near-max through a
sub-zero winter. ``temperature_factor`` is a multiplicative cardinal-temperature
response of the assimilation rate (a populated ``Π fᵢ`` factor). Full Arrhenius
``Vcmax(T)/Jmax(T)/Γ*(T)`` scaling is deferred (Step-11 refinement). ``f_water`` and
``f_N`` (their processes land at Steps 7/10) stay 1.0 with the ``Π fᵢ`` seam in place.

**Area basis (P4).** Leaf-level rates are per m² *leaf*; the aggregator multiplies by
LAI (leaf area per ground area) and by the scenario ``ground_area`` (m²) and the
photoperiod ``daylength_s`` (s), then converts µmol → mol, to yield an **absolute**
daily mol-C flux — the canonical per-area-rate × ground_area convention.

Pure stdlib only. Citation: Farquhar, G.D., von Caemmerer, S. & Berry, J.A. (1980),
"A biochemical model of photosynthetic CO₂ assimilation in leaves of C3 species",
Planta 149:78–90.
"""

import math
from dataclasses import dataclass

from domains.biosphere.canopy import CanopyParams, intercepted_fraction

# µmol → mol (the leaf-level FvCB convention is µmol CO₂; the canonical CARBON unit is
# mol). 1 mol CO₂ assimilated == 1 mol C fixed.
MICROMOL_TO_MOL: float = 1.0e-6


@dataclass(frozen=True)
class PhotosynthesisParams:
    """Loader-produced FvCB parameters in core-ready form (reference temperature).

    Mirrors ``CanopyParams``/``DemoParams``: declared data, no magic numbers in the
    physics. All values are provisional literature-typical placeholders pending the
    Step-11 validation gate (see ``params/photosynthesis.yaml``).
    """

    vcmax: float  # max Rubisco carboxylation rate (µmol CO₂ m⁻² leaf s⁻¹)
    jmax: float  # max electron transport rate (µmol e⁻ m⁻² leaf s⁻¹)
    quantum_yield: float  # α, initial slope of J vs absorbed PAR (mol e⁻ / mol photon)
    theta: float  # θ, non-rectangular-hyperbola curvature (dimensionless, 0 < θ ≤ 1)
    gamma_star: float  # Γ*, CO₂ compensation point w/o dark respiration (µmol mol⁻¹)
    kc: float  # Michaelis constant for carboxylation (µmol mol⁻¹)
    ko: float  # Michaelis constant for oxygenation (mmol mol⁻¹)
    o2: float  # O, oxygen partial pressure / mole fraction (mmol mol⁻¹)
    t_min: float  # cardinal temperatures (°C) for the assimilation response:
    t_opt_lo: float  #   0 below t_min; ramp to 1 over [t_min, t_opt_lo];
    t_opt_hi: float  #   plateau 1 over [t_opt_lo, t_opt_hi];
    t_max: float  #   ramp to 0 over [t_opt_hi, t_max]; 0 above t_max.


def rubisco_limited_rate(
    ci: float, *, vcmax: float, gamma_star: float, kc: float, ko: float, o2: float
) -> float:
    """Rubisco-limited assimilation ``Ac = Vcmax·(Ci − Γ*) / (Ci + Kc·(1 + O/Ko))``.

    All concentrations share their Michaelis-constant units (``Ci``, ``Γ*``, ``Kc`` in
    µmol mol⁻¹; ``O``, ``Ko`` in mmol mol⁻¹). May return a negative value when
    ``Ci < Γ*``; the sign is resolved (clamped) in :func:`gross_leaf_assimilation`.
    """
    return vcmax * (ci - gamma_star) / (ci + kc * (1.0 + o2 / ko))


def electron_transport_rate(
    absorbed_par: float, *, jmax: float, quantum_yield: float, theta: float
) -> float:
    """Electron transport rate ``J`` from absorbed PAR (non-rectangular hyperbola).

    Solves ``θ·J² − (I₂ + Jmax)·J + I₂·Jmax = 0`` for the smaller (physical) root,
    with ``I₂ = quantum_yield·absorbed_par``. ``J`` rises with the initial slope
    ``quantum_yield`` at low light and saturates at ``Jmax``; ``theta`` (0 < θ ≤ 1)
    sets the sharpness of the transition. Returns 0 at zero absorbed PAR.
    """
    i2 = quantum_yield * absorbed_par
    b = i2 + jmax
    # Smaller root of θJ² − bJ + I₂·Jmax: (b − sqrt(b² − 4θ·I₂·Jmax)) / (2θ).
    discriminant = b * b - 4.0 * theta * i2 * jmax
    return (b - math.sqrt(discriminant)) / (2.0 * theta)


def light_limited_rate(ci: float, j: float, *, gamma_star: float) -> float:
    """Light/RuBP-regeneration-limited assimilation ``Aj = J·(Ci − Γ*)/(4·Ci + 8·Γ*)``.

    ``j`` is the electron transport rate (:func:`electron_transport_rate`). May return
    a negative value when ``Ci < Γ*``; the sign is resolved in
    :func:`gross_leaf_assimilation`.
    """
    return j * (ci - gamma_star) / (4.0 * ci + 8.0 * gamma_star)


def gross_leaf_assimilation(
    ci: float, absorbed_par: float, *, params: PhotosynthesisParams
) -> float:
    """Gross leaf assimilation ``Ag = max(0, min(Ac, Aj))`` (µmol CO₂ m⁻² leaf s⁻¹).

    The FvCB co-limitation: the leaf runs at the smaller of the Rubisco- and light-
    limited rates. The ``max(0, …)`` clamp keeps the *source* flow a source — at
    ``Ci ≤ Γ*`` both branches go non-positive (no net carboxylation), so gross uptake
    is 0, never a withdrawal from plant carbon.
    """
    ac = rubisco_limited_rate(
        ci,
        vcmax=params.vcmax,
        gamma_star=params.gamma_star,
        kc=params.kc,
        ko=params.ko,
        o2=params.o2,
    )
    j = electron_transport_rate(
        absorbed_par,
        jmax=params.jmax,
        quantum_yield=params.quantum_yield,
        theta=params.theta,
    )
    aj = light_limited_rate(ci, j, gamma_star=params.gamma_star)
    return max(0.0, min(ac, aj))


def temperature_factor(
    temp_c: float, *, t_min: float, t_opt_lo: float, t_opt_hi: float, t_max: float
) -> float:
    """Cardinal-temperature response of assimilation ``f_temp(T) ∈ [0, 1]`` (TMPFTB).

    Piecewise-linear: 0 at/below ``t_min``, a linear ramp up to 1 over
    ``[t_min, t_opt_lo]``, a plateau at 1 over ``[t_opt_lo, t_opt_hi]``, a linear ramp
    down to 0 over ``[t_opt_hi, t_max]``, and 0 at/above ``t_max``. The cardinal
    temperatures must be non-decreasing (enforced at the config boundary). A
    multiplicative ``Π fᵢ`` limiter, not a re-derivation of FvCB's kinetic constants.
    """
    if temp_c <= t_min or temp_c >= t_max:
        return 0.0
    if temp_c < t_opt_lo:
        return (temp_c - t_min) / (t_opt_lo - t_min)
    if temp_c > t_opt_hi:
        return (t_max - temp_c) / (t_max - t_opt_hi)
    return 1.0


def daily_canopy_assimilation(
    incident_par: float,
    lai: float,
    ci: float,
    temp_c: float,
    daylength_s: float,
    *,
    params: PhotosynthesisParams,
    canopy: CanopyParams,
    ground_area: float,
    limitation: float = 1.0,
) -> float:
    """Daily gross canopy assimilation (absolute mol C day⁻¹) — provisional big-leaf.

    Aggregates the leaf-level FvCB to a daily, ground-area-absolute carbon flux:

    1. Absorbed canopy PAR per ground area ``= incident_par · f_int`` (Beer–Lambert,
       Step 4). Mean absorbed PAR per *leaf* area ``= incident_par · f_int / LAI`` —
       well-defined as LAI→0 (``f_int ≈ k·LAI``, so the ratio → ``k·incident_par``);
       guarded to 0 at exactly ``LAI = 0`` (no leaves intercept nothing).
    2. Gross leaf rate at that mean PAR (:func:`gross_leaf_assimilation`), scaled to a
       canopy rate per ground area by ``× LAI``.
    3. ``× daylength_s`` (photoperiod) ``× ground_area`` (m²) ``× 1e-6`` (µmol→mol)
       ``× f_temp(temp_c)`` ``× limitation`` (the ``f_water·f_N`` seam; 1.0 in Step 5).

    **Provisional high-bias (Jensen).** ``Ag`` is concave in PAR, so this mean-PAR
    big-leaf overestimates the true intra-canopy/diurnal integral. Closing that gap is
    the Step-11 Gaussian, which extends *this* function additively.

    Raises ``ValueError`` for non-positive ``ground_area`` or ``daylength_s`` and for
    a negative ``lai`` (a meaningless geometry).
    """
    if not ground_area > 0.0:
        raise ValueError(f"ground_area must be > 0 m², got {ground_area!r}")
    if not daylength_s > 0.0:
        raise ValueError(f"daylength_s must be > 0 s, got {daylength_s!r}")
    if lai < 0.0:
        raise ValueError(f"lai must be >= 0, got {lai!r}")
    if lai == 0.0:
        return 0.0
    f_int = intercepted_fraction(lai, extinction_coef=canopy.extinction_coef)
    mean_absorbed_par = incident_par * f_int / lai
    leaf_rate = gross_leaf_assimilation(ci, mean_absorbed_par, params=params)
    canopy_rate = leaf_rate * lai  # µmol CO₂ m⁻²(ground) s⁻¹
    f_temp = temperature_factor(
        temp_c,
        t_min=params.t_min,
        t_opt_lo=params.t_opt_lo,
        t_opt_hi=params.t_opt_hi,
        t_max=params.t_max,
    )
    return (
        canopy_rate * daylength_s * ground_area * MICROMOL_TO_MOL * f_temp * limitation
    )
