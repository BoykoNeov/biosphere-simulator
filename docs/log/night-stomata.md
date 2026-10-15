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

**Measurement pending** (plan §5).
