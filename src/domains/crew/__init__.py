"""Crew domain — the last forward-pointer sibling (Phase 5, Step 7).

A minimal, **standalone, mass-conserving** crew life-support-consumer system: three
finite provisioned-consumable POOLs (food / water / O₂) drawn down by three **forced**
metabolic flows, each splitting an ingested quantity across named output fates (respired
CO₂ vs feces; insensible humidity vs urine; consumed O₂). It ships the *machinery* — the
first **net-consumer / open-loop** sibling — **not** a calibrated crew model
(calibration against NASA BVAD / BioSim is Phase 6).

**Crew is the LEAST closed sibling — and that is the point.** Where the biosphere loops,
Power oscillates, and Thermal / ECLSS reach steady states, standalone Crew just **runs
down**: forced draws deplete the stores with no resupply. That standalone incompleteness
is the narrative argument for Phase-6 closure (the crew starves once the stores empty
unless the biosphere + ECLSS regenerate them). Multi-quantity like ECLSS (the every-step
gate asserts CARBON / OXYGEN / WATER); **forced ⇒ RK4 ≡ Euler bit-identical** (the
symmetric bookend to ECLSS/Thermal, which broke that identity).

**Crew is the real version of ECLSS's forced ``CrewMetabolism`` stand-in.** ECLSS (Step
6) defined this seam from the cabin's side (``metabolic_o2_sink`` /
``metabolic_co2_source`` / ``metabolic_h2o_source``); Crew owns the crew's side of it.
This is the "Thermal builds the receiver Phase 6 wires Power's ``waste_heat`` into"
analogue: **Phase 6 deletes ECLSS's stand-in and wires Crew's outputs into the cabin
stocks** (CO₂ → ``cabin_co2``, humidity → ``cabin_h2o``, O₂ intake ← ``cabin_o2``) — a
*subset*; Crew's urine, feces and consumed O₂ route to *other* Phase-6 systems (water
recovery, solid-waste, the atomic-coupling sink), which is exactly why those outputs are
split into separate legs. Composition stocks (CO₂ = ``{CARBON:1, OXYGEN:2}``) and the
crew's atom-level stoichiometry (``C_food + O₂ → CO₂ + H₂O``) are **deferred seams** (no
standalone flow needs them). See ``docs/plans/phase-5-sibling-domains.md`` (Step 7).

Pure stdlib in the simulation spine (``stocks`` / ``flows`` / ``scenario`` /
``system``); the YAML + pydantic Crew params load via ``loader.py`` so the spine runs
headless.
"""
