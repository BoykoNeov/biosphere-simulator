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

## ⚠ The YAML a param file may use is a CLOSED SUBSET (since 2026-08-17)

**A param file is read by the Rust reference, whose YAML reader is hand-rolled over a
deliberately bounded grammar** (`rust/crates/config/src/yaml.rs`, whose docstring is the
normative list). In practice one rule bites, and it bit us:

> **Write tables in block style. Flow style — `- {dvs: 0.0, fl: 0.55, …}` or `[1, 2]` —
> is rejected, not accepted-and-ignored.**

```yaml
    rows:
      - dvs: 0.0        # ✅ block style: one key per line
        fl: 0.55
      - {dvs: 1.0, fl: 0.30}   # ❌ rejected by the reader
```

Also excluded: anchors/aliases (`&`/`*`), tags (`!!…`), block scalars (`|`/`>`), document
markers (`---`), the YAML-1.1 bool aliases (`yes`/`no`/`on`/`off`), and merge keys (`<<`).

**Why this is written down here.** Until 2026-08-17 the param files were read only by
pyyaml, which accepts all of the above, and `allocation.yaml` **was** in flow style — so the
project's own param files sat outside the grammar the project had frozen for authored files.
Moving the param load into the Rust reference (reference-flip slice C1) made that a build
failure rather than a latent inconsistency, and the two tables were reformatted. The
constraint is now real, is enforced by a test in `crates/domains/src/biosphere/params.rs`,
and was **not** discoverable from this document — which is the *"the freeze's prose half is
ungated"* lesson landing on the one page that governs param-file shape.

⚠ One number that is *not* a style question: an unsigned exponent (`1.0e7`) is resolved by
pyyaml as a **string** and coerced to a float by the schema; only `1.0e+7` resolves as a
number. Both load identically on both ports — the boundary parses the scalar's text — so
this is recorded as a hazard to recognise, not a rule to obey.

## ⚠ A param file's LINE ENDINGS are contract-relevant (since 2026-08-17)

**Both manifests record a `param_files` sha-256 per file, and since reference-flip slice C8
the reference produces it** (`rust/crates/config/src/provenance.rs`). The digest is taken
over **newline-normalized** text — `

` and lone `
` fold to `
`, then one trailing
newline is dropped — so a file's recorded hash is a record of its *content*, not of how git
materialized it.

> **Keep param files LF. Never use a character other than newline as a line break.**

Two reasons, both measured rather than assumed:

1. **A CRLF working-tree copy is not hypothetical here.** `git ls-files --eol` over the 24
   param files shows the index is LF on **all** of them and `.gitattributes` declares
   `eol=lf`, yet one working-tree copy (`senescence.yaml`, on the development box) is CRLF.
   The Rust reference embeds files with `include_str!`, which reads the **working tree** — so
   without normalization the reference would emit a different digest on that box than on
   Linux CI, and the regenerated manifest would be red on the other machine. Normalization
   is what makes the hash portable; it is not a licence to mix line endings.
⚠ **How the state arises — measured, and deliberately stopping where the measurement
stops.** A Python `Path.write_text` on Windows translates `\n` to `\r\n` unless told
otherwise, so *any* tooling that rewrites a file in place leaves a CRLF working-tree copy.
Landing slice C8 produced about **190** of them across this repo without anybody choosing
it. The reason that matters here rather than being cosmetic: **`git status` does not show
it** — git normalizes on read, so a CRLF working-tree copy of an LF-indexed file is
*clean* — while `include_str!` reads the working tree verbatim. So the divergence is
invisible to the tool you would check with and visible to the one that builds the
reference.

⚠⚠ **What is NOT claimed, because measuring it produced a near-miss instead.** An attempt
to "just normalize the working tree" rewrote 191 files and git then reported all 191 as
**modified** — the opposite of the clean-by-normalization behaviour above, for a reason
this document does not pretend to explain. It was reverted (`git diff --ignore-cr-at-eol`
was empty for every one, so the change really was line endings alone). The lesson is the
practical one: **the fix for a stray CRLF file is that one file, not a sweep**, because a
sweep here is a 191-file diff that reaches into `src/` and the purity invariant.

2. **The two hashing rules are only equal on a restricted alphabet.** Python's rule is
   `hashlib.sha256("
".join(path.read_text().splitlines()))`, and `str.splitlines` breaks on
   **eight further characters** the reference's narrow rule does not: vertical tab, form feed,
   the three ASCII separators, NEL, LS and PS. A param file containing any of them would hash
   differently on the two sides. Rather than reimplement a Python method, the reference makes
   the divergence **unreachable**: `config::provenance::EXOTIC_LINE_SEPARATORS` enumerates
   them and a test asserts no frozen param file contains one.

⚠ A digest change with no value change is still an **unfreeze** — and it is the one nothing
catches, because the recorded `param_files` hashes are never compared to enforce a value (the
goldens do that). So a whitespace-only or line-ending-only edit needs the honour-system
ceremony from `CLAUDE.md`: advisor review → regenerate the manifest as the git-visible record
→ an entry in the reference doc's unfreeze log.

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
- [ ] **Inside the closed YAML subset** — block-style tables, no flow style / anchors /
      tags / block scalars. The Rust reference rejects them; see the section above.
- [ ] **LF line endings, and no exotic line-break character.** The file's recorded
      `param_files` sha-256 is taken over newline-normalized text; the two ports' rules are
      only equal on that alphabet. See "line endings are contract-relevant" above.
- [ ] **A whitespace-only edit is still an unfreeze.** It moves a recorded hash and reddens
      nothing — run the honour-system ceremony rather than committing it as a tidy-up.
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
