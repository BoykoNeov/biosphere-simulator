"""Errors raised at the config boundary.

Kept in their own module so ``loader`` (YAML/IO) and ``units`` (pint) can share
them without an import cycle. ``simcore`` never imports this package.
"""

from __future__ import annotations


class ConfigError(Exception):
    """A param file could not be read, parsed, or validated at the boundary."""


class UnitValidationError(ConfigError):
    """A param's declared unit is unparseable or dimensionally incompatible with
    its quantity's canonical unit (decision #9)."""
