"""Stem-reserve remobilization — the stem feeding the grain (post-roadmap, 2026-08-12).

The other half of the question ``annual_reset`` answered years earlier. That one kills
the whole plant to litter at each year boundary; this one is what real cereals do
*before* they die: **the stem's stored carbohydrate moves into the filling grain**.

Before this the frozen tree had no such path at all. Stem carbon could only be shed to
litter or burned for maintenance, and the measured consequence was on the record: on
``open_season`` the stem gained **62 % after flowering** and was still gaining within
days of the last step, where a real wheat stem peaks around anthesis and then *loses*
weight. So this is not a mistuned parameter — the mechanism was **absent**.

**The form is [E]'s own, and the alternative was measured before it was chosen.** [E]
§3.2.4 "Formation of shielded reserves" (p. 93):

    "A simple way to deal with the formation of shielded reserves is to assume that **a
    certain fraction of the increase in stem weight will be available for redistribution
    after flowering** (Listing 3 Lines 17, 35). This fraction is assumed to consist only
    of starch. Some data on the magnitude of the remobilizable fraction are given
    Table 7."

So the reserve accumulates **continuously, as a fraction of stem GROWTH** — the stem's
weight is structural + starch all season — and it is *not* a snapshot of 40 % of the
stem taken at flowering. That distinction is the whole design:

* the growth-fraction form is what [E] **programs** (Listing 3), and Table 7 is cited
  *into* it as the source of its parameter;
* a one-shot fill read off Table 7's *caption* is a model **we constructed**, with no
  listing line in the book.

Both were built and measured. The programmed form fixes the stem shape (end ÷ flowering
**1.618 → 0.985**, i.e. it stops gaining) and overshoots the committed oracle's harvest
index by **15 %**; ours lands on the harvest index exactly and leaves the stem still
gaining **+35 %**. ⚠ **The programmed form is what ships**, and the reason is a standing
ruling rather than a preference: *the oracle is a diagnostic, never a fit target*, so
picking the second because it hits the reference number is the refused shape. The
overshoot is recorded as a consequence, not tuned away — and note ``fstr`` was **not**
moved to close it (at ``fstr = 0.20`` the sourced form also lands on the oracle, but
0.20 is **Sorghum's** row, not Wheat's).

⚠ **A THIRD reading exists and was measured and refused: stop the FILL at flowering.**
[E]'s own introduction describes the reserve as *"glucose formed **before flowering**
and stored as polysaccharides, such as starch"* (§2.2.1 p. 46), which reads as "deposit
before anthesis, withdraw after". In [E]'s tree the two readings **cannot be told
apart** — its stem fraction reaches zero, so no stem growth (and therefore no deposit)
survives flowering anyway. In ours they differ sharply, because our partition table
flat-extrapolates and keeps feeding the stem: gating the fill at anthesis hands the
stem its post-flowering growth back and **reinstates the very defect this mechanism
exists to fix** (stem shape 0.985 → **1.267**), while dropping grain 16 %. It also
lands the harvest index on the oracle (1.002×) — the refused shape again, and this time
not even a consolation, since the variant is *also* physically worse. Recorded as a
measured alternative, not a live option.

**THE MECHANISM STOPS AT MATURITY, AT BOTH ENDS, AND THAT BOUND IS [E]'S OWN.** The
window is ``trigger_dvs <= DVS < cessation_dvs`` for the drain and ``DVS <
cessation_dvs`` for the fill — outside it the stem neither stashes starch nor feeds the
grain. The reason is not a physiological claim we are making about dead stems; it is
that **[E]'s program has no state past DS = 2.0 at all.** Listing 3 — the very module
whose Lines 17 and 35 carry this mechanism — ends at Line 114 with

    ``FINISH DS = 2., CELVN = 3.``

and the book says so twice in words: *"Simulation is halted by imposing an end when the
development stage reaches the value of 2.0, by including the statement FINISH DS = 2.0"*
(§3.1.4 p. 81), and *"simulation always stops when either the FINISH conditions of
maturity (DS = 2.0), or that of severe carbohydrate shortage (CELVN = 3.0) is reached"*
(§3.4.2 p. 105).

⚠ **Read that at its exact strength, because it is easy to over-claim.** ``FINISH`` is a
*run-control* statement, not a statement about the redistribution flow: [E] does not say
"remobilization ceases at maturity", it says its model **does not exist** there. So this
is not a cited cessation rule — it is the **domain boundary of the source**, and running
the mechanism past DS = 2.0 is extrapolating a form outside the program that defines it.
Our tree has no ``FINISH``: it runs a fixed number of days and its DVS merely *caps* at
2.0, so `open_season` spends 11 steps past maturity and `sealed_chamber` — which never
re-sows — spends **two years** there. Declining to extrapolate is the choice being made,
and it is the user's call, taken 2026-08-12.

**Four numbers, and their provenance ranking is inverted against what they do** —
measured, not argued:

===========================  ==============================  =========================
number                       what [E] gives                  measured effect
===========================  ==============================  =========================
``remobilizable_fraction``   **tabulated** (Table 7, wheat)  the only one that moves
                             — CABO *unpublished* data       much
``remobilization_rate``      stated bare, no citation        **bit-inert on carbon**
``trigger_dvs``              **ours** — [E]'s cannot fire    near-inert (peak LAI
                                                             bit-identical, DVS 0–1.5)
``cessation_dvs``            [E]'s ``FINISH`` line — its     −2.0 % grain on
                             program's own end, not a        ``open_season``; every
                             cessation rule                  frozen band unmoved
===========================  ==============================  =========================

The drain rate is inert because once carbon is in the reserve it is already outside
maintenance *and* outside senescence, and the grain is too — so moving it between them
is a **rename**. (Checked at ``to_bits()`` over every stock at every step across rates
0.05 / 0.2 / 1.0: only the reserve and the grain differ. At rate ``0`` nitrogen moves
too, because the grain sits inside the nitrogen target's denominator — a degenerate
case, not a candidate.)

⚠ **THE TRIGGER IS OURS AND IS LABELLED AS OURS.** [E] induces redistribution "once
stems stop growing", which **can never fire in this tree**: ``allocation.yaml``'s stem
fraction is 0.10 at DVS 2.0 and the table flat-extrapolates, so our stem keeps growing
for as long as there is assimilate. [E]'s trigger is really a statement about *[E]'s*
partition table, in which the stem fraction reaches zero. The substitute is the weaker
**availability** condition §3.2.4 states in its own words — reserves are "available for
redistribution after flowering" — i.e. ``DVS >= 1.0``. Backfilling our partition table
from [E]'s own Table 18 to make the real trigger fireable was **taken and refused**: it
drives peak LAI to 2.201 against a contract band of 5.0–8.0 (see
``docs/plans/post-roadmap-wheat-partition-backfill.md``).

**Where the extra grain comes from — the transfer, not the exemptions.** Turning off
both of the reserve's exemptions (making the starch pay maintenance *and* be shed at
``rdr_stem``) still gives **+49.6 %** grain against the full form's +53.5 %. Whole-
season accounting on ``open_season``: the system total barely moves (133.95 → 136.78
mol C), litter falls, respiration falls, grain rises. It is a **redistribution, not
extra photosynthesis.**

Pure stdlib only. The fill lives in ``carbon_budget.Allocation`` (a split of its own
stem leg, so the partition maths cannot drift); this module owns the params and the
drain.

Sources:
  [E] Penning de Vries, F.W.T., Jansen, D.M., ten Berge, H.F.M. & Bakema, A. (1989),
      *Simulation of Ecophysiological Processes of Growth in Several Annual Crops*,
      Simulation Monographs 29, PUDOC/IRRI. §2.2.2 + **Table 7** (pp. 46–47),
      **§3.2.4 p. 93**, §3.2.6 p. 95. The book ``allocation.py`` already cites.
"""

from dataclasses import dataclass

from domains.biosphere.phenology import PhenologyParams, development_stage
from simcore.environment import Environment
from simcore.flow import FlowResult, Leg
from simcore.ids import FlowId, StockId
from simcore.state import State


@dataclass(frozen=True)
class StemReserveParams:
    """Loader-produced stem-reserve parameters (the four numbers above).

    ``remobilizable_fraction`` (``fstr``) is the share of each day's stem growth held
    apart as shielded starch; ``remobilization_rate`` is the first-order daily draw on
    the standing reserve once ``trigger_dvs`` is reached; ``cessation_dvs`` closes
    **both** halves — [E]'s ``FINISH DS = 2.`` is where its program ends, so past it
    there is no form to run.
    """

    remobilizable_fraction: float
    remobilization_rate: float  # day⁻¹
    trigger_dvs: float
    cessation_dvs: float


@dataclass(frozen=True)
class StemRemobilization:
    """CARBON ``stem_reserve_c -> storage_c`` on ``trigger <= DVS < cessation`` (Σ = 0).

    First-order on the standing reserve, so the draw is **donor-controlled and therefore
    self-limiting** (``rate · reserve → 0`` as the reserve → 0): the Euler arbitration
    backstop is structurally unreachable on it, the same property every other clamped
    withdrawal in this tree has by construction rather than by clamping.

    ``flux = rate · reserve · dt`` — dt-linear. Outside the window the flow emits **no
    legs at all** rather than a zero one, which is the tree's idiom for "this flow does
    not act today" (``FlowResult`` rejects duplicate legs, and a zero leg would make the
    aux-free structural-zero tests unable to tell the two cases apart).

    **The window is half-open at BOTH ends**, ``trigger <= DVS < cessation``. The upper
    end is [E]'s ``FINISH DS = 2.`` (Listing 3 Line 114) — the point past which its
    program does not exist — and it is **strict** so that the last acting step is the
    last one [E] would have simulated. ⚠ Our ``DVS`` *caps* at 2.0 rather than growing
    past it, so a non-strict ``<=`` would not merely add one step: it would leave the
    drain running for the whole post-maturity tail (11 steps on ``open_season``, **two
    years** on ``sealed_chamber``, which never re-sows), i.e. exactly the behaviour the
    cessation exists to stop. The strictness is load-bearing, not stylistic.

    ⚠ The window reads ``DVS`` off the step-entry snapshot's thermal-time accumulator,
    the same read ``Allocation`` makes, so the fill and the drain cannot disagree about
    the phase within one step — and they now share the *same* upper bound, so they
    stop together on the same step, or the reserve would fill with nothing to drain it.
    """

    id: FlowId
    priority: int
    stem_reserve_c: StockId
    storage_c: StockId
    thermal_time_aux: str
    pheno: PhenologyParams
    params: StemReserveParams

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        dvs = development_stage(
            snapshot.aux.get(self.thermal_time_aux, 0.0),
            tsum_anthesis=self.pheno.tsum_anthesis,
            tsum_maturity=self.pheno.tsum_maturity,
        )
        if not self.params.trigger_dvs <= dvs < self.params.cessation_dvs:
            return FlowResult(legs=())
        # ⚠ Association is load-bearing: float multiplication is not associative, so
        # ``(rate·reserve)·dt`` and ``rate·(reserve·dt)`` can differ in the last bit.
        # This is the grouping the probe measured and the goldens were predicted from.
        reserve = snapshot.stocks[self.stem_reserve_c].amount
        flux = self.params.remobilization_rate * reserve * dt
        if flux == 0.0:
            return FlowResult(legs=())
        return FlowResult(
            legs=(Leg(self.stem_reserve_c, -flux), Leg(self.storage_c, flux))
        )
