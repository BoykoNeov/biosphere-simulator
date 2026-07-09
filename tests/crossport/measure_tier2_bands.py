"""Measure the Tier-2 band basis: the propagated ±1-ULP transcendental sensitivity.

Phase-7 Step 3 (P7.3). The three Step-3 Tier-2 goldens (power / power-self-discharge /
thermal) each have a transcendental in their per-step graph (the half-sine ``math.sin``;
the Stefan-Boltzmann ``t**4``). Rust ``f64::sin`` / ``powf`` and CPython ``math.sin`` /
``**`` resolve to the **same system libm** on one machine, so a direct Rust-vs-Python
comparison there reads 0.0 — a *same-libm artifact*, not a measurement of the thing
Tier-2 exists to bound (cross-libm divergence).

So we measure the **sensitivity** instead, which is genuinely reproducible: perturb each
scenario's transcendental result by ±1 ULP and re-run to the final state; the propagated
``max_abs_relative_deviation`` is how far a one-ULP libm disagreement moves the whole
trajectory. That is the honest, measured floor a Tier-2 band must sit above (the plan's
"measured, never derived" contract — see ``compare.py``). ``test_crossport.py`` asserts
each committed ``tiers.json`` band is strictly greater than this fresh measurement.

Pure Python (no ``cargo``), so it runs on CI (unlike the Rust-example comparison, which
is ``skipif cargo is None`` — the parity gate is local-only; see the Step-3 writeup).
"""

from __future__ import annotations

import math
import types

from lab.oracle_match import max_abs_relative_deviation
from simcore.integrator import EulerIntegrator

# The denominator floor for the relative-deviation metric (leaves below this magnitude
# are compared absolutely). The Step-3 scenarios have no near-zero stocks, so it does
# not bind; it is fixed here so the measurement and the tiers.json band share one floor.
FLOOR = 1e-12


def _amounts(state) -> list[float]:
    """Stock amounts in canonical (sorted-id) order — the numeric leaves compared."""
    return [state.stocks[k].amount for k in sorted(state.stocks)]


def _nudge(x: float, up: bool) -> float:
    """``x`` moved one ULP toward +inf (``up``) or -inf."""
    return math.nextafter(x, math.inf if up else -math.inf)


def _power_final(days: int, *, with_self_discharge: bool) -> list[float]:
    from domains.power.loader import load_charge_params, load_self_discharge_params
    from domains.power.scenario import BOUNDED_SOC_SCENARIO
    from domains.power.system import build_power, power_resolver, run_power

    charge = load_charge_params()
    sd = load_self_discharge_params() if with_self_discharge else None
    state, registry = build_power(charge, BOUNDED_SOC_SCENARIO, sd)
    resolver = power_resolver(charge, BOUNDED_SOC_SCENARIO)
    integrator = EulerIntegrator(registry)
    steps = days * BOUNDED_SOC_SCENARIO.steps_per_day
    states, _, _ = run_power(
        integrator, state, resolver, BOUNDED_SOC_SCENARIO.dt_seconds, steps
    )
    return _amounts(states[-1])


def _measure_power(days: int, *, with_self_discharge: bool) -> float:
    """Propagated deviation of a ±1-ULP perturbation of the half-sine ``math.sin``.

    Only ``domains.power.system``'s ``math`` reference is shimmed (not the global
    ``math`` module), so no other code is affected; it is restored in ``finally``.
    """
    import domains.power.system as psys

    base = _power_final(days, with_self_discharge=with_self_discharge)
    worst = 0.0
    original = psys.math
    for up in (True, False):
        try:
            psys.math = types.SimpleNamespace(
                pi=math.pi, sin=lambda x, _up=up: _nudge(math.sin(x), _up)
            )
            perturbed = _power_final(days, with_self_discharge=with_self_discharge)
        finally:
            psys.math = original
        worst = max(worst, max_abs_relative_deviation(base, perturbed, floor=FLOOR))
    return worst


def _measure_thermal() -> float:
    """Propagated deviation of a ±1-ULP perturbation of the Stefan-Boltzmann ``t**4``.

    Only ``domains.thermal.flows.radiated_power`` is replaced (module attribute), with a
    reimplementation identical except the ``t**4`` term is nudged one ULP; restored in
    ``finally``.
    """
    import domains.thermal.flows as tflows
    from domains.thermal.flows import STEFAN_BOLTZMANN, temperature
    from domains.thermal.loader import load_thermal_params
    from domains.thermal.scenario import EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS
    from domains.thermal.system import build_thermal, run_thermal, thermal_resolver

    params = load_thermal_params()

    def final() -> list[float]:
        state, registry = build_thermal(params, EQUILIBRIUM_SCENARIO)
        resolver = thermal_resolver(EQUILIBRIUM_SCENARIO)
        integrator = EulerIntegrator(registry)
        states, _, _ = run_thermal(
            integrator,
            state,
            resolver,
            EQUILIBRIUM_SCENARIO.dt_seconds,
            EQUILIBRIUM_STEPS,
        )
        return _amounts(states[-1])

    base = final()
    worst = 0.0
    original = tflows.radiated_power
    for up in (True, False):

        def perturbed_radiated_power(node_joules, *, params, _up=up):
            t = temperature(
                node_joules,
                heat_capacity=params.heat_capacity,
                space_temperature=params.space_temperature,
            )
            t4 = _nudge(t**4, _up)  # the transcendental, perturbed one ULP
            return (
                params.emissivity
                * STEFAN_BOLTZMANN
                * params.radiator_area
                * (t4 - params.space_temperature**4)
            )

        try:
            tflows.radiated_power = perturbed_radiated_power
            perturbed = final()
        finally:
            tflows.radiated_power = original
        worst = max(worst, max_abs_relative_deviation(base, perturbed, floor=FLOOR))
    return worst


def measured_sensitivity(key: str) -> float:
    """The measured ±1-ULP propagated sensitivity for a Step-3 Tier-2 golden `key`."""
    if key == "power_bounded_soc":
        return _measure_power(7, with_self_discharge=False)
    if key == "power_self_discharge":
        return _measure_power(14, with_self_discharge=True)
    if key == "thermal_equilibrium":
        return _measure_thermal()
    raise KeyError(f"no Tier-2 sensitivity measurement for key {key!r}")


# The Step-3 Tier-2 goldens whose bands this module justifies (by `tiers.json` key).
STEP3_TIER2_KEYS = ("power_bounded_soc", "power_self_discharge", "thermal_equilibrium")


# --------------------------------------------------------------------------- #
# Step 4 (P7.4): the biosphere Tier-2 band                                     #
# --------------------------------------------------------------------------- #
# All 7 biosphere goldens share ONE band. Rust-vs-Python is bit-exact locally (same UCRT
# libm, measured 0.0, a same-libm artifact), so the band is justified by the propagated
# +/-1-ULP transcendental sensitivity, exactly as Step 3. The dominant per-step
# transcendental is the FvCB canopy `math.exp`; a one-time comprehensive sweep
# (canopy.exp / photosynthesis.sqrt / transpiration.exp / weather.sin over the perennial
# + consumer 15-yr runs) found the WORST at 6.7e-14 (canopy.exp / perennial 15-yr; the
# contracting limit cycle barely amplifies one ULP). `BIOSPHERE_BAND = 1e-11` sits
# ~150x above that (the Step-3 margin) and far below any meaningful drift. This measure
# re-runs the representative worst case (canopy.exp on both 15-yr scenarios); it is slow
# (~20 s: six 15-yr runs), so its guard test is `-m slow`, not the CI-fast Step-3 one.

BIOSPHERE_BAND = 1e-11
# The 7 biosphere `tiers.json` keys that share BIOSPHERE_BAND.
BIOSPHERE_TIER2_KEYS = (
    "open_season",
    "sealed_chamber",
    "perennial_chamber",
    "perennial_long_horizon",
    "consumer_chamber",
    "consumer_long_horizon",
    "drift_summary",
)


def _biosphere_perennial_final(scenario, years: int) -> list[float]:
    from domains.biosphere.season import build_season, run_perennial, weather_resolver

    weather = _weather() * years
    state, registry = build_season(scenario)
    states, _, _ = run_perennial(
        EulerIntegrator(registry),
        state,
        scenario,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
        year=len(_weather()),
    )
    return _amounts(states[-1])


def _weather() -> list[dict]:
    import json
    from pathlib import Path

    fixture = (
        Path(__file__).resolve().parents[2]
        / "tests"
        / "oracle"
        / "winter_wheat_weather.json"
    )
    return json.loads(fixture.read_text(encoding="utf-8"))["weather"]


def measured_biosphere_sensitivity() -> float:
    """Worst propagated 1-ULP `canopy.exp` sensitivity over the two 15-yr runs.

    The representative dominant transcendental; only `domains.biosphere.canopy`'s `math`
    reference is shimmed (restored in `finally`), so no other code is affected.
    """
    import domains.biosphere.canopy as canopy
    from domains.biosphere.scenario import (
        CONSUMER_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_SCENARIO,
    )

    worst = 0.0
    for scenario in (PERENNIAL_CHAMBER_SCENARIO, CONSUMER_CHAMBER_SCENARIO):
        base = _biosphere_perennial_final(scenario, 15)
        original = canopy.math
        for up in (True, False):
            try:
                canopy.math = types.SimpleNamespace(
                    exp=lambda x, _up=up: _nudge(math.exp(x), _up)
                )
                perturbed = _biosphere_perennial_final(scenario, 15)
            finally:
                canopy.math = original
            worst = max(worst, max_abs_relative_deviation(base, perturbed, floor=FLOOR))
    return worst


if __name__ == "__main__":
    for k in STEP3_TIER2_KEYS:
        print(f"{k:24s} sensitivity = {measured_sensitivity(k):.6e}")
    bio = measured_biosphere_sensitivity()
    print(f"{'biosphere (canopy.exp)':24s} sensitivity = {bio:.6e}")
