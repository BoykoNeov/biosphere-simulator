"""Leaf/stem/root/storage partitioning + senescence (Phase-1 Steps 9 + 11).

The DVS-keyed dry-matter partition table + the litter-shedding flow. This module holds
the **partition split functions** (``partition_fractions`` / ``partition``) and the
**``Senescence`` flow**; the ``Allocation`` flow that deposits the partitioned
increment ``DMI`` moved to ``domains.biosphere.carbon_budget`` at Step 11 (the buffer
dissolution — it now sources ``DMI`` from ``co2_atmos``, recomputed via the shared
``CarbonContext``, rather than draining a ``plant_c`` buffer).

* **Senescence** — ``{leaf_c, stem_c, root_c} -> litter_sink`` (Σ legs = 0). A relative
  death rate per organ, ``rate · organ_c`` (→ 0 as the organ → 0, so positivity is
  structural). ``litter_sink`` is a **BOUNDARY** sink, distinct from the numerical
  extinction loss-sink (decision #6): real shed biomass, consumed by Phase-2 litter /
  decomposition dynamics. ``storage_c`` (grain) is excluded — it is harvested, not shed.

**Sum-to-1 under interpolation — the load-bearing balance constraint.** If the organ
fractions did not sum to 1 at the evaluated DVS, the allocation legs would not sum to
``DMI`` and the every-step conservation gate would hard-fail. Designed out two ways:
the partition table is a **single DVS-keyed table of ``(dvs, FL, FS, FR, FO)`` rows**
with shared breakpoints (sum-1 at every knot ⇒ sum-1 everywhere, ``lerp(1, 1) = 1`` —
*not* independent tables, which would sum ≠ 1 between mismatched knots; the loader
enforces the per-row sum); and the ``Allocation`` flow sets its ``co2_atmos`` source leg
to ``−Σ(organ legs)`` so it **balances by construction** regardless.

**Storage organ (FO; the Step-11 precondition — built).** The committed oracle fixture
has ``TWSO ≈ 11.5`` of ``TAGP ≈ 20.4 t/ha`` (grain is ~half the biomass), so a 3-organ
model cannot match the curve. The table carries a 4th fraction ``FO`` (``fo = 0`` before
anthesis; storage fills the reproductive phase, ``DVS > 1``) and the flow deposits to a
``storage_c`` pool. ``storage_c`` is **excluded from maintenance and senescence in Phase
1** (grain is harvested, not shed; grain maintenance is a documented refinement seam) —
it is a pure allocation sink, kept out of the ``Σ organs`` maintenance / ``f_N``
biomass read and the ``Senescence`` flow.

**Deferred Step-11 seam.** **Senescence keying** — DVS / leaf-age / self-shading scaling
of the death rate (the Step-6 ``maturity``-seam shape); standalone is a plain per-organ
constant relative rate.

Pure stdlib only. Citations: Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M.
& Bakema, A. (1989), *Simulation of Ecophysiological Processes of Growth in Several
Annual Crops*, Simulation Monographs, PUDOC, Wageningen (DVS-keyed dry-matter
partitioning); van Keulen, H. & Wolf, J., eds. (1986), *Modelling of Agricultural
Production: Weather, Soils and Crops*, PUDOC, Wageningen.
"""

from dataclasses import dataclass

from domains.biosphere.canopy import CanopyParams, leaf_area_index
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class PartitionRow:
    """One DVS knot of the leaf/stem/root/storage partition table.

    ``fl + fs + fr + fo`` must equal 1 (enforced at the config boundary); shared knots
    across organs keep the interpolated fractions summing to 1 everywhere
    (``lerp(1, 1) = 1``). ``fo`` (storage / grain) is 0 before anthesis and fills the
    reproductive phase (``DVS > 1``) — the Step-11 storage-organ column the committed
    oracle fixture requires (``TWSO`` ≈ half of ``TAGP``).
    """

    dvs: float  # development stage at this knot (0 emergence, 1 anthesis, 2 maturity)
    fl: float  # leaf fraction of the structural increment
    fs: float  # stem fraction
    fr: float  # root fraction
    fo: float  # storage-organ (grain) fraction


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
    shade_rate: float  # ADDITIONAL leaf relative death rate above the LAI threshold
    lai_threshold: float  # LAI above which mutual shading starts killing leaves


def partition_fractions(
    dvs: float, table: tuple[PartitionRow, ...]
) -> tuple[float, float, float, float]:
    """Interpolate ``(FL, FS, FR, FO)`` at ``dvs`` from the partition table.

    Piecewise-linear in ``dvs`` between the table knots; **flat extrapolation** outside
    the table (clamped to the first/last row). Because each knot sums to 1 and the knots
    are shared across organs, the interpolated fractions sum to 1 at every ``dvs``.
    Raises ``ValueError`` on an empty table.
    """
    if not table:
        raise ValueError("partition table must have at least one row")
    if dvs <= table[0].dvs:
        first = table[0]
        return (first.fl, first.fs, first.fr, first.fo)
    if dvs >= table[-1].dvs:
        last = table[-1]
        return (last.fl, last.fs, last.fr, last.fo)
    for lo, hi in zip(table, table[1:], strict=False):
        if lo.dvs <= dvs <= hi.dvs:
            w = (dvs - lo.dvs) / (hi.dvs - lo.dvs)
            return (
                lo.fl + w * (hi.fl - lo.fl),
                lo.fs + w * (hi.fs - lo.fs),
                lo.fr + w * (hi.fr - lo.fr),
                lo.fo + w * (hi.fo - lo.fo),
            )
    # Unreachable: dvs is strictly inside [table[0].dvs, table[-1].dvs] here, and the
    # knots are increasing (loader-enforced), so some adjacent pair always brackets it.
    raise AssertionError(f"no bracketing knot for dvs={dvs!r}")  # pragma: no cover


def partition(
    dmi: float, dvs: float, table: tuple[PartitionRow, ...]
) -> tuple[float, float, float, float]:
    """Split a daily increment ``dmi`` into ``(leaf, stem, root, storage)`` (mol C/day).

    ``dmi · (FL, FS, FR, FO)`` at the interpolated fractions
    (:func:`partition_fractions`). The four values sum to ``dmi`` (fractions sum to 1).
    """
    fl, fs, fr, fo = partition_fractions(dvs, table)
    return (fl * dmi, fs * dmi, fr * dmi, fo * dmi)


def senescence_flux(organ_c: float, *, relative_death_rate: float) -> float:
    """Daily senescence loss of an organ: ``relative_death_rate · organ_c`` (mol C/day).

    Proportional to the standing organ carbon, so it → 0 as the organ → 0 (positivity
    is structural, the Step-5/6/7 self-limiting pattern). DVS / leaf-age keying of the
    rate is the deferred Step-11 seam (standalone is a plain constant relative rate).
    """
    return relative_death_rate * organ_c


def mutual_shading_rate(
    lai: float, *, rdr_leaf: float, shade_rate: float, lai_threshold: float
) -> float:
    """Leaf relative death rate with mutual shading: ``rdr + shade`` above ``LAI*``.

    Van Keulen & Seligman (1987), via [A] p. 101: in **wheat**, leaf area is lost at
    ``5 %/day`` once the leaf area index exceeds ``6 m² m⁻²``, to account for mutual
    shading — leaves buried below a closed canopy fall under their own compensation
    point and die. A **step**, not a ramp: that is the form the source states.

    ⚠ **Cited on leaf AREA, applied to leaf CARBON, and the two are the same rate only
    because SLA is a constant here.** ``LAI = leaf_c · sla / ground_area`` with ``sla``
    fixed, so a 5 %/day relative loss of leaf carbon *is* a 5 %/day relative loss of
    leaf area. Should SLA ever become development-keyed, this identity breaks and the
    term would have to move to an area basis — see the winter-wheat SLA finding in
    ``docs/log/canopy-magnitude.md`` (measured, and refused, 2026-08-15).

    Self-limiting like the base rate (proportional to standing carbon downstream), so
    it cannot drive leaf carbon negative.
    """
    return rdr_leaf + shade_rate if lai > lai_threshold else rdr_leaf


@dataclass(frozen=True)
class Senescence:
    """CARBON loss flow ``{leaf_c, stem_c, root_c} -> litter_sink`` (balanced, P1).

    Each organ sheds ``senescence_flux(organ_c, rdr)·dt`` of carbon to the litter
    boundary sink; the litter leg is the sum. Self-limiting (→ 0 as an organ → 0), so
    positivity is structural. ``litter_sink`` is a BOUNDARY reservoir distinct from the
    numerical extinction loss-sink (decision #6) — real shed biomass.
    ``flux = daily·dt`` is dt-linear.

    ⚠ **The leaf rate gained a mutual-shading term on 2026-08-15** — see
    :func:`mutual_shading_rate`. It is a **state-dependent** rate (it reads leaf carbon
    to form LAI), where the other two organs' rates are constants. It was built because
    the tree grew into the regime the term describes: binding ``specific_leaf_area`` to
    its primary source took ``open_season``'s peak LAI to 6.0228, past the sourced
    threshold of 6.0, and a mechanism that a cited source says fires in that regime is
    then owed. ⚠ It is **bit-identically inert in every chamber scenario** (their
    canopies peak at LAI 0.07–0.63, an order of magnitude below the threshold), so it
    is an open-field mechanism only — the same closure-scaling that makes the canopy
    depth integral inert there.
    """

    id: FlowId
    priority: int
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    litter_sink: StockId
    params: SenescenceParams
    canopy: CanopyParams
    ground_area: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        lai = leaf_area_index(
            snapshot.stocks[self.leaf_c].amount,
            sla_per_mol_c=self.canopy.sla_per_mol_c,
            ground_area=self.ground_area,
        )
        leaf = (
            senescence_flux(
                snapshot.stocks[self.leaf_c].amount,
                relative_death_rate=mutual_shading_rate(
                    lai,
                    rdr_leaf=self.params.rdr_leaf,
                    shade_rate=self.params.shade_rate,
                    lai_threshold=self.params.lai_threshold,
                ),
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
