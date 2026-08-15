# A value-switch harness — "source A vs source B, what changes?" as a command

> ## ⚠ STATUS 2026-08-15 — THE SEAM IS BUILT; THE HARNESS IS NOT. Read this before §5.
>
> The user chose **Option C (the clean seam)** over Option A. It shipped in `666670a` as
> `src/config/overrides.py` + the one-line hook in `config.loader.load_yaml`, with 19
> tests (`tests/test_param_overrides.py`). Full suite **2425 passed / 5 skipped**, up
> exactly the 19 new tests; every golden, both manifests and every science gate green.
>
> ⚠ **§5's price for C was wrong and is now measured.** It claimed an unfreeze question
> via the manifests' completeness half; that half is defined over the param-file set, the
> flow-class set, the horizon constant and file existence, and a boundary hook is in none
> of them. **Nothing in either manifest moved.** The paragraph is kept below as written
> because the correction is the point — see also `memory/asserted-attributions-rot`.
>
> ⚠ **§5's recommendation ("A first, C only if A proves fragile") is SUPERSEDED.** Do not
> follow it. C is the shipped route.
>
> **What is still to build (the actual remaining scope):** §8's runner and comparison
> mode — take a set of substitutions and a scenario, run, and report the readouts with
> §6's five requirements honoured. The seam is the foundation; the reporting layer is the
> deliverable. §9's first target is unchanged.
>
> **⚠ The `extinction_coef` decision is STILL OPEN AND UNTAKEN.** The user delegated the
> choice ("go with the extinction coefficient of your choosing") and the analysis favours
> **0.65** — crop-specific, our exact equation, canopy peak toward mid-band, LAI peak
> moving before anthesis, and the liveness-floor objection dissolved on examination (that
> floor is a self-calibrated continuity tripwire whose own 50-yr subject already sits
> below it at 0.60). **Nothing was changed**: the ceremony did not start, `canopy.yaml`
> still reads 0.6 with its `TODO(cite)`. ⚠ Before shipping 0.65, settle the one live
> operational risk — the perennial liveness floor clears by **0.40 %** and that was
> measured on Windows while CI is Linux, inside the band where libm ULP differences have
> taken locally-green results red. Name that exposure in the commit *before* pushing, and
> do **not** re-anchor the floor in the same batch (that would be indistinguishable from
> the co-adaptation this tree has refused four times).

**PLANNED 2026-08-15, the seam BUILT (see status above), the harness not.** Forward-looking; exempt from
the log index per the paragraph in `docs/post-roadmap-log.md` (call it *"the value-switch
plan"* in any record file — naming it by filename there turns the parity check red).

Successor to the canopy-provenance work (`post-roadmap-canopy-provenance.md`), which
left the `extinction_coef` decision **open and untaken**. This plan does not take it.

## 1. Why — measured, not theoretical

Answering one parameter question in one session cost **three hand-written probes**, all
thrown away:

| probe | what it did |
|---|---|
| `probe1_k_sweep.py` | patched `plants.load_canopy_params`, `dataclasses.replace(..., extinction_coef=k)`, re-ran the season, read peak LAI / harvest / DVS-at-peak |
| `probe2_chamber_bands.py` | drove the CO₂ band's own `_season_low_ppm` helper at each `k`, with cache clearing |
| `probe3_floor_margins.py` | read the liveness floors and their clearances at each `k` |

Three instances of one tool in one sitting — and that is the small number. ⚠ **Counted
across the tree rather than asserted: 42 distinct probe scripts are named in 16 plan
docs**, from `phase-2-closed-chamber.md`'s `bench/probe_multiyear.py` through
`probe1_sizing.py … probe10_which_stock.py` (crew-coupled loop), `probe.py … probe7.py`
(allocation headroom), and the three above. Nearly all live under
`M:/claud_projects/temp/`, outside the repo, and are gone.

**The tool is rebuilt every time a question is asked and deleted every time it is
answered** — 42 times. This plan was first drafted claiming "at least three more times";
measuring it moved the number by an order of magnitude, which is itself the argument.

**The gap this fills is specific.** The tree already has:

* a **documentation** answer to "which numbers are soft" — the three provenance classes
  (CITED / DESIGN / TODO(cite)) in `docs/param-file-conventions.md`, with `source:`
  presence loader-enforced per param file;
* a **finding** answer — the habit of pinning a disagreement as a test
  (`test_gap_2_the_headline_the_two_sources_disagree_on_tuber_onset`,
  `test_the_two_source_tables_disagree_by_an_order_but_agree_on_the_form`,
  `test_cited_and_fitted_partitions_agree_BY_CONSTRUCTION_so_it_is_no_evidence`).

It has **no experiment answer**. Asking "what would source B's value do?" costs a
hand-written patch script — and a hand-written patch script is one careless edit away from
being a real change to the tree.

## 2. The distinction this is built on (the user's, 2026-08-15)

> *"a central place to switch between critical model values — this is a great basis for
> experimentation. Switching this for that results in such and such consequences."*

A **register** is a list you read; it decays, because nothing forces it to stay true. A
**switch** is a thing you run; it cannot decay, because you re-run it. A previous session
argued against the first and, by conflating them, accidentally argued against the second.

⚠ **A central register remains out of scope and is still the wrong build** — it would
relocate information that already lives in enforced places, and
`memory/context-budget-relocation-is-not-a-discipline` is exactly that lesson. Prose and
references stay where they are (param `source:` tags, plan docs, pinned tests); what this
adds is the *reproducible command*, so a future A-vs-B table is regenerated rather than
quoted.

## 3. The safety property that is the whole point

**The harness overrides in memory and writes nothing.** It reads the frozen YAML,
substitutes values for the duration of one run, and touches no file. Therefore:

* no param YAML is edited → no per-file sha-256 moves in either manifest;
* no golden is regenerated;
* no `science_bands` / `liveness_floors` entry moves;
* `git diff src/` stays empty outside `src/lab/`.

**Experiments become cheap precisely because they cannot accidentally become commitments.**
A harness that wrote YAML would be a calibration tool wearing an experiment's name, and
"changing the number is calibration, a separate act" is already the convention doc's rule.

## 4. Placement

`src/lab/`. It already holds `oracle_match.py`, `convergence.py`, `rk45.py` — the
offline-study area, outside the frozen core and outside the port's mirror obligation.
`simcore` purity is untouched: `lab/` may import domains, never the reverse.

## 5. THE OPEN DESIGN DECISION — the injection seam

**Measured 2026-08-15: there is no injection point.** `build_season(scenario)` takes only a
scenario; `_carbon_context` and `build_plants` call `crop_param_set(scenario.crop)` and then
`load_*_params(crop.paths[...])` **internally** (`src/domains/biosphere/plants.py:99-120`,
`:136-146`). Pre-loaded param objects cannot be passed in. Two routes:

### Option A — patch the loaders at their use site (RECOMMENDED to start)

What the three probes did. Wrap `plants.load_canopy_params` (etc.) so it calls through and
applies `dataclasses.replace`. **Zero `src/` change outside `lab/`; proven to work three
times already.**

Costs, stated rather than discovered later:
* the patch must target the **use** site (`plants.load_canopy_params`), not the definition
  site (`loader.load_canopy_params`) — patching the latter silently no-ops;
* the harness must know which module imported which loader, so the mapping is data the
  harness carries and that data can go stale when an import moves.

### Option C — add a narrow injection seam in `src/`

Let the scenario or `build_season` accept pre-loaded params. Cleaner and immune to both
costs above, but it changes shared code under `src/`.

⚠ **Its price is NOT established — this was asserted while drafting and is unverified.**
The claim was that an added public argument falls under the freeze manifests' completeness
half ("something added but exercised by nothing"), making it an unfreeze question rather
than a refactor. **That half is defined over the params/flows/aux the manifest names**, and
whether an optional keyword argument on `build_season` is in that set at all was never
checked. **Verify before choosing C; do not inherit this paragraph as a finding.** (The
shape it would otherwise repeat is `memory/asserted-attributions-rot`.)

**Still not recommended first** — take it only if A proves fragile in practice.

*(Option B — writing temporary crop directories to disk — is rejected: `CROPS_DIR` is a
module constant under `src/`, so this would write into the repo tree, losing §3.)*

## 6. Requirements earned by the canopy-provenance session

Each of these exists because reporting *without* it produced a wrong read on 2026-08-15.

1. **Report distance-from-degenerate, never clearance-vs-bound alone.** "The perennial
   liveness floor's clearance falls 5.12 % → 0.40 %" reads as a plant nearly dying. Against
   the measured stunted-regime baseline (0.253, `test_senescence_form.py:2382`) the plant
   moves **2.29× → 2.18×**. The first number alone misled this session's own recommendation.
   Report both, always.
2. **Label each gate's authority in the output.** `science_gates.py:44-47` keeps
   `science_bands` (bound from outside the repo) and `liveness_floors` (tuned to our own
   calibration) deliberately apart: *"merging two claims of different strength under one
   name is this project's recorded failure mode."* A flip list that does not say which kind
   flipped commits exactly that merge.
3. **Opposed movement is a first-class result, not a footnote.** At `k = 0.65` all five
   chamber CO₂ bands loosen while the liveness floor tightens, *for one reason*. Reading
   either family alone gives the wrong answer. The report must surface the opposition.
4. **Never store a ranking; re-derive it.** "The tightest of the five" inverted in six
   commits (`d8f5583`).
5. **State what did NOT move.** The 3.5× uniform-perturbation amplifier did not transfer to
   this knob (+8.3 % on `k` → +0.8 % of shipped peak LAI). A null result is the finding.

## 7. ⚠ The failure mode this harness is structurally prone to

**A probe that shims a path nothing calls proves nothing.** This tree has already shipped
that defect: `cc44b41` repaired a cross-port gate that had gone vacuous because its ULP
probe shimmed a dead `exp` — and the audit behind `post-roadmap-canopy-provenance.md` found
`intercepted_fraction` now has **no caller in `src/` at all**, surviving only in tests and
the Rust mirror.

A value-switch harness has exactly this failure mode by construction: patch the wrong
symbol, get a clean run, read the baseline, and report "no effect" as a finding.

**Requirement: the harness must prove the substitution took effect, or fail loudly.**
Minimum: assert the run's outputs differ from baseline for a value known to matter, and
refuse to report "no change" without having verified the override was live. A silent no-op
must be impossible, not merely unlikely.

## 8. Scope

**In:**
* one module in `src/lab/` — take a set of `{param: value}` substitutions plus a scenario,
  run, return the readouts (canopy peak, DVS at peak, harvest, chamber CO₂ lows, every
  `science_gate` bound with its margin *and* its distance-from-degenerate, rationing count);
* a comparison mode — baseline vs N variants, tabulated;
* the live-override self-check of §7;
* tests for the module.

**Out:**
* a central register (§2);
* changing **any** parameter value, in YAML or otherwise;
* the `extinction_coef` decision — still open, still the user's;
* the 0.65 ceremony (13 goldens, both manifests, Rust mirror, 3 characterization re-pins);
* touching the perennial liveness floor. It is *misread*, not broken — moving 0.55 is the
  co-adaptation refused four times, and its own comment says so.

## 9. First target

Reproduce the canopy-provenance A/B table as one command: `k ∈ {0.60, 0.65, 0.68}` against
the open-field season and the five chamber gates.

⚠ **The harness must not be read as taking the decision.** It regenerates the evidence the
decision was already priced on; the choice between Penning de Vries (0.60, coherent with the
Goudriaan quadrature shipped 2026-08-15) and Soltani & Sinclair (0.65, wheat-specific, our
exact equation) stays open.

**Then the second layer:** whichever value ships, pin the disagreement as a test — three
sources, three values, two not independent, and the measured consequence. The harness
produces the number; the pinned test freezes the finding. They are complements, not
alternatives, and §1 lists the three precedents.

## 10. Exit criteria

* `git diff src/` empty outside `src/lab/`; no manifest hash, golden, or gate bound moved.
* The §9 table regenerable by one command, on a tree with `extinction_coef` unchanged.
* A test proving a mis-targeted override is **detected**, not silently reported as no-change.
* Suite green (`uv run pytest -n 12`), `ruff`, `pyright`.
* `simcore` imports unchanged (stdlib only).
