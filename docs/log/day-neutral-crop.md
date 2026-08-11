## **Scope (B): the day-neutral habitat crop, validated vs LINTUL3 spring wheat**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**COMPLETE (2026-07-20) — the "second wheat" ceremony 2 left open; the first (and, pending
direction, last) job to wake the laboratory under the pivot.** Additive +
default-preserving: `SeasonScenario.vernalization`/`.photoperiod` flags (both default True) +
`DAY_NEUTRAL_SCENARIO` (both off, thermal time only); **all 7 frozen biosphere goldens
byte-identical**, no unfreeze, `git diff src/simcore` empty. Diagnosed against an **offline,
license-clean** LINTUL3 spring-wheat oracle (`tests/oracle/lintul3_runner.py`; params ship
WITH PCSE, no network/unlicensed cache — a strict upgrade on the winter oracle). **Framing
(advisor): our DVS is the SAME model family as LINTUL3 (linear-GDD TSUM1/TSUM2), so a
phenology "match" is near-tautological — it tests our param CHOICE, not our model.**
Deliverable is honest: "a literature-cited, sane day-neutral crop with its gaps to LINTUL3
documented," NOT "we validated our model." **Three findings**: (1) **the partition finding
corroborates ceremony 2 across a SECOND, independent oracle** — our `tsum` is veg-heavy
(1100/750), *both* WOFOST and LINTUL3 (800/1030) are repro-heavy; maturity coincides at day
135 as the **same two-errors-cancelling** trap (total thermal time ~1855 ≈ ~1830 °C·day);
(2) **both produce a realistic wheat canopy** (~5.6–5.7 peak LAI, 1.02×) — sane, *not*
cross-validation (two independently-parameterized canopy models both landing near a
realistic peak); (3) the genuine non-tautological gap is **canopy TIMING** (ours peaks ~13 d
after anthesis, LINTUL3 2 d before) + LINTUL3 front-loads roots (0.55 vs 0.31 at DVS 0.5).
**Clean-room**: no primary spring-wheat TSUM on the shelf, so reuse our OWN cited
winter-wheat `tsum` (never copy LINTUL3's, = reverse-engineering PCSE) — ruling-B-clean, gap
recorded not fit. **Purpose demonstrated**: in a warm habitat the frozen winter wheat is
**permanently arrested** (max DVS 0.0 — no vernalization ⇒ `verfun`≡0 ⇒ thermal time gated
to 0), the day-neutral crop develops normally + runs the sealed chamber (Euler+RK4,
`rationed==0`, deterministic). Lamp-*photoperiod*-control demo dropped (day-neutral ⇒
flowering ignores daylength). `docs/plans/post-roadmap-day-neutral-crop.md`
