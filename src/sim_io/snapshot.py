"""State snapshot serialization: ``State`` ⇄ JSON, bit-exact and cross-port.

This is the outer-layer realization of the plan's "plain-data state snapshot +
round-trip via outer ``sim_io``" scope item and the "Serialization round-trip"
exit gate (JSON; hex-float in golden files for exact cross-run and cross-port
comparison). ``sim_io`` lives **outside** the pure core; it imports ``simcore``,
never the reverse.

Design (settled with advisor):

* **Floats are stored as hex-float strings** (``float.hex()`` / ``float.fromhex``)
  — ``amount`` and ``extinction_threshold``. This is lossless and bit-exact for
  *every* finite double (including ``-0.0`` and subnormals), and the C99 hex-float
  form is unambiguously parseable by the future Rust port. Storing floats as
  strings also keeps the JSON free of native ``NaN``/``Infinity`` tokens: state
  forbids non-finite amounts, and a tampered hex string like ``"inf"`` is rejected
  on reconstruction (see below).
* **``rng_seed`` is stored as a ``0x``-hex string**, not a JSON number. The seed is
  a full 64-bit value (``rng.py`` masks to 64 bits); a JSON *number* round-trips
  fine through Python's arbitrary-precision ``int`` but silently loses precision
  above 2**53 in a reader that parses JSON numbers as ``f64`` (Rust/JS). Decision
  #12 wants cross-port bit-identical streams, so the same exactness discipline that
  drives hex-float for amounts applies to the seed. ``n`` stays a native JSON int —
  it is a step count, nowhere near 2**53.
* **Stocks serialize as a list sorted by id** (canonical order, #15) rather than a
  JSON object: each stock already carries its own ``id``, so a list avoids any
  key/id divergence and gives a deterministic, byte-stable ordering on its own
  (independent of any insertion order). ``json.dumps(sort_keys=True)`` orders the
  *field* keys within each stock object but does not reorder the list, so the list
  is pre-sorted here.
* **``unit`` is serialized verbatim**, not re-derived from ``quantity`` on load —
  re-deriving would be a silent "fix" that breaks a faithful round-trip if the
  stored label ever diverged from the canonical table.
* **``aux`` (the non-conserved channel, P2) serializes as a key-sorted object of
  hex-float strings** — same exactness/canonical-order discipline as stock amounts
  (added at the v1→v2 bump; empty for any pre-aux run).
* **Reconstruction routes through ``Stock(...)`` / ``State(...)``**, so every core
  invariant re-fires for free: NaN/Inf rejection, the ``unclamped ⇒ BOUNDARY``
  guard, the key/id match, ``n >= 0``. A tampered golden therefore fails loudly at
  load rather than producing a malformed state.

The ``version`` marker is a deliberate, narrow exception to this project's
anti-speculation norm (cf. step 6's "speculative-param smell", step 8's "no
``ledger`` field — no consumer yet"): a serialization format is the rare place
forward-compat genuinely cannot be retrofitted — once goldens are frozen without a
marker, *adding* versioning later is itself a breaking change. So the constant is
recorded and an unknown version is **rejected** at parse (the same "fail loud at
first parse" stance used for referential integrity), but **no** migration or
multi-version dispatch machinery is built until a second version actually exists.
"""

import json
from collections.abc import Mapping
from typing import Any

from simcore.ids import DomainId, StockId, UnitLabel
from simcore.quantities import Quantity, StockKind
from simcore.state import State, Stock

# Snapshot schema version. Bump only when the on-disk shape changes; see the
# module docstring for why a marker exists at all (and why no migration machinery
# does yet). v2 adds the non-conserved ``aux`` channel (Phase-1 P2); v1 goldens are
# rejected outright (no migration) and were regenerated at the bump. v3 adds the
# per-stock element ``composition`` map (P2.1); v2 goldens were likewise
# regenerated (the diff is purely the added ``composition`` block — amounts stay
# bit-identical, the 1:1 fold being exact ×1.0).
SCHEMA_VERSION = 3


def _stock_to_dict(stock: Stock) -> dict[str, object]:
    """One ``Stock`` → plain JSON-able data (floats as hex strings)."""
    return {
        "id": str(stock.id),
        "domain": str(stock.domain),
        "quantity": stock.quantity.value,
        "unit": str(stock.unit),
        "amount": stock.amount.hex(),
        "kind": stock.kind.value,
        "extinction_threshold": stock.extinction_threshold.hex(),
        "unclamped": stock.unclamped,
        # Element composition (P2.1): a key-sorted object of hex-float coeffs,
        # keyed by quantity value. Same exactness/canonical-order discipline as
        # ``aux`` and amounts. A 1:1 stock serializes ``{"<its quantity>": "0x1p+0"}``.
        "composition": {
            q.value: coeff.hex()
            for q, coeff in sorted(
                stock.composition.items(), key=lambda kv: kv[0].value
            )
        },
    }


def _stock_from_dict(data: Mapping[str, Any]) -> Stock:
    """Plain data → ``Stock``, through the constructor (invariants re-fire)."""
    return Stock(
        id=StockId(data["id"]),
        domain=DomainId(data["domain"]),
        quantity=Quantity(data["quantity"]),
        unit=UnitLabel(data["unit"]),
        amount=float.fromhex(data["amount"]),
        kind=StockKind(data["kind"]),
        extinction_threshold=float.fromhex(data["extinction_threshold"]),
        unclamped=data["unclamped"],
        composition={
            Quantity(k): float.fromhex(v) for k, v in data["composition"].items()
        },
    )


def state_to_dict(state: State) -> dict[str, object]:
    """``State`` → plain JSON-able data.

    Stocks are emitted as a list sorted by id (canonical order, #15); the seed is a
    ``0x``-hex string (cross-port precision, see module docstring). The non-conserved
    ``aux`` channel (P2) is a key-sorted object of hex-float strings — the same
    exactness/canonical-order discipline as stock amounts (``-0.0`` and subnormals
    survive bit-for-bit; ``json.dumps(sort_keys=True)`` also sorts the keys, the
    pre-sort here makes the canonical order explicit at the source).
    """
    return {
        "version": SCHEMA_VERSION,
        "n": state.n,
        "rng_seed": hex(state.rng_seed),
        "stocks": [_stock_to_dict(state.stocks[sid]) for sid in sorted(state.stocks)],
        "aux": {name: state.aux[name].hex() for name in sorted(state.aux)},
    }


def state_from_dict(data: Mapping[str, Any]) -> State:
    """Plain data → ``State``, through the constructors (invariants re-fire).

    Rejects an unknown/missing schema version at parse (fail-loud; no migration
    machinery — see module docstring). ``rng_seed`` is parsed with base 0 so a
    ``0x``-hex (or plain decimal) string both work; the constructor revalidates the
    rest.
    """
    version = data.get("version")
    if version != SCHEMA_VERSION:
        raise ValueError(
            f"unsupported snapshot schema version {version!r}; this build writes "
            f"and reads version {SCHEMA_VERSION} only"
        )
    stocks = [_stock_from_dict(s) for s in data["stocks"]]
    aux = {name: float.fromhex(h) for name, h in data["aux"].items()}
    return State(
        n=data["n"],
        stocks={s.id: s for s in stocks},
        rng_seed=int(data["rng_seed"], 0),
        aux=aux,
    )


def dumps(state: State) -> str:
    """Serialize ``state`` to canonical JSON text (deterministic, byte-stable).

    ``sort_keys=True`` fixes field-key order within each object; the stock list is
    already id-sorted by ``state_to_dict``. A trailing newline is appended so the
    text is a well-formed file line. Pair with ``str.encode("utf-8")`` and a
    byte-wise compare for golden files — do not round-trip through text-mode file
    APIs, whose newline translation would desync the on-disk bytes from this output.
    """
    return json.dumps(state_to_dict(state), indent=2, sort_keys=True) + "\n"


def loads(text: str) -> State:
    """Parse canonical JSON text back into a ``State`` (inverse of ``dumps``)."""
    return state_from_dict(json.loads(text))
