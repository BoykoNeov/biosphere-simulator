"""Port-agnostic cross-port snapshot comparator (Phase-7 Step 0, P7.0).

Compares a Python reference golden against *any* port's JSON snapshot (Rust today,
C# at Phase 8) by **parsed f64 values**, never JSON bytes — so a port need only
emit valid JSON `sim_io.loads` accepts, not match `json.dumps`'s string spelling
(advisor #5). It applies the 3-tier parity contract, reading each scenario's tier
from `tiers.json`:

* **Tier 0 (always, EXACT):** the structural/discrete skeleton — schema `version`,
  the integer `n`, `rng_seed`, the stock-id set, each stock's discrete fields
  (`domain`/`quantity`/`unit`/`kind`/`unclamped`) and composition key set, and any
  summary stability booleans (`is_period_2`, `is_stationary`). Enforced regardless
  of the float tier; a mismatch here is a hard parity failure.
* **Tier 1 (bit-exact):** every numeric leaf must match to the IEEE-754 bit — for
  transcendental-free scenarios whose graph is only `+ - * /` (deterministic,
  correctly-rounded across ports).
* **Tier 2 (measured band):** numeric leaves may differ within a *measured*
  relative band (the aggregate `max_abs_relative_deviation` over all numeric
  leaves), for scenarios a transcendental (`sin`/`exp`/`**`/…) touches. The band and
  floor are supplied by the caller (from `tiers.json`, filled in a later step once
  the Rust port produces numbers) — Tier-2 tolerances are measured, never derived,
  so this module refuses to invent one.

The comparator walks both JSON trees structurally (dict key sets and list lengths
must match — that is part of Tier 0), classifying each scalar leaf as discrete
(exact) or numeric (per-tier). It handles both the `state` goldens (18 of 20) and
the two `drift_summary` goldens uniformly.

Python-side tooling — imports `lab.oracle_match` for the band metric; that module is
deliberately NOT ported to Rust (the port ships the engine, not the analysis lab).
"""

from __future__ import annotations

import json
import struct
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from lab.oracle_match import max_abs_relative_deviation

# Float-trajectory tiers (the Tier-0 structural gate is not a float tier — it is
# enforced unconditionally).
TIER_1_BIT_EXACT = 1
TIER_2_BAND = 2

# String-valued fields that are discrete/structural and must match EXACTLY at every
# tier — never treated as numeric hex-float leaves even though some (`rng_seed`)
# superficially parse as one.
DISCRETE_STRING_KEYS = frozenset(
    {"id", "domain", "quantity", "unit", "kind", "rng_seed"}
)


@dataclass
class Diff:
    """One parity discrepancy, with a JSON-path locator."""

    path: str
    detail: str

    def __str__(self) -> str:
        return f"{self.path}: {self.detail}"


@dataclass
class CompareResult:
    """The outcome of a comparison: structural diffs, numeric leaf pairs, verdict."""

    tier: int
    structural_diffs: list[Diff] = field(default_factory=list)
    numeric_diffs: list[Diff] = field(default_factory=list)
    # (path, reference_value, candidate_value) for every numeric leaf compared.
    numeric_pairs: list[tuple[str, float, float]] = field(default_factory=list)
    max_rel_dev: float | None = None

    @property
    def ok(self) -> bool:
        return not self.structural_diffs and not self.numeric_diffs

    def report(self) -> str:
        if self.ok:
            extra = (
                f" (max rel dev {self.max_rel_dev:.3e})"
                if self.max_rel_dev is not None
                else ""
            )
            return f"OK at tier {self.tier}{extra}"
        lines = [f"PARITY FAILURE at tier {self.tier}:"]
        for d in self.structural_diffs:
            lines.append(f"  [structural] {d}")
        for d in self.numeric_diffs:
            lines.append(f"  [numeric]    {d}")
        return "\n".join(lines)


def _bits(x: float) -> bytes:
    """The 8 IEEE-754 bytes of a double — the bit-exact identity key."""
    return struct.pack("<d", x)


def _looks_like_hexfloat(s: str) -> bool:
    """Whether a bare string leaf is a hex-float amount (vs a discrete label)."""
    try:
        float.fromhex(s)
    except (ValueError, TypeError):
        return False
    return True


def _as_number(value: Any, key: str | None) -> float | None:
    """Interpret a scalar leaf as a numeric value, or `None` if it is discrete.

    A plain JSON number is numeric. A string is numeric only if it is a hex-float
    *and* its key is not in the discrete denylist (so `rng_seed`'s `0x0` stays a
    discrete label). Booleans (a subtype of `int` in Python) are always discrete.
    """
    if isinstance(value, bool):
        return None
    if isinstance(value, (int,)):
        return None  # step counts, versions, horizons — discrete/exact
    if isinstance(value, float):
        return value
    if isinstance(value, str):
        if key in DISCRETE_STRING_KEYS:
            return None
        if _looks_like_hexfloat(value):
            return float.fromhex(value)
        return None
    return None


def _walk(
    ref: Any,
    cand: Any,
    path: str,
    key: str | None,
    structural: list[Diff],
    numeric_pairs: list[tuple[str, float, float]],
) -> None:
    """Recursively align two JSON trees, collecting structural diffs and numeric
    leaf pairs. Structure (dict key sets, list membership) must match exactly."""
    # Type-shape agreement is structural.
    if isinstance(ref, Mapping) != isinstance(cand, Mapping):
        structural.append(
            Diff(path, f"type mismatch: {type(ref).__name__} vs {type(cand).__name__}")
        )
        return
    if _is_list(ref) != _is_list(cand):
        structural.append(
            Diff(path, f"type mismatch: {type(ref).__name__} vs {type(cand).__name__}")
        )
        return

    if isinstance(ref, Mapping):
        ref_keys, cand_keys = set(ref), set(cand)
        for k in sorted(ref_keys - cand_keys):
            structural.append(
                Diff(f"{path}.{k}", "present in reference, missing in candidate")
            )
        for k in sorted(cand_keys - ref_keys):
            structural.append(
                Diff(f"{path}.{k}", "present in candidate, missing in reference")
            )
        for k in sorted(ref_keys & cand_keys):
            _walk(ref[k], cand[k], f"{path}.{k}", k, structural, numeric_pairs)
        return

    if _is_list(ref):
        # A list of id-bearing dicts (e.g. `stocks`) is matched by id, so ordering
        # and any missing/extra member is reported precisely and order-independently.
        if _is_id_list(ref) and _is_id_list(cand):
            ref_by_id = {item["id"]: item for item in ref}
            cand_by_id = {item["id"]: item for item in cand}
            for sid in sorted(set(ref_by_id) - set(cand_by_id)):
                structural.append(
                    Diff(
                        f"{path}[id={sid}]",
                        "stock present in reference, missing in candidate",
                    )
                )
            for sid in sorted(set(cand_by_id) - set(ref_by_id)):
                structural.append(
                    Diff(
                        f"{path}[id={sid}]",
                        "stock present in candidate, missing in reference",
                    )
                )
            for sid in sorted(set(ref_by_id) & set(cand_by_id)):
                _walk(
                    ref_by_id[sid],
                    cand_by_id[sid],
                    f"{path}[id={sid}]",
                    None,
                    structural,
                    numeric_pairs,
                )
            return
        # Otherwise positional: lengths must match (structural).
        if len(ref) != len(cand):
            structural.append(Diff(path, f"list length {len(ref)} vs {len(cand)}"))
            return
        for i, (r, c) in enumerate(zip(ref, cand, strict=True)):
            _walk(r, c, f"{path}[{i}]", key, structural, numeric_pairs)
        return

    # Scalar leaf.
    ref_num = _as_number(ref, key)
    cand_num = _as_number(cand, key)
    if ref_num is None or cand_num is None:
        # Discrete leaf: exact equality required (Tier 0).
        if ref != cand:
            structural.append(Diff(path, f"discrete mismatch: {ref!r} vs {cand!r}"))
        return
    numeric_pairs.append((path, ref_num, cand_num))


def _is_list(x: Any) -> bool:
    return isinstance(x, Sequence) and not isinstance(x, (str, bytes))


def _is_id_list(x: Any) -> bool:
    return (
        _is_list(x)
        and len(x) > 0
        and all(isinstance(i, Mapping) and "id" in i for i in x)
    )


def compare(
    reference: Mapping[str, Any],
    candidate: Mapping[str, Any],
    *,
    tier: int,
    band: float | None = None,
    floor: float | None = None,
) -> CompareResult:
    """Compare two parsed snapshots under the given float `tier`.

    Tier 0 (structural/discrete) is always enforced. Tier 1 additionally requires
    every numeric leaf to be bit-exact. Tier 2 requires the aggregate
    `max_abs_relative_deviation` over all numeric leaves to be `<= band` (with the
    given positive `floor`); both must be provided — this module will not invent a
    tolerance (Tier-2 bands are measured, not derived).
    """
    if tier not in (TIER_1_BIT_EXACT, TIER_2_BAND):
        raise ValueError(f"unknown tier {tier!r}; expected 1 or 2")

    result = CompareResult(tier=tier)
    _walk(
        reference, candidate, "$", None, result.structural_diffs, result.numeric_pairs
    )

    if tier == TIER_1_BIT_EXACT:
        for path, r, c in result.numeric_pairs:
            if _bits(r) != _bits(c):
                result.numeric_diffs.append(
                    Diff(path, f"bit mismatch: {r.hex()} vs {c.hex()}")
                )
        return result

    # Tier 2: measured relative band over all numeric leaves.
    if band is None or floor is None:
        raise ValueError(
            "Tier-2 comparison requires an explicit measured `band` and `floor`; "
            "they are null in tiers.json until calibrated against actual port output"
        )
    if not result.numeric_pairs:
        return result
    refs = [r for _, r, _ in result.numeric_pairs]
    cands = [c for _, _, c in result.numeric_pairs]
    result.max_rel_dev = max_abs_relative_deviation(refs, cands, floor=floor)
    if result.max_rel_dev > band:
        # Attribute the failure to the worst leaf for a readable diagnostic.
        worst = max(
            result.numeric_pairs,
            key=lambda t: abs(t[2] - t[1]) / max(abs(t[1]), floor),
        )
        path, r, c = worst
        result.numeric_diffs.append(
            Diff(
                path,
                f"relative deviation {result.max_rel_dev:.3e} exceeds band {band:.3e} "
                f"(worst leaf: {r!r} vs {c!r})",
            )
        )
    return result


def load_json(path: str | Path) -> dict[str, Any]:
    """Parse a snapshot/summary JSON file (a golden or a port's output)."""
    return json.loads(Path(path).read_text(encoding="utf-8"))
