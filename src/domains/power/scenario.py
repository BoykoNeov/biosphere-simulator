"""The Power scenario — initial SOC, the diurnal solar shape, the load sizing (P5.3).

The Power analogue of ``domains.biosphere.scenario``: pure scenario **data** (not the
charge coefficient — that is ``params/charge.yaml`` via ``loader.py``). Every field is
sizing / forcing-shape wiring for the standalone validation run that
``domains.power.system`` assembles.

**The load is sized for *daily energy balance*, derived — not stored.** Both Power flows
are **forced** (``SolarCharge``/``LoadDraw`` read ``env`` power × dt; neither depends on
a stock amount), so the battery SOC is a *pure accumulation* of the forcing sequence —
``battery(n) = battery0 + dt·Σ(η_c·solar(k) − load)``. There is **no restoring force /
no attractor** (unlike the biosphere's nonlinearly-damped limit cycle) — *as long as
this scenario drives only the two forced flows.* (The opt-in ``SelfDischarge`` flow,
P5.5, adds a ``−k·battery`` restoring term; the self-discharge run reuses this scenario
verbatim, so this "forced accumulator" derivation is specifically the load-balancing
argument for the **two-flow** ``BOUNDED_SOC`` build.) So a *bounded*,
day-periodic SOC requires the daily charge and discharge to **balance exactly**: any
net daily imbalance drifts SOC linearly and unboundedly. We therefore do **not** store
a hand-tuned ``load_w`` (which would balance only by comment, and break silently if the
schedule shape or η_c changes); instead the scenario carries a dimensionless
``load_fraction`` and ``system.power_resolver`` *derives*
``load_w = load_fraction · η_c · (Σ_day solar) / steps_per_day`` from the actual
discrete solar schedule and the loaded charge efficiency. ``load_fraction == 1`` ⇒ load
draws 100 % of the daily *stored* solar ⇒ exact balance ⇒ bounded periodic SOC.
(``load_fraction > 1`` is the future brownout knob — drain faster than supply.)

This is the one place a Power resolver reads a flow param (η_c): Power's load is
**intrinsically** η_c-coupled (you can only balance against the energy that actually
reaches the battery), unlike the biosphere's weather forcing, which is param-free. The
scenario *data* stays param-free; the coupling lives in the resolver.

**Time unit is seconds (SI), documented not implicit.** Energy is J, power is W = J/s,
so Power's natural step is seconds — ``dt_seconds = 3600`` (1 h), ``steps_per_day = 24``
(a sub-day step resolves the day/night cycle, the biosphere's ``dt = 1 day`` analogue).
The increment-form flows keep determinism + RK4 order at any ``dt`` (multi-rate-safe).

Pure stdlib only (a frozen dataclass).
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class PowerScenario:
    """Scenario data (not the charge param): initial SOC, solar shape, load sizing.

    Defaults are an illustrative small-station microgrid (not a calibrated system —
    Phase 5 ships machinery, not validated numbers, the biosphere ``TODO(cite)``
    precedent). ``solar_peak_w`` and ``battery0`` are sizing data; ``load_fraction`` /
    ``daylight_hours`` / ``dt_seconds`` / ``steps_per_day`` shape the forcing. The load
    itself is *derived* in ``system.power_resolver`` (see the module docstring).
    """

    # Initial battery state of charge (J). Sized (probe) a few× the within-day SOC
    # drawdown so the battery never empties — the well-fed-sizing discipline that keeps
    # ``rationed == 0`` (the Phase-1 "kept non-limiting" pattern). 2e7 J ≈ 5.6 kWh.
    battery0: float = 2.0e7
    # Diurnal peak solar electrical supply (W) — panel conversion folded into the
    # forcing (incident sunlight is not a tracked stock; the biosphere treats light as
    # forcing too).
    solar_peak_w: float = 1000.0
    # Load sizing as a dimensionless fraction of the daily *stored* solar energy. 1.0 ⇒
    # exact daily balance ⇒ bounded periodic SOC. >1 drains faster than supply (the
    # brownout seam, not exercised here). The actual ``load_w`` is derived in the
    # resolver.
    load_fraction: float = 1.0
    # Daylight window length (h), centred at solar noon; the rest of the day is night
    # (solar 0). 12 h ⇒ sunrise 06:00, sunset 18:00 (equinox-like).
    daylight_hours: float = 12.0
    # Integration step (s) and steps per 24 h day. 3600 s × 24 = 86400 s = one day.
    dt_seconds: float = 3600.0
    steps_per_day: int = 24


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call PowerScenario() in their defaults (ruff B008).
DEFAULT_POWER_SCENARIO: PowerScenario = PowerScenario()

# The standalone validation scenario (P5.3): a well-fed, daily-balanced microgrid whose
# battery SOC oscillates within a bounded band (charge by day, discharge by night) and
# returns to the same level every day. ``rationed == 0`` (battery never empties),
# ``events == ()`` (no POPULATION stock), ENERGY conserved every step, heat-generated
# monotonic. The defaults already encode the validation sizing; this alias names it as
# the canonical run shared by the validation test (and the Step-4 golden) so they cannot
# drift on the sizing. Run ``BOUNDED_SOC_DAYS`` days.
BOUNDED_SOC_SCENARIO: PowerScenario = DEFAULT_POWER_SCENARIO
BOUNDED_SOC_DAYS: int = 7

# The self-discharge validation horizon (P5.5). The self-discharge run reuses
# ``BOUNDED_SOC_SCENARIO`` **verbatim** (same daily-balanced forced flows) and merely
# opts the donor-controlled ``SelfDischarge`` flow in via ``build_power`` — so the leak
# is the **sole** driver of any departure from the balanced baseline (which returns to
# ``battery0`` each day). It is not a new ``PowerScenario`` (no data differs); only the
# horizon does. 14 days lets the realistic ~2.6 %/month leak decay the SOC ~1 % below
# ``battery0`` — well above round-off (so "the leak bit" is unambiguous), far from empty
# (so ``rationed == 0`` holds). The attractor/restoring-force property itself is proved
# magnitude-independently by the two-run contraction test, not by this horizon's shape.
SELF_DISCHARGE_DAYS: int = 14
