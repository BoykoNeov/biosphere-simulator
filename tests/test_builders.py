"""Phase-3 P3.2 tests: the reusable compartment builders.

Each builder (``atmosphere``/``soil``/``plants``) is a pure function returning its own
stocks, flows, aux, and resolver ``shared``-map; ``build_season`` composes them. These
**structural** tests cover what the byte-identical goldens do not — and, for the domain
check, *localize* a failure the golden would also catch:

1. each builder emits only *its own* leaf's modeled stocks. The snapshot serializer
   **does** serialize ``domain`` (``sim_io/snapshot.py:74``), so a mis-stamped leaf
   already breaks the golden byte-compare (``domain`` is amount-invariant — no reduction
   keys on it, P3.1 — but still *emitted*). The check names *which* builder erred.
2. the builders **partition** the season. *Disjoint* ownership is **genuinely
   golden-blind** (``build_season`` unions via ``stocks[id] = s``, a dict that dedups,
   so two builders emitting one id pass silently); *complete* = the union plus the
   composition-level loss-sink == ``build_season``'s stocks / flows / aux.
3. **no builder imports another** — **genuinely golden-blind** (pure source structure):
   compartments meet only at shared stocks read from the catalog / threaded through
   ``ChamberWiring`` (the P3.3 rule).
"""

import ast
from pathlib import Path

import pytest

import domains.biosphere
from domains.biosphere.atmosphere import build_atmosphere
from domains.biosphere.compartments import ATMOSPHERE, PLANTS, SOIL, WATER
from domains.biosphere.plants import build_plants
from domains.biosphere.scenario import (
    DEFAULT_SCENARIO,
    SEALED_CHAMBER_SCENARIO,
    SeasonScenario,
)
from domains.biosphere.season import build_season
from domains.biosphere.soil import build_soil
from domains.biosphere.stocks import CompartmentBuild, chamber_wiring
from domains.biosphere.water import build_water
from simcore import boundary
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.ids import DomainId
from simcore.quantities import Quantity

_BUILDER_MODULES = ("atmosphere", "soil", "plants", "water")
_SCENARIOS = [
    pytest.param(DEFAULT_SCENARIO, id="open"),
    pytest.param(SEALED_CHAMBER_SCENARIO, id="sealed"),
]


def _builds(scenario: SeasonScenario) -> dict[str, tuple[CompartmentBuild, DomainId]]:
    """Each builder's ``CompartmentBuild`` paired with its expected leaf domain."""
    wiring = chamber_wiring(scenario.sealed)
    return {
        "atmosphere": (build_atmosphere(scenario, wiring), ATMOSPHERE),
        "soil": (build_soil(scenario, wiring), SOIL),
        "plants": (build_plants(scenario, wiring), PLANTS),
        "water": (build_water(scenario, wiring), WATER),
    }


# --- (1) each builder owns only its own leaf's modeled stocks ---------------


@pytest.mark.parametrize("scenario", _SCENARIOS)
def test_each_builder_emits_only_its_own_leaf_modeled_stocks(
    scenario: SeasonScenario,
) -> None:
    # Localizes a mis-stamped leaf to its builder. (The snapshot golden serializes
    # ``domain`` too, so it also catches this — here we name the culprit.) Every modeled
    # (non-boundary) stock must carry its builder's leaf domain; boundary stocks (the
    # sources/sinks the compartment's flows drive) stay in the boundary namespace.
    for name, (build, leaf) in _builds(scenario).items():
        for stock in build.stocks:
            if stock.domain == BOUNDARY_DOMAIN:
                continue
            assert stock.domain == leaf, (
                f"{name} builder emitted modeled stock {stock.id} with domain "
                f"{stock.domain}, expected its leaf {leaf}"
            )


def test_sealed_build_populates_all_four_leaves() -> None:
    # Non-vacuity: when sealed, every builder (incl. water, which owns ``condensate``
    # once the Step-3 cycle is closed) genuinely owns modeled stocks (so the per-leaf
    # check above is exercising real content, not passing on empty sets).
    for name, (build, leaf) in _builds(SEALED_CHAMBER_SCENARIO).items():
        modeled = [s for s in build.stocks if s.domain != BOUNDARY_DOMAIN]
        assert modeled, f"{name} should own modeled stocks in the sealed chamber"
        assert all(s.domain == leaf for s in modeled)


def test_open_field_water_builder_is_empty() -> None:
    # The water cycle is sealed-only: in the open field the water leaf holds no stocks /
    # flows (transpiration drains to a boundary, irrigation refills from one), so the
    # open golden is byte-identical and the leaf stays empty (P3.1's "water declared
    # empty").
    build, _ = _builds(DEFAULT_SCENARIO)["water"]
    assert build.stocks == () and build.flows == () and build.aux == ()


# --- (2) the builders partition build_season (disjoint + complete) ----------


@pytest.mark.parametrize("scenario", _SCENARIOS)
def test_builders_partition_build_season_stocks(scenario: SeasonScenario) -> None:
    builds = [b for b, _ in _builds(scenario).values()]
    # Disjoint ownership: no stock id is built by two compartments (equality alone would
    # hide a double-owned stock built identically by two builders — advisor's catch).
    seen: set = set()
    for build in builds:
        ids = {s.id for s in build.stocks}
        assert seen.isdisjoint(ids), f"stock(s) owned by >1 builder: {seen & ids}"
        seen |= ids
    # Complete: the builder union + the composition-level carbon loss-sink == the
    # assembled season's stocks exactly (the same completeness the golden's byte-compare
    # verifies, here as a set identity over ids).
    state, _ = build_season(scenario)
    loss = set(boundary.loss_sinks({Quantity.CARBON}))
    assert seen.isdisjoint(loss)  # the loss-sink is composition-level, not a builder's
    assert seen | loss == set(state.stocks)


@pytest.mark.parametrize("scenario", _SCENARIOS)
def test_builders_partition_build_season_flows_and_aux(
    scenario: SeasonScenario,
) -> None:
    builds = [b for b, _ in _builds(scenario).values()]
    _, registry = build_season(scenario)
    # Flows: disjoint per builder, union == the registry's flow ids.
    seen_flows: set = set()
    for build in builds:
        fids = {f.id for f in build.flows}
        assert seen_flows.isdisjoint(fids), (
            f"flow(s) owned by >1 builder: {seen_flows & fids}"
        )
        seen_flows |= fids
    assert seen_flows == {f.id for f in registry.flows}
    # Aux: union == the registry's aux ids (the thermal-time accumulator, plants-owned).
    seen_aux = {a.id for build in builds for a in build.aux}
    assert seen_aux == {a.id for a in registry.aux_processes}


@pytest.mark.parametrize("scenario", _SCENARIOS)
def test_shared_map_merges_to_build_season_resolver(
    scenario: SeasonScenario,
) -> None:
    # The resolver shared-map (#16) is a builder output merged at composition: soil owns
    # soil_water always; atmosphere owns co2_pool when sealed. Disjoint keys, and the
    # merged map is exactly what the sealed/open chamber needs.
    builds = [b for b, _ in _builds(scenario).values()]
    merged: dict = {}
    for build in builds:
        assert merged.keys().isdisjoint(build.shared), "shared-map key owned twice"
        merged.update(build.shared)
    expected_keys = {"soil_water", "co2_pool"} if scenario.sealed else {"soil_water"}
    assert set(merged) == expected_keys


# --- (3) no builder imports another (the P3.3 shared-stock rule) -------------


def _imported_biosphere_submodules(source: str) -> set[str]:
    """The ``domains.biosphere.<x>`` submodules a source imports (``<x>`` names)."""
    found: set[str] = set()
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.ImportFrom) and node.module and not node.level:
            parts = node.module.split(".")
            if parts[:2] == ["domains", "biosphere"] and len(parts) >= 3:
                found.add(parts[2])
        elif isinstance(node, ast.Import):
            for alias in node.names:
                parts = alias.name.split(".")
                if parts[:2] == ["domains", "biosphere"] and len(parts) >= 3:
                    found.add(parts[2])
    return found


def test_no_builder_imports_another() -> None:
    bdir = Path(domains.biosphere.__file__).parent
    for mod in _BUILDER_MODULES:
        imported = _imported_biosphere_submodules(
            (bdir / f"{mod}.py").read_text(encoding="utf-8")
        )
        siblings = set(_BUILDER_MODULES) - {mod}
        leaked = imported & siblings
        assert not leaked, (
            f"{mod}.py imports sibling builder(s) {sorted(leaked)} — compartments must "
            "meet only at shared stocks (catalog / ChamberWiring), P3.3"
        )


def test_import_guard_is_not_vacuous() -> None:
    # Control: season.py *does* import every builder, so the detector above is able to
    # catch a real sibling import (a guard that can never fire is worthless).
    bdir = Path(domains.biosphere.__file__).parent
    imported = _imported_biosphere_submodules(
        (bdir / "season.py").read_text(encoding="utf-8")
    )
    assert set(_BUILDER_MODULES) <= imported
