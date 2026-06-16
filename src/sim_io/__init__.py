"""Outer I/O layer (outside the pure core).

Owns serialization formats: JSON state checkpoints and hex-float golden/
regression files for exact cross-run and cross-port comparison. May depend on
third-party libraries; ``simcore`` must never import this package.
"""

from sim_io.snapshot import (
    SCHEMA_VERSION,
    dumps,
    loads,
    state_from_dict,
    state_to_dict,
)

__all__ = [
    "SCHEMA_VERSION",
    "dumps",
    "loads",
    "state_from_dict",
    "state_to_dict",
]
