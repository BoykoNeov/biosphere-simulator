"""The minimal consumer: grazing, consumer respiration, mortality (P3 Step 7).

The optional stretch step — one herbivore proving the **trophic pattern** the roadmap's
"optional consumers" compartment names: *graze plant biomass → consumer biomass →
respiration CO₂ + death-to-litter*. It is the codebase's existing minimal consumer
(``decomposition`` + ``microbial_respiration`` — microbes eating *dead* litter,
first-order on the resource, burning biomass back to CO₂) lifted **one trophic level**:
the consumer eats *live* ``leaf_c`` and routes its death to litter. So the three flows
here mirror flows that already exist —

* **Grazing** mirrors :class:`~domains.biosphere.decomposition.Decomposition`
  (``litter_carbon → microbial_carbon``): ``leaf_c -> consumer_carbon`` (Σ legs = 0).
  First-order **donor-controlled** intake ``graze = k_graze · leaf_c`` (mol C day⁻¹),
  proportional to the standing leaf, so it → 0 as leaf → 0 (positivity is structural: a
  clamped POPULATION withdrawal ∝ the resource's own start-of-step amount, so
  ``k_graze·dt < 1`` keeps the Euler backstop unfired). Both pools are pure carbon
  (``{CARBON: 1}``), so this is a single-currency CARBON transfer — no core change.
* **ConsumerRespiration** mirrors
  :class:`~domains.biosphere.microbial_respiration.MicrobialRespiration`
  (``microbial_carbon + o2_pool → carbon_pool``): ``consumer_carbon + o2_pool →
  carbon_pool`` (the CO₂ POOL). First-order in standing consumer biomass ``resp = c_resp
  · consumer_carbon`` (mol C day⁻¹), the metabolic cost. Multi-quantity (CARBON+OXYGEN)
  at PQ=1: each mol C respired returns 1 mol CO₂ to the pool (carrying 2 oxygens) and
  consumes 1 mol O₂ — CARBON ``−1 + 1 = 0``, OXYGEN ``+2 − 2 = 0``, balanced in one flow
  by the P2.1 composition fold. The draw self-limits two ways (substrate ∝ the consumer
  pool; O₂ via the ``f_O2`` Monod factor → 0 as O₂ → 0), so ``rationed == 0`` holds.
* **ConsumerMortality** mirrors :class:`~domains.biosphere.allocation.Senescence`
  (organs → litter): ``consumer_carbon -> litter_carbon`` (Σ legs = 0). A first-order
  relative death rate ``death = mortality · consumer_carbon`` (mol C day⁻¹),
  self-limiting (→ 0 as the consumer → 0). Death routes to the in-system
  ``litter_carbon`` POOL (decomposable), **never** to the numerical extinction loss-sink
  (decision #6 / P3.4 closure-preserving mortality) — the herbivore's carcass re-enters
  the cycle, so the sealed chamber stays genuinely closed.

The three flows compose a closed sub-loop with no control code: ``leaf_c`` → grazing →
``consumer_carbon`` → {respiration → ``carbon_pool`` → photosynthesis; mortality →
``litter_carbon`` → decomposition → … → CO₂}. The cascade (grazing reduces leaf; the
consumer biomass tracks food; respiration returns CO₂; mortality feeds litter) is
**emergent from stock coupling** — each flow names a sibling compartment's shared stock
and the resolver (#16) cannot tell shared from forcing.

**Why first-order donor control, not a bilinear functional response (the honest part).**
A predator-prey ``k · leaf_c · consumer_carbon`` (Lotka–Volterra / Holling Type I)
intake
gives consumer-driven limit cycles — but that is the **explicitly deferred** full
trophic-web / coupled-dynamics scope. It would also forfeit three properties the engine
treats as load-bearing: structural ``rationed == 0`` (a bilinear leaf-withdrawal rate
``k·consumer·dt`` is dynamic, not a fixed ``k·dt < 1``); closure safety (a near-zero
consumer could cross its extinction threshold → route to the BOUNDARY loss-sink → break
"genuinely closed"); and reset-guard safety (an amplifying graze draw makes it harder to
keep the plant filling grain for ``annual_reset``). First-order donor control is the
established minimal idiom (decomposition / senescence) and is exactly what Step 7 must
prove — that a trophic *level composes* under conservation / closure / ``rationed ==
0``,
not that it oscillates. **Deferred seams:** the bilinear/Holling functional response;
assimilation efficiency < 1 (egesta/feces routed straight to litter rather than the 1:1
intake → biomass here); and consumer phenology / multiple consumer levels (trophic
webs).

Pure stdlib only. Citation: the first-order biomass-turnover / donor-controlled intake
form is the standard minimal treatment (e.g. the linear functional response of Lotka,
A.J. (1925), *Elements of Physical Biology*, Williams & Wilkins; and first-order
biomass respiration/turnover as in CENTURY-family soil-carbon models); rate values are
provisional ``TODO(cite)`` placeholders pending the validation gate (see
``params/herbivory.yaml``), clean-room.
"""

from dataclasses import dataclass

from domains.biosphere.chamber import oxygen_limitation_factor
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class HerbivoryParams:
    """Loader-produced consumer parameters: the three first-order rates + the O₂ Monod.

    Provisional literature-typical placeholders pending the validation gate (see
    ``params/herbivory.yaml``). Zero rates are valid (no grazing / no respiration / no
    mortality); negative is rejected at the loader.
    """

    # k_graze, first-order grazing intake leaf_c → consumer_carbon (mol C/mol C/day)
    grazing_rate: float
    # c_resp, first-order consumer respiration consumer_carbon → CO₂ (mol C/mol C/day)
    respiration_rate: float
    # mortality, first-order death-to-litter consumer_carbon → litter (mol C/mol C/day)
    mortality_rate: float
    # O₂ half-saturation (mole fraction) for the ``f_O2`` self-limit (the
    # ``microbial_respiration`` precedent); low/sharp ⇒ ``f_O2 ≈ 1`` until near-anoxia.
    # 0 disables the limit.
    o2_half_saturation: float


def grazing_flux(leaf_c: float, *, grazing_rate: float) -> float:
    """Daily grazing intake ``grazing_rate · leaf_c`` (mol C day⁻¹).

    First-order donor-controlled (the decomposition / senescence form), so it → 0 as the
    leaf → 0 (positivity is structural). The grazed carbon enters the consumer biomass
    pool (the :class:`Grazing` flow).
    """
    return grazing_rate * leaf_c


def consumer_respiration_flux(consumer_c: float, *, respiration_rate: float) -> float:
    """Daily consumer respiration ``respiration_rate · consumer_c`` (mol C day⁻¹).

    First-order in standing consumer biomass, so it → 0 as the consumer → 0 (positivity
    is structural — the microbial-respiration / maintenance self-limiting pattern). The
    respired carbon returns to the CO₂ pool and consumes an equal amount of O₂ (PQ=1;
    the
    :class:`ConsumerRespiration` flow).
    """
    return respiration_rate * consumer_c


def mortality_flux(consumer_c: float, *, mortality_rate: float) -> float:
    """Daily consumer mortality ``mortality_rate · consumer_c`` (mol C day⁻¹).

    First-order relative death rate (the senescence form), so it → 0 as the consumer → 0
    (positivity is structural). The dead biomass routes to the litter POOL (the
    :class:`ConsumerMortality` flow), never the numerical loss-sink (P3.4 / #6).
    """
    return mortality_rate * consumer_c


@dataclass(frozen=True)
class Grazing:
    """CARBON transfer flow ``leaf_c -> consumer_carbon`` (balanced, P3 Step 7).

    Transfers ``grazing_flux(leaf_c, k_graze)·dt`` of carbon from the live leaf into the
    consumer biomass each step — the trophic link (the ``Decomposition`` pattern one
    level up, eating live tissue rather than dead litter). Single-currency CARBON (both
    pools are ``{CARBON: 1}``), so the every-step conservation gate folds it identically
    to the Phase-1 flows; no O₂ is involved (the consumer's gas exchange is the separate
    :class:`ConsumerRespiration`). The draw is self-limiting (∝ the leaf's amount), so
    ``rationed == 0`` is structural (``k_graze·dt < 1``). ``flux = daily·dt``
    (dt-linear).
    """

    id: FlowId
    priority: int
    leaf_c: StockId
    consumer_carbon: StockId
    params: HerbivoryParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        grazed = (
            grazing_flux(
                snapshot.stocks[self.leaf_c].amount,
                grazing_rate=self.params.grazing_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.leaf_c, -grazed),
                Leg(self.consumer_carbon, grazed),
            )
        )


@dataclass(frozen=True)
class ConsumerRespiration:
    """CARBON+OXYGEN flow ``consumer_carbon + o2_pool → carbon_pool`` (P3 Step 7).

    Burns ``consumer_respiration_flux(consumer_carbon, c_resp)·dt`` of consumer biomass
    back to CO₂ each step — the herbivore's metabolic cost and its contribution to the
    chamber gas exchange (closing the carbon loop consumer → CO₂ → photosynthesis). At
    PQ=1 the three legs balance CARBON **and** OXYGEN in one flow (the P2.1 composition
    fold): the CO₂ returned to the pool carries 2 oxygens supplied by the consumed O₂,
    so
    OXYGEN nets to zero. Sealed-chamber only (the pools/consumer exist only then) —
    always
    three legs, no ``source == sink`` netting (the exact :class:`MicrobialRespiration`
    shape). The draw self-limits two ways, so ``rationed == 0`` is structural: in the
    substrate (∝ the consumer pool's amount, ``c_resp·dt < 1``) **and** in O₂ (the
    ``f_O2`` Monod factor → 0 as O₂ → 0). ``flux = daily · f_O2 · dt`` — dt-linear.
    """

    id: FlowId
    priority: int
    consumer_carbon: StockId
    co2_pool: StockId
    o2_pool: StockId
    params: HerbivoryParams
    # Total chamber air (mol) — the intensive basis for the ``f_O2`` O₂ mole fraction
    # (the microbial-respiration precedent). Passed from ``scenario.chamber_air_mol``.
    air_mol: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        # f_O2 self-limit (the microbial-respiration precedent): the O₂ draw shuts off
        # smoothly as O₂ → 0, so it never over-runs the pool on a depleting chamber. At
        # the perennial ample-O₂ fill f_O2 ≈ 1 (low/sharp K_O2).
        f_o2 = oxygen_limitation_factor(
            snapshot.stocks[self.o2_pool].amount,
            air_mol=self.air_mol,
            k_o2=self.params.o2_half_saturation,
        )
        respired = (
            consumer_respiration_flux(
                snapshot.stocks[self.consumer_carbon].amount,
                respiration_rate=self.params.respiration_rate,
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
                Leg(self.consumer_carbon, -respired),
                Leg(self.co2_pool, respired),
                Leg(self.o2_pool, -respired),
            )
        )


@dataclass(frozen=True)
class ConsumerMortality:
    """CARBON loss flow ``consumer_carbon -> litter_carbon`` (balanced, P3 Step 7).

    Routes ``mortality_flux(consumer_carbon, mortality)·dt`` of dead consumer biomass to
    the in-system ``litter_carbon`` POOL each step — death-to-litter (P3.4: the carcass
    is decomposable, never routed to the numerical extinction loss-sink, so the sealed
    chamber stays genuinely closed). The ``Senescence`` pattern for a single pool.
    Single-currency CARBON (both pools are ``{CARBON: 1}``). Self-limiting (∝ the
    consumer
    pool's amount), so ``rationed == 0`` is structural (``mortality·dt < 1``).
    ``flux = daily·dt`` (dt-linear).
    """

    id: FlowId
    priority: int
    consumer_carbon: StockId
    litter_carbon: StockId
    params: HerbivoryParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        died = (
            mortality_flux(
                snapshot.stocks[self.consumer_carbon].amount,
                mortality_rate=self.params.mortality_rate,
            )
            * dt
        )
        return FlowResult(
            legs=(
                Leg(self.consumer_carbon, -died),
                Leg(self.litter_carbon, died),
            )
        )
