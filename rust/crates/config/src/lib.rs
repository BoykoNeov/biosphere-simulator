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
//! * [`json`] — the closed-subset JSON reader, and [`date`] the ISO-date → day-of-year
//!   calendar computation beside it. **Added in slice C9**, so the reference can read
//!   the committed raw-weather fixture directly instead of through a Python generator.
//!   They are here for the same reason as [`yaml`]: this is the crate that turns files
//!   on disk into values, and it is the one that may take no third-party code.
//! * [`errors`] — one error type for every failure decidable from a param file alone.
//!
//! # Why there is no units library here
//!
//! Measured before the slice was designed, not assumed: every declared unit in the
//! frozen tree is validated by **exact string comparison**, and the two Python functions
//! that genuinely convert have six live callers between them, **all identities**. The
//! full census is in `docs/plans/post-roadmap-reference-flip.md` §5d.

pub mod date;
pub mod errors;
pub mod json;
pub mod params;
pub mod provenance;
pub mod yaml;

pub use date::{is_leap_year, iso_day_of_year};
pub use errors::ConfigError;
pub use json::{parse_json, JsonValue};
pub use params::{
    require_closed, require_half_open, require_non_negative, require_positive, Entry, ParamFile,
};
pub use provenance::{normalize_newlines, normalized_sha256};
pub use yaml::{parse_document, YamlValue};
