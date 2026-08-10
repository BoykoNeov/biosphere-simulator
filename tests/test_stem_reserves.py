"""Stem-reserve remobilization — DIAGNOSED AND PRICED (2026-08-10), NOT built.

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

**Because the real defect is one level up: the DVS-keyed partition table keeps sending
10 % of every day's growth to the stem right through grain fill.** [A]'s own trigger
for remobilization is "once stems stop growing" — which is not merely unfireable here,
it is *a statement about [A]'s partition table*, in which the stem fraction reaches
zero. Ours is `fs = 0.10` at DVS 2.0 with flat extrapolation, and `allocation.yaml`
carries the whole table as `TODO(cite) — provisional`. A reserve can move carbon out of
the stem; it cannot stop the allocation putting it back. (And [A]'s *other* programmed
form is out of reach for a sibling reason: an overflow fill needs a grain sink that can
be full, and ours is `fo * DMI` with no capacity at all.)

That is the (C) / canopy-regulator shape a third time — a real, sourced mechanism
blocked by a *different* missing piece — with one difference that matters: this
mechanism is **not inert**. It passes every gate the frozen tree passes, on both
integrators, on the whole manifest roster and on `sealed_station`, and it moves grain by
half. So the refusal is "the science underneath it is uncited", not "it does not work".

⚠ **The oracle harvest index is NOT a target and is not used as one here.** Two sampled
variants land within a few parts in ten thousand of it. That is a coincidence of where
the sweep was sampled, it is pinned as such, and this project's standing ruling is that
the oracle is a diagnostic and never a fit target.

Both forms close every sealed chamber on both integrators — the constructed one was
measured too rather than left as an unmeasured leg, and its discrete one-shot switch was
the specific reason not to assume RK4 would be fine with it.

Read-only: no `src/`, param, golden or manifest change. The candidate flows live in
this module.
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
    load_nitrogen_params,
    load_phenology_params,
)
from domains.biosphere.mineralization import NitrogenSenescence
from domains.biosphere.nitrogen import nitrogen_stress_factor
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
from domains.biosphere.stocks import pool_stock
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

# The committed oracle fixture, quoted in `allocation.py`'s own docstring: "TWSO ~ 11.5
# of TAGP ~ 20.4 t/ha (grain is ~half the biomass)". A DIAGNOSTIC, never a fit target.
ORACLE_HI = 11.5 / 20.4


def _weather(years: int = 1) -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"] * years


_YEAR = len(_weather())


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
    """

    inner: Allocation
    reserve: StockId
    fstr: float

    @property
    def id(self) -> FlowId:
        return self.inner.id

    @property
    def priority(self) -> int:
        return self.inner.priority

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        res = self.inner.evaluate(snapshot, env, dt)
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

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        dvs = development_stage(
            snapshot.aux.get(self.thermal_time_aux, 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        if dvs < self.trigger_dvs:
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
) -> tuple[State, Registry]:
    state, registry = build_season(scenario)
    flows: list[object] = list(registry.flows)
    if reserve:
        stocks = dict(state.stocks)
        stocks[RESERVE_C] = pool_stock(
            RESERVE_C, PLANTS, Quantity.CARBON, canonical_unit(Quantity.CARBON), 0.0
        )
        pheno = load_phenology_params()
        if not snapshot_fill:
            flows = [
                _GrowthFractionFill(f, RESERVE_C, fstr)
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
        if n > 0 and n % _YEAR == 0:
            out = annual_reset(current, scenario)
            if RESERVE_C in out.stocks and out.stocks[RESERVE_C].amount != 0.0:
                st = dict(out.stocks)
                held = st[RESERVE_C].amount
                st[RESERVE_C] = dataclasses.replace(st[RESERVE_C], amount=0.0)
                st[LITTER_CARBON] = dataclasses.replace(
                    st[LITTER_CARBON], amount=st[LITTER_CARBON].amount + held
                )
                out = dataclasses.replace(out, stocks=st)
            return out
        return current

    return run_season(
        integrator(registry),
        state,
        resolver,
        1.0,
        len(w),
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
    every flow against a real mid-season snapshot, not read off the type declarations.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    assert a == 251
    assert total[-1] / total[a] == pytest.approx(1.6184, rel=1e-3)
    # …and it is still gaining within days of the end of the run.
    assert total.index(max(total)) >= len(total) - 15

    _state, registry = build_season(sc.DEFAULT_SCENARIO)
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

    # ⚠ Measured over the WHOLE trajectory, not at one snapshot. A single mid-season
    # step reports only TWO doors, because maintenance draws on the stem only on days
    # when assimilation does not cover it — a CONDITIONAL door. Reading the count off
    # one step would have made "three doors" look wrong; reading it off the type
    # declarations would have made it look like five. Neither is the quantity.
    resolver = weather_resolver(_weather(1), sc.DEFAULT_SCENARIO)
    doors: set[str] = set()
    always_open: set[str] = set()
    both_legs = 0
    for i, snap in enumerate(states[:-1]):
        env = resolver.bind(snap, 1.0)
        here = set()
        for f in registry.flows:
            legs = f.evaluate(snap, env, 1.0).legs
            stem_leg = next((leg for leg in legs if leg.stock == STEM_C), None)
            if stem_leg is None:
                continue
            here.add(str(f.id))
            grain_leg = next((leg for leg in legs if leg.stock == STORAGE_C), None)
            if grain_leg is not None:
                # Allocation names both stocks — but it DEPOSITS into each from the
                # atmosphere. No flow withdraws from the stem and deposits into grain.
                assert stem_leg.amount >= 0.0 and grain_leg.amount >= 0.0
                both_legs += 1
        doors |= here
        always_open = here if i == 0 else (always_open & here)
    assert doors == {
        "biosphere.allocation",
        "biosphere.maintenance_respiration",
        "biosphere.senescence",
    }
    assert always_open == {"biosphere.allocation", "biosphere.senescence"}
    assert both_legs > 0  # the "deposits into both" case really does occur


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
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    starch = _series(states, RESERVE_C)
    assert starch[a] / total[a] == pytest.approx(0.4201, rel=1e-3)
    assert total[-1] / total[a] == pytest.approx(0.9853, rel=1e-3)  # it STOPS gaining
    hi = _harvest_index(states)
    assert hi == pytest.approx(0.6487, rel=1e-3)
    assert hi / ORACLE_HI == pytest.approx(1.1508, rel=1e-3)  # 15 % PAST the reference


def test_OUR_reconstruction_lands_on_the_grain_and_leaves_the_STEM_growing() -> None:
    """The one-shot-at-flowering form: the other half of the trade, and it is OURS.

    ⚠⚠ [A] gives this form no listing line because it is not one of [A]'s forms — the
    book programs a growth fraction (Listing 3) and a sink-limited overflow (Listing 4),
    and cites Table 7 *into the first* as the source of its parameter. So the form that
    lands on the reference is a reconstruction — which is what makes the refusal
    stronger.

    ⚠ The harvest index landing within 0.04 % of the oracle fixture is a COINCIDENCE OF
    WHERE THIS VARIANT WAS SAMPLED, not a match, and is pinned here with that label so
    it cannot later be quoted as one. The oracle is a diagnostic and never a fit target
    (the standing ruling); the sourced form at `fstr = 0.20` lands on it too, and
    0.20 is Sorghum's row, not Wheat's.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, snapshot_fill=True)
    assert rationed == 0
    total = _stem_total(states)
    a = _anthesis(states)
    assert total[-1] / total[a] == pytest.approx(1.3494, rel=1e-3)  # still +35 %
    assert _harvest_index(states) == pytest.approx(0.5635, rel=1e-3)
    # the fill is a genuine one-shot: exactly one step where the reserve rose
    starch = _series(states, RESERVE_C)
    assert sum(1 for i in range(1, len(starch)) if starch[i] > starch[i - 1]) == 1
    assert (
        min(v for v in starch[a + 1 :]) > 0.0
    )  # never returns to 0, so it cannot re-fire


def test_NEITHER_form_fixes_both_halves_and_that_is_the_refusal() -> None:
    """The trade, asserted as one claim so it cannot be quoted half at a time.

    [A]'s own form:      stem shape good (0.985), harvest index 15 % over.
    our reconstruction:  harvest index on the reference, stem shape still +35 %.
    """
    rows = {}
    for label, kw in (
        ("growth", {"reserve": True}),
        ("snapshot", {"reserve": True, "snapshot_fill": True}),
    ):
        states, _, _ = _run(sc.DEFAULT_SCENARIO, 1, **kw)  # type: ignore[arg-type]
        total = _stem_total(states)
        rows[label] = (
            total[-1] / total[_anthesis(states)],
            _harvest_index(states) / ORACLE_HI,
        )
    growth_shape, growth_hi = rows["growth"]
    snap_shape, snap_hi = rows["snapshot"]
    assert growth_shape < 1.0 < snap_shape  # only one of them stops the stem growing
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
    """
    ref, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, rate=0.1)
    for rate in (0.05, 0.2, 1.0):
        sub, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, rate=rate)
        differ: set[str] = set()
        for a, b in zip(ref, sub, strict=True):
            for sid, st in a.stocks.items():
                if _bits(st.amount) != _bits(b.stocks[sid].amount):
                    differ.add(str(sid))
        assert differ == {str(RESERVE_C), str(STORAGE_C)}, (
            f"rate={rate}: {sorted(differ)}"
        )


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

    Sweeping DVS 0.0 ... 1.5 moves final grain by 0.7 % and leaves the canopy bit-alike.
    All the trigger decides is how much starch is standing at its maximum.
    """
    lais, grains, peaks = [], [], []
    for trig in (0.0, 0.5, 1.0, 1.5):
        states, r, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, trigger=trig)
        assert r == 0
        lais.append(_peak_lai(states, sc.DEFAULT_SCENARIO))
        grains.append(_series(states, STORAGE_C)[-1])
        peaks.append(max(_series(states, RESERVE_C)))
    assert len({_bits(v) for v in lais}) == 1  # the canopy does not notice at all
    assert (max(grains) - min(grains)) / max(grains) < 0.01
    assert max(peaks) / min(peaks) > 4.0  # …while the standing starch varies 4.8x


def test_the_fill_fraction_is_the_only_number_that_moves_anything() -> None:
    """And it is the one [A] tabulates — the provenance ranking, inverted.

    Across Table 7's own spread the harvest index runs 0.52 -> 0.69 and Greenwood's `W`
    runs 13.00 -> 14.55 t/ha, i.e. the tripwire clearance goes from 90 % to past 100 %.
    """
    his, ws = {}, {}
    for fstr in (0.1, 0.2, 0.3, 0.4, 0.5):
        states, r, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True, fstr=fstr)
        assert r == 0
        his[fstr] = _harvest_index(states)
        ws[fstr] = _peak_w(states, sc.DEFAULT_SCENARIO)
    assert his[0.1] == pytest.approx(0.5193, rel=1e-3)
    assert his[0.5] == pytest.approx(0.6895, rel=1e-3)
    ordered = [his[k] for k in sorted(his)]
    assert all(a < b for a, b in zip(ordered, ordered[1:], strict=False))  # monotone
    assert ws[0.4] == pytest.approx(14.1516, rel=1e-3)
    assert (
        ws[0.5] > 14.4248 > ws[0.4]
    )  # Table 7's TOP row crosses the Greenwood tripwire


# =====================================================================================
# 4. where the extra grain comes from
# =====================================================================================
def test_the_grain_gain_is_the_TRANSFER_not_the_two_exemptions() -> None:
    """Three mechanisms are entangled; turning both exemptions off keeps +46.6 %.

    (a) the transfer starch -> grain, (b) starch being outside the maintenance biomass,
    (c) starch not being shed at `rdr_stem`. With (b) and (c) both removed — starch
    treated exactly like stem carbon in every respect except where it eventually goes —
    the grain is still up 46.6 % of the frozen 53.5 %. The mechanism does what it says.
    """
    base = _series(_run(sc.DEFAULT_SCENARIO, 1)[0], STORAGE_C)[-1]
    plain = _series(_run(sc.DEFAULT_SCENARIO, 1, reserve=True)[0], STORAGE_C)[-1]
    stripped = _series(
        _run(sc.DEFAULT_SCENARIO, 1, reserve=True, reserve_is_shed=True)[0], STORAGE_C
    )[-1]
    assert plain / base - 1.0 == pytest.approx(0.5346, rel=1e-3)
    assert stripped / base - 1.0 == pytest.approx(0.4957, rel=1e-3)
    assert stripped / plain > 0.9  # the exemptions are a small part of it


def test_the_frozen_harvest_index_is_LOW_and_the_grain_mass_is_LOWER() -> None:
    """Two different quantities, stated apart — the conflation this repo logs twice.

    Against the committed oracle fixture the frozen crop's grain FRACTION is 0.84x and
    its grain MASS is 0.52x. The reserve takes the mass to 0.80x while taking the
    fraction PAST 1.0 — an improvement on one and an overshoot on the other, which is
    only visible if they are not merged.
    """
    frozen, _, _ = _run(sc.DEFAULT_SCENARIO, 1)
    withres, _, _ = _run(sc.DEFAULT_SCENARIO, 1, reserve=True)
    twso_frozen = _t_per_ha(
        frozen[-1].stocks[STORAGE_C].amount, sc.DEFAULT_SCENARIO.ground_area
    )
    twso_res = _t_per_ha(
        withres[-1].stocks[STORAGE_C].amount, sc.DEFAULT_SCENARIO.ground_area
    )
    assert _harvest_index(frozen) / ORACLE_HI == pytest.approx(0.8400, rel=1e-3)
    assert twso_frozen / 11.5 == pytest.approx(0.5202, rel=1e-3)
    assert _harvest_index(withres) / ORACLE_HI > 1.0
    assert twso_res / 11.5 == pytest.approx(0.7983, rel=1e-3)


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
    assert min(_series(frozen, CARBON_POOL)) == pytest.approx(0.055175, rel=1e-4)

    for form in ({}, {"snapshot_fill": True}):
        for scen, years, resets in (
            (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, False),
            (sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS, False),
            (sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS, True),
            (sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS, True),
        ):
            _s, rationed, events = _run(
                scen, years, resets=resets, reserve=True, **form
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

    # A detail worth keeping: in the two shedding-fed chambers our reconstruction leaves
    # the CO2 trough EXACTLY at the frozen value, because the trough happens before the
    # single fill event ever fires.
    for scen, years, frozen_min in (
        (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, 0.076380),
        (sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS, 0.085006),
    ):
        base, _, _ = _run(scen, years)
        snap, _, _ = _run(scen, years, reserve=True, snapshot_fill=True)
        assert min(_series(base, CARBON_POOL)) == pytest.approx(frozen_min, rel=1e-4)
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
    assert out["frozen"][1] == pytest.approx(0.055175, rel=1e-4)
    assert out["frozen"][5] == pytest.approx(0.634352, rel=1e-4)
    assert out["stem0"][1] == pytest.approx(0.046065, rel=1e-4)
    assert out["stem0"][2] is False and out["stem0"][3] is False  # the harness CAN fail
    # …and the subject passes every one of them.
    r = out["reserve"]
    assert r[0] == 0
    assert r[1] == pytest.approx(0.055977, rel=1e-4) and r[1] > 0.05
    assert r[2] is True and r[3] is True and r[4] is True
    assert r[5] == pytest.approx(0.637424, rel=1e-4) and r[5] > 0.55


@pytest.mark.slow
def test_the_reserve_RESCUES_stem_onlys_co2_floor_but_not_its_stationarity() -> None:
    """One of stem-only's two surviving closure legs is discharged; the other is not.

    ⚠ This does NOT reopen stem-only. Its refusal had two legs, the reserve fixes the
    level and not the settledness, and re-deciding a refusal inside the work that moved
    the tree underneath it is the shape this project refuses.
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
    assert min(co2) == pytest.approx(0.053127, rel=1e-4)
    assert non_collapsing(co2, floor=0.05) is True  # stem-only's 0.046065 leg: FIXED
    assert (
        is_stationary(
            same_phase_diffs(co2, period=2),
            bound=0.2 * scale,
            slope_tol=0.02 * scale,
            transient=2,
        )
        is False
    )  # …the other leg: NOT fixed


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
    assert _peak_lai(frozen, sc.DEFAULT_SCENARIO) == pytest.approx(5.1908, rel=1e-4)
    assert _peak_w(frozen, sc.DEFAULT_SCENARIO) == pytest.approx(12.6331, rel=1e-4)
    lai, w = _peak_lai(res, sc.DEFAULT_SCENARIO), _peak_w(res, sc.DEFAULT_SCENARIO)
    assert 5.0 < lai < 8.0 and lai < 6.0
    assert lai / 6.0 == pytest.approx(0.9104, rel=1e-3)
    assert w < 14.4248
    assert w / 14.4248 == pytest.approx(0.9810, rel=1e-3)


def test_n_limited_keeps_the_regime_it_was_built_for() -> None:
    """The reserve takes carbon OUT of `f_N`'s own denominator, so this had to be
    measured.

    `n_limited` is the one place `f_N` bites, and it is not one of the frozen seven.
    Option (A) deleted the knob it is built on; this candidate does not — the bite is
    0.1789 over 186 steps against the recorded 0.1759 over 187.
    """
    frozen, _, _ = _run(sc.N_LIMITED_SCENARIO, sc.N_LIMITED_YEARS)
    res, _, _ = _run(sc.N_LIMITED_SCENARIO, sc.N_LIMITED_YEARS, reserve=True)
    p = load_nitrogen_params()

    def fn(states):
        return [
            nitrogen_stress_factor(
                s.stocks[PLANT_N].amount,
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[ROOT_C].amount,
                n_residual_per_mol_c=p.n_residual_per_mol_c,
                n_critical_per_mol_c=p.n_critical_per_mol_c,
            )
            for s in states
        ]

    a, b = fn(frozen), fn(res)
    assert min(a) == pytest.approx(0.175851, rel=1e-5)  # the recorded value
    assert sum(1 for v in a if v < 1.0) == 187
    assert min(b) == pytest.approx(0.178930, rel=1e-5)
    assert sum(1 for v in b if v < 1.0) == 186
    assert min(b) / min(a) < 1.02  # weakened by under 2 %, regime intact


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
            102.7493,
            104.2185,
        ),
        (
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.PERENNIAL_CHAMBER_YEARS,
            True,
            10.8900,
            12.7991,
        ),
    ):
        for kw, expected in (({}, frozen_cn), ({"reserve": True}, res_cn)):
            states, _, _ = _run(scen, years, resets=resets, **kw)  # type: ignore[arg-type]
            from domains.biosphere.stocks import LITTER_N

            lc, ln = _series(states, LITTER_CARBON), _series(states, LITTER_N)
            i = ln.index(max(ln))
            assert (lc[i] * _M_C) / ln[i] == pytest.approx(expected, rel=1e-3)
