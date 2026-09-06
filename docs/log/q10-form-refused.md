## **The Q10 temperature form is REFUSED as the reference and KEPT as an instrument** (the decision the kinetics item left owed — and it survives its own headline being withdrawn)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> Plan of record: `post-roadmap-temperature-kinetics.md`, the same plan that built the form.
> Filed under the September direction plan (referred to by that name here, never by
> filename — see the log's exemption note).

**DECIDED 2026-09-06 by the user.** No code, param, golden, manifest key, band or floor
moves: the refusal is a decision *not* to unfreeze, and `KineticsForm::Q10Teh` stays exactly
where it has been since 2026-09-04 — a cited alternative reachable only from the lab
(`cargo run --release -q -p domains --example science_switch -- form=q10_teh --long`). The
loader still always sets `Cardinal`, and the test that reddens if that default flips is the
thing that keeps this decision true rather than merely written down.

**FINDING 1 — the refusal stands on two grounds, and the headline that was originally offered
with them has been withdrawn.** `docs/log/temperature-kinetics.md` recommended refusal on
four counts. Two are load-bearing and were re-read here against the numbers, not the prose:

* **it breaks the above-ground biomass cap outright** — 18.366 against a recorded
  `peak_w < 14.4248`, a **27 %** breach. A band failure is a blocking finding that must be
  argued past in writing, and no argument was offered;
* **it collapses the chamber liveness floor**, i.e. the check that the perennial crop is
  still alive after fifteen years.

The other two are real but weaker: it deletes the whole-rate multiplier by an editorial act
[D] does not license for *our* `Jmax`-shaped light branch, and it promotes an **uncited**
`vcmax = 100` from nearly inert into a load-bearing position (Rubisco-bound lit steps
1.1 % → 9.1 %), which is a provenance debt taken on to buy a form change.

⚠ **The fifth thing that record offered was not a ground at all, and it is now refuted.** Its
closing paragraph said the form's real value was showing that *"the peak-LAI band is being
held by a loss term measured to be inert at the frozen params."* `docs/log/mutual-shading-tolerance.md`
(2026-09-05) took that question and **refuted it**: the term is bit-identically inert on peak
LAI at one point and **live on peak W in the same run**, the band's ceiling is reachable
(peak LAI crosses 8.0 at `specific_leaf_area` ×2.014 with the loss modelled, ×1.138 without),
and the loss **orders** the contract rather than blinding it. So the decision recorded here
rests on the biomass breach and the liveness collapse **only**. *A recommendation outlives the
argument that was decorating it; check which of its reasons are still standing before acting
on the recommendation.*

**FINDING 2 — "keep as an instrument" is not a consolation, it is what the build was for.**
The science-switch plan asked for a *scientific* pair and its own §2C had measured that the
tree held no second form of any biosphere process. `Q10Teh` is that second form, and refusing
it as the reference costs none of that: it remains the only object in this tree that can
answer "how much of a gated quantity is an artifact of the temperature treatment?" It is also
the instrument that established, against the plan's own prediction, that the open field runs
on the **light-limited** branch 99 % of lit steps — which is why the `vcmax` ladder every
earlier pricing was read against was measuring the branch that barely binds.

**FINDING 3 — what a future adoption would have to bring, so the refusal does not have to be
re-litigated from scratch.** Not owed by anyone now; recorded so the next person pricing it
starts above zero.

1. A **defensible `Vcmax`** — the refusal's fourth count is the promotion of an uncited
   literal. Wullschleger (1993, *J. Exp. Bot.* 44:907) tabulates wheat and is the retrieval.
2. An argument for the **whole-rate multiplier's deletion** that addresses our `Jmax` branch
   rather than [D]'s Collatz one, or a form that keeps both without double-counting.
3. A **re-posed biomass cap**, argued in writing before the run rather than after — retuning
   a bound so a change fits is the co-adaptation this project has refused four times.

**FINDING 4 — the refusal is enforced by a test that already exists, which is why this record
changes no code.** Flipping the loader's default to `Q10Teh` reddens the bit-identity control
(measured as one of that item's three mutations). So "the reference is Cardinal" is a claim
the suite can see, not a sentence in a doc. ⚠ The converse is *not* gated and is not worth
gating: nothing reddens if someone deletes the lab form entirely. That would lose an
instrument, not corrupt a contract.

**Gates.** None run for this record and none needed: it is a decision not to change anything,
and `git diff` over `rust/` is empty for it. The numbers it quotes are
`docs/log/temperature-kinetics.md`'s and `docs/log/mutual-shading-tolerance.md`'s own,
re-read rather than re-measured — and that is stated because this project's rule is that a
re-read of a numeric claim quotes a measured baseline column. Here the claim being recorded
is *a decision*, not a measurement, so nothing new was measured for it.
