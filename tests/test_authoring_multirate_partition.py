"""Multi-rate authoring, Step 2: the author-facing knob — schema + interpreter.

Step 1 measured the two facts this phase rests on
(``tests/test_authoring_multirate_identity.py``): ``n_sub=1`` + an empty slow set
reproduces the single-rate trajectory bit-for-bit *and* the frozen ECLSS golden
byte-for-byte, while **every** non-empty slow partition at ``n_sub=1`` silently
perturbs it. Both were measured with the partition built **by hand** in the test. This
file pins the layer that now builds it from the file: the ``n_sub`` + per-flow
``rate_class:`` keys, lowered by :func:`authoring.interpret`.

**The load-bearing test here is**
:func:`test_the_interpreter_partition_matches_the_hand_partition` — it ties the new code
directly to Step 1's already-measured payoff and identity, rather than re-proving
trajectories. Everything Step 1 measured about the hand-partition transfers to the
authored one exactly insofar as the two registries are equal, so that is what is
asserted.

**What this file does NOT claim.** Nothing here *runs* multi-rate through
:func:`authoring.run.run_scenario`, which is single-rate until Step 3 — ``interpret``
now *builds* the partition, but no harness consumes it yet. The aux tripwire
(``multirate_step`` never advances aux) likewise belongs to Step 3, where the driver is
actually called.

**The legality matrix, which is the whole design in four rows:**

===========  ==============  ======================================================
``n_sub``    slow set        verdict
===========  ==============  ======================================================
1            empty           legal — the identity path (today's files, untouched)
1            non-empty       ``AuthoringError`` — no separation, no perf win, yet
                             moves the answer (Step 1 measured all three shapes)
> 1          empty           **legal** — uniform sub-stepping; the measured payoff
                             (master ``dt=3600`` + ``n_sub=60`` == ``dt=60``)
> 1          non-empty       legal — true multi-rate, the composability case
===========  ==============  ======================================================
"""

import copy
from pathlib import Path
from typing import Any

import pytest
from pydantic import ValidationError

from authoring.errors import AuthoringError
from authoring.interpreter import BuiltScenario, interpret, load_scenario
from authoring.schema import ScenarioSpec
from config import load_yaml
from simcore.registry import Registry

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
ECLSS_YAML = SCENARIO_DIR / "eclss_cabin.yaml"

# The ECLSS anchor's four flows — the same graph Step 1 measured the identity on.
SCRUBBER = "eclss.co2_scrubber"
MAKEUP = "eclss.o2_makeup"
METABOLISM = "eclss.crew_metabolism"
CONDENSER = "eclss.condenser"


def _raw(slow: tuple[str, ...] = (), **edits: Any) -> dict[str, Any]:
    """The ECLSS anchor's raw YAML: ``rate_class: slow`` on ``slow``, + config edits.

    The anchor **file** is never edited (the ``test_authoring_dt_hazard._build_at`` /
    Step-1 ``_at_dt`` discipline): only its in-memory dict, so the graph under test
    stays the committed one.
    """
    raw: dict[str, Any] = copy.deepcopy(load_yaml(str(ECLSS_YAML)))
    raw.update(edits)
    for flow in raw["flows"]:
        if flow["id"] in slow:
            flow["rate_class"] = "slow"
    return raw


def _build(slow: tuple[str, ...] = (), **edits: Any) -> BuiltScenario:
    return interpret(ScenarioSpec.model_validate(_raw(slow, **edits)), SCENARIO_DIR)


def _ids(registry: Registry) -> tuple[str, ...]:
    return tuple(str(flow.id) for flow in registry.flows)


# ---------------------------------------------------------------------------
# 1. THE IDENTITY SHAPE — a scenario with no multi-rate keys is untouched
# ---------------------------------------------------------------------------


def test_a_scenario_with_no_multirate_keys_lowers_to_the_single_rate_shape() -> None:
    # The golden-preservation argument at the SHAPE level: every committed scenario
    # predates these keys, so it must lower with an empty slow set at n_sub=1 — the
    # combination Step 1 measured as bit-identical to single-rate. `is_multirate` is
    # False, which is what lets Step 3's harness keep today's code path verbatim for
    # every existing file rather than leaning on the identity holding.
    built = load_scenario(str(ECLSS_YAML))
    assert built.n_sub == 1
    assert built.slow_registry.flows == ()
    assert built.fast_registry.flows == built.registry.flows
    assert built.is_multirate is False


def test_the_partition_is_disjoint_and_total() -> None:
    # N3's structural invariant, asserted rather than assumed: slow ∪ fast == registry
    # and slow ∩ fast == ∅. A flow lost from both sets would silently stop being
    # integrated — a conservation-clean, wrong run — and a flow in BOTH would be
    # applied twice per master step. Neither is something a trajectory test localizes.
    built = _build(slow=(SCRUBBER, MAKEUP), n_sub=4)
    slow, fast = set(_ids(built.slow_registry)), set(_ids(built.fast_registry))
    assert slow | fast == set(_ids(built.registry))
    assert slow & fast == set()


# ---------------------------------------------------------------------------
# 2. THE LOAD-BEARING TIE — the authored partition IS Step 1's hand partition
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "slow",
    [
        (SCRUBBER,),  # donor-controlled
        (MAKEUP,),  # demand-controlled — the one rationing cannot see
        (METABOLISM,),  # FORCED
        (SCRUBBER, CONDENSER),  # a two-flow slow set
        (),  # the empty slow set at n_sub > 1 — uniform sub-stepping
    ],
)
def test_the_interpreter_partition_matches_the_hand_partition(
    slow: tuple[str, ...],
) -> None:
    # THE TEST THIS FILE EXISTS FOR. Step 1 measured the identity and the payoff on a
    # partition built BY HAND inside the test (`_run_multirate`'s `slow_ids` filter
    # over `built.registry.flows`). This asserts the interpreter, driven by the
    # `rate_class:` keys in the file, produces exactly that — same flow objects, same
    # canonical
    # order, in both registries. Everything Step 1 measured therefore transfers to the
    # authored path by equality, with no trajectory re-run: the two are the same input
    # to `multirate_step`.
    built = _build(slow=slow, n_sub=60)
    flows = list(built.registry.flows)
    stocks = built.state.stocks
    expected_slow = Registry([f for f in flows if str(f.id) in slow], stocks)
    expected_fast = Registry([f for f in flows if str(f.id) not in slow], stocks)
    assert built.slow_registry.flows == expected_slow.flows
    assert built.fast_registry.flows == expected_fast.flows


def test_the_rate_class_key_does_not_disturb_the_graph_it_partitions() -> None:
    # A `rate_class:` key is a routing annotation, not a modelling one: it must not
    # change WHICH flows exist, their canonical order, their params, or the initial
    # state.
    # Only the partition moves. (If this failed, the Step-1 identity would not
    # transfer — the graph itself would differ before any splitting.)
    plain = _build()
    annotated = _build(slow=(SCRUBBER,), n_sub=2)
    assert annotated.registry.flows == plain.registry.flows
    assert annotated.state == plain.state


# ---------------------------------------------------------------------------
# 3. THE REFUSALS — the two combinations that are errors, not behaviours
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("slow_id", [SCRUBBER, MAKEUP, METABOLISM])
def test_n_sub_1_with_a_slow_flow_is_refused(slow_id: str) -> None:
    # The mirror of Step 1's measurement, promoted from "it perturbs" to "it is
    # refused". Parametrized over the same three flow shapes Step 1 measured, because
    # the two intuitive hypotheses are both false — a forced flow is NOT safe (its own
    # legs split exactly, yet the cabin moves ~1e-01 because the fast flows read a
    # half-metabolised cabin) and a self-contained flow perturbs its OWN stocks. So the
    # refusal cannot be narrowed to "coupled flows only".
    with pytest.raises(AuthoringError, match="n_sub=1 with a non-empty slow set"):
        _build(slow=(slow_id,))


def test_unknown_rate_class_is_refused() -> None:
    # The `quantity`/`kind`/`integrator` discipline: an unknown value in the author's
    # vocabulary is an AuthoringError naming the legal set, not a silent default. Left
    # to a bare truthiness check, `rate_class: fastest` would partition as *slow* — a
    # typo that silently moves the answer, the exact failure `extra="forbid"` exists to
    # prevent one level up.
    raw = _raw()
    raw["flows"][0]["rate_class"] = "fastest"
    with pytest.raises(AuthoringError, match="unknown rate class 'fastest'"):
        interpret(ScenarioSpec.model_validate(raw), SCENARIO_DIR)


@pytest.mark.parametrize("n_sub", [0, -1])
def test_n_sub_below_one_is_a_schema_error(n_sub: int) -> None:
    # `ge=1` mirrors multirate_step's own `n_sub >= 1` guard at the FILE boundary, so a
    # nonsensical value is caught on the author's file rather than surfacing as a
    # ValueError from the core mid-run. A schema error, deliberately — this is a shape
    # constraint (cf. IncludeSpec.prefix's min_length=1), not a graph-level judgement.
    with pytest.raises(ValidationError):
        ScenarioSpec.model_validate(_raw(n_sub=n_sub))


# ---------------------------------------------------------------------------
# 4. THE LEGAL MULTI-RATE SHAPES
# ---------------------------------------------------------------------------


def test_uniform_substepping_needs_no_slow_set() -> None:
    # n_sub > 1 with an EMPTY slow set is legal, and this row is not a curiosity: it is
    # the configuration the phase's measured payoff rests on (Step 1: master dt=3600
    # with n_sub=60 lands on the same cabin_o2 as a single-rate dt=60 run, while
    # exporting 60x less often). It decouples the export cadence from the solver step
    # with no partition at all — refusing it "because nothing is slow" would refuse the
    # headline result.
    built = _build(n_sub=60, dt=3600.0)
    assert built.slow_registry.flows == ()
    assert built.is_multirate is True


def test_a_true_partition_at_n_sub_gt_1_builds() -> None:
    # The composability case the authoring reference calls impossible ("there is no dt
    # natural to both domains"): a slow set stepping at the master cadence while the
    # fast set sub-steps. Step 4 makes this a real Thermal+ECLSS anchor; here it is the
    # shape only.
    built = _build(slow=(SCRUBBER,), n_sub=60, dt=3600.0)
    assert _ids(built.slow_registry) == (SCRUBBER,)
    assert MAKEUP in _ids(built.fast_registry)
    assert built.is_multirate is True


# ---------------------------------------------------------------------------
# 5. COMPOSITION — the reason the check is in the interpreter, not the schema
# ---------------------------------------------------------------------------

_SLOW_BUNDLE = """\
stocks:
  - id: a.pool
    domain: a
    quantity: energy
    kind: pool
    amount: 1.0e+3
  - id: boundary.a_sink
    domain: boundary
    quantity: energy
    kind: boundary
    amount: 0.0
flows:
  - id: a.leak
    rate_class: slow
    kinetics:
      rate: 'stock("a.pool")'
      stoichiometry:
        a.pool: -1
        boundary.a_sink: 1
"""

_HEAD = """\
name: s
integrator: euler
dt: 1.0
steps: 1
"""


def _write(tmp: Path, name: str, text: str) -> Path:
    path = tmp / name
    path.write_text(text, encoding="utf-8")
    return path


def test_a_bundle_contributed_slow_flow_is_seen(tmp_path: Path) -> None:
    # WHY THE VALIDATION LIVES IN THE INTERPRETER. A bundle may declare a slow
    # `rate_class`, and bundles merge via `apply_includes` at the TOP of `interpret` —
    # after schema validation. A pydantic `model_validator` on ScenarioSpec sees only
    # the scenario's own inline `flows`, so it would miss this flow entirely and lower
    # a partitioned scenario as if it were single-rate: silent, and wrong in the
    # direction that moves numbers.
    #
    # This fixture also carries `rate_class:` and `kinetics.rate:` on the SAME flow —
    # the two senses of the word, one nesting level apart. It is why the cadence key is
    # not spelled `rate`: here the distinction is visible, not inferred.
    _write(tmp_path, "b.yaml", _SLOW_BUNDLE)
    scenario = _write(tmp_path, "s.yaml", _HEAD + "n_sub: 4\nincludes:\n  - b.yaml\n")
    built = load_scenario(str(scenario))
    assert _ids(built.slow_registry) == ("a.leak",)
    assert built.is_multirate is True


def test_a_bundle_contributed_slow_flow_reaches_the_n_sub_1_refusal(
    tmp_path: Path,
) -> None:
    # The teeth of the test above: the same include WITHOUT n_sub must hit the refusal.
    # If rate_class were read pre-compose (or not at all), this would build cleanly
    # and run a perturbed trajectory reported as normal — so this is what proves the
    # check actually sees through the merge, rather than the previous test passing for
    # some unrelated reason.
    _write(tmp_path, "b.yaml", _SLOW_BUNDLE)
    scenario = _write(tmp_path, "s.yaml", _HEAD + "includes:\n  - b.yaml\n")
    with pytest.raises(AuthoringError, match="n_sub=1 with a non-empty slow set"):
        load_scenario(str(scenario))


def test_prefixing_a_bundle_rewrites_the_id_but_never_the_rate_class(
    tmp_path: Path,
) -> None:
    # THE ARGUMENT FOR SHAPE (a), MADE EXECUTABLE. The rate class is a *property*
    # carried by the flow, so `{bundle, prefix}` namespacing — which rewrites ids and
    # every reference to them — cannot touch it: the flow arrives as `p.a.leak`, still
    # slow.
    #
    # The rejected alternative, a top-level `fast: [flow-id, …]` list, is a list of id
    # REFERENCES, so it would have needed a matching rewrite in `apply_includes` and
    # would silently mis-fire the moment one was forgotten. This pins that shape (a)
    # has zero referential-integrity surface — the property travels inside the bundle,
    # free.
    _write(tmp_path, "b.yaml", _SLOW_BUNDLE)
    scenario = _write(
        tmp_path,
        "s.yaml",
        _HEAD + "n_sub: 4\nincludes:\n  - bundle: b.yaml\n    prefix: p\n",
    )
    built = load_scenario(str(scenario))
    assert _ids(built.slow_registry) == ("p.a.leak",)
