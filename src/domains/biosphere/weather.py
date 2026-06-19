"""Weather → flow-driver conversions (Phase-1 Step 11; clean-room, cited).

The single-producer season is driven by daily weather. The committed oracle weather
fixture holds **raw NASAPower facts** (global shortwave ``IRRAD`` J m⁻² day⁻¹, air
temperature °C, vapour pressure ``VAP`` hPa, latitude, day-of-year — facts,
license-clean per ``docs/reuse-and-licenses.md``); this module turns those into the
**scalar drivers** the flows read (incident PAR, net radiation, VPD, photoperiod). The
conversions are **our** clean-room equations (cited to standard sources), so they live
in ``src`` where the purity gate and review see them — not in the PCSE-adjacent runner.

Pure stdlib only. Citations:
  * McCree, K.J. (1972), "Test of current definitions of photosynthetically active
    radiation against leaf photosynthesis data", Agric. Meteorol. 10:443–453 (the
    PAR energy fraction of global shortwave and the ~4.57 µmol J⁻¹ photon conversion).
  * Allen, R.G., Pereira, L.S., Raes, D. & Smith, M. (1998), "Crop evapotranspiration",
    FAO Irrigation and Drainage Paper 56 (solar declination eq. 24, sunset hour angle
    eq. 25, daylight hours eq. 34; net shortwave ``Rns = (1−α)·Rs`` with α = 0.23).
"""

import math

from domains.biosphere.transpiration import saturation_vapor_pressure

SECONDS_PER_DAY: float = 86400.0

# PAR is ~50% of global shortwave by energy (McCree 1972; FAO uses 0.5).
PAR_ENERGY_FRACTION: float = 0.5
# PAR photon flux per unit PAR energy: ~4.57 µmol photons per J (PAR band; McCree 1972).
PAR_UMOL_PER_J: float = 4.57
# FAO-56 reference-crop albedo (net shortwave Rns = (1 − α)·Rs).
ALBEDO: float = 0.23


def daylength_seconds(latitude_deg: float, day_of_year: int) -> float:
    """Astronomical daylight duration (s) from latitude + day-of-year (FAO-56).

    Solar declination ``δ = 0.409·sin(2π·doy/365 − 1.39)`` (eq. 24), sunset hour angle
    ``ωs = arccos(−tan φ·tan δ)`` (eq. 25), daylight hours ``N = 24/π·ωs`` (eq. 34). The
    ``arccos`` argument is clamped to ``[−1, 1]`` so polar day/night (``|tan·tan| > 1``)
    yields a full 24 h / 0 h rather than a domain error. At the equator this is exactly
    12 h (``43200 s``) for every day (``ωs = π/2``).
    """
    phi = math.radians(latitude_deg)
    decl = 0.409 * math.sin(2.0 * math.pi * day_of_year / 365.0 - 1.39)
    arg = -math.tan(phi) * math.tan(decl)
    arg = max(-1.0, min(1.0, arg))  # clamp for polar latitudes
    sunset_hour_angle = math.acos(arg)
    daylight_hours = 24.0 / math.pi * sunset_hour_angle
    return daylight_hours * 3600.0


def incident_par(irrad_j_m2_day: float, daylength_s: float) -> float:
    """Daytime-mean incident PAR photon flux (µmol photons m⁻² s⁻¹) from daily IRRAD.

    ``PAR_ENERGY_FRACTION · IRRAD`` is the day's PAR energy (J m⁻² day⁻¹); dividing by
    the **daylight** seconds (not 24 h — photosynthesis only runs in daylight) gives a
    mean PAR irradiance (W m⁻²), and ``× PAR_UMOL_PER_J`` converts to a photon flux. It
    is the per-second PAR the big-leaf FvCB aggregator expects (it re-multiplies by
    ``daylength_s`` for the daily integral). Returns 0 at zero radiation; raises on a
    non-positive daylength (a meaningless integration window).
    """
    if not daylength_s > 0.0:
        raise ValueError(f"daylength_s must be > 0, got {daylength_s!r}")
    if irrad_j_m2_day < 0.0:
        raise ValueError(f"irrad must be >= 0, got {irrad_j_m2_day!r}")
    mean_par_irradiance = (
        PAR_ENERGY_FRACTION * irrad_j_m2_day / daylength_s
    )  # W m⁻² PAR
    return mean_par_irradiance * PAR_UMOL_PER_J


def net_radiation(irrad_j_m2_day: float) -> float:
    """Daily-mean net radiation (W m⁻²) ≈ net shortwave ``(1 − α)·Rs`` (FAO-56).

    ``Rs = IRRAD / 86400`` is the 24 h-mean global shortwave (W m⁻²); ``(1 − ALBEDO)``
    is the net-shortwave fraction. **Net longwave is neglected** (a documented Step-11
    refinement seam) — Phase 1's potential-production run keeps ``f_water = 1``, so the
    transpiration magnitude this feeds is not on the validation path. Returns 0 at zero
    radiation; raises on negative input.
    """
    if irrad_j_m2_day < 0.0:
        raise ValueError(f"irrad must be >= 0, got {irrad_j_m2_day!r}")
    shortwave = irrad_j_m2_day / SECONDS_PER_DAY  # W m⁻² (24 h mean)
    return (1.0 - ALBEDO) * shortwave


def vapor_pressure_deficit(temp_c: float, vap_hpa: float) -> float:
    """Vapour-pressure deficit (Pa) from air temperature + actual vapour pressure.

    ``VPD = e_s(T) − e_a`` where ``e_s`` is the saturation vapour pressure (the Tetens /
    FAO-56 form shared with ``transpiration.saturation_vapor_pressure``, Pa) and ``e_a``
    is the actual vapour pressure (NASAPower ``VAP`` is hPa ⇒ ``× 100`` to Pa). Clamped
    to ``>= 0`` (VPD is non-negative; a daily-mean ``e_a`` can marginally exceed
    ``e_s(T_mean)`` on humid days). Raises on negative ``VAP``.
    """
    if vap_hpa < 0.0:
        raise ValueError(f"vap_hpa must be >= 0, got {vap_hpa!r}")
    e_a = vap_hpa * 100.0  # hPa -> Pa
    return max(0.0, saturation_vapor_pressure(temp_c) - e_a)
