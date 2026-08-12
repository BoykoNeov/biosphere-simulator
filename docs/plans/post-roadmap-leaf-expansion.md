# Sink-limited leaf expansion — the mechanism `WSFL` attaches to

**Taken 2026-08-12 on the user's call**, as the successor named by
`docs/log/water-stress-curves.md` finding 2: *"the successor is a **sink-limited
leaf-expansion phase** (node accumulator, `PHYL`, the `PLACON`/`PLAPOW` allometry, a
`tuTLM` boundary, and — the expensive part — leaf area as a STATE variable), **never
"the missing `WSFL` multiply"**."*

Probes: `M:/claud_projects/temp/leaf-expansion/` (`probe_pden.py`, `probe_state.py`,
`probe_traj.py`, `design-note.md`).

---

## 1. What the source says — all read first-hand, tables off page renders

`[F]` = Soltani, A. & Sinclair, T.R., *Modeling Physiology of Crop Development, Growth
and Yield*, CABI.

### The model (Ch. 9, Box 9.2, printed p. 111)

```vb
If CTU <= tuEMR Then                       ' pre-emergence
    GLAI = 0: DLAI = 0
ElseIf CTU > tuEMR And CTU <= tuTLM Then    ' node-driven — SINK-limited
    INODE = DTU / PHYL                      ' Eqn 9.1
    MSNN  = MSNN + INODE                    ' Eqn 9.2, init MSNN = 1
    PLA2  = PLACON * MSNN ^ PLAPOW          ' Eqn 9.3
    GLAI  = ((PLA2 - PLA1) * PDEN / 10000) * WSFL   ' Eqn 9.5 x Eqn 15.7
    PLA1  = PLA2: DLAI = 0
ElseIf CTU > tuTLM And CTU <= tuBSG Then    ' carbon-driven — SOURCE-limited
    GLAI = GLF * SLA                        ' Eqn 9.6
    BSGLAI = LAI: DLAI = 0
ElseIf CTU > tuBSG Then
    GLAI = 0
    DLAI = DTU / (tuMAT - tuBSG) * BSGLAI   ' Eqn 9.7
End If
' ... LAI = LAI + GLAI - DLAI               ' Eqn 9.8
```

### The parameters, and the two loci that disagree

| | Table 9.1 (p. 109, generic wheat) | Fig. 12.5 (p. 152, cv. Tajan — the working model) |
|---|---|---|
| `PHYL` | **120** | **112** |
| `PLACON` | 1 | 1 |
| `PLAPOW` | 2.464 | 2.464 |
| `SLA` | 0.021 m²/g | 0.021 m²/g |

* **`tuEMRTLM = 724 °C·day`** — Table 6.4 (p. 67) *and* Fig. 12.5 row 12, agreeing.
* **`WSSL = 0.40`** — Table 15.1 (p. 195), wheat row, against `WSSG = 0.30`.
* **`PDEN = 300 plants m⁻²`** — Fig. 12.4 (p. 151), the "Run" sheet's MANAGEMENT INPUTS.
* [F]'s own outputs at those settings: days to emergence 11, TLM 94, BSG 124, maturity
  168; **max LAI 5.15**; crop mass 1155 g/m²; grain 588 g/m²; HI 51 %.

### ⚠ Three provenance facts, stated rather than smoothed

1. **`WSSL` is published** (Sinclair 1986; Amir & Sinclair 1991; Hammer 1995; Sadras &
   Milroy 1996; Soltani 1999). **`tuEMRTLM` is not** — Table 6.4's caption is
   *"Rough estimates ... in north east of Iran (A. Soltani, unpublished data)"* and the
   body text says they are *"given only as a guide and should not be considered
   constants"*. **That is the same provenance class as the `fstr = 0.40` that got stem
   reserves refused on 2026-08-10**, and the comparison is put on the record here rather
   than buried. What is *published* is what TLM **is**: Table 6.1 (p. 61) defines it off
   the Feekes/Zadoks keys (Large 1954; Zadoks 1974) as *"Ligule of flag leaf visible"*.
2. **`PLAPOW` is density-dependent by [F]'s own statement** (Fig. 9.2b fits
   `PLAPOW = −0.0044·PDEN + 2.29` for chickpea; p. 107: at high density "the effect of
   plant density can be simulated by decreasing PLAPOW"), and **no density is stated for
   the wheat row**. Taking `PDEN` from Fig. 12.4 and `PLAPOW`/`PHYL` from Fig. 12.5 keeps
   the pair inside one parameterization — the same model file, `6 SSM_ppm.xls`.
   ⚠ **The residual exposure, stated not settled:** we cannot verify that Table 9.1's
   wheat row and the Run sheet were the same experiment, only the same book's wheat and
   the same workbook.
3. **`PHYL` 112 vs 120 decides a contract gate.** Measured: 5.9129 vs 5.6431 peak LAI.
   We take 112 because 300 comes from the same model. Mixing them would be the **locus**
   error this project has logged three times.

### The scale identity that makes [F]'s °C·day numbers usable here

`DTU = (TP1D − TBD)·tempfun` with wheat `TBD = 0` (Table 6.3; `TP1D = 24` for Tajan)
reduces to **`DTU = TMP` for every temperature between the base and the first optimum**
— plain base-0 degree-days, the same scale our accumulator uses. So `724` needs no
conversion, and it lands at **DVS 0.658**.

⚠ **This validates the DENOMINATOR, not the boundary.** [F] p. 63 puts BSG "about 120 °C
after anthesis", so its implied emergence→anthesis is `724 + 446 − 120 = 1050` against
our `tsum_anthesis = 1100` — 4.5 % apart, two independent sources. That corroborates the
*length of the season* and says nothing about where TLM falls inside it.

---

## 2. Three things measured BEFORE any code was written

### (a) The density sweep — because peak canopy is linear in a number the source
### does not put in its parameter table

Run as a read-only probe so that "does our choice of `PDEN` decide the band?" is
answered by measurement, not by picking a number that lands inside it — the
wheat-partition-backfill error, restated with a new mechanism.

Area-keyed senescence, `PHYL = 112`, `open_season`:

| PDEN | peak LAI | peak W (t/ha) | grain (mol C) |
|---|---|---|---|
| (frozen control) | 5.4624 | 13.9391 | 33.7142 |
| 100 | 4.2903 | 12.6575 | 31.3126 |
| 200 | 5.2318 | 13.8950 | 33.4286 |
| 250 | 5.5916 | 14.2636 | 34.0296 |
| **300 — [F]'s own** | **5.9129** | **14.5554** | 34.5031 |
| 350 | 6.2059 | 14.7953 | 34.8943 |
| 400 | 6.4835 | 14.9976 | 35.2279 |

Peak LAI crosses the contract-standing 5.0 floor at **PDEN 170.9**. **`PDEN = 300` was
read off Fig. 12.4 before this table existed.**

### (b) The clock — [F]'s literal form is refused BY MEASUREMENT

[F]'s `CTU` is bare degree-days; ours carries vernalization, photoperiod and drought.
Running node appearance on a bare degree-day clock (which needs a second accumulator)
gives `open_season` **peak LAI 11.92** — nearly double the mutual-shading ceiling. Our
development clock is the one `DVS` reads and TLM is a developmental stage, so the node
phase runs on it. **Recorded because it is not obvious and it is worth a factor of two.**

### (c) The representation question (LAI-as-state vs an additive offset) is a NON-QUESTION

Measured both, matched: **peak LAI 5.0257 vs 5.0292, peak W 13.7138 vs 13.7174** — 0.1 %
apart. The advisor's algebraic claim that they are the same object holds, and checking it
was worth the run.

**LAI-as-state is chosen anyway, for a defect the offset form has and it does not:** the
offset drives the *winter* canopy negative (our tree grows leaf carbon while the node
clock is frozen at `tt = 0`, so the offset out-runs the derived term) and `max(0, ·)`
hides it. That is `a-clamp-hides-a-wrong-amount`, on this build's own doorstep.

### (d) ⚠ THE REAL FORK, WHICH IS NEITHER OF THE ABOVE: how much AREA dies with a leaf

Our `Senescence` sheds `rdr_leaf · leaf_C` of **carbon**. In area terms:

| `DLAI` | peak LAI | peak W | grain | min LAI |
|---|---|---|---|---|
| **`rdr_leaf · LAI`** (area-keyed) | **5.9129** | **14.5554** | 34.5031 | 0.0086 |
| `rdr_leaf · leaf_C · sla / A` (carbon-keyed) | 5.0257 | 13.7138 | 32.2002 | **0.0000** |
| (frozen control) | 5.4624 | 13.9391 | 33.7142 | 0.0267 |

**The entire 18 % spread is this one term.** Area-keyed is taken: if a fraction of leaf
*mass* dies, the same fraction of leaf *area* dies. Carbon-keyed re-imports the fixed
specific leaf area the mechanism exists to make emergent, and once area and mass have
decoupled it removes the wrong amount — it lands `min LAI` on exactly 0.0.

---

## 3. ⚠ AT THE SOURCE'S OWN DENSITY, A SCIENCE GATE FAILS

At `PDEN = 300`, `PHYL = 112`, area-keyed `DLAI`, on `open_season`:

* peak LAI **5.9129** — inside the 5.0–8.0 band, at **98.5 %** of the Van Keulen &
  Seligman mutual-shading ceiling (6.0).
* peak W **14.5554 t/ha** — **above the Greenwood crossing 14.4248. This gate fails.**
* ⚠ **But the thing the gate stands for did not happen: `min f_N = 1.000000`, measured.**
  The gate is a tripwire on a proxy (W *excluding* fibrous roots) while `f_N` reads a
  concentration over a biomass that *includes* roots. So this is the tripwire firing, not
  nitrogen stress arriving.
* `PDEN = 250` — **not** the source's number — puts everything green (5.5916 / 14.2636).
  **Choosing it on that basis is the refused shape, so it is not done.**

**This is reported to the user, not tuned around.**

---

## 4. What is deferred, named so it is not mistaken for an oversight

* **[F]'s own senescence** — Eqn 9.7's linear `DLAI` and the `GLAI = 0` stop after BSG.
  Our tree has a senescence flow with its own citation; adopting [F]'s would be a second
  form change to one mechanism inside one build, and the golden diff would stop being
  attributable. `senescence.yaml` already records that the flat rate "has been standing
  in for canopy-regulation science the tree does not have" — that thread is untouched
  here.
* **A non-constant phyllochron** — [F]'s own "Additional Notes" (p. 111) flags it.
* **Waterlogging** (Box 16.2's `If WAT1 > 0.95·WSAT1 Then WSFG = 0: WSFL = 0`) — not
  modelled for `WSFG` either, so this changes nothing.
* **`specific_leaf_area` 22.0 → 21.0 and `extinction_coef` 0.6 → 0.65.** [F] Table 9.1
  and Fig. 12.5 would retire **two standing `TODO(cite)`s** in `canopy.yaml`. Deliberately
  **NOT** taken here: it is a separate unfreeze that moves every carbon golden and would
  confound this one's diff beyond reading. **Named as a successor.**

---

## 5. THE PREDICTED GOLDEN DIFF — written BEFORE regeneration

Every crop-growing scenario moves; that is the mechanism working, not a surprise. So the
prediction is which files and in which **direction**, not the digits.

**Predicted to MOVE** — every golden whose run builds a wheat crop
(`plant_density = 300`):

`season_euler_state.json`, `sealed_chamber_state.json`, `perennial_chamber_state.json`,
`perennial_long_horizon_state.json`, `consumer_chamber_state.json`,
`consumer_long_horizon_state.json`, `n_limited_state.json`, `water_biting_state.json`,
`drift_summary.json`, `greenhouse_state.json`, `lighting_state.json`,
`harvest_state.json`, `station_state.json`, `sealed_station_state.json`,
`sealed_energy_drift_summary.json`.

⚠ **The three 7-day station runs are predicted to move HERE and did NOT move under stem
reserves**, and the reason is the whole point of the mechanism: the stem-reserve window
opened at anthesis, 287 days past the end of those runs, whereas the node-driven phase
starts at emergence — **day 0 is inside it**.

**Predicted UNCHANGED** — no crop in the run: `demo_euler_state.json`,
`demo_rk4_state.json`, `state_snapshot.json`, `power_state.json`,
`power_self_discharge_state.json`, `thermal_state.json`, `eclss_state.json`,
`crew_state.json`, `cabin_gas_state.json`, `water_recovery_state.json`.

**Predicted signs**, `open_season`: peak LAI **up** 5.4624 → 5.9129; peak W **up**
13.9391 → 14.5554; grain **up** 33.7142 → 34.5031. Sealed chambers: a bigger early
canopy draws the finite CO₂ pool down **faster**, so the trough goes **down**.
Potato: **bit-identical**, because `plant_density = None`.

**Manifest**: `aux_set` 3 → 4 (`LeafAreaExpansion`), `param_files` 14 → 15
(`leaf_area.yaml`), `flow_set` unchanged at 22 (no new flow — this is an accumulator and
a read, not a transfer). Unlike the `WSFD` build, **both automatic gates are live here**.

---

## 6. WHAT THE BUILD MEASURED — and the blocker it hit

### 6.1 The re-sow defect the rationing gate caught (fixed)

The first run of the 15-year chambers **rationed 85 times**. Cause: `annual_reset`
resets `thermal_time`, `vernalization_days` and `rooted_depth`, and I had not added
`leaf_area_index` — so every new seedling inherited the **dead crop's canopy** while its
leaf carbon reset to a seed, and each cycle assimilated at full canopy from day 0.

⚠ **The question only exists because this build made leaf area a state.** While LAI was
*derived*, it re-sowed itself for free: resetting the organ pools reset the canopy, with
nothing to remember or forget. **That is the standing price of reversing the "LAI is
derived, not stored" lock**, and every accumulator added from here owes the same answer.
Found as a *gate firing*, not as a wrong number — which is what
`post-roadmap-rationing-gate.md` made rationing loud for. Fixed; 85 → 83.

### 6.2 ⚠ THE BLOCKER: the node-driven branch makes AREA WITHOUT CARBON

The residual 83 are not a bug. [F]'s sink-limited branch computes leaf area from node
number **independently of dry matter** — that is its definition. Where carbon is
plentiful this is right. Where carbon is *limited*, area keeps growing while mass does
not, and the emergent specific leaf area — the thing this mechanism deliberately makes
emergent — runs away.

Measured on every scenario (emergent SLA as a multiple of the nominal 22 m²/kg; real
wheat spans roughly 15–35, i.e. **0.7–1.6×**):

| scenario | sealed | rationed | peak LAI | SLA max | SLA median |
|---|---|---|---|---|---|
| open_season | no | 0 | 5.9129 | 3.84 | **0.94** |
| drought | no | 0 | 5.9129 | 3.84 | **0.94** |
| deep_water | no | 0 | 3.6659 | 3.84 | **0.87** |
| day_neutral | no | 0 | 2.6867 | 4.29 | 1.95 |
| water_biting | yes | 0 | 0.9571 | 3.00 | 1.89 |
| **n_limited** | **no** | 0 | 2.9068 | **29.46** | **3.72** |
| sealed_chamber | yes | **27** | 2.9068 | 9.47 | 2.15 |
| perennial_chamber | yes | **41** | 2.9068 | 12.23 | 2.26 |
| consumer_chamber | yes | 0 | 2.9068 | 12.31 | 2.68 |
| perennial_long_horizon | yes | **83** | 2.9068 | 14.72 | 2.41 |
| consumer_long_horizon | yes | 0 | 2.9068 | 14.82 | 2.91 |

**⚠ `n_limited` is the witness that kills the tidy answer.** It is an **open-field** run
with the worst leaf thickness of the whole roster. So the cause is **carbon limitation**,
not chamber sealing — and any "wire it in the field, not in the jar" rule would be
picking scenarios rather than following the evidence. The three well-fed field runs
(median 0.87–0.94×) are the only ones inside the source's domain.

**[F] says so itself.** Ch. 9's opening sentence scopes the method to *"leaf area
development **under non-limiting water and nutrients**, and free of insects, diseases,
and weeds"*, and Ch. 12 — the chapter this parameterization comes from — is titled
*"A Model for **Potential Production**"*. ⚠ And a sealed chamber is outside [F]'s
universe in a stronger sense than "stressed": **[F] has no mechanism by which the
atmosphere runs out of carbon**, because in its world it cannot.

### 6.3 The hybrid form, taken from [F]'s own page and REFUTED BY MEASUREMENT

[F] Ch. 9 "Background" (printed p. 103) names a third approach: *"Boote et al. (1998)
used the hybrid approach by estimating leaf area development as **the smaller of the
estimates** obtained by the temperature and assimilate availability approaches."* That
is `GLAI = min(node-driven, carbon-driven)` — a form the source states, so trying it is
not inventing science.

⚠ LOCUS, stated: this is [F] *describing* Boote et al., not the form [F] implements —
its own submodel is the temperature-based branch.

⚠⚠ **AND THE SENTENCE THIS PARAGRAPH ORIGINALLY USED TO DISMISS BOOTE BELONGS TO A
DIFFERENT MODEL.** The draft above continued *"and [F] adds of the hybrid view that
'there is little experimental evidence to support such a complex view'"*. Re-read off
the page (printed p. 103), that sentence closes the paragraph about **Kropff and van
Laar (1993)** — a *phase switch* (temperature drives expansion while the canopy is thin
enough that leaves do not shade each other, then assimilate supply takes over), which is
a different shape from Boote's per-estimate `min`. The quote was transcribed correctly
and attached to the wrong model: **a locus error inside a correct quote**, which is the
class `nitrogen-option-c-refused` was logged for, committed here in our own record.
Corrected rather than deleted, because the deletion would hide that the dismissal of
Boote was never sourced.

**Measured, and it is refuted:** `min` binds on most days, and because withheld area is
never recovered, the whole roster collapses — `open_season` peak LAI **5.4624 (frozen) →
1.2682**, `n_limited` → 0.0682, every chamber under 0.63. It fixes the leaf thickness
(max 1.00–1.40×) and the rationing by **destroying the canopy**. Not taken.

### 6.4 Where that leaves the build

The mechanism is **built, wired, lint- and type-clean, and correct in the domain its
source states** — the three well-fed field runs carry leaf thickness within 6 % of
nominal and a canopy of 5.91 against [F]'s own model's 5.15.

What it cannot currently do is ship across the frozen roster: **six of the seven frozen
biosphere scenarios are growth-limited**, and in them [F]'s form produces leaves 2–15×
too thin, three of them rationing outright. There is no honest scenario-level switch,
because the discriminator is a *state* (carbon limitation), not a scenario property.

**The options, priced, none of them mine to choose:**

1. **Ship narrow** — mechanism on for the well-fed field runs, off elsewhere. Coherent
   with [F]'s stated domain, but it means one of seven frozen scenarios gets the new
   science and the tree carries two leaf-area forms.
2. **Bound the emergent thinness** — a maximum specific leaf area. Physically real and
   widely used, but the *value* would be **ours**, and a number we choose that decides a
   canopy is the shape this record has refused three times.
3. **Apply [F] Ch. 17's nitrogen factor to the node branch** — `WSFN`, already a named
   successor. It would put `n_limited` back inside the source's reach, and would do
   **nothing** for the sealed chambers, whose limit is CO₂.
4. **Refuse** — record the mechanism, the parameters, and the domain measurement, and
   leave the tree on the derived form.

### 6.5 ⚠ THE DEFECT IS SHARPER THAN "THE LEAVES GET THIN"

Look again at the peak-LAI column: `n_limited` and **all five chambers** peak at
**exactly 2.9068** — six scenarios, different volumes, different nitrogen, horizons of
1, 3, 5 and 15 years, agreeing to four decimals.

That is not a coincidence, it is the diagnosis. **2.9068 is where the node-driven curve
ends**, and in all six the carbon-driven branch never adds anything on top. So below
TLM the canopy is running as a **pure function of the calendar and the sowing density,
with no feedback whatsoever from whether the crop can afford it**. `open_season` differs
(5.9129) only because there the carbon branch keeps building after TLM.

This is why a maximum-thinness cap (option 2) is weaker than it looks: it would bound
the *symptom* and leave the missing feedback exactly as it is. It is also the
number-we-choose shape this record has refused three times. **Recommended against, not
offered as an equal option.**

### 6.6 The Greenwood gate fails in the domain where the mechanism WORKS

"Ship narrow" does not come out clean, and the price has to be stated: at [F]'s own
`PDEN = 300` with `PHYL = 112`, `open_season` peak W is **14.5554 against the crossing
14.4248** — the gate fails by **0.9 %** on the one scenario the narrow option would ship.

⚠ **And that outcome is decided by the phyllochron locus, which cuts both ways:**

| `PHYL` | locus | peak LAI | peak W | vs crossing 14.4248 |
|---|---|---|---|---|
| **112** | Fig. 12.5, cv. Tajan (the working model `PDEN = 300` comes from) | 5.9129 | **14.5554** | **FAILS by 0.9 %** |
| **120** | Table 9.1, the generic-wheat row [F] offers for "wide use" | 5.6431 | 14.3162 | clears by 0.75 % |

Both loci are defensible, and **120 has an argument that has nothing to do with the
gate**: our crop is *not* cv. Tajan — its phenology, partition table, root depth and
stem reserves all come from [E] — so importing one cultivar-specific constant from [F]'s
Tajan sheet is its own locus mixing. ⚠ **This is recorded, NOT acted on. Switching to
120 because it passes would be the wheat-partition error verbatim**, and the two
candidates sit within 1 % of the crossing on either side of it.

### 6.7 Suite-wide scope: MEASURED ONLY FOR THE REGRESSION GOLDENS

Only `tests/test_regression_*.py` has been run. `test_senescence_form.py`,
`test_nitrogen_form.py`, `test_soil_layers.py`, `test_stem_reserves.py` and
`test_acceptance_gate.py` all pin canopy/biomass quantities this build moves, and none
of them has been run. **"20 goldens move" is not the damage report — the suite-wide
scope is unmeasured**, deliberately, because the full run costs seven minutes ahead of a
decision that may be "revert".

**No golden has been regenerated. No manifest has been touched. The tree is mid-build
and red.**

---

## 7. THE SECOND MECHANISM — taken on the user's call, "we need an additional model that behaves where this doesn't"

§6 ended with four priced options and no verdict. The user took none of them and set a
different charge: **build the model that behaves where [F] does not, and supersede [F]
if it can.** This section is that build.

### 7.1 The diagnosis that chose the form: `min` on the RATE is not `min` on the STATE

§6.3 refuted `GLAI = min(node_rate, carbon_rate)` (open_season 5.4624 → **1.2682**) and
recorded it as "the hybrid is refuted". That reading was too broad. The rate-level `min`
fails for a reason that says nothing about colimitation:

* a per-day `min` **ratchets** — the integral of the pointwise minimum is strictly below
  both paths, so the canopy loses ground every day rather than tracking the binding one;
* and the carbon rate is **itself a function of LAI** (a smaller canopy intercepts less
  light), so the ratchet is *self-reinforcing*. It is a death spiral, not a bound.

A bound on the **state** has neither property: a day the crop cannot pay for costs area
it never had, not area it already built. Everything below follows from that distinction.

### 7.2 The shelf search — and what it actually found

Advisor's condition going in was explicit: *"If neither carries a maximum, the honest
outcome is the finding, not an invented number."* Searched: [E] (Penning de Vries et al.
1989) and Teh, *Introduction to Mathematical Modeling of Crop Growth* — both shelved.

| source | what it gives | why it is not a bound |
|---|---|---|
| [F] Table 9.1 / Fig. 12.5 | wheat SLA 0.021 m²/g | a **mean** |
| [F] Fig. 9.6 | wheat SLA 0.025 m²/g (Tajan + Zagros) | a **mean**, and unpublished |
| [F] Fig. 9.9 | relative SLA vs radiation/temperature | **CROPGRO-soybean** — wrong crop, the logged locus error |
| [F] Eqn 9.11 | shading senescence above `LAICR` | keys on LAI alone (`LAICR` ≈ 4–5) and our runaway peaks at 2.9068, so it cannot reach; [F]: *"we are not aware of any experimental approach to estimate"* its two constants |
| Teh, `partsla.txt` | SLA vs development stage | a **mean trajectory** |
| [E] Listing line 53 | `LIMIT(200., 600., SLA)` | a validity range **inside the photosynthesis response**, not a physiological bound |

**⚠ The structural finding underneath the table: every source on this shelf that models
leaf area under *any* limitation computes it as mass ÷ thickness.** [E] is
`ALV = WLV/SLA`; Teh is `LAI += W_leaf · SLA`. [F]'s node branch is the only sink-driven
one, and it is a potential-production device. So the "additional model that behaves where
[F] doesn't" is not exotic — **it is the form already frozen in this tree.**

**And then [E] Table 20 turned out to have a `Wheat, winter` row** (printed p. 102, off a
200-dpi render), which is a *two-sided envelope*, a better object than the one-sided
maximum the search was looking for:

```
FUNCTION SLT = 0.,1., 0.33,1.1, 0.36,1.06, 0.43,1.5,
               0.53,1.05, 0.62,1., 0.77,0.85, 0.95,1.07, 1.14,1., 2.1,1.
```

(development stage, specific leaf **weight** as a fraction of the Table 19 constant).
Range **[0.85, 1.50]**. ⚠ The fractions invert: weight is mass per area, so the *minimum*
fraction is the *thinnest* leaf and hence the *ceiling* on area.

**Corroboration of the centre, three ways:** [E] Table 19 winter wheat 425 kg/ha =
23.5 m²/kg; [F] 21 m²/kg; our own frozen `specific_leaf_area` 22 — a 12 % spread across
two books. ⚠ And note our 22 is a bare `TODO(cite)`, so [E]'s row is **better** provenance
than the constant this tree already ships.

### 7.3 The form, and the two measurements that fixed it

```
LAI = min( max( LAI_prev + (GLAI − DLAI)·dt , leaf_C·SLA/A / SLW_MAX ) ,
                                              leaf_C·SLA/A / SLW_MIN )
```

Both bounds from **one table, one crop, one source**. Two measurements decided the shape:

1. **The ceiling alone is not enough, and gets it backwards.** [F]'s node curve starts at
   `PLACON·1^PLAPOW` = 1 cm²/plant — *smaller* than the seedling's carbon-derived area —
   so a ceiling-only bound lets `min` pull LAI **below** what the standing mass already
   supports, and the milder spiral runs. Measured: ceiling-only at 1.176× gives
   `open_season` peak LAI **4.4300** against the frozen form's own **5.4624** — the
   combined model coming out *worse than the model it was meant to improve*.
2. **[E]'s curve applied instantaneously is worse still.** Using `SLT(DVS)` as the bound
   rather than its extremes gives **2.0403**, because `SLT = 1.5` at DVS 0.43 makes the
   bound *tighter* than nominal for part of the season. The envelope is the **range** of
   thickness the crop exhibits, not the thickness it typically has.

### 7.4 What it does — measured on all eleven scenarios

| scenario | FROZEN | **[E] band** | [F] alone | ration (band / [F]) |
|---|---|---|---|---|
| open_season | 5.4624 | **5.2533** | 5.9129 | 0 / 0 |
| drought | 5.4624 | **5.2533** | 5.9129 | 0 / 0 |
| deep_water | 3.2057 | 2.8831 | 3.6659 | 0 / 0 |
| day_neutral | 0.0861 | 0.1285 | 2.6867 | 0 / 0 |
| water_biting | 0.4161 | 0.4440 | 0.9571 | 0 / 0 |
| n_limited | 0.0773 | 0.0976 | **2.9068** | 0 / 0 |
| sealed_chamber | 0.5411 | 0.6879 | **2.9068** | 0 / **27** |
| perennial_chamber | 0.4960 | 0.6514 | **2.9068** | 0 / **41** |
| consumer_chamber | 0.5626 | 0.7276 | **2.9068** | 0 / 0 |
| perennial_long_horizon | 0.4960 | 0.6514 | **2.9068** | 0 / **83** |
| consumer_long_horizon | 0.5626 | 0.7276 | **2.9068** | 0 / 0 |

* **Rationing 0 on every scenario** (was 27 / 41 / 83). §6.1's re-sow defect and §6.2's
  blocker are both closed.
* **Leaf thickness 1.03–1.18× nominal median, 1.20–1.21× max** (was 0.87–3.72 median,
  3.84–**29.46** max). Every value inside real wheat's ≈15–35 m²/kg.
* **The 2.9068 signature is gone.** §6.5 identified six scenarios peaking at exactly the
  end of the node curve as proof of *no carbon feedback whatsoever* below TLM. They now
  differ from each other, because the bound is a function of leaf carbon.
* **[F]'s node branch still decides ~39–45 % of days** (17 % floored, 43 % capped on
  `open_season`). It is not a no-op — the sink limitation survives.

### 7.5 ⚠ THE GREENWOOD GATE PASSES, AND IT PASSES DOWNWARD

§6.6's blocker: at `PHYL = 112`, `open_season` peak W was **14.5554** against the
14.4248 t/ha crossing — failing by 0.9 % on the one scenario "ship narrow" would ship,
with the *other* phyllochron locus (120) passing. §6.6 recorded that switching for that
reason would be the wheat-partition error verbatim, and refused to.

With the envelope, peak W is **13.6717 — clearing by 5.2 %, at `PHYL = 112`.** The gate
was resolved by a mechanism, not by choosing the constant that passes. Peak LAI 5.2533
also clears both canopy gates (5.0–8.0 band; < 6.0 Van Keulen & Seligman).

**⚠ Corroboration nobody fitted:** [F]'s own working model reports max LAI **5.15**. Ours
is 5.2533 (**+2.0 %**) with the envelope and 5.9129 (**+14.8 %**) without. The envelope
came from [E]'s Table 20 with no reference to [F]'s output, and it moved us *toward* [F]'s
own answer.

### 7.6 ⚠ WHAT THE ENVELOPE COSTS: IT DECIDES A DISAGREEMENT BETWEEN TWO SHELVED SOURCES

This is the finding that must not be filed as a bound's side effect.

* **[F] Table 15.1** gives wheat `WSSL = 0.40` for leaf area against `WSSG = 0.30` for
  growth — leaf expansion is *more* drought-sensitive than growth, i.e. **drought
  thickens leaves**. That is the entire reason [F] gives leaf area a factor of its own.
* **[E] p. 100**: *"Different growing conditions, such as those caused by different plant
  densities or fertilization level in maize (Sibma, 1987), and by irrigation in potato
  (Ng & Loomis, 1984), have little effect on specific leaf weight."*

They disagree about whether drought moves area-per-mass. **The envelope makes [E] win**,
and the price is measurable. On `water_biting` — the only scenario where `WSFL` fires
hard — forcing `WSFL = 1`:

| | peak LAI with `WSFL` | with `WSFL ≡ 1` | Δ |
|---|---|---|---|
| [F] alone | 0.9571 | 3.0146 | **+215.0 %** |
| [E] band | 0.4440 | 0.4461 | **+0.5 %** |

**The envelope reduces the drought curve's leverage by a factor of ~400 in the one run
that exercises it.** `WSFL` was the reason this whole mechanism was built.

⚠ **And the corroborating sentence is weaker than it looks.** Its loci are **potato**
irrigation (Ng & Loomis) and **maize** density (Sibma), not wheat, and [E] flags the
generalization itself: *"The influence of these environmental factors **may also be**
small in other crops; they are disregarded here."* What is wheat, and measured, is Table
20's row.

### 7.7 ⚠ COVERAGE: FIVE OF SEVEN SCENARIOS NEVER EXERCISE `WSFL` AT ALL

Before reading §7.6 as "drought does not matter", the discriminating measurement —
`WSFL = min(1, FTSW/0.40)`:

| scenario | min `WSFL` | days < 1 | mean |
|---|---|---|---|
| open_season | 1.0000 | **0 %** | 1.0000 |
| **drought** | **1.0000** | **0 %** | 1.0000 |
| day_neutral | 1.0000 | 0 % | 1.0000 |
| n_limited | 1.0000 | 0 % | 1.0000 |
| sealed_chamber | 1.0000 | 0 % | 1.0000 |
| deep_water | 0.7402 | 4 % | 0.9932 |
| water_biting | **0.1250** | **100 %** | 0.5606 |

**The scenario named `drought` never stresses leaf expansion at all** — consistent with
`drought-defence-is-the-mechanism-working`: the drought-accelerated phenology shortens
the season so the crop escapes the deficit. So §7.6's small numbers on most of the roster
are a **coverage** fact, and only `water_biting` licenses the physiological reading. Both
are recorded because they have different successors.

### 7.8 ⚠ THE PROVENANCE OF THE TWO NUMBERS, AND WHY THE ANSWER SURVIVES IT

[E] Table 19's winter-wheat row shows **no reference**; the footnote is *"Values without
reference were obtained from colleagues at CABO, Wageningen"*, Table 20's source line is
*"see Table 19"*, and [E] p. 99 says *"Insufficient data were found in the literature to
derive more than a few of these crop specific relations"* and *"Descriptive functions
such as these should be used carefully and checked whenever possible."*

**That is the same provenance class as `tu_tlm` above and as the `fstr = 0.40` that got
stem reserves refused.** Stated, not laundered.

What makes it survivable is a difference in *kind*, and it was measured rather than
asserted: `fstr` **set** an amount; this pair **bounds** one, and where it binds the model
reduces to the frozen carbon-derived form. So a wrong envelope fails *toward the science
already in the tree*. Sensitivity, across envelopes from degenerate to very wide:

| envelope | open_season LAI | water_biting | n_limited | perennial_long | open W | ration |
|---|---|---|---|---|---|---|
| (1.00, 1.00) | 4.9106 | 0.3880 | 0.0760 | 0.5025 | 13.3655 | 0 |
| (0.90, 1.25) | 5.1662 | 0.4235 | 0.0897 | 0.5919 | 13.6034 | 0 |
| **(0.85, 1.50)** cited | **5.2533** | 0.4440 | 0.0976 | 0.6514 | **13.6717** | 0 |
| (0.85, 1.75) | 4.9384 | 0.4165 | 0.0974 | 0.6760 | 13.3293 | 0 |
| (0.80, 1.50) | 5.5830 | 0.4967 | 0.1065 | 0.6731 | 14.0093 | 0 |
| (0.75, 2.00) | 5.5831 | 0.5366 | 0.1169 | 0.7470 | 14.0024 | 0 |
| **none** ([F] alone) | 5.9129 | 0.9571 | **2.9068** | **2.9068** | **14.5554** | **110** |

Every envelope clears the Greenwood crossing with zero rationing and moves `open_season`
by ≤ 19 %; **removing** the envelope moves `n_limited` by **30×** and fails the gate. The
two numbers pick a point inside a flat region.

### 7.9 Three exposures, stated rather than settled

1. **⚠ LOCUS, WITH ITS DIRECTION.** [E] applies `SLT` to the specific weight of **new**
   leaf area (`SLN`, Listing 3 Line 91); we apply it to the **canopy average**. A mixture
   varies less than its newest component, so this envelope is **wider than [E]'s own**
   — biased *generous to [F]'s node branch*. The honest envelope would clip more and
   `open_season` would sit below 5.2533. It cannot be derived from Table 20 without
   running [E]'s mixture model, so it is recorded, not resolved.
   ⚠⚠ **REFUTED 2026-08-12 — §9.** The mixture model was run, and the answer is that
   this pair *is* the canopy-average envelope: the interval is forward-invariant under
   [E]'s own dynamics and tight. The paragraph above conflates the range the average
   **visits** with the range it can **reach**; a bound is about the second. Left standing
   verbatim because §9.4 shows the tempting fix — fitting the envelope to the visited
   range — lands inside the retune window §8.4(a) refused.
2. **⚠ THE PROJECTION IS NOT `dt`-INDEPENDENT, DELIBERATELY.** Unclamped, `evaluate`
   returns `rate·dt`; clamped, it returns `ceiling − LAI`, whose implied rate depends on
   `dt`. That is the correct discretisation of a bound *on the state* — at any step size
   the result never violates the envelope, which a `dt`-independent rate cannot promise.
   Same shape as `root_depth`'s cap. **The Rust mirror carries the rule, not the
   rationale**, so it is written into the docstring rather than left to coincide at
   `dt = 1`.
3. **⚠ "INSIDE THE ENVELOPE BY CONSTRUCTION" WOULD BE ~3 % FALSE.** The bound reads leaf
   carbon at **step entry** while the delta lands after, so the canopy overshoots by one
   step's growth: measured ceiling 1.176×, measured maximum **1.20–1.21×**, i.e. 2.9 %.
   The same lag is why the degenerate `(1.00, 1.00)` envelope does *not* reproduce the
   frozen form exactly — it lands **10.1 % low** (4.9106 vs 5.4624). That control is kept
   precisely because it puts a number on the lag.

### 7.10 The clamp lesson, answered before it is raised

`a-clamp-hides-a-wrong-amount` says a clamp survives until the scale changes, and this one
is active 55–85 % of days. The defence, written down rather than left to be re-derived:
that lesson is about a clamp standing in for an **amount nobody measured**. This one is a
*cited physiological bound* whose binding branch is the tree's own reference form, and
§7.8 measures what happens when the number moves. If a future scale change breaks it, the
control that will show it is the degenerate envelope in §7.9(3), not a golden.

---

## 8. ⚠ BLOCKING: RK4 HARD-ERRORS ON `perennial`, AND EVERY MEASUREMENT ABOVE WAS EULER-ONLY

### 8.1 What the full suite found that §7 could not

§6.7 warned that only the regression goldens had been run. Paying that debt returned
**105 failed, 31 errors, 2195 passed**. The 105 split three ways:

* **~28 expected for any unfreeze** — the frozen goldens moved (that is the point), plus
  `test_freeze_manifest` (regenerate) and `test_context_budget` (the plan-doc index,
  fixed).
* **~49 the value pins §6.7 named** — `test_senescence_form` (20), `test_stem_reserves`
  (16), `test_acceptance_gate` (11), `test_nitrogen_form`, `test_chamber_scale`.
* **~26 §6.7 did NOT name** — `test_soil_fractionation` (15), `test_decade_stability`
  (3), `test_crew_coupled_loop` (2), and one each in `test_water_biting`,
  `test_potato_crop`, `test_oracle_gap`, `test_o2_makeup_reversal`,
  `test_nitrogen_throttle`, `test_harvest_run`, `test_greenhouse_run`.

⚠ **§6.7's own scope estimate was short by a factor of three** — it named five files; the
reach is seventeen. "20 goldens move is not the damage report" was the right instinct
applied to a list that was itself incomplete.

### 8.2 ⚠ THE 31 ERRORS ARE ALL ONE FILE, AND THEY ARE NOT A MOVED NUMBER

Every one is in `test_decade_stability.py`, and none is an assertion:

```
simcore.arbitration.ArbitrationError: flow #0 (canonical order) would over-draw a stock
(scale_f=0.9668362828371428 < 1) under a higher-order scheme; min-scaling is Euler-only
— positivity under RK4+ must come from the kinetics, not the backstop
```

**Every table in §7 ran Euler.** All eleven scenarios report `rationed == 0` under Euler,
across every envelope in the §7.8 sweep — and that is exactly what the guard cannot see.
This is the trap `stem-only-branch-priced-and-refused` logged in the same words
(*"`perennial` goes rationed 0 → 1 UNDER EULER … and hard-errors under RK4"*), met from
the other side: **we read clean under Euler and hard-error under RK4.**

### 8.3 It is ours. The baseline is clean.

| arm | perennial Euler | **perennial RK4** | consumer Euler | consumer RK4 |
|---|---|---|---|---|
| **FROZEN (the shipped derived form)** | 0 rationed | **ok, 0 rationed** | 0 | ok |
| **[F] + [E] envelope (as built)** | 0 rationed | **ArbitrationError `scale_f=0.9668`** | 0 | ok |
| **[F] alone** | **68 rationed** | **ArbitrationError `scale_f=0.6982`** | 0 | ok |

The envelope cuts the overdraw from **30 % to 3.3 %** — a large improvement, and not
enough. The mechanism raises the chamber's peak leaf carbon 0.8446 → 0.9677 (**+15 %**),
and `chamber-scale-diagnosed` already measured that *"the jar holds 2 days of carbon"*.
**A sealed chamber cannot afford the extra canopy, and Euler hides it.**

### 8.4 Two diagnostics that characterise it — measured, NOT adopted

**(a) The failure is marginal in the bound, not fundamental.** Perennial, decade, RK4:

| `slw_fraction_min` | ceiling (× derived) | result |
|---|---|---|
| **0.85 — [E]'s cited value** | 1.176 | **ArbitrationError 0.966836** |
| 0.90 | 1.111 | ok, 0 rationed |
| 0.95 | 1.053 | ok, 0 rationed |
| 1.00 | 1.000 | ok, 0 rationed |

⚠⚠ **0.90 IS NOT TAKEN, AND THE REASON IS THE WHOLE POINT.** `biosphere-reference.md`
step 5: *"retuning a bound so a change fits is the co-adaptation shape this project has
refused"*. It would be doubly tempting here because §7.9(1) independently records that our
envelope is **wider than [E]'s own** (a new-leaf range applied to the canopy average), so a
narrower bound has an argument that has nothing to do with this gate — which is precisely
what would make adopting 0.90 *laundering* rather than a fix. A narrower envelope is
earned by running [E]'s mixture model, not by trying values until RK4 goes quiet.

**(b) ⚠ THE FIRST VERSION OF THIS PARAGRAPH WAS A MECHANISM CLAIM IN THE GRAMMAR OF A
MEASUREMENT, AND IT IS KEPT HERE BECAUSE THAT IS THE ERROR WORTH SEEING.** It read:

> *"1 yr ok; 3 / 5 / 10 / 15 yr all fail with `scale_f = 0.966836` identical every time.
> So this is a specific day from the **second sowing onward**, repeating — which points at
> the `annual_reset` × re-sow interaction. **The identical constant across four horizons
> is the evidence**; a drifting failure would give four different numbers."*

**Every step of that is unsound.** The run **raises on the first violation and aborts**,
so every horizon ≥ 3 executes an identical prefix and dies on the identical day —
identical `scale_f` is a *tautology of fail-fast*, not evidence about mechanism. A
drifting failure would produce those same four numbers. And `years = 1` never reaches an
`annual_reset` at all, so "year 1 clean" cannot separate re-sow from any other post-reset
cause. This is the shape `stem-reserves` logged: **a prediction written in the grammar of
a measurement.** Committed here, on this record's own doorstep, one section after
congratulating itself for measuring rather than asserting.

**THE DISCRIMINATING MEASUREMENT, WHICH THE ABOVE SHOULD HAVE BEEN.** Instrument the
raise and report *which* flow and *which* step:

| | |
|---|---|
| flow #0 in canonical order | **`biosphere.allocation`** — the flow that spends carbon to build tissue |
| failing step index | **502** |
| position in season | **day 197 of season 1** |
| distance to nearest season boundary | **108 days** |

**Mid-season, on the allocation flow, 108 days from any re-sow.** The re-sow hypothesis
is **refuted**; the test's own docstring names both candidates (*"the discrete
`annual_reset` × multistage interaction **or** a needed arbitration scale hit the
hard-error path"*), and the answer is the second. Under RK4 `check_no_overdraw` runs at
each **perturbed stage state** — amounts already shifted by `0.5·k1`, which `_perturb`
documents as legitimately able to go negative — so a canopy 15 % larger than the frozen
one asks `allocation` for carbon a stage state no longer holds. **It is the sealed
chamber's carbon inventory**, which `chamber-scale-diagnosed` had already measured as two
days deep.

⚠⚠ **AND THEREFORE §8.5's ORIGINAL CLAIM THAT THIS BLOCKER IS "INDEPENDENT OF THE §7.6
SCIENCE JUDGEMENT" IS FALSE.** If the cause is inventory, then the envelope's generosity —
letting LAI reach 1.176× the carbon-derived area — and the overdraw are **the same fact**,
not two. The ceiling sweep in (a) is not a coincidence beside the science choice; it *is*
the science choice, measured through a different gate. This correction matters more than
the re-sow one, because it was told to the user as a reason the two decisions could be
taken separately, and they cannot.

**⚠ One thing this DOES clear, and it clears it for a reason worth recording.** The
suspicion that the clamp's `dt`-dependence (§7.9(2)) causes the RK4 failure is **refuted
by the integrator's own contract**: `step_report` advances aux *once per step* at the
step-entry state (`_aux_increments(self._registry, state, env, dt)`), `_perturb` shifts
**stock amounts only** so `State.aux` passes through every stage unchanged, and all four
stages are called with the **full `dt`** (`_rk4_stage(reg, _perturb(state, k1, 0.5), env,
dt)`) — the perturbation is in the *state*, never in the step size. So LAI is identical
across k1–k4 and the clamped branch's absolute delta never enters a stage evaluation.
§7.9(2) remains a real `dt`-independence departure that a future multi-rate or
variable-step consumer must respect; it is **not** this bug.

### 8.5 Where this leaves the build

The mechanism is **not shippable as it stands**. ⚠ An earlier draft of this line called
the blocker "independent of the §7.6 science judgement" — **8.4(b) refutes that**: the
overdraw is the sealed chamber's carbon inventory, so the envelope's generosity and the
RK4 break are one fact. The two decisions are coupled:

* `perennial_chamber` is a **frozen scenario**, and `test_decade_stability` exercises RK4
  deliberately to retire this precondition. An `ArbitrationError` there is a hard break,
  not a moved number.
* The frozen form passes the same run cleanly, so this is a regression we introduce.
* The residual is small (3.3 %) and the boundary is ~6 % away in the bound — near enough
  that a *scientifically earned* narrowing (§7.9(1)'s mixture model) might clear it, and
  far enough that nothing on the shelf currently clears it. **And because §8.4(b) traces
  the overdraw to inventory rather than to a discretisation artifact, that narrowing is
  now the ONE successor that addresses both the RK4 break and the §7.9(1) locus exposure
  at once** — the envelope being wider than [E]'s own is exactly what buys the canopy the
  carbon the jar does not have.

**No golden has been regenerated. No manifest has been touched.** The tree is red, the
mechanism is complete and measured, and the successor is named: **derive [E]'s
canopy-average envelope properly, or refuse.** ⚠ Whatever comes next, the lesson from
§8.2 is the durable one — **this build's entire evidence base was single-integrator, and
the guard it trusted is blind by construction.** Any successor measures both.

---

## 9. THE NAMED SUCCESSOR, TAKEN: derive [E]'s canopy-average envelope properly

§8.5 named one successor — **"derive [E]'s canopy-average envelope properly, or refuse"** —
and §7.9(1) priced it as *"it cannot be derived from Table 20 without running [E]'s mixture
model"*. Taken on the user's call 2026-08-12.

⚠ **THIS SECTION IS WRITTEN BEFORE THE MODEL IS RUN WITH ANY NUMBER IT DERIVES.** §8.4(a)
already measured that `slw_fraction_min = 0.85` errors under RK4 and `0.90` does not, and
already refused 0.90 as retuning. Any derivation landing in that window is, *after the
fact*, indistinguishable from having tuned to make RK4 quiet. The only thing separating an
earned narrowing from a retune is **order**, so the derivation and its numeric consequence
are recorded here first, and the run follows. Same discipline as §5's predicted golden diff.

### 9.1 [E]'s mixture, from its own Listing 3 — four lines, one ODE

The listing (printed pp. 116-117, lines 88-92) is explicit about the two loci that §7.9(1)
says we conflated:

```
88   GLA = GLV / SLN                          new AREA from new leaf MASS
89   LLA = LLV / SLA                          area lost at the CANOPY AVERAGE
91   SLN = SLC * AFGEN(SLT, DS)               new-leaf specific WEIGHT, stage-keyed
92   SLA = (WLV + 0.5*WST*(SLC/SSC)) / ALV    the canopy average is EMERGENT, not input
```

[E]'s prose says the same in words (printed p. 101): *"The rate of leaf area loss is
computed in direct relation to the rate of leaf weight loss, **assuming that the average
value of the specific leaf weight applies***".

Write `S ≡ W/A` for the canopy-average specific leaf weight (dropping line 92's green-stem
term, which this tree does not model). With `dW = GLV − LLV` and
`dA = GLV/SLN − LLV/S`:

```
dS/dt = (dW·A − W·dA) / A²
      = ( A·GLV − A·LLV − W·GLV/SLN + LLV·A ) / A²
      = (GLV/A) · (1 − S/SLN)
```

**The senescence terms cancel exactly** — a direct consequence of line 89 removing area at
the canopy average rather than at a cohort thickness. So the canopy average is driven
toward the *current* new-leaf thickness at a rate proportional to relative leaf-mass
growth, and nothing else moves it.

### 9.2 ⚠ THE INVARIANT-INTERVAL ARGUMENT REFUTES §7.9(1). NO NARROWING IS AVAILABLE

`GLV ≥ 0` always in this tree (`DMI = Yg·max(0, GASS − MRES) ≥ 0`, and `fl ≥ 0`). So from
the ODE, for any growth history whatsoever:

* if `S < SLN_min` then `1 − S/SLN > 0`, so `dS/dt > 0` — S is pushed **up**;
* if `S > SLN_max` then `1 − S/SLN < 0`, so `dS/dt < 0` — S is pushed **down**.

`[SLC·min SLT, SLC·max SLT]` is therefore **forward-invariant** under [E]'s own dynamics.
That interval is `SLC · [0.85, 1.50]` — **the pair already shipped.**

⚠ **THE QUALIFIER IS LOAD-BEARING: THE INVARIANCE IS [E]'s, NOT OURS, AND THAT IS EXACTLY
WHY THIS IS A CLAMP RATHER THAN AN EMERGENT PROPERTY.** It holds for the pair of rules in
lines 88/89 — area grown at `SLN`, area lost at the canopy average. Our tree honours
neither unconditionally: `GLAI` comes from [F]'s node branch below `TLM` (area with no
mass at all), and **herbivory removes leaf carbon with no matching `DLAI` term**, so `S`
can fall here with nothing restoring it. The derivation licenses the *interval*; the
projection in `LeafAreaExpansion.evaluate` is what imposes it. ⚠ That same asymmetry is
what biased the first mixture reconstruction in §9.4.

⚠ **§7.9(1) IS WRONG, AND THE ERROR IS A CONFLATION OF TWO DIFFERENT RANGES.** It argued
*"a mixture varies less than its newest component, so this envelope is wider than [E]'s
own"*. True of the range S **visits**; false of the range S can **reach**. A bound is about
the second. The interval is also **tight, not conservative**: `dS/dt → 0` only as `S → SLN`,
so sustained leaf growth at DS 0.77 drives `S → 0.85·SLC` asymptotically. Nothing between
0.85 and the mixture's typical value is a bound.

**The one structural narrowing that would have been legitimate, and why it is not available
here.** The invariant interval is the hull of `SLT` over the stages at which leaf mass is
*actually created* — if the crop grew no leaf at DS 0.77, no 0.85-thickness cohort would
ever exist and the hull would be genuinely narrower, with zero free parameters. Our
partition table (`allocation.yaml`) is `fl = 0.55 / 0.30 / 0.00` at DVS 0 / 1 / 2,
linearly interpolated, so **leaf takes a nonzero share across the whole of DS [0, 2.0)** —
which contains both DS 0.43 (`SLT` = 1.50, the maximum) and DS 0.77 (`SLT` = 0.85, the
minimum). Both extreme cohorts are created. The window is the full table.

⚠ **A per-scenario window would NOT be a legitimate narrowing even if one existed.** In a
carbon-starved run `DMI` can sit at 0 for stretches, so the *realised* DS-hull is
scenario-dependent — and a range computed on `open_season` and clamped onto
`perennial_chamber` is the locus error this record has already logged three times. The
window above is **phenological**: it comes from the partition table and the DVS clock, not
from carbon supply, so it is the same on all eleven scenarios by construction.

### 9.3 THE PRE-REGISTERED CONSEQUENCE — stated before running

The derivation changes **no parameter**. `slw_fraction_min` stays `0.85`, `slw_fraction_max`
stays `1.50`. Therefore, predicted before measurement:

1. **No golden moves. No manifest moves.** There is nothing to regenerate.
2. **RK4 still fails, at exactly the same place**: `test_decade_stability.py`,
   `ArbitrationError` on `biosphere.allocation` with `scale_f = 0.966836`, 31 errors.
3. ⚠ **§8.5's claim that this is "the ONE successor that addresses both the RK4 break and
   the §7.9(1) locus exposure" is FALSIFIED.** The locus exposure is discharged — by being
   shown wrong, not by being fixed — and it discharges *nothing* about RK4. The two were
   never one fact; §8.4(b) coupled them through an argument that only works if a narrower
   envelope is derivable, and it is not.
4. The corroborating diagnostic below (the mixture *trajectory*) is a **prediction, not a
   bound**, and is recorded as such: it is one run's harmonic mean, and using its extremes
   as an envelope would be exactly the fitted-to-one-scenario error 9.2 refuses.

**So the successor named in §8.5 is discharged with NO CHANGE TO THE MODEL, and the
mechanism remains blocked under RK4 with no route on the shelf.** What that leaves is a
verdict, not a build — recorded in §9.5.

### 9.4 The mixture MEASURED — corroboration, and the trap it exposes

`M:/claud_projects/temp/leaf-expansion/probe_mixture.py` integrates the §9.1 ODE driven by
each scenario's own leaf-carbon trajectory. ⚠ It integrates the **average**, not the area:
in that form every loss term cancels (§9.1), so it needs only the *relative* leaf growth
rate and a second leaf-carbon sink cannot bias it. The area form was tried first and was
biased exactly where that matters — it read `f_min = 0.79` on `consumer_chamber`, below the
invariant interval, because the reconstruction charged herbivory's mass loss to senescence
and so removed too little shadow area. **A reconstruction artifact reported as a violated
invariant is the §8.4(b) shape again**, caught this time before it reached the record.

| scenario | `f` min | `f` max | DS at min | margin to 0.85 | margin to 1.50 |
|---|---|---|---|---|---|
| `open_season` | 0.9449 | 1.2776 | 0.855 | 0.0949 | 0.2224 |
| `drought` | 0.9449 | 1.2776 | 0.855 | 0.0949 | 0.2224 |
| `deep_water` | 0.9675 | 1.2776 | 0.866 | 0.1175 | 0.2224 |
| `day_neutral` | 1.0000 | 1.1126 | — | 0.1500 | 0.3874 |
| `water_biting` | 0.9611 | 1.2158 | 0.861 | 0.1111 | 0.2842 |
| `n_limited` | 1.0000 | 1.1671 | — | 0.1500 | 0.3329 |
| `sealed_chamber` | 1.0000 | 1.0981 | — | 0.1500 | 0.4019 |
| `perennial_chamber` | 1.0000 | 1.0964 | — | 0.1500 | 0.4036 |
| `consumer_chamber` | 1.0000 | 1.2987 | — | 0.1500 | 0.2013 |

⚠ The probe restarts `f = 1` at each re-sow, which would inherit the four chamber minima
of *exactly* 1.0000 from an assumption rather than measuring them — except that the
assumption is **exact**: every scenario's seedling starts at `LAI = 0.029360` against a
carbon-derived `0.029360`, i.e. `f₀ = 1.000000`. So those minima say what they appear to
say — those runs never dip below the derived form at all.

**Every trajectory stays strictly inside the invariant interval, on all nine scenarios** —
the forward-invariance argument of §9.2 confirmed empirically as well as analytically. The
minimum lands at DS ≈ 0.855, *after* `SLT`'s own minimum at 0.77, which is the ODE's lag
showing up exactly where it should.

⚠⚠ **AND HERE IS WHY §9.2 REFUSES TO USE THIS TABLE AS AN ENVELOPE — THE MEASUREMENT MAKES
THE TRAP CONCRETE.** The trajectory extremes are strongly scenario-dependent: `f_min` is
0.9449 on the open field and **exactly 1.0000** on all four carbon-limited runs, which never
dip below the derived form at all. A ceiling fitted to `open_season`'s 0.9449 would be
1.058× the carbon-derived area instead of 1.176×. **§8.4(a) measured that 0.95 clears
RK4.** So the trajectory-fitted envelope would have gone green — and would have been
indistinguishable, after the fact, from the `0.90` retune that §8.4(a) refused. The
tempting route lands inside the refused window. That is the whole reason §9.3 was written
before this table was produced.

### 9.5 THE VERDICT — the successor is DISCHARGED, and it discharges nothing else

All three §9.3 predictions held. No parameter moved; no golden or manifest was touched;
`tests/test_decade_stability.py` still errors at `scale_f = 0.9668362828371428`, the same
value to all ten recorded digits.

**What is settled:**

1. **§7.9(1) is REFUTED.** The shipped `[0.85, 1.50]` *is* [E]'s canopy-average envelope,
   properly derived — it is the forward-invariant interval of [E]'s own mixture dynamics,
   it is tight (attained asymptotically under sustained growth at DS 0.77), and no
   scenario-independent narrowing exists because the partition table creates leaf mass
   across the whole DS range that contains both extremes. The exposure is discharged **by
   being shown wrong**, and the parameter file's `slw_fraction_min` source note carries the
   claim and must be corrected.
2. **§8.5 is FALSIFIED.** "The ONE successor that addresses both the RK4 break and the
   §7.9(1) locus exposure" was wrong: the two were never one fact. §8.4(b) welded them
   together via an argument that holds only if a narrower envelope is derivable. It is not.
3. **The RK4 blocker stands, with NO route on the shelf.** The only measured way to clear
   it remains moving `slw_fraction_min` into [0.90, 1.00], which is the retune
   `biosphere-reference.md` step 5 forbids — and §9.4 now shows that the one derivation
   that would have *looked* principled lands in that same window.

**So the mechanism is complete, correct, fully measured, and unshippable.** What remains is
a verdict, not a build, and it is the user's: this is the third time in this record that a
piece of science has been priced and handed back rather than settled by me
(`root-coupling-refused`'s lesson — *my own refusal is a recommendation, not a verdict*).

---

## 10. ⚠ §9.5(3) IS FALSIFIED — "no route on the shelf" was never measured

**2026-08-12, the same day.** §9.5(3) reads *"The RK4 blocker stands, with NO route on the
shelf"*, and that sentence is doing the work of the refuse-and-revert recommendation. It is
a claim about the space of routes, and **the only thing ever measured was one knob**
(`slw_fraction_min`, §8.4(a)). No sweep of the space was run.

Asked as a question about the **tree** rather than about leaves — *does the frozen tree pass
RK4 by margin or by construction?* — it answers on frozen `main` alone, and it answers
against §9.5. The full record is
[`post-roadmap-allocation-headroom.md`](post-roadmap-allocation-headroom.md); its record row
is [`../log/allocation-headroom.md`](../log/allocation-headroom.md). In one line: **the
blocker is a step-size bound, not a science defect** — the parked branch runs clean at
`dt = 1/2` with no parameter moved and no form changed, and a crop far larger than this
mechanism ever grew runs clean at `dt = 1`.

⚠ **Two things that document establishes which this one must not be read as claiming:**

1. **The reversal is not an endorsement.** What cleared at the finer step is the **guard**.
   §8's own lesson — a guard blind by construction cannot be evidence a mechanism is safe —
   applies identically to shrinking `dt` until `check_no_overdraw` goes quiet. Everything in
   §7.4–§7.7 is **still Euler-only at `dt = 1`** and has to be re-measured at whatever step
   would ship, before this mechanism is called ready.
2. **The successor is not a leaf successor.** It is the `dt = 1` contract for a sealed
   chamber (three freeze contracts, not one), or a *cited* supply-limited assimilation form
   that keeps `dt = 1`. Neither belongs in this plan doc.

**Nothing in `src/` changed for finding 11, and the parked branch was never merged.**
