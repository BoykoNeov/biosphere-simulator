//! The **value-switch lab** — "source A vs source B, what changes?" as a call.
//!
//! Plan: `docs/plans/post-roadmap-value-switch-harness.md`. This module is the
//! substitution half; it turns `canopy.extinction_coef = 0.65` into a
//! [`BiosphereParams`](crate::biosphere::params::BiosphereParams) that
//! [`build_season_with`](crate::biosphere::system::build_season_with) will run.
//!
//! # Why this is not in `src/biosphere/`
//!
//! It reaches the [`config`] boundary (to rewrite a param file's text), and
//! `tests/biosphere_spine_purity.rs` allows exactly two spine modules to do that:
//! `params.rs` and `weather.rs`. A lab tool is not a third boundary — it is a *consumer* of
//! the one that exists, so it lives beside the spine rather than inside it.
//!
//! # What makes an experiment cheap here, and what stops it becoming a commitment
//!
//! The frozen YAML text is `include_str!`-ed in; a substitution rewrites a copy **in
//! memory** and hands it to the ordinary loader. So:
//!
//! * no file is touched, so no per-file digest in any manifest can move;
//! * the substituted value passes the *same* schema, exact-string unit guard, frozen bounds
//!   and boundary folds as a committed one.
//!
//! ⚠ **That second property has a consequence worth stating rather than discovering:** a
//! substitution outside a frozen bound **panics**, exactly as a committed one would. That is
//! the guard doing its job, not a limitation to route around — a value the bound rejects is
//! a request to change the contract, which is an unfreeze, not an experiment.
//!
//! # ⚠ This module takes no decision and endorses no value
//!
//! It regenerates evidence. The `extinction_coef` question it was built for
//! (`docs/log/canopy-provenance.md`) is still open and still the user's.

use crate::biosphere::params::{self, BiosphereParams};
use config::{with_override, ConfigError, ParamFile};

/// The comparison report — §6 of the plan, every requirement earned by a wrong read.
pub mod report;

/// The **science** half: a season with a named flow removed from the assembled registry.
pub mod mechanism;

/// One substitution: a field of one frozen param file, and the value to run instead.
#[derive(Debug, Clone, PartialEq)]
pub struct Substitution {
    /// The param file's basename, e.g. `"canopy.yaml"` — one of [`params::param_files`].
    pub file: String,
    /// The `parameters` key, e.g. `"extinction_coef"`.
    pub field: String,
    /// The value to run instead of the frozen one.
    pub value: f64,
}

impl Substitution {
    /// A substitution addressed by `file` and `field`.
    pub fn new(file: &str, field: &str, value: f64) -> Substitution {
        Substitution {
            file: file.to_string(),
            field: field.to_string(),
            value,
        }
    }

    /// A substitution addressed by **field alone**, resolving which file owns it.
    ///
    /// ⚠ **Ambiguity is an error, not a first match**, and the hazard is live rather than
    /// theoretical: `carbon_fraction` is a key of *both* `canopy.yaml` and `nitrogen.yaml`,
    /// where the two files carry the same number under a documented must-equal constraint
    /// but fold it differently. Silently picking one would produce an A/B table that is
    /// wrong in a way no reader could see. [`tests::the_shared_key_is_refused_by_name`]
    /// pins it.
    pub fn resolve(field: &str, value: f64) -> Result<Substitution, ConfigError> {
        let owners = owners_of(field)?;
        match owners.len() {
            1 => Ok(Substitution::new(owners[0], field, value)),
            0 => Err(ConfigError::new(format!(
                "no frozen biosphere param file has a field {field:?}"
            ))),
            _ => Err(ConfigError::new(format!(
                "{field:?} is a field of {owners:?} — address it as file + field, not by \
                 name alone"
            ))),
        }
    }
}

/// Which frozen files declare `field`, in census order.
pub fn owners_of(field: &str) -> Result<Vec<&'static str>, ConfigError> {
    let mut owners = Vec::new();
    for (name, text) in params::param_files() {
        if ParamFile::parse(text, name)?.fields().contains(&field) {
            owners.push(name);
        }
    }
    Ok(owners)
}

/// The frozen params with `subs` applied — the object a substituted run is built from.
///
/// # ⚠ A substitution that misses is an error, never a quiet baseline
///
/// The plan's §7 names the failure this whole module is prone to: patch the wrong symbol,
/// get a clean run, read the baseline back, and report "no effect" as a finding. Every route
/// to that is closed here rather than made unlikely:
///
/// * an unknown file is rejected against the census;
/// * an unknown or table-shaped field is rejected by [`with_override`], which also requires
///   that **exactly one** line changed and re-reads the value **bit for bit**;
/// * two substitutions of the same field are rejected, because the second would silently win.
///
/// What this function cannot check is whether the *run* reads the field it moved — that is
/// `tests/param_funnel.rs`'s subject, and it is a property of the tree rather than of a call.
pub fn biosphere_with(subs: &[Substitution]) -> Result<BiosphereParams, ConfigError> {
    for (i, s) in subs.iter().enumerate() {
        if subs[..i].iter().any(|p| p.file == s.file && p.field == s.field) {
            return Err(ConfigError::new(format!(
                "{}:{} is substituted twice — the second would silently win",
                s.file, s.field
            )));
        }
    }
    let census = params::param_files();
    for s in subs {
        if !census.iter().any(|(name, _)| *name == s.file) {
            let names: Vec<&str> = census.iter().map(|(n, _)| *n).collect();
            return Err(ConfigError::new(format!(
                "{:?} is not a frozen biosphere param file (have {names:?})",
                s.file
            )));
        }
    }

    // One resolved text per file: the frozen bytes, or the frozen bytes with this file's
    // substitutions applied in order.
    let mut resolved: Vec<(&'static str, String)> = Vec::with_capacity(census.len());
    for (name, text) in census {
        let mut current = text.to_string();
        for s in subs.iter().filter(|s| s.file == name) {
            current = with_override(&current, &s.field, s.value, name)?;
        }
        resolved.push((name, current));
    }
    let text = |file: &str| -> (&'static str, &str) {
        let (name, t) = resolved
            .iter()
            .find(|(n, _)| *n == file)
            .expect("every census file is resolved");
        (name, t.as_str())
    };

    let (n, t) = text("phenology.yaml");
    let pheno = params::phenology_from(t, n);
    let vern = params::vernalization_from(t, n);
    let photoperiod = params::photoperiod_from(t, n);

    let (cn, ct) = text("canopy.yaml");
    let (pn, pt) = text("photosynthesis.yaml");
    let (rn, rt) = text("respiration.yaml");
    let (tn, tt) = text("transpiration.yaml");
    let (sn, st) = text("senescence.yaml");
    let (srn, srt) = text("stem_reserves.yaml");
    let (rdn, rdt) = text("root_depth.yaml");
    let (nn, nt) = text("nitrogen.yaml");
    let (dn, dt) = text("decomposition.yaml");
    let (mn, mt) = text("microbial_respiration.yaml");
    let (hn, ht) = text("humification.yaml");
    let (wn, wt) = text("water_cycle.yaml");
    let (hbn, hbt) = text("herbivory.yaml");
    let (an, at) = text("allocation.yaml");

    Ok(BiosphereParams {
        canopy: params::canopy_from(ct, cn),
        photo: params::photosynthesis_from(pt, pn),
        resp: params::respiration_from(rt, rn),
        transp: params::transpiration_from(tt, tn),
        pheno,
        vern,
        photoperiod,
        senesc: params::senescence_from(st, sn),
        stem_reserve: params::stem_reserves_from(srt, srn),
        rootd: params::root_depth_from(rdt, rdn),
        nitro: params::nitrogen_from(nt, nn),
        decomp: params::decomposition_from(dt, dn),
        micro: params::microbial_respiration_from(mt, mn),
        humi: params::humification_from(ht, hn),
        water: params::water_cycle_from(wt, wn),
        herb: params::herbivory_from(hbt, hbn),
        alloc: params::allocation_from(at, an),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠⚠ **The control that makes every other test here mean something.** With no
    /// substitutions, this module must reproduce [`params::biosphere`] exactly — same
    /// texts, same loaders, same folds. If it did not, an A/B table's "baseline" column
    /// would be a third thing, and every difference it reported would be partly this
    /// module's own arithmetic.
    #[test]
    fn no_substitutions_reproduces_the_frozen_params_exactly() {
        let frozen = params::biosphere();
        let lab = biosphere_with(&[]).expect("no substitutions");
        assert_eq!(
            format!("{frozen:?}"),
            format!("{lab:?}"),
            "the empty substitution set is not the frozen tree"
        );
        // Bit equality on the value this module was built to move, not just Debug equality.
        assert_eq!(
            lab.canopy.extinction_coef.to_bits(),
            frozen.canopy.extinction_coef.to_bits()
        );
    }

    #[test]
    fn a_substitution_moves_exactly_the_field_it_names() {
        let frozen = params::biosphere();
        let lab = biosphere_with(&[Substitution::new("canopy.yaml", "extinction_coef", 0.65)])
            .expect("substitution");
        assert_eq!(lab.canopy.extinction_coef, 0.65);
        assert_ne!(frozen.canopy.extinction_coef, 0.65, "the premise is gone");
        // Its file-mates and every other file are untouched.
        assert_eq!(
            lab.canopy.sla_per_mol_c.to_bits(),
            frozen.canopy.sla_per_mol_c.to_bits()
        );
        assert_eq!(format!("{:?}", lab.photo), format!("{:?}", frozen.photo));
        assert_eq!(format!("{:?}", lab.nitro), format!("{:?}", frozen.nitro));
    }

    /// One file, three loaders — a substitution in `phenology.yaml` must reach whichever of
    /// the three owns the field, and leave the other two alone.
    #[test]
    fn a_shared_file_reaches_all_three_of_its_loaders() {
        let frozen = params::biosphere();
        let lab = biosphere_with(&[Substitution::new("phenology.yaml", "vsen", 0.05)])
            .expect("substitution");
        assert_eq!(lab.vern.vsen, 0.05);
        assert_ne!(frozen.vern.vsen, 0.05, "the premise is gone");
        assert_eq!(format!("{:?}", lab.pheno), format!("{:?}", frozen.pheno));
        assert_eq!(
            format!("{:?}", lab.photoperiod),
            format!("{:?}", frozen.photoperiod)
        );
    }

    #[test]
    fn resolving_by_field_finds_the_owner() {
        let s = Substitution::resolve("extinction_coef", 0.65).expect("unique");
        assert_eq!(s, Substitution::new("canopy.yaml", "extinction_coef", 0.65));
    }

    /// ⚠ The live ambiguity, not a synthetic one: `carbon_fraction` is declared by both
    /// `canopy.yaml` and `nitrogen.yaml`, which fold it differently.
    #[test]
    fn the_shared_key_is_refused_by_name() {
        let owners = owners_of("carbon_fraction").expect("scan");
        assert!(
            owners.len() > 1,
            "the premise is gone — carbon_fraction now has {owners:?}"
        );
        assert!(Substitution::resolve("carbon_fraction", 0.45).is_err());
        // Addressed by file it is accepted, so the refusal is about ambiguity alone.
        assert!(biosphere_with(&[Substitution::new(
            "canopy.yaml",
            "carbon_fraction",
            0.44
        )])
        .is_ok());
    }

    #[test]
    fn an_unknown_field_or_file_is_loud() {
        assert!(Substitution::resolve("no_such_param", 1.0).is_err());
        assert!(biosphere_with(&[Substitution::new("canopy.yaml", "no_such_param", 1.0)]).is_err());
        assert!(biosphere_with(&[Substitution::new("no_such.yaml", "extinction_coef", 1.0)])
            .is_err());
        // `demo.yaml` is excluded from the census BY NAME — it must not be addressable.
        assert!(biosphere_with(&[Substitution::new("demo.yaml", "a_rate", 1.0)]).is_err());
    }

    /// A repeated field would leave one of the two values silently unused — the shape of
    /// finding the harness exists to make impossible.
    #[test]
    fn substituting_one_field_twice_is_refused() {
        let subs = [
            Substitution::new("canopy.yaml", "extinction_coef", 0.65),
            Substitution::new("canopy.yaml", "extinction_coef", 0.68),
        ];
        assert!(biosphere_with(&subs).is_err());
        // Two DIFFERENT fields of the same file are fine, and both must land.
        let both = biosphere_with(&[
            Substitution::new("canopy.yaml", "extinction_coef", 0.65),
            Substitution::new("canopy.yaml", "specific_leaf_area", 22.0),
        ])
        .expect("two fields");
        assert_eq!(both.canopy.extinction_coef, 0.65);
        assert_ne!(
            both.canopy.sla_per_mol_c.to_bits(),
            params::biosphere().canopy.sla_per_mol_c.to_bits()
        );
    }
}
