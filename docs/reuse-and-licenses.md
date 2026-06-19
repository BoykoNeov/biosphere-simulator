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

## Project license status (chosen 2026-06-17 — Apache-2.0)

**Apache License 2.0** (see `/LICENSE`), chosen at the Phase-1 Step-1 precondition
(real crop code is landing). Permissive, so it keeps the core free of copyleft — the
whole point of the clean-room discipline below was to leave that door open, and a
permissive license keeps it open for the eventual Godot product. Apache-2.0 over MIT
for its explicit patent grant. Copyright holder: Boyko Neov.

*(History: the license was deliberately deferred from 2026-06-15 until Phase-1; the
interim status was "all rights reserved" on a public repo.)*

**Clean-room is mandatory regardless of our license** — we never port EUPL (or any
copyleft) code into our core; PCSE stays an offline oracle only. A permissive
outbound license does **not** permit pulling copyleft source inbound.

## Practical rule for contributors (and Claude sessions)

- Parameters and equations: cite the **paper/report**, not PCSE, in the
  param-file headers and docstrings. The header template + review checklist live in
  `docs/param-file-conventions.md`.
- PCSE lives in the **test/dev dependency** set only, never a runtime import of
  `simcore` (also required by core purity).
- If you find yourself copying a block of PCSE Python, stop — reimplement from the
  reference instead.
