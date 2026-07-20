# Post-roadmap: the Rust-primary pivot — a strategy/migration plan

**Status: DECIDED (2026-07-20) — Option A, with the Python laboratory treated as
*dormant, not retired*.** The user's ruling on the two hinge questions (§8):
**Q1 — not done with validated science yet** (they chose to do one more: validate the
day-neutral crop against `lintul3_springwheat`), so the PCSE laboratory keeps forward
value and Python is *not* retired (rules out C). **Q2 — Option A**: new content/product
work is Rust-first (no Python mirror owed); the frozen core + laboratory are preserved and
kept green (Rule 5); validated-science calibration stays Python-canonical (Rule 3). The
day-neutral-crop validation (`post-roadmap-day-neutral-crop.md`) is the first — and,
pending further user direction, the *last committed* — job that wakes the laboratory; it
is now COMPLETE. Rules 1–5 (§5) are the going-forward posture; they still need landing in
`CLAUDE.md` + memory (the one migration step not yet executed — see §6 step 1).

Triggered by the user's question, mid-way through the day-neutral-crop task: *"isn't it
time to start working only in Rust?"* The honest answer is **not a simple yes** — because
"work only in Rust" collides with the invariant the entire project is built on: **Python
is the canonical reference; Rust is a mechanical mirror with no reference authority**
(`CLAUDE.md` "Purity invariants"; `docs/native-port-reference.md`). This doc translates
the instinct — which is *right* about where the product lives — into a policy that keeps
that instinct's benefits without unmooring the validation edifice.

## The gap between what you asked and what I'd recommend — stated up front

You picked **"Full pivot — plan it"**, having *declined* "both ports, by work-type." So I
owe you the daylight honestly: my recommendation below (**A**) is the *lighter* of the
full-pivot readings, and I want you to see exactly why I lean there **and** why B/C remain
genuinely yours to take. **A is not "both ports."** "Both ports" keeps Python-first and
mirrors to Rust; **A reverses the default** — new work is authored **Rust-first and owes no
Python mirror at all**. That is a real pivot in daily practice, not a re-serve of the option
you passed over. The one thing A does *not* do is throw away the Python reference — and
whether that reference is worth keeping turns almost entirely on **one question you own**
(§8 Q2), which I've made the explicit hinge below. **I am not attached to A.** §3 is your
decision; my job here is to make the tradeoff legible, not to win it.

## The one-sentence recommendation (leaning, not fixed)

**Adopt "Rust-primary for all new content & product work; Python frozen-canonical for the
validated-science reference"** — reverse the *default* (new authored work lands Rust-first,
no Python mirror owed) while **keeping Python alive** as (1) the frozen reference-of-record,
(2) the science laboratory for the rare residual calibration, and (3) one half of the
cross-port mutual check. This is target state **A** (§3).

**But the whole case for A over C rests on one contingency** (§4, §8 Q2): Python's *only*
un-portable future value is **minting NEW oracle traces (running PCSE) for NEW validated
science**. The committed goldens and committed oracle traces do **not** need live Python to
*run* — the cross-port gate and the oracle tests compare against **committed** artifacts. So
**if you are done with new validated-science calibration** (scope-B/C), the Python
laboratory's forward value is **~zero**, and **C (retire Python) becomes far more viable
than a naive reading of §4 suggests.** Decide Q2 first; it swings A vs C.

---

## §1 Why this is not a simple flip — the constraint

The project's *value* is its discipline: a science-credible sim whose every frozen number
traces to primary literature, is conserved every step, and is reproduced bit-for-bit by an
independent second implementation. Four mechanisms enforce that, and **all four are
Python-anchored**:

1. **The freeze manifests** (biosphere / station / authoring) freeze the *Python* surface.
   Their gates equate the manifest against the **live Python tree**
   (`test_freeze_manifest.py` &c.). "Frozen" means "frozen in Python".
2. **The cross-port tolerance contract** (`docs/native-port-reference.md`,
   `tests/crossport/`) validates the **Rust** port against **Python-generated goldens**
   (Windows/UCRT hex-float). Rust is judged faithful *to Python*.
3. **The science laboratory is Python-only**: `lab/oracle_match.py` (the WOFOST/PCSE
   oracle comparison — and PCSE is **EUPL, Python, never portable**), `lab/convergence.py`,
   `lab/rk45.py`; plus pint unit-validation at the `config/` boundary and the hypothesis
   property tests (conservation, non-negativity, order-independence).
4. **The purity invariant** `git diff src/ empty` is the *exit criterion* of every port
   phase — it exists precisely to stop Rust from editing the Python reference "to suit
   itself".

You cannot "just move to Rust" without deciding what happens to each of these four. That
decision is what §3 enumerates.

## §2 Inventory — what is Python-anchored today, and can it move?

| Asset | Python-only? | Can it move to Rust? | Notes |
|---|---|---|---|
| The 5 engine crates (`simcore`+domains+station+authoring) | **already in Rust** | done | Rust runs every frozen scenario (`emit_*` examples) |
| Authored **DSL** content (`scenarios/*.yaml`) | no | **already runs Rust-native** | authoring interpreter + Godot `from_file` load it today |
| Authored **crop** content (via `build_plants`/`SeasonScenario`) | today Python-first | **yes** — Rust has the biosphere season builder | the day-neutral crop is the first case |
| The 25 goldens | Python-generated (UCRT) | regenerable from Rust | but see B/C: regenerating *inverts* the reference |
| The 3 freeze manifests | freeze the Python tree | re-anchorable to Rust | large; and it is what makes "frozen" mean something |
| Oracle laboratory (`lab/oracle_match`, PCSE) | **yes, structurally** | **no** — PCSE is EUPL/Python, never ported | the only oracle for validated science |
| pint unit validation (`config/`) | yes | reimplementable, not free | catches unit errors at the data boundary |
| hypothesis property tests | yes | `proptest` exists in Rust | reimplementation, not a move |
| Godot front-end | Rust/GDScript already | done | the actual product |

**The load-bearing rows are the oracle laboratory and the manifests.** Everything else is
either already in Rust or mechanically reimplementable. The oracle laboratory **cannot**
move (PCSE licence), and the manifests are what give "frozen" its teeth.

## §3 Three coherent target states

### A — Rust-primary for new work; Python frozen-canonical (RECOMMENDED)

- New **authored content** (crops, habitats, scenarios) and **product/gameplay** land
  **Rust-first**, and owe **no Python mirror** ("authored ≠ validated" already exempts
  them from freeze/oracle). New authored content gets **conservation + determinism checked
  natively in Rust**, not cross-port.
- The **validated core stays exactly as frozen** — the 3 manifests, the 20-golden
  cross-port contract, the oracle laboratory are untouched and keep their meaning.
- The **one exception**: residual **science calibration** (scope-B decomposer,
  `self_discharge`, any change a manifest names) stays **Python-canonical**, because it
  moves frozen goldens and needs the oracle. These are rare and user-initiated.
- **Daily-practice change**: reverse the mirror default. Today authored work is
  Python-first-then-mirror; under A it is Rust-first-and-stop.
- **Cost**: near-zero disruption; the freeze story is fully preserved. The only new build
  is a **Rust-native conservation+determinism harness for authored content** (the "authored
  ≠ validated" checks, one language over — small; the invariants already run inside the
  Rust integrator).

### B — Flip the reference: Rust becomes canonical, Python becomes the checker

- Regenerate all 25 goldens **from Rust**; rewrite the 3 manifest gates to equate against
  the **Rust** tree; invert the cross-port contract (Python now validates *against* Rust).
- Keep Python as an oracle/cross-check only.
- **Cost**: a large, high-risk re-anchoring of every freeze contract, for **no scientific
  gain** — the science does not change, only which language spells the reference. And it
  **weakens** the cross-port check (see §4).

### C — Retire Python entirely; Rust-only

- Delete the Python tree; Rust is the sole implementation.
- **Cost**: everything B costs, **plus** you lose the oracle laboratory (no PCSE → no way
  to diagnose a science regression), the pint boundary validation, and — decisively — the
  **independent second implementation** whose entire purpose is mutual validation.

## §4 Why A, and precisely what B and C throw away

**The cross-port check is not bureaucracy — it repeatedly catches real bugs.** Scope-B
increment 1's Rust mirror surfaced a genuine cross-port reset bug (year-2 crops skipped
vernalization arrest because the reset didn't re-zero the accumulator). The multi-rate
phase's "near-miss" (a driver with zero trajectory coverage) was caught the same way. That
value **exists only because two independent implementations must agree**. B and C both
collapse the two implementations toward one:

- **B** keeps both languages but makes them no longer *independent* — Rust is now the
  reference Python is fitted to, so a shared conceptual error (the kind cross-port is meant
  to catch) can no longer be caught by disagreement; it is now "correct by definition".
- **C** deletes the second implementation outright, and with it the only oracle for
  validated science (PCSE cannot be ported — it is EUPL and Python).

**Neither B nor C changes a single scientific result.** The sim runs the same numbers
either way. What they change is *which language is authoritative* — a property with no
downstream user value, bought at the price of the validation machinery. That is a bad
trade **for B**. A gets the thing the user actually wants (new work lives in Rust, next to
the product) without paying it.

### The C cost is narrower than it looks — and it is the whole hinge

The instinctive case for A over C is "you lose the oracle laboratory." **Sharpen that,
because most of it is already banked, not live.** The cross-port gate compares Rust against
**committed** Python-generated goldens; the oracle tests compare against **committed**
oracle traces (`commit oracle output + provenance only, never live PCSE`). **Neither needs
a running Python to execute** — a Rust-only repo could keep both as static comparison
fixtures. So the Python laboratory's *un-portable, forward-looking* value reduces to exactly
one thing:

> **Minting NEW oracle traces (running PCSE, which is EUPL/Python and never portable) for
> NEW validated science.**

Everything else — validating the *existing* frozen core, regenerating goldens for the
canonical implementation — is either already committed or doable by whichever port is
canonical.

**Therefore the A-vs-C decision collapses to one question:** *are you going to do more
validated-science calibration?* If **yes** (scope-B decomposer, `self_discharge`, future
reference science) → the PCSE laboratory has real forward value, keep Python → **A**. If
**no** (the science is done, you're building a game now) → that asset is **~zero**, and the
cost of C shrinks to a one-time re-anchor with no ongoing loss → **C is genuinely on the
table.** This is §8 Q2, and it should be answered *first*.

**The honest framing**: the roadmap is done, the science is frozen. ~90 % of remaining
work is **content + gameplay**, which has no reason to be Python-first and every reason to
be Rust/Godot-native. The remaining ~10 % is **residual science calibration** — the *only*
thing that keeps the Python laboratory earning its keep. A bets you'll want that 10 %; C
bets you won't. That bet is yours (§8 Q2), not mine to make for you.

## §5 The going-forward rules (target state A, concrete)

> **Rule 1 — Default new work to Rust.** New authored content (crops, habitats, scenarios)
> and all product/gameplay work is authored **Rust-first** and owes **no Python mirror**.

> **Rule 2 — Rust-native validation for authored content.** Authored content is not held
> to the cross-port golden contract (authored ≠ validated). It gets **conservation +
> determinism asserted natively in Rust** — the same two invariants the Rust integrator
> already re-asserts every step, now exercised by an authored-content test harness.

> **Rule 3 — Validated science stays Python-canonical.** Any change that moves a frozen
> golden or a manifest-named item (scope-B/C calibration, new *reference* science) is done
> **Python-first under the existing unfreeze discipline**, with the oracle laboratory, and
> **then** mirrored to Rust under the cross-port contract. `git diff src/` empty still
> governs *port* work; it does **not** forbid Python-canonical *science* work (it never
> did — that is the unfreeze discipline).

> **Rule 4 — The frozen core does not move.** The 3 manifests, the 20-golden cross-port
> contract, and the oracle laboratory are preserved as-is. The pivot is about *new* work,
> not a re-anchoring of the validated reference.

> **Rule 5 — A frozen reference must stay runnable to stay a reference.** Under A the Python
> tree keeps its CI green (the full suite + the cross-port job). A museum piece that no
> longer runs is not a valid reference/oracle — so "frozen" means *frozen and green*, not
> *abandoned*. This is the standing maintenance cost of A (and it is exactly the cost C
> stops paying).

**The dividing line, in one test:** *does this change a frozen golden or a manifest-named
item?* If **no** → Rust-first (Rules 1–2). If **yes** → Python-canonical (Rule 3).

## §6 Migration steps (small — A is mostly a policy change)

1. **Record the policy** — land Rules 1–4 in `CLAUDE.md` (a new "Development posture"
   section) and a memory note, so the default is durable across sessions. *(The single
   most important step; the rest is tooling.)*
2. **Build the Rust-native authored-content harness** — a `rust/crates/domains` (and/or
   `station`) test module that runs an authored `SeasonScenario`/scenario file and asserts
   `rationed == 0`, `events == ()`, per-quantity conservation, and bit-identical
   re-run determinism. This is the "authored ≠ validated" contract, one language over. The
   invariants already exist inside the integrator; this exercises them from a test.
3. **Add the authored-crop path to the Rust biosphere builder** — the Rust mirror of the
   `SeasonScenario` `vernalization` flag (+ optional crop-file selection), default-
   preserving so every frozen golden stays byte-identical on the Rust side too.
4. **Leave the freeze/cross-port/laboratory machinery untouched.** No golden moves, no
   manifest re-anchors, no Python deletion.

## §7 What this unblocks — the day-neutral crop as the first exercise

The paused day-neutral-crop task becomes the **first piece of work under Rule 1**: it is
authored content (a warm-habitat, photoperiod-only, vernalization-off wheat), so under A it
lands **Rust-first** with the Rust-native conservation+determinism harness (step 2) as its
acceptance, plus the two demonstrations the user chose (warm-arrest contrast + lamp
photoperiod control). No Python mirror is owed. If the user instead wants it in *both*
ports for now (belt-and-braces during the transition), that is a per-task choice, not a
reason to keep Python-first as the default.

## §8 Open questions for the user

1. **⟵ ANSWER THIS FIRST — it swings A vs C. Are you going to do more validated-science
   calibration?** (scope-B decomposer, `self_discharge`, any future *reference* science.)
   This is the single hinge (§4). **Yes** → the PCSE laboratory earns its keep → keep Python
   → lean **A**. **No / "the science is done, I'm building a game"** → the Python
   laboratory's forward value is ~zero (committed goldens+traces still run without it) → the
   cost of **C shrinks to a one-time re-anchor** → C is genuinely on the table. Note the
   flip side of "no": it leaves the scope-B decomposer / `self_discharge` questions **closed
   unresolved** (documented residuals, not fixed).
2. **Target state** — given Q1, adopt **A**, **B** (flip the reference to Rust), or **C**
   (retire Python)? A preserves the validation edifice; B re-anchors it to Rust for no
   scientific gain and weakens the cross-port check; C drops it entirely (viable only if
   Q1 is "no").
3. **The day-neutral crop** — resume it now under Rule 1 (Rust-first), or finish landing
   this policy first?
