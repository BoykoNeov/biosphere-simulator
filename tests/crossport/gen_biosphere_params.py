"""Generate the biosphere param vectors the Rust port reads (Phase-7 Step 4, P7.4).

The reference is the **frozen Python loaders** (`domains.biosphere.loader`). Each of the
13 param files is loaded through its *actual* pydantic schema + unit guard + bound check
+ **core-ready fold**, then the dataclass fields are emitted as C99 hex-floats. The Rust
`domains::biosphere` crate reads this committed file via `include_str!`, so it links no
YAML parser and re-derives no fold.

**Why emit CORE-READY (post-fold) values (advisor-endorsed; the Step-3 Option-C
precedent, superseding the plan's serde-YAML sketch).** The biosphere loaders do more
than the siblings': `load_canopy_params` folds specific-leaf-area + the carbon fraction
into `sla_per_mol_c` (via pint + the molar-mass constant), and `load_nitrogen_params`
folds the N-concentration thresholds kg N/kg DM to kg N/mol C. Emitting the *core-ready*
dataclass fields (what those dataclasses carry) is faithful to "Rust reads the same
frozen files" and avoids porting pint + the fold. Decimal/derived values
round-trip bit-identically through a hex-float, so this pins the exact loader-produced
bits. `test_crossport.py::test_biosphere_params_in_sync` guards drift.

Single writer of `rust/crates/domains/src/biosphere/biosphere_params.txt`.

Regenerate with::

    uv run python tests/crossport/gen_biosphere_params.py
"""

from __future__ import annotations

from pathlib import Path

from domains.biosphere.loader import (
    load_allocation_params,
    load_canopy_params,
    load_decomposition_params,
    load_herbivory_params,
    load_microbial_respiration_params,
    load_mineralization_params,
    load_nitrogen_params,
    load_phenology_params,
    load_photosynthesis_params,
    load_respiration_params,
    load_senescence_params,
    load_transpiration_params,
    load_water_cycle_params,
)

PARAMS_PATH = (
    Path(__file__).resolve().parents[2]
    / "rust"
    / "crates"
    / "domains"
    / "src"
    / "biosphere"
    / "biosphere_params.txt"
)


def _scalars() -> list[tuple[str, float]]:
    """The flat (namespaced) scalar coefficients, each loaded through its frozen loader.

    Names are group-prefixed so the flat table has no collisions (e.g.
    `o2_half_saturation` appears in respiration / microbial / herbivory). The Rust
    reader is keyed, so ordering is only for a stable, human-diffable file.
    """
    canopy = load_canopy_params()
    photo = load_photosynthesis_params()
    resp = load_respiration_params()
    transp = load_transpiration_params()
    pheno = load_phenology_params()
    senesc = load_senescence_params()
    nitro = load_nitrogen_params()
    decomp = load_decomposition_params()
    micro = load_microbial_respiration_params()
    miner = load_mineralization_params()
    water = load_water_cycle_params()
    herb = load_herbivory_params()
    return [
        ("canopy.sla_per_mol_c", canopy.sla_per_mol_c),
        ("canopy.extinction_coef", canopy.extinction_coef),
        ("photo.vcmax", photo.vcmax),
        ("photo.jmax", photo.jmax),
        ("photo.quantum_yield", photo.quantum_yield),
        ("photo.theta", photo.theta),
        ("photo.gamma_star", photo.gamma_star),
        ("photo.kc", photo.kc),
        ("photo.ko", photo.ko),
        ("photo.o2", photo.o2),
        ("photo.t_min", photo.t_min),
        ("photo.t_opt_lo", photo.t_opt_lo),
        ("photo.t_opt_hi", photo.t_opt_hi),
        ("photo.t_max", photo.t_max),
        ("resp.maintenance_coef", resp.maintenance_coef),
        ("resp.q10", resp.q10),
        ("resp.t_ref", resp.t_ref),
        ("resp.growth_efficiency", resp.growth_efficiency),
        ("resp.o2_half_saturation", resp.o2_half_saturation),
        ("transp.aerodynamic_resistance", transp.aerodynamic_resistance),
        ("transp.surface_resistance", transp.surface_resistance),
        ("pheno.t_base", pheno.t_base),
        ("pheno.t_cap", pheno.t_cap),
        ("pheno.tsum_anthesis", pheno.tsum_anthesis),
        ("pheno.tsum_maturity", pheno.tsum_maturity),
        ("senesc.rdr_leaf", senesc.rdr_leaf),
        ("senesc.rdr_stem", senesc.rdr_stem),
        ("senesc.rdr_root", senesc.rdr_root),
        ("nitro.max_uptake_capacity", nitro.max_uptake_capacity),
        ("nitro.n_residual_per_mol_c", nitro.n_residual_per_mol_c),
        ("nitro.n_critical_per_mol_c", nitro.n_critical_per_mol_c),
        ("decomp.decomposition_rate", decomp.decomposition_rate),
        ("micro.microbial_respiration_rate", micro.microbial_respiration_rate),
        ("micro.o2_half_saturation", micro.o2_half_saturation),
        ("miner.n_senescence_rate", miner.n_senescence_rate),
        ("miner.mineralization_rate", miner.mineralization_rate),
        ("water.condensation_rate", water.condensation_rate),
        ("water.recycling_rate", water.recycling_rate),
        ("herb.grazing_rate", herb.grazing_rate),
        ("herb.respiration_rate", herb.respiration_rate),
        ("herb.mortality_rate", herb.mortality_rate),
        ("herb.o2_half_saturation", herb.o2_half_saturation),
    ]


def render() -> str:
    """The committed file's exact text (LF line endings, trailing newline)."""
    lines = [
        "# Cross-port biosphere params (Phase-7 Step 4, P7.4). GENERATED, do not edit.",
        "# Source of truth: the frozen Python loaders (domains.biosphere.loader). Each",
        "# value passes the actual pydantic schema + unit guard + bound check + the",
        "# CORE-READY fold (sla_per_mol_c, n_*_per_mol_c), then is emitted as a C99",
        "# hex-float; the Rust crate reads it via include_str! (no YAML, no pint).",
        "# The partition table (allocation) is emitted as `partition_row` lines.",
        "# Regenerate: uv run python tests/crossport/gen_biosphere_params.py",
        "#",
        "# name\thexfloat",
    ]
    for name, value in _scalars():
        lines.append(f"{name}\t{value.hex()}")
    lines.append("# partition_row\tdvs\tfl\tfs\tfr\tfo")
    for row in load_allocation_params().table:
        lines.append(
            "partition_row\t"
            f"{row.dvs.hex()}\t{row.fl.hex()}\t{row.fs.hex()}\t"
            f"{row.fr.hex()}\t{row.fo.hex()}"
        )
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    PARAMS_PATH.parent.mkdir(parents=True, exist_ok=True)
    PARAMS_PATH.write_text(render(), encoding="utf-8")
    print(f"wrote {PARAMS_PATH}")
