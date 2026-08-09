"""What can the frozen contract's ``rationed == 0`` gate actually refute?

Read-only census. No fixture, no unfreeze, no golden regenerated: every number here is
measured off the committed frozen trajectories, each scenario driven the way its **own**
golden drives it (the runners are imported from the committed regression modules rather
than re-derived — the ``run_season``/``run_perennial`` mix-up the nitrogen work had to
correct twice).

**The question.** The chamber-scale diagnosis closed by stating that the defect
is not the sealed chamber's size but that *the frozen contract uses that rig's
closure gate as the acceptance test for field-scale plant science*, and that
``open_season`` "carries no carbon-scarcity gate at all". This file measures both halves
instead of arguing them.

**The metric is the gate's OWN arithmetic, not one invented here.**
``simcore.arbitration._scale_factors`` computes, per clamped stock,
``scale_s = available_s / demand_s`` and fires when it drops below 1. So the minimum of
that ratio over a run is exactly "how close did the backstop come to firing on this
stock", in the gate's own units, and it is 1.0 at the firing point by construction. The
tie to the shipped code is pinned by
:func:`test_the_margin_is_the_gates_own_scale_factor`, not asserted in prose.

**The classification.**

* ``impossible:boundary``     - the stock is ``unclamped``; arbitration *skips* it
  (decision #13). No gate can exist here, at any value.
* ``impossible:never-withdrawn`` - clamped, but nothing ever draws from it.
* ``rate-determined``         - the margin is **flat** over the whole run (relative
  spread <= 1e-12), which happens exactly when total demand is proportional to the
  stock: the margin is then ``1/(Σk_i·dt)``, a property of the timestep, not of
  scarcity. Such a stock can only ration when ``k·dt > 1``, so there the gate answers
  "is my timestep safe?" rather than "is the resource short?". ⚠ The converse does NOT
  hold and is not claimed: a varying margin is not thereby a scarcity margin
  (``microbial_carbon`` varies only because ``f_O2`` does).
* ``live``                    - trajectory-dependent.

**What the census finds** (the detail, and what it does not license, is in
``docs/plans/post-roadmap-acceptance-gate.md``):

1. In ``open_season`` the crop's carbon source is ``boundary.co2_atmos``, which is
   ``unclamped`` **and holds 0.0 mol C for the entire run**. The gate is structurally
   forbidden from looking at it, so ``assert rationed == 0`` in the season golden cannot
   be read as "the crop was well fed". Its clamped CARBON stocks are the plant's own
   tissue.
2. The qualifier in the committed sentence ("no **carbon**-scarcity gate") is kept, and
   the other currencies are measured rather than inherited: ``soil_water`` and
   ``soil_n`` are live but slack by 2 and 5 orders. Two reasons, recorded separately.
3. Exactly one *stock* in the whole 20-scenario frozen roster is a binding gate:
   ``biosphere.carbon_pool``. The six smallest **live** margins anywhere in the roster
   are that stock, in the six scenarios that seal a chamber.
4. The contract has no plausibility column at all: a manifest scenario entry carries
   only ``scenario``/``golden``/``golden_sha256``(/``years``).

**Deliberately NOT here: adjudication.** That the closure gate and the canopy
plausibility reading disagree about specific refused changes is a measured fact and is
recorded in the plan. Using a gate written here to reverse a measured refusal would be
the co-adaptation shape this project has refused three times (the consumer-chamber 2x,
the DPM/RPM labile re-read, ruling B). Which gate is authoritative is a *contract*
question and is left to the user.
"""

from __future__ import annotations

import json
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from dataclasses import dataclass, replace
from pathlib import Path

import pytest

import simcore.arbitration as arbitration
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock

_REPO_ROOT = Path(__file__).resolve().parent.parent
BIOSPHERE_MANIFEST = _REPO_ROOT / "docs" / "biosphere-reference.manifest.json"
STATION_MANIFEST = _REPO_ROOT / "docs" / "station-reference.manifest.json"


# --------------------------------------------------------------------------- #
# The recorder: the gate's own scale_s, before the min(1, .) clamp             #
# --------------------------------------------------------------------------- #


@dataclass
class Entry:
    """Per-stock census row for one scenario."""

    quantity: Quantity
    kind: StockKind
    unclamped: bool
    amount0: float
    gate: str
    min_margin: float | None = None
    max_margin: float | None = None


class _Recorder:
    def __init__(self) -> None:
        self.meta: dict[str, Stock] = {}
        self.margins: dict[str, list[float]] = {}
        self.calls = 0
        # The distinct demand-sets seen. In a multi-registry scenario the gate fires
        # once per REGISTRY CALL, so these separate the registries (and, within one
        # registry, the phenological phases). See the per-registry-call pin.
        self.signatures: dict[frozenset[str], int] = {}
        self.per_sig_min: dict[tuple[str, frozenset[str]], float] = {}

    def registries(self) -> tuple[frozenset[str], frozenset[str]]:
        """(dominant call-signature, union of all the others).

        In a two-registry scenario the fast registry supplies the overwhelming majority
        of calls, so the dominant signature is it and the rest are the slow registry at
        its various phases. Single-registry scenarios return an empty second element.
        """
        ordered = sorted(self.signatures, key=lambda s: -self.signatures[s])
        rest = ordered[1:]
        return ordered[0], (frozenset().union(*rest) if rest else frozenset())

    def __call__(self, results, stocks):  # noqa: ANN001 - mirrors min_scaling
        if not self.meta:
            self.meta = {str(sid): st for sid, st in stocks.items()}
        self.calls += 1
        # The demand accumulation below is `_scale_factors`' own, verbatim: only
        # withdrawals count and unclamped sources are skipped (decision #13).
        demand: dict[StockId, float] = {}
        for result in results:
            for leg in result.legs:
                if leg.amount < 0.0 and not stocks[leg.stock].unclamped:
                    demand[leg.stock] = demand.get(leg.stock, 0.0) - leg.amount
        sig = frozenset(str(s) for s in demand)
        self.signatures[sig] = self.signatures.get(sig, 0) + 1
        for sid, d in demand.items():
            if d <= 0.0:
                continue
            margin = stocks[sid].amount / d
            key = (str(sid), sig)
            if key not in self.per_sig_min or margin < self.per_sig_min[key]:
                self.per_sig_min[key] = margin
            rec = self.margins.get(str(sid))
            if rec is None:
                self.margins[str(sid)] = [margin, margin]
            else:
                rec[0] = min(rec[0], margin)
                rec[1] = max(rec[1], margin)
        return _ORIGINAL_MIN_SCALING(results, stocks)

    def rows(self) -> dict[str, Entry]:
        out: dict[str, Entry] = {}
        for sid, stock in self.meta.items():
            rec = self.margins.get(sid)
            if stock.unclamped:
                gate = "impossible:boundary"
            elif rec is None:
                gate = "impossible:never-withdrawn"
            else:
                gate = "rate-determined" if _is_flat(rec[0], rec[1]) else "live"
            out[sid] = Entry(
                quantity=stock.quantity,
                kind=stock.kind,
                unclamped=stock.unclamped,
                amount0=stock.amount,
                gate=gate,
                min_margin=None if rec is None else rec[0],
                max_margin=None if rec is None else rec[1],
            )
        return out


# ⚠ A *relative-spread* test, not exact equality — and the difference is not pedantry.
# `margin = x / (k*x)` is algebraically `1/k` but not bit-stable across different `x`:
# `water_vapor` lands on exactly 2.0 at every step only because its rate 0.5 is binary-
# exact, while `litter_carbon`'s 0.011 is not and its margin wobbles by **one ULP**
# (spread 2.2e-16). Classifying by `min == max` therefore files the decomposer's litter
# pool as a *scarcity* gate purely because of a rounding artefact — the exact failure
# this census exists to avoid. The cut sits ~3.7 orders above the observed ULP wobble
# (2.2e-16) and ~6.7 below the smallest genuine variation measured (`microbial_carbon`,
# 4.6e-06) — a ~10.4-order gap, so its exact placement is not load-bearing.
_FLAT_REL_SPREAD = 1e-12


def _is_flat(lo: float, hi: float) -> bool:
    return hi / lo - 1.0 <= _FLAT_REL_SPREAD


_ORIGINAL_MIN_SCALING = arbitration.min_scaling


@contextmanager
def _recording() -> Iterator[_Recorder]:
    recorder = _Recorder()
    arbitration.min_scaling = recorder  # type: ignore[assignment]
    try:
        yield recorder
    finally:
        arbitration.min_scaling = _ORIGINAL_MIN_SCALING  # type: ignore[assignment]


# --------------------------------------------------------------------------- #
# The roster: the manifests' scenario sets, driven by the goldens' own runners  #
# --------------------------------------------------------------------------- #


def _runners() -> dict[str, Callable[[], object]]:
    """Manifest scenario key -> the committed runner that produces its golden.

    Imported lazily (inside the function) so that collecting this module does not
    import 17 regression modules for the tests that only read the manifests.
    """
    import test_regression_cabin as cabin
    import test_regression_consumer_season as consumer
    import test_regression_crew as crew
    import test_regression_eclss as eclss
    import test_regression_greenhouse as greenhouse
    import test_regression_harvest as harvest
    import test_regression_lighting as lighting
    import test_regression_long_horizon as long_horizon
    import test_regression_perennial_season as perennial
    import test_regression_power as power
    import test_regression_power_self_discharge as power_sd
    import test_regression_sealed_season as sealed
    import test_regression_season as season
    import test_regression_station as station
    import test_regression_thermal as thermal
    import test_regression_water_recovery as water_recovery
    from domains.biosphere.scenario import (
        CONSUMER_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_SCENARIO,
    )

    return {
        # --- biosphere manifest (6 runnable + drift_summary, derived below) ---
        "open_season": season._final_state,
        "sealed_chamber": sealed._final_state,
        "perennial_chamber": perennial._final_state,
        "consumer_chamber": consumer._final_state,
        "perennial_long_horizon": lambda: long_horizon._run(PERENNIAL_CHAMBER_SCENARIO),
        "consumer_long_horizon": lambda: long_horizon._run(CONSUMER_CHAMBER_SCENARIO),
        # --- station manifest (13) ---
        "power_bounded_soc": power._final_state,
        "power_self_discharge": power_sd._final_state,
        "thermal_equilibrium": thermal._final_state,
        "eclss_steady_state": eclss._final_state,
        "crew_mission": crew._final_state,
        "station_heat_closure": station._final_state,
        "cabin_gas": cabin._final_state,
        "greenhouse": greenhouse._final_state,
        "water_recovery": water_recovery._final_state,
        "lighting": lighting._final_state,
        "harvest": harvest._final_state,
        "sealed_station": _tier2_states,
        "sealed_energy_drift": _energy_states,
    }


# ``drift_summary`` is a biosphere manifest scenario with **no trajectory of its own** —
# it is the per-year stability signature of the two long-horizon runs already in the
# roster. Named here so its absence from ``_runners`` reads as deliberate, and pinned by
# ``test_the_roster_is_exactly_the_two_manifests`` rather than left to a comment.
_DERIVED_FROM_OTHER_RUNS = {
    "drift_summary": ("perennial_long_horizon", "consumer_long_horizon")
}

# The Tier-2 station run — minutes, not seconds, and the reason the roster-wide slow
# claims are merged into one test rather than spread across four.
_TIER2 = "sealed_station"

# The four whose runs are minutes rather than seconds.
_SLOW = frozenset(
    {
        "perennial_long_horizon",
        "consumer_long_horizon",
        "sealed_station",
        "sealed_energy_drift",
    }
)


def _tier2_states() -> object:
    from sealed_tier2_helper import run_tier2

    return run_tier2()


def _energy_states() -> object:
    """Mirrors ``test_regression_sealed_station.energy_states``.

    Reproduced rather than imported because the original is a module-scoped fixture, so
    it is not callable. ⚠ :func:`test_the_energy_drift_runner_matches_its_golden_module`
    pins it against that module's **constants only** — a change to the fixture's
    integrator or resolver would leave this census measuring a different trajectory with
    that pin still green. The duplication is the exposure; it is named, not closed.
    """
    from domains.power.loader import load_charge_params
    from domains.thermal.loader import load_thermal_params
    from simcore.integrator import EulerIntegrator
    from station.scenario import HEAT_CLOSURE_SCENARIO, SEALED_ENERGY_DAYS
    from station.system import build_station, run_station, station_resolver

    charge = load_charge_params()
    thermal = load_thermal_params()
    state, registry = build_station(charge, thermal, HEAT_CLOSURE_SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(registry),
        state,
        station_resolver(charge, HEAT_CLOSURE_SCENARIO),
        HEAT_CLOSURE_SCENARIO.power.dt_seconds,
        SEALED_ENERGY_DAYS * HEAT_CLOSURE_SCENARIO.power.steps_per_day,
    )
    assert rationed == 0 and events == ()
    return states


_CACHE: dict[str, _Recorder] = {}


def _recorder_for(name: str) -> _Recorder:
    """Run ``name`` under the recorder, once per process."""
    if name not in _CACHE:
        with _recording() as recorder:
            _runners()[name]()
        _CACHE[name] = recorder
    return _CACHE[name]


def census(name: str) -> dict[str, Entry]:
    return _recorder_for(name).rows()


FAST = tuple(k for k in _runners() if k not in _SLOW)


def _live(rows: dict[str, Entry]) -> dict[str, float]:
    return {
        sid: e.min_margin
        for sid, e in rows.items()
        if e.gate == "live" and e.min_margin is not None
    }


# --------------------------------------------------------------------------- #
# 0. the metric is the gate's, and the roster is the manifests'                #
# --------------------------------------------------------------------------- #


def test_the_margin_is_the_gates_own_scale_factor() -> None:
    """``min_margin`` is ``scale_s`` before the ``min(1, .)`` clamp — not a new metric.

    Built as a synthetic three-flow case so the tie is to the shipped code rather than
    to a sentence: one clamped stock drawn by two flows, plus an unclamped source drawn
    by a third (which must contribute no constraint, decision #13).
    """
    pool = Stock(
        id=StockId("bio.pool"),
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=10.0,
        kind=StockKind.POOL,
    )
    source = Stock(
        id=StockId("boundary.src"),
        domain=DomainId("boundary"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=0.0,
        kind=StockKind.BOUNDARY,
        unclamped=True,
    )
    sink = Stock(
        id=StockId("boundary.sink"),
        domain=DomainId("boundary"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=0.0,
        kind=StockKind.BOUNDARY,
    )
    stocks = {s.id: s for s in (pool, source, sink)}
    results = [
        FlowResult(legs=(Leg(pool.id, -3.0), Leg(sink.id, 3.0))),
        FlowResult(legs=(Leg(pool.id, -1.0), Leg(sink.id, 1.0))),
        FlowResult(legs=(Leg(source.id, -999.0), Leg(sink.id, 999.0))),
    ]

    demand = 3.0 + 1.0
    margin = pool.amount / demand  # 2.5 — the census's number

    factors = arbitration._scale_factors(results, stocks)
    assert min(factors) == min(1.0, margin) == 1.0  # 2.5 -> no firing
    # ...and the same construction one step from firing reproduces the clamp exactly.
    tight = {**stocks, pool.id: replace(pool, amount=2.0)}
    tight_margin = 2.0 / demand  # 0.5
    assert min(arbitration._scale_factors(results, tight)) == tight_margin
    _scaled, fired = arbitration.min_scaling(results, tight)
    assert fired == 2  # both flows drawing the short stock; the source-drawer is free


def test_the_roster_is_exactly_the_two_manifests() -> None:
    """The roster comes from the manifests, never from a list typed here.

    Three prior findings in this project were "a scenario list checked against its own
    length rather than against the frozen set" (the (A)-diagnosis's 7-that-were-not-7,
    (B)-finding 4's five rows against seven, the chamber census's three rows). A census
    is exactly that shape, so the roster is derived and the derivation is asserted.
    """
    biosphere = set(json.loads(BIOSPHERE_MANIFEST.read_text("utf-8"))["scenarios"])
    station = set(json.loads(STATION_MANIFEST.read_text("utf-8"))["scenarios"])
    assert len(biosphere) == 7 and len(station) == 13

    covered = set(_runners()) | set(_DERIVED_FROM_OTHER_RUNS)
    assert covered == biosphere | station
    # every derived entry names runs that are themselves in the roster
    for derived, sources in _DERIVED_FROM_OTHER_RUNS.items():
        assert derived not in _runners()
        assert set(sources) <= set(_runners())


def test_the_energy_drift_runner_matches_its_golden_module() -> None:
    """The one runner reproduced here rather than imported (its original is a fixture).

    ⚠ Pinned against the committed module's **constants**, not its run body: this
    catches a changed scenario or horizon, and would NOT catch a changed integrator or
    resolver. Stated as the partial guard it is, not as "kept in step with that module".
    """
    import test_regression_sealed_station as sealed_station
    from station.scenario import HEAT_CLOSURE_SCENARIO, SEALED_ENERGY_DAYS

    assert sealed_station.HEAT_CLOSURE_SCENARIO is HEAT_CLOSURE_SCENARIO
    assert sealed_station.SEALED_ENERGY_DAYS == SEALED_ENERGY_DAYS


@pytest.mark.parametrize("scenario", ["greenhouse", "harvest", "lighting"])
def test_the_gate_fires_per_registry_call_not_per_simulated_step(scenario: str) -> None:
    """⚠ A property of the ENGINE's gate that the census inherits, stated not buried.

    ``greenhouse``/``harvest``/``lighting``/``sealed_station`` step one ``State``
    through **two registries**, so ``min_scaling`` fires once per registry call — the
    fast cabin or power registry many times per simulated day, the slow biosphere
    registry once. Each call sees only its own flows' demand against the same
    ``state.stocks``. So a recorded margin is the tightest *call*, which is exactly what
    the backstop protects, and is **not** headroom against a day's total draw.

    That matters because six stocks — ``carbon_pool`` among them — are demanded by
    *both* registries in ``greenhouse``/``harvest``, where the two registries' minima
    happen to **coincide** (16.667 either way), so no census number is affected.
    ``lighting`` shares nothing across its registries (its fast one is Power alone) and
    is the control. ``sealed_station`` is the case where they do **not** coincide and is
    pinned separately below — that one matters, because finding 4 rests on it.
    """
    rec = _recorder_for(scenario)
    fast, slow = rec.registries()
    assert rec.calls > sum(rec.signatures[s] for s in rec.signatures if s != fast)
    shared = fast & slow
    if scenario == "lighting":
        assert shared == frozenset(), shared
        return
    assert "biosphere.carbon_pool" in shared, sorted(shared)
    assert _registry_minima(rec, "biosphere.carbon_pool") == pytest.approx(
        (16.666666666666664, 16.666666666666664), rel=1e-12
    )
    assert census(scenario)["biosphere.carbon_pool"].min_margin == pytest.approx(
        16.666666666666664, rel=1e-12
    )


def _registry_minima(rec: _Recorder, sid: str) -> tuple[float, float]:
    """(fast-registry minimum, slow-registry minimum) for ``sid``."""
    fast, _slow = rec.registries()
    fast_mins = [
        m for (s, sig), m in rec.per_sig_min.items() if s == sid and sig == fast
    ]
    slow_mins = [
        m for (s, sig), m in rec.per_sig_min.items() if s == sid and sig != fast
    ]
    return min(fast_mins), min(slow_mins)


# ``sealed_station``'s own attribution — that its binding margin is the BIOSPHERE
# registry's call and not the cabin's — is claim 3 inside
# `test_the_roster_wide_claims_that_need_the_expensive_runs`, folded there so the Tier-2
# trajectory is computed once per worker. It is not omitted.


# --------------------------------------------------------------------------- #
# 1. the empty cell: open_season's carbon source cannot be gated               #
# --------------------------------------------------------------------------- #


def test_open_season_has_no_carbon_source_the_gate_can_see() -> None:
    """⚠ THE EMPTY CELL, and it is sharper than "unfalsifiable".

    The field crop's carbon source is ``boundary.co2_atmos``: ``unclamped`` (so
    arbitration skips it, decision #13) **and holding 0.0 mol C for the whole run** — a
    BOUNDARY source is a ledger entry, not a reservoir. Every clamped CARBON stock in
    the scenario is either the plant's own tissue or a sink. So ``assert rationed == 0``
    in ``test_regression_season.py`` is true by construction with respect to carbon: it
    reports that no flow out-ran a *tissue* pool, never that the crop was well fed.
    """
    rows = census("open_season")
    co2 = rows["boundary.co2_atmos"]
    assert co2.quantity is Quantity.CARBON
    assert co2.unclamped and co2.gate == "impossible:boundary"
    assert co2.amount0 == 0.0

    carbon = {sid: e for sid, e in rows.items() if e.quantity is Quantity.CARBON}
    gated = {sid: e for sid, e in carbon.items() if e.gate != "impossible:boundary"}
    # what remains is tissue + never-withdrawn sinks; no source among them
    assert all(
        e.kind is StockKind.POPULATION or e.gate == "impossible:never-withdrawn"
        for e in gated.values()
    ), gated
    assert {sid for sid, e in gated.items() if e.gate == "live"} == {
        "biosphere.leaf_c",
        "biosphere.stem_c",
        "biosphere.root_c",
    }


def test_the_obvious_fix_would_not_create_a_gate_either() -> None:
    """Refuses "just give the open field a finite CO2 pool" for a second, harder reason.

    Even a finite ``co2_atmos`` would not gate the open field, because open-field
    assimilation does not read the pool at all: ``sealed=False`` keeps the Phase-1
    **constant ``ci`` forcing**, and only ``sealed=True`` swaps in
    ``chamber.ci_from_co2_pool``. A gate needs both a clamped stock *and* a draw that
    depends on it. (The primary reason the fix is refused is neither: an open field
    genuinely has an unclamped atmosphere — modelling it as finite would model a
    different system.)
    """
    from domains.biosphere.scenario import DEFAULT_SCENARIO

    assert DEFAULT_SCENARIO.sealed is False
    # the pool-reading law exists but is chamber-only
    rows = census("open_season")
    assert "biosphere.carbon_pool" not in rows
    assert "biosphere.carbon_pool" in census("sealed_chamber")


def test_open_seasons_other_currencies_are_slack_not_absent() -> None:
    """⚠ KEEP THE QUALIFIER. The committed sentence says no *carbon*-scarcity gate.

    Water and nitrogen are live gates in ``open_season`` — they exist — they are simply
    slack by 2 and 5 orders of magnitude. That is a *different* reason from carbon's,
    and running the two together is how "no carbon gate" would silently widen into "no
    gate", this repo's most-repeated failure shape. Both reasons are recorded, apart.

    Note also that scarcity in this model is *designed* to act through the limitation
    factors (``f_water``, ``f_N``, ``f_O2``), not through the backstop — so a biting
    drought would throttle assimilation without ever moving ``rationed``.
    """
    rows = census("open_season")
    water = rows["biosphere.soil_water"]
    nitrogen = rows["biosphere.soil_n"]
    assert water.gate == "live" and nitrogen.gate == "live"
    water_margin, n_margin = water.min_margin, nitrogen.min_margin
    assert water_margin is not None and n_margin is not None
    assert 100.0 < water_margin < 1_000.0, water_margin  # 189.24
    assert 1e5 < n_margin < 1e6, n_margin  # 126_238.75


# --------------------------------------------------------------------------- #
# 2. rate-determined margins measure dt, not scarcity                          #
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    "scenario,stock,rate,label",
    [
        ("perennial_chamber", "biosphere.litter_carbon", 0.011, "decomposition_rate"),
        ("perennial_chamber", "biosphere.litter_n", 0.011, "decomposition_rate (B)"),
        ("perennial_chamber", "biosphere.water_vapor", 0.5, "condensation_rate"),
        ("perennial_chamber", "biosphere.condensate", 0.5, "recycling_rate"),
        # eclss.yaml names both products itself: "k_scrub*dt = 0.06" and "(here 0.03)".
        ("eclss_steady_state", "eclss.cabin_co2", 0.06, "co2_scrub_rate * dt"),
        ("eclss_steady_state", "eclss.cabin_h2o", 0.03, "condense_rate * dt"),
    ],
)
def test_a_first_order_stocks_margin_is_one_over_k_dt(
    scenario: str, stock: str, rate: float, label: str
) -> None:
    """A donor-controlled stock's margin is ``1/(k*dt)`` — the timestep, not scarcity.

    Such a stock can only ration when ``k*dt > 1``, so here ``rationed`` answers "is my
    timestep safe?", not "is the resource short?". ``water_cycle.yaml`` already says as
    much in its own prose ("rate*dt < 1 keeps the backstop off"); this pins it as a
    number and names the constant behind each one.

    ⚠ Scope, corrected from a draft that overreached: the multi-rate build-time
    ``k*h < 1`` precondition lives in ``authoring.interpreter._effective_step`` and
    fires when an **authored file** is interpreted. These scenarios are built directly
    in Python and never pass through ``interpret``, so on the frozen roster this is the
    only check of that inequality — the quantity checked is still the timestep, but it
    is not a redundancy here.

    ⚠ ``litter_n`` is here because option (B) put both litter currencies on the *same*
    flux, so it inherits the carbon rate and holds no rate of its own — that param file
    was deleted. Its presence here is the (B) build's identity, not a duplicate row.
    """
    entry = census(scenario)[stock]
    assert entry.gate == "rate-determined", (entry.min_margin, entry.max_margin)
    assert entry.min_margin == pytest.approx(1.0 / rate, rel=1e-12), label


def test_the_microbial_pair_shares_a_flux_and_the_census_sees_it() -> None:
    """Option (B)'s core identity, confirmed from an unrelated direction.

    (B) replaced the free-rate ``Mineralization`` with N legs *carried by* the carbon
    fluxes, so ``microbial_n`` and ``microbial_carbon`` leave on the same proportional
    draw. Their gate margins are therefore bit-identical — not approximately, exactly —
    and likewise for the litter pair. Neither is ``rate-determined`` here only because
    ``f_O2`` modulates the microbial draw.
    """
    for scenario in ("sealed_chamber", "perennial_chamber", "consumer_chamber"):
        rows = census(scenario)
        assert rows["biosphere.microbial_n"].min_margin == (
            rows["biosphere.microbial_carbon"].min_margin
        )
        assert rows["biosphere.litter_n"].min_margin == pytest.approx(
            rows["biosphere.litter_carbon"].min_margin, rel=1e-12
        )


# --------------------------------------------------------------------------- #
# 3. the one binding gate in the roster                                        #
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    "scenario,expected",
    [
        ("sealed_chamber", 1.4914109879410478),
        ("perennial_chamber", 1.1260232712494462),
        ("consumer_chamber", 1.8016106868650505),
    ],
)
def test_the_jars_carbon_pool_is_the_only_binding_gate(
    scenario: str, expected: float
) -> None:
    """The chamber-scale collision as a number: 11-80 % headroom, on one stock.

    This is what a *live* gate looks like — and it is the acceptance test every
    biosphere science change has actually been judged by.
    """
    rows = census(scenario)
    pool = rows["biosphere.carbon_pool"]
    assert pool.gate == "live"
    margin = pool.min_margin
    assert margin is not None
    assert margin == pytest.approx(expected, rel=1e-9)
    assert margin < 2.0
    # and it is that scenario's tightest, by a wide margin
    others = sorted(v for k, v in _live(rows).items() if k != "biosphere.carbon_pool")
    assert others[0] > 4 * margin, (margin, others[:3])


# The census table: scenario -> (tightest live stock, its minimum margin). Measured
# 2026-08-09 on the committed goldens. This IS the census — pinned as exact values
# because a drift here means either a golden moved or the gate's reach changed.
TIGHTEST: dict[str, tuple[str, float]] = {
    "open_season": ("biosphere.leaf_c", 42.50662430453055),
    "sealed_chamber": ("biosphere.carbon_pool", 1.4914109879410478),
    "perennial_chamber": ("biosphere.carbon_pool", 1.1260232712494462),
    "consumer_chamber": ("biosphere.carbon_pool", 1.8016106868650505),
    "power_bounded_soc": ("power.battery", 11.295323690100386),
    "power_self_discharge": ("power.battery", 11.085836827155921),
    "thermal_equilibrium": ("thermal.node", 257.68121326080376),
    "eclss_steady_state": ("eclss.cabin_o2", 33.333333333333364),
    "crew_mission": ("crew.food_store", 388.55555555555975),
    "station_heat_closure": ("power.battery", 11.295323690100386),
    "cabin_gas": ("eclss.cabin_o2", 35.57253249034074),
    "greenhouse": ("biosphere.carbon_pool", 16.666666666666664),
    "water_recovery": ("eclss.cabin_o2", 35.57253249034074),
    "lighting": ("biosphere.carbon_pool", 14.442998559750455),
    "harvest": ("biosphere.carbon_pool", 16.666666666666664),
    "perennial_long_horizon": ("biosphere.carbon_pool", 1.1260232712494462),
    "consumer_long_horizon": ("biosphere.carbon_pool", 1.8016106868650505),
    "sealed_energy_drift": ("power.battery", 11.295323690100386),
    "sealed_station": ("biosphere.carbon_pool", 5.218197631830118),
}


@pytest.mark.parametrize("scenario", FAST)
def test_the_census_row_for_each_fast_scenario(scenario: str) -> None:
    """Each scenario's tightest gate, by stock and by value."""
    stock, margin = TIGHTEST[scenario]
    live = _live(census(scenario))
    assert live, scenario
    got_stock = min(live, key=lambda k: live[k])
    assert got_stock == stock, (scenario, got_stock, live[got_stock])
    assert live[got_stock] == pytest.approx(margin, rel=1e-9)


@pytest.mark.slow
@pytest.mark.parametrize("scenario", sorted(_SLOW - {_TIER2}))
def test_the_census_row_for_each_slow_scenario(scenario: str) -> None:
    """The same, for the two 15-year runs and the 15-year energy run.

    ``sealed_station`` is deliberately absent: its row is checked inside
    :func:`test_the_roster_wide_claims_that_need_the_expensive_runs`, so that the Tier-2
    trajectory is recomputed once per worker rather than once per test. See that
    function's note.
    """
    stock, margin = TIGHTEST[scenario]
    live = _live(census(scenario))
    got_stock = min(live, key=lambda k: live[k])
    assert got_stock == stock, (scenario, got_stock, live[got_stock])
    assert live[got_stock] == pytest.approx(margin, rel=1e-9)


def test_the_census_table_covers_the_whole_roster() -> None:
    assert set(TIGHTEST) == set(_runners())


@pytest.mark.slow
def test_the_roster_wide_claims_that_need_the_expensive_runs() -> None:
    """⚠ THE ORDERING CLAIM over all 20 frozen scenarios — ``LIVE`` is load-bearing.

    **Why four claims share one test.** Each needs the whole roster, including the
    Tier-2 ``sealed_station`` run (~6 min). ``_CACHE`` is per *process*, and xdist's
    ``--dist load`` spreads tests across workers, so four separate tests can mean four
    Tier-2 recomputations. ``docs/test-suite-runtime.md`` measured that both routes to
    worker affinity are closed (a collection-hook group is silently dropped;
    ``loadgroup`` doubled the full run), so the remaining lever is *fewer tests needing
    it*. The cost is diagnosability, paid down with a labelled message per claim.

    Rank every **live** gate margin in the frozen roster. The six smallest are the same
    stock, ``biosphere.carbon_pool``, in the six scenarios that seal one. The first
    margin on any *other* stock is ``sealed_chamber``'s ``o2_pool`` — and that chamber
    is documented as deliberately O2-poor, so even the runner-up is a chamber property.

    ⚠ **The unqualified sentence is FALSE and the counterexample is in this roster.**
    Rank *all* classes and the 6th tightest is the water-cycle pair at 2.0, sitting
    between the 5th live entry (1.802) and the 6th (5.218). The claim holds only after
    the rate-determined exclusion — argued above and sound, but the qualifier is
    exactly the kind this repo has watched fall off a careful sentence five times while
    the paraphrase travelled, and a test *name* is the most-quoted paraphrase there is.
    So the name carries ``LIVE``, and the raw all-class ranking is asserted too: the
    exclusion is thereby a measured fact about what is removed, not an invisible filter.

    ⚠ A rank claim, deliberately, with **no threshold**: an earlier draft asserted
    "every margin below 9.0", a cut chosen by eye that lands *between* the 6th and 7th
    entries — and it was off by one, because the runner-up is 8.94. A number invented to
    separate two measurements is the fitted comparison this project refuses; the ranking
    needs no cut.
    """
    all_margins = [
        (entry.min_margin, scenario, sid, entry.gate)
        for scenario in _runners()
        for sid, entry in census(scenario).items()
        if entry.min_margin is not None
    ]
    # What the rate-determined exclusion removes, pinned rather than assumed.
    raw = sorted(all_margins)
    assert [sid for _m, _s, sid, _g in raw[:5]] == ["biosphere.carbon_pool"] * 5
    margin6, _s6, sid6, gate6 = raw[5]
    # the water-cycle pair ties at exactly 2.0; which of the two sorts 6th is incidental
    assert sid6 in {"biosphere.condensate", "biosphere.water_vapor"}, raw[5]
    assert gate6 == "rate-determined" and margin6 == 2.0, raw[5]

    ranked = sorted(
        (margin, scenario, sid)
        for scenario in _runners()
        for sid, margin in _live(census(scenario)).items()
    )
    assert [sid for _m, _s, sid in ranked[:6]] == ["biosphere.carbon_pool"] * 6, ranked[
        :8
    ]
    assert {s for _m, s, _sid in ranked[:6]} == {
        "sealed_chamber",
        "perennial_chamber",
        "consumer_chamber",
        "perennial_long_horizon",
        "consumer_long_horizon",
        "sealed_station",
    }
    margin, scenario, sid = ranked[6]
    assert (scenario, sid) == ("sealed_chamber", "biosphere.o2_pool"), "runner-up"
    assert margin == pytest.approx(8.944335455310334, rel=1e-9), "runner-up value"

    # --- claim 2: sealed_station's census row (folded in; see the note above) --------
    stock, expected = TIGHTEST[_TIER2]
    live = _live(census(_TIER2))
    got = min(live, key=lambda k: live[k])
    assert got == stock, ("tier2 row", got, live[got])
    assert live[got] == pytest.approx(expected, rel=1e-9), "tier2 row value"

    # --- claim 3: that row is the PLANT's draw, not the cabin's ----------------------
    # It is a multi-registry scenario, so the row would not belong in the ordering claim
    # above if its minimum came from the fast cabin registry. The biosphere registry's
    # daily call produces 5.218; the cabin's minimum on the same stock is 16.667, the
    # 1/(k*dt) value greenhouse/harvest sit at. So it measures the same quantity as the
    # standalone chambers' 1.126/1.491/1.802.
    fast_min, slow_min = _registry_minima(
        _recorder_for(_TIER2), "biosphere.carbon_pool"
    )
    assert slow_min == pytest.approx(5.218197631830118, rel=1e-9), "tier2 slow registry"
    assert fast_min == pytest.approx(16.666666666666664, rel=1e-12), (
        "tier2 fast registry"
    )
    assert slow_min < fast_min, "tier2: the binding call must be the biosphere's"
    assert census(_TIER2)["biosphere.carbon_pool"].min_margin == slow_min

    # --- claim 4: the metric's consistency check against the gate it measures --------
    # Every frozen golden asserts `rationed == 0`, so no stock in any frozen scenario
    # may reach a margin of 1. A failure means either a golden started rationing or the
    # recorder is not measuring what the backstop reads.
    for name in _runners():
        for sid_, entry in census(name).items():
            if entry.min_margin is not None:
                assert entry.min_margin > 1.0, ("above firing point", name, sid_)


@pytest.mark.slow
def test_tripling_the_horizon_does_not_tighten_the_gate() -> None:
    """The gate's minimum is reached inside the first 5 years, bit-for-bit.

    ``test_chamber_scale.py`` pinned that the long-horizon goldens reuse the same
    scenario objects, so the *inventory* is bit-identical at 5 and 15 years. The gate's
    *margin* is not a t=0 property and had to be run — and it comes back identical too:
    the horizon lengthens the run, not the jar.
    """
    for short, long_ in (
        ("perennial_chamber", "perennial_long_horizon"),
        ("consumer_chamber", "consumer_long_horizon"),
    ):
        a = census(short)["biosphere.carbon_pool"].min_margin
        b = census(long_)["biosphere.carbon_pool"].min_margin
        assert a is not None and b is not None
        assert a.hex() == b.hex(), (short, long_, a, b)


# --------------------------------------------------------------------------- #
# 4. the contract has no plausibility column                                   #
# --------------------------------------------------------------------------- #


def test_a_manifest_scenario_entry_carries_no_plausibility_criterion() -> None:
    """What a frozen scenario is contractually required to satisfy, structurally.

    A manifest entry names a golden and its hash (plus, for the biosphere, a year
    count). There is no field for "the crop must be physical".

    ⚠ **Still true, and no longer the whole story — read it with the test below.** When
    this was written it carried the conclusion "so the acceptance set is {golden bytes,
    ``rationed == 0``, no extinction, conservation, determinism}, all properties of the
    *run*". Since 2026-08-09 the science *does* have standing, via top-level
    ``science_bands`` / ``liveness_floors`` fields keyed by scenario rather than via a
    column inside the entry. So the structural claim below is unchanged — the entry's
    key set really is untouched — while the conclusion drawn from it in the original
    docstring is now false. Kept, because the entry shape is still worth pinning; the
    conclusion moved to where it can be checked.
    """
    rosters = ((BIOSPHERE_MANIFEST, {"years"}), (STATION_MANIFEST, set()))
    for manifest_path, extra in rosters:
        manifest = json.loads(manifest_path.read_text("utf-8"))
        for key, entry in manifest["scenarios"].items():
            assert set(entry) == {"golden", "golden_sha256", "scenario"} | extra, key


def test_the_plausibility_bands_are_now_named_by_a_manifest() -> None:
    """⚠ **THE INVERSE of the pin this replaced, and the replaced pin was not wrong.**

    Until 2026-08-09 this asserted the opposite — that no manifest names
    ``test_senescence_form`` or ``test_nitrogen_form``, so the bands were *records, not
    gates*. That was a true measurement of the contract as it then stood, and it is the
    measurement the adjudication acted on (finding 5 → ``science_bands``). It is
    **resolved, not corrected**: the mechanism it described was removed on purpose.

    It is replaced by its inverse rather than deleted or relaxed, on the option-(B)
    precedent — *a pin guarding a mechanism you removed is decoration*. What must not
    silently regress now is the standing itself: both loci must be reachable from a
    manifest, and the band must still be where this says it is.
    """
    named = ("test_senescence_form", "test_nitrogen_form")
    text = BIOSPHERE_MANIFEST.read_text("utf-8")
    for name in named:
        assert name in text, name
    # ``test_chamber_scale`` stays OUT: its BVAD comparisons are characterizations, not
    # gates — a chamber resized *toward* the flight spec fails them. See the inclusion
    # rule in docs/plans/post-roadmap-acceptance-gate-standing.md.
    assert "test_chamber_scale" not in text
    # ...and the band itself is still where this says it is.
    band_src = (_REPO_ROOT / "tests" / "test_senescence_form.py").read_text("utf-8")
    assert "assert 5.0 < peak < 8.0" in band_src


# --------------------------------------------------------------------------- #
# 5. the crew-coupled route, in the census's own unit                          #
# --------------------------------------------------------------------------- #

# [BVAD] Table 4-91 wheat CO2 uptake, 77.00 g CO2/m2/d = 1.7496 mol C/m2/d — the figure
# `test_chamber_scale.py` reads off the page renders (pdftotext scrambles that table).
# Imported rather than re-derived, so the two documents cannot drift apart.


def test_the_crew_coupled_route_holds_years_where_the_jar_holds_days() -> None:
    """Priced, not proposed — the alternative to the soil-fractionation seam.

    ``crew.food_store`` is a CARBON pool of 4000 mol in the greenhouse/harvest
    scenarios, and the loop back into it exists in the tree (``station.harvest``:
    ``storage_c -> food_store``). In the chamber census's own unit — days of one m2 of
    wheat's CO2 uptake — that is **years**, against the sealed chamber's 2.01 days.

    ⚠ What this does NOT show: those scenarios run 7 days with a seedling, are
    station-side and non-frozen, and no field-scale crop has ever been run against that
    store. The inventory is available; the demonstration is not.
    """
    from test_chamber_scale import BVAD_WHEAT_CO2_UPTAKE_G_PER_M2_D

    # 77.00 g CO2 / 44.0095 g/mol = 1.7496 mol CO2/m2/d; one mol CO2 carries one mol C.
    mol_c_per_m2_d = BVAD_WHEAT_CO2_UPTAKE_G_PER_M2_D / 44.0095
    assert mol_c_per_m2_d == pytest.approx(1.7496, abs=5e-4)

    food_store = census("greenhouse")["crew.food_store"]
    assert food_store.quantity is Quantity.CARBON
    assert food_store.amount0 == 4000.0
    days = food_store.amount0 / mol_c_per_m2_d
    assert days / 365.0 > 6.0, days
    # ordering against the jar, which `test_chamber_scale` measures at ~2.01 days
    assert days > 1000.0 * 2.01
