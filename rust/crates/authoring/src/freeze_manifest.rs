//! Dump the **authoring platform's** half of its freeze manifest as JSON, and — since
//! slice C7 — **write the whole contract**. Reference flip
//! (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! The counterpart of `dump_biosphere_inventory` / `dump_station_inventory`, and it landed
//! in one step rather than two: those two were built in slice 3 as *witnesses* (a Python
//! gate checked them against a Python-generated manifest) and promoted to *producers* in
//! slices 6 and 7. Slice 8 deliberately made this program a producer from its first
//! commit, and C7 finishes the job: `docs/authoring-reference.manifest.json` is written
//! here, by `--write-manifest`, instead of by
//! `tests/test_authoring_freeze_manifest.py::_build_manifest()`. That module keeps its
//! ~15 completeness and conformance gates and has no writer.
//!
//! ⚠⚠ **Read `authoring::surface`'s module docs before trusting the word "derived" here.**
//! The other two dumps enumerate a **built registry**: they ask a canonical scenario which
//! flows it actually wired. This contract freezes the *platform*, which has no such
//! runtime object, and Python's derivation of it uses language introspection Rust does not
//! have (`typing.get_args`, a scan of `vars(module)`, pydantic `model_fields`, a dict).
//! Every axis below is therefore part derived and part hand-maintained roster, and which
//! is which is stated per key in the manifest's `_authority` block. **A green run of this
//! program is not the same evidence a green `dump_station_inventory` is.**
//!
//! ## What is emitted
//!
//! * `expr_nodes` / `binary_ops` — the grammar's node and operator vocabulary. Names come
//!   from an exhaustive `match` (a new `Expr` variant is a compile error) and symbols from
//!   `BinaryOp::symbol()`; the *rosters* of variants are hand-maintained.
//! * `ref_keywords` / `step_token` — the parser's own tables, load-bearing: these are the
//!   values `expr_parser` tests an identifier against, so changing one changes what the
//!   platform parses.
//! * `schema_fields` — the eight `reject_unknown_keys` key sets, likewise load-bearing.
//!   The spec *labels* are Python class names typed by hand (two of them name no Rust type
//!   at all).
//! * `flow_types` / `param_loaders` — the author-selectable registry. Each entry is read
//!   off its `FlowTypeSpec`, and `cls` off a flow this program **constructs** through
//!   `build_frozen_flow`, so it is the class the registry would really instantiate.
//! * `integrator_names` / `rate_classes` — the legal `integrator:` and `rate_class:`
//!   vocabularies.
//!
//! Those nine keys are [`reference_keys`], and **one derivation feeds both halves of this
//! program** — the dump and the writer. C7's biosphere half caught its own first draft
//! re-walking the registries for the writer, which puts two derivations of the same sets
//! in one file with nothing tying them; sharing makes the drift impossible instead of
//! merely detectable.
//!
//! ## What the writer adds on top, and where each piece comes from
//!
//! * **Prose, labels and pointers** — `_comment`, `frozen_at_phase`, `reference_doc`,
//!   `delegates_to`, `grammar_note`, and the `_authority` block itself. Neither port
//!   derives them and neither should; `grammar_note` in particular records *decisions*
//!   (which operators are deferred and why), not state. They moved here as frozen text,
//!   generated from the committed manifest and diffed rather than retyped.
//! * **`parity_vectors`** — the newline-normalized sha-256 of `parse_vectors.txt` and
//!   `traj_vectors.txt`. ⚠⚠ **This program hashes them and they stay classified
//!   `python`**, and the row that says so was corrected in this slice: it used to argue
//!   that "a Rust-side hash would compare the checker's own output with itself", which
//!   conflates who produces a value with who digests it. The biosphere contract has read
//!   `scenarios/*/golden_sha256` as `rust` since slice 4 while *Python* computed that
//!   digest, for the mirror-image reason. The value here is the Python generator's
//!   (`tests/crossport/gen_authoring_vectors.py`, still live and still gated by
//!   `tests/crossport/test_crossport.py`), so `python` is who it names; the key moves when
//!   that generator does.
//!
//! ## ⚠ The trap the biosphere half set, measured for and absent here
//!
//! There, moving the writer into the crate that owns `BIO_DT` put the frozen `dt_days`
//! literal one character away from auto-following the code — and a spliced constant
//! produces a **byte-identical** manifest, so the regeneration diff cannot see it. This
//! contract has no such pair: its hand keys are a phase number, two repo paths and two
//! blocks of prose, and this crate owns no constant any of them could be spliced from
//! (`step_token` and the integrator/rate-class vocabularies are already `rust` and
//! *should* follow the code). Measured, and recorded as "none" rather than answered with
//! an invented guard — a control with no test to redden is the finding.

// ⚠⚠ **RELOCATED from `examples/dump_authoring_inventory.rs` by Stage-3 slice S2** (the reference flip,
// plan §5u). It moved for one structural reason: an `examples/` program is a **binary
// target**, so no integration test can call into it — which is why the byte-for-byte gate
// on the committed manifest was a *Python* program shelling out to `cargo run`, and why
// retiring the checker would have taken the gate with it (FINDING 2's first entry).
//
// The move is deliberately a **relocation, not a rewrite**: the code below is the example's
// verbatim, so the emitted manifest bytes cannot shift. The example keeps only its argument
// parsing and calls in here. `tests/manifest_writer.rs` now `include_str!`s THIS file for
// its anti-derived-literal greps, and compares `manifest_text()` against the committed
// contract.

/// The manifest exactly as it is serialized to disk — the byte gate's subject.
///
/// ⚠ One serialization, three callers: [`write_manifest`] writes it, the byte gate compares
/// it, and nothing re-derives it. C7's own lesson on the inventory walk applies again here —
/// *sharing makes the drift impossible instead of merely detectable*.
pub fn manifest_text() -> String {
    dumps(&manifest())
}
use crate::surface;
use config::canonical_json::{dumps, Json};
use config::provenance::{contains_exotic_line_separator, normalized_sha256};
use std::path::{Path, PathBuf};

/// The two committed cross-port vector files whose provenance hashes the manifest records.
///
/// ⚠ A hand roster, and it has to be: the parity harness reads these through
/// `include_str!` (`crates/authoring/tests/authoring_vectors.rs`), so the reference knows
/// their **bytes** and not their **names** — C9's `weather_fixture` finding in a second
/// place. What keeps the roster honest is on the checker's side, where the files are
/// produced: `test_the_frozen_vector_roster_is_the_generators` ties this set to
/// `gen_authoring_vectors`' own output paths, so a file dropped from here is red rather
/// than quietly unhashed.
const VECTOR_FILES: [&str; 2] = ["parse_vectors.txt", "traj_vectors.txt"];

const COMMENT: &str = "Phase-9 Step-7 authoring freeze manifest (P9.7). Names the frozen AUTHOR-FACING platform surface: the bounded kinetics grammar + VM node/op set, the scenario file schema, the author-selectable flow-type registry, and the named param loaders. Param VALUES are delegated to docs/station-reference.manifest.json (see delegates_to). See docs/authoring-reference.md for the freeze contract + the unfreeze discipline. Hashes are newline-normalized sha-256 PROVENANCE (grammar semantics are enforced by parse_vectors.txt, the VM's arithmetic by traj_vectors.txt, and the interpreter by the crossport anchors). Each key's producer and why is in _authority: this file has MIXED authority since slice 8 of the reference flip, and 'rust' means something weaker here than on the other two manifests (there is no built registry to walk — see authoring::surface). Regenerate on a deliberate unfreeze, from rust/: cargo run --example dump_authoring_inventory -- --write-manifest. Slice C7 moved the WRITER to the reference; tests/test_authoring_freeze_manifest.py has none and is now only a checker.";

/// The `_authority` block, as `(manifest path, side, why)`.
///
/// ⚠ `side` is who produced the **value** — not who hashed it and not who wrote the
/// file. A `python` row is a key still authored by the retiring checker, which under the
/// C plan is a queue and not a classification.
///
/// ⚠ The prose was moved here from `tests/test_authoring_freeze_manifest.py::_AUTHORITY`
/// mechanically (generated from the committed manifest and diffed), not retyped: it is
/// frozen text, and a re-anchoring that quietly reworded the contract would be a value
/// change wearing a refactor's clothes. The one row that *is* reworded is
/// `parity_vectors/*`, deliberately and as a stated diff — see the header.
const AUTHORITY: [(&str, &str, &str); 15] = [
    ("_comment", "hand", "prose header"),
    ("binary_ops", "rust", "BinaryOp::symbol() — the parser's and the S-expr renderer's own spelling. ⚠ Same half-derived shape as expr_nodes: the SYMBOLS are derived, the roster of three variants is hand-maintained. '/' is absent from the TYPE, not merely from the list, so an unsupported op is unrepresentable"),
    ("delegates_to", "hand", "pointer to the station manifest, which owns the param VALUES an authored file reaches through the named loaders. A path, not a derived value — its target's existence is checked by test_manifest_named_files_exist"),
    ("expr_nodes", "rust", "the node names of simcore::expr::Expr, via surface::expr_node_name's exhaustive match. ⚠ HALF-DERIVED: adding a variant is a COMPILE ERROR until someone names it, but the emitted list comes from surface::sample_nodes, a hand-maintained roster of one inhabitant per variant. Python's side is typing.get_args(Expr) and is genuinely derived — that is the checker's advantage on this axis, and it is why the conformance gate below still matters after the flip"),
    ("flow_types/*/*", "rust", "⚠ ROSTER hand-maintained, ENTRIES fully derived. The names come from flow_registry::FLOW_TYPE_NAMES, which that module already documents as hand-maintained because a Rust match cannot be enumerated. Everything about each type is read off its FlowTypeSpec, and `cls` off a flow the dump actually CONSTRUCTS through build_frozen_flow — the same path an authored scenario takes, and the one place slice 2's Flow::type_name() pays out on this contract. So a renamed wiring field, a moved param set, a dropped rate param or a cleared demand-control pair all move the manifest"),
    ("frozen_at_phase", "hand", "the phase this surface froze at"),
    ("grammar_note", "hand", "prose, and the one sentence a reader must not misread: the frozen grammar is DELIBERATELY INCOMPLETE. Neither port derives it and neither should — it records decisions (which ops are deferred and why), not state"),
    ("integrator_names", "rust", "run::INTEGRATOR_NAMES. ⚠ The WEAKEST axis here: Python's copy is the dispatch dict itself and cannot lie, while Rust's dispatch is a match and this is a hand-maintained slice beside it. Two partial closes — both match arms build their error message from the slice, and integrator_names_all_ dispatch RUNS a scenario under every listed name. A match arm added and not listed is caught by nothing on the reference side"),
    ("param_loaders", "rust", "flow_registry::PARAM_SET_NAMES — the sets a kinetics rate's param(\"…\") may read. ⚠ Do NOT confuse this with the param_files key the other two manifests keep Python-retained until slice 9: that one is a list of YAML FILES with their hashes, and this is the loader-name vocabulary. The values behind these names are delegated to the station manifest, not frozen here"),
    ("parity_vectors/*", "python", "PYTHON-RETAINED, and it is the param_files finding reached by a different road. parse_vectors.txt / traj_vectors.txt live in the RUST crate's tests/data and the reference now digests them, because slice C7 moved the WRITER here — but the digest is not the authorship. These files are GENERATED by tests/crossport/gen_authoring_vectors.py and merely re-derived in Rust as the parity check, so the VALUE is the checker's and hashing it here changes nothing, exactly as the biosphere contract's scenarios/*/golden_sha256 read 'rust' through four slices while Python computed the digest. ⚠ This row used to say a Rust-side hash 'would compare the checker's own output with itself'; that conflated who produces a value with who hashes it, and C7 corrected it in place rather than letting the file argue against its own writer. ⚠ Under the 2026-08-17 target state (Rust is the project; Python survives as the external-software oracle and as rewrite scaffolding) these generators are scaffolding, so this key moves when they do — a successor item, not a rider on this ceremony"),
    ("rate_classes", "rust", "interpreter::RATE_CLASSES, the list the interpreter validates a flow's rate_class against — load-bearing like ref_keywords. Closed at two by multirate_step's signature (it takes exactly two Substeppers), so a third cannot appear without a simcore change"),
    ("ref_keywords", "rust", "expr_parser::REF_KEYWORDS. ⚠ LOAD-BEARING, and the strongest tie on this contract: this is the table the parser tests an identifier against, so a name dropped here is not a mis-description, it is a form the grammar stops accepting — the parse vectors and the anchors go red"),
    ("reference_doc", "hand", "pointer to the prose half of the contract"),
    ("schema_fields/*", "rust", "the eight schema::*_KEYS consts. ⚠ LOAD-BEARING on the values: each list IS the argument reject_unknown_keys refuses against, so it decides whether a committed scenario file loads at all. ⚠ The LABELS (ScenarioSpec, ParamPackRef, …) are Python class names typed by hand, and two name no Rust type at all (this port binds params: and includes: through enums) — they are contract names, the way crew.food_metabolism is. std::any::type_name was refused for slice 2's reason: its format is unspecified and a toolchain bump must not move a frozen manifest. ⚠ And the completeness half does NOT port: Python catches a whole new spec model by scanning its module; here a new spec is a new const nothing forces into the dump. The compile-forced version (a SpecKind enum threaded through reject_unknown_keys) was priced in slice 8 and deferred as its own change, per slice 7 declining locked_dt"),
    ("step_token", "rust", "expr_parser::STEP_TOKEN, hoisted out of the lexer's ident match in slice 8 for the reason above: the token spliced here is the token that lowers to StepN. ⚠ NOT the integrator's treatment — that one stayed 'hand' on both other contracts because neither side had an importable name; this one does"),
];

/// The frozen `grammar_note` — prose that records DECISIONS (which operators are
/// deferred and why), not state. Neither port derives it and neither should.
const GRAMMAR_NOTE: &str = "The grammar is bounded and closed, and DELIBERATELY INCOMPLETE (decision D): bare division, the rest of the function set (exp ln pow sqrt abs min max clamp), bounded conditionals and a named-constant surface are all deferred until a real frozen flow forces each semantic choice. Freezing this subset does NOT imply completeness; adding an op is a deliberate unfreeze. 'monod' (S/(S+K)) landed post-roadmap as exactly such an unfreeze — forced by the frozen biosphere.chamber.oxygen_limitation_factor, whose kernel it mirrors (including denom<=0 -> 0, which makes it total). It guards its own denominator, so it resolved x/0 INTERNALLY and bare '/' stays deferred. There is no 'dt' token by construction (RK4-order-safety is structural). Precedence + associativity are enforced by parse_vectors.txt, not recorded here.";


/// The nine keys the reference produces — **one derivation, read by the dump and by the
/// writer**. See the header on why that sharing is deliberate.
///
/// Tier-0 sanity on the program itself, on the axes where an empty answer would be a false
/// claim rather than a legitimate one. ⚠ Unlike the station dump's `aux_set` there is no
/// legitimately-empty axis here — every set below is non-empty by the contract's own
/// definition — so these assertions are cheap and honest. Since C7 they matter more than
/// they did: an empty set here is *written into* the frozen contract by a regeneration
/// run, not merely compared against it.
fn reference_keys() -> Vec<(&'static str, Json)> {
    let expr_nodes = surface::expr_nodes();
    let flow_types = surface::flow_types();
    let schema_fields = surface::schema_fields();
    assert!(!expr_nodes.is_empty(), "no grammar nodes");
    assert!(!flow_types.is_empty(), "no flow types");
    assert!(!schema_fields.is_empty(), "no spec models");

    vec![
        ("binary_ops", Json::strs(surface::binary_ops())),
        ("expr_nodes", Json::strs(expr_nodes)),
        (
            "flow_types",
            Json::obj(flow_types.into_iter().map(|(name, spec)| {
                (
                    name,
                    Json::obj([
                        ("cls", Json::s(spec.cls)),
                        (
                            "demand_controlled",
                            match spec.demand_controlled {
                                Some((field, param)) => {
                                    Json::Array(vec![Json::s(field), Json::s(param)])
                                }
                                None => Json::Null,
                            },
                        ),
                        (
                            "param_set",
                            match spec.param_set {
                                Some(p) => Json::s(p),
                                None => Json::Null,
                            },
                        ),
                        ("rate_params", Json::strs(spec.rate_params.iter().copied())),
                        (
                            "wiring_fields",
                            Json::strs(spec.wiring_fields.iter().copied()),
                        ),
                    ]),
                )
            })),
        ),
        ("integrator_names", Json::strs(surface::integrator_names())),
        ("param_loaders", Json::strs(surface::param_loaders())),
        ("rate_classes", Json::strs(surface::rate_classes())),
        ("ref_keywords", Json::strs(surface::ref_keywords())),
        (
            "schema_fields",
            Json::obj(
                schema_fields
                    .into_iter()
                    .map(|(label, keys)| (label, Json::strs(keys))),
            ),
        ),
        ("step_token", Json::s(surface::STEP_TOKEN)),
    ]
}

/// Print the reference's half of the manifest as JSON — the parity gate's input.
///
/// ⚠ Serialized through the same canonical writer the manifest uses, so the dump and the
/// contract cannot disagree about *shape*. Its consumers parse it
/// (`tests/crossport/test_inventory_parity.py`), so the formatting change C7 made here is
/// invisible to them by construction — checked, not assumed.
pub fn dump() {
    print!("{}", dumps(&Json::obj(reference_keys())));
}

/// The `_authority` block as JSON.
fn authority_json() -> Json {
    Json::obj(AUTHORITY.iter().map(|(path, side, why)| {
        (
            *path,
            Json::obj([("side", Json::s(*side)), ("why", Json::s(*why))]),
        )
    }))
}

/// The whole authoring freeze manifest.
pub fn manifest() -> Json {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    let mut keys = reference_keys();
    keys.extend([
        ("_authority", authority_json()),
        ("_comment", Json::s(COMMENT)),
        (
            "delegates_to",
            Json::s("docs/station-reference.manifest.json"),
        ),
        ("frozen_at_phase", Json::int(9)),
        ("grammar_note", Json::s(GRAMMAR_NOTE)),
        (
            "parity_vectors",
            Json::obj(
                VECTOR_FILES
                    .iter()
                    .map(|name| (*name, Json::s(file_sha256(&data_dir.join(name))))),
            ),
        ),
        ("reference_doc", Json::s("docs/authoring-reference.md")),
    ]);
    Json::obj(keys)
}

/// sha-256 over newline-normalized file content — the provenance rule, applied to a file
/// this program did not compile in.
fn file_sha256(path: &Path) -> String {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {} for hashing: {e}", path.display()));
    assert!(
        contains_exotic_line_separator(&text).is_none(),
        "{} carries a line separator the narrow normalization rule does not handle — \
         see config::provenance",
        path.display()
    );
    normalized_sha256(&text)
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // rust/crates/
        .and_then(Path::parent) // rust/
        .and_then(Path::parent) // the repo root
        .expect("the crate sits three levels below the repo root")
        .to_path_buf()
}

/// The committed contract this writer owns — the default `--write-manifest` target.
///
/// ⚠⚠ **It is a function rather than a literal in the example because it is the
/// dangerous default.** `--write-manifest` with no path rewrites the *freeze contract*,
/// so which file that is has to be reachable by a test; `tests/manifest_writer.rs`
/// asserts this path holds the very bytes the byte gate compares against, which is what
/// says the writer is pointed at its own contract and not at a sibling.
pub fn committed_manifest_path() -> PathBuf {
    repo_root().join("docs").join("authoring-reference.manifest.json")
}

/// Write the manifest to `path`, and report what changed.
///
/// ⚠ Reports rather than asserts: this is the *regeneration* entry point, run on a
/// deliberate unfreeze, so a moved byte is the thing being reviewed. The assertion that
/// the committed file matches lives on the checking side — `tests/manifest_writer.rs`,
/// in this crate since slice S2 — where a stale manifest is red in CI.
///
/// ⚠ **It writes [`manifest_text`] rather than re-deriving the serialization.** It used
/// to call `dumps(&manifest())` itself, one expression away from the gate's subject, while
/// the comment above `manifest_text` claimed *"one serialization, three callers … sharing
/// makes the drift impossible instead of merely detectable"*. Nothing was broken — the two
/// expressions were the same — but the sharing that sentence describes did not exist, which
/// is a doc comment asserting a property nobody tested (S6 build item 1).
pub fn write_manifest(path: &Path) {
    let text = manifest_text();
    let previous = std::fs::read_to_string(path).ok();
    std::fs::write(path, text.as_bytes())
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    match previous {
        Some(old) if old == text => eprintln!("unchanged: {}", path.display()),
        Some(_) => eprintln!("REWRITTEN (review the diff): {}", path.display()),
        None => eprintln!("created: {}", path.display()),
    }
}
