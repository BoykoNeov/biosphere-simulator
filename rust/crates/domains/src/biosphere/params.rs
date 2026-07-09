//! The biosphere coefficients, read from the generated hex-float file (Phase-7 P7.4).
//!
//! [`BIOSPHERE_PARAMS`] is `tests/crossport/gen_biosphere_params.py`'s output — each of
//! the 13 frozen param files loaded through its Python loader (pydantic schema + unit
//! guard + bound check + **core-ready fold**: `sla_per_mol_c`, `n_*_per_mol_c`) and
//! emitted as a C99 hex-float (scalars) or a `partition_row` line (the allocation table).
//! We `include_str!` it and parse with the [`simcore::hexfloat`] codec, so the crate
//! links no YAML parser and re-derives no fold.
//!
//! The struct groups mirror the Python dataclasses (`CanopyParams`, `PhotosynthesisParams`,
//! …). `test_crossport.py::test_biosphere_params_in_sync` guards the file against drift.

use std::collections::BTreeMap;

use simcore::hexfloat;

/// The committed, generated param table (see `gen_biosphere_params.py`).
const BIOSPHERE_PARAMS: &str = include_str!("biosphere_params.txt");

/// Beer–Lambert canopy params (core-ready — `sla_per_mol_c` is pre-folded).
#[derive(Debug, Clone, Copy)]
pub struct CanopyParams {
    pub sla_per_mol_c: f64,
    pub extinction_coef: f64,
}

/// FvCB photosynthesis params (reference temperature).
#[derive(Debug, Clone, Copy)]
pub struct PhotosynthesisParams {
    pub vcmax: f64,
    pub jmax: f64,
    pub quantum_yield: f64,
    pub theta: f64,
    pub gamma_star: f64,
    pub kc: f64,
    pub ko: f64,
    pub o2: f64,
    pub t_min: f64,
    pub t_opt_lo: f64,
    pub t_opt_hi: f64,
    pub t_max: f64,
}

/// Maintenance + growth respiration params.
#[derive(Debug, Clone, Copy)]
pub struct RespirationParams {
    pub maintenance_coef: f64,
    pub q10: f64,
    pub t_ref: f64,
    pub growth_efficiency: f64,
    pub o2_half_saturation: f64,
}

/// Penman–Monteith transpiration params.
#[derive(Debug, Clone, Copy)]
pub struct TranspirationParams {
    pub aerodynamic_resistance: f64,
    pub surface_resistance: f64,
}

/// Thermal-time phenology params.
#[derive(Debug, Clone, Copy)]
pub struct PhenologyParams {
    pub t_base: f64,
    pub t_cap: f64,
    pub tsum_anthesis: f64,
    pub tsum_maturity: f64,
}

/// Per-organ relative senescence (death) rates.
#[derive(Debug, Clone, Copy)]
pub struct SenescenceParams {
    pub rdr_leaf: f64,
    pub rdr_stem: f64,
    pub rdr_root: f64,
}

/// Nitrogen uptake + limitation params (core-ready — thresholds pre-folded to kg N/mol C).
#[derive(Debug, Clone, Copy)]
pub struct NitrogenParams {
    pub max_uptake_capacity: f64,
    pub n_residual_per_mol_c: f64,
    pub n_critical_per_mol_c: f64,
}

/// First-order litter-decay param.
#[derive(Debug, Clone, Copy)]
pub struct DecompositionParams {
    pub decomposition_rate: f64,
}

/// First-order microbial-respiration params.
#[derive(Debug, Clone, Copy)]
pub struct MicrobialRespirationParams {
    pub microbial_respiration_rate: f64,
    pub o2_half_saturation: f64,
}

/// Nitrogen-return-loop params (N-senescence shedding + net mineralization).
#[derive(Debug, Clone, Copy)]
pub struct MineralizationParams {
    pub n_senescence_rate: f64,
    pub mineralization_rate: f64,
}

/// Water-cycle params (condensation + recycling).
#[derive(Debug, Clone, Copy)]
pub struct WaterCycleParams {
    pub condensation_rate: f64,
    pub recycling_rate: f64,
}

/// Minimal-consumer params (grazing + respiration + mortality + f_O2 Monod).
#[derive(Debug, Clone, Copy)]
pub struct HerbivoryParams {
    pub grazing_rate: f64,
    pub respiration_rate: f64,
    pub mortality_rate: f64,
    pub o2_half_saturation: f64,
}

/// One DVS knot of the leaf/stem/root/storage partition table.
#[derive(Debug, Clone, Copy)]
pub struct PartitionRow {
    pub dvs: f64,
    pub fl: f64,
    pub fs: f64,
    pub fr: f64,
    pub fo: f64,
}

/// DVS-keyed partition table.
#[derive(Debug, Clone)]
pub struct AllocationParams {
    pub table: Vec<PartitionRow>,
}

/// All frozen biosphere coefficients, parsed once from the generated file.
#[derive(Debug, Clone)]
pub struct BiosphereParams {
    pub canopy: CanopyParams,
    pub photo: PhotosynthesisParams,
    pub resp: RespirationParams,
    pub transp: TranspirationParams,
    pub pheno: PhenologyParams,
    pub senesc: SenescenceParams,
    pub nitro: NitrogenParams,
    pub decomp: DecompositionParams,
    pub micro: MicrobialRespirationParams,
    pub miner: MineralizationParams,
    pub water: WaterCycleParams,
    pub herb: HerbivoryParams,
    pub alloc: AllocationParams,
}

/// Parse the embedded file into a `name → value` scalar table + the partition rows
/// (comment/blank lines skipped).
fn parse() -> (BTreeMap<&'static str, f64>, Vec<PartitionRow>) {
    let mut scalars: BTreeMap<&'static str, f64> = BTreeMap::new();
    let mut rows: Vec<PartitionRow> = Vec::new();
    for line in BIOSPHERE_PARAMS.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let head = fields.next().expect("biosphere param line has a leading field");
        if head == "partition_row" {
            let mut next = || {
                hexfloat::parse(fields.next().expect("partition_row field"))
                    .expect("partition_row hex-float parses")
            };
            rows.push(PartitionRow {
                dvs: next(),
                fl: next(),
                fs: next(),
                fr: next(),
                fo: next(),
            });
        } else {
            let hex = fields.next().expect("scalar param has a hex-float value");
            let value = hexfloat::parse(hex).expect("scalar param hex-float parses");
            scalars.insert(head, value);
        }
    }
    (scalars, rows)
}

fn get(t: &BTreeMap<&'static str, f64>, key: &str) -> f64 {
    *t.get(key)
        .unwrap_or_else(|| panic!("missing biosphere param {key:?} in biosphere_params.txt"))
}

/// Load the frozen biosphere coefficients from the embedded generated file.
pub fn biosphere() -> BiosphereParams {
    let (t, rows) = parse();
    BiosphereParams {
        canopy: CanopyParams {
            sla_per_mol_c: get(&t, "canopy.sla_per_mol_c"),
            extinction_coef: get(&t, "canopy.extinction_coef"),
        },
        photo: PhotosynthesisParams {
            vcmax: get(&t, "photo.vcmax"),
            jmax: get(&t, "photo.jmax"),
            quantum_yield: get(&t, "photo.quantum_yield"),
            theta: get(&t, "photo.theta"),
            gamma_star: get(&t, "photo.gamma_star"),
            kc: get(&t, "photo.kc"),
            ko: get(&t, "photo.ko"),
            o2: get(&t, "photo.o2"),
            t_min: get(&t, "photo.t_min"),
            t_opt_lo: get(&t, "photo.t_opt_lo"),
            t_opt_hi: get(&t, "photo.t_opt_hi"),
            t_max: get(&t, "photo.t_max"),
        },
        resp: RespirationParams {
            maintenance_coef: get(&t, "resp.maintenance_coef"),
            q10: get(&t, "resp.q10"),
            t_ref: get(&t, "resp.t_ref"),
            growth_efficiency: get(&t, "resp.growth_efficiency"),
            o2_half_saturation: get(&t, "resp.o2_half_saturation"),
        },
        transp: TranspirationParams {
            aerodynamic_resistance: get(&t, "transp.aerodynamic_resistance"),
            surface_resistance: get(&t, "transp.surface_resistance"),
        },
        pheno: PhenologyParams {
            t_base: get(&t, "pheno.t_base"),
            t_cap: get(&t, "pheno.t_cap"),
            tsum_anthesis: get(&t, "pheno.tsum_anthesis"),
            tsum_maturity: get(&t, "pheno.tsum_maturity"),
        },
        senesc: SenescenceParams {
            rdr_leaf: get(&t, "senesc.rdr_leaf"),
            rdr_stem: get(&t, "senesc.rdr_stem"),
            rdr_root: get(&t, "senesc.rdr_root"),
        },
        nitro: NitrogenParams {
            max_uptake_capacity: get(&t, "nitro.max_uptake_capacity"),
            n_residual_per_mol_c: get(&t, "nitro.n_residual_per_mol_c"),
            n_critical_per_mol_c: get(&t, "nitro.n_critical_per_mol_c"),
        },
        decomp: DecompositionParams {
            decomposition_rate: get(&t, "decomp.decomposition_rate"),
        },
        micro: MicrobialRespirationParams {
            microbial_respiration_rate: get(&t, "micro.microbial_respiration_rate"),
            o2_half_saturation: get(&t, "micro.o2_half_saturation"),
        },
        miner: MineralizationParams {
            n_senescence_rate: get(&t, "miner.n_senescence_rate"),
            mineralization_rate: get(&t, "miner.mineralization_rate"),
        },
        water: WaterCycleParams {
            condensation_rate: get(&t, "water.condensation_rate"),
            recycling_rate: get(&t, "water.recycling_rate"),
        },
        herb: HerbivoryParams {
            grazing_rate: get(&t, "herb.grazing_rate"),
            respiration_rate: get(&t, "herb.respiration_rate"),
            mortality_rate: get(&t, "herb.mortality_rate"),
            o2_half_saturation: get(&t, "herb.o2_half_saturation"),
        },
        alloc: AllocationParams { table: rows },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_groups_and_the_partition_table() {
        let p = biosphere();
        assert!(p.canopy.sla_per_mol_c > 0.0);
        assert!(p.photo.t_min < p.photo.t_opt_lo);
        assert_eq!(p.alloc.table.len(), 3);
        // Each partition row sums to 1 (the loader-enforced invariant, round-trip check).
        for row in &p.alloc.table {
            let total = row.fl + row.fs + row.fr + row.fo;
            assert!((total - 1.0).abs() < 1e-9, "row sums to {total}");
        }
    }
}
