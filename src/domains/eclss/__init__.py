"""ECLSS / Atmosphere domain — the third sibling domain (Phase 5, Step 6).

A minimal, **standalone, mass-conserving** cabin-air system: the shared cabin-air
**medium** (three single-quantity POOLs — O₂ / CO₂ / humidity) with a forced crew
metabolic load and three ECLSS control loops (a first-order CO₂ scrubber, a first-order
condenser, and a demand-controlled O₂-makeup regulator). It ships the *machinery* — the
**first multi-quantity sibling** (the every-step gate asserts CARBON, OXYGEN and WATER
simultaneously — the payload), three geometric restoring-force attractors, the golden —
**not** a calibrated life-support model (calibration against NASA BVAD / BioSim is
Phase 6).

ECLSS is the sibling that builds the **cabin-air receiver** the way Thermal built the
receiver for Power's heat: standalone it stands the crew metabolism in as a forced
boundary seam, and **Phase 6 wires the Crew domain's real O₂/CO₂/H₂O exchange (and the
biosphere's O₂/CO₂ interface) into these cabin stocks** — the inward move. Composition
stocks (CO₂ = ``{CARBON:1, OXYGEN:2}``), ideal-gas pressure readouts and an inert N₂
diluent are **deferred seams** (no standalone flow needs them — the discipline that
rejects speculative generality until a consumer exists). See
``docs/plans/phase-5-sibling-domains.md`` (Step 6).

Pure stdlib in the simulation spine (``stocks`` / ``flows`` / ``scenario`` /
``system``); the YAML + pydantic ECLSS params load via ``loader.py`` so the spine runs
headless.
"""
