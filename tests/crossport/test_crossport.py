"""Phase-7 cross-port harness tests (Step 0 P7.0 + Step 1 P7.1).

Gates:

1. The committed hex-float conformance vectors stay in sync with their generator
   (the Rust codec test reads the file; drift would silently weaken it).
1a. **(Step 1)** The committed RNG conformance vectors stay in sync with
   `gen_rng_vectors.py`, AND their fixed rows are *anchored* to the hand-pinned
   known answers in `tests/test_rng.py` (`_GOLDEN` / `_SPLITMIX64_SEED0`, grounded
   against published splitmix64) — so the generated file is externally anchored,
   not self-referential.
2. `tiers.json` classifies exactly the 20 frozen goldens (7 biosphere + 13 station),
   each with a consistent tier / transcendental-free verdict and graph evidence.
3. **The Step-0 acceptance:** a Python `State` round-trips through the Rust
   codec + snapshot emitter and back — i.e. the Rust `emit_crew` example's JSON,
   read by `sim_io.loads` and re-emitted by `sim_io.dumps`, reproduces the
   `crew_state` golden byte-for-byte.
4. The comparator applies the tier rules correctly (self-identity passes at both
   tiers; a 1-ULP flip fails Tier 1 but is absorbed by a Tier-2 band; a gross
   perturbation fails Tier 2).

The Rust side of the Step-1 RNG gate (bit-exact `draw_u64` / `draw` against the
committed vectors) lives in `rust/crates/simcore/tests/rng_vectors.rs`.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

# The crossport helpers live beside this test; make them importable regardless of
# pytest's import mode.
sys.path.insert(0, str(Path(__file__).parent))

import compare  # noqa: E402
import gen_biosphere_params  # noqa: E402
import gen_biosphere_weather  # noqa: E402
import gen_engine_vectors  # noqa: E402
import gen_rng_vectors  # noqa: E402
import gen_sibling_params  # noqa: E402
import gen_station_params  # noqa: E402
import gen_vectors  # noqa: E402
import measure_tier2_bands  # noqa: E402

from sim_io import snapshot  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
GOLDEN_DIR = REPO_ROOT / "tests" / "regression" / "golden"
RUST_CRATE_DIR = REPO_ROOT / "rust" / "crates" / "simcore"
RUST_DOMAINS_DIR = REPO_ROOT / "rust" / "crates" / "domains"
TIERS_PATH = Path(__file__).parent / "tiers.json"

# The hand-pinned RNG known-answer vectors live in tests/test_rng.py; import them to
# anchor the generated cross-port file to external truth (published splitmix64).
sys.path.insert(0, str(REPO_ROOT / "tests"))
from test_rng import _GOLDEN as _RNG_GOLDEN  # noqa: E402
from test_rng import _SPLITMIX64_SEED0  # noqa: E402

# --------------------------------------------------------------------------- #
# 1. Vector file in sync with its generator                                   #
# --------------------------------------------------------------------------- #


def test_hexfloat_vectors_in_sync() -> None:
    """The committed vector file equals `gen_vectors.render()` (regen discipline)."""
    on_disk = gen_vectors.VECTORS_PATH.read_text(encoding="utf-8").replace("\r\n", "\n")
    assert on_disk == gen_vectors.render(), (
        "hex-float vectors are stale — regenerate with "
        "`uv run python tests/crossport/gen_vectors.py`"
    )


def test_rng_vectors_in_sync() -> None:
    """The committed RNG vector file equals `gen_rng_vectors.render()` (regen
    discipline — the Rust `tests/rng_vectors.rs` reads this exact file)."""
    on_disk = gen_rng_vectors.VECTORS_PATH.read_text(encoding="utf-8").replace(
        "\r\n", "\n"
    )
    assert on_disk == gen_rng_vectors.render(), (
        "RNG vectors are stale — regenerate with "
        "`uv run python tests/crossport/gen_rng_vectors.py`"
    )


def test_engine_vectors_in_sync() -> None:
    """The committed engine trajectory file equals `gen_engine_vectors.render()`
    (regen discipline — the Rust `tests/engine_vectors.rs` gates against this exact
    file bit-for-bit under Euler / RK4 / multi-rate / rationing). No external anchor
    is needed: `src/simcore` *is* the cross-port reference, so proving Rust == Python
    is the whole goal (unlike the RNG's published-splitmix64 grounding)."""
    on_disk = gen_engine_vectors.VECTORS_PATH.read_text(encoding="utf-8").replace(
        "\r\n", "\n"
    )
    assert on_disk == gen_engine_vectors.render(), (
        "engine vectors are stale — regenerate with "
        "`uv run python tests/crossport/gen_engine_vectors.py`"
    )


def test_sibling_params_in_sync() -> None:
    """The committed sibling-param file equals `gen_sibling_params.render()` (regen
    discipline — the Rust `domains` crate `include_str!`s this exact file to get the
    12 Phase-5 coefficients, each loaded through its frozen Python loader). No external
    anchor is needed: the Python loaders + the ~1300-test suite ground the values, and
    decimal params round-trip bit-identically across correctly-rounding parsers."""
    on_disk = gen_sibling_params.PARAMS_PATH.read_text(encoding="utf-8").replace(
        "\r\n", "\n"
    )
    assert on_disk == gen_sibling_params.render(), (
        "sibling params are stale — regenerate with "
        "`uv run python tests/crossport/gen_sibling_params.py`"
    )


def test_biosphere_params_in_sync() -> None:
    """The committed biosphere-param file equals `gen_biosphere_params.render()` (Step
    4, P7.4 — the Rust biosphere `include_str!`s it to get the 13 frozen crop params as
    core-ready hex-floats + the partition table). Same rationale as the sibling params:
    the frozen loaders + the golden suite ground the values."""
    on_disk = gen_biosphere_params.PARAMS_PATH.read_text(encoding="utf-8").replace(
        "\r\n", "\n"
    )
    assert on_disk == gen_biosphere_params.render(), (
        "biosphere params are stale — regenerate with "
        "`uv run python tests/crossport/gen_biosphere_params.py`"
    )


def test_biosphere_weather_facts_in_sync() -> None:
    """The committed raw-weather-facts file equals `gen_biosphere_weather.render()`
    (Step 4, P7.4 — the Rust `biosphere::weather` reads it and runs the clean-room
    conversions ITSELF, exercising the daylength sin/tan/acos + vpd exp cross-port).
    Source of truth: the committed oracle fixture."""
    on_disk = gen_biosphere_weather.FACTS_PATH.read_text(encoding="utf-8").replace(
        "\r\n", "\n"
    )
    assert on_disk == gen_biosphere_weather.render(), (
        "biosphere weather facts are stale — regenerate with "
        "`uv run python tests/crossport/gen_biosphere_weather.py`"
    )


def test_rng_vectors_anchor_to_published_known_answers() -> None:
    """Dissolve the circularity: the generated file is computed from `CounterRng`,
    so on its own it only proves Rust == Python. This test anchors its fixed rows to
    the hand-pinned known-answer vectors in `tests/test_rng.py` — which are grounded
    against *published* splitmix64 output — so the full chain is Rust == file ==
    Python, and Python == published. Without this, the whole cross-port RNG gate
    would be self-referential.
    """
    text = gen_rng_vectors.render()
    mix_rows: dict[int, int] = {}
    draw_rows: dict[tuple[int, tuple[int, ...], int], tuple[int, str]] = {}
    for line in text.splitlines():
        if line.startswith("#") or not line.strip():
            continue
        fields = line.split("\t")
        if fields[0] == "mix64":
            mix_rows[int(fields[1], 16)] = int(fields[2], 16)
        elif fields[0] == "draw":
            seed = int(fields[1], 16)
            key = tuple(int(w) for w in fields[2].split(",") if w)
            step = int(fields[3])
            draw_rows[(seed, key, step)] = (int(fields[4], 16), fields[5])

    # Anchor the mix64 primitive: splitmix64(seed=0) emits finalizer(i*GAMMA) for
    # i = 1, 2, 3 — the published constants in _SPLITMIX64_SEED0.
    gamma = 0x9E3779B97F4A7C15
    mask64 = 0xFFFFFFFFFFFFFFFF
    for i, expected in enumerate(_SPLITMIX64_SEED0, start=1):
        assert mix_rows[(i * gamma) & mask64] == expected, (
            f"mix64 vector for i={i} does not match published splitmix64(seed=0)"
        )

    # Anchor the keyed draws: every _GOLDEN row's (u64, float_hex) must appear in the
    # generated file with the pinned values.
    for seed, key, step, u64, fhex in _RNG_GOLDEN:
        assert draw_rows[(seed, key, step)] == (u64, fhex), (
            f"draw vector for (seed={seed:#x}, key={key}, step={step}) does not "
            f"match the hand-pinned _GOLDEN known answer"
        )


# --------------------------------------------------------------------------- #
# 2. tiers.json classifies all 20 frozen goldens, consistently                #
# --------------------------------------------------------------------------- #


def _frozen_goldens() -> set[str]:
    """The 20 frozen golden filenames, read from the two freeze manifests."""
    bio = json.loads(
        (REPO_ROOT / "docs" / "biosphere-reference.manifest.json").read_text()
    )
    sta = json.loads(
        (REPO_ROOT / "docs" / "station-reference.manifest.json").read_text()
    )
    names: set[str] = set()
    for manifest in (bio, sta):
        for entry in manifest["scenarios"].values():
            names.add(entry["golden"])
    return names


def _tiers() -> list[dict]:
    return json.loads(TIERS_PATH.read_text(encoding="utf-8"))["goldens"]


def test_tiers_json_covers_exactly_the_frozen_goldens() -> None:
    frozen = _frozen_goldens()
    classified = {g["golden"] for g in _tiers()}
    assert classified == frozen, (
        f"tiers.json must classify exactly the 20 frozen goldens; "
        f"missing={frozen - classified}, extra={classified - frozen}"
    )
    assert len(frozen) == 20, f"expected 20 frozen goldens, got {len(frozen)}"


def test_tiers_entries_are_internally_consistent() -> None:
    required = {
        "golden",
        "scenario",
        "domain_group",
        "kind",
        "transcendental_free",
        "float_tier",
        "transcendentals",
        "evidence",
        "band",
        "floor",
    }
    for g in _tiers():
        missing = required - set(g)
        assert not missing, f"{g.get('golden')}: missing fields {missing}"
        assert g["float_tier"] in (1, 2), g
        assert isinstance(g["transcendental_free"], bool), g
        # Tier 1 iff transcendental-free — the discriminating rule.
        assert (g["float_tier"] == 1) == g["transcendental_free"], (
            f"{g['golden']}: float_tier {g['float_tier']} inconsistent with "
            f"transcendental_free {g['transcendental_free']}"
        )
        if g["float_tier"] == 1:
            assert g["transcendentals"] == [], (
                f"{g['golden']}: Tier-1 must list no transcendentals"
            )
        else:
            assert g["transcendentals"], (
                f"{g['golden']}: Tier-2 must cite ≥1 transcendental site"
            )
        # Tier 1 is bit-exact — no band/floor. Tier-2 bands are measured, never derived
        # (compare.py), so a Tier-2 entry is EITHER unmeasured (both null) OR measured
        # (both positive floats); a measured band is justified against a fresh
        # sensitivity measurement by test_tier2_bands_sit_above_measured_sensitivity
        # (Step 3, P7.3). Step 3 measured exactly the three standalone-sibling bands.
        if g["float_tier"] == 1:
            assert g["band"] is None and g["floor"] is None, (
                f"{g['golden']}: Tier-1 is bit-exact — band/floor must be null"
            )
        else:
            both_null = g["band"] is None and g["floor"] is None
            both_measured = (
                isinstance(g["band"], (int, float))
                and not isinstance(g["band"], bool)
                and isinstance(g["floor"], (int, float))
                and not isinstance(g["floor"], bool)
                and g["band"] > 0
                and g["floor"] > 0
            )
            assert both_null or both_measured, (
                f"{g['golden']}: Tier-2 band/floor must be both-null (unmeasured) or "
                f"both-positive (measured), got band={g['band']!r} floor={g['floor']!r}"
            )


def test_tier1_set_is_the_four_transcendental_free_scenarios() -> None:
    """Pins the Step-0 verdict: exactly crew / eclss / cabin_gas / water_recovery
    are bit-exact candidates (advisor-flagged, grep-confirmed)."""
    tier1 = {g["golden"] for g in _tiers() if g["float_tier"] == 1}
    assert tier1 == {
        "crew_state.json",
        "eclss_state.json",
        "cabin_gas_state.json",
        "water_recovery_state.json",
    }


def test_power_is_tier2_not_tier1() -> None:
    """The plan's explicit check: power is NOT transcendental-free (half-sine)."""
    by_name = {g["golden"]: g for g in _tiers()}
    assert by_name["power_state.json"]["float_tier"] == 2
    assert not by_name["power_state.json"]["transcendental_free"]
    assert any("sin" in t for t in by_name["power_state.json"]["transcendentals"])


def test_tier2_bands_sit_above_measured_sensitivity() -> None:
    """The three Step-3 Tier-2 bands (power / power_self_discharge / thermal) are
    MEASURED, not derived: each committed `tiers.json` band sits above the freshly
    re-measured ±1-ULP transcendental sensitivity (`measure_tier2_bands.py`). This is
    the honest provenance the relaxed `test_tiers_entries_are_internally_consistent`
    relies on, and it is pure Python (no `cargo`) so it runs on CI. It also pins the
    band *tight* (≤ 1e-9) so a real port defect still trips Tier 2."""
    by_key = {g["key"]: g for g in _tiers()}
    for key in measure_tier2_bands.STEP3_TIER2_KEYS:
        entry = by_key[key]
        assert entry["float_tier"] == 2, key
        band, floor = entry["band"], entry["floor"]
        assert band is not None and floor is not None, f"{key}: band/floor unmeasured"
        sensitivity = measure_tier2_bands.measured_sensitivity(key)
        assert sensitivity < band, (
            f"{key}: measured ±1-ULP sensitivity {sensitivity:.3e} is not below the "
            f"tiers.json band {band:.3e}"
        )
        assert band <= 1e-9, (
            f"{key}: band {band:.3e} is too loose — a Tier-2 band must still catch a "
            f"real port defect"
        )


# --------------------------------------------------------------------------- #
# 3. THE ACCEPTANCE: Python State ⇄ Rust codec/emitter ⇄ Python loads          #
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
def test_rust_crew_snapshot_roundtrips_through_sim_io() -> None:
    """Run the Rust `emit_crew` example, parse its JSON with `sim_io.loads`, and
    assert re-emitting it reproduces the frozen `crew_state` golden byte-for-byte.

    This exercises the full Step-0 interchange: the Rust hex-float codec (amounts
    are parsed from the golden's own hex strings) and the Rust snapshot emitter,
    validated by the Python reader — parsed f64 values, never JSON bytes.
    """
    proc = subprocess.run(
        ["cargo", "run", "-q", "--example", "emit_crew"],
        cwd=RUST_CRATE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo run failed:\n{proc.stderr}"
    rust_json = proc.stdout

    state = snapshot.loads(rust_json)
    reemitted = snapshot.dumps(state)

    golden_text = (
        (GOLDEN_DIR / "crew_state.json")
        .read_text(encoding="utf-8")
        .replace("\r\n", "\n")
    )
    assert reemitted == golden_text, (
        "Rust-emitted crew snapshot did not round-trip to the golden through "
        "sim_io.loads/dumps"
    )


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
def test_rust_composite_snapshot_exercises_aux_and_multicomposition() -> None:
    """The `crew_state` acceptance leaves two emitter branches untested: a non-empty
    `aux` map and a multi-element `composition` (its inter-coefficient comma logic).
    Both are live paths for real frozen data (biosphere `thermal_time` aux; the CO₂
    `{carbon:1, oxygen:2}` stocks in cabin_gas/greenhouse/sealed). The `emit_composite`
    example builds a synthetic State reaching both; here we confirm `sim_io.loads`
    accepts it and the aux value + every composition coefficient survive intact.
    """
    from simcore.ids import StockId
    from simcore.quantities import Quantity

    proc = subprocess.run(
        ["cargo", "run", "-q", "--example", "emit_composite"],
        cwd=RUST_CRATE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo run failed:\n{proc.stderr}"

    state = snapshot.loads(proc.stdout)

    # Non-empty aux branch.
    assert state.aux["thermal_time"] == float.fromhex("0x1.31851eb851eb8p+13")
    # Multi-element composition (comma logic) — CO₂ as {carbon:1, oxygen:2}.
    co2 = state.stocks[StockId("boundary.co2_removed")].composition
    assert co2 == {Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0}
    # Single-element composition with a non-unit coefficient.
    o2 = state.stocks[StockId("eclss.cabin_o2")].composition
    assert o2 == {Quantity.OXYGEN: 2.0}
    # Non-zero seed round-trips through the 0x-hex string form.
    assert state.rng_seed == 0xDEADBEEF

    # The emitter's output is a stable fixed point of the canonical Python form.
    assert snapshot.dumps(state) == snapshot.dumps(
        snapshot.loads(snapshot.dumps(state))
    )


# --------------------------------------------------------------------------- #
# 3b. Step-3 (P7.3): the four ported siblings run in Rust and match their tier #
# --------------------------------------------------------------------------- #

# (example binary, golden filename, tiers.json key). crew/eclss are Tier-1 (bit-exact);
# power/power_self_discharge/thermal are Tier-2 (measured band from tiers.json).
_SIBLING_CASES = [
    ("emit_crew", "crew_state.json", "crew_mission"),
    ("emit_eclss", "eclss_state.json", "eclss_steady_state"),
    ("emit_power", "power_state.json", "power_bounded_soc"),
    (
        "emit_power_self_discharge",
        "power_self_discharge_state.json",
        "power_self_discharge",
    ),
    ("emit_thermal", "thermal_state.json", "thermal_equilibrium"),
]


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
@pytest.mark.parametrize("example,golden,key", _SIBLING_CASES)
def test_rust_siblings_match_their_tier(example: str, golden: str, key: str) -> None:
    """Run each ported standalone sibling (`build_*` + Euler `run` in the Rust `domains`
    crate) and compare its final `State` to the frozen golden at its assigned tier:
    **crew / eclss Tier-1 bit-exact** (transcendental-free — the real proof the engine
    computes the frozen values, not just that Step-0's hand-built ones round-trip);
    **power / power_self_discharge / thermal Tier-2** within the measured `tiers.json`
    band. Compared on parsed f64 (via `sim_io.loads`), never JSON bytes.

    NOTE: `skipif cargo is None` and the Python CI job installs no Rust, so this parity
    gate — including the crew/eclss bit-exact claims — is LOCAL-ONLY, never on CI
    (pre-existing Step-0 precedent). A real cross-libm CI gate (Rust in the Python job,
    or a committed Linux-generated golden) is deferred future work.
    """
    proc = subprocess.run(
        ["cargo", "run", "-q", "--example", example],
        cwd=RUST_DOMAINS_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo run {example} failed:\n{proc.stderr}"

    # Validate it parses as a snapshot, then compare parsed values (port-agnostic).
    snapshot.loads(proc.stdout)
    candidate = json.loads(proc.stdout)
    reference = compare.load_json(GOLDEN_DIR / golden)

    entry = {g["key"]: g for g in _tiers()}[key]
    tier = entry["float_tier"]
    if tier == compare.TIER_1_BIT_EXACT:
        result = compare.compare(reference, candidate, tier=tier)
    else:
        result = compare.compare(
            reference, candidate, tier=tier, band=entry["band"], floor=entry["floor"]
        )
    assert result.ok, f"{example} vs {golden} (tier {tier}):\n{result.report()}"
    assert result.numeric_pairs, "expected numeric leaves to be compared"


# --------------------------------------------------------------------------- #
# 3c. Step-4 (P7.4): the ported biosphere runs match their Tier-2 band         #
# --------------------------------------------------------------------------- #

# (example binary, cli args, golden filename, tiers.json key). All 7 are Tier-2.
_BIOSPHERE_STATE_CASES = [
    ("emit_season", [], "season_euler_state.json", "open_season"),
    ("emit_sealed", [], "sealed_chamber_state.json", "sealed_chamber"),
    ("emit_perennial", [], "perennial_chamber_state.json", "perennial_chamber"),
    (
        "emit_perennial",
        ["long"],
        "perennial_long_horizon_state.json",
        "perennial_long_horizon",
    ),
    ("emit_consumer", [], "consumer_chamber_state.json", "consumer_chamber"),
    (
        "emit_consumer",
        ["long"],
        "consumer_long_horizon_state.json",
        "consumer_long_horizon",
    ),
]

# The transient (in years) dropped before the period-2 check — the value the frozen
# drift_summary golden was captured with (long_horizon test's _PERIOD_TRANSIENT).
_DRIFT_PERIOD_TRANSIENT = 8


def _run_example(example: str, args: list[str]) -> dict:
    """Run a Rust biosphere emit example and parse its JSON stdout."""
    proc = subprocess.run(
        ["cargo", "run", "-q", "--example", example, *(["--", *args] if args else [])],
        cwd=RUST_DOMAINS_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo run {example} {args} failed:\n{proc.stderr}"
    return json.loads(proc.stdout)


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
@pytest.mark.parametrize("example,args,golden,key", _BIOSPHERE_STATE_CASES)
def test_rust_biosphere_states_match_tier2(
    example: str, args: list[str], golden: str, key: str
) -> None:
    """Run each ported biosphere scenario (`build_season` + `run_season`/`run_perennial`
    Euler-daily) and compare its final `State` to the frozen golden at **Tier 2** (the
    measured `tiers.json` band). Every FvCB/transpiration/weather transcendental runs in
    Rust; locally (same UCRT libm) the deviation is bit-exact 0.0, well inside the band.
    Compared on parsed f64 (via `sim_io.loads`), never JSON bytes.

    LOCAL-ONLY (`skipif cargo` + no Rust in the Python CI job) — the Step-0/3 precedent.
    """
    candidate = _run_example(example, args)
    snapshot.loads(json.dumps(candidate))  # validate it is a well-formed snapshot
    reference = compare.load_json(GOLDEN_DIR / golden)
    entry = {g["key"]: g for g in _tiers()}[key]
    result = compare.compare(
        reference,
        candidate,
        tier=entry["float_tier"],
        band=entry["band"],
        floor=entry["floor"],
    )
    assert result.ok, f"{example} {args} vs {golden}:\n{result.report()}"
    assert result.numeric_pairs, "expected numeric leaves to be compared"


def _fold_drift_summary(raw: dict) -> dict:
    """Fold the Rust raw per-step series into the `drift_summary` shape, Python-side.

    The plan (advisor #3) keeps ALL of `drift.py` Python-side: Rust emits the raw
    `leaf_c` / `consumer_carbon` trajectories; here we apply the same per-year
    segmentation the golden used (`year_summaries`: peak over each
    `[y*year:(y+1)*year+1]` segment; year-end = the segment's last state) and the same
    `is_period_2` classifier — so no segmentation logic lives in Rust.
    """
    from domains.biosphere.drift import is_period_2

    year = raw["season_days"]
    horizon = raw["horizon_years"]

    def series(name: str) -> list[float]:
        return [float.fromhex(h) for h in raw[name]]

    p_leaf_s, c_leaf_s, c_carbon_s = (
        series("perennial_leaf"),
        series("consumer_leaf"),
        series("consumer_carbon"),
    )
    n = (len(p_leaf_s) - 1) // year

    def peak(s: list[float]) -> list[float]:
        return [max(s[y * year : (y + 1) * year + 1]) for y in range(n)]

    def year_end(s: list[float]) -> list[float]:
        return [s[(y + 1) * year] for y in range(n)]

    p_leaf, c_leaf, c_carbon = peak(p_leaf_s), peak(c_leaf_s), year_end(c_carbon_s)
    return {
        "horizon_years": horizon,
        "perennial": {
            "peak_leaf": [v.hex() for v in p_leaf],
            "is_period_2": is_period_2(p_leaf, transient=_DRIFT_PERIOD_TRANSIENT),
        },
        "consumer": {
            "peak_leaf": [v.hex() for v in c_leaf],
            "consumer_carbon": [v.hex() for v in c_carbon],
            "is_period_2": is_period_2(c_leaf, transient=_DRIFT_PERIOD_TRANSIENT),
        },
    }


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
def test_rust_biosphere_drift_summary_matches() -> None:
    """The Rust 15-yr perennial + consumer runs reproduce the `drift_summary` golden:
    the per-year `peak_leaf`/`consumer_carbon` vectors within the Tier-2 band, and the
    `is_period_2` stability signature (perennial period-2, consumer period-1) EXACTLY
    (Tier 0 — a flipped period class is a real port bug, the plan's primary gate).

    Rust emits only the raw per-step series; `_fold_drift_summary` applies `drift.py`
    Python-side (advisor #3), so the classifier is never ported.
    """
    raw = _run_example("emit_drift", [])
    derived = _fold_drift_summary(raw)
    golden = compare.load_json(GOLDEN_DIR / "drift_summary.json")
    entry = {g["key"]: g for g in _tiers()}["drift_summary"]
    result = compare.compare(
        golden,
        derived,
        tier=entry["float_tier"],
        band=entry["band"],
        floor=entry["floor"],
    )
    assert result.ok, f"drift_summary parity:\n{result.report()}"
    # Tier-0: the period class is a classification — exact, and the headline result.
    assert derived["perennial"]["is_period_2"] is True, "perennial must be period-2"
    assert derived["consumer"]["is_period_2"] is False, "consumer must be period-1"


@pytest.mark.slow
def test_biosphere_tier2_band_sits_above_measured_sensitivity() -> None:
    """The shared biosphere Tier-2 band (`1e-11`, all 7 goldens) sits above the freshly
    re-measured ±1-ULP transcendental sensitivity — the "measured, never derived"
    provenance (`measure_tier2_bands.measured_biosphere_sensitivity`, the representative
    worst-case `canopy.exp` over both 15-yr runs). Marked `-m slow` (six 15-yr runs),
    unlike the CI-fast Step-3 band test. Also pins the band tight (<= 1e-9) so a real
    port defect still trips Tier 2.
    """
    by_key = {g["key"]: g for g in _tiers()}
    sensitivity = measure_tier2_bands.measured_biosphere_sensitivity()
    for key in measure_tier2_bands.BIOSPHERE_TIER2_KEYS:
        entry = by_key[key]
        assert entry["float_tier"] == 2, key
        band = entry["band"]
        assert band == measure_tier2_bands.BIOSPHERE_BAND, (
            f"{key}: band {band!r} != the shared BIOSPHERE_BAND"
        )
        assert sensitivity < band, (
            f"{key}: 1-ULP sensitivity {sensitivity:.3e} not below band {band:.3e}"
        )
        assert band <= 1e-9, f"{key}: band {band:.3e} too loose to catch a port defect"


# --------------------------------------------------------------------------- #
# 3d. Step-5 (P7.5): the ported station assemblies match their tier            #
# --------------------------------------------------------------------------- #

RUST_WORKSPACE_DIR = REPO_ROOT / "rust"

# (example binary, golden filename, tiers.json key). The 6 fast station State goldens:
# cabin_gas / water_recovery Tier-1 bit-exact (transcendental-free, cabin-only);
# station / greenhouse / lighting / harvest Tier-2 (sin+T^4 or FvCB in the graph). The
# ~1.3M-substep sealed_station and the 15-yr energy drift are separate slow tests below.
_STATION_STATE_CASES = [
    ("emit_cabin_gas", "cabin_gas_state.json", "cabin_gas"),
    ("emit_water_recovery", "water_recovery_state.json", "water_recovery"),
    ("emit_station", "station_state.json", "station_heat_closure"),
    ("emit_greenhouse", "greenhouse_state.json", "greenhouse"),
    ("emit_lighting", "lighting_state.json", "lighting"),
    ("emit_harvest", "harvest_state.json", "harvest"),
]


def _run_station_example(example: str, *, release: bool = False) -> str:
    """Run a Rust `station` emit example from the workspace; return its stdout."""
    cmd = ["cargo", "run", "-q"]
    if release:
        cmd.append("--release")
    cmd += ["-p", "station", "--example", example]
    proc = subprocess.run(cmd, cwd=RUST_WORKSPACE_DIR, capture_output=True, text=True)
    assert proc.returncode == 0, f"cargo run {example} failed:\n{proc.stderr}"
    return proc.stdout


def _station_entry(key: str) -> dict:
    return {g["key"]: g for g in _tiers()}[key]


@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
@pytest.mark.parametrize("example,golden,key", _STATION_STATE_CASES)
def test_rust_station_states_match_their_tier(
    example: str, golden: str, key: str
) -> None:
    """Run each ported station assembly (`build_*` + the single- or two-rate runner in
    the Rust `station` crate) and compare its final `State` to the frozen golden at its
    assigned tier: **cabin_gas / water_recovery Tier-1 bit-exact** (transcendental-free
    — the real proof the engine computes the frozen coupled values); **station /
    greenhouse / lighting / harvest Tier-2** within the measured `tiers.json` band.
    Compared on parsed f64 (via `sim_io.loads`), never JSON bytes.

    LOCAL-ONLY (`skipif cargo` + no Rust in the Python CI job) — the Step-0/3/4
    precedent. The two-rate goldens' Tier-0 conservation gate (every sub-step, in Rust)
    fired inside the driver during the emit run — a completed run is itself the proof.
    """
    candidate = json.loads(_run_station_example(example))
    snapshot.loads(json.dumps(candidate))  # validate it is a well-formed snapshot
    reference = compare.load_json(GOLDEN_DIR / golden)
    entry = _station_entry(key)
    tier = entry["float_tier"]
    if tier == compare.TIER_1_BIT_EXACT:
        result = compare.compare(reference, candidate, tier=tier)
    else:
        result = compare.compare(
            reference, candidate, tier=tier, band=entry["band"], floor=entry["floor"]
        )
    assert result.ok, f"{example} vs {golden} (tier {tier}):\n{result.report()}"
    assert result.numeric_pairs, "expected numeric leaves to be compared"


@pytest.mark.slow
@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
def test_rust_sealed_station_matches_tier2() -> None:
    """The ~1.3 M-sub-step sealed-station run (release build, ~1 min) reproduces
    `sealed_station_state.json` at Tier 2. Its real payload is the per-sub-step
    conservation assert inside the two-rate driver (the Tier-0 gate): a completed run is
    itself proof the five-domain ledger balanced every sub-step over the run."""
    candidate = json.loads(_run_station_example("emit_sealed_station", release=True))
    snapshot.loads(json.dumps(candidate))
    reference = compare.load_json(GOLDEN_DIR / "sealed_station_state.json")
    entry = _station_entry("sealed_station")
    result = compare.compare(
        reference, candidate, tier=2, band=entry["band"], floor=entry["floor"]
    )
    assert result.ok, f"sealed_station:\n{result.report()}"
    assert result.numeric_pairs


def _fold_energy_drift_summary(raw: dict) -> dict:
    """Fold the Rust raw per-step node-heat series into the drift-summary
    shape, Python-side (advisor #3 — all of `drift.py` stays Python-side). Rust emits
    only the raw `thermal.node` heat trajectory; here we apply `temp = space_temp +
    node/C`, the per-year peak segmentation (`year_summaries`' `[y*year:(y+1)*year+1]`
    slice), and the `is_stationary` classifier — no segmentation logic lives in Rust."""
    from domains.biosphere.drift import is_stationary, same_phase_diffs
    from domains.thermal.loader import load_thermal_params
    from station.scenario import SEALED_STATION_SEASON_DAYS

    thermal = load_thermal_params()
    node = [float.fromhex(h) for h in raw["node_heat"]]
    temps = [thermal.space_temperature + q / thermal.heat_capacity for q in node]
    year = raw["steps_per_day"] * SEALED_STATION_SEASON_DAYS
    n = (len(temps) - 1) // year
    peaks = [max(temps[y * year : (y + 1) * year + 1]) for y in range(n)]
    return {
        "horizon_years": raw["horizon_years"],
        "node_peak_temp_k": [v.hex() for v in peaks],
        "is_stationary": is_stationary(
            same_phase_diffs(peaks, period=1), bound=0.1, slope_tol=1e-3
        ),
    }


@pytest.mark.slow
@pytest.mark.skipif(shutil.which("cargo") is None, reason="cargo not installed")
def test_rust_sealed_energy_drift_summary_matches() -> None:
    """The Rust 15-yr Power→Thermal run reproduces the `sealed_energy_drift_summary`
    golden: the per-year `node_peak_temp_k` vector within the station Tier-2 band,
    and the `is_stationary` signature (the node's period-1 fixed point) EXACTLY
    (Tier 0 — a flipped classification is a real port bug). Rust emits only the raw
    per-step node-heat series; `_fold_energy_drift_summary` folds it Python-side."""
    raw = json.loads(_run_station_example("emit_sealed_energy_drift", release=True))
    derived = _fold_energy_drift_summary(raw)
    golden = compare.load_json(GOLDEN_DIR / "sealed_energy_drift_summary.json")
    entry = _station_entry("sealed_energy_drift")
    result = compare.compare(
        golden, derived, tier=2, band=entry["band"], floor=entry["floor"]
    )
    assert result.ok, f"sealed_energy_drift parity:\n{result.report()}"
    # Tier-0: the stability signature is a classification — exact, the headline result.
    assert derived["is_stationary"] is True, "the node must be a period-1 fixed point"


def test_station_params_in_sync() -> None:
    """The committed station-param file equals `gen_station_params.render()` (Step 5,
    P7.5 — the Rust `station` crate `include_str!`s it to get the 3 station-owned
    coefficients as hex-floats). Same rationale as the sibling params: the frozen
    station loaders ground the values, and decimals round-trip bit-exactly."""
    on_disk = gen_station_params.PARAMS_PATH.read_text(encoding="utf-8").replace(
        "\r\n", "\n"
    )
    assert on_disk == gen_station_params.render(), (
        "station params are stale — regenerate with "
        "`uv run python tests/crossport/gen_station_params.py`"
    )


def test_station_energy_tier2_bands_sit_above_measured_sensitivity() -> None:
    """The station_state / sealed_energy_drift bands (1e-12, Power→Thermal only) sit
    above the freshly re-measured ±1-ULP sin+t**4 sensitivity of the coupled 7-day run —
    the measured, never-derived provenance. Pure Python (no cargo), CI-fast (a 7-day
    single-rate run), like the Step-3 band test; pins the band tight (<= 1e-9)."""
    by_key = {g["key"]: g for g in _tiers()}
    sensitivity = measure_tier2_bands.measured_station_energy_sensitivity()
    for key in measure_tier2_bands.STATION_ENERGY_TIER2_KEYS:
        entry = by_key[key]
        assert entry["float_tier"] == 2, key
        band = entry["band"]
        assert band == 1e-12, f"{key}: band {band!r} != 1e-12"
        assert sensitivity < band, (
            f"{key}: 1-ULP sensitivity {sensitivity:.3e} not below band {band:.3e}"
        )
        assert band <= 1e-9, f"{key}: band {band:.3e} too loose to catch a port defect"


@pytest.mark.slow
def test_station_biosphere_tier2_bands_sit_above_measured_sensitivity() -> None:
    """The four biosphere-coupled station bands (greenhouse / lighting / harvest /
    sealed_station) share BIOSPHERE_BAND (1e-11), justified by the 7-day greenhouse
    `canopy.exp` measurement (NOT a sweep of the 1.3M-substep sealed run — the
    deliberate cost choice) + the regulator-erasure argument. Marked
    `-m slow` (the 7-day greenhouse FvCB run); pins the band tight (<= 1e-9)."""
    by_key = {g["key"]: g for g in _tiers()}
    sensitivity = measure_tier2_bands.measured_greenhouse_sensitivity()
    for key in measure_tier2_bands.STATION_BIOSPHERE_TIER2_KEYS:
        entry = by_key[key]
        assert entry["float_tier"] == 2, key
        band = entry["band"]
        assert band == measure_tier2_bands.BIOSPHERE_BAND, (
            f"{key}: band {band!r} != BIOSPHERE_BAND"
        )
        assert sensitivity < band, (
            f"{key}: 1-ULP sensitivity {sensitivity:.3e} not below band {band:.3e}"
        )
        assert band <= 1e-9, f"{key}: band {band:.3e} too loose to catch a port defect"


# --------------------------------------------------------------------------- #
# 4. Comparator applies the tier rules                                        #
# --------------------------------------------------------------------------- #


def test_compare_self_identity_passes_both_tiers() -> None:
    crew = compare.load_json(GOLDEN_DIR / "crew_state.json")
    thermal = compare.load_json(GOLDEN_DIR / "thermal_state.json")

    r1 = compare.compare(crew, crew, tier=compare.TIER_1_BIT_EXACT)
    assert r1.ok, r1.report()
    assert r1.numeric_pairs, "expected numeric leaves to be compared"

    r2 = compare.compare(
        thermal, thermal, tier=compare.TIER_2_BAND, band=1e-9, floor=1e-12
    )
    assert r2.ok, r2.report()
    assert r2.max_rel_dev == 0.0


def test_compare_tier1_detects_one_ulp_flip_but_tier2_absorbs_it() -> None:
    ref = compare.load_json(GOLDEN_DIR / "crew_state.json")
    cand = json.loads(json.dumps(ref))  # deep copy

    # Perturb one amount by a single ULP.
    stock = cand["stocks"][0]
    original = float.fromhex(stock["amount"])
    import math

    bumped = math.nextafter(original, math.inf)
    stock["amount"] = bumped.hex()

    r1 = compare.compare(ref, cand, tier=compare.TIER_1_BIT_EXACT)
    assert not r1.ok, "Tier 1 must catch a 1-ULP difference"
    assert any("bit mismatch" in d.detail for d in r1.numeric_diffs)

    # A modest Tier-2 band absorbs a single ULP (~2e-16 relative).
    r2 = compare.compare(ref, cand, tier=compare.TIER_2_BAND, band=1e-6, floor=1e-12)
    assert r2.ok, r2.report()


def test_compare_tier2_fails_on_gross_perturbation() -> None:
    ref = compare.load_json(GOLDEN_DIR / "crew_state.json")
    cand = json.loads(json.dumps(ref))
    stock = cand["stocks"][0]
    stock["amount"] = (2.0 * float.fromhex(stock["amount"])).hex()  # +100%

    r2 = compare.compare(ref, cand, tier=compare.TIER_2_BAND, band=1e-6, floor=1e-12)
    assert not r2.ok
    assert r2.max_rel_dev is not None and r2.max_rel_dev > 1e-6


def test_compare_tier2_requires_measured_band() -> None:
    """The comparator refuses to invent a tolerance (bands are measured)."""
    crew = compare.load_json(GOLDEN_DIR / "crew_state.json")
    with pytest.raises(ValueError, match="measured"):
        compare.compare(crew, crew, tier=compare.TIER_2_BAND)


def test_compare_detects_structural_mismatch() -> None:
    ref = compare.load_json(GOLDEN_DIR / "crew_state.json")
    cand = json.loads(json.dumps(ref))
    cand["stocks"][0]["quantity"] = "nitrogen"  # discrete field tamper
    cand["n"] = 999  # integer discrete tamper

    r = compare.compare(ref, cand, tier=compare.TIER_1_BIT_EXACT)
    assert not r.ok
    details = " ".join(d.detail for d in r.structural_diffs)
    assert "discrete mismatch" in details
