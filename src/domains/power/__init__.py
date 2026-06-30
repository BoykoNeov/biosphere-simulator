"""Power domain — the first sibling domain (Phase 5, P5.2).

A minimal, **standalone, genuinely energy-conserving** power system: solar (forced)
→ battery (POOL) → dissipative load + charge losses → heat (BOUNDARY sink). It is
the Power analogue of the Phase-1 single-producer season — it ships the *machinery*
(energy-balanced multi-leg flows, the every-step gate now covering ``ENERGY``,
``rationed == 0`` by construction, determinism, a golden), not a calibrated
power-system model.

Phase 5's load-bearing decision (P5.1) was **energy closure**: ``ENERGY`` joined the
asserted conserved set (was balance-exempt through Phase 4). This domain is its first
concrete carrier — every electrical draw names where its energy went, with the
degraded fraction booked as heat (the 3-leg lossy-flow pattern). See
``docs/plans/phase-5-sibling-domains.md``.

Pure stdlib in the simulation spine (``stocks`` / ``flows``); the YAML + pydantic +
pint param loading lives in ``loader.py`` so the spine runs headless.
"""
