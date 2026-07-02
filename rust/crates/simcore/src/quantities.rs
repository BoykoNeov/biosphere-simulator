//! Conserved quantities, stock kinds, and the canonical-unit table — the Rust port
//! of `simcore.quantities`.
//!
//! [`Quantity`] members carry two string projections, both mirrored from the Python
//! `Enum`: [`Quantity::value`] (lowercase — `"carbon"`, used in loss-sink ids,
//! snapshot JSON, and composition keys) and [`Quantity::name`] (uppercase —
//! `"CARBON"`, used in the by-name sort order and error messages). The port replicates
//! both so canonical ordering and diagnostics match byte-for-byte.
//!
//! `ENERGY` is in the asserted conserved set (it joined in Phase 5 — the Power
//! domain); the balance gate asserts every member independently.

use crate::ids::UnitLabel;

/// A conserved quantity tracked by the ledger. `PHOSPHORUS` is reserved in the
/// Python source as a comment (no stock/unit yet); it is intentionally absent here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quantity {
    Carbon,
    Water,
    Nitrogen,
    Oxygen,
    Energy,
}

impl Quantity {
    /// The lowercase canonical value (`"carbon"`) — the Python `Enum` *value*, used
    /// in loss-sink ids, snapshot JSON, and composition keys.
    pub fn value(&self) -> &'static str {
        match self {
            Quantity::Carbon => "carbon",
            Quantity::Water => "water",
            Quantity::Nitrogen => "nitrogen",
            Quantity::Oxygen => "oxygen",
            Quantity::Energy => "energy",
        }
    }

    /// The uppercase member name (`"CARBON"`) — the Python `Enum` *name*, used for
    /// the by-name canonical sort and in error messages.
    pub fn name(&self) -> &'static str {
        match self {
            Quantity::Carbon => "CARBON",
            Quantity::Water => "WATER",
            Quantity::Nitrogen => "NITROGEN",
            Quantity::Oxygen => "OXYGEN",
            Quantity::Energy => "ENERGY",
        }
    }

    /// The canonical-unit label for this quantity (single source of truth, #9).
    /// Total by construction — every member has an entry (Python raises `KeyError`
    /// on a gap; here the match is exhaustive, so a gap is a compile error).
    pub fn canonical_unit(&self) -> UnitLabel {
        match self {
            Quantity::Carbon => "mol",
            Quantity::Water => "kg",
            Quantity::Nitrogen => "kg",
            Quantity::Oxygen => "mol",
            Quantity::Energy => "J",
        }
        .to_string()
    }
}

/// Every `Quantity` in `.name`-sorted order — the canonical iteration order the
/// balance gate uses so the first reported failure is deterministic. Sorting the
/// uppercase names `CARBON < ENERGY < NITROGEN < OXYGEN < WATER` gives this fixed
/// list (mirrors Python's `sorted(ASSERTED_QUANTITIES, key=lambda q: q.name)`).
pub const ASSERTED_QUANTITIES: [Quantity; 5] = [
    Quantity::Carbon,
    Quantity::Energy,
    Quantity::Nitrogen,
    Quantity::Oxygen,
    Quantity::Water,
];

/// How a stock behaves under arbitration and extinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockKind {
    /// A resource pool — never zeroed-with-loss; arbitration may throttle draws.
    Pool,
    /// Absorbing-eligible biomass/population — may go extinct (snap to 0, route the
    /// residual to the loss-sink).
    Population,
    /// An "outside" reservoir — its per-step delta *is* a ledger Input/Output.
    Boundary,
}

impl StockKind {
    /// The lowercase canonical value (`"pool"`) — the Python `Enum` value, used in
    /// snapshot JSON.
    pub fn value(&self) -> &'static str {
        match self {
            StockKind::Pool => "pool",
            StockKind::Population => "population",
            StockKind::Boundary => "boundary",
        }
    }

    /// The uppercase member name (`"POOL"`) — used in error messages.
    pub fn name(&self) -> &'static str {
        match self {
            StockKind::Pool => "POOL",
            StockKind::Population => "POPULATION",
            StockKind::Boundary => "BOUNDARY",
        }
    }
}

/// The balance tolerance: `abs(residual) <= BALANCE_ATOL + BALANCE_RTOL * scale`.
/// Same Phase-0 values as the Python contract (`simcore.quantities`).
pub const BALANCE_ATOL: f64 = 1e-9;
/// The relative term of the balance tolerance (see [`BALANCE_ATOL`]).
pub const BALANCE_RTOL: f64 = 1e-9;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_units_match_python_table() {
        assert_eq!(Quantity::Carbon.canonical_unit(), "mol");
        assert_eq!(Quantity::Water.canonical_unit(), "kg");
        assert_eq!(Quantity::Nitrogen.canonical_unit(), "kg");
        assert_eq!(Quantity::Oxygen.canonical_unit(), "mol");
        assert_eq!(Quantity::Energy.canonical_unit(), "J");
    }

    #[test]
    fn asserted_order_is_name_sorted() {
        // The exact order Python's `sorted(..., key=q.name)` yields.
        let names: Vec<&str> = ASSERTED_QUANTITIES.iter().map(|q| q.name()).collect();
        assert_eq!(names, ["CARBON", "ENERGY", "NITROGEN", "OXYGEN", "WATER"]);
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }
}
