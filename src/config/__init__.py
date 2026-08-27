"""What is left of the outer config layer after slice S6.

This package used to own parameter loading and validation for the Python checker —
YAML plus pydantic schemas, with pint dimensional checks at the boundary. All of it
is gone: `rust/crates/config` is the one that counts (slice C1), and the exact
unit-string guard there replaced every live pint conversion, each of which had been
measured to be an identity.

⚠ What survives is **`paths`, and only because one surviving reader wants one
constant from it** — `tests/oracle/runner.py` wants `WINTER_WHEAT_WEATHER`. It is not
a boundary layer any more, it is a single path constant wearing one.

⚠⚠ **This paragraph said the package dies with S6 build item 2, and that was wrong.**
It named `regen_goldens_from_rust.py` and `GOLDEN_DIR` as the last reader and reasoned
that retiring the tool retires the package. Build item 2 landed and the package did not
die: the PCSE oracle reaches in here too, and the carve-out outlives the checker. The
constant it wanted is a different one, so build item 3 deleted the other five and kept
the file. *A dated claim about who reads you is a claim about a roster — re-derive it
before acting on it.*

Deliberately re-exports nothing. The old `__init__` pulled in `errors`, `loader` and
`units`, so leaving it untouched would have turned a package with two files into an
`ImportError` — which is exactly how it failed on the first run of this deletion.
"""
