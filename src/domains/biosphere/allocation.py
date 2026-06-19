"""Leaf/stem/root biomass allocation + senescence (Phase-1 Step 9).

The first **internal-redistribution** CARBON process and the first **multi-organ**
stock structure. Step 5 deposits gross assimilate and Step 6 books the respiratory
losses, leaving a daily structural increment ``DMI = Yg·max(0, GASS − MRES)``; Step 9
partitions that increment among the plant organs and sheds aged tissue to litter. Two
single-currency CARBON flows, each internally balanced (P1):

* **Allocation** — ``plant_c -> {leaf_c, stem_c, root_c}`` (Σ legs = 0, an internal
  redistribution). A **multi-leg, recompute-DMI** flow (the :class:`GrowthRespiration`
  pattern): because flows ``evaluate`` independently against the step-entry snapshot
  (no flow can read another's result pre-arbitration), the structural increment is
  **recomputed** here — ``GASS`` via the Step-4/5 canopy/FvCB stack, ``MRES`` via the
  shared Step-6 maintenance helper, and ``DMI = Yg·available_for_growth(GASS, MRES)``
  via the **same** :func:`~domains.biosphere.respiration.available_for_growth` that
  ``growth_respiration_flux`` uses (agreement by construction — recomputing that budget
  independently would risk a 3-way drift across assimilation/growth-resp/allocation).
  The increment is split by **DVS-keyed** fractions (``DVS`` derived from the Step-8
  ``thermal_time`` accumulator).

* **Senescence** — ``{leaf_c, stem_c, root_c} -> litter_sink`` (Σ legs = 0). A relative
  death rate per organ, ``rate · organ_c`` (→ 0 as the organ → 0, so positivity is
  structural). ``litter_sink`` is a **BOUNDARY** sink, distinct from the numerical
  extinction loss-sink (decision #6): real shed biomass, consumed by Phase-2 litter /
  decomposition dynamics.

**Sum-to-1 under interpolation — the load-bearing balance constraint.** If the organ
fractions did not sum to 1 at the evaluated DVS, the organ legs would not sum to
``DMI`` and the every-step conservation gate would hard-fail. Designed out two ways:
the partition table is a **single DVS-keyed table of ``(dvs, FL, FS, FR)`` rows** with
shared breakpoints (sum-1 at every knot ⇒ sum-1 everywhere, since ``lerp(1, 1) = 1`` —
*not* three independent tables, which would sum ≠ 1 between mismatched knots; the loader
enforces the per-row sum); and the flow sets the ``plant_c`` leg to ``−Σ(organ legs)``
so it **balances by construction** regardless.

**The Step-11 transition (documented, not done here).** Once allocation drains ``DMI``,
``plant_c`` reframes from "the biomass" to a near-zero **labile carbohydrate buffer**
(``+GASS −MRES −GRES −DMI = 0`` per step when ``GASS ≥ MRES``). At Step 11 the *other*
carbon flows are re-pointed **per-read, not per-flow** — every flow recomputing a shared
quantity reads the same stock: every **LAI** recompute (``GrossAssimilation``,
``GrowthRespiration``, ``Allocation``) reads ``leaf_c``; every **MRES** recompute
(``MaintenanceRespiration``, ``GrowthRespiration``, ``Allocation``) reads
``Σ(leaf + stem + root)``. The trap: ``GrowthRespiration`` reads ``plant_c`` *once* and
feeds it to **both** its LAI and its MRES recompute — switching only the LAI read leaves
its MRES on the ~empty buffer, so it no longer shares :func:`available_for_growth` with
``Allocation`` and the budget stops telescoping. The conservation gate will *not* catch
this (it is physics, not balance). ``Allocation`` here already reads the
post-transition-correct stocks (``leaf_c`` for LAI, ``Σ`` organs for maintenance), so
Step 11 re-points only the other flows to agree with it.

**Deferred Step-11 seams.** (1) **Grain / storage organ** — the title is leaf/stem/root,
but the committed oracle fixture has ``TWSO ≈ 11.5`` of ``TAGP ≈ 20.4 t/ha``, so a
3-organ model cannot match the biomass curve; a 4th fraction ``FO`` + a ``storage_c``
pool is an additive Step-11 precondition. (2) **Senescence keying** — DVS / leaf-age /
self-shading scaling of the death rate (the Step-6 ``maturity``-seam shape); standalone
is a plain per-organ constant relative rate.

Pure stdlib only. Citations: Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M.
& Bakema, A. (1989), *Simulation of Ecophysiological Processes of Growth in Several
Annual Crops*, Simulation Monographs, PUDOC, Wageningen (DVS-keyed dry-matter
partitioning); van Keulen, H. & Wolf, J., eds. (1986), *Modelling of Agricultural
Production: Weather, Soils and Crops*, PUDOC, Wageningen.
"""

from dataclasses import dataclass

from domains.biosphere.canopy import CanopyParams, leaf_area_index
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
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class PartitionRow:
    """One DVS knot of the leaf/stem/root partition table.

    ``fl + fs + fr`` must equal 1 (enforced at the config boundary); shared knots across
    organs keep the interpolated fractions summing to 1 everywhere (``lerp(1, 1) = 1``).
    """

    dvs: float  # development stage at this knot (0 emergence, 1 anthesis, 2 maturity)
    fl: float  # leaf fraction of the structural increment
    fs: float  # stem fraction
    fr: float  # root fraction


@dataclass(frozen=True)
class AllocationParams:
    """Loader-produced allocation parameters: the DVS-keyed partition table.

    ``table`` is a non-empty tuple of :class:`PartitionRow` with strictly increasing
    ``dvs`` (enforced by the loader). Provisional literature-typical placeholders
    pending the Step-11 validation gate (see ``params/allocation.yaml``).
    """

    table: tuple[PartitionRow, ...]


@dataclass(frozen=True)
class SenescenceParams:
    """Loader-produced senescence parameters: per-organ relative death rates (1/day).

    Provisional literature-typical placeholders pending the Step-11 validation gate (see
    ``params/senescence.yaml``). A zero rate is valid (no turnover of that organ).
    """

    rdr_leaf: float  # relative death rate of leaf carbon (mol C lost / mol C / day)
    rdr_stem: float  # relative death rate of stem carbon
    rdr_root: float  # relative death rate of root carbon


def partition_fractions(
    dvs: float, table: tuple[PartitionRow, ...]
) -> tuple[float, float, float]:
    """Interpolate ``(FL, FS, FR)`` at ``dvs`` from the partition table.

    Piecewise-linear in ``dvs`` between the table knots; **flat extrapolation** outside
    the table (clamped to the first/last row). Because each knot sums to 1 and the knots
    are shared across organs, the interpolated fractions sum to 1 at every ``dvs``.
    Raises ``ValueError`` on an empty table.
    """
    if not table:
        raise ValueError("partition table must have at least one row")
    if dvs <= table[0].dvs:
        first = table[0]
        return (first.fl, first.fs, first.fr)
    if dvs >= table[-1].dvs:
        last = table[-1]
        return (last.fl, last.fs, last.fr)
    for lo, hi in zip(table, table[1:], strict=False):
        if lo.dvs <= dvs <= hi.dvs:
            w = (dvs - lo.dvs) / (hi.dvs - lo.dvs)
            return (
                lo.fl + w * (hi.fl - lo.fl),
                lo.fs + w * (hi.fs - lo.fs),
                lo.fr + w * (hi.fr - lo.fr),
            )
    # Unreachable: dvs is strictly inside [table[0].dvs, table[-1].dvs] here, and the
    # knots are increasing (loader-enforced), so some adjacent pair always brackets it.
    raise AssertionError(f"no bracketing knot for dvs={dvs!r}")  # pragma: no cover


def partition(
    dmi: float, dvs: float, table: tuple[PartitionRow, ...]
) -> tuple[float, float, float]:
    """Split a daily increment ``dmi`` into ``(leaf, stem, root)`` (mol C/day).

    ``dmi · (FL, FS, FR)`` at the interpolated fractions (:func:`partition_fractions`).
    The three returned values sum to ``dmi`` (the fractions sum to 1).
    """
    fl, fs, fr = partition_fractions(dvs, table)
    return (fl * dmi, fs * dmi, fr * dmi)


def senescence_flux(organ_c: float, *, relative_death_rate: float) -> float:
    """Daily senescence loss of an organ: ``relative_death_rate · organ_c`` (mol C/day).

    Proportional to the standing organ carbon, so it → 0 as the organ → 0 (positivity
    is structural, the Step-5/6/7 self-limiting pattern). DVS / leaf-age keying of the
    rate is the deferred Step-11 seam (standalone is a plain constant relative rate).
    """
    return relative_death_rate * organ_c


@dataclass(frozen=True)
class Allocation:
    """CARBON redistribution ``plant_c -> {leaf_c, stem_c, root_c}`` (balanced, P1).

    Recomputes the daily structural increment ``DMI = Yg·available_for_growth(GASS,
    MRES)`` from the step-entry snapshot (flows cannot read each other's results), then
    splits it by DVS-keyed fractions. ``GASS`` is recomputed via the Step-4/5
    :func:`daily_canopy_assimilation` (LAI from ``leaf_c`` — the
    post-transition-correct read), ``MRES`` via the Step-6
    :func:`maintenance_respiration_flux` on the total organ biomass
    ``Σ(leaf + stem + root)``, and ``DMI`` via the **shared**
    :func:`available_for_growth` (no budget drift with growth respiration). ``DVS`` is
    derived from the Step-8 ``thermal_time`` accumulator (``snapshot.aux``). The long
    field list is the inherent cost of a flux-and-state-coupled quantity in an
    independent-flow engine (the ``GrowthRespiration`` precedent).

    The ``plant_c`` leg is set to ``−Σ(organ legs)`` so the flow balances by
    construction; the loader's per-row sum-to-1 check enforces that ``Σ(organ legs)``
    equals ``DMI``. ``flux = daily·dt`` — dt-linear (the daily rate is dt-independent;
    the ``max(0, …)`` in ``available_for_growth`` clamps a daily rate, not a dt-gate).
    """

    id: FlowId
    priority: int
    plant_c: StockId
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    par_var: str
    ci_var: str
    temp_var: str
    daylength_var: str
    thermal_time_aux: str
    photo: PhotosynthesisParams
    canopy: CanopyParams
    resp: RespirationParams
    pheno: PhenologyParams
    alloc: AllocationParams
    ground_area: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        incident_par = env.get(self.par_var)
        ci = env.get(self.ci_var)
        temp_c = env.get(self.temp_var)
        daylength_s = env.get(self.daylength_var)

        leaf_carbon = snapshot.stocks[self.leaf_c].amount
        biomass = (
            leaf_carbon
            + snapshot.stocks[self.stem_c].amount
            + snapshot.stocks[self.root_c].amount
        )
        lai = leaf_area_index(
            leaf_carbon,
            sla_per_mol_c=self.canopy.sla_per_mol_c,
            ground_area=self.ground_area,
        )
        gross = daily_canopy_assimilation(
            incident_par,
            lai,
            ci,
            temp_c,
            daylength_s,
            params=self.photo,
            canopy=self.canopy,
            ground_area=self.ground_area,
            limitation=1.0,
        )
        maintenance = maintenance_respiration_flux(biomass, temp_c, params=self.resp)
        dmi = self.resp.growth_efficiency * available_for_growth(gross, maintenance)

        thermal_time = snapshot.aux.get(self.thermal_time_aux, 0.0)
        dvs = development_stage(
            thermal_time,
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        leaf, stem, root = partition(dmi, dvs, self.alloc.table)

        leaf_leg = leaf * dt
        stem_leg = stem * dt
        root_leg = root * dt
        return FlowResult(
            legs=(
                Leg(self.plant_c, -(leaf_leg + stem_leg + root_leg)),
                Leg(self.leaf_c, leaf_leg),
                Leg(self.stem_c, stem_leg),
                Leg(self.root_c, root_leg),
            )
        )


@dataclass(frozen=True)
class Senescence:
    """CARBON loss flow ``{leaf_c, stem_c, root_c} -> litter_sink`` (balanced, P1).

    Each organ sheds ``senescence_flux(organ_c, rdr)·dt`` of carbon to the litter
    boundary sink; the litter leg is the sum. Self-limiting (→ 0 as an organ → 0), so
    positivity is structural. ``litter_sink`` is a BOUNDARY reservoir distinct from the
    numerical extinction loss-sink (decision #6) — real shed biomass.
    ``flux = daily·dt`` is dt-linear.
    """

    id: FlowId
    priority: int
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    litter_sink: StockId
    params: SenescenceParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        leaf = (
            senescence_flux(
                snapshot.stocks[self.leaf_c].amount,
                relative_death_rate=self.params.rdr_leaf,
            )
            * dt
        )
        stem = (
            senescence_flux(
                snapshot.stocks[self.stem_c].amount,
                relative_death_rate=self.params.rdr_stem,
            )
            * dt
        )
        root = (
            senescence_flux(
                snapshot.stocks[self.root_c].amount,
                relative_death_rate=self.params.rdr_root,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.leaf_c, -leaf),
                Leg(self.stem_c, -stem),
                Leg(self.root_c, -root),
                Leg(self.litter_sink, leaf + stem + root),
            )
        )
