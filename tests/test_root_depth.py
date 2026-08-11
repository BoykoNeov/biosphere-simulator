"""Pins for the rooted-depth extension + the root-zone access gate.

⚠ **THESE PINS EXIST BECAUSE NO GOLDEN CAN CATCH THIS MECHANISM.** The gate is
bit-identically inert on every frozen scenario (measured — see
``docs/plans/post-roadmap-root-functional-coupling.md``), so deleting
``RootDepthExtension`` outright, or the ``root_access`` factor from
``NitrogenUptake``, leaves all 25 goldens green. The freeze manifest's ``aux_set``
catches the *process* going missing; nothing else here is covered by the regression
suite, so it is covered here instead.

Every assertion below was **mutation-verified**: each was seen to fail against a
deliberately broken variant before being committed (a passing test proves nothing until
it has been seen to fail — the bioregenerative-station discipline).
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from domains.biosphere.loader import (
    crop_param_set,
    load_phenology_params,
    load_photosynthesis_params,
    load_root_depth_params,
)
from domains.biosphere.root_depth import (
    RootDepthExtension,
    RootDepthParams,
    root_zone_fraction,
)
from domains.biosphere.scenario import DEFAULT_SCENARIO, SeasonScenario
from domains.biosphere.season import build_season, run_season, weather_resolver
from domains.biosphere.stocks import (
    PLANT_N,
    ROOTED_DEPTH,
    SOIL_WATER,
    TEMP_VAR,
    THERMAL_TIME,
)
from simcore.auxiliary import AuxId
from simcore.integrator import EulerIntegrator

_WEATHER = json.loads(
    (Path(__file__).parent / "oracle" / "winter_wheat_weather.json").read_text(
        encoding="utf-8"
    )
)["weather"]

_ROOTD = load_root_depth_params()


def _run(scenario: SeasonScenario = DEFAULT_SCENARIO, years: int = 1):
    weather = _WEATHER * years
    state, registry = build_season(scenario)
    return run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
    )


# --- the pure function ---------------------------------------------------------------
@pytest.mark.parametrize(
    ("depth", "layer", "expected"),
    [
        (0.0, 0.30, 0.0),  # a sown seed reaches nothing
        (-1.0, 0.30, 0.0),  # defensive: never negative
        (0.15, 0.30, 0.5),  # half the layer
        (0.30, 0.30, 1.0),  # exactly the layer
        (1.30, 0.30, 1.0),  # deeper than the layer still saturates at 1
    ],
)
def test_root_zone_fraction_is_a_clamped_ratio(
    depth: float, layer: float, expected: float
) -> None:
    assert root_zone_fraction(depth, soil_layer_depth=layer) == expected


def test_root_zone_fraction_can_only_reduce_never_reverse() -> None:
    # The gate multiplies a SUPPLY term, so it must live in [0, 1] for every input a run
    # could hand it: a value > 1 would manufacture nitrogen, a value < 0 would reverse
    # the flow into a ReversedFlowError. Structural positivity, pinned.
    for depth in (0.0, 1e-9, 0.05, 0.3, 1.3, 1e6):
        assert 0.0 <= root_zone_fraction(depth, soil_layer_depth=0.30) <= 1.0


# --- the cited parameters ------------------------------------------------------------
def test_winter_wheat_carries_table_25s_own_row() -> None:
    # [E] Table 25 p. 137, "Wheat winter" (Gregory et al., 1978), read off the page
    # image. Pinned as VALUES because the file's provenance is the whole point: this is
    # the row for winter wheat, not spring wheat (0.012 / 1.8) and not the generic
    # 3-5 cm/day body-text range.
    assert RootDepthParams(max_extension_rate=0.018, max_rooted_depth=1.3) == _ROOTD


def test_potato_overrides_root_depth_rather_than_sharing_wheats() -> None:
    # [E] Table 25 gives potato its own row from its own reference (Vos & Groenwold,
    # 1986), differing in BOTH values — so sharing wheat's file would assert a rooting
    # habit the source contradicts. The crop-set partition makes that a checked claim.
    potato = crop_param_set("potato")
    assert "root_depth" in potato.overridden
    assert "root_depth" not in potato.shared
    params = load_root_depth_params(potato.paths["root_depth"])
    assert params.max_extension_rate == 0.014
    # 0.9 is the MIDPOINT of the source's "0.8-1.0" range, recorded as such in the file.
    assert params.max_rooted_depth == 0.9
    # The qualitative fact the numbers really assert: potato roots shallower than wheat.
    assert params.max_rooted_depth < _ROOTD.max_rooted_depth


def test_a_non_positive_parameter_is_rejected_at_the_boundary(tmp_path: Path) -> None:
    # Both bounds disable the mechanism SILENTLY (zero rate freezes depth at 0 so the
    # gate is shut forever; zero depth divides by a crop that cannot root), and no
    # golden
    # would notice. So they are rejected where they are read.
    src = Path("src/domains/biosphere/params/root_depth.yaml").read_text(
        encoding="utf-8"
    )
    for field in ("max_extension_rate", "max_rooted_depth"):
        bad = tmp_path / f"{field}.yaml"
        original = "value: 0.018" if field == "max_extension_rate" else "value: 1.3"
        bad.write_text(src.replace(original, "value: 0.0", 1), encoding="utf-8")
        with pytest.raises(ValueError, match=f"{field} must be > 0"):
            load_root_depth_params(bad)


# --- the law -------------------------------------------------------------------------
def test_depth_follows_es_law_and_stops_at_the_cited_cap() -> None:
    states, _, _ = _run()
    depths = [s.aux[ROOTED_DEPTH] for s in states]

    assert depths[0] == 0.0  # a sown seed has no root system
    assert all(
        b >= a for a, b in zip(depths, depths[1:], strict=False)
    )  # monotone: roots only deepen
    # The cap binds, and the overshoot is bounded by ONE step's unstressed extension —
    # the documented consequence of cutting the RATE at the cap rather than clamping the
    # increment (which would break the aux channel's dt-independence contract).
    assert max(depths) >= _ROOTD.max_rooted_depth
    assert max(depths) <= _ROOTD.max_rooted_depth + _ROOTD.max_extension_rate
    # No step ever extends faster than the unstressed maximum (f_water, f_temp <= 1).
    steps = [b - a for a, b in zip(depths, depths[1:], strict=False)]
    assert max(steps) <= _ROOTD.max_extension_rate + 1e-15


def test_extension_is_temperature_and_water_gated_not_a_flat_rate() -> None:
    # [E] gives GZRT = GZRTC * WSERT * TERT, so a real run must extend SLOWER than the
    # flat maximum on at least some steps. Without this, "0.018 m/day" and "0.018 m/day
    # x factors" are indistinguishable — and the flat version would reach the cap in
    # exactly 73 steps.
    states, _, _ = _run()
    depths = [s.aux[ROOTED_DEPTH] for s in states]
    steps = [b - a for a, b in zip(depths, depths[1:], strict=False)]
    throttled = [s for s in steps if 0.0 < s < _ROOTD.max_extension_rate - 1e-12]
    assert len(throttled) > 30, "the two response factors are not being applied"


def test_root_growth_stops_at_flowering() -> None:
    # [E] p. 136: "Root growth generally stops around flowering". Exercised directly,
    # because for the frozen winter wheat the 1.3 m cap binds FIRST (~day 140 vs
    # anthesis
    # ~day 255), so a full run cannot tell the two cut-offs apart. A deep-capped crop
    # can.
    pheno = load_phenology_params()
    photo = load_photosynthesis_params()
    proc = RootDepthExtension(
        id=AuxId("test.rooted_depth"),
        accumulator=ROOTED_DEPTH,
        thermal_time_aux=THERMAL_TIME,
        temp_var=TEMP_VAR,
        soil_water=SOIL_WATER,
        params=RootDepthParams(max_extension_rate=0.018, max_rooted_depth=99.0),
        photo=photo,
        pheno=pheno,
        sw_wilting=DEFAULT_SCENARIO.sw_wilting,
        sw_critical=DEFAULT_SCENARIO.sw_critical,
    )
    state, _ = build_season()

    class _Env:
        def get(self, var: str) -> float:
            return 20.0  # comfortably inside the optimum plateau

    vegetative = state.__class__(
        n=0,
        stocks=state.stocks,
        rng_seed=0,
        aux={THERMAL_TIME: 0.0, ROOTED_DEPTH: 0.5},
    )
    flowering = state.__class__(
        n=0,
        stocks=state.stocks,
        rng_seed=0,
        # past tsum_anthesis ⇒ DVS >= 1
        aux={THERMAL_TIME: pheno.tsum_anthesis + 1.0, ROOTED_DEPTH: 0.5},
    )
    assert proc.evaluate(vegetative, _Env(), 1.0)[ROOTED_DEPTH] > 0.0
    assert proc.evaluate(flowering, _Env(), 1.0)[ROOTED_DEPTH] == 0.0


def test_a_resown_crop_starts_with_no_root_system() -> None:
    # Rooted depth is a property of the standing crop, not of the soil, so it resets
    # with
    # the other per-cycle accumulators. Pinned because the chambers re-sow many times
    # and
    # the goldens cannot see it (measured bit-identical either way).
    from domains.biosphere.scenario import PERENNIAL_CHAMBER_SCENARIO
    from domains.biosphere.season import run_perennial

    scenario = PERENNIAL_CHAMBER_SCENARIO
    weather = _WEATHER * 3
    state, registry = build_season(scenario)
    states, _, _ = run_perennial(
        EulerIntegrator(registry),
        state,
        scenario,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
        year=len(_WEATHER),
    )
    depths = [s.aux[ROOTED_DEPTH] for s in states]
    drops = [
        i for i, (a, b) in enumerate(zip(depths, depths[1:], strict=False)) if b < a
    ]
    assert drops, "rooted depth never reset — a re-sown crop kept the old root system"
    # The recorded value just after a reset is not exactly 0: annual_reset zeroes the
    # accumulator and the SAME step then applies one extension increment before the
    # state is snapshotted. So the post-reset depth must be within one unstressed step
    # of zero — which still pins "the new crop starts from bare soil", while pinning 0.0
    # exactly would be pinning the reset's position within the step instead.
    for i in drops:
        assert 0.0 <= depths[i + 1] <= _ROOTD.max_extension_rate
        # and it must be a genuine reset, not a small dip
        assert depths[i + 1] < depths[i] / 2.0


# --- the gate actually gates ---------------------------------------------------------
def test_the_gate_scales_uptake_capacity() -> None:
    # The headline behavioural claim. It cannot be made against a golden (the mechanism
    # is inert on all of them), so it is made against a scenario built to expose it: a
    # short run whose reference layer is far deeper than the crop can reach, so FROOT1
    # stays well below 1 and uptake is throttled. Contrast with the same run at a
    # shallow
    # layer, where the gate is fully open.
    shallow = SeasonScenario(soil_layer_depth=0.01)  # FROOT1 saturates after one step
    deep = SeasonScenario(soil_layer_depth=50.0)  # FROOT1 stays tiny all season
    open_states, _, _ = _run(shallow)
    gated_states, _, _ = _run(deep)
    assert (
        gated_states[-1].stocks[PLANT_N].amount < open_states[-1].stocks[PLANT_N].amount
    ), "the root-zone gate does not restrict nitrogen uptake"


def test_the_gate_is_inert_on_the_frozen_reference_and_that_is_recorded() -> None:
    # ⚠ NOT a claim that the mechanism does nothing — see the test above. This pins the
    # measured fact the whole plan doc turns on: on the FROZEN scenario the gate changes
    # nothing, because uptake is demand-bound there and the gate only shrinks supply.
    # If this ever starts failing, the frozen crop has become supply-bound and
    # `soil_layer_depth` (a DESIGN value) has silently become load-bearing.
    default_states, _, _ = _run()
    wide = SeasonScenario(soil_layer_depth=0.0001)  # gate fully open from step 1
    wide_states, _, _ = _run(wide)
    assert (
        default_states[-1].stocks[PLANT_N].amount.hex()
        == wide_states[-1].stocks[PLANT_N].amount.hex()
    )


def test_the_harvest_scenarios_root_system_tracks_the_crop_maximum() -> None:
    # ⚠ A CONSISTENCY REQUIREMENT ACROSS TWO FILES, enforced here because nothing else
    # can. `HarvestScenario.rooted_depth0` exists because that scenario starts its crop
    # past anthesis, and its justification is precisely that the value IS the crop
    # maximum — the extension law has necessarily finished by flowering. That is a
    # claim
    # relating two numbers in two different files, asserted only in a comment.
    #
    # The harvest golden cannot check it: the golden records whatever
    # `rooted_depth0` says, so if `root_depth.yaml`'s cap ever moved the golden would
    # stay green and the stated relationship would quietly become false — the
    # `asserted-attributions-rot`
    # shape. Same enforcement precedent as nitrogen.yaml's "carbon_fraction MUST equal
    # canopy.yaml's value", which is likewise checked in a test rather than trusted.
    from station.scenario import HARVEST_SCENARIO

    assert HARVEST_SCENARIO.rooted_depth0 == _ROOTD.max_rooted_depth
