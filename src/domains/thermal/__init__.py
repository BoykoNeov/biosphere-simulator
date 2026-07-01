"""Thermal domain — the second sibling domain (Phase 5, Step 5).

A minimal, **standalone, energy-conserving** thermal system: a forced heat input → an
in-system thermal node (POOL, sensible heat J with a derived temperature) → a
Stefan-Boltzmann radiator rejecting to the permanent deep-space boundary. It ships the
*machinery* (a nonlinear donor-controlled radiator, temperature + heat capacity, an
emergent equilibrium temperature, the every-step ENERGY gate, a golden) — not a
calibrated thermal-control model.

Thermal is the sibling that reveals **where Power's ``waste_heat`` went**: Power (P5.2)
dumped degraded joules into a terminal ``boundary.waste_heat`` sink — a temporary seam.
Thermal makes the "somewhere" concrete (an in-system node + a radiator) and gives heat a
place to go. The genuinely-new content over Power is the **nonlinear ``T⁴`` radiator** →
a *real* emergent equilibrium temperature (a genuine attractor, unlike Power's
*constructed* daily balance). It reuses the Phase-5 energy-closure machinery (P5.1);
``boundary.space`` stays a permanent boundary — heat truly leaves. **Phase 6 rewires
Power's dissipation legs to feed ``thermal.node``** (the inward move); standalone
Thermal builds the receiver. See ``docs/plans/phase-5-sibling-domains.md``.

Pure stdlib in the simulation spine (``stocks`` / ``flows`` / ``scenario`` /
``system``); the YAML + pydantic radiator params load via ``loader.py`` so the spine
runs headless.
"""
