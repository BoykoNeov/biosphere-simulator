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

**Why no ``f_O2`` (O₂ self-limitation) yet — deferred to Step 7, a magnitude bet.**
P2.2 flags an O₂ Michaelis factor (``f_O2 = O2 / (K_O2 + O2)`` → 0 as O₂ → 0) so
respiration keeps ``rationed == 0`` on a *depleting* O₂ pool. Microbial respiration's O₂
draw is ``m_resp · microbial_C`` — **not** self-limiting on the O₂ pool — so a small
enough O₂ fill would ration. But at the realistic chamber fill (~210 mol O₂) the
standing microbial biomass is O(0.01) mol C and the per-step O₂ draw is O(1e-4) mol, so
over a single sealed season O₂ never approaches its floor (measured min(O₂) ≈
0.99997·fill — O₂ falls only ~0.0065 mol of ~210 over the 305-day season, ~4 orders from
rationing), exactly as Step 3 found for plant respiration. So ``f_O2`` would be ≈ 1
throughout and untestable-in-anger here; it is **deferred to Step 7**, where the
multi-year sealed run is *sized to deplete* O₂ (the explicit O₂-depletion validation
target) and ``f_O2`` is applied to **both** microbial respiration and the plant
maintenance shortfall. The deferral is guarded by ``test_gas_exchange``'s O₂ ≫ 0 check
(now also covering the microbial O₂ draw): if a future change pushes O₂ toward zero,
that test breaks and flags ``f_O2`` has become load-bearing. (This mirrors the Step-3
scope decision, advisor-reviewed; the plan's "designed at step 5" wording predates the
Step-3 magnitude discovery.)

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
    self-limits (∝ the microbial pool's amount), so ``rationed == 0`` is structural
    (``m_resp·dt < 1``; the O₂ draw is far from the O₂ pool's floor — see the module's
    ``f_O2`` deferral). ``flux = daily·dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    microbial_carbon: StockId
    co2_pool: StockId
    o2_pool: StockId
    params: MicrobialRespirationParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        respired = (
            microbial_respiration_flux(
                snapshot.stocks[self.microbial_carbon].amount,
                microbial_respiration_rate=self.params.microbial_respiration_rate,
            )
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
