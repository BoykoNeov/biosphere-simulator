"""Phase-9 Step-0 gate: the declarative interpreter is faithful (the crew anchor).

The composition-subset interpreter (``src/authoring``) is proven against the frozen
standalone Crew system three ways, strongest first:

1. **Structural equality** (the primary, failure-localizing gate): the graph the
   interpreter builds from ``crew_mission.yaml`` is *identical* to the one
   ``build_crew(...)`` builds — same ``State`` (ids, quantities, units, amounts,
   kinds, compositions) and the same canonical-ordered flow tuple (including each
   flow's ``params: CrewParams``, which is equatable). This proves "same graph incl.
   params" directly, without going through the golden.
2. **Loaded values equal the frozen scenario** — the initial conditions and forcing
   constants parse as *floats* equal to ``MISSION_SCENARIO`` (the pyyaml YAML-1.1
   numeric-string guard: a silent string-parse would fail here).
3. **Byte-identity end-to-end** (the corollary): run the interpreted graph 7 days
   Euler and reproduce ``crew_state.json`` byte-for-byte — no new golden; the frozen
   Crew golden is reused as the end-to-end oracle.

Plus the **decision-B safety property**: a well-formed but mis-wired scenario
(``crew_broken_wiring.yaml``) interprets cleanly and then raises ``ConservationError``
on the first step — bad wiring surfaces at runtime, never silently fixed. The
authoring layer buys *conservation safety*, not scientific validity.

What byte-identity genuinely tests is wiring + ICs + forcings; the param contribution
is byte-identical *trivially* because Step 0 reuses the frozen ``load_crew_params``
loader by name (inline/override param packs are Step 1).
"""

from pathlib import Path

import pytest

import sim_io
from authoring.interpreter import interpret, load_scenario
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from domains.crew.loader import load_crew_params
from domains.crew.scenario import MISSION_DAYS, MISSION_SCENARIO
from domains.crew.system import build_crew, crew_resolver, run_crew
from simcore.flow import ConservationError
from simcore.integrator import EulerIntegrator

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
CREW_YAML = SCENARIO_DIR / "crew_mission.yaml"
BROKEN_YAML = SCENARIO_DIR / "crew_broken_wiring.yaml"

GOLDEN_PATH = Path(__file__).parent / "regression" / "golden" / "crew_state.json"


def _built():
    return load_scenario(str(CREW_YAML))


def test_interpreted_graph_equals_build_crew() -> None:
    # The primary gate: same State and same canonical flow tuple (incl. params) as
    # the frozen imperative build. Localizes any faithfulness failure to the graph.
    built = _built()
    expected_state, expected_registry = build_crew(load_crew_params(), MISSION_SCENARIO)
    assert built.state == expected_state
    assert built.registry.flows == expected_registry.flows


def test_run_config_matches_mission_scenario() -> None:
    built = _built()
    assert built.integrator == "euler"
    assert built.dt == MISSION_SCENARIO.dt_seconds
    assert built.steps == MISSION_DAYS * MISSION_SCENARIO.steps_per_day


def test_loaded_values_equal_frozen_scenario() -> None:
    # The pyyaml numeric-string guard: ICs and forcing constants must parse as
    # floats equal to the frozen scenario (a dotless-exponent string-parse fails).
    spec = ScenarioSpec.model_validate(load_yaml(str(CREW_YAML)))
    ics = {s.id: s.amount for s in spec.stocks}
    assert ics["crew.food_store"] == MISSION_SCENARIO.food_store0
    assert ics["crew.water_store"] == MISSION_SCENARIO.water_store0
    assert ics["crew.o2_store"] == MISSION_SCENARIO.o2_store0
    for amount in ics.values():
        assert isinstance(amount, float)
    forcings = {name: f.const for name, f in spec.forcings.items()}
    assert forcings["crew_o2_intake"] == MISSION_SCENARIO.o2_intake_rate
    assert forcings["crew_food_intake"] == MISSION_SCENARIO.food_intake_rate
    assert forcings["crew_water_intake"] == MISSION_SCENARIO.water_intake_rate
    for value in forcings.values():
        assert isinstance(value, float)


def test_interpreted_run_matches_crew_golden_bytes() -> None:
    # The end-to-end corollary: the interpreted run reproduces the frozen Crew
    # golden byte-for-byte (no new golden — the frozen one is the oracle).
    built = _built()
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    produced = sim_io.dumps(states[-1]).encode("utf-8")
    assert produced == GOLDEN_PATH.read_bytes()


def test_interpreted_run_matches_frozen_run_trajectory() -> None:
    # Belt-and-suspenders: the interpreted final State equals the frozen run's final
    # State object (not just its serialized bytes) — the two drivers agree exactly.
    built = _built()
    interp_states, _, _ = run_scenario(built)
    state, registry = build_crew(load_crew_params(), MISSION_SCENARIO)
    frozen_states, _, _ = run_crew(
        EulerIntegrator(registry),
        state,
        crew_resolver(MISSION_SCENARIO),
        MISSION_SCENARIO.dt_seconds,
        MISSION_DAYS * MISSION_SCENARIO.steps_per_day,
    )
    assert interp_states[-1] == frozen_states[-1]


def test_mis_wired_scenario_raises_conservation_error() -> None:
    # Decision-B safety: a well-formed scenario that wires a CARBON flow's
    # withdrawal at an OXYGEN stock interprets cleanly, then fails the per-quantity
    # conservation gate on the first step — surfaced, never silently fixed.
    built = load_scenario(str(BROKEN_YAML))
    with pytest.raises(ConservationError):
        run_scenario(built)


def test_interpret_is_deterministic_object_reuse() -> None:
    # interpret() on a re-parsed spec yields an equal graph (no hidden per-call
    # state) — the determinism the whole authoring path relies on.
    spec = ScenarioSpec.model_validate(load_yaml(str(CREW_YAML)))
    assert interpret(spec).state == interpret(spec).state
