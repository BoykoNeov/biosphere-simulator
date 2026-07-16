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

**Cross-port boundary, stated honestly.** The manifest freezes the **Python** surface of
record. The Rust mirror (``rust/crates/authoring``) is gated by the parse/traj vectors +
the anchors, not by this gate — a Python schema field added with no Rust mirror is
caught only once an anchor exercises it. That is the same division as everywhere else:
parity surfaces own cross-port fidelity, this gate owns single-port completeness.

Regeneration is a deliberate, separate ``__main__`` action (the golden discipline): on
an advisor-reviewed unfreeze, run ``uv run python
tests/test_authoring_freeze_manifest.py`` and review the manifest diff. Zero ``simcore``
change, zero ``domains`` change (docs + tests only).
"""

from __future__ import annotations

import hashlib
import inspect
import json
import sys
from pathlib import Path
from typing import Any, get_args

from pydantic import BaseModel

import authoring.schema
import simcore.expr
from authoring.expr_parser import _REF_KEYWORDS
from authoring.flow_registry import FLOW_TYPES, PARAM_LOADERS
from authoring.run import _INTEGRATORS
from simcore.expr import _BINARY_OPS

_REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = _REPO_ROOT / "docs" / "authoring-reference.manifest.json"

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
    interpreter demands, and the named param set (or ``None`` for a param-free flow).
    The wiring names are part of the contract an author writes against, not an
    implementation detail — a renamed wiring field breaks every file that names it.
    """
    return {
        name: {
            "cls": spec.cls.__name__,
            "wiring_fields": sorted(spec.wiring_fields),
            "param_set": spec.param_set,
        }
        for name, spec in FLOW_TYPES.items()
    }


def _build_manifest() -> dict[str, object]:
    """Assemble the manifest from the live tree — the single source for regeneration."""
    return {
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
            "Regenerate on a deliberate unfreeze: "
            "uv run python tests/test_authoring_freeze_manifest.py."
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
        "expr_nodes": _expr_nodes(),
        "binary_ops": sorted(_BINARY_OPS),
        "ref_keywords": sorted(_REF_KEYWORDS),
        "step_token": "n",
        "integrator_names": sorted(_INTEGRATORS),
        "schema_fields": _schema_fields(),
        "flow_types": _flow_types(),
        "param_loaders": sorted(PARAM_LOADERS),
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
    # integrator names are the rest of the author-visible vocabulary. Deriving each from
    # its single source of truth is what makes "someone added `/`" fail here.
    manifest = _load_manifest()
    assert set(manifest["binary_ops"]) == set(_BINARY_OPS)
    assert set(manifest["ref_keywords"]) == set(_REF_KEYWORDS)
    assert set(manifest["integrator_names"]) == set(_INTEGRATORS)


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
