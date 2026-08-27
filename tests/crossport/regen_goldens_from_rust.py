"""Regenerate the regression goldens **from the Rust port** — the blessed path.

Built in slice 4 as the committed, reviewable entry point for the act the plan names —
*the goldens are generated from Rust* (`docs/plans/post-roadmap-reference-flip.md` §3).
**Slice 5 made it the only one.** Until then the per-scenario ``_regenerate()``
``__main__`` in each ``tests/test_regression_*.py`` could also rewrite these files, i.e.
**Python** could author the reference; those mains now route through
``golden_platform.write_python_golden``, which refuses the 19 files listed below.
"The reference moved" is not a true statement while a committed path runs the checker.

Run it::

    uv run python tests/crossport/regen_goldens_from_rust.py            # report only
    uv run python tests/crossport/regen_goldens_from_rust.py --write   # REFUSES (S6)

**Reporting is the default and ``--write`` is explicit**, matching the discipline every
Python regeneration main already follows: rewriting a golden is a deliberate act whose
diff is reviewed, never a side effect of running something. ⚠ Three of these are in a
**freeze manifest** (two biosphere, plus ``sealed_energy_drift_summary`` in the station
manifest since slice C5), so a ``--write`` that moves a frozen golden desynchronises the
manifest. Re-run the manifest writer as part of the unfreeze ceremony — from ``rust/``,
``cargo run --example dump_biosphere_inventory -- --write-manifest``; see
``docs/biosphere-reference.md``.

⚠ **The half of that warning that said "turns *nothing* red" was true until slice C7 and
is now false for every frozen golden** (measured, not assumed: a moved golden was fed to
the gate and it went red). ``tests/crossport/test_manifest_writer.py`` regenerates each
manifest and compares the committed file byte for byte, and every writer hashes the
goldens from disk — so a desynchronised hash is red rather than silent.

⚠ **This sentence has now been corrected twice and both times by the slice that
falsified it.** C7's biosphere half made it false for the biosphere and said *"it is
still true for the station"*; C7's **station** half made that clause false in turn, by
moving the last writer. The regeneration command above is the biosphere's — the
station's is ``cargo run --example dump_station_inventory -- --write-manifest``, and the
authoring contract's is ``dump_authoring_inventory``; all three take the same flag.

⚠ **What this tool can and cannot establish, stated plainly.** While the two ports emit
identical bytes, *no* byte-level check can tell which side produced a golden —
provenance
is not recoverable from the artifact. What is structural is the **path**: this module is
the Rust-side one, and it carries the mapping as data so the mapping itself can be gated
(`test_golden_provenance.py`).

---

## The census — 21 goldens, and Rust produces 19 of them

⚠ **This corrects the plan's own arithmetic.** §2f reads *"24 `emit_*` programs against
golden files. One is missing or one program emits two; identifying which is slice 4's
first act."* Measured, that is wrong on three counts, and the gap is **7, not 1**:

* two programs each serve **two** goldens (``emit_perennial`` / ``emit_consumer``, by a
  ``long`` argument);
* four ``emit_*`` programs serve no golden in ``rust/data/golden/`` at all
  (``emit_authored``, ``emit_perturbed_brownout``, ``emit_sealed_resume``,
  ``emit_composite`` — authoring fixtures and Godot cross-boundary references);
* **six** goldens have no Rust program that emits their bytes — one folded summary
  (``drift_summary``; ``sealed_energy_drift_summary`` moved to Group 1 in slice C5)
  and the five below.

The three groups are the classification slice 3's finding demands be done *before* a
ceremony rather than during one: for every artifact, ask where each side's copy came
from, because if one side's came from the other the comparison is a round trip.

## ⚠ The two goldens the two ports did not agree on — measured 2026-08-16, now written

Sixteen of them were already **byte-identical** between the ports on this
Windows/UCRT box when slice 4 measured them — Rust's stdout equalled the committed file
exactly. Two were not:

| golden | leaves differing | worst deviation |
|---|---|---|
| ``consumer_chamber_state.json`` | 7 of 205 | 4.6e-16 (~2 ULP) |
| ``perennial_long_horizon_state.json`` | 1 of 196 | 1.6e-16 (~1 ULP) |

Diagnosed as **accumulated last-bit noise, not an op-level port difference**: slice 1's
trajectory export walks 2440 steps of the perennial scenario with *zero* bitwise
divergence, and the ~1.3M-substep sealed station is byte-identical, so there is no
systematic disagreement to hunt. Both sat ~5 orders inside their Tier-2 band (1e-11), so
the plan's stop-rule ("a value beyond band is a port bug") did not fire.

**Slice 5 wrote both** (8 changed hex-float leaves across the two files, exactly as
predicted; no structural field moved). So all of them are now Rust's bytes, and the
divergence moved to the other side of the comparison: it is *Python* that differs from
the reference by those amounts. The roster followed its meaning to
``golden_platform.PYTHON_DIVERGES`` — see the note there on why it was renamed.

⚠ **Profile is byte-neutral.** All 18 were run under both ``--release`` and debug and
every output matched across profiles, so the ``release`` flag below is a speed choice
only. Measured rather than assumed: if a profile ever *did* change bytes, "regenerate
from Rust" would be under-specified until the profile joined the reference definition.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from config.paths import GOLDEN_DIR

REPO_ROOT = Path(__file__).resolve().parents[2]
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
# Group 1 — Rust emits the golden's exact artifact (19 since C5)               #
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
    # ⚠ Moved out of `PYTHON_FOLDED` by slice C5, and it is the *only* one of the
    # two that could move: `domains::biosphere::drift` now carries the fold kit, so
    # `emit_sealed_energy_drift` emits the summary rather than the raw node-heat
    # series. Measured byte-identical to the committed golden BEFORE the change, so
    # this re-anchors AUTHORSHIP with no value moving — the `golden_sha256` in the
    # station manifest does not change, only its `_authority` side. Plan §5h.
    "sealed_energy_drift_summary.json": Emitter(
        "station", "emit_sealed_energy_drift", release=True
    ),
}

# --------------------------------------------------------------------------- #
# Group 2 — Rust emits a raw series; the golden is folded Python-side (1)      #
# --------------------------------------------------------------------------- #
# ⚠ Same shape as slice 3's `param_files` finding: there IS a Rust program, but what it
# emits is not the artifact. Regenerating these "from Rust" would still route the bytes
# through `drift.py`, so the fold — the part that decides what the summary *says* —
# stays on the checker's side. Giving them a Rust referent means porting the fold, which
# Phase-7 deliberately declined (advisor #3). Slice 6/7's classification inherits this.
PYTHON_FOLDED: dict[str, str] = {
    "drift_summary.json": (
        "⚠ SLICE C5 ported the fold to Rust (`domains::biosphere::drift`) and "
        "this file still did NOT move. The reason is measured, not a deferral of "
        "convenience: folding the Rust series moves 4 of its 45 values (<=7 ULP, "
        "consumer years 3-4 — a 1-ULP divergence at step 4095 that the "
        "contracting attractor damps back to a bit-identical final state by year 15). "
        "Python would therefore need tolerance-gating, i.e. an entry on "
        "`golden_platform.PYTHON_DIVERGES` — and "
        "`test_every_diverging_scenario_keeps_a_byte_gated_sibling` goes RED, because "
        "`emit_drift` serves exactly one golden and so has no byte-gated sibling under "
        "that gate's emitter-program key. Widening that key from inside the slice that "
        "needs it widened is the co-adaptation this repo refuses, so the authorship "
        "move is DEFERRED to its own ceremony. See §5h of "
        "docs/plans/post-roadmap-reference-flip.md."
    ),
}


# --------------------------------------------------------------------------- #
# Group 3 — no Rust referent at all (1)                                        #
# --------------------------------------------------------------------------- #
# ⚠ FIVE UNTIL 2026-08-18. `n_limited`, `water_biting`, `demo_euler` and `demo_rk4`
# were retired outright by C6 of the reference flip — a golden with no Rust referent
# and no manifest entry is a checker asserting science the reference cannot run, which
# is exactly what the flip's posture says to delete rather than port. Their reasons are
# kept in `docs/log/reference-flip.md`, and every claim that HAD a successor moved to
# the reference before the deletion (see the plan's C6 section). The one that remains
# is not a simulation output at all, and it is not in either manifest.
# ⚠ Each reason is a measured statement about the Rust tree, not a category judgement —
# `test_golden_provenance.py` re-derives what it can (that no example emits them) and
# this roster carries the part a test cannot: *why*.
NO_RUST_REFERENT: dict[str, str] = {
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
    # ⚠⚠ THE WRITE PATH IS DISABLED, and this is a stated LOSS rather than a cleanup.
    #
    # Until slice S6 every candidate was validated before it could reach the disk --
    # `sim_io.snapshot.loads(produced)`, i.e. "a golden that does not round-trip must
    # never be written". S6 deleted `src/sim_io`, so that check is gone, and a
    # regeneration tool that writes UNVALIDATED bytes over a freeze contract's values is
    # worse than one that refuses. Reporting still works and is unaffected: it compares
    # the emitter's bytes against the committed ones and needs no validator.
    #
    # The successor is S6 build item 2 -- the blessed path moves to Rust, where the
    # emitters already live and the snapshot reader with them. Until it lands, a golden
    # is regenerated by running its emitter by hand and reviewing the diff.
    if write:
        raise SystemExit(
            "--write is disabled since S6 (2026-08-27): the snapshot round-trip check "
            "that gated every write died with `src/sim_io`. Report mode is unaffected. "
            "See S6 build item 2 in docs/plans/post-roadmap-reference-flip.md."
        )

    changed = 0
    for name, emitter in RUST_EMITTERS.items():
        produced = emitter.run()
        current = (GOLDEN_DIR / name).read_bytes()
        if produced == current:
            print(f"  identical  {name}")
            continue
        changed += 1
        # ⚠ Since slice 5 the golden IS Rust's output, so *any* line here is the
        # reference itself moving — never "the ports have drifted". Review the diff as a
        # science change, and re-run the freeze-manifest ceremony if it is frozen.
        print(f"  CHANGED    {name}  [the reference has moved — review the diff]")
        if write:
            (GOLDEN_DIR / name).write_bytes(produced)
    verb = "rewritten" if write else "would change"
    print(f"\n{len(RUST_EMITTERS)} emitters run; {changed} {verb}.")
    if not write and changed:
        print("Re-run with --write to rewrite them, then review the diff.")
    return changed


if __name__ == "__main__":
    raise SystemExit(0 if regenerate(write="--write" in sys.argv[1:]) >= 0 else 1)
