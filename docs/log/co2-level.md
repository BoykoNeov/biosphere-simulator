## **The cabin's CO₂ level** (the uncited-value triage's first shortlisted item — the scrubber's rate set the cabin and the crop at 400 ppm, a level nobody chose)

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**OPENED and PRICED 2026-10-19** on the user's *"1"*. A design note and a measurement, lab-only.
No value, golden, manifest or `source:` string moved. **The choice of level is the user's.** Plan
and tables: `docs/plans/post-roadmap-co2-level.md`; tool: `station/examples/co2_level_price.rs`.

* **Today's level is an accident.** The first-order scrubber settles the cabin at crew output ÷
  rate = 3.796 mol, 400 ppm (Earth's outdoor air). The rate was picked for solver stability. BVAD
  Table 4-1 (p. 63, read as the page image) gives plants 0.13 kPa nominal and crew 0.267 / 0.507 /
  1.01 kPa. The ISS runs to a 0.40 kPa cap.
* **BVAD Rev 2 has no scrubber equipment figures.** The review's Step 5 said it did. The only CDRA
  figure in the tree is still `eclss.yaml`'s Watanabe 2015.
* **Cite the level, not the machine.** Sizing the rate from a real CDRA's airflow would carry the
  crew's 13.3× carbon oversizing into the level (≈0.96 kPa). The triage's "÷8" label mistook a
  bed-swing period for a removal time constant.
* **Priced, sealed station, 4 years.** 0.13 kPa: grain +48.6 %. 0.40 kPa: +65.4 %. 0.507 kPa:
  +67.2 %. Every run had `rationed 0`. The crop model saturates and has no high-CO₂ penalty. It has
  been checked against a real chamber only at about 1000 ppm, so the gains above about 1200 ppm
  are extrapolation.
* **The proposed form** is the condenser's: remove only the excess above a cited share of the
  cabin's air. ⚠ A one-sided proportional remover holds `P/k` above its target. That is 400 ppm
  at today's rate, so the form needs a ~10× rate (offset 40 ppm). The authored multirate path
  already breaks `k·dt < 1` when the scrubber is classed slow.
* **Lesson: a first-order remover's level is the load's, not the equipment's.** "Size it from the
  real machine" sounds like realism, but the level then reports the crew's size. Cite the
  quantity the source actually states, here the level.
* **Lesson: a review's "the source is on the shelf" is a claim to grep.** BVAD was on the shelf;
  the equipment figures were not in it.
