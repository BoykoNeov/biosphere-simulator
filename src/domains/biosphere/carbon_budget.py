"""The coupled daily carbon budget (Phase-1 Step 11; the buffer-dissolution rewiring).

Steps 5/6/9 built gross assimilation, the two respirations, and allocation as
standalone flows over a provisional ``plant_c`` pool that was meant to reframe at
integration into a near-zero "labile carbohydrate buffer". That reframing **does not
survive the arbitration backstop**: ``simcore.arbitration`` computes
``scale_s = stocks[s].amount / demand_s`` against the **start-of-step** amount —
"withdrawals never draw against same-step inflows" — so a pool that nets to ~0 each
step cannot source *any* same-step withdrawal (gross assimilation's same-step deposit
does not count), and the Euler backstop would fire every step. So ``plant_c`` is
**dissolved**: every carbon fate is sourced from the unclamped ``co2_atmos`` boundary
and the organ pools, where every clamped withdrawal is self-limiting (∝ the organ's
amount) and the backstop is structurally impossible. ``GrossAssimilation``-the-flow
dissolves entirely — gross assimilation ``GASS`` becomes a recomputed *quantity*.

Three flows recompute the same daily budget from the step-entry snapshot (flows cannot
read each other's results pre-arbitration), so the budget lives in **one shared place**,
:class:`CarbonContext` — held and called by all three, so they **cannot drift** on the
gross-assimilation, maintenance, or limitation computation:

* **Allocation** — ``co2_atmos -> {leaf_c, stem_c, root_c, storage_c}`` deposits the
  structural increment ``DMI = Yg·available_for_growth(GASS, MRES)``, split by the
  DVS-keyed partition fractions (Step 9). Source is the unclamped atmosphere; sinks are
  the organs.
* **GrowthRespiration** — ``co2_atmos -> co2_resp`` the growth-conversion loss
  ``GRES = (1−Yg)·available_for_growth(GASS, MRES)`` (assimilated and immediately
  respired; never becomes biomass).
* **MaintenanceRespiration** — ``{co2_atmos(covered), organs(shortfall)} -> co2_resp``.
  The **maintenance-first** budget (McCree–de Vries–Thornley): maintenance ``MRES`` is
  paid from today's assimilate where it covers it (``covered = min(GASS, MRES)`` drawn
  from ``co2_atmos``) and from standing biomass where it does not
  (``shortfall = max(0, MRES − GASS)`` drawn from the organ pools, proportional to each
  organ's share — biomass shrinks in carbon deficit, e.g. overwintering). Because
  ``covered + shortfall = MRES`` and ``GRES + DMI = available = max(0, GASS − MRES)``,
  the per-step carbon booking telescopes: ``co2_atmos`` loses exactly ``GASS`` on a
  surplus day (``DMI + GRES + covered``), organs grow by ``DMI``; on a deficit day
  ``co2_atmos`` loses ``GASS`` and organs shrink by ``shortfall``. The split is the
  **only** structure that conserves *and* avoids double-charging maintenance: drawing
  the full ``MRES`` from organs while ``available`` still subtracts it would charge
  maintenance twice (the conservation gate would NOT catch it; it balances).

**CO₂ is two stocks (the Step-11 lock).** ``co2_atmos`` (unclamped BOUNDARY) is the
non-limiting atmosphere the FvCB rate draws from (the rate reads the ``ci_var`` forcing,
never the stock amount); ``co2_resp`` (BOUNDARY) is the respiration sink accumulating
``MRES + GRES`` — the carbon mirror of transpiration's ``vapor_sink`` and senescence's
``litter_sink``. (Step 6's single-reservoir note was coupled to ``plant_c`` being the
respiration source; the dissolution removes that premise — see the plan's Step-11 CO₂
lock. A single stock would make ``GrowthRespiration`` a permanent no-op and respiration
a silent ``Yg`` factor, which plan line 348 forbids.)

**Sealed chamber — one CO₂ pool, the gas loop closed (Phase-2 Step 3).** When the carbon
source *is* the respiration sink (one finite CO₂ POOL, ``{CARBON:1, OXYGEN:2}``), the
flows detect ``co2_atmos == co2_resp`` and **net the assimilate-respired round trips**:
``GrowthRespiration`` and the *covered* part of ``MaintenanceRespiration`` (carbon
gross-assimilated and immediately respired) become CO₂→CO₂ no-ops — exactly the single-
reservoir degeneracy the open field avoids, but here it is *correct* (the carbon returns
to the same pool). What remains is the genuine multi-quantity (CARBON+OXYGEN) gas
exchange at PQ=1: :class:`Allocation` is ``CO₂ → biomass + O₂`` (an O₂ leg = the carbon
fixed) and the maintenance *shortfall* is ``biomass + O₂ → CO₂`` (O₂ consumed = the
carbon burned). The composition fold (P2.1) balances OXYGEN; the open-field
single-currency paths are byte-identical (``o2_pool=None``, distinct source/sink).

**Limitation (the ``Π fᵢ`` seam closes here).** ``CarbonContext.limitation`` forms
``f_water · f_N`` and applies it **inside** the shared budget, so every flow gets the
identical factor — no flow passes its own. ``f_water`` reads the ``soil_water`` sibling
stock via ``env.get`` (#16); ``f_N`` reads the plant's own ``plant_n`` pool (a direct
snapshot read) against the ``Σ(leaf + stem + root)`` biomass. ``storage_c`` is
**excluded** from the ``f_N``/maintenance biomass and the shortfall draw (grain is a
pure allocation sink in Phase 1 — see ``allocation.py``).

Pure stdlib only. Citations live with the rate laws they reuse (``photosynthesis.py``,
``respiration.py``, ``allocation.py``, ``transpiration.py``, ``nitrogen.py``).
"""

from dataclasses import dataclass

from domains.biosphere.allocation import AllocationParams, partition
from domains.biosphere.canopy import CanopyParams, leaf_area_index
from domains.biosphere.chamber import ci_from_co2_pool, oxygen_limitation_factor
from domains.biosphere.nitrogen import NitrogenParams, nitrogen_stress_factor
from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.photosynthesis import (
    PhotosynthesisParams,
    daily_canopy_assimilation,
)
from domains.biosphere.respiration import (
    RespirationParams,
    available_for_growth,
    maintenance_respiration_flux,
)
from domains.biosphere.transpiration import water_stress_factor
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class CarbonContext:
    """Shared inputs for the recomputed daily carbon budget ``(GASS, MRES, available)``.

    The three budget-coupled flows (:class:`Allocation`, :class:`GrowthRespiration`,
    :class:`MaintenanceRespiration`) each hold **one** of these and call
    :meth:`budget`, so they cannot drift on the gross-assimilation, maintenance, or
    limitation computation (structural agreement, not disciplined agreement — the
    advisor's Step-11 sharpening). ``leaf_c`` drives LAI/GASS; ``(leaf_c, stem_c,
    root_c)`` sum to the maintenance / ``f_N`` biomass (``storage_c`` excluded — it is a
    pure allocation sink). The forcing-var names resolve through ``env.get`` (#16);
    the scenario thresholds (``ground_area``, the soil-water band) are call/field data,
    not crop params (P4).
    """

    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    par_var: str
    ci_var: str
    temp_var: str
    daylength_var: str
    # f_water: soil water read as a sibling stock via env.get (#16); the band is soil /
    # scenario data (rooting depth + soil type), like ground_area — not a crop param.
    soil_water_var: str
    sw_wilting: float
    sw_critical: float
    # f_N: the plant's own N pool (a direct snapshot read, #16); the concentration
    # thresholds live in the (loader-folded) nitrogen params.
    plant_n: StockId
    photo: PhotosynthesisParams
    canopy: CanopyParams
    resp: RespirationParams
    nitro: NitrogenParams
    ground_area: float
    # Ci source (P2.2). Default None → the Phase-1 forcing read of ``ci_var`` (open
    # field; the regression golden is unchanged). When ``co2_pool_var`` is set, Ci is
    # derived from a finite chamber carbon pool read as a shared stock (#16) via
    # ``chamber.ci_from_co2_pool`` — the sealed-chamber draw-down feedback (Step 2): the
    # pool falls → Ci falls → assimilation falls, with no control code.
    # ``chamber_air_mol`` (total chamber air) and ``ci_ratio`` (the C3 Ci/Ca set point)
    # are chamber/scenario data (P4), required together with ``co2_pool_var`` (the
    # all-or-nothing guard below).
    co2_pool_var: str | None = None
    chamber_air_mol: float | None = None
    ci_ratio: float | None = None

    def __post_init__(self) -> None:
        # The chamber Ci-source fields are all-or-nothing: either the full sealed triple
        # is wired or Ci stays the forcing read. A partial wiring is a build bug.
        chamber = (self.co2_pool_var, self.chamber_air_mol, self.ci_ratio)
        if any(v is not None for v in chamber) and any(v is None for v in chamber):
            raise ValueError(
                "CarbonContext chamber Ci-source fields (co2_pool_var, "
                "chamber_air_mol, ci_ratio) must be set together or all left None"
            )

    def _ci(self, env: Environment) -> float:
        """Intercellular CO₂ ``Ci`` (µmol mol⁻¹) — the forcing-vs-pool seam (P2.2).

        Phase-1 open field: the ``ci_var`` forcing read. Sealed chamber: derived from
        the live finite carbon pool (``co2_pool_var``, a shared stock #16) so the pool's
        draw-down lowers ``Ci`` — the emergent feedback, no controller. The caller
        (:meth:`budget`) cannot tell which branch answered.
        """
        if self.co2_pool_var is None:
            return env.get(self.ci_var)
        # __post_init__ guarantees the air/ratio are present when the pool var is; the
        # explicit raise (vs assert) narrows the type and survives ``python -O``.
        air_mol, ci_ratio = self.chamber_air_mol, self.ci_ratio
        if air_mol is None or ci_ratio is None:
            raise ValueError(
                "sealed CarbonContext is missing chamber_air_mol/ci_ratio "
                "(the all-or-nothing Ci-source guard should have caught this)"
            )
        return ci_from_co2_pool(
            env.get(self.co2_pool_var), air_mol=air_mol, ci_ratio=ci_ratio
        )

    def _leaf_and_biomass(self, snapshot: State) -> tuple[float, float]:
        """``(leaf_carbon, Σ(leaf + stem + root))`` — the LAI and biomass reads.

        The single biomass expression every maintenance / ``f_N`` consumer shares
        (storage excluded). Reading the organ pools directly from the immutable
        snapshot (the plant's own state, #16).
        """
        leaf = snapshot.stocks[self.leaf_c].amount
        biomass = (
            leaf
            + snapshot.stocks[self.stem_c].amount
            + snapshot.stocks[self.root_c].amount
        )
        return leaf, biomass

    def limitation(self, snapshot: State, env: Environment) -> float:
        """The ``Π fᵢ`` factor ``f_water · f_N ∈ [0, 1]`` applied to gross assimilation.

        ``f_water`` reads the ``soil_water`` sibling stock via ``env.get`` (#16) and
        ramps over the soil-water band; ``f_N`` reads the plant's own ``plant_n`` pool
        against the ``Σ(leaf + stem + root)`` biomass. Computed **once, here**, so every
        flow's gross-assimilation recompute is limited identically (no flow passes its
        own factor — the structural-agreement win).
        """
        soil_water = env.get(self.soil_water_var)
        f_water = water_stress_factor(
            soil_water, sw_wilting=self.sw_wilting, sw_critical=self.sw_critical
        )
        _, biomass = self._leaf_and_biomass(snapshot)
        plant_n = snapshot.stocks[self.plant_n].amount
        f_n = nitrogen_stress_factor(
            plant_n,
            biomass,
            n_residual_per_mol_c=self.nitro.n_residual_per_mol_c,
            n_critical_per_mol_c=self.nitro.n_critical_per_mol_c,
        )
        return f_water * f_n

    def budget(self, snapshot: State, env: Environment) -> tuple[float, float, float]:
        """Daily ``(GASS, MRES, available)`` at the step-entry snapshot (mol C/day).

        ``GASS`` is gross canopy assimilation (Step 4/5) at the ``leaf_c``-derived LAI,
        limited by :meth:`limitation`; ``MRES`` is maintenance respiration (Step 6) on
        the ``Σ(leaf + stem + root)`` biomass; ``available = max(0, GASS − MRES)`` is
        the shared maintenance-first budget. All daily rates (dt-independent); the flows
        multiply by ``dt``. Maintenance is **not** limited (it is unconditional); only
        gross assimilation carries the ``Π fᵢ`` factor.
        """
        leaf, biomass = self._leaf_and_biomass(snapshot)
        lai = leaf_area_index(
            leaf, sla_per_mol_c=self.canopy.sla_per_mol_c, ground_area=self.ground_area
        )
        gass = daily_canopy_assimilation(
            env.get(self.par_var),
            lai,
            self._ci(env),
            env.get(self.temp_var),
            env.get(self.daylength_var),
            params=self.photo,
            canopy=self.canopy,
            ground_area=self.ground_area,
            limitation=self.limitation(snapshot, env),
        )
        mres = maintenance_respiration_flux(
            biomass, env.get(self.temp_var), params=self.resp
        )
        return gass, mres, available_for_growth(gass, mres)


@dataclass(frozen=True)
class Allocation:
    """CARBON growth ``co2_atmos -> {leaf_c, stem_c, root_c, storage_c}`` (balanced).

    Deposits the daily structural increment ``DMI = Yg·available`` (recomputed via the
    shared :class:`CarbonContext`) split by the DVS-keyed partition fractions (Step 9;
    ``DVS`` derived from the ``thermal_time`` aux accumulator). The ``co2_atmos`` leg is
    ``−DMI = −Σ(organ legs)`` so the flow balances by construction; the loader's per-row
    sum-to-1 check enforces ``Σ(organ legs) == DMI``. ``flux = daily·dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    ctx: CarbonContext
    co2_atmos: StockId
    storage_c: StockId
    thermal_time_aux: str
    pheno: PhenologyParams
    alloc: AllocationParams
    # Sealed chamber (P2.1 multi-quantity, Step 3). When set, photosynthesis is the
    # genuine ``CO₂ → biomass + O₂`` flow: every mol C fixed into an organ releases
    # 1 mol O₂ (PQ=1, pure-carbon biomass), deposited here. The organs carry no oxygen,
    # so the O₂ leg supplies exactly the 2 oxygens the CO₂ pool gave up. None (open
    # field) keeps the single-currency Phase-1 legs byte-identical.
    o2_pool: StockId | None = None

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        _, _, available = self.ctx.budget(snapshot, env)
        dmi = self.ctx.resp.growth_efficiency * available
        thermal_time = snapshot.aux.get(self.thermal_time_aux, 0.0)
        dvs = development_stage(
            thermal_time,
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        leaf, stem, root, storage = partition(dmi, dvs, self.alloc.table)
        leaf_leg = leaf * dt
        stem_leg = stem * dt
        root_leg = root * dt
        storage_leg = storage * dt
        organ_total = leaf_leg + stem_leg + root_leg + storage_leg
        legs = [
            Leg(self.co2_atmos, -organ_total),
            Leg(self.ctx.leaf_c, leaf_leg),
            Leg(self.ctx.stem_c, stem_leg),
            Leg(self.ctx.root_c, root_leg),
            Leg(self.storage_c, storage_leg),
        ]
        if self.o2_pool is not None:
            # O₂ released = carbon fixed into biomass (PQ=1). Uses the same
            # ``organ_total`` sum that sources the CO₂ leg, so OXYGEN balances exactly
            # (−2·organ_total from the CO₂ pool + 2·organ_total from this O₂ leg).
            legs.append(Leg(self.o2_pool, organ_total))
        return FlowResult(legs=tuple(legs))


@dataclass(frozen=True)
class GrowthRespiration:
    """CARBON growth-conversion loss ``co2_atmos -> co2_resp`` (balanced, P1).

    ``GRES = (1 − Yg)·available`` (recomputed via the shared :class:`CarbonContext`, so
    it shares ``available`` with :class:`Allocation` — ``GRES + DMI = available``, the
    budget telescopes). Assimilated and immediately respired: the carbon is drawn from
    the unclamped atmosphere ``co2_atmos`` and deposited into the respiration sink
    ``co2_resp`` (never becomes biomass). ``flux = daily·dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    ctx: CarbonContext
    co2_atmos: StockId
    co2_resp: StockId

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        if self.co2_atmos == self.co2_resp:
            # Closed chamber (Step 3): growth-conversion carbon is gross-assimilated and
            # immediately respired — a CO₂→CO₂ round trip on the single pool, with the
            # photosynthetic O₂ release reconsumed by the respiration (PQ=1). Net gas
            # change is zero, so this is an empty no-op flow. (Emitting both legs on the
            # one pool stock would also trip ``FlowResult``'s duplicate-leg guard.)
            return FlowResult(legs=())
        _, _, available = self.ctx.budget(snapshot, env)
        gres = (1.0 - self.ctx.resp.growth_efficiency) * available
        flux = gres * dt
        return FlowResult(legs=(Leg(self.co2_atmos, -flux), Leg(self.co2_resp, flux)))


@dataclass(frozen=True)
class MaintenanceRespiration:
    """CARBON maintenance ``{co2_atmos(covered), organs(shortfall)} -> co2_resp`` (P1).

    Maintenance-first: ``MRES`` (on the ``Σ(leaf + stem + root)`` biomass) is paid from
    today's assimilate where it covers it (``covered = min(GASS, MRES)`` from the
    unclamped atmosphere) and from standing biomass where it does not
    (``shortfall = max(0, MRES − GASS)`` drawn from the organ pools, **proportional to
    each organ's carbon share** — so the draw is self-limiting (∝ organ amount) and the
    backstop is structurally impossible). ``covered + shortfall = MRES``, so the
    ``co2_resp`` leg is ``+MRES`` and the flow balances. ``storage_c`` is excluded
    (grain pays no maintenance in Phase 1). ``flux = daily·dt`` — dt-linear (``min`` /
    ``max`` of daily rates scales with ``dt``).

    The shortfall is what makes biomass shrink in a carbon deficit (cold/dark
    overwintering) — the WOFOST-faithful behavior the dissolved buffer could not host.

    **O₂ self-limitation (Step 7).** In the sealed chamber the shortfall's O₂ draw is
    throttled by ``f_O2`` (``chamber.oxygen_limitation_factor`` of the live O₂ mole
    fraction, ``K_O2 = ctx.resp.o2_half_saturation``, ``air_mol`` this flow's chamber
    basis), so on a depleting O₂ pool the burn shuts off smoothly before the pool is
    over-run (``rationed == 0`` from kinetics — the respiratory mirror of FvCB's
    Ci-shutoff). The unburned ``(1−f_O2)·shortfall`` maintenance just stays in biomass.
    ``f_O2`` only bites on deficit days (``shortfall > 0``, where ``available`` is
    already 0), so it never perturbs allocation / growth respiration; at the PP fill it
    is ≈ 1. Open field (``o2_pool=None``) is byte-identical Phase-1 behaviour.
    """

    id: FlowId
    priority: int
    ctx: CarbonContext
    co2_atmos: StockId
    co2_resp: StockId
    # Sealed chamber (P2.1 multi-quantity, Step 3). When set, the biomass-sourced
    # shortfall is the genuine ``biomass + O₂ → CO₂`` respiration: each mol C burned
    # consumes 1 mol O₂ (PQ=1) drawn from here. None (open field) keeps the
    # single-currency Phase-1 legs byte-identical.
    o2_pool: StockId | None = None
    # Total chamber air (mol) — the intensive basis for the ``f_O2`` O₂ mole fraction
    # (Step 7), mirroring ``MicrobialRespiration.air_mol``. Set with ``o2_pool`` when
    # sealed (``scenario.chamber_air_mol``); None in the open field (f_O2 unused).
    air_mol: float | None = None

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        gass, mres, _ = self.ctx.budget(snapshot, env)
        leaf, biomass = self.ctx._leaf_and_biomass(snapshot)
        covered = min(gass, mres)
        shortfall = mres - covered  # == max(0, MRES − GASS); biomass-sourced
        covered_flux = covered * dt
        if self.co2_atmos == self.co2_resp:
            # Closed chamber (Step 3): the ``covered`` maintenance (paid from today's
            # assimilate) is a CO₂→CO₂ round trip on the single pool — net zero, and
            # would duplicate the pool leg — so it is dropped. Only the biomass-burned
            # ``shortfall`` is a real respiration: organs → CO₂ pool, consuming O₂.
            # f_O2 self-limit (Step 7): the O₂-consuming shortfall is throttled by the
            # live O₂ mole fraction so it shuts off smoothly on a depleting pool
            # (rationed == 0 from kinetics — the Ci-shutoff mirror). The unburned
            # (1−f_O2)·shortfall maintenance simply stays in biomass (O₂-limited
            # respiration). f_O2 only ever applies on a deficit day (shortfall > 0),
            # where ``available`` is already 0 — so it never perturbs allocation /
            # growth respiration. At the PP fill f_O2 ≈ 1 (low/sharp K_O2). K_O2 is the
            # plant respiration param (``ctx.resp``); ``air_mol`` is this flow's chamber
            # basis (mirrors MicrobialRespiration).
            f_o2 = 1.0
            if self.o2_pool is not None:
                if self.air_mol is None:
                    raise ValueError(
                        "sealed MaintenanceRespiration is missing air_mol "
                        "(must be set together with o2_pool)"
                    )
                f_o2 = oxygen_limitation_factor(
                    snapshot.stocks[self.o2_pool].amount,
                    air_mol=self.air_mol,
                    k_o2=self.ctx.resp.o2_half_saturation,
                )
            legs: list[Leg] = []
            organ_burn = 0.0  # Σ actual organ withdrawals (the carbon that returns)
            if biomass > 0.0 and shortfall > 0.0:
                stem = snapshot.stocks[self.ctx.stem_c].amount
                root = snapshot.stocks[self.ctx.root_c].amount
                for organ_id, organ_c in (
                    (self.ctx.leaf_c, leaf),
                    (self.ctx.stem_c, stem),
                    (self.ctx.root_c, root),
                ):
                    share = f_o2 * shortfall * (organ_c / biomass) * dt
                    legs.append(Leg(organ_id, -share))
                    organ_burn += share
            if organ_burn != 0.0:
                # CO₂ returned to the pool = burned carbon; O₂ consumed = burned carbon
                # (PQ=1). Both use the same ``organ_burn`` sum, so CARBON and OXYGEN
                # balance exactly against the organ withdrawals.
                legs.append(Leg(self.co2_resp, organ_burn))
                if self.o2_pool is not None:
                    legs.append(Leg(self.o2_pool, -organ_burn))
            return FlowResult(legs=tuple(legs))
        # Open field (Phase-1, byte-identical): covered from the atmosphere, shortfall
        # from the organs, all deposited into the co2_resp boundary sink.
        legs = [Leg(self.co2_atmos, -covered_flux)]
        respired = covered_flux  # sum the actual withdrawals so the deposit balances
        if biomass > 0.0 and shortfall > 0.0:
            # Proportional to each organ's share of the biomass (storage excluded).
            stem = snapshot.stocks[self.ctx.stem_c].amount
            root = snapshot.stocks[self.ctx.root_c].amount
            for organ_id, organ_c in (
                (self.ctx.leaf_c, leaf),
                (self.ctx.stem_c, stem),
                (self.ctx.root_c, root),
            ):
                share = shortfall * (organ_c / biomass) * dt
                legs.append(Leg(organ_id, -share))
                respired += share
        # co2_resp deposit = Σ|withdrawals| so the flow balances by construction (the
        # three organ shares need not sum to shortfall·dt exactly in float). respired
        # equals MRES·dt up to that rounding — the honest carbon actually moved.
        legs.append(Leg(self.co2_resp, respired))
        return FlowResult(legs=tuple(legs))
