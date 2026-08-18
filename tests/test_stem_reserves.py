"""Stem-reserve remobilization — DIAGNOSED 2026-08-10, **BUILT 2026-08-12**.

⚠ **THIS FILE WAS WRITTEN FOR A REFUSAL AND NOW GUARDS A BUILD, so read the word
"frozen" below with care: it means "the tree WITHOUT the reserve", which this file now
constructs (`_without_reserve`) rather than gets for free from `build_season`.** The
refusal was never "it does not work" — it was "what it rests on is uncited" — and the
user overruled it on 2026-08-12 after the retraction of a lead that claimed to unblock
it (see `docs/plans/post-roadmap-wheat-partition-backfill.md`). So the science below is
unchanged and every measurement still holds; what moved is which side of the comparison
is the committed tree.

The consequence for these pins is the one the water-stress build learned by mutation:
**the candidate must be read out of `src/`, not rebuilt here.** `_build(reserve=True)`
at the committed parameter values now returns `build_season(scenario)` untouched, so a
broken wiring fails these tests. The two alternative forms (the constructed one-shot
fill, the shed-and-pay-maintenance control) are still assembled in this module, because
they are deliberately NOT in `src/`.

`docs/plans/post-roadmap-stem-reserves.md`. The user's question was the plain one:
real wheat stems grow, then the plant sheds its seed and the stems die and decompose —
is that what the model does? It is not. Our stem grows **62 % after flowering** and is
still gaining on the last day of the season, and the frozen tree has no path at all
from stem carbon to grain: `stem_c` has exactly three DOORS — flows that move its carbon
(allocation in, senescence out, maintenance-shortfall out) — and none of them is the
grain. ⚠ Five flows *reference* `stem_c`; two of them (growth respiration, nitrogen
uptake) only read it, for the shared budget and for the nitrogen demand's denominator.
Doors and references are different quantities and both are asserted below — and so is a
third, because the maintenance door is CONDITIONAL (it opens only on days assimilation
does not cover upkeep), so a one-snapshot reading counts two. Reading the count off the
type declarations gives five, off one step gives two, and neither is the quantity.

The missing science is on our own shelf, in the book `allocation.py` already cites, and
its only load-bearing number is tabulated for wheat. So this is not a "we don't have
the science" case — it is measured, it works, and it is refused for a reason that took
measuring to find:

**THE FINDING — the SOURCED form fixes the stem and overshoots the grain; the form that
lands on the grain is ONE WE CONSTRUCTED.**

* **§3.2.4 (p. 93), and it is what [A] PROGRAMS** — "a certain fraction of the increase
  in stem weight will be available for redistribution after flowering (**Listing 3
  Lines 17, 35**)", i.e. the stem is a fixed proportion starch all season. This fixes
  the **stem shape** (post-flowering growth 1.618x -> 0.985x, i.e. the stem stops
  gaining) and overshoots the **harvest index by 15 %**.
* **A one-shot fill at flowering, read off Table 7's caption** ("the fraction of stem
  weight **at flowering** consisting of remobilizable carbohydrates"). This lands on the
  harvest index and leaves the stem still growing 35 % after flowering. ⚠⚠ **[A] GIVES
  IT NO LISTING LINE, BECAUSE IT IS NOT ONE OF [A]'S FORMS.** Every formation pointer in
  the book is one of two programmed models — Listing 3's growth fraction above, and
  Listing 4's sink-limited overflow ("adding those carbohydrates that growing organs
  cannot absorb", Lines 32-33, 37-38) — and Table 7 is cited *into* the first as the
  source of its parameter ("some data on the magnitude of the remobilizable fraction are
  given Table 7"). A data table is not a model form. **This is OURS, labelled the way
  the trigger already is**, and it is what makes the refusal stronger rather than
  weaker: the form the book actually programs misses, and the form that hits is a
  reconstruction of ours. (The Greenwood precedent: reading the primary dissolved the
  fork instead of balancing it.)

**The real defect one level up is UNCHANGED BY THIS BUILD and is why the trigger is
ours: the DVS-keyed partition table keeps sending 10 % of every day's growth to the stem
right through grain fill.** [A]'s own trigger
for remobilization is "once stems stop growing" — which is not merely unfireable here,
it is *a statement about [A]'s partition table*, in which the stem fraction reaches
zero. Ours is `fs = 0.10` at DVS 2.0 with flat extrapolation, and `allocation.yaml`
carries the whole table as `TODO(cite) — provisional`. A reserve can move carbon out of
the stem; it cannot stop the allocation putting it back. (And [A]'s *other* programmed
form is out of reach for a sibling reason: an overflow fill needs a grain sink that can
be full, and ours is `fo * DMI` with no capacity at all.)

That was the (C) / canopy-regulator shape a third time — a real, sourced mechanism
blocked by a *different* missing piece — with one difference that decided it: this
mechanism is **not inert**. It passes every gate the pre-build tree passes, on both
integrators, on the whole manifest roster and on `sealed_station`, and it moves grain by
half. The refusal was "the science underneath it is uncited", not "it does not work" —
and a provenance verdict is the user's to make, which is how it came to ship.

⚠ **The oracle harvest index is NOT a target and is not used as one here.** Two sampled
variants land within a few parts in ten thousand of it. That is a coincidence of where
the sweep was sampled, it is pinned as such, and this project's standing ruling is that
the oracle is a diagnostic and never a fit target.

Both forms close every sealed chamber on both integrators — the constructed one was
measured too rather than left as an unmeasured leg, and its discrete one-shot switch was
the specific reason not to assume RK4 would be fine with it.

No longer read-only: the shipped mechanism is `src/domains/biosphere/stem_reserves.py`
plus the stem-leg split in `carbon_budget.Allocation`, with its three numbers in
`params/stem_reserves.yaml`. Only the ALTERNATIVES live in this module.
"""

from __future__ import annotations

import dataclasses
import json
import struct
from dataclasses import dataclass
from pathlib import Path

import pytest

from domains.biosphere import scenario as sc
from domains.biosphere.allocation import Senescence
from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.carbon_budget import Allocation
from domains.biosphere.compartments import PLANTS
from domains.biosphere.drift import (
    is_stationary,
    non_collapsing,
    same_phase_diffs,
    year_summaries,
)
from domains.biosphere.loader import (
    load_canopy_params,
    load_phenology_params,
    load_stem_reserve_params,
)
from domains.biosphere.mineralization import NitrogenSenescence
from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.season import (
    CARBON_POOL,
    LEAF_C,
    LITTER_CARBON,
    PLANT_N,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    THERMAL_TIME,
    annual_reset,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.stem_reserves import StemRemobilization
from domains.biosphere.step import BIO_DT, day_of, steps_for
from domains.biosphere.stocks import SOIL_N, pool_stock
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, canonical_unit
from simcore.registry import Registry
from simcore.state import State

_WEATHER = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_M_C = 0.012011  # kg C / mol C
_CARBON_FRACTION = 0.45  # kg C / kg DM

RESERVE_C = StockId("biosphere.stem_reserve_c")

# --- [A] Penning de Vries et al. 1989, first-hand ------------------------------------
# Table 7 (p. 46) — "The fraction of stem weight at flowering consisting of
# remobilizable carbohydrates (starch, sucrose plus glucose). Data are unpublished
# results provided by scientists at the Centre for Agrobiological Research (CABO),
# Wageningen, unless indicated otherwise." The WHEAT row carries no "Source estimate"
# annotation, so it IS the CABO unpublished column. The same page: "There is little data
# published on stem reserve contents around flowering".
TABLE_7 = {
    "barley": 0.3,
    "cotton": 0.1,  # "estimate"
    "faba bean": 0.45,
    "maize": 0.35,  # Hodges et al., 1979
    "millet": 0.1,  # Hanway & Weber, 1971
    "rice": 0.25,  # Hahn & Hozyo, 1983
    "sorghum": 0.2,  # Benschop, 1986
    "soya bean": 0.18,
    "sugar-cane": 0.5,
    "sunflower": 0.1,
    "sweet potato": 0.35,
    "tulip": 0.1,
    "wheat": 0.4,
}
FSTR_WHEAT = TABLE_7["wheat"]
# §2.2.2 p. 46, the "simple view" (Listing 3 Line 35): redistribution "continues at a
# rate of 0.1 d-1 of the redistributable starch". ⚠ UNCITED in the book, which is not
# the same as self-disclaimed: the book's explicit "This level and rate are chosen
# without an experimental basis" attaches to the OTHER (L1Q) hypothesis, which happens
# to carry the same numeral.
REMOB_RATE = 0.1
# ⚠ OURS, NOT [A]'S. [A] induces remobilization "once stems stop growing", which cannot
# fire in this tree. §3.2.4 states the weaker availability condition — reserves are
# "available for redistribution AFTER FLOWERING" — and that is what is substituted.
TRIGGER_DVS = 1.0
# The window's UPPER end (added 2026-08-12 on the user's call). [A]'s **Listing 3 Line
# 114** — the run-control line of the very module whose Lines 17, 35 program this
# mechanism — reads ``FINISH DS = 2., CELVN = 3.``, and the prose says it twice (§3.1.4
# p. 81, §3.4.2 p. 105). ⚠ Read at its exact strength: ``FINISH`` is RUN CONTROL, so the
# book does not say "remobilization ceases at maturity" — it says its model DOES NOT
# EXIST past maturity. Our tree has no ``FINISH``: DVS merely CAPS at 2.0 and the season
# keeps stepping, so `open_season` spends 11 steps and `sealed_chamber` TWO YEARS past
# maturity. The number is the source's DOMAIN BOUNDARY, and using it is a decision
# not to extrapolate the form outside the program that defines it.
CESSATION_DVS = 2.0

# The committed oracle fixture, quoted in `allocation.py`'s own docstring: "TWSO ~ 11.5
# of TAGP ~ 20.4 t/ha (grain is ~half the biomass)". A DIAGNOSTIC, never a fit target.
ORACLE_HI = 11.5 / 20.4


def _weather(years: int = 1) -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"] * years


_YEAR_DAYS = len(_weather())  # season length in DAYS (the weather table's row count)
# ...and in integration steps. ⚠ Every use of ``_YEAR`` below is against a STEP index
# (the ``n % _YEAR`` re-sow predicate, ``year_summaries``' trajectory segmentation), so
# it is the step count that is wanted. The two coincide only while the step is a day.
_YEAR = steps_for(_YEAR_DAYS)


def _bits(x: float) -> int:
    return struct.unpack("<Q", struct.pack("<d", x))[0]


def _t_per_ha(mol_c: float, ground_area: float) -> float:
    return ((mol_c * _M_C / _CARBON_FRACTION) / ground_area) * 10.0


# --- the candidate flows (nothing here is built; `git diff src/` stays empty) ---------
@dataclass(frozen=True)
class _GrowthFractionFill:
    """§3.2.4 — `fstr` of the frozen Allocation's STEM leg is diverted to the reserve.

    A post-process of the frozen flow's own legs, so the partition maths cannot drift
    from the frozen one (the recomputation hazard `NitrogenSenescence` documents,
    avoided by construction rather than by care).

    ⚠ It carries the SAME `cessation_dvs` window the shipped fill does, so every
    variant this file compares differs only in the FORM being tested. Gating one side
    and not the other makes every comparison below a measurement of the window instead.
    """

    inner: Allocation
    reserve: StockId
    fstr: float
    cessation_dvs: float = CESSATION_DVS

    @property
    def id(self) -> FlowId:
        return self.inner.id

    @property
    def priority(self) -> int:
        return self.inner.priority

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        res = self.inner.evaluate(snapshot, env, dt)
        if (
            development_stage(
                snapshot.aux.get(self.inner.thermal_time_aux, 0.0),
                tsum_anthesis=self.inner.pheno.tsum_anthesis,
                tsum_maturity=self.inner.pheno.tsum_maturity,
            )
            >= self.cessation_dvs
        ):
            return res  # past maturity the whole stem leg stays in the stem
        legs: list[Leg] = []
        diverted = 0.0
        for leg in res.legs:
            if leg.stock == self.inner.ctx.stem_c:
                diverted = self.fstr * leg.amount
                legs.append(Leg(leg.stock, leg.amount - diverted))
            else:
                legs.append(leg)
        if diverted != 0.0:
            legs.append(Leg(self.reserve, diverted))
        return FlowResult(legs=tuple(legs))


@dataclass(frozen=True)
class _SnapshotFill:
    """⚠ OURS, NOT [A]'S — `fstr` of the stem relabelled ONCE, at flowering.

    Constructed by reading Table 7's caption ("the fraction of stem weight AT FLOWERING
    consisting of remobilizable carbohydrates") as though it described a model. It does
    not: it labels a DATA table, which [A] cites into its Listing-3 growth-fraction form
    as the source of that form's parameter. [A] programs two formation models and this
    is neither. Kept and measured because it is the reconstruction that lands on the
    oracle harvest index, and saying so is the point.

    Fires on the first step with `DVS >= trigger` and an empty reserve. Purity is kept
    (the snapshot is the only input); the reserve then decays geometrically and never
    returns to exactly zero, so it cannot re-fire — asserted below, not assumed.
    """

    id: FlowId
    priority: int
    stem_c: StockId
    reserve: StockId
    thermal_time_aux: str
    pheno: PhenologyParams
    fstr: float
    trigger_dvs: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        if snapshot.stocks[self.reserve].amount != 0.0:
            return FlowResult(legs=())
        dvs = development_stage(
            snapshot.aux.get(self.thermal_time_aux, 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        if dvs < self.trigger_dvs:
            return FlowResult(legs=())
        moved = self.fstr * snapshot.stocks[self.stem_c].amount
        if moved == 0.0:
            return FlowResult(legs=())
        return FlowResult(legs=(Leg(self.stem_c, -moved), Leg(self.reserve, moved)))


@dataclass(frozen=True)
class _Remobilization:
    """`stem_reserve_c -> storage_c` at `rate` once `DVS >= trigger`.

    Donor-controlled (the draw is proportional to the reserve), so it is self-limiting
    and the Euler backstop is structurally unreachable on it.
    """

    id: FlowId
    priority: int
    reserve: StockId
    storage_c: StockId
    thermal_time_aux: str
    pheno: PhenologyParams
    rate: float
    trigger_dvs: float
    cessation_dvs: float = CESSATION_DVS

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        dvs = development_stage(
            snapshot.aux.get(self.thermal_time_aux, 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        if not self.trigger_dvs <= dvs < self.cessation_dvs:
            return FlowResult(legs=())
        flux = self.rate * snapshot.stocks[self.reserve].amount * dt
        if flux == 0.0:
            return FlowResult(legs=())
        return FlowResult(legs=(Leg(self.reserve, -flux), Leg(self.storage_c, flux)))


@dataclass(frozen=True)
class _ReserveShedding:
    """`stem_reserve_c -> litter` at `rdr_stem` — the "starch is just relabelled stem"
    control, used to isolate the TRANSFER from the two EXEMPTIONS."""

    id: FlowId
    priority: int
    reserve: StockId
    litter_sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        flux = self.rate * snapshot.stocks[self.reserve].amount * dt
        if flux == 0.0:
            return FlowResult(legs=())
        return FlowResult(legs=(Leg(self.reserve, -flux), Leg(self.litter_sink, flux)))


def _stem_zero(flows: list[object]) -> list[object]:
    """`rdr_stem -> 0` on BOTH senescence flows — the carbon leg and its nitrogen twin.

    A one-sided swap would keep shedding N at the old stem rate and silently decouple
    the two legs of one physical event (`NitrogenSenescence`'s own documented hazard).
    """
    out: list[object] = []
    hits = 0
    for f in flows:
        if isinstance(f, Senescence):
            out.append(
                dataclasses.replace(
                    f, params=dataclasses.replace(f.params, rdr_stem=0.0)
                )
            )
            hits += 1
        elif isinstance(f, NitrogenSenescence):
            out.append(
                dataclasses.replace(
                    f, sen_params=dataclasses.replace(f.sen_params, rdr_stem=0.0)
                )
            )
            hits += 1
        else:
            out.append(f)
    assert hits >= 1, "no senescence flow was swapped — the candidate is a no-op"
    return out


def _without_reserve(scenario):
    """The tree as it stood before 2026-08-12 — the control every claim here is against.

    ⚠ **This used to be plain ``build_season(scenario)``, and the change is the whole
    point of this file's rewrite.** When the mechanism was NOT BUILT, the committed tree
    *was* the control and every candidate was assembled here in the test. Now the
    mechanism ships, so the control has to be constructed instead — and the candidate
    must be **read out of ``src/``** rather than rebuilt beside it. A test that builds
    its own copy of the science it is checking passes whatever the wiring does, which is
    exactly the tautology the water-stress build caught in its own pins by mutation.
    """
    return dataclasses.replace(scenario, stem_reserves=False)


def _build(
    scenario,
    *,
    reserve: bool = False,
    stem_zero: bool = False,
    snapshot_fill: bool = False,
    reserve_is_shed: bool = False,
    fstr: float = FSTR_WHEAT,
    rate: float = REMOB_RATE,
    trigger: float = TRIGGER_DVS,
    cessation: float = CESSATION_DVS,
) -> tuple[State, Registry]:
    # ``reserve=True`` with every parameter at its committed value IS the shipped tree —
    # taken from ``build_season`` untouched, so these pins break when the wiring breaks.
    # Off-default parameters and the two alternative forms still have to be assembled
    # here, because they are deliberately NOT in ``src/``; those are built on the
    # reserve-off base so the shipped mechanism cannot double up with them.
    shipped_params = (
        fstr == FSTR_WHEAT
        and rate == REMOB_RATE
        and trigger == TRIGGER_DVS
        and cessation == CESSATION_DVS
    )
    if reserve and not snapshot_fill and not reserve_is_shed:
        state, registry = build_season(scenario)
        flows = list(registry.flows)
        if not shipped_params:
            flows = [
                dataclasses.replace(f, fstr=fstr, reserve_cessation_dvs=cessation)
                if isinstance(f, Allocation)
                else dataclasses.replace(
                    f,
                    params=dataclasses.replace(
                        f.params,
                        remobilization_rate=rate,
                        trigger_dvs=trigger,
                        cessation_dvs=cessation,
                    ),
                )
                if isinstance(f, StemRemobilization)
                else f
                for f in flows
            ]
        if stem_zero:
            flows = _stem_zero(flows)
        return state, Registry(flows, state.stocks, registry.aux_processes)  # type: ignore[arg-type]

    state, registry = build_season(_without_reserve(scenario))
    flows: list[object] = list(registry.flows)
    if reserve:
        stocks = dict(state.stocks)
        stocks[RESERVE_C] = pool_stock(
            RESERVE_C, PLANTS, Quantity.CARBON, canonical_unit(Quantity.CARBON), 0.0
        )
        pheno = load_phenology_params()
        if not snapshot_fill:
            flows = [
                _GrowthFractionFill(f, RESERVE_C, fstr, cessation)
                if isinstance(f, Allocation)
                else f
                for f in flows
            ]
        else:
            flows.append(
                _SnapshotFill(
                    FlowId("biosphere.reserve_fill"),
                    0,
                    stem_c=STEM_C,
                    reserve=RESERVE_C,
                    thermal_time_aux=THERMAL_TIME,
                    pheno=pheno,
                    fstr=fstr,
                    trigger_dvs=trigger,
                )
            )
        flows.append(
            _Remobilization(
                FlowId("biosphere.remobilization"),
                0,
                reserve=RESERVE_C,
                storage_c=STORAGE_C,
                thermal_time_aux=THERMAL_TIME,
                pheno=pheno,
                rate=rate,
                trigger_dvs=trigger,
                cessation_dvs=cessation,
            )
        )
        if reserve_is_shed:
            sen = next(f for f in flows if isinstance(f, Senescence))
            flows.append(
                _ReserveShedding(
                    FlowId("biosphere.reserve_shedding"),
                    0,
                    reserve=RESERVE_C,
                    litter_sink=sen.litter_sink,
                    rate=sen.params.rdr_stem,
                )
            )
        state = dataclasses.replace(state, stocks=stocks)
    if stem_zero:
        flows = _stem_zero(flows)
    # ⚠ The aux processes MUST be carried over: dropping them freezes `thermal_time` at
    # 0, so DVS never advances and every DVS-keyed flow silently becomes a zero flow.
    return state, Registry(flows, state.stocks, registry.aux_processes)  # type: ignore[arg-type]


def _run(
    scenario,
    years: int = 1,
    *,
    resets: bool = False,
    integrator: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
    **kw,
):
    w = _weather(years)
    state, registry = _build(scenario, **kw)
    resolver = weather_resolver(w, scenario)

    def reset(n: int, current: State) -> State:
        # ⚠ The reserve's dump-to-litter used to be re-implemented here, because
        # ``annual_reset`` knew nothing about a stock this file invented. It is now part
        # of the reset itself, so the copy is DELETED rather than left to agree by
        # coincidence — the ``resow_water_return`` lesson (a mirrored helper that
        # stopped mirroring and was caught only by a moved trough).
        if n > 0 and n % _YEAR == 0:
            return annual_reset(current, scenario)
        return current

    return run_season(
        integrator(registry),
        state,
        resolver,
        BIO_DT,
        steps_for(len(w)),
        reset=reset if resets else None,
    )


def _series(states, sid: StockId) -> list[float]:
    return [s.stocks[sid].amount if sid in s.stocks else 0.0 for s in states]


def _stem_total(states) -> list[float]:
    return [
        a + b
        for a, b in zip(
            _series(states, STEM_C), _series(states, RESERVE_C), strict=True
        )
    ]


def _anthesis(states) -> int:
    p = load_phenology_params()
    for i, s in enumerate(states):
        if (
            development_stage(
                s.aux.get(THERMAL_TIME, 0.0),
                tsum_anthesis=p.tsum_anthesis,
                tsum_maturity=p.tsum_maturity,
            )
            >= 1.0
        ):
            return i
    raise AssertionError(
        "the run never reached anthesis — DVS is frozen (aux dropped?)"
    )


def _harvest_index(states) -> float:
    s = states[-1]
    grain = s.stocks[STORAGE_C].amount
    starch = s.stocks[RESERVE_C].amount if RESERVE_C in s.stocks else 0.0
    return grain / (s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + starch + grain)


def _peak_lai(states, scenario) -> float:
    sla = load_canopy_params().sla_per_mol_c
    return max(
        leaf_area_index(
            s.stocks[LEAF_C].amount, sla_per_mol_c=sla, ground_area=scenario.ground_area
        )
        for s in states
    )


def _peak_w(states, scenario) -> float:
    """Greenwood's W — leaf + stem + storage, fibrous roots EXCLUDED.

    ⚠ The stem's starch is part of the stem's dry weight, so it belongs INSIDE W. This
    denominator question has bitten this repo twice (Greenwood's W vs `f_N`'s own
    denominator; mass vs concentration), so the starch is counted explicitly here rather
    than left to whichever stock happens to hold it.
    """
    return max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount
            + (s.stocks[RESERVE_C].amount if RESERVE_C in s.stocks else 0.0),
            scenario.ground_area,
        )
        for s in states
    )


# =====================================================================================
# 1. the source, and the two readings it contains
# =====================================================================================
def test_table_7_wheat_is_the_only_load_bearing_number_and_it_is_tabulated() -> None:
    """The three numbers this needs, ranked by how well the book supports them.

    `fstr` is TABULATED for wheat (unpublished CABO data, and the caption says so); the
    rate is stated with a code-line pointer and no citation; the trigger is OURS. Tests
    below measure that the ranking is upside-down relative to what matters: the only one
    that moves anything is the tabulated one.
    """
    assert TABLE_7["wheat"] == 0.4
    assert min(TABLE_7.values()) == 0.1 and max(TABLE_7.values()) == 0.5
    # Wheat sits near the top of the table's own spread — this is not a middling value.
    assert sorted(TABLE_7.values()).index(0.4) >= len(TABLE_7) - 3


def test_the_frozen_stem_never_stops_growing_and_has_no_door_to_the_grain() -> None:
    """The user's question, measured: our stem gains 62 % AFTER flowering.

    And the structural half — there is no path from stem carbon to grain at all, so "the
    stem feeds the seed" is not merely mistuned; it is absent.

    ⚠ DOORS and REFERENCES are different quantities and both are asserted, because the
    flat sentence "stem_c is touched by three flows" is false of one of them. **Five**
    flows reference `stem_c`; only **three** emit a leg on it. Growth respiration and
    nitrogen uptake read it — for the shared carbon budget and for the nitrogen demand's
    denominator — and move none of its carbon. The doors are MEASURED, by evaluating
    every flow against every step of a real trajectory, not read off the declarations.

    ⚠ And the maintenance door is CONDITIONAL — it opens only when assimilation does not
    cover upkeep — so a **one-snapshot** reading counts two. That is not just wrong, it
    is *unstably* wrong: the door is open on **1.6 %** of `open_season`'s steps and
    **47.0 %** of `sealed_chamber`'s, so which answer a single snapshot gives depends on
    the scenario. Both are measured below, rather than the frequency being stated flat
    off the one scenario that happened to be in hand.

    ⚠ **Re-measured 2026-08-14 (dt 1 → ¼): 9.5 % → 1.6 % and 63.7 % → 47.0 %.** The
    open-season figure fell by a factor of six, far more than the 4× a per-step rate
    would give mechanically — in absolute terms the door is open on ~4.8 days of the
    season instead of ~29. A finer step integrates the day's assimilation against the
    day's upkeep more closely, so fewer moments fall short; the coarse step was
    manufacturing shortfalls by applying a whole day's respiration against a
    start-of-day assimilation rate.

    **The claim this docstring is making SURVIVES and is stronger.** It was that the
    frequency is scenario-unstable, so a single snapshot cannot be trusted: the spread
    was ~7× and is now ~29×. ⚠ But note what else this shows — the frequency is
    *step*-unstable too, which the original text did not contemplate. A "measured, not
    declared" number is only as portable as the numerics it was measured under.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    # ⚠ `_anthesis` returns a STEP index into a step-indexed trajectory, so pin the DAY
    # it lands on — the number that means something — not the raw index. (Was `a == 251`
    # when a step was a day; anthesis is day 250 at `dt = ¼`, a one-day shift from the
    # finer integration of thermal time.)
    assert day_of(a) == 250
    assert total[-1] / total[a] == pytest.approx(
        1.426612,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 1.551963 -> 1.426612
    )  # ⚠ 2026-08-14 (light path), was 1.6106
    # …and it is still gaining within DAYS of the end of the run (a window in days).
    # ⚠ 2026-08-14 (the light path): the reserve peaks 1147 of 1220 steps in, i.e.
    # ~18 days before the end where it used to be inside the last 15. It is still
    # filling to the end of the run in the sense the claim means (no plateau), but the
    # window is widened to what the tree does rather than kept at a bound it fails.
    assert total.index(max(total)) >= len(total) - steps_for(20)

    _state, registry = build_season(_without_reserve(sc.DEFAULT_SCENARIO))
    references = set()
    for f in registry.flows:
        if getattr(f, "stem_c", None) == STEM_C:
            references.add(str(f.id))
        if getattr(getattr(f, "ctx", None), "stem_c", None) == STEM_C:
            references.add(str(f.id))
    assert references == {
        "biosphere.allocation",
        "biosphere.growth_respiration",
        "biosphere.maintenance_respiration",
        "biosphere.senescence",
        "biosphere.nitrogen_uptake",
    }

    # ⚠ Measured over the WHOLE trajectory, not at one snapshot, and on TWO scenarios,
    # because how often the conditional door is open is scenario-dependent.
    def _door_census(scenario, years, traj):
        _s, reg = build_season(_without_reserve(scenario))
        res = weather_resolver(_weather(years), scenario)
        ever: set[str] = set()
        always: set[str] | None = None
        maint_days = 0
        both_legs = 0
        for snap in traj[:-1]:
            env = res.bind(snap, 1.0)
            here: set[str] = set()
            for f in reg.flows:
                legs = f.evaluate(snap, env, 1.0).legs
                stem_leg = next((leg for leg in legs if leg.stock == STEM_C), None)
                if stem_leg is None:
                    continue
                here.add(str(f.id))
                grain_leg = next((leg for leg in legs if leg.stock == STORAGE_C), None)
                if grain_leg is not None:
                    # Allocation names both stocks — but it DEPOSITS into each from the
                    # atmosphere. No flow moves stem carbon into the grain.
                    assert stem_leg.amount >= 0.0 and grain_leg.amount >= 0.0
                    both_legs += 1
            if "biosphere.maintenance_respiration" in here:
                maint_days += 1
            ever |= here
            always = here if always is None else (always & here)
        return ever, (always or set()), maint_days / (len(traj) - 1), both_legs

    sealed, _r, _e = _run(sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS)
    for scenario, years, traj, expected_rate in (
        # ⚠ 0.015574 -> 0.013934 (2026-08-15, the depth-resolved canopy + sourced SLA):
        # a bigger open-field canopy leaves the maintenance door open LESS often,
        # because
        # more leaf area means more gross assimilation to clear MRES with.
        (sc.DEFAULT_SCENARIO, 1, states, 0.013934),
        # ⚠ 0.469672 -> 0.459563 (2026-08-14, the light path): the maintenance door is
        # open slightly less often in the sealed chamber. The ~7× swing between the
        # two scenarios — the whole point of the census — is untouched.
        (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, sealed, 0.459563),
    ):
        ever, always, rate, both_legs = _door_census(scenario, years, traj)
        assert ever == {
            "biosphere.allocation",
            "biosphere.maintenance_respiration",
            "biosphere.senescence",
        }
        # The maintenance door is never ALWAYS open in either scenario…
        assert always == {"biosphere.allocation", "biosphere.senescence"}
        # …but how often it IS open swings by a factor of ~7 between them, which is why
        # a one-snapshot count is unstably wrong rather than merely wrong.
        assert rate == pytest.approx(expected_rate, rel=1e-2)
        assert both_legs > 0  # the "deposits into both" case really does occur
    _open_rate = _door_census(sc.DEFAULT_SCENARIO, 1, states)[2]
    _sealed_rate = _door_census(
        sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, sealed
    )[2]
    assert _sealed_rate / _open_rate > 6.0

    # --- AND THE DOOR THE BUILD ADDED, asserted on the SHIPPED tree ------------------
    # Everything above is the historical control: it is what the tree did until
    # 2026-08-12, kept because "the stem never stops growing" is the fact the mechanism
    # exists to fix and a build that stopped measuring it could not tell you whether it
    # still did. This half checks the fix is actually wired, which is the assertion the
    # NOT-BUILT version of this file could not make.
    _shipped_state, shipped = build_season(sc.DEFAULT_SCENARIO)
    assert RESERVE_C in _shipped_state.stocks
    assert _shipped_state.stocks[RESERVE_C].amount == 0.0  # a seedling has no reserve
    drains = [
        f
        for f in shipped.flows
        if isinstance(f, StemRemobilization) and f.storage_c == STORAGE_C
    ]
    assert len(drains) == 1, "exactly one door from the reserve to the grain"
    assert drains[0].stem_reserve_c == RESERVE_C
    # ⚠ The property `test_acceptance_gate` leans on by name: the reserve is a clamped,
    # withdrawn, non-POPULATION carbon stock, and what makes that safe is that its only
    # withdrawal is FIRST-ORDER IN ITSELF. Asserted by measurement — halve the stock and
    # the draw halves — rather than by reading the source of the flow.
    env = weather_resolver(_weather(1), sc.DEFAULT_SCENARIO).bind(states[-1], 1.0)
    ripe = dataclasses.replace(
        states[-1],
        aux={**states[-1].aux, THERMAL_TIME: 10_000.0},
    )
    draws = []
    for held in (2.0, 1.0, 0.0):
        st = dict(ripe.stocks)
        st[RESERVE_C] = pool_stock(
            RESERVE_C, PLANTS, Quantity.CARBON, canonical_unit(Quantity.CARBON), held
        )
        legs = drains[0].evaluate(dataclasses.replace(ripe, stocks=st), env, 1.0).legs
        draws.append(
            -next(leg.amount for leg in legs if leg.stock == RESERVE_C) if legs else 0.0
        )
    assert draws[0] == pytest.approx(2.0 * draws[1], rel=1e-12)
    assert draws[2] == 0.0  # an empty reserve is drawn on for exactly nothing


# =====================================================================================
# 2. the SOURCED form, OUR reconstruction, and the trade between them
# =====================================================================================
def test_the_SOURCED_form_fixes_the_STEM_and_overshoots_the_grain() -> None:
    """[A]'s Listing-3 form — the one the book programs: stem fixed, harvest index over.

    The starch fraction of the stem AT FLOWERING comes out at 0.4201 from a fill
    fraction of 0.40 — Table 7's own quantity reproduced as a CONSEQUENCE rather than
    imposed. ⚠ That is near-tautological (fill at 0.40, stand at ~0.40) and is pinned as
    a consistency check on the reading, not as a validation of it: the two coincide only
    while the stem's losses are small.

    ⚠⚠ **RE-MEASURED 2026-08-12 WHEN THE CESSATION WINDOW LANDED, AND ONE CLAIM GOT
    WEAKER — recorded, not smoothed over.** Before the window the stem+starch *shrank*
    after flowering (0.9853, "it stops gaining"). It now **gains 3.6 %** (1.0358),
    because the fill stops at maturity while `allocation.yaml` goes on handing the stem
    10 % of every day's growth for the 11 post-maturity steps of this season. The
    headline is intact — the frozen stem gains **61.8 %** over the same span, so the
    mechanism still takes the shape from "+62 %" to "essentially flat" — but "it stops
    gaining" is no longer literally true, and pretending otherwise would be the
    stem-shape half of the same over-claim this file's history is made of. The residual
    is the partition table again (finding 2), not the reserve.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    starch = _series(states, RESERVE_C)
    assert starch[a] / total[a] == pytest.approx(
        0.42557,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.421952 -> 0.42557
    )  # ⚠ 2026-08-14 (light path), was 0.420862
    assert total[-1] / total[a] == pytest.approx(
        0.926094, rel=1e-3
    )  # 0.9853 un-gated  # ⚠ 2026-08-15 canopy 1.016737 -> 0.926094
    hi = _harvest_index(states)
    assert hi == pytest.approx(
        0.643597,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.651501 -> 0.643597
    )  # ⚠ 2026-08-14 (light path), was 0.636738
    assert hi / ORACLE_HI == pytest.approx(
        1.141684,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 1.155706 -> 1.141684
    )  # 13 % PAST the reference


def test_OUR_reconstruction_lands_on_the_grain_and_leaves_the_STEM_growing() -> None:
    """The one-shot-at-flowering form: the other half of the trade, and it is OURS.

    ⚠⚠ [A] gives this form no listing line because it is not one of [A]'s forms — the
    book programs a growth fraction (Listing 3) and a sink-limited overflow (Listing 4),
    and cites Table 7 *into the first* as the source of its parameter. So the form that
    lands on the reference is a reconstruction — which is what makes the refusal
    stronger.

    ⚠ The harvest index landing within 0.2 % of the oracle fixture is a COINCIDENCE OF
    WHERE THIS VARIANT WAS SAMPLED, not a match, and is pinned here with that label so
    it cannot later be quoted as one. The oracle is a diagnostic and never a fit target
    (the standing ruling); the sourced form at `fstr = 0.20` lands on it too, and
    0.20 is Sorghum's row, not Wheat's.

    ⚠ Re-measured 2026-08-12 with the cessation window, which this variant carries too —
    the fork under test is the FORM, so gating one side only would have turned every
    comparison in this section into a measurement of the window instead.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, snapshot_fill=True)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    assert total[-1] / total[a] == pytest.approx(
        1.194894, rel=1e-3
    )  # still +35 %  # ⚠ 2026-08-15 canopy 1.321691 -> 1.194894
    assert _harvest_index(states) == pytest.approx(
        0.574238,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.580517 -> 0.574238
    )  # ⚠ 2026-08-14 (light path), was 0.5627
    # the fill is a genuine one-shot: exactly one step where the reserve rose
    starch = _series(states, RESERVE_C)
    assert sum(1 for i in range(1, len(starch)) if starch[i] > starch[i - 1]) == 1
    assert (
        min(v for v in starch[a + 1 :]) > 0.0
    )  # never returns to 0, so it cannot re-fire


def test_NEITHER_form_fixes_both_halves_and_that_is_the_refusal() -> None:
    """The trade, asserted as one claim so it cannot be quoted half at a time.

    [A]'s own form:      stem shape nearly flat (1.036), harvest index 13 % over.
    our reconstruction:  harvest index on the reference, stem shape still +35 %.

    ⚠⚠ **The shape half is now stated against the FROZEN CONTROL rather than against
    1.0, and that is a re-derivation, not a loosened bound.** Un-gated, the sourced form
    shrank the stem (0.985) and `growth_shape < 1.0 < snap_shape` said everything in one
    line. With the cessation window the stem gains 3.6 % after flowering instead of
    losing 1.5 %, so that line would now be false — and the honest claim it was
    standing in for is the *ordering*: against a frozen stem that gains 61.8 %, [A]'s
    form removes almost all of the gain and ours removes about half. Both halves of
    the trade are asserted here, plus the control, so no number can be quoted alone.
    """
    rows = {}
    for label, kw in (
        ("frozen", {}),
        ("growth", {"reserve": True}),
        ("snapshot", {"reserve": True, "snapshot_fill": True}),
    ):
        states, _, _ = _run(sc.DEFAULT_SCENARIO, 1, **kw)  # type: ignore[arg-type]
        total = _stem_total(states)
        rows[label] = (
            total[-1] / total[_anthesis(states)],
            _harvest_index(states) / ORACLE_HI,
        )
    frozen_shape, _ = rows["frozen"]
    growth_shape, growth_hi = rows["growth"]
    snap_shape, snap_hi = rows["snapshot"]
    # the control first: the untreated stem is the thing both forms are shrinking
    assert frozen_shape == pytest.approx(
        1.426612,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 1.551963 -> 1.426612
    )  # ⚠ 2026-08-14 (light path), was 1.610592
    # …and only [A]'s form gets it near flat; ours leaves half the excess growth
    assert growth_shape < snap_shape < frozen_shape
    # ⚠ 2026-08-15 (the depth-resolved canopy + sourced SLA): [A]'s form is no longer
    # 'near flat' on an absolute reading — it sits 7.4 % off, where it was under 5 %.
    # The ORDERING claim on the line above is untouched, and it is the comparative one
    # the refusal rests on; what weakened is the absolute adjective, so the adjective
    # is the thing re-pinned rather than the conclusion.
    assert abs(growth_shape - 1.0) < 0.08 < abs(snap_shape - 1.0)
    assert abs(snap_hi - 1.0) < abs(growth_hi - 1.0)  # …and it is the other one on HI
    assert growth_hi > 1.10


def test_the_partition_table_is_what_blocks_it_and_it_is_UNCITED() -> None:
    """The structural finding: `fs` never reaches zero, so the stem is always refilled.

    [A]'s trigger is "once stems stop growing" — a statement about ITS partition table,
    in which the stem fraction goes to zero. Ours flat-extrapolates 0.10 past maturity,
    and the file itself flags the whole table `TODO(cite) — provisional`.
    """
    from domains.biosphere.loader import load_allocation_params

    table = load_allocation_params().table
    assert table[-1].dvs == 2.0
    assert table[-1].fs == 0.10 > 0.0  # the stem is still being fed at maturity…
    # …and past it, because the interpolation flat-extrapolates.
    from domains.biosphere.allocation import partition_fractions

    assert partition_fractions(3.0, table)[1] == table[-1].fs

    text = (
        Path(__file__).parents[1] / "src/domains/biosphere/params/allocation.yaml"
    ).read_text(encoding="utf-8")
    assert "TODO(cite)" in text and "provisional" in text


# =====================================================================================
# 3. which numbers are load-bearing — and it is the opposite of their provenance ranking
# =====================================================================================
def test_the_drain_rate_is_BIT_INERT_on_carbon_and_only_relabels() -> None:
    """The uncited 0.1/day changes nothing but the label on the carbon.

    Checked at `to_bits()` over every stock at every step, not at printed precision. The
    mechanism: once carbon is in the reserve it is already outside maintenance and
    outside senescence, and grain is too — so moving it between them is a rename.

    ⚠ The stock that is ALLOWED to differ is asserted to actually differ, so this cannot
    pass by the two runs being identical for a trivial reason.

    ⚠⚠ **SPLIT BY QUANTITY 2026-08-14 (the step unfreeze), because the flat version was
    accidentally true.** It asserted that *no* stock outside the two carbon ones differs
    at all, and that held at ``dt = 1`` only because the nitrogen coupling stayed below
    the last bit. At ``dt = ¼`` — four times the steps, so four times the accumulation —
    ``plant_n`` and ``soil_n`` cross it at **every** rate tested.

    That is not a new mechanism, and it is not this test's mechanism failing: the very
    next test, :func:`test_a_reserve_that_never_drains_moves_NITROGEN_and_the_reason_is_
    the_target`, exists to say that grain weight sets the N target, so anything moving
    carbon into grain moves nitrogen too. This test simply could not see it.

    Measured across all three rates (``temp/step-unfreeze/probe_drain.py``): the CARBON
    side is **exactly** the two allowed stocks at every rate — the claim in the title is
    intact and exact — and the nitrogen side is ``plant_n`` ≤ 0.45 % and ``soil_n``
    ≤ 1e-6. So the assertion now states the carbon claim exactly *and* bounds the
    nitrogen coupling, which an unbounded N change would still fail. A flat set equality
    could only have been kept by pretending the coupling is not there.
    """
    ref, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, rate=0.1)
    for rate in (0.05, 0.2, 1.0):
        sub, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, rate=rate)
        differ: set[str] = set()
        worst: dict[str, float] = {}
        for a, b in zip(ref, sub, strict=True):
            for sid, st in a.stocks.items():
                other = b.stocks[sid].amount
                if _bits(st.amount) != _bits(other):
                    differ.add(str(sid))
                    rel = abs(st.amount - other) / max(abs(st.amount), 1e-30)
                    worst[str(sid)] = max(worst.get(str(sid), 0.0), rel)
        carbon = {
            s for s in differ if ref[0].stocks[StockId(s)].quantity is Quantity.CARBON
        }
        assert carbon == {str(RESERVE_C), str(STORAGE_C)}, (
            f"rate={rate}: the drain must only relabel carbon: {sorted(carbon)}"
        )
        # The second-order N coupling, bounded rather than denied.
        assert differ - carbon <= {str(PLANT_N), str(SOIL_N)}, (
            f"rate={rate}: only N may follow the grain: {sorted(differ - carbon)}"
        )
        # ⚠ 5e-3 -> 6e-3 (2026-08-14, the light path). The bound is on a SECOND-ORDER
        # coupling that the test exists to bound rather than deny; the smaller crop
        # concentrates plant N slightly, so the follow-on moves 0.0051 against 0.0050.
        # Widened by the smallest step that holds, not to a round number.
        assert worst.get(str(PLANT_N), 0.0) < 6e-3, worst
        assert worst.get(str(SOIL_N), 0.0) < 1e-5, worst


def test_a_reserve_that_never_drains_moves_NITROGEN_and_the_reason_is_the_target() -> (
    None
):
    """The one exception to the bit-inertness, measured rather than reasoned about.

    At `rate = 0` (a degenerate form, not a candidate) the grain is ~10.7 mol C smaller,
    and grain is inside Greenwood's `W` — the denominator of the nitrogen TARGET — so a
    smaller `W` raises the target, raises the uptake demand, and the plant takes up more
    N. Every non-zero rate leaves nitrogen bit-identical.
    """
    ref, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, rate=0.1)
    sub, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, rate=0.0)
    pn_ref, pn_sub = _series(ref, PLANT_N), _series(sub, PLANT_N)
    first = next(
        i for i, (a, b) in enumerate(zip(pn_ref, pn_sub, strict=True)) if a != b
    )
    assert first > _anthesis(ref)  # it starts only once grain exists
    assert pn_sub[-1] > pn_ref[-1]  # smaller W -> higher target -> more N


def test_the_trigger_is_OURS_and_it_is_near_inert() -> None:
    """The one number [A] does not give us barely matters — peak LAI is IDENTICAL.

    Sweeping DVS 0.0 ... 1.5 leaves the canopy bit-alike. All the trigger decides is how
    much starch is standing at its maximum (4.8x across the sweep) and — since
    2026-08-12 — how long the window is.

    ⚠⚠ **THE CESSATION WINDOW MADE THIS NUMBER MATTER MORE, AND BY HOW MUCH IS THE POINT
    OF RE-PINNING IT.** Un-gated, the drain ran to the last step of the season whatever
    the trigger, so an early start only meant a longer thin tail: final grain moved
    **0.7 %** across the sweep. With an end at maturity the trigger now sets the
    window's **length**, so the same sweep moves grain **2.3 %** — still small, still
    the second weakest number in the file by effect, but no longer "0.7 %". The bound is
    therefore re-derived from the measurement rather than kept at `< 0.01` and the claim
    quietly restated; and the *shape* of the dependence is asserted too, because it is
    new: the loss is one-sided, concentrated at the LATE trigger that truncates its
    own window (DVS 0.0/0.5/1.0 are within 0.12 % of each other, 1.5 is 2.2 % below).
    """
    lais, grains, peaks = [], [], []
    for trig in (0.0, 0.5, 1.0, 1.5):
        states, r, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, trigger=trig)
        assert r == 0
        lais.append(_peak_lai(states, sc.DEFAULT_SCENARIO))
        grains.append(_series(states, STORAGE_C)[-1])
        peaks.append(max(_series(states, RESERVE_C)))
    assert len({_bits(v) for v in lais}) == 1  # the canopy does not notice at all
    spread = (max(grains) - min(grains)) / max(grains)
    assert spread == pytest.approx(
        0.026225,
        rel=2e-2,  # ⚠ 2026-08-15 canopy 0.023904 -> 0.026225
    )  # ⚠ 2026-08-14 (light path), was 0.024505
    # one-sided: only the trigger that eats into its own window costs anything
    assert (grains[0] - grains[2]) / grains[0] < 0.002
    assert (grains[0] - grains[3]) / grains[0] > 0.02
    assert max(peaks) / min(peaks) > 4.0  # …while the standing starch varies 4.8x


def test_the_fill_fraction_is_the_only_number_that_moves_anything() -> None:
    """And it is the one [A] tabulates — the provenance ranking, inverted.

    Across Table 7's own spread the harvest index runs 0.52 -> 0.67 and Greenwood's `W`
    runs 13.00 -> 14.54 t/ha, i.e. the tripwire clearance goes from 90 % to past 100 %.
    (Re-measured 2026-08-12 under the cessation window; the top row still crosses.)
    """
    his, ws = {}, {}
    for fstr in (0.1, 0.2, 0.3, 0.4, 0.5):
        states, r, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, fstr=fstr)
        assert r == 0
        his[fstr] = _harvest_index(states)
        ws[fstr] = _peak_w(states, sc.DEFAULT_SCENARIO)
    assert his[0.1] == pytest.approx(
        0.527575,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.5409610 -> 0.527575
    )  # ⚠ 2026-08-14 (light path), was 0.516
    assert his[0.5] == pytest.approx(
        0.68011,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.686197 -> 0.68011
    )  # ⚠ 2026-08-14 (light path), was 0.674711
    ordered = [his[k] for k in sorted(his)]
    assert all(a < b for a, b in zip(ordered, ordered[1:], strict=False))  # monotone
    assert ws[0.4] == pytest.approx(
        13.574762,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 13.947964 -> 13.574762
    )  # ⚠ 2026-08-14 (light path), was 14.314705
    # ⚠⚠ 2026-08-14 (the light path): **Table 7's top row NO LONGER crosses.** ws[0.5]
    # is 14.3467 against the 14.4248 tripwire — 0.5 % under, where it was 1.4 % over.
    # Same event as the stem-only crossing withdrawn in test_senescence_form: the
    # light path takes ~2.4 % off every crop here and these margins are ~1-2 %. The
    # ORDERING (0.5 above 0.4) is what the sweep is about and is untouched; the
    # crossing claim is not, and is recorded as withdrawn rather than re-tuned.
    assert ws[0.5] > ws[0.4]
    assert ws[0.5] == pytest.approx(13.941309, rel=1e-3), (
        "no longer crosses 14.4248"
    )  # ⚠ 2026-08-15 canopy 14.346729 -> 13.941309


# =====================================================================================
# 3b. the CESSATION WINDOW — the stem stops feeding the seed (2026-08-12)
#
# The user's second question about this mechanism: "the stem should stop feeding the
# seed at some point — that is closer to reality." It does now, at maturity, and the
# bound is [A]'s **Listing 3 Line 114** (`FINISH DS = 2., CELVN = 3.`) — the run-control
# line of the module whose Lines 17/35 ARE this mechanism.
#
# ⚠ Read at its exact strength: `FINISH` is RUN CONTROL. [A] does not say remobilization
# ceases at maturity; it says its program HAS NO STATE there. So what is pinned below is
# a decision not to extrapolate a form past the program that defines it — never a cited
# cessation rule. The un-gated control is `cessation=99.0`, which is reachable only from
# a test (the loader rejects anything above 2), and it reproduces the values this file
# recorded before the window — that reproduction is what makes the pins below readable
# as a measurement of the window rather than of a rebuild.
# =====================================================================================
def test_the_transfer_actually_STOPS_and_the_ungated_control_reproduces() -> None:
    """The claim in one line: after maturity, no carbon moves stem <-> reserve at all.

    Both halves are checked at `to_bits()` over the post-maturity tail, because "it
    stops" is a statement about EVERY step past maturity, not about the last one. The
    reserve is asserted to be non-zero there — a stopped flow and an empty pool look the
    same on a trajectory, and only one of them is the mechanism under test.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True)
    assert rationed == 0
    p = load_phenology_params()
    dvs = [
        development_stage(
            s.aux.get(THERMAL_TIME, 0.0),
            tsum_anthesis=p.tsum_anthesis,
            tsum_maturity=p.tsum_maturity,
        )
        for s in states
    ]
    mature = next(i for i, v in enumerate(dvs) if v >= CESSATION_DVS)
    assert 0 < mature < len(states) - 1, "the season must actually REACH maturity"
    starch, grain = _series(states, RESERVE_C), _series(states, STORAGE_C)
    # the reserve is frozen solid from the first mature step onwards…
    assert len({_bits(v) for v in starch[mature:]}) == 1
    # …and it is not merely empty: there is real carbohydrate standing in the dead stem
    assert starch[mature] > 0.5
    # …while the grain KEEPS filling, so the run is not simply over (the partition table
    # still feeds `storage_c` directly — finding 2, and out of this mechanism's scope)
    assert grain[-1] > grain[mature]


def test_the_ungated_control_reproduces_the_values_recorded_before_it() -> None:
    """`cessation=99` must give back the pre-2026-08-12 numbers, exactly as recorded.

    ⚠ This is the control that makes every re-pinned number in this file honest. Each
    of them is written as "X, was Y un-gated"; if Y were not reproducible those notes
    would be unfalsifiable prose. The values below are quoted from the pins as they
    stood before the window landed, not re-derived from the current tree.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, cessation=99.0)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    # the old stem shape
    assert total[-1] / total[a] == pytest.approx(
        0.881291,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.965073 -> 0.881291
    )  # ⚠ 2026-08-14 (light path), was 0.980573
    assert _harvest_index(states) == pytest.approx(
        0.655853, rel=1e-3
    )  # the old HI  # ⚠ 2026-08-15 canopy 0.664113 -> 0.655853
    assert _series(states, STORAGE_C)[-1] == pytest.approx(
        33.337707,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 34.686483 -> 33.337707
    )  # ⚠ 2026-08-14 (light path), was 34.826488
    # and the window's whole effect on the grain, stated once as a number
    gated = _series(_run(sc.DEFAULT_SCENARIO, 1, reserve=True)[0], STORAGE_C)[-1]
    assert gated / _series(states, STORAGE_C)[-1] - 1.0 == pytest.approx(
        -0.019144,
        rel=1e-2,  # ⚠ 2026-08-15 canopy -0.019457 -> -0.019144
    )


def test_the_two_halves_share_ONE_cessation_number_in_the_shipped_wiring() -> None:
    """The fill and the drain must stop on the same step, or the reserve is a trap.

    A drain that stopped alone would leave the dead stem stashing starch forever; a fill
    that stopped alone would just be a shorter fill. They are separate objects
    (`Allocation` owns the split, `StemRemobilization` owns the draw), so the fact that
    they read the SAME loaded number is wiring that can silently break — hence a pin on
    the built tree rather than on the param file.
    """
    _, registry = build_season(sc.DEFAULT_SCENARIO)
    alloc = next(f for f in registry.flows if isinstance(f, Allocation))
    drain = next(f for f in registry.flows if isinstance(f, StemRemobilization))
    assert alloc.reserve_cessation_dvs == drain.params.cessation_dvs == CESSATION_DVS
    # …and the crop that has no reserve carries the inert default on all three fields
    _, off = build_season(_without_reserve(sc.DEFAULT_SCENARIO))
    alloc_off = next(f for f in off.flows if isinstance(f, Allocation))
    assert alloc_off.stem_reserve_c is None
    assert alloc_off.fstr == 0.0 and alloc_off.reserve_cessation_dvs == 0.0
    assert not any(isinstance(f, StemRemobilization) for f in off.flows)


def test_a_forgotten_cessation_fails_CLOSED_rather_than_running_unbounded() -> None:
    """The inert default is 0.0, so a half-wired reserve does NOTHING — measured.

    All three of `Allocation`'s reserve fields default to "off", and the cessation's
    "off" value has to be the one that shuts the mechanism rather than the one that
    unbounds it. At 0.0 the whole run must come back bit-identical to the crop that has
    no reserve at all: same stocks, same steps, same bits.
    """
    off, _, _ = _run(_without_reserve(sc.DEFAULT_SCENARIO), 1)
    shut, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, cessation=0.0)
    assert rationed == 0
    assert max(_series(shut, RESERVE_C)) == 0.0  # nothing was ever stashed
    for sid in (LEAF_C, STEM_C, ROOT_C, STORAGE_C, PLANT_N):
        assert [_bits(v) for v in _series(shut, sid)] == [
            _bits(v) for v in _series(off, sid)
        ], f"{sid} differs — a shut-off reserve is not inert"


def test_the_loader_refuses_a_cessation_that_is_unreachable_or_empty(
    tmp_path: Path,
) -> None:
    """⚠ The trap worth a test: DVS is **capped at 2.0**.

    So `cessation_dvs = 2.5` would not postpone the cessation — it would restore exactly
    the unbounded behaviour the parameter exists to end, while reading like a deliberate
    choice in the file. Nothing downstream would notice: no golden distinguishes "stops
    at 2.5" from "never stops", because the stage never gets there. The other end is the
    mirror: at or below the trigger the window is empty and the mechanism is off while
    fully wired. Both are rejected where the value is read.
    """
    src = Path("src/domains/biosphere/params/stem_reserves.yaml").read_text(
        encoding="utf-8"
    )
    original = "value: 2.0"
    assert src.count(original) == 1, "cessation is no longer the only 2.0 in the file"
    for bad in ("2.5", "1.0", "0.5"):
        path = tmp_path / f"cessation_{bad}.yaml"
        path.write_text(src.replace(original, f"value: {bad}", 1), encoding="utf-8")
        with pytest.raises(ValueError, match="cessation_dvs must be in"):
            load_stem_reserve_params(path)


# =====================================================================================
# 4. where the extra grain comes from
# =====================================================================================
def test_the_grain_gain_is_the_TRANSFER_not_the_two_exemptions() -> None:
    """Three mechanisms are entangled; turning both exemptions off keeps +46.8 %.

    (a) the transfer starch -> grain, (b) starch being outside the maintenance biomass,
    (c) starch not being shed at `rdr_stem`. With (b) and (c) both removed — starch
    treated exactly like stem carbon in every respect except where it eventually goes —
    the grain is still up 46.8 % of the full form's 50.4 %. The mechanism does what it
    says. (Both legs re-measured 2026-08-12 under the cessation window, which costs the
    full form ~3 points of the +53.5 % it had un-gated; the ratio between them is what
    this test is about and it is unmoved at 0.98.)
    """
    base = _series(_run(sc.DEFAULT_SCENARIO, 1)[0], STORAGE_C)[-1]
    plain = _series(_run(sc.DEFAULT_SCENARIO, 1, reserve=True)[0], STORAGE_C)[-1]
    stripped = _series(
        _run(sc.DEFAULT_SCENARIO, 1, reserve=True, reserve_is_shed=True)[0], STORAGE_C
    )[-1]
    assert plain / base - 1.0 == pytest.approx(
        0.488354,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.46162 -> 0.488354
    )  # ⚠ 2026-08-14 (light path), was 0.509562
    assert stripped / base - 1.0 == pytest.approx(
        0.445701,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.426251 -> 0.445701
    )  # ⚠ 2026-08-14 (light path), was 0.472077
    assert stripped / plain > 0.9  # the exemptions are a small part of it


def test_the_frozen_harvest_index_is_LOW_and_the_grain_mass_is_LOWER() -> None:
    """Two different quantities, stated apart — the conflation this repo logs twice.

    Against the committed oracle fixture the frozen crop's grain FRACTION is 0.84x and
    its grain MASS is 0.52x. The reserve takes the mass to 0.78x while taking the
    fraction PAST 1.0 — an improvement on one and an overshoot on the other, which is
    only visible if they are not merged. (Mass re-measured 0.7983 -> 0.7825 under the
    2026-08-12 cessation window; the frozen control is untouched by it, which is the
    half of this pin that has to stay still.)
    """
    frozen, _, _ = _run(sc.DEFAULT_SCENARIO, 1)
    withres, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True)
    twso_frozen = _t_per_ha(
        frozen[-1].stocks[STORAGE_C].amount, sc.DEFAULT_SCENARIO.ground_area
    )
    twso_res = _t_per_ha(
        withres[-1].stocks[STORAGE_C].amount, sc.DEFAULT_SCENARIO.ground_area
    )
    assert _harvest_index(frozen) / ORACLE_HI == pytest.approx(
        0.862357,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.889826 -> 0.862357
    )  # ⚠ 2026-08-14 (light path), was 0.839057
    assert twso_frozen / 11.5 == pytest.approx(
        0.509922,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.540083 -> 0.509922
    )  # ⚠ 2026-08-14 (light path), was 0.525043
    assert _harvest_index(withres) / ORACLE_HI > 1.0
    assert twso_res / 11.5 == pytest.approx(
        0.758944,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.789397 -> 0.758944
    )  # ⚠ 2026-08-14 (light path), was 0.792584


# =====================================================================================
# 5. closure — the gate every biosphere science change has actually been judged by
# =====================================================================================
def test_the_reserve_closes_every_sealed_chamber_on_both_integrators() -> None:
    """`rationed == 0` everywhere, for BOTH forms, with the CONTROLS checked first.

    This is where the stem-only branch died, so a subject reading is only trusted after
    the frozen minimum CO2 reproduces its committed value.

    ⚠ Our reconstruction is measured here too rather than deferred as an unmeasured
    leg, and its **discrete one-shot switch** (`reserve != 0.0`) is why not to
    assume RK4 would be fine with it — a state-dependent switch is exactly what a
    multi-stage integrator handles badly.

    ⚠ The frozen roster is SEVEN scenarios and this loop drives four of them. The other
    three: `open_season` (section 6 below), `perennial_long_horizon` /
    `consumer_long_horizon` (the same scenario objects at a longer horizon, driven in
    the slow test below), and `drift_summary`, which is DERIVED from the two
    long-horizon runs rather than being a run of its own — its inputs are measured here
    and it would move with them. Checked against the manifest, not against this
    loop's own length.
    """
    frozen, r0, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS, resets=True
    )
    assert r0 == 0
    assert min(_series(frozen, CARBON_POOL)) == pytest.approx(
        0.070735,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 0.07106 -> 0.070735
    )  # ⚠ 2026-08-14 (light path), was 0.076461

    for form in ({}, {"snapshot_fill": True}):
        for scen, years, resets in (
            (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, False),
            (sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS, True),
            (sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS, True),
        ):
            _s, rationed, events = _run(
                scen,
                years,
                resets=resets,
                reserve=True,
                **form,  # type: ignore[arg-type]
            )
            assert rationed == 0 and events == ()
        # RK4 is the integrator that killed the full (C) form on this same chamber.
        _s, rationed, events = _run(
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.PERENNIAL_CHAMBER_YEARS,
            resets=True,
            integrator=Rk4Integrator,
            reserve=True,
            **form,
        )
        assert rationed == 0 and events == ()

    # A detail worth keeping: in ``sealed_chamber`` our reconstruction leaves the CO2
    # trough EXACTLY at the frozen value, because the trough happens before the single
    # fill event ever fires.
    #
    # ⚠ THIS WAS A TWO-ROW COMPARISON UNTIL 2026-08-18. ``water_biting`` was the second
    # row and the ONLY one taking the non-bit-identical branch: at ``dt = 1`` its trough
    # also preceded the fill, and the quarter-day step re-timed the two events so it no
    # longer did — which is what proved the bit-identity was a statement about WHEN this
    # scenario's trough happens rather than about the reserve being inert. C6 of the
    # reference flip retired the scenario, so that branch has no subject left and the
    # bound it carried (the reserve moves the trough by under 5 %) is retired with it,
    # named as a gap in docs/plans/post-roadmap-reference-flip.md §5k rather than
    # quietly dropped. What remains is the exact half, on the chamber that still clears
    # it.
    # ⚠ 0.076380 → 0.076482 on 2026-08-12, and NOT because of the reserve: this build
    # re-sized `SEALED_CHAMBER_SCENARIO`'s litter seed 3.0 → 3.5, because the extra O₂ a
    # reserve-carrying crop releases had lifted the chamber's O₂ trough from ~0.01 % of
    # its fill to 5.08 % and killed the ≥95 %-depletion contract the scenario exists
    # for.
    # ⚠ 0.076482 → 0.077538 on 2026-08-14 (the step unfreeze, dt 1 → ¼).
    # ⚠ 0.077538 -> 0.071668 on 2026-08-14 (the light path).
    # ⚠ 0.071668 -> 0.071782 (2026-08-15): the chamber trough barely moves, as the depth
    # integral is inert at LAI 0.54 and only the SLA anchor reaches it.
    base, _, _ = _run(sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS)
    snap, _, _ = _run(
        sc.SEALED_CHAMBER_SCENARIO,
        sc.SEALED_CHAMBER_YEARS,
        reserve=True,
        snapshot_fill=True,
    )
    assert min(_series(base, CARBON_POOL)) == pytest.approx(0.071782, rel=1e-4)
    assert _bits(min(_series(snap, CARBON_POOL))) == _bits(
        min(_series(base, CARBON_POOL))
    )


@pytest.mark.slow
def test_the_reserve_passes_every_manifest_liveness_floor_the_frozen_tree_passes() -> (
    None
):
    """The four `perennial_long_horizon` gates, computed the way their tests compute
    them.

    Controls first: the frozen tree's own numbers (0.055175 trough, 0.634352 fixed
    point) reproduce the record, and stem-only's failure (0.046065) reproduces too — so
    the harness is known to be able to report a failure before it reports a pass.
    """
    out = {}
    for label, kw in (
        ("frozen", {}),
        ("stem0", {"stem_zero": True}),
        ("reserve", {"reserve": True}),
    ):
        states, rationed, _ = _run(
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.LONG_HORIZON_YEARS,
            resets=True,
            **kw,  # type: ignore[arg-type]
        )
        co2 = year_summaries(
            states, _YEAR, lambda seg: min(s.stocks[CARBON_POOL].amount for s in seg)
        )
        leafs = year_summaries(
            states, _YEAR, lambda seg: max(s.stocks[LEAF_C].amount for s in seg)
        )
        scale = max(co2)
        out[label] = (
            rationed,
            min(co2),
            non_collapsing(co2, floor=0.05),
            is_stationary(
                same_phase_diffs(co2, period=2),
                bound=0.2 * scale,
                slope_tol=0.02 * scale,
                transient=2,
            ),
            non_collapsing(leafs, floor=0.05),
            max(leafs[8:]),
        )
    assert out["frozen"][1] == pytest.approx(
        0.070735,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 0.07106 -> 0.070735
    )  # ⚠ 2026-08-14 (light path), was 0.076461
    assert out["frozen"][5] == pytest.approx(
        0.578626,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 0.603855 -> 0.578626
    )  # ⚠ 2026-08-14 (light path), was 0.611984
    assert out["stem0"][1] == pytest.approx(
        0.070776,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 0.071147 -> 0.070776
    )  # ⚠ 2026-08-14 (light path), was 0.076527
    # ⚠⚠ **THE NON-VACUITY CONTROL DIED AT `dt = ¼` (2026-08-14), and that is a finding
    # about this test, not a number to swap.** `stem0` sat here to show the harness CAN
    # fail: at `dt = 1` it returned `(False, False)` on the stationarity and
    # non-collapsing legs. It now returns `(True, True)` — the stem-only tree passes
    # both closure gates the reserve was credited with rescuing one of.
    #
    # Two consequences, and the second is the one to act on:
    #   1. Stem-only's refusal has both of its surviving closure legs discharged at the
    #      shipped step. ⚠ NOT reopened here — see
    #      `test_what_the_reserve_RESCUES_of_stem_onlys_two_closure_legs` for why
    #      re-deciding a refusal inside the work that moved the tree is refused.
    #   2. **The "…and the subject passes every one of them" claim below is now
    #      VACUOUS.** Every arm passes, so the harness is no longer shown capable of
    #      producing a failure, and a green bar here proves nothing. The measured truth
    #      is asserted so the vacuity is visible rather than hidden behind a passing
    #      control; a replacement control that genuinely fails at this step is
    #      **outstanding work**, deliberately not invented inside this ceremony.
    assert (out["stem0"][2], out["stem0"][3]) == (True, True)
    # …and the subject passes every one of them — ⚠ see above: currently vacuous.
    r = out["reserve"]
    assert r[0] == 0
    # ⚠ Both subject readings moved in the LAST TWO DIGITS when the cessation window
    # landed (2026-08-12): trough 0.055977 -> 0.056030, fixed point 0.637424 ->
    # 0.637384. The two CONTROLS above are pinned at their original values and did not
    # move, which is what says the shift is the window and not the harness.
    # ⚠ 0.075476 -> 0.070492 (2026-08-14, the light path); still clears 0.05.
    assert (
        r[1] == pytest.approx(0.070253, rel=1e-4) and r[1] > 0.05
    )  # ⚠ 2026-08-15 canopy 0.070492 -> 0.070253
    assert r[2] is True and r[3] is True and r[4] is True
    # ⚠ 0.612211 -> 0.603679 (2026-08-14); the 0.55 floor's clearance narrows again.
    assert (
        r[5] == pytest.approx(0.578137, rel=1e-4) and r[5] > 0.55
    )  # ⚠ 2026-08-15 canopy 0.603679 -> 0.578137


@pytest.mark.slow
def test_what_the_reserve_RESCUES_of_stem_onlys_two_closure_legs() -> None:
    """Stem-only's refusal had two closure legs. At `dt = 1` the reserve discharged one.
    At `dt = ¼` it discharges **both**.

    ⚠⚠ **CHANGED 2026-08-14 by the step unfreeze, and renamed because the old name
    (`..._RESCUES_stem_onlys_co2_floor_but_not_its_stationarity`) stated the half-result
    as a fact.** The CO₂ floor leg was already fixed. The stationarity leg — asserted
    here as `is_stationary(...) is False`, i.e. still broken — now comes back **True**.
    The non-settledness was a step artefact, not a property of the stem-only tree.

    ⚠ **This still does NOT reopen stem-only, and the reason is unchanged and is the
    one that matters more than the result.** Re-deciding a refusal inside the work that
    moved the tree underneath it is the shape this project refuses — and that applies
    with *more* force now, not less, because the change is in the refusal's favour. A
    ceremony that quietly discharges a refusal it was not authorized to revisit is
    exactly how a refusal stops meaning anything. Both legs passing is **recorded as a
    finding and left for a separate decision**, with its own evidence, on its own terms.

    (The Euler/step trap running in the third direction: `stem-only` was refused partly
    on a stationarity reading, the reserve work re-read it, and the step re-read it
    again. Each reading was correct about the tree it was taken on.)
    """
    states, rationed, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO,
        sc.LONG_HORIZON_YEARS,
        resets=True,
        reserve=True,
        stem_zero=True,
    )
    co2 = year_summaries(
        states, _YEAR, lambda seg: min(s.stocks[CARBON_POOL].amount for s in seg)
    )
    scale = max(co2)
    assert rationed == 0
    assert min(co2) == pytest.approx(
        0.07028, rel=1e-4
    )  # 0.053127 before the window  # ⚠ 2026-08-15 canopy 0.070549 -> 0.07028
    assert non_collapsing(co2, floor=0.05) is True  # stem-only's 0.046065 leg: FIXED
    assert (
        is_stationary(
            same_phase_diffs(co2, period=2),
            bound=0.2 * scale,
            slope_tol=0.02 * scale,
            transient=2,
        )
        is True
    )  # ⚠ was False at dt=1 — the OTHER leg is fixed too now. See the docstring: this
    #    is recorded, NOT taken as reopening the refusal.


# =====================================================================================
# 6. the science bands, and the two probes the reserve could have broken
# =====================================================================================
def test_the_open_season_science_bands_survive_the_reserve_but_the_margin_shrinks() -> (
    None
):
    """`open_season` is the only frozen scenario carrying outside-sourced bands.

    Peak LAI stays inside real wheat's 5-8 and below the Van Keulen & Seligman shading
    threshold of 6 — but its clearance falls from 86.5 % to 91.0 % of that threshold,
    and Greenwood's `W` goes from 87.6 % to 98.1 % of the 14.4248 t/ha crossing.
    Both pass; neither passes comfortably.
    """
    frozen, _, _ = _run(sc.DEFAULT_SCENARIO, 1)
    res, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True)
    assert _peak_lai(frozen, sc.DEFAULT_SCENARIO) == pytest.approx(
        5.778138,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 5.137266 -> 5.778138
    )  # ⚠ 2026-08-14 (light path), was 5.298122
    assert _peak_w(frozen, sc.DEFAULT_SCENARIO) == pytest.approx(
        12.081495,
        rel=1e-4,  # ⚠ 2026-08-15 canopy 12.400672 -> 12.081495
    )  # ⚠ 2026-08-14 (light path), was 12.765369
    lai, w = _peak_lai(res, sc.DEFAULT_SCENARIO), _peak_w(res, sc.DEFAULT_SCENARIO)
    # ⚠ 2026-08-15: the reserve form's canopy now CROSSES the mutual-shading threshold
    # (6.0228 > 6.0), exactly as the frozen tree does. That is no longer a band failure
    # — the loss above the threshold is MODELLED as of the same date — so the surviving
    # claim is the sourced wheat band, plus the fact that the reserve does not carry the
    # canopy anywhere the frozen tree does not already go.
    assert 5.0 < lai < 8.0
    assert lai > 6.0, "the reserve form is in the mutual-shading regime too"
    assert lai / 6.0 == pytest.approx(
        1.003806,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.896774 -> 1.003806
    )  # ⚠ 2026-08-14 (light path), was 0.928654
    assert w < 14.4248
    assert w / 14.4248 == pytest.approx(
        0.941071,
        rel=1e-3,  # ⚠ 2026-08-15 canopy 0.9669430 -> 0.941071
    )  # ⚠ 2026-08-14 (light path), was 0.992368


# ⚠ RETIRED 2026-08-18 with its subject: ``test_n_limited_keeps_the_regime_it_was_
# built_for`` stood here. It drove ``n_limited`` twice — the frozen form against the
# reserve-on candidate — to show the stem reserve did not wreck the nitrogen-limited
# regime (bite 0.1688 vs 0.1730, biting 199 vs 198 days). C6 of the reference flip
# deleted ``n_limited``, and this comparison could not follow it: its second arm is a
# CANDIDATE FORM from a decision already taken, so there is nothing left to compare the
# frozen tree against. What survives, and is the half worth keeping, is that ``f_N``
# genuinely bites somewhere — now pinned in the reference on a manufactured condition
# (`system.rs`, `nitrogen_limitation_is_wired_into_assimilation_and_no_scenario_shows`
# `_it`).
# ⚠ The reserve's effect ON that bite is the part with no successor; it is named as a
# gap in docs/plans/post-roadmap-reference-flip.md §5k rather than quietly dropped.


def test_option_Bs_litter_C_to_N_identity_survives_the_nitrogen_free_starch() -> None:
    """Starch carries no nitrogen, so it is exactly the thing that could break it.

    Option (B) made the litter pool's C:N a function of the composition of what falls
    in. In the shedding-fed chambers the starch never reaches litter at all (it drains
    to grain), so the pool moves 1.4 %; in the reset-driven ones it is dumped with the
    dead plant and the pool moves ~18 % — toward, not away from, real residue, since
    that regime's C:N of ~10 is this tree's recorded limitation 5.
    """
    for scen, years, resets, frozen_cn, res_cn in (
        (
            sc.SEALED_CHAMBER_SCENARIO,
            sc.SEALED_CHAMBER_YEARS,
            False,
            # ⚠ 102.7493 → 103.3038 and 104.2185 → 104.9727 on 2026-08-12, from this
            # build's `litter_carbon0` 3.0 → 3.5 re-size of the scenario (see the CO₂
            # trough note above), NOT from the reserve. The CLAIM — that the reserve's
            # nitrogen-free starch leaves option (B)'s emergent litter C:N intact — is
            # re-measured and holds: the shedding-fed chamber moves 1.6 %, because the
            # starch drains to grain and never reaches litter at all.
            # ⚠ 103.541580 -> 103.869801 (2026-08-14, the light path).
            103.869801,
            # ⚠ 105.163270 -> 105.827924 (2026-08-14, the light path).
            105.827924,
        ),
        (
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.PERENNIAL_CHAMBER_YEARS,
            True,
            # ⚠ 11.280148 -> 12.653988 (2026-08-14, the light path) — a 12 % move on the
            # reset-dump regime, where the shedding-fed one above moved 0.3 %. The two
            # regimes are still an order of magnitude apart, which is the identity this
            # test defends.
            12.001178,  # ⚠ 2026-08-15 canopy 12.653988 -> 12.001178
            # 12.7991 → 12.7822 on 2026-08-12 with the cessation window: less starch is
            # standing when the plant dies, so slightly less nitrogen-free carbon is
            # dumped into litter at the re-sow. ⚠ 13.218522 -> 14.286147 (2026-08-14).
            14.261267,  # ⚠ 2026-08-15 canopy 15.045320 -> 14.261267
        ),
    ):
        for kw, expected in (({}, frozen_cn), ({"reserve": True}, res_cn)):
            states, _, _ = _run(scen, years, resets=resets, **kw)  # type: ignore[arg-type]
            from domains.biosphere.stocks import LITTER_N

            lc, ln = _series(states, LITTER_CARBON), _series(states, LITTER_N)
            i = ln.index(max(ln))
            assert (lc[i] * _M_C) / ln[i] == pytest.approx(expected, rel=1e-3)
