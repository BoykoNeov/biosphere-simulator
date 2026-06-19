"""Offline PCSE/WOFOST oracle harness (Phase 1, Step 3).

This package sits **outside** every shipped ``src/`` package on purpose: it imports
PCSE (EUPL), which must never enter our distributable core (invariant #11 / the
reuse-and-licenses clean-room rule). It runs WOFOST as a *validation oracle* and
captures its **output** (a reference trajectory — facts, not PCSE code) as a committed
fixture. See ``runner.py`` for the licensing safeguard (we commit only output +
provenance, never the all-rights-reserved crop-parameter YAML).
"""
