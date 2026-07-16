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
# (scenario_file, overrides_dict, byte_identical_golden_or_None, float_tier)
#
# `float_tier` classifies the RUN comparison, exactly as `tiers.json` does for the 20
# frozen goldens, and by the same rule: **classify by the ops the simulator EXECUTES**,
# never by observed equality on one machine.
#
#   * Tier 1 — the reachable per-step graph is transcendental-free (`+ - *`), so the run
#     is bit-exact across ports and `test_rust_authoring_run_matches_python` compares it
#     with `==`.
#   * Tier 2 — the graph has a transcendental, so it is EXCLUDED from that bit-exact
#     comparison. Tier 1 added the first such anchor (`thermal_node.yaml`:
#     `thermal.radiator_reject` evaluates `T**4`, a grep-confirmed transcendental site
#     in tiers.json, ported as `powf(4.0)`).
#
# **Why Tier-2 is excluded rather than band-compared** — a trap worth naming: the
# fresh-vs-fresh `rust_final == python_final` check would actually PASS for thermal_node
# on any single machine, because Rust and CPython there call the same libm. The measured
# Tier-2 bands exist for golden(UCRT)-vs-fresh(glibc) comparisons — a genuinely
# cross-libm measurement. Asserting that same-libm pass would silently label a `powf`
# flow Tier-1 and break the moment the CI matrix changed. And no NEW band is minted
# for it: a measured band is a frozen tolerance, and freezing one for a runtime-only
# authored artifact cuts against "authored != validated".
#
# What covers a Tier-2 anchor instead, in full: the cross-port GRAPH DUMP (bit-exact for
# it — the dump renders authored literals via float.hex() and never calls the radiator's
# evaluate(), so the transcendental cannot reach it), a Rust unit test that the authored
# RadiatorReject is wired to `params::thermal()` (the one sliver the dump cannot see,
# since params are not rendered), and the Python-side run gate in
# tests/test_authoring_frozen_flows.py. Its runtime ARITHMETIC parity is already frozen
# by thermal_state.json's measured band — re-proving it here would add nothing.
ANCHORS: tuple[tuple[str, dict[str, float], str | None, int], ...] = (
    # Composition anchor: the frozen Crew re-expressed — byte-identical to crew_state.
    ("crew_mission.yaml", {}, "crew_state.json", 1),
    # Template @ default (crew_count = 1.0): 1.0 * base == base → byte-identical.
    ("crew_habitat_template.yaml", {}, "crew_state.json", 1),
    # Template @ 4.0: the knob bites (every store ≈ 4×) → NOT byte-identical, but the
    # graph dump (initial amounts + forcing constants, hex-float) proves the Step-3
    # boundary-eval `param('crew_count') * const` matches bit-for-bit across ports.
    ("crew_habitat_template.yaml", {"crew_count": 4.0}, None, 1),
    # Kinetics anchor: the frozen SelfDischarge re-expressed as a file `kinetics` flow
    # (built into a DeclarativeFlow by the Rust interpreter). No golden — its trajectory
    # parity rode Step 4a's traj vectors; here the file→graph path is what is validated.
    ("self_discharge_dsl.yaml", {}, None, 1),
    # --- Step 6b: file composition (`includes` / bundles) ---
    # Single-bundle include: the crew species bundle IS the whole graph. At the bundle's
    # default `crew_count = 1.0` (`1.0 * base == base`) this reproduces the crew golden
    # byte-for-byte — the composition faithfulness anchor, one authoring level up.
    ("crew_station.yaml", {}, "crew_state.json", 1),
    # Same file @ 4.0: the override reaches the BUNDLE-declared `crew_count` through the
    # merge; the graph dump (hex-float amounts / forcing constants) proves the boundary-
    # eval matches across ports. Not byte-identical (every store ≈ 4×).
    ("crew_station.yaml", {"crew_count": 4.0}, None, 1),
    # Two-domain merge (the "it bit"): >1 file merged into one graph (crew + battery).
    # Transcendental-free (`+ - *`, `k·battery`) ⇒ Tier-1; `has_authored_kinetics = 1`
    # (the battery bundle's SelfDischarge) is rendered in the graph dump. No golden
    # (10 stocks, not the crew golden's 8) — run-match + graph-dump carry parity.
    ("station_composed.yaml", {}, None, 1),
    # Mixed include + inline (the only anchor with BOTH sources non-empty): the crew
    # bundle included + the battery domain declared inline. Exercises the includes-
    # first-then-inline merge path end-to-end (the resulting order is pinned by a
    # unit test on `apply_includes`'s Vec — the serialized outputs are id-sorted).
    ("crew_station_inline_battery.yaml", {}, None, 1),
    # --- Step 6c: multi-instance id-namespacing (`{bundle, prefix}` includes) ---
    # The SAME battery bundle included twice under distinct prefixes (bat_a / bat_b) —
    # namespacing rewrites every id + the kinetics rate's `stock(...)` ref, so the two
    # instances compose without colliding. Transcendental-free (`k·battery`) ⇒ Tier-1;
    # run-match + graph-dump carry parity (no golden — 4 namespaced stocks, not a frozen
    # graph). Proves the id-prefixing + rate re-render match across ports.
    ("two_batteries.yaml", {}, None, 1),
    # --- Post-roadmap Tier 1: the nine registered Power/Thermal/ECLSS flow types ---
    # The ECLSS anchor is the strongest of the three: all four ECLSS types wired as the
    # frozen build wires them, reproducing the FROZEN eclss_state.json byte-for-byte —
    # the crew_mission precedent, now for a domain with three conserved quantities and a
    # six-leg forced flow. Transcendental-free ⇒ Tier 1. (It proves the registry LOWERS
    # correctly. It makes no calibration claim: every eclss.yaml param is a TODO(cite)
    # placeholder — see docs/authoring-reference.md, "Frozen is not calibrated".)
    ("eclss_cabin.yaml", {}, "eclss_state.json", 1),
    # All three Power types on one bus. NO golden, and not for want of trying: the
    # frozen power goldens use a computed half-sine solar schedule (`math.sin`), which
    # an authored file cannot express (constant forcings only — a documented deferral).
    # Replacing the schedule with a constant removes Power's only transcendental, so
    # authored file is Tier-1 bit-exact where the frozen scenario it descends from is
    # Tier-2 — more strictly gated than its own reference.
    ("power_bus.yaml", {}, None, 1),
    # Both Thermal types. THE FIRST TIER-2 ANCHOR: `thermal.radiator_reject` evaluates
    # `T**4`, so this is excluded from the bit-exact run comparison and carried by the
    # graph dump (which is bit-exact for it) + a Rust unit test on its param wiring. See
    # the `float_tier` note above for why this is not band-compared instead.
    # Consciously accepted: `thermal.heat_input` rides in this Tier-2 file, so its RUN
    # parity is not bit-checked either — though it is a pure-multiply forced flow that
    # would qualify for Tier 1 alone. A third file to win that buys ~nothing: the graph
    # dump already proves its wiring on both ports, and it is the same `env.get(n)*dt`
    # shape as power.load_draw, which IS bit-checked in power_bus.
    ("thermal_node.yaml", {}, None, 2),
    # The three new param sets reached through authored `kinetics` — the OTHER surface a
    # registered loader opens, and the one with a cross-port hazard the frozen-`type`
    # anchors cannot see: Python DERIVES the param key names via asdict(); Rust pins
    # them in `kinetics_param_map`. A typo there resolves in Python and fails only in
    # Rust. Reads all nine key names. Deliberately nonsense physics — the property under
    # test is param resolution. The `self_discharge_dsl.yaml` precedent.
    ("param_sets_dsl.yaml", {}, None, 1),
)

# The float tiers, mirroring `compare.py`'s constants (not imported: this module is the
# anchor catalog and must stay importable without the comparison harness).
TIER_1_BIT_EXACT = 1
TIER_2_BAND = 2


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
