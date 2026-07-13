"""Declarative scenario authoring (Phase 9, the boundary interpreter).

This package is the **authoring layer**: it reads a declarative scenario file
(YAML), validates a schema, and builds a runnable ``(State, Registry, resolver)``
graph **by calling the existing frozen constructors** — it does **no float math**,
so the pure engine (``simcore``) and the frozen domains are untouched (Phase-9
decision A). Authored artifacts are runtime objects, never frozen, never reference
(decision B): this package imports the frozen domains, never the reverse.

Step 0 covers the **composition subset** (reading "A"): a scenario may select
existing frozen ``Flow`` types (:data:`authoring.flow_registry.FLOW_TYPES` — the
author-selectable surface Step 7 freezes), parametrize them via the frozen domain
loaders (:data:`authoring.flow_registry.PARAM_LOADERS`), set initial conditions,
and wire which stock each leg points at. The bounded kinetics DSL (decision D) is
Step 2; parameter packs are Step 1; the Rust interpreter + parse-parity is Step 4.

The interpreter builds only **single-rate, no-reset** graphs in Step 0 (the
``run_scenario`` harness mirrors the standalone ``run_crew``/``run_power`` drivers,
minus any phenology/reset hook — the two-rate driver is a later step, as on the
Rust side). Pure boundary code: it may depend on pydantic/yaml (via ``config``),
which ``simcore`` may never do.
"""

from authoring.errors import AuthoringError
from authoring.interpreter import BuiltScenario, interpret, load_scenario
from authoring.run import run_scenario
from authoring.schema import (
    FlowSpec,
    ForcingSpec,
    ParamPackRef,
    ScenarioSpec,
    StockSpec,
)

__all__ = [
    "AuthoringError",
    "BuiltScenario",
    "FlowSpec",
    "ForcingSpec",
    "ParamPackRef",
    "ScenarioSpec",
    "StockSpec",
    "interpret",
    "load_scenario",
    "run_scenario",
]
