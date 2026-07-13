"""Authoring-layer error type (the boundary's own failure surface).

An :class:`AuthoringError` marks a scenario-file that is *structurally* invalid at
interpret time — an unknown flow type, a wiring dict that does not match the flow
type's constructor fields, a missing/spurious param-set reference. It is the
authoring analogue of ``config.ConfigError`` (bad param YAML): raised in the
boundary layer, before any engine step runs.

It is **distinct from** a runtime ``simcore.flow.ConservationError``: a well-formed
scenario that *wires* a flow badly (e.g. a carbon flow's withdrawal leg pointed at
an oxygen stock) interprets cleanly and then surfaces as a ``ConservationError`` on
the first step — the "bad wiring surfaces, never silently fixed" safety property
(Phase-9 decision B). ``AuthoringError`` catches only what is decidable from the
file structure alone.
"""

from __future__ import annotations


class AuthoringError(Exception):
    """A scenario file is structurally invalid at interpret time."""
