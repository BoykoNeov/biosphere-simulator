"""Scoped, in-memory parameter overrides for offline experiments.

**What this is for.** Answering "what would source B's number do?" without editing a
param file. A value substituted here flows through the *entire* real boundary — pydantic
schema validation, the pint unit guards, the bound checks, and any folding a loader does
(``load_canopy_params`` folds ``specific_leaf_area`` and ``carbon_fraction`` into
``sla_per_mol_c``) — because the substitution happens on the **parsed YAML mapping**,
before validation, rather than on the finished dataclass. Overriding a folded *input* is
therefore possible; overriding the finished dataclass could not reach one.

**What this is NOT.** Not a calibration mechanism and not a tuning knob. It writes no
file, so no param sha-256 moves in either freeze manifest, no golden moves, and nothing
here can become a committed change by accident. *"Changing the number is calibration, a
separate act with its own unfreeze discipline"* (``docs/param-file-conventions.md``) —
this exists so an experiment does not have to look like one.

Rationale, the priced alternatives, and the reporting requirements a harness built on
this must satisfy: ``docs/plans/post-roadmap-value-switch-harness.md``.

Usage::

    from config.overrides import param_overrides

    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        state, registry = build_season(scenario)   # runs as if canopy.yaml said 0.65

⚠ **THE ANTI-VACUITY GUARANTEE — the reason this exists instead of a patch script.**
On exit the context manager raises ``UnusedOverrideError`` unless **every** override was
actually consumed by a load. A probe that shims a path nothing calls runs clean, reads
the baseline, and reports "no effect" as a finding — this repo has shipped exactly that
defect (``cc44b41``, a cross-port gate gone vacuous because its probe shimmed a dead
``exp``). A silent no-op is made structurally impossible here rather than left to care.
"""

from __future__ import annotations

from collections.abc import Iterator, Mapping
from contextlib import contextmanager
from contextvars import ContextVar
from pathlib import Path
from typing import Any

from config.errors import ConfigError

__all__ = [
    "OverrideError",
    "UnusedOverrideError",
    "param_overrides",
    "apply_overrides",
]


class OverrideError(ConfigError):
    """An override could not be applied to the file it names."""


class UnusedOverrideError(OverrideError):
    """An override was declared but never consumed — the vacuity guard firing."""


# The active overrides, keyed by param-file STEM:
#     {"canopy": {"extinction_coef": 0.65}}
#
# ⚠ Keyed by stem, not by full path, and that is a deliberate limitation worth knowing:
# a crop override directory holds files with the SAME stems as the reference
# (``params/crops/potato/canopy.yaml`` and ``params/canopy.yaml`` are both "canopy"), so
# an override reaches whichever file the crop resolved to. That is the right default for
# "run this crop as if its canopy params said X" and is unable to express "override the
# reference but not the crop". Recorded rather than discovered.
#
# A ContextVar rather than a module global so nesting, threads and async contexts each
# see their own value; the frozen runs never set it and read ``None``.
_ACTIVE: ContextVar[dict[str, dict[str, float]] | None] = ContextVar(
    "param_overrides_active", default=None
)
_CONSUMED: ContextVar[set[tuple[str, str]] | None] = ContextVar(
    "param_overrides_consumed", default=None
)


@contextmanager
def param_overrides(
    overrides: Mapping[str, Mapping[str, float]],
) -> Iterator[None]:
    """Substitute param values for the duration of the block. Writes nothing.

    ``overrides`` maps a param-file stem to ``{param_name: value}`` — e.g.
    ``{"canopy": {"extinction_coef": 0.65}, "decomposition": {"k_litter": 0.02}}``.
    Only the ``value`` is replaced; the declared ``unit`` and ``source`` are kept, so
    the unit guards still run against the file's real declared unit.

    Raises ``ValueError`` for an empty override set (a no-op block is a mistake, not a
    control), ``OverrideError`` if a named param is absent from the file it names, and
    ``UnusedOverrideError`` on exit if any override was never consumed.

    Nesting replaces rather than merges: an inner block's overrides are the only ones
    active inside it, and the outer set is restored on exit. Merging silently would make
    "which value was live" depend on lexical nesting, which is the kind of thing an
    experiment must not have to reason about.
    """
    if not overrides:
        raise ValueError(
            "param_overrides({}) is a no-op block. An empty override set almost always "
            "means a variant list was built wrong; if you want the baseline, run "
            "without the context manager."
        )
    frozen = {
        stem: dict(values) for stem, values in overrides.items() if values is not None
    }
    for stem, values in frozen.items():
        if not values:
            raise ValueError(
                f"param_overrides: {stem!r} maps to an empty dict — nothing to do."
            )

    active_token = _ACTIVE.set(frozen)
    consumed_token = _CONSUMED.set(set())
    try:
        yield
        # ⚠ Only on the success path. If the body raised, the exception is the finding
        # and an unused-override complaint layered on top of it would bury the cause.
        unused = sorted(
            (stem, name)
            for stem, values in frozen.items()
            for name in values
            if (stem, name) not in (_CONSUMED.get() or set())
        )
        if unused:
            raise UnusedOverrideError(
                "these overrides were never consumed by any load: "
                + ", ".join(f"{stem}.{name}" for stem, name in unused)
                + ". The run therefore measured the BASELINE while reporting itself as "
                "a variant. Usual causes: the stem does not match the file's name "
                "(it is the filename stem, e.g. 'canopy' for canopy.yaml), or the code "
                "path under test never loads that file for this scenario. This guard "
                "exists because a probe that shims a dead path reports 'no effect' and "
                "looks exactly like a real result."
            )
    finally:
        _ACTIVE.reset(active_token)
        _CONSUMED.reset(consumed_token)


def apply_overrides(path: Path, data: dict[str, Any]) -> dict[str, Any]:
    """Apply any active overrides for ``path`` to a freshly parsed YAML mapping.

    Called by ``config.loader.load_yaml``. Returns ``data`` unchanged — the same object,
    not a copy — when no overrides are active, so the frozen runs pay one dict lookup
    and nothing else.
    """
    active = _ACTIVE.get()
    if not active:
        return data
    values = active.get(Path(path).stem)
    if not values:
        return data

    params = data.get("parameters")
    if not isinstance(params, dict):
        raise OverrideError(
            f"cannot override {Path(path).name}: it has no top-level 'parameters' "
            f"mapping (got {type(params).__name__}). Overrides address the "
            f"value/unit/source param-file shape described in "
            f"docs/param-file-conventions.md."
        )

    patched = dict(data)
    patched_params = dict(params)
    consumed = _CONSUMED.get()
    stem = Path(path).stem
    for name, value in values.items():
        entry = patched_params.get(name)
        if not isinstance(entry, dict) or "value" not in entry:
            known = sorted(k for k, v in patched_params.items() if isinstance(v, dict))
            raise OverrideError(
                f"cannot override {stem}.{name}: {Path(path).name} has no parameter "
                f"{name!r} with a 'value' field. Known parameters: {known}."
            )
        patched_params[name] = {**entry, "value": value}
        if consumed is not None:
            consumed.add((stem, name))
    patched["parameters"] = patched_params
    return patched
