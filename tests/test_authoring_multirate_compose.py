"""The multi-rate partition x `{bundle, prefix}` namespacing — the compose gap.

**The claim this file exists to anchor.** Both ports assert, in a comment on the line
that copies the field, that ``rate_class`` is a *property* of the flow rather than an id
*reference*, so ``compose``'s prefix rewrite carries it verbatim and needs no case for
it (``authoring/compose.py``, ``rust/crates/authoring/src/compose.rs``). Multi-rate Step
6b named that claim **unanchored** and did not fix it: the cross-port multi-rate anchor
it added (``eclss_multirate_cabin.yaml``) declares no ``includes``, so no test in either
port had ever put a rate class and a prefix in the same file.

**Why an unanchored "carried verbatim" is worth a test at all** — and the asymmetry
between the ports, which was **measured, not assumed**. The failure modes are: a prefix
rewrite that gets ``rate_class`` wrong, and a partition computed *before*
``apply_includes`` (bundle-contributed slow flows are never seen; this is the exact
hazard ``interpreter._slow_flow_ids``'s docstring argues the partition out of being a
pydantic validator for). Both fail the same way — **the slow set comes back empty**,
which at ``n_sub >= 2`` is a legal, quiet build: no error, no rationing, no event, just
a scenario silently integrated on one cadence when the author asked for two.

But *dropping* the field outright is not reachable on either port, and saying otherwise
would overstate this file. Python's ``_namespace_flow`` is a ``model_copy(update=...)``,
so every field it does not name is carried **structurally**; Rust's is a full struct
literal, so an omitted field does not compile. What *is* reachable is a **wrong** value
— hardcoding the default, or reconstructing from a partial source. Verified by mutation
rather than argued: forcing ``rate_class`` to ``"fast"`` in the Python rewrite turns
**three of the four tests below red**.

**Scope, stated rather than implied.** This anchors the *structure* — which set each
flow lands in — on the Python port. `compose.rs` is anchored by the mirror of these
assertions in ``rust/crates/authoring/tests/scenario_files.rs``, and the two ports are
compared to *each other* by the ``two_batteries_multirate.yaml`` row in
``tests/crossport/authoring_files.py``. A Python test alone would leave the named file
exactly as unanchored as Step 6b found it.
"""

from dataclasses import replace
from pathlib import Path

from authoring.compose import apply_includes
from authoring.interpreter import load_scenario
from authoring.run import run_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.ids import StockId
from simcore.registry import Registry

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
TWO_BATTERIES_MULTIRATE = SCENARIO_DIR / "two_batteries_multirate.yaml"

FAST_FLOW = "bat_fast.power.self_discharge"
SLOW_FLOW = "bat_slow.power.self_discharge"
# The stock BOTH rate classes write — what makes the partition trajectory-visible.
SHARED_STOCK = StockId("bat_slow.power.battery")
# The control: touched only by the fast instance, so it must stay bit-identical.
UNSHARED_STOCK = StockId("bat_fast.power.battery")


def _final_state(built):
    states, rationed, events = run_scenario(built)
    return states[-1], rationed, events


def test_a_bundle_declared_rate_class_survives_prefixing() -> None:
    """`apply_includes` — the claim at its own level, before the interpreter sees it.

    The two bundles differ by exactly one key, so the two prefixed copies of the *same*
    flow must come out of the merge with *different* rate classes. A rewrite that
    dropped the field would collapse them both to `"fast"`.
    """
    spec = ScenarioSpec.model_validate(load_yaml(TWO_BATTERIES_MULTIRATE))
    merged = apply_includes(spec, TWO_BATTERIES_MULTIRATE.parent)

    by_id = {flow.id: flow for flow in merged.flows}
    assert SLOW_FLOW in by_id, (
        f"the namespaced bundle flow is missing entirely; merged ids: {sorted(by_id)}"
    )
    assert by_id[SLOW_FLOW].rate_class == "slow", (
        "prefixing dropped `rate_class`: the flow came back as "
        f"{by_id[SLOW_FLOW].rate_class!r}. `rate_class` is a property, not an id "
        "reference — the rewrite must carry it verbatim."
    )
    assert by_id[FAST_FLOW].rate_class == "fast"


def test_the_prefixed_slow_flow_lands_in_the_built_slow_registry() -> None:
    """The interpreter — the partition is computed over the POST-include spec.

    This is the assertion the whole fixture exists for: the slow set is non-empty and
    contains the flow under its **namespaced** id. Note what is being excluded — an
    empty slow set here is not an error on any port, which is why the failure this
    guards against is silent.
    """
    built = load_scenario(str(TWO_BATTERIES_MULTIRATE))

    assert built.is_multirate
    assert {f.id for f in built.slow_registry.flows} == {SLOW_FLOW}, (
        "the bundle-contributed slow flow did not reach the built partition. An empty "
        "slow set here is a legal, silent, single-rate-equivalent build — which is why "
        "this is asserted rather than left to the run."
    )
    assert {f.id for f in built.fast_registry.flows} == {
        FAST_FLOW,
        "power.trickle_load",
    }


def test_an_inline_flow_may_reference_a_namespaced_id() -> None:
    """The merge is one flat graph, not two scopes.

    `power.trickle_load` is declared inline and wired to `bat_slow.power.battery`, an id
    that exists only *because* of the prefix. It resolves, so the scenario builds and
    the stock carries both flows' legs.
    """
    spec = ScenarioSpec.model_validate(load_yaml(TWO_BATTERIES_MULTIRATE))
    merged = apply_includes(spec, TWO_BATTERIES_MULTIRATE.parent)

    touching = {
        flow.id
        for flow in merged.flows
        if flow.kinetics is not None and SHARED_STOCK in flow.kinetics.stoichiometry
    }
    assert touching == {SLOW_FLOW, "power.trickle_load"}, (
        "the shared stock must be written by BOTH rate classes — that overlap is what "
        "makes the partition trajectory-visible (see the decorative-fixture test)."
    )
    # It builds, which is the referential-integrity half: an inline `stock(...)` ref to
    # an id that only exists post-prefix resolves against the one flat merged graph.
    assert load_scenario(str(TWO_BATTERIES_MULTIRATE)).is_multirate


def test_the_partition_is_not_decorative() -> None:
    """The rate-class boundary and the shared-stock boundary OVERLAP — measured.

    Step 6b's lesson, applied to this fixture rather than rediscovered by it: two
    disjoint instances would make the Strang operators commute exactly, so the
    trajectory would be identical whether or not the partition survived, and the run
    half of the cross-port row would pass on a port that had dropped `rate_class`
    entirely.

    `power.trickle_load` (fast) drains the stock the slow flow also writes, so the
    splitting error is genuinely nonzero. **It is also genuinely small** — `k*dt` is
    3.6e-5 here, so the delta is ~1.1 J on ~9.88e6 J (~1.1e-7 relative), not Step 6b's
    29 %. That is reported rather than inflated: the cross-port comparison for this
    Tier-1 file is **bit-exact**, so any nonzero delta is a live gate, and the property
    under test is "the partition reaches the driver at all", not its magnitude.
    """
    built = load_scenario(str(TWO_BATTERIES_MULTIRATE))
    partitioned, rationed, events = _final_state(built)
    assert rationed == 0
    assert events == ()

    # Exactly what a dropped `rate_class` produces downstream: everything fast.
    all_fast = replace(
        built,
        slow_registry=Registry([], built.state.stocks),
        fast_registry=Registry(list(built.registry.flows), built.state.stocks),
    )
    collapsed, _, _ = _final_state(all_fast)

    assert (
        partitioned.stocks[SHARED_STOCK].amount != collapsed.stocks[SHARED_STOCK].amount
    ), (
        "the partitioned and all-fast trajectories agree bit-for-bit on the shared "
        "stock — the fixture has gone decorative. Check that `power.trickle_load` "
        "still drains the SLOW instance's battery."
    )
    # The untouched instance is the control: it shares no stock across the boundary, so
    # it must be bit-identical. This is what makes the delta above attributable.
    assert (
        partitioned.stocks[UNSHARED_STOCK].amount
        == collapsed.stocks[UNSHARED_STOCK].amount
    )
