"""Post-roadmap Tier-2 tests: the ``monod`` grammar unfreeze.

The plan of record is ``docs/plans/post-roadmap-grammar-monod.md``. Four surfaces, in
the order they carry weight:

* **the frozen oracle** — the ``monod`` kernel is *bit-exact* against
  ``domains.biosphere.chamber.oxygen_limitation_factor``, the frozen (Phase-2 Step 7,
  Davidson et al. 2012) flow that forced its definition. This is the ``SelfDischarge``
  re-expression pattern applied to a **grammar primitive**: the op IS the frozen
  science, not merely inspired by it;
* **totality** — for all finite inputs the node returns a finite float: never NaN,
  never ±inf, never raising. This is what dissolves the "``0/0`` crossing a hex-float
  golden contract" hazard the Tier-1 doc flagged;
* **the parser** — arg order, arity, and the comma's confinement to the call;
* **the AST walks** — a ``monod`` subtree is namespaced and ref-validated like any
  other, which two permissive fallbacks used to get silently wrong.
"""

from __future__ import annotations

import math

import pytest

from authoring.compose import _prefix_expr_refs
from authoring.errors import AuthoringError
from authoring.expr_parser import parse_rate_expr, render_rate_expr
from authoring.interpreter import _collect_refs
from authoring.template import eval_numeric_field
from domains.biosphere.chamber import oxygen_limitation_factor
from simcore.expr import Const, Monod, ParamRef, StockRef, eval_expr
from simcore.ids import StockId


def monod(substrate: float, half_saturation: float) -> float:
    """Evaluate the bare kernel — no snapshot/env/params needed for two literals."""
    return eval_expr(Monod(Const(substrate), Const(half_saturation)), None, None, {})  # type: ignore[arg-type]


# --- the frozen oracle (the load-bearing test) -----------------------------
# The frozen f_O2 does TWO things; only the second is monod:
#   x_o2 = max(0.0, o2_mol) / air_mol   <- argument preparation (the author's job)
#   denom = k_o2 + x_o2; if denom <= 0: return 0.0; return x_o2 / denom   <- the kernel
# So the oracle compares the kernel against the frozen function fed prepared args.
_AIR_MOL = 40.0

# The frozen domain: a chamber O₂ amount (mol) x an O₂ half-saturation (mole fraction).
# k=0 is the frozen "limit disabled" setting; 1.5e-4 is the frozen low/sharp K_O2 scale.
_O2_MOL = (0.0, 1e-12, 1e-6, 1e-3, 0.5, 1.0, 8.4, 21.0, 40.0, 1e6)
_K_O2 = (0.0, 1e-9, 1e-6, 1.5e-4, 0.01, 0.5, 1.0)


@pytest.mark.parametrize("o2_mol", _O2_MOL)
@pytest.mark.parametrize("k_o2", _K_O2)
def test_monod_is_bit_exact_against_the_frozen_f_o2(o2_mol: float, k_o2: float) -> None:
    """``monod(x_O2, K_O2)`` reproduces frozen ``f_O2`` bit-for-bit.

    Bit-exact, not approximate: IEEE ``+`` is commutative, so the kernel's
    ``substrate + half_saturation`` and the frozen ``k_o2 + x_o2`` agree exactly, and
    the division that follows is a correctly-rounded IEEE basic op. A failure here is a
    finding (the grammar op has drifted from the frozen science it claims to be), never
    a tolerance to widen.
    """
    frozen = oxygen_limitation_factor(o2_mol, air_mol=_AIR_MOL, k_o2=k_o2)
    prepared_substrate = max(0.0, o2_mol) / _AIR_MOL  # the author composes this half
    assert monod(prepared_substrate, k_o2).hex() == frozen.hex()


def test_monod_matches_the_frozen_f_o2_degenerate_case() -> None:
    """The one case that forced the semantic choice: no O₂ and no limit → 0, not NaN."""
    assert oxygen_limitation_factor(0.0, air_mol=_AIR_MOL, k_o2=0.0) == 0.0
    assert monod(0.0, 0.0) == 0.0


# --- totality: the hazard the Tier-1 doc flagged, measured -----------------
_FINITE = (0.0, -0.0, 1.0, -1.0, 5.0, -5.0, 1e-300, 1e300, -1e300, 0.5, -0.5)


@pytest.mark.parametrize("substrate", _FINITE)
@pytest.mark.parametrize("half_saturation", _FINITE)
def test_monod_is_total_over_finite_inputs(
    substrate: float, half_saturation: float
) -> None:
    """No finite input produces NaN, ±inf, or an exception.

    This is what makes the Tier-1 doc's "NaN crossing a hex-float golden contract"
    hazard *dissolve* rather than need designing against: there is no NaN to cross it,
    and no raw ``x/0`` for the ports to disagree about (Python raise vs Rust inf).
    """
    result = monod(substrate, half_saturation)
    assert math.isfinite(result)


def test_monod_is_the_textbook_curve_on_its_natural_domain() -> None:
    """S >= 0, K > 0 → [0, 1), monotone rising, and half-saturated exactly at S == K."""
    assert monod(0.0, 2.0) == 0.0
    assert monod(2.0, 2.0) == 0.5  # the defining property of the half-saturation const
    assert monod(1e300, 1.0) == pytest.approx(1.0)  # saturates, never exceeds
    rising = [monod(s, 2.0) for s in (0.0, 0.5, 1.0, 2.0, 10.0, 100.0)]
    assert rising == sorted(rising)
    assert all(0.0 <= v < 1.0 for v in rising)


def test_monod_zero_half_saturation_disables_the_limit() -> None:
    """``K == 0`` → 1 for any S > 0 — the frozen "limit off" behaviour, preserved."""
    assert monod(1.0, 0.0) == 1.0
    assert monod(1e-300, 0.0) == 1.0


def test_monod_does_not_sign_flip_on_a_negative_denominator() -> None:
    """A negative denominator returns 0 rather than a positive-looking factor.

    ``monod(-3, 1)`` is ``-3 / -2 = +1.5`` if computed naively — a *positive*
    saturation factor from a negative substrate. The ``denom <= 0`` guard (the frozen
    flow's own) catches it. Off-domain inputs stay the author's responsibility, but
    they must not silently invert a sign.
    """
    assert monod(-3.0, 1.0) == 0.0  # denom = -2
    assert monod(5.0, -5.0) == 0.0  # denom == 0, nonzero numerator: no inf


def test_monod_does_not_clamp_its_substrate() -> None:
    """The kernel is mirrored; the frozen function's ``max(0, ·)`` arg prep is NOT.

    A silent ``max(0, S)`` would make ``monod(a - b, k)`` quietly mean
    ``monod(max(0, a - b), k)``. Here ``0.5 - 2.0 = -1.5`` with ``K = 1.0`` gives a
    negative denominator → 0 by the guard, which is *reachable and explicable*, rather
    than a clamp the author cannot see. (Advisor correction; see the plan.)
    """
    difference = 0.5 - 2.0
    assert monod(difference, 1.0) == 0.0
    # ...and with a K large enough to keep the denominator positive, the negative
    # substrate is passed through honestly rather than clamped to zero.
    assert monod(-1.5, 2.0) == -1.5 / 0.5


# --- the parser ------------------------------------------------------------
def test_parse_monod_arg_order_is_substrate_then_half_saturation() -> None:
    """The frozen semantic choice, pinned. Matches ``oxygen_limitation_factor``'s
    signature and Michaelis-Menten convention; a parse vector pins it cross-port."""
    assert parse_rate_expr('monod(stock("sim.s"), param("k"))') == Monod(
        StockRef(StockId("sim.s")), ParamRef("k")
    )


def test_parse_monod_takes_full_subexpressions_not_just_refs() -> None:
    ast = parse_rate_expr('monod(stock("sim.s") - param("floor"), param("k") * 2.0)')
    assert isinstance(ast, Monod)
    assert ast.substrate == parse_rate_expr('stock("sim.s") - param("floor")')
    assert ast.half_saturation == parse_rate_expr('param("k") * 2.0')


def test_parse_monod_composes_with_the_frozen_arithmetic() -> None:
    """``Vmax · monod(S, K)`` — the 2-arg form's whole justification: Vmax arrives
    through the already-frozen ``*``, so no 3-arg form is needed."""
    ast = parse_rate_expr('param("vmax") * monod(stock("sim.s"), param("k"))')
    assert ast == parse_rate_expr('param("vmax") * (monod(stock("sim.s"), param("k")))')


def test_parse_monod_nests() -> None:
    ast = parse_rate_expr('monod(monod(n, param("a")), param("b"))')
    assert isinstance(ast, Monod)
    assert isinstance(ast.substrate, Monod)


def test_render_monod_round_trips() -> None:
    for text in (
        'monod(stock("sim.s"), param("k"))',
        'param("vmax") * monod(stock("sim.s"), param("k"))',
        'monod(monod(n, param("a")), param("b"))',
        "monod(-n, 1.0 + 2.0)",
    ):
        ast = parse_rate_expr(text)
        assert parse_rate_expr(render_rate_expr(ast)) == ast


@pytest.mark.parametrize(
    "text",
    [
        "monod(n)",  # arity: one arg
        "monod(n, n, n)",  # arity: three args
        "monod()",  # arity: none
        "monod(n n)",  # missing comma
        "monod(n,)",  # dangling comma
        "monod n, n",  # missing parens
        "n, n",  # a bare comma at top level is NOT a sequencing operator
        "monod(n, n",  # unbalanced
        ",",  # a lone comma
    ],
)
def test_parse_rejects_monod_misuse(text: str) -> None:
    """Arity and the comma's confinement. Both under- and over-application error, so
    the arity cannot be got wrong silently; the comma is legal nowhere but the call."""
    with pytest.raises(AuthoringError):
        parse_rate_expr(text)


def test_division_is_still_rejected() -> None:
    """``monod`` landed WITHOUT lifting the ``/`` deferral — it guards its own
    denominator, so it resolves ``x/0`` internally and never exposes the raw form."""
    with pytest.raises(AuthoringError):
        parse_rate_expr("n / n")


def test_the_rest_of_the_function_set_is_still_deferred() -> None:
    for text in ("exp(n)", "ln(n)", "sqrt(n)", "abs(n)", "min(n, n)", "clamp(n, n, n)"):
        with pytest.raises(AuthoringError):
            parse_rate_expr(text)


# --- the AST walks (the silent-fallback trap) ------------------------------
def test_prefixing_reaches_inside_a_monod_subtree() -> None:
    """The walk that used to ``return node`` on an unhandled type.

    Had ``Monod`` been missed, a prefixed bundle's ``monod(stock("o2"), …)`` would keep
    the *unprefixed* id — resolving to the wrong stock, or a ``KeyError`` at step 1.
    Silent wrong answers are exactly what this platform must not ship.
    """
    ast = parse_rate_expr('monod(stock("o2"), param("k")) * forcing("q")')
    prefixed = _prefix_expr_refs(ast, "cabin")
    assert prefixed == parse_rate_expr(
        'monod(stock("cabin.o2"), param("k")) * forcing("cabin.q")'
    )


def test_prefixing_leaves_a_monod_param_ref_alone() -> None:
    """``param(…)`` names a *frozen* set two instances correctly share — never
    prefixed, inside a monod as anywhere else."""
    prefixed = _prefix_expr_refs(parse_rate_expr('monod(stock("o2"), param("k"))'), "a")
    assert prefixed == parse_rate_expr('monod(stock("a.o2"), param("k"))')


def test_ref_collection_reaches_inside_a_monod_subtree() -> None:
    """The walk that used to fall through silently — a missed ``Monod`` would skip
    build-time referential validation, downgrading a clean ``AuthoringError`` into a
    runtime ``KeyError``."""
    params: set[str] = set()
    stocks: set[StockId] = set()
    _collect_refs(parse_rate_expr('monod(stock("sim.s"), param("k"))'), params, stocks)
    assert params == {"k"}
    assert stocks == {StockId("sim.s")}


def test_monod_in_a_template_expression_is_rejected_precisely() -> None:
    """Rate-only (a deliberate, reversible deferral) — but the error must not lie
    about a stock/forcing/n the author never wrote."""
    with pytest.raises(AuthoringError, match="monod"):
        eval_numeric_field("monod(param('a'), param('b'))", {"a": 1.0}, where="amount")
