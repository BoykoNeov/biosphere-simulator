# Post-roadmap: giving the science assertions contract standing

**Status: COMPLETE (2026-08-09).** The adjudication of
`post-roadmap-acceptance-gate.md`'s finding 6, which that document deliberately left to
the user. Read it first — this doc assumes its census and does not repeat it.

**The decision (user, 2026-08-09):** the tree's science-side assertions get standing via a
**manifest field**, and the scope is **bands + liveness floors**, not bands alone.

---

## 1. The reframe that changed what was being decided

Finding 6 was written as "the two gates disagree". Read off the *pins* rather than the
summaries, they do not overlap at all — they return verdicts on **different scenarios**,
and the disagreement is an artefact of aggregating per-scenario verdicts into one
per-change verdict:

| | closure (`rationed == 0`) | liveness floors | literature bands |
|---|---|---|---|
| **the 6 chambers** | **binds** — the roster's six tightest margins | **present** | none possible (a 52 g DM/m² carbon-limited rig) |
| **`open_season`** (the only field-scale scenario) | **structurally empty** for carbon | none | **present** |

* (C)'s full form: closure passes on `perennial` (Euler), LAI fails on `open_season`.
* The canopy regulator: inert on the chambers, flips the canopy on `open_season`.
* Stem-only: both refuse — again on different scenarios.

⚠ This matters for one reason only, and it is the reason the acceptance-gate doc gave for
not adjudicating: **promoting a band on `open_season` cannot reverse a measured closure
refusal, because closure returns no carbon verdict there.** The co-adaptation shape needs
a verdict to overrule. The cell is empty.

**A correction made while deriving this.** A draft argued a manifest band is vacuous
because the golden already freezes peak LAI. It is not: `season_euler_state.json` is a
single **endpoint** (`n = 305`, 14 stocks). Peak LAI is mid-run and is frozen by nothing.
The goldens constrain trajectories only at their last step.

---

## 2. The inclusion rule (written before the survey, on purpose)

"Science-side assertion" meant three different things in the drafting text, and the third
family is already committed in the files this work reads. A gate is:

1. an assertion on a **physical quantity of a frozen-roster scenario, run as frozen**; and
2. against a bound whose value comes from **outside this repo** (⇒ `science_bands`), or a
   bound **tuned to our own calibration** (⇒ `liveness_floors`); and
3. **satisfied by movement toward the cited reference.**

Clause 3 is the discriminator that does the work, and it was written after a measured
counter-example. `test_chamber_scale.py`'s `assert ours_m3_per_m2 / BVAD_..._TOTAL > 20.0`
is outside-sourced and about a frozen scenario as frozen — yet a chamber resized *toward*
the flight spec **fails** it. That is a **characterization**, not a gate.

### What the rule excludes, each by a measurement

* **Margin-ratio and doc-staleness pins.**
  `assert peak_w / 14.4248 > 0.85, "the margin narrative is stale; re-measure it"` is a
  test whose stated purpose is detecting that a *doc sentence* drifted. Freezing it lets
  an unfreeze ceremony fail because prose moved. `0.80 < open_peak / 6.0 < 0.92` is
  two-sided on a margin: a change that *improves* the margin fails it.
* **Diagnosis pins about refused forms.**
  `test_the_primarys_form_takes_the_canopy_unphysical_on_either_table` asserts
  `peak > 15.0` for (C)'s form. It is a record of a form we do not ship — clause 1.
* **Calibration identities — and the file says so itself.**
  Most of `test_bvad_validation.py` asserts quantities the crew params were *calibrated
  to*; its own docstring says every quantity we set matches BVAD "**by construction**",
  with "one assertion that can genuinely fail". A param fitted to a reference, asserted
  against that reference, tests arithmetic. Only `test_rq_structural_prediction` survives.
* **The oracle pins.** The bar is decided (`CLAUDE.md`): the oracle is a **diagnostic,
  never a fit target**. Promoting an oracle-gap pin to a contract gate would reverse that
  ruling silently.

⚠ **The floors do not pass clause 2's outside-sourced half, and that is why they get their
own field.** `floor=0.05` and `> 0.9` were tuned to our own attractor — the decomposer
calibration record shows the plant floor moving `> 1.0` → `> 0.9` when the calibration
shrank the plant ~19 %. Freezing them under the same name as the bands would say "the
frozen tree passes a bound the frozen tree set". They are frozen anyway, because their
catch is real (stem-only's CO₂ attractor fell 3.4× **while staying stationary**, so the
*level* guard caught what a stationarity check passed) — but they are labelled as
guarding **continuity with the current calibration**, not physical plausibility.

**Two fields, not one.** This repo's recorded failure mode is two claims of different
strength merged under one name.

---

## 3. The survey

Method: the roster is the **two manifests'** scenario sets (7 + 13), never a list checked
against its own length. Loci found by sweeping the 21 files that reference a frozen
biosphere scenario constant and the 32 that reference a station one.

### `science_bands` — outside-sourced

| scenario | quantity | bound | source | locus |
|---|---|---|---|---|
| `open_season` | peak LAI | `5.0 < x < 8.0` | real wheat peaks ~5–8 | `test_senescence_form.py::test_frozen_open_season_canopy_is_physical` |
| `open_season` + 5 chambers | peak LAI | `< 6.0` | Van Keulen & Seligman 1987 via [A] p. 101 | `test_senescence_form.py` (**split owed**) |
| `open_season` | peak W excl. fibrous roots | `< 14.4248 t/ha` | Greenwood 1990 eqn (6) `a = 5.697` / `n_critical = 1.5` | `test_nitrogen_form.py` (**split owed**) |
| `crew_mission` | daily-effective molar RQ | `≈ 0.8814` | NASA BVAD Table 3-31 | `test_bvad_validation.py::test_rq_structural_prediction` |

### `liveness_floors` — self-sourced continuity guards

| scenario | quantity | bound | locus |
|---|---|---|---|
| `perennial_chamber`, `consumer_chamber` | annual peak leaf C | `floor = 0.05` | `test_decade_stability.py::test_decade_leaf_cycle_is_stationary` |
| `perennial_chamber` | leaf fixed-point tail | `> 0.9` | `::test_perennial_leaf_cycle_is_a_fixed_point` |
| `consumer_chamber` | consumer carbon | `floor = 5e-4` | `::test_decade_consumer_biomass_is_stationary_and_alive` |
| `perennial_chamber`, `consumer_chamber` | min `carbon_pool` | `floor = 0.05`, post-transient | `::test_decade_min_carbon_pool_stationary` |
| both long-horizons (~328 yr) | peak leaf / plant tail | `floor = 0.05`, `> 0.9` | `test_biosphere_stress.py` |
| `consumer_chamber` | standing biomass | `> 0.02` | `test_consumer.py` |
| `sealed_station` | Tier-1 node temperature | `floor = 100.0` K | `test_sealed_station_stability.py` |

### Measured absences — stated, not left to omission

* **11 of the 13 station scenarios carry no outside-sourced bound.** Established
  mechanically: no station run-test defines a module-level sourced constant at all
  (`^[A-Z][A-Z0-9_]{3,} *=` returns empty across all 12 files). `crew_mission` has one
  via `test_bvad_validation.py`; `sealed_station` has a floor.
* **`drift_summary` takes no entry.** It is a derived stability signature over
  `perennial` + `consumer`, both of which are themselves in the roster. It gets an
  explicit empty entry — an absent key and a deliberately-empty key are different, and
  the gate cannot tell them apart.

---

## 4. The mechanism

* A `science_gate` pytest marker, registered in `pyproject.toml`, carrying the scenario
  key and the field it belongs to.
* Enumeration by **`ast`** over `tests/*.py`, the way `_frozen_param_files()` enumerates
  files on disk. Rejected: a `pytest_collection_modifyitems` registry (goes red on a
  single-file run, because collection is partial) and a subprocess `--collect-only` (a
  second collection of the suite whose runtime was just cut 3.3×).
* The decorator form is **required and pinned as its own assertion**, so static-vs-collected
  drift is a stated convention rather than a silent hole.
* The manifest entry names **quantity + bound + source + locus**, not just a test id.

⚠ **Test granularity is too coarse, measured.** One committed function carries all three
families at once:

```python
assert 12.0 < peak_w < 13.0, peak_w                                  # characterization
assert peak_w < 14.4248, "open_season entered the stressed branch"   # THE GATE
assert peak_w / 14.4248 > 0.85, "the margin narrative is stale"      # staleness detector
```

So entangled tests are **split** so that a marked test carries only its gate. The two
splits owed are in the survey table above.

---

## 5. What shipped, and the two things measurement corrected

**Shipped.** `tests/science_gates.py` (the `ast` enumerator); a `science_gate` marker
registered in `pyproject.toml`; 10 markers across 5 test files; `science_bands` +
`liveness_floors` in **both** manifests, regenerated; 6 new gates in
`test_freeze_manifest.py` and 1 in `test_station_freeze_manifest.py`; the unfreeze
discipline in both reference docs gained a **"report the science gates"** step. Two tests
split. **No value, golden, param hash, or `src/` change** — `git diff src/` empty. Full
suite **2107 passed**, ruff + pyright clean.

**Teeth verified by mutation, not by a green bar.** With the BVAD marker removed,
`test_frozen_station_science_gates_are_complete` goes **red**. "The suite passed" and "the
gate has teeth" are unrelated statements, and every wrong version of the `loadgroup` design
was green.

⚠ **Correction 1 — the convention check counted itself.** The first version of
`test_science_gate_is_decorator_form_only` counted the textual occurrences of
`mark.science_gate` and required the count to equal the collected gates. It failed
**13 vs 10**: its own docstring and code literal were three of the occurrences. Matching
`@pytest.mark.science_gate` instead would have gone green *while losing the case worth
catching* — a `pytestmark = [...]` assignment has no `@`. Replaced with a structural
`ast` check where prose is not an attribute access and so cannot be a false positive.
**A self-referential text check is not a weaker version of the right check; it is a
different one that happens to be green.**

⚠ **Correction 2 — the acceptance-gate diagnosis's own pin caught this work, and it was
right to.** `test_the_plausibility_bands_that_exist_are_named_by_no_manifest` asserted
that no manifest names `test_senescence_form` / `test_nitrogen_form`. This work falsifies
it **by design**. It is **resolved, not corrected** — a true measurement of a contract
that has since changed — and is **replaced by its inverse** (both loci must now be
reachable from a manifest) rather than deleted or relaxed, on the option-(B) precedent:
*a pin guarding a mechanism you removed is decoration.*

Its neighbour, `test_a_manifest_scenario_entry_carries_no_plausibility_criterion`, still
**passes and is still true** — the standing lives in top-level fields keyed by scenario,
not in a column inside the entry. Its *docstring's conclusion* ("so the acceptance set is
all run-properties") is now false while its assertion is untouched. Annotated in place
rather than rewritten, because that gap — a sound assertion under a stale conclusion — is
the shape this repo keeps logging.

---

## 6. What this does NOT claim

* Not that `rationed == 0` is a bad gate. It is a numerical backstop and does that job
  correctly everywhere in the roster. The finding was always about the *duty it has been
  put to*.
* Not that the frozen science is thereby validated. A band the frozen tree passes is a
  floor under future change, not an endorsement of the present value.
* Not that the bands are strong. `open_season` sits **3.8 %** above the LAI lower bound
  and **12 %** below the Greenwood crossing. Freezing a tight margin is a deliberate
  choice, recorded here so it is not read as comfort.
