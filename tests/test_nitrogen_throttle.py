"""Option (D), the N→C throttle — RE-PRICED 2026-08-10 and NOT BUILT.

Post-roadmap work; ``docs/plans/post-roadmap-nitrogen-cycle-form.md``, "THE (D)
RE-PRICE". (D) is the last open option of the N-cycle form gap and the only one with a
carbon effect. Its recorded price, written while (A) and (B) were still the live
options, was:

    "Real soils decompose high-C:N residue *slower* because microbes are N-starved.
     Adding that factor is what would give N a carbon effect — but the decomposer
     calibration measured that closure **requires the fast edge**. **Expect it to break
     closure in every sealed scenario.** This is a genuine scientific conflict, not an
     implementation risk, and it should be priced before being attempted."

Two things happened to that tree afterwards — (A)+(B) made litter C:N physical, and the
humification split changed the decomposer chain — so the price was re-derived rather
than inherited. **The verdict changed KIND: (D) is not refused on closure, it is not
buildable AS RECORDED**, and the closure conflict was never testable on the tree the
prediction was written about.

WHAT IS PINNED HERE, and why each one:

1. **The recorded prediction was about a mechanism that could not have fired.** Before
   (A), litter carried **0.004** C per N (the plan's own line 102: "1 C : 246 N"). An
   N→C throttle is by construction a factor that reduces decay when N is SCARCE
   relative to C, so at 250 N per carbon **every such factor, whatever its form, sits
   at 1**. This leg needs no curve and no threshold — it is the N-rich limit — which is
   why it is the leg the headline rests on.
2. **Neither decomposer primary this tree holds first-hand carries an N throttle**, and
   for RothC the reason is stronger than absence-of-a-term: it has no nitrogen at all.
3. ⚠ **THE STRUCTURAL FINDING: in CENTURY the phenomenon (D) is named for is the SOIL
   FRACTIONATION SEAM.** Parton 1987 does model "high-C:N residue decays slower" — as a
   **pool partition** keyed on **lignin:nitrogen**, not as a multiplier on a rate. So
   (D) and fractionation are ONE mechanism in the cited primary, and (D) inherits
   fractionation's measured blocker.
4. ⚠ **A SELF-CORRECTION THAT MATTERS.** My first reading of Parton was that it carried
   the OPPOSITE sign — because the text says immobilised mineral N "can stimulate the
   decomposition of low-N plant residue". Low-N residue IS high-C:N residue, so that
   sentence states (D)'s premise, not its negation. Refusing (D) on "the primary
   contradicts it" would have been a refusal on a false premise. What differs is the
   FORM and the KEYING QUANTITY, and that is the whole finding.

**Nothing was built and no invented throttle was run.** There is no cited form to run,
and fitting one whose only target is our own goldens is the consumer-chamber-2x /
DPM-RPM-labile / ruling-B shape this project has refused four times. The closure
question is therefore recorded as **UNMEASURED**, not as clean — the ``sealed_station``
precedent from the (C) diagnosis.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

from domains.biosphere import scenario as sc
from domains.biosphere.loader import load_nitrogen_params
from domains.biosphere.season import (
    LITTER_CARBON,
    LITTER_N,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, steps_for
from simcore.integrator import EulerIntegrator
from test_senescence_form import _weather

_M_C = 0.012011  # kg C / mol C — the committed convention (test_nitrogen_form.py:89)

_SOURCES = Path(__file__).resolve().parents[1] / "sources"

# The recorded pre-(A) litter C:N, from docs/plans/post-roadmap-nitrogen-cycle-form.md
# line 102. It is quoted rather than re-measured because the code that produced it was
# deleted by (A) — the honest status is "recorded", and it is used only for a
# form-independent limit argument, never as an input to a calculation.
_PRE_A_LITTER_CN = 0.004


def _pdf_text(name: str) -> str:
    """Extract a source PDF. Absence claims need extraction, never a skim (round 4)."""
    pdf = _SOURCES / name
    if not pdf.exists():
        pytest.skip(f"{name} not on the shelf (sources/ is gitignored)")
    out = subprocess.run(
        ["pdftotext", str(pdf), "-"], capture_output=True, text=True, errors="replace"
    )
    if out.returncode != 0:
        pytest.skip("pdftotext unavailable")
    return out.stdout


def _flat(text: str) -> str:
    """Collapse every run of whitespace to one space, for PHRASE checks.

    ⚠ **Why this exists, because it looks like a convenience and is not.** A quoted
    phrase from a paper is *hard-wrapped* by ``pdftotext``, at a column that depends on
    the poppler version — so ``"stimulate the decomposition of low-N plant residue" in
    text`` is really an assertion about **where the extractor broke the line**, not
    about what the paper says. Three assertions here were written against one layout
    and silently falsified by a poppler upgrade (2026-08-11, poppler 25.09.1), and
    **nothing anywhere could notice**: ``sources/`` is gitignored, so on CI every test
    here ``skip``s. Green-by-skip on CI, rot-only-locally — the mirror image of
    ``memory/ci-python-job-red-on-linux.md``.

    Flattening preserves the assertion's *subject* exactly (the phrase is in the primary
    source, and it is). It is not available to the two **absence** claims, which need
    line structure to tell a bibliography entry from a body sentence.
    """
    return " ".join(text.split())


def _drive(scenario, years: int, perennial: bool):
    w = _weather(years)
    state, registry = build_season(scenario)
    integ = EulerIntegrator(registry)
    resolver = weather_resolver(w, scenario)
    if perennial:
        return run_perennial(
            integ,
            state,
            scenario,
            resolver,
            BIO_DT,
            steps_for(len(w)),
            year=steps_for(len(_weather())),
        )
    return run_season(integ, state, resolver, BIO_DT, steps_for(len(w)))


def _pool_cn(state) -> float | None:
    c = state.stocks[LITTER_CARBON].amount * _M_C  # kg C
    n = state.stocks[LITTER_N].amount  # kg N
    return c / n if n > 0 else None


# --- 1. the direction check — the leg that needs no threshold -------------------------


def test_the_recorded_price_described_a_mechanism_that_could_not_have_fired() -> None:
    """⚠ THE HEADLINE, and it rests entirely on the N-RICH limit (no curve needed).

    "Expect it to break closure in every sealed scenario" was written against a tree
    whose litter sat at C:N **0.004** — roughly 250 nitrogen atoms per carbon. An N→C
    throttle reduces decay when N is scarce *relative to C*; at 250 N per C, N is not
    scarce under any definition, so every such factor evaluates to 1 and the mechanism
    is INERT. The prediction was not wrong so much as untestable on its own subject.

    Post-(A)+(B) the shed ratio is a **parameter identity** — ``M_C /
    n_residual_per_mol_c`` — and lands on exactly 90, both constants cited (Van Hecke
    2020's residual N, and the carbon fraction). So the quantity a throttle would read
    went from "the unconstrained ratio of two unrelated rate constants" to "the
    composition of the material that fell in", which is what (A) and (B) were FOR.
    """
    shed_cn = _M_C / load_nitrogen_params().n_residual_per_mol_c
    assert shed_cn == pytest.approx(90.0, abs=1e-6), shed_cn

    # the form-independent argument, stated as the ratio it depends on
    assert _PRE_A_LITTER_CN < 1.0, "pre-(A) litter held more N than C, by ~250x"
    assert pytest.approx(250.0, rel=1e-9) == 1.0 / _PRE_A_LITTER_CN
    assert shed_cn / _PRE_A_LITTER_CN == pytest.approx(22500.0, rel=1e-6)


def test_the_litter_pool_a_throttle_would_read_is_TWO_REGIMES_not_one_number() -> None:
    """⚠ AND IT WOULD NOT BITE UNIFORMLY — the shedding-fed / reset-driven split again.

    Third appearance of the distinction that correction 2 and (B)-finding 5 both had to
    learn the hard way, so it is measured here rather than assumed: each scenario is
    driven the way its OWN golden drives it, because the annual reset is what makes a
    perennial chamber perennial.

    * shedding-fed (``run_season``): the pool tracks the shed ratio — 92-103.
    * reset-driven (``run_perennial``): at the annual dump the pool is set by the DYING
      PLANT's composition, and sits near **10**, i.e. N-RICH, rising through the season.

    A throttle keyed on litter C:N would therefore be near-saturated right after each
    reset in exactly the two scenarios where closure is tightest. That is a fact about
    where (D) would act, and it is the reason "expect it to break closure in every
    sealed scenario" could not have been read off a single number even post-(A).
    """
    seen = {}
    for label, scen, years, per in (
        ("sealed_chamber", sc.SEALED_CHAMBER_SCENARIO, 3, False),
        ("water_biting", sc.WATER_BITING_SCENARIO, 1, False),
        ("perennial", sc.PERENNIAL_CHAMBER_SCENARIO, 5, True),
        ("consumer", sc.CONSUMER_CHAMBER_SCENARIO, 5, True),
    ):
        states, rationed, _ = _drive(scen, years, per)
        assert rationed == 0, (label, rationed)
        ns = [s.stocks[LITTER_N].amount for s in states]
        peak = max(range(len(ns)), key=lambda i: ns[i])
        seen[label] = _pool_cn(states[peak])

    # shedding-fed: within ~15 % of the shed ratio, and ABOVE it (the N-free seed)
    # ⚠ 2026-08-12 (stem reserves): 103.303768 -> 104.972745 and 99.281016 ->
    # 100.678502.
    # Both still inside the band, but ``sealed_chamber`` now sits 0.03 under its
    # ceiling.
    # Left AS IS rather than widened — the band states a claim ("within ~15 % of the
    # shed
    # ratio") and re-cutting it to buy headroom would be fitting the bound to the tree.
    # Flagged instead: the next carbon-side change here is expected to break this, and
    # when it does the question is whether the CLAIM still holds, not where to move 105.
    # ⚠⚠ **THE PREDICTED BREAK ARRIVED, 2026-08-14, and the comment above says what to
    # do with it: ask whether the CLAIM holds, not where to move 105.** It broke on the
    # step unfreeze, which is not the "next carbon-side change" the prediction named —
    # so the flag was right about the pin and wrong about what would trip it.
    #
    # **The claim does not survive as written, and the reason is that its two halves
    # never agreed.** The prose says "within ~15 % of the shed ratio"; the numeric
    # ceiling 105 is 16.7 % of a shed ratio that is *exactly* 90 by parameter identity.
    # Both readings were ALREADY outside ~15 % before this change (104.97 = 16.6 %), so
    # 105 was never the prose's bound — it was a round number near the measurement.
    # Widening it to 106 would fit the bound to the tree, which the comment above
    # refuses in its own words.
    #
    # So the threshold is dropped and replaced by the two things that are actually
    # claimed, which together say strictly more than any band: the STRUCTURAL half (the
    # pool sits above the shed ratio, because the seed is N-free) is asserted against
    # ``shed_cn`` itself rather than a transcribed 90, and the LEVEL is pinned exactly,
    # so any future move is visible instead of being absorbed by slack.
    shed_cn = _M_C / load_nitrogen_params().n_residual_per_mol_c
    assert shed_cn == 90.0  # the parameter identity this whole band is about
    for label in ("sealed_chamber", "water_biting"):
        assert seen[label] > shed_cn, (label, seen[label], shed_cn)
    # 104.972745 -> this (+0.18 %), and 100.678502 -> this (+0.30 %). As multiples of
    # the shed ratio: 1.1685x and 1.1220x, i.e. 16.9 % and 12.2 % above it.
    assert seen["sealed_chamber"] == pytest.approx(105.163, rel=1e-4)
    assert seen["water_biting"] == pytest.approx(100.985, rel=1e-4)
    # ⚠ The load-bearing claim of this test is not the tightness of either number — it
    # is that these are a DIFFERENT REGIME from the reset-driven pair below, which sits
    # near 13. That separation is asserted at the end of this function.
    # reset-driven: an order of magnitude lower — the dying plant, not the shed ratio
    # ⚠ RE-MEASURED 2026-08-12 (stem reserves): the band was 9.0-12.0 and the values
    # were
    # 10.889961 / 9.787107; they are now 12.799056 / 11.687868. The MECHANISM is the
    # build working as designed: ``annual_reset`` dumps the standing reserve into
    # litter,
    # and reserve starch carries NO nitrogen, so each reset now seeds the pool with
    # carbon the old tree did not have. A C:N ratio rising when N-free carbon is added
    # is
    # the arithmetic, not a surprise.
    #
    # The band is re-cut around the new pair rather than merely stretched, and it keeps
    # its width (3.0) so it is no looser than the one it replaces.
    for label in ("perennial", "consumer"):
        assert 11.0 < seen[label] < 14.0, (label, seen[label])
    # ⚠ THE CLAIM this test is named for is the SEPARATION, and it is re-measured, not
    # inherited: a throttle keyed on litter C:N would still be near-saturated after each
    # reset.
    #
    # ⚠⚠ **THE `> 8.0` CUT IS DROPPED, NOT NUDGED (2026-08-14).** The ratio went
    # 8.20x -> 7.96x at `dt = ¼` — the reset-driven pair rose while the shedding-fed one
    # barely moved — so a round 8.0 chosen when the measurement was 8.20 now fails by
    # 0.5 %. Lowering it to 7.9 would be fitting the bound to the tree, twice in one
    # function, which the note further up refuses in its own words.
    #
    # Replaced by the structural statement the "two regimes" name is actually about, and
    # it needs no cut at all: **the two regimes fall on OPPOSITE SIDES of the shed
    # ratio.** A shedding-fed pool cannot go below it (the seed carries no N, so shed
    # material is the only N source and dilution only raises C:N); a reset-driven pool
    # is set by the dying plant's own composition and sits far below it. That is a
    # sign test against a parameter identity, not a threshold — it cannot drift, and it
    # is what "a throttle would be near-saturated after each reset" depends on.
    for label in ("sealed_chamber", "water_biting"):
        assert seen[label] > shed_cn, (label, seen[label])
    for label in ("perennial", "consumer"):
        assert seen[label] < shed_cn / 4.0, (label, seen[label])
    # ...and the ratio is pinned as the MEASUREMENT it is, so a change is visible.
    # 8.20x -> this.
    assert seen["sealed_chamber"] / seen["perennial"] == pytest.approx(7.96, rel=1e-3)


# --- 2. the retrieval result — no cited form exists on this shelf ---------------------


def test_RothC_carries_no_nitrogen_at_all() -> None:
    """The stronger of the two absence results, and it is stronger than "no N term".

    RothC is first-hand in this repo (the decomposer calibration and the fractionation
    diagnosis both read it). Extracting the whole guide, "nitrogen" occurs **twice, both
    in the bibliography** — it is a carbon-only model. So it cannot license an N→C
    throttle, and it cannot be cited against one either.

    Established by extraction rather than by reading, because an ABSENCE claim is the
    one kind a skim structurally cannot make (round 4's rule).
    """
    text = _pdf_text("RothC_guide_WIN.pdf")
    assert len(text) > 20_000, "extraction failed — the absence claim would be void"
    lines = text.splitlines()
    hits = [i for i, ln in enumerate(lines) if "nitrogen" in ln.lower()]
    # THE ABSENCE CLAIM: exactly two occurrences in the whole guide. Deliberately an
    # exact count and deliberately line-based — `_flat` is unavailable here, because
    # telling a bibliography entry from a body sentence needs the line structure.
    assert len(hits) == 2, [lines[i] for i in hits]
    # both are reference-list entries: author, year, title, journal. ⚠ Read with ONE
    # line of lookahead: pdftotext wraps a reference entry across two physical lines, so
    # the entry's own tokens ("PhD thesis", "transactions of the Royal Society") land on
    # the continuation line. One line is enough for both entries in this guide — if a
    # future extractor wraps one to three, this fails rather than being widened until it
    # passes, which is the point.
    for i in hits:
        entry = _flat(" ".join(lines[i : i + 2]))
        assert "(" in entry and ")" in entry, entry
        assert any(tok in entry for tok in ("thesis", "transactions")), entry


def test_CENTURY_has_NO_nitrogen_limitation_on_the_DECAY_RATE() -> None:
    """Parton 1987 is already first-hand in this tree — the humification split's source.

    Its decay-rate controls are **lignin** and **texture**, not nitrogen:

        K1 = Ks · exp(-3.0 × Ls)          [eq 3]

    where ``Ls`` is the lignin fraction of structural material. Nitrogen enters CENTURY
    in two places, neither of them a rate throttle: the metabolic/structural PARTITION
    (eq 2, below) and the mineralisation/immobilisation bookkeeping that follows the
    carbon flows — which is precisely what option (B) already built.
    """
    text = _flat(_pdf_text("parton1987.pdf"))
    assert len(text) > 20_000, "extraction failed — the absence claim would be void"
    # the decay rate is keyed on lignin, first-hand
    assert "exp(-3.0 X Ls)" in text or "exp(-3.0" in text
    assert "fraction of structural material that is lignin" in text
    # the N flows are stoichiometric followers of the C flows — option (B)'s law
    assert "stoichiometrically related" in text
    assert "fixed N/C ratio of the state variables receiving the" in text


def test_the_tree_has_no_lignin_state_and_the_CUE_BUILD_ALREADY_SAID_SO() -> None:
    """⚠ Measured as a grep, not asserted — the canopy-regulator discipline.

    And the point is not that lignin is absent; it is that **the humification split had
    already recorded this exact obstacle one build earlier**, for its own reason. Half
    of (D)'s blocker was written down before (D) was re-priced, in a param file's own
    comments, and nothing routed attention to it.
    """
    root = Path(__file__).resolve().parents[1]
    hits = subprocess.run(
        ["git", "grep", "-il", "lignin", "--", "src/"],
        cwd=root,
        capture_output=True,
        text=True,
    ).stdout.split()
    # lignin appears ONLY as prose explaining what cannot be expressed
    assert hits, "expected the CUE build's own note"
    yaml = (root / "src/domains/biosphere/params/humification.yaml").read_text(
        encoding="utf-8"
    )
    assert "no lignin fraction" in yaml
    assert "is not" in yaml and "expressible" in yaml
    # and there is no lignin STOCK or PARAM anywhere
    state, _registry = build_season(sc.SEALED_CHAMBER_SCENARIO)
    assert not [sid for sid in state.stocks if "lignin" in str(sid).lower()]


# --- 3. the structural verdict --------------------------------------------------------


def test_in_the_cited_primary_D_and_SOIL_FRACTIONATION_are_ONE_MECHANISM() -> None:
    """⚠⚠ THE FINDING, and it is why (D) is "not buildable as recorded".

    CENTURY *does* model "high-C:N residue decays slower". It does it by splitting fresh
    residue into a fast (metabolic) and a slow (structural) pool:

        FM = 0.85 - 0.018 × (L/N)         [eq 2]

    — the metabolic fraction falls as the lignin:nitrogen ratio rises, cited to Melillo
    et al. 1984. That is **the input half of the soil-fractionation seam**, which this
    project measured and refused twice; its re-opening found both principled sizings
    failing on ``perennial`` (constant-flux rations at step 807, constant-inventory
    hard-errors in ``annual_reset``).

    So (D) as recorded — a multiplier on the single ``decomposition_rate`` — is a form
    the cited primary does not use, and the form it DOES use is blocked by a measurement
    this repo already has. Two independent reasons, and neither is "we ran it and
    closure broke".

    ⚠ The keying quantity differs too: **L/N, not C:N**. (D) as recorded would read a
    quantity CENTURY does not key on, using a state the tree does not have.
    """
    text = _flat(_pdf_text("parton1987.pdf"))
    # the length floor the other two extraction tests carry, added for their reason:
    # `_pdf_text` skips only on a nonzero exit, so a pdftotext that "succeeds" with a
    # short or empty read would turn every assertion below into a vacuous pass.
    assert len(text) > 20_000, "extraction failed — the quotations below would be void"
    assert "0.85 - 0.018" in text, "eq [2], the metabolic/structural partition"
    assert "L/N ratio gets larger" in text
    assert "Melillo et al. (1984)" in text
    # ⚠ THE SELF-CORRECTION: the primary states (D)'s premise, it does not contradict it
    assert "stimulate the decomposition of low-N plant residue" in text
    # the citable threshold the primary does give — a DIFFERENT quantity (immobilisation
    # onset), recorded with that caveat rather than used as a decay threshold
    assert "does not occur if the C/N ratio is < 10" in text


def test_no_invented_throttle_was_run_and_that_is_deliberate() -> None:
    """The unmeasured leg, recorded as unmeasured — the ``sealed_station`` precedent.

    The obvious next step is "write a plausible f(C:N) and see whether closure breaks".
    It is refused: there is no cited form for it on this shelf, so any curve would be
    chosen by its effect on our own goldens, which is the shape refused at the
    consumer-chamber 2x, the DPM/RPM labile re-read, ruling B, and the fractionation
    seed sweep.

    What that costs, stated plainly: the recorded closure conflict is neither confirmed
    nor discharged. It is **untestable as recorded** (finding 1) and **unmeasured on the
    current tree** (here). Creating that debt silently is what this pin exists to
    prevent.

    ⚠ Retrieval is EXHAUSTED FOR THIS SHELF, dated 2026-08-10 — not "the science does
    not exist". The canopy regulator expired that inference in one day, and any "every
    source says X" is a statement about your own shelf.
    """
    shelf = sorted(p.name for p in _SOURCES.glob("*.pdf")) if _SOURCES.exists() else []
    if not shelf:
        pytest.skip("sources/ is gitignored and absent")
    # the three candidates that were opened, so a future round knows where not to look
    for name in ("parton1987.pdf", "RothC_guide_WIN.pdf", "manzoni2012.pdf"):
        assert name in shelf, name
