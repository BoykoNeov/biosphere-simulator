//! Counter-based, keyed RNG — the Rust port of `simcore.rng` (Phase 7, Step 1).
//!
//! Every draw is a pure function of `(seed, key, step)` with **no sequential
//! state**, so a draw's value never depends on how many other draws preceded it
//! (decision #12). Bit-for-bit reproduction of the Python reference is the whole
//! point of this module: the splitmix64 finalizer, the fixed fold order, and the
//! 64-bit masking are the cross-port contract.
//!
//! Python `int` is unbounded and masks every op with `& MASK64`; Rust `u64` wraps
//! natively, so the masks are implicit here and `wrapping_mul` / `wrapping_add`
//! carry the arithmetic. That type difference also **subsumes** the Python
//! `keyed_hash` runtime guards: the three `TypeError` tests over there
//! (non-int / float / `bool` key words) have no Rust analogue *by design* — a
//! non-`u64` word simply does not type-check, so there is nothing to reject at
//! run time. Their absence is the type system doing the check statically, not a
//! dropped behavior.
//!
//! The vectors in `tests/rng_vectors.rs` (Python-generated, `rng_vectors.txt`)
//! pin `mix64` / `draw_u64` / `draw` against the frozen reference bit-for-bit.

/// splitmix64 finalizer constants (externally specified avalanche function).
const M1: u64 = 0xBF58476D1CE4E5B9;
const M2: u64 = 0x94D049BB133111EB;
/// golden-ratio odd gamma, used to spread successive folded words.
const GAMMA: u64 = 0x9E3779B97F4A7C15;
/// 2**53, exact in IEEE-754 double; the divisor mapping a 53-bit int into [0, 1).
const FLOAT_DIVISOR: f64 = (1u64 << 53) as f64;

/// The splitmix64 finalizer: a stateless 64-bit avalanche hash of `z`.
///
/// Used as a keyed mixing primitive (not splitmix64's sequential `next`). Every
/// op is a native `u64` operation — the Python `& MASK64` masks are implicit.
pub fn mix64(z: u64) -> u64 {
    let z = (z ^ (z >> 30)).wrapping_mul(M1);
    let z = (z ^ (z >> 27)).wrapping_mul(M2);
    z ^ (z >> 31)
}

/// Fold `(seed, step, *key)` into a 64-bit value via the finalizer.
///
/// The fold order is part of the cross-port contract: `step` is folded first,
/// then each key word, each mixed as `mix64((h + GAMMA) ^ word)`. Order
/// *independence* (decision #12) is across draws — this is a pure function, so a
/// given `(seed, key, step)` always yields the same value.
pub fn keyed_hash(seed: u64, key: &[u64], step: u64) -> u64 {
    let mut h = seed;
    for &word in std::iter::once(&step).chain(key.iter()) {
        h = h.wrapping_add(GAMMA);
        h = mix64(h ^ word);
    }
    h
}

/// Concrete counter-based RNG. `seed` is the per-run master seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterRng {
    pub seed: u64,
}

impl CounterRng {
    /// Construct a generator keyed on `seed`.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Raw 64-bit draw for `(key, step)` — the cross-port-pinned integer.
    pub fn draw_u64(&self, key: &[u64], step: u64) -> u64 {
        keyed_hash(self.seed, key, step)
    }

    /// Uniform draw in `[0, 1)`: `(x >> 11) / 2**53`, the full 53-bit mantissa.
    ///
    /// Bit-exact across ports by construction: `x >> 11` is at most 53 bits so
    /// `as f64` is lossless, and the divisor is a power of two, so the division
    /// introduces no rounding. Cannot return 1.0 and never produces NaN/Inf.
    pub fn draw(&self, key: &[u64], step: u64) -> f64 {
        (self.draw_u64(key, step) >> 11) as f64 / FLOAT_DIVISOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// mix64 == the published splitmix64(seed=0) output sequence: seeded at 0,
    /// splitmix64's `next` computes `finalizer(x += GAMMA)`, so the first three
    /// outputs are `mix64(i*GAMMA)` for i = 1, 2, 3. The authoritative cross-port
    /// pin is `tests/rng_vectors.rs`; this is a fast in-crate grounding check.
    #[test]
    fn mix64_matches_published_splitmix64_seed0() {
        let expected: [u64; 3] = [
            0xE220A8397B1DCDAF,
            0x6E789E6AA1B965F4,
            0x06C45D188009454F,
        ];
        for (i, &want) in expected.iter().enumerate() {
            let i = (i + 1) as u64;
            assert_eq!(mix64(GAMMA.wrapping_mul(i)), want);
        }
    }

    /// The empty-key path ties directly to the first published constant:
    /// keyed_hash(0, (), 0) folds only `step`, i.e. mix64(0 + GAMMA).
    #[test]
    fn keyed_hash_empty_key_seed0_step0() {
        assert_eq!(keyed_hash(0, &[], 0), 0xE220A8397B1DCDAF);
        assert_eq!(CounterRng::new(0).draw_u64(&[], 0), 0xE220A8397B1DCDAF);
    }

    /// A multi-word key folds each word in order after `step`.
    #[test]
    fn keyed_hash_multiword_key() {
        // (seed 0x3039, key (7, 99), step 42) — a rng_vectors.txt row.
        assert_eq!(keyed_hash(0x3039, &[7, 99], 42), 0x5923E742840BE160);
    }

    /// draw stays in [0, 1) and is a pure function of its inputs.
    #[test]
    fn draw_is_unit_interval_and_pure() {
        let rng = CounterRng::new(0xDEADBEEF);
        let a = rng.draw(&[3], 1_000_000);
        let b = rng.draw(&[3], 1_000_000);
        assert_eq!(a, b);
        assert!((0.0..1.0).contains(&a));
    }
}
