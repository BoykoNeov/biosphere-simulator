"""Reference flip, slice 4: regenerate the regression goldens **from the Rust port**.

This is the committed, reviewable entry point for the act slice 4 names — *the goldens
are generated from Rust* (`docs/plans/post-roadmap-reference-flip.md` §3). Until now the
only blessed way to rewrite a golden was the per-scenario ``_regenerate()`` ``__main__``
in each ``tests/test_regression_*.py``, i.e. **Python**. Whatever the goldens' bytes
turn
out to be, "the reference moved" is not a true statement while the only committed
regeneration path runs the checker.

Run it::

    uv run python tests/crossport/regen_goldens_from_rust.py            # report only
    uv run python tests/crossport/regen_goldens_from_rust.py --write    # rewrite

**Reporting is the default and ``--write`` is explicit**, matching the discipline every
Python regeneration main already follows: rewriting a golden is a deliberate act whose
diff is reviewed, never a side effect of running something.

⚠ **What this tool can and cannot establish, stated plainly.** While the two ports emit
identical bytes, *no* byte-level check can tell which side produced a golden —
provenance
is not recoverable from the artifact. What is structural is the **path**: this module is
the Rust-side one, and it carries the mapping as data so the mapping itself can be gated
(`test_golden_provenance.py`).

---

## The census — 25 goldens, and Rust can produce 18 of them

⚠ **This corrects the plan's own arithmetic.** §2f reads *"24 `emit_*` programs against
25 golden files. One is missing or one program emits two; identifying which is slice 4's
first act."* Measured, that is wrong on three counts, and the gap is **7, not 1**:

* two programs each serve **two** goldens (``emit_perennial`` / ``emit_consumer``, by a
  ``long`` argument);
* four ``emit_*`` programs serve no golden in ``tests/regression/golden/`` at all
  (``emit_authored``, ``emit_perturbed_brownout``, ``emit_sealed_resume``,
  ``emit_composite`` — authoring fixtures and Godot cross-boundary references);
* **seven** goldens have no Rust program that emits their bytes — the two folded
  summaries and the five below.

The three groups are the classification slice 3's finding demands be done *before* a
ceremony rather than during one: for every artifact, ask where each side's copy came
from, because if one side's came from the other the comparison is a round trip.

## ⚠ The two goldens the two ports do not agree on, measured 2026-08-16

Sixteen of the eighteen are **byte-identical** between the ports on this
Windows/UCRT box
— Rust's stdout equals the committed file exactly. Two are not:

| golden | leaves differing | worst deviation |
|---|---|---|
| ``consumer_chamber_state.json`` | 7 of 205 | 4.6e-16 (~2 ULP) |
| ``perennial_long_horizon_state.json`` | 1 of 196 | 1.6e-16 (~1 ULP) |

Diagnosed as **accumulated last-bit noise, not an op-level port difference**: slice 1's
trajectory export walks 2440 steps of the perennial scenario with *zero* bitwise
divergence, and the ~1.3M-substep sealed station is byte-identical, so there is no
systematic disagreement to hunt. Both sit ~5 orders inside their Tier-2 band (1e-11), so
the plan's stop-rule ("a value beyond band is a port bug") did not fire.

⚠ **Their tiers.json evidence strings still claim ``max_rel_dev 0.0``.** That was true
when P7.4 measured it and is now stale — the prose half of a contract that no gate reads
(`docs/log/freeze-prose-half-is-ungated.md`). Recorded here; ``tiers.json`` is slice
5's.

⚠ **Profile is byte-neutral.** All 18 were run under both ``--release`` and debug and
every output matched across profiles, so the ``release`` flag below is a speed choice
only. Measured rather than assumed: if a profile ever *did* change bytes, "regenerate
from Rust" would be under-specified until the profile joined the reference definition.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GOLDEN_DIR = REPO_ROOT / "tests" / "regression" / "golden"
RUST_WORKSPACE_DIR = REPO_ROOT / "rust"
RUST_CRATES_DIR = RUST_WORKSPACE_DIR / "crates"


class Emitter:
    """A Rust program that emits one golden's exact bytes on stdout."""

    def __init__(
        self,
        crate: str,
        example: str,
        args: tuple[str, ...] = (),
        *,
        release: bool = False,
    ) -> None:
        self.crate = crate
        self.example = example
        self.args = args
        self.release = release

    @property
    def source(self) -> Path:
        return RUST_CRATES_DIR / self.crate / "examples" / f"{self.example}.rs"

    def command(self) -> list[str]:
        """⚠ **Always ``-p <crate> --example``, never a built binary path.**

        ``emit_crew`` exists in *both* ``simcore`` and ``domains`` — the pre-existing
        cargo output-filename collision slice 3 found — so
        ``target/*/examples/emit_crew.exe`` is whichever crate built last. And the
        ``simcore`` one **parses ``crew_state.json``'s own hex-floats and re-emits
        them**: it is a codec fixture, not a run. A regeneration that reached it would
        write the golden from itself. ``-p`` is what makes that unreachable.
        """
        cmd = ["cargo", "run", "-q"]
        if self.release:
            cmd.append("--release")
        cmd += ["-p", self.crate, "--example", self.example]
        if self.args:
            cmd += ["--", *self.args]
        return cmd

    def run(self) -> bytes:
        """Run the emitter and return its stdout as the golden's bytes."""
        proc = subprocess.run(
            self.command(), cwd=RUST_WORKSPACE_DIR, capture_output=True, text=True
        )
        if proc.returncode != 0:
            raise RuntimeError(
                f"{' '.join(self.command())} failed "
                f"(rc={proc.returncode}):\n{proc.stderr}"
            )
        return proc.stdout.encode("utf-8")


# --------------------------------------------------------------------------- #
# Group 1 — Rust emits the golden's exact artifact (18)                        #
# --------------------------------------------------------------------------- #
# The profile mirrors what `test_crossport.py` uses per case (debug, except the two
# sealed multi-year runs, which are `--release` for speed there and here). Bytes do not
# depend on it — see the module docstring's profile measurement.
RUST_EMITTERS: dict[str, Emitter] = {
    # biosphere (7 frozen) — `domains`
    "season_euler_state.json": Emitter("domains", "emit_season"),
    "sealed_chamber_state.json": Emitter("domains", "emit_sealed"),
    "perennial_chamber_state.json": Emitter("domains", "emit_perennial"),
    "perennial_long_horizon_state.json": Emitter(
        "domains", "emit_perennial", ("long",)
    ),
    "consumer_chamber_state.json": Emitter("domains", "emit_consumer"),
    "consumer_long_horizon_state.json": Emitter("domains", "emit_consumer", ("long",)),
    # standalone siblings — `domains`. ⚠ crew is `domains`, NOT `simcore`; see
    # `Emitter.command`.
    "crew_state.json": Emitter("domains", "emit_crew"),
    "eclss_state.json": Emitter("domains", "emit_eclss"),
    "power_state.json": Emitter("domains", "emit_power"),
    "power_self_discharge_state.json": Emitter("domains", "emit_power_self_discharge"),
    "thermal_state.json": Emitter("domains", "emit_thermal"),
    # assembled station — `station`
    "cabin_gas_state.json": Emitter("station", "emit_cabin_gas"),
    "water_recovery_state.json": Emitter("station", "emit_water_recovery"),
    "station_state.json": Emitter("station", "emit_station"),
    "greenhouse_state.json": Emitter("station", "emit_greenhouse"),
    "lighting_state.json": Emitter("station", "emit_lighting"),
    "harvest_state.json": Emitter("station", "emit_harvest"),
    "sealed_station_state.json": Emitter(
        "station", "emit_sealed_station", release=True
    ),
}

# ⚠ The two Group-1 goldens whose bytes the ports do NOT agree on today. Recorded as an
# explicit roster, not a tolerance: the point is that the set is *small and named*, so a
# third golden joining it — or one of these leaving — is a change somebody looks at. The
# measurement and the diagnosis are in the module docstring.
PORTS_DISAGREE = {
    "consumer_chamber_state.json": "7 of 205 leaves, worst 4.6e-16 (~2 ULP)",
    "perennial_long_horizon_state.json": "1 of 196 leaves, worst 1.6e-16 (~1 ULP)",
}

# --------------------------------------------------------------------------- #
# Group 2 — Rust emits a raw series; the golden is folded Python-side (2)      #
# --------------------------------------------------------------------------- #
# ⚠ Same shape as slice 3's `param_files` finding: there IS a Rust program, but what it
# emits is not the artifact. Regenerating these "from Rust" would still route the bytes
# through `drift.py`, so the fold — the part that decides what the summary *says* —
# stays on the checker's side. Giving them a Rust referent means porting the fold, which
# Phase-7 deliberately declined (advisor #3). Slice 6/7's classification inherits this.
PYTHON_FOLDED: dict[str, str] = {
    "drift_summary.json": (
        "Rust `emit_drift` streams the raw 15-yr leaf_c / consumer_carbon series; "
        "`drift.py` folds per-year summaries and the period class Python-side "
        "(Phase-7 P7.4, advisor #3). The fold is the artifact, and it has no Rust "
        "referent until it is ported."
    ),
    "sealed_energy_drift_summary.json": (
        "Rust `emit_sealed_energy_drift` streams the raw 15-yr thermal.node heat "
        "series; Python folds temp / year-peaks / is_stationary (Phase-7 P7.5, same "
        "discipline). Same shape as `drift_summary.json`."
    ),
}

# --------------------------------------------------------------------------- #
# Group 3 — no Rust referent at all (5)                                        #
# --------------------------------------------------------------------------- #
# None is in either manifest: all five are additive, non-frozen Python regression pins.
# ⚠ Each reason is a measured statement about the Rust tree, not a category judgement —
# `test_golden_provenance.py` re-derives what it can (that no example emits them) and
# this roster carries the part a test cannot: *why*.
NO_RUST_REFERENT: dict[str, str] = {
    "n_limited_state.json": (
        "`N_LIMITED_SCENARIO` has no equivalent in the Rust scenario roster — the "
        "dormant `f_N` limiter is pinned Python-side only. Building an emitter means "
        "first porting the scenario."
    ),
    "water_biting_state.json": (
        "`WATER_BITING_SCENARIO` likewise has no Rust equivalent, and the port says so "
        "in its own words: `biosphere/system.rs`'s drought-wiring test "
        "manufactures the condition by hand precisely because 'the Rust roster "
        "has no equivalent of' the Python declaration."
    ),
    "demo_euler_state.json": (
        "The Phase-0/1 two-domain demo skeleton (`build_demo`) is Python-only — no "
        "`build_demo` anywhere under `rust/crates/`. Its params are excluded from the "
        "frozen param list too; it is a teaching scaffold, not reference science."
    ),
    "demo_rk4_state.json": "The RK4 half of the same Python-only demo skeleton.",
    "state_snapshot.json": (
        "⚠ Not a simulation run at all: a hand-authored `sim_io` serialization "
        "fixture. And Rust **consumes** it — `simcore/src/snapshot.rs` reads this "
        "very file and "
        "reconstructs its bits — so it is an INPUT to the port. 'Regenerating "
        "it from Rust' would be the round trip in its purest form."
    ),
}


def committed_goldens() -> set[str]:
    """Every golden on disk. ⚠ Enumerated from the directory, never hand-listed."""
    return {p.name for p in GOLDEN_DIR.glob("*.json")}


def regenerate(*, write: bool) -> int:
    """Run every Group-1 emitter, report the diff against the committed bytes.

    Returns the number of goldens whose bytes would change (or did, with ``--write``).
    Each output is validated as a well-formed snapshot **before** it can be written: a
    golden that does not round-trip through `sim_io` must never reach the disk.
    """
    sys.path.insert(0, str(REPO_ROOT / "src"))
    from sim_io import snapshot  # noqa: PLC0415  (kept off the import path of the gate)

    changed = 0
    for name, emitter in RUST_EMITTERS.items():
        produced = emitter.run()
        snapshot.loads(produced.decode("utf-8"))  # refuse to write an invalid golden
        current = (GOLDEN_DIR / name).read_bytes()
        if produced == current:
            print(f"  identical  {name}")
            continue
        changed += 1
        note = PORTS_DISAGREE.get(name, "⚠ NOT in the recorded disagreement roster")
        print(f"  CHANGED    {name}  [{note}]")
        if write:
            (GOLDEN_DIR / name).write_bytes(produced)
    verb = "rewritten" if write else "would change"
    print(f"\n{len(RUST_EMITTERS)} emitters run; {changed} {verb}.")
    if not write and changed:
        print("Re-run with --write to rewrite them, then review the diff.")
    return changed


if __name__ == "__main__":
    raise SystemExit(0 if regenerate(write="--write" in sys.argv[1:]) >= 0 else 1)
