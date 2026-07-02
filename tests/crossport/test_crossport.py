"""Phase-7 Step 0 (P7.0) cross-port harness tests.

Four gates:

1. The committed hex-float conformance vectors stay in sync with their generator
   (the Rust codec test reads the file; drift would silently weaken it).
2. `tiers.json` classifies exactly the 20 frozen goldens (7 biosphere + 13 station),
   each with a consistent tier / transcendental-free verdict and graph evidence.
3. **The Step-0 acceptance:** a Python `State` round-trips through the Rust
   codec + snapshot emitter and back — i.e. the Rust `emit_crew` example's JSON,
   read by `sim_io.loads` and re-emitted by `sim_io.dumps`, reproduces the
   `crew_state` golden byte-for-byte.
4. The comparator applies the tier rules correctly (self-identity passes at both
   tiers; a 1-ULP flip fails Tier 1 but is absorbed by a Tier-2 band; a gross
   perturbation fails Tier 2).
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
import gen_vectors  # noqa: E402

from sim_io import snapshot  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
GOLDEN_DIR = REPO_ROOT / "tests" / "regression" / "golden"
RUST_CRATE_DIR = REPO_ROOT / "rust" / "crates" / "simcore"
TIERS_PATH = Path(__file__).parent / "tiers.json"


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
        # Bands are unmeasured until a later step produces port numbers.
        assert g["band"] is None and g["floor"] is None, (
            f"{g['golden']}: band/floor must stay null until measured vs port output"
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
