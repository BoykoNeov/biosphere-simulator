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
//! substitution outside a loader's range check **panics** on [`biosphere_with`], exactly as a
//! committed one would. Those checks are not frozen science ranges — they reject values that
//! are *impossible or degenerate*: a zero or negative rate, a fraction outside `[0, 1]`, a
//! cardinal band that divides by zero. On the default route that is the guard catching a typo
//! (`-0.65` for `0.65`).
//!
//! **To ask the question on purpose, use [`biosphere_what_if`]** (since 2026-09-29, the user's
//! WHAT-IF rule: `docs/param-file-conventions.md`). It skips the range checks and nothing else
//! — schema, exact unit strings, the one-line rewrite and its bit re-read all still run — and
//! it refuses a substitution whose folded params come out non-finite. Its report column is
//! labelled `WHAT-IF` by the code, not by convention ([`report::Change::WhatIf`]).
//!
//! # ⚠ This module takes no decision and endorses no value
//!
//! It regenerates evidence. The `extinction_coef` question it was built for
//! (`docs/log/canopy-provenance.md`) is still open and still the user's.

use crate::biosphere::params::{self, BiosphereParams, Bounds};
use crate::biosphere::science::{KineticsForm, LeafAreaForm, O2Form};
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

/// The frozen params under an alternative **oxygen** form — [`biosphere_with_form`]'s sibling.
///
/// # ⚠ This one is only HALF a switch, and the other half is not here
///
/// The temperature form is complete on the params object: both branches are functions of air
/// temperature, which every scenario already has. The oxygen form is not. Its VALUE is a
/// **stock**, so setting this field selects the form and supplies nothing —
/// `CarbonContext::o2_pool_var` reads the amount per step, and a scenario with no O₂ pool
/// falls through to the frozen constant.
///
/// The consequence for reading a report: an `open_season` row under [`O2Form::LivePool`] is
/// **unchanged by construction**, not measured to be small. See
/// `docs/plans/post-roadmap-o2-form.md` §4c.
///
/// # ⚠ This endorses no form
///
/// [`O2Form::Constant`] is the reference and stays the reference.
pub fn biosphere_with_o2_form(
    subs: &[Substitution],
    form: O2Form,
) -> Result<BiosphereParams, ConfigError> {
    let mut p = biosphere_with(subs)?;
    p.photo.o2_form = form;
    Ok(p)
}

/// The frozen params under an alternative **leaf-area** form — the third form sibling.
///
/// [`LeafAreaForm::NodeEnvelope`] makes leaf area a state (the parked leaf mechanism,
/// re-implemented for re-measurement: `docs/plans/post-roadmap-leaf-rust-remeasure.md`). Unlike
/// the two siblings above it changes the build's SHAPE — it adds an aux process and an aux key
/// — so a perennial run of these params must re-sow through `annual_reset_with`; the plain
/// `annual_reset` refuses the state rather than skip the reset.
///
/// # ⚠ This endorses no form
///
/// [`LeafAreaForm::Derived`] is the reference and stays the reference.
pub fn biosphere_with_leaf_form(
    subs: &[Substitution],
    form: LeafAreaForm,
) -> Result<BiosphereParams, ConfigError> {
    let mut p = biosphere_with(subs)?;
    p.canopy.leaf_form = form;
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
    build(subs, Bounds::Enforce)
}

/// [`biosphere_with`] as a **WHAT-IF**: the loaders' range checks are skipped, so a zero rate,
/// a fraction of 0 or 1, or an unordered band can be asked about on purpose.
///
/// Everything else [`biosphere_with`] refuses is still refused (an unknown file or field, a
/// table-shaped field, a doubled substitution, a unit mismatch). And one thing is refused that
/// the default route never needs to: **non-finite folded params**. A skipped range check is
/// exactly how an infinity gets in — `carbon_fraction = 0` divides in two folds — so a what-if
/// that cannot be computed is an error here, not a quiet column.
///
/// ⚠ **Corrected 2026-09-29.** This said the refusal was needed because the engine's
/// conservation check compares `residual > tol`, which a NaN residual passes silently. The
/// comparison is blind, but the engine never reaches it with a NaN: legs and stock amounts are
/// refused when non-finite first (`tests/non_finite_refusal.rs`). Measured with this refusal
/// bypassed, `carbon_fraction = 0` left every stock finite and put the infinity in a REPORT
/// fold — peak LAI printed as `inf`. The refusal stands for that reason: an infinite param
/// cannot be run as a question, and the report now marks a non-finite fold dead too.
///
/// ⚠ A run can still go non-finite later, from params that are finite but degenerate;
/// [`report::measure_composed`] marks such a run dead rather than printing its numbers.
pub fn biosphere_what_if(subs: &[Substitution]) -> Result<BiosphereParams, ConfigError> {
    let p = build(subs, Bounds::WhatIf)?;
    if let Some(field) = first_non_finite(&format!("{p:?}")) {
        return Err(ConfigError::new(format!(
            "the what-if {subs:?} folds to a non-finite value ({field}) — the question cannot \
             be computed, so it is refused rather than run"
        )));
    }
    Ok(p)
}

/// The first `field: inf|-inf|NaN` in a `Debug` rendering, if any.
///
/// ⚠ Read off `Debug` rather than a field list so a param added tomorrow is covered without
/// anyone remembering to list it. Matched on whole value TOKENS after `: `, because a bare
/// substring search for `inf` would fire on any field whose name contains it.
fn first_non_finite(debug: &str) -> Option<String> {
    debug.split([',', '{', '}', '(', ')', '[', ']']).find_map(|part| {
        let (field, value) = part.split_once(": ")?;
        matches!(value.trim(), "inf" | "-inf" | "NaN").then(|| field.trim().to_string())
    })
}

/// The one assembly body behind both routes.
fn build(subs: &[Substitution], bounds: Bounds) -> Result<BiosphereParams, ConfigError> {
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
    let pheno = params::phenology_from_bounded(t, n, bounds);
    let vern = params::vernalization_from_bounded(t, n, bounds);
    let photoperiod = params::photoperiod_from_bounded(t, n, bounds);

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
        canopy: params::canopy_from_bounded(ct, cn, bounds),
        photo: params::photosynthesis_from_bounded(pt, pn, bounds),
        resp: params::respiration_from_bounded(rt, rn, bounds),
        transp: params::transpiration_from_bounded(tt, tn, bounds),
        pheno,
        vern,
        photoperiod,
        senesc: params::senescence_from_bounded(st, sn, bounds),
        stem_reserve: params::stem_reserves_from_bounded(srt, srn, bounds),
        rootd: params::root_depth_from_bounded(rdt, rdn, bounds),
        nitro: params::nitrogen_from_bounded(nt, nn, bounds),
        decomp: params::decomposition_from_bounded(dt, dn, bounds),
        micro: params::microbial_respiration_from_bounded(mt, mn, bounds),
        humi: params::humification_from_bounded(ht, hn, bounds),
        water: params::water_cycle_from_bounded(wt, wn, bounds),
        herb: params::herbivory_from_bounded(hbt, hbn, bounds),
        // ⚠ No bounded twin: a substitution cannot address the partition TABLE
        // (`with_override` refuses a table-shaped field), so there is no what-if to pass
        // through. `lab::partition` is that table's own instrument.
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

    /// The control for the WHAT-IF route: with nothing substituted it is the frozen tree,
    /// so a what-if column's differences are the what-if's and not the route's.
    #[test]
    fn an_empty_what_if_is_the_frozen_params_exactly() {
        let frozen = params::biosphere();
        let lab = biosphere_what_if(&[]).expect("no substitutions");
        assert_eq!(format!("{frozen:?}"), format!("{lab:?}"));
    }

    /// ⚠⚠ **The pair this route exists for, asserted on the SAME value both ways.** A zero
    /// root-extension rate ("what if roots never grow?") is refused by the guarded route — a
    /// positivity check — and accepted by the what-if route, where it must LAND, not merely
    /// return `Ok`. Delete the `Bounds::Enforce` branch and the first half goes red; route the
    /// what-if through `Enforce` and the second does.
    #[test]
    fn a_range_check_refuses_by_default_and_yields_to_a_what_if() {
        let zero_rate = [Substitution::new("root_depth.yaml", "max_extension_rate", 0.0)];
        let guarded = std::panic::catch_unwind(|| biosphere_with(&zero_rate));
        assert!(guarded.is_err(), "the default route accepted a zero rate");

        let p = biosphere_what_if(&zero_rate).expect("the what-if route accepts it");
        assert_eq!(p.rootd.max_extension_rate.to_bits(), 0.0_f64.to_bits());
        // Its file-mate is untouched.
        assert_eq!(
            p.rootd.max_rooted_depth.to_bits(),
            params::biosphere().rootd.max_rooted_depth.to_bits()
        );
    }

    /// The same, for a raw ORDERING rule rather than a `require_*` helper — the two are
    /// routed separately (`Range::ensure` vs `Range::check`), so each needs its own witness.
    #[test]
    fn an_ordering_rule_refuses_by_default_and_yields_to_a_what_if() {
        let whole_stem = [Substitution::new(
            "stem_reserves.yaml",
            "remobilizable_fraction",
            1.0,
        )];
        assert!(std::panic::catch_unwind(|| biosphere_with(&whole_stem)).is_err());
        let p = biosphere_what_if(&whole_stem).expect("the what-if route accepts it");
        assert_eq!(p.stem_reserve.remobilizable_fraction, 1.0);
    }

    /// What the what-if route still refuses: a question it cannot compute. `carbon_fraction
    /// = 0` passes the skipped range check and divides in the canopy fold.
    #[test]
    fn a_what_if_that_folds_to_infinity_is_refused() {
        let e = biosphere_what_if(&[Substitution::new("canopy.yaml", "carbon_fraction", 0.0)])
            .expect_err("an infinite fold was run");
        assert!(e.to_string().contains("sla_per_mol_c"), "{e}");
        // The typo guards are not what-if territory: an unknown field is refused as before.
        assert!(biosphere_what_if(&[Substitution::new("canopy.yaml", "no_such", 1.0)]).is_err());
    }

    /// Behind that refusal, the report's own guard: an infinite param that reaches a run puts
    /// the infinity in a FOLD, not in a stock, and the fold is marked dead rather than printed.
    /// Reached here only by bypassing the refusal above — which is the point: it is the second
    /// line, and before 2026-09-29 it printed peak LAI as `inf`.
    #[test]
    fn an_infinite_param_that_reaches_a_run_is_reported_dead_not_printed() {
        let subs = [Substitution::new("canopy.yaml", "carbon_fraction", 0.0)];
        let p = build(&subs, Bounds::WhatIf).expect("the unguarded build");
        let col = report::measure_composed("inf", &p, false, None).expect("measured");
        assert!(
            col.values.iter().all(|(_, v)| v.is_finite()),
            "a non-finite readout was printed: {:?}",
            col.values
        );
        let dead: Vec<&str> = col.failed.iter().map(|(_, why)| why.as_str()).collect();
        assert!(
            dead.iter().any(|w| w.starts_with("open_season") && w.contains("non-finite")),
            "open_season's infinite peak LAI was not marked dead: {dead:?}"
        );
    }

    #[test]
    fn the_non_finite_scan_reads_values_not_names() {
        assert_eq!(first_non_finite("P { infiltration: 1.0, info: 2.0 }"), None);
        assert_eq!(
            first_non_finite("P { infiltration: 1.0, rate: inf }"),
            Some("rate".to_string())
        );
        assert_eq!(first_non_finite("Q { a: -inf }"), Some("a".to_string()));
        assert_eq!(
            first_non_finite("R { x: S { y: NaN } }"),
            Some("y".to_string())
        );
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
