# The native-port reference — Phase 7, P7.6 (the cross-port tolerance contract)

Phase 7 ports the **frozen** multi-domain station — `simcore` + the four Phase-5 siblings
(power / thermal / eclss / crew) + the biosphere + the station seams — to a native **Rust**
core (`rust/` workspace). This file is the **cross-port tolerance contract**: the mirror of
the freeze contracts ([`docs/station-reference.md`](station-reference.md),
[`docs/biosphere-reference.md`](biosphere-reference.md)), one language boundary out. It
records **how the port is judged faithful** — the per-scenario tier assignment, the measured
Tier-2 bands and their provenance, the op-for-op libm audit, and the discovered-discrepancy
protocol.

Like the freeze contracts this is **boundary-side docs only**: `git diff src/` stays
**empty** — the Python reference is untouched; the Rust port lives under `rust/`.
The machine-readable companion is **`rust/data/tiers.json`** (the per-golden tier +
band table — the **authoritative** source; this doc's prose must not contradict it). The
plan of record is [`docs/plans/phase-7-native-core.md`](plans/phase-7-native-core.md).

> ## ⚠ 2026-08-25 — UNFREEZE: the contract's numbers moved, and the reference now enforces them
>
> **What changed: the file's home and who reads it. No band, floor or tier moved.**
>
> Until this date the table lived at `tests/crossport/tiers.json` and was read by **no program
> in `rust/`** — the whole tolerance contract was enforced by the Python checker alone, inside
> the tree the reference flip is deleting. Worse than stranded data: the reference's own golden
> comparison (`domains::goldens::compare`) carries **no numeric tolerance at all**. It is
> byte-exact for pure-arithmetic goldens and on Windows, and otherwise falls back to a
> *structural* walk that asserts a hex-float leaf parses finite and says nothing about its
> value. So on the `crossport` CI job — glibc Rust against UCRT-generated goldens, the repo's
> only genuine cross-libm measurement — the banded assertion existed only in Python.
>
> Under the flip's posture that is a hole in the **reference**, so:
>
> * the table moved to **`rust/data/tiers.json`**, beside the goldens it classifies;
> * `domains::tiers` reads it and implements the comparison — Tier 1 bit-exact on parsed f64,
>   Tier 2 `max |c−r| / max(|r|, floor) ≤ band`;
> * `domains/tests/tier_contract.rs` and `station/tests/tier_contract.rs` are the gates: the
>   contract classifies exactly the 20 frozen goldens, every row is internally consistent, the
>   Tier-1 set is the four transcendental-free scenarios, and every classified run sits inside
>   its own measured band;
> * the Python checker follows the file to its new path and stays green until it is retired.
>
> ⚠ **What did NOT move, and was recorded as outstanding — LANDED 2026-08-27, see the block
> above; this paragraph is kept because the deferral is part of the record:** the four
> `band > measured sensitivity` re-derivations. Those perturb a transcendental by one ULP and propagate it
> through the engine, and the tool that does it (`tests/crossport/measure_tier2_bands.py`)
> substitutes a `math` reference **inside the Python domain modules** and runs the **Python
> engine** — so the instrument is built out of the tree being deleted and dies with its
> subject. Porting it means re-measuring against the Rust engine, not translating. See
> `docs/plans/post-roadmap-reference-flip.md` §5ac.
>
> Ceremony: advisor-reviewed, gates written before the data moved, mutation-controlled
> (a golden nudged 1e-11 fails and 1e-13 passes against a 1e-12 band), documented here.

> ## ⚠ 2026-08-27 — UNFREEZE: the reference now measures its own band basis
>
> **What changed: which port justifies the bands. No band, floor or tier moved — the
> re-measurement moved nothing, which was the point.**
>
> The 2026-08-25 unfreeze moved the *numbers* and the *comparison* into the reference and left
> one half named and outstanding: the four `band > measured sensitivity` re-derivations, whose
> instrument (`tests/crossport/measure_tier2_bands.py`) substitutes a `math` reference inside
> the Python domain modules and runs the Python engine. Until today the committed bands were
> **asserted** by the reference and **justified** only by a program built out of the tree being
> deleted. They are now both.
>
> * **`domains::ulp_probe`** carries the perturbation, the metric and the four sibling/biosphere
>   measurements; **`station::ulp_probe`** composes the same seams onto the two coupled runs.
> * **`domains/tests/tier_sensitivity.rs`** (10 keys) and **`station/tests/tier_sensitivity.rs`**
>   (6 keys) are the gates — the whole Tier-2 roster.
>
> **The four seams, and what each one perturbs.** Three need no engine change:
>
> | run | the Python shim | the reference's seam |
> |---|---|---|
> | power ×2 | `domains.power.system.math.sin` | a mirrored schedule nudging the `sin` result, and the load derived from it |
> | thermal | `domains.thermal.flows.radiated_power` | a mirrored `RadiatorReject` swapped into the registry |
> | biosphere / greenhouse | `canopy.math.exp` + `photosynthesis.math.exp` | the `par` forcing |
> | station energy | both of the above | the same two, perturbed **separately** and the worse taken |
>
> ⚠ The `par` seam is **exact, not an approximation**, and it was checked rather than argued:
> `incident_par` enters `canopy_assimilation` in exactly one place — `absorbed_par = k ·
> incident_par · exp(−k·depth·lai)` — so a relative one-ULP nudge of either is the same
> perturbation of `absorbed_par`; and `env.get("par")` has exactly one consumer in the whole
> workspace (the station's lighting and sealed seams *write* the var, none reads it). The
> thermal `t⁴` is the one place a cheaper seam is measurably wrong — the subtraction
> `t⁴ − T_space⁴` can cancel, so perturbing the flow's *output* would understate it.
>
> **Three defences against the vacuous-zero trap this file records twice below**, and the third
> is one the Python instrument never had:
>
> 1. with the nudge **off**, every probe harness must emit its golden emitter's exact bytes —
>    so a harness that has drifted cannot go on measuring a scenario nobody froze;
> 2. every reading must be `> 0.0`;
> 3. every reading must land within an **order of magnitude** of the Python instrument's figure
>    for the same scenario. A probe that reads `1e-30` is non-zero and still wrong, and only the
>    number this re-measures can see that.
>
> ### ⚠⚠ The finding: CPython's `sum()` is compensated, and the port's comment said otherwise
>
> The two power readings came back at **exactly half** the Python instrument's. Chasing an exact
> factor of two rather than accepting it inside the order-of-magnitude window is what turned it
> into a finding. The 24 schedule values are bit-identical across the ports, nudged and
> un-nudged alike; the **derived load** differs by one ULP.
>
> **CPython's builtin `sum()` has used Neumaier compensation for floats since 3.12.** It is not a
> left-to-right accumulation and agrees with `math.fsum`, while `domains::power::daily_solar_energy`
> — whose own doc comment claimed to mirror it — accumulates naively. The two agree bit-for-bit on
> the frozen scenario, which is why every golden matches and nothing caught it, and they diverge
> the moment the summands move. Compensating that one sum in a throwaway Rust probe reproduced
> **both** Python figures exactly (`5.215406e-15`, `4.146325e-15`), which is what makes this
> measured rather than inferred. The naive accumulation is what the reference computes and is
> therefore the definition; what was corrected is the false claim about the other port.
>
> ⚠ **The durable half is about this contract.** A compensated reduction is a **second** source of
> cross-port divergence, independent of libm, and the tier classification at the top of
> `tiers.json` names only transcendentals. Five more `sum()` calls over floats live in the
> retiring Python tree — the drift regression and the perennial re-sow among them, both feeding
> frozen goldens. Nothing diverges today; until the checker is retired, a **Tier-1** byte
> disagreement could come from this rather than from a port defect, and the discovered-discrepancy
> protocol below should be read with that in mind.
>
> Ceremony: advisor-reviewed, the harness controls written before the measurements were trusted,
> mutation-controlled (8 mutations, each naming the test it must redden), documented here and in
> `rust/data/tiers.json`'s `measured_2026_08_27_by_the_reference` block.

## ⚠ 2026-08-16 — the reference flip inverted who is judged (slice 5)

Everything below was written with **Python as the reference and Rust as the candidate**.
The user's decision to make Rust canonical (`docs/plans/post-roadmap-reference-flip.md`)
reversed that, and slice 5 is where it took effect on this contract. Read the rest of this
file with the following three corrections in hand; they are stated here rather than edited
into every paragraph, because the paragraphs are also the historical record of how the
bands were measured and that history is what makes them credible.

**1. Eighteen of the twenty-five goldens were the Rust port's own bytes when this was
written.** ⚠ Slice S6 (2026-08-27) deleted the Python tree, so of the twenty-one goldens on
disk today **nineteen** are regenerable — from their `emit_*` examples, by hand, see below.
The two that are not: `state_snapshot.json` never was and never should be (a hand-authored
`sim_io` fixture that Rust **reads**, so regenerating it from Rust is the round trip in its
purest form), and `drift_summary.json` is a **new gap** — its Python fold went with the
checker, and the measured reason it had not already moved to Rust (4 of 45 values shift by
≤7 ULP, needing tolerance-gating) was a property of the Python comparator that no longer
exists.

⚠⚠ **`--write` REFUSES since 2026-08-27 (slice S6), and the interim path is below.**
That tool validated every candidate through `sim_io.snapshot` before it could reach the
disk — *a golden that does not round-trip must never be written* — and S6 deleted the
Python tree that check lived in. A regeneration tool that writes **unvalidated** bytes over
a freeze contract's values is worse than one that refuses, so it refuses. **Reporting is
unaffected** (`uv run python tests/crossport/regen_goldens_from_rust.py`, no flag): it
compares the emitter's bytes against the committed ones and needs no validator, so the
*"which goldens would move"* half of this step still works exactly as before.

Until the blessed path moves to Rust (S6 build item 2), regenerate **one golden at a time,
by hand**, from `rust/`:

```
cargo run -q -p <crate> --example <emitter> > ../rust/data/golden/<name>
git diff -- rust/data/golden/<name>      # review it; this diff IS the record
```

⚠ Always `-p <crate> --example`, never a built binary path: `emit_crew` exists in **two**
crates and `target/*/examples/emit_crew` is whichever built last — one of which re-emits the
golden *from itself*. `-p` is what makes that unreachable. The emitter for each golden is
named in `regen_goldens_from_rust.py`'s `RUST_EMITTERS`, which is still the roster even
though its write half is disabled.

**2. "The port has no reference authority" is superseded for those eighteen** (it still
holds verbatim for the seven). Its replacement is not weaker, it points the other way: a
Rust-vs-golden byte difference is no longer a port defect to hunt, it is **the reference
moving**, and it is regenerated deliberately with the freeze ceremony that follows. What
the discipline protects has not changed — that a disagreement is *diagnosed* rather than
absorbed — only which side is presumed right. The discovered-discrepancy protocol below
inverts accordingly: step 1's question becomes "is this a **Python** defect, or a reference
change Python has not followed yet?", and step 2 still routes a genuine finding through the
owning freeze contract's unfreeze discipline.

**3. Byte-exactness is now a Rust claim, and Python's is the tolerance-gated side.** Rust
reproduces all eighteen exactly on the generation platform, with **no exemptions** — that
census is `tests/crossport/test_golden_provenance.py`. Python reproduces sixteen of them
exactly and diverges on two by ~1–2 ULP; those two are named in
`tests/golden_platform.PYTHON_DIVERGES` with their measured sizes, gated at a **1e-14**
last-bit-noise ceiling (a thousand times tighter than the Tier-2 bands below), and the
roster is checked in both directions so a divergence that heals is as red as one that
appears.

⚠ **What did *not* change: the tiers, the bands, and their measured provenance.** The
question the bands answer — how far a one-ULP libm disagreement moves a whole trajectory —
is a property of the *scenario's dynamics*, not of which language holds the reference, and
the two engines are demonstrably running the same arithmetic (16/18 byte-identical, 18/18
inside 2 ULP, and a 2440-step per-step export with zero bitwise divergence). Every
sensitivity in the table below was re-measured on 2026-08-16 and every figure reproduced.
⚠ The plan's stated reason for expecting otherwise was wrong and is corrected here:
`post-roadmap-reference-flip.md` §5 says the bands were measured "through the *Rust-side*
transcendentals"; `measure_tier2_bands.py` is pure Python and shims CPython's own `math`,
so the basis was always Python-side and the inversion does not reach it.

⚠ The purity invariant this file opens with — *`git diff src/` stays empty* — is A's rule
and is **not** rewritten here. It is slice 11's, together with the standing posture section
in `CLAUDE.md`.

## Why a tolerance contract at all — the frozen goldens don't cross the language boundary

Phases 4 and 6 froze the reference as **byte-identical within a Python build** (hex-float
goldens). That guarantee **stops at the language boundary**. The frozen scenarios are
saturated with transcendentals — FvCB photosynthesis (`exp`/`sqrt`), the weather half-sine
and daylength (`sin`/`tan`/`acos`), phenology, the Stefan–Boltzmann `T⁴` radiator, the
ECLSS/thermal equilibria — and `exp`/`pow`/`sin` differ at the last ULP between one
platform's libm and another's. A raw byte-compare of a Rust snapshot against a Python golden
would fail on **physically-meaningless noise**. So the contract is **three tiers, applied per
scenario** (the tier is a property of the scenario's *evaluation graph*, not of individual
flows — in a coupled run every downstream flow operates on already-diverged inputs).

The port has **no reference authority.** A discrepancy the port surfaces is a **finding
routed through the freeze unfreeze discipline**, never a silent Rust-side fix (see *The
discovered-discrepancy protocol* below).

## The three tiers

**Tier 0 — structural / discrete invariants: EXACT for every scenario (the primary gate).**
Integers and classifications; a float divergence large enough to flip one is a real port bug,
not last-ULP noise, so they are asserted exactly *even for* tolerance-tier scenarios. The
snapshot-visible set (`tiers.json._tier0`):

- the integer step count `n` (`t = n·dt`), the `rng_seed` (`0x`-hex), and the **stock-id
  set** (structure never drifts mid-run);
- per-stock `domain` / `quantity` / `unit` / `kind` / `unclamped` and the **composition key
  set** (which quantities a stock carries);
- the **aux-accumulator key set** — the non-conserved accumulators a run carries
  (`thermal_time`, `vernalization_days`, and since 2026-08-11 `rooted_depth`). An
  accumulator present in one port and absent in the other is reported by the comparator as
  a **structural** diff, not a numeric one, so it is a tier-0 failure at every tier. ⚠ This
  bullet was ADDED 2026-08-11: the aux channel had been inside the comparison since the
  accumulators existed, but this list — the fourth freeze contract's prose, the one with
  **no manifest to gate it** — never said so. Found only because adding a third accumulator
  made the comparator print `$.aux.rooted_depth: present in reference, missing in candidate`
  before the Rust mirror landed. Exactly the `freeze-prose-half-is-ungated` shape;
  the *values* of those accumulators are then compared at the scenario's own float tier;
- the **stability-signature booleans** — `is_period_2` / `is_stationary` in the drift-summary
  goldens (the period class: since scope (B) increment 1, **perennial and consumer are both
  period-1** — the perennial's former period-2 cycle was a broken-canopy artifact that
  closing the canopy dissolved; station is period-1 too). Both ports must agree on the class.

Plus two Rust-run-side invariants that a completed emit run *proves* rather than the snapshot
carrying: **`events == ()`** and **`rationed == 0`** (asserted in the emit examples), and
**conservation holds every step in Rust** — the per-quantity ledger residual is re-asserted
inside the Rust integrator, and for the two-rate driver after **every** fast sub-step (the
sealed run's ~1.3 M sub-steps completing *is* the proof the five-domain ledger balanced
throughout). This is the single strongest structural-fidelity signal the port has.

**Tier 1 — bit-exact float trajectories: scenarios with no transcendental in the graph.**
Where the whole evaluation graph is pure IEEE-754 arithmetic (`+ − × ÷`, comparisons,
`min`/`max`), determinism is exact across ports *given identical operation order* — which the
core's canonical id-sorted reductions and ASCII-only ids (Python `str` sort == Rust UTF-8
byte sort) preserve. These get a **bit-pattern-exact** gate (via `struct`-packed f64), and
they hold on **any** conformant platform regardless of libm. Classify by the ops **executed**,
not the closed form: a geometric contraction `dₙ = d₀·(1−k·dt)ⁿ` is *n* sequential multiplies
(basic ops, bit-identical), **not** a `pow()` call — so it is Tier 1.

**The four Tier-1 goldens** (verified transcendental-free, all cabin-based, no biosphere):
`crew_state`, `eclss_state`, `cabin_gas_state`, `water_recovery_state`.

**Tier 2 — tolerance-gated float trajectories: everything a transcendental touches.** The
default for the biosphere and the coupled scenarios (the other 16 goldens). The gate is a
**relative-deviation band** on the parsed final-State amounts (with a per-quantity `floor`),
reusing `lab/oracle_match.py`'s `max_abs_relative_deviation`. The Tier-0 invariants still hold
exactly. Bands are **measured, never derived** (see below).

**Comparison mechanics.** Compare **parsed f64 values, never JSON bytes.** Rust *emits* the
`sim_io` hex-float snapshot; **Python does all parsing and comparison** (`tests/crossport/
compare.py`). This sidesteps any "does Rust's hex-float spelling match `float.hex()`
byte-for-byte" question — we compare decoded *values*. The comparator validates **any** port's
snapshot (see *Port-agnostic* below).

## Per-scenario tier assignment (the 20 frozen goldens)

`tiers.json` is authoritative; this table is the human-readable summary. Bands are the
measured Tier-2 tolerances (`floor` is `1e-12` throughout).

| Golden | Group | Float tier | Band | Transcendentals in graph |
|---|---|---|---|---|
| `crew_state` | station | **1 (bit-exact)** | — | none (forced linear depletion) |
| `eclss_state` | station | **1 (bit-exact)** | — | none (first-order linear controls) |
| `cabin_gas_state` | station | **1 (bit-exact)** | — | none (crew↔ECLSS, no biosphere) |
| `water_recovery_state` | station | **1 (bit-exact)** | — | none (linear recovery atop cabin) |
| `season_euler_state` | biosphere | 2 | `1e-11` | FvCB `exp`/`sqrt`, transpiration `exp`, weather trig |
| `sealed_chamber_state` | biosphere | 2 | `1e-11` | FvCB + transpiration + weather trig |
| `perennial_chamber_state` | biosphere | 2 | `1e-11` | FvCB + transpiration + weather trig |
| `perennial_long_horizon_state` | biosphere | 2 | `1e-11` | FvCB + transpiration + weather trig (15 yr) |
| `consumer_chamber_state` | biosphere | 2 | `1e-11` | FvCB + transpiration + weather trig |
| `consumer_long_horizon_state` | biosphere | 2 | `1e-11` | FvCB + transpiration + weather trig (15 yr) |
| `drift_summary` | biosphere | 2 | `1e-11` | FvCB (peak series); `is_period_2` is Tier-0 exact |
| `power_state` | station | 2 | `1e-12` | half-sine solar `sin` |
| `power_self_discharge_state` | station | 2 | `1e-12` | inherits half-sine `sin` |
| `thermal_state` | station | 2 | `1e-12` | `RadiatorReject` `T⁴` |
| `station_state` | station | 2 | `1e-12` | half-sine `sin` + `T⁴` (Power→Thermal) |
| `sealed_energy_drift_summary` | station | 2 | `1e-12` | `sin` + `T⁴`; `is_stationary` Tier-0 exact |
| `sealed_station_state` | station | 2 | `1e-11` | biosphere FvCB + `T⁴` + all seams (multi-year) |
| `greenhouse_state` | station | 2 | `1e-11` | biosphere FvCB coupled into the cabin |
| `harvest_state` | station | 2 | `1e-11` | biosphere FvCB (built on the greenhouse) |
| `lighting_state` | station | 2 | `1e-11` | biosphere FvCB (lamp forces PAR) |

## Tier-2 bands — measured, never derived, framed by use

A band must sit **above** the last-ULP-propagated cross-port noise and **below** any
physically-meaningful drift. The trap: on a single machine Rust `f64::sin`/`powf` and CPython
`math.sin`/`**` resolve to the **same system libm**, so the direct Rust-vs-Python deviation
reads **0.0** — a same-libm artifact, not a cross-libm measurement. A band set "above 0" would
be a *derived guess* violating the contract. So each band is justified by the **propagated
±1-ULP transcendental sensitivity** (`tests/crossport/measure_tier2_bands.py`): perturb the
relevant `sin` / `exp` / `t**4` by one ULP and re-run to the final state.

| Scenario group | Python instrument | **The reference (2026-08-27)** | Band | Margin |
|---|---|---|---|---|
| power (half-sine `sin`) | `5.215406e-15` | **`2.607703e-15`** | `1e-12` | ~380× |
| power + self-discharge | `4.146325e-15` | **`1.130816e-15`** | `1e-12` | ~880× |
| thermal (`T⁴`, contracting attractor damps it) | `1.909423e-16` | **`1.909423e-16`** | `1e-12` | ~5200× |
| station energy (`sin` + `T⁴`, coupled 7-day) | `5.215406e-15` | **`2.607703e-15`** | `1e-12` | ~380× |
| biosphere (Beer–Lambert `exp`, perennial 15-yr — *representative*, see below) | `3.519726e-15` | **`2.471757e-15`** | `1e-11` | ~4000× |
| greenhouse (7-day Beer–Lambert `exp`) | `2.814887e-16` | **`2.762079e-16`** | `1e-11` | ~36000× |

⚠⚠ **"worst" was the wrong word, and this table carried it until 2026-08-27.** The biosphere
row used to read *"worst: Beer–Lambert `exp`"*, and the Python instrument's own comment calls
it *"the dominant per-step transcendental"*. Measured on the perennial 15-year chamber, a
±1-ULP nudge of each weather forcing in turn gives:

| nudged forcing | propagated deviation | has a transcendental in its derivation? |
|---|---|---|
| `par` (the Beer–Lambert seam) | `2.412586e-15` | yes (`cos`, `exp`) |
| `daylength` | `1.981152e-15` | yes (solar geometry `sin`/`cos`/`tan`/`acos`) |
| `vpd` | `1.812943e-14` | yes (`exp`) |
| `net_radiation` | `1.926252e-14` | **no** — a raw fixture value |
| `temp` | `3.399269e-14` | **no** — a raw fixture value |

The two that cannot diverge across libm at all — they are JSON numbers parsed identically by
both ports — perturb the trajectory the *most*. So the spread is the **conditioning of the
15-year trajectory**, not a ranking of transcendental paths: a daily-constant table value
applied at every step moves more than a within-day value that is zero at night. The probe site
is therefore a **representative** choice and never was a maximum, in either port. Nothing here
touches the contract — the largest reading is still ~300× under the `1e-11` band — but a band
whose provenance says *worst* is claiming something it does not measure.

⚠ **The reference column is now the one that counts**, and the two columns differ for two
measured reasons, neither of which is a band moving. Thermal matches to every digit. The two
power rows read half because of CPython's compensated `sum()` (the 2026-08-27 unfreeze block).
The biosphere reads 0.70× because a one-ULP step is a *relative* perturbation of between
`2⁻⁵³` and `2⁻⁵²` depending on where the value sits inside its binade, and the `par` seam and
the `exp` it stands in for sit in different binades — the greenhouse, on a shorter run, reads
0.98×. Every margin is computed against the reference's own number.

⚠ **The two biosphere rows were re-measured 2026-08-14 (the within-day light path) and had
been stale in the other direction: `6.7e-14` → `8.2e-15` and `2.7e-15` → `6.1e-15`.** The
*gate* was never stale — `test_crossport.py` re-derives each sensitivity from the tree and
asserts `band > sensitivity`, so it would have gone red had a band stopped covering its
scenario — but the **recorded numbers** are prose, and prose is the ungated half of every
contract in this repo (`freeze-prose-half-is-ungated`). A green suite proves the bands
hold; it does not prove the table describing them is current. ⚠ Note the biosphere's
sensitivity *improved* by an order of magnitude while its goldens all moved: adding a
`cos` to the PAR path did not add ULP sensitivity, it changed which trajectory the
existing `exp` sensitivity propagates along. Neither band moved.

⚠⚠ **RE-MEASURED AGAIN 2026-08-15 (the layered canopy) — and this time the sentence above
("the *gate* was never stale") stopped being true, in a way it did not anticipate.** The
probe shimmed `domains.biosphere.canopy`'s `math`, because `intercepted_fraction`'s `exp`
was the assimilation path's only transcendental. The layered canopy moved that `exp` into
`photosynthesis.canopy_assimilation` — one call per Gaussian depth point — so the shim was
perturbing a function the carbon path **no longer calls**, and both biosphere rows measured
**exactly `0.0`**. `band > sensitivity` then held **vacuously, against zero**, which is the
precise failure the band contract's own rule was written against: a reading of `0.0` is
named three paragraphs above as *"a same-libm artifact, not a cross-libm measurement"*, and
the gate accepted one as proof.

⇒ Two repairs, both shipped. The probe now shims **both** modules that hold a Beer–Lambert
`exp` on the carbon path, giving `3.5e-15` (15-yr chambers) and `2.8e-16` (7-day
greenhouse); and both band tests now **reject a zero sensitivity outright**, so a probe that
stops perturbing the trajectory goes red instead of quiet. ⚠ The generalisation is the same
one the layered canopy's own record carries: **a probe is validated by what it perturbs, not
by what it is named after.** A gate that re-derives its input from the tree is only as live
as its handle on the tree.

⚠ **2026-08-16 — and the *other* prose half had missed both corrections.** Re-measuring for
the reference flip reproduced this table exactly (`3.5e-15`, `2.8e-16`, and the three Step-3
rows unchanged), but `tiers.json`'s per-scenario `evidence` strings still carried the
original `6.7e-14` and `2.7e-15` — neither the 2026-08-14 nor the 2026-08-15 correction
reached them. That is the direction nobody checks: this file names `tiers.json` as **the
authoritative source** whose prose it must not contradict, and the authoritative copy was the
stale one for two days. The gates were live throughout and no band moved; what rotted was the
record of why each band sits where it does. `tiers.json` now carries the current figures and
says where they were superseded.

The bands absorb realistic **multi-ULP cross-libm** divergence while a real port defect still
trips them. `test_crossport.py` re-measures each sensitivity and asserts `band > sensitivity`
(and `≤ 1e-9` for teeth). The sealed-station band reuses `BIOSPHERE_BAND` (`1e-11`) on the
regulator-erasure / period-1 argument — the ECLSS scrubber and O₂ makeup hold the shared gas
pools at their setpoints between the once-daily biosphere lumps, so a one-ULP nudge cannot
amplify across master days — **not** a ±1-ULP sweep of the 1.3 M-substep run (a deliberate
cost choice).

**Framed by use:** the sim's scientific claims survive at these bands — period class matches
*exactly* (Tier 0), equilibria and biomass agree to ~11 significant figures, conservation
holds to eps-scale. A deviation that *exceeds* a band is a port bug to hunt, not a tolerance
to loosen.

## The op-for-op libm audit

Matching the *mathematical* answer is not enough — the port must mirror the exact primitive
CPython called. `T**4` in CPython routes through C `pow()`, so Rust uses `powf(4.0)`, **not**
`powi(4)` (repeated multiply — bit-different, and it widens the Tier-2 deviation needlessly).
Every `**` / `math.*` site maps to its exact Rust equivalent:

| Python site | Primitive | Rust equivalent | Rust site |
|---|---|---|---|
| `power/system.py:156` | `math.sin`, `math.pi` | `.sin()`, `std::f64::consts::PI` | `domains/src/power.rs:299` |
| `thermal/flows.py:130` | `t**4 − space**4` (C `pow`) | `.powf(4.0)` | `domains/src/thermal.rs:99` |
| `thermal` equilibrium temp | `**0.25`, `**4` | `.powf(0.25)`, `.powf(4.0)` | `domains/src/thermal.rs:89` |
| `biosphere/canopy.py:77` | `math.exp` | `.exp()` | `domains/src/biosphere/science.rs:37` |
| `biosphere/photosynthesis.py:106` | `math.sqrt` | `.sqrt()` | `domains/src/biosphere/science.rs:52` |
| respiration `q10**e` | `**` (C `pow`) | `.powf(e)` | `domains/src/biosphere/science.rs:110` |
| transpiration `(t+c)**2` | `**` (C `pow`) | `.powf(2.0)` | `domains/src/biosphere/science.rs:128` |
| `biosphere/transpiration.py:108` | `math.exp` | `.exp()` | `domains/src/biosphere/science.rs:107` |
| `biosphere/weather.py:43-48` | `radians`/`sin`/`tan`/`acos` | `.to_radians()`/`.sin()`/`.tan()`/`.acos()` | `domains/src/biosphere/weather.rs:81-87` |

Beyond libm, bit-exactness lives on **operation order, not math**: float `+`/`*` are
commutative but not associative, so every integrator grouping and every `sorted()` reduction
is mirrored character-for-character (the three distinct reduction orders — flow×leg,
sorted-leg, sorted-stock — each walk the correct ordered source, never collect-then-refold).
The RNG's `u64` fold order is likewise load-bearing. See the Step 1–5 records in the plan.

## The cross-port CI gate (Step 6)

Steps 0–5 ran the parity comparison **locally only** (`skipif cargo is None`; the Python CI
job had no Rust). Step 6 closes that gap: a dedicated **`crossport`** CI job
(`.github/workflows/ci.yml`, Ubuntu, **both** the uv and Rust toolchains) runs the whole
`tests/crossport/` suite — **including `-m slow`**, so the two sealed goldens (`sealed_station`
~1.3 M sub-steps + the 15-yr energy drift) are gated too, all 20 not 18. Each parity test
shells out to `cargo run --example …`; the Python comparator applies the tier rules.

This is the repo's **first genuine cross-libm gate.** The Ubuntu runner uses glibc's libm for
*both* CPython and Rust, so Rust-vs-fresh-Python on that runner would be a same-libm no-op —
but the committed goldens were generated on **Windows / UCRT**, so the CI comparison is
**glibc-Rust vs UCRT-golden**: the real UCRT-vs-glibc measurement the Tier-2 bands were sized
for. **De-risk result (Linux container, before landing the gate blocking): all 20 goldens
pass their tier** — the four Tier-1 bit-exact (as guaranteed for transcendental-free graphs on
any platform), every Tier-2 within band including the multi-year sealed runs, every Tier-0
invariant exact. The measured bands genuinely absorb real cross-libm divergence over
decade-scale horizons.

## The discovered-discrepancy protocol

The port has **no reference authority.** If the cross-port gate ever fails — a Tier-2
deviation exceeds its band, a Tier-0 invariant flips, or the port surfaces a Python
ambiguity/bug — the resolution is **never** to loosen a band or patch the Rust side to match.
Instead:

1. **Diagnose** whether the divergence is (a) a Rust port defect (wrong op-order, `powi` where
   the reference uses `pow`, a mistranslated reduction) — **fix the Rust**, it is not the
   reference; or (b) a genuine finding about the **Python reference** (an underspecified
   corner, a latent bug the port exposed).
2. If (b), **route it through the freeze unfreeze discipline** — `docs/station-reference.md`
   or `docs/biosphere-reference.md`, whichever owns the item — and record the finding there.
   Any change to the Python reference is an unfreeze event with its own re-capture ceremony;
   the port does not get to move it silently.
3. A band is loosened **only** if a *re-measurement* (`measure_tier2_bands.py`) shows the
   ±1-ULP sensitivity legitimately rose — never to paper over an unexplained deviation.

As of Phase 7 exit, **no such discrepancy has been found**: all 20 goldens pass their tier
across both the local same-libm run and a **Linux container replicating the CI cross-libm
comparison** (glibc-Rust vs the UCRT goldens — the de-risk described above), with zero
changes to the Python reference (`git diff src/` empty throughout). The `crossport` CI job
enforces this on every push going forward; the container run is the pre-landing proof that
the measured bands absorb the real cross-libm divergence.

## Port-agnostic — and C# at the Phase-8 boundary

The interchange (`sim_io` hex-float JSON) and the comparator (`domains::tiers` +
`rust/data/tiers.json`, and `compare.py` until it is retired) are
**port-agnostic**: they validate *any* port's snapshot, not Rust's specifically. Phase 7 is
Rust-only, but the roadmap's second port target — **C#** for the Godot front-end (Phase 8) —
reuses this whole harness for free: a C# emitter producing the same `sim_io` snapshot is
gated by the identical Python comparator against the identical goldens at the identical tiers.
The tolerance contract is the port's, not the language's.
