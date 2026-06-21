"""Microbial respiration: the decomposer gas flux (Phase-2 Step 5; CARBON+OXYGEN).

Step 4 built the carbon-only litter→microbial transfer, leaving microbial biomass a
pure intermediate that only *grew* (decay deposits; nothing withdrew it). Step 5 closes
the carbon loop: aerobic microbial respiration burns microbial biomass back to CO₂,
consuming O₂ — ``microbial_C + O₂ → CO₂`` — the decomposer's mirror of plant
maintenance respiration's biomass-burned shortfall
(``carbon_budget.MaintenanceRespiration``). This is the O₂-drawing decomposer flux (the
Biosphere-2 soil-respiration O₂-sink mechanism) and the gate-forced O₂ coupling the
Step-4 doc deferred to here (CO₂ into the ``{CARBON:1, OXYGEN:2}`` pool drags 2 oxygens
pure-carbon microbes cannot supply — they come from the O₂ pool, exactly like plant
maintenance).

* **Microbial respiration** — ``microbial_carbon + o2_pool → carbon_pool`` (the CO₂
  POOL). First-order in standing microbial biomass: ``resp = m_resp · microbial_C``
  (mol C day⁻¹), self-limiting → 0 as microbial → 0 (the maintenance / senescence /
  decomposition positivity pattern: a clamped POPULATION withdrawal ∝ the stock's own
  start-of-step amount, so ``m_resp·dt < 1`` keeps the Euler backstop unfired).
  Multi-quantity (CARBON+OXYGEN) at the photosynthetic/respiratory quotient PQ=1: each
  mol C respired returns 1 mol CO₂ to the pool (carrying 2 oxygens) and consumes 1 mol
  O₂ (those 2 oxygens) — CARBON ``−1 + 1 = 0``, OXYGEN ``+2 − 2 = 0``, balanced in one
  flow by the P2.1 composition fold. The three legs ``(microbial −b, co2_pool +b,
  o2_pool −b)`` mirror the plant maintenance *shortfall*, but **simpler**: microbial
  biomass and the CO₂/O₂ pools exist only in the sealed chamber, so this flow is
  sealed-only and **always three legs** — no open-field branch, and no ``source ==
  sink`` netting (``microbial_carbon ≠`` the CO₂ pool, unlike the *covered*
  plant-maintenance CO₂→CO₂ round trip).

**``f_O2`` (O₂ self-limitation) — deferred from here, built at Step 7.** P2.2 flags an
O₂ Michaelis factor (``f_O2 = x_O2 / (K_O2 + x_O2)`` → 0 as O₂ → 0) so respiration keeps
``rationed == 0`` on a *depleting* O₂ pool. Microbial respiration's bare O₂ draw
``m_resp · microbial_C`` is **not** self-limiting on the O₂ pool, so a small enough fill
*would* ration. Steps 3/5 deferred the factor — at the ~210 mol PP fill the per-step O₂
draw is O(1e-4) mol, so O₂ never approached its floor (~4 orders from rationing) and
``f_O2 ≈ 1`` would have been untestable. **Step 7 builds it** (``chamber.oxygen_
limitation_factor``, applied here and to the plant maintenance shortfall): the flux is
scaled by ``f_O2`` of the live O₂ mole fraction, so on the canonical multi-year run
(sized to deplete O₂) the draw shuts off smoothly before the pool is over-run — the
respiratory mirror of FvCB's Ci-shutoff. ``K_O2`` is low/sharp (``f_O2 ≈ 1`` until
near-anoxia), so the prior PP-sealed behaviour is preserved and ``f_O2`` is load-bearing
only as O₂ approaches its floor.

**Deferred seams (unchanged from Step 4).** Microbial death / turnover as a
``microbial_C → litter_C`` *recycling* (vs this respiration → CO₂); microbe-explicit
Michaelis substrate kinetics (``Vmax · microbial_C · litter/(K_m + litter)``); and the
``f_O2`` O₂ self-limitation above. First-order donor control is the right minimal
Step-5 pick (the Step-4 decomposition rhythm).

Pure stdlib only. Citation: the first-order microbial-respiration / biomass-turnover
form is the standard soil-carbon-model treatment (e.g. the microbial-pool turnover of
CENTURY / RothC); the rate value is a provisional ``TODO(cite)`` placeholder pending the
Phase-2 validation gate (see ``params/microbial_respiration.yaml``), clean-room.
"""

from dataclasses import dataclass

from domains.biosphere.chamber import oxygen_limitation_factor
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class MicrobialRespirationParams:
    """Loader-produced microbial-respiration parameter: the first-order rate.

    Provisional literature-typical placeholder pending the Phase-2 validation gate (see
    ``params/microbial_respiration.yaml``). A zero rate is valid (no microbial
    respiration — microbial biomass would only grow, the Step-4 behaviour); negative is
    rejected at the loader.
    """

    # m_resp, first-order microbial respiration (mol C / mol C / day)
    microbial_respiration_rate: float
    # O₂ half-saturation (mole fraction) for the ``f_O2`` self-limit (Phase-2 Step 7;
    # ``chamber.oxygen_limitation_factor``). Low/sharp ⇒ ``f_O2 ≈ 1`` until near-anoxia,
    # so the O₂ draw only self-throttles on a depleting pool (the central Step-7 guard
    # that keeps ``rationed == 0`` as O₂ depletes). 0 disables the limit.
    o2_half_saturation: float


def microbial_respiration_flux(
    microbial_c: float, *, microbial_respiration_rate: float
) -> float:
    """Daily microbial respiration ``m_resp · microbial_c`` (mol C day⁻¹).

    First-order in standing microbial biomass, so it → 0 as microbial → 0 (positivity is
    structural — the maintenance / decomposition self-limiting pattern). The respired
    carbon returns to the CO₂ pool and consumes an equal amount of O₂ (PQ=1; the
    :class:`MicrobialRespiration` flow).
    """
    return microbial_respiration_rate * microbial_c


@dataclass(frozen=True)
class MicrobialRespiration:
    """CARBON+OXYGEN flow ``microbial_carbon + o2_pool → carbon_pool`` (P2 Step 5).

    Burns ``microbial_respiration_flux(microbial_carbon, m_resp)·dt`` of microbial
    biomass back to CO₂ each step — the decomposer's contribution to the chamber gas
    exchange and the close of the carbon loop (litter → microbial → CO₂ →
    photosynthesis). At PQ=1 the three legs balance CARBON **and** OXYGEN in one flow
    (the P2.1 composition fold): the CO₂ returned to the pool carries 2 oxygens supplied
    by the consumed O₂, so OXYGEN nets to zero. Sealed-chamber only (the pools/microbes
    exist only when sealed) — always three legs, no ``source == sink`` netting. The draw
    self-limits two ways, so ``rationed == 0`` is structural: in the substrate
    (∝ the microbial pool's amount, ``m_resp·dt < 1``) **and** in O₂ (the ``f_O2``
    Monod factor → 0 as O₂ → 0, Step 7), so neither the microbial pool nor the O₂ pool
    is ever over-run. ``flux = daily · f_O2 · dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    microbial_carbon: StockId
    co2_pool: StockId
    o2_pool: StockId
    params: MicrobialRespirationParams
    # Total chamber air (mol) — the intensive basis for the ``f_O2`` O₂ mole fraction
    # (Step 7). Chamber/scenario data (P4), passed from ``scenario.chamber_air_mol``.
    air_mol: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        # f_O2 self-limit (Step 7): the O₂ draw shuts off smoothly as O₂ → 0, so it
        # never over-runs the pool on a depleting chamber (rationed == 0 from kinetics —
        # the Ci-shutoff mirror). At the PP fill f_O2 ≈ 1 (low/sharp K_O2), so the
        # Steps-3/5 behaviour is preserved until O₂ actually depletes.
        f_o2 = oxygen_limitation_factor(
            snapshot.stocks[self.o2_pool].amount,
            air_mol=self.air_mol,
            k_o2=self.params.o2_half_saturation,
        )
        respired = (
            microbial_respiration_flux(
                snapshot.stocks[self.microbial_carbon].amount,
                microbial_respiration_rate=self.params.microbial_respiration_rate,
            )
            * f_o2
            * dt
        )
        # CO₂ returned to the pool = carbon respired; O₂ consumed = carbon respired
        # (PQ=1). All three legs use the same ``respired`` magnitude, so CARBON (−b + b)
        # and OXYGEN (the pool's +2b vs the O₂ pool's −2b, via the composition fold)
        # balance exactly.
        return FlowResult(
            legs=(
                Leg(self.microbial_carbon, -respired),
                Leg(self.co2_pool, respired),
                Leg(self.o2_pool, -respired),
            )
        )
