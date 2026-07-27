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
   same peak, because both are zero below DS 1.0. So it is not a reading question.

   ⚠ **This section originally continued "…and (C) cannot be built until something else
   regulates the canopy on the way up", which read as "the science is missing". THAT IS
   MEASURED FALSE (2026-07-27, one day later** —
   ``docs/plans/post-roadmap-canopy-regulator.md``**).** The regulator was on the shelf
   all along: [A] **p. 101** quotes Van Keulen & Seligman (1987) at *5 %/day of leaf
   area once LAI exceeds 6*, for **wheat**, and it takes the peak from 16.40 to **6.24**
   with nothing fitted. Five citation rounds and the (C) diagnosis missed it because
   every search went to §3.2.6 (p. 95, "Senescence and death"), and V-K&S's rule lives
   six pages later in the **leaf area** section — *the (C) locus finding one level up:
   right book, right topic, wrong section, and the conclusion drawn was not "we did not
   find it" but "it does not exist".* The original wording is kept because the way it is
   wrong IS the finding. **Section 6 below has the replacement claim, which is narrower
   and survives: the regulator fixes the canopy and does NOT unblock (C).**
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
6. **THE CANOPY REGULATOR: found, sourced, measured — and it does NOT unblock (C).**
   It fires in **exactly one of eight** scenarios. Every chamber peaks at LAI
   0.068-0.632 against a threshold of 6, so on the frozen tree it is **bit-identically
   inert**, and ``perennial`` hard-errors under RK4 at ``scale_f =
   0.9527733243688737`` **with and without it, to all sixteen digits**. The chambers
   are carbon-limited by design; a mutual-shading rule regulates canopy CLOSURE, and
   their canopies never close. So exactly ONE of (C)'s three branches is discharged
   ("requires the canopy-regulation science") and discharging it did not help — closure
   is measured identical, and a fitted table stays refused. The regulator's own residual
   is a new tripwire: ``open_season`` peaks at 86 % of the threshold.

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
from domains.biosphere.canopy import CanopyParams, leaf_area_index
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

# --- the canopy regulator: [A] p. 101, quoting Van Keulen & Seligman (1987) -----------
# "Van Keulen & Seligman (1987) calculated the rate of leaf area loss in wheat
#  independently of leaf weight loss. They put it at 5 % d-1 once the leaf area exceeds
#  the value of 6 m2 m-2 to account for mutual shading."
# Read off the rendered page image, not off extraction: the same page's Table 20 comes
# out of pdftotext visibly mangled, so the digits at issue are exactly the ones the
# mechanical channel cannot be trusted for (the round-6 discipline).
# ⚠ FLAT above the threshold ("once ... exceeds"), NOT proportional to the excess. The
# SUCROS/WOFOST (LAI-LAIcrit)/LAIcrit shape is a different lineage and is not imported.
# ⚠ This is [A] QUOTING V-K&S 1987, which is NOT on the shelf: first-hand [A], not
# first-hand V-K&S, so the transmission and locus legs are unverified (Dunn 2011 is why
# that distinction is kept).
VKS_SHADE_RATE = 0.05  # 1/day
VKS_LAI_THRESHOLD = 6.0  # m2 m-2


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
    # The Van Keulen & Seligman mutual-shading regulator (finding 1 of the canopy plan).
    # ``shade_rate = 0`` recovers the bare DVS form exactly, so one class covers all
    # four cells of {frozen, Listing 5} x {no regulator, regulator} and the N leg below
    # cannot drift from the carbon leg.
    shade_rate: float = 0.0
    lai_threshold: float = VKS_LAI_THRESHOLD
    sla_per_mol_c: float = 0.0
    ground_area: float = 1.0

    def dvs(self, snapshot: State) -> float:
        return development_stage(
            snapshot.aux.get("thermal_time", 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )

    def leaf_rate(self, snapshot: State) -> float:
        """The DVS-keyed base rate plus the shading term, if the canopy has closed."""
        base = _rdr_at(self.dvs(snapshot), self.leaf_table)
        if not self.shade_rate:
            return base
        lai = leaf_area_index(
            snapshot.stocks[self.leaf_c].amount,
            sla_per_mol_c=self.sla_per_mol_c,
            ground_area=self.ground_area,
        )
        return base + self.shade_rate if lai > self.lai_threshold else base

    def rates(self, snapshot: State) -> tuple[float, float, float]:
        d = self.dvs(snapshot)
        return (
            self.leaf_rate(snapshot),
            _rdr_at(d, self.stem_table),
            _rdr_at(d, self.root_table),
        )

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        del env
        legs = []
        total = 0.0
        for stock, rate in zip(
            (self.leaf_c, self.stem_c, self.root_c),
            self.rates(snapshot),
            strict=True,
        ):
            lost = (
                senescence_flux(snapshot.stocks[stock].amount, relative_death_rate=rate)
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
        # ⚠ Through ``rates()``, NOT a second interpolation off the tables — otherwise a
        # shading term added to the carbon leg would silently not apply to the N leg.
        r_leaf, r_stem, r_root = self.carbon.rates(snapshot)
        shed_carbon = (
            senescence_flux(leaf, relative_death_rate=r_leaf)
            + senescence_flux(stem, relative_death_rate=r_stem)
            + senescence_flux(root, relative_death_rate=r_root)
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
    registry: Registry,
    state: State,
    *,
    leaf: _Knots,
    stem: _Knots,
    root: _Knots,
    shade: float = 0.0,
    ground_area: float = 1.0,
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
        shade_rate=shade,
        sla_per_mol_c=load_canopy_params().sla_per_mol_c,
        ground_area=ground_area,
    )
    flows: list[object] = []
    for f in registry.flows:
        if isinstance(f, Senescence):
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
    return Registry(flows, state.stocks, registry.aux_processes)  # type: ignore[arg-type]


def _n_legs(registry: Registry) -> int:
    return sum(1 for f in registry.flows if isinstance(f, _DvsNitrogenSenescence))


def _run(
    scenario,
    years: int,
    *,
    tables: dict[str, _Knots] | None = None,
    resets: bool = False,
    integrator: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
    shade: float = 0.0,
):
    w = _weather(years)
    state, registry = build_season(scenario)
    if tables is not None or shade:
        registry = _candidate(
            registry,
            state,
            **(tables or _FROZEN_AS_TABLES()),
            shade=shade,
            ground_area=scenario.ground_area,
        )
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


def _FROZEN_AS_TABLES() -> dict[str, _Knots]:
    """The frozen flat rates as one-knot tables — the degenerate case of the DVS form.

    Lets the regulator be run on top of the FROZEN form without a second candidate
    class. ``_rdr_at`` returns the single knot's value at every DVS, so this reproduces
    ``allocation.Senescence`` exactly (asserted below, at ``to_bits()``).
    """
    p = load_senescence_params()
    return {
        "leaf": ((0.0, p.rdr_leaf),),
        "stem": ((0.0, p.rdr_stem),),
        "root": ((0.0, p.rdr_root),),
    }


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
    # ⚠ No "which is nearer the midpoint" assertion. NONE of the three is IN the band,
    # and a nearness metric invented to rank two misses is a fitted comparison — in a
    # module whose subject is refusing fitted comparisons. The three bounds above are
    # the whole claim.


def test_the_n_leg_is_actually_swapped_and_it_is_SEALED_ONLY() -> None:
    """⚠ Otherwise ``_DvsNitrogenSenescence`` is dead code documenting a live hazard.

    Step 6b's lesson: when the argument for carrying something is "otherwise the two
    halves drift", the test that settles it is the one that RUNS it. Most pins in this
    module drive ``open_season``, which builds **no** ``NitrogenSenescence`` at all —
    ``litter_n`` is sealed-only, the structural fact (B) established. So the N branch is
    exercised by the ``perennial`` pins and nowhere else, and that is asserted here
    rather than assumed.
    """
    for scenario, expected in (
        (sc.DEFAULT_SCENARIO, 0),  # open field: no litter_n, so no shed-N flow exists
        (sc.SEALED_CHAMBER_SCENARIO, 1),
        (sc.PERENNIAL_CHAMBER_SCENARIO, 1),
    ):
        state, registry = build_season(scenario)
        assert _n_legs(_candidate(registry, state, **_LISTING5)) == expected, scenario


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


# --- 6. THE CANOPY REGULATOR (docs/plans/post-roadmap-canopy-regulator.md) ------------
# The successor this module named. It exists, it is sourced, it fixes the canopy — and
# it does NOT unblock (C). These pin the whole shape, because "we looked and there was
# nothing" is the claim that rotted here in the first place.

_ROSTER = [
    ("open_season", sc.DEFAULT_SCENARIO, 1, False),
    ("sealed_chamber", sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, False),
    (
        "perennial_chamber",
        sc.PERENNIAL_CHAMBER_SCENARIO,
        sc.PERENNIAL_CHAMBER_YEARS,
        True,
    ),
    ("consumer_chamber", sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS, True),
    ("n_limited", sc.N_LIMITED_SCENARIO, sc.N_LIMITED_YEARS, False),
    ("water_biting", sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS, False),
]


def test_the_area_rule_transfers_because_lai_is_LINEAR_in_leaf_carbon() -> None:
    """⚠ THE LICENSING STEP for a rule stated on AREA in a tree with no area state.

    V-K&S give a rate of leaf AREA loss, "independently of leaf weight loss"; P2 says
    "LAI is derived, not stored". The transfer is legitimate only because
    ``specific_leaf_area`` is a single constant with no DVS keying, so LAI is linear in
    leaf carbon and a RELATIVE area rate IS a relative carbon rate, exactly. This
    asserts the linearity rather than the constancy, because linearity is what the
    identity needs and it is checkable — and it is [A]'s OWN default, stated one
    sentence before the quote ("computed in direct relation to the rate of leaf weight
    loss, assuming that the average value of the specific leaf weight applies").

    ⚠ THE LIMITATION, pinned so it cannot be dropped: V-K&S separated area from weight
    BECAUSE specific leaf weight varies by leaf cohort — [A]'s Figure 40, on the very
    same page, plots it from ~230 to ~530 kg/ha over a season. Our single constant
    cannot express that, so we would inherit their rule under an assumption they
    explicitly declined to make.
    """
    cp = load_canopy_params()
    assert [f for f in CanopyParams.__dataclass_fields__] == [
        "sla_per_mol_c",
        "extinction_coef",
    ], "CanopyParams grew a field — is SLA still a single constant?"
    one = leaf_area_index(1.0, sla_per_mol_c=cp.sla_per_mol_c, ground_area=3.0)
    for x in (0.5, 2.0, 7.5, 1e3):
        # ⚠ Not bit-exact, and the reason is arithmetic rather than modelling:
        # ``(x*sla)/A`` and ``x*((1*sla)/A)`` associate differently and can land 1 ULP
        # apart. The IDENTITY is exact; its float evaluation is not, so the tolerance is
        # a few ULP rather than zero. Stated instead of quietly loosened.
        assert leaf_area_index(
            x, sla_per_mol_c=cp.sla_per_mol_c, ground_area=3.0
        ) == pytest.approx(x * one, rel=1e-15), x


def test_the_regulator_brings_the_primarys_form_back_into_the_realistic_band() -> None:
    """FINDING 3 — Q1 answered YES, with a sourced threshold and a sourced rate.

    C-finding 5's unphysical 16.4 lands at 6.2, inside real wheat's ~5-8. Nothing is
    fitted: both numbers come off page 101.
    """
    bare, _r, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=_LISTING5)
    reg, _r2, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=_LISTING5, shade=VKS_SHADE_RATE)
    peak_bare, _ = _peak_lai(bare, sc.DEFAULT_SCENARIO)
    peak_reg, _ = _peak_lai(reg, sc.DEFAULT_SCENARIO)
    assert peak_bare > 15.0, peak_bare  # 16.397
    assert 5.0 < peak_reg < 8.0, peak_reg  # 6.244 — the band


@pytest.mark.parametrize(
    "label,scenario,years,resets", _ROSTER, ids=[r[0] for r in _ROSTER]
)
def test_the_regulator_is_BIT_IDENTICALLY_inert_on_the_frozen_form(
    label, scenario, years, resets
) -> None:
    """⚠ FINDING 4, first half. Added to the FROZEN tree the regulator changes nothing.

    At ``to_bits()`` over every stock at every step, not "the same to three decimals" —
    the inertness claim is only worth making at that precision. It holds because the
    threshold (LAI 6) is above every frozen peak: ``open_season`` 5.191, and every
    chamber between 0.068 and 0.632, i.e. 9-88x below it.

    ⚠ **THREE runs, not two, and the third is the point.** Running the regulator on the
    frozen form needs the flat rates re-expressed as one-knot tables
    (``_FROZEN_AS_TABLES``), i.e. a RECONSTRUCTION of a frozen quantity. Comparing
    reconstruction-without-regulator against reconstruction-with-regulator would prove
    only that the regulator is inert **relative to the reconstruction** — a bug in the
    reconstruction cancels perfectly and the test stays green. That is exactly the
    hazard finding 10 names: *reconstruct a frozen quantity only to CHECK it against the
    recorded one, never to replace it.* So the real ``allocation.Senescence`` is the
    baseline, and the two legs are asserted SEPARATELY:

      1. ``frozen`` == ``reconstruction`` — the reconstruction is faithful;
      2. ``reconstruction`` == ``reconstruction + regulator`` — the regulator is inert.

    Asserted apart rather than as one conjunction so that a failure says WHICH.
    """

    def run(*, reconstruct: bool, shade: float):
        # tables=None AND shade=0 skips ``_candidate`` entirely => the real frozen flow.
        states, rationed, _ = _run(
            scenario,
            years,
            resets=resets,
            tables=_FROZEN_AS_TABLES() if reconstruct else None,
            shade=shade,
        )
        assert rationed == 0
        return [
            tuple(
                (str(sid), st.amount.hex())
                for sid, st in sorted(s.stocks.items(), key=lambda kv: str(kv[0]))
            )
            for s in states
        ]

    frozen = run(reconstruct=False, shade=0.0)
    rebuilt = run(reconstruct=True, shade=0.0)
    regulated = run(reconstruct=True, shade=VKS_SHADE_RATE)
    assert frozen == rebuilt, f"{label}: the one-knot reconstruction is NOT faithful"
    assert rebuilt == regulated, f"{label}: the regulator is NOT inert"


def test_frozen_peak_lai_is_below_the_threshold_and_open_season_is_CLOSE() -> None:
    """⚠ FINDING 5 — the new tripwire, in the style of the 14.4248 t/ha Greenwood one.

    The chambers are 9-88x below the LAI-6 threshold and will never reach it; they are
    CARBON-limited by design (the (A) diagnosis measured their plant at 52 g DM/m2).
    ``open_season`` is different: it peaks at 5.191, **86 % of the way to the
    threshold**. So a calibration growing the open-field canopy ~16 % would start a
    sourced, non-fitted mechanism firing in a frozen scenario. A margin that lives only
    in prose is the "freeze's prose half is ungated" shape, so it is asserted.
    """
    peaks = {}
    for label, scenario, years, resets in _ROSTER:
        states, _r, _ = _run(scenario, years, resets=resets)
        peaks[label], _ = _peak_lai(states, scenario)
    for label, peak in peaks.items():
        assert peak < VKS_LAI_THRESHOLD, (label, peak)
    chambers = [v for k, v in peaks.items() if k != "open_season"]
    assert max(chambers) < 1.0, peaks  # 0.632 — an order below, not a near miss
    open_peak = peaks["open_season"]
    assert 0.80 < open_peak / VKS_LAI_THRESHOLD < 0.92, open_peak  # 0.865


def test_the_regulator_does_NOT_rescue_perennial_under_rk4() -> None:
    """⚠ FINDING 4, the headline: the regulator and (C)'s blocker are DISJOINT.

    ``perennial`` is where (C) died, and it dies identically with the regulator in
    place, because its peak LAI is ~0.56 under RK4 — the regulator never fires on the
    failing trajectory. The two scale factors agree to all sixteen digits, which is a
    stronger statement than "it still fails": it is the SAME failure.

    A mutual-shading rule is a canopy-CLOSURE mechanism. A canopy that never closes
    cannot be regulated by one. That is scope (A)'s finding 11 on the other side of the
    plant — "making N faithful does not make the CHAMBER faithful".
    """
    scale = []
    for shade in (0.0, VKS_SHADE_RATE):
        with pytest.raises(Exception, match="over-draw") as exc:
            _run(
                sc.PERENNIAL_CHAMBER_SCENARIO,
                sc.PERENNIAL_CHAMBER_YEARS,
                tables=_LISTING5,
                resets=True,
                integrator=Rk4Integrator,
                shade=shade,
            )
        scale.append(str(exc.value))
    assert scale[0] == scale[1], scale
    assert "0.9527733243688737" in scale[0], scale[0]


def test_the_greenwood_tripwire_fires_WITHOUT_f_n_biting() -> None:
    """⚠ FINDING 6 — a counterexample to a causal claim in ``test_nitrogen_form.py``.

    Its docstring says any calibration growing the open-field crop ~15 % "pushes the
    target below n_critical AND moves a frozen golden". Listing 5 + the regulator grows
    it **+24.5 %** (12.633 -> 15.725) and DOES push the target under n_critical — and
    ``f_N`` stays exactly 1.0 for all 306 steps. The first conjunct does not imply the
    second: ``f_N`` reads the plant's ACTUAL concentration, and demand-deficit uptake
    clamps at zero deficit, so past the crossing the plant sits 15-30 % ABOVE its target
    with no route back down.

    ⚠ The PIN is sound and conservative and is NOT touched — 14.4248 fires before the
    earliest measured bite (Listing 5 first crosses n_critical at W = 15.068 t/ha). It
    is the causal SENTENCE that overstates. "The value may stand" and "its justification
    is falsified" are both true, and the first does not rescue the second (round 4's
    self_discharge).

    And peak mass does not even ORDER the bite: the regulated run reaches a HIGHER peak
    than nothing-bites would suggest and never crosses, while the bare run crosses at a
    LOWER mass. The bite is trajectory-dependent.
    """
    nitro = load_nitrogen_params()

    def f_n_profile(shade: float) -> tuple[float, int, float]:
        states, _r, _ = _run(sc.DEFAULT_SCENARIO, 1, tables=_LISTING5, shade=shade)
        fns, ws = [], []
        for s in states:
            veg = (
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[ROOT_C].amount
            )
            fns.append(
                nitrogen_stress_factor(
                    s.stocks[PLANT_N].amount,
                    veg,
                    n_residual_per_mol_c=nitro.n_residual_per_mol_c,
                    n_critical_per_mol_c=nitro.n_critical_per_mol_c,
                )
            )
            ws.append(
                _t_per_ha(
                    s.stocks[LEAF_C].amount
                    + s.stocks[STEM_C].amount
                    + s.stocks[STORAGE_C].amount,
                    sc.DEFAULT_SCENARIO.ground_area,
                )
            )
        return min(fns), sum(1 for v in fns if v < 1.0), max(ws)

    bare_min, bare_n, bare_w = f_n_profile(0.0)
    reg_min, reg_n, reg_w = f_n_profile(VKS_SHADE_RATE)
    # the bare (C) form: over the crossing AND biting — test_nitrogen_form's expectation
    assert bare_w > _CROSSING_T_HA and bare_min < 1.0 and bare_n == 6
    # the regulated form: over the crossing by +9 %, crop +24.5 %, and NOT biting
    assert reg_w > _CROSSING_T_HA, reg_w  # 15.725
    assert reg_w / 12.633 > 1.20, reg_w  # +24.5 %, well past the "~15 %" of the claim
    assert reg_min == 1.0 and reg_n == 0, (reg_min, reg_n)


def test_the_frozen_concentration_margin_is_wider_than_the_frozen_mass_margin() -> None:
    """Why the mass pin is the right guard to have: it is the tighter of the two.

    ``open_season``'s mass margin to the crossing is ~12 %; its actual N concentration
    never comes within ~28 % of ``n_critical``. Recorded because finding 6's mechanism
    (f_N reads concentration, the pin watches mass) is only reassuring if the pin is
    known to fire first.
    """
    nitro = load_nitrogen_params()
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1)
    assert rationed == 0
    concs = [
        s.stocks[PLANT_N].amount
        / (s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount)
        for s in states
        if s.stocks[LEAF_C].amount > 0
    ]
    assert min(concs) / nitro.n_critical_per_mol_c > 1.28, min(concs)
