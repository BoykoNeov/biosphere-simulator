# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine for closed habitats: a biosphere
(crop, soil, decomposers, water, nitrogen) coupled to power, thermal, life-support and
crew domains, run headless or inside a Godot front-end from the *same* core. The subject
is closure of matter and energy cycles, asserted every step.

**Rust is the reference** (`rust/crates/`, since 2026-08-16). The Python checker that
preceded it is retired; what remains in Python is an opt-in validation oracle.

- **Engine invariants, layout, commands, contracts:** `CLAUDE.md` (a map, kept small)
- **What is next:** `docs/plans/post-roadmap-direction-2026-09.md`
- **What happened:** `docs/post-roadmap-log.md` (index) → `docs/log/` (one file per item)
- **The freeze contracts:** `docs/{biosphere,station,native-port,authoring}-reference.md`
- **Reuse / licensing:** `docs/reuse-and-licenses.md`

## Layout

```
rust/crates/simcore/       # PURE engine — zero third-party deps
rust/crates/config/        # file boundary: YAML/JSON readers, unit guard, provenance
rust/crates/domains/       # biosphere + the four siblings (power/thermal/eclss/crew)
rust/crates/station/       # the multi-domain assembly, drivers, goldens, regeneration
rust/crates/authoring/     # the scenario-authoring platform (grammar, VM, flow registry)
rust/crates/godot_bridge/  # the ONLY crate that depends on gdext
rust/crates/repo_gates/    # dev-only gates whose subject is this repository
rust/data/golden/          # the reference's own hex-float goldens
rust/data/scenarios/       # fixtures / cross-port anchors (never reference science)
scenarios/                 # authored content — runtime artefacts, never frozen
godot/                     # the front-end
tests/oracle/              # the PCSE/WOFOST oracle carve-out (hand-run, never a gate)
docs/                      # plans, the work log, the freeze contracts
```

## Development

```
cd rust
cargo test --workspace --no-fail-fast          # the reference's own gates
cargo clippy --all-targets -- -D warnings
cargo run --release -q -p station --example regen_goldens   # golden report (no write)
```

The Python remnant needs [uv](https://docs.astral.sh/uv/) and Python ≥ 3.12:

```
uv sync && uv run pytest        # 8 fixture tests + 4 oracle skips, under a second
uv run pytest -m oracle         # the PCSE oracle (needs the `oracle` group + network)
```

## License

**BNCL-1.0** (Boyko Non-Commercial License v1.0): free to use and modify for
non-commercial purposes; commercial use requires separate written permission. See
`LICENSE`, `NOTICE` and `docs/reuse-and-licenses.md`.
