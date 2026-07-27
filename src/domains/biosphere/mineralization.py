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

* **LitterNitrogenTransfer** — ``litter_n -> microbial_n`` (Σ legs = 0), and
* **MicrobialNitrogenRelease** — ``microbial_n -> soil_n`` (Σ legs = 0).

  Together these are the **microbe-mediated** return leg, and they replaced a direct
  first-order ``litter_n → soil_n`` net mineralization — see "The return leg rides the
  carbon" below.

Both are **single-currency NITROGEN** flows (``litter_n``/``soil_n``/``plant_n`` are all
``{NITROGEN: 1}``), so the every-step conservation gate folds them exactly like Phase 1
— no core change. Sealed-chamber only (``litter_n`` exists only when sealed); appended
to the registry like ``Decomposition`` / ``MicrobialRespiration``.

**The return leg rides the carbon (post-roadmap, the N-cycle form gap, option (B)).**
Step 6 shipped the *direct* first-order ``litter_n → soil_n`` flux at a free
``mineralization_rate``, deferring the microbe-mediated path as a refinement seam. That
seam is now built, and the reason is not realism-for-its-own-sake — it is that **the
free rate is gone**:

    litter_n    -> microbial_n     moved = decomposed_C · (litter_n / litter_C)
    microbial_n -> soil_n          moved = respired_C   · (microbial_n / microbial_C)

Each leg carries the nitrogen that belongs to the carbon its sibling flow **already**
moved this step — ``Decomposition``'s ``decomposed_C`` and ``MicrobialRespiration``'s
``respired_C`` (``f_O2`` included). Since ``decomposed_C / litter_C ≡
decomposition_rate`` identically, the first leg *is* ``decomposition_rate · litter_n``:
the form **replaces an uncited free rate with the carbon rate stoichiometry forces it to
equal**. ``mineralization_rate`` is therefore **RETIRED** — no parameter takes its
place, and ``params/mineralization.yaml`` is gone with it. That is the second
weakly-supported parameter in this file discharged by a **form** change rather than a
citation hunt (after ``n_senescence_rate``, below).

⚠ **Written recomputed-stoichiometric, never collapsed to the rate.** The identity above
holds only while ``Decomposition`` stays first-order; a collapsed
``decomposition_rate · litter_n`` would read identically today and silently outlive that
premise. The respiration leg makes the point plainly — it carries ``f_O2``, so there is
no bare rate to collapse *to*.

⚠ **This is N transit, NOT immobilization, and the distinction is measured.** The
canonical immobilization treatment imposes a homeostatic microbial C:N (~8) and draws
the shortfall from mineral N. That requires our ``microbial_carbon`` to *mean* what
CENTURY/RothC's microbial pool means, and it does not: carbon-use efficiency is **1.0**
here (``Decomposition`` moves 100 % of decayed litter C into microbes and respiration is
a separate draw — the deliberate Step-4/5 split), so the pool holds carbon a real model
would already have respired. Measured, it peaks *comparable to* the litter pool rather
than at the few percent standing microbial biomass actually is, and imposing C:N = 8 on
it would demand 90–152× the litter N present. Re-labelling the pool to make that
constant fit is the same move this project refused for ``decomposition.yaml``'s DPM/RPM
labile-fraction re-read and for the soil-N₀-vs-litter-N re-anchoring below: *redefining
what a pool MEANS so a literature constant fits is a semantic model change wearing a
provenance hat.* **So immobilization remains a deferred seam — now with a measured
obstacle instead of a deferral,** which is the more useful record.

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

Still deferred: **C:N-driven immobilization** (a mineral-N draw when residue is
N-poor — see the measured obstacle above, which is why this is not merely "not built
yet") and DS-dependent shedding rates (option (C); the DS-dependence gap moved to the
*carbon* side when shedding became carbon-driven, where ``senescence.yaml``'s flat
``rdr_*`` now own it).

Pure stdlib only. Citations: Penning de Vries, F.W.T. et al. (1989), "Simulation of
Ecophysiological Processes of Growth in Several Annual Crops", Simulation Monographs,
PUDOC, Wageningen (the relative-death-rate senescence form the N-shedding mirrors); Van
Hecke, M.M. et al. (2020) for ``n_residual`` (see ``params/nitrogen.yaml``).
⚠ **Stanford & Smith (1972) is no longer cited here, and its removal is a finding,
not tidying** — it supported the retired ``mineralization_rate``, and the
citation-scope work had
established first-hand that its measured pool (soil organic N₀) was **not our pool**
(fresh dead-plant N). The value it was cited for sat ~2.2× above the fastest of its own
39 soils. Retiring the parameter is what discharged that mismatch; the provenance record
is preserved in ``docs/plans/post-roadmap-nitrogen-cycle-form.md``. This module's two
decomposer-carried rates now come from ``params/decomposition.yaml`` and
``params/microbial_respiration.yaml``, which own their own provenance.
"""

from dataclasses import dataclass

from domains.biosphere.allocation import SenescenceParams, senescence_flux
from domains.biosphere.chamber import oxygen_limitation_factor
from domains.biosphere.decomposition import DecompositionParams, decomposition_flux
from domains.biosphere.microbial_respiration import (
    MicrobialRespirationParams,
    microbial_respiration_flux,
)
from domains.biosphere.nitrogen import NitrogenParams
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


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


def carried_nitrogen(moved_carbon: float, pool_n: float, pool_c: float) -> float:
    """The nitrogen belonging to ``moved_carbon`` at the donor pool's own N:C.

    The one kernel behind both microbe-mediated legs: a carbon flux leaving a pool takes
    that pool's nitrogen with it, ``moved_C · (pool_N / pool_C)``. Uniform composition —
    the pool is well-mixed, so what leaves carries the average, not a preferentially
    enriched or depleted fraction.

    Positivity is structural and inherited, not re-argued: the carbon flux is itself a
    clamped first-order withdrawal ``k · pool_C``, so the N withdrawal is ``k · pool_N``
    — the same bound (``k·dt < 1``) that keeps the carbon leg from over-running its pool
    keeps this one from over-running the N pool. Returns 0.0 for an empty or
    non-positive pool (never a divide-by-zero, never a negative leg).
    """
    if moved_carbon <= 0.0 or pool_n <= 0.0 or pool_c <= 0.0:
        return 0.0
    return moved_carbon * (pool_n / pool_c)


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
class LitterNitrogenTransfer:
    """NITROGEN flow ``litter_n -> microbial_n``, carried by the decomposed carbon.

    The nitrogen leg of :class:`decomposition.Decomposition`: the N that belongs to the
    litter carbon decomposing into microbial biomass this step, at the litter pool's own
    N:C. **The carbon flux is recomputed here from the same** ``DecompositionParams``
    **object**, not shared through state — a flow may read only the step-entry snapshot,
    so recomputation is the only pure form (the :class:`NitrogenSenescence` idiom, one
    flow over). The hazard that creates — the two drifting apart if someone changes the
    carbon side alone — is pinned by a test asserting this flow's ``decomposed_C``
    equals ``Decomposition``'s own leg.

    Single-currency NITROGEN (both pools are ``{NITROGEN: 1}``), so the
    conservation gate folds it exactly. Self-limiting (the withdrawal is
    ``decomposition_rate · litter_n``
    by the identity in the module docstring), so ``rationed == 0`` is structural under
    the same ``k·dt < 1`` the carbon leg already carries. Sealed-chamber only.
    ``flux = daily·dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    litter_n: StockId
    microbial_n: StockId
    litter_carbon: StockId
    params: DecompositionParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        stocks = snapshot.stocks
        litter_c = stocks[self.litter_carbon].amount
        # The identical flux Decomposition sends litter_carbon → microbial_carbon.
        decomposed = (
            decomposition_flux(
                litter_c, decomposition_rate=self.params.decomposition_rate
            )
            * dt
        )
        moved = carried_nitrogen(decomposed, stocks[self.litter_n].amount, litter_c)
        return FlowResult(
            legs=(
                Leg(self.litter_n, -moved),
                Leg(self.microbial_n, moved),
            )
        )


@dataclass(frozen=True)
class MicrobialNitrogenRelease:
    """NITROGEN flow ``microbial_n -> soil_n``, carried by the respired carbon.

    The nitrogen leg of :class:`microbial_respiration.MicrobialRespiration`, and the
    close of the cycle ``soil_n → plant_n → litter_n → microbial_n → soil_n`` that
    Phase 1 fed externally from an ``n_source`` BOUNDARY. As microbes respire their
    carbon back to CO₂ they release the nitrogen that carbon carried, at the microbial
    pool's own N:C.

    **``f_O2`` is applied here too, and that is the clearest reason this flow recomputes
    rather than reusing a bare rate:** microbial respiration self-throttles as O₂
    depletes, so the N release must throttle with it — a collapsed
    ``microbial_respiration_rate · microbial_n`` would keep mineralizing nitrogen from a
    pool whose carbon had stopped moving.

    Single-currency NITROGEN. Self-limiting in the substrate (∝ the microbial N pool)
    and in O₂ (the ``f_O2`` Monod factor → 0 as O₂ → 0), so ``rationed == 0`` is
    structural.
    Sealed-chamber only. ``flux = daily · f_O2 · dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    microbial_n: StockId
    soil_n: StockId
    microbial_carbon: StockId
    o2_pool: StockId
    params: MicrobialRespirationParams
    # Total chamber air (mol) — the intensive basis for the ``f_O2`` O₂ mole fraction,
    # exactly as MicrobialRespiration takes it (from ``scenario.chamber_air_mol``).
    air_mol: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        stocks = snapshot.stocks
        microbial_c = stocks[self.microbial_carbon].amount
        # The identical flux MicrobialRespiration burns to CO₂ — f_O2 included.
        f_o2 = oxygen_limitation_factor(
            stocks[self.o2_pool].amount,
            air_mol=self.air_mol,
            k_o2=self.params.o2_half_saturation,
        )
        respired = (
            microbial_respiration_flux(
                microbial_c,
                microbial_respiration_rate=self.params.microbial_respiration_rate,
            )
            * f_o2
            * dt
        )
        moved = carried_nitrogen(respired, stocks[self.microbial_n].amount, microbial_c)
        return FlowResult(
            legs=(
                Leg(self.microbial_n, -moved),
                Leg(self.soil_n, moved),
            )
        )
