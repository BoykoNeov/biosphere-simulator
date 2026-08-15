"""The experiment seam: scoped, in-memory param overrides (``config.overrides``).

Plan of record: ``docs/plans/post-roadmap-value-switch-harness.md``. This is the "clean
path" of that plan's §5 — a real seam in the boundary rather than monkey-patching the
loaders at their use sites, which is what 42 throwaway probe scripts across 16 plan docs
have done until now.

The two properties that earn the seam its place, and which these tests exist to hold:

1. **Inert unless opened.** No frozen run enters a ``param_overrides`` block, so
   goldens, both manifests and the Rust mirror are untouched by its presence.
2. **It cannot silently do nothing.** The guard that separates this from a patch script.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from config.errors import ConfigError
from config.loader import load_yaml
from config.overrides import (
    OverrideError,
    UnusedOverrideError,
    apply_overrides,
    param_overrides,
)
from domains.biosphere.loader import (
    CANOPY_PARAMS_PATH,
    load_canopy_params,
    load_decomposition_params,
)

# --- 1. inert by default ------------------------------------------------------------


def test_no_active_override_returns_the_same_object_not_a_copy() -> None:
    """The frozen path pays one dict lookup and nothing else.

    Asserted on identity rather than equality, because a copy would be a real (if small)
    cost imposed on every frozen run by a facility none of them use.
    """
    data = {"parameters": {"extinction_coef": {"value": 0.6}}}
    assert apply_overrides(Path("canopy.yaml"), data) is data


def test_loading_outside_a_block_is_unchanged() -> None:
    before = load_canopy_params()
    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        load_canopy_params()
    assert load_canopy_params() == before


def test_a_file_the_active_override_does_not_name_is_untouched() -> None:
    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        # decomposition.yaml is loaded inside the block but is not named by it
        assert load_decomposition_params() == load_decomposition_params()
        load_canopy_params()  # consume, so the vacuity guard stays quiet


# --- 2. the override actually reaches the value -------------------------------------


def test_override_replaces_a_plain_field() -> None:
    assert load_canopy_params().extinction_coef == 0.6
    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        assert load_canopy_params().extinction_coef == 0.65


def test_override_reaches_a_FOLDED_input_which_is_why_it_sits_before_validation() -> (
    None
):
    """⚠ The design decision this test exists to pin.

    ``load_canopy_params`` folds ``specific_leaf_area`` and ``carbon_fraction`` into the
    single ``sla_per_mol_c`` field, so neither survives as a dataclass attribute. An
    override applied to the finished dataclass could not reach them at all; applied to
    the parsed YAML *before* validation, it can. That is the whole reason the seam sits
    in ``load_yaml`` rather than after the loaders.
    """
    base = load_canopy_params()
    with param_overrides({"canopy": {"carbon_fraction": 0.40}}):
        folded = load_canopy_params()
    assert folded.sla_per_mol_c != base.sla_per_mol_c
    # sla_per_mol_c = sla * M_C / carbon_fraction, so lowering the fraction RAISES it
    assert folded.sla_per_mol_c == pytest.approx(base.sla_per_mol_c * 0.45 / 0.40)


def test_override_reaches_a_loader_called_with_NO_path_argument() -> None:
    """``soil.py`` calls ``load_decomposition_params()`` path-less, on its default.

    Keying on the resolved path's stem covers that, but only because the default is
    still a concrete path by the time ``load_yaml`` sees it. Pinned because a seam
    that could not reach the soil params would be exactly the silent partial coverage
    this whole facility is built to rule out.
    """
    base = load_decomposition_params()
    with param_overrides({"decomposition": {"decomposition_rate": 0.02}}):
        assert load_decomposition_params().decomposition_rate == 0.02
    assert load_decomposition_params() == base


def test_an_overridden_value_still_goes_through_the_normal_bound_check() -> None:
    """Substituted values are not privileged — they meet every guard a file's would."""
    with (
        pytest.raises(ValueError, match="extinction_coef must be > 0"),
        param_overrides({"canopy": {"extinction_coef": -1.0}}),
    ):
        load_canopy_params()


def test_the_declared_unit_and_source_survive_an_override() -> None:
    """Only ``value`` is replaced, so the unit guards still run on the real unit."""
    raw = load_yaml(CANOPY_PARAMS_PATH)
    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        patched = load_yaml(CANOPY_PARAMS_PATH)
    before = raw["parameters"]["extinction_coef"]
    after = patched["parameters"]["extinction_coef"]
    assert after["value"] == 0.65
    assert after["unit"] == before["unit"]
    assert after["source"] == before["source"]


# --- 3. THE ANTI-VACUITY GUARD ------------------------------------------------------


def test_an_override_nothing_consumes_RAISES() -> None:
    """⚠ The property that separates this from a probe script.

    ``cc44b41`` repaired a cross-port gate that had gone vacuous because its probe
    shimmed a dead ``exp``; the audit behind the canopy-provenance work then found
    ``intercepted_fraction`` has no caller in ``src/`` at all. A value-switch facility
    has that failure mode by construction — patch the wrong name, get a clean run, read
    the baseline, report "no effect" as a finding. Here it is a hard error.
    """
    with (
        pytest.raises(UnusedOverrideError, match="never consumed"),
        param_overrides({"canopy": {"extinction_coef": 0.65}}),
    ):
        pass


def test_a_misspelled_stem_is_caught_by_the_vacuity_guard() -> None:
    with (
        pytest.raises(UnusedOverrideError, match="canopyy.extinction_coef"),
        param_overrides({"canopyy": {"extinction_coef": 0.65}}),
    ):
        load_canopy_params()


def test_a_partially_consumed_override_set_still_raises() -> None:
    """One good override does not license a silent second one."""
    with (
        pytest.raises(UnusedOverrideError, match="decomposition.decomposition_rate"),
        param_overrides(
            {
                "canopy": {"extinction_coef": 0.65},
                "decomposition": {"decomposition_rate": 0.02},
            }
        ),
    ):
        load_canopy_params()  # only this one is loaded


def test_the_guard_does_NOT_mask_an_exception_from_the_body() -> None:
    """The body's failure is the finding; a vacuity complaint on top would bury it."""
    with (
        pytest.raises(RuntimeError, match="the real failure"),
        param_overrides({"canopy": {"extinction_coef": 0.65}}),
    ):
        raise RuntimeError("the real failure")


def test_an_unknown_param_name_raises_and_names_the_known_ones() -> None:
    with (
        pytest.raises(OverrideError, match="no parameter 'k_litter'"),
        param_overrides({"decomposition": {"k_litter": 0.02}}),
    ):
        load_decomposition_params()


def test_a_file_with_no_parameters_mapping_raises(tmp_path: Path) -> None:
    odd = tmp_path / "canopy.yaml"
    odd.write_text("name: x\nprocess: y\n", encoding="utf-8")
    with (
        pytest.raises(OverrideError, match="no top-level 'parameters' mapping"),
        param_overrides({"canopy": {"extinction_coef": 0.65}}),
    ):
        load_yaml(odd)


def test_override_errors_are_config_errors() -> None:
    """So a harness can catch the boundary's own error type."""
    assert issubclass(OverrideError, ConfigError)
    assert issubclass(UnusedOverrideError, OverrideError)


# --- 4. scoping -------------------------------------------------------------------


def test_an_empty_override_set_is_rejected() -> None:
    with pytest.raises(ValueError, match="no-op block"), param_overrides({}):
        pass


def test_an_empty_inner_mapping_is_rejected() -> None:
    with pytest.raises(ValueError, match="empty dict"), param_overrides({"canopy": {}}):
        pass


def test_nesting_REPLACES_rather_than_merges() -> None:
    """Documented behaviour, pinned: merging would make "which value is live" depend on
    lexical nesting, which an experiment must not have to reason about."""
    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        with param_overrides({"canopy": {"extinction_coef": 0.70}}):
            assert load_canopy_params().extinction_coef == 0.70
        assert load_canopy_params().extinction_coef == 0.65


def test_the_outer_block_is_restored_after_an_inner_one_raises() -> None:
    with param_overrides({"canopy": {"extinction_coef": 0.65}}):
        with (
            pytest.raises(UnusedOverrideError),
            param_overrides({"canopy": {"nonexistent_param_name": 1.0}}),
        ):
            pass
        assert load_canopy_params().extinction_coef == 0.65
