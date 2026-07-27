"""Nitrogen return loop: senescence-N shedding + net mineralization (P2 Step 6).

The carbon decomposer loop is closed (Steps 4–5: senescence → litter_carbon →
microbial → CO₂ → photosynthesis). This module closes the **nitrogen** loop — the N
analogue — that Phase 1 left open: there, ``soil_n`` was drained into ``plant_n`` by
uptake and refilled by an *external* ``n_source`` (fertilization), with ``plant_n`` only
ever **growing** (nothing withdrew it). Step 6 returns plant nitrogen to the soil
internally, so the cycle ``soil_n → plant_n → litter_n → soil_n`` closes with no
external supply:

* **N-senescence** — ``plant_n -> litter_n`` (Σ legs = 0). When the plant senesces it
  sheds nitrogen into a finite ``litter_n`` POOL, **coupled to the senescing carbon**:
  ``shed = min(plant_n/biomass_c, n_residual_per_mol_c) · shed_C`` (kg N day⁻¹), where
  ``shed_C`` is the very flux ``allocation.Senescence`` sends to ``litter_carbon``.
  Self-limiting → 0 as plant_n → 0 (the senescence / decomposition positivity pattern: a
  clamped POOL withdrawal ∝ its own start-of-step amount, so ``max(rdr)·dt < 1`` keeps
  the Euler backstop unfired). This **drains** ``plant_n`` (Phase 1 left it
  monotone-*growing*) — the consumption side the open N loop lacked.

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

**Mechanism, not feedback — the deliverable, framed honestly (the f_O2 mirror).** With
the chamber sized for potential production (PP, non-limiting N), ``f_N ≡ 1`` and this
loop is a **parallel cycle with zero effect on the carbon / plant trajectory** — the
sealed carbon run is bit-identical to before Step 6, and every prior sealed test passes
unchanged.

⚠ **The MARGIN behind that changed by ~2.5 orders of magnitude when uptake became
demand-driven, even though the conclusion did not.** Step 6 could say ``plant_n`` sits
"~1000× above the critical-N concentration" because capacity-uptake pinned it at
``max_uptake_capacity / n_senescence_rate`` — a huge equilibrium set by two unrelated
rate constants. Demand-deficit uptake fills to the *target* instead, so the plant now
sits at Greenwood's curve: **3.8× critical on the plateau**, and as little as **~1.07×
at ``open_season``'s peak crop mass** (12.633 t/ha, where the curve gives 1.60 % against
a 1.50 % critical). ``f_N ≡ 1`` is still measured in all seven frozen scenarios, but it
is now a **12 % margin, not a 1000× one** — which is why the crossing point is pinned as
a test rather than mentioned in prose. The Step-6 deliverable is therefore **"nitrogen
mass cycles internally and is conserved,"** NOT "emergent N feedback." The N-limited
regime — where ``plant_n`` is drawn low enough that ``f_N < 1`` throttles photosynthesis
— is **deferred to Step 7**'s sized multi-year run (exactly as Step 2 shipped a
"draw-down decline, not oscillation" and Steps 3/5 deferred ``f_O2``). The decoupling is
*verified* (``test_mineralization`` recomputes f_N
each step and asserts ``== 1.0``), not merely asserted.

**N-shedding is N:C-COUPLED (post-roadmap, the N-cycle form gap — this replaces the
Step-6 first-order rate).** Step 6 shipped a plain first-order relative rate on the
whole-plant ``plant_n`` POOL and recorded the consequence honestly: litter C:N was
**emergent** from two *independent* first-order rates, i.e. unconstrained. Measured,
that cost 1–4 orders of magnitude — the frozen form gave a litter C:N of **0.004**
in-run (≈ 1 C : 246 N) against wheat straw's ~80. Nothing tied the nitrogen leaving the
plant to the carbon it was part of, so the ratio was free.

Two things are fixed by the same change, and the second is why it was worth an unfreeze:

* **litter C:N is now a modelled quantity**, set by the tissue concentration the N
  leaves at, not by the accident of two rate constants;
* **the uncitable parameter is gone.** ``n_senescence_rate`` was a bare 1/day N-shedding
  rate, and retrieval for it was declared *exhausted because no source has that shape* —
  the highest clean-room risk in the project. The coupled form's parameter is a **tissue
  N concentration**, which is exactly what the literature does publish, and it is one
  this file's sibling already cites (``n_residual`` ← Van Hecke et al. 2020, measured as
  N remaining in mature straw *after remobilization*). **Changing the form changed which
  citation shape was needed** — that, not calibration, is what got the param off TODO.

Still deferred: microbe-mediated immobilization (``litter_n → microbial_n → soil_n``,
below) and DS-dependent shedding rates.

Pure stdlib only. Citations: Stanford, G. & Smith, S.J. (1972), "Nitrogen mineralization
potentials of soils", Soil Science Society of America Journal 36(3):465–472 (first-order
net N mineralization); Penning de Vries, F.W.T. et al. (1989), "Simulation
of Ecophysiological Processes of Growth in Several Annual Crops", Simulation Monographs,
PUDOC, Wageningen (the relative-death-rate senescence form the N-shedding mirrors).
Provisional ``TODO(cite)`` rate values pending the Phase-2 validation gate (see
``params/mineralization.yaml``), clean-room.
"""

from dataclasses import dataclass

from domains.biosphere.allocation import SenescenceParams, senescence_flux
from domains.biosphere.nitrogen import NitrogenParams
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

    # first-order net mineralization rate, litter_n → soil_n (kg N / kg N / day)
    mineralization_rate: float


def nitrogen_shedding_flux(
    shed_carbon: float,
    plant_n: float,
    biomass_c: float,
    *,
    n_residual_per_mol_c: float,
) -> float:
    """Daily plant-N shedding, **coupled to the senescing carbon** (kg N day⁻¹).

    ``shed_N = min(tissue_conc, residual_conc) · shed_C``, where ``shed_C`` is the same
    per-organ senescence flux ``allocation.Senescence`` sends to litter and
    ``tissue_conc = plant_n / biomass_c`` (kg N / mol C).

    **Why the min, and why it needs no new parameter.** Senescing tissue does not shed
    its nitrogen at the live-tissue concentration — the plant remobilizes N out of dying
    organs first, and what is left behind is exactly the quantity ``n_residual`` already
    measures: Van Hecke et al. (2020) report *"the concentration of residual N in the
    mature straw, i.e. N left after N remobilization to the grain"*. So the shed
    concentration **is** the cited residual concentration, and the retained difference
    stays in the whole-plant ``plant_n`` POOL (which is where remobilized N goes in a
    one-pool model). A separate "resorption efficiency" param would be a *second*,
    uncited way of saying the same thing; this way the literature supplies the number.
    The ``min`` covers the case where the tissue is already at or below residual — then
    there is nothing to remobilize and shedding runs at the actual concentration.

    Positivity is structural, as for the rate form it replaces: the shed nitrogen is at
    most ``plant_n · (shed_C / biomass_c)``, and ``shed_C ≤ max(rdr) · biomass_c``, so
    ``max(rdr)·dt < 1`` bounds the withdrawal below the pool — the same precondition the
    carbon senescence flow already carries.

    Returns 0.0 for a non-positive plant, biomass or carbon flux (never a
    divide-by-zero, never a negative leg).
    """
    if shed_carbon <= 0.0 or plant_n <= 0.0 or biomass_c <= 0.0:
        return 0.0
    tissue_conc = plant_n / biomass_c
    return min(tissue_conc, n_residual_per_mol_c) * shed_carbon


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

    Sheds ``nitrogen_shedding_flux(...)·dt`` of nitrogen from the whole-plant
    ``plant_n`` POOL into the ``litter_n`` POOL each step — no longer merely the
    *counterpart* of carbon senescence but **driven by it**: this flow recomputes the
    identical per-organ ``senescence_flux`` that ``allocation.Senescence`` sends to
    ``litter_carbon``, from the same ``SenescenceParams`` object, so the two legs of one
    physical event cannot drift apart. Single-currency NITROGEN (both pools are
    ``{NITROGEN: 1}``), so the gate folds it identically to Phase 1. Self-limiting, so
    ``rationed == 0`` stays structural (``max(rdr)·dt < 1``). Sealed-only; ``flux =
    daily·dt`` (dt-linear).

    **The carbon rates are recomputed here, not shared through state.** A flow may only
    read the step-entry snapshot, so there is no channel by which ``Senescence`` could
    hand
    this flow its computed flux; recomputation from the same params on the same snapshot
    is the only form that keeps both flows pure. The hazard that creates — the two
    drifting if someone changes one organ's rate handling and not the other — is pinned
    by a test that asserts the shed carbon here equals ``Senescence``'s litter leg, so
    it fails loudly rather than silently decoupling the C:N of litter.
    """

    id: FlowId
    priority: int
    plant_n: StockId
    litter_n: StockId
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    sen_params: SenescenceParams
    nitro_params: NitrogenParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        stocks = snapshot.stocks
        leaf = stocks[self.leaf_c].amount
        stem = stocks[self.stem_c].amount
        root = stocks[self.root_c].amount
        # The identical per-organ flux allocation.Senescence sends to litter_carbon.
        shed_carbon = (
            senescence_flux(leaf, relative_death_rate=self.sen_params.rdr_leaf)
            + senescence_flux(stem, relative_death_rate=self.sen_params.rdr_stem)
            + senescence_flux(root, relative_death_rate=self.sen_params.rdr_root)
        )
        shed = (
            nitrogen_shedding_flux(
                shed_carbon,
                stocks[self.plant_n].amount,
                leaf + stem + root,
                n_residual_per_mol_c=self.nitro_params.n_residual_per_mol_c,
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
