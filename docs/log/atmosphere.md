## **The habitat gets an atmosphere** (the decided framing was wrong in one word, and the trigger it was taken on had already been tripped)

**BUILT 2026-09-08.** Plan: `docs/plans/post-roadmap-atmosphere.md`. Predecessor:
`docs/log/perturbation-suite.md`, whose eleven probes were built to be this item's instrument.

This is **B**, decided by the user at the close of the perturbation item — *"ok A now, but
immediately after that (next session) B"* — and taken this session on *"That one carries the
full ceremony, and it trips a trigger a parameter file wrote for itself two days ago. — work on
it"*. Slices 1–3 landed with the full unfreeze ceremony. **Slice 4 is deferred, and the reason
is itself the sharpest finding here.**

---

## ⚠ The decided framing was wrong in one word, and the word inverted the work

Both the plan that decided B and the memory recording it said the leaf should read *"partial
pressures rather than mole fractions **over a constant**."* The second half is a misdiagnosis,
and building it would have been worse than building nothing.

At fixed volume and temperature — and V and T *are* constants in this model; there is no volume
state and no cabin thermodynamics — the gas law gives `p_i = n_i·R·T/V`. A species' partial
pressure is proportional to **its own mole count alone**, so

```
p_i / P_ref  =  (n_i·RT/V) / (n_ref·RT/V)  =  n_i / n_ref
```

where `n_ref` is the moles filling the room at reference pressure — **a property of the room**,
which is exactly what the old `chamber_air_mol` constant already was. `n_i / n_total_live` is
the mole *fraction*, and it does not move when a chamber depressurizes at fixed composition: a
leaf reading it would be **blind to a hull breach**, the precise opposite of what B was for.

**So the arithmetic was right and the name was wrong.** And the substitution is invisible to
every gate in the tree: at charge `n_total == n_ref`, so no golden can tell the two denominators
apart. That is why slice 1 is a rename (`chamber_air_capacity_mol`, `air_capacity_mol`) carrying
the derivation at the scenario field and at all three science functions that divide by it — the
rename exists to be the stop a future "fix" runs into.

**What was actually missing was that nothing conserved the air.** No inert gas, so the
numerators could never move together, a leak could only ever take one species, and total
pressure was not a quantity that existed.

## What was built

**Slice 1 — the rename.** 6 files, 32 sites, bit-neutral: zero bytes changed in any of the 21
goldens, 802 tests green before any other edit.

**Slice 2 — the inert fill (the unfreeze).** One POOL stock `biosphere.chamber_inert` per sealed
chamber — `NITROGEN`, kg, 1:1 default composition, exactly the water-vapour stock's shape.

* **Deliberately NOT a total-gas stock.** A stock that must equal the sum of other stocks is a
  redundancy the conservation gate cannot police, because it is not independent. Total gas and
  pressure are **folded** from the species (`readouts::total_gas_mol` / `dry_gas_mol` /
  `pressure_ratio`), so they cannot desynchronize from what they summarize.
* **The charge is derived**, `capacity − CO₂ − O₂`, so every chamber starts at exactly reference
  pressure by construction and no scenario gains a number anyone must defend.
* **No new `Quantity`** — `simcore` untouched, `ASSERTED_QUANTITIES` and the snapshot schema
  unmoved. ⚠ Checked rather than assumed: conservation's tolerance scales with the *transfer*
  magnitude, not the stored total, so ~200 kg of parked inert nitrogen does not loosen the
  nitrogen ledger while it sits still.
* ⚠ The pool is **unreachable from the plant** — there is no nitrogen fixation, and neither
  wheat nor potato fixes N₂. Recorded so it is not later filed as a missing edge.

**Slice 3 — the breach (diagnostics, no golden).** `domains::breach::with_hull_breach` vents
every gas to **its own** boundary sink, first-order at one rate. One `k` is the physics, not a
simplification: a well-mixed volume venting to vacuum loses each species in proportion to its
own partial pressure, so `n_i(t) = n_i(0)·e^{-kt}` for all species and composition is preserved
with nothing tuned. The predecessor had deferred a two-gas leak because `LEAK_SINK` is a single
constant and four legs into one sink would need that sink to carry four compositions.

## Predictions, scored

Written before any code. **All held**; two held in a form the predictions had wrong.

**The golden control — the one that mattered.** Predicted: *every sealed golden gains exactly
one stock entry, not one existing hex-float changes, the open field is byte-identical.*
Measured: **9 of 20 goldens changed and every one is `+13 −0`.** Five biosphere
(`sealed_chamber`, both perennial, both consumer) and four station (`greenhouse`, `lighting`,
`harvest`, `sealed_station`); `season_euler_state.json`, `drift_summary` and
`sealed_energy_drift_summary` untouched. The charges match the plan's table to the bit — the
greenhouse's `0x1.a444b98cc807cp+7` is exactly `(9500 − 3.796 − 1995) × 0.0280134` kg. Nine
manifest `golden_sha256` lines followed and **nothing else in either manifest**: no flow, no
aux, no param file.

**Gas neutrality — the claim that made the design right.** `n_CO₂ + n_O₂` is conserved to
≤ 1e-12 across all three chambers, because photosynthesis takes one mole of gas and returns one
(PQ = 1) and every respiration runs it backwards. That is what makes ONE inert stock
*sufficient*; had it failed, a total-gas stock would have been needed after all.

## Findings

**1. The dry chamber never leaves reference pressure — structurally.** The reactive pair
self-cancels and the inert fill is written by nothing, so the dry total *is* the capacity for
the whole run on every chamber. ⚠ The consequence bounds what this model can say: **a sealed
chamber's dry pressure is structurally incapable of drifting here.** A slow pressure loss — the
thing a real habitat's leak-rate budget is about — has no representation in the nominal tree at
all; it takes an explicit breach.

**2. ⚠⚠ The chamber's gas-phase water obeys no saturation law, and counting vapour as a gas is
what exposed it.**

| chamber | capacity (mol) | peak wet pressure | peak vapour (mol) |
|---|---|---|---|
| `sealed_chamber` | 1000 | 1.5369950225921074 | 536.9950225921074 |
| `perennial_chamber` | 1000 | 1.5369950225921083 | 536.9950225921083 |
| `consumer_chamber` | 2000 | 1.2684975112960535 | 536.9950225921069 |

The three vapour loads are the **same number**, identical to 8e-16 relative, in rooms of 1000
and 2000 mol — the pressures differ only because the same vapour is divided by a different room.
So the vapour is set entirely by the plot's transpiration and is **completely uncoupled from the
air it enters**. Saturation at 20 °C caps a chamber near **1.023**; the jars hold ~20× what
physics allows and would have been raining long before.

⚠ **Room-independence is what makes this the water model's defect rather than a scenario's
sizing**, and *one chamber could not have distinguished those* — the second and third chambers
were measured on the advisor's point, and they are what turned a threshold into a diagnosis.
Recorded, **not fixed**: a saturation bound is a water-science change with its own ceremony, and
this item's charge was the atmosphere. Held by a **labelled tripwire** — a test meant to redden
when the water model is corrected, which must be deleted rather than re-thresholded.

**3. ⚠⚠ The `o2_setpoint` trigger's condition was already true on the day the trigger was
written — and that is why slice 4 is deferred.**

The param file's own warning, added 2026-09-06: *"If an ECLSS is ever wired to a cabin whose air
is not 9500 mol, this MUST become a mole fraction and the flow MUST read the air."*

* B does **not** meet that condition — the capacity stays 9500 wherever an ECLSS reaches. What B
  does is make the encoded 9500 newly *dangerous*, because an inert stock now sits beside it for
  a reader to mistakenly point at. The plan's first draft claimed the trigger was tripped; that
  justification was false and was rewritten before landing.
* But `scenarios/bioregenerative_station.yaml:373` **already wires `eclss.o2_makeup` with
  `params: eclss`**, to a cabin whose `cabin.o2` starts at **8.0 mol**. That file's own
  direction-gate reasoning says the regulator *"approaches 9.76275 MONOTONICALLY FROM BELOW"* —
  and `10 − 0.14235/0.6 = 9.76275` **exactly**, the arithmetic of `o2_setpoint = 10.0`.
* The habitat was authored **2026-08-11** (`00c3d9a`); the setpoint became 1995.0 on
  **2026-09-06** (`147527b`).

**The commit that wrote the conditional warning is the commit that satisfied its condition**,
and it left the habitat's stated fixed point falsified. Nothing went red because **nothing in
`rust/crates` runs the authored scenarios at all** — runtime-only content by the project's own
*"authored ≠ validated"* rule, which is exactly the blind spot that lets a warning be written
about an event that has already happened.

Slice 4 therefore cannot ship as designed: putting the capacity in `eclss.yaml` hands 9500 to
that habitat too, relocating the silent default rather than closing it. The honest slice needs
an **authoring grammar change — a second unfreeze** — and is its own item.

## What no prediction covered

The breach composer's first home, `biosphere/perturbations.rs`, tripped
`tests/one_assembly_body.rs`: the spine must assemble a season in exactly one place, and
`into_parts → append → Registry::new` reads as a second assembly body to a text scan. The gate's
own remedy says *"the fix is never to widen this list"*, so the composer moved to
`domains::breach`, beside the spine — where `lab::mechanism`, `ulp_probe` and
`station::perturbations::with_station_leak` already recompose the same way. **The gate was right
and the placement was wrong**, and it is worth recording because the tempting reading was the
opposite one.

## Three test premises that were wrong, kept rather than repaired

All three failed on a correct breach, and each failure is recorded in the test that replaced it:

1. *"The CO₂:O₂ ratio holds across the breach window"* — failed at **1.5e-2**. Photosynthesis
   and respiration write both gases throughout, so the ratio moves for reasons unrelated to
   venting. A vent's composition-preservation is only measurable over an interval short enough
   that the biology is identical in both arms: **one step**, where all four species lose exactly
   `k·dt = 5e-3` (to ≤ 1e-13).
2. *"Interior + sinks is constant"* — failed at **6.7e-1**. Water vapour enters the gas phase
   from soil water and leaves to condensate, so total gas is not a closed quantity even without
   a breach. The general proof was never this file's to give: `simcore` asserts conservation
   every step, so a mass-destroying breach aborts the run.
3. *"CO₂ falls across the breach"* — false; the *unbreached* jar's CO₂ rises over that window
   (0.399 → 1.349) because respiration outruns the crop. **Every breach claim is a contrast
   against a `k = 0` arm**, which is the perturbation discipline this repo already had.

Two bit-equality claims were also over-stated and corrected to measured floors rather than
absorbed: `inert + sink` drifts 4 ULP (pool subtracts, sink adds, at different magnitudes), and
the dry total drifts to ~1e-12 by step 3186 of the 5-year run.

## Ceremony

Advisor-reviewed before the design and again before the regeneration. The golden diff was
predicted **in writing first**, and the two-direction control (`+13 −0`, open field untouched)
is what scored it. Goldens regenerated with `regen_goldens --write`; both manifests regenerated
with their own writers; the biosphere freeze doc has its unfreeze-log entry.

## Left open

* **Slice 4** — the setpoint as a mole fraction, now blocked behind an authoring grammar
  decision (finding 3).
* **The water saturation bound** — finding 2's named successor, and it now has an instrument.
* **The station census row** for a cabin gas band, which the predecessor deferred to "after B,
  when the quantity it would freeze is the one B leaves behind". B leaves `pressure_ratio`
  behind — and finding 2 says freezing the *wet* one would freeze a defect, so the row should
  name the dry total.
* **Pressure as a driver.** It is an observable here; making it drive anything needs a consumer
  (a crew hypoxia response, a structural event), which is a separate item.
