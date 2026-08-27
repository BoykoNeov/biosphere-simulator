"""Where the reference's committed data lives, for the surviving oracle to read.

⚠ **Slice S1 of the reference flip moved the param YAML out of the Python packages.**
The 23 frozen param files (plus ``demo.yaml`` and the four potato overrides) now live
inside the Rust crates that compile them in — ``rust/crates/domains/params/<domain>/``
and ``rust/crates/station/params/`` — because the reference cannot stand on ground that
is scheduled for deletion (``docs/plans/post-roadmap-reference-flip.md``, FINDING 1).

So Python now reaches **into** the reference for its ground instead of owning it. That
direction is the point of the move, not a side effect: under the posture (Rust is
canonical, Python is the checker) a param file living in a Python package made the
checker the landlord of the reference's own data.

**One definition, not six.** Every loader used to spell its own
``Path(__file__).parent / "params"``; a repo-root climb copied into six modules is the
"a rule with two copies has one that is stale" shape this repo refuses. The climb lives
here once and dies here once.

⚠ **S6 build item 3 cut this file down to what is actually READ.** It carried six
constants — the param directories, the golden directory, the scenario fixtures — and the
last reader of all of them was `regen_goldens_from_rust.py`, retired by build item 2 in
favour of `station::regen`. Keeping them would leave five path constants that resolve,
that nothing loads, and that no gate would redden if the directories moved: the "green
check guarding a dead path" shape `CLAUDE.md` names for `config/units.py`, arriving at
the file that outlived it. One constant is read, by `tests/oracle/runner.py`, and one
remains.
"""

from __future__ import annotations

from pathlib import Path

#: The repository root. ``src/config/paths.py`` → ``config`` → ``src`` → root.
REPO_ROOT: Path = Path(__file__).resolve().parents[2]

#: The winter-wheat weather series the reference compiles in (`biosphere::weather`).
#:
#: ⚠ Not a test fixture any more, and S1 stopped it living like one. It used to sit in
#: ``tests/oracle/`` — the PCSE carve-out — while the reference reached five directories
#: up to embed it at compile time (slice C9 guarded that reach-out rather than removing
#: it). It now lives beside the crate that reads it, and the surviving oracle reaches
#: into the reference for it instead of the other way round. The other two series
#: (``potato_weather.json``, ``spring_wheat_weather.json``) stay in ``tests/oracle/``:
#: the discriminator is *the reference compiles this one in*, not *these three are a
#: set*.
WINTER_WHEAT_WEATHER: Path = (
    REPO_ROOT / "rust" / "crates" / "domains" / "data" / "winter_wheat_weather.json"
)
