# Post-roadmap: potato — the first *second species*, validated against an offline WOFOST oracle

**Status: STAGE 1 COMPLETE (2026-08-11). Steps 1–6 landed; see "OUTCOME" at the foot.
Stage 2 (the Rust habitat mirror) is DEFERRED by the user's own "both, staged" decision —
stated, not dropped. NO frozen golden or manifest moved; 2146 tests green.**

User decision: "both, staged" + "potato only" — the validated Python crop first, the Rust
habitat mirror after.

Everything the biosphere has ever grown is **one crop**: winter wheat. The "day-neutral
crop" ([`post-roadmap-day-neutral-crop.md`](post-roadmap-day-neutral-crop.md)) is *not* a
second species — its own OUTCOME says so: *"winter-wheat physiology with the cold/daylength
gates removed … **not** a new param file."* So the tree has never held two parameterizations
at once, and `_carbon_context()` / `build_plants()` call every loader **argument-free**.

This is the first genuine second species.

## Why potato, and why now — the two facts that decided it

### 1. A validated second species became possible (the branch-deciding check)

The last crop's oracle was constrained: the winter-wheat oracle needed the **unlicensed**
`WOFOST_crop_parameters` YAML repo **plus network**; only `lintul3_springwheat` shipped
offline. That made "more crops" look like authored-only work under the pivot.

**That reading was stale.** `pcse/tests/test_data/pcse_dump.sql` — PCSE's own bundled test
database, loaded automatically at `import pcse` — ships **fully offline**:

| what | coverage |
|---|---|
| crops with complete WOFOST parameter sets | **6**: winter wheat (1), grain maize (2), spring barley (3), **potato (7)**, winter rapeseed (10), sunflower (11) |
| agromanagement (crop calendars) | all 6, grid 31031, year 2000 |
| weather | grid 31031 (**lat 37.64, lon −6.09, alt 47 m** — Andalusia), 1999-12-01 → 2001-11-30 |
| site + soil | present |
| **pre-computed reference trajectories** | `wofost_unittest_benchmarks`: daily DVS/LAI/TAGP/TWSO/TWLV/TWST/TWRT/SM/TRA/RD for all 6 crops, in **both** `pp` and `wlp` mode |

`pcse.start_wofost(grid=31031, crop=7, year=2000, mode='pp')` is a **first-class supported
API** and runs offline. Verified 2026-08-11:

| milestone | WOFOST potato (pp) |
|---|---|
| emergence (day 0) | 2000-02-20 |
| DVS 1.0 (anthesis) | 2000-04-04 = **day 44** |
| DVS 2.0 (maturity) | 2000-05-26 = **day 96** |
| peak LAI | **8.88** |
| final tuber `TWSO` | **7249.8 kg ha⁻¹** (~7.2 t ha⁻¹) |

Our own run reproduces the shipped `wofost_unittest_benchmarks` row **exactly**
(max LAI 8.8847…, max TWSO 7249.78…), so the benchmark table is an independent
cross-check on the runner rather than a second oracle. **Both are recorded.**

⚠ **Same licence discipline, no relaxation.** PCSE is EUPL → running it is *mere use* and
its **output is facts**. We commit **only the output trajectory + provenance + the weather
mapped to our schema — never a parameter value** (`docs/reuse-and-licenses.md`; the
`pcse-oracle-licensing` rule). The potato's own params are sourced **independently** from
primary literature below; copying `crop_parameter_value` rows out of the dump would be
reverse-engineering PCSE and is **refused**.

⚠ **The oracle is a diagnostic, never a fit target** (ruling B, from
[`post-roadmap-oracle-match.md`](post-roadmap-oracle-match.md)). No potato value is ever
moved to close a gap to WOFOST. A gap is **measured and recorded**.

### 2. The shelf actually serves potato — and serves it *better than it serves wheat*

Audited per-parameter (2026-08-11) against `sources/`, **before** authoring anything —
because "a species" here is only the subset our param files can express.

| our param file | frozen wheat's citation status | potato on the shelf? |
|---|---|---|
| `phenology` — `tsum_anthesis`, `tsum_maturity` | **CITED** [E] Tables 12 / 15 | ✅ **[E] Tables 12 + 15**, two cultivars (Mara late, Favorita early), ref. T 18 °C |
| `phenology` — daylength gate | cited (long-day wheat) | ✅ **[E] Table 12 marks potato's daylength column "--" = not relevant** → day-neutral *by the source's own marking* |
| `allocation` — the partition table | **UNCITED** ("literature-typical") | ✅ **[E] Table 18**, cited, DVS-keyed, per-cultivar (CALVT/CASTT/CASST) |
| `canopy` — specific leaf area | **UNCITED** | ✅ **[E] Table 19** (specific leaf *weight*; SLA = 1/SLW — a documented derivation) |
| `ci_ratio` (scenario, not a param file) | scenario default 0.7 | ✅ **[E] Table 22** — potato 0.67–0.69 |
| `respiration` — maintenance etc. | **UNCITED** | ❌ **[E] Table 8 has no potato row** (Barley, Cotton, Faba bean, Field bean, Maize, Millet, Rice, Sorghum, Sunflower, Wheat only) |
| `photosynthesis` — all 12 FvCB params | **UNCITED**, tagged *"literature-typical C3"* | ❌ [E] predates FvCB-in-crop-models (it uses PLMX, a max-leaf-rate formalism) |
| `senescence`, `transpiration` | **UNCITED** | ❌ |
| `nitrogen` | **CITED — but to a *generic C3* curve** (Greenwood 1990), not to wheat | ✅ the same C3 curve applies unchanged |

**Two conclusions fall out of that table, and both must survive into the write-up.**

**(a) The FvCB gap is NOT a gap this crop opens.** The advisor flagged the risk of
inventing `Vcmax` for a new species to make the set look complete. That risk does not
arise here: all twelve FvCB params are `TODO(cite)` placeholders explicitly tagged
*"literature-typical C3"* — they were **never wheat-specific**. Potato is a C3 plant.
Sharing them is applying the same generic C3 placeholder set to a second C3 crop, which
is *exactly as justified as its current use*, and requires no invention. **Stated as a
design choice in the crop file's header, never glossed.**

**(b) A cited potato would be BETTER cited than the frozen wheat** on the partition table
and specific leaf area — the two rows where wheat is still a placeholder, and where
[`post-roadmap-stem-reserves.md`](post-roadmap-stem-reserves.md) already named the uncited
partition table *"the real successor."* That is a genuine result, and it lands **without
touching the freeze**: the wheat files do not move.

⚠ **[E]'s text layer is garbled exactly on the table digits** — the round-6 hazard
`phenology.yaml`'s header already records. Every potato value **must** be transcribed from
**rendered page images**, never from `pdftotext`. Non-negotiable; the wheat `tsum` values
were bound this way and the same discipline applies.

### Why not the other five

* **Grain maize — REFUSED, not deprioritized.** C4. Our photosynthesis is C3-only, so
  maize needs *new science* (a C4 pathway), not new parameters. Out of scope by
  construction.
* **Spring barley** — in the oracle and in [E], but a cool-season grain-filling cousin of
  wheat: agreement would be near-automatic, the same *same-family tautology* the
  day-neutral write-up flagged. Breadth, not evidence.
* **Sunflower / winter rapeseed** — real contrast (warm-season oilseed; a second
  vernalizing crop), but thinner [E] coverage, so more rows fall back to placeholders.

Recorded so a later session does not re-litigate the shortlist.

## The seam — how the tree comes to hold two crops at all

Nearly free, with an exact precedent: the additive, default-preserving
`SeasonScenario.vernalization` / `.photoperiod` bools.

1. **Every biosphere loader already takes `path: str | Path = <frozen default>`.** The
   only reason a second crop is inexpressible is that `_carbon_context()` and
   `build_plants()` call them **argument-free**.
2. Add a **crop param-set selector** to `SeasonScenario`, defaulting to `None` → today's
   frozen files, byte-for-byte.
3. Consume it in `_carbon_context` + `build_plants`.
4. **The safety check that this is additive, not an unfreeze**: all **7** biosphere
   goldens byte-identical + `tests/test_freeze_manifest.py` green + `tests/crossport`
   green.

**Explicitly NOT done: registering biosphere flow types in the authoring platform.** The
authoring registry's *param pack* (`params: {pack: …}` → an alternative file read by the
*frozen* loader, so it passes the frozen guards) is conceptually the same idea, and this
seam reuses the **concept**. But `FLOW_TYPES` is named by the authoring manifest, so
adding biosphere flows to it is an **authoring-platform unfreeze** — a far bigger ask than
this task, and unnecessary: the biosphere builders already accept a scenario.

### The freeze-gate directory choice — a DELIBERATE, WRITTEN decision

`tests/test_freeze_manifest.py::test_frozen_param_files_are_complete` does
`PARAMS_DIR.glob("*.yaml")` — **non-recursive**. Therefore:

* `params/potato_phenology.yaml` → **trips the gate** (an unfreeze ceremony).
* `params/crops/potato/phenology.yaml` → **does not**.

**Decision: the subdirectory, `src/domains/biosphere/params/crops/potato/`.** Reasoning,
recorded rather than left as a quiet consequence of file placement:

* The gate's own stated job is catching *"a new file wired into no committed golden."* A
  new crop is **deliberately** wired into no golden — "authored ≠ validated" is a standing
  project invariant. The gate is not blind to it by accident; the file is genuinely not a
  frozen-reference file.
* The frozen surface is *the winter-wheat reference*, and `params/*.yaml` **is** that
  surface. Adding a second species to it would enlarge the frozen reference to mean
  "every crop", which is not what `docs/biosphere-reference.md` freezes.
* The alternative — list the potato files in `param_files` via a full unfreeze ceremony —
  is **also legitimate**; it just pays a ceremony to freeze something we are simultaneously
  declaring unvalidated. Rejected as incoherent, not as expensive.

**A gate that a layout choice can route around is a finding in itself**, and it is
recorded here and in the log so the next person meets it as a decision, not a trap.

## The four traps — designed for, not discovered

Flagged by the advisor before any code was written; each gets a pin.

1. **`carbon_fraction` is duplicated** in `canopy.yaml` **and** `nitrogen.yaml`, and the
   loader comment says they MUST be equal — divergence "models a silently inconsistent
   plant". A crop set must keep that per-crop. **Pin it across *every* crop set that
   exists, not just the default.**
2. **`annual_reset` raises `ValueError` when `grain < seedling_total`.** Potato's partition
   table is different, so the perennial driver can die at the first year boundary.
   Predictable ⇒ pinned, and if it fires it is a **scenario-sizing** result to record, not
   a bug to patch by moving a cited value.
3. **Sealed-chamber CO₂ over-draw.** The chamber-scale finding is that the sealed jar holds
   ~2 days of *one* crop's carbon; `run_scenario` now **raises** `RationedError`, and
   `ReversedFlowError` guards direction. Which scenario the potato runs in is a
   **deliberate sizing decision**, decided up front, not a surprise at test time.
4. **`t_base ≠ 0` sharpens the tsum derivation's caveat.** `phenology.yaml` derives
   `TSUM = (Tref − t_base)/r` and justifies it as *base-temp-free* because [E]'s rate
   constant carries no base temperature. Potato's reference temperature is **18 °C**, not
   wheat's 20 °C, and potato's base temperature is not 0. The recorded caveat — *"the two
   formalisms agree only NEAR the reference"* — must be **restated with potato's own
   numbers**, never copied from the wheat sentence.

## Staging (living; outcomes appended as each lands)

1. **The seam** — the crop param-set selector; 7 goldens byte-identical, freeze manifest
   green, `git diff` on frozen params empty.
2. **The oracle fixture** — an offline `wofost_potato_runner.py` (the `lintul3_runner.py`
   idiom) + `potato_reference.json` (trajectory + provenance) + `potato_weather.json`
   (grid-31031 daily facts mapped to our `{TEMP, IRRAD, VAP}` schema). Cross-checked
   against `wofost_unittest_benchmarks`.
3. **Param sourcing** — [E] Tables 12, 15, 18, 19, 22 (+13 for the temperature response)
   read **off page images**; the derivations (SLW→SLA, CALVT/CASTT/CASST→fl/fs/fr/fo)
   written out, not asserted. *Blocks the crop files.*
4. **The crop** — `params/crops/potato/{phenology,allocation,canopy}.yaml` cited + the
   shared-placeholder reuse stated; `POTATO_SCENARIO` (day-neutral: both gates off, per
   [E] Table 12's own "--").
5. **Diagnostic comparison** — matched-DVS phenology, LAI shape, tuber-biomass note,
   pinned in the `test_oracle_gap*.py` idiom as measured gaps with causes. Never backfit.
6. **Habitat runnability + the four trap pins** — conservation, determinism,
   `rationed == 0`, both integrators.
7. **Docs, gates, memory, commit** — this doc's outcomes, the log row, one line in
   `CLAUDE.md`'s index, memory; `ruff`/`pyright`/`pytest` green.

**Stage 2 (deferred, stated not silent): the Rust habitat mirror.** Under the pivot,
authored habitat content is Rust-first — but the *validation* here is Python (the oracle is
Python and never portable). The potato as **habitat content** is a Rust deliverable and is
explicitly deferred to a later session, exactly as the day-neutral crop's lamp-lit wiring
was.

## Sources

* **[E]** Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M. & Bakema, A. (1989),
  *Simulation of Ecophysiological Processes of Growth in Several Annual Crops*, Simulation
  Monographs 29, PUDOC/IRRI. — Tables 8, 12, 13, 15, 18, 19, 22. Already first-hand for the
  wheat `tsum` values; **page images, not the text layer.**
* **[A]** Greenwood, D.J. et al. (1990) — the generic **C3** critical-N dilution curve
  already cited in `nitrogen.yaml`; applies to potato unchanged.
* **The oracle (facts only, never params)**: WOFOST 7.2 potential production via
  `pcse.start_wofost(grid=31031, crop=7, year=2000, mode='pp')`, PCSE 6.0.13, bundled
  demo DB.

---

# OUTCOME (2026-08-11) — stage 1 landed, and what the diagnostic actually says

**The framing held.** This establishes *a literature-cited, physically-sane potato,
runnable as habitat content, with its gaps to WOFOST measured* — **not** "we validated our
model against an oracle". The single most valuable thing it produced is a **disagreement
between two independent sources**, which no amount of calibration would have surfaced.

## What shipped

* **The seam** (commit 1): `SeasonScenario.crop` + `loader.crop_param_set` — a crop is the
  **eight plant-side param files**, resolved as a set, with `overridden`/`shared`
  partitioning the vocabulary so a reuse claim is *testable*. `tests/test_crop_param_set.py`
  (8 pins). Additive and default-preserving, proven: 7 goldens + both manifests
  byte-identical.
* **The oracle**: `tests/oracle/wofost_potato_runner.py` + `potato_reference.json` (97 d) +
  `potato_weather.json` (201 d). Offline, licence-clean, plus a **runner cross-check**
  against PCSE's own shipped `wofost_unittest_benchmarks` (max |Δ| ≈ 1e-3, i.e. SQL text
  round-trip noise). `tests/oracle/test_potato_regeneration.py` (2, oracle-marked).
* **The crop**: `params/crops/potato/{phenology,allocation,canopy}.yaml`, cited to [E] off
  page images; `POTATO_SCENARIO`.
* **The diagnostic**: `tests/test_potato_crop.py` (13 pins).

## The measured diagnostic (emergence = day 0, aligned day-for-day)

| metric | ours | WOFOST potato | read |
|---|---|---|---|
| DVS 1.0 (anthesis) | day **33** | day 44 | veg ~25 % fast |
| DVS 2.0 (maturity) | day **108** | day 96 | fill long (75 d vs 52 d) |
| **first tuber carbon** | **day 7** (DVS 0.19) | **day 46** (DVS 1.03) | **the headline** |
| peak LAI | **3.18** @ day 34 | **8.88** @ day 51 | **2.79× low** |
| tuber @ day 96 | ~14 260 kg ha⁻¹ | 7 250 kg ha⁻¹ | **1.97× high** |
| root fraction @ DVS 0.5 | 0.360 | 0.200 | we are root-heavy |

## The findings

1. **THE HEADLINE — two sources disagree *qualitatively* about the same organ of the same
   crop.** [E] Table 18's van Heemst curve starts filling the tuber essentially at
   emergence (a positive share the moment development passes 0.15); WOFOST's potato holds
   tuber weight at **exactly 0** until flowering. Both are "potato", both are cited, and
   they differ by ~39 days of a 96-day season. This is **cultivar/parameterization
   variation recorded, not a defect calibrated away** (ruling B) — the same shape as the
   winter-wheat `tsum` finding, but sharper because it is qualitative rather than a
   magnitude.

2. **One cause, two symptoms — so "fix the canopy" and "fix the yield" are one question.**
   The starved canopy (2.79× low) and the over-filled tuber (1.97× high) are *both*
   downstream of finding 1: assimilate diverted into the tuber from day 7 is assimilate the
   leaves never got. Recorded explicitly because treating them as two independent defects
   would invite two independent (and wrong) calibrations.

3. **A canopy agreement is not a property of our canopy.** The day-neutral crop matched its
   oracle's peak LAI within 2 % and the write-up was careful to call that "both are sane",
   not cross-validation. This crop is 2.79× low against a different oracle. **Both facts
   survive together only because neither was fitted** — which is the strongest evidence
   yet that ruling B is doing real work rather than being a slogan.

4. **The FvCB "gap" was a false alarm, and finding that out was worth the audit.** The
   advisor flagged the risk of inventing `Vcmax` for a new species. The per-parameter audit
   showed all twelve FvCB params are `TODO(cite)` placeholders tagged *"literature-typical
   C3"* — they were **never wheat-specific**. Sharing them with a second C3 crop is exactly
   as justified as their current use. The general lesson: **before deciding a shared
   parameter is a compromise, check whether it was ever specific to anything.**

5. **A new crop can be BETTER cited than the frozen reference.** Potato's partition table
   and specific leaf area are cited first-hand to [E]; wheat's are still `TODO(cite)`, and
   [`post-roadmap-stem-reserves.md`](post-roadmap-stem-reserves.md) named the uncited
   partition table *"the real successor"* — the gap blocking a piece of science. Note what
   this does **not** license: backfilling wheat's table from the same source is an
   **unfreeze** with its own ceremony, deliberately not done here.

6. **Where reuse is weak, the DIRECTION of the error is written down.** Potato's extinction
   coefficient is the reference crop's own placeholder, and a broad-leaved planophile canopy
   plausibly extinguishes light *more* strongly than an erectophile cereal (~0.8–1.0 vs
   0.6). So it is not merely uncited — it is probably biased **low**, and the next session
   inherits the finding rather than just the number.

7. **All four designed-for traps behaved, and none was a discovery.** (1) `carbon_fraction`
   agreement now spans a *crop boundary* (potato overrides canopy but not nitrogen) and is
   pinned across every crop set. (2) The seed-bank guard never fired — potato's tuber is
   ~430× the seedling. (3) The sealed chamber does **not** over-draw on the larger crop:
   FvCB's Ci-shutoff self-limits before the arbitration backstop is needed, under **both**
   integrators. (4) The `t_base ≠ 0` caveat was restated with potato's own numbers and came
   out **sharper and reversed in sign** — our cap sits *at* the optimum, so above 18 °C we
   accumulate development where [E]'s response is declining. That is a live warm-window
   over-run on the Andalusian season, not a cold-window softness.

## Honest residuals (documented, not fixed)

* **van Heemst (1986) was not opened.** [E] attributes every potato row to it; [E] is a
  printed primary presenting the values in its own tables, but the LOCUS check that caught
  Dunn 2011 has **not** been run on van Heemst. Dated residual risk, same standing as the
  reference crop's [E]-sourced `tsum`.
* **[E] Table 19's potato row carries no per-row reference**, which its footnote makes a
  CABO *personal communication* — weaker than a published measurement.
* **The partition derivation approximates.** [E] interpolates then multiplies; we multiply
  at the knots then interpolate. Using the union of all three curves' own knots bounds the
  deviation, and conservation is exact by linearity, but it is an approximation and is
  stated in the file.
* **`ci_ratio` was left at the frozen 0.7.** [E] Table 22 offers potato 0.67–0.69 — close,
  and available — but it is scenario data tied to the sealed chamber's sizing. Noted, not
  taken.
* **The model-form gap above 18 °C** (finding 7.4) is not closed by any value.
* **The oracle's cultivar is recorded but not fully separable.** The fixture now pins
  `variety_no = 2830` (the demo DB's `crop_calendar` row for this run), so a reader can
  see *which* potato WOFOST grew. But variety 2830's parameter values are PCSE's and are
  deliberately never read (clean-room), so we know which cultivar without knowing how its
  partition curve is shaped. Finding 1 therefore stands as **"two cited
  parameterizations disagree"** and does **not** claim the disagreement is purely
  structural rather than partly cultivar. Stated, not papered over.
* **A lint autofix silently disabled the runner cross-check, and only the `assert
  deltas` guard caught it.** Rewriting `variable not in reference.keys()` to `variable
  not in reference` looks equivalent, but `sqlite3.Row` is a **sequence**, so `in` tests
  its VALUES — every variable was skipped and `benchmark_deltas` returned `{}`, which
  reads as "the cross-check passed" while checking nothing. Ruff's SIM118 is wrong on
  `sqlite3.Row`; the call now carries a `noqa` and the reason. **A check that can pass
  vacuously needs a test that it did something** — that assertion was written before the
  bug existed and is the only reason this was not shipped green-and-empty.

## Stage 2 — deferred, and what it is

The **Rust habitat mirror**. Under the pivot, authored habitat content is Rust-first, but
the *validation* lives in the Python laboratory (the oracle is Python and never portable).
The day-neutral crop's lamp-lit wiring was deferred the same way and landed later; this
follows that precedent. Nothing in stage 1 owes Rust anything — no golden moved, no
cross-port tier touched.
