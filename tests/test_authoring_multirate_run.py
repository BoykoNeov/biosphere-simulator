"""Multi-rate authoring, Step 3: the run harness — the knob finally drives.

Step 1 measured the identity and the payoff; Step 2 built the partition from the
file. Both stopped short of :func:`authoring.run.run_scenario`, which was single-rate
— Step 1's helper called ``multirate_step`` **by hand**, so nothing an author can
actually invoke had ever run multi-rate. This file pins the layer that closes that:
``run_scenario`` routes a multi-rate scenario through the driver, and every other
scenario down today's code path **verbatim**.

**The load-bearing test is**
:func:`test_a_single_rate_scenario_never_touches_the_driver`.
The phase's whole golden-preservation argument is that no golden moves, and the
mechanism is the branch, *not* the ``n_sub=1`` identity. The identity is measured — but
"measured" and "load-bearing for all 25 goldens" are different risk postures. That test
asserts the branch does not leak, which is the failure the goldens themselves would
report only as a diff with no cause attached.

**What Step 3 does NOT add:** no schema field, no integrator name, no flow type. The
authoring freeze manifest must be **untouched** by this step — if
``test_authoring_freeze_manifest.py`` goes red here, something moved that should not
have.
The build-time ``k·(dt/n_sub) < 1`` precondition is Step 5; until it lands, an unsafe
effective sub-step is caught at *run* time by ``RationedError`` (and, for the
demand-controlled flow, **not caught at all** — see
``tests/test_authoring_export_fidelity.py``).
"""

import copy
from collections.abc import Mapping
from dataclasses import replace
from pathlib import Path
from typing import Any

import pytest

from authoring.errors import AuthoringError, RationedError
from authoring.interpreter import BuiltScenario, interpret
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.auxiliary import AuxId
from simcore.environment import Environment
from simcore.ids import StockId
from simcore.registry import Registry
from simcore.state import State

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"

O2_EQ = 8.0  # o2_setpoint − Con_o2/k_makeup = 10 − 0.004/0.002 — the truth
UNSAFE_DT = 3600.0  # k_makeup·dt = 7.2, k_scrub·dt = 3.6 — single-rate wrecks the cabin
HOURS = 24
CABIN_O2 = StockId("eclss.cabin_o2")


def _build(
    n_sub: int = 1,
    dt: float = UNSAFE_DT,
    steps: int = HOURS,
    *,
    allow_unsafe_step: bool = False,
) -> BuiltScenario:
    """The ECLSS anchor re-interpreted at an overridden run config.

    The anchor **file** is never edited (the ``test_authoring_dt_hazard._build_at``
    discipline): only its in-memory dict, so the graph under test stays the committed
    one and only its cadence moves.

    Unlike ``test_authoring_multirate_identity._at_dt``, the cadence here is declared on
    the **spec**, so the Step-5 precondition is judging the configuration that actually
    runs — and ``allow_unsafe_step`` therefore defaults to **False**. The rows that pass
    ``True`` are the ones deliberately running a step the platform now refuses, in order
    to show what it is refusing.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw["dt"] = dt
    raw["steps"] = steps
    raw["n_sub"] = n_sub
    return interpret(
        ScenarioSpec.model_validate(raw),
        base_dir=SCENARIO_DIR,
        allow_unsafe_step=allow_unsafe_step,
    )


# ---------------------------------------------------------------------------
# 1. THE BRANCH — the golden-preservation guarantee, at the code-path level
# ---------------------------------------------------------------------------


def test_a_single_rate_scenario_never_touches_the_driver(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    # THE TEST THIS FILE EXISTS FOR. Every committed scenario predates the multi-rate
    # keys, so every one of them must keep taking today's `step_report` loop over the
    # whole `registry` — not `multirate_step` at n_sub=1.
    #
    # The n_sub=1 identity is measured (Step 1) and would make the routed path produce
    # the same bytes. That is exactly why this test is needed rather than redundant: if
    # the branch leaked, the goldens would stay GREEN and the leak would go unnoticed
    # until some future simcore change to the driver quietly moved 25 files at once.
    # Keeping `BuiltScenario.registry` alongside the partition only buys the safety if
    # something asserts the harness actually uses it.
    def _boom(*_args: object, **_kwargs: object) -> object:
        raise AssertionError(
            "run_scenario routed a single-rate scenario through multirate_step; the "
            "is_multirate branch leaked and all 25 goldens now rest on the n_sub=1 "
            "identity rather than on taking the untouched code path"
        )

    monkeypatch.setattr("authoring.run.multirate_step", _boom)
    built = _build(n_sub=1, dt=60.0)  # the anchor's own frozen sizing — a safe run
    assert built.is_multirate is False
    states, rationed, _events = run_scenario(built)
    assert rationed == 0
    assert len(states) == built.steps + 1


# ---------------------------------------------------------------------------
# 2. THE PAYOFF — through the harness an author actually calls
# ---------------------------------------------------------------------------


def test_the_harness_rescues_the_cabin_at_the_unsafe_master_dt() -> None:
    # The phase's headline, now reachable from `run_scenario`: master dt=3600 (what
    # NEIGHBOURS see) with n_sub=60 lands on the truth, with the backstop silent. Step 1
    # measured this by calling `multirate_step` by hand; an author cannot do that.
    built = _build(n_sub=60, dt=UNSAFE_DT)
    assert built.is_multirate is True
    states, rationed, _events = run_scenario(built)
    assert rationed == 0
    assert states[-1].stocks[CABIN_O2].amount == pytest.approx(O2_EQ, abs=1e-6)
    # The master cadence is untouched by sub-stepping: one commit per master step (N2),
    # 60 sub-steps notwithstanding. This is what makes the export hourly.
    assert len(states) == HOURS + 1
    assert states[-1].n == HOURS


def test_the_same_scenario_single_rate_raises() -> None:
    # The contrast that gives the test above its meaning: the identical file at the
    # identical dt, minus the n_sub knob, is refused. Multi-rate is what moved it from
    # "refused" to "correct" — not some other edit in this step.
    #
    # Step 5 moved WHERE it is refused: the build now rejects it (k_scrub*3600 = 3.6),
    # so the author never reaches the run. Both stages are asserted — the run-time gate
    # is still the one that catches state-dependent over-draws no build check can
    # decide.
    with pytest.raises(AuthoringError, match="co2_scrub_rate"):
        _build(n_sub=1, dt=UNSAFE_DT)
    with pytest.raises(RationedError):
        run_scenario(_build(n_sub=1, dt=UNSAFE_DT, allow_unsafe_step=True))


# ---------------------------------------------------------------------------
# 3. MULTI-RATE IS NOT A SAFETY KNOB — and Step 5 is what closes that
# ---------------------------------------------------------------------------


def test_the_build_refuses_an_unsafe_effective_substep() -> None:
    # THE ADVISOR'S POINT, CLOSED. `n_sub=2` at dt=3600 is the case that made
    # "multi-rate is the performance enabler, NOT the hazard closer" true: the knob
    # READS as safety,
    # the effective sub-step is 1800 s, and the cabin lands on 36.0 against a truth of
    # 8.0. Step 5 is the direct closer, and this is it — refused at BUILD, from params +
    # dt + n_sub alone, before a step runs.
    #
    # The check is on the EFFECTIVE sub-step dt/n_sub, never the master dt. That
    # distinction is the whole reason this file's headline case (n_sub=60, same dt)
    # builds happily two tests above while this one does not: same dt, same graph, same
    # params — different effective step.
    with pytest.raises(AuthoringError) as excinfo:
        _build(n_sub=2, dt=UNSAFE_DT)
    msg = str(excinfo.value)
    assert "dt/n_sub=1800.0" in msg, f"the EFFECTIVE step must be named, got: {msg}"
    assert "n_sub" in msg  # the fast-flow remedy: raise n_sub, cadence untouched


def test_an_unsafe_effective_substep_still_rations_through_the_harness() -> None:
    # The run-time half of the same case, reached now only via the study hatch. Kept
    # rather than deleted: it is the evidence that the build refusal above is refusing
    # something REAL, not being conservative for its own sake.
    #
    # That this raises at all is luck of shape, not the run-time gate working: the
    # backstop sees the DONOR-controlled scrubber over-draw. `o2_makeup` is
    # demand-controlled and its near-setpoint oscillation is invisible to rationing at
    # any dt (test_authoring_export_fidelity.py) — which is exactly why the build check
    # had to exist, and why it is not a mere convenience over this one.
    with pytest.raises(RationedError):
        run_scenario(_build(n_sub=2, dt=UNSAFE_DT, allow_unsafe_step=True))


def test_the_escape_hatch_works_on_the_multirate_path_too() -> None:
    # `allow_rationing=True` is for STUDYING a rationed run, and multi-rate must not
    # quietly become the one path where the hatch is unavailable — the n_sub=2 run above
    # is exactly the kind of run worth inspecting rather than only failing.
    #
    # BOTH hatches are needed now, and they are not redundant: allow_unsafe_step opens
    # the BUILD, allow_rationing opens the RUN. Two independent gates at two stages, so
    # studying a hazard that trips both means saying so twice. That verbosity is the
    # feature — neither hatch silently implies the other.
    states, rationed, _events = run_scenario(
        _build(n_sub=2, dt=UNSAFE_DT, allow_unsafe_step=True), allow_rationing=True
    )
    assert rationed > 0  # summed across sub-operations by multirate_step's own contract
    assert states[-1].stocks[CABIN_O2].amount != pytest.approx(O2_EQ, abs=1e-6)


# ---------------------------------------------------------------------------
# 4. THE MESSAGE — what the author reads at the moment they hit the hazard
# ---------------------------------------------------------------------------


def test_the_rationed_message_offers_n_sub_only_where_n_sub_exists() -> None:
    # The message is part of the fix. On the multi-rate path the honest advice is
    # "increase n_sub or reduce dt"; on the single-rate path there IS no n_sub to raise,
    # and naming a knob the scenario does not have would send the author looking for a
    # key that (until they add it) does nothing. Conditional, therefore — not one string
    # with everything in it.
    with pytest.raises(RationedError) as multi:
        run_scenario(_build(n_sub=2, dt=UNSAFE_DT, allow_unsafe_step=True))
    assert "n_sub" in str(multi.value)

    with pytest.raises(RationedError) as single:
        run_scenario(_build(n_sub=1, dt=UNSAFE_DT, allow_unsafe_step=True))
    assert "n_sub" not in str(single.value)


# ---------------------------------------------------------------------------
# 5. THE AUX TRIPWIRE — a guard for a shape no authored file can express yet
# ---------------------------------------------------------------------------


class _CountingAux:
    """A minimal ``AuxProcess`` — the shape ``interpret`` cannot yet produce."""

    @property
    def id(self) -> AuxId:
        return AuxId("test.counter")

    def evaluate(
        self, snapshot: State, env: Environment, dt: float
    ) -> Mapping[str, float]:
        return {"ticks": dt}


def test_multirate_refuses_a_graph_with_aux_processes() -> None:
    # THE TRIPWIRE. `step_report` advances aux; `multirate_step` deliberately never does
    # (simcore P2, "Aux x multi-rate is out of scope"). So routing an aux-bearing graph
    # through the driver would SILENTLY FREEZE the accumulators: no error, conservation
    # green (aux is non-conserved by definition, so the gate cannot see it), just a
    # number that stops moving.
    #
    # This cannot fire from an authored file today — `interpret` calls
    # Registry(flows, stocks) and never wires aux_processes, which is precisely the
    # unstated precondition Step 1's identity rests on
    # (test_the_identity_precondition_no_authored_graph_has_aux). It is built by hand
    # here because the day the biosphere (the one aux-bearing domain) becomes
    # authorable, this guard is the difference between an error and silence.
    built = _build(n_sub=60, dt=UNSAFE_DT)
    with_aux = replace(
        built,
        registry=Registry(
            list(built.registry.flows), built.state.stocks, [_CountingAux()]
        ),
    )
    with pytest.raises(AuthoringError, match="aux"):
        run_scenario(with_aux)


def test_the_aux_tripwire_does_not_fire_on_the_single_rate_path() -> None:
    # The guard is scoped to the multi-rate branch, and that scoping is deliberate
    # rather than incidental: single-rate `step_report` advances aux CORRECTLY, so
    # refusing aux there would ban a shape the harness handles fine. The guard names a
    # driver limitation, not a policy about aux.
    built = _build(n_sub=1, dt=60.0)
    with_aux = replace(
        built,
        registry=Registry(
            list(built.registry.flows), built.state.stocks, [_CountingAux()]
        ),
        state=replace(built.state, aux={"ticks": 0.0}),
    )
    states, rationed, _events = run_scenario(with_aux)
    assert rationed == 0
    assert states[-1].aux["ticks"] == pytest.approx(60.0 * built.steps)
