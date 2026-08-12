"""Sink-limited leaf expansion — LAI as a STATE variable ([F] Ch. 9 + [E] Table 20).

**What this is.** The node-driven, *sink-limited* phase of leaf-area development, the
water-deficit factor ``WSFL`` that attaches to it, and — because [F]'s branch is scoped
to potential production and this tree is mostly not — the **measured leaf-thickness
envelope from [E] that bounds it** (:func:`thickness_envelope`). The two are one
mechanism here: [F] decides where inside the envelope the canopy sits, and [E] decides
that it cannot leave. Neither half ships alone, and the reason is measured, not
stylistic: [F] alone rations on three scenarios and puts leaves 2–15× too thin, while
[E] alone is the frozen form that has no sink limitation for ``WSFL`` to attach to.

Before ``TLM`` (termination of leaf growth on the main stem) leaf area comes from
**main-stem node number through an allometric power law** and is *independent of dry
matter*; after ``TLM`` it comes from
leaf dry matter as it always did. [F] Eqn 9.8 integrates the two branches into one
state variable::

    LAI_t = LAI_{t-1} + GLAI - DLAI

**⚠ THIS REVERSES THE P2 LOCK, "LAI IS DERIVED, NOT STORED", AND THAT IS THE POINT.**
``WSFL`` scales an *expansion rate*, so the area a drought withholds must **stay**
withheld — a factor applied to the standing ``LAI`` instead would make the canopy
shrink and re-grow with the soil water, which is wilting/rolling, a different
mechanism and not this citation. Leaf area therefore decouples from leaf carbon, and
the conventional specific leaf area stops being an input to the canopy and becomes an
**emergent** ratio (drought gives less area for the same mass — thicker leaves, which
is exactly what [F] says the sink-limited branch is for). ``canopy.leaf_area_index``
survives as the **initial condition** and as the whole story for a crop with no leaf
parameters of its own (potato — see below).

**The two branches, and why only the first is new** ([F] Box 9.2, printed p. 111)::

    If CTU <= tuEMR Then                       ' pre-emergence
        GLAI = 0: DLAI = 0
    ElseIf CTU > tuEMR And CTU <= tuTLM Then    ' node-driven — SINK-limited
        INODE = DTU / PHYL
        MSNN  = MSNN + INODE
        PLA2  = PLACON * MSNN ^ PLAPOW
        GLAI  = ((PLA2 - PLA1) * PDEN / 10000) * WSFL
    ElseIf CTU > tuTLM And CTU <= tuBSG Then    ' carbon-driven — SOURCE-limited
        GLAI = GLF * SLA

Our accumulator **starts at emergence** (``thermal_time = 0 ⇒ DVS = 0``), so [F]'s
``tuEMR`` gate is satisfied by construction and the first branch never applies. The
third branch — ``GLAI = 0`` after ``BSG`` with a linear ``DLAI`` to maturity ([F] Eqn
9.7) — is **deliberately NOT built**: our tree already has a senescence flow with its
own citation, and adopting [F]'s would be a second form change to one mechanism inside
one build, which makes the golden diff unattributable. See "What is deferred".

**``MSNN`` needs no accumulator of its own.** [F] initialises ``MSNN = 1`` and adds
``DTU/PHYL`` every step, which for a constant phyllochron integrates exactly to
``1 + CTU/PHYL`` — a *derived* quantity, the same derived-not-stored idiom
:func:`phenology.development_stage` exists for. ⚠ This holds **because** ``WSFL`` is
applied to ``GLAI`` (Eqn 15.7) and not to ``INODE`` (Eqn 15.6): [F] offers both and
says the choice is a species question ("Some species respond to water deficit by a
slower rate of leaf appearance, but some others keep a constant rate of leaf
appearance and decrease leaf expansion and leaf size", printed p. 197), and **Box 16.2
programs 15.7**. Under 15.6 the node count would carry the stress history and would
have to be stored.

**⚠ THE CLOCK IS OURS, NOT [F]'S, AND THAT WAS MEASURED RATHER THAN ASSUMED.** [F]'s
``CTU`` is bare degree-days; our ``thermal_time`` carries vernalization, photoperiod
and drought. Running node appearance on a bare degree-day clock — [F]'s literal form,
which needs a second accumulator — puts ``open_season``'s peak LAI at **11.92**
against a mutual-shading ceiling of 6.0. On our own clock it lands at 5.91. The
development clock is the one ``DVS`` reads and ``TLM`` is a developmental stage, so
the bare clock is refused **by measurement**, and the measurement is in
``docs/plans/post-roadmap-leaf-expansion.md``.

**The scale identity that makes [F]'s °C·day numbers usable here.** [F]'s
``DTU = (TP1D - TBD) * tempfun`` with wheat's ``TBD = 0`` (Table 6.3; ``TP1D = 24``
for cv. Tajan, Fig. 12.5) reduces to ``DTU = TMP`` for every temperature between the
base and the first optimum — i.e. **plain base-0 degree-days, the same scale our
accumulator uses**. So ``tuEMRTLM = 724 °C·day`` is directly comparable to our
``thermal_time`` rather than needing a conversion, and it lands at ``DVS = 0.658``.

**What is deferred, named so it is not mistaken for an oversight:**

* **[F]'s own senescence** (Eqn 9.7, ``DLAI = DTU/(tuMAT-tuBSG)·BSGLAI``) and its
  ``GLAI = 0`` after ``BSG``. We keep our own senescence flow, re-expressed in area
  terms below.
* **Waterlogging** (Box 16.2's ``If WAT1 > 0.95*WSAT1 Then WSFG = 0: WSFL = 0``) —
  not modelled for ``WSFG`` either, so this changes nothing.
* **A non-constant phyllochron** ([F]'s own "Additional Notes", printed p. 111).

**⚠ ``DLAI`` IS KEYED ON AREA, NOT ON CARBON, AND THE CHOICE IS WORTH 18 %.** Our
``Senescence`` flow sheds ``rdr_leaf · leaf_C`` of *carbon* per day. The area that
leaves with it is ``rdr_leaf · LAI``: if a fraction of leaf **mass** dies, the same
fraction of leaf **area** dies. The alternative — ``rdr_leaf · leaf_C · sla / A`` —
re-imports the fixed specific leaf area that this mechanism exists to make emergent,
and once area and mass have decoupled it removes the wrong amount (measured: it lands
``open_season``'s minimum LAI on exactly 0.0). Measured either way, the whole spread
between the two is this one term: peak LAI 5.9129 vs 5.0257, peak W 14.5554 vs
13.7138.

Pure stdlib; no third-party imports (the core-purity invariant).

Sources:
  [F] Soltani, A. & Sinclair, T.R., *Modeling Physiology of Crop Development, Growth
      and Yield*, CABI. **Ch. 9** (Eqns 9.1-9.5, 9.8; Box 9.1; Box 9.2 — the
      authoritative order of operations), **Ch. 15** (Eqn 15.5 ``WSFL``, Eqns
      15.6/15.7 and the species sentence that picks between them, Table 15.1 wheat
      ``WSSL = 0.40``), **Ch. 12** (Fig. 12.4 the "Run" sheet and Fig. 12.5 the
      "Crops" sheet — the book's OWN working wheat parameterization, cv. Tajan).
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

from domains.biosphere.allocation import AllocationParams, partition
from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.carbon_budget import CarbonContext
from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.transpiration import soil_water_stress
from simcore.auxiliary import AuxId, AuxProcess
from simcore.environment import Environment
from simcore.ids import StockId
from simcore.state import State

# [F] Eqn 9.5's cm^2-per-plant -> m^2-per-plant conversion, written where the equation
# is rather than folded into a parameter: PLACON is tabulated in cm^2 and PLAPOW is a
# bare exponent, so folding would silently re-scale a cited value.
_CM2_PER_M2 = 10_000.0


@dataclass(frozen=True)
class LeafAreaParams:
    """Crop parameters for the node-driven leaf-area branch ([F] Ch. 9)."""

    phyllochron: float  # PHYL, degC per leaf/node
    pla_constant: float  # PLACON, cm^2 (plant leaf area at one node)
    pla_exponent: float  # PLAPOW, dimensionless
    tu_tlm: float  # tuEMRTLM, degC.day from emergence to TLM
    wssl: float  # WSSL, threshold FTSW for leaf-area development
    slw_fraction_min: float  # [E] Table 20 SLT min -> the AREA CEILING (thinnest leaf)
    slw_fraction_max: float  # [E] Table 20 SLT max -> the AREA FLOOR (thickest leaf)


def main_stem_nodes(thermal_time: float, *, phyllochron: float) -> float:
    """``MSNN = 1 + CTU/PHYL`` — main-stem node number, DERIVED ([F] Eqns 9.1-9.2).

    [F] Box 9.2 initialises ``MSNN = 1`` and adds ``INODE = DTU/PHYL`` on every step
    from emergence, which integrates to this closed form for a constant phyllochron.
    ``thermal_time`` is our emergence-based accumulator, i.e. [F]'s ``CTU - tuEMR``.
    """
    if not phyllochron > 0.0:
        raise ValueError(f"phyllochron must be > 0 degC/node, got {phyllochron!r}")
    return 1.0 + thermal_time / phyllochron


def plant_leaf_area(nodes: float, *, pla_constant: float, pla_exponent: float) -> float:
    """``PLA = PLACON · MSNN^PLAPOW`` (cm² per plant) — [F] Eqn 9.3, the allometry.

    [F] p. 107 notes the fit deviates most at large ``MSNN`` and that this is
    "generally satisfactory because the calculation of light interception is done
    using an exponential equation", i.e. the error is absorbed by Beer-Lambert
    saturation — which is exactly how our canopy consumes it.
    """
    return pla_constant * nodes**pla_exponent


def node_area_growth_rate(
    thermal_time: float,
    thermal_time_rate: float,
    *,
    params: LeafAreaParams,
    plant_density: float,
    water_factor: float,
) -> float:
    """``GLAI`` for the node-driven branch (m²/m² per day) — [F] Eqn 9.5 × Eqn 15.7.

    [F] differences ``PLA`` between consecutive days; this differentiates it instead::

        dPLA/dt = PLACON · PLAPOW · MSNN^(PLAPOW-1) · (dCTU/dt) / PHYL

    ⚠ **The analytic derivative is a deliberate delta from the source**, taken to keep
    the aux channel's ``dt``-independence contract (``rate`` must not depend on ``dt``
    — see ``simcore.auxiliary``); a day-over-day difference is a ``dt = 1`` object. The
    two agree to O(dt) and exactly in the limit, and at the frozen ``dt = 1`` the
    difference is the second-order term of a smooth power law. This is the same shape
    of choice ``root_depth`` recorded for its cap, and the Rust mirror carries it
    rather than re-deciding it.

    ``water_factor`` is ``WSFL`` ([F] Eqn 15.5), supplied by the caller so this stays a
    bare rate law.
    """
    nodes = main_stem_nodes(thermal_time, phyllochron=params.phyllochron)
    dpla = (
        params.pla_constant
        * params.pla_exponent
        * nodes ** (params.pla_exponent - 1.0)
        * thermal_time_rate
        / params.phyllochron
    )
    return dpla * plant_density / _CM2_PER_M2 * water_factor


def thickness_envelope(
    leaf_c: float,
    *,
    sla_per_mol_c: float,
    ground_area: float,
    params: LeafAreaParams,
) -> tuple[float, float]:
    """``(floor, ceiling)`` on LAI from [E] Table 20's measured leaf thickness.

    ⚠ **THE FRACTIONS INVERT.** [E]'s ``SLT`` is specific leaf *weight* — mass per area
    — as a fraction of the Table 19 constant, so the **smallest** fraction is the
    **thinnest** leaf and therefore the **largest** area the standing mass can be spread
    over. Reading the pair the other way round would still run; it would simply pin the
    canopy to the wrong side of the envelope, which is why :func:`loader.
    load_leaf_area_params` checks the ordering rather than trusting the names.

    The carbon-derived area ``leaf_C · SLA / A`` is the frozen form this tree already
    ships, so the envelope is a band *around the reference science*, and the reference
    science is what it collapses to wherever it binds.

    ⚠ **THE PAIR IS [E]'s MIXTURE ANSWER, NOT A LOOSE STAND-IN FOR ONE.** [E] applies
    ``SLT`` to *new* leaf area and this applies it to the canopy average, which was
    recorded as an exposure ("the honest envelope would clip more") and then **refuted
    by deriving it**. [E] Listing 3 lines 88-92 — new area ``GLV/SLN``, area loss at the
    canopy average ``LLV/SLA``, and ``SLA = WLV/ALV`` emergent — reduce, for
    ``S ≡ W/A``, to ``dS/dt = (GLV/A)·(1 − S/SLN)``, in which the senescence terms
    cancel exactly. With ``GLV ≥ 0`` the interval ``SLC·[min SLT, max SLT]`` is
    **forward-invariant for any growth history**, and tight. No scenario-independent
    narrowing exists, because the partition table creates leaf mass across the whole DS
    range containing both extremes. The full derivation is §9 of
    ``docs/plans/post-roadmap-leaf-expansion.md``; the **Rust mirror carries the rule,
    not the rationale**, so it is stated here rather than left to be re-derived.
    """
    derived = leaf_area_index(
        leaf_c, sla_per_mol_c=sla_per_mol_c, ground_area=ground_area
    )
    return derived / params.slw_fraction_max, derived / params.slw_fraction_min


@dataclass(frozen=True)
class LeafAreaExpansion:
    """``AuxProcess`` advancing the ``leaf_area_index`` accumulator (the fourth one).

    Returns ``{accumulator: (GLAI - DLAI)·dt}`` — [F] Eqn 9.8 — where ``GLAI`` is the
    node-driven branch below ``tu_tlm`` and the carbon-driven branch at or above it,
    and ``DLAI`` is our own ``Senescence`` re-expressed in area terms.

    ⚠ **The carbon-driven branch RECOMPUTES ``Allocation``'s leaf leg** through the
    shared :class:`CarbonContext` rather than observing it — an aux process sees only
    the step-entry snapshot. That is the same recompute ``Allocation`` itself performs
    (structural agreement, not disciplined agreement), and it carries the same
    exposure: under **arbitration scaling** the realised leaf delta is smaller than the
    demand recomputed here, so area and mass drift apart by the rationed fraction.

    ⚠⚠ **THAT PARAGRAPH HAS NOW BEEN WRONG ONCE IN EACH DIRECTION, AND BOTH ARE KEPT.**
    Draft 1 said the drift was "gated rather than merely hoped about, because every
    golden asserts ``rationed == 0``". Draft 2 corrected it: with the node branch alone,
    three sealed scenarios rationed (27 / 41 / 83 firings), so the guard was red in
    exactly the runs where the drift mattered. With the envelope below in place the
    guard is green again — **0 firings on all eleven scenarios, and 0 across every
    envelope in the sensitivity sweep**. The claim is therefore restored, but it is
    restored as a *measured* claim with its own falsification on the record beside it,
    because what made draft 1 dangerous was never the conclusion — it was asserting a
    guard's colour without running it.

    **The envelope, and why the mechanism above cannot ship without it** ([E] Table 20).
    [F] Ch. 9 scopes itself to leaf area "under non-limiting water and nutrients" and
    Ch. 12 is titled "A Model for *Potential* Production"; [F] has no mechanism by which
    the atmosphere runs out of carbon, because in its world it cannot. Six of our eleven
    scenarios are growth-limited, and there the node branch makes **area without
    carbon** — measured leaf thinness 2–15× nominal, six scenarios peaking at *exactly*
    2.9068 (the end of the node curve) because below ``TLM`` there is no carbon feedback
    at all. :func:`thickness_envelope` bounds the standing canopy to [E]'s measured
    winter-wheat leaf thickness, and where it binds this model **reduces to the frozen
    carbon-derived form** — the reference science already in this tree.

    ⚠ **THE ENVELOPE DECIDES A DISAGREEMENT BETWEEN TWO SHELVED SOURCES, NOT A DETAIL.**
    [F] Table 15.1 makes leaf expansion *more* drought-sensitive than growth
    (``WSSL`` 0.40 > ``WSSG`` 0.30) — drought thickens leaves. [E] p. 100 reports that
    irrigation and fertilization have "little effect on specific leaf weight". Measured
    on ``water_biting``, the only scenario where ``WSFL`` fires hard (below 1 on 100 %
    of days, minimum 0.1250): switching ``WSFL`` off moves peak LAI **+215.0 % under
    [F] alone and +0.5 % under the envelope**. The envelope makes [E] win, and that is
    a science choice, stated here so it is not mistaken for a bound's side effect.
    ⚠ It is also *why* ``WSFL``'s leverage is small — not evidence that drought does not
    matter. And on five of seven scenarios ``WSFL`` never fires at all (``FTSW`` never
    reaches 0.40), **including the one named ``drought``**, which is a coverage fact
    about the roster rather than a physiological one.

    ⚠ **THE CLAMP IS ACTIVE 55–85 % OF DAYS, AND ``a-clamp-hides-a-wrong-amount`` IS THE
    FIRST THING A REVIEWER WILL REACH FOR.** The defence, stated rather than left to be
    re-derived: that lesson is about a clamp standing in for an amount nobody measured,
    which survives until the scale changes. This one is a *cited physiological bound*
    whose binding branch is the tree's own reference form, and it was measured across
    envelopes from ``(1.00, 1.00)`` to ``(0.75, 2.00)`` — ``open_season`` peak LAI moves
    only 4.91–5.58 and every envelope clears the Greenwood crossing with zero rationing,
    while *removing* it moves ``n_limited`` by 30× and fails the gate. The numbers pick
    a point inside a flat region.

    ⚠ **THE PROJECTION IS A STATE CONSTRAINT, NOT A RATE LAW, AND SO IT IS NOT
    ``dt``-INDEPENDENT.** In the unclamped branch ``evaluate`` returns ``rate·dt`` and
    honours the aux channel's contract; in the clamped branch it returns
    ``ceiling − LAI``, an absolute delta whose implied rate depends on ``dt``. That is
    deliberate and it is the correct discretisation of a *bound on the state*: at any
    step size the result never violates the envelope, which a ``dt``-independent rate
    cannot promise. It is the same shape of choice ``root_depth`` recorded for its cap,
    and — like the analytic derivative above — **the Rust mirror carries the rule, not
    the rationale**, so it is written here rather than left to coincide at ``dt = 1``.

    ⚠ **"INSIDE THE ENVELOPE BY CONSTRUCTION" WOULD BE ~3 % FALSE.** The bound reads
    leaf carbon from the **step-entry** snapshot while the delta lands after it, so the
    canopy can overshoot by one step's leaf growth. The honest claim is "inside [E]'s
    range to within one step's growth". The same one-step lag is why the degenerate
    ``(1.00, 1.00)`` envelope does **not** reproduce the frozen form exactly: it lands
    10.1 % low on ``open_season`` peak LAI (4.9106 against 5.4624). That control is
    worth keeping precisely because it puts a number on the lag.
    """

    id: AuxId
    accumulator: str  # the aux name written, e.g. "leaf_area_index"
    ctx: CarbonContext
    thermal_time_aux: str
    params: LeafAreaParams
    pheno: PhenologyParams
    alloc: AllocationParams
    rdr_leaf: float  # Senescence's own relative leaf death rate (1/day)
    plant_density: float  # PDEN, plants m-2 — SCENARIO/management data (P4)
    soil_water: StockId
    rooted_depth_aux: str
    thermal_time_rate: AuxProcess  # the tree's ThermalTimeAccumulation, for its DTU

    def _water_factor(self, snapshot: State, env: Environment) -> float:
        """``WSFL = min(1, FTSW/WSSL)`` — [F] Eqn 15.5.

        The same ``FTSW`` every other consumer computes, against ``WSSL`` rather than
        ``WSSG``: [F] Table 15.1 gives wheat 0.40 for leaf-area development and 0.30
        for growth, i.e. **leaf expansion is the more sensitive process**, which is the
        whole reason it carries a factor of its own.
        """
        return soil_water_stress(
            snapshot.stocks[self.soil_water].amount,
            snapshot.aux.get(self.rooted_depth_aux, 0.0),
            soil_extractable_water=self.ctx.soil_extractable_water,
            ground_area=self.ctx.ground_area,
            threshold=self.params.wssl,
        )

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        thermal_time = snapshot.aux.get(self.thermal_time_aux, 0.0)
        lai = snapshot.aux.get(self.accumulator, 0.0)
        if thermal_time < self.params.tu_tlm:
            # SINK-limited: area from node number, independent of dry matter. The DTU
            # comes from the tree's own accumulator process, so the node clock and the
            # development clock can never disagree about the day's temperature unit.
            rate = self.thermal_time_rate.evaluate(snapshot, env, 1.0)[
                self.thermal_time_aux
            ]
            glai = node_area_growth_rate(
                thermal_time,
                rate,
                params=self.params,
                plant_density=self.plant_density,
                water_factor=self._water_factor(snapshot, env),
            )
        else:
            # SOURCE-limited: [F] Eqn 9.6, GLAI = GLF · SLA. Our GLF is Allocation's
            # leaf share of DMI; the deficit is already in it through the limitation
            # factor on gross assimilation, which is why [F] applies no WSFL here.
            _, _, available = self.ctx.budget(snapshot, env)
            dmi = self.ctx.resp.growth_efficiency * available
            dvs = development_stage(
                thermal_time,
                tsum_anthesis=self.pheno.tsum_anthesis,
                tsum_maturity=self.pheno.tsum_maturity,
            )
            leaf_rate, _, _, _ = partition(dmi, dvs, self.alloc.table)
            glai = leaf_rate * self.ctx.canopy.sla_per_mol_c / self.ctx.ground_area
        # DLAI — our Senescence, in area terms. Relative, so positivity is structural
        # (the loss vanishes as the canopy does) and no clamp is needed anywhere.
        dlai = self.rdr_leaf * lai
        # ...and then [E]'s envelope, which is the only reason any of the above is
        # usable outside potential production. PROJECTION, not a rate law — see the
        # class docstring's dt note.
        floor, ceiling = thickness_envelope(
            snapshot.stocks[self.ctx.leaf_c].amount,
            sla_per_mol_c=self.ctx.canopy.sla_per_mol_c,
            ground_area=self.ctx.ground_area,
            params=self.params,
        )
        target = min(max(lai + (glai - dlai) * dt, floor), ceiling)
        return {self.accumulator: target - lai}
