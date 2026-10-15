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
| `lighting::lighting_power_resolver` (the standalone `lighting` seam) | daily average | **switched**, as the group's mean | one lamp, one law; its energy goes to boundary sinks, no room. ⚠ Its 24 power hours run in 8 groups of 3 over 2 plant windows, and `n` cannot tell a group's hours apart, so the draw is the mean of the group's two windows (found while building; the sealed station's group is one window) |
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

## 7. Measured (2026-10-15)

**The lab switch** (`station::sealed::with_switched_lamp_heat`, `station::lighting::switched_lamp_draw`,
`lighting_power_resolver_switched`; tests `rust/crates/station/tests/room_lamp_heat.rs`). **L5 held**: the
fast minutes after plant step `k` draw the on-power × plant window `k`'s lit share (to 1e-12, both phases,
across a day boundary); a day's draw is the average's energy to 1e-12; the lighting seam's 3-hour groups
draw the light of their two windows.

**The sealed station, 1220 days** (`chamber_latent_heat`, averaged against `--switched`). Controls in both:
`rationed` 0/0, legs re-applied 0 mismatches, water books closing to 7e-12 kg; the averaged run byte-exact to
its golden.

| | averaged (today) | switched |
|---|---|---|
| lamp into the room, warm lit / warm dark windows | 133.3 / 133.3 W | 177.8 / **0** W (partly lit windows count as lit) |
| heater-on minutes, warm / cold phase | 421 / 810 | **257 040 / 38 310** |
| heater energy, warm / cold | 5.0 / 9.7 MJ | 287.5 / 18.4 MJ |
| cooler energy, warm / cold | 7145 / 1064 MJ | 7422 / 1074 MJ |
| warm phase: room above setpoint, max / minutes > 0.1 K / cooler at capacity | +0.053 K / 0 / 0 min | **+0.131 K / 1208 / 1800 min** |
| warm phase: room below setpoint, deepest (the warm-up after each cold period) | −6.46 K | **−11.18 K** |
| warm dark windows: crop latent / heat in | 19.75 / 133.66 W | 19.62 / 18.54 W |
| dark windows whose water heat is not covered | 0 | 1787 warm (0.10 GJ), 17 cold |
| water through the root zone | 2832.3 kg | 2841.3 kg (+0.32 %) |
| end crop: leaf / stem / root / grain C | 6.5068 / 4.2908 / 4.4345 / 0.8963 | 6.5137 / 4.2919 / 4.4390 / 0.8953 (≤ 0.11 %) |
| end battery | 1.1150e10 J | 1.0865e10 J (−2.6 %, the heater) |
| end node | 1.71331e9 J | 1.71533e9 J (+0.12 %) |

**Graded.** L5 held. **L2 held**: carbon moves, ≤ 0.11 % on every crop stock. **L3 held**: the heater fires
on the warm nights (257 040 minutes against 421); the cooler removes 3.9 % more in the warm phase, not "about
the same" — the heater's night heat leaves through it by day. **L4 held, smaller than feared**: the cooler
saturates 1800 minutes (30 hours in 3.3 years) and the room peaks 0.13 K above its setpoint — the warmest
outdoor day is 22.20 °C, 0.05 K above the setpoint. **L6 held** (no rationing). L1 is graded at adoption.

**Not predicted:** the warm-up after each cold period is slower — the room dips to 11.2 K below its new
setpoint, where the averaged lamp's night heat (133 W on top of the 200 W heater) held it to 6.5 K. That is
the real lamp: off at night, so only the heater warms the room in the dark.

**No fork** (§5): the cooler's saturation is 0.13 K for 30 hours, nothing rations or sheds, and the form has
no free choice — the lamp is on when its light is on. So it is adopted (§8).

**For the latent-heat follow-on** (`post-roadmap-chamber-latent-heat.md`): with the room's heat switched, the
dark windows' water heat is now booked against what actually comes in (the walls and the heater): 1787 warm
dark windows are short, 0.10 GJ over the run.

## 8. Adoption (2026-10-15)

### 8a. What changes

* `sealed_fast_resolver` and `lighting_power_resolver` carry the switched draw (the phase twins on the
  sealed station). The lab switch inverts: `with_averaged_lamp_heat` restores the average for instruments
  that reproduce older records; `lighting_power_resolver_switched` folds into `lighting_power_resolver`.
* `chamber_heat0`'s start input is the first window's draw (0: window 0 is dark), not the average.
* `lamp_shed` (lab): `nominal` is the program's switched draw; a group with nothing owed (dark) delivers 1.
* `sealed_node_heat` keeps the average (§2).
* The tests that pin the average by name are restated to the switched lamp, not deleted.

### 8b. Predictions (written before the resolvers change)

* **A1** `regen_goldens`: `sealed_station_state` moves; `lighting_state` moves at most at rounding in
  `power.battery`, `boundary.light_used` and `boundary.waste_heat` only, or not at all; the other 18
  byte-identical (the energy-drift golden has no lamp).
* **A2** `sealed_station_state`'s moved keys include carbon, water, energy and the chamber; its end
  state matches the §7 switched run's to the digits printed there.
* **A3** Manifests: the station manifest's `golden_sha256` rows for the moved goldens only; biosphere and
  authoring byte-identical; `simcore` untouched.
* **A4** Before regeneration the reds are: the goldens' byte and band checks, `chamber_heat`'s
  failed-cooler linear warming and nominal day-end checks, `chamber_walls`' lamp-energy saving,
  `sowing_clock`'s average-draw check, and the lab `lamp_shed` tests that read a delivery or a nominal.
  Others are findings.
