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
**empty** — the Python reference is untouched; the Rust port lives under `rust/`, and the
cross-port comparator + classification table live under `tests/crossport/` (test tooling).
The machine-readable companion is **`tests/crossport/tiers.json`** (the per-golden tier +
band table — the **authoritative** source; this doc's prose must not contradict it). The
plan of record is [`docs/plans/phase-7-native-core.md`](plans/phase-7-native-core.md).

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

| Scenario group | Measured ±1-ULP sensitivity | Band | Margin |
|---|---|---|---|
| power (half-sine `sin`) | `5.2e-15` | `1e-12` | ~190× |
| power + self-discharge | `4.1e-15` | `1e-12` | ~240× |
| thermal (`T⁴`, contracting attractor damps it) | `1.9e-16` | `1e-12` | ~5000× |
| station energy (`sin` + `T⁴`, coupled 7-day) | `5.2e-15` | `1e-12` | ~190× |
| biosphere (worst: `canopy.exp`, perennial 15-yr) | `6.7e-14` | `1e-11` | ~150× |
| greenhouse (7-day `canopy.exp`) | `2.7e-15` | `1e-11` | ~3700× |

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

The interchange (`sim_io` hex-float JSON) and the comparator (`compare.py` + `tiers.json`) are
**port-agnostic**: they validate *any* port's snapshot, not Rust's specifically. Phase 7 is
Rust-only, but the roadmap's second port target — **C#** for the Godot front-end (Phase 8) —
reuses this whole harness for free: a C# emitter producing the same `sim_io` snapshot is
gated by the identical Python comparator against the identical goldens at the identical tiers.
The tolerance contract is the port's, not the language's.
