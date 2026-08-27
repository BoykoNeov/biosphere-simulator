//! Regenerate the regression goldens from the reference — Stage-3 slice **S6, build item
//! 2**, the Rust successor to `tests/crossport/regen_goldens_from_rust.py`.
//!
//! ⚠ **The program is the thin part**, exactly as the three manifest writers are: an
//! `examples/` program is a binary target, so nothing in `cargo test` can call into this
//! `main`. Everything decidable — the argument parse, the selection, the validation, the
//! two-phase write — lives in [`station::regen`], where tests reach it.
//!
//! ```text
//! cd rust
//! cargo run --release -p station --example regen_goldens                 # report only
//! cargo run --release -p station --example regen_goldens -- --write      # rewrite
//! cargo run -p station --example regen_goldens -- --only season          # one golden
//! ```

use station::regen::{parse_args, regenerate, summary, target_dir};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let request = match parse_args(&args) {
        Ok(request) => request,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    eprintln!("goldens: {}", target_dir().display());
    match regenerate(&request) {
        Ok(outcomes) => println!("{}", summary(&request, &outcomes)),
        Err(why) => {
            eprintln!("{why}");
            std::process::exit(1);
        }
    }
}
