"""Step-11 gate: ``simcore`` is stdlib-pure (ZERO third-party dependencies).

Invariant #11 — the pure core imports only the Python standard library, so the
eventual Rust port stays a near-mechanical translation. This is enforced here as
an automated guard rather than by packaging.

The check is **static** (AST-based), not a runtime ``sys.modules`` inspection:
walking the parse tree catches imports nested in functions, ``TYPE_CHECKING``
blocks, or lazy branches that a plain ``import simcore`` would never execute. The
allow-set is deliberately tight — **stdlib OR the ``simcore`` package itself**,
nothing else — so the same scan also catches a core→outer-layer leak (``simcore``
importing ``sim_io`` / ``config`` / ``domains``), which is the other half of
"core is pure".

Known limitation: a *dynamic* import (``importlib.import_module("numpy")``) is not
statically visible. ``simcore`` does not do this; the guard does not engineer
around it.
"""

import ast
import sys
from pathlib import Path

import pytest

import simcore

_CORE_PACKAGE = "simcore"
_CORE_DIR = Path(simcore.__file__).parent
_CORE_FILES = sorted(_CORE_DIR.rglob("*.py"))


def _is_pure_top_level(name: str) -> bool:
    """A top-level module name that the core is allowed to import."""
    return name in sys.stdlib_module_names or name == _CORE_PACKAGE


def _third_party_imports(source: str) -> set[str]:
    """Top-level module names imported by ``source`` that are neither stdlib nor
    part of the ``simcore`` package.

    Relative imports (``from . import x``) are intra-package and skipped. The
    whole AST is walked, so imports inside functions / ``TYPE_CHECKING`` guards
    are covered too. Returns the offending *top-level* names (empty set ⇒ pure).
    """
    offenders: set[str] = set()
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.Import):
            for alias in node.names:
                top = alias.name.split(".", 1)[0]
                if not _is_pure_top_level(top):
                    offenders.add(top)
        elif isinstance(node, ast.ImportFrom):
            if node.level:  # relative import → intra-package, allowed
                continue
            if node.module is None:
                continue
            top = node.module.split(".", 1)[0]
            if not _is_pure_top_level(top):
                offenders.add(top)
    return offenders


# --- the scan: every simcore source file is stdlib-pure ----------------------
@pytest.mark.parametrize("module_path", _CORE_FILES, ids=lambda p: p.name)
def test_simcore_module_is_stdlib_pure(module_path: Path) -> None:
    offenders = _third_party_imports(module_path.read_text(encoding="utf-8"))
    assert not offenders, (
        f"{module_path.name} imports non-stdlib / non-simcore modules: "
        f"{sorted(offenders)} (core must be stdlib-only, invariant #11)"
    )


def test_scan_is_not_vacuous() -> None:
    """Guard the parametrized scan: if discovery globbed nothing (broken path),
    pytest would report zero cases and the suite would stay green vacuously."""
    names = {p.name for p in _CORE_FILES}
    assert names, "no simcore source files discovered — the purity scan is vacuous"
    # Anchor on modules that must exist so a broken glob can't silently pass.
    assert {"__init__.py", "state.py", "integrator.py"} <= names


# --- discrimination: the detector actually catches a violation ---------------
# "all green on a clean tree" only proves the detector doesn't false-positive; it
# says nothing about whether it would catch `import numpy`. These controls prove
# it discriminates (a purity test that cannot fail is worthless).
def test_detector_flags_third_party() -> None:
    source = "import numpy\nfrom pint import Quantity\n"
    assert _third_party_imports(source) == {"numpy", "pint"}


def test_detector_passes_stdlib_and_intra_package() -> None:
    source = (
        "from __future__ import annotations\n"
        "import math\n"
        "from collections.abc import Mapping\n"
        "from types import MappingProxyType\n"
        "from simcore.flow import FlowResult\n"
        "from simcore import arbitration\n"
        "from . import state\n"
    )
    assert _third_party_imports(source) == set()
