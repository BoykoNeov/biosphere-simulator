"""Golden policy: who *authors* a golden, and who is gated against it how.

Two questions live here, and the reference flip (slice 5,
``docs/plans/post-roadmap-reference-flip.md``) turned them from one into two.

**1. Platform.** The committed hex-float goldens are byte-exact only *within a single
build on the platform that generated them* — Windows/UCRT here (determinism invariant #7
is same-build). A scenario whose evaluation graph touches a transcendental
(``exp`` / ``sqrt`` / ``sin`` / ``**``) lands last-ULP-different on a different libm
(Linux/glibc, macOS), so a byte-exact (or exact ``State``-equality) compare of a
*re-run*
against the Windows golden fails off the generation platform — physically meaningless
noise, not a regression. Cross-libm parity for those scenarios is gated instead by
``tests/crossport/`` with *measured tolerance bands*
(see ``docs/native-port-reference.md``).

So the transcendental regression goldens carry ``windows_golden_only``: their
byte-exactness is asserted only on the generation platform. The gate is by
transcendental
*classification*, not by the set that happens to diverge on one glibc build — a few
contracting/regulator-erased finals (thermal, greenhouse, lighting, the 5-yr consumer
chamber) coincidentally match on some libms, but relying on that would make CI brittle
across glibc versions. Pure-arithmetic goldens (crew / eclss / cabin_gas /
water_recovery / the basic-arithmetic demo) are bit-exact on every conformant platform
and stay ungated.

**2. Authorship — new in slice 5.** ⚠ **Nineteen of the twenty-one goldens are now
written by the Rust port, not by Python.** (Slice 5 wrote *eighteen of twenty-five*; C5
folded the station drift summary in Rust and C6 retired four Python-only goldens,
and neither correction reached this sentence until S1 re-measured from disk. ⚠ The
numbers in this docstring are **prose and nothing gates them** — the roster below is
what the suite checks. `test_golden_provenance.py` now carries a counted forcing
literal so the next drift is loud — ⚠ in a file S2/S6 retires, so that gate
needs a Rust-side successor, recorded in the plan FINDING 2.) That is the reference
flip: Rust is canonical, Python is the checker. Two consequences this module owns,
because both are policy rather than per-scenario detail:

* **Python must not author them** (:func:`write_python_golden`). Every
  ``tests/test_regression_*.py`` carries a ``_regenerate()`` ``__main__``; for a
  Rust-authored golden that path now *silently undoes the flip*, which is the same
  defect
  shape slice 4 refused to ship for ``emit_crew`` (a golden regenerated from itself).
  The blessed path is ``tests/crossport/regen_goldens_from_rust.py --write``.
* **Python's byte-exactness against them is an observation, not a contract**
  (:data:`PYTHON_DIVERGES`, :func:`assert_matches_golden`). Seventeen of the nineteen
  still
  match bit for bit and stay byte-gated — see the note on that roster for why keeping
  them tight matters. Two do not.

⚠ **Both are choke points, and the comparison one had to become universal.** Every
regression module routes its golden compare through :func:`assert_matches_golden`,
not only the two currently on the roster — including the seven Python-authored
goldens, where the helper is a plain byte compare. The first draft converted only the
two, and a negative control caught what that costs: the roster's *heal* direction (an
entry whose divergence went away is as red as one that appeared) is only live for a
golden whose module consults it, so a third entry landing on a module with a raw ``==``
would sit
there inert forever. A policy with two implementations has one that is stale; this file
holds the only one.

⚠ **The direction of the dependency is deliberate.** The golden *names* live here,
in the
base test suite; the Rust *commands* that produce them live in
``tests/crossport/regen_goldens_from_rust.py``, which imports this module. The reverse
would drag ``tests/crossport`` into the import path of every regression test. The two
rosters are asserted equal by ``test_golden_provenance.py``.
"""

from __future__ import annotations

import sys

import pytest

windows_golden_only = pytest.mark.skipif(
    sys.platform != "win32",
    reason=(
        "hex-float regression golden is byte-exact only on its Windows/UCRT "
        "generation platform; cross-libm parity is gated by tests/crossport "
        "(measured tolerance bands, docs/native-port-reference.md)"
    ),
)

# ⚠ **THE MARKER STAYS ON THE TWO TOLERANCE-CONVERTED GATES, AND THE REASON IS NOT THE
# ORIGINAL ONE.** Adjudicated in slice C3 (2026-08-18), which the reference-flip plan
# left this decision to. For the two goldens on `PYTHON_DIVERGES` below the compare is
# no longer byte-exact, so the rationale above ("byte-exactness is platform-bound") no
# longer reaches them — the obvious move is to unskip them on Linux and let the
# `DISAGREEMENT_CEILING` do the work. It was refused **on a number**, not on caution:
# the worst propagated +/-1-ULP transcendental sensitivity in this scenario group is
# 3.520e-15 (`tiers.json`; canopy `exp`, the perennial 15-yr, re-measured 2026-08-16)
# against a 1e-14 ceiling — under 3x of headroom for a *single* perturbed site, while
# glibc-vs-UCRT differs at all four sites `tiers.json` lists for these scenarios at
# once, and by more than one ULP at some of them. So the assumption "it will
# fit" is the one the evidence argues against, and writing a band nobody measured is
# the derived-not-measured move this contract exists to refuse.
#
# EXPIRY CONDITION — what retires this comment: one Linux run of the two
# `*_matches_the_reference` tests reporting their **max observed** deviation against the
# UCRT-authored goldens. With that number in hand, either drop the marker (if it sits
# under the ceiling) or restate the ceiling *with the measurement attached*. Nothing
# else unblocks it; re-reading this file cannot.

# --------------------------------------------------------------------------- #
# Authorship — the 18 goldens the Rust port writes                            #
# --------------------------------------------------------------------------- #
# ⚠ Names only. The `(crate, example)` commands that produce them are
# `regen_goldens_from_rust.RUST_EMITTERS`, and `test_golden_provenance.py` asserts the
# two rosters name exactly the same files — the duplication is the gate, not an
# oversight.
#
# The TWO NOT here stay Python-authored, and each for a stated reason (the census in
# `regen_goldens_from_rust.py`): `drift_summary` is folded Python-side from a raw Rust
# series (*the fold is the artifact*), and `state_snapshot` is a hand-authored `sim_io`
# fixture that Rust *reads*.
#
# ⚠ SIX UNTIL 2026-08-18, and the other four did not become Rust-authored — they
# were DELETED (C6 of the reference flip). `n_limited`, `water_biting`, `demo_euler`
# and `demo_rk4` had no Rust scenario at all and no manifest entry, so each was a
# golden the reference could not produce and no contract required. The roster went 25
# goldens to 21 with no value moving anywhere, and the reference's share of it went
# 19/25 to 19/21.
#
# ⚠ **Nineteen since slice C5, and only one of the two folded summaries could move.**
# `domains::biosphere::drift` now carries the fold kit, so `emit_sealed_energy_drift`
# emits its summary directly (measured byte-identical to the committed golden — an
# authorship re-anchoring with no value moving). `drift_summary` did NOT move, and the
# reason is a measurement, not a preference: folding the Rust series moves 4 of its 45
# values, which would put it on `PYTHON_DIVERGES` below and turn
# `test_every_diverging_scenario_keeps_a_byte_gated_sibling` red. See §5h of
# docs/plans/post-roadmap-reference-flip.md.
RUST_AUTHORED = frozenset(
    {
        # biosphere (7 frozen, minus the folded drift summary)
        "season_euler_state.json",
        "sealed_chamber_state.json",
        "perennial_chamber_state.json",
        "perennial_long_horizon_state.json",
        "consumer_chamber_state.json",
        "consumer_long_horizon_state.json",
        # standalone siblings
        "crew_state.json",
        "eclss_state.json",
        "power_state.json",
        "power_self_discharge_state.json",
        "thermal_state.json",
        # assembled station
        "cabin_gas_state.json",
        "water_recovery_state.json",
        "station_state.json",
        "greenhouse_state.json",
        "lighting_state.json",
        "harvest_state.json",
        "sealed_station_state.json",
        # the station's 15-yr energy stability signature — folded in Rust since C5
        "sealed_energy_drift_summary.json",
    }
)

# --------------------------------------------------------------------------- #
# The divergence roster — where Python is tolerance-gated instead of byte-gated #
# --------------------------------------------------------------------------- #
# ⚠ **This roster changed meaning when the contract inverted, and so did its name.**
# Slice 4 called it `PORTS_DISAGREE` and kept it beside the Rust census: it recorded the
# two goldens whose bytes *the two ports* did not agree on, and the Rust byte census
# consulted it. That name was symmetric because the contract was. It no longer is —
# after the flip the golden IS Rust's output, so "does Rust agree with the golden" has
# exactly one allowed answer (yes, no exemptions) and the open question is entirely
# about
# the other side. Same two files, same measured sizes, opposite consumer.
#
# ⚠ Seventeen of the nineteen stay **byte**-gated on the Python side, and that is not
# fussiness. A change to reduction order — canonical flow-id order on every reduction, a
# non-negotiable invariant — moves values by a ULP or two, i.e. *inside* any band this
# repo would call a tolerance. The byte compare is the only Python-side gate that sees
# that class of regression at all, so it is given up only where it is provably no longer
# available.
#
# ⚠ And it is given up for these two only at *this* horizon: `emit_consumer` and
# `emit_perennial` each serve two goldens, and in both cases the sibling horizon is
# still
# byte-gated (5-yr perennial, 15-yr consumer). Reduction-order coverage for both
# scenarios therefore survives — `test_golden_provenance.py` asserts that, so a third
# entry landing here cannot silently take the last byte gate off a scenario.
PYTHON_DIVERGES: dict[str, str] = {
    "consumer_chamber_state.json": "7 of 205 leaves, worst 4.6e-16 (~2 ULP)",
    "perennial_long_horizon_state.json": "1 of 196 leaves, worst 1.6e-16 (~1 ULP)",
}

# The ceiling a rostered golden's Python-vs-reference deviation must stay under. ⚠ A
# *last-bit noise* ceiling, not a science band: the measured worst is 4.6e-16, this is
# ~20x above it and **1000x tighter** than the Tier-2 band (1e-11) those same scenarios
# are gated at next door. A real defect blows through it long before `tiers.json` cares.
# One ceiling, shared with `test_golden_provenance.py`, so the two sides cannot drift.
DISAGREEMENT_CEILING = 1e-14
DISAGREEMENT_FLOOR = 1e-12


# ⚠ `write_python_golden` and `assert_matches_golden` were DELETED here by slice S6
# (2026-08-27), and the constants above them were not.
#
# Both functions policed one act: a Python regression main rewriting a committed golden.
# Slice 5 had already made `regen_goldens_from_rust.py` the only blessed path and
# routed those mains through the refusal below; S6 deleted the mains themselves, along
# with every `tests/test_regression_*.py`. A refusal with nothing left to refuse is not
# a guard, and `assert_matches_golden`'s tolerance arm imported `crossport.compare`,
# which went with them -- so keeping either would leave an import that cannot resolve.
#
# `RUST_AUTHORED`, `PYTHON_DIVERGES` and the two disagreement bounds SURVIVE because
# `test_golden_provenance.py` reads them as data: which goldens Rust authors, which
# ones Python used to disagree with in the last bits, and by how much. That roster is a
# record
# of a measurement, not a live comparison, and it is the reason those numbers are not
# deleted along with the code that once used them.
