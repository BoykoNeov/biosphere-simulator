"""Phase-9 Step-6 gate: file composition — reusable domain/species **bundles**.

A scenario ``includes`` one or more bundle files (``authoring.schema.BundleSpec``); the
interpreter merges each bundle's parameters/stocks/flows/forcings into the scenario's
flat graph (``authoring.compose.apply_includes``). A *species* is a flow-set + param-set
+ stock-template (the crew bundle, parametrized by ``crew_count``); a *domain* is a
stock+flow bundle over a quantity set (the battery/ENERGY bundle). This is the
"authored, not programmed" payoff: a station is *composed* from reusable files.

Proven two ways, mirroring the phase discipline:

1. **Single-bundle byte-identity (the faithfulness anchor).** A scenario that is *only*
   an include of the crew species bundle reproduces the frozen ``crew_state.json``
   byte-for-byte: the include contributes the whole graph. (The bundle is a template at
   ``crew_count = 1.0`` ⇒ ``1.0*base == base`` exactly, so byte-identity holds.)

2. **Two-domain merge (the "it bit" — the new capability over Step 3).** A station that
   includes BOTH the crew species and a disjoint battery domain merges >1 file into one
   graph. The domains share no stock/forcing/quantity, so every quantity conserves every
   step — the run completing IS the merge proof. Faithfulness is by **projection**: the
   crew half matches the frozen crew golden and the battery half matches a frozen
   ``SelfDischarge`` run (both independently anchored); a composed 10-stock state cannot
   be byte-identical to the 8-stock crew golden, so projection is the only gate.

Plus the **merge semantics** (all cross-port — Step 6b's Rust port + Step 7's freeze
inherit them): a duplicate stock id / flow id / forcing key / parameter name across any
two sources is an ``AuthoringError`` (no silent override); packs inside a bundle are
deferred; run config and nested includes in a bundle are schema-rejected; ``overrides``
reach a bundle-declared parameter; ``has_authored_kinetics`` ORs across bundles.
"""

from pathlib import Path

import pytest
from pydantic import ValidationError

import sim_io
from authoring.compose import apply_includes
from authoring.errors import AuthoringError
from authoring.interpreter import load_scenario
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from domains.crew.loader import load_crew_params
from domains.crew.scenario import MISSION_SCENARIO
from domains.crew.system import build_crew
from domains.power.flows import SelfDischarge
from domains.power.loader import load_self_discharge_params
from simcore.environment import SourceResolver
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
CREW_STATION = SCENARIO_DIR / "crew_station.yaml"
STATION_COMPOSED = SCENARIO_DIR / "station_composed.yaml"
MIXED_INLINE_BATTERY = SCENARIO_DIR / "crew_station_inline_battery.yaml"
GOLDEN_PATH = Path(__file__).parent / "regression" / "golden" / "crew_state.json"

_ENERGY = Quantity.ENERGY


# --- 1. Single-bundle include: the byte-identity faithfulness anchor ------------------


def test_single_bundle_include_reproduces_crew_golden_bytes() -> None:
    # A scenario that is only an include of the crew species bundle builds the eight
    # frozen crew ids and reproduces the frozen golden byte-for-byte. If `includes` were
    # a no-op the graph would be empty, not the golden — so this also proves the
    # include contributes the entire graph.
    built = load_scenario(str(CREW_STATION))
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    assert len(states[-1].stocks) == 8
    assert built.has_authored_kinetics is False
    produced = sim_io.dumps(states[-1]).encode("utf-8")
    assert produced == GOLDEN_PATH.read_bytes()


def test_single_bundle_include_equals_build_crew_graph() -> None:
    # Structural equality (the failure-localizing gate): the merged graph is identical
    # to `build_crew(...)`'s — same State, same canonical flow tuple incl. params.
    built = load_scenario(str(CREW_STATION))
    expected_state, expected_registry = build_crew(load_crew_params(), MISSION_SCENARIO)
    assert built.state == expected_state
    assert built.registry.flows == expected_registry.flows


def test_reused_bundle_scales_with_override() -> None:
    # The same species bundle is genuinely reusable/parametrizable: `crew_count` is
    # declared in the INCLUDED bundle, and an `overrides` reaches it through the merge.
    # At 4.0 every store is 4x and the run is no longer byte-identical to the golden (a
    # non-scaling knob could not move it — the Step-1/Step-3 "it bit" discipline).
    built = load_scenario(str(CREW_STATION), overrides={"crew_count": 4.0})
    assert built.state.stocks[StockId("crew.food_store")].amount == 4000.0
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    golden = sim_io.loads(GOLDEN_PATH.read_text())
    ratio = (
        states[-1].stocks[StockId("crew.food_store")].amount
        / golden.stocks[StockId("crew.food_store")].amount
    )
    assert ratio == pytest.approx(4.0, rel=1e-12)
    assert sim_io.dumps(states[-1]).encode("utf-8") != GOLDEN_PATH.read_bytes()


# --- 2. Two-domain merge: the composition "it bit" ------------------------------------


def _frozen_self_discharge_final() -> State:
    """Run the frozen ``SelfDischarge`` alone over the composed horizon (the oracle)."""
    bat = Stock(
        id=StockId("power.battery"),
        domain=DomainId("power"),
        quantity=_ENERGY,
        unit=canonical_unit(_ENERGY),
        amount=1.0e7,
        kind=StockKind.POOL,
    )
    waste = Stock(
        id=StockId("boundary.waste_heat"),
        domain=DomainId("boundary"),
        quantity=_ENERGY,
        unit=canonical_unit(_ENERGY),
        amount=0.0,
        kind=StockKind.BOUNDARY,
    )
    stocks = {bat.id: bat, waste.id: waste}
    flow = SelfDischarge(
        FlowId("power.self_discharge"),
        0,
        battery=StockId("power.battery"),
        waste_heat=StockId("boundary.waste_heat"),
        params=load_self_discharge_params(),
    )
    integrator = EulerIntegrator(Registry([flow], stocks))
    state = State(n=0, stocks=stocks, rng_seed=0)
    for _ in range(168):
        report = integrator.step_report(state, SourceResolver(forcings={}), 3600.0)
        state = report.state
    return state


def test_two_domain_merge_runs_and_conserves() -> None:
    # >1 file merged into one graph (crew + battery). Disjoint domains ⇒ the run
    # completing (every-step conservation gate inside step_report) IS the merge proof.
    # The composed run inherits the battery's "authored != validated" marker.
    built = load_scenario(str(STATION_COMPOSED))
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    assert len(states[-1].stocks) == 10
    assert built.has_authored_kinetics is True


def test_two_domain_merge_crew_half_matches_golden() -> None:
    # Projection faithfulness: the crew half of the composed final state equals the
    # frozen crew golden (the battery domain is disjoint — it perturbs crew stocks not
    # at all).
    built = load_scenario(str(STATION_COMPOSED))
    final = run_scenario(built)[0][-1]
    golden = sim_io.loads(GOLDEN_PATH.read_text())
    for sid, stock in golden.stocks.items():
        assert final.stocks[sid].amount == stock.amount


def test_two_domain_merge_battery_half_matches_self_discharge() -> None:
    # Projection faithfulness: the battery half equals a frozen SelfDischarge run over
    # the same horizon (the Step-2 authored-kinetics oracle), bit-for-bit.
    built = load_scenario(str(STATION_COMPOSED))
    final = run_scenario(built)[0][-1]
    frozen = _frozen_self_discharge_final()
    for sid in (StockId("power.battery"), StockId("boundary.waste_heat")):
        assert final.stocks[sid].amount == frozen.stocks[sid].amount


# --- Merge order: includes-first-then-inline (positively pinned, both ports) ----------


def test_apply_includes_orders_includes_before_inline() -> None:
    # Carry-forward (ii): 6a documented + collision-tested the includes-first ordering
    # but never *positively* order-tested it (no byte-identity depends on it — the
    # serialized outputs are id-sorted). Pin it on the list `apply_includes` returns,
    # *before* the interpreter canonicalizes it to id-sorted maps. The mixed anchor has
    # BOTH a crew-bundle include and an inline battery domain, so a port that merged
    # inline-first would fail here. (The Rust `scenario_files.rs` has the mirror.)
    spec = ScenarioSpec.model_validate(load_yaml(str(MIXED_INLINE_BATTERY)))
    merged = apply_includes(spec, MIXED_INLINE_BATTERY.parent)
    assert [s.id for s in merged.stocks] == [
        "crew.food_store",
        "crew.water_store",
        "crew.o2_store",
        "boundary.exhaled_co2",
        "boundary.fecal_waste",
        "boundary.crew_humidity",
        "boundary.urine",
        "boundary.crew_o2_consumed",
        "power.battery",
        "boundary.waste_heat",
    ]
    assert [f.id for f in merged.flows] == [
        "crew.oxygen_consumption",
        "crew.food_metabolism",
        "crew.water_balance",
        "power.self_discharge",
    ]
    assert merged.includes == []  # emptied after the merge


def test_mixed_include_and_inline_equals_two_bundle_composition() -> None:
    # The mixed anchor (crew bundle + inline battery) builds the same graph as
    # station_composed (crew bundle + battery bundle); its final state matches, and the
    # inline SelfDischarge marks it authored.
    mixed = run_scenario(load_scenario(str(MIXED_INLINE_BATTERY)))[0][-1]
    composed = run_scenario(load_scenario(str(STATION_COMPOSED)))[0][-1]
    assert mixed.stocks == composed.stocks


# --- Merge semantics: no silent override, deferrals, schema fences --------------------


def _write(tmp: Path, name: str, text: str) -> Path:
    path = tmp / name
    path.write_text(text, encoding="utf-8")
    return path


_MINI_BUNDLE = """\
stocks:
  - id: a.pool
    domain: a
    quantity: energy
    kind: pool
    amount: 1.0e+3
  - id: boundary.a_sink
    domain: boundary
    quantity: energy
    kind: boundary
    amount: 0.0
flows:
  - id: a.leak
    kinetics:
      rate: 'stock("a.pool")'
      stoichiometry:
        a.pool: -1
        boundary.a_sink: 1
"""

_SCENARIO_HEAD = """\
name: s
integrator: euler
dt: 1.0
steps: 1
"""


def test_including_same_bundle_twice_is_duplicate(tmp_path: Path) -> None:
    # Two instances of the same bundle collide on every id — the concrete reason
    # multi-instance composition needs id-namespacing/prefixing (deferred to Step 6c).
    _write(tmp_path, "b.yaml", _MINI_BUNDLE)
    scenario = _write(
        tmp_path, "s.yaml", _SCENARIO_HEAD + "includes:\n  - b.yaml\n  - b.yaml\n"
    )
    with pytest.raises(AuthoringError, match="duplicate stock id"):
        load_scenario(str(scenario))


def test_duplicate_stock_across_include_and_inline_raises(tmp_path: Path) -> None:
    _write(tmp_path, "b.yaml", _MINI_BUNDLE)
    scenario = _write(
        tmp_path,
        "s.yaml",
        _SCENARIO_HEAD
        + "includes:\n  - b.yaml\n"
        + "stocks:\n  - id: a.pool\n    domain: a\n    quantity: energy\n"
        + "    kind: pool\n    amount: 5.0\n",
    )
    with pytest.raises(AuthoringError, match="duplicate stock id 'a.pool'"):
        load_scenario(str(scenario))


def test_duplicate_parameter_across_bundles_raises(tmp_path: Path) -> None:
    _write(tmp_path, "b1.yaml", "parameters:\n  k: 1.0\n")
    _write(tmp_path, "b2.yaml", "parameters:\n  k: 2.0\n")
    scenario = _write(
        tmp_path, "s.yaml", _SCENARIO_HEAD + "includes:\n  - b1.yaml\n  - b2.yaml\n"
    )
    with pytest.raises(AuthoringError, match="duplicate parameter 'k'"):
        load_scenario(str(scenario))


def test_duplicate_forcing_across_sources_raises(tmp_path: Path) -> None:
    _write(tmp_path, "b.yaml", "forcings:\n  f:\n    const: 1.0\n")
    scenario = _write(
        tmp_path,
        "s.yaml",
        _SCENARIO_HEAD + "includes:\n  - b.yaml\nforcings:\n  f:\n    const: 2.0\n",
    )
    with pytest.raises(AuthoringError, match="duplicate forcing 'f'"):
        load_scenario(str(scenario))


def test_bundle_flow_param_pack_is_deferred(tmp_path: Path) -> None:
    # A parameter pack inside an included bundle is a clean AuthoringError (it would
    # resolve against the bundle's dir — deferred, matching Step 1 / the Rust port).
    _write(
        tmp_path,
        "b.yaml",
        "flows:\n  - id: crew.food_metabolism\n    type: crew.food_metabolism\n"
        "    wiring:\n      food_store: x\n      exhaled_co2: y\n      fecal_waste: z\n"
        "    params:\n      pack: some_pack.yaml\n",
    )
    scenario = _write(tmp_path, "s.yaml", _SCENARIO_HEAD + "includes:\n  - b.yaml\n")
    with pytest.raises(
        AuthoringError, match="parameter packs inside an included bundle"
    ):
        load_scenario(str(scenario))


def test_run_config_in_bundle_is_schema_rejected(tmp_path: Path) -> None:
    # A bundle carries no run config — `steps:` is an extra key (extra="forbid").
    _write(tmp_path, "b.yaml", _MINI_BUNDLE + "steps: 5\n")
    scenario = _write(tmp_path, "s.yaml", _SCENARIO_HEAD + "includes:\n  - b.yaml\n")
    with pytest.raises(ValidationError):
        load_scenario(str(scenario))


def test_nested_include_in_bundle_is_schema_rejected(tmp_path: Path) -> None:
    # Includes are flat, one level deep — a bundle with its own `includes` is rejected.
    _write(tmp_path, "inner.yaml", _MINI_BUNDLE)
    _write(tmp_path, "b.yaml", _MINI_BUNDLE + "includes:\n  - inner.yaml\n")
    scenario = _write(tmp_path, "s.yaml", _SCENARIO_HEAD + "includes:\n  - b.yaml\n")
    with pytest.raises(ValidationError):
        load_scenario(str(scenario))


def test_missing_include_raises(tmp_path: Path) -> None:
    scenario = _write(tmp_path, "s.yaml", _SCENARIO_HEAD + "includes:\n  - nope.yaml\n")
    with pytest.raises(AuthoringError, match="could not be read"):
        load_scenario(str(scenario))
