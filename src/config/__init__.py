"""Outer config layer (outside the pure core).

Owns parameter loading and validation: YAML (``safe_load``) + pydantic schemas,
with pint dimensional checks at the boundary. Converts to the core's canonical
units and plain floats. ``simcore`` must never import this package.
"""

from config.errors import ConfigError, UnitValidationError
from config.loader import load_yaml
from config.units import convert, to_canonical

__all__ = [
    "ConfigError",
    "UnitValidationError",
    "convert",
    "load_yaml",
    "to_canonical",
]
