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

Pure stdlib only. Citation: Farquhar, G.D. & Sharkey, T.D. (1982), "Stomatal
conductance and photosynthesis", Annu. Rev. Plant Physiol. 33:317–345 (the C3
``Ci/Ca ≈ 0.7`` regulation set point).
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
