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
   ``biosphere.carbon_pool``. The **five** smallest **live** margins anywhere in the
   roster are that stock, in the five *standalone* chamber runs.
   ⚠ It was **six** until 2026-08-14 — the sixth being ``sealed_station``, the one
   sealed scenario that also runs a cabin. The step unfreeze multiplied the plant's
   per-call draw by ~4 and left the cabin's scrubber check alone, so on the station the
   two crossed and the binding call is now the cabin's. Same stock, different registry:
   the station's ``rationed == 0`` no longer answers a question about the plant. Pinned
   in ``test_the_roster_wide_claims_that_need_the_expensive_runs`` (claim 3).
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
from domains.biosphere.step import BIO_DT, STEPS_PER_DAY
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
    *both* registries in ``greenhouse``/``harvest``.
    ``lighting`` shares nothing across its registries (its fast one is Power alone) and
    is the control. ``sealed_station`` is pinned separately below — that one matters,
    because finding 4 rests on it.

    ⚠⚠ **A RETRACTION, 2026-08-14.** This docstring used to say the two registries'
    minima "happen to **coincide** (16.667 either way), so no census number is
    affected". They do not coincide, and — measured, not inferred — they never did:

        greenhouse biosphere registry   308.158 at dt = 1   1150.749 at dt = 1/4
        harvest    biosphere registry   376.409 at dt = 1   1483.263 at dt = 1/4
        both       cabin registry        16.667 at either

    The dt = 1 column was measured by checking out the pre-unfreeze tree (``c419e14``)
    and applying the corrected split to it, rather than by dividing the new numbers by
    four — the arithmetic would have given the right verdict here for a reason that is
    not always available, and this repo keeps a note about asserted attributions
    rotting. Note the columns are not a clean 4x (308.158 * 4 = 1232.6 against the
    measured 1150.7), so the inference really was doing work.

    16.667 is the cabin's ``1/(k*dt)`` scrubber check. The sentence was reading the
    cabin twice, because :func:`_registry_minima` split the calls by frequency rather
    than by content — see its docstring for how. The old helper returns
    ``(16.667, 16.667)`` on the pre-unfreeze tree too, which is the direct evidence
    that this was wrong from the day it was written and not something the step change
    broke.

    The conclusion the sentence was supporting **survives**: no census number is
    affected, because the cabin's 16.667 is genuinely the smaller of the two and so is
    genuinely what the census reports. Only the reason was wrong, and it was wrong in
    the direction that mattered — it claimed the biosphere was *also* at 16.667, which
    would have made ``greenhouse``'s row a plant measurement. It is not one.
    """
    rec = _recorder_for(scenario)
    fast, slow = rec.registries()
    assert rec.calls > sum(rec.signatures[s] for s in rec.signatures if s != fast)
    shared = fast & slow
    if scenario == "lighting":
        assert shared == frozenset(), shared
        return
    assert "biosphere.carbon_pool" in shared, sorted(shared)
    # ⚠ was `== approx((16.667, 16.667))` — see the retraction above. The two numbers
    # are three orders apart, and pinning BOTH is what keeps them from being confused
    # again: the cabin's is a timestep constant, the biosphere's is a trajectory.
    cabin_min, bio_min = _registry_minima(rec, "biosphere.carbon_pool")
    assert cabin_min == pytest.approx(16.666666666666664, rel=1e-12)
    # ⚠ both halved by the light path (2026-08-14): 1150.7494 -> 537.8638 and
    # 1483.2630 -> 693.4340. The biosphere carbon pool now spends part of every day
    # being drawn on by respiration with no assimilation to offset it, so its tightest
    # moment is tighter. Three orders of slack becomes two and a half; still live.
    expected_bio = {"greenhouse": 537.8638049097283, "harvest": 693.4339678481122}
    assert bio_min == pytest.approx(expected_bio[scenario], rel=1e-9)
    # ...and the census reports the tighter of the two, which is the cabin's.
    assert cabin_min < bio_min
    assert census(scenario)["biosphere.carbon_pool"].min_margin == pytest.approx(
        16.666666666666664, rel=1e-12
    )


@pytest.mark.parametrize(
    "scenario,fast_per_day",
    [("greenhouse", 1440), ("harvest", 1440), ("lighting", None)],
)
def test_the_two_registries_are_separated_by_what_they_demand(
    scenario: str, fast_per_day: int | None
) -> None:
    """The discriminator :func:`_registry_minima` uses, asserted not assumed.

    It classifies a call as the biosphere registry's when the call demands **only**
    ``biosphere.*`` stocks. That is a claim about the roster, not a definition, so it is
    checked here against a quantity the driver fixes independently: ``run_master_day``
    takes ``STEPS_PER_DAY`` biosphere sub-steps and ``steps_per_day`` fast sub-steps per
    master day, so the two groups' call counts must stand in exactly that ratio. A cabin
    phase leaking into the biosphere group — the bug this replaced — breaks the ratio,
    because the leaked calls are counted at the fast rate.

    ``lighting`` is the control and is checked differently: its fast registry is Power
    alone, so the two groups share no stock at all and the ratio has nothing to say.
    """
    rec = _recorder_for(scenario)
    bio_calls = sum(
        n
        for sig, n in rec.signatures.items()
        if sig and all(s.startswith("biosphere.") for s in sig)
    )
    other_calls = rec.calls - bio_calls
    assert bio_calls and other_calls, (bio_calls, other_calls)
    # every group member is internally consistent: no biosphere-group signature names a
    # non-biosphere stock, by construction, so the content half is the count half below.
    assert bio_calls % STEPS_PER_DAY == 0, bio_calls
    days = bio_calls // STEPS_PER_DAY
    if fast_per_day is None:
        fast, slow = rec.registries()
        assert fast & slow == frozenset(), sorted(fast & slow)
        return
    assert other_calls == days * fast_per_day, (days, other_calls)


def _registry_minima(rec: _Recorder, sid: str) -> tuple[float, float]:
    """(non-biosphere-registry minimum, biosphere-registry minimum) for ``sid``.

    ⚠⚠ **CORRECTED 2026-08-14, and it had been reading the same registry TWICE.** This
    used to split the call signatures as "the most frequent one" versus "all the rest",
    on the reasoning that the fast registry supplies the overwhelming majority of calls
    so the dominant signature is it. The first half is true; the second does not follow.
    A fast registry has *phases* too — before a crew store comes online its demand set
    is a strict subset — so ``greenhouse`` has a 1-call cabin signature and ``harvest``
    three, and those landed in "all the rest" alongside the biosphere's. Since a cabin
    call is far tighter than a biosphere one there, ``min(rest)`` returned the cabin's
    number and the "slow registry" reading was the fast registry wearing a hat.

    The step unfreeze is what made it visible rather than merely wrong: the biosphere's
    minima moved by ~4x and the cabin's did not move at all, and the pin did not budge.
    That is the signature of a measurement that was never looking where it said. It is
    confirmed directly — the old helper returns ``(16.667, 16.667)`` on the
    pre-unfreeze tree as well, where the biosphere's true minima are 308.158 and
    376.409.

    The split is now by what a call DEMANDS: a biosphere-registry call demands only
    ``biosphere.*`` stocks, while every cabin/power/thermal call in the roster reaches
    at least one of its own domain's. Verified on all four multi-registry scenarios —
    ``greenhouse``/``harvest``/``lighting``/``sealed_station`` — by inspecting the
    signatures directly, and it is asserted rather than assumed by
    :func:`test_the_two_registries_are_separated_by_what_they_demand`.
    """
    bio = [
        m
        for (s, sig), m in rec.per_sig_min.items()
        if s == sid and sig and all(x.startswith("biosphere.") for x in sig)
    ]
    other = [
        m
        for (s, sig), m in rec.per_sig_min.items()
        if s == sid and not (sig and all(x.startswith("biosphere.") for x in sig))
    ]
    return min(other), min(bio)


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
    the scenario is either the plant's own carbon or a sink. So ``assert rationed == 0``
    in ``test_regression_season.py`` is true by construction with respect to carbon: it
    reports that no flow out-ran a *tissue* pool, never that the crop was well fed.

    ⚠ **The stem reserve (2026-08-12) is the first exception to the POPULATION shorthand
    this test used to lean on, and it is a real one rather than a bookkeeping quibble.**
    ``stem_reserve_c`` is clamped, is genuinely withdrawn from, and is a **POOL** — a
    carbohydrate store is not a population that can go extinct. What licensed the
    original predicate was never the ``StockKind`` label but the property behind it:
    the only withdrawal is **first-order in the stock itself** (``rate · reserve``), so
    it approaches zero as the stock does and can never out-run it. The reserve has that
    property by construction, so it is named here explicitly instead of the predicate
    being loosened to "any POOL", which would have let a genuine source through.

    ⚠ **AND THE CENSUS SAW IT WITHOUT BEING TOLD — my first draft of this pin was wrong
    in a way worth keeping.** I assumed the reserve would land in the ``live`` set
    beside
    the three tissue pools and wrote it in. It does not: the gate classifies it
    ``rate-determined``, with a margin of **exactly 1/(k·dt)**, because that is
    what
    a first-order self-limiting draw IS. So the structural property the paragraph above
    argues for by hand is one the census already measures, and the assertion below reads
    it rather than restating it. Asserting the margin equals 1/(k·dt) is the strong
    form:
    it would go red if the draw ever stopped being first-order in the reserve.

    ⚠ **The value used to be written here as ``= 10`` and is now ``= 40`` (2026-08-14,
    the step unfreeze) — and the docstring no longer names either.** ``1/(k·dt)`` is the
    claim; a number is only that claim evaluated at one step size, and this file had it
    both in prose and in an assertion that spelled ``1.0 / 0.1`` with ``dt`` silently
    equal to 1. That is the same shape as the ``_is_one_over_k_dt`` bug two sections
    below: the ``dt`` was in the name and missing from the arithmetic.
    """
    rows = census("open_season")
    co2 = rows["boundary.co2_atmos"]
    assert co2.quantity is Quantity.CARBON
    assert co2.unclamped and co2.gate == "impossible:boundary"
    assert co2.amount0 == 0.0

    # The one clamped, withdrawn, non-POPULATION carbon stock — allowed by name, with
    # its self-limiting property asserted rather than assumed (below).
    donor_controlled = {"biosphere.stem_reserve_c"}

    carbon = {sid: e for sid, e in rows.items() if e.quantity is Quantity.CARBON}
    gated = {sid: e for sid, e in carbon.items() if e.gate != "impossible:boundary"}
    # what remains is the plant's own carbon + never-withdrawn sinks; no source
    assert all(
        e.kind is StockKind.POPULATION
        or e.gate == "impossible:never-withdrawn"
        or sid in donor_controlled
        for sid, e in gated.items()
    ), gated
    # ⚠ UNCHANGED from before the reserve: the three tissue pools, and only those.
    assert {sid for sid, e in gated.items() if e.gate == "live"} == {
        "biosphere.leaf_c",
        "biosphere.stem_c",
        "biosphere.root_c",
    }
    # ...and the reserve is separated STRUCTURALLY rather than by the name-list above,
    # which is only a backstop. 1/(k*dt), k = 0.1 /day, dt = BIO_DT — written as the
    # formula so the step stays visible; at dt = 1/4 it reads 40, at dt = 1 it read 10.
    reserve = rows["biosphere.stem_reserve_c"]
    assert reserve.kind is StockKind.POOL
    assert reserve.gate == "rate-determined"
    assert reserve.min_margin == pytest.approx(1.0 / (0.1 * BIO_DT), rel=1e-12)


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
    slack. That is a *different* reason from carbon's, and running the two together is
    how "no carbon gate" would silently widen into "no gate", this repo's most-repeated
    failure shape. Both reasons are recorded, apart.

    Note also that scarcity in this model is *designed* to act through the limitation
    factors (``f_water``, ``f_N``, ``f_O2``), not through the backstop — so a biting
    drought would throttle assimilation without ever moving ``rationed``.

    ⚠ **RE-MEASURED 2026-08-12 (the soil-water re-basing), and WATER'S SLACK FELL BY
    20x: 189.24 -> 9.31.** The docstring used to say "slack by 2 and 5 orders of
    magnitude"; water is now slack by *one*. Nothing about the gate changed — the store
    did. ``soil_water`` used to hold 1000 kg over 1 m2, which is 1000 mm of extractable
    water, i.e. a 7.7 m soil column; it now holds what the root zone can physically
    hold. A margin of 189x was never a fact about safety, it was a fact about a bucket
    that could not exist. This is the honest number, and it is still an order of
    magnitude of headroom.

    ⚠⚠ **RE-STATED IN DAYS 2026-08-14 (the step unfreeze), and this is the correction
    that matters most in this file.** A census margin is ``stock / demand-per-CALL``, so
    it is denominated in **steps**, not in time. Quartering the step quartered every
    demand and multiplied every margin by ~4 — 9.31 became 37.26 — while nothing about
    the water moved at all. The old bound ``9.0 < margin < 10.0`` was therefore a claim
    about the integrator's step wearing the clothes of a claim about soil.

    So both bounds are multiplied by ``BIO_DT`` and read in DAYS, which is the quantity
    the sentence was always about: **the root zone covers 9.31 days of peak draw**. That
    number is unchanged to sixteen digits across the step change, and the pin now
    survives the next one. Nitrogen's is not bit-stable (126 238 -> 126 574 day-units,
    0.3 %) because its trajectory genuinely differs at the finer step; water's is,
    because water's tightest moment is the run's first call, on the initial store — the
    same reason the identical 9.3139 shows up in three unrelated scenarios.
    """
    rows = census("open_season")
    water = rows["biosphere.soil_water"]
    nitrogen = rows["biosphere.soil_n"]
    assert water.gate == "live" and nitrogen.gate == "live"
    water_margin, n_margin = water.min_margin, nitrogen.min_margin
    assert water_margin is not None and n_margin is not None
    # ⚠ in DAYS of peak draw (margin * dt), not in steps — see the docstring.
    assert 9.0 < water_margin * BIO_DT < 10.0, water_margin  # 9.31 (189.24 pre-rebase)
    assert 1e5 < n_margin * BIO_DT < 1e6, n_margin  # 126_573.61 (was 126_238.75)


# --------------------------------------------------------------------------- #
# 2. rate-determined margins measure dt, not scarcity                          #
# --------------------------------------------------------------------------- #


# ⚠ The `rate` column carries TWO different things, and conflating them is what broke
# this test at `dt = ¼` (2026-08-14). The biosphere rows are a bare per-day `k` and need
# the biosphere's step to become the `k·dt` the margin actually inverts; the ECLSS rows
# are already the **product** `k·dt` (their labels say so, and eclss.yaml names them),
# on the cabin's own dt, which this ceremony did not touch. So the step is a column, not
# a global multiplier. The test was named `..._is_one_over_k_dt` all along — the `dt`
# was in its title and missing from its arithmetic, invisible while `dt` was 1.
@pytest.mark.parametrize(
    "scenario,stock,rate,dt,label",
    [
        ("perennial_chamber", "biosphere.water_vapor", 0.5, BIO_DT, "condensation"),
        ("perennial_chamber", "biosphere.condensate", 0.5, BIO_DT, "recycling_rate"),
        # eclss.yaml names both products itself: "k_scrub*dt = 0.06" and "(here 0.03)".
        ("eclss_steady_state", "eclss.cabin_co2", 0.06, 1.0, "co2_scrub * dt"),
        ("eclss_steady_state", "eclss.cabin_h2o", 0.03, 1.0, "condense * dt"),
    ],
)
def test_a_first_order_stocks_margin_is_one_over_k_dt(
    scenario: str, stock: str, rate: float, dt: float, label: str
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

    ⚠ **The litter pair used to be here and no longer is (2026-08-10).** Option (B) put
    both litter currencies on the same flux, so ``litter_n`` inherited the carbon rate
    and
    the pair sat at exactly ``1/0.011 = 90.909``. The humification split gave
    ``Decomposition`` a CO₂ leg, an O₂ draw comes with it, and an O₂-drawing flow must
    self-throttle — so the pair now carries ``f_O2`` and is **live** (90.952, varying in
    the last digits with the O₂ pool) rather than rate-determined. They moved into
    exactly
    the category the microbial pair already occupied, and for exactly the reason the
    next
    test's docstring already gave for it. Pinned in
    :func:`test_the_litter_pair_became_live_when_it_gained_an_o2_draw`.
    """
    entry = census(scenario)[stock]
    assert entry.gate == "rate-determined", (entry.min_margin, entry.max_margin)
    assert entry.min_margin == pytest.approx(1.0 / (rate * dt), rel=1e-12), label


def test_the_microbial_pair_shares_a_flux_and_the_census_sees_it() -> None:
    """Option (B)'s core identity, confirmed from an unrelated direction.

    (B) replaced the free-rate ``Mineralization`` with N legs *carried by* the carbon
    fluxes, so ``microbial_n`` and ``microbial_carbon`` leave on the same proportional
    draw. Their gate margins are therefore bit-identical — not approximately, exactly —
    and likewise for the litter pair. Neither is ``rate-determined`` here only because
    ``f_O2`` modulates the microbial draw.

    ⚠ **The humus pair (2026-08-10) is the same identity on a third pool**, and it is
    free corroboration a second time: nothing about the humification split was designed
    to
    make this test pass, yet ``HumusNitrogenRelease`` rides ``HumusDecomposition``'s
    flux
    and the census sees it without being told. It is pinned at 1 ULP rather than
    bit-exactly, for the same reason the litter pair is: the two flows reach the same
    algebraic quantity by different float paths (``humus_c/decayed`` versus
    ``humus_n/(decayed*humus_n/humus_c)``), and an exact-equality pin here would be
    asserting an association order, not an identity.
    """
    for scenario in ("sealed_chamber", "perennial_chamber", "consumer_chamber"):
        rows = census(scenario)
        # ⚠ Was bit-exact until 2026-08-10; now 1 ULP, like its two siblings. The
        # exactness was a property of the NUMBERS, not of the design: both flows compute
        # ``pool/(flux)`` by different float paths (``c/turned`` versus
        # ``n/(turned*n/c)``) and happened to round identically while each had a single
        # destination leg. The humification split gave both a second leg, the demand
        # fold
        # associates differently, and the last bit parts company. Asserting ``==`` here
        # was therefore asserting an association order; 1 ULP is the identity.
        assert rows["biosphere.microbial_n"].min_margin == pytest.approx(
            rows["biosphere.microbial_carbon"].min_margin, rel=1e-12
        )
        assert rows["biosphere.litter_n"].min_margin == pytest.approx(
            rows["biosphere.litter_carbon"].min_margin, rel=1e-12
        )
        assert rows["biosphere.humus_n"].min_margin == pytest.approx(
            rows["biosphere.humus_carbon"].min_margin, rel=1e-12
        )


def test_the_litter_pair_became_live_when_it_gained_an_o2_draw() -> None:
    """The one census row the humification split moved between CATEGORIES.

    Before 2026-08-10 ``litter_carbon``/``litter_n`` were ``rate-determined`` at exactly
    ``1/(k*dt) = 90.909``: a pure timestep check, flat over the whole run. The split
    gave
    ``Decomposition`` a CO2 leg; the composition gate forces an O2 draw with it; an
    O2-drawing flow must self-throttle or ``rationed == 0`` stops being structural. So
    the
    draw now carries ``f_O2``, the margin varies with the O2 pool, and the row is live.

    The size of the effect is the point: ``f_O2`` is ~0.9995 at the chamber's fill, so
    the
    margin moved from 90.909 to ~90.952 and *varies in the last digits*. This is not a
    scarcity gate appearing — it is a timestep check that stopped being exactly flat.
    The
    diagnosis's own warning applies in reverse here: filing this as "live" on a 4e-4
    relative wobble would overstate it just as ``min == max`` overstated flatness.
    """
    rows = census("perennial_chamber")
    for sid in ("biosphere.litter_carbon", "biosphere.litter_n"):
        entry = rows[sid]
        assert entry.gate == "live"
        margin = entry.min_margin
        assert margin is not None
        # ⚠ 90.95231882247269 -> 90.95231898732729 (2026-08-12, stem reserves). It moved
        # in the SIXTEENTH digit — 1.8e-10 relative — because this margin is set by the
        # rate constant and only perturbed by the trajectory. The assertion below is the
        # one that carries the claim, and it did not move at all.
        #
        # ⚠ 90.95231898732729 -> this (2026-08-14, the step unfreeze). The move is a
        # near-exact 4x, as it must be for a margin whose denominator is one step's
        # demand — but only NEAR-exact (90.95231898732729 * 4 = 363.80927594930916
        # against the measured 363.809291983146, parting company in the 8th digit).
        # That residue is the ``f_O2`` wobble this test is named for, and it is the
        # reason the row is `live` rather than `rate-determined`: a pure 1/(k*dt) row
        # would have rescaled to the last bit.
        # ⚠ 2026-08-14: parts company in the 7th digit rather than the 8th.
        assert margin == pytest.approx(363.80928788521334, rel=1e-9)
        # still within 0.05 % of the bare 1/(k*dt) it used to sit on exactly — written
        # as the formula, with dt in it, so the bound does not silently become a
        # different claim at the next step change (it was `1.0 / 0.011`, dt implicit).
        bare = 1.0 / (0.011 * BIO_DT)
        assert abs(margin - bare) / bare < 5e-4


# --------------------------------------------------------------------------- #
# 3. the one binding gate in the roster                                        #
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    "scenario,expected,runner_up",
    [
        # ⚠ all three tightest-gate margins re-measured TWICE on 2026-08-12 — the
        # stem-reserve build, then its cessation window. Was 2.3404741281202655 /
        # 1.5124880369468734 / 2.112066494173573 before the build. The RUNNER-UP
        # column is unchanged to the last digit in all three rows through BOTH moves,
        # because it is a WATER margin and neither touched water — the same
        # falsifiable half that held in the golden prediction, and it has held twice.
        #
        # ⚠ RE-MEASURED AGAIN 2026-08-14 (the step unfreeze). Was 1.9016721361221138 /
        # 1.552788483797351 / 2.1271916795585084 against 9.313939636232975 /
        # 8.437936564620642 / 8.437936564620642. THE RUNNER-UP COLUMN'S THREE-DAY RUN
        # OF NOT MOVING ENDS HERE — every number in the table moved — and the SHAPE of
        # how each moved is the readable part:
        #   sealed_chamber's runner-up is EXACTLY 4x (9.313939636232975 * 4 ==
        #     37.2557585449319, bit-exact), the open-fed soil store's signature: its
        #     tightest moment is the run's FIRST call, so only the per-call demand
        #     changed and the stock it divides is the declared initial amount.
        #   perennial/consumer's runner-up is 4.0034x, not 4x (8.437936564620642 * 4 is
        #     33.751746 against the measured 33.779983). Same stock, same currency —
        #     but a sealed chamber recycles its water, so the tightest moment is inside
        #     the trajectory and the trajectory genuinely re-integrated.
        #   the carbon column moved by 4.04 / 3.59 / 4.36 — all live, none of them 4x.
        # The stock IDENTITY is unchanged in all six cells.
        # ⚠ all three carbon margins roughly HALVE on 2026-08-14 (the light path):
        # 7.6877 -> 3.7794, 5.5755 -> 3.5343, 9.2680 -> 4.4670. The night half of the
        # day draws the pool with no assimilation against it, which is exactly what
        # this column measures. The RANK claim the table exists for is unchanged in
        # all three: the carbon pool is still each scenario's tightest live gate, and
        # the runner-up columns did not move at all.
        ("sealed_chamber", 3.7793861201161016, 37.2557585449319),
        ("perennial_chamber", 3.5343019459308773, 33.77998307475926),
        ("consumer_chamber", 4.466989660058212, 33.77998307475926),
    ],
)
def test_the_jars_carbon_pool_is_the_only_binding_gate(
    scenario: str, expected: float, runner_up: float
) -> None:
    """The chamber-scale collision as a number: 51-134 % headroom, on one stock.

    This is what a *live* gate looks like — and it is the acceptance test every
    biosphere science change has actually been judged by.

    ⚠ **RE-MEASURED 2026-08-10 (the humification split), and the headroom LOOSENED**:
    1.126/1.491/1.802 -> 1.512/2.340/2.112, so the docstring's "11-80 %" became "51-134
    %"
    and the ``margin < 2.0`` bound became false for two of the three. The cause is
    mechanical and is worth stating rather than absorbing: the split returns 45 % of
    decayed litter carbon to the atmosphere immediately instead of routing all of it
    through a microbial pool with a ~62-day residence time, so the CO2 trough is higher.
    ⚠ That is NOT a benefit to quote on its own — the trough is also higher because the
    plant is ~40 % smaller (see the liveness floor in ``test_decade_stability``). The
    two
    facts travel together.

    The ``< 2.0`` bound is REPLACED BY A RANK rather than re-tuned upward: an amplitude
    cut chosen after seeing the measurement is the fitted comparison this file already
    refuses once (the "every margin below 9.0" draft, off by one). What makes this stock
    the binding gate is that it is the tightest in its scenario, and that is what is
    asserted below.

    ⚠ **THE "BY A WIDE FACTOR" HALF DIED ON 2026-08-12, AND IS NOT BEING RE-TUNED.**
    It asserted ``runner_up > 4 * margin``. After the soil-water re-basing the runner-up
    is ``soil_water``/``subsoil_water`` at 9.31/8.44 instead of ~30, so the ratio is
    3.98 on ``sealed_chamber`` and 4.00 on ``consumer_chamber`` — it fails, barely.
    Loosening 4 to 3.5 would be exactly the fitted cut the paragraph above refuses, so
    the factor is dropped and replaced by something strictly stronger: the RANK (this
    stock is the minimum) plus the runner-up pinned as an EXACT value. A threshold can
    only catch a change bigger than its slack; an exact pin catches every change.
    """
    rows = census(scenario)
    pool = rows["biosphere.carbon_pool"]
    assert pool.gate == "live"
    margin = pool.min_margin
    assert margin is not None
    assert margin == pytest.approx(expected, rel=1e-9)
    # It is that scenario's tightest — the claim the test is named for.
    others = sorted(v for k, v in _live(rows).items() if k != "biosphere.carbon_pool")
    assert others[0] > margin, (margin, others[:3])
    # ...and the runner-up is pinned exactly, so the GAP is a measurement rather than a
    # threshold. Post-re-basing this is soil_water (open-fed) or subsoil_water (sealed).
    assert others[0] == pytest.approx(runner_up, rel=1e-9), (margin, others[:3])


# The census table: scenario -> (tightest live stock, its minimum margin). Measured
# 2026-08-09 on the committed goldens; the six sealed-chamber rows and `lighting`
# RE-MEASURED 2026-08-10 after the humification split. This IS the census — pinned as
# exact values
# because a drift here means either a golden moved or the gate's reach changed.
TIGHTEST: dict[str, tuple[str, float]] = {
    # ⚠ CHANGED 2026-08-11 (soil layers): `open_season`'s tightest live gate is no
    # longer `leaf_c` (42.50662430453055) but the NEW `subsoil_water` store, at 30.49.
    # This is the "gate's reach changed" case this table's own comment anticipates, not
    # a golden drifting: `RootZoneCapture` withdraws ~2.3 kg/day from a store that falls
    # from 195 kg to 45 kg over the season, so at its tightest the store still covers
    # ~30x one step's demand. It is the tightest row in the open field and still an
    # enormous margin — the ranking moved, the safety did not. Recorded rather than
    # quietly re-pointed, because a new stock displacing an established gate is exactly
    # the kind of change this census exists to surface.
    # ⚠ CHANGED 2026-08-12 (the soil-water re-basing): `soil_water` displaces
    # `subsoil_water` here and `carbon_pool` in `greenhouse`/`lighting`, at 9.3139 in
    # all three. This is the "gate's reach changed" case again, and this time the cause
    # is the STORE rather than a new flow: the root zone used to be declared at 1000 kg
    # over 1 m2 — 1000 mm of extractable water, a 7.7 m soil column — and is now
    # `rooted_depth x EXTR x rho x area`. A physically sized bucket has a physically
    # sized margin. The ranking moved; the safety is an order of magnitude, not a
    # threshold breach.
    # ⚠⚠ RE-MEASURED WHOLESALE 2026-08-14 (the step unfreeze, dt 1 -> 1/4). READ THE
    # UNITS BEFORE READING THE NUMBERS: a margin is ``stock / demand-per-CALL``, so it
    # is denominated in STEPS. Quartering the step quartered every per-call demand and
    # so multiplied every BIOSPHERE margin by ~4, while every row driven by a registry
    # this ceremony did not touch — power, thermal, ECLSS, crew — is unchanged to the
    # last bit. Nine of the nineteen rows below did not move at all, and that split is
    # the cleanest evidence available that the flip reached exactly what it aimed at.
    #
    # ⚠ TWO ROWS RE-RANKED, and in the same direction, for the same reason. Where a
    # scenario's tightest gate used to be a biosphere stock by a small margin, the
    # biosphere's number rose 4x past a neighbouring stock whose number did not:
    #   greenhouse      soil_water 9.3139  ->  carbon_pool 16.667 (the CABIN's scrub)
    #   sealed_station  carbon_pool 5.0232 ->  carbon_pool 16.667  (same stock, but now
    #                   the cabin's call rather than the plant's — see claim 3)
    # This is the "gate's reach changed" case this table's own comments were written to
    # surface, and it is the third time. It is a finding, not a drift: in both rows the
    # binding gate has passed OUT of the biosphere and into the ECLSS timestep check.
    "open_season": ("biosphere.soil_water", 37.2557585449319),
    # ⚠ 2026-08-12, twice (the stem-reserve build, then its cessation window). Before
    # the build: 2.3404741281202655 / 1.5124880369468734 / 2.112066494173573. The
    # IDENTITY of the tightest gate is unchanged in every row through both moves — only
    # the margin moved — so the ranking claim this table exists for is intact.
    # ⚠ 2026-08-14: 1.9016721361221138 / 1.552788483797351 / 2.1271916795585084. The
    # ratios are 4.04 / 3.59 / 4.36 — near 4x but not 4x, which is the signature of a
    # LIVE gate re-integrated rather than a rate-determined one rescaled.
    "sealed_chamber": ("biosphere.carbon_pool", 3.7793861201161016),
    "perennial_chamber": ("biosphere.carbon_pool", 3.5343019459308773),
    "consumer_chamber": ("biosphere.carbon_pool", 4.466989660058212),
    "power_bounded_soc": ("power.battery", 11.295323690100386),
    "power_self_discharge": ("power.battery", 11.085836827155921),
    "thermal_equilibrium": ("thermal.node", 257.68121326080376),
    "eclss_steady_state": ("eclss.cabin_o2", 33.333333333333364),
    "crew_mission": ("crew.food_store", 388.55555555555975),
    "station_heat_closure": ("power.battery", 11.295323690100386),
    "cabin_gas": ("eclss.cabin_o2", 35.57253249034074),
    # ⚠ RE-RANKED — see the header note. Its biosphere registry's own carbon-pool
    # minimum is 1150.7494, three orders slack; 16.667 is the cabin's 1/(k*dt).
    "greenhouse": ("biosphere.carbon_pool", 16.666666666666664),
    "water_recovery": ("eclss.cabin_o2", 35.57253249034074),
    # ⚠⚠ **THE BINDING STOCK CHANGED IDENTITY HERE (2026-08-14), the first time any
    # row in this table has.** `lighting` was gated by `soil_water` at 37.26; it is
    # now gated by `biosphere.carbon_pool` at 33.43. The lamp's photoperiod became a
    # within-day top-hat, so the lit chamber has real dark hours in which the crop
    # respires into its pool with nothing fixing carbon back — and the pool overtakes
    # the water as the tightest gate. ⚠ This is the census doing its job: a rank
    # change is the observation it exists to make, and it would have been invisible in
    # any test that pinned only the value.
    "lighting": ("biosphere.carbon_pool", 33.430399346096465),
    "harvest": ("biosphere.carbon_pool", 16.666666666666664),
    # ⚠ Both long-horizon rows are now BIT-IDENTICAL to their 5-year siblings, where
    # perennial's used to differ by 0.11 %. See
    # ``test_whether_the_perennial_gate_needs_the_LONG_horizon``.
    # ⚠ 2026-08-14 (the light path), with their 5-year siblings above.
    "perennial_long_horizon": ("biosphere.carbon_pool", 3.5343019459308773),
    "consumer_long_horizon": ("biosphere.carbon_pool", 4.466989660058212),
    "sealed_energy_drift": ("power.battery", 11.295323690100386),
    # ⚠ RE-RANKED WITHIN THE ROW — see the header note and claim 3. The stock is the
    # same; the CALL that binds is not. The biosphere registry's minimum rose
    # 5.0232 -> 19.0209 and so crossed above the cabin's unchanged 16.667.
    # ⚠ 16.6667 (the ECLSS scrubber constant) -> 11.8868 on 2026-08-14: the light
    # path made this a plant-driven number again. See the ranking test.
    "sealed_station": ("biosphere.carbon_pool", 11.88679216141662),
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

    Rank every **live** gate margin in the frozen roster. The smallest are the same
    stock, ``biosphere.carbon_pool``, in the scenarios that seal one.

    ⚠⚠ **"THE SIX SMALLEST" BECAME "THE FIVE SMALLEST" ON 2026-08-14, AND THIS IS THE
    STEP UNFREEZE'S HEADLINE FINDING ABOUT THE CENSUS.** The six were the five
    standalone chamber runs plus ``sealed_station``. At ``dt = 1`` the station's
    binding call was the *plant's* draw on the shared CO2 pool, at 5.0232 — tighter
    than the cabin's
    ``1/(k*dt) = 16.667`` on the same stock, which is why it belonged beside the
    chambers. Quartering the biosphere's step quartered that draw per call and so
    multiplied the margin by ~3.8, to **19.0209**. The cabin's step did not change.
    So the station's gate handed over: its census row is still ``carbon_pool``, but the
    number is now the ECLSS scrubber's timestep check, and the row has fallen from 6th
    to 12th, behind four ``power.battery`` entries.

    **Read plainly: the sealed station no longer has a plant-limited acceptance gate.**
    Nothing about the plant changed — the same trajectory, integrated more finely, is
    simply never within a quarter-day of out-running the pool. That is a fact about
    what ``rationed == 0`` can still refute on the station, and it makes the frozen
    contract's acceptance test weaker there than it was, not stronger.

    The first margin on any *other* stock is ``power.battery`` — outside the biosphere
    entirely, so the old corollary "even the runner-up is a chamber property" stays
    retired (it was retired once before, on 2026-08-10, and drifted back on 08-12).

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
    # ⚠⚠ RE-MEASURED 2026-08-10, and the QUALIFIER GOT MORE LOAD-BEARING, not less.
    # On 2026-08-09 the raw ranking led with FIVE ``carbon_pool`` entries before the
    # water-cycle pair's rate-determined 2.0 appeared. The humification split loosened
    # the carbon-pool margins (1.126/1.491/1.802 -> 1.500/1.512/2.112/2.340), so only
    # TWO now sort below 2.0 and the rate-determined ties come next. The live ranking
    # below is unchanged in shape — still six ``carbon_pool`` entries, still the same
    # six
    # scenarios — which is exactly the point this test's name carries: the ordering
    # claim is about LIVE gates, and the raw ranking is asserted alongside it so the
    # exclusion stays a measured fact rather than an invisible filter.
    #
    # ⚠⚠ RE-MEASURED AGAIN 2026-08-12 (stem reserves), and the count moved BACK: FOUR
    # ``carbon_pool`` entries now sort below the water pair's 2.0, where the
    # humification
    # split had cut it to two. ``sealed_chamber`` fell 2.340 -> 1.902 and
    # ``consumer_chamber``/``consumer_long_horizon`` sit at 2.125, so the tightest four
    # are 1.551 / 1.552 / 1.902 and the pair at 2.0 now lands FIFTH.
    #
    # ⚠ The count is asserted by MEASUREMENT rather than typed as a literal 4: the
    # number
    # has now moved twice in three days (5 -> 2 -> 4) and is plainly an artefact of
    # where
    # two unrelated quantities happen to cross, not a property worth freezing. What IS
    # asserted is the structural claim the qualifier exists for — everything below the
    # first rate-determined entry is ``carbon_pool``, and the first rate-determined
    # entry
    # is the water pair at exactly 2.0.
    #
    # ⚠ RE-MEASURED 2026-08-14 (the step unfreeze) and the count moved a third time,
    # 5 -> 2 -> 4 -> 3. Still not typed as a literal, for the reason above.
    first_rd = next(
        i for i, (_m, _s, _sid, g) in enumerate(raw) if g == "rate-determined"
    )
    assert [sid for _m, _s, sid, _g in raw[:first_rd]] == [
        "biosphere.carbon_pool"
    ] * first_rd, raw[:first_rd]
    assert first_rd >= 2, first_rd
    margin_rd, _s3, sid_rd, gate_rd = raw[first_rd]
    # the water-cycle pair ties exactly; which of the two sorts next is incidental
    assert sid_rd in {"biosphere.condensate", "biosphere.water_vapor"}, raw[first_rd]
    # ⚠ was the literal 2.0. It is `1/(k*dt)` with k = 0.5 /day — the SAME constant this
    # file's `test_a_first_order_stocks_margin_is_one_over_k_dt` inverts — so it is now
    # written that way rather than as the number it evaluates to at one step size. At
    # dt = 1 that read 2.0; at dt = 1/4 it reads 8.0, and nothing about the water moved.
    assert gate_rd == "rate-determined", raw[first_rd]
    assert margin_rd == 1.0 / (0.5 * BIO_DT), raw[first_rd]

    ranked = sorted(
        (margin, scenario, sid)
        for scenario in _runners()
        for sid, margin in _live(census(scenario)).items()
    )
    # ⚠⚠ SIX -> FIVE (2026-08-14). See the docstring: ``sealed_station`` left the head
    # of this ranking because its binding CALL changed registry, not because anything
    # about its plant changed. The five that remain are exactly the five STANDALONE
    # chamber runs — the ones whose only registry is the biosphere's.
    assert [sid for _m, _s, sid in ranked[:5]] == ["biosphere.carbon_pool"] * 5, ranked[
        :8
    ]
    assert {s for _m, s, _sid in ranked[:5]} == {
        "sealed_chamber",
        "perennial_chamber",
        "consumer_chamber",
        "perennial_long_horizon",
        "consumer_long_horizon",
    }
    # ...and ``sealed_station`` is asserted to be OUT of the head, by name, so that its
    # departure is a pinned fact rather than an absence nobody notices.
    #
    # ⚠⚠ **ITS BINDING CALL WENT BACK TO THE PLANT ON 2026-08-14 (the light path).** It
    # left this head when the binding call changed REGISTRY, and it is
    # ``biosphere.carbon_pool`` again at **11.8868**, for a reason on the plant's side:
    # the crop respires into the shared cabin pool through the lamp's dark hours, so the
    # pool's tightest moment tightens past the ECLSS scrubber's 1/(k·dt). ⚠ It sits just
    # BELOW the five standalone chambers, so the head of the ranking — the claim this
    # test is named for — is unchanged. What is withdrawn is the tie: it no longer
    # shares a value with ``greenhouse``/``harvest``, so that three-way assertion is
    # replaced by the two facts that survive (the pair still ties with each other; the
    # station does not join them).
    station_rank = next(
        i for i, (_m, s, _sid) in enumerate(ranked) if s == "sealed_station"
    )
    assert station_rank > 5, (station_rank, ranked[:13])
    assert ranked[station_rank][2] == "biosphere.carbon_pool"
    assert ranked[station_rank][0] == pytest.approx(11.88679216141662, rel=1e-12)
    scrubber = pytest.approx(1.0 / 0.06, rel=1e-12)
    assert {s for m, s, _sid in ranked if m == scrubber} == {"greenhouse", "harvest"}
    #
    # ⚠⚠ THE RUNNER-UP HAS NOW CHANGED IDENTITY THREE TIMES, AND IT HAS COME BACK TO A
    # VALUE IT ALREADY HELD ONCE. History, because the churn is the finding:
    #
    #   2026-08-09  ``sealed_chamber``'s ``o2_pool`` at 8.944  — licensed the corollary
    #               "even the runner-up is a chamber property".
    #   2026-08-10  the humification split moved O2 around, and the 7th row became
    #               ``power.battery`` at 11.086 — outside the biosphere entirely, so
    #               that corollary was RETIRED rather than restated.
    #   2026-08-12  the soil-water re-basing sized the root zone from geometry, and the
    #               7th row is ``biosphere.soil_water`` at 8.4379 — a chamber property
    #               again, on a stock that was never in contention before because it
    #               used to hold 1000 kg in a bucket that could physically hold 19.5.
    #   2026-08-14  the step unfreeze multiplied every biosphere margin by ~4 and left
    #               ``power.battery`` alone, so the row is ``power.battery`` at 11.086
    #               AGAIN — the same stock and the same value as on 08-10, reached by a
    #               completely different route.
    #
    # ⚠ The corollary is STILL NOT restored, and this is now the second time it would
    # have been wrong to restore it. It was retired on the evidence that the ranking
    # does not respect it; a claim that has been false once is not made true by drifting
    # back, and this row has now drifted back and forth twice in five days.
    #
    # ⚠ The ``> 7.0`` ratio bound stays DROPPED. Rank plus exact values say strictly
    # more than any threshold would.
    margin, _scenario, sid = ranked[5]
    assert sid == "power.battery", ("runner-up", ranked[5:9])
    # Four rows are ``power.battery``; three of them tie at 11.2953 and one sorts first
    # at 11.0858, so the SCENARIO of row 5 is not sort-incidental here and IS asserted.
    assert _scenario == "power_self_discharge", ("runner-up", ranked[5:9])
    assert {r[2] for r in ranked[5:9]} == {"power.battery"}, ranked[5:9]
    assert margin == pytest.approx(11.085836827155921, rel=1e-9), "runner-up value"
    assert margin > ranked[0][0], "…and the binding gate is still the tighter one"

    # --- claim 2: sealed_station's census row (folded in; see the note above) --------
    stock, expected = TIGHTEST[_TIER2]
    live = _live(census(_TIER2))
    got = min(live, key=lambda k: live[k])
    assert got == stock, ("tier2 row", got, live[got])
    assert live[got] == pytest.approx(expected, rel=1e-9), "tier2 row value"

    # --- claim 3: WHICH registry's call binds — and it changed hands ------------------
    # ⚠⚠ **INVERTED 2026-08-14, and this claim used to be named for its answer.** It
    # read "that row is the PLANT's draw, not the cabin's", and asserted
    # ``slow_min < fast_min`` with the message "the binding call must be the
    # biosphere's". That is now false, and the assertion that carried it would have
    # been the thing to delete if it had been written as a threshold. It was written as
    # a comparison of two pinned values, so it caught the handover instead.
    #
    #   2026-08-09/12  biosphere 5.218198 -> 5.023213, cabin 16.666667. Plant binds.
    #   2026-08-14     biosphere 19.020864, cabin 16.666667. CABIN binds.
    #
    # The biosphere's number rose by 3.786x — near the 4x a per-call denominator gives,
    # and short of it by the amount the trajectory genuinely re-integrated — and crossed
    # a cabin constant that did not move at all. Nothing here is a re-tuning: both
    # values are pinned exactly and the INEQUALITY between them is asserted in whichever
    # direction it currently points, with the direction named.
    cabin_min, bio_min = _registry_minima(
        _recorder_for(_TIER2), "biosphere.carbon_pool"
    )
    #   2026-08-14b    biosphere 11.886792, cabin 16.666667. PLANT binds AGAIN.
    #
    # ⚠ The handover reversed the same day, and by a bigger step than it took: the
    # light path cut the biosphere's number 19.0209 -> 11.8868 (-37 %) while the cabin
    # constant again did not move at all. The crop now respires into the shared pool
    # through the lamp's dark hours, so its own tightest call tightens well past the
    # scrubber's. ⚠ Two crossings in two changes says this pair sits close enough that
    # the DIRECTION is not a stable property — which is exactly why it is written as two
    # exact pins plus whichever inequality currently holds, and not as a claim about
    # which subsystem "the" binding call belongs to.
    assert bio_min == pytest.approx(11.88679216141662, rel=1e-9), "tier2 bio registry"
    assert cabin_min == pytest.approx(16.666666666666664, rel=1e-12), (
        "tier2 cabin registry"
    )
    assert bio_min < cabin_min, "tier2: the binding call is the PLANT's again"
    assert census(_TIER2)["biosphere.carbon_pool"].min_margin == bio_min

    # --- claim 4: the metric's consistency check against the gate it measures --------
    # Every frozen golden asserts `rationed == 0`, so no stock in any frozen scenario
    # may reach a margin of 1. A failure means either a golden started rationing or the
    # recorder is not measuring what the backstop reads.
    for name in _runners():
        for sid_, entry in census(name).items():
            if entry.min_margin is not None:
                assert entry.min_margin > 1.0, ("above firing point", name, sid_)


@pytest.mark.slow
def test_whether_the_perennial_gate_needs_the_LONG_horizon() -> None:
    """Does tripling the horizon find a tighter perennial gate? The answer has flipped
    twice, so this is named for the QUESTION.

    ⚠ **Renamed 2026-08-14.** It was
    ``test_tripling_the_horizon_now_TIGHTENS_the_perennial_gate`` — named for the
    answer, which this docstring had **already** flagged once as a fact that stopped
    being true, and which then stopped being true a second time, in the opposite
    direction, at ``dt = ¼``. A test whose *name* asserts a contingent measurement goes
    stale silently; the name now survives the next flip and the body records which way
    it currently points.

    **The history it was originally written for (2026-08-10) — kept, because it is still
    the reason this test exists.**

    As measured on 2026-08-09: the gate's minimum was reached inside the first 5 years
    and the 5-yr and 15-yr runs returned a **bit-identical** margin, so "the horizon
    lengthens the run, not the jar". ``test_chamber_scale.py`` had pinned the companion
    fact that the long-horizon goldens reuse the same scenario objects, making the
    *inventory* a t=0 property; the *margin* was measured, not inherited, and came back
    identical.

    It no longer does, for ``perennial``: 1.5124880369468734 at 5 years against
    **1.5004031863217981** at 15. The humus pool is still filling at year 5 (it reaches
    equilibrium around year 45), so the chamber's tightest moment now lies **outside**
    the 5-year window. This is the same single fact that restated the two
    decade-stability pins, the station biomass gate and the stem-only attractor: the
    settling transient outgrew the horizons the frozen contract was measured on.

    ``consumer`` is **unchanged and bit-identical**, which is what makes this a finding
    rather than a wobble — the herbivore chamber's tightest moment still falls inside
    five years. Both are asserted, so the asymmetry cannot be lost.
    """
    # ⚠⚠ **REVERTED 2026-08-14 by the step unfreeze, and the test is named for a fact
    # that is false again.** This test has now flipped twice, which is the whole reason
    # it is worth keeping. The history, in one line each:
    #   2026-08-09  a == b bit-identical — the tightest moment was inside 5 years.
    #   2026-08-10  b < a by 0.8 % — the humification split's settling transient pushed
    #               the tightest moment OUTSIDE the 5-year window.
    #   2026-08-12  b < a by 0.11 % — the gap narrowed (stem reserves); recorded rather
    #               than smoothed, precisely because a shrinking gap is how this pin
    #               would quietly stop being able to catch what it was written for.
    #   2026-08-14  a == b bit-identical AGAIN, at dt = ¼.
    #
    # The 0.11 % gap did not shrink further — it closed completely. Both margins moved
    # (1.5527884837973509 → 5.575540262132649 and 1.5506375020695391 → the same value),
    # so this is a genuine re-measurement, NOT the 4× rescaling a rate-determined stock
    # would show: 1.5528 × 4 = 6.211, and the measured value is 5.5755.
    #
    # ⚠ Read it as a finding, not a repair. The finer step does not remove the humus
    # settling transient — that is a decades-long process and the horizon still does not
    # contain it. What changed is that the chamber's tightest CARBON moment is once
    # again inside the first five years, so the 15-year run no longer finds anything
    # the 5-year run misses. The 2026-08-10 observation was real; it was contingent on a
    # step size, which is exactly the kind of dependency worth having on record.
    a = census("perennial_chamber")["biosphere.carbon_pool"].min_margin
    b = census("perennial_long_horizon")["biosphere.carbon_pool"].min_margin
    assert a is not None and b is not None
    assert a.hex() == b.hex(), (
        "the perennial gate is a 5-year property again — if this splits, the tightest "
        f"moment has moved back outside the window: {(a, b)}"
    )
    #   2026-08-14  a == b bit-identical AGAIN, at the light path; both 5.5755 ->
    #               3.5343, so the 5-year/15-year identity survives a change that
    #               moved the value by 37 %.
    assert a == pytest.approx(3.5343019459308773, rel=1e-9)

    # consumer has been a 5-year property throughout, on both steps. It is asserted
    # alongside perennial so the two cannot be conflated: for three of the four dates
    # above the pair DISAGREED, and that asymmetry was what made the finding legible.
    c = census("consumer_chamber")["biosphere.carbon_pool"].min_margin
    d = census("consumer_long_horizon")["biosphere.carbon_pool"].min_margin
    assert c is not None and d is not None
    assert c.hex() == d.hex(), ("consumer is still a 5-year property", c, d)


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
