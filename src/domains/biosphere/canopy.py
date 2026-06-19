"""Beer–Lambert canopy light interception (Phase-1 Step 4; Monsi & Saeki 1953).

The first biological process — but **not** a mass flow and **not** an aux
accumulator. It is the **canopy diagnostic**: a pair of pure, derived quantities
computed on demand from leaf carbon (P2: "LAI is derived, not stored"). It feeds
photosynthesis (Step 5) — absorbed PAR is the incident PAR times the intercepted
fraction — and changes no ``State``.

**The two derived quantities.**

* **Leaf area index** ``LAI = leaf_carbon · sla_per_mol_c / ground_area`` — leaf
  area per unit ground area (dimensionless m²/m²). The currency is leaf carbon
  (mol C); ``sla_per_mol_c`` (m² leaf per mol leaf C) folds the conventional
  specific leaf area (m²/kg DM) with the kg-DM⇄mol-C carbon fraction at the config
  boundary (see ``domains.biosphere.loader.load_canopy_params``), so this core stays
  pure and never holds the molar-mass constant (the Step-1 lock).
* **Intercepted fraction** ``f_int = 1 − exp(−k · LAI)`` — the Beer–Lambert /
  Monsi & Saeki (1953) extinction law: ``I/I₀ = exp(−k·LAI)`` transmitted, so
  ``1 − exp(−k·LAI)`` is intercepted by the canopy. ``k`` is the dimensionless
  canopy extinction coefficient. ``f_int ∈ [0, 1)``: 0 at no leaf area, → 1 as the
  canopy closes — exactly the shape of a P1 limitation factor.

**Area basis (P4).** ``ground_area`` (m²) is **scenario** data, not crop data, so it
is a call argument, not a ``CanopyParams`` field or a ``canopy.yaml`` entry. Note its
role here is a **divisor** (LAI = leaf area ÷ ground area) — the mirror of the
per-area-rate × ground_area *multiply* that physiological flows use to turn a per-m²
rate into an absolute leg.

Pure stdlib only. Citation: Monsi, M. & Saeki, T. (1953), "Über den Lichtfaktor in
den Pflanzengesellschaften und seine Bedeutung für die Stoffproduktion", Japanese
Journal of Botany 14:22–52 (English translation: Annals of Botany 95:549–567, 2005).
"""

import math
from dataclasses import dataclass


@dataclass(frozen=True)
class CanopyParams:
    """Loader-produced canopy parameters in core-ready (mol-C, dimensionless) form.

    Mirrors ``DemoParams``: declared data, no magic numbers in the physics. The
    config boundary (``load_canopy_params``) validates the param file and **folds**
    the conventional specific leaf area (m²/kg DM) and carbon fraction (kg C/kg DM)
    into ``sla_per_mol_c`` (m² per mol leaf C), so the pure physics below never sees
    kg-DM or the molar-mass constant.
    """

    sla_per_mol_c: float  # m² leaf area per mol leaf carbon
    extinction_coef: float  # k, dimensionless Beer–Lambert canopy extinction


def leaf_area_index(
    leaf_carbon: float, *, sla_per_mol_c: float, ground_area: float
) -> float:
    """Leaf area index ``LAI = leaf_carbon · sla_per_mol_c / ground_area`` (m²/m²).

    ``leaf_carbon`` is the absolute leaf-carbon amount (mol C) over ``ground_area``
    (m², the scenario footprint — a divisor here, P4). Raises ``ValueError`` for a
    non-positive ``ground_area`` (a zero/negative footprint is meaningless and would
    divide by zero).
    """
    if not ground_area > 0.0:
        raise ValueError(f"ground_area must be > 0 m², got {ground_area!r}")
    return leaf_carbon * sla_per_mol_c / ground_area


def intercepted_fraction(lai: float, *, extinction_coef: float) -> float:
    """Fraction of incident PAR intercepted: ``1 − exp(−k · LAI)`` (Monsi & Saeki).

    ``lai`` is the leaf area index (dimensionless); ``extinction_coef`` is the
    canopy extinction coefficient ``k`` (dimensionless). Returns a value in ``[0, 1)``
    for ``lai ≥ 0, k > 0``: 0 at ``lai = 0``, approaching 1 as ``k·LAI`` grows. The
    parameter bounds (``k > 0``) are enforced at the config boundary; this stays a
    bare rate law.
    """
    return 1.0 - math.exp(-extinction_coef * lai)
