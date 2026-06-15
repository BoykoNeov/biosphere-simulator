# Biosphere / Station Simulator

A deterministic **stock-and-flow** simulation engine. Multi-domain from the
first commit; biosphere is the first domain. Python is the canonical reference
("laboratory"); a Rust core and Godot front-end come later.

- **Current work:** `docs/plans/phase-0-engine-skeleton.md`
- **Reuse / licensing:** `docs/reuse-and-licenses.md`
- **Engine invariants & conventions:** `CLAUDE.md`

## Layout

```
src/simcore/            # PURE engine — stdlib only, zero third-party deps
src/sim_io/             # outer: JSON checkpoints, hex-float goldens
src/config/             # outer: YAML loader + pydantic schemas + pint units
src/domains/biosphere/  # Phase-0 minimal demo domain
tests/                  # invariant + regression suite
```

## Development

Requires [uv](https://docs.astral.sh/uv/). Targets Python 3.13 (`>=3.12`).

```
uv sync                 # create .venv + install (incl. dev group), write uv.lock
uv run pytest           # tests (pytest + hypothesis)
uv run ruff check .     # lint
uv run ruff format .    # format
uv run pyright          # types
```

## License

Undecided / all rights reserved for now — see `docs/reuse-and-licenses.md`.
