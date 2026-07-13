"""Phase-9 Step-1 gate: parameter packs (the "cultivar = param pack" primitive).

A parameter pack is an alternate param file read by the *same frozen loader*
(``load_crew_params(path=…)``), so a pack's values pass the frozen schema/bounds/
unit validation — a pack is a param file, not a way around the guards. Proven:

1. **No-op pack → byte-identity** (faithfulness): a pack that restates the frozen
   0.949/0.675 (with a different ``source:``, which is recorded-not-parsed)
   reproduces ``crew_state.json`` byte-for-byte — the pack *path* is faithful.
2. **Changed pack → predicted shift** (the "it bit" reconstruct-the-factor gate,
   the ``n_limited``/``water_biting`` precedent): a cultivar pack with
   ``respired_carbon_fraction`` 0.949→0.80 respires less CO₂ and egests more feces,
   in the exact 0.80/0.20 split, while the forced ``food_store`` depletion and the
   whole WATER/OXYGEN side stay bit-identical, and CARBON is conserved every step.
   An un-biting pack (identical values) could not pin this.
3. **Packs reuse the frozen validation**: a pack with an out-of-range value
   (``respired_carbon_fraction`` 1.5) raises ``ValueError`` from the frozen loader's
   bound check — the guards are not bypassed.

Full-file packs only (no partial-merge/bundling — deferred). Pack paths resolve
relative to the scenario file's directory (the modder-bundle-relocatable form).
"""

import math
from pathlib import Path

import pytest

import sim_io
from authoring.interpreter import load_scenario
from authoring.run import run_scenario
from domains.crew.loader import load_crew_params
from simcore.conservation import compute_ledger
from simcore.ids import StockId
from simcore.quantities import Quantity

SCENARIO_DIR = Path(__file__).parent / "authoring" / "scenarios"
BASELINE_YAML = SCENARIO_DIR / "crew_mission.yaml"
NOOP_YAML = SCENARIO_DIR / "crew_pack_noop.yaml"
LOW_RESP_YAML = SCENARIO_DIR / "crew_pack_low_resp.yaml"
BAD_BOUND_YAML = SCENARIO_DIR / "crew_pack_bad_bound.yaml"
NOOP_PACK = SCENARIO_DIR / "packs" / "crew_noop.yaml"

GOLDEN_PATH = Path(__file__).parent / "regression" / "golden" / "crew_state.json"

# Stocks that must stay bit-identical between baseline and the low-resp cultivar: the
# WATER/OXYGEN side (insensible fraction unchanged; O₂ flow is param-free) and the
# forced food_store depletion (the -q leg is independent of the carbon split).
_UNCHANGED_STOCKS = (
    StockId("crew.food_store"),
    StockId("crew.water_store"),
    StockId("crew.o2_store"),
    StockId("boundary.crew_humidity"),
    StockId("boundary.urine"),
    StockId("boundary.crew_o2_consumed"),
)
_EXHALED = StockId("boundary.exhaled_co2")
_FECES = StockId("boundary.fecal_waste")
_FOOD = StockId("crew.food_store")


def _final(path: Path):
    states, rationed, events = run_scenario(load_scenario(str(path)))
    return states, rationed, events


def test_noop_pack_matches_crew_golden_bytes() -> None:
    # Faithfulness: a no-op pack reproduces the frozen Crew golden byte-for-byte.
    states, rationed, events = _final(NOOP_YAML)
    assert rationed == 0
    assert events == ()
    produced = sim_io.dumps(states[-1]).encode("utf-8")
    assert produced == GOLDEN_PATH.read_bytes()


def test_noop_pack_loads_same_params_as_default() -> None:
    # The pack path reuses the frozen loader: the no-op pack yields the exact frozen
    # CrewParams (source string differs but is recorded-not-parsed).
    assert load_crew_params(NOOP_PACK) == load_crew_params()


def test_changed_pack_reconstructs_the_carbon_split() -> None:
    # The "it bit" gate: the cultivar pack shifts the carbon split to 0.80/0.20 while
    # leaving the forced depletion and the water/oxygen side bit-identical.
    base_states, _, _ = _final(BASELINE_YAML)
    low_states, low_rationed, low_events = _final(LOW_RESP_YAML)
    assert low_rationed == 0
    assert low_events == ()
    base, low = base_states[-1], low_states[-1]

    # Everything not downstream of respired_carbon_fraction is bit-identical.
    for sid in _UNCHANGED_STOCKS:
        assert low.stocks[sid].amount == base.stocks[sid].amount

    base_exhaled = base.stocks[_EXHALED].amount
    base_feces = base.stocks[_FECES].amount
    low_exhaled = low.stocks[_EXHALED].amount
    low_feces = low.stocks[_FECES].amount

    # The pack bit: less CO₂ respired, more feces egested.
    assert low_exhaled < base_exhaled
    assert low_feces > base_feces

    # The total carbon drawn is unchanged (forced intake) and split by the fraction:
    # exhaled = f·total, feces = (1−f)·total. Reconstruct f from each run's outputs.
    total_low = low_exhaled + low_feces
    total_base = base_exhaled + base_feces
    assert math.isclose(total_low, total_base, rel_tol=1e-12)
    assert math.isclose(low_exhaled / total_low, 0.80, rel_tol=1e-9)
    assert math.isclose(base_exhaled / total_base, 0.949, rel_tol=1e-9)
    # And the carbon drawn from the store equals what reached the two sinks.
    drawn = 1000.0 - low.stocks[_FOOD].amount
    assert math.isclose(drawn, total_low, rel_tol=1e-9)


def test_changed_pack_conserves_carbon_every_step() -> None:
    # The cultivar run keeps CARBON closed every step (the pack changed a split, not
    # the balance) — an imbalanced cultivar is not a valid authored artifact.
    states, _, _ = _final(LOW_RESP_YAML)
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.CARBON].residual) <= 1e-6


def test_bad_bound_pack_is_rejected_by_frozen_loader() -> None:
    # Packs do not bypass the frozen validation: an out-of-range value raises
    # ValueError from load_crew_params' bound check (via the pack path).
    with pytest.raises(ValueError):
        load_scenario(str(BAD_BOUND_YAML))
