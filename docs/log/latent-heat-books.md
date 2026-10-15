## **Evaporation's heat in the room's books** (the latent-heat estimate's third follow-on — the plants' water cost the room no heat, so a held room never paid for its own transpiration)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-15** on the user's *"yes"*, the third of the three follow-ons in
`log/chamber-latent-heat.md` (night stomata and the switched lamp heat adopted earlier the same day). Plan,
readers and predictions written before any run: `docs/plans/post-roadmap-latent-heat-books.md`.

**BUILT lab-only, MEASURED 2026-10-15** (plan §7). `station::latent_heat::with_latent_heat` records each
plant window's evaporation rate from the registry's own transpiration flow (one instance, shared with the
recorder) and draws λ × it from `thermal.chamber` to `thermal.node` minute by minute across the window — a
lump at the plant step would drop the room ~5 K every 90 minutes. The simple form credits the coil at
evaporation, not condensation; its bound is λ × the air's vapour range, 8.2 MJ. Tests:
`rust/crates/station/tests/latent_heat.rs`.

Measured over 1220 days on the reference walls: the heater 0.31 → 1.34 GJ (by day too: Penman–Monteith's
dry-air term drives the crop past what the lamp pays), the battery −1.03 GJ, no rationing; the cooler alone
no longer saturates in the warm phase but cooler + latent exceeds its 200 W as often as before (a side
note before the run was right: the latent path is uncapped). Every season is one day longer and its grain
+3.4 % — ripening is checked at day ends, and the room's ~0.05 K cooler day tips it over one. The lab
structure walls cannot afford the evaporation's heat: the battery empties and the fast side rations.

**Adoption: not yet taken** (plan §5: a lab surroundings rations).
