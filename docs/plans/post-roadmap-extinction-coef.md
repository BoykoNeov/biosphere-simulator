# Binding `extinction_coef` — the decision the canopy-provenance item left open

**Taken 2026-09-06.** The user's answer is **0.60**, which is the value the tree already
carries, so this is a **provenance-only unfreeze**: one `source:` string, one manifest hash,
no number, no golden. The measurement it rests on was made on 2026-08-15
(`docs/log/canopy-provenance.md`), re-checked against a run on 2026-09-04, and re-measured
from scratch for this build on 2026-09-06 — see §3, which is the column the record quotes.

Predecessor plan: `post-roadmap-canopy-provenance.md`, which built `carbon_fraction` and
left this half explicitly **measured-not-built**, *"the decision is the user's and is
untaken."* Filed under the September direction plan (§2.1 item 2 there).

---

## 1. What is being decided, in one paragraph

`extinction_coef` is the `k` of Beer–Lambert light interception: how fast light is used up
going down through the canopy. The tree has carried `0.6` since Phase 1 as a `TODO(cite)`
placeholder — a number chosen as literature-typical, never bound to a page. The shelf holds
**three** readings for wheat and they disagree: 0.60, 0.65, 0.68. The decision is not what
the number should be — it is **which of the three the file is allowed to say it came from**,
and 0.60 is the one that is (a) primary literature, (b) already in this file as source [B],
and (c) the conservative side of every gate.

## 2. Why 0.60 and not 0.65 or 0.68

- **0.60** — [B] Penning de Vries et al. (1989) p. 36: *"about 0.6 for a canopy with erect
  leaves and 0.8 for one with horizontal leaves (Goudriaan, 1977)"*. An **architecture-class**
  PAR coefficient; winter wheat is erect-leaved. [B] already sources `specific_leaf_area` in
  the same file, and Goudriaan wrote the depth quadrature this domain's canopy integrates
  over — so the binding is coherent with the tree's own light path, not merely available.
- **0.65** — Soltani & Sinclair (2012) Table 10.1, the "Wheat" row, for the *identical*
  equation. Crop-specific, and therefore the tempting one. ⚠ Its own caption lists [B] among
  its sources, so it is **partly derived** rather than an independent measurement — which is
  why "crop-specific beats architecture-class" does not settle it.
- **0.68** — ibid. Fig. 10.8a, wheat cv. Zagros, explicitly **unpublished data**. It cannot
  retire a `TODO(cite)` in a tree whose rule is *cite the primary literature*, and §3 adds a
  second, harder ground: it is **red on a gate**, and **non-monotone**.

⚠ **The two risks are opposed, and the file records that rather than resolving it.** On
provenance, 0.60 may be ~8 % low against the crop-specific reading. On the gates, 0.60 is the
**conservative** side — it is the alternatives that spend the liveness floor. No direction is
safe on both axes. *A citation that conceals a live alternative is worse than the `TODO(cite)`
that admitted one.*

## 3. The ladder, re-measured for this build (2026-09-06, not quoted from a record)

`cargo run --release -q -p domains --example value_switch -- extinction_coef=0.60,0.65,0.68 --long`

| row (bound as recorded) | frozen | **0.60** | 0.65 | 0.68 |
|---|---|---|---|---|
| `open_season` peak LAI (`5.0 < peak < 8.0`) | 6.022837 | **6.022837** | 6.069990 | 6.058617 |
| `open_season` peak W (`< 14.4248`) | 13.379084 | **13.379084** | 13.552334 | 13.604833 |
| `sealed_chamber` low CO₂ (`> 61.07`) | 71.435803 | **71.435803** | 72.755378 | 73.504418 |
| `perennial_chamber` low CO₂ | 70.252606 | **70.252606** | 71.593623 | 72.402275 |
| `consumer_chamber` low CO₂ | 73.338613 | **73.338613** | 73.908835 | 74.671389 |
| `perennial_long_horizon` peak-leaf (`> 0.55`) | 0.578137 | **0.578137** | 0.552202 | **0.538913 — RED** |

**The 0.60 column is the control, and it is bit-identical to frozen in all 8 rows** (the
report's own summary line: *"0 rose, 0 fell, 8 did not move"*). That is the measurement that
makes this a provenance edit rather than a calibration — and it is exactly the check the
2026-08-15 `specific_leaf_area` edit did not have, which *looked* like provenance and moved
every golden by +7 %.

Two facts about the alternatives that are **not** in `docs/log/canopy-provenance.md`'s summary
and were confirmed here:

* **0.68 breaks the perennial liveness floor** (0.538913 against `> 0.55`). It is not merely
  unpublished — it is red.
* **0.68 is non-monotone on the observable it was proposed to move**: its peak LAI (6.058617)
  is *below* 0.65's (6.069990). More interception per unit leaf is a faster draw on a fixed
  carbon inventory, so past some `k` the canopy ends smaller rather than larger.

⚠ The **converged** (step-limit) half of the 0.65 pricing (+7.3 % of peak LAI) is *not*
re-measured here; it needs the step sweep, not this harness. It is not load-bearing for a
decision that keeps the shipped value.

## 4. The ceremony (`docs/biosphere-reference.md`, "The unfreeze discipline")

1. **Justify + review** — this doc; advisor-reviewed before any edit.
2. **Make the change boundary-side** — `git diff rust/crates/simcore/` stays empty.
3. **Regenerate the affected goldens** — *predicted: none.* `regen_goldens` must report
   20 of 20, 0 would change.
4. **Regenerate the manifest** — *predicted: exactly one line*,
   `param_files["canopy.yaml"]`. ⚠ Predict before running: C4 regenerated corrupted prose
   into a contract with every gate green, and the prediction is what caught it.
5. **Report the science gates** — §3 is the report; nothing moved.
6. **Record provenance** — this doc, the unfreeze log, `docs/log/extinction-coef-bound.md`.
7. **Re-run the gates** and commit.

## 5. Scope — the three stale sentences this commit may fix, and the two it may not

The September direction plan §3.2 lists four "left deliberately" stale sentences, each to be
corrected *"the next time that file's manifest entry moves for a real reason."* **This is that
time for `canopy.yaml`, and only for `canopy.yaml`:**

- ✔ `canopy.yaml`: *"which the manifest records and no test can see"* — falsified by slice C7
  on 2026-08-18, three days after it was written.
- ✔ `canopy.yaml`: *"It is the live provenance queue, alone"* — that queue empties here.
- ✔ `canopy.yaml`: the **~3.5× amplifier**, which does not apply to `k` and was refuted in the
  same record that priced this binding (+8.3 % on `k` buys +0.78 % of peak LAI, not +30 %).
  It was a live false claim about the very parameter being bound.
- ✗ `senescence.yaml`'s `shade_rate` note and `self_discharge.yaml`'s pointer — **left
  alone.** Different files, their manifest entries do not move here, and §3.2 lists them
  precisely so nobody fixes them into an unfreeze.

**And one file outside the freeze, which is why it is free:** `crops/potato/canopy.yaml`
described the wheat value as *"the reference's own TODO(cite) placeholder"*, which this commit
falsifies. The crop overrides are excluded from the manifest census by construction
(non-recursive, slice C8), so correcting them is an ordinary edit. ⚠ The correction **inverts**
the sentence's force: [B] p. 36 assigns 0.6 to *erect* leaves and 0.8 to *horizontal* ones, so
potato's recorded low bias stops being a guess about a placeholder and becomes a **cited**
mismatch of architecture class. Rule applied, from the potato item: *check which side of the
freeze a stale sentence is on before pricing its correction.*

## 6. What this does NOT do

- It does not move a number, a golden, a band or a floor.
- It does not close §2.1 item 1 (what leaf population [B] Table 19's figure describes) — a
  different retrieval, on a different parameter.
- It does not endorse 0.60 as the best estimate of wheat's `k`. It endorses it as the reading
  this tree can **cite**, with the disagreement recorded in the file rather than buried.
