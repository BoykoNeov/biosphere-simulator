# A real atmosphere — total gas conserved, pressure derived, the breach that takes everything

**Written 2026-09-08.** This is **B**, decided (not proposed) by the user at the close of the
perturbation-suite item: *"ok A now, but immediately after that (next session) B"*. A built the
instrument (`rust/crates/domains/tests/gas_composition_perturbations.rs`, 11 probes); this plan
builds the subject.

Named in records as **the atmosphere plan**. Its predecessor is *the perturbation-suite plan*.

---

## 0. ⚠ The decided framing is WRONG in one word, and the word is load-bearing

Both the plan that decided B and the memory that recorded it say:

> *"the leaf reading **partial pressures** rather than mole fractions **over a constant**."*

The second half of that sentence is a misdiagnosis, and building it would be worse than
building nothing. The derivation, in full, because it inverts the work:

At fixed volume and temperature — and V and T **are** constants in this model, there is no
volume state and no cabin thermodynamics — the ideal gas law gives each species

```
p_i = n_i · R·T / V
```

so the partial pressure of a gas is proportional to **its own mole count alone**. The
reference-pressure ratio the FvCB constants are calibrated against is therefore

```
p_i / P_ref  =  (n_i · RT/V) / (n_ref · RT/V)  =  n_i / n_ref
```

where `n_ref` is *the number of moles that fill this chamber at reference pressure* — a
**property of the room**, not an inventory. That is a constant, and it is exactly what
`chamber_air_mol` already is.

**So the existing arithmetic is right and the existing NAME is wrong.** `n_i / n_total_live`
is the mole *fraction*, and a leaf that read it would be **blind to depressurization**: vent
half the atmosphere at fixed composition and every mole fraction is unchanged, so a
fraction-reading leaf sees nothing. The fraction-reading model is the one that cannot
represent a hull breach. Pointing the denominator at a live total would move this tree from
"right for an unstated reason" to "wrong", and — this is the part that makes it dangerous —
**every gate would stay green**, because at charge `n_total == n_ref` and the nominal
goldens cannot tell the two denominators apart.

**What is actually missing is therefore not the denominator. It is that nothing conserves
the air**: there is no inert gas, so the numerators can never move together, a leak can only
ever take one gas at a time, and total pressure is not a quantity that exists.

⚠ This correction is the first deliverable. A future reader who finds `chamber_air_mol` next
to a live N₂ stock will "fix" the denominator, and nothing in the suite will stop them. The
rename in slice 1 exists to be that stop.

## 1. The design

### 1.1 The denominator is a room property — say so in the name (slice 1)

`SeasonScenario.chamber_air_mol` → **`chamber_air_capacity_mol`**; the flow context's
`chamber_air_mol: Option<f64>` → **`air_capacity_mol`**. Pure rename plus doc comments
carrying §0's derivation. No arithmetic touched.

### 1.2 The inert gas is a STOCK; total gas and pressure are DERIVED (slice 2)

**Not a total-gas stock.** A stock whose value must equal the sum of other stocks is a
redundancy that can drift, and this engine's conservation gate cannot police it because it is
not independent. Instead:

* one new POOL stock, `biosphere.chamber_inert`, in the sealed chamber's atmosphere
  compartment;
* total gas and pressure are **computed** from the species, so they are correct by
  construction and nothing can desynchronize them.

**Currency.** The inert fill of a spacecraft cabin is N₂ (+ Ar). `Quantity::Nitrogen` already
exists, its canonical unit is **kg**, and its composition convention is *canonical units of
each quantity per unit of stock amount* — so the stock is `quantity: nitrogen, unit: kg,
composition {nitrogen: 1.0}`, mirroring the water-vapour stock's pattern exactly. No new
`Quantity` member, so `ASSERTED_QUANTITIES`, the canonical sort order, the snapshot schema and
every `simcore` frozen set are untouched.

⚠ **Checked, not assumed:** conservation's tolerance is `atol + rtol·scale` where `scale` is
the *transfer magnitude* (`max |per-stock Δ·coeff|`), **not** the stored total
(`simcore/src/conservation.rs:97-122`). So parking ~200 kg of inert nitrogen beside a plant
carrying grams does **not** loosen the nitrogen ledger's tolerance while the pool sits still.
It does loosen it *during a breach*, which is proportionate and correct.

⚠ **The model has no nitrogen fixation, so this pool is unreachable from the plant.** That is
a true statement about a wheat/potato habitat, not a gap: neither crop fixes N₂. Recorded so
it is not later mistaken for an omission.

**The charge is DERIVED, not a new cited constant:**

```
inert_mol_0 = chamber_air_capacity_mol − chamber_co2_mol0 − chamber_o2_mol0
```

so the chamber starts at **exactly** reference pressure by construction, and no scenario gains
a number anyone has to defend. It lumps N₂ with Ar, which is what "the rest of the air" means.

| scenario | capacity | CO₂ | O₂ | inert balance |
|---|---|---|---|---|
| `sealed_chamber` (O₂-poor jar) | 1000 | 0.357 | 2.0 | 997.643 |
| `perennial_chamber` | 1000 | 0.357 | 210.0 | 789.643 |
| `consumer_chamber` | 2000 | 0.714 | 420.0 | 1579.286 |
| `greenhouse_bio` (station) | 9500 | 3.796 | 1995.0 | 7501.204 |

**Water vapour is counted in the total.** `biosphere.water_vapor` is already an atmosphere
stock, and vapour occupies volume like any other gas. Excluding it would be a silent physics
error in the one quantity this work exists to create. Every scenario charges `water_vapor0 =
0.0`, so the charge table above is exact; during a run the vapour rises and **pressure with
it**. ⚠ That excursion is a MEASUREMENT owed by §3, not a number this plan predicts away, and
if it is absurd that is a finding about the water model — recorded, not fixed here.

**Readouts, not auxiliaries.** `total_gas_mol`, `dry_gas_mol` and `pressure_ratio` land in
`biosphere::readouts` beside `min_ppm`/`peak_w`. (`dry_gas_mol` was added during the build, not
planned — see finding 2 in §3 for what forced the split.) No flow reads them, and growing the frozen
`aux_set` for a quantity with no live consumer is precisely the inert-gate failure this repo
has recorded four times. ⚠ Stated as a scope decision, not an oversight: **pressure is an
observable here, not a driver.** Making it a driver needs a consumer (a crew hypoxia response,
a structural event) and that consumer is not in scope.

### 1.3 The breach takes everything, at one rate (slice 3)

`LeakFlow` exists; `LEAK_SINK` is a single constant, so leaking two pools collides on the sink
id — the reason the predecessor deferred E3. Give the breach a per-pool sink
(`boundary.breach_sink.<pool>`; as built it is a new composer rather than a change to
`LEAK_SINK`, so the station's single-pool leak is untouched), then vent every gas stock.

The rate law falls out and needs no new parameter: a well-mixed chamber venting to vacuum
loses each species in proportion to **its own partial pressure**, i.e. first-order in `n_i`
with the **same** `k` for every species. One constant, and composition is preserved
automatically — the breach *is* E2 arrived at rather than applied by hand.

Diagnostic only: no golden, no manifest, no unfreeze (the perturbation precedent).

### 1.4 The trigger the params wrote for themselves — slice 4, **DEFERRED, and why**

`eclss.yaml`'s `o2_setpoint` source string, written 2026-09-06, two days before B was decided:

> *"⚠ TRIGGER — this value silently encodes the ONE cabin air inventory within O2Makeup's
> reach (9500 mol) … If an ECLSS is ever wired to a cabin whose air is not 9500 mol, this
> MUST become a mole fraction and the flow MUST read the air."*

The intended slice: `o2_setpoint: 1995.0 mol` → `o2_setpoint_fraction: 0.21` plus a cabin air
capacity, with `O2Makeup` computing `fraction × capacity`. It is **bit-neutral by a luck that
was checked before the edit rather than after**: `0.21 * 9500.0 == 1995.0` exactly in IEEE-754
(`0x1.f2c0000000000p+10` both ways), and `1995.0 / 9500.0 == 0.21` exactly too.

⚠ **Correction to this plan's first draft, which claimed "B trips exactly that trigger".**
It does not. The trigger's condition is *an ECLSS wired to a cabin whose air is **not** 9500
mol*; under this design the capacity stays 9500 wherever an ECLSS reaches, and the regulator's
arithmetic is untouched. What B actually does is make the encoded 9500 newly **dangerous** — an
inert stock now sits beside it, so a future reader has a live total to point the setpoint at.
That is a real reason to do the slice. It is not the reason the trigger states, and committing
the wrong one would have been this repo's own recorded failure mode (*a recorded blocker is
dated*).

⚠⚠ **And then the check that decides the slice came back dirty — with a finding.** The design
was to put the capacity in `eclss.yaml` rather than on `CabinScenario`, because `O2Makeup` is
built in five places and one of them, `authoring/src/flow_registry.rs:300`, has authored specs
carrying **wiring ids and frozen params only, no numeric fields**. But params are handed to
*every* authored `eclss.o2_makeup`, so that route hands 9500 to an authored habitat whose cabin
is not 9500 — the same silent default, relocated. Whether that matters is one grep, and the
grep says:

* `scenarios/bioregenerative_station.yaml:373` **does** wire `eclss.o2_makeup`, with
  `params: eclss` — the frozen bundle.
* Its cabin is **not** a 9500-mol cabin. `cabin.o2` starts at **8.0 mol**, and the file's own
  direction-gate reasoning states the regulator *"approaches 9.76275 MONOTONICALLY FROM
  BELOW"*. That fixed point is `10 − 0.14235/0.6 = 9.76275` **exactly** — it is the arithmetic
  of `o2_setpoint = 10.0`.
* The habitat was authored **2026-08-11** (`00c3d9a`). The setpoint became 1995.0 on
  **2026-09-06** (`147527b`).

**So the trigger's condition was already true on the day the trigger was written.** The commit
that added the warning *"if an ECLSS is ever wired to a cabin whose air is not 9500 mol"* was
the same commit that broke the one ECLSS already wired to such a cabin, and left that habitat's
own stated fixed point falsified. Nothing went red, because **nothing in `rust/crates` runs the
authored scenarios at all** — they are runtime-only content by the project's own rule
(*"authored ≠ validated"*), which is exactly the blind spot that let a conditional warning be
written about an event that had already happened.

**Therefore slice 4 is deferred as its own item**, and its scope is now larger than a param
edit: it must decide what an authored habitat's cabin capacity is, which is an **authoring
grammar change and a second unfreeze**. Landing it as designed would have moved the defect
rather than closed it. The stale habitat reasoning is a separate, older defect, recorded rather
than fixed here — this item's charge was the atmosphere.

## 2. PREDICTIONS — written before any code, to be scored in §3

**Numbered so a miss cannot be quietly reworded. §3 is written only after the runs.**

1. **Slice 1 is bit-neutral.** Zero bytes change in any of the 21 goldens; `cargo test` and
   `cargo clippy -D warnings` green with no other edit. A rename that moved a number would
   mean the name was load-bearing somewhere it should not be.
2. **Slice 2 moves the goldens in exactly ONE way.** Every **sealed** state golden gains
   exactly **one** stock entry (`biosphere.chamber_inert`, amount = the table above, constant
   across the whole run) and **not one existing hex-float changes**. `season_euler_state.json`
   (the open field) is byte-identical, because the open field builds no chamber.
   ⚠ This is the two-direction control: a diff that also moves an existing value is a finding
   about the wiring, and a diff that adds nothing means the stock was never built.
3. **Photosynthesis is gas-neutral.** CO₂ + H₂O → CH₂O + O₂ consumes one mole of gas and
   produces one, so with vapour held out, `n_CO2 + n_O2` is conserved by assimilation to
   within the conservation tolerance. This is the claim that makes an inert stock *sufficient*
   — if it fails, a total-gas stock would have been needed after all.
4. **Pressure is not flat in a nominal run**, and it rises, because transpiration puts vapour
   into a fixed volume and `water_vapor0 = 0`. Magnitude **unpredicted** — this is the one
   number I am not calling in advance, and §3 reports it whatever it is.
5. **The breach is composition-preserving.** Under one first-order `k` on every gas, mole
   fractions are invariant to ≤ 1e-12 relative while total gas falls, and interior + sink is
   conserved every step.
6. **The breach reproduces E0's fingerprint on a RUN.** As pressure falls at fixed
   composition: the light-limited branch is unchanged (homogeneous of degree zero) and the
   Rubisco-limited branch falls (`Kc` is a constant term). ⚠ Written against `n_ref`
   explicitly — a test written against the live total cannot tell "correct" from "blind".
7. **Slice 4 is bit-neutral, and it is bit-neutral by LUCK that has been checked.**
   `0.21 * 9500.0 == 1995.0` **exactly** in IEEE-754 (`0x1.f2c0000000000p+10` both ways), and
   the round trip `1995.0 / 9500.0 == 0.21` exactly too. Verified before the edit, not after.
   So no ECLSS/station golden moves. ⚠ If it had *not* been exact, slice 4 would have been an
   unfreeze of every cabin golden and would have needed its own decision.
8. **Finding 4 of the predecessor gets easier to reach, not harder.** Halving the jar's O₂
   already takes it anoxic with no backstop firing and no event. A breach scales *all* species,
   so the breach scenarios are checked for well-posedness before any output of theirs is read
   as science.

## 3. RESULTS

**Slices 1, 2 and 3 BUILT. Slice 4 DEFERRED (§1.4).** Scored against §2, in order.

1. **Slice 1 bit-neutral — HELD.** The rename touched 6 files and 32 sites; zero bytes changed
   in any of the 21 goldens (`git status` on `rust/data/golden` empty), 802 tests green before
   any other edit.
2. **Slice 2's golden diff — HELD EXACTLY, and this was the control.** 9 of 20 goldens changed
   and **every one is `+13 −0`**: one `biosphere.chamber_inert` stock entry, no deleted line,
   no existing hex-float touched anywhere. `season_euler_state.json` (the open field) is
   byte-identical, as are `drift_summary` and `sealed_energy_drift_summary`. The four charges
   match §1.2's table to the bit — the greenhouse's `0x1.a444b98cc807cp+7` is exactly
   `(9500 − 3.796 − 1995) × 0.0280134` kg. 9 manifest `golden_sha256` lines followed and
   **nothing else in either manifest** moved: no flow, no aux, no param file.
3. **Photosynthesis is gas-neutral — HELD**, and it is the claim that made the design right.
   `n_CO₂ + n_O₂` is conserved to ≤ 1e-12 relative across all three chambers at their own
   horizons. Had it failed, a total-gas stock would have been needed after all.
4. **Pressure is not flat in a nominal run — HELD, and the magnitude is a FINDING.** Predicted
   direction (up, from transpiration); magnitude deliberately not predicted. Measured: the
   1000-mol jars peak at **1.5370**, the 2000-mol consumer chamber at **1.2685**.
   ⚠⚠ Their vapour loads are the **same number** — 536.9950225921074 / …83 / …69 mol, identical
   to 8e-16 relative across a room of twice the size. So the vapour is set entirely by the
   plot's transpiration and is **uncoupled from the air it enters**; saturation at 20 °C would
   cap a chamber near **1.023**, so the jars hold ~20× what physics allows. Room-independence
   is what makes this a defect of the **water model** and not of any scenario's sizing — and
   one chamber could not have distinguished those. Recorded as a labelled tripwire, not fixed.
   ⚠ The companion result: the **dry** total is the capacity for the whole run on every
   chamber, so *a sealed chamber's dry pressure is structurally incapable of drifting here.*
5. **The breach is composition-preserving — HELD, but not the way §2 said.** The first version
   asserted the CO₂:O₂ ratio holds across the window and **failed at 1.5e-2** — the test's
   fault, not the breach's: photosynthesis and respiration write both gases throughout, so
   their ratio moves for reasons that have nothing to do with venting. Measured where it is
   measurable: over the **first active step**, where the breached and baseline arms are still
   bit-identical, every one of the four species loses exactly `k·dt = 5e-3` of itself
   (agreement to ≤ 1e-13). ⚠ Two neighbouring test premises were wrong the same way and both
   are recorded in the file rather than quietly repaired: "interior + sinks is constant" is
   false because vapour enters the gas phase from soil water (it failed at 6.7e-1), and "CO₂
   falls across the breach" is false because the unbreached jar's CO₂ *rises* over that window
   anyway. A vent's effect is only visible **against a baseline**.
6. **E0's fingerprint on a run — HELD.** At the pressure ratio the breach actually reached, the
   light-limited branch is unchanged to ≤ 1e-12 relative and the Rubisco-limited branch falls.
7. **Slice 4 bit-neutral — VERIFIED but NOT LANDED.** The IEEE-754 exactness was confirmed
   before the edit; the slice was then deferred for the reason in §1.4, which is a finding
   rather than a scheduling choice.
8. **The anoxia path — CHECKED FIRST, not tripped.** `the_breached_run_is_well_posed` runs
   before anything in that file is read as science: 0 backstop firings, 0 events, both gas
   pools positive throughout.

**One thing no prediction covered.** The breach composer's first home,
`biosphere/perturbations.rs`, tripped `tests/one_assembly_body.rs` — the spine must assemble a
season in exactly one place, and the composer's `into_parts → append → Registry::new` reads as
a second assembly body to a text scan. The gate's own remedy line is *"the fix is never to
widen this list"*, so the composer moved to `domains::breach`, beside the spine, which is where
`lab::mechanism`, `ulp_probe` and `station::perturbations::with_station_leak` already do the
same recomposition. **The gate was right and the placement was wrong** — worth recording
because the tempting reading was the opposite one.

## 4. Scope — what is deliberately not here

* **No pressure-driven mechanism.** Pressure is a readout (§1.2). A crew hypoxia response or a
  structural depressurization event would make it a driver, and each is its own item with its
  own consumer to justify it.
* **No new `Quantity`.** The inert gas rides `NITROGEN`, which costs `simcore` nothing.
* **No nitrogen fixation.** The inert pool is unreachable from the plant, and for wheat and
  potato that is correct.
* **No station science-gate row.** The predecessor deferred *"the station census carries no CO₂
  or cabin-oxygen band"* to after B on the grounds that the quantity it would freeze should be
  the one B leaves behind. B leaves `pressure_ratio` behind; promoting it to a census row is
  the natural successor slice and is still a separate ceremony.

## 5. Filing

Index line + pointer row + `docs/log/atmosphere.md` + a memory file. Nothing in `CLAUDE.md`.
