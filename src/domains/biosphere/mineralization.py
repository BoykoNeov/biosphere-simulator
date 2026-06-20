"""Nitrogen return loop: senescence-N shedding + net mineralization (P2 Step 6).

The carbon decomposer loop is closed (Steps 4–5: senescence → litter_carbon →
microbial → CO₂ → photosynthesis). This module closes the **nitrogen** loop — the N
analogue — that Phase 1 left open: there, ``soil_n`` was drained into ``plant_n`` by
uptake and refilled by an *external* ``n_source`` (fertilization), with ``plant_n`` only
ever **growing** (nothing withdrew it). Step 6 returns plant nitrogen to the soil
internally, so the cycle ``soil_n → plant_n → litter_n → soil_n`` closes with no
external supply:

* **N-senescence** — ``plant_n -> litter_n`` (Σ legs = 0). When the plant senesces it
  sheds nitrogen into a finite ``litter_n`` POOL (the N analogue of carbon senescence
  feeding ``litter_carbon``). First-order in the standing plant N:
  ``shed = n_senescence_rate · plant_n`` (kg N day⁻¹), self-limiting → 0 as plant_n → 0
  (the senescence / decomposition positivity pattern: a clamped POOL withdrawal ∝ its
  own start-of-step amount, so ``n_senescence_rate·dt < 1`` keeps the Euler backstop
  unfired). This **drains** ``plant_n`` (Phase 1 left it monotone-*growing*; it now
  declines toward the uptake/shedding equilibrium) — the consumption side the open N
  loop lacked.

* **Mineralization** — ``litter_n -> soil_n`` (Σ legs = 0). Decomposing litter releases
  mineral N back to the soil pool. First-order donor-controlled net mineralization
  (Stanford & Smith 1972): ``min = mineralization_rate · litter_n`` (kg N day⁻¹),
  self-limiting → 0 as litter_n → 0 (the same structural positivity). This is the
  **DIRECT net-mineralization** flux ``litter_n → soil_n``.

Both are **single-currency NITROGEN** flows (``litter_n``/``soil_n``/``plant_n`` are all
``{NITROGEN: 1}``), so the every-step conservation gate folds them exactly like Phase 1
— no core change. Sealed-chamber only (``litter_n`` exists only when sealed); appended
to the registry like ``Decomposition`` / ``MicrobialRespiration``.

**Scope refinement vs the plan wording — DIRECT vs microbe-mediated N (advisor-reviewed,
like Steps 4/5).** The plan says "litter/**microbial** N → soil_n"; this ships the
direct first-order ``litter_n → soil_n`` net mineralization and **defers** the
microbe-mediated path (N immobilization ``litter_n → microbial_n`` during decomposition,
then ``microbial_n → soil_n`` during microbial turnover). First-order net mineralization
is the standard minimal soil-N treatment (Stanford & Smith 1972) and matches how Step 4
chose first-order donor decay over microbe-explicit Michaelis kinetics; microbe-mediated
immobilization is the C:N-ratio-driven advanced path, a documented refinement seam.

**Mechanism, not feedback — the deliverable, framed honestly (the f_O2 mirror).**
With the chamber sized for potential production (PP, non-limiting N), ``plant_n`` stays
~1000× above the critical-N concentration all season, so ``f_N ≡ 1`` and this loop is a
**parallel cycle with zero effect on the carbon / plant trajectory** — the sealed carbon
run is bit-identical to before Step 6, and every prior sealed test passes unchanged. The
Step-6 deliverable is therefore **"nitrogen mass cycles internally and is conserved,"**
NOT "emergent N feedback." The N-limited regime — where ``plant_n`` is drawn low enough
that ``f_N < 1`` throttles photosynthesis — is **deferred to Step 7**'s sized multi-year
run (exactly as Step 2 shipped a "draw-down decline, not oscillation" and Steps 3/5
deferred ``f_O2``). The decoupling is *verified* (``test_mineralization`` recomputes f_N
each step and asserts ``== 1.0``), not merely asserted.

**Why first-order in plant_n — the litter C:N is emergent (a deferred refinement).**
N-shedding is a plain first-order relative rate on the whole-plant ``plant_n`` POOL
(the carbon-senescence pattern). Tying N-shedding to the carbon-senescence flux at a
fixed tissue N:C would *control* the litter C:N ratio; here the litter C:N
(``litter_carbon / litter_n``) is instead **emergent** from two independent first-order
rates. The simpler decoupled pick is the JIT-minimal Step-6 choice; N:C-coupled
shedding and N resorption before abscission (this model sheds all standing plant N) are
documented refinement seams.

Pure stdlib only. Citations: Stanford, G. & Smith, S.J. (1972), "Nitrogen mineralization
potentials of soils", Soil Science Society of America Journal 36(3):465–472 (first-order
net N mineralization); Penning de Vries, F.W.T. et al. (1989), "Simulation
of Ecophysiological Processes of Growth in Several Annual Crops", Simulation Monographs,
PUDOC, Wageningen (the relative-death-rate senescence form the N-shedding mirrors).
Provisional ``TODO(cite)`` rate values pending the Phase-2 validation gate (see
``params/mineralization.yaml``), clean-room.
"""

from dataclasses import dataclass

from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class MineralizationParams:
    """Loader-produced nitrogen-return-loop parameters: the two first-order rates.

    Provisional literature-typical placeholders pending the Phase-2 validation gate (see
    ``params/mineralization.yaml``). Zero rates are valid (no shedding / no
    mineralization); negative is rejected at the loader.
    """

    # first-order plant-N shedding rate, plant_n → litter_n (kg N / kg N / day)
    n_senescence_rate: float
    # first-order net mineralization rate, litter_n → soil_n (kg N / kg N / day)
    mineralization_rate: float


def nitrogen_senescence_flux(plant_n: float, *, n_senescence_rate: float) -> float:
    """Daily plant-N shedding ``n_senescence_rate · plant_n`` (kg N day⁻¹).

    First-order in the standing whole-plant N, so it → 0 as plant_n → 0 (positivity is
    structural — the carbon-senescence self-limiting pattern). The shed nitrogen enters
    the ``litter_n`` POOL (the :class:`NitrogenSenescence` flow).
    """
    return n_senescence_rate * plant_n


def mineralization_flux(litter_n: float, *, mineralization_rate: float) -> float:
    """Daily net mineralization ``mineralization_rate · litter_n`` (kg N day⁻¹).

    First-order donor-controlled (Stanford & Smith 1972): proportional to the standing
    litter N, so it → 0 as litter_n → 0 (positivity is structural — the decomposition
    self-limiting pattern). The mineralized nitrogen returns to the ``soil_n`` POOL (the
    :class:`Mineralization` flow).
    """
    return mineralization_rate * litter_n


@dataclass(frozen=True)
class NitrogenSenescence:
    """NITROGEN flow ``plant_n -> litter_n`` (balanced, P2 Step 6).

    Sheds ``nitrogen_senescence_flux(plant_n, n_senescence_rate)·dt`` of nitrogen from
    the whole-plant ``plant_n`` POOL into the ``litter_n`` POOL each step — the N
    counterpart of carbon senescence (``allocation.Senescence`` feeding
    ``litter_carbon``), and the consumption side the Phase-1 open N loop lacked
    (``plant_n`` is now drained — no longer monotone-growing). Single-currency NITROGEN
    (both pools are
    ``{NITROGEN: 1}``), so the gate folds it identically to Phase 1. Self-limiting
    (∝ the plant-N pool's amount), so ``rationed == 0`` is structural
    (``n_senescence_rate·dt < 1``). Sealed-only; ``flux = daily·dt`` (dt-linear).
    """

    id: FlowId
    priority: int
    plant_n: StockId
    litter_n: StockId
    params: MineralizationParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        shed = (
            nitrogen_senescence_flux(
                snapshot.stocks[self.plant_n].amount,
                n_senescence_rate=self.params.n_senescence_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.plant_n, -shed),
                Leg(self.litter_n, shed),
            )
        )


@dataclass(frozen=True)
class Mineralization:
    """NITROGEN flow ``litter_n -> soil_n`` (balanced, P2 Step 6).

    Releases ``mineralization_flux(litter_n, mineralization_rate)·dt`` of nitrogen from
    the ``litter_n`` POOL back to the ``soil_n`` POOL each step — closing the nitrogen
    cycle ``soil_n → plant_n → litter_n → soil_n`` that Phase 1 fed externally from an
    ``n_source`` BOUNDARY. Direct net first-order mineralization (Stanford & Smith 1972;
    the microbe-mediated immobilization path is deferred — see the module docstring).
    Single-currency NITROGEN (both pools are ``{NITROGEN: 1}``). Self-limiting (∝ the
    litter-N pool's amount), so ``rationed == 0`` is structural
    (``mineralization_rate·dt < 1``). Sealed-chamber only. ``flux = daily·dt`` —
    dt-linear.
    """

    id: FlowId
    priority: int
    litter_n: StockId
    soil_n: StockId
    params: MineralizationParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        mineralized = (
            mineralization_flux(
                snapshot.stocks[self.litter_n].amount,
                mineralization_rate=self.params.mineralization_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.litter_n, -mineralized),
                Leg(self.soil_n, mineralized),
            )
        )
