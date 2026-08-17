"""Phase-9 Step-7 (P9.7): the AUTHORING freeze manifest + its completeness gate.

The machine-readable companion to ``docs/authoring-reference.md`` (the human-readable
authoring freeze contract), one level up from the biosphere/station manifests: those
freeze **science** (params, flows, scenarios → goldens); this one freezes the **author-
facing platform surface** — the bounded kinetics **grammar** + the **VM**'s node/op set,
the scenario **file schema**, the author-selectable **flow-type registry**, and the
named **param loaders**. A mod authored against these today must keep working tomorrow,
so moving any of them is an unfreeze event, not an ordinary edit.

**Why a manifest here and not a doc-only reference (the Phase-8 contrast).** Phase 8 got
``docs/phase-8-reference.md`` with *no* manifest because it added a consumer (the Godot
binding) and changed no science — there was no new frozen surface to gate. Phase 9 is
the opposite: it adds real author-facing surface with no prior owner. A grammar/schema/
registry change is exactly the kind of silent break a completeness gate exists to catch.

**What the existing artifacts already enforce (so this file does NOT re-check).**

* **Grammar semantics** — ``rust/crates/authoring/tests/data/parse_vectors.txt`` (20
  accept → an identical canonical S-expr on both ports + 16 reject → both-error). A
  precedence or associativity change moves an S-expr rendering and fails there.
* **The VM's arithmetic** — ``…/tests/data/traj_vectors.txt`` (the frozen
  ``SelfDischarge`` re-expression, bit-exact per step under Euler *and* RK4). An
  op-order or evaluation change fails there.
* **The interpreter** — the nine crossport anchors
  (``tests/crossport/authoring_files.py`` :data:`ANCHORS`), incl. the byte-identity of
  an authored crew run vs the frozen ``crew_state.json``.
* **Param values** — the ``crew`` / ``self_discharge`` param files are frozen by
  ``docs/station-reference.manifest.json`` (named here via ``delegates_to``, **not**
  re-hashed — the biosphere/station delegation discipline).

**The gap this gate owns: completeness.** None of the above can see a grammar node, a
binary op, a schema field, a spec model, a flow type, or a param loader **added to the
live tree but exercised by no vector and no anchor** — the biosphere's "added a flow,
wired into no golden" hole, one level up. So every frozen set below is **derived from
its live single source of truth, never hand-listed**, and asserted set-equal to it:

* ``expr_nodes`` from ``typing.get_args(simcore.expr.Expr)`` (the closed node union);
* ``binary_ops`` from ``simcore.expr._BINARY_OPS``, ``ref_keywords`` from
  ``authoring.expr_parser._REF_KEYWORDS``, ``integrator_names`` from
  ``authoring.run._INTEGRATORS`` (private, and deliberately so — each *is* the single
  source of truth, which is the whole point of deriving rather than transcribing);
* ``schema_fields`` from pydantic ``model_fields``, over the spec models found by
  **scanning** ``authoring.schema`` (so a whole new spec model is caught too, not just a
  new field on a known one);
* ``flow_types`` from :data:`authoring.flow_registry.FLOW_TYPES` (+ each type's
  ``wiring_fields`` / ``param_set`` — the wiring names are as much the authoring
  contract as the type name); ``param_loaders`` from
  :data:`~authoring.flow_registry.PARAM_LOADERS`.

**The VM is frozen by its grammar surface, not by a hash of ``expr.py``** (advisor): a
code hash would add reformat/lint noise without a real gate, and the VM's *behavior* is
already pinned bit-exactly by ``traj_vectors.txt``. Its **grammar** — the node union and
the op set — is the part an author writes against, so that is what the manifest freezes.

**⚠⚠ SLICE 8 OF THE REFERENCE FLIP INVERTED THIS FILE, AND THE OLD SENTENCE IS WORTH
QUOTING BECAUSE IT IS NOW FALSE.** It read: *"The manifest freezes the **Python**
surface of record. The Rust mirror is gated by the parse/traj vectors + the anchors, not
by this gate."* Since 2026-08-17 the manifest's whole platform half is **generated from
the Rust tree** — ``_build_manifest`` shells ``cargo run --example
dump_authoring_inventory`` and splices in every key :data:`_AUTHORITY` marks ``rust`` —
and the derivations below have become **conformance checks on the checker**: they still
run, still derive from the live Python package, and now answer *"has Python drifted from
the frozen contract?"* rather than *"what is the contract?"*.

**⚠⚠ What that trade actually cost, stated plainly, because this contract is the one
where the flip is NOT free.** The biosphere and station manifests re-anchored to a
**runtime enumeration of a built registry**, which Rust does identically. Nothing here
has that shape. Python's derivations use language introspection Rust does not have —
``typing.get_args`` over a closed union, a scan of ``vars(authoring.schema)``, pydantic
``model_fields``, a dict — so on the reference side an ``enum``, a ``match`` and a set
of ``const``s stand in their place, and **the completeness census is weaker there than
it is here**. `authoring::surface`'s module docs say which half of each axis is derived
and which is a hand-maintained roster; the manifest's ``_authority`` block repeats it
per key. Read "side: rust" on *this* contract with that qualifier attached.

The mitigation, where it exists, is that the reference-side roster is **load-bearing**
rather than descriptive: ``ref_keywords``, ``step_token`` and every ``schema_fields``
list are the tables the Rust parser actually rejects against, so dropping a name changes
what the platform *accepts* and reddens the parse vectors and the crossport anchors.
Where even that is unavailable (``expr_nodes``, ``binary_ops``, the ``flow_types``
roster, ``integrator_names``) the residue is written down rather than implied.

Regeneration is a deliberate, separate ``__main__`` action (the golden discipline): on
an advisor-reviewed unfreeze, run ``uv run python
tests/test_authoring_freeze_manifest.py`` and review the manifest diff. ⚠ **Since slice
8 it needs ``cargo``**, because it reads the reference. The tests do not: nothing in
this module shells cargo, and the base suite stays offline-clean. The cargo-side
staleness gate lives in ``tests/crossport/test_inventory_parity.py``. Zero ``simcore``
change, zero ``domains`` change on the Python side (``git diff src/`` is empty for
this slice).
"""

from __future__ import annotations

import hashlib
import inspect
import json
import subprocess
import sys
from functools import lru_cache
from pathlib import Path
from typing import Any, get_args

from pydantic import BaseModel

import authoring.schema
import simcore.expr
from authoring.expr_parser import _REF_KEYWORDS
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS
from authoring.interpreter import _RATE_CLASSES
from authoring.run import _INTEGRATORS
from simcore.expr import _BINARY_OPS

_REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = _REPO_ROOT / "docs" / "authoring-reference.manifest.json"

#: The reference tree's own dump of the platform surface — the producer half of this
#: manifest since slice 8. Its doc comment is the authority on what it emits and why.
_RUST_CRATE_DIR = _REPO_ROOT / "rust" / "crates" / "authoring"
_RUST_DUMP_EXAMPLE = "dump_authoring_inventory"

#: The keys :func:`_build_manifest` consumes out of that dump, asserted as its **exact**
#: key set. ⚠ A forcing function, not a filter (slice 3's move, kept through 6, 7 and
# 8): : a key added to the dump turns regeneration into a loud error rather than
# silently : entering — or silently *not* entering — the frozen surface. In particular a
#: ``parity_vectors`` key must not reach the manifest this way; see :data:`_AUTHORITY`.
_RUST_DUMP_KEYS = frozenset(
    {
        "binary_ops",
        "expr_nodes",
        "flow_types",
        "integrator_names",
        "param_loaders",
        "rate_classes",
        "ref_keywords",
        "schema_fields",
        "step_token",
    }
)


@lru_cache(maxsize=1)
def _rust_reference() -> dict[str, Any]:
    """Run the reference tree's platform dump and parse its JSON.

    ⚠ **Called only from :func:`_build_manifest`, i.e. only from the regeneration
    ``__main__``.** No test in this module reaches it, so the base suite neither needs
    ``cargo`` nor pays for a build.
    """
    proc = subprocess.run(
        ["cargo", "run", "-q", "--example", _RUST_DUMP_EXAMPLE],
        cwd=_RUST_CRATE_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"cargo run --example {_RUST_DUMP_EXAMPLE} failed — this manifest is "
            f"regenerated FROM the Rust reference since slice 8 of the reference flip, "
            f"so regeneration needs a working Rust toolchain:\n{proc.stderr}"
        )
    dump: dict[str, Any] = json.loads(proc.stdout)
    if set(dump) != _RUST_DUMP_KEYS:
        raise SystemExit(
            f"{_RUST_DUMP_EXAMPLE} emitted {sorted(dump)}, expected "
            f"{sorted(_RUST_DUMP_KEYS)}. Read _AUTHORITY before widening this: a new "
            "key has to be classified, and one that cannot honestly come from the "
            "reference (a hash of a Python-generated vector file) must not enter the "
            "manifest through here."
        )
    return dump


# The station manifest owns the frozen param VALUES an authored scenario reaches through
# the named loaders (`crew.yaml`, `self_discharge.yaml`). Delegated, never re-hashed —
# the same pointer discipline by which the station manifest delegates the biosphere.
STATION_MANIFEST = "docs/station-reference.manifest.json"

# The two committed cross-port vector files that own the grammar's SEMANTICS (parse) and
# the VM's ARITHMETIC (traj). Recorded here as provenance — a re-derivable record of
# which cases were frozen — not asserted (their content is gated by the crossport
# in-sync guards + the Rust vector tests).
VECTOR_DIR = _REPO_ROOT / "rust" / "crates" / "authoring" / "tests" / "data"
VECTOR_FILES: tuple[str, ...] = ("parse_vectors.txt", "traj_vectors.txt")


def _normalized_sha256(path: Path) -> str:
    """sha-256 over newline-normalized (LF) content — a reproducible provenance hash.

    Hashing raw bytes would make the value depend on the checkout's line endings
    (``autocrlf`` on Windows vs. LF on Linux). Normalizing to LF first makes the hash a
    stable record of *content*. Provenance, not a gate (see the module docstring).
    """
    text = path.read_text(encoding="utf-8")
    normalized = "\n".join(text.splitlines())
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def _expr_nodes() -> list[str]:
    """The frozen AST node set — derived from the closed ``Expr`` union itself.

    ``Expr = Const | StockRef | … | BinOp`` is the grammar's node vocabulary; a node
    added to the union but exercised by no vector still appears here.
    """
    return sorted(node.__name__ for node in get_args(simcore.expr.Expr))


def _spec_models() -> dict[str, type[BaseModel]]:
    """The scenario-file spec models — found by scanning ``authoring.schema``.

    Derived, not hand-listed, so a whole **new** spec model (not merely a new field on a
    known one) is caught by the completeness gate. The ``__module__`` filter drops
    ``BaseModel`` and anything merely imported into the module's namespace.
    """
    return {
        name: obj
        for name, obj in vars(authoring.schema).items()
        if inspect.isclass(obj)
        and issubclass(obj, BaseModel)
        and obj.__module__ == authoring.schema.__name__
    }


def _schema_fields() -> dict[str, list[str]]:
    """Each spec model → its frozen field names (pydantic ``model_fields``).

    The file schema *is* the authoring contract: a field added (or, worse, renamed)
    changes what a committed scenario file may say. Every model is ``extra="forbid"``,
    so the field set is exactly the legal key set.
    """
    return {name: sorted(model.model_fields) for name, model in _spec_models().items()}


def _flow_types() -> dict[str, dict[str, Any]]:
    """The author-selectable frozen-flow surface — derived from ``FLOW_TYPES``.

    Each entry records the frozen class it lowers to, the exact ``wiring`` field set the
    interpreter demands, the named param set (or ``None`` for a param-free flow), and
    the ``rate_params`` the build-time ``k·h < 1`` precondition checks. The wiring names
    are part of the contract an author writes against, not an implementation detail — a
    renamed wiring field breaks every file that names it.

    **``rate_params`` belongs here because it is author-visible behavior, not metadata**
    (multi-rate Step 5): it decides which scenarios the platform *refuses to build*.
    Dropping a name from it silently un-checks a flow — a committed file that used to be
    rejected starts lowering, which is exactly the "added but exercised by nothing" hole
    one level up. Emitting it from *this* derivation is what freezes it: the gate
    compares the manifest against this function, so a field the function omits is a
    field the manifest cannot pin, however faithfully ``FlowTypeSpec`` records it.

    **``demand_controlled`` belongs here for the identical reason** (the reversal gate,
    2026-08-11): it decides which scenarios ``run_scenario`` *refuses to return*.
    Clearing it on a type silently un-arms the direction check for that type — a
    committed file that used to raise ``ReversedFlowError`` starts completing quietly,
    the same hole in the same shape. Recorded as ``[field, param]``, or ``None``.
    """
    return {
        name: {
            "cls": spec.cls.__name__,
            "wiring_fields": sorted(spec.wiring_fields),
            "param_set": spec.param_set,
            "rate_params": sorted(spec.rate_params),
            "demand_controlled": (
                list(spec.demand_controlled) if spec.demand_controlled else None
            ),
        }
        for name, spec in FLOW_TYPES.items()
    }


#: Who produces each frozen field, keyed by ``/``-joined leaf **path** — the slice-6/7
#: classification, applied to the platform contract.
#:
#: ⚠ The three sides are claims of different kinds. ``rust`` — produced by the reference
#: tree and spliced in by :func:`_build_manifest`. ``python`` — produced by the checker
#: because the reference has no honest referent, with the reason and the condition under
#: which that could change. ``hand`` — a literal, a label or prose deliberately derived
#: from neither.
#:
#: ⚠⚠ **``rust`` means something weaker on this contract than on the other two, and
# every : entry below says how.** There is no built registry to walk here (see the
# module : docstring), so each Rust-side axis is part derived and part hand-maintained
# roster. : Recorded per key rather than once, because the halves differ per key:
# ``schema_fields`` : is load-bearing on the parser, ``expr_nodes`` is compiler-forced
# but roster-listed, and : ``flow_types`` is roster-listed with fully derived entries.
_AUTHORITY: dict[str, dict[str, str]] = {
    "_comment": {"side": "hand", "why": "prose header"},
    "frozen_at_phase": {"side": "hand", "why": "the phase this surface froze at"},
    "reference_doc": {
        "side": "hand",
        "why": "pointer to the prose half of the contract",
    },
    "delegates_to": {
        "side": "hand",
        "why": (
            "pointer to the station manifest, which owns the param VALUES an authored "
            "file reaches through the named loaders. A path, not a derived value — its "
            "target's existence is checked by test_manifest_named_files_exist"
        ),
    },
    "grammar_note": {
        "side": "hand",
        "why": (
            "prose, and the one sentence a reader must not misread: the frozen grammar "
            "is DELIBERATELY INCOMPLETE. Neither port derives it and neither should — "
            "it records decisions (which ops are deferred and why), not state"
        ),
    },
    "expr_nodes": {
        "side": "rust",
        "why": (
            "the node names of simcore::expr::Expr, via surface::expr_node_name's "
            "exhaustive match. ⚠ HALF-DERIVED: adding a variant is a COMPILE ERROR "
            "until someone names it, but the emitted list comes from "
            "surface::sample_nodes, a hand-maintained roster of one inhabitant per "
            "variant. Python's side is typing.get_args(Expr) and is genuinely derived "
            "— that is the checker's advantage on this axis, and it is why the "
            "conformance gate below still matters after the flip"
        ),
    },
    "binary_ops": {
        "side": "rust",
        "why": (
            "BinaryOp::symbol() — the parser's and the S-expr renderer's own spelling. "
            "⚠ Same half-derived shape as expr_nodes: the SYMBOLS are derived, the "
            "roster of three variants is hand-maintained. '/' is absent from the TYPE, "
            "not merely from the list, so an unsupported op is unrepresentable"
        ),
    },
    "ref_keywords": {
        "side": "rust",
        "why": (
            "expr_parser::REF_KEYWORDS. ⚠ LOAD-BEARING, and the strongest tie on this "
            "contract: this is the table the parser tests an identifier against, so a "
            "name dropped here is not a mis-description, it is a form the grammar "
            "stops accepting — the parse vectors and the anchors go red"
        ),
    },
    "step_token": {
        "side": "rust",
        "why": (
            "expr_parser::STEP_TOKEN, hoisted out of the lexer's ident match in slice "
            "8 for the reason above: the token spliced here is the token that lowers "
            "to StepN. ⚠ NOT the integrator's treatment — that one stayed 'hand' on "
            "both other contracts because neither side had an importable name; this "
            "one does"
        ),
    },
    "integrator_names": {
        "side": "rust",
        "why": (
            "run::INTEGRATOR_NAMES. ⚠ The WEAKEST axis here: Python's copy is the "
            "dispatch dict itself and cannot lie, while Rust's dispatch is a match and "
            "this is a hand-maintained slice beside it. Two partial closes — both "
            "match arms build their error message from the slice, and "
            "integrator_names_all_ dispatch RUNS a scenario under every listed name. A "
            "match arm added and not listed is caught by nothing on the reference side"
        ),
    },
    "rate_classes": {
        "side": "rust",
        "why": (
            "interpreter::RATE_CLASSES, the list the interpreter validates a flow's "
            "rate_class against — load-bearing like ref_keywords. Closed at two by "
            "multirate_step's signature (it takes exactly two Substeppers), so a third "
            "cannot appear without a simcore change"
        ),
    },
    "schema_fields/*": {
        "side": "rust",
        "why": (
            "the eight schema::*_KEYS consts. ⚠ LOAD-BEARING on the values: each list "
            "IS the argument reject_unknown_keys refuses against, so it decides "
            "whether a committed scenario file loads at all. ⚠ The LABELS "
            "(ScenarioSpec, ParamPackRef, …) are Python class names typed by hand, and "
            "two name no Rust type at all (this port binds params: and includes: "
            "through enums) — they are contract names, the way crew.food_metabolism "
            "is. std::any::type_name was refused for slice 2's reason: its format is "
            "unspecified and a toolchain bump must not move a frozen manifest. ⚠ And "
            "the completeness half does NOT port: Python catches a whole new spec "
            "model by scanning its module; here a new spec is a new const nothing "
            "forces into the dump. The compile-forced version (a SpecKind enum "
            "threaded through reject_unknown_keys) was priced in slice 8 and deferred "
            "as its own change, per slice 7 declining locked_dt"
        ),
    },
    "flow_types/*/*": {
        "side": "rust",
        "why": (
            "⚠ ROSTER hand-maintained, ENTRIES fully derived. The names come from "
            "flow_registry::FLOW_TYPE_NAMES, which that module already documents as "
            "hand-maintained because a Rust match cannot be enumerated. Everything "
            "about each type is read off its FlowTypeSpec, and `cls` off a flow the "
            "dump actually CONSTRUCTS through build_frozen_flow — the same path an "
            "authored scenario takes, and the one place slice 2's Flow::type_name() "
            "pays out on this contract. So a renamed wiring field, a moved param set, "
            "a dropped rate param or a cleared demand-control pair all move the "
            "manifest"
        ),
    },
    "param_loaders": {
        "side": "rust",
        "why": (
            'flow_registry::PARAM_SET_NAMES — the sets a kinetics rate\'s param("…") '
            "may read. ⚠ Do NOT confuse this with the param_files key the other two "
            "manifests keep Python-retained until slice 9: that one is a list of YAML "
            "FILES with their hashes, and this is the loader-name vocabulary. The "
            "values behind these names are delegated to the station manifest, not "
            "frozen here"
        ),
    },
    "parity_vectors/*": {
        "side": "python",
        "why": (
            "PYTHON-RETAINED, and it is the param_files finding reached by a different "
            "road. parse_vectors.txt / traj_vectors.txt live in the RUST crate's "
            "tests/data, which makes hashing them from the reference side look natural "
            "— but they are GENERATED by tests/crossport/gen_authoring_vectors.py and "
            "merely re-derived in Rust as the parity check. A Rust-side hash would "
            "compare the checker's own output with itself. ⚠ Under the 2026-08-17 "
            "target state (Rust is the project; Python survives as the "
            "external-software oracle and as rewrite scaffolding) these generators are "
            "scaffolding, so this key moves when they do — a successor item, not a "
            "rider on this ceremony"
        ),
    },
}


def _build_manifest() -> dict[str, object]:
    """Assemble the manifest — the reference tree's keys spliced into the checker's.

    ⚠ Since slice 8 this reads the **Rust** tree for everything :data:`_AUTHORITY` marks
    ``rust``, so it needs ``cargo``. It is reachable only from :func:`_regenerate`.
    """
    reference = _rust_reference()
    return {
        "_authority": _AUTHORITY,
        "_comment": (
            "Phase-9 Step-7 authoring freeze manifest (P9.7). Names the frozen "
            "AUTHOR-FACING platform surface: the bounded kinetics grammar + VM node/op "
            "set, the scenario file schema, the author-selectable flow-type registry, "
            "and the named param loaders. Param VALUES are delegated to "
            "docs/station-reference.manifest.json (see delegates_to). See "
            "docs/authoring-reference.md for the freeze contract + the unfreeze "
            "discipline. Hashes are newline-normalized sha-256 PROVENANCE (grammar "
            "semantics are enforced by parse_vectors.txt, the VM's arithmetic by "
            "traj_vectors.txt, and the interpreter by the crossport anchors). "
            "Each key's producer and why is in _authority: this file has MIXED "
            "authority since slice 8 of the reference flip, and 'rust' means something "
            "weaker here than on the other two manifests (there is no built registry "
            "to walk — see authoring::surface). Regenerate on a deliberate unfreeze: "
            "uv run python tests/test_authoring_freeze_manifest.py — which now shells "
            "cargo, because the _authority 'rust' keys are read from the reference "
            "tree."
        ),
        "frozen_at_phase": 9,
        "reference_doc": "docs/authoring-reference.md",
        "delegates_to": STATION_MANIFEST,
        "grammar_note": (
            "The grammar is bounded and closed, and DELIBERATELY INCOMPLETE "
            "(decision D): bare division, the rest of the function set (exp ln pow "
            "sqrt abs min max clamp), bounded conditionals and a named-constant "
            "surface are all deferred until a real frozen flow forces each semantic "
            "choice. Freezing this subset does NOT imply completeness; adding an op is "
            "a deliberate unfreeze. 'monod' (S/(S+K)) landed post-roadmap as exactly "
            "such an unfreeze — forced by the frozen "
            "biosphere.chamber.oxygen_limitation_factor, whose kernel it mirrors "
            "(including denom<=0 -> 0, which makes it total). It guards its own "
            "denominator, so it resolved x/0 INTERNALLY and bare '/' stays deferred. "
            "There is no 'dt' token by construction (RK4-order-safety is structural). "
            "Precedence + associativity are enforced by parse_vectors.txt, not "
            "recorded here."
        ),
        # ⚠ Every line below used to call the derivation directly above it. Since slice
        # 8 they come from the reference tree, and those derivations have become the
        # conformance gates further down — the checker is checked against the contract
        # instead of writing it.
        "expr_nodes": reference["expr_nodes"],
        "binary_ops": reference["binary_ops"],
        "ref_keywords": reference["ref_keywords"],
        "step_token": reference["step_token"],
        "integrator_names": reference["integrator_names"],
        "rate_classes": reference["rate_classes"],
        "schema_fields": reference["schema_fields"],
        "flow_types": reference["flow_types"],
        "param_loaders": reference["param_loaders"],
        "parity_vectors": {
            name: _normalized_sha256(VECTOR_DIR / name) for name in sorted(VECTOR_FILES)
        },
    }


def _manifest_dumps(manifest: dict[str, object]) -> str:
    """Serialize the manifest to canonical JSON — the project golden discipline.

    ``indent=2, sort_keys=True`` + a trailing newline, matching ``sim_io.dumps`` and the
    biosphere/station manifests, so it reads and diffs like every other committed
    snapshot.
    """
    return json.dumps(manifest, indent=2, sort_keys=True) + "\n"


def _load_manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


# --- the completeness gate (what the vectors + anchors are blind to) ---------


def test_frozen_grammar_node_set_is_complete() -> None:
    # The manifest's AST node set equals the live closed `Expr` union. Catches a grammar
    # node added but exercised by no parse/traj vector — the whole point of the gate.
    manifest = _load_manifest()
    assert set(manifest["expr_nodes"]) == set(_expr_nodes())


def test_frozen_operator_and_token_sets_are_complete() -> None:
    # The op set + the closed identifier set (the three ref keywords) + the legal
    # integrator names + the legal rate classes are the rest of the author-visible
    # vocabulary. Deriving each from its single source of truth is what makes "someone
    # added `/`" fail here.
    #
    # `rate_classes` is the multi-rate analogue of `integrator_names`, and it exists
    # because `schema_fields` records that `rate` EXISTS but not what it may say — the
    # same gap `integrator_names` fills for `integrator`. Unlike the flow-type registry
    # (explicitly expected to grow), this one is closed at two by `multirate_step`'s own
    # signature: it takes exactly two `Substepper`s, so a third class cannot be added
    # without a `simcore` change.
    manifest = _load_manifest()
    assert set(manifest["binary_ops"]) == set(_BINARY_OPS)
    assert set(manifest["ref_keywords"]) == set(_REF_KEYWORDS)
    assert set(manifest["integrator_names"]) == set(_INTEGRATORS)
    assert set(manifest["rate_classes"]) == set(_RATE_CLASSES)


def test_frozen_schema_surface_is_complete() -> None:
    # Every spec model and every field on it is frozen. `extra="forbid"` makes the field
    # set exactly the legal key set of an authored file, so this IS the file grammar.
    manifest = _load_manifest()
    assert manifest["schema_fields"] == _schema_fields()


def test_frozen_flow_type_registry_is_complete() -> None:
    # The author-selectable frozen-flow surface, incl. each type's wiring field set and
    # param set. Catches a flow type added (the registry is explicitly expected to grow)
    # or a wiring field renamed — either silently breaks committed scenario files.
    manifest = _load_manifest()
    assert manifest["flow_types"] == _flow_types()


def test_the_manifest_actually_records_the_rate_params() -> None:
    # Teeth for a hole the equality gate STRUCTURALLY cannot see, and it nearly shipped
    # (advisor catch, multi-rate Step 5). `test_frozen_flow_type_registry_is_complete`
    # compares the manifest against `_flow_types()` — so if `_flow_types()` omitted
    # `rate_params`, BOTH sides would omit it, the gate would stay GREEN, and the field
    # would never be frozen at all despite sitting on `FlowTypeSpec` in the live tree.
    # An equality gate cannot detect a field absent from both of the things it equates.
    # This test names the value from outside that derivation, so the omission goes red.
    #
    # `eclss.o2_makeup` is the row that matters most: its `o2_makeup_gain` is the ONE
    # rate the run-time backstop can never catch (demand-controlled — the draw tracks
    # the setpoint error, not the stock), so if this entry silently emptied, the hazard
    # would be unguarded at BOTH build and run time. `o2_setpoint` must never join it:
    # it is an inventory, not a rate.
    flow_types = _load_manifest()["flow_types"]
    assert flow_types["eclss.o2_makeup"]["rate_params"] == ["o2_makeup_gain"]
    assert flow_types["eclss.co2_scrubber"]["rate_params"] == ["co2_scrub_rate"]
    assert flow_types["eclss.condenser"]["rate_params"] == ["condense_rate"]
    assert flow_types["power.self_discharge"]["rate_params"] == ["self_discharge_rate"]
    # The uncheckable shapes are recorded as EMPTY, not absent — documented, not faked.
    # radiator_reject's constraint is tau >> dt ("≫" is not a predicate);
    # crew_metabolism's is state-dependent (forced draw < stock), which a build check
    # cannot decide.
    assert flow_types["thermal.radiator_reject"]["rate_params"] == []
    assert flow_types["eclss.crew_metabolism"]["rate_params"] == []


def test_frozen_param_loader_set_is_complete() -> None:
    # The named loaders a scenario's `params:` (or a kinetics rate's `param("…")`) may
    # reach. A pack still flows through the frozen loader, so this set bounds which
    # frozen bounds/unit guards an authored file can invoke.
    manifest = _load_manifest()
    assert set(manifest["param_loaders"]) == set(PARAM_LOADERS)


def test_completeness_gate_detects_an_unfrozen_flow_type(monkeypatch) -> None:
    # Teeth (flow registry): the gate is plain equality, so an unregistered-but-live
    # flow type must break it. Seed a phantom into a COPY of the live registry and
    # confirm the derivation no longer matches. The real registry is untouched.
    frozen = _load_manifest()["flow_types"]
    phantom = dict(FLOW_TYPES)
    phantom["crew.phantom"] = next(iter(FLOW_TYPES.values()))
    monkeypatch.setattr(sys.modules[__name__], "FLOW_TYPES", phantom)
    assert _flow_types() != frozen  # the phantom is detected — teeth


def test_completeness_gate_detects_an_unfrozen_spec_model(monkeypatch) -> None:
    # Teeth (schema): a whole new spec model added to `authoring.schema` must break the
    # gate, not just a new field on a known model. The module-scan derivation is what
    # gives that; monkeypatch proves it fires. The real module is restored after.
    frozen = _load_manifest()["schema_fields"]

    class PhantomSpec(BaseModel):
        ghost: float

    # The scan filters on `__module__`, so the phantom must claim to live there.
    monkeypatch.setattr(PhantomSpec, "__module__", authoring.schema.__name__)
    monkeypatch.setattr(authoring.schema, "PhantomSpec", PhantomSpec, raising=False)
    assert _schema_fields() != frozen  # the phantom is detected — teeth


def test_manifest_delegates_param_values_to_the_station() -> None:
    # An authored file reaches frozen param VALUES through the named loaders; those
    # files are frozen by the station manifest. Pin the delegation pointer (and that it
    # exists) so this manifest never grows a duplicate, drifting copy of those hashes.
    manifest = _load_manifest()
    assert manifest["delegates_to"] == STATION_MANIFEST
    assert (_REPO_ROOT / manifest["delegates_to"]).is_file()


def test_manifest_named_files_exist() -> None:
    # Every artifact the manifest names is present on disk — a renamed/deleted vector
    # file or reference doc fails here, not as a mystery load error later.
    manifest = _load_manifest()
    for name in manifest["parity_vectors"]:
        assert (VECTOR_DIR / name).is_file(), name
    assert (_REPO_ROOT / manifest["reference_doc"]).is_file()


def test_manifest_records_the_grammar_is_incomplete() -> None:
    # The one thing a reader must not misread: freezing the arithmetic core does NOT
    # freeze a COMPLETE grammar. Division / the function set / named constants are
    # deferred by decision, each pending a real flow that forces its semantics. Pin that
    # the manifest says so, so a future op lands as a deliberate unfreeze.
    manifest = _load_manifest()
    assert "DELIBERATELY INCOMPLETE" in manifest["grammar_note"]
    assert set(manifest["binary_ops"]) == {"+", "-", "*"}  # no "/" — deferred


def _leaf_paths(node: Any, prefix: str = "") -> list[str]:
    """Every leaf path of the manifest, ``/``-joined.

    Dicts recurse; anything else (including a list) is a leaf, because the frozen units
    here are whole lists — ``expr_nodes`` is one claim, not eight. ``/`` rather than
    ``.`` because flow-type keys *are* dotted names (``crew.food_metabolism``).
    """
    if isinstance(node, dict):
        return [p for k, v in node.items() for p in _leaf_paths(v, f"{prefix}{k}/")]
    return [prefix.rstrip("/")]


def _authority_matches(path: str) -> list[tuple[int, str, dict[str, str]]]:
    """Every ``_AUTHORITY`` pattern matching ``path``, with its specificity score."""
    segments = path.split("/")
    matches = []
    for pattern, entry in _AUTHORITY.items():
        parts = pattern.split("/")
        if len(parts) != len(segments):
            continue
        if any(p not in ("*", s) for p, s in zip(parts, segments, strict=True)):
            continue
        matches.append((sum(p != "*" for p in parts), pattern, entry))
    return matches


def _authority_for(path: str) -> tuple[str, dict[str, str]] | None:
    """Resolve a leaf path against :data:`_AUTHORITY`, most specific wins."""
    matches = _authority_matches(path)
    if not matches:
        return None
    _score, pattern, entry = max(matches, key=lambda m: m[0])
    return pattern, entry


def test_every_frozen_field_declares_who_produced_it() -> None:
    """The manifest states its own mixed authority, and the block cannot go stale.

    ⚠ **Checked in both directions**, because each fails differently: an unclassified
    field is a frozen value whose producer nobody stated (the thing slice 8 exists to
    prevent), while a pattern matching nothing is a stale row describing a field that
    has been renamed or removed — which reads as coverage and is not.
    """
    manifest = _load_manifest()
    classified = {k: v for k, v in manifest.items() if k != "_authority"}
    paths = _leaf_paths(classified)
    unclassified = [p for p in paths if _authority_for(p) is None]
    assert not unclassified, (
        f"frozen fields with no _authority entry: {unclassified}. Every field of this "
        "contract has to say which side produces it — see _AUTHORITY in "
        "tests/test_authoring_freeze_manifest.py."
    )

    # ⚠ "Most specific wins" only decides anything while no two patterns TIE. Two of
    # equal specificity matching one path would resolve by dict order — a silent answer
    # to a question nobody asked, and the field would read as classified either way.
    for path in paths:
        top = max(s for s, _, _ in _authority_matches(path))
        tied = sorted(p for s, p, _ in _authority_matches(path) if s == top)
        assert len(tied) == 1, (
            f"{path} is matched by {len(tied)} _authority patterns of equal "
            f"specificity: {tied}. Which one applies would be decided by dict order — "
            "make one of them strictly more specific."
        )

    matched = {_authority_for(p)[0] for p in paths}  # type: ignore[index]
    stale = sorted(set(manifest["_authority"]) - matched)
    assert not stale, f"_authority patterns matching no field: {stale}"
    assert manifest["_authority"] == _AUTHORITY, (
        "the committed _authority block is not the one this module would write — "
        "regenerate the manifest"
    )


def test_the_reference_side_keys_are_exactly_what_the_generator_splices() -> None:
    """The ``rust`` classification and the splice cannot drift apart.

    ⚠ Two independent things say which keys come from the reference: the
    :data:`_RUST_DUMP_KEYS` forcing function (what regeneration will accept from the
    dump) and :data:`_AUTHORITY` (what the committed file *claims*). If they disagree,
    one of them is a lie — either the manifest advertises a Rust-derived field that
    regeneration writes from Python, or a spliced field goes unclassified. Neither is
    visible from a green regeneration, because regeneration writes both.

    Top-level only: ``schema_fields`` and ``flow_types`` are classified per leaf but
    spliced whole, so the comparison is over the first path segment.
    """
    rust_keys = {
        pattern.split("/")[0]
        for pattern, entry in _AUTHORITY.items()
        if entry["side"] == "rust"
    }
    assert rust_keys == set(_RUST_DUMP_KEYS), (
        "the keys _AUTHORITY calls 'rust' are not the keys the generator splices:\n"
        f"  classified rust: {sorted(rust_keys)}\n"
        f"  spliced:         {sorted(_RUST_DUMP_KEYS)}\n"
        "One of the two is wrong. _RUST_DUMP_KEYS is what regeneration enforces "
        "against the dump; _AUTHORITY is what the frozen file tells a reader."
    )


def test_the_python_derivations_are_conformance_checks_now() -> None:
    """⚠ What the gates above mean changed in slice 8, and it is stated, not implied.

    Before the flip ``manifest["expr_nodes"] == _expr_nodes()`` asserted the manifest
    was a faithful record of the Python package — Python *wrote* both sides. Now the
    manifest is written from Rust, so the identical assertion asks the opposite
    question: **has the checker drifted from the contract the reference defines?** The
    assertion is unchanged and its meaning is inverted, which is exactly the kind of
    silent reversal this repo has been caught by
    (`o2-makeup-reversal-inside-the-freeze`).

    This test pins the direction so the reading cannot be lost: every axis the reference
    produces has a Python derivation that must still agree, and the set of such axes is
    the spliced set. A future key spliced from Rust with no Python derivation beside it
    would leave the checker silently unchecked on that axis — allowed, but it must be a
    decision, and this makes it show up as a failure first.
    """
    derived_here = {
        "expr_nodes": _expr_nodes(),
        "binary_ops": sorted(_BINARY_OPS),
        "ref_keywords": sorted(_REF_KEYWORDS),
        "step_token": "n",
        "integrator_names": sorted(_INTEGRATORS),
        "rate_classes": sorted(_RATE_CLASSES),
        "schema_fields": _schema_fields(),
        "flow_types": _flow_types(),
        "param_loaders": sorted(PARAM_LOADERS),
    }
    assert set(derived_here) == set(_RUST_DUMP_KEYS), (
        "an axis is spliced from the reference with no Python derivation beside it (or "
        "vice versa). The checker would be unchecked on that axis — decide it "
        "deliberately and say so in _AUTHORITY."
    )
    manifest = _load_manifest()
    for key, value in derived_here.items():
        assert manifest[key] == value, (
            f"the checker's {key} has drifted from the frozen contract. ⚠ Since slice "
            f"8 this manifest is generated from the RUST tree, so the first question "
            f"is 'what changed in src/authoring?', not 'is the manifest stale?'. If "
            f"the reference moved, that is an unfreeze (docs/authoring-reference.md) "
            f"and the ceremony ends in `uv run python "
            f"tests/test_authoring_freeze_manifest.py`, which reads Rust."
        )


def _regenerate() -> None:
    """Rewrite the committed authoring manifest from the current live tree.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_authoring_freeze_manifest.py

    Review the diff before committing: a change means the frozen authoring surface moved
    (a new grammar node/op, a schema field, a flow type, a param loader), i.e. an
    **unfreeze**, which the discipline in docs/authoring-reference.md governs. Written
    via ``write_bytes`` (explicit LF, like the goldens) so the manifest is byte-stable
    across platforms.
    """
    MANIFEST_PATH.write_bytes(_manifest_dumps(_build_manifest()).encode("utf-8"))
    print(f"wrote {MANIFEST_PATH}")


if __name__ == "__main__":
    _regenerate()
