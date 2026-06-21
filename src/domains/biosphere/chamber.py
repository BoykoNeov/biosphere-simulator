"""Chamber atmosphere ↔ FvCB seam (Phase-2 Step 2; the ``Ci``-from-stock read, P2.2).

Phase 1 read intercellular CO₂ (``Ci``) as a constant **forcing** schedule. A sealed
chamber instead derives ``Ci`` from a **live, finite carbon pool**: photosynthesis
withdraws carbon, the pool's mole fraction falls, ``Ci`` falls, and FvCB assimilation
weakens — the first emergent feedback, with no control code (roadmap Phase-2; plan
P2.2). The pool amount is read through the source resolver as a *shared stock* (#16),
exactly the mechanism Phase 1 used for ``f_water`` reading ``soil_water``; this module
holds the pure amount→``Ci`` conversion that ``carbon_budget.CarbonContext`` applies to
that read.

**Honest naming (Step-2 scope).** At Step 2 the chamber carbon pool is a finite
**carbon source pool** (composition ``{CARBON: 1}``), *not* yet a molecular CO₂ stock:
the gas-exchange flows are still single-currency CARBON (they drain to the ``co2_resp``
sink, so the chamber is **not** closed — the pool is monotonic draw-down, not an
oscillating closed loop). The pool is promoted to a true CO₂ stock
(``{CARBON: 1, OXYGEN: 2}``) with an O₂ counterpart and multi-quantity legs at
**Step 3** (the gas-exchange rework). :func:`ci_from_co2_pool` reads the pool's
mol-carbon amount,
which is unchanged by that promotion — so this seam needs no rework when composition is
generalized.

**The conversion.** The chamber air is a fixed total amount ``air_mol`` (well-mixed 0-D,
roadmap). The pool's CO₂ mole fraction is ``co2_mol / air_mol`` (mol CO₂ / mol air);
expressed in µmol mol⁻¹ (the FvCB ``Ci`` unit) the ambient mole fraction is
``Ca = co2_mol / air_mol · 1e6``. Intercellular ``Ci`` tracks ambient through the C3
draw-down ratio ``Ci/Ca`` (stomatal regulation holds ``Ci ≈ 0.7·Ca`` for C3 species;
Farquhar & Sharkey 1982): ``Ci = ci_ratio · Ca``. This reproduces the Phase-1 constant
forcing at the chamber's initial fill (``Ci0 = ci_ratio · co2_mol0 / air_mol · 1e6``)
while making ``Ci`` a live function of the draw-down. The ``Ci/Ca`` ratio is held
constant (a fixed-ratio stomatal closure model); a responsive ``gs`` is a later
refinement.

**O₂ self-limitation (Phase-2 Step 7; ``f_O2``, the respiratory Ci-shutoff mirror).**
:func:`oxygen_limitation_factor` is the O₂ analogue of the Ci draw-down: a Monod factor
in the chamber O₂ **mole fraction** (``x_O2 = o2_mol / air_mol``, the same intensive
basis as ``Ca``) that throttles the O₂-consuming respiration fluxes (plant maintenance
shortfall + microbial respiration) toward 0 as O₂ → 0. It is the kinetic self-limit
P2.2 flagged: on a *depleting* O₂ pool respiration must shut off smoothly so the draw
never over-runs the pool (``rationed == 0`` from kinetics, not the Euler backstop) —
exactly mirroring how FvCB's ``(Ci − Γ*)`` factor self-limits the carbon draw. Steps
3/5 deferred it (at the ~21 % PP fill O₂ never approached its floor); it lands here,
where the canonical multi-year run is sized to deplete O₂. The half-saturation
``K_O2`` is **low/sharp** — for a well-mixed 0-D chamber there is no soil-microsite
diffusion limitation, so the relevant constant is the intrinsic respiratory O₂ affinity
(terminal-oxidase apparent ``Km`` for O₂ is sub-µM dissolved O₂ → a tiny gas-phase mole
fraction), i.e. aerobic respiration is O₂-saturated (``f_O2 ≈ 1``) until near-anoxia.
(The larger apparent ``Km`` of soil-respiration models like DAMM, Davidson et al. 2012,
reflects intra-aggregate O₂ diffusion — a regime the 0-D well-mixed chamber does not
model.)

Pure stdlib only. Citations: Farquhar, G.D. & Sharkey, T.D. (1982), "Stomatal
conductance and photosynthesis", Annu. Rev. Plant Physiol. 33:317–345 (the C3
``Ci/Ca ≈ 0.7`` regulation set point); Davidson, E.A., Samanta, S., Caramori, S.S. &
Savage, K. (2012), "The Dual Arrhenius and Michaelis–Menten kinetics model for
decomposition of soil organic matter at hourly to seasonal time scales", Global Change
Biology 18:371–384 (the Michaelis–Menten O₂-limitation form for respiration).
"""

import math

# mol/mol → µmol/mol (the FvCB ``Ci``/``Ca`` mole-fraction unit is µmol mol⁻¹).
MOLEFRAC_TO_MICRO: float = 1.0e6


def ci_from_co2_pool(co2_mol: float, *, air_mol: float, ci_ratio: float) -> float:
    """Intercellular ``Ci`` (µmol mol⁻¹) from a finite chamber carbon pool.

    ``Ca = co2_mol / air_mol · 1e6`` is the chamber's CO₂ mole fraction; ``Ci =
    ci_ratio · Ca`` applies the fixed C3 draw-down ratio (``Ci/Ca ≈ 0.7``). As
    photosynthesis withdraws carbon, ``co2_mol`` falls → ``Ci`` falls → FvCB
    assimilation weakens (the emergent P2.2 feedback; no controller).

    ``co2_mol`` is read live from the chamber pool's mol-carbon amount (a shared stock,
    #16) and is non-negative (a POOL never goes negative — arbitration + FvCB Ci-shutoff
    keep it so). ``air_mol`` (total chamber air) and ``ci_ratio`` are chamber/scenario
    data, not crop params (P4). Raises ``ValueError`` for a non-positive ``air_mol`` or
    ``ci_ratio`` (a degenerate chamber), or a non-finite/negative ``co2_mol``.
    """
    if not air_mol > 0.0:
        raise ValueError(f"air_mol must be > 0 mol, got {air_mol!r}")
    if not ci_ratio > 0.0:
        raise ValueError(f"ci_ratio must be > 0 (dimensionless), got {ci_ratio!r}")
    if not math.isfinite(co2_mol) or co2_mol < 0.0:
        raise ValueError(f"co2_mol must be finite and >= 0 mol, got {co2_mol!r}")
    ca = co2_mol / air_mol * MOLEFRAC_TO_MICRO
    return ci_ratio * ca


def oxygen_limitation_factor(o2_mol: float, *, air_mol: float, k_o2: float) -> float:
    """O₂ self-limitation ``f_O2 = x_O2 / (K_O2 + x_O2) ∈ [0, 1]`` (Step 7).

    A Monod factor in the chamber O₂ mole fraction ``x_O2 = o2_mol / air_mol`` (the same
    intensive basis as ``Ca``/``Ci``): → 1 as O₂ ≫ ``K_O2`` (O₂-saturated respiration,
    the PP regime) and → 0 as O₂ → 0 (anoxia). Multiplies the O₂-consuming respiration
    fluxes (plant maintenance shortfall, microbial respiration) so that on a depleting
    O₂ pool the draw shuts off smoothly before the pool is over-run — ``rationed == 0``
    from kinetics, the respiratory mirror of FvCB's Ci-shutoff.

    ``o2_mol`` is read live from the chamber O₂ pool. ``air_mol`` (total chamber air) is
    chamber/scenario data (P4); ``k_o2`` is the O₂ half-saturation **mole fraction** (a
    respiration kinetic param). ``k_o2 = 0`` disables the limit (``f_O2 = 1`` for any
    O₂ > 0 — the Step-3/5 "off" behaviour). A POOL is clamped ≥ 0 by arbitration, but at
    **full depletion** float rounding can leave the amount a tiny negative (dust); any
    non-positive O₂ is therefore treated as the **anoxic floor** (``f_O2 = 0``,
    respiration off) rather than rejected — so a depleting run self-limits smoothly to 0
    without tripping on dust. Raises ``ValueError`` for a non-positive ``air_mol``, a
    negative/non-finite ``k_o2``, or a non-finite ``o2_mol``.
    """
    if not air_mol > 0.0:
        raise ValueError(f"air_mol must be > 0 mol, got {air_mol!r}")
    if not (math.isfinite(k_o2) and k_o2 >= 0.0):
        raise ValueError(f"k_o2 must be finite and >= 0 (mol/mol), got {k_o2!r}")
    if not math.isfinite(o2_mol):
        raise ValueError(f"o2_mol must be finite, got {o2_mol!r}")
    x_o2 = max(0.0, o2_mol) / air_mol  # clamp depletion float-dust to the anoxic floor
    denom = k_o2 + x_o2
    if denom <= 0.0:  # k_o2 == 0 and O₂ == 0: degenerate; no O₂ ⇒ no respiration
        return 0.0
    return x_o2 / denom
