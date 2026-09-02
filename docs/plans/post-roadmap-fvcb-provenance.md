# The FvCB constants bound to their sources — the direction plan's standing free item

**Taken 2026-09-02, in an unattended session, as the first item off the September re-read
of the open queue.** The 2026-08-13 direction plan listed *"`Γ*`'s citation"* under "what
does not touch the gate and can move now" — *"one targeted retrieval attempt, not a
reopening of the citation bucket"* — and the 2026-08-31 re-read confirmed it still stood.
It is the floor every chamber CO₂ band is measured against (`Γ*/ci_ratio = 61.07 ppm`),
and the freeze doc's own step-unfreeze entry says why that matters: *"clearing a
provisional threshold is a weaker result than clearing a cited one."*

| | Outcome |
|---|---|
| `gamma_star`, `kc`, `ko` | **BOUND** to Bernacchi et al. (2001) — the values were the paper's to every digit |
| `o2` | **BOUND** to the atmosphere's mole fraction (and [A]'s own `O = 210 mbar`) |
| the five band `source` strings | **CORRECTED** in the same ceremony — they asserted the citation was missing |
| the page-level check | **OWED** — no scholarly host was reachable from the box; stated in the file |
| `regen_goldens` on Linux | **FIXED** — it could not give this ceremony its control (FINDING 2) |
| the eight other `TODO(cite)` in the file | **NOT TAKEN** — named with candidate sources in the direction plan |

The record is `docs/log/fvcb-provenance.md`; the unfreeze-log entry is in
`docs/biosphere-reference.md` (2026-09-02).

---

## 1. What the shelf already said

`photosynthesis.yaml` carried, under `TODO(cite)`:

| param | value | unit |
|---|---|---|
| `kc` | 404.9 | µmol/mol |
| `ko` | 278.4 | mmol/mol |
| `gamma_star` | 42.75 | µmol/mol |
| `o2` | 210.0 | mmol/mol |

Those three kinetic constants, to four significant figures each, are the 25 °C values of
Bernacchi, Singsaas, Pimentel, Portis & Long (2001), *Improved temperature response
functions for models of Rubisco-limited photosynthesis*, Plant, Cell & Environment
24(2):253–259 — the in-vivo Rubisco parameterization the FvCB literature has used as its
default since (Medlyn et al. 2002 adopt it; Sharkey et al. 2007's fitting tool ships it).
No other parameterization on the shelf produces that triple: Teh's specificity-factor
route (`Γ* = O₂/(2τ)`, τ = 2600) gives 40.4 µmol/mol, and the gate
`the_shipped_floor_is_the_conservative_one_against_the_cited_route` has measured that
difference since 2026-08-14.

**So the values were taken from Bernacchi and the file never said so.** This is the fifth
instance of *check your own shelf before treating a value as unsourced* — the four before
it (`canopy-regulator`, `stem-reserve-form`, the SLA anchor, `carbon_fraction`) all found
the missing record inside this repo; this one found it in the literature the values were
copied from. The retrieval the direction plan asked for was one lookup, and the lookup was
of our own file.

`o2` is not a fitted constant at all: 210 mmol/mol is the atmosphere's O₂ mole fraction
(20.95 %) rounded, the O at which [A] runs its model and at which [C]'s constants apply.
It is bound to that, as a property of the air rather than a measurement of a leaf.

## 2. What was NOT checked, at its size

The project's shelf rule is *verified off the page image*. That was not possible here:
`sources/` holds no Bernacchi PDF, and every scholarly host tried from the session's box
(PubMed, Semantic Scholar, Crossref) was blocked by the network policy. The binding is
therefore to the paper's constants **as the literature reproduces them**, and the
`source:` strings and the file header say so in as many words, with the page-level check
named as owed.

What the check can do: confirm digits that already match, or expose a transcription error
this record would then be wrong about. What it cannot do: change the *value* — that was
never on the table, and is why this is provenance-only. ⚠ If the check finds the paper's
Table 1 differs from these digits, the correct move is to record the disagreement, not to
move the number: a value change is a 13-golden ceremony and a separate decision.

⚠ **Species.** Bernacchi fit in vivo on transgenic tobacco. Treating Rubisco kinetics as
conserved across C3 species is the FvCB tradition's convention, not a wheat measurement;
the file names it as a convention. A wheat-specific in-vivo set would be a *value*
question and belongs to the direction plan's provenance queue, not here.

## 3. The five band strings — why they moved in the same ceremony

All five `..._stays_above_the_compensation_point` gates carried a `source` reading
*"⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so the verdict is
provenance-insensitive — test_the_shipped_floor_is_the_conservative_one_against_the_cited_route"*.
Two claims in that string went false with the citation: that `Γ*` is uncited, and (already
false since S6 on 2026-08-27) that a Python test named `test_…` exists. C4's module note had
left the strings alone deliberately — *"editing it would be a value change to the contract
rather than the locus re-anchoring C4 is"* — and named them as residue for the slice that
retired Python. S6 retired Python and the residue stayed.

The freeze's own 2026-08-14 lesson is *a correction lands where it is written, not
everywhere the claim was repeated*, and the 2026-08-31 record adds *a pointer does not
inherit its target's corrections*. Leaving five frozen strings asserting "uncited" beside a
file that now cites would be exactly that. So the strings moved, as frozen `science_bands`
content, inside the ceremony that already had the manifest open. ⚠ The module note said
"two" strings; there were five. Corrected in the note.

## 4. The manifest diff, predicted then run

Predicted: six lines — `param_files["photosynthesis.yaml"]` and the five `source` values.
Nothing else: no golden hash (no number moved), no `flow_set`/`aux_set`, no `bound`.

Run (`cargo run -q -p domains --example dump_biosphere_inventory -- --write-manifest`):
**six lines, exactly those.** `git diff --stat` on the manifest: 6 insertions, 6 deletions.

## 5. FINDING 2 — the regeneration tool could not give this ceremony its control

The ceremony's control is *"`regen_goldens` reports the same thing before and after"*. On
the untouched tree, on this Linux box, it reported:

```
19 of 19 goldens run; 11 would change.
```

while `cargo test` was green (1101 passed). The eleven are every `Numerics::Transcendental`
golden: UCRT-minted, last-ULP different under glibc by design, and compared
**structurally** by `golden_regression.rs` off their generation platform. The tool
byte-compared, so on any platform but Windows it called the reference "moved" on a tree
nothing had moved — and a `--write` there would have re-minted eleven goldens on the wrong
platform and turned the byte-exact Windows gate red (the `ci-python-job-red-on-linux`
hazard with the arrow reversed).

**Fixed in `station/src/regen.rs`:** the tool now reaches the gate's verdict through
`domains::goldens::compare`. A last-bit difference on a transcendental golden off-platform
is reported as `ulp-only`, counted separately in the summary, and **never rewritten, even
under `--write`**. On Windows nothing changes (the gate accepts nothing less than
byte-exact there, and so does the tool). The control test seeds a fresh golden with one
flipped hex digit and asserts the classification, the no-write, and that a real (non-float)
difference is still `CHANGED`. After the fix, on the same untouched tree:

```
19 of 19 goldens run; 0 would change.
⚠ 11 differ only in the last bits of their floats: …
```

which is the sentence the control needed, and the same sentence after the provenance edit.

⚠ This is a tooling fix inside a science batch, against the standing rule *do not take a
science item and a re-anchoring slice in one batch*. It is not a re-anchoring — no contract,
golden or manifest key moved, and the classification it adopts is the gate's own — and the
ceremony could not have been controlled without it. Stated rather than slipped in.

## 6. Two form gaps found while reading the file, neither built

1. **`o2` is a constant; chamber O₂ is a stock.** Every sealed scenario carries
   `chamber_o2_mol0` as a live pool the crop fills and the crew and decomposers draw, but
   the oxygenation term in `rubisco_limited_rate` reads the fixed 210 mmol/mol. The crop
   photorespires against an atmosphere it no longer breathes. In a regulated habitat the
   error is small (the O₂ regulator holds near 21 %); in the unregulated sealed chambers it
   is unmeasured. A measurement is cheap — the O₂ trajectory is in the goldens.
2. **Single-temperature kinetics.** Kc, Ko, Γ* are 25 °C constants and the whole
   assimilation rate is scaled by the [B] cardinal multiplier. [C] gives each its own
   Arrhenius response, from the same paper now cited. That is a *second form* of a
   biosphere process with a citation attached — the thing the science-switch plan's slice
   3b measured the tree as lacking (*"no alternative form of any biosphere process exists
   in the tree"*).

Both go to the September direction plan as candidates for the science switch's first
scientific pair.

## 7. Gates run

Recorded in the log record (`docs/log/fvcb-provenance.md`), with the counts read off the
whole output.
