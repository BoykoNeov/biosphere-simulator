# NASA BVAD reference — nominal human metabolic interface values

Primary reference for the Phase-6 Step-9 integrated crew-metabolic validation gate.
Recorded verbatim from the primary source so the validation test and the recalibrated
crew params cite the **document**, not a memory or a secondary quote (the clean-room
discipline in `reuse-and-licenses.md` — BVAD is a US-Government work, public domain).

## Source

**NASA/TP-2015-218570, Rev 2** — *Life Support Baseline Values and Assumptions
Document* (BVAD), M.S. Anderson, M.K. Ewert, J.F. Keener (eds.), NASA Johnson Space
Center, **February 2022**.
Table **3-31**, "Summary of Nominal Human Metabolic Interface Values", **p. 58**.
Retrieved 2026-07-02 from NTRS: <https://ntrs.nasa.gov/citations/20210024855>
(PDF `20210024855/downloads/BVAD_2.15.22-final.pdf`).

Basis (from the table's own notes, p. 58): a standard **82 kg** reference crewmember
(mean 2015 male astronaut), respiratory quotient **0.860** during intravehicular
activity, including a daily exercise regimen (30 min aerobic + 60 min resistive).
Values from 2019 runs of the MetMan (41-Node Man) model correlated to 2018 sweat-test
results. `+m` = consumed by the crewmember; `-m` = rejected.

## Table 3-31 — nominal per-crewmember-per-day values (verbatim)

| Balance | Interface | Units | Nominal value |
|---|---|---|---|
| — | Overall Body Mass | kg/CM | 82 |
| — | **Respiratory Quotient** (molar CO₂/O₂) | — | **0.860** |
| −m | **Carbon Dioxide Load** | kg/CM-d | **1.085** |
| +m | **Oxygen Consumed** | kg/CM-d | **0.895** |
| +m | Food Solids; Mass (without packaging) | kg/CM-d | 0.800 |
| +E | Food Consumed; Energy Content | MJ/CM-d | 12.778 |
| +m | Potable Water Content | kg/CM-d | 3.217 |
| +m | Water in Food Prior to Rehydration | kg/CM-d | 0.760 |
| — | Metabolic Water (produced internally) | kg/CM-d | 0.490 |
| −E | Total Metabolic Heat Load | MJ/CM-d | 12.426 |
| −E | · Sensible Metabolic Heat Load | MJ/CM-d | 6.308 |
| −E | · Latent Metabolic Heat Load | MJ/CM-d | 6.118 |
| −m | Fecal Solid Waste (dry basis) | kg/CM-d | 0.032 |
| −m | Perspiration & misc. Solid Waste (dry) | kg/CM-d | 0.027 |
| −m | Urine Solid Waste (dry basis) | kg/CM-d | 0.061 |
| −m | Fecal Water | kg/CM-d | 0.101 |
| −m | Respiration and Perspiration Water | kg/CM-d | 2.946 |
| −m | Urine Water | kg/CM-d | 1.420 |

RQ note (p. 59): RQ used in the 2019 runs is **0.86 nominal/sleep**, 0.95 aerobic
exercise, 0.96 resistive exercise. The daily-integrated O₂/CO₂ pair above
(1.085 kg CO₂, 0.895 kg O₂) blends nominal + exercise, giving a daily-effective molar
ratio ≈ 0.88 (see conversions below).

## Molar conversions (for our CARBON/OXYGEN mol accounting)

Molar masses: CO₂ 44.009 g/mol, O₂ 31.998 g/mol, C 12.011 g/mol.

- CO₂ load  1.085 kg/CM-d ÷ 44.009 = **24.654 mol CO₂ /CM-d** (= 24.654 mol C respired)
- O₂ consumed 0.895 kg/CM-d ÷ 31.998 = **27.970 mol O₂ /CM-d**
- Daily-effective molar RQ = 24.654 / 27.970 = **0.8814**
- Total metabolic heat 12.426 MJ/CM-d ÷ 86400 s = **143.8 W /CM**

## What this validates (and what it cannot) — the three columns

Our crew flows are **forced** (intake rates are scenario data) and the split fractions
are params, so quantities we *set* match BVAD by construction (calibration, not
validation). The one genuinely un-tuned structural output is the **O₂:CO₂ molar ratio**,
which our `CrewRespiration` fixes at **RQ = 1.0** (PQ = 1: one mol O₂ consumed per mol
CO₂ produced). See `phase-6-station-integration.md` Step 9 and `test_bvad_validation.py`.

- **Calibrated (vacuous — set, then matches):** food-carbon intake, water intake, CO₂
  production, feces, humidity, urine.
- **Structural prediction (genuine):** with CO₂ calibrated to BVAD (24.654 mol),
  RQ = 1 predicts O₂ = 24.654 mol = 0.789 kg vs BVAD 0.895 kg → **O₂ ≈ 11.8 % low**
  (equivalently, calibrating O₂ makes CO₂ ≈ 13.4 % high). RQ = 1 **cannot hit both**.
  This ~12 % miss is the headline result of the step.
- **Not modeled (honest gaps):**
  - *Metabolic water* (0.490 kg/CM-d) — our `WaterBalance` is intake-split only (no
    water produced from food oxidation), so total water in ≠ BVAD's in/out balance.
  - *Metabolic heat* (12.426 MJ/CM-d ≈ 143.8 W/CM) — crew is not an ENERGY source into
    `thermal.node`; an energy-side gap.
  - *RQ variation with activity* (0.86 nominal / 0.95–0.96 exercise) — we carry a
    single fixed RQ.
