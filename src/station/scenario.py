"""The Station scenario — the coupled Power → Thermal heat-closure run (P6.1).

The station analogue of ``domains.power.scenario`` / ``domains.thermal.scenario``, one
level up: a station scenario is **not** a new set of coefficients — it *references* the
already-validated sibling scenarios and adds only the cross-domain wiring choices. For
Step 1 the only sibling with tunable run data is Power (the diurnal microgrid); Thermal
contributes its ``radiator.yaml`` params (loaded, not scenario data) and no run data of
its own — the radiator is the restoring force, so it needs no forcing schedule.

**What Step 1 couples.** The standalone Power domain dumped every degraded joule into a
terminal ``boundary.waste_heat`` sink (a deliberate seam). The standalone Thermal domain
received a *forced* ``heat_load`` stand-in for "Power's dissipation" into its node. Step
1 removes both stand-ins and lets them meet at one shared stock: Power's dissipation
legs now deposit into ``thermal.node`` directly, and the Stefan-Boltzmann radiator
rejects that **real** load to deep space. The seam is pure sink re-wiring
(``system.build_station`` passes ``thermal.node``'s id where the Power flows took
``waste_heat``) — zero domain change, zero core change (finding #1).

**The node's initial heat is DERIVED from Power's actual output, not hand-set.** Because
Power runs daily-balanced (``load_fraction = 1`` ⇒ SOC returns to ``battery0`` every
day), in steady state *all* supplied solar energy ends up as heat in the node
(charge-conversion loss ``(1−η_c)·S`` + the 100 %-dissipative load ``η_c·S`` = ``S`` per
day). The mean dissipated power sets an emergent equilibrium node temperature;
``build_station`` starts the node at the corresponding ``Q_eq``
(``system.equilibrium_node_heat``), so the run begins near the attractor the *actual*
dissipation implies. That the equilibrium is set by dissipation independent of the
initial condition is proved **non-circularly** by the two-start convergence test (two
``node0`` values under identical Power forcing converge to one band — the radiator alone
governs the difference), not by starting there.

**Time unit / step come from the Power scenario** (``dt = 3600 s``, ``steps_per_day =
24``) — Thermal standalone used the same ``dt``, so there is no rate mismatch to
reconcile (the increment-form flows are dt-linear anyway, #multi-rate-safe). The horizon
is a day count, like Power's ``BOUNDED_SOC_DAYS``.

Pure stdlib only (a frozen dataclass wrapping the Power scenario).
"""

from dataclasses import dataclass

from domains.power.scenario import BOUNDED_SOC_SCENARIO, PowerScenario


@dataclass(frozen=True)
class StationScenario:
    """Station run data: which sibling scenarios the coupled station is assembled from.

    Thin by design — it references the already-validated ``PowerScenario`` rather than
    re-declaring its fields, so the coupled run cannot drift from the standalone one it
    reuses (the battery trajectory stays bit-identical; see ``test_station_run.py``).
    The radiator params are loaded separately (``radiator.yaml``), like the charge
    param, and the node's initial heat is derived in ``system.build_station`` from
    Power's actual dissipation — neither is scenario data here. Later steps (crew /
    ECLSS / biosphere) add their own scenario references to this struct as their seams
    are built.
    """

    # The Power sub-scenario driving the station: the daily-balanced microgrid whose
    # dissipation the Thermal node now receives. Reused verbatim so the coupled battery
    # SOC matches standalone Power to the bit (coupling is pure sink re-wiring).
    power: PowerScenario = BOUNDED_SOC_SCENARIO


# Module-level default (immutable, frozen dataclass) — the canonical Step-1 station.
DEFAULT_STATION_SCENARIO: StationScenario = StationScenario()

# The Step-1 validation scenario: Power's daily-balanced microgrid feeding the Thermal
# node, the node started at the equilibrium its mean dissipation implies. ENERGY
# conserved every step over the combined ledger (solar_source + battery + node + space),
# ``rationed == 0``, ``events == ()``, the battery bit-identical to standalone Power,
# the node bounded near the predicted equilibrium, ``boundary.space`` monotonic
# (carrying the real load). The defaults already encode the sizing; this alias names the
# canonical run shared by the validation test and the golden so they cannot drift.
HEAT_CLOSURE_SCENARIO: StationScenario = DEFAULT_STATION_SCENARIO

# The golden / bounded-node horizon (days). Started at Q_eq the node stays within a
# tight band (~0.1 K over a week), so a short horizon suffices to pin a bounded
# near-equilibrium endpoint (the Power ``BOUNDED_SOC_DAYS`` length; cheap +
# deterministic).
HEAT_CLOSURE_DAYS: int = 7

# The two-start convergence horizon (days). The relaxation time τ ≈ 14.6 days (long, set
# by the large radiator heat capacity), so this is ~3 τ — enough for two bracketing
# ``node0`` starts to contract to a small fraction of their initial gap (the emergent
# attractor, visible; 7 days would show only ~0.48 τ ≈ 62 % of the gap remaining). Its
# own horizon, longer than the golden's — the attractor claim needs the length, the
# golden does not.
CONTRACTION_DAYS: int = 45
