# The room's lamp heat switched with the light — a lab switch first

**Opened 2026-10-15** on the user's *"then to the others in order"*: the second of the three follow-ons
the latent-heat estimate left (`post-roadmap-chamber-latent-heat.md` §5b), after night stomata
(`post-roadmap-night-stomata.md`, adopted the same day). **This doc builds a lab switch and measures it.**
Adoption is a station unfreeze; it is taken here only if the measurement leaves one sensible form, and
put to the user in plain words if it shows a fork (§5).

## 0. Why

The grow lamp's light switches: the plants get a top-hat, on for the photoperiod, off at night. Its
**heat does not**: the fast side's `Lamp` flow draws the lamp's daily average (133.3 W warm, 13.3 W
cold) on every minute, lit or dark, and that draw is the plant chamber's heat input. So the room is
never dark to its heat books. The latent-heat estimate found this hides the night from the heater (421
heater minutes in 1220 days) and confounds any per-window heat check. A real lamp heats the room only
while it is on.

The asymmetry was deliberate when written (`lighting.rs`: *"Power is the fast operator and `substep`
freezes `n`, so a within-day shape is not expressible there"*). That reason is gone: since the
interleaved day (2026-09-30) and the 1/16-day plant step, the fast side sees `n` take 16 values a day,
one per plant window — the resolution the plants' own light has.

## 1. The form

The fast-side `lamp_power` (each phase twin) becomes the lamp's on-power on the plant step's own
top-hat: for phase `p`,

`lamp_power(n) = top_hat_window_mean(frac(w·dt_bio), dt_bio, P_on(p), photoperiod_s(p))`,
`w = n + steps_per_day − 1` (the window `n − 1`, a whole day later so it never underflows).

* **The window, `n − 1`.** The interleaved day runs plant step `k` (which moves `n` to `k + 1`), then
  that window's 90 minutes on the fast side with `n = k + 1`. The plants' light for those minutes is
  window `k`'s (`gas_exchange.rs`: the plant step records its window). Keyed on `n` the lamp would heat
  the room 90 minutes before the light came on and go dark 90 minutes early — the one-step lead every
  fast forcing keyed on `n` has (§23j of the room-temperature plan). The outdoor temperature and the
  setpoint keep that lead; this change does not touch them.
* **The dose.** Partly lit windows get their lit share; a day's energy is `P_on · photoperiod`, the
  average's `P_avg · 86 400 s`, to rounding.
* **Phase twins.** The sowing clock picks the twin by the state's phase. A phase change falls at a day
  boundary, whose window `n − 1` is the previous day's last (dark at every photoperiod ≤ 21 h), so the
  lead cannot pick the wrong lamp in a lit window.

## 2. Every reader of the lamp's draw

| Reader | Today | Under the switch | Why |
|---|---|---|---|
| `sealed::sealed_fast_resolver` (`lamp_power` twins) | daily average | **switched** | the change |
| `lighting::lighting_power_resolver` (the standalone `lighting` seam) | daily average | **switched** | one lamp, one law; its energy goes to boundary sinks, no room |
| `chamber::chamber_heat0` via `sealed.rs` (the chamber's start) | steady state of the average | the **first window's** draw (0: window 0 is dark) | the start assumes a constant input; the first minutes are dark |
| `sealed::sealed_node_heat` (the thermal node's start) | the average | **kept** | the node's time constant is days; its start is a daily-mean equilibrium, and the switch keeps the daily mean |
| `lamp_shed.rs` (lab) `nominal` | the average | the program's switched draw | otherwise every group compares a switched delivery with an averaged nominal; dark groups need a stated rule (0/0) |
| `perturbations.rs` (lighting failure scales `lamp_power`) | composes | composes | a map over the schedule, unchanged |
| `godot_bridge` | `sealed_fast_resolver` | inherits | the same resolver |

Tests that pin the average by name: `chamber_heat.rs` (a failed cooler warms at `w/C` exactly linear,
"the lamp's draw is the daily average"; the nominal chamber holds one temperature at every day end),
`chamber_walls.rs` (lamp energy saved over a 5-day failure), `sowing_clock.rs` (the lamp draws the
phase's average on a sampled step).

## 3. Hazards (each checked by the measurement)

* **The cooler is at its limit by day.** `cooling_capacity` is 200 W — the lamp's nameplate (BVAD's
  class rule) — and the lit lamp now puts in exactly 200 W. On a day the outdoor air is warmer than the
  chamber, the walls add heat on top, the cooler saturates and the chamber warms above its setpoint by
  day. The plants read that temperature.
* **The heater fires at night**, on the battery, whenever the walls lose heat with the lamp off.
* **The battery** swings within the day against a constant solar average (±~67 W for 16 h, ~3.8 MJ
  against a 2e10 J store): no rationing or shedding expected in the reference.

## 4. Predictions (written before any code ran)

* **L1** `regen_goldens` under the switch: `sealed_station_state` moves; `lighting_state` moves at most
  at rounding, in the battery and the two lamp sinks only (a day end carries the same daily energy);
  the other 18 identical. `sealed_energy_drift_summary` cannot move: it is built from the heat-closure
  station, which has no lamp.
* **L2** In `sealed_station_state` carbon moves as well as energy and water (the plants read the
  chamber temperature, which now steps at each light change) — by well under 1 % on every carbon stock.
* **L3** The chamber's books over the 1220-day run: the heater fires on most warm-phase nights (thousands
  of minutes, not 421); the cooler removes about as much heat in total as before (the dose is the same).
* **L4** The cooler saturates on some lit windows (any day the outdoor air is above ~22 °C), and the
  chamber's peak rises above the setpoint by up to a few K there. If it does, that is the fork of §5.
* **L5** One day's switched draw equals the average × 86 400 s to ≤ 1e-12 relative, for both phases; the
  fast lamp energy over each window equals the plants' lit share × on-power × 5400 s.
* **L6** No rationing (`rationed == 0`) and no lamp shed in any reference run.

## 5. When to stop and ask

Adopt with the full ceremony if one form is the sensible one. Stop and put it to the user first if the
cooler saturates by day (whether a real chamber's cooler would be sized above the lamp is a design
choice: `cooling_capacity` is DESIGN, "sized by a class rule"), if anything rations or sheds, or if a
scope question has no answer on physics.

## 6. Measurements

1. The lab switch (`station::sealed::with_switched_lamp_heat`, lab-only; the loader unchanged): the
   L5 checks as tests.
2. `chamber_latent_heat` (the sealed station, 1220 days) with `--switched`: the chamber's books
   (lamp, walls, heater, cooler; heater and saturated minutes), the chamber temperature's range, the
   dark windows' unfunded heat, the end state against the averaged run.
3. A scratch worktree with the switch wired in: `regen_goldens` (report) and the suite.
