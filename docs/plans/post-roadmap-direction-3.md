# Post-roadmap direction — the THIRD plan, written 2026-09-06

**Supersedes `post-roadmap-direction-2026-09.md` (2026-09-02) under that document's own
rule 5**, and the trigger was measured rather than felt: that file is 48,821 B in 635 lines,
of which **30 complete strike-through spans hold 34,949 B — 71.6 % of it**. Rule 5 says
*"when the struck spans outnumber the live ones, write the successor"*; at seven-tenths of
the bytes there is no reading of the rule under which a fourth pass of strike-through is
the right move. ⚠ It also carries **61 `~~` markers — an odd number**, so one strike was
opened and never closed. That is a small thing and it is the argument in miniature: a
document edited five times by striking cannot be checked by reading it.

**Re-read against the record's last row:** `atmosphere.md`

⚠ **That line is a gate, not a note.** `repo_gates` asserts it names the record table's
*last* row. Landing a new item appends a row, so this doc goes red until someone re-reads it
against that item — strikes what it discharged, adds what it named — and moves the marker.
It cannot judge the re-read. Do not move the marker without one.

**Referred to in records as "the third direction plan", never by filename** — the plan-doc
parity gate reads the log's index section against the record files, and a filename in a
record with no index row turns it red.

**What is different about this plan.** Its predecessor was a *queue*: a list of work with
rankings, and the rankings went stale faster than the work got done. There is no queue left
to rank. Everything that could be built without a decision has been built, so §2 is three
decisions and one blocked retrieval, and the bulk of this document is §3 — **what has been
closed by measurement and must not be re-proposed**. That section is the expensive part of
the predecessor and the part worth carrying forward; the strike-through history stays in the
predecessor, which is kept whole.

---

## 1. Where the project stands (2026-09-06)

- Roadmap Phases 0–9 COMPLETE. **64 post-roadmap items** in `docs/post-roadmap-log.md`
  (65 index rows, 64 record rows, the surplus of 1 asserted — counted 2026-09-06, and the
  predecessor's "53" was correct on 2026-09-02: eleven items landed in four days).
- **Rust is the reference** (2026-08-16); the Python checker is gone (S6, 2026-08-27).
- Four freeze contracts hold: biosphere (Euler, `dt = ¼`, 15 param files, 7 scenarios),
  station (13 scenarios, biosphere delegated), native-port tolerance, authoring platform.
- ⚠ **The two lines below were true on 2026-09-06 and were ended on 2026-09-08** by the
  perturbation batch and the decision that followed it (§6): the stop was a stop of the
  *nominal-roster* thread, and the model's behaviour space off that roster had never been
  entered. Kept because the reasoning that produced them was sound and the boundary it missed
  is the finding.
- **The science thread has reached a natural stop.** Not "ran out of ideas" — every mechanism
  offered to the open canopy question over three weeks was measured and either refused on its
  sign or shipped and found inert (§3). The three decisions in §2 are what is left, and none
  of them is a mechanism shortage.

## 2. What is open

Three decisions and one blocked retrieval. **Nothing here is work you can just do**, which is
why they are not presented as a queue.

### 2.1 DECISION — the parked leaf mechanism. Recommendation: REFUSE

`leaf-expansion-blocked` sits below the frozen tree on **both** gated observables at **every**
step (`log/leaf-remeasurement.md`). Its ship/refuse call has been the user's since 2026-08-14
— the longest-standing open item in the project.

* **If refused:** the branch is retired with a record, exactly as the Q10 form was
  (`log/q10-form-refused.md` is the shape). Nothing moves; no unfreeze.
* **If shipped:** it makes two frozen observables worse. There is no reading of the evidence
  that recommends this.
* ⚠ **Nothing is waiting on it either way.** It has been listed as open for three weeks
  because refusing something is still a decision, not because anything depends on the answer.

### 2.2 DECISION — the chamber CO₂ setpoint controller. Recommendation: NOT YET

State unchanged since `log/co2-controller.md`: a setpoint controller does not remove the step
defect, needs `~3000 ppm` to clear the margins on the plant-only chambers, and a two-sided
setpoint **vents 235 of 429 mol** — an odd thing for a simulator whose subject is closure of
matter cycles. It is buildable now that the step is `¼`.

* The measured surprise worth remembering: holding the sealed chamber at the 357 ppm it
  starts from is **four times worse** than letting it deplete, because the uncontrolled
  chamber self-limits and a controller removes that feedback.
  ⚠ **A SECOND, independent arrival at that mechanism, 2026-09-08** (`log/perturbation-suite.md`):
  shrinking the chamber while holding its composition rations at ×0.75, while *dropping* the
  composition does not ration even at ×0.25 with a third the carbon — because a falling mole
  fraction drops demand along with supply. Holding the fraction holds the appetite and removes
  the buffer. Two unrelated routes to one mechanism; this bullet is no longer a single
  measurement.
* **Why "not yet" rather than "no":** whether a habitat that vents carbon is the right realism
  move is a design question about what this simulator is *for*, not a measurement. It is a
  reasonable thing to want; it is not a defect to fix.

### 2.3 ~~DECISION — adopt the live-O₂ FvCB form, or leave it in the lab~~ — TAKEN and BUILT, both slices

⚠⚠ **DECIDED 2026-09-06 and this section's pricing was FALSE.** The user chose adoption, then —
shown the measurement — chose **course (B): fix the cabin oxygen setpoint first**. Slice 1 is
BUILT (`log/o2-setpoint-cited.md`); **adoption itself is still owed.** Three of this section's
own claims were struck by measurement rather than by argument:

* ⚠ *"only `sealed_chamber` — the jar"* is **FALSE about the tree.** It was measured through the
  biosphere lab's roster, which cannot reach a station assembly. Adoption moves
  `sealed_station`'s principal stocks by **60–75 %** — and the cause was a cabin charged at
  1.05 mmol/mol O₂, not the science.
* ⚠ *"at the ratios this form produces (12 at charge, 685 at the end) it stops
  discriminating"* — **the ×685 is at the wrong instant.** The band's true minimum is
  **×10.674948**, at step 779 of 3661. In a sealed chamber CO₂ and O₂ are anticorrelated
  (PQ = 1), so the live floor is HIGHEST exactly when CO₂ is lowest: the pointwise band is
  evaluated at the hardest instant the run contains, automatically.
* ⚠ *"Adopting means re-posing that band first"* — **discharged.** The re-posing is
  `post-roadmap-o2-form-adoption.md` §3/§8a, and it holds: the band binds on all five
  scenarios.

~~**What remains is slice 2**~~ — **BUILT 2026-09-07** (`log/o2-form-adopted.md`). No
scientific objection survived: the `ci_ratio` worry raised while re-posing was refuted **on its
sign** (a fixed Ci/Ca understates Ci at low ambient CO₂, so it is the conservative assumption).

✅ **The unmeasured prediction is MEASURED and it held.** *"With slice 1 landed, adoption's
station move should shrink to the control-sized one it was priced at"* — `sealed_station`'s
largest carbon move is **+0.058 %**, against the 60–75 % the row above records. The station's
three sealed assemblies are controls on this form now, so this section's original *"only the
jar"* pricing is **true of today's tree and was false of the tree it was written on**, which is
a stranger outcome than either being simply right.

⚠ **Two things this item NAMED, neither of them a decision:**

* **The station census still carries no CO₂ or cabin-oxygen band**, and adoption did not add
  one. A gate was designed and **refused on measurement**: `O2Makeup` holds the cabin at
  209.800 mmol/mol with ~50× of headroom against any defensible band, so it would have been
  inert by construction. Closing it needs a subject with a reachable falsifier — a **perturbed**
  run, not a regulated nominal one.
  ⚠ **STILL OPEN, and the 2026-09-08 batch deliberately did not close it.** That batch built
  perturbed subjects, which is the prerequisite this bullet names — but a census row lands in
  `docs/station-reference.manifest.json`, so closing it is **an unfreeze with a ceremony**, not
  a free consequence of having a subject. It is better taken **after B**, when the quantity it
  would freeze is the one B leaves behind rather than the one B replaces.
* ~~**A form can be nearly inert on the frozen roster and loud under perturbation.**~~
  **TAKEN and BUILT 2026-09-08** (`log/perturbation-suite.md`), on the user's *"start
  deliberately breaking things"*. The claim stands and its scope was too small: the perturbation
  suite existed but had **never reached the biosphere** (`grep -rln "perturbations::"` returned
  nothing under `domains/tests/`), so no perturbation had ever touched the plant science.
  ⚠⚠ **And the +9.8 % is scoped to REGULATION, not to the science.** Halving oxygen alone gives
  **jar −1.45 %, big perennial chamber −0.86 %** — both opposite in sign to the station —
  because O₂ enters twice (leaf `Γ*`; soil `x/(K+x)` on the decomposers, who *are* a sealed
  chamber's CO₂ supply) and only the station's cabin is **defended** by `O2Makeup`. So *"less
  oxygen is better for plants"* is a statement about a regulated habitat and is false of every
  chamber in the frozen roster. ⚠ The obvious explanation — the jar's low charge — was
  **refuted by its own control** inside that batch; the big chamber's route is 18× its soil
  throttle and is left **open**.

~~The original pricing follows, struck, because its numbers are still the record of what was
believed:~~


**NEW 2026-09-06.** Both halves are built and lab-only (`log/o2-form-built.md`): the loader
still selects `O2Form::Constant`, and the form is reachable as
`science_switch -- o2form=live_pool`. Adoption is a separate decision from the build, and
nothing waits on it.

* **What it would change:** only `sealed_chamber` — the jar — where the observable moves
  71.435803 → **7.294541**. The two big chambers move +0.140 % / +0.118 %; the open field is
  bit-identical *by construction*, having no oxygen stock.
* **Why the jar and nothing else:** its oxygen fraction is 2.0 mmol/mol at charge and 0.033
  at the end against a frozen constant of 210 — 105× and 6329× out. For the other two
  chambers the frozen constant is *exactly right* (210.0 against 210.2).
* ⚠ **The price is not the number, it is the guard.** The `min > Γ*/ci_ratio` band is written
  against a **constant** floor; a live Γ* turns it into a pointwise claim — a different
  assertion, not a re-tuned one — and at the ratios this form produces (12 at charge, 685 at
  the end) it stops discriminating. **Adopting means re-posing that band first**, in writing,
  before a run. The build deliberately did not do this, because re-posing a band inside a
  build that claims to unfreeze nothing would have been the unfreeze it was avoiding.

### 2.4 BLOCKED — the Bernacchi page check

The FvCB item bound Kc/Ko/Γ* to Bernacchi et al. (2001) *as the literature reproduces them*;
the paper itself is unreachable. Ten minutes with the PDF: read Table 1, confirm 404.9 /
278.4 / 42.75 at 25 °C, strike the "owed" in the file header. If a digit differs, **record it
and do not move the number** — a value move is a 13-golden ceremony and its own decision.

⚠ **The blocker is not the box and not the network** (re-checked 2026-09-02: web access works,
Wiley returns 403, no open copy exists). It needs the PDF put into `sources/` by hand, or a
logged-in fetch. Asked of the user 2026-09-02; **still owed**.

⚠ Do **not** discharge it with a secondary table (PhotoGEA, Sharkey 2007, an R package's
constants). Those are the same reproduced-by-the-literature evidence the file already cites,
so they would close the item without adding a single new observation.

⚠ **Before treating any future retrieval as blocked, check `sources/` first.** That is the
repeat finding in §5, and this item is the only one on this list that has actually earned the
label.

### 2.5 The provenance queue — 56 markers, and it is not a task list

`grep -rl 'TODO(cite)' rust/crates/domains/params` — **56 markers across 20 files** in the
biosphere and sibling params, plus 4 in `station` (re-counted 2026-09-06; unchanged since
2026-09-02). Bucket 3(C) closed the no-oracle set as **blocked on retrieval, not effort**.
Do not reopen it wholesale. Take a single param only when a *finding* leans on it — which is
how `carbon_fraction`, the FvCB constants, `specific_leaf_area` and `extinction_coef` were
all taken, and all four are now closed.

---

## 3. Closed by measurement — do not re-propose these

This is the section that earns the supersede. Each line is a direction that looks open to a
fresh reader and is not, with the record that closed it.

⚠⚠ **EVERY VERDICT BELOW WAS MEASURED ON THE NOMINAL ROSTER, AND THAT ROSTER IS NOT THE
MODEL'S BEHAVIOUR SPACE.** This caveat lived in §2.3 until 2026-09-08, where discharging the
build it was attached to would have carried it off with them; it belongs here, against the
verdicts it qualifies. The 2026-09-08 perturbation batch (`log/perturbation-suite.md`) **did
not re-measure any row below** — it built the instrument that could. So a row reading *"inert
on the chambers"* is a claim about nominal runs, and the one thing that batch demonstrated is
that a claim of that exact shape can invert off them: an oxygen leak moves `sealed_station`
0.058 % nominally and 9.8 % perturbed, and the *sign* of an oxygen change is opposite in a
regulated cabin and an unregulated chamber. **Do not re-propose the mechanisms below. Asking
whether one of their verdicts is scoped to an observable the nominal roster silences is a
different question, and it is open.**

**The canopy magnitude question is CLOSED, and it was smaller than three weeks of planning
said.** `open_season` peak LAI reads **6.0228** at the shipped step and **5.4273** converged,
inside the `5.0 < peak < 8.0` band **at every step in the sweep** — the deviation the plans
kept citing was retired by the frozen contract before the second plan was even written
(`log/partition-sensitivity.md`). The separate `peak < 6.0` check is 0.38 % over and is
satisfied by its own restated *"…or the 5 %/day mutual-shading loss is MODELLED"* clause.

**Every mechanism offered to it was measured. Do not offer them again:**

| candidate | verdict | record |
|---|---|---|
| the canopy regulator | built; **inert on the chambers** | `log/canopy-regulator.md` |
| the parked leaf mechanism | below the frozen tree on both observables at every step | `log/leaf-remeasurement.md` |
| the intra-canopy light path | **sign is backwards** — `Ag` is concave in PAR | `log/gross-net-gas-exchange.md` |
| re-tuning the band | refused three times | `log/canopy-magnitude.md` |
| the partition table | absorbs a **threefold** error at the knot the canopy responds to | `log/partition-leaf-direction.md` |
| `specific_leaf_area` keyed to development | the source **refuses the question** | `log/sla-leaf-population.md` |
| the Q10 temperature form | refused as the reference, kept as an instrument | `log/q10-form-refused.md` |

**Four more measured facts that close directions:**

* **`extinction_coef` is BOUND at 0.60** (`log/extinction-coef-bound.md`), the value the tree
  already carried, shipped as a provenance-only unfreeze — no number, no golden. `0.68` is not
  merely uncited: it is **red on the perennial liveness floor** (0.538913 against `> 0.55`)
  *and* non-monotone on the observable it was proposed to move.
* **The mutual-shading loss ORDERS the contract, it does not blind it**
  (`log/mutual-shading-tolerance.md`). It is bit-identically inert on peak LAI at the frozen
  params and **live on peak W in the same run** (+0.206 %). Without it the two open-field
  bounds are near-redundant (2.8 % apart); with it they separate by 1.9×.
* **The band's upper half is reachable, not decorative:** peak LAI crosses 8.0 at
  `specific_leaf_area` ×2.014 with the loss modelled and ×1.138 without — tolerant by 1.77×.
* **The peak-W crest is light saturation, not nitrogen** — pinned by
  `the_peak_w_crest_is_light_saturation_and_not_nitrogen`. Its 0.13 % agreement with
  `nitrogen.yaml`'s "the two curves coincide only at W ≈ 14.44 t/ha" is a coincidence of two
  unrelated derivations, and was asserted from arithmetic before it was measured.

**One frozen-tree fact worth keeping in view:** the open field runs on the **light-limited**
branch, and the crossover to the Rubisco branch sits only ~15 % below the shipped `Vcmax`
(`log/o2-coupling-measured.md`). Anything that cuts `Vcmax` at the cool end of the season
does not "raise assimilation" — it **switches which process limits**. Two plans priced items
against the branch that does not bind.

---

## 4. The structural queue — the repository, not the simulation

**Closed 2026-09-06, this item:**

* **The doc↔code bound pair is now asserted.** `docs/context-budget.md` stated the byte
  bounds in prose and `rust/crates/repo_gates/tests/context_budget.rs` declared them as
  constants, and **nothing compared them** — the surviving instance of *"a rule with two
  copies has one that is stale"*, flagged by the fourth bound's own commit as out of scope
  there. A normative table under rule 3 is now compared to the constants by exact set
  equality in both directions, so a raise that edits one side is red. ⚠ The retraction that
  had stood above `MAX_MEMORY_INDEX_BYTES` — *"the Python copy is GONE, this file is now the
  only copy"* — was right about the file it named and **wrong about the count**.

**Open, and each is a fact to know rather than a defect to fix:**

* **The memory index is outside the repo.** `repo_gates` reads it from the user's profile, so
  on CI and on any box but the author's the memory bounds are **unchecked and say so out
  loud**. A CI-green claim about the memory budget is not a claim about the memory budget.
* **`senescence.yaml`'s `shade_rate` note is stale** — it says the term is *"BIT-IDENTICALLY
  inert"*, written 2026-07-27 and falsified by the layered-canopy commit. Left deliberately: a
  comment edit in a param file is a manifest hash and therefore an unfreeze. Correct it the
  next time that file's manifest entry moves for a real reason — that is exactly how
  `canopy.yaml`'s three stale sentences came out on 2026-09-06.
* **`self_discharge.yaml`'s pointer is stale**, on the same terms and still waiting for the
  same kind of occasion.
* **`rust/data/tiers.json`'s `drift_summary` evidence string** (`max_rel_dev 0.0`, dated P7.4,
  measured 9.955e-16) is the **native-port** contract with its own ceremony, ruled out of
  scope when the drift work was planned. Still out of scope; listed so it is not lost.
* **Two records say "Γ* is TODO(cite)" as of their date** (`co2-margin-pin.md`,
  `co2-compensation-band.md`). Dated records are not maintained — listed so nobody "fixes"
  them.
* ⚠ **`scenarios/bioregenerative_station.yaml`'s direction-gate reasoning is FALSE**, found
  2026-09-08 by B. It states the O₂ regulator *"approaches 9.76275 MONOTONICALLY FROM BELOW"* —
  the arithmetic of `o2_setpoint = 10.0`, superseded by 1995.0 on 2026-09-06. Nothing caught it
  because **nothing in `rust/crates` runs the authored scenarios at all**; they are runtime-only
  content under *"authored ≠ validated"*. Listed as a fact rather than a defect to fix here,
  because the fix belongs with the slice-4 authoring decision above — but note the shape, which
  is new: the *unvalidated* half of the tree can be silently broken by a frozen-side change, and
  the freeze ceremony does not look there.
* **The eleven transcendental goldens can only be regenerated on Windows/UCRT.** An unfreeze
  that moves one of them from a Linux box has no regeneration step there. Record it in the
  ceremony; do not `--write` around it.

---

## 5. What keeps happening — the repeat findings, with counts

Not lessons in the abstract. Each of these has bitten this project more than once, and the
count is the reason it is here rather than in a record nobody re-reads.

1. **CHECK THE SHELF FIRST.** A retrieval priced as blocked, ranked, re-ranked and deferred,
   whose source was already in `sources/` or already cited elsewhere in the tree. Four
   records name themselves as instances: `canopy-regulator.md`, `canopy-provenance.md`
   (a literal cited a file away), `temperature-kinetics.md` (the temperature response a plan
   called *"understood, not retrieved"* was tabulated in Teh ch. 6, on our own shelf, and
   already cited by constant in `science_gates.rs`), and `sla-leaf-population.md` — ranked
   three times across five days without anyone opening the PDF. **Before pricing a retrieval,
   open the shelf.**
   ⚠ **SIXTH instance, 2026-09-06** (`log/o2-setpoint-cited.md`): the BVAD cabin-atmosphere
   page was in the same PDF the 2026-07-02 crew-params retrieval had already fetched, with an
   extract of it still on disk. **The first time the shelf held the NEXT SECTION of a document
   already cited** — which is a harder case to notice than a missing source, because the
   document is already in the citation list.
   ⚠ **Do not quote a count for this.** Those records number themselves *"fourth instance"*,
   *"fourth time"*, *"fifth instance"* and *"5th"* in an order that cannot all be right, and
   `fvcb-provenance.md` claims a fifth as well. The pattern is solid and the tally is not —
   which is finding 3 below, happening to the finding above it.
2. **A re-read checks the claims it is looking at; the claims it *writes* are checked by
   nothing.** The 2026-09-02 re-read added a brand-new false claim about a harness it had not
   run, in the same pass that was fixing stale claims.
   ⚠ **THREE more instances on 2026-09-06, all inside one document written to prevent exactly
   this** (`post-roadmap-o2-form-adoption.md`): a headline built on a ratio read at the wrong
   instant (§8a); a `ci_ratio` objection whose sign was backwards (§8c); and a recommendation
   to re-charge a pool that a controller regulates, written four lines after the same document
   flagged that pool's constancy as *"worth understanding before re-charging it"* (§10a).
   ⚠ **The sharper form, from the two unpredicted reds in §12d:** *a search that finds an
   instance of what it is looking for stops looking.* Two `makeup_flux_*` tests were found,
   correctly reasoned about, and the conclusion written as though it covered the family — the
   flow-level pair sat one screen further down the same file.
3. **A number quoted in three places is a number nobody re-measures.** The merge-remedy
   figure was an eyeball carried through three documents while load-bearing for a refusal;
   measured, it was a third to a half of the claim.
4. **A record's probe figures are not the tree's.** Two plans quoted 5.03 / 4.42 as the tree's
   peak LAI. They were a probe's numbers, for a candidate that record **refused**, computed
   with a rule it retracted in the same paragraph.
5. **A qualifier that converts "lower priority" into "not on the list" is how an available
   item disappears from a queue.** The predecessor's work list declared itself empty while an
   unblocked item sat on it, excluded by the words *"neither blocked nor re-ranked"*.
6. **A gate's prescribed remedy can be unaudited.** The memory ceiling told three raises to
   *merge* memory files, and nothing checked that a merge was finished — a half-done one
   orphans a file and every byte bound reads that loss as an improvement.

---

## 6. Recommended order

**Work with no decision needed: NONE.** Said plainly, because the predecessor's version of
this line was false when written. There is genuinely nothing left that can be built without
either a decision or the Bernacchi PDF.

**The three decisions (§2.1–2.3) are independent** — none blocks another and none blocks any
work — so they can be taken in one sitting or left indefinitely. Recommended: refuse the
parked leaf mechanism (§2.1), because it is the only one where the evidence points one way
and it has been open the longest.

~~**Then the project needs a direction, not an item.**~~ **ANSWERED 2026-09-08, and not with
the product track.** Shown that the perturbation axis was unexplored biosphere-side, the user
chose it (*"i agree, go with it"*), and then, shown that the habitat has no atmosphere to lose,
chose the follow-on explicitly: **"ok A now, but immediately after that (next session) B"**.

* **A — BUILT 2026-09-08**, `log/perturbation-suite.md`.
* ~~**B — DECIDED, not proposed: give the habitat a real atmosphere.** Total gas becomes a
  stock, pressure becomes state, and the leaf reads partial pressures instead of mole fractions
  over a constant. It trips the trigger `eclss.yaml`'s own `o2_setpoint` source string wrote on
  2026-09-06.~~ **BUILT 2026-09-08**, `log/atmosphere.md` — slices 1–3 with the full ceremony,
  9 goldens and 9 manifest hashes. ⚠⚠ **Two sentences of the entry above were FALSIFIED by
  building it, and both are struck rather than quietly reworded, because each was load-bearing
  when it was written:**

  1. *"the leaf reads partial pressures instead of mole fractions **over a constant**"* — the
     second half is a **misdiagnosis**. At fixed V and T, `p_i = n_i·R·T/V`, so
     `p_i/P_ref = n_i/n_ref` where `n_ref` is the moles filling the **room** at reference
     pressure — which is exactly what the constant already was. Dividing by a *live* total
     gives the mole fraction, which does not move under depressurization at fixed composition,
     so building this sentence literally would have made the leaf **blind to a hull breach** —
     and no golden could have caught it, since `n_total == n_ref` at charge. What was missing
     was never the denominator: it was that **nothing conserved the air**.
  2. *"Total gas becomes a **stock**"* — it did not, deliberately. A stock that must equal a
     sum of other stocks is a redundancy the conservation gate cannot police. The inert fill is
     the stock; total gas and pressure are **folded** from the species.
  3. *"It trips the trigger"* — **it does not.** The trigger's condition is an ECLSS wired to a
     cabin whose air is *not* 9500 mol, and the capacity stays 9500 wherever an ECLSS reaches.
     ⚠ But the condition was **already true when the trigger was written**:
     `scenarios/bioregenerative_station.yaml` has wired `eclss.o2_makeup` to an 8-mol cabin
     since 2026-08-11, and the 2026-09-06 commit that added the warning is the one that
     falsified that habitat's own stated fixed point (9.76275, the arithmetic of the old
     setpoint 10.0). Nothing went red because nothing in `rust/crates` runs the authored
     scenarios.

**What B named on its way out — three successors, none of them scheduled:**

* **A saturation bound on the chamber's gas-phase water.** B's finding 2, and the strongest
  result it produced: all three chambers hold the **same 536.995 mol** of vapour (identical to
  8e-16) regardless of room size, so wet pressure reaches 1.537 against a saturation-implied
  ceiling near 1.023 — ~20× what physics allows, and **room-independence is what proves it is
  the water model's defect rather than a scenario's sizing**. Held by a labelled tripwire that
  is meant to redden when this is fixed. This is now the biggest known physical defect in the
  tree.
* **The `o2_setpoint` mole-fraction conversion (B's slice 4), now BLOCKED behind an authoring
  decision.** It is bit-neutral (`0.21 × 9500.0 == 1995.0` exactly, verified both directions)
  but cannot ship as designed: the frozen params reach every authored `eclss.o2_makeup`, so
  putting the cabin capacity there hands 9500 to a habitat that is not 9500 — relocating the
  silent default rather than closing it. The honest version is an **authoring grammar change**,
  a second unfreeze, and its own item.
* **The station census row for a cabin gas band** — the predecessor deferred this to "after B,
  when the quantity it would freeze is the one B leaves behind". B leaves `pressure_ratio`
  behind, and finding 2 says freezing the **wet** one would freeze a defect. The row should
  name the **dry** total, which B measured to be structurally incapable of drifting.

The **product track** remains dormant and remains the standing candidate now that B is built;
its 2026-08-13 re-open condition is still met and nothing here consumes it.

**Not recommended:** reopening the citation bucket wholesale; any value move on the FvCB
constants before the page check; a `--write` of a transcendental golden from a Linux box;
building the oxygenation half of the O₂ form without the Γ* half (measured to be the one
change that makes the jar's band *tighter*).

---

## 7. Rules for this doc

- It is a **plan**, not a record. A finished item earns the normal three (index line, pointer
  row, record file) and *leaves* this doc — struck in place with the record named, never
  silently deleted.
- **Records refer to it as "the third direction plan", never by filename.**
- **The re-read marker moves only with a re-read.** The gate cannot tell the difference; the
  reader of the diff can.
- **Supersede, do not strike a fourth time** — and the trigger is now a measurement, not a
  judgement: when the bytes inside strike-through spans pass half the document, write the
  successor, move `DIRECTION_PLAN` in `repo_gates`, and banner this one. The predecessor
  reached **71.6 %** before anyone counted.
