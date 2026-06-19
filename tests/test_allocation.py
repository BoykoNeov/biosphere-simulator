"""Phase-1 Step-9 tests: leaf/stem/root allocation + senescence (organ-carbon flows).

The first internal-redistribution CARBON process and multi-organ stock structure.
Layers:

* **Pure split functions** (``domains.biosphere.allocation``, stdlib): DVS-keyed
  ``partition_fractions`` (knot literals, the sum-to-1 invariant across a DVS sweep,
  the out-of-range clamp), ``partition`` (an exact split of a given DMI), and
  ``senescence_flux`` (∝ organ carbon, → 0 at zero).
* **The assembled ``Senescence`` flow**: a carbon-balanced ``{organs → litter}``
  ``FlowResult`` with the hand-computed losses; dt-linear.
* **Config boundary** (``load_allocation_params`` / ``load_senescence_params``): the
  committed files load; a non-sum-1 row, non-increasing DVS, an out-of-range fraction /
  death rate, a bad unit, and a missing source are rejected.

The ``Allocation`` flow moved to ``domains.biosphere.carbon_budget`` at Step 11 (the
buffer dissolution; it now sources from ``co2_atmos``, not ``plant_c``) and is tested in
``test_carbon_budget.py``; the DVS-keyed **split functions** stay here.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.allocation import (
    AllocationParams,
    PartitionRow,
    Senescence,
    SenescenceParams,
    partition,
    partition_fractions,
    senescence_flux,
)
from domains.biosphere.loader import (
    ALLOCATION_PARAMS_PATH,
    SENESCENCE_PARAMS_PATH,
    load_allocation_params,
    load_senescence_params,
)
from simcore.environment import SourceResolver
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# Committed partition table (mirrors allocation.yaml; leaf/stem/root/storage fractions).
_TABLE = (
    PartitionRow(dvs=0.0, fl=0.55, fs=0.10, fr=0.35, fo=0.00),
    PartitionRow(dvs=1.0, fl=0.30, fs=0.50, fr=0.20, fo=0.00),
    PartitionRow(dvs=2.0, fl=0.00, fs=0.10, fr=0.10, fo=0.80),
)

# Committed senescence placeholders (mirror senescence.yaml).
_RDR_LEAF, _RDR_STEM, _RDR_ROOT = 0.02, 0.005, 0.01


# --- partition_fractions: knot literals + the clamp -------------------------
@pytest.mark.parametrize(
    ("dvs", "expected"),
    [
        (0.0, (0.55, 0.10, 0.35, 0.00)),  # emergence knot
        (1.0, (0.30, 0.50, 0.20, 0.00)),  # anthesis knot
        (2.0, (0.00, 0.10, 0.10, 0.80)),  # maturity knot (grain dominant)
        (0.5, (0.425, 0.30, 0.275, 0.00)),  # midpoint of [0, 1] (independent lerp)
        (1.5, (0.15, 0.30, 0.15, 0.40)),  # midpoint of [1, 2] (grain filling)
        (-1.0, (0.55, 0.10, 0.35, 0.00)),  # below the table → clamp to first row
        (3.0, (0.00, 0.10, 0.10, 0.80)),  # above the table → clamp to last row
    ],
)
def test_partition_fractions_known_values(
    dvs: float, expected: tuple[float, float, float, float]
) -> None:
    fl, fs, fr, fo = partition_fractions(dvs, _TABLE)
    assert math.isclose(fl, expected[0], abs_tol=1e-12)
    assert math.isclose(fs, expected[1], abs_tol=1e-12)
    assert math.isclose(fr, expected[2], abs_tol=1e-12)
    assert math.isclose(fo, expected[3], abs_tol=1e-12)


@pytest.mark.parametrize("dvs", [0.0, 0.13, 0.5, 0.99, 1.0, 1.37, 2.0, 5.0, -2.0])
def test_partition_fractions_sum_to_one_everywhere(dvs: float) -> None:
    # The load-bearing invariant: a shared-breakpoint table is sum-1 at every DVS
    # (else the allocation legs would not sum to DMI and the gate would hard-fail).
    fl, fs, fr, fo = partition_fractions(dvs, _TABLE)
    assert math.isclose(fl + fs + fr + fo, 1.0, abs_tol=1e-12)


def test_partition_fractions_empty_table_raises() -> None:
    with pytest.raises(ValueError, match="at least one row"):
        partition_fractions(0.5, ())


# --- partition: exact split of a given DMI ----------------------------------
def test_partition_splits_dmi_exactly() -> None:
    # DVS 0.5 → (0.425, 0.30, 0.275, 0.0); storage = 0 in the vegetative phase.
    leaf, stem, root, storage = partition(2.0, 0.5, _TABLE)
    assert math.isclose(leaf, 0.85, rel_tol=1e-12)
    assert math.isclose(stem, 0.60, rel_tol=1e-12)
    assert math.isclose(root, 0.55, rel_tol=1e-12)
    assert storage == 0.0
    assert math.isclose(leaf + stem + root + storage, 2.0, rel_tol=1e-12)


def test_partition_fills_storage_in_the_reproductive_phase() -> None:
    # DVS 1.5 → fractions (0.15, 0.30, 0.15, 0.40): the grain sink is now nonzero.
    leaf, stem, root, storage = partition(2.0, 1.5, _TABLE)
    assert math.isclose(leaf, 0.30, rel_tol=1e-12)
    assert math.isclose(stem, 0.60, rel_tol=1e-12)
    assert math.isclose(root, 0.30, rel_tol=1e-12)
    assert math.isclose(storage, 0.80, rel_tol=1e-12)
    assert math.isclose(leaf + stem + root + storage, 2.0, rel_tol=1e-12)


# --- senescence_flux: independent literals ----------------------------------
def test_senescence_flux_proportional_to_organ_carbon() -> None:
    assert math.isclose(
        senescence_flux(4.0, relative_death_rate=0.02), 0.08, rel_tol=1e-12
    )


def test_senescence_flux_zero_organ_is_zero() -> None:
    assert senescence_flux(0.0, relative_death_rate=0.02) == 0.0


# --- the assembled Senescence flow ------------------------------------------
_BIO = DomainId("biosphere")
_BND = DomainId("boundary")
_LEAF_C = StockId("biosphere.leaf_c")
_STEM_C = StockId("biosphere.stem_c")
_ROOT_C = StockId("biosphere.root_c")
_LITTER = StockId("boundary.litter")


def _organ(stock_id: StockId, amount: float) -> Stock:
    return Stock(
        id=stock_id,
        domain=_BIO,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=0.0,
    )


def _state(*, leaf_c: float = 3.0, stem_c: float = 1.0, root_c: float = 1.0) -> State:
    litter = Stock(
        id=_LITTER,
        domain=_BND,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=0.0,
        kind=StockKind.BOUNDARY,
    )
    stocks = {
        _LEAF_C: _organ(_LEAF_C, leaf_c),
        _STEM_C: _organ(_STEM_C, stem_c),
        _ROOT_C: _organ(_ROOT_C, root_c),
        _LITTER: litter,
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _env(snapshot: State, dt: float):  # noqa: ANN202 - Senescence ignores env (no forcing)
    return SourceResolver().bind(snapshot, dt)


def _senescence_params() -> SenescenceParams:
    return SenescenceParams(rdr_leaf=_RDR_LEAF, rdr_stem=_RDR_STEM, rdr_root=_RDR_ROOT)


def _senescence_flow() -> Senescence:
    return Senescence(
        id=FlowId("biosphere.senescence"),
        priority=0,
        leaf_c=_LEAF_C,
        stem_c=_STEM_C,
        root_c=_ROOT_C,
        litter_sink=_LITTER,
        params=_senescence_params(),
    )


def test_senescence_legs_are_the_hand_computed_losses() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    result = _senescence_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    leaf, stem, root = 3.0 * _RDR_LEAF, 1.0 * _RDR_STEM, 1.0 * _RDR_ROOT
    assert math.isclose(legs[_LEAF_C], -leaf, rel_tol=1e-12)
    assert math.isclose(legs[_STEM_C], -stem, rel_tol=1e-12)
    assert math.isclose(legs[_ROOT_C], -root, rel_tol=1e-12)
    assert math.isclose(legs[_LITTER], leaf + stem + root, rel_tol=1e-12)


def test_senescence_flow_is_carbon_balanced() -> None:
    state = _state()
    result = _senescence_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_senescence_scales_linearly_with_dt() -> None:
    state = _state()
    flow = _senescence_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _LITTER
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _LITTER
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- config boundary: load_allocation_params --------------------------------
def test_allocation_params_file_exists() -> None:
    assert ALLOCATION_PARAMS_PATH.is_file()


def test_load_allocation_params_matches_committed_values() -> None:
    p = load_allocation_params()
    assert isinstance(p, AllocationParams)
    assert p.table == _TABLE


def _valid_alloc() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "allocation",
        "parameters": {
            "partition_table": {
                "source": "[A]",
                "rows": [
                    {"dvs": 0.0, "fl": 0.55, "fs": 0.10, "fr": 0.35, "fo": 0.00},
                    {"dvs": 1.0, "fl": 0.30, "fs": 0.50, "fr": 0.20, "fo": 0.00},
                    {"dvs": 2.0, "fl": 0.00, "fs": 0.10, "fr": 0.10, "fo": 0.80},
                ],
            }
        },
    }


def _write_alloc(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "allocation.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_alloc_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_allocation_params(_write_alloc(tmp_path, _valid_alloc()))
    assert p.table[0] == PartitionRow(dvs=0.0, fl=0.55, fs=0.10, fr=0.35, fo=0.00)


def test_alloc_loader_rejects_a_row_not_summing_to_one(tmp_path: Path) -> None:
    data = _valid_alloc()
    data["parameters"]["partition_table"]["rows"][1]["fl"] = 0.40  # now sums to 1.10
    with pytest.raises(ValueError, match="sum to 1"):
        load_allocation_params(_write_alloc(tmp_path, data))


def test_alloc_loader_rejects_non_increasing_dvs(tmp_path: Path) -> None:
    data = _valid_alloc()
    data["parameters"]["partition_table"]["rows"][1]["dvs"] = 0.0  # equal to row 0
    with pytest.raises(ValueError, match="strictly increase"):
        load_allocation_params(_write_alloc(tmp_path, data))


@pytest.mark.parametrize("bad", [-0.1, 1.5])
def test_alloc_loader_rejects_out_of_range_fraction(tmp_path: Path, bad: float) -> None:
    data = _valid_alloc()
    # Keep the row summing to 1 so the range check (not the sum check) is what bites:
    # set fl = bad and fs = 1 - bad - fr.
    row0 = data["parameters"]["partition_table"]["rows"][0]
    row0["fl"] = bad
    row0["fs"] = 1.0 - bad - row0["fr"] - row0["fo"]
    with pytest.raises(ValueError, match=r"must be in \[0, 1\]"):
        load_allocation_params(_write_alloc(tmp_path, data))


def test_alloc_loader_rejects_too_few_rows(tmp_path: Path) -> None:
    data = _valid_alloc()
    data["parameters"]["partition_table"]["rows"] = [
        {"dvs": 0.0, "fl": 0.55, "fs": 0.10, "fr": 0.35, "fo": 0.00}
    ]
    with pytest.raises(ValueError, match=">= 2 rows"):
        load_allocation_params(_write_alloc(tmp_path, data))


def test_alloc_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_alloc()
    del data["parameters"]["partition_table"]["source"]
    with pytest.raises(ValidationError):
        load_allocation_params(_write_alloc(tmp_path, data))


def test_alloc_loader_rejects_an_unknown_field(tmp_path: Path) -> None:
    data = _valid_alloc()
    data["parameters"]["partition_table"]["rows"][0]["fx"] = 0.0
    with pytest.raises(ValidationError):
        load_allocation_params(_write_alloc(tmp_path, data))


# --- config boundary: load_senescence_params --------------------------------
def test_senescence_params_file_exists() -> None:
    assert SENESCENCE_PARAMS_PATH.is_file()


def test_load_senescence_params_matches_committed_values() -> None:
    p = load_senescence_params()
    assert isinstance(p, SenescenceParams)
    assert (p.rdr_leaf, p.rdr_stem, p.rdr_root) == (_RDR_LEAF, _RDR_STEM, _RDR_ROOT)


def _valid_senescence() -> dict[str, Any]:
    return {
        "name": "winter_wheat",
        "process": "senescence",
        "parameters": {
            "rdr_leaf": {"value": 0.02, "unit": "1/day", "source": "[A]"},
            "rdr_stem": {"value": 0.005, "unit": "1/day", "source": "[A]"},
            "rdr_root": {"value": 0.01, "unit": "1/day", "source": "[A]"},
        },
    }


def _write_senescence(tmp_path: Path, data: dict[str, Any]) -> Path:
    p = tmp_path / "senescence.yaml"
    p.write_text(yaml.safe_dump(data), encoding="utf-8")
    return p


def test_senescence_loader_round_trips_a_valid_file(tmp_path: Path) -> None:
    p = load_senescence_params(_write_senescence(tmp_path, _valid_senescence()))
    assert p.rdr_leaf == 0.02


def test_senescence_loader_accepts_zero_rate(tmp_path: Path) -> None:
    # A zero relative death rate is valid (no turnover of that organ).
    data = _valid_senescence()
    data["parameters"]["rdr_stem"]["value"] = 0.0
    p = load_senescence_params(_write_senescence(tmp_path, data))
    assert p.rdr_stem == 0.0


def test_senescence_loader_rejects_a_wrong_unit(tmp_path: Path) -> None:
    data = _valid_senescence()
    data["parameters"]["rdr_leaf"]["unit"] = "1/s"  # right dim, wrong scale
    with pytest.raises(ValueError, match="rdr_leaf"):
        load_senescence_params(_write_senescence(tmp_path, data))


def test_senescence_loader_rejects_a_negative_rate(tmp_path: Path) -> None:
    data = _valid_senescence()
    data["parameters"]["rdr_root"]["value"] = -0.01
    with pytest.raises(ValueError, match="rdr_root"):
        load_senescence_params(_write_senescence(tmp_path, data))


def test_senescence_loader_rejects_a_missing_source(tmp_path: Path) -> None:
    data = _valid_senescence()
    del data["parameters"]["rdr_leaf"]["source"]
    with pytest.raises(ValidationError):
        load_senescence_params(_write_senescence(tmp_path, data))
