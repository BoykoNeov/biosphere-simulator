//! Phase-8 (P8.8) **objectives** — the stability/failure goals a player pursues, expressed
//! as pure predicates over the session diagnostics the front-end already exposes.
//!
//! The roadmap's Phase-8 exit criterion asks that a player "observe failure or stability":
//! *survive N years, keep every quantity conserved under a perturbation, avoid `rationed >
//! 0`*. Those map **directly** onto the session's existing readouts — the integer step count
//! `n`, the cumulative `rationed` count, the extinction `events`, and the last-step
//! conservation residual — so an objective is nothing more than a boolean fold over them.
//!
//! **Zero domain logic, zero parity concern** (the display/inspection-split precedent). This
//! computes no science: it reads diagnostics the frozen engine produced and reports whether a
//! goal is met. It is *not* a goal-tracking DSL or a scheduler — a single [`Objective`] is a
//! target horizon plus a conservation tolerance, and [`evaluate`] is a pure function of the
//! four diagnostics. The interesting dynamics come from the perturbations (Step 5): a deep
//! brownout drives `rationed > 0`, flipping [`ObjectiveReport::no_rationing`] to `false` — so
//! the same objective distinguishes a stable run from a failing one, which is the whole point.

/// A player objective: reach `target_step` while staying healthy (conserving, un-rationed,
/// no extinctions). The one tunable is the conservation tolerance — the engine already
/// *asserts* conservation every step, so a run that is still going conserves by construction;
/// this bound is the display-level "is the ledger balancing to round-off" band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Objective {
    /// The horizon to survive to (steps single-rate; master days two-rate).
    pub target_step: u64,
    /// The largest per-quantity conservation residual still considered "conserved". The
    /// engine gates at its own (tight) tolerance every step; a healthy run reads ~1e-15, so
    /// this is a generous health band, not the safety gate.
    pub residual_tolerance: f64,
}

impl Objective {
    /// A "survive to `target_step`" objective with the default health tolerance (`1e-6` —
    /// far above the ~1e-15 round-off a healthy run reads, far below any real imbalance).
    pub fn survive(target_step: u64) -> Self {
        Objective {
            target_step,
            residual_tolerance: 1e-6,
        }
    }
}

/// The evaluation of an [`Objective`] against a session's current diagnostics — each clause
/// plus the composite `survived`. Plain data; serialized for the UI by [`ObjectiveReport::to_json`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectiveReport {
    /// The current step / master-day count.
    pub n: u64,
    /// The objective's horizon.
    pub target_step: u64,
    /// `n >= target_step` — the horizon has been reached.
    pub reached_target: bool,
    /// The last-step max residual (`None` before the first step).
    pub max_residual: Option<f64>,
    /// `max_residual` within tolerance (or no step taken yet — nothing has moved to violate).
    pub conserved: bool,
    /// Cumulative flows scaled by the Euler backstop.
    pub rationed: u64,
    /// `rationed == 0` — no brownout / no supply rationing.
    pub no_rationing: bool,
    /// Extinction events emitted.
    pub events: usize,
    /// `events == 0` — no population collapsed.
    pub no_extinction: bool,
    /// The composite goal: reached the horizon **and** stayed healthy the whole way
    /// (conserving, un-rationed, no extinctions). This is the player's win condition; a
    /// perturbation that flips any clause flips this.
    pub survived: bool,
}

/// Evaluate `objective` against the four session diagnostics — a pure fold, no domain logic.
/// The caller reads the diagnostics off the [`SimSession`](crate::session::SimSession)
/// (`n()`, `total_rationed()`, `events().len()`, `max_residual()`) and hands them here, so
/// this stays trivially testable without building a session.
pub fn evaluate(
    objective: &Objective,
    n: u64,
    rationed: u64,
    events: usize,
    max_residual: Option<f64>,
) -> ObjectiveReport {
    let reached_target = n >= objective.target_step;
    // Before the first step there is no residual reading; nothing has moved, so "conserved"
    // holds vacuously. Otherwise it must be within the health band.
    let conserved = match max_residual {
        None => true,
        Some(r) => r.abs() <= objective.residual_tolerance,
    };
    let no_rationing = rationed == 0;
    let no_extinction = events == 0;
    ObjectiveReport {
        n,
        target_step: objective.target_step,
        reached_target,
        max_residual,
        conserved,
        rationed,
        no_rationing,
        events,
        no_extinction,
        survived: reached_target && conserved && no_rationing && no_extinction,
    }
}

impl ObjectiveReport {
    /// Serialize to plain JSON for the Godot HUD (`JSON.parse_string`) — booleans + the raw
    /// diagnostics behind each. Zero-parity, so plain formatting (like [`crate::display`]).
    pub fn to_json(&self) -> String {
        let residual = match self.max_residual {
            Some(r) if r.is_finite() => format!("{r}"),
            _ => "null".to_string(),
        };
        format!(
            "{{\"n\":{},\"target_step\":{},\"reached_target\":{},\"max_residual\":{},\
             \"conserved\":{},\"rationed\":{},\"no_rationing\":{},\"events\":{},\
             \"no_extinction\":{},\"survived\":{}}}",
            self.n,
            self.target_step,
            self.reached_target,
            residual,
            self.conserved,
            self.rationed,
            self.no_rationing,
            self.events,
            self.no_extinction,
            self.survived,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_run_that_reaches_the_horizon_survives() {
        let obj = Objective::survive(900);
        let r = evaluate(&obj, 900, 0, 0, Some(1e-15));
        assert!(r.reached_target && r.conserved && r.no_rationing && r.no_extinction);
        assert!(r.survived);
    }

    #[test]
    fn short_of_the_horizon_has_not_survived_yet() {
        let r = evaluate(&Objective::survive(900), 300, 0, 0, Some(1e-15));
        assert!(!r.reached_target);
        assert!(!r.survived, "not yet at the horizon");
        // ...but the health clauses still hold.
        assert!(r.conserved && r.no_rationing && r.no_extinction);
    }

    #[test]
    fn rationing_flips_survived_even_at_the_horizon() {
        // The perturbation case: reached the horizon but a brownout rationed → failed.
        let r = evaluate(&Objective::survive(100), 120, 5, 0, Some(1e-15));
        assert!(r.reached_target);
        assert!(!r.no_rationing);
        assert!(!r.survived, "a rationed run is a failure even if it reached the horizon");
    }

    #[test]
    fn extinction_flips_survived() {
        let r = evaluate(&Objective::survive(10), 10, 0, 2, Some(1e-15));
        assert!(!r.no_extinction);
        assert!(!r.survived);
    }

    #[test]
    fn residual_beyond_tolerance_is_not_conserved() {
        let obj = Objective {
            target_step: 10,
            residual_tolerance: 1e-9,
        };
        let r = evaluate(&obj, 10, 0, 0, Some(1e-3));
        assert!(!r.conserved);
        assert!(!r.survived);
    }

    #[test]
    fn no_step_yet_conserves_vacuously() {
        let r = evaluate(&Objective::survive(10), 0, 0, 0, None);
        assert!(r.conserved, "nothing has moved to violate conservation");
        assert!(!r.reached_target);
    }

    #[test]
    fn json_carries_every_clause() {
        let json = evaluate(&Objective::survive(900), 900, 0, 0, Some(1e-15)).to_json();
        for key in [
            "\"n\":900",
            "\"target_step\":900",
            "\"reached_target\":true",
            "\"conserved\":true",
            "\"no_rationing\":true",
            "\"no_extinction\":true",
            "\"survived\":true",
        ] {
            assert!(json.contains(key), "missing {key} in {json}");
        }
        // A None residual serializes as JSON null.
        let none = evaluate(&Objective::survive(1), 0, 0, 0, None).to_json();
        assert!(none.contains("\"max_residual\":null"));
    }
}
