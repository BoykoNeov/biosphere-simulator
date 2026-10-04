## **Tooling: CI went red on a Rust release alone, and the runner is now pinned**

> One row of the record table in [`../post-roadmap-log.md`](../post-roadmap-log.md),
> written out. The heading is that row's Work cell verbatim — a gate checks it.

**FIXED 2026-10-04** on the user's calls (*"check the status of recent github ci runs, fix if
needed"*, then *"work on the upcoming changes"*). No science touched, no golden regenerated.

## What was red, and why

CI's Rust job had failed on every push since 2026-10-01 12:47 — about 25 runs — with **no code
cause**. `cargo test` passed throughout; only `cargo clippy --all-targets -- -D warnings`
failed. The runner's stable toolchain moved from Rust 1.98.1 to 1.99.0 between the last green
run (36860604749) and the first red one (36864133734), read off each run's install-step log.
The new clippy reports `redundant_field_names` on the `base` field of both `#[class(init)]`
structs in `godot_bridge`: gdext 0.5.4's init expansion writes `#name: base,`
(`godot-macros-0.5.4/src/class/derive_godot_class.rs:396`) spanned on our field. A
struct-level `#[allow]` does not reach the generated impl (measured under 1.99.0), so the
allow is crate-level in `godot_bridge/src/lib.rs` (2bc2838). Reproduced and verified locally
under 1.99.0 from an isolated `RUSTUP_HOME`; green on CI in run 37197862410.

## The two notices that were next

- **Node 20 actions.** `actions/checkout@v4` → `@v7` and `astral-sh/setup-uv@v5` →
  `@v10.2.0`. setup-uv publishes no `@vN` tags since v8, so the full version is required.
  Neither project's breaking changes since touch a job that only checks out and runs `uv`.
- **`ubuntu-latest` becomes Ubuntu 26.04 from 2026-10-19.** All three jobs now name
  `ubuntu-26.04` explicitly (2742509), so the next platform change is a visible commit.

**Trial before landing:** PR #1, run 37199290413 on `ubuntu-26.04`, all three jobs green, and
the per-step test results **identical** to the last 24.04 run — 162 test-binary results;
Rust 1280 passed / 4 ignored / 0 failed, Godot parity 18 passed, pytest 8 passed / 3 skipped.
⚠ Compared **order-insensitively per step**: cargo's `Running` lines (stderr) and its
`test result:` lines (stdout) interleave in GitHub's log, so pairing a name with a count by
position mismatched dozens of rows that were in fact equal. Main green after the merge (run
37200367649).

## Not done

The Rust toolchain is still `dtolnay/rust-toolchain@stable`, which is what moved under us.
Pinning it the same way was offered to the user, not taken.
