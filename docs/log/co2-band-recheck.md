## **The allowance that was already discharged** (the direction plan's first free item, taken and refused — and the margin pin the Python deletion took with it)

**REFUSED 2026-08-31, and the refusal is the deliverable.** The item taken was the direction
plan's §4 first bullet — write a known-deviation note into `docs/biosphere-reference.md` saying
the shipped step leaves the sealed chamber's season-low CO₂ below its own compensation point. It
is listed there as *"do this first, regardless of both answers in section 7"* and marked free
(the freeze's prose half is ungated, so it moves no hash). **Writing it would have put a false
statement into a frozen contract.** No scenario has been below the floor since 2026-08-14.

**FINDING 1 — the item written to be answer-independent was the one the answer cancelled.** The
bullet is scoped to the *waiting* (*"while the decision is pending"*), not to the defect, and §7's
step decision landed the next day and **fixed** the crossing (`dt = 1 → ¼`) instead of documenting
it. So the sentence claiming it holds *"regardless of both answers in section 7"* is exactly
backwards: the answer is what discharged it. It then sat unstruck for seventeen days, at the top
of a list, labelled cheap — *a free item is the one nothing forces you to re-read*.

**FINDING 2 — it inherited a locus error that had already been corrected in its target file.**
The bullet's `57.9 ppm` is the sealed chamber driven through an unconditional re-sow **no golden
performs**; the real crossing was the *perennial* chamber's, at 56.03. The freeze doc corrected
that on 2026-08-14, in the very section the bullet asked to edit. A pointer does not inherit its
target's corrections, and this one would have carried a retracted number back into the contract
the correction lives in.

**MEASURED, not assumed** (shipped tree `7f60442`), each scenario driven the way its own golden
drives it — four through the lab report's frozen column, the fifth through a throwaway example
deleted after the run:

| scenario | season-low CO₂ | margin | vs the `cc44b41` table in the freeze doc |
|---|---|---|---|
| `sealed_chamber` | 71.435803 ppm | 1.1697× | identical |
| `perennial_chamber` | 70.252606 | 1.1503× | identical |
| `consumer_chamber` | 73.338613 | 1.2009× | identical |
| `perennial_long_horizon` | 70.252606 | 1.1503× | identical |
| `consumer_long_horizon` | 73.338613 | 1.2009× | identical |

Floor `61.071429 ppm`; all five clear, all five gated in Rust. Two unfreezes and sixteen days
later the doc's table is exact to every digit — worth recording in the one section whose own
lesson is that *a value written into prose acquires no owner*, because **a re-measurement that
confirms is the only thing that discharges that** without waiting for the next unfreeze.

**FINDING 3 — the five-margin pin did not survive the Python deletion, and nothing replaced it.**
`test_the_five_margins_are_pinned_not_merely_positive` is the guard the freeze doc singles out as
*"the pin written for exactly this event DID fire"* — it went red at the light path and forced a
re-pin. It lived only in `tests/test_co2_compensation_band.py`, deleted with the checker on
2026-08-27. C4 moved the **band** and both tripwires into Rust; the classification table named
the residue as *"the probe arithmetic"*, and the margin pin went out with that phrase. Today no
assertion in `rust/` records how NEAR any of the five sits to its floor.

⚠ **Stated at its real size — closer to a supersession than a loss, and the difference is which
of the pin's two jobs is unowned.** *Detection* is mostly re-owned: all five scenarios have
byte-frozen goldens, so a change that moves the run reddens one. But those goldens are
**final-state snapshots** (`perennial_chamber_state.json` is the state at `n = 6100`), not
trajectories — the trough is not among the pinned quantities, and a change halving every margin
still leaves all five one-sided gates green, which is the exact hazard
[`co2-compensation-band.md`](co2-compensation-band.md) wrote the pin to cover. *Visibility* is
unowned outright: the lab report prints the number on demand for **four of the five** (its readout
roster has no `consumer_long_horizon` row), and nothing requires anyone to run it at unfreeze
time.

**NOT BUILT, on purpose.** A five-margin characterisation pin in Rust is a new assertion on a
frozen contract's observable, not a prose correction, so it is named as a candidate and left to
the user rather than folded into a doc pass taken as an hour of free work.

**What shipped:** the §4 bullet struck in place with all three reasons; the direction plan's
status header struck too (it still announced the step decision as the one open item, sixteen days
after it was taken); and in `docs/biosphere-reference.md`, a *"RE-CHECKED 2026-08-31"* subsection
with the table above plus strike marks on the band section's two dead references — the link to
`tests/test_co2_compensation_band.py` and the evidence sentence *"`git diff src/` empty"*, both
naming a tree S6 deleted. ⚠ The vacuous one is worse than the dead one: a broken link is visibly
broken, while *"`git diff src/` empty"* still reads as a passing check.

⚠ **The general shape, and it is the third instance this doc set has logged:** a forward-looking
list is written once and read many times, and **nothing re-checks it**. The freeze doc has gates
beside it; the direction plan has none, and its items go stale from the front, where the cheap
ones are.

**No code changed. No golden, param, manifest entry or gate bound moved** — `git status` clean
outside the three docs and the memory file.
