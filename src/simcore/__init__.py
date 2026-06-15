"""Pure simulation core — Python stdlib only, ZERO third-party dependencies.

Holds the deterministic stock-and-flow engine. Imports nothing beyond the
standard library so the eventual Rust port stays a near-mechanical translation.
Serialization, units, and param parsing live in the outer layers (``sim_io``,
``config``), never here.
"""
