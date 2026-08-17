# Reuse & Licensing

How we may (and may not) reuse existing models, and why. Checked 2026-06-15.

## TL;DR — the safe default

**Clean-room reimplementation from primary literature, with third-party models
used only as offline validation oracles.** This keeps *our* engine free of
copyleft so the eventual Godot product can carry whatever license we choose.

| Source | License | What we may do |
|---|---|---|
| **PCSE** (Python crop sim env; WOFOST/LINGRA/LINTUL) | **EUPL v1.1+** (copyleft) | **Use as an offline oracle** freely (run it to generate Tier-2 golden fixtures). **Do NOT copy/port its source** unless we accept EUPL on our derivative. |
| **WOFOST_crop_parameters** (YAML coefficient sets) | **No license** = all rights reserved | **Do NOT copy the files.** Source parameter *values* from cited publications instead, or request permission from Wageningen. |
| **WOFOST equations / Farquhar–FvCB / Penman–Monteith** | Published science (not copyrightable) | **Reimplement freely from the papers.** Algorithms and equations are not subject to copyright. |
| **NASA BVAD** (Baseline Values & Assumptions Doc) | US Government work → public domain (US) | Use the values with citation (Phase 6 reference). |
| **NASA BioSim** | Verify when we reach Phase 6 (historically NASA Open Source Agreement) | Architecture reference now; re-check license before any code reuse. |
| **MELiSSA** (ESA closed-loop life support) | Research consortium; software generally not openly licensed | Use *published papers* as an architecture/validation reference, not code. |

## Why EUPL matters here

EUPL is **strong copyleft** (GPL-class). The trigger is *distribution of a
derivative work*:

- **Running PCSE to produce test fixtures is mere use, not distribution of a
  derivative** → no copyleft obligation. Our Tier-2 "golden master" approach
  (run PCSE offline, capture numeric outputs, assert our independent port matches
  within tolerance) is license-clean. Captured numbers are facts, not PCSE code.
- **Translating/porting PCSE's source into our engine *is* a derivative** → if we
  ever distribute it, our core would have to be EUPL. For a (potentially
  proprietary or differently-licensed) game core, that is likely undesirable.
- **The science is free.** WOFOST, Farquhar–von Caemmerer–Berry, Penman–Monteith
  are peer-reviewed algorithms. Reimplementing from the primary literature yields
  code we own outright. PCSE then only ever serves as the verification oracle.

## Project license status (BNCL-1.0 since 2026-06-28)

**Boyko Non-Commercial License v1.0** (see `/LICENSE`). Free to use and modify for
non-commercial purposes; commercial use requires separate written permission from the
copyright holder. Copyright holder: Boyko Neov.

⚠ *This section said **Apache-2.0** until 2026-08-17, and had been wrong since commit
`c205560` (2026-06-28) swapped the license file. Nothing gates a prose claim about the
tree, which is why it survived ~7 weeks — the same failure mode recorded for the freeze
docs. The **clean-room requirement below is unaffected and if anything harder**: a
non-commercial outbound license is no more compatible with pulling EUPL source inbound
than a permissive one was, and the door the clean-room discipline exists to keep open —
choosing our own terms for the eventual Godot product — is exactly the door BNCL walks
through.*

*(History: the license was deliberately deferred from 2026-06-15 until Phase-1; the
interim status was "all rights reserved" on a public repo. Apache-2.0 was chosen
2026-06-17 at the Phase-1 Step-1 precondition — permissive, patent grant — and replaced
by BNCL-1.0 eleven days later.)*

**Clean-room is mandatory regardless of our license** — we never port EUPL (or any
copyleft) code into our core; PCSE stays an offline oracle only. A permissive
outbound license does **not** permit pulling copyleft source inbound.

## The weather fixture is now *distributed*, not merely *used* (2026-08-17, slice C9)

The EUPL analysis above turns on **distribution of a derivative work**, and it settled the
oracle question with *"running PCSE to produce test fixtures is mere use."* Slice C9 changed
which half of that sentence applies to one file: `tests/oracle/winter_wheat_weather.json` is
`include_str!`-embedded into the native core, so it ships **inside the binary** rather than
sitting in `tests/` as offline oracle input. That is a distribution question, and it deserves
an answer written down rather than assumed.

**It is clean, and the reason is that the data never came from PCSE's model.** The rows are
NASA POWER daily observations (`weather_source: NASAPowerWeatherDataProvider` names the
*client* that fetched them, not an author); NASA POWER is US Government work → public domain,
the same footing this document already grants NASA BVAD. Observational facts are not
copyrightable, and a derivative work of PCSE would have to derive from **PCSE**, which a
temperature reading does not. The fixture's own `provenance.description` says as much —
*"raw observational FACTS (not PCSE code, not crop-parameter YAML)"*.

**The rule this leaves for future embeds.** `include_str!` silently promotes a file from test
input to shipped content, and nothing in the build says so. Before embedding any fixture,
check its provenance block: **observational facts and our own generated output may ship;
PCSE-computed model output and WOFOST parameter values may not** — the latter would put
copyleft (or unlicensed) content inside a BNCL-1.0 binary. The one derived quantity in this
file, `VAP`, is a scalar humidity conversion of public-domain readings, which carries no
independent authorship.

## Practical rule for contributors (and Claude sessions)

- Parameters and equations: cite the **paper/report**, not PCSE, in the
  param-file headers and docstrings. The header template + review checklist live in
  `docs/param-file-conventions.md`.
- PCSE lives in the **test/dev dependency** set only, never a runtime import of
  `simcore` (also required by core purity).
- If you find yourself copying a block of PCSE Python, stop — reimplement from the
  reference instead.
