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
use crate::biosphere::science::KineticsForm;
use config::{with_override, ConfigError, ParamFile};

/// The comparison report — §6 of the plan, every requirement earned by a wrong read.
pub mod report;

/// The **science** half: a season with a named flow removed from the assembled registry.
pub mod mechanism;

/// The **partition table**: the one frozen param [`Substitution`] cannot address, because
/// `with_override` refuses a table-shaped field. Perturbed by re-emitting the rows.
pub mod partition;

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

/// The frozen params read under a different **temperature form** — the science half's
/// second instrument, and the first alternative *form* of any biosphere process in this tree.
///
/// Plan: `docs/plans/post-roadmap-temperature-kinetics.md`. Where [`Substitution`] changes a
/// number and [`mechanism`] changes which flows are assembled, this changes the rate law the
/// assembled flows evaluate — over the **frozen numbers**, so an A/B attributes to the form
/// and not to a moved value.
///
/// # ⚠ Why this is one field on the params object and not three flow replacements
///
/// `Allocation`, `GrowthRespiration` and `MaintenanceRespiration` each hold a
/// `CarbonContext` and each calls `budget()`. Replacing one leaves a step whose growth
/// respiration is computed off the frozen assimilation and whose allocation is not —
/// internally inconsistent, and it would look entirely plausible in a report. Replacing all
/// three means rebuilding their contexts, i.e. a **second assembly body**, which
/// [`mechanism`]'s header names as the defect that module exists to prevent. The form rides
/// the one funnel `tests/param_funnel.rs` gates instead, and all three follow.
///
/// # ⚠ This endorses no form
///
/// [`KineticsForm::Cardinal`] is the reference and stays the reference. A column measured
/// under [`KineticsForm::Q10Teh`] is evidence about a cited alternative, not a proposal.
pub fn biosphere_with_form(
    subs: &[Substitution],
    form: KineticsForm,
) -> Result<BiosphereParams, ConfigError> {
    let mut p = biosphere_with(subs)?;
    p.photo.kinetics = form;
    Ok(p)
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

/// The value-switch command's spec grammar, parsed into the columns it asks for.
///
/// ```text
/// [file.yaml:]field=v1[,v2,...]              one column PER value, one substitution each
/// [file.yaml:]field=v + [file.yaml:]field=v  ONE column, several substitutions at once
/// ```
///
/// # ⚠ Why the `+` form exists, stated rather than left to be inferred
///
/// A sweep answers *"how sensitive is the tree to this one number?"*. It cannot answer
/// *"what would this FORM do?"* whenever a form moves two numbers together — and the
/// physically coupled case is exactly that: O₂ enters the Rubisco denominator *and* sets
/// `Γ* = 0.5·O/(S_c/o)`, so a column varying one of them is a counterfactual no atmosphere
/// produces. With single-substitution columns only, the combined effect can be *argued*
/// across two columns but never *measured*, which is the difference between evidence and
/// arithmetic done by the reader. [`report::compare`] always accepted a multi-substitution
/// variant; only this grammar could not spell one (`docs/log/o2-coupling-measured.md`).
///
/// # What is refused rather than guessed
///
/// * **`,` and `+` in one spec.** `a=1,2+b=3` could mean two coupled columns or three
///   independent ones; a harness that picks one produces a table whose caption is wrong in
///   a way no reader can see. Same discipline as [`Substitution::resolve`]'s ambiguity.
/// * **An empty or malformed part**, so `a=1+` is an error, not a silent one-substitution
///   column that would read as the coupled measurement and is not one.
/// * **The same target twice in one column** — refused downstream by [`biosphere_with`],
///   where the second value would silently win.
pub fn parse_variants(spec: &str) -> Result<Vec<(String, Vec<Substitution>)>, ConfigError> {
    let combined = spec.contains('+');
    if combined && spec.contains(',') {
        return Err(ConfigError::new(format!(
            "{spec:?} mixes `,` (a sweep of one target) with `+` (one column of several \
             targets) — write them as separate specs, because the combination has two \
             readings and this harness guesses at neither"
        )));
    }
    if !combined {
        let (target, values) = split_once_or_err(spec)?;
        let mut out = Vec::new();
        for raw in values.split(',') {
            let sub = one(target, raw)?;
            out.push((label_of(std::slice::from_ref(&sub)), vec![sub]));
        }
        return Ok(out);
    }
    let mut subs = Vec::new();
    for part in spec.split('+') {
        let (target, value) = split_once_or_err(part.trim())?;
        subs.push(one(target, value)?);
    }
    Ok(vec![(label_of(&subs), subs)])
}

/// `target=value`, or a loud error naming the whole spec rather than the fragment.
fn split_once_or_err(spec: &str) -> Result<(&str, &str), ConfigError> {
    spec.split_once('=').ok_or_else(|| {
        ConfigError::new(format!("{spec:?} is not `[file.yaml:]field=value`"))
    })
}

/// One `[file.yaml:]field` + one number, resolved the same way the sweep form resolves it.
fn one(target: &str, raw: &str) -> Result<Substitution, ConfigError> {
    let value: f64 = raw
        .trim()
        .parse()
        .map_err(|_| ConfigError::new(format!("{:?} is not a number", raw.trim())))?;
    match target.trim().split_once(':') {
        Some((file, field)) => Ok(Substitution::new(file.trim(), field.trim(), value)),
        None => Substitution::resolve(target.trim(), value),
    }
}

/// The column heading. The file prefix is printed once and then only when it CHANGES —
/// `photosynthesis.yaml:o2=2+gamma_star=0.4071` rather than the same 19 bytes twice, which
/// at the report's column width is the difference between a readable table and a wrapped one.
///
/// ⚠ It tracks only the **previous** part, not every file seen, so an interleaved spec
/// (`a.yaml:x=1+b.yaml:y=2+a.yaml:z=3`) re-prints `a.yaml` on the third part and reads at a
/// glance as three files rather than two. Cosmetic, and it never *drops* a prefix that is
/// needed — but a heading is quoted as evidence in the record, so it is not a faithful
/// serialization for three-or-more parts and should not be treated as one.
fn label_of(subs: &[Substitution]) -> String {
    let mut out = String::new();
    let mut last_file: Option<&str> = None;
    for s in subs {
        if !out.is_empty() {
            out.push('+');
        }
        if last_file != Some(s.file.as_str()) {
            out.push_str(&s.file);
            out.push(':');
            last_file = Some(s.file.as_str());
        }
        out.push_str(&format!("{}={}", s.field, s.value));
    }
    out
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

    /// The sweep form: N values of one target become N columns of ONE substitution each.
    #[test]
    fn a_swept_spec_is_one_column_per_value() {
        let v = parse_variants("extinction_coef=0.60,0.65").expect("sweep");
        assert_eq!(v.len(), 2, "a two-value sweep is two columns");
        assert!(v.iter().all(|(_, subs)| subs.len() == 1));
        assert_eq!(v[0].1[0], Substitution::new("canopy.yaml", "extinction_coef", 0.60));
        assert_eq!(v[1].1[0], Substitution::new("canopy.yaml", "extinction_coef", 0.65));
    }

    /// ⚠⚠ **The property the `+` form exists for, and the one a column count cannot see.**
    /// A parser that returned two columns of one substitution here would pass any "did it
    /// parse?" check while producing exactly the two-counterfactual table that could not
    /// measure the coupled form. So this asserts the SHAPE — one column, two substitutions —
    /// not merely that the call succeeded.
    #[test]
    fn a_combined_spec_is_one_column_carrying_every_substitution() {
        let v = parse_variants("o2=2.0+gamma_star=0.4071").expect("combined");
        assert_eq!(v.len(), 1, "a `+` spec is ONE column, not a sweep");
        let (label, subs) = &v[0];
        assert_eq!(subs.len(), 2, "both substitutions must reach the column");
        assert_eq!(subs[0], Substitution::new("photosynthesis.yaml", "o2", 2.0));
        assert_eq!(
            subs[1],
            Substitution::new("photosynthesis.yaml", "gamma_star", 0.4071)
        );
        // The heading names both, with the shared file printed once.
        assert_eq!(label, "photosynthesis.yaml:o2=2+gamma_star=0.4071");
        // And it must actually LAND — the substitutions are live, not just parsed.
        let p = biosphere_with(subs).expect("both substitutions apply");
        assert_eq!(p.photo.o2, 2.0);
        assert_eq!(p.photo.gamma_star, 0.4071);
    }

    /// A combined spec across two files keeps each file's prefix.
    #[test]
    fn a_combined_spec_may_span_files() {
        let v = parse_variants("o2=2.0+canopy.yaml:extinction_coef=0.65").expect("two files");
        let (label, subs) = &v[0];
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[1].file, "canopy.yaml");
        assert_eq!(
            label,
            "photosynthesis.yaml:o2=2+canopy.yaml:extinction_coef=0.65"
        );
    }

    /// ⚠ The ambiguous spec is REFUSED, not resolved to a house reading. `a=1,2+b=3` is two
    /// coupled columns or three independent ones depending on precedence, and a table built
    /// on the wrong one is wrong in a way its caption hides.
    #[test]
    fn mixing_a_sweep_and_a_combination_is_refused() {
        assert!(parse_variants("o2=2.0,0.033+gamma_star=0.4071").is_err());
        // Each half alone is fine, so the refusal is about the MIXTURE and nothing else.
        assert!(parse_variants("o2=2.0,0.033").is_ok());
        assert!(parse_variants("o2=2.0+gamma_star=0.4071").is_ok());
    }

    /// ⚠ **Where the `+` split falls on an exponent, pinned because it is surprising.**
    /// The combined/sweep choice is made by a *global* `contains('+')`, and `+` is also legal
    /// inside an f64 literal — so `1e+5` puts the spec on the combined branch and splits
    /// mid-number. The outcome is still an error rather than a wrong column, which is the
    /// property that matters; what it is NOT is a good error message. Pinned so the next
    /// reader learns this from a test instead of from a confusing failure, and so that
    /// teaching the splitter about exponents is a visible change rather than a silent one.
    #[test]
    fn an_exponent_is_not_a_combination_but_the_split_does_not_know_that() {
        assert!(parse_variants("o2=1e+5").is_err());
        assert!(parse_variants("o2=2.0+gamma_star=1e+5").is_err());
        // Written without the `+`, the same magnitude parses fine — so the refusal is about
        // the SPLITTER, not about the value being rejected somewhere downstream.
        let v = parse_variants("o2=1e5").expect("a bare exponent is a number");
        assert_eq!(v[0].1[0].value, 1e5);
    }

    /// A malformed part is loud. ⚠ `a=1+` must NOT degrade to the one-substitution column:
    /// that column would be read as the coupled measurement and would not be one.
    #[test]
    fn a_malformed_or_unknown_part_is_loud() {
        assert!(parse_variants("o2=2.0+").is_err());
        assert!(parse_variants("o2").is_err());
        assert!(parse_variants("o2=notanumber").is_err());
        assert!(parse_variants("o2=2.0+no_such_param=1.0").is_err());
        // The live ambiguity is still refused through this grammar, not only through resolve.
        assert!(parse_variants("carbon_fraction=0.45").is_err());
    }
}
