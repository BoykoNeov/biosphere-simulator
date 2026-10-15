## **Stomata that close in the dark** (the latent-heat estimate's first follow-on — the crop's surface resistance has no light term, so it transpires at night at 92 % of its lit rate)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-15** on the user's *"Night stomata closing"*, the first of the three follow-ons in
`log/chamber-latent-heat.md`. Plan, sources, predictions written before any run:
`docs/plans/post-roadmap-night-stomata.md`.

**BUILT lab-only, 2026-10-15.** `science::NightStomata { Open, Closing(f) }` on `TranspirationParams`,
read by the one production builder into `CanopyRead`; on a step with net radiation `≤ 0` the canopy
surface resistance is the day value `/ f` (ASCE-EWRI's hourly day/night rule). The two cited ratios
(McAusland et al. 2023 Table 3, control: 0.123 booting, 0.194 heading) are consts in `science.rs`, not a
YAML key. The shared `science::canopy_surface_resistance` is unchanged (its other readers want the day
value). Switch: `domains::lab::with_night_stomata`. Tests: `rust/crates/domains/tests/night_stomata.rs` —
the rule; the loader keeps `Open`; the open season bit-identical under both ratios on every step (no dark
step outdoors); on a forced-dark step the flow's draw is Penman–Monteith at `rs/f` to 1e-12 and a lit step
is untouched. Full suite green; goldens unchanged.

**Corrected on the way:** the latent-heat plan's *"it would move water in every sealed golden"* — the
biosphere crate's sealed jars read the outdoor daily-mean net radiation and never see a dark step; only
the three lamp-lit station goldens can move (plan §2a).

**MEASURED 2026-10-15** (plan §6), in a scratch worktree with the loader line flipped to each ratio;
every control held (no rationing, legs bit-exact, water books closing). Per ratio, booting 0.123 /
heading 0.194, against today's open stomata:

* warm dark windows' crop latent **116.19 W → 19.75 / 30.20 W**; the dark-hours heat with nothing to pay
  for it **1.80 → 0.31 / 0.47 GJ**; the chamber's water over 1220 days 3473 → 2832 / 2918 kg;
* the night air keeps its vapour (3.333 → 3.338 / 3.332 kg), so the drop is not offset by drier air;
* TM 102788 water W1 **1.041 → 0.958 / 0.967**; its carbon rows identical;
* `regen_goldens`: **2 move** (`lighting_state`, `sealed_station_state`), **water only** (condensate, soil
  and subsoil water, the soil-surface counters); 18 identical.

**Prediction missed:** three goldens were predicted. `sealed_energy_drift_summary` holds only the node's
peak temperature and stationarity, and the model gives evaporation no heat, so a water-only change cannot
reach it. The prediction asked whether the switch can FIRE in a build, not whether the golden RECORDS
anything the switch moves — ask the second question too.

**The rule's bias, estimated:** ASCE's binary rule counts a partly lit step as day, which leaves 2 of
the warm phase's 8 dark hours with open stomata — at the open dark rate ~0.60 GJ over the run, of which a
lit-fraction rule would remove ~0.44–0.50 GJ more. Not built: the flow has no lit-fraction signal.

**Open, the user's:** adopt or not, and at which ratio (an unfreeze: two station goldens, water only).
