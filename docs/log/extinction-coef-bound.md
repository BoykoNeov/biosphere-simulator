## **`extinction_coef` is BOUND at 0.60** (the canopy's last uncited literal — and the citation is an ARCHITECTURE class, which strengthens it for wheat and REFUTES it for potato)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md), written
> under rule 4 of [`../context-budget.md`](../context-budget.md) — one file per work item.
> Plan of record: `post-roadmap-extinction-coef.md`. Filed under the September direction plan
> (referred to by that name here, never by filename — see the log's exemption note).

**DECIDED by the user 2026-09-06 ("0.60") and shipped the same day as a provenance-only
unfreeze:** one `source:` string, three stale header sentences, one manifest hash. **No number
moved, no golden moved, no band and no floor moved.** The honour-system ceremony was followed
in order — advisor review before the first edit, prediction before each regeneration, this
record after.

**FINDING 1 — the decision was between three citations, not three values, and the shelf
disagrees with itself.** `0.6` has been in the tree since Phase 1 as a `TODO(cite)`
placeholder. Bound now to **[B] Penning de Vries et al. (1989) p. 36** — *"about 0.6 for a
canopy with erect leaves and 0.8 for one with horizontal leaves (Goudriaan, 1977)"* — an
**architecture-class** PAR coefficient, and winter wheat is erect-leaved. The two rejected
readings are recorded in the file rather than dropped: Soltani & Sinclair (2012) Table 10.1
gives 0.65 for wheat under the *identical* equation but its own caption lists [B] among its
sources, so it is partly derived; Fig. 10.8a's 0.68 is explicitly **unpublished data**, which
cannot retire a `TODO(cite)` in a tree whose rule is cite-the-primary-literature.

⚠ **The two risks are OPPOSED, which is why the file records the disagreement instead of
closing it.** On provenance 0.60 may be ~8 % low against the crop-specific reading; on the
gates 0.60 is the **conservative** side, because it is the alternatives that spend the
liveness floor. *A citation that conceals a live alternative is worse than the `TODO(cite)`
that admitted one.*

**FINDING 2 — the control is the 0.60 column, and it is what makes this provenance rather
than calibration.** Re-measured for this build rather than quoted from the 2026-08-15 record:
`value_switch extinction_coef=0.60,0.65,0.68 --long` reports **"0 rose, 0 fell, 8 did not
move"** for the 0.60 column — bit-identical to frozen in all eight gated rows. That check is
exactly what the 2026-08-15 `specific_leaf_area` edit lacked: it *looked* like provenance,
and it moved every golden by +7 %.

**FINDING 3 — two facts about the alternatives that are not in the 2026-08-15 record's
summary.** Both confirmed on a run here, and both make the refusal of 0.68 harder than
"unpublished":

* **0.68 is RED on a gate.** `perennial_long_horizon`'s converged peak-leaf fixed point is
  **0.538913** against a recorded `> 0.55`. 0.65 clears it by 0.40 % (0.552202) where the
  shipped value clears by 5.12 % (0.578137).
* **0.68 is NON-MONOTONE on the observable it was proposed to move.** Its peak LAI is
  6.058617, *below* 0.65's 6.069990. More interception per unit leaf is a faster draw on a
  fixed carbon inventory, so past some `k` the canopy ends smaller rather than larger.

**FINDING 4 — the C7 gate fired, on the first provenance edit since it was written, and its
diff was one line as predicted.** `tests/manifest_writer.rs` went red on the edited tree —
`9 passed; 1 failed`, `line 163`, `param_files["canopy.yaml"]` and nothing else — before the
manifest was regenerated. ⚠ This is the sentence the file itself carried until today:
*"a provenance-only unfreeze, which the manifest records and no test can see."* True when it
was written on 2026-08-15, **false three days later** when C7 made the reference write its own
manifest, and unwatched ever since. It is now corrected in place, dated, with what falsified
it named. *A stale sentence in a frozen file is protected by the same freeze that protects the
value; it comes out when the file's manifest entry moves for a real reason.*

**FINDING 5 — the citation INVERTS the potato override's argument, and that correction was
free because the file is on the other side of the freeze.** `crops/potato/canopy.yaml`
described the wheat value as *"the reference's own TODO(cite) placeholder … reusing a
placeholder, not borrowing a wheat measurement"* and recorded a suspected low bias for a
broad-leaved canopy. The binding falsifies the first half and **promotes** the second: [B]
p. 36 assigns 0.6 to *erect* leaves and **0.8 to horizontal** ones, so potato's bias stops
being a guess about a placeholder and becomes a **cited** mismatch of architecture class. The
crop overrides are outside the manifest census by construction (non-recursive, slice C8), so
this was an ordinary edit and not a second unfreeze. *Check which side of the freeze a stale
sentence is on before pricing its correction* — the rule the potato item left, applied.

**FINDING 6 — a live FALSE claim came out with the stale ones, and it was about the parameter
being bound.** The file's header asserted that `extinction_coef` sat *"under a ~3.5×
amplifier"*, so that *"a ±2 % disagreement about k is roughly the whole margin of the
peak-LAI science band."* That amplifier is the elasticity to a **uniform** perturbation; `k`
raises absorption at the canopy top and lowers penetration to the base, so it self-cancels in
a closed canopy. Measured: **+8.3 % on `k` buys +0.78 % of peak LAI, not +30 %.** The claim
was refuted in the same 2026-08-15 record that priced this binding, and it survived in the
file for three weeks because nothing reads a header comment. ⚠ Had it been believed, it would
have argued *for* the value move this decision declined.

**What this does NOT close.** §2.1 item 1 — what leaf population [B] Table 19's figure
describes — is a different retrieval on a different parameter and stays open. This decision
does not endorse 0.60 as the best estimate of wheat's `k`; it endorses it as the reading this
tree can cite, with the disagreement kept visible in the file.

**Gates, run on the committed tree, counts read off the whole output and written after the
run.** `cargo test --workspace --no-fail-fast` from `rust/`: **1134 passed, 0 failed across 67
result lines** (4 ignored, the declared long-horizon set). ⚠ That is **exactly** the figure the
September direction plan re-measured this morning before any of this work — 1134 over 67 —
which is the check this project asks for: a provenance edit adds no test and breaks none, so
the total must not move at all. This is Windows; the Linux total is **one higher** by
construction, `regen.rs`'s `#[cfg(not(windows))]` ulp-only control, which is unreachable on the
goldens' own platform. `cargo clippy --all-targets -- -D warnings`: **clean**.
`cargo run --release -q -p station --example regen_goldens`: **20 of 20 run; 0 would change**,
every one reported `identical` (predicted: none would move). Manifest regeneration diff:
**one line**, `param_files["canopy.yaml"]`, exactly as predicted in the plan's §4 before it was
run — and the same one line the gate had named when it went red.
