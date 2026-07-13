//! `sim` — the headless command-line driver of the frozen station (Phase-8 Step 8).
//!
//! # The "runs headless on a server" artifact (confirmed decision #2)
//!
//! The Phase-8 exit criterion requires that *"the exact same simulation runs headless."*
//! This is the concrete, no-Godot entry point that demonstrates it: a plain binary that
//! builds a fixed-palette session through the **same** [`station::palette::build_scenario`]
//! the Godot cdylib uses, advances it, and prints the bit-exact `sim_io` hex-float snapshot
//! to stdout. No network, no UI, no `gdext` — just the frozen engine driven from a shell
//! (confirmed decision #2: "runs headless on a server" is satisfied by architecture, not
//! netcode).
//!
//! Because it shares the builder with the front-end, its output is the **same simulation**
//! by construction, not by a re-implementation that happens to agree. And because it prints
//! through the frozen [`simcore::snapshot`] codec (the golden discipline), `sim cabin_gas
//! 900` is **byte-identical** to the headless `emit_cabin_gas` example — the cheap
//! bit-identity proof (`tests/crossport/test_headless_cli.py`).
//!
//! ```text
//! sim <scenario> <steps>
//!   <scenario>  one of: cabin_gas | station | greenhouse | sealed
//!   <steps>     natural units to advance (steps single-rate; master days two-rate)
//! ```

use std::process::ExitCode;

use simcore::snapshot::from_engine;
use station::palette::build_scenario;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!(
            "usage: {} <scenario> <steps>\n  \
             scenario: cabin_gas | station | greenhouse | sealed\n  \
             steps:    natural units to advance (steps single-rate; master days two-rate)",
            args.first().map(String::as_str).unwrap_or("sim"),
        );
        return ExitCode::FAILURE;
    }
    let scenario_id = &args[1];
    let steps: u64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("sim: <steps> must be a non-negative integer, got {:?}", args[2]);
            return ExitCode::FAILURE;
        }
    };

    // The SAME shared builder the Godot cdylib uses — "the exact same simulation" by
    // construction, not by an agreeing re-implementation.
    let (mut session, _display) = match build_scenario(scenario_id) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("sim: {err:?}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(err) = session.step_n(steps) {
        eprintln!("sim: stepping failed at n={}: {err:?}", session.n());
        return ExitCode::FAILURE;
    }

    // The bit-exact hex-float snapshot — the frozen golden codec, printed with no trailing
    // newline so the bytes match the `emit_*` examples exactly.
    print!("{}", from_engine(session.state()).to_json());
    ExitCode::SUCCESS
}
