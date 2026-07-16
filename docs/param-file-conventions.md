# Parameter-file conventions (clean-room discipline)

How crop/biology parameter files are authored so the project stays clean-room from
primary literature and free of the unlicensed WOFOST coefficient YAML. Established at
Phase 1, Step 3 (P5). Companion to `docs/reuse-and-licenses.md`.

Phase 1 ships **no crop param files yet** — they land per process (Steps 4–10). This
file fixes the convention *now* so every process step follows it. An automated
header-presence check is deferred until the first param file exists (anti-speculation;
nothing to check against today).

## The rule (non-negotiable)

Every parameter **value** is sourced from a **cited primary publication** (or a
public-domain dataset such as NASA BVAD) — **never** copied from the unlicensed
`WOFOST_crop_parameters` repo or transcribed from PCSE source. PCSE/WOFOST is an
**offline oracle only**: we compare our independently-parameterised model's
*trajectory* to PCSE's output within a tolerance band (`lab/oracle_match.py`); we do
not import its coefficients. See `docs/reuse-and-licenses.md` (the param YAML is
all-rights-reserved: "Do NOT copy the files").

If you cannot find a literature source for a value, that is a flag — request the value
from a citable source, mark it provisional with a `TODO(cite)`, or omit the process —
do **not** fill the gap from the WOFOST YAML.

## Two provenance classes: a CITED value and a DESIGN choice

*Added post-roadmap (bucket 3 scope C, 2026-07-16), when discharging the `TODO(cite)`
debt on the 27 params no oracle can validate. See
[`docs/plans/post-roadmap-citation.md`](plans/post-roadmap-citation.md).*

The rule above presumes every value *has* a citation. **Many do not — and cannot.** A
param file's `Sources:` block may support either of two very different things:

* **the functional FORM** — "first-order gas scrubbing is textbook" (Seader), "P-control
  of a first-order process is textbook" (Ogata), "litter decays exponentially" (Olson);
* **the VALUE** — a source that names a number, or a range our number sits inside.

A form citation does **not** license a value. Seader establishes that CO₂ scrubbing is
first-order; he says nothing about whether *our* rate is `1e-3 /s`. There is no primary
source for *"our station's radiator is 10 m²"*, or for a heat capacity chosen so
`τ >> dt` — those are **modelling choices**, and dressing one in a citation is exactly
the fabrication this document exists to prevent. **A wrong citation is worse than an
admitted gap:** `TODO(cite)` is honest, whereas a fabricated locus survives review by
looking finished.

So a value's `source:` tag resolves to exactly one of:

| class | when | tag |
|---|---|---|
| **CITED** | a primary source supports the value | `source: "[A], Table 2: …"` — the locus **must** have been opened |
| **DESIGN** | the number is a sizing/modelling choice; no source can fix it | `source: "DESIGN — <what kind of choice>, not a literature value: <rationale>"` |
| **TODO(cite)** | genuinely unresolved — still looking | `source: "TODO(cite) — provisional…"` |

A DESIGN tag is a **positive, finished statement**, not deferred debt: it records that
someone looked, established no source could exist, and says what the number *is* instead
(a sizing choice, a stability constraint, a chosen behaviour). It still names the form
citation where one exists — the form is cited, the value is ours:

```yaml
  heat_capacity:
    value: 1.0e7
    unit: "J/K"
    source: "DESIGN — sizing choice, not a literature value: node thermal mass ~2.4 t
      water-equivalent, sized so the radiator relaxation time τ = C/(4εσA·T_eq³) >> dt
      (well-fed sizing, keeps Euler off the backstop). Radiative form from [A]; the
      magnitude is ours. Not calibrated."
```

**Three rules for the DESIGN class:**

1. **State the rationale, not just the label.** "DESIGN — illustrative" is useless; the
   reader needs *why this number* — the constraint it satisfies (`k·dt < 1`), the
   behaviour it produces, or the scale it represents.
2. **Never use DESIGN to dodge a findable citation.** It is for numbers no source *can*
   fix, not for numbers you did not look for. Look first.
3. **DESIGN does not mean arbitrary.** Where a real system's figure is known, record it
   as context so a reader can see whether ours is plausible or wildly off — even though
   the value stays a choice.

**Citing ≠ calibrating.** Binding a value to a source that merely *permits* it does not
validate the model, and neither class asserts correctness (`docs/authoring-reference.md`,
"Frozen is not calibrated"). And when a source *disagrees* with a frozen value, the
disagreement is recorded as a **finding** — **changing the number is calibration**, a
separate act with its own unfreeze discipline and moved goldens.

## Param-file header template

Every param file opens with a provenance header citing the source of each value:

```yaml
# <crop> — <process> parameters
# Currency/units: <e.g. CARBON in mol; per-area rates in umol/m^2/s>
# Clean-room: values from cited primary literature ONLY. NOT from the unlicensed
#   WOFOST_crop_parameters YAML or PCSE source. See docs/param-file-conventions.md.
#
# Sources:
#   [A] Farquhar, von Caemmerer & Berry (1980), Planta 149:78-90.
#   [B] Monteith (1965), Symp. Soc. Exp. Biol. 19:205-234.
#
name: winter_wheat
process: photosynthesis
parameters:
  vcmax25:
    value: 80.0
    unit: "umol/m^2/s"
    source: "[A], Table 2"        # every value carries a source tag
  # ...
```

Each value carries an inline `source:` tag resolving to an entry in the header's
`Sources:` block (paper/report + table/page). A reviewer can audit every number to a
citation without leaving the file. If a value is not yet bound to a primary source,
mark it `source: "TODO(cite) — provisional…"` rather than fabricate a citation or
backfill from the WOFOST YAML.

**Unit notation (pint).** Declared units must be pint-parseable: write powers and
quotients with `^`/`**` and `/`, e.g. `"m^2/kg"`, `"umol/m^2/s"`, `"mm/day"`. Do
**not** use the implicit-product `"m2 kg-1"` / `"umol m-2 s-1"` form — pint reads
`kg-1` as `kg minus 1` (a `DimensionalityError`) and does not know `m2`. Per-area
rate params are validated/converted at the boundary by `config.convert` (Scope-A
discipline) when they have a fixed target unit, or recorded-and-trusted per P4 when
they feed a deferred per-leg `Flow` dimensional check.

## Review checklist (per param file / PR)

- [ ] Header present with a `Sources:` block and the clean-room notice.
- [ ] **Every** value has a `source:` tag resolving to a `Sources:` entry — **or** a
      `DESIGN` tag carrying its rationale (see "Two provenance classes").
- [ ] **Every cited locus was actually opened.** No page/table/figure reference is
      written from a search snippet, an abstract, or memory. If it could not be read,
      it is not cited — cite the verifiable range instead, or mark `TODO(cite)`.
- [ ] **No form citation is passed off as a value citation.** If the source only
      establishes the equation, the value is `DESIGN` (or `TODO(cite)`), not `[A]`.
- [ ] No value copied from the `WOFOST_crop_parameters` YAML or PCSE source (the
      values are *independently* literature-derived — the oracle match is behavioral,
      so they need not, and should not, equal WOFOST's).
- [ ] Units declared per value; amounts are unit-validated and per-area rate-law
      params carry their declared unit (P4 — converted via `ground_area` in
      `evaluate`).
- [ ] The flow/aux docstring that consumes the file states its rate-law dimensions
      and cites the same primary source.
