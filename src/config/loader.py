"""YAML param loading at the config boundary.

``load_yaml`` reads a param file with ``yaml.safe_load`` (never ``load`` — no
arbitrary object construction) and wraps IO / parse failures as ``ConfigError``.
Schema validation (pydantic) and unit validation (pint) live in the domain loader
and ``units`` respectively; this module is just the safe read.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml

from config.errors import ConfigError
from config.overrides import apply_overrides


def load_yaml(path: str | Path) -> dict[str, Any]:
    """Read ``path`` as YAML and return the top-level mapping.

    Raises ``ConfigError`` if the file cannot be read, is not valid YAML, or does
    not contain a mapping at the top level.
    """
    p = Path(path)
    try:
        text = p.read_text(encoding="utf-8")
    except OSError as exc:
        raise ConfigError(f"cannot read param file {str(p)!r}: {exc}") from exc
    try:
        data = yaml.safe_load(text)
    except yaml.YAMLError as exc:
        raise ConfigError(f"invalid YAML in {str(p)!r}: {exc}") from exc
    if not isinstance(data, dict):
        raise ConfigError(
            f"param file {str(p)!r} must contain a top-level mapping, got "
            f"{type(data).__name__}"
        )
    # The experiment seam. Inert (one dict lookup, the same object back) unless a
    # ``config.overrides.param_overrides`` block is active, which no frozen run opens —
    # so goldens, manifests and the port are untouched by its presence. It sits HERE,
    # before schema validation, so a substituted value goes through pydantic, the pint
    # unit guards, the bound checks and any folding a loader does, exactly as if the
    # file had said it. See docs/plans/post-roadmap-value-switch-harness.md.
    return apply_overrides(p, data)
