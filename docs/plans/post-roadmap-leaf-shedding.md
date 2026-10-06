# Tissue shedding that reads no temperature — a design note (2026-10-06)

**Status: OPENED 2026-10-06 as a design note, no code.** The user's call after Step 3c slice 3b
(`post-roadmap-room-temperature.md` §24m, "DECIDED 2026-10-06": *"Keep 100, study shedding"*).
Any change to `Senescence` is a **biosphere unfreeze** (the frozen science, `docs/biosphere-
reference.md`): it moves every biosphere golden and the station's. Nothing is built until the
user picks.

## 1. Why this is open

Slice 3b dimmed the cold weeks' lamp, and the seedling lost half its carbon over the 56 cold days
(0.160 → 0.077 mol C). Booked flow by flow (a temporary example replaying the reference step order;
the booked legs close on the observed change exactly):

| 56 cold days, mol C | 3b, dim lamp | 3a's world, full lamp |
|---|---|---|
| growth into the organs (`Allocation`) | +0.0089 | +0.4691 |
| maintenance respiration | −0.0185 | −0.0165 |
| shed, **leaf** | −0.0338 | −0.1153 |
| shed, **root** | −0.0327 | −0.0636 |
| shed, **stem** | −0.0069 | −0.0100 |
| change | −0.0829 | +0.2636 |

So the cold seedling is not starving — growth and upkeep nearly cancel (−0.0096) — it is
**shedding**: `Senescence` removes a fixed fraction of each organ per day (`rdr_leaf` 0.02,
`rdr_root` 0.01, `rdr_stem` 0.005, all `TODO(cite)`), reading no temperature and no development
stage. At 4 °C tissue dies as fast as at 22 °C. ⚠ First reported to the user as "leaves"; roots shed
nearly as much (corrected the same day).

## 2. What the shelf says (searched 2026-10-06 under several headings — "senesc", "leaf death",
"death rate", "life span", "longevity", "turnover", "freez" — across every PDF in `sources/`)

| Source (on the shelf) | Locus | Before flowering | After flowering / seed growth |
|---|---|---|---|
| Penning de Vries et al. 1989, §3.2.6 p. 95 + Listing 5 p. 212 (already `senescence.yaml` [A]) | rice IR36 crop file; potential production | leaf and root death **0** below DS 1.0; "except for their reserves, stems do not lose weight" | DS-keyed rates (leaf 0.007 → 0.012 /day, root 0.011 → 0.010) |
| Soltani & Sinclair 2012, Box 9.1 + Eqn 9.7, pp. 105–109 (already cited [C] for phenology) | generic grain crops, **non-limiting N** (Ch. 9's stated assumption) | **"No leaf senescence"** from emergence to beginning seed growth | LAI falls linearly to 0 at maturity, paced in **temperature units** (DTU) |
| Teh, *Introduction to Mathematical Modeling of Crop Growth*, Eqn 7.17 pp. 155–156, after Goudriaan & van Laar 1994 (SUCROS) | generic; Teh's model | leaf death from age **0** before flowering | `DVR / max(0.1, 2 − DVS)` — paced by the **development rate** (temperature-driven) |
| Teh, p. 158 (line "no root turnover or root death") | Teh's model | no root death | no root death |
| Shading: Soltani Eqn 9.11 / Teh Eqn 7.18 (Goudriaan & van Laar: ≤ 0.03 /day above LAI 4–5); Van Keulen & Seligman via Penning de Vries p. 101 (5 %/day above LAI 6, **built** 2026-08-15) | canopy closure | a seedling never reaches it | — |
| Freezing: Soltani Eqn 9.13 | min temperature below 0 °C | never fires at 4 °C | — |
| Soltani Ch. 17 (N mobilization drives leaf senescence) | N-limited crops; vegetative senescence only when N uptake is very limited | — | seed-fill N mobilization |

**All three crop-model books agree: no age-related leaf death before flowering, and what comes
after is paced by temperature or development.** None supports a daily rate during vegetative
growth. Our flat `rdr_leaf` was recorded in July as standing in for canopy regulation the tree did
not then have (`senescence.yaml`, the (C) diagnosis).

## 3. The July refusal, and which of its blockers are still gates (2026-10-06)

The DS-keyed form ("(C)") was priced and refused 2026-07-27/28 (`post-roadmap-nitrogen-cycle-
form.md`, "THE (C) DIAGNOSIS"; `senescence.yaml` header). Its blockers, re-checked against today's
tree — a refusal is dated to its state, and so is each gate it failed:

| July blocker | A gate today? |
|---|---|
| canopy peak LAI 16.4 without vegetative leaf death | **discharged** — the shading rule (5 %/day above LAI 6) was BUILT 2026-08-15 |
| `perennial` hard-errors under RK4 | **not a reference gate** — the biosphere is frozen at Euler; RK4 appears only in lab tests (`tests/leaf_form.rs`) |
| `perennial`'s decade CO₂ floor 0.05 | **yes** — `science_gates.rs:566`, `non_collapsing(floor=0.05)` (its window was removed 2026-08-10, a tightening) |
| Greenwood biomass cap 14.4248 t/ha | **yes** — `tests/mutual_shading_tolerance.rs` |
| `rationed == 0` in every golden | **yes** |

The tree has changed since July in ways that bear on all of these: the step (`dt` 1/16), the
specific leaf area re-cited, the shading rule, temperature kinetics, the live-O₂ form, the soil
layers. None of July's numbers can be quoted as today's.

## 4. The options (for the user)

* **A — the sources' form.** No leaf or root death before flowering; after it, paced by
  development (Teh/SUCROS's `DVR/(2 − DVS)`, or Soltani's linear decline in temperature units);
  stems per Penning de Vries (no structural death). CITED. A biosphere unfreeze; it hits the
  carbon-limited chambers that refused (C) in July.
* **B — the flat rates converted to degree-days.** Keep today's magnitudes but count them per
  °C·day (so 4 °C sheds about a fifth as fast as 22 °C). ⚠ UNCITED: the conversion needs a
  reference temperature, which would be fitted — WHAT-IF, lab-only by the project's rules.
* **C — leave it, recorded.** The cold-week loss stays; slice 4 goes ahead.

## 5. The measurement plan (owed before the user can pick A or B; nothing frozen moves)

Through the existing lab seam (`domains::lab::mechanism`, `build_season_replacing` — one assembly
body), never by editing the frozen `senescence.yaml`. Predictions written before each run.

1. **Control:** the frozen form through the seam reproduces every golden bit for bit.
2. **A**, organ by organ (leaf, root, stem separately, then together — July's stem-only lesson:
   the organs do not move together), and **B** (WHAT-IF), on every biosphere golden scenario plus
   the sealed station, reporting the chambers and the station in **separate columns** (the
   station's shared air is not CO₂-limited, so it will look like an easy win while the jar and
   `perennial` pay).
3. Against today's live gates: the CO₂ floor, the Greenwood cap, `rationed == 0`, and the peak LAI
   band.

**One prediction already written:** with no vegetative shedding, the dim seedling ends the cold
weeks near **0.150 mol C** (0.160 − 0.0096), not 0.077.

## 6. Decisions owed to the user

1. Run the measurement plan (§5)? It means a lab flow for A (and B), predictions, and runs across
   every biosphere scenario — a sizeable piece of lab work, no frozen change.
2. If yes: A only, or A and B (B is WHAT-IF, lab-only, and cannot become the reference uncited)?
