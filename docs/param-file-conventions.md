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
- [ ] **Every** value has a `source:` tag resolving to a `Sources:` entry.
- [ ] No value copied from the `WOFOST_crop_parameters` YAML or PCSE source (the
      values are *independently* literature-derived — the oracle match is behavioral,
      so they need not, and should not, equal WOFOST's).
- [ ] Units declared per value; amounts are unit-validated and per-area rate-law
      params carry their declared unit (P4 — converted via `ground_area` in
      `evaluate`).
- [ ] The flow/aux docstring that consumes the file states its rate-law dimensions
      and cites the same primary source.
