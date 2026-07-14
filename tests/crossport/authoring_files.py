"""The file-level parse-parity harness for the Phase-9 Step-4b Rust `authoring` port.

Unlike Step 4a (rate *strings* → committed vector files), the shared cross-port
artifact here is the scenario **`.yaml` file itself** — both the Python interpreter
(`src/authoring`) and the Rust interpreter (`rust/crates/authoring`) read the *same*
committed file. So there is no generated vector file; the gates are:

* **byte-identity** — the Rust interpreter's run of an anchor reproduces the frozen
  golden byte-for-byte (crew / template@1.0 → `crew_state.json`), and every anchor's
  Rust run == the Python interpreter's run of the same file (Rust-parse ≡ Python-parse
  ≡ same trajectory). Those live in `test_crossport.py`.
* **structural graph-dump parity** — this module's :func:`render_graph_dump` renders an
  interpreted `BuiltScenario` to a canonical text; the Rust `dump_graph` example renders
  the *same* text from its interpreter, and `test_crossport.py` diffs them. This catches
  graph facts a final-state snapshot is blind to (flow priorities, present-but-inert
  flows, the bit-exact boundary-eval of an initial amount / forcing constant).

**The dump format is a parity contract** — keep it byte-identical to the Rust
`authoring::graph_dump::render_graph_dump` (the `sexpr`/`gen_engine_vectors` discipline,
one level up to whole files). Every float goes through `float.hex()` (the Rust side
uses the same hex-float codec), so boundary-eval parity is proven bit-exact.
"""

from __future__ import annotations

from authoring.interpreter import BuiltScenario, load_scenario

# The anchor scenarios (relative to tests/authoring/scenarios) + the template overrides
# each is exercised with. Kept here so `test_crossport.py` and any Rust-side driver
# iterate the same set. `overrides` is the CLI form `param=value` the Rust example uses.
#
# (scenario_file, overrides_dict, byte_identical_golden_or_None)
ANCHORS: tuple[tuple[str, dict[str, float], str | None], ...] = (
    # Composition anchor: the frozen Crew re-expressed — byte-identical to crew_state.
    ("crew_mission.yaml", {}, "crew_state.json"),
    # Template @ default (crew_count = 1.0): 1.0 * base == base → byte-identical.
    ("crew_habitat_template.yaml", {}, "crew_state.json"),
    # Template @ 4.0: the knob bites (every store ≈ 4×) → NOT byte-identical, but the
    # graph dump (initial amounts + forcing constants, hex-float) proves the Step-3
    # boundary-eval `param('crew_count') * const` matches bit-for-bit across ports.
    ("crew_habitat_template.yaml", {"crew_count": 4.0}, None),
    # Kinetics anchor: the frozen SelfDischarge re-expressed as a file `kinetics` flow
    # (built into a DeclarativeFlow by the Rust interpreter). No golden — its trajectory
    # parity rode Step 4a's traj vectors; here the file→graph path is what is validated.
    ("self_discharge_dsl.yaml", {}, None),
)


def load_anchor(
    scenarios_dir, scenario_file: str, overrides: dict[str, float]
) -> BuiltScenario:
    """Interpret one anchor file through the Python `authoring` interpreter."""
    return load_scenario(
        str(scenarios_dir / scenario_file), overrides=overrides or None
    )


def render_graph_dump(built: BuiltScenario) -> str:
    """Render an interpreted `BuiltScenario` to the canonical structural dump.

    MUST stay byte-identical to Rust `authoring::graph_dump::render_graph_dump`. LF
    newlines; a trailing newline. Floats via `float.hex()` (bit-exact vs the Rust
    hex-float codec).
    """
    lines: list[str] = []
    lines.append(f"scenario\t{built.name}")
    lines.append(
        f"config\t{built.integrator}\t{built.dt.hex()}\t{built.steps}\t{built.state.rng_seed}"
    )
    lines.append(f"has_authored_kinetics\t{1 if built.has_authored_kinetics else 0}")

    # Stocks — id-sorted.
    for sid in sorted(built.state.stocks):
        stock = built.state.stocks[sid]
        comp = sorted((q.value, coeff) for q, coeff in stock.composition.items())
        comp_str = ",".join(f"{q}={coeff.hex()}" for q, coeff in comp)
        lines.append(
            f"stock\t{sid}\t{stock.domain}\t{stock.quantity.value}\t{stock.kind.value}\t"
            f"{stock.unit}\t{stock.amount.hex()}\t{1 if stock.unclamped else 0}\t"
            f"{stock.extinction_threshold.hex()}\t{comp_str}"
        )

    # Flows — canonical id-sorted registry order, with priority.
    for flow in built.registry.flows:
        lines.append(f"flow\t{flow.id}\t{flow.priority}")

    # Forcings — name-sorted; the constant value is the schedule at (n=0, dt=0).
    for name in sorted(built.resolver.forcings):
        value = built.resolver.forcings[name](0, 0.0)
        lines.append(f"forcing\t{name}\t{value.hex()}")

    return "\n".join(lines) + "\n"
