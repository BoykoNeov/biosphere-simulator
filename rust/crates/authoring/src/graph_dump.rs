//! A canonical structural dump of an interpreted scenario graph — the Tier-0
//! **file-level parse-parity** surface (Phase 9, Step 4b / decision E).
//!
//! The scenario `.yaml` is the shared cross-port artifact (unlike Step 4a's rate
//! *strings*, which had no file), so the primary gate is **byte-identity**: the Rust
//! interpreter loads an anchor, runs it, emits `sim_io` JSON, and the crossport test
//! diffs it against the frozen golden. But a final-state snapshot is **blind** to graph
//! facts that do not move *this* run — a flow's priority, a present-but-inert flow, the
//! exact boundary-eval of an initial amount. This dump makes those explicit: both ports
//! render the *same* canonical text from their interpreted graph, and the crossport
//! test diffs the two. It is exactly the structural surface Step 7 freezes ("same
//! stock-id set, same flow-id set + priorities, same param values").
//!
//! **The format is a parity contract** (like `authoring::sexpr`): the Python
//! `render_graph_dump` in `tests/crossport` must render byte-identically. Every float
//! goes through the hex-float codec so an amount / forcing constant / composition
//! coefficient is compared **bit-exact** — this is where the Step-3 boundary-eval
//! (`param('crew_count') * 1000.0`) parity is proven without needing a golden per
//! instantiation.
//!
//! **What the dump deliberately does NOT carry: per-flow wiring.** A frozen `Flow`
//! exposes no static stock refs (its wired ids are private), so wiring cannot be
//! read out here generically. Wiring parity is instead **trajectory-covered** — a
//! mis-wire moves the run (or breaks conservation), so it surfaces in the byte-identity
//! / run-match gates. Do not read this dump as claiming wiring-parity it does not carry.

use simcore::hexfloat;

use crate::interpreter::BuiltScenario;

/// Render `built` to the canonical structural dump text (LF-terminated lines).
///
/// Sections, each already in a deterministic order:
/// * `scenario <name>` and `config <integrator> <dt.hex> <steps> <rng_seed>`;
/// * one `stock` line per stock, **id-sorted** (the `State.stocks` are a `BTreeMap`),
///   carrying every field the interpreter sets — the initial amount is hex-float so
///   the boundary-eval is bit-exact;
/// * one `flow` line per flow in **canonical id-sorted** registry order, with its
///   priority (the fact final-state byte-identity is blind to);
/// * one `forcing` line per forcing, name-sorted, with its constant value (evaluated at
///   `n=0, dt=0`) hex-float — the other boundary-eval site.
pub fn render_graph_dump(built: &BuiltScenario) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("scenario\t{}", built.name));
    lines.push(format!(
        "config\t{}\t{}\t{}\t{}",
        built.integrator,
        hexfloat::format(built.dt),
        built.steps,
        built.state.rng_seed,
    ));
    lines.push(format!(
        "has_authored_kinetics\t{}",
        if built.has_authored_kinetics { 1 } else { 0 }
    ));

    // Stocks — id-sorted (BTreeMap iteration order).
    for (id, stock) in &built.state.stocks {
        // Composition, quantity-value-sorted, each coeff hex-float.
        let mut comp: Vec<(String, f64)> = stock
            .composition
            .iter()
            .map(|(q, c)| (q.value().to_string(), *c))
            .collect();
        comp.sort_by(|a, b| a.0.cmp(&b.0));
        let comp_str = comp
            .iter()
            .map(|(q, c)| format!("{q}={}", hexfloat::format(*c)))
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!(
            "stock\t{id}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{comp_str}",
            stock.domain,
            stock.quantity.value(),
            stock.kind.value(),
            stock.unit,
            hexfloat::format(stock.amount),
            if stock.unclamped { 1 } else { 0 },
            hexfloat::format(stock.extinction_threshold),
        ));
    }

    // Flows — canonical id-sorted registry order, with priority.
    for flow in built.registry.flows() {
        lines.push(format!("flow\t{}\t{}", flow.id(), flow.priority()));
    }

    // Forcings — name-sorted; the constant value is the schedule at (n=0, dt=0).
    let mut forcing_names: Vec<&String> = built.resolver.forcings().keys().collect();
    forcing_names.sort();
    for name in forcing_names {
        let schedule = &built.resolver.forcings()[name];
        let value = schedule(0, 0.0);
        lines.push(format!("forcing\t{name}\t{}", hexfloat::format(value)));
    }

    let mut out = lines.join("\n");
    out.push('\n');
    out
}
