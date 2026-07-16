# Post-roadmap — bucket 3 scope (C): cite the no-oracle params

**Status: COMPLETE** (2026-07-16), then **ROUND 2 COMPLETE** (2026-07-16) — the user
supplied ~10 of the 12 sources on this doc's own retrieval list, and the open items were
re-run against them. **Jump to "ROUND 2" at the bottom for what changed.** Round 1's text is
preserved below unedited, including the predictions round 2 falsified — the point of a
living record is that you can see what was believed at the time.

The third and last scope of bucket 3 (validation). Scope (A) — diagnose + pin the oracle
gap — is COMPLETE (`post-roadmap-validation.md`). Scope (B) — the full oracle match —
remains open and is untouched by this work.

The charge: discharge the `TODO(cite)` debt on every parameter the WOFOST oracle **cannot**
validate, by binding each value to a primary source — or, where no source can exist, by
saying so honestly instead of inventing one.

## What scope (C) is, precisely

Scope (A) established that the 57 params split by *what can validate them*, not by domain:

| set | validated by | act |
|---|---|---|
| the **7** potential-production crop param files (`photosynthesis`, `canopy`, `phenology`, `allocation`, `respiration`, `senescence`, `transpiration`) — 30 uncited params | the WOFOST oracle trajectory | **calibration** — citation and calibration are the same act ⇒ **scope (B)** |
| everything else — the **6** non-PP biosphere files, all 10 Power/Thermal/ECLSS params, and the 4 station params | **no oracle exists** | **literature citation** — a lighter, separate act ⇒ **scope (C), this doc** |

Scope (C) is the second row. It is independent of scope (B)'s two structural gaps
(vernalization; juvenile canopy expansion) and has **no blocker**.

## Two corrections to the scope as sold

**1. The count was wrong: 29 params, not 15.** `CLAUDE.md` and the direction memory both
said "15 no-oracle params (10 sibling + 5 non-PP biosphere)". The "5" counted **files**, not
params — and it was itself short by one file. Measured (2026-07-16):

| set | files | uncited params |
|---|---|---|
| non-PP biosphere | `decomposition`, `mineralization`, `microbial_respiration`, `nitrogen`, `herbivory`, **`water_cycle`** | **15** |
| sibling (Power/Thermal/ECLSS) | `charge`, `self_discharge`, `radiator`, `eclss` | **10** |
| station | `harvest`, `lamp`, `water_recovery` | **4** |
| crew | `crew` | 0 — already BVAD-calibrated (Phase 6 Step 9) |
| **total** | | **29** |

**`water_cycle.yaml` had fallen through a crack that predates this scope, and is now closed.**
Scope (A)'s own two-column table sorted the biosphere into "7 PP files" and "5 non-PP files"
— **water_cycle is in neither**, so its 2 params were invisible to both scopes. It is
scope (C)'s, established empirically rather than assumed: the `Condensation` flow that
consumes these params exists only in the `if scenario.sealed:` branch of `atmosphere.py`, and
the oracle trajectory runs `build_season()`, whose default scenario is `sealed=False` and
whose assembled registry contains **no condensation flow at all**. The params are never even
*loaded* in an oracle run, so they cannot move that trajectory. (Caught on advisor review of
this document, after it had already been written up as "complete" — an inherited omission is
still an omission.)

The 4 station params were excluded from scope (C) as sold but carry identical debt and
already sit under the **same station manifest** the 10 sibling params force a regeneration
of. **User decision (2026-07-16): fold them in** — the marginal cost is ~zero and leaving
them out means regenerating the same manifest twice.

**2. The unfreeze mechanics are far lighter than recorded — and the recorded claim is
false.** `bucket3-oracle-gap-diagnosed` and `CLAUDE.md` both state that a `source:` edit
"moves the hash → **fails the manifest gate** → *is* an unfreeze". The first and last
clauses are true; **the middle one is not**. Verified read-only (2026-07-16):

* **`source` is recorded, not parsed.** Every loader schema requires a non-empty `source:`
  string and never reads its content (`domains/biosphere/loader.py`: *"the required
  clean-room provenance tag (recorded, not parsed)"*). A citation edit cannot move a value.
* **No test asserts the param hashes.** `tests/test_freeze_manifest.py` is explicit that the
  manifest's `_normalized_sha256` is **provenance — "a re-derivable record of *which
  content* was frozen … **not** an assertion"**. The gate owns **completeness** (the param
  *file set*); **values** are the goldens' job, and a `source:` edit moves no value.
* **Nothing cascades to the Rust port.** `tests/crossport/gen_biosphere_params.py` emits
  `name\t<value.hex()>` only — no `source` field crosses the port boundary. The generated
  `biosphere_params.txt` stays byte-identical.

**So a citation-only edit turns nothing red.** It remains an unfreeze by
`biosphere-reference.md`'s own definition (*"Changing **any** frozen item … is an
unfreeze"*), but the ceremony is **honor-system, not CI-enforced**. That cuts both ways:
mechanically trivial, and *because* nothing catches a skipped discipline, the discipline is
followed deliberately — advisor review, manifest regeneration as the git-visible record,
and this document.

**Two manifests are touched**: biosphere (the 5 non-PP files) and station (`charge`,
`eclss`, `radiator`, `self_discharge`, `harvest`, `lamp`, `water_recovery`). The authoring
manifest is untouched.

## The bins: not every number has a source, and that is the finding

The scope was picked as "cite the params", which presumes every value *has* a citation. It
does not. Sorting all 27 by **what their existing `Sources:` block actually supports**:

* **BIND** — the source supports the **value** (it names a number or a range ours sits in).
  Discharge = bind to a verified primary source.
* **DESIGN** — the source supports only the **functional form**. Seader/Ogata say
  first-order scrubbing and proportional control are textbook; they say nothing about
  `co2_scrub_rate = 1e-3`. There is no primary source for *"our toy station has a 10 m²
  radiator"*. These numbers are **modelling choices**, and **citing them would be
  fabrication** — the exact failure mode `param-file-conventions.md` exists to prevent.

Every one of the 5 non-PP biosphere files carries a `Sources:` block, but **all of them are
method-only** (Olson's first-order litter decay; Stanford & Smith's mineralization kinetics;
Lotka's linear functional response; Davidson's Michaelis–Menten form; the CENTURY/RothC
microbial-pool treatment). Several say *"primary citation pending"* outright. So the
biosphere set is **bind-fresh**, not bind-to-existing.

**User decision (2026-07-16): the DESIGN tag.** A design/sizing param discharges its TODO by
declaring itself, in the `source:` string, as a design choice with its rationale — e.g.
`source: "DESIGN — sizing choice, not a literature value: …"`. Chosen over a schema field
(which would touch `src/` loaders across 5 domains + the Rust schema) and over sharpened
TODO wording (which leaves settled decisions reading as unpaid debt forever). Cost: an
amendment to `docs/param-file-conventions.md`, whose rule currently admits only cited
values. Recorded there as the second legitimate provenance class.

## Verified so far

Each entry below was **fetched and read** — no locus is recorded that was not opened. This
is the anti-fabrication discipline of this scope: WebSearch can confirm a value is
"literature-typical"; it rarely licenses a specific table.

| param | value | verdict |
|---|---|---|
| `thermal.space_temperature` | 2.7 K | **BIND** — Fixsen, D.J. (2009), "The Temperature of the Cosmic Microwave Background", *ApJ* **707**:916–920: T_CMB = **2.72548 ± 0.00057 K**. **Delta finding:** ours is rounded, 0.93 % low; not changed (a value change is scope (B)). |
| `station.lamp.photon_efficacy` | 2.5 µmol/J | **BIND** — Kusuma, P., Pattison, P.M., Bugbee, B. (2020), "From physics to fixtures to food: current and potential LED efficacy", *Horticulture Research* **7**:56, Table 2 + "Typical Fixture Efficacy": commercial white+red **2.5–2.8**, blue+red **3.0**; limits **3.4** (white+red) / **4.1** (blue+red). Ours sits at the bottom of the measured commercial range; the header's "bio max ~4.1" is confirmed. |

**Open finding — `power.self_discharge_rate` (1.0e-8 /s ≈ 2.6 %/month) may not be
bindable.** The header cites Dunn (2011) *Science* and Divya (2009) *EPSR* for "a few
%/month". A literature sweep found the canonical "1–3 %/month" figure lives in **vendor and
trade sources** (Battery University, cell-maker blogs), not primary literature; the
peer-reviewed self-discharge papers (e.g. Schmidt, Weber & Ivers-Tiffée (2015), *J. Power
Sources* **274**:1231–1238) characterize **measurement methods**, and modern good cells are
reported **well below** 1 %/month. So 2.6 %/month is at the pessimistic/older-cell end of a
range whose primary-source anchor is weak. Resolution pending: bind to a genuine primary
source, or reclassify as DESIGN with the range recorded as a delta finding. **Not to be
resolved by citing a blog.**

## Discipline (carried from scope (A), non-negotiable)

1. **Never invent a source, a page, or a table.** If it was not opened, it is not written.
   Prefer an accessible primary source (arXiv, PMC, a public-domain NASA TP) over an
   unverifiable textbook page. Where only a range is verifiable, cite the range and state
   the value sits in it — do not manufacture a locus.
2. **Disagreement is a finding, never a value change.** If the literature says 0.03/day and
   ours is 0.02/day, record the delta. **Changing the number is calibration = scope (B)**,
   and would move goldens.
3. **Clean-room on the biosphere set.** Cite primary soil/crop literature only — never
   WOFOST/PCSE-derived values. A web search for "typical crop decomposition rate" can land
   on a WOFOST-sourced number and silently contaminate the clean-room.
4. **"Cited" ≠ "calibrated".** Binding a value to a source that merely *permits* it does not
   validate the model (`authoring-reference.md`, "Frozen is not calibrated"). Scope (C)
   retires a provenance debt, not a scientific one.

---

# OUTCOME — scope (C) COMPLETE (2026-07-16)

**The headline is not the citations. It is that the decomposer side of the carbon cycle runs
3–28× fast — a second structural finding, sitting beside scope (A)'s canopy collapse.**
Scope (A) found the model's *intake* side broken (the canopy intercepts 1.75 % of light and
collapses before anthesis). Scope (C) went looking for citations and found the *return* side
is wrong too, in the opposite direction: litter decay, N mineralization and microbial
respiration are each **faster than anything in the accessible literature** for the process
their header names. Nobody had measured this because nobody had looked up the numbers.

**Every number below was verified against a source that was actually opened and read.** Where
a subagent reported a locus, it was re-fetched before it landed — which caught a real error
(see "What verification caught").

## The tally: a PARTIAL discharge, honestly

29 params carried `TODO(cite)`. They did **not** all become citations, because half had no
citation to find:

| class | count | meaning |
|---|---|---|
| **CITED** | **8** (+2 crew already cited = 10 of 31) | a primary source, opened and read, supports the value |
| **DESIGN** | **14** | a deliberate sizing/stability/behavioural choice; no source *can* fix it |
| **TODO(cite) + finding** | **7** | looked, found nothing that binds — and found evidence the value is off |

Scope (C)'s full surface is **31** params across 13 files (29 that carried debt + crew's 2,
already cited). Audited file-by-file after the fact; the **30** `TODO(cite)` params that
remain in the tree are all in the 7 potential-production files, which are scope (B)'s by
construction.

The 7 that stay open are **not** a failure to look. Each now carries a *measured* finding in
its `source:` tag instead of a promise. Per `param-file-conventions.md`, a `TODO(cite)` is
honest where a fabricated locus would not be — and these are the params where the honest
answer is "no source binds this, and here is what the literature does say".

**The 8 new CITED:** `emissivity`, `space_temperature`, `photon_efficacy`,
`charge_efficiency`, `carbon_fraction`, `n_critical`, `n_residual`, and microbial
`o2_half_saturation`.

**The 14 DESIGN:** the 4 ECLSS loop constants, `radiator_area`, `heat_capacity`,
`harvest_rate`, `recovery_rate`, `recovery_efficiency`, both `water_cycle` rates, and
herbivory's `grazing_rate` / `respiration_rate` / `mortality_rate`.

**The 7 still open, with their deltas:** `self_discharge_rate` (~6–7× the measured 25 °C
rate), `decomposition_rate` (~1.5× above a 293-value global max), `mineralization_rate`
(~3.9× the Stanford & Smith mean), `microbial_respiration_rate` (~28× RothC BIO),
`n_senescence_rate` (no accessible primary at all), `max_uptake_capacity` (~6× the fastest
reported rate), herbivory `o2_half_saturation` (~4× sharper than animal physiology allows).

## The findings, ranked by what they mean for the model

1. **The decomposer side runs 3–28× fast (the structural one).** `microbial_respiration_rate`
   = 18.25/yr vs RothC's BIO pool at **0.66/yr** (~28×) and CLM5/CENTURY's active-SOM pool at
   5.9/yr (~3.1×) — across two independent model lineages, and both apply temp/moisture
   modifiers ≤ 1 *on top of* k, widening the gap. `mineralization_rate` = 0.21/wk vs the
   Stanford & Smith mean **0.054/wk** (~3.9×) and above the fastest of 43 soils ever measured
   (0.174/wk) — and those are *already* optimal-condition potential rates (35 °C, field
   capacity), so temperature/moisture cannot explain it away. `decomposition_rate` = 7.3/yr
   vs a 293-value global database spanning **0.006–4.993/yr** (~1.5× the maximum, ~24× the
   median). **This is a scope-(B) candidate, and it is new**: scope (A) diagnosed the
   production side only.
2. **A cross-kingdom parameter copy (the clean bug).** The herbivory `o2_half_saturation` was
   set by "mirroring `microbial_respiration.yaml`" — but the consumer is an **animal**, and
   the soil-bacteria Km values that legitimately bind the microbial file (99–288 nM) do not
   transfer. Animal mitochondrial effective Km is ≈ 0.49 µM, so ours is **~4× sharper than
   animal physiology warrants**. Behaviourally inert today (f_O2 ≈ 1 at the ample-O₂ fill) —
   it bites only near anoxia, which is exactly where the error lives.
3. **Two live miscitation risks in the frozen tree.** `charge.yaml` and `self_discharge.yaml`
   attributed specific numbers to Dunn (2011) *Science* and Divya (2009) *EPSR* — **both
   paywalled, neither ever opened**. The claims may be true; nobody has checked. Worse, [A]
   is a grid-storage *review*, so even a matching sentence would be a review assertion, not
   primary data. Both are now demoted to UNVERIFIED leads, and `charge_efficiency` no longer
   rests on them (it binds to Bašić 2023 instead). `radiator.yaml` had the same shape —
   Gilmore (an unopenable commercial handbook) was cited for "emissivity ~0.8–0.9"; replaced
   by a public-domain NASA table that was read.
4. **"1–3 %/month" Li-ion self-discharge is vendor folklore.** The figure that would bracket
   our value saturates the trade press and is **absent from primary literature**, which
   reports self-discharge as µA float currents or temperature curves — never a canonical
   %/month constant. A number everyone repeats and nobody publishes.
5. **`n_critical` as a constant has a quantifiable cost.** Greenwood's critical N is a
   *dilution curve* (%N = 5.7·W^−0.5), not a constant. Our 1.5 % equals it only at
   W ≈ 14.4 t/ha — near maturity. At 2 t/ha the curve gives ≈ 4.0 % (~2.7× ours), so a fixed
   constant **under-states critical N all through early/mid season** and makes N limitation
   trigger late or never. It is effectively the late-season asymptote.
6. **Two headers were checked and VINDICATED** — worth recording, because the checks were run
   expecting the opposite. `water_recovery`'s "deliberately below a real ISS-class recovery"
   **holds**: real ISS total recovery is 93–94 % pre-BPA and 98 % with the BPA, both above
   our 0.9. And `charge.yaml`'s "0.95 is optimistic vs a real ~0.90 round-trip" **holds**:
   Bašić measures the round-trip at 0.8991. Both concessions were right, and are now
   evidenced rather than asserted.

## What verification caught (the reason the merge step exists)

Six subagents did the literature sweep; every load-bearing citation was re-fetched before it
landed. That caught a real error: the station agent, reading BVAD Table 4-89 through naive
PDF text extraction, reported wheat's growth period as **97 dAP** — the value from the
**Soybean row**, one row down. The table's rows shift under text extraction. Rendering the
page and reading it directly gives the true Wheat row: **75–90 dAP**, printed p. **172** (not
171). The agent had honestly flagged its own alignment as "inferred, not read cleanly" and
asked for a visual render — the flag was correct and the number was wrong.

Two agents also **self-corrected mid-flight**, stripping unverified figures after advisor
review rather than reporting them (the nitrogen agent dropped snippet-only Ma and Justes
coefficients; the herbivory agent refused to report a Davidson kMO₂ it could not open). And
the thermal agent reported that a "STRONG LEAD" supplied in its own brief **did not pan out**
(the NASA SOA thermal chapter lists absorptivity but no emissivity column) rather than
stretching it. *Reporting a lead as a bind would have been fabrication* — the discipline
held under pressure to produce.

## The trap that was refused

The soil agent recommended: *"Re-anchoring that header to a labile-pool source would make the
value defensible without changing it"* — i.e. relabel `decomposition_rate` as a
decomposable-fraction constant, since 7.3/yr is unremarkable next to RothC's DPM (10/yr).
**Refused, on advisor review.** This model has **one** litter pool, not a DPM/RPM split.
Relabelling the parameter to make a citation fit is reinterpreting what the parameter *means*
— confirmation bias wearing a provenance hat, and a semantic **model** change rather than a
citation edit. The delta is recorded **both ways** in `decomposition.yaml` so a future
maintainer inherits the choice rather than just the conclusion. Re-anchoring the pool's
meaning is scope (B).

## Verification: nothing moved

* **All 25 goldens + the generated `rust/.../biosphere_params.txt` are byte-identical** to a
  baseline hashed before the first edit (26/26). Proven, not asserted — `source` is
  recorded-not-parsed, and the Rust generator emits `name\t<value.hex()>` only.
* `git diff src/` touches **param YAML only** (12 files) — no engine, domain or station code.
* `uv run pytest -m "not slow"`: **1779 passed**, 1 skipped (opt-in oracle), 48 deselected.
  `uv run ruff check .`: clean.
* Both manifests regenerated; the diff is **exactly the 12 param hashes** (5 biosphere + 7
  station) and nothing else — no golden hash, no flow/aux set. That diff *is* the git-visible
  record of this unfreeze.
* **No new tests**, deliberately: these values are already golden-pinned (any change moves a
  golden), so the findings belong in the source tags and this doc. A test asserting "0.02 is
  1.5× above Zhang's max" would restate a doc in code. Pinning *behaviour* is scope (B)'s job,
  where a recalibration actually moves goldens.

## PAYWALLED — the retrieval list

Sources that could not be opened but would settle something real. Ranked by value.

| # | Source | Settles | Why it is worth the effort |
|---|---|---|---|
| 1 | **Parton, W.J., Schimel, D.S., Cole, C.V., Ojima, D.S. (1987)**, "Analysis of factors controlling soil organic matter levels in Great Plains grasslands", *SSSAJ* 51:1173–1179. [doi:10.2136/sssaj1987.03615995005100050015x](https://doi.org/10.2136/sssaj1987.03615995005100050015x) | `microbial_respiration_rate`, `decomposition_rate` | **Highest value.** Its table gives the **maximum (unmodified) decay rates** — the true analogue of our bare constants, since CENTURY applies modifiers afterward. Would convert the ~28× finding from two secondary implementations into a primary bind. Abstract elided even on Semantic Scholar. |
| 2 | **Penning de Vries, F.W.T., et al. (1989)**, *Simulation of Ecophysiological Processes of Growth in Several Annual Crops*, Simulation Monographs 29, PUDOC, Wageningen. ISBN 90-220-0937-8 (print/library only) | `n_senescence_rate` | **The only clean-room-safe route.** Every accessible numeric RDR source traces to WOFOST/PCSE and is legally unusable here. A library scan of the relevant table settles the one param with *no* accessible support. |
| 3 | **Dunn, B., Kamath, H., Tarascon, J.-M. (2011)**, "Electrical Energy Storage for the Grid: A Battery of Choices", *Science* 334(6058):928–935. [doi:10.1126/science.1212741](https://doi.org/10.1126/science.1212741) | audits **two live citations** | It is **already cited in our tree**. If it does not contain the numbers our headers attributed to it, that is a real miscitation to fix. Confirmed paywalled (S2 `isOpenAccess=false`). |
| 4 | **Divya, K.C., Østergaard, J. (2009)**, "Battery energy storage technology for power systems — An overview", *EPSR* 79(4):511–520. [doi:10.1016/j.epsr.2008.09.017](https://doi.org/10.1016/j.epsr.2008.09.017) | `self_discharge_rate` | The single most likely home of a citable by-chemistry %/month figure — the number that is otherwise vendor-only. Elsevier paywall; DTU Orbit has metadata only (Semantic Scholar's open-access flag is a false positive). |
| 5 | **Justes, E., Mary, B., Meynard, J.-M., Machet, J.-M., Thelier-Huché, L. (1994)**, "Determination of a Critical Nitrogen Dilution Curve for Winter Wheat Crops", *Annals of Botany* 74(4):397–407 | `n_critical` | *The* wheat-specific critical-N curve (Greenwood is generic C3, pooled across species). Would place 1.5 % on the winter-wheat curve and give the validated biomass ceiling — which matters because our 14.4 t/ha result may sit outside it. OUP returned no abstract at all, only a €53 offer. |
| 6 | **Stanford, G. & Smith, S.J. (1972)**, "Nitrogen mineralization potentials of soils", *SSSAJ* 36:465–472. [doi:10.2136/sssaj1972.03615995003600030029x](https://doi.org/10.2136/sssaj1972.03615995003600030029x) | `mineralization_rate` | Replaces our Schomberg-quoting-S&S chain with the first-hand k = 0.054 wk⁻¹ and the per-soil distribution across 39 soils. Wiley HTTP 402. |
| 7 | **Olson, J.S. (1963)**, "Energy storage and the balance of producers and decomposers in ecological systems", *Ecology* 44(2):322–331. [doi:10.2307/1932179](https://doi.org/10.2307/1932179) | `decomposition_rate` | Already cited for method; its own k table (by ecosystem) is the one document that would say directly whether 7.3/yr is inside Olson's tabulated range. Wiley/JSTOR closed. |
| 8 | **Coleman, K. & Jenkinson, D.S.**, "RothC-26.3 — A Model for the turnover of carbon in soil", in *Evaluation of Soil Organic Matter Models* (Springer). [doi:10.1007/978-3-642-61094-3_17](https://doi.org/10.1007/978-3-642-61094-3_17) | `microbial_respiration_rate` | The authoritative primary for BIO = 0.66/yr, which we currently have only via FAO. **Note:** the correct citation target — `microbial_respiration.yaml` cites Jenkinson 1990 *Phil. Trans.*, but the per-pool constants live in *this* model description, not that paper. |
| 9 | **Davidson, E.A., et al. (2012)**, "The DAMM kinetics model…", *Global Change Biology* 18:371–384. [doi:10.1111/j.1365-2486.2011.02546.x](https://doi.org/10.1111/j.1365-2486.2011.02546.x) | the Km scale gap | **Already in our header.** A free PDF exists at harvardforest1.fas.harvard.edu but is an **image-only CCITTFax scan** — OCR would resolve it. Its parameter table converts "~3 orders looser" from an estimate into an exact ratio. |
| 10 | **Greenwood, D.J., et al. (1990)** — *full text* (abstract is open). [aob 66:425](https://academic.oup.com/aob/article/66/4/425/257815) | `n_critical` validity | The abstract gives only the lower bound (W > 1 t/ha). The data tables would confirm whether W ≈ 14.4 t/ha is inside the fitted database — the one open question in an otherwise-verified bind. Also reports crop-specific *b* values. |
| 11 | **Van Hecke, J., et al. (2020)** — *full text*. [doi:10.1007/s11104-020-04600-6](https://doi.org/10.1007/s11104-020-04600-6) | `n_residual` | The abstract gives 0.29–0.69 % but no per-N-level breakdown. Straw N at the *lowest* N supply is the closest empirical estimate of a true structural floor — it would say whether 0.5 % is right or whether ~0.3 % is. |
| 12 | **Gilmore, D.G. (ed.) (2002)**, *Spacecraft Thermal Control Handbook, Vol. 1*, 2nd ed., Aerospace Press | `emissivity`, `radiator_area` | **Now a nice-to-have, not a blocker** — NASA RP-1121 already gives a public-domain bind. Would add broader coating tables and radiator sizing heuristics (W/m²). The canonical unopenable-commercial-textbook case. |

**Free but tooling-blocked** (no paywall — retrievable in a browser): **Azzam et al. (2023)**,
*Energies* 16(9):3889 ([doi](https://doi.org/10.3390/en16093889)) — modern-cell self-discharge
float currents; PDF > 10 MB and mdpi.com 403s automated fetchers. **Longmuir (1954)**,
*Biochem. J.* 57(1):81–87 ([PMC1269709](https://pmc.ncbi.nlm.nih.gov/articles/PMC1269709/)) —
the classic Km-vs-species table; PMC hosts page *images* only. **Roth et al. (2023)**,
*JES* 170(2):020502 — reportedly contrasts the historical 2–3 %/month against modern
< 0.1 %/month measurements, which would settle finding 4 outright; IOPscience 403s.

## What this leaves for scope (B)

Scope (B) was scoped by (A) as *vernalization + juvenile canopy expansion, then recalibrate*.
**(C) adds a third piece of new science to it:** the decomposer rates. That is a real scope
increase, and it should be decided deliberately rather than absorbed — the soil rates are a
*separate* system from the canopy, they are wrong in the *opposite* direction (too fast, not
too slow), and fixing them without fixing the canopy would change the chamber's carbon
balance in ways neither finding predicts alone. Note also that any recalibration here moves
biosphere goldens **and** cascades to the station goldens (`station/{greenhouse,lighting,
sealed,harvest}.py` re-run biosphere science).

---

# ROUND 2 — the retrieval list, delivered (2026-07-16)

The user went and found the sources. **10 of the 12 items** on the paywalled list above,
plus 2 of the 3 "free but tooling-blocked" ones, plus ~15 unlisted extras — dropped into
`sources/` (gitignored: this repo is public and most are copyrighted; we cite them, we do
not redistribute them). Round 1 had reached these through secondaries, abstracts and
inference. Round 2 opened them.

**Nothing moved.** No `value:` line was touched; 1779 passed / 1 skipped (round 1's exact
baseline); the two manifests regenerated to a diff of **exactly the 6 param hashes** and
nothing else; `rust/.../biosphere_params.txt` regenerated byte-identical (proven, not
asserted). The unfreeze ceremony was followed deliberately, per the honor-system note above.

## The headline: round 1's inferences held, but its biggest number moved

Two things happened at once, and they point opposite ways.

**Round 1's reasoning was vindicated at a level worth recording.** Where it inferred from a
secondary and hedged with a "~", the primary landed on the digit. Davidson's kMO2 is the
sharpest case: round 1 estimated ours was "~1200x sharper", "~3 orders", vs "~150 uM",
"~58 % of air saturation" — the primary gives **1210x, 3.08 orders, 149 uM, 58 %**. Van
Hecke's structural floor was reasoned to "sit nearer the low-N end (~0.29 %)" without being
able to see the per-level data; it does, monotonically. The FAO-paraphrase chain for Olson,
flagged in round 1 as a risk, checked out faithfully against Olson himself.

**But the flagship "~28x fast" finding moved, and honesty requires saying so loudly.**
Parton 1987 — item #1, "highest value", precisely because its table gives the *maximum
unmodified* rates that are the true analogue of our bare constants — gives CENTURY's active
SOM pool at **K5 = 0.14/wk = 7.3/yr**. Against that anchor ours is **~2.5x fast, not 28x**.
RothC's BIO = 0.66/yr is confirmed from Coleman & Jenkinson's own model description (not
FAO), so the 28x reading survives too. **The two canonical lineages disagree by ~11x about
what "the microbial pool" is**, and that spread is now the finding: 2.5x and 28x are both
honest and neither is *the* answer. Round 1's "3-28x" becomes **~2.5-28x**.

Both anchors are recorded in `microbial_respiration.yaml` deliberately. Taking the flattering
2.5x would be the **same re-anchoring trap this scope already refused** for
`decomposition_rate` — defending a value by quietly redefining what the parameter means.

## What each source settled

| Source | List # | Outcome |
|---|---|---|
| **Parton 1987** | 1 | K5 (active SOM) = 0.14/wk **max** = 7.3/yr → ours ~2.5x. Index alignment verified 3 ways. |
| **Coleman & Jenkinson RothC** | 8 | BIO = 0.66/yr **primary**, replacing the FAO secondary. Confirms modifiers `a·b·c ≤ 1` multiply `k`. |
| **Olson 1963** | 7 | **Answers its open question: NO.** His own p. 326 estimates span ~0.009-4/yr; ours (7.3/yr) is ~1.8x the top. FAO paraphrase vindicated. |
| **Davidson 2012** | 9 | kMO2 = 0.121 cm³/cm³ air → exactly 1210x. **And it is not an affinity at all** — set equal to modelled ambient O₂ at mean site moisture. A stronger disqualification than round 1's scale argument. |
| **Greenwood 1990** | 10 | W = 14.44 t/ha **is** inside the database — at **~99 %** of its maximum extent (14.59). Valid, not extrapolation; quantifies "late-season asymptote". |
| **Van Hecke 2020** | 11 | Round 1's inference **confirmed as measurement**: 0.29 % at 60 kg N/ha → 0.69 % at 280. Ours (0.5 %) is ~1.7x the floor. |
| **Longmuir 1954** | free | Supplies the **mechanism** round 1 asserted but could not source: apparent Km ~ d^2.6, a *diffusion* artifact, so it is not transferable across a ~5-order size gap. The cross-kingdom copy is wrong in **direction**, not just magnitude. |
| **Azzam 2023** | free | Corroborates FINDING 1 **structurally**: a 2023 study devoted entirely to self-discharge *still* reports no %/month constant, only µA float currents. |
| **Penning de Vries 1989** | 2 | **The prediction was wrong** — see below. |
| **Gilmore** | 12 | Nice-to-have, unused; NASA RP-1121 already binds `emissivity`. |

## Three results worth more than the citations

**1. A retrieval-list prediction was falsified.** Item #2 billed Penning de Vries 1989 as
"the only clean-room-safe route" that would "settle the one param with no accessible
support" (`n_senescence_rate`). It was retrieved. **It does not settle it, because the book
has no N-shedding rate to give** — its senescence is leaf *biomass* death, and it names N
remobilization as a distinct process that is "difficult to quantify", corroborating round
1's diagnosis from the horse's mouth. Retrieval cannot fix a parameter whose quantity the
literature does not publish in that shape.

That failure produced something better: **a new structural finding.** The book's relative
loss rate is a *function of development stage* (0/day before anthesis → 0.15/day at DS 2.0);
**ours is a bare constant** — the degenerate case of the form we cite, non-zero exactly where
the source is zero. That is a **form** gap, not a value gap, which promotes `n_senescence_rate`
from citation debt to a **scope-(B) structural candidate**, alongside the canopy and the
decomposer rates.

**2. A negative result, recorded so nobody re-runs it.** Stanford & Smith 1972 (#6) was not
supplied. A search of the whole 29-source corpus found **no first-hand mineralization k
anywhere** — including Ros 2011, a dedicated N-mineralization review *and* meta-analysis and
the likeliest substitute, which cites S&S for the form and never reproduces its number.
`mineralization_rate` is now the **weakest-provenance** member of the decomposer finding: its
3.9x rests on Schomberg quoting S&S. (The finding survives independently: ours is above
Schomberg's *own* 43-soil maximum, which does not depend on S&S at all.)

**3. The live miscitation risk survived.** Dunn 2011 (#3) and Divya 2009 (#4) — the two
citations *already in our tree* attributing numbers nobody ever checked — were not supplied.
They stay unverified. Flagged loudly in `self_discharge.yaml` precisely because every *other*
open item in that file advanced this round, which makes it easy to misread as settled.
FINDING 1 makes the skeptical read likelier still: if a dedicated 2023 measurement study does
not state a %/month constant, a 2011 grid-storage *review* is an improbable place for one to
be, and the attribution may simply be wrong.

## The discipline that earned its keep

Round 1's one real error was a **misaligned table row** (BVAD wheat read from the soybean
row). Round 2 was mostly tables, extracted by `pypdf`, which collapses columns. Every
positional read was therefore verified against something the paper itself asserts:

* **Parton's K-index** — 3 independent checks: p. 1176 names K6 outright; Eq. [5] names K5;
  and the stated "surface litter 20 % lower than soil litter" holds *exactly* on the
  positional reading (0.076/0.094 = 0.809, 0.28/0.35 = 0.800).
* **Longmuir's Table 1** — reconstructed rows regenerate **his own published regression**
  (log Km = 2.6032·log d − 7.0460) for **9/9 organisms** inside his stated ±0.2605 fiducial
  band. A misaligned read cannot do that.
* **Penning de Vries' LLVT** — the OCR is mangled (`PUNCTION LLVT = 020., 1.,0:, 1:5,0.03;
  2-,0.15`). The reading was confirmed by *arithmetic*: the exercise states the pattern it
  reproduces ("first 15 days, loss is 20 %, in the second 15 days 75 % of what remained"),
  and integrating the reconstructed table gives **20.1 %** and **74.1 %**.

And where a table could **not** be read cleanly, nothing was written: Greenwood's
crop-specific *b* coefficients were left unquoted, and only 9 of his 15 Table 1 rows
extracted — so the 14.59 t/ha maximum is recorded as a **floor** on the true maximum, with no
per-crop attribution. Azzam's per-cell float currents live in figures, so only its abstract's
order-of-magnitude claim was used. Justes 1994 (#5) **was** supplied but is an image-only
scan — it stays unread rather than guessed.

**A trap was also caught.** Longmuir's pig-/ox-heart rows (19-24 nM) are *sharper* than our
herbivory value and could be misquoted as vindicating it. They are subcellular preparations
with the diffusion barrier removed — comparing them to an organism-scale half-saturation is
the very error the paper exists to demonstrate. Flagged in the file.

## What this leaves

**Scope (B) grew again, and in the same direction (A) and (C) already pointed.** It is now
**four** pieces of new science, not three: vernalization, juvenile canopy expansion, the
decomposer rates, and now `n_senescence_rate`'s missing DS-dependence — a *form* change, so it
cannot be absorbed by recalibration.

**The retrieval list is now 3 items, not 12**, and all three are load-bearing in different
ways: **Stanford & Smith 1972** (would firm up the weakest decomposer delta), **Dunn 2011**
and **Divya 2009** (would settle whether our own tree miscites them). Items 1, 2, 7-12 are
discharged — retrieved and read.

**The corpus is not exhausted.** ~15 supplied sources were never opened, because they are
scope-(B) science rather than citation targets — Soltani & Sinclair (*Modeling physiology of
crop development*), Teh, Luo & Smith's *Land Carbon Cycle Modeling*, the soil-microbiology
and soil-modeling texts. Those are exactly the vernalization / canopy-expansion /
decomposer-recalibration references scope (B) would need. They are on disk and gitignored;
scope (B) starts with a fuller shelf than it did this morning.
