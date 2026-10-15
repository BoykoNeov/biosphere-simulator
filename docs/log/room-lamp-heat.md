## **The room's lamp heat switched with the light** (the latent-heat estimate's second follow-on — the room drew the lamp's daily average on every minute, lit or dark, so its heat books never had a night)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED 2026-10-15** on the user's *"then to the others in order"*, the second of the three follow-ons in
`log/chamber-latent-heat.md` (night stomata, the first, adopted earlier the same day). Plan, readers and
predictions written before any code: `docs/plans/post-roadmap-room-lamp-heat.md`.

**The form.** The fast side's `lamp_power` (each phase twin on the sealed station) is
`station::lighting::switched_lamp_draw`: the lamp's on-power on the plant step's own top-hat, for the
windows of the interleaved day's group behind `n` — window `n − 1` on the sealed station (one plant step,
then its 90 minutes), the mean of two windows on the lighting seam (3 power hours over 2 plant windows,
which `n` cannot tell apart). Keyed on `n` the room would have heated 90 minutes before the light came
on. A day's energy is the average's to 1e-12. The old reason for the average — *"`substep` freezes `n`,
so no within-day shape is expressible"* — held only while a plant step was a day.

**Every reader of the lamp's draw** (plan §2): the sealed and lighting resolvers switched; the chamber's
start reads the first window's draw (0, dark); the lab lamp shed's nominal is the UNPERTURBED switched
program (read off the resolver, a lamp failure cut into it would have read as delivered — caught while
building); `sealed_node_heat` keeps the daily mean, stated (the node's time constant is days). The daily
average survives as the lab switch `station::sealed::with_averaged_lamp_heat`.

**MEASURED, lab switch, 1220 days** (plan §7): the heater fires on the warm nights (257 040 minutes, was
421; 287 MJ off the battery); the cooler, sized to the lamp's 200 W nameplate, saturates 30 hours in 3.3
years and the room peaks 0.13 K above its setpoint; the warm-up after each cold period is slower (−11.2 K
at its deepest, was −6.5: only the heater warms a dark room); crop carbon ≤ 0.11 %, water +0.32 %; no
rationing. Predictions L2–L6 held; L4's saturation was smaller than feared. **No fork**, so adopted.

**ADOPTED 2026-10-15** (plan §8). 2 goldens: `sealed_station_state` (carbon, water, energy; its end state
the lab run's to every printed digit) and `lighting_state` (its two lamp sinks at ~1e-15, the battery
unmoved); the station manifest's two `golden_sha256` rows; biosphere and authoring manifests unchanged.

**The reds** (plan §8c; A4 graded *held in part*): 13 tests went red, none deleted or loosened. Eight
were in the files predicted (`chamber_heat` — on its node line, not the two lines named; `chamber_walls`'
dead-lamp test; `sowing_clock`; five lab `lamp_shed` tests). Five were findings: the direction plan's
re-read gate (process, the same every item); the retired day order, which
runs every power hour at the day's last `n` and so draws a switched lamp in the dark all day (the
order claim is now held on the averaged lamp, `lighting_power_resolver_averaged`, and the dark day
asserted); and three long-horizon pins in `chamber_walls`, re-pinned as measured (the node 0.28 K warmer
on the mean) with the chamber's day-end bands now DERIVED — a day end is dark, so the room sits within
one step's wall loss (0.015 K) of its setpoint. **A bug of this item's own:** the lab lamp shed wrote
"fully delivered" for a dark group, and the next lit window reads that share, so a shed lamp came back
at every dawn; a dark group now repeats the last share (`a_shed_lamp_on_the_sealed_station_never_
comes_back` caught it, unchanged). **Not predicted:** in the lab's full-lamp cold period the lit lamp
and the walls outrun the cooler and warm the room ~4 K by day, so the plain crop feels a cut lamp only
in the lit hours — a day-end state cannot see it.

**For the third follow-on** (latent heat in the room's books): with the room's heat switched, 1787 warm
dark windows' water heat is not covered by what comes in (walls and heater) — 0.10 GJ over the run.
