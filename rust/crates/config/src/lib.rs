//! The param-file boundary — the Rust port of Python's `src/config` (reference flip,
//! slice C1).
//!
//! Python's layering puts `config/` **below** the domains: `domains.*.loader` imports
//! `config`, never the other way round, and the pure core (`simcore`) imports neither.
//! This crate mirrors that exactly — it is zero-dependency, including on `simcore`, so
//! nothing about reading a file can reach into the engine.
//!
//! # What lives here
//!
//! * [`yaml`] — the closed-subset YAML reader. **Moved here from `authoring`, not
//!   reimplemented.** `authoring` re-exports it at its original path, so its public
//!   surface is unchanged; a second reader would have been the defect
//!   `docs/log/reference-flip.md` records from slice 5 (*a policy with two
//!   implementations has one that is stale*).
//! * [`params`] — the `{value, unit, source}` entry schema, the exact-string unit guard,
//!   and the bound helpers: the shared half of what was a hand-written pydantic schema
//!   per process.
//! * [`errors`] — one error type for every failure decidable from a param file alone.
//!
//! # Why there is no units library here
//!
//! Measured before the slice was designed, not assumed: every declared unit in the
//! frozen tree is validated by **exact string comparison**, and the two Python functions
//! that genuinely convert have six live callers between them, **all identities**. The
//! full census is in `docs/plans/post-roadmap-reference-flip.md` §5d.

pub mod errors;
pub mod params;
pub mod yaml;

pub use errors::ConfigError;
pub use params::{
    require_closed, require_half_open, require_non_negative, require_positive, Entry,
    ParamFile,
};
pub use yaml::{parse_document, YamlValue};
