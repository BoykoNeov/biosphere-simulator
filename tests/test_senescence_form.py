"""The senescence FORM gap — option (C), DIAGNOSED AND PRICED, deliberately NOT taken.

Post-roadmap work — ``docs/plans/post-roadmap-nitrogen-cycle-form.md``, "THE (C)
DIAGNOSIS". The gap is real and first-hand: [A] Penning de Vries et al. 1989 §3.2.6
(p. 95) gives the relative death rate as a **function of development stage**, zero
before anthesis; our ``rdr_leaf/stem/root`` are bare constants applied from DVS 0, i.e.
the degenerate case of the form we cite, non-zero exactly where the source is zero.

This module exists because **the reason we did NOT take it is a measurement**, and a
measurement kept only in prose rots. What is pinned, and why each one is here:

1. **The two source tables, and the arithmetic that authenticates them.** The book
   carries two ``LLVT`` definitions — the crop file §3.2.6 cites (Listing 5, rice IR36,
   p. 212, peak **0.012**/day) and an exercise answer (T10, p. 113, peak **0.15**/day).
   Our own record quoted the exercise for five citation rounds. T10 states the loss
   pattern its digits reproduce, so the read is checkable by arithmetic rather than by
   eye — the round-5 discipline.
2. **⚠ THE STRUCTURAL FINDING: the flat ``rdr_leaf`` is standing in for canopy
   regulation the tree does not have.** Taking the primary's form takes ``open_season``
   peak LAI from 5.19 to 16.4, against real wheat's ~5-8 — and *both* tables give the
   same peak, because both are zero below DS 1.0. So it is not a reading question, and
   (C) cannot be built until something else regulates the canopy on the way up.
3. **The frozen flat form is nearer the primary's own stated outcome than the primary's
   own table is** (38.5 % leaf lost by season end vs the stated 40-60 % band; Listing 5
   gives 30.0 %). It does not vindicate the flat form — it says the constant was
   implicitly sized to the right *integrated* loss with the *timing* entirely wrong.
4. **The tripwire fires, and which table you read decides whether it does.**
   ``test_nitrogen_form.py`` pins ``open_season``'s 12.633 t/ha against the 14.4248 t/ha
   Greenwood crossing as an explicit tripwire for "any calibration that grows the crop
   ~15 %". (C) is that calibration: 18.678 t/ha, and ``f_N`` bites for the first time in
   a frozen scenario. Under the *exercise* table it would not have (13.879).
5. **Euler reads clean and RK4 does not.** ``perennial`` reports ``rationed == 0`` under
   Euler and hard-errors under RK4 — increment 1's record repeating exactly, and the
   reason a Euler-only screen of a carbon change is not evidence.

The candidate flows live in this module, not in ``src/``: nothing here is built, and
``git diff src/simcore`` stays empty. They are the same ones the read-only probes ran.
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path

import pytest

from domains.biosphere import scenario as sc
from domains.biosphere.allocation import Senescence, senescence_flux
from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import (
    load_canopy_params,
    load_nitrogen_params,
    load_phenology_params,
    load_senescence_params,
)
from domains.biosphere.mineralization import (
    NitrogenSenescence,
    nitrogen_shedding_flux,
)
from domains.biosphere.nitrogen import NitrogenParams, nitrogen_stress_factor
from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.season import (
    LEAF_C,
    PLANT_N,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.registry import Registry
from simcore.state import State

_WEATHER = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_M_C = 0.012011  # kg C / mol C
_CARBON_FRACTION = 0.45  # kg C / kg DM
_CROSSING_T_HA = 14.4248  # Greenwood target meets n_critical (test_nitrogen_form.py)

# --- the source tables, read off the page images (extraction garbles the numerals) ----
# [A] Listing 5, "Crop data for rice (variety IR36)", p. 212 — the table §3.2.6 cites.
LISTING5_LEAF = ((0.0, 0.0), (1.0, 0.0), (1.3, 0.007), (1.8, 0.012), (2.5, 0.012))
LISTING5_ROOT = ((0.0, 0.0), (1.0, 0.0), (1.3, 0.011), (1.8, 0.010), (2.5, 0.010))
# p. 95: "except for their reserves, stems do not lose weight" — and Listing 5 carries
# no stem death function at all.
LISTING5_STEM = ((0.0, 0.0),)
# [A] T10, p. 113 — an EXERCISE ANSWER, not the crop file. This is the table our own
# record quoted for five citation rounds.
T10_LEAF = ((0.0, 0.0), (1.0, 0.0), (1.5, 0.03), (2.0, 0.15))


def _weather(years: int = 1) -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"] * years


def _rdr_at(dvs: float, table: tuple[tuple[float, float], ...]) -> float:
    """Piecewise-linear with flat extrapolation — the ``partition_fractions`` idiom."""
    if dvs <= table[0][0]:
        return table[0][1]
    if dvs >= table[-1][0]:
        return table[-1][1]
    for (lo_d, lo_v), (hi_d, hi_v) in zip(table, table[1:], strict=False):
        if lo_d <= dvs <= hi_d:
            return lo_v + ((dvs - lo_d) / (hi_d - lo_d)) * (hi_v - lo_v)
    raise AssertionError(f"no bracketing knot for dvs={dvs!r}")  # pragma: no cover


@dataclass(frozen=True)
class _DvsSenescence:
    """Candidate CARBON senescence at a DVS-keyed relative death rate. TEST-ONLY."""

    id: FlowId
    priority: int
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    litter_sink: StockId
    pheno: PhenologyParams
    leaf_table: tuple[tuple[float, float], ...]
    stem_table: tuple[tuple[float, float], ...]
    root_table: tuple[tuple[float, float], ...]

    def dvs(self, snapshot: State) -> float:
        return development_stage(
            snapshot.aux.get("thermal_time", 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        del env
        d = self.dvs(snapshot)
        legs = []
        total = 0.0
        for stock, table in (
            (self.leaf_c, self.leaf_table),
            (self.stem_c, self.stem_table),
            (self.root_c, self.root_table),
        ):
            lost = (
                senescence_flux(
                    snapshot.stocks[stock].amount,
                    relative_death_rate=_rdr_at(d, table),
                )
                * dt
            )
            legs.append(Leg(stock, -lost))
            total += lost
        legs.append(Leg(self.litter_sink, total))
        return FlowResult(legs=tuple(legs))


@dataclass(frozen=True)
class _DvsNitrogenSenescence:
    """The N leg, recomputing the SAME DVS-keyed carbon flux. TEST-ONLY.

    Present because ``mineralization.py`` recomputes the senescence flux off the FLAT
    params: a (C) that keyed only ``allocation.Senescence`` would silently keep shedding
    N at the old rates. That is the (A) recomputation-drift hazard one flow over, and it
    is a design requirement a build would inherit.
    """

    id: FlowId
    priority: int
    plant_n: StockId
    litter_n: StockId
    leaf_c: StockId
    stem_c: StockId
    root_c: StockId
    carbon: _DvsSenescence
    nitro_params: NitrogenParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        del env
        stocks = snapshot.stocks
        leaf = stocks[self.leaf_c].amount
        stem = stocks[self.stem_c].amount
        root = stocks[self.root_c].amount
        d = self.carbon.dvs(snapshot)
        shed_carbon = (
            senescence_flux(
                leaf, relative_death_rate=_rdr_at(d, self.carbon.leaf_table)
            )
            + senescence_flux(
                stem, relative_death_rate=_rdr_at(d, self.carbon.stem_table)
            )
            + senescence_flux(
                root, relative_death_rate=_rdr_at(d, self.carbon.root_table)
            )
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
        return FlowResult(legs=(Leg(self.plant_n, -shed), Leg(self.litter_n, shed)))


_Knots = tuple[tuple[float, float], ...]


def _candidate(
    registry: Registry, state: State, *, leaf: _Knots, stem: _Knots, root: _Knots
) -> Registry:
    pheno = load_phenology_params()
    old = next(f for f in registry.flows if isinstance(f, Senescence))
    new = _DvsSenescence(
        old.id,
        old.priority,
        leaf_c=old.leaf_c,
        stem_c=old.stem_c,
        root_c=old.root_c,
        litter_sink=old.litter_sink,
        pheno=pheno,
        leaf_table=leaf,
        stem_table=stem,
        root_table=root,
    )
    flows = []
    for f in registry.flows:
        if str(f.id) == "biosphere.senescence":
            flows.append(new)
        elif isinstance(f, NitrogenSenescence):
            flows.append(
                _DvsNitrogenSenescence(
                    f.id,
                    f.priority,
                    plant_n=f.plant_n,
                    litter_n=f.litter_n,
                    leaf_c=f.leaf_c,
                    stem_c=f.stem_c,
                    root_c=f.root_c,
                    carbon=new,
                    nitro_params=load_nitrogen_params(),
                )
            )
        else:
            flows.append(f)
    # ⚠ The aux processes MUST be carried over. Dropping them freezes ``thermal_time``
    # at 0, so DVS never advances and a DVS-keyed flow silently becomes a zero flow —
    # probe B2's first bug, which a clean control did not catch.
    return Registry(flows, state.stocks, registry.aux_processes)


def _run(
    scenario,
    years: int,
    *,
    tables: dict[str, _Knots] | None = None,
    resets: bool = False,
    integrator: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
):
    w = _weather(years)
    state, registry = build_season(scenario)
    if tables is not None:
        registry = _candidate(registry, state, **tables)
    resolver = weather_resolver(w, scenario)
    if resets:
        return run_perennial(
            integrator(registry),
            state,
            scenario,
            resolver,
            1.0,
            len(w),
            year=len(_weather()),
        )
    return run_season(integrator(registry), state, resolver, 1.0, len(w))


_LISTING5 = {"leaf": LISTING5_LEAF, "stem": LISTING5_STEM, "root": LISTING5_ROOT}
_T10 = {"leaf": T10_LEAF, "stem": LISTING5_STEM, "root": LISTING5_ROOT}


def _peak_lai(states, scenario) -> tuple[float, float]:
    cp = load_canopy_params()
    lais = [
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=cp.sla_per_mol_c,
            ground_area=scenario.ground_area,
        )
        for s in states
    ]
    return max(lais), lais[-1]


def _t_per_ha(mol_c: float, ground_area: float) -> float:
    return ((mol_c * _M_C / _CARBON_FRACTION) / ground_area) * 10.0


# --- 1. the source tables, and the arithmetic that authenticates them -----------------
def test_the_exercise_table_reproduces_the_loss_pattern_it_states() -> None:
    """T10's digits are checkable against T10's own prose — the round-5 discipline.

    "The reproductive period lasts about 30 days. In the first 15 days, loss is 20 %, in
    the second 15 days 75 % of what remained." A quote check verifies characters; only
    arithmetic verifies numbers, and here BOTH halves must hold simultaneously off one
    set of digits.
    """
    # DS 1.0 -> 1.5: the rate ramps 0 -> 0.03, mean 0.015/day over 15 days.
    first = 1.0 - math.exp(-_rdr_at(1.25, T10_LEAF) * 15.0)
    # DS 1.5 -> 2.0: ramps 0.03 -> 0.15, mean 0.09/day over 15 days.
    second = 1.0 - math.exp(-_rdr_at(1.75, T10_LEAF) * 15.0)
    assert 0.19 < first < 0.21, first  # "loss is 20 %"
    assert 0.73 < second < 0.77, second  # "75 % of what remained"


def test_the_two_source_tables_disagree_by_an_order_but_agree_on_the_form() -> None:
    """⚠ Our own record quoted the EXERCISE table for five citation rounds.

    §3.2.6 p. 95 cites Listing 5 by name. A locus error survives inside a
    correctly-attributed quote, so the two tables are held side by side here rather than
    one of them being written down as "the source's function".
    """
    assert _rdr_at(2.0, T10_LEAF) / _rdr_at(2.0, LISTING5_LEAF) == pytest.approx(12.5)
    # The FORM claim is what survives both readings, and it is the load-bearing half.
    for table in (LISTING5_LEAF, LISTING5_ROOT, T10_LEAF):
        for dvs in (0.0, 0.25, 0.5, 0.75, 0.999, 1.0):
            assert _rdr_at(dvs, table) == 0.0, (table, dvs)
    # ...and the frozen tree is non-zero over exactly that range.
    sen = load_senescence_params()
    assert sen.rdr_leaf > 0.0 and sen.rdr_stem > 0.0 and sen.rdr_root > 0.0


def test_rdr_root_is_the_closest_of_the_three_to_its_source() -> None:
    """Do not quote ``rdr_leaf``'s "runs fast" finding as covering all three.

    Listing 5's root plateau is 0.010-0.011/day; ours is 0.01/day flat. The root gap is
    the FORM only. The leaf gap is form AND value; the stem has no counterpart at all.
    """
    sen = load_senescence_params()
    plateau = _rdr_at(2.0, LISTING5_ROOT)
    assert abs(sen.rdr_root - plateau) / plateau < 0.10
    assert sen.rdr_leaf / _rdr_at(2.0, LISTING5_LEAF) == pytest.approx(1.667, rel=1e-3)
    assert LISTING5_STEM == ((0.0, 0.0),)  # the source has no stem death function


# --- 2/3. the canopy: what the flat rate has actually been doing ----------------------
def test_frozen_open_season_canopy_is_physical() -> None:
    """The baseline half of the finding, so the comparison below has a floor."""
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1)
    assert rationed == 0
    peak, _final = _peak_lai(states, sc.DEFAULT_SCENARIO)
    assert 5.0 < peak < 8.0, peak  # real wheat peaks at ~5-8


@pytest.mark.parametrize("tables", [_LISTING5, _T10], ids=["listing5", "exercise_t10"])
def test_the_primarys_form_takes_the_canopy_unphysical_on_either_table(tables) -> None:
    """⚠ THE STRUCTURAL FINDING. The flat ``rdr_leaf`` is the canopy's only regulator.

    BOTH tables give the same peak, because both are zero below DS 1.0 and the peak is
    reached at anthesis — so this is not the locus question above, it is the half of the
    form every reading of the source agrees on. The tree has no self-shading leaf death,
    no leaf-age cohorts and no SLA aging; remove the flat rate and nothing holds the
    canopy down. That is why (C) is blocked on a missing science rather than on effort.
    """
    states, _rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=tables)
    peak, _final = _peak_lai(states, sc.DEFAULT_SCENARIO)
    assert peak > 15.0, peak  # vs real wheat's ~5-8


def test_frozen_form_is_nearer_the_primarys_stated_outcome_than_its_table() -> None:
    """[A] p. 95: the descriptive form "usually results in a loss of 40-60 % of leaf
    area at harvest time".

    ⚠ This does NOT vindicate the flat form as a form. It says the constant was
    implicitly sized to roughly the right INTEGRATED loss while getting the TIMING
    entirely wrong — shedding hardest where the source says exactly zero. Recorded
    because a finding that cuts against one's own recommendation is the one that goes
    missing.
    """

    def lost(tables) -> float:
        states, _r, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=tables)
        peak, final = _peak_lai(states, sc.DEFAULT_SCENARIO)
        return 1.0 - final / peak

    frozen, listing5, t10 = lost(None), lost(_LISTING5), lost(_T10)
    assert 0.35 < frozen < 0.45, frozen  # 38.5 % — just under the band
    assert 0.25 < listing5 < 0.35, listing5  # 30.0 % — below it
    assert t10 > 0.9, t10  # 97.9 % — the crop is stripped
    assert abs(frozen - 0.50) < abs(listing5 - 0.50)  # nearer the band's midpoint


# --- 4. the tripwire ------------------------------------------------------------------
def test_the_dvs_form_crosses_the_greenwood_tripwire_and_f_n_bites() -> None:
    """The first ``f_N`` bite in a FROZEN scenario, and it is measured, not inferred.

    ``test_nitrogen_form.py`` laid the 14.4248 t/ha crossing down as a tripwire for "any
    calibration that grows the open-field crop ~15 %". (C) is that calibration. ``f_N``
    is a live feedback (``carbon_budget`` multiplies gross assimilation by
    ``f_water * f_N``), so the peak below is already self-consistent.

    ⚠ The bite is 0.5 % over 6 of 306 steps. The tripwire fires; nitrogen does not
    thereby become load-bearing, and the two must not be blurred.
    """
    npar = load_nitrogen_params()
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=_LISTING5)
    assert rationed == 0
    peak_w = max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount,
            sc.DEFAULT_SCENARIO.ground_area,
        )
        for s in states
    )
    assert peak_w > _CROSSING_T_HA, peak_w
    assert 18.0 < peak_w < 19.5, peak_w
    fns = [
        nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount,
            n_residual_per_mol_c=npar.n_residual_per_mol_c,
            n_critical_per_mol_c=npar.n_critical_per_mol_c,
        )
        for s in states
    ]
    assert min(fns) < 1.0
    assert sum(1 for f in fns if f < 1.0) < 20  # a handful of steps, not a regime


def test_the_exercise_table_would_have_reported_no_tripwire() -> None:
    """The locus error was worth ~4 % of clearance either side of a threshold.

    Under T10 the crop peaks at 96.2 % of the crossing, so the wrong table reports "the
    tripwire does not fire" — a qualitatively different conclusion from the same source.
    """
    states, _r, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=_T10)
    peak_w = max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount,
            sc.DEFAULT_SCENARIO.ground_area,
        )
        for s in states
    )
    assert peak_w < _CROSSING_T_HA, peak_w
    assert peak_w / _CROSSING_T_HA > 0.95, peak_w


# --- 5. closure: Euler reads clean, RK4 does not --------------------------------------
def test_euler_reports_no_rationing_and_that_is_the_trap() -> None:
    """Half of the pair. Alone this reads as "closure survives (C)"; it does not."""
    _states, rationed, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO,
        sc.PERENNIAL_CHAMBER_YEARS,
        tables=_LISTING5,
        resets=True,
    )
    assert rationed == 0


def test_rk4_hard_errors_on_the_perennial_chamber_under_the_dvs_form() -> None:
    """Increment 1's record repeating exactly: "rationed under Euler, hard-errored under
    RK4".

    A needed scale is a hard error under a higher-order scheme (positivity must come
    from the kinetics), so this is not a near-miss — and ``test_decade_closure_held``
    runs RK4 over the full 15 years.
    """
    with pytest.raises(Exception, match="over-draw"):
        _run(
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.PERENNIAL_CHAMBER_YEARS,
            tables=_LISTING5,
            resets=True,
            integrator=Rk4Integrator,
        )


def test_the_exercise_table_breaks_the_re_sow_outright() -> None:
    """T10 never fills grain, so closure fails at the sow rather than at the draw."""
    with pytest.raises(ValueError, match="seed bank too small"):
        _run(
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.PERENNIAL_CHAMBER_YEARS,
            tables=_T10,
            resets=True,
        )
