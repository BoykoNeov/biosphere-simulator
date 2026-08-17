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

**2. Authorship — new in slice 5.** ⚠ **Eighteen of the twenty-five goldens are now
written by the Rust port, not by Python.** That is the reference flip: Rust is
canonical,
Python is the checker. Two consequences this module owns, because both are policy rather
than per-scenario detail:

* **Python must not author them** (:func:`write_python_golden`). Every
  ``tests/test_regression_*.py`` carries a ``_regenerate()`` ``__main__``; for a
  Rust-authored golden that path now *silently undoes the flip*, which is the same
  defect
  shape slice 4 refused to ship for ``emit_crew`` (a golden regenerated from itself).
  The blessed path is ``tests/crossport/regen_goldens_from_rust.py --write``.
* **Python's byte-exactness against them is an observation, not a contract**
  (:data:`PYTHON_DIVERGES`, :func:`assert_matches_golden`). Sixteen of the eighteen
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
from pathlib import Path

import pytest

windows_golden_only = pytest.mark.skipif(
    sys.platform != "win32",
    reason=(
        "hex-float regression golden is byte-exact only on its Windows/UCRT "
        "generation platform; cross-libm parity is gated by tests/crossport "
        "(measured tolerance bands, docs/native-port-reference.md)"
    ),
)

# --------------------------------------------------------------------------- #
# Authorship — the 18 goldens the Rust port writes                            #
# --------------------------------------------------------------------------- #
# ⚠ Names only. The `(crate, example)` commands that produce them are
# `regen_goldens_from_rust.RUST_EMITTERS`, and `test_golden_provenance.py` asserts the
# two rosters name exactly the same files — the duplication is the gate, not an
# oversight.
#
# The six NOT here stay Python-authored, and each for a stated reason (the census in
# `regen_goldens_from_rust.py`): `drift_summary` is folded Python-side from a raw Rust
# series (*the fold is the artifact*); `n_limited` / `water_biting` / `demo_euler` /
# `demo_rk4` have no Rust scenario at all; and `state_snapshot` is a hand-authored
# `sim_io` fixture that Rust *reads*.
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
# ⚠ Sixteen of the eighteen stay **byte**-gated on the Python side, and that is not
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


def write_python_golden(path: Path, data: bytes) -> None:
    """Write a **Python-authored** golden, refusing the 18 Rust authors.

    Every ``tests/test_regression_*.py`` has a ``_regenerate()`` ``__main__``. Before
    the
    flip that was the only blessed way to move a golden; after it, for a Rust-authored
    file it is the wrong path and running it silently reverts the reference to the
    checker. Making the refusal a *choke point* rather than a docstring note is
    deliberate: a future scenario's regeneration main will reach for this helper, and
    will be told which side owns its artifact without having to know the rule.
    """
    if path.name in RUST_AUTHORED:
        raise SystemExit(
            f"refusing to write {path.name} from Python: it is one of the "
            f"{len(RUST_AUTHORED)} goldens the Rust port authors (the reference flip, "
            "docs/plans/post-roadmap-reference-flip.md). Regenerate it with:\n"
            "    uv run python tests/crossport/regen_goldens_from_rust.py --write\n"
            "then review the diff and re-run the freeze-manifest ceremony if it is a "
            "frozen golden."
        )
    path.write_bytes(data)
    print(f"wrote {path}")


def assert_matches_golden(path: Path, produced: str) -> None:
    """Assert Python's ``produced`` serialization matches the committed golden.

    Byte-exact unless ``path`` is on :data:`PYTHON_DIVERGES`, in which case the
    comparison is Tier-0-exact (structure, discrete fields, stock-id set) plus a numeric
    deviation under :data:`DISAGREEMENT_CEILING`.

    ⚠ **Argument order is the inversion, spelled out.** The golden is the *reference*
    and
    Python's fresh output is the *candidate* — the roles slice 5 swapped. Before the
    flip
    the golden was Python's own output and the port was the candidate.
    """
    expected = path.read_bytes()
    if produced.encode("utf-8") == expected:
        assert path.name not in PYTHON_DIVERGES, (
            f"{path.name} is on PYTHON_DIVERGES as {PYTHON_DIVERGES[path.name]!r}, but "
            "Python now reproduces the reference byte for byte. Drop it from the "
            "roster "
            "and restore the byte gate — a divergence that healed must not be left "
            "standing as an exemption nobody re-measures."
        )
        return

    # ⚠ Which side is the reference depends on the golden, and so does the advice. The
    # message below is the *only* place a reader is told which way to look, and no test
    # can catch it being backwards — the assertion fires identically either way.
    if path.name in RUST_AUTHORED:
        whose = (
            "⚠ This golden is the Rust port's output — the reference. So this is a "
            "*Python* regression, or a reference move Python has not followed yet. It "
            "is NOT a golden to regenerate from Python; "
            "`tests/crossport/regen_goldens_from_rust.py` owns it. Do not widen the "
            "roster to silence this."
        )
    else:
        whose = (
            "⚠ This golden is **Python-authored** — the reference flip does not reach "
            "it (no Rust program produces this artifact; see the census in "
            "`tests/crossport/regen_goldens_from_rust.py`). So Python *is* the "
            "reference here, and if the move is intended the regeneration main in this "
            "module is the right path. The divergence roster does not apply to it: a "
            "Python-authored golden cannot disagree with Python."
        )
    assert path.name in PYTHON_DIVERGES, (
        f"{path.name}: Python's output no longer matches the committed golden, "
        f"and it is not on the divergence roster.\n{whose}"
    )

    # Rostered: pin *how much*. Imported lazily so the base regression suite does not
    # take `tests/crossport` onto its import path for the sixteen byte-exact cases.
    import json  # noqa: PLC0415

    sys.path.insert(0, str(Path(__file__).parent / "crossport"))
    import compare  # noqa: PLC0415

    result = compare.compare(
        json.loads(expected.decode("utf-8")),
        json.loads(produced),
        tier=compare.TIER_2_BAND,
        band=DISAGREEMENT_CEILING,
        floor=DISAGREEMENT_FLOOR,
    )
    assert result.ok, (
        f"{path.name} is a known last-bit divergence ({PYTHON_DIVERGES[path.name]}), "
        f"but Python has moved past {DISAGREEMENT_CEILING:g} from the Rust "
        f"reference:\n{result.report()}"
    )
    assert result.numeric_pairs, "expected numeric leaves to be compared"
