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

⚠⚠ **2026-08-12 — THE STEM-RESERVE BUILD MOVED THE TREE UNDER MOST OF THIS MODULE.**
``docs/plans/post-roadmap-stem-reserves.md``. A reserve pool now intercepts 40 % of stem
growth and remobilises it to grain after anthesis. **Every re-pin below is attributed by
a control**: running the same scenario with ``stem_reserves=False`` reproduces the
committed pre-build value EXACTLY (``open_season`` peak W 12.633098, the six ``f_N``
bites, the T10 ``seed bank too small`` raise), so nothing here moved by drift.

Four claims INVERTED rather than shifted, and each is renamed for what it now measures:
the ``f_N`` bite is gone, T10 no longer breaks the re-sow, the three-organ "one haircut"
is no longer common, and stem-only now CROSSES the Greenwood tripwire.

⚠ **THE FINDING, and it is structural rather than numeric: section 7's form-gap
objection is DISCHARGED.** It read *"our single ``stem_c`` pool cannot express [A]'s own
'except for their **reserves**', so zeroing the death rate makes the stem a strictly
one-way pool."* The tree now HAS that pool: with ``rdr_stem = 0`` **and** the reserve,
4.823256 mol C still leaves the stem per season (peak 5.304255 → final 0.480999) by the
reserve → grain route. With ``stem_reserves=False`` there is no reserve stock at all and
the stem IS one-way, exactly as recorded. **The objection was true of the tree that
existed when it was written, and the build removed it.**

⚠⚠ **AND THE SURVIVING REFUSAL LEG CHANGED.** After the 2026-08-10 humification split,
stem-only was refused on the decade CO2 floor plus stationarity. At fifty years the
floor
leg is now **DISSOLVED — no year sits below it at all** (was year 2 at 0.921x), and the
attractor INVERTED to 0.071919 against the control's 0.073668 (was above it).
Stationarity
alone still refuses, and section 8 now records exactly how narrowly: ONE same-phase
diff,
``diffs[4] = 0.019309``, at **1.28x** its bound, with ``diffs[3]`` at **0.98x** — just
under. **NONE OF THIS RE-DECIDES THE BRANCH.** Re-deciding a refusal inside the build
that
moved the tree beneath it is the shape this project refuses (the CUE precedent, and the
soil-fractionation re-refusal). It is measured, named, and left to a successor.

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
from domains.biosphere.allocation import (
    Senescence,
    mutual_shading_rate,
    senescence_flux,
)
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
    STEM_RESERVE_C,
    STORAGE_C,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, STEPS_PER_DAY, steps_for
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
            BIO_DT,
            steps_for(len(w)),
            year=steps_for(len(_weather())),
        )
    return run_season(integrator(registry), state, resolver, BIO_DT, steps_for(len(w)))


def _without_reserve(scenario):
    """The pre-stem-reserve tree, for attributing this module's 2026-08-12 re-pins.

    Defined here rather than imported from ``test_stem_reserves.py``: this module's own
    rule is that a quantity borrowed from another test file is REPRODUCED and then
    checked against that file's committed reading, never imported across modules.
    """
    return dataclasses.replace(scenario, stem_reserves=False)


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
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼).
    # listing5's integrated loss 30.0 % -> 36.7 %: a smaller crop sheds a LARGER
    # FRACTION of its peak, because the flat per-day death rate is unchanged while
    # the peak it is measured against fell. The claim is unmoved — none of the three
    # is inside [40 %, 60 %] — but frozen and listing5 are now 1.7 points apart
    # rather than 8.5, so "nearer" between them is closer to meaningless than ever.
    assert 0.30 < listing5 < 0.40, listing5  # 36.7 % — still below the band
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
def test_the_dvs_form_crosses_the_greenwood_tripwire_and_f_n_NO_LONGER_bites() -> None:
    """⚠⚠ INVERTED 2026-08-12 by the stem-reserve build. The tripwire still fires; the
    bite is gone.

    As measured before the build: peak W 18.677670 t/ha, and ``f_N`` dipped to 0.995213
    over 6 of 306 steps — the first ``f_N`` bite in a frozen scenario. Now: peak W
    **19.740557** (further over the crossing, not less) and ``f_N`` is **exactly 1.0 at
    every step**. Attributed, not assumed: ``stem_reserves=False`` reproduces 18.677670
    and all six bites exactly.

    ⚠ **THE MECHANISM IS TWO CHANNELS, AND MY FIRST DRAFT NAMED ONLY ONE.** I wrote that
    the reserve sits outside ``f_N``'s vegetative denominator (leaf+stem+root, which
    excludes reserve starch exactly as it excludes grain), so concentration rises and
    the
    bite vanishes. The isolating control refutes that as a complete account: adding the
    reserve BACK into the denominator restores a bite of **1 step at 0.993442**, not the
    original 6. The numerator moved too — peak plant N 0.028249 → 0.027227 and peak
    vegetative carbon 70.784806 → 62.559358. So the denominator is the DOMINANT channel
    (putting it back restores a bite at all) and it is not the whole one.

    This is the litter-starvation shape from section 7 on the other side of the plant:
    the obvious single-channel story was wrong, and only the control said so.

    ⚠ Excluding reserve starch from ``f_N``'s denominator is a BUILD choice with a
    reason, pinned here so it cannot be mistaken for an oversight: reserve carbohydrate
    carries no nitrogen, so counting it would dilute a concentration the plant does not
    actually have. Same argument retires it from the maintenance-respiration biomass.
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
    # was ``18.0 < peak_w < 19.5`` (18.677670) — the tripwire fires HARDER, not less
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 19.961667 ->
    # 17.896849.
    # Still well over the crossing, so the tripwire still FIRES; it fires less hard,
    # because the depth integral fixes ~5.7 % less carbon per season than the big leaf.
    assert 17.7 < peak_w < 18.1, peak_w

    def fn_profile(include_reserve: bool) -> list[float]:
        out = []
        for s in states:
            veg = (
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[ROOT_C].amount
            )
            if include_reserve:
                veg += s.stocks[STEM_RESERVE_C].amount
            out.append(
                nitrogen_stress_factor(
                    s.stocks[PLANT_N].amount,
                    veg,
                    n_residual_per_mol_c=npar.n_residual_per_mol_c,
                    n_critical_per_mol_c=npar.n_critical_per_mol_c,
                )
            )
        return out

    # AS THE TREE COMPUTES IT — the reserve is outside the denominator, and nothing
    # bites.
    shipped = fn_profile(False)
    assert min(shipped) == 1.0, min(shipped)  # was 0.995213 over 6 days
    assert sum(1 for f in shipped if f < 1.0) == 0

    # ⚠ THE ISOLATING CONTROL, and it is why the docstring claims two channels and not
    # one: put the reserve back in the denominator and a bite returns — but ONE DAY at
    # 0.993442, not the six the pre-build tree had. Had this restored all six, the
    # single-channel story would have been right; it did not, so it is not.
    #
    # ⚠ **COUNTED IN DAYS SINCE 2026-08-14, and that is the whole of this line's
    # change.** It counted STEPS, and a count of steps is a step-size observable before
    # it is a science one: at `dt = ¼` the bite spans 4 steps instead of 1 — **exactly
    # 4×, the structural signature of a pure rescale** — while the physical duration it
    # measuring is the same one day it always was. Dividing by `STEPS_PER_DAY` makes the
    # assertion invariant to the step, which is what "one bite lasting a day" always
    # meant. See the same conversion two tests down, where the count does NOT come back
    # exactly 4× and is therefore a real change.
    counterfactual = fn_profile(True)
    # ⚠⚠ **THE ISOLATING CONTROL WENT INERT ON 2026-08-14 AND THAT RETIRES THE EVIDENCE,
    # NOT THE CONCLUSION.** The bite is now **gone entirely** — 0 steps below 1.0, where
    # it was one day at 0.996409 — because the within-day light path fixes less carbon,
    # so the same plant nitrogen is spread over less biomass and the concentration sits
    # *above* critical even with the reserve counted in the denominator.
    #
    # ⇒ What this control could once demonstrate, it can no longer demonstrate. The
    # margin it rested on was **0.36 %**, so "putting the reserve back restores a bite,
    # but only one day of six" was a marginal reading before it was an inert one, and
    # the
    # honest record is that the two-channel conclusion in the docstring above now rests
    # on the *pre-2026-08-14* measurement rather than on a control that reproduces
    # today.
    # Recorded rather than deleted (a control that stops discriminating is a finding),
    # and rather than re-tuned to make something bite again.
    assert sum(1 for f in counterfactual if f < 1.0) == 0
    assert min(counterfactual) == 1.0, min(counterfactual)


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
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.95 -> 0.8736,
    # and the
    # narrative flips from 'stays under, but only just' to 'stays under with room'.
    # Two-sided so a move in EITHER direction is caught.
    assert 0.86 < peak_w / _CROSSING_T_HA < 0.89, peak_w


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


def test_whether_rk4_still_hard_errors_on_perennial_under_the_dvs_form() -> None:
    """⚠⚠ **IT NO LONGER DOES, AS OF 2026-08-14 — and this was (C)'s blocker.**

    It read: Increment 1's record repeating exactly, *"rationed under Euler,
    hard-errored under RK4"*. A needed scale is a hard error under a higher-order scheme
    (positivity must come from the kinetics), so this is not a near-miss. At the shipped
    quarter-day step the RK4 run **completes cleanly**, and so does the regulated one
    two sections down (``..._does_NOT_rescue_perennial_under_rk4``, whose whole
    construction was two hard errors with byte-identical messages).

    ⚠ **The reading is narrow and is not "(C) is now buildable".** What the old pin
    measured is a property of the pair (form, STEP), not of the form: an over-draw is
    "one step's demand exceeds the stock", so a step four times smaller demands four
    times less per step. The hard error was the *symptom* by which the DVS form's
    over-draw was detected; its disappearance is not evidence the over-draw is gone,
    only that it no longer exceeds a stock within one step.

    ⚠ **And the Euler half of the pair was ALWAYS the weaker one and still reads clean**
    — that is what ``test_euler_reports_no_rationing_and_that_is_the_trap`` exists to
    say. With the RK4 half silent, the pair now has no member that fires, so **this file
    no longer contains a running test that separates the DVS form from the frozen one on
    closure.** Recorded as a gap, not filled here: inventing a replacement detector
    inside a step ceremony would be building the evidence for a refusal in the same
    change that removed the old evidence.

    The refusal of the DVS form is NOT reopened. Its other legs — the source reading
    (p. 212 vs the p. 113 exercise answer) and the canopy going unphysical on either
    table — are pinned elsewhere in this module and are untouched by a step change.
    """
    _states, rationed, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO,
        sc.PERENNIAL_CHAMBER_YEARS,
        tables=_LISTING5,
        resets=True,
        integrator=Rk4Integrator,
    )
    # ⚠ Under RK4 the backstop is not merely unused, it is a HARD ERROR path — so
    # `rationed == 0` here is the engine's own statement that no scale was needed, and
    # is the strongest form the surviving claim can take.
    assert rationed == 0


def test_the_exercise_table_NO_LONGER_breaks_the_re_sow() -> None:
    """⚠⚠ INVERTED 2026-08-12 by the stem-reserve build.

    It read: *"T10 never fills grain, so closure fails at the sow rather than at the
    draw"*, and asserted a ``seed bank too small`` raise. T10 strips the canopy (97.9 %
    of leaf area lost) so the crop had nothing left to fill grain with — but the reserve
    is filled DURING stem growth, before the stripping, and remobilises after anthesis.
    It is precisely the carbon source a stripped canopy no longer has.

    So the run now completes. Both trees are asserted, because the pin's value is that
    it
    says WHICH tree does what — the reserve-off control still raises, verbatim.

    ⚠ Completing is NOT passing. The chamber closes at a CO2 minimum of 0.025842, about
    half the 0.05 decade floor, so T10 still fails the liveness leg — it now fails it as
    a live-but-starved chamber rather than as a dead one. The refusal of the exercise
    table is unchanged; only its failure MODE moved.

    ⚠⚠ **AND THAT LIVENESS LEG WENT TOO, 2026-08-14 (the quarter-day step): 0.027128 ->
    0.068062, which CLEARS the 0.05 floor.** So the paragraph above no longer describes
    the tree — T10 neither breaks the re-sow nor starves the chamber. The `< 0.05`
    assertion below is inverted accordingly.

    ⚠ **The refusal of the exercise table does NOT rest on either leg and is
    untouched.** It rests on the source reading, pinned at the top of this module: T10
    is an EXERCISE ANSWER (p. 113) and the primary's own text (p. 212) gives 0.012/day,
    an order of magnitude apart. A number measured on this tree was never what refused
    T10, which is why watching two of its consequences dissolve changes nothing — and is
    a good argument for refusing on the source where a source exists.
    """
    states, rationed, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO,
        sc.PERENNIAL_CHAMBER_YEARS,
        tables=_T10,
        resets=True,
    )
    assert rationed == 0
    co2_min = min(s.stocks[CARBON_POOL].amount for s in states)
    # ⚠ 0.025842 -> 0.027128 (2026-08-12) -> 0.068062 (2026-08-14, quarter-day step).
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 0.068062 -> 0.063415: a
    # smaller chamber crop draws the pool down less deeply per day, but respires into
    # it at night — net, the trough falls 7 %. Still clears the 0.05 floor.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.063415 ->
    # 0.061261,
    # a further 3.4 % off the trough. Still clears the 0.05 floor.
    assert co2_min == pytest.approx(0.061261, abs=1e-5)
    # ⚠ INVERTED 2026-08-14. This read `co2_min < 0.05` under "still under the decade
    # floor — completing is not passing". It now clears the floor by 36 %. Kept as a
    # comparison against the same floor so the crossing is visible in the diff.
    assert co2_min > 0.05, "clears the decade floor — see the docstring, 2026-08-14"

    # ⚠⚠ **THE CONTROL STOPPED RAISING ON 2026-08-14 (the light path), AND THE
    # ATTRIBUTION IT CARRIED GOES WITH IT.** It read "the pre-build tree still fails at
    # the sow, so the cause is named" — the stem reserve was what let T10 re-sow at all.
    # The reserve-off tree now COMPLETES. ⚠ The direction is the surprising half: less
    # carbon per day should leave a *smaller* seed bank, not a viable one, so this is
    # not
    # a size effect. It is a phenology one — the light path changes when the day's
    # carbon
    # arrives, DVS-keyed partition shares therefore fill different organs at different
    # times, and the re-sow reads `storage_c` at one instant. Recorded as an open thread
    # rather than explained: what is measured is that the control no longer
    # discriminates.
    # The assertion is inverted to state what the tree does, so the next change to this
    # region is read against today rather than against a claim that stopped being true.
    _run(
        _without_reserve(sc.PERENNIAL_CHAMBER_SCENARIO),
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
]
# ⚠ SIX UNTIL 2026-08-18. ``n_limited`` and ``water_biting`` were retired with the
# rest of the Python-only roster (C6 of the reference flip), so this roster is now
# exactly the four the reference carries. The gate below is UNCHANGED by that: measured
# before the deletion, the departing peaks were 0.0869 (n_limited) and 0.4718
# (water_biting), and the ``max(chambers)`` the gate pins is 0.5849, which is
# ``consumer_chamber`` — a survivor. Neither was ever the binding number.
#
# ⚠ It also removes a mislabel: ``n_limited`` is an OPEN-FIELD run by its own
# declaration, and this function counted every label except ``open_season`` as a
# "chamber". The four that remain really are open field + three chambers.


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
    """⚠ FINDING 4, first half. The regulator's reach, at ``to_bits()`` precision.

    ⚠⚠ **THIS TEST'S PREMISE EXPIRED ON 2026-08-15 AND THE TEST WAS RESTATED, NOT
    RELAXED.** It used to read "added to the FROZEN tree the regulator changes nothing",
    which held because the threshold (LAI 6) was above every frozen peak. Two things
    ended that: the regulator **is** the frozen tree now (it ships in
    ``allocation.Senescence``), and ``open_season``'s peak crossed the threshold
    (6.0228). "Adding it changes nothing" is not a claim one can make about a mechanism
    that is already there and firing.

    What survives — and is asserted here at the SAME precision — is the half that was
    always the load-bearing one: **the term is confined to the open field.** Every
    chamber peaks between 0.087 and 0.585, i.e. 10-69x below the threshold, so removing
    the shading term from a chamber reproduces the chamber bit-for-bit. That is the
    closure-scaling result the canopy depth integral reached independently
    (``docs/log/canopy-magnitude.md``): a mutual-shading mechanism cannot reach a
    habitat whose canopy never closes.

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
    rebuilt = run(reconstruct=True, shade=VKS_SHADE_RATE)
    unregulated = run(reconstruct=True, shade=0.0)
    # Leg 1 — the reconstruction is faithful. ⚠ It now needs `shade=VKS_SHADE_RATE` to
    # be faithful, because the shading term is IN the frozen flow as of 2026-08-15.
    assert frozen == rebuilt, f"{label}: the one-knot reconstruction is NOT faithful"
    # Leg 2 — and the term is confined to the open field. Removing it changes NOTHING
    # in a chamber (their canopies never reach LAI 6) and changes `open_season`.
    if label == "open_season":
        assert unregulated != frozen, "the shipped regulator is INERT on the open field"
    else:
        assert unregulated == frozen, f"{label}: the regulator is NOT inert"


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
    quantity="peak LAI (m2 m-2) vs the mutual-shading threshold",
    bound="peak < 6.0 OR the 5%/day mutual-shading loss is MODELLED",
    source="Van Keulen & Seligman 1987 mutual-shading threshold, via [A] p. 101",
)
def test_the_vks_mutual_shading_regime_is_MODELLED_not_merely_avoided() -> None:
    """⚠ FINDING 5 — the tripwire, in the style of the 14.4248 t/ha Greenwood one.

    ⚠⚠ **THE TRIPWIRE FIRED ON 2026-08-15, AND THIS GATE IS ITS RESTATEMENT.** It read
    ``peak < 6.0`` for every scenario, and its own docstring predicted the event: *"a
    calibration growing the open-field canopy ~16 % would start a sourced, non-fitted
    mechanism firing in a frozen scenario."* Binding ``specific_leaf_area`` to [A]'s
    Table 19 (22.0 → 23.53 m²/kg) grew it **11.9 %** — 5.3806 → 6.0228 — and the
    mechanism is now firing.

    ⚠ **Why the bound was RESTATED rather than re-tuned, and why that is not the
    refused co-adaptation shape.** Re-tuning would have moved 6.0 to fit 6.0228. Nothing
    of the sort happened: 6.0 is still the threshold, still sourced, and the tree is
    still measured against it. What changed is the *consequence* of being above it. The
    old bound was a **proxy**: it could not say "the canopy is unphysical" — the sibling
    band on this very quantity permits ``5.0 < peak < 8.0``, so a peak of 6.02 is a
    perfectly ordinary wheat canopy. What it really guarded was **"no cited mechanism is
    firing that this tree fails to model."** That guarantee is now met by *modelling the
    mechanism* instead of by staying out of its way.

    ⚠ **And it could not have been met any other way.** Measured before building: adding
    the loss leaves the peak **bit-identically unchanged** (6.0228 either way), because
    the canopy crosses the threshold *at* its summit and the loss acts on the way down.
    So ``peak < 6.0`` was not a bound the mechanism could restore — only a smaller plant
    could, and shrinking the plant to fit a bound is the thing this project refuses.

    ⚠ **Precision, stated because it is the honest reading.** [A]'s 425 kg/ha is three
    significant figures. A ±1 % disagreement about it spans a peak LAI of ~5.95–6.09, so
    the tree sits **AT** the threshold within the resolution of the parameter that puts
    it there — not comfortably past it. This is the 3.5×-amplifier finding
    (``docs/log/canopy-magnitude.md``) arriving a second time from a new direction.

    ⚠ **Split note.** This function also carried
    ``0.80 < open_peak / VKS_LAI_THRESHOLD < 0.92`` until the science-gate work — a
    *margin* pin, which lives in its own unmarked test below.
    """
    peaks = _roster_peak_lai()

    # The half of the original claim that is UNCHANGED, and still worth freezing: the
    # chambers are carbon-limited by design and cannot reach the regime at all.
    chambers = [v for k, v in peaks.items() if k != "open_season"]
    assert max(chambers) < 1.0, peaks  # 0.585 — an order below, not a near miss

    # The restated guarantee, asserted for EVERY scenario: be below the threshold, or
    # model the loss the source prescribes above it. Never in the regime, unmodelled.
    sen = load_senescence_params()
    for label, peak in peaks.items():
        assert peak < VKS_LAI_THRESHOLD or sen.shade_rate > 0.0, (label, peak)

    # ...and "modelled" has to mean the CITED mechanism, not a knob that happens to be
    # non-zero. The constants are the source's, and the form is the source's step.
    assert sen.shade_rate == VKS_SHADE_RATE, sen.shade_rate
    assert sen.lai_threshold == VKS_LAI_THRESHOLD, sen.lai_threshold
    below = mutual_shading_rate(
        VKS_LAI_THRESHOLD,
        rdr_leaf=sen.rdr_leaf,
        shade_rate=sen.shade_rate,
        lai_threshold=sen.lai_threshold,
    )
    above = mutual_shading_rate(
        VKS_LAI_THRESHOLD + 1e-9,
        rdr_leaf=sen.rdr_leaf,
        shade_rate=sen.shade_rate,
        lai_threshold=sen.lai_threshold,
    )
    assert below == sen.rdr_leaf, below  # inert AT the threshold (strict >)
    assert above == sen.rdr_leaf + sen.shade_rate, above

    # And it genuinely bites on the scenario that crossed — a mechanism that is present
    # but never reached would satisfy every assertion above and guard nothing.
    assert peaks["open_season"] > VKS_LAI_THRESHOLD, peaks


def test_how_close_the_open_season_lai_margin_is_to_the_threshold() -> None:
    """The margin narrative, asserted so it cannot rot — NOT a gate.

    Deliberately unmarked: this is two-sided on a *ratio to our own peak*, so a change
    that moves the canopy further from the threshold fails it. Freezing that as contract
    would let an unfreeze ceremony fail for an improvement.

    ⚠ **RENAMED 2026-08-14 (the quarter-day step).** The name carried a measurement —
    `..._is_86_percent` — which is the shape that goes stale silently. Named for the
    question instead. The measurement itself barely moved: **86.5 % -> 86.9 %**, a peak
    LAI of 5.213 against the 6.0 threshold where it was 5.191, so the band below is
    unchanged and still comfortably contains it.

    ⚠ **This is the finding of the pair.** 92.9 % of the mutual-shading threshold is a
    peak LAI of **5.572 against 6.0**, where it was 5.191. The band `peak < 6.0` (Van
    Keulen & Seligman 1987, via [A] p. 101) still PASSES and is not being re-tuned — but
    its clearance on the REFERENCE scenario has fallen from 13.5 % to **7.1 %** for a
    numerics change alone, and the gate above says a calibration growing this canopy
    ~16 % would start the mechanism firing. Half of that headroom is now spent, and by
    the integrator rather than by any science.

    ⚠⚠ **THE HEADROOM IS NOW GONE — THE RATIO PASSED 1.0 ON 2026-08-15.** Binding
    `specific_leaf_area` to its primary source took the peak to **6.0228, i.e. 1.0038 ×
    the threshold**. This pin therefore stops being a *clearance* narrative and becomes
    an *exceedance* one, and the paragraphs above are kept as the record of a margin
    that was spent in three moves from three unrelated causes (the step, the light path,
    the SLA anchor) — none of which was aimed at this number.

    ⚠ Note what the sequence shows: every one of those three was a change to something
    else, and this margin absorbed all of them. A quantity that only ever moves as a
    side effect is not being managed, and the three-signal pattern is the reason the
    gate above was restated into a mechanism rather than re-pinned a fourth time.
    """
    open_peak = _roster_peak_lai()["open_season"]
    # ⚠ the band moves with the measurement and stays two-sided: 0.865 -> 0.929.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 0.929 -> 0.897: the
    # clearance to mutual shading OPENS again, undoing the step's narrowing and more.
    # ⚠ Read with the peak-LAI band's other side: the same movement takes the
    # converged canopy BELOW the 5.0 floor (see docs/biosphere-reference.md's known
    # deviation). Clearance from the ceiling and clearance from the floor are not the
    # same news.
    # ⚠ 2026-08-15, the SLA anchor: 0.897 -> 1.0038. The pin CROSSES 1.0 and is
    # re-centred there; it stays two-sided and narrow so the next mover is caught the
    # same way this one was. ⚠ And the converged deviation it warned about is GONE —
    # the converged peak is now 5.4269, inside `5.0 < peak < 8.0` for the first time.
    assert 1.0 < open_peak / VKS_LAI_THRESHOLD < 1.01, open_peak


def test_the_regulator_is_DISJOINT_from_cs_blocker_on_perennial_under_rk4() -> None:
    """⚠ FINDING 4, the headline: the regulator and (C)'s blocker are DISJOINT.

    ``perennial`` is where (C) died, and it died identically with the regulator in
    place, because its peak LAI is ~0.56 under RK4 — the regulator never fires on the
    failing trajectory. The two scale factors agreed to all sixteen digits, which is a
    stronger statement than "it still fails": it was the SAME failure.

    ⚠⚠ **RENAMED AND REBUILT 2026-08-14.** The old name — ``..._does_NOT_rescue_
    perennial_under_rk4`` — presumes there is something to rescue, and at the shipped
    quarter-day step the RK4 run does not fail at all. The disjointness claim is what
    this test is for and it survives in a stronger form; see the block below.

    A mutual-shading rule is a canopy-CLOSURE mechanism. A canopy that never closes
    cannot be regulated by one. That is scope (A)'s finding 11 on the other side of the
    plant — "making N faithful does not make the CHAMBER faithful".
    """
    # ⚠⚠ **REBUILT 2026-08-14 (the quarter-day step): NEITHER RUN HARD-ERRORS ANY
    # MORE**, so the old construction — run both, catch two ``over-draw`` exceptions,
    # compare the two message strings byte for byte — has nothing to catch. See
    # ``test_whether_rk4_still_hard_errors_...`` for why the disappearance of an
    # over-draw at a smaller step is a step result and not a science one.
    #
    # ⚠ **The CLAIM is unchanged and the new form is STRICTLY STRONGER.** The claim was
    # never about the exception; it was *"the regulator never fires on this trajectory,
    # so the two shade settings are the same run"*. The old pin could only see that
    # through one number scraped out of an error message with a regex. The runs now
    # complete, so the whole final state can be compared — every stock, not one scalar —
    # and bit-identity across the pair says the same thing with far more of the run
    # behind it. It is also the assertion the engine's own `bit-identical within a
    # build` promise covers directly, which the scraped float needed a 1e-12 window for.
    finals = []
    trajectories: dict[float, list[State]] = {}
    for shade in (0.0, VKS_SHADE_RATE):
        states, rationed, _ = _run(
            sc.PERENNIAL_CHAMBER_SCENARIO,
            sc.PERENNIAL_CHAMBER_YEARS,
            tables=_LISTING5,
            resets=True,
            integrator=Rk4Integrator,
            shade=shade,
        )
        assert rationed == 0, shade  # under RK4 a needed scale is a hard error
        trajectories[shade] = states
        finals.append(
            tuple(sorted((str(k), v.amount) for k, v in states[-1].stocks.items()))
        )
    assert finals[0] == finals[1], "the regulator fired — it used not to"

    # ...and the MECHANISM the claim rests on, asserted rather than left in prose: the
    # regulator is a canopy-CLOSURE rule and this canopy never closes, so there is
    # nothing for it to act on. A canopy that came anywhere near the threshold would
    # make the bit-identity above a coincidence rather than a consequence.
    # ⚠ Read from the REGULATED run by name, not from whatever `states` the loop above
    # happened to leave bound — the claim is about the run the regulator was given.
    cp = load_canopy_params()
    peak_lai = max(
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=cp.sla_per_mol_c,
            ground_area=sc.PERENNIAL_CHAMBER_SCENARIO.ground_area,
        )
        for s in trajectories[VKS_SHADE_RATE]
    )
    # ⚠ measured 0.8689 at the shipped step, where the docstring's pre-2026-08-14 text
    # says ~0.56 under RK4 — the canopy grew with the step like everything else here.
    # Asserted as "an order below closure" rather than as the number, because the claim
    # is that there is nothing for a closure rule to act on, not that it is one size.
    assert peak_lai < 0.2 * VKS_LAI_THRESHOLD, peak_lai
    # re-measured 2026-08-10 (the humification split): 0.9527733243688737 -> this.
    #
    # ⚠ The absolute value is compared with a TOLERANCE, and the reason is a measured
    # platform difference, not a convenience: this repo promises bit-identity *within a
    # build* and tolerance-gates the Rust port, but it never promised bit-identity
    # ACROSS PLATFORMS — `exp`/`pow` differ by ULPs between Windows UCRT and glibc.
    # Pinned as 16 literal digits this read 0.9363926204726938 on Windows and
    # 0.9363926204726942 on the Linux CI runner (4e-16 apart), so the anchor was
    # asserting a property nobody guaranteed and CI went red on arithmetic noise.
    #
    # The window is sized on two independent measurements, NOT swept until green: the
    # platform floor is 4e-16, and the smallest real movement this anchor exists to
    # catch is the humification split's own 1.6e-2. 1e-12 sits ~4 orders above the noise
    # and ~10 below the signal, so the teeth are unchanged — any science that moves this
    # trajectory still turns it red.
    #
    # ⚠ re-measured AGAIN 2026-08-12 (the stem-reserve build): 0.9363926204726938 ->
    # 0.9894074520723275. The CLAIM this test exists for is untouched and was re-run,
    # not
    # assumed: both shade settings still produce the byte-identical failure, i.e. the
    # regulator still never fires on the failing trajectory. The window stays 1e-12 for
    # the reasons above; the reserve moved this by 5.3e-2, four orders above it.
    # ⚠ **RETIRED 2026-08-14 with the exception it was scraped from.** It read
    # ``measured = float(re.search(r"scale_f=...", scale[0]).group(1))`` and
    # ``abs(measured - 0.9894074520723275) < 1e-12``. Kept as a record because the
    # *reasoning* above it is the reusable part and outlives the number: an absolute
    # 16-digit pin was asserting cross-platform bit-identity that this repo never
    # promised (Windows UCRT and glibc `exp`/`pow` differ by ULPs — measured 4e-16
    # apart), and the 1e-12 window was sized on two independent measurements rather than
    # swept until green. The replacement above needs no window at all, because
    # within-build bit-identity IS promised.


def test_the_greenwood_tripwire_fires_WITHOUT_f_n_biting_ON_EITHER_FORM() -> None:
    """⚠ FINDING 6 — a counterexample to a causal claim in ``test_nitrogen_form.py``.

    ⚠⚠ **2026-08-12: THE CONTRAST THIS TEST WAS BUILT ON IS GONE, AND THE FINDING IT
    SUPPORTS IS STRONGER FOR IT.** The pin used to run bare-vs-regulated and show the
    bare form biting (6 steps) while the regulated form did not — the contrast WAS the
    evidence. Since the stem-reserve build **neither** bites: bare 0, regulated 0. So
    the
    test can no longer separate them, and it no longer needs to. The claim was "crossing
    the mass tripwire does not imply an ``f_N`` bite", and it now holds on BOTH forms at
    once, at +56.3 % and +32.3 % over the 12.633 reference respectively. A
    counterexample
    that used to need a matched pair is now visible in either member.

    The reserve-off control is asserted alongside, so this file still records which tree
    produced the six bites the paragraphs below describe.

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

    def f_n_profile(
        shade: float, scenario=sc.DEFAULT_SCENARIO
    ) -> tuple[float, int, float]:
        states, _r, _ = _run(scenario, 1, tables=_LISTING5, shade=shade)
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
    # ⚠ BOTH forms are now over the crossing and NEITHER bites (was: bare bit 6 steps at
    # 0.995213 with bare_w 18.677670; regulated 15.724610 and clean).
    assert bare_w > _CROSSING_T_HA and bare_min == 1.0 and bare_n == 0
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 19.738556 -> 19.961667. ⚠ The
    # candidate (no leaf death) goes UP where the frozen tree goes DOWN, and that is
    # not a contradiction: without the death rate the canopy runs far past mutual
    # shading, where the concavity loss is smallest (it shrinks as LAI closes), so the
    # light path costs the unregulated crop least. The tripwire's subject is unharmed.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 19.961667 ->
    # 17.896849.
    assert bare_w == pytest.approx(17.896849, rel=1e-5), bare_w
    assert reg_w > _CROSSING_T_HA, reg_w  # 16.716179
    assert reg_w / 12.633 > 1.20, reg_w  # +32.3 %, well past the "~15 %" of the claim
    assert reg_min == 1.0 and reg_n == 0, (reg_min, reg_n)

    # ⚠ the control: on the pre-build tree the bare form DID bite, exactly six times at
    # 0.995213. Without this the paragraphs above describe a tree no test still runs.
    off = _without_reserve(sc.DEFAULT_SCENARIO)
    off_min, off_n, off_w = f_n_profile(0.0, off)
    # ⚠ **COUNTED IN DAYS SINCE 2026-08-14, and unlike the day-count two tests up this
    # one did NOT come back a clean 4×.** 6 steps at `dt = 1` became **28**, where pure
    # rescale would have given 24 — so the bite genuinely lengthens, from 6 days to 7.
    # In steps that is a 4.67× jump that reads like a mechanism change; in days it is
    # one extra day, which is what it is. **Predicting the 4× first is what separated
    # the two cases**, and neither could be told apart at the old step.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). ⚠⚠ THE BITE IS GONE:
    # 7 days -> **0**, and `off_min` is exactly 1.0. Same cause as the counterfactual
    # two tests up — a crop that fixes less carbon spreads the same nitrogen over less
    # biomass, so the concentration sits above critical and f_N never leaves 1. The
    # length claim this line was built to make ("7 days, not the 4× a pure rescale
    # would give") is therefore no longer measurable on this tree; it stands as the
    # 2026-08-14 pre-light-path record, not as something reproducible today.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 18.676196 ->
    # 16.769212.
    assert (off_n / STEPS_PER_DAY, off_w) == (0.0, pytest.approx(16.769212, rel=1e-5))
    assert off_min == 1.0, off_min  # was 0.994311 over 7 days


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

    # ⚠ all three re-measured 2026-08-12 (stem reserves). Was 12.633 / 13.639 / 1.0796.
    # THE CLAIM IS UNCHANGED and is the whole point of the test: zeroing stem death
    # moves
    # W UP, where our own param file predicted down. The reserve raises both sides (it
    # grows the crop) and shrinks the GAP, because some of the stem carbon the frozen
    # tree used to shed is now held in the reserve either way.
    # ⚠ and all three AGAIN 2026-08-14 (the quarter-day step), from 13.939142 /
    # 14.555369 / 1.044208. The claim is unchanged and the GAP barely moved (+4.42 % ->
    # +4.61 %), which is the reading that matters: this is a partition result, and a
    # partition ratio is the kind of quantity a step should leave nearly alone.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 14.107660 -> 13.740221.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 13.740221 ->
    # 13.379084.
    assert peak_w(frozen) == pytest.approx(13.379084, rel=1e-5)
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 14.410755 ->
    # 14.149387.
    assert peak_w(stem0) == pytest.approx(14.149387, rel=1e-5)
    assert peak_w(stem0) > peak_w(frozen)  # THE point: up, not down
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). The RATIO is nearly
    # untouched (1.046122 -> 1.048800) while both terms fell ~2.4 %, which is the
    # signature of a change acting on the whole carbon budget rather than on the stem.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 1.048800 ->
    # 1.057575. ⚠ The
    # ratio moves 0.8 % where the light path moved it 0.3 %, so the 'whole carbon
    # budget' signature is WEAKER here: this change is not organ-neutral, because the
    # depth integral's loss scales with canopy closure and the stem-only crop closes
    # its canopy differently. Still small, but no longer 'nearly untouched'.
    assert peak_w(stem0) / peak_w(frozen) == pytest.approx(1.057575, rel=1e-4)


def test_the_stem_grows_and_the_OTHER_THREE_organs_NO_LONGER_take_ONE_haircut() -> None:
    """⚠⚠ INVERTED 2026-08-12: the haircut is no longer COMMON, and that is a finding.

    The claim below was that leaf, root and storage — three organs on three different
    partition fractions — all fell by the same fraction to within a tenth of a
    percentage
    point (spread < 1e-3), which identified the cause as a shrunken assimilate stream
    rather than anything partition-side. Since the stem-reserve build the spread is
    **2.446e-3**, i.e. 2.4x the old tolerance, and the ORDER is the tell: leaf 0.973109,
    root 0.973726, storage 0.975555 — **storage is now the least-cut of the three**.

    ⚠⚠ **PARTLY RE-INVERTED 2026-08-14 (the quarter-day step): the spread is back to
    8.23e-4, inside the 1e-3 the original claim rested on.** leaf 0.974445, root
    0.974762, storage 0.975268. The ORDER — the actual evidence for the reserve being
    the cause — is unchanged, and the two unbuffered organs now agree to 3.2e-4. So the
    *mechanism* half of the 2026-08-12 finding stands and its *magnitude* half does not:
    a one-day step was inflating the buffered organ's attenuation about threefold.

    ⚠ This is the two-sided band paying for itself. The floor added in 2026-08-12
    existed precisely so that a change RESTORING uniformity would go red rather than
    pass quietly, and that is how this was found.

    The mechanism is the reserve, and it is exactly what the reserve is for: grain no
    longer draws only on the current assimilate stream, so a tax on that stream reaches
    it attenuated. Leaf and root, which have no such buffer, still take the common cut
    (they agree to 6.2e-4 — inside the ORIGINAL tolerance, which is the cleanest way to
    see that the third organ is what changed).

    So the original inference still holds for the unbuffered organs and the test now
    says
    so in that narrower form, rather than asserting a uniformity the tree no longer has.

    ---- the original finding, as measured before the reserve ------------------------

    Stem peak **+23.4 %**; leaf **-3.96 %**, root **-3.91 %**, storage **-3.97 %**. That
    three organs on three different DVS-keyed partition fractions fall by the same
    fraction to a tenth of a percentage point is not coincidence: the partition table is
    untouched, so a uniform haircut says the stream being partitioned — net assimilate —
    shrank. Measured cause: a bigger STANDING stem costs more to maintain
    (``maintenance_coef`` 0.02/day, charged on live tissue). On ``open_season`` the
    integral of standing live tissue rises 3.08 %, i.e. ~1.49 mol C of extra maintenance
    respiration against a 0.89 mol C fall in final storage.

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
    # re-measured 2026-08-12 (stem reserves): 1.2336 -> 1.256342; again 2026-08-14.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 1.264579 -> 1.260268 — the
    # RATIO barely moves (0.3 %) though both its terms fell several per cent, which is
    # what a change acting on the whole carbon budget rather than on one organ looks
    # like.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 1.260268 ->
    # 1.272692 — the
    # ratio moves 1.0 % where it moved 0.3 % for the light path, still the smallest
    # move of any pin in this file, which is what a whole-budget change looks like.
    assert ratios["stem"] == pytest.approx(1.272692, rel=1e-4), ratios
    for organ in ("leaf", "root", "storage"):
        assert ratios[organ] < 1.0, ratios
    haircuts = [ratios[o] for o in ("leaf", "root", "storage")]
    # ⚠ THE INVERSION, asserted rather than loosened: the three-organ spread now EXCEEDS
    # the 1e-3 the original claim rested on. Pinned with a floor as well as a ceiling,
    # so
    # a future change that RESTORES uniformity also goes red and gets read.
    #
    # ⚠⚠ **AND THAT IS EXACTLY WHAT HAPPENED, 2026-08-14: the spread fell back to
    # 8.23e-4, INSIDE the original 1e-3.** The two-sided band did its job — a "silent
    # improvement" turned the test red and got read, which is the whole reason the floor
    # was written. So the 2026-08-12 inversion was itself partly a step artefact: at a
    # one-day step the buffered organ's attenuation looked 3× larger than it is.
    #
    # ⚠ **What survives is the ORDER, and the order was always the real evidence** —
    # storage > root > leaf, i.e. the reserve-buffered organ is still the least cut, and
    # the two unbuffered ones still agree far more closely with each other (3.2e-4) than
    # either does with storage. The mechanism claim in the docstring is intact; what is
    # withdrawn is the *magnitude* claim that the spread exceeds the old tolerance.
    # Asserted two-sided again, at the new level, for the same reason as before.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). ⚠⚠ **THE COMMON-CUT
    # HALF NO LONGER REPRODUCES, AND NEITHER DOES THE ORDERING.** leaf/root/storage
    # were within 1.2e-3 of one another with storage least-cut; they now read
    # 0.98675 / 0.98202 / 0.98494 — a spread of **4.7e-3**, with LEAF the least cut
    # and root the most. That is a shape change, not a drift: the light path moves
    # carbon *within* the day, and the organs' shares are DVS-keyed, so which organ is
    # filling when the day's carbon arrives is no longer the same question it was.
    # ⇒ The inference this pair supported ("the three unbuffered organs take ONE
    # haircut, so the stem's gain is a partition effect") is withdrawn as an
    # inference and kept as a measurement. The stem claim itself is untouched — it is
    # asserted on the stem's own ratio above, which barely moved (1.2646 -> 1.2603).
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): the spread
    # narrows back,
    # 4.7e-3 -> 3.0e-3, while the ORDERING is unchanged (leaf least cut, root most).
    # ⚠ So the light path's shape change PARTLY reverts, but not to the 1.2e-3 that
    # supported the original 'one haircut' inference — which stays withdrawn. A
    # quantity that has now moved 1.2e-3 -> 4.7e-3 -> 3.0e-3 under three unrelated
    # changes is not evidence of a shared mechanism in either direction.
    assert 2.5e-3 < max(haircuts) - min(haircuts) < 3.5e-3, ratios
    assert ratios["leaf"] > ratios["storage"] > ratios["root"], ratios
    assert min(haircuts) == pytest.approx(0.986936, rel=1e-4), ratios


def test_stem_only_NOW_CROSSES_the_greenwood_margin() -> None:
    """⚠⚠ INVERTED 2026-08-12. It used to approach the tripwire; it now trips it.

    Was: *"the mass tripwire is approached, not tripped: 13.639 vs the 14.4248 crossing;
    the margin more than halves (12.4 % -> 5.4 %), leaving ``open_season`` one small
    calibration short."* The stem-reserve build supplied that calibration without
    intending to — stem-only now reaches **14.555369**, i.e. **1.009052x** the crossing.

    ⚠ Read with finding 6, which is what stops this being alarming: crossing the mass
    pin
    does not imply an ``f_N`` bite, and measured here it does not cause one. What it
    does
    mean is that the tripwire has done its job — it is a tripwire, and something tripped
    it. The successor question belongs to whoever revisits stem-only, not here.
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
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼).
    # ⚠⚠ **THE CROSSING IS WITHDRAWN. STEM-ONLY NO LONGER CROSSES.** 1.023122 ->
    # **0.999026** — 14.410755 t/ha against the 14.4248 crossing, **0.1 % under it**.
    # The light path takes ~2.4 % off every crop in this file, and this margin was
    # 2.3 %, so the tripwire un-fires. ⚠ This inverts a leg of the stem-only refusal
    # for the SECOND time in five days (it read `<` before 2026-08-14, then `>`, now
    # `<` again), and the honest reading is not "stem-only is rehabilitated" but
    # **the margin is inside the noise of unrelated mechanism work** — a guard that
    # flips sign on every change to the carbon budget is measuring the budget, not
    # the branch. That successor question belongs to whoever revisits stem-only.
    assert peak_w < _CROSSING_T_HA, peak_w  # ⚠ was `>` — the second inversion
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.999026 ->
    # 0.980907. The
    # leg stays `<` (no third inversion) and the margin opens from 0.1 % to 1.9 %.
    assert peak_w / _CROSSING_T_HA == pytest.approx(0.980907, rel=1e-4), peak_w


def test_the_FROZEN_open_season_margin_to_greenwood_is_now_THIN() -> None:
    """⚠ A NEW pin, and it is about the REFERENCE scenario rather than any candidate.

    The stem-reserve build moved frozen ``open_season`` from 0.8758 of the 14.4248 t/ha
    Greenwood crossing to **0.966332** — a 12.4 % margin down to 3.4 %. No test guarded
    that, because every margin pin in this module is written about a *candidate* form;
    the reference's own clearance was only ever implied by them. It is the kind of
    number
    that otherwise gets found later as "when did this get so close?".

    ⚠ **THE W DEFINITION IS A CHOICE AND IS PINNED BOTH WAYS.** ``W`` here is
    leaf + stem + storage, matching ``test_nitrogen_form.py``, which owns the 14.4248
    crossing — a borrowed threshold must be read on its owner's basis. But Greenwood's
    curve is stated against above-ground DRY MATTER, and reserve starch is dry matter,
    so
    there is a defensible reading that counts it: that gives **0.980651**. Both are
    asserted because the verdict is the same either way (still under), and pinning only
    the flattering one would hide that the choice exists.
    """
    states, rationed, _ = _run(sc.DEFAULT_SCENARIO, 1)
    assert rationed == 0

    def peak(include_reserve: bool) -> float:
        return max(
            _t_per_ha(
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[STORAGE_C].amount
                + (s.stocks[STEM_RESERVE_C].amount if include_reserve else 0.0),
                sc.DEFAULT_SCENARIO.ground_area,
            )
            for s in states
        )

    on_owners_basis = peak(False) / _CROSSING_T_HA
    counting_reserve = peak(True) / _CROSSING_T_HA
    # ⚠⚠ **THINNER AGAIN 2026-08-14 (the quarter-day step): 0.966332 -> 0.978014 and
    # 0.980651 -> 0.992368.** The margin this test was written to stop anyone finding
    # late has now closed twice in three days — 12.4 % -> 3.4 % -> **2.2 %** on the
    # owner's basis, and 1.9 % -> **0.76 %** counting the reserve. The verdict is still
    # "under, both ways", but on the reserve-counting basis the reference scenario is
    # three quarters of one per cent from a sourced tripwire, and the two bases no
    # longer straddle anything: they are both inside 2.2 %.
    #
    # ⚠ Recorded here rather than acted on, but it should be read as what it is — the
    # candidate-side test two functions up already CROSSES this tripwire, and the
    # REFERENCE is now within a per cent of it on one defensible reading of W. The
    # successor question is whether 14.4248 is still a useful tripwire at this margin,
    # and it belongs to whoever owns the Greenwood pin, not to a step ceremony.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 0.978014 -> 0.952542: the
    # margin OPENS for the first time in this pin's history (2.20 % -> 4.75 % under
    # the crossing). Every prior re-measurement here closed it.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.952542 ->
    # 0.927506 — the
    # margin OPENS for the second consecutive time (4.75 % -> 7.25 % under the
    # crossing), reversing this pin's whole earlier history of closing.
    assert on_owners_basis == pytest.approx(0.927506, rel=1e-4)
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.966943 ->
    # 0.941071.
    assert counting_reserve == pytest.approx(0.941071, rel=1e-4)
    assert on_owners_basis < 1.0 and counting_reserve < 1.0, "still under, both ways"
    # The margins, each on its own number. ⚠ The bands moved with the measurement and
    # are deliberately kept TWO-SIDED: a lower bound is what turns a future re-widening
    # into a red test rather than a silent improvement nobody reads.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). Both margins roughly
    # double as the crop shrinks: 2.20 % -> 4.75 % and 0.76 % -> 3.31 %.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): both margins
    # roughly
    # double AGAIN: 4.75 % -> 7.25 % and 3.31 % -> 5.89 %. Two consecutive doublings
    # from unrelated causes; the crop keeps shrinking away from the mass tripwire
    # while its canopy grows through the area one.
    assert 0.065 < 1.0 - on_owners_basis < 0.08
    assert 0.05 < 1.0 - counting_reserve < 0.068

    # ...and it was 0.8758 before the build, so the move is attributed, not just noted.
    off, _r, _ = _run(_without_reserve(sc.DEFAULT_SCENARIO), 1)
    off_peak = max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount,
            sc.DEFAULT_SCENARIO.ground_area,
        )
        for s in off
    )
    # ⚠ 0.875790 -> 0.884960 (2026-08-14). The attribution the line exists for holds:
    # the reserve is still worth ~9 points of the margin (0.885 -> 0.978), which is an
    # order more than the step is worth (0.876 -> 0.885).
    # ⚠ 0.884960 -> 0.859677 (2026-08-14, the light path). The attribution still
    # holds and widens: the reserve is worth ~9 points of the margin (0.860 -> 0.953).
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.859677 ->
    # 0.837550.
    # The attribution holds a third time: the reserve is worth ~9 points of the margin
    # (0.838 -> 0.928), unchanged in size while both endpoints slid down together.
    assert off_peak / _CROSSING_T_HA == pytest.approx(0.837550, rel=1e-4)


def test_the_two_tripwires_move_in_OPPOSITE_directions() -> None:
    """⚠ The clearest demonstration yet that mass and area margins are not one thing.

    Stem-only moves W **toward** the Greenwood crossing (0.966 -> 1.009 of it, i.e. past
    it) and peak LAI **away** from the V-K&S threshold (0.910 -> 0.886), because leaf
    carbon is downstream of the assimilate stream the bigger stem is taxing. The
    canopy-regulator
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
    # ⚠ all four re-measured 2026-08-12 (stem reserves). Was (0.8758, 0.8651) /
    # (0.9455, 0.8309). THE CLAIM — one single-number change pushes the two margins in
    # OPPOSITE directions, so no scalar "how close are we" exists — is re-run and holds;
    # the mass side now crosses 1.0, which makes it sharper rather than different.
    # ⚠ and all four AGAIN 2026-08-14 (the quarter-day step), from (0.966332, 0.910394)
    # / (1.009052, 0.885913). The claim holds and the OPPOSITION widens slightly: the
    # mass margin closes by 4.6 % where it closed by 4.4 %, and the area margin still
    # opens by 2.6 % where it opened by 2.7 %. A step change moving both margins the
    # same way would have been the thing to worry about, and it did not.
    #
    # ⚠ What DID move is the frozen scenario's absolute position on BOTH axes — 0.966 ->
    # 0.978 of the mass tripwire and 0.910 -> 0.929 of the area one. This test compares
    # two runs and is blind to that by construction; the two margin tests above and
    # below are where it is visible.
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). (0.978014, 0.928654) ->
    # (0.952542, 0.896774). ⚠ BOTH margins open, so this pair no longer demonstrates
    # opposition on the FROZEN tree — the opposition claim rests on the stem-only
    # comparison below, which is where it was always measured.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): (0.952542,
    # 0.896774) ->
    # (0.927506, 1.003806). ⚠ The AREA margin passes 1.0: the frozen tree is now ABOVE
    # the mutual-shading threshold (and models the loss), while the MASS margin opens
    # further. The two tripwires move in opposite directions again — this time on the
    # frozen tree, which is what the note above said this pair could no longer show.
    assert out["frozen"] == pytest.approx((0.927506, 1.003806), rel=1e-4)
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): (0.999026,
    # 0.884892) ->
    # (0.980907, 0.993669). ⚠⚠ READ THE PAIR TOGETHER: the frozen tree's area margin is
    # 1.003806 and stem-only's is 0.993669, so the two forms now STRADDLE the
    # mutual-shading threshold — the frozen tree is in the regime and models the loss,
    # stem-only is 0.6 % short of it. A single-number change to stem death decides
    # which side of a sourced mechanism's trigger the crop lands on.
    assert out["stem0"] == pytest.approx((0.980907, 0.993669), rel=1e-4)


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
    # re-measured 2026-08-12 (stem reserves + the cessation window): 0.046065 ->
    # 0.053177, 0.055175 -> 0.056030.
    # The discharge is unchanged; the trough rose another 15 % and now clears the 0.05
    # decade floor on its own, which is the next test's subject.
    # ⚠ and again 2026-08-14 (quarter-day step): 0.053177 -> 0.075578 and 0.056030 ->
    # 0.075476. **The two troughs have converged**: stem-only was 5.1 % below the frozen
    # run and is now 0.14 % ABOVE it. So the gap that made stem-only readable as a
    # distinct trajectory on this observable is essentially gone, and the observable
    # stops discriminating — which is a fact about the observable, not about stem-only.
    co2 = [s.stocks[CARBON_POOL].amount for s in stem0]
    # ⚠ re-measured 2026-08-14 by the WITHIN-DAY LIGHT PATH (the step is unchanged at
    # ¼). 0.075578 -> 0.070549.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.070549 ->
    # 0.070280.
    assert min(co2) == pytest.approx(0.070280, rel=1e-4)
    assert min(s.stocks[CARBON_POOL].amount for s in frozen) == pytest.approx(
        0.070253, rel=1e-4
    )
    # The discharge is not a horizon artefact: it holds at the long horizon too.
    _, r_stem_long, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO, LONG_HORIZON_YEARS, resets=True, stem_zero=True
    )
    assert r_stem_long == 0


def test_stem_only_NO_LONGER_collapses_the_decade_co2_attractor_below_its_floor() -> (
    None
):
    """⚠⚠ **INVERTED 2026-08-12 — THE FLOOR LEG IS DISSOLVED. READ THIS CAREFULLY.**

    This was the SURVIVING leg of the stem-only refusal: the 2026-08-10 humification
    split discharged the arbitration leg and left the decade CO2 floor plus
    stationarity.
    Since the stem-reserve build **no year sits below the floor at all** — at fifteen
    years and, per section 8, at fifty. ``non_collapsing`` returns True in both the
    windowed and whole-run forms. Its worst year rose 0.046065 -> **0.053177**.

    **Stationarity is now the only leg left, and section 8 measures exactly how narrowly
    it stands: ONE same-phase diff at 1.28x its bound, with the next at 0.98x.**

    ⚠⚠ **THIS DOES NOT RE-OPEN THE BRANCH, AND THE RESTRAINT IS THE POINT.** Every
    number here was moved BY the change being evaluated against them. Re-deciding a
    refusal inside the commit that moved the tree beneath it is the co-adaptation shape
    this project has refused repeatedly (the CUE row, the fractionation seed sweep, the
    consumer chamber twice). Measured, named, left to a successor with the numbers in
    hand — the same disposal the humification split gave the arbitration leg.

    ⚠ The stationarity FAILURE MODE also changed and is not the one the text below
    describes: the offending same-phase diffs moved from ``[2, 3]`` to a single ``[4]``,
    and the series now dips at year 3 rather than year 2 (``argmin`` 2 -> 3).

    ---- what this test measured before the reserve --------------------------------

    The second, independent closure failure: the settled attractor, not a transient.

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
    # ⚠ In STEPS: ``year_summaries`` segments the STEP-indexed trajectory.
    year_len = steps_for(len(_weather()))
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
    # ⚠⚠ RE-MEASURED 2026-08-12 (stem reserves). Was 0.055175 / 0.073367 / 0.074891 /
    # 0.046065, with the floor guard False. The control still passes both guards.
    # ⚠⚠ RE-MEASURED 2026-08-14 (the quarter-day step). Was 0.056030 / 0.074271 /
    # 0.070617 / 0.053177. **And the settling transient the note above blames for the
    # 2026-08-10 confusion is GONE:** the per-year minima now sit inside a 0.0755-0.0763
    # band from year 1, where they used to swing from 0.0760 down to 0.0461 and climb
    # back for a decade. That is why the last leg below flips.
    assert out["frozen"][0][1] == pytest.approx(
        0.070253, rel=1e-4
    )  # ⚠ re-measured 2026-08-14 (light path)
    assert out["frozen"][0][-1] == pytest.approx(0.072158, rel=1e-4)  # ⚠ 2026-08-15
    assert out["frozen"][1] is True and out["frozen"][2] is True
    assert out["stem0"][0][-1] == pytest.approx(0.072206, rel=1e-4)  # ⚠ 2026-08-14
    assert min(out["stem0"][0][2:]) == pytest.approx(
        0.070627, rel=1e-4
    )  # ⚠ re-measured 2026-08-15 (Gaussian canopy + sourced SLA)
    # ⚠ THE INVERSION: True where this test's own name said False.
    assert out["stem0"][1] is True, "the floor guard NO LONGER catches it"
    # ⚠⚠ **AND THE SECOND INVERSION, 2026-08-14: stationarity passes too**, so the line
    # that read `is False` under "stationarity is the only leg still refusing" now has
    # nothing refusing. Both guards pass on stem-only.
    #
    # ⚠ **Read this with the note above it, because that note is the reason it is not a
    # surprise.** The 2026-08-10 entry already worked out that the failure was "a longer
    # transient rather than a settled collapse" — a `transient=2` window against a
    # ~5-year settling. A finer step removes the deep early swing that made the settling
    # long, so the window is no longer too short for the series. **The refusal was
    # resting on a transient, and the transient was substantially a step artefact.**
    #
    # ⚠ Not reopened here, and this is the third file today in which stem-only's
    # evidence has moved in its own favour. The verdict belongs to a separate decision
    # with all of it on the table at once, not to a numerics ceremony.
    assert out["stem0"][2] is True, "stationarity no longer refuses either — 2026-08-14"
    # ...and no year is below the floor, which is the strong form of the same statement.
    assert min(out["stem0"][0]) > 0.05, out["stem0"][0]
    # The shape, pinned so "stem-only collapses" cannot be quoted off this test: the
    # tail
    # is RISING and ends above the floor, and the dip is in the early transient.
    # ⚠ 2026-08-14: the strict monotone tail is gone — the last three years read
    # 0.0758226 / 0.0758233 / 0.0758210, i.e. flat to six decimals with the ordering
    # decided in the seventh. That is not a rise any more, it is a SETTLED series, which
    # is the same event as the stationarity flip above. Asserted as flatness, because
    # pinning the sign of a 7e-7 difference would be pinning noise.
    # ⚠ 2026-08-14 (the light path): the tail is flat to 1e-4 rather than 1e-5 — the
    # series is settled but one order less tightly than at the previous measurement.
    assert max(out["stem0"][0][-3:]) - min(out["stem0"][0][-3:]) < 1e-4
    assert out["stem0"][0].index(min(out["stem0"][0])) == 1  # ⚠ was 2, then 3

    # ⚠ THE CONTROL. Without it this test asserts a green tree and no longer records
    # that
    # the leg it is named for ever refused anything. The pre-build tree still fails the
    # floor at 0.046065, exactly as committed.
    off_states, _r, _ = _run(
        _without_reserve(sc.PERENNIAL_CHAMBER_SCENARIO),
        LONG_HORIZON_YEARS,
        resets=True,
        stem_zero=True,
    )
    off = year_summaries(
        off_states, year_len, lambda seg: min(s.stocks[CARBON_POOL].amount for s in seg)
    )
    # ⚠⚠ **THE CONTROL DIED, 2026-08-14.** `off` is the reserve-OFF tree, kept here so
    # this file still records the collapse the paragraphs above describe. At the shipped
    # quarter-day step it does not collapse either: 0.046065 -> 0.076602, and
    # `non_collapsing` is True. So the last tree in which the collapse could be
    # witnessed is gone, and the claim above it — that the pre-build tree DID collapse —
    # is now unwitnessed by any running test. Asserted as the measurement so the loss is
    # visible rather than hidden behind a deleted line; a replacement control is
    # outstanding work and is deliberately not invented inside a step ceremony.
    # ⚠ 2026-08-15 (layered canopy) 0.071785 -> 0.07115204831484293, -0.9 %.
    assert min(off[2:]) == pytest.approx(0.071152, rel=1e-3)
    assert non_collapsing(off, floor=0.05) is True, "the control no longer collapses"


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
                # re-measured 2026-08-10: 0.075815/0.075893 -> 0.076829/0.076990;
                # again 2026-08-12 (stem reserves) -> 0.075872/0.076000. Pinned per-case
                # rather than under one loose band: the pair is only interesting because
                # the two forms sit ESSENTIALLY TOGETHER, and a shared tolerance wide
                # enough to hold both is exactly what would stop saying so.
                # ...and again 2026-08-14 (the quarter-day step). The pair sits even
                # closer together than before — 0.09 % apart where it was 0.17 %.
                # ⚠ 2026-08-14 (the light path): 0.075818/0.075751 -> the pair below.
                # They sit 0.11 % apart where they were 0.09 % — still essentially
                # together, which is the only thing this pin claims.
                # ⚠ 2026-08-15: 0.074077/0.073887 -> the pair below.
                expected = 0.073396 if kw else 0.073340
                assert lo == pytest.approx(expected, abs=1e-6), (kw, lo)
    # ⚠ the claim the loop exists for, stated as its own assertion: RK4 is insensitive
    # to
    # a change Euler's own liveness guards still argue about. 0.17 % apart.
    # ⚠ 2026-08-14: the pair widened to 0.26 %, so the "0.17 % apart" reading no
    # longer holds at 2e-3 — asserted at the level it actually sits, still an order
    # tighter than what Euler's liveness guards argue about.
    assert abs(0.074077 - 0.073887) / 0.073887 < 3e-3


# The frozen `sealed_chamber` run's CO2-trough DAY; both runs are sampled here so
# the deltas are taken at one instant (they trough one step apart since 2026-08-10).
# ⚠ In DAYS since 2026-08-14, converted at the point of use. It was a bare `195` used
# directly as a trajectory index, i.e. a day count standing in for a step count — the
# sixth conflation class, in the shape a name-based sweep cannot see because the name
# says neither.
_COMMON_TROUGH_DAY = 195


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
    #
    # ⚠⚠ **AND IT HAPPENED A FOURTH TIME, 2026-08-12 — ON THE OTHER TUPLE.** The stem
    # reserve is PLANT carbon, so the structural guard above (which owns the SOIL set)
    # could not see it, and the ``tissue`` sum below is local to this test and ungated.
    # The identity broke by 4.243e-04 and read like a conservation bug in the engine; it
    # was this tuple. Folding ``stem_reserve_c`` in closes it to 1.776e-15 and gives a
    # sealed total of exactly 4.017000000. **The lesson is not "remember the pool" — it
    # is that an ungated local tuple reports a test's own omission as a MODEL failure.**
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
        lo = steps_for(_COMMON_TROUGH_DAY)
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
            + st.stocks[STORAGE_C].amount
            + st.stocks[STEM_RESERVE_C].amount,  # ⚠ 2026-08-12, see the note above
            "soil": sum(by_short.get(p, 0.0) for p in soil_pools),
        }
    b, n = snaps["frozen"], snaps["stem0"]
    # ⚠ the one-step offset the humification split introduced is GONE (2026-08-12): both
    # runs trough at 195 again. Kept as an equality on the pair rather than deleted,
    # because "they trough together" is what licenses the common-step differencing below
    # and it has now been false once.
    # ⚠ **COMPARED IN DAYS SINCE 2026-08-14.** Both runs now trough at step 780, and
    # 195 × 4 = 780 **exactly** — the structural signature of a pure rescale, so the
    # trough is on the same physical day it always was and only the unit moved.
    # Asserting the day keeps this stable across any future step change; asserting the
    # step made it a step-size pin wearing a science label.
    # ⚠ 2026-08-14 (the light path): both troughs move ONE STEP earlier, to 194.75 —
    # a quarter of a day, i.e. the trough is now reached in the last dark window of the
    # day before. The pair still troughs TOGETHER, which is the whole claim (it licenses
    # the common-step differencing below); what moved is the day, not the agreement.
    assert (b["own_step"] / STEPS_PER_DAY, n["own_step"] / STEPS_PER_DAY) == (
        194.75,
        194.75,
    ), (b["own_step"], n["own_step"])
    assert b["step"] == n["step"] == steps_for(_COMMON_TROUGH_DAY)
    # The inventory is closed: what one group gained, the others lost, exactly.
    total_b = b["co2"] + b["tissue"] + b["soil"]
    total_n = n["co2"] + n["tissue"] + n["soil"]
    assert abs(total_n - total_b) < 1e-9, (total_b, total_n)
    # re-measured 2026-08-12: 3.517 -> 4.017, and it is EXACT to fifteen digits because
    # a
    # sealed chamber's carbon inventory is a constant of the scenario, not a result.
    assert total_b == pytest.approx(4.017, rel=1e-9)
    d_tissue = n["tissue"] - b["tissue"]
    d_soil = n["soil"] - b["soil"]
    d_co2 = n["co2"] - b["co2"]
    # re-measured 2026-08-10 (humification split). The MECHANISM is unchanged and is
    # what this test exists for: the extra standing stem is funded by the other pools,
    # mostly the soil. The split moves the split of the funding, not the story.
    # ...and re-measured again 2026-08-12 (stem reserves), with ``stem_reserve_c`` now
    # inside ``tissue``. Was 0.084649 / -0.085210 / 0.000562. The MECHANISM is unchanged
    # a second time, which is the point of keeping this test: the extra standing stem is
    # still funded almost entirely by the soil.
    # ⚠ all three re-measured 2026-08-14 (the light path). The MECHANISM is unchanged
    # a THIRD time — the extra standing stem is still funded almost entirely by the
    # soil — while the amount funded falls ~20 % with the smaller crop.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.051746 ->
    # 0.055845. The
    # MECHANISM is unchanged a FOURTH time — the extra standing stem is still funded
    # almost entirely by the soil — while the amount funded rises ~8 %.
    assert d_tissue == pytest.approx(0.055845, rel=2e-3), d_tissue
    assert d_soil == pytest.approx(-0.055939, rel=2e-3), d_soil
    # ⚠ 2026-08-14: 0.0000205 -> 0.0000721. The CO₂ leg is the small residual of the
    # three and it grew 3.5× while the two large legs shrank ~20 % — the night
    # respiration returning carbon to the pool is exactly the kind of term that shows
    # up here. Still three orders below the tissue/soil transfer this test is about.
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.0000721 ->
    # 0.0000939. The residual grows a further 30 % while the two large legs grow ~8 %,
    # so the pattern of 2026-08-14 repeats in miniature. Still three orders below the
    # tissue/soil transfer this test is about, which is the only claim made here.
    assert d_co2 == pytest.approx(0.0000939, rel=2e-2), d_co2
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
    """Per-year summaries for the re-price. Returns (co2, leaf, stem, stem+reserve,
    storage, rationed).

    ⚠ ``stem`` and ``stem_plus_reserve`` are BOTH returned, and section 4 asserts on
    both, because since 2026-08-12 they are different quantities and the choice changes
    the headline number a lot: "peak stem +56.6 %" counts ``stem_c`` alone, while
    counting the reserve as part of the stem — which is what it physically is, starch
    inside stem tissue — gives +21.6 %. Returning only one would silently pick a side.
    """
    states, rationed, _ = _run(
        sc.PERENNIAL_CHAMBER_SCENARIO, years, resets=True, stem_zero=stem_zero
    )
    # ⚠ In STEPS: ``year_summaries`` segments the STEP-indexed trajectory.
    year_len = steps_for(len(_weather()))
    return (
        year_summaries(states, year_len, _co2_min_of),
        year_summaries(states, year_len, _peak_leaf_of),
        year_summaries(
            states, year_len, lambda s: max(x.stocks[STEM_C].amount for x in s)
        ),
        year_summaries(
            states,
            year_len,
            lambda s: max(
                x.stocks[STEM_C].amount + x.stocks[STEM_RESERVE_C].amount for x in s
            ),
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


def test_the_ONE_REMAINING_stem_only_guard_is_not_a_horizon_question() -> None:
    """⚠⚠ **2026-08-12: THERE IS ONLY ONE GUARD LEFT TO ASK THIS OF.**

    The stem-reserve build dissolved the FLOOR leg entirely — ``non_collapsing`` is now
    True in both the windowed and whole-run forms, at fifteen years and at fifty. The
    question "is the floor verdict a horizon question?" is therefore moot rather than
    answered: there is no floor verdict left to move. Kept, because what it measured was
    true of the tree it measured, and because the STATIONARITY half is untouched and is
    now the whole refusal.

    ⚠ **AND THE STATIONARITY LEG IS NARROW — measured, because "it fails stationarity"
    and "it fails on one diff at 1.28x its bound" are very different records to hand a
    successor.** The offending same-phase diff is a single index (was two), and:

        diffs[4] = 0.019309   1.2843x the bound
        diffs[3] = 0.014731   0.9798x the bound   <- just under
        diffs[2] = -0.005649  0.3757x the bound

    So the last leg of the stem-only refusal rests on one diff 28 % over a threshold,
    with the next one 2.0 % under it. The bound is ``0.2 * max(series)`` and the max
    sits
    in year 0, which the reserve also moved — so the threshold this is measured against
    is itself downstream of the change being measured. Stated rather than resolved: the
    verdict is not this build's to re-decide.

    ⚠ The horizon-invariance claim is now measured with a STRONGER control than before:
    the offender indices AND ``diffs[4]`` itself are bit-identical at 15 and 50 years,
    because the series maximum lands in year 0 in both. Appending years cannot move it.

    ---- the framing correction, as written for the two-guard tree -------------------

    ⚠ THE FRAMING CORRECTION, and it is cheap enough to pin without a 50-year run.

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
    co2, _leaf, _stem, _stemres, _store, rationed = _reprice_series(
        LONG_HORIZON_YEARS, True
    )
    assert rationed == 0

    # the 15-year readings, re-measured 2026-08-12 (was 0.046065 / argmin 2 / 0.074891)
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.071132 ->
    # 0.070627.
    assert min(co2[2:]) == pytest.approx(0.070627, rel=1e-4)
    assert co2.index(min(co2)) == 1, "the dip moved back to year 1 — 2026-08-14"
    assert co2[-1] == pytest.approx(0.072206, rel=1e-4)  # was 0.075823

    # ⚠ THE FLOOR LEG IS GONE — both forms, where both used to be False.
    assert non_collapsing(co2[2:], floor=_DECADE_CO2_FLOOR) is True
    assert non_collapsing(co2, floor=_DECADE_CO2_FLOOR) is True

    # ...and stationarity is what is left. Locate the offender rather than assert the
    # verdict, and pin HOW FAR OVER it is — a leg carrying a refusal on its own should
    # not be recorded as a bare boolean.
    scale = max(co2)
    bound = 0.2 * scale
    diffs = same_phase_diffs(co2, period=2)
    offenders = [i for i, d in enumerate(diffs) if i >= 2 and abs(d) > bound]
    # ⚠⚠ **NO OFFENDERS AT ALL, 2026-08-14 — was [2, 3], then [4], now [].** The single
    # marginal diff that carried the whole stem-only refusal (1.2843× its bound, with
    # the runner-up at 0.99×) is under the bound at the shipped step. The two retired
    # pins were `diffs[4] == 0.019309` and `abs(diffs[4]) / bound == 1.2843`.
    assert offenders == [], offenders
    # ...and the replacement says the same thing quantitatively: the WORST
    # post-transient diff, whichever index it lands on, is now under its own bound.
    # Pinned two-sided so a change pushing it back toward 1.0 is read, not just passed.
    worst = max(abs(d) for i, d in enumerate(diffs) if i >= 2) / bound
    assert 0.0 < worst < 1.0, worst
    # ⚠ the runner-up sat JUST under at 0.9800 — "the margin the successor needs to
    # see". It is now **0.008555**, two orders below its bound. Together with the
    # offender list emptying, that is the whole distance this leg travelled: the series
    # is not marginally stationary, it is flatly stationary.
    # ⚠ 0.008555 -> 0.033142 (2026-08-14). Still two orders inside the bound; the leg
    # is flatly stationary either way, which is all this line claims.
    assert abs(diffs[3]) / bound == pytest.approx(0.037162, abs=1e-5)
    assert _stationary_verdict(co2) is True, "no longer refuses — 2026-08-14"

    # ⚠ the control, so this test still records a tree in which the floor leg refused.
    off_states, _r2, _ = _run(
        _without_reserve(sc.PERENNIAL_CHAMBER_SCENARIO),
        LONG_HORIZON_YEARS,
        resets=True,
        stem_zero=True,
    )
    off = year_summaries(
        off_states,
        steps_for(len(_weather())),  # STEPS: segments a step-indexed trajectory
        lambda seg: min(s.stocks[CARBON_POOL].amount for s in seg),
    )
    # ⚠⚠ **THE CONTROL DIED HERE TOO, 2026-08-14** — same event as in the 15-year test:
    # the reserve-off tree no longer collapses either, so this file no longer contains a
    # running tree in which the floor leg refuses. Named, asserted at its measured
    # value, and left as outstanding work rather than patched over.
    assert non_collapsing(off, floor=_DECADE_CO2_FLOOR) is True
    assert off.index(min(off)) == 1  # ⚠ was 2 — the dip moved with the transient


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

    =================================================================================
    ⚠⚠ RE-PRICED AGAIN 2026-08-12 (the stem-reserve build), AND THE VERDICT CHANGED.
    =================================================================================

    Everything above belongs to the pre-reserve tree. What fifty years now say:

    1. **THE FLOOR LEG IS GONE.** ``below == []`` — not one year of fifty sits under
       0.05, where the pre-reserve tree had year 2 at 0.921x. Its worst year rose
       0.046065 -> 0.053177. ``non_collapsing`` is True in BOTH forms.
    2. **THE ATTRACTOR COMPARISON INVERTED.** Stem-only settles at 0.071919 against the
       control's 0.073668 — **below** the reference, where it used to sit above it
       (0.075339 vs 0.073291). Claim 1 of the old re-price is now false, and in the
       direction UNFAVOURABLE to the branch. Recorded plainly: the build did not simply
       make stem-only look better, it moved both sides and swapped their order.
    3. **STATIONARITY IS THE WHOLE REFUSAL NOW**, and it rests on ONE same-phase diff at
       1.28x its bound with the runner-up at 0.99x (measured in the test above, at both
       horizons). A refusal carried by a single marginal diff is a different object from
       one carried by two independent guards, and a successor must be handed that.
    4. **The liveness gate still clears** (0.637504 against the 0.55 bound) and the
    plant
       is still not free — but that claim's SIZE now depends on a choice:

    ⚠ **"STEM UP, GRAIN DOWN" MUST SAY WHICH POOLS IT COUNTS.** With the reserve
    intercepting 40 % of stem growth, ``stem_c`` alone no longer means what it meant:
    peak stem reads **+57.9 %** counting ``stem_c``, and **+21.6 %** counting
    ``stem_c + stem_reserve_c``, which is what the stem physically holds. Both are
    asserted below. The DIRECTION is robust to the choice and the MAGNITUDE is not, so
    quoting the big number without its basis would be the flattering half of a fork.

    ⚠⚠ **AND THE BRANCH IS NOT RE-DECIDED HERE.** Two of its three legs have now
    dissolved under changes made for other reasons, and its form-gap objection was
    removed by this very build (see the module docstring). That is a strong case for
    revisiting it — and revisiting it *inside the commit that moved every one of its
    numbers* is exactly the co-adaptation this project refuses. Named as a successor.
    """
    f_co2, f_leaf, f_stem, f_stemres, f_store, f_r = _reprice_series(
        _REPRICE_YEARS, False
    )
    s_co2, s_leaf, s_stem, s_stemres, s_store, s_r = _reprice_series(
        _REPRICE_YEARS, True
    )

    # --- the discharged leg must not un-discharge at 3.3x the frozen horizon ---------
    assert f_r == 0, "frozen control rations at 50 yr — the comparison would be void"
    assert s_r == 0, "stem-only rations at 50 yr — the discharge was horizon-bound"

    # --- 1. the attractor, subject AND control on the same harness ------------------
    # ⚠ re-measured 2026-08-12. Was 0.075339 / 0.073291 / 1.5068, with s ABOVE f.
    # ⚠ re-measured AGAIN 2026-08-14 (the quarter-day step): 0.071919 -> 0.075821 and
    # 0.073668 -> 0.075845. **The two attractors have converged to within 0.03 %** (they
    # were 2.4 % apart). The ordering below still holds, but it now rests on a gap three
    # orders smaller than the values it separates — read it as "indistinguishable", not
    # as "stem-only settles lower".
    # ⚠ 2026-08-15 (the depth-resolved canopy + the sourced SLA anchor): 0.073435 ->
    # 0.072292.
    assert s_co2[-1] == pytest.approx(0.072292, abs=1e-5), "stem-only CO2 attractor"
    assert f_co2[-1] == pytest.approx(0.072238, abs=1e-5), "frozen CO2 attractor"
    assert s_co2[-1] > _DECADE_CO2_FLOOR and f_co2[-1] > _DECADE_CO2_FLOOR
    assert s_co2[-1] / _DECADE_CO2_FLOOR == pytest.approx(1.445831, abs=1e-3)
    # ⚠⚠ **THE ORDERING FLIPPED A SECOND TIME (2026-08-14) AND IS THEREFORE RETIRED AS
    # AN ORDERING.** It read "ABOVE", was flipped to "BELOW" on 2026-08-14 (the step),
    # and the light path flips it back the same day: stem-only 0.073435 vs frozen
    # 0.073326, a **0.15 %** gap. Two sign changes from two unrelated numerics/forcing
    # changes is the definition of a quantity whose sign carries no information, and
    # asserting it a third time would be pinning which way the noise fell.
    #
    # ⇒ Asserted as **indistinguishability**, which is what the previous comment already
    # said the reading should be and what both measurements actually support. A future
    # change that genuinely separates the two attractors will fail this bound and be
    # read, which an ordering that flips on its own cannot deliver.
    assert abs(s_co2[-1] - f_co2[-1]) / f_co2[-1] < 0.005, (s_co2[-1], f_co2[-1])
    assert max(s_co2[-5:]) - min(s_co2[-5:]) < 1e-6, "and it is genuinely settled"

    # --- 2. the floor leg has DISSOLVED; stationarity carries the refusal alone ------
    below = [i for i, v in enumerate(s_co2) if v < _DECADE_CO2_FLOOR]
    assert below == [], below  # ⚠ was [2]
    assert [i for i, v in enumerate(f_co2) if v < _DECADE_CO2_FLOOR] == []
    # re-measured: the subject's worst year 0.046065 -> 0.053177, now ABOVE the floor;
    # the control's own trough 0.055175 -> 0.056030.
    # ⚠ and again 2026-08-14: 0.053177 -> 0.075578 and 0.056030 -> 0.075476. The
    # subject's worst year is now 1.51x the floor where it was 1.06x, i.e. the
    # establishment transient that carried the whole floor leg is gone rather than
    # merely cleared — and the subject's worst year now sits ABOVE the control's.
    # ⚠ 2026-08-15 (layered canopy) 0.070549 -> 0.07027972343618574, -0.4 %; the
    # subject still sits above the floor and above the control, so the reading above
    # is unchanged in substance.
    assert min(s_co2) == pytest.approx(0.070280, abs=1e-5), "the subject's worst year"
    assert min(s_co2) / _DECADE_CO2_FLOOR == pytest.approx(1.405594, abs=1e-3)
    assert min(f_co2) == pytest.approx(0.070253, abs=1e-5), "the control's own trough"
    # ⚠ the floor now returns True in BOTH forms, for BOTH runs — four assertions where
    # three used to be False. Kept in full rather than collapsed: the pair of forms is
    # what shows the verdict is window-independent, and that is still worth saying when
    # the verdict is a pass.
    assert non_collapsing(s_co2[2:], floor=_DECADE_CO2_FLOOR) is True
    assert non_collapsing(s_co2, floor=_DECADE_CO2_FLOOR) is True
    assert non_collapsing(f_co2[2:], floor=_DECADE_CO2_FLOOR) is True
    assert non_collapsing(f_co2, floor=_DECADE_CO2_FLOOR) is True
    # ⚠⚠ **AND AT FIFTY YEARS TOO, 2026-08-14: stationarity passes.** This read
    # `is False` under "stationarity is the one leg still separating subject from
    # control". It was the LAST leg of the stem-only refusal on this file's evidence,
    # and it is gone at both horizons (see the 15-year test above). Not reopened here —
    # three legs have now dissolved under changes made for other reasons, which is a
    # strong case for a successor that re-decides the branch with all of it on the
    # table, and exactly not a case for deciding it inside the change that moved them.
    assert _stationary_verdict(s_co2) is True, "no leg refuses at 50 yr — 2026-08-14"
    assert _stationary_verdict(f_co2) is True
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
    #
    # ⚠⚠ 2026-08-12: THE COUNTERFACTUAL IS NOW HALF VACUOUS, and saying so is the honest
    # move. There is no sub-floor year left to splice out, so the floor half asserts
    # True-from-True and proves nothing. It is KEPT — the stationarity half still
    # carries
    # the independence claim the paragraph above is about, and deleting the pair would
    # quietly retire a refuted hypothesis's own record. The vacuity is named, not
    # hidden.
    spliced = list(s_co2)
    spliced[2] = f_co2[2]
    assert non_collapsing(spliced[2:], floor=_DECADE_CO2_FLOOR) is True  # vacuous now
    # ⚠ 2026-08-14: this half became vacuous too — with stationarity now passing on the
    # unspliced series, splicing one year cannot make it fail. The whole splice control
    # is therefore inert at the shipped step; kept, asserted at its measured value, and
    # named as inert rather than deleted.
    assert _stationary_verdict(spliced) is True  # ...and now this half is vacuous too
    assert all(v > _DECADE_CO2_FLOOR for v in s_co2[6:]), "clean from year 6 on"

    # --- 3. the MANIFEST-named liveness gate -----------------------------------------
    f_tail = f_leaf[_LIVENESS_TRANSIENT:]
    s_tail = s_leaf[_LIVENESS_TRANSIENT:]
    # validate the reconstruction against the gate's own committed 15-yr reading first.
    # ⚠ That committed 0.634352 is ``max(tail)``, NOT ``tail[-1]``: the tail declines
    # monotonically, so max(tail) == tail[0] == year 8. My first version of this check
    # compared the wrong element and went red on a correct reconstruction.
    # ⚠ all four re-measured 2026-08-12: 0.634352 -> 0.637384, 0.594984 -> 0.593883,
    # 0.643676 -> 0.637504. The gate's VERDICT is unchanged — both clear 0.55 — and the
    # two runs are now much closer together (0.02 % apart, was 1.5 %).
    f15_leaf = _reprice_series(LONG_HORIZON_YEARS, False)[1]
    # ⚠ 0.612211 -> 0.603679 (2026-08-14, the light path) -> 0.578137 (2026-08-15,
    # the layered canopy). Still clears the 0.55 bound, by 5.1 %.
    assert max(f15_leaf[_LIVENESS_TRANSIENT:]) == pytest.approx(0.578137, abs=1e-5)
    # ⚠ 0.594984 -> 0.593883 -> 0.577062 -> **0.567715** (2026-08-14, the light path,
    # hours after the step took it to 0.577062). The manifest's `max(tail) > 0.55` bound
    # for `perennial_long_horizon` is anchored on this measured equilibrium, and its
    # clearance has now fallen 8.0 % -> 4.9 % -> **3.2 %** across two changes in one
    # day.
    # ⚠ The bound still passes and is NOT re-tuned. But two consecutive moves in the
    # same
    # direction, from unrelated causes, is the pattern worth naming rather than the
    # individual number: **a liveness floor anchored on a measured equilibrium is being
    # approached, not held**, and whoever next raises assimilation cost or lowers it
    # should read this line before assuming the floor has room.
    # ⚠⚠ 2026-08-15 (the layered canopy): 0.567715 -> **0.543748, and that is BELOW the
    # 0.55 the manifest floor is anchored on.** The warning two paragraphs up said this
    # floor was being approached rather than held; it has now been passed, by a third
    # change from a fourth unrelated cause. ⚠ The floor is NOT moved and the MANIFEST
    # GATE IS NOT RED: the bound is `max(tail) > 0.55` on the 15-year run, which reads
    # 0.578137 above and passes. What sits below 0.55 is the FIFTY-year settled value,
    # which is a probe here and not a contract anywhere. The two are asserted side by
    # side so the gap between what the contract checks and where the attractor actually
    # goes is visible rather than inferred.
    assert f_leaf[-1] == pytest.approx(0.543748, abs=1e-5), "the anchored equilibrium"
    assert max(f_tail) > _LIVENESS_FLOOR, "control clears its own floor"
    assert max(s_tail) > _LIVENESS_FLOOR, "⚠ stem-only CLEARS it too — no third leg"
    assert max(s_tail) == pytest.approx(0.581452, abs=1e-5)  # ⚠ 2026-08-15 was 0.607029
    assert s_leaf[-1] / _DEAD_BASELINE > 2.0, "and it is nowhere near the dead baseline"

    # --- 4. the plant is NOT free: read the CO2 gain WITH its cost -------------------
    # ⚠ THE POOL CHOICE IS LOAD-BEARING HERE, so BOTH readings are asserted. Counting
    # ``stem_c`` alone the stem is +57.9 %; counting the reserve as the stem carbon it
    # physically is, +21.6 %. Was 1.518 on a tree with no reserve to argue about.
    # ⚠ 1.5827 -> 1.4384 (2026-08-14, the light path) -> 1.44846 (2026-08-15, the
    # layered canopy). The magnitude moves; the sign, which is what section 4 rests on,
    # does not — through three consecutive changes now.
    assert s_stem[-1] / f_stem[-1] == pytest.approx(1.44846, rel=2e-3), (
        "stem UP (stem_c)"  # was 1.5827
    )
    assert s_stemres[-1] / f_stemres[-1] == pytest.approx(1.177450, rel=2e-3), (
        "stem UP (stem_c + reserve) — same sign, less than half the magnitude"
    )
    assert s_stem[-1] / f_stem[-1] > s_stemres[-1] / f_stemres[-1] > 1.0, (
        "the direction is robust to the pool choice; the magnitude is not"
    )
    # grain: **-0.21 %**, was -4.5 %, was -11.8 %. The reserve feeds grain in BOTH runs,
    # so the penalty stem-only imposes on it keeps shrinking — and after the light path
    # it is two orders below where it started. ⚠ Read the pair of assertions as what
    # they
    # now are: the SIGN survives (the next line), the magnitude has all but dissolved.
    # A third shrink of this size would make "stem up, grain down" a claim about the
    # seventh decimal, and the successor should say so rather than re-pin again.
    assert s_store[-1] / f_store[-1] == pytest.approx(0.99793, rel=2e-3), "grain DOWN"
    assert s_store[-1] < f_store[-1], (
        "(C)-finding 8's 'stem up, grain down' holds in THIS scenario too"
    )
    # ⚠ the leaf sign does NOT transfer from open_season (-2.56 % there, +1.00 % here).
    # ⚠ 1.0182 -> 1.0100 (2026-08-14): the SIGN is what the claim rests on and it holds,
    # but the margin has halved, so this scenario is now within 1 % of agreeing with
    # open_season rather than clearly disagreeing with it.
    assert s_leaf[-1] > f_leaf[-1]
    assert s_leaf[-1] / f_leaf[-1] == pytest.approx(1.00753, rel=2e-3)
