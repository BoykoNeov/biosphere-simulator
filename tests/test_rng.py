"""Step-2 RNG tests (decision #12).

Two tiers of guarantee:

* **Same-build** (determinism, order-independence, range) — proves the generator
  is a pure, order-independent function. Exercised with Hypothesis.
* **Cross-port** (known-answer / golden vectors) — pins the *exact* bytes the
  future Rust port must reproduce. This is the discriminating test: any pure
  function passes the same-build tier, but only a byte-faithful reimplementation
  reproduces these hex vectors. The ``mix64`` primitive is additionally checked
  against the externally published splitmix64(seed=0) output sequence.
"""

import random

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore.rng import CounterRng, Rng, keyed_hash, mix64

# ---------------------------------------------------------------------------
# Cross-port: mix64 == canonical splitmix64 finalizer.
# splitmix64 next(x) computes finalizer(x += GAMMA); seeded at 0 the first three
# outputs are these widely published constants.
# ---------------------------------------------------------------------------
_GAMMA = 0x9E3779B97F4A7C15
_MASK64 = 0xFFFFFFFFFFFFFFFF
_SPLITMIX64_SEED0 = (
    0xE220A8397B1DCDAF,
    0x6E789E6AA1B965F4,
    0x06C45D188009454F,
)


def test_mix64_matches_published_splitmix64_seed0() -> None:
    for i, expected in enumerate(_SPLITMIX64_SEED0, start=1):
        assert mix64((i * _GAMMA) & _MASK64) == expected


def test_mix64_masks_to_64_bits() -> None:
    # Input far wider than 64 bits must not leak high bits into the output.
    assert 0 <= mix64((1 << 200) | 0x1234) <= _MASK64


# ---------------------------------------------------------------------------
# Cross-port: golden keyed-hash + float-draw vectors. Regenerate ONLY with a
# deliberate, documented contract change (and bump the plan's cross-port note).
# Floats are pinned as hex (exact, per the project's hex-float convention).
# ---------------------------------------------------------------------------
# (seed, key, step, u64, float_hex)
_GOLDEN: tuple[tuple[int, tuple[int, ...], int, int, str], ...] = (
    (0x0, (), 0, 0xE220A8397B1DCDAF, "0x1.c4415072f63b9p-1"),
    (0x0, (1,), 0, 0x2A98F501AF37E97F, "0x1.54c7a80d79bf4p-3"),
    (0x0, (1,), 1, 0x5155E650B56274F2, "0x1.45579942d589cp-2"),
    (0x0, (2,), 0, 0x82876E1C4F0B438C, "0x1.050edc389e168p-1"),
    (0x3039, (7, 99), 42, 0x5923E742840BE160, "0x1.648f9d0a102f8p-2"),
    (0xDEADBEEF, (3,), 1000000, 0xB6E3ECEBB0709AD6, "0x1.6dc7d9d760e13p-1"),
)


@pytest.mark.parametrize(("seed", "key", "step", "u64", "fhex"), _GOLDEN)
def test_golden_vectors(
    seed: int, key: tuple[int, ...], step: int, u64: int, fhex: str
) -> None:
    rng = CounterRng(seed)
    assert rng.draw_u64(key, step) == u64
    assert keyed_hash(seed, key, step) == u64
    assert rng.draw(key, step).hex() == fhex


# ---------------------------------------------------------------------------
# Same-build: determinism, range, order-independence.
# ---------------------------------------------------------------------------
@given(
    seed=st.integers(min_value=0, max_value=_MASK64),
    key=st.tuples(st.integers(min_value=0), st.integers(min_value=0)),
    step=st.integers(min_value=0, max_value=10**9),
)
def test_draw_is_deterministic_and_in_unit_interval(
    seed: int, key: tuple[int, int], step: int
) -> None:
    rng = CounterRng(seed)
    a = rng.draw(key, step)
    b = rng.draw(key, step)
    assert a == b  # pure function: repeated call is identical
    assert 0.0 <= a < 1.0  # never exactly 1.0, never NaN/Inf


@given(
    seed=st.integers(min_value=0, max_value=_MASK64),
    pairs=st.lists(
        st.tuples(
            st.tuples(st.integers(min_value=0, max_value=1000)),
            st.integers(min_value=0, max_value=1000),
        ),
        min_size=1,
        max_size=25,
        unique=True,
    ),
)
def test_draws_are_order_independent(
    seed: int, pairs: list[tuple[tuple[int], int]]
) -> None:
    # The value for a given (key, step) must not depend on the order in which
    # draws are requested — the core of decision #12.
    rng = CounterRng(seed)
    forward = {(k, s): rng.draw(k, s) for k, s in pairs}
    shuffled = list(pairs)
    random.Random(seed).shuffle(shuffled)
    backward = {(k, s): rng.draw(k, s) for k, s in shuffled}
    assert forward == backward


def test_distinct_inputs_generally_differ() -> None:
    rng = CounterRng(0)
    # Not a strict guarantee, but a 64-bit avalanche collision here would signal a
    # broken mixer. Sample a grid of (key, step) and require all-distinct u64s.
    vals = {rng.draw_u64((k,), s) for k in range(16) for s in range(16)}
    assert len(vals) == 256


# ---------------------------------------------------------------------------
# Phase-0 key constraint: ints only (no silent string-folding), bool rejected.
# ---------------------------------------------------------------------------
def test_non_int_key_word_rejected() -> None:
    with pytest.raises(TypeError):
        keyed_hash(0, ("oops",), 0)  # type: ignore[arg-type]


def test_non_int_step_rejected() -> None:
    with pytest.raises(TypeError):
        keyed_hash(0, (1,), 1.5)  # type: ignore[arg-type]


def test_bool_rejected_as_word() -> None:
    # bool is an int subclass; accepting it would make True/1 alias silently.
    with pytest.raises(TypeError):
        keyed_hash(0, (True,), 0)


def test_counter_rng_satisfies_protocol() -> None:
    assert isinstance(CounterRng(0), Rng)
