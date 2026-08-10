"""The humification split — a carbon-use efficiency for the decomposer chain.

Post-roadmap, 2026-08-10 (``docs/plans/post-roadmap-cue-humification.md``). The seam the
soil-fractionation diagnosis named as its own replacement, and the obstacle option (B)
hit one flow over: **our decomposer chain had a carbon-use efficiency of 1.0**, so every
carbon atom that left the litter pool arrived intact in ``microbial_carbon`` and every
one that left ``microbial_carbon`` arrived intact in the atmosphere. Nothing was ever
*stabilised*, so a slow soil pool could be seeded but never refilled (fractionation
finding 3, measured over 4,575 steps).

**What the frozen form was asserting.** CENTURY (Parton et al. 1987) splits every
decomposer flux between CO₂ and the pool the remainder stabilises into. Read in its
variables, the frozen tree asserted a litter CO₂ fraction of **0.0** (against a measured
0.45–0.55) and ``Es = 1.0`` — and eq. [6] ``Es = 0.85 − 0.68·T`` **cannot reach 1.0 at
any texture**, its range over ``T ∈ [0, 1]`` being ``[0.17, 0.85]``. That is the shape
bucket-3 scope C found for the decomposer *rates*, one level down: the citation covered
the **rate** and never covered **where the decayed carbon goes**.
``microbial_respiration_rate`` is anchored to CENTURY's K5, and K5 *is* a **decay** rate
(eq. [5], "the decay rate of active SOM"); ``Es`` partitions the flow it drives.

**The structure (three flows, one partition each).**

* ``Decomposition``    — ``litter_carbon``    → CO₂ 0.45 + ``microbial_carbon`` 0.55
* ``MicrobialRespiration`` — ``microbial_carbon`` → CO₂ 0.85 + ``humus_carbon`` 0.15
* ``HumusDecomposition`` (here) — ``humus_carbon`` → CO₂ 0.55 + ``microbial_carbon``
0.45

``microbial_carbon`` is CENTURY's **active SOM** (the pool K5 names); ``humus_carbon``
is
its **slow SOM**. Every CO₂ leg draws O₂ at PQ=1 (the ``{CARBON:1, OXYGEN:2}``
composition fold), so all three flows are CARBON+OXYGEN and all three are throttled by
``f_O2`` on the **whole** flux — aerobic decomposition *is* the O₂-consuming process,
and
throttling only the CO₂ leg would let litter keep decaying into biomass under anoxia,
which is a different organism.

**``humus_carbon`` is a POOL, not a POPULATION** — the option-(B) ``microbial_n``
precedent: ``organ_stock``'s extinction pass would zero a POPULATION below threshold and
route the residual to the loss sink, orphaning carbon the rest of the system still
counts.

**Nitrogen follows the carbon partition in the same fractions** (``mineralization.py``),
so every organic pool inherits the C:N of the material that fell into it and mineral N
is
released in proportion to respired C — option (B)'s identity, extended, with **no new N
parameter**. ⚠ Recorded limitation, not a thing to fit: real humus runs C:N ~10 while
ours runs at the shed ratio (~90). Imposing a humus C:N would be the pool-identity
re-anchoring this project has refused three times.

**What this does NOT claim.** The chamber's soil is not thereby realistic: humus reaches
1.367 mol C against the chamber-scale census's 94×-short litter pile, and the sealed
chambers only tolerate the split at the sandiest end of eq. [6]'s domain (``T ≤ ~0.10``;
at 0.15 the perennial chamber cannot re-sow). That is a property of a 1 m² / 3.5 mol C
jar — the chamber-scale diagnosis's fourth independent witness — not of the science.

Pure stdlib only. Citation: Parton, W.J., Schimel, D.S., Cole, C.V. & Ojima, D.S.
(1987),
"Analysis of factors controlling soil organic matter levels in Great Plains grasslands",
Soil Sci. Soc. Am. J. 51:1173–1179 (first-hand; see ``params/humification.yaml``).
"""

from dataclasses import dataclass

from domains.biosphere.chamber import oxygen_limitation_factor
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class HumificationParams:
    """Loader-produced humification params: three CO₂ fractions + the slow-pool rate.

    Each fraction is the share of a decomposer flux lost as CO₂; the complement is
    stabilised into the receiving organic pool. All three are bound-checked to
    ``[0, 1]`` at the loader — outside it a leg would create or destroy carbon. A zero
    fraction is valid (it is the pre-2026-08-10 behaviour at that step), and a zero
    ``slow_decomposition_rate`` makes humus a permanent sink.
    """

    # [A] p. 1174 — nonlignin SURFACE structural litter (our litter is shed residue)
    litter_respired_fraction: float
    # [A] eq. [6] at T = 0 — Es, active SOM stabilised into slow SOM
    active_stabilization_co2_fraction: float
    # [A] p. 1174 — slow (and passive) SOM decomposition
    slow_respired_fraction: float
    # [A] p. 1176 — K6 = 0.0038 /week, expressed per day
    slow_decomposition_rate: float


def respired_and_stabilized(moved_c: float, co2_fraction: float) -> tuple[float, float]:
    """Split a decomposer carbon flux into ``(respired, stabilized)`` (mol C).

    The single kernel all three humified flows share, so they cannot drift on the
    partition arithmetic (the ``carbon_budget`` structural-agreement rhythm, and the
    option-(A) recomputation-drift lesson). ``respired + stabilized == moved_c`` by
    construction: the complement is computed by subtraction, never as
    ``moved_c · (1 − f)``, so the two legs sum back to the withdrawal exactly in
    floating point and the conservation gate cannot see a partition round-off.
    """
    respired = moved_c * co2_fraction
    return respired, moved_c - respired


@dataclass(frozen=True)
class HumusDecomposition:
    """CARBON+OXYGEN flow ``humus_carbon + o2_pool → carbon_pool + microbial_carbon``.

    CENTURY's slow-SOM decomposition: first-order in the standing humus pool at
    ``K6``, with ``slow_respired_fraction`` of the flow lost as CO₂ (drawing an equal
    amount of O₂ at PQ=1) and the remainder returned to active SOM. Self-limiting two
    ways, so ``rationed == 0`` is structural: in the substrate (∝ the humus pool's own
    start-of-step amount, ``k·dt < 1``) and in O₂ (the ``f_O2`` Monod factor → 0 as
    O₂ → 0). ``flux = daily · f_O2 · dt`` — dt-linear.

    ⚠ CENTURY also routes 3 % of the flow out of slow SOM into a PASSIVE pool
    (``K7``, 800–1600 yr). That pool is not built here; omitting it is conservative for
    a closed chamber, since a passive pool would lock **more** carbon away.
    """

    id: FlowId
    priority: int
    humus_carbon: StockId
    microbial_carbon: StockId
    co2_pool: StockId
    o2_pool: StockId
    params: HumificationParams
    o2_half_saturation: float
    # Total chamber air (mol) — the intensive basis for the ``f_O2`` mole fraction.
    air_mol: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        f_o2 = oxygen_limitation_factor(
            snapshot.stocks[self.o2_pool].amount,
            air_mol=self.air_mol,
            k_o2=self.o2_half_saturation,
        )
        decayed = (
            snapshot.stocks[self.humus_carbon].amount
            * self.params.slow_decomposition_rate
            * f_o2
            * dt
        )
        respired, stabilized = respired_and_stabilized(
            decayed, self.params.slow_respired_fraction
        )
        return FlowResult(
            legs=(
                Leg(self.humus_carbon, -decayed),
                Leg(self.microbial_carbon, stabilized),
                Leg(self.co2_pool, respired),
                Leg(self.o2_pool, -respired),
            )
        )
