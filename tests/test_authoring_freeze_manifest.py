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
the Rust tree**, and the derivations below have become **conformance checks on the
checker**: they still run, still derive from the live Python package, and now answer
*"has Python drifted from the frozen contract?"* rather than *"what is the contract?"*.

**⚠⚠ SLICE C7 TOOK THE REST: THIS MODULE HAS NO WRITER.** Slice 8 left the contract
*authored* by the reference and *written* here — ``_build_manifest`` shelled the dump,
spliced its keys into the checker's own, serialized and wrote the file. Since 2026-08-18
the reference writes it (``cd rust && cargo run --example dump_authoring_inventory --
--write-manifest``), and everything the writer needed left with it: the ``_AUTHORITY``
literal, the prose, the vector-file roster, the splice-key set and the ``__main__``.
What is left here is a **checker**, and two consequences follow that a reader must not
miss:

* the gates below now read ``_authority`` **out of the committed file** instead of
  comparing it to a module-level copy. A copy kept purely to assert against would be the
  stale second source this repo keeps being bitten by;
* the manifest is a **generated artifact** again, so a hand edit to it is red —
  ``tests/crossport/test_manifest_writer.py`` regenerates and compares bytes. Before C7
  a typo in ``_comment`` or a hand-patched hash simply stood.

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

Regeneration is a deliberate, separate action (the golden discipline): on an
advisor-reviewed unfreeze, run ``cd rust && cargo run --example
dump_authoring_inventory -- --write-manifest`` and review the manifest diff. The tests
here do not shell cargo, and the base suite stays offline-clean; the cargo-side gates
live in ``tests/crossport/`` (``test_inventory_parity.py`` for staleness of the derived
axes, ``test_manifest_writer.py`` for the whole file byte for byte).
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
from authoring.interpreter import _RATE_CLASSES
from authoring.run import _INTEGRATORS
from simcore.expr import _BINARY_OPS

_REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = _REPO_ROOT / "docs" / "authoring-reference.manifest.json"

# The two committed cross-port vector files whose provenance hashes the manifest
# records. ⚠ The ROSTER is not here any more — the reference writes it (slice C7,
# `rust/crates/authoring/examples/dump_authoring_inventory.rs::VECTOR_FILES`), and a
# copy kept here would be the stale second source. What this module owns instead is the
# tie: :func:`test_the_frozen_vector_roster_is_the_generators` reads the roster out of
# the committed file and checks it against the paths the generator actually writes.
VECTOR_DIR = _REPO_ROOT / "rust" / "crates" / "authoring" / "tests" / "data"


def _normalized_sha256(path: Path) -> str:
    """sha-256 over newline-normalized (LF) content — a reproducible provenance hash.

    Hashing raw bytes would make the value depend on the checkout's line endings
    (``autocrlf`` on Windows vs. LF on Linux). Normalizing to LF first makes the hash a
    stable record of *content*.

    ⚠ **Its standing changed in slice C7 and it is worth stating.** Until then this was
    the *writer's* hashing rule and nothing compared its output to anything — the two
    ``parity_vectors`` digits were provenance that no test ever recomputed. The
    reference writes them now, under the narrower rule in ``config::provenance``, so
    this function has become the checker's independent second opinion: the roster gate
    below recomputes both hashes here and demands agreement. That is the same
    two-rules-held-equal tie slice C8 established for ``param_files``, reached on this
    contract by C7 rather than by C8.
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
    """An authored file reaches frozen param VALUES through the named loaders, and those
    files are frozen elsewhere — so this contract must point at a manifest that really
    freezes them, and must never grow a duplicate, drifting copy of those hashes.

    ⚠ **What this asserts changed in slice C7.** It used to compare ``delegates_to``
    against ``STATION_MANIFEST``, a path literal in this module — and once the reference
    writes the pointer, that is a Python literal checked against a Rust literal, which
    is the duplicate-asserted-against-its-own-duplicate shape the biosphere half
    deleted.
    What is checkable from the checker's side is the property the pointer exists FOR:
    the target is a real manifest, and it carries the ``param_files`` census this one
    declines to re-hash. A pointer at the wrong file, or at a manifest that froze no
    params, now fails; a renamed station manifest fails too, where the literal
    comparison would merely have to be updated in two places.
    """
    manifest = _load_manifest()
    target = _REPO_ROOT / manifest["delegates_to"]
    assert target.is_file(), manifest["delegates_to"]
    delegate = json.loads(target.read_text(encoding="utf-8"))
    assert delegate.get("param_files"), (
        f"{manifest['delegates_to']} freezes no param_files, so delegating param "
        "VALUES to it is a promise nothing keeps"
    )
    # ⚠ The delegation is a promise about SPECIFIC VALUES, so check they resolve rather
    # than merely that the target froze something. The file behind each loader is read
    # off the loader itself — its default path — and looked up in the delegate's census.
    # A loader with no frozen file behind it is an authored scenario reaching values
    # nothing freezes, which is what delegating instead of re-hashing is meant to buy.
    #
    # ⚠⚠ DERIVED, and the first draft was not. It matched `<loader>.yaml` by naming
    # convention, which is a rule this repo never adopted: `thermal` loads
    # `radiator.yaml`, so the convention reported a real gap that does not exist. A
    # convention invented at the gate is not a property of the tree — ask the loader.
    unresolved = {}
    for loader in manifest["param_loaders"]:
        default = next(
            iter(inspect.signature(PARAM_LOADERS[loader]).parameters.values())
        ).default
        if Path(default).name not in delegate["param_files"]:
            unresolved[loader] = Path(default).name
    assert not unresolved, (
        f"param loaders whose file {manifest['delegates_to']} does not freeze: "
        f"{unresolved}. Either the loader reads a param file that manifest does not "
        "census, or the file was renamed on one side only — an authored scenario would "
        "be reaching values nothing freezes."
    )


def test_the_frozen_vector_roster_is_the_generators(monkeypatch) -> None:
    """The two hashed vector files are the two the generator writes, and the hashes are
    of those files.

    ⚠⚠ **New in slice C7, and it closes a hole the slice opened.** The roster — which
    files ``parity_vectors`` records — lived in this module *and was written from it*,
    so it needed no gate: the manifest could not disagree with its own source. Moving
    the writer to the reference turned one source into two copies with nothing holding
    them together, and the copy at risk is exactly the one ``_authority`` marks
    ``python``:
    a file dropped from the reference's ``VECTOR_FILES`` would simply stop being hashed,
    and every other gate would stay green.

    ⚠ The tie is to ``gen_authoring_vectors``' own output paths rather than to a list
    retyped here, because that generator is what *produces* these files — it is the
    roster's single source of truth, the same way ``golden_platform.RUST_AUTHORED`` is
    for the biosphere goldens.

    ⚠ The value half never existed before this slice. The two digits were provenance
    that nothing recomputed; now the reference writes them under
    ``config::provenance``'s narrow newline rule and this recomputes them under
    Python's broader ``splitlines`` one, which is the same two-rules-held-equal tie
    slice C8 built for ``param_files``.

    The generator is imported inside the test so it is not a collection-time cost for
    the rest of the base suite; it pulls in ``authoring`` and ``simcore`` only — no
    ``cargo``, so this stays offline-clean. ⚠ Via ``monkeypatch.syspath_prepend`` rather
    than a bare ``sys.path.insert``, which would outlive the test: ``tests/crossport/``
    holds generically-named modules (``compare``, ``authoring_files``) and leaving it at
    position 0 would let a later plain import in the same worker resolve there.
    """
    monkeypatch.syspath_prepend(_REPO_ROOT / "tests" / "crossport")
    import gen_authoring_vectors  # noqa: PLC0415

    manifest = _load_manifest()
    written = {
        gen_authoring_vectors.PARSE_PATH.name: gen_authoring_vectors.PARSE_PATH,
        gen_authoring_vectors.TRAJ_PATH.name: gen_authoring_vectors.TRAJ_PATH,
    }
    assert set(manifest["parity_vectors"]) == set(written), (
        "the frozen parity-vector roster is not the set gen_authoring_vectors writes:\n"
        f"  manifest:  {sorted(manifest['parity_vectors'])}\n"
        f"  generator: {sorted(written)}\n"
        "The roster lives in "
        "rust/crates/authoring/examples/dump_authoring_inventory.rs (VECTOR_FILES) "
        "since slice C7. A file added to the generator has to be "
        "classified and hashed; one dropped from the writer stops being frozen "
        "silently."
    )
    for name, path in written.items():
        assert manifest["parity_vectors"][name] == _normalized_sha256(path), (
            f"the frozen {name} hash is not this file's. Either the vectors were "
            "regenerated without regenerating the manifest (`cd rust && cargo run "
            "--example dump_authoring_inventory -- --write-manifest`), or the two "
            "newline-normalization rules have stopped agreeing — see "
            "config::provenance."
        )


def test_manifest_named_files_exist() -> None:
    # Every artifact the manifest names is present on disk — a renamed/deleted vector
    # file or reference doc fails here, not as a mystery load error later.
    manifest = _load_manifest()
    for name in manifest["parity_vectors"]:
        assert (VECTOR_DIR / name).is_file(), name
    assert (_REPO_ROOT / manifest["reference_doc"]).is_file()


def test_manifest_records_the_grammar_is_incomplete() -> None:
    # ⚠ Since slice C7 this prose is written by the REFERENCE, so the assertion is
    # unchanged and its meaning inverted, the same way the derivation gates below were
    # inverted by slice 8: it asks whether the contract the reference writes still says
    # this, not whether the checker's copy does.
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


def _authority_matches(
    path: str, authority: dict[str, dict[str, str]]
) -> list[tuple[int, str, dict[str, str]]]:
    """Every ``_authority`` pattern matching ``path``, with its specificity score.

    ⚠ Takes the block as an argument since slice C7. It used to read a module-level
    ``_AUTHORITY`` literal that the writer here spliced into the manifest and a test
    then compared back — with the writer gone there is no second copy, and a checker
    that kept one would be asserting a duplicate against the file it duplicates.
    """
    segments = path.split("/")
    matches = []
    for pattern, entry in authority.items():
        parts = pattern.split("/")
        if len(parts) != len(segments):
            continue
        if any(p not in ("*", s) for p, s in zip(parts, segments, strict=True)):
            continue
        matches.append((sum(p != "*" for p in parts), pattern, entry))
    return matches


def _authority_for(
    path: str, authority: dict[str, dict[str, str]]
) -> tuple[str, dict[str, str]] | None:
    """Resolve a leaf path against the manifest's ``_authority``, most specific wins."""
    matches = _authority_matches(path, authority)
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
    authority = manifest["_authority"]
    classified = {k: v for k, v in manifest.items() if k != "_authority"}
    paths = _leaf_paths(classified)
    unclassified = [p for p in paths if _authority_for(p, authority) is None]
    assert not unclassified, (
        f"frozen fields with no _authority entry: {unclassified}. Every field of this "
        "contract has to say which side produces it — see the AUTHORITY table in "
        "rust/crates/authoring/examples/dump_authoring_inventory.rs, which writes it."
    )

    # ⚠ "Most specific wins" only decides anything while no two patterns TIE. Two of
    # equal specificity matching one path would resolve by dict order — a silent answer
    # to a question nobody asked, and the field would read as classified either way.
    for path in paths:
        top = max(s for s, _, _ in _authority_matches(path, authority))
        tied = sorted(p for s, p, _ in _authority_matches(path, authority) if s == top)
        assert len(tied) == 1, (
            f"{path} is matched by {len(tied)} _authority patterns of equal "
            f"specificity: {tied}. Which one applies would be decided by dict order — "
            "make one of them strictly more specific."
        )

    matched = {_authority_for(p, authority)[0] for p in paths}  # type: ignore[index]
    stale = sorted(set(authority) - matched)
    assert not stale, f"_authority patterns matching no field: {stale}"

    # ⚠ What replaced the fourth check, and why it is not a weakening. Until slice C7
    # this ended with ``manifest["_authority"] == _AUTHORITY`` — the committed block
    # against this module's own literal. C7 deleted the literal with the rest of the
    # writer, and keeping a copy purely to assert against would be the stale second copy
    # this repo has been bitten by. What is checkable from here is the block's SHAPE,
    # and a malformed row is a failure the equality never caught either: a row is
    # ``{side, why}``, ``side`` is one of the three the contract defines, and ``why`` is
    # prose someone wrote rather than an empty string standing in for a reason.
    for pattern, entry in sorted(authority.items()):
        assert set(entry) == {"side", "why"}, (pattern, sorted(entry))
        assert entry["side"] in {"rust", "python", "hand"}, (pattern, entry["side"])
        assert len(entry["why"]) > 10, (pattern, entry["why"])


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
    produces has a Python derivation that must still agree. A future Rust-written key
    with no Python derivation beside it would leave the checker silently unchecked on
    that axis — allowed, but it must be a decision, and this makes it show up as a
    failure first.

    ⚠ **The roster it compares against moved in slice C7, and this test absorbed a
    second one.** It used to read ``_RUST_DUMP_KEYS``, the module's own record of what
    regeneration would splice; the writer owns that now, so the honest anchor is what
    the committed contract *claims* — the keys its ``_authority`` block marks ``rust``.
    ``test_the_reference_side_keys_are_exactly_what_the_generator_splices`` was retired
    with the writer for the same reason: it tied those two module-level literals to each
    other, and neither exists here any more. The check it performed — that the
    classification and the reference-written set are the same set — is the first
    assertion below, now stated against the file instead of against a copy of it.
    """
    manifest = _load_manifest()
    classified_rust = {
        pattern.split("/")[0]
        for pattern, entry in manifest["_authority"].items()
        if entry["side"] == "rust"
    }
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
    assert set(derived_here) == classified_rust, (
        "an axis the contract calls reference-produced has no Python derivation beside "
        "it (or vice versa):\n"
        f"  classified rust: {sorted(classified_rust)}\n"
        f"  derived here:    {sorted(derived_here)}\n"
        "The checker would be unchecked on that axis — decide it deliberately and say "
        "so in the AUTHORITY table in "
        "rust/crates/authoring/examples/dump_authoring_inventory.rs."
    )
    for key, value in derived_here.items():
        assert manifest[key] == value, (
            f"the checker's {key} has drifted from the frozen contract. ⚠ Since slice "
            f"8 this manifest is generated from the RUST tree, so the first question "
            f"is 'what changed in rust/crates/authoring?', not 'is the manifest "
            f"stale?'. If the reference moved, that is an unfreeze "
            f"(docs/authoring-reference.md) and the ceremony ends in `cd rust && cargo "
            f"run --example dump_authoring_inventory -- --write-manifest`."
        )
