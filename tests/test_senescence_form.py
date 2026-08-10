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

import dataclasses
import json
import math
from dataclasses import dataclass
from pathlib import Path

import pytest

from domains.biosphere import scenario as sc
from domains.biosphere.allocation import Senescence, senescence_flux
from domains.biosphere.canopy import CanopyParams, leaf_area_index
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
    load_senescence_params,
)
from domains.biosphere.mineralization import (
    NitrogenSenescence,
    nitrogen_shedding_flux,
)
from domains.biosphere.nitrogen import NitrogenParams, nitrogen_stress_factor
from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.season import (
    CARBON_POOL,
    LEAF_C,
    LONG_HORIZON_YEARS,
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


def _stem_zero(registry: Registry, state: State) -> Registry:
    """``rdr_stem -> 0.0`` on BOTH senescence flows, carrying the aux processes.

    Section 7's candidate, and it is a change to exactly ONE number — leaf and root keep
    their frozen flat rates. This is (C)'s **existence** claim in isolation ([A] p. 95:
    "except for their reserves, stems do not lose weight"; Listing 5 carries LLVT and
    LRTT and no stem function at all), not the DVS-keyed form.

    ⚠ The swap MUST hit ``mineralization.NitrogenSenescence`` too. That flow
    **recomputes** the identical per-organ carbon flux from its own ``SenescenceParams``
    (a flow may read only the step-entry snapshot, so recomputation is the only pure
    form), which is precisely the drift hazard its own docstring names: a one-sided swap
    would keep shedding N at the old stem rate and silently decouple the two legs of one
    physical event. The committed pin guarding that does not run inside a hand-built
    registry, so the invariant is asserted here instead.
    """
    flows: list[object] = []
    hits = 0
    for f in registry.flows:
        if isinstance(f, Senescence):
            flows.append(
                dataclasses.replace(
                    f, params=dataclasses.replace(f.params, rdr_stem=0.0)
                )
            )
            hits += 1
        elif isinstance(f, NitrogenSenescence):
            flows.append(
                dataclasses.replace(
                    f, sen_params=dataclasses.replace(f.sen_params, rdr_stem=0.0)
                )
            )
            hits += 1
        else:
            flows.append(f)
    assert hits >= 1, "no senescence flow was swapped — the candidate is a no-op"
    # The aux processes MUST be carried over (probe B2's bug, ``_candidate`` above).
    return Registry(flows, state.stocks, registry.aux_processes)  # type: ignore[arg-type]


def _run(
    scenario,
    years: int,
    *,
    tables: dict[str, _Knots] | None = None,
    resets: bool = False,
    integrator: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
    shade: float = 0.0,
    stem_zero: bool = False,
):
    w = _weather(years)
    state, registry = build_season(scenario)
    if stem_zero:
        assert tables is None and not shade, "stem-only is measured ALONE, by design"
        registry = _stem_zero(registry, state)
    elif tables is not None or shade:
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
@pytest.mark.science_gate(
    scenario="open_season",
    field="science_bands",
    quantity="peak LAI (m2 m-2)",
    bound="5.0 < peak < 8.0",
    source="real wheat peaks at ~5-8 LAI",
)
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


def _roster_peak_lai() -> dict[str, float]:
    """Peak LAI per frozen scenario — shared by the gate and the margin pin below.

    They were one test until the science-gate work; see the split note on the gate.
    """
    peaks = {}
    for label, scenario, years, resets in _ROSTER:
        states, _r, _ = _run(scenario, years, resets=resets)
        peaks[label], _ = _peak_lai(states, scenario)
    return peaks


@pytest.mark.science_gate(
    scenario="open_season",
    field="science_bands",
    quantity="peak LAI (m2 m-2)",
    bound="peak < 6.0",
    source="Van Keulen & Seligman 1987 mutual-shading threshold, via [A] p. 101",
)
def test_frozen_peak_lai_is_below_the_vks_threshold() -> None:
    """⚠ FINDING 5 — the tripwire, in the style of the 14.4248 t/ha Greenwood one.

    The chambers are 9-88x below the LAI-6 threshold and will never reach it; they are
    CARBON-limited by design (the (A) diagnosis measured their plant at 52 g DM/m2).
    ``open_season`` is different: it peaks at 5.191, and a calibration growing the
    open-field canopy ~16 % would start a sourced, non-fitted mechanism firing in a
    frozen scenario.

    ⚠ **Split note.** This function also carried
    ``0.80 < open_peak / VKS_LAI_THRESHOLD < 0.92`` until the science-gate work. That
    assertion is a *margin* pin — a change IMPROVING the margin fails its upper side —
    so it is not a gate under the inclusion rule and now lives in its own unmarked test
    below. The bound frozen here is the threshold itself, which is sourced.
    """
    peaks = _roster_peak_lai()
    for label, peak in peaks.items():
        assert peak < VKS_LAI_THRESHOLD, (label, peak)
    chambers = [v for k, v in peaks.items() if k != "open_season"]
    assert max(chambers) < 1.0, peaks  # 0.632 — an order below, not a near miss


def test_open_season_lai_margin_to_the_threshold_is_86_percent() -> None:
    """The margin narrative, asserted so it cannot rot — NOT a gate.

    Deliberately unmarked: this is two-sided on a *ratio to our own peak*, so a change
    that moves the canopy further from the threshold fails it. Freezing that as contract
    would let an unfreeze ceremony fail for an improvement.
    """
    open_peak = _roster_peak_lai()["open_season"]
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
    # re-measured 2026-08-10 (the humification split): 0.9527733243688737 -> this. The
    # claim is unchanged and is the point — the regulator still does not rescue
    # `perennial` under RK4, and the two shade settings still produce the IDENTICAL
    # sixteen digits, i.e. the regulator never fires on the failing trajectory.
    assert "0.9363926204726938" in scale[0], scale[0]


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


# --- 7. STEM-ONLY: the branch this module recorded as UNPRICED, now priced ------------
# ``params/senescence.yaml``'s rdr_stem tag said, and this module inherited:
#     "the (C) diagnosis measured the COMBINED form, and stem-only is the one piece
#      plausibly SEPARABLE from the canopy problem (zeroing stem death shrinks the plant
#      rather than blowing up LAI). STEM-ONLY WAS NOT MEASURED, so this is unpriced,
#      not priced-and-rejected."
# Measured 2026-07-28. BOTH halves of that parenthesis are false, and the branch is
# separable from the CANOPY problem but not from the CLOSURE one — which, after the
# canopy-regulator work, is the only thing still blocking (C). Each pin below is named
# for the claim it settles.
_STEM_RATIONED_YEAR_1_DAY = 197  # where the single Euler firing lands (year-1 trough)


def test_the_stem_swap_is_real_and_the_N_LEG_IS_SEALED_ONLY() -> None:
    """The candidate is not a no-op, and it reaches the flow that RECOMPUTES the flux.

    ``NitrogenSenescence`` is built only in a sealed chamber (an open field has no
    ``litter_n`` to shed into), so the count is 0 in ``open_season`` and 1 in the
    chambers. Asserted before any measurement below is believed: that flow recomputes
    the identical per-organ carbon flux from its *own* ``SenescenceParams``, so a swap
    that missed it would leave the two legs of one physical event on different stem
    rates — the drift hazard its own docstring names, whose committed guard does not run
    inside a hand-built registry.
    """
    seen = {}
    frozen = load_senescence_params()
    for label, scenario in (
        ("open_season", sc.DEFAULT_SCENARIO),
        ("sealed_chamber", sc.SEALED_CHAMBER_SCENARIO),
        ("perennial", sc.PERENNIAL_CHAMBER_SCENARIO),
    ):
        state, registry = build_season(scenario)
        swapped = _stem_zero(registry, state)
        carbon = [f for f in swapped.flows if isinstance(f, Senescence)]
        nitro = [f for f in swapped.flows if isinstance(f, NitrogenSenescence)]
        assert len(carbon) == 1 and carbon[0].params.rdr_stem == 0.0, label
        assert all(f.sen_params.rdr_stem == 0.0 for f in nitro), label
        # …and the two organs NOT under test really are untouched.
        assert carbon[0].params.rdr_leaf == frozen.rdr_leaf
        assert carbon[0].params.rdr_root == frozen.rdr_root
        seen[label] = len(nitro)
    assert seen == {"open_season": 0, "sealed_chamber": 1, "perennial": 1}, seen


def test_zeroing_stem_death_GROWS_the_plant_and_our_own_file_predicted_SHRINKS() -> (
    None
):
    """⚠ Claim 1 FALSIFIED — a prediction written as a fact, in a param file.

    ``rdr_stem`` is a LOSS term, so removing it retains stem carbon and W (Greenwood's
    basis: leaf + stem + storage) goes **UP** +7.96 % on ``open_season``. The
    parenthesis in ``senescence.yaml`` — "zeroing stem death shrinks the plant" — reads
    as a recorded measurement and was never run.

    It was not baseless, and the next pin is why: three of the four organs really do
    shrink. What the sentence did was name the whole plant for the behaviour of the
    majority of its organs, when the one dissenting term is the largest.
    """
    frozen, _r0, _ = _run(sc.DEFAULT_SCENARIO, 1)
    stem0, _r1, _ = _run(sc.DEFAULT_SCENARIO, 1, stem_zero=True)

    def peak_w(states) -> float:
        return max(
            _t_per_ha(
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[STORAGE_C].amount,
                sc.DEFAULT_SCENARIO.ground_area,
            )
            for s in states
        )

    assert peak_w(frozen) == pytest.approx(12.633, rel=1e-3)
    assert peak_w(stem0) == pytest.approx(13.639, rel=1e-3)
    assert peak_w(stem0) > peak_w(frozen)  # THE point: up, not down
    assert peak_w(stem0) / peak_w(frozen) == pytest.approx(1.0796, rel=1e-3)


def test_the_stem_grows_and_the_OTHER_THREE_organs_take_ONE_haircut() -> None:
    """The per-organ split a scalar W hides — and the common haircut names the cause.

    Stem peak **+23.4 %**; leaf **-3.96 %**, root **-3.91 %**, storage **-3.97 %**. That
    three organs on three different DVS-keyed partition fractions fall by the same
    fraction to a tenth of a percentage point is not coincidence: the partition table is
    untouched, so a uniform haircut says the stream being partitioned — net assimilate —
    shrank. Measured cause: a bigger STANDING stem costs more to maintain
    (``maintenance_coef`` 0.02/day, charged on live tissue). On ``open_season`` the
    integral of standing live tissue rises 3.08 %, i.e. ~1.49 mol C of extra maintenance
    respiration against a 0.89 mol C fall in final storage.

    ⚠ The honest one-line reading is **"stem up, grain down"** — the plant is bigger and
    worse. And the branch would introduce a form gap rather than close one: our single
    ``stem_c`` pool cannot express [A]'s own "except for their **reserves**", so zeroing
    the death rate makes the stem a strictly one-way pool.
    """
    frozen, _r0, _ = _run(sc.DEFAULT_SCENARIO, 1)
    stem0, _r1, _ = _run(sc.DEFAULT_SCENARIO, 1, stem_zero=True)

    def peak(states, stock) -> float:
        return max(s.stocks[stock].amount for s in states)

    ratios = {
        name: peak(stem0, stock) / peak(frozen, stock)
        for name, stock in (
            ("leaf", LEAF_C),
            ("stem", STEM_C),
            ("root", ROOT_C),
            ("storage", STORAGE_C),
        )
    }
    assert ratios["stem"] == pytest.approx(1.2336, rel=1e-3), ratios
    for organ in ("leaf", "root", "storage"):
        assert ratios[organ] < 1.0, ratios
    haircuts = [ratios[o] for o in ("leaf", "root", "storage")]
    assert max(haircuts) - min(haircuts) < 1e-3, ratios
    assert min(haircuts) == pytest.approx(0.9603, rel=1e-3), ratios


def test_stem_only_shrinks_the_greenwood_margin_WITHOUT_crossing_it() -> None:
    """The mass tripwire is approached, not tripped: 13.639 vs the 14.4248 crossing.

    The margin more than halves (12.4 % -> 5.4 %), leaving ``open_season`` one small
    calibration short of the first ``f_N`` bite in a frozen scenario. Recorded as a near
    miss, not as a pass: finding 6 established that crossing the mass pin does not by
    itself move a golden, and "staying under it is not safety" is that same distinction
    from the other side.
    """
    stem0, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1, stem_zero=True)
    assert rationed == 0
    peak_w = max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount,
            sc.DEFAULT_SCENARIO.ground_area,
        )
        for s in stem0
    )
    assert peak_w < _CROSSING_T_HA, peak_w
    assert peak_w / _CROSSING_T_HA == pytest.approx(0.9455, rel=1e-3), peak_w


def test_the_two_tripwires_move_in_OPPOSITE_directions() -> None:
    """⚠ The clearest demonstration yet that mass and area margins are not one thing.

    Stem-only moves W **toward** the Greenwood crossing (0.876 -> 0.946 of it) and peak
    LAI **away** from the V-K&S threshold (0.865 -> 0.831 of it), because leaf carbon is
    downstream of the assimilate stream the bigger stem is taxing. The canopy-regulator
    work flagged that conflating a *mass* margin with a *leaf-area* one is an ambiguity
    that has bitten this repo twice; here one single-number change pushes them in
    opposite directions at once, so no scalar "how close are we" exists.
    """
    cp = load_canopy_params()
    out = {}
    for label, kw in (("frozen", {}), ("stem0", {"stem_zero": True})):
        states, _r, _ = _run(sc.DEFAULT_SCENARIO, 1, **kw)  # type: ignore[arg-type]
        out[label] = (
            max(
                _t_per_ha(
                    s.stocks[LEAF_C].amount
                    + s.stocks[STEM_C].amount
                    + s.stocks[STORAGE_C].amount,
                    sc.DEFAULT_SCENARIO.ground_area,
                )
                for s in states
            )
            / _CROSSING_T_HA,
            max(
                leaf_area_index(
                    s.stocks[LEAF_C].amount,
                    sla_per_mol_c=cp.sla_per_mol_c,
                    ground_area=sc.DEFAULT_SCENARIO.ground_area,
                )
                for s in states
            )
            / VKS_LAI_THRESHOLD,
        )
    assert out["stem0"][0] > out["frozen"][0], out  # the mass margin CLOSES
    assert out["stem0"][1] < out["frozen"][1], out  # the area margin OPENS
    assert out["frozen"] == pytest.approx((0.8758, 0.8651), rel=1e-3)
    assert out["stem0"] == pytest.approx((0.9455, 0.8309), rel=1e-3)


def test_stem_only_NO_LONGER_rations_the_perennial_chamber_UNDER_EULER() -> None:
    """⚠⚠ **RESOLVED 2026-08-10 — THIS FINDING IS DISCHARGED, AND IT WAS THE (C)
    DIAGNOSIS'S LAST STANDING BRANCH.** Everything below the line was a true measurement
    of the pre-humification tree and is kept because the way it was reached is the
    value;
    what it measured is gone.

    Under the humification split (``docs/plans/post-roadmap-cue-humification.md``)
    stem-only runs ``rationed == 0`` on ``perennial`` under Euler at dt=1 — the frozen
    reference configuration — at BOTH the 5-year and 15-year horizons. The mechanism is
    the one the inventory test below identifies: the split returns 45 % of decayed
    litter
    carbon to the atmosphere immediately instead of routing all of it through a
    microbial
    pool with a ~62-day residence time, so the CO2 trough that the growing stem was
    drawing down is simply higher. Measured: the 15-yr minimum goes 0.008674 ->
    0.046065.

    ⚠⚠ **THIS DOES NOT RE-OPEN (C), AND SAYING SO PRECISELY MATTERS.** Stem-only's
    refusal had two closure legs. This one is discharged. The other — the decade CO2
    liveness floor — **survives, and narrowly**: see the next test, where the settled
    attractor is now comfortably above the floor but a single post-transient year dips
    to 0.046065 against 0.05. The refusal therefore now rests on an 8 % miss in one year
    rather than on a hard break plus a 3.4x collapse. Whether that still justifies
    refusing the branch is a question for whoever revisits (C) with the measurement in
    hand; it is **not** settled here, and re-deciding it inside the commit that changed
    the tree underneath it would be exactly the co-adaptation shape this project
    refuses.

    ---- the original finding, as measured before the split
    -----------------------------

    ⚠ THE BLOCKING FINDING, and it is independent of any tuned guard.

    ``perennial`` goes ``rationed 0 -> 1`` under **Euler at dt=1**, the frozen reference
    configuration. One firing is a hard break rather than a drift, and the site is
    specific: ``test_regression_perennial_season.py:77`` asserts ``rationed == 0`` as
    that golden's pre-capture closure gate. So stem-only dies on ``perennial``'s
    CLOSURE — the same wall the combined (C) form hit, reached by a different road.

    This is what settles "separable". Stem-only IS separable from the canopy problem
    (the tripwire pin above: peak LAI *falls*), and is NOT separable from the closure
    problem — which, after the canopy regulator discharged branch 2, is the only branch
    of (C)'s refusal still standing.

    ⚠ **WHERE it fires is MEASURED, not read off the CO2 argmin.** My first version of
    this pin asserted the location of the **CO2 minimum** under a constant named for the
    **rationing** step and a docstring claiming they were the same event. They are two
    different quantities and nothing had measured that they coincide — while the claim
    was load-bearing, since "within-season" is exactly what distinguishes this from the
    beyond-horizon tiling/reset artefact the decomposer calibration documents. Measured
    here by **horizon truncation**, which needs no internal API: the run is
    deterministic, so the smallest horizon that rations IS the firing step. They do
    coincide — and the reason is worth having: at step 502 the pool is in free fall
    (0.727 -> 0.504 -> 0.222 -> 0.009 over four steps), so **the trough is the value the
    backstop CLAMPED to**, not one the dynamics reached on their own. The argmin is
    downstream of the firing, which is why reading the firing off it was circular.
    """
    frozen, r_frozen, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS, resets=True
    )
    stem0, r_stem0, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO,
        sc.PERENNIAL_CHAMBER_YEARS,
        resets=True,
        stem_zero=True,
    )
    assert r_frozen == 0
    assert r_stem0 == 0, r_stem0  # ⚠ was 1 — the discharge, and it is the whole point

    # The trough that used to be a backstop CLAMP (0.008674, reached in free fall) is
    # now
    # a value the dynamics actually reach, and it is 5.3x higher.
    co2 = [s.stocks[CARBON_POOL].amount for s in stem0]
    assert min(co2) == pytest.approx(0.046065, rel=1e-3)
    assert min(s.stocks[CARBON_POOL].amount for s in frozen) == pytest.approx(
        0.055175, rel=1e-3
    )
    # The discharge is not a horizon artefact: it holds at the long horizon too.
    _, r_stem_long, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO, LONG_HORIZON_YEARS, resets=True, stem_zero=True
    )
    assert r_stem_long == 0


def test_stem_only_collapses_the_decade_co2_attractor_below_its_floor() -> None:
    """The second, independent closure failure: the settled attractor, not a transient.

    ``test_decade_stability.test_decade_min_carbon_pool_stationary`` pins the per-year
    CO2 minimum above a 0.05 floor. The frozen tree settled at **0.05484** (min past the
    transient 0.054208 — reproduced here, and validated against that test's own comment,
    "dips to ~0.039 … settling to ~0.055", before it was trusted: finding 10's rule).
    Stem-only settled at **0.01619**, missing the floor by 3.4x.

    ⚠ EVERY NUMBER IN THE PARAGRAPH ABOVE BELONGS TO THE PRE-HUMIFICATION-SPLIT TREE,
    and so does the comment it validated against — which no longer exists, being the
    stale prose the guard's 2026-08-10 re-anchor removed. Kept rather than deleted
    because the *method* is the point (reconstruct a frozen quantity only to CHECK it
    against the recorded one). The current readings are in the assertions below and in
    the re-measurement note after them. The guard also no longer skips the sow-in years:
    its window was measured inert on the frozen tree and removed, which is a tightening,
    so the ``[2:]`` slice this test used to mirror is gone here too. Stem-only's verdict
    is unchanged either way — it misses at year 2, inside and outside the window.

    ⚠ And it does so while STAYING STATIONARY — a clean attractor in the wrong place.
    That is a different failure mode from the combined (C) form, which lost stationarity
    and wandered 0.006-0.027. A stationarity check alone would have passed this; the
    level check is what catches it, which is precisely the "alive" guard
    ``is_stationary`` is documented to be blind to. Both halves are asserted, so a
    future change that swapped which guard fires would go red.
    """
    year_len = len(_weather())
    out = {}
    for label, kw in (("frozen", {}), ("stem0", {"stem_zero": True})):
        states, _r, _ = _run(
            sc.PERENNIAL_CHAMBER_SCENARIO,
            LONG_HORIZON_YEARS,
            resets=True,
            **kw,  # type: ignore[arg-type]
        )
        summaries = year_summaries(
            states, year_len, lambda seg: min(s.stocks[CARBON_POOL].amount for s in seg)
        )
        scale = max(summaries)
        out[label] = (
            summaries,
            non_collapsing(summaries, floor=0.05),
            is_stationary(
                same_phase_diffs(summaries, period=2),
                bound=0.2 * scale,
                slope_tol=0.02 * scale,
                transient=2,
            ),
        )
    # ⚠⚠ RE-MEASURED 2026-08-10 (the humification split). The VERDICT holds — stem-only
    # still fails this floor — but the failure MODE changed twice over, and my first
    # rewrite of this block got it wrong in the direction that flattered the change.
    #
    # Before: a clean, stationary attractor at 0.01619, missing the floor by 3.4x. The
    # docstring above calls that "a different failure mode from the combined (C) form,
    # which lost stationarity"; that contrast no longer holds.
    #
    # ⚠ I first wrote that the attractor is now "0.074891, comfortably above the floor"
    # with only a single year dipping — i.e. that the refusal had thinned to an 8 %
    # miss.
    # THE STATIONARITY ASSERTION BELOW CAUGHT IT: the series is NOT settled at 15 years.
    # Per-year minima run [0.0760, 0.0626, 0.0461, 0.0539, 0.0752, 0.0708, 0.0678,
    # 0.0673, 0.0680, 0.0693, 0.0707, 0.0720, 0.0731, 0.0741, 0.0749] — a deep early
    # swing, then a slow monotone climb still rising at the horizon. So 0.074891 is the
    # LAST value, not the attractor, and both guards fire.
    #
    # The honest reading is the same one that restated the two decade-stability pins and
    # the station biomass gate: the humification split lengthens the settling transient
    # past the horizon these guards assume (their ``transient=2`` skips two years; this
    # transient runs about five). Stem-only's refusal on this leg therefore stands, and
    # stands on a longer transient rather than on a settled collapse.
    assert out["frozen"][0][1] == pytest.approx(0.055175, rel=1e-3)
    assert out["frozen"][0][-1] == pytest.approx(0.073367, rel=1e-3)
    assert out["frozen"][1] is True and out["frozen"][2] is True
    assert out["stem0"][0][-1] == pytest.approx(0.074891, rel=1e-3)
    assert min(out["stem0"][0][2:]) == pytest.approx(0.046065, rel=1e-3)
    assert out["stem0"][1] is False, "the floor guard catches it"
    assert out["stem0"][2] is False, "…and so, now, does stationarity"
    # The shape, pinned so "stem-only collapses" cannot be quoted off this test: the
    # tail
    # is RISING and ends above the floor, and the miss is in the early transient.
    assert out["stem0"][0][-1] > out["stem0"][0][-2] > out["stem0"][0][-3]
    assert out["stem0"][0][-1] > 0.05
    assert out["stem0"][0].index(min(out["stem0"][0])) == 2


def test_RK4_survives_stem_only_which_INVERTS_the_pattern_C_established() -> None:
    """⚠⚠ **THE INVERSION IS GONE (2026-08-10) — because its Euler half was
    discharged.**

    As measured before the humification split: (C) had Euler report ``rationed == 0``
    while RK4 hard-errored ("Euler reading clean is the trap"), and stem-only was the
    **opposite** — Euler rationed, RK4 was clean. Two cases pointing opposite ways was
    the evidence that neither integrator screens for the other.

    Stem-only no longer rations under Euler (see the test above), so this is no longer a
    counter-example: both integrators now agree that stem-only's rationing gate passes.
    **The generalisation it supported survives on (C) alone plus this history**, and is
    weaker for it — recorded rather than quietly kept at full strength, because "neither
    integrator screens for the other" was a two-case claim and it is now a one-case
    claim
    with a retired second case.

    What this test still measures, and is worth keeping: RK4 completes cleanly on both
    forms at both horizons with the CO2 minimum essentially unmoved between them — so
    RK4 is insensitive to a change that Euler's own liveness floor still refuses. That
    is
    the same lesson from the surviving direction.
    """
    for years in (sc.PERENNIAL_CHAMBER_YEARS, LONG_HORIZON_YEARS):
        for kw in ({}, {"stem_zero": True}):
            states, rationed, _ = _run(
                sc.PERENNIAL_CHAMBER_SCENARIO,
                years,
                resets=True,
                integrator=Rk4Integrator,
                **kw,  # type: ignore[arg-type]
            )
            assert rationed == 0, (years, kw)
            if years == LONG_HORIZON_YEARS:
                lo = min(s.stocks[CARBON_POOL].amount for s in states)
                # re-measured 2026-08-10: 0.075815/0.075893 -> 0.076829/0.076990
                assert lo == pytest.approx(0.0769, rel=1e-2), (kw, lo)


# The frozen `sealed_chamber` run's CO2-trough step; both runs are sampled here so
# the deltas are taken at one instant (they trough one step apart since 2026-08-10).
_COMMON_TROUGH_STEP = 195


def test_the_sealed_carbon_inventory_is_CONSERVED_and_the_stem_is_where_it_went() -> (
    None
):
    """⚠ THE MECHANISM — and my first hypothesis was wrong, which is why it is here.

    I predicted LITTER STARVATION: stem carbon that never sheds never reaches
    ``litter_carbon``, so less CO2 is returned. The pools refute it — the litter pool's
    mean falls ~13 % and peak ``microbial_carbon`` 0.5 %, against a 55 % fall in the CO2
    minimum. The recycling is not starved.

    What actually happens is a STANDING STOCK. A sealed chamber's carbon inventory is
    fixed (measured: identical to <1e-9 between the two runs), so the CO2 trough is
    whatever the other pools are not holding; and a pool's equilibrium size scales as
    1/(loss rate), so zeroing that rate makes the stem a one-way sink within a season
    and every other pool funds it. At ``sealed_chamber``'s trough, standing tissue is
    **+0.1179 mol C**, drawn ~67 % from the soil pools (litter -0.0627, microbial
    -0.0160) and ~33 % from the atmosphere (-0.0392).

    That is why ``open_season`` grows while the chambers choke on the same change: the
    open field draws on an unbounded CO2 reservoir and the chamber pays immediately.
    Scope (A)'s finding 11 from the other direction — a field-scale improvement is not a
    chamber-scale one.
    """
    # ⚠ ``humus_carbon`` joined this list with the humification split (2026-08-10).
    # Omitting it would break the closed-inventory identity below by exactly the humus
    # amount — the third site in this repo where a "sum the organic pools" tuple had to
    # learn about a new pool, and the reason
    # ``test_decomposition`` ::
    # ``test_every_organic_carbon_pool_is_named_by_the_summary_tuples``
    # now guards the set structurally.
    soil_pools = ("litter_carbon", "microbial_carbon", "humus_carbon")
    snaps = {}
    for label, kw in (("frozen", {}), ("stem0", {"stem_zero": True})):
        states, _r, _ = _run(
            sc.SEALED_CHAMBER_SCENARIO,
            sc.SEALED_CHAMBER_YEARS,
            **kw,  # type: ignore[arg-type]
        )
        co2 = [s.stocks[CARBON_POOL].amount for s in states]
        own_lo = min(range(len(co2)), key=lambda i: co2[i])
        # ⚠ The two runs used to trough at the SAME step (196), and this test compared
        # each at its own. Since the humification split they trough one step apart (195
        # frozen, 194 stem-only), so every delta below is taken at a COMMON step — the
        # frozen run's trough — rather than silently differencing two different
        # instants.
        # The one-step offset is asserted rather than absorbed.
        lo = _COMMON_TROUGH_STEP
        st = states[lo]
        by_short = {
            str(sid).rsplit(".", 1)[-1]: s.amount for sid, s in st.stocks.items()
        }
        snaps[label] = {
            "step": lo,
            "own_step": own_lo,
            "co2": co2[lo],
            "tissue": st.stocks[LEAF_C].amount
            + st.stocks[STEM_C].amount
            + st.stocks[ROOT_C].amount
            + st.stocks[STORAGE_C].amount,
            "soil": sum(by_short.get(p, 0.0) for p in soil_pools),
        }
    b, n = snaps["frozen"], snaps["stem0"]
    assert (b["own_step"], n["own_step"]) == (195, 194), (b["own_step"], n["own_step"])
    assert b["step"] == n["step"] == _COMMON_TROUGH_STEP
    # The inventory is closed: what one group gained, the others lost, exactly.
    total_b = b["co2"] + b["tissue"] + b["soil"]
    total_n = n["co2"] + n["tissue"] + n["soil"]
    assert abs(total_n - total_b) < 1e-9, (total_b, total_n)
    assert total_b == pytest.approx(3.517, rel=1e-6)
    d_tissue = n["tissue"] - b["tissue"]
    d_soil = n["soil"] - b["soil"]
    d_co2 = n["co2"] - b["co2"]
    # re-measured 2026-08-10 (humification split). The MECHANISM is unchanged and is
    # what this test exists for: the extra standing stem is funded by the other pools,
    # mostly the soil. The split moves the split of the funding, not the story.
    assert d_tissue == pytest.approx(0.084649, rel=2e-3), d_tissue
    assert d_soil == pytest.approx(-0.085210, rel=2e-3), d_soil
    assert d_co2 == pytest.approx(0.000562, rel=2e-2), d_co2
    assert abs(d_tissue + d_soil + d_co2) < 1e-9
    # ⚠⚠ **WHERE THE FUNDING COMES FROM IS THE FINDING, AND IT INVERTED (2026-08-10).**
    # Before the humification split the extra standing stem was drawn ~67 % from the
    # soil
    # pools and ~33 % from the atmosphere, and the atmospheric third is what pushed the
    # CO2 trough into the backstop — the measured reason stem-only was refused on
    # `perennial`'s closure. Under the split the soil funds essentially ALL of it
    # (d_soil/d_tissue ≈ -1.007) and the CO2 pool at the trough is **very slightly UP**.
    #
    # That is the mechanism behind the discharge two tests above, and it is worth having
    # explicitly: the split gives the soil a third pool with a ~5-yr residence time, so
    # the soil can fund a standing sink out of its own inventory instead of out of the
    # atmosphere. The generalisation this file recorded — "any change parking carbon in
    # a
    # standing pool is paid out of the CO2 trough" — is therefore FALSE as a law; it was
    # true of a soil with one fast pool.
    assert d_soil / d_tissue == pytest.approx(-1.0, abs=0.02)
    assert d_co2 > 0.0, "the atmosphere no longer funds the standing stem"
    assert abs(d_co2) / b["co2"] < 0.01, d_co2  # <1 % off the CO2 trough, and upward


# =====================================================================================
# 8. THE (C)/STEM-ONLY RE-PRICE (2026-08-10) — the parked question, measured
# =====================================================================================
#
# The humification split discharged stem-only's rationing leg and deliberately did NOT
# re-decide the branch inside the commit that moved the tree underneath it, leaving:
#
#   "Whether that still justifies refusing the branch is a question for whoever
#    revisits (C) with the measurement in hand."
#
# This section is that measurement. It settles what a run can settle and stops there:
# the acceptability call is a CONTRACT question (which window the guards use), and
# picking a window because the subject goes green is the shape this project has refused
# four times (the consumer-chamber 2x, the DPM/RPM labile re-read, ruling B, the
# fractionation seed sweep).
#
# ⚠ THE FRAMING I STARTED FROM WAS HALF WRONG AND THE MEASUREMENT CORRECTED IT. The
# surviving leg was expected to split into a HORIZON question (stationarity — "the
# series is still rising at 15 years") and a WINDOW question (the floor — the failing
# year IS index 2, the first year ``transient=2`` lets the guard see, so no horizon
# moves it). Measured: BOTH are window questions. The offending same-phase diffs sit at
# fixed indices 2 and 3, so ``is_stationary`` returns False at 15 AND at 50 years even
# though the series is flat to eight decimals across its last five years.


def _manifest_floor(quantity_fragment: str, bound_prefix: str) -> float:
    """Read a ``perennial_long_horizon`` liveness floor out of the biosphere manifest.

    ⚠ Hand-copied, these drift silently: the leaf floor has already moved TWICE
    (``> 1.0`` at the decomposer calibration, ``> 0.9``, then ``> 0.55`` at the
    humification split), and a stale copy would keep asserting the old bound under a
    comment still claiming it is *"the manifest bound"* — the ungated-prose shape, one
    level inside a test. Reading it costs a line and removes the drift.
    """
    manifest_path = (
        Path(__file__).parent.parent / "docs" / "biosphere-reference.manifest.json"
    )
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = [
        e
        for e in manifest["liveness_floors"]["perennial_long_horizon"]
        if quantity_fragment in e["quantity"] and e["bound"].startswith(bound_prefix)
    ]
    assert len(entries) == 1, f"expected exactly one {quantity_fragment!r} floor"
    tail = entries[0]["bound"][len(bound_prefix) :]
    return float(tail.split(")")[0].split()[0])


_DECADE_CO2_FLOOR = _manifest_floor("chamber CO2 pool", "non_collapsing(floor=")
_LIVENESS_FLOOR = _manifest_floor("peak-leaf fixed point", "max(tail) > ")
_DEAD_BASELINE = 0.253  # what that floor exists to separate a live plant from
_LIVENESS_TRANSIENT = 8  # test_decade_stability._PERIOD_TRANSIENT
_REPRICE_YEARS = 50  # the horizon the humification split anchored its own floor on


def test_the_two_floors_this_section_asserts_against_come_FROM_the_manifest() -> None:
    """Both bounds are contractual, so they are read, not typed — and pinned by value.

    The read is only worth having if it is also checked: a parser that silently
    returned the wrong number would be worse than the hand-copy it replaced. So the
    values are asserted here (the drift shows up as ONE red test naming the change),
    and the section below consumes the constants.
    """
    assert _DECADE_CO2_FLOOR == 0.05
    assert _LIVENESS_FLOOR == 0.55


def _peak_leaf_of(segment) -> float:
    """``test_decade_stability._peak_leaf``, reproduced (not imported across modules).

    Validated against that gate's own committed 15-year reading before use, per the
    (A)-diagnosis finding-10 rule: reconstruct a frozen quantity only to CHECK it
    against the recorded one, never to replace it.
    """
    return max(s.stocks[LEAF_C].amount for s in segment)


def _co2_min_of(segment) -> float:
    return min(s.stocks[CARBON_POOL].amount for s in segment)


def _reprice_series(years: int, stem_zero: bool):
    states, rationed, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO, years, resets=True, stem_zero=stem_zero
    )
    year_len = len(_weather())
    return (
        year_summaries(states, year_len, _co2_min_of),
        year_summaries(states, year_len, _peak_leaf_of),
        year_summaries(
            states, year_len, lambda s: max(x.stocks[STEM_C].amount for x in s)
        ),
        year_summaries(
            states, year_len, lambda s: max(x.stocks[STORAGE_C].amount for x in s)
        ),
        rationed,
    )


def _stationary_verdict(series: list[float]) -> bool:
    """Exactly the committed decade-CO2 test's stationarity call, same constants."""
    scale = max(series)
    return is_stationary(
        same_phase_diffs(series, period=2),
        bound=0.2 * scale,
        slope_tol=0.02 * scale,
        transient=2,
    )


def test_neither_stem_only_guard_is_a_horizon_question() -> None:
    """⚠ THE FRAMING CORRECTION, and it is cheap enough to pin without a 50-year run.

    The parked question read as "the series has not settled inside the frozen horizon",
    which invites the answer "then run it longer". For the FLOOR guard that answer is
    structurally unavailable: the committed pins already say the failing year IS index
    2 and that ``argmin == 2``, so ``non_collapsing(summaries[2:], floor=0.05)``
    contains 0.046065 at EVERY horizon. Only ``transient`` can move it.

    ⚠ THAT LAST SENTENCE — and this test's original name, "…are BOTH WINDOW questions"
    — WERE TRUE OF A GUARD THAT NO LONGER EXISTS, and the correction runs in the
    STRENGTHENING direction. On 2026-08-10 the floor's ``[_TRANSIENT:]`` window was
    measured **inert on the frozen tree** (whole-run min 0.055175 = 1.103x the floor,
    nothing below it, ``non_collapsing`` True sliced *and* whole) and removed, so the
    committed guard is now ``non_collapsing(summaries, floor=0.05)``: **not even
    ``transient`` can move the floor verdict, because there is no window left to move.**
    Kept rather than rewritten, because what it measured was true of the guard it was
    measuring — resolved, not corrected. The STATIONARITY half is untouched: its
    ``transient=2`` stays, its binding diff sitting at index 2 and not dropped by the
    window anyway. ``docs/plans/post-roadmap-co2-guard-reanchor.md``.

    The same turns out to be true of the STATIONARITY guard, which is the part that was
    not obvious: its offending same-phase diffs are ``diffs[2] = series[4] - series[2]``
    and ``diffs[3] = series[5] - series[3]``, both at fixed indices inside the
    establishment dip. Appending years cannot remove them.

    ⚠ Scope this honestly: the invariance is MEASURED AT TWO HORIZONS (15 and 50), not
    proved. ``bound = 0.2 * max(series)`` and the max lands in year 0 in both runs, so a
    change that raised the series' maximum could in principle loosen the bound enough to
    matter. What is asserted is what was run.
    """
    co2, _leaf, _stem, _store, rationed = _reprice_series(LONG_HORIZON_YEARS, True)
    assert rationed == 0

    # the committed 15-year readings, reproduced before anything is built on them
    assert min(co2[2:]) == pytest.approx(0.046065, rel=1e-3)
    assert co2.index(min(co2)) == 2, "the failing year is the FIRST the guard sees"
    assert co2[-1] == pytest.approx(0.074891, rel=1e-3)

    # the floor guard's verdict is decided by a year no horizon can remove...
    assert non_collapsing(co2[2:], floor=_DECADE_CO2_FLOOR) is False
    # ...and, since 2026-08-10, no WINDOW can remove it either: the committed guard
    # dropped the slice, and the same year decides the whole-run form. Both are pinned
    # because the pair is the claim — the verdict is window-independent, not re-tuned.
    assert non_collapsing(co2, floor=_DECADE_CO2_FLOOR) is False

    # and so is stationarity's — locate the offenders rather than assert the verdict
    scale = max(co2)
    diffs = same_phase_diffs(co2, period=2)
    offenders = [i for i, d in enumerate(diffs) if i >= 2 and abs(d) > 0.2 * scale]
    assert offenders == [2, 3], offenders
    assert _stationary_verdict(co2) is False


@pytest.mark.slow
def test_the_stem_only_refusal_at_FIFTY_years_with_its_control() -> None:
    """⚠⚠ THE RE-PRICE. Every claim from ONE pair of 50-year runs, labelled per claim.

    Consolidated into a single function on the acceptance-gate diagnosis's precedent: a
    50-year ``perennial`` pair is expensive, a session cache is a PER-WORKER cache under
    xdist, and the lever that actually works is fewer tests that need the computation.

    ⚠ THE HORIZON IS THE FAIRNESS REQUIREMENT, NOT A SEARCH. The humification split
    lengthened this chamber's settling transient from ~3 years to ~35 and anchored its
    OWN liveness floor on a measured equilibrium at ~yr 45 rather than on the 15-year
    reading. The soil-fractionation re-refusal then asked the same question of a change
    it was about to refuse, BEFORE writing the refusal. Asking it here is that
    discipline applied to a change already refused — and the answer comes out the other
    way, which is precisely why it had to be asked.

    WHAT IS MEASURED:

    1. **stem-only reaches an attractor, and it is ABOVE the frozen tree's own.**
       0.075339 vs the control's 0.073291 — 1.0279x the reference and **1.51x the 0.05
       floor**. Contrast the soil-fractionation re-refusal, where the same 50-year
       question returned 0.031741, **1.58x BELOW** the floor. Same test, opposite
       answer: there the failure was the attractor, here it is not.
    2. **Both guards fire exclusively inside years 2-5**, the establishment transient.
       Exactly ONE year of fifty sits below the floor (year 2, 0.046065 — 7.87 % below);
       the frozen control has none, its own worst year being 0.055175 at year 1.
    3. **The manifest-named gate CLEARS.** ``perennial_long_horizon``'s
       ``liveness_floors`` bound ``max(tail) > 0.55`` — contractually named, where the
       decade-CO2 pin is only a committed test — gives 0.643676 for stem-only against
       0.634352 frozen. There is no third leg.
    4. ⚠ **AND THE PLANT IS NOT FREE.** Reading the CO2 improvement alone is the thing
       the CUE row forbids. Peak stem +51.8 %, peak **storage (grain) -11.8 %** — so
       (C)-finding 8's "stem up, grain down" holds here and HARDER than on
       ``open_season`` (+23.4 % / -3.97 %). Peak leaf goes the other way (+1.98 % where
       ``open_season`` gave -3.96 %), so the leaf sign does NOT transfer between
       scenarios and is measured here rather than inherited.

    WHAT IS **NOT** SETTLED, DELIBERATELY: whether ``transient=2`` is the right window.
    Moving it to 3 clears the floor and to 5 clears stationarity; the frozen control
    passes at ``transient=0``. Choosing a window because the subject goes green is the
    refused shape, and the current window is not tuned to the reference either. That is
    a contract question and it is left open.

    ⚠ THE WINDOW QUESTION WAS ANSWERED ON 2026-08-10 AND NOT BY CHOOSING A WINDOW: the
    floor's slice was measured **inert on the frozen tree** and removed (a strict
    tightening, since ``non_collapsing(whole)`` implies ``non_collapsing(sliced)``),
    while stationarity's ``transient=2`` stays on a measurement of its own. The
    paragraph above is kept as the question that was open, not as one that is.

    ⚠ WHAT IS STILL OPEN IS A DIFFERENT QUESTION, and this test is what sharpened it:
    is a **deeper sow-in transient with a healthier attractor** a failure? The frozen
    tree's own worst year sits at 1.103x the floor and stem-only's at 0.921x, while
    stem-only settles ABOVE the reference. That is a contract call and it is the user's
    — re-deciding it inside the commit that moved its guard is the refused shape, so
    the guard moved without this verdict moving. ``non_collapsing`` is asserted here in
    BOTH forms for that reason: the refusal does not depend on the removed window.
    """
    f_co2, f_leaf, f_stem, f_store, f_r = _reprice_series(_REPRICE_YEARS, False)
    s_co2, s_leaf, s_stem, s_store, s_r = _reprice_series(_REPRICE_YEARS, True)

    # --- the discharged leg must not un-discharge at 3.3x the frozen horizon ---------
    assert f_r == 0, "frozen control rations at 50 yr — the comparison would be void"
    assert s_r == 0, "stem-only rations at 50 yr — the discharge was horizon-bound"

    # --- 1. the attractor, subject AND control on the same harness ------------------
    assert s_co2[-1] == pytest.approx(0.075339, abs=1e-5), "stem-only CO2 attractor"
    assert f_co2[-1] == pytest.approx(0.073291, abs=1e-5), "frozen CO2 attractor"
    # the control's asymptote is independently pinned by test_soil_fractionation.py at
    # 0.073291 — a free cross-harness agreement, and the reason these numbers are usable
    assert s_co2[-1] > _DECADE_CO2_FLOOR and f_co2[-1] > _DECADE_CO2_FLOOR
    assert s_co2[-1] / _DECADE_CO2_FLOOR == pytest.approx(1.5068, abs=1e-3)
    assert s_co2[-1] > f_co2[-1], "stem-only settles ABOVE the reference, not below"
    assert max(s_co2[-5:]) - min(s_co2[-5:]) < 1e-6, "and it is genuinely settled"

    # --- 2. the failure is one establishment year, not the attractor ----------------
    below = [i for i, v in enumerate(s_co2) if v < _DECADE_CO2_FLOOR]
    assert below == [2], below
    assert [i for i, v in enumerate(f_co2) if v < _DECADE_CO2_FLOOR] == []
    assert s_co2[2] / _DECADE_CO2_FLOOR == pytest.approx(0.9213, abs=1e-3)
    assert min(f_co2) == pytest.approx(0.055175, abs=1e-5), "the control's own trough"
    # both guards still refuse, at fifty years exactly as at fifteen
    assert non_collapsing(s_co2[2:], floor=_DECADE_CO2_FLOOR) is False
    assert _stationary_verdict(s_co2) is False
    assert non_collapsing(f_co2[2:], floor=_DECADE_CO2_FLOOR) is True
    assert _stationary_verdict(f_co2) is True
    # ...and under the guard AS COMMITTED SINCE 2026-08-10 (no window), unchanged both
    # ways — which is the point: the re-anchor did not decide this verdict, and the
    # control clears the whole-run form too, so the removal cost the reference nothing.
    assert non_collapsing(s_co2, floor=_DECADE_CO2_FLOOR) is False
    assert non_collapsing(f_co2, floor=_DECADE_CO2_FLOOR) is True
    # ⚠ THE COUNTERFACTUAL, and it REFUTED MY OWN HYPOTHESIS. I expected the two guards
    # to be two readings of ONE event (the year-2 trough), which would have made the
    # committed test's "both halves are asserted" independence claim wrong. Splice the
    # control's year 2 into the subject and change nothing else: the floor guard flips
    # True, stationarity stays False, because diffs[3] = series[5] - series[3] does not
    # involve year 2 at all. The committed claim stands and mine did not.
    #
    # ⚠ State it precisely, though: this does NOT make them causally separate events.
    # Year 3 (0.053922) is itself inside the same establishment dip. The accurate
    # statement is that the stationarity failure does not DEPEND on the single sub-floor
    # year, and that both guards fire exclusively within years 2-5.
    spliced = list(s_co2)
    spliced[2] = f_co2[2]
    assert non_collapsing(spliced[2:], floor=_DECADE_CO2_FLOOR) is True
    assert _stationary_verdict(spliced) is False
    assert all(v > _DECADE_CO2_FLOOR for v in s_co2[6:]), "clean from year 6 on"

    # --- 3. the MANIFEST-named liveness gate -----------------------------------------
    f_tail = f_leaf[_LIVENESS_TRANSIENT:]
    s_tail = s_leaf[_LIVENESS_TRANSIENT:]
    # validate the reconstruction against the gate's own committed 15-yr reading first.
    # ⚠ That committed 0.634352 is ``max(tail)``, NOT ``tail[-1]``: the tail declines
    # monotonically, so max(tail) == tail[0] == year 8. My first version of this check
    # compared the wrong element and went red on a correct reconstruction.
    f15_leaf = _reprice_series(LONG_HORIZON_YEARS, False)[1]
    assert max(f15_leaf[_LIVENESS_TRANSIENT:]) == pytest.approx(0.634352, abs=1e-5)
    assert f_leaf[-1] == pytest.approx(0.594984, abs=1e-5), "the anchored equilibrium"
    assert max(f_tail) > _LIVENESS_FLOOR, "control clears its own floor"
    assert max(s_tail) > _LIVENESS_FLOOR, "⚠ stem-only CLEARS it too — no third leg"
    assert max(s_tail) == pytest.approx(0.643676, abs=1e-5)
    assert s_leaf[-1] / _DEAD_BASELINE > 2.0, "and it is nowhere near the dead baseline"

    # --- 4. the plant is NOT free: read the CO2 gain WITH its cost -------------------
    assert s_stem[-1] / f_stem[-1] == pytest.approx(1.518, rel=2e-3), "stem UP"
    assert s_store[-1] / f_store[-1] == pytest.approx(0.882, rel=2e-3), "grain DOWN"
    assert s_store[-1] < f_store[-1], (
        "(C)-finding 8's 'stem up, grain down' holds in THIS scenario too"
    )
    # ⚠ the leaf sign does NOT transfer from open_season (-3.96 % there, +1.98 % here)
    assert s_leaf[-1] > f_leaf[-1]
    assert s_leaf[-1] / f_leaf[-1] == pytest.approx(1.0198, rel=2e-3)
