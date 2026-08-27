"""What is left of the outer config layer after slice S6.

This package used to own parameter loading and validation for the Python checker —
YAML plus pydantic schemas, with pint dimensional checks at the boundary. All of it
is gone: `rust/crates/config` is the one that counts (slice C1), and the exact
unit-string guard there replaced every live pint conversion, each of which had been
measured to be an identity.

⚠ What survives is **`paths`, and only because a surviving gate reads one constant
from it** — `regen_goldens_from_rust.py` wants `GOLDEN_DIR`. That tool is S6 build
item 2 and dies with its successor; this package dies in the same commit. It is not
a boundary layer any more, it is a single path constant wearing one.

Deliberately re-exports nothing. The old `__init__` pulled in `errors`, `loader` and
`units`, so leaving it untouched would have turned a package with two files into an
`ImportError` — which is exactly how it failed on the first run of this deletion.
"""
