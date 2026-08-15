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

* **Canopy aggregator** ``canopy_assimilation`` — a **three-point Gaussian depth
  integral** (Goudriaan 1986) over the Beer–Lambert light profile, evaluated on one
  integration *window*. ``Ag`` is concave in PAR (saturating ``J``, then ``min``), so
  evaluating at a mean PAR **overestimates** the true integral (Jensen) — exactly why
  WOFOST does the intra-canopy/diurnal Gaussian, and why this function no longer
  evaluates at a mean.

  ⚠ **The diurnal half of that bias is now closed by the forcing, not by a quadrature
  scheme** (post-roadmap, 2026-08-14). This function used to be handed one
  *daytime-mean* PAR and the *photoperiod* as its window, which made it the whole day's
  assimilation in one evaluation and left it structurally unable to say that the sun
  sets. It is now handed the PAR of a **step-sized window** by
  ``light_path.half_sine_window_mean`` and the window's own length, so stepping the
  season performs the diurnal integral directly, at the step's resolution, and PAR is
  **0** at night. The deferred Step-11 diurnal Gaussian is therefore discharged by a
  different route than the one it named — it was a fast approximation to an integral
  this tree now takes.

  ⚠ **The intra-canopy half closed 2026-08-15, and by the route it named** (the
  quadrature above). ⚠ Its sign was the trap: the direction plan booked a layered
  canopy as something that could move canopy assimilation **up**, and this docstring
  had said the opposite since Step 5. Concavity runs one way — resolving depth
  redistributes the same photons onto a concave response and can only **lower** the
  sum. Measured on the real season: 0 of 2598 lit calls came back above the big leaf.

**Temperature (the WOFOST TMPFTB idiom).** Photosynthesis is strongly temperature-
limited; FvCB at a single reference temperature would assimilate near-max through a
sub-zero winter. ``temperature_factor`` is a multiplicative cardinal-temperature
response of the assimilation rate (a populated ``Π fᵢ`` factor). Full Arrhenius
``Vcmax(T)/Jmax(T)/Γ*(T)`` scaling is deferred (Step-11 refinement). ``f_water`` and
``f_N`` (their processes land at Steps 7/10) stay 1.0 with the ``Π fᵢ`` seam in place.

**Area basis (P4).** Leaf-level rates are per m² *leaf*; the aggregator multiplies by
LAI (leaf area per ground area) and by the scenario ``ground_area`` (m²) and the
integration window ``window_s`` (s), then converts µmol → mol, to yield an **absolute**
mol-C flux over that window — the canonical per-area-rate × ground_area convention.
Callers on the daily budget (``carbon_budget.CarbonContext.budget``) pass one **day** of
seconds, so what comes back is the mol C day⁻¹ rate *at this step's PAR*, which the
flows then multiply by ``dt``.

Pure stdlib only. Citation: Farquhar, G.D., von Caemmerer, S. & Berry, J.A. (1980),
"A biochemical model of photosynthetic CO₂ assimilation in leaves of C3 species",
Planta 149:78–90.
"""

import math
from dataclasses import dataclass

from domains.biosphere.canopy import CanopyParams

# µmol → mol (the leaf-level FvCB convention is µmol CO₂; the canonical CARBON unit is
# mol). 1 mol CO₂ assimilated == 1 mol C fixed.
MICROMOL_TO_MOL: float = 1.0e-6

# **Three-point Gaussian integration over canopy depth** (Goudriaan 1986) — the cited
# scheme for the intra-canopy integral, and the one WOFOST uses. The abscissae are the
# standard Gauss–Legendre nodes mapped to [0, 1] (``0.5 ± 0.5·√0.6``, i.e. ≈ 0.1127,
# 0.5, 0.8873) and the weights are ``5/18, 8/18, 5/18``; together they integrate a
# 5th-degree polynomial exactly, which is why three evaluations track the exact depth
# integral of the Beer–Lambert profile to < 0.2 % over the whole (LAI, PAR) range this
# tree visits.
#
# ⚠ These are **derived, not transcribed**: ``0.5 - 0.5*sqrt(0.6)`` is computed the same
# way in the Rust mirror so the two ports agree bit-for-bit on correctly-rounded IEEE
# ``sqrt``, where a hand-typed decimal (the literature prints 0.1127016654) would not.
# ⚠ The tuple order is **canonical and load-bearing**: it is the reduction order of the
# depth sum, and reductions are ordered by contract in this project (determinism).
_GAUSS_DEPTHS: tuple[float, float, float] = (
    0.5 - 0.5 * math.sqrt(0.6),
    0.5,
    0.5 + 0.5 * math.sqrt(0.6),
)
_GAUSS_WEIGHTS: tuple[float, float, float] = (5.0 / 18.0, 8.0 / 18.0, 5.0 / 18.0)


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


def canopy_assimilation(
    incident_par: float,
    lai: float,
    ci: float,
    temp_c: float,
    window_s: float,
    *,
    params: PhotosynthesisParams,
    canopy: CanopyParams,
    ground_area: float,
    limitation: float = 1.0,
) -> float:
    """Gross canopy assimilation over one window (absolute mol C) — depth-resolved.

    Aggregates the leaf-level FvCB to a ground-area-absolute carbon flux over a window
    of ``window_s`` seconds during which ``incident_par`` is taken as constant:

    1. PAR absorbed **per unit leaf area at cumulative canopy depth** ``L`` is
       ``k·incident_par·exp(−k·L)`` — the derivative of the Beer–Lambert profile
       (Step 4) with respect to depth, so integrating it over ``L ∈ [0, LAI]`` returns
       exactly ``incident_par·(1 − exp(−k·LAI)) = incident_par · f_int``, the total the
       canopy absorbs. **The depth resolution redistributes photons; it creates and
       destroys none.**
    2. Gross leaf rate at each of three Gaussian depths
       (:func:`gross_leaf_assimilation`), combined by ``LAI · Σ wᵢ·Ag(Lᵢ)`` into a
       canopy rate per ground area.
    3. ``× window_s`` ``× ground_area`` (m²) ``× 1e-6`` (µmol→mol)
       ``× f_temp(temp_c)`` ``× limitation`` (the ``f_water·f_N`` seam).

    ⚠ **This was a big leaf at the canopy-mean PAR until 2026-08-15**, i.e. one
    evaluation of ``Ag`` at ``incident_par · f_int / LAI``. Because ``Ag`` is *concave*
    in PAR, that form is a **Jensen high-bias**: it reads the whole canopy at a light
    level the sunlit top actually exceeds and the shaded base never sees, and a concave
    response evaluated at the mean exceeds the mean of the response. The bias is
    near-zero in an open canopy and reaches **+13.7 %** at ``LAI 6`` under full sun —
    it scales with canopy *closure*, which is why it was invisible in every chamber
    scenario (whose canopies peak at ``LAI 0.07–0.63``) and confined to the field.

    ⚠ **``window_s`` was ``daylength_s`` until 2026-08-14, and the difference is the
    whole point of the light path.** Passing the photoperiod made this the *day's*
    assimilation, computed once from a daytime-mean PAR — a form in which the sun never
    sets and the plant's night-time gas exchange does not exist. The window is now the
    caller's integration window (the daily budget passes one day of seconds and gets a
    per-day *rate* at this step's PAR), and the day/night structure lives in the PAR
    forcing (``light_path``). ⚠ It follows that ``window_s`` must **not** be fed a
    photoperiod any more: doing so would multiply the day-length in twice.

    **Both halves of the Jensen bias are now closed, by different routes.** The
    *diurnal* half is taken by the forcing — ``light_path`` hands this function the PAR
    of a step-sized window, so stepping the season performs the within-day integral at
    the step's resolution. The *intra-canopy* half is taken here, by the quadrature the
    Step-11 note always named. What remains unresolved is the sunlit/shaded split
    *within* a depth (direct-beam vs diffuse), which Beer–Lambert does not represent.

    Raises ``ValueError`` for non-positive ``ground_area`` or ``window_s`` and for
    a negative ``lai`` (a meaningless geometry).
    """
    if not ground_area > 0.0:
        raise ValueError(f"ground_area must be > 0 m², got {ground_area!r}")
    if not window_s > 0.0:
        raise ValueError(f"window_s must be > 0 s, got {window_s!r}")
    if lai < 0.0:
        raise ValueError(f"lai must be >= 0, got {lai!r}")
    if lai == 0.0:
        return 0.0
    k = canopy.extinction_coef
    # Canonical (fixed-tuple) reduction order — see ``_GAUSS_DEPTHS``.
    weighted_leaf_rate = 0.0
    for depth, weight in zip(_GAUSS_DEPTHS, _GAUSS_WEIGHTS, strict=True):
        absorbed_par = k * incident_par * math.exp(-k * depth * lai)
        weighted_leaf_rate += weight * gross_leaf_assimilation(
            ci, absorbed_par, params=params
        )
    canopy_rate = weighted_leaf_rate * lai  # µmol CO₂ m⁻²(ground) s⁻¹
    f_temp = temperature_factor(
        temp_c,
        t_min=params.t_min,
        t_opt_lo=params.t_opt_lo,
        t_opt_hi=params.t_opt_hi,
        t_max=params.t_max,
    )
    return canopy_rate * window_s * ground_area * MICROMOL_TO_MOL * f_temp * limitation
