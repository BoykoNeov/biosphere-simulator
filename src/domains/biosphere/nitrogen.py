"""Nitrogen uptake + limitation (Phase-1 Step 10; the last of the seven processes).

The **NITROGEN**-currency process (P1, single-currency) and the structural mirror of
Step 7 (water): a depletable soil-N pool drained by a self-limiting uptake flow and
refilled by a supply flow, plus the ``f_N`` stress factor wired into the photosynthesis
``Π fᵢ`` limiter at Step 11 (via ``CarbonContext.limitation`` in
``domains.biosphere.carbon_budget``; delivered + unit-tested standalone here).

* **N uptake** — ``soil_n -> plant_n``, balanced in NITROGEN:

      ``actual = max_uptake_capacity · ground_area · soil_n_availability(soil_n)``

  (kg N m⁻² day⁻¹ → kg N day⁻¹ via ground area). As ``soil_n → sn_residual`` the
  availability factor → 0 and uptake shuts off, so positivity is **structural** (the
  NITROGEN analogue of transpiration's ``water_stress_factor`` → 0 at wilting, P3).

* **Fertilization** — ``n_source -> soil_n``, balanced in NITROGEN: a scheduled supply
  (kg N m⁻² day⁻¹ forcing → kg N day⁻¹) that refills the depleting pool — the
  ``Irrigation`` mirror.

**Uptake is DEMAND-DEFICIT (the fixed-flux lock is gone — post-roadmap, the N-cycle form
gap).** Phase 1 shipped a max *capacity* gated only by availability, which ignored plant
need by construction and left N-limitation to arise by **dilution** alone; the
``target_conc·biomass − plant_n`` form it named as a Step-11 seam is now the flow. The
uptake flow consequently *does* read biomass (it enters the consistency web the Step-11
checklist manages), and the target concentration is **Greenwood's published curve**
rather than a free parameter — see :func:`target_n_concentration`.

**Why this changed at all: the shedding side had no citable form.** The paired change is
N:C-coupled shedding in ``mineralization`` (nitrogen leaves the plant with the carbon it
was in, at a *cited* residual concentration, instead of at a bare 1/day rate no source
publishes). The two are **one change, not two**: coupled shedding removes the
``max_uptake_capacity / n_senescence_rate`` equilibrium that used to pin ``plant_n``, so
a *capacity*-driven uptake alongside it would let ``plant_n`` grow without bound. See
``docs/plans/post-roadmap-nitrogen-cycle-form.md``.

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
    # Greenwood 1990 eqn (6) target-concentration curve (the demand-deficit target).
    n_target_coefficient: float  # ``a`` — kg N/kg DM at/below the plateau bound
    n_target_exponent: float  # ``b`` — the dilution exponent (dimensionless)
    n_target_w_plateau: float  # the curve's lower domain bound (t DM/ha)
    # kg DM per mol C (= M_C / carbon_fraction), loader-folded so the pure core never
    # holds the molar mass — the ``sla_per_mol_c`` precedent, reused for the t/ha basis.
    dm_kg_per_mol_c: float


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


def target_n_concentration(
    w_t_ha: float, *, coefficient: float, exponent: float, w_plateau: float
) -> float:
    """Greenwood's target whole-crop N concentration (kg N / kg DM) at crop mass ``W``.

    Greenwood et al. (1990) eqn (6), read first-hand off the primary::

        %N = a · W^-b        for W > 1.0 t ha⁻¹     (a = 5.697 for C3, b = 0.5)
        %N = a               for W ≤ 1.0 t ha⁻¹     (the plateau — see below)

    **The plateau is the paper's own statement, not an interpolation of ours**, and this
    is the load-bearing reading: *"Data obtained with W less than 1 t ha⁻¹ were always
    omitted"*, and *"The value of `a` is the %N in the crop when W = 1 t ha⁻¹. At this
    weight the growth rate gradually changes from being almost exponential to linear.
    **When growth is exponential plant %N remains constant and the critical
    concentration does not change with increase in plant mass** (Ågren, 1985). a = 5.7 %
    is therefore the best estimate of %N needed in the dry matter of **young tissue** to
    permit the maximum growth rate of C3 crops."* So below the bound the curve is not
    extrapolated — the primary supplies a constant, with a mechanism and a citation.

    Extrapolating the *declining* branch below the bound is what an earlier draft of
    this work did, and it is the one form the paper contradicts (see the plan doc,
    finding 9): it manufactures a late-season decline for crops an order of magnitude
    too small to have one.

    ``W`` is the dry mass of *"the whole plant (excluding fibrous roots)"* in t ha⁻¹ —
    Greenwood's own definition, which is **not** ``f_N``'s denominator (see
    :class:`NitrogenUptake` for the two-pool delta this leaves, measured not glossed).

    Raises ``ValueError`` if ``w_plateau`` is not strictly positive (``W^-b`` and the
    domain test are both meaningless at or below zero).
    """
    if not w_plateau > 0.0:
        raise ValueError(f"require n_target_w_plateau > 0, got {w_plateau!r}")
    if w_t_ha <= w_plateau:
        return coefficient
    return coefficient * w_t_ha**-exponent


@dataclass(frozen=True)
class NitrogenUptake:
    """NITROGEN flow ``soil_n -> plant_n`` (**demand-deficit**, supply-gated; balanced).

    **The fixed-flux lock is gone.** Uptake is now the WOFOST demand-deficit form the
    Phase-1 docstring named as a seam: the plant draws what it *needs* to reach its
    target
    tissue concentration, capped by what the soil can supply::

        target = target_n_concentration(W)          (Greenwood eqn (6); kg N/kg DM)
        deficit = max(0, target · biomass_c − plant_n)                      (kg N)
        capacity = max_uptake_capacity · ground_area · soil_n_availability   (kg N/day)
        flux = min(deficit, capacity) · dt

    Positivity is doubly structural: the deficit clamps at 0 once the target is met, and
    :func:`soil_n_availability` → 0 as ``soil_n → sn_residual``. N limitation therefore
    no longer arises *only* by dilution against a fixed flux — it arises when supply
    cannot meet demand, which is the mechanism the literature describes.

    **The deficit is a stock (kg N) read as a per-day rate**, i.e. "close the shortfall
    within one day" — WOFOST's daily formulation, and dt-linear in the sense the RK4
    increment contract needs (the rate is computed from the snapshot alone, never from
    ``dt``). Its implicit rate constant is 1/day, so the biosphere's frozen ``dt = 1``
    sits exactly at ``k·dt = 1``: the **deadbeat** case, which closes the deficit in one
    step and cannot overshoot (the deficit clamps at 0 — there is no restoring force to
    oscillate against, unlike the demand-controlled ``eclss.o2_makeup`` of bucket 2's
    export-fidelity finding). At ``dt > 1`` it would over-fill; that is the ``k·dt < 1``
    family, and the frozen contract pins ``dt = 1``.

    **Two pools, one measured delta — recorded, not glossed.** Greenwood's ``W`` is
    *"the whole plant (excluding fibrous roots)"*, so the curve is evaluated on ``leaf +
    stem + storage``; the deficit is applied to ``f_N``'s own denominator (``leaf + stem
    + root``), because a target measured over a *different* pool than the limiter reads
    would build a systematic offset into the stress factor. Feeding the curve the
    root-inclusive mass instead is the tempting alternative and it is **measurably
    worse**: it drops ``f_N`` to 0.9750 in ``open_season`` (3 steps), moving a frozen
    golden, where the primary's own definition holds ``f_N ≡ 1``. That the faithful
    reading is also the invariant one is a **coincidence worth naming rather than a
    principle** — it holds on a 12 % margin (see ``n_target`` in
    ``params/nitrogen.yaml``).
    """

    id: FlowId
    priority: int
    soil_n: StockId
    plant_n: StockId
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    storage_c: StockId
    params: NitrogenParams
    ground_area: float
    sn_residual: float
    sn_critical: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        stocks = snapshot.stocks
        leaf = stocks[self.leaf_c].amount
        stem = stocks[self.stem_c].amount
        # Greenwood's W excludes fibrous roots; f_N's denominator includes them.
        w_mol_c = leaf + stem + stocks[self.storage_c].amount
        biomass_c = leaf + stem + stocks[self.root_c].amount
        dm_per_mol_c = self.params.dm_kg_per_mol_c
        # mol C → kg DM → t DM/ha  (1 kg m⁻² == 10 t ha⁻¹)
        w_t_ha = (w_mol_c * dm_per_mol_c / self.ground_area) * 10.0
        target_per_mol_c = (
            target_n_concentration(
                w_t_ha,
                coefficient=self.params.n_target_coefficient,
                exponent=self.params.n_target_exponent,
                w_plateau=self.params.n_target_w_plateau,
            )
            * dm_per_mol_c
        )
        deficit = max(0.0, target_per_mol_c * biomass_c - stocks[self.plant_n].amount)

        availability = soil_n_availability(
            stocks[self.soil_n].amount,
            sn_residual=self.sn_residual,
            sn_critical=self.sn_critical,
        )
        capacity = self.params.max_uptake_capacity * self.ground_area * availability
        flux = min(deficit, capacity) * dt
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
