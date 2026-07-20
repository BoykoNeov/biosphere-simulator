# Post-roadmap: the day-neutral crop, validated against LINTUL3 spring wheat

**Status: COMPLETE (2026-07-20). All 7 steps landed; see "OUTCOME" at the foot. The
day-neutral crop ships as additive scenario data (`DAY_NEUTRAL_SCENARIO`), diagnosed
against an offline LINTUL3 spring-wheat oracle, with the warm-arrest demonstration and
sealed-chamber habitat-runnability pinned. NO frozen golden moved; `git diff src/simcore`
empty.**

This is the "second wheat" left open by scope-(B) ceremony 2
([`post-roadmap-oracle-match.md`](post-roadmap-oracle-match.md), "Open / deferred after
ceremony 2"): a warm-habitat crop with **no cold requirement**, so it flowers where the
frozen winter wheat would arrest. The user chose to **validate** it against the bundled
`lintul3_springwheat` oracle rather than ship it as bare authored content — waking the
Python laboratory for one scoped job (see
[`post-roadmap-rust-primary-pivot.md`](post-roadmap-rust-primary-pivot.md), Option A).

## What this exercise actually establishes — read before the findings

**Honest framing, decided up front (advisor).** Our DVS is the **same model family** as
LINTUL3 — both are linear growing-degree-day thermal time with a two-phase `TSUM1`/`TSUM2`
DVS. So a phenology "match" is **near-tautological**: it tests our *param choice*, not our
model. The genuine cross-model signal lives where the families differ — **canopy (LAI
dynamics)** and **biomass (LINTUL3's light-use-efficiency vs our FvCB)** — and there we
**cannot fit and will not** (ruling B: the oracle is a diagnostic, never a fit target).

So the deliverable is **not** "we validated our crop against an oracle." It is: *a
literature-cited, physically-sane day-neutral spring-wheat crop, runnable in the habitat,
with its gaps to LINTUL3 measured and documented.* The winter-wheat plan's false-positive
discipline applies doubly — a same-family DVS agreement must not be written up as
scientific validation of the model.

## The oracle — offline, bundled, license-clean (a strict upgrade on the winter one)

`tests/oracle/lintul3_runner.py` runs PCSE's **LINTUL3** model for spring wheat entirely
**offline**: its inputs (`lintul3_springwheat.{agro,crop,site,soil}` + the CABO `NL1`
weather) **ship with PCSE** as its own EUPL test data — no network, no unlicensed
`WOFOST_crop_parameters` cache (the winter-wheat oracle needed both). We commit **only the
output trajectory + provenance** (`spring_wheat_reference.json`) and the season weather
mapped to our schema (`spring_wheat_weather.json`), never parameter values.

**The measured oracle (emergence 1997-03-31 = day 0):**

| milestone | LINTUL3 spring wheat |
|---|---|
| DVS 0.5 | day 47 |
| DVS 1.0 (anthesis) | day 74 |
| DVS 2.0 (maturity) | day 135 |
| peak LAI | **5.73 at day 72 — just *before* anthesis** |
| final grain `WSO` | 812.9 g m⁻² (~8.1 t ha⁻¹) |

Vegetative phase 74 d, reproductive 61 d. **LAI peaking just before anthesis is the
physically-correct wheat pattern** — and it is exactly what our winter-wheat model failed
(ours peaked 34–51 d *after* anthesis; `post-roadmap-oracle-match.md`). So the LAI
comparison is a real cross-model diagnostic, not a formality.

**LINTUL3 is effectively day-neutral, confirmed from its own params** (read for
understanding, NOT copied): `IDSL=1` nominally makes it daylength-sensitive, but `DLO=8 h`
sits far below the actual 13–16.5 h daylength at NL 52°N in spring/summer, so the
photoperiod factor is pinned at 1 all season. DVS advances on **thermal time alone**
(`TBASE=0`, cap 50). No vernalization (a spring crop). Hence our matching crop drops
**both** vernalization and photoperiod — the user's original word "day-neutral" was right;
"photoperiod-only" (an earlier reading) was wrong for this target.

## Design

1. **The crop is day-neutral**: vernalization OFF, photoperiod OFF, thermal time only.
   Plumbed as an **additive, default-preserving** `SeasonScenario` flag (the
   N_LIMITED/WATER_BITING precedent) so `build_plants` builds `ThermalTimeAccumulation`
   with neither modifier and no `VernalizationAccumulation`. **All 7 frozen biosphere
   goldens must stay byte-identical** — the safety check that this is additive, not an
   unfreeze.
2. **Params are clean-room from primary literature**, cited on entry, **never** copied
   from `lintul3_springwheat.crop` (`TSUM1=800`/`TSUM2=1030` are visible there; using them
   is reverse-engineering PCSE). Spring wheat's `t_base`/`t_cap`/`tsum_anthesis`/
   `tsum_maturity` are sourced independently; if independent literature lands near
   LINTUL3's partition, that is the (modest) result — if not, that is the finding.
3. **The comparison is matched-DVS** (scope-A discipline: matched-DVS, not matched-day),
   on phenology + LAI shape; absolute biomass is reported but **not** a pass/fail criterion
   (LUE vs FvCB). Pinned in the `test_oracle_gap.py` idiom (a measured gap with its cause),
   never backfit.
4. **The crop stays a shippable artifact** — runnable in the sealed-chamber habitat form
   (conservation + determinism), not just a report. **Lamp-lit `LightingScenario` wiring
   is deferred, not done** (see Demonstrations): it is lamp-lit-habitat *product* work,
   which is Rust-first under the pivot, and the current lighting path drives the *cold*
   winter fixture, where a day-neutral crop would not show the warm-arrest contrast
   anyway. So the shippable form here is the sealed chamber, and the warm regime is an
   authored warm fixture — not the lighting scenario.

## Demonstrations (revised)

The user picked "both" (warm-arrest + lamp-photoperiod control) *before* the day-neutral
reframing. **Lamp-photoperiod control is now dead** — a day-neutral crop's flowering does
not respond to lamp daylength. Kept: the **warm-arrest contrast** — under a warm
constant-temp regime the frozen winter wheat is *permanently arrested* (`verfun` pinned at
0, never flowers) while the day-neutral crop develops normally. That proves the crop's
actual purpose. (Lamp control of PAR/energy still works via `lighting.py`; only *flowering*
control is gone.)

## Staging (living; outcomes appended as each lands)

1. **Oracle + weather fixtures — DONE.** `tests/oracle/lintul3_runner.py`,
   `spring_wheat_reference.json` (225 d), `spring_wheat_weather.json` (301 d from
   emergence). Offline, license-clean. Milestones table above.
2. **Param sourcing** — spring-wheat `t_base`/`t_cap`/`tsum_anthesis`/`tsum_maturity` from
   cited primary literature (check `sources/`: Penning de Vries 1989 [E], Soltani &
   Sinclair — a spring-wheat row). *Blocks the crop file.*
3. **Plumbing** — the `vernalization`/photoperiod-off `SeasonScenario` flag + crop-file
   selection in `build_plants`; frozen goldens byte-identical.
4. **The crop** — a cited `scenarios/` (or params) day-neutral spring-wheat file + the
   `SeasonScenario` constant.
5. **Diagnostic comparison + findings** — matched-DVS phenology, LAI shape, biomass note;
   pinned test; honest write-up per the framing above.
6. **Warm-arrest demo + habitat run** — warm fixture; the arrest contrast; the crop
   runnable in the sealed-chamber habitat form (lamp-lit `LightingScenario` wiring
   deferred — step 4's reason).
7. **Docs, gates, memory, commit** — this doc's outcomes, `CLAUDE.md`, the pivot decision,
   memory; `pytest -m "not slow"`/`ruff`/`pyright` green; frozen goldens untouched.

## Rust

Per the pivot (Option A): the **validation** is Python-only (the oracle is Python). The
crop itself is a candidate for a Rust mirror later (it is authored content, Rust-first
under the going-forward rule) — but the diagnostic exercise that earns it the "validated"
label lives in the laboratory, which stays Python. Deferred, stated not silent.

---

# OUTCOME (2026-07-20) — landed, and what the diagnostic actually says

**The framing held: this establishes "a literature-cited, sane day-neutral crop with its
gaps to LINTUL3 documented," not "we validated our model against an oracle."** The
phenology agreement is same-family (near-tautological); the value is the canopy magnitude
agreement and the cross-oracle corroboration of ceremony 2.

## What shipped

* `tests/oracle/lintul3_runner.py` + `spring_wheat_reference.json` (225 d) +
  `spring_wheat_weather.json` (301 d) — the offline, license-clean oracle + weather.
* `SeasonScenario.vernalization` / `.photoperiod` flags (both default `True`) read by
  `build_plants`; `DAY_NEUTRAL_SCENARIO` (both off). **Additive + default-preserving**:
  all 7 frozen biosphere goldens + `test_freeze_manifest` byte-identical (92 tests green).
* `tests/test_oracle_gap_spring_wheat.py` (6 pins) — the diagnostic.
* `tests/test_day_neutral_warm_habitat.py` (4 pins) — the warm-arrest demo + sealed-chamber
  habitat-runnability (Euler + RK4, `rationed == 0`, deterministic).
* `tests/oracle/test_lintul3_fixture.py` (4) + `test_lintul3_regeneration.py` (1, oracle) —
  fixture hygiene.

## The measured diagnostic (emergence = day 0, aligned day-for-day)

| metric | ours (day-neutral) | LINTUL3 | read |
|---|---|---|---|
| DVS 1.0 (anthesis) | day ~94 | day 74 | **veg-heavy** (+20 d) |
| DVS 2.0 (maturity) | day ~135 | day 135 | coincides — *two errors cancelling* |
| reproductive phase | ~41 d | 61 d | short grain fill |
| **peak LAI** | **5.60** | **5.73** | **1.02× — both realistic wheat canopies** |
| peak LAI day | ~107 (after anthesis) | 72 (before) | timing slip (real cross-model) |
| root fraction @ DVS 0.5 | 0.31 | 0.55 | LINTUL3 front-loads roots |

## The three findings

1. **The partition finding corroborates ceremony 2 across a SECOND, independent oracle.**
   Our `tsum` is vegetative-heavy (1100/750); *both* WOFOST (ceremony 2) and LINTUL3
   (800/1030) are reproductive-heavy. Two oracles of *different model families* agree
   wheat's grain fill is longer than our winter-wheat `tsum` gives — new, independent
   evidence for ceremony 2's "cultivar variation, not our error" (ruling B: recorded, not
   fitted). The maturity coincidence at day 135 is the same **two-errors-cancelling**
   trap ceremony 2 flagged (our total thermal time ~1855 °C·day ≈ LINTUL3's ~1830).

2. **Both models produce a realistic wheat canopy (~5.6–5.7 peak LAI, 1.02×)** — the
   day-neutral crop's canopy closes to the oracle's height with no canopy science and no
   param fit. Read carefully: two *independently-parameterized* canopy models both landing
   near a realistic wheat peak is "both are sane," **not** cross-validation (the same
   caution as the phenology tautology, one level over). Still worth recording — the canopy
   closes rather than collapsing — but not a positive *match* result.

3. **The genuine (non-tautological) gap is canopy TIMING**: our LAI peaks ~13 d *after*
   anthesis; LINTUL3 peaks 2 d *before*. This is where the FvCB-allocation family and the
   LUE family actually differ (same direction as the winter-wheat oracle), so it is a real
   cross-model signal, not a same-family artefact. Plus a partition-model difference:
   LINTUL3 front-loads roots (0.55 vs our 0.31 at DVS 0.5), converging by anthesis.

## The purpose demonstrated

Under a warm constant-temperature habitat (20 °C): the frozen **winter wheat is
permanently arrested** (max DVS 0.0 — `verfun` pinned at 0, no thermal time ever accrues,
never flowers), while the **day-neutral crop develops normally** (anthesis ~day 55,
maturity ~day 93). That is the crop's reason to exist, and it runs in the sealed-chamber
habitat form under both integrators with conservation + determinism (no over-draw — the
un-enlarged sealed CO₂ pool suffices, unlike the ~5× consumer chamber).

## Clean-room note

No clean primary spring-wheat `TSUM` exists on the shelf (`[E]` Penning de Vries has
winter cultivars only; `[C]` Soltani & Sinclair uses a biological-day formalism;
connor/hoogenboom/pereira/Teh carry none). Reusing our own cited winter-wheat `tsum`
(rather than copying LINTUL3's `TSUM1=800`/`TSUM2=1030`, which would reverse-engineer
PCSE) is the ruling-B-clean choice — independently justified params, gap recorded not fit.
The day-neutral crop is therefore winter-wheat physiology with the cold/daylength gates
removed (ceremony 2: "vernalization is optional by design"), **not** a new param file.

---

# FOLLOW-UP (2026-07-21) — the lamp-lit `LightingScenario` wiring, landed **Rust-first**

Step 4 / "Demonstrations" deferred the lamp-lit habitat wiring as *"lamp-lit-habitat
**product** work, which is Rust-first under the pivot."* That deferred piece is now done —
**and the port was the load-bearing decision, not the design.** The design was Python-shaped
in my head; an advisor catch pointed out the plan had already decided the port for this exact
item (the pivot's dividing line: *authored content, no golden moved → Rust*), and that
"validation is Python" (true, done) does not govern where the **product wiring** lives.

**What shipped (Rust only; no Python touched — `git diff src/` empty):**

* `SeasonScenario` gained `vernalization` / `photoperiod` **bool** fields (both default
  `true`), and `build_plants` now gates the two modifiers on them — the additive,
  default-preserving mirror of the Python scope-(B)-inc-1 change. This is the **port
  converging** on a capability Python already had (not the port claiming reference
  authority): every existing scenario spreads `..DEFAULT_SCENARIO`, so all cross-port
  trajectories stay bit-identical. Rust could not previously express a gate-removed crop
  (`build_plants` hard-wired `Some(p.vern)` / `Some(p.photoperiod)`).
* `LightingScenario` gained an optional `habitat_temp_c: Option<f64>` (default `None` →
  frozen `lighting_scenario()` byte-identical). When `Some`, `lighting_bio_resolver`
  overrides `TEMP_VAR` with a constant — **one line beside the existing lamp PAR/daylength
  overrides, in the non-frozen `crates/station/src/lighting.rs`** (the embedded cold
  `weather_facts()` table is untouched; no synthetic-weather facility needed — the initial
  "warm weather isn't expressible in Rust" read was an overstatement the advisor corrected).
* `day_neutral_lighting_scenario()` — a **warm (20 °C), lamp-lit, day-neutral** sealed
  habitat (`litter_carbon0 = 3`, both gates off), battery sized well-fed for the 120-day
  development horizon (`battery0 = 3e9` J; the lamp draws `1.152e7` J/day, ~46 % drawdown).
* `tests/day_neutral_lighting.rs` (8 pins) — **authored ≠ validated**: conservation +
  determinism, **no golden**. The payload is the **contrast** — under the *identical* warm +
  lamp habitat the day-neutral crop reaches maturity (DVS ≥ 2) while the frozen winter wheat
  is pinned at DVS 0 / `thermal_time == 0` (arrest, `verfun ≡ 0`); plus ENERGY closes over
  the Power ledger, the biosphere internal water loop stays closed, `rationed == 0`,
  `events == ()`, lamp-on-grows / lamp-off-declines, bit-identical re-run.

**The dead demo stayed dead.** The lamp is framed as carrying ENERGY/PAR that drives fixation
(real) — never as photoperiod/flowering control (a day-neutral crop ignores daylength).

**Honest residual (a documented, not fixed, gap):** the temperature override warms `TEMP_VAR`
but leaves VPD / net-radiation weather-driven — a partial controlled environment, exactly the
"fully controlled-environment chamber is a deferred refinement" the lighting docstring already
names. Acceptable for authored content; noted on `LightingScenario.habitat_temp_c`.

**Gates:** the full `domains` + `station` suites pass; `cargo clippy --all-targets` clean;
frozen `lighting_state.json` byte-identical + its cross-port tier green; `git diff src/`
empty. This is the **first application of the pivot's "new content is Rust-first, no Python
mirror owed"** rule to a concrete deliverable.
