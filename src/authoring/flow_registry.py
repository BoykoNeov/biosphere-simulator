"""The author-selectable frozen-flow surface (the thing Step 7 freezes).

A scenario file selects frozen ``Flow`` types by name; this module is the explicit
mapping from an authoring type name → the frozen class + its wiring/param shape.

**Explicit, not introspected — by design.** A stock-id field is a plain ``str``
alias at runtime (``StockId = NewType("StockId", str)``), so field-type
introspection cannot tell a wiring field from any other string field. More to the
point, this registry *is* the authoring contract — the declared set of frozen
primitives a scenario may compose, and the wiring names it exposes for each. It
mirrors frozen constructor signatures; the signatures it mirrors are frozen, so the
"duplication" is a stable, deliberately-curated public surface (frozen in Step 7),
not incidental drift.

Step 0 registered the standalone **Crew** flows (the composition anchor); the
post-roadmap **Tier-1 unfreeze** grew this to the nine standalone Power / Thermal /
ECLSS flows — the rest of the frozen flow set that *fits this shape*
(``docs/plans/post-roadmap-flow-registry-growth.md``). A flow type's **param set** (its
``param_set`` name, a key into :data:`PARAM_LOADERS`, or ``None`` for a param-free
flow) is a fixed fact of the class — it names *which frozen loader* produces the
flow's params object. "Which concrete param values" is the per-flow authoring choice
(Step 1): the named default committed file, or a **parameter pack** — a param file in
the same ``{value, unit, source}`` schema that the *same frozen loader* reads (so a
pack's values are validated by the frozen bounds/unit guards, never bypassing them).

**The biosphere is deliberately absent, for a STRUCTURAL reason** (not a calibration
one): ``Allocation`` takes a composite ``ctx: CarbonContext`` bundling four param
objects with four stock ids, plus ``pheno``/``alloc`` on top — which a flat
``wiring_fields`` tuple + a single ``param_set`` cannot express. It also needs the aux
accumulator, the shared ``co2_pool`` feedback var and the two-rate master-day driver,
all deferred. It wants a *frozen-compartment include*, not flow-type entries.

**Registered ≠ calibrated.** Selecting a frozen type here means the *rate law* is
frozen and literature-derived, **not** that its numbers are validated: every param
these flows read except ``crew``'s two is a ``TODO(cite)`` placeholder pending the
deferred validation gate. A scenario built only from frozen types carries no
``has_authored_kinetics`` marker — which means "no authored kinetics", **not**
"validated". See ``docs/authoring-reference.md``, "Frozen is not calibrated".
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from domains.crew.flows import (
    FoodMetabolism,
    OxygenConsumption,
    WaterBalance,
)
from domains.crew.loader import load_crew_params
from domains.eclss.flows import (
    CO2Scrubber,
    Condenser,
    CrewMetabolism,
    O2Makeup,
)
from domains.eclss.loader import load_eclss_params
from domains.power.flows import (
    LoadDraw,
    SelfDischarge,
    SolarCharge,
)
from domains.power.loader import load_charge_params, load_self_discharge_params
from domains.thermal.flows import HeatInput, RadiatorReject
from domains.thermal.loader import load_thermal_params
from simcore.flow import Flow


@dataclass(frozen=True)
class FlowTypeSpec:
    """How one authoring flow-type name lowers to a frozen ``Flow`` constructor.

    ``cls`` is the frozen flow dataclass (typed ``Callable[..., Flow]`` because
    ``Flow`` is a structural ``Protocol``, not a nominal base — each concrete
    dataclass is a callable that returns a ``Flow``-conforming instance);
    ``wiring_fields`` the exact set of constructor keyword fields that take a
    ``StockId`` (the interpreter requires the scenario's ``wiring`` dict to match
    this set exactly); ``param_set`` names the frozen loader that produces this
    flow's params object (a key into :data:`PARAM_LOADERS`), or ``None`` for a
    param-free flow. Every frozen flow shares the ``(id, priority, *wiring[,
    params])`` shape.
    """

    cls: Callable[..., Flow]
    wiring_fields: tuple[str, ...]
    param_set: str | None


# The author-selectable frozen-flow surface. Keys are stable authoring type names
# (ASCII, dotted-namespace, matching the flows' own ids by convention). Step 0: the
# three standalone Crew flows. Tier 1: the nine standalone Power / Thermal / ECLSS
# flows. Grouped by domain for reading; the interpreter and the manifest both key by
# name, so the source order here is inert.
#
# FORCED vs DONOR-CONTROLLED is the fact an author most needs from this table, because
# it decides how positivity holds (see docs/authoring-reference.md):
#   * FORCED flows read a **hardcoded module forcing name** through ``env.get`` — NOT
#     through wiring. The scenario must declare a forcing under that exact name, and
#     the flow is therefore NOT multi-instanceable under a `{bundle, prefix}` include
#     (a namespaced forcing key is unreachable — the documented crew boundary).
#     Positivity is by SIZING (well-fed), never structural.
#   * DONOR-CONTROLLED flows read a stock, so their draw self-limits — positivity is
#     structural, but only while `k·dt < 1`. The frozen `k`s carry implicit `dt`
#     assumptions; an author picks `dt`. See the per-flow notes below.
FLOW_TYPES: dict[str, FlowTypeSpec] = {
    "crew.oxygen_consumption": FlowTypeSpec(
        cls=OxygenConsumption,
        wiring_fields=("o2_store", "o2_consumed"),
        param_set=None,
    ),
    "crew.food_metabolism": FlowTypeSpec(
        cls=FoodMetabolism,
        wiring_fields=("food_store", "exhaled_co2", "fecal_waste"),
        param_set="crew",
    ),
    "crew.water_balance": FlowTypeSpec(
        cls=WaterBalance,
        wiring_fields=("water_store", "crew_humidity", "urine"),
        param_set="crew",
    ),
    # --- Power (ENERGY, J; rates in W = J/s) ---------------------------------
    # FORCED on ``solar_power`` (W). Three legs: the η_c split names the charge-
    # conversion loss as heat. At η_c = 1 the heat leg is exactly 0 (still emitted).
    "power.solar_charge": FlowTypeSpec(
        cls=SolarCharge,
        wiring_fields=("solar_source", "battery", "waste_heat"),
        param_set="charge",
    ),
    # FORCED on ``load_power`` (W). 100 % dissipative — positivity by well-fed sizing
    # (a constant load CAN over-draw an empty battery; the backstop would fire).
    "power.load_draw": FlowTypeSpec(
        cls=LoadDraw,
        wiring_fields=("battery", "waste_heat"),
        param_set=None,
    ),
    # DONOR-CONTROLLED (leak ∝ battery). k = 1.0e-8 /s ⇒ k·dt ≈ 3.6e-5 at dt = 3600 s:
    # safe across any plausible authored dt (k·dt < 1 needs dt < 1.0e8 s ≈ 3 years).
    "power.self_discharge": FlowTypeSpec(
        cls=SelfDischarge,
        wiring_fields=("battery", "waste_heat"),
        param_set="self_discharge",
    ),
    # --- Thermal (ENERGY, J) --------------------------------------------------
    # FORCED on ``heat_load`` (W). Heat → heat, no form change ⇒ two legs, no loss leg.
    "thermal.heat_input": FlowTypeSpec(
        cls=HeatInput,
        wiring_fields=("heat_source", "node"),
        param_set=None,
    ),
    # DONOR-CONTROLLED and NONLINEAR (Stefan-Boltzmann T⁴). Positivity is structural
    # only AT THE FLOOR (R → 0 as Q → 0); near equilibrium it is by SIZING — the frozen
    # heat_capacity C = 1.0e7 J/K is sized so τ = C/(4εσA·T_eq³) ≈ 65 steps at
    # dt = 3600 s. A much larger dt overshoots. The one registered flow whose per-step
    # graph contains a transcendental (`**4`) ⇒ Tier-2 cross-port (tiers.json).
    "thermal.radiator_reject": FlowTypeSpec(
        cls=RadiatorReject,
        wiring_fields=("node", "space"),
        param_set="thermal",
    ),
    # --- ECLSS (OXYGEN mol / CARBON mol / WATER kg) ---------------------------
    # FORCED on THREE names at once (``o2_consumption`` / ``co2_production`` /
    # ``h2o_production``). Six legs across three quantities, each balanced
    # independently. The cabin pools it wires are SINGLE-QUANTITY (cabin_co2 is CARBON
    # with the 1:1 default composition, NOT {carbon:1, oxygen:2}) — atomic coupling is
    # the Phase-6 seam, so an authored cabin must not annotate composition here.
    "eclss.crew_metabolism": FlowTypeSpec(
        cls=CrewMetabolism,
        wiring_fields=(
            "cabin_o2",
            "cabin_co2",
            "cabin_h2o",
            "metabolic_o2_sink",
            "metabolic_co2_source",
            "metabolic_h2o_source",
        ),
        param_set=None,
    ),
    # DONOR-CONTROLLED. k_scrub = 1.0e-3 /s is sized for dt = 60 s (k·dt = 0.06). THE
    # sharpest authored-dt trap: at dt = 3600 s, k·dt = 3.6 > 1 — the draw exceeds the
    # stock and the Euler backstop rations (silently, unless the author reads
    # `rationed`). See docs/authoring-reference.md, "The dt constraint".
    "eclss.co2_scrubber": FlowTypeSpec(
        cls=CO2Scrubber,
        wiring_fields=("cabin_co2", "co2_removed"),
        param_set="eclss",
    ),
    # DONOR-CONTROLLED. k_cond = 5.0e-4 /s, likewise sized for dt = 60 s (k·dt = 0.03);
    # k·dt = 1.8 > 1 at dt = 3600 s.
    "eclss.condenser": FlowTypeSpec(
        cls=Condenser,
        wiring_fields=("cabin_h2o", "humidity_condensate"),
        param_set="eclss",
    ),
    # DEMAND-CONTROLLED toward o2_setpoint (the one flow of this shape). Not clamped:
    # above the setpoint the frozen law goes NEGATIVE, reversing the flow's direction —
    # the frozen "above-setpoint venting clamp is a deferred seam" boundary, reachable
    # by an author who wires cabin_o2 above the frozen 10.0 mol setpoint.
    "eclss.o2_makeup": FlowTypeSpec(
        cls=O2Makeup,
        wiring_fields=("o2_supply", "cabin_o2"),
        param_set="eclss",
    ),
}


# Named frozen param loaders. Each takes an **optional path** (defaulting to the
# committed frozen param file). A flow references its set by name (``params: crew``
# → the loader's default file) or supplies a **pack** (``params: {pack: …}`` → the
# same loader called with the pack's path), so a pack's values flow through the
# frozen loader's schema/bounds/unit validation — a pack is a param file, not a way
# around the guards.
PARAM_LOADERS: dict[str, Callable[..., object]] = {
    "crew": load_crew_params,
    # The Power self-discharge rate set (P5.5). Registered for Step 2's
    # authored-kinetics anchor: a ``DeclarativeFlow`` re-expressing ``SelfDischarge``
    # reads its ``k`` from this frozen loader (``param("self_discharge_rate")``), so the
    # authored value is the frozen one — bit-identical — passing the frozen guards.
    "self_discharge": load_self_discharge_params,
    # --- Tier 1: the sets the newly-registered flow types name ----------------
    # Each is reachable BOTH as a frozen type's `param_set` and — since every loader
    # returns a flat dataclass of floats — as a `kinetics` rate's `param("…")` source,
    # exactly like `self_discharge`. So registering these also widens the authored-
    # kinetics surface: an authored rate may now read η_c, the radiator properties, or
    # the ECLSS gains and get the *frozen* value through the *frozen* guards.
    # charge: charge_efficiency (η_c).
    "charge": load_charge_params,
    # thermal: emissivity / radiator_area / heat_capacity / space_temperature.
    "thermal": load_thermal_params,
    # eclss: co2_scrub_rate / condense_rate / o2_makeup_gain / o2_setpoint.
    "eclss": load_eclss_params,
}


def load_param_set(param_set: str, pack_path: Path | None) -> object:
    """Load a flow type's params: the committed default, or a pack file.

    ``param_set`` is the flow type's :attr:`FlowTypeSpec.param_set`; ``pack_path``
    is ``None`` for the committed default or an already-resolved path to a pack file
    (a param file in the frozen loader's own ``{value, unit, source}`` schema). The
    pack is read by the *same frozen loader*, so its values are validated identically.
    """
    loader = PARAM_LOADERS[param_set]
    return loader() if pack_path is None else loader(pack_path)
