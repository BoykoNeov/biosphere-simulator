"""Tier-1: the nine registered Power / Thermal / ECLSS flow types, authored.

The post-roadmap Tier-1 unfreeze (``docs/plans/post-roadmap-flow-registry-growth.md``)
grew ``authoring.flow_registry`` from the three standalone Crew flows to twelve, making
the frozen sibling science **author-selectable**: a scenario file can now *compose* it
instead of inventing it. This file is that surface's gate, strongest claim first:

1. **Registry faithfulness** — every ``FlowTypeSpec`` mirrors its frozen class's real
   dataclass fields. See the test's own note: nothing else in the tree owned this.
2. **Structural equality + byte-identity (ECLSS)** — the authored ``eclss_cabin.yaml``
   builds a graph *identical* to ``build_eclss(...)`` and reproduces the frozen
   ``eclss_state.json`` byte-for-byte. The ``crew_mission.yaml`` precedent, now for a
   three-quantity domain with a six-leg forced flow.
3. **Power / Thermal anchors** — conservation, ``rationed == 0``, determinism, and each
   landing where its own design arithmetic predicts.
4. **Teeth** — the wiring-field check rejects a misnamed field at build time.

**REGISTERED IS NOT CALIBRATED — this file must not be misread as claiming otherwise.**
Reproducing the frozen ECLSS golden proves the registry *lowers correctly* — that the
authored graph IS the frozen graph. It says nothing about whether the frozen graph
describes a real cabin. It does not: every param these nine flows read is a TODO(cite)
placeholder except crew's two (eclss.yaml says so in as many words — "NOT a NASA BVAD
number"). None of these scenarios carries an ``UNCALIBRATED`` banner, because
``has_authored_kinetics`` tracks *who wrote the rate law*, not whether the science is
validated — the two axes are independent, and always have been. See
``docs/authoring-reference.md``, "Frozen is not calibrated".

The ``dt`` hazard that registration created has its own gate:
``tests/test_authoring_dt_hazard.py``.
"""

from __future__ import annotations

from dataclasses import fields
from pathlib import Path
from typing import Any, cast

import pytest

import sim_io
from authoring.errors import AuthoringError
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS, load_param_set
from authoring.interpreter import interpret, load_scenario
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.ids import StockId

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"

ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"
POWER_YAML = SCENARIO_DIR / "power_bus.yaml"
THERMAL_YAML = SCENARIO_DIR / "thermal_node.yaml"

# StockId is a NewType over str, so a bare literal will not type-check as a key
# (the test_authored_habitat.py precedent).
CABIN_O2 = StockId("eclss.cabin_o2")
CABIN_CO2 = StockId("eclss.cabin_co2")
BATTERY = StockId("power.battery")
WASTE_HEAT = StockId("boundary.waste_heat")
NODE = StockId("thermal.node")

# The nine types Tier 1 added, by domain — the surface under test.
TIER1_TYPES = (
    "power.solar_charge",
    "power.load_draw",
    "power.self_discharge",
    "thermal.heat_input",
    "thermal.radiator_reject",
    "eclss.crew_metabolism",
    "eclss.co2_scrubber",
    "eclss.condenser",
    "eclss.o2_makeup",
)


# --- 1. registry faithfulness ------------------------------------------------


@pytest.mark.parametrize("type_name", sorted(FLOW_TYPES))
def test_spec_mirrors_the_frozen_constructor_exactly(type_name: str) -> None:
    """Each ``FlowTypeSpec`` matches its frozen class's real dataclass fields.

    **This gate did not exist before Tier 1, and the registry's own docstring is why it
    should.** That docstring argues the duplication is "a stable, deliberately-curated
    public surface ... not incidental drift" — but nothing checked that the duplicated
    copy was *accurate*. The manifest gate owns completeness (added-but-frozen-nowhere);
    the anchors own does-it-run; neither notices a ``wiring_fields`` entry that simply
    does not name a real constructor field. Such a typo would surface only as a
    ``TypeError`` from ``type_spec.cls(...)``, and only if some anchor happened to
    exercise that type — silent for any type without one.

    Every frozen flow shares the ``(id, priority, *wiring[, params])`` shape, so the
    declared spec fully determines the expected field tuple. Deriving it from the live
    class is what makes this catch a rename on the frozen side too.
    """
    spec = FLOW_TYPES[type_name]
    actual = tuple(f.name for f in fields(cast("Any", spec.cls)))
    expected = (
        ("id", "priority")
        + spec.wiring_fields
        + (("params",) if spec.param_set is not None else ())
    )
    assert actual == expected, (
        f"{type_name} declares {expected} but {spec.cls.__name__} has {actual}"
    )


def test_every_named_param_set_resolves_to_a_flat_float_dataclass() -> None:
    # Each loader must produce a flat frozen dataclass of floats: the frozen-`type` path
    # passes the object straight to a constructor, and the authored-`kinetics` path
    # flattens it via asdict() for `param("…")`. A loader returning anything else breaks
    # the second path with a confusing AuthoringError at build time.
    for name in sorted(PARAM_LOADERS):
        obj = cast("Any", load_param_set(name, None))
        assert fields(obj), f"{name} loaded a param object with no fields"
        for f in fields(obj):
            assert isinstance(getattr(obj, f.name), float), f"{name}.{f.name}"


def test_the_kinetics_anchor_reads_every_key_of_the_three_new_sets() -> None:
    """`param_sets_dsl.yaml` reads all nine key names of `charge`/`thermal`/`eclss`.

    Registering a loader opens **two** surfaces: a frozen type's `param_set`, and an
    authored rate's ``param("…")``. Only the second has a cross-port hazard — Python
    derives the key names via ``asdict()``, Rust hardcodes them in
    `kinetics_param_map` — so the anchor makes both ports resolve the same names from
    one file.

    This test guards the *anchor*, not the ports: it asserts the file still reads every
    key, so that adding a param to a frozen set (which the Rust map must mirror by hand)
    cannot leave a key silently un-anchored while the crossport test still passes.
    """
    text = (SCENARIO_DIR / "param_sets_dsl.yaml").read_text(encoding="utf-8")
    for set_name in ("charge", "thermal", "eclss"):
        obj = cast("Any", load_param_set(set_name, None))
        for f in fields(obj):
            assert f'param("{f.name}")' in text, (
                f"{set_name}.{f.name} is reachable from a kinetics rate but "
                f"param_sets_dsl.yaml never reads it — the Rust kinetics_param_map "
                f"key for it is unanchored"
            )


def test_every_flow_types_param_set_is_a_known_loader() -> None:
    # A `param_set` naming a loader that does not exist is a registry typo that would
    # surface as a KeyError deep inside load_param_set, at run time, for whoever first
    # authored that type.
    for name, spec in FLOW_TYPES.items():
        if spec.param_set is not None:
            assert spec.param_set in PARAM_LOADERS, name


def test_tier1_registered_the_nine_expected_types() -> None:
    # The unfreeze's scope, stated once: 3 crew (Step 0) + 9 siblings = 12. If this
    # changes, the authoring manifest moved and the unfreeze discipline applies.
    assert set(TIER1_TYPES) <= set(FLOW_TYPES)
    assert len(FLOW_TYPES) == 12
    assert set(PARAM_LOADERS) == {
        "crew",
        "self_discharge",
        "charge",
        "thermal",
        "eclss",
    }


def test_the_biosphere_is_deliberately_not_registered() -> None:
    # Not an oversight — a structural exclusion (Allocation takes a composite
    # CarbonContext that a flat wiring_fields tuple + single param_set cannot express,
    # plus aux / the shared co2_pool / the two-rate driver). Pinning it keeps a future
    # reader from "fixing" the gap without reading why it is there.
    assert not any(name.startswith("biosphere.") for name in FLOW_TYPES)


# --- 2. the ECLSS anchor: structural equality + byte-identity ----------------


def test_authored_eclss_graph_equals_build_eclss() -> None:
    """The primary ECLSS gate: same ``State`` and same canonical flow tuple (incl. the
    ``EclssParams`` objects, which are equatable) as the frozen imperative build.

    Stronger than the byte-identity below and it localizes failure: this compares the
    *graph*, so a wiring/composition/param error shows up here rather than as a diff in
    900-step-old float bytes.
    """
    from domains.eclss.loader import load_eclss_params
    from domains.eclss.scenario import STEADY_STATE_SCENARIO
    from domains.eclss.system import build_eclss

    built = load_scenario(str(ECLSS_YAML))
    expected_state, expected_registry = build_eclss(
        load_eclss_params(), STEADY_STATE_SCENARIO
    )
    assert built.state == expected_state
    assert built.registry.flows == expected_registry.flows


def test_authored_eclss_run_matches_the_frozen_golden_bytes() -> None:
    # The end-to-end corollary: 900 steps Euler reproduces the frozen ECLSS golden
    # byte-for-byte. No new golden — the frozen one is reused as the oracle, exactly as
    # crew_mission.yaml reuses crew_state.json. This is what "the registry lowers frozen
    # science faithfully" means, end to end.
    built = load_scenario(str(ECLSS_YAML))
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    produced = sim_io.dumps(states[-1]).encode("utf-8")
    assert produced == (GOLDEN_DIR / "eclss_state.json").read_bytes()


def test_authored_eclss_carries_no_authored_kinetics_marker() -> None:
    # Every flow is a frozen `type`, so the marker is False — and this is the fact most
    # open to misreading. False means "no authored kinetics". It does NOT mean the
    # validated: the eclss.yaml params it just used are all TODO(cite) placeholders. The
    # marker measures authorship of the rate LAW, never the calibration of its VALUES.
    assert load_scenario(str(ECLSS_YAML)).has_authored_kinetics is False


def test_authored_eclss_cabin_pools_are_single_quantity() -> None:
    # The composition trap, pinned. Standalone ECLSS does not tie the crew's atoms
    # together (atomic coupling is a Phase-6 seam), so cabin_co2 is CARBON with the 1:1
    # default — NOT {carbon: 1, oxygen: 2}. Annotating composition would look like a
    # refinement and would in fact break OXYGEN balance on crew_metabolism. It would NOT
    # be caught at build: the interpreter's stoichiometry check runs for authored
    # `kinetics` flows only, never for frozen `type` flows — it would surface as a
    # runtime ConservationError. Contrast scenarios/algae_habitat.yaml, whose authored
    # respiration REQUIRES the fold.
    from simcore.quantities import Quantity

    stocks = load_scenario(str(ECLSS_YAML)).state.stocks
    assert stocks[CABIN_CO2].composition == {Quantity.CARBON: 1.0}
    assert stocks[CABIN_O2].composition == {Quantity.OXYGEN: 1.0}


# --- 3. the Power anchor -----------------------------------------------------


def test_authored_power_bus_conserves_energy_and_never_rations() -> None:
    # All three Power flows on one bus. ENERGY closes over the augmented system
    # (battery + the unclamped solar source + the waste-heat sink) to roundoff, and
    # the well-fed sizing keeps the forced LoadDraw from over-drawing.
    built = load_scenario(str(POWER_YAML))
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    total0 = sum(s.amount for s in states[0].stocks.values())
    total1 = sum(s.amount for s in states[-1].stocks.values())
    assert total1 == pytest.approx(total0, rel=1e-12)


def test_authored_power_battery_decays_at_exactly_the_self_discharge_law() -> None:
    """The SOC drifts by the donor-controlled leak and nothing else — the point.

    Solar and load balance *through* the charge efficiency (eta_c * 1000 W = 950 W =
    load), so the two FORCED flows cancel on the battery and the only surviving term is
    ``power.self_discharge``'s ``-k*battery``. The trajectory must be the exact
    geometric contraction ``b_n = b_0 * (1 - k*dt)^n`` — the frozen SelfDischarge idiom,
    now reached from a scenario file. Predicted from the frozen loader, not hardcoded.
    """
    from domains.power.loader import load_self_discharge_params

    built = load_scenario(str(POWER_YAML))
    states, _, _ = run_scenario(built)
    k = load_self_discharge_params().self_discharge_rate
    b0 = states[0].stocks[BATTERY].amount
    predicted = b0 * (1.0 - k * built.dt) ** built.steps
    assert states[-1].stocks[BATTERY].amount == pytest.approx(predicted, rel=1e-9)
    # ...and it really does decay (a scenario where nothing moved would pass the
    # conservation test above and prove nothing).
    assert states[-1].stocks[BATTERY].amount < b0


def test_authored_power_waste_heat_is_monotonic() -> None:
    # Every degraded joule lands in waste_heat and none ever leaves — monotonic by
    # construction, which is what makes it a free "heat generated" diagnostic.
    states, _, _ = run_scenario(load_scenario(str(POWER_YAML)))
    heat = [s.stocks[WASTE_HEAT].amount for s in states]
    assert all(b >= a for a, b in zip(heat[:-1], heat[1:], strict=True))
    assert heat[-1] > heat[0]


# --- 4. the Thermal anchor (the one Tier-2 file) -----------------------------


def test_authored_thermal_node_conserves_energy_and_never_rations() -> None:
    # rationed == 0 here is a SIZING claim, not a structural one: the T^4 law has no
    # k*dt < 1 guarantee, and only the frozen heat_capacity (tau ~ 65 steps at dt=3600)
    # keeps Euler from overshooting.
    built = load_scenario(str(THERMAL_YAML))
    states, rationed, events = run_scenario(built)
    assert rationed == 0
    assert events == ()
    total0 = sum(s.amount for s in states[0].stocks.values())
    total1 = sum(s.amount for s in states[-1].stocks.values())
    assert total1 == pytest.approx(total0, rel=1e-12)


def test_authored_thermal_node_warms_toward_the_emergent_equilibrium() -> None:
    """The node climbs monotonically toward ``T_eq`` and never overshoots it.

    ``T_eq`` is *emergent* — it is not in the scenario or the params; it is where
    ``eps*sigma*A*(T^4 - T_space^4)`` balances the forced load. The run starts far below
    it (T ~ 102.7 K) deliberately: a node that "equilibrates" because it began at
    equilibrium would satisfy conservation and prove nothing. Overshoot would mean the
    tau >> dt sizing had failed — the one thing keeping the nonlinear radiator stable
    under Euler.
    """
    from domains.thermal.flows import temperature
    from domains.thermal.loader import load_thermal_params
    from domains.thermal.system import equilibrium_temperature

    params = load_thermal_params()
    built = load_scenario(str(THERMAL_YAML))
    states, _, _ = run_scenario(built)
    temps = [
        temperature(
            s.stocks[NODE].amount,
            heat_capacity=params.heat_capacity,
            space_temperature=params.space_temperature,
        )
        for s in states
    ]
    t_eq = equilibrium_temperature(params)

    assert temps[0] == pytest.approx(102.7, abs=0.1)  # starts far below
    # Monotone warming, never overshooting — the tau >> dt sizing holding, step by step.
    assert all(b >= a for a, b in zip(temps[:-1], temps[1:], strict=True))
    assert max(temps) <= t_eq  # never overshoots — the sizing holds
    assert temps[-1] > 0.95 * t_eq  # and gets most of the way there (~5.2 tau)


def test_authored_thermal_run_is_deterministic() -> None:
    # Bit-identity within a build, for the one anchor carrying a transcendental. This is
    # a WITHIN-port claim (determinism) and is unrelated to the CROSS-port tiering that
    # keeps this file out of the bit-exact Rust run comparison — a Tier-2 flow is still
    # perfectly deterministic on one machine.
    a, _, _ = run_scenario(load_scenario(str(THERMAL_YAML)))
    b, _, _ = run_scenario(load_scenario(str(THERMAL_YAML)))
    for sa, sb in zip(a, b, strict=True):
        for sid, stock in sa.stocks.items():
            assert stock.amount == sb.stocks[sid].amount


# --- 5. teeth ----------------------------------------------------------------


def test_a_misnamed_wiring_field_is_rejected_at_build_time() -> None:
    # The registry's wiring names ARE the authoring contract, so the interpreter demands
    # an exact set match. A plausible near-miss (`waste` for `waste_heat`) must fail at
    # build with a message naming the real fields — not construct a broken flow, and not
    # wait for a runtime TypeError.
    raw = load_yaml(str(POWER_YAML))
    for flow in raw["flows"]:
        if flow["id"] == "power.load_draw":
            flow["wiring"]["waste"] = flow["wiring"].pop("waste_heat")
    with pytest.raises(AuthoringError, match="do not match this flow type's fields"):
        interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)


def test_a_param_free_flow_given_params_is_rejected() -> None:
    # The converse teeth: thermal.heat_input takes no params. Naming a set for it is an
    # author error worth catching at build — silently ignoring it would let a modder
    # believe they had configured something.
    raw = load_yaml(str(THERMAL_YAML))
    for flow in raw["flows"]:
        if flow["id"] == "thermal.heat_input":
            flow["params"] = "thermal"
    with pytest.raises(AuthoringError, match="takes no params"):
        interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)


def test_a_frozen_flow_given_the_wrong_param_set_is_rejected() -> None:
    # A flow type's param set is a fixed fact of the class, not an author choice: the
    # author picks which VALUES (the default file or a pack), never which SCHEMA. Naming
    # `charge` for a scrubber must fail rather than load an object the constructor
    # use.
    raw = load_yaml(str(ECLSS_YAML))
    for flow in raw["flows"]:
        if flow["id"] == "eclss.co2_scrubber":
            flow["params"] = "charge"
    with pytest.raises(AuthoringError, match="does not match this flow type's set"):
        interpret(ScenarioSpec.model_validate(raw), base_dir=SCENARIO_DIR)
