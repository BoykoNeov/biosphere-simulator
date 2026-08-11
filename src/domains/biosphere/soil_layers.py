"""Root-zone capture (``EWAT``) — the water side of rooted depth (post-roadmap).

**What this is.** The one flow that makes rooting depth *do work*. Water in the soil
below the current rooted depth is physically present and currently **unreachable**; as
the roots extend, the newly explored soil releases its extractable water into the root
zone. [F] Soltani & Sinclair Ch. 14 names the quantity and gives the law:

    "As the depth of the soil explored by roots increases, there is an increase in the
    amount of water available to the crop **if the deeper soil contains water**."

    ``EWAT = min(GRTD · EXTR, WSTORG)``                                    (Eqn 14.10)

Here ``WSTORG`` is our ``subsoil_water`` POOL ("the soil water stored below rooting
zone", Fig. 14.2), ``GRTD`` is the gated extension rate — **read from
:func:`root_depth.extension_rate`, never recomputed** — and ``EXTR`` the volumetric
extractable soil water. Per unit ground area the depth increment ``GRTD·dt`` (m) times
``EXTR`` (m³ m⁻³) is a water depth (m); times ρ_water and ``ground_area`` it is kg, the
canonical WATER unit. So::

    flux [kg] = min(GRTD · dt · EXTR · ρ_water · ground_area,  subsoil_water)

**Why two stores and not N layers, with the citation.** [F] opens its own soil-water
chapter by settling the resolution question: *"for models attempting to simulate crop
growth and yield as is the objective of this book, a two-layered soil or even a
one-layer soil seems satisfactory (Robertson and Fukai, 1994)"* — and then specifies the
two stores as the root zone (which grows) and the water below it. The discretization the
earlier price doc assumed would be necessary is not what the source does.

**⚠ THE ``min`` IS A CITED CLAMP, AND IT IS THE ONE NON-``dt``-LINEAR TERM HERE.**
[F] Box 14.1 writes it as ``EWAT = GRTD * EXTR : If EWAT > WSTORG Then EWAT = WSTORG``.
It can only bite on the step that empties the donor, because ``extension_rate`` already
returns 0 once ``subsoil_water <= 0`` — so the clamp is the *closing* step of a store
whose depletion is otherwise gated upstream. It is kept rather than dropped because
without it a single step could overdraw the donor, which the arbitration backstop would
have to catch, and every golden run asserts that backstop fires **zero** times. The
biosphere is frozen Euler at ``dt = 1``; under a higher-order integrator this term is a
kinetic nonlinearity, not an arbitration event, and behaves like the other ``max(0, …)``
clamps the tree already carries.

**What is NOT here, deliberately** (see ``docs/plans/post-roadmap-soil-layers.md``):
drainage (Eqn 14.11 — ``WSTORG``'s only *input* in [F], so our below-root store is
one-way within a season), runoff, soil evaporation, and the ``FTSW = ATSW/TTSW`` stress
conversion. The re-sow return that keeps the one-way capture from ratcheting across
years lives in ``season.annual_reset`` — [F] is single-season and silent there, so that
rule is **ours**, derived from conservation-plus-geometry.

Pure stdlib; no third-party imports (the core-purity invariant).
"""

from __future__ import annotations

from dataclasses import dataclass

from domains.biosphere.phenology import PhenologyParams
from domains.biosphere.photosynthesis import PhotosynthesisParams
from domains.biosphere.root_depth import RootDepthParams, extension_rate
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State

# Water density, kg m⁻³. The same identity ``transpiration.py`` states in words ("at
# water density 1000 kg m⁻³, 1 mm of depth over 1 m² is exactly 1 kg"), written as a
# constant here because this flow multiplies a *soil* depth by a volumetric water
# content, so the metres do not cancel to millimetres on their own.
WATER_DENSITY: float = 1000.0


def captured_water(
    depth_increment: float, *, soil_extractable_water: float, ground_area: float
) -> float:
    """The water a newly explored soil column of thickness ``depth_increment`` holds.

    ``m · (m³ m⁻³) · kg m⁻³ · m² = kg``. Shared with ``season.annual_reset``, which
    applies it in reverse when a re-sow abandons a root zone — the two must use one
    formula or a season stops being a closed cycle.
    """
    return depth_increment * soil_extractable_water * WATER_DENSITY * ground_area


@dataclass(frozen=True)
class RootZoneCapture:
    """WATER flow ``subsoil_water -> soil_water`` (``EWAT``; balanced in water).

    An internal transfer between two in-system soil stocks: no boundary is crossed, so
    it changes no ``Inputs``/``Outputs`` term and only re-labels which water the crop
    can reach. Donor-clamped per [F] Eqn 14.10.

    The gate fields mirror :class:`root_depth.RootDepthExtension` exactly because both
    call :func:`root_depth.extension_rate` with the same snapshot reads — that is the
    point of the shared function, not duplication that drifted.
    """

    id: FlowId
    priority: int
    subsoil_water: StockId
    soil_water: StockId
    rooted_depth_aux: str
    thermal_time_aux: str
    temp_var: str
    params: RootDepthParams
    photo: PhotosynthesisParams
    pheno: PhenologyParams
    sw_wilting: float
    sw_critical: float
    soil_depth: float
    soil_extractable_water: float
    ground_area: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        available = snapshot.stocks[self.subsoil_water].amount
        rate = extension_rate(
            snapshot.aux.get(self.rooted_depth_aux, 0.0),
            snapshot.aux.get(self.thermal_time_aux, 0.0),
            env.get(self.temp_var),
            snapshot.stocks[self.soil_water].amount,
            available,
            params=self.params,
            photo=self.photo,
            pheno=self.pheno,
            sw_wilting=self.sw_wilting,
            sw_critical=self.sw_critical,
            soil_depth=self.soil_depth,
        )
        demand = captured_water(
            rate * dt,
            soil_extractable_water=self.soil_extractable_water,
            ground_area=self.ground_area,
        )
        flux = demand if demand < available else available  # [F] Eqn 14.10's min
        return FlowResult(
            legs=(Leg(self.subsoil_water, -flux), Leg(self.soil_water, flux))
        )
