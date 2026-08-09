"""Enumerate the tree's ``science_gate`` markers — the derivation behind the manifests'
``science_bands`` / ``liveness_floors`` fields.

Why this exists
---------------
Both freeze manifests name only properties of the **run** (golden bytes,
``rationed == 0``, no extinction, conservation, determinism). Every assertion about the
*science* — that the canopy is a real wheat canopy, that the closed chamber's CO₂
attractor has not collapsed — lived in test files reachable from no manifest, so none
could fail an unfreeze ceremony. See ``docs/plans/post-roadmap-acceptance-gate.md``
(finding 5, which measured the hole) and ``post-roadmap-acceptance-gate-standing.md``
(the adjudication, the inclusion rule, and the survey).

Why ``ast`` and not pytest
--------------------------
The gate must enumerate the same way ``_frozen_param_files()`` enumerates files on disk:
**independently of how the suite was invoked**. Two alternatives were considered and
rejected:

* a ``pytest_collection_modifyitems`` registry — collection is *partial* on a one-file
  run, so ``uv run pytest tests/test_freeze_manifest.py`` would see an almost-empty set
  and the completeness check would go red for no reason;
* a subprocess ``--collect-only`` — a second collection of a suite whose runtime was
  just cut 3.3x (``docs/test-suite-runtime.md``), paid on every run.

Static parsing has one hole — a marker applied dynamically (``pytestmark``, a fixture, a
parametrized indirection) is invisible here. That hole is closed by **convention rather
than by mechanism**, and the convention is pinned as its own assertion in
``test_freeze_manifest.py``: ``science_gate`` may only appear as a literal decorator
with literal keyword arguments. A stated convention with a test is honest; a silent hole
is not.
"""

from __future__ import annotations

import ast
from dataclasses import dataclass
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent

MARKER = "science_gate"

#: The two fields a gate may belong to. They are kept apart deliberately: a band's bound
#: comes from OUTSIDE this repo, a floor's was tuned to our own calibration. Merging
#: two claims of different strength under one name is this project's recorded failure
#: mode, so the field is part of the marker rather than inferred from the value.
FIELDS = ("science_bands", "liveness_floors")

#: Required keyword arguments. ``quantity``/``bound``/``source`` make the manifest entry
#: readable without opening the test — the manifest names the CLAIM, not just a test id.
REQUIRED_KEYS = ("scenario", "field", "quantity", "bound", "source")


@dataclass(frozen=True, order=True)
class ScienceGate:
    """One committed assertion that gates a frozen scenario's science."""

    scenario: str
    field: str
    quantity: str
    bound: str
    source: str
    locus: str  # "tests/<file>.py::<test name>"

    def entry(self) -> dict[str, str]:
        """The manifest form — the claim, minus the scenario key it is filed under."""
        return {
            "quantity": self.quantity,
            "bound": self.bound,
            "source": self.source,
            "locus": self.locus,
        }


def _marker_calls(node: ast.FunctionDef) -> list[ast.Call]:
    """The ``@pytest.mark.science_gate(...)`` decorator calls on one test function."""
    calls: list[ast.Call] = []
    for dec in node.decorator_list:
        if not isinstance(dec, ast.Call):
            continue
        func = dec.func
        # match the dotted tail so both ``pytest.mark.science_gate``
        # and a ``from pytest import mark`` spelling are recognized.
        if (
            isinstance(func, ast.Attribute)
            and func.attr == MARKER
            and isinstance(func.value, ast.Attribute)
            and func.value.attr == "mark"
        ):
            calls.append(dec)
    return calls


def _literal_kwargs(call: ast.Call, locus: str) -> dict[str, str]:
    """Read the marker's keyword arguments, refusing anything non-literal.

    Refusing a computed value is the point, not pedantry: a bound assembled at import
    time would make the manifest's recorded claim depend on evaluation the manifest gate
    never performs, so the frozen text could drift from the executed assertion silently.
    """
    if call.args:
        raise ValueError(f"{locus}: {MARKER} takes keyword arguments only")
    out: dict[str, str] = {}
    for kw in call.keywords:
        if kw.arg is None:
            raise ValueError(f"{locus}: {MARKER} does not accept **kwargs")
        if not isinstance(kw.value, ast.Constant) or not isinstance(
            kw.value.value, str
        ):
            raise ValueError(f"{locus}: {MARKER}'s {kw.arg} must be a string literal")
        out[kw.arg] = kw.value.value
    missing = [k for k in REQUIRED_KEYS if k not in out]
    if missing:
        raise ValueError(f"{locus}: {MARKER} is missing {missing}")
    extra = [k for k in out if k not in REQUIRED_KEYS]
    if extra:
        raise ValueError(f"{locus}: {MARKER} got unexpected {extra}")
    if out["field"] not in FIELDS:
        raise ValueError(
            f"{locus}: field must be one of {FIELDS}, got {out['field']!r}"
        )
    return out


def collect_science_gates() -> tuple[ScienceGate, ...]:
    """Every ``science_gate`` marker in ``tests/``, sorted — the live derivation.

    A gate is discovered by *being written*, so one no manifest names turns the
    completeness gate red. That is the same lever ``_flow_set()`` uses: derive from the
    tree, never hand-list.
    """
    gates: list[ScienceGate] = []
    for path in sorted(TESTS_DIR.glob("test_*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if not isinstance(node, ast.FunctionDef):
                continue
            for call in _marker_calls(node):
                locus = f"tests/{path.name}::{node.name}"
                kwargs = _literal_kwargs(call, locus)
                gates.append(
                    ScienceGate(
                        scenario=kwargs["scenario"],
                        field=kwargs["field"],
                        quantity=kwargs["quantity"],
                        bound=kwargs["bound"],
                        source=kwargs["source"],
                        locus=locus,
                    )
                )
    return tuple(sorted(gates))


def gates_for(scenarios: frozenset[str], field: str) -> dict[str, list[dict[str, str]]]:
    """The manifest fragment for one field, restricted to one manifest's roster.

    Every scenario in the roster gets a key — an **explicitly empty list** where no gate
    exists. An absent key and a deliberately-empty one are different claims, and a gate
    that reads ``manifest[field].get(name, [])`` cannot tell them apart.
    """
    out: dict[str, list[dict[str, str]]] = {name: [] for name in sorted(scenarios)}
    for gate in collect_science_gates():
        if gate.field == field and gate.scenario in scenarios:
            out[gate.scenario].append(gate.entry())
    return out


def non_decorator_marker_sites() -> tuple[str, ...]:
    """Every ``mark.science_gate`` reference that is NOT a decorator on a test function.

    This is what makes static enumeration sound. A text count cannot do the job — the
    test that counts occurrences contains occurrences, and its own docstring is a false
    positive (measured: 13 textual vs 10 real). Matching ``@pytest.mark.science_gate``
    would fix that but miss the case actually worth catching, a ``pytestmark = [...]``
    assignment, which has no ``@`` and which ``ast`` cannot associate with a test.

    So the check is structural: collect the attribute accesses, subtract those sitting
    in a decorator position, and report the remainder. Prose is invisible to it because
    prose is not an attribute access.
    """
    stray: list[str] = []
    for path in sorted(TESTS_DIR.glob("test_*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        decorator_nodes = {
            id(call.func)
            for node in ast.walk(tree)
            if isinstance(node, ast.FunctionDef)
            for call in _marker_calls(node)
        }
        for node in ast.walk(tree):
            if (
                isinstance(node, ast.Attribute)
                and node.attr == MARKER
                and isinstance(node.value, ast.Attribute)
                and node.value.attr == "mark"
                and id(node) not in decorator_nodes
            ):
                stray.append(f"tests/{path.name}:{node.lineno}")
    return tuple(stray)


def unknown_scenarios(known: frozenset[str]) -> tuple[ScienceGate, ...]:
    """Gates naming a scenario in neither manifest — a typo, or a gate on a non-frozen
    scenario claiming standing it cannot have. Either way the manifests would silently
    drop it, so it is surfaced rather than filtered."""
    return tuple(g for g in collect_science_gates() if g.scenario not in known)
