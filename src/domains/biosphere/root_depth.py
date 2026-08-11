"""Rooted-depth extension (the third aux accumulator) + the root-zone access fraction.

**What this is.** Rooted depth — *"the depth from which the crop effectively extracts
water"* ([E] p. 136) — advanced as a non-conserved accumulator, and the fraction of the
reference soil layer that depth has reached. That fraction gates ``NitrogenUptake``'s
supply term, so a crop that has not yet explored the layer cannot draw all of its N.

**⚠ IT IS DELIBERATELY NOT A FUNCTION OF ROOT CARBON, and that is the primary's call,
not ours.** [E] p. 136, in the paragraph that introduces the extension rate:

    "The length of fibrous roots can vary enormously without much impact on root
    weight. Hence, simulation of rooted depth occurs independently of the growth of
    root mass."

`[F]` Soltani & Sinclair reach the same design independently (``DEPORT_i =
DEPORT_{i-1} + GRTD`` from a crop constant). So ``ROOT_C`` remains uncoupled here **by
citation**, and anyone arriving to "finally make root carbon do work" should read
``docs/plans/post-roadmap-root-functional-coupling.md`` first: that route was measured
and refused, and this module is what was built instead — at the user's explicit
direction, over that refusal, with the inertness below known in advance.

**⚠ THIS MECHANISM IS BIT-IDENTICALLY INERT ON EVERY FROZEN SCENARIO.** Measured, not
assumed: across the manifest's whole roster at its own horizons (including both 15-year
runs), for initial depths 0.0 and 0.15 m, with and without the re-sow reset, and
combined with a 5x-lower uptake ceiling. The reason is structural — nitrogen uptake is
**demand-bound on every step of every scenario**, and this gate shrinks *supply*, which
carries >=1.9x headroom at its tightest. Depth also saturates the gate by ~day 20 while
demand peaks near day 210, so the gate is anti-correlated with the need for it. **No
golden can catch this mechanism's removal**, which is why its pins are unit-level and
mutation-verified rather than regression-level.

**The law** ([E] p. 137, Listing 7 Line 34 — ``GZRT = GZRTC * WSERT * TERT``)::

    d(depth)/dt = max_extension_rate · f_water · f_temp     (m/day)
    FROOT1      = min(depth / soil_layer_depth, 1)

[E] states what the two factors are, so neither is a new response curve of ours:

    "the effect of temperature on root extension is supposed to equal that of
    photosynthesis. The effect of water stress on the rate of increase in rooted depth
    is supposed to equal that of water uptake"

— hence ``f_temp`` is :func:`photosynthesis.temperature_factor` and ``f_water`` is
:func:`transpiration.water_stress_factor`, the same two the rest of the tree uses. And
([E] p. 136) *"Root growth generally stops around flowering"*, so the rate is zero at
``DVS >= 1``.

⚠ **The flowering stop never binds for the frozen winter wheat** — the 1.3 m cap is
reached near day 140 while anthesis falls near day 255. It is implemented because the
source states it, not because it is load-bearing here; it *would* bind for a crop with
a deeper cap or a shorter season.

**The cap form is a deliberate choice, recorded rather than made silently.** The aux
channel's contract is that ``rate`` be ``dt``-independent, and ``min(rate·dt, cap −
depth)`` breaks it. This module instead cuts the **rate to zero once the cap is
reached**, which keeps the contract and overshoots by at most one step's extension
(0.018 m on a 1.3 m cap, 1.4 %) at the frozen ``dt = 1``. The Rust mirror carries the
same choice — per ``port-mirror-carries-rule-not-rationale``, the port does not
re-decide it.

⚠ **DELTA against `[F]`'s ``SNAVL``, stated not glossed.** Soltani's availability is
``(NCON − 1e-6) · ATSW1 · 1000 · FROOT1`` — a solution *concentration*, times the top
layer's *transpirable water*, times the depth fraction. We keep only ``FROOT1`` and
apply it to our own ``soil_n_availability`` ramp, because our soil N is a single
undifferentiated POOL with no layer, no solution concentration and no per-layer water.
So this is **one factor of a three-factor product applied to a different object** —
faithful in shape, not a transcription. ``soil_layer_depth`` is carried as
SCENARIO/soil data (like ``ground_area`` and the ``sn_*`` thresholds), which is the
honest way to say "we are declaring the pool to *be* this layer".

Pure stdlib; no third-party imports (the core-purity invariant).
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.photosynthesis import PhotosynthesisParams, temperature_factor
from domains.biosphere.transpiration import water_stress_factor
from simcore.auxiliary import AuxId
from simcore.environment import Environment
from simcore.ids import StockId
from simcore.state import State


@dataclass(frozen=True)
class RootDepthParams:
    """Crop rooting parameters ([E] Table 25, read off the page image)."""

    max_extension_rate: float  # m/day, in moist soil at optimum temperature
    max_rooted_depth: float  # m


def root_zone_fraction(rooted_depth: float, *, soil_layer_depth: float) -> float:
    """``FROOT1 = min(depth/layer, 1)`` — the layer fraction the roots have reached.

    `[F]` Soltani & Sinclair's ``FROOT1 = min(DEPORT / DEP1, 1)``, clamped to
    ``[0, 1]``: 0 for a crop with no root system yet, 1 once the layer is fully
    explored. Structural
    positivity — this is a multiplicative gate on a supply term, so it can only ever
    reduce, never reverse, a flow (it cannot manufacture a ``ReversedFlowError``).

    ``soil_layer_depth`` must be > 0; the loader/scenario boundary enforces it.
    """
    if rooted_depth <= 0.0:
        return 0.0
    fraction = rooted_depth / soil_layer_depth
    return fraction if fraction < 1.0 else 1.0


@dataclass(frozen=True)
class RootDepthExtension:
    """``AuxProcess`` advancing the ``rooted_depth`` accumulator (the third one).

    Returns the per-step increment ``{accumulator: rate · dt}`` in the same increment
    form as ``ThermalTimeAccumulation``, where ``rate = max_extension_rate · f_water ·
    f_temp``, zero at/after flowering and zero once the cap is reached (see the module
    docstring for why the cap is a rate cut-off rather than an increment clamp).

    Reads DVS from the ``thermal_time`` accumulator on the snapshot — the same
    derived-not-stored idiom :func:`phenology.development_stage` exists for, so no
    second DVS accumulator is introduced.
    """

    id: AuxId
    accumulator: str
    thermal_time_aux: str
    temp_var: str
    soil_water: StockId
    params: RootDepthParams
    photo: PhotosynthesisParams
    pheno: PhenologyParams
    sw_wilting: float
    sw_critical: float

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        depth = snapshot.aux.get(self.accumulator, 0.0)
        if depth >= self.params.max_rooted_depth:
            return {self.accumulator: 0.0}
        dvs = development_stage(
            snapshot.aux.get(self.thermal_time_aux, 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        if dvs >= 1.0:  # [E]: "Root growth generally stops around flowering"
            return {self.accumulator: 0.0}
        f_temp = temperature_factor(
            env.get(self.temp_var),
            t_min=self.photo.t_min,
            t_opt_lo=self.photo.t_opt_lo,
            t_opt_hi=self.photo.t_opt_hi,
            t_max=self.photo.t_max,
        )
        f_water = water_stress_factor(
            snapshot.stocks[self.soil_water].amount,
            sw_wilting=self.sw_wilting,
            sw_critical=self.sw_critical,
        )
        rate = self.params.max_extension_rate * f_water * f_temp
        return {self.accumulator: rate * dt}
