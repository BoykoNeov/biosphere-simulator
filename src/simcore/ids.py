"""Stable, canonical-sortable identifiers and the canonical-unit label type.

These are thin ``NewType`` wrappers over ``str`` so the type checker keeps a
``StockId`` from being passed where a ``FlowId`` is expected, while the runtime
representation stays a plain string. Strings are chosen because they are
*canonically sortable* (decision #7/#15: every reduction runs in a deterministic,
id-sorted order) and trivially serializable and cross-port stable.

Pure stdlib — no third-party imports (the ``simcore`` purity rule).
"""

from typing import NewType

# Identifiers. Plain strings at runtime; distinct types under the checker.
StockId = NewType("StockId", str)
DomainId = NewType("DomainId", str)
FlowId = NewType("FlowId", str)

# Canonical-unit *label* (e.g. "mol", "kg", "J"). The core stores only the label
# as a plain string; dimensional validation/conversion via pint lives in the
# outer ``config`` loader, never here (decision #9). The Quantity -> UnitLabel
# table — the shared source of truth — lives in ``simcore.quantities``.
UnitLabel = NewType("UnitLabel", str)
