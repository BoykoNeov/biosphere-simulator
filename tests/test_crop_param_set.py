"""The crop param-set seam — how the tree comes to hold more than one crop at all.

Plan of record: ``docs/plans/post-roadmap-potato-crop.md``.

Until this seam existed the biosphere could hold **exactly one crop**:
``_carbon_context``/``build_plants`` called every loader argument-free, so the frozen
``params/*.yaml`` defaults *were* the crop. (The "day-neutral crop" is not a
counterexample — its own outcome says it is the same winter-wheat files with the
cold/daylength gates switched off, "not a new param file".)

This file pins the two properties the seam has to have:

1. **It is additive and default-preserving.** ``crop=None`` — the default on every
   frozen scenario — resolves to the committed reference files, path for path. The
   value proof is the goldens (``tests/regression``), which stay byte-identical; this
   file pins the *path* level, so a future edit that redirects the default fails here
   with a readable message instead of as an opaque golden diff.
2. **What a crop claims as its own is explicit and testable.** ``overridden`` /
   ``shared`` partition the ten param names, so "potato shares wheat's photosynthesis"
   is an assertion rather than a comment. That claim is load-bearing and easy to get
   wrong in the flattering direction, which is exactly why it is pinned.

Pure stdlib + the loader boundary; no goldens (the seam moves no value).
"""

from __future__ import annotations

from pathlib import Path

import pytest

from domains.biosphere import loader
from domains.biosphere import scenario as scenario_mod
from domains.biosphere.loader import (
    _CROP_PARAM_DEFAULTS,
    CROPS_DIR,
    REFERENCE_CROP,
    crop_param_set,
)
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_SCENARIO,
    DEFAULT_SCENARIO,
    PERENNIAL_CHAMBER_SCENARIO,
    SEALED_CHAMBER_SCENARIO,
)

# The ten plant-side param names a crop may override. Spelled out rather than derived
# from the module so that *adding* an eleventh is a deliberate edit here too — the set
# is the crop vocabulary, and silently growing it is what this pin exists to notice.
#
# ⚠ GREW 8 -> 9 on 2026-08-11 (root functional coupling); this pin is how that
# was
# noticed: `root_depth` was added to the vocabulary in the loader and the suite went
# red
# here, exactly as the sentence above promised it would. Recorded rather than quietly
# re-listed. Rooted depth is plant-side (it is a crop property — [E] Table 25 tabulates
# it per species, and potato's row differs from wheat's in both values), so it belongs
# in
# the vocabulary rather than alongside the soil/scenario data.
#
# ⚠ GREW 9 -> 10 on 2026-08-12 (stem reserves), and the pin did its job a second time.
# `stem_reserves` is plant-side for the same reason: [E] Table 7 tabulates the
# remobilizable fraction PER SPECIES, and the rows genuinely differ (wheat 0.4, barley
# 0.3, sugar-cane 0.5). ⚠ Note what the fallback means here and why the mechanism ALSO
# has a scenario flag: a crop with no `stem_reserves.yaml` falls back to WHEAT's file,
# so a value-only switch would hand every second species wheat's 0.40 in silence. The
# boolean `SeasonScenario.stem_reserves` is what actually decides whether the crop has
# the mechanism, and potato turns it off — [E] gives potato a RANGE (0.2-0.4), not a
# point.
EXPECTED_PARAM_NAMES = frozenset(
    {
        "allocation",
        "canopy",
        "nitrogen",
        "phenology",
        "photosynthesis",
        "respiration",
        "root_depth",
        "senescence",
        "stem_reserves",
        "transpiration",
    }
)


def test_crop_vocabulary_is_the_ten_plant_side_files() -> None:
    assert set(_CROP_PARAM_DEFAULTS) == EXPECTED_PARAM_NAMES


def test_none_resolves_to_the_frozen_reference_files() -> None:
    # The additivity property at the path level: an unmodified scenario reads exactly
    # the files it read before this seam existed.
    resolved = crop_param_set(None)
    assert resolved is REFERENCE_CROP
    assert resolved.name == "winter_wheat"
    assert resolved.overridden == ()
    assert set(resolved.shared) == EXPECTED_PARAM_NAMES
    assert resolved.paths == {
        "allocation": loader.ALLOCATION_PARAMS_PATH,
        "canopy": loader.CANOPY_PARAMS_PATH,
        "nitrogen": loader.NITROGEN_PARAMS_PATH,
        "phenology": loader.PHENOLOGY_PARAMS_PATH,
        "photosynthesis": loader.PHOTOSYNTHESIS_PARAMS_PATH,
        "respiration": loader.RESPIRATION_PARAMS_PATH,
        "root_depth": loader.ROOT_DEPTH_PARAMS_PATH,
        "senescence": loader.SENESCENCE_PARAMS_PATH,
        "stem_reserves": loader.STEM_RESERVE_PARAMS_PATH,
        "transpiration": loader.TRANSPIRATION_PARAMS_PATH,
    }
    # Every reference file is a real committed file directly under ``params/`` — i.e.
    # the frozen surface the manifest names, not something under ``params/crops/``.
    for path in resolved.paths.values():
        assert path.is_file()
        assert path.parent == loader.PHENOLOGY_PARAMS_PATH.parent


def test_default_scenario_and_the_frozen_chambers_carry_no_crop() -> None:
    # The frozen roster must stay on the reference crop, or the goldens are describing
    # a different plant than the freeze contract says they do.
    assert scenario_mod.SeasonScenario().crop is None
    for frozen in (
        DEFAULT_SCENARIO,
        SEALED_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_SCENARIO,
        CONSUMER_CHAMBER_SCENARIO,
    ):
        assert frozen.crop is None


def test_overridden_and_shared_partition_the_vocabulary(tmp_path: Path) -> None:
    # A crop's claim about its own science is a partition, not two loose lists: nothing
    # may be both, and nothing may be neither.
    crop_dir = tmp_path / "crops" / "fictional"
    crop_dir.mkdir(parents=True)
    (crop_dir / "phenology.yaml").write_text("", encoding="utf-8")
    (crop_dir / "allocation.yaml").write_text("", encoding="utf-8")

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(loader, "CROPS_DIR", tmp_path / "crops")
        resolved = crop_param_set("fictional")

    assert set(resolved.overridden) == {"allocation", "phenology"}
    assert set(resolved.shared) == EXPECTED_PARAM_NAMES - {"allocation", "phenology"}
    assert not set(resolved.overridden) & set(resolved.shared)
    assert set(resolved.overridden) | set(resolved.shared) == EXPECTED_PARAM_NAMES
    # Overridden names point INTO the crop dir; shared names point at the frozen files.
    assert resolved.paths["phenology"] == crop_dir / "phenology.yaml"
    assert resolved.paths["canopy"] == loader.CANOPY_PARAMS_PATH


def test_unknown_crop_name_raises_and_lists_what_exists() -> None:
    with pytest.raises(ValueError, match="unknown crop 'no_such_crop'"):
        crop_param_set("no_such_crop")


def test_a_typo_in_a_param_filename_is_rejected_not_ignored(tmp_path: Path) -> None:
    # The failure mode this check exists for: ``phenolgy.yaml`` would otherwise be
    # silently skipped and the crop would quietly run on the reference file, which is
    # the *flattering* failure (it still runs, it is just not the crop you authored).
    crop_dir = tmp_path / "crops" / "typo"
    crop_dir.mkdir(parents=True)
    (crop_dir / "phenolgy.yaml").write_text("", encoding="utf-8")

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(loader, "CROPS_DIR", tmp_path / "crops")
        with pytest.raises(ValueError, match="unknown param file"):
            crop_param_set("typo")


def test_a_crop_that_overrides_nothing_is_rejected(tmp_path: Path) -> None:
    crop_dir = tmp_path / "crops" / "empty"
    crop_dir.mkdir(parents=True)

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr(loader, "CROPS_DIR", tmp_path / "crops")
        with pytest.raises(ValueError, match="overrides no param file"):
            crop_param_set("empty")


def test_crops_dir_is_outside_the_frozen_param_glob() -> None:
    # The freeze gate (``test_freeze_manifest.py``) globs ``params/*.yaml``
    # NON-recursively. This pin states the consequence the layout DEPENDS on, so that
    # if the gate is ever made recursive this fails loudly here — beside the reasoning
    # — instead of surfacing as a mystery manifest failure. The decision itself (why a
    # crop is deliberately not part of the frozen surface) is in the plan doc and in
    # ``loader.py``'s section header.
    frozen_dir = loader.PHENOLOGY_PARAMS_PATH.parent
    assert CROPS_DIR.parent == frozen_dir
    assert CROPS_DIR not in set(frozen_dir.glob("*.yaml"))
    if CROPS_DIR.is_dir():
        for crop_file in CROPS_DIR.rglob("*.yaml"):
            assert crop_file not in set(frozen_dir.glob("*.yaml"))
