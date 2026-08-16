# Post-roadmap: the reference flip — Rust becomes canonical

**Status: PLANNED 2026-08-16, NOTHING BUILT.** `git diff src/` empty, `git diff rust/`
empty, no golden regenerated, no manifest touched. Everything below §3 is a proposal;
everything in §2 is a measurement taken today on frozen `main`.

**The decision.** The user re-opened the Rust question on 2026-08-16 ("let's discuss again
the complete switch to Rust") and, given the fork below, chose **target state B of
`post-roadmap-rust-primary-pivot.md` §3 — flip the reference: Rust becomes canonical,
Python becomes the checker.** I priced B's cost and named what it gives up (§4); the user
took it anyway. That is their call and this doc executes it. The cost is recorded once, in
§4, as a **documented consequence — not an argument to be re-run at each slice.**

**The user's second instruction, which shapes this whole doc: *"only plan now, work in
different slices. Don't bundle the whole work into one slice."*** So §5 is eleven slices,
each landable on its own, ordered so the two genuinely unknown pieces are de-risked before
anything with blast radius moves. There is deliberately **no big-bang slice**.

---

## §1 Why the question came back — the pivot's premise was empirically false

Option A (2026-07-20) split work by *type*: new content and gameplay land Rust-first and
owe no Python mirror; validated science stays Python-canonical and mirrors to Rust. The
plan doc justified the split with an estimate: *"~90 % of remaining work is **content +
gameplay** … the remaining ~10 % is residual science calibration."*

**Measured on 2026-08-16, over the 157 commits since that decision:**

| Area | Commits / files touched since 2026-07-20 |
|---|---|
| `godot/` (the product front-end) | **0 commits** |
| `scenarios/` (authored content) | **1 commit** |
| `src/domains/` (Python science) | 122 file-touches |
| `rust/crates/domains/` (the mirror) | 71 file-touches |
| `tests/`, `docs/` | 463, 323 file-touches |

The mix was not 90/10. It was approximately **0/100**. ⚠ **Rule 1 — "new content is
Rust-first" — has never once fired, because no work of the kind it governs has been done.**
The project has spent a month running in exactly the *"both ports, Python-first"* mode the
user declined when A was chosen, not because the policy changed but because **the policy
only had opinions about work that wasn't happening.**

That is the honest reason the question returned, and it reframes it: A was not *wrong*, it
was **inert**. The felt cost is the double build on every science item, which A's rules
never addressed because A assumed science items would be rare.

⚠ **A related miss, recorded because it explains why the posture never became a default:**
§6 step 1 of the pivot plan called landing Rules 1–5 in `CLAUDE.md` *"the single most
important step; the rest is tooling."* **It was never done.** `CLAUDE.md` has no
development-posture section; the posture lives only in that plan doc and a memory file. A
standing posture rule is squarely the *"what you need before you know what you're working
on"* category `CLAUDE.md` exists for — structurally distinct from the finished-work rows
`docs/context-budget.md` retires from it. Slice 11 closes this, for B's posture rather than
A's.

## §2 What was measured today — and two of my own worries, retracted

Every number here was taken on frozen `main` with nothing built. **Two of the three costs I
led with were refuted by measurement, one was confirmed and one was found that neither the
old plan nor the advisor had priced.**

### 2a ⚠ RETRACTED: "regenerating the goldens from Rust will disturb the science gates"

I raised this as the reason B might be expensive, on the reasoning that the cross-port
contract is a **tolerance** contract (so the ports are *not* bit-identical), and that
several science gates now sit at sub-percent clearance — the perennial liveness floor is at
**0.40 %** after the canopy-provenance work.

**Refuted by the contract's own measured bands** (`tests/crossport/tiers.json`,
`docs/native-port-reference.md`): the Tier-2 bands are **`1e-11` to `1e-12` relative**, with
a `1e-12` floor throughout — the two ports agree to about **eleven significant figures**.
The tightest science gate in the tree is `4e-3`. Regenerating from Rust therefore perturbs
the goldens roughly **eight orders of magnitude below anything that can go red.**

**Numerically, the flip is nearly free. The entire cost of B is structural.** This is the
single most decision-relevant fact in this doc and it points *toward* B, not away.

⚠ Do not over-read it: the bands are the **measured ±1-ULP transcendental sensitivity**
plus margin, and CI is the genuine cross-libm gate (Linux glibc Rust vs UCRT-generated
goldens). "Eight orders of margin" is a statement about *magnitude*, not a licence to skip
predicting the diff before regenerating (`soil-layers`' own lesson).

### 2b ⚠ RETRACTED: "the completeness gates can't port, because Rust has no introspection"

The three freeze manifests' gates (1,367 lines across
`test_freeze_manifest.py` / `test_station_freeze_manifest.py` /
`test_authoring_freeze_manifest.py`) own **completeness** — something added to the tree but
exercised by nothing. I assumed they introspect the Python *module namespace*, which Rust
cannot do.

**They don't.** `_flow_set()` (`tests/test_freeze_manifest.py:183`) is explicitly
*"derived, never hand-listed"* — it **builds the canonical registries and reads
`registry.flows`**, taking `type(flow).__name__`. `_aux_set()` is symmetric over
`registry.aux_processes`. That is a **runtime enumeration of what is actually wired into a
built scenario**, and Rust can do it identically: `rust/crates/simcore/src/flow.rs:81`
already declares `trait Flow { fn id(&self) -> &str; }`, and `auxiliary.rs:29` the same for
aux.

**Measured derived-vs-frozen ratio across the three gates** (the number the advisor
correctly identified as the real size of this item):

| Gate | Tests | Assertions against a derived-from-tree helper | Deliberately hard-coded literals |
|---|---|---|---|
| biosphere | 14 | 8 | **2** |
| station | 10 | 7 | 0 |
| authoring | 11 | 8 | 0 |
| **total** | **35** | **23** | **2** |

The two anti-derived sites are both in the biosphere gate and both concern the time step
(`tests/test_freeze_manifest.py:254` *"Do not 'simplify' either one to import `BIO_DT`"*;
`:479` *"⚠ KEEP THESE HARD-CODED LITERALS"* — a gate that imports its own value from the
code auto-follows the code, which is the opposite of a gate). Those get **re-authored by
hand** against the Rust tree, as judgements about what the literal guards. Everything else
is mechanical.

⚠ So this item is *a real rewrite with two judgement calls in it*, **not** 1,400 lines of
new thinking. The advisor's concern was right in kind and small in size, and I had it
backwards in the other direction an hour earlier — **both readings were corrected by
reading the file rather than reasoning about the language.**

### 2c CONFIRMED and larger than the old plan said: the crop-model comparison

`src/lab/oracle_match.py` — the comparison against WOFOST/PCSE — is **pure arithmetic over
`Sequence[float]`** (`nrmse`, `max_abs_relative_deviation`, `within_band`). No engine
objects, no Python state. That half ports or is simply re-called, freely.

**But its callers do not.** `tests/test_oracle_gap.py:103,175` call `build_season()` and
walk the season **in-process, day by day**, building the candidate series in memory; the
reference is a committed JSON `["trajectory"]`. Rust's 24 `emit_*` examples produce
**final-state snapshots**, not day-by-day histories.

⚠ **So B needs a per-step trajectory export from Rust that does not exist today.** That is
a new interface, not plumbing — and it is the one item in this plan where I would expect
surprises. It is why slice 1 exists.

### 2d The PCSE laboratory is banked for running, live for minting — and was woken 5 days ago

`pyproject.toml:29` — `oracle = ["pcse"]`, an **optional** dependency group behind an
opt-in `-m oracle` marker, `importorskip`-guarded, skipped cleanly when absent. The
everyday suite runs entirely off committed traces (`winter_wheat_reference.json`,
`spring_wheat_reference.json`, `potato_reference.json`). **Minting a new trace needs live
Python**, and one was minted **2026-08-11** for the potato.

This was the A-vs-C hinge and it is recorded here only as context: **B does not retire
Python**, so the laboratory survives the flip. It does, however, make 2c load-bearing — the
laboratory now has to compare *Rust's* trajectory against those traces.

### 2e The trap: unit validation becomes a green test guarding nothing

Python's `config/` boundary validates units (pint) as it loads every param YAML and hands
the core a canonical-unit label. **After the flip Rust is the loader for the canonical
run.** Leaving the Python check in place as a lint over the same files leaves it *passing*
while the path that actually executes is unvalidated.

⚠ That is precisely the defect shape already in this project's log — *the pin that should
have seen it had been reading one registry twice since the day it was written*
(`docs/log/step-unfreeze.md`). **This needs a decision, not a leave-in-place**, which is
why it is slice 9 and not a footnote.

### 2f The other four items, priced and small

- **The 25 goldens**: `rust/crates/{domains,station,authoring,simcore}/examples/` carry
  **24** `emit_*` programs against **25** golden files. One is missing or one program emits
  two; identifying which is slice 4's first act.
- **The two-ports-agree check**: `tiers.json` + the comparator invert as configuration.
- **The mathematical-law tests**: **12** `@given` sites across 13 files — conservation,
  non-negativity, order-independence. Rust carries **295** tests already and has **no**
  `proptest` dependency today. Small. ⚠ Note the "56 k lines of Python tests" figure is
  real but is overwhelmingly worked-example, regression and science-gate tests, **not**
  laws; quoting it as the reimplementation bill would be wrong.
- **The class-name / instance-id mismatch**: Python freezes `type(flow).__name__` (a
  **class**); Rust's `Flow::id()` is an **instance** identifier — one class may be
  instantiated per compartment with distinct ids, so they are not the same set. Rust needs
  a `type_name()` on the trait. ⚠ **That is a change to the frozen Rust core, so the
  opening move of making Rust the reference is an unfreeze of Rust.** Not a blocker; worth
  seeing.

## §3 What B is, precisely — and what it is not

**Is:** the goldens are generated from Rust; the three freeze manifests are equated against
the **Rust** tree; the cross-port contract inverts (Python is now judged faithful to Rust);
new *reference science* is authored Rust-first under the unfreeze discipline.

**Is not:** a retirement of Python (that is C, and 2d keeps it off the table while the
laboratory can still mint traces); a change to any scientific result (§2a — nothing moves
above `1e-11`); or a licence to stop keeping Python green (§4).

## §4 The cost, recorded once

**After the flip the two implementations stop being independent.** Python is fitted to
Rust rather than checked against it, so a disagreement can no longer arbitrate a shared
conceptual error — it is resolved in Rust's favour by definition.

**What that has cost historically, concretely:** the scope-B increment-1 mirror caught a
year-2 vernalization reset bug; the multi-rate phase's zero-coverage driver was caught the
same way; and **post-pivot, 2026-08-12, the stem-reserve mirror caught a scenario constant
the entire Python suite could not see** (`docs/log/stem-reserves.md`). That third one is
the important one: the mechanism was still paying out four days before this decision.

**This is a documented consequence of a decision the user made with the cost stated. It is
not re-litigated per slice.** What survives the flip and must be kept: Python stays green
(A's Rule 5 applies unchanged, with the roles swapped), so a *disagreement* is still
detected — what is lost is the ability to conclude that **Python** is the correct side.

## §5 The slices

Each slice lands on its own, has its own commit(s), and is listed with its blast radius.
**Slices 1–2 build nothing that the reference depends on** — they exist to de-risk 2c and
2f before anything with a contract behind it moves. Nothing here is scheduled; the user
takes them one at a time.

| # | Slice | Depends on | Blast radius | Reversible? |
|---|---|---|---|---|
| 1 | **Rust per-step trajectory export** | — | additive; no contract | yes |
| 2 | **`type_name()` on `Flow`/`Aux`** | — | frozen Rust core (small unfreeze) | yes |
| 3 | **Rust dumps its own inventory, checked against the *existing* manifest** | 2 | additive test only | yes |
| 4 | **Find the 25th emitter; regenerate the goldens from Rust** | — | **25 goldens** | yes (git) |
| 5 | **Invert the cross-port contract** | 4 | `tiers.json` + comparator | yes |
| 6 | **Re-anchor the biosphere manifest to Rust** | 3, 5 | freeze contract 1 | ceremony |
| 7 | **Re-anchor the station manifest** | 6 | freeze contract 2 | ceremony |
| 8 | **Re-anchor the authoring manifest** | 6 | freeze contract 3 | ceremony |
| 9 | **Unit validation: decide and build** | 6 | `config/` + Rust loader | design decision |
| 10 | **The 12 laws get Rust equivalents** | — | additive (`proptest`) | yes |
| 11 | **Purity invariant re-written; posture landed in `CLAUDE.md`; memory + log** | all | docs + one `CLAUDE.md` section | yes |

**Slice 1 — the per-step trajectory export (do this first).** 2c is the only unknown with
real surprise potential, and it is *additive*: Rust gains a way to emit a per-step series,
nothing consumes it yet, no contract notices. Acceptance: the exported series for one
biosphere scenario reproduces the Python in-process series inside the scenario's own
Tier-2 band. ⚠ Design question to settle **inside** this slice, not before: which
quantities, and at what step granularity (the step is now `dt = ¼`, so a "day" is four
rows — and `docs/log/step-unfreeze.md` records seven distinct days-vs-steps conflations
found the last time this distinction was assumed rather than checked).

**Slice 2 — `type_name()` on the traits.** Small, and the first frozen-Rust unfreeze. Add
`fn type_name(&self) -> &'static str` to `Flow` and the aux trait, implemented so it
matches the Python class name exactly. Acceptance: no golden moves (it is a pure addition),
`cargo clippy --all-targets -D warnings` clean. ⚠ Use `rustfmt <file>`, **never bare
`cargo fmt`** — it reformats the frozen simcore tree.

**Slice 3 — Rust dumps its inventory, checked against the *existing* Python manifest.** The
pilot with zero blast radius: a Rust program builds the canonical registries and dumps the
flow set / aux set / param file list as JSON; a **new, additional** Python test asserts it
equals the manifest that is still Python-anchored. ⚠ **If Rust's inventory and Python's
manifest disagree, that is a finding to hunt before any re-anchoring** — the completeness
contract is the thing the flip is riskiest for, and this proves Rust can express it before
anything depends on it.

**Slice 4 — regenerate the goldens from Rust.** First identify which of the 25 golden files
has no `emit_*` program (24 exist). ⚠ **Predict the diff before regenerating** — the
prediction is `< 1e-11` relative on every value and *no* structural field moving at all
(Tier 0 is exact at every tier). A structural diff, or any value beyond band, is a port
bug to hunt and **stops this slice**.

**Slice 5 — invert the cross-port contract.** Python now the tolerance-gated side, Rust the
exact one. ⚠ The Tier-2 bands were measured as **±1-ULP sensitivity propagated through the
Rust-side transcendentals**; inverting the roles does not automatically make them valid in
the other direction. Re-measure rather than re-use, and update
`docs/native-port-reference.md`'s prose half — which **no gate checks**
(`docs/log/freeze-prose-half-is-ungated.md`).

**Slices 6–8 — re-anchor the three manifests, one per slice, biosphere first.** Biosphere
first because it is the one carrying both anti-derived literals (§2b) and the crop-model
comparison, so it surfaces every hard problem at a third of the blast radius while the
other two contracts remain a working control. Each is a full unfreeze ceremony per its own
reference doc: advisor review → regenerate the manifest as the git-visible record →
document. ⚠ The two hard-coded `BIO_DT`-adjacent literals are **re-authored, not ported** —
each one is a decision about what it guards.

**Slice 9 — unit validation.** The §2e trap. Two candidate answers, to be priced when
taken, not now: (a) reimplement the dimensional check in Rust's loader, so the validated
path is the executed path; (b) keep pint in Python but make the Python check provably read
*what Rust actually loaded* rather than the file Rust happens to also read. ⚠ Whatever is
chosen, the acceptance criterion is that **removing the check turns something red** — a
lint whose deletion changes nothing is the defect this slice exists to avoid.

**Slice 10 — the 12 laws in Rust.** Add `proptest`; mirror conservation, non-negativity and
order-independence natively. Independent of every other slice; can be taken at any point.

**Slice 11 — the posture lands.** Rewrite the purity invariant (`git diff src/` empty is
A's rule and becomes wrong under B — the replacement governs *Python* as the consumer), add
the development-posture section `CLAUDE.md` has never had (§1), and close this doc out with
the normal three: index line, pointer row, record file in `docs/log/`, plus a memory file.
⚠ **This paragraph's own exemption in `post-roadmap-log.md` is deleted as part of this
slice** — an exemption written for a temporary state is a deletion someone must remember,
and the log records that forgetting it left three checks red for five commits.

## §6 Open questions — none blocking, all answerable when their slice is taken

1. **Does new *reference science* wait for the flip?** B makes Rust the place science is
   authored, but slices 6–8 are where that becomes true. Work taken before then is still
   Python-canonical. ⚠ Do not start a science item and a re-anchor slice in the same batch.
2. **What happens to the deferred mirrors already on the books?** Potato stage 2 (the Rust
   habitat mirror) is deferred. Under B a Python-side item with no Rust mirror is now a gap
   in the *reference*, not in the copy — so it changes category.
3. **Does the Godot consumer notice?** It consumes Rust and should not, but slice 4 moves
   the goldens it is indirectly pinned against. Check, do not assume.
