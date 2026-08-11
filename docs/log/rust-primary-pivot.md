## **Development posture — the Rust-primary pivot**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> split out on 2026-08-12 under rule 4 of
> [`../context-budget.md`](../context-budget.md). It was a single physical line of a
> markdown table; the only change is where it breaks. The fuller record is the plan
> doc this row names.

<!-- record-body: rejoin the lines below on single spaces for the original cell -->

**DECIDED (2026-07-20): Option A — Rust-primary for new content/product; Python
frozen-canonical + kept green for validated science.** The user asked "isn't it time to work
only in Rust?"; the honest answer is a *work-type split*, not a flip. Python anchors all
four validation mechanisms (freeze manifests freeze the *Python* tree; cross-port validates
Rust *against* Python goldens; the PCSE oracle laboratory is EUPL/Python and **never
portable**; `git diff src/` purity). **The A-vs-C hinge is one question**: more
validated-science calibration coming? Python's *only* un-portable forward value is minting
NEW PCSE oracle traces — committed goldens+traces still *run* without live Python. User:
**not done** (chose to validate the day-neutral crop), so Python stays (rules out C; **B** —
flip the reference — rejected: re-anchors every freeze contract for zero scientific gain and
*weakens* the cross-port bug-catch by making the two ports non-independent). **Going-forward
rules**: (1) new content/gameplay is **Rust-first, no Python mirror owed**; (2) it gets
**Rust-native conservation+determinism**, not cross-port goldens (authored ≠ validated); (3)
validated-science/manifest-named changes stay **Python-canonical** under the unfreeze
discipline; (4) the frozen core/laboratory don't move; (5) **a frozen reference must stay
green to stay a reference** (CI cost of keeping Python). The laboratory is **dormant, not
retired** — woken only for a decided validation job.
`docs/plans/post-roadmap-rust-primary-pivot.md`
