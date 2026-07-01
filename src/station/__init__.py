"""Station layer — cross-domain coupling (Phase 6).

The assembly layer that **closes the station, not just the biosphere**: it imports every
sibling domain (Power / Thermal / ECLSS / Crew / the frozen biosphere) and wires them
together **only through shared stocks** — electrical energy, waste heat, O₂ / CO₂,
water, biomass. **No domain imports another**; this layer owns all cross-domain wiring,
so ``git diff src/simcore/`` stays empty (zero core change is the target) and every
standalone domain golden stays byte-identical (the station is a *separate* assembly).
The Phase-3 "coupling machinery lives outside the coupled units" discipline, now
cross-domain.

**Step 1 (P6.1) — the first seam: Power → Thermal heat closure.** Power's dissipation
legs, which standalone dumped into a terminal ``boundary.waste_heat`` sink, are
redirected into ``thermal.node`` (the receiver Thermal built standalone); the
Stefan-Boltzmann radiator rejects that **real** load to deep space. Single-quantity
(ENERGY) — the cleanest possible first integration — and it stands up the station
harness (``build_station`` / ``station_resolver`` / ``run_station``) every later step
reuses. Coupling is pure sink re-wiring: the assembly passes ``thermal.node``'s id where
the Power flows took ``waste_heat`` — zero domain change, zero core change (finding #1).
See ``docs/plans/phase-6-station-integration.md``.

Pure stdlib in the spine (``scenario`` / ``system``); it composes the siblings' stocks +
flows, and reuses their loaders for the YAML/pydantic params.
"""
