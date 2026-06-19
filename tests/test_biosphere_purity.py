"""Gate: the biosphere **simulation spine** is stdlib-pure (no config/pint/pydantic).

The biosphere domain is split (CLAUDE.md, Phase-1 carried invariant #11): the
flows / aux-rates / scenario assembly stay **stdlib-pure** so the simulation runs
headless, while the **loader** is the sole config boundary that imports the outer
stack (pydantic + pint via ``config``). The ``simcore`` AST purity gate
(``test_simcore_purity.py``) only scans ``simcore``, so it never sees
``domains/biosphere/`` — this module closes that gap for the pure spine, exactly as
``test_oracle_match.py`` pins "``lab`` imports no pcse".

A pure biosphere module may import only the standard library or the pure project
packages (``simcore`` — itself gate-proven pure — and ``domains``). Importing
``config`` (or pint/pydantic/yaml directly) is the violation this catches; that is
the loader's job alone, so ``loader.py`` is **excluded** by name (and a test pins
that the loader genuinely is the boundary, so the exclusion is not hiding a leak).
"""

import ast
import sys
from pathlib import Path

import pytest

import domains.biosphere

_PURE_PACKAGES = {"simcore", "domains"}
_BIOSPHERE_DIR = Path(domains.biosphere.__file__).parent
# The config boundary: the ONE biosphere module allowed to import the outer stack.
_BOUNDARY = "loader.py"
_PURE_FILES = sorted(
    p for p in _BIOSPHERE_DIR.rglob("*.py") if p.name != _BOUNDARY
)


def _is_pure_top_level(name: str) -> bool:
    """A top-level module name a pure biosphere module is allowed to import."""
    return name in sys.stdlib_module_names or name in _PURE_PACKAGES


def _impure_imports(source: str) -> set[str]:
    """Top-level imported names that are neither stdlib nor a pure project package.

    Whole-AST walk (covers function-local / ``TYPE_CHECKING`` imports); relative
    imports are intra-package and skipped. Empty set ⇒ pure.
    """
    offenders: set[str] = set()
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.Import):
            for alias in node.names:
                top = alias.name.split(".", 1)[0]
                if not _is_pure_top_level(top):
                    offenders.add(top)
        elif isinstance(node, ast.ImportFrom):
            if node.level or node.module is None:
                continue
            top = node.module.split(".", 1)[0]
            if not _is_pure_top_level(top):
                offenders.add(top)
    return offenders


@pytest.mark.parametrize("module_path", _PURE_FILES, ids=lambda p: p.name)
def test_biosphere_pure_module_is_stdlib_pure(module_path: Path) -> None:
    offenders = _impure_imports(module_path.read_text(encoding="utf-8"))
    assert not offenders, (
        f"{module_path.name} imports the outer/3rd-party stack: {sorted(offenders)} "
        "(the biosphere spine must be stdlib-pure; config/pint/pydantic belong in "
        "loader.py only — invariant #11)"
    )


def test_scan_is_not_vacuous() -> None:
    # If discovery globbed nothing, the parametrized scan would pass vacuously.
    names = {p.name for p in _PURE_FILES}
    assert names, "no pure biosphere source files discovered — the scan is vacuous"
    assert {"flows.py", "demo.py", "canopy.py"} <= names


def test_loader_is_the_excluded_boundary() -> None:
    # The exclusion is real, not a typo: loader.py exists and is the boundary that
    # *does* import config — so the scan would (correctly) have flagged it.
    loader = _BIOSPHERE_DIR / _BOUNDARY
    assert loader.is_file()
    assert "config" in _impure_imports(loader.read_text(encoding="utf-8"))


def test_detector_flags_outer_stack() -> None:
    # Discrimination control: the detector must actually catch a leak (a gate that
    # cannot fail is worthless).
    source = "from config import to_canonical\nimport pint\n"
    assert _impure_imports(source) == {"config", "pint"}


def test_detector_passes_stdlib_and_pure_packages() -> None:
    source = (
        "import math\n"
        "from dataclasses import dataclass\n"
        "from simcore.flow import FlowResult\n"
        "from domains.biosphere.canopy import CanopyParams\n"
        "from . import flows\n"
    )
    assert _impure_imports(source) == set()
