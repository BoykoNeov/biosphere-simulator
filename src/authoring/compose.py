"""File composition: merge included **domain/species bundles** into a scenario (Step 6).

A scenario ``includes`` a list of bundle-file paths
(:class:`authoring.schema.BundleSpec`); :func:`apply_includes` reads each and merges its
``parameters`` / ``stocks`` / ``flows`` / ``forcings`` into the scenario's, producing a
single **flat** :class:`ScenarioSpec` the interpreter lowers exactly as before. This is
the "authored, not programmed" payoff one level up: a station is *composed* from
reusable bundles (a crew species, a battery domain) rather than re-declared inline.

**Merge semantics** (cross-port — Step 6b's Rust port + Step 7's freeze inherit them):

- **Order:** included bundles first (in ``includes`` order), then the scenario's own
  inline declarations. So a scenario that is *only* an include reproduces the bundle's
  declaration order exactly (the single-bundle byte-identity anchor).
- **No silent override:** a duplicate stock id, flow id, forcing key, or parameter name
  across any two sources is an :class:`AuthoringError` (``extra="forbid"``, a level up).
  Disjoint-domain composition (crew + battery) needs no id-namespacing; **multi-instance
  prefixing is deferred** — only *forced* by two instances of one bundle (Step 6c).
- **Flat, one level deep:** a bundle carries no ``includes`` of its own (rejected by
  ``BundleSpec``'s ``extra="forbid"``) — nested includes are out of scope.
- **Run config lives only in the top-level scenario** (a bundle has no integrator/dt/
  steps — also enforced by ``BundleSpec``).

**Parameter packs inside an included bundle are deferred** (advisor-endorsed scope,
matching Step 1's "full-file packs only" and the Rust port's "packs deferred"): a pack
path in a bundle would have to resolve against *the bundle's* directory, which needs
per-flow source-dir threading no Step-6 anchor exercises. A ``{pack: …}`` reference on a
bundle flow is a clean :class:`AuthoringError`; a bundle flow names a frozen param set
instead. (Top-level scenario flows resolve packs against the scenario dir — Step 1.)
"""

from __future__ import annotations

from pathlib import Path

from authoring.errors import AuthoringError
from authoring.schema import (
    BundleSpec,
    FlowSpec,
    ForcingSpec,
    ParamPackRef,
    ScenarioSpec,
    StockSpec,
)
from config import ConfigError, load_yaml


def apply_includes(spec: ScenarioSpec, base_dir: Path) -> ScenarioSpec:
    """Merge ``spec.includes`` bundle files into a flat :class:`ScenarioSpec`.

    Returns ``spec`` unchanged if it has no includes (the pre-Step-6 path, so existing
    scenarios and bare specs are untouched). Otherwise reads each bundle relative to
    ``base_dir`` (the scenario file's directory), validates it, and returns a new spec
    with the merged parameters/stocks/flows/forcings and an emptied ``includes`` — which
    the interpreter then lowers identically to a hand-flattened scenario.
    """
    if not spec.includes:
        return spec

    parameters: dict[str, float] = {}
    stocks: list[StockSpec] = []
    flows: list[FlowSpec] = []
    forcings: dict[str, ForcingSpec] = {}
    stock_ids: dict[str, str] = {}  # id -> source label (for the collision message)
    flow_ids: dict[str, str] = {}

    def merge(
        source: str,
        params: dict[str, float],
        sspecs: list[StockSpec],
        fspecs: list[FlowSpec],
        fcs: dict[str, ForcingSpec],
    ) -> None:
        for name, value in params.items():
            if name in parameters:
                raise AuthoringError(
                    f"duplicate parameter {name!r}: declared by both {source} and an "
                    f"earlier source (parameters share one flat namespace across "
                    f"includes; rename or use 'overrides')"
                )
            parameters[name] = value
        for st in sspecs:
            if st.id in stock_ids:
                raise AuthoringError(
                    f"duplicate stock id {st.id!r}: declared by both {source} and "
                    f"{stock_ids[st.id]}"
                )
            stock_ids[st.id] = source
            stocks.append(st)
        for fl in fspecs:
            if fl.id in flow_ids:
                raise AuthoringError(
                    f"duplicate flow id {fl.id!r}: declared by both {source} and "
                    f"{flow_ids[fl.id]}"
                )
            if source != "the scenario" and isinstance(fl.params, ParamPackRef):
                raise AuthoringError(
                    f"flow {fl.id!r} in {source}: parameter packs inside an included "
                    f"bundle are deferred (a bundle pack must resolve against the "
                    f"bundle's directory); name a frozen param set instead"
                )
            flow_ids[fl.id] = source
            flows.append(fl)
        for key, forcing in fcs.items():
            if key in forcings:
                raise AuthoringError(
                    f"duplicate forcing {key!r}: declared by both {source} and an "
                    f"earlier source"
                )
            forcings[key] = forcing

    for inc in spec.includes:
        bundle = _load_bundle(base_dir / inc, inc)
        merge(
            f"bundle {inc!r}",
            bundle.parameters,
            bundle.stocks,
            bundle.flows,
            bundle.forcings,
        )
    merge("the scenario", spec.parameters, spec.stocks, spec.flows, spec.forcings)

    return spec.model_copy(
        update={
            "includes": [],
            "parameters": parameters,
            "stocks": stocks,
            "flows": flows,
            "forcings": forcings,
        }
    )


def _load_bundle(path: Path, label: str) -> BundleSpec:
    """Read + validate one bundle (``extra="forbid"`` — no run config, no nesting)."""
    try:
        raw = load_yaml(str(path))
    except ConfigError as exc:
        raise AuthoringError(
            f"included bundle {label!r} could not be read from {path}: {exc}"
        ) from exc
    return BundleSpec.model_validate(raw)
