//! **Tissue shedding paced by development** — the lab form of `Senescence` (and its nitrogen
//! twin) that the leaf-shedding design note measures (`docs/plans/post-roadmap-leaf-shedding.md`
//! §7). LAB-ONLY: nothing in a canonical build constructs these flows; they reach a run only
//! through [`composition`] and the mechanism seam.
//!
//! The frozen `Senescence` removes a fixed share of each organ per day (`rdr_leaf` 0.02,
//! `rdr_root` 0.01, `rdr_stem` 0.005, all `TODO(cite)`), whatever the temperature or the crop's
//! stage. Three books on the shelf agree there is no age-related leaf death before flowering;
//! this module is that form, one organ at a time:
//!
//! * **Leaf** ([`LeafShedding::Development`]) — Teh, Eqn 7.17, after Goudriaan & van Laar (1994):
//!   0 while `DVS < 1`, then `DVR / max(0.1, 2 − DVS)` with `DVR = daily_thermal_time(T) /
//!   tsum_maturity`. Integrated, the leaf falls linearly in development toward maturity —
//!   Soltani & Sinclair (2012) Eqn 9.7's shape. The mutual-shading term is added unchanged.
//!   ⚠ `DVR` omits the drought hastening factor (1 whenever the crop is unstressed).
//! * **Leaf, the second cited reading** ([`LeafShedding::PenningDeVries`]) — Penning de Vries et
//!   al. (1989) Listing 5, p. 212 (rice IR36, the crop file §3.2.6 cites): `LLVT = 0.0,0.0
//!   1.0,0.0 1.3,0.007 1.8,0.012 2.5,0.012`, linear between the points (an AFGEN table), per day.
//!   ⚠ LOCUS: rice. Milder than Teh's after anthesis: leaves remain at maturity.
//! * **Root** ([`RootShedding::AfterAnthesis`]) — Penning de Vries et al. (1989): 0 while
//!   `DVS < 1`, the frozen `rdr_root` after (the source's post-anthesis 0.010–0.011, rice).
//! * **Stem** ([`StemShedding::None`]) — Penning de Vries et al. (1989) p. 95: "except for their
//!   reserves, stems do not lose weight".
//!
//! Every organ also has a `Flat` setting that is the frozen rate, computed with the frozen
//! expression in the frozen order, so [`SheddingForm::FLAT`] reproduces the frozen run bit for
//! bit — the control (`tests/shedding_form.rs`).

use crate::biosphere::params::BiosphereParams;
use crate::biosphere::science;
use crate::biosphere::stocks::{
    chamber_wiring, LEAF_C, LITTER_N, PLANT_N, ROOT_C, STEM_C, TEMP_VAR, THERMAL_TIME,
};
use crate::biosphere::system::SeasonScenario;
use crate::lab::mechanism::{Composition, FlowFactory};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::state::State;

/// The frozen carbon flow's id (`build_season_with`).
pub const SENESCENCE_ID: &str = "biosphere.senescence";
/// The frozen nitrogen twin's id (sealed scenarios only).
pub const NITROGEN_SENESCENCE_ID: &str = "biosphere.nitrogen_senescence";

/// How leaves are shed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafShedding {
    /// The frozen flat `rdr_leaf` (plus shading).
    Flat,
    /// 0 before anthesis, `DVR / max(0.1, 2 − DVS)` after (plus shading).
    Development,
    /// Penning de Vries et al. (1989) Listing 5's `LLVT` table on DVS (plus shading).
    PenningDeVries,
}

/// How roots are shed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootShedding {
    /// The frozen flat `rdr_root`.
    Flat,
    /// 0 before anthesis, `rdr_root` after.
    AfterAnthesis,
}

/// How stems are shed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemShedding {
    /// The frozen flat `rdr_stem`.
    Flat,
    /// No structural stem death.
    None,
}

/// One setting per organ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheddingForm {
    pub leaf: LeafShedding,
    pub root: RootShedding,
    pub stem: StemShedding,
}

impl SheddingForm {
    /// The frozen rates through the lab flows — the control.
    pub const FLAT: SheddingForm = SheddingForm {
        leaf: LeafShedding::Flat,
        root: RootShedding::Flat,
        stem: StemShedding::Flat,
    };
    /// The sources' leaf form alone.
    pub const LEAF: SheddingForm = SheddingForm {
        leaf: LeafShedding::Development,
        ..SheddingForm::FLAT
    };
    /// Leaf and root.
    pub const LEAF_ROOT: SheddingForm = SheddingForm {
        root: RootShedding::AfterAnthesis,
        ..SheddingForm::LEAF
    };
    /// Leaf, root and stem — the sources' form for all three organs.
    pub const ALL: SheddingForm = SheddingForm {
        stem: StemShedding::None,
        ..SheddingForm::LEAF_ROOT
    };
    /// [`SheddingForm::ALL`] with Penning de Vries' leaf table after anthesis instead of Teh's.
    pub const ALL_PDV: SheddingForm = SheddingForm {
        leaf: LeafShedding::PenningDeVries,
        ..SheddingForm::ALL
    };
}

/// Penning de Vries et al. (1989) Listing 5 `LLVT` (rice IR36): `(DVS, 1/day)`.
pub const LLVT: [(f64, f64); 5] = [
    (0.0, 0.0),
    (1.0, 0.0),
    (1.3, 0.007),
    (1.8, 0.012),
    (2.5, 0.012),
];

/// Linear interpolation in a `(x, y)` table, clamped at both ends (an AFGEN).
pub fn afgen(table: &[(f64, f64)], x: f64) -> f64 {
    if x <= table[0].0 {
        return table[0].1;
    }
    for w in table.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x <= x1 {
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0);
        }
    }
    table[table.len() - 1].1
}

/// The rate law both flows share, so carbon and nitrogen cannot shed at different rates.
#[derive(Debug, Clone, Copy)]
pub struct SheddingLaw {
    pub form: SheddingForm,
    pub rdr_leaf: f64,
    pub rdr_stem: f64,
    pub rdr_root: f64,
    pub shade_rate: f64,
    pub lai_threshold: f64,
    pub sla_per_mol_c: f64,
    pub ground_area: f64,
    pub t_base: f64,
    pub t_cap: f64,
    pub tsum_anthesis: f64,
    pub tsum_maturity: f64,
}

impl SheddingLaw {
    /// The law for `scenario` at params `p`.
    pub fn new(form: SheddingForm, scenario: &SeasonScenario, p: &BiosphereParams) -> SheddingLaw {
        SheddingLaw {
            form,
            rdr_leaf: p.senesc.rdr_leaf,
            rdr_stem: p.senesc.rdr_stem,
            rdr_root: p.senesc.rdr_root,
            shade_rate: p.senesc.shade_rate,
            lai_threshold: p.senesc.lai_threshold,
            sla_per_mol_c: p.canopy.sla_per_mol_c,
            ground_area: scenario.ground_area,
            t_base: p.pheno.t_base,
            t_cap: p.pheno.t_cap,
            tsum_anthesis: p.pheno.tsum_anthesis,
            tsum_maturity: p.pheno.tsum_maturity,
        }
    }

    /// `(leaf, stem, root)` relative death rates (1/day) on this snapshot.
    ///
    /// ⚠ The `Flat` arms are the frozen expressions in the frozen order — `mutual_shading_rate`
    /// on the flat `rdr_leaf`, the two flat constants — so [`SheddingForm::FLAT`] is bit-identical
    /// to the frozen flows. Temperature is read only when an organ needs the clock.
    pub fn rates(
        &self,
        snapshot: &State,
        env: &dyn Environment,
    ) -> Result<(f64, f64, f64), SimError> {
        let leaf_c = amount(snapshot, LEAF_C);
        let lai = leaf_c * self.sla_per_mol_c / self.ground_area;
        let tt = snapshot.aux.get(THERMAL_TIME).copied().unwrap_or(0.0);
        let dvs = science::development_stage(tt, self.tsum_anthesis, self.tsum_maturity);
        let leaf_age = match self.form.leaf {
            LeafShedding::Flat => self.rdr_leaf,
            LeafShedding::Development if dvs < 1.0 => 0.0,
            LeafShedding::Development => {
                let dvr = science::daily_thermal_time(env.get(TEMP_VAR)?, self.t_base, self.t_cap)
                    / self.tsum_maturity;
                dvr / (2.0 - dvs).max(0.1)
            }
            LeafShedding::PenningDeVries => afgen(&LLVT, dvs),
        };
        let leaf = science::mutual_shading_rate(lai, leaf_age, self.shade_rate, self.lai_threshold);
        let stem = match self.form.stem {
            StemShedding::Flat => self.rdr_stem,
            StemShedding::None => 0.0,
        };
        let root = match self.form.root {
            RootShedding::Flat => self.rdr_root,
            RootShedding::AfterAnthesis if dvs < 1.0 => 0.0,
            RootShedding::AfterAnthesis => self.rdr_root,
        };
        Ok((leaf, stem, root))
    }
}

fn amount(snapshot: &State, id: &str) -> f64 {
    snapshot.stocks.get(id).map(|s| s.amount).unwrap_or(0.0)
}

fn leg(stock: &str, amount: f64) -> Result<Leg, SimError> {
    Leg::new(stock.to_string(), amount)
}

/// CARBON `{leaf, stem, root} -> litter`, on [`SheddingLaw`]. Its own type name, never the
/// frozen `"Senescence"` (`tests/lab_only_mechanisms.rs`: a lab copy must not wear a frozen name).
pub struct LabSenescence {
    pub id: String,
    pub litter_sink: String,
    pub law: SheddingLaw,
}

impl Flow for LabSenescence {
    fn type_name(&self) -> &'static str {
        "LabSenescence"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let (r_leaf, r_stem, r_root) = self.law.rates(snapshot, env)?;
        let leaf = r_leaf * amount(snapshot, LEAF_C) * dt;
        let stem = r_stem * amount(snapshot, STEM_C) * dt;
        let root = r_root * amount(snapshot, ROOT_C) * dt;
        FlowResult::new(vec![
            leg(LEAF_C, -leaf)?,
            leg(STEM_C, -stem)?,
            leg(ROOT_C, -root)?,
            leg(&self.litter_sink, leaf + stem + root)?,
        ])
    }
}

/// NITROGEN `plant_n -> litter_n`, driven by the same carbon shedding (the frozen
/// `NitrogenSenescence`'s expression, on [`SheddingLaw`]).
pub struct LabNitrogenSenescence {
    pub id: String,
    pub n_residual_per_mol_c: f64,
    pub law: SheddingLaw,
}

impl Flow for LabNitrogenSenescence {
    fn type_name(&self) -> &'static str {
        "LabNitrogenSenescence"
    }
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let leaf = amount(snapshot, LEAF_C);
        let stem = amount(snapshot, STEM_C);
        let root = amount(snapshot, ROOT_C);
        let (r_leaf, r_stem, r_root) = self.law.rates(snapshot, env)?;
        let shed_carbon = r_leaf * leaf + r_stem * stem + r_root * root;
        let plant_n = amount(snapshot, PLANT_N);
        let biomass_c = leaf + stem + root;
        let shed = if shed_carbon <= 0.0 || plant_n <= 0.0 || biomass_c <= 0.0 {
            0.0
        } else {
            (plant_n / biomass_c).min(self.n_residual_per_mol_c) * shed_carbon * dt
        };
        FlowResult::new(vec![leg(PLANT_N, -shed)?, leg(LITTER_N, shed)?])
    }
}

/// The carbon flow `form` builds for `scenario`.
pub fn carbon_flow(
    form: SheddingForm,
    scenario: &SeasonScenario,
    p: &BiosphereParams,
) -> Box<dyn Flow> {
    Box::new(LabSenescence {
        id: SENESCENCE_ID.to_string(),
        litter_sink: chamber_wiring(scenario.sealed).litter_carbon_target,
        law: SheddingLaw::new(form, scenario, p),
    })
}

/// The nitrogen flow `form` builds for `scenario`.
pub fn nitrogen_flow(
    form: SheddingForm,
    scenario: &SeasonScenario,
    p: &BiosphereParams,
) -> Box<dyn Flow> {
    Box::new(LabNitrogenSenescence {
        id: NITROGEN_SENESCENCE_ID.to_string(),
        n_residual_per_mol_c: p.nitro.n_residual_per_mol_c,
        law: SheddingLaw::new(form, scenario, p),
    })
}

/// The composition for a scenario WITH the nitrogen twin (the sealed ones): both flows replaced.
/// A scenario without it reports n/a under this composition — use [`carbon_only`] there.
pub fn composition(form: SheddingForm) -> Composition {
    Composition {
        replacements: vec![
            (SENESCENCE_ID.to_string(), carbon_factory(form)),
            (NITROGEN_SENESCENCE_ID.to_string(), nitrogen_factory(form)),
        ],
        ..Composition::default()
    }
}

/// The composition for a scenario WITHOUT the nitrogen twin (the open field): carbon only.
/// ⚠ Applied to a sealed scenario it would shed carbon on the new law and nitrogen on the old;
/// a caller merges its cells only where [`composition`] is n/a.
pub fn carbon_only(form: SheddingForm) -> Composition {
    Composition::replacing(SENESCENCE_ID, carbon_factory(form))
}

fn carbon_factory(form: SheddingForm) -> FlowFactory {
    Box::new(move |s: &SeasonScenario, p: &BiosphereParams| carbon_flow(form, s, p))
}

fn nitrogen_factory(form: SheddingForm) -> FlowFactory {
    Box::new(move |s: &SeasonScenario, p: &BiosphereParams| nitrogen_flow(form, s, p))
}
