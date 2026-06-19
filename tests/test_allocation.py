"""Phase-1 Step-9 tests: leaf/stem/root allocation + senescence (organ-carbon flows).

The first internal-redistribution CARBON process and multi-organ stock structure.
Layers:

* **Pure split functions** (``domains.biosphere.allocation``, stdlib): DVS-keyed
  ``partition_fractions`` (knot literals, the sum-to-1 invariant across a DVS sweep,
  the out-of-range clamp), ``partition`` (an exact split of a given DMI), and
  ``senescence_flux`` (∝ organ carbon, → 0 at zero).
* **The assembled flows**: ``Allocation`` (a carbon-balanced 4-leg redistribution whose
  organ legs equal the recomposed ``DMI·fractions`` and whose ``plant_c`` leg is
  ``−ΣDMI``; the dark-day ``GASS < MRES`` → ``DMI = 0`` clamp; DVS read from
  ``snapshot.aux``; dt-linear) and ``Senescence`` (a carbon-balanced
  ``{organs → litter}`` flow; dt-linear).
* **Config boundary** (``load_allocation_params`` / ``load_senescence_params``): the
  committed files load; a non-sum-1 row, non-increasing DVS, an out-of-range fraction /
  death rate, a bad unit, and a missing source are rejected.
"""

import math
from pathlib import Path
from typing import Any

import pytest
import yaml
from pydantic import ValidationError

from domains.biosphere.allocation import (
    Allocation,
    AllocationParams,
    PartitionRow,
    Senescence,
    SenescenceParams,
    partition,
    partition_fractions,
    senescence_flux,
)
from domains.biosphere.canopy import CanopyParams
from domains.biosphere.loader import (
    ALLOCATION_PARAMS_PATH,
    SENESCENCE_PARAMS_PATH,
    load_allocation_params,
    load_senescence_params,
)
from domains.biosphere.phenology import PhenologyParams
from domains.biosphere.photosynthesis import (
    PhotosynthesisParams,
    daily_canopy_assimilation,
)
from domains.biosphere.respiration import (
    RespirationParams,
    available_for_growth,
    maintenance_respiration_flux,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# Committed partition table (mirrors allocation.yaml).
_TABLE = (
    PartitionRow(dvs=0.0, fl=0.55, fs=0.10, fr=0.35),
    PartitionRow(dvs=1.0, fl=0.30, fs=0.50, fr=0.20),
    PartitionRow(dvs=2.0, fl=0.00, fs=0.50, fr=0.50),
)

# Committed senescence placeholders (mirror senescence.yaml).
_RDR_LEAF, _RDR_STEM, _RDR_ROOT = 0.02, 0.005, 0.01

# FvCB + canopy + respiration + phenology placeholders (mirror the committed yamls;
# the same operating point as the Step-6 GrowthRespiration test, so DMI = 3·GRES there).
_VCMAX, _JMAX, _ALPHA, _THETA = 100.0, 180.0, 0.3, 0.7
_GAMMA_STAR, _KC, _KO, _O2 = 42.75, 404.9, 278.4, 210.0
_TMIN, _TOPT_LO, _TOPT_HI, _TMAX = 0.0, 15.0, 25.0, 35.0
_SLA_PER_MOL_C, _K = 0.5872044444444445, 0.6
_M_REF, _Q10, _T_REF, _YG = 0.02, 2.0, 25.0, 0.75
_TSUM_ANTHESIS, _TSUM_MATURITY = 1100.0, 750.0
_T_BASE, _T_CAP = 0.0, 30.0


def _alloc_params() -> AllocationParams:
    return AllocationParams(table=_TABLE)


def _photo() -> PhotosynthesisParams:
    return PhotosynthesisParams(
        vcmax=_VCMAX,
        jmax=_JMAX,
        quantum_yield=_ALPHA,
        theta=_THETA,
        gamma_star=_GAMMA_STAR,
        kc=_KC,
        ko=_KO,
        o2=_O2,
        t_min=_TMIN,
        t_opt_lo=_TOPT_LO,
        t_opt_hi=_TOPT_HI,
        t_max=_TMAX,
    )


def _canopy() -> CanopyParams:
    return CanopyParams(sla_per_mol_c=_SLA_PER_MOL_C, extinction_coef=_K)


def _resp() -> RespirationParams:
    return RespirationParams(
        maintenance_coef=_M_REF, q10=_Q10, t_ref=_T_REF, growth_efficiency=_YG
    )


def _pheno() -> PhenologyParams:
    return PhenologyParams(
        t_base=_T_BASE,
        t_cap=_T_CAP,
        tsum_anthesis=_TSUM_ANTHESIS,
        tsum_maturity=_TSUM_MATURITY,
    )


# --- partition_fractions: knot literals + the clamp -------------------------
@pytest.mark.parametrize(
    ("dvs", "expected"),
    [
        (0.0, (0.55, 0.10, 0.35)),  # emergence knot
        (1.0, (0.30, 0.50, 0.20)),  # anthesis knot
        (2.0, (0.00, 0.50, 0.50)),  # maturity knot
        (0.5, (0.425, 0.30, 0.275)),  # midpoint of [0, 1] (independent lerp)
        (1.5, (0.15, 0.50, 0.35)),  # midpoint of [1, 2]
        (-1.0, (0.55, 0.10, 0.35)),  # below the table → clamp to first row
        (3.0, (0.00, 0.50, 0.50)),  # above the table → clamp to last row
    ],
)
def test_partition_fractions_known_values(
    dvs: float, expected: tuple[float, float, float]
) -> None:
    fl, fs, fr = partition_fractions(dvs, _TABLE)
    assert math.isclose(fl, expected[0], abs_tol=1e-12)
    assert math.isclose(fs, expected[1], abs_tol=1e-12)
    assert math.isclose(fr, expected[2], abs_tol=1e-12)


@pytest.mark.parametrize("dvs", [0.0, 0.13, 0.5, 0.99, 1.0, 1.37, 2.0, 5.0, -2.0])
def test_partition_fractions_sum_to_one_everywhere(dvs: float) -> None:
    # The load-bearing invariant: a shared-breakpoint table is sum-1 at every DVS
    # (else the allocation legs would not sum to DMI and the gate would hard-fail).
    fl, fs, fr = partition_fractions(dvs, _TABLE)
    assert math.isclose(fl + fs + fr, 1.0, abs_tol=1e-12)


def test_partition_fractions_empty_table_raises() -> None:
    with pytest.raises(ValueError, match="at least one row"):
        partition_fractions(0.5, ())


# --- partition: exact split of a given DMI ----------------------------------
def test_partition_splits_dmi_exactly() -> None:
    leaf, stem, root = partition(2.0, 0.5, _TABLE)  # fractions (0.425, 0.30, 0.275)
    assert math.isclose(leaf, 0.85, rel_tol=1e-12)
    assert math.isclose(stem, 0.60, rel_tol=1e-12)
    assert math.isclose(root, 0.55, rel_tol=1e-12)
    assert math.isclose(leaf + stem + root, 2.0, rel_tol=1e-12)


# --- senescence_flux: independent literals ----------------------------------
def test_senescence_flux_proportional_to_organ_carbon() -> None:
    assert math.isclose(
        senescence_flux(4.0, relative_death_rate=0.02), 0.08, rel_tol=1e-12
    )


def test_senescence_flux_zero_organ_is_zero() -> None:
    assert senescence_flux(0.0, relative_death_rate=0.02) == 0.0


# --- the assembled Allocation flow ------------------------------------------
_BIO = DomainId("biosphere")
_BND = DomainId("boundary")
_PLANT_C = StockId("biosphere.plant_c")
_LEAF_C = StockId("biosphere.leaf_c")
_STEM_C = StockId("biosphere.stem_c")
_ROOT_C = StockId("biosphere.root_c")
_LITTER = StockId("boundary.litter")
_THERMAL_TIME = "thermal_time"


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


def _state(
    *,
    plant_c: float = 10.0,
    leaf_c: float = 3.0,
    stem_c: float = 1.0,
    root_c: float = 1.0,
    thermal_time: float = 550.0,  # DVS = 550/1100 = 0.5 (mid vegetative ramp)
) -> State:
    litter = Stock(
        id=_LITTER,
        domain=_BND,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=0.0,
        kind=StockKind.BOUNDARY,
    )
    stocks = {
        _PLANT_C: _organ(_PLANT_C, plant_c),
        _LEAF_C: _organ(_LEAF_C, leaf_c),
        _STEM_C: _organ(_STEM_C, stem_c),
        _ROOT_C: _organ(_ROOT_C, root_c),
        _LITTER: litter,
    }
    return State(n=0, stocks=stocks, rng_seed=0, aux={_THERMAL_TIME: thermal_time})


def _env(snapshot: State, dt: float, *, par: float = 800.0, temp: float = 20.0):  # noqa: ANN202
    resolver = SourceResolver(
        forcings={
            "par": constant(par),
            "ci": constant(400.0),
            "temp": constant(temp),
            "daylength_s": constant(43200.0),
        }
    )
    return resolver.bind(snapshot, dt)


def _allocation_flow() -> Allocation:
    return Allocation(
        id=FlowId("biosphere.allocation"),
        priority=0,
        plant_c=_PLANT_C,
        leaf_c=_LEAF_C,
        stem_c=_STEM_C,
        root_c=_ROOT_C,
        par_var="par",
        ci_var="ci",
        temp_var="temp",
        daylength_var="daylength_s",
        thermal_time_aux=_THERMAL_TIME,
        photo=_photo(),
        canopy=_canopy(),
        resp=_resp(),
        pheno=_pheno(),
        alloc=_alloc_params(),
        ground_area=1.0,
    )


def _expected_dmi(*, leaf_c: float, biomass: float, par: float = 800.0) -> float:
    """Recompute DMI from the public pure functions (an independent flow check)."""
    lai = leaf_c * _SLA_PER_MOL_C / 1.0
    gross = daily_canopy_assimilation(
        par,
        lai,
        400.0,
        20.0,
        43200.0,
        params=_photo(),
        canopy=_canopy(),
        ground_area=1.0,
    )
    mres = maintenance_respiration_flux(biomass, 20.0, params=_resp())
    return _YG * available_for_growth(gross, mres)


def test_allocation_legs_are_the_partitioned_structural_increment() -> None:
    state = _state()
    dmi = _expected_dmi(leaf_c=3.0, biomass=5.0)
    leaf, stem, root = partition(dmi, 0.5, _TABLE)  # DVS = 0.5
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_LEAF_C], leaf, rel_tol=1e-12)
    assert math.isclose(legs[_STEM_C], stem, rel_tol=1e-12)
    assert math.isclose(legs[_ROOT_C], root, rel_tol=1e-12)
    assert math.isclose(legs[_PLANT_C], -(leaf + stem + root), rel_tol=1e-12)


def test_allocation_dmi_agrees_with_step6_growth_resp_budget() -> None:
    # Agreement-by-construction (the shared available_for_growth helper): at the Step-6
    # GrowthRespiration point (all biomass in leaf, so LAI and maintenance read
    # the same 5.0 mol C) DMI = Yg·A and GRES = (1−Yg)·A share A, so DMI = 3·GRES with
    # Yg = 0.75. The Step-6 test pins GRES = 0.32678769775306143 at this point.
    state = _state(leaf_c=5.0, stem_c=0.0, root_c=0.0)
    dmi = _expected_dmi(leaf_c=5.0, biomass=5.0)
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(-legs[_PLANT_C], dmi, rel_tol=1e-12)
    assert math.isclose(dmi, 3.0 * 0.32678769775306143, rel_tol=1e-12)


def test_allocation_flow_is_carbon_balanced() -> None:
    state = _state()
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)


def test_allocation_plant_leg_is_minus_sum_of_organ_legs() -> None:
    # Balance-by-construction: the buffer drain equals the deposited organ carbon.
    state = _state()
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    organ_sum = legs[_LEAF_C] + legs[_STEM_C] + legs[_ROOT_C]
    assert math.isclose(legs[_PLANT_C], -organ_sum, rel_tol=1e-12)


def test_allocation_clamps_to_zero_in_the_dark() -> None:
    # No light ⇒ GASS = 0 ⇒ MRES > GASS ⇒ available_for_growth = 0 ⇒ DMI = 0 (no leg).
    state = _state()
    result = _allocation_flow().evaluate(state, _env(state, 1.0, par=0.0), 1.0)
    for leg in result.legs:
        assert leg.amount == 0.0


def test_allocation_reads_dvs_from_aux() -> None:
    # A different thermal_time ⇒ different DVS ⇒ different fractions (the aux is read).
    veg = _allocation_flow().evaluate(
        _state(thermal_time=550.0), _env(_state(), 1.0), 1.0
    )
    repro = _allocation_flow().evaluate(
        _state(thermal_time=1100.0 + 375.0), _env(_state(), 1.0), 1.0
    )
    # DVS 0.5 → FR 0.275; DVS 1.5 → FR 0.35: the root share rises, so the ratio shifts.
    veg_legs = {leg.stock: leg.amount for leg in veg.legs}
    repro_legs = {leg.stock: leg.amount for leg in repro.legs}
    veg_root_share = veg_legs[_ROOT_C] / -veg_legs[_PLANT_C]
    repro_root_share = repro_legs[_ROOT_C] / -repro_legs[_PLANT_C]
    assert math.isclose(veg_root_share, 0.275, rel_tol=1e-9)
    assert math.isclose(repro_root_share, 0.35, rel_tol=1e-9)


def test_allocation_scales_linearly_with_dt() -> None:
    state = _state()
    flow = _allocation_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _LEAF_C
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _LEAF_C
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- the assembled Senescence flow ------------------------------------------
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
                    {"dvs": 0.0, "fl": 0.55, "fs": 0.10, "fr": 0.35},
                    {"dvs": 1.0, "fl": 0.30, "fs": 0.50, "fr": 0.20},
                    {"dvs": 2.0, "fl": 0.00, "fs": 0.50, "fr": 0.50},
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
    assert p.table[0] == PartitionRow(dvs=0.0, fl=0.55, fs=0.10, fr=0.35)


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
    fr = data["parameters"]["partition_table"]["rows"][0]["fr"]
    data["parameters"]["partition_table"]["rows"][0]["fl"] = bad
    data["parameters"]["partition_table"]["rows"][0]["fs"] = 1.0 - bad - fr
    with pytest.raises(ValueError, match=r"must be in \[0, 1\]"):
        load_allocation_params(_write_alloc(tmp_path, data))


def test_alloc_loader_rejects_too_few_rows(tmp_path: Path) -> None:
    data = _valid_alloc()
    data["parameters"]["partition_table"]["rows"] = [
        {"dvs": 0.0, "fl": 0.55, "fs": 0.10, "fr": 0.35}
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
    data["parameters"]["partition_table"]["rows"][0]["fo"] = 0.0
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
