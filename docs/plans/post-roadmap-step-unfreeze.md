# The step unfreeze — the biosphere moves to `dt = ¼` (authorized 2026-08-14)

**Status: PLANNED 2026-08-14, IN PROGRESS. Authorized by the user** after Step 0's two
measurement axes (`docs/log/co2-controller.md`, `docs/log/step-sweep.md`). The sweep
recommended `dt = ½`; **the user chose `dt = ¼`** — the same ceremony, with 4.8× headroom
instead of 2.1×, so the next mechanism added to the tree probably does not force a repeat.

This is the largest unfreeze this project has run. It follows the discipline in
`docs/biosphere-reference.md` §"The unfreeze discipline", and it is deliberately **split into
two commits** so the golden diff is attributable.

---

## 0. Why, in one paragraph

In a sealed chamber the crop draws the air's CO₂ down over the season. Below the CO₂
compensation point (`Γ*/ci_ratio = 61.07 ppm`) assimilation is exactly zero — a hard floor.
At the shipped step the model drives the sealed and perennial chambers **below that floor**
(57.89 and 56.03 ppm) and keeps fixing carbon anyway. It is a truncation error, not a
threshold crossing: the observable *converges* as the step shrinks (57.89 → 75.06 → 75.82 →
76.03 against an RK4 limit of 76.29). At `dt = ¼` every sealed scenario clears, at every CO₂
enrichment level, with `k·h` ≈ 0.18–0.21.

**The price, all three numbers** (sealed chamber, Euler, against the converged limit): the
season-low CO₂ error goes **−24 % → −0.6 %**, peak leaf carbon **+4.0 % → +0.7 %**, harvest
**−0.7 % → −0.1 %**.

---

## 1. ⚠ The step change is a FORCING-INDEX change, not a constant edit

**This is the part that makes it a design job rather than a search-and-replace.** Every
forcing in this tree is a pure function of the integer step count `n`, never of `n·dt`
(Step 0 axis 1, scope finding 1, audited at all five sites). So simply passing `dt = 0.25`
and `steps = 4·len(weather)` would feed the crop **the entire season's weather in the first
quarter-year** and then clamp on the final day for the remaining three quarters. The probe
sidestepped this by tiling the weather list; production cannot tile at 75 call sites.

Four things are indexed by `n` and each needs a rule:

| site | today | needs |
|---|---|---|
| `season.py:_table` (5 weather forcings + 3 constants) | `values[min(n, last)]` | index physical time |
| `run_perennial(..., year=)` — the annual reset | `len(weather)`, a step count | scale with the step |
| `biosphere/perturbations.py:window_override` | `start <= n < end` | scale with the step |
| `station/perturbations.py` | same | already writes `3 * _SPD` — the house pattern |

**The chosen design:** `_table` indexes **physical time** —
`values[min(int(n * dt), last)]`. At `dt = 1.0` that is `values[min(n, last)]`
character-for-character, and `n * 0.25` is exact in binary so `int()` truncates cleanly.
Then `dt`, steps-per-day and steps-per-year go behind one small helper in the biosphere
domain, and the call sites are routed through it so they read as **physical time** rather
than step counts.

## 2. The two-commit split (the reason for it)

**Commit 1 — the indexing change, at `dt = 1.0`, with EVERY GOLDEN BIT-IDENTICAL.**
`_table` moves to physical-time indexing, the helper lands, the call sites are routed through
it, the perturbation windows are expressed in days. The step itself does **not** move. The
exit criterion is that the full suite passes with **no golden regenerated**.

⚠ **Commit 1 is a FORM change that moves nothing in the manifest** — the `WSFD` shape that
`docs/biosphere-reference.md` calls out explicitly as *the unfreeze nothing catches*. Both
automatic gates are blind to it, so the ceremony on commit 1 is honor-system and is followed
deliberately, not waited for.

**Commit 2 — the step moves.** One edit to the helper's constants, then regenerate. Because
commit 1 proved bit-identical, **every byte of commit 2's golden diff is attributable to the
step**, which is exactly what this project's *"predict the golden diff before regenerating"*
discipline needs.

## 3. ⚠ What is NOT in this ceremony

The direction plan proposed one ceremony carrying the step **plus** the parked leaf mechanism
**plus** a chamber-CO₂ science band. **The leaf merge is deliberately excluded.** The user
authorized a decision about *the step*; and the plan's own condition for merging the leaf
branch is that its evidence base be **re-measured at the shipping step** — every number that
got it accepted (the Greenwood gate clearing by 5.2 %, thickness inside real wheat's range,
`rationed == 0`, the `WSFL` leverage cut) was measured at Euler `dt = 1`, and that measurement
has not been done at `dt = ¼`. Bundling it would also make the golden diff unreadable.

Land the step, regenerate, verify. **Then** put the leaf question with its evidence
re-measured. Same authorization, two attributable regenerations.

The chamber-CO₂ science band is a small addition and may ride along **after** the step lands
and its readings are known — a band written before the regeneration would be a prediction, not
a contract.

## 4. Predicted golden diff — WRITTEN BEFORE REGENERATING

From the sweep (`docs/plans/post-roadmap-step-sweep.md` §1–2), at Euler `dt = ¼`:

| observable | now | predicted | direction |
|---|---|---|---|
| sealed season-low CO₂ | 57.89 ppm | ~75.8 ppm | **up ~18 ppm** |
| perennial season-low CO₂ | 56.03 | ~75.5 | up ~19 |
| consumer season-low CO₂ | 73.29 | ~74.4 | up ~1 |
| water_biting season-low CO₂ | 87.96 | ~95.7 | up ~8 |
| sealed peak leaf C | 0.9215 | ~0.8923 | **down ~3 %** |
| sealed harvest | 0.7189 | ~0.7234 | up ~0.6 % |
| open field peak leaf C | 9.3023 | ~9.4889 | up ~2 % |
| open field harvest | 33.7142 | ~34.1489 | up ~1.3 % |
| unclamped margin (sealed) | 1.3072 | ~5.4574 | **up ~4×** |
| `rationed` | 0 | **0** | unchanged — asserted |
| extinction events | () | **()** | unchanged — asserted |

**Anything outside this table is a finding, not a nuisance.** In particular `rationed` and
`events` are *assertions*, not predictions: if either moves, stop.

## 5. The rest of the ceremony, in order

1. ⚠ **`run_master_day`** must take `STEPS_PER_DAY` slow substeps per master day
   (`src/station/driver.py`) — engine code, and the cost Step 0 found that the direction
   plan never priced. **Confirmed 2026-08-14: this IS a station-contract unfreeze too.**
   `run_master_day` is not itself named in `docs/station-reference.manifest.json`, but the
   manifest's `numerics_note` reads *"Sealed reference: biosphere-slow **dt=1 day** +
   everything-fast dt=60 s"* — the step is written into the station contract in prose, so
   it moves with this change.
   ⚠ **The commit-1 indexing change makes this driver edit much smaller than feared.** The
   driver's stated reason for one slow step per day was *"so `n` stays the day count the
   day-indexed weather resolver reads"* (`driver.py:44-46`, and again at 109-112). That is
   now obsolete: the resolver indexes `int(n · dt)`, so `n` may be four times the day count
   and still read the right weather row. The `fast_dt · steps_per_day == 86400` guard is
   about the **fast** domain and is untouched. Add a `slow_steps_per_day` keyword defaulting
   to `1` so existing callers stay byte-identical, and update the two stale comments.
2. **Regenerate the affected goldens**, each through its own explicit `__main__` action, and
   review the byte diff against §4.
3. **Regenerate both manifests** and review the diff — the git-visible record of what moved.
   ✅ **Good news, found 2026-08-14: the step change is NOT honor-system.**
   `docs/biosphere-reference.manifest.json` records `dt_days: 1.0`, and
   `tests/test_freeze_manifest.py:444` asserts it **against a hard-coded literal**
   (`assert manifest["dt_days"] == 1.0`). So the step move fails loudly and the contract is
   updated deliberately — unlike commit 1, which nothing catches. Keep that assertion a
   **literal**; rewriting it to compare against `BIO_DT` would make the contract auto-follow
   the code, which is the opposite of a freeze.
4. **Report the science gates** — every `science_bands` and `liveness_floors` reading. ⚠ A
   band failure is a **blocking finding to be argued past in writing**, never a number to
   re-tune; and a band *passing* is not an endorsement (`open_season` sits 3.8 % above the LAI
   lower bound — a tight margin, not comfort).
5. **Cross-port tiers** — 7 biosphere + 4 station goldens move, so `tests/crossport/tiers.json`
   bands need **re-measuring**, not just re-running.
6. **The Rust mirror** — the port has no reference authority; it mirrors the rule. The
   `_table` analogue is `rust/crates/domains/src/biosphere/system.rs::table_schedule`
   (mirrored in `6861e2b`, bit-identical at `dt = 1`); the step constant needs the same
   treatment when it moves.
7. ~~⚠ **The CI hazard**~~ ✅ **DISCHARGED 2026-08-14 — no new work.** `tests/golden_platform.py`
   already gates every transcendental golden behind `windows_golden_only`
   (`skipif sys.platform != "win32"`), precisely because hex-float goldens are byte-exact
   only on their generation platform. All the biosphere goldens are transcendental and
   therefore already gated, so regenerating on Windows will not redden CI. The genuinely
   cross-platform gate is the cross-port tolerance band — which item 5 re-measures anyway.
8. **Gates**: full suite including `-m slow`, `ruff`, `pyright`, `cargo test`, `cargo clippy`.
   ⚠ `¼` is **4× raw simulation work**, so the full-suite upper bound is ~25 min, not the
   ~12 min quoted for `½`. Use targeted regeneration during the work and one full run at the
   end.

## 6. Invariants that hold throughout

* `git diff src/simcore/` **stays empty, unconditionally** — there is no unfreeze path that
  edits the core. The step is a domain-side instantiation choice.
* The arbitration backstop stays Euler-only; `rationed == 0` stays asserted on golden runs.
* No test is weakened or deleted to make this pass. A test that goes red is a finding.
* Conventional Commits naming the unfreeze; commit and push to `main`.
