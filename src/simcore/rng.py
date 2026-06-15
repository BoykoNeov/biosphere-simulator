"""Counter-based, keyed, pure-Python RNG (decision #12).

Every draw is a pure function of ``(seed, key, step)`` — there is **no sequential
state**. This makes draws independent of consumption order (a draw's value never
depends on how many other draws happened first), which is required by determinism
(#7) and is what a sequential generator like PCG64/Mersenne cannot give.

Cross-port contract (this is the part the future Rust port must reproduce
bit-for-bit; the golden hex vectors in the tests are the conformance target):

  * The mixing primitive is the **splitmix64 finalizer** — the externally
    specified avalanche function, with its two fixed odd constants.
  * Every arithmetic op is masked to 64 bits (``& MASK64``). Python ints are
    unbounded; Rust ``u64`` wraps — masking is what keeps them identical.
  * ``(seed, step, *key)`` are folded in a fixed order with the golden-ratio
    gamma. ``key`` is a tuple of **ints** for Phase 0 (string-folding would mean
    committing to a portable byte-hash like FNV-1a plus its own vectors — out of
    scope until a scenario needs it).
  * The float draw is ``(x >> 11) / 2**53`` → ``[0, 1)``: the full 53-bit
    mantissa, never exactly 1.0, never NaN/Inf.

Pure stdlib only.
"""

from dataclasses import dataclass
from typing import Protocol, runtime_checkable

# --- cross-port constants (do not change without re-pinning golden vectors) ---
MASK64 = 0xFFFFFFFFFFFFFFFF
# splitmix64 finalizer constants (externally specified avalanche function).
_M1 = 0xBF58476D1CE4E5B9
_M2 = 0x94D049BB133111EB
# golden-ratio odd gamma, used to spread successive folded words.
_GAMMA = 0x9E3779B97F4A7C15
# 2**53, exact in IEEE-754 double; divisor mapping a 53-bit int into [0, 1).
_FLOAT_DIVISOR = float(1 << 53)


def mix64(z: int) -> int:
    """The splitmix64 finalizer: a stateless 64-bit avalanche hash of ``z``.

    Used here as a keyed mixing primitive (not as splitmix64's sequential
    ``next``). All ops masked to 64 bits for cross-port (u64-wrap) parity.
    """
    z &= MASK64
    z = ((z ^ (z >> 30)) * _M1) & MASK64
    z = ((z ^ (z >> 27)) * _M2) & MASK64
    z ^= z >> 31
    return z & MASK64


def keyed_hash(seed: int, key: tuple[int, ...], step: int) -> int:
    """Fold ``(seed, step, *key)`` into a 64-bit value via the finalizer.

    Fixed fold order is part of the cross-port contract. Order-*independence*
    (decision #12) is across *draws*: this is a pure function, so a given
    ``(seed, key, step)`` always yields the same value no matter when or in what
    order it is requested relative to other draws.
    """
    if not isinstance(step, int) or isinstance(step, bool):
        raise TypeError(f"step must be int, got {type(step).__name__}")
    h = seed & MASK64
    for word in (step, *key):
        if not isinstance(word, int) or isinstance(word, bool):
            raise TypeError(
                "rng key/step words must be plain ints (Phase-0 cross-port "
                f"constraint); got {type(word).__name__}"
            )
        h = (h + _GAMMA) & MASK64
        h = mix64(h ^ (word & MASK64))
    return h


@runtime_checkable
class Rng(Protocol):
    """Frozen-API RNG surface: a seed plus order-independent keyed draws."""

    seed: int

    def draw(self, key: tuple[int, ...], step: int) -> float: ...


@dataclass(frozen=True)
class CounterRng:
    """Concrete counter-based RNG. ``seed`` is the per-run master seed."""

    seed: int

    def draw_u64(self, key: tuple[int, ...], step: int) -> int:
        """Raw 64-bit draw for ``(key, step)`` — the cross-port-pinned integer."""
        return keyed_hash(self.seed, key, step)

    def draw(self, key: tuple[int, ...], step: int) -> float:
        """Uniform draw in ``[0, 1)`` for ``(key, step)``.

        Uses the top 53 bits: ``(x >> 11) / 2**53``. Cannot return 1.0 and never
        produces NaN/Inf, so it is always safe to store in state.
        """
        return (self.draw_u64(key, step) >> 11) / _FLOAT_DIVISOR
