"""Parse a rate-expression string into a :mod:`simcore.expr` AST (the boundary).

Parsing text → AST is a one-time **boundary** act (decision A): it runs when a
scenario file is interpreted, never per step, so it lives here in ``src/authoring``
and not in the pure core. The core only *evaluates* the AST this produces.

The grammar is the bounded arithmetic core Step 2 ships (see
:mod:`simcore.expr` for the closed-set rationale and the deferred ops). A tiny
recursive-descent parser with **explicitly pinned precedence and associativity**
(the *only* Tier-0 parse-parity risk surface the Step-4 Rust parser must match
exactly):

    expr    := term  (("+" | "-") term)*         # left-associative
    term    := factor ("*" factor)*              # left-associative
    factor  := "-" factor | primary              # unary minus binds tighter than "*"
    primary := number
             | "stock"   "(" string ")"
             | "param"   "(" string ")"
             | "forcing" "(" string ")"
             | "n"
             | "(" expr ")"

``number`` is standard decimal/float syntax (``1``, ``1.5``, ``1.0e-8`` — parsed by
*this* parser, so the YAML-1.1 dotless-``1e-3``-is-a-string hazard does not apply
here). A ``stock``/``param``/``forcing`` argument is a **quoted string** id/name, so
any dotted id (``power.battery``) is expressible without an identifier sub-grammar.

Every malformed input (unknown identifier, bad token, unbalanced parens, trailing
junk, an unsupported operator such as ``/``) raises :class:`AuthoringError` with the
offending position — a scenario file's rate string is author input, validated at the
boundary like the rest of the schema.
"""

from __future__ import annotations

from dataclasses import dataclass

from authoring.errors import AuthoringError
from simcore.expr import BinOp, Const, Expr, ForcingRef, Neg, ParamRef, StepN, StockRef
from simcore.ids import StockId

# The three reference forms and the bare ``n`` token — the closed identifier set.
_REF_KEYWORDS: frozenset[str] = frozenset({"stock", "param", "forcing"})


@dataclass(frozen=True)
class _Token:
    kind: str  # "num" | "ident" | "str" | "+" | "-" | "*" | "(" | ")"
    value: str
    pos: int


def _tokenize(text: str) -> list[_Token]:
    """Split ``text`` into tokens; a stray character is an ``AuthoringError``."""
    tokens: list[_Token] = []
    i = 0
    n = len(text)
    while i < n:
        c = text[i]
        if c.isspace():
            i += 1
            continue
        if c in "+-*()":
            tokens.append(_Token(c, c, i))
            i += 1
            continue
        if c in "'\"":
            # A quoted string literal (the ref argument). No escapes: an id/var name
            # is simple ASCII, so the string ends at the next matching quote.
            end = text.find(c, i + 1)
            if end == -1:
                raise AuthoringError(
                    f"unterminated string literal at position {i} in {text!r}"
                )
            tokens.append(_Token("str", text[i + 1 : end], i))
            i = end + 1
            continue
        if c.isdigit() or c == ".":
            start = i
            # A permissive numeric scan (digits, one dot, an ``e``/``E`` exponent with
            # optional sign); ``float()`` is the final arbiter of validity.
            while i < n and (text[i].isdigit() or text[i] in ".eE+-"):
                # Only consume a sign when it is part of an exponent (prev char e/E).
                if text[i] in "+-" and (i == start or text[i - 1] not in "eE"):
                    break
                i += 1
            literal = text[start:i]
            try:
                float(literal)  # validate here for a precise position in the error
            except ValueError as exc:
                raise AuthoringError(
                    f"invalid number {literal!r} at position {start} in {text!r}"
                ) from exc
            tokens.append(_Token("num", literal, start))
            continue
        if c.isalpha() or c == "_":
            start = i
            while i < n and (text[i].isalnum() or text[i] == "_"):
                i += 1
            tokens.append(_Token("ident", text[start:i], start))
            continue
        raise AuthoringError(
            f"unexpected character {c!r} at position {i} in {text!r} "
            f"(the rate grammar is arithmetic + stock/param/forcing/n only)"
        )
    return tokens


class _Parser:
    """A single-use recursive-descent parser over the token stream."""

    def __init__(self, tokens: list[_Token], text: str) -> None:
        self._tokens = tokens
        self._text = text
        self._i = 0

    def _peek(self) -> _Token | None:
        return self._tokens[self._i] if self._i < len(self._tokens) else None

    def _advance(self) -> _Token:
        tok = self._tokens[self._i]
        self._i += 1
        return tok

    def _error(self, msg: str) -> AuthoringError:
        return AuthoringError(f"{msg} in rate expression {self._text!r}")

    def parse(self) -> Expr:
        expr = self._expr()
        extra = self._peek()
        if extra is not None:
            raise self._error(
                f"unexpected trailing token {extra.value!r} at position {extra.pos}"
            )
        return expr

    def _expr(self) -> Expr:
        node = self._term()
        while True:
            tok = self._peek()
            if tok is not None and tok.kind in ("+", "-"):
                self._advance()
                node = BinOp(tok.kind, node, self._term())
            else:
                return node

    def _term(self) -> Expr:
        node = self._factor()
        while True:
            tok = self._peek()
            if tok is not None and tok.kind == "*":
                self._advance()
                node = BinOp("*", node, self._factor())
            else:
                return node

    def _factor(self) -> Expr:
        tok = self._peek()
        if tok is not None and tok.kind == "-":
            self._advance()
            return Neg(self._factor())
        return self._primary()

    def _primary(self) -> Expr:
        tok = self._peek()
        if tok is None:
            raise self._error("unexpected end of expression (expected a value)")
        if tok.kind == "num":
            self._advance()
            return Const(float(tok.value))
        if tok.kind == "(":
            self._advance()
            node = self._expr()
            closer = self._peek()
            if closer is None or closer.kind != ")":
                raise self._error(f"missing ')' (opened at position {tok.pos})")
            self._advance()
            return node
        if tok.kind == "ident":
            return self._ident(tok)
        raise self._error(f"unexpected token {tok.value!r} at position {tok.pos}")

    def _ident(self, tok: _Token) -> Expr:
        self._advance()
        if tok.value == "n":
            return StepN()
        if tok.value in _REF_KEYWORDS:
            arg = self._call_string_arg(tok)
            if tok.value == "stock":
                return StockRef(StockId(arg))
            if tok.value == "param":
                return ParamRef(arg)
            return ForcingRef(arg)
        raise self._error(
            f"unknown identifier {tok.value!r} at position {tok.pos} "
            f"(expected a number, 'n', or stock/param/forcing(\"…\"))"
        )

    def _call_string_arg(self, keyword: _Token) -> str:
        """Consume ``( "string" )`` after a ``stock``/``param``/``forcing`` keyword."""
        opener = self._peek()
        if opener is None or opener.kind != "(":
            raise self._error(
                f"expected '(' after {keyword.value!r} at position {keyword.pos}"
            )
        self._advance()
        arg = self._peek()
        if arg is None or arg.kind != "str":
            raise self._error(
                f"{keyword.value}(…) takes a quoted string argument "
                f"(at position {keyword.pos})"
            )
        self._advance()
        closer = self._peek()
        if closer is None or closer.kind != ")":
            raise self._error(
                f"missing ')' after {keyword.value}(…) at position {keyword.pos}"
            )
        self._advance()
        if not arg.value:
            raise self._error(
                f"{keyword.value}(…) argument is empty (at position {keyword.pos})"
            )
        return arg.value


def parse_rate_expr(text: str) -> Expr:
    """Parse a rate-expression string into a :mod:`simcore.expr` AST.

    The single public entry point. Raises :class:`AuthoringError` for any malformed
    input (empty expression, stray character, unknown identifier, unbalanced parens,
    an unsupported operator, trailing junk).
    """
    tokens = _tokenize(text)
    if not tokens:
        raise AuthoringError(f"empty rate expression {text!r}")
    return _Parser(tokens, text).parse()
