"""Outer config layer (outside the pure core).

Owns parameter loading and validation: YAML (``safe_load``) + pydantic schemas,
with pint dimensional checks at the boundary. Converts to the core's canonical
units and plain floats. ``simcore`` must never import this package.
"""
