"""Step-11 regression-snapshot gate: the golden demo run.

Realizes the "Regression snapshot" exit gate — *golden demo run; any bit change is
surfaced*. The full two-domain demo (``build_demo`` + the coupled resolver, the #16
read path) is run a fixed number of steps with each integrator; the **final State**
is serialized via the step-9 ``sim_io`` hex-float serializer and **byte-compared** to
a committed golden. Any bit change in the engine's output — a flow law, the reduction
order, the integrator arithmetic, the params YAML — fails here.

Design (settled with advisor):

* **Snapshot the full ``State`` via ``sim_io.dumps``**, not ``observe``. The
  observation cluster's own note settles this: Phase 0 has no ``observe`` consumer and
  "the golden snapshot round-trips a full ``State`` via ``sim_io``". Reusing the
  step-9 serializer is DRY and gives bit-exact hex-float bytes for free; ``observe``
  would drop exactly the engine internals (the loss-sink, ``unclamped`` light) a
  regression guard wants to keep watching.
* **``coupled_resolver`` is the canonical wiring** (plan: "the canonical golden uses
  ``coupled_resolver``") — bit-identical to the forcing wiring (decision #16) but it
  exercises the internal-resolver read path every step, so the golden also pins #16.
* **Both integrators get a golden.** Euler and RK4 produce different trajectories; a
  regression in either scheme's arithmetic must surface, so each is pinned separately.
* **200 steps** — the exact length the step-10 well-fed gates already validate
  (``test_well_fed_*`` assert ``rationed == 0`` / ``events == ()`` at 200 coupled steps
  for both schemes). So the golden run is fully covered by those behavioural gates and
  the golden only *adds* the bit-pin. ``_final_state`` re-asserts ``rationed`` /
  ``events`` as belt-and-suspenders: the golden is a well-fed, no-extinction run.
* **Regeneration is a separate, explicit action** (run the module as a script),
  never a side effect of a test run: a verify run is strictly read-only. Folding regen
  into the test body under an env flag risks an ambiently-set flag silently overwriting
  the golden and passing — masking the very regression this gate exists to catch.

Bit-stability: the demo flows touch only ``+ − × ÷`` (no transcendentals), every op
IEEE-754 correctly-rounded, reduced in canonical id-order (#15) — so the golden is
well-defined and platform-stable, matching the project's "bit-identical within a
build" contract. The always-on conservation gate also fires every step *during* the
run (the integrator's ``_finalize`` tail), so a conservation regression raises before
any bytes are even produced.
"""

from pathlib import Path

import pytest

import sim_io
from domains.biosphere.demo import build_demo, coupled_resolver, run
from domains.biosphere.loader import load_demo_params
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

# Fixed golden run length — matches the step-10 well-fed gates (``test_well_fed_*``),
# which already assert rationed==0 / events==() at 200 coupled steps for both schemes.
GOLDEN_STEPS = 200

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"

# The committed canonical demo params (params/demo.yaml is the single source of truth).
DEMO_PARAMS = load_demo_params()

# One golden per integrator: Euler and RK4 produce different trajectories, so each
# scheme's arithmetic is pinned in its own file.
_GOLDENS: tuple[tuple[type, str], ...] = (
    (EulerIntegrator, "demo_euler_state.json"),
    (Rk4Integrator, "demo_rk4_state.json"),
)


def _final_state(integrator_cls: type) -> State:
    """Run the demo ``GOLDEN_STEPS`` coupled, well-fed steps; return the final State.

    The single source of truth for both the committed goldens and the verify tests.
    Asserts the run is well-fed and extinction-free — the golden is produced from a
    non-arbitrating, non-extinction trajectory by construction.
    """
    state, reg = build_demo(DEMO_PARAMS)
    final, rationed, events = run(
        integrator_cls(reg), state, coupled_resolver(), DEMO_PARAMS.dt, GOLDEN_STEPS
    )
    assert rationed == 0, "golden run must be well-fed (no arbitration firing)"
    assert events == (), "golden run must be extinction-free"
    return final


@pytest.mark.parametrize("integrator_cls,filename", _GOLDENS)
def test_demo_golden_bytes_match(integrator_cls: type, filename: str) -> None:
    # Byte-exact (not text-mode) compare against the committed golden — the cross-run /
    # cross-port conformance target. Any bit change in the demo output fails here.
    expected = sim_io.dumps(_final_state(integrator_cls)).encode("utf-8")
    assert expected == (GOLDEN_DIR / filename).read_bytes()


@pytest.mark.parametrize("integrator_cls,filename", _GOLDENS)
def test_demo_golden_loads_back(integrator_cls: type, filename: str) -> None:
    # The committed golden round-trips back to the exact final State (reconstruction
    # routes through the core constructors, so a tampered golden would fail to load).
    text = (GOLDEN_DIR / filename).read_text(encoding="utf-8")
    assert sim_io.loads(text) == _final_state(integrator_cls)


def _regenerate() -> None:
    """Rewrite the committed goldens from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run — so a
    verify run can never overwrite the golden it is meant to check. Run via::

        uv run python tests/test_regression_demo.py

    Review the diff before committing: a change here means the engine output moved.
    """
    for integrator_cls, filename in _GOLDENS:
        path = GOLDEN_DIR / filename
        path.write_bytes(sim_io.dumps(_final_state(integrator_cls)).encode("utf-8"))
        print(f"wrote {path}")


if __name__ == "__main__":
    _regenerate()
